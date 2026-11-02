//! Addition/subtraction cancellation over all source bits, without eta.
use super::*;

fn not_twice(d: usize, x: Expr) -> Typed {
    typed(
        cube_type(d),
        arith::bit_not(d, arith::bit_not(d, x.clone())),
        canon(d, x.clone()),
        bn("NotTwice", d, vec![x]),
    )
}
fn proof(p: &mut Producer, d: usize) -> Typed {
    let x = v("word");
    let mut q = arith::not_def(d, arith::bit_not(d, x.clone()));
    if d == 0 {
        q = change(p, q, arith::not_def(0, x.clone()));
        q = next(
            q,
            typed(
                bool_ty(),
                not(not(x.clone())),
                x.clone(),
                rc("NotTwice", vec![x.clone()]),
            ),
        );
    } else {
        for high in [false, true] {
            q = change(p, q, arith::not_part(d, x.clone(), high));
        }
        for high in [false, true] {
            q = change(p, q, not_twice(d - 1, project(x.clone(), high)));
        }
    }
    next(q, backwards(canon_def(d, x)))
}
pub(super) fn undo(x: numeric::Word, y: numeric::Word) -> Typed {
    typed(
        word_ty(),
        arith::difference(
            WORD,
            sum(WORD, x.value.clone(), y.value.clone(), literal(false)),
            x.value.clone(),
        ),
        y.value.clone(),
        bc("Undo", vec![x.value, y.value, x.normal, y.normal]),
    )
}
fn undo_proof(p: &mut Producer) -> Typed {
    let x = v("offset");
    let y = v("index");
    let nx = arith::bit_not(WORD, x.clone());
    let mut q = arith::inverse(WORD, y.clone(), nx.clone(), literal(false), false);
    q = numeric::change_left(p, q, not_twice(WORD, x.clone()));
    q = numeric::change_left(
        p,
        q,
        typed(
            word_ty(),
            canon(WORD, x.clone()),
            x.clone(),
            v("offset_canonical"),
        ),
    );
    q = numeric::change_left(
        p,
        q,
        arith::commute(WORD, y.clone(), x.clone(), literal(false), false),
    );
    let c = normalize(not(literal(false)), p);
    q = numeric::change_left(p, q, c);
    next(
        q,
        typed(word_ty(), canon(WORD, y.clone()), y, v("index_canonical")),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    for d in 0..=WORD {
        let q = proof(p, d);
        theorem(
            p,
            &format!("SliceAccessValidityResliceBulkNotTwice{d}"),
            &[("word".into(), cube_type(d))],
            q,
        );
    }
    let x = v("offset");
    let y = v("index");
    let args = vec![
        ("offset".into(), word_ty()),
        ("index".into(), word_ty()),
        (
            "offset_canonical".into(),
            eq_ty(word_ty(), canon(WORD, x.clone()), x),
        ),
        (
            "index_canonical".into(),
            eq_ty(word_ty(), canon(WORD, y.clone()), y),
        ),
    ];
    let q = undo_proof(p);
    theorem(p, "SliceAccessValidityResliceBulkUndo", &args, q);
    save_trace(p, trace, "copy-shift");
}
