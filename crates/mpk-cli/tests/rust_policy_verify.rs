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
use mpk_cert::encode::DeclarationKind;
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate, encode_certificate, export_block_hash, hash_hex,
};
use mpk_vc::{validate_release_registry, InputKind, ValidatedReleaseRegistry};
use policy_scan::v1::OwnedCapturedInput;
use policy_schema::{
    PolicyAxiomCategoryCountsV1, PolicyAxiomReportV1, PolicyEvidenceReferenceV1, PolicyEvidenceV1,
};
use policy_verify_v1::{
    run_policy_verify_v1_with, run_policy_verify_v1_with_assembler,
    validate_package_release_policy_v1, ActiveReleasePolicyV1, PackagePolicyV1,
};
use program_certificate::{
    assemble_program_certificate_alpha, CheckedProgramCertificate, ProgramCertificateErrorKind,
    ProgramCertificateOutcome, ProgramCheckerVerdict, UnacceptedProgramCertificate,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_ID: &str = "mpk.release.registry.v0";
const REGISTRY_SHA256: &str = "226baa5e744f2966615a5fe03d6bfa0395db4b191e92bc099e63436fa9936aba";
const RUST_FRONTEND: &str = "frontend.rust.rust2vir.candidate.v0";
const RUST_TOOLCHAIN: &str = "toolchain.rust.nightly-2025-06-01.candidate.v0";
const UPDATE_PROGRAM_CERTIFICATE_ENV: &str = "MPK_UPDATE_PROGRAM_CERTIFICATE_FIXTURE";

#[test]
fn rust_module_calls_emits_one_dual_checked_zero_axiom_program_certificate() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);
    let argv = rust_verify_argv();

    let first_directory = tempfile::tempdir().unwrap();
    let first = run_fixture(
        first_directory.path(),
        &argv,
        inputs.clone(),
        accepted.clone(),
    );

    assert!(first.invocation.strict);
    assert!(first.evidence.document().verification_options.strict);
    let outcome = first
        .program_certificate
        .as_ref()
        .expect("Rust policy verification retains the program-certificate outcome");
    let candidate = match outcome {
        ProgramCertificateOutcome::Candidate(candidate) => candidate,
        ProgramCertificateOutcome::Pending {
            missing_member_ids, ..
        } => {
            panic!("module-calls unexpectedly remained pending: {missing_member_ids:?}")
        }
        ProgramCertificateOutcome::Unaccepted(candidate) => {
            panic!(
                "module-calls candidate was not dual accepted: {}",
                candidate.failure_detail
            )
        }
    };

    assert_eq!(candidate.generated_declarations.len(), 6);
    assert_eq!(
        candidate.generated_declarations.len(),
        first.skeleton.skeleton().theorem_declarations.len()
    );
    assert_eq!(candidate.rust_report.axiom_count, 0);
    assert_eq!(candidate.reference_report.axiom_count, 0);
    assert_eq!(
        candidate.rust_report.module,
        candidate.reference_report.module
    );
    assert_eq!(
        hash_hex(&candidate.rust_report.certificate_hash),
        candidate.reference_report.certificate_hash
    );
    assert_eq!(
        hash_hex(&candidate.rust_report.export_hash),
        candidate.reference_report.export_hash
    );
    assert_eq!(
        hash_hex(&candidate.rust_report.axiom_report_hash),
        candidate.reference_report.axiom_report_hash
    );
    assert!(candidate.certificate.imports.is_empty());
    assert!(candidate.certificate.proof_node_table.is_empty());
    assert!(candidate.certificate.theory_certificates.is_empty());

    let generated_exports = candidate
        .certificate
        .export_block
        .iter()
        .map(|export| {
            (
                candidate.certificate.name_table[export.name as usize].as_str(),
                hash_hex(&export.declaration_hash),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for declaration in &candidate.generated_declarations {
        assert_eq!(
            generated_exports.get(declaration.name.as_str()),
            Some(&declaration.declaration_hash),
            "generated evidence must use the checked certificate declaration hash"
        );
    }

    let document = first.evidence.document();
    assert_eq!(document.trusted_evidence.certificates.len(), 1);
    let evidence_certificate = &document.trusted_evidence.certificates[0];
    assert_eq!(evidence_certificate.id, "program");
    assert_eq!(evidence_certificate.checked_declarations.len(), 6);
    assert_eq!(
        evidence_certificate.certificate_hash,
        hash_hex(&candidate.rust_report.certificate_hash)
    );
    assert_eq!(
        evidence_certificate.export_hash,
        hash_hex(&candidate.rust_report.export_hash)
    );
    assert_eq!(
        evidence_certificate.axiom_report_hash,
        hash_hex(&candidate.rust_report.axiom_report_hash)
    );
    for declaration in &evidence_certificate.checked_declarations {
        assert_eq!(
            generated_exports.get(declaration.name.as_str()),
            Some(&declaration.declaration_hash)
        );
    }
    assert_eq!(document.trusted_evidence.checker_verdicts.len(), 2);
    assert!(document
        .trusted_evidence
        .checker_verdicts
        .iter()
        .all(|verdict| verdict.verdict == "accepted" && verdict.certificate_ids == ["program"]));
    assert_eq!(
        document
            .trusted_evidence
            .checker_verdicts
            .iter()
            .map(|verdict| verdict.checker.as_str())
            .collect::<Vec<_>>(),
        ["rust_fast_kernel", "reference_checker"]
    );
    match &document.trusted_evidence.axiom_report {
        PolicyAxiomReportV1::Checked {
            axiom_report_hash,
            category_counts,
        } => {
            assert_eq!(
                axiom_report_hash,
                &hash_hex(&candidate.rust_report.axiom_report_hash)
            );
            assert_eq!(category_counts.total_axiom_count, 0);
            assert_eq!(category_counts.core_axiom_count, 0);
            assert_eq!(category_counts.builtin_theory_axiom_count, 0);
            assert_eq!(category_counts.go_semantics_axiom_count, 0);
            assert_eq!(category_counts.external_axiom_count, 0);
        }
        PolicyAxiomReportV1::NotGenerated => panic!("candidate omitted its checked axiom report"),
    }

    let package = PackagePolicyV1 {
        checker_profile: "mvp-strict".to_owned(),
        allowed_axiom_profiles: vec!["mvp-theory".to_owned()],
    };
    let active = ActiveReleasePolicyV1 {
        source_language: "rust".to_owned(),
        semantic_profile: "mpk.rust.checked.v0".to_owned(),
        strategy_profile: "payment-policy-rust-alpha".to_owned(),
        checker_profile: "mvp-strict".to_owned(),
        axiom_profile: "mvp-theory".to_owned(),
    };
    let checked_axiom_report = recomputed_zero_axiom_policy_report(candidate);
    validate_package_release_policy_v1(
        &first.evidence,
        &package,
        &active,
        &first.vc,
        &first.certificate_manifest,
        &checked_axiom_report,
    )
    .unwrap();
    assert_eq!(
        validate_package_release_policy_v1(
            &first.evidence,
            &PackagePolicyV1 {
                checker_profile: "mvp-strict".to_owned(),
                allowed_axiom_profiles: vec!["zero-axiom".to_owned()],
            },
            &active,
            &first.vc,
            &first.certificate_manifest,
            &checked_axiom_report,
        )
        .unwrap_err()
        .code(),
        "POLICY_PACKAGE_PROFILE"
    );
    let frontend_manifest = &first
        .scan
        .frontend
        .envelope
        .artifacts
        .as_ref()
        .expect("accepted frontend artifacts")
        .source_manifest;
    assert_eq!(
        validate_package_release_policy_v1(
            &first.evidence,
            &package,
            &active,
            &first.vc,
            frontend_manifest,
            &checked_axiom_report,
        )
        .unwrap_err()
        .code(),
        "POLICY_PACKAGE_LINKAGE"
    );
    assert_eq!(
        validate_package_release_policy_v1(
            &first.evidence,
            &package,
            &active,
            &first.vc,
            &first.certificate_manifest,
            &PolicyAxiomReportV1::NotGenerated,
        )
        .unwrap_err()
        .code(),
        "POLICY_PACKAGE_AXIOM_REPORT"
    );

    assert_eq!(document.properties.len(), 2);
    assert!(document.properties.iter().all(|property| {
        property.status == "mpk_verified"
            && property.members.iter().all(|member| {
                member.status == "mpk_verified"
                    && member.evidence
                        == [PolicyEvidenceReferenceV1::CheckedDeclaration {
                            certificate_id: "program".to_owned(),
                        }]
            })
    }));

    let retained_manifest = candidate
        .certificate
        .source_manifest
        .as_ref()
        .expect("program certificate embeds the certificate-stage manifest");
    assert_eq!(
        retained_manifest.payload,
        first.certificate_manifest.canonical_bytes()
    );
    assert_eq!(retained_manifest, &first.certificate_source_manifest);
    assert_eq!(
        document.certificate_source_manifest_hash,
        first.certificate_manifest.hash().as_str()
    );
    let manifest: Value = serde_json::from_slice(&retained_manifest.payload).unwrap();
    assert_eq!(manifest["vc_hash"], first.vc.hash().as_str());

    let second_directory = tempfile::tempdir().unwrap();
    let second = run_fixture(second_directory.path(), &argv, inputs, accepted);
    let second_candidate = match second.program_certificate.as_ref().unwrap() {
        ProgramCertificateOutcome::Candidate(candidate) => candidate,
        ProgramCertificateOutcome::Pending {
            missing_member_ids, ..
        } => {
            panic!("repeat unexpectedly remained pending: {missing_member_ids:?}")
        }
        ProgramCertificateOutcome::Unaccepted(candidate) => {
            panic!(
                "repeat candidate was not dual accepted: {}",
                candidate.failure_detail
            )
        }
    };
    assert_eq!(candidate.bytes, second_candidate.bytes);
    assert_eq!(
        first.evidence.canonical_bytes(),
        second.evidence.canonical_bytes()
    );
    assert_eq!(
        fs::read(first_directory.path().join("evidence.json")).unwrap(),
        first.evidence.canonical_bytes()
    );
    assert!(first_directory.path().join("evidence.md").is_file());
    assert_program_certificate_fixtures(candidate, first.certificate_manifest.canonical_bytes());
}

#[test]
fn deterministic_checker_rejection_or_acceptance_disagreement_is_published_then_fails() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);

    for (rust_verdict, reference_verdict, failure_kind, expected_code) in [
        (
            ProgramCheckerVerdict::Rejected,
            ProgramCheckerVerdict::Rejected,
            ProgramCertificateErrorKind::CheckerRejected,
            "POLICY_CHECKER_REJECTED",
        ),
        (
            ProgramCheckerVerdict::Accepted,
            ProgramCheckerVerdict::Rejected,
            ProgramCertificateErrorKind::CheckerDisagreement,
            "POLICY_CHECKER_DISAGREEMENT",
        ),
        (
            ProgramCheckerVerdict::Rejected,
            ProgramCheckerVerdict::Accepted,
            ProgramCertificateErrorKind::CheckerDisagreement,
            "POLICY_CHECKER_DISAGREEMENT",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let error = run_policy_verify_v1_with_assembler(
            &rust_verify_argv(),
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| Ok(accepted.clone()),
            |_| false,
            |vc, skeleton, source_manifest| {
                let outcome = assemble_program_certificate_alpha(vc, skeleton, source_manifest)?;
                let ProgramCertificateOutcome::Candidate(candidate) = outcome else {
                    panic!("module-calls should produce the injectable candidate");
                };
                let CheckedProgramCertificate {
                    bytes,
                    certificate,
                    rust_report,
                    reference_report,
                    generated_declarations,
                } = *candidate;
                Ok(ProgramCertificateOutcome::Unaccepted(Box::new(
                    UnacceptedProgramCertificate {
                        bytes,
                        certificate,
                        rust_report: (rust_verdict == ProgramCheckerVerdict::Accepted)
                            .then_some(rust_report),
                        reference_report: (reference_verdict == ProgramCheckerVerdict::Accepted)
                            .then_some(reference_report),
                        rust_verdict,
                        reference_verdict,
                        failure_kind,
                        failure_detail: "injected deterministic checker outcome".to_owned(),
                        generated_declarations,
                    },
                )))
            },
        )
        .unwrap_err();
        assert_eq!(error.code(), expected_code);

        let evidence_bytes = fs::read(directory.path().join("evidence.json")).unwrap();
        let evidence: PolicyEvidenceV1 = serde_json::from_slice(&evidence_bytes).unwrap();
        assert_eq!(evidence.trusted_evidence.certificates.len(), 1);
        assert!(matches!(
            evidence.trusted_evidence.axiom_report,
            PolicyAxiomReportV1::Checked { .. }
        ));
        assert_eq!(
            evidence
                .trusted_evidence
                .checker_verdicts
                .iter()
                .map(|verdict| (verdict.verdict.as_str(), verdict.certificate_ids.as_slice()))
                .collect::<Vec<_>>(),
            [
                (rust_verdict.as_str(), ["program".to_owned()].as_slice()),
                (
                    reference_verdict.as_str(),
                    ["program".to_owned()].as_slice()
                ),
            ]
        );
        assert!(evidence.properties.iter().all(|property| {
            property.status == "proof_pending"
                && property.members.iter().all(|member| {
                    member.status == "proof_pending"
                        && member.evidence
                            == [PolicyEvidenceReferenceV1::HelperArtifact {
                                artifact_id: "vc".to_owned(),
                            }]
                })
        }));
        assert!(directory.path().join("evidence.md").is_file());
    }
}

#[test]
fn checker_execution_outcome_is_rejected_before_evidence_publication() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);
    let directory = tempfile::tempdir().unwrap();

    let error = run_policy_verify_v1_with_assembler(
        &rust_verify_argv(),
        directory.path(),
        inputs,
        |_| Ok(()),
        |(), _| Ok(accepted.clone()),
        |_| false,
        |vc, skeleton, source_manifest| {
            let outcome = assemble_program_certificate_alpha(vc, skeleton, source_manifest)?;
            let ProgramCertificateOutcome::Candidate(candidate) = outcome else {
                panic!("module-calls should produce the injectable candidate");
            };
            let CheckedProgramCertificate {
                bytes,
                certificate,
                generated_declarations,
                ..
            } = *candidate;
            Ok(ProgramCertificateOutcome::Unaccepted(Box::new(
                UnacceptedProgramCertificate {
                    bytes,
                    certificate,
                    rust_report: None,
                    reference_report: None,
                    rust_verdict: ProgramCheckerVerdict::Rejected,
                    reference_verdict: ProgramCheckerVerdict::Rejected,
                    failure_kind: ProgramCertificateErrorKind::CheckerExecution,
                    failure_detail: "injected checker execution failure".to_owned(),
                    generated_declarations,
                },
            )))
        },
    )
    .unwrap_err();

    assert_eq!(error.code(), "POLICY_CHECKER_EXECUTION");
    assert!(!directory.path().join("evidence.json").exists());
    assert!(!directory.path().join("evidence.md").exists());
}

