use mpk_cli::policy_report::render_policy_evidence_v1_markdown;
use mpk_cli::policy_schema::{
    expected_reproduction_recipes, import_policy_evidence_v1_json, import_policy_scan_v1_json,
    render_posix_argv, PolicyAxiomReportV1, PolicyCheckedDeclaration, PolicyEvidenceLinkageContext,
    PolicyEvidenceV1, PolicyExpectedCertificateV1, PolicyExpectedMemberV1,
    PolicyExpectedPropertyV1, PolicyHelperArtifact, PolicyIssue, PolicyScanLinkageContext,
    PolicySelection, PolicySemanticParameters, PolicyTheoryCertificateEvidenceV1,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, FrontendIdentity, ReleaseRegistryIdentity,
    StrictJsonLimits, ToolchainIdentity,
};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn validated_v1_evidence_renders_all_frozen_sections_deterministically() {
    let evidence = validated_rust_v1_evidence();
    let first = render_policy_evidence_v1_markdown(&evidence).expect("v1 report renders");
    let second = render_policy_evidence_v1_markdown(&evidence).expect("repeat render succeeds");
    assert_eq!(first, second);
    assert!(first.ends_with('\n'));
    assert!(!first.ends_with("\n\n"));
    assert!(!first.contains('\r'));
    assert!(first.lines().all(|line| !line.ends_with(' ')));
    assert!(!first.contains("/Users/"));
    assert!(!first.contains("checker command"));
    assert!(first.contains("- Untrusted helper `source:src/lib.rs`"));
    assert!(first.contains("Frontend source manifest SHA-256"));
    assert!(first.contains("Certificate source manifest SHA-256"));
    assert!(first.contains("Dependency `VC.Function."));
    assert!(first.contains("Only checker-accepted canonical certificate and theory-certificate bytes are trusted evidence."));
    assert!(first.contains("Policy JSON, source text, contracts, VIR, VC, AI analysis, CI status, and this Markdown report are not proof evidence."));

    let headings = first
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect::<Vec<_>>();
    assert_eq!(
        headings,
        [
            "Target and Profiles",
            "Source and Release Identities",
            "Verification Summary",
            "Properties",
            "Trusted Evidence",
            "Helper Artifacts",
            "Reproduction Recipes",
            "Trust-Boundary Notes",
        ]
    );
}

#[test]
fn rejected_v1_candidate_remains_reportable_but_not_verified() {
    let evidence = validated_rust_v1_evidence_with(|document| {
        document.trusted_evidence.checker_verdicts[0].verdict = "rejected".to_owned();
        document.properties[0].status = "proof_pending".to_owned();
        document.properties[0].members[0].status = "proof_pending".to_owned();
        document.properties[0].members[0].evidence = vec![
            mpk_cli::policy_schema::PolicyEvidenceReferenceV1::HelperArtifact {
                artifact_id: "vc".to_owned(),
            },
        ];
        document.reproduction_recipes = expected_reproduction_recipes(document);
    });
    let markdown = render_policy_evidence_v1_markdown(&evidence).expect("report renders");

    assert!(markdown.contains("- mpk_verified: `0`\n"));
    assert!(markdown.contains("- proof_pending: `1`\n"));
    assert!(markdown.contains("- Verdict: `rejected`\n"));
}

#[test]
fn policy_v1_posix_display_executes_every_render_vector() {
    let vectors = load_value("develop/specs/vectors/policy-recipes-v1.json");
    assert_eq!(
        vectors["owner_test"],
        "crates/mpk-cli/tests/policy_recipes_v1.rs"
    );
    let mut ids = std::collections::BTreeSet::new();
    for case in vectors["render_cases"].as_array().expect("render cases") {
        let id = case["id"].as_str().expect("render ID");
        assert!(ids.insert(id));
        let argv = serde_json::from_value::<Vec<String>>(case["argv"].clone()).unwrap();
        assert_eq!(render_posix_argv(&argv), case["expected_posix"], "{id}");
    }
    assert_eq!(ids.len(), vectors["render_cases"].as_array().unwrap().len());
}

