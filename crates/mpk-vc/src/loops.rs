//! Loop-invariant VC generation for explicitly annotated GIR loops.
//!
//! VC-007 handles the first loop CFG shape: a preheader path reaching an
//! annotated header block, a header `Branch` whose true edge enters the loop
//! body and false edge exits, a body path that jumps back to the header, and an
//! exit path that returns. More complex loop CFGs are rejected fail-closed.

use std::collections::{BTreeMap, BTreeSet};

use crate::expr_encode::{ExprEncoder, MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_NOT};
use crate::gir::{
    GirBlock, GirContractExpr, GirFunction, GirLoopContract, GirModule, GirTerminator,
    GirTerminatorKind, GirType, GirTypeKind,
};
use crate::vc::{VcModule, VcObligation, VcObligationKind};
use crate::wp::{
    encode_requires, initial_environment, substitute_term, validate_contract_references,
    validate_value_reference, WpError, WpGenerator,
};

pub fn generate_loop_vcs(input: &GirModule) -> Result<VcModule, WpError> {
    LoopVcGenerator::new().generate_module(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LoopVcGenerator;

impl LoopVcGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_module(self, input: &GirModule) -> Result<VcModule, WpError> {
        let mut output = VcModule::empty_for_gir(input);
        for package in &input.packages {
            for function in &package.functions {
                output.obligations.extend(self.generate_function(function)?);
            }
        }
        Ok(output)
    }

    pub fn generate_function(self, function: &GirFunction) -> Result<Vec<VcObligation>, WpError> {
        reject_common_unsupported(function)?;
        if function.contracts.loops.is_empty() {
            return Ok(Vec::new());
        }
        if function.contracts.ensures.is_empty() {
            return Err(WpError::MissingPostcondition {
                function_id: function.id.clone(),
            });
        }
        if function.blocks.is_empty() {
            return Err(WpError::UnsupportedBlockCount {
                function_id: function.id.clone(),
                block_count: 0,
            });
        }

        let encoder = ExprEncoder::for_function(function);
        let initial_env = initial_environment(function);
        let requires = encode_requires(function, &encoder, &initial_env)?;
        let blocks = block_map(function);
        let mut output = Vec::new();
        for loop_contract in &function.contracts.loops {
            output.extend(self.generate_loop(
                function,
                &blocks,
                &encoder,
                &initial_env,
                &requires,
                loop_contract,
            )?);
        }
        Ok(output)
    }

    fn generate_loop(
        self,
        function: &GirFunction,
        blocks: &BTreeMap<String, &GirBlock>,
        encoder: &ExprEncoder,
        initial_env: &BTreeMap<String, MpkExprTerm>,
        requires: &[MpkExprTerm],
        loop_contract: &GirLoopContract,
    ) -> Result<Vec<VcObligation>, WpError> {
        if loop_contract.invariants.is_empty() {
            return Err(WpError::MissingLoopInvariant {
                function_id: function.id.clone(),
                block_label: loop_contract.block_id.clone(),
            });
        }

        let header = blocks
            .get(&loop_contract.block_id)
            .copied()
            .ok_or_else(|| WpError::UnknownBlockLabel {
                function_id: function.id.clone(),
                context: "loop contract".to_owned(),
                block_label: loop_contract.block_id.clone(),
            })?;
        ensure_no_block_parameters(function, header)?;
        if header.terminator.kind != GirTerminatorKind::Branch {
            return Err(unsupported_loop_shape(
                function,
                &loop_contract.block_id,
                "loop header must end in Branch",
            ));
        }
        ensure_branch_terminator_shape(function, header, &header.terminator)?;

        let header_env = execute_preheader_path(
            function,
            blocks,
            encoder,
            &function.blocks[0].label,
            &loop_contract.block_id,
            initial_env.clone(),
        )?;
        let condition_env =
            WpGenerator::new().execute_block(function, header, encoder, initial_env.clone())?;
        let condition = encode_branch_condition(function, header, encoder, &condition_env)?;

        let body_label =
            header
                .terminator
                .then_label
                .as_deref()
                .ok_or_else(|| WpError::MissingBranchLabel {
                    function_id: function.id.clone(),
                    block_label: header.label.clone(),
                    label_kind: "then",
                })?;
        let exit_label =
            header
                .terminator
                .else_label
                .as_deref()
                .ok_or_else(|| WpError::MissingBranchLabel {
                    function_id: function.id.clone(),
                    block_label: header.label.clone(),
                    label_kind: "else",
                })?;

        let body_env = execute_body_path_to_header(
            function,
            blocks,
            encoder,
            body_label,
            &loop_contract.block_id,
            condition_env.clone(),
        )?;
        let (exit_env, result_terms) = execute_exit_path_to_return(
            function,
            blocks,
            encoder,
            exit_label,
            condition_env.clone(),
        )?;

        let mut output = Vec::new();
        output.extend(initial_invariant_obligations(
            function,
            encoder,
            initial_env,
            loop_contract,
            requires,
            &header_env,
        )?);
        output.extend(preservation_obligations(
            function,
            encoder,
            initial_env,
            loop_contract,
            &condition_env,
            &condition,
            &body_env,
        )?);
        output.extend(exit_obligations(
            function,
            encoder,
            initial_env,
            loop_contract,
            &condition_env,
            &condition,
            &exit_env,
            &result_terms,
        )?);
        output.extend(decreases_obligations(
            function,
            encoder,
            initial_env,
            loop_contract,
            &condition_env,
            &condition,
            &body_env,
        )?);
        Ok(output)
    }
}

fn reject_common_unsupported(function: &GirFunction) -> Result<(), WpError> {
    if !function.rejected_features.is_empty() {
        return Err(WpError::FunctionHasRejectedFeatures {
            function_id: function.id.clone(),
            rejected_feature_count: function.rejected_features.len(),
        });
    }
    if !function.contracts.modifies.is_empty() {
        return Err(WpError::NonEmptyModifies {
            function_id: function.id.clone(),
            modifies: function.contracts.modifies.clone(),
        });
    }
    Ok(())
}

fn initial_invariant_obligations(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    requires: &[MpkExprTerm],
    header_env: &BTreeMap<String, MpkExprTerm>,
) -> Result<Vec<VcObligation>, WpError> {
    loop_contract
        .invariants
        .iter()
        .enumerate()
        .map(|(index, invariant)| {
            Ok(VcObligation {
                id: format!(
                    "{}.loop.{}.initial.inv{index}",
                    function.id, loop_contract.block_id
                ),
                function_id: function.id.clone(),
                kind: VcObligationKind::LoopInvariantInitial,
                assumptions: requires.to_vec(),
                conclusion: encode_loop_expr(
                    function,
                    encoder,
                    contract_env,
                    invariant,
                    false,
                    format!("loop {} invariant[{index}]", loop_contract.block_id),
                    header_env,
                    &BTreeMap::new(),
                )?,
            })
        })
        .collect()
}

fn preservation_obligations(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    condition_env: &BTreeMap<String, MpkExprTerm>,
    condition: &MpkExprTerm,
    body_env: &BTreeMap<String, MpkExprTerm>,
) -> Result<Vec<VcObligation>, WpError> {
    let assumptions = invariant_assumptions(
        function,
        encoder,
        contract_env,
        loop_contract,
        condition_env,
    )?
    .into_iter()
    .chain([condition.clone()])
    .collect::<Vec<_>>();
    loop_contract
        .invariants
        .iter()
        .enumerate()
        .map(|(index, invariant)| {
            Ok(VcObligation {
                id: format!(
                    "{}.loop.{}.preservation.inv{index}",
                    function.id, loop_contract.block_id
                ),
                function_id: function.id.clone(),
                kind: VcObligationKind::LoopInvariantPreservation,
                assumptions: assumptions.clone(),
                conclusion: encode_loop_expr(
                    function,
                    encoder,
                    contract_env,
                    invariant,
                    false,
                    format!(
                        "loop {} preservation invariant[{index}]",
                        loop_contract.block_id
                    ),
                    body_env,
                    &BTreeMap::new(),
                )?,
            })
        })
        .collect()
}

fn exit_obligations(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    condition_env: &BTreeMap<String, MpkExprTerm>,
    condition: &MpkExprTerm,
    exit_env: &BTreeMap<String, MpkExprTerm>,
    result_terms: &BTreeMap<u32, MpkExprTerm>,
) -> Result<Vec<VcObligation>, WpError> {
    let assumptions = invariant_assumptions(
        function,
        encoder,
        contract_env,
        loop_contract,
        condition_env,
    )?
    .into_iter()
    .chain([MpkExprTerm::apply(STD_BOOL_NOT, [condition.clone()])])
    .collect::<Vec<_>>();
    function
        .contracts
        .ensures
        .iter()
        .enumerate()
        .map(|(index, ensure)| {
            Ok(VcObligation {
                id: format!(
                    "{}.loop.{}.exit.post{index}",
                    function.id, loop_contract.block_id
                ),
                function_id: function.id.clone(),
                kind: VcObligationKind::LoopExit,
                assumptions: assumptions.clone(),
                conclusion: encode_loop_expr(
                    function,
                    encoder,
                    contract_env,
                    ensure,
                    true,
                    format!("loop {} exit ensures[{index}]", loop_contract.block_id),
                    exit_env,
                    result_terms,
                )?,
            })
        })
        .collect()
}

fn decreases_obligations(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    condition_env: &BTreeMap<String, MpkExprTerm>,
    condition: &MpkExprTerm,
    body_env: &BTreeMap<String, MpkExprTerm>,
) -> Result<Vec<VcObligation>, WpError> {
    let assumptions = invariant_assumptions(
        function,
        encoder,
        contract_env,
        loop_contract,
        condition_env,
    )?
    .into_iter()
    .chain([condition.clone()])
    .collect::<Vec<_>>();
    let mut output = Vec::new();
    for (index, decreases) in loop_contract.decreases.iter().enumerate() {
        let shape = infer_variant_shape(function, contract_env, loop_contract, index, decreases)?;
        let before = encode_loop_expr(
            function,
            encoder,
            contract_env,
            decreases,
            false,
            format!("loop {} decreases[{index}]", loop_contract.block_id),
            condition_env,
            &BTreeMap::new(),
        )?;
        let after = encode_loop_expr(
            function,
            encoder,
            contract_env,
            decreases,
            false,
            format!(
                "loop {} decreases[{index}] after body",
                loop_contract.block_id
            ),
            body_env,
            &BTreeMap::new(),
        )?;

        if shape.signed {
            output.push(VcObligation {
                id: format!(
                    "{}.loop.{}.decreases{index}.nonnegative",
                    function.id, loop_contract.block_id
                ),
                function_id: function.id.clone(),
                kind: VcObligationKind::Decreases,
                assumptions: assumptions.clone(),
                conclusion: MpkExprTerm::apply(
                    bitvec_function(shape.width, "sge"),
                    [before.clone(), bitvec_literal(0, shape)],
                ),
            });
        }
        output.push(VcObligation {
            id: format!(
                "{}.loop.{}.decreases{index}.strict",
                function.id, loop_contract.block_id
            ),
            function_id: function.id.clone(),
            kind: VcObligationKind::Decreases,
            assumptions: assumptions.clone(),
            conclusion: MpkExprTerm::apply(
                bitvec_function(shape.width, if shape.signed { "slt" } else { "ult" }),
                [after, before],
            ),
        });
    }
    Ok(output)
}

fn invariant_assumptions(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<Vec<MpkExprTerm>, WpError> {
    loop_contract
        .invariants
        .iter()
        .enumerate()
        .map(|(index, invariant)| {
            encode_loop_expr(
                function,
                encoder,
                contract_env,
                invariant,
                false,
                format!("loop {} invariant[{index}]", loop_contract.block_id),
                env,
                &BTreeMap::new(),
            )
        })
        .collect()
}

fn execute_preheader_path(
    function: &GirFunction,
    blocks: &BTreeMap<String, &GirBlock>,
    encoder: &ExprEncoder,
    start_label: &str,
    header_label: &str,
    mut env: BTreeMap<String, MpkExprTerm>,
) -> Result<BTreeMap<String, MpkExprTerm>, WpError> {
    if start_label == header_label {
        return Ok(env);
    }
    let mut label = start_label.to_owned();
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(label.clone()) {
            return Err(unsupported_loop_shape(
                function,
                header_label,
                format!("preheader path cycles at {label:?}"),
            ));
        }
        let block = block_by_label(function, blocks, "loop preheader", &label)?;
        ensure_no_block_parameters(function, block)?;
        env = WpGenerator::new().execute_block(function, block, encoder, env)?;
        match block.terminator.kind {
            GirTerminatorKind::Jump => {
                ensure_jump_terminator_shape(function, block, &block.terminator)?;
                let next = block
                    .terminator
                    .label
                    .as_deref()
                    .expect("jump shape validates label");
                if next == header_label {
                    return Ok(env);
                }
                label = next.to_owned();
            }
            kind => {
                return Err(unsupported_loop_shape(
                    function,
                    header_label,
                    format!("preheader block {:?} ends in {kind:?}", block.label),
                ));
            }
        }
    }
}

fn execute_body_path_to_header(
    function: &GirFunction,
    blocks: &BTreeMap<String, &GirBlock>,
    encoder: &ExprEncoder,
    start_label: &str,
    header_label: &str,
    mut env: BTreeMap<String, MpkExprTerm>,
) -> Result<BTreeMap<String, MpkExprTerm>, WpError> {
    let mut label = start_label.to_owned();
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(label.clone()) {
            return Err(unsupported_loop_shape(
                function,
                header_label,
                format!("body path cycles at {label:?} before returning to header"),
            ));
        }
        let block = block_by_label(function, blocks, "loop body", &label)?;
        ensure_no_block_parameters(function, block)?;
        env = WpGenerator::new().execute_block(function, block, encoder, env)?;
        match block.terminator.kind {
            GirTerminatorKind::Jump => {
                ensure_jump_terminator_shape(function, block, &block.terminator)?;
                let next = block
                    .terminator
                    .label
                    .as_deref()
                    .expect("jump shape validates label");
                if next == header_label {
                    return Ok(env);
                }
                label = next.to_owned();
            }
            kind => {
                return Err(unsupported_loop_shape(
                    function,
                    header_label,
                    format!("loop body block {:?} ends in {kind:?}", block.label),
                ));
            }
        }
    }
}

