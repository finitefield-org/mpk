use mpk_cert::encode::{
    AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, Import, ProofNode,
    TheoryCertificate, ZERO_HASH,
};
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
    bind_closed_instances, bind_practical_artifact_context, build_concrete_operation_tables,
    build_semantic_bindings, canonical_source_declaration_id, capture_original_inputs,
    CapturedInputSet, ConcreteOperationTables, OriginalInput, OriginalInputKind,
    PracticalArtifactContext, SourceDeclarationIdentity, SourceDeclarationKind,
    ValidatedPracticalArtifact, SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA, SUCCESSOR_VC_SCHEMA,
};
use mpk_vc::csharp_practical_vc_model::{
    emit_csharp_practical_vc_skeleton, generate_csharp_practical_program_assembly_plan,
    generate_csharp_practical_vc, import_csharp_practical_program_assembly_plan_json,
    import_csharp_practical_vc_json, import_csharp_practical_vc_skeleton_json,
    ordinary_check_route, ordinary_control_route, ordinary_operation_route, ordinary_pattern_route,
    ordinary_type_route, validate_csharp_practical_certificate_structure, LaterProofOwner,
    OrdinaryCheckRoute, OrdinaryControlRoute, OrdinaryTypeRoute, PracticalVcErrorCode,
    PracticalVcSource, PracticalVcValidationPhase, BINDER_DEPTH_MAX, CERTIFICATE_V0_FORMAT,
    CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE, CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN,
    CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE, CSHARP_PRACTICAL_VC_HASH_DOMAIN,
    CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE, GENERATED_DECLARATIONS_MAX,
    ORDINARY_TERM_NODES_MAX, STATIC_TRANSFORMERS_MAX,
};
use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, derive_closed_instances,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_closed_root_set, validate_registered_foundation_bundle, AbruptCompletion,
    ClosedOperationSignature, ClosedOperationTag, ControlNode, ControlNodeTag, PatternTag,
    RequiredCheck, RequiredCheckTag,
};
use mpk_vc::csharp_practical_vir_validation::{
    canonical_csharp_practical_vir_transport, import_csharp_practical_vir_json, PracticalVirBlock,
    PracticalVirContents, PracticalVirFunction, PracticalVirImportContext, ValidatedPracticalVir,
};
use mpk_vc::successor_vc::SUCCESSOR_VC_SCHEMA as PREDECESSOR_SUCCESSOR_VC_SCHEMA;
use mpk_vc::{canonical_json_bytes, StrictJsonValue, VC_SCHEMA_VERSION};
use serde_json::{json, Map, Value};

const WORK_ITEM: &str = "CSHARP-03-T02-W06";
const SOURCE: &[u8] = b"public static class Order { public static int Run() => 1; }\n";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const UNIT_TYPE_ID: &str = "mpk.csharp.value.unit.v1";

struct Fixture {
    context: PracticalArtifactContext,
    captures: CapturedInputSet,
    roots_transport: Vec<u8>,
    closed_transport: Vec<u8>,
    semantic_bindings: ValidatedPracticalArtifact,
    operations: ConcreteOperationTables,
    root_id: String,
}

impl Fixture {
    fn import_context(&self) -> PracticalVirImportContext<'_> {
        PracticalVirImportContext {
            artifact_context: &self.context,
            captured_inputs: &self.captures,
            foundation_descriptor_transport: registered_foundation_descriptor_transport(),
            foundation_definitions_transport: registered_foundation_definitions_transport(),
            closed_roots_transport: &self.roots_transport,
            closed_instances_transport: &self.closed_transport,
            semantic_bindings_transport: self.semantic_bindings.canonical_bytes(),
            required_checks_transport: self.operations.required_checks().canonical_bytes(),
            operations_transport: self.operations.operations().canonical_bytes(),
        }
    }

    fn validated_vir(&self, label: &str) -> ValidatedPracticalVir {
        let transport = canonical_csharp_practical_vir_transport(
            self.import_context(),
            minimal_contents(&self.root_id, label),
        )
        .expect("candidate VIR transport");
        import_csharp_practical_vir_json(&transport, self.import_context())
            .expect("validated practical VIR")
    }
}

