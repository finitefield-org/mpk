use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, sha256_raw_file_bytes,
    HashDomain, StrictJsonLimits, StrictJsonValue,
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const VECTOR_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(2 * 1024 * 1024, 200_000, 64, 1024 * 1024);
const REGISTRY_TRANSPORT_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    REGISTRY_TRANSPORT_BYTES_MAX as u64,
    REGISTRY_TRANSPORT_BYTES_MAX as u64,
    JSON_NESTING_MAX as u64,
    IDENTIFIER_BYTES_MAX as u64,
);
const ENTRY_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-ENTRY-1.0");
const REGISTRY_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-REGISTRY-1.0");

const REGISTRY_SCHEMA: &str = "mpk.semantic_profile.registry.v1";
const ENTRY_SCHEMA: &str = "mpk.semantic_profile.entry.v1";
const GO_PROFILE: &str = "mpk.go.fixed.v0";
const RUST_PROFILE: &str = "mpk.rust.checked.v0";
const REVISION_1_ROOT_HASH: &str =
    "7c9163571cda32aa47984e3e6d949c8857bf62f00110dd1b2c3958eed5e537cc";

const REGISTRY_CANONICAL_BYTES_MAX: i64 = 524_288;
const REGISTRY_TRANSPORT_BYTES_MAX: i64 = 524_289;
const JSON_NESTING_MAX: i64 = 32;
const IDENTIFIER_BYTES_MAX: i64 = 128;
const SOURCE_LANGUAGE_BYTES_MAX: i64 = 64;
const PROFILES_MAX: i64 = 256;
const SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX: i64 = 65_536;
const SELECTION_CANONICAL_BYTES_MAX: i64 = 65_536;
const COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX: i64 = 1_048_576;
const REVISION_MAX: i64 = 9_007_199_254_740_991;

