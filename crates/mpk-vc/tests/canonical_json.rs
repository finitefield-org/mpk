use mpk_vc::{
    canonical_json_bytes, hash_canonical_inventory, hash_canonical_json, hash_domain_separated_raw,
    normalize_unordered_set_by, normalize_unordered_utf8_strings, parse_strict_json,
    sha256_raw_file_bytes, CanonicalJsonError, HashDomain, HashError, StrictJsonError,
    StrictJsonLimits, StrictJsonValue, MAX_SAFE_JSON_INTEGER, MAX_SUPPORTED_JSON_DEPTH,
    MIN_SAFE_JSON_INTEGER,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const VIR_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-0.1");
const CONTRACT_DOMAIN: HashDomain = HashDomain::new("MPK-CONTRACT-0.1");
const WRONG_VIR_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-0.2");

const VECTOR_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(64 * 1024 * 1024, 4_000_000, 256, 32 * 1024 * 1024);

#[test]
fn canonical_json_strict_parser_rejects_lexical_and_limit_violations() {
    let ordinary = StrictJsonLimits::new(1_024, 64, 16, 64);

    let duplicate = parse_strict_json(br#"{"a":1,"a":2}"#, ordinary)
        .expect_err("duplicate object name must reject");
    assert_eq!(
        duplicate,
        StrictJsonError::DuplicateObjectName {
            name: "a".to_owned()
        }
    );

    for rejected in ["1.0", "1e0", "1E+3", "-2.5"] {
        assert!(matches!(
            parse_strict_json(rejected.as_bytes(), ordinary),
            Err(StrictJsonError::FloatingPointNumber { .. })
        ));
    }
    for rejected in [
        "9007199254740992",
        "-9007199254740992",
        "18446744073709551616",
    ] {
        assert!(matches!(
            parse_strict_json(rejected.as_bytes(), ordinary),
            Err(StrictJsonError::IntegerOutOfRange { .. })
        ));
    }

    assert!(matches!(
        parse_strict_json(&[0xef, 0xbb, 0xbf, b'{', b'}'], ordinary),
        Err(StrictJsonError::Bom)
    ));
    assert!(matches!(
        parse_strict_json(&[b'"', 0xff, b'"'], ordinary),
        Err(StrictJsonError::InvalidUtf8 { .. })
    ));
    assert!(matches!(
        parse_strict_json(br#""\ud800""#, ordinary),
        Err(StrictJsonError::InvalidJson { .. })
    ));
    assert!(matches!(
        parse_strict_json(br#""\udc00""#, ordinary),
        Err(StrictJsonError::InvalidJson { .. })
    ));
    assert_eq!(
        parse_strict_json(br#""\ud83d\ude00""#, ordinary).expect("a valid surrogate pair decodes"),
        StrictJsonValue::String("😀".to_owned())
    );
    for rejected in [br#"{}x"#.as_slice(), br#"{}{}"#.as_slice()] {
        assert!(matches!(
            parse_strict_json(rejected, ordinary),
            Err(StrictJsonError::TrailingBytes { .. })
        ));
    }
    for rejected in [br#"01"#.as_slice(), br#"[1,]"#.as_slice()] {
        assert!(matches!(
            parse_strict_json(rejected, ordinary),
            Err(StrictJsonError::InvalidJson { .. })
        ));
    }

    let exact = "[{\"é\":0}]".as_bytes();
    let exact_limits = StrictJsonLimits::new(exact.len() as u64, 3, 2, 2);
    parse_strict_json(exact, exact_limits).expect("all inclusive limits accept equality");
    assert!(matches!(
        parse_strict_json(
            exact,
            StrictJsonLimits::new(exact.len() as u64 - 1, 3, 2, 2)
        ),
        Err(StrictJsonError::InputBytesExceeded { .. })
    ));
    assert!(matches!(
        parse_strict_json(exact, StrictJsonLimits::new(exact.len() as u64, 2, 2, 2)),
        Err(StrictJsonError::NodeLimitExceeded { .. })
    ));
    assert!(matches!(
        parse_strict_json(exact, StrictJsonLimits::new(exact.len() as u64, 3, 1, 2)),
        Err(StrictJsonError::DepthLimitExceeded { .. })
    ));
    assert!(matches!(
        parse_strict_json(exact, StrictJsonLimits::new(exact.len() as u64, 3, 2, 1)),
        Err(StrictJsonError::StringBytesExceeded { .. })
    ));

    assert!(matches!(
        parse_strict_json(
            b"null",
            StrictJsonLimits::new(4, 1, MAX_SUPPORTED_JSON_DEPTH + 1, 0)
        ),
        Err(StrictJsonError::UnsupportedDepthLimit {
            requested,
            maximum: MAX_SUPPORTED_JSON_DEPTH,
        }) if requested == MAX_SUPPORTED_JSON_DEPTH + 1
    ));
    let maximum_depth = usize::try_from(MAX_SUPPORTED_JSON_DEPTH).expect("depth fits usize");
    let deepest = format!(
        "{}0{}",
        "[".repeat(maximum_depth),
        "]".repeat(maximum_depth)
    );
    parse_strict_json(
        deepest.as_bytes(),
        StrictJsonLimits::new(
            deepest.len() as u64,
            MAX_SUPPORTED_JSON_DEPTH + 1,
            MAX_SUPPORTED_JSON_DEPTH,
            0,
        ),
    )
    .expect("the maximum supported depth parses");
    let too_deep = format!("[{deepest}]");
    assert!(matches!(
        parse_strict_json(
            too_deep.as_bytes(),
            StrictJsonLimits::new(
                too_deep.len() as u64,
                MAX_SUPPORTED_JSON_DEPTH + 2,
                MAX_SUPPORTED_JSON_DEPTH,
                0,
            ),
        ),
        Err(StrictJsonError::DepthLimitExceeded {
            maximum: MAX_SUPPORTED_JSON_DEPTH
        })
    ));

    let boundary = format!("[{MIN_SAFE_JSON_INTEGER},{MAX_SAFE_JSON_INTEGER}]");
    parse_strict_json(boundary.as_bytes(), ordinary).expect("safe integer endpoints accept");
    let spaced = parse_strict_json(b" \n { \"b\" : 1, \"a\" : null } \t", ordinary)
        .expect("generic strict parser accepts insignificant JSON whitespace");
    assert_eq!(
        canonical_json_bytes(&spaced).expect("spaced value canonicalizes"),
        br#"{"a":null,"b":1}"#
    );
}

#[test]
fn canonical_json_encoder_uses_utf16_key_order_and_exact_string_escaping() {
    let ordered = StrictJsonValue::Object(vec![
        ("\u{e000}".to_owned(), StrictJsonValue::Integer(1)),
        ("😀".to_owned(), StrictJsonValue::Integer(2)),
        ("a".to_owned(), StrictJsonValue::Integer(3)),
    ]);
    assert_eq!(
        canonical_json_bytes(&ordered).expect("object canonicalizes"),
        "{\"a\":3,\"😀\":2,\"\u{e000}\":1}".as_bytes()
    );

    let escaped =
        StrictJsonValue::String("\u{0008}\t\n\u{000c}\r\"\\/\u{0000}é\u{2028}".to_owned());
    let encoded = canonical_json_bytes(&escaped).expect("string canonicalizes");
    assert_eq!(
        encoded,
        "\"\\b\\t\\n\\f\\r\\\"\\\\/\\u0000é\u{2028}\"".as_bytes()
    );
    assert!(!encoded.contains(&b'\n'));
    assert!(!encoded.starts_with(&[0xef, 0xbb, 0xbf]));

    let first = StrictJsonValue::Array(vec![
        StrictJsonValue::Bool(true),
        StrictJsonValue::Bool(false),
    ]);
    let second = StrictJsonValue::Array(vec![
        StrictJsonValue::Bool(false),
        StrictJsonValue::Bool(true),
    ]);
    assert_eq!(
        canonical_json_bytes(&first).expect("array encodes"),
        b"[true,false]"
    );
    assert_eq!(
        canonical_json_bytes(&second).expect("array encodes"),
        b"[false,true]"
    );

    let duplicate = StrictJsonValue::Object(vec![
        ("same".to_owned(), StrictJsonValue::Integer(1)),
        ("same".to_owned(), StrictJsonValue::Integer(2)),
    ]);
    assert!(matches!(
        canonical_json_bytes(&duplicate),
        Err(CanonicalJsonError::DuplicateObjectName { .. })
    ));
    assert!(matches!(
        canonical_json_bytes(&StrictJsonValue::Integer(MAX_SAFE_JSON_INTEGER + 1)),
        Err(CanonicalJsonError::IntegerOutOfRange { .. })
    ));

    let duplicate_self_hash = StrictJsonValue::Object(vec![
        (
            "vir_hash".to_owned(),
            StrictJsonValue::String("a".to_owned()),
        ),
        (
            "vir_hash".to_owned(),
            StrictJsonValue::String("b".to_owned()),
        ),
    ]);
    assert!(matches!(
        duplicate_self_hash.clone_without_fields(&["vir_hash"]),
        Err(mpk_vc::ObjectFieldsError::DuplicateField { .. })
    ));
}

#[test]
fn canonical_json_unordered_set_normalization_is_explicit() {
    let mut strings = vec!["é".to_owned(), "z".to_owned(), "a".to_owned()];
    normalize_unordered_utf8_strings(&mut strings).expect("distinct set normalizes");
    assert_eq!(strings, ["a", "z", "é"]);

    let mut duplicates = vec!["b".to_owned(), "a".to_owned(), "b".to_owned()];
    assert!(normalize_unordered_utf8_strings(&mut duplicates).is_err());

    let mut integers = vec![3, 1, 2];
    normalize_unordered_set_by(&mut integers, i32::cmp).expect("custom set normalizes");
    assert_eq!(integers, [1, 2, 3]);
    assert_eq!(
        1_i32.cmp(&2),
        Ordering::Less,
        "normalizer comparator remains an ordinary total order"
    );
}

#[test]
fn canonical_json_hash_api_separates_domains_and_raw_preimages() {
    let value = parse_strict_json(br#"{"b":1,"a":2}"#, VECTOR_LIMITS).expect("hash fixture parses");
    let vir = hash_canonical_json(VIR_DOMAIN, &value).expect("VIR-domain hash succeeds");
    let contract =
        hash_canonical_json(CONTRACT_DOMAIN, &value).expect("contract-domain hash succeeds");
    assert_ne!(vir, contract);
    assert_eq!(vir.to_hex().len(), 64);
    assert!(vir
        .to_hex()
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));

    let inventory = StrictJsonValue::Array(vec![StrictJsonValue::String("entry".to_owned())]);
    assert_eq!(
        hash_canonical_inventory(VIR_DOMAIN, &inventory).expect("inventory hashes"),
        hash_canonical_json(VIR_DOMAIN, &inventory).expect("equivalent array hashes")
    );
    assert!(matches!(
        hash_canonical_inventory(VIR_DOMAIN, &StrictJsonValue::Null),
        Err(HashError::InventoryNotContainer)
    ));

    let raw = b"{ \"not\": \"canonical JSON\" }\n";
    assert_ne!(
        sha256_raw_file_bytes(raw),
        hash_domain_separated_raw(VIR_DOMAIN, raw).expect("explicit raw domain hashes")
    );
    assert!(matches!(
        hash_canonical_json(HashDomain::new(""), &value),
        Err(HashError::InvalidDomain { .. })
    ));
    assert!(matches!(
        hash_canonical_json(HashDomain::new("bad domain"), &value),
        Err(HashError::InvalidDomain { .. })
    ));
}

#[test]
fn canonical_json_every_frozen_vector_container_is_loaded_and_digest_matched() {
    let root = repository_root();
    let manifest_path = root.join("develop/specs/vectors/manifest.json");
    let manifest_bytes = fs::read(&manifest_path).expect("read vector manifest");
    let manifest =
        parse_strict_json(&manifest_bytes, VECTOR_LIMITS).expect("vector manifest is strict JSON");
    assert_eq!(
        text(field(&manifest, "schema")),
        "mpk.spec.vector_manifest.v0"
    );

    let vectors = array(field(&manifest, "vectors"));
    assert_eq!(
        vectors.len(),
        21,
        "closed specification-vector manifest size changed"
    );
    let mut seen_paths = BTreeSet::new();
    for entry in vectors {
        let relative = text(field(entry, "path"));
        assert!(relative.starts_with("develop/specs/vectors/"));
        assert!(!relative.split('/').any(|component| component == ".."));
        assert!(
            seen_paths.insert(relative.to_owned()),
            "duplicate vector path"
        );

        let bytes = fs::read(root.join(relative)).expect("read owned vector file");
        match parse_strict_json(&bytes, VECTOR_LIMITS) {
            Ok(parsed) => {
                canonical_json_bytes(&parsed)
                    .unwrap_or_else(|error| panic!("{relative} must canonicalize: {error}"));
            }
            Err(StrictJsonError::FloatingPointNumber { .. })
                if relative == "develop/specs/vectors/ai-explain-v1.json" =>
            {
                let provider: serde_json::Value =
                    serde_json::from_slice(&bytes).expect("provider vector is valid general JSON");
                let mut float_paths = Vec::new();
                collect_floating_number_paths(&provider, "", &mut float_paths);
                assert_eq!(
                    float_paths,
                    [
                        "/request_fixtures/0/generation_config/temperature",
                        "/request_fixtures/1/generation_config/temperature",
                        "/request_fixtures/2/generation_config/temperature",
                    ],
                    "the only vector-container floats are the frozen provider temperatures"
                );
                for path in &float_paths {
                    assert_eq!(
                        provider.pointer(path).and_then(serde_json::Value::as_f64),
                        Some(0.0)
                    );
                }
            }
            Err(error) => panic!("{relative} must satisfy the shared JSON rules: {error}"),
        }
        assert_eq!(
            sha256_raw_file_bytes(&bytes).to_hex(),
            text(field(entry, "sha256")),
            "raw vector digest mismatch for {relative}"
        );
    }
}

#[test]
fn canonical_json_vir_hash_canonical_cases_and_equivalences_match_vectors() {
    let fixtures = HashFixtures::load();
    let canonical_cases = array(field(&fixtures.hash_vectors, "canonical_cases"));
    assert_eq!(canonical_cases.len(), 4);
    for case in canonical_cases {
        let mut target = fixtures.source_target(case);
        target = remove_fields(&target, field(case, "excluded_fields"));
        let canonical = canonical_json_bytes(&target).expect("canonical case encodes");
        assert_eq!(
            std::str::from_utf8(&canonical).expect("JCS is UTF-8"),
            text(field(case, "expected_jcs")),
            "JCS mismatch for {}",
            text(field(case, "id"))
        );
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "expected_jcs_utf8_length"))
        );
        let domain = fixtures.domain(text(field(case, "domain")));
        assert_eq!(
            canonical.len() as i64 + domain.as_str().len() as i64 + 1,
            integer(field(case, "expected_preimage_length"))
        );
        assert_eq!(
            hash_canonical_json(domain, &target)
                .expect("canonical case hashes")
                .to_hex(),
            text(field(case, "expected_sha256"))
        );
    }

    fixtures.assert_domain_metadata();
    let equivalences = array(field(&fixtures.hash_vectors, "canonical_equivalence_cases"));
    assert_eq!(equivalences.len(), 4);
    let mut seen = BTreeSet::new();
    for case in equivalences {
        let id = text(field(case, "id"));
        assert!(seen.insert(id));
        match id {
            "canonical.object_key_order" => {
                let variants = array(field(case, "json_texts"));
                let first = parse_strict_json(text(&variants[0]).as_bytes(), VECTOR_LIMITS)
                    .expect("first equivalent JSON parses");
                let second = parse_strict_json(text(&variants[1]).as_bytes(), VECTOR_LIMITS)
                    .expect("second equivalent JSON parses");
                let first_jcs = canonical_json_bytes(&first).expect("first canonicalizes");
                let second_jcs = canonical_json_bytes(&second).expect("second canonicalizes");
                assert_eq!(first_jcs, second_jcs);
                assert_eq!(first_jcs, text(field(case, "expected_jcs")).as_bytes());
            }
            "canonical.root_hash_excluded" => {
                let mut module = fixtures.source_case(text(field(case, "source_case")));
                apply_patches(&mut module, field(case, "patches"));
                let preimage = module
                    .clone_without_fields(&["vir_hash"])
                    .expect("root hash field exists");
                assert_eq!(
                    hash_canonical_json(VIR_DOMAIN, &preimage)
                        .expect("root hashes")
                        .to_hex(),
                    text(field(case, "expected_sha256"))
                );
            }
            "canonical.contract_hash_excluded" => {
                let mut contract = fixtures.source_target(case);
                apply_patches(&mut contract, field(case, "patches"));
                let preimage = contract
                    .clone_without_fields(&["contract_hash"])
                    .expect("contract hash field exists");
                assert_eq!(
                    hash_canonical_json(CONTRACT_DOMAIN, &preimage)
                        .expect("contract hashes")
                        .to_hex(),
                    text(field(case, "expected_sha256"))
                );
            }
            "canonical.domain_separator_required" => {
                let module = fixtures.source_case(text(field(case, "source_case")));
                let preimage = module
                    .clone_without_fields(&["vir_hash"])
                    .expect("root hash field exists");
                let canonical = canonical_json_bytes(&preimage).expect("module canonicalizes");
                assert_eq!(
                    hash_canonical_json(VIR_DOMAIN, &preimage)
                        .expect("domain-separated hash succeeds")
                        .to_hex(),
                    text(field(case, "expected_sha256"))
                );
                let mut without_separator = VIR_DOMAIN.as_str().as_bytes().to_vec();
                without_separator.extend_from_slice(&canonical);
                assert_eq!(
                    sha256_raw_file_bytes(&without_separator).to_hex(),
                    text(field(case, "without_separator_sha256"))
                );
                assert_eq!(text(field(case, "wrong_domain_text")), "MPK-VIR-0.2");
                assert_eq!(
                    hash_canonical_json(WRONG_VIR_DOMAIN, &preimage)
                        .expect("wrong-domain control hashes")
                        .to_hex(),
                    text(field(case, "wrong_domain_sha256"))
                );
            }
            _ => panic!("unhandled canonical equivalence vector {id}"),
        }
    }
}

#[test]
fn canonical_json_vir_hash_raw_ordered_and_mutation_cases_match_vectors() {
    let fixtures = HashFixtures::load();

    let raw_cases = array(field(&fixtures.hash_vectors, "raw_contract_cases"));
    assert_eq!(raw_cases.len(), 1);
    for case in raw_cases {
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
        let preimage = contract
            .clone_without_fields(&["contract_hash"])
            .expect("normalized contract hash exists");
        assert_eq!(
            hash_canonical_json(CONTRACT_DOMAIN, &preimage)
                .expect("normalized contract hashes")
                .to_hex(),
            text(field(case, "expected_contract_hash"))
        );
    }

    let ordered_cases = array(field(&fixtures.hash_vectors, "ordered_array_cases"));
    assert_eq!(ordered_cases.len(), 1);
    for case in ordered_cases {
        let mut digests = Vec::new();
        for variant in array(field(case, "variants")) {
            let mut contract = fixtures.source_target(case);
            apply_patches(&mut contract, field(variant, "patches"));
            let preimage = contract
                .clone_without_fields(&["contract_hash"])
                .expect("contract hash exists");
            let digest =
                hash_canonical_json(fixtures.domain(text(field(case, "domain"))), &preimage)
                    .expect("ordered variant hashes");
            assert_eq!(digest.to_hex(), text(field(variant, "expected_sha256")));
            digests.push(digest);
        }
        assert!(boolean(field(case, "expect_different")));
        assert_ne!(digests[0], digests[1]);
    }

    let mutations = array(field(&fixtures.hash_vectors, "mutation_cases"));
    assert_eq!(mutations.len(), 10);
    let mut seen = BTreeSet::new();
    for case in mutations {
        let id = text(field(case, "id"));
        assert!(seen.insert(id), "duplicate mutation ID {id}");
        let mut module = fixtures.source_case(text(field(case, "source_case")));
        apply_patches(&mut module, field(case, "patches"));
        let preimage = module
            .clone_without_fields(&["vir_hash"])
            .expect("root hash exists");
        let canonical = canonical_json_bytes(&preimage).expect("mutation canonicalizes");
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "expected_jcs_utf8_length")),
            "canonical length mismatch for {id}"
        );
        let digest = hash_canonical_json(VIR_DOMAIN, &preimage)
            .expect("mutation hashes")
            .to_hex();
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        assert_ne!(digest, text(field(case, "different_from")), "{id}");
    }
}

