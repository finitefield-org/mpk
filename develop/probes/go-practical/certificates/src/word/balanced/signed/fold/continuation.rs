//! Compose actual continuing-branch checks, deriving every intermediate bound.
//! Neither per-step loop invariants nor carry safety are input assumptions.
use super::*;

fn zero(d: usize) -> Expr {
    tree("ContinueZero", d, vec![])
}
fn zero_projection(d: usize, high: bool) -> Typed {
    typed(
        cube_type(d - 1),
        project(zero(d), high),
        zero(d - 1),
        c(&name(&format!(
            "TreeContinueZeroProjection{d}{}",
            if high { "High" } else { "Low" }
        ))),
    )
}
fn zero_sign(d: usize) -> Typed {
    typed(
        bool_ty(),
        sign(d, zero(d)),
        literal(false),
        c(&name(&format!("TreeContinueZeroSign{d}"))),
    )
}
fn carry_not_zero(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        carry(
            d,
            x.clone(),
            order::complement_word(d, zero(d)),
            literal(true),
        ),
        literal(true),
        apply(c(&name(&format!("TreeContinueCarryNotZero{d}"))), vec![x]),
    )
}
fn zero_within(d: usize, l: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        order::compare(d, zero(d), l.clone()),
        literal(true),
        apply(c(&name(&format!("TreeContinueZeroWithin{d}"))), vec![l, h]),
    )
}
fn zero_value(d: usize, f: Expr) -> Equality {
    instance(
        &format!("TreeContinueValueZero{d}"),
        vec![f.clone()],
        value(d, f, zero(d)),
        unit(),
    )
}
pub(super) fn primitives(p: &mut Producer) {
    let args = vec![("s".into(), bool_ty()), ("b".into(), bool_ty())];
    let condition = and(not(v("s")), v("b"));
    for sign in [false, true] {
        let result = if sign { v("s") } else { v("b") };
        let target = literal(!sign);
        let left = choose(condition.clone(), result, target.clone());
        let proof = prove_with_checked_normalization(
            &eq(left.clone(), target.clone()),
            &[v("s"), v("b")],
            &mut p.fresh,
        );
        theorem(
            p,
            if sign {
                "ContinueItemSign"
            } else {
                "ContinueItemBound"
            },
            &args,
            typed(bool_ty(), left, target, proof),
        );
    }
    let out = local(v("x"), literal(true), literal(true)).1;
    let proof =
        prove_with_checked_normalization(&eq(out.clone(), literal(true)), &[v("x")], &mut p.fresh);
    theorem(
        p,
        "ContinueCarryOneBit",
        &[("x".into(), bool_ty())],
        typed(bool_ty(), out, literal(true), proof),
    );
    let args = vec![("f".into(), function()), ("initial".into(), nat())];
    let right = apply(v("f"), vec![v("initial")]);
    theorem(
        p,
        "ContinueLeftIdentity",
        &args,
        typed(
            nat(),
            apply(cmp(unit(), v("f")), vec![v("initial")]),
            right.clone(),
            apply(c(REFL), vec![nat(), right]),
        ),
    );
}
pub(super) fn zero_library(p: &mut Producer, d: usize, trace: Option<&Path>) {
    function_definition(
        p,
        &format!("TreeContinueZero{d}"),
        &[],
        cube_type(d),
        cube_lambda(d, "__zero_", literal(false)),
    );
    if d > 0 {
        for high in [false, true] {
            let mut q = zero_projection(d, high);
            q.proof = apply(c(REFL), vec![cube_type(d - 1), q.right.clone()]);
            theorem(
                p,
                &format!(
                    "TreeContinueZeroProjection{d}{}",
                    if high { "High" } else { "Low" }
                ),
                &[],
                q,
            );
        }
    }
    let e = if d == 0 {
        let mut e = zero_sign(d);
        e.proof = refl(literal(false));
        e
    } else {
        let mut e = sign_definition(d, zero(d));
        e = change(p, e, zero_projection(d, true));
        next(e, zero_sign(d - 1))
    };
    theorem(p, &format!("TreeContinueZeroSign{d}"), &[], e);
    let x = v("x");
    let mut e = order::carry_definition(
        d,
        x.clone(),
        order::complement_word(d, zero(d)),
        literal(true),
    );
    if d == 0 {
        let z = typed(bool_ty(), zero(0), literal(false), refl(literal(false)));
        let q = mapped(
            z,
            bool_ty(),
            lambda("__zero_bit", bool_ty(), not(v("__zero_bit"))),
            order::complement_word(0, zero(0)),
            literal(true),
        );
        e = change(p, e, q);
        let out = local(x.clone(), literal(true), literal(true)).1;
        e = next(
            e,
            typed(
                bool_ty(),
                out,
                literal(true),
                apply(c(&name("ContinueCarryOneBit")), vec![x.clone()]),
            ),
        );
    } else {
        for high in [false, true] {
            e = change(p, e, order::complementary_projection(d, zero(d), high));
            e = change(p, e, zero_projection(d, high));
        }
        e = change(p, e, carry_not_zero(d - 1, project(x.clone(), false)));
        e = next(e, carry_not_zero(d - 1, project(x.clone(), true)));
    }
    theorem(
        p,
        &format!("TreeContinueCarryNotZero{d}"),
        &[("x".into(), cube_type(d))],
        e,
    );
    let l = v("limit");
    let body = choose(
        bool_eq(sign(d, zero(d)), sign(d, l.clone())),
        carry(
            d,
            l.clone(),
            order::complement_word(d, zero(d)),
            literal(true),
        ),
        sign(d, zero(d)),
    );
    let mut e = typed(
        bool_ty(),
        order::compare(d, zero(d), l.clone()),
        body.clone(),
        refl(body),
    );
    e = change(p, e, zero_sign(d));
    e = change(
        p,
        e,
        typed(bool_ty(), sign(d, l.clone()), literal(false), v("h")),
    );
    let q = order::normalized(e.right.clone(), p);
    e = next(e, q);
    e = next(e, carry_not_zero(d, l.clone()));
    theorem(
        p,
        &format!("TreeContinueZeroWithin{d}"),
        &[
            ("limit".into(), cube_type(d)),
            ("h".into(), eq(sign(d, l), literal(false))),
        ],
        e,
    );
    let f = v("f");
    let mut e = word_unfold(d, f.clone(), zero(d));
    if d == 0 {
        e = p.change(
            e,
            selector(
                f.clone(),
                typed(bool_ty(), zero(0), literal(false), refl(literal(false))),
            ),
        );
        e = transit(
            e,
            instance(
                "OrderMaybeFalse",
                vec![f.clone()],
                conditional(f.clone(), literal(false)),
                unit(),
            ),
        );
    } else {
        let q = replace(p, function(), e.right.clone(), zero_projection(d, false));
        e = transit(e, as_fn(q));
        let q = replace(p, function(), e.right.clone(), zero_projection(d, true));
        e = transit(e, as_fn(q));
        e = p.change(e, zero_value(d - 1, f.clone()));
        e = p.change(e, zero_value(d - 1, weight(d - 1, f.clone())));
        e = transit(
            e,
            Equality {
                left: cmp(unit(), unit()),
                right: unit(),
                proof: reflf(unit()),
            },
        );
    }
    p.theorem(
        &format!("TreeContinueValueZero{d}"),
        &[("f".into(), function())],
        e,
    );
    save_trace(p, trace, &format!("continuation-zero-{d}"));
}

