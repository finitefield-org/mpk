use std::collections::BTreeSet;

use mpk_vc::{
    canonical_contract_hash_payload, canonical_json_bytes, canonical_vir_hash_payload,
    contract_hash, hash_canonical_json, hash_domain_separated_raw, parse_strict_json,
    sha256_raw_file_bytes, vir_hash, HashDomain, StrictJsonLimits, StrictJsonValue, VirContract,
    VirModule, CONTRACT_HASH_DOMAIN, VIR_HASH_DOMAIN,
};
use serde_json::Value;

const HASH_VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/vir-hash-v0.json");
const VIR_VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/vir-v0.json");
const VECTOR_LIMITS: StrictJsonLimits = StrictJsonLimits::new(1 << 20, 100_000, 256, 1 << 20);

#[test]
fn typed_vir_and_contract_hash_apis_match_every_canonical_vector() {
    let fixtures = Fixtures::load();
    let cases = array(field(&fixtures.hash, "canonical_cases"));
    assert_eq!(cases.len(), 4);

    for case in cases {
        assert_fields(
            case,
            &[
                "id",
                "domain",
                "source_case",
                "source_pointer",
                "excluded_fields",
                "expected_jcs",
                "expected_jcs_utf8_length",
                "expected_preimage_length",
                "expected_sha256",
            ],
        );
        let source = fixtures.source_case_value(text(field(case, "source_case")));
        let (canonical, digest) = match text(field(case, "domain")) {
            "vir" => {
                let module: VirModule = serde_json::from_value(source).expect("module decodes");
                (
                    canonical_vir_hash_payload(&module).expect("module payload canonicalizes"),
                    vir_hash(&module)
                        .expect("module hashes")
                        .as_str()
                        .to_owned(),
                )
            }
            "contract" => {
                let contract: VirContract = source
                    .pointer(text(field(case, "source_pointer")))
                    .cloned()
                    .and_then(|value| serde_json::from_value(value).ok())
                    .expect("contract decodes");
                (
                    canonical_contract_hash_payload(&contract)
                        .expect("contract payload canonicalizes"),
                    contract_hash(&contract)
                        .expect("contract hashes")
                        .as_str()
                        .to_owned(),
                )
            }
            domain => panic!("unknown domain {domain}"),
        };
        let id = text(field(case, "id"));
        assert_eq!(
            canonical,
            text(field(case, "expected_jcs")).as_bytes(),
            "{id}"
        );
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "expected_jcs_utf8_length")),
            "{id}"
        );
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        let domain_len = match text(field(case, "domain")) {
            "contract" => CONTRACT_HASH_DOMAIN.as_str().len(),
            "vir" => VIR_HASH_DOMAIN.as_str().len(),
            _ => unreachable!(),
        };
        assert_eq!(
            canonical.len() + domain_len + 1,
            integer(field(case, "expected_preimage_length")) as usize,
            "{id}"
        );
    }
}

