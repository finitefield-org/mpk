use mpk_cli::csharp_practical_frontend_protocol::{
    build_csharp_practical_frontend_request, emit_csharp_practical_frontend_diagnostic,
    emit_csharp_practical_frontend_success, import_csharp_practical_frontend_request,
    validate_csharp_practical_frontend_process, PracticalDiagnosticFamily,
    PracticalDiagnosticFinding, PracticalDiagnosticLocation, PracticalFrontendOutcome,
    PracticalFrontendProtocolCode, PracticalFrontendValidationContext, PracticalSidecarDescriptor,
    ValidatedPracticalFrontendRequest, CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA,
    CSHARP_PRACTICAL_FRONTEND_STDERR_BYTES_MAX, CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA,
    CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE,
};
use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_vc::csharp_practical_registry::{
    canonical_successor_registry_transport, csharp_practical_selection_hash,
    successor_profile_entry_hash, successor_profile_registry_hash,
    successor_validated_request_hash, validate_candidate_successor_registry,
    validate_successor_semantic_request, SuccessorCompiledSemanticProfile,
    SuccessorProfileContract, ValidatedSuccessorRequest, CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
    CSHARP_PRACTICAL_SELECTION_SCHEMA, FOUNDATION_DESCRIPTOR_CONTENT_SHA256,
    FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA, SUCCESSOR_CANDIDATE_REVISION,
    SUCCESSOR_CONTRACT_FIELDS, SUCCESSOR_PROFILE_ORDER, SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
    SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA, SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
    SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
};
use mpk_vc::csharp_practical_source_artifacts::{
    bind_closed_instances, bind_practical_artifact_context, build_certificate_source_manifest,
    build_concrete_operation_tables, build_frontend_source_artifacts,
    build_frontend_source_manifest, build_practical_source_map, build_semantic_bindings,
    canonical_practical_json_bytes, canonical_source_declaration_id, capture_original_inputs,
    parse_canonical_practical_json, ArtifactRef, CapturedInputSet, FrontendManifestArtifacts,
    FrontendSourceArtifactLinks, OriginalInput, OriginalInputKind, PracticalArtifactContext,
    PracticalArtifactKind, PracticalJsonValue, SourceDeclarationIdentity, SourceDeclarationKind,
    SourceMapDeclaration, SourceMapIdentity, ValidatedPracticalArtifact, METHOD_CONTRACT_SCHEMA,
    SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA, SUCCESSOR_VC_SCHEMA, SUCCESSOR_VIR_SCHEMA,
    TYPE_CONTRACT_SCHEMA,
};
use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, derive_closed_instances,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_closed_root_set, validate_registered_foundation_bundle,
};
use mpk_vc::{canonical_json_bytes, StrictJsonValue};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

const PACKAGE: &str =
    include_str!("../../../develop/specs/vectors/csharp-practical-profile-v1.json");
const WORK_ITEM: &str = "CSHARP-03-T02-W07";
const OWNER: &str = "crates/mpk-cli/tests/csharp_practical_frontend_protocol.rs#CSHARP-03-T02-W07";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SOURCE: &[u8] = b"public static class Order { public static int Run() => 1; }\n";

struct Fixture {
    registry: mpk_vc::csharp_practical_registry::ValidatedSuccessorRegistry,
    semantic_request: ValidatedSuccessorRequest,
    context: PracticalArtifactContext,
    captures: CapturedInputSet,
    request: ValidatedPracticalFrontendRequest,
    source_map: ValidatedPracticalArtifact,
    frontend_manifest: ValidatedPracticalArtifact,
    certificate_manifest: ValidatedPracticalArtifact,
    source_artifacts: ValidatedPracticalArtifact,
}

#[test]
fn csharp_03_t02_w07_executes_every_frozen_schema_vector() {
    let fixture = fixture();
    let package: Value = serde_json::from_str(PACKAGE).expect("profile package");
    let vectors = package["vectors"]
        .as_array()
        .expect("vectors")
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
        .collect::<Vec<_>>();
    assert_eq!(vectors.len(), 26);
    assert!(vectors
        .iter()
        .all(|vector| vector["production_test_owner"] == OWNER));

    let base = fixture.request.value().clone();
    let mut seen = BTreeSet::new();
    for vector in vectors {
        let id = vector["id"].as_str().expect("vector id");
        seen.insert(id.to_owned());
        let accepted = execute_schema_vector(&fixture, &base, id);
        assert_eq!(accepted, vector["expected"]["accept"] == true, "{id}");
    }

    assert_eq!(seen.len(), 26);
    assert_all_member_mutations_reject(&fixture, &base);
}

