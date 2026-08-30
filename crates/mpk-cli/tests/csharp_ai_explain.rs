#[path = "support/successor_policy.rs"]
mod successor_policy_support;

use std::fs;

use mpk_cli::ai_explain::{ExplainLanguageV1, DEFAULT_GEMINI_MODEL};
use mpk_cli::successor_ai_explain::{
    build_successor_ai_explanation, import_successor_ai_explanation_json,
    import_successor_ai_request_json, prepare_successor_ai_explanation, SuccessorAiCode,
    SuccessorAiProviderProvenance, SuccessorAiReportRequest, SuccessorAiSource,
    SUCCESSOR_AI_EXPLAIN_REQUEST_SCHEMA, SUCCESSOR_AI_EXPLANATION_SCHEMA,
};
use mpk_vc::{canonical_json_bytes, parse_strict_json, StrictJsonLimits};
use serde_json::{json, Value};

use successor_policy_support::{
    ai_contract, checked_in_json, complete_successor_policy_runs, repository_path,
    validated_registry,
};

const UPDATE_FIXTURES_ENV: &str = "MPK_UPDATE_CSHARP_AI_FIXTURES";
const FIXTURE_ROOT: &str = "fixtures/csharp/ai";
const JSON_LIMITS: StrictJsonLimits = StrictJsonLimits::new(2_097_152, 2_097_152, 128, 2_097_152);

fn canonical(value: &Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).expect("JSON serializes");
    let strict = parse_strict_json(&encoded, JSON_LIMITS).expect("strict JSON");
    canonical_json_bytes(&strict).expect("canonical JSON")
}

fn fixture(relative: &str, generated: &[u8]) {
    let path = repository_path(FIXTURE_ROOT).join(relative);
    if std::env::var_os(UPDATE_FIXTURES_ENV).as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
        fs::write(&path, generated).expect("write fixture");
    }
    assert_eq!(
        fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
        generated,
        "fixture drift: {}",
        path.display()
    );
}

fn property_refs(request: &Value) -> Vec<String> {
    request["properties"]
        .as_array()
        .expect("sanitized properties")
        .iter()
        .map(|property| {
            property["ref"]
                .as_str()
                .expect("sanitized property alias")
                .to_owned()
        })
        .collect()
}

fn provider_response(refs: &[String]) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "overview":"The supplied MPK evidence has been summarized without changing its status.",
        "property_explanations":refs.iter().map(|property_ref| json!({
            "property_ref":property_ref,
            "explanation":"This explanation is helper prose for the status supplied by MPK."
        })).collect::<Vec<_>>(),
        "limitations":["This helper response is not proof evidence."],
        "next_steps":["Use the referenced MPK evidence and checkers for verification status."]
    }))
    .expect("provider response")
}

fn report_request() -> SuccessorAiReportRequest {
    SuccessorAiReportRequest {
        project: "mpk-test-project".to_owned(),
        location: "asia-northeast1".to_owned(),
        requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
        language: ExplainLanguageV1::En,
    }
}

fn provenance(profile: &str) -> SuccessorAiProviderProvenance {
    SuccessorAiProviderProvenance {
        model_version: "gemini-3.5-flash-001".to_owned(),
        response_id: format!("response-{profile}"),
        create_time: "2026-08-28T00:00:00Z".to_owned(),
        finish_reason: "STOP".to_owned(),
        attempts: 1,
        prompt_tokens: Some(128),
        thinking_tokens: Some(0),
        response_tokens: Some(64),
        total_tokens: Some(192),
    }
}

fn strings_at<'a>(value: &'a Value, key: &str, output: &mut Vec<&'a str>) {
    match value {
        Value::Array(values) => {
            for value in values {
                strings_at(value, key, output);
            }
        }
        Value::Object(object) => {
            for (name, value) in object {
                if name == key {
                    match value {
                        Value::String(value) => output.push(value),
                        Value::Array(values) => {
                            output.extend(values.iter().filter_map(Value::as_str))
                        }
                        _ => {}
                    }
                }
                strings_at(value, key, output);
            }
        }
        _ => {}
    }
}