#[test]
fn hash_domains_and_equivalence_controls_match_all_vectors() {
    let fixtures = Fixtures::load();
    let domains = field(&fixtures.hash, "domains");
    assert_fields(domains, &["contract", "vir"]);
    for (name, domain, hex) in [
        (
            "contract",
            CONTRACT_HASH_DOMAIN,
            "4d504b2d434f4e54524143542d302e31",
        ),
        ("vir", VIR_HASH_DOMAIN, "4d504b2d5649522d302e31"),
    ] {
        let metadata = field(domains, name);
        assert_fields(metadata, &["text", "utf8_hex", "separator_hex", "digest"]);
        assert_eq!(text(field(metadata, "text")), domain.as_str());
        assert_eq!(text(field(metadata, "utf8_hex")), hex);
        assert_eq!(text(field(metadata, "separator_hex")), "00");
        assert_eq!(text(field(metadata, "digest")), "sha256");
    }

    let cases = array(field(&fixtures.hash, "canonical_equivalence_cases"));
    assert_eq!(cases.len(), 4);
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = text(field(case, "id"));
        assert!(seen.insert(id), "duplicate case {id}");
        match id {
            "canonical.object_key_order" => {
                assert_fields(case, &["id", "json_texts", "expected_jcs"]);
                let values: Vec<_> = array(field(case, "json_texts"))
                    .iter()
                    .map(|input| strict(text(input).as_bytes()))
                    .collect();
                let canonical: Vec<_> = values
                    .iter()
                    .map(|value| canonical_json_bytes(value).expect("JCS succeeds"))
                    .collect();
                assert_eq!(canonical[0], canonical[1]);
                assert_eq!(canonical[0], text(field(case, "expected_jcs")).as_bytes());
            }
            "canonical.root_hash_excluded" => {
                assert_fields(case, &["id", "source_case", "patches", "expected_sha256"]);
                let mut module = fixtures.source_case(text(field(case, "source_case")));
                apply_patches(&mut module, field(case, "patches"));
                assert_hash_without(&module, "vir_hash", VIR_HASH_DOMAIN, case);
            }
            "canonical.contract_hash_excluded" => {
                assert_fields(
                    case,
                    &[
                        "id",
                        "source_case",
                        "source_pointer",
                        "patches",
                        "expected_sha256",
                    ],
                );
                let mut contract = fixtures.source_target(case);
                apply_patches(&mut contract, field(case, "patches"));
                assert_hash_without(&contract, "contract_hash", CONTRACT_HASH_DOMAIN, case);
            }
            "canonical.domain_separator_required" => {
                assert_fields(
                    case,
                    &[
                        "id",
                        "source_case",
                        "expected_sha256",
                        "without_separator_sha256",
                        "wrong_domain_text",
                        "wrong_domain_sha256",
                    ],
                );
                let module = fixtures.source_case(text(field(case, "source_case")));
                let payload = module
                    .clone_without_fields(&["vir_hash"])
                    .expect("vir_hash exists");
                assert_hash(&payload, VIR_HASH_DOMAIN, case);
                let canonical = canonical_json_bytes(&payload).expect("JCS succeeds");
                let mut no_separator = VIR_HASH_DOMAIN.as_str().as_bytes().to_vec();
                no_separator.extend_from_slice(&canonical);
                assert_eq!(
                    sha256_raw_file_bytes(&no_separator).to_hex(),
                    text(field(case, "without_separator_sha256"))
                );
                assert_eq!(text(field(case, "wrong_domain_text")), "MPK-VIR-0.2");
                let wrong = HashDomain::new("MPK-VIR-0.2");
                assert_eq!(
                    hash_canonical_json(wrong, &payload)
                        .expect("control hash succeeds")
                        .to_hex(),
                    text(field(case, "wrong_domain_sha256"))
                );
            }
            _ => panic!("unhandled equivalence vector {id}"),
        }
    }
}

#[test]
fn raw_ordered_and_every_semantic_mutation_vector_match() {
    let fixtures = Fixtures::load();
    let raw_cases = array(field(&fixtures.hash, "raw_contract_cases"));
    assert_eq!(raw_cases.len(), 1);
    for case in raw_cases {
        assert_fields(
            case,
            &[
                "id",
                "raw_json_texts",
                "expected_raw_utf8_lengths",
                "expected_raw_sha256",
                "normalized_source_case",
                "expected_contract_hash",
            ],
        );
        let texts = array(field(case, "raw_json_texts"));
        let lengths = array(field(case, "expected_raw_utf8_lengths"));
        let digests = array(field(case, "expected_raw_sha256"));
        assert_eq!(texts.len(), lengths.len());
        assert_eq!(texts.len(), digests.len());
        for index in 0..texts.len() {
            let bytes = text(&texts[index]).as_bytes();
            assert_eq!(bytes.len() as i64, integer(&lengths[index]));
            assert_eq!(sha256_raw_file_bytes(bytes).to_hex(), text(&digests[index]));
        }
        let contract = pointer(
            &fixtures.source_case(text(field(case, "normalized_source_case"))),
            "/units/0/functions/0/contracts",
        )
        .clone();
        let payload = contract
            .clone_without_fields(&["contract_hash"])
            .expect("contract hash exists");
        assert_eq!(
            hash_canonical_json(CONTRACT_HASH_DOMAIN, &payload)
                .expect("normalized contract hashes")
                .to_hex(),
            text(field(case, "expected_contract_hash"))
        );
    }

    let ordered = array(field(&fixtures.hash, "ordered_array_cases"));
    assert_eq!(ordered.len(), 1);
    for case in ordered {
        assert_fields(
            case,
            &[
                "id",
                "domain",
                "source_case",
                "source_pointer",
                "variants",
                "expect_different",
            ],
        );
        let mut digests = Vec::new();
        for variant in array(field(case, "variants")) {
            assert_fields(variant, &["patches", "expected_sha256"]);
            let mut contract = fixtures.source_target(case);
            apply_patches(&mut contract, field(variant, "patches"));
            let payload = contract
                .clone_without_fields(&["contract_hash"])
                .expect("contract hash exists");
            let digest = hash_canonical_json(CONTRACT_HASH_DOMAIN, &payload)
                .expect("ordered contract hashes");
            assert_eq!(digest.to_hex(), text(field(variant, "expected_sha256")));
            digests.push(digest);
        }
        assert!(boolean(field(case, "expect_different")));
        assert_ne!(digests[0], digests[1]);
    }

    let mutations = array(field(&fixtures.hash, "mutation_cases"));
    assert_eq!(mutations.len(), 10);
    let mut seen = BTreeSet::new();
    for case in mutations {
        assert_fields(
            case,
            &[
                "id",
                "source_case",
                "patches",
                "expected_jcs_utf8_length",
                "expected_sha256",
                "different_from",
            ],
        );
        let id = text(field(case, "id"));
        assert!(seen.insert(id), "duplicate mutation {id}");
        let mut module = fixtures.source_case(text(field(case, "source_case")));
        apply_patches(&mut module, field(case, "patches"));
        let payload = module
            .clone_without_fields(&["vir_hash"])
            .expect("vir_hash exists");
        let canonical = canonical_json_bytes(&payload).expect("mutation canonicalizes");
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "expected_jcs_utf8_length")),
            "{id}"
        );
        let digest = hash_canonical_json(VIR_HASH_DOMAIN, &payload)
            .expect("mutation hashes")
            .to_hex();
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        assert_ne!(digest, text(field(case, "different_from")), "{id}");
    }
}