#[test]
fn csharp_03_t02_w06_round_trips_linked_vc_skeleton_and_assembly_plan() {
    assert_eq!(WORK_ITEM, "CSHARP-03-T02-W06");
    let fixture = build_fixture("business.vc.vector");
    let vir = fixture.validated_vir("vector");
    let source = practical_source(&fixture, &vir);

    let vc = generate_csharp_practical_vc(source).expect("successor VC");
    let vc_value: Value = serde_json::from_slice(vc.canonical_bytes()).unwrap();
    assert_eq!(vc_value["schema"], SUCCESSOR_VC_SCHEMA);
    assert_eq!(vc_value["source_ir"]["sha256"], vir.hash());
    assert_eq!(
        vc_value["ordinary_encoding_profile"],
        CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE
    );
    assert_eq!(
        vc_value["verification_limit_profile"],
        CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE
    );
    assert_eq!(
        vc_value["ordinary_term_forms"],
        json!(["sort", "var", "const", "app", "lam", "pi", "let"])
    );
    assert_eq!(vc.artifact_ref().schema(), SUCCESSOR_VC_SCHEMA);
    assert_eq!(vc.artifact_ref().sha256(), vc.hash());
    assert_eq!(
        vc.limits().ordinary_term_nodes_maximum(),
        ORDINARY_TERM_NODES_MAX
    );
    assert_eq!(
        vc.limits().generated_declarations_maximum(),
        GENERATED_DECLARATIONS_MAX
    );
    assert_eq!(vc.limits().binder_depth_maximum(), BINDER_DEPTH_MAX);
    assert_eq!(
        vc.limits().static_transformers_maximum(),
        STATIC_TRANSFORMERS_MAX
    );
    assert!(vc
        .obligation_groups()
        .iter()
        .all(|group| !group.subject_ids().is_empty()));
    assert_eq!(
        vc.obligation_groups()[0].id(),
        "vc.group.0000.ordinary_foundation"
    );
    assert!(!String::from_utf8_lossy(vc.canonical_bytes()).contains("intrinsic"));
    assert_eq!(
        import_csharp_practical_vc_json(vc.canonical_bytes(), source)
            .expect("VC re-import")
            .hash(),
        vc.hash()
    );

    let skeleton = emit_csharp_practical_vc_skeleton(source, &vc).expect("VC skeleton");
    let skeleton_value: Value = serde_json::from_slice(skeleton.canonical_bytes()).unwrap();
    assert_eq!(
        skeleton_value["schema"],
        SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA
    );
    assert_eq!(skeleton_value["source_vc"]["sha256"], vc.hash());
    assert_eq!(
        skeleton_value["program_assembly_profile"],
        CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE
    );
    assert_eq!(
        skeleton.theorem_declarations().len(),
        vc.obligation_groups().len()
    );
    assert_eq!(
        import_csharp_practical_vc_skeleton_json(skeleton.canonical_bytes(), source, &vc)
            .expect("skeleton re-import")
            .hash(),
        skeleton.hash()
    );

    let assembly = generate_csharp_practical_program_assembly_plan(source, &vc, &skeleton)
        .expect("ordinary-context assembly plan");
    assert_eq!(
        vc.hash(),
        "2a66a554285ed4f5f2263a7996bfbb87c35da421c38e81052dbe5369db5df598"
    );
    assert_eq!(
        skeleton.hash(),
        "03dbe460131c1240bf35aac8b7b954b9cac5a215a184efef66eea0735fd2f420"
    );
    assert_eq!(
        assembly.hash(),
        "90002cbc6f509d02365c2ec04c12c708112c7ab2ecaf3ad40448bf5ca7ac38c1"
    );
    let assembly_value: Value = serde_json::from_slice(assembly.canonical_bytes()).unwrap();
    assert_eq!(
        assembly_value["schema"],
        CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE
    );
    assert_eq!(assembly_value["certificate_format"], CERTIFICATE_V0_FORMAT);
    assert_eq!(assembly_value["source_ir"]["sha256"], vir.hash());
    assert_eq!(assembly_value["source_vc"]["sha256"], vc.hash());
    assert_eq!(assembly_value["source_skeleton"]["sha256"], skeleton.hash());
    assert_eq!(
        assembly_value["generated_declaration_kinds"],
        json!(["def", "theorem"])
    );
    assert_eq!(assembly_value["imports"], json!([]));
    assert_eq!(assembly_value["proof_node_table"], json!([]));
    assert_eq!(assembly_value["theory_certificates"], json!([]));
    assert_eq!(assembly.axiom_report().total_axiom_count(), 0);
    assert!(assembly.axiom_report().entries().is_empty());
    assert!(assembly
        .axiom_report()
        .declaration_dependencies()
        .is_empty());
    assert_eq!(
        import_csharp_practical_program_assembly_plan_json(
            assembly.canonical_bytes(),
            source,
            &vc,
            &skeleton,
        )
        .expect("assembly re-import")
        .hash(),
        assembly.hash()
    );

    assert_eq!(CSHARP_PRACTICAL_VC_HASH_DOMAIN.as_str(), "MPK-VC-3.0");
    assert_eq!(
        CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN.as_str(),
        "MPK-PROGRAM-ASSEMBLY-2.0"
    );
    assert_eq!(vc.hash().len(), 64);
    assert_eq!(skeleton.hash().len(), 64);
    assert_eq!(assembly.hash().len(), 64);
}

