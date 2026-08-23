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

#[path = "common/mod.rs"]
mod common;
#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;
#[path = "support/rustc_harness.rs"]
mod rustc_harness;

use rust2vir_internal::json::JsonValue;
use rustc_driver_adapter::mir_projection::{validate_index_pattern, IndexPatternVector};
use rustc_driver_adapter::RustcDriverError;
use std::fs;
use std::os::unix::fs::PermissionsExt;

const SOURCE: &[u8] = include_bytes!("../testdata/array-index/checked.rs");
const EXPECTED_VIR: &[u8] = include_bytes!("../testdata/array-index/expected-vir.json");
type PatternMutation = Box<dyn Fn(&mut IndexPatternVector)>;

#[test]
fn both_registered_targets_emit_one_target_width_usize_index_check() {
    let mut hashes = Vec::new();
    let mut modules = Vec::new();
    for (target, width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let lowering = lower(
            target,
            width,
            "read",
            &tautology_contract("vector::read", width),
        );
        let vir = member(&lowering.raw_lowering, "vir");
        assert_eq!(
            member(member(vir, "semantic_parameters"), "target_id").as_str(),
            Some(target)
        );
        assert_eq!(
            member(member(vir, "semantic_parameters"), "pointer_width").integer(),
            Some(i64::from(width))
        );
        let instruction = index_instruction(vir);
        assert_eq!(member(instruction, "kind").as_str(), Some("Index"));
        assert_bv_type(
            variable_type(
                vir,
                member(member(instruction, "index"), "var")
                    .as_str()
                    .unwrap(),
            ),
            width,
            false,
        );
        assert_eq!(
            member(instruction, "safety_checks").as_array().unwrap(),
            &[JsonValue::Object(std::collections::BTreeMap::from([(
                "kind".to_owned(),
                JsonValue::String("index_in_bounds".to_owned()),
            )]))]
        );
        hashes.push(member(vir, "vir_hash").as_str().unwrap().to_owned());
        modules.push(vir.clone());
    }
    assert_ne!(hashes[0], hashes[1]);
    let expected = rust2vir_internal::json::parse(EXPECTED_VIR, EXPECTED_VIR.len())
        .expect("expected dual-target array-index VIR fixture");
    assert_eq!(JsonValue::Array(modules), expected);
}

#[test]
fn zero_last_and_length_boundaries_keep_the_same_pending_bounds_check() {
    for name in ["zero", "last", "length"] {
        for (target, width) in [
            ("i686-unknown-linux-gnu", 32),
            ("x86_64-unknown-linux-gnu", 64),
        ] {
            let function = format!("vector::{name}");
            let contract = if name == "length" {
                index_contract(&function, width, 4)
            } else {
                tautology_contract(&function, width)
            };
            let lowering = lower(target, width, name, &contract);
            let vir = member(&lowering.raw_lowering, "vir");
            let index = index_instruction(vir);
            assert_eq!(
                member(
                    member(index, "safety_checks")
                        .as_array()
                        .unwrap()
                        .first()
                        .unwrap(),
                    "kind"
                )
                .as_str(),
                Some("index_in_bounds")
            );
            assert_bv_type(
                variable_type(vir, member(member(index, "index"), "var").as_str().unwrap()),
                width,
                false,
            );
        }
    }
}

#[test]
fn every_pinned_index_pattern_component_is_fail_closed() {
    let valid = IndexPatternVector::pinned();
    assert_eq!(validate_index_pattern(&valid), Ok(()));
    let mutations: Vec<PatternMutation> = vec![
        Box::new(|v| v.base_is_fixed_array = false),
        Box::new(|v| v.index_is_target_usize = false),
        Box::new(|v| v.element_is_copy = false),
        Box::new(|v| v.predicate_matches = false),
        Box::new(|v| v.message_matches = false),
        Box::new(|v| v.length_matches = false),
        Box::new(|v| v.operand_modes_match = false),
        Box::new(|v| v.projection_is_copy = false),
        Box::new(|v| v.expected_true = false),
        Box::new(|v| v.condition_moved = false),
        Box::new(|v| v.continuation_matches = false),
        Box::new(|v| v.unwind_unreachable = false),
        Box::new(|v| v.assertion_uses = 0),
        Box::new(|v| v.assertion_uses = 2),
        Box::new(|v| v.guard_uses = 0),
        Box::new(|v| v.guard_uses = 2),
        Box::new(|v| v.index_uses = 2),
        Box::new(|v| v.index_uses = 4),
        Box::new(|v| v.projection_uses = 0),
        Box::new(|v| v.projection_uses = 2),
    ];
    for mutate in mutations {
        let mut changed = valid.clone();
        mutate(&mut changed);
        assert_eq!(
            validate_index_pattern(&changed)
                .expect_err("mutated bounds/index pattern must reject")
                .as_str(),
            "RUST_MIR_ASSERTION"
        );
    }
}

#[test]
fn each_bounds_assertion_is_bound_to_its_own_projection() {
    for (target, width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let lowering = lower(
            target,
            width,
            "independent",
            &tautology_contract("vector::independent", width),
        );
        let indexes = instructions(member(&lowering.raw_lowering, "vir"))
            .filter(|instruction| member(instruction, "kind").as_str() == Some("Index"))
            .collect::<Vec<_>>();
        assert_eq!(indexes.len(), 2);
        for index in indexes {
            assert_eq!(
                member(index, "safety_checks").as_array().unwrap(),
                &[JsonValue::Object(std::collections::BTreeMap::from([(
                    "kind".to_owned(),
                    JsonValue::String("index_in_bounds".to_owned()),
                )]))]
            );
        }
    }
}