#[test]
fn policy_v1_recipe_builder_executes_every_recipe_vector() {
    let recipes = load_value("develop/specs/vectors/policy-recipes-v1.json");
    let evidence = load_value("develop/specs/vectors/policy-evidence-v1.json");
    let mut ids = std::collections::BTreeSet::new();
    for case in recipes["recipe_cases"].as_array().expect("recipe cases") {
        let id = case["id"].as_str().unwrap();
        assert!(ids.insert(id));
        let fixture_id = match case["invocation"].as_str().unwrap() {
            "invocation.go_verify" => "evidence.go_identity_pending",
            "invocation.rust_verify_fixture_update" => "evidence.rust_call_pair_verified",
            other => panic!("unknown recipe invocation {other}"),
        };
        let mut document: PolicyEvidenceV1 =
            serde_json::from_value(find_id(&evidence["fixtures"], fixture_id)["input"].clone())
                .unwrap();
        let invocation = find_id(
            &recipes["invocations"],
            case["invocation"].as_str().unwrap(),
        );
        document.verification_options.strict = invocation["parsed"]["strict"].as_bool().unwrap();
        document.verification_options.update_fixtures =
            invocation["parsed"]["update_fixtures"].as_bool().unwrap();
        assert_eq!(
            serde_json::to_value(expected_reproduction_recipes(&document)).unwrap(),
            case["expect"]["recipes"],
            "{id}"
        );
    }
    assert_eq!(ids.len(), recipes["recipe_cases"].as_array().unwrap().len());
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportScanContext {
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
    limit_profile: String,
    frontend_source_manifest_hash: String,
    input_set_hash: String,
    source_map_hash: String,
    source_ir_schema: String,
    source_ir_hash: String,
    helper_artifacts: Vec<PolicyHelperArtifact>,
    rejected_features: Vec<PolicyIssue>,
    diagnostics: Vec<PolicyIssue>,
}

fn validated_rust_v1_evidence() -> mpk_cli::policy_schema::ValidatedPolicyEvidenceV1 {
    validated_rust_v1_evidence_with(|_| {})
}

fn validated_rust_v1_evidence_with(
    mutate: impl FnOnce(&mut PolicyEvidenceV1),
) -> mpk_cli::policy_schema::ValidatedPolicyEvidenceV1 {
    let scan_vectors = load_value("develop/specs/vectors/policy-scan-v1.json");
    let evidence_vectors = load_value("develop/specs/vectors/policy-evidence-v1.json");
    let scan_fixture = find_id(&scan_vectors["fixtures"], "scan.rust_call_pair_ready");
    let scan_context_value = find_id(
        &scan_vectors["linkage_contexts"],
        scan_fixture["linkage_context"].as_str().unwrap(),
    );
    let scan_context: ReportScanContext =
        serde_json::from_value(scan_context_value.clone()).unwrap();
    assert_eq!(scan_context.id, "context.rust_call_pair_ready");
    let scan_linkage = PolicyScanLinkageContext {
        frontend_status: scan_context.frontend_status,
        frontend_phase: scan_context.frontend_phase,
        source_language: scan_context.source_language,
        semantic_profile: scan_context.semantic_profile,
        semantic_parameters: scan_context.semantic_parameters,
        selection: scan_context.selection,
        release_registry: scan_context.release_registry,
        frontend: scan_context.frontend,
        toolchain: scan_context.toolchain,
        rejected_features: scan_context.rejected_features,
        diagnostics: scan_context.diagnostics,
        limit_profile: Some(scan_context.limit_profile),
        frontend_source_manifest_hash: Some(scan_context.frontend_source_manifest_hash),
        input_set_hash: Some(scan_context.input_set_hash),
        source_map_hash: Some(scan_context.source_map_hash),
        source_ir_schema: Some(scan_context.source_ir_schema),
        source_ir_hash: Some(scan_context.source_ir_hash),
        helper_artifacts: Some(scan_context.helper_artifacts),
    };
    let scan_bytes = canonical_transport(&scan_fixture["input"]);
    let scan = import_policy_scan_v1_json(&scan_bytes, &scan_linkage).unwrap();

    let evidence_fixture = find_id(
        &evidence_vectors["fixtures"],
        "evidence.rust_call_pair_verified",
    );
    let mut document: PolicyEvidenceV1 =
        serde_json::from_value(evidence_fixture["input"].clone()).unwrap();
    mutate(&mut document);
    let context = find_id(
        &evidence_vectors["linkage_contexts"],
        evidence_fixture["linkage_context"].as_str().unwrap(),
    );
    let declarations =
        serde_json::from_value::<Vec<PolicyCheckedDeclaration>>(context["declarations"].clone())
            .unwrap();
    let expected_members = declarations
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
    let expected_certificate = PolicyExpectedCertificateV1 {
        module: document.trusted_evidence.certificates[0].module.clone(),
        certificate_hash: context["accepted_certificate_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
        export_hash: context["accepted_export_hash"].as_str().unwrap().to_owned(),
        axiom_report_hash: context["accepted_axiom_report_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
    };
    let expected_theory_certificates: Vec<PolicyTheoryCertificateEvidenceV1> =
        document.trusted_evidence.theory_certificates.clone();
    let expected_axiom_report: PolicyAxiomReportV1 = document.trusted_evidence.axiom_report.clone();
    let expected_checker_verdicts = document.trusted_evidence.checker_verdicts.clone();
    let expected_properties = document
        .properties
        .iter()
        .map(|property| PolicyExpectedPropertyV1 {
            id: property.id.clone(),
            description: property.description.clone(),
            member_ids: property
                .members
                .iter()
                .map(|member| member.member_id.clone())
                .collect(),
            notes: property.notes.clone(),
        })
        .collect();
    let linkage = PolicyEvidenceLinkageContext {
        scan: &scan,
        certificate_source_manifest_hash: context["certificate_source_manifest_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
        source_vc_schema: context["source_vc_schema"].as_str().unwrap().to_owned(),
        vc_hash: context["vc_hash"].as_str().unwrap().to_owned(),
        verification_limit_profile: context["verification_limit_profile"]
            .as_str()
            .unwrap()
            .to_owned(),
        expected_members,
        expected_declarations: declarations,
        expected_certificate: Some(expected_certificate),
        expected_theory_certificates,
        expected_axiom_report,
        expected_checker_verdicts,
        expected_properties,
        expected_unsupported_codes: Vec::new(),
        expected_optional_helpers: Vec::new(),
    };
    let evidence_bytes = canonical_transport(&serde_json::to_value(document).unwrap());
    import_policy_evidence_v1_json(&evidence_bytes, &linkage).unwrap()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576),
    )
    .unwrap();
    let mut bytes = canonical_json_bytes(&strict).unwrap();
    bytes.push(b'\n');
    bytes
}

fn find_id<'a>(values: &'a Value, id: &str) -> &'a Value {
    values
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["id"] == id)
        .unwrap_or_else(|| panic!("missing vector ID {id}"))
}

fn load_value(relative: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