#[test]
fn hash_vector_container_has_exact_frozen_shape() {
    let fixtures = Fixtures::load();
    assert_fields(
        &fixtures.hash,
        &[
            "schema",
            "spec_schema",
            "source_vector",
            "owner_test",
            "domains",
            "canonical_cases",
            "canonical_equivalence_cases",
            "raw_contract_cases",
            "ordered_array_cases",
            "mutation_cases",
        ],
    );
    assert_eq!(
        text(field(&fixtures.hash, "schema")),
        "mpk.vir.hash_vectors.v0"
    );
    assert_eq!(text(field(&fixtures.hash, "spec_schema")), "mpk.vir.v0");
    assert_eq!(
        text(field(&fixtures.hash, "source_vector")),
        "develop/specs/vectors/vir-v0.json"
    );
    assert_eq!(
        text(field(&fixtures.hash, "owner_test")),
        "crates/mpk-vc/tests/vir_hash.rs"
    );
}

fn assert_hash_without(
    value: &StrictJsonValue,
    field_name: &str,
    domain: HashDomain,
    case: &StrictJsonValue,
) {
    let payload = value
        .clone_without_fields(&[field_name])
        .expect("self-hash field exists");
    assert_hash(&payload, domain, case);
}

fn assert_hash(value: &StrictJsonValue, domain: HashDomain, case: &StrictJsonValue) {
    assert_eq!(
        hash_canonical_json(domain, value)
            .expect("domain-separated hash succeeds")
            .to_hex(),
        text(field(case, "expected_sha256"))
    );
}

struct Fixtures {
    hash: StrictJsonValue,
    source: StrictJsonValue,
    source_value: Value,
}

impl Fixtures {
    fn load() -> Self {
        let hash = strict(HASH_VECTORS);
        let source = strict(VIR_VECTORS);
        let source_value: Value =
            serde_json::from_slice(VIR_VECTORS).expect("source vectors parse");
        Self {
            hash,
            source,
            source_value,
        }
    }

    fn source_case(&self, id: &str) -> StrictJsonValue {
        field(
            find_case(array(field(&self.source, "module_cases")), id),
            "input",
        )
        .clone()
    }

    fn source_case_value(&self, id: &str) -> Value {
        self.source_value["module_cases"]
            .as_array()
            .and_then(|cases| cases.iter().find(|case| case["id"] == id))
            .and_then(|case| case.get("input"))
            .cloned()
            .unwrap_or_else(|| panic!("missing module case {id}"))
    }

    fn source_target(&self, case: &StrictJsonValue) -> StrictJsonValue {
        let source = self.source_case(text(field(case, "source_case")));
        match case.get("source_pointer") {
            Some(path) => pointer(&source, text(path)).clone(),
            None => source,
        }
    }
}

