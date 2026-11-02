//! Ordinary arbitrary-Nat rank termination on complete 128-bit frames.
//! All premises are explicit obligations. No Go source/totality claim is made.
use super::*;

fn frame() -> Expr {
    cube_type(7)
}
fn unary(out: Expr) -> Expr {
    forall("__state_arg", frame(), out)
}
fn eqt(t: Expr, a: Expr, b: Expr) -> Expr {
    apply(c(EQ), vec![t, a, b])
}
fn rt(t: Expr, a: Expr) -> Expr {
    apply(c(REFL), vec![t, a])
}
fn ct(a: Expr, b: Expr, f: Expr, x: Expr, y: Expr, p: Expr) -> Expr {
    apply(c("Std.Eq.congr"), vec![a, b, f, x, y, p])
}
fn tt(t: Expr, x: Expr, y: Expr, z: Expr, p: Expr, q: Expr) -> Expr {
    apply(c("Std.Eq.trans"), vec![t, x, y, z, p, q])
}
fn halted(s: Expr) -> Expr {
    apply(v("halted"), vec![s])
}
fn next(s: Expr) -> Expr {
    apply(v("next"), vec![s])
}
fn inv(s: Expr) -> Expr {
    eq(apply(v("invariant"), vec![s]), literal(true))
}
fn rank(s: Expr) -> Expr {
    apply(v("rank"), vec![s])
}
fn step(s: Expr) -> Expr {
    app("VStep", vec![v("halted"), v("next"), s])
}
fn step_function() -> Expr {
    lambda("__step_state", frame(), step(v("__step_state")))
}
fn iter(f: Expr, n: Expr, s: Expr) -> Expr {
    app("VIter", vec![f, n, s])
}
fn run(n: Expr, s: Expr) -> Expr {
    iter(step_function(), n, s)
}
fn stopped(n: Expr, s: Expr) -> Expr {
    eq(halted(run(n, s)), literal(true))
}
fn control_args() -> Vec<(String, Expr)> {
    vec![
        ("halted".into(), unary(bool_ty())),
        ("next".into(), unary(frame())),
    ]
}

fn trans_type(a: Expr, b: Expr, n: Expr) -> Expr {
    forall(
        "hab",
        lt(a.clone(), b.clone()),
        forall("hbc", lt(b, n.clone()), lt(a, n)),
    )
}
fn trans_body() -> Expr {
    let n = v("tn");
    let a = v("ta");
    let b = v("tb");
    let args = |a: Expr, b: Expr, n: Expr| {
        vec![("hab".into(), lt(a, b.clone())), ("hbc".into(), lt(b, n))]
    };
    let zero_case = wrapped(
        &[("ta".into(), nt()), ("tb".into(), nt())],
        wrapped(
            &args(a.clone(), b.clone(), zero()),
            app("FalseElim", vec![lt(a.clone(), zero()), v("hbc")]),
        ),
    );
    let a_zero = lambda(
        "tb",
        nt(),
        wrapped(
            &args(zero(), b.clone(), succ(n.clone())),
            refl(literal(true)),
        ),
    );
    let b_zero = wrapped(
        &args(succ(a.clone()), zero(), succ(n.clone())),
        app(
            "FalseElim",
            vec![lt(succ(a.clone()), succ(n.clone())), v("hab")],
        ),
    );
    let b_step = lambda(
        "tb",
        nt(),
        lambda(
            "_bih",
            trans_type(succ(a.clone()), b.clone(), succ(n.clone())),
            wrapped(
                &args(succ(a.clone()), succ(b.clone()), succ(n.clone())),
                apply(v("tih"), vec![a.clone(), b.clone(), v("hab"), v("hbc")]),
            ),
        ),
    );
    let a_step = lambda(
        "ta",
        nt(),
        lambda(
            "_aih",
            forall(
                "tb",
                nt(),
                trans_type(a.clone(), b.clone(), succ(n.clone())),
            ),
            lambda(
                "tb",
                nt(),
                induct(
                    lambda(
                        "tb",
                        nt(),
                        trans_type(succ(a.clone()), b.clone(), succ(n.clone())),
                    ),
                    b_zero,
                    b_step,
                    b.clone(),
                ),
            ),
        ),
    );
    let n_step = lambda(
        "tn",
        nt(),
        lambda(
            "tih",
            forall(
                "ta",
                nt(),
                forall("tb", nt(), trans_type(a.clone(), b.clone(), n.clone())),
            ),
            lambda(
                "ta",
                nt(),
                induct(
                    lambda(
                        "ta",
                        nt(),
                        forall(
                            "tb",
                            nt(),
                            trans_type(a.clone(), b.clone(), succ(n.clone())),
                        ),
                    ),
                    a_zero,
                    a_step,
                    a.clone(),
                ),
            ),
        ),
    );
    let proof = induct(
        lambda(
            "tn",
            nt(),
            forall(
                "ta",
                nt(),
                forall("tb", nt(), trans_type(a.clone(), b.clone(), n.clone())),
            ),
        ),
        zero_case,
        n_step,
        v("n"),
    );
    apply(proof, vec![v("a"), v("b"), v("hab"), v("hbc")])
}

