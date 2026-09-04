use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, canonical_monomorphic_value_transport,
    csharp_practical_closed_instance_id, csharp_practical_declaration_id,
    csharp_practical_stored_member_id, derive_closed_exception_universe, derive_closed_instances,
    import_monomorphic_value, registered_foundation_definitions_transport,
    registered_foundation_descriptor_transport, validate_binding_operation_commutation,
    validate_closed_instance_set, validate_closed_operation_signature, validate_closed_root_set,
    validate_explicit_control_graph, validate_explicit_exception_value,
    validate_finally_completion, validate_foundation_structural_limit, validate_monomorphic_value,
    validate_operation_invocation, validate_practical_foundation_limit,
    validate_registered_foundation_bundle, AbruptCompletion, AbruptCompletionTag,
    BindingOperationCommutation, BindingTypeProjection, BoundaryArm, CatchHandler,
    CheckCommutation, ClosedExceptionUniverse, ClosedInstanceSet, ClosedOperationSignature,
    ClosedOperationTag, ConstructionActionTag, ConstructionStatus, ControlNode, ControlNodeTag,
    ExceptionFilterRule, ExceptionHandlerRegion, ExceptionUnwindPlan, ExceptionalSuccessor,
    ExplicitControlGraph, FinallyCompletionRule, FoundationErrorCode, FoundationLimit,
    FoundationValidationError, LoopRegion, MonomorphicMapEntry, MonomorphicValue,
    NamedMonomorphicValue, OperationInvocation, OptionArm, ParseErrorArm, PatternArm,
    PatternDecision, PatternPropertyAccess, PatternTag, PracticalVirErrorCode, RequiredCheck,
    RequiredCheckTag, SequenceConstructionAction, SequenceConstructionState,
    SourceExceptionDefinition, TypedValueRef, ValidatedClosedRootSet, ValidatedFoundationBundle,
    CSHARP_PRACTICAL_OPERATIONS_SCHEMA, CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
    SEQUENCE_CONSTRUCTION_CAPACITY_MAX,
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const FOUNDATION_VECTORS: &str = "develop/specs/vectors/csharp-practical-foundation-v1.json";
const PROFILE_VECTORS: &str = "develop/specs/vectors/csharp-practical-profile-v1.json";
const WORK_ITEM: &str = "CSHARP-03-T02-W02";
const OWNER: &str = "crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W02";
const W03_WORK_ITEM: &str = "CSHARP-03-T02-W03";
const W03_OWNER: &str = "crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W03";

#[test]
fn csharp_03_t02_w02_executes_all_descriptor_vectors_through_registered_validation() {
    let package = read_json(FOUNDATION_VECTORS);
    let vectors = owned_vectors(&package, "descriptor");
    assert_eq!(vectors.len(), 19, "descriptor owner-vector count drift");
    let bundle = registered_bundle();
    assert_eq!(bundle.template_names().count(), 12);
    assert_eq!(bundle.non_template_definitions().len(), 4);

    let descriptor = read_json_bytes(registered_foundation_descriptor_transport());
    let definitions = read_json_bytes(registered_foundation_definitions_transport());
    let mut actual = BTreeMap::new();
    actual.insert("descriptor.content".to_owned(), json!({"accept": true}));
    for key in descriptor
        .as_object()
        .expect("descriptor object")
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        let mut changed = descriptor.clone();
        changed
            .as_object_mut()
            .expect("descriptor object")
            .remove(&key);
        actual.insert(
            format!("descriptor.missing_{key}"),
            rejection(validate_registered_foundation_bundle(
                &canonical_transport(&changed),
                registered_foundation_definitions_transport(),
            )),
        );
    }
    let mut changed_definitions = definitions;
    changed_definitions["ordinary_core"]["zero"] = Value::String("tampered".to_owned());
    actual.insert(
        "descriptor.member_body_mutation".to_owned(),
        rejection(validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            &canonical_transport(&changed_definitions),
        )),
    );
    for (id, bytes) in [
        (
            "descriptor.duplicate_json_key",
            b"{\"id\":0,\"id\":1}\n".as_slice(),
        ),
        (
            "descriptor.floating_json",
            b"{\"version\":1.0}\n".as_slice(),
        ),
        (
            "descriptor.nonfinite_json",
            b"{\"version\":NaN}\n".as_slice(),
        ),
    ] {
        actual.insert(
            id.to_owned(),
            rejection(validate_registered_foundation_bundle(
                bytes,
                registered_foundation_definitions_transport(),
            )),
        );
    }

    assert_vector_results(&vectors, &actual);
}

#[test]
fn csharp_03_t02_w02_executes_all_specialization_vectors_through_the_engine() {
    let package = read_json(FOUNDATION_VECTORS);
    let vectors = owned_vectors(&package, "specialization");
    assert_eq!(vectors.len(), 57, "specialization owner-vector count drift");
    let by_id = vectors_by_id(&vectors);
    let bundle = registered_bundle();
    let all_inputs = &by_id["specialization.all_templates"]["inputs"];
    let roots = all_inputs["roots"].clone();
    let source_types = all_inputs["source_types"].clone();
    for (index, root) in roots.as_array().expect("root array").iter().enumerate() {
        try_root_set(&bundle, &Value::Array(vec![root.clone()]), &source_types)
            .unwrap_or_else(|error| panic!("all-template root {index} failed: {error:?}"));
    }
    let validated_roots = root_set(&bundle, &roots, &source_types);
    let closed = derive_closed_instances(&bundle, &validated_roots).expect("all templates close");
    let mut actual = BTreeMap::new();
    actual.insert(
        "specialization.all_templates".to_owned(),
        closed.value().clone(),
    );

    let mut reversed = roots.as_array().expect("root array").clone();
    reversed.reverse();
    let reversed_roots = root_set(&bundle, &Value::Array(reversed), &source_types);
    let reversed_closed = derive_closed_instances(&bundle, &reversed_roots)
        .expect("root order does not affect specialization");
    actual.insert(
        "specialization.root_permutation".to_owned(),
        json!({"value": reversed_closed.value()}),
    );
    actual.insert(
        "specialization.registry_cardinality".to_owned(),
        json!({"value": bundle.template_names().count()}),
    );
    actual.insert(
        "specialization.non_template_cardinality".to_owned(),
        json!({"value": bundle.non_template_definitions().len()}),
    );

    for key in closed
        .value()
        .as_object()
        .expect("closed set object")
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        let mut changed = closed.value().clone();
        changed
            .as_object_mut()
            .expect("closed set object")
            .remove(&key);
        actual.insert(
            format!("specialization.missing_{key}"),
            rejection(validate_closed_instance_set(
                &bundle,
                &validated_roots,
                &canonical_transport(&changed),
            )),
        );
    }
    for (name, mutation) in [
        ("omit_dependency", mutate_omit_dependency as fn(&mut Value)),
        ("reorder", mutate_reorder),
        ("duplicate", mutate_duplicate),
        ("provenance", mutate_provenance),
        ("residual_generic", mutate_residual_generic),
        ("operation_body", mutate_operation_body),
        ("counter", mutate_counter),
    ] {
        let mut changed = closed.value().clone();
        mutation(&mut changed);
        actual.insert(
            format!("specialization.{name}"),
            rejection(validate_closed_instance_set(
                &bundle,
                &validated_roots,
                &canonical_transport(&changed),
            )),
        );
    }

    let first_root = roots.as_array().expect("root array")[0].clone();
    actual.insert(
        "specialization.duplicate_root".to_owned(),
        rejection(try_root_set(
            &bundle,
            &json!([first_root.clone(), first_root]),
            &json!({}),
        )),
    );
    for id in [
        "specialization.user_generic",
        "specialization.unknown_template",
        "specialization.wrong_arity",
        "specialization.nested_option",
        "specialization.float_key",
        "specialization.linear_payload",
        "specialization.money_integer_currency",
    ] {
        let ty = by_id[id]["inputs"].clone();
        actual.insert(
            id.to_owned(),
            rejection(try_root_set(
                &bundle,
                &json!([{
                    "origin": "semantic_binding",
                    "provenance_id": "root.invalid",
                    "type": ty,
                }]),
                &json!({}),
            )),
        );
    }
    let lookup_nullable = by_id["specialization.lookup_nullable"]["expected"]["value"].clone();
    root_set(
        &bundle,
        &json!([{
            "origin": "semantic_binding",
            "provenance_id": "root.lookup_nullable",
            "type": lookup_nullable,
        }]),
        &json!({}),
    );
    actual.insert(
        "specialization.lookup_nullable".to_owned(),
        json!({"value": lookup_nullable}),
    );

    let source_inputs = &by_id["specialization.source_member_closure"]["inputs"];
    let source_roots = root_set(&bundle, &source_inputs["roots"], &source_inputs["sources"]);
    let source_closed =
        derive_closed_instances(&bundle, &source_roots).expect("source member instance closure");
    let mut template_ids = source_closed
        .entries()
        .iter()
        .map(|entry| {
            entry["template_id"]
                .as_str()
                .expect("template ID")
                .to_owned()
        })
        .collect::<Vec<_>>();
    template_ids.sort();
    actual.insert(
        "specialization.source_member_closure".to_owned(),
        json!({"value": template_ids}),
    );

    let cycle = source_fixture(
        "Cycle",
        "readonly_struct",
        &[("next", json!({"kind": "source", "id": source_id("Cycle")}))],
        &[],
    );
    let cycle_id = cycle["id"].as_str().expect("cycle source ID");
    actual.insert(
        "specialization.source_cycle".to_owned(),
        rejection(try_root_set(&bundle, &json!([]), &json!({cycle_id: cycle}))),
    );

    let mut wrong_origin = roots.as_array().expect("root array")[0].clone();
    wrong_origin["origin"] = Value::String("source_construction".to_owned());
    actual.insert(
        "specialization.invalid_root_derivation".to_owned(),
        rejection(try_root_set(
            &bundle,
            &Value::Array(vec![wrong_origin]),
            &json!({}),
        )),
    );

    for count in [15_u64, 16, 17] {
        let ty = nested_sequence(count);
        let result = try_root_set(
            &bundle,
            &json!([{
                "origin": "semantic_binding",
                "provenance_id": format!("root.depth.{count}"),
                "type": ty,
            }]),
            &json!({}),
        );
        actual.insert(
            format!("specialization.depth_{count}"),
            if count <= 16 {
                result.expect("depth at limit");
                json!({"value": ty})
            } else {
                rejection(result)
            },
        );
    }

    for count in [255_usize, 256, 257] {
        let (large_roots, large_sources) = large_instance_inputs(count);
        let result = try_root_set(&bundle, &large_roots, &large_sources)
            .and_then(|roots| derive_closed_instances(&bundle, &roots));
        actual.insert(
            format!("specialization.instances_{count}"),
            if count <= 256 {
                json!({"value": result.expect("instance count at limit").entries().len()})
            } else {
                rejection(result)
            },
        );
    }

    for (name, limit) in [
        ("binding_count", FoundationLimit::BindingCount),
        (
            "closed_instance_count",
            FoundationLimit::ClosedInstanceCount,
        ),
        (
            "closed_instance_depth",
            FoundationLimit::ClosedInstanceDepth,
        ),
        (
            "expanded_declarations",
            FoundationLimit::ExpandedDeclarations,
        ),
        ("expanded_operations", FoundationLimit::ExpandedOperations),
        (
            "expanded_recipe_nodes",
            FoundationLimit::ExpandedRecipeNodes,
        ),
        (
            "projection_obligations_per_binding",
            FoundationLimit::ProjectionObligationsPerBinding,
        ),
    ] {
        let maximum = limit.inclusive_maximum();
        for value in [maximum - 1, maximum] {
            validate_foundation_structural_limit(limit, value).expect("inclusive limit");
            actual.insert(
                format!("specialization.counter_{name}_{value}"),
                json!({"accept": true}),
            );
        }
        actual.insert(
            format!("specialization.counter_{name}_over"),
            rejection(validate_foundation_structural_limit(limit, maximum + 1)),
        );
    }

    assert_vector_results(&vectors, &actual);

    let single_root = root_set(
        &bundle,
        &Value::Array(vec![roots.as_array().expect("root array")[0].clone()]),
        &json!({}),
    );
    assert_value_reject(
        validate_closed_instance_set(&bundle, &single_root, &canonical_transport(closed.value())),
        FoundationErrorCode::ClosedSetRecomputation,
    );
}