fn hash_strings<'a>(value: &'a Value, output: &mut Vec<&'a str>) {
    match value {
        Value::Array(values) => {
            for value in values {
                hash_strings(value, output);
            }
        }
        Value::Object(object) => {
            for (name, value) in object {
                if (name == "sha256" || name.ends_with("_hash") || name.ends_with("_sha256"))
                    && value.as_str().is_some()
                {
                    output.push(value.as_str().expect("string checked"));
                }
                hash_strings(value, output);
            }
        }
        _ => {}
    }
}

fn assert_redacted(profile: &str, request_body: &[u8], evidence: &Value) {
    let body = std::str::from_utf8(request_body).expect("UTF-8 provider request");
    for key in [
        "function",
        "methods",
        "sources",
        "contracts",
        "package",
        "crate",
        "normalized_path",
        "function_id",
        "member_id",
        "declaration_name",
        "description",
        "notes",
    ] {
        let mut sensitive = Vec::new();
        strings_at(evidence, key, &mut sensitive);
        for value in sensitive {
            assert!(
                !body.contains(value),
                "{profile} leaked {key} value {value:?} into the provider request"
            );
        }
    }
    for property in evidence["properties"]
        .as_array()
        .expect("evidence properties")
    {
        let property_id = property["id"].as_str().expect("evidence property ID");
        assert!(
            !body.contains(property_id),
            "{profile} leaked property ID {property_id:?} into the provider request"
        );
    }
    let mut hashes = Vec::new();
    hash_strings(evidence, &mut hashes);
    for hash in hashes {
        assert!(
            !body.contains(hash),
            "{profile} leaked evidence hash {hash:?} into the provider request"
        );
    }
    assert!(!body.contains("public static class"));
    assert!(!body.contains("abrupt_completion"));
    assert!(!body.contains("diagnostics"));
    assert!(!body.contains("compiler"));
}

