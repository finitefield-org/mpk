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

use std::collections::BTreeSet;

use rust2vir_internal::json::{self, JsonValue};
use rustc_driver_adapter::MirLowering;

const SOURCE: &[u8] = include_bytes!("../testdata/static-calls/wp_graph.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/static-calls/expected-vir.json");
const REORDERED_SOURCE: &[u8] = br#"
pub fn z_repeated(value: u8) -> u8 {
    value / c_right(c_right(value))
}

pub fn z_diamond(value: u8, enabled: bool) -> u8 {
    if enabled {
        b_left(value)
    } else {
        c_right(value)
    }
}

fn c_right(value: u8) -> u8 {
    a_leaf(value)
}

fn b_left(value: u8) -> u8 {
    a_leaf(value)
}

fn a_leaf(value: u8) -> u8 {
    value
}
"#;

#[test]
fn diamond_calls_lower_callee_first_with_exact_reachable_edges() {
    let lowering = lower_diamond(SOURCE, false);
    let vir = member(&lowering.raw_lowering, "vir");
    assert_eq!(json::canonical(vir).unwrap(), expected_vir_transport());
    assert_eq!(
        functions(vir)
            .iter()
            .map(|function| member(function, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "vector::a_leaf",
            "vector::b_left",
            "vector::c_right",
            "vector::z_diamond",
        ]
    );
    assert_eq!(
        call_targets(function(vir, "vector::a_leaf")),
        BTreeSet::new()
    );
    assert_eq!(
        call_targets(function(vir, "vector::b_left")),
        BTreeSet::from(["vector::a_leaf"])
    );
    assert_eq!(
        call_targets(function(vir, "vector::c_right")),
        BTreeSet::from(["vector::a_leaf"])
    );
    assert_eq!(
        call_targets(function(vir, "vector::z_diamond")),
        BTreeSet::from(["vector::b_left", "vector::c_right"])
    );

    let selected = function(vir, "vector::z_diamond");
    assert_eq!(features(selected), vec!["branch", "call_static"]);
    let calls = call_instructions(selected).collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert_eq!(member(calls[0], "id").as_str(), Some("t0"));
    assert_eq!(member(calls[1], "id").as_str(), Some("t1"));
    assert_eq!(
        member(calls[0], "function").as_str(),
        Some("vector::c_right")
    );
    assert_eq!(
        member(calls[1], "function").as_str(),
        Some("vector::b_left")
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::z_diamond", "t0", SOURCE),
        "c_right(value)"
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::z_diamond", "t1", SOURCE),
        "b_left(value)"
    );
    assert_call_hashes_repeat_exact_callee_contracts(vir);
}

#[test]
fn repeated_calls_remain_distinct_while_the_reachable_edge_is_unique() {
    let lowering = lower_repeated(SOURCE);
    let vir = member(&lowering.raw_lowering, "vir");
    assert_eq!(
        functions(vir)
            .iter()
            .map(|function| member(function, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["vector::a_leaf", "vector::c_right", "vector::z_repeated"]
    );
    let selected = function(vir, "vector::z_repeated");
    let calls = call_instructions(selected).collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert_eq!(call_targets(selected), BTreeSet::from(["vector::c_right"]));
    assert_eq!(member(calls[0], "id").as_str(), Some("t0"));
    assert_eq!(member(calls[1], "id").as_str(), Some("t1"));
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::z_repeated", "t0", SOURCE),
        "c_right(value)"
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::z_repeated", "t1", SOURCE),
        "c_right(c_right(value))"
    );
    assert_eq!(
        member(calls[0], "contract_hash"),
        member(calls[1], "contract_hash")
    );
    let instructions = instructions(selected).collect::<Vec<_>>();
    assert_eq!(instructions.len(), 3);
    assert_eq!(member(instructions[2], "kind").as_str(), Some("BinOp"));
    assert_eq!(member(instructions[2], "op").as_str(), Some("bv_udiv"));
    let safety = member(instructions[2], "safety_checks").as_array().unwrap();
    assert_eq!(safety.len(), 1);
    assert_eq!(member(&safety[0], "kind").as_str(), Some("divisor_nonzero"));
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::z_repeated", "t2", SOURCE),
        "value / c_right(c_right(value))"
    );
    assert_call_hashes_repeat_exact_callee_contracts(vir);
}

#[test]
fn clean_runs_and_reordered_source_functions_and_contracts_preserve_vir_bytes() {
    let first = lower_diamond(SOURCE, false);
    let second = lower_diamond(SOURCE, false);
    let reordered = lower_diamond(REORDERED_SOURCE, true);
    let first_bytes = json::canonical(member(&first.raw_lowering, "vir")).unwrap();
    assert_eq!(
        first_bytes,
        json::canonical(member(&second.raw_lowering, "vir")).unwrap()
    );
    assert_eq!(
        first_bytes,
        json::canonical(member(&reordered.raw_lowering, "vir")).unwrap()
    );
    assert_eq!(first_bytes, expected_vir_transport());
}

fn expected_vir_transport() -> &'static [u8] {
    EXPECTED_VIR
        .strip_suffix(b"\n")
        .expect("source-controlled expected VIR ends with one newline")
}

