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
    build_semantic_bindings, canonical_source_declaration_id, canonical_source_stored_member_id,
    capture_original_inputs, ConcreteOperationTables, OriginalInput, OriginalInputKind,
    PracticalArtifactContext, SemanticBindingInput, SemanticBindingMember,
    SourceDeclarationIdentity, SourceDeclarationKind, SourceStoredMemberIdentity,
    SourceStoredMemberStorage, ValidatedPracticalArtifact, SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, derive_closed_instances,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_closed_root_set, validate_registered_foundation_bundle, AbruptCompletion,
    BindingTypeProjection, CatchHandler, ClosedInstanceSet, ClosedOperationSignature,
    ClosedOperationTag, ConstructionStatus, ControlNode, ControlNodeTag, ExceptionHandlerRegion,
    LoopRegion, OperationInvocation, PatternArm, PatternDecision, PatternTag, RequiredCheck,
    RequiredCheckTag, SequenceConstructionAction as ModelConstructionAction,
    SequenceConstructionState, SourceExceptionDefinition, TypedValueRef,
};
use mpk_vc::csharp_practical_vir_validation::{
    canonical_csharp_practical_vir_transport, import_csharp_practical_vir_json,
    PracticalConstructionAction, PracticalVirBlock, PracticalVirContents, PracticalVirFunction,
    PracticalVirImportContext, PracticalVirImportErrorCode, PracticalVirImportPhase,
    PracticalVirPhiIncoming, PracticalVirPhiValue,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::successor_source_artifacts::import_successor_vir_json;
use mpk_vc::{canonical_json_bytes, import_vir_json, StrictJsonValue};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

const WORK_ITEM: &str = "CSHARP-03-T02-W05";
const SOURCE: &[u8] = b"public static class Order { public static int Run() => 1; }\n";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const UNIT_TYPE_ID: &str = "mpk.csharp.value.unit.v1";
const I32_TYPE_ID: &str = "mpk.csharp.value.i32.v1";
const BOOL_TYPE_ID: &str = "mpk.csharp.value.bool.v1";

struct Fixture {
    context: PracticalArtifactContext,
    captures: mpk_vc::csharp_practical_source_artifacts::CapturedInputSet,
    roots_transport: Vec<u8>,
    closed_transport: Vec<u8>,
    closed: ClosedInstanceSet,
    semantic_bindings: ValidatedPracticalArtifact,
    operations: ConcreteOperationTables,
    operation_signatures: Vec<ClosedOperationSignature>,
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

    fn transport(&self, contents: PracticalVirContents) -> Vec<u8> {
        canonical_csharp_practical_vir_transport(self.import_context(), contents)
            .expect("candidate VIR transport")
    }
}

#[test]
fn csharp_03_t02_w05_round_trips_context_bound_monomorphic_vir() {
    let fixture = build_fixture("business.core", Vec::new());
    let transport = fixture.transport(minimal_contents(&fixture.root_id, "run"));
    let validated = import_csharp_practical_vir_json(&transport, fixture.import_context())
        .expect("strict successor VIR import");

    assert!(String::from_utf8_lossy(&transport)
        .starts_with("{\"schema\":\"mpk.vir.v2\",\"semantic_context\":"));
    assert_eq!(validated.canonical_bytes(), transport);
    assert_eq!(validated.functions().len(), 1);
    assert_eq!(validated.functions()[0].id, fixture.root_id);
    assert_eq!(validated.artifact_ref().schema(), SUCCESSOR_VIR_SCHEMA);
    assert_eq!(validated.artifact_ref().sha256(), validated.hash());
    assert_eq!(
        validated.artifact_ref().canonical_bytes(),
        u64::try_from(transport.len()).unwrap()
    );
    assert!(validated.source_exceptions().is_empty());
    assert!(validated.binding_projections().is_empty());
    assert!(validated.binding_commutations().is_empty());

    let expanded = build_option_fixture("business.expanded");
    let transport = expanded.transport(minimal_contents(&expanded.root_id, "expanded"));
    let validated = import_csharp_practical_vir_json(&transport, expanded.import_context())
        .expect("registered option<i32> expansion imports");
    let value: Value = serde_json::from_slice(validated.canonical_bytes()).unwrap();
    assert_eq!(value["expanded_foundation"].as_array().unwrap().len(), 1);
    let text = std::str::from_utf8(validated.canonical_bytes()).unwrap();
    assert!(!text.contains("\"template_id\""));
    assert!(!text.contains("mpk.csharp.semantic."));

    let mut handler = minimal_contents(&fixture.root_id, "handler");
    handler.functions[0].blocks[0].node.normal_successor_ids = vec!["vir.node.handler.try".into()];
    handler.functions[0].blocks[1].node.ordinal = 3;
    handler.functions[0].blocks[1].node.region_stack = vec!["region.handler".into()];
    handler.functions[0].blocks[2].node.ordinal = 4;
    let mut try_entry = empty_block(control_node(
        "vir.node.handler.try",
        1,
        ControlNodeTag::Jump,
        vec!["vir.node.handler.catch".into()],
        None,
    ));
    try_entry.node.region_stack = vec!["region.handler".into()];
    let mut catch = empty_block(control_node(
        "vir.node.handler.catch",
        2,
        ControlNodeTag::HandlerEntry,
        vec!["vir.node.handler.return".into()],
        None,
    ));
    catch.node.region_stack = vec!["region.handler".into()];
    catch.handler_exception_value = Some(TypedValueRef {
        id: "vir.value.handler.exception".into(),
        type_id: "mpk.csharp.value.exception.v1".into(),
    });
    handler.functions[0].blocks.insert(1, try_entry);
    handler.functions[0].blocks.insert(2, catch);
    handler.functions[0].exception_regions = vec![ExceptionHandlerRegion {
        id: "region.handler".into(),
        parent_region_id: None,
        nesting_depth: 0,
        try_entry_node_id: "vir.node.handler.try".into(),
        catches: vec![CatchHandler {
            ordinal: 0,
            exception_type_id: "System.ArgumentException".into(),
            filter: None,
            handler_entry_node_id: "vir.node.handler.catch".into(),
        }],
        finally_entry_node_id: None,
    }];
    import_csharp_practical_vir_json(
        &fixture.transport(handler.clone()),
        fixture.import_context(),
    )
    .expect("handler entry defines its closed exception value");
    handler.functions[0].exception_regions[0].catches[0].exception_type_id =
        "System.Exception".into();
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(handler), fixture.import_context()),
        PracticalVirImportPhase::Exception,
        PracticalVirImportErrorCode::Exception,
    );
}

