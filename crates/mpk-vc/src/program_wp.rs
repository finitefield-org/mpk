//! Unified acyclic VIR weakest-precondition and operation-safety generation.
//!
//! The engine consumes only fully validated VIR. It propagates one bounded
//! symbolic state per block, substitutes block arguments at edges, and merges
//! predecessor states deterministically. Postconditions and safety checks are
//! collected during that same traversal so their path semantics cannot drift.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::expr_encode::{
    MpkExprTerm, STD_BOOL_AND, STD_BOOL_FALSE, STD_BOOL_IF, STD_BOOL_NOT, STD_BOOL_OR,
    STD_BOOL_TRUE,
};
use crate::program_encode::{
    encode_vir_contract_expr, encode_vir_instruction_expr, encode_vir_value, ProgramExprContext,
    ProgramExprEncodeError,
};
use crate::safety_check::{
    classify_safety_evidence, encode_instruction_safety, SafetyCheckError, SafetyEvidenceRoute,
};
use crate::type_encode::MpkTypeTerm;
use crate::vir::{
    VirBinding, VirFunction, VirInstruction, VirModule, VirTerminator, VirUnit, VirValue,
};
use crate::vir_validate::{validate_vir, VirValidationError};

pub const VC_MEMBERS_PER_FUNCTION_MAX: usize = 100_000;
pub const VC_MEMBERS_PER_DOCUMENT_MAX: usize = 262_144;
pub const VC_ASSUMPTIONS_PER_MEMBER_MAX: usize = 4_096;
pub const VC_EXPRESSION_NODES_PER_MEMBER_MAX: usize = 8_192;
pub const VC_EXPRESSION_NODES_PER_DOCUMENT_MAX: usize = 4_194_304;
pub const VC_MEMBER_EXPRESSION_DEPTH_MAX: usize = 256;

