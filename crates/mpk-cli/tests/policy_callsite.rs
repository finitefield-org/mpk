use std::path::PathBuf;

use mpk_cli::policy_callsite::{
    analyze_policy_call_site, analyze_policy_call_site_text, PolicyCallSiteAnalysis,
    PolicyCallSiteAnalysisRequest, PolicyCallSiteInvariant, PolicyCallSiteTextRequest,
};
use mpk_cli::policy_evidence::{
    PolicyCallSiteEvidenceLabel, PolicyCallSitePreconditionEvidence,
    PolicyCallSitePreconditionStatus, PolicyContractArtifact, PolicyEvidenceReport,
    PolicyEvidenceTarget, PolicyHelperArtifacts, PolicySourceArtifact, PolicyTrustedEvidence,
};
use mpk_cli::policy_report::render_policy_evidence_markdown;
use serde_json::{json, Value};

const ORDER_POLICY_FUNCTION: &str = "example.com/orderpolicy.ApprovedReserveCents";
const ORDER_CONTRACT_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/order_policy/policy_contract.json"
));

#[test]
fn call_site_precondition_status_labels_are_stable() {
    let cases = [
        (
            PolicyCallSitePreconditionStatus::CheckedByLocalGuard,
            "checked_by_local_guard",
        ),
        (
            PolicyCallSitePreconditionStatus::DeclaredUpstreamInvariant,
            "declared_upstream_invariant",
        ),
        (
            PolicyCallSitePreconditionStatus::NotObserved,
            "not_observed",
        ),
        (
            PolicyCallSitePreconditionStatus::UnsupportedControlFlow,
            "unsupported_control_flow",
        ),
    ];

    for (status, label) in cases {
        assert_eq!(status.as_str(), label);
        assert_eq!(serde_json::to_value(status).unwrap(), json!(label));
    }
}

#[test]
fn order_policy_webapp_reports_requested_cents_local_guard() {
    let analysis = analyze_order_policy_webapp(Vec::new());
    let requested = find_precondition(&analysis, "requestedCents >= 0");

    assert_eq!(
        requested.status,
        PolicyCallSitePreconditionStatus::CheckedByLocalGuard
    );
    assert_eq!(
        requested.evidence_label,
        PolicyCallSiteEvidenceLabel::HelperAnalysis
    );
    assert!(requested.summary.contains("not proof evidence"));
}

#[test]
fn upstream_invariant_requires_explicit_config() {
    let source = r#"
package webapp

func reserve(balanceCents int64, requestedCents int64) {
	_ = orderpolicy.ApprovedReserveCents(balanceCents, requestedCents)
}
"#;
    let without_invariant = analyze_inline(source, &[]);
    assert_eq!(
        find_precondition(&without_invariant, "balanceCents >= 0").status,
        PolicyCallSitePreconditionStatus::NotObserved
    );

    let invariant = PolicyCallSiteInvariant::new(
        "balanceCents >= 0",
        "wallet service contract declares available balances non-negative",
    );
    let with_invariant = analyze_inline(source, &[invariant]);
    let balance = find_precondition(&with_invariant, "balanceCents >= 0");

    assert_eq!(
        balance.status,
        PolicyCallSitePreconditionStatus::DeclaredUpstreamInvariant
    );
    assert!(balance
        .summary
        .contains("caller-declared upstream invariant"));
    assert_eq!(
        find_precondition(&with_invariant, "requestedCents >= 0").status,
        PolicyCallSitePreconditionStatus::NotObserved
    );
}

#[test]
fn missing_checks_report_not_observed() {
    let source = r#"
package webapp

func reserve(balanceCents int64, requestedCents int64) {
	_ = orderpolicy.ApprovedReserveCents(balanceCents, requestedCents)
}
"#;
    let analysis = analyze_inline(source, &[]);

    assert_eq!(
        find_precondition(&analysis, "requestedCents >= 0").status,
        PolicyCallSitePreconditionStatus::NotObserved
    );
}

