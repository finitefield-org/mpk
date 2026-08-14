#![cfg(feature = "vertex-ai")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mpk_cli::ai_explain::{
    build_sanitized_request, build_vertex_request, read_evidence_file, AiExplainErrorCode,
    ExplainLanguage, SanitizedEvidenceKind, SourcePropertyStatus, MAX_INPUT_BYTES,
    SYSTEM_INSTRUCTION_V0, USER_TEMPLATE_V0,
};
use mpk_cli::policy_evidence::{
    PolicyCertificateEvidence, PolicyContractArtifact, PolicyEvidenceReport, PolicyEvidenceTarget,
    PolicyHelperArtifactKind, PolicyHelperArtifacts, PolicyPropertyEvidence,
    PolicyPropertyEvidenceRef, PolicyPropertyEvidenceStatus, PolicySourceArtifact,
    PolicyTheoryCertificateEvidence, PolicyTrustedEvidence,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const EVIDENCE_FIXTURE: &[u8] =
    include_bytes!("../../../examples/payment_policies/reserve/evidence_alpha.json");
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn valid_fixture_is_redacted_and_request_is_byte_deterministic() {
    let first = build_vertex_request(EVIDENCE_FIXTURE, ExplainLanguage::English)
        .expect("fixture builds a request");
    let second = build_vertex_request(EVIDENCE_FIXTURE, ExplainLanguage::English)
        .expect("fixture builds the same request");

    assert_eq!(first.request_body, second.request_body);
    assert_eq!(first.evidence_sha256, sha256(EVIDENCE_FIXTURE));
    assert_eq!(first.request_body_sha256, sha256(&first.request_body));
    assert_eq!(
        first.sanitized_payload_sha256,
        sha256(first.payload_json.as_bytes())
    );
    assert_eq!(
        first.payload_json,
        serde_json::to_string(&first.payload).unwrap()
    );
    assert_eq!(first.request_body.last(), Some(&b'\n'));
    assert_eq!(first.payload.properties.len(), 8);
    assert_eq!(first.payload.summary.total, 8);
    assert_eq!(first.payload.summary.mpk_verified, 8);
    assert_eq!(
        first.payload.trusted_evidence_summary.theory_formats,
        ["bool", "linarith"]
    );

    let body_text = String::from_utf8(first.request_body.clone()).unwrap();
    for forbidden in [
        "example.com/payment/reserve",
        "ApprovedReserveCents",
        "theory:policy-linarith-0001",
        "examples/payment_policies/reserve/policy.go",
        "examples/payment_policies/reserve/policy_contract.json",
        "6f3f6af22f9ca554c6ca73d5384299711d83e46d8c3ae156275607e6daae3010",
        "Closed by checked linarith evidence",
        "mpk policy verify examples/payment_policies/reserve",
    ] {
        assert!(
            !body_text.contains(forbidden),
            "forbidden value leaked: {forbidden}"
        );
    }
    assert!(body_text.contains("USER_DATA:"));

    let body: Value = serde_json::from_slice(&first.request_body).unwrap();
    assert_eq!(
        body["systemInstruction"]["parts"][0]["text"],
        SYSTEM_INSTRUCTION_V0
    );
    assert_eq!(body["generationConfig"]["candidateCount"], 1);
    assert_eq!(body["generationConfig"]["temperature"], 0.0);
    assert_eq!(body["generationConfig"]["maxOutputTokens"], 8192);
    assert_eq!(
        body["generationConfig"]["responseMimeType"],
        "application/json"
    );
    assert_eq!(
        body["generationConfig"]["thinkingConfig"]["thinkingLevel"],
        "MINIMAL"
    );
    assert_eq!(
        body["generationConfig"]["thinkingConfig"]["includeThoughts"],
        false
    );
    assert_eq!(
        body["generationConfig"]["responseSchema"]["properties"]["property_explanations"]
            ["minItems"],
        8
    );
    assert_eq!(
        body["generationConfig"]["responseSchema"]["properties"]["property_explanations"]
            ["maxItems"],
        8
    );
    assert_eq!(
        body["generationConfig"]["responseSchema"]["properties"]["property_explanations"]["items"]
            ["properties"]["property_ref"]["enum"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
}

#[test]
fn prompt_hash_uses_the_pinned_byte_sequence() {
    let prepared = build_vertex_request(EVIDENCE_FIXTURE, ExplainLanguage::English).unwrap();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"systemInstruction\0");
    bytes.extend_from_slice(SYSTEM_INSTRUCTION_V0.as_bytes());
    bytes.extend_from_slice(b"userTemplate\0");
    bytes.extend_from_slice(USER_TEMPLATE_V0.as_bytes());
    assert_eq!(prepared.prompt_template_sha256, sha256(&bytes));
    assert!(SYSTEM_INSTRUCTION_V0.ends_with('\n'));
    assert!(USER_TEMPLATE_V0.ends_with('\n'));
    assert_eq!(
        USER_TEMPLATE_V0
            .matches("{{SANITIZED_PAYLOAD_JSON}}")
            .count(),
        1
    );
}

#[test]
fn exact_property_boundaries_and_invalid_reports_are_local() {
    assert_eq!(
        build_sanitized_request(&report_json(1))
            .unwrap()
            .properties
            .len(),
        1
    );
    assert_eq!(
        build_sanitized_request(&report_json(32))
            .unwrap()
            .properties
            .len(),
        32
    );
    assert_eq!(
        build_sanitized_request(&report_json(0)).unwrap_err().code(),
        AiExplainErrorCode::AiExplainNoProperties
    );
    assert_eq!(
        build_sanitized_request(&report_json(33))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainTooManyProperties
    );

    let mut unknown_schema = report(1);
    unknown_schema.schema = "mpk.policy.scan.v0".to_owned();
    assert_eq!(
        build_sanitized_request(&json_bytes(&unknown_schema))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );

    let mut unknown_field: Value = serde_json::from_slice(&json_bytes(&report(1))).unwrap();
    unknown_field["unexpected_source_field"] = Value::String("must not reach the model".to_owned());
    assert_eq!(
        build_sanitized_request(&serde_json::to_vec(&unknown_field).unwrap())
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );

    let mut duplicate_property = report(2);
    duplicate_property.properties[1].id = duplicate_property.properties[0].id.clone();
    assert_eq!(
        build_sanitized_request(&json_bytes(&duplicate_property))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );

    let mut duplicate_theory = report(1);
    duplicate_theory
        .trusted_evidence
        .theory_certificates
        .push(duplicate_theory.trusted_evidence.theory_certificates[0].clone());
    assert_eq!(
        build_sanitized_request(&json_bytes(&duplicate_theory))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );

    let mut duplicate_certificate = report(1);
    duplicate_certificate
        .trusted_evidence
        .certificates
        .push(PolicyCertificateEvidence::new(
            "certificate-1",
            "module",
            "certificate-path",
            "certificate-hash",
            "export-hash",
            "axiom-hash",
            Vec::new(),
        ));
    duplicate_certificate
        .trusted_evidence
        .certificates
        .push(PolicyCertificateEvidence::new(
            "certificate-1",
            "module-2",
            "certificate-path-2",
            "certificate-hash-2",
            "export-hash-2",
            "axiom-hash-2",
            Vec::new(),
        ));
    assert_eq!(
        build_sanitized_request(&json_bytes(&duplicate_certificate))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );

    let mut bidi_property = report(1);
    bidi_property.properties[0].id.push('\u{202e}');
    assert_eq!(
        build_sanitized_request(&json_bytes(&bidi_property))
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );
}

#[test]
fn projection_uses_allowlists_fixed_order_and_local_aliases() {
    let mut evidence = report(4);
    evidence.strategy_profile = "untrusted-strategy".to_owned();
    evidence.checker_profile = "untrusted-checker".to_owned();
    evidence.allowed_axiom_profiles = vec![
        "experimental-external".to_owned(),
        "zero-axiom".to_owned(),
        "zero-axiom".to_owned(),
        "untrusted-axiom".to_owned(),
    ];
    evidence.trusted_evidence.theory_certificates[0].format = "untrusted-format".to_owned();
    evidence.trusted_evidence.theory_certificates[0].theory = "untrusted-theory".to_owned();
    evidence
        .helper_artifacts
        .warnings
        .push(mpk_cli::policy_evidence::PolicyHelperWarning::new(
            "secret-warning-code",
            "secret warning message",
            PolicyHelperArtifactKind::Contract,
        ));
    evidence
        .helper_artifacts
        .warnings
        .push(mpk_cli::policy_evidence::PolicyHelperWarning::new(
            "other-warning-code",
            "other warning message",
            PolicyHelperArtifactKind::GoSource,
        ));
    evidence.properties[0].description = "not generated grammar".to_owned();
    evidence.properties[1].description =
        "Payment policy obligation classified as integer_runtime_safety.".to_owned();
    evidence.properties[2].description =
        "Payment policy obligation classified as zzz_unrecognized.".to_owned();
    evidence.properties[3].description =
        "Payment policy obligation classified as non_negative_result.".to_owned();

    let prepared = build_vertex_request(&json_bytes(&evidence), ExplainLanguage::Japanese).unwrap();
    let payload = prepared.payload;
    assert_eq!(payload.language, ExplainLanguage::Japanese);
    assert_eq!(payload.policy.strategy_profile, "unrecognized");
    assert_eq!(payload.policy.checker_profile, "unrecognized");
    assert_eq!(
        payload.policy.allowed_axiom_profiles,
        ["zero-axiom", "experimental-external", "unrecognized"]
    );
    assert_eq!(
        payload.trusted_evidence_summary.theory_formats,
        ["unrecognized"]
    );
    assert_eq!(payload.helper_warning_summary.len(), 2);
    assert_eq!(
        payload.helper_warning_summary[0].artifact.to_string(),
        "go_source"
    );
    assert_eq!(
        payload.helper_warning_summary[1].artifact.to_string(),
        "contract"
    );
    assert_eq!(payload.properties[0].property_ref, "property-0001");
    assert_eq!(payload.properties[0].category, "non_negative_result");
    assert_eq!(payload.properties[1].category, "integer_runtime_safety");
    assert_eq!(payload.properties[2].category, "unrecognized");
    assert_eq!(payload.properties[3].category, "unrecognized");
    assert_eq!(
        payload.properties[0].status,
        SourcePropertyStatus::MpkVerified
    );
    assert!(payload
        .properties
        .iter()
        .all(|property| property.property_ref.starts_with("property-")));

    let request_text = String::from_utf8(prepared.request_body).unwrap();
    for forbidden in [
        "untrusted-strategy",
        "untrusted-checker",
        "untrusted-axiom",
        "untrusted-format",
        "untrusted-theory",
        "secret-warning-code",
        "secret warning message",
        "not generated grammar",
    ] {
        assert!(
            !request_text.contains(forbidden),
            "untrusted value leaked: {forbidden}"
        );
    }
}

#[test]
fn evidence_kinds_are_deduplicated_in_compiled_order() {
    let mut evidence = report(1);
    evidence.properties[0]
        .evidence
        .push(PolicyPropertyEvidenceRef::HelperArtifact {
            artifact: PolicyHelperArtifactKind::Contract,
            summary: "untrusted summary".to_owned(),
        });
    evidence.properties[0].evidence.insert(
        0,
        PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id: "theory-1".to_owned(),
            obligation_id: "obligation-0".to_owned(),
        },
    );
    let payload = build_sanitized_request(&json_bytes(&evidence)).unwrap();
    assert_eq!(
        payload.properties[0].evidence_kinds,
        [
            SanitizedEvidenceKind::CheckedTheoryCertificate,
            SanitizedEvidenceKind::HelperArtifact
        ]
    );
}

#[test]
fn streaming_input_limit_and_dangling_reference_are_rejected() {
    let directory = test_directory("input-limit");
    let oversized = directory.join("oversized.json");
    fs::write(&oversized, vec![b'x'; MAX_INPUT_BYTES + 1]).unwrap();
    assert_eq!(
        read_evidence_file(&oversized).unwrap_err().code(),
        AiExplainErrorCode::AiExplainInputTooLarge
    );

    let mut dangling: Value = serde_json::from_slice(&json_bytes(&report(1))).unwrap();
    dangling["properties"][0]["evidence"][0]["theory_certificate_id"] =
        Value::String("missing-theory".to_owned());
    dangling["properties"][0]["evidence"][0]["obligation_id"] =
        Value::String("missing-obligation".to_owned());
    assert_eq!(
        build_sanitized_request(&serde_json::to_vec(&dangling).unwrap())
            .unwrap_err()
            .code(),
        AiExplainErrorCode::AiExplainInvalidEvidence
    );
    cleanup(&directory);
}

#[test]
fn dry_run_is_offline_no_clobber_and_escapes_status_path() {
    let directory = test_directory("dry-run");
    let evidence_path = directory.join("evidence.json");
    let output_path = directory.join("request \"preview\".json");
    fs::write(&evidence_path, EVIDENCE_FIXTURE).unwrap();

    let output = run_mpk(&[
        "explain",
        evidence_path.to_str().unwrap(),
        "--provider",
        "vertex-ai",
        "--language",
        "en",
        "--dry-run",
        "--request-json-out",
        output_path.to_str().unwrap(),
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    assert_eq!(
        stdout(&output),
        format!(
            "ok explain dry_run=1 network=0 model=gemini-3.5-flash cleanup=complete request_json={}\n",
            serde_json::to_string(output_path.to_str().unwrap()).unwrap()
        )
    );
    let first_body = fs::read(&output_path).unwrap();
    let prepared = build_vertex_request(EVIDENCE_FIXTURE, ExplainLanguage::English).unwrap();
    assert_eq!(first_body, prepared.request_body);
    assert_eq!(prepared.request_body_sha256, sha256(&first_body));
    let second = run_mpk(&[
        "explain",
        evidence_path.to_str().unwrap(),
        "--provider",
        "vertex-ai",
        "--dry-run",
        "--request-json-out",
        output_path.to_str().unwrap(),
    ]);
    assert!(!second.status.success());
    assert!(stderr(&second).contains("AI_EXPLAIN_OUTPUT_FAILED"));
    assert_eq!(fs::read(&output_path).unwrap(), first_body);
    cleanup(&directory);
}

#[test]
fn dry_run_rejects_network_and_credential_surface_flags() {
    let directory = test_directory("flags");
    let evidence_path = directory.join("evidence.json");
    let output_path = directory.join("request.json");
    fs::write(&evidence_path, EVIDENCE_FIXTURE).unwrap();
    for flag in [
        "--project",
        "--location",
        "--gcloud",
        "--output-json",
        "--output-md",
        "--overwrite",
    ] {
        let output = run_mpk(&[
            "explain",
            evidence_path.to_str().unwrap(),
            "--provider",
            "vertex-ai",
            "--dry-run",
            "--request-json-out",
            output_path.to_str().unwrap(),
            flag,
            "value",
        ]);
        assert_eq!(output.status.code(), Some(2), "flag {flag} was accepted");
        assert!(stdout(&output).is_empty());
    }
    let output = run_mpk(&[
        "explain",
        evidence_path.to_str().unwrap(),
        "--provider",
        "vertex-ai",
        "--dry-run",
        "--request-json-out",
        output_path.to_str().unwrap(),
        "--access-token",
        "secret",
    ]);
    assert_eq!(output.status.code(), Some(2));
    cleanup(&directory);
}

#[test]
fn dry_run_rejects_invalid_model_language_provider_and_paths() {
    let directory = test_directory("validation");
    let evidence_path = directory.join("evidence.json");
    let output_path = directory.join("request.json");
    fs::write(&evidence_path, EVIDENCE_FIXTURE).unwrap();

    let cases = [
        vec!["--model", "gemini-unreviewed"],
        vec!["--language", "fr"],
        vec!["--provider", "other-provider"],
        vec!["--unknown", "value"],
        vec!["--provider"],
        vec!["--provider", "vertex-ai", "--provider", "vertex-ai"],
    ];
    for extra in cases {
        let mut args = vec![
            "explain",
            evidence_path.to_str().unwrap(),
            "--provider",
            "vertex-ai",
            "--dry-run",
            "--request-json-out",
            output_path.to_str().unwrap(),
        ];
        args.extend(extra);
        let output = run_mpk(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args were accepted: {args:?}"
        );
        assert!(stdout(&output).is_empty());
    }

    let traversal_output = run_mpk(&[
        "explain",
        evidence_path.to_str().unwrap(),
        "--provider",
        "vertex-ai",
        "--dry-run",
        "--request-json-out",
        "target/../request.json",
    ]);
    assert_eq!(traversal_output.status.code(), Some(2));

    let missing_parent_output = run_mpk(&[
        "explain",
        evidence_path.to_str().unwrap(),
        "--provider",
        "vertex-ai",
        "--dry-run",
        "--request-json-out",
        directory.join("missing/request.json").to_str().unwrap(),
    ]);
    assert!(!missing_parent_output.status.success());
    assert!(stderr(&missing_parent_output).contains("AI_EXPLAIN_OUTPUT_FAILED"));
    cleanup(&directory);
}

fn report_json(property_count: usize) -> Vec<u8> {
    json_bytes(&report(property_count))
}

fn report(property_count: usize) -> PolicyEvidenceReport {
    let mut trusted = PolicyTrustedEvidence::empty();
    trusted
        .theory_certificates
        .push(PolicyTheoryCertificateEvidence::new(
            "theory-1",
            "linarith",
            "mpk.linarith.v0",
            "theory-hash",
            "mvp-strict",
            (0..property_count)
                .map(|index| format!("obligation-{index}"))
                .collect(),
        ));
    let helper = PolicyHelperArtifacts::new(
        PolicySourceArtifact::new("source-root", "source-hash", Vec::new()),
        PolicyContractArtifact::new("contract-path", "mpk.go.contract.v0", "contract-hash"),
    );
    let mut report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("package-path", "function-id"),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        trusted,
        helper,
    );
    for index in 0..property_count {
        let mut property = PolicyPropertyEvidence::new(
            format!("original-property-{index}"),
            if index % 2 == 0 {
                "Payment policy obligation classified as non_negative_result."
            } else {
                "Payment policy obligation classified as result_bounded_by_input."
            },
            PolicyPropertyEvidenceStatus::MpkVerified,
        );
        property
            .evidence
            .push(PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
                theory_certificate_id: "theory-1".to_owned(),
                obligation_id: format!("obligation-{index}"),
            });
        report.properties.push(property);
    }
    report
}

fn json_bytes(report: &PolicyEvidenceReport) -> Vec<u8> {
    report
        .to_deterministic_json()
        .expect("synthetic report serializes")
        .into_bytes()
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_mpk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .args(args)
        .output()
        .expect("mpk command runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn test_directory(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let directory = Path::new("target").join(format!(
        "vertex-ai-test-{label}-{}-{timestamp}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn cleanup(directory: &Path) {
    fs::remove_dir_all(directory).unwrap();
}

trait ArtifactKindLabel {
    fn to_string(self) -> String;
}

impl ArtifactKindLabel for mpk_cli::ai_explain::SanitizedArtifactKind {
    fn to_string(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .trim_matches('"')
            .to_owned()
    }
}
