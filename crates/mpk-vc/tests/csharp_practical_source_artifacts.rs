use mpk_vc::csharp_practical_registry::{
    canonical_successor_registry_transport, csharp_practical_selection_hash,
    successor_profile_entry_hash, successor_profile_registry_hash,
    successor_validated_request_hash, validate_candidate_successor_registry,
    validate_successor_semantic_request, SuccessorCompiledSemanticProfile,
    SuccessorProfileContract, CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
    CSHARP_PRACTICAL_SELECTION_SCHEMA, FOUNDATION_DESCRIPTOR_CONTENT_SHA256,
    FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA, SUCCESSOR_CANDIDATE_REVISION,
    SUCCESSOR_CONTRACT_FIELDS, SUCCESSOR_PROFILE_ORDER, SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
    SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA, SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
    SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
};
use mpk_vc::csharp_practical_source_artifacts::{
    bind_closed_instances, bind_practical_artifact_context, build_boundary_input_capture,
    build_boundary_output_capture, build_certificate_source_manifest,
    build_concrete_operation_tables, build_frontend_source_artifacts,
    build_frontend_source_manifest, build_practical_source_map, build_semantic_bindings,
    canonical_practical_json_bytes, canonical_source_declaration_id,
    canonical_source_stored_member_id, capture_original_inputs, parse_canonical_practical_json,
    validate_artifact_ref_document, validate_boundary_input_capture,
    validate_boundary_output_capture, validate_contract_artifact, validate_expected_artifact,
    validate_selection_document, validate_semantic_bindings_document,
    validate_source_location_document, ArtifactRef, FrontendManifestArtifacts,
    FrontendSourceArtifactLinks, OriginalInput, OriginalInputKind, PracticalArtifactContext,
    PracticalArtifactErrorCode, PracticalArtifactKind, PracticalJsonValue, SemanticArmMapping,
    SemanticBindingInput, SemanticBindingMember, SemanticOperationMapping,
    SourceDeclarationIdentity, SourceDeclarationKind, SourceMapDeclaration, SourceMapIdentity,
    SourceStoredMemberIdentity, SourceStoredMemberStorage, ValidatedPracticalArtifact,
    BOUNDARY_CONTRACT_HASH_DOMAIN, BOUNDARY_CONTRACT_SCHEMA, CERTIFICATE_SOURCE_MANIFEST_SCHEMA,
    FRONTEND_SOURCE_MANIFEST_SCHEMA, METHOD_CONTRACT_HASH_DOMAIN, METHOD_CONTRACT_SCHEMA,
    PRACTICAL_SOURCE_ARTIFACT_HASH_DOMAINS, PRACTICAL_SOURCE_ARTIFACT_IDENTITIES,
    SEMANTIC_BINDINGS_SCHEMA, SEMANTIC_BINDING_SET_HASH_DOMAIN, SOURCE_ARTIFACTS_SCHEMA,
    SOURCE_MAP_SCHEMA, SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA, SUCCESSOR_VC_SCHEMA,
    SUCCESSOR_VIR_SCHEMA, TRANSITION_CONTRACT_HASH_DOMAIN, TRANSITION_CONTRACT_SCHEMA,
    TYPE_CONTRACT_HASH_DOMAIN, TYPE_CONTRACT_SCHEMA,
};
use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, derive_closed_instances,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_closed_root_set, validate_registered_foundation_bundle, ClosedInstanceSet,
    ClosedOperationSignature, ClosedOperationTag, RequiredCheck, RequiredCheckTag,
    ValidatedClosedRootSet, ValidatedFoundationBundle,
};
use mpk_vc::source_manifest::{input_set_hash, InputEntry};
use mpk_vc::source_map::InputKind;
use mpk_vc::{canonical_json_bytes, hash_domain_separated_raw, HashDomain, StrictJsonValue};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const PACKAGE: &str =
    include_str!("../../../develop/specs/vectors/csharp-practical-profile-v1.json");
const FOUNDATION_PACKAGE: &str =
    include_str!("../../../develop/specs/vectors/csharp-practical-foundation-v1.json");
const WORK_ITEM: &str = "CSHARP-03-T02-W04";
const OWNER: &str = "crates/mpk-vc/tests/csharp_practical_source_artifacts.rs#CSHARP-03-T02-W04";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SOURCE: &[u8] = b"public static class Order { public static int Run() => 1; }\n";

#[test]
fn csharp_03_t02_w04_executes_every_frozen_schema_vector() {
    let fixture = fixture();
    let package: Value = serde_json::from_str(PACKAGE).expect("profile package");
    let vectors = package["vectors"]
        .as_array()
        .expect("vectors")
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
        .collect::<Vec<_>>();
    assert_eq!(vectors.len(), 16);
    assert!(vectors
        .iter()
        .all(|vector| vector["production_test_owner"] == OWNER));

    let semantic = fixture.semantic_bindings.value().clone();
    let artifact_ref = fixture.vir.value();
    let location = PracticalJsonValue::object(vec![
        ("source_file_ordinal", PracticalJsonValue::U64(0)),
        ("start_byte", PracticalJsonValue::U64(0)),
        ("end_byte", PracticalJsonValue::U64(1)),
    ]);
    let mut actual = BTreeMap::new();
    for vector in &vectors {
        let id = vector["id"].as_str().expect("vector id");
        let result = if id.contains("csharp_semantic_bindings") {
            execute_shape_vector(id, &semantic, |bytes| {
                validate_semantic_bindings_document(None, None, bytes).map(|_| ())
            })
        } else if id.contains("artifact_ref") {
            execute_shape_vector(id, &artifact_ref, validate_artifact_ref_document)
        } else if id.contains("source_location") {
            execute_shape_vector(id, &location, |bytes| {
                validate_source_location_document(bytes).map(|_| ())
            })
        } else {
            panic!("unexpected W04 vector {id}")
        };
        actual.insert(id.to_owned(), result);
    }
    for vector in vectors {
        let id = vector["id"].as_str().expect("vector id");
        assert_eq!(actual[id], vector["expected"], "{id}");
    }
}

