use mpk_cert::encode::SourceManifest as CertificateSourceManifest;
use mpk_cert::{certificate_hash, decode_canonical_certificate, hash_hex};
use mpk_cli::policy_profile::lookup_strategy_registration;
use mpk_cli::policy_schema::{PolicyEvidenceV1, PolicyHelperArtifact, PolicyVerificationOptions};
use mpk_cli::program_certificate::{assemble_program_certificate_alpha, ProgramCertificateOutcome};
use mpk_cli::successor_policy::{
    generate_successor_policy_scan, import_successor_policy_scan_json, run_successor_policy,
    SuccessorPolicyCode, SuccessorPolicyPhase, SuccessorPolicyScanSource,
    SUCCESSOR_POLICY_EVIDENCE_SCHEMA, SUCCESSOR_POLICY_SCAN_SCHEMA,
    SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE,
};
use mpk_vc::semantic_profile_registry::CompiledSemanticProfile;
use mpk_vc::{sha256_raw_file_bytes, SemanticParameters, SemanticProfile, VcDocument};
use serde_json::Value;

#[path = "support/successor_policy.rs"]
mod successor_policy_support;

use successor_policy_support::*;

#[test]
fn csharp_policy_reaches_identical_byte_dual_checker_certificate_and_closed_evidence() {
    let registry = registry();
    let source = csharp_source(&registry);
    let vc_contract = profile_contract("csharp", "vc");
    let policy_contract = profile_contract("csharp", "policy");
    let evidence_contract = profile_contract("csharp", "evidence");
    let pair = generated_pair(&registry, &source, &vc_contract);
    let captured = captured_refs(&source.storage);
    let boundary = source_boundary(
        &registry,
        &source,
        &pair,
        &policy_contract,
        &evidence_contract,
        &captured,
    );
    let options = PolicyVerificationOptions {
        strict: true,
        update_fixtures: false,
    };
    let first =
        run_successor_policy(boundary, options.clone()).expect("C# successor policy verification");
    let second =
        run_successor_policy(boundary, options).expect("repeat C# successor policy verification");

    assert_eq!(
        first.scan().canonical_bytes(),
        second.scan().canonical_bytes()
    );
    assert_eq!(
        first.evidence().canonical_bytes(),
        second.evidence().canonical_bytes()
    );
    assert_eq!(
        first.scan().document().schema(),
        SUCCESSOR_POLICY_SCAN_SCHEMA
    );
    assert_eq!(
        first.evidence().document().schema(),
        SUCCESSOR_POLICY_EVIDENCE_SCHEMA
    );
    assert_eq!(
        first.evidence().document().program_certificate_profile(),
        SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE
    );
    assert_eq!(
        first.registration().profile(),
        CompiledSemanticProfile::CSharpScalarV0
    );
    assert_eq!(
        first.registration().strategy_profile(),
        "payment-policy-csharp-alpha"
    );
    assert_eq!(first.registration().checker_profile(), "mvp-strict");
    assert_eq!(first.registration().axiom_profile(), "mvp-theory");
    assert_eq!(
        first.registration().recipe_profile_id(),
        "mpk.csharp.evidence_recipe.v0"
    );
    let expected_contract_helper = PolicyHelperArtifact::Contract {
        id: format!("contract:{CSHARP_FUNCTION}"),
        normalized_path: "contracts/identity.json".to_owned(),
        schema: "mpk.csharp.contract.v0".to_owned(),
        raw_input_sha256: sha256_raw_file_bytes(CSHARP_CONTRACT).to_hex(),
        function_id: CSHARP_FUNCTION.to_owned(),
        contract_hash: source.vir.module().units()[0].functions()[0]
            .contracts()
            .contract_hash()
            .as_str()
            .to_owned(),
    };
    assert!(first
        .scan()
        .document()
        .helper_artifacts()
        .contains(&expected_contract_helper));

    let ProgramCertificateOutcome::Candidate(candidate) = first.program_certificate() else {
        panic!("the reflexive C# obligation must produce a dual-accepted candidate");
    };
    assert_eq!(candidate.rust_report.axiom_count, 0);
    assert_eq!(candidate.reference_report.axiom_count, 0);
    let certificate_digest = hash_hex(&certificate_hash(&candidate.bytes));
    assert_eq!(
        hash_hex(&candidate.rust_report.certificate_hash),
        certificate_digest
    );
    assert_eq!(
        candidate.reference_report.certificate_hash,
        certificate_digest
    );
    assert_eq!(
        hash_hex(&candidate.rust_report.export_hash),
        candidate.reference_report.export_hash
    );
    assert_eq!(
        hash_hex(&candidate.rust_report.axiom_report_hash),
        candidate.reference_report.axiom_report_hash
    );
    let decoded = decode_canonical_certificate(&candidate.bytes).expect("Certificate v0 bytes");
    assert_eq!(decoded, candidate.certificate);
    assert!(decoded.imports.is_empty());
    assert!(decoded.theory_certificates.is_empty());
    assert_eq!(
        decoded
            .source_manifest
            .as_ref()
            .expect("certificate-stage manifest")
            .payload,
        first.certificate_manifest().canonical_bytes()
    );

    let trusted = first.evidence().document().trusted_evidence();
    assert_eq!(trusted.certificates.len(), 1);
    assert!(trusted.theory_certificates.is_empty());
    assert_eq!(trusted.checker_verdicts.len(), 2);
    assert!(trusted.checker_verdicts.iter().all(|verdict| {
        verdict.verdict == "accepted" && verdict.certificate_ids == ["program"]
    }));
    assert!(first
        .evidence()
        .document()
        .properties()
        .iter()
        .flat_map(|property| &property.members)
        .all(|member| {
            member.status == "mpk_verified"
                && member.evidence
                    == [
                        mpk_cli::policy_schema::PolicyEvidenceReferenceV1::CheckedDeclaration {
                            certificate_id: "program".to_owned(),
                        },
                    ]
        }));
    assert!(first
        .evidence()
        .document()
        .reproduction_recipes()
        .iter()
        .all(|recipe| {
            recipe.working_directory_role == "source_root"
                && recipe
                    .argv
                    .contains(&"--profile-registry-sha256".to_owned())
                && recipe.argv.contains(&"--profile-entry-sha256".to_owned())
                && recipe.argv.contains(&"--compilation".to_owned())
                && recipe.argv.contains(&"--method".to_owned())
                && recipe
                    .argv
                    .iter()
                    .all(|argument| !argument.starts_with('/'))
        }));

    let frontend: Value =
        serde_json::from_slice(source.manifest.canonical_bytes()).expect("frontend manifest");
    let mut certificate: Value =
        serde_json::from_slice(first.certificate_manifest().canonical_bytes())
            .expect("certificate manifest");
    let vc_hash = certificate
        .as_object_mut()
        .expect("manifest object")
        .remove("vc_hash")
        .expect("certificate VC hash");
    certificate
        .as_object_mut()
        .expect("manifest object")
        .remove("source_manifest_hash");
    let mut frontend_without_hash = frontend;
    frontend_without_hash
        .as_object_mut()
        .expect("manifest object")
        .remove("source_manifest_hash");
    assert_eq!(certificate, frontend_without_hash);
    assert_eq!(vc_hash, pair.vc.hash().as_str());

    fixture("scan.json", first.scan().canonical_bytes());
    fixture(
        "source-manifest.certificate.json",
        first.certificate_manifest().canonical_bytes(),
    );
    fixture("evidence.json", first.evidence().canonical_bytes());
    fixture("program-certificate.hex", &hex_fixture(&candidate.bytes));

    let mut old_scan = first.scan().canonical_bytes().to_vec();
    old_scan.push(b'\n');
    let error = first.import_scan_json(&old_scan).unwrap_err();
    assert_eq!(error.phase(), SuccessorPolicyPhase::CanonicalTransport);
    assert_eq!(error.code(), SuccessorPolicyCode::CanonicalTransport);

    let mut promoted: Value =
        serde_json::from_slice(first.evidence().canonical_bytes()).expect("evidence JSON");
    promoted["trusted_evidence"]["checker_verdicts"][0]["checker"] =
        Value::String("compiler".to_owned());
    let error = first
        .import_evidence_json(&canonical(&promoted))
        .unwrap_err();
    assert_eq!(error.phase(), SuccessorPolicyPhase::DocumentLinkage);
    assert_eq!(error.code(), SuccessorPolicyCode::DocumentLinkage);

    let mut predecessor: Value =
        serde_json::from_slice(first.evidence().canonical_bytes()).expect("evidence JSON");
    predecessor["schema"] = Value::String("mpk.policy.evidence.v1".to_owned());
    assert_eq!(
        first
            .import_evidence_json(&canonical(&predecessor))
            .unwrap_err()
            .code(),
        SuccessorPolicyCode::DocumentLinkage
    );
    assert!(
        serde_json::from_slice::<PolicyEvidenceV1>(first.evidence().canonical_bytes()).is_err()
    );

    let crossed_policy = profile_contract("go", "policy");
    let crossed = source_boundary(
        &registry,
        &source,
        &pair,
        &crossed_policy,
        &evidence_contract,
        &captured,
    );
    let error = run_successor_policy(
        crossed,
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    )
    .unwrap_err();
    assert_eq!(error.phase(), SuccessorPolicyPhase::ProfileContract);
    assert_eq!(error.code(), SuccessorPolicyCode::ProfileContract);

    let crossed_evidence = profile_contract("go", "evidence");
    let crossed = source_boundary(
        &registry,
        &source,
        &pair,
        &policy_contract,
        &crossed_evidence,
        &captured,
    );
    let error = run_successor_policy(
        crossed,
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    )
    .unwrap_err();
    assert_eq!(error.phase(), SuccessorPolicyPhase::ProfileContract);
    assert_eq!(error.code(), SuccessorPolicyCode::ProfileContract);
    assert!(lookup_strategy_registration("payment-policy-csharp-alpha").is_none());
}

