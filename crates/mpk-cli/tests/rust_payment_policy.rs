#![allow(dead_code)]

#[path = "../src/frontend_protocol.rs"]
mod frontend_protocol;

mod policy_profile {
    pub use mpk_cli::policy_profile::*;
}

mod policy_report {
    pub use mpk_cli::policy_report::*;
}

mod policy_schema {
    pub use mpk_cli::policy_schema::*;
}

mod program_certificate {
    pub use mpk_cli::program_certificate::*;
}

mod frontend_runner {
    use crate::frontend_protocol::AcceptedFrontendEnvelope;
    use mpk_vc::{
        CapturedInput, FrontendIdentity, ReleaseRegistryIdentity, ReleaseSelectionRequest,
        ToolchainIdentity, ValidatedReleaseRegistry,
    };
    use serde_json::Value;

    pub(crate) struct FrontendRunRequest<'a> {
        pub(crate) release: ReleaseSelectionRequest,
        pub(crate) semantic_parameters: &'a Value,
        pub(crate) selection: &'a Value,
        pub(crate) captured_inputs: &'a [CapturedInput<'a>],
        pub(crate) staged_directories: &'a [&'a str],
        pub(crate) staged_placeholders: &'a [&'a str],
        pub(crate) contracts: &'a [String],
    }

    #[derive(Clone, Debug)]
    pub(crate) struct FrontendReleaseIdentity {
        pub(crate) release_registry: ReleaseRegistryIdentity,
        pub(crate) frontend: FrontendIdentity,
        pub(crate) toolchain: ToolchainIdentity,
        pub(crate) limit_profile: String,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct AcceptedFrontendRun {
        pub(crate) envelope: AcceptedFrontendEnvelope,
        pub(crate) release: FrontendReleaseIdentity,
        pub(crate) registry: ValidatedReleaseRegistry,
    }

    #[derive(Debug)]
    pub(crate) struct PreparedFrontendRun;

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct FrontendRunCode;

    impl FrontendRunCode {
        pub(crate) const fn as_str(self) -> &'static str {
            "FRONTEND_SANDBOX_UNAVAILABLE"
        }
    }

    #[derive(Debug)]
    pub(crate) struct FrontendRunError;

    impl FrontendRunError {
        pub(crate) const fn code(&self) -> FrontendRunCode {
            FrontendRunCode
        }
    }

    pub(crate) fn prepare_installed_frontend(
        _release: &ReleaseSelectionRequest,
    ) -> Result<PreparedFrontendRun, FrontendRunError> {
        Err(FrontendRunError)
    }

    pub(crate) fn run_prepared_frontend(
        _prepared: PreparedFrontendRun,
        _request: FrontendRunRequest<'_>,
    ) -> Result<AcceptedFrontendRun, FrontendRunError> {
        Err(FrontendRunError)
    }

    pub(crate) fn rust_pointer_width(target: &str) -> Option<i64> {
        match target {
            "i686-unknown-linux-gnu" => Some(32),
            "x86_64-unknown-linux-gnu" => Some(64),
            _ => None,
        }
    }

    pub(crate) fn rust_package_name(value: &str) -> bool {
        if value.len() > 1_024 {
            return false;
        }
        let mut bytes = value.bytes();
        bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }

    fn rust_identifier(value: &str) -> bool {
        if value == "_" || value.len() > 255 || !value.is_ascii() {
            return false;
        }
        let mut bytes = value.bytes();
        bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    }

    pub(crate) fn rust_function_id(value: &str, crate_name: &str) -> bool {
        if value.len() > 1_024 {
            return false;
        }
        let mut segments = value.split("::");
        rust_identifier(crate_name)
            && segments.next() == Some(crate_name)
            && segments.next().is_some_and(rust_identifier)
            && segments.all(rust_identifier)
    }
}

#[path = "../src/policy_scan.rs"]
mod policy_scan;

#[path = "../src/policy_verify/v1.rs"]
mod policy_verify_v1;

