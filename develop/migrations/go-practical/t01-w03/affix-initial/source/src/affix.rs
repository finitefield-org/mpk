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
    q = maybe(p, q, nn);
    q = maybe(p, q, first);
    q = maybe(p, q, last);
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
        |p, i| canonical_read(p, suf, s.clone(), i),
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
pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) {
    let d = max_depth(count);
    install_observer(p, d);
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
        theorem(
            p,
            &name_af(&format!("{}Range", label(suf))),
            &conditional,
            q,
        );
        let q = length_proof(p, suf);
        theorem(
            p,
            &name_af(&format!("{}Length", label(suf))),
            &conditional,
            q,
        );
    }
    save_trace(p, trace, "affix-arithmetic");
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
    }
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}