pub fn generate_program_vcs(input: &VirModule) -> Result<ProgramVcModule, ProgramWpError> {
    ProgramWpGenerator::new().generate_module(input)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramVcModule {
    pub functions: Vec<ProgramVcFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramVcFunction {
    pub function_id: String,
    pub requires: Vec<MpkExprTerm>,
    pub members: Vec<ProgramVcMember>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProgramVcMemberKind {
    OperationSafety,
    Postcondition,
}

impl ProgramVcMemberKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationSafety => "operation_safety",
            Self::Postcondition => "postcondition",
        }
    }

    const fn group_suffix(self) -> &'static str {
        match self {
            Self::OperationSafety => "panic_free",
            Self::Postcondition => "contract",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramVcMember {
    pub id: String,
    pub function_id: String,
    pub kind: ProgramVcMemberKind,
    pub local_binders: Vec<MpkTypeTerm>,
    pub assumptions: Vec<MpkExprTerm>,
    pub conclusion: MpkExprTerm,
    pub group_id: String,
    pub safety_evidence: Option<SafetyEvidenceRoute>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramWpGenerator {
    limits: ProgramWpLimits,
}

impl Default for ProgramWpGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramWpGenerator {
    pub const fn new() -> Self {
        Self {
            limits: ProgramWpLimits::production(),
        }
    }

    pub fn generate_module(self, input: &VirModule) -> Result<ProgramVcModule, ProgramWpError> {
        validate_vir(input).map_err(ProgramWpError::Validation)?;

        let mut functions = input
            .units
            .iter()
            .flat_map(|unit| unit.functions.iter().map(move |function| (unit, function)))
            .collect::<Vec<_>>();
        // T08 rejects calls below, so every function is ready in the
        // callee-first order and the normative tie-break is the UTF-8 ID.
        functions.sort_by(|(_, lhs), (_, rhs)| lhs.id.as_bytes().cmp(rhs.id.as_bytes()));

        let mut budget = GenerationBudget::new(self.limits);
        let mut output = Vec::with_capacity(functions.len());
        for (unit, function) in functions {
            output.push(self.generate_function(input, unit, function, &mut budget)?);
        }
        Ok(ProgramVcModule { functions: output })
    }

    fn generate_function(
        self,
        module: &VirModule,
        unit: &VirUnit,
        function: &VirFunction,
        budget: &mut GenerationBudget,
    ) -> Result<ProgramVcFunction, ProgramWpError> {
        if function.contracts.loops.is_empty() {
            self.generate_acyclic_function(module, unit, function, budget)
        } else {
            Err(ProgramWpError::UnsupportedLoopCutpoint {
                function_id: function.id.clone(),
            })
        }
    }

    fn generate_acyclic_function(
        self,
        module: &VirModule,
        unit: &VirUnit,
        function: &VirFunction,
        budget: &mut GenerationBudget,
    ) -> Result<ProgramVcFunction, ProgramWpError> {
        let context = ProgramExprContext::for_validated_function(module, unit, function)
            .map_err(|source| expression_error(function, "encoder context", source))?;
        let builder = TermBuilder::new(self.limits);
        let base_bindings = function
            .params
            .iter()
            .chain(function.locals.iter())
            .map(|binding| binding.id.clone())
            .collect::<BTreeSet<_>>();
        let initial_env = function
            .params
            .iter()
            .map(|binding| {
                (
                    binding.id.clone(),
                    MpkExprTerm::Var {
                        name: binding.id.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let empty_results = BTreeMap::new();

        let mut requires = Vec::with_capacity(function.contracts.requires.len());
        for (index, require) in function.contracts.requires.iter().enumerate() {
            let raw = encode_vir_contract_expr(&context, require).map_err(|source| {
                expression_error(function, format!("requires[{index}]"), source)
            })?;
            let ceiling = budget.document_node_ceiling()?;
            let encoded = builder.substitute(&raw, &initial_env, &empty_results, ceiling)?;
            budget.record_document_term(&encoded)?;
            requires.push(encoded);
        }

        let labels = function
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.label.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        let mut remaining_predecessors = vec![0_usize; function.blocks.len()];
        for block in &function.blocks {
            for successor in successors(&block.terminator) {
                let target =
                    labels
                        .get(successor)
                        .copied()
                        .ok_or_else(|| ProgramWpError::InvalidGraph {
                            function_id: function.id.clone(),
                            detail: format!("unknown successor {successor}"),
                        })?;
                remaining_predecessors[target] = remaining_predecessors[target]
                    .checked_add(1)
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "CFG predecessor count".to_owned(),
                    })?;
            }
        }

        let ready = remaining_predecessors
            .iter()
            .enumerate()
            .filter_map(|(index, count)| (*count == 0).then_some(index))
            .collect::<BTreeSet<_>>();
        let mut worklist = DataflowWorklist {
            incoming: vec![Vec::<IncomingState>::new(); function.blocks.len()],
            remaining_predecessors,
            ready,
        };
        let mut processed = 0_usize;
        let mut pending = Vec::new();
        let mut function_member_count = 0_usize;

        while let Some(block_index) = worklist.ready.pop_first() {
            let block = &function.blocks[block_index];
            let mut state = if block_index == 0 {
                if !worklist.incoming[block_index].is_empty() {
                    return Err(ProgramWpError::InvalidGraph {
                        function_id: function.id.clone(),
                        detail: "entry block has an incoming edge".to_owned(),
                    });
                }
                SymbolicState {
                    env: initial_env.clone(),
                    assumptions: Vec::new(),
                }
            } else {
                merge_incoming_states(
                    function,
                    &builder,
                    std::mem::take(&mut worklist.incoming[block_index]),
                )?
            };
            processed =
                processed
                    .checked_add(1)
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "processed block count".to_owned(),
                    })?;

            for (instruction_index, instruction) in block.instructions.iter().enumerate() {
                if matches!(instruction, VirInstruction::CallStatic { .. }) {
                    return Err(ProgramWpError::UnsupportedStaticCall {
                        function_id: function.id.clone(),
                        block_label: block.label.clone(),
                        instruction_id: instruction_id(instruction).to_owned(),
                    });
                }

                let safety =
                    encode_instruction_safety(&context, instruction).map_err(|source| {
                        safety_error(function, instruction_id(instruction), source)
                    })?;
                for (check_index, predicate) in safety.into_iter().enumerate() {
                    let reservation =
                        budget.begin_member(&mut function_member_count, &state.assumptions)?;
                    let proposition = builder.substitute(
                        &predicate.proposition,
                        &state.env,
                        &empty_results,
                        reservation.conclusion_ceiling,
                    )?;
                    budget.finish_member(reservation, &proposition)?;
                    pending.push(PendingMember {
                        block_index,
                        instruction_index,
                        item_index: check_index,
                        kind: ProgramVcMemberKind::OperationSafety,
                        assumptions: state.assumptions.clone(),
                        conclusion: proposition.clone(),
                        safety_evidence: Some(classify_safety_evidence(
                            module.semantic_profile,
                            &proposition,
                        )),
                    });
                }

                let raw = encode_vir_instruction_expr(&context, instruction).map_err(|source| {
                    expression_error(
                        function,
                        format!("instruction {}", instruction_id(instruction)),
                        source,
                    )
                })?;
                let value = builder.substitute(
                    &raw,
                    &state.env,
                    &empty_results,
                    NodeCeiling::member(self.limits.expression_nodes_per_member),
                )?;
                state
                    .env
                    .insert(instruction_id(instruction).to_owned(), value.clone());
                if let VirInstruction::Copy { target, .. } = instruction {
                    state.env.insert(target.clone(), value);
                }
            }

            match &block.terminator {
                VirTerminator::Return { values } => {
                    let mut results = BTreeMap::new();
                    for (index, value) in values.iter().enumerate() {
                        let raw = encode_vir_value(&context, value).map_err(|source| {
                            expression_error(function, format!("return[{index}]"), source)
                        })?;
                        let value = builder.substitute(
                            &raw,
                            &state.env,
                            &empty_results,
                            NodeCeiling::member(self.limits.expression_nodes_per_member),
                        )?;
                        let index =
                            u32::try_from(index).map_err(|_| ProgramWpError::CounterOverflow {
                                context: format!("{} return index", function.id),
                            })?;
                        results.insert(index, value);
                    }
                    for (ensure_index, ensure) in function.contracts.ensures.iter().enumerate() {
                        let reservation =
                            budget.begin_member(&mut function_member_count, &state.assumptions)?;
                        let raw = encode_vir_contract_expr(&context, ensure).map_err(|source| {
                            expression_error(
                                function,
                                format!("bb{block_index}.ensures[{ensure_index}]"),
                                source,
                            )
                        })?;
                        let conclusion = builder.substitute(
                            &raw,
                            &state.env,
                            &results,
                            reservation.conclusion_ceiling,
                        )?;
                        budget.finish_member(reservation, &conclusion)?;
                        pending.push(PendingMember {
                            block_index,
                            instruction_index: usize::MAX,
                            item_index: ensure_index,
                            kind: ProgramVcMemberKind::Postcondition,
                            assumptions: state.assumptions.clone(),
                            conclusion,
                            safety_evidence: None,
                        });
                    }
                }
                VirTerminator::Jump { label, args } => {
                    let target_index = labels[label.as_str()];
                    let edge_state = edge_state(
                        function,
                        &context,
                        &builder,
                        &base_bindings,
                        &state,
                        &function.blocks[target_index].parameters,
                        args,
                    )?;
                    push_incoming(
                        function,
                        block_index,
                        0,
                        target_index,
                        edge_state,
                        &mut worklist,
                    )?;
                }
                VirTerminator::Branch {
                    cond,
                    then_label,
                    then_args,
                    else_label,
                    else_args,
                } => {
                    let raw = encode_vir_value(&context, cond).map_err(|source| {
                        expression_error(
                            function,
                            format!("{} branch condition", block.label),
                            source,
                        )
                    })?;
                    let condition = builder.substitute(
                        &raw,
                        &state.env,
                        &empty_results,
                        NodeCeiling::member(self.limits.expression_nodes_per_member),
                    )?;
                    let negative = builder.negate(condition.clone())?;

                    // False before true is the canonical VIR successor order.
                    for (edge_index, label, args, guard) in [
                        (0_usize, else_label, else_args, negative),
                        (1_usize, then_label, then_args, condition),
                    ] {
                        let target_index = labels[label.as_str()];
                        let mut branch_state = state.clone();
                        push_assumption(&mut branch_state.assumptions, guard, self.limits)?;
                        let edge_state = edge_state(
                            function,
                            &context,
                            &builder,
                            &base_bindings,
                            &branch_state,
                            &function.blocks[target_index].parameters,
                            args,
                        )?;
                        push_incoming(
                            function,
                            block_index,
                            edge_index,
                            target_index,
                            edge_state,
                            &mut worklist,
                        )?;
                    }
                }
            }
        }

        if processed != function.blocks.len() {
            return Err(ProgramWpError::UnsupportedLoopCutpoint {
                function_id: function.id.clone(),
            });
        }

        Ok(ProgramVcFunction {
            function_id: function.id.clone(),
            requires,
            members: finalize_members(function, pending)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProgramWpLimits {
    members_per_function: usize,
    members_per_document: usize,
    assumptions_per_member: usize,
    expression_nodes_per_member: usize,
    expression_nodes_per_document: usize,
    member_expression_depth: usize,
}

impl ProgramWpLimits {
    const fn production() -> Self {
        Self {
            members_per_function: VC_MEMBERS_PER_FUNCTION_MAX,
            members_per_document: VC_MEMBERS_PER_DOCUMENT_MAX,
            assumptions_per_member: VC_ASSUMPTIONS_PER_MEMBER_MAX,
            expression_nodes_per_member: VC_EXPRESSION_NODES_PER_MEMBER_MAX,
            expression_nodes_per_document: VC_EXPRESSION_NODES_PER_DOCUMENT_MAX,
            member_expression_depth: VC_MEMBER_EXPRESSION_DEPTH_MAX,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SymbolicState {
    env: BTreeMap<String, MpkExprTerm>,
    assumptions: Vec<MpkExprTerm>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IncomingState {
    predecessor_index: usize,
    edge_index: usize,
    state: SymbolicState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DataflowWorklist {
    incoming: Vec<Vec<IncomingState>>,
    remaining_predecessors: Vec<usize>,
    ready: BTreeSet<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingMember {
    block_index: usize,
    instruction_index: usize,
    item_index: usize,
    kind: ProgramVcMemberKind,
    assumptions: Vec<MpkExprTerm>,
    conclusion: MpkExprTerm,
    safety_evidence: Option<SafetyEvidenceRoute>,
}

fn finalize_members(
    function: &VirFunction,
    mut pending: Vec<PendingMember>,
) -> Result<Vec<ProgramVcMember>, ProgramWpError> {
    pending.sort_by_key(|member| {
        (
            member.kind,
            member.block_index,
            member.instruction_index,
            member.item_index,
        )
    });
    let mut ordinals = BTreeMap::<ProgramVcMemberKind, usize>::new();
    let mut members = Vec::with_capacity(pending.len());
    for member in pending {
        let ordinal = ordinals.entry(member.kind).or_default();
        if *ordinal > 999_999 {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_MEMBERS_PER_FUNCTION",
                limit: 1_000_000,
                actual: ordinal.saturating_add(1),
            });
        }
        let kind = member.kind.as_str();
        members.push(ProgramVcMember {
            id: format!("{}#{kind}#{:06}", function.id, *ordinal),
            function_id: function.id.clone(),
            kind: member.kind,
            local_binders: Vec::new(),
            assumptions: member.assumptions,
            conclusion: member.conclusion,
            group_id: format!("{}.{}", function.id, member.kind.group_suffix()),
            safety_evidence: member.safety_evidence,
        });
        *ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: format!("{} {kind} ordinal", function.id),
            })?;
    }
    members.sort_by(|lhs, rhs| lhs.id.as_bytes().cmp(rhs.id.as_bytes()));
    Ok(members)
}

#[allow(clippy::too_many_arguments)]
fn edge_state(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    base_bindings: &BTreeSet<String>,
    state: &SymbolicState,
    target_parameters: &[VirBinding],
    args: &[VirValue],
) -> Result<SymbolicState, ProgramWpError> {
    let mut env = state
        .env
        .iter()
        .filter(|(name, _)| base_bindings.contains(*name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    for (index, (parameter, argument)) in target_parameters.iter().zip(args).enumerate() {
        let raw = encode_vir_value(context, argument).map_err(|source| {
            expression_error(function, format!("edge argument[{index}]"), source)
        })?;
        let value = builder.substitute(
            &raw,
            &state.env,
            &BTreeMap::new(),
            NodeCeiling::member(builder.limits.expression_nodes_per_member),
        )?;
        env.insert(parameter.id.clone(), value);
    }
    Ok(SymbolicState {
        env,
        assumptions: state.assumptions.clone(),
    })
}

fn push_incoming(
    function: &VirFunction,
    predecessor_index: usize,
    edge_index: usize,
    target_index: usize,
    state: SymbolicState,
    worklist: &mut DataflowWorklist,
) -> Result<(), ProgramWpError> {
    worklist.incoming[target_index].push(IncomingState {
        predecessor_index,
        edge_index,
        state,
    });
    worklist.remaining_predecessors[target_index] = worklist.remaining_predecessors[target_index]
        .checked_sub(1)
        .ok_or_else(|| ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: format!(
                "block {} received more edges than declared",
                function.blocks[target_index].label
            ),
        })?;
    if worklist.remaining_predecessors[target_index] == 0 {
        worklist.ready.insert(target_index);
    }
    Ok(())
}

fn merge_incoming_states(
    function: &VirFunction,
    builder: &TermBuilder,
    mut incoming: Vec<IncomingState>,
) -> Result<SymbolicState, ProgramWpError> {
    if incoming.is_empty() {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "reachable non-entry block has no predecessor state".to_owned(),
        });
    }
    incoming.sort_by_key(|edge| (edge.predecessor_index, edge.edge_index));
    if incoming.len() == 1 {
        return incoming
            .pop()
            .map(|edge| edge.state)
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: "incoming state disappeared".to_owned(),
            });
    }

    let common_length = longest_common_assumption_prefix(&incoming);
    let selectors = incoming
        .iter()
        .map(|edge| builder.conjoin(&edge.state.assumptions[common_length..]))
        .collect::<Result<Vec<_>, _>>()?;
    let mut assumptions = incoming[0].state.assumptions[..common_length].to_vec();
    let reach = builder.disjoin(&selectors)?;
    if !is_true(&reach) {
        push_assumption(&mut assumptions, reach, builder.limits)?;
    }

    let keys = incoming[0]
        .state
        .env
        .keys()
        .filter(|key| {
            incoming[1..]
                .iter()
                .all(|edge| edge.state.env.contains_key(*key))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut env = BTreeMap::new();
    for key in keys {
        let values = incoming
            .iter()
            .map(|edge| edge.state.env[&key].clone())
            .collect::<Vec<_>>();
        env.insert(key, builder.select(&selectors, &values)?);
    }
    Ok(SymbolicState { env, assumptions })
}

fn longest_common_assumption_prefix(incoming: &[IncomingState]) -> usize {
    let maximum = incoming
        .iter()
        .map(|edge| edge.state.assumptions.len())
        .min()
        .unwrap_or(0);
    (0..maximum)
        .take_while(|index| {
            let expected = &incoming[0].state.assumptions[*index];
            incoming[1..]
                .iter()
                .all(|edge| edge.state.assumptions[*index] == *expected)
        })
        .count()
}

fn push_assumption(
    assumptions: &mut Vec<MpkExprTerm>,
    assumption: MpkExprTerm,
    limits: ProgramWpLimits,
) -> Result<(), ProgramWpError> {
    let actual =
        assumptions
            .len()
            .checked_add(1)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "path assumption count".to_owned(),
            })?;
    if actual > limits.assumptions_per_member {
        return Err(ProgramWpError::Limit {
            code: "VC_LIMIT_ASSUMPTIONS_PER_MEMBER",
            limit: limits.assumptions_per_member,
            actual,
        });
    }
    assumptions.push(assumption);
    Ok(())
}

fn successors(terminator: &VirTerminator) -> Vec<&str> {
    match terminator {
        VirTerminator::Return { .. } => Vec::new(),
        VirTerminator::Jump { label, .. } => vec![label],
        VirTerminator::Branch {
            then_label,
            else_label,
            ..
        } => vec![else_label, then_label],
    }
}

fn instruction_id(instruction: &VirInstruction) -> &str {
    match instruction {
        VirInstruction::Const { id, .. }
        | VirInstruction::Copy { id, .. }
        | VirInstruction::BinOp { id, .. }
        | VirInstruction::UnaryOp { id, .. }
        | VirInstruction::Convert { id, .. }
        | VirInstruction::Field { id, .. }
        | VirInstruction::Index { id, .. }
        | VirInstruction::MakeStruct { id, .. }
        | VirInstruction::MakeArray { id, .. }
        | VirInstruction::CallStatic { id, .. } => id,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TermMetrics {
    nodes: usize,
    depth: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NodeCeiling {
    maximum: usize,
    code: &'static str,
    reported_limit: usize,
    actual_base: usize,
}

impl NodeCeiling {
    const fn member(maximum: usize) -> Self {
        Self {
            maximum,
            code: "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
            reported_limit: maximum,
            actual_base: 0,
        }
    }

    const fn document(maximum: usize) -> Self {
        Self {
            maximum,
            code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
            reported_limit: maximum,
            actual_base: 0,
        }
    }

    const fn remaining(
        maximum: usize,
        code: &'static str,
        reported_limit: usize,
        actual_base: usize,
    ) -> Self {
        Self {
            maximum,
            code,
            reported_limit,
            actual_base,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TermBuilder {
    limits: ProgramWpLimits,
}

impl TermBuilder {
    const fn new(limits: ProgramWpLimits) -> Self {
        Self { limits }
    }

    fn substitute(
        self,
        input: &MpkExprTerm,
        variables: &BTreeMap<String, MpkExprTerm>,
        results: &BTreeMap<u32, MpkExprTerm>,
        ceiling: NodeCeiling,
    ) -> Result<MpkExprTerm, ProgramWpError> {
        projected_metrics(
            input,
            variables,
            results,
            ceiling,
            self.limits.member_expression_depth,
        )?;
        substitute_unchecked(input, variables, results)
    }

    fn negate(self, value: MpkExprTerm) -> Result<MpkExprTerm, ProgramWpError> {
        if let MpkExprTerm::Apply { function, args } = &value {
            if function == STD_BOOL_NOT && args.len() == 1 {
                return Ok(args[0].clone());
            }
        }
        self.apply(STD_BOOL_NOT, vec![value])
    }

    fn conjoin(self, values: &[MpkExprTerm]) -> Result<MpkExprTerm, ProgramWpError> {
        let mut result = MpkExprTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        };
        for value in values {
            result = self.bool_and(result, value.clone())?;
        }
        Ok(result)
    }

    fn disjoin(self, values: &[MpkExprTerm]) -> Result<MpkExprTerm, ProgramWpError> {
        let mut result = MpkExprTerm::Constant {
            name: STD_BOOL_FALSE.to_owned(),
        };
        for value in values {
            result = self.bool_or(result, value.clone())?;
        }
        Ok(result)
    }

    fn bool_and(self, lhs: MpkExprTerm, rhs: MpkExprTerm) -> Result<MpkExprTerm, ProgramWpError> {
        if is_false(&lhs) || is_false(&rhs) || are_complements(&lhs, &rhs) {
            return Ok(MpkExprTerm::Constant {
                name: STD_BOOL_FALSE.to_owned(),
            });
        }
        if is_true(&lhs) {
            return Ok(rhs);
        }
        if is_true(&rhs) || lhs == rhs {
            return Ok(lhs);
        }
        self.apply(STD_BOOL_AND, vec![lhs, rhs])
    }

    fn bool_or(self, lhs: MpkExprTerm, rhs: MpkExprTerm) -> Result<MpkExprTerm, ProgramWpError> {
        if is_true(&lhs) || is_true(&rhs) || are_complements(&lhs, &rhs) {
            return Ok(MpkExprTerm::Constant {
                name: STD_BOOL_TRUE.to_owned(),
            });
        }
        if is_false(&lhs) {
            return Ok(rhs);
        }
        if is_false(&rhs) || lhs == rhs {
            return Ok(lhs);
        }
        self.apply(STD_BOOL_OR, vec![lhs, rhs])
    }

    fn select(
        self,
        selectors: &[MpkExprTerm],
        values: &[MpkExprTerm],
    ) -> Result<MpkExprTerm, ProgramWpError> {
        if selectors.len() != values.len() || values.is_empty() {
            return Err(ProgramWpError::InvalidGraph {
                function_id: "<dataflow>".to_owned(),
                detail: "selector/value cardinality mismatch".to_owned(),
            });
        }
        if values[1..].iter().all(|value| *value == values[0]) {
            return Ok(values[0].clone());
        }
        if values.len() == 2 && are_complements(&selectors[0], &selectors[1]) {
            if let Some(condition) = stripped_negation(&selectors[0]) {
                return self.apply(
                    STD_BOOL_IF,
                    vec![condition.clone(), values[1].clone(), values[0].clone()],
                );
            }
            return self.apply(
                STD_BOOL_IF,
                vec![selectors[0].clone(), values[0].clone(), values[1].clone()],
            );
        }
        let mut result = values
            .last()
            .cloned()
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: "<dataflow>".to_owned(),
                detail: "missing selected value".to_owned(),
            })?;
        for index in (0..values.len() - 1).rev() {
            if is_true(&selectors[index]) {
                result = values[index].clone();
            } else {
                result = self.apply(
                    STD_BOOL_IF,
                    vec![selectors[index].clone(), values[index].clone(), result],
                )?;
            }
        }
        Ok(result)
    }

    fn apply(self, function: &str, args: Vec<MpkExprTerm>) -> Result<MpkExprTerm, ProgramWpError> {
        let term = MpkExprTerm::Apply {
            function: function.to_owned(),
            args,
        };
        measure_term(
            &term,
            NodeCeiling::member(self.limits.expression_nodes_per_member),
            self.limits.member_expression_depth,
        )?;
        Ok(term)
    }
}

fn is_true(term: &MpkExprTerm) -> bool {
    matches!(term, MpkExprTerm::Constant { name } if name == STD_BOOL_TRUE)
}

fn is_false(term: &MpkExprTerm) -> bool {
    matches!(term, MpkExprTerm::Constant { name } if name == STD_BOOL_FALSE)
}

fn stripped_negation(term: &MpkExprTerm) -> Option<&MpkExprTerm> {
    match term {
        MpkExprTerm::Apply { function, args } if function == STD_BOOL_NOT && args.len() == 1 => {
            args.first()
        }
        _ => None,
    }
}

fn are_complements(lhs: &MpkExprTerm, rhs: &MpkExprTerm) -> bool {
    stripped_negation(lhs).is_some_and(|value| value == rhs)
        || stripped_negation(rhs).is_some_and(|value| value == lhs)
}

fn substitute_unchecked(
    input: &MpkExprTerm,
    variables: &BTreeMap<String, MpkExprTerm>,
    results: &BTreeMap<u32, MpkExprTerm>,
) -> Result<MpkExprTerm, ProgramWpError> {
    match input {
        MpkExprTerm::Var { name } => variables
            .get(name)
            .cloned()
            .ok_or_else(|| ProgramWpError::UnclosedValue { name: name.clone() }),
        MpkExprTerm::Result { index } => results
            .get(index)
            .cloned()
            .ok_or(ProgramWpError::UnclosedResult { index: *index }),
        MpkExprTerm::Constant { .. } | MpkExprTerm::BitVecLiteral { .. } => Ok(input.clone()),
        MpkExprTerm::Apply { function, args } => Ok(MpkExprTerm::Apply {
            function: function.clone(),
            args: args
                .iter()
                .map(|arg| substitute_unchecked(arg, variables, results))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        MpkExprTerm::Convert { value, target } => Ok(MpkExprTerm::Convert {
            value: Box::new(substitute_unchecked(value, variables, results)?),
            target: target.clone(),
        }),
    }
}

fn projected_metrics(
    input: &MpkExprTerm,
    variables: &BTreeMap<String, MpkExprTerm>,
    results: &BTreeMap<u32, MpkExprTerm>,
    ceiling: NodeCeiling,
    depth_limit: usize,
) -> Result<TermMetrics, ProgramWpError> {
    match input {
        MpkExprTerm::Var { name } => variables
            .get(name)
            .ok_or_else(|| ProgramWpError::UnclosedValue { name: name.clone() })
            .and_then(|term| measure_term(term, ceiling, depth_limit)),
        MpkExprTerm::Result { index } => results
            .get(index)
            .ok_or(ProgramWpError::UnclosedResult { index: *index })
            .and_then(|term| measure_term(term, ceiling, depth_limit)),
        MpkExprTerm::Constant { .. } | MpkExprTerm::BitVecLiteral { .. } => {
            check_metrics(TermMetrics { nodes: 1, depth: 1 }, ceiling, depth_limit)
        }
        MpkExprTerm::Apply { args, .. } => {
            let children = args
                .iter()
                .map(|arg| projected_metrics(arg, variables, results, ceiling, depth_limit));
            combine_metrics(children, 1, ceiling, depth_limit)
        }
        MpkExprTerm::Convert { value, target } => {
            let value = projected_metrics(value, variables, results, ceiling, depth_limit)?;
            let depth = 1_usize
                .checked_add(value.depth.max(type_depth(target)?))
                .ok_or_else(|| ProgramWpError::CounterOverflow {
                    context: "converted expression depth".to_owned(),
                })?;
            let nodes =
                value
                    .nodes
                    .checked_add(1)
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "converted expression nodes".to_owned(),
                    })?;
            check_metrics(TermMetrics { nodes, depth }, ceiling, depth_limit)
        }
    }
}

fn measure_term(
    input: &MpkExprTerm,
    ceiling: NodeCeiling,
    depth_limit: usize,
) -> Result<TermMetrics, ProgramWpError> {
    match input {
        MpkExprTerm::Var { .. }
        | MpkExprTerm::Result { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => {
            check_metrics(TermMetrics { nodes: 1, depth: 1 }, ceiling, depth_limit)
        }
        MpkExprTerm::Apply { args, .. } => combine_metrics(
            args.iter()
                .map(|arg| measure_term(arg, ceiling, depth_limit)),
            1,
            ceiling,
            depth_limit,
        ),
        MpkExprTerm::Convert { value, target } => {
            let value = measure_term(value, ceiling, depth_limit)?;
            let metrics = TermMetrics {
                nodes: value.nodes.checked_add(1).ok_or_else(|| {
                    ProgramWpError::CounterOverflow {
                        context: "converted expression nodes".to_owned(),
                    }
                })?,
                depth: 1_usize
                    .checked_add(value.depth.max(type_depth(target)?))
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "converted expression depth".to_owned(),
                    })?,
            };
            check_metrics(metrics, ceiling, depth_limit)
        }
    }
}

fn combine_metrics<I>(
    children: I,
    root_nodes: usize,
    ceiling: NodeCeiling,
    depth_limit: usize,
) -> Result<TermMetrics, ProgramWpError>
where
    I: IntoIterator<Item = Result<TermMetrics, ProgramWpError>>,
{
    let mut nodes = root_nodes;
    let mut maximum_depth = 0_usize;
    for child in children {
        let child = child?;
        nodes = nodes
            .checked_add(child.nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "expression node count".to_owned(),
            })?;
        if nodes > ceiling.maximum {
            return Err(expression_node_limit(ceiling, nodes));
        }
        maximum_depth = maximum_depth.max(child.depth);
    }
    let depth = maximum_depth
        .checked_add(1)
        .ok_or_else(|| ProgramWpError::CounterOverflow {
            context: "expression depth".to_owned(),
        })?;
    check_metrics(TermMetrics { nodes, depth }, ceiling, depth_limit)
}

fn check_metrics(
    metrics: TermMetrics,
    ceiling: NodeCeiling,
    depth_limit: usize,
) -> Result<TermMetrics, ProgramWpError> {
    if metrics.nodes > ceiling.maximum {
        return Err(expression_node_limit(ceiling, metrics.nodes));
    }
    if metrics.depth > depth_limit {
        return Err(ProgramWpError::Limit {
            code: "VC_LIMIT_MEMBER_EXPRESSION_DEPTH",
            limit: depth_limit,
            actual: metrics.depth,
        });
    }
    Ok(metrics)
}

fn expression_node_limit(ceiling: NodeCeiling, nodes: usize) -> ProgramWpError {
    match ceiling.actual_base.checked_add(nodes) {
        Some(actual) => ProgramWpError::Limit {
            code: ceiling.code,
            limit: ceiling.reported_limit,
            actual,
        },
        None => ProgramWpError::CounterOverflow {
            context: "reported expression node count".to_owned(),
        },
    }
}

fn type_depth(input: &MpkTypeTerm) -> Result<usize, ProgramWpError> {
    match input {
        MpkTypeTerm::Constant { .. }
        | MpkTypeTerm::NatLiteral { .. }
        | MpkTypeTerm::StringLiteral { .. } => Ok(1),
        MpkTypeTerm::Apply { args, .. } => {
            let mut maximum = 0_usize;
            for arg in args {
                maximum = maximum.max(type_depth(arg)?);
            }
            maximum
                .checked_add(1)
                .ok_or_else(|| ProgramWpError::CounterOverflow {
                    context: "type depth".to_owned(),
                })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemberReservation {
    assumption_nodes: usize,
    conclusion_ceiling: NodeCeiling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GenerationBudget {
    limits: ProgramWpLimits,
    document_members: usize,
    document_nodes: usize,
}

impl GenerationBudget {
    const fn new(limits: ProgramWpLimits) -> Self {
        Self {
            limits,
            document_members: 0,
            document_nodes: 0,
        }
    }

    fn document_node_ceiling(self) -> Result<NodeCeiling, ProgramWpError> {
        let remaining = self
            .limits
            .expression_nodes_per_document
            .checked_sub(self.document_nodes)
            .ok_or(ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: self.limits.expression_nodes_per_document,
                actual: self.document_nodes,
            })?;
        Ok(NodeCeiling::remaining(
            remaining,
            "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
            self.limits.expression_nodes_per_document,
            self.document_nodes,
        ))
    }

    fn record_document_term(&mut self, term: &MpkExprTerm) -> Result<(), ProgramWpError> {
        let metrics = measure_term(
            term,
            NodeCeiling::document(self.limits.expression_nodes_per_document),
            self.limits.member_expression_depth,
        )?;
        self.document_nodes = self
            .document_nodes
            .checked_add(metrics.nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "document expression nodes".to_owned(),
            })?;
        if self.document_nodes > self.limits.expression_nodes_per_document {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: self.limits.expression_nodes_per_document,
                actual: self.document_nodes,
            });
        }
        Ok(())
    }

    fn begin_member(
        &mut self,
        function_members: &mut usize,
        assumptions: &[MpkExprTerm],
    ) -> Result<MemberReservation, ProgramWpError> {
        let next_function =
            function_members
                .checked_add(1)
                .ok_or_else(|| ProgramWpError::CounterOverflow {
                    context: "function member count".to_owned(),
                })?;
        if next_function > self.limits.members_per_function {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_MEMBERS_PER_FUNCTION",
                limit: self.limits.members_per_function,
                actual: next_function,
            });
        }
        let next_document = self.document_members.checked_add(1).ok_or_else(|| {
            ProgramWpError::CounterOverflow {
                context: "document member count".to_owned(),
            }
        })?;
        if next_document > self.limits.members_per_document {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_MEMBERS_PER_DOCUMENT",
                limit: self.limits.members_per_document,
                actual: next_document,
            });
        }
        if assumptions.len() > self.limits.assumptions_per_member {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_ASSUMPTIONS_PER_MEMBER",
                limit: self.limits.assumptions_per_member,
                actual: assumptions.len(),
            });
        }

        let mut assumption_nodes = 0_usize;
        for assumption in assumptions {
            let metrics = measure_term(
                assumption,
                NodeCeiling::member(self.limits.expression_nodes_per_member),
                self.limits.member_expression_depth,
            )?;
            assumption_nodes = assumption_nodes.checked_add(metrics.nodes).ok_or_else(|| {
                ProgramWpError::CounterOverflow {
                    context: "member assumption nodes".to_owned(),
                }
            })?;
            if assumption_nodes > self.limits.expression_nodes_per_member {
                return Err(ProgramWpError::Limit {
                    code: "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
                    limit: self.limits.expression_nodes_per_member,
                    actual: assumption_nodes,
                });
            }
        }
        let member_remaining = self
            .limits
            .expression_nodes_per_member
            .checked_sub(assumption_nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "member expression-node remainder".to_owned(),
            })?;
        let document_with_assumptions = self
            .document_nodes
            .checked_add(assumption_nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "document assumption nodes".to_owned(),
            })?;
        if document_with_assumptions > self.limits.expression_nodes_per_document {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: self.limits.expression_nodes_per_document,
                actual: document_with_assumptions,
            });
        }
        let document_remaining = self
            .limits
            .expression_nodes_per_document
            .checked_sub(document_with_assumptions)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "document expression-node remainder".to_owned(),
            })?;
        let conclusion_ceiling = if member_remaining <= document_remaining {
            NodeCeiling::remaining(
                member_remaining,
                "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
                self.limits.expression_nodes_per_member,
                assumption_nodes,
            )
        } else {
            NodeCeiling::remaining(
                document_remaining,
                "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                self.limits.expression_nodes_per_document,
                document_with_assumptions,
            )
        };

        *function_members = next_function;
        self.document_members = next_document;
        Ok(MemberReservation {
            assumption_nodes,
            conclusion_ceiling,
        })
    }

    fn finish_member(
        &mut self,
        reservation: MemberReservation,
        conclusion: &MpkExprTerm,
    ) -> Result<(), ProgramWpError> {
        let metrics = measure_term(
            conclusion,
            reservation.conclusion_ceiling,
            self.limits.member_expression_depth,
        )?;
        let member_nodes = reservation
            .assumption_nodes
            .checked_add(metrics.nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "member expression nodes".to_owned(),
            })?;
        if member_nodes > self.limits.expression_nodes_per_member {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
                limit: self.limits.expression_nodes_per_member,
                actual: member_nodes,
            });
        }
        self.document_nodes = self
            .document_nodes
            .checked_add(member_nodes)
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "document expression nodes".to_owned(),
            })?;
        if self.document_nodes > self.limits.expression_nodes_per_document {
            return Err(ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: self.limits.expression_nodes_per_document,
                actual: self.document_nodes,
            });
        }
        Ok(())
    }
}

