//! Concrete descriptor validity and actual local-index discharge.
use super::*;
mod embedding;
mod reslice;
pub(super) fn reslice(p: &mut Producer, count: usize, trace: Option<&Path>) {
    reslice::install(p, count, trace);
}
pub(super) fn emit_reslice(base: Certificate, output: &Path, count: usize) {
    reslice::emit(base, output, count);
}

fn vc(s: &str, args: Vec<Expr>) -> Expr {
    ac(&format!("Validity{s}"), args)
}
fn both(xs: &[Expr]) -> Expr {
    xs.iter()
        .rev()
        .fold(literal(true), |tail, x| and(x.clone(), tail))
}
fn valid(d: Expr, n: Expr, live: Expr) -> Expr {
    vc("Predicate", vec![d, n, live])
}
fn clauses(d: Expr, n: Expr, live: Expr) -> Vec<Expr> {
    let o = offset(d.clone());
    let l = length(d.clone());
    let k = capacity(d.clone());
    vec![
        apply(live, vec![region(d)]),
        not(sign_bit(WORD, o.clone())),
        not(sign_bit(WORD, l.clone())),
        not(sign_bit(WORD, k.clone())),
        not(sign_bit(WORD, n.clone())),
        not(word_less(WORD, k.clone(), l)),
        not(carry(
            WORD,
            canon(WORD, o.clone()),
            canon(WORD, k.clone()),
            literal(false),
        )),
        not(word_less(WORD, n, address(o, k))),
    ]
}
fn nil_clauses(d: Expr) -> Vec<Expr> {
    vec![
        word_eq(WORD, length(d.clone()), literal_word(WORD, 0)),
        word_eq(WORD, capacity(d), literal_word(WORD, 0)),
    ]
}
fn valid_body(d: Expr, n: Expr, live: Expr) -> Expr {
    choose(
        nil(d.clone()),
        both(&nil_clauses(d.clone())),
        both(&clauses(d, n, live)),
    )
}
fn valid_def(d: Expr, n: Expr, live: Expr) -> Typed {
    typed(
        bool_ty(),
        valid(d.clone(), n.clone(), live.clone()),
        valid_body(d.clone(), n.clone(), live.clone()),
        vc("Definition", vec![d, n, live]),
    )
}
fn index_def(d: Expr, i: Expr) -> Typed {
    typed(
        bool_ty(),
        defined(d.clone(), i.clone()),
        and(
            not(nil(d.clone())),
            signed_guard(length(d.clone()), i.clone()),
        ),
        vc("IndexDefinition", vec![d, i]),
    )
}
fn address_def(o: Expr, i: Expr) -> Typed {
    typed(
        word_ty(),
        address(o.clone(), i.clone()),
        sum(
            WORD,
            canon(WORD, o.clone()),
            canon(WORD, i.clone()),
            literal(false),
        ),
        vc("AddressDefinition", vec![o, i]),
    )
}
fn canon_equal(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), canon(d, x.clone())),
        literal(true),
        an("CanonicalEqual", d, vec![x]),
    )
}
fn index_facts(p: &mut Producer, d: Expr, i: Expr, h: Expr) -> (Typed, Typed) {
    let e = index_def(d.clone(), i.clone());
    let h = next(
        backwards(e.clone()),
        typed(bool_ty(), e.left, literal(true), h),
    );
    let a = not(nil(d.clone()));
    let b = signed_guard(length(d.clone()), i);
    let n = conjunction(p, a.clone(), b.clone(), h.proof.clone(), false);
    let g = conjunction(p, a, b, h.proof, true);
    (not_true(p, nil(d), n.proof), g)
}
fn selected_valid(
    p: &mut Producer,
    d: Expr,
    n: Expr,
    live: Expr,
    h: Expr,
    is_nil: bool,
    hn: Expr,
) -> Typed {
    let e = valid_def(d.clone(), n, live);
    let selected = select(p, e.clone(), nil(d), is_nil, hn);
    next(
        backwards(selected),
        typed(bool_ty(), e.left, literal(true), h),
    )
}
fn facts(p: &mut Producer, xs: &[Expr], h: Expr) -> Vec<Typed> {
    let mut out = vec![];
    let mut current = h;
    for j in 0..xs.len() {
        let tail = both(&xs[j + 1..]);
        out.push(conjunction(
            p,
            xs[j].clone(),
            tail.clone(),
            current.clone(),
            false,
        ));
        current = conjunction(p, xs[j].clone(), tail, current, true).proof;
    }
    out
}
fn nonnil_facts(p: &mut Producer, d: Expr, n: Expr, live: Expr, hv: Expr, hn: Expr) -> Vec<Typed> {
    let h = selected_valid(p, d.clone(), n.clone(), live.clone(), hv, false, hn);
    facts(p, &clauses(d, n, live), h.proof)
}
fn signed_nonnegative(l: Expr, i: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        signed_guard(l.clone(), i.clone()),
        active(i.clone(), l.clone()),
        vc("SignedNonnegative", vec![l, i, h]),
    )
}
fn less_nonnegative(x: Expr, y: Expr, hxy: Expr, hy: Expr) -> Typed {
    typed(
        bool_ty(),
        sign_bit(WORD, x.clone()),
        literal(false),
        vc("LessNonnegative", vec![x, y, hxy, hy]),
    )
}
fn primary_args() -> Vec<(String, Expr)> {
    vec![
        ("descriptor".into(), descriptor()),
        ("storage_length".into(), word_ty()),
        ("live_mask".into(), cube_type(1)),
        ("index".into(), word_ty()),
    ]
}
fn conditional_args() -> Vec<(String, Expr)> {
    let mut a = primary_args();
    a.push((
        "valid".into(),
        eq(
            valid(v("descriptor"), v("storage_length"), v("live_mask")),
            literal(true),
        ),
    ));
    a.push((
        "index_guard".into(),
        eq(defined(v("descriptor"), v("index")), literal(true)),
    ));
    a
}
fn physical(
    p: &mut Producer,
    d: Expr,
    n: Expr,
    live: Expr,
    i: Expr,
    hv: Expr,
    hg: Expr,
) -> Vec<Typed> {
    let (hnil, hguard) = index_facts(p, d.clone(), i.clone(), hg);
    let fs = nonnil_facts(p, d.clone(), n.clone(), live, hv, hnil.proof);
    let l = length(d.clone());
    let k = capacity(d.clone());
    let o = offset(d.clone());
    let hl = not_true(p, sign_bit(WORD, l.clone()), fs[2].proof.clone());
    let g = signed_nonnegative(l.clone(), i.clone(), hl.proof);
    let hactive = next(backwards(g), hguard);
    let ci = canon(WORD, i.clone());
    let hlt = conjunction(
        p,
        not(sign_bit(WORD, ci.clone())),
        word_less(WORD, ci.clone(), l.clone()),
        hactive.proof,
        true,
    );
    let hkle = not_true(
        p,
        word_less(WORD, k.clone(), l.clone()),
        fs[5].proof.clone(),
    );
    let hik = mixed_transitive(WORD, ci.clone(), l, k.clone(), hlt.proof, hkle.proof);
    let can = canon_equal(WORD, k.clone());
    let q = substitute_equal(
        WORD,
        false,
        true,
        k.clone(),
        canon(WORD, k.clone()),
        ci.clone(),
        can.proof,
    );
    let hik = next(backwards(q), hik);
    let nc = carry(
        WORD,
        canon(WORD, o.clone()),
        canon(WORD, k.clone()),
        literal(false),
    );
    let hc = not_true(p, nc, fs[6].proof.clone());
    let si = sum(WORD, canon(WORD, o.clone()), ci.clone(), literal(false));
    let sk = sum(
        WORD,
        canon(WORD, o.clone()),
        canon(WORD, k.clone()),
        literal(false),
    );
    let carry_i = carry(WORD, canon(WORD, o.clone()), ci, literal(false));
    let e = typed(
        bool_ty(),
        and(
            not(carry_i.clone()),
            word_less(WORD, si.clone(), sk.clone()),
        ),
        literal(true),
        an(
            "Monotone",
            WORD,
            vec![
                canon(WORD, o.clone()),
                canon(WORD, i.clone()),
                canon(WORD, k.clone()),
                hik.proof,
                hc.proof,
            ],
        ),
    );
    let no = conjunction(
        p,
        not(carry_i.clone()),
        word_less(WORD, si.clone(), sk.clone()),
        e.proof.clone(),
        false,
    );
    let no = not_true(p, carry_i, no.proof);
    let mut lt = conjunction(
        p,
        not(no.left.clone()),
        word_less(WORD, si, sk),
        e.proof,
        true,
    );
    let mut view = reflexive_t(bool_ty(), lt.left.clone());
    view = change(p, view, backwards(address_def(o.clone(), i.clone())));
    view = change(p, view, backwards(address_def(o.clone(), k.clone())));
    lt = next(backwards(view), lt);
    let end_le = not_true(
        p,
        word_less(WORD, n.clone(), address(o.clone(), k.clone())),
        fs[7].proof.clone(),
    );
    let abs = address(o, i);
    let lt = mixed_transitive(
        WORD,
        abs.clone(),
        address(offset(d.clone()), k),
        n.clone(),
        lt.proof,
        end_le.proof,
    );
    let can = canon_equal(WORD, abs.clone());
    let q = substitute_equal(
        WORD,
        false,
        false,
        abs.clone(),
        canon(WORD, abs.clone()),
        n.clone(),
        can.proof,
    );
    let lt = next(backwards(q), lt);
    let nn = not_true(p, sign_bit(WORD, n.clone()), fs[4].proof.clone());
    let sign = less_nonnegative(canon(WORD, abs), n, lt.proof.clone(), nn.proof);
    vec![fs[0].clone(), no, lt, sign]
}
fn physical_call(n: usize, d: Expr, s: Expr, live: Expr, i: Expr, hv: Expr, hg: Expr) -> Typed {
    let fields = clauses(d.clone(), s.clone(), live.clone());
    let o = offset(d.clone());
    let a = address(o.clone(), i.clone());
    let (left, right) = match n {
        0 => (fields[0].clone(), literal(true)),
        1 => (
            carry(WORD, canon(WORD, o), canon(WORD, i.clone()), literal(false)),
            literal(false),
        ),
        2 => (word_less(WORD, canon(WORD, a), s.clone()), literal(true)),
        _ => (sign_bit(WORD, canon(WORD, a)), literal(false)),
    };
    typed(
        bool_ty(),
        left,
        right,
        vc(&format!("Physical{n}"), vec![d, s, live, i, hv, hg]),
    )
}
fn copy_or_full(p: &mut Producer, d: Expr, n: Expr, live: Expr, hv: Expr, is_full: bool) -> Typed {
    let view = if is_full {
        full(d.clone())
    } else {
        copy(d.clone())
    };
    let mut e = valid_def(view.clone(), n.clone(), live.clone());
    for f in 0..5 {
        e = change(p, e, view_projection(d.clone(), f, is_full));
    }
    if !is_full {
        let original = typed(
            bool_ty(),
            valid(d.clone(), n.clone(), live.clone()),
            literal(true),
            hv,
        );
        return next(e, next(backwards(valid_def(d, n, live)), original));
    }
    let goal = eq(valid(view.clone(), n.clone(), live.clone()), literal(true));
    let mut branches = vec![];
    for is_nil in [false, true] {
        let mut q = select(p, e.clone(), nil(d.clone()), is_nil, v("__valid_nil"));
        if is_nil {
            let h = selected_valid(
                p,
                d.clone(),
                n.clone(),
                live.clone(),
                hv.clone(),
                true,
                v("__valid_nil"),
            );
            let f = facts(p, &nil_clauses(d.clone()), h.proof);
            q = change(p, q, f[1].clone());
        } else {
            let f = nonnil_facts(
                p,
                d.clone(),
                n.clone(),
                live.clone(),
                hv.clone(),
                v("__valid_nil"),
            );
            for j in [0, 1, 3, 4, 6, 7] {
                q = change(p, q, f[j].clone());
            }
            let same = equal_not_less(
                WORD,
                capacity(d.clone()),
                capacity(d.clone()),
                refl_equal(WORD, capacity(d.clone())).proof,
            );
            q = change(p, q, same);
        }
        let nn = normalize(q.right.clone(), p);
        q = next(q, nn);
        assert_eq!(q.right, literal(true));
        branches.push(q.proof);
    }
    typed(
        bool_ty(),
        valid(view, n, live),
        literal(true),
        cases(
            nil(d),
            "__valid_nil",
            goal,
            branches[0].clone(),
            branches[1].clone(),
        ),
    )
}
fn install_core(p: &mut Producer) {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    let i = v("index");
    let va = primary_args();
    function_definition(
        p,
        "SliceAccessValidityPredicate",
        &va[..3],
        bool_ty(),
        valid_body(d.clone(), n.clone(), live.clone()),
    );
    let mut e = valid_def(d.clone(), n.clone(), live.clone());
    e.proof = refl(e.right.clone());
    theorem(p, "SliceAccessValidityDefinition", &va[..3], e);
    let mut e = index_def(d.clone(), i.clone());
    e.proof = refl(e.right.clone());
    theorem(
        p,
        "SliceAccessValidityIndexDefinition",
        &[
            ("descriptor".into(), descriptor()),
            ("index".into(), word_ty()),
        ],
        e,
    );
    let mut e = address_def(v("offset"), i.clone());
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(
        p,
        "SliceAccessValidityAddressDefinition",
        &[("offset".into(), word_ty()), ("index".into(), word_ty())],
        e,
    );
    for depth in 0..=WORD {
        let x = v("word");
        let qs = addresses(depth, "__access_value_");
        let q = typed(
            bool_ty(),
            apply(canon(depth, x.clone()), qs.clone()),
            apply(x.clone(), qs.clone()),
            apply(canon_bits(depth, x.clone()), qs),
        );
        let h = cube_lambda(depth, "__access_value_", backwards(q).proof);
        let e = equal_from_bits(depth, x.clone(), canon(depth, x), h);
        theorem(
            p,
            &format!("SliceAccessCanonicalEqual{depth}"),
            &[("word".into(), cube_type(depth))],
            e,
        );
    }
    let l = v("length");
    let mut e = reflexive_t(bool_ty(), signed_guard(l.clone(), i.clone()));
    e = change(
        p,
        e,
        typed(
            bool_ty(),
            sign_bit(WORD, l.clone()),
            literal(false),
            v("nonnegative"),
        ),
    );
    let si = sign_bit(WORD, canon(WORD, i.clone()));
    let lt = word_less(WORD, canon(WORD, i.clone()), l.clone());
    let q = typed(
        bool_ty(),
        e.right.clone(),
        active(i.clone(), l.clone()),
        ac("SignedGuard", vec![si, lt]),
    );
    e = next(e, q);
    theorem(
        p,
        "SliceAccessValiditySignedNonnegative",
        &[
            ("length".into(), word_ty()),
            ("index".into(), word_ty()),
            ("nonnegative".into(), eq(sign_bit(WORD, l), literal(false))),
        ],
        e,
    );
    let x = v("x");
    let y = v("y");
    let sign = sign_bit(WORD, x.clone());
    let less = word_less(WORD, x.clone(), y.clone());
    let goal = eq(sign.clone(), literal(false));
    let reverse = sign_less(
        WORD,
        y.clone(),
        x.clone(),
        v("right_nonnegative"),
        v("__left_negative"),
    );
    let bad = transitive(
        WORD,
        x.clone(),
        y.clone(),
        x.clone(),
        v("less"),
        reverse.proof,
    );
    let no = equal_not_less(
        WORD,
        x.clone(),
        x.clone(),
        refl_equal(WORD, x.clone()).proof,
    );
    let absurd = impossible(
        bool_ty(),
        sign.clone(),
        literal(false),
        contradiction(bad, no.proof),
    );
    let e = typed(
        bool_ty(),
        sign.clone(),
        literal(false),
        cases(
            sign,
            "__left_negative",
            goal,
            v("__left_negative"),
            absurd.proof,
        ),
    );
    theorem(
        p,
        "SliceAccessValidityLessNonnegative",
        &[
            ("x".into(), word_ty()),
            ("y".into(), word_ty()),
            ("less".into(), eq(less, literal(true))),
            (
                "right_nonnegative".into(),
                eq(sign_bit(WORD, y), literal(false)),
            ),
        ],
        e,
    );
    let out = physical(
        p,
        d.clone(),
        n.clone(),
        live.clone(),
        i.clone(),
        v("valid"),
        v("index_guard"),
    );
    for (j, e) in out.into_iter().enumerate() {
        theorem(
            p,
            &format!("SliceAccessValidityPhysical{j}"),
            &conditional_args(),
            e,
        );
    }
    let mut fa = va[..3].to_vec();
    fa.push((
        "valid".into(),
        eq(valid(d.clone(), n.clone(), live.clone()), literal(true)),
    ));
    for is_full in [false, true] {
        let e = copy_or_full(p, d.clone(), n.clone(), live.clone(), v("valid"), is_full);
        theorem(
            p,
            if is_full {
                "SliceAccessValidityFull"
            } else {
                "SliceAccessValidityCopy"
            },
            &fa,
            e,
        );
    }
}
pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) {
    let padding = 1usize << capacity_depth(count);
    super::super::super::coverage::install_unsigned(p, padding, trace);
    install_core(p);
    save_trace(p, trace, "access-validity");
    embedding::install(p, count, trace);
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    embedding::emit(base, output, count);
}