#[test]
fn all_profiles_stage_deterministic_redacted_requests_and_untrusted_reports() {
    let registry = validated_registry();
    let active_vectors = checked_in_json("develop/specs/vectors/ai-explain-v1.json");
    let mut saw_csharp = false;

    for named in complete_successor_policy_runs(&registry) {
        let profile = named.profile;
        let evidence_before = named.run.evidence().canonical_bytes().to_vec();
        let certificate_before = named.run.program_certificate().clone();
        let contract = ai_contract(profile);
        let source = SuccessorAiSource {
            registry: &registry,
            evidence: named.run.evidence(),
            ai_contract: &contract,
        };
        let first = prepare_successor_ai_explanation(source, ExplainLanguageV1::En)
            .unwrap_or_else(|error| panic!("{profile} successor AI request: {error}"));
        let second = prepare_successor_ai_explanation(source, ExplainLanguageV1::En)
            .unwrap_or_else(|error| panic!("{profile} repeated successor AI request: {error}"));
        assert_eq!(first, second, "{profile} request is not deterministic");
        assert_eq!(
            first.document().schema(),
            SUCCESSOR_AI_EXPLAIN_REQUEST_SCHEMA
        );
        first
            .import_request_json(first.canonical_request_bytes())
            .expect("exact request reimport");
        assert_eq!(
            import_successor_ai_request_json(
                first.canonical_request_bytes(),
                source,
                ExplainLanguageV1::En,
            )
            .expect("source-regenerated request import"),
            first
        );

        let request_value: Value = serde_json::from_slice(first.canonical_request_bytes())
            .expect("sanitized request JSON");
        let evidence_value: Value =
            serde_json::from_slice(&evidence_before).expect("evidence JSON");
        let (display, projection, strategy, axiom) = match profile {
            "go" => (
                "Go",
                "mpk.go.ai_projection.v0",
                "payment-policy-alpha",
                "zero-axiom",
            ),
            "rust" => (
                "Rust",
                "mpk.rust.ai_projection.v0",
                "payment-policy-rust-alpha",
                "mvp-theory",
            ),
            "csharp" => (
                "C#",
                "mpk.csharp.ai_projection.v0",
                "payment-policy-csharp-alpha",
                "mvp-theory",
            ),
            other => panic!("unexpected profile {other}"),
        };
        assert_eq!(first.registration().display_language(), display);
        assert_eq!(first.registration().projection_profile_id(), projection);
        assert_eq!(first.registration().redaction_profile_id(), "minimal-v1");
        assert_eq!(request_value["display_language"], display);
        assert_eq!(request_value["projection_profile_id"], projection);
        assert_eq!(request_value["redaction_profile_id"], "minimal-v1");
        assert_eq!(request_value["policy"]["strategy_profile"], strategy);
        assert_eq!(request_value["policy"]["checker_profile"], "mvp-strict");
        assert_eq!(request_value["policy"]["axiom_profile"], axiom);
        assert!(request_value.get("selection").is_none());
        assert!(request_value.get("semantic_context").is_none());
        assert_redacted(profile, first.request_body(), &evidence_value);

        let mut widened = contract.clone();
        widened["value"]["source_access"] = Value::Bool(true);
        assert_eq!(
            prepare_successor_ai_explanation(
                SuccessorAiSource {
                    registry: &registry,
                    evidence: named.run.evidence(),
                    ai_contract: &widened,
                },
                ExplainLanguageV1::En,
            )
            .unwrap_err()
            .code(),
            SuccessorAiCode::ProfileContract,
            "{profile} widened AI contract must reject"
        );

        let request_body: Value =
            serde_json::from_slice(first.request_body()).expect("Vertex request JSON");
        assert_eq!(
            request_body["systemInstruction"]["parts"][0]["text"],
            active_vectors["prompt"]["system_instruction_utf8"]
        );
        assert_eq!(
            request_body["generationConfig"]["responseFormat"][0]["text"]["mimeType"],
            "APPLICATION_JSON"
        );
        let refs = property_refs(&request_value);
        let provider_text = provider_response(&refs);
        let first_report = build_successor_ai_explanation(
            &first,
            &report_request(),
            &provenance(profile),
            &provider_text,
        )
        .unwrap_or_else(|error| panic!("{profile} successor AI report: {error}"));
        let second_report = build_successor_ai_explanation(
            &first,
            &report_request(),
            &provenance(profile),
            &provider_text,
        )
        .unwrap_or_else(|error| panic!("{profile} repeated successor AI report: {error}"));
        assert_eq!(
            first_report.canonical_bytes(),
            second_report.canonical_bytes(),
            "{profile} report is not deterministic"
        );
        assert_eq!(
            first_report.document().schema(),
            SUCCESSOR_AI_EXPLANATION_SCHEMA
        );
        assert!(!first_report.document().proof_evidence());
        assert_eq!(
            first_report.document().trust_classification(),
            "untrusted_helper_analysis"
        );
        assert_eq!(
            first_report.document().semantic_context(),
            named.run.evidence().document().semantic_context()
        );
        assert_eq!(
            first_report.document().policy_contract(),
            named.run.evidence().document().policy_contract()
        );
        let expected_properties = named
            .run
            .evidence()
            .document()
            .properties()
            .iter()
            .map(|property| (property.id.as_str(), property.status.as_str()))
            .collect::<Vec<_>>();
        let restored = first_report
            .document()
            .property_explanations()
            .map(|(id, status, _)| (id, status))
            .collect::<Vec<_>>();
        assert_eq!(restored, expected_properties);
        first_report
            .import_json(first_report.canonical_bytes())
            .expect("exact report reimport");
        assert_eq!(
            import_successor_ai_explanation_json(
                first_report.canonical_bytes(),
                &first,
                &report_request(),
                &provenance(profile),
                &provider_text,
            )
            .expect("source-regenerated report import")
            .canonical_bytes(),
            first_report.canonical_bytes()
        );

        assert_eq!(named.run.evidence().canonical_bytes(), evidence_before);
        assert_eq!(named.run.program_certificate(), &certificate_before);

        if profile == "csharp" {
            saw_csharp = true;
            fixture("request.json", first.canonical_request_bytes());
            fixture("explanation.json", first_report.canonical_bytes());

            let mut old_request = request_value.clone();
            old_request["schema"] = Value::String("mpk.ai.explain.request.v1".to_owned());
            assert_eq!(
                first
                    .import_request_json(&canonical(&old_request))
                    .unwrap_err()
                    .code(),
                SuccessorAiCode::CanonicalTransport
            );
            let mut noncanonical = first.canonical_request_bytes().to_vec();
            noncanonical.push(b'\n');
            assert_eq!(
                first.import_request_json(&noncanonical).unwrap_err().code(),
                SuccessorAiCode::CanonicalTransport
            );

            let crossed = ai_contract("go");
            let error = prepare_successor_ai_explanation(
                SuccessorAiSource {
                    registry: &registry,
                    evidence: named.run.evidence(),
                    ai_contract: &crossed,
                },
                ExplainLanguageV1::En,
            )
            .expect_err("crossed AI contract must reject");
            assert_eq!(error.code(), SuccessorAiCode::ProfileContract);
            let injected = json!({
                "overview":"Injected metadata must reject.",
                "property_explanations":[{
                    "property_ref":refs[0],
                    "explanation":"Text.",
                    "source_status":"mpk_verified"
                }],
                "limitations":[],
                "next_steps":[],
                "proof_evidence":true
            });
            assert_eq!(
                build_successor_ai_explanation(
                    &first,
                    &report_request(),
                    &provenance(profile),
                    &serde_json::to_vec(&injected).unwrap(),
                )
                .unwrap_err()
                .code(),
                SuccessorAiCode::ResponseInvalid
            );
            let unknown_alias = json!({
                "overview":"Unknown aliases reject.",
                "property_explanations":[{
                    "property_ref":"property-9999",
                    "explanation":"Text."
                }],
                "limitations":[],
                "next_steps":[]
            });
            assert_eq!(
                build_successor_ai_explanation(
                    &first,
                    &report_request(),
                    &provenance(profile),
                    &serde_json::to_vec(&unknown_alias).unwrap(),
                )
                .unwrap_err()
                .code(),
                SuccessorAiCode::ResponseInvalid
            );
            let bidi = json!({
                "overview":"Bidi \u{202e} text rejects.",
                "property_explanations":[{
                    "property_ref":refs[0],
                    "explanation":"Text."
                }],
                "limitations":[],
                "next_steps":[]
            });
            assert_eq!(
                build_successor_ai_explanation(
                    &first,
                    &report_request(),
                    &provenance(profile),
                    &serde_json::to_vec(&bidi).unwrap(),
                )
                .unwrap_err()
                .code(),
                SuccessorAiCode::ResponseInvalid
            );

            let mut changed_report: Value =
                serde_json::from_slice(first_report.canonical_bytes()).unwrap();
            changed_report["trust"]["proof_evidence"] = Value::Bool(true);
            assert_eq!(
                first_report
                    .import_json(&canonical(&changed_report))
                    .unwrap_err()
                    .code(),
                SuccessorAiCode::CanonicalTransport
            );
            changed_report["trust"]["proof_evidence"] = Value::Bool(false);
            changed_report["schema"] = Value::String("mpk.ai.explanation.v1".to_owned());
            assert_eq!(
                first_report
                    .import_json(&canonical(&changed_report))
                    .unwrap_err()
                    .code(),
                SuccessorAiCode::CanonicalTransport
            );
            assert_eq!(named.run.evidence().canonical_bytes(), evidence_before);
            assert_eq!(named.run.program_certificate(), &certificate_before);
        }
    }
    assert!(saw_csharp);
}

#[test]
fn csharp_ai_owner_is_registered_for_every_consumed_frozen_vector() {
    let manifest = checked_in_json("develop/specs/vectors/manifest.json");
    for path in [
        "develop/specs/vectors/ai-explain-v1.json",
        "develop/specs/vectors/csharp-profile-v0.json",
        "develop/specs/vectors/semantic-profile-registry-v1.json",
        "develop/specs/vectors/semantic-profile-registry-v2.json",
    ] {
        let record = manifest["vectors"]
            .as_array()
            .expect("vector manifest")
            .iter()
            .find(|record| record["path"] == path)
            .unwrap_or_else(|| panic!("missing vector record {path}"));
        assert!(
            record["implementation_test_owners"]
                .as_array()
                .expect("implementation owners")
                .iter()
                .any(|owner| owner == "crates/mpk-cli/tests/csharp_ai_explain.rs"),
            "missing T17 owner for {path}"
        );
    }
}