#[test]
fn canonical_json_duplicate_vector_case_rejects_before_typed_deserialization() {
    let source = read_strict("develop/specs/vectors/vir-v0.json");
    let case = find_case(
        array(field(&source, "module_cases")),
        "module.reject_duplicate_key",
    );
    let error = parse_strict_json(text(field(case, "json_text")).as_bytes(), VECTOR_LIMITS)
        .expect_err("normative duplicate-key input rejects");
    assert!(matches!(error, StrictJsonError::DuplicateObjectName { .. }));
    assert_eq!(
        text(field(field(case, "expect"), "code")),
        "VIR_JSON_DUPLICATE_KEY"
    );
}

struct HashFixtures {
    hash_vectors: StrictJsonValue,
    source_vectors: StrictJsonValue,
}

impl HashFixtures {
    fn load() -> Self {
        let hash_vectors = read_strict("develop/specs/vectors/vir-hash-v0.json");
        let source_path = text(field(&hash_vectors, "source_vector"));
        let source_vectors = read_strict(source_path);
        assert_eq!(
            text(field(&hash_vectors, "schema")),
            "mpk.vir.hash_vectors.v0"
        );
        assert_eq!(text(field(&hash_vectors, "spec_schema")), "mpk.vir.v0");
        assert_eq!(
            text(field(&source_vectors, "schema")),
            "mpk.vir.conformance.v0"
        );
        Self {
            hash_vectors,
            source_vectors,
        }
    }

