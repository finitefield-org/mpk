//! Checked byte prefix/suffix relations and the pinned length/slice/equality shape.
use super::*;
mod emit;

fn af(s: &str, args: Vec<Expr>) -> Expr {
    sc(&format!("Affix{s}"), args)
}
fn name_af(s: &str) -> String {
    suffix(&format!("Affix{s}"))
}
fn label(suffix: bool) -> &'static str {
    if suffix {
        "Suffix"
    } else {
        "Prefix"
    }
}
fn words_affix() -> (numeric::Word, numeric::Word) {
    (
        numeric::source(v("source_length")),
        numeric::source(v("pattern_length")),
    )
}
fn base_args() -> Vec<(String, Expr)> {
    let (l, n) = words_affix();
    vec![
        ("source_length".into(), word_ty()),
        ("source".into(), store_ty()),
        ("pattern_length".into(), word_ty()),
        ("pattern".into(), store_ty()),
        (
            "source_nonnegative".into(),
            eq(sign_bit(WORD, l.value), literal(false)),
        ),
        (
            "pattern_nonnegative".into(),
            eq(sign_bit(WORD, n.value), literal(false)),
        ),
    ]
}
fn guard() -> Expr {
    let (l, n) = words_affix();
    numeric::le(n.value, l.value)
}
fn old_guard() -> Expr {
    let (l, n) = words_affix();
    model::signed_le(n.value, l.value)
}
fn start(suf: bool) -> numeric::Word {
    let (l, n) = words_affix();
    if suf {
        numeric::difference(l.value, n.value)
    } else {
        numeric::zero()
    }
}
fn end(suf: bool) -> Expr {
    v(if suf {
        "source_length"
    } else {
        "pattern_length"
    })
}
fn raw_affix(suf: bool, source: Expr) -> Expr {
    if !suf {
        source
    } else {
        lambda(
            "__affix_raw_index",
            word_ty(),
            read_byte(
                source,
                sum(
                    WORD,
                    start(true).value,
                    v("__affix_raw_index"),
                    literal(false),
                ),
            ),
        )
    }
}
fn raw_read(suf: bool, source: Expr, i: Expr) -> Typed {
    let index = if suf {
        sum(
            WORD,
            start(true).value,
            canon(WORD, i.clone()),
            literal(false),
        )
    } else {
        i.clone()
    };
    let right = read_byte(source.clone(), index);
    typed(
        byte(),
        read_byte(raw_affix(suf, source), i),
        right.clone(),
        refl_t(byte(), right),
    )
}
fn cells(d: usize, n: Expr, source: Expr, pattern: Expr) -> Expr {
    let bits = addresses(d, "__affix_cell_");
    let i = index_word(&bits);
    cube_lambda(
        d,
        "__affix_cell_",
        word_eq(
            BYTE,
            read_byte(mask(n.clone(), source), i.clone()),
            read_byte(mask(n, pattern), i),
        ),
    )
}
fn relation(suf: bool, d: usize, source: Expr, pattern: Expr) -> Expr {
    af(
        &format!("{}{d}", label(suf)),
        vec![v("source_length"), source, v("pattern_length"), pattern],
    )
}
fn relation_body(suf: bool, d: usize, source: Expr, pattern: Expr) -> Expr {
    let (_, n) = words_affix();
    and(
        guard(),
        observer(
            d,
            false,
            n.value.clone(),
            raw_affix(suf, source),
            n.value,
            pattern,
        ),
    )
}
fn algorithm(suf: bool, d: usize, source: Expr, pattern: Expr) -> Expr {
    af(
        &format!("{}Source{d}", label(suf)),
        vec![v("source_length"), source, v("pattern_length"), pattern],
    )
}
fn subst_expr(mut e: Expr, env: &[(&str, Expr)]) -> Expr {
    for (k, x) in env {
        e = substitute(&e, &v(k), x);
    }
    e
}
fn subst_t(e: Typed, env: &[(&str, Expr)]) -> Typed {
    typed(
        subst_expr(e.ty, env),
        subst_expr(e.left, env),
        subst_expr(e.right, env),
        subst_expr(e.proof, env),
    )
}
fn sub_env(suf: bool, source: Expr, i: Expr, rg: Expr, ig: Expr) -> Vec<(&'static str, Expr)> {
    vec![
        ("length", v("source_length")),
        ("lower", start(suf).value),
        ("upper", end(suf)),
        ("source", source),
        ("index", i),
        ("range_guard", rg),
        ("index_guard", ig),
    ]
}
fn sliced_length(suf: bool) -> numeric::Word {
    let w = numeric::source(end(suf));
    numeric::difference(w.value, canon(WORD, start(suf).value))
}
fn sliced_store(suf: bool, source: Expr) -> Expr {
    subst_expr(
        sub_store(v("source")),
        &sub_env(suf, source, v("index"), v("unused"), v("unused")),
    )
}
fn range_expr(suf: bool) -> Expr {
    subst_expr(
        sub_guard(),
        &sub_env(suf, v("source"), v("index"), v("unused"), v("unused")),
    )
}
fn algorithm_body(suf: bool, d: usize, source: Expr, pattern: Expr) -> Expr {
    let (_, n) = words_affix();
    and(
        old_guard(),
        observer(
            d,
            false,
            sliced_length(suf).value,
            sliced_store(suf, source),
            n.value,
            pattern,
        ),
    )
}
fn bool_normal(p: &mut Producer, q: Typed) -> Typed {
    let n = normalize(q.right.clone(), p);
    next(q, n)
}
fn normal(w: numeric::Word) -> Typed {
    typed(word_ty(), canon(WORD, w.value.clone()), w.value, w.normal)
}
fn nonnegative(source: bool) -> Typed {
    let (l, n) = words_affix();
    typed(
        bool_ty(),
        sign_bit(WORD, if source { l.value } else { n.value }),
        literal(false),
        v(if source {
            "source_nonnegative"
        } else {
            "pattern_nonnegative"
        }),
    )
}
fn unsigned_zero(p: &mut Producer, x: Expr) -> Typed {
    let q = replace(
        p,
        bool_ty(),
        not(word_less(WORD, x.clone(), literal_word(WORD, 0))),
        less_zero(WORD, x),
    );
    bool_normal(p, q)
}
fn signed_order(
    p: &mut Producer,
    x: numeric::Word,
    y: numeric::Word,
    hx: Typed,
    hy: Typed,
    ordered: Typed,
) -> Typed {
    let mut q = reflexive_t(bool_ty(), model::signed_le(x.value, y.value));
    q = maybe(p, q, hx);
    q = maybe(p, q, hy);
    q = bool_normal(p, q);
    next(q, ordered)
}
fn guard_equivalence(p: &mut Producer) -> Typed {
    let mut q = reflexive_t(bool_ty(), old_guard());
    q = change(p, q, nonnegative(true));
    q = change(p, q, nonnegative(false));
    bool_normal(p, q)
}
fn difference_zero(p: &mut Producer, x: numeric::Word) -> Typed {
    let z = numeric::zero();
    let d = numeric::difference(x.value.clone(), z.value.clone());
    let h = unsigned_zero(p, x.value.clone());
    let q = next(arith::zero_sum(WORD, d.value.clone()), normal(d));
    next(backwards(q), numeric::restore(x, z, h.proof, false, false))
}
fn range_proof(p: &mut Producer, suf: bool) -> Typed {
    let (l, n) = words_affix();
    let lo = start(suf);
    let hi = if suf { l.clone() } else { n.clone() };
    let hg = typed(bool_ty(), guard(), literal(true), v("guard"));
    let nn = if suf {
        numeric::difference_nonnegative(
            l.clone(),
            n.clone(),
            hg.proof.clone(),
            nonnegative(true).proof,
        )
    } else {
        typed(
            bool_ty(),
            sign_bit(WORD, lo.value.clone()),
            literal(false),
            refl(literal(false)),
        )
    };
    let order = if suf {
        let c = numeric::restore(l.clone(), n.clone(), hg.proof.clone(), false, true);
        let q = numeric::sum_lower(lo.clone(), n.clone(), c.proof);
        numeric::change_left(
            p,
            q,
            numeric::restore(l.clone(), n.clone(), hg.proof.clone(), false, false),
        )
    } else {
        unsigned_zero(p, n.value.clone())
    };
    let first = signed_order(
        p,
        lo.clone(),
        hi.clone(),
        nn.clone(),
        nonnegative(suf),
        order,
    );
    let last = if suf {
        let h = numeric::reflexive_le(p, l.value.clone());
        signed_order(
            p,
            l.clone(),
            l.clone(),
            nonnegative(true),
            nonnegative(true),
            h,
        )
    } else {
        signed_order(p, n, l, nonnegative(false), nonnegative(true), hg)
    };
    let cs = subst_expr(
        both(&sub_checks()),
        &sub_env(suf, v("source"), v("index"), v("unused"), v("unused")),
    );
    let mut q = typed(bool_ty(), range_expr(suf), cs.clone(), refl(cs));
    q = change(p, q, normal(lo));
    q = maybe(p, q, first);
    q = maybe(p, q, last);
    q = maybe(p, q, nn);
    bool_normal(p, q)
}
fn length_proof(p: &mut Producer, suf: bool) -> Typed {
    let (l, n) = words_affix();
    let lo = start(suf);
    let q = if suf {
        let q = copying::cancel_sum(lo.clone(), n.clone());
        numeric::change_left(p, q, numeric::restore(l, n, v("guard"), false, false))
    } else {
        difference_zero(p, n)
    };
    numeric::change_left(p, q, backwards(normal(lo)))
}
fn range_call(suf: bool) -> Typed {
    typed(
        bool_ty(),
        range_expr(suf),
        literal(true),
        af(
            &format!("{}Range", label(suf)),
            vec![
                v("source_length"),
                v("source"),
                v("pattern_length"),
                v("pattern"),
                v("source_nonnegative"),
                v("pattern_nonnegative"),
                v("guard"),
            ],
        ),
    )
}
fn length_call(suf: bool) -> Typed {
    let (_, n) = words_affix();
    typed(
        word_ty(),
        sliced_length(suf).value,
        n.value,
        af(
            &format!("{}Length", label(suf)),
            vec![
                v("source_length"),
                v("source"),
                v("pattern_length"),
                v("pattern"),
                v("source_nonnegative"),
                v("pattern_nonnegative"),
                v("guard"),
            ],
        ),
    )
}
fn inactive(p: &mut Producer, l: Expr, s: Expr, i: Expr, h: Expr) -> Typed {
    select(
        p,
        mask_read(l.clone(), s, i.clone()),
        active(i, l),
        false,
        h,
    )
}
fn zero_left(_p: &mut Producer, i: Expr) -> Typed {
    let ci = canon(WORD, i.clone());
    let q = next(
        arith::commute(
            WORD,
            literal_word(WORD, 0),
            ci.clone(),
            literal(false),
            false,
        ),
        arith::zero_sum(WORD, ci),
    );
    next(q, canon_idem(WORD, i))
}
fn old_index(p: &mut Producer, suf: bool, i: Expr, hindex: Expr) -> Typed {
    let (_, n) = words_affix();
    let mut h = reflexive_t(bool_ty(), active(i.clone(), sliced_length(suf).value));
    h = change(p, h, length_call(suf));
    h = next(
        h,
        typed(bool_ty(), active(i.clone(), n.value), literal(true), hindex),
    );
    let mut q = subst_t(
        call_sub("Index", true),
        &sub_env(suf, v("source"), i.clone(), range_call(suf).proof, h.proof),
    );
    q = numeric::change_left(p, q, normal(start(suf)));
    if !suf {
        let z = zero_left(p, i.clone());
        q = numeric::change_left(p, q, z);
        q = numeric::change_left(p, q, canon_idem(WORD, i));
    }
    q
}
fn sub_read_at(p: &mut Producer, suf: bool, source: Expr, i: Expr, hindex: Expr) -> Typed {
    let (_, n) = words_affix();
    let mut h = reflexive_t(bool_ty(), active(i.clone(), sliced_length(suf).value));
    h = change(p, h, length_call(suf));
    h = next(
        h,
        typed(bool_ty(), active(i.clone(), n.value), literal(true), hindex),
    );
    let q = sub_read(p, v("source"));
    let mut q = subst_t(
        q,
        &sub_env(suf, source, i.clone(), range_call(suf).proof, h.proof),
    );
    q = maybe(p, q, normal(start(suf)));
    if !suf {
        let z = zero_left(p, i.clone());
        q = change(p, q, z);
        q = change(p, q, canon_idem(WORD, i));
    }
    q
}

