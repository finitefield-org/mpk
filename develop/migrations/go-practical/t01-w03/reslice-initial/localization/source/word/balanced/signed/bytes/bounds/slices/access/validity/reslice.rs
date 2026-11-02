//! Source-sized endpoint arithmetic and ordinary two/three-index slicing.
use super::*;
mod arith;
mod assoc;
mod numeric;
mod model;

fn rc(s: &str, args: Vec<Expr>) -> Expr {
    vc(&format!("Reslice{s}"), args)
}
fn rn(s: &str, d: usize, args: Vec<Expr>) -> Expr {
    rc(&format!("{s}{d}"), args)
}
fn maybe(p: &mut Producer, e: Typed, q: Typed) -> Typed {
    if substitute(&e.right, &q.left, &q.right) == e.right { e } else { change(p, e, q) }
}
pub(super) fn install(p: &mut Producer, _count: usize, trace: Option<&Path>) {
    arith::install(p, trace);
    assoc::install(p, trace);
    numeric::install(p, trace);
    model::install(p, trace);
}