#[test]
fn csharp_03_t02_w04_builds_a_complete_context_bound_artifact_graph() {
    let fixture = fixture();
    assert_eq!(fixture.semantic_bindings.schema(), SEMANTIC_BINDINGS_SCHEMA);
    let binding = &fixture
        .semantic_bindings
        .value()
        .get("bindings")
        .and_then(PracticalJsonValue::as_array)
        .unwrap()[0];
    assert_eq!(
        binding
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "schema",
            "source_type_id",
            "source_content_sha256",
            "role",
            "member_map",
            "tag_arms",
            "inferred_argument_ids",
            "default_arm",
            "bounds",
            "operation_map",
            "binding_sha256",
        ]
    );
    assert_eq!(
        binding
            .get("member_map")
            .and_then(PracticalJsonValue::as_object)
            .unwrap()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["tag", "value"]
    );
    assert_eq!(fixture.source_map.schema(), SOURCE_MAP_SCHEMA);
    let source_map_entry = &fixture
        .source_map
        .value()
        .get("entries")
        .and_then(PracticalJsonValue::as_array)
        .unwrap()[0];
    assert_eq!(
        source_map_entry
            .get("declaration_id")
            .and_then(PracticalJsonValue::as_str),
        Some("mpk.csharp.source.9d47fef8d2187ff4509c3d060ed838b1d353aa31be1e7fe6d74f6ea0386488f9")
    );
    assert_eq!(
        source_map_entry
            .get("provenance_sha256")
            .and_then(PracticalJsonValue::as_str),
        Some("c4640da3e16e96d1f159ce11e6456c1925cf52efaea28fe61802cc7b54755997")
    );
    assert_eq!(
        fixture.frontend_manifest.schema(),
        FRONTEND_SOURCE_MANIFEST_SCHEMA
    );
    let manifest_inputs = fixture
        .frontend_manifest
        .value()
        .get("inputs")
        .and_then(PracticalJsonValue::as_array)
        .expect("manifest inputs");
    let retained_inputs = manifest_inputs
        .iter()
        .map(|input| {
            assert_eq!(
                input
                    .as_object()
                    .expect("input row")
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>(),
                vec!["kind", "normalized_path", "size_bytes", "sha256"]
            );
            InputEntry {
                kind: match input.get("kind").and_then(PracticalJsonValue::as_str) {
                    Some("source") => InputKind::Source,
                    Some("contract") => InputKind::Contract,
                    other => panic!("unexpected retained input kind {other:?}"),
                },
                normalized_path: input
                    .get("normalized_path")
                    .and_then(PracticalJsonValue::as_str)
                    .expect("normalized input path")
                    .to_owned(),
                size_bytes: i64::try_from(
                    input
                        .get("size_bytes")
                        .and_then(PracticalJsonValue::as_u64)
                        .expect("input byte size"),
                )
                .expect("u32 input size fits i64"),
                sha256: input
                    .get("sha256")
                    .and_then(PracticalJsonValue::as_str)
                    .expect("input hash")
                    .to_owned(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        retained_inputs
            .iter()
            .map(|input| input.kind)
            .collect::<Vec<_>>(),
        vec![InputKind::Contract, InputKind::Source]
    );
    assert_eq!(
        retained_inputs
            .iter()
            .map(|input| input.normalized_path.as_str())
            .collect::<Vec<_>>(),
        vec!["contracts/order.json", "src/Order.cs"]
    );
    let recomputed_input_set = input_set_hash(&retained_inputs).expect("retained input-set hash");
    assert_eq!(
        fixture
            .frontend_manifest
            .value()
            .get("input_set_sha256")
            .and_then(PracticalJsonValue::as_str),
        Some(recomputed_input_set.as_str())
    );
    assert_eq!(
        fixture.certificate_manifest.schema(),
        CERTIFICATE_SOURCE_MANIFEST_SCHEMA
    );
    assert_eq!(fixture.source_artifacts.schema(), SOURCE_ARTIFACTS_SCHEMA);
    assert!(fixture
        .source_artifacts
        .value()
        .get("foundation_descriptor")
        .is_some());
    assert_eq!(
        fixture.source_artifacts.value().get("semantic_context"),
        Some(fixture.context.semantic_context())
    );
    assert_eq!(
        fixture
            .source_artifacts
            .value()
            .get("selection_sha256")
            .and_then(PracticalJsonValue::as_str),
        Some(fixture.context.selection_sha256())
    );
    assert_eq!(
        fixture.operations.operations().schema(),
        "mpk.csharp.operations.v1"
    );
    assert_eq!(
        fixture.operations.required_checks().schema(),
        "mpk.csharp.required_checks.v1"
    );
    assert_eq!(
        fixture
            .closed
            .entries()
            .iter()
            .flat_map(|entry| entry["operation_definitions"].as_array().unwrap())
            .count(),
        fixture
            .operations
            .operations()
            .value()
            .get("operations")
            .and_then(PracticalJsonValue::as_array)
            .unwrap()
            .len()
    );

    validate_expected_artifact(
        &fixture.source_artifacts,
        fixture.source_artifacts.canonical_bytes(),
    )
    .expect("source-artifact round trip");
    validate_boundary_input_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        "boundary.request.0",
        fixture.boundary_input.raw_bytes(),
        fixture.boundary_input.canonical_document(),
        fixture.boundary_input.artifact().canonical_bytes(),
    )
    .expect("input bytes/value linkage");
    validate_boundary_output_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        boundary_value("42"),
        fixture.boundary_output.canonical_document(),
        fixture.boundary_output.artifact().canonical_bytes(),
    )
    .expect("output/value/reparse linkage");
}

#[test]
fn csharp_03_t02_w04_rejects_field_hash_context_and_schema_splicing() {
    let fixture = fixture();
    let mut reordered = fixture.boundary_contract.value().clone();
    if let PracticalJsonValue::Object(entries) = &mut reordered {
        entries.swap(0, 1);
    }
    let bytes = canonical_practical_json_bytes(&reordered).expect("reordered bytes");
    assert_code(
        validate_contract_artifact(
            &fixture.context,
            &fixture.captures,
            PracticalArtifactKind::BoundaryContract,
            &bytes,
        ),
        PracticalArtifactErrorCode::FieldOrder,
    );

    let mut unknown = fixture.method_contract.value().clone();
    if let PracticalJsonValue::Object(entries) = &mut unknown {
        entries.push(("unknown".to_owned(), PracticalJsonValue::Null));
    }
    let bytes = canonical_practical_json_bytes(&unknown).expect("unknown bytes");
    assert_code(
        validate_contract_artifact(
            &fixture.context,
            &fixture.captures,
            PracticalArtifactKind::MethodContract,
            &bytes,
        ),
        PracticalArtifactErrorCode::Shape,
    );

    let mut substituted_hash = fixture.type_contract.value().clone();
    set_field(
        &mut substituted_hash,
        "contract_sha256",
        PracticalJsonValue::string(fixture.method_contract.hash()),
    );
    let bytes = canonical_practical_json_bytes(&substituted_hash).expect("substitution bytes");
    assert_code(
        validate_contract_artifact(
            &fixture.context,
            &fixture.captures,
            PracticalArtifactKind::TypeContract,
            &bytes,
        ),
        PracticalArtifactErrorCode::Hash,
    );

    let other = fixture_with_compilation("business.other");
    assert_code(
        validate_contract_artifact(
            &other.context,
            &other.captures,
            PracticalArtifactKind::MethodContract,
            fixture.method_contract.canonical_bytes(),
        ),
        PracticalArtifactErrorCode::Compilation,
    );
    assert_code(
        build_frontend_source_artifacts(
            &other.context,
            &other.foundation,
            FrontendSourceArtifactLinks {
                vir: &fixture.vir,
                source_map: &fixture.source_map.artifact_ref(),
                source_manifest: &fixture.frontend_manifest,
                semantic_bindings: &fixture.semantic_bindings.artifact_ref(),
                closed_instances: &fixture.closed_ref,
                boundary_contracts: vec![fixture.boundary_contract.artifact_ref()],
                transition_contracts: vec![fixture.transition_contract.artifact_ref()],
            },
        ),
        PracticalArtifactErrorCode::Linkage,
    );

    let alternate_captures = capture_original_inputs(
        &fixture.context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".to_owned(),
                bytes: [SOURCE, b"// changed\n"].concat(),
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/order.json".to_owned(),
                bytes: b"{}".to_vec(),
            },
        ],
    )
    .unwrap();
    assert_code(
        build_frontend_source_manifest(
            &fixture.context,
            &fixture.foundation,
            &alternate_captures,
            frontend_manifest_artifacts(&fixture),
        ),
        PracticalArtifactErrorCode::Linkage,
    );

    let alternate_vir = ArtifactRef::opaque_successor(
        &fixture.context,
        SUCCESSOR_VIR_SCHEMA,
        &"e".repeat(64),
        1_024,
    )
    .unwrap();
    let mut alternate_vir_artifacts = frontend_manifest_artifacts(&fixture);
    alternate_vir_artifacts.vir = alternate_vir;
    assert_code(
        build_frontend_source_manifest(
            &fixture.context,
            &fixture.foundation,
            &fixture.captures,
            alternate_vir_artifacts,
        ),
        PracticalArtifactErrorCode::Linkage,
    );

    let mut alternate_boundary_value = fixture.boundary_contract.value().clone();
    set_field(
        &mut alternate_boundary_value,
        "boundary_id",
        PracticalJsonValue::string("boundary.order.other"),
    );
    let alternate_boundary_bytes = rehash_document(
        alternate_boundary_value,
        BOUNDARY_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
    );
    let alternate_boundary = validate_contract_artifact(
        &fixture.context,
        &fixture.captures,
        PracticalArtifactKind::BoundaryContract,
        &alternate_boundary_bytes,
    )
    .unwrap();
    let mut alternate_boundary_artifacts = frontend_manifest_artifacts(&fixture);
    alternate_boundary_artifacts.boundary_contracts = vec![alternate_boundary.artifact_ref()];
    assert_code(
        build_frontend_source_manifest(
            &fixture.context,
            &fixture.foundation,
            &fixture.captures,
            alternate_boundary_artifacts,
        ),
        PracticalArtifactErrorCode::Linkage,
    );

    let mut duplicate = br#"{"schema":"mpk.csharp.semantic_bindings.v1","schema":"mpk.csharp.semantic_bindings.v1"}"#.to_vec();
    assert_code(
        validate_semantic_bindings_document(None, None, &duplicate),
        PracticalArtifactErrorCode::DuplicateField,
    );
    duplicate.clear();

    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(
        &canonical_successor_registry_transport(&registry_value).unwrap(),
    )
    .unwrap();
    let scalar_request = request_fixture(
        scalar_context_fixture(&registry_value),
        json!({
            "schema": "mpk.selection.csharp_methods.v0",
            "value": {
                "compilation": "legacy.core",
                "contracts": ["contracts/legacy.json"],
                "methods": ["Legacy.Run"],
                "sources": ["src/Legacy.cs"]
            }
        }),
    );
    let scalar = validate_successor_semantic_request(&registry, &canonical(&scalar_request))
        .expect("valid scalar successor request");
    assert_code(
        bind_practical_artifact_context(&scalar, &fixture.foundation),
        PracticalArtifactErrorCode::Context,
    );
}

