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

const DRIVER_VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");
const IDENTITY_CONTRACT: &[u8] = include_bytes!("../testdata/contracts/identity.json");

#[test]
fn frozen_identity_lowers_to_the_exact_driver_vector() {
    let source = b"pub fn identity(x: i8) -> i8 { x }\n";
    let lowering = rustc_harness::lower(
        source,
        "vector::identity",
        &[("contracts/identity.json", IDENTITY_CONTRACT)],
    )
    .expect("lower identity MIR");
    let vector = json::parse(DRIVER_VECTOR, DRIVER_VECTOR.len()).expect("driver vector");
    let expected = &vector.as_object().expect("vector object")["valid_lowered"]
        .as_object()
        .expect("valid lowered fixture")["value"];
    let expected = expected.as_object().expect("lowered output object");
    assert_eq!(lowering.raw_lowering, expected["raw_lowering"]);
    assert_eq!(lowering.raw_source_map, expected["raw_source_map"]);
}

#[test]
fn constants_updates_comparisons_branches_and_early_returns_are_canonical() {
    let source = br#"pub fn select(flag: bool, x: i8, y: i8) -> i8 {
    let mut value = x;
    if flag {
        value = y;
    }
    if value < x {
        return x;
    }
    value
}
"#;
    let contract = tautology_contract("vector::select");
    let first = rustc_harness::lower(
        source,
        "vector::select",
        &[("contracts/select.json", &contract)],
    )
    .expect("lower basic control flow");
    let second = rustc_harness::lower(
        source,
        "vector::select",
        &[("contracts/select.json", &contract)],
    )
    .expect("repeat basic control flow lowering");
    assert_eq!(first, second);
    let vir = vir(&first.raw_lowering);
    let function = first_function(vir);
    assert_eq!(
        strings(function, "features_used"),
        vec!["branch", "mutable_local"]
    );
    assert_eq!(ids(function, "params"), vec!["arg0", "arg1", "arg2"]);
    assert_eq!(ids(function, "results"), vec!["result0"]);
    assert_eq!(ids(function, "locals"), vec!["local0"]);
    let blocks = function["blocks"].as_array().expect("blocks");
    assert_eq!(
        blocks
            .iter()
            .map(|block| member(block, "label").as_str().expect("block label"))
            .collect::<Vec<_>>(),
        (0..blocks.len())
            .map(|index| format!("bb{index}"))
            .collect::<Vec<_>>()
    );
    let instruction_ids = blocks
        .iter()
        .flat_map(|block| {
            member(block, "instructions")
                .as_array()
                .expect("instructions")
        })
        .map(|instruction| member(instruction, "id").as_str().expect("instruction id"))
        .collect::<Vec<_>>();
    assert_eq!(
        instruction_ids,
        (0..instruction_ids.len())
            .map(|index| format!("t{index}"))
            .collect::<Vec<_>>()
    );
    assert!(blocks
        .iter()
        .any(|block| { member(member(block, "terminator"), "kind").as_str() == Some("Branch") }));
    assert!(blocks
        .iter()
        .any(|block| { member(member(block, "terminator"), "kind").as_str() == Some("Return") }));

    assert_eq!(
        source_origin_range(&first.raw_source_map, "instruction", "bb0", Some("t0")),
        source_range(source, b"let mut value = x;")
    );
    assert_eq!(
        source_origin_range(&first.raw_source_map, "terminator", "bb0", None),
        source_range(source, b"if flag {\n        value = y;\n    }")
    );
    assert_eq!(
        source_origin_range(&first.raw_source_map, "terminator", "bb3", None),
        source_range(source, b"if value < x {\n        return x;\n    }")
    );

    let encoded = json::canonical(&first.raw_lowering).expect("canonical lowering");
    let text = std::str::from_utf8(&encoded).expect("UTF-8 lowering");
    assert!(!text.contains("_0"));
    assert!(!text.contains("rust2vir-mir-lower"));
    assert!(!text.contains("/root/"));
    let map = first.raw_source_map.as_object().expect("raw source map");
    for entry in map["entries"].as_array().expect("source map entries") {
        let origin = member(entry, "origin").as_object().expect("source origin");
        assert_eq!(origin["normalized_path"].as_str(), Some("src/lib.rs"));
        let start = origin["start"].integer().expect("range start");
        let end = origin["end"].integer().expect("range end");
        assert!(0 <= start && start < end && end <= source.len() as i64);
    }
}