#[test]
fn csharp_03_t02_w06_rejects_noncanonical_mutated_and_cross_context_documents() {
    let fixture = build_fixture("business.vc.reject");
    let vir = fixture.validated_vir("reject");
    let source = practical_source(&fixture, &vir);
    let vc = generate_csharp_practical_vc(source).unwrap();

    let mut old_schema = vc.canonical_bytes().to_vec();
    replace_first(&mut old_schema, b"mpk.vc.v3", b"mpk.vc.v2");
    assert_vc_error(
        import_csharp_practical_vc_json(&old_schema, source),
        PracticalVcValidationPhase::Schema,
        PracticalVcErrorCode::Schema,
    );

    let mut noncanonical = vc.canonical_bytes().to_vec();
    noncanonical.push(b'\n');
    assert_vc_error(
        import_csharp_practical_vc_json(&noncanonical, source),
        PracticalVcValidationPhase::Canonical,
        PracticalVcErrorCode::Canonical,
    );

    let value: Value = serde_json::from_slice(vc.canonical_bytes()).unwrap();
    let reordered = serde_json::to_vec(&value).unwrap();
    assert_ne!(reordered, vc.canonical_bytes());
    assert_vc_error(
        import_csharp_practical_vc_json(&reordered, source),
        PracticalVcValidationPhase::Canonical,
        PracticalVcErrorCode::Canonical,
    );

    let mut duplicate = vc.canonical_bytes().to_vec();
    replace_first_resized(
        &mut duplicate,
        b"{\"schema\":\"mpk.vc.v3\"",
        b"{\"schema\":\"mpk.vc.v3\",\"schema\":\"mpk.vc.v3\"",
    );
    assert_vc_error(
        import_csharp_practical_vc_json(&duplicate, source),
        PracticalVcValidationPhase::Transport,
        PracticalVcErrorCode::Json,
    );

    let mut changed_encoding = vc.canonical_bytes().to_vec();
    mutate_string_field(&mut changed_encoding, "declaration_name");
    assert_vc_error(
        import_csharp_practical_vc_json(&changed_encoding, source),
        PracticalVcValidationPhase::Encoding,
        PracticalVcErrorCode::Encoding,
    );

    let mut changed_limit = vc.canonical_bytes().to_vec();
    replace_first(
        &mut changed_limit,
        b"\"ordinary_term_nodes_maximum\":262144",
        b"\"ordinary_term_nodes_maximum\":262145",
    );
    assert_vc_error(
        import_csharp_practical_vc_json(&changed_limit, source),
        PracticalVcValidationPhase::Limits,
        PracticalVcErrorCode::Limit,
    );

    let mut changed_hash = vc.canonical_bytes().to_vec();
    mutate_hash_field(&mut changed_hash, "vc_sha256");
    assert_vc_error(
        import_csharp_practical_vc_json(&changed_hash, source),
        PracticalVcValidationPhase::Hash,
        PracticalVcErrorCode::Hash,
    );

    let other = build_fixture("business.vc.other");
    let other_vir = other.validated_vir("other");
    assert_vc_error(
        import_csharp_practical_vc_json(vc.canonical_bytes(), practical_source(&other, &other_vir)),
        PracticalVcValidationPhase::Linkage,
        PracticalVcErrorCode::Linkage,
    );
}

