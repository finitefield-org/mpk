//! JAVA-03-T08 owner: private Java VC/policy/evidence/certificate/AI/API integration.

#![cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64")),
    allow(unused_imports, dead_code)
)]

#[path = "support/java_lowering.rs"]
#[allow(dead_code)]
mod java_lowering;
#[path = "support/successor_policy.rs"]
mod successor_policy_support;

use mpk_api::successor_api::{
    SuccessorApiErrorCode, SuccessorApiService, SuccessorFrontendArtifactStore,
    SuccessorFrontendArtifacts, SuccessorSessionSourceState, SUCCESSOR_AI_API_PROFILE,
};
use mpk_api::SessionId;
use mpk_cert::{certificate_hash, decode_canonical_certificate, hash_hex};
use mpk_cli::policy_profile::lookup_strategy_registration;
use mpk_cli::program_certificate::ProgramCertificateOutcome;
use mpk_cli::reference_checker::execute_reference_checker;
use mpk_cli::successor_ai_explain::{
    build_successor_ai_explanation, prepare_successor_ai_explanation, ExplainLanguageV1,
    SuccessorAiCode, SuccessorAiProviderProvenance, SuccessorAiReportRequest, SuccessorAiSource,
    DEFAULT_GEMINI_MODEL,
};
use mpk_cli::successor_policy::{
    run_successor_policy, PolicyVerificationOptions, SuccessorPolicyCode, SuccessorPolicyPhase,
};
use mpk_kernel::verify_certificate_bytes;
use mpk_vc::semantic_profile_registry::CompiledSemanticProfile;
use mpk_vc::sha256_raw_file_bytes;
use serde_json::{json, Value};

use successor_policy_support::{
    ai_contract, candidate_registry, canonical, captured_refs, checked_in_json, generated_pair,
    java_source, java_source_with_callee_preconditions, profile_contract, source_boundary,
};

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = canonical(value);
    bytes.push(b'\n');
    bytes
}

fn response_value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("canonical API response")
}

fn start_request(source: &successor_policy_support::ValidatedSource) -> Value {
    json!({
        "api_profile":SUCCESSOR_AI_API_PROFILE,
        "module_name":"Example.Java.PrivateCandidate",
        "proof_profile":"mvp-strict",
        "semantic_context":source.vir.module().semantic_context(),
        "selection":source.manifest.manifest().selection()
    })
}

fn import_request(session_id: &str, source: &successor_policy_support::ValidatedSource) -> Value {
    json!({
        "api_profile":SUCCESSOR_AI_API_PROFILE,
        "session_id":session_id,
        "semantic_context":source.vir.module().semantic_context(),
        "selection":source.manifest.manifest().selection(),
        "source_ir_schema":"mpk.vir.v1",
        "source_ir_hash":source.vir.hash().as_str(),
        "vir":serde_json::from_slice::<Value>(source.vir.canonical_bytes()).expect("Java VIR JSON")
    })
}

fn generate_request(session_id: &str, source: &successor_policy_support::ValidatedSource) -> Value {
    json!({
        "api_profile":SUCCESSOR_AI_API_PROFILE,
        "session_id":session_id,
        "semantic_context":source.vir.module().semantic_context(),
        "selection":source.manifest.manifest().selection(),
        "source_ir_schema":"mpk.vir.v1",
        "source_ir_hash":source.vir.hash().as_str(),
        "source_manifest_schema":"mpk.source_manifest.v1",
        "source_manifest_hash":source.manifest.hash().as_str(),
        "input_set_hash":source.manifest.manifest().input_set_hash().as_str()
    })
}

