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

use rust2vir_internal::call_closure::{canonical_callee_first_order, CallClosureError};
use rust2vir_internal::contract::ContractCode;
use rust2vir_internal::json::{self, JsonValue};
use rustc_driver_adapter::mir_call::{validate_direct_call_pattern, DirectCallPatternVector};
use rustc_driver_adapter::{HirCheckCode, RustcDriverError};

const SOURCE: &[u8] = include_bytes!("../testdata/static-calls/checked.rs");
type PatternMutation = Box<dyn Fn(&mut DirectCallPatternVector)>;

#[test]
fn private_cross_module_chain_is_contract_bound_and_callee_first() {
    let leaf = tautology_contract("vector::helpers::z_private_leaf", 64);
    let helper = tautology_contract("vector::helpers::a_public_helper", 64);
    let selected = tautology_contract("vector::selected", 64);
    let first = rustc_harness::lower(
        SOURCE,
        "vector::selected",
        &[
            ("contracts/selected.json", &selected),
            ("contracts/leaf.json", &leaf),
            ("contracts/helper.json", &helper),
        ],
    )
    .expect("lower direct static-call chain");
    let reordered = rustc_harness::lower(
        SOURCE,
        "vector::selected",
        &[
            ("contracts/helper.json", &helper),
            ("contracts/leaf.json", &leaf),
            ("contracts/selected.json", &selected),
        ],
    )
    .expect("contract option order must be irrelevant");
    assert_eq!(first, reordered);
    assert_eq!(
        json::canonical(&first.raw_lowering).unwrap(),
        json::canonical(&reordered.raw_lowering).unwrap()
    );
    assert_eq!(
        json::canonical(&first.raw_source_map).unwrap(),
        json::canonical(&reordered.raw_source_map).unwrap()
    );

    let vir = member(&first.raw_lowering, "vir");
    assert_eq!(
        functions(vir)
            .iter()
            .map(|function| member(function, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "vector::helpers::z_private_leaf",
            "vector::helpers::a_public_helper",
            "vector::selected",
        ]
    );
    assert!(
        call_instructions(function(vir, "vector::helpers::z_private_leaf"))
            .next()
            .is_none()
    );
    assert_call(
        vir,
        "vector::helpers::a_public_helper",
        "vector::helpers::z_private_leaf",
    );
    assert_call(vir, "vector::selected", "vector::helpers::a_public_helper");
    assert_eq!(
        instruction_source(
            &first.raw_source_map,
            "vector::helpers::a_public_helper",
            "t0"
        ),
        "z_private_leaf(value)"
    );
    assert_eq!(
        instruction_source(&first.raw_source_map, "vector::selected", "t0"),
        "helpers::a_public_helper(value)"
    );
    let canonical = json::canonical(&first.raw_lowering).expect("canonical lowering");
    assert!(!std::str::from_utf8(&canonical).unwrap().contains("/root/"));
}

#[test]
fn source_dead_callee_is_emitted_without_an_invented_reachable_edge() {
    let dead = tautology_contract("vector::a_source_dead", 64);
    let helper = tautology_contract("vector::dead_helpers::z_source_dead", 64);
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::a_source_dead",
        &[
            ("contracts/helper.json", &helper),
            ("contracts/dead.json", &dead),
        ],
    )
    .expect("lower conservative HIR closure with source-dead call");
    let vir = member(&lowering.raw_lowering, "vir");
    assert_eq!(
        functions(vir)
            .iter()
            .map(|function| member(function, "id").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "vector::a_source_dead",
            "vector::dead_helpers::z_source_dead"
        ]
    );
    assert!(functions(vir)
        .iter()
        .all(|function| call_instructions(function).next().is_none()));
}