// Finite comparison uses complete Byte equalities at each address, without eta.
fn observer_rewrite(
    p: &mut Producer,
    d: usize,
    n: Expr,
    a: Expr,
    b: Expr,
    t: Expr,
    mut rewrite: impl FnMut(&mut Producer, Expr) -> Typed,
) -> Typed {
    observer_both(p, d, n, a, b, t.clone(), t, |p, i| vec![rewrite(p, i)])
}
#[allow(clippy::too_many_arguments)]
fn observer_both(
    p: &mut Producer,
    d: usize,
    n: Expr,
    a: Expr,
    b: Expr,
    t: Expr,
    u: Expr,
    mut rewrite: impl FnMut(&mut Producer, Expr) -> Vec<Typed>,
) -> Typed {
    let bits = addresses(d, "__affix_cell_");
    let i = index_word(&bits);
    let ca = cells(d, n.clone(), a.clone(), t.clone());
    let cb = cells(d, n.clone(), b.clone(), u.clone());
    let opened = weak_beta(&apply(ca.clone(), bits.clone()));
    let mut q = reflexive_t(bool_ty(), opened);
    for r in rewrite(p, i) {
        q = maybe(p, q, r);
    }
    assert_eq!(q.right, weak_beta(&apply(cb.clone(), bits.clone())));
    let h = cube_lambda(d, "__affix_cell_", q.proof);
    let f = fold_congr(d, false, ca, cb, h);
    let oa = observer(d, false, n.clone(), a.clone(), n.clone(), t.clone());
    let ob = observer(d, false, n.clone(), b.clone(), n.clone(), u.clone());
    let body = observer_body(d, false, n.clone(), a, n.clone(), t.clone());
    let mut out = typed(bool_ty(), oa, body.clone(), refl(body));
    // Bound variable spelling is irrelevant to kernel conversion, but `change`
    // uses exact syntax. Retain the observer's address names at its endpoints.
    let fold_in = match &out.right {
        Expr::Apply(_, xs) => xs[2].clone(),
        _ => unreachable!(),
    };
    let fold_out = match observer_body(d, false, n.clone(), b.clone(), n.clone(), u.clone()) {
        Expr::Apply(_, xs) => xs[2].clone(),
        _ => unreachable!(),
    };
    let f = typed(bool_ty(), fold_in, fold_out, f.proof);
    out = change(p, out, f);
    let tail = typed(
        bool_ty(),
        ob,
        observer_body(d, false, n.clone(), b, n, u),
        refl(out.right.clone()),
    );
    next(out, backwards(tail))
}

