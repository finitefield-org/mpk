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

use rust2vir_internal::contract::{
    ContractCode, ContractError, ContractFunction, ContractInput, ContractType,
};
use rust2vir_internal::contract_typecheck::attach_contracts;
use rust2vir_internal::json::{self, JsonValue};
use rustc_driver_adapter::RustcDriverError;

const IDENTITY: &[u8] = include_bytes!("../testdata/contracts/identity.json");
const IDENTITY_WHITESPACE: &[u8] = include_bytes!("../testdata/contracts/identity-whitespace.json");
const SUBSET_VECTOR: &[u8] = include_bytes!("../testdata/rust-subset-v0.json");

#[test]
fn frozen_complete_operator_vector_is_compiler_typed_and_normalized() {
    let vector = json::parse(SUBSET_VECTOR, SUBSET_VECTOR.len()).expect("parse subset vector");
    let case = vector.as_object().expect("vector object")["accepted_cases"]
        .as_array()
        .expect("accepted cases")
        .iter()
        .find_map(|case| {
            let case = case.as_object()?;
            (case.get("id")?.as_str()? == "contract.complete_operator_set").then_some(case)
        })
        .expect("complete contract vector");
    let source = case["source"].as_str().expect("vector source").as_bytes();
    let contract = json::canonical(&case["contract"]).expect("canonical sidecar");
    let analysis =
        rustc_harness::analyze_contracts(source, "vector::f", &[("contracts/f.json", &contract)])
            .expect("complete operator set is accepted");

    assert_eq!(analysis.hir.call_closure.len(), 1);
    let normalized = &analysis.contracts.contracts()[0];
    assert_eq!(normalized.function_id, "vector::f");
    assert_eq!(normalized.contract_hash.len(), 64);
    let root = normalized.value.as_object().expect("contract object");
    assert_eq!(root["requires"].as_array().expect("requires").len(), 27);
    assert_eq!(
        root["ensures"].as_array().expect("ensures")[0],
        object([
            ("op", JsonValue::String("eq".to_owned())),
            (
                "lhs",
                object([("result", JsonValue::Number("0".to_owned()))])
            ),
            (
                "rhs",
                object([("var", JsonValue::String("arg0".to_owned()))])
            ),
        ])
    );
}

#[test]
fn attachment_is_order_independent_and_closure_membership_is_exact() {
    let functions = vec![
        scalar_function("vector::f", "x", unsigned(8)),
        scalar_function("vector::helper", "value", unsigned(8)),
    ];
    let f = sidecar(
        "vector::f",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"parameter":"x"}]}"#,
    );
    let helper = sidecar(
        "vector::helper",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"parameter":"value"}]}"#,
    );
    let forward = attach_contracts(
        vec![
            input("contracts/f.json", &f),
            input("contracts/helper.json", &helper),
        ],
        &functions,
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("forward order");
    let reverse = attach_contracts(
        vec![
            input("contracts/helper.json", &helper),
            input("contracts/f.json", &f),
        ],
        &functions,
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("reverse order");
    assert_eq!(forward, reverse);
    assert_eq!(
        forward
            .contracts()
            .iter()
            .map(|contract| contract.function_id.as_str())
            .collect::<Vec<_>>(),
        ["vector::f", "vector::helper"]
    );

    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &f)],
            &functions,
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Missing,
    );
    let unused = sidecar("vector::unused", 64, "", r#"{"bool":true}"#);
    assert_error(
        attach_contracts(
            vec![
                input("contracts/f.json", &f),
                input("contracts/unused.json", &unused),
            ],
            &functions,
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Unused,
    );
    assert_error(
        attach_contracts(
            vec![
                input("contracts/f-a.json", &f),
                input("contracts/f-b.json", &f),
                input("contracts/helper.json", &helper),
            ],
            &functions,
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Duplicate,
    );
}

#[test]
fn compiler_resolved_call_closure_receives_one_typed_contract_per_function() {
    let source = b"pub fn f(x: u8) -> u8 { helper(x) }\nfn helper(value: u8) -> u8 { value }\n";
    let f = sidecar(
        "vector::f",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"parameter":"x"}]}"#,
    );
    let helper = sidecar(
        "vector::helper",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"parameter":"value"}]}"#,
    );
    let analysis = rustc_harness::analyze_contracts(
        source,
        "vector::f",
        &[
            ("contracts/helper.json", helper.as_bytes()),
            ("contracts/f.json", f.as_bytes()),
        ],
    )
    .expect("compiler-resolved closure contracts");

    assert_eq!(analysis.hir.call_closure.len(), 2);
    assert_eq!(
        analysis
            .contracts
            .contracts()
            .iter()
            .map(|contract| contract.function_id.as_str())
            .collect::<Vec<_>>(),
        ["vector::f", "vector::helper"]
    );
}