#[test]
fn csharp_03_t02_w02_executes_all_shared_limit_vectors() {
    let package = read_json(PROFILE_VECTORS);
    let vectors = package["vectors"]
        .as_array()
        .expect("profile vectors")
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
        .collect::<Vec<_>>();
    assert_eq!(vectors.len(), 12, "shared W02 limit-vector count drift");
    for vector in vectors {
        assert_eq!(vector["production_test_owner"], OWNER);
        let inputs = &vector["inputs"];
        let counter = inputs["counter"].as_str().expect("counter ID");
        let maximum = inputs["inclusive_maximum"].as_u64().expect("maximum");
        let value = inputs["value"].as_u64().expect("counter value");
        let limit = FoundationLimit::from_id(counter).expect("registered W02 counter");
        assert_eq!(limit.inclusive_maximum(), maximum);
        let actual = match validate_practical_foundation_limit(counter, value) {
            Ok(()) => json!({"accept": true}),
            Err(error) => json!({"reject": error.code().as_str()}),
        };
        assert_eq!(actual, vector["expected"], "{}", vector["id"]);
    }
}

#[test]
fn csharp_03_t02_w02_round_trips_every_monomorphic_value_family() {
    let package = read_json(FOUNDATION_VECTORS);
    let vectors = owned_vectors(&package, "specialization");
    let by_id = vectors_by_id(&vectors);
    let bundle = registered_bundle();
    let all_inputs = &by_id["specialization.all_templates"]["inputs"];
    let mut roots = all_inputs["roots"].as_array().expect("root array").clone();

    for (provenance_id, template, arguments) in [
        (
            "root.value.sequence_string",
            "bounded_sequence",
            vec![primitive_type("string")],
        ),
        (
            "root.value.ordered_set_decimal",
            "ordered_set",
            vec![primitive_type("decimal")],
        ),
    ] {
        roots.push(json!({
            "origin": "semantic_binding",
            "provenance_id": provenance_id,
            "type": instance_type(template, arguments),
        }));
    }

    let product = source_fixture(
        "LineValue",
        "readonly_struct",
        &[
            ("quantity", primitive_type("i32")),
            ("label", primitive_type("string")),
        ],
        &[],
    );
    let enumeration = source_fixture("CurrencyCode", "enum", &[], &[0, 2]);
    let exception = source_fixture(
        "BusinessException",
        "sealed_class",
        &[("code", primitive_type("i32"))],
        &[],
    );
    let product_id = product["id"].as_str().expect("product ID").to_owned();
    let enum_id = enumeration["id"].as_str().expect("enum ID").to_owned();
    let exception_id = exception["id"].as_str().expect("exception ID").to_owned();
    for (index, source_id) in [&product_id, &enum_id, &exception_id]
        .into_iter()
        .enumerate()
    {
        roots.push(json!({
            "origin": "contract",
            "provenance_id": format!("root.value.source.{index}"),
            "type": {"kind": "source", "id": source_id},
        }));
    }
    let source_types = json!({
        product_id.clone(): product,
        enum_id.clone(): enumeration,
        exception_id.clone(): exception,
    });
    let roots = root_set(&bundle, &Value::Array(roots), &source_types);
    let closed = derive_closed_instances(&bundle, &roots).expect("concrete-value closure");

    let sequence_i32 = closed_instance_id(&bundle, "bounded_sequence", &[primitive_type("i32")]);
    let sequence_string =
        closed_instance_id(&bundle, "bounded_sequence", &[primitive_type("string")]);
    let ordered_entry = closed_instance_id(
        &bundle,
        "ordered_entry",
        &[primitive_type("i32"), primitive_type("i32")],
    );
    let ordered_map = closed_instance_id(
        &bundle,
        "ordered_map",
        &[primitive_type("i32"), primitive_type("i32")],
    );
    let ordered_set = closed_instance_id(&bundle, "ordered_set", &[primitive_type("i32")]);
    let decimal_set = closed_instance_id(&bundle, "ordered_set", &[primitive_type("decimal")]);
    let option = closed_instance_id(&bundle, "option", &[primitive_type("i32")]);
    let lookup = closed_instance_id(&bundle, "lookup", &[primitive_type("i32")]);
    let result = closed_instance_id(
        &bundle,
        "result",
        &[primitive_type("i32"), primitive_type("i32")],
    );
    let validation = closed_instance_id(
        &bundle,
        "validation",
        &[primitive_type("i32"), primitive_type("i32")],
    );
    let boundary = closed_instance_id(&bundle, "boundary_field", &[primitive_type("i32")]);
    let money = closed_instance_id(&bundle, "money", &[primitive_type("string")]);
    let transition = closed_instance_id(
        &bundle,
        "transition",
        &[
            primitive_type("i32"),
            primitive_type("i32"),
            primitive_type("i32"),
        ],
    );

    let values = vec![
        ("unit", unit_value()),
        (
            "bool",
            MonomorphicValue::Bool {
                type_id: value_type_id("bool"),
                value: true,
            },
        ),
        ("signed", signed_i32("-17")),
        (
            "unsigned",
            MonomorphicValue::Unsigned {
                type_id: value_type_id("u64"),
                value: u64::MAX.to_string(),
            },
        ),
        (
            "char",
            MonomorphicValue::Char {
                type_id: value_type_id("char"),
                utf16: 0xd800,
            },
        ),
        ("string", string_value(&[0x41, 0xd800, 0x42])),
        (
            "f32_bits",
            MonomorphicValue::F32Bits {
                type_id: value_type_id("f32"),
                bits: "7fc00001".to_owned(),
            },
        ),
        (
            "f64_bits",
            MonomorphicValue::F64Bits {
                type_id: value_type_id("f64"),
                bits: "8000000000000000".to_owned(),
            },
        ),
        ("decimal_bits", decimal_value(false, 2, "12345")),
        (
            "source_enum",
            MonomorphicValue::Enum {
                type_id: enum_id.clone(),
                underlying: "i32".to_owned(),
                carrier: "2".to_owned(),
            },
        ),
        (
            "day_of_week",
            MonomorphicValue::Enum {
                type_id: value_type_id("day_of_week"),
                underlying: "i32".to_owned(),
                carrier: "6".to_owned(),
            },
        ),
        (
            "immutable_product",
            MonomorphicValue::Product {
                type_id: product_id.clone(),
                fields: vec![
                    named_value("quantity", signed_i32("3")),
                    named_value("label", string_value(&[0x6f, 0x6b])),
                ],
            },
        ),
        (
            "array",
            MonomorphicValue::Array {
                type_id: sequence_i32.clone(),
                elements: vec![signed_i32("1"), signed_i32("2")],
            },
        ),
        (
            "sequence",
            MonomorphicValue::Sequence {
                type_id: sequence_i32.clone(),
                elements: vec![signed_i32("1"), signed_i32("2")],
            },
        ),
        (
            "ordered_entry",
            MonomorphicValue::OrderedEntry {
                type_id: ordered_entry.clone(),
                key: Box::new(signed_i32("2")),
                value: Box::new(signed_i32("20")),
            },
        ),
        (
            "ordered_map",
            ordered_map_value(&ordered_map, &[(2, 20), (10, 100)]),
        ),
        (
            "ordered_set",
            MonomorphicValue::OrderedSet {
                type_id: ordered_set.clone(),
                elements: vec![signed_i32("-2"), signed_i32("10")],
            },
        ),
        (
            "decimal_ordered_set",
            MonomorphicValue::OrderedSet {
                type_id: decimal_set.clone(),
                elements: vec![decimal_value(false, 1, "2"), decimal_value(false, 0, "1")],
            },
        ),
        (
            "option_none",
            MonomorphicValue::Option {
                type_id: option.clone(),
                arm: OptionArm::None,
                value: None,
            },
        ),
        (
            "option_some",
            MonomorphicValue::Option {
                type_id: option.clone(),
                arm: OptionArm::Some,
                value: Some(Box::new(signed_i32("7"))),
            },
        ),
        (
            "lookup_missing",
            tagged_value(&lookup, "missing_key", vec![]),
        ),
        (
            "lookup_found",
            tagged_value(&lookup, "found", vec![signed_i32("7")]),
        ),
        (
            "result_ok",
            tagged_value(&result, "ok", vec![signed_i32("7")]),
        ),
        (
            "result_error",
            tagged_value(&result, "error", vec![signed_i32("8")]),
        ),
        (
            "validation_valid",
            tagged_value(&validation, "valid", vec![signed_i32("7")]),
        ),
        (
            "validation_invalid",
            tagged_value(
                &validation,
                "invalid",
                vec![MonomorphicValue::Sequence {
                    type_id: sequence_i32.clone(),
                    elements: vec![signed_i32("8")],
                }],
            ),
        ),
        (
            "boundary_missing",
            boundary_value(&boundary, BoundaryArm::Missing, None),
        ),
        (
            "boundary_null",
            boundary_value(&boundary, BoundaryArm::Null, None),
        ),
        (
            "boundary_value",
            boundary_value(&boundary, BoundaryArm::Value, Some(signed_i32("9"))),
        ),
        (
            "date",
            MonomorphicValue::Date {
                type_id: value_type_id("date"),
                day_number: 3_652_058,
            },
        ),
        (
            "time",
            MonomorphicValue::Time {
                type_id: value_type_id("time"),
                ticks: "863999999999".to_owned(),
            },
        ),
        (
            "duration",
            MonomorphicValue::Duration {
                type_id: value_type_id("duration"),
                ticks: i64::MIN.to_string(),
            },
        ),
        (
            "instant",
            MonomorphicValue::Instant {
                type_id: value_type_id("instant"),
                milliseconds: i64::MAX.to_string(),
            },
        ),
        (
            "guid",
            MonomorphicValue::Guid {
                type_id: value_type_id("guid"),
                n: "00112233445566778899aabbccddeeff".to_owned(),
            },
        ),
        (
            "money",
            MonomorphicValue::Money {
                type_id: money,
                amount: Box::new(decimal_value(true, 2, "12345")),
                currency: Box::new(string_value(&[0x4a, 0x50, 0x59])),
            },
        ),
        (
            "transition",
            MonomorphicValue::Transition {
                type_id: transition,
                state: Box::new(signed_i32("1")),
                events: vec![signed_i32("2"), signed_i32("3")],
                response: Box::new(signed_i32("4")),
            },
        ),
        (
            "closed_builtin_exception",
            MonomorphicValue::ClosedException {
                type_id: value_type_id("exception"),
                tag: 8,
                source_type_id: None,
                payload: None,
            },
        ),
        (
            "closed_source_exception",
            MonomorphicValue::ClosedException {
                type_id: value_type_id("exception"),
                tag: 9,
                source_type_id: Some(exception_id.clone()),
                payload: Some(Box::new(MonomorphicValue::Product {
                    type_id: exception_id,
                    fields: vec![named_value("code", signed_i32("42"))],
                })),
            },
        ),
    ];

    for (arm_name, arm) in [
        ("input_bound", ParseErrorArm::InputBound),
        ("syntax", ParseErrorArm::Syntax),
        ("noncanonical", ParseErrorArm::Noncanonical),
        ("scale_precision", ParseErrorArm::ScalePrecision),
        ("range", ParseErrorArm::Range),
    ] {
        round_trip_value(
            &bundle,
            &roots,
            &closed,
            &format!("parse_error.{arm_name}"),
            &MonomorphicValue::ParseError {
                type_id: value_type_id("parse_error"),
                arm,
            },
        );
    }
    for (name, value) in &values {
        round_trip_value(&bundle, &roots, &closed, name, value);
    }

    let invalid_nested = MonomorphicValue::OrderedEntry {
        type_id: ordered_entry,
        key: Box::new(signed_i32("1")),
        value: Box::new(signed_i32("01")),
    };
    assert_value_reject(
        validate_monomorphic_value(&bundle, &roots, &closed, &invalid_nested),
        FoundationErrorCode::ConcreteValueInvariant,
    );
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &ordered_map_value(&ordered_map, &[(10, 100), (2, 20)]),
        ),
        FoundationErrorCode::ConcreteValueInvariant,
    );
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &MonomorphicValue::OrderedSet {
                type_id: decimal_set.clone(),
                elements: vec![decimal_value(false, 0, "1"), decimal_value(false, 1, "2")],
            },
        ),
        FoundationErrorCode::ConcreteValueInvariant,
    );
    for equivalent_decimal_values in [
        vec![
            decimal_value(false, 1, "10"),
            decimal_value(false, 2, "100"),
        ],
        vec![decimal_value(true, 0, "0"), decimal_value(false, 0, "0")],
    ] {
        assert_value_reject(
            validate_monomorphic_value(
                &bundle,
                &roots,
                &closed,
                &MonomorphicValue::OrderedSet {
                    type_id: decimal_set.clone(),
                    elements: equivalent_decimal_values,
                },
            ),
            FoundationErrorCode::ConcreteValueInvariant,
        );
    }
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &MonomorphicValue::Product {
                type_id: ordered_map.clone(),
                fields: Vec::new(),
            },
        ),
        FoundationErrorCode::ConcreteValueType,
    );
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &MonomorphicValue::String {
                type_id: value_type_id("string"),
                utf16: vec![0; 16_385],
            },
        ),
        FoundationErrorCode::ConcreteValueBound,
    );
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &MonomorphicValue::Sequence {
                type_id: sequence_i32,
                elements: vec![signed_i32("0"); 4_097],
            },
        ),
        FoundationErrorCode::ConcreteValueBound,
    );
    assert_value_reject(
        validate_monomorphic_value(
            &bundle,
            &roots,
            &closed,
            &MonomorphicValue::Sequence {
                type_id: sequence_string,
                elements: vec![string_value(&[0; 16]); 4_096],
            },
        ),
        FoundationErrorCode::TotalValueCells,
    );

    let generic_transport = canonical_transport(&json!({"kind": "parameter", "index": 0}));
    assert_value_reject(
        import_monomorphic_value(&bundle, &roots, &closed, &generic_transport),
        FoundationErrorCode::ConcreteValueShape,
    );
    let mut noncanonical =
        canonical_monomorphic_value_transport(&bundle, &roots, &closed, &unit_value())
            .expect("canonical unit");
    noncanonical.pop();
    assert_value_reject(
        import_monomorphic_value(&bundle, &roots, &closed, &noncanonical),
        FoundationErrorCode::CanonicalTransport,
    );
}

