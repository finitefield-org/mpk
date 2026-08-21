//! Shared VIR v0 value and contract-expression encoding.
//!
//! Value operations are profile-independent. A validated semantic profile is
//! carried by the context so profile-specific source/contract admissibility
//! and target-sized types cannot be confused with the total value semantics.

use std::collections::BTreeMap;
use std::fmt;

use crate::expr_encode::{
    MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_IF, STD_BOOL_NOT, STD_BOOL_OR, STD_EQ,
};
use crate::semantic_profile::{
    validate_semantic_parameters, SemanticParameters, SemanticProfile, SemanticProfileError,
};
use crate::type_encode::{ProgramTypeEncoder, TypeEncodeError};
use crate::vir::{
    VirBinaryOperator, VirConstDecl, VirContractConvertOperator, VirContractExpr,
    VirContractNaryOperator, VirContractUnaryOperator, VirFunction, VirInstruction,
    VirInstructionKind, VirIntLiteral, VirLiteral, VirModule, VirStructDecl, VirType,
    VirUnaryOperator, VirUnit, VirValue,
};
use crate::vir_validate::{validate_vir, VirValidationError};

pub fn encode_vir_value(
    context: &ProgramExprContext,
    input: &VirValue,
) -> Result<MpkExprTerm, ProgramExprEncodeError> {
    ProgramExprEncoder::new(context).encode_value(input)
}

pub fn encode_vir_contract_expr(
    context: &ProgramExprContext,
    input: &VirContractExpr,
) -> Result<MpkExprTerm, ProgramExprEncodeError> {
    ProgramExprEncoder::new(context).encode_contract_expr(input)
}

pub fn encode_vir_instruction_expr(
    context: &ProgramExprContext,
    input: &VirInstruction,
) -> Result<MpkExprTerm, ProgramExprEncodeError> {
    ProgramExprEncoder::new(context).encode_instruction(input)
}

