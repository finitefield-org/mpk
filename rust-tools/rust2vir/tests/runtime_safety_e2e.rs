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
    validate_div_rem_pattern, validate_pattern_vector, validate_shift_pattern, ArithmeticOperation,
    DivRemOperation, DivRemPatternVector, PatternVector, ShiftOperation, ShiftPatternVector,
};
use rustc_driver_adapter::mir_projection::{validate_index_pattern, IndexPatternVector};

const RUNTIME_SAFETY_SOURCE: &[u8] = include_bytes!("../testdata/runtime-safety/checked.rs");
const EXPECTED: &[u8] = include_bytes!("../testdata/runtime-safety/expected.json");
const GUARDED_EXPECTED_VIR: &[u8] = include_bytes!("../testdata/runtime-safety/expected-vir.json");

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    source: &'static [u8],
    function: &'static str,
    sufficient_contract: &'static [u8],
    insufficient_contract: &'static [u8],
    safety_count: usize,
}

fn cases() -> [Case; 4] {
    [
        Case {
            id: "arithmetic",
            source: RUNTIME_SAFETY_SOURCE,
            function: "vector::add_u8",
            sufficient_contract: include_bytes!(
                "../testdata/runtime-safety/sufficient/arithmetic.json"
            ),
            insufficient_contract: include_bytes!(
                "../testdata/runtime-safety/insufficient/arithmetic.json"
            ),
            safety_count: 1,
        },
        Case {
            id: "div-rem",
            source: RUNTIME_SAFETY_SOURCE,
            function: "vector::div_i8",
            sufficient_contract: include_bytes!(
                "../testdata/runtime-safety/sufficient/div-rem.json"
            ),
            insufficient_contract: include_bytes!(
                "../testdata/runtime-safety/insufficient/div-rem.json"
            ),
            safety_count: 2,
        },
        Case {
            id: "shift",
            source: RUNTIME_SAFETY_SOURCE,
            function: "vector::shl_u32_i64",
            sufficient_contract: include_bytes!("../testdata/runtime-safety/sufficient/shift.json"),
            insufficient_contract: include_bytes!(
                "../testdata/runtime-safety/insufficient/shift.json"
            ),
            safety_count: 2,
        },
        Case {
            id: "index",
            source: RUNTIME_SAFETY_SOURCE,
            function: "vector::read",
            sufficient_contract: include_bytes!("../testdata/runtime-safety/sufficient/index.json"),
            insufficient_contract: include_bytes!(
                "../testdata/runtime-safety/insufficient/index.json"
            ),
            safety_count: 1,
        },
    ]
}

#[test]
fn every_runtime_safety_family_lowers_exact_checks_deterministically() {
    for case in cases() {
        for (contract, requires_safety_preconditions) in [
            (case.sufficient_contract, true),
            (case.insufficient_contract, false),
        ] {
            let first = lower(&case, contract);
            let second = lower(&case, contract);
            assert_eq!(
                json::canonical(&first).unwrap(),
                json::canonical(&second).unwrap(),
                "{} VIR differs between clean runs",
                case.id
            );
            assert_eq!(
                function_requires(&first).is_empty(),
                !requires_safety_preconditions,
                "{} precondition fixture",
                case.id
            );
            assert_eq!(
                instruction_safety_count(&first),
                case.safety_count,
                "{} safety count",
                case.id
            );
        }
    }
}

#[test]
fn guarded_runtime_safety_stays_on_the_guarded_branch() {
    let contract = include_bytes!("../testdata/runtime-safety/insufficient/guarded-div.json");
    let case = Case {
        id: "guarded-div",
        source: RUNTIME_SAFETY_SOURCE,
        function: "vector::guarded_div",
        sufficient_contract: contract,
        insufficient_contract: contract,
        safety_count: 2,
    };
    let vir = lower(&case, contract);
    assert_eq!(
        vir,
        json::parse(GUARDED_EXPECTED_VIR, GUARDED_EXPECTED_VIR.len()).unwrap(),
        "guarded runtime-safety VIR golden"
    );
    assert_eq!(instruction_safety_count(&vir), 2);
    let blocks = function_blocks(&vir);
    assert!(blocks.len() >= 3);
    assert!(blocks.iter().any(|block| {
        member(block, "instructions")
            .as_array()
            .unwrap()
            .iter()
            .any(|instruction| {
                member(instruction, "safety_checks")
                    .as_array()
                    .unwrap()
                    .len()
                    == 2
            })
    }));
}