#[test]
fn csharp_03_t02_w03_closes_every_operation_control_and_pattern_tag() {
    assert!(W03_OWNER.ends_with(W03_WORK_ITEM));
    assert_eq!(
        CSHARP_PRACTICAL_OPERATIONS_SCHEMA,
        "mpk.csharp.operations.v1"
    );
    assert_eq!(
        CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
        "mpk.csharp.required_checks.v1"
    );
    for (id, tag) in [
        ("foundation", ClosedOperationTag::Foundation),
        ("field_read", ClosedOperationTag::FieldRead),
        ("value_construct", ClosedOperationTag::ValueConstruct),
        ("source_call", ClosedOperationTag::SourceCall),
        ("binding_project", ClosedOperationTag::BindingProject),
        (
            "binding_reconstruct",
            ClosedOperationTag::BindingReconstruct,
        ),
        ("structural_equal", ClosedOperationTag::StructuralEqual),
        ("canonical_compare", ClosedOperationTag::CanonicalCompare),
        ("boundary_parse", ClosedOperationTag::BoundaryParse),
        ("boundary_format", ClosedOperationTag::BoundaryFormat),
        ("data", ClosedOperationTag::Data),
        (
            "exception_construct",
            ClosedOperationTag::ExceptionConstruct,
        ),
        ("exception_is_type", ClosedOperationTag::ExceptionIsType),
        ("exception_payload", ClosedOperationTag::ExceptionPayload),
    ] {
        assert_eq!(ClosedOperationTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for (id, tag) in [
        ("static_obligation", RequiredCheckTag::StaticObligation),
        ("parse_error", RequiredCheckTag::ParseError),
        ("exception", RequiredCheckTag::Exception),
        ("error_outcome", RequiredCheckTag::ErrorOutcome),
    ] {
        assert_eq!(RequiredCheckTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for (id, tag) in [
        ("allocate", ConstructionActionTag::Allocate),
        ("read", ConstructionActionTag::Read),
        ("fill", ConstructionActionTag::Fill),
        ("rewrite", ConstructionActionTag::Rewrite),
        ("borrow", ConstructionActionTag::Borrow),
        ("end_borrow", ConstructionActionTag::EndBorrow),
        ("transfer", ConstructionActionTag::Transfer),
        ("freeze", ConstructionActionTag::Freeze),
        ("discard", ConstructionActionTag::Discard),
    ] {
        assert_eq!(ConstructionActionTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for (id, tag) in [
        ("normal", AbruptCompletionTag::Normal),
        ("return", AbruptCompletionTag::Return),
        ("break", AbruptCompletionTag::Break),
        ("continue", AbruptCompletionTag::Continue),
        ("throw", AbruptCompletionTag::Throw),
    ] {
        assert_eq!(AbruptCompletionTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for (id, tag) in [
        ("entry", ControlNodeTag::Entry),
        ("operation", ControlNodeTag::Operation),
        ("branch", ControlNodeTag::Branch),
        ("jump", ControlNodeTag::Jump),
        ("loop_header", ControlNodeTag::LoopHeader),
        ("pattern_decision", ControlNodeTag::PatternDecision),
        ("return", ControlNodeTag::Return),
        ("break", ControlNodeTag::Break),
        ("continue", ControlNodeTag::Continue),
        ("throw", ControlNodeTag::Throw),
        ("rethrow", ControlNodeTag::Rethrow),
        ("handler_entry", ControlNodeTag::HandlerEntry),
        ("finally_entry", ControlNodeTag::FinallyEntry),
        ("finally_exit", ControlNodeTag::FinallyExit),
        ("exit", ControlNodeTag::Exit),
    ] {
        assert_eq!(ControlNodeTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for (id, tag) in [
        ("constant", PatternTag::Constant),
        ("discard", PatternTag::Discard),
        ("var", PatternTag::Var),
        ("null", PatternTag::Null),
        ("not_null", PatternTag::NotNull),
        ("relational", PatternTag::Relational),
        ("parenthesized", PatternTag::Parenthesized),
        ("and", PatternTag::And),
        ("or", PatternTag::Or),
        ("not", PatternTag::Not),
        ("declaration_type", PatternTag::DeclarationType),
        ("exact_tag", PatternTag::ExactTag),
        ("property", PatternTag::Property),
        ("list", PatternTag::List),
    ] {
        assert_eq!(PatternTag::from_id(id), Some(tag), "{id}");
        assert_eq!(tag.as_str(), id);
    }
    for forbidden in [
        "iterator",
        "yield",
        "async",
        "await",
        "task",
        "suspension",
        "scheduler",
        "continuation",
        "goto",
    ] {
        assert_eq!(ClosedOperationTag::from_id(forbidden), None, "{forbidden}");
        assert_eq!(ControlNodeTag::from_id(forbidden), None, "{forbidden}");
        assert_eq!(PatternTag::from_id(forbidden), None, "{forbidden}");
        assert_eq!(AbruptCompletionTag::from_id(forbidden), None, "{forbidden}");
        assert_eq!(
            ConstructionActionTag::from_id(forbidden),
            None,
            "{forbidden}"
        );
    }
}

#[test]
fn csharp_03_t02_w03_validates_closed_operations_checks_and_edges() {
    let fixture = w03_fixture();
    let foundation = foundation_signature(
        &fixture.closed,
        &format!("{}.read", fixture.construction_type_id),
        &fixture.error_type_id,
    );
    let parse = ClosedOperationSignature {
        id: "codec.integer.i32.parse".to_owned(),
        tag: ClosedOperationTag::BoundaryParse,
        argument_type_ids: vec![value_type_id("string")],
        normal_result_type_id: fixture.parse_result_type_id.clone(),
        ordered_checks: vec![
            parse_check("parse_error.input_bound"),
            parse_check("parse_error.syntax"),
            parse_check("parse_error.noncanonical"),
            parse_check("parse_error.range"),
        ],
    };
    let format = ClosedOperationSignature {
        id: "codec.integer.i32.format".to_owned(),
        tag: ClosedOperationTag::BoundaryFormat,
        argument_type_ids: vec![value_type_id("i32")],
        normal_result_type_id: value_type_id("string"),
        ordered_checks: vec![static_check("obligation.output_bound")],
    };
    let money_type_id = find_closed_instance(&fixture.closed, "money", &[value_type_id("string")]);
    let option_i32_type_id =
        find_closed_instance(&fixture.closed, "option", &[value_type_id("i32")]);
    let operation_tags = vec![
        foundation.clone(),
        foundation_signature(
            &fixture.closed,
            &format!("{money_type_id}.create"),
            &fixture.error_type_id,
        ),
        ClosedOperationSignature {
            id: "field.read.value".to_owned(),
            tag: ClosedOperationTag::FieldRead,
            argument_type_ids: vec![fixture.source_type_id.clone()],
            normal_result_type_id: value_type_id("i32"),
            ordered_checks: Vec::new(),
        },
        ClosedOperationSignature {
            id: "value.construct.source".to_owned(),
            tag: ClosedOperationTag::ValueConstruct,
            argument_type_ids: vec![value_type_id("i32")],
            normal_result_type_id: fixture.source_type_id.clone(),
            ordered_checks: Vec::new(),
        },
        ClosedOperationSignature {
            id: "mpk.csharp.source.callable.demo".to_owned(),
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![fixture.source_type_id.clone()],
            normal_result_type_id: fixture.source_type_id.clone(),
            ordered_checks: Vec::new(),
        },
        projection_signature(
            "binding.project.demo",
            ClosedOperationTag::BindingProject,
            &fixture.source_type_id,
            &fixture.sequence_type_id,
        ),
        projection_signature(
            "binding.reconstruct.demo",
            ClosedOperationTag::BindingReconstruct,
            &fixture.sequence_type_id,
            &fixture.source_type_id,
        ),
        ClosedOperationSignature {
            id: "structural.equal.i32".to_owned(),
            tag: ClosedOperationTag::StructuralEqual,
            argument_type_ids: vec![value_type_id("i32"), value_type_id("i32")],
            normal_result_type_id: value_type_id("bool"),
            ordered_checks: Vec::new(),
        },
        ClosedOperationSignature {
            id: "canonical.compare.i32".to_owned(),
            tag: ClosedOperationTag::CanonicalCompare,
            argument_type_ids: vec![value_type_id("i32"), value_type_id("i32")],
            normal_result_type_id: value_type_id("i32"),
            ordered_checks: Vec::new(),
        },
        parse.clone(),
        format.clone(),
        ClosedOperationSignature {
            id: "decimal.add".to_owned(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: vec![value_type_id("decimal"), value_type_id("decimal")],
            normal_result_type_id: value_type_id("decimal"),
            ordered_checks: vec![exception_check(
                "exception.overflow",
                "System.OverflowException",
            )],
        },
        ClosedOperationSignature {
            id: "lifted.i32.add".to_owned(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: vec![option_i32_type_id.clone(), option_i32_type_id.clone()],
            normal_result_type_id: option_i32_type_id,
            ordered_checks: vec![exception_check(
                "exception.overflow",
                "System.OverflowException",
            )],
        },
        ClosedOperationSignature {
            id: "mpk.csharp.value.exception.v1.construct".to_owned(),
            tag: ClosedOperationTag::ExceptionConstruct,
            argument_type_ids: vec![fixture.source_exception_type_id.clone()],
            normal_result_type_id: value_type_id("exception"),
            ordered_checks: Vec::new(),
        },
        ClosedOperationSignature {
            id: "mpk.csharp.value.exception.v1.is_type".to_owned(),
            tag: ClosedOperationTag::ExceptionIsType,
            argument_type_ids: vec![value_type_id("exception")],
            normal_result_type_id: value_type_id("bool"),
            ordered_checks: Vec::new(),
        },
        ClosedOperationSignature {
            id: "mpk.csharp.value.exception.v1.payload".to_owned(),
            tag: ClosedOperationTag::ExceptionPayload,
            argument_type_ids: vec![value_type_id("exception")],
            normal_result_type_id: fixture.source_exception_type_id.clone(),
            ordered_checks: vec![exception_check(
                "invalid_operation",
                "System.InvalidOperationException",
            )],
        },
    ];
    assert_eq!(operation_tags.len(), 16);
    for operation in &operation_tags {
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, operation)
            .unwrap_or_else(|error| panic!("{}: {error:?}", operation.tag.as_str()));
    }

    let invocation = invocation_for(&foundation, "block.normal", "block.exception");
    validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &invocation)
        .expect("typed foundation invocation");

    let mut changed = invocation.clone();
    changed.operands.pop();
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::Arity,
    );
    let mut changed = invocation.clone();
    changed.operands[0].type_id = value_type_id("i32");
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::OperandType,
    );
    let mut changed = invocation.clone();
    changed.result.type_id = value_type_id("bool");
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::ResultType,
    );
    let mut changed = invocation.clone();
    changed.ordered_check_ids.reverse();
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::CheckOrder,
    );
    let mut changed = invocation.clone();
    changed.normal_successor_id.clear();
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::NormalSuccessor,
    );
    let mut changed = invocation.clone();
    changed.exceptional_successors.clear();
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::ExceptionalSuccessor,
    );
    let mut changed = invocation;
    changed.exceptional_successors[0].target_id = changed.normal_successor_id.clone();
    assert_vir_reject(
        validate_operation_invocation(&fixture.roots, &fixture.closed, &foundation, &changed),
        PracticalVirErrorCode::ExceptionalSuccessor,
    );

    let mut changed = foundation.clone();
    changed.id.push_str(".unknown");
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &changed),
        PracticalVirErrorCode::UnknownOperation,
    );
    let mut changed = foundation;
    changed.ordered_checks[1].tag = RequiredCheckTag::StaticObligation;
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &changed),
        PracticalVirErrorCode::CheckKind,
    );
    let mut changed = parse;
    changed.ordered_checks.swap(1, 2);
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &changed),
        PracticalVirErrorCode::CheckOrder,
    );
    let mut changed = format;
    changed.ordered_checks[0].id = "unknown".to_owned();
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &changed),
        PracticalVirErrorCode::UnknownCheck,
    );
    let forbidden = ClosedOperationSignature {
        id: "async.task.await".to_owned(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: vec![value_type_id("i32")],
        normal_result_type_id: value_type_id("i32"),
        ordered_checks: Vec::new(),
    };
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &forbidden),
        PracticalVirErrorCode::UnknownOperation,
    );
    let unknown_data = ClosedOperationSignature {
        id: "decimal.made_up".to_owned(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: vec![value_type_id("decimal")],
        normal_result_type_id: value_type_id("decimal"),
        ordered_checks: Vec::new(),
    };
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &unknown_data),
        PracticalVirErrorCode::UnknownOperation,
    );
    let unknown_lifted = ClosedOperationSignature {
        id: "lifted.i32.rotate".to_owned(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: vec![value_type_id("i32")],
        normal_result_type_id: value_type_id("i32"),
        ordered_checks: Vec::new(),
    };
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &unknown_lifted),
        PracticalVirErrorCode::UnknownOperation,
    );
    let mut unknown_codec = ClosedOperationSignature {
        id: "codec.unknown.parse".to_owned(),
        ..operation_tags
            .iter()
            .find(|operation| operation.tag == ClosedOperationTag::BoundaryParse)
            .expect("parse operation")
            .clone()
    };
    unknown_codec.tag = ClosedOperationTag::BoundaryParse;
    assert_vir_reject(
        validate_closed_operation_signature(&fixture.roots, &fixture.closed, &unknown_codec),
        PracticalVirErrorCode::UnknownOperation,
    );
}