#[test]
fn csharp_03_t02_w07_binds_complete_inventory_maps_and_manifests() {
    let fixture = fixture();
    let imported = import_csharp_practical_frontend_request(
        &fixture.registry,
        &fixture.captures,
        fixture.request.canonical_bytes(),
    )
    .expect("request round trip");
    assert_eq!(imported.request_sha256(), fixture.request.request_sha256());
    assert_eq!(
        imported.input_set_sha256(),
        fixture.captures.snapshot_sha256()
    );

    let request = imported.value();
    assert_eq!(
        request_fields(request),
        vec![
            "schema",
            "semantic_request",
            "source_snapshot",
            "sidecars",
            "request_sha256",
        ]
    );
    let source_entries = request
        .get("source_snapshot")
        .and_then(|value| value.get("entries"))
        .and_then(PracticalJsonValue::as_array)
        .expect("source snapshot entries");
    assert_eq!(source_entries.len(), 1);
    assert_eq!(
        source_entries[0]
            .get("raw_sha256")
            .and_then(PracticalJsonValue::as_str),
        fixture
            .captures
            .entry("src/Order.cs")
            .map(|entry| entry.raw_sha256())
    );
    assert_eq!(
        source_entries[0]
            .get("size_bytes")
            .and_then(PracticalJsonValue::as_u64),
        Some(SOURCE.len() as u64)
    );
    let sidecar_entries = request
        .get("sidecars")
        .and_then(|value| value.get("entries"))
        .and_then(PracticalJsonValue::as_array)
        .expect("sidecar entries");
    assert_eq!(sidecar_entries.len(), 1);
    assert_eq!(
        sidecar_entries[0]
            .get("schema")
            .and_then(PracticalJsonValue::as_str),
        Some(TYPE_CONTRACT_SCHEMA)
    );
    assert_code(
        build_csharp_practical_frontend_request(
            &fixture.semantic_request,
            &fixture.captures,
            &[PracticalSidecarDescriptor {
                schema: METHOD_CONTRACT_SCHEMA.into(),
                path: "contracts/order.json".into(),
            }],
        ),
        PracticalFrontendProtocolCode::Inventory,
    );

    let map_entries = fixture
        .source_map
        .value()
        .get("entries")
        .and_then(PracticalJsonValue::as_array)
        .expect("source map entries");
    assert_eq!(map_entries.len(), 1);
    let location = map_entries[0]
        .get("source_location")
        .expect("source location");
    assert_eq!(
        location
            .get("source_file_ordinal")
            .and_then(PracticalJsonValue::as_u64),
        Some(0)
    );
    assert_eq!(
        location
            .get("end_byte")
            .and_then(PracticalJsonValue::as_u64),
        Some(SOURCE.len() as u64)
    );
    assert_eq!(
        fixture
            .frontend_manifest
            .value()
            .get("input_set_sha256")
            .and_then(PracticalJsonValue::as_str),
        Some(fixture.captures.snapshot_sha256())
    );
    assert!(fixture
        .source_artifacts
        .matches_validated_lineage(&fixture.context, &fixture.captures));
    assert!(fixture
        .certificate_manifest
        .matches_validated_lineage(&fixture.context, &fixture.captures));

    let different_captures = capture_original_inputs(
        &fixture.context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".into(),
                bytes: b"public static class Order { public static int Run() => 2; }\n".to_vec(),
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/order.json".into(),
                bytes: sidecar_bytes(),
            },
        ],
    )
    .expect("different capture set");
    assert!(!fixture
        .source_artifacts
        .matches_validated_lineage(&fixture.context, &different_captures));
    assert_code(
        emit_csharp_practical_frontend_success(
            &fixture.request,
            &fixture.context,
            &different_captures,
            &fixture.source_artifacts,
        ),
        PracticalFrontendProtocolCode::Artifact,
    );
}

