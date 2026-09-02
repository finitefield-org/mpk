//! Profile-derived VIR safety checks and checked proposition encoding.
//!
//! Safety metadata names only a closed check kind. The proposition is always
//! regenerated from the validated instruction operands and semantic profile;
//! no frontend-provided proposition enters the VC path.

use std::fmt;

use crate::expr_encode::{
    MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_NOT, STD_BOOL_OR, STD_BOOL_TRUE, STD_EQ,
};
use crate::program_encode::{encode_vir_value, ProgramExprContext, ProgramExprEncodeError};
use crate::semantic_profile::SemanticProfile;
use crate::vir::{
    DivRemOperation, OverflowOperation, VirBinaryOperator, VirInstruction, VirInstructionKind,
    VirSafetyCheck, VirType, VirUnaryOperator, VirValue,
};

pub const SAFETY_OBLIGATION_KIND_COMPONENT: &str = "operation_safety";
pub const SAFETY_BITVEC_THEORY_FORMAT: &str = "mpk.bitvec-ground.v0";
pub const SAFETY_GROUPED_CERTIFICATE_FOUNDATION: &str = "Std.Program.Base";

/// Closed required-check dispatch used by the inactive successor VC path.
///
/// The active VIR v0 API continues to expose only [`SemanticProfile`].  The
/// C# variant is crate-private and can therefore be selected only after the
/// successor semantic-registry and source-artifact boundaries have validated
/// the complete C# context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompiledRequiredCheckProfile {
    GoFixedV0,
    RustCheckedV0,
    CSharpScalarV0,
    JavaScalarV0,
}

impl CompiledRequiredCheckProfile {
    const fn encoding_profile(self) -> SemanticProfile {
        match self {
            Self::GoFixedV0 | Self::CSharpScalarV0 | Self::JavaScalarV0 => {
                SemanticProfile::GoFixedV0
            }
            Self::RustCheckedV0 => SemanticProfile::RustCheckedV0,
        }
    }
}