fn masked_sub_read(p: &mut Producer, suf: bool, source: Expr, i: Expr) -> Typed {
    let (_, n) = words_affix();
    let a = sliced_store(suf, source.clone());
    let b = raw_affix(suf, source.clone());
    let left = read_byte(mask(n.value.clone(), a.clone()), i.clone());
    let right = read_byte(mask(n.value.clone(), b.clone()), i.clone());
    let active = active(i.clone(), n.value.clone());
    let no = next(
        inactive(p, n.value.clone(), a.clone(), i.clone(), v("defined")),
        backwards(inactive(
            p,
            n.value.clone(),
            b.clone(),
            i.clone(),
            v("defined"),
        )),
    );
    let yes = next(
        index_observation(n.value.clone(), a, i.clone(), v("defined")),
        sub_read_at(p, suf, source.clone(), i.clone(), v("defined")),
    );
    let tail = next(
        index_observation(n.value, b, i.clone(), v("defined")),
        raw_read(suf, source, i),
    );
    let yes = next(yes, backwards(tail));
    typed(
        byte(),
        left.clone(),
        right.clone(),
        cases(
            active,
            "defined",
            eq_ty(byte(), left, right),
            no.proof,
            yes.proof,
        ),
    )
}
fn definition_relation(suf: bool, d: usize, s: Expr, t: Expr) -> Typed {
    let body = relation_body(suf, d, s.clone(), t.clone());
    typed(bool_ty(), relation(suf, d, s, t), body.clone(), refl(body))
}
fn definition_algorithm(suf: bool, d: usize, s: Expr, t: Expr) -> Typed {
    let body = algorithm_body(suf, d, s.clone(), t.clone());
    typed(bool_ty(), algorithm(suf, d, s, t), body.clone(), refl(body))
}
fn binding_proof(p: &mut Producer, suf: bool, d: usize) -> Typed {
    let s = v("source");
    let t = v("pattern");
    let (_, n) = words_affix();
    let goal = eq(
        algorithm(suf, d, s.clone(), t.clone()),
        relation(suf, d, s.clone(), t.clone()),
    );
    let mut no = definition_algorithm(suf, d, s.clone(), t.clone());
    let ge = guard_equivalence(p);
    no = change(p, no, ge);
    no = change(p, no, typed(bool_ty(), guard(), literal(false), v("guard")));
    no = bool_normal(p, no);
    let mut nr = definition_relation(suf, d, s.clone(), t.clone());
    nr = change(p, nr, typed(bool_ty(), guard(), literal(false), v("guard")));
    nr = bool_normal(p, nr);
    let no = next(no, backwards(nr));
    let mut yes = definition_algorithm(suf, d, s.clone(), t.clone());
    let g = guard_equivalence(p);
    yes = change(p, yes, g);
    yes = change(p, yes, typed(bool_ty(), guard(), literal(true), v("guard")));
    yes = bool_normal(p, yes);
    yes = change(p, yes, length_call(suf));
    let q = observer_rewrite(
        p,
        d,
        n.value,
        sliced_store(suf, s.clone()),
        raw_affix(suf, s.clone()),
        t.clone(),
        |p, i| masked_sub_read(p, suf, s.clone(), i),
    );
    yes = next(yes, q);
    let mut yr = definition_relation(suf, d, s.clone(), t.clone());
    yr = change(p, yr, typed(bool_ty(), guard(), literal(true), v("guard")));
    yr = bool_normal(p, yr);
    let yes = next(yes, backwards(yr));
    typed(
        bool_ty(),
        algorithm(suf, d, s.clone(), t.clone()),
        relation(suf, d, s, t),
        cases(guard(), "guard", goal, no.proof, yes.proof),
    )
}