#[test]
fn csharp_03_t02_w07_accepts_only_complete_success_and_artifact_free_failure() {
    let fixture = fixture();
    let success = emit_csharp_practical_frontend_success(
        &fixture.request,
        &fixture.context,
        &fixture.captures,
        &fixture.source_artifacts,
    )
    .expect("success envelope");
    assert_eq!(
        success,
        emit_csharp_practical_frontend_success(
            &fixture.request,
            &fixture.context,
            &fixture.captures,
            &fixture.source_artifacts,
        )
        .expect("deterministic success envelope")
    );
    let accepted = validate_csharp_practical_frontend_process(
        valid_process_context(&fixture),
        process(0, &success),
    )
    .expect("validated success");
    assert_eq!(accepted.outcome(), PracticalFrontendOutcome::Success);
    assert_eq!(accepted.phase(), None);
    assert!(accepted.diagnostics().is_empty());
    assert_eq!(
        accepted.artifacts_sha256(),
        Some(fixture.source_artifacts.hash())
    );
    assert_eq!(accepted.canonical_transport(), success);
    assert!(String::from_utf8_lossy(&success).contains(CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA));

    let findings = [
        PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Lowering,
            location: None,
        },
        PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Initializer,
            location: Some(PracticalDiagnosticLocation {
                source_file_ordinal: 0,
                start_byte: 10,
                end_byte: 12,
            }),
        },
        PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Object,
            location: Some(PracticalDiagnosticLocation {
                source_file_ordinal: 0,
                start_byte: 20,
                end_byte: 21,
            }),
        },
        PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Object,
            location: Some(PracticalDiagnosticLocation {
                source_file_ordinal: 0,
                start_byte: 2,
                end_byte: 3,
            }),
        },
    ];
    let rejected = emit_csharp_practical_frontend_diagnostic(
        fixture.request.canonical_bytes(),
        Some(&fixture.request),
        &findings,
    )
    .expect("diagnostic envelope");
    let rejected_text = String::from_utf8_lossy(&rejected);
    assert!(rejected_text.contains(CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA));
    assert!(rejected_text.contains(CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE));
    for forbidden in [
        "src/Order.cs",
        "Business.Order",
        "public static class",
        "/Users/",
        "stack trace",
        "CurrentCulture",
        "generated type",
        "compiler error",
    ] {
        assert!(!rejected_text.contains(forbidden), "leaked {forbidden}");
    }
    assert!(!rejected_text.contains("artifacts"));
    let accepted = validate_csharp_practical_frontend_process(
        valid_process_context(&fixture),
        process(3, &rejected),
    )
    .expect("validated diagnostic");
    assert_eq!(accepted.outcome(), PracticalFrontendOutcome::Rejected);
    assert_eq!(accepted.phase(), Some(6));
    assert_eq!(accepted.diagnostics().len(), 3);
    assert_eq!(
        accepted.diagnostics()[0].family(),
        PracticalDiagnosticFamily::Object
    );
    assert_eq!(
        accepted.diagnostics()[0]
            .location()
            .expect("location")
            .start_byte,
        2
    );
    assert_eq!(
        accepted.diagnostics()[1].family(),
        PracticalDiagnosticFamily::Object
    );
    assert_eq!(
        accepted.diagnostics()[2].family(),
        PracticalDiagnosticFamily::Initializer
    );
    assert_eq!(accepted.artifacts_sha256(), None);

    let reversed = findings.iter().copied().rev().collect::<Vec<_>>();
    assert_eq!(
        rejected,
        emit_csharp_practical_frontend_diagnostic(
            fixture.request.canonical_bytes(),
            Some(&fixture.request),
            &reversed,
        )
        .expect("deterministic diagnostics")
    );
}

