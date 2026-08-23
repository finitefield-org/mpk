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
    validate_shift_pattern, ShiftOperation, ShiftPatternVector,
};

const SOURCE: &[u8] = include_bytes!("../testdata/bitwise-shift/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/bitwise-shift/expected-vir.json");
type PatternMutation = Box<dyn Fn(&mut ShiftPatternVector)>;

#[test]
fn emitted_module_matches_the_independently_validated_fixture() {
    let function = "vector::shl_u32_i64";
    let lowering = lower_fixture("shl_u32_i64", &tautology_contract(function));
    let expected = json::parse(EXPECTED_VIR, EXPECTED_VIR.len()).expect("expected VIR fixture");
    assert_eq!(*member(&lowering.raw_lowering, "vir"), expected);
}

#[test]
fn primitive_bitwise_matrix_emits_total_operations_without_checks() {
    for (source_operation, vir_operation) in [("and", "bv_and"), ("or", "bv_or"), ("xor", "bv_xor")]
    {
        for signedness in ["i", "u"] {
            for width in [8, 16, 32, 64] {
                let name = format!("{source_operation}_{signedness}{width}");
                let lowering =
                    lower_fixture(&name, &tautology_contract(&format!("vector::{name}")));
                let instruction = operation_instruction(&lowering.raw_lowering, vir_operation);
                assert_eq!(member(instruction, "kind").as_str(), Some("BinOp"));
                assert_empty_checks(instruction);
                assert_bv_type(member(instruction, "type"), width, signedness == "i");
            }
        }
    }

    for signedness in ["i", "u"] {
        for width in [8, 16, 32, 64] {
            let name = format!("not_{signedness}{width}");
            let lowering = lower_fixture(&name, &tautology_contract(&format!("vector::{name}")));
            let instruction = operation_instruction(&lowering.raw_lowering, "bv_not");
            assert_eq!(member(instruction, "kind").as_str(), Some("UnaryOp"));
            assert_empty_checks(instruction);
            assert_bv_type(member(instruction, "type"), width, signedness == "i");
        }
    }
}

#[test]
fn every_lhs_width_and_signedness_selects_the_exact_shift_operation() {
    for source_operation in ["shl", "shr"] {
        for signedness in ["i", "u"] {
            for width in [8, 16, 32, 64] {
                let name = format!("{source_operation}_{signedness}{width}_u8");
                let lowering =
                    lower_fixture(&name, &tautology_contract(&format!("vector::{name}")));
                let expected = match (source_operation, signedness) {
                    ("shl", _) => "bv_shl",
                    ("shr", "i") => "bv_ashr",
                    ("shr", "u") => "bv_lshr",
                    _ => unreachable!("matrix values"),
                };
                let instruction = operation_instruction(&lowering.raw_lowering, expected);
                assert_bv_type(member(instruction, "type"), width, signedness == "i");
                assert_shift_checks(instruction, false);
            }
        }
    }
}

#[test]
fn cross_width_counts_keep_the_original_rhs_type_and_canonical_checks() {
    for signedness in ["i", "u"] {
        for width in [8, 16, 32, 64] {
            let name = format!("shl_u32_{signedness}{width}");
            let lowering = lower_fixture(&name, &tautology_contract(&format!("vector::{name}")));
            let instruction = operation_instruction(&lowering.raw_lowering, "bv_shl");
            assert_shift_checks(instruction, signedness == "i");
            let rhs_id = member(member(instruction, "rhs"), "var")
                .as_str()
                .expect("shift RHS variable");
            let rhs_type = variable_type(&lowering.raw_lowering, rhs_id);
            assert_bv_type(rhs_type, width, signedness == "i");
            assert!(instructions(&lowering.raw_lowering).all(|candidate| member(
                candidate, "kind"
            )
            .as_str()
                != Some("Convert")));
        }
    }
}

#[test]
fn negative_exact_width_and_above_width_counts_lower_as_pending_safety_checks() {
    for (name, decimal, signed) in [
        ("shl_u8_i16", "-1", true),
        ("shl_u8_u16", "8", false),
        ("shl_u8_u16", "9", false),
    ] {
        let function = format!("vector::{name}");
        let contract = count_contract(&function, 16, signed, decimal);
        let lowering = lower_fixture(name, &contract);
        let instruction = operation_instruction(&lowering.raw_lowering, "bv_shl");
        assert_shift_checks(instruction, signed);
    }
}

#[test]
fn every_pinned_shift_pattern_component_is_fail_closed() {
    for signed_rhs in [false, true] {
        let valid = ShiftPatternVector::pinned(ShiftOperation::Shl, signed_rhs);
        assert_eq!(validate_shift_pattern(&valid), Ok(()));
        let mutations: Vec<PatternMutation> = vec![
            Box::new(|v| v.operation = ShiftOperation::Shr),
            Box::new(|v| v.value_operation = ShiftOperation::Shr),
            Box::new(|v| v.operands_match = false),
            Box::new(|v| v.operand_modes_match = false),
            Box::new(|v| v.lhs_type_matches = false),
            Box::new(|v| v.rhs_type_matches = false),
            Box::new(|v| v.cast_matches = false),
            Box::new(|v| v.cast_matches = true),
            Box::new(|v| v.predicate_matches = false),
            Box::new(|v| v.threshold_matches = false),
            Box::new(|v| v.message_matches = false),
            Box::new(|v| v.expected_true = false),
            Box::new(|v| v.condition_moved = false),
            Box::new(|v| v.unwind_unreachable = false),
            Box::new(|v| v.continuation_matches = false),
            Box::new(|v| v.assertion_uses = 0),
            Box::new(|v| v.assertion_uses = 2),
            Box::new(|v| v.guard_uses_match = false),
        ];
        for mutate in mutations {
            let mut vector = valid.clone();
            mutate(&mut vector);
            if vector != valid {
                assert_eq!(
                    validate_shift_pattern(&vector)
                        .expect_err("mutated pattern must reject")
                        .as_str(),
                    "RUST_MIR_ASSERTION"
                );
            }
        }
    }
}

