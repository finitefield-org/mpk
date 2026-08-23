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
    validate_pattern_vector, ArithmeticOperation, PatternVector,
};
use rustc_driver_adapter::RustcDriverError;

const SOURCE: &[u8] = include_bytes!("../testdata/arithmetic/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/arithmetic/expected-vir.json");
type PatternMutation = Box<dyn Fn(&mut PatternVector)>;

#[test]
fn emitted_modules_match_the_independently_validated_fixture() {
    let cases: &[(&[u8], &str)] = &[
        (
            b"pub fn signed(left: i8, middle: i8, right: i8) -> i8 { -(left + middle) * right }\n",
            "vector::signed",
        ),
        (
            b"pub fn unsigned(left: u8, right: u8) -> u8 { left + right - left * right }\n",
            "vector::unsigned",
        ),
        (b"pub fn minimum() -> i8 { -128_i8 }\n", "vector::minimum"),
    ];
    let mut modules = Vec::new();
    for (source, function) in cases {
        let contract = tautology_contract(function);
        let lowering = rustc_harness::lower(
            source,
            function,
            &[("contracts/arithmetic.json", &contract)],
        )
        .expect("lower validator fixture");
        modules.push(member(&lowering.raw_lowering, "vir").clone());
    }
    let expected = json::parse(EXPECTED_VIR, EXPECTED_VIR.len()).expect("expected VIR fixture");
    assert_eq!(JsonValue::Array(modules), expected);
}

#[test]
fn checked_binary_matrix_emits_the_exact_operation_and_overflow_check() {
    for operation in ["add", "sub", "mul"] {
        for signedness in ["i", "u"] {
            for width in [8, 16, 32, 64] {
                let name = format!("{operation}_{signedness}{width}");
                let function = format!("vector::{name}");
                let contract = tautology_contract(&function);
                let lowering = rustc_harness::lower(
                    SOURCE,
                    &function,
                    &[("contracts/arithmetic.json", &contract)],
                )
                .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
                let instruction = arithmetic_instruction(&lowering.raw_lowering);
                assert_eq!(member(instruction, "kind").as_str(), Some("BinOp"));
                let expected_operation = format!("bv_{operation}");
                assert_eq!(
                    member(instruction, "op").as_str(),
                    Some(expected_operation.as_str())
                );
                assert_overflow_check(instruction, operation, signedness == "i");
                assert_eq!(
                    member(member(instruction, "type"), "width").integer(),
                    Some(width)
                );
            }
        }
    }
}

#[test]
fn checked_negation_matrix_emits_one_negate_with_its_required_check() {
    for width in [8, 16, 32, 64] {
        let name = format!("neg_i{width}");
        let function = format!("vector::{name}");
        let contract = tautology_contract(&function);
        let lowering = rustc_harness::lower(
            SOURCE,
            &function,
            &[("contracts/arithmetic.json", &contract)],
        )
        .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
        let instruction = arithmetic_instruction(&lowering.raw_lowering);
        assert_eq!(member(instruction, "kind").as_str(), Some("UnaryOp"));
        assert_eq!(member(instruction, "op").as_str(), Some("bv_neg"));
        assert_overflow_check(instruction, "neg", true);
        assert_eq!(
            instruction_source(&lowering.raw_source_map, SOURCE),
            "-value"
        );
    }
}

#[test]
fn minimum_literals_are_single_constants_without_negation_checks() {
    for (width, value) in [
        (8, "-128"),
        (16, "-32768"),
        (32, "-2147483648"),
        (64, "-9223372036854775808"),
    ] {
        let name = format!("min_i{width}");
        let function = format!("vector::{name}");
        let contract = tautology_contract(&function);
        let lowering = rustc_harness::lower(
            SOURCE,
            &function,
            &[("contracts/arithmetic.json", &contract)],
        )
        .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
        let instruction = arithmetic_instruction(&lowering.raw_lowering);
        assert_eq!(member(instruction, "kind").as_str(), Some("Const"));
        assert_eq!(
            member(member(member(instruction, "value"), "int"), "value").as_str(),
            Some(value)
        );
        assert!(member(instruction, "safety_checks")
            .as_array()
            .expect("safety checks")
            .is_empty());
        assert_eq!(
            instruction_source(&lowering.raw_source_map, SOURCE),
            format!("{value}_i{width}")
        );
    }

    let contract = tautology_contract("vector::above_min_i8");
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::above_min_i8",
        &[("contracts/arithmetic.json", &contract)],
    )
    .expect("lower literal immediately above the signed minimum");
    let instruction = arithmetic_instruction(&lowering.raw_lowering);
    assert_eq!(member(instruction, "kind").as_str(), Some("Const"));
    assert_eq!(
        member(member(member(instruction, "value"), "int"), "value").as_str(),
        Some("-127")
    );
    assert!(member(instruction, "safety_checks")
        .as_array()
        .expect("safety checks")
        .is_empty());
}

#[test]
fn negative_zero_and_positive_literal_operands_remain_checked() {
    for (name, value) in [
        ("add_below_i8", "-1"),
        ("add_at_i8", "0"),
        ("add_above_i8", "1"),
    ] {
        let function = format!("vector::{name}");
        let contract = tautology_contract(&function);
        let lowering = rustc_harness::lower(
            SOURCE,
            &function,
            &[("contracts/arithmetic.json", &contract)],
        )
        .unwrap_or_else(|error| panic!("lower {name}: {error:?}"));
        let instruction = arithmetic_instruction(&lowering.raw_lowering);
        assert_overflow_check(instruction, "add", true);
        assert_eq!(
            member(member(member(instruction, "rhs"), "int"), "value").as_str(),
            Some(value)
        );
    }
}