#[test]
fn signed_fixed_width_cast_reference_slice_mutation_and_partial_move_forms_reject() {
    let cases: &[(&[u8], &str)] = &[
        (
            b"pub fn bad(a: [u8; 4], i: isize) -> u8 { a[i] }\n",
            "isize",
        ),
        (b"pub fn bad(a: [u8; 4], i: u32) -> u8 { a[i] }\n", "u32"),
        (b"pub fn bad(a: [u8; 4], i: u64) -> u8 { a[i] }\n", "u64"),
        (
            b"pub fn bad(a: [u8; 4], i: u32) -> u8 { a[i as usize] }\n",
            "cast",
        ),
        (
            b"pub fn bad(a: &[u8; 4], i: usize) -> u8 { a[i] }\n",
            "reference",
        ),
        (b"pub fn bad(a: &[u8], i: usize) -> u8 { a[i] }\n", "slice"),
        (
            b"pub fn bad(mut a: [u8; 4], i: usize) -> u8 { a[i] = 1; a[0] }\n",
            "mutation",
        ),
        (
            b"pub struct Item { pub value: u8 }\npub fn bad(a: [Item; 1]) -> Item { a[0] }\n",
            "partial move",
        ),
        (
            b"pub struct UserIndex(pub [u8; 1]);\nimpl core::ops::Index<usize> for UserIndex { type Output = u8; fn index(&self, index: usize) -> &u8 { &self.0[index] } }\npub fn bad(a: UserIndex, i: usize) -> u8 { a[i] }\n",
            "user Index",
        ),
    ];
    for (source, label) in cases {
        let contract = tautology_contract("vector::bad", 64);
        assert!(
            rustc_harness::lower(source, "vector::bad", &[("contracts/bad.json", &contract)])
                .is_err(),
            "{label} form must reject"
        );
    }
}

#[test]
fn crossed_target_and_pointer_width_reject_before_lowering() {
    let contract = tautology_contract("vector::read", 32);
    let error = rustc_harness::lower_with_session_target(
        SOURCE,
        "vector::read",
        &[("contracts/array-index.json", &contract)],
        "x86_64-unknown-linux-gnu",
        "i686-unknown-linux-gnu",
        32,
    )
    .expect_err("crossed target and width must reject");
    assert_eq!(error, RustcDriverError::Session);
}

#[test]
fn either_registered_target_library_digest_mismatch_rejects() {
    for relative in [
        "lib/rustlib/i686-unknown-linux-gnu/lib/libstd.rlib",
        "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib",
    ] {
        let fixture = common::Fixture::new();
        let library = fixture.toolchain_root().join(relative);
        fs::set_permissions(&library, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&library, b"changed-target-library").unwrap();
        let error = fixture
            .candidate()
            .validate_for(fixture.request())
            .expect_err("changed target library must reject");
        assert_eq!(error.code(), "RUST_TOOLCHAIN_COMPONENT");
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
        &[("contracts/array-index.json", contract)],
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

fn index_contract(function: &str, pointer_width: u8, index: u8) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":{pointer_width},\"function\":\"{function}\",\"requires\":[{{\"op\":\"eq\",\"args\":[{{\"parameter\":\"index\"}},{{\"bv\":{{\"decimal\":\"{index}\",\"width\":{pointer_width},\"signed\":false}}}}]}}],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn index_instruction(vir: &JsonValue) -> &JsonValue {
    instructions(vir)
        .find(|instruction| member(instruction, "kind").as_str() == Some("Index"))
        .expect("Index instruction")
}

fn instructions(vir: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    member(member(member(member(vir, "units"), 0), "functions"), 0)
        .as_object()
        .unwrap()["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|block| member(block, "instructions").as_array().unwrap())
}

fn variable_type<'a>(vir: &'a JsonValue, id: &str) -> &'a JsonValue {
    let function = member(member(member(member(vir, "units"), 0), "functions"), 0);
    for collection in ["params", "locals"] {
        if let Some(binding) = member(function, collection)
            .as_array()
            .unwrap()
            .iter()
            .find(|binding| member(binding, "id").as_str() == Some(id))
        {
            return member(binding, "type");
        }
    }
    for block in member(function, "blocks").as_array().unwrap() {
        for collection in ["parameters", "instructions"] {
            if let Some(binding) = member(block, collection)
                .as_array()
                .unwrap()
                .iter()
                .find(|binding| member(binding, "id").as_str() == Some(id))
            {
                return member(binding, "type");
            }
        }
    }
    panic!("unknown VIR value {id}")
}

fn assert_bv_type(value: &JsonValue, width: u8, signed: bool) {
    assert_eq!(member(value, "kind").as_str(), Some("bv"));
    assert_eq!(member(value, "width").integer(), Some(i64::from(width)));
    assert_eq!(member(value, "signed").as_bool(), Some(signed));
}

trait MemberKey {
    fn get(self, value: &JsonValue) -> &JsonValue;
}

impl MemberKey for &str {
    fn get(self, value: &JsonValue) -> &JsonValue {
        &value.as_object().expect("object")[self]
    }
}

impl MemberKey for usize {
    fn get(self, value: &JsonValue) -> &JsonValue {
        &value.as_array().expect("array")[self]
    }
}

fn member<K: MemberKey>(value: &JsonValue, key: K) -> &JsonValue {
    key.get(value)
}