fn canonical_read(p: &mut Producer, suf: bool, s: Expr, i: Expr) -> Typed {
    let (l, n) = words_affix();
    let masked = raw_affix(suf, mask(l.value.clone(), s.clone()));
    let raw = raw_affix(suf, s.clone());
    let left = read_byte(mask(n.value.clone(), masked.clone()), i.clone());
    let right = read_byte(mask(n.value.clone(), raw.clone()), i.clone());
    let no = next(
        inactive(p, n.value.clone(), masked.clone(), i.clone(), v("defined")),
        backwards(inactive(
            p,
            n.value.clone(),
            raw.clone(),
            i.clone(),
            v("defined"),
        )),
    );
    let mut yes = next(
        index_observation(n.value.clone(), masked, i.clone(), v("defined")),
        raw_read(suf, mask(l.value.clone(), s.clone()), i.clone()),
    );
    let mapped = if suf {
        sum(
            WORD,
            start(suf).value,
            canon(WORD, i.clone()),
            literal(false),
        )
    } else {
        i.clone()
    };
    let valid = old_index(p, suf, i.clone(), v("defined"));
    yes = next(
        yes,
        index_observation(l.value, s.clone(), mapped, valid.proof),
    );
    let tail = next(
        index_observation(n.value.clone(), raw, i.clone(), v("defined")),
        raw_read(suf, s, i.clone()),
    );
    yes = next(yes, backwards(tail));
    typed(
        byte(),
        left.clone(),
        right.clone(),
        cases(
            active(i, n.value),
            "defined",
            eq_ty(byte(), left, right),
            no.proof,
            yes.proof,
        ),
    )
}
fn canonical_read_call(suf: bool, s: Expr, i: Expr) -> Typed {
    let (l, n) = words_affix();
    let left = read_byte(
        mask(n.value.clone(), raw_affix(suf, mask(l.value, s.clone()))),
        i.clone(),
    );
    let right = read_byte(mask(n.value, raw_affix(suf, s.clone())), i.clone());
    typed(
        byte(),
        left,
        right,
        af(
            &format!("{}CanonicalRead", label(suf)),
            vec![
                v("source_length"),
                s,
                v("pattern_length"),
                v("pattern"),
                v("source_nonnegative"),
                v("pattern_nonnegative"),
                v("guard"),
                i,
            ],
        ),
    )
}
fn canonical_proof(p: &mut Producer, suf: bool, d: usize) -> Typed {
    let (l, n) = words_affix();
    let s = v("source");
    let t = v("pattern");
    let ms = mask(l.value, s.clone());
    let mt = mask(n.value.clone(), t.clone());
    let left = relation(suf, d, ms.clone(), mt.clone());
    let right = relation(suf, d, s.clone(), t.clone());
    let goal = eq(left.clone(), right.clone());
    let mut no = definition_relation(suf, d, ms.clone(), mt.clone());
    no = change(p, no, typed(bool_ty(), guard(), literal(false), v("guard")));
    no = bool_normal(p, no);
    let mut nr = definition_relation(suf, d, s.clone(), t.clone());
    nr = change(p, nr, typed(bool_ty(), guard(), literal(false), v("guard")));
    nr = bool_normal(p, nr);
    let no = next(no, backwards(nr));
    let mut yes = definition_relation(suf, d, ms.clone(), mt.clone());
    yes = change(p, yes, typed(bool_ty(), guard(), literal(true), v("guard")));
    yes = bool_normal(p, yes);
    let q = observer_rewrite(
        p,
        d,
        n.value.clone(),
        raw_affix(suf, ms),
        raw_affix(suf, s.clone()),
        mt.clone(),
        |_, i| canonical_read_call(suf, s.clone(), i),
    );
    yes = next(yes, q);
    let q = observer_both(
        p,
        d,
        n.value.clone(),
        raw_affix(suf, s.clone()),
        raw_affix(suf, s.clone()),
        mt,
        t.clone(),
        |_, i| vec![remask(n.value.clone(), t.clone(), i)],
    );
    yes = next(yes, q);
    let mut yr = definition_relation(suf, d, s, t);
    yr = change(p, yr, typed(bool_ty(), guard(), literal(true), v("guard")));
    yr = bool_normal(p, yr);
    let yes = next(yes, backwards(yr));
    typed(
        bool_ty(),
        left,
        right,
        cases(guard(), "guard", goal, no.proof, yes.proof),
    )
}

