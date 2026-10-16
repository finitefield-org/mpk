//! Source-sized endpoint arithmetic and ordinary two/three-index slicing.
use super::*;
mod arith;
mod assoc;
mod copying;
mod emit;
pub(super) fn install_copy(p: &mut Producer, trace: Option<&Path>) {
    copying::install(p, trace);
}
mod model;
mod numeric;
mod preservation;
mod readable;

fn rc(s: &str, args: Vec<Expr>) -> Expr {
    vc(&format!("Reslice{s}"), args)
}
fn rn(s: &str, d: usize, args: Vec<Expr>) -> Expr {
    rc(&format!("{s}{d}"), args)
}
fn maybe(p: &mut Producer, e: Typed, q: Typed) -> Typed {
    if substitute(&e.right, &q.left, &q.right) == e.right {
        e
    } else {
        change(p, e, q)
    }
}
pub(super) fn install(p: &mut Producer, _count: usize, trace: Option<&Path>) {
    arith::install(p, trace);
    assoc::install(p, trace);
    numeric::install(p, trace);
    model::install(p, trace);
    preservation::install(p, trace);
    readable::install(p, trace);
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}

pub(super) fn emit_copy(base: Certificate, output: &Path, count: usize) {
    copying::emit(base, output, count);
}

pub(super) fn install_construction(p: &mut Producer, trace: Option<&Path>) {
    copying::construction(p, trace);
}
pub(super) fn emit_construction(base: Certificate, output: &Path, count: usize) {
    copying::emit_construction(base, output, count);
}