#[test]
fn csharp_03_t02_w07_covers_every_frozen_diagnostic_family_and_phase() {
    let fixture = fixture();
    let families = [
        (PracticalDiagnosticFamily::Protocol, 0),
        (PracticalDiagnosticFamily::Limit, 0),
        (PracticalDiagnosticFamily::Dependency, 1),
        (PracticalDiagnosticFamily::Declaration, 2),
        (PracticalDiagnosticFamily::Type, 2),
        (PracticalDiagnosticFamily::Generic, 3),
        (PracticalDiagnosticFamily::SourceBinding, 4),
        (PracticalDiagnosticFamily::Foundation, 4),
        (PracticalDiagnosticFamily::Boundary, 5),
        (PracticalDiagnosticFamily::Transition, 5),
        (PracticalDiagnosticFamily::Object, 6),
        (PracticalDiagnosticFamily::Initializer, 6),
        (PracticalDiagnosticFamily::Ownership, 6),
        (PracticalDiagnosticFamily::Array, 6),
        (PracticalDiagnosticFamily::Collection, 6),
        (PracticalDiagnosticFamily::Order, 6),
        (PracticalDiagnosticFamily::String, 6),
        (PracticalDiagnosticFamily::ParseFormat, 6),
        (PracticalDiagnosticFamily::Float, 6),
        (PracticalDiagnosticFamily::Decimal, 6),
        (PracticalDiagnosticFamily::Nullable, 6),
        (PracticalDiagnosticFamily::Result, 6),
        (PracticalDiagnosticFamily::BusinessValue, 6),
        (PracticalDiagnosticFamily::LoopContract, 7),
        (PracticalDiagnosticFamily::Switch, 7),
        (PracticalDiagnosticFamily::Pattern, 7),
        (PracticalDiagnosticFamily::Exception, 7),
        (PracticalDiagnosticFamily::Effect, 7),
        (PracticalDiagnosticFamily::Lowering, 8),
    ];
    assert_eq!(families.len(), 29);
    assert_eq!(
        families
            .iter()
            .map(|(family, _)| family.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        29
    );

    for (family, phase) in families {
        let output = emit_csharp_practical_frontend_diagnostic(
            fixture.request.canonical_bytes(),
            Some(&fixture.request),
            &[PracticalDiagnosticFinding {
                family,
                location: None,
            }],
        )
        .expect("family diagnostic");
        let accepted = validate_csharp_practical_frontend_process(
            valid_process_context(&fixture),
            process(3, &output),
        )
        .expect("family round trip");
        assert_eq!(accepted.phase(), Some(phase), "{}", family.as_str());
        assert_eq!(accepted.diagnostics().len(), 1);
        assert_eq!(accepted.diagnostics()[0].family(), family);
    }
}

#[test]
fn csharp_03_t02_w07_fails_closed_on_protocol_and_diagnostic_mutations() {
    let fixture = fixture();
    let raw_invalid = b"customer path /private/tmp/order.cs and compiler stack";
    let unvalidated = emit_csharp_practical_frontend_diagnostic(
        raw_invalid,
        None,
        &[PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Protocol,
            location: None,
        }],
    )
    .expect("unvalidated diagnostic");
    let public = String::from_utf8_lossy(&unvalidated);
    assert!(!public.contains("customer path"));
    assert!(!public.contains("/private/tmp"));
    validate_csharp_practical_frontend_process(
        PracticalFrontendValidationContext {
            raw_request: raw_invalid,
            validated_request: None,
            artifact_context: None,
            captured_inputs: None,
            expected_artifacts: None,
        },
        process(3, &unvalidated),
    )
    .expect("unvalidated failure round trip");

    let mut truncated = unvalidated.clone();
    truncated.pop();
    assert_code(
        validate_csharp_practical_frontend_process(
            PracticalFrontendValidationContext {
                raw_request: raw_invalid,
                validated_request: None,
                artifact_context: None,
                captured_inputs: None,
                expected_artifacts: None,
            },
            process(3, &truncated),
        ),
        PracticalFrontendProtocolCode::Truncated,
    );
    assert_code(
        validate_csharp_practical_frontend_process(
            valid_process_context(&fixture),
            FrontendProcessFacts {
                exit_code: Some(3),
                signaled: false,
                stdout: &unvalidated,
                stderr_observed_bytes: CSHARP_PRACTICAL_FRONTEND_STDERR_BYTES_MAX + 1,
            },
        ),
        PracticalFrontendProtocolCode::Limit,
    );
    assert_code(
        validate_csharp_practical_frontend_process(
            valid_process_context(&fixture),
            process(0, &unvalidated),
        ),
        PracticalFrontendProtocolCode::StatusExit,
    );

    let valid = emit_csharp_practical_frontend_diagnostic(
        fixture.request.canonical_bytes(),
        Some(&fixture.request),
        &[
            PracticalDiagnosticFinding {
                family: PracticalDiagnosticFamily::Object,
                location: Some(PracticalDiagnosticLocation {
                    source_file_ordinal: 0,
                    start_byte: 1,
                    end_byte: 2,
                }),
            },
            PracticalDiagnosticFinding {
                family: PracticalDiagnosticFamily::Initializer,
                location: Some(PracticalDiagnosticLocation {
                    source_file_ordinal: 0,
                    start_byte: 3,
                    end_byte: 4,
                }),
            },
        ],
    )
    .unwrap();

    let success = emit_csharp_practical_frontend_success(
        &fixture.request,
        &fixture.context,
        &fixture.captures,
        &fixture.source_artifacts,
    )
    .expect("success envelope");
    let duplicate_success = duplicate_at(
        &success,
        b"{\"schema\":\"mpk.frontend.success.v2\"",
        b"{\"schema\":\"mpk.frontend.success.v2\",\"schema\":\"mpk.frontend.success.v2\"",
    );
    assert_code(
        validate_csharp_practical_frontend_process(
            valid_process_context(&fixture),
            process(0, &duplicate_success),
        ),
        PracticalFrontendProtocolCode::DuplicateField,
    );
    let mut value = parse_output(&success);
    set_field(
        &mut value,
        "schema",
        PracticalJsonValue::string("mpk.frontend.success.v3"),
    );
    assert_output_code_with_exit(&fixture, value, 0, PracticalFrontendProtocolCode::Schema);
    let mut value = parse_output(&success);
    set_field(
        &mut value,
        "success_sha256",
        PracticalJsonValue::string(ZERO_SHA256),
    );
    assert_output_code_with_exit(&fixture, value, 0, PracticalFrontendProtocolCode::Hash);
    let mut value = parse_output(&success);
    set_field(&mut value, "artifacts", PracticalJsonValue::Null);
    assert_output_code_with_exit(&fixture, value, 0, PracticalFrontendProtocolCode::Linkage);

    let mut value = parse_output(&valid);
    set_field(&mut value, "phase", PracticalJsonValue::U64(8));
    assert_output_code(
        &fixture,
        value,
        PracticalFrontendProtocolCode::DiagnosticPhase,
    );

    let mut value = parse_output(&valid);
    let diagnostics = value
        .get("diagnostics")
        .and_then(PracticalJsonValue::as_array)
        .unwrap()
        .to_vec();
    set_field(
        &mut value,
        "diagnostics",
        PracticalJsonValue::Array(vec![diagnostics[1].clone(), diagnostics[0].clone()]),
    );
    assert_output_code(
        &fixture,
        value,
        PracticalFrontendProtocolCode::DiagnosticOrder,
    );

    let mut value = parse_output(&valid);
    let entry = value
        .get("diagnostics")
        .and_then(PracticalJsonValue::as_array)
        .unwrap()[0]
        .clone();
    let mut entry = entry;
    set_field(
        &mut entry,
        "message",
        PracticalJsonValue::string("compiler error Business.Order at /tmp/source.cs"),
    );
    set_field(
        &mut value,
        "diagnostics",
        PracticalJsonValue::Array(vec![entry]),
    );
    assert_output_code(&fixture, value, PracticalFrontendProtocolCode::PublicData);

    let mut value = parse_output(&valid);
    set_field(
        &mut value,
        "schema",
        PracticalJsonValue::string("mpk.frontend.diagnostic.v3"),
    );
    assert_output_code(&fixture, value, PracticalFrontendProtocolCode::Schema);

    let mut value = parse_output(&valid);
    if let PracticalJsonValue::Object(entries) = &mut value {
        entries.insert(
            entries.len() - 1,
            (
                "artifacts".to_owned(),
                fixture.source_artifacts.value().clone(),
            ),
        );
    }
    assert_output_code(
        &fixture,
        value,
        PracticalFrontendProtocolCode::PartialArtifacts,
    );

    let invalid_location = emit_csharp_practical_frontend_diagnostic(
        fixture.request.canonical_bytes(),
        Some(&fixture.request),
        &[PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Object,
            location: Some(PracticalDiagnosticLocation {
                source_file_ordinal: 0,
                start_byte: SOURCE.len() as u32,
                end_byte: SOURCE.len() as u32 + 1,
            }),
        }],
    );
    assert_code(
        invalid_location,
        PracticalFrontendProtocolCode::DiagnosticLocation,
    );
    assert_code(
        emit_csharp_practical_frontend_diagnostic(
            raw_invalid,
            None,
            &[PracticalDiagnosticFinding {
                family: PracticalDiagnosticFamily::Dependency,
                location: None,
            }],
        ),
        PracticalFrontendProtocolCode::DiagnosticPhase,
    );
}

