use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use mpk_vc::{
    classify_payment_policy_obligations, emit_theorem_obligations, generate_branch_vcs,
    import_gir_json, policy_theory_goal_from_obligation, PaymentPolicyClassificationOutcome,
    PaymentPolicyClassifierPropertyStatus, PaymentPolicyEvidenceLabel,
    PaymentPolicyObligationPattern, PolicyBoolTautology, PolicyTheoryGoalKind,
};

const UPDATE_ENV: &str = "MPK_UPDATE_PAYMENT_POLICY_EXAMPLES";

#[derive(Clone, Copy)]
struct PaymentPolicyExample {
    name: &'static str,
    function_id: &'static str,
    first_theorem: &'static str,
    last_theorem: &'static str,
    expected_patterns: &'static [(PaymentPolicyObligationPattern, usize)],
}

#[test]
fn payment_policy_examples_generate_stable_vc_outputs() {
    for example in payment_policy_examples() {
        let example_dir = payment_policy_dir(example.name);
        let gir_json = fs::read_to_string(example_dir.join("gir.json"))
            .unwrap_or_else(|error| panic!("read {} GIR: {error}", example.name));
        let gir = import_gir_json(&gir_json)
            .unwrap_or_else(|error| panic!("import {} GIR: {error}", example.name));

        let vc_module = generate_branch_vcs(&gir)
            .unwrap_or_else(|error| panic!("generate {} branch VCs: {error}", example.name));
        let skeleton = emit_theorem_obligations(&vc_module)
            .unwrap_or_else(|error| panic!("emit {} theorem skeletons: {error}", example.name));
        let classification = classify_payment_policy_obligations(&vc_module);

        assert_eq!(
            vc_module.source_gir_hash, gir.gir_hash,
            "{} source GIR hash",
            example.name
        );
        assert_eq!(
            vc_module.obligations.len(),
            8,
            "{} obligation count",
            example.name
        );
        assert_eq!(
            skeleton.theorem_declarations.len(),
            8,
            "{} skeleton theorem count",
            example.name
        );
        assert!(
            vc_module
                .obligations
                .iter()
                .all(|obligation| obligation.function_id == example.function_id),
            "{} obligation function ids",
            example.name
        );
        assert_eq!(
            skeleton.theorem_declarations[0].name, example.first_theorem,
            "{} first theorem",
            example.name
        );
        assert_eq!(
            skeleton.theorem_declarations[7].name, example.last_theorem,
            "{} last theorem",
            example.name
        );
        assert_eq!(
            classification.source_gir_hash, gir.gir_hash,
            "{} classification source GIR hash",
            example.name
        );
        assert_eq!(
            classification.obligations.len(),
            8,
            "{} classification count",
            example.name
        );
        assert!(
            classification.obligations.iter().all(|classification| {
                classification.outcome == PaymentPolicyClassificationOutcome::SupportedProperty
                    && classification.evidence_label == PaymentPolicyEvidenceLabel::HelperAnalysis
                    && classification.property_status
                        == PaymentPolicyClassifierPropertyStatus::ProofPending
            }),
            "{} classifications should be helper-only proof-pending outputs: {:#?}",
            example.name,
            classification.obligations
        );
        for (pattern, count) in example.expected_patterns {
            let actual = classification
                .obligations
                .iter()
                .filter(|classification| classification.pattern == Some(*pattern))
                .count();
            assert_eq!(actual, *count, "{} pattern {:?}", example.name, pattern);
        }
        let extracted_goals = vc_module
            .obligations
            .iter()
            .zip(&classification.obligations)
            .map(|(obligation, classification)| {
                policy_theory_goal_from_obligation(obligation, classification).unwrap_or_else(
                    |error| panic!("extract {} theory goal: {error}", obligation.id),
                )
            })
            .collect::<Vec<_>>();
        let linear_goal_count = extracted_goals
            .iter()
            .filter(|goal| {
                matches!(
                    goal,
                    Some(mpk_vc::PolicyTheoryGoal {
                        kind: PolicyTheoryGoalKind::Linear(_),
                        ..
                    })
                )
            })
            .count();
        assert_eq!(linear_goal_count, 6, "{} linear theory goals", example.name);
        let true_or_opaque_count = extracted_goals
            .iter()
            .filter(|goal| {
                matches!(
                    goal,
                    Some(mpk_vc::PolicyTheoryGoal {
                        kind: PolicyTheoryGoalKind::BoolTautology(goal),
                        ..
                    }) if goal.tautology == PolicyBoolTautology::TrueOrOpaque
                )
            })
            .count();
        let opaque_or_true_count = extracted_goals
            .iter()
            .filter(|goal| {
                matches!(
                    goal,
                    Some(mpk_vc::PolicyTheoryGoal {
                        kind: PolicyTheoryGoalKind::BoolTautology(goal),
                        ..
                    }) if goal.tautology == PolicyBoolTautology::OpaqueOrTrue
                )
            })
            .count();
        assert_eq!(
            true_or_opaque_count, 1,
            "{} true-or-opaque branch equality goal",
            example.name
        );
        assert_eq!(
            opaque_or_true_count, 1,
            "{} opaque-or-true branch equality goal",
            example.name
        );
        assert_eq!(
            extracted_goals.iter().filter(|goal| goal.is_none()).count(),
            0,
            "{} supported payment-policy goals should all extract neutrally by PAYALPHA-COV-T05",
            example.name
        );
        let classification_json =
            serde_json::to_string(&classification).expect("classification serializes");
        assert!(classification_json.contains("\"evidence_label\":\"helper_analysis\""));
        assert!(classification_json.contains("\"property_status\":\"proof_pending\""));
        assert!(!classification_json.contains("mpk_verified"));

        assert_fixture(&example_dir.join("vc.json"), &pretty_json(&vc_module));
        assert_fixture(
            &example_dir.join("vc_skeleton.json"),
            &pretty_json(&skeleton),
        );
    }
}

