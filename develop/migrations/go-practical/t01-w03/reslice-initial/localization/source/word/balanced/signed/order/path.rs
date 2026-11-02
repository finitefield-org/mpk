//! Source-shaped signed comparisons discharge the actual addition carry guard.
use super::*;

fn le(d: usize, a: Expr, b: Expr) -> Expr {
    named("LE", d, vec![a, b])
}
fn le_body(d: usize, a: Expr, b: Expr) -> Expr {
    choose(
        bool_eq(sign(d, a.clone()), sign(d, b.clone())),
        carry(d, b, complement(d, a.clone()), literal(true)),
        sign(d, a),
    )
}
fn not_sign(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        sign(d, complement(d, x.clone())),
        not(sign(d, x.clone())),
        apply(c(&name(&format!("TreeOrderNotSign{d}"))), vec![x]),
    )
}
fn local_equation(kind: usize, a: Expr, b: Expr, incoming: Expr) -> (Expr, Expr, Expr) {
    if kind == 2 {
        let (s, k) = local(a.clone(), b.clone(), incoming.clone());
        (
            and(not(a), and(not(b), not(s))),
            bool_eq(incoming, k),
            literal(true),
        )
    } else {
        let (s, k) = local(a.clone(), not(b.clone()), incoming);
        if kind == 0 {
            (and(not(a), and(not(b), k)), s, literal(false))
        } else {
            (and(not(a), k), b, literal(false))
        }
    }
}
fn local_name(kind: usize) -> &'static str {
    match kind {
        0 => "OrderLocalSubNonnegative",
        1 => "OrderLocalUpperSign",
        _ => "OrderLocalAddGuard",
    }
}
pub(super) fn primitives(p: &mut Producer) {
    p.theorem(
        "OrderMaybeFalse",
        &[("f".into(), function())],
        Equality {
            left: conditional(v("f"), literal(false)),
            right: unit(),
            proof: reflf(unit()),
        },
    );
    let args = vec![
        ("f".into(), function()),
        ("g".into(), function()),
        ("initial".into(), nat()),
    ];
    let right = apply(v("f"), vec![apply(v("g"), vec![v("initial")])]);
    theorem(
        p,
        "OrderComposeApplication",
        &args,
        typed(
            nat(),
            apply(cmp(v("f"), v("g")), vec![v("initial")]),
            right.clone(),
            apply(c(REFL), vec![nat(), right]),
        ),
    );
    let args = vec![("f".into(), function()), ("initial".into(), nat())];
    let right = apply(v("f"), vec![v("initial")]);
    theorem(
        p,
        "OrderPointIdentity",
        &args,
        typed(
            nat(),
            apply(cmp(v("f"), unit()), vec![v("initial")]),
            right.clone(),
            apply(c(REFL), vec![nat(), right]),
        ),
    );
    let args = vec![
        ("a".into(), bool_ty()),
        ("b".into(), bool_ty()),
        ("cin".into(), bool_ty()),
    ];
    for kind in 0..3 {
        let (cond, result, target) = local_equation(kind, v("a"), v("b"), v("cin"));
        let left = choose(cond, result, target.clone());
        let proof = prove_with_checked_normalization(
            &eq(left.clone(), target.clone()),
            &[v("a"), v("b"), v("cin")],
            &mut p.fresh,
        );
        theorem(
            p,
            local_name(kind),
            &args,
            typed(bool_ty(), left, target, proof),
        );
    }
    for bound in [false, true] {
        let args = vec![("r".into(), bool_ty()), ("t".into(), bool_ty())];
        let condition = relation(literal(true), literal(true), v("r"), v("t"));
        let target = literal(bound);
        let left = choose(
            condition,
            if bound { v("t") } else { v("r") },
            target.clone(),
        );
        let proof = prove_with_checked_normalization(
            &eq(left.clone(), target.clone()),
            &[v("r"), v("t")],
            &mut p.fresh,
        );
        theorem(
            p,
            if bound {
                "OrderFinalBoundCarry"
            } else {
                "OrderFinalNoWrap"
            },
            &args,
            typed(bool_ty(), left, target, proof),
        );
    }
}
fn word_equation(d: usize, kind: usize, a: Expr, b: Expr) -> (Expr, Expr, Expr) {
    if kind == 2 {
        let result = sum(d, a.clone(), b.clone(), literal(false));
        (
            and(
                not(sign(d, a.clone())),
                and(not(sign(d, b.clone())), not(sign(d, result))),
            ),
            guard(d, a, b, literal(false)),
            literal(true),
        )
    } else {
        let rem = sum(d, a.clone(), complement(d, b.clone()), literal(true));
        let k = carry(d, a.clone(), complement(d, b.clone()), literal(true));
        if kind == 0 {
            (
                and(not(sign(d, a)), and(not(sign(d, b)), k)),
                sign(d, rem),
                literal(false),
            )
        } else {
            (and(not(sign(d, a)), k), sign(d, b), literal(false))
        }
    }
}
fn word_name(kind: usize, d: usize) -> String {
    format!(
        "TreeOrder{}Guarded{d}",
        match kind {
            0 => "SubNonnegative",
            1 => "UpperSign",
            _ => "Add",
        }
    )
}
fn word_guarded(d: usize, kind: usize, a: Expr, b: Expr) -> Typed {
    let (condition, result, target) = word_equation(d, kind, a.clone(), b.clone());
    typed(
        bool_ty(),
        choose(condition, result, target.clone()),
        target,
        apply(c(&name(&word_name(kind, d))), vec![a, b]),
    )
}
fn both_nonnegative_le(
    p: &mut Producer,
    d: usize,
    a: Expr,
    b: Expr,
    ha: Typed,
    hb: Typed,
) -> Typed {
    let body = le_body(d, a.clone(), b.clone());
    let mut e = typed(bool_ty(), le(d, a, b), body.clone(), refl(body));
    e = change(p, e, ha);
    e = change(p, e, hb);
    let q = normalized(e.right.clone(), p);
    next(e, q)
}
fn conjunction(p: &mut Producer, condition: Expr, inputs: Vec<Typed>) -> Typed {
    let mut e = typed(
        bool_ty(),
        condition.clone(),
        condition.clone(),
        refl(condition),
    );
    for q in inputs {
        e = change(p, e, q);
    }
    let q = normalized(e.right.clone(), p);
    e = next(e, q);
    assert_eq!(e.right, literal(true));
    e
}
fn prove_guarded(p: &mut Producer, d: usize, kind: usize, a: Expr, b: Expr, h: Typed) -> Typed {
    let (cond, result, target) = word_equation(d, kind, a.clone(), b.clone());
    assert_eq!(h.left, cond);
    under(
        p,
        cond,
        result,
        target,
        h.proof,
        word_guarded(d, kind, a, b).proof,
    )
}
fn path_args(d: usize) -> Vec<(String, Expr)> {
    let l = v("limit");
    let a = v("acc");
    let x = v("item");
    let rem = sum(d, l.clone(), complement(d, a.clone()), literal(true));
    vec![
        ("limit".into(), cube_type(d)),
        ("acc".into(), cube_type(d)),
        ("item".into(), cube_type(d)),
        (
            "limit_nonnegative".into(),
            eq(sign(d, l.clone()), literal(false)),
        ),
        (
            "acc_nonnegative".into(),
            eq(sign(d, a.clone()), literal(false)),
        ),
        ("acc_within_limit".into(), eq(le(d, a, l), literal(true))),
        (
            "item_nonnegative".into(),
            eq(sign(d, x.clone()), literal(false)),
        ),
        (
            "item_within_remaining".into(),
            eq(le(d, x, rem), literal(true)),
        ),
    ]
}
fn path_call() -> Vec<Expr> {
    [
        "limit",
        "acc",
        "item",
        "limit_nonnegative",
        "acc_nonnegative",
        "acc_within_limit",
        "item_nonnegative",
        "item_within_remaining",
    ]
    .into_iter()
    .map(v)
    .collect()
}
fn known_result(d: usize, suffix: &str) -> Typed {
    let l = v("limit");
    let a = v("acc");
    let x = v("item");
    let result = sum(d, a.clone(), x.clone(), literal(false));
    let (left, right) = match suffix {
        "NoWrap" => (carry(d, a, x, literal(false)), literal(false)),
        "Nonnegative" => (sign(d, result), literal(false)),
        "WithinLimit" => (le(d, result, l), literal(true)),
        "AddGuard" => (guard(d, a, x, literal(false)), literal(true)),
        _ => panic!("unknown path theorem"),
    };
    typed(
        bool_ty(),
        left,
        right,
        apply(c(&name(&format!("TreeOrderPath{suffix}{d}"))), path_call()),
    )
}
fn point_identity(f: Expr) -> Typed {
    typed(
        nat(),
        apply(cmp(f.clone(), unit()), vec![v("initial")]),
        apply(f.clone(), vec![v("initial")]),
        apply(c(&name("OrderPointIdentity")), vec![f, v("initial")]),
    )
}
fn mathematical(p: &mut Producer, d: usize) -> Typed {
    let f = c("Std.Nat.succ");
    let a = v("acc");
    let x = v("item");
    let result = sum(d, a.clone(), x.clone(), literal(false));
    let va = value(d, f.clone(), a.clone());
    let vx = value(d, f.clone(), x.clone());
    let vr = value(d, f.clone(), result);
    let add = addition(d, f.clone(), a, x, literal(false));
    let mf = |root: Expr| {
        instance(
            "OrderMaybeFalse",
            vec![root.clone()],
            conditional(root, literal(false)),
            unit(),
        )
    };
    let lhs = p.rewrite(add.left.clone(), mf(f.clone()));
    let mut e = transit(reverse(lhs), add);
    e = p.change(e, selector(weight(d, f.clone()), known_result(d, "NoWrap")));
    e = p.change(e, mf(weight(d, f)));
    let observer = lambda(
        "__order_observer",
        function(),
        apply(v("__order_observer"), vec![v("initial")]),
    );
    let observed = mapped(
        from_fn(e.clone()),
        nat(),
        observer,
        apply(e.left, vec![v("initial")]),
        apply(e.right, vec![v("initial")]),
    );
    let outer = typed(
        nat(),
        observed.left.clone(),
        apply(
            va.clone(),
            vec![apply(cmp(vx.clone(), unit()), vec![v("initial")])],
        ),
        apply(
            c(&name("OrderComposeApplication")),
            vec![va.clone(), cmp(vx.clone(), unit()), v("initial")],
        ),
    );
    let inner = point_identity(vx.clone());
    let lifted = mapped(
        inner,
        nat(),
        va.clone(),
        outer.right.clone(),
        apply(va, vec![apply(vx, vec![v("initial")])]),
    );
    let left = next(outer, lifted);
    next(next(backwards(left), observed), point_identity(vr))
}
fn path_results(p: &mut Producer, d: usize) -> Vec<Typed> {
    let l = v("limit");
    let a = v("acc");
    let x = v("item");
    let zero = literal(false);
    let one = literal(true);
    let hl = typed(
        bool_ty(),
        sign(d, l.clone()),
        zero.clone(),
        v("limit_nonnegative"),
    );
    let ha = typed(
        bool_ty(),
        sign(d, a.clone()),
        zero.clone(),
        v("acc_nonnegative"),
    );
    let hx = typed(bool_ty(), sign(d, x.clone()), zero, v("item_nonnegative"));
    let rem = sum(d, l.clone(), complement(d, a.clone()), one.clone());
    let result = sum(d, a.clone(), x.clone(), literal(false));
    let hd = next(
        backwards(both_nonnegative_le(
            p,
            d,
            a.clone(),
            l.clone(),
            ha.clone(),
            hl.clone(),
        )),
        typed(
            bool_ty(),
            le(d, a.clone(), l.clone()),
            one.clone(),
            v("acc_within_limit"),
        ),
    );
    let cond = word_equation(d, 0, l.clone(), a.clone()).0;
    let h = conjunction(p, cond, vec![hl.clone(), ha.clone(), hd.clone()]);
    let hrem = prove_guarded(p, d, 0, l.clone(), a.clone(), h);
    let he = next(
        backwards(both_nonnegative_le(
            p,
            d,
            x.clone(),
            rem.clone(),
            hx.clone(),
            hrem,
        )),
        typed(
            bool_ty(),
            le(d, x.clone(), rem),
            one.clone(),
            v("item_within_remaining"),
        ),
    );
    let incoming = vec![one.clone(), one.clone(), literal(false), one.clone()];
    let initial = normalized(related(&incoming), p);
    assert_eq!(initial.right, one);
    let h = conserved(d, l.clone(), a.clone(), x.clone(), &incoming, initial.proof);
    let q = replace(p, bool_ty(), h.left.clone(), hd);
    let q = change(p, q, he);
    let h = next(backwards(q), h);
    let flags = chains(d, l.clone(), a.clone(), x.clone(), &incoming);
    let mut final_flags = vec![];
    for bound in [false, true] {
        final_flags.push(under(
            p,
            h.left.clone(),
            flags[if bound { 3 } else { 2 }].clone(),
            literal(bound),
            h.proof.clone(),
            apply(
                c(&name(if bound {
                    "OrderFinalBoundCarry"
                } else {
                    "OrderFinalNoWrap"
                })),
                vec![flags[2].clone(), flags[3].clone()],
            ),
        ));
    }
    let no_wrap = final_flags[0].clone();
    let within = final_flags[1].clone();
    let cond = word_equation(d, 1, l.clone(), result.clone()).0;
    let h = conjunction(p, cond, vec![hl.clone(), within.clone()]);
    let nonnegative = prove_guarded(p, d, 1, l.clone(), result.clone(), h);
    let within = next(
        both_nonnegative_le(p, d, result.clone(), l, nonnegative.clone(), hl),
        within,
    );
    let cond = word_equation(d, 2, a.clone(), x.clone()).0;
    let h = conjunction(p, cond, vec![ha, hx, nonnegative.clone()]);
    let safe = prove_guarded(p, d, 2, a, x, h);
    vec![no_wrap, nonnegative, within, safe]
}
pub(super) fn build(p: &mut Producer, d: usize, trace: Option<&Path>) {
    let x = v("x");
    let args = vec![("x".into(), cube_type(d))];
    let e = if d == 0 {
        let mut e = not_sign(d, x.clone());
        e.proof = refl(not(x));
        e
    } else {
        let mut e = sign_definition(d, complement(d, x.clone()));
        e = change(p, e, complementary_projection(d, x.clone(), true));
        e = next(e, not_sign(d - 1, project(x.clone(), true)));
        change(p, e, backwards(sign_definition(d, x)))
    };
    theorem(p, &format!("TreeOrderNotSign{d}"), &args, e);
    let a = v("a");
    let b = v("b");
    let ab = vec![("a".into(), cube_type(d)), ("b".into(), cube_type(d))];
    function_definition(
        p,
        &format!("TreeOrderLE{d}"),
        &ab,
        bool_ty(),
        le_body(d, a.clone(), b.clone()),
    );
    for kind in 0..3 {
        let (cond, result, target) = word_equation(d, kind, a.clone(), b.clone());
        let left = choose(cond, result, target.clone());
        let mut e = typed(bool_ty(), left.clone(), left.clone(), refl(left));
        let incoming = if kind == 2 {
            literal(false)
        } else {
            literal(true)
        };
        let second = if kind == 2 {
            b.clone()
        } else {
            complement(d, b.clone())
        };
        if kind != 1 {
            e = change(
                p,
                e,
                sum_binding(d, a.clone(), second.clone(), incoming.clone()),
            );
        }
        e = change(
            p,
            e,
            carry_binding(d, a.clone(), second.clone(), incoming.clone()),
        );
        if kind != 2 {
            e = change(p, e, not_sign(d, b.clone()));
        }
        let cin = into(d, a.clone(), second, incoming);
        let (condition, r, t) =
            local_equation(kind, sign(d, a.clone()), sign(d, b.clone()), cin.clone());
        let tail = typed(
            bool_ty(),
            choose(condition, r, t),
            target,
            apply(
                c(&name(local_name(kind))),
                vec![sign(d, a.clone()), sign(d, b.clone()), cin],
            ),
        );
        e = next(e, tail);
        theorem(p, &word_name(kind, d), &ab, e);
    }
    let args = path_args(d);
    for (suffix, e) in ["NoWrap", "Nonnegative", "WithinLimit", "AddGuard"]
        .into_iter()
        .zip(path_results(p, d))
    {
        theorem(p, &format!("TreeOrderPath{suffix}{d}"), &args, e);
    }
    let mut args = args;
    args.push(("initial".into(), nat()));
    let e = mathematical(p, d);
    theorem(p, &format!("TreeOrderPathNat{d}"), &args, e);
    save_trace(p, trace, &format!("order-{d}-path"));
}
pub(super) fn emit(base: &Certificate, output: &Path, d: usize, cases: &mut Vec<Value>) {
    if !(3..=6).contains(&d) {
        return;
    }
    for family in ["path", "natural"] {
        for mutation in 0..3 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut args = path_args(d);
            if (family == "path" && mutation == 1) || (family == "natural" && mutation == 2) {
                args[7].1 = eq(literal(true), literal(true));
            }
            if family == "path" && mutation == 2 {
                args[6].1 = eq(literal(true), literal(true));
            }
            let (suffix, ty) = if family == "path" {
                let mut facts = ["NoWrap", "Nonnegative", "WithinLimit", "AddGuard"]
                    .into_iter()
                    .map(|s| known_result(d, s))
                    .collect::<Vec<_>>();
                let goal = and(
                    not(facts[0].left.clone()),
                    and(
                        not(facts[1].left.clone()),
                        and(facts[2].left.clone(), facts[3].left.clone()),
                    ),
                );
                // Rewrite the outer guard before its carry subexpression.
                // Otherwise a valid earlier rewrite changes that theorem's
                // syntactic endpoint; no new premise is needed.
                facts.reverse();
                let e = conjunction(&mut p, goal.clone(), facts);
                let ty = eq(goal, literal(true));
                theorem(&mut p, "OrderPathClaim", &args, e);
                ("OrderPathClaim", ty)
            } else {
                args.push(("initial".into(), nat()));
                let e = mathematical(&mut p, d);
                let left = if mutation == 1 {
                    apply(value(d, c("Std.Nat.succ"), v("item")), vec![v("initial")])
                } else {
                    e.left.clone()
                };
                let ty = apply(c(EQ), vec![nat(), left, e.right]);
                let (closed, proof) = fold_binders(&args, ty.clone(), e.proof);
                p.declare("OrderPathNatClaim", closed, proof, true);
                ("OrderPathNatClaim", ty)
            };
            let label = if family == "path" {
                match mutation {
                    0 => "step-invariant",
                    1 => "unrelated-remaining-bound",
                    _ => "unrelated-item-sign",
                }
            } else {
                match mutation {
                    0 => "exact-nat-addition",
                    1 => "dropped-accumulator",
                    _ => "unrelated-remaining-bound",
                }
            };
            let id = format!("order-{}-{family}-{label}", 1usize << d);
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"depth":d,"width":1usize<<d,"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},
            "terms":terms,"declarations":declarations,"bytes":bytes.len(),"theorem":name(suffix),"operand_binders":3,"quantified_operand_bits":3usize<<d,
            "initial_natural_quantified":family=="natural","premises":"limit and accumulator nonnegative, accumulator <= limit, item nonnegative, item <= wrapping(limit - accumulator); named premise substitutions are deliberate mutations",
            "scope":"private source-shaped Quote continuation arithmetic; Go frontend/source binding and whole-loop proof remain pending","goal_debug":format!("{ty:?}")}));
        }
    }
}
