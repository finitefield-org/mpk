//! Two's-complement exact-addition relation under the actual word carry guard.
//! Integers are interpreted as balanced natural differences. The claim checks
//! equality of cross sums, rather than assuming unsigned and signed coincide.

use super::*;
mod fold;
mod order;
pub(super) fn generate_returned(root: &Path, output: &Path, count: usize, trace: bool) {
    fold::generate_returned(root, output, count, trace);
}
pub(super) fn generate_continuation(root: &Path, output: &Path, count: usize, trace: bool) {
    fold::generate_continuation(root, output, count, trace);
}
pub(super) fn generate_order(root: &Path, output: &Path, depth: usize, trace: bool) {
    order::generate(root, output, depth, trace);
}
pub(super) fn generate_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    fold::generate(root, output, count, trace);
}
pub(super) fn generate_signed_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    fold::generate_signed(root, output, count, trace);
}

#[derive(Clone)]
struct Typed {
    ty: Expr,
    left: Expr,
    right: Expr,
    proof: Expr,
}
fn typed(ty: Expr, left: Expr, right: Expr, proof: Expr) -> Typed {
    Typed {
        ty,
        left,
        right,
        proof,
    }
}
fn backwards(e: Typed) -> Typed {
    typed(
        e.ty.clone(),
        e.right.clone(),
        e.left.clone(),
        apply(c("Std.Eq.symm"), vec![e.ty, e.left, e.right, e.proof]),
    )
}
fn next(a: Typed, b: Typed) -> Typed {
    assert_eq!(a.ty, b.ty);
    assert_eq!(a.right, b.left);
    typed(
        a.ty.clone(),
        a.left.clone(),
        b.right.clone(),
        apply(
            c("Std.Eq.trans"),
            vec![a.ty, a.left, a.right, b.right, a.proof, b.proof],
        ),
    )
}
fn mapped(e: Typed, ty: Expr, context: Expr, left: Expr, right: Expr) -> Typed {
    typed(
        ty.clone(),
        left,
        right,
        apply(
            c("Std.Eq.congr"),
            vec![e.ty, ty, context, e.left, e.right, e.proof],
        ),
    )
}
fn replace(p: &mut Producer, ty: Expr, whole: Expr, e: Typed) -> Typed {
    if e.left == e.right {
        return typed(
            ty.clone(),
            whole.clone(),
            whole.clone(),
            apply(c(REFL), vec![ty, whole]),
        );
    }
    let result = substitute(&whole, &e.left, &e.right);
    assert_ne!(result, whole, "signed rewrite absent");
    let n = format!("__signed_hole_{}", p.fresh);
    p.fresh += 1;
    let context = lambda(&n, e.ty.clone(), substitute(&whole, &e.left, &v(&n)));
    mapped(e, ty, context, whole, result)
}
fn change(p: &mut Producer, a: Typed, e: Typed) -> Typed {
    let tail = replace(p, a.ty.clone(), a.right.clone(), e);
    next(a, tail)
}
fn as_fn(e: Typed) -> Equality {
    assert_eq!(e.ty, function());
    Equality {
        left: e.left,
        right: e.right,
        proof: e.proof,
    }
}
fn from_fn(e: Equality) -> Typed {
    typed(function(), e.left, e.right, e.proof)
}
fn theorem(p: &mut Producer, suffix: &str, binders: &[(String, Expr)], e: Typed) {
    let (ty, proof) = fold_binders(binders, apply(c(EQ), vec![e.ty, e.left, e.right]), e.proof);
    p.declare(suffix, ty, proof, true);
}
fn sign(d: usize, x: Expr) -> Expr {
    tree("Sign", d, vec![x])
}
fn into(d: usize, x: Expr, y: Expr, c: Expr) -> Expr {
    tree("SignIncoming", d, vec![x, y, c])
}
fn local(x: Expr, y: Expr, c: Expr) -> (Expr, Expr) {
    let (s, k) = adder(&[x], &[y], c);
    (s[0].clone(), k)
}
fn guard(d: usize, x: Expr, y: Expr, c: Expr) -> Expr {
    bool_eq(into(d, x.clone(), y.clone(), c.clone()), carry(d, x, y, c))
}
fn sign_body(d: usize, x: Expr) -> Expr {
    if d == 0 {
        x
    } else {
        sign(d - 1, project(x, true))
    }
}
fn into_body(d: usize, x: Expr, y: Expr, c: Expr) -> Expr {
    if d == 0 {
        c
    } else {
        into(
            d - 1,
            project(x.clone(), true),
            project(y.clone(), true),
            carry(d - 1, project(x, false), project(y, false), c),
        )
    }
}
fn sign_definition(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        sign(d, x.clone()),
        sign_body(d, x.clone()),
        apply(c(&name(&format!("TreeSignDefinition{d}"))), vec![x]),
    )
}
fn into_definition(d: usize, x: Expr, y: Expr, incoming: Expr) -> Typed {
    typed(
        bool_ty(),
        into(d, x.clone(), y.clone(), incoming.clone()),
        into_body(d, x.clone(), y.clone(), incoming.clone()),
        apply(
            c(&name(&format!("TreeSignIncomingDefinition{d}"))),
            vec![x, y, incoming],
        ),
    )
}
fn sum_binding(d: usize, x: Expr, y: Expr, incoming: Expr) -> Typed {
    let (s, _) = local(
        sign(d, x.clone()),
        sign(d, y.clone()),
        into(d, x.clone(), y.clone(), incoming.clone()),
    );
    typed(
        bool_ty(),
        sign(d, sum(d, x.clone(), y.clone(), incoming.clone())),
        s,
        apply(
            c(&name(&format!("TreeSumSignBinding{d}"))),
            vec![x, y, incoming],
        ),
    )
}
fn carry_binding(d: usize, x: Expr, y: Expr, incoming: Expr) -> Typed {
    let (_, k) = local(
        sign(d, x.clone()),
        sign(d, y.clone()),
        into(d, x.clone(), y.clone(), incoming.clone()),
    );
    typed(
        bool_ty(),
        carry(d, x.clone(), y.clone(), incoming.clone()),
        k,
        apply(
            c(&name(&format!("TreeCarrySignBinding{d}"))),
            vec![x, y, incoming],
        ),
    )
}
fn selector(root: Expr, e: Typed) -> Equality {
    let context = lambda(
        "__sign_selector",
        bool_ty(),
        conditional(root.clone(), v("__sign_selector")),
    );
    let left = conditional(root.clone(), e.left.clone());
    let right = conditional(root, e.right.clone());
    as_fn(mapped(e, function(), context, left, right))
}

