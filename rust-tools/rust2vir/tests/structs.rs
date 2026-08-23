#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;
#[path = "support/rustc_harness.rs"]
mod rustc_harness;

use rust2vir_internal::json::{self, JsonValue};
use rustc_driver_adapter::mir_aggregate::{
    validate_struct_aggregate_pattern, StructAggregatePatternVector,
};
use rustc_driver_adapter::mir_projection::{
    validate_field_projection_pattern, FieldProjectionPatternVector,
};

const SOURCE: &[u8] = include_bytes!("../testdata/structs/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/structs/expected-vir.json");
type StructPatternMutation = Box<dyn Fn(&mut StructAggregatePatternVector)>;
type FieldPatternMutation = Box<dyn Fn(&mut FieldProjectionPatternVector)>;

#[test]
fn nominal_construction_is_declaration_ordered_and_target_stable() {
    let mut modules = Vec::new();
    for (target, width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let contract = tautology_contract("vector::construct", width);
        let first = lower(target, width, "construct", &contract);
        let second = lower(target, width, "construct", &contract);
        assert_eq!(first, second);
        let vir = member(&first.raw_lowering, "vir");
        modules.push(vir.clone());

        let declarations = member(first_unit(vir), "type_decls").as_array().unwrap();
        assert_eq!(declarations.len(), 1);
        assert_eq!(
            member(&declarations[0], "id").as_str(),
            Some("vector::Point")
        );
        assert_eq!(member(&declarations[0], "name").as_str(), Some("Point"));
        assert_eq!(field_names(&declarations[0]), vec!["x", "y"]);

        let instruction = instructions(vir)
            .find(|instruction| member(instruction, "kind").as_str() == Some("MakeStruct"))
            .expect("MakeStruct instruction");
        assert_eq!(member(instruction, "id").as_str(), Some("t0"));
        assert_struct_type(member(instruction, "type"), "vector::Point");
        let fields = member(instruction, "fields").as_array().unwrap();
        assert_eq!(
            fields
                .iter()
                .map(|field| member(field, "name").as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["x", "y"]
        );
        assert_eq!(
            member(member(&fields[0], "value"), "var").as_str(),
            Some("arg0")
        );
        assert_eq!(
            member(member(&fields[1], "value"), "var").as_str(),
            Some("arg1")
        );
        assert_eq!(features(vir), vec!["struct"]);
        assert_eq!(
            instruction_source(&first.raw_source_map, "t0"),
            "Point { y, x }"
        );

        let text = std::str::from_utf8(&json::canonical(vir).unwrap())
            .unwrap()
            .to_owned();
        for forbidden in [
            "Unused",
            "offset",
            "padding",
            "alignment",
            "endian",
            "discriminant",
            "niche",
        ] {
            assert!(
                !text.contains(forbidden),
                "{forbidden} leaked into nominal VIR"
            );
        }
    }
    let expected = json::parse(EXPECTED_VIR, EXPECTED_VIR.len()).expect("struct golden fixture");
    assert_eq!(JsonValue::Array(modules), expected);
}

#[test]
fn direct_copy_field_and_whole_move_lower_without_layout_semantics() {
    let read_contract = tautology_contract("vector::read_x", 64);
    let read = lower("x86_64-unknown-linux-gnu", 64, "read_x", &read_contract);
    let vir = member(&read.raw_lowering, "vir");
    let field = instructions(vir)
        .find(|instruction| member(instruction, "kind").as_str() == Some("Field"))
        .expect("Field instruction");
    assert_eq!(member(field, "field").as_str(), Some("x"));
    assert_eq!(member(member(field, "base"), "var").as_str(), Some("arg0"));
    assert_eq!(instruction_source(&read.raw_source_map, "t0"), "point.x");

    let constructed_contract = tautology_contract("vector::constructed_x", 64);
    let constructed = lower(
        "x86_64-unknown-linux-gnu",
        64,
        "constructed_x",
        &constructed_contract,
    );
    let vir = member(&constructed.raw_lowering, "vir");
    assert_eq!(
        instructions(vir)
            .map(|instruction| member(instruction, "kind").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["MakeStruct", "Field"]
    );
    assert_eq!(features(vir), vec!["struct"]);

    let move_contract = struct_result_contract("vector::move_whole", 64, "point");
    let moved = lower("x86_64-unknown-linux-gnu", 64, "move_whole", &move_contract);
    let vir = member(&moved.raw_lowering, "vir");
    assert_eq!(features(vir), vec!["mutable_local", "struct"]);
    assert!(instructions(vir).all(|instruction| {
        !matches!(
            member(instruction, "kind").as_str(),
            Some("Field" | "MakeStruct")
        )
    }));
    let ensure = &member(member(first_function(vir), "contracts"), "ensures")
        .as_array()
        .unwrap()[0];
    assert_eq!(member(ensure, "op").as_str(), Some("eq"));
    assert_eq!(member(member(ensure, "rhs"), "var").as_str(), Some("arg0"));
}

#[test]
fn nested_declarations_are_dependency_ordered_and_inner_values_precede_outer_values() {
    let contract = tautology_contract("vector::nested", 64);
    let lowering = lower("x86_64-unknown-linux-gnu", 64, "nested", &contract);
    let vir = member(&lowering.raw_lowering, "vir");
    let declarations = member(first_unit(vir), "type_decls").as_array().unwrap();
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| member(declaration, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["vector::Point", "vector::Envelope"]
    );
    assert_struct_type(
        member(
            &member(&declarations[1], "fields").as_array().unwrap()[0],
            "type",
        ),
        "vector::Point",
    );
    let aggregates = instructions(vir)
        .filter(|instruction| member(instruction, "kind").as_str() == Some("MakeStruct"))
        .collect::<Vec<_>>();
    assert_eq!(
        aggregates
            .iter()
            .map(|instruction| member(instruction, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["t0", "t1"]
    );
    let outer_fields = member(aggregates[1], "fields").as_array().unwrap();
    assert_eq!(
        member(member(&outer_fields[0], "value"), "var").as_str(),
        Some("t0")
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "t0"),
        "Point { x, y }"
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "t1"),
        "Envelope {\n        point: Point { x, y },\n        enabled,\n    }"
    );
}

#[test]
fn every_struct_aggregate_and_field_pattern_component_is_fail_closed() {
    let aggregate = StructAggregatePatternVector::pinned();
    assert_eq!(validate_struct_aggregate_pattern(&aggregate), Ok(()));
    let aggregate_mutations: Vec<StructPatternMutation> = vec![
        Box::new(|vector| vector.aggregate_is_adt = false),
        Box::new(|vector| vector.definition_is_local_named_struct = false),
        Box::new(|vector| vector.variant_is_only_variant = false),
        Box::new(|vector| vector.arguments_are_empty = false),
        Box::new(|vector| vector.active_union_field_absent = false),
        Box::new(|vector| vector.destination_matches = false),
        Box::new(|vector| vector.arity_matches = false),
        Box::new(|vector| vector.field_types_match = false),
        Box::new(|vector| vector.within_limit = false),
    ];
    for mutate in aggregate_mutations {
        let mut changed = aggregate.clone();
        mutate(&mut changed);
        assert_eq!(
            validate_struct_aggregate_pattern(&changed)
                .expect_err("mutated struct aggregate must reject")
                .as_str(),
            "RUST_MIR_RVALUE"
        );
    }

    let field = FieldProjectionPatternVector::pinned();
    assert_eq!(validate_field_projection_pattern(&field), Ok(()));
    let field_mutations: Vec<FieldPatternMutation> = vec![
        Box::new(|vector| vector.projection_is_direct_field = false),
        Box::new(|vector| vector.base_is_local_named_struct = false),
        Box::new(|vector| vector.field_is_declared = false),
        Box::new(|vector| vector.projected_type_matches = false),
        Box::new(|vector| vector.projected_value_is_copy = false),
    ];
    for mutate in field_mutations {
        let mut changed = field.clone();
        mutate(&mut changed);
        assert_eq!(
            validate_field_projection_pattern(&changed)
                .expect_err("mutated field projection must reject")
                .as_str(),
            "RUST_MIR_PROJECTION"
        );
    }
}

#[test]
fn unsupported_struct_forms_moves_mutation_and_limits_reject() {
    let cases = [
        ("pub struct Bad(u8); pub fn bad(value: Bad) -> Bad { value }".to_owned(), "tuple struct"),
        ("pub enum Bad { Value(u8) } pub fn bad(value: Bad) -> Bad { value }".to_owned(), "enum"),
        ("pub union Bad { value: u8 } pub fn bad(value: Bad) -> Bad { value }".to_owned(), "union"),
        ("pub struct Point { pub x: u8, pub y: u8 } pub fn bad(base: Point) -> Point { Point { x: 1, ..base } }".to_owned(), "update"),
        ("pub struct Point { pub x: u8, pub y: u8 } pub fn bad() -> Point { Point { x: 1 } }".to_owned(), "missing field"),
        ("pub struct Point { pub x: u8, pub y: u8 } pub fn bad() -> Point { Point { x: 1, x: 2, y: 3 } }".to_owned(), "duplicate field"),
        ("pub struct Point { pub x: u8, pub y: u8 } pub fn bad(mut point: Point) -> Point { point.x = 1; point }".to_owned(), "field mutation"),
        ("pub struct Inner { pub value: u8 } pub struct Outer { pub inner: Inner, pub other: u8 } pub fn bad(value: Outer) -> Inner { value.inner }".to_owned(), "partial move"),
        ("pub struct Point { pub x: u8 } pub fn bad(point: Point) -> u8 { let moved = point; point.x }".to_owned(), "use after move"),
        ("pub struct Bad { pub pair: (u8, u8) } pub fn bad(value: Bad) -> Bad { value }".to_owned(), "tuple field"),
        ("pub struct Point { pub x: u8 } pub fn bad(point: &Point) -> u8 { point.x }".to_owned(), "dereference"),
        ("pub fn bad(values: [u8; 3]) -> u8 { let [first, ..] = values; first }".to_owned(), "subslice pattern"),
        (many_fields_source(), "field limit"),
        (deep_struct_source(), "depth limit"),
    ];
    for (source, label) in cases {
        let contract = tautology_contract("vector::bad", 64);
        assert!(
            rustc_harness::lower(
                source.as_bytes(),
                "vector::bad",
                &[("contracts/bad.json", &contract)]
            )
            .is_err(),
            "{label} must reject"
        );
    }
}

fn many_fields_source() -> String {
    let fields = (0..65)
        .map(|index| format!("pub f{index}: u8"))
        .collect::<Vec<_>>()
        .join(",");
    format!("pub struct TooMany {{ {fields} }} pub fn bad(value: TooMany) -> TooMany {{ value }}")
}

fn deep_struct_source() -> String {
    let mut source = "pub struct S00 { pub value: u8 }".to_owned();
    for depth in 1..=17 {
        source.push_str(&format!(
            " pub struct S{depth:02} {{ pub inner: S{:02} }}",
            depth - 1
        ));
    }
    source.push_str(" pub fn bad(value: S17) -> S17 { value }");
    source
}

fn lower(
    target: &str,
    width: u8,
    name: &str,
    contract: &[u8],
) -> rustc_driver_adapter::MirLowering {
    rustc_harness::lower_for_target(
        SOURCE,
        &format!("vector::{name}"),
        &[("contracts/structs.json", contract)],
        target,
        width,
    )
    .unwrap_or_else(|error| panic!("lower {name} for {target}: {error:?}"))
}

fn tautology_contract(function: &str, pointer_width: u8) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":{pointer_width},\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn struct_result_contract(function: &str, pointer_width: u8, parameter: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":{pointer_width},\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"parameter\":\"{parameter}\"}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn first_unit(vir: &JsonValue) -> &JsonValue {
    &member(vir, "units").as_array().unwrap()[0]
}

fn first_function(vir: &JsonValue) -> &JsonValue {
    &member(first_unit(vir), "functions").as_array().unwrap()[0]
}

fn instructions(vir: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    member(first_function(vir), "blocks")
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
}

fn features(vir: &JsonValue) -> Vec<&str> {
    member(first_function(vir), "features_used")
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature.as_str().unwrap())
        .collect()
}

fn field_names(declaration: &JsonValue) -> Vec<&str> {
    member(declaration, "fields")
        .as_array()
        .unwrap()
        .iter()
        .map(|field| member(field, "name").as_str().unwrap())
        .collect()
}

fn assert_struct_type(value: &JsonValue, id: &str) {
    assert_eq!(member(value, "kind").as_str(), Some("struct"));
    assert_eq!(member(value, "id").as_str(), Some(id));
}

fn instruction_source(source_map: &JsonValue, id: &str) -> String {
    let entry = member(source_map, "entries")
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            let reference = member(entry, "reference");
            member(reference, "kind").as_str() == Some("instruction")
                && member(reference, "instruction").as_str() == Some(id)
        })
        .expect("instruction source-map entry");
    let origin = member(entry, "origin");
    let start = usize::try_from(member(origin, "start").integer().unwrap()).unwrap();
    let end = usize::try_from(member(origin, "end").integer().unwrap()).unwrap();
    std::str::from_utf8(&SOURCE[start..end]).unwrap().to_owned()
}

fn member<'a>(value: &'a JsonValue, name: &str) -> &'a JsonValue {
    value
        .as_object()
        .and_then(|object| object.get(name))
        .unwrap_or_else(|| panic!("missing {name}"))
}