#[test]
fn csharp_03_t02_w04_accounts_for_inputs_declarations_spans_and_operations_once() {
    let fixture = fixture();
    assert_code(
        capture_original_inputs(
            &fixture.context,
            vec![OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".to_owned(),
                bytes: SOURCE.to_vec(),
            }],
        ),
        PracticalArtifactErrorCode::SourceInventory,
    );
    assert_code(
        capture_original_inputs(
            &fixture.context,
            vec![
                OriginalInput {
                    kind: OriginalInputKind::Source,
                    path: "src/Order.cs".to_owned(),
                    bytes: SOURCE.to_vec(),
                },
                OriginalInput {
                    kind: OriginalInputKind::Source,
                    path: "src/Order.cs".to_owned(),
                    bytes: SOURCE.to_vec(),
                },
                OriginalInput {
                    kind: OriginalInputKind::Sidecar,
                    path: "contracts/order.json".to_owned(),
                    bytes: b"{}".to_vec(),
                },
            ],
        ),
        PracticalArtifactErrorCode::SourceInventory,
    );

    let sidecar_hash = fixture
        .captures
        .entry("contracts/order.json")
        .unwrap()
        .raw_sha256()
        .to_owned();
    assert_code(
        validate_contract_artifact(
            &fixture.context,
            &fixture.captures,
            PracticalArtifactKind::TypeContract,
            &type_contract_bytes(&fixture.context, &sidecar_hash),
        ),
        PracticalArtifactErrorCode::SourceInventory,
    );
    assert_code(
        build_semantic_bindings(
            &fixture.context,
            &fixture.captures,
            vec![semantic_binding_input(
                sidecar_hash,
                fixture.closed.entries()[0]["instance_id"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            )],
        ),
        PracticalArtifactErrorCode::SourceInventory,
    );

    let source_hash = fixture
        .captures
        .entry("src/Order.cs")
        .unwrap()
        .raw_sha256()
        .to_owned();
    let dependency_id = fixture.closed.entries()[0]["instance_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut wrong_member = semantic_binding_input(source_hash.clone(), dependency_id.clone());
    wrong_member.member_map[0].role = "payload".to_owned();
    assert_code(
        build_semantic_bindings(&fixture.context, &fixture.captures, vec![wrong_member]),
        PracticalArtifactErrorCode::Shape,
    );
    let mut noncanonical_member =
        semantic_binding_input(source_hash.clone(), dependency_id.clone());
    noncanonical_member.member_map[0].member_id = "app.order.option.tag".to_owned();
    assert_code(
        build_semantic_bindings(
            &fixture.context,
            &fixture.captures,
            vec![noncanonical_member],
        ),
        PracticalArtifactErrorCode::Shape,
    );
    let mut noncanonical_argument =
        semantic_binding_input(source_hash.clone(), dependency_id.clone());
    noncanonical_argument.inferred_argument_ids[0] = "System.Int32".to_owned();
    assert_code(
        build_semantic_bindings(
            &fixture.context,
            &fixture.captures,
            vec![noncanonical_argument],
        ),
        PracticalArtifactErrorCode::Shape,
    );
    let mut noncanonical_operation =
        semantic_binding_input(source_hash.clone(), dependency_id.clone());
    noncanonical_operation
        .operation_map
        .push(SemanticOperationMapping {
            operation: "has_value".to_owned(),
            member_id: "OrderOption.HasValue".to_owned(),
        });
    assert_code(
        build_semantic_bindings(
            &fixture.context,
            &fixture.captures,
            vec![noncanonical_operation],
        ),
        PracticalArtifactErrorCode::Shape,
    );
    let mut wrong_tag = semantic_binding_input(source_hash.clone(), dependency_id.clone());
    wrong_tag.tag_arms[0].source_tag = "+0".to_owned();
    assert_code(
        build_semantic_bindings(&fixture.context, &fixture.captures, vec![wrong_tag]),
        PracticalArtifactErrorCode::Shape,
    );
    let mut wrong_default = semantic_binding_input(source_hash, dependency_id);
    wrong_default.default_arm = "some".to_owned();
    assert_code(
        build_semantic_bindings(&fixture.context, &fixture.captures, vec![wrong_default]),
        PracticalArtifactErrorCode::Shape,
    );

    let mut noncanonical_compilation = fixture.semantic_bindings.value().clone();
    set_field(
        &mut noncanonical_compilation,
        "compilation_id",
        PracticalJsonValue::string("Business..Core"),
    );
    let noncanonical_compilation = rehash_document(
        noncanonical_compilation,
        SEMANTIC_BINDING_SET_HASH_DOMAIN,
        "binding_set_sha256",
    );
    assert_code(
        validate_semantic_bindings_document(None, None, &noncanonical_compilation),
        PracticalArtifactErrorCode::Shape,
    );

    let member_owner = source_type_id("Order");
    let amount_member = stored_i32_member_id(&member_owner, "Amount");
    let count_member = stored_i32_member_id(&member_owner, "Count");
    let mut repeated_member = fixture.type_contract.value().clone();
    set_field(
        &mut repeated_member,
        "ordered_member_ids",
        PracticalJsonValue::Array(vec![
            PracticalJsonValue::string(&amount_member),
            PracticalJsonValue::string(count_member),
            PracticalJsonValue::string(amount_member),
        ]),
    );
    let repeated_member = rehash_document(
        repeated_member,
        TYPE_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
    );
    assert_code(
        validate_contract_artifact(
            &fixture.context,
            &fixture.captures,
            PracticalArtifactKind::TypeContract,
            &repeated_member,
        ),
        PracticalArtifactErrorCode::Shape,
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::TypeContract,
        &fixture.type_contract,
        TYPE_CONTRACT_HASH_DOMAIN,
        "source_type_id",
        PracticalJsonValue::string("app.order"),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::TypeContract,
        &fixture.type_contract,
        TYPE_CONTRACT_HASH_DOMAIN,
        "ordered_member_ids",
        PracticalJsonValue::Array(vec![PracticalJsonValue::string("app.order.amount")]),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::MethodContract,
        &fixture.method_contract,
        METHOD_CONTRACT_HASH_DOMAIN,
        "callable_id",
        PracticalJsonValue::string("app.order.run"),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::BoundaryContract,
        &fixture.boundary_contract,
        BOUNDARY_CONTRACT_HASH_DOMAIN,
        "selected_callable_id",
        PracticalJsonValue::string("app.order.run"),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::BoundaryContract,
        &fixture.boundary_contract,
        BOUNDARY_CONTRACT_HASH_DOMAIN,
        "boundary_id",
        PracticalJsonValue::string("boundary..order"),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::TransitionContract,
        &fixture.transition_contract,
        TRANSITION_CONTRACT_HASH_DOMAIN,
        "state_type_id",
        PracticalJsonValue::string("app.state"),
    );
    assert_contract_field_shape_rejected(
        &fixture,
        PracticalArtifactKind::TransitionContract,
        &fixture.transition_contract,
        TRANSITION_CONTRACT_HASH_DOMAIN,
        "transition_binding_id",
        PracticalJsonValue::string("binding..transition"),
    );

    let root_id = fixture.context.selected_root_ids()[0].clone();
    assert_eq!(
        root_id,
        canonical_source_declaration_id(&declaration_identity("Run")).unwrap()
    );
    assert_eq!(
        canonical_source_stored_member_id(&SourceStoredMemberIdentity {
            owner_source_type_id: format!("mpk.csharp.source.{}", "0".repeat(64)),
            source_name: "Value".to_owned(),
            closed_type: json!({"kind": "primitive", "id": "i32"}),
            storage: SourceStoredMemberStorage::ReadonlyField,
        })
        .unwrap(),
        "mpk.csharp.member.70949e9f2f0776ef00f964ba1ee8ec7edd89576bd050676ec4b35a3c3fe6933e"
    );
    let mut invalid_identity = declaration_identity("Run.Method");
    assert_code(
        canonical_source_declaration_id(&invalid_identity),
        PracticalArtifactErrorCode::Shape,
    );
    invalid_identity.source_name = "Run".to_owned();
    invalid_identity.namespace = "Business..Orders".to_owned();
    assert_code(
        canonical_source_declaration_id(&invalid_identity),
        PracticalArtifactErrorCode::Shape,
    );
    let mut invalid_owner = declaration_identity("Run");
    invalid_owner.containing_source_type_id = Some("app.order".to_owned());
    assert_code(
        canonical_source_declaration_id(&invalid_owner),
        PracticalArtifactErrorCode::Shape,
    );
    let mut invalid_result = declaration_identity("Run");
    invalid_result.result_type_id = Some("System.Int32".to_owned());
    assert_code(
        canonical_source_declaration_id(&invalid_result),
        PracticalArtifactErrorCode::Shape,
    );
    let mut invalid_parameter = declaration_identity("Run");
    invalid_parameter
        .parameter_type_ids
        .push("System.Int32".to_owned());
    assert_code(
        canonical_source_declaration_id(&invalid_parameter),
        PracticalArtifactErrorCode::Shape,
    );
    assert_code(
        canonical_source_stored_member_id(&SourceStoredMemberIdentity {
            owner_source_type_id: "app.order".to_owned(),
            source_name: "Value".to_owned(),
            closed_type: json!({"kind": "primitive", "id": "i32"}),
            storage: SourceStoredMemberStorage::ReadonlyField,
        }),
        PracticalArtifactErrorCode::Shape,
    );
    let invalid_provenance = SourceMapDeclaration {
        declaration_id: root_id.clone(),
        identity: SourceMapIdentity::Declaration(declaration_identity("Run")),
        provenance_id: "Source.order.run".to_owned(),
        source_path: "src/Order.cs".to_owned(),
        start_byte: 0,
        end_byte: u32::try_from(SOURCE.len()).unwrap(),
        artifact_node_ids: vec!["vir.node.0000".to_owned()],
    };
    assert_code(
        build_practical_source_map(
            &fixture.context,
            &fixture.captures,
            &fixture.vir,
            vec![invalid_provenance],
        ),
        PracticalArtifactErrorCode::Shape,
    );
    let invalid_artifact_node = SourceMapDeclaration {
        declaration_id: root_id.clone(),
        identity: SourceMapIdentity::Declaration(declaration_identity("Run")),
        provenance_id: "source.order.run".to_owned(),
        source_path: "src/Order.cs".to_owned(),
        start_byte: 0,
        end_byte: u32::try_from(SOURCE.len()).unwrap(),
        artifact_node_ids: vec!["VIR.node.0000".to_owned()],
    };
    assert_code(
        build_practical_source_map(
            &fixture.context,
            &fixture.captures,
            &fixture.vir,
            vec![invalid_artifact_node],
        ),
        PracticalArtifactErrorCode::Shape,
    );
    let bad_span = SourceMapDeclaration {
        declaration_id: root_id.clone(),
        identity: SourceMapIdentity::Declaration(declaration_identity("Run")),
        provenance_id: "source.order.run".to_owned(),
        source_path: "src/Order.cs".to_owned(),
        start_byte: 0,
        end_byte: u32::try_from(SOURCE.len() + 1).unwrap(),
        artifact_node_ids: vec!["vir.node.0000".to_owned()],
    };
    assert_code(
        build_practical_source_map(
            &fixture.context,
            &fixture.captures,
            &fixture.vir,
            vec![bad_span],
        ),
        PracticalArtifactErrorCode::SourceSpan,
    );
    let missing_identity = declaration_identity("Other");
    let missing = SourceMapDeclaration {
        declaration_id: canonical_source_declaration_id(&missing_identity).unwrap(),
        identity: SourceMapIdentity::Declaration(missing_identity),
        provenance_id: "source.other.run".to_owned(),
        source_path: "src/Order.cs".to_owned(),
        start_byte: 0,
        end_byte: 1,
        artifact_node_ids: vec!["vir.node.0000".to_owned()],
    };
    assert_code(
        build_practical_source_map(
            &fixture.context,
            &fixture.captures,
            &fixture.vir,
            vec![missing],
        ),
        PracticalArtifactErrorCode::MissingMember,
    );

    assert_code(
        build_concrete_operation_tables(
            &fixture.context,
            &fixture.roots,
            &fixture.closed,
            &fixture.closed_ref,
            Vec::new(),
        ),
        PracticalArtifactErrorCode::MissingMember,
    );
    let mut duplicated = foundation_signatures(&fixture.closed);
    duplicated.push(duplicated[0].clone());
    assert_code(
        build_concrete_operation_tables(
            &fixture.context,
            &fixture.roots,
            &fixture.closed,
            &fixture.closed_ref,
            duplicated,
        ),
        PracticalArtifactErrorCode::DuplicateMember,
    );
}

#[test]
fn csharp_03_t02_w04_rejects_boundary_byte_and_value_substitution() {
    let fixture = fixture();
    assert_code(
        build_boundary_input_capture(
            &fixture.context,
            &fixture.boundary_contract.artifact_ref(),
            "boundary..request",
            fixture.boundary_input.raw_bytes(),
            fixture.boundary_input.canonical_document(),
        ),
        PracticalArtifactErrorCode::BoundaryBytes,
    );
    let different = canonical_practical_json_bytes(&boundary_value("43")).expect("value bytes");
    assert_code(
        validate_boundary_input_capture(
            &fixture.context,
            &fixture.boundary_contract.artifact_ref(),
            "boundary.request.0",
            fixture.boundary_input.raw_bytes(),
            &different,
            fixture.boundary_input.artifact().canonical_bytes(),
        ),
        PracticalArtifactErrorCode::Linkage,
    );
    assert_code(
        validate_boundary_input_capture(
            &fixture.context,
            &fixture.boundary_contract.artifact_ref(),
            "boundary.request.0",
            b"adapter:43",
            fixture.boundary_input.canonical_document(),
            fixture.boundary_input.artifact().canonical_bytes(),
        ),
        PracticalArtifactErrorCode::Linkage,
    );
    assert_code(
        validate_boundary_output_capture(
            &fixture.context,
            &fixture.boundary_contract.artifact_ref(),
            boundary_value("43"),
            fixture.boundary_output.canonical_document(),
            fixture.boundary_output.artifact().canonical_bytes(),
        ),
        PracticalArtifactErrorCode::BoundaryBytes,
    );

    let noncanonical = b"{ \"amount\":\"42\" }";
    assert!(
        parse_canonical_practical_json(PracticalArtifactKind::BoundaryInput, noncanonical).is_err()
    );
    assert!(build_boundary_input_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        "boundary.request.noncanonical",
        b"adapter:42",
        noncanonical,
    )
    .is_err());

    let utf16_value = PracticalJsonValue::object(vec![(
        "text",
        PracticalJsonValue::utf16_string(vec![0xd800, 0x0078, 0xdc00]),
    )]);
    let utf16_document = canonical_practical_json_bytes(&utf16_value).unwrap();
    let utf16_input = build_boundary_input_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        "boundary.request.utf16",
        b"adapter:utf16",
        &utf16_document,
    )
    .expect("UTF-16 boundary input");
    validate_boundary_input_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        "boundary.request.utf16",
        b"adapter:utf16",
        &utf16_document,
        utf16_input.artifact().canonical_bytes(),
    )
    .expect("UTF-16 input evidence");
    let utf16_output = build_boundary_output_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        utf16_value.clone(),
    )
    .expect("UTF-16 boundary output");
    validate_boundary_output_capture(
        &fixture.context,
        &fixture.boundary_contract.artifact_ref(),
        utf16_value,
        &utf16_document,
        utf16_output.artifact().canonical_bytes(),
    )
    .expect("UTF-16 output evidence");
}