impl From<SemanticProfile> for CompiledRequiredCheckProfile {
    fn from(value: SemanticProfile) -> Self {
        match value {
            SemanticProfile::GoFixedV0 => Self::GoFixedV0,
            SemanticProfile::RustCheckedV0 => Self::RustCheckedV0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirSafetyOperation {
    None(VirInstructionKind),
    Binary(VirBinaryOperator),
    Unary(VirUnaryOperator),
    Index,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyObligationKind {
    OperationSafety,
}

impl SafetyObligationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationSafety => SAFETY_OBLIGATION_KIND_COMPONENT,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyEvidenceRoute {
    ZeroAxiomGround,
    MvpTheoryGround { format: &'static str },
    GroupedCertificate { foundation: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedSafetyPredicate {
    pub check: VirSafetyCheck,
    pub obligation_kind: SafetyObligationKind,
    pub stable_id_component: &'static str,
    pub proposition: MpkExprTerm,
    pub evidence_route: SafetyEvidenceRoute,
}

/// Derives the one canonical check array for an operation and operand types.
pub fn required_safety_checks(
    profile: SemanticProfile,
    operation: VirSafetyOperation,
    operand_types: &[VirType],
) -> Result<Vec<VirSafetyCheck>, SafetyCheckError> {
    required_safety_checks_for_profile(profile.into(), operation, operand_types, &[])
}

pub(crate) fn required_safety_checks_for_profile(
    profile: CompiledRequiredCheckProfile,
    operation: VirSafetyOperation,
    operand_types: &[VirType],
    actual: &[VirSafetyCheck],
) -> Result<Vec<VirSafetyCheck>, SafetyCheckError> {
    if profile == CompiledRequiredCheckProfile::JavaScalarV0 {
        return required_java_safety_checks(operation, operand_types);
    }
    match operation {
        VirSafetyOperation::None(_) => {
            if !operand_types.is_empty() {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "non-safety operation context must not contain operands",
                ));
            }
            Ok(Vec::new())
        }
        VirSafetyOperation::Binary(operator) => {
            let [lhs, rhs] = operand_types else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "binary safety context requires two operands",
                ));
            };
            validate_binary_operands(profile.encoding_profile(), operator, lhs, rhs)?;
            let signed = matches!(lhs, VirType::Bv { signed: true, .. });
            Ok(match (profile, operator) {
                (CompiledRequiredCheckProfile::RustCheckedV0, VirBinaryOperator::BvAdd) => {
                    vec![integer_check(OverflowOperation::Add, signed)]
                }
                (CompiledRequiredCheckProfile::RustCheckedV0, VirBinaryOperator::BvSub) => {
                    vec![integer_check(OverflowOperation::Sub, signed)]
                }
                (CompiledRequiredCheckProfile::RustCheckedV0, VirBinaryOperator::BvMul) => {
                    vec![integer_check(OverflowOperation::Mul, signed)]
                }
                (CompiledRequiredCheckProfile::CSharpScalarV0, VirBinaryOperator::BvAdd)
                    if has_explicit_overflow_check(actual) =>
                {
                    vec![integer_check(OverflowOperation::Add, signed)]
                }
                (CompiledRequiredCheckProfile::CSharpScalarV0, VirBinaryOperator::BvSub)
                    if has_explicit_overflow_check(actual) =>
                {
                    vec![integer_check(OverflowOperation::Sub, signed)]
                }
                (CompiledRequiredCheckProfile::CSharpScalarV0, VirBinaryOperator::BvMul)
                    if has_explicit_overflow_check(actual) =>
                {
                    vec![integer_check(OverflowOperation::Mul, signed)]
                }
                (_, VirBinaryOperator::BvSdiv) => {
                    let mut checks = vec![VirSafetyCheck::DivisorNonzero {}];
                    if matches!(
                        profile,
                        CompiledRequiredCheckProfile::RustCheckedV0
                            | CompiledRequiredCheckProfile::CSharpScalarV0
                    ) {
                        checks.push(VirSafetyCheck::SignedDivremRepresentable {
                            operation: DivRemOperation::Div,
                        });
                    }
                    checks
                }
                (_, VirBinaryOperator::BvSrem) => {
                    let mut checks = vec![VirSafetyCheck::DivisorNonzero {}];
                    if matches!(
                        profile,
                        CompiledRequiredCheckProfile::RustCheckedV0
                            | CompiledRequiredCheckProfile::CSharpScalarV0
                    ) {
                        checks.push(VirSafetyCheck::SignedDivremRepresentable {
                            operation: DivRemOperation::Rem,
                        });
                    }
                    checks
                }
                (_, VirBinaryOperator::BvUdiv | VirBinaryOperator::BvUrem) => {
                    vec![VirSafetyCheck::DivisorNonzero {}]
                }
                (
                    _,
                    VirBinaryOperator::BvShl
                    | VirBinaryOperator::BvAshr
                    | VirBinaryOperator::BvLshr,
                ) => {
                    let rhs_signed = matches!(rhs, VirType::Bv { signed: true, .. });
                    let mut checks = Vec::new();
                    if rhs_signed {
                        checks.push(VirSafetyCheck::ShiftCountNonnegative {});
                    }
                    if profile == CompiledRequiredCheckProfile::RustCheckedV0 {
                        checks.push(VirSafetyCheck::ShiftCountLessThanWidth {});
                    }
                    if profile == CompiledRequiredCheckProfile::CSharpScalarV0 {
                        checks.clear();
                    }
                    checks
                }
                _ => Vec::new(),
            })
        }
        VirSafetyOperation::Unary(operator) => {
            let [operand] = operand_types else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "unary safety context requires one operand",
                ));
            };
            validate_unary_operand(profile.encoding_profile(), operator, operand)?;
            if operator == VirUnaryOperator::BvNeg
                && (profile == CompiledRequiredCheckProfile::RustCheckedV0
                    || (profile == CompiledRequiredCheckProfile::CSharpScalarV0
                        && has_explicit_overflow_check(actual)))
            {
                Ok(vec![integer_check(OverflowOperation::Neg, true)])
            } else {
                Ok(Vec::new())
            }
        }
        VirSafetyOperation::Index => {
            let [base, index] = operand_types else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "index safety context requires array and index operands",
                ));
            };
            if !matches!(base, VirType::Array { .. }) || !matches!(index, VirType::Bv { .. }) {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "index safety context requires an array and a bitvector index",
                ));
            }
            if profile == CompiledRequiredCheckProfile::CSharpScalarV0 {
                return Err(invalid(
                    "VIR_CSHARP_INDEX_TYPE",
                    "C# scalar profile does not admit Index",
                ));
            }
            if profile == CompiledRequiredCheckProfile::RustCheckedV0
                && matches!(index, VirType::Bv { signed: true, .. })
            {
                return Err(invalid(
                    "VIR_RUST_INDEX_TYPE",
                    "Rust Index requires an unsigned pointer-width index",
                ));
            }
            Ok(vec![VirSafetyCheck::IndexInBounds {}])
        }
    }
}

