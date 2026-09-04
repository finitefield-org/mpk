use mpk_vc::csharp_practical_registry::{
    bind_successor_compiled_profile_envelope, canonical_successor_registry_transport,
    csharp_frontend_dispatch, csharp_practical_parameters_hash, csharp_practical_selection_hash,
    successor_compiled_profile_contract_hash, successor_profile_contracts,
    successor_profile_entry_hash, successor_profile_registry_hash, successor_semantic_context_hash,
    successor_validated_request_hash, validate_candidate_successor_registry,
    validate_successor_context_linkage, validate_successor_predecessor_projection,
    validate_successor_registry_document, validate_successor_registry_limit,
    validate_successor_semantic_context, validate_successor_semantic_request,
    CSharpFrontendDispatch, SuccessorCompiledSemanticProfile, SuccessorProfileContract,
    SuccessorProfileContractField, SuccessorRegistryDocumentKind, SuccessorRegistryErrorCode,
    SuccessorRegistryLimit, CSHARP_PRACTICAL_ENTRY_SHA256, CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
    CSHARP_PRACTICAL_PROFILE, CSHARP_PRACTICAL_SELECTION_SCHEMA,
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
    SUCCESSOR_CANDIDATE_REGISTRY_SHA256, SUCCESSOR_CANDIDATE_REVISION,
    SUCCESSOR_CONTEXT_HASH_DOMAINS, SUCCESSOR_CONTEXT_IDENTITIES, SUCCESSOR_CONTRACT_FIELDS,
    SUCCESSOR_CSHARP_SCALAR_ENTRY_SHA256, SUCCESSOR_GO_FIXED_ENTRY_SHA256,
    SUCCESSOR_JAVA_SCALAR_ENTRY_SHA256, SUCCESSOR_PARAMETER_HASH_DOMAINS,
    SUCCESSOR_PARAMETER_IDENTITIES, SUCCESSOR_PROFILE_ORDER, SUCCESSOR_REGISTRY_HASH_DOMAINS,
    SUCCESSOR_REGISTRY_IDENTITIES, SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX,
    SUCCESSOR_RUST_CHECKED_ENTRY_SHA256, SUCCESSOR_SELECTION_HASH_DOMAINS,
    SUCCESSOR_SELECTION_IDENTITIES, SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
    SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA, SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
    SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::{canonical_json_bytes, StrictJsonValue};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

const PROFILE_PACKAGE: &str =
    include_str!("../../../develop/specs/vectors/csharp-practical-profile-v1.json");
const REVISION_1_VECTORS: &str =
    include_str!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const REVISION_2_VECTORS: &str =
    include_str!("../../../develop/specs/vectors/semantic-profile-registry-v2.json");
const REVISION_3_VECTORS: &str =
    include_str!("../../../develop/specs/vectors/semantic-profile-registry-v3.json");
const CSHARP_SCALAR_VECTORS: &str =
    include_str!("../../../develop/specs/vectors/csharp-profile-v0.json");
const JAVA_SCALAR_VECTORS: &str =
    include_str!("../../../develop/specs/vectors/java-profile-v0.json");
const ACTIVE_REGISTRY: &[u8] =
    include_bytes!("../../../release/bundles/semantic-profile-registry.json");
const OWNER: &str = "CSHARP-03-T02-W01";
const PRODUCTION_TEST_OWNER: &str =
    "crates/mpk-vc/tests/csharp_practical_registry.rs#CSHARP-03-T02-W01";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn csharp_03_t02_w01_candidate_registry_is_closed_and_strict() {
    let fixture = candidate_registry();
    let transport = canonical_successor_registry_transport(&fixture).expect("candidate transport");
    let registry =
        validate_candidate_successor_registry(&transport).expect("valid successor candidate");

    assert_eq!(
        registry.identity().schema(),
        SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA
    );
    assert_eq!(registry.identity().id(), SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA);
    assert_eq!(registry.identity().revision(), SUCCESSOR_CANDIDATE_REVISION);
    assert_eq!(
        registry.identity().registry_sha256(),
        SUCCESSOR_CANDIDATE_REGISTRY_SHA256
    );
    assert_eq!(registry.entries().len(), SUCCESSOR_PROFILE_ORDER.len());
    for (index, (entry, expected_profile)) in registry
        .entries()
        .iter()
        .zip(SUCCESSOR_PROFILE_ORDER)
        .enumerate()
    {
        assert_eq!(entry.compiled_profile(), expected_profile);
        assert_eq!(entry.source_language(), expected_profile.source_language());
        assert_eq!(
            entry.semantic_profile(),
            expected_profile.semantic_profile()
        );
        assert_eq!(
            entry.entry_sha256(),
            expected_profile.expected_entry_sha256()
        );
        assert_eq!(
            entry.foundation_descriptor().content_sha256(),
            FOUNDATION_DESCRIPTOR_CONTENT_SHA256
        );
        assert_eq!(
            successor_profile_entry_hash(&fixture["profiles"][index]).expect("entry hash"),
            entry.entry_sha256()
        );
    }
    assert_eq!(
        successor_profile_registry_hash(&fixture).expect("registry hash"),
        SUCCESSOR_CANDIDATE_REGISTRY_SHA256
    );

    let mut unknown_root = fixture.clone();
    object_mut(&mut unknown_root).insert("unknown".to_owned(), Value::Null);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&unknown_root)),
        SuccessorRegistryErrorCode::Shape,
        "unknown root member",
    );

    let mut wrong_entry_type = fixture.clone();
    wrong_entry_type["profiles"][0]["source_language"] = Value::Bool(false);
    wrong_entry_type["profiles"]
        .as_array_mut()
        .expect("profiles")
        .reverse();
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&wrong_entry_type)),
        SuccessorRegistryErrorCode::Shape,
        "entry shape precedes ordering",
    );

    let mut old_revision = fixture.clone();
    old_revision["revision"] = json!(3);
    rehash_registry(&mut old_revision);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&old_revision)),
        SuccessorRegistryErrorCode::Scalar,
        "wrong successor revision",
    );

    let mut reordered = fixture.clone();
    reordered["profiles"]
        .as_array_mut()
        .expect("profiles")
        .swap(0, 1);
    rehash_registry(&mut reordered);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&reordered)),
        SuccessorRegistryErrorCode::Order,
        "profile order",
    );

    let mut later_entry = fixture.clone();
    later_entry["profiles"][0]["schema"] = json!("mpk.semantic_profile.entry.v3");
    rehash_entry(&mut later_entry, 0);
    rehash_registry(&mut later_entry);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&later_entry)),
        SuccessorRegistryErrorCode::Scalar,
        "later entry schema",
    );

    let mut mixed_identity = fixture.clone();
    mixed_identity["profiles"][4]["source_language"] = json!("java");
    rehash_entry(&mut mixed_identity, 4);
    rehash_registry(&mut mixed_identity);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&mixed_identity)),
        SuccessorRegistryErrorCode::Contract,
        "mixed language/profile pair",
    );

    let mut invalid_identifier = fixture.clone();
    invalid_identifier["profiles"][0]["semantic_profile"] = json!("mpk.csharp!practical.v1");
    rehash_entry(&mut invalid_identifier, 0);
    rehash_registry(&mut invalid_identifier);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&invalid_identifier)),
        SuccessorRegistryErrorCode::Scalar,
        "registry identifier grammar",
    );

    let mut wrong_contract = fixture.clone();
    wrong_contract["profiles"][0]["contracts"]["vir"] = json!("mpk.profile.vir.unknown.v1");
    rehash_entry(&mut wrong_contract, 0);
    rehash_registry(&mut wrong_contract);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&wrong_contract)),
        SuccessorRegistryErrorCode::Contract,
        "unknown contract",
    );

    let mut wrong_foundation = fixture.clone();
    wrong_foundation["profiles"][0]["foundation_descriptor"]["content_sha256"] = json!(ZERO_SHA256);
    rehash_entry(&mut wrong_foundation, 0);
    rehash_registry(&mut wrong_foundation);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&wrong_foundation)),
        SuccessorRegistryErrorCode::Foundation,
        "foundation binding",
    );

    let mut wrong_entry_hash = fixture.clone();
    wrong_entry_hash["profiles"][0]["entry_sha256"] = json!(ZERO_SHA256);
    rehash_registry(&mut wrong_entry_hash);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&wrong_entry_hash)),
        SuccessorRegistryErrorCode::EntryHash,
        "entry hash",
    );

    let mut wrong_registry_hash = fixture.clone();
    wrong_registry_hash["registry_sha256"] = json!(ZERO_SHA256);
    assert_error(
        validate_candidate_successor_registry(&registry_transport(&wrong_registry_hash)),
        SuccessorRegistryErrorCode::RegistryHash,
        "registry hash",
    );

    let without_newline = canonical(&fixture);
    assert_error(
        validate_candidate_successor_registry(&without_newline),
        SuccessorRegistryErrorCode::Canonical,
        "missing transport newline",
    );
    let pretty = serde_json::to_vec_pretty(&fixture).expect("pretty JSON");
    assert_error(
        validate_candidate_successor_registry(&pretty),
        SuccessorRegistryErrorCode::Canonical,
        "noncanonical transport",
    );
    assert_error(
        validate_candidate_successor_registry(br#"{"schema":0,"schema":0}"#),
        SuccessorRegistryErrorCode::Transport,
        "duplicate key",
    );
    let oversized = vec![
        b' ';
        usize::try_from(SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX + 1)
            .expect("transport maximum fits usize")
    ];
    assert_error(
        validate_candidate_successor_registry(&oversized),
        SuccessorRegistryErrorCode::Transport,
        "transport limit",
    );
}

