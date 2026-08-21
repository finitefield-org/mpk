use mpk_vc::vir::VirIntegerLiteral;
use mpk_vc::{
    canonical_json_bytes, import_source_map_json, import_vir_json, parse_strict_json,
    source_map_hash, validate_normalized_path, validate_source_map_canonical_size,
    validate_source_map_entry_count, CapturedInput, InputKind, LowercaseSha256, SourceInputKind,
    SourceMap, SourceMapEntry, SourceMapError, SourceMapValidationContext, SourceOrigin,
    SourceReference, StrictJsonLimits, SyntheticPermission, VirInstruction, VirIntLiteral,
    VirLiteral, VirModule, VirType,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

const ALL_JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);

struct SyntheticFixture {
    vir: VirModule,
    map: SourceMap,
    captures: Vec<(InputKind, String, Vec<u8>)>,
    permissions: Vec<SyntheticPermission>,
}

#[test]
fn source_map_vectors_are_exhaustive_and_match_normative_outcomes() {
    let vectors = load_json("develop/specs/vectors/source-map-v0.json");
    assert_exact_keys(
        &vectors,
        &[
            "schema",
            "spec_schema",
            "dependencies",
            "owner_tests",
            "fixture_sources",
            "map_cases",
            "reference_cases",
            "mapping_cases",
            "path_cases",
            "hash_cases",
            "limit_cases",
        ],
    );
    assert_eq!(vectors["schema"], "mpk.source_map.conformance.v0");
    assert_eq!(vectors["spec_schema"], "mpk.source_map.v0");
    assert_exact_keys(&vectors["dependencies"], &["vir_vector", "vir_case"]);
    assert_eq!(
        vectors["owner_tests"],
        json!([
            "crates/mpk-vc/tests/source_map.rs",
            "go-tools/go2vir/corpus_test.go",
            "rust-tools/rust2vir/tests/frontend_envelope.rs"
        ])
    );

    let vir = valid_vir();
    let fixtures = fixture_sources(&vectors);
    let base = vectors["map_cases"][0]["input"].clone();
    let mut all_ids = BTreeSet::new();
    let mut visited = BTreeSet::new();

    for group in [
        "map_cases",
        "reference_cases",
        "mapping_cases",
        "path_cases",
        "hash_cases",
        "limit_cases",
    ] {
        for case in vectors[group].as_array().expect("case array") {
            assert_case_shape(group, case);
            let id = case["id"].as_str().expect("case id");
            assert!(all_ids.insert(id.to_owned()), "duplicate case ID {id}");
            visited.insert(id.to_owned());
            match group {
                "hash_cases" => run_hash_case(case, &base),
                "limit_cases" => run_limit_case(case),
                _ => run_map_case(case, &base, &vir, &fixtures),
            }
        }
    }
    assert_eq!(visited, all_ids, "a normative source-map case was skipped");
}

fn run_map_case(
    case: &Value,
    base: &Value,
    default_vir: &VirModule,
    fixtures: &BTreeMap<String, (InputKind, Vec<u8>)>,
) {
    let id = case["id"].as_str().unwrap();
    if id == "mapping.accept_profile_synthetic_instruction" {
        let fixture = synthetic_fixture(default_vir, fixtures);
        let refs: Vec<_> = fixture
            .captures
            .iter()
            .map(|(kind, path, bytes)| CapturedInput {
                kind: *kind,
                normalized_path: path,
                bytes,
            })
            .collect();
        import_source_map_json(
            &serde_json::to_vec(&fixture.map).unwrap(),
            SourceMapValidationContext {
                vir: &fixture.vir,
                captured_inputs: &refs,
                synthetic_permissions: &fixture.permissions,
            },
        )
        .expect("synthetic profile fixture must validate");
        return;
    }

    let bytes = if let Some(text) = case.get("json_text").and_then(Value::as_str) {
        text.as_bytes().to_vec()
    } else {
        let value = if let Some(input) = case.get("input") {
            input.clone()
        } else {
            apply_construction(base.clone(), &case["construction"])
        };
        serde_json::to_vec(&value).unwrap()
    };

    let mut source_ids = vec!["source.identity_go"];
    if let Some(ids) = case
        .pointer("/construction/context/source_cases")
        .and_then(Value::as_array)
    {
        source_ids = ids.iter().map(|id| id.as_str().unwrap()).collect();
    }
    let mut captures: Vec<(InputKind, String, Vec<u8>)> = source_ids
        .iter()
        .map(|id| {
            let (kind, bytes) = &fixtures[*id];
            let path = if *id == "source.unicode_go" {
                "unicode.go"
            } else {
                "identity.go"
            };
            (*kind, path.to_owned(), bytes.clone())
        })
        .collect();
    if case
        .pointer("/construction/context/additional_manifest_inputs")
        .is_some()
    {
        captures.push((
            InputKind::Contract,
            "contracts/identity.json".to_owned(),
            Vec::new(),
        ));
    }
    let refs: Vec<_> = captures
        .iter()
        .map(|(kind, path, bytes)| CapturedInput {
            kind: *kind,
            normalized_path: path,
            bytes,
        })
        .collect();
    let result = import_source_map_json(
        &bytes,
        SourceMapValidationContext {
            vir: default_vir,
            captured_inputs: &refs,
            synthetic_permissions: &[],
        },
    );
    if let (Ok(validated), Some(expected_length)) = (
        result.as_ref(),
        case.pointer("/expect/canonical_jcs_utf8_length")
            .and_then(Value::as_u64),
    ) {
        assert_eq!(
            validated.canonical_bytes().len() as u64,
            expected_length,
            "{id}"
        );
    }
    assert_expected(case, result.map(|_| ()), id);
}