#[test]
fn csharp_03_t02_w03_validates_application_binding_commutation() {
    let fixture = w03_fixture();
    let projections = vec![
        BindingTypeProjection {
            id: "projection.sequence".to_owned(),
            binding_id: "binding.sequence".to_owned(),
            source_type_id: fixture.source_type_id.clone(),
            semantic_type_id: fixture.sequence_type_id.clone(),
            project: projection_signature(
                "binding.project.sequence",
                ClosedOperationTag::BindingProject,
                &fixture.source_type_id,
                &fixture.sequence_type_id,
            ),
            reconstruct: projection_signature(
                "binding.reconstruct.sequence",
                ClosedOperationTag::BindingReconstruct,
                &fixture.sequence_type_id,
                &fixture.source_type_id,
            ),
        },
        BindingTypeProjection {
            id: "projection.i32".to_owned(),
            binding_id: "binding.i32.identity".to_owned(),
            source_type_id: value_type_id("i32"),
            semantic_type_id: value_type_id("i32"),
            project: projection_signature(
                "binding.project.i32",
                ClosedOperationTag::BindingProject,
                &value_type_id("i32"),
                &value_type_id("i32"),
            ),
            reconstruct: projection_signature(
                "binding.reconstruct.i32",
                ClosedOperationTag::BindingReconstruct,
                &value_type_id("i32"),
                &value_type_id("i32"),
            ),
        },
    ];
    let semantic_operation = foundation_signature(
        &fixture.closed,
        &format!("{}.read", fixture.sequence_type_id),
        &fixture.error_type_id,
    );
    let commutation = BindingOperationCommutation {
        binding_id: "binding.sequence".to_owned(),
        source_operation: ClosedOperationSignature {
            id: "mpk.csharp.source.callable.sequence_read".to_owned(),
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![fixture.source_type_id.clone(), value_type_id("i32")],
            normal_result_type_id: value_type_id("i32"),
            ordered_checks: semantic_operation.ordered_checks.clone(),
        },
        semantic_operation,
        operand_projection_ids: vec![
            "projection.sequence".to_owned(),
            "projection.i32".to_owned(),
        ],
        result_projection_id: "projection.i32".to_owned(),
        ordered_outcomes: vec![CheckCommutation {
            ordinal: 0,
            source_check_id: "index_range".to_owned(),
            semantic_check_id: "index_range".to_owned(),
            failure_projection_id: None,
        }],
    };
    validate_binding_operation_commutation(
        &fixture.roots,
        &fixture.closed,
        &projections,
        &commutation,
    )
    .expect("well-typed commuting operation square");

    let mut changed = commutation.clone();
    changed.operand_projection_ids.swap(0, 1);
    assert_vir_reject(
        validate_binding_operation_commutation(
            &fixture.roots,
            &fixture.closed,
            &projections,
            &changed,
        ),
        PracticalVirErrorCode::BindingCommutation,
    );
    let mut changed = commutation.clone();
    changed.result_projection_id = "projection.sequence".to_owned();
    assert_vir_reject(
        validate_binding_operation_commutation(
            &fixture.roots,
            &fixture.closed,
            &projections,
            &changed,
        ),
        PracticalVirErrorCode::BindingCommutation,
    );
    let mut changed = commutation.clone();
    changed.ordered_outcomes[0].ordinal = 1;
    assert_vir_reject(
        validate_binding_operation_commutation(
            &fixture.roots,
            &fixture.closed,
            &projections,
            &changed,
        ),
        PracticalVirErrorCode::CheckOrder,
    );
    let mut changed = commutation.clone();
    changed.binding_id = "binding.unrelated".to_owned();
    assert_vir_reject(
        validate_binding_operation_commutation(
            &fixture.roots,
            &fixture.closed,
            &projections,
            &changed,
        ),
        PracticalVirErrorCode::BindingCommutation,
    );
    let mut changed = commutation;
    changed.semantic_operation.argument_type_ids.pop();
    assert_vir_reject(
        validate_binding_operation_commutation(
            &fixture.roots,
            &fixture.closed,
            &projections,
            &changed,
        ),
        PracticalVirErrorCode::OperandType,
    );
}