pub(super) fn primitives(p: &mut Producer) {
    let f = v("f");
    let x = v("x");
    let y = v("y");
    let incoming = v("incoming");
    let (s, k) = local(x.clone(), y.clone(), incoming.clone());
    let no_overflow = bool_eq(incoming.clone(), k.clone());
    let left = cmp(conditional(f.clone(), k), conditional(f.clone(), s));
    let right = cmp(conditional(f, x), conditional(v("f"), y));
    let conditional_left = choose_t(function(), no_overflow.clone(), left.clone(), right.clone());
    let binders = vec![
        ("f".into(), function()),
        ("x".into(), bool_ty()),
        ("y".into(), bool_ty()),
        ("incoming".into(), bool_ty()),
    ];
    let proof = p.prove_local(
        &eqf(conditional_left.clone(), right.clone()),
        &[v("x"), v("y"), incoming],
    );
    p.theorem(
        "SignedLocalGuard",
        &binders,
        Equality {
            left: conditional_left.clone(),
            right: right.clone(),
            proof,
        },
    );
    let mut guarded = binders;
    guarded.push(("h".into(), eq(no_overflow.clone(), literal(true))));
    let context = lambda(
        "__signed_guard",
        bool_ty(),
        choose_t(function(), v("__signed_guard"), left.clone(), right.clone()),
    );
    let at_true = choose_t(function(), literal(true), left.clone(), right.clone());
    let selection = Equality {
        left: conditional_left.clone(),
        right: at_true.clone(),
        proof: apply(
            c("Std.Eq.congr"),
            vec![
                bool_ty(),
                function(),
                context,
                no_overflow,
                literal(true),
                v("h"),
            ],
        ),
    };
    let start = Equality {
        left: left.clone(),
        right: at_true,
        proof: reflf(left),
    };
    let end = instance(
        "SignedLocalGuard",
        vec![v("f"), v("x"), v("y"), v("incoming")],
        conditional_left,
        right,
    );
    let result = transit(transit(start, reverse(selection)), end);
    p.theorem("SignedLocalBalance", &guarded, result);
}