#[test]
fn csharp_03_t02_w07_rejects_truncation_duplicates_versions_limits_and_fuzz() {
    let fixture = fixture();
    let transport = fixture.request.canonical_bytes();
    let duplicate = duplicate_at(
        transport,
        b"{\"schema\":\"mpk.frontend.request.v2\"",
        b"{\"schema\":\"mpk.frontend.request.v2\",\"schema\":\"mpk.frontend.request.v2\"",
    );
    assert_code(
        import_csharp_practical_frontend_request(&fixture.registry, &fixture.captures, &duplicate),
        PracticalFrontendProtocolCode::DuplicateField,
    );

    let mut later = fixture.request.value().clone();
    set_field(
        &mut later,
        "schema",
        PracticalJsonValue::string("mpk.frontend.request.v3"),
    );
    assert_request_rejects(&fixture, &later);

    let mut wrong_order = fixture.request.value().clone();
    if let PracticalJsonValue::Object(entries) = &mut wrong_order {
        entries.swap(0, 1);
    }
    assert_code(
        import_value(&fixture, &wrong_order),
        PracticalFrontendProtocolCode::FieldOrder,
    );

    let mut truncated = transport.to_vec();
    truncated.pop();
    assert_request_rejects_bytes(&fixture, &truncated);

    for index in (0..transport.len()).step_by((transport.len() / 97).max(1)) {
        let mut mutated = transport.to_vec();
        mutated[index] ^= 1;
        assert_request_rejects_bytes(&fixture, &mutated);
    }

    let oversized_source = vec![b'x'; 1_048_577];
    let oversized_captures = capture_original_inputs(
        &fixture.context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".into(),
                bytes: oversized_source,
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/order.json".into(),
                bytes: sidecar_bytes(),
            },
        ],
    )
    .expect("capture does not apply frontend limits");
    assert_code(
        build_csharp_practical_frontend_request(
            &fixture.semantic_request,
            &oversized_captures,
            &sidecar_descriptors(),
        ),
        PracticalFrontendProtocolCode::Limit,
    );

    let too_many = vec![
        PracticalDiagnosticFinding {
            family: PracticalDiagnosticFamily::Protocol,
            location: None,
        };
        1_025
    ];
    assert_code(
        emit_csharp_practical_frontend_diagnostic(b"bad", None, &too_many),
        PracticalFrontendProtocolCode::Limit,
    );
}