fn execute_exit_path_to_return(
    function: &GirFunction,
    blocks: &BTreeMap<String, &GirBlock>,
    encoder: &ExprEncoder,
    start_label: &str,
    mut env: BTreeMap<String, MpkExprTerm>,
) -> Result<(BTreeMap<String, MpkExprTerm>, BTreeMap<u32, MpkExprTerm>), WpError> {
    let mut label = start_label.to_owned();
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(label.clone()) {
            return Err(unsupported_loop_shape(
                function,
                start_label,
                format!("exit path cycles at {label:?}"),
            ));
        }
        let block = block_by_label(function, blocks, "loop exit", &label)?;
        ensure_no_block_parameters(function, block)?;
        env = WpGenerator::new().execute_block(function, block, encoder, env)?;
        match block.terminator.kind {
            GirTerminatorKind::Return => {
                ensure_return_terminator_shape(function, block, &block.terminator)?;
                let result_terms = encode_return_values(function, encoder, &env, block)?;
                return Ok((env, result_terms));
            }
            GirTerminatorKind::Jump => {
                ensure_jump_terminator_shape(function, block, &block.terminator)?;
                label = block
                    .terminator
                    .label
                    .as_ref()
                    .expect("jump shape validates label")
                    .clone();
            }
            kind => {
                return Err(unsupported_loop_shape(
                    function,
                    start_label,
                    format!("exit block {:?} ends in {kind:?}", block.label),
                ));
            }
        }
    }
}