use frontend_protocol::{
    validate_frontend_process_from_staging, FrontendProcessFacts, FrontendStagingRequest,
};
use frontend_runner::{AcceptedFrontendRun, FrontendReleaseIdentity};
use mpk_cert::decode_canonical_certificate;
use mpk_vc::{
    canonical_json_bytes, canonical_vir_json, parse_strict_json, sha256_raw_file_bytes,
    validate_release_registry, InputKind, StrictJsonLimits, ValidatedReleaseRegistry,
};
use policy_scan::v1::OwnedCapturedInput;
use policy_schema::{PolicyAxiomReportV1, PolicyEvidenceReferenceV1, PolicyEvidenceV1};
use policy_verify_v1::run_policy_verify_v1_with_assembler;
use program_certificate::{
    assemble_program_certificate_alpha, CheckedProgramCertificate, ProgramCertificateOutcome,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_ID: &str = "mpk.release.registry.v0";
const REGISTRY_SHA256: &str = "5f25417656b9acb3d5272c7e8d3fcc7fd5873d18f7985bbc301579b64a462279";
const RUST_FRONTEND: &str = "frontend.rust.rust2vir.candidate.v0";
const RUST_TOOLCHAIN: &str = "toolchain.rust.nightly-2025-06-01.candidate.v0";
const UPDATE_ENV: &str = "MPK_UPDATE_RUST_PAYMENT_POLICY";
const POSITIVE_ENVELOPE: &str = "frontend-envelope.json";
const PENDING_ENVELOPE: &str = "insufficient-precondition.frontend-envelope.json";
const FRONTEND_ARTIFACTS: [&str; 8] = [
    POSITIVE_ENVELOPE,
    "vir.json",
    "source-map.json",
    "source-manifest.frontend.json",
    PENDING_ENVELOPE,
    "insufficient-precondition.vir.json",
    "insufficient-precondition.source-map.json",
    "insufficient-precondition.source-manifest.frontend.json",
];
const JSON_LIMITS: StrictJsonLimits = StrictJsonLimits {
    max_input_bytes: 268_435_456,
    max_nodes: 16_777_216,
    max_depth: 256,
    max_string_bytes: 1_048_576,
};

#[test]
fn rust_payment_policy_freezes_dual_checked_and_pending_structural_results() {
    let example = repo_root().join("examples/rust-payment-policy");
    let artifacts = example.join("artifacts");
    let positive_envelope = artifacts.join(POSITIVE_ENVELOPE);
    let pending_envelope = artifacts.join(PENDING_ENVELOPE);
    assert!(
        positive_envelope.is_file() && pending_envelope.is_file(),
        "frontend fixtures are missing; generate {POSITIVE_ENVELOPE} and {PENDING_ENVELOPE} first"
    );

    let positive_inputs = captured_example_inputs(&example, &positive_envelope);
    let positive_accepted = accepted_ready_run(&positive_envelope, &positive_inputs);
    let first_directory = tempfile::tempdir().unwrap();
    let first = run_ready(
        first_directory.path(),
        &verify_argv(true, false),
        positive_inputs.clone(),
        positive_accepted.clone(),
    );
    assert_positive(&first);

    let second_directory = tempfile::tempdir().unwrap();
    let second = run_ready(
        second_directory.path(),
        &verify_argv(true, false),
        positive_inputs,
        positive_accepted,
    );
    assert_positive(&second);
    assert_deterministic(
        &first,
        &second,
        first_directory.path(),
        second_directory.path(),
    );

    let pending_inputs = captured_example_inputs(&example, &pending_envelope);
    let pending_accepted = accepted_ready_run(&pending_envelope, &pending_inputs);
    let pending_directory = tempfile::tempdir().unwrap();
    let pending = run_ready(
        pending_directory.path(),
        &verify_argv(false, true),
        pending_inputs.clone(),
        pending_accepted.clone(),
    );
    assert_pending(&pending);
    let pending_repeat_directory = tempfile::tempdir().unwrap();
    let pending_repeat = run_ready(
        pending_repeat_directory.path(),
        &verify_argv(false, true),
        pending_inputs.clone(),
        pending_accepted.clone(),
    );
    assert_pending(&pending_repeat);
    assert_pending_deterministic(
        &pending,
        &pending_repeat,
        pending_directory.path(),
        pending_repeat_directory.path(),
    );

    let strict_directory = tempfile::tempdir().unwrap();
    let strict_error = run_policy_verify_v1_with_assembler(
        &verify_argv(true, true),
        strict_directory.path(),
        pending_inputs,
        |_| Ok(()),
        |(), _| Ok(pending_accepted.clone()),
        |_| false,
        assemble_program_certificate_alpha,
    )
    .unwrap_err();
    assert_eq!(strict_error.code(), "POLICY_PROOF_PENDING");
    let strict_evidence: PolicyEvidenceV1 =
        serde_json::from_slice(&fs::read(strict_directory.path().join("evidence.json")).unwrap())
            .unwrap();
    assert!(strict_evidence.verification_options.strict);
    assert!(strict_evidence.trusted_evidence.certificates.is_empty());
    assert!(strict_evidence
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .any(|member| member.kind == "callee_precondition" && member.status == "proof_pending"));
    assert!(strict_directory.path().join("evidence.md").is_file());

    assert_frontend_artifacts(&artifacts, &first, &pending);

    let frozen = frozen_artifacts(
        &first,
        first_directory.path(),
        &pending,
        pending_directory.path(),
    );
    assert_no_local_paths(
        &frozen,
        &[&example, first_directory.path(), pending_directory.path()],
    );
    assert_or_update_frozen(&artifacts, frozen);
}

fn assert_positive(output: &policy_verify_v1::PolicyVerifyV1RunOutput) {
    assert!(output.invocation.strict);
    assert_eq!(
        output.invocation.strategy_profile,
        "payment-policy-rust-alpha"
    );
    assert_eq!(output.invocation.checker_profile, "mvp-strict");
    assert_eq!(output.invocation.axiom_profile, "mvp-theory");
    let candidate = candidate(output);
    assert_eq!(
        candidate.generated_declarations.len(),
        output.skeleton.skeleton().theorem_declarations.len()
    );
    assert!(!candidate.generated_declarations.is_empty());
    assert_eq!(candidate.rust_report.axiom_count, 0);
    assert_eq!(candidate.reference_report.axiom_count, 0);
    assert!(candidate.certificate.imports.is_empty());
    assert!(candidate.certificate.proof_node_table.is_empty());
    assert!(candidate.certificate.theory_certificates.is_empty());
    assert_eq!(
        decode_canonical_certificate(&candidate.bytes).unwrap(),
        candidate.certificate
    );

    let evidence = output.evidence.document();
    assert_eq!(evidence.source_language, "rust");
    assert_eq!(evidence.semantic_profile, "mpk.rust.checked.v0");
    assert_eq!(evidence.strategy_profile, "payment-policy-rust-alpha");
    assert_eq!(evidence.checker_profile, "mvp-strict");
    assert_eq!(evidence.axiom_profile, "mvp-theory");
    assert!(evidence.verification_options.strict);
    assert_eq!(evidence.trusted_evidence.certificates.len(), 1);
    assert!(evidence.trusted_evidence.theory_certificates.is_empty());
    assert_eq!(evidence.trusted_evidence.checker_verdicts.len(), 2);
    assert_eq!(
        evidence
            .trusted_evidence
            .checker_verdicts
            .iter()
            .map(|verdict| (verdict.checker.as_str(), verdict.verdict.as_str()))
            .collect::<Vec<_>>(),
        [
            ("rust_fast_kernel", "accepted"),
            ("reference_checker", "accepted")
        ]
    );
    match &evidence.trusted_evidence.axiom_report {
        PolicyAxiomReportV1::Checked {
            category_counts, ..
        } => {
            assert_eq!(category_counts.total_axiom_count, 0);
            assert_eq!(category_counts.core_axiom_count, 0);
            assert_eq!(category_counts.builtin_theory_axiom_count, 0);
            assert_eq!(category_counts.go_semantics_axiom_count, 0);
            assert_eq!(category_counts.external_axiom_count, 0);
        }
        PolicyAxiomReportV1::NotGenerated => panic!("accepted example omitted its axiom report"),
    }
    assert!(!evidence.properties.is_empty());
    assert_eq!(
        evidence
            .properties
            .iter()
            .map(|property| (property.id.as_str(), property.members[0].member_id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "approved_reserve_cents_callee_panic_free",
                "payment_policy::approved_reserve_cents#callee_panic_free#000000"
            ),
            (
                "approved_reserve_cents_callee_precondition",
                "payment_policy::approved_reserve_cents#callee_precondition#000000"
            ),
            (
                "approved_reserve_cents_postcondition",
                "payment_policy::approved_reserve_cents#postcondition#000000"
            ),
        ]
    );
    assert!(evidence.properties.iter().all(|property| {
        property.status == "mpk_verified"
            && property.members.iter().all(|member| {
                member.status == "mpk_verified"
                    && member.evidence
                        == [PolicyEvidenceReferenceV1::CheckedDeclaration {
                            certificate_id: "program".to_owned(),
                        }]
            })
    }));
    let members = evidence
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .collect::<Vec<_>>();
    assert!(members.iter().any(|member| {
        member.function_id == "payment_policy::approved_reserve_cents"
            && member.kind == "callee_precondition"
            && member.status == "mpk_verified"
    }));
    assert_exact_declaration_dependencies(output);
}

fn assert_pending(output: &policy_verify_v1::PolicyVerifyV1RunOutput) {
    assert!(!output.invocation.strict);
    let (generated, missing) = match output.program_certificate.as_ref().unwrap() {
        ProgramCertificateOutcome::Pending {
            generated_declarations,
            missing_member_ids,
        } => (generated_declarations, missing_member_ids),
        ProgramCertificateOutcome::Candidate(_) => {
            panic!("insufficient precondition unexpectedly proved")
        }
        ProgramCertificateOutcome::Unaccepted(candidate) => {
            panic!(
                "pending example reached checkers: {}",
                candidate.failure_detail
            )
        }
    };
    assert_eq!(
        generated.len(),
        output.skeleton.skeleton().theorem_declarations.len(),
        "pending output must retain the complete deterministic interface plan"
    );
    assert_eq!(
        missing.len(),
        1,
        "exactly the missing callee precondition must remain pending"
    );
    assert_eq!(
        missing[0],
        "payment_policy::approved_reserve_cents#callee_precondition#000000"
    );
    let evidence = output.evidence.document();
    assert!(!evidence.verification_options.strict);
    assert!(evidence.trusted_evidence.certificates.is_empty());
    assert!(evidence.trusted_evidence.theory_certificates.is_empty());
    assert!(evidence
        .trusted_evidence
        .checker_verdicts
        .iter()
        .all(|verdict| { verdict.verdict == "not_run" && verdict.certificate_ids.is_empty() }));
    assert!(matches!(
        evidence.trusted_evidence.axiom_report,
        PolicyAxiomReportV1::NotGenerated
    ));

    let members = evidence
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .collect::<Vec<_>>();
    let missing_row = members
        .iter()
        .find(|member| member.member_id == missing[0])
        .expect("the assembler's missing member must occur in policy evidence");
    assert_eq!(missing_row.kind, "callee_precondition");
    assert_eq!(
        missing_row.function_id,
        "payment_policy::approved_reserve_cents"
    );
    assert!(members.iter().all(|member| {
        member.status == "proof_pending"
            && member.evidence
                == [PolicyEvidenceReferenceV1::HelperArtifact {
                    artifact_id: "vc".to_owned(),
                }]
    }));
    assert!(evidence
        .properties
        .iter()
        .all(|property| property.status == "proof_pending"));
}

fn assert_exact_declaration_dependencies(output: &policy_verify_v1::PolicyVerifyV1RunOutput) {
    let candidate = candidate(output);
    let declarations =
        &output.evidence.document().trusted_evidence.certificates[0].checked_declarations;
    assert_eq!(declarations.len(), candidate.generated_declarations.len());
    let by_name = candidate
        .generated_declarations
        .iter()
        .map(|declaration| {
            (
                declaration.name.as_str(),
                declaration.declaration_hash.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(by_name.len(), candidate.generated_declarations.len());
    let group_by_name = declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration.group_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let observed = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.group_id.as_str(),
                declaration.group_kind.as_str(),
                declaration
                    .member_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                declaration
                    .dependencies
                    .iter()
                    .map(|dependency| group_by_name[dependency.name.as_str()])
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        [
            (
                "payment_policy::retain_approval.contract",
                "contract",
                vec!["payment_policy::retain_approval#postcondition#000000"],
                vec![],
            ),
            (
                "payment_policy::retain_approval.panic_free",
                "panic_free",
                vec![],
                vec!["payment_policy::retain_approval.contract"],
            ),
            (
                "payment_policy::approved_reserve_cents.contract",
                "contract",
                vec![
                    "payment_policy::approved_reserve_cents#callee_precondition#000000",
                    "payment_policy::approved_reserve_cents#postcondition#000000",
                ],
                vec!["payment_policy::retain_approval.contract"],
            ),
            (
                "payment_policy::approved_reserve_cents.panic_free",
                "panic_free",
                vec!["payment_policy::approved_reserve_cents#callee_panic_free#000000"],
                vec![
                    "payment_policy::approved_reserve_cents.contract",
                    "payment_policy::retain_approval.contract",
                    "payment_policy::retain_approval.panic_free",
                ],
            ),
        ]
    );
    for declaration in declarations {
        assert_eq!(
            by_name.get(declaration.name.as_str()),
            Some(&declaration.declaration_hash.as_str())
        );
        let mut dependency_names = BTreeSet::new();
        for dependency in &declaration.dependencies {
            assert!(dependency_names.insert(dependency.name.as_str()));
            assert_eq!(
                by_name.get(dependency.name.as_str()),
                Some(&dependency.declaration_hash.as_str()),
                "every dependency must name an exact checked declaration"
            );
        }
    }
}

fn assert_deterministic(
    first: &policy_verify_v1::PolicyVerifyV1RunOutput,
    second: &policy_verify_v1::PolicyVerifyV1RunOutput,
    first_root: &Path,
    second_root: &Path,
) {
    assert_eq!(
        first.scan.scan.canonical_bytes(),
        second.scan.scan.canonical_bytes()
    );
    assert_eq!(first.vc.canonical_bytes(), second.vc.canonical_bytes());
    assert_eq!(
        first.skeleton.canonical_bytes(),
        second.skeleton.canonical_bytes()
    );
    assert_eq!(
        first.certificate_manifest.canonical_bytes(),
        second.certificate_manifest.canonical_bytes()
    );
    assert_eq!(candidate(first).bytes, candidate(second).bytes);
    assert_eq!(
        first.evidence.canonical_bytes(),
        second.evidence.canonical_bytes()
    );
    assert_eq!(
        fs::read(first_root.join("evidence.md")).unwrap(),
        fs::read(second_root.join("evidence.md")).unwrap()
    );
}

fn assert_pending_deterministic(
    first: &policy_verify_v1::PolicyVerifyV1RunOutput,
    second: &policy_verify_v1::PolicyVerifyV1RunOutput,
    first_root: &Path,
    second_root: &Path,
) {
    assert_eq!(
        first.scan.scan.canonical_bytes(),
        second.scan.scan.canonical_bytes()
    );
    assert_eq!(first.vc.canonical_bytes(), second.vc.canonical_bytes());
    assert_eq!(
        first.skeleton.canonical_bytes(),
        second.skeleton.canonical_bytes()
    );
    assert_eq!(
        first.certificate_manifest.canonical_bytes(),
        second.certificate_manifest.canonical_bytes()
    );
    assert_eq!(first.program_certificate, second.program_certificate);
    assert_eq!(
        first.evidence.canonical_bytes(),
        second.evidence.canonical_bytes()
    );
    assert_eq!(
        fs::read(first_root.join("evidence.md")).unwrap(),
        fs::read(second_root.join("evidence.md")).unwrap()
    );
}

fn frozen_artifacts(
    positive: &policy_verify_v1::PolicyVerifyV1RunOutput,
    positive_root: &Path,
    pending: &policy_verify_v1::PolicyVerifyV1RunOutput,
    pending_root: &Path,
) -> BTreeMap<String, Vec<u8>> {
    let candidate = candidate(positive);
    let axiom = mpk_kernel::verify_certificate_bytes_axiom_report_json_output(&candidate.bytes);
    assert!(axiom.accepted);
    let rust_report: Value = serde_json::from_str(&mpk_kernel::render_verification_report_json(
        &candidate.rust_report,
    ))
    .unwrap();
    let checker_reports = json!({
        "reference_checker": {
            "axiom_count": candidate.reference_report.axiom_count,
            "axiom_report_hash": candidate.reference_report.axiom_report_hash,
            "certificate_hash": candidate.reference_report.certificate_hash,
            "declaration_count": candidate.reference_report.declaration_count,
            "export_hash": candidate.reference_report.export_hash,
            "module": candidate.reference_report.module,
            "verdict": "accepted"
        },
        "rust_fast_kernel": rust_report,
        "schema": "mpk.rust_payment_policy.checker_reports.v0"
    });
    let findings = json!({
        "findings": [],
        "schema": "mpk.rust_payment_policy.findings.v0"
    });
    let reproduction = json!({
        "recipes": positive.evidence.document().reproduction_recipes,
        "schema": "mpk.rust_payment_policy.reproduction.v0"
    });
    let package_manifest = json!({
        "certificates": [{
            "expected_axiom_report_hash": candidate.reference_report.axiom_report_hash,
            "expected_certificate_hash": candidate.reference_report.certificate_hash,
            "expected_export_hash": candidate.reference_report.export_hash,
            "module": candidate.certificate.module,
            "path": "examples/rust-payment-policy/artifacts/program.mpcert"
        }],
        "imports": [],
        "module": "Example.Rust.PaymentPolicy",
        "policy": {
            "allowed_axiom_profiles": ["mvp-theory"],
            "checker_profile": "mvp-strict",
            "require_reference_checker": true,
            "require_source_free_check": true
        },
        "schema": "mpk.package.v0"
    });
    BTreeMap::from([
        (
            "policy-scan.json".to_owned(),
            positive.scan.scan.canonical_bytes().to_vec(),
        ),
        ("vc.json".to_owned(), positive.vc.canonical_bytes().to_vec()),
        (
            "vc-skeleton.json".to_owned(),
            positive.skeleton.canonical_bytes().to_vec(),
        ),
        (
            "source-manifest.certificate.json".to_owned(),
            positive.certificate_manifest.canonical_bytes().to_vec(),
        ),
        ("program.mpcert".to_owned(), candidate.bytes.clone()),
        (
            "axiom-report.json".to_owned(),
            canonical_value(&serde_json::from_str::<Value>(&axiom.json).unwrap()),
        ),
        (
            "checker-reports.json".to_owned(),
            canonical_value(&checker_reports),
        ),
        (
            "evidence.json".to_owned(),
            positive.evidence.canonical_bytes().to_vec(),
        ),
        (
            "evidence.md".to_owned(),
            fs::read(positive_root.join("evidence.md")).unwrap(),
        ),
        (
            "reproduction.json".to_owned(),
            canonical_value(&reproduction),
        ),
        ("findings.json".to_owned(), canonical_value(&findings)),
        (
            "package-manifest.json".to_owned(),
            canonical_value(&package_manifest),
        ),
        (
            "insufficient-precondition.policy-scan.json".to_owned(),
            pending.scan.scan.canonical_bytes().to_vec(),
        ),
        (
            "insufficient-precondition.vc.json".to_owned(),
            pending.vc.canonical_bytes().to_vec(),
        ),
        (
            "insufficient-precondition.vc-skeleton.json".to_owned(),
            pending.skeleton.canonical_bytes().to_vec(),
        ),
        (
            "insufficient-precondition.source-manifest.certificate.json".to_owned(),
            pending.certificate_manifest.canonical_bytes().to_vec(),
        ),
        (
            "insufficient-precondition.evidence.json".to_owned(),
            pending.evidence.canonical_bytes().to_vec(),
        ),
        (
            "insufficient-precondition.evidence.md".to_owned(),
            fs::read(pending_root.join("evidence.md")).unwrap(),
        ),
    ])
}

fn assert_frontend_artifacts(
    root: &Path,
    positive: &policy_verify_v1::PolicyVerifyV1RunOutput,
    pending: &policy_verify_v1::PolicyVerifyV1RunOutput,
) {
    for (prefix, output) in [("", positive), ("insufficient-precondition.", pending)] {
        let envelope = &output.scan.frontend.envelope;
        let frontend = envelope.artifacts.as_ref().unwrap();
        let expected = [
            (
                format!("{prefix}frontend-envelope.json"),
                envelope.canonical_bytes.clone(),
            ),
            (
                format!("{prefix}vir.json"),
                canonical_vir_json(&frontend.vir).unwrap(),
            ),
            (
                format!("{prefix}source-map.json"),
                frontend.source_map.canonical_bytes().to_vec(),
            ),
            (
                format!("{prefix}source-manifest.frontend.json"),
                frontend.source_manifest.canonical_bytes().to_vec(),
            ),
        ];
        for (relative, bytes) in expected {
            assert_eq!(
                fs::read(root.join(&relative))
                    .unwrap_or_else(|error| panic!("read {relative}: {error}")),
                bytes,
                "{relative} must be regenerated by the pinned frontend owner"
            );
        }
    }
}

fn assert_or_update_frozen(root: &Path, mut artifacts: BTreeMap<String, Vec<u8>>) {
    let mut inventory = FRONTEND_ARTIFACTS
        .iter()
        .map(|path| {
            let bytes = fs::read(root.join(path))
                .unwrap_or_else(|error| panic!("read frontend artifact {path}: {error}"));
            json!({
                "bytes": bytes.len(),
                "path": path,
                "sha256": sha256_raw_file_bytes(&bytes).to_string()
            })
        })
        .chain(artifacts.iter().map(|(path, bytes)| {
            json!({
                "bytes": bytes.len(),
                "path": path,
                "sha256": sha256_raw_file_bytes(bytes).to_string()
            })
        }))
        .collect::<Vec<_>>();
    inventory.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    artifacts.insert(
        "manifest.json".to_owned(),
        canonical_value(&json!({
            "artifacts": inventory,
            "findings": [],
            "generator": {
                "environment": UPDATE_ENV,
                "test": "rust_payment_policy_freezes_dual_checked_and_pending_structural_results"
            },
            "profiles": {
                "axiom": "mvp-theory",
                "checker": "mvp-strict",
                "semantic": "mpk.rust.checked.v0",
                "strategy": "payment-policy-rust-alpha",
                "target": "x86_64-unknown-linux-gnu"
            },
            "schema": "mpk.rust_payment_policy.artifacts.v0"
        })),
    );
    for (relative, expected) in artifacts {
        let path = root.join(&relative);
        if std::env::var_os(UPDATE_ENV).is_some() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, expected)
                .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        } else {
            assert_eq!(
                fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
                expected,
                "{} is stale; rerun with {UPDATE_ENV}=1",
                path.display()
            );
        }
    }
}

fn assert_no_local_paths(artifacts: &BTreeMap<String, Vec<u8>>, roots: &[&Path]) {
    for (name, bytes) in artifacts {
        for root in roots {
            let needle = root.to_string_lossy();
            assert!(
                !contains(bytes, needle.as_bytes()),
                "{name} leaks local path {needle}"
            );
        }
        for forbidden in [
            b"/tmp/".as_slice(),
            b"/root/".as_slice(),
            b"/mpk/input".as_slice(),
            b"/mpk/toolchain".as_slice(),
            b"/not-emitted/".as_slice(),
        ] {
            assert!(
                !contains(bytes, forbidden),
                "{name} leaks forbidden path {}",
                String::from_utf8_lossy(forbidden)
            );
        }
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn canonical_value(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(&serialized, JSON_LIMITS).unwrap();
    canonical_json_bytes(&strict).unwrap()
}

fn candidate(output: &policy_verify_v1::PolicyVerifyV1RunOutput) -> &CheckedProgramCertificate {
    match output.program_certificate.as_ref().unwrap() {
        ProgramCertificateOutcome::Candidate(candidate) => candidate,
        ProgramCertificateOutcome::Pending {
            missing_member_ids, ..
        } => {
            panic!("positive example remained pending: {missing_member_ids:?}")
        }
        ProgramCertificateOutcome::Unaccepted(candidate) => {
            panic!(
                "positive example failed checker acceptance: {}",
                candidate.failure_detail
            )
        }
    }
}

fn run_ready(
    working_directory: &Path,
    argv: &[String],
    inputs: Vec<OwnedCapturedInput>,
    accepted: AcceptedFrontendRun,
) -> policy_verify_v1::PolicyVerifyV1RunOutput {
    run_policy_verify_v1_with_assembler(
        argv,
        working_directory,
        inputs,
        |_| Ok(()),
        |(), _| Ok(accepted.clone()),
        |_| false,
        assemble_program_certificate_alpha,
    )
    .unwrap()
    .unwrap()
}

fn accepted_ready_run(envelope_path: &Path, inputs: &[OwnedCapturedInput]) -> AcceptedFrontendRun {
    let bytes = fs::read(envelope_path).unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let registry = tracked_registry();
    let captured = inputs
        .iter()
        .map(OwnedCapturedInput::as_ref)
        .collect::<Vec<_>>();
    let accepted = validate_frontend_process_from_staging(
        FrontendStagingRequest {
            source_language: "rust",
            semantic_profile: "mpk.rust.checked.v0",
            semantic_parameters: &value["semantic_parameters"],
            selection: &value["selection"],
            release_registry: Some(&registry),
            available_inputs: &captured,
        },
        FrontendProcessFacts {
            exit_code: Some(0),
            signaled: false,
            stdout: &bytes,
            stderr_observed_bytes: 0,
        },
    )
    .unwrap();
    AcceptedFrontendRun {
        envelope: accepted,
        release: release_from_manifest(&value["source_manifest"]),
        registry,
    }
}

fn captured_example_inputs(example: &Path, envelope_path: &Path) -> Vec<OwnedCapturedInput> {
    let envelope: Value = serde_json::from_slice(&fs::read(envelope_path).unwrap()).unwrap();
    envelope["source_manifest"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| {
            let normalized_path = input["normalized_path"].as_str().unwrap();
            OwnedCapturedInput {
                kind: match input["kind"].as_str().unwrap() {
                    "source" => InputKind::Source,
                    "contract" => InputKind::Contract,
                    "build_manifest" => InputKind::BuildManifest,
                    "lockfile" => InputKind::Lockfile,
                    kind => panic!("unknown example input kind {kind}"),
                },
                normalized_path: normalized_path.to_owned(),
                bytes: fs::read(example.join(normalized_path)).unwrap_or_else(|error| {
                    panic!("read captured input {normalized_path}: {error}")
                }),
            }
        })
        .collect()
}

fn release_from_manifest(manifest: &Value) -> FrontendReleaseIdentity {
    FrontendReleaseIdentity {
        release_registry: serde_json::from_value(manifest["release_registry"].clone()).unwrap(),
        frontend: serde_json::from_value(manifest["frontend"].clone()).unwrap(),
        toolchain: serde_json::from_value(manifest["toolchain"].clone()).unwrap(),
        limit_profile: manifest["limit_profile"].as_str().unwrap().to_owned(),
    }
}

fn tracked_registry() -> ValidatedReleaseRegistry {
    validate_release_registry(include_bytes!(
        "../../../release/bundles/bundle-registry.json"
    ))
    .unwrap()
}

fn verify_argv(strict: bool, insufficient_precondition: bool) -> Vec<String> {
    let selected = if insufficient_precondition {
        "contracts/insufficient-precondition.json"
    } else {
        "contracts/selected.json"
    };
    let mut argv = [
        "mpk",
        "policy",
        "verify",
        ".",
        "--language",
        "rust",
        "--semantic-profile",
        "mpk.rust.checked.v0",
        "--require-release-registry-id",
        REGISTRY_ID,
        "--require-release-registry-sha256",
        REGISTRY_SHA256,
        "--frontend-bundle",
        RUST_FRONTEND,
        "--toolchain-bundle",
        RUST_TOOLCHAIN,
        "--target",
        "x86_64-unknown-linux-gnu",
        "--package",
        "payment-policy",
        "--function",
        "payment_policy::approved_reserve_cents",
        "--contract",
        "contracts/helper.json",
        "--contract",
        selected,
        "--strategy-profile",
        "payment-policy-rust-alpha",
        "--checker-profile",
        "mvp-strict",
        "--axiom-profile",
        "mvp-theory",
        "--evidence-json",
        "evidence.json",
        "--evidence-md",
        "evidence.md",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    if strict {
        argv.push("--strict".to_owned());
    }
    argv
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