#[test]
fn csharp_03_t02_w01_candidate_hashes_are_immutable() {
    let registry = candidate_registry();
    let actual_entries = registry["profiles"]
        .as_array()
        .expect("profiles")
        .iter()
        .map(|entry| successor_profile_entry_hash(entry).expect("entry hash"))
        .collect::<Vec<_>>();
    let expected_entries = vec![
        CSHARP_PRACTICAL_ENTRY_SHA256.to_owned(),
        SUCCESSOR_CSHARP_SCALAR_ENTRY_SHA256.to_owned(),
        SUCCESSOR_GO_FIXED_ENTRY_SHA256.to_owned(),
        SUCCESSOR_JAVA_SCALAR_ENTRY_SHA256.to_owned(),
        SUCCESSOR_RUST_CHECKED_ENTRY_SHA256.to_owned(),
    ];
    assert_eq!(
        actual_entries, expected_entries,
        "frozen candidate entry hashes"
    );
    assert_eq!(
        successor_profile_registry_hash(&registry).expect("registry hash"),
        SUCCESSOR_CANDIDATE_REGISTRY_SHA256,
        "frozen candidate registry hash"
    );
}

#[test]
fn csharp_03_t02_w01_all_fifty_published_vectors_execute_production_validation() {
    let package = load(PROFILE_PACKAGE);
    let vectors = package["vectors"].as_array().expect("vectors");
    let owned = vectors
        .iter()
        .filter(|vector| text(&vector["implementation_owner"]) == OWNER)
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 50, "published W01 vector count");
    assert!(owned
        .iter()
        .all(|vector| { text(&vector["production_test_owner"]) == PRODUCTION_TEST_OWNER }));
    assert_eq!(
        owned
            .iter()
            .map(|vector| text(&vector["id"]))
            .collect::<BTreeSet<_>>()
            .len(),
        owned.len(),
        "published W01 vector IDs are unique"
    );

    let schema_vectors = owned
        .iter()
        .copied()
        .filter(|vector| text(&vector["family"]) == "schema")
        .collect::<Vec<_>>();
    assert_eq!(schema_vectors.len(), 44, "published W01 schema vectors");
    for vector in schema_vectors {
        execute_schema_vector(vector);
    }

    let context_vectors = owned
        .iter()
        .copied()
        .filter(|vector| text(&vector["family"]) == "context")
        .collect::<Vec<_>>();
    assert_eq!(context_vectors.len(), 6, "published W01 context vectors");
    execute_context_vectors(&context_vectors);
}

