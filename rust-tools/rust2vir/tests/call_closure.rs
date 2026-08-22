#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;
#[path = "support/rustc_harness.rs"]
mod rustc_harness;

use rust2vir_internal::call_closure::{resolve_call_closure, CallClosureError};
use rustc_driver_adapter::{HirCheckCode, RustcDriverError};

#[test]
fn source_dead_calls_remain_in_the_compiler_resolved_closure() {
    let source = br##"
fn dead(x: u8) -> u8 { x }
fn helper(x: u8) -> u8 { x }
pub fn f(x: u8) -> u8 { if false { dead(x) } else { helper(x) } }
"##;
    let analysis = rustc_harness::analyze(source, "vector::f").expect("acyclic closure");
    assert_eq!(
        analysis
            .call_closure
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<Vec<_>>(),
        ["vector::dead", "vector::f", "vector::helper"]
    );
}

#[test]
fn source_dead_recursive_cycles_are_rejected_before_mir_dce() {
    let source = br##"
fn a(x: u8) -> u8 { if false { b(x) } else { x } }
fn b(x: u8) -> u8 { a(x) }
pub fn f(x: u8) -> u8 { a(x) }
"##;
    assert_eq!(
        rustc_harness::analyze(source, "vector::f"),
        Err(RustcDriverError::Subset(HirCheckCode::Call))
    );
}

#[test]
fn graph_resolution_is_deterministic_and_rejects_cycles_and_unknown_edges() {
    let graph = [
        ("vector::f", vec!["vector::z", "vector::a"]),
        ("vector::z", vec![]),
        ("vector::a", vec![]),
        ("vector::unused", vec![]),
    ];
    assert_eq!(
        resolve_call_closure(graph, "vector::f", 128),
        Ok(vec![
            "vector::a".to_owned(),
            "vector::f".to_owned(),
            "vector::z".to_owned(),
        ])
    );
    assert_eq!(
        resolve_call_closure([("f", vec!["missing"])], "f", 128),
        Err(CallClosureError::UnknownCallee)
    );
    assert_eq!(
        resolve_call_closure([("f", vec!["g"]), ("g", vec!["f"])], "f", 128),
        Err(CallClosureError::Cycle)
    );
}

#[test]
fn closure_limit_counts_only_compiler_reachable_functions() {
    assert_eq!(
        resolve_call_closure(
            [("f", vec!["a"]), ("a", vec![]), ("unused", vec![])],
            "f",
            1,
        ),
        Err(CallClosureError::Limit)
    );
    assert_eq!(
        resolve_call_closure([("f", Vec::<&str>::new()), ("unused", vec![])], "f", 1),
        Ok(vec!["f".to_owned()])
    );

    let at_limit = linear_graph(128);
    assert_eq!(
        resolve_call_closure(at_limit.clone(), "f000", 128)
            .expect("128 reachable functions are accepted")
            .len(),
        128
    );
    let above_limit = linear_graph(129);
    assert_eq!(
        resolve_call_closure(above_limit, "f000", 128),
        Err(CallClosureError::Limit)
    );
}

fn linear_graph(count: usize) -> Vec<(String, Vec<String>)> {
    (0..count)
        .map(|index| {
            let function = format!("f{index:03}");
            let callees = if index + 1 < count {
                vec![format!("f{:03}", index + 1)]
            } else {
                Vec::new()
            };
            (function, callees)
        })
        .collect()
}