#[test]
fn csharp_03_t02_w05_accepts_only_finite_reachable_acyclic_calls() {
    let helper_id = source_declaration_id("Helper");
    let signature = source_call_signature(&helper_id);
    let fixture = build_fixture("business.calls", vec![signature.clone()]);
    let mut functions = vec![
        calling_function(&fixture.root_id, "run", &signature),
        minimal_function(&helper_id, "helper"),
    ];
    functions.sort_by(|left, right| left.id.cmp(&right.id));
    let transport = fixture.transport(PracticalVirContents {
        functions,
        ..PracticalVirContents::default()
    });
    import_csharp_practical_vir_json(&transport, fixture.import_context())
        .expect("selected root reaches one acyclic helper");

    let self_signature = source_call_signature(&fixture.root_id);
    let cyclic = build_fixture("business.cycle", vec![self_signature.clone()]);
    let transport = cyclic.transport(PracticalVirContents {
        functions: vec![calling_function(&cyclic.root_id, "cycle", &self_signature)],
        ..PracticalVirContents::default()
    });
    assert_error(
        import_csharp_practical_vir_json(&transport, cyclic.import_context()),
        PracticalVirImportPhase::Graph,
        PracticalVirImportErrorCode::CallGraph,
    );
}

#[test]
fn csharp_03_t02_w05_requires_complete_operation_tables_and_normal_edge_results() {
    let extra = ClosedOperationSignature {
        id: "mpk.csharp.value.unit.v1.make".into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: Vec::new(),
        normal_result_type_id: UNIT_TYPE_ID.into(),
        ordered_checks: Vec::new(),
    };
    let fixture = build_fixture("business.extra_operation", vec![extra]);
    let transport = fixture.transport(minimal_contents(&fixture.root_id, "extra_operation"));
    assert_error(
        import_csharp_practical_vir_json(&transport, fixture.import_context()),
        PracticalVirImportPhase::ArtifactLinkage,
        PracticalVirImportErrorCode::Linkage,
    );

    let construction = build_construction_fixture("business.construction_bypass");
    let allocate = construction
        .operation_signatures
        .iter()
        .find(|signature| {
            signature.tag == ClosedOperationTag::Foundation && signature.id.ends_with(".allocate")
        })
        .expect("expanded construction allocate operation");
    let transport = construction.transport(PracticalVirContents {
        functions: vec![invoking_function(
            &construction.root_id,
            "construction_bypass",
            allocate,
        )],
        ..PracticalVirContents::default()
    });
    assert_error(
        import_csharp_practical_vir_json(&transport, construction.import_context()),
        PracticalVirImportPhase::Ownership,
        PracticalVirImportErrorCode::Ownership,
    );

    let merging = construction_merge_function(&construction, "construction_merge");
    let transport = construction.transport(PracticalVirContents {
        functions: vec![merging.clone()],
        ..PracticalVirContents::default()
    });
    import_csharp_practical_vir_json(&transport, construction.import_context())
        .expect("branch ownership facts merge by W03 intersection");

    let mut wrong_merge = merging;
    wrong_merge
        .blocks
        .iter_mut()
        .find(|block| block.node.id == "vir.node.construction_merge.merge")
        .expect("merge block")
        .ownership_in[0]
        .initialized_indices
        .insert(0);
    let transport = construction.transport(PracticalVirContents {
        functions: vec![wrong_merge],
        ..PracticalVirContents::default()
    });
    assert_error(
        import_csharp_practical_vir_json(&transport, construction.import_context()),
        PracticalVirImportPhase::Ownership,
        PracticalVirImportErrorCode::Ownership,
    );

    let helper_id = source_declaration_id("BoolHelper");
    let root_id = source_declaration_id("Run");
    let root_signature = ClosedOperationSignature {
        id: root_id.clone(),
        tag: ClosedOperationTag::SourceCall,
        argument_type_ids: vec![BOOL_TYPE_ID.into()],
        normal_result_type_id: UNIT_TYPE_ID.into(),
        ordered_checks: Vec::new(),
    };
    let helper_signature = ClosedOperationSignature {
        id: helper_id.clone(),
        tag: ClosedOperationTag::SourceCall,
        argument_type_ids: vec![BOOL_TYPE_ID.into()],
        normal_result_type_id: BOOL_TYPE_ID.into(),
        ordered_checks: Vec::new(),
    };
    let fixture = build_fixture(
        "business.bypassed_result",
        vec![root_signature, helper_signature.clone()],
    );
    let mut functions = vec![
        bypassed_result_function(&root_id, &helper_signature),
        bool_identity_function(&helper_id),
    ];
    functions.sort_by(|left, right| left.id.cmp(&right.id));
    let transport = fixture.transport(PracticalVirContents {
        functions,
        ..PracticalVirContents::default()
    });
    assert_error(
        import_csharp_practical_vir_json(&transport, fixture.import_context()),
        PracticalVirImportPhase::Dominance,
        PracticalVirImportErrorCode::Dominance,
    );
}