// Java owns this finite rule independently of C# checked intent and Go shift
// checks. Artifact validation additionally proves the exact linked shift mask.
fn required_java_safety_checks(
    operation: VirSafetyOperation,
    operands: &[VirType],
) -> Result<Vec<VirSafetyCheck>, SafetyCheckError> {
    use crate::java_profile::{is_integer, is_scalar};
    use crate::vir::BitVectorWidth::{Bits32, Bits64};
    use VirBinaryOperator as Op;
    let valid = match (operation, operands) {
        (VirSafetyOperation::None(kind), []) => matches!(
            kind,
            VirInstructionKind::Const
                | VirInstructionKind::Copy
                | VirInstructionKind::Convert
                | VirInstructionKind::CallStatic
        ),
        (VirSafetyOperation::Unary(VirUnaryOperator::Not), [ty]) => matches!(ty, VirType::Bool {}),
        (VirSafetyOperation::Unary(VirUnaryOperator::BvNeg | VirUnaryOperator::BvNot), [ty]) => {
            is_integer(ty)
        }
        (VirSafetyOperation::Binary(Op::Eq | Op::NotEq), [left, right]) => {
            left == right && is_scalar(left)
        }
        (
            VirSafetyOperation::Binary(
                Op::BvAdd
                | Op::BvSub
                | Op::BvMul
                | Op::BvSdiv
                | Op::BvSrem
                | Op::BvAnd
                | Op::BvOr
                | Op::BvXor
                | Op::SignedLt
                | Op::SignedLe
                | Op::SignedGt
                | Op::SignedGe,
            ),
            [left, right],
        ) => left == right && is_integer(left),
        (VirSafetyOperation::Binary(Op::BvShl | Op::BvAshr), [left, right]) => {
            is_integer(left)
                && matches!(
                    right,
                    VirType::Bv {
                        width: Bits32,
                        signed: true
                    }
                )
        }
        (VirSafetyOperation::Binary(Op::BvLshr), [left, right]) => {
            matches!(
                left,
                VirType::Bv {
                    width: Bits32 | Bits64,
                    signed: false
                }
            ) && matches!(
                right,
                VirType::Bv {
                    width: Bits32,
                    signed: true
                }
            )
        }
        _ => false,
    };
    if !valid {
        return Err(invalid(
            "VIR_JAVA_OPERATION",
            "operation or operands are outside the Java scalar profile",
        ));
    }
    Ok(
        if matches!(
            operation,
            VirSafetyOperation::Binary(Op::BvSdiv | Op::BvSrem)
        ) {
            vec![VirSafetyCheck::DivisorNonzero {}]
        } else {
            Vec::new()
        },
    )
}

/// Validates exact kind, order, operation, and signedness metadata.
pub fn validate_safety_check_sequence(
    actual: &[VirSafetyCheck],
    expected: &[VirSafetyCheck],
) -> Result<(), SafetyCheckError> {
    for (index, check) in actual.iter().enumerate() {
        if actual[..index].contains(check) {
            return Err(invalid(
                "VIR_SAFETY_CHECK_DUPLICATE",
                "duplicate safety check",
            ));
        }
    }
    if actual == expected {
        return Ok(());
    }
    if actual.len() < expected.len() {
        return Err(invalid("VIR_SAFETY_CHECK_MISSING", "missing safety check"));
    }
    if actual.len() > expected.len() {
        return Err(invalid("VIR_SAFETY_CHECK_EXTRA", "extra safety check"));
    }

    let actual_kinds = actual.iter().map(VirSafetyCheck::kind).collect::<Vec<_>>();
    let expected_kinds = expected
        .iter()
        .map(VirSafetyCheck::kind)
        .collect::<Vec<_>>();
    let mut sorted_actual = actual_kinds.clone();
    let mut sorted_expected = expected_kinds.clone();
    sorted_actual.sort_by_key(|kind| safety_kind_order(*kind));
    sorted_expected.sort_by_key(|kind| safety_kind_order(*kind));
    if sorted_actual == sorted_expected && actual_kinds != expected_kinds {
        return Err(invalid("VIR_SAFETY_CHECK_ORDER", "safety checks reordered"));
    }

    for (actual, expected) in actual.iter().zip(expected) {
        match (actual, expected) {
            (
                VirSafetyCheck::IntegerNoOverflow {
                    operation: actual_operation,
                    signed: actual_signed,
                },
                VirSafetyCheck::IntegerNoOverflow {
                    operation: expected_operation,
                    signed: expected_signed,
                },
            ) => {
                if actual_operation != expected_operation {
                    return Err(invalid(
                        "VIR_SAFETY_CHECK_OPERATION",
                        "overflow-check operation mismatch",
                    ));
                }
                if actual_signed != expected_signed {
                    return Err(invalid(
                        "VIR_SAFETY_CHECK_SIGNEDNESS",
                        "overflow-check signedness mismatch",
                    ));
                }
            }
            (
                VirSafetyCheck::SignedDivremRepresentable {
                    operation: actual_operation,
                },
                VirSafetyCheck::SignedDivremRepresentable {
                    operation: expected_operation,
                },
            ) if actual_operation != expected_operation => {
                return Err(invalid(
                    "VIR_SAFETY_CHECK_OPERATION",
                    "division/remainder check operation mismatch",
                ));
            }
            _ => {}
        }
    }
    Err(invalid(
        "VIR_SAFETY_CHECK_EXTRA",
        "safety-check kind does not match the required set",
    ))
}