#[test]
fn boolean_not_and_scalar_comparisons_emit_only_supported_operations() {
    let source =
        b"pub fn different(left: u8, right: u8) -> bool { let same = left == right; !same }\n";
    let contract = tautology_contract("vector::different");
    let lowering = rustc_harness::lower(
        source,
        "vector::different",
        &[("contracts/different.json", &contract)],
    )
    .expect("lower Boolean not and equality");
    let function = first_function(vir(&lowering.raw_lowering));
    let operations = function["blocks"]
        .as_array()
        .expect("blocks")
        .iter()
        .flat_map(|block| {
            member(block, "instructions")
                .as_array()
                .expect("instructions")
        })
        .filter_map(|instruction| {
            instruction
                .as_object()
                .and_then(|instruction| instruction.get("op"))
                .and_then(JsonValue::as_str)
        })
        .collect::<Vec<_>>();
    assert_eq!(operations, vec!["eq", "not"]);
}

#[test]
fn final_conditional_return_maps_to_the_complete_result_expression() {
    let source = b"pub fn choose(flag: bool, x: u8, y: u8) -> u8 { if flag { x } else { y } }\n";
    let contract = tautology_contract("vector::choose");
    let lowering = rustc_harness::lower(
        source,
        "vector::choose",
        &[("contracts/choose.json", &contract)],
    )
    .expect("lower final conditional");
    let function = first_function(vir(&lowering.raw_lowering));
    let return_block = function["blocks"]
        .as_array()
        .expect("blocks")
        .iter()
        .find(|block| member(member(block, "terminator"), "kind").as_str() == Some("Return"))
        .and_then(|block| member(block, "label").as_str())
        .expect("return block");
    assert_eq!(
        source_origin_range(&lowering.raw_source_map, "terminator", return_block, None,),
        source_range(source, b"if flag { x } else { y }")
    );
}

#[test]
fn scalar_literal_emits_one_typed_const() {
    let source = "// π\npub fn seven() -> u8 { 7 }\n".as_bytes();
    let contract = tautology_contract("vector::seven");
    let lowering = rustc_harness::lower(
        source,
        "vector::seven",
        &[("contracts/seven.json", &contract)],
    )
    .expect("lower scalar literal");
    let function = first_function(vir(&lowering.raw_lowering));
    let instructions = function["blocks"].as_array().expect("blocks")[0]
        .as_object()
        .expect("block")["instructions"]
        .as_array()
        .expect("instructions");
    assert_eq!(instructions.len(), 1);
    assert_eq!(member(&instructions[0], "id").as_str(), Some("t0"));
    assert_eq!(member(&instructions[0], "kind").as_str(), Some("Const"));
    assert_eq!(
        member(member(&instructions[0], "value"), "int")
            .as_object()
            .expect("integer literal")["value"]
            .as_str(),
        Some("7")
    );
    let source_text = std::str::from_utf8(source).expect("source UTF-8");
    for entry in lowering.raw_source_map.as_object().expect("raw source map")["entries"]
        .as_array()
        .expect("source map entries")
    {
        let origin = member(entry, "origin").as_object().expect("source origin");
        let start = usize::try_from(origin["start"].integer().expect("range start")).unwrap();
        let end = usize::try_from(origin["end"].integer().expect("range end")).unwrap();
        assert!(source_text.is_char_boundary(start));
        assert!(source_text.is_char_boundary(end));
    }
}

#[test]
fn nested_bindings_keep_source_declaration_order() {
    let source = b"pub fn nested(x: u8) -> u8 { let outer = { let inner = x; inner }; outer }\n";
    let contract = tautology_contract("vector::nested");
    let lowering = rustc_harness::lower(
        source,
        "vector::nested",
        &[("contracts/nested.json", &contract)],
    )
    .expect("lower nested bindings");
    let function = first_function(vir(&lowering.raw_lowering));
    assert_eq!(ids(function, "locals"), vec!["local0", "local1"]);
    let copy_targets = function["blocks"]
        .as_array()
        .expect("blocks")
        .iter()
        .flat_map(|block| {
            member(block, "instructions")
                .as_array()
                .expect("instructions")
        })
        .filter(|instruction| member(instruction, "kind").as_str() == Some("Copy"))
        .map(|instruction| member(instruction, "target").as_str().expect("copy target"))
        .collect::<Vec<_>>();
    assert_eq!(copy_targets, vec!["local1", "local0"]);
}

