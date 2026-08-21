//! Normative AI API v1 conformance owner.

use std::collections::BTreeSet;

use mpk_core::TermNode;
use mpk_vc::{
    canonical_json_bytes, import_frontend_source_manifest_json, import_source_map_json,
    parse_strict_json, validate_release_registry, CapturedInput, InputKind,
    SourceManifestValidationContext, SourceMapValidationContext, StrictJsonLimits,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::proof_api::{IntroProofRequest, ReflProofRequest};
use crate::session::{SessionId, StartSessionRequest};
use crate::v1_router::{
    resolve_route, V1ApiError, V1ErrorCode, V1ValidationPhase, AI_API_V1_PROFILE, V1_ROUTES,
};
use crate::vir_api::{
    SessionSourceState, V1ApiService, ValidatedFrontendArtifactStore, ValidatedFrontendArtifacts,
};

const AI_API_VECTOR: &str = include_str!("../../../develop/specs/vectors/ai-api-v1.json");
const SOURCE_MAP_VECTOR: &str = include_str!("../../../develop/specs/vectors/source-map-v0.json");
const SOURCE_MANIFEST_VECTOR: &str =
    include_str!("../../../develop/specs/vectors/source-manifest-v0.json");
const RELEASE_VECTOR: &str = include_str!("../../../develop/specs/vectors/release-bundles-v0.json");

#[test]
fn v1_router_matches_the_frozen_registry_and_rejects_unknown_aliases() {
    let vectors = vectors();
    validate_vector_model(&vectors);
    assert_eq!(vectors["api_profile"], AI_API_V1_PROFILE);
    let registry = vectors["route_registry"]
        .as_array()
        .expect("registry array");
    assert_eq!(registry.len(), V1_ROUTES.len());
    for (expected, actual) in registry.iter().zip(V1_ROUTES) {
        assert_eq!(expected["method"], actual.method);
        assert_eq!(expected["path"], actual.path);
        assert_eq!(expected["handler"], actual.handler.as_str());
        assert!(!actual.path.contains("/removed/"));
    }

    let mut executed = BTreeSet::new();
    for case in vectors["route_cases"].as_array().expect("route cases") {
        let id = text(&case["id"]);
        assert!(executed.insert(id));
        let result = resolve_route(text(&case["method"]), text(&case["path"]));
        match text(&case["expect"]["outcome"]) {
            "resolve" => assert_eq!(
                result.expect("route resolves").as_str(),
                text(&case["expect"]["handler"])
            ),
            "reject" => assert_rejection(result.expect_err("route rejects"), &case["expect"]),
            other => panic!("unknown route outcome {other}"),
        }
    }
    assert_eq!(
        executed.len(),
        vectors["route_cases"].as_array().unwrap().len()
    );

    let example = crate::v1_router::V1ApiError::new(
        V1ValidationPhase::Artifact,
        V1ErrorCode::VirHash,
        Some("source_ir_hash"),
        None::<String>,
    );
    assert_eq!(
        serde_json::to_value(example).unwrap(),
        vectors["error_contract"]["operation_example"]
    );
}

fn validate_vector_model(vectors: &Value) {
    assert_exact_fields(
        vectors,
        &[
            "schema",
            "api_profile",
            "dependencies",
            "owner_test",
            "route_registry",
            "error_contract",
            "artifact_contexts",
            "import_fixtures",
            "generate_fixtures",
            "route_cases",
            "import_cases",
            "context_cases",
        ],
    );
    assert_eq!(vectors["schema"], "mpk.ai.api.conformance.v1");
    assert_eq!(vectors["owner_test"], "crates/mpk-api/src/v1_tests.rs");
    assert_exact_fields(
        &vectors["dependencies"],
        &[
            "api_spec",
            "frontend_spec",
            "release_spec",
            "vir_spec",
            "vir_vector",
            "manifest_spec",
            "manifest_vector",
            "source_map_spec",
            "source_map_vector",
            "vc_spec",
            "certificate_spec",
        ],
    );
    assert_exact_fields(
        &vectors["error_contract"],
        &[
            "operation_shape",
            "operation_example",
            "proof_diagnostic_shape",
            "proof_diagnostic_example",
            "optional_field_encoding",
            "forbidden_dynamic_sources",
        ],
    );

    assert_unique_ids(&vectors["artifact_contexts"]);
    for context in vectors["artifact_contexts"].as_array().unwrap() {
        let fields = if context.get("vir_case").is_some() {
            vec![
                "id",
                "vir_case",
                "source_ir_schema",
                "source_ir_hash",
                "source_language",
                "semantic_profile",
                "semantic_parameters",
            ]
        } else {
            vec![
                "id",
                "vc_fixture",
                "vc_source_context",
                "source_ir_schema",
                "source_ir_hash",
                "source_language",
                "semantic_profile",
                "semantic_parameters",
                "source_manifest_schema",
                "frontend_source_manifest_hash",
                "input_set_hash",
                "source_vc_schema",
                "vc_hash",
            ]
        };
        assert_exact_fields(context, &fields);
    }

    assert_unique_ids(&vectors["import_fixtures"]);
    for fixture in vectors["import_fixtures"].as_array().unwrap() {
        assert_exact_fields(
            fixture,
            &[
                "id",
                "artifact_context",
                "request",
                "canonical_request_utf8_length",
                "canonical_request_sha256",
                "expected_response",
            ],
        );
    }
    assert_unique_ids(&vectors["generate_fixtures"]);
    for fixture in vectors["generate_fixtures"].as_array().unwrap() {
        assert_exact_fields(
            fixture,
            &[
                "id",
                "artifact_context",
                "source_manifest_case",
                "request",
                "canonical_request_utf8_length",
                "canonical_request_sha256",
                "expected_response",
                "canonical_response_utf8_length",
                "canonical_response_sha256",
            ],
        );
    }

    let registry = &vectors["route_registry"];
    for route in registry.as_array().unwrap() {
        assert_exact_fields(route, &["method", "path", "handler"]);
    }
    assert_unique_pairs(registry, "method", "path");

    assert_unique_ids(&vectors["route_cases"]);
    for case in vectors["route_cases"].as_array().unwrap() {
        assert_exact_fields(case, &["id", "method", "path", "expect"]);
        let expected = &case["expect"];
        if expected["outcome"] == "resolve" {
            assert_exact_fields(expected, &["outcome", "handler"]);
        } else {
            assert_exact_fields(expected, &["outcome", "phase", "code"]);
        }
    }

    assert_unique_ids(&vectors["import_cases"]);
    for case in vectors["import_cases"].as_array().unwrap() {
        let selector = ["input_from", "construction", "transport_from", "json_text"]
            .into_iter()
            .filter(|field| case.get(*field).is_some())
            .collect::<Vec<_>>();
        assert_eq!(selector.len(), 1, "import case has one input selector");
        assert_exact_fields(case, &["id", selector[0], "expect"]);
        validate_construction_fields(case);
        validate_case_expect(&case["expect"]);
    }

    assert_unique_ids(&vectors["context_cases"]);
    for case in vectors["context_cases"].as_array().unwrap() {
        let selector = ["request", "request_from", "construction"]
            .into_iter()
            .filter(|field| case.get(*field).is_some())
            .collect::<Vec<_>>();
        assert_eq!(selector.len(), 1, "context case has one input selector");
        assert_exact_fields(
            case,
            &["id", "operation", "session_state", selector[0], "expect"],
        );
        validate_construction_fields(case);
        assert_allowed_fields(
            &case["session_state"],
            &[
                "session_id",
                "state",
                "artifact_context",
                "foreign_session_id",
                "foreign_artifact_context",
                "target_id",
                "candidate_id",
                "proof_recipe",
                "proof_root",
            ],
        );
        let state = case["session_state"].as_object().unwrap();
        assert!(state.contains_key("session_id"));
        assert!(state.contains_key("state"));
        assert!(state.contains_key("artifact_context"));
        assert_eq!(
            state.contains_key("proof_recipe"),
            state.contains_key("proof_root")
        );
        validate_case_expect(&case["expect"]);
    }
}

fn validate_construction_fields(case: &Value) {
    if let Some(construction) = case.get("construction") {
        assert_exact_fields(construction, &["base", "operations"]);
        for operation in construction["operations"].as_array().unwrap() {
            let op = text(&operation["op"]);
            if op == "remove" {
                assert_exact_fields(operation, &["op", "path"]);
            } else {
                assert_exact_fields(operation, &["op", "path", "value"]);
            }
            assert!(matches!(op, "add" | "remove" | "replace"));
        }
    }
    if let Some(transport) = case.get("transport_from") {
        assert_exact_fields(transport, &["fixture", "encoding"]);
    }
}

fn validate_case_expect(expect: &Value) {
    assert_allowed_fields(
        expect,
        &[
            "outcome",
            "mutation_count",
            "phase",
            "code",
            "response_from",
            "member_count",
            "target_id",
            "proof_acceptance",
            "diagnostic_code",
        ],
    );
    let object = expect.as_object().unwrap();
    assert!(object.contains_key("outcome"));
    assert!(object.contains_key("mutation_count"));
    if expect["outcome"] == "reject" {
        assert!(object.contains_key("phase"));
        assert!(object.contains_key("code"));
    }
}

fn assert_unique_ids(values: &Value) {
    let mut ids = BTreeSet::new();
    for value in values.as_array().unwrap() {
        assert!(ids.insert(text(&value["id"])), "duplicate vector case ID");
    }
}

fn assert_unique_pairs(values: &Value, first: &str, second: &str) {
    let mut pairs = BTreeSet::new();
    for value in values.as_array().unwrap() {
        assert!(
            pairs.insert((text(&value[first]), text(&value[second]))),
            "duplicate vector route key"
        );
    }
}

fn assert_exact_fields(value: &Value, expected: &[&str]) {
    let actual = value.as_object().expect("vector object");
    assert_eq!(actual.len(), expected.len(), "vector object is closed");
    for field in expected {
        assert!(actual.contains_key(*field), "missing vector field {field}");
    }
}

fn assert_allowed_fields(value: &Value, allowed: &[&str]) {
    for field in value.as_object().expect("vector object").keys() {
        assert!(
            allowed.contains(&field.as_str()),
            "unknown vector field {field}"
        );
    }
}

#[test]
fn vir_api_executes_every_import_vector_atomically() {
    let vectors = vectors();
    verify_transport_digests(&vectors);
    let fixtures = index_by_id(&vectors["import_fixtures"]);
    let mut executed = BTreeSet::new();

    for case in vectors["import_cases"].as_array().expect("import cases") {
        let id = text(&case["id"]);
        assert!(executed.insert(id));
        let mut service = V1ApiService::new(ValidatedFrontendArtifactStore::empty());
        let input = import_case_transport(case, &fixtures);
        let request_session = request_session_id(&input).unwrap_or_else(|| "s1".to_owned());
        start_through(&mut service, session_number(&request_session).max(1));
        let before = service_fingerprint(&service, &request_session);
        let before_mutations = service.mutation_count();
        let result = service.handle_vir_import(&input);
        let expected = &case["expect"];
        match text(&expected["outcome"]) {
            "accept" => {
                let response = result.expect("import accepts");
                assert_eq!(service.mutation_count() - before_mutations, 1);
                let fixture_id = text(&case["input_from"]);
                assert_eq!(
                    response,
                    canonical_transport(&fixtures[fixture_id]["expected_response"])
                );
                assert!(matches!(
                    service.source_state(&SessionId(request_session)),
                    Some(SessionSourceState::VirImported(_))
                ));
            }
            "reject" => {
                let error = result.expect_err("import rejects");
                assert_rejection(error, expected);
                assert_eq!(service.mutation_count(), before_mutations);
                assert_eq!(service_fingerprint(&service, &request_session), before);
            }
            other => panic!("unknown import outcome {other}"),
        }
    }
    assert_eq!(
        executed.len(),
        vectors["import_cases"].as_array().unwrap().len()
    );
}

#[test]
fn vc_api_executes_every_context_vector_without_acceptance_state() {
    let vectors = vectors();
    let mut executed = BTreeSet::new();
    for case in vectors["context_cases"].as_array().expect("context cases") {
        let id = text(&case["id"]);
        assert!(executed.insert(id));
        match id {
            "context.generate_matching_vc" => assert_generate_matching(&vectors, case),
            "context.reject_cross_session_source" => assert_cross_session(&vectors, case),
            "context.list_matching_vc" => assert_list(&vectors, case),
            "context.start_member_proof" => assert_start(&vectors, case),
            "context.attach_candidate" => assert_attach(&vectors, case),
            "context.reject_stale_source_hash"
            | "context.reject_source_manifest_schema"
            | "context.reject_unknown_source_context"
            | "context.reject_stale_input_set_hash" => assert_generate_rejection(&vectors, case),
            "context.reject_second_import" => assert_second_import(&vectors, case),
            "context.reject_stale_proof_root" => assert_stale_proof_root(&vectors, case),
            "context.rejected_candidate_does_not_commit" => {
                assert_candidate_check(&vectors, case, false)
            }
            "context.accepted_candidate_remains_helper_only" => {
                assert_candidate_check(&vectors, case, true)
            }
            other => panic!("unexecuted context vector {other}"),
        }
    }
    assert_eq!(
        executed.len(),
        vectors["context_cases"].as_array().unwrap().len()
    );
    assert_inherited_errors_remain_compatible(&vectors);
}

fn assert_inherited_errors_remain_compatible(vectors: &Value) {
    let mut service = V1ApiService::new(ValidatedFrontendArtifactStore::empty());
    let import = fixture(vectors, "import_fixtures", "import.go_identity");
    let error = service
        .handle_vir_import(&canonical_transport(&import["request"]))
        .expect_err("unknown session rejects");
    assert_eq!(error.code, V1ErrorCode::UnknownSession);
    assert_eq!(error.message, "API session s1 does not exist");
    assert_eq!(error.field, Some("session_id"));
    assert_eq!(service.mutation_count(), 0);

    let mut service = service_with_generated_go(vectors);
    start_member_target(&mut service, vectors);
    let request = vc_context_request(
        vectors,
        json!({
            "target_id":"t1",
            "candidate_id":"candidate-missing",
            "proof_root":99
        }),
    );
    let before = service_fingerprint(&service, "s10");
    let error = service
        .handle_vc_attach_candidate(&canonical_transport(&request))
        .expect_err("unknown proof rejects");
    assert_eq!(error.code, V1ErrorCode::UnknownProof);
    assert_eq!(
        error.message,
        "proof id 99 is not registered in this API session"
    );
    assert_eq!(error.field, Some("proof_root"));
    assert_eq!(service_fingerprint(&service, "s10"), before);
}

fn assert_generate_matching(vectors: &Value, case: &Value) {
    let (mut service, request) = service_with_imported_go(vectors, "s10");
    let before = service.mutation_count();
    let response = service.handle_vc_generate(&request).expect("VC generates");
    assert_eq!(service.mutation_count() - before, 1);
    let expected = &vectors["generate_fixtures"][0]["expected_response"];
    assert_eq!(response, canonical_transport(expected));
    assert_eq!(
        text(&case["expect"]["response_from"]),
        "generate.go_identity"
    );
    assert_eq!(
        service.retained_context(&SessionId("s10".to_owned())),
        Some((
            "mpk.source_manifest.v0",
            "14a180a4d319bf66d288b8e7e65321f361d5fc48739513a42032c593865f125f"
        ))
    );
    assert!(!response
        .windows(b"accepted".len())
        .any(|bytes| bytes == b"accepted"));
    assert!(!response
        .windows(b"mpk_verified".len())
        .any(|bytes| bytes == b"mpk_verified"));
}

fn assert_cross_session(vectors: &Value, case: &Value) {
    let mut service = V1ApiService::new(validated_store());
    start_through(&mut service, 11);
    let rust = fixture(vectors, "import_fixtures", "import.rust_identity");
    let mut request = rust["request"].clone();
    request["session_id"] = json!("s11");
    service
        .handle_vir_import(&canonical_transport(&request))
        .expect("Rust VIR imports");
    let request = construct_from_case(vectors, case);
    assert_atomic_rejection(
        &mut service,
        "s11",
        |service| service.handle_vc_generate(&canonical_transport(&request)),
        &case["expect"],
    );
}

fn assert_list(vectors: &Value, case: &Value) {
    let service = service_with_generated_go(vectors);
    let before = service_fingerprint(&service, "s10");
    let response = service
        .handle_vc_list(&canonical_transport(&case["request"]))
        .expect("list succeeds");
    let response = response_value(&response);
    assert_eq!(response["members"].as_array().unwrap().len(), 1);
    assert_eq!(response["helper_only"], true);
    assert_eq!(service_fingerprint(&service, "s10"), before);
}

fn assert_start(vectors: &Value, case: &Value) {
    let mut service = service_with_generated_go(vectors);
    let before = service.mutation_count();
    let response = service
        .handle_vc_start_proof(&canonical_transport(&case["request"]))
        .expect("target starts");
    let response = response_value(&response);
    assert_eq!(response["target_id"], case["expect"]["target_id"]);
    assert_eq!(response["helper_only"], true);
    assert_eq!(service.mutation_count() - before, 1);
    assert!(service
        .target_term_id(&SessionId("s10".to_owned()), "t1")
        .is_some());
    assert_eq!(
        service.target_binding(&SessionId("s10".to_owned()), "t1"),
        Some(&crate::vc_api::VcProofTarget::Member {
            id: "example.com/mpk/vector.Identity#postcondition#000000".to_owned()
        })
    );
}

fn assert_attach(vectors: &Value, case: &Value) {
    let mut service = service_with_generated_go(vectors);
    start_member_target(&mut service, vectors);
    build_good_recipe(&mut service, "s10", "t1");
    let before = service.mutation_count();
    let response = service
        .handle_vc_attach_candidate(&canonical_transport(&case["request"]))
        .expect("candidate attaches");
    assert_eq!(response_value(&response)["helper_only"], true);
    assert_eq!(service.mutation_count() - before, 1);
}

fn assert_generate_rejection(vectors: &Value, case: &Value) {
    let (mut service, _) = service_with_imported_go(vectors, "s10");
    let request = construct_from_case(vectors, case);
    assert_atomic_rejection(
        &mut service,
        "s10",
        |service| service.handle_vc_generate(&canonical_transport(&request)),
        &case["expect"],
    );
}

fn assert_second_import(vectors: &Value, case: &Value) {
    let (mut service, _) = service_with_imported_go(vectors, "s1");
    let import =
        canonical_transport(&fixture(vectors, "import_fixtures", "import.go_identity")["request"]);
    assert_atomic_rejection(
        &mut service,
        "s1",
        |service| service.handle_vir_import(&import),
        &case["expect"],
    );
}

fn assert_stale_proof_root(vectors: &Value, case: &Value) {
    let mut service = service_with_generated_go(vectors);
    start_member_target(&mut service, vectors);
    build_good_recipe(&mut service, "s10", "t1");
    let attach = fixture(vectors, "context_cases", "context.attach_candidate");
    service
        .handle_vc_attach_candidate(&canonical_transport(&attach["request"]))
        .expect("candidate attaches");
    assert_atomic_rejection(
        &mut service,
        "s10",
        |service| service.handle_vc_check_candidate(&canonical_transport(&case["request"])),
        &case["expect"],
    );
}

fn assert_candidate_check(vectors: &Value, case: &Value, valid: bool) {
    let mut service = service_with_generated_go(vectors);
    start_member_target(&mut service, vectors);
    if valid {
        build_good_recipe(&mut service, "s10", "t1");
    } else {
        build_wrong_head_recipe(&mut service, "s10", "t1");
    }
    let candidate_id = text(&case["session_state"]["candidate_id"]);
    let root = case["session_state"]["proof_root"].as_u64().unwrap();
    let attach = vc_context_request(
        vectors,
        json!({
            "target_id":"t1",
            "candidate_id":candidate_id,
            "proof_root":root
        }),
    );
    service
        .handle_vc_attach_candidate(&canonical_transport(&attach))
        .expect("candidate attaches");
    let before = service_fingerprint(&service, "s10");
    let response = service
        .handle_vc_check_candidate(&canonical_transport(&case["request"]))
        .expect("bound candidate returns helper verdict");
    let response = response_value(&response);
    let result = &response["results"][0];
    assert_eq!(
        result["helper_status"],
        if valid { "valid" } else { "invalid" }
    );
    assert_eq!(response["helper_only"], true);
    assert!(response.get("accepted").is_none());
    if valid {
        assert!(result.get("diagnostic").is_none());
    } else {
        assert_eq!(
            result["diagnostic"]["error_code"],
            case["expect"]["diagnostic_code"]
        );
    }
    assert_eq!(service_fingerprint(&service, "s10"), before);
}

fn service_with_generated_go(vectors: &Value) -> V1ApiService {
    let (mut service, generate) = service_with_imported_go(vectors, "s10");
    service
        .handle_vc_generate(&generate)
        .expect("VC fixture generates");
    service
}

fn service_with_imported_go(vectors: &Value, session_id: &str) -> (V1ApiService, Vec<u8>) {
    let mut service = V1ApiService::new(validated_store());
    start_through(&mut service, session_number(session_id));
    let import = fixture(vectors, "import_fixtures", "import.go_identity");
    let mut import_request = import["request"].clone();
    import_request["session_id"] = json!(session_id);
    let import = canonical_transport(&import_request);
    service.handle_vir_import(&import).expect("Go VIR imports");
    let generate = fixture(vectors, "generate_fixtures", "generate.go_identity");
    let mut generate_request = generate["request"].clone();
    generate_request["session_id"] = json!(session_id);
    (service, canonical_transport(&generate_request))
}

fn start_member_target(service: &mut V1ApiService, vectors: &Value) {
    let case = fixture(vectors, "context_cases", "context.start_member_proof");
    service
        .handle_vc_start_proof(&canonical_transport(&case["request"]))
        .expect("member target starts");
}

fn build_good_recipe(service: &mut V1ApiService, session_id: &str, target_id: &str) {
    let session_id = SessionId(session_id.to_owned());
    let target = service
        .target_term_id(&session_id, target_id)
        .expect("target term exists");
    let (outer_domain, inner, inner_domain, equality, reflected) = {
        let session = service.legacy.session(&session_id).unwrap();
        let outer = session.core_term_id(target).unwrap();
        let TermNode::Pi {
            ty: outer_domain,
            body: inner,
        } = session.terms().node(outer).clone()
        else {
            panic!("member target has outer binder")
        };
        let TermNode::Pi {
            ty: inner_domain,
            body: equality,
        } = session.terms().node(inner).clone()
        else {
            panic!("member target has implication binder")
        };
        let TermNode::App { arguments, .. } = session.terms().node(equality).clone() else {
            panic!("member target has equality")
        };
        (outer_domain, inner, inner_domain, equality, arguments[0])
    };
    let (outer_domain, inner, inner_domain, equality, reflected) = {
        let session = service.legacy.session_mut(&session_id).unwrap();
        (
            session.register_term_id(outer_domain).unwrap(),
            session.register_term_id(inner).unwrap(),
            session.register_term_id(inner_domain).unwrap(),
            session.register_term_id(equality).unwrap(),
            session.register_term_id(reflected).unwrap(),
        )
    };
    let refl = service
        .legacy
        .proof_refl(ReflProofRequest {
            session_id: session_id.clone(),
            term: reflected,
            expected_type: equality,
        })
        .unwrap();
    assert_eq!(refl.proof_id.0, 0);
    let inner_intro = service
        .legacy
        .proof_intro(IntroProofRequest {
            session_id: session_id.clone(),
            domain_type: inner_domain,
            body_proof: refl.proof_id,
            expected_type: inner,
        })
        .unwrap();
    assert_eq!(inner_intro.proof_id.0, 1);
    let outer_intro = service
        .legacy
        .proof_intro(IntroProofRequest {
            session_id,
            domain_type: outer_domain,
            body_proof: inner_intro.proof_id,
            expected_type: target,
        })
        .unwrap();
    assert_eq!(outer_intro.proof_id.0, 2);
}

fn build_wrong_head_recipe(service: &mut V1ApiService, session_id: &str, target_id: &str) {
    let session_id = SessionId(session_id.to_owned());
    let target = service.target_term_id(&session_id, target_id).unwrap();
    let (truth, equality_function) = {
        let session = service.legacy.session(&session_id).unwrap();
        let outer = session.core_term_id(target).unwrap();
        let inner = match session.terms().node(outer) {
            TermNode::Pi { body, .. } => *body,
            _ => panic!("outer pi"),
        };
        let (truth, equality) = match session.terms().node(inner) {
            TermNode::Pi { ty, body } => (*ty, *body),
            _ => panic!("inner pi"),
        };
        let equality_function = match session.terms().node(equality) {
            TermNode::App { function, .. } => *function,
            _ => panic!("equality app"),
        };
        (truth, equality_function)
    };
    let (truth, wrong_equality) = {
        let session = service.legacy.session_mut(&session_id).unwrap();
        let wrong = session
            .terms_mut()
            .app(equality_function, vec![truth, truth]);
        (
            session.register_term_id(truth).unwrap(),
            session.register_term_id(wrong).unwrap(),
        )
    };
    let proof = service
        .legacy
        .proof_refl(ReflProofRequest {
            session_id,
            term: truth,
            expected_type: wrong_equality,
        })
        .unwrap();
    assert_eq!(proof.proof_id.0, 0);
}

fn vc_context_request(vectors: &Value, additions: Value) -> Value {
    let list = fixture(vectors, "context_cases", "context.list_matching_vc");
    let mut request = list["request"].clone();
    for (key, value) in additions.as_object().unwrap() {
        request[key] = value.clone();
    }
    request
}

fn validated_store() -> ValidatedFrontendArtifactStore {
    let ai = vectors();
    let source_maps: Value = serde_json::from_str(SOURCE_MAP_VECTOR).unwrap();
    let manifests: Value = serde_json::from_str(SOURCE_MANIFEST_VECTOR).unwrap();
    let releases: Value = serde_json::from_str(RELEASE_VECTOR).unwrap();
    let vir_bytes = canonical_value(&ai["import_fixtures"][0]["request"]["vir"]);
    let vir = mpk_vc::import_vir_json(&vir_bytes).expect("VIR validates");

    const CONTRACT: &[u8] = br#"{"schema":"mpk.go.contract.v0","function":"example.com/mpk/vector.Identity","requires":[],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}],"modifies":[],"loops":[]}
"#;
    const GO_MOD: &[u8] = b"module example.com/mpk/vector\n\ngo 1.25\n";
    const GO_SUM: &[u8] = b"";
    const SOURCE: &[u8] = b"package vector\n\nfunc Identity(value int8) int8 { return value }\n";
    let inputs = [
        CapturedInput {
            kind: InputKind::Contract,
            normalized_path: "contracts/identity.json",
            bytes: CONTRACT,
        },
        CapturedInput {
            kind: InputKind::BuildManifest,
            normalized_path: "go.mod",
            bytes: GO_MOD,
        },
        CapturedInput {
            kind: InputKind::Lockfile,
            normalized_path: "go.sum",
            bytes: GO_SUM,
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: "identity.go",
            bytes: SOURCE,
        },
    ];
    let map_case = source_maps["map_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "map.valid_go_identity")
        .unwrap();
    let source_map = import_source_map_json(
        &canonical_value(&map_case["input"]),
        SourceMapValidationContext {
            vir: &vir,
            captured_inputs: &inputs,
            synthetic_permissions: &[],
        },
    )
    .expect("source map validates");
    let mut registry_bytes = canonical_value(&releases["fixtures"]["valid_registry"]);
    registry_bytes.push(b'\n');
    let registry = validate_release_registry(&registry_bytes).expect("release validates");
    let manifest_case = manifests["manifest_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "manifest.valid_go_frontend_stage")
        .unwrap();
    let source_manifest = import_frontend_source_manifest_json(
        &canonical_value(&manifest_case["input"]),
        SourceManifestValidationContext {
            vir: &vir,
            source_map: &source_map,
            captured_inputs: &inputs,
            release_registry: &registry,
            expected_language_configuration: None,
        },
    )
    .expect("source manifest validates");
    ValidatedFrontendArtifactStore::from_frontend_successes([ValidatedFrontendArtifacts {
        vir,
        source_map,
        source_manifest,
    }])
    .expect("validated store builds")
}

fn verify_transport_digests(vectors: &Value) {
    for fixture in vectors["import_fixtures"].as_array().unwrap() {
        let request = canonical_transport(&fixture["request"]);
        assert_eq!(
            request.len() as u64,
            fixture["canonical_request_utf8_length"]
        );
        assert_eq!(
            hex_sha256(&request),
            text(&fixture["canonical_request_sha256"])
        );
    }
    for fixture in vectors["generate_fixtures"].as_array().unwrap() {
        let request = canonical_transport(&fixture["request"]);
        let response = canonical_transport(&fixture["expected_response"]);
        assert_eq!(
            request.len() as u64,
            fixture["canonical_request_utf8_length"]
        );
        assert_eq!(
            response.len() as u64,
            fixture["canonical_response_utf8_length"]
        );
        assert_eq!(
            hex_sha256(&request),
            text(&fixture["canonical_request_sha256"])
        );
        assert_eq!(
            hex_sha256(&response),
            text(&fixture["canonical_response_sha256"])
        );
    }
}

fn assert_atomic_rejection(
    service: &mut V1ApiService,
    session_id: &str,
    operation: impl FnOnce(&mut V1ApiService) -> Result<Vec<u8>, V1ApiError>,
    expected: &Value,
) {
    let before = service_fingerprint(service, session_id);
    let mutations = service.mutation_count();
    let error = operation(service).expect_err("operation rejects");
    assert_rejection(error, expected);
    assert_eq!(service.mutation_count(), mutations);
    assert_eq!(service_fingerprint(service, session_id), before);
}

fn assert_rejection(error: V1ApiError, expected: &Value) {
    assert_eq!(error.code.as_str(), text(&expected["code"]));
    assert_eq!(error.phase().as_str(), text(&expected["phase"]));
    let serialized = serde_json::to_value(&error).unwrap();
    assert_eq!(serialized["message"], "AI API v1 request rejected");
    assert!(serialized.get("phase").is_none());
    assert!(serialized
        .as_object()
        .unwrap()
        .values()
        .all(|value| !value.is_null()));
}

fn service_fingerprint(service: &V1ApiService, session_id: &str) -> String {
    let id = SessionId(session_id.to_owned());
    let state = service
        .source_state(&id)
        .map(|state| format!("{}:{state:?}", state.label()))
        .unwrap_or_else(|| "missing".to_owned());
    let session = service.legacy.session(&id);
    format!(
        "{}|{}|{}|{}|{}",
        service.mutation_count(),
        state,
        session.map_or(0, |session| session.terms().len()),
        session.map_or(0, |session| session.proof_node_count()),
        session.map_or(0, |session| session.theory_certificate_count())
    )
}

fn import_case_transport(
    case: &Value,
    fixtures: &std::collections::BTreeMap<&str, &Value>,
) -> Vec<u8> {
    if let Some(id) = case.get("input_from").and_then(Value::as_str) {
        return canonical_transport(&fixtures[id]["request"]);
    }
    if let Some(construction) = case.get("construction") {
        let mut value = fixtures[text(&construction["base"])]["request"].clone();
        apply_operations(&mut value, &construction["operations"]);
        return canonical_transport(&value);
    }
    if let Some(transport) = case.get("transport_from") {
        let request = &fixtures[text(&transport["fixture"])]["request"];
        let mut bytes = serde_json::to_vec_pretty(request).unwrap();
        bytes.push(b'\n');
        return bytes;
    }
    case["json_text"].as_str().unwrap().as_bytes().to_vec()
}

fn construct_from_case(vectors: &Value, case: &Value) -> Value {
    let construction = &case["construction"];
    let base = text(&construction["base"]);
    let mut value = if base.starts_with("generate.") {
        fixture(vectors, "generate_fixtures", base)["request"].clone()
    } else {
        fixture(vectors, "import_fixtures", base)["request"].clone()
    };
    apply_operations(&mut value, &construction["operations"]);
    value
}

fn apply_operations(value: &mut Value, operations: &Value) {
    for operation in operations.as_array().unwrap() {
        let path = text(&operation["path"]);
        let (parent, key) = pointer_parent_mut(value, path);
        let object = parent.as_object_mut().expect("vector patch parent object");
        match text(&operation["op"]) {
            "add" | "replace" => {
                object.insert(key.to_owned(), operation["value"].clone());
            }
            "remove" => {
                object.remove(key).expect("field exists");
            }
            other => panic!("unsupported vector patch {other}"),
        }
    }
}

fn pointer_parent_mut<'a, 'b>(value: &'a mut Value, path: &'b str) -> (&'a mut Value, &'b str) {
    let split = path.rfind('/').expect("JSON pointer path");
    let (parent, key) = path.split_at(split);
    let key = &key[1..];
    let parent = if parent.is_empty() {
        value
    } else {
        value.pointer_mut(parent).expect("patch parent exists")
    };
    (parent, key)
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = canonical_value(value);
    bytes.push(b'\n');
    bytes
}

fn canonical_value(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576),
    )
    .unwrap();
    canonical_json_bytes(&strict).unwrap()
}

fn response_value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes.strip_suffix(b"\n").unwrap()).unwrap()
}

fn request_session_id(bytes: &[u8]) -> Option<String> {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|value| value["session_id"].as_str().map(str::to_owned))
}

fn start_through(service: &mut V1ApiService, count: u64) {
    while service.legacy.session_count() < usize::try_from(count).unwrap() {
        service
            .start_session(StartSessionRequest::new(format!(
                "Example.AiV1.Session{}",
                service.legacy.session_count() + 1
            )))
            .expect("session starts");
    }
}

fn session_number(session_id: &str) -> u64 {
    session_id.strip_prefix('s').unwrap().parse().unwrap()
}

fn fixture<'a>(vectors: &'a Value, field: &str, id: &str) -> &'a Value {
    vectors[field]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["id"] == id)
        .unwrap_or_else(|| panic!("missing {field} fixture {id}"))
}

fn index_by_id(value: &Value) -> std::collections::BTreeMap<&str, &Value> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| (text(&value["id"]), value))
        .collect()
}

fn vectors() -> Value {
    serde_json::from_str(AI_API_VECTOR).expect("AI API vector parses")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("vector string")
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