/// Regenerates all safety propositions for one validated VIR instruction.
pub fn encode_instruction_safety(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
) -> Result<Vec<EncodedSafetyPredicate>, SafetyCheckError> {
    encode_instruction_safety_for_profile(context, instruction, context.profile().into())
}

pub(crate) fn encode_instruction_safety_for_profile(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
    profile: CompiledRequiredCheckProfile,
) -> Result<Vec<EncodedSafetyPredicate>, SafetyCheckError> {
    let actual = instruction_checks(instruction);
    let (operation, operand_types) = safety_operation_context(context, instruction)?;
    let expected = required_safety_checks_for_profile(profile, operation, &operand_types, actual)?;
    validate_safety_check_sequence(actual, &expected)?;
    validate_result_and_profile(context, instruction, &operand_types, profile)?;

    actual
        .iter()
        .map(|check| encode_safety_predicate(context, instruction, check, profile))
        .collect()
}

fn safety_operation_context(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
) -> Result<(VirSafetyOperation, Vec<VirType>), SafetyCheckError> {
    match instruction {
        VirInstruction::BinOp { op, lhs, rhs, .. } => Ok((
            VirSafetyOperation::Binary(*op),
            vec![context.value_type(lhs)?, context.value_type(rhs)?],
        )),
        VirInstruction::UnaryOp { op, value, .. } => Ok((
            VirSafetyOperation::Unary(*op),
            vec![context.value_type(value)?],
        )),
        VirInstruction::Index { base, index, .. } => Ok((
            VirSafetyOperation::Index,
            vec![context.value_type(base)?, context.value_type(index)?],
        )),
        other => Ok((VirSafetyOperation::None(other.kind()), Vec::new())),
    }
}

