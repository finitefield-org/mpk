use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, classify_semantic_registry_failure, semantic_profile_entry_hash,
    semantic_profile_registry_hash, validate_compiled_profile_envelope,
    validate_registry_selection_envelope, validate_registry_semantic_context,
    validate_semantic_context_linkage, validate_semantic_profile_registry,
    validate_semantic_registry_limit, validate_semantic_request, ProfileContractField,
    RegistryRevision, SemanticRegistryFailureDisposition, SemanticRegistryFailureSurface,
    SemanticRegistryLimit, SemanticRegistryValidationError, ValidatedSemanticProfileRegistry,
    COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX, REGISTRY_CANONICAL_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX, SELECTION_CANONICAL_BYTES_MAX,
    SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX,
    SEMANTIC_REGISTRY_JSON_NESTING_MAX, SEMANTIC_REGISTRY_PROFILES_MAX,
    SEMANTIC_REGISTRY_REVISION_MAX, SOURCE_LANGUAGE_BYTES_MAX,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, sha256_raw_file_bytes, StrictJsonLimits,
    StrictJsonValue,
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

const REGISTRY_V1_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");

const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(4 * 1024 * 1024, 1_000_000, 128, 2 * 1024 * 1024);

#[test]
fn revision_one_transport_hash_and_registry_vectors_execute_runtime_validation() {
    let vectors = load(REGISTRY_V1_BYTES, "revision-1 vectors");
    assert_exact_fields(
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
        text(field(&vectors, "owner_test")),
        "crates/mpk-vc/tests/semantic_profile_registry.rs"
    );
    assert_eq!(
        array(field(&vectors, "spec_schemas")),
        [
            Value::String("mpk.semantic_profile.entry.v1".to_owned()),
            Value::String("mpk.semantic_profile.registry.v1".to_owned()),
        ]
    );
    let fixtures = field(&vectors, "fixtures");
    assert_exact_fields(
        fixtures,
        &[
            "base_registry",
            "go_request",
            "rust_request",
            "go_frontend_profile",
            "rust_frontend_profile",
        ],
    );
    let registry = field(fixtures, "base_registry");

    for case in array(field(&vectors, "transport_cases")) {
        let id = text(field(case, "id"));
        let transport = construct_transport(registry, text(field(case, "construction")));
        assert_expected(
            validate_semantic_profile_registry(&transport, RegistryRevision::Revision1),
            field(case, "expect"),
            id,
        );
    }

    for case in array(field(&vectors, "hash_cases")) {
        let id = text(field(case, "id"));
        let pointer = text(field(case, "source_pointer"));
        let complete = if pointer.is_empty() {
            registry
        } else {
            registry
                .pointer(pointer)
                .unwrap_or_else(|| panic!("missing hash pointer {pointer}"))
        };
        let canonical = canonical(complete);
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "expected_complete_jcs_utf8_length")),
            "{id}"
        );
        let digest = match text(field(case, "excluded_field")) {
            "entry_sha256" => semantic_profile_entry_hash(complete),
            "registry_sha256" => semantic_profile_registry_hash(complete),
            other => panic!("unknown excluded hash field {other}"),
        }
        .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        if let Some(expected) = case.get("expected_transport_sha256") {
            let transport = canonical_registry_transport(complete).expect("canonical transport");
            assert_eq!(
                sha256_raw_file_bytes(&transport).to_hex(),
                text(expected),
                "{id}"
            );
        }
    }

    for case in array(field(&vectors, "registry_cases")) {
        let id = text(field(case, "id"));
        let (mut registry, canonical_transport) =
            mutate_registry(registry, text(field(case, "mutation")));
        if boolean(field(case, "rehash_entry")) {
            rehash_entry(&mut registry, 0);
        }
        if boolean(field(case, "rehash_registry")) {
            rehash_registry(&mut registry);
        }
        let transport = registry_transport(&registry, canonical_transport);
        assert_expected(
            validate_semantic_profile_registry(&transport, RegistryRevision::Revision1),
            field(case, "expect"),
            id,
        );
    }
}

