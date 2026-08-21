//! GIR expression to unresolved MPK expression-term encoding.
//!
//! The encoder is intentionally name-based. It resolves only stable exported
//! names such as `Std.BitVec.BV64.add`; certificate global ids are emitted by
//! later VC milestones.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::gir::{
    GirBinding, GirContractExpr, GirFunction, GirInstruction, GirInstructionKind, GirIntLiteral,
    GirType, GirTypeKind, GirValue,
};
use crate::type_encode::{encode_gir_type, MpkTypeTerm, TypeEncodeError};

pub const STD_BOOL_TRUE: &str = "Std.Bool.true";
pub const STD_BOOL_FALSE: &str = "Std.Bool.false";
pub const STD_BOOL_NOT: &str = "Std.Bool.not";
pub const STD_BOOL_AND: &str = "Std.Bool.and";
pub const STD_BOOL_OR: &str = "Std.Bool.or";
pub const STD_BOOL_IF: &str = "Std.Bool.if";
pub const STD_EQ: &str = "Std.Eq";
pub const STD_BITVEC_MODULE: &str = "Std.BitVec";

pub fn encode_gir_value(input: &GirValue) -> Result<MpkExprTerm, ExprEncodeError> {
    ExprEncoder::new().encode_value(input)
}

pub fn encode_contract_expr(input: &GirContractExpr) -> Result<MpkExprTerm, ExprEncodeError> {
    ExprEncoder::new().encode_contract_expr(input)
}