fn install_observer(p: &mut Producer, d: usize) {
    for n in 0..=d {
        fold_library(p, n, false);
    }
    let a = vec![
        ("left_length".into(), word_ty()),
        ("left_store".into(), store_ty()),
        ("right_length".into(), word_ty()),
        ("right_store".into(), store_ty()),
    ];
    function_definition(
        p,
        &format!("ByteStringEqual{d}"),
        &a,
        bool_ty(),
        observer_body(
            d,
            false,
            v("left_length"),
            v("left_store"),
            v("right_length"),
            v("right_store"),
        ),
    );
}

fn all_true_cells(d: usize) -> Expr {
    cube_lambda(d, "__affix_true_", literal(true))
}
fn all_true(d: usize) -> Typed {
    typed(
        bool_ty(),
        fold(d, false, all_true_cells(d)),
        literal(true),
        af(&format!("AllTrue{d}"), vec![]),
    )
}
fn all_get(d: usize, m: Expr, bits: &[Expr], h: Expr) -> Typed {
    let mut a = vec![m.clone()];
    a.extend_from_slice(bits);
    a.push(h);
    typed(
        bool_ty(),
        apply(m, bits.to_vec()),
        literal(true),
        af(&format!("AllGet{d}"), a),
    )
}
fn install_folds(p: &mut Producer, d: usize, trace: Option<&Path>) {
    for k in 0..=d {
        let mut q = fold_definition(k, false, all_true_cells(k));
        if k > 0 {
            for high in [false, true] {
                let ty = cube_type(k - 1);
                let child = project(all_true_cells(k), high);
                let target = all_true_cells(k - 1);
                let beta = typed(
                    ty.clone(),
                    child.clone(),
                    target.clone(),
                    refl_t(ty.clone(), target.clone()),
                );
                let ctx = lambda(
                    "__affix_constant_cells",
                    ty,
                    fold(k - 1, false, v("__affix_constant_cells")),
                );
                let point = mapped(
                    beta,
                    bool_ty(),
                    ctx,
                    fold(k - 1, false, child),
                    fold(k - 1, false, target),
                );
                q = change(p, q, next(point, all_true(k - 1)));
            }
            q = bool_normal(p, q);
        }
        theorem(p, &name_af(&format!("AllTrue{k}")), &[], q);
        save_trace(p, trace, &format!("affix-all-true-{k}"));
        let m = v("comparison_cells");
        let bits = addresses(k, "__affix_get_");
        let mut a = vec![("comparison_cells".into(), cube_type(k))];
        a.extend((0..k).map(|j| (format!("__affix_get_{j}"), bool_ty())));
        a.push((
            "all_matched".into(),
            eq(fold(k, false, m.clone()), literal(true)),
        ));
        let q = if k == 0 {
            typed(bool_ty(), m.clone(), literal(true), v("all_matched"))
        } else {
            let f = fold_definition(k, false, m.clone());
            let h = next(
                backwards(f.clone()),
                typed(bool_ty(), f.left, literal(true), v("all_matched")),
            );
            let lo = fold(k - 1, false, project(m.clone(), false));
            let hi = fold(k - 1, false, project(m.clone(), true));
            let mut branches = vec![];
            for high in [false, true] {
                let branch = conjunction(p, lo.clone(), hi.clone(), h.proof.clone(), high);
                let q = all_get(k - 1, project(m.clone(), high), &bits[1..], branch.proof);
                let bit = typed(bool_ty(), bits[0].clone(), literal(high), v("half"));
                branches.push(numeric::change_left(p, q, backwards(bit)).proof);
            }
            typed(
                bool_ty(),
                apply(m.clone(), bits.clone()),
                literal(true),
                cases(
                    bits[0].clone(),
                    "half",
                    eq(apply(m.clone(), bits), literal(true)),
                    branches[0].clone(),
                    branches[1].clone(),
                ),
            )
        };
        theorem(p, &name_af(&format!("AllGet{k}")), &a, q);
        save_trace(p, trace, &format!("affix-all-get-{k}"));
    }
}
fn fold_all_true(d: usize, m: Expr, h: Expr) -> Typed {
    next(fold_congr(d, false, m, all_true_cells(d), h), all_true(d))
}
fn zero_read_call(s: Expr, i: Expr) -> Typed {
    typed(
        byte(),
        read_byte(mask(literal_word(WORD, 0), s.clone()), i.clone()),
        zero_byte(),
        af("ZeroRead", vec![s, i]),
    )
}
fn zero_read_proof(p: &mut Producer) -> Typed {
    let i = v("index");
    let z = literal_word(WORD, 0);
    let h = replace(
        p,
        bool_ty(),
        active(i.clone(), z.clone()),
        less_zero(WORD, canon(WORD, i.clone())),
    );
    let h = bool_normal(p, h);
    inactive(p, z, v("source"), i, h.proof)
}
fn empty_observer_proof(p: &mut Producer, d: usize) -> Typed {
    let z = literal_word(WORD, 0);
    let s = v("source");
    let t = v("pattern");
    let bits = addresses(d, "__affix_cell_");
    let i = index_word(&bits);
    let m = cells(d, z.clone(), s.clone(), t.clone());
    let mut q = reflexive_t(bool_ty(), weak_beta(&apply(m.clone(), bits.clone())));
    q = change(p, q, zero_read_call(s.clone(), i.clone()));
    q = change(p, q, zero_read_call(t.clone(), i));
    q = next(q, refl_equal(BYTE, zero_byte()));
    let f = fold_all_true(d, m, cube_lambda(d, "__affix_cell_", q.proof));
    let obs = observer(d, false, z.clone(), s.clone(), z.clone(), t.clone());
    let body = observer_body(d, false, z.clone(), s.clone(), z.clone(), t.clone());
    let mut out = typed(bool_ty(), obs, body.clone(), refl(body));
    let folded = match &out.right {
        Expr::Apply(_, a) => a[2].clone(),
        _ => panic!("comparison body"),
    };
    out = change(p, out, typed(bool_ty(), folded, literal(true), f.proof));
    out = change(p, out, refl_equal(WORD, z));
    bool_normal(p, out)
}
fn empty_proof(p: &mut Producer, suf: bool, d: usize) -> Typed {
    let z = literal_word(WORD, 0);
    let env = [("pattern_length", z)];
    let mut q = subst_t(definition_relation(suf, d, v("source"), v("pattern")), &env);
    q = change(p, q, arith::canonical_zero(WORD));
    let (l, _) = words_affix();
    let h = unsigned_zero(p, l.value);
    q = change(p, q, h);
    q = bool_normal(p, q);
    let Expr::Apply(_, parts) = &q.right else {
        panic!("empty observer")
    };
    let last = typed(
        bool_ty(),
        q.right.clone(),
        literal(true),
        af(
            &format!("EmptyObserver{d}"),
            vec![parts[1].clone(), parts[3].clone()],
        ),
    );
    next(q, last)
}