#[test]
fn chained_bitwise_and_shifts_consume_each_assertion_once() {
    let source = b"pub fn chained(value: u16, mask: u16, first: i8, second: u64) -> u16 { (!value & mask) << first >> second }\n";
    let contract = tautology_contract("vector::chained");
    let lowering = rustc_harness::lower(
        source,
        "vector::chained",
        &[("contracts/bitwise-shift.json", &contract)],
    )
    .expect("lower chained bitwise shifts");
    let operations = instructions(&lowering.raw_lowering)
        .filter_map(|instruction| member(instruction, "op").as_str())
        .collect::<Vec<_>>();
    assert_eq!(operations, ["bv_not", "bv_and", "bv_shl", "bv_lshr"]);
    let shifts = instructions(&lowering.raw_lowering)
        .filter(|instruction| {
            matches!(
                member(instruction, "op").as_str(),
                Some("bv_shl" | "bv_lshr")
            )
        })
        .collect::<Vec<_>>();
    assert_shift_checks(shifts[0], true);
    assert_shift_checks(shifts[1], false);
}

#[test]
fn helper_trait_cast_and_unsupported_count_forms_are_rejected() {
    for source in [
        b"pub fn bad(value: u32, count: u32) -> u32 { value.wrapping_shl(count) }\n".as_slice(),
        b"pub fn bad(value: u32, count: u64) -> u32 { <u32 as core::ops::Shl<u64>>::shl(value, count) }\n".as_slice(),
        b"pub fn bad(value: u32, count: u64) -> u32 { value << (count as u8) }\n".as_slice(),
        b"pub fn bad(value: u32, count: bool) -> u32 { value << count }\n".as_slice(),
    ] {
        let contract = tautology_contract("vector::bad");
        rustc_harness::lower(
            source,
            "vector::bad",
            &[("contracts/bitwise-shift.json", &contract)],
        )
        .expect_err("non-primitive or unsupported shift form must reject");
    }
}

fn lower_fixture(name: &str, contract: &[u8]) -> rustc_driver_adapter::MirLowering {
    rustc_harness::lower(
        SOURCE,
        &format!("vector::{name}"),
        &[("contracts/bitwise-shift.json", contract)],
    )
    .unwrap_or_else(|error| panic!("lower {name}: {error:?}"))
}

fn operation_instruction<'a>(lowering: &'a JsonValue, operation: &str) -> &'a JsonValue {
    instructions(lowering)
        .find(|instruction| member(instruction, "op").as_str() == Some(operation))
        .unwrap_or_else(|| panic!("missing {operation} instruction"))
}

fn instructions(lowering: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    let function = first_function(lowering);
    member(function, "blocks")
        .as_array()
        .expect("blocks")
        .iter()
        .flat_map(|block| {
            member(block, "instructions")
                .as_array()
                .expect("instructions")
        })
}

fn variable_type<'a>(lowering: &'a JsonValue, variable: &str) -> &'a JsonValue {
    let function = first_function(lowering);
    member(function, "params")
        .as_array()
        .expect("parameters")
        .iter()
        .chain(
            member(function, "blocks")
                .as_array()
                .expect("blocks")
                .iter()
                .flat_map(|block| {
                    member(block, "parameters")
                        .as_array()
                        .expect("block parameters")
                }),
        )
        .find(|binding| member(binding, "id").as_str() == Some(variable))
        .map(|binding| member(binding, "type"))
        .unwrap_or_else(|| panic!("missing type for {variable}"))
}

fn first_function(lowering: &JsonValue) -> &JsonValue {
    let units = member(member(lowering, "vir"), "units")
        .as_array()
        .expect("units");
    &member(&units[0], "functions")
        .as_array()
        .expect("functions")[0]
}

fn assert_shift_checks(instruction: &JsonValue, signed_rhs: bool) {
    let checks = member(instruction, "safety_checks")
        .as_array()
        .expect("safety checks");
    let expected = if signed_rhs {
        vec!["shift_count_nonnegative", "shift_count_less_than_width"]
    } else {
        vec!["shift_count_less_than_width"]
    };
    assert_eq!(
        checks
            .iter()
            .map(|check| member(check, "kind").as_str().expect("check kind"))
            .collect::<Vec<_>>(),
        expected
    );
}

fn assert_empty_checks(instruction: &JsonValue) {
    assert!(member(instruction, "safety_checks")
        .as_array()
        .expect("safety checks")
        .is_empty());
}

fn assert_bv_type(ty: &JsonValue, width: i64, signed: bool) {
    assert_eq!(member(ty, "kind").as_str(), Some("bv"));
    assert_eq!(member(ty, "width").integer(), Some(width));
    assert_eq!(member(ty, "signed").as_bool(), Some(signed));
}

fn tautology_contract(function: &str) -> Vec<u8> {
    contract(function, "")
}

fn count_contract(function: &str, width: u8, signed: bool, decimal: &str) -> Vec<u8> {
    contract(
        function,
        &format!(
            "{{\"op\":\"eq\",\"args\":[{{\"parameter\":\"count\"}},{{\"bv\":{{\"decimal\":\"{decimal}\",\"width\":{width},\"signed\":{signed}}}}}]}}"
        ),
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
