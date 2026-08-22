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

use rustc_driver_adapter::{HirCheckCode, RustcDriverError};

#[test]
fn accepted_hir_profile_records_stable_function_and_parameter_names() {
    let source = br##"#![no_std]
const N: usize = 2;
pub struct Pair { pub left: u32, right: u32 }
pub fn helper(value: u32) -> u32 { let _length = N; value }
pub fn f(input: u32, values: [u8; N]) -> Pair {
    let _values = values;
    Pair { left: helper(input), right: 0 }
}
"##;
    let analysis = rustc_harness::analyze(source, "vector::f").expect("accepted HIR profile");
    assert_eq!(analysis.selected_function, "vector::f");
    assert_eq!(
        analysis
            .call_closure
            .iter()
            .map(|function| {
                (
                    function.function_id.clone(),
                    function.parameter_names.clone(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                "vector::f".to_owned(),
                vec!["input".to_owned(), "values".to_owned()]
            ),
            ("vector::helper".to_owned(), vec!["value".to_owned()]),
        ]
    );
}

#[test]
fn module_paths_use_stable_canonical_function_ids() {
    let source = br##"
mod arithmetic {
    pub fn increment(value: u32) -> u32 { value + 1 }
}
pub fn f(value: u32) -> u32 { crate::arithmetic::increment(value) }
"##;
    let analysis = rustc_harness::analyze(source, "vector::f").expect("accepted module call");
    assert_eq!(
        analysis
            .call_closure
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<Vec<_>>(),
        ["vector::arithmetic::increment", "vector::f"]
    );
}

#[test]
fn complete_primitive_aggregate_and_control_flow_profile_is_accepted() {
    let source = br##"
pub struct Values {
    b: bool, i8v: i8, i16v: i16, i32v: i32, i64v: i64,
    u8v: u8, u16v: u16, u32v: u32, u64v: u64, iv: isize,
    uv: usize, a: [u8; 2]
}
fn inc(x: u32) -> u32 { x + 1 }
pub fn f(x: u32, enabled: bool) -> u32 {
    let mut y: u32 = x;
    if enabled { y = inc(y); } else { return x; }
    y
}
"##;
    rustc_harness::analyze(source, "vector::f").expect("accepted type and control-flow profile");
}

#[test]
fn complete_primitive_operation_matrix_is_accepted() {
    for source in [
        "pub struct P { v: u32 } pub fn f(a: u32, b: u32, i: usize, q: bool, r: bool) -> u32 { let x = (a + b) - (a * b); let y = (x / b) % (b | 1); let z = (y & b) | (y ^ b); let s = (z << i) >> i; let p = P { v: s }; let xs = [p.v, a]; if (!q && r) || q { xs[i] } else { 0 } }",
        "pub fn f(a: i32, b: i32, s: i32) -> bool { let x = -a; ((x >> s) < b) && (a != b) }",
        "pub fn f(a: u32, s: i16, w: u64, q: bool) -> u32 { let x = a << s; let y = a >> w; if q { x } else { y } }",
        "pub fn f(x: i8) -> i8 { let _y = x; -128_i8 }",
    ] {
        rustc_harness::analyze(source.as_bytes(), "vector::f")
            .unwrap_or_else(|error| panic!("source: {source}; error: {error:?}"));
    }
}

#[test]
fn closed_hir_rejection_codes_match_the_frozen_vectors() {
    for (source, expected) in [
        (
            "enum E { A } pub fn f(x: u8) -> u8 { x }",
            HirCheckCode::Item,
        ),
        (
            "pub unsafe fn f(x: u8) -> u8 { x }",
            HirCheckCode::FunctionKind,
        ),
        ("pub fn f<T>(_: T) -> u8 { 0 }", HirCheckCode::Generic),
        (
            "trait T {} pub fn f(x: u8) -> u8 { x }",
            HirCheckCode::Trait,
        ),
        (
            "struct S; impl S {} pub fn f(x: u8) -> u8 { x }",
            HirCheckCode::Impl,
        ),
        (
            "static X: u8 = 1; pub fn f(x: u8) -> u8 { let _y = x; X }",
            HirCheckCode::Static,
        ),
        ("pub fn f(x: &u8) -> u8 { *x }", HirCheckCode::Type),
        ("pub fn f(x: String) -> String { x }", HirCheckCode::Drop),
        (
            "pub fn f((x, y): (u8, u8)) -> u8 { x + y }",
            HirCheckCode::Pattern,
        ),
        (
            "pub fn f(mut x: u8) -> u8 { x = 1; x }",
            HirCheckCode::Pattern,
        ),
        (
            "pub fn f(x: u8) -> u8 { let x = x; x }",
            HirCheckCode::Binding,
        ),
        (
            "pub fn f(mut x: u8) -> u8 { while x > 0 { x = x - 1; } x }",
            HirCheckCode::ControlFlow,
        ),
        (
            "struct S { x: u8 } pub fn f(mut s: S) -> S { s.x = 1; s }",
            HirCheckCode::Mutation,
        ),
        (
            "pub fn f(x: u8) -> u16 { x as u16 }",
            HirCheckCode::Operation,
        ),
        (
            "pub fn f(a: [u8; 2], b: [u8; 2]) -> bool { a == b }",
            HirCheckCode::Operation,
        ),
    ] {
        assert_eq!(
            rustc_harness::analyze(source.as_bytes(), "vector::f"),
            Err(RustcDriverError::Subset(expected)),
            "source: {source}"
        );
    }
}

#[test]
fn purity_rejects_external_calls_and_function_values() {
    for source in [
        "pub fn f(x: u8) -> u8 { core::cmp::max(x, 1) }",
        "fn helper(x: u8) -> u8 { x } pub fn f(x: u8) -> usize { let _x = x; helper as usize }",
    ] {
        let actual = rustc_harness::analyze(source.as_bytes(), "vector::f");
        assert!(
            matches!(
                actual,
                Err(RustcDriverError::Subset(
                    HirCheckCode::Call | HirCheckCode::Operation
                ))
            ),
            "source: {source}; actual: {actual:?}"
        );
    }
}

#[test]
fn restricted_visibility_is_rejected_by_the_source_gate_before_hir() {
    let error = rustc_harness::analyze(b"pub(crate) fn f(x: u8) -> u8 { x }", "vector::f")
        .expect_err("restricted visibility must reject");
    let RustcDriverError::Source(error) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(error.code.as_str(), "RUST_SUBSET_VISIBILITY");
}

#[test]
fn explicit_rust_abi_is_rejected_as_an_extern_function() {
    let error = rustc_harness::analyze(b"pub extern \"Rust\" fn f(x: u8) -> u8 { x }", "vector::f")
        .expect_err("explicit Rust ABI must still reject as extern");
    let RustcDriverError::Source(error) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(error.code.as_str(), "RUST_SUBSET_FUNCTION_KIND");
    assert_eq!(error.code.phase(), "subset");
}

#[test]
fn aggregate_limits_have_the_frozen_limit_code() {
    let array_at_limit = "pub fn f(value: [u8; 256]) -> [u8; 256] { value }";
    rustc_harness::analyze(array_at_limit.as_bytes(), "vector::f")
        .expect("256 array elements are accepted");

    let array = "pub fn f(value: [u8; 257]) -> [u8; 257] { value }";
    assert_eq!(
        rustc_harness::analyze(array.as_bytes(), "vector::f"),
        Err(RustcDriverError::Subset(HirCheckCode::AggregateLimit))
    );

    let fields_at_limit = (0..64)
        .map(|index| format!("f{index}: u8"))
        .collect::<Vec<_>>()
        .join(",");
    let structure_at_limit =
        format!("pub struct S {{ {fields_at_limit} }} pub fn f(value: S) -> S {{ value }}");
    rustc_harness::analyze(structure_at_limit.as_bytes(), "vector::f")
        .expect("64 struct fields are accepted");

    let fields = (0..65)
        .map(|index| format!("f{index}: u8"))
        .collect::<Vec<_>>()
        .join(",");
    let structure = format!("pub struct S {{ {fields} }} pub fn f(value: S) -> S {{ value }}");
    assert_eq!(
        rustc_harness::analyze(structure.as_bytes(), "vector::f"),
        Err(RustcDriverError::Subset(HirCheckCode::AggregateLimit))
    );

    let at_depth = format!(
        "pub fn f(value: {}) -> {} {{ value }}",
        nested_array_type(16),
        nested_array_type(16)
    );
    rustc_harness::analyze(at_depth.as_bytes(), "vector::f")
        .expect("16 aggregate levels are accepted");

    let nested = format!(
        "pub fn f(value: {}) -> {} {{ value }}",
        nested_array_type(17),
        nested_array_type(17)
    );
    assert_eq!(
        rustc_harness::analyze(nested.as_bytes(), "vector::f"),
        Err(RustcDriverError::Subset(HirCheckCode::AggregateLimit))
    );
}

#[test]
fn source_type_shapes_reject_computed_lengths_and_external_primitive_paths() {
    for source in [
        "const N: u8 = 2; pub fn f(value: [u8; N as usize]) -> [u8; N as usize] { value }",
        "pub fn f(value: std::primitive::u8) -> u8 { value }",
    ] {
        assert_eq!(
            rustc_harness::analyze(source.as_bytes(), "vector::f"),
            Err(RustcDriverError::Subset(HirCheckCode::Type)),
            "source: {source}"
        );
    }
}

fn nested_array_type(levels: usize) -> String {
    (0..levels).fold("u8".to_owned(), |element, _| format!("[{element}; 1]"))
}
