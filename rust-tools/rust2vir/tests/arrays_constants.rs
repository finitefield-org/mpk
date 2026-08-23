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

use rust2vir_internal::json::JsonValue;
use rustc_driver_adapter::mir_aggregate::{
    validate_array_aggregate_pattern, ArrayAggregatePatternVector,
};

const SOURCE: &[u8] = include_bytes!("../testdata/arrays/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/arrays/expected-vir.json");
type PatternMutation = Box<dyn Fn(&mut ArrayAggregatePatternVector)>;

#[test]
fn referenced_constants_and_explicit_array_are_canonical_for_both_targets() {
    let mut hashes = Vec::new();
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
        hashes.push(member(vir, "vir_hash").as_str().unwrap().to_owned());
        let constants = member(first_unit(vir), "const_decls")
            .as_array()
            .expect("constant declarations");
        assert_eq!(
            constants
                .iter()
                .map(|constant| member(constant, "id").as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["vector::FIRST", "vector::LENGTH"]
        );
        assert_eq!(member(&constants[0], "name").as_str(), Some("FIRST"));
        assert_integer_literal(member(&constants[0], "value"), "7", 8, false);
        assert_eq!(member(&constants[1], "name").as_str(), Some("LENGTH"));
        assert_bv_type(member(&constants[1], "type"), width, false);
        assert_integer_literal(member(&constants[1], "value"), "3", width, false);

        let instruction = instructions(vir)
            .find(|instruction| member(instruction, "kind").as_str() == Some("MakeArray"))
            .expect("MakeArray instruction");
        assert_eq!(member(instruction, "id").as_str(), Some("t0"));
        assert_array_type(member(instruction, "type"), 3, 8, false);
        let elements = member(instruction, "elements").as_array().unwrap();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            member(&elements[0], "const").as_str(),
            Some("vector::FIRST")
        );
        assert_eq!(member(&elements[1], "var").as_str(), Some("arg0"));
        assert_integer_literal(&elements[2], "9", 8, false);
        assert_eq!(
            member(first_function(vir), "features_used")
                .as_array()
                .unwrap()
                .iter()
                .map(|feature| feature.as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["array", "constant_decl"]
        );
        assert_eq!(
            instruction_source(&first.raw_source_map, "t0"),
            "[FIRST, middle, 9]"
        );

        let canonical = rust2vir_internal::json::canonical(&first.raw_lowering).unwrap();
        let text = std::str::from_utf8(&canonical).unwrap();
        assert!(!text.contains("UNUSED"));
        assert!(!text.contains("size_of"));
        assert!(!text.contains("/root/"));
    }
    assert_ne!(hashes[0], hashes[1]);
    let expected = rust2vir_internal::json::parse(EXPECTED_VIR, EXPECTED_VIR.len())
        .expect("expected dual-target constant/array VIR fixture");
    assert_eq!(JsonValue::Array(modules), expected);
}

#[test]
fn construction_index_copy_and_contract_array_equality_integrate() {
    for (target, width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let contract = tautology_contract("vector::read_constructed", width);
        let lowering = lower(target, width, "read_constructed", &contract);
        let kinds = instructions(member(&lowering.raw_lowering, "vir"))
            .map(|instruction| member(instruction, "kind").as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"MakeArray"));
        assert!(kinds.contains(&"Index"));

        let contract = tautology_contract("vector::local_length", width);
        let lowering = lower(target, width, "local_length", &contract);
        let constants = member(
            first_unit(member(&lowering.raw_lowering, "vir")),
            "const_decls",
        )
        .as_array()
        .unwrap();
        assert_eq!(constants.len(), 1);
        assert_eq!(
            member(&constants[0], "id").as_str(),
            Some("vector::LOCAL_LENGTH")
        );
        assert_bv_type(member(&constants[0], "type"), width, false);

        let contract = array_result_contract("vector::copy_array", width, "values");
        let first = lower(target, width, "copy_array", &contract);
        let second = lower(target, width, "copy_array", &contract);
        assert_eq!(first, second);
        let function = first_function(member(&first.raw_lowering, "vir"));
        let ensures = member(member(function, "contracts"), "ensures")
            .as_array()
            .unwrap();
        assert_eq!(member(&ensures[0], "op").as_str(), Some("eq"));
        assert_eq!(
            member(member(&ensures[0], "lhs"), "result").integer(),
            Some(0)
        );
        assert_eq!(
            member(member(&ensures[0], "rhs"), "var").as_str(),
            Some("arg0")
        );
    }
}

#[test]
fn nested_arrays_emit_inner_then_outer_aggregates_with_stable_ids() {
    let contract = tautology_contract("vector::nested", 64);
    let lowering = lower("x86_64-unknown-linux-gnu", 64, "nested", &contract);
    let arrays = instructions(member(&lowering.raw_lowering, "vir"))
        .filter(|instruction| member(instruction, "kind").as_str() == Some("MakeArray"))
        .collect::<Vec<_>>();
    assert_eq!(arrays.len(), 3);
    assert_eq!(
        arrays
            .iter()
            .map(|instruction| member(instruction, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["t0", "t1", "t2"]
    );
    let outer = member(arrays[2], "elements").as_array().unwrap();
    assert_eq!(member(&outer[0], "var").as_str(), Some("t0"));
    assert_eq!(member(&outer[1], "var").as_str(), Some("t1"));
    assert_eq!(instruction_source(&lowering.raw_source_map, "t0"), "[1, 2]");
    assert_eq!(instruction_source(&lowering.raw_source_map, "t1"), "[3, 4]");
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "t2"),
        "[[1, 2], [3, 4]]"
    );
}