fn encode_return_values(
    function: &GirFunction,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
    block: &GirBlock,
) -> Result<BTreeMap<u32, MpkExprTerm>, WpError> {
    if block.terminator.values.len() != function.results.len() {
        return Err(WpError::ReturnArityMismatch {
            function_id: function.id.clone(),
            expected: function.results.len(),
            actual: block.terminator.values.len(),
        });
    }
    block
        .terminator
        .values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            validate_value_reference(function, value, env, format!("return[{index}]"))?;
            let encoded = encoder
                .encode_value(value)
                .map_err(|source| WpError::Expression {
                    function_id: function.id.clone(),
                    context: format!("return[{index}]"),
                    source,
                })?;
            let index = u32::try_from(index).map_err(|_| WpError::ReturnIndexOverflow {
                function_id: function.id.clone(),
                index,
            })?;
            Ok((index, substitute_term(&encoded, env, &BTreeMap::new())))
        })
        .collect()
}

fn encode_branch_condition(
    function: &GirFunction,
    block: &GirBlock,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<MpkExprTerm, WpError> {
    let condition =
        block
            .terminator
            .cond
            .as_ref()
            .ok_or_else(|| WpError::MissingBranchCondition {
                function_id: function.id.clone(),
                block_label: block.label.clone(),
            })?;
    validate_value_reference(function, condition, env, "loop branch condition")?;
    let encoded = encoder
        .encode_value(condition)
        .map_err(|source| WpError::Expression {
            function_id: function.id.clone(),
            context: "loop branch condition".to_owned(),
            source,
        })?;
    Ok(substitute_term(&encoded, env, &BTreeMap::new()))
}

fn encode_loop_expr(
    function: &GirFunction,
    encoder: &ExprEncoder,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    input: &GirContractExpr,
    allow_results: bool,
    context: impl Into<String>,
    env: &BTreeMap<String, MpkExprTerm>,
    result_terms: &BTreeMap<u32, MpkExprTerm>,
) -> Result<MpkExprTerm, WpError> {
    let context = context.into();
    validate_contract_references(function, input, contract_env, allow_results, "loop")?;
    let encoded = encoder
        .encode_contract_expr(input)
        .map_err(|source| WpError::Expression {
            function_id: function.id.clone(),
            context,
            source,
        })?;
    Ok(substitute_term(&encoded, env, result_terms))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BitVectorShape {
    width: u32,
    signed: bool,
}

fn infer_variant_shape(
    function: &GirFunction,
    contract_env: &BTreeMap<String, MpkExprTerm>,
    loop_contract: &GirLoopContract,
    variant_index: usize,
    input: &GirContractExpr,
) -> Result<BitVectorShape, WpError> {
    if let Some(name) = &input.var {
        if !contract_env.contains_key(name) {
            return Err(unsupported_variant(
                function,
                loop_contract,
                variant_index,
                format!("unknown variable {name:?}"),
            ));
        }
        let binding = function
            .params
            .iter()
            .chain(function.locals.iter())
            .find(|binding| binding.name == *name)
            .ok_or_else(|| {
                unsupported_variant(
                    function,
                    loop_contract,
                    variant_index,
                    format!("missing type for variable {name:?}"),
                )
            })?;
        return bitvector_shape_from_type(function, loop_contract, variant_index, &binding.r#type);
    }
    if let Some(literal) = &input.int {
        return Ok(BitVectorShape {
            width: literal.width,
            signed: literal.signed,
        });
    }
    if let Some(op) = input.op.as_deref() {
        if op == "convert" {
            let target = input.r#type.as_ref().ok_or_else(|| {
                unsupported_variant(
                    function,
                    loop_contract,
                    variant_index,
                    "convert variant is missing target type",
                )
            })?;
            return bitvector_shape_from_type(function, loop_contract, variant_index, target);
        }
        if matches!(
            op,
            "bv_add" | "bv_sub" | "bv_mul" | "bv_and" | "bv_or" | "bv_xor"
        ) {
            let lhs = input.lhs.as_deref().ok_or_else(|| {
                unsupported_variant(
                    function,
                    loop_contract,
                    variant_index,
                    format!("{op} variant is missing lhs"),
                )
            })?;
            let rhs = input.rhs.as_deref().ok_or_else(|| {
                unsupported_variant(
                    function,
                    loop_contract,
                    variant_index,
                    format!("{op} variant is missing rhs"),
                )
            })?;
            let lhs_shape =
                infer_variant_shape(function, contract_env, loop_contract, variant_index, lhs)?;
            let rhs_shape =
                infer_variant_shape(function, contract_env, loop_contract, variant_index, rhs)?;
            if lhs_shape != rhs_shape {
                return Err(unsupported_variant(
                    function,
                    loop_contract,
                    variant_index,
                    format!("{op} variant operands must have matching bitvector shape"),
                ));
            }
            return Ok(lhs_shape);
        }
    }
    Err(unsupported_variant(
        function,
        loop_contract,
        variant_index,
        "variant must be a bitvector variable, literal, or convert expression",
    ))
}

fn bitvector_shape_from_type(
    function: &GirFunction,
    loop_contract: &GirLoopContract,
    variant_index: usize,
    r#type: &GirType,
) -> Result<BitVectorShape, WpError> {
    if r#type.kind != GirTypeKind::BitVector {
        return Err(unsupported_variant(
            function,
            loop_contract,
            variant_index,
            "variant is not a bitvector",
        ));
    }
    let width = r#type.width.ok_or_else(|| {
        unsupported_variant(
            function,
            loop_contract,
            variant_index,
            "variant bitvector type is missing width",
        )
    })?;
    let signed = r#type.signed.ok_or_else(|| {
        unsupported_variant(
            function,
            loop_contract,
            variant_index,
            "variant bitvector type is missing signedness",
        )
    })?;
    Ok(BitVectorShape { width, signed })
}