const ROOT_KEYS: &[&str] = &["schema", "id", "revision", "profiles", "registry_sha256"];
const ENTRY_KEYS: &[&str] = &[
    "schema",
    "source_language",
    "semantic_profile",
    "semantic_parameters_schema",
    "selection_schema",
    "contracts",
    "entry_sha256",
];
const CONTRACT_KEYS: &[&str] = &[
    "ai",
    "evidence",
    "frontend",
    "manifest",
    "policy",
    "release",
    "source_map",
    "vc",
    "vir",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Finding {
    phase: &'static str,
    code: &'static str,
}

#[test]
fn semantic_profile_registry_vectors_are_closed_and_owned() {
    let vectors = load_vectors();
    assert_exact_keys(
        &vectors,
        &[
            "schema",
            "spec_schemas",
            "owner_test",
            "fixtures",
            "transport_cases",
            "hash_cases",
            "registry_cases",
            "context_cases",
            "profile_envelope_cases",
            "limit_cases",
            "hash_domain_migration_cases",
            "migration_cases",
        ],
    );
    assert_eq!(
        text(field(&vectors, "schema")),
        "mpk.semantic_profile.registry.conformance.v1"
    );
    assert_eq!(
        array(field(&vectors, "spec_schemas")),
        [
            Value::String(ENTRY_SCHEMA.to_owned()),
            Value::String(REGISTRY_SCHEMA.to_owned()),
        ]
    );
    assert_eq!(
        text(field(&vectors, "owner_test")),
        "crates/mpk-vc/tests/semantic_profile_registry.rs"
    );
    assert_exact_keys(
        field(&vectors, "fixtures"),
        &[
            "base_registry",
            "go_request",
            "rust_request",
            "go_frontend_profile",
            "rust_frontend_profile",
        ],
    );

    let case_arrays = [
        "transport_cases",
        "hash_cases",
        "registry_cases",
        "context_cases",
        "profile_envelope_cases",
        "limit_cases",
    ];
    let mut ids = BTreeSet::new();
    for array_name in case_arrays {
        for case in array(field(&vectors, array_name)) {
            let id = text(field(case, "id"));
            assert!(ids.insert(id), "duplicate global case ID {id}");
        }
    }
    assert_eq!(ids.len(), 87, "frozen vector case total changed");

    let transport_constructions = array(field(&vectors, "transport_cases"))
        .iter()
        .map(|case| {
            assert_exact_keys(case, &["id", "construction", "expect"]);
            assert_expect_closed(field(case, "expect"));
            text(field(case, "construction"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        transport_constructions,
        BTreeSet::from([
            "canonical_lf",
            "missing_lf",
            "extra_lf",
            "crlf",
            "pretty",
            "escaped_ascii",
            "bom",
            "invalid_utf8",
            "duplicate_name",
            "float",
            "unsafe_integer",
            "depth_33",
            "above_byte_limit",
        ])
    );

    for case in array(field(&vectors, "hash_cases")) {
        let expected = if case.get("expected_transport_utf8_length").is_some() {
            &[
                "id",
                "source_pointer",
                "domain",
                "excluded_field",
                "expected_payload_utf8_length",
                "expected_preimage_length",
                "expected_complete_jcs_utf8_length",
                "expected_transport_utf8_length",
                "expected_transport_sha256",
                "expected_sha256",
            ][..]
        } else {
            &[
                "id",
                "source_pointer",
                "domain",
                "excluded_field",
                "expected_payload_utf8_length",
                "expected_preimage_length",
                "expected_complete_jcs_utf8_length",
                "expected_sha256",
            ][..]
        };
        assert_exact_keys(case, expected);
    }

    let registry_mutations = array(field(&vectors, "registry_cases"))
        .iter()
        .map(|case| {
            assert_exact_keys(
                case,
                &[
                    "id",
                    "mutation",
                    "rehash_entry",
                    "rehash_registry",
                    "expect",
                ],
            );
            assert_expect_closed(field(case, "expect"));
            text(field(case, "mutation"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        registry_mutations,
        BTreeSet::from([
            "none",
            "wrong_schema",
            "missing_profiles",
            "unknown_root_field",
            "entry_callback_field",
            "contract_checker_field",
            "contract_executable_field",
            "contract_plugin_uri_field",
            "malformed_id",
            "zero_revision",
            "too_many_profiles",
            "reverse_profiles",
            "duplicate_profile",
            "bad_entry_hash",
            "unknown_contract",
            "crossed_pair",
            "bad_root_hash",
            "missing_rust_profile",
            "revision_two",
            "pretty_transport",
        ])
    );

    let context_mutations = array(field(&vectors, "context_cases"))
        .iter()
        .map(|case| {
            assert_exact_keys(case, &["id", "fixture", "mutation", "expect"]);
            assert_expect_closed(field(case, "expect"));
            text(field(case, "mutation"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        context_mutations,
        BTreeSet::from([
            "none",
            "registry_and_profile",
            "registry_hash",
            "profile_and_parameters",
            "unknown_profile",
            "crossed_pair",
            "entry_hash",
            "parameters_schema",
            "parameters_unknown_field",
            "selection_schema",
            "selection_unknown_field",
        ])
    );

    let profile_envelope_mutations = array(field(&vectors, "profile_envelope_cases"))
        .iter()
        .map(|case| {
            assert_exact_keys(
                case,
                &["id", "fixture", "contract_field", "mutation", "expect"],
            );
            assert_expect_closed(field(case, "expect"));
            text(field(case, "mutation"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        profile_envelope_mutations,
        BTreeSet::from([
            "none",
            "unknown_envelope_field",
            "nonobject_value",
            "unknown_entry",
            "release_contract",
            "unknown_contract",
            "unknown_value_field",
            "callback_value_field",
        ])
    );

    for case in array(field(&vectors, "limit_cases")) {
        assert_exact_keys(case, &["id", "limit", "value", "expect"]);
    }
    for case in array(field(&vectors, "hash_domain_migration_cases")) {
        assert_exact_keys(case, &["surface", "current", "successor"]);
    }
    for case in array(field(&vectors, "migration_cases")) {
        assert_exact_keys(
            case,
            &[
                "surface",
                "current",
                "successor",
                "successor_accepts_current",
                "current_accepts_successor",
            ],
        );
    }
}

#[test]
fn semantic_profile_registry_transport_vectors_are_exact() {
    let vectors = load_vectors();
    let registry = field(field(&vectors, "fixtures"), "base_registry");
    for case in array(field(&vectors, "transport_cases")) {
        let id = text(field(case, "id"));
        let transport = construct_transport(registry, text(field(case, "construction")));
        assert_expected(
            validate_registry_transport(&transport),
            field(case, "expect"),
            id,
        );
    }
}

#[test]
fn semantic_profile_registry_hash_vectors_are_exact() {
    let vectors = load_vectors();
    let registry = field(field(&vectors, "fixtures"), "base_registry");

    for case in array(field(&vectors, "hash_cases")) {
        let id = text(field(case, "id"));
        let pointer = text(field(case, "source_pointer"));
        let complete = if pointer.is_empty() {
            registry.clone()
        } else {
            registry
                .pointer(pointer)
                .unwrap_or_else(|| panic!("missing hash source pointer {pointer}"))
                .clone()
        };
        let canonical_complete = canonical(&complete);
        assert_eq!(
            canonical_complete.len() as i64,
            integer(field(case, "expected_complete_jcs_utf8_length")),
            "{id}"
        );

        let mut payload = complete;
        object_mut(&mut payload)
            .remove(text(field(case, "excluded_field")))
            .unwrap_or_else(|| panic!("missing excluded self-hash field for {id}"));
        let canonical_payload = canonical(&payload);
        let domain = hash_domain(text(field(case, "domain")));
        assert_eq!(
            canonical_payload.len() as i64,
            integer(field(case, "expected_payload_utf8_length")),
            "{id}"
        );
        assert_eq!(
            domain.as_str().len() as i64 + 1 + canonical_payload.len() as i64,
            integer(field(case, "expected_preimage_length")),
            "{id}"
        );
        assert_eq!(
            hash_canonical_json(domain, &strict_value(&payload))
                .expect("profile registry payload hashes")
                .to_hex(),
            text(field(case, "expected_sha256")),
            "{id}"
        );

        if let Some(expected_length) = case.get("expected_transport_utf8_length") {
            let mut transport = canonical_complete;
            transport.push(b'\n');
            assert_eq!(transport.len() as i64, integer(expected_length), "{id}");
            assert_eq!(
                sha256_raw_file_bytes(&transport).to_hex(),
                text(field(case, "expected_transport_sha256")),
                "{id}"
            );
        }
    }
}

#[test]
fn semantic_profile_registry_validation_and_precedence_match_vectors() {
    let vectors = load_vectors();
    let base = field(field(&vectors, "fixtures"), "base_registry");

    for case in array(field(&vectors, "registry_cases")) {
        let id = text(field(case, "id"));
        let (mut registry, canonical_transport) =
            mutate_registry(base, text(field(case, "mutation")));
        if boolean(field(case, "rehash_entry")) {
            rehash_entry(&mut registry, 0);
        }
        if boolean(field(case, "rehash_registry")) {
            rehash_registry(&mut registry);
        }
        let transport = registry_transport(&registry, canonical_transport);
        assert_expected(
            validate_registry(&registry, &transport),
            field(case, "expect"),
            id,
        );
    }
}

#[test]
fn semantic_context_selection_and_precedence_match_vectors() {
    let vectors = load_vectors();
    let fixtures = field(&vectors, "fixtures");
    let registry = field(fixtures, "base_registry");
    assert_eq!(
        validate_registry(registry, &registry_transport(registry, true)),
        None
    );

    for case in array(field(&vectors, "context_cases")) {
        let id = text(field(case, "id"));
        let fixture = field(fixtures, text(field(case, "fixture")));
        let request = mutate_context(fixture, text(field(case, "mutation")));
        assert_expected(
            validate_request(registry, &request),
            field(case, "expect"),
            id,
        );
    }
}

#[test]
fn compiled_profile_envelopes_match_vectors() {
    let vectors = load_vectors();
    let fixtures = field(&vectors, "fixtures");
    let registry = field(fixtures, "base_registry");
    for case in array(field(&vectors, "profile_envelope_cases")) {
        let id = text(field(case, "id"));
        let envelope = mutate_profile_envelope(
            field(fixtures, text(field(case, "fixture"))),
            text(field(case, "mutation")),
        );
        assert_expected(
            validate_profile_envelope(registry, &envelope, text(field(case, "contract_field"))),
            field(case, "expect"),
            id,
        );
    }
}

#[test]
fn semantic_profile_limits_and_atomic_migration_are_frozen() {
    let vectors = load_vectors();
    let limits = BTreeMap::from([
        ("registry_canonical_bytes", REGISTRY_CANONICAL_BYTES_MAX),
        ("registry_transport_bytes", REGISTRY_TRANSPORT_BYTES_MAX),
        ("json_nesting", JSON_NESTING_MAX),
        ("identifier_bytes", IDENTIFIER_BYTES_MAX),
        ("source_language_bytes", SOURCE_LANGUAGE_BYTES_MAX),
        ("profiles", PROFILES_MAX),
        (
            "semantic_parameters_canonical_bytes",
            SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX,
        ),
        ("selection_canonical_bytes", SELECTION_CANONICAL_BYTES_MAX),
        (
            "compiled_profile_payload_canonical_bytes",
            COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX,
        ),
        ("revision", REVISION_MAX),
    ]);
    let mut seen = BTreeMap::<&str, Vec<i64>>::new();
    for case in array(field(&vectors, "limit_cases")) {
        let id = text(field(case, "id"));
        let limit = text(field(case, "limit"));
        let value = boundary_integer(field(case, "value"));
        let maximum = *limits
            .get(limit)
            .unwrap_or_else(|| panic!("unknown frozen limit {limit}"));
        let actual = if value <= maximum { "accept" } else { "reject" };
        assert_eq!(actual, text(field(case, "expect")), "{id}");
        seen.entry(limit).or_default().push(value);
    }
    assert_eq!(seen.len(), limits.len());
    for (limit, maximum) in limits {
        assert_eq!(
            seen.get(limit),
            Some(&vec![maximum - 1, maximum, maximum + 1]),
            "below/at/above vectors changed for {limit}"
        );
    }

    let expected_domains = [
        ("contract", "MPK-CONTRACT-0.1", "MPK-CONTRACT-1.0"),
        ("vir", "MPK-VIR-0.1", "MPK-VIR-1.0"),
        ("source_map", "MPK-SOURCE-MAP-0.1", "MPK-SOURCE-MAP-1.0"),
        (
            "source_manifest",
            "MPK-SOURCE-MANIFEST-0.1",
            "MPK-SOURCE-MANIFEST-1.0",
        ),
        (
            "release_registry",
            "MPK-BUNDLE-REGISTRY-0.1",
            "MPK-BUNDLE-REGISTRY-1.0",
        ),
        (
            "rust_driver_request",
            "MPK-RUST-DRIVER-REQUEST-0.1",
            "MPK-RUST-DRIVER-REQUEST-1.0",
        ),
        (
            "rust_driver_payload",
            "MPK-RUST-DRIVER-PAYLOAD-0.1",
            "MPK-RUST-DRIVER-PAYLOAD-1.0",
        ),
        ("vc", "MPK-VC-1.0", "MPK-VC-2.0"),
    ];
    let domain_migrations = array(field(&vectors, "hash_domain_migration_cases"));
    assert_eq!(domain_migrations.len(), expected_domains.len());
    for (case, (surface, current, successor)) in domain_migrations.iter().zip(expected_domains) {
        assert_eq!(text(field(case, "surface")), surface);
        assert_eq!(text(field(case, "current")), current, "{surface}");
        assert_eq!(text(field(case, "successor")), successor, "{surface}");
        assert_ne!(current, successor, "{surface}");
    }

    let expected = [
        ("vir", "mpk.vir.v0", "mpk.vir.v1"),
        ("frontend", "mpk.frontend.cli.v0", "mpk.frontend.cli.v1"),
        (
            "rust_driver_request",
            "mpk.rust.driver.request.v0",
            "mpk.rust.driver.request.v1",
        ),
        (
            "rust_driver_result",
            "mpk.rust.driver.v0",
            "mpk.rust.driver.v1",
        ),
        (
            "rust_driver_lowering",
            "mpk.rust.driver.lowering.v0",
            "mpk.rust.driver.lowering.v1",
        ),
        (
            "rust_driver_raw_source_map",
            "mpk.rust.driver.raw_source_map.v0",
            "mpk.rust.driver.raw_source_map.v1",
        ),
        ("source_map", "mpk.source_map.v0", "mpk.source_map.v1"),
        (
            "source_manifest",
            "mpk.source_manifest.v0",
            "mpk.source_manifest.v1",
        ),
        (
            "release_registry",
            "mpk.release.bundle_registry.v0",
            "mpk.release.bundle_registry.v1",
        ),
        (
            "release_registry_id",
            "mpk.release.registry.v0",
            "mpk.release.registry.v1",
        ),
        (
            "frontend_descriptor",
            "mpk.release.frontend_bundle.v0",
            "mpk.release.frontend_bundle.v1",
        ),
        (
            "toolchain_descriptor",
            "mpk.release.toolchain_bundle.v0",
            "mpk.release.toolchain_bundle.v1",
        ),
        (
            "bundle_candidate",
            "mpk.release.bundle_candidate.v0",
            "mpk.release.bundle_candidate.v1",
        ),
        ("vc", "mpk.vc.v1", "mpk.vc.v2"),
        (
            "vc_skeleton",
            "mpk.vc.cert_skeleton.v1",
            "mpk.vc.cert_skeleton.v2",
        ),
        ("policy_scan", "mpk.policy.scan.v1", "mpk.policy.scan.v2"),
        (
            "policy_evidence",
            "mpk.policy.evidence.v1",
            "mpk.policy.evidence.v2",
        ),
        (
            "program_certificate_profile",
            "mpk.program_certificate.alpha.v0",
            "mpk.program_certificate.alpha.v1",
        ),
        ("ai_api", "mpk.ai.api.v1", "mpk.ai.api.v2"),
        (
            "ai_explain_request",
            "mpk.ai.explain.request.v1",
            "mpk.ai.explain.request.v2",
        ),
        (
            "ai_explanation",
            "mpk.ai.explanation.v1",
            "mpk.ai.explanation.v2",
        ),
    ];
    let migrations = array(field(&vectors, "migration_cases"));
    assert_eq!(migrations.len(), expected.len());
    for (case, (surface, current, successor)) in migrations.iter().zip(expected) {
        assert_eq!(text(field(case, "surface")), surface);
        assert_eq!(text(field(case, "current")), current, "{surface}");
        assert_eq!(text(field(case, "successor")), successor, "{surface}");
        assert!(
            !boolean(field(case, "successor_accepts_current")),
            "{surface}"
        );
        assert!(
            !boolean(field(case, "current_accepts_successor")),
            "{surface}"
        );
    }
}

fn validate_registry(registry: &Value, transport: &[u8]) -> Option<Finding> {
    if transport.len() as i64 > REGISTRY_TRANSPORT_BYTES_MAX {
        return reject("transport", "SEMANTIC_REGISTRY_TRANSPORT");
    }
    if !has_exact_keys(registry, ROOT_KEYS)
        || registry.get("schema").and_then(Value::as_str) != Some(REGISTRY_SCHEMA)
        || registry.get("profiles").and_then(Value::as_array).is_none()
        || array(field(registry, "profiles")).is_empty()
    {
        return reject("shape", "SEMANTIC_REGISTRY_SHAPE");
    }
    for entry in array(field(registry, "profiles")) {
        if !has_exact_keys(entry, ENTRY_KEYS)
            || entry.get("schema").and_then(Value::as_str) != Some(ENTRY_SCHEMA)
            || entry.get("contracts").and_then(Value::as_object).is_none()
            || !has_exact_keys(field(entry, "contracts"), CONTRACT_KEYS)
        {
            return reject("shape", "SEMANTIC_REGISTRY_SHAPE");
        }
    }

    let id = registry.get("id").and_then(Value::as_str);
    let revision = registry.get("revision").and_then(Value::as_i64);
    let root_hash = registry.get("registry_sha256").and_then(Value::as_str);
    if id != Some(REGISTRY_SCHEMA)
        || !id.is_some_and(|value| valid_id(value, IDENTIFIER_BYTES_MAX))
        || !revision.is_some_and(|value| (1..=REVISION_MAX).contains(&value))
        || !root_hash.is_some_and(valid_hash)
    {
        return reject("scalar", "SEMANTIC_REGISTRY_SCALAR");
    }
    for entry in array(field(registry, "profiles")) {
        let language = entry.get("source_language").and_then(Value::as_str);
        let profile = entry.get("semantic_profile").and_then(Value::as_str);
        let parameter_schema = entry
            .get("semantic_parameters_schema")
            .and_then(Value::as_str);
        let selection_schema = entry.get("selection_schema").and_then(Value::as_str);
        let entry_hash = entry.get("entry_sha256").and_then(Value::as_str);
        if !language.is_some_and(|value| valid_id(value, SOURCE_LANGUAGE_BYTES_MAX))
            || !profile.is_some_and(|value| valid_id(value, IDENTIFIER_BYTES_MAX))
            || !parameter_schema.is_some_and(|value| valid_id(value, IDENTIFIER_BYTES_MAX))
            || !selection_schema.is_some_and(|value| valid_id(value, IDENTIFIER_BYTES_MAX))
            || !entry_hash.is_some_and(valid_hash)
        {
            return reject("scalar", "SEMANTIC_REGISTRY_SCALAR");
        }
        for contract in object(field(entry, "contracts")).values() {
            if !contract
                .as_str()
                .is_some_and(|value| valid_id(value, IDENTIFIER_BYTES_MAX))
            {
                return reject("scalar", "SEMANTIC_REGISTRY_SCALAR");
            }
        }
    }

    let canonical_size = canonical(registry).len() as i64;
    if canonical_size > REGISTRY_CANONICAL_BYTES_MAX
        || canonical_size + 1 > REGISTRY_TRANSPORT_BYTES_MAX
        || array(field(registry, "profiles")).len() as i64 > PROFILES_MAX
    {
        return reject("limits", "SEMANTIC_REGISTRY_LIMIT");
    }

    let mut prior: Option<(&str, &str)> = None;
    let mut profiles = BTreeSet::new();
    for entry in array(field(registry, "profiles")) {
        let pair = (
            text(field(entry, "source_language")),
            text(field(entry, "semantic_profile")),
        );
        if prior.is_some_and(|previous| previous >= pair) || !profiles.insert(pair.1) {
            return reject("order", "SEMANTIC_REGISTRY_ORDER");
        }
        prior = Some(pair);
    }

    for entry in array(field(registry, "profiles")) {
        if hash_without(entry, "entry_sha256", ENTRY_HASH_DOMAIN)
            != text(field(entry, "entry_sha256"))
        {
            return reject("entry_hash", "SEMANTIC_REGISTRY_ENTRY_HASH");
        }
    }

    let compiled_contracts = compiled_contracts();
    for entry in array(field(registry, "profiles")) {
        for contract in object(field(entry, "contracts")).values() {
            if !compiled_contracts.contains(text(contract)) {
                return reject("contract_binding", "SEMANTIC_REGISTRY_CONTRACT");
            }
        }
    }

    for entry in array(field(registry, "profiles")) {
        if !entry_invariants_hold(entry) {
            return reject("invariant", "SEMANTIC_REGISTRY_INVARIANT");
        }
    }

    if hash_without(registry, "registry_sha256", REGISTRY_HASH_DOMAIN)
        != text(field(registry, "registry_sha256"))
    {
        return reject("registry_hash", "SEMANTIC_REGISTRY_HASH");
    }
    if revision != Some(1) || root_hash != Some(REVISION_1_ROOT_HASH) {
        return reject("embedded_identity", "SEMANTIC_REGISTRY_ASSERTION");
    }
    if transport != registry_transport(registry, true) {
        return reject("canonical_transport", "SEMANTIC_REGISTRY_CANONICAL");
    }
    None
}

fn validate_registry_transport(transport: &[u8]) -> Option<Finding> {
    if transport.len() as i64 > REGISTRY_TRANSPORT_BYTES_MAX {
        return reject("transport", "SEMANTIC_REGISTRY_TRANSPORT");
    }
    if parse_strict_json(transport, REGISTRY_TRANSPORT_LIMITS).is_err() {
        return reject("transport", "SEMANTIC_REGISTRY_TRANSPORT");
    }
    let Ok(registry) = serde_json::from_slice::<Value>(transport) else {
        return reject("transport", "SEMANTIC_REGISTRY_TRANSPORT");
    };
    validate_registry(&registry, transport)
}

fn validate_request(registry: &Value, request: &Value) -> Option<Finding> {
    if !has_exact_keys(request, &["semantic_context", "selection"])
        || !has_exact_keys(
            field(request, "semantic_context"),
            &[
                "profile_registry",
                "profile_entry_sha256",
                "source_language",
                "semantic_profile",
                "semantic_parameters",
            ],
        )
        || !has_exact_keys(
            field(field(request, "semantic_context"), "profile_registry"),
            &["schema", "id", "revision", "registry_sha256"],
        )
        || !has_exact_keys(
            field(field(request, "semantic_context"), "semantic_parameters"),
            &["schema", "value"],
        )
        || !has_exact_keys(field(request, "selection"), &["schema", "value"])
    {
        return reject("shape", "SEMANTIC_REGISTRY_SHAPE");
    }

    let context = field(request, "semantic_context");
    let expected_identity = registry_identity(registry);
    if field(context, "profile_registry") != &expected_identity {
        return reject("registry_identity", "SEMANTIC_REGISTRY_ASSERTION");
    }

    let language = text(field(context, "source_language"));
    let profile = text(field(context, "semantic_profile"));
    let Some(entry) = array(field(registry, "profiles")).iter().find(|entry| {
        entry.get("source_language").and_then(Value::as_str) == Some(language)
            && entry.get("semantic_profile").and_then(Value::as_str) == Some(profile)
    }) else {
        return reject("profile_lookup", "SEMANTIC_PROFILE_UNKNOWN");
    };

    if field(context, "profile_entry_sha256") != field(entry, "entry_sha256") {
        return reject("profile_entry", "SEMANTIC_PROFILE_ENTRY");
    }

    let parameters = field(context, "semantic_parameters");
    if field(parameters, "schema") != field(entry, "semantic_parameters_schema") {
        return reject("parameters_schema", "SEMANTIC_PARAMETERS_SCHEMA");
    }
    if canonical(parameters).len() as i64 > SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX
        || !parameters_valid(profile, field(parameters, "value"))
    {
        return reject("parameters_value", "SEMANTIC_PARAMETERS_INVALID");
    }

    let selection = field(request, "selection");
    if field(selection, "schema") != field(entry, "selection_schema") {
        return reject("selection_schema", "SEMANTIC_SELECTION_SCHEMA");
    }
    if canonical(selection).len() as i64 > SELECTION_CANONICAL_BYTES_MAX
        || !selection_valid(profile, field(selection, "value"))
    {
        return reject("selection_value", "SEMANTIC_SELECTION_INVALID");
    }
    None
}

fn validate_profile_envelope(
    registry: &Value,
    envelope: &Value,
    contract_field: &str,
) -> Option<Finding> {
    if !has_exact_keys(envelope, &["profile_entry_sha256", "contract_id", "value"])
        || envelope
            .get("profile_entry_sha256")
            .and_then(Value::as_str)
            .is_none()
        || envelope
            .get("contract_id")
            .and_then(Value::as_str)
            .is_none()
        || envelope.get("value").and_then(Value::as_object).is_none()
    {
        return reject("profile_envelope", "SEMANTIC_PROFILE_ENVELOPE");
    }

    let entry_hash = text(field(envelope, "profile_entry_sha256"));
    let Some(entry) = array(field(registry, "profiles"))
        .iter()
        .find(|entry| text(field(entry, "entry_sha256")) == entry_hash)
    else {
        return reject("profile_entry", "SEMANTIC_PROFILE_ENTRY");
    };
    let expected_contract = field(field(entry, "contracts"), contract_field);
    if field(envelope, "contract_id") != expected_contract {
        return reject("profile_contract", "SEMANTIC_PROFILE_CONTRACT");
    }
    if canonical(envelope).len() as i64 > COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX
        || contract_field != "frontend"
        || !frontend_profile_value_valid(
            text(field(entry, "semantic_profile")),
            field(envelope, "value"),
        )
    {
        return reject("profile_payload", "SEMANTIC_PROFILE_PAYLOAD");
    }
    None
}

fn entry_invariants_hold(entry: &Value) -> bool {
    let language = text(field(entry, "source_language"));
    let profile = text(field(entry, "semantic_profile"));
    let (expected_language, stem, parameters, selection) = match profile {
        GO_PROFILE => (
            "go",
            "go_fixed",
            "mpk.semantic_parameters.go_fixed.v0",
            "mpk.selection.go_function.v0",
        ),
        RUST_PROFILE => (
            "rust",
            "rust_checked",
            "mpk.semantic_parameters.rust_checked.v0",
            "mpk.selection.rust_function.v0",
        ),
        _ => return false,
    };
    if language != expected_language
        || text(field(entry, "semantic_parameters_schema")) != parameters
        || text(field(entry, "selection_schema")) != selection
    {
        return false;
    }
    CONTRACT_KEYS.iter().all(|contract| {
        text(field(field(entry, "contracts"), contract))
            == format!("mpk.profile.{contract}.{stem}.v0")
    })
}

fn parameters_valid(profile: &str, value: &Value) -> bool {
    match profile {
        GO_PROFILE => {
            has_exact_keys(value, &["target_id", "pointer_width"])
                && value.get("target_id").and_then(Value::as_str) == Some("linux/amd64")
                && value.get("pointer_width").and_then(Value::as_i64) == Some(64)
        }
        RUST_PROFILE => {
            has_exact_keys(
                value,
                &["target_id", "pointer_width", "overflow_mode", "panic_mode"],
            ) && value.get("target_id").and_then(Value::as_str) == Some("x86_64-unknown-linux-gnu")
                && value.get("pointer_width").and_then(Value::as_i64) == Some(64)
                && value.get("overflow_mode").and_then(Value::as_str) == Some("checked")
                && value.get("panic_mode").and_then(Value::as_str) == Some("abort")
        }
        _ => false,
    }
}

fn selection_valid(profile: &str, value: &Value) -> bool {
    match profile {
        GO_PROFILE => {
            if !has_exact_keys(value, &["package", "function"]) {
                return false;
            }
            let Some(package) = value.get("package").and_then(Value::as_str) else {
                return false;
            };
            let Some(function) = value.get("function").and_then(Value::as_str) else {
                return false;
            };
            !package.is_empty()
                && function
                    .strip_prefix(package)
                    .is_some_and(|suffix| suffix.starts_with('.') && suffix.len() > 1)
        }
        RUST_PROFILE => {
            if !has_exact_keys(value, &["package", "crate", "kind", "function"]) {
                return false;
            }
            let Some(package) = value.get("package").and_then(Value::as_str) else {
                return false;
            };
            let Some(crate_name) = value.get("crate").and_then(Value::as_str) else {
                return false;
            };
            let Some(kind) = value.get("kind").and_then(Value::as_str) else {
                return false;
            };
            let Some(function) = value.get("function").and_then(Value::as_str) else {
                return false;
            };
            !package.is_empty()
                && !crate_name.is_empty()
                && matches!(kind, "lib" | "bin")
                && function
                    .strip_prefix(crate_name)
                    .is_some_and(|suffix| suffix.starts_with("::") && suffix.len() > 2)
        }
        _ => false,
    }
}

fn frontend_profile_value_valid(profile: &str, value: &Value) -> bool {
    if !has_exact_keys(
        value,
        &[
            "limit_profile_id",
            "environment_profile_id",
            "argument_profile_id",
        ],
    ) || value.get("limit_profile_id").and_then(Value::as_str) != Some("mpk.vir.limits.v0")
    {
        return false;
    }
    match profile {
        GO_PROFILE => {
            value.get("environment_profile_id").and_then(Value::as_str)
                == Some("mpk.go.frontend_environment.v0")
                && value.get("argument_profile_id").and_then(Value::as_str)
                    == Some("mpk.go.frontend_arguments.v0")
        }
        RUST_PROFILE => {
            value.get("environment_profile_id").and_then(Value::as_str)
                == Some("mpk.rust.frontend_environment.v0")
                && value.get("argument_profile_id").and_then(Value::as_str)
                    == Some("mpk.rust.frontend_arguments.v0")
        }
        _ => false,
    }
}

fn mutate_registry(base: &Value, mutation: &str) -> (Value, bool) {
    let mut registry = base.clone();
    let mut canonical_transport = true;
    match mutation {
        "none" => {}
        "wrong_schema" => registry["schema"] = Value::String("mpk.semantic.registry.v1".into()),
        "missing_profiles" => {
            object_mut(&mut registry).remove("profiles");
        }
        "unknown_root_field" => {
            registry["validator_path"] = Value::String("/tmp/validator".into());
        }
        "entry_callback_field" => {
            registry["profiles"][0]["callback"] = Value::String("validate".into());
        }
        "contract_checker_field" => {
            registry["profiles"][0]["contracts"]["checker"] =
                Value::String("mpk.profile.checker.go_fixed.v0".into());
        }
        "contract_executable_field" => {
            registry["profiles"][0]["contracts"]["executable"] =
                Value::String("bin/validator".into());
        }
        "contract_plugin_uri_field" => {
            registry["profiles"][0]["contracts"]["plugin_uri"] =
                Value::String("file:///tmp/plugin.wasm".into());
        }
        "malformed_id" => registry["id"] = Value::String("MPK bad".into()),
        "zero_revision" => registry["revision"] = Value::from(0),
        "too_many_profiles" => {
            let template = registry["profiles"][0].clone();
            let profiles = registry["profiles"]
                .as_array_mut()
                .expect("profiles fixture is an array");
            while profiles.len() <= PROFILES_MAX as usize {
                profiles.push(template.clone());
            }
        }
        "reverse_profiles" => registry["profiles"]
            .as_array_mut()
            .expect("profiles fixture is an array")
            .reverse(),
        "duplicate_profile" => {
            registry["profiles"][1]["semantic_profile"] = Value::String(GO_PROFILE.to_owned());
        }
        "bad_entry_hash" => {
            registry["profiles"][0]["entry_sha256"] = Value::String("0".repeat(64));
        }
        "unknown_contract" => {
            registry["profiles"][0]["contracts"]["ai"] =
                Value::String("mpk.profile.ai.unknown.v0".into());
        }
        "crossed_pair" => {
            registry["profiles"][0]["source_language"] = Value::String("rust".into());
        }
        "bad_root_hash" => registry["registry_sha256"] = Value::String("0".repeat(64)),
        "missing_rust_profile" => {
            registry["profiles"]
                .as_array_mut()
                .expect("profiles fixture is an array")
                .pop();
        }
        "revision_two" => registry["revision"] = Value::from(2),
        "pretty_transport" => canonical_transport = false,
        _ => panic!("unknown registry mutation {mutation}"),
    }
    (registry, canonical_transport)
}

fn construct_transport(registry: &Value, construction: &str) -> Vec<u8> {
    let canonical = registry_transport(registry, true);
    match construction {
        "canonical_lf" => canonical,
        "missing_lf" => canonical[..canonical.len() - 1].to_vec(),
        "extra_lf" => {
            let mut transport = canonical;
            transport.push(b'\n');
            transport
        }
        "crlf" => {
            let mut transport = canonical[..canonical.len() - 1].to_vec();
            transport.extend_from_slice(b"\r\n");
            transport
        }
        "pretty" => registry_transport(registry, false),
        "escaped_ascii" => String::from_utf8(canonical)
            .expect("canonical registry is UTF-8")
            .replacen(
                "mpk.semantic_profile.entry.v1",
                "\\u006dpk.semantic_profile.entry.v1",
                1,
            )
            .into_bytes(),
        "bom" => {
            let mut transport = vec![0xef, 0xbb, 0xbf];
            transport.extend_from_slice(&canonical);
            transport
        }
        "invalid_utf8" => vec![0xff, b'\n'],
        "duplicate_name" => b"{\"schema\":\"mpk.semantic_profile.registry.v1\",\"schema\":\"mpk.semantic_profile.registry.v1\"}\n".to_vec(),
        "float" => b"{\"revision\":1.5}\n".to_vec(),
        "unsafe_integer" => b"{\"revision\":9007199254740992}\n".to_vec(),
        "depth_33" => {
            let mut transport = vec![b'['; (JSON_NESTING_MAX + 1) as usize];
            transport.push(b'0');
            transport.extend(vec![b']'; (JSON_NESTING_MAX + 1) as usize]);
            transport.push(b'\n');
            transport
        }
        "above_byte_limit" => vec![b' '; (REGISTRY_TRANSPORT_BYTES_MAX + 1) as usize],
        _ => panic!("unknown transport construction {construction}"),
    }
}

fn mutate_context(base: &Value, mutation: &str) -> Value {
    let mut request = base.clone();
    match mutation {
        "none" => {}
        "registry_and_profile" => {
            request["semantic_context"]["profile_registry"]["registry_sha256"] =
                Value::String("0".repeat(64));
            request["semantic_context"]["semantic_profile"] =
                Value::String("mpk.unknown.v0".into());
        }
        "registry_hash" => {
            request["semantic_context"]["profile_registry"]["registry_sha256"] =
                Value::String("0".repeat(64));
        }
        "profile_and_parameters" => {
            request["semantic_context"]["semantic_profile"] =
                Value::String("mpk.unknown.v0".into());
            request["semantic_context"]["semantic_parameters"]["value"]["unknown"] =
                Value::Bool(true);
        }
        "unknown_profile" => {
            request["semantic_context"]["semantic_profile"] =
                Value::String("mpk.unknown.v0".into());
        }
        "crossed_pair" => {
            request["semantic_context"]["source_language"] = Value::String("rust".into());
        }
        "entry_hash" => {
            request["semantic_context"]["profile_entry_sha256"] = Value::String("0".repeat(64));
        }
        "parameters_schema" => {
            request["semantic_context"]["semantic_parameters"]["schema"] =
                Value::String("mpk.semantic_parameters.rust_checked.v0".into());
        }
        "parameters_unknown_field" => {
            request["semantic_context"]["semantic_parameters"]["value"]["unknown"] =
                Value::Bool(true);
        }
        "selection_schema" => {
            request["selection"]["schema"] = Value::String("mpk.selection.rust_function.v0".into());
        }
        "selection_unknown_field" => {
            request["selection"]["value"]["unknown"] = Value::Bool(true);
        }
        _ => panic!("unknown context mutation {mutation}"),
    }
    request
}

fn mutate_profile_envelope(base: &Value, mutation: &str) -> Value {
    let mut envelope = base.clone();
    match mutation {
        "none" => {}
        "unknown_envelope_field" => envelope["validator"] = Value::String("dynamic".into()),
        "nonobject_value" => envelope["value"] = Value::String("dynamic".into()),
        "unknown_entry" => {
            envelope["profile_entry_sha256"] = Value::String("0".repeat(64));
        }
        "release_contract" => {
            envelope["contract_id"] = Value::String("mpk.profile.release.go_fixed.v0".into());
        }
        "unknown_contract" => {
            envelope["contract_id"] = Value::String("mpk.profile.frontend.unknown.v0".into());
        }
        "unknown_value_field" => envelope["value"]["unknown"] = Value::Bool(true),
        "callback_value_field" => {
            envelope["value"]["validator_callback"] = Value::String("run".into());
        }
        _ => panic!("unknown profile envelope mutation {mutation}"),
    }
    envelope
}

fn registry_identity(registry: &Value) -> Value {
    let mut identity = Map::new();
    for key in ["schema", "id", "revision", "registry_sha256"] {
        identity.insert(key.to_owned(), field(registry, key).clone());
    }
    Value::Object(identity)
}

fn compiled_contracts() -> BTreeSet<String> {
    ["go_fixed", "rust_checked"]
        .into_iter()
        .flat_map(|stem| {
            CONTRACT_KEYS
                .iter()
                .map(move |contract| format!("mpk.profile.{contract}.{stem}.v0"))
        })
        .collect()
}

fn valid_id(value: &str, maximum: i64) -> bool {
    if value.is_empty() || value.len() as i64 > maximum || !value.is_ascii() {
        return false;
    }
    let mut needs_alphanumeric = true;
    for byte in value.bytes() {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            needs_alphanumeric = false;
        } else if matches!(byte, b'.' | b'_' | b'-') && !needs_alphanumeric {
            needs_alphanumeric = true;
        } else {
            return false;
        }
    }
    !needs_alphanumeric
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn rehash_entry(registry: &mut Value, index: usize) {
    let digest = hash_without(
        &registry["profiles"][index],
        "entry_sha256",
        ENTRY_HASH_DOMAIN,
    );
    registry["profiles"][index]["entry_sha256"] = Value::String(digest);
}

fn rehash_registry(registry: &mut Value) {
    let digest = hash_without(registry, "registry_sha256", REGISTRY_HASH_DOMAIN);
    registry["registry_sha256"] = Value::String(digest);
}

fn hash_without(value: &Value, excluded_field: &str, domain: HashDomain) -> String {
    let mut payload = value.clone();
    object_mut(&mut payload)
        .remove(excluded_field)
        .unwrap_or_else(|| panic!("missing self-hash field {excluded_field}"));
    hash_canonical_json(domain, &strict_value(&payload))
        .expect("semantic profile payload hashes")
        .to_hex()
}

fn hash_domain(value: &str) -> HashDomain {
    match value {
        "MPK-SEMANTIC-PROFILE-ENTRY-1.0" => ENTRY_HASH_DOMAIN,
        "MPK-SEMANTIC-PROFILE-REGISTRY-1.0" => REGISTRY_HASH_DOMAIN,
        _ => panic!("unknown semantic profile hash domain {value}"),
    }
}

fn registry_transport(registry: &Value, canonical_transport: bool) -> Vec<u8> {
    let mut transport = if canonical_transport {
        canonical(registry)
    } else {
        serde_json::to_vec_pretty(registry).expect("pretty registry transport serializes")
    };
    transport.push(b'\n');
    transport
}

fn reject(phase: &'static str, code: &'static str) -> Option<Finding> {
    Some(Finding { phase, code })
}

fn assert_expected(actual: Option<Finding>, expected: &Value, id: &str) {
    match text(field(expected, "outcome")) {
        "accept" => assert_eq!(actual, None, "{id}"),
        "reject" => {
            let actual = actual.unwrap_or_else(|| panic!("{id} unexpectedly accepted"));
            assert_eq!(actual.phase, text(field(expected, "phase")), "{id}");
            assert_eq!(actual.code, text(field(expected, "code")), "{id}");
        }
        outcome => panic!("unknown expected outcome {outcome} for {id}"),
    }
}

fn assert_expect_closed(expect: &Value) {
    match text(field(expect, "outcome")) {
        "accept" => {
            assert_exact_keys(expect, &["outcome", "phase", "code"]);
            assert_eq!(text(field(expect, "phase")), "complete");
            assert_eq!(text(field(expect, "code")), "");
        }
        "reject" => assert_exact_keys(expect, &["outcome", "phase", "code"]),
        outcome => panic!("unknown expectation outcome {outcome}"),
    }
}

fn load_vectors() -> Value {
    parse_strict_json(VECTOR_BYTES, TEST_LIMITS).expect("strict semantic profile vectors parse");
    serde_json::from_slice(VECTOR_BYTES).expect("semantic profile vector container parses")
}

fn strict_value(value: &Value) -> StrictJsonValue {
    let bytes = serde_json::to_vec(value).expect("serialize semantic profile vector value");
    parse_strict_json(&bytes, TEST_LIMITS).expect("strict semantic profile value parses")
}

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&strict_value(value)).expect("semantic profile value canonicalizes")
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name:?}"))
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("value is an array")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("value is an object")
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("value is an object")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("value is a string")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().expect("value is an integer")
}

fn boundary_integer(value: &Value) -> i64 {
    value.as_i64().unwrap_or_else(|| {
        text(value)
            .parse()
            .expect("decimal boundary value fits the test model")
    })
}

fn boolean(value: &Value) -> bool {
    value.as_bool().expect("value is a Boolean")
}

fn has_exact_keys(value: &Value, expected: &[&str]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    actual == expected
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    assert!(has_exact_keys(value, expected), "closed object keys differ");
}
