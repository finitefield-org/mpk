//! Two's-complement prefix cross sums under every actual sign-carry guard.
//! A balance law telescopes unsigned carries against input/result signs.
use super::*;

fn negative(f: Expr, x: Expr) -> Expr {
    conditional(weight(W, f), sign(W, x))
}
fn negatives(node: Node, f: Expr, xs: Expr) -> Expr {
    call("Negatives", node, vec![f, xs])
}
fn negatives_body(node: Node, f: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        unit()
    } else if node.depth == 0 {
        negative(f, xs)
    } else {
        let (lo, hi) = children(node);
        cmp(
            negatives(lo, f.clone(), project(xs.clone(), false)),
            negatives(hi, f, project(xs, true)),
        )
    }
}
fn negatives_unfold(node: Node, f: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("NegativesDefinition", node),
        vec![f.clone(), xs.clone()],
        negatives(node, f.clone(), xs.clone()),
        negatives_body(node, f, xs),
    )
}
fn guards(node: Node, acc: Expr, xs: Expr) -> Expr {
    call("SignedGuards", node, vec![acc, xs])
}
fn guards_body(node: Node, acc: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        cube_lambda(node.depth, "__unused_signed_", literal(true))
    } else if node.depth == 0 {
        guard(W, acc, xs, literal(false))
    } else {
        let (lo, hi) = children(node);
        let mid = folded(lo, acc.clone(), project(xs.clone(), false));
        lambda(
            "__signed_half",
            bool_ty(),
            choose_t(
                cube_type(node.depth - 1),
                v("__signed_half"),
                guards(hi, mid, project(xs.clone(), true)),
                guards(lo, acc, project(xs, false)),
            ),
        )
    }
}
fn safe(node: Node, acc: Expr, xs: Expr, unrelated: bool) -> Expr {
    let observed = if unrelated {
        literal(true)
    } else {
        apply(
            guards(node, acc, xs),
            addresses(node.depth, "__signed_index_"),
        )
    };
    (0..node.depth)
        .rev()
        .fold(eq(observed, literal(true)), |body, i| {
            forall(&format!("__signed_index_{i}"), bool_ty(), body)
        })
}
fn negative_fold(node: Node, f: Expr, acc: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("NegativeFold", node),
        vec![f.clone(), acc.clone(), xs.clone()],
        negative(f.clone(), folded(node, acc.clone(), xs.clone())),
        negative(f, fold_body(node, acc, xs)),
    )
}
fn balance(node: Node, f: Expr, acc: Expr, xs: Expr, h: Expr) -> Equality {
    instance(
        &symbol("SignedBalance", node),
        vec![f.clone(), acc.clone(), xs.clone(), h],
        cmp(
            overflow(node, f.clone(), acc.clone(), xs.clone()),
            negative(f.clone(), folded(node, acc.clone(), xs.clone())),
        ),
        cmp(negative(f.clone(), acc), negatives(node, f, xs)),
    )
}
fn relation(node: Node, f: Expr, acc: Expr, xs: Expr, h: Expr) -> Equality {
    instance(
        &symbol("SignedInvariant", node),
        vec![f.clone(), acc.clone(), xs.clone(), h],
        cmp(
            cmp(
                value(W, f.clone(), acc.clone()),
                math(node, f.clone(), xs.clone()),
            ),
            negative(f.clone(), folded(node, acc.clone(), xs.clone())),
        ),
        cmp(
            value(W, f.clone(), folded(node, acc.clone(), xs.clone())),
            cmp(negative(f.clone(), acc), negatives(node, f, xs)),
        ),
    )
}
fn definitions(p: &mut Producer, node: Node) {
    let acc = v("acc");
    let xs = v("xs");
    let f = v("f");
    let args = operands(node);
    let nb = vec![
        ("f".into(), function()),
        ("xs".into(), cube_type(W + node.depth)),
    ];
    function_definition(
        p,
        &symbol("Negatives", node),
        &nb,
        function(),
        negatives_body(node, f.clone(), xs.clone()),
    );
    p.theorem(
        &symbol("NegativesDefinition", node),
        &nb,
        computed(negatives_unfold(node, f.clone(), xs.clone())),
    );
    function_definition(
        p,
        &symbol("SignedGuards", node),
        &args,
        cube_type(node.depth),
        guards_body(node, acc.clone(), xs.clone()),
    );
    let from = folded(node, acc.clone(), xs.clone());
    let to = fold_body(node, acc.clone(), xs.clone());
    let e = typed(
        cube_type(W),
        from.clone(),
        to.clone(),
        apply(
            c(&name(&symbol("FoldDefinition", node))),
            vec![acc.clone(), xs.clone()],
        ),
    );
    let context = lambda(
        "__signed_folded",
        cube_type(W),
        negative(f.clone(), v("__signed_folded")),
    );
    let lifted = as_fn(mapped(
        e,
        function(),
        context,
        negative(f.clone(), from),
        negative(f.clone(), to),
    ));
    let mut binders = vec![("f".into(), function())];
    binders.extend(args.clone());
    p.theorem(&symbol("NegativeFold", node), &binders, lifted);
    if node.depth > 0 && node.count > 0 {
        let (lo, hi) = children(node);
        let mid = folded(lo, acc.clone(), project(xs.clone(), false));
        for high in [false, true] {
            let selected = if high {
                guards(hi, mid.clone(), project(xs.clone(), true))
            } else {
                guards(lo, acc.clone(), project(xs.clone(), false))
            };
            let observed = project(guards(node, acc.clone(), xs.clone()), high);
            let ty = apply(
                c(EQ),
                vec![cube_type(node.depth - 1), observed, selected.clone()],
            );
            let proof = apply(c(REFL), vec![cube_type(node.depth - 1), selected]);
            let (ty, proof) = fold_binders(&args, ty, proof);
            p.declare(
                &symbol(
                    if high {
                        "SignedGuardHigh"
                    } else {
                        "SignedGuardLow"
                    },
                    node,
                ),
                ty,
                proof,
                true,
            );
        }
    }
}
fn child_safe(node: Node, acc: Expr, xs: Expr, high: bool) -> Expr {
    let (lo, hi) = children(node);
    let child = if high { hi } else { lo };
    let mid = if high {
        folded(lo, acc.clone(), project(xs.clone(), false))
    } else {
        acc.clone()
    };
    let observed = project(guards(node, acc.clone(), xs.clone()), high);
    let selected = guards(child, mid, project(xs.clone(), high));
    let q = typed(
        cube_type(node.depth - 1),
        observed.clone(),
        selected.clone(),
        apply(
            c(&name(&symbol(
                if high {
                    "SignedGuardHigh"
                } else {
                    "SignedGuardLow"
                },
                node,
            ))),
            vec![acc, xs],
        ),
    );
    let indices = addresses(node.depth - 1, "__signed_child_");
    let context = lambda(
        "__signed_guard_view",
        cube_type(node.depth - 1),
        apply(v("__signed_guard_view"), indices.clone()),
    );
    let q = mapped(
        q,
        bool_ty(),
        context,
        apply(observed, indices.clone()),
        apply(selected, indices.clone()),
    );
    let mut parent_indices = vec![literal(high)];
    parent_indices.extend(indices);
    let h = typed(
        bool_ty(),
        q.left.clone(),
        literal(true),
        apply(v("safe"), parent_indices),
    );
    let e = next(backwards(q), h);
    (0..node.depth - 1).rev().fold(e.proof, |body, i| {
        lambda(&format!("__signed_child_{i}"), bool_ty(), body)
    })
}
pub(super) fn build(p: &mut Producer, node: Node, trace: Option<&Path>) {
    definitions(p, node);
    let f = v("f");
    let acc = v("acc");
    let xs = v("xs");
    let mut binders = vec![("f".into(), function())];
    binders.extend(operands(node));
    binders.push(("safe".into(), safe(node, acc.clone(), xs.clone(), false)));
    let target = balance(node, f.clone(), acc.clone(), xs.clone(), v("safe"));
    let mut e = p.rewrite(
        target.left,
        overflow_unfold(node, f.clone(), acc.clone(), xs.clone()),
    );
    e = p.change(e, negative_fold(node, f.clone(), acc.clone(), xs.clone()));
    if node.count == 0 {
        let n = negative(f.clone(), acc.clone());
        e = transit(
            e,
            instance(
                "PrefixIdentityCommutes",
                vec![n.clone()],
                cmp(unit(), n.clone()),
                cmp(n, unit()),
            ),
        );
    } else if node.depth == 0 {
        let actual = guard(W, acc.clone(), xs.clone(), literal(false));
        let observed = guards(node, acc.clone(), xs.clone());
        let q = typed(bool_ty(), actual.clone(), observed.clone(), refl(actual));
        let h = next(q, typed(bool_ty(), observed, literal(true), v("safe")));
        let step = Equality {
            left: e.right.clone(),
            right: cmp(
                negative(f.clone(), acc.clone()),
                negative(f.clone(), xs.clone()),
            ),
            proof: apply(
                c(&name(&format!("TreeSignedBalance{W}"))),
                vec![f.clone(), acc.clone(), xs.clone(), literal(false), h.proof],
            ),
        };
        e = transit(e, step);
    } else {
        let (lo, hi) = children(node);
        let xl = project(xs.clone(), false);
        let xh = project(xs.clone(), true);
        let mid = folded(lo, acc.clone(), xl.clone());
        let ol = overflow(lo, f.clone(), acc.clone(), xl.clone());
        let oh = overflow(hi, f.clone(), mid.clone(), xh.clone());
        let nf = negative(f.clone(), folded(hi, mid.clone(), xh.clone()));
        let norm = p.monoid(e.right.clone(), cmp(ol.clone(), cmp(oh, nf)));
        e = transit(e, norm);
        e = p.change(
            e,
            balance(
                hi,
                f.clone(),
                mid.clone(),
                xh.clone(),
                child_safe(node, acc.clone(), xs.clone(), true),
            ),
        );
        let nh = negatives(hi, f.clone(), xh);
        let norm = p.monoid(
            e.right.clone(),
            cmp(cmp(ol, negative(f.clone(), mid)), nh.clone()),
        );
        e = transit(e, norm);
        e = p.change(
            e,
            balance(
                lo,
                f.clone(),
                acc.clone(),
                xl.clone(),
                child_safe(node, acc.clone(), xs.clone(), false),
            ),
        );
        let norm = p.monoid(
            e.right.clone(),
            cmp(
                negative(f.clone(), acc.clone()),
                cmp(negatives(lo, f.clone(), xl), nh),
            ),
        );
        e = transit(e, norm);
    }
    e = p.change(e, reverse(negatives_unfold(node, f.clone(), xs.clone())));
    assert_eq!(e.right, target.right);
    p.theorem(&symbol("SignedBalance", node), &binders, e);
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-signed-balance", node.depth, node.count),
    );
    let r = relation(node, f.clone(), acc.clone(), xs.clone(), v("safe"));
    let mut e = Producer::on_left(
        raw_invariant(node, f.clone(), acc.clone(), xs.clone()),
        negative(f.clone(), folded(node, acc.clone(), xs.clone())),
    );
    let vf = value(W, f.clone(), folded(node, acc.clone(), xs.clone()));
    let norm = p.monoid(
        e.right.clone(),
        cmp(
            vf,
            cmp(
                overflow(node, f.clone(), acc.clone(), xs.clone()),
                negative(f.clone(), folded(node, acc.clone(), xs.clone())),
            ),
        ),
    );
    e = transit(e, norm);
    e = p.change(e, balance(node, f, acc, xs, v("safe")));
    assert_eq!(e.left, r.left);
    assert_eq!(e.right, r.right);
    p.theorem(&symbol("SignedInvariant", node), &binders, e);
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-signed-invariant", node.depth, node.count),
    );
}
fn claim(p: &mut Producer, node: Node, mutation: usize) -> Expr {
    let f = c("Std.Nat.succ");
    let acc = v("acc");
    let xs = v("xs");
    let e = relation(node, f.clone(), acc.clone(), xs.clone(), v("safe"));
    let left = if mutation == 1 {
        cmp(
            value(W, f.clone(), acc.clone()),
            math(node, f.clone(), xs.clone()),
        )
    } else if mutation == 3 {
        cmp(
            math(node, f.clone(), xs.clone()),
            negative(f, folded(node, acc.clone(), xs.clone())),
        )
    } else {
        e.left.clone()
    };
    let context = lambda(
        "__signed_prefix_eval",
        function(),
        apply(v("__signed_prefix_eval"), vec![v("initial")]),
    );
    let proof = apply(
        c("Std.Eq.congr"),
        vec![function(), nat(), context, e.left, e.right.clone(), e.proof],
    );
    let goal = apply(
        c(EQ),
        vec![
            nat(),
            apply(left, vec![v("initial")]),
            apply(e.right, vec![v("initial")]),
        ],
    );
    let binders = vec![
        ("initial".into(), nat()),
        ("acc".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
        ("safe".into(), safe(node, acc, xs, mutation == 2)),
    ];
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("SignedPrefixClaim", ty.clone(), proof, true);
    ty
}
pub(super) fn emit(base: Certificate, output: &Path, node: Node) {
    let mut cases = vec![];
    for mutation in 0..4 {
        let id = format!(
            "signed-prefix-{}-{}",
            node.count,
            match mutation {
                0 => "exact-cross-sum",
                1 => "dropped-result-sign",
                2 =>
                    if node.count == 0 {
                        "vacuous-empty-guard"
                    } else {
                        "unrelated-guard"
                    },
                _ => "dropped-initial",
            }
        );
        let mut p = Producer {
            builder: Builder::from(base.clone()),
            fresh: 0,
        };
        let ty = claim(&mut p, node, mutation);
        let terms = p.builder.c.term_table.len();
        let declarations = p.builder.c.declarations.len();
        let bytes = p.builder.finish();
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        cases.push(json!({"id":id,"count":node.count,"address_depth":node.depth,"storage_capacity":1usize<<node.depth,"source_word_bits":64,
            "quantified_storage_bits":64usize<<node.depth,"quantified_accumulator_bits":64,"initial_natural_quantified":true,"mutation":mutation,
            "expected":if mutation==0 || node.count==0 && mutation==2 {"accepted"} else {"rejected"},
            "declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name("SignedPrefixClaim"),
            "premise":if mutation==2 {"all inactive true equalities (vacuous only for empty prefix)"} else {"each actual intermediate sign-stage incoming/outgoing carry agrees"},
            "scope":"conditional exact fixed-count modular64 prefix cross sums for two's-complement differences; Go source guard/order bindings remain pending",
            "goal_debug":format!("{ty:?}")}));
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.signed_prefix_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