fn run_hash_case(case: &Value, base: &Value) {
    let id = case["id"].as_str().unwrap();
    if id == "hash.reject_wrong_hash" {
        let value = apply_construction(base.clone(), &case["construction"]);
        let vir = valid_vir();
        let source = decode_base64(
            "cGFja2FnZSB2ZWN0b3IKCmZ1bmMgSWRlbnRpdHkodmFsdWUgaW50OCkgaW50OCB7IHJldHVybiB2YWx1ZSB9Cg==",
        );
        let captures = [CapturedInput {
            kind: InputKind::Source,
            normalized_path: "identity.go",
            bytes: &source,
        }];
        let result = import_source_map_json(
            &serde_json::to_vec(&value).unwrap(),
            SourceMapValidationContext {
                vir: &vir,
                captured_inputs: &captures,
                synthetic_permissions: &[],
            },
        );
        assert_expected(case, result.map(|_| ()), id);
        return;
    }

    let map: SourceMap = serde_json::from_value(base.clone()).unwrap();
    let expected = case
        .get("expected_sha256")
        .and_then(Value::as_str)
        .unwrap_or(map.source_map_hash.as_str());
    if id == "hash.valid_go_identity" {
        assert_eq!(source_map_hash(&map).unwrap().as_str(), expected);
        let strict =
            parse_strict_json(&serde_json::to_vec(&map).unwrap(), ALL_JSON_LIMITS).unwrap();
        let preimage = strict.clone_without_fields(&["source_map_hash"]).unwrap();
        assert_eq!(
            canonical_json_bytes(&preimage).unwrap().len() as u64,
            case["canonical_without_hash_utf8_length"].as_u64().unwrap()
        );
    } else {
        let domain = case
            .get("wrong_domain_utf8")
            .and_then(Value::as_str)
            .unwrap_or("MPK-SOURCE-MAP-0.1");
        let separator = case
            .get("wrong_separator_hex")
            .and_then(Value::as_str)
            .unwrap_or("00");
        let mut preimage = domain.as_bytes().to_vec();
        if separator == "00" {
            preimage.push(0);
        }
        let strict =
            parse_strict_json(&serde_json::to_vec(&map).unwrap(), ALL_JSON_LIMITS).unwrap();
        let without_hash = strict.clone_without_fields(&["source_map_hash"]).unwrap();
        preimage.extend(canonical_json_bytes(&without_hash).unwrap());
        let actual = sha256_hex(&preimage);
        assert_eq!(
            actual,
            case["wrong_domain_sha256"]
                .as_str()
                .or_else(|| case["wrong_separator_sha256"].as_str())
                .unwrap()
        );
        assert_ne!(actual, map.source_map_hash);
    }
}

fn run_limit_case(case: &Value) {
    let id = case["id"].as_str().unwrap();
    let count = case["construction"]["count"].as_u64().unwrap();
    let result = if id.contains("entries") {
        validate_source_map_entry_count(count)
    } else if id.contains("canonical_bytes") {
        validate_source_map_canonical_size(count)
    } else {
        validate_normalized_path(&portable_path(count as usize))
    };
    assert_expected(case, result, id);
}