fn strict(bytes: &[u8]) -> StrictJsonValue {
    parse_strict_json(bytes, VECTOR_LIMITS).expect("vector JSON parses strictly")
}

fn field<'a>(value: &'a StrictJsonValue, name: &str) -> &'a StrictJsonValue {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name:?}"))
}

fn array(value: &StrictJsonValue) -> &[StrictJsonValue] {
    value.as_array().expect("value is an array")
}

fn text(value: &StrictJsonValue) -> &str {
    value.as_str().expect("value is a string")
}

fn integer(value: &StrictJsonValue) -> i64 {
    value.as_i64().expect("value is an integer")
}

fn boolean(value: &StrictJsonValue) -> bool {
    value.as_bool().expect("value is Boolean")
}

fn assert_fields(value: &StrictJsonValue, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("value is an object")
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected, "object field set changed");
}

fn find_case<'a>(cases: &'a [StrictJsonValue], id: &str) -> &'a StrictJsonValue {
    cases
        .iter()
        .find(|case| text(field(case, "id")) == id)
        .unwrap_or_else(|| panic!("missing case {id}"))
}

fn pointer<'a>(value: &'a StrictJsonValue, path: &str) -> &'a StrictJsonValue {
    if path == "/" {
        return value;
    }
    let mut current = value;
    for token in pointer_tokens(path) {
        current = match current {
            StrictJsonValue::Object(entries) => entries
                .iter()
                .find_map(|(name, item)| (name == &token).then_some(item))
                .unwrap_or_else(|| panic!("missing pointer field {token}")),
            StrictJsonValue::Array(values) => &values[token.parse::<usize>().expect("array index")],
            _ => panic!("pointer descends through scalar"),
        };
    }
    current
}

fn pointer_mut<'a>(value: &'a mut StrictJsonValue, tokens: &[String]) -> &'a mut StrictJsonValue {
    let Some((first, rest)) = tokens.split_first() else {
        return value;
    };
    let child = match value {
        StrictJsonValue::Object(entries) => entries
            .iter_mut()
            .find_map(|(name, item)| (name == first).then_some(item))
            .unwrap_or_else(|| panic!("missing pointer field {first}")),
        StrictJsonValue::Array(values) => &mut values[first.parse::<usize>().expect("array index")],
        _ => panic!("pointer descends through scalar"),
    };
    pointer_mut(child, rest)
}

fn pointer_tokens(path: &str) -> Vec<String> {
    path.strip_prefix('/')
        .expect("JSON pointer starts with slash")
        .split('/')
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn apply_patches(target: &mut StrictJsonValue, patches: &StrictJsonValue) {
    for patch in array(patches) {
        assert_fields(patch, &["op", "path", "value"]);
        let operation = text(field(patch, "op"));
        let mut tokens = pointer_tokens(text(field(patch, "path")));
        let last = tokens.pop().expect("patch does not target root");
        let parent = pointer_mut(target, &tokens);
        match parent {
            StrictJsonValue::Object(entries) => {
                let position = entries.iter().position(|(name, _)| name == &last);
                match operation {
                    "add" => {
                        let replacement = field(patch, "value").clone();
                        if let Some(index) = position {
                            entries[index].1 = replacement;
                        } else {
                            entries.push((last, replacement));
                        }
                    }
                    "replace" => {
                        entries[position.expect("replace field exists")].1 =
                            field(patch, "value").clone();
                    }
                    _ => panic!("unsupported object patch {operation}"),
                }
            }
            StrictJsonValue::Array(values) => {
                let index = last.parse::<usize>().expect("array index");
                match operation {
                    "replace" => values[index] = field(patch, "value").clone(),
                    "add" => values.insert(index, field(patch, "value").clone()),
                    _ => panic!("unsupported array patch {operation}"),
                }
            }
            _ => panic!("patch parent is scalar"),
        }
    }
}

#[test]
fn domain_separator_is_exactly_one_zero_byte() {
    let payload = b"{}";
    let digest = hash_domain_separated_raw(VIR_HASH_DOMAIN, payload).expect("hash succeeds");
    let mut explicit = VIR_HASH_DOMAIN.as_str().as_bytes().to_vec();
    explicit.push(0);
    explicit.extend_from_slice(payload);
    assert_eq!(digest, sha256_raw_file_bytes(&explicit));
}