fn expression_error(
    function: &VirFunction,
    context: impl Into<String>,
    source: ProgramExprEncodeError,
) -> ProgramWpError {
    ProgramWpError::Expression {
        function_id: function.id.clone(),
        context: context.into(),
        source,
    }
}

fn safety_error(
    function: &VirFunction,
    instruction_id: &str,
    source: SafetyCheckError,
) -> ProgramWpError {
    ProgramWpError::Safety {
        function_id: function.id.clone(),
        instruction_id: instruction_id.to_owned(),
        source,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramWpError {
    Validation(VirValidationError),
    Expression {
        function_id: String,
        context: String,
        source: ProgramExprEncodeError,
    },
    Safety {
        function_id: String,
        instruction_id: String,
        source: SafetyCheckError,
    },
    UnsupportedStaticCall {
        function_id: String,
        block_label: String,
        instruction_id: String,
    },
    UnsupportedLoopCutpoint {
        function_id: String,
    },
    InvalidGraph {
        function_id: String,
        detail: String,
    },
    UnclosedValue {
        name: String,
    },
    UnclosedResult {
        index: u32,
    },
    Limit {
        code: &'static str,
        limit: usize,
        actual: usize,
    },
    CounterOverflow {
        context: String,
    },
}

impl ProgramWpError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VC_PROGRAM_VIR_INVALID",
            Self::Expression { .. } => "VC_PROGRAM_EXPRESSION",
            Self::Safety { .. } => "VC_PROGRAM_SAFETY",
            Self::UnsupportedStaticCall { .. } => "VC_PROGRAM_CALL_UNSUPPORTED",
            Self::UnsupportedLoopCutpoint { .. } => "VC_PROGRAM_LOOP_UNSUPPORTED",
            Self::InvalidGraph { .. } => "VC_PROGRAM_GRAPH",
            Self::UnclosedValue { .. } | Self::UnclosedResult { .. } => "VC_PROGRAM_UNCLOSED_TERM",
            Self::Limit { code, .. } => code,
            Self::CounterOverflow { .. } => "VC_LIMIT_COUNTER_OVERFLOW",
        }
    }
}