#[test]
fn csharp_03_t02_w04_uses_only_frozen_identities_domains_and_canonical_tokens() {
    let package: Value = serde_json::from_str(PACKAGE).expect("profile package");
    let frozen = &package["frozen_contract"];
    let identities = frozen["identity_families"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|family| family["successor_identities"].as_array().unwrap())
        .map(|value| value.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let domains = frozen["identity_families"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|family| {
            family["successor_hash_domains"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap())
                .chain(
                    family["retained_hash_domains"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .or_else(|| value.get("id").and_then(Value::as_str))
                                .unwrap()
                        }),
                )
        })
        .collect::<BTreeSet<_>>();
    assert!(PRACTICAL_SOURCE_ARTIFACT_IDENTITIES
        .iter()
        .all(|identity| identities.contains(identity)));
    assert!(PRACTICAL_SOURCE_ARTIFACT_HASH_DOMAINS
        .iter()
        .all(|domain| domains.contains(domain)));
    assert_eq!(
        PRACTICAL_SOURCE_ARTIFACT_HASH_DOMAINS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        PRACTICAL_SOURCE_ARTIFACT_HASH_DOMAINS.len()
    );

    let foundation: Value = serde_json::from_str(FOUNDATION_PACKAGE).expect("foundation package");
    let published_bindings = foundation["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|vector| {
            vector["family"] == "binding"
                && vector["inputs"]
                    .get("binding")
                    .is_some_and(Value::is_object)
        })
        .collect::<Vec<_>>();
    assert_eq!(published_bindings.len(), 12);
    for vector in published_bindings {
        let binding = published_binding_value(&vector["inputs"]["binding"]);
        let document = hashed_document(
            SEMANTIC_BINDING_SET_HASH_DOMAIN,
            "binding_set_sha256",
            vec![
                (
                    "schema",
                    PracticalJsonValue::string(SEMANTIC_BINDINGS_SCHEMA),
                ),
                ("semantic_context", PracticalJsonValue::Object(Vec::new())),
                (
                    "compilation_id",
                    PracticalJsonValue::string("business.core"),
                ),
                ("bindings", PracticalJsonValue::Array(vec![binding])),
            ],
        );
        validate_semantic_bindings_document(None, None, &document)
            .unwrap_or_else(|error| panic!("{}: {error}", vector["id"]));
    }

    let maximum =
        PracticalJsonValue::object(vec![("canonical_bytes", PracticalJsonValue::U64(u64::MAX))]);
    let bytes = canonical_practical_json_bytes(&maximum).unwrap();
    assert_eq!(bytes, br#"{"canonical_bytes":18446744073709551615}"#);
    assert_eq!(
        parse_canonical_practical_json(PracticalArtifactKind::ArtifactRef, &bytes).unwrap(),
        maximum
    );
    assert!(parse_canonical_practical_json(
        PracticalArtifactKind::ArtifactRef,
        br#"{"text":"\n"}"#,
    )
    .is_err());
    assert_eq!(
        canonical_practical_json_bytes(&PracticalJsonValue::object(vec![(
            "text",
            PracticalJsonValue::string("\n"),
        )]))
        .unwrap(),
        br#"{"text":"\u000a"}"#
    );
    let lone_surrogates = br#"{"text":"\u0001\ud800x\udc00"}"#;
    let parsed =
        parse_canonical_practical_json(PracticalArtifactKind::BoundaryInput, lone_surrogates)
            .expect("lone UTF-16 surrogates");
    assert_eq!(
        parsed.get("text"),
        Some(&PracticalJsonValue::utf16_string(vec![
            0x0001, 0xd800, 0x0078, 0xdc00,
        ]))
    );
    assert_eq!(
        canonical_practical_json_bytes(&parsed).unwrap(),
        lone_surrogates
    );
    assert!(parse_canonical_practical_json(
        PracticalArtifactKind::BoundaryInput,
        br#"{"text":"\ud83d\ude00"}"#,
    )
    .is_err());
    assert!(
        canonical_practical_json_bytes(&PracticalJsonValue::string("x".repeat(
            mpk_vc::csharp_practical_source_artifacts::PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX + 1,
        )))
        .is_err()
    );
}