#[test]
fn csharp_03_t02_w03_enforces_linear_sequence_construction_state() {
    let fixture = w03_fixture();
    let state = SequenceConstructionState::allocate(
        &fixture.closed,
        "construction.result",
        &fixture.construction_type_id,
        "owner.initial",
        2,
        false,
        4_096,
    )
    .expect("allocate concrete construction state");
    assert_eq!(state.version, 0);
    assert!(state.initialized_indices.is_empty());
    assert_vir_reject(
        state.apply(
            &fixture.closed,
            &SequenceConstructionAction::Read {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                result_type_id: value_type_id("i32"),
            },
        ),
        PracticalVirErrorCode::ConstructionInitialization,
    );
    let left = state
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Fill {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                value_type_id: value_type_id("i32"),
            },
        )
        .expect("first fill")
        .state;
    let right = state
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Fill {
                actor_id: "owner.initial".to_owned(),
                index: 1,
                value_type_id: value_type_id("i32"),
            },
        )
        .expect("other branch fill")
        .state;
    for action in [
        SequenceConstructionAction::Borrow {
            actor_id: "owner.initial".to_owned(),
            borrower_id: "premature.borrow".to_owned(),
        },
        SequenceConstructionAction::Transfer {
            actor_id: "owner.initial".to_owned(),
            new_owner_id: "premature.owner".to_owned(),
        },
    ] {
        assert_vir_reject(
            left.apply(&fixture.closed, &action),
            PracticalVirErrorCode::ConstructionOwnership,
        );
    }
    let merged = SequenceConstructionState::merge(&fixture.closed, &left, &right)
        .expect("identical owner/version states merge");
    assert!(merged.initialized_indices.is_empty());
    let complete = left
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Fill {
                actor_id: "owner.initial".to_owned(),
                index: 1,
                value_type_id: value_type_id("i32"),
            },
        )
        .expect("second fill")
        .state;
    let read = complete
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Read {
                actor_id: "owner.initial".to_owned(),
                index: 1,
                result_type_id: value_type_id("i32"),
            },
        )
        .expect("initialized read");
    assert_eq!(
        read.read_type_id.as_deref(),
        Some("mpk.csharp.value.i32.v1")
    );
    let rewritten = complete
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Rewrite {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                value_type_id: value_type_id("i32"),
            },
        )
        .expect("complete unique rewrite")
        .state;
    let borrowed = rewritten
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Borrow {
                actor_id: "owner.initial".to_owned(),
                borrower_id: "foreach.borrow".to_owned(),
            },
        )
        .expect("read-only borrow")
        .state;
    borrowed
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Read {
                actor_id: "foreach.borrow".to_owned(),
                index: 0,
                result_type_id: value_type_id("i32"),
            },
        )
        .expect("borrowed read");
    assert_vir_reject(
        borrowed.apply(
            &fixture.closed,
            &SequenceConstructionAction::Rewrite {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                value_type_id: value_type_id("i32"),
            },
        ),
        PracticalVirErrorCode::ConstructionOwnership,
    );
    let unborrowed = borrowed
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::EndBorrow {
                actor_id: "owner.initial".to_owned(),
                borrower_id: "foreach.borrow".to_owned(),
            },
        )
        .expect("end exact borrow")
        .state;
    let transferred = unborrowed
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Transfer {
                actor_id: "owner.initial".to_owned(),
                new_owner_id: "owner.next".to_owned(),
            },
        )
        .expect("linear transfer")
        .state;
    assert_vir_reject(
        transferred.apply(
            &fixture.closed,
            &SequenceConstructionAction::Read {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                result_type_id: value_type_id("i32"),
            },
        ),
        PracticalVirErrorCode::ConstructionOwnership,
    );
    let frozen = transferred
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Freeze {
                actor_id: "owner.next".to_owned(),
                result_type_id: fixture.sequence_type_id.clone(),
            },
        )
        .expect("complete publication");
    assert_eq!(frozen.state.status, ConstructionStatus::Frozen);
    assert_eq!(
        frozen.published_type_id,
        Some(fixture.sequence_type_id.clone())
    );
    let mut forged_overbound_frozen = frozen.state.clone();
    forged_overbound_frozen.publication_length_maximum = 1;
    assert_vir_reject(
        forged_overbound_frozen.validate(&fixture.closed),
        PracticalVirErrorCode::ConstructionBound,
    );

    let discarded = state
        .apply(
            &fixture.closed,
            &SequenceConstructionAction::Discard {
                actor_id: "owner.initial".to_owned(),
            },
        )
        .expect("partial abrupt discard")
        .state;
    assert_eq!(discarded.status, ConstructionStatus::Discarded);
    assert!(discarded.initialized_indices.is_empty());

    assert_vir_reject(
        SequenceConstructionState::allocate(
            &fixture.closed,
            "construction.over",
            &fixture.construction_type_id,
            "owner.initial",
            i64::from(SEQUENCE_CONSTRUCTION_CAPACITY_MAX) + 1,
            false,
            SEQUENCE_CONSTRUCTION_CAPACITY_MAX,
        ),
        PracticalVirErrorCode::ConstructionBound,
    );
    assert_vir_reject(
        SequenceConstructionState::allocate(
            &fixture.closed,
            "construction.bad_publication_bound",
            &fixture.construction_type_id,
            "owner.initial",
            0,
            false,
            SEQUENCE_CONSTRUCTION_CAPACITY_MAX + 1,
        ),
        PracticalVirErrorCode::ConstructionBound,
    );
    let publication_limited = SequenceConstructionState::allocate(
        &fixture.closed,
        "construction.publication",
        &fixture.construction_type_id,
        "owner.initial",
        2,
        true,
        1,
    )
    .expect("construction allocation may exceed its later publication role");
    assert_vir_reject(
        publication_limited.apply(
            &fixture.closed,
            &SequenceConstructionAction::Freeze {
                actor_id: "owner.initial".to_owned(),
                result_type_id: fixture.sequence_type_id.clone(),
            },
        ),
        PracticalVirErrorCode::ConstructionBound,
    );
    assert_vir_reject(
        state.apply(
            &fixture.closed,
            &SequenceConstructionAction::Fill {
                actor_id: "owner.initial".to_owned(),
                index: -1,
                value_type_id: value_type_id("i32"),
            },
        ),
        PracticalVirErrorCode::ConstructionIndex,
    );
    assert_vir_reject(
        left.apply(
            &fixture.closed,
            &SequenceConstructionAction::Fill {
                actor_id: "owner.initial".to_owned(),
                index: 0,
                value_type_id: value_type_id("i32"),
            },
        ),
        PracticalVirErrorCode::ConstructionInitialization,
    );
    assert_vir_reject(
        left.apply(
            &fixture.closed,
            &SequenceConstructionAction::Freeze {
                actor_id: "owner.initial".to_owned(),
                result_type_id: fixture.sequence_type_id.clone(),
            },
        ),
        PracticalVirErrorCode::ConstructionInitialization,
    );
    let mut wrong_version = right;
    wrong_version.version += 1;
    assert_vir_reject(
        SequenceConstructionState::merge(&fixture.closed, &left, &wrong_version),
        PracticalVirErrorCode::ConstructionState,
    );
    let mut wrong_instance = state;
    wrong_instance.instance_id = fixture.sequence_type_id.clone();
    assert_vir_reject(
        wrong_instance.validate(&fixture.closed),
        PracticalVirErrorCode::ConstructionInstance,
    );
}

