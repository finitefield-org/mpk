//! Root entry and actual success/failure conclusions for the private Quote model.
//! `empty` is an abstract byte-length test until the source binding is completed.
use super::*;

fn entry(empty: Expr, l: Expr) -> Expr {
    or(empty, sign(W, l))
}
fn quote(node: Node, empty: Expr, l: Expr, xs: Expr) -> Expr {
    call("ReturnQuote", node, vec![empty, l, xs])
}
fn data(node: Node) -> Vec<(String, Expr)> {
    vec![
        ("empty".into(), bool_ty()),
        ("limit".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
    ]
}
fn qargs() -> Vec<Expr> {
    vec![v("empty"), v("limit"), v("xs")]
}
fn result(node: Node) -> Expr {
    quote(node, v("empty"), v("limit"), v("xs"))
}
fn body(node: Node, empty: Expr, l: Expr, xs: Expr) -> Expr {
    choose_t(
        frame_ty(),
        entry(empty, l.clone()),
        bad_frame(),
        run::run(node, l, initial(), xs),
    )
}
fn definition(node: Node, empty: Expr, l: Expr, xs: Expr) -> Typed {
    typed(
        frame_ty(),
        quote(node, empty.clone(), l.clone(), xs.clone()),
        body(node, empty.clone(), l.clone(), xs.clone()),
        call("ReturnQuoteDefinition", node, vec![empty, l, xs]),
    )
}
fn entry_case(node: Node, empty: Expr, l: Expr, xs: Expr, truth: bool, h: Expr) -> Typed {
    typed(
        frame_ty(),
        quote(node, empty.clone(), l.clone(), xs.clone()),
        if truth {
            bad_frame()
        } else {
            run::run(node, l.clone(), initial(), xs.clone())
        },
        call(
            if truth {
                "ReturnQuoteBadEntry"
            } else {
                "ReturnQuoteGoodEntry"
            },
            node,
            vec![empty, l, xs, h],
        ),
    )
}
fn succeeded_entry(node: Node, empty: Expr, l: Expr, xs: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        entry(empty.clone(), l.clone()),
        literal(false),
        call("ReturnQuoteSuccessEntry", node, vec![empty, l, xs, h]),
    )
}
fn success(node: Node, suffix: &str) -> Expr {
    let mut args = qargs();
    args.push(v("output_running"));
    call(&format!("ReturnQuoteSuccess{suffix}"), node, args)
}
fn payload(node: Node) -> Typed {
    typed(
        cube_type(W),
        total(result(node)),
        folded(node, zero(W), v("xs")),
        success(node, "Payload"),
    )
}
fn onehot(s: Expr, b: Expr) -> Expr {
    let nil = not(s.clone());
    let bad = and(s.clone(), b.clone());
    let limit = and(s, not(b));
    and(
        or(nil.clone(), or(bad.clone(), limit.clone())),
        not(or(
            and(nil.clone(), bad.clone()),
            or(and(nil, limit.clone()), and(bad, limit)),
        )),
    )
}
pub(super) fn primitives(p: &mut Producer) {
    let a = v("a");
    let b = v("b");
    let original = choose(or(a.clone(), b.clone()), literal(false), b.clone());
    let proof = prove_with_checked_normalization(
        &eq(original.clone(), literal(false)),
        &[a.clone(), b.clone()],
        &mut p.fresh,
    );
    theorem(
        p,
        "ReturnEntryLimitGuarded",
        &[("a".into(), bool_ty()), ("b".into(), bool_ty())],
        typed(bool_ty(), original, literal(false), proof),
    );
    let goal = onehot(a.clone(), b.clone());
    let proof =
        prove_with_checked_normalization(&eq(goal.clone(), literal(true)), &[a, b], &mut p.fresh);
    theorem(
        p,
        "ReturnClosedPartition",
        &[("a".into(), bool_ty()), ("b".into(), bool_ty())],
        typed(bool_ty(), goal, literal(true), proof),
    );
}
fn success_entry_proof(node: Node) -> Typed {
    let empty = v("empty");
    let l = v("limit");
    let xs = v("xs");
    let e = entry(empty.clone(), l.clone());
    let goal = eq(e.clone(), literal(false));
    let error = entry_case(node, empty, l, xs, true, v("__entry"));
    let impossible = absurd(
        bool_ty(),
        e.clone(),
        literal(false),
        contradict(successful_stop(error, "BadInput"), v("output_running")),
    );
    typed(
        bool_ty(),
        e.clone(),
        literal(false),
        cases_eq(e, "__entry", goal, v("__entry"), impossible.proof),
    )
}
fn success_limit_proof(p: &mut Producer, node: Node) -> Typed {
    let b = sign(W, v("limit"));
    let e = entry(v("empty"), v("limit"));
    let original = choose(e.clone(), literal(false), b.clone());
    let h = succeeded_entry(node, v("empty"), v("limit"), v("xs"), v("output_running"));
    let selected = select(
        p,
        reflexive_t(bool_ty(), original.clone()),
        e,
        false,
        h.proof,
    );
    next(
        backwards(selected),
        typed(
            bool_ty(),
            original,
            literal(false),
            apply(c(&name("ReturnEntryLimitGuarded")), vec![v("empty"), b]),
        ),
    )
}
fn good_success(node: Node) -> (Typed, Expr) {
    let h = succeeded_entry(node, v("empty"), v("limit"), v("xs"), v("output_running"));
    let good = entry_case(node, v("empty"), v("limit"), v("xs"), false, h.proof);
    let hout = next(
        backwards(projection(good.clone(), "Stopped")),
        typed(
            bool_ty(),
            stopped(result(node)),
            literal(false),
            v("output_running"),
        ),
    )
    .proof;
    (good, hout)
}
fn success_payload_proof(p: &mut Producer, node: Node) -> Typed {
    let (good, hout) = good_success(node);
    let run_payload = run::payload(
        node,
        v("limit"),
        initial(),
        v("xs"),
        named_projection("Initial", "Stopped").proof,
        hout,
    );
    let e = next(projection(good, "Total"), run_payload);
    change(p, e, named_projection("Initial", "Total"))
}
fn success_checks_proof(p: &mut Producer, node: Node) -> Expr {
    let (_, hout) = good_success(node);
    let indices = addresses(node.depth, "__check_index_");
    let observed = apply(
        checks(node, v("limit"), total(initial()), v("xs")),
        indices.clone(),
    );
    let transport = replace(
        p,
        bool_ty(),
        observed.clone(),
        named_projection("Initial", "Total"),
    );
    let h = run::succeeded_checks(
        node,
        v("limit"),
        initial(),
        v("xs"),
        named_projection("Initial", "Stopped").proof,
        hout,
    );
    let e = next(
        backwards(transport),
        typed(bool_ty(), observed, literal(true), apply(h, indices)),
    );
    (0..node.depth).rev().fold(e.proof, |body, i| {
        lambda(&format!("__check_index_{i}"), bool_ty(), body)
    })
}
fn failure_proof(node: Node) -> Typed {
    let empty = v("empty");
    let l = v("limit");
    let xs = v("xs");
    let out = result(node);
    let goal = apply(c(EQ), vec![cube_type(W), total(out.clone()), zero(W)]);
    let bad = entry_case(
        node,
        empty.clone(),
        l.clone(),
        xs.clone(),
        true,
        v("__entry"),
    );
    let bad = next(
        projection(bad, "Total"),
        named_projection("BadInput", "Total"),
    );
    let good = entry_case(
        node,
        empty.clone(),
        l.clone(),
        xs.clone(),
        false,
        v("__entry"),
    );
    let hout = next(
        backwards(projection(good.clone(), "Stopped")),
        typed(bool_ty(), stopped(out.clone()), literal(true), v("failed")),
    )
    .proof;
    let init_valid = lambda(
        "__initial_failed",
        eq(stopped(initial()), literal(true)),
        named_projection("Initial", "Total").proof,
    );
    let h = apply(
        run::invariant(node, l.clone(), initial(), xs, init_valid),
        vec![hout],
    );
    let loop_zero = typed(cube_type(W), total(good.right.clone()), zero(W), h);
    let good = next(projection(good, "Total"), loop_zero);
    typed(
        cube_type(W),
        total(out),
        zero(W),
        cases_eq(entry(empty, l), "__entry", goal, good.proof, bad.proof),
    )
}
fn bounds_proof(p: &mut Producer, node: Node) -> Typed {
    let l = v("limit");
    let xs = v("xs");
    let hs = zero_hypotheses_with(l.clone(), success(node, "Limit"), success(node, "Checks"));
    let tail = state_claim(p, node, l.clone(), zero(W), xs, &hs);
    let goal = and(
        not(sign(W, total(result(node)))),
        order::compare(W, total(result(node)), l),
    );
    let e = change(p, reflexive_t(bool_ty(), goal), payload(node));
    next(e, tail)
}
fn sum_proof(p: &mut Producer, node: Node) -> Typed {
    let e = zero_math_with(
        p,
        node,
        v("limit"),
        v("xs"),
        success(node, "Limit"),
        success(node, "Checks"),
    );
    let context = lambda(
        "__return_sum_word",
        cube_type(W),
        apply(
            value(W, c("Std.Nat.succ"), v("__return_sum_word")),
            vec![v("initial")],
        ),
    );
    let q = payload(node);
    let left = apply(
        value(W, c("Std.Nat.succ"), q.left.clone()),
        vec![v("initial")],
    );
    let right = apply(
        value(W, c("Std.Nat.succ"), q.right.clone()),
        vec![v("initial")],
    );
    next(e, backwards(mapped(q, nat(), context, left, right)))
}
pub(super) fn build(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let args = data(node);
    let empty = v("empty");
    let l = v("limit");
    let xs = v("xs");
    function_definition(
        p,
        &symbol("ReturnQuote", node),
        &args,
        frame_ty(),
        body(node, empty.clone(), l.clone(), xs.clone()),
    );
    let mut d = definition(node, empty.clone(), l.clone(), xs.clone());
    d.proof = apply(c(REFL), vec![frame_ty(), d.right.clone()]);
    theorem(p, &symbol("ReturnQuoteDefinition", node), &args, d.clone());
    for truth in [false, true] {
        let mut a = args.clone();
        a.push((
            "entry_bad".into(),
            eq(entry(empty.clone(), l.clone()), literal(truth)),
        ));
        let e = select(
            p,
            d.clone(),
            entry(empty.clone(), l.clone()),
            truth,
            v("entry_bad"),
        );
        theorem(
            p,
            &symbol(
                if truth {
                    "ReturnQuoteBadEntry"
                } else {
                    "ReturnQuoteGoodEntry"
                },
                node,
            ),
            &a,
            e,
        );
    }
    let mut success_args = args.clone();
    success_args.push((
        "output_running".into(),
        eq(stopped(result(node)), literal(false)),
    ));
    let q = success_entry_proof(node);
    theorem(
        p,
        &symbol("ReturnQuoteSuccessEntry", node),
        &success_args,
        q,
    );
    let q = success_limit_proof(p, node);
    theorem(
        p,
        &symbol("ReturnQuoteSuccessLimit", node),
        &success_args,
        q,
    );
    let q = success_payload_proof(p, node);
    theorem(
        p,
        &symbol("ReturnQuoteSuccessPayload", node),
        &success_args,
        q,
    );
    let proof = success_checks_proof(p, node);
    let (ty, proof) = fold_binders(
        &success_args,
        all_checks(node, l.clone(), zero(W), xs, false),
        proof,
    );
    p.declare(&symbol("ReturnQuoteSuccessChecks", node), ty, proof, true);
    let mut failed_args = args.clone();
    failed_args.push(("failed".into(), eq(stopped(result(node)), literal(true))));
    let q = failure_proof(node);
    theorem(p, &symbol("ReturnQuoteFailureZero", node), &failed_args, q);
    let q = bounds_proof(p, node);
    theorem(
        p,
        &symbol("ReturnQuoteSuccessBounds", node),
        &success_args,
        q,
    );
    success_args.push(("initial".into(), nat()));
    let q = sum_proof(p, node);
    theorem(p, &symbol("ReturnQuoteSuccessSum", node), &success_args, q);
    save_trace(
        p,
        trace,
        &format!("returned-quote-{}-{}", node.depth, node.count),
    );
}
const FAMILIES: [&str; 7] = [
    "absorbing-return",
    "failure-zero",
    "success-bounds",
    "success-sum",
    "entry-error",
    "closed-outcome",
    "first-failure",
];
fn premises(family: &str) -> &str {
    match family {
        "absorbing-return" => {
            "actual input frame is stopped; all 128 input frame bits are arbitrary"
        }
        "first-failure" => {
            "actual low-prefix result is stopped; all 128 input frame bits are arbitrary"
        }
        "failure-zero" => "actual rooted return is stopped",
        "success-bounds" | "success-sum" => {
            "actual rooted return is running; entry and item checks are derived"
        }
        "entry-error" => "actual abstract empty-name or negative-limit entry guard is true",
        _ => "none; closed three-outcome representation only",
    }
}
pub(super) fn emit(base: Certificate, output: &Path, node: Node) {
    let mut cases = vec![];
    for family in FAMILIES {
        if family == "first-failure" && (node.count == 0 || node.depth == 0) {
            continue;
        }
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut args = data(node);
            let l = v("limit");
            let xs = v("xs");
            let out = result(node);
            let mut e = match family {
                "absorbing-return" | "first-failure" => {
                    args = vec![
                        ("limit".into(), cube_type(W)),
                        ("state".into(), frame_ty()),
                        ("xs".into(), cube_type(W + node.depth)),
                    ];
                    let s = v("state");
                    let observed = if family == "first-failure" {
                        run::run(
                            children(node).0,
                            l.clone(),
                            s.clone(),
                            project(xs.clone(), false),
                        )
                    } else {
                        s.clone()
                    };
                    args.push(("stopped".into(), eq(stopped(observed), literal(true))));
                    if family == "first-failure" {
                        run::first_failure(node, l, s, xs, v("stopped"))
                    } else {
                        run::absorb(node, l, s, xs, v("stopped"))
                    }
                }
                "failure-zero" => {
                    args.push(("failed".into(), eq(stopped(out.clone()), literal(true))));
                    let mut a = qargs();
                    a.push(v("failed"));
                    typed(
                        cube_type(W),
                        total(out.clone()),
                        zero(W),
                        call("ReturnQuoteFailureZero", node, a),
                    )
                }
                "success-bounds" | "success-sum" => {
                    args.push((
                        "output_running".into(),
                        if mutation == 0 {
                            eq(stopped(out.clone()), literal(false))
                        } else {
                            eq(literal(false), literal(false))
                        },
                    ));
                    if family == "success-sum" {
                        args.push(("initial".into(), nat()));
                        let mut a = qargs();
                        a.push(v("output_running"));
                        a.push(v("initial"));
                        typed(
                            nat(),
                            apply(
                                math(node, c("Std.Nat.succ"), xs.clone()),
                                vec![v("initial")],
                            ),
                            apply(
                                value(W, c("Std.Nat.succ"), total(out.clone())),
                                vec![v("initial")],
                            ),
                            call("ReturnQuoteSuccessSum", node, a),
                        )
                    } else {
                        typed(
                            bool_ty(),
                            and(
                                not(sign(W, total(out.clone()))),
                                order::compare(W, total(out.clone()), l.clone()),
                            ),
                            literal(true),
                            success(node, "Bounds"),
                        )
                    }
                }
                "entry-error" => {
                    args.push((
                        "entry_bad".into(),
                        eq(entry(v("empty"), l.clone()), literal(true)),
                    ));
                    entry_case(node, v("empty"), l, xs, true, v("entry_bad"))
                }
                "closed-outcome" => typed(
                    bool_ty(),
                    onehot(stopped(out.clone()), bad(out.clone())),
                    literal(true),
                    apply(
                        c(&name("ReturnClosedPartition")),
                        vec![stopped(out.clone()), bad(out)],
                    ),
                ),
                _ => unreachable!(),
            };
            if mutation == 1 {
                match family {
                    "absorbing-return" => {
                        e.right = pack(total(v("state")), literal(false), literal(false))
                    }
                    "first-failure" => e.right = v("state"),
                    "failure-zero" => {
                        args.push(("replacement".into(), cube_type(W)));
                        e.right = v("replacement");
                    }
                    "entry-error" => e.right = limit_frame(),
                    "closed-outcome" => e.right = literal(false),
                    _ => {}
                }
            }
            let suffix = format!("ReturnedClaim{}", family.replace('-', ""));
            let ty = apply(c(EQ), vec![e.ty.clone(), e.left.clone(), e.right.clone()]);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let id = format!(
                "returned-{}-{family}-{}",
                node.count,
                if mutation == 0 { "valid" } else { "altered" }
            );
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":node.count,"address_depth":node.depth,"storage_capacity":1usize<<node.depth,"source_word_bits":64,"quantified_limit_bits":64,"quantified_storage_bits":64usize<<node.depth,"frame_bits":128,"root_entry":family!="absorbing-return" && family!="first-failure","initial_natural_quantified":family=="success-sum","premises":premises(family),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"private full-payload halting Quote model; actual return status derives item checks and zero-entry arithmetic; Go source/operator/string/slice/descriptor binding remains pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.returned_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
