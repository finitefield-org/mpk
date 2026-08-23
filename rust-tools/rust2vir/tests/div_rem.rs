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
use rustc_driver_adapter::mir_arithmetic::{
    validate_div_rem_pattern, DivRemOperation, DivRemPatternVector,
};

const SOURCE: &[u8] = include_bytes!("../testdata/div-rem/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/div-rem/expected-vir.json");
type PatternMutation = Box<dyn Fn(&mut DivRemPatternVector)>;

#[test]
fn emitted_module_matches_the_independently_validated_fixture() {
    let contract = tautology_contract("vector::div_i8");
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::div_i8",
        &[("contracts/div-rem.json", &contract)],
    )
    .expect("lower fixture");
    let expected = json::parse(EXPECTED_VIR, EXPECTED_VIR.len()).expect("expected VIR fixture");
    assert_eq!(*member(&lowering.raw_lowering, "vir"), expected);
}

#[test]
fn signed_unsigned_width_matrix_emits_total_operations_and_exact_checks() {
    for operation in [DivRemOperation::Div, DivRemOperation::Rem] {
        let source_name = operation_name(operation);
        for signedness in ["i", "u"] {
            for width in [8, 16, 32, 64] {
                let name = format!("{source_name}_{signedness}{width}");
                let function = format!("vector::{name}");
                let contract = tautology_contract(&function);
                let lowering = rustc_harness::lower(
                    SOURCE,
                    &function,
                    &[("contracts/div-rem.json", &contract)],
                )
                .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
                let instruction = div_rem_instruction(&lowering.raw_lowering);
                let signed = signedness == "i";
                assert_eq!(member(instruction, "kind").as_str(), Some("BinOp"));
                assert_eq!(
                    member(instruction, "op").as_str(),
                    Some(vir_operation(operation, signed))
                );
                assert_exact_checks(instruction, operation, signed);
                assert_eq!(
                    member(member(instruction, "type"), "width").integer(),
                    Some(width)
                );
                assert_eq!(
                    member(member(instruction, "type"), "signed").as_bool(),
                    Some(signed)
                );
            }
        }
    }
}

#[test]
fn zero_and_minimum_negative_one_boundaries_remain_proof_pending_checks() {
    for (name, operation) in [
        ("min_div_i8", DivRemOperation::Div),
        ("min_rem_i8", DivRemOperation::Rem),
        ("div_neg_one_i8", DivRemOperation::Div),
        ("rem_neg_one_i8", DivRemOperation::Rem),
    ] {
        let function = format!("vector::{name}");
        let contract = tautology_contract(&function);
        let lowering =
            rustc_harness::lower(SOURCE, &function, &[("contracts/div-rem.json", &contract)])
                .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
        assert_exact_checks(div_rem_instruction(&lowering.raw_lowering), operation, true);
    }

    for (name, operation) in [
        ("div_i8", DivRemOperation::Div),
        ("rem_i8", DivRemOperation::Rem),
    ] {
        let function = format!("vector::{name}");
        let insufficient = tautology_contract(&function);
        let sufficient = sufficient_i8_contract(&function);
        for contract in [&insufficient, &sufficient] {
            let lowering =
                rustc_harness::lower(SOURCE, &function, &[("contracts/div-rem.json", contract)])
                    .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
            assert_exact_checks(div_rem_instruction(&lowering.raw_lowering), operation, true);
        }
    }
}

#[test]
fn every_pinned_div_rem_pattern_component_is_fail_closed() {
    for signed in [false, true] {
        let valid = DivRemPatternVector::pinned(DivRemOperation::Div, signed);
        assert_eq!(validate_div_rem_pattern(&valid), Ok(()));
        let mutations: Vec<PatternMutation> = vec![
            Box::new(|v| v.operation = DivRemOperation::Rem),
            Box::new(|v| v.value_operation = DivRemOperation::Rem),
            Box::new(|v| v.operands_match = false),
            Box::new(|v| v.operand_modes_match = false),
            Box::new(|v| v.type_matches = false),
            Box::new(|v| v.zero_guard_matches = false),
            Box::new(|v| v.zero_message_matches = false),
            Box::new(|v| v.representability_guard_matches = false),
            Box::new(|v| v.representability_guard_matches = true),
            Box::new(|v| v.representability_message_matches = false),
            Box::new(|v| v.representability_message_matches = true),
            Box::new(|v| v.guard_order_matches = false),
            Box::new(|v| v.expected_false = false),
            Box::new(|v| v.conditions_moved = false),
            Box::new(|v| v.unwind_unreachable = false),
            Box::new(|v| v.continuation_matches = false),
            Box::new(|v| v.assertion_uses = 0),
            Box::new(|v| v.assertion_uses = 3),
            Box::new(|v| v.guard_uses_match = false),
        ];
        for mutate in mutations {
            let mut vector = valid.clone();
            mutate(&mut vector);
            if vector != valid {
                assert_eq!(
                    validate_div_rem_pattern(&vector)
                        .expect_err("mutated pattern must reject")
                        .as_str(),
                    "RUST_MIR_ASSERTION"
                );
            }
        }
    }
}

