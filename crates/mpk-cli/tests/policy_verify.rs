use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;

use mpk_cli::policy_evidence::{
    PolicyEvidenceReport, PolicyPropertyEvidenceRef, PolicyPropertyEvidenceStatus,
};
use serde::Deserialize;

const RESERVE_FUNCTION: &str = "example.com/payment/reserve.ApprovedReserveCents";
const RESERVE_TARGET: &str = "examples/payment_policies/reserve";
const RESERVE_CONTRACT: &str = "examples/payment_policies/reserve/policy_contract.json";
const RESERVE_TRACKED_EVIDENCE: &str = "examples/payment_policies/reserve/evidence_alpha.json";
const PAYMENT_POLICY_MANIFEST: &str = "examples/payment_policies/manifest.json";
static BUILD_GO2GIR: Once = Once::new();

#[derive(Debug, Deserialize)]
struct PaymentPolicyManifest {
    positive: Vec<PaymentPolicyManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct PaymentPolicyManifestEntry {
    name: String,
    path: String,
    function_id: String,
    contract: String,
}

#[derive(Debug)]
struct PaymentPolicyCorpusCase {
    name: String,
    path: String,
    function_id: String,
    contract: String,
    expected_bound_pattern: &'static str,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ensure_go2gir() -> PathBuf {
    let repo = repo_root();
    let go2gir = repo.join("target/debug/go2gir");
    BUILD_GO2GIR.call_once(|| {
        let output = Command::new("go")
            .current_dir(repo.join("go-tools/go2gir"))
            .args(["build", "-o", "../../target/debug/go2gir", "."])
            .output()
            .expect("go build runs");
        assert!(
            output.status.success(),
            "go2gir build failed\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    });
    go2gir
}

fn run_mpk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("mpk command runs")
}

fn temp_artifact_paths(name: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("mpk-{name}-{}", std::process::id()));
    let json = base.with_extension("json");
    let md = base.with_extension("md");
    let _ = fs::remove_file(&json);
    let _ = fs::remove_file(&md);
    (json, md)
}

fn unsupported_property_fixture() -> (PathBuf, PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("mpk-unsupported-property-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp fixture directory is created");
    fs::write(
        dir.join("go.mod"),
        "module example.com/payment/unsupportedprop\n\ngo 1.23\n",
    )
    .expect("go.mod is written");
    fs::write(
        dir.join("policy.go"),
        r#"package unsupportedprop

func ApprovedPositiveCents(balanceCents int64) int64 {
	if balanceCents > 0 {
		return balanceCents
	}
	return 0
}
"#,
    )
    .expect("policy.go is written");
    let contract = dir.join("policy_contract.json");
    fs::write(
        &contract,
        r#"{
  "schema": "mpk.go.contract.v0",
  "function": "unsupportedprop.ApprovedPositiveCents",
  "requires": [
    {
      "op": "signed_ge",
      "lhs": { "var": "balanceCents" },
      "rhs": { "int": { "value": "0", "width": 64, "signed": true } }
    }
  ],
  "ensures": [
    {
      "op": "signed_gt",
      "lhs": { "result": 0 },
      "rhs": { "int": { "value": "0", "width": 64, "signed": true } }
    }
  ]
}
"#,
    )
    .expect("contract is written");
    (
        dir,
        contract,
        "example.com/payment/unsupportedprop.ApprovedPositiveCents".to_owned(),
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn read_evidence(path: &Path) -> PolicyEvidenceReport {
    let json = fs::read_to_string(path).expect("evidence JSON is readable");
    PolicyEvidenceReport::from_json(&json).expect("evidence JSON parses")
}

fn positive_payment_policy_corpus_cases() -> Vec<PaymentPolicyCorpusCase> {
    let manifest_path = repo_root().join(PAYMENT_POLICY_MANIFEST);
    let manifest_json = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));
    let manifest = serde_json::from_str::<PaymentPolicyManifest>(&manifest_json)
        .expect("payment policy manifest parses");
    let mut cases = manifest
        .positive
        .into_iter()
        .map(|entry| PaymentPolicyCorpusCase {
            expected_bound_pattern: expected_bound_pattern(&entry.name),
            name: entry.name,
            path: entry.path,
            function_id: entry.function_id,
            contract: entry.contract,
        })
        .collect::<Vec<_>>();
    cases.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    assert_eq!(
        cases
            .iter()
            .map(|case| case.name.as_str())
            .collect::<Vec<_>>(),
        ["discount", "fee", "points", "refund", "reserve"],
        "positive payment-policy corpus should cover the known five examples"
    );
    cases
}

fn expected_bound_pattern(name: &str) -> &'static str {
    match name {
        "reserve" | "points" => "result_bounded_by_input",
        "refund" => "refund_bounded_by_available_paid_amount",
        "discount" | "fee" => "fee_or_discount_bounded_by_cap",
        other => panic!("unexpected positive payment-policy example {other}"),
    }
}