#[test]
fn csharp_03_t02_w06_routes_every_closed_w03_form_without_intrinsics() {
    assert_eq!(
        ordinary_type_route("mpk.csharp.value.bool.v1").unwrap(),
        OrdinaryTypeRoute::CheckedBoolLeaf
    );
    assert_eq!(
        ordinary_type_route("mpk.csharp.value.i32.v1").unwrap(),
        OrdinaryTypeRoute::RegisteredBooleanCube
    );
    assert_eq!(
        ordinary_type_route("mpk.csharp.instance.option.example").unwrap(),
        OrdinaryTypeRoute::RegisteredBooleanCube
    );
    assert_eq!(
        ordinary_type_route("mpk.csharp.source.example").unwrap(),
        OrdinaryTypeRoute::ApplicationBooleanCubeProjection
    );
    assert_vc_error(
        ordinary_type_route("System.String"),
        PracticalVcValidationPhase::Encoding,
        PracticalVcErrorCode::Encoding,
    );

    let operation_tags = [
        ClosedOperationTag::Foundation,
        ClosedOperationTag::FieldRead,
        ClosedOperationTag::ValueConstruct,
        ClosedOperationTag::SourceCall,
        ClosedOperationTag::BindingProject,
        ClosedOperationTag::BindingReconstruct,
        ClosedOperationTag::StructuralEqual,
        ClosedOperationTag::CanonicalCompare,
        ClosedOperationTag::BoundaryParse,
        ClosedOperationTag::BoundaryFormat,
        ClosedOperationTag::Data,
        ClosedOperationTag::ExceptionConstruct,
        ClosedOperationTag::ExceptionIsType,
        ClosedOperationTag::ExceptionPayload,
    ];
    assert_eq!(operation_tags.len(), 14);
    for tag in operation_tags {
        let (_route, owner) = ordinary_operation_route(tag);
        assert!(owner.as_str().starts_with("CSHARP-03-T06-W"));
    }

    let check_cases = [
        (
            "already_initialized",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "construction_bound",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "incomplete",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "ownership",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "publication_bound",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "uninitialized",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        (
            "invalid_representation",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "obligation.output_bound",
            RequiredCheckTag::StaticObligation,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "parse_error.input_bound",
            RequiredCheckTag::ParseError,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        (
            "parse_error.syntax",
            RequiredCheckTag::ParseError,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        (
            "parse_error.noncanonical",
            RequiredCheckTag::ParseError,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        (
            "parse_error.scale_precision",
            RequiredCheckTag::ParseError,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        (
            "parse_error.range",
            RequiredCheckTag::ParseError,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        (
            "negative_length",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "exception.overflow",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "index_range",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "invalid_operation",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "exception.division_by_zero",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "exception.range",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "exception.null_receiver",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "exception.null_argument",
            RequiredCheckTag::Exception,
            LaterProofOwner::ExceptionalControl,
        ),
        (
            "event_bound",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::PureTransition,
        ),
        (
            "capacity",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "currency_mismatch",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "decimal_overflow",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "division_by_zero",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "duplicate_element",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "duplicate_key",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "empty_errors",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "invalid_currency",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "invalid_precision",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "invalid_rounding",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "invalid_scale",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "missing_key",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "precision",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "range",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        (
            "validation_bound",
            RequiredCheckTag::ErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
    ];
    assert_eq!(check_cases.len(), 37);
    for (check_id, tag, expected_owner) in check_cases {
        let (route, owner) = ordinary_check_route(check_id, tag).unwrap();
        let expected_route = match tag {
            RequiredCheckTag::StaticObligation => OrdinaryCheckRoute::BooleanPredicate,
            RequiredCheckTag::ParseError => OrdinaryCheckRoute::TaggedParseOutcome,
            RequiredCheckTag::Exception => OrdinaryCheckRoute::ClosedExceptionalEdge,
            RequiredCheckTag::ErrorOutcome => OrdinaryCheckRoute::TaggedErrorOutcome,
        };
        assert_eq!(route, expected_route, "{check_id}");
        assert_eq!(owner, expected_owner, "{check_id}");
    }
    assert_vc_error(
        ordinary_check_route("future", RequiredCheckTag::StaticObligation),
        PracticalVcValidationPhase::Encoding,
        PracticalVcErrorCode::Encoding,
    );
    assert_vc_error(
        ordinary_check_route("event_bound", RequiredCheckTag::StaticObligation),
        PracticalVcValidationPhase::Encoding,
        PracticalVcErrorCode::Encoding,
    );

    let construction = build_fixture_with_roots(
        "business.vc.construction",
        json!([{
            "origin": "source_construction",
            "provenance_id": "root.source.construction_i32",
            "type": {
                "kind": "instance",
                "template": "sequence_construction",
                "arguments": [{"kind": "primitive", "id": "i32"}]
            }
        }]),
    );
    let construction_vir = construction.validated_vir("construction");
    let construction_vc =
        generate_csharp_practical_vc(practical_source(&construction, &construction_vir)).unwrap();
    assert!(construction_vc
        .operation_encodings()
        .iter()
        .any(|operation| {
            operation.checks().iter().any(|check| {
                check.check_id() == "construction_bound"
                    && check.proof_owner() == LaterProofOwner::ConstructionAndTypeInvariants
            })
        }));

    let transition = build_fixture_with_roots(
        "business.vc.transition",
        json!([{
            "origin": "transition",
            "provenance_id": "root.transition.i32_i32_i32",
            "type": {
                "kind": "instance",
                "template": "transition",
                "arguments": [
                    {"kind": "primitive", "id": "i32"},
                    {"kind": "primitive", "id": "i32"},
                    {"kind": "primitive", "id": "i32"}
                ]
            }
        }]),
    );
    let transition_vir = transition.validated_vir("transition");
    let transition_vc =
        generate_csharp_practical_vc(practical_source(&transition, &transition_vir)).unwrap();
    assert!(transition_vc.operation_encodings().iter().any(|operation| {
        operation.checks().iter().any(|check| {
            check.check_id() == "event_bound"
                && check.proof_owner() == LaterProofOwner::PureTransition
        })
    }));

    let control_tags = [
        ControlNodeTag::Entry,
        ControlNodeTag::Operation,
        ControlNodeTag::Branch,
        ControlNodeTag::Jump,
        ControlNodeTag::LoopHeader,
        ControlNodeTag::PatternDecision,
        ControlNodeTag::Return,
        ControlNodeTag::Break,
        ControlNodeTag::Continue,
        ControlNodeTag::Throw,
        ControlNodeTag::Rethrow,
        ControlNodeTag::HandlerEntry,
        ControlNodeTag::FinallyEntry,
        ControlNodeTag::FinallyExit,
        ControlNodeTag::Exit,
    ];
    assert_eq!(control_tags.len(), 15);
    for tag in control_tags {
        let (_route, owner) = ordinary_control_route(tag);
        assert!(owner.as_str().starts_with("CSHARP-03-T06-W"));
    }

    let pattern_tags = [
        PatternTag::Constant,
        PatternTag::Discard,
        PatternTag::Var,
        PatternTag::Null,
        PatternTag::NotNull,
        PatternTag::Relational,
        PatternTag::Parenthesized,
        PatternTag::And,
        PatternTag::Or,
        PatternTag::Not,
        PatternTag::DeclarationType,
        PatternTag::ExactTag,
        PatternTag::Property,
        PatternTag::List,
    ];
    assert_eq!(pattern_tags.len(), 14);
    for tag in pattern_tags {
        let (route, owner) = ordinary_pattern_route(tag);
        assert_eq!(route, OrdinaryControlRoute::PatternDecision);
        assert_eq!(owner, LaterProofOwner::LoopSwitchAndPatterns);
    }
}

#[test]
fn csharp_03_t02_w06_rejects_nonempty_proof_theory_and_axiom_surfaces() {
    let fixture = build_fixture("business.vc.structure");
    let vir = fixture.validated_vir("structure");
    let source = practical_source(&fixture, &vir);
    let vc = generate_csharp_practical_vc(source).unwrap();
    let skeleton = emit_csharp_practical_vc_skeleton(source, &vc).unwrap();
    let assembly = generate_csharp_practical_program_assembly_plan(source, &vc, &skeleton).unwrap();

    let mut noncanonical_skeleton = skeleton.canonical_bytes().to_vec();
    noncanonical_skeleton.push(b'\n');
    assert_vc_error(
        import_csharp_practical_vc_skeleton_json(&noncanonical_skeleton, source, &vc),
        PracticalVcValidationPhase::Canonical,
        PracticalVcErrorCode::Canonical,
    );

    let mut changed_assembly_hash = assembly.canonical_bytes().to_vec();
    mutate_hash_field(&mut changed_assembly_hash, "assembly_sha256");
    assert_vc_error(
        import_csharp_practical_program_assembly_plan_json(
            &changed_assembly_hash,
            source,
            &vc,
            &skeleton,
        ),
        PracticalVcValidationPhase::Hash,
        PracticalVcErrorCode::Hash,
    );

    for (empty, nonempty) in [
        (
            b"\"proof_node_table\":[]".as_slice(),
            b"\"proof_node_table\":[{}]".as_slice(),
        ),
        (
            b"\"theory_certificates\":[]".as_slice(),
            b"\"theory_certificates\":[{}]".as_slice(),
        ),
    ] {
        let mut mutated = assembly.canonical_bytes().to_vec();
        replace_first_resized(&mut mutated, empty, nonempty);
        assert_vc_error(
            import_csharp_practical_program_assembly_plan_json(&mutated, source, &vc, &skeleton),
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::CertificateStructure,
        );
    }

    let certificate = empty_certificate();
    validate_csharp_practical_certificate_structure(&certificate)
        .expect("empty ordinary-term Certificate v0 structure");

    let mut with_proof = certificate.clone();
    with_proof.proof_node_table.push(ProofNode::Exact {
        term: 0,
        expected_type: 0,
    });
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&with_proof),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::CertificateStructure,
    );

    let mut with_import = certificate.clone();
    with_import.imports.push(Import {
        module_name: "Future".into(),
        export_hash: ZERO_HASH,
        certificate_hash: None,
    });
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&with_import),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::CertificateStructure,
    );

    let mut with_theory = certificate.clone();
    with_theory.theory_certificates.push(TheoryCertificate {
        format: "future".into(),
        payload: Vec::new(),
    });
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&with_theory),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::CertificateStructure,
    );

    let mut with_axiom = certificate.clone();
    with_axiom.declarations.push(Declaration {
        name: 0,
        kind: DeclarationKind::Axiom { ty: 0 },
    });
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&with_axiom),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::CertificateStructure,
    );

    let mut with_theory_primitive = certificate.clone();
    with_theory_primitive.declarations.push(Declaration {
        name: 0,
        kind: DeclarationKind::TheoryPrimitive { ty: 0 },
    });
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&with_theory_primitive),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::CertificateStructure,
    );

    let mut forged_report = certificate;
    forged_report.axiom_report.summary.total_axiom_count = 1;
    assert_vc_error(
        validate_csharp_practical_certificate_structure(&forged_report),
        PracticalVcValidationPhase::Assembly,
        PracticalVcErrorCode::AxiomReport,
    );
}