#[test]
fn candidate_evidence_is_derived_from_exact_bytes_and_certificate_stage_manifest() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);

    for (mutation, expected_code) in [
        ("generated_hash", "POLICY_CERTIFICATE_ASSEMBLY"),
        ("certificate_object", "POLICY_CERTIFICATE_ASSEMBLY"),
        ("source_manifest", "POLICY_CERTIFICATE_ASSEMBLY"),
        (
            "certificate_hash_placeholder",
            "POLICY_CERTIFICATE_ASSEMBLY",
        ),
        ("axiom_report", "POLICY_CERTIFICATE_AXIOM_REPORT"),
        ("checker_report", "POLICY_CHECKER_DISAGREEMENT"),
        ("reference_checker_report", "POLICY_CHECKER_DISAGREEMENT"),
        ("checker_reports_unbound", "POLICY_CHECKER_EXECUTION"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let error = run_policy_verify_v1_with_assembler(
            &rust_verify_argv(),
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| Ok(accepted.clone()),
            |_| false,
            |vc, skeleton, source_manifest| {
                let outcome = assemble_program_certificate_alpha(vc, skeleton, source_manifest)?;
                let ProgramCertificateOutcome::Candidate(mut candidate) = outcome else {
                    panic!("module-calls should produce the injectable candidate");
                };
                match mutation {
                    "generated_hash" => {
                        candidate.generated_declarations[0].declaration_hash = "00".repeat(32);
                    }
                    "certificate_object" => {
                        candidate.certificate.module = "Policy.Mutated".to_owned();
                    }
                    "source_manifest" => {
                        candidate
                            .certificate
                            .source_manifest
                            .as_mut()
                            .expect("candidate source manifest")
                            .payload
                            .push(b' ');
                        candidate.bytes = encode_certificate(&candidate.certificate);
                    }
                    "certificate_hash_placeholder" => {
                        candidate.certificate.hashes.certificate_hash = [1; 32];
                        candidate.bytes = encode_certificate(&candidate.certificate);
                        let exact_hash = certificate_hash(&candidate.bytes);
                        candidate.rust_report.certificate_hash = exact_hash;
                        candidate.reference_report.certificate_hash = hash_hex(&exact_hash);
                    }
                    "axiom_report" => {
                        let declaration = candidate
                            .certificate
                            .declarations
                            .last_mut()
                            .expect("candidate declaration");
                        let ty = match &declaration.kind {
                            DeclarationKind::Theorem { ty, .. } => *ty,
                            _ => panic!("last generated declaration should be a theorem"),
                        };
                        declaration.kind = DeclarationKind::Axiom { ty };
                        candidate.certificate.export_block =
                            build_export_block(&candidate.certificate).unwrap();
                        candidate.certificate.axiom_report =
                            build_axiom_report(&candidate.certificate).unwrap();
                        candidate.certificate.hashes.export_hash =
                            export_block_hash(&candidate.certificate.export_block);
                        candidate.certificate.hashes.axiom_report_hash =
                            axiom_report_hash_for_report(&candidate.certificate.axiom_report);
                        candidate.bytes = encode_certificate(&candidate.certificate);
                    }
                    "checker_report" => {
                        candidate.rust_report.certificate_hash = [0; 32];
                    }
                    "reference_checker_report" => {
                        candidate.reference_report.certificate_hash = "00".repeat(32);
                    }
                    "checker_reports_unbound" => {
                        candidate.rust_report.certificate_hash = [0; 32];
                        candidate.reference_report.certificate_hash = "00".repeat(32);
                    }
                    _ => unreachable!(),
                }
                Ok(ProgramCertificateOutcome::Candidate(candidate))
            },
        )
        .unwrap_err();

        assert_eq!(error.code(), expected_code, "{mutation}");
        assert!(!directory.path().join("evidence.json").exists());
        assert!(!directory.path().join("evidence.md").exists());
    }
}