#[test]
fn context_profile_envelope_and_limit_vectors_execute_runtime_dispatch() {
    let vectors = load(REGISTRY_V1_BYTES, "revision-1 vectors");
    let fixtures = field(&vectors, "fixtures");
    let registry = validate_registry_fixture(
        field(fixtures, "base_registry"),
        RegistryRevision::Revision1,
    );

    for case in array(field(&vectors, "context_cases")) {
        let id = text(field(case, "id"));
        let fixture = field(fixtures, text(field(case, "fixture")));
        let request = mutate_context(fixture, text(field(case, "mutation")));
        assert_expected(
            validate_semantic_request(&registry, &request),
            field(case, "expect"),
            id,
        );
    }

    for case in array(field(&vectors, "profile_envelope_cases")) {
        let id = text(field(case, "id"));
        let fixture = field(fixtures, text(field(case, "fixture")));
        let envelope = mutate_profile_envelope(fixture, text(field(case, "mutation")));
        let contract_field = ProfileContractField::from_name(text(field(case, "contract_field")))
            .expect("known profile contract field");
        assert_expected(
            validate_compiled_profile_envelope(&registry, &envelope, contract_field),
            field(case, "expect"),
            id,
        );
    }

    let go_request = field(fixtures, "go_request");
    let rust_request = field(fixtures, "rust_request");
    let go_context =
        validate_registry_semantic_context(&registry, field(go_request, "semantic_context"))
            .expect("standalone Go context");
    let rust_context =
        validate_registry_semantic_context(&registry, field(rust_request, "semantic_context"))
            .expect("standalone Rust context");
    validate_registry_selection_envelope(&registry, &go_context, field(go_request, "selection"))
        .expect("standalone Go selection");
    validate_semantic_context_linkage(&go_context, &go_context).expect("identical linkage");
    let linkage_error = validate_semantic_context_linkage(&go_context, &rust_context)
        .expect_err("different contexts reject linkage");
    assert_eq!(linkage_error.code().as_str(), "SEMANTIC_CONTEXT_LINKAGE");

    let caller_error =
        validate_semantic_request(&registry, &mutate_context(go_request, "unknown_profile"))
            .expect_err("unknown caller profile");
    for (surface, disposition) in [
        (
            SemanticRegistryFailureSurface::CallerConfiguration,
            SemanticRegistryFailureDisposition::PrelaunchConfiguration,
        ),
        (
            SemanticRegistryFailureSurface::InstalledRegistry,
            SemanticRegistryFailureDisposition::ReleaseFrontendError,
        ),
        (
            SemanticRegistryFailureSurface::LaunchedChildContext,
            SemanticRegistryFailureDisposition::ChildFrontendError,
        ),
        (
            SemanticRegistryFailureSurface::ImportedArtifact,
            SemanticRegistryFailureDisposition::InvalidArtifact,
        ),
    ] {
        let actual = classify_semantic_registry_failure(&caller_error, surface);
        assert_eq!(actual, disposition);
        assert!(!actual.may_publish_artifact());
        assert!(!actual.may_become_ready_or_verified());
    }
    assert_eq!(
        SemanticRegistryFailureDisposition::PrelaunchConfiguration.exit_code(),
        Some(2)
    );
    assert_eq!(
        SemanticRegistryFailureDisposition::ReleaseFrontendError.status(),
        Some("frontend-error")
    );
    assert_eq!(
        SemanticRegistryFailureDisposition::ChildFrontendError.status(),
        Some("frontend-error")
    );
    assert!(SemanticRegistryFailureDisposition::ChildFrontendError.child_started());

    let expected_limits = [
        ("registry_canonical_bytes", REGISTRY_CANONICAL_BYTES_MAX),
        ("registry_transport_bytes", REGISTRY_TRANSPORT_BYTES_MAX),
        ("json_nesting", SEMANTIC_REGISTRY_JSON_NESTING_MAX),
        ("identifier_bytes", SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX),
        ("source_language_bytes", SOURCE_LANGUAGE_BYTES_MAX),
        ("profiles", SEMANTIC_REGISTRY_PROFILES_MAX),
        (
            "semantic_parameters_canonical_bytes",
            SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX,
        ),
        ("selection_canonical_bytes", SELECTION_CANONICAL_BYTES_MAX),
        (
            "compiled_profile_payload_canonical_bytes",
            COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX,
        ),
        ("revision", SEMANTIC_REGISTRY_REVISION_MAX),
    ];
    for (id, maximum) in expected_limits {
        let limit = SemanticRegistryLimit::from_id(id).expect("known registry limit");
        assert_eq!(limit.inclusive_maximum(), maximum, "{id}");
    }
    for case in array(field(&vectors, "limit_cases")) {
        let id = text(field(case, "id"));
        let limit_id = text(field(case, "limit"));
        let limit = SemanticRegistryLimit::from_id(limit_id).expect("known registry limit");
        let value = boundary_integer(field(case, "value"));
        let accepted = validate_semantic_registry_limit(limit, value).is_ok();
        assert_eq!(accepted, text(field(case, "expect")) == "accept", "{id}");
    }
}

