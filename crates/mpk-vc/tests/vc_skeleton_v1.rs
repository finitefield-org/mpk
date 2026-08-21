use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use mpk_vc::{
    canonical_json_bytes, emit_validated_vc_skeleton_v1, emit_vc_skeleton_v1,
    import_vc_skeleton_v1_json, import_vc_v1_json, parse_strict_json,
    validate_policy_member_binding, validate_verification_limit, GroupedTheoremType,
    PolicyMemberBindingError, StrictJsonLimits, VcSourceContext, VcTerm,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const VECTOR_SCHEMA: &str = "mpk.vc.cert_skeleton.conformance.v1";
const SPEC_SCHEMA: &str = "mpk.vc.cert_skeleton.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    schema: String,
    spec_schema: String,
    source_vector: String,
    owner_test: String,
    fixture_digest_semantics: DigestSemantics,
    construction_operations: Vec<ConstructionOperation>,
    emission_cases: Vec<EmissionCase>,
    mutation_cases: Vec<MutationCase>,
    limit_cases: Vec<LimitCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DigestSemantics {
    algorithm: String,
    payload: String,
    artifact_identity: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstructionOperation {
    op: String,
    rule: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmissionCase {
    id: String,
    construction: Construction,
    expect: EmissionExpectation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmissionExpectation {
    outcome: Outcome,
    canonical_jcs_utf8_length: usize,
    canonical_jcs_sha256: String,
    declarations: Vec<DeclarationExpectation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclarationExpectation {
    name: String,
    group_id: String,
    member_ids: Vec<String>,
    dependencies: Vec<String>,
    theorem_type_jcs_utf8_length: usize,
    theorem_type_jcs_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationCase {
    id: String,
    construction: Option<Construction>,
    json_text: Option<String>,
    expect: Expectation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Construction {
    emit_from: String,
    mutations: Vec<Mutation>,
    transport_encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Mutation {
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
        first: u64,
        second: u64,
    },
    Reverse {
        path: String,
    },
    RightAssociateGroupMembers {
        declaration_index: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expectation {
    outcome: Outcome,
    phase: Option<String>,
    code: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Outcome {
    Accept,
    Reject,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitCase {
    id: String,
    limit_id: String,
    maximum: u64,
    below: LimitPoint,
    at: LimitPoint,
    above: LimitPoint,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitPoint {
    construction: LimitConstruction,
    expect: Expectation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitConstruction {
    fixture: String,
    count: u64,
}

#[derive(Clone)]
struct VcFixture {
    context_id: String,
    input: Value,
}

#[test]
fn emits_every_canonical_skeleton_vector() {
    let vector = load_vector();
    validate_vector_header(&vector);
    assert_eq!(vector.emission_cases.len(), 3);
    let (contexts, fixtures) = load_vc_sources(&vector);

    for case in &vector.emission_cases {
        assert_eq!(case.expect.outcome, Outcome::Accept, "{}", case.id);
        assert!(case.construction.mutations.is_empty(), "{}", case.id);
        assert!(
            case.construction.transport_encoding.is_none(),
            "{}",
            case.id
        );
        let fixture = &fixtures[case.construction.emit_from.as_str()];
        let context = &contexts[fixture.context_id.as_str()];
        let vc_bytes = canonical_value(&fixture.input);
        let first = emit_vc_skeleton_v1(&vc_bytes, context)
            .unwrap_or_else(|error| panic!("{} rejected: {error}", case.id));
        let second = emit_vc_skeleton_v1(&vc_bytes, context).expect("repeat emission");
        let validated_vc = import_vc_v1_json(&vc_bytes, context).expect("source VC imports");
        let retained =
            emit_validated_vc_skeleton_v1(&validated_vc).expect("retained validated VC emits");
        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(first.canonical_bytes(), retained.canonical_bytes());
        assert_eq!(
            first.canonical_bytes().len(),
            case.expect.canonical_jcs_utf8_length,
            "{}",
            case.id
        );
        assert_eq!(
            sha256(first.canonical_bytes()),
            case.expect.canonical_jcs_sha256
        );
        assert_eq!(
            first.skeleton().theorem_declarations.len(),
            case.expect.declarations.len(),
            "{}",
            case.id
        );
        for (actual, expected) in first
            .skeleton()
            .theorem_declarations
            .iter()
            .zip(&case.expect.declarations)
        {
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.group_id, expected.group_id);
            assert_eq!(actual.member_ids, expected.member_ids);
            assert_eq!(actual.dependencies, expected.dependencies);
            let theorem_bytes = canonical_serializable(&actual.theorem_type);
            assert_eq!(theorem_bytes.len(), expected.theorem_type_jcs_utf8_length);
            assert_eq!(sha256(&theorem_bytes), expected.theorem_type_jcs_sha256);
        }
    }
}

#[test]
fn rejects_every_skeleton_mutation_at_the_owned_phase() {
    let vector = load_vector();
    validate_vector_header(&vector);
    assert_eq!(vector.mutation_cases.len(), 19);
    let (contexts, fixtures) = load_vc_sources(&vector);

    for case in &vector.mutation_cases {
        let input_sources =
            usize::from(case.construction.is_some()) + usize::from(case.json_text.is_some());
        assert_eq!(input_sources, 1, "{}", case.id);
        let (bytes, vc_bytes, context) = if let Some(text) = &case.json_text {
            let fixture = &fixtures["vc.rust_identity"];
            (
                text.as_bytes().to_vec(),
                canonical_value(&fixture.input),
                &contexts[fixture.context_id.as_str()],
            )
        } else {
            let construction = case.construction.as_ref().expect("construction");
            let fixture = &fixtures[construction.emit_from.as_str()];
            let context = &contexts[fixture.context_id.as_str()];
            let vc_bytes = canonical_value(&fixture.input);
            let emitted = emit_vc_skeleton_v1(&vc_bytes, context).expect("base skeleton emits");
            let mut value: Value =
                serde_json::from_slice(emitted.canonical_bytes()).expect("skeleton value");
            for mutation in &construction.mutations {
                apply_mutation(&mut value, mutation);
            }
            let bytes = match construction.transport_encoding.as_deref() {
                None => canonical_value(&value),
                Some("two_space_indent_no_final_lf") => serde_json::to_string_pretty(&value)
                    .expect("pretty JSON")
                    .into_bytes(),
                Some(encoding) => panic!("unknown transport encoding {encoding:?}"),
            };
            (bytes, vc_bytes, context)
        };

        assert_eq!(case.expect.outcome, Outcome::Reject, "{}", case.id);
        let error = import_vc_skeleton_v1_json(&bytes, &vc_bytes, context).unwrap_err();
        assert_eq!(
            error.phase().as_str(),
            case.expect.phase.as_deref().expect("reject phase"),
            "{}: {error}",
            case.id
        );
        assert_eq!(
            error.code(),
            case.expect.code.as_deref().expect("reject code"),
            "{}: {error}",
            case.id
        );
    }
}

#[test]
fn executes_every_skeleton_limit_vector() {
    let vector = load_vector();
    assert_eq!(vector.limit_cases.len(), 2);
    for case in &vector.limit_cases {
        assert_eq!(
            case.below.construction.count + 1,
            case.maximum,
            "{}",
            case.id
        );
        assert_eq!(case.at.construction.count, case.maximum, "{}", case.id);
        assert_eq!(
            case.above.construction.count,
            case.maximum + 1,
            "{}",
            case.id
        );
        for point in [&case.below, &case.at] {
            assert!(!point.construction.fixture.is_empty());
            validate_verification_limit(&case.limit_id, point.construction.count)
                .unwrap_or_else(|error| panic!("{} should accept: {error}", case.id));
            assert_eq!(point.expect.outcome, Outcome::Accept);
            assert!(point.expect.phase.is_none());
            assert!(point.expect.code.is_none());
        }
        let error = validate_verification_limit(&case.limit_id, case.above.construction.count)
            .expect_err("above maximum rejects");
        assert_eq!(case.above.expect.outcome, Outcome::Reject);
        assert_eq!(error.code(), case.above.expect.code.as_deref().unwrap());
    }
}

#[test]
fn policy_member_binding_names_the_checked_containing_declaration() {
    let vector = load_vector();
    let (contexts, fixtures) = load_vc_sources(&vector);
    let fixture = &fixtures["vc.rust_call_pair"];
    let context = &contexts[fixture.context_id.as_str()];
    let vc_bytes = canonical_value(&fixture.input);
    let skeleton = emit_vc_skeleton_v1(&vc_bytes, context).expect("skeleton emits");
    let declaration = &skeleton.skeleton().theorem_declarations[3];
    let member = &declaration.member_ids[0];

    validate_policy_member_binding(&skeleton, member, &declaration.group_id, &declaration.name)
        .expect("exact containment tuple accepts");
    assert!(matches!(
        validate_policy_member_binding(&skeleton, member, "wrong.group", &declaration.name),
        Err(PolicyMemberBindingError::WrongGroup { .. })
    ));
    assert!(matches!(
        validate_policy_member_binding(&skeleton, member, &declaration.group_id, "Wrong.Name"),
        Err(PolicyMemberBindingError::WrongDeclaration { .. })
    ));
    assert!(matches!(
        validate_policy_member_binding(
            &skeleton,
            "missing#member",
            &declaration.group_id,
            &declaration.name
        ),
        Err(PolicyMemberBindingError::MissingMember(_))
    ));
}

fn validate_vector_header(vector: &Vector) {
    assert_eq!(vector.schema, VECTOR_SCHEMA);
    assert_eq!(vector.spec_schema, SPEC_SCHEMA);
    assert_eq!(vector.source_vector, "develop/specs/vectors/vc-v1.json");
    assert_eq!(vector.owner_test, "crates/mpk-vc/tests/vc_skeleton_v1.rs");
    assert_eq!(vector.fixture_digest_semantics.algorithm, "sha256");
    assert_eq!(
        vector.fixture_digest_semantics.payload,
        "jcs_without_domain_separator"
    );
    assert!(!vector.fixture_digest_semantics.artifact_identity);
    let operations = vector
        .construction_operations
        .iter()
        .map(|operation| {
            assert!(!operation.rule.is_empty());
            operation.op.as_str()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        operations,
        BTreeSet::from([
            "emit",
            "add",
            "remove",
            "replace",
            "copy",
            "swap",
            "reverse",
            "right_associate_group_members",
        ])
    );
}

fn load_vector() -> Vector {
    let bytes = fs::read(skeleton_vector_path()).expect("read skeleton vector");
    serde_json::from_slice(&bytes).expect("closed skeleton vector")
}

fn load_vc_sources(
    vector: &Vector,
) -> (
    BTreeMap<String, VcSourceContext>,
    BTreeMap<String, VcFixture>,
) {
    let path = repo_root().join(&vector.source_vector);
    let value: Value =
        serde_json::from_slice(&fs::read(path).expect("read VC vector")).expect("parse VC vector");
    let contexts = value["source_contexts"]
        .as_array()
        .expect("source contexts")
        .iter()
        .map(|value| {
            let context: VcSourceContext = serde_json::from_value(value.clone()).expect("context");
            let id = context.id.clone();
            (id, context)
        })
        .collect::<BTreeMap<_, _>>();
    let fixtures = value["fixtures"]
        .as_array()
        .expect("VC fixtures")
        .iter()
        .map(|value| {
            let id = value["id"].as_str().expect("fixture id").to_owned();
            (
                id,
                VcFixture {
                    context_id: value["source_context"]
                        .as_str()
                        .expect("context id")
                        .to_owned(),
                    input: value["input"].clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    (contexts, fixtures)
}

fn skeleton_vector_path() -> PathBuf {
    repo_root().join("develop/specs/vectors/vc-skeleton-v1.json")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical_value(value: &Value) -> Vec<u8> {
    canonical_serializable(value)
}

fn canonical_serializable<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let bytes = serde_json::to_vec(value).expect("serialize value");
    let strict = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576),
    )
    .expect("strict JSON");
    canonical_json_bytes(&strict).expect("canonical JSON")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn apply_mutation(root: &mut Value, mutation: &Mutation) {
    match mutation {
        Mutation::Add { path, value } => insert_value(root, path, value.clone()),
        Mutation::Remove { path } => {
            remove_value(root, path);
        }
        Mutation::Replace { path, value } => {
            *root
                .pointer_mut(path)
                .unwrap_or_else(|| panic!("replace {path}")) = value.clone();
        }
        Mutation::Copy { from, path } => {
            let value = root
                .pointer(from)
                .unwrap_or_else(|| panic!("copy {from}"))
                .clone();
            insert_value(root, path, value);
        }
        Mutation::Swap {
            path,
            first,
            second,
        } => {
            let array = root
                .pointer_mut(path)
                .and_then(Value::as_array_mut)
                .expect("swap array");
            array.swap(
                usize::try_from(*first).unwrap(),
                usize::try_from(*second).unwrap(),
            );
        }
        Mutation::Reverse { path } => root
            .pointer_mut(path)
            .and_then(Value::as_array_mut)
            .expect("reverse array")
            .reverse(),
        Mutation::RightAssociateGroupMembers { declaration_index } => {
            right_associate_group_members(root, usize::try_from(*declaration_index).unwrap());
        }
    }
}

fn right_associate_group_members(root: &mut Value, declaration_index: usize) {
    let path = format!("/theorem_declarations/{declaration_index}/theorem_type");
    let value = root.pointer_mut(&path).expect("theorem type");
    let mut theorem: GroupedTheoremType = serde_json::from_value(value.clone()).expect("theorem");
    let VcTerm::Apply { function, args } = &mut theorem.body else {
        panic!("group body is implication")
    };
    assert_eq!(function, "Std.Logic.Imp");
    assert_eq!(args.len(), 2);
    let mut members = Vec::new();
    flatten_conjunction(&args[1], &mut members);
    assert!(members.len() >= 3);
    args[1] = right_fold_conjunction(&members);
    *value = serde_json::to_value(theorem).expect("theorem value");
}

fn flatten_conjunction(term: &VcTerm, output: &mut Vec<VcTerm>) {
    match term {
        VcTerm::Apply { function, args } if function == "Std.Bool.and" && args.len() == 2 => {
            flatten_conjunction(&args[0], output);
            flatten_conjunction(&args[1], output);
        }
        term => output.push(term.clone()),
    }
}

fn right_fold_conjunction(terms: &[VcTerm]) -> VcTerm {
    let (first, rest) = terms.split_first().expect("nonempty members");
    if rest.is_empty() {
        first.clone()
    } else {
        VcTerm::Apply {
            function: "Std.Bool.and".to_owned(),
            args: vec![first.clone(), right_fold_conjunction(rest)],
        }
    }
}

fn insert_value(root: &mut Value, path: &str, value: Value) {
    let (parent_path, key) = split_pointer(path);
    let parent = root.pointer_mut(&parent_path).expect("add parent");
    match parent {
        Value::Object(object) => assert!(object.insert(key, value).is_none()),
        Value::Array(array) => array.insert(key.parse().expect("array index"), value),
        _ => panic!("add parent is not a container"),
    }
}

fn remove_value(root: &mut Value, path: &str) -> Value {
    let (parent_path, key) = split_pointer(path);
    let parent = root.pointer_mut(&parent_path).expect("remove parent");
    match parent {
        Value::Object(object) => object.remove(&key).expect("object key"),
        Value::Array(array) => array.remove(key.parse().expect("array index")),
        _ => panic!("remove parent is not a container"),
    }
}

fn split_pointer(path: &str) -> (String, String) {
    let (parent, encoded) = path.rsplit_once('/').expect("JSON pointer");
    (
        parent.to_owned(),
        encoded.replace("~1", "/").replace("~0", "~"),
    )
}