#[test]
fn csharp_03_t02_w06_leaves_predecessor_schema_identities_unchanged() {
    assert_eq!(VC_SCHEMA_VERSION, "mpk.vc.v1");
    assert_eq!(PREDECESSOR_SUCCESSOR_VC_SCHEMA, "mpk.vc.v2");
    assert_eq!(SUCCESSOR_VC_SCHEMA, "mpk.vc.v3");
}

fn practical_source<'a>(
    fixture: &'a Fixture,
    vir: &'a ValidatedPracticalVir,
) -> PracticalVcSource<'a> {
    PracticalVcSource {
        artifact_context: &fixture.context,
        captured_inputs: &fixture.captures,
        vir,
    }
}

fn build_fixture(compilation_id: &str) -> Fixture {
    build_fixture_with_roots(compilation_id, json!([]))
}

fn build_fixture_with_roots(compilation_id: &str, roots_value: Value) -> Fixture {
    let registry_value = candidate_registry();
    let registry_transport =
        canonical_successor_registry_transport(&registry_value).expect("registry transport");
    let registry = validate_candidate_successor_registry(&registry_transport).expect("registry");
    let root_id = source_declaration_id("Run");
    let request = request_fixture(
        context_fixture(&registry_value),
        practical_selection(compilation_id, &root_id),
    );
    let request = validate_successor_semantic_request(&registry, &canonical(&request))
        .expect("validated request");
    let foundation = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .expect("registered foundation");
    let context = bind_practical_artifact_context(&request, &foundation).expect("artifact context");
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
                bytes: b"{}".to_vec(),
            },
        ],
    )
    .expect("captured inputs");
    let roots_transport =
        canonical_closed_root_set_transport(&foundation, &roots_value, &json!({}))
            .expect("closed roots");
    let roots = validate_closed_root_set(&foundation, &roots_transport).expect("validated roots");
    let closed = derive_closed_instances(&foundation, &roots).expect("closed instances");
    let closed_ref = bind_closed_instances(&context, &foundation, &captures, &roots, &closed)
        .expect("closed-instance reference");
    let semantic_bindings =
        build_semantic_bindings(&context, &captures, Vec::new()).expect("semantic bindings");
    let mut signatures = foundation_signatures(&closed);
    signatures.push(ClosedOperationSignature {
        id: root_id.clone(),
        tag: ClosedOperationTag::SourceCall,
        argument_type_ids: Vec::new(),
        normal_result_type_id: UNIT_TYPE_ID.into(),
        ordered_checks: Vec::new(),
    });
    let operations =
        build_concrete_operation_tables(&context, &roots, &closed, &closed_ref, signatures)
            .expect("operation tables");
    Fixture {
        context,
        captures,
        roots_transport,
        closed_transport: closed.canonical_json().to_vec(),
        semantic_bindings,
        operations,
        root_id,
    }
}