// Keep the result-type checks visibly separate from operation selection: this
// ordering is part of the reviewed fail-closed validation precedence.
#[allow(clippy::collapsible_match)]
fn validate_result_and_profile(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
    operand_types: &[VirType],
    profile: CompiledRequiredCheckProfile,
) -> Result<(), SafetyCheckError> {
    match instruction {
        VirInstruction::BinOp {
            op:
                VirBinaryOperator::BvAdd
                | VirBinaryOperator::BvSub
                | VirBinaryOperator::BvMul
                | VirBinaryOperator::BvSdiv
                | VirBinaryOperator::BvSrem
                | VirBinaryOperator::BvUdiv
                | VirBinaryOperator::BvUrem
                | VirBinaryOperator::BvShl
                | VirBinaryOperator::BvAshr
                | VirBinaryOperator::BvLshr,
            r#type,
            ..
        } => {
            if operand_types.first() != Some(r#type) {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "safety operation result type does not match its left operand",
                ));
            }
        }
        VirInstruction::UnaryOp {
            op: VirUnaryOperator::BvNeg,
            r#type,
            ..
        } => {
            if operand_types.first() != Some(r#type) {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "negation result type does not match its operand",
                ));
            }
        }
        VirInstruction::Index { r#type, .. } => {
            let Some(VirType::Array { element, .. }) = operand_types.first() else {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Index base is not an array",
                ));
            };
            if element.as_ref() != r#type {
                return Err(invalid(
                    "VIR_INSTRUCTION_TYPE",
                    "Index result type does not match the array element",
                ));
            }
            if profile == CompiledRequiredCheckProfile::RustCheckedV0 {
                let Some(VirType::Bv { width, signed }) = operand_types.get(1) else {
                    return Err(invalid("VIR_INSTRUCTION_TYPE", "Index is not a bitvector"));
                };
                if *signed || width.bits() != context.parameters().pointer_width().bits() {
                    return Err(invalid(
                        "VIR_RUST_INDEX_TYPE",
                        "Rust Index requires an unsigned pointer-width index",
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn encode_safety_predicate(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
    check: &VirSafetyCheck,
    profile: CompiledRequiredCheckProfile,
) -> Result<EncodedSafetyPredicate, SafetyCheckError> {
    let proposition = match (check, instruction) {
        (
            VirSafetyCheck::IntegerNoOverflow { operation, signed },
            VirInstruction::BinOp { op, lhs, rhs, .. },
        ) => encode_binary_overflow(context, *operation, *signed, *op, lhs, rhs)?,
        (
            VirSafetyCheck::IntegerNoOverflow {
                operation: OverflowOperation::Neg,
                signed: true,
            },
            VirInstruction::UnaryOp {
                op: VirUnaryOperator::BvNeg,
                value,
                ..
            },
        ) => {
            let ty = context.value_type(value)?;
            let shape = bitvector_shape("negation", &ty)?;
            MpkExprTerm::apply(
                bitvec_function(shape.width, "sgt"),
                [encode_vir_value(context, value)?, signed_min(shape)],
            )
        }
        (VirSafetyCheck::DivisorNonzero {}, VirInstruction::BinOp { rhs, .. }) => {
            let ty = context.value_type(rhs)?;
            let shape = bitvector_shape("division divisor", &ty)?;
            MpkExprTerm::apply(
                bitvec_function(shape.width, "ugt"),
                [encode_vir_value(context, rhs)?, zero(shape)],
            )
        }
        (
            VirSafetyCheck::SignedDivremRepresentable { .. },
            VirInstruction::BinOp { lhs, rhs, .. },
        ) => {
            let ty = context.value_type(lhs)?;
            let shape = bitvector_shape("signed division", &ty)?;
            bool_or(
                not_equal(encode_vir_value(context, lhs)?, signed_min(shape)),
                not_equal(encode_vir_value(context, rhs)?, minus_one(shape)),
            )
        }
        (VirSafetyCheck::ShiftCountNonnegative {}, VirInstruction::BinOp { rhs, .. }) => {
            let ty = context.value_type(rhs)?;
            let shape = bitvector_shape("shift count", &ty)?;
            MpkExprTerm::apply(
                bitvec_function(shape.width, "sge"),
                [encode_vir_value(context, rhs)?, zero(shape)],
            )
        }
        (VirSafetyCheck::ShiftCountLessThanWidth {}, VirInstruction::BinOp { lhs, rhs, .. }) => {
            let lhs_shape = bitvector_shape("shift value", &context.value_type(lhs)?)?;
            let rhs_shape = bitvector_shape("shift count", &context.value_type(rhs)?)?;
            MpkExprTerm::apply(
                bitvec_function(rhs_shape.width, "ult"),
                [
                    encode_vir_value(context, rhs)?,
                    bitvector_literal(lhs_shape.width.to_string(), rhs_shape),
                ],
            )
        }
        (VirSafetyCheck::IndexInBounds {}, VirInstruction::Index { base, index, .. }) => {
            encode_index_bounds(context, base, index)?
        }
        _ => {
            return Err(invalid(
                "VIR_SAFETY_CHECK_KIND",
                "safety check does not match its owning instruction",
            ));
        }
    };
    let evidence_route = classify_safety_evidence_for_profile(profile, &proposition);
    Ok(EncodedSafetyPredicate {
        check: check.clone(),
        obligation_kind: SafetyObligationKind::OperationSafety,
        stable_id_component: SAFETY_OBLIGATION_KIND_COMPONENT,
        proposition,
        evidence_route,
    })
}

fn encode_binary_overflow(
    context: &ProgramExprContext,
    operation: OverflowOperation,
    signed: bool,
    operator: VirBinaryOperator,
    lhs: &VirValue,
    rhs: &VirValue,
) -> Result<MpkExprTerm, SafetyCheckError> {
    let expected_operator = match operation {
        OverflowOperation::Add => VirBinaryOperator::BvAdd,
        OverflowOperation::Sub => VirBinaryOperator::BvSub,
        OverflowOperation::Mul => VirBinaryOperator::BvMul,
        OverflowOperation::Neg => {
            return Err(invalid(
                "VIR_SAFETY_CHECK_OPERATION",
                "negation overflow check requires UnaryOp",
            ));
        }
    };
    if operator != expected_operator {
        return Err(invalid(
            "VIR_SAFETY_CHECK_OPERATION",
            "overflow check operation does not match BinOp",
        ));
    }
    let ty = context.value_type(lhs)?;
    let shape = bitvector_shape("overflow operation", &ty)?;
    if shape.signed != signed {
        return Err(invalid(
            "VIR_SAFETY_CHECK_SIGNEDNESS",
            "overflow check signedness does not match operands",
        ));
    }
    let lhs = encode_vir_value(context, lhs)?;
    let rhs = encode_vir_value(context, rhs)?;
    let suffix = match operation {
        OverflowOperation::Add => "add",
        OverflowOperation::Sub => "sub",
        OverflowOperation::Mul => "mul",
        OverflowOperation::Neg => {
            return Err(invalid(
                "VIR_SAFETY_CHECK_OPERATION",
                "negation overflow check requires UnaryOp",
            ));
        }
    };
    let result = MpkExprTerm::apply(
        bitvec_function(shape.width, suffix),
        [lhs.clone(), rhs.clone()],
    );
    if !signed {
        return Ok(match operation {
            OverflowOperation::Add => {
                MpkExprTerm::apply(bitvec_function(shape.width, "uge"), [result, lhs])
            }
            OverflowOperation::Sub => {
                MpkExprTerm::apply(bitvec_function(shape.width, "uge"), [lhs, rhs])
            }
            OverflowOperation::Mul => bool_or(
                equal(rhs.clone(), zero(shape)),
                equal(
                    MpkExprTerm::apply(bitvec_function(shape.width, "udiv"), [result, rhs]),
                    lhs,
                ),
            ),
            OverflowOperation::Neg => {
                return Err(invalid(
                    "VIR_SAFETY_CHECK_OPERATION",
                    "negation overflow check requires UnaryOp",
                ));
            }
        });
    }

    let lhs_negative = MpkExprTerm::apply(
        bitvec_function(shape.width, "slt"),
        [lhs.clone(), zero(shape)],
    );
    let rhs_negative = MpkExprTerm::apply(
        bitvec_function(shape.width, "slt"),
        [rhs.clone(), zero(shape)],
    );
    let result_negative = MpkExprTerm::apply(
        bitvec_function(shape.width, "slt"),
        [result.clone(), zero(shape)],
    );
    Ok(match operation {
        OverflowOperation::Add => bool_not(bool_or(
            bool_and(
                bool_not(lhs_negative.clone()),
                bool_and(bool_not(rhs_negative.clone()), result_negative.clone()),
            ),
            bool_and(
                lhs_negative,
                bool_and(rhs_negative, bool_not(result_negative)),
            ),
        )),
        OverflowOperation::Sub => bool_not(bool_or(
            bool_and(
                bool_not(lhs_negative.clone()),
                bool_and(rhs_negative.clone(), result_negative.clone()),
            ),
            bool_and(
                lhs_negative,
                bool_and(bool_not(rhs_negative), bool_not(result_negative)),
            ),
        )),
        OverflowOperation::Mul => bool_or(
            equal(lhs.clone(), zero(shape)),
            bool_and(
                bool_not(bool_and(
                    equal(lhs.clone(), minus_one(shape)),
                    equal(rhs.clone(), signed_min(shape)),
                )),
                equal(
                    MpkExprTerm::apply(bitvec_function(shape.width, "sdiv"), [result, lhs]),
                    rhs,
                ),
            ),
        ),
        OverflowOperation::Neg => {
            return Err(invalid(
                "VIR_SAFETY_CHECK_OPERATION",
                "negation overflow check requires UnaryOp",
            ));
        }
    })
}

fn encode_index_bounds(
    context: &ProgramExprContext,
    base: &VirValue,
    index: &VirValue,
) -> Result<MpkExprTerm, SafetyCheckError> {
    let VirType::Array { length, .. } = context.value_type(base)? else {
        return Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "Index base is not an array",
        ));
    };
    let shape = bitvector_shape("array index", &context.value_type(index)?)?;
    let index = encode_vir_value(context, index)?;
    let length = u64::from(length.get());
    let upper_is_trivial = if shape.signed {
        u128::from(length) > ((1_u128 << (shape.width - 1)) - 1)
    } else {
        u128::from(length) > ((1_u128 << shape.width) - 1)
    };
    let upper = if upper_is_trivial {
        MpkExprTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        }
    } else {
        MpkExprTerm::apply(
            bitvec_function(shape.width, if shape.signed { "slt" } else { "ult" }),
            [index.clone(), bitvector_literal(length.to_string(), shape)],
        )
    };
    if shape.signed {
        let lower = MpkExprTerm::apply(bitvec_function(shape.width, "sge"), [index, zero(shape)]);
        if upper_is_trivial {
            Ok(lower)
        } else {
            Ok(bool_and(lower, upper))
        }
    } else {
        Ok(upper)
    }
}

pub(crate) fn classify_safety_evidence_for_profile(
    profile: CompiledRequiredCheckProfile,
    proposition: &MpkExprTerm,
) -> SafetyEvidenceRoute {
    if !is_ground(proposition) {
        return SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION,
        };
    }
    if profile == CompiledRequiredCheckProfile::GoFixedV0 {
        return SafetyEvidenceRoute::ZeroAxiomGround;
    }
    if mvp_bitvec_ground_supports(proposition) {
        SafetyEvidenceRoute::MvpTheoryGround {
            format: SAFETY_BITVEC_THEORY_FORMAT,
        }
    } else {
        SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION,
        }
    }
}