#[test]
fn csharp_03_t02_w05_rejects_control_dominance_ownership_exception_and_binding_mutations() {
    let fixture = build_fixture("business.graphs", Vec::new());

    let mut control = minimal_contents(&fixture.root_id, "control");
    control.functions[0].blocks[0].node.normal_successor_ids = vec!["vir.node.missing".into()];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(control), fixture.import_context()),
        PracticalVirImportPhase::Graph,
        PracticalVirImportErrorCode::Control,
    );

    let mut dominance = minimal_contents(&fixture.root_id, "dominance");
    dominance.functions[0].blocks[1].phi_values = vec![PracticalVirPhiValue {
        value: TypedValueRef {
            id: "vir.value.dominance.phi".into(),
            type_id: I32_TYPE_ID.into(),
        },
        incoming: vec![PracticalVirPhiIncoming {
            predecessor_node_id: "vir.node.dominance.entry".into(),
            value_id: "vir.value.missing".into(),
        }],
    }];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(dominance), fixture.import_context()),
        PracticalVirImportPhase::Dominance,
        PracticalVirImportErrorCode::Dominance,
    );

    let mut signature = minimal_contents(&fixture.root_id, "signature");
    signature.functions[0].parameter_values = vec![TypedValueRef {
        id: "vir.value.signature.parameter".into(),
        type_id: I32_TYPE_ID.into(),
    }];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(signature), fixture.import_context()),
        PracticalVirImportPhase::Vocabulary,
        PracticalVirImportErrorCode::Operation,
    );

    let mut return_shape = minimal_contents(&fixture.root_id, "return_shape");
    return_shape.functions[0].blocks[1].node.abrupt = Some(AbruptCompletion::Return {
        value_type_id: Some(I32_TYPE_ID.into()),
    });
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(return_shape),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Graph,
        PracticalVirImportErrorCode::Control,
    );

    let mut ownership = minimal_contents(&fixture.root_id, "ownership");
    ownership.functions[0].blocks[0].ownership_out = vec![invalid_construction_state()];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(ownership), fixture.import_context()),
        PracticalVirImportPhase::Ownership,
        PracticalVirImportErrorCode::Ownership,
    );

    let mut construction_value = minimal_contents(&fixture.root_id, "construction_value");
    construction_value.functions[0].blocks[0]
        .node
        .normal_successor_ids = vec!["vir.node.construction_value.fill".into()];
    construction_value.functions[0].blocks[1].node.ordinal = 2;
    construction_value.functions[0].blocks[2].node.ordinal = 3;
    let mut fill = empty_block(control_node(
        "vir.node.construction_value.fill",
        1,
        ControlNodeTag::Operation,
        vec!["vir.node.construction_value.return".into()],
        None,
    ));
    fill.construction_actions = vec![PracticalConstructionAction::Fill {
        construction_id: "construction.missing".into(),
        actor_id: "owner.missing".into(),
        index: 0,
        value: TypedValueRef {
            id: "vir.value.construction.missing".into(),
            type_id: I32_TYPE_ID.into(),
        },
    }];
    construction_value.functions[0].blocks.insert(1, fill);
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(construction_value),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Dominance,
        PracticalVirImportErrorCode::Dominance,
    );

    let mut exception = minimal_contents(&fixture.root_id, "exception");
    exception.source_exceptions = vec![SourceExceptionDefinition {
        type_id: format!("mpk.csharp.source.{}", "0".repeat(64)),
        sealed: true,
        direct_base_type_id: "System.Exception".into(),
        payload_member_ids: Vec::new(),
    }];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(exception), fixture.import_context()),
        PracticalVirImportPhase::Exception,
        PracticalVirImportErrorCode::Exception,
    );

    let (bound, projection) = build_instant_binding_fixture("business.binding", "i64");
    let transport = bound.transport(PracticalVirContents {
        binding_projections: vec![projection],
        functions: vec![minimal_function(&bound.root_id, "binding")],
        ..PracticalVirContents::default()
    });
    import_csharp_practical_vir_json(&transport, bound.import_context())
        .expect("source-linked instant projection imports");

    let (wrong_member_type, projection) =
        build_instant_binding_fixture("business.binding_wrong_type", "i32");
    assert_error(
        canonical_csharp_practical_vir_transport(
            wrong_member_type.import_context(),
            PracticalVirContents {
                binding_projections: vec![projection],
                functions: vec![minimal_function(
                    &wrong_member_type.root_id,
                    "binding_wrong_type",
                )],
                ..PracticalVirContents::default()
            },
        ),
        PracticalVirImportPhase::Binding,
        PracticalVirImportErrorCode::Binding,
    );

    let mut binding = minimal_contents(&fixture.root_id, "binding");
    binding.binding_projections = vec![BindingTypeProjection {
        id: "projection.unregistered".into(),
        binding_id: "binding.unregistered".into(),
        source_type_id: I32_TYPE_ID.into(),
        semantic_type_id: I32_TYPE_ID.into(),
        project: ClosedOperationSignature {
            id: "binding.project.unregistered".into(),
            tag: ClosedOperationTag::BindingProject,
            argument_type_ids: vec![I32_TYPE_ID.into()],
            normal_result_type_id: I32_TYPE_ID.into(),
            ordered_checks: Vec::new(),
        },
        reconstruct: ClosedOperationSignature {
            id: "binding.reconstruct.unregistered".into(),
            tag: ClosedOperationTag::BindingReconstruct,
            argument_type_ids: vec![I32_TYPE_ID.into()],
            normal_result_type_id: I32_TYPE_ID.into(),
            ordered_checks: Vec::new(),
        },
    }];
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(binding), fixture.import_context()),
        PracticalVirImportPhase::Vocabulary,
        PracticalVirImportErrorCode::Operation,
    );
}

#[test]
fn csharp_03_t02_w05_recomputes_foundation_and_rejects_cross_artifact_splicing() {
    let fixture = build_fixture("business.linkage", Vec::new());
    let transport = fixture.transport(minimal_contents(&fixture.root_id, "linkage"));

    let mut foundation = registered_foundation_definitions_transport().to_vec();
    replace_first(&mut foundation, b"\"unit\"", b"\"faux\"");
    let mut context = fixture.import_context();
    context.foundation_definitions_transport = &foundation;
    assert_error(
        import_csharp_practical_vir_json(&transport, context),
        PracticalVirImportPhase::Foundation,
        PracticalVirImportErrorCode::Foundation,
    );

    let mut closed = fixture.closed_transport.clone();
    mutate_hash_field(&mut closed, "closed_set_sha256");
    let mut context = fixture.import_context();
    context.closed_instances_transport = &closed;
    assert_error(
        import_csharp_practical_vir_json(&transport, context),
        PracticalVirImportPhase::Foundation,
        PracticalVirImportErrorCode::Foundation,
    );

    let mut checks = fixture
        .operations
        .required_checks()
        .canonical_bytes()
        .to_vec();
    mutate_hash_field(&mut checks, "required_checks_sha256");
    let mut context = fixture.import_context();
    context.required_checks_transport = &checks;
    assert_error(
        import_csharp_practical_vir_json(&transport, context),
        PracticalVirImportPhase::ArtifactLinkage,
        PracticalVirImportErrorCode::Linkage,
    );

    let alternate_captures = capture_original_inputs(
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
                bytes: b"{}".to_vec(),
            },
        ],
    )
    .expect("alternate captured inputs");
    let mut context = fixture.import_context();
    context.captured_inputs = &alternate_captures;
    assert_error(
        import_csharp_practical_vir_json(&transport, context),
        PracticalVirImportPhase::ArtifactLinkage,
        PracticalVirImportErrorCode::Linkage,
    );

    let crossed = build_fixture("business.other", Vec::new());
    assert_error(
        import_csharp_practical_vir_json(&transport, crossed.import_context()),
        PracticalVirImportPhase::ArtifactLinkage,
        PracticalVirImportErrorCode::Linkage,
    );

    let crossed_profile = String::from_utf8(transport.clone()).unwrap().replacen(
        "mpk.csharp.practical.v1",
        "mpk.csharp.scalar.v0",
        1,
    );
    assert_error(
        import_csharp_practical_vir_json(crossed_profile.as_bytes(), fixture.import_context()),
        PracticalVirImportPhase::Context,
        PracticalVirImportErrorCode::Context,
    );

    let mut bad_hash = transport.clone();
    mutate_hash_field(&mut bad_hash, "vir_sha256");
    assert_error(
        import_csharp_practical_vir_json(&bad_hash, fixture.import_context()),
        PracticalVirImportPhase::Hash,
        PracticalVirImportErrorCode::Hash,
    );

    let noncanonical_context = String::from_utf8(transport).unwrap().replacen(
        "\"semantic_context\":{",
        "\"semantic_context\":{ ",
        1,
    );
    assert_error(
        import_csharp_practical_vir_json(noncanonical_context.as_bytes(), fixture.import_context()),
        PracticalVirImportPhase::Context,
        PracticalVirImportErrorCode::Context,
    );
}

