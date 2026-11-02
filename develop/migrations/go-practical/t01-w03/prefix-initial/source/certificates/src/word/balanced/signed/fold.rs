//! A real left-to-right modular64 accumulator related to an exact Nat sum.
//! The proof parameter refers to every actual step carry, not callee truth.
//! Partial final trees retain all unused backing values as quantified inputs.

use super::*;
use std::collections::BTreeSet;

const W: usize = 6;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Node {
    depth: usize,
    count: usize,
}
fn symbol(base: &str, node: Node) -> String {
    format!("Prefix{base}{}_{}", node.depth, node.count)
}
fn call(base: &str, node: Node, args: Vec<Expr>) -> Expr {
    apply(c(&name(&symbol(base, node))), args)
}
fn folded(node: Node, acc: Expr, xs: Expr) -> Expr {
    call("Fold", node, vec![acc, xs])
}
fn math(node: Node, f: Expr, xs: Expr) -> Expr {
    call("Math", node, vec![f, xs])
}
fn carries(node: Node, acc: Expr, xs: Expr) -> Expr {
    call("Carries", node, vec![acc, xs])
}
fn children(node: Node) -> (Node, Node) {
    assert!(node.depth > 0 && node.count > 0);
    let half = 1usize << (node.depth - 1);
    let low = node.count.min(half);
    (
        Node {
            depth: node.depth - 1,
            count: low,
        },
        Node {
            depth: node.depth - 1,
            count: node.count - low,
        },
    )
}
fn collect(node: Node, nodes: &mut BTreeSet<Node>) {
    if !nodes.insert(node) {
        return;
    }
    if node.depth > 0 && node.count > 0 {
        let (lo, hi) = children(node);
        collect(lo, nodes);
        collect(hi, nodes);
    }
}
fn safety(node: Node, acc: Expr, xs: Expr, unrelated: bool) -> Expr {
    let args = addresses(node.depth, "__safe_index_");
    let observed = if unrelated {
        literal(false)
    } else {
        apply(carries(node, acc, xs), args)
    };
    let mut ty = eq(observed, literal(false));
    for i in (0..node.depth).rev() {
        ty = forall(&format!("__safe_index_{i}"), bool_ty(), ty);
    }
    ty
}
fn fold_body(node: Node, acc: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        acc
    } else if node.depth == 0 {
        sum(W, acc, xs, literal(false))
    } else {
        let (lo, hi) = children(node);
        folded(
            hi,
            folded(lo, acc, project(xs.clone(), false)),
            project(xs, true),
        )
    }
}
fn math_body(node: Node, f: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        unit()
    } else if node.depth == 0 {
        value(W, f, xs)
    } else {
        let (lo, hi) = children(node);
        cmp(
            math(lo, f.clone(), project(xs.clone(), false)),
            math(hi, f, project(xs, true)),
        )
    }
}
fn carries_body(node: Node, acc: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        cube_lambda(node.depth, "__inactive_", literal(false))
    } else if node.depth == 0 {
        carry(W, acc, xs, literal(false))
    } else {
        let (lo, hi) = children(node);
        let state = folded(lo, acc.clone(), project(xs.clone(), false));
        lambda(
            "__carry_half",
            bool_ty(),
            choose_t(
                cube_type(node.depth - 1),
                v("__carry_half"),
                carries(hi, state, project(xs.clone(), true)),
                carries(lo, acc, project(xs, false)),
            ),
        )
    }
}
fn operands(node: Node) -> Vec<(String, Expr)> {
    vec![
        ("acc".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
    ]
}
fn invariant(node: Node, f: Expr, acc: Expr, xs: Expr, proof: Expr) -> Equality {
    instance(
        &symbol("Invariant", node),
        vec![f.clone(), acc.clone(), xs.clone(), proof],
        cmp(
            value(W, f.clone(), acc.clone()),
            math(node, f.clone(), xs.clone()),
        ),
        value(W, f, folded(node, acc, xs)),
    )
}
fn value_fold(node: Node, f: Expr, acc: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("ValueFold", node),
        vec![f.clone(), acc.clone(), xs.clone()],
        value(W, f.clone(), folded(node, acc.clone(), xs.clone())),
        value(W, f, fold_body(node, acc, xs)),
    )
}
fn math_unfold(node: Node, f: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("MathDefinition", node),
        vec![f.clone(), xs.clone()],
        math(node, f.clone(), xs.clone()),
        math_body(node, f, xs),
    )
}
fn algebra(p: &mut Producer) {
    let binders = vec![("f".into(), function()), ("x".into(), cube_type(W))];
    let right = value(W, v("f"), v("x"));
    p.theorem(
        "PrefixValueRightIdentity",
        &binders,
        Equality {
            left: cmp(right.clone(), unit()),
            right: right.clone(),
            proof: reflf(right),
        },
    );
    let root = vec![("f".into(), function())];
    p.theorem(
        "PrefixMaybeFalse",
        &root,
        Equality {
            left: conditional(v("f"), literal(false)),
            right: unit(),
            proof: reflf(unit()),
        },
    );

    let binders = vec![
        ("f".into(), function()),
        ("acc".into(), cube_type(W)),
        ("element".into(), cube_type(W)),
        (
            "h".into(),
            eq(
                carry(W, v("acc"), v("element"), literal(false)),
                literal(false),
            ),
        ),
    ];
    let f = v("f");
    let acc = v("acc");
    let element = v("element");
    let add = addition(W, f.clone(), acc.clone(), element.clone(), literal(false));
    let false_choice = instance(
        "PrefixMaybeFalse",
        vec![f.clone()],
        conditional(f.clone(), literal(false)),
        unit(),
    );
    let lhs = p.rewrite(add.left.clone(), false_choice);
    let right_id = instance(
        "PrefixValueRightIdentity",
        vec![f.clone(), element.clone()],
        cmp(value(W, f.clone(), element.clone()), unit()),
        value(W, f.clone(), element.clone()),
    );
    let lhs = p.change(lhs, right_id);
    let mut e = transit(reverse(lhs), add);
    let actual = carry(W, acc.clone(), element.clone(), literal(false));
    let q = typed(bool_ty(), actual, literal(false), v("h"));
    e = p.change(e, selector(weight(W, f.clone()), q));
    e = p.change(
        e,
        instance(
            "PrefixMaybeFalse",
            vec![weight(W, f.clone())],
            conditional(weight(W, f.clone()), literal(false)),
            unit(),
        ),
    );
    let result = sum(W, acc, element, literal(false));
    e = transit(
        e,
        instance(
            "PrefixValueRightIdentity",
            vec![f.clone(), result.clone()],
            cmp(value(W, f.clone(), result.clone()), unit()),
            value(W, f, result),
        ),
    );
    p.theorem("PrefixStep", &binders, e);
}
fn definitions(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let args = operands(node);
    let fargs = vec![
        ("f".into(), function()),
        ("xs".into(), cube_type(W + node.depth)),
    ];
    let acc = v("acc");
    let xs = v("xs");
    let f = v("f");
    let body = fold_body(node, acc.clone(), xs.clone());
    function_definition(p, &symbol("Fold", node), &args, cube_type(W), body.clone());
    function_definition(
        p,
        &symbol("Carries", node),
        &args,
        cube_type(node.depth),
        carries_body(node, acc.clone(), xs.clone()),
    );
    function_definition(
        p,
        &symbol("Math", node),
        &fargs,
        function(),
        math_body(node, f.clone(), xs.clone()),
    );
    let ty = apply(
        c(EQ),
        vec![
            cube_type(W),
            folded(node, acc.clone(), xs.clone()),
            body.clone(),
        ],
    );
    let proof = apply(c(REFL), vec![cube_type(W), body.clone()]);
    let (ty, proof) = fold_binders(&args, ty, proof);
    p.declare(&symbol("FoldDefinition", node), ty, proof, true);
    let folded_word = folded(node, acc.clone(), xs.clone());
    let word_eq = typed(
        cube_type(W),
        folded_word.clone(),
        body.clone(),
        apply(
            c(&name(&symbol("FoldDefinition", node))),
            vec![acc.clone(), xs.clone()],
        ),
    );
    let ctx = lambda(
        "__folded_word",
        cube_type(W),
        value(W, f.clone(), v("__folded_word")),
    );
    let lifted = mapped(
        word_eq,
        function(),
        ctx,
        value(W, f.clone(), folded_word),
        value(W, f.clone(), body),
    );
    let mut binders = vec![("f".into(), function())];
    binders.extend(args.clone());
    p.theorem(&symbol("ValueFold", node), &binders, as_fn(lifted));
    p.theorem(
        &symbol("MathDefinition", node),
        &fargs,
        computed(math_unfold(node, f, xs)),
    );
    if node.depth > 0 && node.count > 0 {
        let (lo, hi) = children(node);
        let state = folded(lo, acc.clone(), project(v("xs"), false));
        for high in [false, true] {
            let selected = if high {
                carries(hi, state.clone(), project(v("xs"), true))
            } else {
                carries(lo, acc.clone(), project(v("xs"), false))
            };
            let observed = project(carries(node, acc.clone(), v("xs")), high);
            let ty = apply(
                c(EQ),
                vec![cube_type(node.depth - 1), observed, selected.clone()],
            );
            let proof = apply(c(REFL), vec![cube_type(node.depth - 1), selected]);
            let (ty, proof) = fold_binders(&args, ty, proof);
            p.declare(
                &symbol(if high { "CarryHigh" } else { "CarryLow" }, node),
                ty,
                proof,
                true,
            );
        }
    }
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-definitions", node.depth, node.count),
    );
}
fn child_safety(node: Node, acc: Expr, xs: Expr, high: bool) -> Expr {
    let (lo, hi) = children(node);
    let child = if high { hi } else { lo };
    let state = if high {
        folded(lo, acc.clone(), project(xs.clone(), false))
    } else {
        acc.clone()
    };
    let selected = carries(child, state, project(xs.clone(), high));
    let observed = project(carries(node, acc.clone(), xs.clone()), high);
    let q = typed(
        cube_type(node.depth - 1),
        observed.clone(),
        selected.clone(),
        apply(
            c(&name(&symbol(
                if high { "CarryHigh" } else { "CarryLow" },
                node,
            ))),
            vec![acc, xs],
        ),
    );
    let indices = addresses(node.depth - 1, "__child_index_");
    let context = lambda(
        "__carry_view",
        cube_type(node.depth - 1),
        apply(v("__carry_view"), indices.clone()),
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
    let parent = typed(
        bool_ty(),
        q.left.clone(),
        literal(false),
        apply(v("safe"), parent_indices),
    );
    let e = next(backwards(q), parent);
    (0..node.depth - 1).rev().fold(e.proof, |body, i| {
        lambda(&format!("__child_index_{i}"), bool_ty(), body)
    })
}
fn prove_node(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let f = v("f");
    let acc = v("acc");
    let xs = v("xs");
    let mut binders = vec![("f".into(), function())];
    binders.extend(operands(node));
    binders.push(("safe".into(), safety(node, acc.clone(), xs.clone(), false)));
    let target = invariant(node, f.clone(), acc.clone(), xs.clone(), v("safe"));
    let mut e = p.rewrite(target.left, math_unfold(node, f.clone(), xs.clone()));
    if node.count == 0 {
        e = transit(
            e,
            instance(
                "PrefixValueRightIdentity",
                vec![f.clone(), acc.clone()],
                cmp(value(W, f.clone(), acc.clone()), unit()),
                value(W, f.clone(), acc.clone()),
            ),
        );
    } else if node.depth == 0 {
        let actual = carries(node, acc.clone(), xs.clone());
        let raw = carry(W, acc.clone(), xs.clone(), literal(false));
        let q = typed(bool_ty(), raw.clone(), actual.clone(), refl(raw.clone()));
        let h = typed(bool_ty(), actual, literal(false), v("safe"));
        let h = next(q, h);
        let proof = apply(
            c(&name("PrefixStep")),
            vec![f.clone(), acc.clone(), xs.clone(), h.proof],
        );
        e = transit(
            e,
            Equality {
                left: cmp(
                    value(W, f.clone(), acc.clone()),
                    value(W, f.clone(), xs.clone()),
                ),
                right: value(
                    W,
                    f.clone(),
                    sum(W, acc.clone(), xs.clone(), literal(false)),
                ),
                proof,
            },
        );
    } else {
        let (lo, hi) = children(node);
        let xl = project(xs.clone(), false);
        let xh = project(xs.clone(), true);
        let low_math = math(lo, f.clone(), xl.clone());
        let high_math = math(hi, f.clone(), xh.clone());
        let grouped = cmp(cmp(value(W, f.clone(), acc.clone()), low_math), high_math);
        let q = p.monoid(e.right.clone(), grouped);
        e = transit(e, q);
        let low = invariant(
            lo,
            f.clone(),
            acc.clone(),
            xl.clone(),
            child_safety(node, acc.clone(), xs.clone(), false),
        );
        e = p.change(e, low);
        let mid = folded(lo, acc.clone(), xl);
        e = transit(
            e,
            invariant(
                hi,
                f.clone(),
                mid,
                xh,
                child_safety(node, acc.clone(), xs.clone(), true),
            ),
        );
    }
    e = transit(e, reverse(value_fold(node, f, acc, xs)));
    assert_eq!(e.right, target.right);
    p.theorem(&symbol("Invariant", node), &binders, e);
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-invariant", node.depth, node.count),
    );
}
fn claim(p: &mut Producer, node: Node, mutation: usize) -> Expr {
    let f = c("Std.Nat.succ");
    let acc = v("acc");
    let xs = v("xs");
    let e = invariant(node, f.clone(), acc.clone(), xs.clone(), v("safe"));
    let left = if mutation == 1 {
        math(node, f.clone(), xs.clone())
    } else if mutation == 3 {
        cmp(
            value(W, f.clone(), acc.clone()),
            math(
                Node {
                    depth: node.depth,
                    count: node.count - 1,
                },
                f.clone(),
                xs.clone(),
            ),
        )
    } else {
        e.left.clone()
    };
    let goal = apply(
        c(EQ),
        vec![
            nat(),
            apply(left, vec![v("initial")]),
            apply(e.right.clone(), vec![v("initial")]),
        ],
    );
    let ctx = lambda(
        "__prefix_observer",
        function(),
        apply(v("__prefix_observer"), vec![v("initial")]),
    );
    let proof = apply(
        c("Std.Eq.congr"),
        vec![function(), nat(), ctx, e.left, e.right, e.proof],
    );
    let binders = vec![
        ("initial".into(), nat()),
        ("acc".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
        ("safe".into(), safety(node, acc, xs, mutation == 2)),
    ];
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("PrefixClaim", ty.clone(), proof, true);
    ty
}

pub(super) fn generate(root: &Path, output: &Path, count: usize, trace: bool) {
    assert!(
        count <= 129,
        "private boundary measurement through 129 elements"
    );
    fs::create_dir_all(output).unwrap();
    let depth = if count <= 1 {
        0
    } else {
        (usize::BITS - (count - 1).leading_zeros()) as usize
    };
    let node = Node { depth, count };
    let mut nodes = BTreeSet::new();
    collect(node, &mut nodes);
    if count > 0 {
        collect(
            Node {
                depth,
                count: count - 1,
            },
            &mut nodes,
        );
    }
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.PrefixCount{count}");
    p.primitive_library();
    for d in 0..=W {
        super::super::definitions(&mut p, d, None);
        commutation(&mut p, d, None);
        adding(&mut p, d);
    }
    algebra(&mut p);
    for n in nodes {
        definitions(&mut p, n, trace.then_some(output));
        prove_node(&mut p, n, trace.then_some(output));
    }
    let base = p.builder.c;
    let mut cases = vec![];
    for mutation in 0..if count == 0 { 3 } else { 4 } {
        let id = format!(
            "prefix-{count}-{}",
            match mutation {
                0 => "exact-nat-sum",
                1 => "dropped-initial",
                2 =>
                    if count == 0 {
                        "vacuous-empty-safety"
                    } else {
                        "unrelated-safety"
                    },
                _ => "skipped-last",
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
        cases.push(json!({"id":id,"count":count,"address_depth":depth,"storage_capacity":1usize<<depth,
            "source_word_bits":64,"quantified_storage_bits":64usize<<depth,"mutation":mutation,
            "expected":if mutation==0 || count==0 && mutation==2 {"accepted"} else {"rejected"},
            "declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name("PrefixClaim"),
            "premise":if mutation==2 {"all inactive false equalities (vacuous only for empty prefix)"} else {"each actual intermediate unsigned carry equals false"},
            "scope":"conditional exact fixed-count modular64 prefix fold to Nat sum; signed/nonnegative and Go guard bindings remain pending",
            "goal_debug":format!("{ty:?}")}));
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.prefix_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