#[test]
fn csharp_03_t02_w01_context_request_and_compiled_contract_matrix_are_bound() {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(&registry_transport(&registry_value))
        .expect("successor registry");

    let contracts = successor_profile_contracts().collect::<Vec<_>>();
    assert_eq!(contracts.len(), 45, "five profiles by nine contract fields");
    assert_eq!(
        contracts.iter().copied().collect::<BTreeSet<_>>().len(),
        contracts.len(),
        "compiled contracts are unique"
    );
    for entry in registry.entries() {
        let context_value = context_fixture(&registry_value, entry.compiled_profile());
        let context = validate_successor_semantic_context(&registry, &canonical(&context_value))
            .expect("profile context");
        let request_value =
            request_fixture(context_value, profile_selection(entry.compiled_profile()));
        let request = validate_successor_semantic_request(&registry, &canonical(&request_value))
            .unwrap_or_else(|error| panic!("{} request: {error}", entry.semantic_profile()));
        assert_eq!(request.compiled_profile(), entry.compiled_profile());
        for field in SUCCESSOR_CONTRACT_FIELDS {
            let contract = SuccessorProfileContract::new(entry.compiled_profile(), field);
            let envelope = json!({
                "profile_entry_sha256": entry.entry_sha256(),
                "contract_id": contract.contract_id(),
                "value": {}
            });
            let bound = bind_successor_compiled_profile_envelope(
                &registry,
                &context,
                field,
                &canonical(&envelope),
            )
            .unwrap_or_else(|error| panic!("{}: {error}", contract.contract_id()));
            assert_eq!(bound.contract(), contract);
            assert_eq!(bound.profile_entry_sha256(), entry.entry_sha256());
            assert_eq!(
                bound.envelope_sha256(),
                successor_compiled_profile_contract_hash(&envelope).expect("envelope hash")
            );
        }
    }

    assert_eq!(
        csharp_frontend_dispatch(SuccessorCompiledSemanticProfile::CSharpScalarV0),
        Some(CSharpFrontendDispatch::ScalarV0)
    );
    assert_eq!(
        csharp_frontend_dispatch(SuccessorCompiledSemanticProfile::CSharpPracticalV1),
        Some(CSharpFrontendDispatch::PracticalV1)
    );
    for profile in [
        SuccessorCompiledSemanticProfile::GoFixedV0,
        SuccessorCompiledSemanticProfile::JavaScalarV0,
        SuccessorCompiledSemanticProfile::RustCheckedV0,
    ] {
        assert_eq!(csharp_frontend_dispatch(profile), None);
    }

    let practical_context_value = context_fixture(
        &registry_value,
        SuccessorCompiledSemanticProfile::CSharpPracticalV1,
    );
    let practical_context =
        validate_successor_semantic_context(&registry, &canonical(&practical_context_value))
            .expect("practical context");
    let wrong_contract = json!({
        "profile_entry_sha256": practical_context.profile_entry_sha256(),
        "contract_id": "mpk.profile.vir.csharp_scalar.v1",
        "value": {}
    });
    assert_error(
        bind_successor_compiled_profile_envelope(
            &registry,
            &practical_context,
            SuccessorProfileContractField::Vir,
            &canonical(&wrong_contract),
        ),
        SuccessorRegistryErrorCode::Contract,
        "cross-profile compiled contract",
    );
    let wrong_entry = json!({
        "profile_entry_sha256": ZERO_SHA256,
        "contract_id": "mpk.profile.vir.csharp_practical.v1",
        "value": {}
    });
    assert_error(
        bind_successor_compiled_profile_envelope(
            &registry,
            &practical_context,
            SuccessorProfileContractField::Vir,
            &canonical(&wrong_entry),
        ),
        SuccessorRegistryErrorCode::ProfileEntryMismatch,
        "cross-entry compiled contract",
    );

    for (label, field, invalid) in [
        (
            "nonportable source path",
            "source_paths",
            json!(["src/C:/Order.cs"]),
        ),
        (
            "display-string callable ID",
            "selected_root_ids",
            json!(["Business.Order::Create"]),
        ),
    ] {
        let mut selection = practical_selection();
        selection[field] = invalid;
        rehash_selection(&mut selection);
        let request = request_fixture(practical_context_value.clone(), selection);
        assert_error(
            validate_successor_semantic_request(&registry, &canonical(&request)),
            SuccessorRegistryErrorCode::SelectionMismatch,
            label,
        );
    }

    let mut wrong_context_foundation = practical_context_value.clone();
    wrong_context_foundation["foundation_descriptor"]["content_sha256"] = json!(ZERO_SHA256);
    assert_error(
        validate_successor_semantic_context(&registry, &canonical(&wrong_context_foundation)),
        SuccessorRegistryErrorCode::Foundation,
        "context foundation content hash",
    );

    let mut wrong_context_parameters = practical_context_value.clone();
    wrong_context_parameters["semantic_parameters"]["value"]["check_overflow_default"] =
        json!(false);
    assert_error(
        validate_successor_semantic_context(&registry, &canonical(&wrong_context_parameters)),
        SuccessorRegistryErrorCode::ParametersMismatch,
        "context practical parameter value",
    );

    let mut wrong_request_hash = request_fixture(practical_context_value, practical_selection());
    wrong_request_hash["request_sha256"] = json!(ZERO_SHA256);
    assert_error(
        validate_successor_semantic_request(&registry, &canonical(&wrong_request_hash)),
        SuccessorRegistryErrorCode::RequestHash,
        "validated request hash",
    );
}