fn payment_policy_examples() -> [PaymentPolicyExample; 5] {
    [
        PaymentPolicyExample {
            name: "reserve",
            function_id: "example.com/payment/reserve.ApprovedReserveCents",
            first_theorem:
                "VC.Obligation.example.com.payment.reserve.ApprovedReserveCents.then.post0",
            last_theorem:
                "VC.Obligation.example.com.payment.reserve.ApprovedReserveCents.else.post3",
            expected_patterns: &[
                (PaymentPolicyObligationPattern::NonNegativeResult, 2),
                (PaymentPolicyObligationPattern::ResultBoundedByInput, 4),
                (
                    PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput,
                    2,
                ),
            ],
        },
        PaymentPolicyExample {
            name: "refund",
            function_id: "example.com/payment/refund.ApprovedRefundCents",
            first_theorem:
                "VC.Obligation.example.com.payment.refund.ApprovedRefundCents.then.post0",
            last_theorem: "VC.Obligation.example.com.payment.refund.ApprovedRefundCents.else.post3",
            expected_patterns: &[
                (PaymentPolicyObligationPattern::NonNegativeResult, 2),
                (
                    PaymentPolicyObligationPattern::RefundBoundedByAvailablePaidAmount,
                    4,
                ),
                (
                    PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput,
                    2,
                ),
            ],
        },
        PaymentPolicyExample {
            name: "discount",
            function_id: "example.com/payment/discount.ApprovedDiscountCents",
            first_theorem:
                "VC.Obligation.example.com.payment.discount.ApprovedDiscountCents.then.post0",
            last_theorem:
                "VC.Obligation.example.com.payment.discount.ApprovedDiscountCents.else.post3",
            expected_patterns: &[
                (PaymentPolicyObligationPattern::NonNegativeResult, 2),
                (PaymentPolicyObligationPattern::FeeOrDiscountBoundedByCap, 4),
                (
                    PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput,
                    2,
                ),
            ],
        },
        PaymentPolicyExample {
            name: "fee",
            function_id: "example.com/payment/fee.AppliedPlatformFeeCents",
            first_theorem:
                "VC.Obligation.example.com.payment.fee.AppliedPlatformFeeCents.then.post0",
            last_theorem:
                "VC.Obligation.example.com.payment.fee.AppliedPlatformFeeCents.else.post3",
            expected_patterns: &[
                (PaymentPolicyObligationPattern::NonNegativeResult, 2),
                (PaymentPolicyObligationPattern::FeeOrDiscountBoundedByCap, 4),
                (
                    PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput,
                    2,
                ),
            ],
        },
        PaymentPolicyExample {
            name: "points",
            function_id: "example.com/payment/points.ApprovedRedemptionPoints",
            first_theorem:
                "VC.Obligation.example.com.payment.points.ApprovedRedemptionPoints.then.post0",
            last_theorem:
                "VC.Obligation.example.com.payment.points.ApprovedRedemptionPoints.else.post3",
            expected_patterns: &[
                (PaymentPolicyObligationPattern::NonNegativeResult, 2),
                (PaymentPolicyObligationPattern::ResultBoundedByInput, 4),
                (
                    PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput,
                    2,
                ),
            ],
        },
    ]
}

fn payment_policy_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/payment_policies")
        .join(name)
        .components()
        .collect()
}

fn pretty_json(value: &impl Serialize) -> String {
    let mut output = serde_json::to_string_pretty(value).expect("serialize fixture JSON");
    output.push('\n');
    output
}

fn assert_fixture(path: &PathBuf, actual: &str) {
    if env::var_os(UPDATE_ENV).is_some() {
        fs::write(path, actual).expect("write updated payment policy fixture");
        return;
    }

    let expected = fs::read_to_string(path).expect("read payment policy fixture");
    assert_eq!(actual, expected, "fixture mismatch for {}", path.display());
}