fn execute_schema_vector(fixture: &Fixture, base: &PracticalJsonValue, id: &str) -> bool {
    if id.ends_with(".valid") {
        return import_value(fixture, base).is_ok();
    }
    if id.ends_with(".duplicate_key") {
        let (needle, replacement): (&[u8], &[u8]) = if id.contains("frontend_request") {
            (
                b"{\"schema\":\"mpk.frontend.request.v2\"",
                b"{\"schema\":\"mpk.frontend.request.v2\",\"schema\":\"mpk.frontend.request.v2\"",
            )
        } else if id.contains("source_snapshot_entry") {
            (
                b"{\"path\":\"src/Order.cs\"",
                b"{\"path\":\"src/Order.cs\",\"path\":\"src/Order.cs\"",
            )
        } else if id.contains("source_snapshot_v2") {
            (
                b"\"source_snapshot\":{\"entries\":",
                b"\"source_snapshot\":{\"entries\":[],\"entries\":",
            )
        } else if id.contains("sidecar_ref") {
            (
                b"{\"schema\":\"mpk.csharp.type_contract.v1\",\"path\":",
                b"{\"schema\":\"mpk.csharp.type_contract.v1\",\"schema\":\"mpk.csharp.type_contract.v1\",\"path\":",
            )
        } else {
            (
                b"\"sidecars\":{\"entries\":",
                b"\"sidecars\":{\"entries\":[],\"entries\":",
            )
        };
        return import_csharp_practical_frontend_request(
            &fixture.registry,
            &fixture.captures,
            &duplicate_at(fixture.request.canonical_bytes(), needle, replacement),
        )
        .is_ok();
    }
    let mut mutated = base.clone();
    let target = schema_target_mut(&mut mutated, id);
    if id.ends_with(".later_version") {
        set_field(
            target,
            "schema",
            PracticalJsonValue::string("mpk.frontend.request.v3"),
        );
    } else if id.ends_with(".missing_field") {
        remove_field(target, 0);
    } else if id.ends_with(".unknown_field") {
        push_field(target, "unknown", PracticalJsonValue::Null);
    } else if id.ends_with(".wrong_field_type") {
        set_first_field(target, PracticalJsonValue::Null);
    } else {
        panic!("unhandled vector {id}");
    }
    import_value(fixture, &mutated).is_ok()
}

fn assert_all_member_mutations_reject(fixture: &Fixture, base: &PracticalJsonValue) {
    for target in [
        "frontend_request",
        "source_snapshot_v2",
        "source_snapshot_entry",
        "sidecar_set_v2",
        "sidecar_ref",
    ] {
        let field_count = {
            let mut value = base.clone();
            schema_target_mut(&mut value, target)
                .as_object()
                .unwrap()
                .len()
        };
        for index in 0..field_count {
            let mut missing = base.clone();
            remove_field(schema_target_mut(&mut missing, target), index);
            assert_request_rejects(fixture, &missing);

            let mut wrong = base.clone();
            set_field_at(
                schema_target_mut(&mut wrong, target),
                index,
                PracticalJsonValue::Null,
            );
            assert_request_rejects(fixture, &wrong);
        }
    }
}

fn schema_target_mut<'a>(
    value: &'a mut PracticalJsonValue,
    id: &str,
) -> &'a mut PracticalJsonValue {
    if id.contains("source_snapshot_entry") {
        return nested_array_first_mut(value, "source_snapshot", "entries");
    }
    if id.contains("source_snapshot_v2") {
        return object_field_mut(value, "source_snapshot");
    }
    if id.contains("sidecar_ref") {
        return nested_array_first_mut(value, "sidecars", "entries");
    }
    if id.contains("sidecar_set_v2") {
        return object_field_mut(value, "sidecars");
    }
    value
}

fn import_value(
    fixture: &Fixture,
    value: &PracticalJsonValue,
) -> Result<
    ValidatedPracticalFrontendRequest,
    mpk_cli::csharp_practical_frontend_protocol::PracticalFrontendProtocolError,
> {
    let bytes = canonical_practical_json_bytes(value).expect("canonical mutation");
    import_csharp_practical_frontend_request(&fixture.registry, &fixture.captures, &bytes)
}

fn assert_request_rejects(fixture: &Fixture, value: &PracticalJsonValue) {
    assert!(import_value(fixture, value).is_err());
}

fn assert_request_rejects_bytes(fixture: &Fixture, bytes: &[u8]) {
    assert!(
        import_csharp_practical_frontend_request(&fixture.registry, &fixture.captures, bytes)
            .is_err()
    );
}

