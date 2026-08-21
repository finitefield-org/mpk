//! Straight-line weakest-precondition VC generation for GIR functions.
//!
//! This module handles the VC-004 subset: one basic block, supported pure
//! expression instructions, and a `Return` terminator. Branches, loops, runtime
//! safety checks, and calls are left to later VC milestones.

use std::collections::BTreeMap;
use std::fmt;

use crate::expr_encode::{ExprEncodeError, ExprEncoder, MpkExprTerm};
use crate::gir::{
    GirBlock, GirContractExpr, GirFunction, GirInstruction, GirInstructionKind, GirModule,
    GirTerminatorKind, GirValue,
};
use crate::vc::{VcModule, VcObligation, VcObligationKind};

pub fn generate_straight_line_vcs(input: &GirModule) -> Result<VcModule, WpError> {
    WpGenerator::new().generate_module(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WpGenerator;

impl WpGenerator {
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
        if function.blocks.len() != 1 {
            return Err(WpError::UnsupportedBlockCount {
                function_id: function.id.clone(),
                block_count: function.blocks.len(),
            });
        }

        let block = &function.blocks[0];
        if !block.parameters.is_empty() {
            return Err(WpError::BlockParametersUnsupported {
                function_id: function.id.clone(),
                block_label: block.label.clone(),
                parameter_count: block.parameters.len(),
            });
        }
        if block.terminator.kind != GirTerminatorKind::Return {
            return Err(WpError::UnsupportedTerminator {
                function_id: function.id.clone(),
                block_label: block.label.clone(),
                kind: block.terminator.kind,
            });
        }
        if block.terminator.values.len() != function.results.len() {
            return Err(WpError::ReturnArityMismatch {
                function_id: function.id.clone(),
                expected: function.results.len(),
                actual: block.terminator.values.len(),
            });
        }

        let encoder = ExprEncoder::for_function(function);
        let initial_env = initial_environment(function);
        let assumptions = encode_requires(function, &encoder, &initial_env)?;
        let final_env = self.execute_block(function, block, &encoder, initial_env.clone())?;
        let result_terms = encode_return_terms(function, &encoder, &final_env)?;

        function
            .contracts
            .ensures
            .iter()
            .enumerate()
            .map(|(index, ensure)| {
                validate_contract_references(function, ensure, &initial_env, true, "ensures")?;
                let encoded =
                    encoder
                        .encode_contract_expr(ensure)
                        .map_err(|source| WpError::Expression {
                            function_id: function.id.clone(),
                            context: format!("ensures[{index}]"),
                            source,
                        })?;
                let conclusion = substitute_term(&encoded, &final_env, &result_terms);
                Ok(VcObligation {
                    id: format!("{}.post{index}", function.id),
                    function_id: function.id.clone(),
                    kind: VcObligationKind::Postcondition,
                    assumptions: assumptions.clone(),
                    conclusion,
                })
            })
            .collect()
    }

    pub(crate) fn execute_block(
        self,
        function: &GirFunction,
        block: &GirBlock,
        encoder: &ExprEncoder,
        mut env: BTreeMap<String, MpkExprTerm>,
    ) -> Result<BTreeMap<String, MpkExprTerm>, WpError> {
        for instruction in &block.instructions {
            self.execute_instruction(function, block, encoder, instruction, &mut env)?;
        }
        Ok(env)
    }

    pub(crate) fn execute_instruction(
        self,
        function: &GirFunction,
        block: &GirBlock,
        encoder: &ExprEncoder,
        instruction: &GirInstruction,
        env: &mut BTreeMap<String, MpkExprTerm>,
    ) -> Result<(), WpError> {
        ensure_supported_instruction(function, block, instruction)?;
        ensure_instruction_shape(function, block, instruction)?;
        validate_instruction_references(function, instruction, env)?;

        let encoded =
            encoder
                .encode_instruction(instruction)
                .map_err(|source| WpError::Expression {
                    function_id: function.id.clone(),
                    context: format!("instruction {}", instruction.id),
                    source,
                })?;
        let value = substitute_term(&encoded, env, &BTreeMap::new());

        if instruction.id.is_empty() {
            return Err(WpError::EmptyInstructionId {
                function_id: function.id.clone(),
                block_label: block.label.clone(),
            });
        }
        env.insert(instruction.id.clone(), value.clone());

        if instruction.kind == GirInstructionKind::Copy {
            if let Some(target) = instruction
                .target
                .as_deref()
                .filter(|target| !target.is_empty())
            {
                if !env.contains_key(target) {
                    return Err(WpError::UnknownVariable {
                        function_id: function.id.clone(),
                        context: format!("copy target {}", instruction.id),
                        name: target.to_owned(),
                    });
                }
                env.insert(target.to_owned(), value);
            }
        }

        Ok(())
    }
}

pub(crate) fn initial_environment(function: &GirFunction) -> BTreeMap<String, MpkExprTerm> {
    let mut env = BTreeMap::new();
    for binding in function.params.iter().chain(function.locals.iter()) {
        if !binding.name.is_empty() {
            env.insert(
                binding.name.clone(),
                MpkExprTerm::Var {
                    name: binding.name.clone(),
                },
            );
        }
    }
    env
}

pub(crate) fn encode_requires(
    function: &GirFunction,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<Vec<MpkExprTerm>, WpError> {
    function
        .contracts
        .requires
        .iter()
        .enumerate()
        .map(|(index, require)| {
            validate_contract_references(function, require, env, false, "requires")?;
            let encoded =
                encoder
                    .encode_contract_expr(require)
                    .map_err(|source| WpError::Expression {
                        function_id: function.id.clone(),
                        context: format!("requires[{index}]"),
                        source,
                    })?;
            Ok(substitute_term(&encoded, env, &BTreeMap::new()))
        })
        .collect()
}

fn encode_return_terms(
    function: &GirFunction,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<BTreeMap<u32, MpkExprTerm>, WpError> {
    function
        .blocks
        .first()
        .expect("caller validates a single block")
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

pub(crate) fn ensure_supported_instruction(
    function: &GirFunction,
    block: &GirBlock,
    instruction: &GirInstruction,
) -> Result<(), WpError> {
    match instruction.kind {
        GirInstructionKind::Const
        | GirInstructionKind::Copy
        | GirInstructionKind::BinOp
        | GirInstructionKind::UnaryOp
        | GirInstructionKind::Convert => Ok(()),
        kind => Err(WpError::UnsupportedInstructionKind {
            function_id: function.id.clone(),
            block_label: block.label.clone(),
            instruction_id: instruction.id.clone(),
            kind,
        }),
    }
}

pub(crate) fn ensure_instruction_shape(
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
        _ => None,
    };

    if let Some(reason) = reason {
        return Err(WpError::UnsupportedInstructionShape {
            function_id: function.id.clone(),
            block_label: block.label.clone(),
            instruction_id: instruction.id.clone(),
            kind: instruction.kind,
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

pub(crate) fn validate_instruction_references(
    function: &GirFunction,
    instruction: &GirInstruction,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<(), WpError> {
    if let Some(value) = &instruction.value {
        validate_value_reference(
            function,
            value,
            env,
            format!("instruction {} value", instruction.id),
        )?;
    }
    if let Some(lhs) = &instruction.lhs {
        validate_value_reference(
            function,
            lhs,
            env,
            format!("instruction {} lhs", instruction.id),
        )?;
    }
    if let Some(rhs) = &instruction.rhs {
        validate_value_reference(
            function,
            rhs,
            env,
            format!("instruction {} rhs", instruction.id),
        )?;
    }
    Ok(())
}

pub(crate) fn validate_value_reference(
    function: &GirFunction,
    value: &GirValue,
    env: &BTreeMap<String, MpkExprTerm>,
    context: impl Into<String>,
) -> Result<(), WpError> {
    if let Some(name) = &value.var {
        if !env.contains_key(name) {
            return Err(WpError::UnknownVariable {
                function_id: function.id.clone(),
                context: context.into(),
                name: name.clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_contract_references(
    function: &GirFunction,
    input: &GirContractExpr,
    env: &BTreeMap<String, MpkExprTerm>,
    allow_results: bool,
    context: &'static str,
) -> Result<(), WpError> {
    if let Some(name) = &input.var {
        if !env.contains_key(name) {
            return Err(WpError::UnknownVariable {
                function_id: function.id.clone(),
                context: context.to_owned(),
                name: name.clone(),
            });
        }
    }
    if let Some(index) = input.result {
        if !allow_results {
            return Err(WpError::ResultInRequires {
                function_id: function.id.clone(),
                result_index: index,
            });
        }
        if index as usize >= function.results.len() {
            return Err(WpError::UnknownResult {
                function_id: function.id.clone(),
                context: context.to_owned(),
                index,
            });
        }
    }
    for arg in &input.args {
        validate_contract_references(function, arg, env, allow_results, context)?;
    }
    if let Some(lhs) = &input.lhs {
        validate_contract_references(function, lhs, env, allow_results, context)?;
    }
    if let Some(rhs) = &input.rhs {
        validate_contract_references(function, rhs, env, allow_results, context)?;
    }
    if let Some(value) = &input.value {
        validate_contract_references(function, value, env, allow_results, context)?;
    }
    Ok(())
}

pub(crate) fn substitute_term(
    input: &MpkExprTerm,
    variables: &BTreeMap<String, MpkExprTerm>,
    results: &BTreeMap<u32, MpkExprTerm>,
) -> MpkExprTerm {
    match input {
        MpkExprTerm::Var { name } => variables
            .get(name)
            .cloned()
            .unwrap_or_else(|| input.clone()),
        MpkExprTerm::Result { index } => {
            results.get(index).cloned().unwrap_or_else(|| input.clone())
        }
        MpkExprTerm::Bound { .. }
        | MpkExprTerm::Constant { .. }
        | MpkExprTerm::BitVecLiteral { .. } => input.clone(),
        MpkExprTerm::Apply { function, args } => MpkExprTerm::Apply {
            function: function.clone(),
            args: args
                .iter()
                .map(|arg| substitute_term(arg, variables, results))
                .collect(),
        },
        MpkExprTerm::Convert { value, target } => MpkExprTerm::Convert {
            value: Box::new(substitute_term(value, variables, results)),
            target: target.clone(),
        },
        MpkExprTerm::Forall { binder_type, body } => MpkExprTerm::Forall {
            binder_type: binder_type.clone(),
            body: Box::new(substitute_term(body, variables, results)),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WpError {
    FunctionHasRejectedFeatures {
        function_id: String,
        rejected_feature_count: usize,
    },
    MissingPostcondition {
        function_id: String,
    },
    NonEmptyModifies {
        function_id: String,
        modifies: Vec<String>,
    },
    LoopContractsUnsupported {
        function_id: String,
        loop_count: usize,
    },
    UnsupportedBlockCount {
        function_id: String,
        block_count: usize,
    },
    BlockParametersUnsupported {
        function_id: String,
        block_label: String,
        parameter_count: usize,
    },
    UnsupportedTerminator {
        function_id: String,
        block_label: String,
        kind: GirTerminatorKind,
    },
    MissingBranchCondition {
        function_id: String,
        block_label: String,
    },
    MissingBranchLabel {
        function_id: String,
        block_label: String,
        label_kind: &'static str,
    },
    UnknownBlockLabel {
        function_id: String,
        context: String,
        block_label: String,
    },
    CyclicBranchPath {
        function_id: String,
        block_label: String,
    },
    UnsupportedTerminatorShape {
        function_id: String,
        block_label: String,
        kind: GirTerminatorKind,
        reason: &'static str,
    },
    MissingLoopInvariant {
        function_id: String,
        block_label: String,
    },
    UnsupportedLoopShape {
        function_id: String,
        block_label: String,
        reason: String,
    },
    UnsupportedLoopVariant {
        function_id: String,
        block_label: String,
        variant_index: usize,
        reason: String,
    },
    ReturnArityMismatch {
        function_id: String,
        expected: usize,
        actual: usize,
    },
    UnsupportedInstructionKind {
        function_id: String,
        block_label: String,
        instruction_id: String,
        kind: GirInstructionKind,
    },
    UnsupportedInstructionShape {
        function_id: String,
        block_label: String,
        instruction_id: String,
        kind: GirInstructionKind,
        reason: &'static str,
    },
    EmptyInstructionId {
        function_id: String,
        block_label: String,
    },
    UnknownVariable {
        function_id: String,
        context: String,
        name: String,
    },
    UnknownResult {
        function_id: String,
        context: String,
        index: u32,
    },
    ResultInRequires {
        function_id: String,
        result_index: u32,
    },
    ReturnIndexOverflow {
        function_id: String,
        index: usize,
    },
    Expression {
        function_id: String,
        context: String,
        source: ExprEncodeError,
    },
    UnsupportedSafetyCheck {
        function_id: String,
        block_label: String,
        instruction_id: String,
        reason: String,
    },
}

impl fmt::Display for WpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FunctionHasRejectedFeatures {
                function_id,
                rejected_feature_count,
            } => write!(
                formatter,
                "function {function_id:?} has {rejected_feature_count} rejected features"
            ),
            Self::MissingPostcondition { function_id } => {
                write!(formatter, "function {function_id:?} has no postconditions")
            }
            Self::NonEmptyModifies {
                function_id,
                modifies,
            } => write!(
                formatter,
                "function {function_id:?} has non-empty modifies clause {modifies:?}"
            ),
            Self::LoopContractsUnsupported {
                function_id,
                loop_count,
            } => write!(
                formatter,
                "function {function_id:?} has {loop_count} loop contracts; straight-line VC generation does not support loops"
            ),
            Self::UnsupportedBlockCount {
                function_id,
                block_count,
            } => write!(
                formatter,
                "function {function_id:?} has {block_count} blocks; straight-line VC generation requires one block"
            ),
            Self::BlockParametersUnsupported {
                function_id,
                block_label,
                parameter_count,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} has {parameter_count} parameters; straight-line VC generation does not support block parameters"
            ),
            Self::UnsupportedTerminator {
                function_id,
                block_label,
                kind,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} has unsupported terminator {kind:?}"
            ),
            Self::MissingBranchCondition {
                function_id,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} is missing branch condition"
            ),
            Self::MissingBranchLabel {
                function_id,
                block_label,
                label_kind,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} is missing {label_kind} branch label"
            ),
            Self::UnknownBlockLabel {
                function_id,
                context,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} {context} references unknown block {block_label:?}"
            ),
            Self::CyclicBranchPath {
                function_id,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} branch path cycles at block {block_label:?}"
            ),
            Self::UnsupportedTerminatorShape {
                function_id,
                block_label,
                kind,
                reason,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} has invalid {kind:?} terminator shape: {reason}"
            ),
            Self::MissingLoopInvariant {
                function_id,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} loop block {block_label:?} has no invariants"
            ),
            Self::UnsupportedLoopShape {
                function_id,
                block_label,
                reason,
            } => write!(
                formatter,
                "function {function_id:?} loop block {block_label:?} has unsupported shape: {reason}"
            ),
            Self::UnsupportedLoopVariant {
                function_id,
                block_label,
                variant_index,
                reason,
            } => write!(
                formatter,
                "function {function_id:?} loop block {block_label:?} decreases[{variant_index}] is unsupported: {reason}"
            ),
            Self::ReturnArityMismatch {
                function_id,
                expected,
                actual,
            } => write!(
                formatter,
                "function {function_id:?} returns {actual} values; expected {expected}"
            ),
            Self::UnsupportedInstructionKind {
                function_id,
                block_label,
                instruction_id,
                kind,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} instruction {instruction_id:?} has unsupported kind {kind:?}"
            ),
            Self::UnsupportedInstructionShape {
                function_id,
                block_label,
                instruction_id,
                kind,
                reason,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} instruction {instruction_id:?} has invalid {kind:?} shape: {reason}"
            ),
            Self::EmptyInstructionId {
                function_id,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} has instruction with empty id"
            ),
            Self::UnknownVariable {
                function_id,
                context,
                name,
            } => write!(
                formatter,
                "function {function_id:?} {context} references unknown variable {name:?}"
            ),
            Self::UnknownResult {
                function_id,
                context,
                index,
            } => write!(
                formatter,
                "function {function_id:?} {context} references unknown result {index}"
            ),
            Self::ResultInRequires {
                function_id,
                result_index,
            } => write!(
                formatter,
                "function {function_id:?} requires references result {result_index}"
            ),
            Self::ReturnIndexOverflow { function_id, index } => write!(
                formatter,
                "function {function_id:?} return index {index} does not fit u32"
            ),
            Self::Expression {
                function_id,
                context,
                source,
            } => write!(
                formatter,
                "function {function_id:?} {context} expression encoding failed: {source}"
            ),
            Self::UnsupportedSafetyCheck {
                function_id,
                block_label,
                instruction_id,
                reason,
            } => write!(
                formatter,
                "function {function_id:?} block {block_label:?} instruction {instruction_id:?} cannot generate runtime-safety check: {reason}"
            ),
        }
    }
}

impl std::error::Error for WpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Expression { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gir::import_gir_json;

    fn generate(input: &str) -> Result<VcModule, WpError> {
        let gir = import_gir_json(input).expect("GIR imports");
        generate_straight_line_vcs(&gir)
    }

    fn snapshot(module: &VcModule) -> String {
        serde_json::to_string_pretty(module).expect("VC module serializes")
    }

    fn identity_gir_json() -> &'static str {
        r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Identity","package":"example/pkg","name":"Identity","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"value"}]}}],"contracts":{"requires":[],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}],"modifies":[],"loops":[]},"supported_features":["params","return"],"rejected_features":[]}]}],"gir_hash":"abc123"}"#
    }

    #[test]
    fn straight_line_identity_vc_snapshot() {
        let module = generate(identity_gir_json()).expect("straight-line VCs generate");

        assert_eq!(
            snapshot(&module),
            r#"{
  "source_gir_hash": "abc123",
  "obligations": [
    {
      "id": "example/pkg.Identity.post0",
      "function_id": "example/pkg.Identity",
      "kind": "postcondition",
      "assumptions": [],
      "conclusion": {
        "kind": "apply",
        "function": "Std.Eq",
        "args": [
          {
            "kind": "var",
            "name": "value"
          },
          {
            "kind": "var",
            "name": "value"
          }
        ]
      }
    }
  ]
}"#
        );
    }

    #[test]
    fn straight_line_add_vc_substitutes_instruction_and_return_snapshot() {
        let module = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Inc","package":"example/pkg","name":"Inc","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[{"id":"tmp0","kind":"BinOp","op":"bv_add","type":{"kind":"bv","width":64,"signed":true},"lhs":{"var":"value"},"rhs":{"int":{"value":"1","width":64,"signed":true}}}],"terminator":{"kind":"Return","values":[{"var":"tmp0"}]}}],"contracts":{"requires":[],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"op":"bv_add","lhs":{"var":"value"},"rhs":{"int":{"value":"1","width":64,"signed":true}}}}],"modifies":[],"loops":[]},"supported_features":["params","return"],"rejected_features":[]}]}],"gir_hash":"addhash"}"#,
        )
        .expect("straight-line VCs generate");

        assert_eq!(
            snapshot(&module),
            r#"{
  "source_gir_hash": "addhash",
  "obligations": [
    {
      "id": "example/pkg.Inc.post0",
      "function_id": "example/pkg.Inc",
      "kind": "postcondition",
      "assumptions": [],
      "conclusion": {
        "kind": "apply",
        "function": "Std.Eq",
        "args": [
          {
            "kind": "apply",
            "function": "Std.BitVec.BV64.add",
            "args": [
              {
                "kind": "var",
                "name": "value"
              },
              {
                "kind": "bit_vec_literal",
                "value": "1",
                "width": 64,
                "signed": true
              }
            ]
          },
          {
            "kind": "apply",
            "function": "Std.BitVec.BV64.add",
            "args": [
              {
                "kind": "var",
                "name": "value"
              },
              {
                "kind": "bit_vec_literal",
                "value": "1",
                "width": 64,
                "signed": true
              }
            ]
          }
        ]
      }
    }
  ]
}"#
        );
    }

    #[test]
    fn requires_become_obligation_assumptions_snapshot() {
        let module = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.NonNegativeIdentity","package":"example/pkg","name":"NonNegativeIdentity","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"value"}]}}],"contracts":{"requires":[{"op":"signed_ge","lhs":{"var":"value"},"rhs":{"int":{"value":"0","width":64,"signed":true}}}],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}],"modifies":[],"loops":[]},"supported_features":["params","return"],"rejected_features":[]}]}],"gir_hash":"prehash"}"#,
        )
        .expect("straight-line VCs generate");

        assert_eq!(
            snapshot(&module),
            r#"{
  "source_gir_hash": "prehash",
  "obligations": [
    {
      "id": "example/pkg.NonNegativeIdentity.post0",
      "function_id": "example/pkg.NonNegativeIdentity",
      "kind": "postcondition",
      "assumptions": [
        {
          "kind": "apply",
          "function": "Std.BitVec.BV64.sge",
          "args": [
            {
              "kind": "var",
              "name": "value"
            },
            {
              "kind": "bit_vec_literal",
              "value": "0",
              "width": 64,
              "signed": true
            }
          ]
        }
      ],
      "conclusion": {
        "kind": "apply",
        "function": "Std.Eq",
        "args": [
          {
            "kind": "var",
            "name": "value"
          },
          {
            "kind": "var",
            "name": "value"
          }
        ]
      }
    }
  ]
}"#
        );
    }

    #[test]
    fn rejects_missing_postconditions() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.NoPost","package":"example/pkg","name":"NoPost","params":[],"results":[],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[]}}],"contracts":{"requires":[],"ensures":[],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("missing postcondition rejects");

        assert_eq!(
            error,
            WpError::MissingPostcondition {
                function_id: "example/pkg.NoPost".to_owned()
            }
        );
    }

    #[test]
    fn rejects_branch_terminator_for_straight_line_generation() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Branch","package":"example/pkg","name":"Branch","params":[{"name":"cond","type":{"kind":"bool"}}],"results":[],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":{"var":"cond"},"then_label":"then","else_label":"else"}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("branch terminator rejects");

        assert_eq!(
            error,
            WpError::UnsupportedTerminator {
                function_id: "example/pkg.Branch".to_owned(),
                block_label: "entry".to_owned(),
                kind: GirTerminatorKind::Branch
            }
        );
    }

    #[test]
    fn rejects_unknown_return_variable() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.BadReturn","package":"example/pkg","name":"BadReturn","params":[],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"missing"}]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("unknown return variable rejects");

        assert_eq!(
            error,
            WpError::UnknownVariable {
                function_id: "example/pkg.BadReturn".to_owned(),
                context: "return[0]".to_owned(),
                name: "missing".to_owned()
            }
        );
    }

    #[test]
    fn rejects_extra_payload_fields_on_supported_instruction_kind() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.BadInstructionShape","package":"example/pkg","name":"BadInstructionShape","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[{"id":"tmp0","kind":"BinOp","op":"bv_add","type":{"kind":"bv","width":64,"signed":true},"value":{"var":"value"},"lhs":{"var":"value"},"rhs":{"int":{"value":"1","width":64,"signed":true}}}],"terminator":{"kind":"Return","values":[{"var":"tmp0"}]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("extra instruction payload rejects");

        assert_eq!(
            error,
            WpError::UnsupportedInstructionShape {
                function_id: "example/pkg.BadInstructionShape".to_owned(),
                block_label: "entry".to_owned(),
                instruction_id: "tmp0".to_owned(),
                kind: GirInstructionKind::BinOp,
                reason: "BinOp cannot have value"
            }
        );
    }

    #[test]
    fn rejects_instruction_temp_referenced_by_contract() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.TempContract","package":"example/pkg","name":"TempContract","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[{"id":"tmp0","kind":"BinOp","op":"bv_add","type":{"kind":"bv","width":64,"signed":true},"lhs":{"var":"value"},"rhs":{"int":{"value":"1","width":64,"signed":true}}}],"terminator":{"kind":"Return","values":[{"var":"tmp0"}]}}],"contracts":{"requires":[],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"tmp0"}}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("contract temp reference rejects");

        assert_eq!(
            error,
            WpError::UnknownVariable {
                function_id: "example/pkg.TempContract".to_owned(),
                context: "ensures".to_owned(),
                name: "tmp0".to_owned()
            }
        );
    }

    #[test]
    fn rejects_result_reference_in_requires() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.BadRequire","package":"example/pkg","name":"BadRequire","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"value"}]}}],"contracts":{"requires":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("requires result reference rejects");

        assert_eq!(
            error,
            WpError::ResultInRequires {
                function_id: "example/pkg.BadRequire".to_owned(),
                result_index: 0
            }
        );
    }
}