    fn source_case(&self, id: &str) -> StrictJsonValue {
        field(
            find_case(array(field(&self.source_vectors, "module_cases")), id),
            "input",
        )
        .clone()
    }

    fn source_target(&self, case: &StrictJsonValue) -> StrictJsonValue {
        let source = self.source_case(text(field(case, "source_case")));
        match case.get("source_pointer") {
            Some(pointer_value) => pointer(&source, text(pointer_value)).clone(),
            None => source,
        }
    }

    fn domain(&self, name: &str) -> HashDomain {
        match name {
            "contract" => CONTRACT_DOMAIN,
            "vir" => VIR_DOMAIN,
            _ => panic!("unknown hash vector domain {name}"),
        }
    }

    fn assert_domain_metadata(&self) {
        let domains = field(&self.hash_vectors, "domains");
        let expected = [
            (
                "contract",
                CONTRACT_DOMAIN,
                "4d504b2d434f4e54524143542d302e31",
            ),
            ("vir", VIR_DOMAIN, "4d504b2d5649522d302e31"),
        ];
        for (name, domain, expected_hex) in expected {
            let metadata = field(domains, name);
            assert_eq!(text(field(metadata, "text")), domain.as_str());
            assert_eq!(text(field(metadata, "utf8_hex")), expected_hex);
            assert_eq!(text(field(metadata, "separator_hex")), "00");
            assert_eq!(text(field(metadata, "digest")), "sha256");
        }
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn read_strict(relative: &str) -> StrictJsonValue {
    let bytes = fs::read(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    parse_strict_json(&bytes, VECTOR_LIMITS)
        .unwrap_or_else(|error| panic!("parse {relative} strictly: {error}"))
}

fn field<'a>(value: &'a StrictJsonValue, name: &str) -> &'a StrictJsonValue {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing object field {name:?}"))
}

fn array(value: &StrictJsonValue) -> &[StrictJsonValue] {
    value.as_array().expect("JSON value must be an array")
}

fn text(value: &StrictJsonValue) -> &str {
    value.as_str().expect("JSON value must be a string")
}

fn integer(value: &StrictJsonValue) -> i64 {
    value.as_i64().expect("JSON value must be an integer")
}

fn boolean(value: &StrictJsonValue) -> bool {
    value.as_bool().expect("JSON value must be a Boolean")
}

fn collect_floating_number_paths(value: &serde_json::Value, path: &str, output: &mut Vec<String>) {
    match value {
        serde_json::Value::Number(number) if number.is_f64() => output.push(path.to_owned()),
        serde_json::Value::Array(values) => {
            for (index, item) in values.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                collect_floating_number_paths(item, &child_path, output);
            }
        }
        serde_json::Value::Object(entries) => {
            for (name, item) in entries {
                let escaped = name.replace('~', "~0").replace('/', "~1");
                let child_path = format!("{path}/{escaped}");
                collect_floating_number_paths(item, &child_path, output);
            }
        }
        _ => {}
    }
}