fn validate_registry_fixture(
    registry: &Value,
    revision: RegistryRevision,
) -> ValidatedSemanticProfileRegistry {
    let transport = canonical_registry_transport(registry).expect("canonical registry transport");
    validate_semantic_profile_registry(&transport, revision)
        .unwrap_or_else(|error| panic!("frozen registry validates: {error}"))
}

fn assert_expected<T>(
    result: Result<T, SemanticRegistryValidationError>,
    expected: &Value,
    id: &str,
) {
    match text(field(expected, "outcome")) {
        "accept" => assert!(result.is_ok(), "{id}: {:?}", result.err()),
        "reject" => {
            let error = result.unwrap_err_or_else(id);
            assert_eq!(
                error.phase().as_str(),
                text(field(expected, "phase")),
                "{id}"
            );
            assert_eq!(error.code().as_str(), text(field(expected, "code")), "{id}");
        }
        outcome => panic!("unknown expected outcome {outcome} for {id}"),
    }
}

trait ResultErrorExt<T> {
    fn unwrap_err_or_else(self, id: &str) -> SemanticRegistryValidationError;
}

impl<T> ResultErrorExt<T> for Result<T, SemanticRegistryValidationError> {
    fn unwrap_err_or_else(self, id: &str) -> SemanticRegistryValidationError {
        match self {
            Ok(_) => panic!("{id} unexpectedly accepted"),
            Err(error) => error,
        }
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
            let profiles = registry["profiles"].as_array_mut().expect("profiles array");
            while profiles.len() <= SEMANTIC_REGISTRY_PROFILES_MAX as usize {
                profiles.push(template.clone());
            }
        }
        "reverse_profiles" => registry["profiles"].as_array_mut().unwrap().reverse(),
        "duplicate_profile" => {
            registry["profiles"][1]["semantic_profile"] = Value::String("mpk.go.fixed.v0".into());
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
            registry["profiles"].as_array_mut().unwrap().pop();
        }
        "revision_two" => registry["revision"] = Value::from(2),
        "pretty_transport" => canonical_transport = false,
        other => panic!("unknown registry mutation {other}"),
    }
    (registry, canonical_transport)
}

fn construct_transport(registry: &Value, construction: &str) -> Vec<u8> {
    let canonical = registry_transport(registry, true);
    match construction {
        "canonical_lf" => canonical,
        "missing_lf" => canonical[..canonical.len() - 1].to_vec(),
        "extra_lf" => {
            let mut value = canonical;
            value.push(b'\n');
            value
        }
        "crlf" => {
            let mut value = canonical[..canonical.len() - 1].to_vec();
            value.extend_from_slice(b"\r\n");
            value
        }
        "pretty" => registry_transport(registry, false),
        "escaped_ascii" => String::from_utf8(canonical)
            .unwrap()
            .replacen(
                "mpk.semantic_profile.entry.v1",
                "\\u006dpk.semantic_profile.entry.v1",
                1,
            )
            .into_bytes(),
        "bom" => {
            let mut value = vec![0xef, 0xbb, 0xbf];
            value.extend_from_slice(&canonical);
            value
        }
        "invalid_utf8" => vec![0xff, b'\n'],
        "duplicate_name" => b"{\"schema\":\"mpk.semantic_profile.registry.v1\",\"schema\":\"mpk.semantic_profile.registry.v1\"}\n".to_vec(),
        "float" => b"{\"revision\":1.5}\n".to_vec(),
        "unsafe_integer" => b"{\"revision\":9007199254740992}\n".to_vec(),
        "depth_33" => {
            let mut value = vec![b'['; (SEMANTIC_REGISTRY_JSON_NESTING_MAX + 1) as usize];
            value.push(b'0');
            value.extend(vec![b']'; (SEMANTIC_REGISTRY_JSON_NESTING_MAX + 1) as usize]);
            value.push(b'\n');
            value
        }
        "above_byte_limit" => vec![b' '; (REGISTRY_TRANSPORT_BYTES_MAX + 1) as usize],
        other => panic!("unknown transport construction {other}"),
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
        other => panic!("unknown context mutation {other}"),
    }
    request
}

