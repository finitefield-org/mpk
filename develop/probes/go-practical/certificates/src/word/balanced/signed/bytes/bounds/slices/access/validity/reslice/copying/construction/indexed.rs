//! Actual active-index facts and computed descriptor validity, including nil.
use super::*;

pub(super) fn active_facts(
    p: &mut Producer,
    d: Expr,
    n: Expr,
    live: Expr,
    hv: Expr,
    i: Expr,
    hg: Expr,
) -> (Typed, Typed) {
    let (_, guard) = index_facts(p, d.clone(), i.clone(), hg);
    let nn = counts::length_nn(d.clone(), n, live, hv);
    let nn = next(backwards(counts::sign_canon(length(d.clone()))), nn);
    let q = next(
        backwards(signed_nonnegative(length(d.clone()), i.clone(), nn.proof)),
        guard,
    );
    let ci = canon(WORD, i.clone());
    let hn = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), length(d.clone())),
        q.proof.clone(),
        false,
    );
    let hn = not_true(p, sign_bit(WORD, ci.clone()), hn.proof);
    let lt = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), length(d.clone())),
        q.proof,
        true,
    );
    let raw = length(d);
    let e = canon_equal(WORD, raw.clone());
    let t = substitute_equal(
        WORD,
        false,
        true,
        raw.clone(),
        canon(WORD, raw),
        ci,
        e.proof,
    );
    (hn, next(backwards(t), lt))
}
fn data() -> Vec<(String, Expr)> {
    let (d, n, live, i) = (
        v("descriptor"),
        v("storage_length"),
        v("live_mask"),
        v("index"),
    );
    vec![
        ("descriptor".into(), descriptor()),
        ("storage_length".into(), word_ty()),
        ("live_mask".into(), cube_type(1)),
        ("valid".into(), eq(valid(d.clone(), n, live), literal(true))),
        ("index".into(), word_ty()),
        (
            "active".into(),
            eq(active(i, canon(WORD, length(d))), literal(true)),
        ),
    ]
}
pub(super) fn lift(d: Expr, storage: [Expr; 2], hv: Expr, i: Expr, ha: Expr) -> Typed {
    typed(
        bool_ty(),
        defined(d.clone(), i.clone()),
        literal(true),
        nc(
            "ActiveIndex",
            vec![d, storage[0].clone(), storage[1].clone(), hv, i, ha],
        ),
    )
}
fn proof(p: &mut Producer) -> Typed {
    let (d, n, live, i) = (
        v("descriptor"),
        v("storage_length"),
        v("live_mask"),
        v("index"),
    );
    let ci = canon(WORD, i.clone());
    let cl = canon(WORD, length(d.clone()));
    let hn = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), cl.clone()),
        v("active"),
        false,
    );
    let hn = not_true(p, sign_bit(WORD, ci.clone()), hn.proof);
    let lt = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), cl.clone()),
        v("active"),
        true,
    );
    let z = counts::nil_zero(
        p,
        d.clone(),
        n.clone(),
        live.clone(),
        v("valid"),
        v("__nil"),
    );
    let bad = numeric::change_left(p, lt.clone(), z);
    let no = less_zero(WORD, ci.clone());
    let bad = impossible(
        bool_ty(),
        nil(d.clone()),
        literal(false),
        contradiction(bad, no.proof),
    );
    let hnil = typed(
        bool_ty(),
        nil(d.clone()),
        literal(false),
        cases(
            nil(d.clone()),
            "__nil",
            eq(nil(d.clone()), literal(false)),
            v("__nil"),
            bad.proof,
        ),
    );
    let nn = counts::length_nn(d.clone(), n, live, v("valid"));
    let nn = next(backwards(counts::sign_canon(length(d.clone()))), nn);
    let raw = length(d.clone());
    let ce = canon_equal(WORD, raw.clone());
    let e = substitute_equal(WORD, false, true, raw, cl, ci, ce.proof);
    let lt = next(e, lt);
    let mut q = index_def(d.clone(), i.clone());
    q = change(p, q, signed_nonnegative(length(d), i, nn.proof));
    for f in [hnil, hn, lt] {
        q = maybe(p, q, f);
    }
    let q = finish(p, q);
    assert_eq!(q.right, literal(true));
    q
}
pub(super) fn full_valid(d: Expr, n: Expr, live: Expr, hv: Expr) -> Typed {
    typed(
        bool_ty(),
        valid(full(d.clone()), n.clone(), live.clone()),
        literal(true),
        vc("Full", vec![d, n, live, hv]),
    )
}
pub(super) fn cap_nn(p: &mut Producer, d: Expr, n: Expr, live: Expr, hv: Expr) -> Typed {
    let fv = full_valid(d.clone(), n.clone(), live.clone(), hv);
    let q = counts::length_nn(full(d.clone()), n, live, fv.proof);
    numeric::change_left(p, q, view_projection(d, 3, true))
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let q = proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionActiveIndex",
        &data(),
        q,
    );
    save_trace(p, trace, "construct-active-index");
}