#[test]
fn boolean_constant_reference_is_declared_without_literal_reinference() {
    let contract = tautology_contract("vector::enabled", 64);
    let lowering = lower("x86_64-unknown-linux-gnu", 64, "enabled", &contract);
    let vir = member(&lowering.raw_lowering, "vir");
    let constants = member(first_unit(vir), "const_decls").as_array().unwrap();
    assert_eq!(constants.len(), 1);
    assert_eq!(
        member(&constants[0], "id").as_str(),
        Some("vector::ENABLED")
    );
    assert_eq!(
        member(member(&constants[0], "value"), "bool").as_bool(),
        Some(true)
    );
    let returned = member(
        member(
            member(first_function(vir), "blocks")
                .as_array()
                .unwrap()
                .last()
                .unwrap(),
            "terminator",
        ),
        "values",
    )
    .as_array()
    .unwrap();
    assert_eq!(
        member(&returned[0], "const").as_str(),
        Some("vector::ENABLED")
    );

    let contract = tautology_contract("vector::negative", 64);
    let lowering = lower("x86_64-unknown-linux-gnu", 64, "negative", &contract);
    let vir = member(&lowering.raw_lowering, "vir");
    let constants = member(first_unit(vir), "const_decls").as_array().unwrap();
    assert_eq!(constants.len(), 1);
    assert_eq!(
        member(&constants[0], "id").as_str(),
        Some("vector::NEGATIVE")
    );
    assert_integer_literal(member(&constants[0], "value"), "-7", 8, true);
}

#[test]
fn every_array_aggregate_pattern_component_is_fail_closed() {
    let valid = ArrayAggregatePatternVector::pinned();
    assert_eq!(validate_array_aggregate_pattern(&valid), Ok(()));
    let mutations: Vec<PatternMutation> = vec![
        Box::new(|vector| vector.aggregate_is_array = false),
        Box::new(|vector| vector.destination_is_array = false),
        Box::new(|vector| vector.arity_matches = false),
        Box::new(|vector| vector.element_types_match = false),
        Box::new(|vector| vector.within_limit = false),
    ];
    for mutate in mutations {
        let mut changed = valid.clone();
        mutate(&mut changed);
        assert_eq!(
            validate_array_aggregate_pattern(&changed)
                .expect_err("mutated aggregate pattern must reject")
                .as_str(),
            "RUST_MIR_RVALUE"
        );
    }
}

#[test]
fn expressions_statics_repeat_arrays_wrong_lengths_and_mutation_reject() {
    let cases: &[(&[u8], &str)] = &[
        (b"pub const BAD: u8 = 1 + 2; pub fn bad() -> u8 { BAD }\n", "const expression"),
        (b"pub static BAD: u8 = 1; pub fn bad() -> u8 { BAD }\n", "static"),
        (b"pub fn bad(value: u8) -> [u8; 3] { [value; 3] }\n", "repeat array"),
        (b"pub const N: u8 = 3; pub fn bad() -> [u8; N] { [1, 2, 3] }\n", "non-usize length"),
        (b"pub const N: usize = core::mem::size_of::<u32>(); pub fn bad() -> [u8; N] { [1, 2, 3, 4] }\n", "layout constant"),
        (b"pub fn bad(mut values: [u8; 2]) -> [u8; 2] { values[0] = 9; values }\n", "element mutation"),
    ];
    for (source, label) in cases {
        let contract = tautology_contract("vector::bad", 64);
        assert!(
            rustc_harness::lower(source, "vector::bad", &[("contracts/bad.json", &contract)])
                .is_err(),
            "{label} must reject"
        );
    }
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
        &[("contracts/arrays.json", contract)],
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

fn array_result_contract(function: &str, pointer_width: u8, parameter: &str) -> Vec<u8> {
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

fn assert_bv_type(value: &JsonValue, width: u8, signed: bool) {
    assert_eq!(member(value, "kind").as_str(), Some("bv"));
    assert_eq!(member(value, "width").integer(), Some(i64::from(width)));
    assert_eq!(member(value, "signed").as_bool(), Some(signed));
}

fn assert_array_type(value: &JsonValue, length: i64, width: u8, signed: bool) {
    assert_eq!(member(value, "kind").as_str(), Some("array"));
    assert_eq!(member(value, "length").integer(), Some(length));
    assert_bv_type(member(value, "element"), width, signed);
}

fn assert_integer_literal(value: &JsonValue, decimal: &str, width: u8, signed: bool) {
    let integer = member(value, "int");
    assert_eq!(member(integer, "value").as_str(), Some(decimal));
    assert_eq!(member(integer, "width").integer(), Some(i64::from(width)));
    assert_eq!(member(integer, "signed").as_bool(), Some(signed));
}

fn instruction_source(source_map: &JsonValue, instruction_id: &str) -> &'static str {
    let entry = member(source_map, "entries")
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            let reference = member(entry, "reference");
            member(reference, "kind").as_str() == Some("instruction")
                && member(reference, "instruction").as_str() == Some(instruction_id)
        })
        .expect("instruction source entry");
    let origin = member(entry, "origin");
    let start = usize::try_from(member(origin, "start").integer().unwrap()).unwrap();
    let end = usize::try_from(member(origin, "end").integer().unwrap()).unwrap();
    std::str::from_utf8(&SOURCE[start..end]).unwrap()
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}
