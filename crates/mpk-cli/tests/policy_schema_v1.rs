use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use mpk_cli::policy_schema::{
    canonical_policy_evidence_v1_json, canonical_policy_scan_v1_json,
    import_policy_evidence_v1_json, import_policy_scan_v1_json, validate_policy_limit,
    PolicyCheckedDeclaration, PolicyEvidenceLinkageContext, PolicyExpectedCertificateV1,
    PolicyExpectedMemberV1, PolicyExpectedPropertyV1, PolicyHelperArtifact, PolicyIssue,
    PolicyPropertyV1, PolicyScanLinkageContext, PolicySelection, PolicySemanticParameters,
    PolicyTrustedEvidenceV1,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, FrontendIdentity, ReleaseRegistryIdentity,
    StrictJsonLimits, ToolchainIdentity,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const POLICY_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScanVector {
    schema: String,
    spec_schemas: Vec<String>,
    dependencies: Value,
    owner_test: String,
    linkage_contexts: Vec<ScanContext>,
    fixtures: Vec<Fixture>,
    scan_cases: Vec<Case>,
    limit_cases: Vec<LimitCase>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    linkage_context: String,
    canonical_transport_utf8_length: u64,
    canonical_transport_sha256: String,
    input: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    input_from: Option<String>,
    transport_from: Option<TransportFrom>,
    json_text: Option<String>,
    construction: Option<Construction>,
    expect: Expectation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransportFrom {
    fixture: String,
    encoding: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Construction {
    base: String,
    patches: Vec<Patch>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Patch {
    Add {
        path: String,
        value: Value,
    },
    Remove {
        path: String,
    },
    Replace {
        path: String,
        value: Value,
    },
    Copy {
        from: String,
        path: String,
    },
    Swap {
        path: String,
        first: usize,
        second: usize,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expectation {
    outcome: String,
    phase: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitCase {
    id: String,
    limit: u64,
    counter: String,
    below: LimitPoint,
    at: LimitPoint,
    above: LimitPoint,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitPoint {
    count: u64,
    outcome: String,
    phase: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceVector {
    schema: String,
    spec_schemas: Vec<String>,
    dependencies: Value,
    owner_test: String,
    linkage_contexts: Vec<EvidenceContext>,
    fixtures: Vec<Fixture>,
    evidence_cases: Vec<Case>,
    limit_cases: Vec<LimitCase>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceContext {
    id: String,
    scan_fixture: String,
    #[serde(default)]
    frontend_manifest_case: Option<String>,
    #[serde(default)]
    certificate_manifest_case: Option<String>,
    #[serde(default)]
    vc_fixture: Option<String>,
    frontend_source_manifest_hash: String,
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
#[serde(deny_unknown_fields)]
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
fn policy_scan_v1_executes_every_normative_case() {
    let vector: ScanVector = load("develop/specs/vectors/policy-scan-v1.json");
    assert_eq!(vector.schema, "mpk.policy.scan.conformance.v1");
    assert_eq!(vector.spec_schemas[0], "mpk.policy.scan.v1");
    assert!(vector.dependencies.is_object());
    assert_eq!(
        vector.owner_test,
        "crates/mpk-cli/tests/policy_schema_v1.rs"
    );
    run_scan_cases(&vector);
    run_limit_cases(&vector.limit_cases);
}

#[test]
fn policy_evidence_v1_executes_every_normative_case() {
    let scan: ScanVector = load("develop/specs/vectors/policy-scan-v1.json");
    let vector: EvidenceVector = load("develop/specs/vectors/policy-evidence-v1.json");
    assert_eq!(vector.schema, "mpk.policy.evidence.conformance.v1");
    assert_eq!(vector.spec_schemas[0], "mpk.policy.evidence.v1");
    assert!(vector.dependencies.is_object());
    assert_eq!(
        vector.owner_test,
        "crates/mpk-cli/tests/policy_schema_v1.rs"
    );

    let scan_contexts = by_id(&scan.linkage_contexts, |value| &value.id);
    let scan_fixtures = by_id(&scan.fixtures, |value| &value.id);
    let contexts = by_id(&vector.linkage_contexts, |value| &value.id);
    let fixtures = by_id(&vector.fixtures, |value| &value.id);
    assert_unique_case_ids(&vector.evidence_cases);

    for fixture in &vector.fixtures {
        assert_fixture_transport(fixture);
    }

    let mut visited = BTreeSet::new();
    for case in &vector.evidence_cases {
        assert!(
            visited.insert(case.id.clone()),
            "duplicate case {}",
            case.id
        );
        let (bytes, fixture) = case_bytes(case, &fixtures);
        let context = &contexts[fixture.linkage_context.as_str()];
        assert_eq!(
            context.frontend_source_manifest_hash,
            scan_fixtures[context.scan_fixture.as_str()].input["frontend_source_manifest_hash"]
        );
        assert!(
            context
                .frontend_manifest_case
                .as_deref()
                .unwrap_or("")
                .is_empty()
                || context
                    .frontend_manifest_case
                    .as_deref()
                    .unwrap()
                    .starts_with("manifest.")
        );
        assert!(
            context
                .certificate_manifest_case
                .as_deref()
                .unwrap_or("")
                .is_empty()
                || context
                    .certificate_manifest_case
                    .as_deref()
                    .unwrap()
                    .starts_with("lifecycle.")
        );
        assert!(
            context.vc_fixture.as_deref().unwrap_or("").is_empty()
                || context.vc_fixture.as_deref().unwrap().starts_with("vc.")
        );

        let scan_fixture = &scan_fixtures[context.scan_fixture.as_str()];
        let scan_context = &scan_contexts[scan_fixture.linkage_context.as_str()];
        let scan_bytes = canonical_transport(&scan_fixture.input);
        let validated_scan = import_policy_scan_v1_json(&scan_bytes, &scan_context.linkage())
            .expect("evidence context uses a validated ready scan");
        let (expected_members, expected_declarations) = normalized_evidence_context(context);
        let expected_certificate = context.accepted_certificate_hash.as_ref().map(|hash| {
            assert_eq!(context.accepted_certificate_id.as_deref(), Some("program"));
            PolicyExpectedCertificateV1 {
                module: "Policy.Generated".to_owned(),
                certificate_hash: hash.clone(),
                export_hash: context.accepted_export_hash.clone().expect("export hash"),
                axiom_report_hash: context
                    .accepted_axiom_report_hash
                    .clone()
                    .expect("axiom report hash"),
            }
        });
        let linkage = PolicyEvidenceLinkageContext {
            scan: &validated_scan,
            certificate_source_manifest_hash: context.certificate_source_manifest_hash.clone(),
            source_vc_schema: context.source_vc_schema.clone(),
            vc_hash: context.vc_hash.clone(),
            verification_limit_profile: context.verification_limit_profile.clone(),
            expected_members,
            expected_declarations,
            expected_certificate,
            expected_theory_certificates: baseline_trusted(fixture).theory_certificates,
            expected_axiom_report: baseline_trusted(fixture).axiom_report,
            expected_checker_verdicts: baseline_trusted(fixture).checker_verdicts,
            expected_properties: baseline_properties(fixture),
            expected_unsupported_codes: Vec::new(),
            expected_optional_helpers: Vec::new(),
        };
        let result = import_policy_evidence_v1_json(&bytes, &linkage);
        if let Ok(validated) = &result {
            assert_eq!(validated.canonical_bytes(), bytes, "{}", case.id);
            assert_eq!(
                canonical_policy_evidence_v1_json(validated.document()).unwrap(),
                bytes,
                "{}",
                case.id
            );
        }
        assert_outcome(case, result.map(|_| ()));
    }
    assert_eq!(visited.len(), vector.evidence_cases.len());
    run_limit_cases(&vector.limit_cases);
}

#[test]
fn policy_field_limits_precede_schema_shape_validation() {
    let vector: ScanVector = load("develop/specs/vectors/policy-scan-v1.json");
    let context = vector.linkage_contexts.first().expect("scan context");
    let value = serde_json::json!({
        "schema": "mpk.policy.scan.v0",
        "helper_artifacts": vec![Value::Null; 65_537],
    });
    let error = import_policy_scan_v1_json(&canonical_transport(&value), &context.linkage())
        .expect_err("field-specific transport limit rejects before v0 schema");

    assert_eq!(error.phase().as_str(), "transport");
    assert_eq!(error.code(), "POLICY_LIMIT_COLLECTION");
}

#[test]
fn policy_public_text_rejects_file_locators() {
    let vector: ScanVector = load("develop/specs/vectors/policy-scan-v1.json");
    let contexts = by_id(&vector.linkage_contexts, |value| &value.id);
    let fixture = vector
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "scan.go_identity_ready")
        .expect("Go scan fixture");
    let context = &contexts[fixture.linkage_context.as_str()];
    let mut value = fixture.input.clone();
    value["diagnostics"] = serde_json::json!([{
        "code": "GO_SOURCE_READ",
        "message": "failed to read FILE:/tmp/source.go"
    }]);
    let error = import_policy_scan_v1_json(&canonical_transport(&value), &context.linkage())
        .expect_err("machine-local file locator rejects");

    assert_eq!(error.phase().as_str(), "scalar");
    assert_eq!(error.code(), "POLICY_SCALAR");
}

fn baseline_trusted(fixture: &Fixture) -> PolicyTrustedEvidenceV1 {
    serde_json::from_value(fixture.input["trusted_evidence"].clone())
        .expect("fixture has closed trusted evidence")
}

fn baseline_properties(fixture: &Fixture) -> Vec<PolicyExpectedPropertyV1> {
    serde_json::from_value::<Vec<PolicyPropertyV1>>(fixture.input["properties"].clone())
        .expect("fixture has closed properties")
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

fn run_scan_cases(vector: &ScanVector) {
    let contexts = by_id(&vector.linkage_contexts, |value| &value.id);
    let fixtures = by_id(&vector.fixtures, |value| &value.id);
    assert_unique_case_ids(&vector.scan_cases);
    for fixture in &vector.fixtures {
        assert_fixture_transport(fixture);
    }
    let mut visited = BTreeSet::new();
    for case in &vector.scan_cases {
        assert!(
            visited.insert(case.id.clone()),
            "duplicate case {}",
            case.id
        );
        let (bytes, fixture) = case_bytes(case, &fixtures);
        let context = &contexts[fixture.linkage_context.as_str()];
        let result = import_policy_scan_v1_json(&bytes, &context.linkage());
        if let Ok(validated) = &result {
            assert_eq!(validated.canonical_bytes(), bytes, "{}", case.id);
            assert_eq!(
                canonical_policy_scan_v1_json(validated.document()).unwrap(),
                bytes,
                "{}",
                case.id
            );
        }
        assert_outcome(case, result.map(|_| ()));
    }
    assert_eq!(visited.len(), vector.scan_cases.len());
}

fn normalized_evidence_context(
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
                group_kind: member
                    .group_id
                    .rsplit_once('.')
                    .expect("group suffix")
                    .1
                    .to_owned(),
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
                let _ordinal = parts.next().expect("member ordinal");
                let kind = parts.next().expect("member kind");
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

fn run_limit_cases(cases: &[LimitCase]) {
    let mut ids = BTreeSet::new();
    for case in cases {
        assert!(ids.insert(&case.id));
        assert_eq!(case.below.count + 1, case.limit, "{}", case.id);
        assert_eq!(case.at.count, case.limit, "{}", case.id);
        assert_eq!(case.above.count, case.limit + 1, "{}", case.id);
        for point in [&case.below, &case.at] {
            validate_policy_limit(&case.counter, point.count)
                .unwrap_or_else(|error| panic!("{} should accept: {error}", case.id));
            assert_eq!(point.outcome, "accept");
            assert!(point.phase.is_none());
            assert!(point.code.is_none());
        }
        let error = validate_policy_limit(&case.counter, case.above.count)
            .expect_err("above limit rejects");
        assert_eq!(case.above.outcome, "reject");
        assert_eq!(error.phase().as_str(), case.above.phase.as_deref().unwrap());
        assert_eq!(error.code(), case.above.code.as_deref().unwrap());
    }
}

fn assert_fixture_transport(fixture: &Fixture) {
    let bytes = canonical_transport(&fixture.input);
    assert_eq!(bytes.len() as u64, fixture.canonical_transport_utf8_length);
    assert_eq!(sha256_hex(&bytes), fixture.canonical_transport_sha256);
}

fn case_bytes<'a>(
    case: &Case,
    fixtures: &'a BTreeMap<&str, &'a Fixture>,
) -> (Vec<u8>, &'a Fixture) {
    let source_count = usize::from(case.input_from.is_some())
        + usize::from(case.transport_from.is_some())
        + usize::from(case.json_text.is_some())
        + usize::from(case.construction.is_some());
    assert_eq!(source_count, 1, "{} has one input source", case.id);
    if let Some(id) = &case.input_from {
        let fixture = fixtures[id.as_str()];
        return (canonical_transport(&fixture.input), fixture);
    }
    if let Some(transport) = &case.transport_from {
        let fixture = fixtures[transport.fixture.as_str()];
        let bytes = match transport.encoding.as_str() {
            "two_space_indent_with_final_lf" => {
                let mut text = serde_json::to_string_pretty(&fixture.input).unwrap();
                text.push('\n');
                text.into_bytes()
            }
            "jcs_without_final_lf" => canonical_without_lf(&fixture.input),
            other => panic!("unknown transport encoding {other}"),
        };
        return (bytes, fixture);
    }
    if let Some(text) = &case.json_text {
        return (
            text.as_bytes().to_vec(),
            fixtures.values().next().copied().unwrap(),
        );
    }
    let construction = case.construction.as_ref().unwrap();
    let fixture = fixtures[construction.base.as_str()];
    let mut value = fixture.input.clone();
    for patch in &construction.patches {
        apply_patch(&mut value, patch);
    }
    (canonical_transport(&value), fixture)
}

fn apply_patch(value: &mut Value, patch: &Patch) {
    match patch {
        Patch::Add { path, value: added } => add_pointer(value, path, added.clone()),
        Patch::Remove { path } => {
            take_pointer(value, path);
        }
        Patch::Replace { path, value: next } => {
            *value.pointer_mut(path).unwrap() = next.clone();
        }
        Patch::Copy { from, path } => {
            let copied = value.pointer(from).unwrap().clone();
            add_pointer(value, path, copied);
        }
        Patch::Swap {
            path,
            first,
            second,
        } => {
            value
                .pointer_mut(path)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .swap(*first, *second);
        }
    }
}

fn add_pointer(root: &mut Value, pointer: &str, value: Value) {
    let (parent, leaf) = pointer.rsplit_once('/').expect("non-root pointer");
    let parent = root.pointer_mut(parent).expect("patch parent");
    if let Some(object) = parent.as_object_mut() {
        assert!(object.insert(unescape_pointer(leaf), value).is_none());
    } else {
        let index = leaf.parse::<usize>().expect("array index");
        parent.as_array_mut().unwrap().insert(index, value);
    }
}

fn take_pointer(root: &mut Value, pointer: &str) -> Value {
    let (parent, leaf) = pointer.rsplit_once('/').expect("non-root pointer");
    let parent = root.pointer_mut(parent).expect("patch parent");
    if let Some(object) = parent.as_object_mut() {
        object
            .remove(&unescape_pointer(leaf))
            .expect("object member")
    } else {
        parent
            .as_array_mut()
            .unwrap()
            .remove(leaf.parse::<usize>().expect("array index"))
    }
}

fn unescape_pointer(value: &str) -> String {
    value.replace("~1", "/").replace("~0", "~")
}

fn assert_outcome(case: &Case, result: Result<(), mpk_cli::policy_schema::PolicyValidationError>) {
    match case.expect.outcome.as_str() {
        "accept" => {
            result.unwrap_or_else(|error| panic!("{} unexpectedly rejected: {error}", case.id));
            assert!(case.expect.phase.is_none());
            assert!(case.expect.code.is_none());
        }
        "reject" => {
            let error = result.unwrap_err();
            assert_eq!(
                error.phase().as_str(),
                case.expect.phase.as_deref().unwrap(),
                "{}: {error}",
                case.id
            );
            assert_eq!(
                error.code(),
                case.expect.code.as_deref().unwrap(),
                "{}: {error}",
                case.id
            );
        }
        other => panic!("unknown outcome {other}"),
    }
}

fn assert_unique_case_ids(cases: &[Case]) {
    let mut ids = BTreeSet::new();
    for case in cases {
        assert!(ids.insert(&case.id), "duplicate case {}", case.id);
    }
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = canonical_without_lf(value);
    bytes.push(b'\n');
    bytes
}

fn canonical_without_lf(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(&serialized, POLICY_LIMITS).unwrap();
    canonical_json_bytes(&strict).unwrap()
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn by_id<T>(values: &[T], id: impl Fn(&T) -> &str) -> BTreeMap<&str, &T> {
    let result = values
        .iter()
        .map(|value| (id(value), value))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(result.len(), values.len());
    result
}

fn load<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}