impl fmt::Display for ProgramWpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => write!(formatter, "VIR validation failed: {error}"),
            Self::Expression {
                function_id,
                context,
                source,
            } => write!(
                formatter,
                "program expression failed in {function_id} at {context}: {source}"
            ),
            Self::Safety {
                function_id,
                instruction_id,
                source,
            } => write!(
                formatter,
                "program safety failed in {function_id} at {instruction_id}: {source}"
            ),
            Self::UnsupportedStaticCall {
                function_id,
                block_label,
                instruction_id,
            } => write!(
                formatter,
                "static call {instruction_id} in {function_id}/{block_label} awaits VIR-01-T10"
            ),
            Self::UnsupportedLoopCutpoint { function_id } => {
                write!(
                    formatter,
                    "loop cutpoint in {function_id} awaits VIR-01-T09"
                )
            }
            Self::InvalidGraph {
                function_id,
                detail,
            } => write!(
                formatter,
                "invalid program graph in {function_id}: {detail}"
            ),
            Self::UnclosedValue { name } => write!(formatter, "unclosed program value {name}"),
            Self::UnclosedResult { index } => write!(formatter, "unclosed result index {index}"),
            Self::Limit {
                code,
                limit,
                actual,
            } => write!(formatter, "{code}: limit={limit}; actual={actual}"),
            Self::CounterOverflow { context } => write!(formatter, "counter overflow: {context}"),
        }
    }
}