struct Fixture {
    context: PracticalArtifactContext,
    foundation: ValidatedFoundationBundle,
    roots: ValidatedClosedRootSet,
    closed: ClosedInstanceSet,
    closed_ref: ArtifactRef,
    captures: mpk_vc::csharp_practical_source_artifacts::CapturedInputSet,
    semantic_bindings: ValidatedPracticalArtifact,
    type_contract: ValidatedPracticalArtifact,
    method_contract: ValidatedPracticalArtifact,
    boundary_contract: ValidatedPracticalArtifact,
    transition_contract: ValidatedPracticalArtifact,
    operations: mpk_vc::csharp_practical_source_artifacts::ConcreteOperationTables,
    vir: ArtifactRef,
    source_map: ValidatedPracticalArtifact,
    boundary_input: mpk_vc::csharp_practical_source_artifacts::BoundaryInputCapture,
    boundary_output: mpk_vc::csharp_practical_source_artifacts::BoundaryOutputCapture,
    frontend_manifest: ValidatedPracticalArtifact,
    certificate_manifest: ValidatedPracticalArtifact,
    source_artifacts: ValidatedPracticalArtifact,
}

fn frontend_manifest_artifacts(fixture: &Fixture) -> FrontendManifestArtifacts {
    FrontendManifestArtifacts {
        type_contracts: vec![fixture.type_contract.artifact_ref()],
        method_contracts: vec![fixture.method_contract.artifact_ref()],
        semantic_bindings: fixture.semantic_bindings.artifact_ref(),
        boundary_contracts: vec![fixture.boundary_contract.artifact_ref()],
        boundary_inputs: vec![fixture.boundary_input.artifact().artifact_ref()],
        boundary_outputs: vec![fixture.boundary_output.artifact().artifact_ref()],
        transition_contracts: vec![fixture.transition_contract.artifact_ref()],
        closed_instances: fixture.closed_ref.clone(),
        operations: fixture.operations.operations().artifact_ref(),
        required_checks: fixture.operations.required_checks().artifact_ref(),
        vir: fixture.vir.clone(),
        source_map: fixture.source_map.artifact_ref(),
    }
}

fn fixture() -> Fixture {
    fixture_with_compilation("business.core")
}

fn fixture_with_compilation(compilation_id: &str) -> Fixture {
    let registry_value = candidate_registry();
    let registry_transport =
        canonical_successor_registry_transport(&registry_value).expect("registry transport");
    let registry = validate_candidate_successor_registry(&registry_transport).expect("registry");
    let context_value = context_fixture(&registry_value);
    let selection = practical_selection(compilation_id);
    let request_value = request_fixture(context_value, selection);
    let request = validate_successor_semantic_request(&registry, &canonical(&request_value))
        .expect("validated request");
    let foundation = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .expect("registered foundation");
    let context = bind_practical_artifact_context(&request, &foundation).expect("artifact context");
    let roots_value = json!([{
        "origin": "source_nullable",
        "provenance_id": "root.source.option_i32",
        "type": {
            "kind": "instance",
            "template": "option",
            "arguments": [{"kind": "primitive", "id": "i32"}]
        }
    }]);
    let root_transport = canonical_closed_root_set_transport(&foundation, &roots_value, &json!({}))
        .expect("root transport");
    let roots = validate_closed_root_set(&foundation, &root_transport).expect("roots");
    let closed = derive_closed_instances(&foundation, &roots).expect("closed instances");
    let captures = capture_original_inputs(
        &context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".to_owned(),
                bytes: SOURCE.to_vec(),
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/order.json".to_owned(),
                bytes: b"{}".to_vec(),
            },
        ],
    )
    .expect("captured inputs");
    let closed_ref = bind_closed_instances(&context, &foundation, &captures, &roots, &closed)
        .expect("closed ref");
    let source_hash = captures
        .entry("src/Order.cs")
        .unwrap()
        .raw_sha256()
        .to_owned();
    let semantic_bindings = build_semantic_bindings(
        &context,
        &captures,
        vec![semantic_binding_input(
            source_hash.clone(),
            closed.entries()[0]["instance_id"]
                .as_str()
                .unwrap()
                .to_owned(),
        )],
    )
    .expect("semantic bindings");
    validate_semantic_bindings_document(
        Some(&context),
        Some(&captures),
        semantic_bindings.canonical_bytes(),
    )
    .expect("semantic-binding import");

    let type_contract = validate_contract_artifact(
        &context,
        &captures,
        PracticalArtifactKind::TypeContract,
        &type_contract_bytes(&context, &source_hash),
    )
    .expect("type contract");
    let method_contract = validate_contract_artifact(
        &context,
        &captures,
        PracticalArtifactKind::MethodContract,
        &method_contract_bytes(&context, &source_hash),
    )
    .expect("method contract");
    let boundary_contract = validate_contract_artifact(
        &context,
        &captures,
        PracticalArtifactKind::BoundaryContract,
        &boundary_contract_bytes(&context),
    )
    .expect("boundary contract");
    let transition_contract = validate_contract_artifact(
        &context,
        &captures,
        PracticalArtifactKind::TransitionContract,
        &transition_contract_bytes(&context),
    )
    .expect("transition contract");
    let operations = build_concrete_operation_tables(
        &context,
        &roots,
        &closed,
        &closed_ref,
        foundation_signatures(&closed),
    )
    .expect("operation tables");
    let vir = ArtifactRef::opaque_successor(&context, SUCCESSOR_VIR_SCHEMA, &"a".repeat(64), 1024)
        .expect("VIR ref");
    let declaration = SourceMapDeclaration {
        declaration_id: context.selected_root_ids()[0].clone(),
        identity: SourceMapIdentity::Declaration(declaration_identity("Run")),
        provenance_id: "source.order.run".to_owned(),
        source_path: "src/Order.cs".to_owned(),
        start_byte: 0,
        end_byte: u32::try_from(SOURCE.len()).unwrap(),
        artifact_node_ids: vec!["vir.node.0000".to_owned()],
    };
    let source_map = build_practical_source_map(&context, &captures, &vir, vec![declaration])
        .expect("source map");
    let input_document = canonical_practical_json_bytes(&boundary_value("42")).unwrap();
    let boundary_input = build_boundary_input_capture(
        &context,
        &boundary_contract.artifact_ref(),
        "boundary.request.0",
        b"adapter:42",
        &input_document,
    )
    .expect("boundary input");
    let boundary_output = build_boundary_output_capture(
        &context,
        &boundary_contract.artifact_ref(),
        boundary_value("42"),
    )
    .expect("boundary output");
    let frontend_manifest = build_frontend_source_manifest(
        &context,
        &foundation,
        &captures,
        FrontendManifestArtifacts {
            type_contracts: vec![type_contract.artifact_ref()],
            method_contracts: vec![method_contract.artifact_ref()],
            semantic_bindings: semantic_bindings.artifact_ref(),
            boundary_contracts: vec![boundary_contract.artifact_ref()],
            boundary_inputs: vec![boundary_input.artifact().artifact_ref()],
            boundary_outputs: vec![boundary_output.artifact().artifact_ref()],
            transition_contracts: vec![transition_contract.artifact_ref()],
            closed_instances: closed_ref.clone(),
            operations: operations.operations().artifact_ref(),
            required_checks: operations.required_checks().artifact_ref(),
            vir: vir.clone(),
            source_map: source_map.artifact_ref(),
        },
    )
    .expect("frontend manifest");
    let vc = ArtifactRef::opaque_successor(&context, SUCCESSOR_VC_SCHEMA, &"b".repeat(64), 2048)
        .expect("VC ref");
    let skeleton = ArtifactRef::opaque_successor(
        &context,
        SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA,
        &"c".repeat(64),
        4096,
    )
    .expect("skeleton ref");
    let certificate_manifest = build_certificate_source_manifest(
        &context,
        &frontend_manifest.artifact_ref(),
        &vc,
        &skeleton,
        &"d".repeat(64),
    )
    .expect("certificate manifest");
    let source_artifacts = build_frontend_source_artifacts(
        &context,
        &foundation,
        FrontendSourceArtifactLinks {
            vir: &vir,
            source_map: &source_map.artifact_ref(),
            source_manifest: &frontend_manifest,
            semantic_bindings: &semantic_bindings.artifact_ref(),
            closed_instances: &closed_ref,
            boundary_contracts: vec![boundary_contract.artifact_ref()],
            transition_contracts: vec![transition_contract.artifact_ref()],
        },
    )
    .expect("source artifacts");
    let selection_bytes = selection_bytes(&context);
    validate_selection_document(&context, &selection_bytes).expect("selection artifact");
    Fixture {
        context,
        foundation,
        roots,
        closed,
        closed_ref,
        captures,
        semantic_bindings,
        type_contract,
        method_contract,
        boundary_contract,
        transition_contract,
        operations,
        vir,
        source_map,
        boundary_input,
        boundary_output,
        frontend_manifest,
        certificate_manifest,
        source_artifacts,
    }
}