fn run_policy_verify_for_case(
    case: &PaymentPolicyCorpusCase,
    artifact_name: &str,
    go2gir: &Path,
) -> (Output, PathBuf, PathBuf) {
    let (evidence_json, evidence_md) = temp_artifact_paths(artifact_name);
    let target = format!("examples/payment_policies/{}", case.path);
    let contract = format!("examples/payment_policies/{}", case.contract);
    let output = run_mpk(&[
        "policy",
        "verify",
        &target,
        "--function",
        &case.function_id,
        "--contract",
        &contract,
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        evidence_json.to_str().expect("temp JSON path is UTF-8"),
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
        "--go2gir",
        go2gir.to_str().expect("go2gir path is UTF-8"),
        "--strict",
    ]);
    (output, evidence_json, evidence_md)
}

fn count_status(report: &PolicyEvidenceReport, status: PolicyPropertyEvidenceStatus) -> usize {
    report
        .properties
        .iter()
        .filter(|property| property.status == status)
        .count()
}

fn assert_reserve_report_fully_verified(report: &PolicyEvidenceReport) {
    assert_payment_policy_report_fully_verified(
        report,
        "reserve",
        RESERVE_FUNCTION,
        "result_bounded_by_input",
    );
}

fn assert_payment_policy_report_fully_verified(
    report: &PolicyEvidenceReport,
    example_name: &str,
    function_id: &str,
    expected_bound_pattern: &str,
) {
    assert_eq!(
        report.target.package_path,
        format!("example.com/payment/{example_name}")
    );
    assert_eq!(report.target.function_id, function_id);
    assert_eq!(report.strategy_profile, "payment-policy-alpha");
    assert_eq!(report.checker_profile, "mvp-strict");
    assert_eq!(
        count_status(report, PolicyPropertyEvidenceStatus::MpkVerified),
        8
    );
    assert_eq!(
        count_status(report, PolicyPropertyEvidenceStatus::ProofPending),
        0
    );
    assert_eq!(
        count_status(report, PolicyPropertyEvidenceStatus::Unsupported),
        0
    );
    assert!(report.trusted_evidence.certificates.is_empty());
    assert_eq!(report.trusted_evidence.theory_certificates.len(), 8);
    assert_pattern_closed_count(report, "non_negative_result", 2);
    assert_pattern_closed_count(report, expected_bound_pattern, 4);
    assert_pattern_closed_count(report, "selected_branch_result_equals_input", 2);

    let mut unique_hashes = std::collections::BTreeSet::new();
    let mut theory_by_id = std::collections::BTreeMap::new();
    for theory in &report.trusted_evidence.theory_certificates {
        assert_eq!(theory.theory_certificate_hash.len(), 64);
        assert_eq!(theory.checker_profile, "mvp-strict");
        assert_eq!(theory.checked_obligations.len(), 1);
        assert!(unique_hashes.insert(theory.theory_certificate_hash.clone()));
        assert!(
            theory_by_id.insert(theory.id.as_str(), theory).is_none(),
            "duplicate theory certificate id {}",
            theory.id
        );
    }
    assert_eq!(
        unique_hashes.len(),
        8,
        "payload-bound theory certificates should differ by concrete VC payload"
    );

    for index in 1..=6 {
        let id = format!("theory:policy-linarith-{index:04}");
        let theory = theory_by_id
            .get(id.as_str())
            .unwrap_or_else(|| panic!("missing linarith certificate {id}"));
        assert_eq!(theory.theory, "linarith");
        assert_eq!(theory.format, "mpk.linarith.v0");
    }
    for index in 1..=2 {
        let id = format!("theory:policy-bool-tautology-{index:04}");
        let theory = theory_by_id
            .get(id.as_str())
            .unwrap_or_else(|| panic!("missing bool certificate {id}"));
        assert_eq!(theory.theory, "bool_tautology");
        assert_eq!(theory.format, "mpk.bool-normalize.v0");
    }

    assert_eq!(report.properties.len(), 8);
    for property in &report.properties {
        assert_eq!(property.status, PolicyPropertyEvidenceStatus::MpkVerified);
        let [PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id,
            obligation_id,
        }] = property.evidence.as_slice()
        else {
            panic!("verified property should reference exactly one checked theory certificate");
        };
        assert_eq!(obligation_id, &property.id);
        let theory = theory_by_id
            .get(theory_certificate_id.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "property {} references missing theory certificate",
                    property.id
                )
            });
        assert_eq!(
            theory.checked_obligations[0].as_str(),
            obligation_id.as_str()
        );
        match theory.theory.as_str() {
            "linarith" => assert!(property
                .notes
                .iter()
                .any(|note| note.contains("checked linarith evidence"))),
            "bool_tautology" => assert!(property
                .notes
                .iter()
                .any(|note| note.contains("checked bool tautology evidence"))),
            other => panic!("unexpected theory kind {other}"),
        }
    }
}