pub(super) fn build(p: &mut Producer, d: usize, trace: Option<&Path>) {
    let operands = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("incoming".into(), bool_ty()),
    ];
    let unary = vec![("x".into(), cube_type(d))];
    function_definition(
        p,
        &format!("TreeSign{d}"),
        &unary,
        bool_ty(),
        sign_body(d, v("x")),
    );
    function_definition(
        p,
        &format!("TreeSignIncoming{d}"),
        &operands,
        bool_ty(),
        into_body(d, v("x"), v("y"), v("incoming")),
    );
    let mut e = sign_definition(d, v("x"));
    e.proof = refl(e.right.clone());
    theorem(p, &format!("TreeSignDefinition{d}"), &unary, e);
    let mut e = into_definition(d, v("x"), v("y"), v("incoming"));
    e.proof = refl(e.right.clone());
    theorem(p, &format!("TreeSignIncomingDefinition{d}"), &operands, e);
    let x = v("x");
    let y = v("y");
    let incoming = v("incoming");
    let mut sums = if d == 0 {
        let target = sum_binding(d, x.clone(), y.clone(), incoming.clone());
        typed(
            bool_ty(),
            target.left,
            target.right.clone(),
            refl(target.right),
        )
    } else {
        let mut e = sign_definition(d, sum(d, x.clone(), y.clone(), incoming.clone()));
        let (sl, sh, _) = half_results(d, x.clone(), y.clone(), incoming.clone());
        let _ = sl;
        let projected = project(sum(d, x.clone(), y.clone(), incoming.clone()), true);
        let q = typed(
            cube_type(d - 1),
            projected.clone(),
            sh.clone(),
            apply(
                c(&name(&format!("TreeSumProjection{d}High"))),
                vec![x.clone(), y.clone(), incoming.clone()],
            ),
        );
        let ctx = lambda(
            "__signed_high_sum",
            cube_type(d - 1),
            sign(d - 1, v("__signed_high_sum")),
        );
        e = next(
            e,
            mapped(q, bool_ty(), ctx, sign(d - 1, projected), sign(d - 1, sh)),
        );
        let low = carry(
            d - 1,
            project(x.clone(), false),
            project(y.clone(), false),
            incoming.clone(),
        );
        next(
            e,
            sum_binding(
                d - 1,
                project(x.clone(), true),
                project(y.clone(), true),
                low,
            ),
        )
    };
    if d > 0 {
        sums = change(
            p,
            sums,
            backwards(into_definition(d, x.clone(), y.clone(), incoming.clone())),
        );
        sums = change(p, sums, backwards(sign_definition(d, x.clone())));
        sums = change(p, sums, backwards(sign_definition(d, y.clone())));
    }
    theorem(p, &format!("TreeSumSignBinding{d}"), &operands, sums);
    save_trace(p, trace, &format!("signed-{d}-sum-binding"));
    let mut carries = typed(
        bool_ty(),
        carry(d, x.clone(), y.clone(), incoming.clone()),
        if d == 0 {
            local(x.clone(), y.clone(), incoming.clone()).1
        } else {
            half_results(d, x.clone(), y.clone(), incoming.clone()).2
        },
        apply(
            c(&name(&format!("TreeCarryDefinition{d}"))),
            vec![x.clone(), y.clone(), incoming.clone()],
        ),
    );
    if d > 0 {
        let low = carry(
            d - 1,
            project(x.clone(), false),
            project(y.clone(), false),
            incoming.clone(),
        );
        carries = next(
            carries,
            carry_binding(
                d - 1,
                project(x.clone(), true),
                project(y.clone(), true),
                low,
            ),
        );
    }
    carries = change(
        p,
        carries,
        backwards(into_definition(d, x.clone(), y.clone(), incoming.clone())),
    );
    carries = change(p, carries, backwards(sign_definition(d, x.clone())));
    carries = change(p, carries, backwards(sign_definition(d, y.clone())));
    theorem(p, &format!("TreeCarrySignBinding{d}"), &operands, carries);
    save_trace(p, trace, &format!("signed-{d}-carry-binding"));

    let mut binders = vec![("f".into(), function())];
    binders.extend(operands.clone());
    let predicate = guard(d, x.clone(), y.clone(), incoming.clone());
    binders.push(("h".into(), eq(predicate.clone(), literal(true))));
    let carry_eq = carry_binding(d, x.clone(), y.clone(), incoming.clone());
    let predicate_eq = replace(p, bool_ty(), predicate.clone(), carry_eq.clone());
    let h = typed(bool_ty(), predicate, literal(true), v("h"));
    let local_h = next(backwards(predicate_eq), h);
    let root = weight(d, v("f"));
    let balance_left = cmp(
        conditional(
            root.clone(),
            carry(d, x.clone(), y.clone(), incoming.clone()),
        ),
        conditional(
            root.clone(),
            sign(d, sum(d, x.clone(), y.clone(), incoming.clone())),
        ),
    );
    let mut e = from_fn(reflexive(balance_left));
    let q = selector(root.clone(), carry_eq);
    e = change(p, e, from_fn(q));
    let q = selector(
        root.clone(),
        sum_binding(d, x.clone(), y.clone(), incoming.clone()),
    );
    e = change(p, e, from_fn(q));
    let (_, local_carry) = local(
        sign(d, x.clone()),
        sign(d, y.clone()),
        into(d, x.clone(), y.clone(), incoming.clone()),
    );
    let (local_sum, _) = local(
        sign(d, x.clone()),
        sign(d, y.clone()),
        into(d, x.clone(), y.clone(), incoming.clone()),
    );
    let end = instance(
        "SignedLocalBalance",
        vec![
            root.clone(),
            sign(d, x.clone()),
            sign(d, y.clone()),
            into(d, x.clone(), y.clone(), incoming.clone()),
            local_h.proof,
        ],
        cmp(
            conditional(root.clone(), local_carry),
            conditional(root.clone(), local_sum),
        ),
        cmp(
            conditional(root.clone(), sign(d, x.clone())),
            conditional(root.clone(), sign(d, y.clone())),
        ),
    );
    e = next(e, from_fn(end));
    p.theorem(&format!("TreeSignedBalance{d}"), &binders, as_fn(e));
    save_trace(p, trace, &format!("signed-{d}-balance"));
    let unsigned = addition(d, v("f"), x.clone(), y.clone(), incoming.clone());
    let result = sum(d, x.clone(), y.clone(), incoming.clone());
    let negative_result = conditional(root.clone(), sign(d, result.clone()));
    let left = cmp(unsigned.left.clone(), negative_result.clone());
    let mut e = p.rewrite(left, unsigned);
    let grouped = cmp(
        value(d, v("f"), result),
        cmp(
            conditional(
                root.clone(),
                carry(d, x.clone(), y.clone(), incoming.clone()),
            ),
            negative_result,
        ),
    );
    let norm = p.monoid(e.right.clone(), grouped);
    e = transit(e, norm);
    let q = instance(
        &format!("TreeSignedBalance{d}"),
        vec![v("f"), x.clone(), y.clone(), incoming.clone(), v("h")],
        cmp(
            conditional(
                root.clone(),
                carry(d, x.clone(), y.clone(), incoming.clone()),
            ),
            conditional(
                root.clone(),
                sign(d, sum(d, x.clone(), y.clone(), incoming.clone())),
            ),
        ),
        cmp(
            conditional(root.clone(), sign(d, x)),
            conditional(root, sign(d, y)),
        ),
    );
    e = p.change(e, q);
    p.theorem(&format!("TreeSignedAddition{d}"), &binders, e);
    save_trace(p, trace, &format!("signed-{d}-addition"));
}

