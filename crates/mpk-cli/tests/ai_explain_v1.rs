#[path = "../src/ai_explain_v1.rs"]
mod ai_explain_v1;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ai_explain_v1::*;
use mpk_cli::policy_schema::{
    import_policy_evidence_v1_json, import_policy_scan_v1_json, PolicyCheckedDeclaration,
    PolicyEvidenceLinkageContext, PolicyExpectedCertificateV1, PolicyExpectedMemberV1,
    PolicyExpectedPropertyV1, PolicyHelperArtifact, PolicyIssue, PolicyPropertyV1,
    PolicyScanLinkageContext, PolicySelection, PolicySemanticParameters, PolicyTrustedEvidenceV1,
    ValidatedPolicyEvidenceV1,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, FrontendIdentity, ReleaseRegistryIdentity,
    StrictJsonLimits, ToolchainIdentity,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const POLICY_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576);

#[derive(Debug, Deserialize)]
struct ScanVector {
    linkage_contexts: Vec<ScanContext>,
    fixtures: Vec<Fixture>,
}

#[derive(Clone, Debug, Deserialize)]
struct ScanContext {
    id: String,
    frontend_status: String,
    frontend_phase: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: PolicySemanticParameters,
    selection: PolicySelection,
    release_registry: ReleaseRegistryIdentity,
    frontend: FrontendIdentity,
    toolchain: ToolchainIdentity,
    #[serde(default)]
    limit_profile: Option<String>,
    #[serde(default)]
    frontend_source_manifest_hash: Option<String>,
    #[serde(default)]
    input_set_hash: Option<String>,
    #[serde(default)]
    source_map_hash: Option<String>,
    #[serde(default)]
    source_ir_schema: Option<String>,
    #[serde(default)]
    source_ir_hash: Option<String>,
    #[serde(default)]
    helper_artifacts: Option<Vec<PolicyHelperArtifact>>,
    rejected_features: Vec<PolicyIssue>,
    diagnostics: Vec<PolicyIssue>,
}