/// Executable reference for the total VIR v0 bitvector equations.
///
/// The encoder never constant-folds with this helper; it exists so foundation
/// and frontend conformance vectors can check the exact zero-divisor,
/// `MIN/-1`, and full-count shift results independently of a host language's
/// overflow behavior.
pub fn evaluate_total_bitvector_operation(
    operation: VirBinaryOperator,
    width: crate::vir::BitVectorWidth,
    lhs: u64,
    rhs_width: crate::vir::BitVectorWidth,
    rhs: u64,
) -> Result<TotalBitVectorResult, ProgramExprEncodeError> {
    let width = width.bits();
    let rhs_width = rhs_width.bits();
    validate_bit_pattern("lhs", lhs, width)?;
    validate_bit_pattern("rhs", rhs, rhs_width)?;
    if width != rhs_width
        && !matches!(
            operation,
            VirBinaryOperator::BvShl | VirBinaryOperator::BvAshr | VirBinaryOperator::BvLshr
        )
    {
        return Err(ProgramExprEncodeError::BitVectorWidthMismatch {
            operation: vir_binary_name(operation),
            lhs_width: width,
            rhs_width,
        });
    }
    let mask = bit_mask(width);
    let value = match operation {
        VirBinaryOperator::Eq => return Ok(TotalBitVectorResult::Boolean(lhs == rhs)),
        VirBinaryOperator::NotEq => return Ok(TotalBitVectorResult::Boolean(lhs != rhs)),
        VirBinaryOperator::BvAdd => lhs.wrapping_add(rhs) & mask,
        VirBinaryOperator::BvSub => lhs.wrapping_sub(rhs) & mask,
        VirBinaryOperator::BvMul => lhs.wrapping_mul(rhs) & mask,
        VirBinaryOperator::BvAnd => lhs & rhs,
        VirBinaryOperator::BvOr => lhs | rhs,
        VirBinaryOperator::BvXor => lhs ^ rhs,
        VirBinaryOperator::BvUdiv => lhs.checked_div(rhs).unwrap_or(mask),
        VirBinaryOperator::BvUrem => lhs.checked_rem(rhs).unwrap_or(lhs),
        VirBinaryOperator::BvSdiv => signed_division(lhs, rhs, width),
        VirBinaryOperator::BvSrem => signed_remainder(lhs, rhs, width),
        VirBinaryOperator::BvShl => {
            if rhs >= u64::from(width) {
                0
            } else {
                lhs.wrapping_shl(rhs as u32) & mask
            }
        }
        VirBinaryOperator::BvLshr => {
            if rhs >= u64::from(width) {
                0
            } else {
                lhs >> rhs
            }
        }
        VirBinaryOperator::BvAshr => {
            if rhs >= u64::from(width) {
                if signed_value(lhs, width) < 0 {
                    mask
                } else {
                    0
                }
            } else {
                wrap_signed(signed_value(lhs, width) >> rhs, width)
            }
        }
        VirBinaryOperator::SignedLt => {
            return Ok(TotalBitVectorResult::Boolean(
                signed_value(lhs, width) < signed_value(rhs, width),
            ));
        }
        VirBinaryOperator::SignedLe => {
            return Ok(TotalBitVectorResult::Boolean(
                signed_value(lhs, width) <= signed_value(rhs, width),
            ));
        }
        VirBinaryOperator::SignedGt => {
            return Ok(TotalBitVectorResult::Boolean(
                signed_value(lhs, width) > signed_value(rhs, width),
            ));
        }
        VirBinaryOperator::SignedGe => {
            return Ok(TotalBitVectorResult::Boolean(
                signed_value(lhs, width) >= signed_value(rhs, width),
            ));
        }
        VirBinaryOperator::UnsignedLt => return Ok(TotalBitVectorResult::Boolean(lhs < rhs)),
        VirBinaryOperator::UnsignedLe => return Ok(TotalBitVectorResult::Boolean(lhs <= rhs)),
        VirBinaryOperator::UnsignedGt => return Ok(TotalBitVectorResult::Boolean(lhs > rhs)),
        VirBinaryOperator::UnsignedGe => return Ok(TotalBitVectorResult::Boolean(lhs >= rhs)),
    };
    Ok(TotalBitVectorResult::BitVector(value))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TotalBitVectorResult {
    BitVector(u64),
    Boolean(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramExprContext {
    profile: SemanticProfile,
    parameters: SemanticParameters,
    variables: BTreeMap<String, VirType>,
    results: BTreeMap<u32, VirType>,
    constants: BTreeMap<String, (VirType, VirLiteral)>,
    declarations: Vec<VirStructDecl>,
}

impl ProgramExprContext {
    pub fn new(
        profile: SemanticProfile,
        parameters: SemanticParameters,
        declarations: Vec<VirStructDecl>,
    ) -> Result<Self, ProgramExprEncodeError> {
        validate_semantic_parameters(profile, &parameters)
            .map_err(ProgramExprEncodeError::SemanticProfile)?;
        ProgramTypeEncoder::new(profile, &parameters, &declarations)?;
        Ok(Self {
            profile,
            parameters,
            variables: BTreeMap::new(),
            results: BTreeMap::new(),
            constants: BTreeMap::new(),
            declarations,
        })
    }

    /// Builds an encoder context only after validating the complete VIR module.
    pub fn for_function(
        module: &VirModule,
        unit: &VirUnit,
        function: &VirFunction,
    ) -> Result<Self, ProgramExprEncodeError> {
        validate_vir(module).map_err(ProgramExprEncodeError::Validation)?;
        let validated_unit = module
            .units
            .iter()
            .find(|candidate| candidate.id == unit.id)
            .filter(|candidate| *candidate == unit)
            .ok_or_else(|| ProgramExprEncodeError::UnitNotInModule {
                unit_id: unit.id.clone(),
            })?;
        let validated_function = validated_unit
            .functions
            .iter()
            .find(|candidate| candidate.id == function.id)
            .filter(|candidate| *candidate == function)
            .ok_or_else(|| ProgramExprEncodeError::FunctionNotInUnit {
                function_id: function.id.clone(),
                unit_id: unit.id.clone(),
            })?;

        Self::for_validated_function(module, validated_unit, validated_function)
    }

    pub(crate) fn for_validated_function(
        module: &VirModule,
        validated_unit: &VirUnit,
        validated_function: &VirFunction,
    ) -> Result<Self, ProgramExprEncodeError> {
        let mut context = Self::new(
            module.semantic_profile,
            module.semantic_parameters.clone(),
            validated_unit.type_decls.clone(),
        )?;
        for binding in validated_function
            .params
            .iter()
            .chain(validated_function.locals.iter())
            .chain(
                validated_function
                    .blocks
                    .iter()
                    .flat_map(|block| block.parameters.iter()),
            )
        {
            context
                .variables
                .insert(binding.id.clone(), binding.r#type.clone());
        }
        for (index, binding) in validated_function.results.iter().enumerate() {
            let index =
                u32::try_from(index).map_err(|_| ProgramExprEncodeError::ResultIndexOverflow {
                    function_id: validated_function.id.clone(),
                })?;
            context.results.insert(index, binding.r#type.clone());
        }
        for instruction in validated_function
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
        {
            context.variables.insert(
                instruction_id(instruction).to_owned(),
                instruction_type(instruction).clone(),
            );
        }
        for declaration in &validated_unit.const_decls {
            context.insert_constant(declaration);
        }
        Ok(context)
    }

    pub fn with_variable(mut self, id: impl Into<String>, r#type: VirType) -> Self {
        self.variables.insert(id.into(), r#type);
        self
    }

    pub fn with_result(mut self, index: u32, r#type: VirType) -> Self {
        self.results.insert(index, r#type);
        self
    }

    pub fn with_constant(mut self, declaration: VirConstDecl) -> Self {
        self.insert_constant(&declaration);
        self
    }

    pub const fn profile(&self) -> SemanticProfile {
        self.profile
    }

    pub fn parameters(&self) -> &SemanticParameters {
        &self.parameters
    }

    pub fn declarations(&self) -> &[VirStructDecl] {
        &self.declarations
    }

    /// Resolves the exact VIR type of a value in this validated context.
    pub fn value_type(&self, input: &VirValue) -> Result<VirType, ProgramExprEncodeError> {
        match input {
            VirValue::Variable(reference) => {
                self.variables.get(&reference.var).cloned().ok_or_else(|| {
                    ProgramExprEncodeError::UnknownVariable {
                        id: reference.var.clone(),
                    }
                })
            }
            VirValue::Constant(reference) => self
                .constants
                .get(&reference.constant)
                .map(|(r#type, _)| r#type.clone())
                .ok_or_else(|| ProgramExprEncodeError::UnknownConstant {
                    id: reference.constant.clone(),
                }),
            VirValue::Boolean(_) => Ok(VirType::Bool {}),
            VirValue::Integer(literal) => Ok(VirType::Bv {
                width: literal.int.width,
                signed: literal.int.signed,
            }),
        }
    }

    fn insert_constant(&mut self, declaration: &VirConstDecl) {
        self.constants.insert(
            declaration.id.clone(),
            (declaration.r#type.clone(), declaration.value.clone()),
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProgramExprEncoder<'a> {
    context: &'a ProgramExprContext,
}

impl<'a> ProgramExprEncoder<'a> {
    pub const fn new(context: &'a ProgramExprContext) -> Self {
        Self { context }
    }

    pub fn encode_value(&self, input: &VirValue) -> Result<MpkExprTerm, ProgramExprEncodeError> {
        Ok(self.encode_value_typed(input)?.term)
    }

    pub fn encode_contract_expr(
        &self,
        input: &VirContractExpr,
    ) -> Result<MpkExprTerm, ProgramExprEncodeError> {
        Ok(self.encode_contract_expr_typed(input)?.term)
    }

    pub fn encode_instruction(
        &self,
        input: &VirInstruction,
    ) -> Result<MpkExprTerm, ProgramExprEncodeError> {
        let encoded = match input {
            VirInstruction::Const { r#type, value, .. } => {
                let encoded = self.encode_literal_typed(value)?;
                require_type("Const", r#type, &encoded.ty)?;
                encoded
            }
            VirInstruction::Copy { r#type, value, .. } => {
                let encoded = self.encode_value_typed(value)?;
                require_type("Copy", r#type, &encoded.ty)?;
                encoded
            }
            VirInstruction::BinOp {
                op,
                r#type,
                lhs,
                rhs,
                ..
            } => self.encode_binary(
                *op,
                self.encode_value_typed(lhs)?,
                self.encode_value_typed(rhs)?,
                Some(r#type),
                false,
            )?,
            VirInstruction::UnaryOp {
                op, r#type, value, ..
            } => self.encode_unary(
                vir_unary_name(*op),
                self.encode_value_typed(value)?,
                Some(r#type),
            )?,
            VirInstruction::Convert { r#type, value, .. } => {
                if self.context.profile == SemanticProfile::RustCheckedV0 {
                    return Err(ProgramExprEncodeError::ProfileOperation {
                        profile: self.context.profile,
                        operation: "convert",
                    });
                }
                self.encode_convert(self.encode_value_typed(value)?, r#type)?
            }
            other => {
                return Err(ProgramExprEncodeError::UnsupportedInstruction { kind: other.kind() });
            }
        };
        Ok(encoded.term)
    }

    fn encode_value_typed(
        &self,
        input: &VirValue,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        match input {
            VirValue::Variable(reference) => {
                let ty = self
                    .context
                    .variables
                    .get(&reference.var)
                    .cloned()
                    .ok_or_else(|| ProgramExprEncodeError::UnknownVariable {
                        id: reference.var.clone(),
                    })?;
                Ok(TypedProgramExpr {
                    term: MpkExprTerm::Var {
                        name: reference.var.clone(),
                    },
                    ty,
                })
            }
            VirValue::Constant(reference) => {
                let (ty, literal) =
                    self.context
                        .constants
                        .get(&reference.constant)
                        .ok_or_else(|| ProgramExprEncodeError::UnknownConstant {
                            id: reference.constant.clone(),
                        })?;
                let encoded = self.encode_literal_typed(literal)?;
                require_type("constant", ty, &encoded.ty)?;
                Ok(encoded)
            }
            VirValue::Boolean(literal) => Ok(TypedProgramExpr {
                term: MpkExprTerm::bool_literal(literal.value),
                ty: VirType::Bool {},
            }),
            VirValue::Integer(literal) => self.encode_int_literal(&literal.int),
        }
    }

    fn encode_literal_typed(
        &self,
        input: &VirLiteral,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        match input {
            VirLiteral::Boolean(literal) => Ok(TypedProgramExpr {
                term: MpkExprTerm::bool_literal(literal.value),
                ty: VirType::Bool {},
            }),
            VirLiteral::Integer(literal) => self.encode_int_literal(&literal.int),
        }
    }

    fn encode_int_literal(
        &self,
        input: &VirIntLiteral,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        validate_int_literal(input)?;
        Ok(TypedProgramExpr {
            term: MpkExprTerm::BitVecLiteral {
                value: input.value.as_str().to_owned(),
                width: input.width.bits(),
                signed: input.signed,
            },
            ty: VirType::Bv {
                width: input.width,
                signed: input.signed,
            },
        })
    }

    fn encode_contract_expr_typed(
        &self,
        input: &VirContractExpr,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        match input {
            VirContractExpr::Variable(reference) => {
                self.encode_value_typed(&VirValue::Variable(reference.clone()))
            }
            VirContractExpr::Result(reference) => {
                let ty = self.context.results.get(&reference.result).cloned().ok_or(
                    ProgramExprEncodeError::UnknownResult {
                        index: reference.result,
                    },
                )?;
                Ok(TypedProgramExpr {
                    term: MpkExprTerm::Result {
                        index: reference.result,
                    },
                    ty,
                })
            }
            VirContractExpr::Boolean(literal) => Ok(TypedProgramExpr {
                term: MpkExprTerm::bool_literal(literal.value),
                ty: VirType::Bool {},
            }),
            VirContractExpr::Integer(literal) => self.encode_int_literal(&literal.int),
            VirContractExpr::Unary(expression) => self.encode_unary(
                contract_unary_name(expression.op),
                self.encode_contract_expr_typed(&expression.value)?,
                None,
            ),
            VirContractExpr::Nary(expression) => {
                if !(2..=64).contains(&expression.args.len()) {
                    return Err(ProgramExprEncodeError::ContractArity {
                        operation: match expression.op {
                            VirContractNaryOperator::And => "and",
                            VirContractNaryOperator::Or => "or",
                        },
                        actual: expression.args.len(),
                    });
                }
                let function = match expression.op {
                    VirContractNaryOperator::And => STD_BOOL_AND,
                    VirContractNaryOperator::Or => STD_BOOL_OR,
                };
                let mut encoded = expression
                    .args
                    .iter()
                    .map(|arg| self.encode_contract_expr_typed(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                for arg in &encoded {
                    require_bool("contract boolean tree", &arg.ty)?;
                }
                let first = encoded.remove(0).term;
                let term = encoded.into_iter().fold(first, |lhs, rhs| {
                    MpkExprTerm::apply(function, [lhs, rhs.term])
                });
                Ok(TypedProgramExpr {
                    term,
                    ty: VirType::Bool {},
                })
            }
            VirContractExpr::Binary(expression) => {
                if self.context.profile == SemanticProfile::RustCheckedV0
                    && matches!(
                        expression.op,
                        VirBinaryOperator::BvSdiv
                            | VirBinaryOperator::BvSrem
                            | VirBinaryOperator::BvUdiv
                            | VirBinaryOperator::BvUrem
                    )
                {
                    return Err(ProgramExprEncodeError::ProfileOperation {
                        profile: self.context.profile,
                        operation: vir_binary_name(expression.op),
                    });
                }
                self.encode_binary(
                    expression.op,
                    self.encode_contract_expr_typed(&expression.lhs)?,
                    self.encode_contract_expr_typed(&expression.rhs)?,
                    None,
                    true,
                )
            }
            VirContractExpr::Convert(expression) => {
                if expression.op != VirContractConvertOperator::Convert
                    || self.context.profile != SemanticProfile::GoFixedV0
                {
                    return Err(ProgramExprEncodeError::ProfileOperation {
                        profile: self.context.profile,
                        operation: "convert",
                    });
                }
                self.encode_convert(
                    self.encode_contract_expr_typed(&expression.value)?,
                    &expression.r#type,
                )
            }
        }
    }

    fn encode_unary(
        &self,
        op: &'static str,
        value: TypedProgramExpr,
        result_type: Option<&VirType>,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        match op {
            "not" => {
                require_bool(op, &value.ty)?;
                if let Some(result_type) = result_type {
                    require_bool(op, result_type)?;
                }
                Ok(TypedProgramExpr {
                    term: MpkExprTerm::apply(STD_BOOL_NOT, [value.term]),
                    ty: VirType::Bool {},
                })
            }
            "bv_neg" | "bv_not" => {
                let (width, signed) = bitvector_shape(op, &value.ty)?;
                if op == "bv_neg"
                    && !signed
                    && self.context.profile == SemanticProfile::RustCheckedV0
                    && result_type.is_some()
                {
                    return Err(ProgramExprEncodeError::ProfileOperation {
                        profile: self.context.profile,
                        operation: "bv_neg",
                    });
                }
                if let Some(result_type) = result_type {
                    require_type(op, &value.ty, result_type)?;
                }
                Ok(TypedProgramExpr {
                    term: MpkExprTerm::apply(
                        bitvec_function(width, if op == "bv_neg" { "neg" } else { "not" }),
                        [value.term],
                    ),
                    ty: value.ty,
                })
            }
            _ => Err(ProgramExprEncodeError::UnsupportedOperation { operation: op }),
        }
    }

    fn encode_binary(
        &self,
        op: VirBinaryOperator,
        lhs: TypedProgramExpr,
        rhs: TypedProgramExpr,
        result_type: Option<&VirType>,
        contract: bool,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        let name = vir_binary_name(op);
        if matches!(op, VirBinaryOperator::Eq | VirBinaryOperator::NotEq) {
            require_type(name, &lhs.ty, &rhs.ty)?;
            if let Some(result_type) = result_type {
                require_bool(name, result_type)?;
                if self.context.profile == SemanticProfile::RustCheckedV0
                    && !contract
                    && matches!(lhs.ty, VirType::Array { .. } | VirType::Struct { .. })
                {
                    return Err(ProgramExprEncodeError::ProfileOperation {
                        profile: self.context.profile,
                        operation: name,
                    });
                }
            }
            validate_aggregate_equality_type(&lhs.ty, &self.context.declarations)?;
            // Std.Program.Base models arrays as ordered fixed carriers and
            // structs as declaration-ID-indexed, declaration-order carriers.
            // Equality on that exact carrier is componentwise while retaining
            // nominal struct identity; inequality is its checked negation.
            let equality = MpkExprTerm::apply(STD_EQ, [lhs.term, rhs.term]);
            return Ok(TypedProgramExpr {
                term: if op == VirBinaryOperator::NotEq {
                    MpkExprTerm::apply(STD_BOOL_NOT, [equality])
                } else {
                    equality
                },
                ty: VirType::Bool {},
            });
        }

        let (lhs_width, lhs_signed) = bitvector_shape(name, &lhs.ty)?;
        let (rhs_width, rhs_signed) = bitvector_shape(name, &rhs.ty)?;
        let result = match op {
            VirBinaryOperator::BvShl | VirBinaryOperator::BvAshr | VirBinaryOperator::BvLshr => {
                if op == VirBinaryOperator::BvAshr && !lhs_signed {
                    return Err(ProgramExprEncodeError::Signedness {
                        operation: name,
                        expected_signed: true,
                    });
                }
                if op == VirBinaryOperator::BvLshr && lhs_signed {
                    return Err(ProgramExprEncodeError::Signedness {
                        operation: name,
                        expected_signed: false,
                    });
                }
                if let Some(result_type) = result_type {
                    require_type(name, &lhs.ty, result_type)?;
                }
                let suffix = match op {
                    VirBinaryOperator::BvShl => "shl",
                    VirBinaryOperator::BvAshr => "ashr",
                    VirBinaryOperator::BvLshr => "lshr",
                    _ => {
                        return Err(ProgramExprEncodeError::UnsupportedOperation {
                            operation: name,
                        });
                    }
                };
                TypedProgramExpr {
                    term: self.encode_full_count_shift(
                        suffix, lhs.term, rhs.term, lhs_width, lhs_signed, rhs_width, rhs_signed,
                    )?,
                    ty: lhs.ty,
                }
            }
            VirBinaryOperator::SignedLt
            | VirBinaryOperator::SignedLe
            | VirBinaryOperator::SignedGt
            | VirBinaryOperator::SignedGe
            | VirBinaryOperator::UnsignedLt
            | VirBinaryOperator::UnsignedLe
            | VirBinaryOperator::UnsignedGt
            | VirBinaryOperator::UnsignedGe => {
                require_type(name, &lhs.ty, &rhs.ty)?;
                let expected_signed = matches!(
                    op,
                    VirBinaryOperator::SignedLt
                        | VirBinaryOperator::SignedLe
                        | VirBinaryOperator::SignedGt
                        | VirBinaryOperator::SignedGe
                );
                if lhs_signed != expected_signed || rhs_signed != expected_signed {
                    return Err(ProgramExprEncodeError::Signedness {
                        operation: name,
                        expected_signed,
                    });
                }
                if let Some(result_type) = result_type {
                    require_bool(name, result_type)?;
                }
                let suffix = match op {
                    VirBinaryOperator::SignedLt => "slt",
                    VirBinaryOperator::SignedLe => "sle",
                    VirBinaryOperator::SignedGt => "sgt",
                    VirBinaryOperator::SignedGe => "sge",
                    VirBinaryOperator::UnsignedLt => "ult",
                    VirBinaryOperator::UnsignedLe => "ule",
                    VirBinaryOperator::UnsignedGt => "ugt",
                    VirBinaryOperator::UnsignedGe => "uge",
                    _ => {
                        return Err(ProgramExprEncodeError::UnsupportedOperation {
                            operation: name,
                        });
                    }
                };
                TypedProgramExpr {
                    term: MpkExprTerm::apply(
                        bitvec_function(lhs_width, suffix),
                        [lhs.term, rhs.term],
                    ),
                    ty: VirType::Bool {},
                }
            }
            _ => {
                require_type(name, &lhs.ty, &rhs.ty)?;
                if let Some(result_type) = result_type {
                    require_type(name, &lhs.ty, result_type)?;
                }
                let (suffix, expected_signed) = match op {
                    VirBinaryOperator::BvAdd => ("add", None),
                    VirBinaryOperator::BvSub => ("sub", None),
                    VirBinaryOperator::BvMul => ("mul", None),
                    VirBinaryOperator::BvSdiv => ("sdiv", Some(true)),
                    VirBinaryOperator::BvSrem => ("srem", Some(true)),
                    VirBinaryOperator::BvUdiv => ("udiv", Some(false)),
                    VirBinaryOperator::BvUrem => ("urem", Some(false)),
                    VirBinaryOperator::BvAnd => ("and", None),
                    VirBinaryOperator::BvOr => ("or", None),
                    VirBinaryOperator::BvXor => ("xor", None),
                    _ => {
                        return Err(ProgramExprEncodeError::UnsupportedOperation {
                            operation: name,
                        });
                    }
                };
                if let Some(expected_signed) = expected_signed {
                    if expected_signed != lhs_signed {
                        return Err(ProgramExprEncodeError::Signedness {
                            operation: name,
                            expected_signed,
                        });
                    }
                }
                TypedProgramExpr {
                    term: MpkExprTerm::apply(
                        bitvec_function(lhs_width, suffix),
                        [lhs.term, rhs.term],
                    ),
                    ty: lhs.ty,
                }
            }
        };
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_full_count_shift(
        &self,
        suffix: &str,
        lhs: MpkExprTerm,
        rhs: MpkExprTerm,
        lhs_width: u32,
        lhs_signed: bool,
        rhs_width: u32,
        rhs_signed: bool,
    ) -> Result<MpkExprTerm, ProgramExprEncodeError> {
        if lhs_width == rhs_width {
            return Ok(MpkExprTerm::apply(
                bitvec_function(lhs_width, suffix),
                [lhs, rhs],
            ));
        }

        // Only the guarded `rhs < lhs_width` branch converts the count. On
        // that branch the mathematical count fits the LHS width, so neither a
        // wider count nor a signed negative bit pattern can be truncated.
        let bound = bitvec_literal(lhs_width.to_string(), rhs_width, rhs_signed);
        let in_range = MpkExprTerm::apply(bitvec_function(rhs_width, "ult"), [rhs.clone(), bound]);
        let target = ProgramTypeEncoder::new(
            self.context.profile,
            &self.context.parameters,
            &self.context.declarations,
        )?
        .encode(&VirType::Bv {
            width: crate::vir::BitVectorWidth::try_from(lhs_width).map_err(|_| {
                ProgramExprEncodeError::UnsupportedBitVectorWidth { width: lhs_width }
            })?,
            signed: lhs_signed,
        })?;
        let narrowed_count = MpkExprTerm::Convert {
            value: Box::new(rhs),
            target,
        };
        let shifted = MpkExprTerm::apply(
            bitvec_function(lhs_width, suffix),
            [lhs.clone(), narrowed_count],
        );
        let zero = bitvec_literal("0", lhs_width, lhs_signed);
        let out_of_range = if suffix == "ashr" {
            let negative =
                MpkExprTerm::apply(bitvec_function(lhs_width, "slt"), [lhs, zero.clone()]);
            MpkExprTerm::apply(
                STD_BOOL_IF,
                [negative, bitvec_literal("-1", lhs_width, true), zero],
            )
        } else {
            zero
        };
        Ok(MpkExprTerm::apply(
            STD_BOOL_IF,
            [in_range, shifted, out_of_range],
        ))
    }

    fn encode_convert(
        &self,
        value: TypedProgramExpr,
        target: &VirType,
    ) -> Result<TypedProgramExpr, ProgramExprEncodeError> {
        bitvector_shape("convert source", &value.ty)?;
        bitvector_shape("convert target", target)?;
        let target_term = ProgramTypeEncoder::new(
            self.context.profile,
            &self.context.parameters,
            &self.context.declarations,
        )?
        .encode(target)?;
        Ok(TypedProgramExpr {
            term: MpkExprTerm::Convert {
                value: Box::new(value.term),
                target: target_term,
            },
            ty: target.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TypedProgramExpr {
    term: MpkExprTerm,
    ty: VirType,
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

fn instruction_type(instruction: &VirInstruction) -> &VirType {
    match instruction {
        VirInstruction::Const { r#type, .. }
        | VirInstruction::Copy { r#type, .. }
        | VirInstruction::BinOp { r#type, .. }
        | VirInstruction::UnaryOp { r#type, .. }
        | VirInstruction::Convert { r#type, .. }
        | VirInstruction::Field { r#type, .. }
        | VirInstruction::Index { r#type, .. }
        | VirInstruction::MakeStruct { r#type, .. }
        | VirInstruction::MakeArray { r#type, .. }
        | VirInstruction::CallStatic { r#type, .. } => r#type,
    }
}

fn vir_unary_name(op: VirUnaryOperator) -> &'static str {
    match op {
        VirUnaryOperator::Not => "not",
        VirUnaryOperator::BvNeg => "bv_neg",
        VirUnaryOperator::BvNot => "bv_not",
    }
}

fn contract_unary_name(op: VirContractUnaryOperator) -> &'static str {
    match op {
        VirContractUnaryOperator::Not => "not",
        VirContractUnaryOperator::BvNeg => "bv_neg",
        VirContractUnaryOperator::BvNot => "bv_not",
    }
}

fn vir_binary_name(op: VirBinaryOperator) -> &'static str {
    match op {
        VirBinaryOperator::Eq => "eq",
        VirBinaryOperator::NotEq => "not_eq",
        VirBinaryOperator::BvAdd => "bv_add",
        VirBinaryOperator::BvSub => "bv_sub",
        VirBinaryOperator::BvMul => "bv_mul",
        VirBinaryOperator::BvSdiv => "bv_sdiv",
        VirBinaryOperator::BvSrem => "bv_srem",
        VirBinaryOperator::BvUdiv => "bv_udiv",
        VirBinaryOperator::BvUrem => "bv_urem",
        VirBinaryOperator::BvAnd => "bv_and",
        VirBinaryOperator::BvOr => "bv_or",
        VirBinaryOperator::BvXor => "bv_xor",
        VirBinaryOperator::BvShl => "bv_shl",
        VirBinaryOperator::BvAshr => "bv_ashr",
        VirBinaryOperator::BvLshr => "bv_lshr",
        VirBinaryOperator::SignedLt => "signed_lt",
        VirBinaryOperator::SignedLe => "signed_le",
        VirBinaryOperator::SignedGt => "signed_gt",
        VirBinaryOperator::SignedGe => "signed_ge",
        VirBinaryOperator::UnsignedLt => "unsigned_lt",
        VirBinaryOperator::UnsignedLe => "unsigned_le",
        VirBinaryOperator::UnsignedGt => "unsigned_gt",
        VirBinaryOperator::UnsignedGe => "unsigned_ge",
    }
}

fn require_type(
    operation: &'static str,
    expected: &VirType,
    actual: &VirType,
) -> Result<(), ProgramExprEncodeError> {
    if expected != actual {
        return Err(ProgramExprEncodeError::TypeMismatch {
            operation,
            expected: expected.clone(),
            actual: actual.clone(),
        });
    }
    Ok(())
}

fn require_bool(operation: &'static str, actual: &VirType) -> Result<(), ProgramExprEncodeError> {
    if !matches!(actual, VirType::Bool {}) {
        return Err(ProgramExprEncodeError::ExpectedBool {
            operation,
            actual: actual.clone(),
        });
    }
    Ok(())
}

fn bitvector_shape(
    operation: &'static str,
    actual: &VirType,
) -> Result<(u32, bool), ProgramExprEncodeError> {
    match actual {
        VirType::Bv { width, signed } => Ok((width.bits(), *signed)),
        _ => Err(ProgramExprEncodeError::ExpectedBitVector {
            operation,
            actual: actual.clone(),
        }),
    }
}

fn validate_aggregate_equality_type(
    ty: &VirType,
    declarations: &[VirStructDecl],
) -> Result<(), ProgramExprEncodeError> {
    match ty {
        VirType::Struct { id } if !declarations.iter().any(|declaration| declaration.id == *id) => {
            Err(ProgramExprEncodeError::UnknownStruct { id: id.clone() })
        }
        VirType::Array { element, .. } => validate_aggregate_equality_type(element, declarations),
        _ => Ok(()),
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

fn bitvec_literal(value: impl Into<String>, width: u32, signed: bool) -> MpkExprTerm {
    MpkExprTerm::BitVecLiteral {
        value: value.into(),
        width,
        signed,
    }
}

fn validate_bit_pattern(
    operand: &'static str,
    value: u64,
    width: u32,
) -> Result<(), ProgramExprEncodeError> {
    if value > bit_mask(width) {
        return Err(ProgramExprEncodeError::BitPatternOutOfRange {
            operand,
            value,
            width,
        });
    }
    Ok(())
}

fn bit_mask(width: u32) -> u64 {
    if width == 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    }
}

fn signed_value(value: u64, width: u32) -> i128 {
    let sign_bit = 1_u64 << (width - 1);
    if value & sign_bit == 0 {
        i128::from(value)
    } else {
        i128::from(value) - (1_i128 << width)
    }
}

fn wrap_signed(value: i128, width: u32) -> u64 {
    let modulus = 1_i128 << width;
    let normalized = value.rem_euclid(modulus);
    normalized as u64
}

fn absolute_bit_pattern(value: u64, width: u32) -> u64 {
    let signed = signed_value(value, width);
    if signed < 0 {
        wrap_signed(-signed, width)
    } else {
        value
    }
}

fn signed_division(lhs: u64, rhs: u64, width: u32) -> u64 {
    let lhs_negative = signed_value(lhs, width) < 0;
    let rhs_negative = signed_value(rhs, width) < 0;
    let lhs_absolute = absolute_bit_pattern(lhs, width);
    let rhs_absolute = absolute_bit_pattern(rhs, width);
    let quotient = lhs_absolute
        .checked_div(rhs_absolute)
        .unwrap_or_else(|| bit_mask(width));
    if lhs_negative != rhs_negative {
        wrap_signed(-i128::from(quotient), width)
    } else {
        quotient
    }
}

fn signed_remainder(lhs: u64, rhs: u64, width: u32) -> u64 {
    let lhs_negative = signed_value(lhs, width) < 0;
    let lhs_absolute = absolute_bit_pattern(lhs, width);
    let rhs_absolute = absolute_bit_pattern(rhs, width);
    let remainder = lhs_absolute
        .checked_rem(rhs_absolute)
        .unwrap_or(lhs_absolute);
    if lhs_negative {
        wrap_signed(-i128::from(remainder), width)
    } else {
        remainder
    }
}

fn validate_int_literal(input: &VirIntLiteral) -> Result<(), ProgramExprEncodeError> {
    let value = input.value.as_str().parse::<i128>().map_err(|_| {
        ProgramExprEncodeError::InvalidIntegerLiteral {
            value: input.value.as_str().to_owned(),
        }
    })?;
    let width = input.width.bits();
    let in_range = if input.signed {
        let min = -(1_i128 << (width - 1));
        let max = (1_i128 << (width - 1)) - 1;
        (min..=max).contains(&value)
    } else {
        value >= 0 && (value as u128) <= ((1_u128 << width) - 1)
    };
    if !in_range {
        return Err(ProgramExprEncodeError::InvalidIntegerLiteral {
            value: input.value.as_str().to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramExprEncodeError {
    SemanticProfile(SemanticProfileError),
    Validation(VirValidationError),
    Type(TypeEncodeError),
    UnitNotInModule {
        unit_id: String,
    },
    FunctionNotInUnit {
        function_id: String,
        unit_id: String,
    },
    ResultIndexOverflow {
        function_id: String,
    },
    UnknownVariable {
        id: String,
    },
    UnknownResult {
        index: u32,
    },
    UnknownConstant {
        id: String,
    },
    UnknownStruct {
        id: String,
    },
    TypeMismatch {
        operation: &'static str,
        expected: VirType,
        actual: VirType,
    },
    ExpectedBool {
        operation: &'static str,
        actual: VirType,
    },
    ExpectedBitVector {
        operation: &'static str,
        actual: VirType,
    },
    Signedness {
        operation: &'static str,
        expected_signed: bool,
    },
    ContractArity {
        operation: &'static str,
        actual: usize,
    },
    UnsupportedInstruction {
        kind: VirInstructionKind,
    },
    UnsupportedOperation {
        operation: &'static str,
    },
    UnsupportedBitVectorWidth {
        width: u32,
    },
    ProfileOperation {
        profile: SemanticProfile,
        operation: &'static str,
    },
    InvalidIntegerLiteral {
        value: String,
    },
    BitPatternOutOfRange {
        operand: &'static str,
        value: u64,
        width: u32,
    },
    BitVectorWidthMismatch {
        operation: &'static str,
        lhs_width: u32,
        rhs_width: u32,
    },
}

impl From<TypeEncodeError> for ProgramExprEncodeError {
    fn from(error: TypeEncodeError) -> Self {
        Self::Type(error)
    }
}

impl fmt::Display for ProgramExprEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SemanticProfile(error) => write!(formatter, "invalid semantic profile: {error}"),
            Self::Validation(error) => write!(formatter, "invalid VIR module: {error}"),
            Self::Type(error) => write!(formatter, "type encoding failed: {error}"),
            Self::UnitNotInModule { unit_id } => {
                write!(formatter, "VIR unit {unit_id:?} is not in the module")
            }
            Self::FunctionNotInUnit {
                function_id,
                unit_id,
            } => write!(
                formatter,
                "VIR function {function_id:?} is not in unit {unit_id:?}"
            ),
            Self::ResultIndexOverflow { function_id } => {
                write!(
                    formatter,
                    "VIR function {function_id:?} has too many results"
                )
            }
            Self::UnknownVariable { id } => write!(formatter, "unknown VIR variable {id:?}"),
            Self::UnknownResult { index } => write!(formatter, "unknown VIR result {index}"),
            Self::UnknownConstant { id } => write!(formatter, "unknown VIR constant {id:?}"),
            Self::UnknownStruct { id } => write!(formatter, "unknown VIR struct {id:?}"),
            Self::TypeMismatch {
                operation,
                expected,
                actual,
            } => write!(
                formatter,
                "VIR operation {operation:?} expected {expected:?}, found {actual:?}"
            ),
            Self::ExpectedBool { operation, actual } => write!(
                formatter,
                "VIR operation {operation:?} expected bool, found {actual:?}"
            ),
            Self::ExpectedBitVector { operation, actual } => write!(
                formatter,
                "VIR operation {operation:?} expected a bitvector, found {actual:?}"
            ),
            Self::Signedness {
                operation,
                expected_signed,
            } => write!(
                formatter,
                "VIR operation {operation:?} requires signed={expected_signed} operands"
            ),
            Self::ContractArity { operation, actual } => write!(
                formatter,
                "VIR contract operation {operation:?} needs 2 through 64 operands, found {actual}"
            ),
            Self::UnsupportedInstruction { kind } => {
                write!(
                    formatter,
                    "VIR instruction {kind:?} is not a scalar value expression"
                )
            }
            Self::UnsupportedOperation { operation } => {
                write!(formatter, "VIR operation {operation:?} is not supported")
            }
            Self::UnsupportedBitVectorWidth { width } => {
                write!(formatter, "VIR bitvector width {width} is not supported")
            }
            Self::ProfileOperation { profile, operation } => write!(
                formatter,
                "VIR operation {operation:?} is not admitted by profile {profile:?}"
            ),
            Self::InvalidIntegerLiteral { value } => {
                write!(
                    formatter,
                    "VIR integer literal {value:?} does not fit its type"
                )
            }
            Self::BitPatternOutOfRange {
                operand,
                value,
                width,
            } => write!(
                formatter,
                "VIR {operand} bit pattern {value} does not fit BV{width}"
            ),
            Self::BitVectorWidthMismatch {
                operation,
                lhs_width,
                rhs_width,
            } => write!(
                formatter,
                "VIR operation {operation:?} requires equal widths, found BV{lhs_width} and BV{rhs_width}"
            ),
        }
    }
}

impl std::error::Error for ProgramExprEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SemanticProfile(error) => Some(error),
            Self::Validation(error) => Some(error),
            Self::Type(error) => Some(error),
            _ => None,
        }
    }
}
