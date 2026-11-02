//! A left-to-right halting fold. No hypothesis supplies the hidden item checks.
use super::*;

pub(super) fn run(node: Node, l: Expr, s: Expr, xs: Expr) -> Expr {
    call("ReturnRun", node, vec![l, s, xs])
}
fn body(node: Node, l: Expr, s: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        s
    } else if node.depth == 0 {
        step(l, s, xs)
    } else {
        let (lo, hi) = children(node);
        run(
            hi,
            l.clone(),
            run(lo, l, s, project(xs.clone(), false)),
            project(xs, true),
        )
    }
}
fn data(node: Node) -> Vec<(String, Expr)> {
    vec![
        ("limit".into(), cube_type(W)),
        ("state".into(), frame_ty()),
        ("xs".into(), cube_type(W + node.depth)),
    ]
}
fn definition(node: Node, l: Expr, s: Expr, xs: Expr) -> Typed {
    typed(
        frame_ty(),
        run(node, l.clone(), s.clone(), xs.clone()),
        body(node, l.clone(), s.clone(), xs.clone()),
        call("ReturnRunDefinition", node, vec![l, s, xs]),
    )
}
pub(super) fn absorb(node: Node, l: Expr, s: Expr, xs: Expr, h: Expr) -> Typed {
    typed(
        frame_ty(),
        run(node, l.clone(), s.clone(), xs.clone()),
        s.clone(),
        call("ReturnRunStopped", node, vec![l, s, xs, h]),
    )
}
fn cannot_unstop(node: Node, l: Expr, s: Expr, xs: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        stopped(s.clone()),
        literal(false),
        call("ReturnRunCannotUnstop", node, vec![l, s, xs, h]),
    )
}
pub(super) fn invariant(node: Node, l: Expr, s: Expr, xs: Expr, hvalid: Expr) -> Expr {
    call("ReturnRunValid", node, vec![l, s, xs, hvalid])
}
pub(super) fn payload(node: Node, l: Expr, s: Expr, xs: Expr, hin: Expr, hout: Expr) -> Typed {
    typed(
        cube_type(W),
        total(run(node, l.clone(), s.clone(), xs.clone())),
        folded(node, total(s.clone()), xs.clone()),
        call("ReturnRunSuccessPayload", node, vec![l, s, xs, hin, hout]),
    )
}
pub(super) fn succeeded_checks(
    node: Node,
    l: Expr,
    s: Expr,
    xs: Expr,
    hin: Expr,
    hout: Expr,
) -> Expr {
    call("ReturnRunSuccessChecks", node, vec![l, s, xs, hin, hout])
}
pub(super) fn first_failure(node: Node, l: Expr, s: Expr, xs: Expr, h: Expr) -> Typed {
    let mid = run(
        children(node).0,
        l.clone(),
        s.clone(),
        project(xs.clone(), false),
    );
    typed(
        frame_ty(),
        run(node, l.clone(), s.clone(), xs.clone()),
        mid,
        call("ReturnRunFirstFailure", node, vec![l, s, xs, h]),
    )
}
fn output_body(node: Node, l: Expr, s: Expr, xs: Expr, h: Expr, truth: bool) -> Typed {
    let q = projection(definition(node, l, s, xs), "Stopped");
    next(
        backwards(q.clone()),
        typed(bool_ty(), q.left, literal(truth), h),
    )
}
fn children_success(node: Node, l: Expr, s: Expr, xs: Expr) -> (Expr, Expr, Expr) {
    let (lo, hi) = children(node);
    let mid = run(lo, l.clone(), s.clone(), project(xs.clone(), false));
    let hout = output_body(node, l.clone(), s, xs.clone(), v("output_running"), false).proof;
    let hmid = cannot_unstop(hi, l, mid.clone(), project(xs, true), hout.clone()).proof;
    (mid, hmid, hout)
}
fn success_payload(p: &mut Producer, node: Node) -> Typed {
    let l = v("limit");
    let s = v("state");
    let xs = v("xs");
    let e = projection(definition(node, l.clone(), s.clone(), xs.clone()), "Total");
    let hout = output_body(
        node,
        l.clone(),
        s.clone(),
        xs.clone(),
        v("output_running"),
        false,
    )
    .proof;
    let tail = if node.count == 0 {
        reflexive_t(cube_type(W), total(s.clone()))
    } else if node.depth == 0 {
        typed(
            cube_type(W),
            total(step(l.clone(), s.clone(), xs.clone())),
            sum(W, total(s.clone()), xs.clone(), literal(false)),
            apply(
                c(&name("ReturnStepSuccessPayload")),
                vec![l.clone(), s.clone(), xs.clone(), v("input_running"), hout],
            ),
        )
    } else {
        let (lo, hi) = children(node);
        let (mid, hmid, hout) = children_success(node, l.clone(), s.clone(), xs.clone());
        let low = payload(
            lo,
            l.clone(),
            s.clone(),
            project(xs.clone(), false),
            v("input_running"),
            hmid.clone(),
        );
        let high = payload(hi, l, mid, project(xs.clone(), true), hmid, hout);
        change(p, high, low)
    };
    next(next(e, tail), backwards(fold_eq(node, total(s), xs)))
}
fn success_checks(p: &mut Producer, node: Node) -> Expr {
    let l = v("limit");
    let s = v("state");
    let xs = v("xs");
    if node.count == 0 {
        return cube_lambda(node.depth, "__check_index_", refl(literal(true)));
    }
    let hout = output_body(
        node,
        l.clone(),
        s.clone(),
        xs.clone(),
        v("output_running"),
        false,
    )
    .proof;
    if node.depth == 0 {
        return apply(
            c(&name("ReturnStepSuccessChecks")),
            vec![l, s, xs, v("input_running"), hout],
        );
    }
    let (lo, hi) = children(node);
    let (mid, hmid, hout) = children_success(node, l.clone(), s.clone(), xs.clone());
    let low_payload = payload(
        lo,
        l.clone(),
        s.clone(),
        project(xs.clone(), false),
        v("input_running"),
        hmid.clone(),
    );
    let rest = (1..node.depth)
        .map(|i| v(&format!("__check_index_{i}")))
        .collect::<Vec<_>>();
    let mut branches = vec![];
    for high in [false, true] {
        let child = if high { hi } else { lo };
        let state = if high { mid.clone() } else { s.clone() };
        let child_acc = if high {
            folded(lo, total(s.clone()), project(xs.clone(), false))
        } else {
            total(s.clone())
        };
        let q = check_projection(node, l.clone(), total(s.clone()), xs.clone(), high);
        let left = apply(q.left.clone(), rest.clone());
        let right = apply(q.right.clone(), rest.clone());
        let mut q = mapped(
            q,
            bool_ty(),
            lambda(
                "__return_check_view",
                cube_type(node.depth - 1),
                apply(v("__return_check_view"), rest.clone()),
            ),
            left,
            right,
        );
        let observed = apply(
            checks(
                child,
                l.clone(),
                total(state.clone()),
                project(xs.clone(), high),
            ),
            rest.clone(),
        );
        if high {
            let normal = apply(
                checks(child, l.clone(), child_acc, project(xs.clone(), true)),
                rest.clone(),
            );
            let transport = replace(p, bool_ty(), observed.clone(), low_payload.clone());
            assert_eq!(transport.right, normal);
            q = next(q, backwards(transport));
        }
        let h = succeeded_checks(
            child,
            l.clone(),
            state,
            project(xs.clone(), high),
            if high {
                hmid.clone()
            } else {
                v("input_running")
            },
            if high { hout.clone() } else { hmid.clone() },
        );
        let e = next(
            q,
            typed(bool_ty(), observed, literal(true), apply(h, rest.clone())),
        );
        branches.push((1..node.depth).rev().fold(e.proof, |body, i| {
            lambda(&format!("__check_index_{i}"), bool_ty(), body)
        }));
    }
    let mut indices = vec![v("__half")];
    indices.extend(rest);
    let mut goal = eq(apply(checks(node, l, total(s), xs), indices), literal(true));
    for i in (1..node.depth).rev() {
        goal = forall(&format!("__check_index_{i}"), bool_ty(), goal);
    }
    lambda(
        "__check_index_0",
        bool_ty(),
        apply(
            c("Std.Bool.cases"),
            vec![
                lambda("__half", bool_ty(), goal),
                branches[0].clone(),
                branches[1].clone(),
                v("__check_index_0"),
            ],
        ),
    )
}
pub(super) fn build(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let l = v("limit");
    let s = v("state");
    let xs = v("xs");
    let args = data(node);
    function_definition(
        p,
        &symbol("ReturnRun", node),
        &args,
        frame_ty(),
        body(node, l.clone(), s.clone(), xs.clone()),
    );
    let mut d = definition(node, l.clone(), s.clone(), xs.clone());
    d.proof = apply(c(REFL), vec![frame_ty(), d.right.clone()]);
    theorem(p, &symbol("ReturnRunDefinition", node), &args, d.clone());
    let mut stopped_args = args.clone();
    stopped_args.push((
        "already_stopped".into(),
        eq(stopped(s.clone()), literal(true)),
    ));
    let mut a = d.clone();
    if node.count > 0 {
        if node.depth == 0 {
            a = next(
                a,
                instantiated_step(
                    "Stopped",
                    l.clone(),
                    s.clone(),
                    xs.clone(),
                    vec![v("already_stopped")],
                ),
            );
        } else {
            let (lo, hi) = children(node);
            a = change(
                p,
                a,
                absorb(
                    lo,
                    l.clone(),
                    s.clone(),
                    project(xs.clone(), false),
                    v("already_stopped"),
                ),
            );
            a = next(
                a,
                absorb(
                    hi,
                    l.clone(),
                    s.clone(),
                    project(xs.clone(), true),
                    v("already_stopped"),
                ),
            );
        }
    }
    theorem(p, &symbol("ReturnRunStopped", node), &stopped_args, a);
    let mut success_args = args.clone();
    success_args.push((
        "output_running".into(),
        eq(
            stopped(run(node, l.clone(), s.clone(), xs.clone())),
            literal(false),
        ),
    ));
    let goal = eq(stopped(s.clone()), literal(false));
    let out = next(
        projection(
            absorb(node, l.clone(), s.clone(), xs.clone(), v("__input_stop")),
            "Stopped",
        ),
        typed(
            bool_ty(),
            stopped(s.clone()),
            literal(true),
            v("__input_stop"),
        ),
    );
    let impossible = absurd(
        bool_ty(),
        stopped(s.clone()),
        literal(false),
        contradict(out, v("output_running")),
    );
    let q = typed(
        bool_ty(),
        stopped(s.clone()),
        literal(false),
        cases_eq(
            stopped(s.clone()),
            "__input_stop",
            goal,
            v("__input_stop"),
            impossible.proof,
        ),
    );
    theorem(p, &symbol("ReturnRunCannotUnstop", node), &success_args, q);
    let mut valid_args = args.clone();
    valid_args.push(("valid".into(), valid(s.clone())));
    let hout = output_body(node, l.clone(), s.clone(), xs.clone(), v("__failed"), true).proof;
    let hvalid = if node.count == 0 {
        v("valid")
    } else if node.depth == 0 {
        apply(
            c(&name("ReturnStepValid")),
            vec![l.clone(), s.clone(), xs.clone(), v("valid")],
        )
    } else {
        let (lo, hi) = children(node);
        let low = invariant(
            lo,
            l.clone(),
            s.clone(),
            project(xs.clone(), false),
            v("valid"),
        );
        invariant(
            hi,
            l.clone(),
            run(lo, l.clone(), s.clone(), project(xs.clone(), false)),
            project(xs.clone(), true),
            low,
        )
    };
    let tail = typed(
        cube_type(W),
        total(d.right.clone()),
        zero(W),
        apply(hvalid, vec![hout]),
    );
    let proof = next(projection(d.clone(), "Total"), tail).proof;
    let proof = lambda(
        "__failed",
        eq(stopped(d.left.clone()), literal(true)),
        proof,
    );
    let (ty, proof) = fold_binders(&valid_args, valid(d.left.clone()), proof);
    p.declare(&symbol("ReturnRunValid", node), ty, proof, true);
    success_args.insert(
        3,
        (
            "input_running".into(),
            eq(stopped(s.clone()), literal(false)),
        ),
    );
    let q = success_payload(p, node);
    theorem(
        p,
        &symbol("ReturnRunSuccessPayload", node),
        &success_args,
        q,
    );
    let proof = success_checks(p, node);
    let (ty, proof) = fold_binders(
        &success_args,
        all_checks(node, l.clone(), total(s.clone()), xs.clone(), false),
        proof,
    );
    p.declare(&symbol("ReturnRunSuccessChecks", node), ty, proof, true);
    if node.depth > 0 && node.count > 0 {
        let (lo, hi) = children(node);
        let mid = run(lo, l.clone(), s.clone(), project(xs.clone(), false));
        let mut fa = args;
        fa.push(("low_failed".into(), eq(stopped(mid.clone()), literal(true))));
        let q = next(d, absorb(hi, l, mid, project(xs, true), v("low_failed")));
        theorem(p, &symbol("ReturnRunFirstFailure", node), &fa, q);
    }
    save_trace(
        p,
        trace,
        &format!("returned-run-{}-{}", node.depth, node.count),
    );
}