#[test]
fn csharp_03_t02_w03_validates_exception_values_handlers_and_explicit_control() {
    let fixture = w03_fixture();
    let universe = w03_exception_universe(&fixture);
    assert_eq!(universe.arms().len(), 10);
    assert_eq!(universe.arms()[0].type_id, "System.DivideByZeroException");
    assert_eq!(
        universe.arms()[8].type_id,
        "System.Runtime.CompilerServices.SwitchExpressionException"
    );
    assert_eq!(universe.arms()[9].type_id, fixture.source_exception_type_id);

    let builtin = MonomorphicValue::ClosedException {
        type_id: value_type_id("exception"),
        tag: 1,
        source_type_id: None,
        payload: None,
    };
    validate_explicit_exception_value(
        &fixture.bundle,
        &fixture.roots,
        &fixture.closed,
        &universe,
        &builtin,
    )
    .expect("built-in exception arm");
    let source_exception = MonomorphicValue::ClosedException {
        type_id: value_type_id("exception"),
        tag: 9,
        source_type_id: Some(fixture.source_exception_type_id.clone()),
        payload: Some(Box::new(MonomorphicValue::Product {
            type_id: fixture.source_exception_type_id.clone(),
            fields: vec![NamedMonomorphicValue {
                name: "code".to_owned(),
                value: Box::new(signed_i32("7")),
            }],
        })),
    };
    validate_explicit_exception_value(
        &fixture.bundle,
        &fixture.roots,
        &fixture.closed,
        &universe,
        &source_exception,
    )
    .expect("source exception arm and payload");
    let mut wrong_tag = source_exception;
    if let MonomorphicValue::ClosedException { tag, .. } = &mut wrong_tag {
        *tag = 10;
    }
    assert_vir_reject(
        validate_explicit_exception_value(
            &fixture.bundle,
            &fixture.roots,
            &fixture.closed,
            &universe,
            &wrong_tag,
        ),
        PracticalVirErrorCode::ExceptionType,
    );

    let graph = valid_control_graph();
    validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &graph)
        .expect("reducible explicit control graph");

    let mut shared_handler = graph.clone();
    shared_handler.nodes[1]
        .exceptional_successors
        .push(ExceptionalSuccessor {
            check_id: "operation.range".to_owned(),
            exception_type_id: "System.ArgumentOutOfRangeException".to_owned(),
            target_id: "handler.broad".to_owned(),
        });
    shared_handler.unwind_plans.push(ExceptionUnwindPlan {
        source_node_id: "try".to_owned(),
        check_id: "operation.range".to_owned(),
        from_region_id: Some("region.outer".to_owned()),
        selected_handler_region_id: Some("region.outer".to_owned()),
        finally_region_ids: Vec::new(),
        destination_node_id: "handler.broad".to_owned(),
    });
    validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &shared_handler)
        .expect("distinct exceptional checks may share one handler target");

    for tag in [
        PatternTag::Constant,
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
    ] {
        let mut variant = graph.clone();
        let arm = &mut variant.patterns[0].arms[0];
        arm.tag = tag;
        if tag == PatternTag::DeclarationType {
            arm.finite_sealed_type = true;
        }
        if tag == PatternTag::Property {
            arm.property_accesses.push(PatternPropertyAccess {
                member_id: "member.total".to_owned(),
                total: true,
                pure: true,
            });
        }
        if tag == PatternTag::List {
            arm.bounded_list = true;
        }
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &variant)
            .unwrap_or_else(|error| panic!("{} pattern: {error:?}", tag.as_str()));
    }
    let mut var_pattern = graph.clone();
    var_pattern.patterns[0].arms = vec![PatternArm {
        ordinal: 0,
        tag: PatternTag::Var,
        target_node_id: "return".to_owned(),
        guard_ordinal: None,
        guard_type_id: None,
        bound_parameter_type_ids: vec![value_type_id("i32")],
        property_accesses: Vec::new(),
        finite_sealed_type: false,
        bounded_list: false,
        has_slice: false,
    }];
    var_pattern.nodes[6].normal_successor_ids = vec!["return".to_owned()];
    validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &var_pattern)
        .expect("var catch-all pattern");
    let mut non_exhaustive = graph.clone();
    non_exhaustive.patterns[0].arms[1].tag = PatternTag::Constant;
    non_exhaustive.patterns[0].exhaustive = false;
    let switch_edge = ExceptionalSuccessor {
        check_id: "switch.non_exhaustive".to_owned(),
        exception_type_id: "System.Runtime.CompilerServices.SwitchExpressionException".to_owned(),
        target_id: "inner.finally".to_owned(),
    };
    non_exhaustive.patterns[0].non_exhaustive_exceptional_successor = Some(switch_edge.clone());
    non_exhaustive.nodes[6].exceptional_successors = vec![switch_edge];
    non_exhaustive.unwind_plans.push(ExceptionUnwindPlan {
        source_node_id: "pattern".to_owned(),
        check_id: "switch.non_exhaustive".to_owned(),
        from_region_id: Some("region.inner".to_owned()),
        selected_handler_region_id: None,
        finally_region_ids: vec!["region.inner".to_owned(), "region.outer".to_owned()],
        destination_node_id: "exit".to_owned(),
    });
    validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &non_exhaustive)
        .expect("non-exhaustive expression has an explicit exception edge");
    let mut rethrow = graph.clone();
    rethrow.nodes[8].tag = ControlNodeTag::Rethrow;
    rethrow.nodes[8].region_stack = vec!["region.outer".to_owned()];
    rethrow.nodes[8].exceptional_successors[0].target_id = "outer.finally".to_owned();
    if let Some(AbruptCompletion::Throw {
        rethrow_from_catch_id,
        ..
    }) = &mut rethrow.nodes[8].abrupt
    {
        *rethrow_from_catch_id = Some("handler".to_owned());
    }
    rethrow.unwind_plans[1].from_region_id = Some("region.outer".to_owned());
    rethrow.unwind_plans[1].selected_handler_region_id = None;
    rethrow.unwind_plans[1].finally_region_ids = vec!["region.outer".to_owned()];
    rethrow.unwind_plans[1].destination_node_id = "exit".to_owned();
    validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &rethrow)
        .expect("explicit rethrow retains its active catch identity");

    let root_throw_graph = ExplicitControlGraph {
        nodes: vec![
            control_node(0, "root.entry", ControlNodeTag::Entry, &["root.throw"], &[]),
            ControlNode {
                exceptional_successors: vec![ExceptionalSuccessor {
                    check_id: "root.explicit.throw".to_owned(),
                    exception_type_id: "System.OverflowException".to_owned(),
                    target_id: "root.exit".to_owned(),
                }],
                abrupt: Some(AbruptCompletion::Throw {
                    exception_type_id: "System.OverflowException".to_owned(),
                    rethrow_from_catch_id: None,
                }),
                ..control_node(1, "root.throw", ControlNodeTag::Throw, &[], &[])
            },
            ControlNode {
                abrupt: Some(AbruptCompletion::Normal),
                ..control_node(2, "root.exit", ControlNodeTag::Exit, &[], &[])
            },
        ],
        loops: Vec::new(),
        patterns: Vec::new(),
        exception_regions: Vec::new(),
        unwind_plans: vec![ExceptionUnwindPlan {
            source_node_id: "root.throw".to_owned(),
            check_id: "root.explicit.throw".to_owned(),
            from_region_id: None,
            selected_handler_region_id: None,
            finally_region_ids: Vec::new(),
            destination_node_id: "root.exit".to_owned(),
        }],
    };
    validate_explicit_control_graph(
        &fixture.roots,
        &fixture.closed,
        &universe,
        &root_throw_graph,
    )
    .expect("uncaught root exception retains an explicit method-exit edge");

    let mut changed = graph.clone();
    changed.nodes[2].condition_type_id = Some(value_type_id("i32"));
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ControlShape,
    );
    let mut changed = graph.clone();
    changed.nodes[3].ordinal = 9;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ControlOrder,
    );
    let mut changed = graph.clone();
    changed.nodes[1].tag = ControlNodeTag::Entry;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ControlShape,
    );
    let mut changed = graph.clone();
    changed.nodes[6].loop_id = Some("method#loop#0000".to_owned());
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ControlShape,
    );
    let mut changed = graph.clone();
    changed.nodes[1]
        .exceptional_successors
        .push(ExceptionalSuccessor {
            check_id: "operation.exception".to_owned(),
            exception_type_id: "System.OverflowException".to_owned(),
            target_id: "inner.finally".to_owned(),
        });
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ExceptionalSuccessor,
    );
    let mut changed = graph.clone();
    changed.nodes[0].normal_successor_ids[0] = "missing.node".to_owned();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::ControlEdge,
    );
    let mut changed = graph.clone();
    changed.loops[0].backedge_source_ids.clear();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::LoopShape,
    );
    let mut changed = graph.clone();
    changed.loops.clear();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::LoopShape,
    );
    let mut changed = graph.clone();
    changed.patterns[0].governing_evaluation_count = 2;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternShape,
    );
    let mut changed = graph.clone();
    changed.patterns[0].arms[0].guard_ordinal = Some(1);
    changed.patterns[0].arms[0].guard_type_id = Some(value_type_id("bool"));
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternOrder,
    );
    let mut changed = graph.clone();
    changed.patterns[0].arms[1].guard_ordinal = Some(0);
    changed.patterns[0].arms[1].guard_type_id = Some(value_type_id("bool"));
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternOrder,
    );
    let mut changed = graph.clone();
    changed.patterns[0].exhaustive = false;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternOrder,
    );
    let mut changed = non_exhaustive;
    changed.patterns[0].non_exhaustive_exceptional_successor = None;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternExhaustiveness,
    );
    let mut changed = graph.clone();
    changed.patterns[0].arms[0].tag = PatternTag::Property;
    changed.patterns[0].arms[0].property_accesses = vec![PatternPropertyAccess {
        member_id: "member.code".to_owned(),
        total: false,
        pure: true,
    }];
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternShape,
    );
    let mut changed = graph.clone();
    changed.patterns[0].arms[0].tag = PatternTag::List;
    changed.patterns[0].arms[0].bounded_list = true;
    changed.patterns[0].arms[0].has_slice = true;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::PatternShape,
    );
    let mut changed = graph.clone();
    changed.exception_regions[0].catches[0]
        .filter
        .as_mut()
        .expect("filter")
        .preserves_original_exception = false;
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::HandlerShape,
    );
    let mut changed = graph.clone();
    let broad = changed.exception_regions[0]
        .catches
        .pop()
        .expect("broad catch");
    changed.exception_regions[0].catches.insert(0, broad);
    for (ordinal, catch) in changed.exception_regions[0].catches.iter_mut().enumerate() {
        catch.ordinal = u32::try_from(ordinal).expect("catch ordinal");
    }
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::HandlerOrder,
    );
    let mut changed = graph.clone();
    changed.exception_regions[0].catches[0].exception_type_id = "System.Exception".to_owned();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::HandlerOrder,
    );
    let mut changed = graph.clone();
    changed.exception_regions[0].catches[1].handler_entry_node_id = "handler".to_owned();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::HandlerOrder,
    );
    let mut changed = graph;
    changed.unwind_plans[0].finally_region_ids = vec!["region.outer".to_owned()];
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::UnwindOrder,
    );
    let mut changed = valid_control_graph();
    changed.unwind_plans.clear();
    assert_vir_reject(
        validate_explicit_control_graph(&fixture.roots, &fixture.closed, &universe, &changed),
        PracticalVirErrorCode::UnwindOrder,
    );

    let incoming = AbruptCompletion::Return {
        value_type_id: Some(value_type_id("i32")),
    };
    validate_finally_completion(&FinallyCompletionRule {
        incoming: incoming.clone(),
        produced: AbruptCompletion::Normal,
        outgoing: incoming.clone(),
    })
    .expect("normal finally preserves incoming completion");
    let replacement = AbruptCompletion::Throw {
        exception_type_id: "System.OverflowException".to_owned(),
        rethrow_from_catch_id: None,
    };
    validate_finally_completion(&FinallyCompletionRule {
        incoming: incoming.clone(),
        produced: replacement.clone(),
        outgoing: replacement,
    })
    .expect("finally throw replaces incoming completion");
    assert_vir_reject(
        validate_finally_completion(&FinallyCompletionRule {
            incoming,
            produced: AbruptCompletion::Break {
                loop_id: "method#loop#0000".to_owned(),
                target_id: "loop.exit".to_owned(),
            },
            outgoing: AbruptCompletion::Normal,
        }),
        PracticalVirErrorCode::FinallyAbrupt,
    );
}

struct W03Fixture {
    bundle: ValidatedFoundationBundle,
    roots: ValidatedClosedRootSet,
    closed: ClosedInstanceSet,
    source_type_id: String,
    error_type_id: String,
    source_exception_type_id: String,
    source_exception_member_ids: Vec<String>,
    sequence_type_id: String,
    construction_type_id: String,
    parse_result_type_id: String,
}

fn w03_fixture() -> W03Fixture {
    let package = read_json(FOUNDATION_VECTORS);
    let vector = package["vectors"]
        .as_array()
        .expect("vectors")
        .iter()
        .find(|vector| vector["id"] == "specialization.all_templates")
        .expect("all-template vector");
    let mut roots = vector["inputs"]["roots"]
        .as_array()
        .expect("root array")
        .clone();
    roots.push(json!({
        "origin": "codec_result",
        "provenance_id": "root.w03.parse_result",
        "type": instance_type("result", vec![
            primitive_type("i32"),
            primitive_type("parse_error"),
        ]),
    }));
    let source = source_fixture(
        "SequenceValue",
        "readonly_struct",
        &[("value", primitive_type("i32"))],
        &[],
    );
    let error = source_fixture("OperationError", "enum", &[], &[0, 1]);
    let exception = source_fixture(
        "BusinessException",
        "sealed_class",
        &[("code", primitive_type("i32"))],
        &[],
    );
    let source_type_id = source["id"].as_str().expect("source type ID").to_owned();
    let error_type_id = error["id"].as_str().expect("error type ID").to_owned();
    let source_exception_type_id = exception["id"]
        .as_str()
        .expect("exception type ID")
        .to_owned();
    let source_exception_member_ids = exception["members"]
        .as_array()
        .expect("exception members")
        .iter()
        .map(|member| member["id"].as_str().expect("member ID").to_owned())
        .collect::<Vec<_>>();
    for (ordinal, source_id) in [
        source_type_id.as_str(),
        error_type_id.as_str(),
        source_exception_type_id.as_str(),
    ]
    .iter()
    .enumerate()
    {
        roots.push(json!({
            "origin": "semantic_binding",
            "provenance_id": format!("root.w03.source.{ordinal}"),
            "type": {"kind": "source", "id": source_id},
        }));
    }
    let bundle = registered_bundle();
    let roots = root_set(
        &bundle,
        &Value::Array(roots),
        &json!({
            source_type_id.clone(): source,
            error_type_id.clone(): error,
            source_exception_type_id.clone(): exception,
        }),
    );
    let closed = derive_closed_instances(&bundle, &roots).expect("W03 closed instances");
    let sequence_type_id =
        find_closed_instance(&closed, "bounded_sequence", &[value_type_id("i32")]);
    let construction_type_id =
        find_closed_instance(&closed, "sequence_construction", &[value_type_id("i32")]);
    let parse_result_type_id = find_closed_instance(
        &closed,
        "result",
        &[value_type_id("i32"), value_type_id("parse_error")],
    );
    W03Fixture {
        bundle,
        roots,
        closed,
        source_type_id,
        error_type_id,
        source_exception_type_id,
        source_exception_member_ids,
        sequence_type_id,
        construction_type_id,
        parse_result_type_id,
    }
}