pub(super) fn claim(p: &mut Producer, d: usize, mutation: usize) -> Expr {
    let x = v("x");
    let y = v("y");
    let c0 = literal(false);
    let f = c("Std.Nat.succ");
    let root = weight(d, f.clone());
    let result = sum(d, x.clone(), y.clone(), c0.clone());
    let (unsigned, _) = add_equation(d, f.clone(), x.clone(), y.clone(), c0.clone());
    let correct_left = cmp(
        unsigned.clone(),
        conditional(root.clone(), sign(d, result.clone())),
    );
    let right = cmp(
        value(d, f.clone(), result),
        cmp(
            conditional(root.clone(), sign(d, x.clone())),
            conditional(root, sign(d, y.clone())),
        ),
    );
    let left = if mutation == 1 {
        unsigned
    } else {
        correct_left.clone()
    };
    let predicate = if mutation == 2 {
        literal(true)
    } else {
        guard(d, x.clone(), y.clone(), c0.clone())
    };
    let proof = apply(
        c(&name(&format!("TreeSignedAddition{d}"))),
        vec![f, x, y, c0, v("h")],
    );
    let ctx = lambda(
        "__signed_eval",
        function(),
        apply(v("__signed_eval"), vec![v("initial")]),
    );
    let proof = apply(
        c("Std.Eq.congr"),
        vec![function(), nat(), ctx, correct_left, right.clone(), proof],
    );
    let goal = apply(
        c(EQ),
        vec![
            nat(),
            apply(left, vec![v("initial")]),
            apply(right, vec![v("initial")]),
        ],
    );
    let binders = vec![
        ("initial".into(), nat()),
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("h".into(), eq(predicate, literal(true))),
    ];
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("SignedTreeClaim", ty.clone(), proof, true);
    ty
}
