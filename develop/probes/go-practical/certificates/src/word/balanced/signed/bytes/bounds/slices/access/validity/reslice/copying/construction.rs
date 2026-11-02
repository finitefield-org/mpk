//! Fresh initialized allocation and non-reallocating append over whole cells.
use super::*;
mod appended;
mod indexed;
mod made;
fn nc(s: &str, a: Vec<Expr>) -> Expr {
    bc(&format!("Construction{s}"), a)
}
fn scalar(w: &numeric::Word) -> Typed {
    typed(
        word_ty(),
        canon(WORD, w.value.clone()),
        w.value.clone(),
        w.normal.clone(),
    )
}
fn finish(p: &mut Producer, q: Typed) -> Typed {
    let n = normalize(q.right.clone(), p);
    next(q, n)
}
fn zero_left(x: numeric::Word, is_carry: bool) -> Typed {
    let z = literal_word(WORD, 0);
    let mut q = arith::commute(WORD, z, x.value.clone(), literal(false), is_carry);
    q = next(
        q,
        if is_carry {
            arith::zero_carry(WORD, x.value.clone())
        } else {
            arith::zero_sum(WORD, x.value.clone())
        },
    );
    if !is_carry {
        q = next(q, scalar(&x));
    }
    q
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    made::install(p, trace);
    indexed::install(p, trace);
    appended::install(p, trace);
    made::install_composed(p, trace);
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    appended::emit(base, output, count);
}