fn match_args(suf: bool, d: usize, index: bool, count: usize) -> Vec<(String, Expr)> {
    let mut a = base_args();
    a.push((
        "matched".into(),
        eq(relation(suf, d, v("source"), v("pattern")), literal(true)),
    ));
    if index {
        let (_, n) = words_affix();
        a.push(("index".into(), word_ty()));
        a.push((
            "index_guard".into(),
            eq(active(v("index"), n.value.clone()), literal(true)),
        ));
        a.push((
            "pattern_bound".into(),
            eq(
                numeric::le(n.value, literal_word(WORD, count as u64)),
                literal(true),
            ),
        ));
    }
    a
}
fn matched_parts(p: &mut Producer, suf: bool, d: usize) -> (Typed, Typed) {
    let def = definition_relation(suf, d, v("source"), v("pattern"));
    let h = next(
        backwards(def.clone()),
        typed(bool_ty(), def.left, literal(true), v("matched")),
    );
    let (_, n) = words_affix();
    let obs = observer(
        d,
        false,
        n.value.clone(),
        raw_affix(suf, v("source")),
        n.value,
        v("pattern"),
    );
    (
        conjunction(p, guard(), obs.clone(), h.proof.clone(), false),
        conjunction(p, guard(), obs, h.proof, true),
    )
}
fn matched_guard(suf: bool, d: usize) -> Typed {
    typed(
        bool_ty(),
        guard(),
        literal(true),
        af(
            &format!("{}MatchedLength{d}", label(suf)),
            vec![
                v("source_length"),
                v("source"),
                v("pattern_length"),
                v("pattern"),
                v("source_nonnegative"),
                v("pattern_nonnegative"),
                v("matched"),
            ],
        ),
    )
}
fn matched_index(p: &mut Producer, suf: bool, d: usize) -> Typed {
    let q = old_index(p, suf, v("index"), v("index_guard"));
    subst_t(q, &[("guard", matched_guard(suf, d).proof)])
}
fn direct_coverage(p: &mut Producer, count: usize, i: Expr) -> Typed {
    let d = max_depth(count);
    let (_, n) = words_affix();
    let q = covered(
        count,
        n.value,
        i.clone(),
        v("pattern_bound"),
        v("index_guard"),
    );
    let prefix = string_reconstruction(count, i.clone());
    let ci = canon(WORD, i.clone());
    let e = index_word(&low_bits(ci.clone(), d));
    let b = canon_congr(
        WORD,
        prefix,
        e,
        apply(
            c(&name(&format!("IndexPrefixEmbeddingBits{WORD}_{d}"))),
            vec![ci],
        ),
    );
    let mut q = next(q, b);
    for j in 0..d {
        let bits = bit_address(j);
        q = change(
            p,
            q,
            typed(
                bool_ty(),
                apply(canon(WORD, i.clone()), bits.clone()),
                apply(i.clone(), bits.clone()),
                apply(canon_bits(WORD, i.clone()), bits),
            ),
        );
    }
    q
}
fn matched_read(p: &mut Producer, suf: bool, d: usize, count: usize) -> Typed {
    let (_, ob) = matched_parts(p, suf, d);
    let (_, n) = words_affix();
    let s = raw_affix(suf, v("source"));
    let t = v("pattern");
    let body = observer_body(
        d,
        false,
        n.value.clone(),
        s.clone(),
        n.value.clone(),
        t.clone(),
    );
    let obs = observer(
        d,
        false,
        n.value.clone(),
        s.clone(),
        n.value.clone(),
        t.clone(),
    );
    let h = next(
        backwards(typed(bool_ty(), obs, body.clone(), refl(body.clone()))),
        ob,
    );
    let Expr::Apply(_, a) = body else {
        panic!("observer conjunction")
    };
    let all = conjunction(p, a[3].clone(), a[2].clone(), h.proof, true);
    let Expr::Apply(_, args) = &all.left else {
        panic!("comparison fold")
    };
    let m = args[0].clone();
    let i = v("index");
    let g = all_get(d, m, &low_bits(i.clone(), d), all.proof);
    let mut q = typed(bool_ty(), weak_beta(&g.left), literal(true), g.proof);
    let cover = direct_coverage(p, count, i.clone());
    q = numeric::change_left(p, q, backwards(cover));
    q = numeric::change_left(
        p,
        q,
        index_observation(n.value.clone(), s.clone(), i.clone(), v("index_guard")),
    );
    q = numeric::change_left(
        p,
        q,
        index_observation(n.value, t, i.clone(), v("index_guard")),
    );
    numeric::change_left(p, q, raw_read(suf, v("source"), i))
}
fn longer_proof(p: &mut Producer, suf: bool, d: usize) -> Typed {
    let (l, n) = words_affix();
    let mut q = definition_relation(suf, d, v("source"), v("pattern"));
    q = change(
        p,
        q,
        typed(
            bool_ty(),
            word_less(WORD, l.value, n.value),
            literal(true),
            v("longer"),
        ),
    );
    bool_normal(p, q)
}
fn binding_call(suf: bool, d: usize) -> Typed {
    typed(
        bool_ty(),
        algorithm(suf, d, v("source"), v("pattern")),
        relation(suf, d, v("source"), v("pattern")),
        af(
            &format!("{}Binding{d}", label(suf)),
            vec![
                v("source_length"),
                v("source"),
                v("pattern_length"),
                v("pattern"),
                v("source_nonnegative"),
                v("pattern_nonnegative"),
            ],
        ),
    )
}
fn source_matched(suf: bool, d: usize) -> Typed {
    let b = binding_call(suf, d);
    next(
        backwards(b.clone()),
        typed(bool_ty(), b.left, literal(true), v("source_success")),
    )
}

pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) {
    let d = max_depth(count);
    install_observer(p, d);
    let zr = zero_read_proof(p);
    theorem(
        p,
        &name_af("ZeroRead"),
        &[("source".into(), store_ty()), ("index".into(), word_ty())],
        zr,
    );
    save_trace(p, trace, "affix-zero-read");
    install_folds(p, d, trace);
    save_trace(p, trace, "affix-folds");
    let q = empty_observer_proof(p, d);
    theorem(
        p,
        &name_af(&format!("EmptyObserver{d}")),
        &[
            ("source".into(), store_ty()),
            ("pattern".into(), store_ty()),
        ],
        q,
    );
    save_trace(p, trace, "affix-empty-observer");
    affix_embedding(p, count);
    let q = guard_equivalence(p);
    theorem(p, &name_af("SignedGuard"), &base_args(), q);
    for suf in [false, true] {
        let args = base_args();
        let decl = &args[..4];
        function_definition(
            p,
            &name_af(&format!("{}{d}", label(suf))),
            decl,
            bool_ty(),
            relation_body(suf, d, v("source"), v("pattern")),
        );
        function_definition(
            p,
            &name_af(&format!("{}Source{d}", label(suf))),
            decl,
            bool_ty(),
            algorithm_body(suf, d, v("source"), v("pattern")),
        );
        let mut conditional = args;
        conditional.push(("guard".into(), eq(guard(), literal(true))));
        let q = range_proof(p, suf);
        assert_eq!(q.left, range_expr(suf));
        assert_eq!(q.right, literal(true));
        theorem(
            p,
            &name_af(&format!("{}Range", label(suf))),
            &conditional,
            q,
        );
        let q = length_proof(p, suf);
        assert_eq!(q.left, sliced_length(suf).value);
        assert_eq!(q.right, words_affix().1.value);
        theorem(
            p,
            &name_af(&format!("{}Length", label(suf))),
            &conditional,
            q,
        );
    }
    save_trace(p, trace, "affix-arithmetic");
    for suf in [false, true] {
        let mut a = base_args();
        a.push(("guard".into(), eq(guard(), literal(true))));
        a.push(("index".into(), word_ty()));
        let (_, n) = words_affix();
        a.push((
            "index_guard".into(),
            eq(active(v("index"), n.value), literal(true)),
        ));
        let q = old_index(p, suf, v("index"), v("index_guard"));
        theorem(p, &name_af(&format!("{}SourceIndex", label(suf))), &a, q);
        save_trace(
            p,
            trace,
            if suf {
                "affix-suffix-source-index"
            } else {
                "affix-prefix-source-index"
            },
        );
    }

    for suf in [false, true] {
        let q = binding_proof(p, suf, d);
        theorem(
            p,
            &name_af(&format!("{}Binding{d}", label(suf))),
            &base_args(),
            q,
        );
        save_trace(
            p,
            trace,
            if suf {
                "affix-suffix-binding"
            } else {
                "affix-prefix-binding"
            },
        );
        let mut a = base_args();
        a.push(("guard".into(), eq(guard(), literal(true))));
        a.push(("index".into(), word_ty()));
        let q = canonical_read(p, suf, v("source"), v("index"));
        theorem(p, &name_af(&format!("{}CanonicalRead", label(suf))), &a, q);
        save_trace(
            p,
            trace,
            if suf {
                "affix-suffix-canonical-read"
            } else {
                "affix-prefix-canonical-read"
            },
        );
        let q = canonical_proof(p, suf, d);
        theorem(
            p,
            &name_af(&format!("{}Canonical{d}", label(suf))),
            &base_args(),
            q,
        );
        save_trace(
            p,
            trace,
            if suf {
                "affix-suffix-canonical"
            } else {
                "affix-prefix-canonical"
            },
        );
        let q = empty_proof(p, suf, d);
        assert_eq!(q.right, literal(true));
        theorem(
            p,
            &name_af(&format!("{}Empty{d}", label(suf))),
            &base_args(),
            q,
        );
        let (l, n) = words_affix();
        let mut a = base_args();
        a.push((
            "longer".into(),
            eq(word_less(WORD, l.value, n.value), literal(true)),
        ));
        let q = longer_proof(p, suf, d);
        assert_eq!(q.right, literal(false));
        theorem(p, &name_af(&format!("{}Longer{d}", label(suf))), &a, q);
        let q = matched_parts(p, suf, d).0;
        theorem(
            p,
            &name_af(&format!("{}MatchedLength{d}", label(suf))),
            &match_args(suf, d, false, count),
            q,
        );
        let q = matched_index(p, suf, d);
        theorem(
            p,
            &name_af(&format!("{}MatchedIndex{d}", label(suf))),
            &match_args(suf, d, true, count),
            q,
        );
        let q = matched_read(p, suf, d, count);
        theorem(
            p,
            &name_af(&format!("{}MatchedRead{d}", label(suf))),
            &match_args(suf, d, true, count),
            q,
        );
        save_trace(
            p,
            trace,
            if suf {
                "affix-suffix-observations"
            } else {
                "affix-prefix-observations"
            },
        );
    }
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}