#[test]
fn unaccepted_checker_report_shape_is_validated_before_publication() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);

    for (injection, expected_code) in [
        ("missing_rust_report", "POLICY_CHECKER_EXECUTION"),
        ("mismatched_reference_report", "POLICY_CHECKER_EXECUTION"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let error = run_policy_verify_v1_with_assembler(
            &rust_verify_argv(),
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| Ok(accepted.clone()),
            |_| false,
            |vc, skeleton, source_manifest| {
                let outcome = assemble_program_certificate_alpha(vc, skeleton, source_manifest)?;
                let ProgramCertificateOutcome::Candidate(candidate) = outcome else {
                    panic!("module-calls should produce the injectable candidate");
                };
                let CheckedProgramCertificate {
                    bytes,
                    certificate,
                    rust_report: _,
                    mut reference_report,
                    generated_declarations,
                } = *candidate;
                let (rust_report, reference_report, rust_verdict, reference_verdict) =
                    match injection {
                        "missing_rust_report" => (
                            None,
                            None,
                            ProgramCheckerVerdict::Accepted,
                            ProgramCheckerVerdict::Rejected,
                        ),
                        "mismatched_reference_report" => {
                            reference_report.certificate_hash = "00".repeat(32);
                            (
                                None,
                                Some(reference_report),
                                ProgramCheckerVerdict::Rejected,
                                ProgramCheckerVerdict::Accepted,
                            )
                        }
                        _ => unreachable!(),
                    };
                Ok(ProgramCertificateOutcome::Unaccepted(Box::new(
                    UnacceptedProgramCertificate {
                        bytes,
                        certificate,
                        rust_report,
                        reference_report,
                        rust_verdict,
                        reference_verdict,
                        failure_kind: ProgramCertificateErrorKind::CheckerDisagreement,
                        failure_detail: "injected malformed checker-report shape".to_owned(),
                        generated_declarations,
                    },
                )))
            },
        )
        .unwrap_err();

        assert_eq!(error.code(), expected_code, "{injection}");
        assert!(!directory.path().join("evidence.json").exists());
        assert!(!directory.path().join("evidence.md").exists());
    }
}