#[test]
fn chained_operations_consume_each_assertion_once() {
    let source =
        b"pub fn chained(left: i16, middle: i16, right: i16) -> i16 { left / middle % right }\n";
    let contract = tautology_contract("vector::chained");
    let lowering = rustc_harness::lower(
        source,
        "vector::chained",
        &[("contracts/div-rem.json", &contract)],
    )
    .expect("lower chained division and remainder");
    let operations = instructions(&lowering.raw_lowering)
        .filter_map(|instruction| member(instruction, "op").as_str())
        .collect::<Vec<_>>();
    assert_eq!(operations, ["bv_sdiv", "bv_srem"]);
    for instruction in instructions(&lowering.raw_lowering)
        .filter(|instruction| member(instruction, "op").as_str().is_some())
    {
        assert_eq!(
            member(instruction, "safety_checks")
                .as_array()
                .expect("safety checks")
                .len(),
            2
        );
    }
}

#[test]
fn methods_overloads_casts_and_intrinsic_calls_are_rejected() {
    for source in [
        b"pub fn bad(left: i8, right: i8) -> i8 { left.wrapping_div(right) }\n".as_slice(),
        b"pub fn bad(left: i8, right: i8) -> i8 { <i8 as core::ops::Div>::div(left, right) }\n"
            .as_slice(),
        b"pub fn bad(left: i8, right: i8) -> i16 { (left as i16) / (right as i16) }\n".as_slice(),
        b"#![feature(core_intrinsics)]\npub fn bad(left: i8, right: i8) -> i8 { unsafe { core::intrinsics::unchecked_div(left, right) } }\n".as_slice(),
    ] {
        let contract = tautology_contract("vector::bad");
        rustc_harness::lower(
            source,
            "vector::bad",
            &[("contracts/div-rem.json", &contract)],
        )
        .expect_err("non-primitive division/remainder form must reject");
    }
}

fn div_rem_instruction(lowering: &JsonValue) -> &JsonValue {
    instructions(lowering)
        .find(|instruction| {
            matches!(
                member(instruction, "op").as_str(),
                Some("bv_sdiv" | "bv_udiv" | "bv_srem" | "bv_urem")
            )
        })
        .expect("division or remainder instruction")
}

fn instructions(lowering: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    let units = member(member(lowering, "vir"), "units")
        .as_array()
        .expect("units");
    let functions = member(&units[0], "functions")
        .as_array()
        .expect("functions");
    member(&functions[0], "blocks")
        .as_array()
        .expect("blocks")
        .iter()
        .flat_map(|block| {
            member(block, "instructions")
                .as_array()
                .expect("instructions")
        })
}

fn assert_exact_checks(instruction: &JsonValue, operation: DivRemOperation, signed: bool) {
    let checks = member(instruction, "safety_checks")
        .as_array()
        .expect("safety checks");
    assert_eq!(checks.len(), if signed { 2 } else { 1 });
    assert_eq!(member(&checks[0], "kind").as_str(), Some("divisor_nonzero"));
    if signed {
        assert_eq!(
            member(&checks[1], "kind").as_str(),
            Some("signed_divrem_representable")
        );
        assert_eq!(
            member(&checks[1], "operation").as_str(),
            Some(operation_name(operation))
        );
    }
}

fn operation_name(operation: DivRemOperation) -> &'static str {
    match operation {
        DivRemOperation::Div => "div",
        DivRemOperation::Rem => "rem",
    }
}

fn vir_operation(operation: DivRemOperation, signed: bool) -> &'static str {
    match (operation, signed) {
        (DivRemOperation::Div, true) => "bv_sdiv",
        (DivRemOperation::Div, false) => "bv_udiv",
        (DivRemOperation::Rem, true) => "bv_srem",
        (DivRemOperation::Rem, false) => "bv_urem",
    }
}

fn tautology_contract(function: &str) -> Vec<u8> {
    contract(function, "")
}

fn sufficient_i8_contract(function: &str) -> Vec<u8> {
    contract(
        function,
        r#"{"op":"not_eq","args":[{"parameter":"right"},{"bv":{"decimal":"0","width":8,"signed":true}}]},{"op":"not","args":[{"op":"and","args":[{"op":"eq","args":[{"parameter":"left"},{"bv":{"decimal":"-128","width":8,"signed":true}}]},{"op":"eq","args":[{"parameter":"right"},{"bv":{"decimal":"-1","width":8,"signed":true}}]}]}]}"#,
    )
}

fn contract(function: &str, requires: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":64,\"function\":\"{function}\",\"requires\":[{requires}],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}
