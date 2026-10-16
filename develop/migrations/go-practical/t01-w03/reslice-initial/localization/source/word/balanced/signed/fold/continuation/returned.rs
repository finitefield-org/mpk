//! An ordinary full-payload return frame with absorbing closed-error outcomes.
//! Metadata is internal; no open Go interface or new checker rule is admitted.
use super::*;
mod quote;
mod run;

fn frame_ty() -> Expr {
    cube_type(W + 1)
}
fn pack(a: Expr, stopped: Expr, bad: Expr) -> Expr {
    apply(c(&name("ReturnPack")), vec![a, stopped, bad])
}
fn total(s: Expr) -> Expr {
    apply(c(&name("ReturnTotal")), vec![s])
}
fn stopped(s: Expr) -> Expr {
    apply(c(&name("ReturnStopped")), vec![s])
}
fn bad(s: Expr) -> Expr {
    apply(c(&name("ReturnBad")), vec![s])
}
fn initial() -> Expr {
    c(&name("ReturnInitial"))
}
fn bad_frame() -> Expr {
    c(&name("ReturnBadInput"))
}
fn limit_frame() -> Expr {
    c(&name("ReturnLimitExceeded"))
}
fn reflexive_t(ty: Expr, x: Expr) -> Typed {
    typed(
        ty.clone(),
        x.clone(),
        x.clone(),
        apply(c(REFL), vec![ty, x]),
    )
}
fn fields(s: Expr, field: &str) -> Expr {
    match field {
        "Total" => total(s),
        "Stopped" => stopped(s),
        _ => bad(s),
    }
}
fn projection(e: Typed, field: &str) -> Typed {
    let ty = if field == "Total" {
        cube_type(W)
    } else {
        bool_ty()
    };
    mapped(
        e.clone(),
        ty,
        lambda(
            "__return_frame",
            frame_ty(),
            fields(v("__return_frame"), field),
        ),
        fields(e.left, field),
        fields(e.right, field),
    )
}
fn pack_projection(a: Expr, s: Expr, b: Expr, field: &str) -> Typed {
    let ty = if field == "Total" {
        cube_type(W)
    } else {
        bool_ty()
    };
    let right = match field {
        "Total" => a.clone(),
        "Stopped" => s.clone(),
        _ => b.clone(),
    };
    typed(
        ty,
        fields(pack(a.clone(), s.clone(), b.clone()), field),
        right,
        apply(c(&name(&format!("ReturnPack{field}"))), vec![a, s, b]),
    )
}
fn named_projection(kind: &str, field: &str) -> Typed {
    let (s, stop, b) = match kind {
        "Initial" => (initial(), literal(false), literal(false)),
        "BadInput" => (bad_frame(), literal(true), literal(true)),
        _ => (limit_frame(), literal(true), literal(false)),
    };
    let e = pack_projection(zero(W), stop, b, field);
    typed(e.ty, fields(s, field), e.right, e.proof)
}
fn step(l: Expr, s: Expr, x: Expr) -> Expr {
    apply(c(&name("ReturnStep")), vec![l, s, x])
}
fn remaining(l: Expr, s: Expr, x: Expr) -> Expr {
    order::compare(W, x, order::subtract(W, l, total(s)))
}
fn step_body(l: Expr, s: Expr, x: Expr) -> Expr {
    choose_t(
        frame_ty(),
        stopped(s.clone()),
        s.clone(),
        choose_t(
            frame_ty(),
            sign(W, x.clone()),
            bad_frame(),
            choose_t(
                frame_ty(),
                remaining(l, s.clone(), x.clone()),
                pack(
                    sum(W, total(s), x, literal(false)),
                    literal(false),
                    literal(false),
                ),
                limit_frame(),
            ),
        ),
    )
}
fn step_definition(l: Expr, s: Expr, x: Expr) -> Typed {
    typed(
        frame_ty(),
        step(l.clone(), s.clone(), x.clone()),
        step_body(l.clone(), s.clone(), x.clone()),
        apply(c(&name("ReturnStepDefinition")), vec![l, s, x]),
    )
}
fn select(p: &mut Producer, e: Typed, condition: Expr, truth: bool, h: Expr) -> Typed {
    let e = change(p, e, typed(bool_ty(), condition, literal(truth), h));
    let Expr::Apply(head, args) = &e.right else {
        panic!("return selection absent")
    };
    assert_eq!(**head, c("Std.Bool.cases"));
    assert_eq!(args[3], literal(truth));
    let result = args[if truth { 2 } else { 1 }].clone();
    let tail = typed(
        e.ty.clone(),
        e.right.clone(),
        result.clone(),
        apply(c(REFL), vec![e.ty.clone(), result]),
    );
    next(e, tail)
}
fn step_case(p: &mut Producer, kind: &str, l: Expr, s: Expr, x: Expr, hs: &[Expr]) -> Typed {
    let mut e = step_definition(l.clone(), s.clone(), x.clone());
    e = select(p, e, stopped(s.clone()), kind == "Stopped", hs[0].clone());
    if kind == "Stopped" {
        return e;
    }
    e = select(p, e, sign(W, x.clone()), kind == "BadInput", hs[1].clone());
    if kind == "BadInput" {
        return e;
    }
    select(p, e, remaining(l, s, x), kind == "Continue", hs[2].clone())
}
fn instantiated_step(kind: &str, l: Expr, s: Expr, x: Expr, hs: Vec<Expr>) -> Typed {
    let right = match kind {
        "Stopped" => s.clone(),
        "BadInput" => bad_frame(),
        "LimitExceeded" => limit_frame(),
        _ => pack(
            sum(W, total(s.clone()), x.clone(), literal(false)),
            literal(false),
            literal(false),
        ),
    };
    let left = step(l.clone(), s.clone(), x.clone());
    let mut args = vec![l, s, x];
    args.extend(hs);
    typed(
        frame_ty(),
        left,
        right,
        apply(c(&name(&format!("ReturnStep{kind}"))), args),
    )
}
fn cases_eq(major: Expr, hname: &str, goal: Expr, no: Expr, yes: Expr) -> Expr {
    let motive = lambda(
        "__return_case",
        bool_ty(),
        forall(hname, eq(major.clone(), v("__return_case")), goal),
    );
    let no = lambda(hname, eq(major.clone(), literal(false)), no);
    let yes = lambda(hname, eq(major.clone(), literal(true)), yes);
    apply(
        apply(c("Std.Bool.cases"), vec![motive, no, yes, major.clone()]),
        vec![refl(major)],
    )
}
fn absurd(ty: Expr, left: Expr, right: Expr, false_true: Expr) -> Typed {
    let context = lambda(
        "__impossible_flag",
        bool_ty(),
        choose_t(
            ty.clone(),
            v("__impossible_flag"),
            right.clone(),
            left.clone(),
        ),
    );
    typed(
        ty.clone(),
        left,
        right,
        apply(
            c("Std.Eq.congr"),
            vec![
                bool_ty(),
                ty,
                context,
                literal(false),
                literal(true),
                false_true,
            ],
        ),
    )
}
fn contradict(out: Typed, hfalse: Expr) -> Expr {
    assert_eq!(out.right, literal(true));
    next(
        backwards(typed(bool_ty(), out.left.clone(), literal(false), hfalse)),
        out,
    )
    .proof
}
fn successful_stop(e: Typed, kind: &str) -> Typed {
    next(projection(e, "Stopped"), named_projection(kind, "Stopped"))
}
fn valid(s: Expr) -> Expr {
    forall(
        "__failed",
        eq(stopped(s.clone()), literal(true)),
        apply(c(EQ), vec![cube_type(W), total(s), zero(W)]),
    )
}

