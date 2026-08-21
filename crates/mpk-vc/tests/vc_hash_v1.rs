use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use mpk_vc::{
    canonical_vc_hash_payload, canonical_vc_json, hash_domain_separated_raw, parse_strict_json,
    vc_hash, HashDomain, StrictJsonLimits, StrictJsonValue, VcDocument, VC_HASH_DOMAIN,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HashVector {
    schema: String,
    spec_schema: String,
    source_vector: String,
    owner_test: String,
    domain: Domain,
    canonical_cases: Vec<CanonicalCase>,
    equivalence_cases: Vec<EquivalenceCase>,
    ordered_array_cases: Vec<MutationCase>,
    mutation_cases: Vec<MutationCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Domain {
    text: String,
    utf8_hex: String,
    separator_hex: String,
    digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalCase {
    id: String,
    source_fixture: String,
    excluded_fields: Vec<String>,
    expected_payload_jcs_utf8_length: usize,
    expected_complete_jcs_utf8_length: usize,
    expected_preimage_length: usize,
    expected_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EquivalenceCase {
    id: String,
    source_fixture: String,
    encodings: Option<Vec<String>>,
    patches: Option<Vec<Patch>>,
    expected_payload_jcs_utf8_length: Option<usize>,
    expected_sha256: String,
    without_separator_preimage_length: Option<usize>,
    without_separator_sha256: Option<String>,
    wrong_domain_text: Option<String>,
    wrong_domain_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationCase {
    id: String,
    source_fixture: String,
    patches: Vec<Patch>,
    expected_payload_jcs_utf8_length: usize,
    expected_sha256: String,
    different_from: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Patch {
    Replace { path: String, value: Value },
    Reverse { path: String },
}

#[test]
fn vc_hash_v1_executes_every_vector_case() {
    let vector = load_hash_vector();
    assert_eq!(vector.schema, "mpk.vc.hash_vectors.v1");
    assert_eq!(vector.spec_schema, "mpk.vc.v1");
    assert_eq!(vector.source_vector, "develop/specs/vectors/vc-v1.json");
    assert_eq!(vector.owner_test, "crates/mpk-vc/tests/vc_hash_v1.rs");
    assert_eq!(vector.domain.text, VC_HASH_DOMAIN.as_str());
    assert_eq!(vector.domain.utf8_hex, "4d504b2d56432d312e30");
    assert_eq!(vector.domain.separator_hex, "00");
    assert_eq!(vector.domain.digest, "sha256");
    assert_eq!(vector.canonical_cases.len(), 3);
    assert_eq!(vector.equivalence_cases.len(), 3);
    assert_eq!(vector.ordered_array_cases.len(), 1);
    assert_eq!(vector.mutation_cases.len(), 9);

    let fixtures = load_source_fixtures();
    for case in &vector.canonical_cases {
        let document = document(&fixtures[case.source_fixture.as_str()]);
        let payload = canonical_vc_hash_payload(&document).expect("canonical payload");
        let complete = canonical_vc_json(&document).expect("complete canonical VC");
        let hash = vc_hash(&document).expect("VC hash");
        assert_eq!(case.excluded_fields, ["vc_hash"], "{}", case.id);
        assert_eq!(
            payload.len(),
            case.expected_payload_jcs_utf8_length,
            "{}",
            case.id
        );
        assert_eq!(
            complete.len(),
            case.expected_complete_jcs_utf8_length,
            "{}",
            case.id
        );
        assert_eq!(
            VC_HASH_DOMAIN.as_str().len() + 1 + payload.len(),
            case.expected_preimage_length,
            "{}",
            case.id
        );
        assert_eq!(hash.as_str(), case.expected_sha256, "{}", case.id);
        assert_eq!(document.vc_hash, case.expected_sha256, "{}", case.id);
    }

    for case in &vector.equivalence_cases {
        let fixture = &fixtures[case.source_fixture.as_str()];
        match case.id.as_str() {
            "canonical.object_key_order_and_whitespace" => {
                let encodings = case.encodings.as_ref().expect("encodings");
                assert_eq!(
                    encodings.iter().map(String::as_str).collect::<Vec<_>>(),
                    [
                        "fixture_jcs",
                        "reverse_each_object_key_order_with_two_space_indent"
                    ]
                );
                let canonical = serde_json::to_vec(fixture).expect("fixture JSON");
                let strict = parse(&canonical);
                let reversed = reverse_pretty(&strict);
                let first: VcDocument = serde_json::from_slice(&canonical).expect("canonical doc");
                let second: VcDocument =
                    serde_json::from_slice(reversed.as_bytes()).expect("reordered pretty doc");
                assert_ne!(canonical, reversed.as_bytes());
                assert_eq!(first, second);
                assert_hash_case(&first, case);
            }
            "canonical.root_self_hash_excluded" => {
                let mut value = fixture.clone();
                apply_patches(&mut value, case.patches.as_ref().expect("patches"));
                let mutated = document(&value);
                assert_hash_case(&mutated, case);
            }
            "canonical.domain_separator_required" => {
                let document = document(fixture);
                let payload = canonical_vc_hash_payload(&document).expect("payload");
                assert_eq!(vc_hash(&document).unwrap().as_str(), case.expected_sha256);

                let mut hasher = Sha256::new();
                hasher.update(VC_HASH_DOMAIN.as_str().as_bytes());
                hasher.update(&payload);
                let without = hex(&hasher.finalize());
                assert_eq!(
                    VC_HASH_DOMAIN.as_str().len() + payload.len(),
                    case.without_separator_preimage_length.expect("length")
                );
                assert_eq!(without, case.without_separator_sha256.as_deref().unwrap());
                assert_eq!(case.wrong_domain_text.as_deref(), Some("MPK-VC-1.1"));
                let wrong = hash_domain_separated_raw(HashDomain::new("MPK-VC-1.1"), &payload)
                    .expect("wrong domain still hashes");
                assert_eq!(wrong.to_hex(), case.wrong_domain_sha256.as_deref().unwrap());
            }
            other => panic!("unexecuted equivalence case {other}"),
        }
    }

    for case in vector
        .ordered_array_cases
        .iter()
        .chain(&vector.mutation_cases)
    {
        let mut value = fixtures[case.source_fixture.as_str()].clone();
        apply_patches(&mut value, &case.patches);
        let mutated = document(&value);
        let payload = canonical_vc_hash_payload(&mutated).expect("mutated payload");
        let hash = vc_hash(&mutated).expect("mutated hash");
        assert_eq!(
            payload.len(),
            case.expected_payload_jcs_utf8_length,
            "{}",
            case.id
        );
        assert_eq!(hash.as_str(), case.expected_sha256, "{}", case.id);
        if let Some(different) = &case.different_from {
            assert_ne!(hash.as_str(), different, "{}", case.id);
        }
    }
}

fn assert_hash_case(document: &VcDocument, case: &EquivalenceCase) {
    let payload = canonical_vc_hash_payload(document).expect("payload");
    assert_eq!(
        payload.len(),
        case.expected_payload_jcs_utf8_length
            .expect("payload length"),
        "{}",
        case.id
    );
    assert_eq!(
        vc_hash(document).expect("hash").as_str(),
        case.expected_sha256,
        "{}",
        case.id
    );
}

fn load_hash_vector() -> HashVector {
    let path = root().join("develop/specs/vectors/vc-hash-v1.json");
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("closed hash vector")
}

fn load_source_fixtures() -> BTreeMap<String, Value> {
    let path = root().join("develop/specs/vectors/vc-v1.json");
    let value: Value = serde_json::from_slice(&fs::read(path).expect("source vector"))
        .expect("source vector JSON");
    let expected = [
        "dependencies",
        "fixtures",
        "limit_cases",
        "owner_tests",
        "schema",
        "source_contexts",
        "spec_schema",
        "vc_cases",
    ];
    let object = value.as_object().expect("source vector object");
    assert_eq!(object.len(), expected.len());
    assert!(expected.iter().all(|name| object.contains_key(*name)));
    value["fixtures"]
        .as_array()
        .expect("fixtures")
        .iter()
        .map(|fixture| {
            (
                fixture["id"].as_str().expect("fixture id").to_owned(),
                fixture["input"].clone(),
            )
        })
        .collect()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn document(value: &Value) -> VcDocument {
    serde_json::from_value(value.clone()).expect("VC document shape")
}

fn apply_patches(value: &mut Value, patches: &[Patch]) {
    for patch in patches {
        match patch {
            Patch::Replace { path, value: next } => {
                *value
                    .pointer_mut(path)
                    .unwrap_or_else(|| panic!("replace path {path:?}")) = next.clone();
            }
            Patch::Reverse { path } => value
                .pointer_mut(path)
                .and_then(Value::as_array_mut)
                .unwrap_or_else(|| panic!("reverse path {path:?}"))
                .reverse(),
        }
    }
}

fn parse(bytes: &[u8]) -> StrictJsonValue {
    parse_strict_json(
        bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576),
    )
    .expect("strict JSON")
}

fn reverse_pretty(value: &StrictJsonValue) -> String {
    let mut output = String::new();
    write_pretty_reversed(value, 0, &mut output);
    output
}

fn write_pretty_reversed(value: &StrictJsonValue, indent: usize, output: &mut String) {
    match value {
        StrictJsonValue::Null => output.push_str("null"),
        StrictJsonValue::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        StrictJsonValue::Integer(value) => output.push_str(&value.to_string()),
        StrictJsonValue::String(value) => {
            output.push_str(&serde_json::to_string(value).expect("quoted string"));
        }
        StrictJsonValue::Array(values) => {
            output.push('[');
            if !values.is_empty() {
                output.push('\n');
                for (index, value) in values.iter().enumerate() {
                    output.push_str(&" ".repeat(indent + 2));
                    write_pretty_reversed(value, indent + 2, output);
                    if index + 1 != values.len() {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&" ".repeat(indent));
            }
            output.push(']');
        }
        StrictJsonValue::Object(entries) => {
            output.push('{');
            if !entries.is_empty() {
                output.push('\n');
                for (index, (name, value)) in entries.iter().rev().enumerate() {
                    output.push_str(&" ".repeat(indent + 2));
                    output.push_str(&serde_json::to_string(name).expect("quoted key"));
                    output.push_str(": ");
                    write_pretty_reversed(value, indent + 2, output);
                    if index + 1 != entries.len() {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&" ".repeat(indent));
            }
            output.push('}');
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(*byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(*byte & 0x0f)]));
    }
    output
}