fn recomputed_zero_axiom_policy_report(
    candidate: &CheckedProgramCertificate,
) -> PolicyAxiomReportV1 {
    let decoded = decode_canonical_certificate(&candidate.bytes).unwrap();
    assert_eq!(decoded, candidate.certificate);
    let report = build_axiom_report(&decoded).unwrap();
    assert_eq!(report, candidate.rust_report.axiom_report);
    let report_hash = axiom_report_hash_for_report(&report);
    assert_eq!(
        hash_hex(&report_hash),
        candidate.reference_report.axiom_report_hash
    );
    assert_eq!(report.summary.total_axiom_count, 0);

    PolicyAxiomReportV1::Checked {
        axiom_report_hash: hash_hex(&report_hash),
        category_counts: PolicyAxiomCategoryCountsV1 {
            total_axiom_count: i64::try_from(report.summary.total_axiom_count).unwrap(),
            core_axiom_count: i64::try_from(report.summary.core_axiom_count).unwrap(),
            builtin_theory_axiom_count: i64::try_from(report.summary.builtin_theory_axiom_count)
                .unwrap(),
            go_semantics_axiom_count: i64::try_from(report.summary.go_semantics_axiom_count)
                .unwrap(),
            external_axiom_count: i64::try_from(report.summary.external_axiom_count).unwrap(),
        },
    }
}