impl std::error::Error for ProgramWpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::Expression { source, .. } => Some(source),
            Self::Safety { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_wp_substitution_rejects_projected_growth_before_building() {
        let limits = ProgramWpLimits {
            expression_nodes_per_member: 4,
            ..ProgramWpLimits::production()
        };
        let builder = TermBuilder::new(limits);
        let replacement = MpkExprTerm::apply(
            "Std.BitVec.BV8.add",
            [
                MpkExprTerm::BitVecLiteral {
                    value: "1".to_owned(),
                    width: 8,
                    signed: false,
                },
                MpkExprTerm::BitVecLiteral {
                    value: "2".to_owned(),
                    width: 8,
                    signed: false,
                },
            ],
        );
        let input = MpkExprTerm::apply(
            "Std.BitVec.BV8.add",
            [
                MpkExprTerm::Var {
                    name: "arg0".to_owned(),
                },
                MpkExprTerm::Var {
                    name: "arg0".to_owned(),
                },
            ],
        );
        let variables = BTreeMap::from([("arg0".to_owned(), replacement)]);
        let error = builder
            .substitute(&input, &variables, &BTreeMap::new(), NodeCeiling::member(4))
            .unwrap_err();
        assert_eq!(error.code(), "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER");
    }

    #[test]
    fn program_wp_budget_reserves_member_before_conclusion() {
        let limits = ProgramWpLimits {
            members_per_function: 0,
            ..ProgramWpLimits::production()
        };
        let mut budget = GenerationBudget::new(limits);
        let error = budget.begin_member(&mut 0, &[]).unwrap_err();
        assert_eq!(error.code(), "VC_LIMIT_MEMBERS_PER_FUNCTION");
    }

    #[test]
    fn program_wp_budget_rejects_document_assumptions_before_reserving_member() {
        let limits = ProgramWpLimits {
            expression_nodes_per_document: 1,
            ..ProgramWpLimits::production()
        };
        let term = MpkExprTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        };
        let mut budget = GenerationBudget::new(limits);
        budget.record_document_term(&term).expect("one node fits");
        let mut function_members = 0;

        let error = budget
            .begin_member(&mut function_members, std::slice::from_ref(&term))
            .unwrap_err();

        assert_eq!(
            error,
            ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: 1,
                actual: 2,
            }
        );
        assert_eq!(function_members, 0);
        assert_eq!(budget.document_members, 0);
    }

    #[test]
    fn program_wp_budget_reports_cumulative_node_limit_for_projected_conclusion() {
        let limits = ProgramWpLimits {
            expression_nodes_per_document: 2,
            ..ProgramWpLimits::production()
        };
        let term = MpkExprTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        };
        let mut budget = GenerationBudget::new(limits);
        budget.record_document_term(&term).expect("require fits");
        let mut function_members = 0;
        let reservation = budget
            .begin_member(&mut function_members, std::slice::from_ref(&term))
            .expect("assumption fits exactly");

        let error = TermBuilder::new(limits)
            .substitute(
                &term,
                &BTreeMap::new(),
                &BTreeMap::new(),
                reservation.conclusion_ceiling,
            )
            .unwrap_err();

        assert_eq!(
            error,
            ProgramWpError::Limit {
                code: "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
                limit: 2,
                actual: 3,
            }
        );
    }

    #[test]
    fn program_wp_budget_enforces_assumption_member_and_depth_limits() {
        let term = MpkExprTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        };

        let assumption_limits = ProgramWpLimits {
            assumptions_per_member: 0,
            ..ProgramWpLimits::production()
        };
        let assumption_error = GenerationBudget::new(assumption_limits)
            .begin_member(&mut 0, std::slice::from_ref(&term))
            .unwrap_err();
        assert_eq!(assumption_error.code(), "VC_LIMIT_ASSUMPTIONS_PER_MEMBER");

        let document_member_limits = ProgramWpLimits {
            members_per_document: 0,
            ..ProgramWpLimits::production()
        };
        let document_member_error = GenerationBudget::new(document_member_limits)
            .begin_member(&mut 0, &[])
            .unwrap_err();
        assert_eq!(
            document_member_error.code(),
            "VC_LIMIT_MEMBERS_PER_DOCUMENT"
        );

        let depth_limits = ProgramWpLimits {
            member_expression_depth: 1,
            ..ProgramWpLimits::production()
        };
        let depth_error = TermBuilder::new(depth_limits)
            .apply(STD_BOOL_NOT, vec![term])
            .unwrap_err();
        assert_eq!(depth_error.code(), "VC_LIMIT_MEMBER_EXPRESSION_DEPTH");
    }
}