impl ScanContext {
    fn linkage(&self) -> PolicyScanLinkageContext {
        PolicyScanLinkageContext {
            frontend_status: self.frontend_status.clone(),
            frontend_phase: self.frontend_phase.clone(),
            source_language: self.source_language.clone(),
            semantic_profile: self.semantic_profile.clone(),
            semantic_parameters: self.semantic_parameters.clone(),
            selection: self.selection.clone(),
            release_registry: self.release_registry.clone(),
            frontend: self.frontend.clone(),
            toolchain: self.toolchain.clone(),
            rejected_features: self.rejected_features.clone(),
            diagnostics: self.diagnostics.clone(),
            limit_profile: self.limit_profile.clone(),
            frontend_source_manifest_hash: self.frontend_source_manifest_hash.clone(),
            input_set_hash: self.input_set_hash.clone(),
            source_map_hash: self.source_map_hash.clone(),
            source_ir_schema: self.source_ir_schema.clone(),
            source_ir_hash: self.source_ir_hash.clone(),
            helper_artifacts: self.helper_artifacts.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Fixture {
    id: String,
    linkage_context: String,
    input: Value,
}

#[derive(Debug, Deserialize)]
struct EvidenceVector {
    linkage_contexts: Vec<EvidenceContext>,
    fixtures: Vec<Fixture>,
}

#[derive(Clone, Debug, Deserialize)]
struct EvidenceContext {
    id: String,
    scan_fixture: String,
    certificate_source_manifest_hash: String,
    source_vc_schema: String,
    vc_hash: String,
    verification_limit_profile: String,
    #[serde(default)]
    members: Vec<VectorMember>,
    #[serde(default)]
    declarations: Vec<PolicyCheckedDeclaration>,
    #[serde(default)]
    accepted_certificate_id: Option<String>,
    #[serde(default)]
    accepted_certificate_hash: Option<String>,
    #[serde(default)]
    accepted_export_hash: Option<String>,
    #[serde(default)]
    accepted_axiom_report_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct VectorMember {
    member_id: String,
    function_id: String,
    kind: String,
    group_id: String,
    declaration_name: String,
    declaration_hash: String,
    dependencies: Vec<mpk_cli::policy_schema::PolicyDeclarationDependency>,
}

#[test]
fn ai_explain_v1_normative_payload_and_request_vectors_match_exact_bytes() {
    let vectors = load_value("develop/specs/vectors/ai-explain-v1.json");
    assert_eq!(
        vectors["owner_test"],
        "crates/mpk-cli/tests/ai_explain_v1.rs"
    );
    assert_eq!(prompt_template_sha256_v1(), vectors["prompt"]["sha256"]);
    assert_eq!(
        SYSTEM_INSTRUCTION_V1,
        vectors["prompt"]["system_instruction_utf8"]
    );
    assert_eq!(USER_TEMPLATE_V1, vectors["prompt"]["user_template_utf8"]);

    for fixture in vectors["evidence_fixtures"].as_array().unwrap() {
        let validated = validated_evidence(fixture["policy_fixture"].as_str().unwrap());
        let language = language(fixture["language"].as_str().unwrap());
        let first = build_vertex_request_v1(&validated, language).unwrap();
        let second = build_vertex_request_v1(&validated, language).unwrap();
        assert_eq!(first.request_body, second.request_body, "{}", fixture["id"]);
        assert_eq!(
            serde_json::to_value(&first.payload).unwrap(),
            fixture["expected_sanitized_payload"],
            "{}",
            fixture["id"]
        );
        assert_eq!(
            serde_json::to_value(&first.alias_map).unwrap(),
            fixture["alias_map"],
            "{}",
            fixture["id"]
        );
        assert_eq!(
            first.payload_json.len() as u64,
            fixture["sanitized_payload_utf8_length"].as_u64().unwrap()
        );
        assert_eq!(
            first.sanitized_payload_sha256,
            fixture["sanitized_payload_sha256"]
        );
        assert_eq!(
            validated.canonical_bytes().len() as u64,
            fixture["source_evidence_utf8_length"].as_u64().unwrap()
        );
        assert_eq!(first.evidence_sha256, fixture["source_evidence_sha256"]);

        let request_fixture = vectors["request_fixtures"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["evidence_fixture"] == fixture["id"])
            .unwrap();
        assert_eq!(
            first.prompt_template_sha256,
            request_fixture["prompt_template_sha256"]
        );
        assert_eq!(
            first.response_schema_sha256,
            request_fixture["response_schema_sha256"]
        );
        assert_eq!(
            first.request_body.len() as u64,
            request_fixture["request_body_utf8_length"]
                .as_u64()
                .unwrap()
        );
        assert_eq!(
            first.request_body_sha256,
            request_fixture["request_body_sha256"]
        );

        let body = String::from_utf8(first.request_body).unwrap();
        assert!(!body.contains("go_source"));
        assert!(!body.contains("\"gir\""));
        for forbidden in fixture["forbidden_substrings"].as_array().unwrap() {
            let forbidden = forbidden.as_str().unwrap();
            assert!(!body.contains(forbidden), "leaked {forbidden}");
            assert!(
                !first.payload_json.contains(forbidden),
                "leaked {forbidden}"
            );
        }
    }
}

#[test]
fn ai_explain_v1_profiles_future_strategy_and_v0_rejection_are_fail_closed() {
    let go = validated_evidence("evidence.go_identity_pending");
    let rust = validated_evidence("evidence.rust_call_pair_verified");
    assert_eq!(
        build_vertex_request_v1(&go, ExplainLanguageV1::En)
            .unwrap()
            .payload
            .policy
            .strategy_profile,
        "payment-policy-alpha"
    );
    assert_eq!(
        build_vertex_request_v1(&rust, ExplainLanguageV1::En)
            .unwrap()
            .payload
            .policy
            .strategy_profile,
        "payment-policy-rust-alpha"
    );

    let rust_document = rust.document();
    let crossed = ExplainProfileInputV1 {
        source_language: rust_document.source_language.clone(),
        semantic_profile: rust_document.semantic_profile.clone(),
        semantic_parameters: rust_document.semantic_parameters.clone(),
        strategy_profile: "payment-policy-alpha".to_owned(),
        checker_profile: rust_document.checker_profile.clone(),
        axiom_profile: rust_document.axiom_profile.clone(),
        upstream_registry_authorized: false,
    };
    assert_eq!(
        validate_profile_v1(&crossed).unwrap_err().code(),
        AiExplainV1ErrorCode::ProfileTuple
    );
    let future = ExplainProfileInputV1 {
        strategy_profile: "payment-policy-future-alpha".to_owned(),
        upstream_registry_authorized: true,
        ..crossed.clone()
    };
    assert_eq!(validate_profile_v1(&future).unwrap(), "unrecognized");
    assert_eq!(
        validate_profile_v1(&ExplainProfileInputV1 {
            upstream_registry_authorized: false,
            ..future
        })
        .unwrap_err()
        .code(),
        AiExplainV1ErrorCode::InvalidEvidence
    );

    let vectors = load_value("develop/specs/vectors/ai-explain-v1.json");
    let v0 = &vectors["projection_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "projection.reject_evidence_v0")
        .unwrap()["input"];
    assert_eq!(
        reject_non_v1_evidence(&serde_json::to_vec(v0).unwrap())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::InvalidEvidence
    );
    assert_eq!(
        reject_non_v1_evidence(&vec![b' '; 2 * 1024 * 1024 + 1])
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::InputTooLarge
    );
}

#[test]
fn ai_explain_v1_synthetic_alias_vector_is_deterministic_and_rejects_bidi() {
    let properties = vec![
        SyntheticPropertyV1 {
            original_index: 0,
            original_id: "z-secret".to_owned(),
            category: "unrecognized".to_owned(),
            status: SourcePropertyStatusV1::Unsupported,
            evidence_kinds: vec![SanitizedEvidenceKindV1::UnsupportedFeature],
        },
        SyntheticPropertyV1 {
            original_index: 1,
            original_id: "a-secret".to_owned(),
            category: "unrecognized".to_owned(),
            status: SourcePropertyStatusV1::MpkVerified,
            evidence_kinds: vec![SanitizedEvidenceKindV1::CheckedDeclaration],
        },
        SyntheticPropertyV1 {
            original_index: 2,
            original_id: "m-secret".to_owned(),
            category: "unrecognized".to_owned(),
            status: SourcePropertyStatusV1::MpkVerified,
            evidence_kinds: vec![
                SanitizedEvidenceKindV1::CheckedDeclaration,
                SanitizedEvidenceKindV1::CheckedTheoryCertificate,
            ],
        },
    ];
    let aliases = project_synthetic_properties_v1(properties).unwrap();
    assert_eq!(
        aliases
            .iter()
            .map(|alias| (alias.property_ref.as_str(), alias.original_index))
            .collect::<Vec<_>>(),
        [
            ("property-0001", 1),
            ("property-0002", 2),
            ("property-0003", 0)
        ]
    );
    assert_eq!(
        project_synthetic_properties_v1(vec![SyntheticPropertyV1 {
            original_index: 0,
            original_id: "safe\u{202e}evil".to_owned(),
            category: "unrecognized".to_owned(),
            status: SourcePropertyStatusV1::HelperOnly,
            evidence_kinds: vec![SanitizedEvidenceKindV1::HelperArtifact],
        }])
        .unwrap_err()
        .code(),
        AiExplainV1ErrorCode::InvalidEvidence
    );
    assert_eq!(
        project_synthetic_properties_v1(Vec::new())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::NoProperties
    );
}

#[test]
fn ai_explain_v1_report_restores_local_ids_and_matches_normative_transport() {
    let vectors = load_value("develop/specs/vectors/ai-explain-v1.json");
    let validated = validated_evidence("evidence.rust_call_pair_verified");
    let prepared = build_vertex_request_v1(&validated, ExplainLanguageV1::En).unwrap();
    let model_text = serde_json::to_vec(&json!({
        "overview": "The supplied MPK evidence marks one property as verified.",
        "property_explanations": [{
            "property_ref": "property-0001",
            "explanation": "Both supplied checker verdicts are accepted for this property."
        }],
        "limitations": ["This explanation is helper analysis, not proof evidence."],
        "next_steps": []
    }))
    .unwrap();
    let report = build_explanation_report_v1(
        &prepared,
        &ExplanationReportRequestV1 {
            project: "sample-project".to_owned(),
            location: "global".to_owned(),
            requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
            language: ExplainLanguageV1::En,
        },
        &ProviderProvenanceInputV1 {
            model_version: "gemini-3.5-flash-001".to_owned(),
            response_id: "resp-vector-1".to_owned(),
            create_time: "2026-08-21T00:00:00Z".to_owned(),
            finish_reason: "STOP".to_owned(),
            attempts: 1,
            prompt_tokens: None,
            thinking_tokens: None,
            response_tokens: None,
            total_tokens: None,
        },
        &model_text,
    )
    .unwrap();
    assert_eq!(
        report.ai_analysis.property_explanations[0].property_id,
        "caller_panic_free"
    );
    assert_eq!(
        report.ai_analysis.property_explanations[0].source_status,
        SourcePropertyStatusV1::MpkVerified
    );
    let fixture = vectors["explanation_fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fixture| fixture["id"] == "explanation.rust_verified.v1")
        .unwrap();
    assert_eq!(serde_json::to_value(&report).unwrap(), fixture["input"]);
    let transport = serialize_explanation_v1(&report).unwrap();
    assert_eq!(
        transport.len() as u64,
        fixture["pretty_transport_utf8_length"].as_u64().unwrap()
    );
    assert_eq!(sha256_hex(&transport), fixture["pretty_transport_sha256"]);
    assert_eq!(parse_explanation_v1(&transport).unwrap(), report);

    let mut changed_status = serde_json::to_value(&report).unwrap();
    changed_status["ai_analysis"]["property_explanations"][0]["source_status"] =
        Value::String("proof_pending".to_owned());
    assert_eq!(
        parse_explanation_v1(&serde_json::to_vec(&changed_status).unwrap())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::ResponseInvalid
    );

    let mut malformed_provenance = serde_json::to_value(&report).unwrap();
    malformed_provenance["provider_response"]["response_id"] =
        Value::String("response id with spaces".to_owned());
    assert_eq!(
        parse_explanation_v1(&serde_json::to_vec(&malformed_provenance).unwrap())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::ResponseInvalid
    );

    let historical = vectors["explanation_fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fixture| fixture["id"] == "explanation.historical_v0")
        .unwrap();
    assert_eq!(
        parse_explanation_v1(&serde_json::to_vec(&historical["input"]).unwrap())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::ResponseInvalid
    );

    let injected = json!({
        "overview": "Unsafe helper explanation.",
        "property_explanations": [{"property_ref": "property-0001", "explanation": "text"}],
        "limitations": [],
        "next_steps": [],
        "status": "mpk_verified"
    });
    assert_eq!(
        parse_provider_response_v0(&serde_json::to_vec(&injected).unwrap())
            .unwrap_err()
            .code(),
        AiExplainV1ErrorCode::ResponseInvalid
    );

    let english = render_explanation_markdown_v1(&report);
    assert_eq!(english, render_explanation_markdown_v1(&report));
    assert!(english.starts_with("> **UNTRUSTED AI-GENERATED EXPLANATION**"));
    assert!(english.contains("caller\\_panic\\_free [mpk_verified]"));
    let escaped_model_text = serde_json::to_vec(&json!({
        "overview": "Safe first line.\n    # generated heading",
        "property_explanations": [{
            "property_ref": "property-0001",
            "explanation": "Explanation."
        }],
        "limitations": [],
        "next_steps": []
    }))
    .unwrap();
    let escaped_report = build_explanation_report_v1(
        &prepared,
        &ExplanationReportRequestV1 {
            project: "sample-project".to_owned(),
            location: "global".to_owned(),
            requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
            language: ExplainLanguageV1::En,
        },
        &ProviderProvenanceInputV1 {
            model_version: "gemini-3.5-flash-001".to_owned(),
            response_id: "resp-vector-1".to_owned(),
            create_time: "2026-08-21T00:00:00Z".to_owned(),
            finish_reason: "STOP".to_owned(),
            attempts: 1,
            prompt_tokens: None,
            thinking_tokens: None,
            response_tokens: None,
            total_tokens: None,
        },
        &escaped_model_text,
    )
    .unwrap();
    assert!(render_explanation_markdown_v1(&escaped_report)
        .contains("&#32;&#32;&#32;&#32;\\# generated heading"));

    let ja_prepared = build_vertex_request_v1(&validated, ExplainLanguageV1::Ja).unwrap();
    let japanese_report = build_explanation_report_v1(
        &ja_prepared,
        &ExplanationReportRequestV1 {
            project: "sample-project".to_owned(),
            location: "global".to_owned(),
            requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
            language: ExplainLanguageV1::Ja,
        },
        &ProviderProvenanceInputV1 {
            model_version: "gemini-3.5-flash-001".to_owned(),
            response_id: "resp-vector-1".to_owned(),
            create_time: "2026-08-21T00:00:00Z".to_owned(),
            finish_reason: "STOP".to_owned(),
            attempts: 1,
            prompt_tokens: None,
            thinking_tokens: None,
            response_tokens: None,
            total_tokens: None,
        },
        &model_text,
    )
    .unwrap();
    let japanese = render_explanation_markdown_v1(&japanese_report);
    assert_eq!(japanese, render_explanation_markdown_v1(&japanese_report));
    assert!(japanese.starts_with("> **信頼できないAI生成の説明**"));
}

#[test]
fn ai_explain_v1_dry_run_is_no_clobber_and_never_reads_source() {
    let validated = validated_evidence("evidence.rust_call_pair_verified");
    let expected = build_vertex_request_v1(&validated, ExplainLanguageV1::En).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("src").join("lib.rs");
    fs::create_dir(directory.path().join("src")).unwrap();
    fs::write(&source, "MPK_AI_SOURCE_SENTINEL_DO_NOT_SEND_7f8e9d0c").unwrap();
    let evidence_path = directory.path().join("evidence.json");
    fs::write(&evidence_path, validated.canonical_bytes()).unwrap();
    let output = directory.path().join("request.json");
    let status = execute_dry_run_v1(
        &validated,
        &evidence_path,
        &output,
        DEFAULT_GEMINI_MODEL,
        ExplainLanguageV1::En,
    )
    .unwrap();
    assert!(status.contains("network=0"));
    assert_eq!(fs::read(&output).unwrap(), expected.request_body);
    assert!(!fs::read_to_string(&output)
        .unwrap()
        .contains("MPK_AI_SOURCE_SENTINEL_DO_NOT_SEND_7f8e9d0c"));
    assert_eq!(
        execute_dry_run_v1(
            &validated,
            &evidence_path,
            &output,
            DEFAULT_GEMINI_MODEL,
            ExplainLanguageV1::En,
        )
        .unwrap_err()
        .code(),
        AiExplainV1ErrorCode::OutputFailed
    );
}

#[test]
fn ai_explain_v1_limit_vectors_match_checked_boundaries() {
    let vectors = load_value("develop/specs/vectors/ai-explain-v1.json");
    for case in vectors["limit_cases"].as_array().unwrap() {
        let counter = case["counter"].as_str().unwrap();
        let limit = usize::try_from(case["limit"].as_u64().unwrap()).unwrap();
        let below = usize::try_from(case["below"]["count"].as_u64().unwrap()).unwrap();
        let at = usize::try_from(case["at"]["count"].as_u64().unwrap()).unwrap();
        let above = usize::try_from(case["above"]["count"].as_u64().unwrap()).unwrap();
        assert_eq!(below + 1, limit, "{}", case["id"]);
        assert_eq!(at, limit, "{}", case["id"]);
        assert_eq!(above, limit + 1, "{}", case["id"]);
        validate_limit_counter_v1(counter, below).unwrap();
        validate_limit_counter_v1(counter, at).unwrap();
        assert_eq!(
            validate_limit_counter_v1(counter, above)
                .unwrap_err()
                .code()
                .as_str(),
            case["above"]["code"].as_str().unwrap(),
            "{}",
            case["id"]
        );
    }
}

fn validated_evidence(id: &str) -> ValidatedPolicyEvidenceV1 {
    let scan: ScanVector = load("develop/specs/vectors/policy-scan-v1.json");
    let evidence: EvidenceVector = load("develop/specs/vectors/policy-evidence-v1.json");
    let scan_contexts = by_id(&scan.linkage_contexts, |value| &value.id);
    let scan_fixtures = by_id(&scan.fixtures, |value| &value.id);
    let evidence_contexts = by_id(&evidence.linkage_contexts, |value| &value.id);
    let fixture = evidence
        .fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing evidence fixture {id}"));
    let context = evidence_contexts[fixture.linkage_context.as_str()];
    let scan_fixture = scan_fixtures[context.scan_fixture.as_str()];
    let scan_context = scan_contexts[scan_fixture.linkage_context.as_str()];
    let validated_scan = import_policy_scan_v1_json(
        &canonical_transport(&scan_fixture.input),
        &scan_context.linkage(),
    )
    .unwrap();
    let (expected_members, expected_declarations) = normalized_context(context);
    let expected_certificate = context.accepted_certificate_hash.as_ref().map(|hash| {
        assert_eq!(context.accepted_certificate_id.as_deref(), Some("program"));
        PolicyExpectedCertificateV1 {
            module: "Policy.Generated".to_owned(),
            certificate_hash: hash.clone(),
            export_hash: context.accepted_export_hash.clone().unwrap(),
            axiom_report_hash: context.accepted_axiom_report_hash.clone().unwrap(),
        }
    });
    let trusted = baseline_trusted(fixture);
    let linkage = PolicyEvidenceLinkageContext {
        scan: &validated_scan,
        certificate_source_manifest_hash: context.certificate_source_manifest_hash.clone(),
        source_vc_schema: context.source_vc_schema.clone(),
        vc_hash: context.vc_hash.clone(),
        verification_limit_profile: context.verification_limit_profile.clone(),
        expected_members,
        expected_declarations,
        expected_certificate,
        expected_theory_certificates: trusted.theory_certificates,
        expected_axiom_report: trusted.axiom_report,
        expected_checker_verdicts: trusted.checker_verdicts,
        expected_properties: baseline_properties(fixture),
        expected_unsupported_codes: Vec::new(),
        expected_optional_helpers: Vec::new(),
    };
    import_policy_evidence_v1_json(&canonical_transport(&fixture.input), &linkage).unwrap()
}

fn normalized_context(
    context: &EvidenceContext,
) -> (Vec<PolicyExpectedMemberV1>, Vec<PolicyCheckedDeclaration>) {
    if !context.members.is_empty() {
        let declarations = context
            .members
            .iter()
            .map(|member| PolicyCheckedDeclaration {
                name: member.declaration_name.clone(),
                declaration_hash: member.declaration_hash.clone(),
                function_id: member.function_id.clone(),
                group_id: member.group_id.clone(),
                group_kind: member.group_id.rsplit_once('.').unwrap().1.to_owned(),
                member_ids: vec![member.member_id.clone()],
                dependencies: member.dependencies.clone(),
            })
            .collect::<Vec<_>>();
        let members = context
            .members
            .iter()
            .map(|member| PolicyExpectedMemberV1 {
                member_id: member.member_id.clone(),
                function_id: member.function_id.clone(),
                kind: member.kind.clone(),
                group_id: member.group_id.clone(),
                declaration_name: member.declaration_name.clone(),
                declaration_hash: member.declaration_hash.clone(),
            })
            .collect();
        return (members, declarations);
    }
    let members = context
        .declarations
        .iter()
        .flat_map(|declaration| {
            declaration.member_ids.iter().map(|member_id| {
                let mut parts = member_id.rsplitn(3, '#');
                let _ordinal = parts.next().unwrap();
                let kind = parts.next().unwrap();
                PolicyExpectedMemberV1 {
                    member_id: member_id.clone(),
                    function_id: declaration.function_id.clone(),
                    kind: kind.to_owned(),
                    group_id: declaration.group_id.clone(),
                    declaration_name: declaration.name.clone(),
                    declaration_hash: declaration.declaration_hash.clone(),
                }
            })
        })
        .collect();
    (members, context.declarations.clone())
}

fn baseline_trusted(fixture: &Fixture) -> PolicyTrustedEvidenceV1 {
    serde_json::from_value(fixture.input["trusted_evidence"].clone()).unwrap()
}

fn baseline_properties(fixture: &Fixture) -> Vec<PolicyExpectedPropertyV1> {
    serde_json::from_value::<Vec<PolicyPropertyV1>>(fixture.input["properties"].clone())
        .unwrap()
        .into_iter()
        .map(|property| PolicyExpectedPropertyV1 {
            id: property.id,
            description: property.description,
            member_ids: property
                .members
                .into_iter()
                .map(|member| member.member_id)
                .collect(),
            notes: property.notes,
        })
        .collect()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(&serialized, POLICY_LIMITS).unwrap();
    let mut bytes = canonical_json_bytes(&strict).unwrap();
    bytes.push(b'\n');
    bytes
}

fn language(value: &str) -> ExplainLanguageV1 {
    match value {
        "en" => ExplainLanguageV1::En,
        "ja" => ExplainLanguageV1::Ja,
        _ => panic!("unknown language {value}"),
    }
}

fn by_id<T>(values: &[T], id: impl Fn(&T) -> &str) -> BTreeMap<&str, &T> {
    values.iter().map(|value| (id(value), value)).collect()
}

fn load<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    serde_json::from_slice(&fs::read(repo_path(relative)).unwrap()).unwrap()
}

fn load_value(relative: &str) -> Value {
    load(relative)
}

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