#[test]
fn profile_resolution_type_operator_and_shape_errors_have_stable_codes() {
    let function = scalar_function("vector::f", "x", unsigned(8));
    for (raw, code) in [
        (
            sidecar("vector::f", 32, "", r#"{"bool":true}"#),
            ContractCode::Profile,
        ),
        (
            sidecar(
                "vector::f",
                64,
                "",
                r#"{"op":"signed_lt","args":[{"parameter":"x"},{"bv":{"decimal":"0","width":8,"signed":false}}]}"#,
            ),
            ContractCode::Type,
        ),
        (
            sidecar(
                "vector::f",
                64,
                "",
                r#"{"op":"bv_udiv","args":[{"parameter":"x"},{"parameter":"x"}]}"#,
            ),
            ContractCode::Operator,
        ),
        (
            sidecar("vector::f", 64, "", r#"{"parameter":"local"}"#),
            ContractCode::Resolution,
        ),
        (
            sidecar("vector::f", 64, r#"{"result":0}"#, r#"{"bool":true}"#),
            ContractCode::Resolution,
        ),
    ] {
        assert_error(
            attach_contracts(
                vec![input("contracts/f.json", &raw)],
                std::slice::from_ref(&function),
                "x86_64-unknown-linux-gnu",
                64,
            ),
            code,
        );
    }

    let malformed = br#"{"schema":"mpk.rust.contract.v0",}"#;
    assert_error(
        attach_contracts(
            vec![ContractInput::new("contracts/f.json", malformed.as_slice())],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Json,
    );
    let duplicate_key = sidecar("vector::f", 64, "", r#"{"bool":true}"#).replacen(
        r#""schema":"mpk.rust.contract.v0""#,
        r#""schema":"mpk.rust.contract.v0","schema":"mpk.rust.contract.v0""#,
        1,
    );
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &duplicate_key)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Json,
    );
    let wrong_schema = sidecar("vector::f", 64, "", r#"{"bool":true}"#)
        .replace("mpk.rust.contract.v0", "mpk.rust.contract.v1");
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &wrong_schema)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Schema,
    );
    let unknown_field = sidecar("vector::f", 64, "", r#"{"bool":true}"#).replacen(
        r#""loops":[]"#,
        r#""loops":[],"extra":true"#,
        1,
    );
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &unknown_field)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Shape,
    );

    let oversized_function_id = format!("vector::{}", "f".repeat(1_017));
    let oversized_identity = sidecar(&oversized_function_id, 64, "", r#"{"bool":true}"#);
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &oversized_identity)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Identity,
    );
}