fn step_success(p: &mut Producer, what: &str) -> Typed {
    let l = v("limit");
    let s = v("state");
    let x = v("item");
    let actual = step(l.clone(), s.clone(), x.clone());
    let (ty, left, right) = if what == "Payload" {
        (
            cube_type(W),
            total(actual.clone()),
            sum(W, total(s.clone()), x.clone(), literal(false)),
        )
    } else {
        (
            bool_ty(),
            item_check(l.clone(), total(s.clone()), x.clone()),
            literal(true),
        )
    };
    let goal = apply(c(EQ), vec![ty.clone(), left.clone(), right.clone()]);
    let bad_e = instantiated_step(
        "BadInput",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("input_running"), v("__item_sign")],
    );
    let bad_proof = absurd(
        ty.clone(),
        left.clone(),
        right.clone(),
        contradict(successful_stop(bad_e, "BadInput"), v("output_running")),
    )
    .proof;
    let fail_e = instantiated_step(
        "LimitExceeded",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("input_running"), v("__item_sign"), v("__remaining")],
    );
    let fail_proof = absurd(
        ty.clone(),
        left.clone(),
        right.clone(),
        contradict(
            successful_stop(fail_e, "LimitExceeded"),
            v("output_running"),
        ),
    )
    .proof;
    let next_e = instantiated_step(
        "Continue",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("input_running"), v("__item_sign"), v("__remaining")],
    );
    let next_proof = if what == "Payload" {
        next(
            projection(next_e, "Total"),
            pack_projection(
                sum(W, total(s.clone()), x.clone(), literal(false)),
                literal(false),
                literal(false),
                "Total",
            ),
        )
        .proof
    } else {
        let mut e = reflexive_t(bool_ty(), left.clone());
        e = change(
            p,
            e,
            typed(
                bool_ty(),
                sign(W, x.clone()),
                literal(false),
                v("__item_sign"),
            ),
        );
        e = change(
            p,
            e,
            typed(
                bool_ty(),
                remaining(l.clone(), s.clone(), x.clone()),
                literal(true),
                v("__remaining"),
            ),
        );
        let q = order::normalized(e.right.clone(), p);
        next(e, q).proof
    };
    let sign_false = cases_eq(
        remaining(l, s, x.clone()),
        "__remaining",
        goal.clone(),
        fail_proof,
        next_proof,
    );
    typed(
        ty,
        left,
        right,
        cases_eq(sign(W, x), "__item_sign", goal, sign_false, bad_proof),
    )
}
fn step_valid() -> Expr {
    let l = v("limit");
    let s = v("state");
    let x = v("item");
    let out = step(l.clone(), s.clone(), x.clone());
    let goal = apply(c(EQ), vec![cube_type(W), total(out.clone()), zero(W)]);
    let already = instantiated_step(
        "Stopped",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("__was_stopped")],
    );
    let old = next(
        projection(already, "Total"),
        typed(
            cube_type(W),
            total(s.clone()),
            zero(W),
            apply(v("valid"), vec![v("__was_stopped")]),
        ),
    )
    .proof;
    let bad_e = instantiated_step(
        "BadInput",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("__was_stopped"), v("__item_sign")],
    );
    let bad_proof = next(
        projection(bad_e, "Total"),
        named_projection("BadInput", "Total"),
    )
    .proof;
    let fail_e = instantiated_step(
        "LimitExceeded",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("__was_stopped"), v("__item_sign"), v("__remaining")],
    );
    let fail_proof = next(
        projection(fail_e, "Total"),
        named_projection("LimitExceeded", "Total"),
    )
    .proof;
    let go_e = instantiated_step(
        "Continue",
        l.clone(),
        s.clone(),
        x.clone(),
        vec![v("__was_stopped"), v("__item_sign"), v("__remaining")],
    );
    let stop = next(
        projection(go_e, "Stopped"),
        pack_projection(
            sum(W, total(s.clone()), x.clone(), literal(false)),
            literal(false),
            literal(false),
            "Stopped",
        ),
    );
    let impossible = next(
        backwards(stop),
        typed(
            bool_ty(),
            stopped(out.clone()),
            literal(true),
            v("__failed"),
        ),
    );
    let go_proof = absurd(cube_type(W), total(out), zero(W), impossible.proof).proof;
    let checked = cases_eq(
        remaining(l, s.clone(), x.clone()),
        "__remaining",
        goal.clone(),
        fail_proof,
        go_proof,
    );
    let sign_checked = cases_eq(sign(W, x), "__item_sign", goal.clone(), checked, bad_proof);
    let body = cases_eq(stopped(s), "__was_stopped", goal, sign_checked, old);
    lambda(
        "__failed",
        eq(
            stopped(step(v("limit"), v("state"), v("item"))),
            literal(true),
        ),
        body,
    )
}