#[test]
fn csharp_03_t02_w05_enforces_the_generic_free_barrier_before_shape_decoding() {
    let fixture = build_fixture("business.generic", Vec::new());
    let transport = fixture.transport(minimal_contents(&fixture.root_id, "generic"));
    let suffix = std::str::from_utf8(&transport[1..]).unwrap();
    let mutations = [
        format!("{{\"type_parameter\":\"T\",{suffix}"),
        format!("{{\"payload\":\"Foo<T>\",{suffix}"),
        format!("{{\"payload\":\"generic_definition\",{suffix}"),
        format!("{{\"payload\":\"System.Collections.Generic.List\",{suffix}"),
        format!("{{\"generic_call\":true,{suffix}"),
        format!("{{\"payload\":\"mpk.csharp.semantic.option.v1\",{suffix}"),
    ];
    for mutation in mutations {
        assert_error(
            import_csharp_practical_vir_json(mutation.as_bytes(), fixture.import_context()),
            PracticalVirImportPhase::GenericBarrier,
            PracticalVirImportErrorCode::Generic,
        );
    }
}

#[test]
fn csharp_03_t02_w05_rejects_bounded_parser_fuzz_seeds_and_structural_limits() {
    let fixture = build_fixture("business.limits", Vec::new());
    let malformed: &[&[u8]] = &[
        b"",
        b"{",
        b"[]",
        b"\xef\xbb\xbf{\"schema\":\"mpk.vir.v2\"}",
        b"{\"schema\":\"mpk.vir.v2\",\"schema\":\"mpk.vir.v2\"}",
        b"{\"schema\":\"mpk.vir.v2\",\"number\":1.0}",
        b"{\"schema\":\"mpk.vir.v2\",\"escape\":\"\\u0041\"}",
    ];
    for seed in malformed {
        assert!(
            import_csharp_practical_vir_json(seed, fixture.import_context()).is_err(),
            "bounded malformed seed must reject: {seed:?}"
        );
    }

    let mut deep = vec![b'['; 257];
    deep.push(b'0');
    deep.extend(std::iter::repeat_n(b']', 257));
    assert_error(
        import_csharp_practical_vir_json(&deep, fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let large_string = format!(
        "{{\"schema\":\"{}\",\"payload\":\"{}\"}}",
        SUCCESSOR_VIR_SCHEMA,
        "x".repeat(1_048_577)
    );
    assert_error(
        import_csharp_practical_vir_json(large_string.as_bytes(), fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let functions = (0..129)
        .map(|ordinal| {
            minimal_function(
                &format!("mpk.csharp.source.{ordinal:064x}"),
                &format!("limit{ordinal}"),
            )
        })
        .collect();
    let transport = fixture.transport(PracticalVirContents {
        functions,
        ..PracticalVirContents::default()
    });
    assert_error(
        import_csharp_practical_vir_json(&transport, fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut parameters = minimal_contents(&fixture.root_id, "parameter_limit");
    parameters.functions[0].parameter_values = (0..257)
        .map(|ordinal| TypedValueRef {
            id: format!("vir.value.parameter.{ordinal:04}"),
            type_id: I32_TYPE_ID.into(),
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(parameters), fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut phi_values = minimal_contents(&fixture.root_id, "phi_limit");
    phi_values.functions[0].blocks[1].phi_values = (0..4_097)
        .map(|ordinal| PracticalVirPhiValue {
            value: TypedValueRef {
                id: format!("vir.value.phi.{ordinal:04}"),
                type_id: I32_TYPE_ID.into(),
            },
            incoming: Vec::new(),
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(phi_values), fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut edges = minimal_contents(&fixture.root_id, "edge_limit");
    edges.functions[0].blocks[0].node.normal_successor_ids =
        std::iter::repeat_n("vir.node.edge_limit.return".into(), 16_001).collect();
    assert_error(
        import_csharp_practical_vir_json(&fixture.transport(edges), fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut constructions = minimal_contents(&fixture.root_id, "construction_limit");
    constructions.functions[0].blocks[0].node.tag = ControlNodeTag::Operation;
    constructions.functions[0].blocks[0].construction_actions = (0..33)
        .map(|ordinal| PracticalConstructionAction::Allocate {
            construction_id: format!("construction.{ordinal:04}"),
            instance_id: "foundation.instance.unused".into(),
            owner_id: "owner.unused".into(),
            length: 0,
            default_eligible: false,
            publication_length_maximum: 0,
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(constructions),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut nested_loops = minimal_contents(&fixture.root_id, "loop_limit");
    nested_loops.functions[0].loops = (0..9)
        .map(|depth| LoopRegion {
            id: format!("loop.{depth:04}"),
            parent_loop_id: (depth > 0).then(|| format!("loop.{:04}", depth - 1)),
            header_node_id: "loop.header".into(),
            body_entry_node_id: "loop.body".into(),
            continue_target_node_id: "loop.continue".into(),
            break_target_node_id: "loop.break".into(),
            backedge_source_ids: vec!["loop.backedge".into()],
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(nested_loops),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut pattern_arms = minimal_contents(&fixture.root_id, "pattern_limit");
    pattern_arms.functions[0].patterns = (0..2)
        .map(|decision| PatternDecision {
            node_id: format!("vir.node.pattern_limit.decision.{decision}"),
            governing_value_id: "vir.value.unused".into(),
            governing_type_id: I32_TYPE_ID.into(),
            governing_evaluation_count: 1,
            expression: false,
            exhaustive: true,
            arms: (0..129)
                .map(|ordinal| PatternArm {
                    ordinal,
                    tag: PatternTag::Discard,
                    target_node_id: "vir.node.pattern_limit.return".into(),
                    guard_ordinal: None,
                    guard_type_id: None,
                    bound_parameter_type_ids: Vec::new(),
                    property_accesses: Vec::new(),
                    finite_sealed_type: false,
                    bounded_list: false,
                    has_slice: false,
                })
                .collect(),
            no_match_target_id: None,
            non_exhaustive_exceptional_successor: None,
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(pattern_arms),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let mut exception_regions = minimal_contents(&fixture.root_id, "exception_region_limit");
    exception_regions.functions[0].exception_regions = (0..2)
        .map(|region| ExceptionHandlerRegion {
            id: format!("region.limit.{region}"),
            parent_region_id: None,
            nesting_depth: 0,
            try_entry_node_id: "vir.node.exception_region_limit.entry".into(),
            catches: (0..17)
                .map(|ordinal| CatchHandler {
                    ordinal,
                    exception_type_id: "System.Exception".into(),
                    filter: None,
                    handler_entry_node_id: format!("vir.node.limit.handler.{region}.{ordinal:04}"),
                })
                .collect(),
            finally_entry_node_id: None,
        })
        .collect();
    assert_error(
        import_csharp_practical_vir_json(
            &fixture.transport(exception_regions),
            fixture.import_context(),
        ),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );

    let valid = fixture.transport(minimal_contents(&fixture.root_id, "live_limit"));
    let states = std::iter::repeat_n("{}", 9).collect::<Vec<_>>().join(",");
    let over_live = String::from_utf8(valid).unwrap().replacen(
        "\"ownership_in\":[]",
        &format!("\"ownership_in\":[{states}]"),
        1,
    );
    assert_error(
        import_csharp_practical_vir_json(over_live.as_bytes(), fixture.import_context()),
        PracticalVirImportPhase::Resource,
        PracticalVirImportErrorCode::Limit,
    );
}

#[test]
fn csharp_03_t02_w05_keeps_all_vir_parser_families_disjoint() {
    let fixture = build_fixture("business.schema", Vec::new());
    let transport = fixture.transport(minimal_contents(&fixture.root_id, "schema"));
    assert!(import_vir_json(&transport).is_err());

    let registry = validate_semantic_profile_registry(
        include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("installed predecessor registry");
    assert!(import_successor_vir_json(&transport, &registry).is_err());

    for predecessor in [
        br#"{"schema":"mpk.vir.v0"}"#.as_slice(),
        br#"{"schema":"mpk.vir.v1"}"#.as_slice(),
    ] {
        assert_error(
            import_csharp_practical_vir_json(predecessor, fixture.import_context()),
            PracticalVirImportPhase::Schema,
            PracticalVirImportErrorCode::Schema,
        );
    }

    let mut noncanonical = transport;
    noncanonical.push(b'\n');
    assert_error(
        import_csharp_practical_vir_json(&noncanonical, fixture.import_context()),
        PracticalVirImportPhase::Transport,
        PracticalVirImportErrorCode::Canonical,
    );
}

fn build_fixture(compilation_id: &str, signatures: Vec<ClosedOperationSignature>) -> Fixture {
    build_fixture_with_roots(compilation_id, signatures, json!([]))
}

fn build_option_fixture(compilation_id: &str) -> Fixture {
    build_fixture_with_roots(
        compilation_id,
        Vec::new(),
        json!([{
            "origin": "source_nullable",
            "provenance_id": "root.source.option_i32",
            "type": {
                "kind": "instance",
                "template": "option",
                "arguments": [{"kind": "primitive", "id": "i32"}]
            }
        }]),
    )
}

fn build_construction_fixture(compilation_id: &str) -> Fixture {
    let root_id = source_declaration_id("Run");
    build_fixture_with_roots(
        compilation_id,
        vec![ClosedOperationSignature {
            id: root_id,
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![I32_TYPE_ID.into(), BOOL_TYPE_ID.into()],
            normal_result_type_id: UNIT_TYPE_ID.into(),
            ordered_checks: Vec::new(),
        }],
        json!([{
            "origin": "source_construction",
            "provenance_id": "root.source.construction_i32",
            "type": {
                "kind": "instance",
                "template": "sequence_construction",
                "arguments": [{"kind": "primitive", "id": "i32"}]
            }
        }]),
    )
}

fn build_instant_binding_fixture(
    compilation_id: &str,
    carrier: &str,
) -> (Fixture, BindingTypeProjection) {
    let registry_value = candidate_registry();
    let registry_transport =
        canonical_successor_registry_transport(&registry_value).expect("registry transport");
    let registry = validate_candidate_successor_registry(&registry_transport).expect("registry");
    let root_id = source_declaration_id("Run");
    let selection = practical_selection(compilation_id, &root_id);
    let request = request_fixture(context_fixture(&registry_value), selection);
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
    let source_hash = captures
        .entry("src/Order.cs")
        .expect("captured source")
        .raw_sha256()
        .to_owned();
    let source_identity = SourceDeclarationIdentity {
        namespace: "Business.Orders".into(),
        kind: SourceDeclarationKind::Type,
        containing_source_type_id: None,
        source_name: "BusinessInstant".into(),
        parameter_type_ids: Vec::new(),
        result_type_id: None,
    };
    let source_type_id = canonical_source_declaration_id(&source_identity).unwrap();
    let member_type = json!({"kind": "primitive", "id": carrier});
    let member_id = canonical_source_stored_member_id(&SourceStoredMemberIdentity {
        owner_source_type_id: source_type_id.clone(),
        source_name: "Milliseconds".into(),
        closed_type: member_type.clone(),
        storage: SourceStoredMemberStorage::ReadonlyField,
    })
    .unwrap();
    let source_type = json!({
        "id": source_type_id,
        "identity": {
            "kind": "type",
            "namespace": source_identity.namespace,
            "owner": "",
            "name": source_identity.source_name,
            "parameter_type_ids": [],
            "result_type_id": "",
        },
        "kind": "readonly_struct",
        "members": [{
            "id": member_id,
            "name": "Milliseconds",
            "type": member_type,
            "storage": "readonly_field",
            "ordinal": 0,
            "required": false,
        }],
        "enum_values": [],
        "enum_underlying": null,
        "actual_default": {member_id.clone(): 0},
        "public_default": true,
        "identity_sensitive": false,
        "source_sha256": source_hash,
    });
    let roots_transport = canonical_closed_root_set_transport(
        &foundation,
        &json!([{
            "origin": "semantic_binding",
            "provenance_id": "root.binding.instant",
            "type": {"kind": "source", "id": source_type_id},
        }]),
        &json!({source_type_id.clone(): source_type}),
    )
    .expect("binding roots");
    let roots = validate_closed_root_set(&foundation, &roots_transport).expect("validated roots");
    let closed = derive_closed_instances(&foundation, &roots).expect("closed instances");
    let closed_ref = bind_closed_instances(&context, &foundation, &captures, &roots, &closed)
        .expect("closed-instance reference");
    let semantic_bindings = build_semantic_bindings(
        &context,
        &captures,
        vec![SemanticBindingInput {
            source_type_id: source_type_id.clone(),
            source_content_sha256: source_hash,
            role: "instant".into(),
            member_map: vec![SemanticBindingMember {
                role: "milliseconds".into(),
                member_id,
            }],
            tag_arms: Vec::new(),
            inferred_argument_ids: Vec::new(),
            default_arm: "ineligible".into(),
            bounds: Vec::new(),
            operation_map: Vec::new(),
        }],
    )
    .expect("semantic binding");
    let binding_hash = semantic_bindings
        .value()
        .get("bindings")
        .and_then(|value| value.as_array())
        .and_then(|bindings| bindings.first())
        .and_then(|binding| binding.get("binding_sha256"))
        .and_then(|value| value.as_str())
        .expect("binding hash")
        .to_owned();
    let project = ClosedOperationSignature {
        id: format!("binding.project.{binding_hash}"),
        tag: ClosedOperationTag::BindingProject,
        argument_type_ids: vec![source_type_id.clone()],
        normal_result_type_id: "mpk.csharp.value.instant.v1".into(),
        ordered_checks: Vec::new(),
    };
    let reconstruct = ClosedOperationSignature {
        id: format!("binding.reconstruct.{binding_hash}"),
        tag: ClosedOperationTag::BindingReconstruct,
        argument_type_ids: vec!["mpk.csharp.value.instant.v1".into()],
        normal_result_type_id: source_type_id.clone(),
        ordered_checks: Vec::new(),
    };
    let mut operation_signatures = vec![project.clone(), reconstruct.clone()];
    operation_signatures.push(source_call_signature(&root_id));
    let operations = build_concrete_operation_tables(
        &context,
        &roots,
        &closed,
        &closed_ref,
        operation_signatures.clone(),
    )
    .expect("binding operation tables");
    (
        Fixture {
            context,
            captures,
            roots_transport,
            closed_transport: closed.canonical_json().to_vec(),
            closed,
            semantic_bindings,
            operations,
            operation_signatures,
            root_id,
        },
        BindingTypeProjection {
            id: format!("projection.{binding_hash}"),
            binding_id: format!("binding.{binding_hash}"),
            source_type_id,
            semantic_type_id: "mpk.csharp.value.instant.v1".into(),
            project,
            reconstruct,
        },
    )
}

fn build_fixture_with_roots(
    compilation_id: &str,
    mut signatures: Vec<ClosedOperationSignature>,
    roots_value: Value,
) -> Fixture {
    assert_eq!(WORK_ITEM, "CSHARP-03-T02-W05");
    let registry_value = candidate_registry();
    let registry_transport =
        canonical_successor_registry_transport(&registry_value).expect("registry transport");
    let registry = validate_candidate_successor_registry(&registry_transport).expect("registry");
    let root_id = source_declaration_id("Run");
    let selection = practical_selection(compilation_id, &root_id);
    let request = request_fixture(context_fixture(&registry_value), selection);
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
    signatures.extend(foundation_signatures(&closed));
    if !signatures.iter().any(|signature| signature.id == root_id) {
        signatures.push(source_call_signature(&root_id));
    }
    let operation_signatures = signatures.clone();
    let operations =
        build_concrete_operation_tables(&context, &roots, &closed, &closed_ref, signatures)
            .expect("operation tables");
    Fixture {
        context,
        captures,
        roots_transport,
        closed_transport: closed.canonical_json().to_vec(),
        closed,
        semantic_bindings,
        operations,
        operation_signatures,
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
    match id {
        "invalid_operation" => RequiredCheck {
            id: id.into(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some("System.InvalidOperationException".into()),
        },
        "negative_length" => RequiredCheck {
            id: id.into(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some("System.OverflowException".into()),
        },
        "index_range" => RequiredCheck {
            id: id.into(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some("System.IndexOutOfRangeException".into()),
        },
        "construction_bound"
        | "ownership"
        | "uninitialized"
        | "already_initialized"
        | "incomplete"
        | "publication_bound" => RequiredCheck {
            id: id.into(),
            tag: RequiredCheckTag::StaticObligation,
            failure_type_id: None,
        },
        other => panic!("unexpected option check {other}"),
    }
}

fn minimal_contents(function_id: &str, label: &str) -> PracticalVirContents {
    PracticalVirContents {
        functions: vec![minimal_function(function_id, label)],
        ..PracticalVirContents::default()
    }
}

fn minimal_function(function_id: &str, label: &str) -> PracticalVirFunction {
    PracticalVirFunction {
        id: function_id.into(),
        parameter_values: Vec::new(),
        result_type_ids: Vec::new(),
        blocks: vec![
            empty_block(control_node(
                &format!("vir.node.{label}.entry"),
                0,
                ControlNodeTag::Entry,
                vec![format!("vir.node.{label}.return")],
                None,
            )),
            empty_block(control_node(
                &format!("vir.node.{label}.return"),
                1,
                ControlNodeTag::Return,
                Vec::new(),
                Some(AbruptCompletion::Return {
                    value_type_id: None,
                }),
            )),
            empty_block(control_node(
                &format!("vir.node.{label}.exit"),
                2,
                ControlNodeTag::Exit,
                Vec::new(),
                Some(AbruptCompletion::Normal),
            )),
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: Vec::new(),
    }
}

fn calling_function(
    function_id: &str,
    label: &str,
    signature: &ClosedOperationSignature,
) -> PracticalVirFunction {
    let operation_node_id = format!("vir.node.{label}.call");
    let return_node_id = format!("vir.node.{label}.return");
    PracticalVirFunction {
        id: function_id.into(),
        parameter_values: Vec::new(),
        result_type_ids: Vec::new(),
        blocks: vec![
            empty_block(control_node(
                &format!("vir.node.{label}.entry"),
                0,
                ControlNodeTag::Entry,
                vec![operation_node_id.clone()],
                None,
            )),
            PracticalVirBlock {
                node: control_node(
                    &operation_node_id,
                    1,
                    ControlNodeTag::Operation,
                    vec![return_node_id.clone()],
                    None,
                ),
                phi_values: Vec::new(),
                condition_value_id: None,
                return_value_ids: Vec::new(),
                abrupt_value_id: None,
                handler_exception_value: None,
                invocation: Some(OperationInvocation {
                    operation_id: signature.id.clone(),
                    operands: Vec::new(),
                    result: TypedValueRef {
                        id: format!("vir.value.{label}.call"),
                        type_id: UNIT_TYPE_ID.into(),
                    },
                    ordered_check_ids: Vec::new(),
                    normal_successor_id: return_node_id.clone(),
                    exceptional_successors: Vec::new(),
                }),
                ownership_in: Vec::new(),
                construction_actions: Vec::new(),
                ownership_out: Vec::new(),
            },
            empty_block(control_node(
                &return_node_id,
                2,
                ControlNodeTag::Return,
                Vec::new(),
                Some(AbruptCompletion::Return {
                    value_type_id: None,
                }),
            )),
            empty_block(control_node(
                &format!("vir.node.{label}.exit"),
                3,
                ControlNodeTag::Exit,
                Vec::new(),
                Some(AbruptCompletion::Normal),
            )),
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: Vec::new(),
    }
}

fn invoking_function(
    function_id: &str,
    label: &str,
    signature: &ClosedOperationSignature,
) -> PracticalVirFunction {
    let parameters = signature
        .argument_type_ids
        .iter()
        .enumerate()
        .map(|(ordinal, type_id)| TypedValueRef {
            id: format!("vir.value.{label}.parameter.{ordinal:04}"),
            type_id: type_id.clone(),
        })
        .collect::<Vec<_>>();
    let operation_node_id = format!("vir.node.{label}.operation");
    let return_node_id = format!("vir.node.{label}.return");
    let mut operation = empty_block(control_node(
        &operation_node_id,
        1,
        ControlNodeTag::Operation,
        vec![return_node_id.clone()],
        None,
    ));
    operation.invocation = Some(OperationInvocation {
        operation_id: signature.id.clone(),
        operands: parameters.clone(),
        result: TypedValueRef {
            id: format!("vir.value.{label}.result"),
            type_id: signature.normal_result_type_id.clone(),
        },
        ordered_check_ids: signature
            .ordered_checks
            .iter()
            .map(|check| check.id.clone())
            .collect(),
        normal_successor_id: return_node_id.clone(),
        exceptional_successors: Vec::new(),
    });
    PracticalVirFunction {
        id: function_id.into(),
        parameter_values: parameters,
        result_type_ids: Vec::new(),
        blocks: vec![
            empty_block(control_node(
                &format!("vir.node.{label}.entry"),
                0,
                ControlNodeTag::Entry,
                vec![operation_node_id],
                None,
            )),
            operation,
            empty_block(control_node(
                &return_node_id,
                2,
                ControlNodeTag::Return,
                Vec::new(),
                Some(AbruptCompletion::Return {
                    value_type_id: None,
                }),
            )),
            empty_block(control_node(
                &format!("vir.node.{label}.exit"),
                3,
                ControlNodeTag::Exit,
                Vec::new(),
                Some(AbruptCompletion::Normal),
            )),
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: Vec::new(),
    }
}

fn construction_merge_function(fixture: &Fixture, label: &str) -> PracticalVirFunction {
    let instance_id = fixture
        .closed
        .entries()
        .iter()
        .find(|entry| {
            entry.get("template_id").and_then(Value::as_str)
                == Some("mpk.csharp.semantic.sequence_construction.v1")
        })
        .and_then(|entry| entry.get("instance_id"))
        .and_then(Value::as_str)
        .expect("sequence-construction instance")
        .to_owned();
    let construction_id = format!("construction.{label}");
    let owner_id = format!("owner.{label}");
    let value_parameter = TypedValueRef {
        id: format!("vir.value.{label}.element"),
        type_id: I32_TYPE_ID.into(),
    };
    let condition_parameter = TypedValueRef {
        id: format!("vir.value.{label}.condition"),
        type_id: BOOL_TYPE_ID.into(),
    };
    let allocated = SequenceConstructionState::allocate(
        &fixture.closed,
        &construction_id,
        &instance_id,
        &owner_id,
        2,
        false,
        2,
    )
    .expect("allocated construction state");
    let left = allocated
        .apply(
            &fixture.closed,
            &ModelConstructionAction::Fill {
                actor_id: owner_id.clone(),
                index: 0,
                value_type_id: I32_TYPE_ID.into(),
            },
        )
        .expect("left branch fill")
        .state;
    let right = allocated
        .apply(
            &fixture.closed,
            &ModelConstructionAction::Fill {
                actor_id: owner_id.clone(),
                index: 1,
                value_type_id: I32_TYPE_ID.into(),
            },
        )
        .expect("right branch fill")
        .state;
    let merged = SequenceConstructionState::merge(&fixture.closed, &left, &right)
        .expect("compatible branch states merge");
    assert!(merged.initialized_indices.is_empty());
    let discarded = merged
        .apply(
            &fixture.closed,
            &ModelConstructionAction::Discard {
                actor_id: owner_id.clone(),
            },
        )
        .expect("merged construction discard")
        .state;

    let entry_id = format!("vir.node.{label}.entry");
    let allocate_id = format!("vir.node.{label}.allocate");
    let branch_id = format!("vir.node.{label}.branch");
    let left_id = format!("vir.node.{label}.left");
    let right_id = format!("vir.node.{label}.right");
    let merge_id = format!("vir.node.{label}.merge");
    let return_id = format!("vir.node.{label}.return");
    let exit_id = format!("vir.node.{label}.exit");

    let mut allocate = empty_block(control_node(
        &allocate_id,
        1,
        ControlNodeTag::Operation,
        vec![branch_id.clone()],
        None,
    ));
    allocate.construction_actions = vec![PracticalConstructionAction::Allocate {
        construction_id: construction_id.clone(),
        instance_id,
        owner_id: owner_id.clone(),
        length: 2,
        default_eligible: false,
        publication_length_maximum: 2,
    }];
    allocate.ownership_out = vec![allocated.clone()];

    let mut branch = empty_block(control_node(
        &branch_id,
        2,
        ControlNodeTag::Branch,
        vec![left_id.clone(), right_id.clone()],
        None,
    ));
    branch.node.condition_type_id = Some(BOOL_TYPE_ID.into());
    branch.condition_value_id = Some(condition_parameter.id.clone());
    branch.ownership_in = vec![allocated.clone()];
    branch.ownership_out = vec![allocated.clone()];

    let mut left_block = empty_block(control_node(
        &left_id,
        3,
        ControlNodeTag::Operation,
        vec![merge_id.clone()],
        None,
    ));
    left_block.ownership_in = vec![allocated.clone()];
    left_block.construction_actions = vec![PracticalConstructionAction::Fill {
        construction_id: construction_id.clone(),
        actor_id: owner_id.clone(),
        index: 0,
        value: value_parameter.clone(),
    }];
    left_block.ownership_out = vec![left];

    let mut right_block = empty_block(control_node(
        &right_id,
        4,
        ControlNodeTag::Operation,
        vec![merge_id.clone()],
        None,
    ));
    right_block.ownership_in = vec![allocated];
    right_block.construction_actions = vec![PracticalConstructionAction::Fill {
        construction_id: construction_id.clone(),
        actor_id: owner_id.clone(),
        index: 1,
        value: value_parameter.clone(),
    }];
    right_block.ownership_out = vec![right];

    let mut merge = empty_block(control_node(
        &merge_id,
        5,
        ControlNodeTag::Operation,
        vec![return_id.clone()],
        None,
    ));
    merge.ownership_in = vec![merged];
    merge.construction_actions = vec![PracticalConstructionAction::Discard {
        construction_id,
        actor_id: owner_id,
    }];
    merge.ownership_out = vec![discarded.clone()];

    let mut returned = empty_block(control_node(
        &return_id,
        6,
        ControlNodeTag::Return,
        Vec::new(),
        Some(AbruptCompletion::Return {
            value_type_id: None,
        }),
    ));
    returned.ownership_in = vec![discarded.clone()];
    returned.ownership_out = vec![discarded];

    PracticalVirFunction {
        id: fixture.root_id.clone(),
        parameter_values: vec![value_parameter, condition_parameter],
        result_type_ids: Vec::new(),
        blocks: vec![
            empty_block(control_node(
                &entry_id,
                0,
                ControlNodeTag::Entry,
                vec![allocate_id],
                None,
            )),
            allocate,
            branch,
            left_block,
            right_block,
            merge,
            returned,
            empty_block(control_node(
                &exit_id,
                7,
                ControlNodeTag::Exit,
                Vec::new(),
                Some(AbruptCompletion::Normal),
            )),
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: Vec::new(),
    }
}

fn bypassed_result_function(
    function_id: &str,
    signature: &ClosedOperationSignature,
) -> PracticalVirFunction {
    let parameter = TypedValueRef {
        id: "vir.value.bypassed_result.parameter".into(),
        type_id: BOOL_TYPE_ID.into(),
    };
    let result = TypedValueRef {
        id: "vir.value.bypassed_result.call".into(),
        type_id: BOOL_TYPE_ID.into(),
    };
    let mut operation = empty_block(control_node(
        "vir.node.bypassed_result.call",
        2,
        ControlNodeTag::Operation,
        vec!["vir.node.bypassed_result.merge".into()],
        None,
    ));
    operation.invocation = Some(OperationInvocation {
        operation_id: signature.id.clone(),
        operands: vec![parameter.clone()],
        result: result.clone(),
        ordered_check_ids: Vec::new(),
        normal_successor_id: "vir.node.bypassed_result.merge".into(),
        exceptional_successors: Vec::new(),
    });
    let mut branch = empty_block(control_node(
        "vir.node.bypassed_result.merge",
        4,
        ControlNodeTag::Branch,
        vec![
            "vir.node.bypassed_result.return_true".into(),
            "vir.node.bypassed_result.return_false".into(),
        ],
        None,
    ));
    branch.node.condition_type_id = Some(BOOL_TYPE_ID.into());
    branch.condition_value_id = Some(result.id);
    PracticalVirFunction {
        id: function_id.into(),
        parameter_values: vec![parameter.clone()],
        result_type_ids: Vec::new(),
        blocks: vec![
            empty_block(control_node(
                "vir.node.bypassed_result.entry",
                0,
                ControlNodeTag::Entry,
                vec!["vir.node.bypassed_result.choose".into()],
                None,
            )),
            {
                let mut choose = empty_block(control_node(
                    "vir.node.bypassed_result.choose",
                    1,
                    ControlNodeTag::Branch,
                    vec![
                        "vir.node.bypassed_result.call".into(),
                        "vir.node.bypassed_result.skip".into(),
                    ],
                    None,
                ));
                choose.node.condition_type_id = Some(BOOL_TYPE_ID.into());
                choose.condition_value_id = Some(parameter.id.clone());
                choose
            },
            operation,
            empty_block(control_node(
                "vir.node.bypassed_result.skip",
                3,
                ControlNodeTag::Jump,
                vec!["vir.node.bypassed_result.merge".into()],
                None,
            )),
            branch,
            empty_block(control_node(
                "vir.node.bypassed_result.return_true",
                5,
                ControlNodeTag::Return,
                Vec::new(),
                Some(AbruptCompletion::Return {
                    value_type_id: None,
                }),
            )),
            empty_block(control_node(
                "vir.node.bypassed_result.return_false",
                6,
                ControlNodeTag::Return,
                Vec::new(),
                Some(AbruptCompletion::Return {
                    value_type_id: None,
                }),
            )),
            empty_block(control_node(
                "vir.node.bypassed_result.exit",
                7,
                ControlNodeTag::Exit,
                Vec::new(),
                Some(AbruptCompletion::Normal),
            )),
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: Vec::new(),
    }
}

fn bool_identity_function(function_id: &str) -> PracticalVirFunction {
    let parameter = TypedValueRef {
        id: "vir.value.bool_identity.parameter".into(),
        type_id: BOOL_TYPE_ID.into(),
    };
    let mut function = minimal_function(function_id, "bool_identity");
    function.parameter_values = vec![parameter.clone()];
    function.result_type_ids = vec![BOOL_TYPE_ID.into()];
    function.blocks[1].node.abrupt = Some(AbruptCompletion::Return {
        value_type_id: Some(BOOL_TYPE_ID.into()),
    });
    function.blocks[1].return_value_ids = vec![parameter.id];
    function
}

fn source_call_signature(id: &str) -> ClosedOperationSignature {
    ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::SourceCall,
        argument_type_ids: Vec::new(),
        normal_result_type_id: UNIT_TYPE_ID.into(),
        ordered_checks: Vec::new(),
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

fn control_node(
    id: &str,
    ordinal: u32,
    tag: ControlNodeTag,
    normal_successor_ids: Vec<String>,
    abrupt: Option<AbruptCompletion>,
) -> ControlNode {
    ControlNode {
        id: id.into(),
        ordinal,
        tag,
        condition_type_id: None,
        normal_successor_ids,
        exceptional_successors: Vec::new(),
        abrupt,
        loop_id: None,
        region_stack: Vec::new(),
    }
}

fn invalid_construction_state() -> SequenceConstructionState {
    SequenceConstructionState {
        construction_id: "construction.invalid".into(),
        instance_id: "foundation.instance.missing".into(),
        element_type_id: I32_TYPE_ID.into(),
        published_type_id: I32_TYPE_ID.into(),
        owner_id: "owner.invalid".into(),
        version: 0,
        length: 0,
        publication_length_maximum: 0,
        initialized_indices: BTreeSet::new(),
        borrower_id: None,
        status: ConstructionStatus::Discarded,
    }
}

fn assert_error<T>(
    result: Result<T, mpk_vc::csharp_practical_vir_validation::PracticalVirImportError>,
    phase: PracticalVirImportPhase,
    code: PracticalVirImportErrorCode,
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

fn replace_first(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("replacement source");
    bytes[start..start + from.len()].copy_from_slice(to);
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
        result_type_id: Some(I32_TYPE_ID.into()),
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