fn bound_args() -> Vec<(String, Expr)> {
    vec![
        ("a".into(), nt()),
        ("b".into(), nt()),
        ("n".into(), nt()),
        ("hab".into(), lt(v("a"), v("b"))),
        ("hbn".into(), lt(v("b"), succ(v("n")))),
    ]
}
fn bound_body() -> Expr {
    let lower = lambda(
        "hlo",
        lt(v("b"), v("n")),
        app(
            "VLessTrans",
            vec![v("a"), v("b"), v("n"), v("hab"), v("hlo")],
        ),
    );
    let equal = lambda(
        "heq",
        neq(v("b"), v("n")),
        apply(
            c("Std.Eq.rewrite"),
            vec![
                nt(),
                v("b"),
                v("n"),
                lambda("bound", nt(), lt(v("a"), v("bound"))),
                v("heq"),
                v("hab"),
            ],
        ),
    );
    app(
        "Split",
        vec![lt(v("a"), v("n")), v("n"), v("b"), lower, equal, v("hbn")],
    )
}
fn one_step_proof(value: bool) -> Expr {
    let s = v("s");
    ct(
        bool_ty(),
        frame(),
        lambda(
            "test",
            bool_ty(),
            choose_t(frame(), v("test"), s.clone(), next(s.clone())),
        ),
        halted(s),
        literal(value),
        v("guard"),
    )
}
fn absorb_body() -> Expr {
    let s = v("s");
    let k = v("ak");
    let previous = run(k.clone(), s.clone());
    let first = step(previous.clone());
    let middle = step(s.clone());
    let h = ct(
        frame(),
        frame(),
        step_function(),
        previous,
        s.clone(),
        v("aih"),
    );
    let last = app(
        "VStepStopped",
        vec![v("halted"), v("next"), s.clone(), v("guard")],
    );
    let induction_step = lambda(
        "ak",
        nt(),
        lambda(
            "aih",
            eqt(frame(), run(k, s.clone()), s.clone()),
            tt(frame(), first, middle, s.clone(), h, last),
        ),
    );
    induct(
        lambda("ak", nt(), eqt(frame(), run(v("ak"), s.clone()), s.clone())),
        rt(frame(), s),
        induction_step,
        v("n"),
    )
}
fn advance_body() -> Expr {
    let at = |n| iter(v("f"), n, v("s"));
    let shifted = |n| iter(v("f"), n, apply(v("f"), vec![v("s")]));
    let k = v("advk");
    let p = ct(
        frame(),
        frame(),
        v("f"),
        at(succ(k.clone())),
        shifted(k.clone()),
        v("advih"),
    );
    induct(
        lambda(
            "advk",
            nt(),
            eqt(frame(), at(succ(v("advk"))), shifted(v("advk"))),
        ),
        rt(frame(), apply(v("f"), vec![v("s")])),
        lambda(
            "advk",
            nt(),
            lambda("advih", eqt(frame(), at(succ(k.clone())), shifted(k)), p),
        ),
        v("n"),
    )
}