#[test]
fn successor_scan_is_frontend_only_and_exactly_reimported() {
    let registry = registry();
    let source = staged_source(
        &registry,
        "fixtures/vir-go/frontend/basic-arith",
        "fixtures/go-basic",
    );
    let policy_contract = profile_contract("go", "policy");
    let captured = captured_refs(&source.storage);
    let boundary = SuccessorPolicyScanSource {
        registry: &registry,
        vir: &source.vir,
        source_map: &source.source_map,
        frontend_manifest: &source.manifest,
        policy_contract: &policy_contract,
        captured_inputs: &captured,
    };

    let scan =
        generate_successor_policy_scan(boundary).expect("frontend-only successor policy scan");
    assert_eq!(scan.document().schema(), SUCCESSOR_POLICY_SCAN_SCHEMA);
    let imported = import_successor_policy_scan_json(scan.canonical_bytes(), boundary)
        .expect("exact successor scan reimport");
    assert_eq!(imported.canonical_bytes(), scan.canonical_bytes());
}

#[test]
fn staged_go_and_rust_keep_their_existing_policy_and_checker_verdicts() {
    let registry = registry();
    for (profile, artifacts, sources, expected_profile, strategy, axiom) in [
        (
            "go",
            "",
            "",
            CompiledSemanticProfile::GoFixedV0,
            "payment-policy-alpha",
            "zero-axiom",
        ),
        (
            "rust",
            "fixtures/rust-basic/positive/module-calls/artifacts",
            "rust-tools/rust2vir/testdata/positive/module-calls/source",
            CompiledSemanticProfile::RustCheckedV0,
            "payment-policy-rust-alpha",
            "mvp-theory",
        ),
    ] {
        let source = if profile == "go" {
            go_identity_source(&registry)
        } else {
            staged_source(&registry, artifacts, sources)
        };
        let vc_contract = profile_contract(profile, "vc");
        let policy_contract = profile_contract(profile, "policy");
        let evidence_contract = profile_contract(profile, "evidence");
        let pair = generated_pair(&registry, &source, &vc_contract);
        let captured = captured_refs(&source.storage);
        let run = run_successor_policy(
            source_boundary(
                &registry,
                &source,
                &pair,
                &policy_contract,
                &evidence_contract,
                &captured,
            ),
            PolicyVerificationOptions {
                strict: true,
                update_fixtures: false,
            },
        )
        .unwrap_or_else(|error| panic!("{profile} successor policy: {error}"));
        assert_eq!(run.registration().profile(), expected_profile);
        assert_eq!(run.registration().strategy_profile(), strategy);
        assert_eq!(run.registration().checker_profile(), "mvp-strict");
        assert_eq!(run.registration().axiom_profile(), axiom);
        assert_eq!(run.scan().document().schema(), SUCCESSOR_POLICY_SCAN_SCHEMA);
        assert_eq!(
            run.evidence().document().schema(),
            SUCCESSOR_POLICY_EVIDENCE_SCHEMA
        );
        let context = source.vir.module().semantic_context();
        let active_vc = VcDocument {
            schema: "mpk.vc.v1".to_owned(),
            source_ir_schema: "mpk.vir.v0".to_owned(),
            source_ir_hash: pair.vc.document().source_ir_hash().as_str().to_owned(),
            input_set_hash: pair.vc.document().input_set_hash().as_str().to_owned(),
            semantic_profile: match expected_profile {
                CompiledSemanticProfile::GoFixedV0 => SemanticProfile::GoFixedV0,
                CompiledSemanticProfile::RustCheckedV0 => SemanticProfile::RustCheckedV0,
                CompiledSemanticProfile::CSharpScalarV0 => unreachable!("loop has no C# case"),
            },
            semantic_parameters: serde_json::from_value::<SemanticParameters>(
                context.semantic_parameters().value().clone(),
            )
            .expect("active semantic parameters"),
            verification_limit_profile: pair.vc.document().verification_limit_profile().to_owned(),
            functions: pair.vc.document().functions().to_vec(),
            vc_hash: ZERO_SHA256.to_owned(),
        };
        let active_outcome = assemble_program_certificate_alpha(
            &active_vc,
            pair.skeleton.skeleton().theorem_declarations(),
            CertificateSourceManifest {
                payload: run.certificate_manifest().canonical_bytes().to_vec(),
            },
        )
        .unwrap_or_else(|error| panic!("{profile} active certificate assembly: {error}"));
        assert_eq!(
            run.program_certificate(),
            &active_outcome,
            "{profile} successor verdict or certificate bytes differ from the active assembler"
        );

        let selected_function = source.manifest.manifest().selection().value()["function"]
            .as_str()
            .expect("selected function");
        assert!(run
            .evidence()
            .document()
            .properties()
            .iter()
            .flat_map(|property| &property.members)
            .all(|member| member.function_id == selected_function));
        match run.program_certificate() {
            ProgramCertificateOutcome::Candidate(candidate) => {
                assert_eq!(candidate.rust_report.axiom_count, 0);
                assert_eq!(candidate.reference_report.axiom_count, 0);
                assert!(run
                    .evidence()
                    .document()
                    .trusted_evidence()
                    .checker_verdicts
                    .iter()
                    .all(|verdict| verdict.verdict == "accepted"));
            }
            outcome => panic!("{profile} staged verdict changed: {outcome:?}"),
        }
    }
}

#[test]
fn csharp_policy_owner_is_registered_for_every_consumed_frozen_vector() {
    let manifest = load("develop/specs/vectors/manifest.json");
    for path in [
        "develop/specs/vectors/csharp-profile-v0.json",
        "develop/specs/vectors/semantic-profile-registry-v1.json",
        "develop/specs/vectors/semantic-profile-registry-v2.json",
    ] {
        let record = manifest["vectors"]
            .as_array()
            .expect("vector manifest records")
            .iter()
            .find(|record| record["path"] == path)
            .unwrap_or_else(|| panic!("missing vector manifest record {path}"));
        assert!(record["implementation_test_owners"]
            .as_array()
            .expect("implementation owners")
            .iter()
            .any(|owner| owner == "crates/mpk-cli/tests/csharp_policy_verify.rs"));
    }
}
