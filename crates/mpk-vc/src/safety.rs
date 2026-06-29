//! Runtime-safety VC generation for supported GIR operations.
//!
//! VC-006 emits obligations for operations whose Go panic condition is
//! expressible in GIR: division/remainder by zero, negative shift counts, and
//! fixed-array index bounds. The generator is path-sensitive for the same
//! straight-line and simple if/else shapes handled by the first WP milestones.

use std::collections::{BTreeMap, BTreeSet};

use crate::expr_encode::{
    ExprEncodeError, ExprEncoder, MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_NOT, STD_EQ,
};
use crate::gir::{
    GirBlock, GirFunction, GirInstruction, GirInstructionKind, GirModule, GirTerminator,
    GirTerminatorKind, GirType, GirTypeKind, GirValue,
};
use crate::type_encode::encode_gir_type;
use crate::vc::{VcModule, VcObligation, VcObligationKind};
use crate::wp::{encode_requires, initial_environment, substitute_term, WpError};

pub fn generate_safety_vcs(input: &GirModule) -> Result<VcModule, WpError> {
    SafetyVcGenerator::new().generate_module(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SafetyVcGenerator;

impl SafetyVcGenerator {
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
        if function.blocks.is_empty() {
            return Err(WpError::UnsupportedBlockCount {
                function_id: function.id.clone(),
                block_count: 0,
            });
        }

        let encoder = ExprEncoder::for_function(function);
        let initial_env = SafetyEnv::for_function(function);
        let assumptions = encode_requires(function, &encoder, &initial_env.terms)?;
        let blocks = block_map(function);
        let entry = &function.blocks[0];
        ensure_no_block_parameters(function, entry)?;

        match entry.terminator.kind {
            GirTerminatorKind::Return => {
                if function.blocks.len() != 1 {
                    return Err(WpError::UnsupportedBlockCount {
                        function_id: function.id.clone(),
                        block_count: function.blocks.len(),
                    });
                }
                ensure_return_terminator_shape(function, entry, &entry.terminator)?;
                let mut output = Vec::new();
                let mut env = initial_env;
                self.scan_block(
                    function,
                    entry,
                    &encoder,
                    "entry",
                    &assumptions,
                    &mut env,
                    &mut output,
                )?;
                Ok(output)
            }
            GirTerminatorKind::Branch => {
                ensure_branch_terminator_shape(function, entry, &entry.terminator)?;
                let mut output = Vec::new();
                let mut branch_env = initial_env;
                self.scan_block(
                    function,
                    entry,
                    &encoder,
                    "entry",
                    &assumptions,
                    &mut branch_env,
                    &mut output,
                )?;
                let condition = encode_branch_condition(function, entry, &encoder, &branch_env)?;
                let then_label = entry.terminator.then_label.as_deref().ok_or_else(|| {
                    WpError::MissingBranchLabel {
                        function_id: function.id.clone(),
                        block_label: entry.label.clone(),
                        label_kind: "then",
                    }
                })?;
                let else_label = entry.terminator.else_label.as_deref().ok_or_else(|| {
                    WpError::MissingBranchLabel {
                        function_id: function.id.clone(),
                        block_label: entry.label.clone(),
                        label_kind: "else",
                    }
                })?;

                let mut then_assumptions = assumptions.clone();
                then_assumptions.push(condition.clone());
                self.scan_path_to_return(
                    function,
                    &blocks,
                    &encoder,
                    "then",
                    then_label,
                    branch_env.clone(),
                    then_assumptions,
                    &mut output,
                )?;

                let mut else_assumptions = assumptions;
                else_assumptions.push(MpkExprTerm::apply(STD_BOOL_NOT, [condition]));
                self.scan_path_to_return(
                    function,
                    &blocks,
                    &encoder,
                    "else",
                    else_label,
                    branch_env,
                    else_assumptions,
                    &mut output,
                )?;
                Ok(output)
            }
            kind => Err(WpError::UnsupportedTerminator {
                function_id: function.id.clone(),
                block_label: entry.label.clone(),
                kind,
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_path_to_return(
        self,
        function: &GirFunction,
        blocks: &BTreeMap<String, &GirBlock>,
        encoder: &ExprEncoder,
        path_name: &'static str,
        start_label: &str,
        mut env: SafetyEnv,
        assumptions: Vec<MpkExprTerm>,
        output: &mut Vec<VcObligation>,
    ) -> Result<(), WpError> {
        let mut label = start_label.to_owned();
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(label.clone()) {
                return Err(WpError::CyclicBranchPath {
                    function_id: function.id.clone(),
                    block_label: label,
                });
            }
            let block = blocks
                .get(&label)
                .copied()
                .ok_or_else(|| WpError::UnknownBlockLabel {
                    function_id: function.id.clone(),
                    context: "runtime-safety path".to_owned(),
                    block_label: label.clone(),
                })?;
            ensure_no_block_parameters(function, block)?;
            self.scan_block(
                function,
                block,
                encoder,
                path_name,
                &assumptions,
                &mut env,
                output,
            )?;

            match block.terminator.kind {
                GirTerminatorKind::Return => {
                    ensure_return_terminator_shape(function, block, &block.terminator)?;
                    return Ok(());
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
                    return Err(WpError::UnsupportedTerminator {
                        function_id: function.id.clone(),
                        block_label: block.label.clone(),
                        kind,
                    });
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_block(
        self,
        function: &GirFunction,
        block: &GirBlock,
        encoder: &ExprEncoder,
        path_name: &'static str,
        assumptions: &[MpkExprTerm],
        env: &mut SafetyEnv,
        output: &mut Vec<VcObligation>,
    ) -> Result<(), WpError> {
        for instruction in &block.instructions {
            ensure_instruction_id(function, block, instruction)?;
            ensure_safety_instruction_shape(function, block, instruction)?;
            output.extend(runtime_obligations_for_instruction(
                function,
                block,
                instruction,
                encoder,
                path_name,
                assumptions,
                env,
            )?);
            env.record_instruction_type(instruction);
            if let Some(term) = instruction_term(function, block, instruction, encoder, env)? {
                env.terms.insert(instruction.id.clone(), term.clone());
                if instruction.kind == GirInstructionKind::Copy {
                    if let Some(target) = instruction
                        .target
                        .as_deref()
                        .filter(|target| !target.is_empty())
                    {
                        if !env.types.contains_key(target) {
                            return Err(WpError::UnknownVariable {
                                function_id: function.id.clone(),
                                context: format!("copy target {}", instruction.id),
                                name: target.to_owned(),
                            });
                        }
                        env.terms.insert(target.to_owned(), term);
                        env.types
                            .insert(target.to_owned(), instruction.r#type.clone());
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SafetyEnv {
    terms: BTreeMap<String, MpkExprTerm>,
    types: BTreeMap<String, GirType>,
}

impl SafetyEnv {
    fn for_function(function: &GirFunction) -> Self {
        let mut types = BTreeMap::new();
        for binding in function.params.iter().chain(function.locals.iter()) {
            if !binding.name.is_empty() {
                types.insert(binding.name.clone(), binding.r#type.clone());
            }
        }
        Self {
            terms: initial_environment(function),
            types,
        }
    }

    fn record_instruction_type(&mut self, instruction: &GirInstruction) {
        if !instruction.id.is_empty() {
            self.types
                .insert(instruction.id.clone(), instruction.r#type.clone());
        }
    }

    fn value_type(&self, value: &GirValue) -> Option<GirType> {
        if let Some(name) = &value.var {
            return self.types.get(name).cloned();
        }
        if let Some(literal) = &value.int {
            return Some(GirType {
                kind: GirTypeKind::BitVector,
                name: None,
                width: Some(literal.width),
                signed: Some(literal.signed),
                length: None,
                element: None,
                fields: Vec::new(),
            });
        }
        if value.bool.is_some() {
            return Some(GirType {
                kind: GirTypeKind::Bool,
                name: None,
                width: None,
                signed: None,
                length: None,
                element: None,
                fields: Vec::new(),
            });
        }
        None
    }

    fn value_term(
        &self,
        function: &GirFunction,
        context: impl Into<String>,
        value: &GirValue,
        encoder: &ExprEncoder,
    ) -> Result<MpkExprTerm, WpError> {
        let context = context.into();
        if let Some(name) = &value.var {
            if !self.terms.contains_key(name) {
                return Err(WpError::UnknownVariable {
                    function_id: function.id.clone(),
                    context,
                    name: name.clone(),
                });
            }
        }
        let encoded = encoder
            .encode_value(value)
            .map_err(|source| WpError::Expression {
                function_id: function.id.clone(),
                context,
                source,
            })?;
        Ok(substitute_term(&encoded, &self.terms, &BTreeMap::new()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BitVectorShape {
    width: u32,
    signed: bool,
}

fn reject_common_unsupported(function: &GirFunction) -> Result<(), WpError> {
    if !function.rejected_features.is_empty() {
        return Err(WpError::FunctionHasRejectedFeatures {
            function_id: function.id.clone(),
            rejected_feature_count: function.rejected_features.len(),
        });
    }
    if function.contracts.ensures.is_empty() {
        return Err(WpError::MissingPostcondition {
            function_id: function.id.clone(),
        });
    }
    if !function.contracts.modifies.is_empty() {
        return Err(WpError::NonEmptyModifies {
            function_id: function.id.clone(),
            modifies: function.contracts.modifies.clone(),
        });
    }
    if !function.contracts.loops.is_empty() {
        return Err(WpError::LoopContractsUnsupported {
            function_id: function.id.clone(),
            loop_count: function.contracts.loops.len(),
        });
    }
    Ok(())
}

fn block_map(function: &GirFunction) -> BTreeMap<String, &GirBlock> {
    function
        .blocks
        .iter()
        .map(|block| (block.label.clone(), block))
        .collect()
}

fn runtime_obligations_for_instruction(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    encoder: &ExprEncoder,
    path_name: &'static str,
    assumptions: &[MpkExprTerm],
    env: &SafetyEnv,
) -> Result<Vec<VcObligation>, WpError> {
    match instruction.kind {
        GirInstructionKind::BinOp => {
            let op = required_op(function, block, instruction)?;
            match op {
                "bv_sdiv" | "bv_udiv" | "bv_srem" | "bv_urem" => {
                    let rhs = required_rhs(function, block, instruction)?;
                    let rhs_term = env.value_term(
                        function,
                        format!("instruction {} rhs", instruction.id),
                        rhs,
                        encoder,
                    )?;
                    let rhs_type =
                        require_value_type(function, block, instruction, "rhs", env, rhs)?;
                    let rhs_bv =
                        require_bitvector_type(function, block, instruction, "rhs", &rhs_type)?;
                    let zero = bitvec_literal(0, rhs_bv);
                    Ok(vec![runtime_obligation(
                        function,
                        block,
                        instruction,
                        path_name,
                        "divisor_nonzero",
                        assumptions,
                        MpkExprTerm::apply(
                            STD_BOOL_NOT,
                            [MpkExprTerm::apply(STD_EQ, [rhs_term, zero])],
                        ),
                    )])
                }
                "bv_shl" | "bv_ashr" | "bv_lshr" => {
                    let rhs = required_rhs(function, block, instruction)?;
                    let rhs_type =
                        require_value_type(function, block, instruction, "rhs", env, rhs)?;
                    let rhs_bv =
                        require_bitvector_type(function, block, instruction, "rhs", &rhs_type)?;
                    if !rhs_bv.signed {
                        return Ok(Vec::new());
                    }
                    let rhs_term = env.value_term(
                        function,
                        format!("instruction {} rhs", instruction.id),
                        rhs,
                        encoder,
                    )?;
                    Ok(vec![runtime_obligation(
                        function,
                        block,
                        instruction,
                        path_name,
                        "shift_nonnegative",
                        assumptions,
                        MpkExprTerm::apply(
                            bitvec_function(rhs_bv.width, "sge"),
                            [rhs_term, bitvec_literal(0, rhs_bv)],
                        ),
                    )])
                }
                _ => Ok(Vec::new()),
            }
        }
        GirInstructionKind::Index => {
            let base = required_base(function, block, instruction)?;
            let index = required_index(function, block, instruction)?;
            let base_type = require_value_type(function, block, instruction, "base", env, base)?;
            let array_length = require_array_length(function, block, instruction, &base_type)?;
            let index_type = require_value_type(function, block, instruction, "index", env, index)?;
            let index_bv =
                require_bitvector_type(function, block, instruction, "index", &index_type)?;
            let index_term = env.value_term(
                function,
                format!("instruction {} index", instruction.id),
                index,
                encoder,
            )?;
            let length_literal =
                checked_length_literal(function, block, instruction, array_length, index_bv)?;

            let mut output = Vec::new();
            if index_bv.signed {
                output.push(runtime_obligation(
                    function,
                    block,
                    instruction,
                    path_name,
                    "index_nonnegative",
                    assumptions,
                    MpkExprTerm::apply(
                        bitvec_function(index_bv.width, "sge"),
                        [index_term.clone(), bitvec_literal(0, index_bv)],
                    ),
                ));
                output.push(runtime_obligation(
                    function,
                    block,
                    instruction,
                    path_name,
                    "index_in_bounds",
                    assumptions,
                    MpkExprTerm::apply(
                        bitvec_function(index_bv.width, "slt"),
                        [index_term, length_literal],
                    ),
                ));
            } else {
                output.push(runtime_obligation(
                    function,
                    block,
                    instruction,
                    path_name,
                    "index_in_bounds",
                    assumptions,
                    MpkExprTerm::apply(
                        bitvec_function(index_bv.width, "ult"),
                        [index_term, length_literal],
                    ),
                ));
            }
            Ok(output)
        }
        _ => Ok(Vec::new()),
    }
}

fn runtime_obligation(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    path_name: &'static str,
    suffix: &'static str,
    assumptions: &[MpkExprTerm],
    conclusion: MpkExprTerm,
) -> VcObligation {
    VcObligation {
        id: format!(
            "{}.{path_name}.{}.{}.{}",
            function.id, block.label, instruction.id, suffix
        ),
        function_id: function.id.clone(),
        kind: VcObligationKind::RuntimeSafety,
        assumptions: assumptions.to_vec(),
        conclusion,
    }
}

fn instruction_term(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    encoder: &ExprEncoder,
    env: &SafetyEnv,
) -> Result<Option<MpkExprTerm>, WpError> {
    match instruction.kind {
        GirInstructionKind::Const
        | GirInstructionKind::Copy
        | GirInstructionKind::UnaryOp
        | GirInstructionKind::Convert => {
            let encoded =
                encoder
                    .encode_instruction(instruction)
                    .map_err(|source| WpError::Expression {
                        function_id: function.id.clone(),
                        context: format!("instruction {}", instruction.id),
                        source,
                    })?;
            Ok(Some(substitute_term(
                &encoded,
                &env.terms,
                &BTreeMap::new(),
            )))
        }
        GirInstructionKind::BinOp => {
            let op = required_op(function, block, instruction)?;
            let lhs = required_lhs(function, block, instruction)?;
            let rhs = required_rhs(function, block, instruction)?;
            let lhs_term = env.value_term(
                function,
                format!("instruction {} lhs", instruction.id),
                lhs,
                encoder,
            )?;
            let rhs_term = env.value_term(
                function,
                format!("instruction {} rhs", instruction.id),
                rhs,
                encoder,
            )?;
            match op {
                "bv_sdiv" | "bv_udiv" | "bv_srem" | "bv_urem" => {
                    let result_bv =
                        require_division_shapes(function, block, instruction, op, lhs, rhs, env)?;
                    let suffix = match op {
                        "bv_sdiv" => "sdiv",
                        "bv_udiv" => "udiv",
                        "bv_srem" => "srem",
                        "bv_urem" => "urem",
                        _ => unreachable!("outer match limits division operators"),
                    };
                    Ok(Some(MpkExprTerm::apply(
                        bitvec_function(result_bv.width, suffix),
                        [lhs_term, rhs_term],
                    )))
                }
                _ => {
                    let encoded = encoder.encode_instruction(instruction).map_err(|source| {
                        WpError::Expression {
                            function_id: function.id.clone(),
                            context: format!("instruction {}", instruction.id),
                            source,
                        }
                    })?;
                    Ok(Some(substitute_term(
                        &encoded,
                        &env.terms,
                        &BTreeMap::new(),
                    )))
                }
            }
        }
        _ => Ok(None),
    }
}

fn encode_branch_condition(
    function: &GirFunction,
    block: &GirBlock,
    encoder: &ExprEncoder,
    env: &SafetyEnv,
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
    env.value_term(function, "branch condition", condition, encoder)
}

fn result_bitvector_shape(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
) -> Result<BitVectorShape, WpError> {
    require_bitvector_type(function, block, instruction, "result", &instruction.r#type)
}

fn require_division_shapes(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    op: &str,
    lhs: &GirValue,
    rhs: &GirValue,
    env: &SafetyEnv,
) -> Result<BitVectorShape, WpError> {
    let lhs_type = require_value_type(function, block, instruction, "lhs", env, lhs)?;
    let rhs_type = require_value_type(function, block, instruction, "rhs", env, rhs)?;
    let lhs_bv = require_bitvector_type(function, block, instruction, "lhs", &lhs_type)?;
    let rhs_bv = require_bitvector_type(function, block, instruction, "rhs", &rhs_type)?;
    let result_bv = result_bitvector_shape(function, block, instruction)?;
    if lhs_bv.width != rhs_bv.width || lhs_bv.width != result_bv.width {
        return Err(unsupported_safety_check(
            function,
            block,
            instruction,
            "division operands and result must have matching bitvector widths",
        ));
    }

    let expected_signed = matches!(op, "bv_sdiv" | "bv_srem");
    if lhs_bv.signed != expected_signed
        || rhs_bv.signed != expected_signed
        || result_bv.signed != expected_signed
    {
        return Err(unsupported_safety_check(
            function,
            block,
            instruction,
            format!(
                "{op} requires {} bitvectors",
                if expected_signed {
                    "signed"
                } else {
                    "unsigned"
                }
            ),
        ));
    }
    Ok(result_bv)
}

fn require_value_type(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    operand: &'static str,
    env: &SafetyEnv,
    value: &GirValue,
) -> Result<GirType, WpError> {
    env.value_type(value).ok_or_else(|| {
        unsupported_safety_check(
            function,
            block,
            instruction,
            format!("missing type for {operand}"),
        )
    })
}

fn require_bitvector_type(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    operand: &'static str,
    r#type: &GirType,
) -> Result<BitVectorShape, WpError> {
    encode_gir_type(r#type).map_err(|source| WpError::Expression {
        function_id: function.id.clone(),
        context: format!("instruction {} {operand} type", instruction.id),
        source: ExprEncodeError::Type(source),
    })?;
    if r#type.kind != GirTypeKind::BitVector {
        return Err(unsupported_safety_check(
            function,
            block,
            instruction,
            format!("{operand} is not a bitvector"),
        ));
    }
    Ok(BitVectorShape {
        width: r#type.width.expect("type encoder validated width"),
        signed: r#type.signed.expect("type encoder validated signed"),
    })
}

fn require_array_length(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    r#type: &GirType,
) -> Result<u64, WpError> {
    encode_gir_type(r#type).map_err(|source| WpError::Expression {
        function_id: function.id.clone(),
        context: format!("instruction {} base type", instruction.id),
        source: ExprEncodeError::Type(source),
    })?;
    if r#type.kind != GirTypeKind::Array {
        return Err(unsupported_safety_check(
            function,
            block,
            instruction,
            "base is not a fixed array",
        ));
    }
    r#type.length.ok_or_else(|| {
        unsupported_safety_check(function, block, instruction, "array base is missing length")
    })
}

fn checked_length_literal(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    length: u64,
    bitvec: BitVectorShape,
) -> Result<MpkExprTerm, WpError> {
    let limit = max_positive_value(bitvec);
    if u128::from(length) > limit {
        return Err(unsupported_safety_check(
            function,
            block,
            instruction,
            format!(
                "array length {length} does not fit {}{} index type",
                if bitvec.signed {
                    "signed "
                } else {
                    "unsigned "
                },
                bitvec.width
            ),
        ));
    }
    Ok(bitvec_literal(length, bitvec))
}

fn unsupported_safety_check(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    reason: impl Into<String>,
) -> WpError {
    WpError::UnsupportedSafetyCheck {
        function_id: function.id.clone(),
        block_label: block.label.clone(),
        instruction_id: instruction.id.clone(),
        reason: reason.into(),
    }
}

fn max_positive_value(bitvec: BitVectorShape) -> u128 {
    let bits = if bitvec.signed {
        bitvec.width - 1
    } else {
        bitvec.width
    };
    (1_u128 << bits) - 1
}

fn bitvec_literal(value: u64, bitvec: BitVectorShape) -> MpkExprTerm {
    MpkExprTerm::BitVecLiteral {
        value: value.to_string(),
        width: bitvec.width,
        signed: bitvec.signed,
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

fn required_op<'a>(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &'a GirInstruction,
) -> Result<&'a str, WpError> {
    instruction
        .op
        .as_deref()
        .ok_or_else(|| unsupported_shape(function, block, instruction, "BinOp must have op"))
}

fn required_lhs<'a>(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &'a GirInstruction,
) -> Result<&'a GirValue, WpError> {
    instruction
        .lhs
        .as_ref()
        .ok_or_else(|| unsupported_shape(function, block, instruction, "BinOp must have lhs"))
}

fn required_rhs<'a>(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &'a GirInstruction,
) -> Result<&'a GirValue, WpError> {
    instruction
        .rhs
        .as_ref()
        .ok_or_else(|| unsupported_shape(function, block, instruction, "BinOp must have rhs"))
}

fn required_base<'a>(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &'a GirInstruction,
) -> Result<&'a GirValue, WpError> {
    instruction
        .base
        .as_ref()
        .ok_or_else(|| unsupported_shape(function, block, instruction, "Index must have base"))
}

fn required_index<'a>(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &'a GirInstruction,
) -> Result<&'a GirValue, WpError> {
    instruction
        .index
        .as_ref()
        .ok_or_else(|| unsupported_shape(function, block, instruction, "Index must have index"))
}

fn unsupported_shape(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
    reason: &'static str,
) -> WpError {
    WpError::UnsupportedInstructionShape {
        function_id: function.id.clone(),
        block_label: block.label.clone(),
        instruction_id: instruction.id.clone(),
        kind: instruction.kind,
        reason,
    }
}

fn ensure_instruction_id(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
) -> Result<(), WpError> {
    if instruction.id.is_empty() {
        return Err(WpError::EmptyInstructionId {
            function_id: function.id.clone(),
            block_label: block.label.clone(),
        });
    }
    Ok(())
}

fn ensure_safety_instruction_shape(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
) -> Result<(), WpError> {
    let reason = match instruction.kind {
        GirInstructionKind::Const => first_present([
            (instruction.op.is_some(), "Const cannot have op"),
            (instruction.target.is_some(), "Const cannot have target"),
            (instruction.base.is_some(), "Const cannot have base"),
            (instruction.index.is_some(), "Const cannot have index"),
            (instruction.field.is_some(), "Const cannot have field"),
            (!instruction.fields.is_empty(), "Const cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "Const cannot have elements",
            ),
            (instruction.lhs.is_some(), "Const cannot have lhs"),
            (instruction.rhs.is_some(), "Const cannot have rhs"),
            (instruction.function.is_some(), "Const cannot have function"),
            (!instruction.args.is_empty(), "Const cannot have args"),
        ]),
        GirInstructionKind::Copy => first_present([
            (instruction.op.is_some(), "Copy cannot have op"),
            (instruction.base.is_some(), "Copy cannot have base"),
            (instruction.index.is_some(), "Copy cannot have index"),
            (instruction.field.is_some(), "Copy cannot have field"),
            (!instruction.fields.is_empty(), "Copy cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "Copy cannot have elements",
            ),
            (instruction.lhs.is_some(), "Copy cannot have lhs"),
            (instruction.rhs.is_some(), "Copy cannot have rhs"),
            (instruction.function.is_some(), "Copy cannot have function"),
            (!instruction.args.is_empty(), "Copy cannot have args"),
        ]),
        GirInstructionKind::BinOp => first_present([
            (instruction.target.is_some(), "BinOp cannot have target"),
            (instruction.value.is_some(), "BinOp cannot have value"),
            (instruction.base.is_some(), "BinOp cannot have base"),
            (instruction.index.is_some(), "BinOp cannot have index"),
            (instruction.field.is_some(), "BinOp cannot have field"),
            (!instruction.fields.is_empty(), "BinOp cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "BinOp cannot have elements",
            ),
            (instruction.function.is_some(), "BinOp cannot have function"),
            (!instruction.args.is_empty(), "BinOp cannot have args"),
        ]),
        GirInstructionKind::UnaryOp => first_present([
            (instruction.target.is_some(), "UnaryOp cannot have target"),
            (instruction.base.is_some(), "UnaryOp cannot have base"),
            (instruction.index.is_some(), "UnaryOp cannot have index"),
            (instruction.field.is_some(), "UnaryOp cannot have field"),
            (!instruction.fields.is_empty(), "UnaryOp cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "UnaryOp cannot have elements",
            ),
            (instruction.lhs.is_some(), "UnaryOp cannot have lhs"),
            (instruction.rhs.is_some(), "UnaryOp cannot have rhs"),
            (
                instruction.function.is_some(),
                "UnaryOp cannot have function",
            ),
            (!instruction.args.is_empty(), "UnaryOp cannot have args"),
        ]),
        GirInstructionKind::Convert => first_present([
            (instruction.op.is_some(), "Convert cannot have op"),
            (instruction.target.is_some(), "Convert cannot have target"),
            (instruction.base.is_some(), "Convert cannot have base"),
            (instruction.index.is_some(), "Convert cannot have index"),
            (instruction.field.is_some(), "Convert cannot have field"),
            (!instruction.fields.is_empty(), "Convert cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "Convert cannot have elements",
            ),
            (instruction.lhs.is_some(), "Convert cannot have lhs"),
            (instruction.rhs.is_some(), "Convert cannot have rhs"),
            (
                instruction.function.is_some(),
                "Convert cannot have function",
            ),
            (!instruction.args.is_empty(), "Convert cannot have args"),
        ]),
        GirInstructionKind::Index => first_present([
            (instruction.op.is_some(), "Index cannot have op"),
            (instruction.target.is_some(), "Index cannot have target"),
            (instruction.value.is_some(), "Index cannot have value"),
            (instruction.base.is_none(), "Index must have base"),
            (instruction.index.is_none(), "Index must have index"),
            (instruction.field.is_some(), "Index cannot have field"),
            (!instruction.fields.is_empty(), "Index cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "Index cannot have elements",
            ),
            (instruction.lhs.is_some(), "Index cannot have lhs"),
            (instruction.rhs.is_some(), "Index cannot have rhs"),
            (instruction.function.is_some(), "Index cannot have function"),
            (!instruction.args.is_empty(), "Index cannot have args"),
        ]),
        GirInstructionKind::Field => first_present([
            (instruction.op.is_some(), "Field cannot have op"),
            (instruction.target.is_some(), "Field cannot have target"),
            (instruction.value.is_some(), "Field cannot have value"),
            (instruction.base.is_none(), "Field must have base"),
            (instruction.index.is_some(), "Field cannot have index"),
            (instruction.field.is_none(), "Field must have field"),
            (!instruction.fields.is_empty(), "Field cannot have fields"),
            (
                !instruction.elements.is_empty(),
                "Field cannot have elements",
            ),
            (instruction.lhs.is_some(), "Field cannot have lhs"),
            (instruction.rhs.is_some(), "Field cannot have rhs"),
            (instruction.function.is_some(), "Field cannot have function"),
            (!instruction.args.is_empty(), "Field cannot have args"),
        ]),
        GirInstructionKind::MakeArray => first_present([
            (instruction.op.is_some(), "MakeArray cannot have op"),
            (instruction.target.is_some(), "MakeArray cannot have target"),
            (instruction.value.is_some(), "MakeArray cannot have value"),
            (instruction.base.is_some(), "MakeArray cannot have base"),
            (instruction.index.is_some(), "MakeArray cannot have index"),
            (instruction.field.is_some(), "MakeArray cannot have field"),
            (
                !instruction.fields.is_empty(),
                "MakeArray cannot have fields",
            ),
            (instruction.lhs.is_some(), "MakeArray cannot have lhs"),
            (instruction.rhs.is_some(), "MakeArray cannot have rhs"),
            (
                instruction.function.is_some(),
                "MakeArray cannot have function",
            ),
            (!instruction.args.is_empty(), "MakeArray cannot have args"),
        ]),
        GirInstructionKind::MakeStruct => first_present([
            (instruction.op.is_some(), "MakeStruct cannot have op"),
            (
                instruction.target.is_some(),
                "MakeStruct cannot have target",
            ),
            (instruction.value.is_some(), "MakeStruct cannot have value"),
            (instruction.base.is_some(), "MakeStruct cannot have base"),
            (instruction.index.is_some(), "MakeStruct cannot have index"),
            (instruction.field.is_some(), "MakeStruct cannot have field"),
            (
                !instruction.elements.is_empty(),
                "MakeStruct cannot have elements",
            ),
            (instruction.lhs.is_some(), "MakeStruct cannot have lhs"),
            (instruction.rhs.is_some(), "MakeStruct cannot have rhs"),
            (
                instruction.function.is_some(),
                "MakeStruct cannot have function",
            ),
            (!instruction.args.is_empty(), "MakeStruct cannot have args"),
        ]),
        GirInstructionKind::CallStatic => first_present([
            (instruction.op.is_some(), "CallStatic cannot have op"),
            (
                instruction.target.is_some(),
                "CallStatic cannot have target",
            ),
            (instruction.value.is_some(), "CallStatic cannot have value"),
            (instruction.base.is_some(), "CallStatic cannot have base"),
            (instruction.index.is_some(), "CallStatic cannot have index"),
            (instruction.field.is_some(), "CallStatic cannot have field"),
            (
                !instruction.fields.is_empty(),
                "CallStatic cannot have fields",
            ),
            (
                !instruction.elements.is_empty(),
                "CallStatic cannot have elements",
            ),
            (instruction.lhs.is_some(), "CallStatic cannot have lhs"),
            (instruction.rhs.is_some(), "CallStatic cannot have rhs"),
            (
                instruction.function.is_none(),
                "CallStatic must have function",
            ),
        ]),
        GirInstructionKind::Phi | GirInstructionKind::Unsupported => {
            Some("instruction kind is not supported for runtime-safety generation")
        }
    };

    if let Some(reason) = reason {
        return Err(unsupported_shape(function, block, instruction, reason));
    }
    Ok(())
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
        generate_safety_vcs(&gir)
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

    fn bv_type(width: u32, signed: bool) -> String {
        format!(r#"{{"kind":"bv","width":{width},"signed":{signed}}}"#)
    }

    fn contracts() -> &'static str {
        r#""contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]}"#
    }

    fn zero(width: u32, signed: bool) -> MpkExprTerm {
        literal(0, width, signed)
    }

    fn literal(value: u64, width: u32, signed: bool) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_string(),
            width,
            signed,
        }
    }

    #[test]
    fn emits_division_by_zero_obligation() {
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.Div","package":"example/pkg","name":"Div","params":[{{"name":"a","type":{ty}}},{{"name":"b","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"bv_sdiv","type":{ty},"lhs":{{"var":"a"}},"rhs":{{"var":"b"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}],"gir_hash":"safetyhash"}}"#,
            ty = bv_type(64, true),
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert_eq!(module.source_gir_hash.as_deref(), Some("safetyhash"));
        assert_eq!(module.obligations.len(), 1);
        let obligation = &module.obligations[0];
        assert_eq!(
            obligation.id,
            "example/pkg.Div.entry.entry.t0.divisor_nonzero"
        );
        assert_eq!(obligation.kind, VcObligationKind::RuntimeSafety);
        assert!(obligation.assumptions.is_empty());
        assert_eq!(
            obligation.conclusion,
            apply(
                STD_BOOL_NOT,
                vec![apply(STD_EQ, vec![var("b"), zero(64, true)])]
            )
        );
    }

    #[test]
    fn rejects_signed_division_on_unsigned_bitvectors() {
        let ty = bv_type(64, false);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.BadDiv","package":"example/pkg","name":"BadDiv","params":[{{"name":"a","type":{ty}}},{{"name":"b","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"bv_sdiv","type":{ty},"lhs":{{"var":"a"}},"rhs":{{"var":"b"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = ty,
            contracts = contracts(),
        );

        let error = generate(&input).expect_err("malformed signed division rejects");

        assert_eq!(
            error,
            WpError::UnsupportedSafetyCheck {
                function_id: "example/pkg.BadDiv".to_owned(),
                block_label: "entry".to_owned(),
                instruction_id: "t0".to_owned(),
                reason: "bv_sdiv requires signed bitvectors".to_owned(),
            }
        );
    }

    #[test]
    fn emits_shift_nonnegative_obligation_for_signed_shift_count() {
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.Shift","package":"example/pkg","name":"Shift","params":[{{"name":"value","type":{ty}}},{{"name":"count","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"bv_shl","type":{ty},"lhs":{{"var":"value"}},"rhs":{{"var":"count"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = bv_type(32, true),
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert_eq!(module.obligations.len(), 1);
        assert_eq!(
            module.obligations[0].id,
            "example/pkg.Shift.entry.entry.t0.shift_nonnegative"
        );
        assert_eq!(
            module.obligations[0].conclusion,
            apply(
                format!("{STD_BITVEC_MODULE}.BV32.sge"),
                vec![var("count"), zero(32, true)]
            )
        );
    }

    #[test]
    fn emits_signed_array_index_bounds_obligations() {
        let array_ty =
            r#"{"kind":"array","length":2,"element":{"kind":"bv","width":64,"signed":true}}"#;
        let index_ty = bv_type(64, true);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.Pick","package":"example/pkg","name":"Pick","params":[{{"name":"values","type":{array_ty}}},{{"name":"i","type":{index_ty}}}],"results":[{{"name":"result0","type":{element_ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"Index","type":{element_ty},"base":{{"var":"values"}},"index":{{"var":"i"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            array_ty = array_ty,
            index_ty = index_ty,
            element_ty = bv_type(64, true),
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert_eq!(module.obligations.len(), 2);
        assert_eq!(
            module.obligations[0].id,
            "example/pkg.Pick.entry.entry.t0.index_nonnegative"
        );
        assert_eq!(
            module.obligations[0].conclusion,
            apply(
                format!("{STD_BITVEC_MODULE}.BV64.sge"),
                vec![var("i"), zero(64, true)]
            )
        );
        assert_eq!(
            module.obligations[1].id,
            "example/pkg.Pick.entry.entry.t0.index_in_bounds"
        );
        assert_eq!(
            module.obligations[1].conclusion,
            apply(
                format!("{STD_BITVEC_MODULE}.BV64.slt"),
                vec![var("i"), literal(2, 64, true)]
            )
        );
    }

    #[test]
    fn emits_branch_path_assumptions_for_safety_obligations() {
        let ty = bv_type(64, true);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.BranchDiv","package":"example/pkg","name":"BranchDiv","params":[{{"name":"a","type":{ty}}},{{"name":"b","type":{ty}}},{{"name":"cond","type":{{"kind":"bool"}}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[],"terminator":{{"kind":"Branch","cond":{{"var":"cond"}},"then_label":"then","else_label":"else"}}}},{{"label":"then","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"bv_sdiv","type":{ty},"lhs":{{"var":"a"}},"rhs":{{"var":"b"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}},{{"label":"else","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[{{"var":"a"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = ty,
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert_eq!(module.obligations.len(), 1);
        assert_eq!(
            module.obligations[0].id,
            "example/pkg.BranchDiv.then.then.t0.divisor_nonzero"
        );
        assert_eq!(module.obligations[0].assumptions, vec![var("cond")]);
    }

    #[test]
    fn branch_condition_can_use_comparison_instruction() {
        let ty = bv_type(64, true);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.CompareBranch","package":"example/pkg","name":"CompareBranch","params":[{{"name":"a","type":{ty}}},{{"name":"b","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"signed_gt","type":{{"kind":"bool"}},"lhs":{{"var":"a"}},"rhs":{{"var":"b"}}}}],"terminator":{{"kind":"Branch","cond":{{"var":"t0"}},"then_label":"then","else_label":"else"}}}},{{"label":"then","parameters":[],"instructions":[{{"id":"t1","kind":"BinOp","op":"bv_sdiv","type":{ty},"lhs":{{"var":"a"}},"rhs":{{"var":"b"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t1"}}]}}}},{{"label":"else","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[{{"var":"a"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = ty,
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert_eq!(module.obligations.len(), 1);
        assert_eq!(
            module.obligations[0].assumptions,
            vec![apply(
                format!("{STD_BITVEC_MODULE}.BV64.sgt"),
                vec![var("a"), var("b")]
            )]
        );
    }

    #[test]
    fn unsigned_shift_count_emits_no_obligation() {
        let value_ty = bv_type(64, true);
        let count_ty = bv_type(64, false);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.UnsignedShift","package":"example/pkg","name":"UnsignedShift","params":[{{"name":"value","type":{value_ty}}},{{"name":"count","type":{count_ty}}}],"results":[{{"name":"result0","type":{value_ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"BinOp","op":"bv_lshr","type":{value_ty},"lhs":{{"var":"value"}},"rhs":{{"var":"count"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            value_ty = value_ty,
            count_ty = count_ty,
            contracts = contracts(),
        );

        let module = generate(&input).expect("safety VCs generate");

        assert!(module.obligations.is_empty());
    }

    #[test]
    fn rejects_extra_blocks_for_straight_line_safety_generation() {
        let ty = bv_type(64, true);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.ExtraBlock","package":"example/pkg","name":"ExtraBlock","params":[{{"name":"value","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[{{"var":"value"}}]}}}},{{"label":"dead","parameters":[],"instructions":[],"terminator":{{"kind":"Return","values":[{{"var":"value"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = ty,
            contracts = contracts(),
        );

        let error = generate(&input).expect_err("extra straight-line block rejects");

        assert_eq!(
            error,
            WpError::UnsupportedBlockCount {
                function_id: "example/pkg.ExtraBlock".to_owned(),
                block_count: 2,
            }
        );
    }

    #[test]
    fn rejects_index_against_non_array_base() {
        let ty = bv_type(64, true);
        let input = format!(
            r#"{{"schema_version":"mpk.gir.v0","packages":[{{"package_path":"example/pkg","name":"example","functions":[{{"id":"example/pkg.BadIndex","package":"example/pkg","name":"BadIndex","params":[{{"name":"value","type":{ty}}},{{"name":"i","type":{ty}}}],"results":[{{"name":"result0","type":{ty}}}],"locals":[],"blocks":[{{"label":"entry","parameters":[],"instructions":[{{"id":"t0","kind":"Index","type":{ty},"base":{{"var":"value"}},"index":{{"var":"i"}}}}],"terminator":{{"kind":"Return","values":[{{"var":"t0"}}]}}}}],{contracts},"supported_features":[],"rejected_features":[]}}]}}]}}"#,
            ty = ty,
            contracts = contracts(),
        );

        let error = generate(&input).expect_err("non-array base rejects");

        assert_eq!(
            error,
            WpError::UnsupportedSafetyCheck {
                function_id: "example/pkg.BadIndex".to_owned(),
                block_label: "entry".to_owned(),
                instruction_id: "t0".to_owned(),
                reason: "base is not a fixed array".to_owned(),
            }
        );
    }
}