fn has_explicit_overflow_check(actual: &[VirSafetyCheck]) -> bool {
    actual
        .iter()
        .any(|check| matches!(check, VirSafetyCheck::IntegerNoOverflow { .. }))
}

fn mvp_bitvec_ground_supports(proposition: &MpkExprTerm) -> bool {
    let MpkExprTerm::Apply { function, args } = proposition else {
        return false;
    };
    let Some((width, operation)) = bitvec_function_parts(function) else {
        return false;
    };
    matches!(
        operation,
        "ult" | "ule" | "ugt" | "uge" | "slt" | "sle" | "sgt" | "sge"
    ) && args.len() == 2
        && args
            .iter()
            .all(|argument| mvp_bitvec_value_supports(argument, width))
}

fn mvp_bitvec_value_supports(term: &MpkExprTerm, expected_width: u32) -> bool {
    match term {
        MpkExprTerm::BitVecLiteral { width, .. } => *width == expected_width,
        MpkExprTerm::Apply { function, args } => {
            let Some((width, operation)) = bitvec_function_parts(function) else {
                return false;
            };
            if width != expected_width {
                return false;
            }
            match operation {
                "not" | "neg" => {
                    args.len() == 1
                        && args
                            .iter()
                            .all(|argument| mvp_bitvec_value_supports(argument, width))
                }
                "and" | "or" | "xor" | "add" | "sub" | "mul" | "shl" | "lshr" | "ashr" => {
                    args.len() == 2
                        && args
                            .iter()
                            .all(|argument| mvp_bitvec_value_supports(argument, width))
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn bitvec_function_parts(function: &str) -> Option<(u32, &str)> {
    let suffix = function.strip_prefix("Std.BitVec.BV")?;
    let (width, operation) = suffix.split_once('.')?;
    let width = width.parse().ok()?;
    matches!(width, 8 | 16 | 32 | 64).then_some((width, operation))
}

fn validate_binary_operands(
    profile: SemanticProfile,
    operator: VirBinaryOperator,
    lhs: &VirType,
    rhs: &VirType,
) -> Result<(), SafetyCheckError> {
    use VirBinaryOperator as Op;
    match operator {
        Op::Eq | Op::NotEq if lhs == rhs => {
            if profile == SemanticProfile::RustCheckedV0
                && !matches!(lhs, VirType::Bool {} | VirType::Bv { .. })
            {
                return Err(invalid(
                    "VIR_PROFILE_OPERATION",
                    "Rust program equality accepts only bool and bitvector values",
                ));
            }
            Ok(())
        }
        Op::BvShl | Op::BvAshr | Op::BvLshr => {
            let lhs_shape = bitvector_shape("shift LHS", lhs)?;
            bitvector_shape("shift RHS", rhs)?;
            if (operator == Op::BvAshr && !lhs_shape.signed)
                || (operator == Op::BvLshr && lhs_shape.signed)
            {
                Err(invalid("VIR_INSTRUCTION_TYPE", "shift signedness mismatch"))
            } else {
                Ok(())
            }
        }
        Op::SignedLt | Op::SignedLe | Op::SignedGt | Op::SignedGe => {
            require_matching_bv(lhs, rhs, Some(true))
        }
        Op::UnsignedLt | Op::UnsignedLe | Op::UnsignedGt | Op::UnsignedGe => {
            require_matching_bv(lhs, rhs, Some(false))
        }
        Op::BvSdiv | Op::BvSrem => require_matching_bv(lhs, rhs, Some(true)),
        Op::BvUdiv | Op::BvUrem => require_matching_bv(lhs, rhs, Some(false)),
        Op::BvAdd | Op::BvSub | Op::BvMul | Op::BvAnd | Op::BvOr | Op::BvXor => {
            require_matching_bv(lhs, rhs, None)
        }
        _ => Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "binary safety context has invalid operand types",
        )),
    }
}

fn validate_unary_operand(
    profile: SemanticProfile,
    operator: VirUnaryOperator,
    operand: &VirType,
) -> Result<(), SafetyCheckError> {
    match operator {
        VirUnaryOperator::Not if matches!(operand, VirType::Bool {}) => Ok(()),
        VirUnaryOperator::BvNot if matches!(operand, VirType::Bv { .. }) => Ok(()),
        VirUnaryOperator::BvNeg if matches!(operand, VirType::Bv { .. }) => {
            if profile == SemanticProfile::RustCheckedV0
                && matches!(operand, VirType::Bv { signed: false, .. })
            {
                Err(invalid(
                    "VIR_PROFILE_OPERATION",
                    "Rust unsigned negation is not accepted",
                ))
            } else {
                Ok(())
            }
        }
        _ => Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "unary safety context has invalid operand type",
        )),
    }
}

fn require_matching_bv(
    lhs: &VirType,
    rhs: &VirType,
    signed: Option<bool>,
) -> Result<(), SafetyCheckError> {
    if lhs != rhs {
        return Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "bitvector safety operands do not have matching types",
        ));
    }
    let shape = bitvector_shape("binary operation", lhs)?;
    if signed.is_some_and(|expected| expected != shape.signed) {
        return Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            "bitvector safety operand signedness mismatch",
        ));
    }
    Ok(())
}

