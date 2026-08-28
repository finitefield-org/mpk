use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, classify_semantic_registry_failure, semantic_profile_entry_hash,
    semantic_profile_registry_hash, validate_compiled_profile_envelope,
    validate_inactive_semantic_profile_registry, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_revision_2_append_only,
    validate_semantic_context_linkage, validate_semantic_registry_limit, validate_semantic_request,
    CompiledSemanticProfile, InactiveRegistryRevision, ProfileContractField,
    SemanticRegistryFailureDisposition, SemanticRegistryFailureSurface, SemanticRegistryLimit,
    SemanticRegistryValidationError, ValidatedSemanticProfileRegistry,
    COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX, CSHARP_SCALAR_ENTRY_SHA256,
    CSHARP_SCALAR_PROFILE, REGISTRY_CANONICAL_BYTES_MAX, REGISTRY_TRANSPORT_BYTES_MAX,
    REVISION_1_REGISTRY_SHA256, REVISION_2_REGISTRY_SHA256, SELECTION_CANONICAL_BYTES_MAX,
    SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX,
    SEMANTIC_REGISTRY_JSON_NESTING_MAX, SEMANTIC_REGISTRY_PROFILES_MAX,
    SEMANTIC_REGISTRY_REVISION_MAX, SOURCE_LANGUAGE_BYTES_MAX,
};
use mpk_vc::{
    canonical_json_bytes, import_vir_json, parse_strict_json, sha256_raw_file_bytes,
    SemanticProfile, StrictJsonLimits, StrictJsonValue, VirImportError,
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

const REGISTRY_V1_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const REGISTRY_V2_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v2.json");
const CSHARP_PROFILE_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/csharp-profile-v0.json");
const MANIFEST_BYTES: &[u8] = include_bytes!("../../../develop/specs/vectors/manifest.json");
const ACTIVE_VIR_BYTES: &[u8] =
    include_bytes!("../../../fixtures/vir-go/frontend/basic-branch/vir.json");

const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(4 * 1024 * 1024, 1_000_000, 128, 2 * 1024 * 1024);
const RUNTIME_OWNER: &str = "crates/mpk-vc/tests/semantic_profile_registry_runtime.rs";
const SOURCE_ARTIFACT_OWNER: &str = "crates/mpk-vc/tests/successor_source_artifacts.rs";
const SUCCESSOR_VC_OWNER: &str = "crates/mpk-vc/tests/successor_vc.rs";
const SUCCESSOR_POLICY_OWNER: &str = "crates/mpk-cli/tests/csharp_policy_verify.rs";
const V1_SHA256: &str = "f7007417279f5173d0102ec2833095f2d97f271e1cdf2622d381d31e6ab86ae7";
const V2_SHA256: &str = "19c657283836cb920f5c971f9c84ab267d48ea724c05bc1628de4889b4dd059f";

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
            validate_inactive_semantic_profile_registry(
                &transport,
                InactiveRegistryRevision::Revision1,
            ),
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
            validate_inactive_semantic_profile_registry(
                &transport,
                InactiveRegistryRevision::Revision1,
            ),
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
        InactiveRegistryRevision::Revision1,
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

#[test]
fn revision_two_csharp_hash_append_only_and_payload_vectors_execute_runtime_code() {
    let v1_vectors = load(REGISTRY_V1_BYTES, "revision-1 vectors");
    let v2_vectors = load(REGISTRY_V2_BYTES, "revision-2 vectors");
    assert_exact_fields(
        &v2_vectors,
        &[
            "schema",
            "owner_test",
            "mechanism_spec",
            "profile_spec",
            "predecessor",
            "csharp_entry",
            "registry",
            "hash_cases",
            "append_only_cases",
            "activation_cases",
        ],
    );
    assert_eq!(
        text(field(&v2_vectors, "schema")),
        "mpk.semantic_profile.registry.conformance.v2"
    );
    assert_eq!(
        text(field(&v2_vectors, "owner_test")),
        "crates/mpk-vc/tests/csharp_profile_spec.rs"
    );
    assert_eq!(
        text(field(&v2_vectors, "mechanism_spec")),
        "develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md"
    );
    assert_eq!(
        text(field(&v2_vectors, "profile_spec")),
        "develop/specs/CSHARP_PROFILE_V0.md"
    );

    let predecessor = validate_registry_fixture(
        field(&v2_vectors, "predecessor"),
        InactiveRegistryRevision::Revision1,
    );
    let successor = validate_registry_fixture(
        field(&v2_vectors, "registry"),
        InactiveRegistryRevision::Revision2,
    );
    assert_eq!(
        predecessor.identity().registry_sha256(),
        REVISION_1_REGISTRY_SHA256
    );
    assert_eq!(
        successor.identity().registry_sha256(),
        REVISION_2_REGISTRY_SHA256
    );
    validate_revision_2_append_only(&predecessor, &successor)
        .expect("revision 2 is an exact append-only successor");

    for case in array(field(&v2_vectors, "hash_cases")) {
        let id = text(field(case, "id"));
        let pointer = text(field(case, "source_pointer"));
        let complete = if pointer.is_empty() {
            field(&v2_vectors, "registry")
        } else {
            field(&v2_vectors, "registry")
                .pointer(pointer)
                .unwrap_or_else(|| panic!("missing hash pointer {pointer}"))
        };
        let digest = match text(field(case, "excluded_field")) {
            "entry_sha256" => semantic_profile_entry_hash(complete),
            "registry_sha256" => semantic_profile_registry_hash(complete),
            other => panic!("unknown excluded hash field {other}"),
        }
        .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        assert_eq!(
            canonical(complete).len() as i64,
            integer(field(case, "expected_complete_jcs_utf8_length")),
            "{id}"
        );
    }

    let csharp_entry = successor
        .lookup("csharp", CSHARP_SCALAR_PROFILE)
        .expect("revision 2 C# entry");
    assert_eq!(csharp_entry.entry_sha256(), CSHARP_SCALAR_ENTRY_SHA256);
    assert_eq!(
        csharp_entry.canonical_json(),
        canonical(field(&v2_vectors, "csharp_entry"))
    );
    assert_eq!(
        csharp_entry.compiled_profile(),
        CompiledSemanticProfile::CSharpScalarV0
    );
    assert!(predecessor
        .lookup("csharp", CSHARP_SCALAR_PROFILE)
        .is_none());
    for later in ["java", "dart", "typescript", "python"] {
        assert!(successor
            .lookup(later, &format!("mpk.{later}.unknown.v0"))
            .is_none());
    }

    let v1_go_request = field(field(&v1_vectors, "fixtures"), "go_request");
    let error = validate_semantic_request(&successor, v1_go_request)
        .expect_err("revision-2 consumer rejects revision-1 context identity");
    assert_eq!(error.code().as_str(), "SEMANTIC_REGISTRY_ASSERTION");
    assert_eq!(error.phase().as_str(), "registry_identity");
    let v1_go_context =
        validate_registry_semantic_context(&predecessor, field(v1_go_request, "semantic_context"))
            .expect("revision-1 Go context");
    let error = validate_registry_selection_envelope(
        &successor,
        &v1_go_context,
        field(v1_go_request, "selection"),
    )
    .expect_err("selection validation reasserts the complete registry identity");
    assert_eq!(error.code().as_str(), "SEMANTIC_REGISTRY_ASSERTION");
    assert_eq!(error.phase().as_str(), "registry_identity");

    for case in array(field(&v2_vectors, "append_only_cases")) {
        assert_eq!(text(field(case, "expect")), "pass");
        match text(field(case, "id")) {
            "append.csharp_first" => assert_eq!(
                successor.entries()[0].compiled_profile(),
                CompiledSemanticProfile::CSharpScalarV0
            ),
            "append.go_bytes_unchanged" => assert_eq!(
                successor.entries()[1].canonical_json(),
                predecessor.entries()[0].canonical_json()
            ),
            "append.rust_bytes_unchanged" => assert_eq!(
                successor.entries()[2].canonical_json(),
                predecessor.entries()[1].canonical_json()
            ),
            "append.exact_count" => assert_eq!(successor.entries().len(), 3),
            "append.no_later_language" => assert_eq!(successor.entries().len(), 3),
            "append.no_contract_mutation" => {
                validate_revision_2_append_only(&predecessor, &successor).unwrap()
            }
            "append.old_root_rejects_csharp" => {
                assert!(predecessor
                    .lookup("csharp", CSHARP_SCALAR_PROFILE)
                    .is_none())
            }
            "append.new_root_rejects_old_context" => {
                assert!(validate_semantic_request(&successor, v1_go_request).is_err())
            }
            id => panic!("unknown append-only case {id}"),
        }
    }

    let csharp_profile = load(CSHARP_PROFILE_BYTES, "C# profile vectors");
    let profile_identity = field(&csharp_profile, "profile_identity");
    assert_exact_fields(
        profile_identity,
        &[
            "source_language",
            "semantic_profile",
            "semantic_parameters_schema",
            "selection_schema",
            "contract_schema",
            "profile_entry_sha256",
            "registry_revision",
            "registry_sha256",
        ],
    );
    let identity = successor.identity();
    assert_eq!(
        text(field(profile_identity, "source_language")),
        csharp_entry.source_language()
    );
    assert_eq!(
        text(field(profile_identity, "semantic_profile")),
        csharp_entry.semantic_profile()
    );
    assert_eq!(
        text(field(profile_identity, "semantic_parameters_schema")),
        csharp_entry.semantic_parameters_schema()
    );
    assert_eq!(
        text(field(profile_identity, "selection_schema")),
        csharp_entry.selection_schema()
    );
    assert_eq!(
        text(field(profile_identity, "contract_schema")),
        "mpk.csharp.contract.v0"
    );
    assert_eq!(
        text(field(profile_identity, "profile_entry_sha256")),
        csharp_entry.entry_sha256()
    );
    assert_eq!(
        boundary_integer(field(profile_identity, "registry_revision")),
        identity.revision()
    );
    assert_eq!(
        text(field(profile_identity, "registry_sha256")),
        identity.registry_sha256()
    );
    let csharp_request = serde_json::json!({
        "semantic_context": {
            "profile_registry": {
                "schema": identity.schema(),
                "id": identity.id(),
                "revision": identity.revision(),
                "registry_sha256": identity.registry_sha256()
            },
            "profile_entry_sha256": field(profile_identity, "profile_entry_sha256"),
            "source_language": field(profile_identity, "source_language"),
            "semantic_profile": field(profile_identity, "semantic_profile"),
            "semantic_parameters": field(&csharp_profile, "semantic_parameters")
        },
        "selection": field(&csharp_profile, "selection_fixture")
    });
    let validated = validate_semantic_request(&successor, &csharp_request)
        .expect("frozen C# parameter and selection dispatch");
    assert_eq!(
        validated.compiled_profile(),
        CompiledSemanticProfile::CSharpScalarV0
    );
    assert_eq!(
        serde_json::to_value(&validated).expect("validated request serialization"),
        csharp_request,
        "the sealed validated model serializes only its revalidated wire fields"
    );

    let mut seen_fields = BTreeSet::new();
    for record in array(field(&csharp_profile, "profile_contracts")) {
        let field_name = text(field(record, "field"));
        let contract_field =
            ProfileContractField::from_name(field_name).expect("known C# contract field");
        assert!(seen_fields.insert(contract_field));
        validate_compiled_profile_envelope(&successor, field(record, "envelope"), contract_field)
            .unwrap_or_else(|error| panic!("C# {field_name} payload: {error}"));
    }
    assert_eq!(seen_fields.len(), 9);
}

#[test]
fn runtime_ownership_is_appended_without_changing_frozen_vectors_or_active_routes() {
    assert_eq!(sha256_raw_file_bytes(REGISTRY_V1_BYTES).to_hex(), V1_SHA256);
    assert_eq!(sha256_raw_file_bytes(REGISTRY_V2_BYTES).to_hex(), V2_SHA256);

    let manifest = load(MANIFEST_BYTES, "vector manifest");
    for (path, expected) in [
        (
            "develop/specs/vectors/semantic-profile-registry-v1.json",
            vec![
                "crates/mpk-vc/tests/semantic_profile_registry.rs",
                RUNTIME_OWNER,
                SOURCE_ARTIFACT_OWNER,
                SUCCESSOR_VC_OWNER,
                SUCCESSOR_POLICY_OWNER,
            ],
        ),
        (
            "develop/specs/vectors/semantic-profile-registry-v2.json",
            vec![
                "crates/mpk-vc/tests/csharp_profile_spec.rs",
                RUNTIME_OWNER,
                SUCCESSOR_VC_OWNER,
                SUCCESSOR_POLICY_OWNER,
            ],
        ),
    ] {
        let record = array(field(&manifest, "vectors"))
            .iter()
            .find(|record| text(field(record, "path")) == path)
            .unwrap_or_else(|| panic!("missing vector manifest record {path}"));
        let expected = expected
            .into_iter()
            .map(|owner| Value::String(owner.to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(array(field(record, "implementation_test_owners")), expected);
    }

    assert!(serde_json::from_str::<SemanticProfile>(r#""mpk.csharp.scalar.v0""#).is_err());
    let mut successor_vir = load(ACTIVE_VIR_BYTES, "active VIR fixture");
    successor_vir["schema"] = Value::String("mpk.vir.v1".into());
    let error = import_vir_json(&canonical(&successor_vir))
        .expect_err("the complete successor VIR stays outside the active importer");
    match error {
        VirImportError::Validation(error) => {
            assert_eq!(error.code(), "VIR_SCHEMA_UNSUPPORTED");
        }
        other => panic!("unexpected active VIR rejection: {other}"),
    }
}

fn validate_registry_fixture(
    registry: &Value,
    revision: InactiveRegistryRevision,
) -> ValidatedSemanticProfileRegistry {
    let transport = canonical_registry_transport(registry).expect("canonical registry transport");
    validate_inactive_semantic_profile_registry(&transport, revision)
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