fn assert_program_certificate_fixtures(
    candidate: &CheckedProgramCertificate,
    certificate_manifest: &[u8],
) {
    let fixture_root = repo_root().join("fixtures/program-certificate");
    let hashes = format!(
        "fixture,export_hash,axiom_report_hash,certificate_hash\nalpha-module-calls,{},{},{}\n",
        hash_hex(&candidate.rust_report.export_hash),
        hash_hex(&candidate.rust_report.axiom_report_hash),
        hash_hex(&candidate.rust_report.certificate_hash),
    );
    let fixtures = [
        (
            fixture_root.join("alpha-module-calls.source-manifest.certificate.json"),
            certificate_manifest.to_vec(),
        ),
        (
            fixture_root.join("alpha-module-calls.hex"),
            certificate_hex(&candidate.bytes),
        ),
        (fixture_root.join("hashes.csv"), hashes.into_bytes()),
    ];
    for (path, bytes) in fixtures {
        if std::env::var_os(UPDATE_PROGRAM_CERTIFICATE_ENV).is_some() {
            fs::write(&path, bytes)
                .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        } else {
            assert_eq!(
                fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
                bytes,
                "fixture {} is stale; rerun with {UPDATE_PROGRAM_CERTIFICATE_ENV}=1",
                path.display()
            );
        }
    }
}

