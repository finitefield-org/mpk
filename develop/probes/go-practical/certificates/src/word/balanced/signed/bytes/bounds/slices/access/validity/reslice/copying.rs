//! Whole-cell pre-state copy semantics, including overlapping local views.
use super::*;
mod counts;
mod emit;
mod shift;
mod store;

fn bc(s: &str, args: Vec<Expr>) -> Expr {
    rc(&format!("Bulk{s}"), args)
}
fn bn(s: &str, d: usize, args: Vec<Expr>) -> Expr {
    bc(&format!("{s}{d}"), args)
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    shift::install(p, trace);
    counts::install(p, trace);
    store::install(p, trace);
}

pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}