fn lower_diamond(source: &[u8], reverse_contracts: bool) -> MirLowering {
    let mut contracts = vec![
        ("contracts/leaf.json", identity_contract("vector::a_leaf")),
        ("contracts/left.json", identity_contract("vector::b_left")),
        ("contracts/right.json", identity_contract("vector::c_right")),
        (
            "contracts/selected.json",
            identity_contract("vector::z_diamond"),
        ),
    ];
    if reverse_contracts {
        contracts.reverse();
    }
    lower(source, "vector::z_diamond", &contracts)
}

fn lower_repeated(source: &[u8]) -> MirLowering {
    lower(
        source,
        "vector::z_repeated",
        &[
            (
                "contracts/repeated.json",
                identity_contract("vector::z_repeated"),
            ),
            ("contracts/right.json", identity_contract("vector::c_right")),
            ("contracts/leaf.json", identity_contract("vector::a_leaf")),
        ],
    )
}

fn lower(source: &[u8], selected: &str, contracts: &[(&str, Vec<u8>)]) -> MirLowering {
    let references = contracts
        .iter()
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<Vec<_>>();
    rustc_harness::lower(source, selected, &references)
        .unwrap_or_else(|error| panic!("{selected} lowering failed: {error:?}"))
}

fn identity_contract(function: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":64,\"function\":\"{function}\",\"requires\":[{{\"op\":\"eq\",\"args\":[{{\"parameter\":\"value\"}},{{\"parameter\":\"value\"}}]}}],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"parameter\":\"value\"}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn assert_call_hashes_repeat_exact_callee_contracts(vir: &JsonValue) {
    for caller in functions(vir) {
        for call in call_instructions(caller) {
            let callee = function(vir, member(call, "function").as_str().unwrap());
            assert_eq!(
                member(call, "contract_hash"),
                member(member(callee, "contracts"), "contract_hash")
            );
        }
    }
}

fn call_targets(function: &JsonValue) -> BTreeSet<&str> {
    call_instructions(function)
        .map(|call| member(call, "function").as_str().unwrap())
        .collect()
}

fn call_instructions(function: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    instructions(function)
        .filter(|instruction| member(instruction, "kind").as_str() == Some("CallStatic"))
}

fn instructions(function: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    member(function, "blocks")
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
}

fn features(function: &JsonValue) -> Vec<&str> {
    member(function, "features_used")
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature.as_str().unwrap())
        .collect()
}

fn functions(vir: &JsonValue) -> &[JsonValue] {
    member(&member(vir, "units").as_array().unwrap()[0], "functions")
        .as_array()
        .unwrap()
}

fn function<'a>(vir: &'a JsonValue, id: &str) -> &'a JsonValue {
    functions(vir)
        .iter()
        .find(|function| member(function, "id").as_str() == Some(id))
        .unwrap_or_else(|| panic!("missing function {id}"))
}

fn instruction_source(
    source_map: &JsonValue,
    function_id: &str,
    instruction_id: &str,
    source: &'static [u8],
) -> &'static str {
    let entry = member(source_map, "entries")
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            let reference = member(entry, "reference");
            member(reference, "kind").as_str() == Some("instruction")
                && member(reference, "function_id").as_str() == Some(function_id)
                && member(reference, "instruction").as_str() == Some(instruction_id)
        })
        .expect("instruction source entry");
    let origin = member(entry, "origin");
    let start = usize::try_from(member(origin, "start").integer().unwrap()).unwrap();
    let end = usize::try_from(member(origin, "end").integer().unwrap()).unwrap();
    std::str::from_utf8(&source[start..end]).unwrap()
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}