#[test]
fn call_result_assigned_to_a_source_local_keeps_canonical_ids() {
    let leaf = tautology_contract("vector::helpers::z_private_leaf", 64);
    let helper = tautology_contract("vector::helpers::a_public_helper", 64);
    let selected = tautology_contract("vector::call_through_local", 64);
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::call_through_local",
        &[
            ("contracts/selected.json", &selected),
            ("contracts/helper.json", &helper),
            ("contracts/leaf.json", &leaf),
        ],
    )
    .expect("lower call result assigned to source local");
    let vir = member(&lowering.raw_lowering, "vir");
    let caller = function(vir, "vector::call_through_local");
    let instructions = member(caller, "blocks")
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        instructions
            .iter()
            .map(|instruction| member(instruction, "kind").as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["CallStatic", "Copy"]
    );
    assert_eq!(member(instructions[0], "id").as_str(), Some("t0"));
    assert_eq!(member(instructions[1], "id").as_str(), Some("t1"));
    assert_eq!(member(instructions[1], "target").as_str(), Some("local0"));
    assert_eq!(
        member(member(instructions[1], "value"), "var").as_str(),
        Some("t0")
    );
    assert_eq!(features(caller), vec!["call_static", "mutable_local"]);
    let returned = member(
        member(
            member(caller, "blocks").as_array().unwrap().last().unwrap(),
            "terminator",
        ),
        "values",
    )
    .as_array()
    .unwrap();
    assert_eq!(member(&returned[0], "var").as_str(), Some("local0"));
}

#[test]
fn repeated_same_callee_calls_keep_distinct_hir_source_origins() {
    let leaf = tautology_contract("vector::helpers::z_private_leaf", 64);
    let helper = tautology_contract("vector::helpers::a_public_helper", 64);
    let selected = tautology_contract("vector::repeated_call", 64);
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::repeated_call",
        &[
            ("contracts/selected.json", &selected),
            ("contracts/helper.json", &helper),
            ("contracts/leaf.json", &leaf),
        ],
    )
    .expect("lower repeated calls to one callee");
    let vir = member(&lowering.raw_lowering, "vir");
    let caller = function(vir, "vector::repeated_call");
    let calls = call_instructions(caller).collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert_eq!(member(calls[0], "id").as_str(), Some("t0"));
    assert_eq!(member(calls[1], "id").as_str(), Some("t1"));
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::repeated_call", "t0"),
        "helpers::a_public_helper(value)"
    );
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::repeated_call", "t1"),
        "helpers::a_public_helper(helpers::a_public_helper(value))"
    );
    assert_eq!(
        member(calls[0], "contract_hash"),
        member(calls[1], "contract_hash")
    );
}

#[test]
fn call_in_short_circuit_rhs_remains_path_conditional() {
    let helper = tautology_contract("vector::bool_identity", 64);
    let selected = tautology_contract("vector::short_circuit", 64);
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::short_circuit",
        &[
            ("contracts/selected.json", &selected),
            ("contracts/helper.json", &helper),
        ],
    )
    .expect("lower short-circuit call");
    let vir = member(&lowering.raw_lowering, "vir");
    let caller = function(vir, "vector::short_circuit");
    assert_eq!(features(caller), vec!["branch", "call_static"]);
    let blocks = member(caller, "blocks").as_array().unwrap();
    assert_eq!(
        member(member(&blocks[0], "terminator"), "kind").as_str(),
        Some("Branch")
    );
    let call_blocks = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| {
            member(block, "instructions")
                .as_array()
                .unwrap()
                .iter()
                .any(|instruction| member(instruction, "kind").as_str() == Some("CallStatic"))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(call_blocks.len(), 1);
    assert_ne!(call_blocks[0], 0);
    let call_id = member(
        call_instructions(caller)
            .next()
            .expect("short-circuit call"),
        "id",
    )
    .as_str()
    .unwrap();
    assert_eq!(
        instruction_source(&lowering.raw_source_map, "vector::short_circuit", call_id),
        "bool_identity(enabled)"
    );
}

