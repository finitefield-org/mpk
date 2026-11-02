//! A real left-to-right modular64 accumulator related to an exact Nat sum.
//! The proof parameter refers to every actual step carry, not callee truth.
//! Partial final trees retain all unused backing values as quantified inputs.

use super::*;
use std::collections::BTreeSet;
mod continuation;
mod signed_prefix;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Unsigned,
    Signed,
    Continuation,
    Returned,
}
pub(super) fn generate_returned(root: &Path, output: &Path, count: usize, trace: bool) {
    generate_mode(root, output, count, trace, Mode::Returned);
}
pub(super) fn generate_continuation(root: &Path, output: &Path, count: usize, trace: bool) {
    generate_mode(root, output, count, trace, Mode::Continuation);
}

pub(super) fn generate_signed(root: &Path, output: &Path, count: usize, trace: bool) {
    generate_mode(root, output, count, trace, Mode::Signed);
}

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
fn overflow(node: Node, f: Expr, acc: Expr, xs: Expr) -> Expr {
    call("Overflow", node, vec![f, acc, xs])
}
fn overflow_body(node: Node, f: Expr, acc: Expr, xs: Expr) -> Expr {
    if node.count == 0 {
        unit()
    } else if node.depth == 0 {
        conditional(weight(W, f), carry(W, acc, xs, literal(false)))
    } else {
        let (lo, hi) = children(node);
        let mid = folded(lo, acc.clone(), project(xs.clone(), false));
        cmp(
            overflow(lo, f.clone(), acc, project(xs.clone(), false)),
            overflow(hi, f, mid, project(xs, true)),
        )
    }
}
fn overflow_unfold(node: Node, f: Expr, acc: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("OverflowDefinition", node),
        vec![f.clone(), acc.clone(), xs.clone()],
        overflow(node, f.clone(), acc.clone(), xs.clone()),
        overflow_body(node, f, acc, xs),
    )
}
fn math_commutes(node: Node, f: Expr, g: Expr, h: Equality, xs: Expr) -> Equality {
    let m = math(node, f.clone(), xs.clone());
    instance(
        &symbol("MathCommutes", node),
        vec![f, g.clone(), h.proof, xs],
        cmp(m.clone(), g.clone()),
        cmp(g, m),
    )
}
fn overflow_commutes(node: Node, f: Expr, g: Expr, h: Equality, acc: Expr, xs: Expr) -> Equality {
    let o = overflow(node, f.clone(), acc.clone(), xs.clone());
    instance(
        &symbol("OverflowCommutes", node),
        vec![f, g.clone(), h.proof, acc, xs],
        cmp(o.clone(), g.clone()),
        cmp(g, o),
    )
}
fn raw_invariant(node: Node, f: Expr, acc: Expr, xs: Expr) -> Equality {
    instance(
        &symbol("RawInvariant", node),
        vec![f.clone(), acc.clone(), xs.clone()],
        cmp(
            value(W, f.clone(), acc.clone()),
            math(node, f.clone(), xs.clone()),
        ),
        cmp(
            value(W, f.clone(), folded(node, acc.clone(), xs.clone())),
            overflow(node, f, acc, xs),
        ),
    )
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
fn algebra(p: &mut Producer, trace: Option<&Path>) {
    let g = v("g");
    let right = cmp(g.clone(), unit());
    p.theorem(
        "PrefixIdentityCommutes",
        &[("g".into(), function())],
        Equality {
            left: cmp(unit(), g),
            right: right.clone(),
            proof: reflf(right),
        },
    );
    let fg = vec![("f".into(), function()), ("g".into(), function())];
    let pair = cmp(v("f"), v("g"));
    p.theorem(
        "PrefixCompositeIdentity",
        &fg,
        Equality {
            left: cmp(pair.clone(), unit()),
            right: pair.clone(),
            proof: reflf(pair),
        },
    );
    let binders = vec![("f".into(), function()), ("x".into(), cube_type(W))];
    let unfolded = word_unfold(W, v("f"), v("x"));
    let mut e = Producer::on_left(unfolded.clone(), unit());
    let (low, high) = Producer::comp_parts(&unfolded.right).unwrap();
    e = transit(
        e,
        instance(
            "PrefixCompositeIdentity",
            vec![low, high],
            cmp(unfolded.right.clone(), unit()),
            unfolded.right.clone(),
        ),
    );
    e = transit(e, reverse(unfolded));
    p.theorem("PrefixValueRightIdentity", &binders, e);
    save_trace(p, trace, "prefix-algebra-identity");
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
    save_trace(p, trace, "prefix-algebra-maybe-false");

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
    let raw = transit(reverse(lhs), add);
    let bare = binders[..3].to_vec();
    p.theorem("PrefixStepAll", &bare, raw.clone());
    let mut e = raw;
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
    let mut f_operands = vec![("f".into(), function())];
    f_operands.extend(args.clone());
    function_definition(
        p,
        &symbol("Overflow", node),
        &f_operands,
        function(),
        overflow_body(node, f.clone(), acc.clone(), xs.clone()),
    );
    p.theorem(
        &symbol("OverflowDefinition", node),
        &f_operands,
        computed(overflow_unfold(node, f.clone(), acc.clone(), xs.clone())),
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
fn commutative_sums(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let f = v("f");
    let g = v("g");
    let acc = v("acc");
    let xs = v("xs");
    let h = Equality {
        left: cmp(f.clone(), g.clone()),
        right: cmp(g.clone(), f.clone()),
        proof: v("h"),
    };
    let basic = vec![
        ("f".into(), function()),
        ("g".into(), function()),
        ("h".into(), commute(f.clone(), g.clone())),
    ];
    let mut mb = basic.clone();
    mb.push(("xs".into(), cube_type(W + node.depth)));
    let mut e = p.rewrite(
        cmp(math(node, f.clone(), xs.clone()), g.clone()),
        math_unfold(node, f.clone(), xs.clone()),
    );
    let step = if node.count == 0 {
        instance(
            "PrefixIdentityCommutes",
            vec![g.clone()],
            cmp(unit(), g.clone()),
            cmp(g.clone(), unit()),
        )
    } else if node.depth == 0 {
        value_commutes(W, f.clone(), g.clone(), h.clone(), xs.clone())
    } else {
        let (lo, hi) = children(node);
        let ml = math(lo, f.clone(), project(xs.clone(), false));
        let mh = math(hi, f.clone(), project(xs.clone(), true));
        let hl = math_commutes(
            lo,
            f.clone(),
            g.clone(),
            h.clone(),
            project(xs.clone(), false),
        );
        let hh = math_commutes(
            hi,
            f.clone(),
            g.clone(),
            h.clone(),
            project(xs.clone(), true),
        );
        instance(
            "ProductCommutes",
            vec![ml.clone(), mh.clone(), g.clone(), hl.proof, hh.proof],
            cmp(cmp(ml.clone(), mh.clone()), g.clone()),
            cmp(g.clone(), cmp(ml, mh)),
        )
    };
    e = transit(e, step);
    e = p.change(e, reverse(math_unfold(node, f.clone(), xs.clone())));
    p.theorem(&symbol("MathCommutes", node), &mb, e);
    let mut ob = basic;
    ob.extend(operands(node));
    let mut e = p.rewrite(
        cmp(
            overflow(node, f.clone(), acc.clone(), xs.clone()),
            g.clone(),
        ),
        overflow_unfold(node, f.clone(), acc.clone(), xs.clone()),
    );
    let step = if node.count == 0 {
        instance(
            "PrefixIdentityCommutes",
            vec![g.clone()],
            cmp(unit(), g.clone()),
            cmp(g.clone(), unit()),
        )
    } else if node.depth == 0 {
        let hw = weight_commutes(W, f.clone(), g.clone(), h.clone());
        Producer::maybe_left(
            weight(W, f.clone()),
            g.clone(),
            carry(W, acc.clone(), xs.clone(), literal(false)),
            hw,
        )
    } else {
        let (lo, hi) = children(node);
        let xl = project(xs.clone(), false);
        let xh = project(xs.clone(), true);
        let mid = folded(lo, acc.clone(), xl.clone());
        let ol = overflow(lo, f.clone(), acc.clone(), xl.clone());
        let oh = overflow(hi, f.clone(), mid.clone(), xh.clone());
        let hl = overflow_commutes(lo, f.clone(), g.clone(), h.clone(), acc.clone(), xl);
        let hh = overflow_commutes(hi, f.clone(), g.clone(), h, mid, xh);
        instance(
            "ProductCommutes",
            vec![ol.clone(), oh.clone(), g.clone(), hl.proof, hh.proof],
            cmp(cmp(ol.clone(), oh.clone()), g.clone()),
            cmp(g, cmp(ol, oh)),
        )
    };
    e = transit(e, step);
    e = p.change(e, reverse(overflow_unfold(node, f, acc, xs)));
    p.theorem(&symbol("OverflowCommutes", node), &ob, e);
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-commutation", node.depth, node.count),
    );
}
fn prove_raw(p: &mut Producer, node: Node, trace: Option<&Path>) {
    let f = v("f");
    let acc = v("acc");
    let xs = v("xs");
    let target = raw_invariant(node, f.clone(), acc.clone(), xs.clone());
    let mut binders = vec![("f".into(), function())];
    binders.extend(operands(node));
    let mut e = p.rewrite(target.left, math_unfold(node, f.clone(), xs.clone()));
    if node.count == 0 {
        e = p.change(
            e,
            reverse(value_fold(node, f.clone(), acc.clone(), xs.clone())),
        );
        e = p.change(
            e,
            reverse(overflow_unfold(node, f.clone(), acc.clone(), xs.clone())),
        );
    } else if node.depth == 0 {
        let proof = apply(
            c(&name("PrefixStepAll")),
            vec![f.clone(), acc.clone(), xs.clone()],
        );
        let right = cmp(
            value(
                W,
                f.clone(),
                sum(W, acc.clone(), xs.clone(), literal(false)),
            ),
            conditional(
                weight(W, f.clone()),
                carry(W, acc.clone(), xs.clone(), literal(false)),
            ),
        );
        e = transit(
            e,
            Equality {
                left: e_right_step(f.clone(), acc.clone(), xs.clone()),
                right,
                proof,
            },
        );
        e = p.change(
            e,
            reverse(value_fold(node, f.clone(), acc.clone(), xs.clone())),
        );
        e = p.change(
            e,
            reverse(overflow_unfold(node, f.clone(), acc.clone(), xs.clone())),
        );
    } else {
        let (lo, hi) = children(node);
        let xl = project(xs.clone(), false);
        let xh = project(xs.clone(), true);
        let mid = folded(lo, acc.clone(), xl.clone());
        let ml = math(lo, f.clone(), xl.clone());
        let mh = math(hi, f.clone(), xh.clone());
        let ol = overflow(lo, f.clone(), acc.clone(), xl.clone());
        let oh = overflow(hi, f.clone(), mid.clone(), xh.clone());
        let norm = p.monoid(
            e.right.clone(),
            cmp(cmp(value(W, f.clone(), acc.clone()), ml), mh.clone()),
        );
        e = transit(e, norm);
        e = p.change(e, raw_invariant(lo, f.clone(), acc.clone(), xl.clone()));
        let norm = p.monoid(
            e.right.clone(),
            cmp(
                value(W, f.clone(), mid.clone()),
                cmp(ol.clone(), mh.clone()),
            ),
        );
        e = transit(e, norm);
        let hroot = reflexive(cmp(f.clone(), f.clone()));
        let hol = overflow_commutes(lo, f.clone(), f.clone(), hroot, acc.clone(), xl);
        let swap = reverse(math_commutes(
            hi,
            f.clone(),
            ol.clone(),
            reverse(hol.clone()),
            xh.clone(),
        ));
        e = p.change(e, swap);
        let norm = p.monoid(
            e.right.clone(),
            cmp(cmp(value(W, f.clone(), mid.clone()), mh), ol.clone()),
        );
        e = transit(e, norm);
        e = p.change(e, raw_invariant(hi, f.clone(), mid.clone(), xh.clone()));
        let final_state = folded(hi, mid.clone(), xh.clone());
        let norm = p.monoid(
            e.right.clone(),
            cmp(value(W, f.clone(), final_state), cmp(oh, ol.clone())),
        );
        e = transit(e, norm);
        let swap = overflow_commutes(hi, f.clone(), ol, reverse(hol), mid, xh);
        e = p.change(e, swap);
        e = p.change(
            e,
            reverse(overflow_unfold(node, f.clone(), acc.clone(), xs.clone())),
        );
        e = p.change(
            e,
            reverse(value_fold(node, f.clone(), acc.clone(), xs.clone())),
        );
    }
    assert_eq!(e.right, target.right);
    p.theorem(&symbol("RawInvariant", node), &binders, e);
    save_trace(
        p,
        trace,
        &format!("fold-{}-{}-raw-invariant", node.depth, node.count),
    );
}
fn e_right_step(f: Expr, acc: Expr, xs: Expr) -> Expr {
    cmp(value(W, f.clone(), acc), value(W, f, xs))
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

fn raw_claim(p: &mut Producer, node: Node, mutation: usize) -> Expr {
    let f = c("Std.Nat.succ");
    let acc = v("acc");
    let xs = v("xs");
    let mut e = raw_invariant(node, f.clone(), acc.clone(), xs.clone());
    // The empty correction is genuinely the identity. Supply its checked
    // reduction explicitly; do not rely on converting the whole word value.
    if node.count == 0 && mutation == 1 {
        e = p.change(e, overflow_unfold(node, f.clone(), acc.clone(), xs.clone()));
        e = transit(
            e,
            instance(
                "PrefixValueRightIdentity",
                vec![f.clone(), folded(node, acc.clone(), xs.clone())],
                cmp(
                    value(W, f.clone(), folded(node, acc.clone(), xs.clone())),
                    unit(),
                ),
                value(W, f.clone(), folded(node, acc.clone(), xs.clone())),
            ),
        );
    }
    let left = if mutation == 2 {
        math(node, f.clone(), xs.clone())
    } else {
        e.left.clone()
    };
    let right = if mutation == 1 {
        value(W, f, folded(node, acc, xs))
    } else {
        e.right.clone()
    };
    let goal = apply(
        c(EQ),
        vec![
            nat(),
            apply(left, vec![v("initial")]),
            apply(right, vec![v("initial")]),
        ],
    );
    let context = lambda(
        "__raw_observer",
        function(),
        apply(v("__raw_observer"), vec![v("initial")]),
    );
    let proof = apply(
        c("Std.Eq.congr"),
        vec![function(), nat(), context, e.left, e.right, e.proof],
    );
    let binders = vec![
        ("initial".into(), nat()),
        ("acc".into(), cube_type(W)),
        ("xs".into(), cube_type(W + node.depth)),
    ];
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("PrefixRawClaim", ty.clone(), proof, true);
    ty
}

pub(super) fn generate(root: &Path, output: &Path, count: usize, trace: bool) {
    generate_mode(root, output, count, trace, Mode::Unsigned);
}
fn generate_mode(root: &Path, output: &Path, count: usize, trace: bool, mode: Mode) {
    let signed = mode != Mode::Unsigned;
    let returned = mode == Mode::Returned;
    let bounded = mode == Mode::Continuation || returned;
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
    p.builder.c.module = format!(
        "{PREFIX}.{}PrefixCount{count}",
        if returned {
            "Return"
        } else if bounded {
            "Continuation"
        } else if signed {
            "Signed"
        } else {
            ""
        }
    );
    p.primitive_library();
    if signed {
        super::primitives(&mut p);
    }
    if bounded {
        order::install_primitives(&mut p);
        continuation::primitives(&mut p);
    }
    for d in 0..=W {
        super::super::definitions(&mut p, d, None);
        commutation(&mut p, d, None);
        adding(&mut p, d);
        if signed {
            super::build(&mut p, d, None);
        }
        if bounded {
            order::install_word(&mut p, d, None);
            continuation::zero_library(&mut p, d, trace.then_some(output));
        }
    }
    algebra(&mut p, trace.then_some(output));
    if returned {
        continuation::returned_primitives(&mut p, trace.then_some(output));
    }
    for n in nodes {
        definitions(&mut p, n, trace.then_some(output));
        commutative_sums(&mut p, n, trace.then_some(output));
        prove_raw(&mut p, n, trace.then_some(output));
        prove_node(&mut p, n, trace.then_some(output));
        if signed && !bounded {
            signed_prefix::build(&mut p, n, trace.then_some(output));
        }
        if bounded {
            continuation::build(&mut p, n, trace.then_some(output));
        }
        if returned {
            continuation::returned_build(&mut p, n, trace.then_some(output));
        }
    }
    let base = p.builder.c;
    if returned {
        continuation::returned_emit(base, output, node);
        return;
    }
    if bounded {
        continuation::emit(base, output, node);
        return;
    }
    if signed {
        signed_prefix::emit(base, output, node);
        return;
    }
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
        cases.push(json!({"id":id,"family":"conditional","count":count,"address_depth":depth,"storage_capacity":1usize<<depth,
            "source_word_bits":64,"quantified_storage_bits":64usize<<depth,"mutation":mutation,
            "quantified_accumulator_bits":64,"initial_natural_quantified":true,
            "expected":if mutation==0 || count==0 && mutation==2 {"accepted"} else {"rejected"},
            "declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name("PrefixClaim"),
            "premise":if mutation==2 {"all inactive false equalities (vacuous only for empty prefix)"} else {"each actual intermediate unsigned carry equals false"},
            "scope":"conditional exact fixed-count modular64 prefix fold to Nat sum; signed/nonnegative and Go guard bindings remain pending",
            "goal_debug":format!("{ty:?}")}));
    }
    for mutation in 0..3 {
        let id = format!(
            "prefix-{count}-{}",
            match mutation {
                0 => "exact-nat-sum-with-overflow",
                1 =>
                    if count == 0 {
                        "empty-overflow-identity"
                    } else {
                        "dropped-overflow"
                    },
                _ => "raw-dropped-initial",
            }
        );
        let mut p = Producer {
            builder: Builder::from(base.clone()),
            fresh: 0,
        };
        let ty = raw_claim(&mut p, node, mutation);
        let terms = p.builder.c.term_table.len();
        let declarations = p.builder.c.declarations.len();
        let bytes = p.builder.finish();
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        cases.push(json!({"id":id,"family":"unconditional","count":count,"address_depth":depth,"storage_capacity":1usize<<depth,
            "source_word_bits":64,"quantified_storage_bits":64usize<<depth,"mutation":mutation,
            "quantified_accumulator_bits":64,"initial_natural_quantified":true,
            "expected":if mutation==0 || count==0 && mutation==1 {"accepted"} else {"rejected"},
            "declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name("PrefixRawClaim"),
            "premise":"none",
            "scope":"unconditional exact fixed-count modular64 prefix fold plus actual step-carry correction to Nat sum; signed/nonnegative and Go bindings remain pending",
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
