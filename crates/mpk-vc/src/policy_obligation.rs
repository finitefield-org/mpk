//! Conservative helper classification for payment-policy VC obligations.
//!
//! This module classifies obvious payment-policy obligation shapes so product
//! evidence can explain them. It does not produce trusted proof evidence.

use serde::{Deserialize, Serialize};

use crate::expr_encode::{MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_OR, STD_EQ};
use crate::vc::{VcModule, VcObligation, VcObligationKind};

pub const PAYMENT_OBLIGATION_CLASSIFICATION_SCHEMA: &str =
    "mpk.vc.payment_obligation_classification.v0";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentPolicyClassificationReport {
    pub schema: String,
    pub source_gir_hash: Option<String>,
    pub obligations: Vec<PaymentPolicyObligationClassification>,
}

pub fn classify_payment_policy_obligations(module: &VcModule) -> PaymentPolicyClassificationReport {
    PaymentPolicyClassificationReport {
        schema: PAYMENT_OBLIGATION_CLASSIFICATION_SCHEMA.to_owned(),
        source_gir_hash: module.source_gir_hash.clone(),
        obligations: module
            .obligations
            .iter()
            .map(classify_payment_policy_obligation)
            .collect(),
    }
}

pub fn classify_payment_policy_obligation(
    obligation: &VcObligation,
) -> PaymentPolicyObligationClassification {
    match classify_obligation_pattern(obligation) {
        Ok(pattern) => PaymentPolicyObligationClassification::supported(obligation, pattern),
        Err(diagnostic) => {
            PaymentPolicyObligationClassification::unsupported(obligation, diagnostic)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentPolicyObligationClassification {
    pub obligation_id: String,
    pub function_id: String,
    pub outcome: PaymentPolicyClassificationOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<PaymentPolicyObligationPattern>,
    pub evidence_label: PaymentPolicyEvidenceLabel,
    pub property_status: PaymentPolicyClassifierPropertyStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<UnsupportedPropertyDiagnostic>,
}

impl PaymentPolicyObligationClassification {
    fn supported(obligation: &VcObligation, pattern: PaymentPolicyObligationPattern) -> Self {
        Self {
            obligation_id: obligation.id.clone(),
            function_id: obligation.function_id.clone(),
            outcome: PaymentPolicyClassificationOutcome::SupportedProperty,
            pattern: Some(pattern),
            evidence_label: PaymentPolicyEvidenceLabel::HelperAnalysis,
            property_status: PaymentPolicyClassifierPropertyStatus::ProofPending,
            diagnostic: None,
        }
    }

    fn unsupported(obligation: &VcObligation, diagnostic: UnsupportedPropertyDiagnostic) -> Self {
        Self {
            obligation_id: obligation.id.clone(),
            function_id: obligation.function_id.clone(),
            outcome: PaymentPolicyClassificationOutcome::UnsupportedProperty,
            pattern: None,
            evidence_label: PaymentPolicyEvidenceLabel::HelperAnalysis,
            property_status: PaymentPolicyClassifierPropertyStatus::Unsupported,
            diagnostic: Some(diagnostic),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentPolicyClassificationOutcome {
    SupportedProperty,
    UnsupportedProperty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentPolicyObligationPattern {
    NonNegativeResult,
    ResultBoundedByInput,
    RefundBoundedByAvailablePaidAmount,
    FeeOrDiscountBoundedByCap,
    SelectedBranchResultEqualsInput,
    IntegerRuntimeSafety,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentPolicyEvidenceLabel {
    HelperAnalysis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentPolicyClassifierPropertyStatus {
    ProofPending,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedPropertyDiagnostic {
    pub code: UnsupportedPropertyCode,
    pub message: String,
}

impl UnsupportedPropertyDiagnostic {
    fn new(code: UnsupportedPropertyCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnsupportedPropertyCode {
    UnsupportedBooleanStructure,
    UnsupportedArithmetic,
    UnsupportedType,
    UnsupportedObligationKind,
    UnsupportedPropertyShape,
}

fn classify_obligation_pattern(
    obligation: &VcObligation,
) -> Result<PaymentPolicyObligationPattern, UnsupportedPropertyDiagnostic> {
    if obligation.kind == VcObligationKind::RuntimeSafety {
        return Ok(PaymentPolicyObligationPattern::IntegerRuntimeSafety);
    }
    if obligation.kind != VcObligationKind::Postcondition {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedObligationKind,
            format!(
                "payment-policy classifier supports postcondition and runtime-safety obligations, got {:?}",
                obligation.kind
            ),
        ));
    }

    classify_conclusion(&obligation.function_id, &obligation.conclusion)
}

fn classify_conclusion(
    function_id: &str,
    conclusion: &MpkExprTerm,
) -> Result<PaymentPolicyObligationPattern, UnsupportedPropertyDiagnostic> {
    let MpkExprTerm::Apply { function, args } = conclusion else {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            "payment-policy classifier expects an applied predicate conclusion",
        ));
    };

    if function == STD_BOOL_OR {
        return classify_branch_equality(args);
    }
    if function == STD_BOOL_AND {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "payment-policy classifier does not support conjunction conclusions",
        ));
    }

    let Some((width, suffix)) = bitvec_function_suffix(function) else {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            format!("unsupported predicate function {function:?}"),
        ));
    };
    if width != 64 {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedType,
            format!("payment-policy classifier supports only BV64 predicates, got BV{width}"),
        ));
    }
    if suffix != "sge" && suffix != "sle" {
        if is_bitvec_arithmetic_suffix(suffix) {
            return Err(UnsupportedPropertyDiagnostic::new(
                UnsupportedPropertyCode::UnsupportedArithmetic,
                format!("payment-policy classifier does not support BV64 arithmetic predicate {suffix:?}"),
            ));
        }
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            format!("unsupported BV64 predicate suffix {suffix:?}"),
        ));
    }
    if args.len() != 2 {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            format!(
                "comparison predicate requires 2 arguments, got {}",
                args.len()
            ),
        ));
    }
    validate_simple_bitvec_term(&args[0])?;
    validate_simple_bitvec_term(&args[1])?;

    if suffix == "sge" && is_named_or_result_term(&args[0]) && is_zero_i64_literal(&args[1]) {
        return Ok(PaymentPolicyObligationPattern::NonNegativeResult);
    }

    if !is_named_or_result_term(&args[0]) || !is_named_or_result_term(&args[1]) {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            "payment-policy bound classifier supports only variable or result operands",
        ));
    }

    let function_lower = function_id.to_ascii_lowercase();
    if function_lower.contains("refund") {
        return Ok(PaymentPolicyObligationPattern::RefundBoundedByAvailablePaidAmount);
    }
    if function_lower.contains("discount") || function_lower.contains("fee") {
        return Ok(PaymentPolicyObligationPattern::FeeOrDiscountBoundedByCap);
    }
    if suffix == "sle" {
        return Ok(PaymentPolicyObligationPattern::ResultBoundedByInput);
    }

    Err(UnsupportedPropertyDiagnostic::new(
        UnsupportedPropertyCode::UnsupportedPropertyShape,
        format!("unsupported payment-policy comparison shape {suffix:?} for {function_id}"),
    ))
}

