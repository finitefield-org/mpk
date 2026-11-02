//! Computed min(len(dst),len(src)) and actual source index discharge.
use super::*;

pub(super) fn minimum(a: Expr, b: Expr) -> Expr {
    choose_t(word_ty(), word_less(WORD, a.clone(), b.clone()), a, b)
}
pub(super) fn count(d: Expr, s: Expr) -> Expr {
    minimum(canon(WORD, length(d)), canon(WORD, length(s)))
}
fn min_selected(p: &mut Producer, a: Expr, b: Expr, yes: bool, h: Expr) -> Typed {
    let m = minimum(a.clone(), b.clone());
    select(p, reflexive_t(word_ty(), m), word_less(WORD, a, b), yes, h)
}
fn min_bound(p: &mut Producer, left: bool) -> Typed {
    let (a, b) = (v("a"), v("b"));
    let m = minimum(a.clone(), b.clone());
    let bound = if left { a.clone() } else { b.clone() };
    let goal = numeric::le(m, bound);
    let mut proofs = vec![];
    for yes in [false, true] {
        let mut q = reflexive_t(bool_ty(), goal.clone());
        let chosen = min_selected(p, a.clone(), b.clone(), yes, v("__less"));
        q = change(p, q, chosen);
        let tail = match (left, yes) {
            (true, true) => numeric::reflexive_le(p, a.clone()),
            (false, false) => numeric::reflexive_le(p, b.clone()),
            (true, false) => {
                let mut t = reflexive_t(bool_ty(), numeric::le(b.clone(), a.clone()));
                t = change(
                    p,
                    t,
                    typed(
                        bool_ty(),
                        word_less(WORD, a.clone(), b.clone()),
                        literal(false),
                        v("__less"),
                    ),
                );
                let n = normalize(t.right.clone(), p);
                next(t, n)
            }
            (false, true) => numeric::less_le(a.clone(), b.clone(), v("__less")),
        };
        proofs.push(next(q, tail).proof);
    }
    typed(
        bool_ty(),
        goal.clone(),
        literal(true),
        cases(
            word_less(WORD, a, b),
            "__less",
            eq(goal, literal(true)),
            proofs[0].clone(),
            proofs[1].clone(),
        ),
    )
}
pub(super) fn bound(d: Expr, s: Expr, left: bool) -> Typed {
    let (a, b) = (canon(WORD, length(d)), canon(WORD, length(s)));
    typed(
        bool_ty(),
        numeric::le(
            minimum(a.clone(), b.clone()),
            if left { a.clone() } else { b.clone() },
        ),
        literal(true),
        bc(if left { "CountLeft" } else { "CountRight" }, vec![a, b]),
    )
}
pub(super) fn sign_canon(x: Expr) -> Typed {
    typed(
        bool_ty(),
        sign_bit(WORD, canon(WORD, x.clone())),
        sign_bit(WORD, x.clone()),
        apply(canon_bits(WORD, x), vec![literal(true); WORD]),
    )
}
fn valid_data() -> Vec<(String, Expr)> {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    vec![
        ("descriptor".into(), descriptor()),
        ("storage_length".into(), word_ty()),
        ("live_mask".into(), cube_type(1)),
        ("valid".into(), eq(valid(d, n, live), literal(true))),
    ]
}
pub(super) fn nil_zero(
    p: &mut Producer,
    d: Expr,
    n: Expr,
    live: Expr,
    hv: Expr,
    hn: Expr,
) -> Typed {
    let q = selected_valid(p, d.clone(), n, live, hv, true, hn);
    let fs = facts(p, &nil_clauses(d.clone()), q.proof);
    let raw = length(d);
    let z = literal_word(WORD, 0);
    let q = canon_congr(
        WORD,
        raw.clone(),
        z.clone(),
        equal_bits(WORD, raw, z, fs[0].proof.clone()),
    );
    next(q, arith::canonical_zero(WORD))
}
fn nonnegative_proof(p: &mut Producer) -> Typed {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    let lhs = sign_bit(WORD, canon(WORD, length(d.clone())));
    let mut yes = reflexive_t(bool_ty(), lhs.clone());
    let z = nil_zero(
        p,
        d.clone(),
        n.clone(),
        live.clone(),
        v("valid"),
        v("__nil"),
    );
    yes = change(p, yes, z);
    let tail = normalize(yes.right.clone(), p);
    yes = next(yes, tail);
    let fs = nonnil_facts(p, d.clone(), n, live, v("valid"), v("__nil"));
    let raw = not_true(p, sign_bit(WORD, length(d.clone())), fs[2].proof.clone());
    let no = next(sign_canon(length(d.clone())), raw);
    typed(
        bool_ty(),
        lhs.clone(),
        literal(false),
        cases(
            nil(d),
            "__nil",
            eq(lhs, literal(false)),
            no.proof,
            yes.proof,
        ),
    )
}
pub(super) fn length_nn(d: Expr, n: Expr, live: Expr, hv: Expr) -> Typed {
    typed(
        bool_ty(),
        sign_bit(WORD, canon(WORD, length(d.clone()))),
        literal(false),
        bc("LengthNonnegative", vec![d, n, live, hv]),
    )
}
pub(super) fn data() -> Vec<(String, Expr)> {
    let mut a = vec![
        ("destination".into(), descriptor()),
        ("source".into(), descriptor()),
        ("storage_length".into(), word_ty()),
        ("live_mask".into(), cube_type(1)),
    ];
    for (k, d) in [
        ("destination_valid", v("destination")),
        ("source_valid", v("source")),
    ] {
        a.push((
            k.into(),
            eq(valid(d, v("storage_length"), v("live_mask")), literal(true)),
        ));
    }
    a
}
pub(super) fn access_data() -> Vec<(String, Expr)> {
    let mut a = data();
    a.push(("index".into(), word_ty()));
    a.push((
        "copied_guard".into(),
        eq(
            active(v("index"), count(v("destination"), v("source"))),
            literal(true),
        ),
    ));
    a
}
fn access_proof(p: &mut Producer, left: bool) -> Typed {
    let dst = v("destination");
    let src = v("source");
    let i = v("index");
    let d = if left { dst.clone() } else { src.clone() };
    let hv = v(if left {
        "destination_valid"
    } else {
        "source_valid"
    });
    let m = count(dst.clone(), src.clone());
    let ci = canon(WORD, i.clone());
    let hn = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), m.clone()),
        v("copied_guard"),
        false,
    );
    let hi = not_true(p, sign_bit(WORD, ci.clone()), hn.proof);
    let lt = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), m.clone()),
        v("copied_guard"),
        true,
    );
    let le = bound(dst, src, left);
    let nn = length_nn(d.clone(), v("storage_length"), v("live_mask"), hv.clone());
    let rev = not_true(
        p,
        word_less(WORD, canon(WORD, length(d.clone())), m.clone()),
        le.proof,
    );
    let lt = mixed_transitive(
        WORD,
        ci.clone(),
        m,
        canon(WORD, length(d.clone())),
        lt.proof,
        rev.proof,
    );
    // A nil descriptor has zero length, so a copied index cannot be nil.
    let z = nil_zero(
        p,
        d.clone(),
        v("storage_length"),
        v("live_mask"),
        hv,
        v("__nil"),
    );
    let bad = numeric::change_left(p, lt.clone(), z);
    let no = less_zero(WORD, ci.clone());
    let impossible_nil = impossible(
        bool_ty(),
        nil(d.clone()),
        literal(false),
        contradiction(bad, no.proof),
    );
    let nil_false = typed(
        bool_ty(),
        nil(d.clone()),
        literal(false),
        cases(
            nil(d.clone()),
            "__nil",
            eq(nil(d.clone()), literal(false)),
            v("__nil"),
            impossible_nil.proof,
        ),
    );
    let raw = length(d.clone());
    let ce = canon_equal(WORD, raw.clone());
    let transport = substitute_equal(
        WORD,
        false,
        true,
        raw.clone(),
        canon(WORD, raw.clone()),
        ci,
        ce.proof,
    );
    let lt = next(transport, lt);
    let nnraw = next(backwards(sign_canon(raw.clone())), nn);
    let mut q = index_def(d, i);
    q = change(
        p,
        q,
        signed_nonnegative(raw.clone(), v("index"), nnraw.proof),
    );
    for f in [nil_false, hi, lt] {
        q = maybe(p, q, f);
    }
    let n = normalize(q.right.clone(), p);
    q = next(q, n);
    assert_eq!(q.right, literal(true));
    q
}
pub(super) fn access(
    left: bool,
    d: Expr,
    s: Expr,
    storage: [Expr; 2],
    hv: [Expr; 2],
    i: Expr,
    hg: Expr,
) -> Typed {
    let target = if left { d.clone() } else { s.clone() };
    typed(
        bool_ty(),
        defined(target, i.clone()),
        literal(true),
        bc(
            if left {
                "DestinationIndex"
            } else {
                "SourceIndex"
            },
            vec![
                d,
                s,
                storage[0].clone(),
                storage[1].clone(),
                hv[0].clone(),
                hv[1].clone(),
                i,
                hg,
            ],
        ),
    )
}
pub(super) fn metadata() -> Typed {
    let d = v("destination");
    let s = v("source");
    let m = count(d.clone(), s.clone());
    typed(
        bool_ty(),
        both(&[
            not(sign_bit(WORD, m)),
            bound(d.clone(), s.clone(), true).left,
            bound(d, s, false).left,
        ]),
        literal(true),
        bc("Metadata", data().iter().map(|(n, _)| v(n)).collect()),
    )
}
fn metadata_proof(p: &mut Producer) -> Typed {
    let d = v("destination");
    let s = v("source");
    let m = count(d.clone(), s.clone());
    let l = bound(d.clone(), s.clone(), true);
    let r = bound(d.clone(), s, false);
    let nn = length_nn(
        d.clone(),
        v("storage_length"),
        v("live_mask"),
        v("destination_valid"),
    );
    let nn = numeric::nonnegative(m, canon(WORD, length(d)), l.proof.clone(), nn.proof);
    let mut q = reflexive_t(bool_ty(), metadata().left);
    for f in [nn, l, r] {
        q = change(p, q, f);
    }
    let n = normalize(q.right.clone(), p);
    next(q, n)
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let a = vec![("a".into(), word_ty()), ("b".into(), word_ty())];
    for left in [true, false] {
        let q = min_bound(p, left);
        theorem(
            p,
            if left {
                "SliceAccessValidityResliceBulkCountLeft"
            } else {
                "SliceAccessValidityResliceBulkCountRight"
            },
            &a,
            q,
        );
    }
    let q = nonnegative_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkLengthNonnegative",
        &valid_data(),
        q,
    );
    for left in [true, false] {
        let q = access_proof(p, left);
        theorem(
            p,
            if left {
                "SliceAccessValidityResliceBulkDestinationIndex"
            } else {
                "SliceAccessValidityResliceBulkSourceIndex"
            },
            &access_data(),
            q,
        );
    }
    let q = metadata_proof(p);
    theorem(p, "SliceAccessValidityResliceBulkMetadata", &data(), q);
    save_trace(p, trace, "copy-count");
}