fn foundation_signatures(closed: &ClosedInstanceSet) -> Vec<ClosedOperationSignature> {
    closed
        .entries()
        .iter()
        .flat_map(|entry| entry["operation_definitions"].as_array().unwrap())
        .map(|operation| ClosedOperationSignature {
            id: operation["id"].as_str().unwrap().to_owned(),
            tag: ClosedOperationTag::Foundation,
            argument_type_ids: operation["argument_type_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect(),
            normal_result_type_id: operation["normal_result_type_id"]
                .as_str()
                .unwrap()
                .to_owned(),
            ordered_checks: operation["error_precedence"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| foundation_check(value.as_str().unwrap()))
                .collect(),
        })
        .collect()
}

fn semantic_binding_input(
    source_content_sha256: String,
    _dependency_id: String,
) -> SemanticBindingInput {
    let source_type_id = canonical_source_declaration_id(&SourceDeclarationIdentity {
        namespace: "Business.Orders".to_owned(),
        kind: SourceDeclarationKind::Type,
        containing_source_type_id: None,
        source_name: "OrderOption".to_owned(),
        parameter_type_ids: Vec::new(),
        result_type_id: None,
    })
    .unwrap();
    let tag_type_id = canonical_source_declaration_id(&SourceDeclarationIdentity {
        namespace: "Business.Orders".to_owned(),
        kind: SourceDeclarationKind::Type,
        containing_source_type_id: None,
        source_name: "OrderOptionTag".to_owned(),
        parameter_type_ids: Vec::new(),
        result_type_id: None,
    })
    .unwrap();
    let tag_member_id = canonical_source_stored_member_id(&SourceStoredMemberIdentity {
        owner_source_type_id: source_type_id.clone(),
        source_name: "Tag".to_owned(),
        closed_type: json!({"kind": "source", "id": tag_type_id}),
        storage: SourceStoredMemberStorage::ReadonlyField,
    })
    .unwrap();
    let value_member_id = canonical_source_stored_member_id(&SourceStoredMemberIdentity {
        owner_source_type_id: source_type_id.clone(),
        source_name: "Value".to_owned(),
        closed_type: json!({"kind": "primitive", "id": "i32"}),
        storage: SourceStoredMemberStorage::ReadonlyField,
    })
    .unwrap();
    SemanticBindingInput {
        source_type_id,
        source_content_sha256,
        role: "option".to_owned(),
        member_map: vec![
            SemanticBindingMember {
                role: "tag".to_owned(),
                member_id: tag_member_id,
            },
            SemanticBindingMember {
                role: "value".to_owned(),
                member_id: value_member_id,
            },
        ],
        tag_arms: vec![
            SemanticArmMapping {
                source_tag: "0".to_owned(),
                semantic_arm: "none".to_owned(),
            },
            SemanticArmMapping {
                source_tag: "1".to_owned(),
                semantic_arm: "some".to_owned(),
            },
        ],
        inferred_argument_ids: vec!["mpk.csharp.value.i32.v1".to_owned()],
        default_arm: "none".to_owned(),
        bounds: Vec::new(),
        operation_map: Vec::new(),
    }
}

fn foundation_check(id: &str) -> RequiredCheck {
    match id {
        "invalid_operation" => RequiredCheck {
            id: id.to_owned(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some("System.InvalidOperationException".to_owned()),
        },
        other => panic!("unexpected option check {other}"),
    }
}

fn boundary_value(amount: &str) -> PracticalJsonValue {
    PracticalJsonValue::object(vec![("amount", PracticalJsonValue::string(amount))])
}

fn published_binding_value(binding: &Value) -> PracticalJsonValue {
    let role = binding["role"].as_str().unwrap();
    let member_names: &[&str] = match role {
        "option" | "lookup" | "boundary_field" => &["tag", "value"],
        "result" => &["tag", "value", "error"],
        "validation" => &["tag", "value", "errors"],
        "transition" => &["state", "events", "response"],
        "instant" => &["milliseconds"],
        "money" => &["amount", "currency"],
        "bounded_sequence" | "ordered_set" => &["elements"],
        "ordered_entry" => &["key", "value"],
        "ordered_map" => &["entries"],
        other => panic!("unknown published role {other}"),
    };
    let arm_names: &[&str] = match role {
        "option" => &["none", "some"],
        "lookup" => &["missing_key", "found"],
        "result" => &["ok", "error"],
        "validation" => &["valid", "invalid"],
        "boundary_field" => &["missing", "null", "value"],
        _ => &[],
    };
    let ordered_string_object = |field: &str, names: &[&str]| {
        PracticalJsonValue::Object(
            names
                .iter()
                .map(|name| {
                    (
                        (*name).to_owned(),
                        PracticalJsonValue::string(binding[field][*name].as_str().unwrap()),
                    )
                })
                .collect(),
        )
    };
    let bounds = binding["bounds"].as_object().unwrap();
    let bounds = PracticalJsonValue::Object(
        bounds
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    PracticalJsonValue::U64(value.as_u64().unwrap()),
                )
            })
            .collect(),
    );
    assert!(binding["operation_map"].as_object().unwrap().is_empty());
    PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(binding["schema"].as_str().unwrap()),
        ),
        (
            "source_type_id",
            PracticalJsonValue::string(binding["source_type_id"].as_str().unwrap()),
        ),
        (
            "source_content_sha256",
            PracticalJsonValue::string(binding["source_content_sha256"].as_str().unwrap()),
        ),
        ("role", PracticalJsonValue::string(role)),
        (
            "member_map",
            ordered_string_object("member_map", member_names),
        ),
        ("tag_arms", ordered_string_object("tag_arms", arm_names)),
        (
            "inferred_argument_ids",
            PracticalJsonValue::Array(
                binding["inferred_argument_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| PracticalJsonValue::string(value.as_str().unwrap()))
                    .collect(),
            ),
        ),
        (
            "default_arm",
            PracticalJsonValue::string(binding["default_arm"].as_str().unwrap()),
        ),
        ("bounds", bounds),
        ("operation_map", PracticalJsonValue::Object(Vec::new())),
        (
            "binding_sha256",
            PracticalJsonValue::string(binding["binding_sha256"].as_str().unwrap()),
        ),
    ])
}

