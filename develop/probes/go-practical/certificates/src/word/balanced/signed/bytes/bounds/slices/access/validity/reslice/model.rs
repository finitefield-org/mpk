//! Actual signed endpoint checks, capacity-based reslicing and field facts.
use super::*;
use numeric::Word;

fn call(three: bool, s: &str, args: Vec<Expr>) -> Expr {
    rc(
        &format!("{}{}", if three { "Three" } else { "Two" }, s),
        args,
    )
}
pub(super) fn data(three: bool) -> Vec<(String, Expr)> {
    let mut a = vec![
        ("descriptor".into(), descriptor()),
        ("lower".into(), word_ty()),
        ("upper".into(), word_ty()),
    ];
    if three {
        a.push(("maximum".into(), word_ty()));
    }
    a
}
pub(super) fn arguments(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Vec<Expr> {
    let mut a = vec![d, l, h];
    if three {
        a.push(m);
    }
    a
}
fn max(three: bool, d: Expr, m: Expr) -> Expr {
    if three {
        m
    } else {
        capacity(d)
    }
}
pub(super) fn values(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Vec<Expr> {
    let lo = numeric::source(l);
    let hi = numeric::source(h);
    let ma = numeric::source(max(three, d.clone(), m));
    let o = numeric::source(offset(d.clone()));
    vec![
        nil(d.clone()),
        region(d),
        numeric::addition(o.value, lo.value.clone()).value,
        numeric::difference(hi.value, lo.value.clone()).value,
        numeric::difference(ma.value, lo.value).value,
    ]
}
pub(super) fn view(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Expr {
    call(three, "View", arguments(three, d, l, h, m))
}
pub(super) fn projection(three: bool, f: usize, d: Expr, l: Expr, h: Expr, m: Expr) -> Typed {
    let vals = values(three, d.clone(), l.clone(), h.clone(), m.clone());
    typed(
        field_type(f),
        accessor(view(three, d.clone(), l.clone(), h.clone(), m.clone()), f),
        vals[f].clone(),
        call(three, &format!("Field{f}"), arguments(three, d, l, h, m)),
    )
}
pub(super) fn signed_le(x: Expr, y: Expr) -> Expr {
    let sx = sign_bit(WORD, x.clone());
    let sy = sign_bit(WORD, y.clone());
    choose(bool_eq(sx.clone(), sy), numeric::le(x, y), sx)
}
fn endpoints(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Vec<Word> {
    let mut xs = vec![numeric::source(l), numeric::source(h)];
    if three {
        xs.push(numeric::source(m));
    }
    xs.push(numeric::source(capacity(d)));
    xs
}
fn checks(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Vec<Expr> {
    let xs = endpoints(three, d, l, h, m);
    let mut cs = vec![not(sign_bit(WORD, xs[0].value.clone()))];
    for pair in xs.windows(2) {
        cs.push(signed_le(pair[0].value.clone(), pair[1].value.clone()));
    }
    cs
}
pub(super) fn guard(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Expr {
    call(three, "Guard", arguments(three, d, l, h, m))
}
fn guard_def(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> Typed {
    typed(
        bool_ty(),
        guard(three, d.clone(), l.clone(), h.clone(), m.clone()),
        both(&checks(three, d.clone(), l.clone(), h.clone(), m.clone())),
        call(three, "GuardDefinition", arguments(three, d, l, h, m)),
    )
}
pub(super) struct GuardFacts {
    pub low: Word,
    pub high: Word,
    pub maximum: Word,
    pub capacity: Word,
    pub nonnegative: Vec<Typed>,
    pub ordered: Vec<Typed>,
}
fn shape(three: bool, d: Expr, l: Expr, h: Expr, m: Expr) -> GuardFacts {
    let low = numeric::source(l);
    let high = numeric::source(h);
    let maximum = numeric::source(max(three, d.clone(), m));
    let cap = numeric::source(capacity(d));
    GuardFacts {
        low,
        high,
        maximum,
        capacity: cap,
        nonnegative: vec![],
        ordered: vec![],
    }
}
fn fact_left(g: &GuardFacts, sign: bool, n: usize) -> Expr {
    if sign {
        sign_bit(
            WORD,
            [&g.low, &g.high, &g.maximum, &g.capacity][n].value.clone(),
        )
    } else {
        numeric::le(
            [&g.low, &g.high, &g.maximum][n].value.clone(),
            [&g.high, &g.maximum, &g.capacity][n].value.clone(),
        )
    }
}
pub(super) fn guard_facts(three: bool, d: Expr, l: Expr, h: Expr, m: Expr, hg: Expr) -> GuardFacts {
    let mut a = arguments(three, d.clone(), l.clone(), h.clone(), m.clone());
    a.push(hg);
    let mut g = shape(three, d, l, h, m);
    g.nonnegative = (0..4)
        .map(|n| {
            typed(
                bool_ty(),
                fact_left(&g, true, n),
                literal(false),
                call(three, &format!("Nonnegative{n}"), a.clone()),
            )
        })
        .collect();
    g.ordered = (0..3)
        .map(|n| {
            typed(
                bool_ty(),
                fact_left(&g, false, n),
                literal(true),
                call(three, &format!("Ordered{n}"), a.clone()),
            )
        })
        .collect();
    g
}
fn derive_guard(p: &mut Producer, three: bool) -> GuardFacts {
    let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
    let xs = endpoints(three, d.clone(), l.clone(), h.clone(), m.clone());
    let cs = checks(three, d.clone(), l.clone(), h.clone(), m.clone());
    let e = guard_def(three, d.clone(), l.clone(), h.clone(), m.clone());
    let hb = next(
        backwards(e.clone()),
        typed(bool_ty(), e.left, literal(true), v("range_guard")),
    );
    let fs = facts(p, &cs, hb.proof);
    let mut nn = vec![not_true(
        p,
        sign_bit(WORD, xs[0].value.clone()),
        fs[0].proof.clone(),
    )];
    let mut ord = vec![];
    for j in 0..xs.len() - 1 {
        let sx = sign_bit(WORD, xs[j].value.clone());
        let sy = sign_bit(WORD, xs[j + 1].value.clone());
        let rev = word_less(WORD, xs[j + 1].value.clone(), xs[j].value.clone());
        let condition = and(not(sx.clone()), cs[j + 1].clone());
        let mut q = reflexive_t(bool_ty(), condition.clone());
        q = change(p, q, fs[j + 1].clone());
        q = change(p, q, nn[j].clone());
        let n = normalize(q.right.clone(), p);
        q = next(q, n);
        let result = and(not(sy.clone()), not(rev.clone()));
        let q = under(
            p,
            condition,
            result,
            literal(true),
            q.proof,
            rc("SignedForward", vec![sx, sy.clone(), rev.clone()]),
        );
        let ns = conjunction(p, not(sy.clone()), not(rev.clone()), q.proof.clone(), false);
        nn.push(not_true(p, sy, ns.proof));
        ord.push(conjunction(
            p,
            not(sign_bit(WORD, xs[j + 1].value.clone())),
            not(rev),
            q.proof,
            true,
        ));
    }
    let mut g = shape(three, d, l, h, m);
    if three {
        g.nonnegative = nn;
        g.ordered = ord;
    } else {
        g.nonnegative = vec![nn[0].clone(), nn[1].clone(), nn[2].clone(), nn[2].clone()];
        g.ordered = vec![
            ord[0].clone(),
            ord[1].clone(),
            numeric::reflexive_le(p, g.capacity.value.clone()),
        ];
    }
    g
}
pub(super) fn install_signed(p: &mut Producer) {
    let (sx, sy, rev) = (v("sx"), v("sy"), v("reverse"));
    let source = choose(
        bool_eq(sx.clone(), sy.clone()),
        not(rev.clone()),
        sx.clone(),
    );
    bool_local(
        p,
        "SliceAccessValidityResliceSignedForward",
        &["sx", "sy", "reverse"],
        choose(and(not(sx), source), and(not(sy), not(rev)), literal(true)),
        literal(true),
    );
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    install_signed(p);
    for three in [false, true] {
        let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
        let a = data(three);
        let prefix = if three { "Three" } else { "Two" };
        let vals = values(three, d.clone(), l.clone(), h.clone(), m.clone());
        function_definition(
            p,
            &format!("SliceAccessValidityReslice{prefix}View"),
            &a,
            descriptor(),
            at("Descriptor", vals.clone()),
        );
        for f in 0..5 {
            let mut q = projection(three, f, d.clone(), l.clone(), h.clone(), m.clone());
            q.proof = ctor_projection(vals.clone(), f).proof;
            theorem(
                p,
                &format!("SliceAccessValidityReslice{prefix}Field{f}"),
                &a,
                q,
            );
        }
        function_definition(
            p,
            &format!("SliceAccessValidityReslice{prefix}Guard"),
            &a,
            bool_ty(),
            both(&checks(three, d.clone(), l.clone(), h.clone(), m.clone())),
        );
        let mut q = guard_def(three, d.clone(), l.clone(), h.clone(), m.clone());
        q.proof = refl(q.right.clone());
        theorem(
            p,
            &format!("SliceAccessValidityReslice{prefix}GuardDefinition"),
            &a,
            q,
        );
        let mut a = a;
        a.push((
            "range_guard".into(),
            eq(guard(three, d, l, h, m), literal(true)),
        ));
        let g = derive_guard(p, three);
        for (n, q) in g.nonnegative.into_iter().enumerate() {
            theorem(
                p,
                &format!("SliceAccessValidityReslice{prefix}Nonnegative{n}"),
                &a,
                q,
            );
        }
        for (n, q) in g.ordered.into_iter().enumerate() {
            theorem(
                p,
                &format!("SliceAccessValidityReslice{prefix}Ordered{n}"),
                &a,
                q,
            );
        }
        save_trace(
            p,
            trace,
            if three {
                "reslice-three-model"
            } else {
                "reslice-two-model"
            },
        );
    }
}