fn bitvec_literal(value: u64, shape: BitVectorShape) -> MpkExprTerm {
    MpkExprTerm::BitVecLiteral {
        value: value.to_string(),
        width: shape.width,
        signed: shape.signed,
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

fn block_map(function: &GirFunction) -> BTreeMap<String, &GirBlock> {
    function
        .blocks
        .iter()
        .map(|block| (block.label.clone(), block))
        .collect()
}

fn block_by_label<'a>(
    function: &GirFunction,
    blocks: &BTreeMap<String, &'a GirBlock>,
    context: &'static str,
    label: &str,
) -> Result<&'a GirBlock, WpError> {
    blocks
        .get(label)
        .copied()
        .ok_or_else(|| WpError::UnknownBlockLabel {
            function_id: function.id.clone(),
            context: context.to_owned(),
            block_label: label.to_owned(),
        })
}

fn unsupported_loop_shape(
    function: &GirFunction,
    block_label: &str,
    reason: impl Into<String>,
) -> WpError {
    WpError::UnsupportedLoopShape {
        function_id: function.id.clone(),
        block_label: block_label.to_owned(),
        reason: reason.into(),
    }
}

fn unsupported_variant(
    function: &GirFunction,
    loop_contract: &GirLoopContract,
    variant_index: usize,
    reason: impl Into<String>,
) -> WpError {
    WpError::UnsupportedLoopVariant {
        function_id: function.id.clone(),
        block_label: loop_contract.block_id.clone(),
        variant_index,
        reason: reason.into(),
    }
}