fn find_closed_instance(
    closed: &ClosedInstanceSet,
    template: &str,
    arguments: &[String],
) -> String {
    closed
        .entries()
        .iter()
        .find(|entry| {
            entry["template_id"] == format!("mpk.csharp.semantic.{template}.v1")
                && entry["argument_ids"] == json!(arguments)
        })
        .and_then(|entry| entry["instance_id"].as_str())
        .unwrap_or_else(|| panic!("missing closed {template} instance"))
        .to_owned()
}

fn foundation_signature(
    closed: &ClosedInstanceSet,
    operation_id: &str,
    error_type_id: &str,
) -> ClosedOperationSignature {
    let operation = closed
        .entries()
        .iter()
        .flat_map(|entry| {
            entry["operation_definitions"]
                .as_array()
                .expect("operation definitions")
        })
        .find(|operation| operation["id"] == operation_id)
        .unwrap_or_else(|| panic!("missing foundation operation {operation_id}"));
    ClosedOperationSignature {
        id: operation_id.to_owned(),
        tag: ClosedOperationTag::Foundation,
        argument_type_ids: operation["argument_type_ids"]
            .as_array()
            .expect("argument types")
            .iter()
            .map(|value| value.as_str().expect("type ID").to_owned())
            .collect(),
        normal_result_type_id: operation["normal_result_type_id"]
            .as_str()
            .expect("result type")
            .to_owned(),
        ordered_checks: operation["error_precedence"]
            .as_array()
            .expect("checks")
            .iter()
            .map(|value| w03_check(value.as_str().expect("check ID"), error_type_id))
            .collect(),
    }
}

fn w03_check(id: &str, error_type_id: &str) -> RequiredCheck {
    match id {
        "negative_length" => exception_check(id, "System.OverflowException"),
        "index_range" => exception_check(id, "System.IndexOutOfRangeException"),
        "invalid_operation" => exception_check(id, "System.InvalidOperationException"),
        "ownership"
        | "incomplete"
        | "already_initialized"
        | "uninitialized"
        | "construction_bound"
        | "publication_bound"
        | "invalid_representation" => static_check(id),
        _ => RequiredCheck {
            id: id.to_owned(),
            tag: RequiredCheckTag::ErrorOutcome,
            failure_type_id: Some(error_type_id.to_owned()),
        },
    }
}

fn static_check(id: &str) -> RequiredCheck {
    RequiredCheck {
        id: id.to_owned(),
        tag: RequiredCheckTag::StaticObligation,
        failure_type_id: None,
    }
}

fn parse_check(id: &str) -> RequiredCheck {
    RequiredCheck {
        id: id.to_owned(),
        tag: RequiredCheckTag::ParseError,
        failure_type_id: Some(value_type_id("parse_error")),
    }
}

fn exception_check(id: &str, type_id: &str) -> RequiredCheck {
    RequiredCheck {
        id: id.to_owned(),
        tag: RequiredCheckTag::Exception,
        failure_type_id: Some(type_id.to_owned()),
    }
}

fn projection_signature(
    id: &str,
    tag: ClosedOperationTag,
    source: &str,
    result: &str,
) -> ClosedOperationSignature {
    ClosedOperationSignature {
        id: id.to_owned(),
        tag,
        argument_type_ids: vec![source.to_owned()],
        normal_result_type_id: result.to_owned(),
        ordered_checks: Vec::new(),
    }
}

fn invocation_for(
    signature: &ClosedOperationSignature,
    normal: &str,
    exceptional: &str,
) -> OperationInvocation {
    OperationInvocation {
        operation_id: signature.id.clone(),
        operands: signature
            .argument_type_ids
            .iter()
            .enumerate()
            .map(|(ordinal, type_id)| TypedValueRef {
                id: format!("operand.{ordinal}"),
                type_id: type_id.clone(),
            })
            .collect(),
        result: TypedValueRef {
            id: "result.0".to_owned(),
            type_id: signature.normal_result_type_id.clone(),
        },
        ordered_check_ids: signature
            .ordered_checks
            .iter()
            .map(|check| check.id.clone())
            .collect(),
        normal_successor_id: normal.to_owned(),
        exceptional_successors: signature
            .ordered_checks
            .iter()
            .filter(|check| check.tag == RequiredCheckTag::Exception)
            .map(|check| ExceptionalSuccessor {
                check_id: check.id.clone(),
                exception_type_id: check
                    .failure_type_id
                    .clone()
                    .expect("exception failure type"),
                target_id: exceptional.to_owned(),
            })
            .collect(),
    }
}

fn w03_exception_universe(fixture: &W03Fixture) -> ClosedExceptionUniverse {
    derive_closed_exception_universe(
        &fixture.roots,
        &fixture.closed,
        &[SourceExceptionDefinition {
            type_id: fixture.source_exception_type_id.clone(),
            sealed: true,
            direct_base_type_id: "System.Exception".to_owned(),
            payload_member_ids: fixture.source_exception_member_ids.clone(),
        }],
    )
    .expect("closed exception universe")
}

fn valid_control_graph() -> ExplicitControlGraph {
    let loop_id = "method#loop#0000";
    let nodes = vec![
        control_node(0, "entry", ControlNodeTag::Entry, &["try"], &[]),
        ControlNode {
            exceptional_successors: vec![ExceptionalSuccessor {
                check_id: "operation.exception".to_owned(),
                exception_type_id: "System.ArgumentNullException".to_owned(),
                target_id: "handler.broad".to_owned(),
            }],
            ..control_node(
                1,
                "try",
                ControlNodeTag::Operation,
                &["header"],
                &["region.outer"],
            )
        },
        ControlNode {
            condition_type_id: Some(value_type_id("bool")),
            loop_id: Some(loop_id.to_owned()),
            ..control_node(
                2,
                "header",
                ControlNodeTag::LoopHeader,
                &["body", "pattern"],
                &["region.outer", "region.inner"],
            )
        },
        ControlNode {
            condition_type_id: Some(value_type_id("bool")),
            ..control_node(
                3,
                "body",
                ControlNodeTag::Branch,
                &["continue", "break"],
                &["region.outer", "region.inner"],
            )
        },
        ControlNode {
            abrupt: Some(AbruptCompletion::Continue {
                loop_id: loop_id.to_owned(),
                target_id: "backedge".to_owned(),
            }),
            loop_id: Some(loop_id.to_owned()),
            ..control_node(
                4,
                "continue",
                ControlNodeTag::Continue,
                &[],
                &["region.outer", "region.inner"],
            )
        },
        ControlNode {
            abrupt: Some(AbruptCompletion::Break {
                loop_id: loop_id.to_owned(),
                target_id: "pattern".to_owned(),
            }),
            loop_id: Some(loop_id.to_owned()),
            ..control_node(
                5,
                "break",
                ControlNodeTag::Break,
                &[],
                &["region.outer", "region.inner"],
            )
        },
        control_node(
            6,
            "pattern",
            ControlNodeTag::PatternDecision,
            &["return", "throw"],
            &["region.outer", "region.inner"],
        ),
        ControlNode {
            abrupt: Some(AbruptCompletion::Return {
                value_type_id: Some(value_type_id("i32")),
            }),
            ..control_node(
                7,
                "return",
                ControlNodeTag::Return,
                &[],
                &["region.outer", "region.inner"],
            )
        },
        ControlNode {
            exceptional_successors: vec![ExceptionalSuccessor {
                check_id: "explicit.throw".to_owned(),
                exception_type_id: "System.ArgumentNullException".to_owned(),
                target_id: "inner.finally".to_owned(),
            }],
            abrupt: Some(AbruptCompletion::Throw {
                exception_type_id: "System.ArgumentNullException".to_owned(),
                rethrow_from_catch_id: None,
            }),
            ..control_node(
                8,
                "throw",
                ControlNodeTag::Throw,
                &[],
                &["region.outer", "region.inner"],
            )
        },
        control_node(
            9,
            "handler",
            ControlNodeTag::HandlerEntry,
            &["outer.finally"],
            &["region.outer"],
        ),
        control_node(
            10,
            "handler.broad",
            ControlNodeTag::HandlerEntry,
            &["outer.finally"],
            &["region.outer"],
        ),
        control_node(
            11,
            "inner.finally",
            ControlNodeTag::FinallyEntry,
            &["inner.finally.exit"],
            &["region.outer", "region.inner"],
        ),
        control_node(
            12,
            "inner.finally.exit",
            ControlNodeTag::FinallyExit,
            &["handler"],
            &["region.outer", "region.inner"],
        ),
        control_node(
            13,
            "outer.finally",
            ControlNodeTag::FinallyEntry,
            &["outer.finally.exit"],
            &["region.outer"],
        ),
        control_node(
            14,
            "outer.finally.exit",
            ControlNodeTag::FinallyExit,
            &["exit"],
            &["region.outer"],
        ),
        ControlNode {
            abrupt: Some(AbruptCompletion::Normal),
            ..control_node(15, "exit", ControlNodeTag::Exit, &[], &[])
        },
        control_node(
            16,
            "backedge",
            ControlNodeTag::Jump,
            &["header"],
            &["region.outer", "region.inner"],
        ),
    ];
    ExplicitControlGraph {
        nodes,
        loops: vec![LoopRegion {
            id: loop_id.to_owned(),
            parent_loop_id: None,
            header_node_id: "header".to_owned(),
            body_entry_node_id: "body".to_owned(),
            continue_target_node_id: "backedge".to_owned(),
            break_target_node_id: "pattern".to_owned(),
            backedge_source_ids: vec!["backedge".to_owned()],
        }],
        patterns: vec![PatternDecision {
            node_id: "pattern".to_owned(),
            governing_value_id: "switch.value".to_owned(),
            governing_type_id: value_type_id("i32"),
            governing_evaluation_count: 1,
            expression: true,
            exhaustive: true,
            arms: vec![
                PatternArm {
                    ordinal: 0,
                    tag: PatternTag::Constant,
                    target_node_id: "return".to_owned(),
                    guard_ordinal: None,
                    guard_type_id: None,
                    bound_parameter_type_ids: Vec::new(),
                    property_accesses: Vec::new(),
                    finite_sealed_type: false,
                    bounded_list: false,
                    has_slice: false,
                },
                PatternArm {
                    ordinal: 1,
                    tag: PatternTag::Discard,
                    target_node_id: "throw".to_owned(),
                    guard_ordinal: None,
                    guard_type_id: None,
                    bound_parameter_type_ids: Vec::new(),
                    property_accesses: Vec::new(),
                    finite_sealed_type: false,
                    bounded_list: false,
                    has_slice: false,
                },
            ],
            no_match_target_id: None,
            non_exhaustive_exceptional_successor: None,
        }],
        exception_regions: vec![
            ExceptionHandlerRegion {
                id: "region.outer".to_owned(),
                parent_region_id: None,
                nesting_depth: 0,
                try_entry_node_id: "try".to_owned(),
                catches: vec![
                    CatchHandler {
                        ordinal: 0,
                        exception_type_id: "System.ArgumentNullException".to_owned(),
                        filter: Some(ExceptionFilterRule {
                            condition_type_id: value_type_id("bool"),
                            thrown_filter_exception_successor_id: "handler.broad".to_owned(),
                            throw_means_false: true,
                            preserves_original_exception: true,
                        }),
                        handler_entry_node_id: "handler".to_owned(),
                    },
                    CatchHandler {
                        ordinal: 1,
                        exception_type_id: "System.ArgumentException".to_owned(),
                        filter: None,
                        handler_entry_node_id: "handler.broad".to_owned(),
                    },
                ],
                finally_entry_node_id: Some("outer.finally".to_owned()),
            },
            ExceptionHandlerRegion {
                id: "region.inner".to_owned(),
                parent_region_id: Some("region.outer".to_owned()),
                nesting_depth: 1,
                try_entry_node_id: "header".to_owned(),
                catches: Vec::new(),
                finally_entry_node_id: Some("inner.finally".to_owned()),
            },
        ],
        unwind_plans: vec![
            ExceptionUnwindPlan {
                source_node_id: "try".to_owned(),
                check_id: "operation.exception".to_owned(),
                from_region_id: Some("region.outer".to_owned()),
                selected_handler_region_id: Some("region.outer".to_owned()),
                finally_region_ids: Vec::new(),
                destination_node_id: "handler.broad".to_owned(),
            },
            ExceptionUnwindPlan {
                source_node_id: "throw".to_owned(),
                check_id: "explicit.throw".to_owned(),
                from_region_id: Some("region.inner".to_owned()),
                selected_handler_region_id: Some("region.outer".to_owned()),
                finally_region_ids: vec!["region.inner".to_owned()],
                destination_node_id: "handler".to_owned(),
            },
        ],
    }
}