#[test]
fn target_sized_call_signatures_bind_target_specific_contract_hashes() {
    let mut hashes = Vec::new();
    for (target, width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let helper = tautology_contract("vector::usize_identity", width);
        let selected = tautology_contract("vector::usize_call", width);
        let lowering = rustc_harness::lower_for_target(
            SOURCE,
            "vector::usize_call",
            &[
                ("contracts/selected.json", &selected),
                ("contracts/helper.json", &helper),
            ],
            target,
            width,
        )
        .unwrap_or_else(|error| panic!("lower target-sized call for {target}: {error:?}"));
        let vir = member(&lowering.raw_lowering, "vir");
        let call = call_instructions(function(vir, "vector::usize_call"))
            .next()
            .expect("target-sized CallStatic");
        assert_eq!(member(member(call, "type"), "kind").as_str(), Some("bv"));
        assert_eq!(
            member(member(call, "type"), "width").integer(),
            Some(i64::from(width))
        );
        assert_eq!(
            member(member(call, "type"), "signed").as_bool(),
            Some(false)
        );
        let repeated = member(call, "contract_hash").as_str().unwrap();
        assert_eq!(
            repeated,
            member(
                member(function(vir, "vector::usize_identity"), "contracts"),
                "contract_hash",
            )
            .as_str()
            .unwrap()
        );
        hashes.push(repeated.to_owned());
    }
    assert_ne!(hashes[0], hashes[1]);
}

#[test]
fn direct_call_pattern_and_contract_hash_binding_are_fail_closed() {
    let valid = DirectCallPatternVector::pinned();
    assert_eq!(validate_direct_call_pattern(&valid), Ok(()));
    let mutations: Vec<PatternMutation> = vec![
        Box::new(|vector| vector.call_source_is_normal = false),
        Box::new(|vector| vector.callee_is_constant_fn_def = false),
        Box::new(|vector| vector.callee_is_local_free_function = false),
        Box::new(|vector| vector.generic_arguments_are_empty = false),
        Box::new(|vector| vector.callee_is_in_hir_closure = false),
        Box::new(|vector| vector.callee_differs_from_caller = false),
        Box::new(|vector| vector.hir_call_site_matches = false),
        Box::new(|vector| vector.signature_matches_hir = false),
        Box::new(|vector| vector.argument_modes_are_supported = false),
        Box::new(|vector| vector.argument_types_match = false),
        Box::new(|vector| vector.destination_is_plain = false),
        Box::new(|vector| vector.destination_type_matches = false),
        Box::new(|vector| vector.normal_target_is_present = false),
        Box::new(|vector| vector.caller_contract_matches = false),
        Box::new(|vector| vector.callee_contract_matches = false),
        Box::new(|vector| vector.semantic_context_matches = false),
        Box::new(|vector| vector.unit_identity_matches = false),
    ];
    for mutate in mutations {
        let mut changed = valid.clone();
        mutate(&mut changed);
        assert_eq!(
            validate_direct_call_pattern(&changed)
                .expect_err("mutated direct-call pattern must reject")
                .as_str(),
            "RUST_MIR_CALL"
        );
    }
    let mut contract_hash = valid.clone();
    contract_hash.contract_hash_matches = false;
    assert_eq!(
        validate_direct_call_pattern(&contract_hash)
            .expect_err("mismatched repeated contract hash must reject")
            .as_str(),
        "RUST_SEMANTICS_TYPE"
    );
    let mut unwind = valid;
    unwind.unwind_is_unreachable = false;
    assert_eq!(
        validate_direct_call_pattern(&unwind)
            .expect_err("call unwind must reject")
            .as_str(),
        "RUST_MIR_CLEANUP"
    );
}

#[test]
fn reachable_graph_order_is_reconstructed_and_cycles_reject_again() {
    assert_eq!(
        canonical_callee_first_order([
            ("vector::caller", vec!["vector::z_callee"]),
            ("vector::a_dead", vec![]),
            ("vector::z_callee", vec![]),
        ]),
        Ok(vec![
            "vector::a_dead".to_owned(),
            "vector::z_callee".to_owned(),
            "vector::caller".to_owned(),
        ])
    );
    assert_eq!(
        canonical_callee_first_order([
            ("vector::a", vec!["vector::b"]),
            ("vector::b", vec!["vector::a"]),
        ]),
        Err(CallClosureError::Cycle)
    );
}