fn termination_args(mutation: &str) -> Vec<(String, Expr)> {
    let mut a = control_args();
    a.extend([
        ("invariant".into(), unary(bool_ty())),
        ("rank".into(), unary(nt())),
    ]);
    let obligation = |conclusion| {
        forall(
            "vs",
            frame(),
            forall(
                "vi",
                inv(v("vs")),
                forall(
                    "vh",
                    eq(halted(v("vs")), literal(false)),
                    forall(
                        "vnext_live",
                        eq(halted(next(v("vs"))), literal(false)),
                        conclusion,
                    ),
                ),
            ),
        )
    };
    let trivial = || eq(literal(true), literal(true));
    let preservation = if mutation == "preservation" {
        trivial()
    } else {
        inv(next(v("vs")))
    };
    let decrease = if mutation == "decrease" {
        neq(rank(next(v("vs"))), rank(v("vs")))
    } else {
        lt(rank(next(v("vs"))), rank(v("vs")))
    };
    a.extend([
        ("preserve".into(), obligation(preservation)),
        ("decrease".into(), obligation(decrease)),
        ("n".into(), nt()),
        ("s".into(), frame()),
        (
            "entry".into(),
            if mutation == "entry" {
                trivial()
            } else {
                inv(v("s"))
            },
        ),
        (
            "bound".into(),
            if mutation == "bound" {
                trivial()
            } else {
                lt(
                    rank(v("s")),
                    if mutation == "weak-bound" {
                        succ(v("n"))
                    } else {
                        v("n")
                    },
                )
            },
        ),
    ]);
    a
}
fn induction_type(n: Expr) -> Expr {
    forall(
        "s",
        frame(),
        forall(
            "entry",
            inv(v("s")),
            forall("bound", lt(rank(v("s")), n.clone()), stopped(n, v("s"))),
        ),
    )
}
fn termination_body() -> Expr {
    let s = v("s");
    let k = v("vk");
    let count = succ(k.clone());
    let goal = stopped(count.clone(), s.clone());
    let local_args = |n: Expr| {
        vec![
            ("s".into(), frame()),
            ("entry".into(), inv(s.clone())),
            ("bound".into(), lt(rank(s.clone()), n)),
        ]
    };
    let base = wrapped(
        &local_args(zero()),
        app("FalseElim", vec![stopped(zero(), s.clone()), v("bound")]),
    );
    // A stopped state stays identical, including every frame bit.
    let absorb = app(
        "VAbsorb",
        vec![v("halted"), v("next"), s.clone(), v("guard"), count.clone()],
    );
    let hrun = halted(run(count.clone(), s.clone()));
    let true_result = tt(
        bool_ty(),
        hrun.clone(),
        halted(s.clone()),
        literal(true),
        ct(
            frame(),
            bool_ty(),
            v("halted"),
            run(count.clone(), s.clone()),
            s.clone(),
            absorb,
        ),
        v("guard"),
    );
    // Only continuing backedges preserve I and strictly lower rank.
    let ih_entry = apply(
        v("preserve"),
        vec![s.clone(), v("entry"), v("guard"), v("next_guard")],
    );
    let decrease = apply(
        v("decrease"),
        vec![s.clone(), v("entry"), v("guard"), v("next_guard")],
    );
    let ih_bound = app(
        "VBelowSucc",
        vec![
            rank(next(s.clone())),
            rank(s.clone()),
            k.clone(),
            decrease,
            v("bound"),
        ],
    );
    let ih = apply(v("vih"), vec![next(s.clone()), ih_entry, ih_bound]);
    // Exiting transitions require no rank decrease or next-entry invariant.
    let next_stopped = app(
        "VAbsorb",
        vec![
            v("halted"),
            v("next"),
            next(s.clone()),
            v("next_guard"),
            k.clone(),
        ],
    );
    let next_h = ct(
        frame(),
        bool_ty(),
        v("halted"),
        run(k.clone(), next(s.clone())),
        next(s.clone()),
        next_stopped,
    );
    let next_exit = tt(
        bool_ty(),
        halted(run(k.clone(), next(s.clone()))),
        halted(next(s.clone())),
        literal(true),
        next_h,
        v("next_guard"),
    );
    let next_goal = stopped(k.clone(), next(s.clone()));
    let next_stop = apply(
        c("Std.Bool.cases"),
        vec![
            lambda(
                "next_test",
                bool_ty(),
                forall(
                    "next_guard",
                    eq(halted(next(s.clone())), v("next_test")),
                    next_goal,
                ),
            ),
            lambda(
                "next_guard",
                eq(halted(next(s.clone())), literal(false)),
                ih,
            ),
            lambda(
                "next_guard",
                eq(halted(next(s.clone())), literal(true)),
                next_exit,
            ),
            halted(next(s.clone())),
            refl(halted(next(s.clone()))),
        ],
    );
    let middle = iter(step_function(), k.clone(), step(s.clone()));
    let final_frame = run(k.clone(), next(s.clone()));
    let advance = app("VAdvance", vec![step_function(), k.clone(), s.clone()]);
    let next_eq = app(
        "VStepNext",
        vec![v("halted"), v("next"), s.clone(), v("guard")],
    );
    let shift = ct(
        frame(),
        frame(),
        lambda("as", frame(), run(k.clone(), v("as"))),
        step(s.clone()),
        next(s.clone()),
        next_eq,
    );
    let frames = tt(
        frame(),
        run(count.clone(), s.clone()),
        middle,
        final_frame.clone(),
        advance,
        shift,
    );
    let hs = ct(
        frame(),
        bool_ty(),
        v("halted"),
        run(count.clone(), s.clone()),
        final_frame.clone(),
        frames,
    );
    let false_result = tt(
        bool_ty(),
        hrun,
        halted(final_frame),
        literal(true),
        hs,
        next_stop,
    );
    let motive = lambda(
        "test",
        bool_ty(),
        forall("guard", eq(halted(s.clone()), v("test")), goal),
    );
    let split = apply(
        c("Std.Bool.cases"),
        vec![
            motive,
            lambda("guard", eq(halted(s.clone()), literal(false)), false_result),
            lambda("guard", eq(halted(s.clone()), literal(true)), true_result),
            halted(s.clone()),
            refl(halted(s.clone())),
        ],
    );
    let induction_step = lambda(
        "vk",
        nt(),
        lambda("vih", induction_type(k), wrapped(&local_args(count), split)),
    );
    let all = induct(
        lambda("vk", nt(), induction_type(v("vk"))),
        base,
        induction_step,
        v("n"),
    );
    apply(all, vec![v("s"), v("entry"), v("bound")])
}