fn fixture() -> Fixture {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(
        &canonical_successor_registry_transport(&registry_value).expect("registry transport"),
    )
    .expect("candidate registry");
    let selection = practical_selection();
    let request_value = request_fixture(context_fixture(&registry_value), selection);
    let semantic_request =
        validate_successor_semantic_request(&registry, &canonical(&request_value))
            .expect("semantic request");
    let foundation = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .expect("foundation");
    let context =
        bind_practical_artifact_context(&semantic_request, &foundation).expect("artifact context");
    let captures = capture_original_inputs(
        &context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Order.cs".into(),
                bytes: SOURCE.to_vec(),
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/order.json".into(),
                bytes: sidecar_bytes(),
            },
        ],
    )
    .expect("captures");
    let request = build_csharp_practical_frontend_request(
        &semantic_request,
        &captures,
        &sidecar_descriptors(),
    )
    .expect("frontend request");

    let roots_transport = canonical_closed_root_set_transport(&foundation, &json!([]), &json!({}))
        .expect("closed roots transport");
    let roots = validate_closed_root_set(&foundation, &roots_transport).expect("closed roots");
    let closed = derive_closed_instances(&foundation, &roots).expect("closed instances");
    let closed_ref = bind_closed_instances(&context, &foundation, &captures, &roots, &closed)
        .expect("closed instance ref");
    let semantic_bindings =
        build_semantic_bindings(&context, &captures, Vec::new()).expect("empty binding set");
    let operations =
        build_concrete_operation_tables(&context, &roots, &closed, &closed_ref, Vec::new())
            .expect("empty operation tables");
    let vir = ArtifactRef::opaque_successor(&context, SUCCESSOR_VIR_SCHEMA, &"a".repeat(64), 1024)
        .expect("VIR reference");
    let declaration = SourceMapDeclaration {
        declaration_id: context.selected_root_ids()[0].clone(),
        identity: SourceMapIdentity::Declaration(declaration_identity()),
        provenance_id: "source.order.run".into(),
        source_path: "src/Order.cs".into(),
        start_byte: 0,
        end_byte: SOURCE.len() as u32,
        artifact_node_ids: vec!["vir.node.0000".into()],
    };
    let source_map = build_practical_source_map(&context, &captures, &vir, vec![declaration])
        .expect("source map");
    let frontend_manifest = build_frontend_source_manifest(
        &context,
        &foundation,
        &captures,
        FrontendManifestArtifacts {
            type_contracts: Vec::new(),
            method_contracts: Vec::new(),
            semantic_bindings: semantic_bindings.artifact_ref(),
            boundary_contracts: Vec::new(),
            boundary_inputs: Vec::new(),
            boundary_outputs: Vec::new(),
            transition_contracts: Vec::new(),
            closed_instances: closed_ref.clone(),
            operations: operations.operations().artifact_ref(),
            required_checks: operations.required_checks().artifact_ref(),
            vir: vir.clone(),
            source_map: source_map.artifact_ref(),
        },
    )
    .expect("frontend manifest");
    let source_artifacts = build_frontend_source_artifacts(
        &context,
        &foundation,
        FrontendSourceArtifactLinks {
            vir: &vir,
            source_map: &source_map.artifact_ref(),
            source_manifest: &frontend_manifest,
            semantic_bindings: &semantic_bindings.artifact_ref(),
            closed_instances: &closed_ref,
            boundary_contracts: Vec::new(),
            transition_contracts: Vec::new(),
        },
    )
    .expect("source artifacts");
    let vc = ArtifactRef::opaque_successor(&context, SUCCESSOR_VC_SCHEMA, &"b".repeat(64), 2048)
        .expect("VC reference");
    let skeleton = ArtifactRef::opaque_successor(
        &context,
        SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA,
        &"c".repeat(64),
        4096,
    )
    .expect("skeleton reference");
    let certificate_manifest = build_certificate_source_manifest(
        &context,
        &frontend_manifest.artifact_ref(),
        &vc,
        &skeleton,
        &"d".repeat(64),
    )
    .expect("certificate manifest");

    Fixture {
        registry,
        semantic_request,
        context,
        captures,
        request,
        source_map,
        frontend_manifest,
        certificate_manifest,
        source_artifacts,
    }
}

fn sidecar_bytes() -> Vec<u8> {
    canonical_practical_json_bytes(&PracticalJsonValue::object(vec![(
        "schema",
        PracticalJsonValue::string(TYPE_CONTRACT_SCHEMA),
    )]))
    .expect("sidecar bytes")
}

fn sidecar_descriptors() -> Vec<PracticalSidecarDescriptor> {
    vec![PracticalSidecarDescriptor {
        schema: TYPE_CONTRACT_SCHEMA.into(),
        path: "contracts/order.json".into(),
    }]
}

fn valid_process_context(fixture: &Fixture) -> PracticalFrontendValidationContext<'_> {
    PracticalFrontendValidationContext {
        raw_request: fixture.request.canonical_bytes(),
        validated_request: Some(&fixture.request),
        artifact_context: Some(&fixture.context),
        captured_inputs: Some(&fixture.captures),
        expected_artifacts: Some(&fixture.source_artifacts),
    }
}

fn process<'a>(exit_code: i32, stdout: &'a [u8]) -> FrontendProcessFacts<'a> {
    FrontendProcessFacts {
        exit_code: Some(exit_code),
        signaled: false,
        stdout,
        stderr_observed_bytes: 0,
    }
}

fn parse_output(transport: &[u8]) -> PracticalJsonValue {
    parse_canonical_practical_json(
        PracticalArtifactKind::SourceArtifacts,
        &transport[..transport.len() - 1],
    )
    .expect("output JSON")
}

fn assert_output_code(
    fixture: &Fixture,
    value: PracticalJsonValue,
    expected: PracticalFrontendProtocolCode,
) {
    assert_output_code_with_exit(fixture, value, 3, expected);
}

fn assert_output_code_with_exit(
    fixture: &Fixture,
    value: PracticalJsonValue,
    exit_code: i32,
    expected: PracticalFrontendProtocolCode,
) {
    let mut transport = canonical_practical_json_bytes(&value).expect("mutated output");
    transport.push(b'\n');
    assert_code(
        validate_csharp_practical_frontend_process(
            valid_process_context(fixture),
            process(exit_code, &transport),
        ),
        expected,
    );
}