pub fn encode_instruction_expr(input: &GirInstruction) -> Result<MpkExprTerm, ExprEncodeError> {
    ExprEncoder::new().encode_instruction(input)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExprContext {
    variables: BTreeMap<String, GirType>,
    results: BTreeMap<u32, GirType>,
}

impl ExprContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_function(function: &GirFunction) -> Self {
        let mut context = Self::new();
        for binding in function.params.iter().chain(function.locals.iter()).chain(
            function
                .blocks
                .iter()
                .flat_map(|block| block.parameters.iter()),
        ) {
            context.insert_binding(binding);
        }
        for (index, binding) in function.results.iter().enumerate() {
            if let Ok(index) = u32::try_from(index) {
                context.results.insert(index, binding.r#type.clone());
            }
        }
        for instruction in function
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
        {
            if !instruction.id.is_empty() {
                context
                    .variables
                    .insert(instruction.id.clone(), instruction.r#type.clone());
            }
        }
        context
    }

    pub fn with_variable(mut self, name: impl Into<String>, r#type: GirType) -> Self {
        self.variables.insert(name.into(), r#type);
        self
    }

    pub fn with_result(mut self, index: u32, r#type: GirType) -> Self {
        self.results.insert(index, r#type);
        self
    }

    pub fn variable_type(&self, name: &str) -> Option<&GirType> {
        self.variables.get(name)
    }

    pub fn result_type(&self, index: u32) -> Option<&GirType> {
        self.results.get(&index)
    }

    fn insert_binding(&mut self, binding: &GirBinding) {
        if !binding.name.is_empty() {
            self.variables
                .insert(binding.name.clone(), binding.r#type.clone());
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExprEncoder {
    context: ExprContext,
}

impl ExprEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_context(context: ExprContext) -> Self {
        Self { context }
    }

    pub fn for_function(function: &GirFunction) -> Self {
        Self::with_context(ExprContext::for_function(function))
    }

    pub fn context(&self) -> &ExprContext {
        &self.context
    }

    pub fn encode_value(&self, input: &GirValue) -> Result<MpkExprTerm, ExprEncodeError> {
        self.encode_value_typed(input).map(|encoded| encoded.term)
    }

    pub fn encode_contract_expr(
        &self,
        input: &GirContractExpr,
    ) -> Result<MpkExprTerm, ExprEncodeError> {
        self.encode_contract_expr_typed(input)
            .map(|encoded| encoded.term)
    }

    pub fn encode_instruction(
        &self,
        input: &GirInstruction,
    ) -> Result<MpkExprTerm, ExprEncodeError> {
        self.encode_instruction_typed(input)
            .map(|encoded| encoded.term)
    }

    fn encode_value_typed(&self, input: &GirValue) -> Result<TypedExpr, ExprEncodeError> {
        let atom_count = usize::from(input.var.is_some())
            + usize::from(input.int.is_some())
            + usize::from(input.bool.is_some());
        if atom_count != 1 {
            return Err(ExprEncodeError::GirValueAtomCount { atom_count });
        }

        if let Some(name) = &input.var {
            return self.encode_variable(name);
        }
        if let Some(literal) = &input.int {
            return encode_int_literal(literal);
        }
        Ok(TypedExpr {
            term: MpkExprTerm::bool_literal(
                input.bool.expect("atom_count guarantees bool is present"),
            ),
            ty: Some(MpkExprType::Bool),
        })
    }

    fn encode_contract_expr_typed(
        &self,
        input: &GirContractExpr,
    ) -> Result<TypedExpr, ExprEncodeError> {
        if let Some(op) = &input.op {
            if has_contract_atom(input) {
                return Err(ExprEncodeError::OperatorContainsAtom { op: op.clone() });
            }
            return self.encode_contract_operator(input, op);
        }

        let atom_count = usize::from(input.var.is_some())
            + usize::from(input.result.is_some())
            + usize::from(input.int.is_some())
            + usize::from(input.bool.is_some());
        if atom_count != 1 {
            return Err(ExprEncodeError::ContractAtomCount { atom_count });
        }
        if input.value.is_some()
            || input.lhs.is_some()
            || input.rhs.is_some()
            || !input.args.is_empty()
            || input.r#type.is_some()
        {
            return Err(ExprEncodeError::AtomContainsOperatorFields);
        }

        if let Some(name) = &input.var {
            return self.encode_variable(name);
        }
        if let Some(index) = input.result {
            return self.encode_result(index);
        }
        if let Some(literal) = &input.int {
            return encode_int_literal(literal);
        }
        Ok(TypedExpr {
            term: MpkExprTerm::bool_literal(
                input.bool.expect("atom_count guarantees bool is present"),
            ),
            ty: Some(MpkExprType::Bool),
        })
    }

    fn encode_instruction_typed(
        &self,
        input: &GirInstruction,
    ) -> Result<TypedExpr, ExprEncodeError> {
        match input.kind {
            GirInstructionKind::Const => {
                let value = required_instruction_value(input)?;
                self.encode_value_typed(value)
            }
            GirInstructionKind::Copy => {
                let value = required_instruction_value(input)?;
                self.encode_value_typed(value)
            }
            GirInstructionKind::BinOp => {
                let op = required_instruction_op(input)?;
                let lhs = self.encode_value_typed(required_instruction_lhs(input)?)?;
                let rhs = self.encode_value_typed(required_instruction_rhs(input)?)?;
                let result_type = expr_type_from_gir_type(&input.r#type)?;
                encode_binary_or_comparison(op, lhs, rhs, Some(result_type))
            }
            GirInstructionKind::UnaryOp => {
                let op = required_instruction_op(input)?;
                let value = self.encode_value_typed(required_instruction_value(input)?)?;
                let result_type = expr_type_from_gir_type(&input.r#type)?;
                encode_unary(op, value, Some(result_type))
            }
            GirInstructionKind::Convert => {
                let value = self.encode_value_typed(required_instruction_value(input)?)?;
                let target = encode_target_type(&input.r#type)?;
                let target_ty = expr_type_from_gir_type(&input.r#type)?;
                Ok(TypedExpr {
                    term: MpkExprTerm::Convert {
                        value: Box::new(value.term),
                        target,
                    },
                    ty: Some(target_ty),
                })
            }
            kind => Err(ExprEncodeError::UnsupportedInstructionKind {
                instruction_id: input.id.clone(),
                kind,
            }),
        }
    }

    fn encode_variable(&self, name: &str) -> Result<TypedExpr, ExprEncodeError> {
        if name.is_empty() {
            return Err(ExprEncodeError::EmptyVariableName);
        }
        let ty = self
            .context
            .variable_type(name)
            .map(expr_type_from_gir_type)
            .transpose()?;
        Ok(TypedExpr {
            term: MpkExprTerm::Var {
                name: name.to_owned(),
            },
            ty,
        })
    }

    fn encode_result(&self, index: u32) -> Result<TypedExpr, ExprEncodeError> {
        let ty = self
            .context
            .result_type(index)
            .map(expr_type_from_gir_type)
            .transpose()?;
        Ok(TypedExpr {
            term: MpkExprTerm::Result { index },
            ty,
        })
    }

    fn encode_contract_operator(
        &self,
        input: &GirContractExpr,
        op: &str,
    ) -> Result<TypedExpr, ExprEncodeError> {
        match op {
            "not" | "bv_neg" | "bv_not" => {
                let value = self.encode_unary_contract_value(input, op, false)?;
                encode_unary(op, value, None)
            }
            "and" | "or" => {
                if input.value.is_some()
                    || input.lhs.is_some()
                    || input.rhs.is_some()
                    || input.r#type.is_some()
                {
                    return Err(ExprEncodeError::OperatorShape {
                        op: op.to_owned(),
                        expected: "args",
                    });
                }
                if input.args.len() < 2 {
                    return Err(ExprEncodeError::OperatorArity {
                        op: op.to_owned(),
                        expected: "at least 2 args",
                        actual: input.args.len(),
                    });
                }
                let args = input
                    .args
                    .iter()
                    .map(|arg| self.encode_contract_expr_typed(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                encode_variadic(op, args)
            }
            "convert" => {
                let value = self.encode_unary_contract_value(input, op, true)?;
                let target = input
                    .r#type
                    .as_ref()
                    .ok_or_else(|| ExprEncodeError::MissingConvertType { op: op.to_owned() })?;
                let target_ty = expr_type_from_gir_type(target)?;
                Ok(TypedExpr {
                    term: MpkExprTerm::Convert {
                        value: Box::new(value.term),
                        target: encode_target_type(target)?,
                    },
                    ty: Some(target_ty),
                })
            }
            _ => {
                if !is_binary_or_comparison_operator(op) {
                    return Err(ExprEncodeError::UnsupportedOperator { op: op.to_owned() });
                }
                let (lhs, rhs) = self.encode_binary_contract_values(input, op)?;
                encode_binary_or_comparison(op, lhs, rhs, None)
            }
        }
    }

    fn encode_unary_contract_value(
        &self,
        input: &GirContractExpr,
        op: &str,
        allow_type: bool,
    ) -> Result<TypedExpr, ExprEncodeError> {
        if input.lhs.is_some() || input.rhs.is_some() {
            return Err(ExprEncodeError::OperatorShape {
                op: op.to_owned(),
                expected: "one value",
            });
        }
        if input.r#type.is_some() && !allow_type {
            return Err(ExprEncodeError::OperatorShape {
                op: op.to_owned(),
                expected: "one value without type",
            });
        }
        if let Some(value) = &input.value {
            if !input.args.is_empty() {
                return Err(ExprEncodeError::OperatorShape {
                    op: op.to_owned(),
                    expected: "value or one arg",
                });
            }
            return self.encode_contract_expr_typed(value);
        }
        if input.args.len() != 1 {
            return Err(ExprEncodeError::OperatorArity {
                op: op.to_owned(),
                expected: "1 arg",
                actual: input.args.len(),
            });
        }
        self.encode_contract_expr_typed(&input.args[0])
    }

    fn encode_binary_contract_values(
        &self,
        input: &GirContractExpr,
        op: &str,
    ) -> Result<(TypedExpr, TypedExpr), ExprEncodeError> {
        if input.value.is_some() || input.r#type.is_some() {
            return Err(ExprEncodeError::OperatorShape {
                op: op.to_owned(),
                expected: "lhs/rhs or two args",
            });
        }
        if input.lhs.is_some() || input.rhs.is_some() {
            if input.lhs.is_none() || input.rhs.is_none() || !input.args.is_empty() {
                return Err(ExprEncodeError::OperatorShape {
                    op: op.to_owned(),
                    expected: "both lhs and rhs",
                });
            }
            return Ok((
                self.encode_contract_expr_typed(input.lhs.as_ref().expect("lhs checked"))?,
                self.encode_contract_expr_typed(input.rhs.as_ref().expect("rhs checked"))?,
            ));
        }
        if input.args.len() != 2 {
            return Err(ExprEncodeError::OperatorArity {
                op: op.to_owned(),
                expected: "2 args",
                actual: input.args.len(),
            });
        }
        Ok((
            self.encode_contract_expr_typed(&input.args[0])?,
            self.encode_contract_expr_typed(&input.args[1])?,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TypedExpr {
    term: MpkExprTerm,
    ty: Option<MpkExprType>,
}

fn has_contract_atom(input: &GirContractExpr) -> bool {
    input.var.is_some() || input.result.is_some() || input.int.is_some() || input.bool.is_some()
}

fn encode_int_literal(input: &GirIntLiteral) -> Result<TypedExpr, ExprEncodeError> {
    validate_int_literal(input)?;
    Ok(TypedExpr {
        term: MpkExprTerm::BitVecLiteral {
            value: input.value.clone(),
            width: input.width,
            signed: input.signed,
        },
        ty: Some(MpkExprType::BitVector {
            width: input.width,
            signed: input.signed,
        }),
    })
}

fn validate_int_literal(input: &GirIntLiteral) -> Result<(), ExprEncodeError> {
    if !matches!(input.width, 8 | 16 | 32 | 64) {
        return Err(ExprEncodeError::UnsupportedIntLiteralWidth { width: input.width });
    }
    let trimmed = input.value.trim();
    if trimmed.is_empty() {
        return Err(ExprEncodeError::InvalidIntLiteral {
            value: input.value.clone(),
            reason: "empty literal",
        });
    }
    if trimmed != input.value {
        return Err(ExprEncodeError::InvalidIntLiteral {
            value: input.value.clone(),
            reason: "literal contains leading or trailing whitespace",
        });
    }
    let parsed =
        parse_int_literal(&input.value).ok_or_else(|| ExprEncodeError::InvalidIntLiteral {
            value: input.value.clone(),
            reason: "cannot parse literal",
        })?;

    if input.signed {
        let min = -(1_i128 << (input.width - 1));
        let max = (1_i128 << (input.width - 1)) - 1;
        let value = parsed
            .to_i128()
            .ok_or_else(|| ExprEncodeError::InvalidIntLiteral {
                value: input.value.clone(),
                reason: "literal is too large for signed range",
            })?;
        if value < min || value > max {
            return Err(ExprEncodeError::InvalidIntLiteral {
                value: input.value.clone(),
                reason: "literal does not fit signed width",
            });
        }
        return Ok(());
    }

    if parsed.negative {
        return Err(ExprEncodeError::InvalidIntLiteral {
            value: input.value.clone(),
            reason: "unsigned literal is negative",
        });
    }
    let max = (1_u128 << input.width) - 1;
    if parsed.magnitude > max {
        return Err(ExprEncodeError::InvalidIntLiteral {
            value: input.value.clone(),
            reason: "literal does not fit unsigned width",
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParsedIntLiteral {
    negative: bool,
    magnitude: u128,
}

impl ParsedIntLiteral {
    fn to_i128(self) -> Option<i128> {
        if self.negative {
            let min_magnitude = 1_u128 << 127;
            if self.magnitude > min_magnitude {
                return None;
            }
            if self.magnitude == min_magnitude {
                return Some(i128::MIN);
            }
            Some(-(self.magnitude as i128))
        } else {
            i128::try_from(self.magnitude).ok()
        }
    }
}

fn parse_int_literal(input: &str) -> Option<ParsedIntLiteral> {
    let trimmed = input.trim();
    let (negative, digits) = match trimmed.as_bytes().first().copied() {
        Some(b'-') => (true, &trimmed[1..]),
        Some(b'+') => (false, &trimmed[1..]),
        _ => (false, trimmed),
    };
    if digits.is_empty() {
        return None;
    }

    let (radix, digits) = if let Some(rest) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        (16, rest)
    } else if let Some(rest) = digits
        .strip_prefix("0b")
        .or_else(|| digits.strip_prefix("0B"))
    {
        (2, rest)
    } else if let Some(rest) = digits
        .strip_prefix("0o")
        .or_else(|| digits.strip_prefix("0O"))
    {
        (8, rest)
    } else {
        (10, digits)
    };
    if digits.is_empty() {
        return None;
    }
    let magnitude = u128::from_str_radix(digits, radix).ok()?;
    Some(ParsedIntLiteral {
        negative,
        magnitude,
    })
}

fn encode_target_type(input: &GirType) -> Result<MpkTypeTerm, ExprEncodeError> {
    encode_gir_type(input).map_err(ExprEncodeError::Type)
}

fn expr_type_from_gir_type(input: &GirType) -> Result<MpkExprType, ExprEncodeError> {
    encode_gir_type(input).map_err(ExprEncodeError::Type)?;
    match input.kind {
        GirTypeKind::Bool => Ok(MpkExprType::Bool),
        GirTypeKind::BitVector => Ok(MpkExprType::BitVector {
            width: input.width.expect("type encoder validated width"),
            signed: input.signed.expect("type encoder validated signed"),
        }),
        GirTypeKind::Array | GirTypeKind::Struct => Ok(MpkExprType::Other),
    }
}

fn required_instruction_op(input: &GirInstruction) -> Result<&str, ExprEncodeError> {
    input
        .op
        .as_deref()
        .ok_or_else(|| ExprEncodeError::MissingInstructionOp {
            instruction_id: input.id.clone(),
        })
}

fn required_instruction_value(input: &GirInstruction) -> Result<&GirValue, ExprEncodeError> {
    input
        .value
        .as_ref()
        .ok_or_else(|| ExprEncodeError::MissingInstructionValue {
            instruction_id: input.id.clone(),
        })
}

fn required_instruction_lhs(input: &GirInstruction) -> Result<&GirValue, ExprEncodeError> {
    input
        .lhs
        .as_ref()
        .ok_or_else(|| ExprEncodeError::MissingInstructionOperand {
            instruction_id: input.id.clone(),
            operand: "lhs",
        })
}

fn required_instruction_rhs(input: &GirInstruction) -> Result<&GirValue, ExprEncodeError> {
    input
        .rhs
        .as_ref()
        .ok_or_else(|| ExprEncodeError::MissingInstructionOperand {
            instruction_id: input.id.clone(),
            operand: "rhs",
        })
}

fn encode_unary(
    op: &str,
    value: TypedExpr,
    result_type: Option<MpkExprType>,
) -> Result<TypedExpr, ExprEncodeError> {
    match op {
        "not" => {
            require_bool(op, "value", value.ty)?;
            if let Some(result_type) = result_type {
                require_bool_type(op, "result", result_type)?;
            }
            Ok(TypedExpr {
                term: MpkExprTerm::apply(STD_BOOL_NOT, [value.term]),
                ty: Some(MpkExprType::Bool),
            })
        }
        "bv_neg" => encode_unary_bitvec(op, "neg", value, result_type),
        "bv_not" => encode_unary_bitvec(op, "not", value, result_type),
        _ => Err(ExprEncodeError::UnsupportedOperator { op: op.to_owned() }),
    }
}

fn encode_unary_bitvec(
    op: &str,
    suffix: &str,
    value: TypedExpr,
    result_type: Option<MpkExprType>,
) -> Result<TypedExpr, ExprEncodeError> {
    let bitvec = require_bitvector(op, "value", value.ty)?;
    if let Some(result_type) = result_type {
        let result_bv = require_bitvector_type(op, "result", result_type)?;
        require_same_bitvector_width(op, bitvec, result_bv)?;
    }
    Ok(TypedExpr {
        term: MpkExprTerm::apply(bitvec_function(bitvec.width, suffix), [value.term]),
        ty: Some(MpkExprType::BitVector {
            width: bitvec.width,
            signed: bitvec.signed,
        }),
    })
}

fn encode_variadic(op: &str, args: Vec<TypedExpr>) -> Result<TypedExpr, ExprEncodeError> {
    let function = match op {
        "and" => STD_BOOL_AND,
        "or" => STD_BOOL_OR,
        _ => return Err(ExprEncodeError::UnsupportedOperator { op: op.to_owned() }),
    };
    for arg in &args {
        require_bool(op, "arg", arg.ty)?;
    }
    let mut args = args.into_iter();
    let first = args.next().expect("caller guarantees at least two args");
    let term = args.fold(first.term, |acc, arg| {
        MpkExprTerm::apply(function, [acc, arg.term])
    });
    Ok(TypedExpr {
        term,
        ty: Some(MpkExprType::Bool),
    })
}

fn encode_binary_or_comparison(
    op: &str,
    lhs: TypedExpr,
    rhs: TypedExpr,
    result_type: Option<MpkExprType>,
) -> Result<TypedExpr, ExprEncodeError> {
    match op {
        "eq" => {
            require_matching_known_types(op, lhs.ty, rhs.ty)?;
            if let Some(result_type) = result_type {
                require_bool_type(op, "result", result_type)?;
            }
            Ok(TypedExpr {
                term: MpkExprTerm::apply(STD_EQ, [lhs.term, rhs.term]),
                ty: Some(MpkExprType::Bool),
            })
        }
        "not_eq" => {
            require_matching_known_types(op, lhs.ty, rhs.ty)?;
            if let Some(result_type) = result_type {
                require_bool_type(op, "result", result_type)?;
            }
            let equality = MpkExprTerm::apply(STD_EQ, [lhs.term, rhs.term]);
            Ok(TypedExpr {
                term: MpkExprTerm::apply(STD_BOOL_NOT, [equality]),
                ty: Some(MpkExprType::Bool),
            })
        }
        _ => {
            let Some(class) = bitvec_operator_class(op) else {
                return Err(ExprEncodeError::UnsupportedOperator { op: op.to_owned() });
            };
            encode_bitvec_operator(op, class, lhs, rhs, result_type)
        }
    }
}

fn encode_bitvec_operator(
    op: &str,
    class: BitVecOperatorClass,
    lhs: TypedExpr,
    rhs: TypedExpr,
    result_type: Option<MpkExprType>,
) -> Result<TypedExpr, ExprEncodeError> {
    let lhs_bv = require_bitvector(op, "lhs", lhs.ty)?;
    let rhs_bv = require_bitvector(op, "rhs", rhs.ty)?;
    match class {
        BitVecOperatorClass::Arithmetic { suffix } => {
            require_same_bitvector_width(op, lhs_bv, rhs_bv)?;
            if let Some(result_type) = result_type {
                let result_bv = require_bitvector_type(op, "result", result_type)?;
                require_same_bitvector_width(op, lhs_bv, result_bv)?;
            }
            Ok(TypedExpr {
                term: MpkExprTerm::apply(
                    bitvec_function(lhs_bv.width, suffix),
                    [lhs.term, rhs.term],
                ),
                ty: Some(MpkExprType::BitVector {
                    width: lhs_bv.width,
                    signed: lhs_bv.signed,
                }),
            })
        }
        BitVecOperatorClass::Shift { suffix } => {
            if let Some(result_type) = result_type {
                let result_bv = require_bitvector_type(op, "result", result_type)?;
                require_same_bitvector_width(op, lhs_bv, result_bv)?;
            }
            Ok(TypedExpr {
                term: MpkExprTerm::apply(
                    bitvec_function(lhs_bv.width, suffix),
                    [lhs.term, rhs.term],
                ),
                ty: Some(MpkExprType::BitVector {
                    width: lhs_bv.width,
                    signed: lhs_bv.signed,
                }),
            })
        }
        BitVecOperatorClass::Comparison { suffix, signed } => {
            require_same_bitvector_width(op, lhs_bv, rhs_bv)?;
            if lhs_bv.signed != signed || rhs_bv.signed != signed {
                return Err(ExprEncodeError::SignednessMismatch {
                    op: op.to_owned(),
                    expected_signed: signed,
                    lhs: lhs_bv.signed,
                    rhs: rhs_bv.signed,
                });
            }
            if let Some(result_type) = result_type {
                require_bool_type(op, "result", result_type)?;
            }
            Ok(TypedExpr {
                term: MpkExprTerm::apply(
                    bitvec_function(lhs_bv.width, suffix),
                    [lhs.term, rhs.term],
                ),
                ty: Some(MpkExprType::Bool),
            })
        }
    }
}

fn require_bool(
    op: &str,
    operand: &'static str,
    ty: Option<MpkExprType>,
) -> Result<(), ExprEncodeError> {
    match ty {
        Some(MpkExprType::Bool) => Ok(()),
        Some(actual) => Err(ExprEncodeError::ExpectedBool {
            op: op.to_owned(),
            operand,
            actual,
        }),
        None => Err(ExprEncodeError::MissingExpressionType {
            op: op.to_owned(),
            operand,
        }),
    }
}

fn require_bool_type(
    op: &str,
    operand: &'static str,
    ty: MpkExprType,
) -> Result<(), ExprEncodeError> {
    require_bool(op, operand, Some(ty))
}

fn require_bitvector(
    op: &str,
    operand: &'static str,
    ty: Option<MpkExprType>,
) -> Result<BitVectorType, ExprEncodeError> {
    match ty {
        Some(ty) => require_bitvector_type(op, operand, ty),
        None => Err(ExprEncodeError::MissingExpressionType {
            op: op.to_owned(),
            operand,
        }),
    }
}

fn require_bitvector_type(
    op: &str,
    operand: &'static str,
    ty: MpkExprType,
) -> Result<BitVectorType, ExprEncodeError> {
    match ty {
        MpkExprType::BitVector { width, signed } => Ok(BitVectorType { width, signed }),
        actual => Err(ExprEncodeError::ExpectedBitVector {
            op: op.to_owned(),
            operand,
            actual,
        }),
    }
}

fn require_matching_known_types(
    op: &str,
    lhs: Option<MpkExprType>,
    rhs: Option<MpkExprType>,
) -> Result<(), ExprEncodeError> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) if lhs != rhs => Err(ExprEncodeError::TypeMismatch {
            op: op.to_owned(),
            lhs,
            rhs,
        }),
        _ => Ok(()),
    }
}

fn require_same_bitvector_width(
    op: &str,
    lhs: BitVectorType,
    rhs: BitVectorType,
) -> Result<(), ExprEncodeError> {
    if lhs.width != rhs.width {
        return Err(ExprEncodeError::BitVectorWidthMismatch {
            op: op.to_owned(),
            lhs_width: lhs.width,
            rhs_width: rhs.width,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BitVectorType {
    width: u32,
    signed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BitVecOperatorClass {
    Arithmetic { suffix: &'static str },
    Shift { suffix: &'static str },
    Comparison { suffix: &'static str, signed: bool },
}

fn is_binary_or_comparison_operator(op: &str) -> bool {
    op == "eq" || op == "not_eq" || bitvec_operator_class(op).is_some()
}

fn bitvec_operator_class(op: &str) -> Option<BitVecOperatorClass> {
    match op {
        "bv_add" => Some(BitVecOperatorClass::Arithmetic { suffix: "add" }),
        "bv_sub" => Some(BitVecOperatorClass::Arithmetic { suffix: "sub" }),
        "bv_mul" => Some(BitVecOperatorClass::Arithmetic { suffix: "mul" }),
        "bv_and" => Some(BitVecOperatorClass::Arithmetic { suffix: "and" }),
        "bv_or" => Some(BitVecOperatorClass::Arithmetic { suffix: "or" }),
        "bv_xor" => Some(BitVecOperatorClass::Arithmetic { suffix: "xor" }),
        "bv_shl" => Some(BitVecOperatorClass::Shift { suffix: "shl" }),
        "bv_ashr" => Some(BitVecOperatorClass::Shift { suffix: "ashr" }),
        "bv_lshr" => Some(BitVecOperatorClass::Shift { suffix: "lshr" }),
        "signed_lt" => Some(BitVecOperatorClass::Comparison {
            suffix: "slt",
            signed: true,
        }),
        "signed_le" => Some(BitVecOperatorClass::Comparison {
            suffix: "sle",
            signed: true,
        }),
        "signed_gt" => Some(BitVecOperatorClass::Comparison {
            suffix: "sgt",
            signed: true,
        }),
        "signed_ge" => Some(BitVecOperatorClass::Comparison {
            suffix: "sge",
            signed: true,
        }),
        "unsigned_lt" => Some(BitVecOperatorClass::Comparison {
            suffix: "ult",
            signed: false,
        }),
        "unsigned_le" => Some(BitVecOperatorClass::Comparison {
            suffix: "ule",
            signed: false,
        }),
        "unsigned_gt" => Some(BitVecOperatorClass::Comparison {
            suffix: "ugt",
            signed: false,
        }),
        "unsigned_ge" => Some(BitVecOperatorClass::Comparison {
            suffix: "uge",
            signed: false,
        }),
        _ => None,
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MpkExprType {
    Bool,
    BitVector { width: u32, signed: bool },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MpkExprTerm {
    Var {
        name: String,
    },
    Bound {
        index: u32,
    },
    Result {
        index: u32,
    },
    Constant {
        name: String,
    },
    BitVecLiteral {
        value: String,
        width: u32,
        signed: bool,
    },
    Apply {
        function: String,
        args: Vec<MpkExprTerm>,
    },
    Convert {
        value: Box<MpkExprTerm>,
        target: MpkTypeTerm,
    },
}

impl MpkExprTerm {
    pub fn bool_literal(value: bool) -> Self {
        Self::Constant {
            name: if value {
                STD_BOOL_TRUE.to_owned()
            } else {
                STD_BOOL_FALSE.to_owned()
            },
        }
    }

    pub fn apply<I>(function: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = MpkExprTerm>,
    {
        Self::Apply {
            function: function.into(),
            args: args.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprEncodeError {
    GirValueAtomCount {
        atom_count: usize,
    },
    ContractAtomCount {
        atom_count: usize,
    },
    EmptyVariableName,
    AtomContainsOperatorFields,
    OperatorContainsAtom {
        op: String,
    },
    OperatorShape {
        op: String,
        expected: &'static str,
    },
    OperatorArity {
        op: String,
        expected: &'static str,
        actual: usize,
    },
    MissingConvertType {
        op: String,
    },
    MissingExpressionType {
        op: String,
        operand: &'static str,
    },
    ExpectedBool {
        op: String,
        operand: &'static str,
        actual: MpkExprType,
    },
    ExpectedBitVector {
        op: String,
        operand: &'static str,
        actual: MpkExprType,
    },
    TypeMismatch {
        op: String,
        lhs: MpkExprType,
        rhs: MpkExprType,
    },
    BitVectorWidthMismatch {
        op: String,
        lhs_width: u32,
        rhs_width: u32,
    },
    SignednessMismatch {
        op: String,
        expected_signed: bool,
        lhs: bool,
        rhs: bool,
    },
    MissingInstructionOp {
        instruction_id: String,
    },
    MissingInstructionValue {
        instruction_id: String,
    },
    MissingInstructionOperand {
        instruction_id: String,
        operand: &'static str,
    },
    UnsupportedInstructionKind {
        instruction_id: String,
        kind: GirInstructionKind,
    },
    UnsupportedOperator {
        op: String,
    },
    UnsupportedIntLiteralWidth {
        width: u32,
    },
    InvalidIntLiteral {
        value: String,
        reason: &'static str,
    },
    Type(TypeEncodeError),
}

impl fmt::Display for ExprEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GirValueAtomCount { atom_count } => {
                write!(
                    formatter,
                    "GIR value has {atom_count} atoms; expected exactly one"
                )
            }
            Self::ContractAtomCount { atom_count } => write!(
                formatter,
                "GIR contract expression has {atom_count} atoms; expected exactly one"
            ),
            Self::EmptyVariableName => write!(formatter, "GIR variable name is empty"),
            Self::AtomContainsOperatorFields => {
                write!(formatter, "GIR atom contains operator fields")
            }
            Self::OperatorContainsAtom { op } => {
                write!(formatter, "GIR operator {op:?} also contains atom fields")
            }
            Self::OperatorShape { op, expected } => {
                write!(formatter, "GIR operator {op:?} expects {expected}")
            }
            Self::OperatorArity {
                op,
                expected,
                actual,
            } => write!(
                formatter,
                "GIR operator {op:?} expects {expected}; got {actual}"
            ),
            Self::MissingConvertType { op } => {
                write!(formatter, "GIR operator {op:?} is missing target type")
            }
            Self::MissingExpressionType { op, operand } => write!(
                formatter,
                "GIR operator {op:?} needs type for {operand}"
            ),
            Self::ExpectedBool {
                op,
                operand,
                actual,
            } => write!(
                formatter,
                "GIR operator {op:?} expected bool {operand}; got {actual:?}"
            ),
            Self::ExpectedBitVector {
                op,
                operand,
                actual,
            } => write!(
                formatter,
                "GIR operator {op:?} expected bitvector {operand}; got {actual:?}"
            ),
            Self::TypeMismatch { op, lhs, rhs } => write!(
                formatter,
                "GIR operator {op:?} expected matching operand types; got {lhs:?} and {rhs:?}"
            ),
            Self::BitVectorWidthMismatch {
                op,
                lhs_width,
                rhs_width,
            } => write!(
                formatter,
                "GIR operator {op:?} expected matching bitvector widths; got {lhs_width} and {rhs_width}"
            ),
            Self::SignednessMismatch {
                op,
                expected_signed,
                lhs,
                rhs,
            } => write!(
                formatter,
                "GIR operator {op:?} expected signed={expected_signed} operands; got lhs signed={lhs}, rhs signed={rhs}"
            ),
            Self::MissingInstructionOp { instruction_id } => {
                write!(formatter, "GIR instruction {instruction_id:?} is missing op")
            }
            Self::MissingInstructionValue { instruction_id } => {
                write!(formatter, "GIR instruction {instruction_id:?} is missing value")
            }
            Self::MissingInstructionOperand {
                instruction_id,
                operand,
            } => write!(
                formatter,
                "GIR instruction {instruction_id:?} is missing {operand}"
            ),
            Self::UnsupportedInstructionKind {
                instruction_id,
                kind,
            } => write!(
                formatter,
                "GIR instruction {instruction_id:?} has unsupported expression kind {kind:?}"
            ),
            Self::UnsupportedOperator { op } => {
                write!(formatter, "GIR operator {op:?} is not supported")
            }
            Self::UnsupportedIntLiteralWidth { width } => {
                write!(formatter, "GIR int literal width {width} is not supported")
            }
            Self::InvalidIntLiteral { value, reason } => {
                write!(formatter, "GIR int literal {value:?} is invalid: {reason}")
            }
            Self::Type(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ExprEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Type(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bool_type() -> GirType {
        GirType {
            kind: GirTypeKind::Bool,
            name: None,
            width: None,
            signed: None,
            length: None,
            element: None,
            fields: Vec::new(),
        }
    }

    fn bv_type(width: u32, signed: bool) -> GirType {
        GirType {
            kind: GirTypeKind::BitVector,
            name: None,
            width: Some(width),
            signed: Some(signed),
            length: None,
            element: None,
            fields: Vec::new(),
        }
    }

    fn value_var(name: &str) -> GirValue {
        GirValue {
            var: Some(name.to_owned()),
            int: None,
            bool: None,
        }
    }

    fn value_int(value: &str, width: u32, signed: bool) -> GirValue {
        GirValue {
            var: None,
            int: Some(GirIntLiteral {
                value: value.to_owned(),
                width,
                signed,
            }),
            bool: None,
        }
    }

    fn value_bool(value: bool) -> GirValue {
        GirValue {
            var: None,
            int: None,
            bool: Some(value),
        }
    }

    fn expr_atom() -> GirContractExpr {
        GirContractExpr {
            op: None,
            args: Vec::new(),
            lhs: None,
            rhs: None,
            value: None,
            r#type: None,
            var: None,
            result: None,
            bool: None,
            int: None,
        }
    }

    fn expr_var(name: &str) -> GirContractExpr {
        GirContractExpr {
            var: Some(name.to_owned()),
            ..expr_atom()
        }
    }

    fn expr_result(index: u32) -> GirContractExpr {
        GirContractExpr {
            result: Some(index),
            ..expr_atom()
        }
    }

    fn expr_bool(value: bool) -> GirContractExpr {
        GirContractExpr {
            bool: Some(value),
            ..expr_atom()
        }
    }

    fn expr_int(value: &str, width: u32, signed: bool) -> GirContractExpr {
        GirContractExpr {
            int: Some(GirIntLiteral {
                value: value.to_owned(),
                width,
                signed,
            }),
            ..expr_atom()
        }
    }

    fn expr_binary(op: &str, lhs: GirContractExpr, rhs: GirContractExpr) -> GirContractExpr {
        GirContractExpr {
            op: Some(op.to_owned()),
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            ..expr_atom()
        }
    }

    fn instruction_binop(
        id: &str,
        op: &str,
        result_type: GirType,
        lhs: GirValue,
        rhs: GirValue,
    ) -> GirInstruction {
        GirInstruction {
            id: id.to_owned(),
            kind: GirInstructionKind::BinOp,
            op: Some(op.to_owned()),
            r#type: result_type,
            target: None,
            value: None,
            base: None,
            index: None,
            field: None,
            fields: Vec::new(),
            elements: Vec::new(),
            lhs: Some(lhs),
            rhs: Some(rhs),
            function: None,
            args: Vec::new(),
        }
    }

    fn instruction_unary(
        id: &str,
        op: &str,
        result_type: GirType,
        value: GirValue,
    ) -> GirInstruction {
        GirInstruction {
            id: id.to_owned(),
            kind: GirInstructionKind::UnaryOp,
            op: Some(op.to_owned()),
            r#type: result_type,
            target: None,
            value: Some(value),
            base: None,
            index: None,
            field: None,
            fields: Vec::new(),
            elements: Vec::new(),
            lhs: None,
            rhs: None,
            function: None,
            args: Vec::new(),
        }
    }

    fn snapshot(term: &MpkExprTerm) -> String {
        serde_json::to_string_pretty(term).expect("expression term serializes")
    }

    #[test]
    fn encodes_constants_and_variables_snapshot() {
        let true_term = encode_gir_value(&value_bool(true)).expect("bool literal encodes");
        let var_term = encode_gir_value(&value_var("x")).expect("variable encodes");

        assert_eq!(
            snapshot(&true_term),
            r#"{
  "kind": "constant",
  "name": "Std.Bool.true"
}"#
        );
        assert_eq!(
            snapshot(&var_term),
            r#"{
  "kind": "var",
  "name": "x"
}"#
        );
    }

    #[test]
    fn encodes_bool_false_name() {
        let term = encode_gir_value(&value_bool(false)).expect("bool literal encodes");

        assert_eq!(
            term,
            MpkExprTerm::Constant {
                name: STD_BOOL_FALSE.to_owned()
            }
        );
    }

    #[test]
    fn encodes_contract_comparison_snapshot() {
        let encoder = ExprEncoder::with_context(
            ExprContext::new()
                .with_variable("a", bv_type(64, true))
                .with_result(0, bv_type(64, true)),
        );
        let term = encoder
            .encode_contract_expr(&expr_binary("signed_ge", expr_var("a"), expr_result(0)))
            .expect("comparison encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "apply",
  "function": "Std.BitVec.BV64.sge",
  "args": [
    {
      "kind": "var",
      "name": "a"
    },
    {
      "kind": "result",
      "index": 0
    }
  ]
}"#
        );
    }

    #[test]
    fn encodes_instruction_binop_snapshot() {
        let encoder =
            ExprEncoder::with_context(ExprContext::new().with_variable("a", bv_type(64, true)));
        let instruction = instruction_binop(
            "sum",
            "bv_add",
            bv_type(64, true),
            value_var("a"),
            value_int("1", 64, true),
        );
        let term = encoder
            .encode_instruction(&instruction)
            .expect("binop instruction encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "apply",
  "function": "Std.BitVec.BV64.add",
  "args": [
    {
      "kind": "var",
      "name": "a"
    },
    {
      "kind": "bit_vec_literal",
      "value": "1",
      "width": 64,
      "signed": true
    }
  ]
}"#
        );
    }

    #[test]
    fn encodes_convert_snapshot() {
        let input = GirContractExpr {
            op: Some("convert".to_owned()),
            value: Some(Box::new(expr_var("x"))),
            r#type: Some(bv_type(32, false)),
            ..expr_atom()
        };
        let term = encode_contract_expr(&input).expect("convert encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "convert",
  "value": {
    "kind": "var",
    "name": "x"
  },
  "target": {
    "kind": "constant",
    "name": "Std.Program.Base.Uint32"
  }
}"#
        );
    }

    #[test]
    fn encodes_variadic_bool_as_nested_binary_snapshot() {
        let input = GirContractExpr {
            op: Some("or".to_owned()),
            args: vec![expr_bool(false), expr_bool(true), expr_bool(false)],
            ..expr_atom()
        };
        let term = encode_contract_expr(&input).expect("variadic bool op encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "apply",
  "function": "Std.Bool.or",
  "args": [
    {
      "kind": "apply",
      "function": "Std.Bool.or",
      "args": [
        {
          "kind": "constant",
          "name": "Std.Bool.false"
        },
        {
          "kind": "constant",
          "name": "Std.Bool.true"
        }
      ]
    },
    {
      "kind": "constant",
      "name": "Std.Bool.false"
    }
  ]
}"#
        );
    }

    #[test]
    fn rejects_multiple_gir_value_atoms() {
        let input = GirValue {
            var: Some("x".to_owned()),
            int: None,
            bool: Some(true),
        };

        assert_eq!(
            encode_gir_value(&input),
            Err(ExprEncodeError::GirValueAtomCount { atom_count: 2 })
        );
    }

    #[test]
    fn rejects_unknown_operator() {
        assert_eq!(
            encode_contract_expr(&expr_binary("unknown", expr_bool(true), expr_bool(false))),
            Err(ExprEncodeError::UnsupportedOperator {
                op: "unknown".to_owned()
            })
        );
    }

    #[test]
    fn rejects_unsupported_bitvec_division_operator() {
        assert_eq!(
            encode_contract_expr(&expr_binary(
                "bv_sdiv",
                expr_int("4", 64, true),
                expr_int("2", 64, true)
            )),
            Err(ExprEncodeError::UnsupportedOperator {
                op: "bv_sdiv".to_owned()
            })
        );
    }

    #[test]
    fn rejects_negative_unsigned_literal() {
        assert_eq!(
            encode_gir_value(&value_int("-1", 8, false)),
            Err(ExprEncodeError::InvalidIntLiteral {
                value: "-1".to_owned(),
                reason: "unsigned literal is negative"
            })
        );
    }

    #[test]
    fn rejects_whitespace_int_literal() {
        assert_eq!(
            encode_gir_value(&value_int(" 1", 8, true)),
            Err(ExprEncodeError::InvalidIntLiteral {
                value: " 1".to_owned(),
                reason: "literal contains leading or trailing whitespace"
            })
        );
    }

    #[test]
    fn rejects_missing_binary_rhs() {
        let encoder =
            ExprEncoder::with_context(ExprContext::new().with_variable("a", bv_type(64, true)));
        let input = GirContractExpr {
            op: Some("signed_ge".to_owned()),
            lhs: Some(Box::new(expr_var("a"))),
            ..expr_atom()
        };

        assert_eq!(
            encoder.encode_contract_expr(&input),
            Err(ExprEncodeError::OperatorShape {
                op: "signed_ge".to_owned(),
                expected: "both lhs and rhs"
            })
        );
    }

    #[test]
    fn rejects_missing_bitvec_type_context() {
        assert_eq!(
            encode_contract_expr(&expr_binary("signed_ge", expr_var("a"), expr_result(0))),
            Err(ExprEncodeError::MissingExpressionType {
                op: "signed_ge".to_owned(),
                operand: "lhs"
            })
        );
    }

    #[test]
    fn rejects_instruction_eq_with_non_bool_result_type() {
        let encoder =
            ExprEncoder::with_context(ExprContext::new().with_variable("a", bv_type(64, true)));
        let instruction = instruction_binop(
            "bad_eq",
            "eq",
            bv_type(64, true),
            value_var("a"),
            value_int("1", 64, true),
        );

        assert_eq!(
            encoder.encode_instruction(&instruction),
            Err(ExprEncodeError::ExpectedBool {
                op: "eq".to_owned(),
                operand: "result",
                actual: MpkExprType::BitVector {
                    width: 64,
                    signed: true
                }
            })
        );
    }

    #[test]
    fn rejects_instruction_not_with_non_bool_result_type() {
        let instruction = instruction_unary("bad_not", "not", bv_type(8, false), value_bool(true));

        assert_eq!(
            encode_instruction_expr(&instruction),
            Err(ExprEncodeError::ExpectedBool {
                op: "not".to_owned(),
                operand: "result",
                actual: MpkExprType::BitVector {
                    width: 8,
                    signed: false
                }
            })
        );
    }

    #[test]
    fn function_context_includes_bindings_results_and_instructions() {
        let function = GirFunction {
            id: "pkg.f".to_owned(),
            package: "pkg".to_owned(),
            name: "f".to_owned(),
            params: vec![GirBinding {
                name: "a".to_owned(),
                r#type: bv_type(64, true),
            }],
            results: vec![GirBinding {
                name: "ret".to_owned(),
                r#type: bool_type(),
            }],
            locals: Vec::new(),
            blocks: vec![crate::gir::GirBlock {
                label: "entry".to_owned(),
                parameters: vec![GirBinding {
                    name: "p".to_owned(),
                    r#type: bv_type(32, false),
                }],
                instructions: vec![instruction_binop(
                    "sum",
                    "bv_add",
                    bv_type(64, true),
                    value_var("a"),
                    value_int("1", 64, true),
                )],
                terminator: crate::gir::GirTerminator {
                    kind: crate::gir::GirTerminatorKind::Return,
                    values: Vec::new(),
                    cond: None,
                    label: None,
                    then_label: None,
                    else_label: None,
                    args: Vec::new(),
                    reason: None,
                },
            }],
            contracts: crate::gir::GirContracts {
                requires: Vec::new(),
                ensures: Vec::new(),
                modifies: Vec::new(),
                loops: Vec::new(),
            },
            supported_features: Vec::new(),
            rejected_features: Vec::new(),
        };
        let context = ExprContext::for_function(&function);

        assert_eq!(context.variable_type("a").and_then(|ty| ty.width), Some(64));
        assert_eq!(context.variable_type("p").and_then(|ty| ty.width), Some(32));
        assert_eq!(
            context.variable_type("sum").and_then(|ty| ty.width),
            Some(64)
        );
        assert_eq!(
            context.result_type(0).map(|ty| ty.kind),
            Some(GirTypeKind::Bool)
        );
    }
}