fn variant_library(root: &Path) -> Certificate {
    let mut l = Library {
        b: Builder::from(library(root)),
    };
    l.declare(
        "VLessTrans",
        &[
            ("a".into(), nt()),
            ("b".into(), nt()),
            ("n".into(), nt()),
            ("hab".into(), lt(v("a"), v("b"))),
            ("hbc".into(), lt(v("b"), v("n"))),
        ],
        lt(v("a"), v("n")),
        trans_body(),
        true,
    );
    l.declare(
        "VBelowSucc",
        &bound_args(),
        lt(v("a"), v("n")),
        bound_body(),
        true,
    );
    let a = vec![
        ("f".into(), unary(frame())),
        ("n".into(), nt()),
        ("s".into(), frame()),
    ];
    let body = induct(
        lambda("_iter_n", nt(), frame()),
        v("s"),
        lambda(
            "_iter_n",
            nt(),
            lambda("_iter_s", frame(), apply(v("f"), vec![v("_iter_s")])),
        ),
        v("n"),
    );
    l.declare("VIter", &a, frame(), body, false);
    let mut a = control_args();
    a.push(("s".into(), frame()));
    l.declare(
        "VStep",
        &a,
        frame(),
        choose_t(frame(), halted(v("s")), v("s"), next(v("s"))),
        false,
    );
    for (suffix, guard, target) in [
        ("VStepStopped", true, v("s")),
        ("VStepNext", false, next(v("s"))),
    ] {
        let mut a = a.clone();
        a.push(("guard".into(), eq(halted(v("s")), literal(guard))));
        l.declare(
            suffix,
            &a,
            eqt(frame(), step(v("s")), target),
            one_step_proof(guard),
            true,
        );
    }
    a.extend([
        ("guard".into(), eq(halted(v("s")), literal(true))),
        ("n".into(), nt()),
    ]);
    l.declare(
        "VAbsorb",
        &a,
        eqt(frame(), run(v("n"), v("s")), v("s")),
        absorb_body(),
        true,
    );
    l.declare(
        "VAdvance",
        &[
            ("f".into(), unary(frame())),
            ("n".into(), nt()),
            ("s".into(), frame()),
        ],
        eqt(
            frame(),
            iter(v("f"), succ(v("n")), v("s")),
            iter(v("f"), v("n"), apply(v("f"), vec![v("s")])),
        ),
        advance_body(),
        true,
    );
    l.declare(
        "VTerminates",
        &termination_args(""),
        stopped(v("n"), v("s")),
        termination_body(),
        true,
    );
    let self_goal = |n: Expr| lt(n.clone(), succ(n));
    let self_proof = induct(
        lambda("self_n", nt(), self_goal(v("self_n"))),
        refl(literal(true)),
        lambda(
            "self_n",
            nt(),
            lambda("self_ih", self_goal(v("self_n")), v("self_ih")),
        ),
        v("n"),
    );
    l.declare(
        "VSelfBound",
        &[("n".into(), nt())],
        self_goal(v("n")),
        self_proof,
        true,
    );
    let args = rank_arguments();
    let proof = rank_instance();
    l.declare(
        "VRankTerminates",
        &args,
        stopped(succ(rank(v("s"))), v("s")),
        proof,
        true,
    );
    l.b.c
}

