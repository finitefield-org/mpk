#![cfg_attr(not(target_os = "linux"), allow(unused_imports, dead_code))]

use mpk_cert::encode::SourceManifest as CertificateSourceManifest;
use mpk_cli::program_certificate::{assemble_program_certificate_alpha, ProgramCertificateOutcome};
use mpk_cli::successor_policy::{
    generate_successor_policy_scan, import_successor_policy_scan_json, run_successor_policy,
    PolicyVerificationOptions, SuccessorPolicyScanSource, SUCCESSOR_POLICY_EVIDENCE_SCHEMA,
    SUCCESSOR_POLICY_SCAN_SCHEMA,
};
use mpk_vc::semantic_profile_registry::CompiledSemanticProfile;
use mpk_vc::{SemanticParameters, SemanticProfile, VcDocument};

#[path = "support/successor_policy.rs"]
mod successor_policy_support;

use successor_policy_support::*;

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
#[cfg(target_os = "linux")]
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
fn go_rust_policy_owner_is_registered_for_every_consumed_frozen_vector() {
    let manifest = load("develop/specs/vectors/manifest.json");
    {
        let path = "develop/specs/vectors/semantic-profile-registry-v1.json";
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
            .any(|owner| owner == "crates/mpk-cli/tests/successor_policy_verify.rs"));
    }
}