fn synthetic_fixture(
    base_vir: &VirModule,
    fixtures: &BTreeMap<String, (InputKind, Vec<u8>)>,
) -> SyntheticFixture {
    let mut vir = base_vir.clone();
    vir.units[0].functions[0].blocks[0]
        .instructions
        .push(VirInstruction::Const {
            id: "t0".to_owned(),
            r#type: VirType::Bv {
                width: mpk_vc::BitVectorWidth::Bits8,
                signed: true,
            },
            value: VirLiteral::Integer(VirIntegerLiteral {
                int: VirIntLiteral {
                    value: mpk_vc::DecimalInteger::new("0".to_owned()).unwrap(),
                    width: mpk_vc::BitVectorWidth::Bits8,
                    signed: true,
                },
            }),
            safety_checks: vec![],
        });
    vir.vir_hash = LowercaseSha256::new("0".repeat(64)).unwrap();
    vir.vir_hash = mpk_vc::vir_hash(&vir).unwrap();
    let reference = SourceReference::Instruction {
        unit_id: "example.com/mpk/vector".to_owned(),
        function_id: "example.com/mpk/vector.Identity".to_owned(),
        block: "bb0".to_owned(),
        instruction: "t0".to_owned(),
    };
    let mut map = SourceMap {
        schema: "mpk.source_map.v0".to_owned(),
        source_ir_schema: "mpk.vir.v0".to_owned(),
        source_ir_hash: vir.vir_hash.as_str().to_owned(),
        entries: vec![
            SourceMapEntry {
                reference: SourceReference::Function {
                    unit_id: "example.com/mpk/vector".to_owned(),
                    function_id: "example.com/mpk/vector.Identity".to_owned(),
                },
                origin: SourceOrigin::Source {
                    input_kind: SourceInputKind::Source,
                    normalized_path: "identity.go".to_owned(),
                    start: 16,
                    end: 63,
                },
            },
            SourceMapEntry {
                reference: reference.clone(),
                origin: SourceOrigin::Synthetic {
                    reason: "profile.control_flow_join".to_owned(),
                },
            },
            SourceMapEntry {
                reference: SourceReference::Terminator {
                    unit_id: "example.com/mpk/vector".to_owned(),
                    function_id: "example.com/mpk/vector.Identity".to_owned(),
                    block: "bb0".to_owned(),
                },
                origin: SourceOrigin::Source {
                    input_kind: SourceInputKind::Source,
                    normalized_path: "identity.go".to_owned(),
                    start: 49,
                    end: 61,
                },
            },
        ],
        source_map_hash: "0".repeat(64),
    };
    map.source_map_hash = source_map_hash(&map).unwrap().as_str().to_owned();
    let (_, source) = &fixtures["source.identity_go"];
    SyntheticFixture {
        vir,
        map,
        captures: vec![(InputKind::Source, "identity.go".to_owned(), source.clone())],
        permissions: vec![SyntheticPermission {
            reference,
            reason: "profile.control_flow_join".to_owned(),
        }],
    }
}

fn valid_vir() -> VirModule {
    let vectors = load_json("develop/specs/vectors/vir-v0.json");
    let input = vectors["module_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "module.valid_go_identity")
        .unwrap()["input"]
        .clone();
    import_vir_json(&serde_json::to_vec(&input).unwrap()).unwrap()
}

fn fixture_sources(vectors: &Value) -> BTreeMap<String, (InputKind, Vec<u8>)> {
    vectors["fixture_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fixture| {
            assert_exact_keys(
                fixture,
                &[
                    "id",
                    "kind",
                    "normalized_path",
                    "base64",
                    "size_bytes",
                    "sha256",
                ],
            );
            assert_eq!(fixture["kind"], "source");
            validate_normalized_path(fixture["normalized_path"].as_str().unwrap()).unwrap();
            let bytes = decode_base64(fixture["base64"].as_str().unwrap());
            assert_eq!(bytes.len() as u64, fixture["size_bytes"].as_u64().unwrap());
            assert_eq!(sha256_hex(&bytes), fixture["sha256"]);
            (
                fixture["id"].as_str().unwrap().to_owned(),
                (InputKind::Source, bytes),
            )
        })
        .collect()
}

fn apply_construction(mut value: Value, construction: &Value) -> Value {
    if construction["fixture"] == "swap_inputs" {
        let indices = construction["indices"].as_array().unwrap();
        value["inputs"].as_array_mut().unwrap().swap(
            indices[0].as_u64().unwrap() as usize,
            indices[1].as_u64().unwrap() as usize,
        );
    }
    if let Some(patches) = construction.get("patches").and_then(Value::as_array) {
        for patch in patches {
            apply_patch(&mut value, patch);
        }
    }
    value
}

fn apply_patch(root: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().unwrap();
    let (parent_path, token) = path.rsplit_once('/').unwrap();
    let parent = if parent_path.is_empty() {
        root
    } else {
        root.pointer_mut(parent_path).unwrap()
    };
    let token = token.replace("~1", "/").replace("~0", "~");
    match patch["op"].as_str().unwrap() {
        "replace" => *child_mut(parent, &token) = patch["value"].clone(),
        "remove" => match parent {
            Value::Array(values) => {
                values.remove(token.parse::<usize>().unwrap());
            }
            Value::Object(values) => {
                values.remove(&token);
            }
            _ => panic!("patch parent is not a container"),
        },
        "add" => match parent {
            Value::Array(values) => {
                let index = token.parse::<usize>().unwrap();
                if index == values.len() {
                    values.push(patch["value"].clone());
                } else {
                    values.insert(index, patch["value"].clone());
                }
            }
            Value::Object(values) => {
                values.insert(token, patch["value"].clone());
            }
            _ => panic!("patch parent is not a container"),
        },
        operation => panic!("unknown patch operation {operation}"),
    }
}