#[test]
fn aggregate_values_are_restricted_to_exact_equality() {
    let functions = [ContractFunction {
        function_id: "vector::f".to_owned(),
        parameter_names: vec!["array".to_owned(), "record".to_owned()],
        parameter_types: vec![
            ContractType::Array {
                element: Box::new(unsigned(8)),
                length: 2,
            },
            ContractType::Struct {
                id: "vector::Record".to_owned(),
            },
        ],
        result_type: ContractType::Array {
            element: Box::new(unsigned(8)),
            length: 2,
        },
    }];
    let accepted = sidecar(
        "vector::f",
        64,
        r#"{"op":"not_eq","args":[{"parameter":"record"},{"parameter":"record"}]}"#,
        r#"{"op":"eq","args":[{"result":0},{"parameter":"array"}]}"#,
    );
    attach_contracts(
        vec![input("contracts/f.json", &accepted)],
        &functions,
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("exact aggregate equality is accepted");

    let rejected = sidecar(
        "vector::f",
        64,
        "",
        r#"{"op":"bv_add","args":[{"parameter":"array"},{"parameter":"array"}]}"#,
    );
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &rejected)],
            &functions,
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Type,
    );
}

#[test]
fn bitvector_literal_spelling_and_ranges_are_exact() {
    let unsigned_function = scalar_function("vector::f", "x", unsigned(64));
    let accepted_unsigned = sidecar(
        "vector::f",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"bv":{"decimal":"18446744073709551615","width":64,"signed":false}}]}"#,
    );
    attach_contracts(
        vec![input("contracts/f.json", &accepted_unsigned)],
        std::slice::from_ref(&unsigned_function),
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("maximum u64 literal is accepted");

    let signed_function = scalar_function(
        "vector::f",
        "x",
        ContractType::BitVector {
            width: 8,
            signed: true,
        },
    );
    let accepted_signed = sidecar(
        "vector::f",
        64,
        "",
        r#"{"op":"eq","args":[{"result":0},{"bv":{"decimal":"-128","width":8,"signed":true}}]}"#,
    );
    attach_contracts(
        vec![input("contracts/f.json", &accepted_signed)],
        std::slice::from_ref(&signed_function),
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("minimum i8 literal is accepted");

    for literal in [
        r#"{"bv":{"decimal":"256","width":8,"signed":false}}"#,
        r#"{"bv":{"decimal":"-129","width":8,"signed":true}}"#,
        r#"{"bv":{"decimal":"-1","width":8,"signed":false}}"#,
        r#"{"bv":{"decimal":"01","width":8,"signed":false}}"#,
        r#"{"bv":{"decimal":"+1","width":8,"signed":false}}"#,
    ] {
        let raw = sidecar("vector::f", 64, "", literal);
        assert_error(
            attach_contracts(
                vec![input("contracts/f.json", &raw)],
                std::slice::from_ref(&signed_function),
                "x86_64-unknown-linux-gnu",
                64,
            ),
            ContractCode::Type,
        );
    }

    let width_outside_u8 = sidecar("vector::f", 255, "", r#"{"bool":true}"#).replace(
        "\"target_pointer_width\":255",
        "\"target_pointer_width\":300",
    );
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &width_outside_u8)],
            &[signed_function],
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Profile,
    );
}

#[test]
fn contract_limits_accept_the_boundary_and_reject_boundary_plus_one() {
    let function = scalar_function("vector::f", "x", unsigned(8));
    let at_clause_limit = sidecar(
        "vector::f",
        64,
        &repeat_clauses(r#"{"bool":true}"#, 63),
        r#"{"bool":true}"#,
    );
    attach_contracts(
        vec![input("contracts/f.json", &at_clause_limit)],
        std::slice::from_ref(&function),
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("64 clauses are accepted");
    let above_clause_limit = sidecar(
        "vector::f",
        64,
        &repeat_clauses(r#"{"bool":true}"#, 64),
        r#"{"bool":true}"#,
    );
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &above_clause_limit)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Limit,
    );

    let depth_32 = nested_not(31);
    let at_depth_limit = sidecar("vector::f", 64, "", &depth_32);
    attach_contracts(
        vec![input("contracts/f.json", &at_depth_limit)],
        std::slice::from_ref(&function),
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("expression depth 32 is accepted");
    let depth_33 = nested_not(32);
    let above_depth_limit = sidecar("vector::f", 64, "", &depth_33);
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &above_depth_limit)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Limit,
    );
    let far_above_depth_limit = sidecar("vector::f", 64, "", &nested_not(80));
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &far_above_depth_limit)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Limit,
    );

    let nodes_1024 = node_boundary_contract("vector::f", 62);
    attach_contracts(
        vec![input("contracts/f.json", &nodes_1024)],
        std::slice::from_ref(&function),
        "x86_64-unknown-linux-gnu",
        64,
    )
    .expect("1,024 expression nodes are accepted");
    let nodes_1025 = node_boundary_contract("vector::f", 63);
    assert_error(
        attach_contracts(
            vec![input("contracts/f.json", &nodes_1025)],
            std::slice::from_ref(&function),
            "x86_64-unknown-linux-gnu",
            64,
        ),
        ContractCode::Limit,
    );
}