fn classify_branch_equality(
    args: &[MpkExprTerm],
) -> Result<PaymentPolicyObligationPattern, UnsupportedPropertyDiagnostic> {
    if args.len() != 2 {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            format!(
                "selected-branch equality expects exactly 2 disjuncts, got {}",
                args.len()
            ),
        ));
    }

    let (first_lhs, first_rhs) = equality_operands(&args[0])?;
    let (second_lhs, second_rhs) = equality_operands(&args[1])?;
    if first_lhs != second_lhs {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "selected-branch equality disjuncts must compare the same result expression",
        ));
    }
    if first_rhs == second_rhs {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "selected-branch equality disjuncts must compare against distinct branch inputs",
        ));
    }

    Ok(PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput)
}

fn equality_operands(
    term: &MpkExprTerm,
) -> Result<(&MpkExprTerm, &MpkExprTerm), UnsupportedPropertyDiagnostic> {
    let MpkExprTerm::Apply { function, args } = term else {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "selected-branch disjunct is not an equality",
        ));
    };
    if function != STD_EQ || args.len() != 2 {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "selected-branch disjunct must be a binary Std.Eq",
        ));
    }
    if !is_named_or_result_term(&args[0]) || !is_named_or_result_term(&args[1]) {
        return Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedBooleanStructure,
            "selected-branch equality supports only variable or result operands",
        ));
    }
    Ok((&args[0], &args[1]))
}

fn validate_simple_bitvec_term(term: &MpkExprTerm) -> Result<(), UnsupportedPropertyDiagnostic> {
    match term {
        MpkExprTerm::Var { .. } | MpkExprTerm::Result { .. } => Ok(()),
        MpkExprTerm::BitVecLiteral { width, signed, .. } if *width == 64 && *signed => Ok(()),
        MpkExprTerm::BitVecLiteral { width, signed, .. } => {
            Err(UnsupportedPropertyDiagnostic::new(
                UnsupportedPropertyCode::UnsupportedType,
                format!(
                    "payment-policy classifier supports only signed BV64 literals, got signed={signed} width={width}"
                ),
            ))
        }
        MpkExprTerm::Apply { function, .. } if bitvec_function_suffix(function).is_some() => {
            Err(UnsupportedPropertyDiagnostic::new(
                UnsupportedPropertyCode::UnsupportedArithmetic,
                "payment-policy classifier does not rewrite arithmetic expressions",
            ))
        }
        _ => Err(UnsupportedPropertyDiagnostic::new(
            UnsupportedPropertyCode::UnsupportedPropertyShape,
            "payment-policy classifier supports only variables, results, and signed BV64 literals",
        )),
    }
}

