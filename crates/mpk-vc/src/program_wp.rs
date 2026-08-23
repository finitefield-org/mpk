//! Unified VIR weakest-precondition and operation-safety generation.
//!
//! The engine consumes only fully validated VIR. It propagates bounded symbolic
//! states, substitutes block arguments at edges, and merges predecessor states
//! with identical relational-call scopes deterministically. Validated Go loop
//! backedges are cut at their contracted headers. Contract and safety members
//! are collected during that same traversal so their path semantics cannot
//! drift.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::call_wp::{
    program_declaration_name, CallWpError, ProgramCallDependencies, ProgramCallGraph,
    ProgramDeclarationKind,
};
use crate::expr_encode::{
    MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_FALSE, STD_BOOL_IF, STD_BOOL_NOT,
    STD_BOOL_OR, STD_BOOL_TRUE, STD_EQ,
};
use crate::program_encode::{
    encode_vir_contract_expr, encode_vir_instruction_expr, encode_vir_value, ProgramExprContext,
    ProgramExprEncodeError,
};
use crate::safety_check::{
    classify_safety_evidence, encode_instruction_safety, SafetyCheckError, SafetyEvidenceRoute,
};
use crate::type_encode::{encode_vir_type, MpkTypeTerm};
use crate::vir::{
    VirBinaryOperator, VirBinding, VirContractExpr, VirFunction, VirInstruction, VirLoopContract,
    VirModule, VirTerminator, VirType, VirUnit, VirValue,
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
    pub direct_callees: Vec<String>,
    pub contract_dependencies: Vec<String>,
    pub panic_free_dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProgramVcMemberKind {
    CalleePanicFree,
    CalleePrecondition,
    LoopDecreases,
    LoopExit,
    LoopInitialization,
    LoopPreservation,
    OperationSafety,
    Postcondition,
}

impl ProgramVcMemberKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CalleePanicFree => "callee_panic_free",
            Self::CalleePrecondition => "callee_precondition",
            Self::LoopDecreases => "loop_decreases",
            Self::LoopExit => "loop_exit",
            Self::LoopInitialization => "loop_initialization",
            Self::LoopPreservation => "loop_preservation",
            Self::OperationSafety => "operation_safety",
            Self::Postcondition => "postcondition",
        }
    }

    const fn group_suffix(self) -> &'static str {
        match self {
            Self::CalleePanicFree | Self::OperationSafety => "panic_free",
            Self::CalleePrecondition
            | Self::LoopDecreases
            | Self::LoopExit
            | Self::LoopInitialization
            | Self::LoopPreservation
            | Self::Postcondition => "contract",
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
        let call_graph = ProgramCallGraph::analyze(input).map_err(ProgramWpError::Call)?;

        let mut budget = GenerationBudget::new(self.limits);
        let mut output = Vec::new();
        for (unit, function) in call_graph.ordered_functions() {
            output.push(self.generate_function(input, unit, function, &call_graph, &mut budget)?);
        }
        Ok(ProgramVcModule { functions: output })
    }

    fn generate_function(
        self,
        module: &VirModule,
        unit: &VirUnit,
        function: &VirFunction,
        call_graph: &ProgramCallGraph<'_>,
        budget: &mut GenerationBudget,
    ) -> Result<ProgramVcFunction, ProgramWpError> {
        self.generate_program_function(module, unit, function, call_graph, budget)
    }

    fn generate_program_function(
        self,
        module: &VirModule,
        unit: &VirUnit,
        function: &VirFunction,
        call_graph: &ProgramCallGraph<'_>,
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
        let mut deferred_array_values = function
            .blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .filter_map(|instruction| match instruction {
                VirInstruction::Index { id, .. } | VirInstruction::MakeArray { id, .. } => {
                    Some(id.clone())
                }
                _ => None,
            })
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
        let loop_analysis = LoopAnalysis::for_validated_function(function, &labels)?;
        let mut loop_runtime = LoopRuntime::default();
        let mut remaining_predecessors = vec![0_usize; function.blocks.len()];
        for (source_index, block) in function.blocks.iter().enumerate() {
            for successor in successors(&block.terminator) {
                let target =
                    labels
                        .get(successor)
                        .copied()
                        .ok_or_else(|| ProgramWpError::InvalidGraph {
                            function_id: function.id.clone(),
                            detail: format!("unknown successor {successor}"),
                        })?;
                if loop_analysis.is_cut_edge(source_index, target) {
                    continue;
                }
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
            delivered_edges: BTreeSet::new(),
            ready,
        };
        let mut processed = 0_usize;
        let mut pending = Vec::new();
        let mut loop_returns = BTreeMap::<usize, Vec<LoopReturnState>>::new();
        let mut function_member_count = 0_usize;

        while let Some(block_index) = worklist.ready.pop_first() {
            let block = &function.blocks[block_index];
            let states = if block_index == 0 {
                if !worklist.incoming[block_index].is_empty() {
                    return Err(ProgramWpError::InvalidGraph {
                        function_id: function.id.clone(),
                        detail: "entry block has an incoming edge".to_owned(),
                    });
                }
                vec![SymbolicState {
                    env: initial_env.clone(),
                    assumptions: Vec::new(),
                    outer_assumptions: Vec::new(),
                    call_scopes: Vec::new(),
                    origin_header: None,
                }]
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
            let loop_contract = loop_analysis.contract(block_index, function);
            let mut loop_body_processed = false;

            for mut state in states {
                if let Some(loop_contract) = loop_contract {
                    generate_loop_initialization(
                        function,
                        &context,
                        &builder,
                        budget,
                        &mut function_member_count,
                        &mut pending,
                        block_index,
                        loop_contract,
                        &state,
                    )?;
                    if loop_body_processed {
                        continue;
                    }
                    state = loop_cutpoint_state(
                        function,
                        &context,
                        &builder,
                        block_index,
                        loop_contract,
                    )?;
                    loop_body_processed = true;
                }

                for (instruction_index, instruction) in block.instructions.iter().enumerate() {
                    if matches!(instruction, VirInstruction::CallStatic { .. }) {
                        process_static_call(
                            module,
                            function,
                            &context,
                            &builder,
                            budget,
                            &mut function_member_count,
                            &mut pending,
                            call_graph,
                            block_index,
                            instruction_index,
                            instruction,
                            &mut state,
                        )?;
                        continue;
                    }

                    let safety =
                        encode_instruction_safety(&context, instruction).map_err(|source| {
                            safety_error(function, instruction_id(instruction), source)
                        })?;
                    for (check_index, predicate) in safety.into_iter().enumerate() {
                        let assumptions = member_assumptions(&state).to_vec();
                        let reservation =
                            budget.begin_member(&mut function_member_count, &assumptions)?;
                        let proposition = builder.substitute(
                            &predicate.proposition,
                            &state.env,
                            &empty_results,
                            reservation.conclusion_ceiling,
                        )?;
                        let evidence =
                            classify_safety_evidence(module.semantic_profile, &proposition);
                        let conclusion = wrap_call_continuation(
                            &builder,
                            &state,
                            proposition,
                            reservation.conclusion_ceiling,
                        )?;
                        budget.finish_member(reservation, &conclusion)?;
                        pending.push(close_pending_member(
                            function,
                            &context,
                            state.origin_header,
                            MemberOrigin::new(block_index, instruction_index, check_index),
                            ProgramVcMemberKind::OperationSafety,
                            assumptions,
                            conclusion,
                            Some(evidence),
                        )?);
                    }

                    // Array constructors and projections are deliberately
                    // deferred until the checked array-value foundation is
                    // integrated. Safety is independently encoded above; a
                    // substantive semantic use of an omitted value still
                    // fails as an unclosed VIR value. Copy only propagates the
                    // deferred identity needed to reach an Index operation.
                    if let VirInstruction::Copy {
                        id,
                        target,
                        value: VirValue::Variable(reference),
                        ..
                    } = instruction
                    {
                        if deferred_array_values.contains(&reference.var) {
                            deferred_array_values.insert(id.clone());
                            deferred_array_values.insert(target.clone());
                            continue;
                        }
                    }
                    if matches!(
                        instruction,
                        VirInstruction::Index { .. } | VirInstruction::MakeArray { .. }
                    ) {
                        continue;
                    }

                    let raw =
                        encode_vir_instruction_expr(&context, instruction).map_err(|source| {
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

                if let Some(loop_contract) = loop_analysis.contract(block_index, function) {
                    loop_runtime.record_before_variants(
                        function,
                        &context,
                        &builder,
                        block_index,
                        loop_contract,
                        &state,
                    )?;
                }

                match &block.terminator {
                    VirTerminator::Return { values } => {
                        let mut results = BTreeMap::new();
                        for (index, value) in values.iter().enumerate() {
                            let result_index = u32::try_from(index).map_err(|_| {
                                ProgramWpError::CounterOverflow {
                                    context: format!("{} return index", function.id),
                                }
                            })?;
                            let raw = encode_vir_value(&context, value).map_err(|source| {
                                expression_error(function, format!("return[{index}]"), source)
                            })?;
                            let value = builder.substitute(
                                &raw,
                                &state.env,
                                &empty_results,
                                NodeCeiling::member(self.limits.expression_nodes_per_member),
                            );
                            match value {
                                Ok(value) => {
                                    results.insert(result_index, value);
                                }
                                Err(ProgramWpError::UnclosedValue { name })
                                    if deferred_array_values.contains(&name)
                                        && !function.contracts.ensures.iter().any(|ensure| {
                                            contract_result_is_required(ensure, result_index)
                                        }) => {}
                                Err(error) => return Err(error),
                            }
                        }
                        if let Some(header) = state.origin_header {
                            loop_returns
                                .entry(header)
                                .or_default()
                                .push(LoopReturnState {
                                    block_index,
                                    state,
                                    results,
                                });
                        } else {
                            for (ensure_index, ensure) in
                                function.contracts.ensures.iter().enumerate()
                            {
                                let assumptions = member_assumptions(&state).to_vec();
                                let reservation = budget
                                    .begin_member(&mut function_member_count, &assumptions)?;
                                let raw = encode_vir_contract_expr(&context, ensure).map_err(
                                    |source| {
                                        expression_error(
                                            function,
                                            format!("bb{block_index}.ensures[{ensure_index}]"),
                                            source,
                                        )
                                    },
                                )?;
                                let raw = close_unavailable_result_tautologies(&raw, &results);
                                let conclusion = builder.substitute(
                                    &raw,
                                    &state.env,
                                    &results,
                                    reservation.conclusion_ceiling,
                                )?;
                                let conclusion = wrap_call_continuation(
                                    &builder,
                                    &state,
                                    conclusion,
                                    reservation.conclusion_ceiling,
                                )?;
                                budget.finish_member(reservation, &conclusion)?;
                                pending.push(close_pending_member(
                                    function,
                                    &context,
                                    None,
                                    MemberOrigin::new(block_index, ensure_index, 0),
                                    ProgramVcMemberKind::Postcondition,
                                    assumptions,
                                    conclusion,
                                    None,
                                )?);
                            }
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
                        if loop_analysis.is_cut_edge(block_index, target_index) {
                            generate_loop_backedge_members(
                                function,
                                &context,
                                &builder,
                                budget,
                                &mut function_member_count,
                                &mut pending,
                                &loop_analysis,
                                &loop_runtime,
                                block_index,
                                target_index,
                                &edge_state,
                            )?;
                        } else {
                            push_incoming(
                                function,
                                block_index,
                                0,
                                target_index,
                                edge_state,
                                &mut worklist,
                            )?;
                        }
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
                            if edge_index == 1 {
                                if let Some(loop_contract) =
                                    loop_analysis.contract(block_index, function)
                                {
                                    generate_loop_nonnegative_members(
                                        function,
                                        &context,
                                        &builder,
                                        budget,
                                        &mut function_member_count,
                                        &mut pending,
                                        block_index,
                                        loop_contract,
                                        &loop_runtime,
                                        &branch_state,
                                    )?;
                                }
                            }
                            let edge_state = edge_state(
                                function,
                                &context,
                                &builder,
                                &base_bindings,
                                &branch_state,
                                &function.blocks[target_index].parameters,
                                args,
                            )?;
                            if loop_analysis.is_cut_edge(block_index, target_index) {
                                return Err(ProgramWpError::InvalidGraph {
                                    function_id: function.id.clone(),
                                    detail: "validated loop backedge is not a Jump".to_owned(),
                                });
                            }
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
        }

        generate_loop_exit_members(
            function,
            &context,
            &builder,
            budget,
            &mut function_member_count,
            &mut pending,
            loop_returns,
        )?;

        if processed != function.blocks.len() {
            return Err(ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: "cut CFG traversal did not process every block".to_owned(),
            });
        }

        let ProgramCallDependencies {
            direct_callees,
            contract_dependencies,
            panic_free_dependencies,
        } = call_graph
            .dependencies(&function.id)
            .map_err(ProgramWpError::Call)?;
        Ok(ProgramVcFunction {
            function_id: function.id.clone(),
            requires,
            members: finalize_members(function, pending)?,
            direct_callees,
            contract_dependencies,
            panic_free_dependencies,
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
    outer_assumptions: Vec<MpkExprTerm>,
    call_scopes: Vec<CallScope>,
    origin_header: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CallScope {
    binder_type: MpkTypeTerm,
    ensures: Vec<MpkExprTerm>,
    continuation_assumptions: Vec<MpkExprTerm>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoopReturnState {
    block_index: usize,
    state: SymbolicState,
    results: BTreeMap<u32, MpkExprTerm>,
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
    delivered_edges: BTreeSet<(usize, usize, usize)>,
    ready: BTreeSet<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LoopAnalysis {
    contract_by_header: BTreeMap<usize, usize>,
    cut_edges: BTreeSet<(usize, usize)>,
    backedges_by_header: BTreeMap<usize, Vec<usize>>,
}

impl LoopAnalysis {
    fn for_validated_function(
        function: &VirFunction,
        labels: &BTreeMap<&str, usize>,
    ) -> Result<Self, ProgramWpError> {
        if function.contracts.loops.is_empty() {
            return Ok(Self::default());
        }

        let edges = function
            .blocks
            .iter()
            .map(|block| {
                successors(&block.terminator)
                    .into_iter()
                    .map(|label| {
                        labels
                            .get(label)
                            .copied()
                            .ok_or_else(|| ProgramWpError::InvalidGraph {
                                function_id: function.id.clone(),
                                detail: format!("unknown loop successor {label}"),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let components = strongly_connected_components(&edges);
        let mut component_by_block = vec![0_usize; function.blocks.len()];
        for (component_index, component) in components.iter().enumerate() {
            for block in component {
                component_by_block[*block] = component_index;
            }
        }
        let mut analysis = Self::default();
        let mut contracted_components = BTreeSet::new();

        for (contract_index, contract) in function.contracts.loops.iter().enumerate() {
            let header = labels
                .get(contract.header.as_str())
                .copied()
                .ok_or_else(|| ProgramWpError::InvalidGraph {
                    function_id: function.id.clone(),
                    detail: format!("unknown loop header {}", contract.header),
                })?;
            if analysis
                .contract_by_header
                .insert(header, contract_index)
                .is_some()
            {
                return Err(ProgramWpError::InvalidGraph {
                    function_id: function.id.clone(),
                    detail: format!("duplicate loop header {}", contract.header),
                });
            }

            let component_index = component_by_block[header];
            if !contracted_components.insert(component_index) {
                return Err(ProgramWpError::InvalidGraph {
                    function_id: function.id.clone(),
                    detail: "validated loop component has multiple headers".to_owned(),
                });
            }
            let members = &components[component_index];

            let mut backedges = members
                .iter()
                .copied()
                .filter(|source| *source != header && edges[*source].contains(&header))
                .collect::<Vec<_>>();
            backedges.sort_unstable();
            if backedges.is_empty() {
                return Err(ProgramWpError::InvalidGraph {
                    function_id: function.id.clone(),
                    detail: format!("loop header {} has no backedge", contract.header),
                });
            }
            for source in &backedges {
                if !matches!(
                    function.blocks[*source].terminator,
                    VirTerminator::Jump { .. }
                ) {
                    return Err(ProgramWpError::InvalidGraph {
                        function_id: function.id.clone(),
                        detail: "validated loop backedge is not a Jump".to_owned(),
                    });
                }
                analysis.cut_edges.insert((*source, header));
            }
            analysis.backedges_by_header.insert(header, backedges);
        }

        if !is_acyclic_after_cut(&edges, &analysis.cut_edges) {
            return Err(ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: "loop cutpoints leave an uncovered cycle".to_owned(),
            });
        }
        Ok(analysis)
    }

    fn contract<'a>(
        &self,
        header: usize,
        function: &'a VirFunction,
    ) -> Option<&'a VirLoopContract> {
        self.contract_by_header
            .get(&header)
            .map(|index| &function.contracts.loops[*index])
    }

    fn is_cut_edge(&self, source: usize, target: usize) -> bool {
        self.cut_edges.contains(&(source, target))
    }

    fn backedge_rank(&self, header: usize, source: usize) -> Option<usize> {
        self.backedges_by_header
            .get(&header)
            .and_then(|sources| sources.binary_search(&source).ok())
    }
}

fn strongly_connected_components(edges: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut reverse = vec![Vec::new(); edges.len()];
    for (source, successors) in edges.iter().enumerate() {
        for target in successors {
            reverse[*target].push(source);
        }
    }
    let mut seen = vec![false; edges.len()];
    let mut order = Vec::with_capacity(edges.len());
    for source in 0..edges.len() {
        if seen[source] {
            continue;
        }
        seen[source] = true;
        let mut pending = vec![(source, 0_usize)];
        while let Some((node, next_edge)) = pending.last_mut() {
            if let Some(target) = edges[*node].get(*next_edge).copied() {
                *next_edge += 1;
                if !seen[target] {
                    seen[target] = true;
                    pending.push((target, 0));
                }
            } else {
                order.push(*node);
                pending.pop();
            }
        }
    }
    seen.fill(false);
    let mut components = Vec::new();
    while let Some(source) = order.pop() {
        if seen[source] {
            continue;
        }
        seen[source] = true;
        let mut component = Vec::new();
        let mut pending = vec![source];
        while let Some(node) = pending.pop() {
            component.push(node);
            for target in reverse[node].iter().rev() {
                if !seen[*target] {
                    seen[*target] = true;
                    pending.push(*target);
                }
            }
        }
        components.push(component);
    }
    components
}

fn is_acyclic_after_cut(edges: &[Vec<usize>], cut_edges: &BTreeSet<(usize, usize)>) -> bool {
    let mut indegree = vec![0_usize; edges.len()];
    for (source, successors) in edges.iter().enumerate() {
        for target in successors {
            if !cut_edges.contains(&(source, *target)) {
                indegree[*target] += 1;
            }
        }
    }
    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect::<Vec<_>>();
    let mut processed = 0_usize;
    while let Some(source) = ready.pop() {
        processed += 1;
        for target in &edges[source] {
            if cut_edges.contains(&(source, *target)) {
                continue;
            }
            indegree[*target] -= 1;
            if indegree[*target] == 0 {
                ready.push(*target);
            }
        }
    }
    processed == edges.len()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoopVariant {
    before: MpkExprTerm,
    width: u32,
    signed: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LoopRuntime {
    variants_by_header: BTreeMap<usize, Vec<LoopVariant>>,
}

impl LoopRuntime {
    fn record_before_variants(
        &mut self,
        function: &VirFunction,
        context: &ProgramExprContext,
        builder: &TermBuilder,
        header: usize,
        contract: &VirLoopContract,
        state: &SymbolicState,
    ) -> Result<(), ProgramWpError> {
        let mut variants = Vec::with_capacity(contract.decreases.len());
        for (index, expression) in contract.decreases.iter().enumerate() {
            let raw = encode_vir_contract_expr(context, expression).map_err(|source| {
                expression_error(
                    function,
                    format!("loop {} decreases[{index}]", contract.header),
                    source,
                )
            })?;
            let before = builder.substitute(
                &raw,
                &state.env,
                &BTreeMap::new(),
                NodeCeiling::member(builder.limits.expression_nodes_per_member),
            )?;
            let VirType::Bv { width, signed } = context
                .contract_expr_type(expression)
                .map_err(|source| expression_error(function, "loop decreases type", source))?
            else {
                return Err(ProgramWpError::InvalidGraph {
                    function_id: function.id.clone(),
                    detail: "validated loop decreases is not a bitvector".to_owned(),
                });
            };
            variants.push(LoopVariant {
                before,
                width: width.bits(),
                signed,
            });
        }
        if self.variants_by_header.insert(header, variants).is_some() {
            return Err(ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: format!("loop header {} processed twice", contract.header),
            });
        }
        Ok(())
    }

    fn variants(
        &self,
        function: &VirFunction,
        header: usize,
    ) -> Result<&[LoopVariant], ProgramWpError> {
        self.variants_by_header
            .get(&header)
            .map(Vec::as_slice)
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: format!(
                    "loop header {} has no cutpoint variants",
                    function.blocks[header].label
                ),
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingMember {
    origin: MemberOrigin,
    kind: ProgramVcMemberKind,
    local_binders: Vec<MpkTypeTerm>,
    assumptions: Vec<MpkExprTerm>,
    conclusion: MpkExprTerm,
    safety_evidence: Option<SafetyEvidenceRoute>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MemberOrigin {
    primary: usize,
    secondary: usize,
    tertiary: usize,
}

impl MemberOrigin {
    const fn new(primary: usize, secondary: usize, tertiary: usize) -> Self {
        Self {
            primary,
            secondary,
            tertiary,
        }
    }
}

fn member_assumptions(state: &SymbolicState) -> &[MpkExprTerm] {
    if state.call_scopes.is_empty() {
        &state.assumptions
    } else {
        &state.outer_assumptions
    }
}

fn wrap_call_continuation(
    builder: &TermBuilder,
    state: &SymbolicState,
    mut body: MpkExprTerm,
    ceiling: NodeCeiling,
) -> Result<MpkExprTerm, ProgramWpError> {
    if state.call_scopes.is_empty() {
        return Ok(body);
    }
    for (index, scope) in state.call_scopes.iter().enumerate().rev() {
        let continuation_assumptions = if index + 1 == state.call_scopes.len() {
            state.assumptions.as_slice()
        } else {
            scope.continuation_assumptions.as_slice()
        };
        if !continuation_assumptions.is_empty() {
            let antecedent = builder.conjoin_exact(continuation_assumptions, ceiling)?;
            body = builder.imply(antecedent, body, ceiling)?;
        }
        let ensures = builder.conjoin_exact(&scope.ensures, ceiling)?;
        body = builder.imply(ensures, body, ceiling)?;
        body = builder.forall(scope.binder_type.clone(), body, ceiling)?;
    }
    Ok(body)
}

fn loop_cutpoint_state(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    header: usize,
    contract: &VirLoopContract,
) -> Result<SymbolicState, ProgramWpError> {
    let mut env = function
        .params
        .iter()
        .chain(function.locals.iter())
        .chain(function.blocks[header].parameters.iter())
        .map(|binding| {
            (
                binding.id.clone(),
                MpkExprTerm::Var {
                    name: binding.id.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut assumptions = Vec::with_capacity(contract.invariants.len());
    for (index, invariant) in contract.invariants.iter().enumerate() {
        let raw = encode_vir_contract_expr(context, invariant).map_err(|source| {
            expression_error(
                function,
                format!("loop {} invariant[{index}]", contract.header),
                source,
            )
        })?;
        let encoded = builder.substitute(
            &raw,
            &env,
            &BTreeMap::new(),
            NodeCeiling::member(builder.limits.expression_nodes_per_member),
        )?;
        push_assumption(&mut assumptions, encoded, builder.limits)?;
    }
    // Instruction temporaries are introduced only while traversing the header.
    env.retain(|name, _| {
        function.params.iter().any(|binding| binding.id == *name)
            || function.locals.iter().any(|binding| binding.id == *name)
            || function.blocks[header]
                .parameters
                .iter()
                .any(|binding| binding.id == *name)
    });
    Ok(SymbolicState {
        env,
        assumptions,
        outer_assumptions: Vec::new(),
        call_scopes: Vec::new(),
        origin_header: Some(header),
    })
}

#[allow(clippy::too_many_arguments)]
fn generate_loop_initialization(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    budget: &mut GenerationBudget,
    function_member_count: &mut usize,
    pending: &mut Vec<PendingMember>,
    header: usize,
    contract: &VirLoopContract,
    incoming: &SymbolicState,
) -> Result<(), ProgramWpError> {
    for (invariant_index, invariant) in contract.invariants.iter().enumerate() {
        let assumptions = member_assumptions(incoming).to_vec();
        let reservation = budget.begin_member(function_member_count, &assumptions)?;
        let raw = encode_vir_contract_expr(context, invariant).map_err(|source| {
            expression_error(
                function,
                format!("loop {} initialization[{invariant_index}]", contract.header),
                source,
            )
        })?;
        let conclusion = builder.substitute(
            &raw,
            &incoming.env,
            &BTreeMap::new(),
            reservation.conclusion_ceiling,
        )?;
        let conclusion = wrap_call_continuation(
            builder,
            incoming,
            conclusion,
            reservation.conclusion_ceiling,
        )?;
        budget.finish_member(reservation, &conclusion)?;
        pending.push(close_pending_member(
            function,
            context,
            incoming.origin_header,
            MemberOrigin::new(header, invariant_index, 0),
            ProgramVcMemberKind::LoopInitialization,
            assumptions,
            conclusion,
            None,
        )?);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_static_call(
    module: &VirModule,
    caller: &VirFunction,
    caller_context: &ProgramExprContext,
    builder: &TermBuilder,
    budget: &mut GenerationBudget,
    function_member_count: &mut usize,
    pending: &mut Vec<PendingMember>,
    call_graph: &ProgramCallGraph<'_>,
    block_index: usize,
    instruction_index: usize,
    instruction: &VirInstruction,
    state: &mut SymbolicState,
) -> Result<(), ProgramWpError> {
    let VirInstruction::CallStatic {
        id,
        r#type,
        function,
        args,
        ..
    } = instruction
    else {
        return Err(ProgramWpError::InvalidGraph {
            function_id: caller.id.clone(),
            detail: "non-call reached static-call processing".to_owned(),
        });
    };
    let (callee_unit, callee) = call_graph.resolve(function).map_err(ProgramWpError::Call)?;
    if callee.results.len() != 1
        || callee.results[0].r#type != *r#type
        || callee.params.len() != args.len()
    {
        return Err(ProgramWpError::CallSignature {
            caller: caller.id.clone(),
            callee: callee.id.clone(),
        });
    }
    let mut encoded_args = Vec::with_capacity(args.len());
    for (argument_index, (argument, parameter)) in args.iter().zip(&callee.params).enumerate() {
        let actual_type = caller_context.value_type(argument).map_err(|source| {
            expression_error(
                caller,
                format!("call {id} argument[{argument_index}] type"),
                source,
            )
        })?;
        if actual_type != parameter.r#type {
            return Err(ProgramWpError::CallSignature {
                caller: caller.id.clone(),
                callee: callee.id.clone(),
            });
        }
        let raw = encode_vir_value(caller_context, argument).map_err(|source| {
            expression_error(
                caller,
                format!("call {id} argument[{argument_index}]"),
                source,
            )
        })?;
        encoded_args.push(builder.substitute(
            &raw,
            &state.env,
            &BTreeMap::new(),
            NodeCeiling::member(builder.limits.expression_nodes_per_member),
        )?);
    }

    let callee_context = ProgramExprContext::for_validated_function(module, callee_unit, callee)
        .map_err(|source| expression_error(caller, format!("callee context {function}"), source))?;
    let callee_variables = callee
        .params
        .iter()
        .zip(&encoded_args)
        .map(|(parameter, argument)| (parameter.id.clone(), argument.clone()))
        .collect::<BTreeMap<_, _>>();
    let outer_assumptions = member_assumptions(state).to_vec();
    for (require_index, require) in callee.contracts.requires.iter().enumerate() {
        let reservation = budget.begin_member(function_member_count, &outer_assumptions)?;
        let raw = encode_vir_contract_expr(&callee_context, require).map_err(|source| {
            expression_error(
                caller,
                format!("call {id} requires[{require_index}]"),
                source,
            )
        })?;
        let conclusion = builder.substitute(
            &raw,
            &callee_variables,
            &BTreeMap::new(),
            reservation.conclusion_ceiling,
        )?;
        let conclusion =
            wrap_call_continuation(builder, state, conclusion, reservation.conclusion_ceiling)?;
        budget.finish_member(reservation, &conclusion)?;
        pending.push(close_pending_member(
            caller,
            caller_context,
            state.origin_header,
            MemberOrigin::new(block_index, instruction_index, require_index),
            ProgramVcMemberKind::CalleePrecondition,
            outer_assumptions.clone(),
            conclusion,
            None,
        )?);
    }

    let reservation = budget.begin_member(function_member_count, &outer_assumptions)?;
    let panic_name = program_declaration_name(&callee.id, ProgramDeclarationKind::PanicFree);
    let panic_free = if encoded_args.is_empty() {
        MpkExprTerm::Constant { name: panic_name }
    } else {
        builder.apply_with_ceiling(
            &panic_name,
            encoded_args.clone(),
            reservation.conclusion_ceiling,
        )?
    };
    let panic_free =
        wrap_call_continuation(builder, state, panic_free, reservation.conclusion_ceiling)?;
    budget.finish_member(reservation, &panic_free)?;
    pending.push(close_pending_member(
        caller,
        caller_context,
        state.origin_header,
        MemberOrigin::new(block_index, instruction_index, 0),
        ProgramVcMemberKind::CalleePanicFree,
        outer_assumptions,
        panic_free,
        None,
    )?);

    if state.call_scopes.is_empty() {
        state.outer_assumptions = std::mem::take(&mut state.assumptions);
    } else {
        let current = std::mem::take(&mut state.assumptions);
        let scope = state
            .call_scopes
            .last_mut()
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: caller.id.clone(),
                detail: "call scope disappeared".to_owned(),
            })?;
        scope.continuation_assumptions = current;
    }
    for value in state.env.values_mut() {
        *value = shift_bound_indices(value, 1, 0)?;
    }
    let shifted_args = encoded_args
        .iter()
        .map(|argument| shift_bound_indices(argument, 1, 0))
        .collect::<Result<Vec<_>, _>>()?;
    let shifted_variables = callee
        .params
        .iter()
        .zip(&shifted_args)
        .map(|(parameter, argument)| (parameter.id.clone(), argument.clone()))
        .collect::<BTreeMap<_, _>>();
    let fresh_result = MpkExprTerm::Bound { index: 0 };
    let result_terms = BTreeMap::from([(0_u32, fresh_result.clone())]);
    let mut ensures = Vec::with_capacity(callee.contracts.ensures.len());
    for (ensure_index, ensure) in callee.contracts.ensures.iter().enumerate() {
        let raw = encode_vir_contract_expr(&callee_context, ensure).map_err(|source| {
            expression_error(caller, format!("call {id} ensures[{ensure_index}]"), source)
        })?;
        ensures.push(builder.substitute(
            &raw,
            &shifted_variables,
            &result_terms,
            NodeCeiling::member(builder.limits.expression_nodes_per_member),
        )?);
    }
    let binder_type = encode_vir_type(
        module.semantic_profile,
        &module.semantic_parameters,
        &callee_unit.type_decls,
        r#type,
    )
    .map_err(|source| expression_error(caller, format!("call {id} result type"), source.into()))?;
    state.env.insert(id.clone(), fresh_result);
    state.call_scopes.push(CallScope {
        binder_type,
        ensures,
        continuation_assumptions: Vec::new(),
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_loop_exit_members(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    budget: &mut GenerationBudget,
    function_member_count: &mut usize,
    pending: &mut Vec<PendingMember>,
    loop_returns: BTreeMap<usize, Vec<LoopReturnState>>,
) -> Result<(), ProgramWpError> {
    for (header, returns) in loop_returns {
        for returns in group_loop_return_states(returns) {
            let (state, results) = merge_loop_return_states(function, builder, header, returns)?;
            for (ensure_index, ensure) in function.contracts.ensures.iter().enumerate() {
                let assumptions = member_assumptions(&state).to_vec();
                let reservation = budget.begin_member(function_member_count, &assumptions)?;
                let raw = encode_vir_contract_expr(context, ensure).map_err(|source| {
                    expression_error(
                        function,
                        format!(
                            "loop {} exit ensures[{ensure_index}]",
                            function.blocks[header].label
                        ),
                        source,
                    )
                })?;
                let raw = close_unavailable_result_tautologies(&raw, &results);
                let conclusion = builder.substitute(
                    &raw,
                    &state.env,
                    &results,
                    reservation.conclusion_ceiling,
                )?;
                let conclusion = wrap_call_continuation(
                    builder,
                    &state,
                    conclusion,
                    reservation.conclusion_ceiling,
                )?;
                budget.finish_member(reservation, &conclusion)?;
                pending.push(close_pending_member(
                    function,
                    context,
                    Some(header),
                    MemberOrigin::new(header, ensure_index, 0),
                    ProgramVcMemberKind::LoopExit,
                    assumptions,
                    conclusion,
                    None,
                )?);
            }
        }
    }
    Ok(())
}

fn group_loop_return_states(returns: Vec<LoopReturnState>) -> Vec<Vec<LoopReturnState>> {
    let mut groups = Vec::<Vec<LoopReturnState>>::new();
    for state in returns {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| same_relational_scope(&group[0].state, &state.state))
        {
            group.push(state);
        } else {
            groups.push(vec![state]);
        }
    }
    groups
}

fn merge_loop_return_states(
    function: &VirFunction,
    builder: &TermBuilder,
    header: usize,
    mut returns: Vec<LoopReturnState>,
) -> Result<(SymbolicState, BTreeMap<u32, MpkExprTerm>), ProgramWpError> {
    if returns.is_empty() {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop exit has no return state".to_owned(),
        });
    }
    returns.sort_by_key(|state| state.block_index);
    if returns
        .iter()
        .any(|state| state.state.origin_header != Some(header))
    {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop exit return has the wrong cutpoint scope".to_owned(),
        });
    }
    if returns.len() == 1 {
        let state = returns.pop().ok_or_else(|| ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop exit return disappeared".to_owned(),
        })?;
        return Ok((state.state, state.results));
    }

    let common_length = returns
        .iter()
        .map(|state| state.state.assumptions.len())
        .min()
        .unwrap_or(0);
    let common_length = (0..common_length)
        .take_while(|index| {
            let expected = &returns[0].state.assumptions[*index];
            returns[1..]
                .iter()
                .all(|state| state.state.assumptions[*index] == *expected)
        })
        .count();
    let selectors = returns
        .iter()
        .map(|state| builder.conjoin(&state.state.assumptions[common_length..]))
        .collect::<Result<Vec<_>, _>>()?;
    let mut assumptions = returns[0].state.assumptions[..common_length].to_vec();
    let reach = builder.disjoin(&selectors)?;
    if !is_true(&reach) {
        push_assumption(&mut assumptions, reach, builder.limits)?;
    }

    let env_keys = returns[0]
        .state
        .env
        .keys()
        .filter(|key| {
            returns[1..]
                .iter()
                .all(|state| state.state.env.contains_key(*key))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut env = BTreeMap::new();
    for key in env_keys {
        let values = returns
            .iter()
            .map(|state| state.state.env[&key].clone())
            .collect::<Vec<_>>();
        env.insert(key, builder.select(&selectors, &values)?);
    }

    let result_keys = returns[0].results.keys().copied().collect::<Vec<_>>();
    if returns[1..]
        .iter()
        .any(|state| state.results.keys().copied().collect::<Vec<_>>() != result_keys)
    {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop exits have mismatched return results".to_owned(),
        });
    }
    let mut results = BTreeMap::new();
    for key in result_keys {
        let values = returns
            .iter()
            .map(|state| state.results[&key].clone())
            .collect::<Vec<_>>();
        results.insert(key, builder.select(&selectors, &values)?);
    }
    Ok((
        SymbolicState {
            env,
            assumptions,
            outer_assumptions: returns[0].state.outer_assumptions.clone(),
            call_scopes: returns[0].state.call_scopes.clone(),
            origin_header: Some(header),
        },
        results,
    ))
}

#[allow(clippy::too_many_arguments)]
fn generate_loop_nonnegative_members(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    budget: &mut GenerationBudget,
    function_member_count: &mut usize,
    pending: &mut Vec<PendingMember>,
    header: usize,
    contract: &VirLoopContract,
    runtime: &LoopRuntime,
    state: &SymbolicState,
) -> Result<(), ProgramWpError> {
    let variants = runtime.variants(function, header)?;
    if variants.len() != contract.decreases.len() {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: format!("loop {} decrease count changed", contract.header),
        });
    }
    for (decrease_index, variant) in variants.iter().enumerate() {
        if !variant.signed {
            continue;
        }
        let assumptions = member_assumptions(state).to_vec();
        let reservation = budget.begin_member(function_member_count, &assumptions)?;
        let conclusion = builder.apply_with_ceiling(
            &bitvec_function(variant.width, "sge"),
            vec![
                variant.before.clone(),
                MpkExprTerm::BitVecLiteral {
                    value: "0".to_owned(),
                    width: variant.width,
                    signed: true,
                },
            ],
            reservation.conclusion_ceiling,
        )?;
        let conclusion =
            wrap_call_continuation(builder, state, conclusion, reservation.conclusion_ceiling)?;
        budget.finish_member(reservation, &conclusion)?;
        pending.push(close_pending_member(
            function,
            context,
            Some(header),
            MemberOrigin::new(header, decrease_index, 0),
            ProgramVcMemberKind::LoopDecreases,
            assumptions,
            conclusion,
            None,
        )?);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_loop_backedge_members(
    function: &VirFunction,
    context: &ProgramExprContext,
    builder: &TermBuilder,
    budget: &mut GenerationBudget,
    function_member_count: &mut usize,
    pending: &mut Vec<PendingMember>,
    analysis: &LoopAnalysis,
    runtime: &LoopRuntime,
    backedge_source: usize,
    header: usize,
    state: &SymbolicState,
) -> Result<(), ProgramWpError> {
    if state.origin_header != Some(header) {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop backedge escaped its cutpoint scope".to_owned(),
        });
    }
    let contract =
        analysis
            .contract(header, function)
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: "loop backedge targets an uncontracted header".to_owned(),
            })?;
    let backedge_rank = analysis
        .backedge_rank(header, backedge_source)
        .ok_or_else(|| ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "loop backedge has no canonical rank".to_owned(),
        })?;

    for (invariant_index, invariant) in contract.invariants.iter().enumerate() {
        let assumptions = member_assumptions(state).to_vec();
        let reservation = budget.begin_member(function_member_count, &assumptions)?;
        let raw = encode_vir_contract_expr(context, invariant).map_err(|source| {
            expression_error(
                function,
                format!(
                    "loop {} backedge bb{backedge_source} invariant[{invariant_index}]",
                    contract.header
                ),
                source,
            )
        })?;
        let conclusion = builder.substitute(
            &raw,
            &state.env,
            &BTreeMap::new(),
            reservation.conclusion_ceiling,
        )?;
        let conclusion =
            wrap_call_continuation(builder, state, conclusion, reservation.conclusion_ceiling)?;
        budget.finish_member(reservation, &conclusion)?;
        pending.push(close_pending_member(
            function,
            context,
            Some(header),
            MemberOrigin::new(header, backedge_source, invariant_index),
            ProgramVcMemberKind::LoopPreservation,
            assumptions,
            conclusion,
            None,
        )?);
    }

    let variants = runtime.variants(function, header)?;
    for (decrease_index, (expression, variant)) in
        contract.decreases.iter().zip(variants).enumerate()
    {
        let assumptions = member_assumptions(state).to_vec();
        let reservation = budget.begin_member(function_member_count, &assumptions)?;
        let raw = encode_vir_contract_expr(context, expression).map_err(|source| {
            expression_error(
                function,
                format!(
                    "loop {} backedge bb{backedge_source} decreases[{decrease_index}]",
                    contract.header
                ),
                source,
            )
        })?;
        let after = builder.substitute(
            &raw,
            &state.env,
            &BTreeMap::new(),
            reservation.conclusion_ceiling,
        )?;
        let conclusion = builder.apply_with_ceiling(
            &bitvec_function(variant.width, if variant.signed { "slt" } else { "ult" }),
            vec![after, variant.before.clone()],
            reservation.conclusion_ceiling,
        )?;
        let conclusion =
            wrap_call_continuation(builder, state, conclusion, reservation.conclusion_ceiling)?;
        budget.finish_member(reservation, &conclusion)?;
        let phase = backedge_rank
            .checked_add(usize::from(variant.signed))
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "loop decrease origin".to_owned(),
            })?;
        pending.push(close_pending_member(
            function,
            context,
            Some(header),
            MemberOrigin::new(header, decrease_index, phase),
            ProgramVcMemberKind::LoopDecreases,
            assumptions,
            conclusion,
            None,
        )?);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn close_pending_member(
    function: &VirFunction,
    context: &ProgramExprContext,
    binder_header: Option<usize>,
    origin: MemberOrigin,
    kind: ProgramVcMemberKind,
    assumptions: Vec<MpkExprTerm>,
    conclusion: MpkExprTerm,
    safety_evidence: Option<SafetyEvidenceRoute>,
) -> Result<PendingMember, ProgramWpError> {
    let parameter_ids = function
        .params
        .iter()
        .map(|binding| binding.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut referenced = BTreeSet::new();
    for assumption in &assumptions {
        collect_term_variables(assumption, &mut referenced);
    }
    collect_term_variables(&conclusion, &mut referenced);
    let non_parameters = referenced
        .iter()
        .filter(|name| !parameter_ids.contains(name.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();

    let candidates = binder_header
        .map(|header| {
            function
                .locals
                .iter()
                .chain(function.blocks[header].parameters.iter())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let candidate_ids = candidates
        .iter()
        .map(|binding| binding.id.as_str())
        .collect::<BTreeSet<_>>();
    if let Some(name) = non_parameters
        .iter()
        .find(|name| !candidate_ids.contains(name.as_str()))
    {
        return Err(ProgramWpError::UnclosedValue { name: name.clone() });
    }
    let selected = candidates
        .into_iter()
        .filter(|binding| non_parameters.contains(&binding.id))
        .collect::<Vec<_>>();
    let binder_count = selected.len();
    let mut replacements = BTreeMap::new();
    let mut local_binders = Vec::with_capacity(binder_count);
    for (index, binding) in selected.into_iter().enumerate() {
        let debruijn = binder_count
            .checked_sub(index + 1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| ProgramWpError::CounterOverflow {
                context: "loop local-binder index".to_owned(),
            })?;
        replacements.insert(binding.id.clone(), debruijn);
        local_binders.push(
            encode_vir_type(
                context.profile(),
                context.parameters(),
                context.declarations(),
                &binding.r#type,
            )
            .map_err(|source| {
                expression_error(function, "loop local-binder type", source.into())
            })?,
        );
    }
    let assumptions = assumptions
        .iter()
        .map(|term| bind_term(term, &replacements))
        .collect::<Result<Vec<_>, _>>()?;
    let conclusion = bind_term(&conclusion, &replacements)?;
    Ok(PendingMember {
        origin,
        kind,
        local_binders,
        assumptions,
        conclusion,
        safety_evidence,
    })
}

fn collect_term_variables(term: &MpkExprTerm, output: &mut BTreeSet<String>) {
    match term {
        MpkExprTerm::Var { name } => {
            output.insert(name.clone());
        }
        MpkExprTerm::Apply { args, .. } => {
            for argument in args {
                collect_term_variables(argument, output);
            }
        }
        MpkExprTerm::Convert { value, .. } => collect_term_variables(value, output),
        MpkExprTerm::Forall { body, .. } => collect_term_variables(body, output),
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Result { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => {}
    }
}

fn bind_term(
    term: &MpkExprTerm,
    replacements: &BTreeMap<String, u32>,
) -> Result<MpkExprTerm, ProgramWpError> {
    bind_term_at_depth(term, replacements, 0)
}

fn bind_term_at_depth(
    term: &MpkExprTerm,
    replacements: &BTreeMap<String, u32>,
    inline_depth: u32,
) -> Result<MpkExprTerm, ProgramWpError> {
    match term {
        MpkExprTerm::Var { name } => replacements.get(name).map_or_else(
            || Ok(term.clone()),
            |index| {
                index
                    .checked_add(inline_depth)
                    .map(|index| MpkExprTerm::Bound { index })
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "member-local de Bruijn shift".to_owned(),
                    })
            },
        ),
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => Ok(term.clone()),
        MpkExprTerm::Result { index } => Err(ProgramWpError::UnclosedResult { index: *index }),
        MpkExprTerm::Apply { function, args } => Ok(MpkExprTerm::Apply {
            function: function.clone(),
            args: args
                .iter()
                .map(|argument| bind_term_at_depth(argument, replacements, inline_depth))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        MpkExprTerm::Convert { value, target } => Ok(MpkExprTerm::Convert {
            value: Box::new(bind_term_at_depth(value, replacements, inline_depth)?),
            target: target.clone(),
        }),
        MpkExprTerm::Forall { binder_type, body } => Ok(MpkExprTerm::Forall {
            binder_type: binder_type.clone(),
            body: Box::new(bind_term_at_depth(
                body,
                replacements,
                inline_depth
                    .checked_add(1)
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "inline binder depth".to_owned(),
                    })?,
            )?),
        }),
    }
}

fn shift_bound_indices(
    term: &MpkExprTerm,
    amount: u32,
    cutoff: u32,
) -> Result<MpkExprTerm, ProgramWpError> {
    match term {
        MpkExprTerm::Bound { index } if *index >= cutoff => Ok(MpkExprTerm::Bound {
            index: index
                .checked_add(amount)
                .ok_or_else(|| ProgramWpError::CounterOverflow {
                    context: "de Bruijn index shift".to_owned(),
                })?,
        }),
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Var { .. }
        | MpkExprTerm::Result { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => Ok(term.clone()),
        MpkExprTerm::Apply { function, args } => Ok(MpkExprTerm::Apply {
            function: function.clone(),
            args: args
                .iter()
                .map(|argument| shift_bound_indices(argument, amount, cutoff))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        MpkExprTerm::Convert { value, target } => Ok(MpkExprTerm::Convert {
            value: Box::new(shift_bound_indices(value, amount, cutoff)?),
            target: target.clone(),
        }),
        MpkExprTerm::Forall { binder_type, body } => Ok(MpkExprTerm::Forall {
            binder_type: binder_type.clone(),
            body: Box::new(shift_bound_indices(
                body,
                amount,
                cutoff
                    .checked_add(1)
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "de Bruijn cutoff shift".to_owned(),
                    })?,
            )?),
        }),
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

fn finalize_members(
    function: &VirFunction,
    mut pending: Vec<PendingMember>,
) -> Result<Vec<ProgramVcMember>, ProgramWpError> {
    pending.sort_by_key(|member| (member.kind, member.origin));
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
            local_binders: member.local_binders,
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
        outer_assumptions: state.outer_assumptions.clone(),
        call_scopes: state.call_scopes.clone(),
        origin_header: state.origin_header,
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
    if worklist
        .delivered_edges
        .insert((predecessor_index, edge_index, target_index))
    {
        worklist.remaining_predecessors[target_index] = worklist.remaining_predecessors
            [target_index]
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
    }
    Ok(())
}

fn merge_incoming_states(
    function: &VirFunction,
    builder: &TermBuilder,
    mut incoming: Vec<IncomingState>,
) -> Result<Vec<SymbolicState>, ProgramWpError> {
    if incoming.is_empty() {
        return Err(ProgramWpError::InvalidGraph {
            function_id: function.id.clone(),
            detail: "reachable non-entry block has no predecessor state".to_owned(),
        });
    }
    incoming.sort_by_key(|edge| (edge.predecessor_index, edge.edge_index));
    let mut groups = Vec::<Vec<IncomingState>>::new();
    for edge in incoming {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| same_relational_scope(&group[0].state, &edge.state))
        {
            group.push(edge);
        } else {
            groups.push(vec![edge]);
        }
    }
    groups
        .into_iter()
        .map(|group| merge_incoming_group(function, builder, group))
        .collect()
}

fn same_relational_scope(lhs: &SymbolicState, rhs: &SymbolicState) -> bool {
    lhs.origin_header == rhs.origin_header
        && lhs.outer_assumptions == rhs.outer_assumptions
        && lhs.call_scopes == rhs.call_scopes
}

fn merge_incoming_group(
    function: &VirFunction,
    builder: &TermBuilder,
    mut incoming: Vec<IncomingState>,
) -> Result<SymbolicState, ProgramWpError> {
    if incoming.len() == 1 {
        return incoming
            .pop()
            .map(|edge| edge.state)
            .ok_or_else(|| ProgramWpError::InvalidGraph {
                function_id: function.id.clone(),
                detail: "incoming state disappeared".to_owned(),
            });
    }
    let origin_header = incoming[0].state.origin_header;
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
    Ok(SymbolicState {
        env,
        assumptions,
        outer_assumptions: incoming[0].state.outer_assumptions.clone(),
        call_scopes: incoming[0].state.call_scopes.clone(),
        origin_header,
    })
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

    fn conjoin_exact(
        self,
        values: &[MpkExprTerm],
        ceiling: NodeCeiling,
    ) -> Result<MpkExprTerm, ProgramWpError> {
        match values {
            [] => Ok(MpkExprTerm::Constant {
                name: STD_BOOL_TRUE.to_owned(),
            }),
            [value] => Ok(value.clone()),
            values => {
                let split = values.len() / 2;
                let left = self.conjoin_exact(&values[..split], ceiling)?;
                let right = self.conjoin_exact(&values[split..], ceiling)?;
                self.apply_with_ceiling(STD_BOOL_AND, vec![left, right], ceiling)
            }
        }
    }

    fn imply(
        self,
        antecedent: MpkExprTerm,
        consequent: MpkExprTerm,
        ceiling: NodeCeiling,
    ) -> Result<MpkExprTerm, ProgramWpError> {
        self.apply_with_ceiling("Std.Logic.Imp", vec![antecedent, consequent], ceiling)
    }

    fn forall(
        self,
        binder_type: MpkTypeTerm,
        body: MpkExprTerm,
        ceiling: NodeCeiling,
    ) -> Result<MpkExprTerm, ProgramWpError> {
        let term = MpkExprTerm::Forall {
            binder_type,
            body: Box::new(body),
        };
        measure_term(&term, ceiling, self.limits.member_expression_depth)?;
        Ok(term)
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
        self.apply_with_ceiling(
            function,
            args,
            NodeCeiling::member(self.limits.expression_nodes_per_member),
        )
    }

    fn apply_with_ceiling(
        self,
        function: &str,
        args: Vec<MpkExprTerm>,
        ceiling: NodeCeiling,
    ) -> Result<MpkExprTerm, ProgramWpError> {
        let term = MpkExprTerm::Apply {
            function: function.to_owned(),
            args,
        };
        measure_term(&term, ceiling, self.limits.member_expression_depth)?;
        Ok(term)
    }
}

fn is_true(term: &MpkExprTerm) -> bool {
    matches!(term, MpkExprTerm::Constant { name } if name == STD_BOOL_TRUE)
}

fn is_false(term: &MpkExprTerm) -> bool {
    matches!(term, MpkExprTerm::Constant { name } if name == STD_BOOL_FALSE)
}

fn contract_result_is_required(term: &VirContractExpr, expected: u32) -> bool {
    match term {
        VirContractExpr::Result(reference) => reference.result == expected,
        VirContractExpr::Binary(expression)
            if matches!(
                expression.op,
                VirBinaryOperator::Eq | VirBinaryOperator::NotEq
            ) && expression.lhs == expression.rhs =>
        {
            false
        }
        VirContractExpr::Unary(expression) => {
            contract_result_is_required(&expression.value, expected)
        }
        VirContractExpr::Nary(expression) => expression
            .args
            .iter()
            .any(|argument| contract_result_is_required(argument, expected)),
        VirContractExpr::Binary(expression) => {
            contract_result_is_required(&expression.lhs, expected)
                || contract_result_is_required(&expression.rhs, expected)
        }
        VirContractExpr::Convert(expression) => {
            contract_result_is_required(&expression.value, expected)
        }
        VirContractExpr::Variable(_)
        | VirContractExpr::Boolean(_)
        | VirContractExpr::Integer(_) => false,
    }
}

fn close_unavailable_result_tautologies(
    term: &MpkExprTerm,
    results: &BTreeMap<u32, MpkExprTerm>,
) -> MpkExprTerm {
    match term {
        MpkExprTerm::Apply { function, args }
            if function == STD_EQ
                && args.len() == 2
                && args[0] == args[1]
                && contains_unavailable_result(&args[0], results) =>
        {
            MpkExprTerm::bool_literal(true)
        }
        MpkExprTerm::Apply { function, args } => MpkExprTerm::Apply {
            function: function.clone(),
            args: args
                .iter()
                .map(|argument| close_unavailable_result_tautologies(argument, results))
                .collect(),
        },
        MpkExprTerm::Convert { value, target } => MpkExprTerm::Convert {
            value: Box::new(close_unavailable_result_tautologies(value, results)),
            target: target.clone(),
        },
        MpkExprTerm::Forall { binder_type, body } => MpkExprTerm::Forall {
            binder_type: binder_type.clone(),
            body: Box::new(close_unavailable_result_tautologies(body, results)),
        },
        MpkExprTerm::Var { .. }
        | MpkExprTerm::Result { .. }
        | MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => term.clone(),
    }
}

fn contains_unavailable_result(term: &MpkExprTerm, results: &BTreeMap<u32, MpkExprTerm>) -> bool {
    match term {
        MpkExprTerm::Result { index } => !results.contains_key(index),
        MpkExprTerm::Apply { args, .. } => args
            .iter()
            .any(|argument| contains_unavailable_result(argument, results)),
        MpkExprTerm::Convert { value, .. } | MpkExprTerm::Forall { body: value, .. } => {
            contains_unavailable_result(value, results)
        }
        MpkExprTerm::Var { .. }
        | MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => false,
    }
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
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => Ok(input.clone()),
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
        MpkExprTerm::Forall { binder_type, body } => {
            let variables = variables
                .iter()
                .map(|(name, value)| Ok((name.clone(), shift_bound_indices(value, 1, 0)?)))
                .collect::<Result<BTreeMap<_, _>, ProgramWpError>>()?;
            let results = results
                .iter()
                .map(|(index, value)| Ok((*index, shift_bound_indices(value, 1, 0)?)))
                .collect::<Result<BTreeMap<_, _>, ProgramWpError>>()?;
            Ok(MpkExprTerm::Forall {
                binder_type: binder_type.clone(),
                body: Box::new(substitute_unchecked(body, &variables, &results)?),
            })
        }
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
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => {
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
        MpkExprTerm::Forall { .. } => {
            let substituted = substitute_unchecked(input, variables, results)?;
            measure_term(&substituted, ceiling, depth_limit)
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
        | MpkExprTerm::Bound { .. }
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
        MpkExprTerm::Forall { binder_type, body } => {
            let body = measure_term(body, ceiling, depth_limit)?;
            let metrics = TermMetrics {
                nodes: body.nodes.checked_add(1).ok_or_else(|| {
                    ProgramWpError::CounterOverflow {
                        context: "forall expression nodes".to_owned(),
                    }
                })?,
                depth: 1_usize
                    .checked_add(body.depth.max(type_depth(binder_type)?))
                    .ok_or_else(|| ProgramWpError::CounterOverflow {
                        context: "forall expression depth".to_owned(),
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
    Call(CallWpError),
    CallSignature {
        caller: String,
        callee: String,
    },
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
            Self::Call(_) => "VC_PROGRAM_CALL_GRAPH",
            Self::CallSignature { .. } => "VC_PROGRAM_CALL_SIGNATURE",
            Self::Expression { .. } => "VC_PROGRAM_EXPRESSION",
            Self::Safety { .. } => "VC_PROGRAM_SAFETY",
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
            Self::Call(error) => write!(formatter, "program call analysis failed: {error}"),
            Self::CallSignature { caller, callee } => {
                write!(
                    formatter,
                    "static-call signature mismatch for {caller} -> {callee}"
                )
            }
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
            Self::Call(error) => Some(error),
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
