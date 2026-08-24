//! UNTRUSTED, TEST-ONLY VIR evaluator.
//!
//! This module is deliberately located below `tests/`, is never exported by
//! `mpk-vc`, and is not an implementation of the trusted VC semantics.  It is
//! a small executable oracle for differential tests over already validated VIR.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mpk_vc::{
    BitVectorWidth, OverflowOperation, VirBinaryOperator, VirBlock, VirFunction, VirInstruction,
    VirLiteral, VirModule, VirSafetyCheck, VirSafetyCheckKind, VirTerminator, VirType,
    VirUnaryOperator, VirValue,
};

const STEP_LIMIT: usize = 100_000;
const CALL_DEPTH_LIMIT: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    BitVector {
        width: BitVectorWidth,
        signed: bool,
        bits: u64,
    },
    Array(Vec<RuntimeValue>),
    Struct {
        type_id: String,
        fields: Vec<(String, RuntimeValue)>,
    },
}

impl RuntimeValue {
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn unsigned(width: u32, value: u64) -> Self {
        let width = BitVectorWidth::try_from(width).expect("test bitvector width");
        Self::BitVector {
            width,
            signed: false,
            bits: value & bit_mask(width),
        }
    }

    pub fn signed(width: u32, value: i128) -> Self {
        let width = BitVectorWidth::try_from(width).expect("test bitvector width");
        Self::BitVector {
            width,
            signed: true,
            bits: wrap_signed(value, width),
        }
    }

    pub fn array(elements: impl IntoIterator<Item = RuntimeValue>) -> Self {
        Self::Array(elements.into_iter().collect())
    }