#[test]
fn csharp_03_t02_w01_predecessor_revisions_remain_separate_and_revision_three_projects() {
    let revision_1 = load(REVISION_1_VECTORS)["fixtures"]["base_registry"].clone();
    let revision_2 = load(REVISION_2_VECTORS)["registry"].clone();
    let revision_3 = load(REVISION_3_VECTORS)["registry"].clone();
    for (value, revision) in [
        (&revision_1, RegistryRevision::Revision1),
        (&revision_2, RegistryRevision::Revision2),
        (&revision_3, RegistryRevision::Revision3),
    ] {
        let transport = canonical_registry_transport(value).expect("predecessor transport");
        validate_semantic_profile_registry(&transport, revision)
            .unwrap_or_else(|error| panic!("revision {}: {error}", revision.revision()));
        assert!(
            validate_candidate_successor_registry(&transport).is_err(),
            "successor parser must reject predecessor revision {}",
            revision.revision()
        );
    }

    let active = validate_semantic_profile_registry(ACTIVE_REGISTRY, RegistryRevision::Revision3)
        .expect("installed predecessor registry");
    assert_eq!(active.revision(), RegistryRevision::Revision3);
    assert_eq!(active.entries().len(), 4);
    assert!(active.lookup("csharp", CSHARP_PRACTICAL_PROFILE).is_none());
    assert!(!std::str::from_utf8(ACTIVE_REGISTRY)
        .expect("active registry is UTF-8")
        .contains(SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA));

    let successor_value = candidate_registry();
    let successor_transport = registry_transport(&successor_value);
    let successor =
        validate_candidate_successor_registry(&successor_transport).expect("successor candidate");
    validate_successor_predecessor_projection(&active, &successor)
        .expect("revision-3 append-only projection");
    assert!(
        validate_semantic_profile_registry(&successor_transport, RegistryRevision::Revision3)
            .is_err()
    );

    let predecessor_2 = validate_semantic_profile_registry(
        &canonical_registry_transport(&revision_2).expect("revision-2 transport"),
        RegistryRevision::Revision2,
    )
    .expect("revision-2 registry");
    assert_error(
        validate_successor_predecessor_projection(&predecessor_2, &successor),
        SuccessorRegistryErrorCode::PredecessorProjection,
        "only the active predecessor revision projects",
    );
}