fn mutate_profile_envelope(base: &Value, mutation: &str) -> Value {
    let mut envelope = base.clone();
    match mutation {
        "none" => {}
        "unknown_envelope_field" => envelope["validator"] = Value::String("dynamic".into()),
        "nonobject_value" => envelope["value"] = Value::String("dynamic".into()),
        "unknown_entry" => envelope["profile_entry_sha256"] = Value::String("0".repeat(64)),
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
        other => panic!("unknown profile envelope mutation {other}"),
    }
    envelope
}

fn rehash_entry(registry: &mut Value, index: usize) {
    let digest = semantic_profile_entry_hash(&registry["profiles"][index]).unwrap();
    registry["profiles"][index]["entry_sha256"] = Value::String(digest);
}

fn rehash_registry(registry: &mut Value) {
    let digest = semantic_profile_registry_hash(registry).unwrap();
    registry["registry_sha256"] = Value::String(digest);
}

fn registry_transport(registry: &Value, canonical_transport: bool) -> Vec<u8> {
    if canonical_transport {
        canonical_registry_transport(registry).expect("canonical registry transport")
    } else {
        let mut value = serde_json::to_vec_pretty(registry).expect("pretty registry transport");
        value.push(b'\n');
        value
    }
}

fn load(bytes: &[u8], label: &str) -> Value {
    parse_strict_json(bytes, TEST_LIMITS)
        .unwrap_or_else(|error| panic!("{label} strict parse: {error}"));
    serde_json::from_slice(bytes).unwrap_or_else(|error| panic!("{label} parse: {error}"))
}

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&strict_value(value)).expect("canonical JSON")
}

fn strict_value(value: &Value) -> StrictJsonValue {
    match value {
        Value::Null => StrictJsonValue::Null,
        Value::Bool(value) => StrictJsonValue::Bool(*value),
        Value::Number(value) => StrictJsonValue::Integer(value.as_i64().expect("safe integer")),
        Value::String(value) => StrictJsonValue::String(value.clone()),
        Value::Array(values) => StrictJsonValue::Array(values.iter().map(strict_value).collect()),
        Value::Object(fields) => StrictJsonValue::Object(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), strict_value(value)))
                .collect(),
        ),
    }
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name}"))
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON array")
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("JSON object")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().expect("JSON integer")
}

fn boundary_integer(value: &Value) -> u64 {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .expect("boundary integer")
}

fn boolean(value: &Value) -> bool {
    value.as_bool().expect("JSON boolean")
}

fn assert_exact_fields(value: &Value, expected: &[&str]) {
    let fields = value.as_object().expect("JSON object");
    assert_eq!(
        fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected.iter().copied().collect::<BTreeSet<_>>()
    );
}

#[test]
fn installed_registry_contains_only_go_and_rust_and_rejects_other_profiles() {
    use mpk_vc::semantic_profile_registry::CompiledSemanticProfile;
    let raw = include_bytes!("../../../release/bundles/semantic-profile-registry.json");
    let registry = validate_semantic_profile_registry(raw, RegistryRevision::Revision4).unwrap();
    assert_eq!(registry.identity().revision(), 4);
    assert_eq!(
        registry
            .entries()
            .iter()
            .map(|entry| entry.source_language())
            .collect::<Vec<_>>(),
        ["go", "rust"]
    );
    for language in ["java", "csharp", "python", "typescript", "unknown"] {
        assert!(registry
            .lookup(language, &format!("mpk.{language}.scalar.v0"))
            .is_none());
        assert!(CompiledSemanticProfile::from_identity(
            language,
            &format!("mpk.{language}.scalar.v0")
        )
        .is_none());
    }
    assert!(validate_semantic_profile_registry(raw, RegistryRevision::Revision1).is_err());
}