fn checks(node: Node, l: Expr, a: Expr, xs: Expr) -> Expr {
    call("BoundChecks", node, vec![l, a, xs])
}
fn item_check(l: Expr, a: Expr, x: Expr) -> Expr {
    and(
        not(sign(W, x.clone())),
        order::compare(W, x, order::subtract(W, l, a)),
    )
}
fn checks_body(node: Node, l: Expr, a: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        cube_lambda(node.depth, "__unused_check_", literal(true))
    } else if node.depth == 0 {
        item_check(l, a, xs)
    } else {
        let (lo, hi) = children(node);
        let mid = folded(lo, a.clone(), project(xs.clone(), false));
        lambda(
            "__check_half",
            bool_ty(),
            choose_t(
                cube_type(node.depth - 1),
                v("__check_half"),
                checks(hi, l.clone(), mid, project(xs.clone(), true)),
                checks(lo, l, a, project(xs, false)),
            ),
        )
    }
}
fn all_checks(node: Node, l: Expr, a: Expr, xs: Expr, unrelated: bool) -> Expr {
    let observed = if unrelated {
        literal(true)
    } else {
        apply(
            checks(node, l, a, xs),
            addresses(node.depth, "__check_index_"),
        )
    };
    (0..node.depth)
        .rev()
        .fold(eq(observed, literal(true)), |body, i| {
            forall(&format!("__check_index_{i}"), bool_ty(), body)
        })
}
fn data_args(node: Node) -> Vec<(String, Expr)> {
    vec![
        ("limit".into(), cube_type(W)),
        ("acc".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
    ]
}
fn binders(node: Node) -> Vec<(String, Expr)> {
    let mut args = data_args(node);
    args.extend([
        (
            "limit_nonnegative".into(),
            eq(sign(W, v("limit")), literal(false)),
        ),
        (
            "acc_nonnegative".into(),
            eq(sign(W, v("acc")), literal(false)),
        ),
        (
            "acc_within_limit".into(),
            eq(order::compare(W, v("acc"), v("limit")), literal(true)),
        ),
        (
            "checks".into(),
            all_checks(node, v("limit"), v("acc"), v("xs"), false),
        ),
    ]);
    args
}
fn arguments(l: Expr, a: Expr, xs: Expr, hs: &[Expr; 4]) -> Vec<Expr> {
    let mut args = vec![l, a, xs];
    args.extend_from_slice(hs);
    args
}
fn hypotheses() -> [Expr; 4] {
    [
        v("limit_nonnegative"),
        v("acc_nonnegative"),
        v("acc_within_limit"),
        v("checks"),
    ]
}
fn bound(node: Node, suffix: &str, l: Expr, a: Expr, xs: Expr, hs: &[Expr; 4]) -> Typed {
    let result = folded(node, a.clone(), xs.clone());
    let (left, right) = match suffix {
        "Nonnegative" => (sign(W, result), literal(false)),
        "Within" => (order::compare(W, result, l.clone()), literal(true)),
        _ => panic!("unknown bound"),
    };
    typed(
        bool_ty(),
        left,
        right,
        call(&format!("Bound{suffix}"), node, arguments(l, a, xs, hs)),
    )
}
fn safe_proof(node: Node, l: Expr, a: Expr, xs: Expr, hs: &[Expr; 4]) -> Expr {
    call("BoundCarries", node, arguments(l, a, xs, hs))
}
fn bounded_math(node: Node, f: Expr, l: Expr, a: Expr, xs: Expr, hs: &[Expr; 4]) -> Equality {
    let mut args = vec![f.clone()];
    args.extend(arguments(l, a.clone(), xs.clone(), hs));
    instance(
        &symbol("BoundMath", node),
        args,
        cmp(
            value(W, f.clone(), a.clone()),
            math(node, f.clone(), xs.clone()),
        ),
        value(W, f, folded(node, a, xs)),
    )
}
fn fold_eq(node: Node, a: Expr, xs: Expr) -> Typed {
    typed(
        cube_type(W),
        folded(node, a.clone(), xs.clone()),
        fold_body(node, a.clone(), xs.clone()),
        apply(c(&name(&symbol("FoldDefinition", node))), vec![a, xs]),
    )
}
fn bound_fold(node: Node, suffix: &str, l: Expr, a: Expr, xs: Expr) -> Typed {
    let q = fold_eq(node, a, xs);
    let context = lambda(
        "__bound_word",
        cube_type(W),
        if suffix == "Nonnegative" {
            sign(W, v("__bound_word"))
        } else {
            order::compare(W, v("__bound_word"), l.clone())
        },
    );
    let left = apply(context.clone(), vec![q.left.clone()]);
    let right = apply(context.clone(), vec![q.right.clone()]);
    let raw = mapped(q, bool_ty(), context, left, right);
    // Beta endpoints are kept explicit as the same source operation.
    typed(
        bool_ty(),
        weak_beta(&raw.left),
        weak_beta(&raw.right),
        raw.proof,
    )
}
fn check_projection(node: Node, l: Expr, a: Expr, xs: Expr, high: bool) -> Typed {
    let (lo, hi) = children(node);
    let child = if high { hi } else { lo };
    let state = if high {
        folded(lo, a.clone(), project(xs.clone(), false))
    } else {
        a.clone()
    };
    typed(
        cube_type(node.depth - 1),
        project(checks(node, l.clone(), a.clone(), xs.clone()), high),
        checks(child, l.clone(), state, project(xs.clone(), high)),
        apply(
            c(&name(&symbol(
                if high {
                    "BoundCheckHigh"
                } else {
                    "BoundCheckLow"
                },
                node,
            ))),
            vec![l, a, xs],
        ),
    )
}
fn child_checks(node: Node, l: Expr, a: Expr, xs: Expr, high: bool) -> Expr {
    let q = check_projection(node, l, a, xs, high);
    let indices = addresses(node.depth - 1, "__next_check_");
    let context = lambda(
        "__checks_view",
        cube_type(node.depth - 1),
        apply(v("__checks_view"), indices.clone()),
    );
    let q = mapped(
        q.clone(),
        bool_ty(),
        context,
        apply(q.left, indices.clone()),
        apply(q.right, indices.clone()),
    );
    let mut args = vec![literal(high)];
    args.extend(indices);
    let parent = typed(
        bool_ty(),
        q.left.clone(),
        literal(true),
        apply(v("checks"), args),
    );
    let e = next(backwards(q), parent);
    (0..node.depth - 1).rev().fold(e.proof, |body, i| {
        lambda(&format!("__next_check_{i}"), bool_ty(), body)
    })
}
fn child_hypotheses(node: Node, l: Expr, a: Expr, xs: Expr, high: bool) -> [Expr; 4] {
    let low = children(node).0;
    let hs = hypotheses();
    let low_h = [
        hs[0].clone(),
        hs[1].clone(),
        hs[2].clone(),
        child_checks(node, l.clone(), a.clone(), xs.clone(), false),
    ];
    [
        hs[0].clone(),
        if high {
            bound(
                low,
                "Nonnegative",
                l.clone(),
                a.clone(),
                project(xs.clone(), false),
                &low_h,
            )
            .proof
        } else {
            hs[1].clone()
        },
        if high {
            bound(
                low,
                "Within",
                l.clone(),
                a.clone(),
                project(xs.clone(), false),
                &low_h,
            )
            .proof
        } else {
            hs[2].clone()
        },
        child_checks(node, l, a, xs, high),
    ]
}
fn leaf_hypotheses(p: &mut Producer, node: Node) -> [Expr; 5] {
    let l = v("limit");
    let a = v("acc");
    let x = v("xs");
    let body = item_check(l.clone(), a.clone(), x.clone());
    let alias = typed(
        bool_ty(),
        checks(node, l.clone(), a.clone(), x.clone()),
        body.clone(),
        refl(body.clone()),
    );
    let h = next(
        backwards(alias),
        typed(
            bool_ty(),
            checks(node, l.clone(), a.clone(), x.clone()),
            literal(true),
            v("checks"),
        ),
    );
    let s = sign(W, x.clone());
    let b = order::compare(W, x, order::subtract(W, l, a));
    let sign_h = order::under(
        p,
        body.clone(),
        s.clone(),
        literal(false),
        h.proof.clone(),
        apply(c(&name("ContinueItemSign")), vec![s.clone(), b.clone()]),
    );
    let bound_h = order::under(
        p,
        body,
        b.clone(),
        literal(true),
        h.proof,
        apply(c(&name("ContinueItemBound")), vec![s, b]),
    );
    [
        v("limit_nonnegative"),
        v("acc_nonnegative"),
        v("acc_within_limit"),
        sign_h.proof,
        bound_h.proof,
    ]
}
fn carries_parent(node: Node, l: Expr, a: Expr, xs: Expr) -> Expr {
    let mut branches = vec![];
    let rest = (1..node.depth)
        .map(|i| v(&format!("__carry_index_{i}")))
        .collect::<Vec<_>>();
    for high in [false, true] {
        let (lo, hi) = children(node);
        let child = if high { hi } else { lo };
        let state = if high {
            folded(lo, a.clone(), project(xs.clone(), false))
        } else {
            a.clone()
        };
        let child_c = carries(child, state.clone(), project(xs.clone(), high));
        let observed = project(carries(node, a.clone(), xs.clone()), high);
        let q = typed(
            cube_type(node.depth - 1),
            observed.clone(),
            child_c.clone(),
            apply(
                c(&name(&symbol(
                    if high { "CarryHigh" } else { "CarryLow" },
                    node,
                ))),
                vec![a.clone(), xs.clone()],
            ),
        );
        let context = lambda(
            "__carry_view",
            cube_type(node.depth - 1),
            apply(v("__carry_view"), rest.clone()),
        );
        let q = mapped(
            q,
            bool_ty(),
            context,
            apply(observed, rest.clone()),
            apply(child_c.clone(), rest.clone()),
        );
        let proof = apply(
            safe_proof(
                child,
                l.clone(),
                state,
                project(xs.clone(), high),
                &child_hypotheses(node, l.clone(), a.clone(), xs.clone(), high),
            ),
            rest.clone(),
        );
        let e = next(
            q,
            typed(
                bool_ty(),
                apply(child_c, rest.clone()),
                literal(false),
                proof,
            ),
        );
        branches.push((1..node.depth).rev().fold(e.proof, |body, i| {
            lambda(&format!("__carry_index_{i}"), bool_ty(), body)
        }));
    }
    let mut indices = vec![v("__half")];
    indices.extend(rest);
    let mut result = eq(apply(carries(node, a, xs), indices), literal(false));
    for i in (1..node.depth).rev() {
        result = forall(&format!("__carry_index_{i}"), bool_ty(), result);
    }
    lambda(
        "__carry_index_0",
        bool_ty(),
        apply(
            c("Std.Bool.cases"),
            vec![
                lambda("__half", bool_ty(), result),
                branches[0].clone(),
                branches[1].clone(),
                v("__carry_index_0"),
            ],
        ),
    )
}
pub(super) fn build(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let l = v("limit");
    let a = v("acc");
    let xs = v("xs");
    let data = data_args(node);
    function_definition(
        p,
        &symbol("BoundChecks", node),
        &data,
        cube_type(node.depth),
        checks_body(node, l.clone(), a.clone(), xs.clone()),
    );
    if node.depth > 0 && node.count > 0 {
        for high in [false, true] {
            let mut q = check_projection(node, l.clone(), a.clone(), xs.clone(), high);
            q.proof = apply(c(REFL), vec![cube_type(node.depth - 1), q.right.clone()]);
            theorem(
                p,
                &symbol(
                    if high {
                        "BoundCheckHigh"
                    } else {
                        "BoundCheckLow"
                    },
                    node,
                ),
                &data,
                q,
            );
        }
    }
    let args = binders(node);
    let leaf_h = if node.count > 0 && node.depth == 0 {
        Some(leaf_hypotheses(p, node))
    } else {
        None
    };
    for suffix in ["Nonnegative", "Within"] {
        let e = bound_fold(node, suffix, l.clone(), a.clone(), xs.clone());
        let tail = if node.count == 0 {
            typed(
                bool_ty(),
                if suffix == "Nonnegative" {
                    sign(W, a.clone())
                } else {
                    order::compare(W, a.clone(), l.clone())
                },
                literal(suffix == "Within"),
                v(if suffix == "Nonnegative" {
                    "acc_nonnegative"
                } else {
                    "acc_within_limit"
                }),
            )
        } else if node.depth == 0 {
            order::path_result(
                W,
                if suffix == "Within" {
                    "WithinLimit"
                } else {
                    suffix
                },
                l.clone(),
                a.clone(),
                xs.clone(),
                leaf_h.clone().unwrap(),
            )
        } else {
            let (lo, hi) = children(node);
            let mid = folded(lo, a.clone(), project(xs.clone(), false));
            bound(
                hi,
                suffix,
                l.clone(),
                mid,
                project(xs.clone(), true),
                &child_hypotheses(node, l.clone(), a.clone(), xs.clone(), true),
            )
        };
        let e = next(e, tail);
        theorem(p, &symbol(&format!("Bound{suffix}"), node), &args, e);
    }
    let safe = if node.count == 0 {
        cube_lambda(node.depth, "__safe_index_", refl(literal(false)))
    } else if node.depth == 0 {
        let raw = carry(W, a.clone(), xs.clone(), literal(false));
        let q = typed(
            bool_ty(),
            carries(node, a.clone(), xs.clone()),
            raw.clone(),
            refl(raw),
        );
        next(
            q,
            order::path_result(
                W,
                "NoWrap",
                l.clone(),
                a.clone(),
                xs.clone(),
                leaf_h.unwrap(),
            ),
        )
        .proof
    } else {
        carries_parent(node, l.clone(), a.clone(), xs.clone())
    };
    let (ty, proof) = fold_binders(&args, safety(node, a.clone(), xs.clone(), false), safe);
    p.declare(&symbol("BoundCarries", node), ty, proof, true);
    let mut fargs = vec![("f".into(), function())];
    fargs.extend(args);
    let e = invariant(
        node,
        v("f"),
        a.clone(),
        xs.clone(),
        safe_proof(node, l, a, xs, &hypotheses()),
    );
    p.theorem(&symbol("BoundMath", node), &fargs, e);
    save_trace(
        p,
        trace,
        &format!("continuation-{}-{}", node.depth, node.count),
    );
}
fn state_claim(p: &mut Producer, node: Node, l: Expr, a: Expr, xs: Expr, hs: &[Expr; 4]) -> Typed {
    let nn = bound(node, "Nonnegative", l.clone(), a.clone(), xs.clone(), hs);
    let within = bound(node, "Within", l, a, xs, hs);
    let goal = and(not(nn.left.clone()), within.left.clone());
    let mut e = typed(bool_ty(), goal.clone(), goal.clone(), refl(goal));
    e = change(p, e, within);
    e = change(p, e, nn);
    let q = order::normalized(e.right.clone(), p);
    next(e, q)
}
fn observe(e: Equality) -> Typed {
    let left = apply(e.left.clone(), vec![v("initial")]);
    let right = apply(e.right.clone(), vec![v("initial")]);
    mapped(
        from_fn(e),
        nat(),
        lambda(
            "__continue_observer",
            function(),
            apply(v("__continue_observer"), vec![v("initial")]),
        ),
        left,
        right,
    )
}
fn zero_hypotheses(l: Expr) -> [Expr; 4] {
    [
        v("limit_nonnegative"),
        zero_sign(W).proof,
        zero_within(W, l, v("limit_nonnegative")).proof,
        v("checks"),
    ]
}
fn zero_math(p: &mut Producer, node: Node, l: Expr, xs: Expr) -> Typed {
    let f = c("Std.Nat.succ");
    let e = bounded_math(
        node,
        f.clone(),
        l.clone(),
        zero(W),
        xs.clone(),
        &zero_hypotheses(l),
    );
    let observed = observe(e);
    let start = replace(
        p,
        nat(),
        observed.left.clone(),
        from_fn(zero_value(W, f.clone())),
    );
    let m = math(node, f, xs);
    let target = apply(m.clone(), vec![v("initial")]);
    let identity = typed(
        nat(),
        start.right.clone(),
        target,
        apply(c(&name("ContinueLeftIdentity")), vec![m, v("initial")]),
    );
    next(backwards(next(start, identity)), observed)
}
pub(super) fn emit(base: Certificate, output: &Path, node: Node) {
    let mut cases = vec![];
    for family in ["state", "natural", "zero-state", "zero-natural"] {
        let rooted = family.starts_with("zero-");
        let natural = family.ends_with("natural");
        for mutation in 0..if family == "natural" { 3 } else { 2 } {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let l = v("limit");
            let xs = v("xs");
            let mut args = if rooted {
                vec![
                    ("limit".into(), cube_type(W)),
                    ("xs".into(), cube_type(W + node.depth)),
                    (
                        "limit_nonnegative".into(),
                        eq(sign(W, l.clone()), literal(false)),
                    ),
                    (
                        "checks".into(),
                        all_checks(node, l.clone(), zero(W), xs.clone(), mutation == 1),
                    ),
                ]
            } else {
                let mut args = binders(node);
                args.last_mut().unwrap().1 =
                    all_checks(node, l.clone(), v("acc"), xs.clone(), mutation == 1);
                args
            };
            let hs = if rooted {
                zero_hypotheses(l.clone())
            } else {
                hypotheses()
            };
            let acc = if rooted { zero(W) } else { v("acc") };
            let mut e = if natural {
                args.push(("initial".into(), nat()));
                if rooted {
                    zero_math(&mut p, node, l, xs.clone())
                } else {
                    observe(bounded_math(
                        node,
                        c("Std.Nat.succ"),
                        l,
                        acc,
                        xs.clone(),
                        &hs,
                    ))
                }
            } else {
                state_claim(&mut p, node, l, acc, xs.clone(), &hs)
            };
            if mutation == 2 {
                e.left = apply(math(node, c("Std.Nat.succ"), xs), vec![v("initial")]);
            }
            let ty = apply(c(EQ), vec![e.ty.clone(), e.left, e.right]);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            let suffix = match family {
                "state" => "ContinuationStateClaim",
                "natural" => "ContinuationNatClaim",
                "zero-state" => "ContinuationZeroStateClaim",
                _ => "ContinuationZeroNatClaim",
            };
            p.declare(suffix, ty.clone(), proof, true);
            let label = if mutation == 0 {
                if natural {
                    "exact-sum"
                } else {
                    "bounded-result"
                }
            } else if mutation == 1 {
                if node.count == 0 {
                    "vacuous-empty-checks"
                } else {
                    "unrelated-checks"
                }
            } else {
                "dropped-initial"
            };
            let id = format!("continuation-{}-{family}-{label}", node.count);
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":node.count,"address_depth":node.depth,"storage_capacity":1usize<<node.depth,
                "source_word_bits":64,"quantified_limit_bits":64,"quantified_accumulator_bits":if rooted {0} else {64},
                "quantified_storage_bits":64usize<<node.depth,"initial_natural_quantified":natural,"zero_initialized":rooted,"mutation":mutation,
                "expected":if mutation==0 || node.count==0 && mutation==1 {"accepted"} else {"rejected"},
                "declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(suffix),
                "premises":if rooted {"limit nonnegative; accumulator is literal zero; every actual item passes nonnegative and remaining-bound checks"} else {"initial limit/accumulator nonnegative, accumulator within limit; every actual item passes nonnegative and remaining-bound checks"},
                "scope":"private source-shaped all-continuing modular64 prefix; intermediate bounds and carries are derived; Go source/loop/error/descriptor binding remains pending",
                "goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.continuation_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
