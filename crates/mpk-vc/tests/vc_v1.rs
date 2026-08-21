use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use mpk_vc::{
    canonical_json_bytes, generate_vc_v1_from_context, import_vc_v1_json, parse_strict_json,
    validate_verification_limit, StrictJsonLimits, VcSourceContext, VcTerm,
};
use serde::Deserialize;
use serde_json::Value;

const VECTOR_SCHEMA: &str = "mpk.vc.conformance.v1";
const SPEC_SCHEMA: &str = "mpk.vc.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    schema: String,
    spec_schema: String,
    dependencies: Value,
    owner_tests: Vec<String>,
    source_contexts: Vec<VcSourceContext>,
    fixtures: Vec<Fixture>,
    vc_cases: Vec<VcCase>,
    limit_cases: Vec<LimitCase>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    source_context: String,
    input: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcCase {
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
        first: u64,
        second: u64,
    },
}

#[derive(Clone, Debug, Deserialize)]
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

#[test]
fn vc_v1_executes_every_conformance_case() {
    let vector = load_vector();
    assert_eq!(vector.schema, VECTOR_SCHEMA);
    assert_eq!(vector.spec_schema, SPEC_SCHEMA);
    assert_eq!(
        vector.owner_tests,
        ["crates/mpk-vc/tests/vc_v1.rs"],
        "ownership must remain exclusive"
    );
    assert!(vector.dependencies.is_object());
    assert_eq!(vector.vc_cases.len(), 28);

    let contexts = vector
        .source_contexts
        .iter()
        .map(|context| (context.id.as_str(), context))
        .collect::<BTreeMap<_, _>>();
    let fixtures = vector
        .fixtures
        .iter()
        .map(|fixture| (fixture.id.as_str(), fixture))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(contexts.len(), vector.source_contexts.len());
    assert_eq!(fixtures.len(), vector.fixtures.len());

    for fixture in &vector.fixtures {
        let context = contexts[fixture.source_context.as_str()];
        let expected = canonical_value(&fixture.input);
        let first = generate_vc_v1_from_context(context).expect("fixture context generates");
        let second = generate_vc_v1_from_context(context).expect("repeat generation succeeds");
        assert_eq!(first.canonical_bytes(), expected, "fixture {}", fixture.id);
        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(first.document().vc_hash, first.hash().as_str());
    }

    let default_context = &vector.source_contexts[0];
    let mut constructed = BTreeMap::<String, (Value, String)>::new();
    for case in &vector.vc_cases {
        let source_count = usize::from(case.input_from.is_some())
            + usize::from(case.transport_from.is_some())
            + usize::from(case.json_text.is_some())
            + usize::from(case.construction.is_some());
        assert_eq!(source_count, 1, "case {} has one input source", case.id);

        let (bytes, context, stored) = if let Some(id) = &case.input_from {
            let fixture = fixtures[id.as_str()];
            (
                canonical_value(&fixture.input),
                contexts[fixture.source_context.as_str()],
                Some((fixture.input.clone(), fixture.source_context.clone())),
            )
        } else if let Some(transport) = &case.transport_from {
            let fixture = fixtures[transport.fixture.as_str()];
            assert_eq!(transport.encoding, "two_space_indent_no_final_lf");
            let pretty = serde_json::to_string_pretty(&fixture.input).expect("pretty JSON");
            (
                pretty.into_bytes(),
                contexts[fixture.source_context.as_str()],
                Some((fixture.input.clone(), fixture.source_context.clone())),
            )
        } else if let Some(text) = &case.json_text {
            (text.as_bytes().to_vec(), default_context, None)
        } else {
            let construction = case.construction.as_ref().expect("construction exists");
            let (mut value, context_id) =
                if let Some(fixture) = fixtures.get(construction.base.as_str()) {
                    (fixture.input.clone(), fixture.source_context.clone())
                } else {
                    constructed[construction.base.as_str()].clone()
                };
            for patch in &construction.patches {
                apply_patch(&mut value, patch);
            }
            (
                canonical_value(&value),
                contexts[context_id.as_str()],
                Some((value, context_id)),
            )
        };

        let result = import_vc_v1_json(&bytes, context);
        match case.expect.outcome {
            Outcome::Accept => {
                let validated = result.unwrap_or_else(|error| {
                    panic!("case {} unexpectedly rejected: {error}", case.id)
                });
                assert_eq!(validated.canonical_bytes(), bytes);
                assert!(case.expect.phase.is_none());
                assert!(case.expect.code.is_none());
            }
            Outcome::Reject => {
                let error = result.unwrap_err();
                assert_eq!(
                    error.phase().as_str(),
                    case.expect.phase.as_deref().expect("reject phase"),
                    "case {}",
                    case.id
                );
                assert_eq!(
                    error.code(),
                    case.expect.code.as_deref().expect("reject code"),
                    "case {}: {error}",
                    case.id
                );
            }
        }
        if let Some(stored) = stored {
            assert!(constructed.insert(case.id.clone(), stored).is_none());
        }
    }
}