fn find_case<'a>(cases: &'a [StrictJsonValue], id: &str) -> &'a StrictJsonValue {
    cases
        .iter()
        .find(|case| text(field(case, "id")) == id)
        .unwrap_or_else(|| panic!("missing vector case {id}"))
}

fn remove_fields(value: &StrictJsonValue, fields: &StrictJsonValue) -> StrictJsonValue {
    let names: Vec<_> = array(fields).iter().map(text).collect();
    value
        .clone_without_fields(&names)
        .expect("excluded fields must exist exactly once")
}

fn pointer<'a>(value: &'a StrictJsonValue, path: &str) -> &'a StrictJsonValue {
    if path.is_empty() || path == "/" {
        return value;
    }
    let tokens = pointer_tokens(path);
    let mut current = value;
    for token in tokens {
        current = match current {
            StrictJsonValue::Object(entries) => entries
                .iter()
                .find_map(|(name, item)| (name == &token).then_some(item))
                .unwrap_or_else(|| panic!("missing object pointer token {token:?}")),
            StrictJsonValue::Array(values) => {
                let index = token
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("invalid array pointer token {token:?}"));
                values
                    .get(index)
                    .unwrap_or_else(|| panic!("array pointer index {index} is out of range"))
            }
            _ => panic!("pointer descends through a scalar at {token:?}"),
        };
    }
    current
}