fn child_mut<'a>(parent: &'a mut Value, token: &str) -> &'a mut Value {
    match parent {
        Value::Array(values) => &mut values[token.parse::<usize>().unwrap()],
        Value::Object(values) => values.get_mut(token).unwrap(),
        _ => panic!("patch parent is not a container"),
    }
}

fn assert_expected(case: &Value, result: Result<(), SourceMapError>, id: &str) {
    match case["expect"]["outcome"].as_str().unwrap() {
        "accept" => result.unwrap_or_else(|error| panic!("{id} rejected: {error}")),
        "reject" => {
            let error = result.expect_err(&format!("{id} unexpectedly accepted"));
            assert_eq!(error.phase.as_str(), case["expect"]["phase"], "{id}");
            assert_eq!(error.code.as_str(), case["expect"]["code"], "{id}");
        }
        outcome => panic!("unknown outcome {outcome}"),
    }
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected);
}

fn assert_case_shape(group: &str, case: &Value) {
    let allowed: BTreeSet<&str> = match group {
        "hash_cases" => [
            "id",
            "source_case",
            "domain_utf8",
            "wrong_domain_utf8",
            "separator_hex",
            "wrong_separator_hex",
            "canonical_without_hash_utf8_length",
            "expected_sha256",
            "wrong_domain_sha256",
            "wrong_separator_sha256",
            "expect_different",
            "construction",
            "expect",
        ]
        .into_iter()
        .collect(),
        "map_cases" => [
            "id",
            "context",
            "input",
            "json_text",
            "construction",
            "expect",
        ]
        .into_iter()
        .collect(),
        "reference_cases" | "mapping_cases" | "path_cases" | "limit_cases" => {
            ["id", "construction", "expect"].into_iter().collect()
        }
        _ => unreachable!(),
    };
    for key in case.as_object().unwrap().keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unknown {group} case field {key}"
        );
    }
    assert!(case.get("id").is_some());
    if let Some(construction) = case.get("construction") {
        assert_allowed_keys(
            construction,
            &["base", "fixture", "reason", "count", "context", "patches"],
            "source-map construction",
        );
        assert_patches(construction.get("patches"));
        if let Some(context) = construction.get("context") {
            assert_allowed_keys(
                context,
                &["source_cases", "additional_manifest_inputs", "validator"],
                "source-map construction context",
            );
            if let Some(inputs) = context
                .get("additional_manifest_inputs")
                .and_then(Value::as_array)
            {
                for input in inputs {
                    assert_exact_keys(input, &["kind", "normalized_path", "size_bytes", "sha256"]);
                }
            }
        }
    }
    if let Some(context) = case.get("context") {
        assert_allowed_keys(
            context,
            &["vir_case", "source_cases", "synthetic_profile"],
            "source-map case context",
        );
    }
    if let Some(expect) = case.get("expect") {
        assert_allowed_keys(
            expect,
            &["outcome", "phase", "code", "canonical_jcs_utf8_length"],
            "source-map expectation",
        );
    }
}

fn assert_patches(patches: Option<&Value>) {
    let Some(patches) = patches.and_then(Value::as_array) else {
        return;
    };
    for patch in patches {
        let expected = if patch.get("value").is_some() {
            vec!["op", "path", "value"]
        } else {
            vec!["op", "path"]
        };
        assert_exact_keys(patch, &expected);
    }
}

fn assert_allowed_keys(value: &Value, allowed: &[&str], owner: &str) {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    for key in value.as_object().unwrap().keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unknown {owner} field {key}"
        );
    }
}

fn portable_path(count: usize) -> String {
    let component_count = count / 256 + 1;
    let mut letters = count - (component_count - 1);
    let mut components = Vec::with_capacity(component_count);
    for index in 0..component_count {
        let remaining_components = component_count - index - 1;
        let length = (letters - remaining_components).min(255);
        components.push("a".repeat(length));
        letters -= length;
    }
    let path = components.join("/");
    assert_eq!(path.len(), count);
    path
}

fn load_json(path: &str) -> Value {
    serde_json::from_slice(&fs::read(repository_root().join(path)).unwrap()).unwrap()
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn decode_base64(input: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in input.bytes().take_while(|byte| *byte != b'=') {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
            accumulator &= (1_u32 << bits) - 1;
        }
    }
    output
}

fn sha256_hex(input: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(input);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