fn type_contract_bytes(context: &PracticalArtifactContext, source_hash: &str) -> Vec<u8> {
    let source_type_id = source_type_id("Order");
    let amount_member_id = stored_i32_member_id(&source_type_id, "Amount");
    hashed_document(
        TYPE_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
        vec![
            ("schema", PracticalJsonValue::string(TYPE_CONTRACT_SCHEMA)),
            ("semantic_context", context.semantic_context().clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            ("source_type_id", PracticalJsonValue::string(source_type_id)),
            (
                "source_content_sha256",
                PracticalJsonValue::string(source_hash),
            ),
            (
                "ordered_member_ids",
                PracticalJsonValue::Array(vec![PracticalJsonValue::string(amount_member_id)]),
            ),
            (
                "recursive_default",
                PracticalJsonValue::object(vec![
                    (
                        "type_id",
                        PracticalJsonValue::string("mpk.csharp.value.i32.v1"),
                    ),
                    ("value", PracticalJsonValue::string("0")),
                ]),
            ),
            ("default_eligible", PracticalJsonValue::Bool(true)),
            ("required_member_ids", PracticalJsonValue::Array(Vec::new())),
            ("init_member_ids", PracticalJsonValue::Array(Vec::new())),
            ("construction_invariant", PracticalJsonValue::Null),
            ("invariants", PracticalJsonValue::Array(Vec::new())),
            (
                "structural_equality",
                PracticalJsonValue::string("field_complete"),
            ),
            (
                "structural_order",
                PracticalJsonValue::string("canonical_field_order"),
            ),
        ],
    )
}

fn method_contract_bytes(context: &PracticalArtifactContext, source_hash: &str) -> Vec<u8> {
    hashed_document(
        METHOD_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
        vec![
            ("schema", PracticalJsonValue::string(METHOD_CONTRACT_SCHEMA)),
            ("semantic_context", context.semantic_context().clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            (
                "callable_id",
                PracticalJsonValue::string(&context.selected_root_ids()[0]),
            ),
            (
                "source_content_sha256",
                PracticalJsonValue::string(source_hash),
            ),
            ("termination", PracticalJsonValue::string("total")),
            ("requires", PracticalJsonValue::Array(Vec::new())),
            ("ensures", PracticalJsonValue::Array(Vec::new())),
            ("exceptional_cases", PracticalJsonValue::Array(Vec::new())),
            ("modifies", PracticalJsonValue::Array(Vec::new())),
            ("loops", PracticalJsonValue::Array(Vec::new())),
        ],
    )
}

fn boundary_contract_bytes(context: &PracticalArtifactContext) -> Vec<u8> {
    hashed_document(
        BOUNDARY_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
        vec![
            (
                "schema",
                PracticalJsonValue::string(BOUNDARY_CONTRACT_SCHEMA),
            ),
            ("semantic_context", context.semantic_context().clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            ("boundary_id", PracticalJsonValue::string("boundary.order")),
            (
                "selected_callable_id",
                PracticalJsonValue::string(&context.selected_root_ids()[0]),
            ),
            ("input_fields", PracticalJsonValue::Array(Vec::new())),
            ("output_fields", PracticalJsonValue::Array(Vec::new())),
            (
                "canonical_json_profile",
                PracticalJsonValue::string("mpk.csharp.canonical_json.v1"),
            ),
            (
                "parse_format_profile",
                PracticalJsonValue::string("mpk.csharp.parse_format.v1"),
            ),
            (
                "evidence_linkage",
                PracticalJsonValue::object(vec![
                    (
                        "raw_input_domain",
                        PracticalJsonValue::string("MPK-CSHARP-BOUNDARY-INPUT-1.0"),
                    ),
                    (
                        "canonical_value_domain",
                        PracticalJsonValue::string("MPK-CSHARP-CANONICAL-VALUE-1.0"),
                    ),
                    (
                        "canonical_output_domain",
                        PracticalJsonValue::string("MPK-CSHARP-BOUNDARY-OUTPUT-1.0"),
                    ),
                    (
                        "reparse_equality",
                        PracticalJsonValue::string("typed_field_complete"),
                    ),
                ]),
            ),
        ],
    )
}

fn transition_contract_bytes(context: &PracticalArtifactContext) -> Vec<u8> {
    hashed_document(
        TRANSITION_CONTRACT_HASH_DOMAIN,
        "contract_sha256",
        vec![
            (
                "schema",
                PracticalJsonValue::string(TRANSITION_CONTRACT_SCHEMA),
            ),
            ("semantic_context", context.semantic_context().clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            (
                "transition_id",
                PracticalJsonValue::string("transition.order.apply"),
            ),
            (
                "selected_callable_id",
                PracticalJsonValue::string(&context.selected_root_ids()[0]),
            ),
            (
                "state_type_id",
                PracticalJsonValue::string(source_type_id("OrderState")),
            ),
            (
                "command_type_id",
                PracticalJsonValue::string(source_type_id("OrderCommand")),
            ),
            (
                "context_type_id",
                PracticalJsonValue::string(source_type_id("OrderContext")),
            ),
            (
                "apply_result_binding_id",
                PracticalJsonValue::string("binding.apply_result"),
            ),
            (
                "transition_binding_id",
                PracticalJsonValue::string("binding.transition"),
            ),
            (
                "domain_error_binding_id",
                PracticalJsonValue::string("binding.error"),
            ),
            ("state_invariant", empty_expression()),
            ("version_rule", PracticalJsonValue::object(Vec::new())),
            ("idempotency", PracticalJsonValue::object(Vec::new())),
            ("accepted_commands", PracticalJsonValue::Array(Vec::new())),
            ("event_relation", empty_expression()),
            ("response_relation", empty_expression()),
            ("errors", PracticalJsonValue::Array(Vec::new())),
        ],
    )
}

fn empty_expression() -> PracticalJsonValue {
    PracticalJsonValue::object(vec![
        ("tag", PracticalJsonValue::string("literal")),
        (
            "type_id",
            PracticalJsonValue::string("mpk.csharp.value.bool.v1"),
        ),
        ("value", PracticalJsonValue::Bool(true)),
    ])
}

fn selection_bytes(context: &PracticalArtifactContext) -> Vec<u8> {
    canonical_practical_json_bytes(&PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(CSHARP_PRACTICAL_SELECTION_SCHEMA),
        ),
        (
            "compilation_id",
            PracticalJsonValue::string(context.compilation_id()),
        ),
        (
            "source_paths",
            PracticalJsonValue::Array(
                context
                    .source_paths()
                    .iter()
                    .map(PracticalJsonValue::string)
                    .collect(),
            ),
        ),
        (
            "selected_root_ids",
            PracticalJsonValue::Array(
                context
                    .selected_root_ids()
                    .iter()
                    .map(PracticalJsonValue::string)
                    .collect(),
            ),
        ),
        (
            "sidecar_paths",
            PracticalJsonValue::Array(
                context
                    .sidecar_paths()
                    .iter()
                    .map(PracticalJsonValue::string)
                    .collect(),
            ),
        ),
        (
            "selection_sha256",
            PracticalJsonValue::string(context.selection_sha256()),
        ),
    ]))
    .unwrap()
}

fn hashed_document(
    domain: HashDomain,
    hash_field: &str,
    fields: Vec<(&str, PracticalJsonValue)>,
) -> Vec<u8> {
    let preimage = PracticalJsonValue::object(fields.clone());
    let canonical = canonical_practical_json_bytes(&preimage).expect("preimage");
    let digest = hash_domain_separated_raw(domain, &canonical)
        .expect("hash")
        .to_hex();
    let mut complete = fields;
    complete.push((hash_field, PracticalJsonValue::string(digest)));
    canonical_practical_json_bytes(&PracticalJsonValue::object(complete)).expect("document")
}

fn rehash_document(mut value: PracticalJsonValue, domain: HashDomain, hash_field: &str) -> Vec<u8> {
    let PracticalJsonValue::Object(fields) = &mut value else {
        panic!("hashed document must be an object");
    };
    let (removed_name, _) = fields.pop().expect("hash field");
    assert_eq!(removed_name, hash_field);
    let preimage = PracticalJsonValue::Object(fields.clone());
    let canonical = canonical_practical_json_bytes(&preimage).expect("preimage");
    let digest = hash_domain_separated_raw(domain, &canonical)
        .expect("hash")
        .to_hex();
    fields.push((hash_field.to_owned(), PracticalJsonValue::string(digest)));
    canonical_practical_json_bytes(&value).expect("document")
}

fn execute_shape_vector<F, T>(id: &str, valid: &PracticalJsonValue, validate: F) -> Value
where
    F: Fn(&[u8]) -> Result<T, mpk_vc::csharp_practical_source_artifacts::PracticalArtifactError>,
{
    let accepted = if id.ends_with(".valid") {
        validate(&canonical_practical_json_bytes(valid).unwrap()).is_ok()
    } else if id.ends_with(".duplicate_key") {
        let bytes = if id.contains("csharp_semantic_bindings") {
            br#"{"schema":"mpk.csharp.semantic_bindings.v1","schema":"mpk.csharp.semantic_bindings.v1"}"#.as_slice()
        } else if id.contains("artifact_ref") {
            br#"{"schema":0,"schema":0}"#.as_slice()
        } else {
            br#"{"source_file_ordinal":0,"source_file_ordinal":0}"#.as_slice()
        };
        validate(bytes).is_ok()
    } else if id.ends_with(".later_version") {
        let mut mutated = valid.clone();
        set_field(
            &mut mutated,
            "schema",
            PracticalJsonValue::string("mpk.csharp.semantic_bindings.v2"),
        );
        validate(&canonical_practical_json_bytes(&mutated).unwrap()).is_ok()
    } else if id.ends_with(".missing_field") {
        let PracticalJsonValue::Object(entries) = valid else {
            unreachable!()
        };
        entries.iter().enumerate().any(|(index, _)| {
            let mut mutated = valid.clone();
            if let PracticalJsonValue::Object(entries) = &mut mutated {
                entries.remove(index);
            }
            validate(&canonical_practical_json_bytes(&mutated).unwrap()).is_ok()
        })
    } else if id.ends_with(".unknown_field") {
        let mut mutated = valid.clone();
        if let PracticalJsonValue::Object(entries) = &mut mutated {
            entries.push(("unknown".to_owned(), PracticalJsonValue::Null));
        }
        validate(&canonical_practical_json_bytes(&mutated).unwrap()).is_ok()
    } else if id.ends_with(".wrong_field_type") {
        let PracticalJsonValue::Object(entries) = valid else {
            unreachable!()
        };
        entries.iter().enumerate().any(|(index, _)| {
            let mut mutated = valid.clone();
            if let PracticalJsonValue::Object(entries) = &mut mutated {
                entries[index].1 = wrong_type(&entries[index].1);
            }
            validate(&canonical_practical_json_bytes(&mutated).unwrap()).is_ok()
        })
    } else {
        unreachable!("unknown vector {id}")
    };
    if id.ends_with(".valid") {
        json!({"accept": accepted})
    } else {
        assert!(!accepted, "{id} must reject");
        let category = id.rsplit('.').next().unwrap();
        let category = match category {
            "later_version" => "schema_version",
            "duplicate_key" => "duplicate_key",
            "missing_field" => "missing_field",
            "unknown_field" => "unknown_field",
            "wrong_field_type" => "field_type",
            other => panic!("unknown rejection category {other}"),
        };
        json!({"reject": category})
    }
}

fn wrong_type(value: &PracticalJsonValue) -> PracticalJsonValue {
    match value {
        PracticalJsonValue::Null => PracticalJsonValue::Bool(true),
        PracticalJsonValue::Bool(_) => PracticalJsonValue::String("wrong".to_owned()),
        PracticalJsonValue::I64(_) | PracticalJsonValue::U64(_) => {
            PracticalJsonValue::String("wrong".to_owned())
        }
        PracticalJsonValue::String(_) | PracticalJsonValue::Utf16String(_) => {
            PracticalJsonValue::Bool(false)
        }
        PracticalJsonValue::Array(_) => PracticalJsonValue::Object(Vec::new()),
        PracticalJsonValue::Object(_) => PracticalJsonValue::Array(Vec::new()),
    }
}

fn set_field(value: &mut PracticalJsonValue, name: &str, replacement: PracticalJsonValue) {
    let PracticalJsonValue::Object(entries) = value else {
        panic!("object")
    };
    entries
        .iter_mut()
        .find(|(candidate, _)| candidate == name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .1 = replacement;
}

fn assert_contract_field_shape_rejected(
    fixture: &Fixture,
    kind: PracticalArtifactKind,
    artifact: &ValidatedPracticalArtifact,
    domain: HashDomain,
    field: &str,
    replacement: PracticalJsonValue,
) {
    let mut mutated = artifact.value().clone();
    set_field(&mut mutated, field, replacement);
    let transport = rehash_document(mutated, domain, "contract_sha256");
    assert_code(
        validate_contract_artifact(&fixture.context, &fixture.captures, kind, &transport),
        PracticalArtifactErrorCode::Shape,
    );
}

fn assert_code<T>(
    result: Result<T, mpk_vc::csharp_practical_source_artifacts::PracticalArtifactError>,
    expected: PracticalArtifactErrorCode,
) {
    let error = match result {
        Ok(_) => panic!("expected {expected:?} rejection"),
        Err(error) => error,
    };
    assert_eq!(error.code(), expected, "{error}");
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
    let digest = successor_profile_registry_hash(&registry).expect("registry hash");
    registry["registry_sha256"] = Value::String(digest);
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
    let digest = successor_profile_entry_hash(&entry).expect("entry hash");
    entry["entry_sha256"] = Value::String(digest);
    entry
}

fn context_fixture(registry: &Value) -> Value {
    let entry = registry["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_profile"] == "mpk.csharp.practical.v1")
        .unwrap();
    json!({
        "schema": SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
        "profile_registry": {
            "schema": registry["schema"],
            "id": registry["id"],
            "revision": registry["revision"],
            "registry_sha256": registry["registry_sha256"]
        },
        "profile_entry_sha256": entry["entry_sha256"],
        "source_language": "csharp",
        "semantic_profile": "mpk.csharp.practical.v1",
        "semantic_parameters": {
            "schema": CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
            "value": {
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
            }
        },
        "foundation_descriptor": foundation_descriptor()
    })
}

fn scalar_context_fixture(registry: &Value) -> Value {
    let entry = registry["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_profile"] == "mpk.csharp.scalar.v0")
        .unwrap();
    json!({
        "schema": SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
        "profile_registry": {
            "schema": registry["schema"],
            "id": registry["id"],
            "revision": registry["revision"],
            "registry_sha256": registry["registry_sha256"]
        },
        "profile_entry_sha256": entry["entry_sha256"],
        "source_language": "csharp",
        "semantic_profile": "mpk.csharp.scalar.v0",
        "semantic_parameters": {
            "schema": "mpk.semantic_parameters.csharp_scalar.v0",
            "value": {
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
            }
        },
        "foundation_descriptor": foundation_descriptor()
    })
}