#[test]
fn verification_limits_execute_every_below_at_above_case() {
    let vector = load_vector();
    assert_eq!(vector.limit_cases.len(), 9);
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
        assert!(!case.below.construction.fixture.is_empty());
        assert!(!case.at.construction.fixture.is_empty());
        assert!(!case.above.construction.fixture.is_empty());

        for point in [&case.below, &case.at] {
            validate_verification_limit(&case.limit_id, point.construction.count)
                .unwrap_or_else(|error| panic!("{} should accept: {error}", case.id));
            assert_eq!(point.expect.outcome, Outcome::Accept);
            assert!(point.expect.code.is_none());
        }
        let error = validate_verification_limit(&case.limit_id, case.above.construction.count)
            .expect_err("above maximum rejects");
        assert_eq!(case.above.expect.outcome, Outcome::Reject);
        assert!(
            case.above.expect.phase.is_none(),
            "standalone limit-vector outcomes intentionally carry no VC validation phase"
        );
        assert_eq!(
            error.code(),
            case.above.expect.code.as_deref().expect("above code"),
            "{}",
            case.id
        );
    }
}

#[test]
fn generation_reports_the_earliest_limit_without_returning_a_partial_document() {
    let mut context = load_vector().source_contexts.remove(0);
    context.functions[0].regenerated_members[0].assumptions = (0..4_097)
        .map(|_| VcTerm::Constant {
            name: "Std.Bool.true".to_owned(),
        })
        .collect();

    let error = generate_vc_v1_from_context(&context).expect_err("limit rejects generation");

    assert_eq!(error.phase().as_str(), "stream_limits");
    assert_eq!(error.code(), "VC_LIMIT_ASSUMPTIONS_PER_MEMBER");
}

#[test]
fn member_expression_depth_accepts_the_real_serialized_boundary() {
    let mut context = load_vector().source_contexts.remove(0);
    context.functions[0].regenerated_members[0].conclusion = nested_not(256);

    let validated =
        generate_vc_v1_from_context(&context).expect("stored term depth 256 accepts end to end");
    let imported = import_vc_v1_json(validated.canonical_bytes(), &context)
        .expect("depth-256 canonical transport reimports");
    assert_eq!(imported.canonical_bytes(), validated.canonical_bytes());

    context.functions[0].regenerated_members[0].conclusion = nested_not(257);
    let error = generate_vc_v1_from_context(&context).expect_err("stored term depth 257 rejects");
    assert_eq!(error.phase().as_str(), "stream_limits");
    assert_eq!(error.code(), "VC_LIMIT_MEMBER_EXPRESSION_DEPTH");
}

fn load_vector() -> Vector {
    let path = vector_path();
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("closed VC conformance vector")
}

fn vector_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/specs/vectors/vc-v1.json")
}

fn canonical_value(value: &Value) -> Vec<u8> {
    let bytes = serde_json::to_vec(value).expect("serialize vector value");
    let strict = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576),
    )
    .expect("strict vector JSON");
    canonical_json_bytes(&strict).expect("canonical vector JSON")
}

fn apply_patch(root: &mut Value, patch: &Patch) {
    match patch {
        Patch::Add { path, value } => insert_value(root, path, value.clone()),
        Patch::Remove { path } => {
            remove_value(root, path);
        }
        Patch::Replace { path, value } => {
            let target = root
                .pointer_mut(path)
                .unwrap_or_else(|| panic!("replace path {path:?}"));
            *target = value.clone();
        }
        Patch::Copy { from, path } => {
            let value = root
                .pointer(from)
                .unwrap_or_else(|| panic!("copy source {from:?}"))
                .clone();
            insert_value(root, path, value);
        }
        Patch::Swap {
            path,
            first,
            second,
        } => {
            assert_ne!(first, second);
            let array = root
                .pointer_mut(path)
                .and_then(Value::as_array_mut)
                .unwrap_or_else(|| panic!("swap array {path:?}"));
            let first = usize::try_from(*first).expect("first index");
            let second = usize::try_from(*second).expect("second index");
            assert!(first < array.len() && second < array.len());
            array.swap(first, second);
        }
    }
}

fn insert_value(root: &mut Value, path: &str, value: Value) {
    let (parent_path, key) = split_pointer(path);
    let parent = root
        .pointer_mut(&parent_path)
        .unwrap_or_else(|| panic!("add parent {parent_path:?}"));
    match parent {
        Value::Object(object) => {
            assert!(object.insert(key, value).is_none(), "add must not replace");
        }
        Value::Array(array) => {
            let index = key.parse::<usize>().expect("array insertion index");
            assert!(index <= array.len());
            array.insert(index, value);
        }
        _ => panic!("add parent is not a container"),
    }
}

fn remove_value(root: &mut Value, path: &str) -> Value {
    let (parent_path, key) = split_pointer(path);
    let parent = root
        .pointer_mut(&parent_path)
        .unwrap_or_else(|| panic!("remove parent {parent_path:?}"));
    match parent {
        Value::Object(object) => object
            .remove(&key)
            .unwrap_or_else(|| panic!("remove object key {key:?}")),
        Value::Array(array) => {
            let index = key.parse::<usize>().expect("array removal index");
            assert!(index < array.len());
            array.remove(index)
        }
        _ => panic!("remove parent is not a container"),
    }
}

fn split_pointer(path: &str) -> (String, String) {
    let (parent, encoded) = path
        .rsplit_once('/')
        .unwrap_or_else(|| panic!("invalid JSON pointer {path:?}"));
    let key = encoded.replace("~1", "/").replace("~0", "~");
    (parent.to_owned(), key)
}

fn nested_not(depth: usize) -> VcTerm {
    assert!(depth >= 1);
    let mut term = VcTerm::Constant {
        name: "Std.Bool.true".to_owned(),
    };
    for _ in 1..depth {
        term = VcTerm::Apply {
            function: "Std.Bool.not".to_owned(),
            args: vec![term],
        };
    }
    term
}