fn pointer_mut<'a>(value: &'a mut StrictJsonValue, tokens: &[String]) -> &'a mut StrictJsonValue {
    let Some((first, rest)) = tokens.split_first() else {
        return value;
    };
    let next = match value {
        StrictJsonValue::Object(entries) => entries
            .iter_mut()
            .find_map(|(name, item)| (name == first).then_some(item))
            .unwrap_or_else(|| panic!("missing object pointer token {first:?}")),
        StrictJsonValue::Array(values) => {
            let index = first
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("invalid array pointer token {first:?}"));
            values
                .get_mut(index)
                .unwrap_or_else(|| panic!("array pointer index {index} is out of range"))
        }
        _ => panic!("pointer descends through a scalar at {first:?}"),
    };
    pointer_mut(next, rest)
}

fn pointer_tokens(path: &str) -> Vec<String> {
    assert!(
        path.starts_with('/'),
        "JSON pointer must start with '/': {path}"
    );
    path[1..]
        .split('/')
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn apply_patches(target: &mut StrictJsonValue, patches: &StrictJsonValue) {
    for patch in array(patches) {
        let operation = text(field(patch, "op"));
        let path = text(field(patch, "path"));
        let mut tokens = pointer_tokens(path);
        let last = tokens
            .pop()
            .expect("patch path must not target implicit root");
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
                        let index = position.expect("replace object field must exist");
                        entries[index].1 = field(patch, "value").clone();
                    }
                    "remove" => {
                        let index = position.expect("remove object field must exist");
                        entries.remove(index);
                    }
                    _ => panic!("unsupported patch operation {operation}"),
                }
            }
            StrictJsonValue::Array(values) => {
                let index = if last == "-" {
                    values.len()
                } else {
                    last.parse::<usize>()
                        .unwrap_or_else(|_| panic!("invalid array patch token {last:?}"))
                };
                match operation {
                    "add" => values.insert(index, field(patch, "value").clone()),
                    "replace" => {
                        *values
                            .get_mut(index)
                            .expect("replace array index must exist") =
                            field(patch, "value").clone();
                    }
                    "remove" => {
                        values.remove(index);
                    }
                    _ => panic!("unsupported patch operation {operation}"),
                }
            }
            _ => panic!("patch parent must be an object or array"),
        }
    }
}