fn foundation_descriptor() -> Value {
    json!({
        "schema": FOUNDATION_DESCRIPTOR_SCHEMA,
        "id": FOUNDATION_DESCRIPTOR_ID,
        "content_sha256": FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    })
}

fn practical_selection(compilation_id: &str) -> Value {
    let root_id =
        canonical_source_declaration_id(&declaration_identity("Run")).expect("root declaration ID");
    let mut selection = json!({
        "schema": CSHARP_PRACTICAL_SELECTION_SCHEMA,
        "compilation_id": compilation_id,
        "source_paths": ["src/Order.cs"],
        "selected_root_ids": [root_id],
        "sidecar_paths": ["contracts/order.json"],
        "selection_sha256": ZERO_SHA256
    });
    let digest = csharp_practical_selection_hash(&selection).expect("selection hash");
    selection["selection_sha256"] = Value::String(digest);
    selection
}

fn declaration_identity(name: &str) -> SourceDeclarationIdentity {
    SourceDeclarationIdentity {
        namespace: "Business".to_owned(),
        kind: SourceDeclarationKind::Method,
        containing_source_type_id: Some(source_type_id("Order")),
        source_name: name.to_owned(),
        parameter_type_ids: Vec::new(),
        result_type_id: Some("mpk.csharp.value.i32.v1".to_owned()),
    }
}

fn source_type_identity(name: &str) -> SourceDeclarationIdentity {
    SourceDeclarationIdentity {
        namespace: "Business".to_owned(),
        kind: SourceDeclarationKind::Type,
        containing_source_type_id: None,
        source_name: name.to_owned(),
        parameter_type_ids: Vec::new(),
        result_type_id: None,
    }
}

fn source_type_id(name: &str) -> String {
    canonical_source_declaration_id(&source_type_identity(name)).expect("source type ID")
}

fn stored_i32_member_id(owner_source_type_id: &str, name: &str) -> String {
    canonical_source_stored_member_id(&SourceStoredMemberIdentity {
        owner_source_type_id: owner_source_type_id.to_owned(),
        source_name: name.to_owned(),
        closed_type: json!({"kind": "primitive", "id": "i32"}),
        storage: SourceStoredMemberStorage::ReadonlyField,
    })
    .expect("stored member ID")
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

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&to_strict(value)).expect("canonical JSON")
}

fn to_strict(value: &Value) -> StrictJsonValue {
    match value {
        Value::Null => StrictJsonValue::Null,
        Value::Bool(value) => StrictJsonValue::Bool(*value),
        Value::Number(value) => StrictJsonValue::Integer(value.as_i64().unwrap()),
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