#[test]
fn t08_vectors_are_owned_while_installed_java_activation_remains_closed() {
    let manifest = checked_in_json("develop/specs/vectors/manifest.json");
    for path in [
        "develop/specs/vectors/ai-api-v1.json",
        "develop/specs/vectors/ai-explain-v1.json",
        "develop/specs/vectors/java-profile-v0.json",
        "develop/specs/vectors/semantic-profile-registry-v3.json",
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
            .any(|owner| owner == "crates/mpk-cli/tests/java_policy_verify.rs"));
    }

    let installed = successor_policy_support::registry();
    assert!(installed.lookup("java", "mpk.java.scalar.v0").is_none());
    assert!(candidate_registry()
        .lookup("java", "mpk.java.scalar.v0")
        .is_some());
    assert!(lookup_strategy_registration("payment-policy-java-alpha").is_none());
    assert!(
        !successor_policy_support::repository_path("release/bundles/candidates/java.json").exists()
    );
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache, local Linux amd64 image, and embedded reference checker; runs offline"]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn pinned_java_reaches_same_byte_certificate_and_private_consumers() {
    let report = java_lowering::run();
    let case = java_lowering::case(report, "accepted/call.direct");
    let registry = candidate_registry();
    let source = java_source(&registry, case);
    let vc_contract = profile_contract("java", "vc");
    let policy_contract = profile_contract("java", "policy");
    let evidence_contract = profile_contract("java", "evidence");
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
    let first = run_successor_policy(boundary, options.clone()).expect("Java policy verification");
    let second =
        run_successor_policy(boundary, options).expect("repeated Java policy verification");

    assert_eq!(
        first.registration().profile(),
        CompiledSemanticProfile::JavaScalarV0
    );
    assert_eq!(
        first.registration().strategy_profile(),
        "payment-policy-java-alpha"
    );
    assert_eq!(first.registration().checker_profile(), "mvp-strict");
    assert_eq!(first.registration().axiom_profile(), "mvp-theory");
    assert_eq!(
        first.registration().recipe_profile_id(),
        "mpk.java.evidence_recipe.v0"
    );
    assert_eq!(
        first.scan().canonical_bytes(),
        second.scan().canonical_bytes()
    );
    assert_eq!(
        first.evidence().canonical_bytes(),
        second.evidence().canonical_bytes()
    );
    assert_eq!(first.program_certificate(), second.program_certificate());

    let caller = pair
        .vc
        .document()
        .functions()
        .iter()
        .find(|function| function.function_id == "vector.Case::f(int)->int")
        .expect("Java caller VC");
    assert!(caller
        .members
        .iter()
        .any(|member| member.kind.as_str() == "callee_panic_free"));
    assert!(pair
        .vc
        .document()
        .functions()
        .iter()
        .all(|function| function
            .members
            .iter()
            .any(|member| member.kind.as_str() == "postcondition")));

    let precondition_source = java_source_with_callee_preconditions(&registry, case);
    let precondition_pair = generated_pair(&registry, &precondition_source, &vc_contract);
    let precondition_captured = captured_refs(&precondition_source.storage);
    let precondition_run = run_successor_policy(
        source_boundary(
            &registry,
            &precondition_source,
            &precondition_pair,
            &policy_contract,
            &evidence_contract,
            &precondition_captured,
        ),
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    )
    .expect("Java callee-precondition verification");
    let caller = precondition_pair
        .vc
        .document()
        .functions()
        .iter()
        .find(|function| function.function_id == "vector.Case::f(int)->int")
        .expect("Java precondition caller VC");
    assert!(caller
        .members
        .iter()
        .any(|member| member.kind.as_str() == "callee_precondition"));
    assert!(matches!(
        precondition_run.program_certificate(),
        ProgramCertificateOutcome::Candidate(_)
    ));

    let ProgramCertificateOutcome::Candidate(candidate) = first.program_certificate() else {
        panic!("Java body and call obligations must produce a checked certificate")
    };
    let certificate_sha256 = hash_hex(&certificate_hash(&candidate.bytes));
    assert_eq!(
        hash_hex(&candidate.rust_report.certificate_hash),
        certificate_sha256
    );
    assert_eq!(
        candidate.reference_report.certificate_hash,
        certificate_sha256
    );
    assert_eq!(candidate.rust_report.axiom_count, 0);
    assert_eq!(candidate.reference_report.axiom_count, 0);
    assert_eq!(
        decode_canonical_certificate(&candidate.bytes).expect("Certificate v0 bytes"),
        candidate.certificate
    );
    verify_certificate_bytes(&candidate.bytes).expect("same bytes pass Rust checker");
    let reference =
        execute_reference_checker(&candidate.bytes).expect("same bytes reach reference checker");
    assert_eq!(reference.status_code(), Some(0));
    assert!(reference.stderr().is_empty());
    let reference_json: Value =
        serde_json::from_slice(reference.stdout()).expect("reference checker receipt");
    assert_eq!(reference_json["verdict"], "accepted");
    assert_eq!(reference_json["hashes"]["certificate"], certificate_sha256);

    let mut tampered = candidate.bytes.clone();
    tampered[0] ^= 1;
    assert!(verify_certificate_bytes(&tampered).is_err());
    let reference = execute_reference_checker(&tampered).expect("tamper reaches reference checker");
    assert_eq!(reference.status_code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(reference.stdout()).expect("reference rejection")
            ["verdict"],
        "rejected"
    );

    let evidence: Value =
        serde_json::from_slice(first.evidence().canonical_bytes()).expect("Java evidence JSON");
    let manifest: Value =
        serde_json::from_slice(source.manifest.canonical_bytes()).expect("Java manifest JSON");
    for field in ["release_registry", "frontend", "toolchain"] {
        assert_eq!(
            evidence[field], manifest[field],
            "{field} identity propagation"
        );
    }
    assert_eq!(evidence["semantic_context"], manifest["semantic_context"]);
    assert_eq!(evidence["selection"], manifest["selection"]);
    assert!(evidence["trusted_evidence"]["checker_verdicts"]
        .as_array()
        .expect("checker verdicts")
        .iter()
        .all(|verdict| verdict["verdict"] == "accepted"));

    let mut rehashed_evidence = evidence.clone();
    rehashed_evidence["trusted_evidence"]["certificates"][0]["certificate_hash"] =
        Value::String(hash_hex(&certificate_hash(&tampered)));
    assert_eq!(
        first
            .import_evidence_json(&canonical(&rehashed_evidence))
            .expect_err("rehashed altered evidence must not detach from regeneration")
            .code(),
        SuccessorPolicyCode::DocumentLinkage
    );
    let crossed_policy = profile_contract("csharp", "policy");
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
    .expect_err("crossed policy context must reject");
    assert_eq!(error.phase(), SuccessorPolicyPhase::ProfileContract);
    assert_eq!(error.code(), SuccessorPolicyCode::ProfileContract);

    let java_ai_contract = ai_contract("java");
    let ai_source = SuccessorAiSource {
        registry: &registry,
        evidence: first.evidence(),
        ai_contract: &java_ai_contract,
    };
    let prepared = prepare_successor_ai_explanation(ai_source, ExplainLanguageV1::En)
        .expect("private Java AI projection");
    assert_eq!(prepared.registration().display_language(), "Java");
    assert_eq!(
        prepared.registration().projection_profile_id(),
        "mpk.java.ai_projection.v0"
    );
    assert_eq!(prepared.registration().redaction_profile_id(), "minimal-v1");
    let provider_body =
        std::str::from_utf8(prepared.request_body()).expect("provider request UTF-8");
    for forbidden in [
        "src/vector/Case.java",
        "contracts/c000.json",
        "contracts/c001.json",
        "vector.Case::f(int)->int",
        "vector.Case::g(int)->int",
        source.vir.hash().as_str(),
        source.manifest.hash().as_str(),
        certificate_sha256.as_str(),
        "source_access",
        "proof_authority",
        "certificate_only",
    ] {
        assert!(
            !provider_body.contains(forbidden),
            "provider request leaked {forbidden:?}"
        );
    }
    let request_value: Value = serde_json::from_slice(prepared.canonical_request_bytes())
        .expect("sanitized Java AI request");
    assert!(request_value.get("semantic_context").is_none());
    assert!(request_value.get("selection").is_none());
    let property_refs = request_value["properties"]
        .as_array()
        .expect("sanitized properties")
        .iter()
        .map(|property| property["ref"].as_str().expect("property alias"))
        .collect::<Vec<_>>();
    let provider_response = serde_json::to_vec(&json!({
        "overview":"The supplied MPK evidence is summarized as untrusted helper prose.",
        "property_explanations":property_refs.iter().map(|property_ref| json!({
            "property_ref":property_ref,
            "explanation":"This text does not change the MPK checker verdict."
        })).collect::<Vec<_>>(),
        "limitations":["This response is not proof evidence."],
        "next_steps":["Use the source-free checker receipt for acceptance."]
    }))
    .expect("provider response");
    let report_request = SuccessorAiReportRequest {
        project: "mpk-test-project".to_owned(),
        location: "asia-northeast1".to_owned(),
        requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
        language: ExplainLanguageV1::En,
    };
    let provenance = SuccessorAiProviderProvenance {
        model_version: "gemini-3.5-flash-001".to_owned(),
        response_id: "response-java".to_owned(),
        create_time: "2026-09-01T00:00:00Z".to_owned(),
        finish_reason: "STOP".to_owned(),
        attempts: 1,
        prompt_tokens: Some(128),
        thinking_tokens: Some(0),
        response_tokens: Some(64),
        total_tokens: Some(192),
    };
    let ai_report =
        build_successor_ai_explanation(&prepared, &report_request, &provenance, &provider_response)
            .expect("Java AI helper report");
    assert!(!ai_report.document().proof_evidence());
    assert_eq!(
        ai_report.document().semantic_context(),
        first.evidence().document().semantic_context()
    );
    let mut widened_ai = java_ai_contract.clone();
    widened_ai["value"]["source_access"] = Value::Bool(true);
    assert_eq!(
        prepare_successor_ai_explanation(
            SuccessorAiSource {
                registry: &registry,
                evidence: first.evidence(),
                ai_contract: &widened_ai,
            },
            ExplainLanguageV1::En,
        )
        .expect_err("widened Java AI authority must reject")
        .code(),
        SuccessorAiCode::ProfileContract
    );

    let store = SuccessorFrontendArtifactStore::from_frontend_successes(
        &registry,
        [SuccessorFrontendArtifacts {
            vir: source.vir.clone(),
            source_map: source.source_map.clone(),
            source_manifest: source.manifest.clone(),
            vc_profile_contract: vc_contract.clone(),
        }],
    )
    .expect("private Java API store");
    let stored = store
        .get(source.manifest.hash().as_str())
        .expect("stored Java manifest capability");
    assert_eq!(
        stored.source_manifest().canonical_bytes(),
        source.manifest.canonical_bytes()
    );
    let mut service =
        SuccessorApiService::new(registry.clone(), store).expect("private API service");
    let mut crossed_start = start_request(&source);
    crossed_start["semantic_context"]["profile_registry"]["registry_sha256"] =
        Value::String("0".repeat(64));
    let error = service
        .handle_start_session(&canonical_transport(&crossed_start))
        .expect_err("crossed Java registry root rejects before mutation");
    assert_eq!(error.code, SuccessorApiErrorCode::ContextMismatch);
    assert_eq!(service.mutation_count(), 0);

    let start = response_value(
        &service
            .handle_start_session(&canonical_transport(&start_request(&source)))
            .expect("private Java session starts"),
    );
    assert_eq!(start["session_id"], "s1");
    assert_eq!(start["helper_only"], true);
    service
        .handle_vir_import(&canonical_transport(&import_request("s1", &source)))
        .expect("private Java VIR imports");
    let mut crossed_generate = generate_request("s1", &source);
    crossed_generate["source_manifest_hash"] = Value::String("0".repeat(64));
    let error = service
        .handle_vc_generate(&canonical_transport(&crossed_generate))
        .expect_err("crossed Java manifest capability rejects before mutation");
    assert_eq!(error.code, SuccessorApiErrorCode::SourceContextUnknown);
    assert_eq!(service.mutation_count(), 2);
    let generated = response_value(
        &service
            .handle_vc_generate(&canonical_transport(&generate_request("s1", &source)))
            .expect("private Java VC generates"),
    );
    assert_eq!(generated["semantic_context"], manifest["semantic_context"]);
    assert_eq!(generated["selection"], manifest["selection"]);
    assert_eq!(generated["vc_hash"], pair.vc.hash().as_str());
    assert_eq!(canonical(&generated["vc"]), pair.vc.canonical_bytes());
    assert_eq!(service.mutation_count(), 3);
    assert!(matches!(
        service.source_state(&SessionId("s1".to_owned())),
        Some(SuccessorSessionSourceState::VcGenerated(_))
    ));

    println!(
        "JAVA_T08_RECEIPT {}",
        serde_json::to_string(&json!({
            "schema":"mpk.java.t08.receipt.v0",
            "source_ir_sha256":source.vir.hash().as_str(),
            "source_manifest_sha256":source.manifest.hash().as_str(),
            "vc_sha256":pair.vc.hash().as_str(),
            "certificate_sha256":certificate_sha256,
            "evidence_sha256":sha256_raw_file_bytes(first.evidence().canonical_bytes()).to_hex(),
            "ai_request_sha256":sha256_raw_file_bytes(prepared.canonical_request_bytes()).to_hex(),
            "rust_checker":"accepted",
            "reference_checker":"accepted",
            "axiom_count":0,
            "api_mutations":service.mutation_count(),
            "public_activation":false
        }))
        .expect("receipt JSON")
    );
}