fn rank_arguments() -> Vec<(String, Expr)> {
    termination_args("")
        .into_iter()
        .filter(|(n, _)| n != "n" && n != "bound")
        .collect()
}
fn rank_instance() -> Expr {
    app(
        "VTerminates",
        vec![
            v("halted"),
            v("next"),
            v("invariant"),
            v("rank"),
            v("preserve"),
            v("decrease"),
            succ(rank(v("s"))),
            v("s"),
            v("entry"),
            app("VSelfBound", vec![rank(v("s"))]),
        ],
    )
}

fn zero_rank_exit() -> (Vec<(String, Expr)>, Expr, Expr) {
    // Read one exit flag from an arbitrary full frame. The transition sets
    // every bit, including that flag. Rank remains zero on this exiting edge.
    let h = lambda(
        "exit_s",
        frame(),
        apply(v("exit_s"), vec![literal(false); 7]),
    );
    let f = lambda(
        "exit_s",
        frame(),
        cube_lambda(7, "exit_addr", literal(true)),
    );
    let i = lambda("exit_s", frame(), literal(true));
    let r = lambda("exit_s", frame(), zero());
    let instance = |mut e: Expr| {
        for (name, value) in [
            ("halted", &h),
            ("next", &f),
            ("invariant", &i),
            ("rank", &r),
        ] {
            e = substitute(&e, &v(name), value);
        }
        e
    };
    let premises = vec![
        ("vs".into(), frame()),
        ("vi".into(), instance(inv(v("vs")))),
        ("vh".into(), instance(eq(halted(v("vs")), literal(false)))),
        (
            "vnext_live".into(),
            instance(eq(halted(next(v("vs"))), literal(false))),
        ),
    ];
    let preserve = wrapped(&premises, refl(literal(true)));
    let impossible = apply(
        c("Std.Eq.symm"),
        vec![bool_ty(), literal(true), literal(false), v("vnext_live")],
    );
    let decrease = wrapped(
        &premises,
        app(
            "FalseElim",
            vec![instance(lt(rank(next(v("vs"))), rank(v("vs")))), impossible],
        ),
    );
    let goal = instance(stopped(succ(rank(v("s"))), v("s")));
    let proof = app(
        "VRankTerminates",
        vec![h, f, i, r, preserve, decrease, v("s"), refl(literal(true))],
    );
    (vec![("s".into(), frame())], goal, proof)
}

pub(super) fn generate(root: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let base = variant_library(root);
    let mut rows = vec![];
    for mutation in [
        "valid",
        "preservation",
        "decrease",
        "entry",
        "bound",
        "weak-bound",
        "result",
        "rank-count-valid",
        "rank-count-off-by-one",
        "zero-rank-exit",
    ] {
        let mut l = Library {
            b: Builder::from(base.clone()),
        };
        let rank_case = mutation.starts_with("rank-count-");
        let goal = if rank_case {
            stopped(
                if mutation == "rank-count-off-by-one" {
                    rank(v("s"))
                } else {
                    succ(rank(v("s")))
                },
                v("s"),
            )
        } else if mutation == "result" {
            eq(halted(run(v("n"), v("s"))), literal(false))
        } else {
            stopped(v("n"), v("s"))
        };
        let args = if rank_case {
            rank_arguments()
        } else {
            termination_args(mutation)
        };
        let proof = app(
            if rank_case {
                "VRankTerminates"
            } else {
                "VTerminates"
            },
            args.iter().map(|(n, _)| v(n)).collect(),
        );
        let (args, goal, proof) = if mutation == "zero-rank-exit" {
            zero_rank_exit()
        } else {
            (args, goal, proof)
        };
        l.declare("VariantClaim", &args, goal, proof, true);
        let terms = l.b.c.term_table.len();
        let declarations = l.b.c.declarations.len();
        let bytes = l.b.finish();
        let id = format!("variant-{mutation}");
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        rows.push(json!({"id":id,"mutation":mutation,"expected":if mutation=="valid" || mutation=="rank-count-valid" || mutation=="zero-rank-exit" {"accepted"} else {"rejected"},"bytes":bytes.len(),"terms":terms,"declarations":declarations,"theorem":named("VariantClaim"),"frame_bits":128,"iteration_domain":if mutation=="zero-rank-exit" {"rank zero; arbitrary full frame; universal theorem instance"} else {"arbitrary genuine Std.Nat; no finite unrolling"},"scope":"ordinary rank-bound termination schema; preservation and strict decrease only on continuing backedges; general bound explicit, rank+1 corollary derives it; no Go source totality claim"}));
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.variant_termination_pilot.v0","cases":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