fn foundation_signatures(
    closed: &mpk_vc::csharp_practical_vir_model::ClosedInstanceSet,
) -> Vec<ClosedOperationSignature> {
    closed
        .entries()
        .iter()
        .flat_map(|entry| entry["operation_definitions"].as_array().unwrap())
        .map(|operation| ClosedOperationSignature {
            id: operation["id"].as_str().unwrap().into(),
            tag: ClosedOperationTag::Foundation,
            argument_type_ids: operation["argument_type_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().into())
                .collect(),
            normal_result_type_id: operation["normal_result_type_id"].as_str().unwrap().into(),
            ordered_checks: operation["error_precedence"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| foundation_check(value.as_str().unwrap()))
                .collect(),
        })
        .collect()
}

fn foundation_check(id: &str) -> RequiredCheck {
    let (tag, failure_type_id) = match id {
        "already_initialized"
        | "construction_bound"
        | "incomplete"
        | "invalid_representation"
        | "obligation.output_bound"
        | "ownership"
        | "publication_bound"
        | "uninitialized" => (RequiredCheckTag::StaticObligation, None),
        "parse_error.input_bound"
        | "parse_error.syntax"
        | "parse_error.noncanonical"
        | "parse_error.scale_precision"
        | "parse_error.range" => (
            RequiredCheckTag::ParseError,
            Some("mpk.csharp.value.parse_error.v1".to_owned()),
        ),
        "negative_length" | "exception.overflow" => (
            RequiredCheckTag::Exception,
            Some("System.OverflowException".to_owned()),
        ),
        "index_range" => (
            RequiredCheckTag::Exception,
            Some("System.IndexOutOfRangeException".to_owned()),
        ),
        "invalid_operation" => (
            RequiredCheckTag::Exception,
            Some("System.InvalidOperationException".to_owned()),
        ),
        "exception.division_by_zero" => (
            RequiredCheckTag::Exception,
            Some("System.DivideByZeroException".to_owned()),
        ),
        "exception.range" => (
            RequiredCheckTag::Exception,
            Some("System.ArgumentOutOfRangeException".to_owned()),
        ),
        "exception.null_receiver" => (
            RequiredCheckTag::Exception,
            Some("System.NullReferenceException".to_owned()),
        ),
        "exception.null_argument" => (
            RequiredCheckTag::Exception,
            Some("System.ArgumentNullException".to_owned()),
        ),
        "capacity" | "currency_mismatch" | "decimal_overflow" | "division_by_zero"
        | "duplicate_element" | "duplicate_key" | "empty_errors" | "event_bound"
        | "invalid_currency" | "invalid_precision" | "invalid_rounding" | "invalid_scale"
        | "missing_key" | "precision" | "range" | "validation_bound" => (
            RequiredCheckTag::ErrorOutcome,
            Some("mpk.csharp.value.i32.v1".to_owned()),
        ),
        other => panic!("unregistered foundation check {other}"),
    };
    RequiredCheck {
        id: id.into(),
        tag,
        failure_type_id,
    }
}

fn minimal_contents(function_id: &str, label: &str) -> PracticalVirContents {
    PracticalVirContents {
        functions: vec![PracticalVirFunction {
            id: function_id.into(),
            parameter_values: Vec::new(),
            result_type_ids: Vec::new(),
            blocks: vec![
                empty_block(ControlNode {
                    id: format!("vir.node.{label}.entry"),
                    ordinal: 0,
                    tag: ControlNodeTag::Entry,
                    condition_type_id: None,
                    normal_successor_ids: vec![format!("vir.node.{label}.return")],
                    exceptional_successors: Vec::new(),
                    abrupt: None,
                    loop_id: None,
                    region_stack: Vec::new(),
                }),
                empty_block(ControlNode {
                    id: format!("vir.node.{label}.return"),
                    ordinal: 1,
                    tag: ControlNodeTag::Return,
                    condition_type_id: None,
                    normal_successor_ids: Vec::new(),
                    exceptional_successors: Vec::new(),
                    abrupt: Some(AbruptCompletion::Return {
                        value_type_id: None,
                    }),
                    loop_id: None,
                    region_stack: Vec::new(),
                }),
                empty_block(ControlNode {
                    id: format!("vir.node.{label}.exit"),
                    ordinal: 2,
                    tag: ControlNodeTag::Exit,
                    condition_type_id: None,
                    normal_successor_ids: Vec::new(),
                    exceptional_successors: Vec::new(),
                    abrupt: Some(AbruptCompletion::Normal),
                    loop_id: None,
                    region_stack: Vec::new(),
                }),
            ],
            loops: Vec::new(),
            patterns: Vec::new(),
            exception_regions: Vec::new(),
            unwind_plans: Vec::new(),
        }],
        ..PracticalVirContents::default()
    }
}

fn empty_block(node: ControlNode) -> PracticalVirBlock {
    PracticalVirBlock {
        node,
        phi_values: Vec::new(),
        condition_value_id: None,
        return_value_ids: Vec::new(),
        abrupt_value_id: None,
        handler_exception_value: None,
        invocation: None,
        ownership_in: Vec::new(),
        construction_actions: Vec::new(),
        ownership_out: Vec::new(),
    }
}

fn empty_certificate() -> Certificate {
    Certificate {
        module: "CSharpPracticalW06".into(),
        imports: Vec::new(),
        name_table: Vec::new(),
        level_table: Vec::new(),
        term_table: Vec::new(),
        proof_node_table: Vec::new(),
        declarations: Vec::new(),
        theory_certificates: Vec::new(),
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: None,
        hashes: CertificateHashes::default(),
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
        Value::String(successor_profile_registry_hash(&registry).expect("registry hash"));
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

fn practical_selection(compilation_id: &str, root_id: &str) -> Value {
    let mut selection = json!({
        "schema": CSHARP_PRACTICAL_SELECTION_SCHEMA,
        "compilation_id": compilation_id,
        "source_paths": ["src/Order.cs"],
        "selected_root_ids": [root_id],
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

fn source_declaration_id(name: &str) -> String {
    canonical_source_declaration_id(&SourceDeclarationIdentity {
        namespace: "Business".into(),
        kind: SourceDeclarationKind::Method,
        containing_source_type_id: Some(
            canonical_source_declaration_id(&SourceDeclarationIdentity {
                namespace: "Business".into(),
                kind: SourceDeclarationKind::Type,
                containing_source_type_id: None,
                source_name: "Order".into(),
                parameter_type_ids: Vec::new(),
                result_type_id: None,
            })
            .expect("source type ID"),
        ),
        source_name: name.into(),
        parameter_type_ids: Vec::new(),
        result_type_id: Some("mpk.csharp.value.i32.v1".into()),
    })
    .expect("source declaration ID")
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

fn assert_vc_error<T>(
    result: Result<T, mpk_vc::csharp_practical_vc_model::PracticalVcError>,
    phase: PracticalVcValidationPhase,
    code: PracticalVcErrorCode,
) {
    let error = match result {
        Ok(_) => panic!("expected {code:?} rejection"),
        Err(error) => error,
    };
    assert_eq!(error.phase(), phase, "{error}");
    assert_eq!(error.code(), code, "{error}");
}

fn mutate_hash_field(bytes: &mut [u8], field: &str) {
    let needle = format!("\"{field}\":\"");
    let start = bytes
        .windows(needle.len())
        .position(|window| window == needle.as_bytes())
        .unwrap_or_else(|| panic!("missing hash field {field}"))
        + needle.len();
    bytes[start] = if bytes[start] == b'0' { b'1' } else { b'0' };
}

fn mutate_string_field(bytes: &mut [u8], field: &str) {
    let needle = format!("\"{field}\":\"");
    let start = bytes
        .windows(needle.len())
        .position(|window| window == needle.as_bytes())
        .unwrap_or_else(|| panic!("missing string field {field}"))
        + needle.len();
    bytes[start] = if bytes[start] == b'M' { b'N' } else { b'M' };
}

fn replace_first(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("replacement source");
    bytes[start..start + from.len()].copy_from_slice(to);
}

fn replace_first_resized(bytes: &mut Vec<u8>, from: &[u8], to: &[u8]) {
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("replacement source");
    bytes.splice(start..start + from.len(), to.iter().copied());
}