pub(super) fn primitives(p: &mut Producer, trace: Option<&Path>) {
    let a = v("acc");
    let s = v("stopped");
    let b = v("bad");
    let tag = choose(v("__tag_5"), b.clone(), s.clone());
    let tag = cube_lambda(W, "__tag_", tag);
    let body = lambda(
        "__field",
        bool_ty(),
        choose_t(cube_type(W), v("__field"), tag, a.clone()),
    );
    let args = vec![
        ("acc".into(), cube_type(W)),
        ("stopped".into(), bool_ty()),
        ("bad".into(), bool_ty()),
    ];
    function_definition(p, "ReturnPack", &args, frame_ty(), body);
    let one = vec![("state".into(), frame_ty())];
    function_definition(
        p,
        "ReturnTotal",
        &one,
        cube_type(W),
        project(v("state"), false),
    );
    for field in ["Stopped", "Bad"] {
        let mut indices = vec![literal(false); W];
        if field == "Bad" {
            indices[W - 1] = literal(true);
        }
        function_definition(
            p,
            &format!("Return{field}"),
            &one,
            bool_ty(),
            apply(project(v("state"), true), indices),
        );
    }
    for field in ["Total", "Stopped", "Bad"] {
        let mut e = pack_projection(a.clone(), s.clone(), b.clone(), field);
        e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
        theorem(p, &format!("ReturnPack{field}"), &args, e);
    }
    for (name, s, b) in [
        ("ReturnInitial", false, false),
        ("ReturnBadInput", true, true),
        ("ReturnLimitExceeded", true, false),
    ] {
        function_definition(
            p,
            name,
            &[],
            frame_ty(),
            pack(zero(W), literal(s), literal(b)),
        );
    }
    let l = v("limit");
    let s = v("state");
    let x = v("item");
    let args = vec![
        ("limit".into(), cube_type(W)),
        ("state".into(), frame_ty()),
        ("item".into(), cube_type(W)),
    ];
    function_definition(
        p,
        "ReturnStep",
        &args,
        frame_ty(),
        step_body(l.clone(), s.clone(), x.clone()),
    );
    let mut e = step_definition(l.clone(), s.clone(), x.clone());
    e.proof = apply(c(REFL), vec![frame_ty(), e.right.clone()]);
    theorem(p, "ReturnStepDefinition", &args, e);
    for kind in ["Stopped", "BadInput", "LimitExceeded", "Continue"] {
        let mut binders = args.clone();
        binders.push((
            "hs".into(),
            eq(stopped(s.clone()), literal(kind == "Stopped")),
        ));
        let mut hs = vec![v("hs")];
        if kind != "Stopped" {
            binders.push((
                "hx".into(),
                eq(sign(W, x.clone()), literal(kind == "BadInput")),
            ));
            hs.push(v("hx"));
        }
        if kind == "LimitExceeded" || kind == "Continue" {
            binders.push((
                "hb".into(),
                eq(
                    remaining(l.clone(), s.clone(), x.clone()),
                    literal(kind == "Continue"),
                ),
            ));
            hs.push(v("hb"));
        }
        let e = step_case(p, kind, l.clone(), s.clone(), x.clone(), &hs);
        theorem(p, &format!("ReturnStep{kind}"), &binders, e);
    }
    let mut binders = args.clone();
    binders.push((
        "input_running".into(),
        eq(stopped(s.clone()), literal(false)),
    ));
    binders.push((
        "output_running".into(),
        eq(
            stopped(step(l.clone(), s.clone(), x.clone())),
            literal(false),
        ),
    ));
    for what in ["Payload", "Checks"] {
        let e = step_success(p, what);
        theorem(p, &format!("ReturnStepSuccess{what}"), &binders, e);
    }
    let mut binders = args;
    binders.push(("valid".into(), valid(s.clone())));
    let goal = valid(step(l, s, x));
    let proof = step_valid();
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("ReturnStepValid", ty, proof, true);
    quote::primitives(p);
    save_trace(p, trace, "returned-frame-primitives");
}
pub(super) fn build(p: &mut Producer, node: Node, trace: Option<&Path>) {
    run::build(p, node, trace);
    quote::build(p, node, trace);
}
pub(super) fn emit(base: Certificate, output: &Path, node: Node) {
    quote::emit(base, output, node);
}