#[test]
fn dynamic_external_recursive_and_uncontracted_calls_reject() {
    let rejected_sources: &[(&[u8], &str)] = &[
        (
            b"fn helper(x: u8) -> u8 { x } pub fn selected(call: fn(u8) -> u8, x: u8) -> u8 { call(x) }",
            "dynamic function value",
        ),
        (
            b"pub fn selected(x: u8) -> u8 { core::cmp::min(x, x) }",
            "external function",
        ),
        (
            b"fn a(x: u8) -> u8 { b(x) } fn b(x: u8) -> u8 { a(x) } pub fn selected(x: u8) -> u8 { a(x) }",
            "reachable recursion",
        ),
        (
            b"fn a(x: u8) -> u8 { if false { b(x) } else { x } } fn b(x: u8) -> u8 { a(x) } pub fn selected(x: u8) -> u8 { a(x) }",
            "source-dead recursion",
        ),
    ];
    for (source, label) in rejected_sources {
        assert!(
            rustc_harness::analyze(source, "vector::selected").is_err(),
            "{label} must reject during conservative HIR analysis"
        );
    }

    let source = b"fn helper(x: u8) -> u8 { x } pub fn selected(x: u8) -> u8 { helper(x) }";
    let selected = tautology_contract("vector::selected", 64);
    let error = rustc_harness::lower(
        source,
        "vector::selected",
        &[("contracts/selected.json", &selected)],
    )
    .expect_err("callee contract is mandatory");
    let RustcDriverError::Contract(error) = error else {
        panic!("expected contract error, got {error:?}");
    };
    assert_eq!(error.code, ContractCode::Missing);
}

#[test]
fn recursive_call_errors_keep_the_call_subset_code() {
    let source =
        b"fn recurse(x: u8) -> u8 { recurse(x) } pub fn selected(x: u8) -> u8 { recurse(x) }";
    assert_eq!(
        rustc_harness::analyze(source, "vector::selected"),
        Err(RustcDriverError::Subset(HirCheckCode::Call))
    );
}

fn assert_call(vir: &JsonValue, caller_id: &str, callee_id: &str) {
    let caller = function(vir, caller_id);
    let calls = call_instructions(caller).collect::<Vec<_>>();
    assert_eq!(calls.len(), 1);
    let call = calls[0];
    assert_eq!(member(call, "id").as_str(), Some("t0"));
    assert_eq!(member(call, "function").as_str(), Some(callee_id));
    assert_eq!(
        member(call, "contract_hash"),
        member(
            member(function(vir, callee_id), "contracts"),
            "contract_hash"
        )
    );
    assert!(member(call, "safety_checks").as_array().unwrap().is_empty());
    let arguments = member(call, "args").as_array().unwrap();
    assert_eq!(arguments.len(), 1);
    assert_eq!(member(&arguments[0], "var").as_str(), Some("arg0"));
    assert_eq!(features(caller), vec!["call_static"]);
    let call_block = member(caller, "blocks")
        .as_array()
        .unwrap()
        .iter()
        .find(|block| {
            member(block, "instructions")
                .as_array()
                .unwrap()
                .iter()
                .any(|instruction| member(instruction, "kind").as_str() == Some("CallStatic"))
        })
        .unwrap();
    assert_eq!(
        member(member(call_block, "terminator"), "kind").as_str(),
        Some("Jump")
    );
}

fn tautology_contract(function: &str, pointer_width: u8) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":{pointer_width},\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
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

fn call_instructions(function: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    member(function, "blocks")
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
        .filter(|instruction| member(instruction, "kind").as_str() == Some("CallStatic"))
}

fn features(function: &JsonValue) -> Vec<&str> {
    member(function, "features_used")
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature.as_str().unwrap())
        .collect()
}

fn instruction_source(
    source_map: &JsonValue,
    function_id: &str,
    instruction_id: &str,
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
    std::str::from_utf8(&SOURCE[start..end]).unwrap()
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}