fn bitvec_function_suffix(function: &str) -> Option<(u32, &str)> {
    let rest = function
        .strip_prefix(STD_BITVEC_MODULE)?
        .strip_prefix(".BV")?;
    let (width, suffix) = rest.split_once('.')?;
    let width = width.parse::<u32>().ok()?;
    Some((width, suffix))
}

fn is_bitvec_arithmetic_suffix(suffix: &str) -> bool {
    matches!(
        suffix,
        "add" | "sub" | "mul" | "sdiv" | "udiv" | "srem" | "urem" | "neg"
    )
}

fn is_zero_i64_literal(term: &MpkExprTerm) -> bool {
    matches!(
        term,
        MpkExprTerm::BitVecLiteral {
            value,
            width: 64,
            signed: true
        } if value == "0"
    )
}

fn is_named_or_result_term(term: &MpkExprTerm) -> bool {
    matches!(term, MpkExprTerm::Var { .. } | MpkExprTerm::Result { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obligation(conclusion: MpkExprTerm) -> VcObligation {
        VcObligation {
            id: "example.Policy.post0".to_owned(),
            function_id: "example.Policy".to_owned(),
            kind: VcObligationKind::Postcondition,
            assumptions: Vec::new(),
            conclusion,
        }
    }

    fn apply(function: impl Into<String>, args: Vec<MpkExprTerm>) -> MpkExprTerm {
        MpkExprTerm::apply(function, args)
    }

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn int64(value: &str) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_owned(),
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

    fn eq(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(STD_EQ, vec![lhs, rhs])
    }

    #[test]
    fn classifies_non_negative_and_bound_shapes_as_helper_analysis() {
        let non_negative =
            classify_payment_policy_obligation(&obligation(sge(var("approved"), int64("0"))));
        assert_eq!(
            non_negative.pattern,
            Some(PaymentPolicyObligationPattern::NonNegativeResult)
        );
        assert_eq!(
            non_negative.evidence_label,
            PaymentPolicyEvidenceLabel::HelperAnalysis
        );
        assert_eq!(
            non_negative.property_status,
            PaymentPolicyClassifierPropertyStatus::ProofPending
        );

        let bounded =
            classify_payment_policy_obligation(&obligation(sle(var("approved"), var("balance"))));
        assert_eq!(
            bounded.pattern,
            Some(PaymentPolicyObligationPattern::ResultBoundedByInput)
        );
    }

    #[test]
    fn unsupported_shapes_have_deterministic_codes() {
        let unsupported_boolean = classify_payment_policy_obligation(&obligation(apply(
            STD_BOOL_AND,
            vec![
                MpkExprTerm::bool_literal(true),
                MpkExprTerm::bool_literal(true),
            ],
        )));
        assert_unsupported(&unsupported_boolean);
        assert_eq!(
            unsupported_boolean
                .diagnostic
                .expect("unsupported boolean diagnostic")
                .code,
            UnsupportedPropertyCode::UnsupportedBooleanStructure
        );

        let unsupported_arithmetic = classify_payment_policy_obligation(&obligation(sle(
            apply(
                format!("{STD_BITVEC_MODULE}.BV64.add"),
                vec![var("a"), var("b")],
            ),
            var("cap"),
        )));
        assert_unsupported(&unsupported_arithmetic);
        assert_eq!(
            unsupported_arithmetic
                .diagnostic
                .expect("unsupported arithmetic diagnostic")
                .code,
            UnsupportedPropertyCode::UnsupportedArithmetic
        );

        let unsupported_type = classify_payment_policy_obligation(&obligation(apply(
            format!("{STD_BITVEC_MODULE}.BV32.sle"),
            vec![var("a"), var("cap")],
        )));
        assert_unsupported(&unsupported_type);
        assert_eq!(
            unsupported_type
                .diagnostic
                .expect("unsupported type diagnostic")
                .code,
            UnsupportedPropertyCode::UnsupportedType
        );

        let unsupported_literal_only =
            classify_payment_policy_obligation(&obligation(sge(int64("0"), int64("0"))));
        assert_unsupported(&unsupported_literal_only);

        let unsupported_duplicate_branch = classify_payment_policy_obligation(&obligation(apply(
            STD_BOOL_OR,
            vec![
                eq(var("selected"), var("selected")),
                eq(var("selected"), var("selected")),
            ],
        )));
        assert_unsupported(&unsupported_duplicate_branch);
        assert_eq!(
            unsupported_duplicate_branch
                .diagnostic
                .expect("duplicate branch diagnostic")
                .code,
            UnsupportedPropertyCode::UnsupportedBooleanStructure
        );
    }

    fn assert_unsupported(classification: &PaymentPolicyObligationClassification) {
        assert_eq!(
            classification.outcome,
            PaymentPolicyClassificationOutcome::UnsupportedProperty
        );
        assert_eq!(
            classification.evidence_label,
            PaymentPolicyEvidenceLabel::HelperAnalysis
        );
        assert_eq!(
            classification.property_status,
            PaymentPolicyClassifierPropertyStatus::Unsupported
        );
        assert!(classification.pattern.is_none());
    }
}