fn control_node(
    ordinal: u32,
    id: &str,
    tag: ControlNodeTag,
    normal_successors: &[&str],
    region_stack: &[&str],
) -> ControlNode {
    ControlNode {
        id: id.to_owned(),
        ordinal,
        tag,
        condition_type_id: None,
        normal_successor_ids: normal_successors
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        exceptional_successors: Vec::new(),
        abrupt: None,
        loop_id: None,
        region_stack: region_stack
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

fn assert_vir_reject<T: std::fmt::Debug>(
    result: Result<T, mpk_vc::csharp_practical_vir_model::PracticalVirValidationError>,
    expected: PracticalVirErrorCode,
) {
    let error = result.expect_err("W03 structural mutation must reject");
    assert_eq!(error.code(), expected);
}

fn round_trip_value(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    name: &str,
    value: &MonomorphicValue,
) {
    let transport = canonical_monomorphic_value_transport(bundle, roots, closed, value)
        .unwrap_or_else(|error| panic!("encode {name}: {error:?}"));
    assert_eq!(transport.last(), Some(&b'\n'), "{name}");
    let imported = import_monomorphic_value(bundle, roots, closed, &transport)
        .unwrap_or_else(|error| panic!("import {name}: {error:?}"));
    assert_eq!(&imported, value, "{name}");
    assert_eq!(
        canonical_monomorphic_value_transport(bundle, roots, closed, &imported)
            .expect("re-encode imported value"),
        transport,
        "{name}"
    );
    let text = std::str::from_utf8(&transport).expect("canonical UTF-8");
    assert!(!text.contains("\"template\""), "{name}");
    assert!(!text.contains("\"parameter\""), "{name}");
}

fn assert_value_reject<T: std::fmt::Debug>(
    result: Result<T, FoundationValidationError>,
    expected: FoundationErrorCode,
) {
    let error = result.expect_err("concrete value must reject");
    assert_eq!(error.code(), expected);
}

fn value_type_id(name: &str) -> String {
    format!("mpk.csharp.value.{name}.v1")
}

fn primitive_type(name: &str) -> Value {
    json!({"kind": "primitive", "id": name})
}

fn instance_type(template: &str, arguments: Vec<Value>) -> Value {
    json!({"kind": "instance", "template": template, "arguments": arguments})
}

fn closed_instance_id(
    bundle: &ValidatedFoundationBundle,
    template: &str,
    arguments: &[Value],
) -> String {
    csharp_practical_closed_instance_id(bundle, &instance_type(template, arguments.to_vec()))
        .expect("closed instance ID")
}

fn unit_value() -> MonomorphicValue {
    MonomorphicValue::Unit {
        type_id: value_type_id("unit"),
    }
}

fn signed_i32(value: &str) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: value_type_id("i32"),
        value: value.to_owned(),
    }
}

fn string_value(utf16: &[u16]) -> MonomorphicValue {
    MonomorphicValue::String {
        type_id: value_type_id("string"),
        utf16: utf16.to_vec(),
    }
}

fn decimal_value(negative: bool, scale: u8, coefficient: &str) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: value_type_id("decimal"),
        negative,
        scale,
        coefficient: coefficient.to_owned(),
    }
}

fn named_value(name: &str, value: MonomorphicValue) -> NamedMonomorphicValue {
    NamedMonomorphicValue {
        name: name.to_owned(),
        value: Box::new(value),
    }
}

fn ordered_map_value(type_id: &str, entries: &[(i32, i32)]) -> MonomorphicValue {
    MonomorphicValue::OrderedMap {
        type_id: type_id.to_owned(),
        entries: entries
            .iter()
            .map(|(key, value)| MonomorphicMapEntry {
                key: Box::new(signed_i32(&key.to_string())),
                value: Box::new(signed_i32(&value.to_string())),
            })
            .collect(),
    }
}

fn tagged_value(type_id: &str, arm: &str, payload: Vec<MonomorphicValue>) -> MonomorphicValue {
    MonomorphicValue::TaggedSum {
        type_id: type_id.to_owned(),
        arm: arm.to_owned(),
        payload,
    }
}

fn boundary_value(
    type_id: &str,
    arm: BoundaryArm,
    value: Option<MonomorphicValue>,
) -> MonomorphicValue {
    MonomorphicValue::BoundaryPresence {
        type_id: type_id.to_owned(),
        arm,
        value: value.map(Box::new),
    }
}

fn owned_vectors<'a>(package: &'a Value, family: &str) -> Vec<&'a Value> {
    package["vectors"]
        .as_array()
        .expect("foundation vectors")
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM && vector["family"] == family)
        .collect()
}

fn vectors_by_id<'a>(vectors: &[&'a Value]) -> BTreeMap<&'a str, &'a Value> {
    vectors
        .iter()
        .map(|vector| (vector["id"].as_str().expect("vector ID"), *vector))
        .collect()
}

fn assert_vector_results(vectors: &[&Value], actual: &BTreeMap<String, Value>) {
    let expected_ids = vectors
        .iter()
        .map(|vector| vector["id"].as_str().expect("vector ID").to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual.keys().cloned().collect::<BTreeSet<_>>(),
        expected_ids,
        "the production test must execute exactly every owned vector"
    );
    for vector in vectors {
        let id = vector["id"].as_str().expect("vector ID");
        assert_eq!(vector["production_test_owner"], OWNER, "{id}");
        assert_eq!(actual[id], vector["expected"], "{id}");
    }
}

fn registered_bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .expect("registered foundation bundle")
}

fn root_set(
    bundle: &ValidatedFoundationBundle,
    roots: &Value,
    source_types: &Value,
) -> ValidatedClosedRootSet {
    try_root_set(bundle, roots, source_types).expect("validated closed root set")
}

fn try_root_set(
    bundle: &ValidatedFoundationBundle,
    roots: &Value,
    source_types: &Value,
) -> Result<ValidatedClosedRootSet, FoundationValidationError> {
    let transport = canonical_closed_root_set_transport(bundle, roots, source_types)?;
    validate_closed_root_set(bundle, &transport)
}

fn rejection<T: std::fmt::Debug>(result: Result<T, FoundationValidationError>) -> Value {
    let error = result.expect_err("rejection vector must reject");
    json!({"reject": error.code().as_str()})
}

fn mutate_omit_dependency(value: &mut Value) {
    value["entries"].as_array_mut().expect("entries").pop();
}

fn mutate_reorder(value: &mut Value) {
    value["entries"].as_array_mut().expect("entries").reverse();
}

fn mutate_duplicate(value: &mut Value) {
    let entry = value["entries"].as_array().expect("entries")[0].clone();
    value["entries"]
        .as_array_mut()
        .expect("entries")
        .push(entry);
}

fn mutate_provenance(value: &mut Value) {
    value["entries"][0]["provenance_ids"]
        .as_array_mut()
        .expect("provenance")
        .push(json!("fake"));
}

fn mutate_residual_generic(value: &mut Value) {
    value["entries"][0]["type_definition"]["representation"] =
        json!({"kind": "parameter", "index": 0});
}

fn mutate_operation_body(value: &mut Value) {
    value["entries"][0]["operation_definitions"][0]["equation"] = json!("trusted");
}

fn mutate_counter(value: &mut Value) {
    value["counters"]["operations"] = json!(0);
}

fn nested_sequence(count: u64) -> Value {
    let mut ty = json!({"kind": "primitive", "id": "i32"});
    for _ in 0..count {
        ty = json!({"kind": "instance", "template": "bounded_sequence", "arguments": [ty]});
    }
    ty
}

fn large_instance_inputs(count: usize) -> (Value, Value) {
    let mut roots = Vec::with_capacity(count);
    let mut sources = Map::new();
    for index in 0..count {
        let source = source_fixture(&format!("E{index}"), "enum", &[], &[0]);
        let source_id = source["id"].as_str().expect("source ID").to_owned();
        sources.insert(source_id.clone(), source);
        roots.push(json!({
            "origin": "source_nullable",
            "provenance_id": format!("source.{index}"),
            "type": {
                "kind": "instance",
                "template": "option",
                "arguments": [{"kind": "source", "id": source_id}],
            },
        }));
    }
    (Value::Array(roots), Value::Object(sources))
}

fn source_fixture(name: &str, kind: &str, members: &[(&str, Value)], enum_values: &[i64]) -> Value {
    let identity = json!({
        "kind": "type",
        "namespace": "Example",
        "owner": "",
        "name": name,
        "parameter_type_ids": [],
        "result_type_id": "",
    });
    let source_id = csharp_practical_declaration_id(&identity).expect("source declaration ID");
    let mut member_values = Vec::new();
    let mut actual_default = Map::new();
    for (ordinal, (member_name, ty)) in members.iter().enumerate() {
        let member_id =
            csharp_practical_stored_member_id(&source_id, member_name, ty, "readonly_field")
                .expect("stored member ID");
        member_values.push(json!({
            "id": member_id,
            "name": member_name,
            "type": ty,
            "storage": "readonly_field",
            "ordinal": ordinal,
            "required": false,
        }));
        actual_default.insert(member_id, json!(0));
    }
    json!({
        "id": source_id,
        "identity": identity,
        "kind": kind,
        "members": member_values,
        "enum_values": enum_values.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "enum_underlying": if kind == "enum" { Value::String("i32".to_owned()) } else { Value::Null },
        "actual_default": actual_default,
        "public_default": true,
        "identity_sensitive": false,
        "source_sha256": raw_sha256(name.as_bytes()),
    })
}

fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({
        "kind": "type",
        "namespace": "Example",
        "owner": "",
        "name": name,
        "parameter_type_ids": [],
        "result_type_id": "",
    }))
    .expect("source ID")
}

fn raw_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("JSON transport");
    bytes.push(b'\n');
    bytes
}

fn read_json(path: &str) -> Value {
    let bytes = fs::read(repo_path(path)).unwrap_or_else(|error| panic!("read {path}: {error}"));
    read_json_bytes(&bytes)
}

fn read_json_bytes(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("JSON document")
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

#[allow(dead_code)]
fn assert_closed_hash(closed: &ClosedInstanceSet, expected: &str) {
    assert_eq!(closed.closed_set_sha256(), expected);
}