#[test]
fn each_compiler_assertion_family_rejects_a_consumption_mutation() {
    let mut arithmetic = PatternVector::pinned(ArithmeticOperation::Add);
    arithmetic.assertion_uses = 0;
    assert_eq!(
        validate_pattern_vector(&arithmetic).unwrap_err().as_str(),
        "RUST_MIR_ASSERTION"
    );

    let mut div_rem = DivRemPatternVector::pinned(DivRemOperation::Div, true);
    div_rem.assertion_uses = 0;
    assert_eq!(
        validate_div_rem_pattern(&div_rem).unwrap_err().as_str(),
        "RUST_MIR_ASSERTION"
    );

    let mut shift = ShiftPatternVector::pinned(ShiftOperation::Shl, true);
    shift.assertion_uses = 0;
    assert_eq!(
        validate_shift_pattern(&shift).unwrap_err().as_str(),
        "RUST_MIR_ASSERTION"
    );

    let mut index = IndexPatternVector::pinned();
    index.assertion_uses = 0;
    assert_eq!(
        validate_index_pattern(&index).unwrap_err().as_str(),
        "RUST_MIR_ASSERTION"
    );
}

#[test]
fn insufficient_preconditions_remain_pending_and_the_phase_ledger_is_empty() {
    let expected = json::parse(EXPECTED, EXPECTED.len()).expect("runtime-safety fixture manifest");
    assert_eq!(member(&expected, "clean_runs").integer(), Some(2));
    assert_eq!(
        member(&expected, "unchecked_rust_semantic_axioms").integer(),
        Some(0)
    );
    let sufficient = member(&expected, "sufficient_preconditions");
    assert_eq!(
        member(sufficient, "foundation").as_str(),
        Some("Std.Program.Base")
    );
    assert_eq!(
        member(sufficient, "evidence_path").as_str(),
        Some("mpk.vc.cert_skeleton.v1")
    );
    let insufficient = member(&expected, "insufficient_preconditions");
    assert_eq!(
        member(insufficient, "frontend_status").as_str(),
        Some("ir-lowered")
    );
    assert_eq!(
        member(insufficient, "non_strict_status").as_str(),
        Some("proof_pending")
    );
    assert_eq!(
        member(insufficient, "strict_error").as_str(),
        Some("POLICY_PROOF_PENDING")
    );
    assert!(member(&expected, "findings").as_array().unwrap().is_empty());
}

fn lower(case: &Case, contract: &[u8]) -> JsonValue {
    let lowering = rustc_harness::lower(
        case.source,
        case.function,
        &[("contracts/runtime-safety.json", contract)],
    )
    .unwrap_or_else(|error| panic!("{} lowering failed: {error:?}", case.id));
    member(&lowering.raw_lowering, "vir").clone()
}

fn function_requires(vir: &JsonValue) -> &[JsonValue] {
    member(member(first_function(vir), "contracts"), "requires")
        .as_array()
        .expect("requires")
}

fn instruction_safety_count(vir: &JsonValue) -> usize {
    function_blocks(vir)
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
        .map(|instruction| {
            member(instruction, "safety_checks")
                .as_array()
                .unwrap()
                .len()
        })
        .sum()
}

fn first_function(vir: &JsonValue) -> &JsonValue {
    &member(&member(vir, "units").as_array().unwrap()[0], "functions")
        .as_array()
        .unwrap()[0]
}

fn function_blocks(vir: &JsonValue) -> &[JsonValue] {
    member(first_function(vir), "blocks")
        .as_array()
        .expect("blocks")
}

fn member<'a>(value: &'a JsonValue, field: &str) -> &'a JsonValue {
    &value.as_object().expect("JSON object")[field]
}