#[test]
fn csharp_03_t02_w01_identity_domains_and_limits_match_the_freeze() {
    let package = load(PROFILE_PACKAGE);
    let families = package["frozen_contract"]["identity_families"]
        .as_array()
        .expect("identity families");
    assert_family(
        families,
        "semantic_registry",
        SUCCESSOR_REGISTRY_IDENTITIES,
        SUCCESSOR_REGISTRY_HASH_DOMAINS,
    );
    assert_family(
        families,
        "semantic_context",
        SUCCESSOR_CONTEXT_IDENTITIES,
        SUCCESSOR_CONTEXT_HASH_DOMAINS,
    );
    assert_family(
        families,
        "semantic_parameters",
        SUCCESSOR_PARAMETER_IDENTITIES,
        SUCCESSOR_PARAMETER_HASH_DOMAINS,
    );
    assert_family(
        families,
        "selection",
        SUCCESSOR_SELECTION_IDENTITIES,
        SUCCESSOR_SELECTION_HASH_DOMAINS,
    );

    let profile_contract = families
        .iter()
        .find(|family| text(&family["family"]) == "profile_contract")
        .expect("profile-contract family");
    let frozen_contracts = strings(&profile_contract["successor_identities"]);
    let runtime_contracts = successor_profile_contracts()
        .map(SuccessorProfileContract::contract_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(runtime_contracts.len(), 45);
    let frozen_compiled_contracts = frozen_contracts
        .iter()
        .filter(|name| name.starts_with("mpk.profile."))
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(runtime_contracts, frozen_compiled_contracts);
    assert!(strings(&profile_contract["successor_hash_domains"])
        .contains("MPK-COMPILED-PROFILE-CONTRACT-1.0"));

    let limits = [
        (
            "registry_canonical_bytes",
            SuccessorRegistryLimit::RegistryCanonicalBytes,
        ),
        (
            "registry_transport_bytes",
            SuccessorRegistryLimit::RegistryTransportBytes,
        ),
        ("json_nesting", SuccessorRegistryLimit::JsonNesting),
        ("identifier_bytes", SuccessorRegistryLimit::IdentifierBytes),
        (
            "source_language_bytes",
            SuccessorRegistryLimit::SourceLanguageBytes,
        ),
        ("profiles", SuccessorRegistryLimit::Profiles),
        (
            "semantic_parameters_canonical_bytes",
            SuccessorRegistryLimit::SemanticParametersCanonicalBytes,
        ),
        (
            "selection_canonical_bytes",
            SuccessorRegistryLimit::SelectionCanonicalBytes,
        ),
        (
            "compiled_profile_payload_canonical_bytes",
            SuccessorRegistryLimit::CompiledProfileCanonicalBytes,
        ),
        ("revision", SuccessorRegistryLimit::Revision),
    ];
    for (id, expected) in limits {
        let parsed = SuccessorRegistryLimit::from_id(id).expect("registered limit");
        assert_eq!(parsed, expected, "{id}");
        let maximum = parsed.inclusive_maximum();
        if maximum > 0 {
            validate_successor_registry_limit(parsed, maximum - 1)
                .unwrap_or_else(|error| panic!("{id} below: {error}"));
        }
        validate_successor_registry_limit(parsed, maximum)
            .unwrap_or_else(|error| panic!("{id} at: {error}"));
        assert_error(
            validate_successor_registry_limit(parsed, maximum + 1),
            SuccessorRegistryErrorCode::Limit,
            id,
        );
    }
    assert!(SuccessorRegistryLimit::from_id("unknown").is_none());

    let mut oversized_selection = practical_selection();
    oversized_selection["padding"] = Value::String("x".repeat(65_536));
    assert_error(
        csharp_practical_selection_hash(&oversized_selection),
        SuccessorRegistryErrorCode::SelectionMismatch,
        "selection hash ceiling",
    );
    let mut oversized_parameters =
        parameter_envelope(SuccessorCompiledSemanticProfile::CSharpPracticalV1);
    oversized_parameters["padding"] = Value::String("x".repeat(65_536));
    assert_error(
        csharp_practical_parameters_hash(&oversized_parameters),
        SuccessorRegistryErrorCode::ParametersMismatch,
        "parameter hash ceiling",
    );
}

fn execute_schema_vector(vector: &Value) {
    let id = text(&vector["id"]);
    let inputs = &vector["inputs"];
    let target = inputs
        .get("schema")
        .or_else(|| inputs.get("record"))
        .map(text)
        .expect("schema vector target");
    let (kind, fixture) = schema_fixture(target);
    let expected = &vector["expected"];

    if expected.get("accept").is_some() {
        validate_successor_registry_document(kind, &canonical(&fixture))
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        return;
    }

    if let Some(raw) = inputs.get("raw_utf8") {
        assert!(
            validate_successor_registry_document(kind, text(raw).as_bytes()).is_err(),
            "{id}"
        );
        return;
    }

    match text(&inputs["mutation"]) {
        "remove_each_required_field" => {
            for field in fixture.as_object().expect("fixture object").keys() {
                let mut mutated = fixture.clone();
                object_mut(&mut mutated).remove(field);
                assert!(
                    validate_successor_registry_document(kind, &canonical(&mutated)).is_err(),
                    "{id}: missing {field}"
                );
            }
        }
        "append_unknown_field" => {
            let mut mutated = fixture.clone();
            object_mut(&mut mutated).insert("unknown".to_owned(), Value::Null);
            assert!(
                validate_successor_registry_document(kind, &canonical(&mutated)).is_err(),
                "{id}"
            );
        }
        "replace_each_field_with_wrong_json_type" => {
            for field in fixture.as_object().expect("fixture object").keys() {
                let mut mutated = fixture.clone();
                mutated[field] = wrong_type(&fixture[field]);
                assert!(
                    validate_successor_registry_document(kind, &canonical(&mutated)).is_err(),
                    "{id}: wrong type for {field}"
                );
            }
        }
        "later_version" => {
            let mut mutated = fixture.clone();
            let schema = text(&mutated["schema"]);
            let (prefix, version) = schema.rsplit_once(".v").expect("versioned schema");
            let next = version.parse::<u64>().expect("numeric version") + 1;
            mutated["schema"] = json!(format!("{prefix}.v{next}"));
            assert!(
                validate_successor_registry_document(kind, &canonical(&mutated)).is_err(),
                "{id}"
            );
        }
        mutation => panic!("{id}: unknown schema mutation {mutation}"),
    }
}

fn execute_context_vectors(vectors: &[&Value]) {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(&registry_transport(&registry_value))
        .expect("candidate registry");
    let valid_context = context_fixture(
        &registry_value,
        SuccessorCompiledSemanticProfile::CSharpPracticalV1,
    );
    let valid_selection = practical_selection();

    for vector in vectors {
        let id = text(&vector["id"]);
        match id {
            "context.valid" => {
                let context =
                    validate_successor_semantic_context(&registry, &canonical(&valid_context))
                        .expect("valid context");
                let request_value = request_fixture(valid_context.clone(), valid_selection.clone());
                let request =
                    validate_successor_semantic_request(&registry, &canonical(&request_value))
                        .expect("valid request");
                assert_eq!(context, *request.semantic_context());
                validate_successor_context_linkage(&context, request.semantic_context())
                    .expect("complete context equality");
                assert_eq!(
                    serde_json::to_value(&context).expect("serialize context"),
                    valid_context
                );
                assert_eq!(
                    successor_semantic_context_hash(&valid_context)
                        .expect("semantic-context hash")
                        .len(),
                    64
                );
                assert_eq!(
                    csharp_practical_parameters_hash(&valid_context["semantic_parameters"])
                        .expect("practical-parameter hash")
                        .len(),
                    64
                );
            }
            "context.entry_hash_mismatch" => {
                let mut context = valid_context.clone();
                context["profile_entry_sha256"] = json!(ZERO_SHA256);
                assert_error(
                    validate_successor_semantic_context(&registry, &canonical(&context)),
                    SuccessorRegistryErrorCode::ProfileEntryMismatch,
                    id,
                );
            }
            "context.language_profile_mismatch" => {
                let mut context = context_fixture(
                    &registry_value,
                    SuccessorCompiledSemanticProfile::RustCheckedV0,
                );
                context["source_language"] = json!("csharp");
                assert_error(
                    validate_successor_semantic_context(&registry, &canonical(&context)),
                    SuccessorRegistryErrorCode::ContextMismatch,
                    id,
                );
            }
            "context.parameters_schema_mismatch" => {
                let mut context = valid_context.clone();
                context["semantic_parameters"]["schema"] =
                    json!("mpk.semantic_parameters.csharp_scalar.v0");
                context["semantic_parameters"]["value"] =
                    parameter_values(SuccessorCompiledSemanticProfile::CSharpScalarV0);
                assert_error(
                    validate_successor_semantic_context(&registry, &canonical(&context)),
                    SuccessorRegistryErrorCode::ParametersMismatch,
                    id,
                );
            }
            "context.projected_context" => {
                let mut projected = valid_context.clone();
                object_mut(&mut projected).remove("foundation_descriptor");
                assert_error(
                    validate_successor_semantic_context(&registry, &canonical(&projected)),
                    SuccessorRegistryErrorCode::Shape,
                    id,
                );
            }
            "context.selection_schema_mismatch" => {
                let selection = json!({
                    "schema": "mpk.selection.csharp_methods.v0",
                    "value": {
                        "compilation": "business.core",
                        "contracts": ["contracts/order.json"],
                        "methods": ["Business.Order::Create()->i32"],
                        "sources": ["src/Order.cs"]
                    }
                });
                let request = request_fixture(valid_context.clone(), selection);
                assert_error(
                    validate_successor_semantic_request(&registry, &canonical(&request)),
                    SuccessorRegistryErrorCode::SelectionMismatch,
                    id,
                );
            }
            other => panic!("unknown context vector {other}"),
        }
    }
}

fn schema_fixture(target: &str) -> (SuccessorRegistryDocumentKind, Value) {
    let registry = candidate_registry();
    let context = context_fixture(
        &registry,
        SuccessorCompiledSemanticProfile::CSharpPracticalV1,
    );
    match target {
        "csharp_practical_parameter_values_v1" => (
            SuccessorRegistryDocumentKind::PracticalParameterValues,
            parameter_values(SuccessorCompiledSemanticProfile::CSharpPracticalV1),
        ),
        "foundation_descriptor_ref_v1" => (
            SuccessorRegistryDocumentKind::FoundationDescriptorRef,
            foundation_descriptor(),
        ),
        "semantic_parameters_envelope" => (
            SuccessorRegistryDocumentKind::SemanticParametersEnvelope,
            parameter_envelope(SuccessorCompiledSemanticProfile::CSharpPracticalV1),
        ),
        "semantic_profile_registry_ref_v2" => (
            SuccessorRegistryDocumentKind::RegistryIdentity,
            registry_identity(&registry),
        ),
        CSHARP_PRACTICAL_SELECTION_SCHEMA => (
            SuccessorRegistryDocumentKind::PracticalSelection,
            practical_selection(),
        ),
        SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA => {
            (SuccessorRegistryDocumentKind::SemanticContext, context)
        }
        CSHARP_PRACTICAL_PARAMETERS_SCHEMA => (
            SuccessorRegistryDocumentKind::PracticalParameters,
            parameter_envelope(SuccessorCompiledSemanticProfile::CSharpPracticalV1),
        ),
        SUCCESSOR_VALIDATED_REQUEST_SCHEMA => (
            SuccessorRegistryDocumentKind::ValidatedRequest,
            request_fixture(context, practical_selection()),
        ),
        other => panic!("unknown W01 schema target {other}"),
    }
}

fn candidate_registry() -> Value {
    let profiles = SUCCESSOR_PROFILE_ORDER
        .into_iter()
        .map(candidate_entry)
        .collect::<Vec<_>>();
    let mut registry = json!({
        "schema": SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
        "id": SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
        "revision": SUCCESSOR_CANDIDATE_REVISION,
        "profiles": profiles,
        "registry_sha256": ZERO_SHA256
    });
    rehash_registry(&mut registry);
    registry
}

fn candidate_entry(profile: SuccessorCompiledSemanticProfile) -> Value {
    let contracts = SUCCESSOR_CONTRACT_FIELDS
        .into_iter()
        .map(|field| {
            (
                field.as_str().to_owned(),
                Value::String(SuccessorProfileContract::new(profile, field).contract_id()),
            )
        })
        .collect::<Map<_, _>>();
    let mut entry = json!({
        "schema": SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA,
        "source_language": profile.source_language(),
        "semantic_profile": profile.semantic_profile(),
        "semantic_parameters_schema": profile.semantic_parameters_schema(),
        "selection_schema": profile.selection_schema(),
        "foundation_descriptor": foundation_descriptor(),
        "contracts": contracts,
        "entry_sha256": ZERO_SHA256
    });
    let digest = successor_profile_entry_hash(&entry).expect("candidate entry hash");
    entry["entry_sha256"] = Value::String(digest);
    entry
}

fn foundation_descriptor() -> Value {
    json!({
        "schema": FOUNDATION_DESCRIPTOR_SCHEMA,
        "id": FOUNDATION_DESCRIPTOR_ID,
        "content_sha256": FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    })
}

fn registry_identity(registry: &Value) -> Value {
    json!({
        "schema": registry["schema"],
        "id": registry["id"],
        "revision": registry["revision"],
        "registry_sha256": registry["registry_sha256"]
    })
}

fn parameter_envelope(profile: SuccessorCompiledSemanticProfile) -> Value {
    json!({
        "schema": profile.semantic_parameters_schema(),
        "value": parameter_values(profile)
    })
}

fn parameter_values(profile: SuccessorCompiledSemanticProfile) -> Value {
    match profile {
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => json!({
            "check_overflow_default": true,
            "documentation_mode": "none",
            "language_version": "14.0",
            "nullable_context": "enable",
            "optimization": "release",
            "platform": "x64",
            "pointer_width": 64,
            "preprocessor_symbols": [],
            "source_kind": "regular",
            "target_framework": "net10.0",
            "target_id": "linux-x64",
            "unsafe": false
        }),
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => json!({
            "check_overflow_default": false,
            "documentation_mode": "none",
            "language_version": "14.0",
            "nullable_context": "disable",
            "optimization": "release",
            "platform": "x64",
            "pointer_width": 64,
            "preprocessor_symbols": [],
            "source_kind": "regular",
            "target_framework": "net10.0",
            "target_id": "linux-x64",
            "unsafe": false
        }),
        SuccessorCompiledSemanticProfile::GoFixedV0 => json!({
            "target_id": "linux/amd64",
            "pointer_width": 64
        }),
        SuccessorCompiledSemanticProfile::JavaScalarV0 => json!({
            "annotation_processing": "none",
            "encoding": "UTF-8",
            "language_version": "25",
            "preview": false,
            "release": "25",
            "target_id": "linux-x64"
        }),
        SuccessorCompiledSemanticProfile::RustCheckedV0 => json!({
            "target_id": "x86_64-unknown-linux-gnu",
            "pointer_width": 64,
            "overflow_mode": "checked",
            "panic_mode": "abort"
        }),
    }
}

fn context_fixture(registry: &Value, profile: SuccessorCompiledSemanticProfile) -> Value {
    let entry = registry["profiles"]
        .as_array()
        .expect("profiles")
        .iter()
        .find(|entry| text(&entry["semantic_profile"]) == profile.semantic_profile())
        .expect("profile entry");
    json!({
        "schema": SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
        "profile_registry": registry_identity(registry),
        "profile_entry_sha256": entry["entry_sha256"],
        "source_language": profile.source_language(),
        "semantic_profile": profile.semantic_profile(),
        "semantic_parameters": parameter_envelope(profile),
        "foundation_descriptor": foundation_descriptor()
    })
}

fn practical_selection() -> Value {
    let mut selection = json!({
        "schema": CSHARP_PRACTICAL_SELECTION_SCHEMA,
        "compilation_id": "business.core",
        "source_paths": ["src/Order.cs"],
        "selected_root_ids": [format!("mpk.csharp.source.{}", "1".repeat(64))],
        "sidecar_paths": [],
        "selection_sha256": ZERO_SHA256
    });
    rehash_selection(&mut selection);
    selection
}

fn rehash_selection(selection: &mut Value) {
    let digest = csharp_practical_selection_hash(selection).expect("selection hash");
    selection["selection_sha256"] = Value::String(digest);
}

fn profile_selection(profile: SuccessorCompiledSemanticProfile) -> Value {
    match profile {
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => practical_selection(),
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => {
            load(CSHARP_SCALAR_VECTORS)["selection_fixture"].clone()
        }
        SuccessorCompiledSemanticProfile::GoFixedV0 => {
            load(REVISION_1_VECTORS)["fixtures"]["go_request"]["selection"].clone()
        }
        SuccessorCompiledSemanticProfile::JavaScalarV0 => {
            load(JAVA_SCALAR_VECTORS)["selection_fixture"].clone()
        }
        SuccessorCompiledSemanticProfile::RustCheckedV0 => {
            load(REVISION_1_VECTORS)["fixtures"]["rust_request"]["selection"].clone()
        }
    }
}

fn request_fixture(context: Value, selection: Value) -> Value {
    let mut request = json!({
        "schema": SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
        "semantic_context": context,
        "selection": selection,
        "request_sha256": ZERO_SHA256
    });
    let digest = successor_validated_request_hash(&request).expect("request hash");
    request["request_sha256"] = Value::String(digest);
    request
}

fn rehash_entry(registry: &mut Value, index: usize) {
    let digest =
        successor_profile_entry_hash(&registry["profiles"][index]).expect("mutated entry hash");
    registry["profiles"][index]["entry_sha256"] = Value::String(digest);
}

fn rehash_registry(registry: &mut Value) {
    let digest = successor_profile_registry_hash(registry).expect("mutated registry hash");
    registry["registry_sha256"] = Value::String(digest);
}

fn registry_transport(registry: &Value) -> Vec<u8> {
    canonical_successor_registry_transport(registry).expect("registry transport")
}

fn assert_family(
    families: &[Value],
    id: &str,
    expected_identities: &[&str],
    expected_domains: &[&str],
) {
    let family = families
        .iter()
        .find(|family| text(&family["family"]) == id)
        .unwrap_or_else(|| panic!("missing identity family {id}"));
    assert_eq!(
        strings(&family["successor_identities"]),
        expected_identities
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<BTreeSet<_>>(),
        "{id} successor identities"
    );
    assert_eq!(
        strings(&family["successor_hash_domains"]),
        expected_domains
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<BTreeSet<_>>(),
        "{id} successor hash domains"
    );
}

fn wrong_type(value: &Value) -> Value {
    match value {
        Value::Null => Value::Bool(true),
        Value::Bool(_) => Value::String("wrong".to_owned()),
        Value::Number(_) => Value::String("wrong".to_owned()),
        Value::String(_) => Value::Bool(false),
        Value::Array(_) => json!({}),
        Value::Object(_) => json!([]),
    }
}

fn assert_error<T>(
    result: Result<T, mpk_vc::csharp_practical_registry::SuccessorRegistryValidationError>,
    expected: SuccessorRegistryErrorCode,
    case: &str,
) {
    let error = result.unwrap_err_or_else(case);
    assert_eq!(error.code(), expected, "{case}: {error}");
}

trait ResultErrorExt<T> {
    fn unwrap_err_or_else(
        self,
        case: &str,
    ) -> mpk_vc::csharp_practical_registry::SuccessorRegistryValidationError;
}

impl<T> ResultErrorExt<T>
    for Result<T, mpk_vc::csharp_practical_registry::SuccessorRegistryValidationError>
{
    fn unwrap_err_or_else(
        self,
        case: &str,
    ) -> mpk_vc::csharp_practical_registry::SuccessorRegistryValidationError {
        match self {
            Ok(_) => panic!("{case}: expected rejection"),
            Err(error) => error,
        }
    }
}

fn load(source: &str) -> Value {
    serde_json::from_str(source).expect("JSON fixture")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn strings(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .expect("JSON array")
        .iter()
        .map(|value| text(value).to_owned())
        .collect()
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("JSON object")
}

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&to_strict(value)).expect("canonical JSON")
}

fn to_strict(value: &Value) -> StrictJsonValue {
    match value {
        Value::Null => StrictJsonValue::Null,
        Value::Bool(value) => StrictJsonValue::Bool(*value),
        Value::Number(value) => {
            StrictJsonValue::Integer(value.as_i64().expect("integer JSON number"))
        }
        Value::String(value) => StrictJsonValue::String(value.clone()),
        Value::Array(values) => StrictJsonValue::Array(values.iter().map(to_strict).collect()),
        Value::Object(values) => StrictJsonValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), to_strict(value)))
                .collect(),
        ),
    }
}