fn assert_code<T>(
    result: Result<T, mpk_cli::csharp_practical_frontend_protocol::PracticalFrontendProtocolError>,
    expected: PracticalFrontendProtocolCode,
) {
    let error = result.err().expect("expected protocol rejection");
    assert_eq!(error.code(), expected, "{error}");
    let public = error.to_string();
    assert_eq!(public, expected.as_str());
    assert!(!public.contains('/'));
}

fn duplicate_at(input: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    let index = input
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("duplicate insertion point");
    let mut output = Vec::with_capacity(input.len() + replacement.len() - needle.len());
    output.extend_from_slice(&input[..index]);
    output.extend_from_slice(replacement);
    output.extend_from_slice(&input[index + needle.len()..]);
    output
}

fn request_fields(value: &PracticalJsonValue) -> Vec<&str> {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, _)| name.as_str())
        .collect()
}

fn object_field_mut<'a>(
    value: &'a mut PracticalJsonValue,
    name: &str,
) -> &'a mut PracticalJsonValue {
    value
        .as_object()
        .unwrap()
        .iter()
        .position(|(candidate, _)| candidate == name)
        .and_then(move |index| match value {
            PracticalJsonValue::Object(entries) => Some(&mut entries[index].1),
            _ => None,
        })
        .unwrap()
}

fn nested_array_first_mut<'a>(
    value: &'a mut PracticalJsonValue,
    object_name: &str,
    array_name: &str,
) -> &'a mut PracticalJsonValue {
    let object = object_field_mut(value, object_name);
    let array = object_field_mut(object, array_name);
    match array {
        PracticalJsonValue::Array(values) => &mut values[0],
        _ => panic!("array expected"),
    }
}

fn set_field(value: &mut PracticalJsonValue, name: &str, replacement: PracticalJsonValue) {
    let entries = match value {
        PracticalJsonValue::Object(entries) => entries,
        _ => panic!("object expected"),
    };
    entries
        .iter_mut()
        .find(|(candidate, _)| candidate == name)
        .expect("field")
        .1 = replacement;
}

fn remove_field(value: &mut PracticalJsonValue, index: usize) {
    match value {
        PracticalJsonValue::Object(entries) => {
            entries.remove(index);
        }
        _ => panic!("object expected"),
    }
}

fn push_field(value: &mut PracticalJsonValue, name: &str, field: PracticalJsonValue) {
    match value {
        PracticalJsonValue::Object(entries) => entries.push((name.to_owned(), field)),
        _ => panic!("object expected"),
    }
}

fn set_first_field(value: &mut PracticalJsonValue, replacement: PracticalJsonValue) {
    set_field_at(value, 0, replacement);
}

fn set_field_at(value: &mut PracticalJsonValue, index: usize, replacement: PracticalJsonValue) {
    match value {
        PracticalJsonValue::Object(entries) => entries[index].1 = replacement,
        _ => panic!("object expected"),
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
    registry["registry_sha256"] =
        Value::String(successor_profile_registry_hash(&registry).expect("candidate registry hash"));
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
    entry["entry_sha256"] =
        Value::String(successor_profile_entry_hash(&entry).expect("entry hash"));
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

fn foundation_descriptor() -> Value {
    json!({
        "schema": FOUNDATION_DESCRIPTOR_SCHEMA,
        "id": FOUNDATION_DESCRIPTOR_ID,
        "content_sha256": FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    })
}

fn practical_selection() -> Value {
    let mut selection = json!({
        "schema": CSHARP_PRACTICAL_SELECTION_SCHEMA,
        "compilation_id": "business.frontend",
        "source_paths": ["src/Order.cs"],
        "selected_root_ids": [canonical_source_declaration_id(&declaration_identity()).unwrap()],
        "sidecar_paths": ["contracts/order.json"],
        "selection_sha256": ZERO_SHA256
    });
    selection["selection_sha256"] =
        Value::String(csharp_practical_selection_hash(&selection).expect("selection hash"));
    selection
}

fn request_fixture(context: Value, selection: Value) -> Value {
    let mut request = json!({
        "schema": SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
        "semantic_context": context,
        "selection": selection,
        "request_sha256": ZERO_SHA256
    });
    request["request_sha256"] =
        Value::String(successor_validated_request_hash(&request).expect("request hash"));
    request
}

fn declaration_identity() -> SourceDeclarationIdentity {
    let owner = SourceDeclarationIdentity {
        namespace: "Business".into(),
        kind: SourceDeclarationKind::Type,
        containing_source_type_id: None,
        source_name: "Order".into(),
        parameter_type_ids: Vec::new(),
        result_type_id: None,
    };
    SourceDeclarationIdentity {
        namespace: "Business".into(),
        kind: SourceDeclarationKind::Method,
        containing_source_type_id: Some(canonical_source_declaration_id(&owner).unwrap()),
        source_name: "Run".into(),
        parameter_type_ids: Vec::new(),
        result_type_id: Some("mpk.csharp.value.i32.v1".into()),
    }
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
                .map(|(name, value)| (name.clone(), to_strict(value)))
                .collect(),
        ),
    }
}
