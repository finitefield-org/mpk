use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;

use mpk_cli::policy_evidence::{
    PolicyEvidenceReport, PolicyPropertyEvidenceRef, PolicyPropertyEvidenceStatus,
};

const RESERVE_FUNCTION: &str = "example.com/payment/reserve.ApprovedReserveCents";
const RESERVE_TARGET: &str = "examples/payment_policies/reserve";
const RESERVE_CONTRACT: &str = "examples/payment_policies/reserve/policy_contract.json";
const RESERVE_TRACKED_EVIDENCE: &str = "examples/payment_policies/reserve/evidence_alpha.json";
const LINARITH_THEORY_CERT_HASH: &str =
    "a85d54f8d5c32dba5f414490120847013b7c727a3ce8b6ae2c3a44aae4edd7e1";
static BUILD_GO2GIR: Once = Once::new();

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

fn count_status(report: &PolicyEvidenceReport, status: PolicyPropertyEvidenceStatus) -> usize {
    report
        .properties
        .iter()
        .filter(|property| property.status == status)
        .count()
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
            "ok policy verify status=proof_pending verified=1 proof_pending=7 unsupported=0 evidence_json={} evidence_md={}\n",
            evidence_json.display(),
            evidence_md.display()
        )
    );
    assert!(output.stderr.is_empty());

    let report = read_evidence(&evidence_json);
    assert_eq!(report.target.package_path, "example.com/payment/reserve");
    assert_eq!(report.target.function_id, RESERVE_FUNCTION);
    assert_eq!(report.strategy_profile, "payment-policy-alpha");
    assert_eq!(report.checker_profile, "mvp-strict");
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::MpkVerified),
        1
    );
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::ProofPending),
        7
    );
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::Unsupported),
        0
    );
    assert_eq!(report.trusted_evidence.certificates, []);
    assert_eq!(report.trusted_evidence.theory_certificates.len(), 1);
    let theory = &report.trusted_evidence.theory_certificates[0];
    assert_eq!(theory.theory, "linarith");
    assert_eq!(theory.format, "mpk.linarith.v0");
    assert_eq!(theory.theory_certificate_hash, LINARITH_THEORY_CERT_HASH);
    assert_eq!(
        theory.checked_obligations,
        vec!["example.com/payment/reserve.ApprovedReserveCents.then.post0".to_owned()]
    );

    let verified = report
        .properties
        .iter()
        .find(|property| property.status == PolicyPropertyEvidenceStatus::MpkVerified)
        .expect("one property is verified");
    assert!(matches!(
        verified.evidence.as_slice(),
        [PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id,
            obligation_id
        }] if theory_certificate_id == "theory:policy-linarith-001"
            && obligation_id == "example.com/payment/reserve.ApprovedReserveCents.then.post0"
    ));
    assert!(report
        .properties
        .iter()
        .filter(|property| property.status != PolicyPropertyEvidenceStatus::MpkVerified)
        .all(|property| !property.evidence.iter().any(|evidence| {
            matches!(
                evidence,
                PolicyPropertyEvidenceRef::CheckedDeclaration { .. }
                    | PolicyPropertyEvidenceRef::CheckedTheoryCertificate { .. }
            )
        })));

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
fn policy_verify_strict_fails_after_writing_proof_pending_evidence() {
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

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "policy verify failed: proof-pending properties=7\n"
    );
    assert!(evidence_json.is_file());
    assert!(evidence_md.is_file());
    let report = read_evidence(&evidence_json);
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::MpkVerified),
        1
    );
    assert_eq!(
        count_status(&report, PolicyPropertyEvidenceStatus::ProofPending),
        7
    );

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