#[test]
fn every_pinned_pattern_component_is_fail_closed() {
    let valid = PatternVector::pinned(ArithmeticOperation::Add);
    assert_eq!(validate_pattern_vector(&valid), Ok(()));
    let mutations: Vec<PatternMutation> = vec![
        Box::new(|v| v.operation = ArithmeticOperation::Mul),
        Box::new(|v| v.value_operation = ArithmeticOperation::Sub),
        Box::new(|v| v.lhs_matches = false),
        Box::new(|v| v.rhs_matches = false),
        Box::new(|v| v.type_matches = false),
        Box::new(|v| v.tuple_matches = false),
        Box::new(|v| v.operand_modes_match = false),
        Box::new(|v| v.flag_field = 0),
        Box::new(|v| v.flag_moved = false),
        Box::new(|v| v.result_field = 1),
        Box::new(|v| v.result_moved = false),
        Box::new(|v| v.expected = true),
        Box::new(|v| v.message_matches = false),
        Box::new(|v| v.continuation_matches = false),
        Box::new(|v| v.unwind_unreachable = false),
        Box::new(|v| v.assertion_uses = 0),
        Box::new(|v| v.assertion_uses = 2),
        Box::new(|v| v.flag_uses = 0),
        Box::new(|v| v.flag_uses = 2),
        Box::new(|v| v.result_uses = 0),
        Box::new(|v| v.result_uses = 2),
    ];
    for mutate in mutations {
        let mut vector = valid.clone();
        mutate(&mut vector);
        assert_eq!(
            validate_pattern_vector(&vector)
                .expect_err("mutated pattern must reject")
                .as_str(),
            "RUST_MIR_CHECKED_PATTERN"
        );
    }
}

#[test]
fn chained_checked_operations_keep_every_assertion_attached() {
    let source =
        b"pub fn chained(left: i16, middle: i16, right: i16) -> i16 { -(left + middle) * right }\n";
    let contract = tautology_contract("vector::chained");
    let lowering = rustc_harness::lower(
        source,
        "vector::chained",
        &[("contracts/arithmetic.json", &contract)],
    )
    .expect("lower chained checked arithmetic");
    let operations = instructions(&lowering.raw_lowering)
        .filter_map(|instruction| member(instruction, "op").as_str())
        .collect::<Vec<_>>();
    assert_eq!(operations, vec!["bv_add", "bv_neg", "bv_mul"]);
    for instruction in instructions(&lowering.raw_lowering)
        .filter(|instruction| member(instruction, "op").as_str().is_some())
    {
        assert_eq!(
            member(instruction, "safety_checks")
                .as_array()
                .expect("safety checks")
                .len(),
            1
        );
    }
}

#[test]
fn unsigned_negation_and_out_of_range_literals_reject_at_source_analysis() {
    let unsigned = b"pub fn bad(value: u8) -> u8 { -value }\n";
    assert_eq!(
        rustc_harness::analyze(unsigned, "vector::bad"),
        Err(RustcDriverError::Compiler)
    );

    let out_of_range = b"pub fn bad() -> i8 { -129_i8 }\n";
    let contract = tautology_contract("vector::bad");
    assert_eq!(
        rustc_harness::lower(
            out_of_range,
            "vector::bad",
            &[("contracts/arithmetic.json", &contract)],
        ),
        Err(RustcDriverError::Compiler)
    );
}

fn arithmetic_instruction(lowering: &JsonValue) -> &JsonValue {
    instructions(lowering)
        .find(|instruction| {
            matches!(
                member(instruction, "kind").as_str(),
                Some("BinOp" | "UnaryOp" | "Const")
            )
        })
        .expect("arithmetic instruction")
}

fn instructions(lowering: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    let vir = member(lowering, "vir");
    let units = member(vir, "units").as_array().expect("units");
    let functions = member(&units[0], "functions")
        .as_array()
        .expect("functions");
    let blocks = member(&functions[0], "blocks").as_array().expect("blocks");
    blocks.iter().flat_map(|block| {
        member(block, "instructions")
            .as_array()
            .expect("instructions")
    })
}

fn assert_overflow_check(instruction: &JsonValue, operation: &str, signed: bool) {
    let checks = member(instruction, "safety_checks")
        .as_array()
        .expect("safety checks");
    assert_eq!(checks.len(), 1);
    assert_eq!(
        member(&checks[0], "kind").as_str(),
        Some("integer_no_overflow")
    );
    assert_eq!(member(&checks[0], "operation").as_str(), Some(operation));
    assert_eq!(member(&checks[0], "signed").as_bool(), Some(signed));
}

fn tautology_contract(function: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":64,\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}

fn instruction_source<'a>(source_map: &JsonValue, source: &'a [u8]) -> &'a str {
    let entry = member(source_map, "entries")
        .as_array()
        .expect("source map entries")
        .iter()
        .find(|entry| member(member(entry, "reference"), "kind").as_str() == Some("instruction"))
        .expect("instruction source entry");
    let origin = member(entry, "origin");
    let start = usize::try_from(member(origin, "start").integer().expect("origin start")).unwrap();
    let end = usize::try_from(member(origin, "end").integer().expect("origin end")).unwrap();
    std::str::from_utf8(&source[start..end]).expect("source UTF-8")
}