#[test]
fn source_dead_callee_remains_a_standalone_function() {
    let source = br#"pub fn z_helper(x: u8) -> u8 { x }
pub fn a_selected(x: u8) -> u8 {
    if false { z_helper(x) } else { x }
}
"#;
    let helper_contract = tautology_contract("vector::z_helper");
    let selected_contract = tautology_contract("vector::a_selected");
    let lowering = rustc_harness::lower(
        source,
        "vector::a_selected",
        &[
            ("contracts/helper.json", &helper_contract),
            ("contracts/selected.json", &selected_contract),
        ],
    )
    .expect("lower closure with source-dead caller edge");
    let functions = functions(vir(&lowering.raw_lowering));
    assert_eq!(
        functions
            .iter()
            .map(|function| member(function, "id").as_str().expect("function id"))
            .collect::<Vec<_>>(),
        vec!["vector::a_selected", "vector::z_helper"]
    );
    let encoded = json::canonical(&lowering.raw_lowering).expect("canonical lowering");
    assert!(!std::str::from_utf8(&encoded)
        .expect("UTF-8 lowering")
        .contains("CallStatic"));
}

#[test]
fn reachable_call_lowers_as_call_static() {
    let source = b"pub fn helper(x: u8) -> u8 { x }\npub fn selected(x: u8) -> u8 { helper(x) }\n";
    let helper_contract = tautology_contract("vector::helper");
    let selected_contract = tautology_contract("vector::selected");
    let lowering = rustc_harness::lower(
        source,
        "vector::selected",
        &[
            ("contracts/helper.json", &helper_contract),
            ("contracts/selected.json", &selected_contract),
        ],
    )
    .expect("reachable direct call must lower");
    let encoded = json::canonical(&lowering.raw_lowering).expect("canonical lowering");
    assert!(std::str::from_utf8(&encoded)
        .expect("UTF-8 lowering")
        .contains("CallStatic"));
}

#[test]
fn nested_reachable_call_chain_is_callee_first() {
    let source = b"pub fn leaf(x: u8) -> u8 { x }\npub fn helper(x: u8) -> u8 { leaf(x) }\npub fn selected(x: u8) -> u8 { if false { helper(x) } else { x } }\n";
    let leaf_contract = tautology_contract("vector::leaf");
    let helper_contract = tautology_contract("vector::helper");
    let selected_contract = tautology_contract("vector::selected");
    let lowering = rustc_harness::lower(
        source,
        "vector::selected",
        &[
            ("contracts/leaf.json", &leaf_contract),
            ("contracts/helper.json", &helper_contract),
            ("contracts/selected.json", &selected_contract),
        ],
    )
    .expect("lower reachable call chain in a source-dead closure member");
    assert_eq!(
        functions(vir(&lowering.raw_lowering))
            .iter()
            .map(|function| member(function, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["vector::leaf", "vector::helper", "vector::selected"]
    );
}

fn tautology_contract(function: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":64,\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn vir(lowering: &JsonValue) -> &JsonValue {
    member(lowering, "vir")
}

fn first_function(vir: &JsonValue) -> &std::collections::BTreeMap<String, JsonValue> {
    functions(vir)[0].as_object().expect("function")
}

fn functions(vir: &JsonValue) -> &[JsonValue] {
    let unit = &vir.as_object().expect("VIR object")["units"]
        .as_array()
        .expect("units")[0];
    unit.as_object().expect("unit")["functions"]
        .as_array()
        .expect("functions")
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}

fn source_origin_range(
    map: &JsonValue,
    kind: &str,
    block: &str,
    instruction: Option<&str>,
) -> (usize, usize) {
    let entry = map.as_object().expect("raw source map")["entries"]
        .as_array()
        .expect("source map entries")
        .iter()
        .find(|entry| {
            let reference = member(entry, "reference").as_object().expect("reference");
            reference["kind"].as_str() == Some(kind)
                && reference.get("block").and_then(JsonValue::as_str) == Some(block)
                && reference.get("instruction").and_then(JsonValue::as_str) == instruction
        })
        .expect("source map reference");
    let origin = member(entry, "origin").as_object().expect("source origin");
    (
        usize::try_from(origin["start"].integer().expect("origin start")).unwrap(),
        usize::try_from(origin["end"].integer().expect("origin end")).unwrap(),
    )
}

fn source_range(source: &[u8], fragment: &[u8]) -> (usize, usize) {
    let start = source
        .windows(fragment.len())
        .position(|window| window == fragment)
        .expect("source fragment");
    (start, start + fragment.len())
}

fn ids<'a>(object: &'a std::collections::BTreeMap<String, JsonValue>, field: &str) -> Vec<&'a str> {
    object[field]
        .as_array()
        .expect("binding array")
        .iter()
        .map(|binding| member(binding, "id").as_str().expect("binding id"))
        .collect()
}

fn strings<'a>(
    object: &'a std::collections::BTreeMap<String, JsonValue>,
    field: &str,
) -> Vec<&'a str> {
    object[field]
        .as_array()
        .expect("string array")
        .iter()
        .map(|value| value.as_str().expect("string"))
        .collect()
}