#[test]
fn closure_node_limit_is_exact() {
    let mut functions = Vec::new();
    let mut contracts = Vec::new();
    for index in 0..8 {
        let function_id = format!("vector::f{index}");
        functions.push(scalar_function(&function_id, "x", unsigned(8)));
        contracts.push(input(
            &format!("contracts/f{index}.json"),
            &node_boundary_contract(&function_id, 62),
        ));
    }
    attach_contracts(contracts, &functions, "x86_64-unknown-linux-gnu", 64)
        .expect("8,192 closure nodes are accepted");

    let extra_id = "vector::extra";
    functions.push(scalar_function(extra_id, "x", unsigned(8)));
    let mut contracts = (0..8)
        .map(|index| {
            let function_id = format!("vector::f{index}");
            input(
                &format!("contracts/f{index}.json"),
                &node_boundary_contract(&function_id, 62),
            )
        })
        .collect::<Vec<_>>();
    contracts.push(input(
        "contracts/extra.json",
        &sidecar(extra_id, 64, "", r#"{"bool":true}"#),
    ));
    assert_error(
        attach_contracts(contracts, &functions, "x86_64-unknown-linux-gnu", 64),
        ContractCode::Limit,
    );
}

#[test]
fn exhausted_closure_node_budget_precedes_processing_the_next_contract() {
    let mut functions = Vec::new();
    let mut contracts = Vec::new();
    for index in 0..8 {
        let function_id = format!("vector::f{index}");
        functions.push(scalar_function(&function_id, "x", unsigned(8)));
        contracts.push(input(
            &format!("contracts/f{index}.json"),
            &node_boundary_contract(&function_id, 62),
        ));
    }
    functions.push(scalar_function("vector::z", "x", unsigned(8)));
    contracts.push(input(
        "contracts/z.json",
        &sidecar("vector::z", 64, "", r#"{"parameter":"missing"}"#),
    ));

    assert_error(
        attach_contracts(contracts, &functions, "x86_64-unknown-linux-gnu", 64),
        ContractCode::Limit,
    );
}

#[test]
fn closure_limit_precedes_an_already_observed_contract_specific_error() {
    let mut functions = vec![scalar_function("vector::a", "x", unsigned(8))];
    let mut contracts = vec![input(
        "contracts/a.json",
        &sidecar("vector::a", 64, "", r#"{"parameter":"missing"}"#),
    )];
    for index in 0..8 {
        let function_id = format!("vector::f{index}");
        functions.push(scalar_function(&function_id, "x", unsigned(8)));
        contracts.push(input(
            &format!("contracts/f{index}.json"),
            &node_boundary_contract(&function_id, 62),
        ));
    }
    functions.push(scalar_function("vector::z", "x", unsigned(8)));
    contracts.push(input(
        "contracts/z.json",
        &sidecar("vector::z", 64, "", r#"{"bool":true}"#),
    ));

    assert_error(
        attach_contracts(contracts, &functions, "x86_64-unknown-linux-gnu", 64),
        ContractCode::Limit,
    );
}

#[test]
fn whitespace_changes_only_raw_traceability_hash() {
    assert_eq!(
        without_json_formatting_whitespace(IDENTITY),
        without_json_formatting_whitespace(IDENTITY_WHITESPACE),
        "fixtures must differ only in JSON formatting whitespace"
    );
    let source = b"pub fn identity(x: i8) -> i8 { x }\n";
    let compact = rustc_harness::analyze_contracts(
        source,
        "vector::identity",
        &[("contracts/identity.json", IDENTITY)],
    )
    .expect("compact contract");
    let whitespace = rustc_harness::analyze_contracts(
        source,
        "vector::identity",
        &[("contracts/identity.json", IDENTITY_WHITESPACE)],
    )
    .expect("whitespace contract");
    let compact = &compact.contracts.contracts()[0];
    let whitespace = &whitespace.contracts.contracts()[0];
    assert_eq!(compact.contract_hash, whitespace.contract_hash);
    assert_eq!(compact.value, whitespace.value);
    assert_eq!(
        compact.canonical_json().unwrap(),
        whitespace.canonical_json().unwrap()
    );
    assert_ne!(compact.raw_input_sha256, whitespace.raw_input_sha256);
    assert_eq!(
        compact.contract_hash,
        "90ae02bbd45490ce69c1f2a2bf5917f7d578474991f6e93da2a2b90eff9919fc"
    );
}

fn without_json_formatting_whitespace(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect()
}

fn scalar_function(id: &str, parameter: &str, parameter_type: ContractType) -> ContractFunction {
    ContractFunction {
        function_id: id.to_owned(),
        parameter_names: vec![parameter.to_owned()],
        parameter_types: vec![parameter_type.clone()],
        result_type: parameter_type,
    }
}

fn unsigned(width: u8) -> ContractType {
    ContractType::BitVector {
        width,
        signed: false,
    }
}

fn input(path: &str, bytes: &str) -> ContractInput {
    ContractInput::new(path, bytes.as_bytes())
}

fn sidecar(function: &str, width: u8, requires: &str, ensures: &str) -> String {
    format!(
        r#"{{"schema":"mpk.rust.contract.v0","semantic_profile":"mpk.rust.checked.v0","target_pointer_width":{width},"function":"{function}","requires":[{requires}],"ensures":[{ensures}],"modifies":[],"panic":"forbidden","termination":"total","loops":[]}}"#
    )
}

fn repeat_clauses(expression: &str, count: usize) -> String {
    std::iter::repeat_n(expression, count)
        .collect::<Vec<_>>()
        .join(",")
}

fn nested_not(operators: usize) -> String {
    (0..operators).fold(r#"{"bool":true}"#.to_owned(), |expression, _| {
        format!(r#"{{"op":"not","args":[{expression}]}}"#)
    })
}

fn nary_true(arguments: usize) -> String {
    format!(
        r#"{{"op":"and","args":[{}]}}"#,
        repeat_clauses(r#"{"bool":true}"#, arguments)
    )
}

fn node_boundary_contract(function: &str, final_arguments: usize) -> String {
    let mut requires = (0..15).map(|_| nary_true(63)).collect::<Vec<_>>();
    requires.push(r#"{"bool":true}"#.to_owned());
    sidecar(
        function,
        64,
        &requires.join(","),
        &nary_true(final_arguments),
    )
}

fn assert_error<T: std::fmt::Debug>(result: Result<T, ContractError>, expected: ContractCode) {
    let error = result.expect_err("contract must reject");
    assert_eq!(error.code, expected, "error: {error:?}");
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

#[test]
fn driver_surfaces_contract_rejections_in_the_subset_phase() {
    let error = rustc_harness::analyze_contracts(b"pub fn f(x: u8) -> u8 { x }", "vector::f", &[])
        .expect_err("missing contract rejects");
    let RustcDriverError::Contract(error) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(error.code, ContractCode::Missing);
    assert_eq!(error.function_id.as_deref(), Some("vector::f"));
}