fn assert_pattern_closed_count(
    report: &PolicyEvidenceReport,
    pattern: &str,
    expected_count: usize,
) {
    let pattern_description = format!("Payment policy obligation classified as {pattern}.");
    let properties = report
        .properties
        .iter()
        .filter(|property| property.description == pattern_description)
        .collect::<Vec<_>>();
    assert_eq!(
        properties.len(),
        expected_count,
        "pattern {pattern} count for {}",
        report.target.function_id
    );
    assert!(
        properties.iter().all(|property| matches!(
            property.evidence.as_slice(),
            [PolicyPropertyEvidenceRef::CheckedTheoryCertificate { .. }]
        )),
        "pattern {pattern} should close with checked theory evidence for {}",
        report.target.function_id
    );
}

#[test]
fn malformed_evidence_json_rejects_without_panic() {
    let error = PolicyEvidenceReport::from_json("{").expect_err("malformed JSON rejects");

    assert!(!error.to_string().is_empty());
}

#[test]
fn policy_verify_reserve_writes_evidence_and_markdown() {
    let go2gir = ensure_go2gir();
    let (evidence_json, evidence_md) = temp_artifact_paths("reserve-verify");
    let tracked_before =
        fs::read_to_string(repo_root().join(RESERVE_TRACKED_EVIDENCE)).expect("fixture readable");

    let output = run_mpk(&[
        "policy",
        "verify",
        RESERVE_TARGET,
        "--function",
        RESERVE_FUNCTION,
        "--contract",
        RESERVE_CONTRACT,
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        evidence_json.to_str().expect("temp JSON path is UTF-8"),
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
        "--go2gir",
        go2gir.to_str().expect("go2gir path is UTF-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        format!(
            "ok policy verify status=verified verified=8 proof_pending=0 unsupported=0 evidence_json={} evidence_md={}\n",
            evidence_json.display(),
            evidence_md.display()
        )
    );
    assert!(output.stderr.is_empty());

    let report = read_evidence(&evidence_json);
    assert_reserve_report_fully_verified(&report);

    let markdown = fs::read_to_string(&evidence_md).expect("Markdown is readable");
    assert!(markdown.contains("## Verified Properties"));
    assert!(markdown.contains("## Proof-Pending Properties"));
    let tracked_after =
        fs::read_to_string(repo_root().join(RESERVE_TRACKED_EVIDENCE)).expect("fixture readable");
    assert_eq!(tracked_after, tracked_before);

    let _ = fs::remove_file(evidence_json);
    let _ = fs::remove_file(evidence_md);
}

#[test]
fn policy_verify_positive_payment_corpus_has_expected_counts() {
    let go2gir = ensure_go2gir();

    for case in positive_payment_policy_corpus_cases() {
        let (output, evidence_json, evidence_md) =
            run_policy_verify_for_case(&case, &format!("{}-strict-corpus", case.name), &go2gir);

        assert!(
            output.status.success(),
            "{} stderr: {}",
            case.name,
            stderr(&output)
        );
        assert_eq!(
            stdout(&output),
            format!(
                "ok policy verify status=verified verified=8 proof_pending=0 unsupported=0 evidence_json={} evidence_md={}\n",
                evidence_json.display(),
                evidence_md.display()
            ),
            "{} stdout",
            case.name
        );
        assert!(output.stderr.is_empty(), "{} stderr", case.name);

        let report = read_evidence(&evidence_json);
        assert_payment_policy_report_fully_verified(
            &report,
            &case.name,
            &case.function_id,
            case.expected_bound_pattern,
        );
        if case.name != "reserve" {
            assert!(
                report.properties.iter().any(|property| matches!(
                    property.evidence.as_slice(),
                    [PolicyPropertyEvidenceRef::CheckedTheoryCertificate { .. }]
                )),
                "{} should prove this is not reserve-only coverage",
                case.name
            );
        }

        let _ = fs::remove_file(evidence_json);
        let _ = fs::remove_file(evidence_md);
    }
}

#[test]
fn policy_verify_known_non_strict_checker_profiles_keep_supported_properties_pending() {
    let go2gir = ensure_go2gir();

    for checker_profile in ["core-bootstrap", "mvp-structural"] {
        let (evidence_json, evidence_md) =
            temp_artifact_paths(&format!("reserve-{checker_profile}"));
        let output = run_mpk(&[
            "policy",
            "verify",
            RESERVE_TARGET,
            "--function",
            RESERVE_FUNCTION,
            "--contract",
            RESERVE_CONTRACT,
            "--strategy-profile",
            "payment-policy-alpha",
            "--checker-profile",
            checker_profile,
            "--evidence-json",
            evidence_json.to_str().expect("temp JSON path is UTF-8"),
            "--evidence-md",
            evidence_md.to_str().expect("temp Markdown path is UTF-8"),
            "--go2gir",
            go2gir.to_str().expect("go2gir path is UTF-8"),
        ]);

        assert!(
            output.status.success(),
            "{checker_profile} stderr: {}",
            stderr(&output)
        );
        assert_eq!(
            stdout(&output),
            format!(
                "ok policy verify status=proof_pending verified=0 proof_pending=8 unsupported=0 evidence_json={} evidence_md={}\n",
                evidence_json.display(),
                evidence_md.display()
            )
        );
        assert!(output.stderr.is_empty());

        let report = read_evidence(&evidence_json);
        assert_eq!(report.checker_profile, checker_profile);
        assert!(report.trusted_evidence.theory_certificates.is_empty());
        assert_eq!(
            count_status(&report, PolicyPropertyEvidenceStatus::MpkVerified),
            0
        );
        assert_eq!(
            count_status(&report, PolicyPropertyEvidenceStatus::ProofPending),
            8
        );
        assert!(report.properties.iter().all(|property| {
            property.status == PolicyPropertyEvidenceStatus::ProofPending
                && !property.evidence.iter().any(|evidence| {
                    matches!(
                        evidence,
                        PolicyPropertyEvidenceRef::CheckedDeclaration { .. }
                            | PolicyPropertyEvidenceRef::CheckedTheoryCertificate { .. }
                    )
                })
        }));

        let _ = fs::remove_file(evidence_json);
        let _ = fs::remove_file(evidence_md);
    }
}

#[test]
fn policy_verify_repeated_refund_evidence_is_byte_identical() {
    let go2gir = ensure_go2gir();
    let refund = positive_payment_policy_corpus_cases()
        .into_iter()
        .find(|case| case.name == "refund")
        .expect("refund corpus case exists");
    let (first_output, first_json, first_md) =
        run_policy_verify_for_case(&refund, "refund-determinism-a", &go2gir);
    let (second_output, second_json, second_md) =
        run_policy_verify_for_case(&refund, "refund-determinism-b", &go2gir);

    assert!(
        first_output.status.success(),
        "stderr: {}",
        stderr(&first_output)
    );
    assert!(
        second_output.status.success(),
        "stderr: {}",
        stderr(&second_output)
    );

    let first_json_text = fs::read_to_string(&first_json).expect("first evidence JSON is readable");
    let second_json_text =
        fs::read_to_string(&second_json).expect("second evidence JSON is readable");
    let first_md_text = fs::read_to_string(&first_md).expect("first Markdown is readable");
    let second_md_text = fs::read_to_string(&second_md).expect("second Markdown is readable");

    assert_eq!(first_json_text, second_json_text);
    assert_eq!(first_md_text, second_md_text);
    let report = PolicyEvidenceReport::from_json(&first_json_text).expect("refund JSON parses");
    assert_payment_policy_report_fully_verified(
        &report,
        &refund.name,
        &refund.function_id,
        refund.expected_bound_pattern,
    );

    let _ = fs::remove_file(first_json);
    let _ = fs::remove_file(first_md);
    let _ = fs::remove_file(second_json);
    let _ = fs::remove_file(second_md);
}

#[test]
fn policy_verify_repeated_reserve_evidence_is_byte_identical() {
    let go2gir = ensure_go2gir();
    let (first_json, first_md) = temp_artifact_paths("reserve-determinism-a");
    let (second_json, second_md) = temp_artifact_paths("reserve-determinism-b");

    for (evidence_json, evidence_md) in [(&first_json, &first_md), (&second_json, &second_md)] {
        let output = run_mpk(&[
            "policy",
            "verify",
            RESERVE_TARGET,
            "--function",
            RESERVE_FUNCTION,
            "--contract",
            RESERVE_CONTRACT,
            "--strategy-profile",
            "payment-policy-alpha",
            "--checker-profile",
            "mvp-strict",
            "--evidence-json",
            evidence_json.to_str().expect("temp JSON path is UTF-8"),
            "--evidence-md",
            evidence_md.to_str().expect("temp Markdown path is UTF-8"),
            "--go2gir",
            go2gir.to_str().expect("go2gir path is UTF-8"),
        ]);
        assert!(output.status.success(), "stderr: {}", stderr(&output));
    }

    let first_json_text = fs::read_to_string(&first_json).expect("first evidence JSON is readable");
    let second_json_text =
        fs::read_to_string(&second_json).expect("second evidence JSON is readable");
    let first_md_text = fs::read_to_string(&first_md).expect("first Markdown is readable");
    let second_md_text = fs::read_to_string(&second_md).expect("second Markdown is readable");

    assert_eq!(first_json_text, second_json_text);
    assert_eq!(first_md_text, second_md_text);
    assert!(first_json_text.contains("<evidence.json>"));
    assert!(first_json_text.contains("<evidence.md>"));
    assert!(first_json_text.contains("<go2gir>"));
    assert!(!first_json_text.contains(first_json.to_str().expect("temp JSON path is UTF-8")));
    assert!(!first_json_text.contains(second_json.to_str().expect("temp JSON path is UTF-8")));
    assert!(!first_json_text.contains(go2gir.to_str().expect("go2gir path is UTF-8")));

    let _ = fs::remove_file(first_json);
    let _ = fs::remove_file(first_md);
    let _ = fs::remove_file(second_json);
    let _ = fs::remove_file(second_md);
}

#[test]
fn policy_verify_rejects_unknown_profiles_deterministically() {
    let cases: Vec<(&str, Vec<&str>, &str)> = vec![
        (
            "strategy",
            vec![
                "policy",
                "verify",
                RESERVE_TARGET,
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                RESERVE_CONTRACT,
                "--strategy-profile",
                "payment-policy-basic",
                "--checker-profile",
                "mvp-strict",
                "--evidence-json",
                "/tmp/mpk-evidence.json",
                "--evidence-md",
                "/tmp/mpk-evidence.md",
            ],
            "policy verify has unknown strategy profile",
        ),
        (
            "checker",
            vec![
                "policy",
                "verify",
                RESERVE_TARGET,
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                RESERVE_CONTRACT,
                "--strategy-profile",
                "payment-policy-alpha",
                "--checker-profile",
                "unchecked",
                "--evidence-json",
                "/tmp/mpk-evidence.json",
                "--evidence-md",
                "/tmp/mpk-evidence.md",
            ],
            "policy verify has unknown checker profile",
        ),
    ];

    for (name, args, expected) in cases {
        let first = run_mpk(&args);
        let second = run_mpk(&args);

        assert_eq!(first.status.code(), Some(2), "{name} status");
        assert!(first.stdout.is_empty(), "{name} stdout");
        assert_eq!(stderr(&first), stderr(&second), "{name} stderr");
        assert!(
            stderr(&first).contains(expected),
            "{name} stderr: {}",
            stderr(&first)
        );
    }
}

#[test]
fn policy_verify_rejects_path_traversal_product_paths() {
    let cases: Vec<(&str, Vec<&str>)> = vec![
        (
            "target",
            vec![
                "policy",
                "verify",
                "examples/payment_policies/../payment_policies/reserve",
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                RESERVE_CONTRACT,
                "--strategy-profile",
                "payment-policy-alpha",
                "--checker-profile",
                "mvp-strict",
                "--evidence-json",
                "/tmp/mpk-evidence.json",
                "--evidence-md",
                "/tmp/mpk-evidence.md",
            ],
        ),
        (
            "--contract",
            vec![
                "policy",
                "verify",
                RESERVE_TARGET,
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                "examples/payment_policies/reserve/../reserve/policy_contract.json",
                "--strategy-profile",
                "payment-policy-alpha",
                "--checker-profile",
                "mvp-strict",
                "--evidence-json",
                "/tmp/mpk-evidence.json",
                "--evidence-md",
                "/tmp/mpk-evidence.md",
            ],
        ),
        (
            "--evidence-json",
            vec![
                "policy",
                "verify",
                RESERVE_TARGET,
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                RESERVE_CONTRACT,
                "--strategy-profile",
                "payment-policy-alpha",
                "--checker-profile",
                "mvp-strict",
                "--evidence-json",
                "target/proof-ops/../evidence.json",
                "--evidence-md",
                "/tmp/mpk-evidence.md",
            ],
        ),
        (
            "--evidence-md",
            vec![
                "policy",
                "verify",
                RESERVE_TARGET,
                "--function",
                RESERVE_FUNCTION,
                "--contract",
                RESERVE_CONTRACT,
                "--strategy-profile",
                "payment-policy-alpha",
                "--checker-profile",
                "mvp-strict",
                "--evidence-json",
                "/tmp/mpk-evidence.json",
                "--evidence-md",
                "target/proof-ops/../evidence.md",
            ],
        ),
    ];

    for (label, args) in cases {
        let output = run_mpk(&args);

        assert_eq!(output.status.code(), Some(2), "{label} status");
        assert!(output.stdout.is_empty(), "{label} stdout");
        assert!(
            stderr(&output).contains(&format!(
                "policy verify {label} must not contain path traversal components"
            )),
            "{label} stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn policy_verify_strict_reserve_succeeds_with_checked_theory_evidence() {
    let go2gir = ensure_go2gir();
    let (evidence_json, evidence_md) = temp_artifact_paths("reserve-strict");
    let output = run_mpk(&[
        "policy",
        "verify",
        RESERVE_TARGET,
        "--function",
        RESERVE_FUNCTION,
        "--contract",
        RESERVE_CONTRACT,
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        evidence_json.to_str().expect("temp JSON path is UTF-8"),
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
        "--go2gir",
        go2gir.to_str().expect("go2gir path is UTF-8"),
        "--strict",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        format!(
            "ok policy verify status=verified verified=8 proof_pending=0 unsupported=0 evidence_json={} evidence_md={}\n",
            evidence_json.display(),
            evidence_md.display()
        )
    );
    assert!(output.stderr.is_empty());
    assert!(evidence_json.is_file());
    assert!(evidence_md.is_file());
    let report = read_evidence(&evidence_json);
    assert_reserve_report_fully_verified(&report);

    let _ = fs::remove_file(evidence_json);
    let _ = fs::remove_file(evidence_md);
}

#[test]
fn policy_verify_unsupported_scan_writes_untrusted_evidence_and_fails() {
    let go2gir = ensure_go2gir();
    let (evidence_json, evidence_md) = temp_artifact_paths("unsupported-scan");
    let output = run_mpk(&[
        "policy",
        "verify",
        "go-tools/go2gir/testdata/unsupported/map",
        "--function",
        "github.com/finitefield-org/mpk/go-tools/go2gir/testdata/unsupported/map.Lookup",
        "--contract",
        "go-tools/go2gir/testdata/unsupported/map/missing_contract.json",
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        evidence_json.to_str().expect("temp JSON path is UTF-8"),
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
        "--go2gir",
        go2gir.to_str().expect("go2gir path is UTF-8"),
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "policy verify failed: scan status=unsupported\n"
    );
    let report = read_evidence(&evidence_json);
    assert!(report.trusted_evidence.certificates.is_empty());
    assert!(report.trusted_evidence.theory_certificates.is_empty());
    assert!(report.properties.is_empty());
    assert!(report
        .helper_artifacts
        .warnings
        .iter()
        .any(|warning| warning.code == "GO2GIR_REJECTED_MAPS"));
    assert!(evidence_md.is_file());

    let _ = fs::remove_file(evidence_json);
    let _ = fs::remove_file(evidence_md);
}

#[test]
fn policy_verify_unsupported_property_writes_untrusted_evidence_and_fails() {
    let go2gir = ensure_go2gir();
    let (fixture_dir, contract, function_id) = unsupported_property_fixture();
    let (evidence_json, evidence_md) = temp_artifact_paths("unsupported-property");
    let output = run_mpk(&[
        "policy",
        "verify",
        fixture_dir.to_str().expect("fixture path is UTF-8"),
        "--function",
        &function_id,
        "--contract",
        contract.to_str().expect("contract path is UTF-8"),
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        evidence_json.to_str().expect("temp JSON path is UTF-8"),
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
        "--go2gir",
        go2gir.to_str().expect("go2gir path is UTF-8"),
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "policy verify failed: unsupported properties=2\n"
    );
    let report = read_evidence(&evidence_json);
    assert!(report.trusted_evidence.certificates.is_empty());
    assert!(report.trusted_evidence.theory_certificates.is_empty());
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::Unsupported),
        2
    );
    assert!(report.properties.iter().all(|property| {
        property.evidence.iter().any(|evidence| {
            matches!(
                evidence,
                PolicyPropertyEvidenceRef::UnsupportedFeature {
                    code,
                    message: _
                } if code == "UNSUPPORTED_PROPERTY_SHAPE"
            )
        })
    }));

    let _ = fs::remove_file(evidence_json);
    let _ = fs::remove_file(evidence_md);
    let _ = fs::remove_dir_all(fixture_dir);
}

#[test]
fn policy_verify_refuses_to_mutate_tracked_fixture_without_update_flag() {
    let tracked_path = repo_root().join(RESERVE_TRACKED_EVIDENCE);
    let tracked_before = fs::read_to_string(&tracked_path).expect("fixture readable");
    let (_, evidence_md) = temp_artifact_paths("tracked-guard");

    let output = run_mpk(&[
        "policy",
        "verify",
        RESERVE_TARGET,
        "--function",
        RESERVE_FUNCTION,
        "--contract",
        RESERVE_CONTRACT,
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        RESERVE_TRACKED_EVIDENCE,
        "--evidence-md",
        evidence_md.to_str().expect("temp Markdown path is UTF-8"),
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!(
            "policy verify refuses to overwrite tracked fixture {RESERVE_TRACKED_EVIDENCE} without --update-fixtures\n"
        )
    );
    let tracked_after = fs::read_to_string(&tracked_path).expect("fixture readable");
    assert_eq!(tracked_after, tracked_before);
    assert!(!evidence_md.exists());
}