fn instruction_checks(instruction: &VirInstruction) -> &[VirSafetyCheck] {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn integer_check(operation: OverflowOperation, signed: bool) -> VirSafetyCheck {
    VirSafetyCheck::IntegerNoOverflow { operation, signed }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BitVectorShape {
    width: u32,
    signed: bool,
}

fn bitvector_shape(
    operation: &'static str,
    r#type: &VirType,
) -> Result<BitVectorShape, SafetyCheckError> {
    match r#type {
        VirType::Bv { width, signed } => Ok(BitVectorShape {
            width: width.bits(),
            signed: *signed,
        }),
        _ => Err(invalid(
            "VIR_INSTRUCTION_TYPE",
            format!("{operation} requires a bitvector"),
        )),
    }
}

fn bitvec_function(width: u32, suffix: &str) -> String {
    format!("{STD_BITVEC_MODULE}.BV{width}.{suffix}")
}

fn bitvector_literal(value: impl Into<String>, shape: BitVectorShape) -> MpkExprTerm {
    MpkExprTerm::BitVecLiteral {
        value: value.into(),
        width: shape.width,
        signed: shape.signed,
    }
}

fn zero(shape: BitVectorShape) -> MpkExprTerm {
    bitvector_literal("0", shape)
}

fn minus_one(shape: BitVectorShape) -> MpkExprTerm {
    bitvector_literal("-1", shape)
}

fn signed_min(shape: BitVectorShape) -> MpkExprTerm {
    bitvector_literal(format!("-{}", 1_u128 << (shape.width - 1)), shape)
}

fn equal(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
    MpkExprTerm::apply(STD_EQ, [lhs, rhs])
}

fn not_equal(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
    bool_not(equal(lhs, rhs))
}

fn bool_not(value: MpkExprTerm) -> MpkExprTerm {
    MpkExprTerm::apply(STD_BOOL_NOT, [value])
}

fn bool_and(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
    MpkExprTerm::apply(STD_BOOL_AND, [lhs, rhs])
}

fn bool_or(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
    MpkExprTerm::apply(STD_BOOL_OR, [lhs, rhs])
}

fn is_ground(term: &MpkExprTerm) -> bool {
    match term {
        MpkExprTerm::Var { .. }
        | MpkExprTerm::Bound { .. }
        | MpkExprTerm::Result { .. }
        | MpkExprTerm::Forall { .. } => false,
        MpkExprTerm::Constant { .. } | MpkExprTerm::BitVecLiteral { .. } => true,
        MpkExprTerm::Apply { args, .. } => args.iter().all(is_ground),
        MpkExprTerm::Convert { value, .. } => is_ground(value),
    }
}

fn safety_kind_order(kind: crate::vir::VirSafetyCheckKind) -> u8 {
    use crate::vir::VirSafetyCheckKind as Kind;
    match kind {
        Kind::IntegerNoOverflow => 0,
        Kind::DivisorNonzero => 1,
        Kind::SignedDivremRepresentable => 2,
        Kind::ShiftCountNonnegative => 3,
        Kind::ShiftCountLessThanWidth => 4,
        Kind::IndexInBounds => 5,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SafetyCheckError {
    Invalid { code: &'static str, detail: String },
    Expression(ProgramExprEncodeError),
}

impl SafetyCheckError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Invalid { code, .. } => code,
            Self::Expression(_) => "VIR_SAFETY_EXPRESSION",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::Invalid { detail, .. } => detail.clone(),
            Self::Expression(error) => error.to_string(),
        }
    }
}

impl From<ProgramExprEncodeError> for SafetyCheckError {
    fn from(error: ProgramExprEncodeError) -> Self {
        Self::Expression(error)
    }
}

impl fmt::Display for SafetyCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { code, detail } => write!(formatter, "{code}: {detail}"),
            Self::Expression(error) => write!(formatter, "safety expression failed: {error}"),
        }
    }
}

impl std::error::Error for SafetyCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Expression(error) => Some(error),
            Self::Invalid { .. } => None,
        }
    }
}

fn invalid(code: &'static str, detail: impl Into<String>) -> SafetyCheckError {
    SafetyCheckError::Invalid {
        code,
        detail: detail.into(),
    }
}