    pub fn structure(
        type_id: impl Into<String>,
        fields: impl IntoIterator<Item = (impl Into<String>, RuntimeValue)>,
    ) -> Self {
        Self::Struct {
            type_id: type_id.into(),
            fields: fields
                .into_iter()
                .map(|(name, value)| (name.into(), value))
                .collect(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            other => panic!("expected bool, found {other:?}"),
        }
    }

    pub fn as_unsigned(&self) -> u64 {
        match self {
            Self::BitVector { bits, .. } => *bits,
            other => panic!("expected bitvector, found {other:?}"),
        }
    }

    pub fn as_signed(&self) -> i128 {
        match self {
            Self::BitVector { width, bits, .. } => signed_value(*bits, *width),
            other => panic!("expected bitvector, found {other:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeledPanic {
    IntegerOverflow,
    DivisionByZero,
    SignedDivisionOverflow,
    NegativeShift,
    ShiftOutOfRange,
    IndexOutOfBounds,
}

impl ModeledPanic {
    pub const fn safety_kind(self) -> VirSafetyCheckKind {
        match self {
            Self::IntegerOverflow => VirSafetyCheckKind::IntegerNoOverflow,
            Self::DivisionByZero => VirSafetyCheckKind::DivisorNonzero,
            Self::SignedDivisionOverflow => VirSafetyCheckKind::SignedDivremRepresentable,
            Self::NegativeShift => VirSafetyCheckKind::ShiftCountNonnegative,
            Self::ShiftOutOfRange => VirSafetyCheckKind::ShiftCountLessThanWidth,
            Self::IndexOutOfBounds => VirSafetyCheckKind::IndexInBounds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionOutcome {
    Returned(Vec<RuntimeValue>),
    Panicked(ModeledPanic),
}

pub fn execute(module: &VirModule, function_id: &str, args: Vec<RuntimeValue>) -> ExecutionOutcome {
    let mut interpreter = Interpreter { module, steps: 0 };
    match interpreter.call(function_id, args, 0) {
        Ok(values) => ExecutionOutcome::Returned(values),
        Err(panic) => ExecutionOutcome::Panicked(panic),
    }
}

pub fn evaluate_modeled_safety(
    checks: &[VirSafetyCheck],
    lhs: &RuntimeValue,
    rhs: Option<&RuntimeValue>,
) -> Result<(), ModeledPanic> {
    evaluate_safety_checks(checks, lhs, rhs)
}

struct Interpreter<'a> {
    module: &'a VirModule,
    steps: usize,
}

impl Interpreter<'_> {
    fn call(
        &mut self,
        function_id: &str,
        args: Vec<RuntimeValue>,
        depth: usize,
    ) -> Result<Vec<RuntimeValue>, ModeledPanic> {
        assert!(
            depth < CALL_DEPTH_LIMIT,
            "test interpreter call-depth limit"
        );
        let function = find_function(self.module, function_id);
        assert_eq!(function.params.len(), args.len(), "argument count");

        let mut values = BTreeMap::new();
        for (binding, value) in function.params.iter().zip(args) {
            assert_type(&value, &binding.r#type);
            assert!(values.insert(binding.id.clone(), value).is_none());
        }

        let mut block = function.blocks.first().expect("validated entry block");
        loop {
            self.steps += 1;
            assert!(self.steps <= STEP_LIMIT, "test interpreter step limit");

            for instruction in &block.instructions {
                let result =
                    self.evaluate_instruction(function, instruction, &mut values, depth)?;
                let id = instruction_id(instruction);
                assert!(values.insert(id.to_owned(), result).is_none());
            }

            match &block.terminator {
                VirTerminator::Return { values: returned } => {
                    let returned = returned
                        .iter()
                        .map(|value| resolve_value(self.module, function, &values, value))
                        .collect::<Vec<_>>();
                    for (value, binding) in returned.iter().zip(&function.results) {
                        assert_type(value, &binding.r#type);
                    }
                    return Ok(returned);
                }
                VirTerminator::Jump { label, args } => {
                    block = enter_block(self.module, function, &mut values, label, args);
                }
                VirTerminator::Branch {
                    cond,
                    then_label,
                    then_args,
                    else_label,
                    else_args,
                } => {
                    let condition = resolve_value(self.module, function, &values, cond).as_bool();
                    let (label, args) = if condition {
                        (then_label, then_args)
                    } else {
                        (else_label, else_args)
                    };
                    block = enter_block(self.module, function, &mut values, label, args);
                }
            }
        }
    }

    fn evaluate_instruction(
        &mut self,
        function: &VirFunction,
        instruction: &VirInstruction,
        values: &mut BTreeMap<String, RuntimeValue>,
        depth: usize,
    ) -> Result<RuntimeValue, ModeledPanic> {
        match instruction {
            VirInstruction::Const {
                r#type,
                value,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                Ok(literal(value, r#type))
            }
            VirInstruction::Copy {
                r#type,
                target,
                value,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                let value = resolve_value(self.module, function, values, value);
                assert_type(&value, r#type);
                values.insert(target.clone(), value.clone());
                Ok(value)
            }
            VirInstruction::BinOp {
                op,
                r#type,
                lhs,
                rhs,
                safety_checks,
                ..
            } => {
                let lhs = resolve_value(self.module, function, values, lhs);
                let rhs = resolve_value(self.module, function, values, rhs);
                evaluate_safety_checks(safety_checks, &lhs, Some(&rhs))?;
                let result = total_binary(*op, &lhs, &rhs);
                assert_type(&result, r#type);
                Ok(result)
            }
            VirInstruction::UnaryOp {
                op,
                r#type,
                value,
                safety_checks,
                ..
            } => {
                let value = resolve_value(self.module, function, values, value);
                evaluate_safety_checks(safety_checks, &value, None)?;
                let result = total_unary(*op, &value);
                assert_type(&result, r#type);
                Ok(result)
            }
            VirInstruction::Convert {
                r#type,
                value,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                let value = resolve_value(self.module, function, values, value);
                Ok(total_convert(&value, r#type))
            }
            VirInstruction::Field {
                r#type,
                base,
                field,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                let base = resolve_value(self.module, function, values, base);
                let RuntimeValue::Struct { fields, .. } = base else {
                    panic!("validated Field base was not a struct")
                };
                let result = fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *field).then_some(value))
                    .expect("validated struct field");
                assert_type(&result, r#type);
                Ok(result)
            }
            VirInstruction::Index {
                r#type,
                base,
                index,
                safety_checks,
                ..
            } => {
                let base = resolve_value(self.module, function, values, base);
                let index = resolve_value(self.module, function, values, index);
                evaluate_safety_checks(safety_checks, &base, Some(&index))?;
                let RuntimeValue::Array(elements) = base else {
                    panic!("validated Index base was not an array")
                };
                let result = index_value(&index)
                    .and_then(|index| elements.get(index).cloned())
                    .unwrap_or_else(|| zero_value(self.module, r#type));
                assert_type(&result, r#type);
                Ok(result)
            }
            VirInstruction::MakeStruct {
                r#type,
                fields,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                let VirType::Struct { id } = r#type else {
                    panic!("validated MakeStruct result was not a struct")
                };
                Ok(RuntimeValue::Struct {
                    type_id: id.clone(),
                    fields: fields
                        .iter()
                        .map(|field| {
                            (
                                field.name.clone(),
                                resolve_value(self.module, function, values, &field.value),
                            )
                        })
                        .collect(),
                })
            }
            VirInstruction::MakeArray {
                elements,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                Ok(RuntimeValue::Array(
                    elements
                        .iter()
                        .map(|value| resolve_value(self.module, function, values, value))
                        .collect(),
                ))
            }
            VirInstruction::CallStatic {
                r#type,
                function: callee,
                args,
                safety_checks,
                ..
            } => {
                assert!(safety_checks.is_empty());
                let args = args
                    .iter()
                    .map(|value| resolve_value(self.module, function, values, value))
                    .collect();
                let returned = self.call(callee, args, depth + 1)?;
                assert_eq!(returned.len(), 1, "validated static-call result count");
                let result = returned.into_iter().next().unwrap();
                assert_type(&result, r#type);
                Ok(result)
            }
        }
    }
}

fn enter_block<'a>(
    module: &VirModule,
    function: &'a VirFunction,
    values: &mut BTreeMap<String, RuntimeValue>,
    label: &str,
    args: &[VirValue],
) -> &'a VirBlock {
    let block = function
        .blocks
        .iter()
        .find(|block| block.label == label)
        .expect("validated jump target");
    let resolved = args
        .iter()
        .map(|value| resolve_value(module, function, values, value))
        .collect::<Vec<_>>();
    assert_eq!(block.parameters.len(), resolved.len());
    for (binding, value) in block.parameters.iter().zip(resolved) {
        assert_type(&value, &binding.r#type);
        values.insert(binding.id.clone(), value);
    }
    block
}

fn find_function<'a>(module: &'a VirModule, function_id: &str) -> &'a VirFunction {
    module
        .units
        .iter()
        .flat_map(|unit| &unit.functions)
        .find(|function| function.id == function_id)
        .unwrap_or_else(|| panic!("validated function {function_id:?}"))
}

fn resolve_value(
    module: &VirModule,
    function: &VirFunction,
    values: &BTreeMap<String, RuntimeValue>,
    value: &VirValue,
) -> RuntimeValue {
    match value {
        VirValue::Variable(reference) => values
            .get(&reference.var)
            .unwrap_or_else(|| panic!("initialized variable {:?}", reference.var))
            .clone(),
        VirValue::Constant(reference) => {
            let declaration = module
                .units
                .iter()
                .find(|unit| unit.id == function.unit_id)
                .and_then(|unit| {
                    unit.const_decls
                        .iter()
                        .find(|declaration| declaration.id == reference.constant)
                })
                .expect("validated constant reference");
            literal(&declaration.value, &declaration.r#type)
        }
        VirValue::Boolean(literal) => RuntimeValue::Bool(literal.value),
        VirValue::Integer(literal) => int_literal(&literal.int),
    }
}

fn literal(value: &VirLiteral, ty: &VirType) -> RuntimeValue {
    let value = match value {
        VirLiteral::Boolean(value) => RuntimeValue::Bool(value.value),
        VirLiteral::Integer(value) => int_literal(&value.int),
    };
    assert_type(&value, ty);
    value
}

fn int_literal(value: &mpk_vc::VirIntLiteral) -> RuntimeValue {
    let mathematical = value
        .value
        .as_str()
        .parse::<i128>()
        .expect("validated decimal integer");
    RuntimeValue::BitVector {
        width: value.width,
        signed: value.signed,
        bits: wrap_signed(mathematical, value.width),
    }
}

pub fn total_unary(operation: VirUnaryOperator, value: &RuntimeValue) -> RuntimeValue {
    match (operation, value) {
        (VirUnaryOperator::Not, RuntimeValue::Bool(value)) => RuntimeValue::Bool(!value),
        (
            VirUnaryOperator::BvNeg,
            RuntimeValue::BitVector {
                width,
                signed,
                bits,
            },
        ) => RuntimeValue::BitVector {
            width: *width,
            signed: *signed,
            bits: bits.wrapping_neg() & bit_mask(*width),
        },
        (
            VirUnaryOperator::BvNot,
            RuntimeValue::BitVector {
                width,
                signed,
                bits,
            },
        ) => RuntimeValue::BitVector {
            width: *width,
            signed: *signed,
            bits: !bits & bit_mask(*width),
        },
        _ => panic!("validated unary operation {operation:?} on {value:?}"),
    }
}

pub fn total_binary(
    operation: VirBinaryOperator,
    lhs: &RuntimeValue,
    rhs: &RuntimeValue,
) -> RuntimeValue {
    use VirBinaryOperator as Op;

    if matches!(operation, Op::Eq | Op::NotEq) {
        let equal = lhs == rhs;
        return RuntimeValue::Bool(if operation == Op::Eq { equal } else { !equal });
    }

    let (width, signed, lhs_bits) = bitvector(lhs);
    let (_, _, rhs_bits) = bitvector(rhs);
    let mask = bit_mask(width);
    let result = match operation {
        Op::BvAdd => lhs_bits.wrapping_add(rhs_bits) & mask,
        Op::BvSub => lhs_bits.wrapping_sub(rhs_bits) & mask,
        Op::BvMul => lhs_bits.wrapping_mul(rhs_bits) & mask,
        Op::BvUdiv => lhs_bits.checked_div(rhs_bits).unwrap_or(mask),
        Op::BvUrem => lhs_bits.checked_rem(rhs_bits).unwrap_or(lhs_bits),
        Op::BvSdiv => signed_division(lhs_bits, rhs_bits, width),
        Op::BvSrem => signed_remainder(lhs_bits, rhs_bits, width),
        Op::BvAnd => lhs_bits & rhs_bits,
        Op::BvOr => lhs_bits | rhs_bits,
        Op::BvXor => lhs_bits ^ rhs_bits,
        Op::BvShl => {
            if rhs_bits >= u64::from(width.bits()) {
                0
            } else {
                lhs_bits.wrapping_shl(rhs_bits as u32) & mask
            }
        }
        Op::BvLshr => {
            if rhs_bits >= u64::from(width.bits()) {
                0
            } else {
                lhs_bits >> rhs_bits
            }
        }
        Op::BvAshr => {
            if rhs_bits >= u64::from(width.bits()) {
                if signed_value(lhs_bits, width) < 0 {
                    mask
                } else {
                    0
                }
            } else {
                wrap_signed(signed_value(lhs_bits, width) >> rhs_bits, width)
            }
        }
        Op::SignedLt => {
            return RuntimeValue::Bool(
                signed_value(lhs_bits, width) < signed_value(rhs_bits, width),
            )
        }
        Op::SignedLe => {
            return RuntimeValue::Bool(
                signed_value(lhs_bits, width) <= signed_value(rhs_bits, width),
            )
        }
        Op::SignedGt => {
            return RuntimeValue::Bool(
                signed_value(lhs_bits, width) > signed_value(rhs_bits, width),
            )
        }
        Op::SignedGe => {
            return RuntimeValue::Bool(
                signed_value(lhs_bits, width) >= signed_value(rhs_bits, width),
            )
        }
        Op::UnsignedLt => return RuntimeValue::Bool(lhs_bits < rhs_bits),
        Op::UnsignedLe => return RuntimeValue::Bool(lhs_bits <= rhs_bits),
        Op::UnsignedGt => return RuntimeValue::Bool(lhs_bits > rhs_bits),
        Op::UnsignedGe => return RuntimeValue::Bool(lhs_bits >= rhs_bits),
        Op::Eq | Op::NotEq => unreachable!(),
    };
    RuntimeValue::BitVector {
        width,
        signed,
        bits: result,
    }
}

pub fn total_convert(value: &RuntimeValue, target: &VirType) -> RuntimeValue {
    let (source_width, source_signed, bits) = bitvector(value);
    let VirType::Bv {
        width: target_width,
        signed: target_signed,
    } = target
    else {
        panic!("validated Convert target was not a bitvector")
    };
    let bits = if target_width.bits() > source_width.bits() && source_signed {
        wrap_signed(signed_value(bits, source_width), *target_width)
    } else {
        bits & bit_mask(*target_width)
    };
    RuntimeValue::BitVector {
        width: *target_width,
        signed: *target_signed,
        bits,
    }
}

fn evaluate_safety_checks(
    checks: &[VirSafetyCheck],
    lhs: &RuntimeValue,
    rhs: Option<&RuntimeValue>,
) -> Result<(), ModeledPanic> {
    for check in checks {
        let failed = match check {
            VirSafetyCheck::IntegerNoOverflow {
                operation: overflow,
                signed,
            } => !integer_operation_fits(*overflow, *signed, lhs, rhs),
            VirSafetyCheck::DivisorNonzero {} => rhs.expect("divisor").as_unsigned() == 0,
            VirSafetyCheck::SignedDivremRepresentable { .. } => {
                let (width, _, lhs_bits) = bitvector(lhs);
                let (_, _, rhs_bits) = bitvector(rhs.expect("division RHS"));
                signed_value(lhs_bits, width) == signed_min(width)
                    && signed_value(rhs_bits, width) == -1
            }
            VirSafetyCheck::ShiftCountNonnegative {} => {
                let (width, _, bits) = bitvector(rhs.expect("shift RHS"));
                signed_value(bits, width) < 0
            }
            VirSafetyCheck::ShiftCountLessThanWidth {} => {
                let (lhs_width, _, _) = bitvector(lhs);
                rhs.expect("shift RHS").as_unsigned() >= u64::from(lhs_width.bits())
            }
            VirSafetyCheck::IndexInBounds {} => {
                let RuntimeValue::Array(elements) = lhs else {
                    panic!("validated Index base")
                };
                index_value(rhs.expect("index RHS"))
                    .map(|index| index >= elements.len())
                    .unwrap_or(true)
            }
        };
        if failed {
            return Err(match check.kind() {
                VirSafetyCheckKind::IntegerNoOverflow => ModeledPanic::IntegerOverflow,
                VirSafetyCheckKind::DivisorNonzero => ModeledPanic::DivisionByZero,
                VirSafetyCheckKind::SignedDivremRepresentable => {
                    ModeledPanic::SignedDivisionOverflow
                }
                VirSafetyCheckKind::ShiftCountNonnegative => ModeledPanic::NegativeShift,
                VirSafetyCheckKind::ShiftCountLessThanWidth => ModeledPanic::ShiftOutOfRange,
                VirSafetyCheckKind::IndexInBounds => ModeledPanic::IndexOutOfBounds,
            });
        }
    }
    Ok(())
}

fn integer_operation_fits(
    operation: OverflowOperation,
    signed: bool,
    lhs: &RuntimeValue,
    rhs: Option<&RuntimeValue>,
) -> bool {
    let (width, _, lhs_bits) = bitvector(lhs);
    if signed {
        let lhs = signed_value(lhs_bits, width);
        let result = match operation {
            OverflowOperation::Add => lhs + rhs.expect("add RHS").as_signed(),
            OverflowOperation::Sub => lhs - rhs.expect("sub RHS").as_signed(),
            OverflowOperation::Mul => lhs * rhs.expect("mul RHS").as_signed(),
            OverflowOperation::Neg => -lhs,
        };
        (signed_min(width)..=signed_max(width)).contains(&result)
    } else {
        let lhs = u128::from(lhs_bits);
        let rhs = rhs.map(RuntimeValue::as_unsigned).map(u128::from);
        let maximum = u128::from(bit_mask(width));
        match operation {
            OverflowOperation::Add => lhs + rhs.expect("add RHS") <= maximum,
            OverflowOperation::Sub => lhs >= rhs.expect("sub RHS"),
            OverflowOperation::Mul => lhs * rhs.expect("mul RHS") <= maximum,
            OverflowOperation::Neg => false,
        }
    }
}

fn zero_value(module: &VirModule, ty: &VirType) -> RuntimeValue {
    match ty {
        VirType::Bool {} => RuntimeValue::Bool(false),
        VirType::Bv { width, signed } => RuntimeValue::BitVector {
            width: *width,
            signed: *signed,
            bits: 0,
        },
        VirType::Array { length, element } => RuntimeValue::Array(
            (0..length.get())
                .map(|_| zero_value(module, element))
                .collect(),
        ),
        VirType::Struct { id } => {
            let declaration = module
                .units
                .iter()
                .flat_map(|unit| &unit.type_decls)
                .find(|declaration| declaration.id == *id)
                .expect("validated zero struct type");
            RuntimeValue::Struct {
                type_id: id.clone(),
                fields: declaration
                    .fields
                    .iter()
                    .map(|field| (field.name.clone(), zero_value(module, &field.r#type)))
                    .collect(),
            }
        }
    }
}

fn assert_type(value: &RuntimeValue, ty: &VirType) {
    let matches = match (value, ty) {
        (RuntimeValue::Bool(_), VirType::Bool {}) => true,
        (
            RuntimeValue::BitVector {
                width,
                signed,
                bits,
            },
            VirType::Bv {
                width: expected_width,
                signed: expected_signed,
            },
        ) => {
            width == expected_width
                && signed == expected_signed
                && *bits <= bit_mask(*expected_width)
        }
        (RuntimeValue::Array(elements), VirType::Array { length, element }) => {
            elements.len() == usize::from(length.get())
                && elements
                    .iter()
                    .all(|value| value_matches_type(value, element))
        }
        (RuntimeValue::Struct { type_id, .. }, VirType::Struct { id }) => type_id == id,
        _ => false,
    };
    assert!(matches, "runtime value {value:?} does not match {ty:?}");
}

fn value_matches_type(value: &RuntimeValue, ty: &VirType) -> bool {
    match (value, ty) {
        (RuntimeValue::Bool(_), VirType::Bool {}) => true,
        (
            RuntimeValue::BitVector { width, signed, .. },
            VirType::Bv {
                width: expected_width,
                signed: expected_signed,
            },
        ) => width == expected_width && signed == expected_signed,
        (RuntimeValue::Array(elements), VirType::Array { length, element }) => {
            elements.len() == usize::from(length.get())
                && elements
                    .iter()
                    .all(|value| value_matches_type(value, element))
        }
        (RuntimeValue::Struct { type_id, .. }, VirType::Struct { id }) => type_id == id,
        _ => false,
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

fn bitvector(value: &RuntimeValue) -> (BitVectorWidth, bool, u64) {
    match value {
        RuntimeValue::BitVector {
            width,
            signed,
            bits,
        } => (*width, *signed, *bits),
        other => panic!("expected bitvector, found {other:?}"),
    }
}

fn index_value(value: &RuntimeValue) -> Option<usize> {
    let (width, signed, bits) = bitvector(value);
    if signed {
        usize::try_from(signed_value(bits, width)).ok()
    } else {
        usize::try_from(bits).ok()
    }
}

fn bit_mask(width: BitVectorWidth) -> u64 {
    if width.bits() == 64 {
        u64::MAX
    } else {
        (1_u64 << width.bits()) - 1
    }
}

fn signed_value(bits: u64, width: BitVectorWidth) -> i128 {
    let sign = 1_u64 << (width.bits() - 1);
    if bits & sign == 0 {
        i128::from(bits)
    } else {
        i128::from(bits) - (1_i128 << width.bits())
    }
}

fn wrap_signed(value: i128, width: BitVectorWidth) -> u64 {
    value.rem_euclid(1_i128 << width.bits()) as u64
}

fn signed_min(width: BitVectorWidth) -> i128 {
    -(1_i128 << (width.bits() - 1))
}

fn signed_max(width: BitVectorWidth) -> i128 {
    (1_i128 << (width.bits() - 1)) - 1
}

fn absolute_bits(bits: u64, width: BitVectorWidth) -> u64 {
    let signed = signed_value(bits, width);
    if signed < 0 {
        wrap_signed(-signed, width)
    } else {
        bits
    }
}

fn signed_division(lhs: u64, rhs: u64, width: BitVectorWidth) -> u64 {
    let quotient = absolute_bits(lhs, width)
        .checked_div(absolute_bits(rhs, width))
        .unwrap_or_else(|| bit_mask(width));
    if (signed_value(lhs, width) < 0) != (signed_value(rhs, width) < 0) {
        wrap_signed(-i128::from(quotient), width)
    } else {
        quotient
    }
}

fn signed_remainder(lhs: u64, rhs: u64, width: BitVectorWidth) -> u64 {
    let remainder = absolute_bits(lhs, width)
        .checked_rem(absolute_bits(rhs, width))
        .unwrap_or_else(|| absolute_bits(lhs, width));
    if signed_value(lhs, width) < 0 {
        wrap_signed(-i128::from(remainder), width)
    } else {
        remainder
    }
}
