use mpk_vc::csharp_practical_vir_model::{
    canonical_closed_root_set_transport, canonical_monomorphic_value_transport,
    csharp_practical_closed_instance_id, csharp_practical_declaration_id,
    csharp_practical_stored_member_id, derive_closed_instances, import_monomorphic_value,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_closed_instance_set, validate_closed_root_set, validate_foundation_structural_limit,
    validate_monomorphic_value, validate_practical_foundation_limit,
    validate_registered_foundation_bundle, BoundaryArm, ClosedInstanceSet, FoundationErrorCode,
    FoundationLimit, FoundationValidationError, MonomorphicMapEntry, MonomorphicValue,
    NamedMonomorphicValue, OptionArm, ParseErrorArm, ValidatedClosedRootSet,
    ValidatedFoundationBundle,
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
