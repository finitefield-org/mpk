//! Source-width access/extent order; every carry is computed from operands.
use super::*;
mod validity;

pub(super) fn generate_reslice(root: &Path, output: &Path, count: usize, trace: bool) {
    assert!(CAPACITIES.contains(&count));
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.SliceResliceCapacity{count}");
    primitives(&mut p);
    for d in 0..=WORD {
        word_library(&mut p, d);
        super::super::build(&mut p, d, trace.then_some(output));
    }
    super::definitions(&mut p, trace.then_some(output));
    install_local(&mut p);
    for d in 0..=WORD {
        install_word(&mut p, d, trace.then_some(output));
    }
    validity::install(&mut p, count, trace.then_some(output));
    validity::reslice(&mut p, count, trace.then_some(output));
    if trace {
        fs::write(
            output.join("library.mpcert"),
            Builder::from(p.builder.c.clone()).finish(),
        )
        .unwrap();
    }
    validity::emit_reslice(p.builder.c, output, count);
}

fn ac(suffix: &str, args: Vec<Expr>) -> Expr {
    at(&format!("Access{suffix}"), args)
}
fn an(suffix: &str, d: usize, args: Vec<Expr>) -> Expr {
    ac(&format!("{suffix}{d}"), args)
}
fn rel(s: &[Expr]) -> Expr {
    // Six states: equal carries preserve order; unequal carries reverse it.
    choose(
        bool_eq(s[2].clone(), s[3].clone()),
        bool_eq(s[0].clone(), s[1].clone()),
        and(
            bool_eq(s[0].clone(), not(s[2].clone())),
            bool_eq(s[1].clone(), s[2].clone()),
        ),
    )
}
fn flags() -> Vec<Expr> {
    ["source_less", "result_less", "carry_i", "carry_k"]
        .into_iter()
        .map(v)
        .collect()
}
fn flag_args() -> Vec<(String, Expr)> {
    ["source_less", "result_less", "carry_i", "carry_k"]
        .into_iter()
        .map(|n| (n.into(), bool_ty()))
        .collect()
}
fn operands(d: usize) -> Vec<(String, Expr)> {
    let mut a = ["offset", "index", "capacity"]
        .into_iter()
        .map(|n| (n.into(), cube_type(d)))
        .collect::<Vec<_>>();
    a.extend(flag_args());
    a
}
fn actual_args(o: Expr, i: Expr, k: Expr, s: &[Expr]) -> Vec<Expr> {
    let mut a = vec![o, i, k];
    a.extend_from_slice(s);
    a
}
fn scan(d: usize, x: Expr, y: Expr, lower: Expr) -> Expr {
    an("Scan", d, vec![x, y, lower])
}
fn scan_body(d: usize, x: Expr, y: Expr, lower: Expr) -> Expr {
    choose(word_eq(d, x.clone(), y.clone()), lower, word_less(d, x, y))
}
fn scan_def(d: usize, x: Expr, y: Expr, lower: Expr) -> Typed {
    typed(
        bool_ty(),
        scan(d, x.clone(), y.clone(), lower.clone()),
        scan_body(d, x.clone(), y.clone(), lower.clone()),
        an("ScanDefinition", d, vec![x, y, lower]),
    )
}
fn scan_parts(d: usize, x: Expr, y: Expr, lower: Expr) -> Expr {
    if d == 0 {
        choose(bool_eq(x.clone(), y.clone()), lower, and(not(x), y))
    } else {
        let lo = scan(
            d - 1,
            project(x.clone(), false),
            project(y.clone(), false),
            lower,
        );
        scan(d - 1, project(x, true), project(y, true), lo)
    }
}
fn scan_composition(d: usize, x: Expr, y: Expr, lower: Expr) -> Typed {
    typed(
        bool_ty(),
        scan(d, x.clone(), y.clone(), lower.clone()),
        scan_parts(d, x.clone(), y.clone(), lower.clone()),
        an("ScanComposition", d, vec![x, y, lower]),
    )
}
fn scan_false(d: usize, x: Expr, y: Expr) -> Typed {
    typed(
        bool_ty(),
        scan(d, x.clone(), y.clone(), literal(false)),
        word_less(d, x.clone(), y.clone()),
        an("ScanFalse", d, vec![x, y]),
    )
}
fn sum_def(d: usize, x: Expr, y: Expr, c: Expr) -> Typed {
    let body = if d == 0 {
        let (s, _) = adder(
            std::slice::from_ref(&x),
            std::slice::from_ref(&y),
            c.clone(),
        );
        s[0].clone()
    } else {
        let (l, h, _) = half_results(d, x.clone(), y.clone(), c.clone());
        join(d, l, h)
    };
    typed(
        cube_type(d),
        sum(d, x.clone(), y.clone(), c.clone()),
        body,
        an("SumDefinition", d, vec![x, y, c]),
    )
}
fn sum_part(d: usize, x: Expr, y: Expr, c: Expr, high: bool) -> Typed {
    let (lo, hi, _) = half_results(d, x.clone(), y.clone(), c.clone());
    typed(
        cube_type(d - 1),
        project(sum(d, x.clone(), y.clone(), c.clone()), high),
        if high { hi } else { lo },
        an(if high { "SumHigh" } else { "SumLow" }, d, vec![x, y, c]),
    )
}
fn carry_def(d: usize, x: Expr, y: Expr, c: Expr) -> Typed {
    let body = if d == 0 {
        let (_, out) = adder(
            std::slice::from_ref(&x),
            std::slice::from_ref(&y),
            c.clone(),
        );
        out
    } else {
        half_results(d, x.clone(), y.clone(), c.clone()).2
    };
    typed(
        bool_ty(),
        carry(d, x.clone(), y.clone(), c.clone()),
        body,
        an("CarryDefinition", d, vec![x, y, c]),
    )
}
fn outputs(d: usize, o: Expr, i: Expr, k: Expr, s: &[Expr]) -> Vec<Expr> {
    let si = sum(d, o.clone(), i.clone(), s[2].clone());
    let sk = sum(d, o.clone(), k.clone(), s[3].clone());
    vec![
        scan(d, i.clone(), k.clone(), s[0].clone()),
        scan(d, si, sk, s[1].clone()),
        carry(d, o.clone(), i, s[2].clone()),
        carry(d, o, k, s[3].clone()),
    ]
}
fn local_outputs(o: Expr, i: Expr, k: Expr, s: &[Expr]) -> Vec<Expr> {
    let (si, ci) = adder(
        std::slice::from_ref(&o),
        std::slice::from_ref(&i),
        s[2].clone(),
    );
    let (sk, ck) = adder(
        std::slice::from_ref(&o),
        std::slice::from_ref(&k),
        s[3].clone(),
    );
    vec![
        scan_parts(0, i, k, s[0].clone()),
        scan_parts(0, si[0].clone(), sk[0].clone(), s[1].clone()),
        ci,
        ck,
    ]
}
fn conserved(d: usize, o: Expr, i: Expr, k: Expr, s: &[Expr], h: Expr) -> Typed {
    let mut a = actual_args(o.clone(), i.clone(), k.clone(), s);
    a.push(h);
    typed(
        bool_ty(),
        rel(&outputs(d, o, i, k, s)),
        literal(true),
        an("OrderConservation", d, a),
    )
}
fn install_local(p: &mut Producer) {
    bool_local(
        p,
        "SliceAccessSignedGuard",
        &["sign", "less"],
        and(
            not(v("sign")),
            choose(bool_eq(v("sign"), literal(false)), v("less"), v("sign")),
        ),
        and(not(v("sign")), v("less")),
    );
    bool_local(
        p,
        "SliceAccessScanJoin",
        &["el", "eh", "ll", "lh", "t"],
        choose(
            and(v("el"), v("eh")),
            v("t"),
            choose(v("eh"), v("ll"), v("lh")),
        ),
        choose(v("eh"), choose(v("el"), v("t"), v("ll")), v("lh")),
    );
    let s = flags();
    let o = v("offset");
    let i = v("index");
    let k = v("capacity");
    let cond = rel(&s);
    let result = rel(&local_outputs(o.clone(), i.clone(), k.clone(), &s));
    let guard = choose(cond.clone(), result.clone(), literal(true));
    let mut a = actual_args(o, i, k, &s);
    let proof =
        prove_with_checked_normalization(&eq(guard.clone(), literal(true)), &a, &mut p.fresh);
    let mut args = operands(0);
    theorem(
        p,
        "SliceAccessLocalPreservation",
        &args,
        typed(bool_ty(), guard, literal(true), proof),
    );
    let proof = ac("LocalPreservation", std::mem::take(&mut a));
    let e = under(
        p,
        cond.clone(),
        result,
        literal(true),
        v("invariant"),
        proof,
    );
    args.push(("invariant".into(), eq(cond, literal(true))));
    theorem(p, "SliceAccessLocalStep", &args, e);
    let q = normalize(
        rel(&[
            literal(false),
            literal(false),
            literal(false),
            literal(false),
        ]),
        p,
    );
    assert_eq!(q.right, literal(true));
    theorem(p, "SliceAccessOrderStart", &[], q);
    let guard = and(rel(&s), and(s[0].clone(), not(s[3].clone())));
    let out = and(not(s[2].clone()), s[1].clone());
    let whole = choose(guard.clone(), out.clone(), literal(true));
    let proof =
        prove_with_checked_normalization(&eq(whole.clone(), literal(true)), &s, &mut p.fresh);
    theorem(
        p,
        "SliceAccessFinish",
        &flag_args(),
        typed(bool_ty(), whole, literal(true), proof),
    );
}
fn install_scan(p: &mut Producer, d: usize) {
    let x = v("x");
    let y = v("y");
    let t = v("lower");
    let args = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("lower".into(), bool_ty()),
    ];
    function_definition(
        p,
        &format!("SliceAccessScan{d}"),
        &args,
        bool_ty(),
        scan_body(d, x.clone(), y.clone(), t.clone()),
    );
    let mut e = scan_def(d, x.clone(), y.clone(), t.clone());
    e.proof = refl(e.right.clone());
    theorem(
        p,
        &format!("SliceAccessScanDefinition{d}"),
        &args,
        e.clone(),
    );
    e = change(p, e, definition(true, d, x.clone(), y.clone()));
    e = change(p, e, definition(false, d, x.clone(), y.clone()));
    if d > 0 {
        let xl = project(x.clone(), false);
        let yl = project(y.clone(), false);
        let xh = project(x.clone(), true);
        let yh = project(y.clone(), true);
        let right = choose(
            word_eq(d - 1, xh.clone(), yh.clone()),
            choose(
                word_eq(d - 1, xl.clone(), yl.clone()),
                t.clone(),
                word_less(d - 1, xl.clone(), yl.clone()),
            ),
            word_less(d - 1, xh.clone(), yh.clone()),
        );
        let q = typed(
            bool_ty(),
            e.right.clone(),
            right,
            ac(
                "ScanJoin",
                vec![
                    word_eq(d - 1, xl.clone(), yl.clone()),
                    word_eq(d - 1, xh.clone(), yh.clone()),
                    word_less(d - 1, xl.clone(), yl.clone()),
                    word_less(d - 1, xh.clone(), yh.clone()),
                    t.clone(),
                ],
            ),
        );
        e = next(e, q);
        e = change(p, e, backwards(scan_def(d - 1, xl, yl, t.clone())));
        e = next(
            e,
            backwards(scan_def(
                d - 1,
                xh,
                yh,
                scan(
                    d - 1,
                    project(x.clone(), false),
                    project(y.clone(), false),
                    t.clone(),
                ),
            )),
        );
    }
    assert_eq!(e.right, scan_parts(d, x.clone(), y.clone(), t));
    theorem(p, &format!("SliceAccessScanComposition{d}"), &args, e);
    let eqflag = word_eq(d, x.clone(), y.clone());
    let mut branches = vec![];
    for truth in [false, true] {
        let mut q = scan_def(d, x.clone(), y.clone(), literal(false));
        q = change(
            p,
            q,
            typed(bool_ty(), eqflag.clone(), literal(truth), v("__scan_equal")),
        );
        let n = normalize(q.right.clone(), p);
        q = next(q, n);
        if truth {
            q = next(
                q,
                backwards(equal_not_less(d, x.clone(), y.clone(), v("__scan_equal"))),
            );
        }
        branches.push(q.proof);
    }
    let e = typed(
        bool_ty(),
        scan(d, x.clone(), y.clone(), literal(false)),
        word_less(d, x.clone(), y.clone()),
        cases(
            eqflag,
            "__scan_equal",
            eq(
                scan(d, x.clone(), y.clone(), literal(false)),
                word_less(d, x.clone(), y.clone()),
            ),
            branches[0].clone(),
            branches[1].clone(),
        ),
    );
    theorem(p, &format!("SliceAccessScanFalse{d}"), &args[..2], e);
}
fn binding(p: &mut Producer, d: usize, o: Expr, i: Expr, k: Expr, s: &[Expr]) -> Typed {
    let si = sum(d, o.clone(), i.clone(), s[2].clone());
    let sk = sum(d, o.clone(), k.clone(), s[3].clone());
    let mut e = reflexive_t(
        bool_ty(),
        rel(&outputs(d, o.clone(), i.clone(), k.clone(), s)),
    );
    e = change(
        p,
        e,
        scan_composition(d, i.clone(), k.clone(), s[0].clone()),
    );
    e = change(p, e, scan_composition(d, si, sk, s[1].clone()));
    e = change(p, e, carry_def(d, o.clone(), i.clone(), s[2].clone()));
    e = change(p, e, carry_def(d, o.clone(), k.clone(), s[3].clone()));
    if d == 0 {
        e = change(p, e, sum_def(d, o.clone(), i.clone(), s[2].clone()));
        e = change(p, e, sum_def(d, o.clone(), k.clone(), s[3].clone()));
        assert_eq!(e.right, rel(&local_outputs(o, i, k, s)));
    } else {
        for high in [false, true] {
            e = change(p, e, sum_part(d, o.clone(), i.clone(), s[2].clone(), high));
            e = change(p, e, sum_part(d, o.clone(), k.clone(), s[3].clone(), high));
        }
        let mid = outputs(
            d - 1,
            project(o.clone(), false),
            project(i.clone(), false),
            project(k.clone(), false),
            s,
        );
        assert_eq!(
            e.right,
            rel(&outputs(
                d - 1,
                project(o, true),
                project(i, true),
                project(k, true),
                &mid
            ))
        );
    }
    e
}
fn install_word(p: &mut Producer, d: usize, trace: Option<&Path>) {
    install_scan(p, d);
    let oa = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("incoming".into(), bool_ty()),
    ];
    let mut q = sum_def(d, v("x"), v("y"), v("incoming"));
    q.proof = apply(c(REFL), vec![q.ty.clone(), q.right.clone()]);
    theorem(p, &format!("SliceAccessSumDefinition{d}"), &oa, q);
    let mut q = carry_def(d, v("x"), v("y"), v("incoming"));
    q.proof = refl(q.right.clone());
    theorem(p, &format!("SliceAccessCarryDefinition{d}"), &oa, q);
    if d > 0 {
        for high in [false, true] {
            let mut q = sum_part(d, v("x"), v("y"), v("incoming"), high);
            q.proof = apply(c(REFL), vec![q.ty.clone(), q.right.clone()]);
            theorem(
                p,
                &format!("SliceAccessSum{}{d}", if high { "High" } else { "Low" }),
                &oa,
                q,
            );
        }
    }
    let o = v("offset");
    let i = v("index");
    let k = v("capacity");
    let s = flags();
    let args = operands(d);
    let cond = rel(&s);
    let mut e = binding(p, d, o.clone(), i.clone(), k.clone(), &s);
    theorem(p, &format!("SliceAccessOrderBinding{d}"), &args, e.clone());
    let tail = if d == 0 {
        let mut a = actual_args(o, i, k, &s);
        a.push(v("invariant"));
        typed(
            bool_ty(),
            e.right.clone(),
            literal(true),
            ac("LocalStep", a),
        )
    } else {
        let mid = outputs(
            d - 1,
            project(o.clone(), false),
            project(i.clone(), false),
            project(k.clone(), false),
            &s,
        );
        let lo = conserved(
            d - 1,
            project(o.clone(), false),
            project(i.clone(), false),
            project(k.clone(), false),
            &s,
            v("invariant"),
        );
        conserved(
            d - 1,
            project(o, true),
            project(i, true),
            project(k, true),
            &mid,
            lo.proof,
        )
    };
    e = next(e, tail);
    let mut a = args;
    a.push(("invariant".into(), eq(cond, literal(true))));
    theorem(p, &format!("SliceAccessOrderConservation{d}"), &a, e);
    let o = v("offset");
    let i = v("index");
    let k = v("capacity");
    let zero = vec![literal(false); 4];
    let h = ac("OrderStart", vec![]);
    let mut e = conserved(d, o.clone(), i.clone(), k.clone(), &zero, h);
    e.left = rel(&outputs(d, o.clone(), i.clone(), k.clone(), &zero));
    let old = e.clone();
    // Rewrite only the relation's actual comparison observations.
    let mut q = reflexive_t(bool_ty(), e.left.clone());
    q = change(p, q, scan_false(d, i.clone(), k.clone()));
    q = change(
        p,
        q,
        scan_false(
            d,
            sum(d, o.clone(), i.clone(), literal(false)),
            sum(d, o.clone(), k.clone(), literal(false)),
        ),
    );
    e = next(backwards(q), old);
    let out = vec![
        word_less(d, i.clone(), k.clone()),
        word_less(
            d,
            sum(d, o.clone(), i.clone(), literal(false)),
            sum(d, o.clone(), k.clone(), literal(false)),
        ),
        carry(d, o.clone(), i.clone(), literal(false)),
        carry(d, o.clone(), k.clone(), literal(false)),
    ];
    assert_eq!(e.left, rel(&out));
    let mut ma = vec![
        ("offset".into(), cube_type(d)),
        ("index".into(), cube_type(d)),
        ("capacity".into(), cube_type(d)),
    ];
    ma.push(("less".into(), eq(out[0].clone(), literal(true))));
    ma.push(("extent_no_wrap".into(), eq(out[3].clone(), literal(false))));
    let guard = and(rel(&out), and(out[0].clone(), not(out[3].clone())));
    let mut h = reflexive_t(bool_ty(), guard.clone());
    h = change(p, h, e);
    h = change(
        p,
        h,
        typed(bool_ty(), out[0].clone(), literal(true), v("less")),
    );
    h = change(
        p,
        h,
        typed(
            bool_ty(),
            out[3].clone(),
            literal(false),
            v("extent_no_wrap"),
        ),
    );
    let n = normalize(h.right.clone(), p);
    h = next(h, n);
    let target = and(not(out[2].clone()), out[1].clone());
    let proof = ac("Finish", out.clone());
    let q = under(p, guard, target, literal(true), h.proof, proof);
    theorem(p, &format!("SliceAccessMonotone{d}"), &ma, q);
    save_trace(p, trace, &format!("access-order-{d}"));
}
pub(super) fn generate(root: &Path, output: &Path, count: usize, trace: bool) {
    assert!(CAPACITIES.contains(&count));
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.SliceAccessCapacity{count}");
    primitives(&mut p);
    for d in 0..=WORD {
        word_library(&mut p, d);
        super::super::build(&mut p, d, trace.then_some(output));
    }
    super::definitions(&mut p, trace.then_some(output));
    install_local(&mut p);
    for d in 0..=WORD {
        install_word(&mut p, d, trace.then_some(output));
    }
    validity::install(&mut p, count, trace.then_some(output));
    if trace {
        fs::write(
            output.join("library.mpcert"),
            Builder::from(p.builder.c.clone()).finish(),
        )
        .unwrap();
    }
    validity::emit(p.builder.c, output, count);
}