fn ensure_no_block_parameters(function: &GirFunction, block: &GirBlock) -> Result<(), WpError> {
    if block.parameters.is_empty() {
        return Ok(());
    }
    Err(WpError::BlockParametersUnsupported {
        function_id: function.id.clone(),
        block_label: block.label.clone(),
        parameter_count: block.parameters.len(),
    })
}

fn ensure_branch_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (!terminator.values.is_empty(), "Branch cannot have values"),
        (terminator.label.is_some(), "Branch cannot have label"),
        (!terminator.args.is_empty(), "Branch cannot have args"),
        (terminator.reason.is_some(), "Branch cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn ensure_jump_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (!terminator.values.is_empty(), "Jump cannot have values"),
        (terminator.cond.is_some(), "Jump cannot have cond"),
        (terminator.label.is_none(), "Jump must have label"),
        (
            terminator.then_label.is_some(),
            "Jump cannot have then_label",
        ),
        (
            terminator.else_label.is_some(),
            "Jump cannot have else_label",
        ),
        (!terminator.args.is_empty(), "Jump cannot have args"),
        (terminator.reason.is_some(), "Jump cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn ensure_return_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (terminator.cond.is_some(), "Return cannot have cond"),
        (terminator.label.is_some(), "Return cannot have label"),
        (
            terminator.then_label.is_some(),
            "Return cannot have then_label",
        ),
        (
            terminator.else_label.is_some(),
            "Return cannot have else_label",
        ),
        (!terminator.args.is_empty(), "Return cannot have args"),
        (terminator.reason.is_some(), "Return cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn reject_bad_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    kind: GirTerminatorKind,
    reason: Option<&'static str>,
) -> Result<(), WpError> {
    if let Some(reason) = reason {
        return Err(WpError::UnsupportedTerminatorShape {
            function_id: function.id.clone(),
            block_label: block.label.clone(),
            kind,
            reason,
        });
    }
    Ok(())
}

fn first_present<const N: usize>(checks: [(bool, &'static str); N]) -> Option<&'static str> {
    checks
        .into_iter()
        .find_map(|(present, reason)| present.then_some(reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gir::import_gir_json;

    fn generate(input: &str) -> Result<VcModule, WpError> {
        let gir = import_gir_json(input).expect("GIR imports");
        generate_loop_vcs(&gir)
    }

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn apply(function: impl Into<String>, args: Vec<MpkExprTerm>) -> MpkExprTerm {
        MpkExprTerm::Apply {
            function: function.into(),
            args,
        }
    }

    fn int64_type() -> &'static str {
        r#"{"kind":"bv","width":64,"signed":true}"#
    }

    fn int64(value: u64) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_string(),
            width: 64,
            signed: true,
        }
    }

    fn sge(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sge"), vec![lhs, rhs])
    }

    fn sle(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sle"), vec![lhs, rhs])
    }

    fn slt(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.slt"), vec![lhs, rhs])
    }

    fn add(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.add"), vec![lhs, rhs])
    }

    fn sub(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sub"), vec![lhs, rhs])
    }

    fn loop_gir_json(with_decreases: bool) -> String {
        let decreases = if with_decreases {
            r#","decreases":[{"op":"bv_sub","lhs":{"var":"n"},"rhs":{"var":"i"}}]"#
        } else {
            ""
        };
        format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.CountTo","package":"example/pkg","name":"CountTo","params":[{{"name":"n","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[{{"name":"i","type":{ty}}}],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"Const","type":{ty},"value":{{"int":{{"value":"0","width":64,"signed":true}}}}}},{{"id":"t1","kind":"Copy","type":{ty},"target":"i","value":{{"var":"t0"}}}}],"terminator":{{"kind":"Jump","label":"loop"}}}},{{"label":"loop","parameters":[],"instructions":[{{"id":"t2","kind":"BinOp","op":"signed_lt","type":{{"kind":"bool"}},"lhs":{{"var":"i"}},"rhs":{{"var":"n"}}}}],"terminator":{{"kind":"Branch","cond":{{"var":"t2"}},"then_label":"body","else_label":"exit"}}}},{{"label":"body","parameters":[],"instructions":[{{"id":"t3","kind":"BinOp","op":"bv_add","type":{ty},"lhs":{{"var":"i"}},"rhs":{{"int":{{"value":"1","width":64,"signed":true}}}}}},{{"id":"t4","kind":"Copy","type":{ty},"target":"i","value":{{"var":"t3"}}}}],"terminator":{{"kind":"Jump","label":"loop"}}}},{{"label":"exit","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[{{"var":"i"}}]}}}}],"contracts":{{"requires":[{{"op":"signed_ge","lhs":{{"var":"n"}},"rhs":{{"int":{{"value":"0","width":64,"signed":true}}}}}}],"ensures":[{{"op":"eq","lhs":{{"result":0}},"rhs":{{"var":"n"}}}}],"modifies":[],"loops":[{{"block_id":"loop","invariants":[{{"op":"signed_ge","lhs":{{"var":"i"}},"rhs":{{"int":{{"value":"0","width":64,"signed":true}}}}}},{{"op":"signed_le","lhs":{{"var":"i"}},"rhs":{{"var":"n"}}}}]{decreases}}}]}}, "supported_features":["loops"],"rejected_features":[]}}]}}],"gir_hash":"loophash"}}"#,
            ty = int64_type(),
            decreases = decreases,
        )
    }

    #[test]
    fn annotated_loop_produces_partial_correctness_obligations() {
        let module = generate(&loop_gir_json(false)).expect("loop VCs generate");

        assert_eq!(module.source_gir_hash.as_deref(), Some("loophash"));
        assert_eq!(module.obligations.len(), 5);
        assert_eq!(
            module
                .obligations
                .iter()
                .map(|obligation| obligation.kind)
                .collect::<Vec<_>>(),
            vec![
                VcObligationKind::LoopInvariantInitial,
                VcObligationKind::LoopInvariantInitial,
                VcObligationKind::LoopInvariantPreservation,
                VcObligationKind::LoopInvariantPreservation,
                VcObligationKind::LoopExit,
            ]
        );
        assert_eq!(
            module.obligations[0].id,
            "example/pkg.CountTo.loop.loop.initial.inv0"
        );
        assert_eq!(
            module.obligations[0].assumptions,
            vec![sge(var("n"), int64(0))]
        );
        assert_eq!(module.obligations[0].conclusion, sge(int64(0), int64(0)));
        assert_eq!(module.obligations[1].conclusion, sle(int64(0), var("n")));
        assert_eq!(
            module.obligations[2].assumptions,
            vec![
                sge(var("i"), int64(0)),
                sle(var("i"), var("n")),
                slt(var("i"), var("n"))
            ]
        );
        assert_eq!(
            module.obligations[2].conclusion,
            sge(add(var("i"), int64(1)), int64(0))
        );
        assert_eq!(
            module.obligations[3].conclusion,
            sle(add(var("i"), int64(1)), var("n"))
        );
        assert_eq!(
            module.obligations[4].assumptions,
            vec![
                sge(var("i"), int64(0)),
                sle(var("i"), var("n")),
                MpkExprTerm::apply(STD_BOOL_NOT, [slt(var("i"), var("n"))])
            ]
        );
        assert_eq!(
            module.obligations[4].conclusion,
            apply(crate::expr_encode::STD_EQ, vec![var("i"), var("n")])
        );
    }

    #[test]
    fn annotated_loop_produces_decreases_obligations_when_requested() {
        let module = generate(&loop_gir_json(true)).expect("loop VCs generate");

        assert_eq!(module.obligations.len(), 7);
        let nonnegative = &module.obligations[5];
        let strict = &module.obligations[6];
        assert_eq!(nonnegative.kind, VcObligationKind::Decreases);
        assert_eq!(
            nonnegative.id,
            "example/pkg.CountTo.loop.loop.decreases0.nonnegative"
        );
        assert_eq!(
            nonnegative.conclusion,
            sge(sub(var("n"), var("i")), int64(0))
        );
        assert_eq!(strict.kind, VcObligationKind::Decreases);
        assert_eq!(strict.id, "example/pkg.CountTo.loop.loop.decreases0.strict");
        assert_eq!(
            strict.conclusion,
            slt(
                sub(var("n"), add(var("i"), int64(1))),
                sub(var("n"), var("i")),
            )
        );
    }

    #[test]
    fn rejects_loop_contract_without_invariant() {
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.BadLoop","package":"example/pkg","name":"BadLoop","params":[],"results":[],"locals":[],"blocks":[{{"label":"loop","parameters":[],"instructions":[],"terminator":{{"kind":"Branch","cond":{{"bool":false}},"then_label":"body","else_label":"exit"}}}},{{"label":"body","parameters":[],"instructions":[],"terminator":{{"kind":"Jump","label":"loop"}}}},{{"label":"exit","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[]}}}}],"contracts":{{"requires":[],"ensures":[{{"bool":true}}],"modifies":[],"loops":[{{"block_id":"loop","invariants":[]}}]}},"supported_features":[],"rejected_features":[]}}]}}]}}"#
        );

        let error = generate(&input).expect_err("missing invariant rejects");

        assert_eq!(
            error,
            WpError::MissingLoopInvariant {
                function_id: "example/pkg.BadLoop".to_owned(),
                block_label: "loop".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_unsupported_variant_shape() {
        let input = loop_gir_json(true).replace(
            r#""decreases":[{"op":"bv_sub","lhs":{"var":"n"},"rhs":{"var":"i"}}]"#,
            r#""decreases":[{"op":"signed_ge","lhs":{"var":"n"},"rhs":{"var":"i"}}]"#,
        );

        let error = generate(&input).expect_err("unsupported variant rejects");

        assert_eq!(
            error,
            WpError::UnsupportedLoopVariant {
                function_id: "example/pkg.CountTo".to_owned(),
                block_label: "loop".to_owned(),
                variant_index: 0,
                reason: "variant must be a bitvector variable, literal, or convert expression"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn rejects_loop_body_that_does_not_jump_back_to_header() {
        let input = loop_gir_json(false).replace(
            r#""label":"body","parameters":[],"instructions":[{"id":"t3","kind":"BinOp","op":"bv_add","type":{"kind":"bv","width":64,"signed":true},"lhs":{"var":"i"},"rhs":{"int":{"value":"1","width":64,"signed":true}}},{"id":"t4","kind":"Copy","type":{"kind":"bv","width":64,"signed":true},"target":"i","value":{"var":"t3"}}],"terminator":{"kind":"Jump","label":"loop"}}"#,
            r#""label":"body","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"i"}]}}"#,
        );

        let error = generate(&input).expect_err("bad loop shape rejects");

        assert!(matches!(
            error,
            WpError::UnsupportedLoopShape {
                block_label,
                reason,
                ..
            } if block_label == "loop" && reason.contains("loop body block")
        ));
    }
}