fn certificate_hex(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = Vec::with_capacity(bytes.len() * 2 + bytes.len().div_ceil(32));
    for chunk in bytes.chunks(32) {
        for byte in chunk {
            output.push(HEX[usize::from(byte >> 4)]);
            output.push(HEX[usize::from(byte & 0x0f)]);
        }
        output.push(b'\n');
    }
    output
}

fn run_fixture(
    root: &Path,
    argv: &[String],
    inputs: Vec<OwnedCapturedInput>,
    accepted: AcceptedFrontendRun,
) -> policy_verify_v1::PolicyVerifyV1RunOutput {
    run_policy_verify_v1_with(
        argv,
        root,
        inputs,
        |_| Ok(()),
        |(), _| Ok(accepted.clone()),
        |_| false,
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

fn captured_case_inputs(case: &Path, envelope_path: &Path) -> Vec<OwnedCapturedInput> {
    let envelope: Value = serde_json::from_slice(&fs::read(envelope_path).unwrap()).unwrap();
    envelope["source_manifest"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| {
            let normalized_path = input["normalized_path"].as_str().unwrap();
            let source = match normalized_path {
                "Cargo.toml" | "Cargo.lock" => repo_root()
                    .join("fixtures/rust-basic")
                    .join(normalized_path),
                _ => case.join("source").join(normalized_path),
            };
            OwnedCapturedInput {
                kind: match input["kind"].as_str().unwrap() {
                    "source" => InputKind::Source,
                    "contract" => InputKind::Contract,
                    "build_manifest" => InputKind::BuildManifest,
                    "lockfile" => InputKind::Lockfile,
                    kind => panic!("unknown fixture input kind {kind}"),
                },
                normalized_path: normalized_path.to_owned(),
                bytes: fs::read(source).unwrap(),
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

fn rust_verify_argv() -> Vec<String> {
    [
        "mpk",
        "policy",
        "verify",
        "source",
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
        "vector",
        "--function",
        "vector::cross_module",
        "--contract",
        "contracts/selected.json",
        "--contract",
        "contracts/public.json",
        "--contract",
        "contracts/private.json",
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
        "--strict",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