#[test]
fn loops_and_aliasing_report_unsupported_control_flow() {
    let loop_source = r#"
package webapp

func reserve(balanceCents int64, requestedCents int64) {
	for i := 0; i < 1; i++ {
		_ = orderpolicy.ApprovedReserveCents(balanceCents, requestedCents)
	}
}
"#;
    let loop_analysis = analyze_inline(loop_source, &[]);
    assert_eq!(
        find_precondition(&loop_analysis, "requestedCents >= 0").status,
        PolicyCallSitePreconditionStatus::UnsupportedControlFlow
    );

    let alias_source = r#"
package webapp

type requestBody struct {
	RequestedCents int64
}

func reserve(balanceCents int64, request requestBody) {
	amount := request.RequestedCents
	_ = orderpolicy.ApprovedReserveCents(balanceCents, amount)
}
"#;
    let alias_analysis = analyze_inline(alias_source, &[]);
    assert_eq!(
        find_precondition(&alias_analysis, "requestedCents >= 0").status,
        PolicyCallSitePreconditionStatus::UnsupportedControlFlow
    );
}

#[test]
fn call_site_preconditions_integrate_with_helper_artifacts_json_and_markdown() {
    let analysis = analyze_order_policy_webapp(Vec::new());
    let mut helper_artifacts = PolicyHelperArtifacts::new(
        PolicySourceArtifact::new("examples/order_policy", "source-hash", Vec::new()),
        PolicyContractArtifact::new(
            "examples/order_policy/policy_contract.json",
            "mpk.go.contract.v0",
            "contract-hash",
        ),
    );
    helper_artifacts.call_site_preconditions = analysis.preconditions.clone();

    let report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        PolicyTrustedEvidence::empty(),
        helper_artifacts,
    );
    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert!(first.contains("\"call_site_preconditions\""));
    assert!(first.contains("\"evidence_label\": \"helper_analysis\""));
    assert!(first.contains("\"status\": \"checked_by_local_guard\""));

    let reparsed = PolicyEvidenceReport::from_json(&first).expect("JSON reparses");
    assert_eq!(reparsed, report);

    let markdown = render_policy_evidence_markdown(&report);
    assert!(markdown.contains("- Call-site preconditions (helper analysis):"));
    assert!(
        markdown.contains("`requestedCents >= 0`: `checked_by_local_guard` (`helper_analysis`)")
    );
    assert!(markdown.contains("this is not proof evidence"));
}

#[test]
fn unknown_call_site_precondition_field_rejects() {
    let analysis = analyze_order_policy_webapp(Vec::new());
    let mut helper_artifacts = PolicyHelperArtifacts::new(
        PolicySourceArtifact::new("examples/order_policy", "source-hash", Vec::new()),
        PolicyContractArtifact::new(
            "examples/order_policy/policy_contract.json",
            "mpk.go.contract.v0",
            "contract-hash",
        ),
    );
    helper_artifacts.call_site_preconditions = analysis.preconditions;
    let report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        PolicyTrustedEvidence::empty(),
        helper_artifacts,
    );
    let mut value = serde_json::from_str::<Value>(&report.to_deterministic_json().unwrap())
        .expect("valid JSON");
    value["helper_artifacts"]["call_site_preconditions"][0]["proof_evidence"] =
        json!("not allowed");

    let error = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect_err("unknown nested field rejects");
    assert!(error.to_string().contains("unknown field"));
}

fn analyze_order_policy_webapp(
    upstream_invariants: Vec<PolicyCallSiteInvariant>,
) -> PolicyCallSiteAnalysis {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut request = PolicyCallSiteAnalysisRequest::new(
        repo_root.join("examples/order_policy/webapp/handler.go"),
        repo_root.join("examples/order_policy/policy_contract.json"),
        ORDER_POLICY_FUNCTION,
    );
    request.upstream_invariants = upstream_invariants;
    analyze_policy_call_site(&request).expect("order policy webapp analyzes")
}

fn analyze_inline(
    source: &str,
    upstream_invariants: &[PolicyCallSiteInvariant],
) -> PolicyCallSiteAnalysis {
    analyze_policy_call_site_text(&PolicyCallSiteTextRequest {
        source,
        source_path: Some("inline/webapp/handler.go".to_owned()),
        contract_json: ORDER_CONTRACT_JSON,
        function_id: ORDER_POLICY_FUNCTION,
        upstream_invariants,
    })
    .expect("inline call site analyzes")
}

fn find_precondition<'a>(
    analysis: &'a PolicyCallSiteAnalysis,
    expression: &str,
) -> &'a PolicyCallSitePreconditionEvidence {
    analysis
        .preconditions
        .iter()
        .find(|precondition| precondition.expression == expression)
        .unwrap_or_else(|| panic!("precondition {expression:?} not found in {analysis:#?}"))
}
