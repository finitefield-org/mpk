//! Derive full-width index coverage; no caller-supplied fit oracle.
use super::*;
mod coverage;

fn named(base: &str, d: usize, args: Vec<Expr>) -> Expr {
    tree(&format!("Index{base}"), d, args)
}
fn op(equal: bool, d: usize, x: Expr, y: Expr) -> Expr {
    if equal {
        word_eq(d, x, y)
    } else {
        word_less(d, x, y)
    }
}
fn op_body(equal: bool, d: usize, x: Expr, y: Expr) -> Expr {
    if d == 0 {
        if equal {
            bool_eq(x, y)
        } else {
            and(not(x), y)
        }
    } else if equal {
        and(
            word_eq(d - 1, project(x.clone(), false), project(y.clone(), false)),
            word_eq(d - 1, project(x, true), project(y, true)),
        )
    } else {
        choose(
            word_eq(d - 1, project(x.clone(), true), project(y.clone(), true)),
            word_less(d - 1, project(x.clone(), false), project(y.clone(), false)),
            word_less(d - 1, project(x, true), project(y, true)),
        )
    }
}
fn definition(equal: bool, d: usize, x: Expr, y: Expr) -> Typed {
    typed(
        bool_ty(),
        op(equal, d, x.clone(), y.clone()),
        op_body(equal, d, x.clone(), y.clone()),
        named(
            if equal {
                "EqualDefinition"
            } else {
                "LessDefinition"
            },
            d,
            vec![x, y],
        ),
    )
}
fn bool_local(p: &mut Producer, suffix: &str, args: &[&str], left: Expr, right: Expr) {
    let vars = args.iter().map(|s| v(s)).collect::<Vec<_>>();
    let proof =
        prove_with_checked_normalization(&eq(left.clone(), right.clone()), &vars, &mut p.fresh);
    theorem(
        p,
        suffix,
        &args
            .iter()
            .map(|s| (s.to_string(), bool_ty()))
            .collect::<Vec<_>>(),
        typed(bool_ty(), left, right, proof),
    );
}
fn primitives(p: &mut Producer) {
    bool_local(
        p,
        "IndexBoolSame",
        &["a", "b"],
        choose(v("a"), v("b"), v("b")),
        v("b"),
    );
    bool_local(
        p,
        "IndexBoolIdentity",
        &["a"],
        choose(v("a"), literal(true), literal(false)),
        v("a"),
    );
    for high in [false, true] {
        let value = v(if high { "b" } else { "a" });
        let condition = and(v("a"), v("b"));
        bool_local(
            p,
            if high {
                "IndexAndRight"
            } else {
                "IndexAndLeft"
            },
            &["a", "b"],
            choose(condition, value, literal(true)),
            literal(true),
        );
    }
    bool_local(
        p,
        "IndexBoolEqualValue",
        &["a", "b"],
        choose(bool_eq(v("a"), v("b")), v("a"), v("b")),
        v("b"),
    );
    bool_local(
        p,
        "IndexNotValue",
        &["a"],
        choose(not(v("a")), v("a"), literal(false)),
        literal(false),
    );
    bool_local(
        p,
        "IndexFalseThenValue",
        &["a", "b"],
        choose(
            choose(v("a"), literal(false), v("b")),
            v("b"),
            literal(true),
        ),
        literal(true),
    );
}
fn normalize(expr: Expr, p: &mut Producer) -> Typed {
    let beta = weak_beta(&expr);
    let mut e = typed(bool_ty(), expr, beta.clone(), refl(beta.clone()));
    let Expr::Apply(head, args) = &beta else {
        return e;
    };
    if **head != c("Std.Bool.cases") || args.len() != 4 {
        return e;
    }
    let Expr::Lambda(_, _, result) = &args[0] else {
        return e;
    };
    if **result != bool_ty() {
        return e;
    }
    for j in [3, 1, 2] {
        let q = normalize(args[j].clone(), p);
        if q.left != q.right && substitute(&e.right, &q.left, &q.right) != e.right {
            e = change(p, e, q);
        }
    }
    let q = order::normalized(e.right.clone(), p);
    e = next(e, q);
    if let Expr::Apply(head, args) = &e.right {
        if **head == c("Std.Bool.cases") && args.len() == 4 {
            let tail = if args[1] == args[2] {
                Some(typed(
                    bool_ty(),
                    e.right.clone(),
                    args[1].clone(),
                    apply(
                        c(&name("IndexBoolSame")),
                        vec![args[3].clone(), args[1].clone()],
                    ),
                ))
            } else if args[1] == literal(false) && args[2] == literal(true) {
                Some(typed(
                    bool_ty(),
                    e.right.clone(),
                    args[3].clone(),
                    apply(c(&name("IndexBoolIdentity")), vec![args[3].clone()]),
                ))
            } else {
                None
            };
            if let Some(q) = tail {
                e = next(e, q);
            }
        }
    }
    e
}
fn under(
    p: &mut Producer,
    condition: Expr,
    result: Expr,
    target: Expr,
    h: Expr,
    proof: Expr,
) -> Typed {
    order::under(p, condition, result, target, h, proof)
}
fn conjunction(p: &mut Producer, a: Expr, b: Expr, h: Expr, high: bool) -> Typed {
    under(
        p,
        and(a.clone(), b.clone()),
        if high { b.clone() } else { a.clone() },
        literal(true),
        h,
        apply(
            c(&name(if high {
                "IndexAndRight"
            } else {
                "IndexAndLeft"
            })),
            vec![a, b],
        ),
    )
}
fn not_true(p: &mut Producer, a: Expr, h: Expr) -> Typed {
    under(
        p,
        not(a.clone()),
        a.clone(),
        literal(false),
        h,
        apply(c(&name("IndexNotValue")), vec![a]),
    )
}
fn true_else(p: &mut Producer, a: Expr, b: Expr, h: Expr) -> Typed {
    under(
        p,
        choose(a.clone(), literal(false), b.clone()),
        b.clone(),
        literal(true),
        h,
        apply(c(&name("IndexFalseThenValue")), vec![a, b]),
    )
}
fn cases(major: Expr, hname: &str, goal: Expr, no: Expr, yes: Expr) -> Expr {
    apply(
        apply(
            c("Std.Bool.cases"),
            vec![
                lambda(
                    "__index_case",
                    bool_ty(),
                    forall(hname, eq(major.clone(), v("__index_case")), goal),
                ),
                lambda(hname, eq(major.clone(), literal(false)), no),
                lambda(hname, eq(major.clone(), literal(true)), yes),
                major.clone(),
            ],
        ),
        vec![refl(major)],
    )
}
fn impossible(ty: Expr, left: Expr, right: Expr, h: Expr) -> Typed {
    mapped(
        typed(bool_ty(), literal(false), literal(true), h),
        ty.clone(),
        lambda(
            "__index_impossible",
            bool_ty(),
            choose_t(ty, v("__index_impossible"), right.clone(), left.clone()),
        ),
        left,
        right,
    )
}
fn contradiction(yes: Typed, no: Expr) -> Expr {
    assert_eq!(yes.right, literal(true));
    next(
        backwards(typed(bool_ty(), yes.left.clone(), literal(false), no)),
        yes,
    )
    .proof
}
fn equality_parts(p: &mut Producer, d: usize, x: Expr, y: Expr, h: Expr, high: bool) -> Typed {
    let e = definition(true, d, x, y);
    let Expr::Apply(_, args) = &e.right else {
        panic!("equal conjunction absent")
    };
    let a = args[3].clone();
    let b = args[2].clone();
    let result = next(
        backwards(e.clone()),
        typed(bool_ty(), e.left, literal(true), h),
    );
    conjunction(p, a, b, result.proof, high)
}
fn equal_bits(d: usize, x: Expr, y: Expr, h: Expr) -> Expr {
    named("EqualBits", d, vec![x, y, h])
}
fn equal_from_bits(d: usize, x: Expr, y: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), y.clone()),
        literal(true),
        named("EqualFromBits", d, vec![x, y, h]),
    )
}
fn sym_equal(d: usize, x: Expr, y: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), y.clone()),
        word_eq(d, y.clone(), x.clone()),
        named("EqualSymmetric", d, vec![x, y]),
    )
}
fn refl_equal(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), x.clone()),
        literal(true),
        named("EqualReflexive", d, vec![x]),
    )
}
fn substitute_equal(
    d: usize,
    equal: bool,
    right: bool,
    a: Expr,
    b: Expr,
    z: Expr,
    h: Expr,
) -> Typed {
    let left = if right {
        op(equal, d, z.clone(), a.clone())
    } else {
        op(equal, d, a.clone(), z.clone())
    };
    let target = if right {
        op(equal, d, z.clone(), b.clone())
    } else {
        op(equal, d, b.clone(), z.clone())
    };
    typed(
        bool_ty(),
        left,
        target,
        named(
            &format!(
                "{}Substitute{}",
                if equal { "Equal" } else { "Less" },
                if right { "Right" } else { "Left" }
            ),
            d,
            vec![a, b, z, h],
        ),
    )
}
fn equal_not_less(d: usize, x: Expr, y: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        word_less(d, x.clone(), y.clone()),
        literal(false),
        named("EqualNotLess", d, vec![x, y, h]),
    )
}
fn less_not_equal(d: usize, x: Expr, y: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), y.clone()),
        literal(false),
        named("LessNotEqual", d, vec![x, y, h]),
    )
}
fn trichotomy_body(d: usize, x: Expr, y: Expr) -> Expr {
    or(
        word_less(d, x.clone(), y.clone()),
        or(word_eq(d, x.clone(), y.clone()), word_less(d, y, x)),
    )
}
fn trichotomy(d: usize, x: Expr, y: Expr) -> Typed {
    typed(
        bool_ty(),
        trichotomy_body(d, x.clone(), y.clone()),
        literal(true),
        named("Trichotomy", d, vec![x, y]),
    )
}
fn reverse_less(p: &mut Producer, d: usize, x: Expr, y: Expr, hlt: Expr, heq: Expr) -> Typed {
    let t = trichotomy(d, x.clone(), y.clone());
    let mut e = reflexive_t(bool_ty(), t.left.clone());
    e = change(
        p,
        e,
        typed(
            bool_ty(),
            word_less(d, x.clone(), y.clone()),
            literal(false),
            hlt,
        ),
    );
    e = change(
        p,
        e,
        typed(bool_ty(), word_eq(d, x, y), literal(false), heq),
    );
    let q = normalize(e.right.clone(), p);
    next(backwards(next(e, q)), t)
}
fn selected_less(p: &mut Producer, d: usize, x: Expr, y: Expr, truth: bool, h: Expr) -> Typed {
    let e = definition(false, d, x.clone(), y.clone());
    select(
        p,
        e,
        word_eq(d - 1, project(x, true), project(y, true)),
        truth,
        h,
    )
}
fn selected_truth(
    p: &mut Producer,
    d: usize,
    x: Expr,
    y: Expr,
    truth: bool,
    h: Expr,
    hless: Expr,
) -> Typed {
    let q = selected_less(p, d, x, y, truth, h);
    next(
        backwards(q.clone()),
        typed(bool_ty(), q.left, literal(true), hless),
    )
}
fn transitive(d: usize, x: Expr, y: Expr, z: Expr, hxy: Expr, hyz: Expr) -> Typed {
    typed(
        bool_ty(),
        word_less(d, x.clone(), z.clone()),
        literal(true),
        named("Transitive", d, vec![x, y, z, hxy, hyz]),
    )
}
fn mixed_transitive(d: usize, x: Expr, y: Expr, z: Expr, hxy: Expr, hzy: Expr) -> Typed {
    typed(
        bool_ty(),
        word_less(d, x.clone(), z.clone()),
        literal(true),
        named("LessLeTransitive", d, vec![x, y, z, hxy, hzy]),
    )
}
fn sign_bit(d: usize, x: Expr) -> Expr {
    apply(x, vec![literal(true); d])
}
fn sign_less(d: usize, x: Expr, y: Expr, hx: Expr, hy: Expr) -> Typed {
    typed(
        bool_ty(),
        word_less(d, x.clone(), y.clone()),
        literal(true),
        named("SignLess", d, vec![x, y, hx, hy]),
    )
}
fn literal_word(d: usize, n: u64) -> Expr {
    if d == 0 {
        literal(n & 1 != 0)
    } else {
        join(
            d,
            literal_word(d - 1, n),
            literal_word(d - 1, n >> (1usize << (d - 1))),
        )
    }
}
fn literal_projection(d: usize, n: u64, high: bool) -> Typed {
    let target = literal_word(d - 1, if high { n >> (1usize << (d - 1)) } else { n });
    typed(
        cube_type(d - 1),
        project(literal_word(d, n), high),
        target.clone(),
        apply(c(REFL), vec![cube_type(d - 1), target]),
    )
}
fn unfold_literal(p: &mut Producer, e: Typed, d: usize, n: u64) -> Typed {
    if d == 0 {
        return e;
    }
    let mut e = e;
    for high in [false, true] {
        e = change(p, e, literal_projection(d, n, high));
    }
    e
}
fn less_zero(d: usize, x: Expr) -> Typed {
    typed(
        bool_ty(),
        word_less(d, x.clone(), literal_word(d, 0)),
        literal(false),
        named("LessZero", d, vec![x]),
    )
}
fn less_one(d: usize, x: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        word_eq(d, x.clone(), literal_word(d, 0)),
        literal(true),
        named("LessOneZero", d, vec![x, h]),
    )
}
fn transitive_proof(p: &mut Producer, d: usize) -> Typed {
    let x = v("x");
    let y = v("y");
    let z = v("z");
    let xh = project(x.clone(), true);
    let yh = project(y.clone(), true);
    let zh = project(z.clone(), true);
    let output = word_less(d, x.clone(), z.clone());
    let goal = eq(output.clone(), literal(true));
    let mut branches = vec![];
    for first in [false, true] {
        let mut seconds = vec![];
        for second in [false, true] {
            let a = selected_truth(p, d, x.clone(), y.clone(), first, v("__eq_xy"), v("xy"));
            let b = selected_truth(p, d, y.clone(), z.clone(), second, v("__eq_yz"), v("yz"));
            let (heq, tail) = if first {
                let e = substitute_equal(
                    d - 1,
                    true,
                    false,
                    xh.clone(),
                    yh.clone(),
                    zh.clone(),
                    v("__eq_xy"),
                );
                let heq = next(
                    e,
                    typed(
                        bool_ty(),
                        word_eq(d - 1, yh.clone(), zh.clone()),
                        literal(second),
                        v("__eq_yz"),
                    ),
                );
                if second {
                    (
                        heq,
                        transitive(
                            d - 1,
                            project(x.clone(), false),
                            project(y.clone(), false),
                            project(z.clone(), false),
                            a.proof,
                            b.proof,
                        ),
                    )
                } else {
                    let e = substitute_equal(
                        d - 1,
                        false,
                        false,
                        xh.clone(),
                        yh.clone(),
                        zh.clone(),
                        v("__eq_xy"),
                    );
                    (heq, next(e, b))
                }
            } else if second {
                let e = backwards(substitute_equal(
                    d - 1,
                    true,
                    true,
                    yh.clone(),
                    zh.clone(),
                    xh.clone(),
                    v("__eq_yz"),
                ));
                let heq = next(
                    e,
                    typed(
                        bool_ty(),
                        word_eq(d - 1, xh.clone(), yh.clone()),
                        literal(false),
                        v("__eq_xy"),
                    ),
                );
                let e = backwards(substitute_equal(
                    d - 1,
                    false,
                    true,
                    yh.clone(),
                    zh.clone(),
                    xh.clone(),
                    v("__eq_yz"),
                ));
                (heq, next(e, a))
            } else {
                let t = transitive(d - 1, xh.clone(), yh.clone(), zh.clone(), a.proof, b.proof);
                (
                    less_not_equal(d - 1, xh.clone(), zh.clone(), t.proof.clone()),
                    t,
                )
            };
            let e = selected_less(p, d, x.clone(), z.clone(), first && second, heq.proof);
            seconds.push(next(e, tail).proof);
        }
        branches.push(cases(
            word_eq(d - 1, yh.clone(), zh.clone()),
            "__eq_yz",
            goal.clone(),
            seconds[0].clone(),
            seconds[1].clone(),
        ));
    }
    typed(
        bool_ty(),
        output,
        literal(true),
        cases(
            word_eq(d - 1, xh, yh),
            "__eq_xy",
            goal,
            branches[0].clone(),
            branches[1].clone(),
        ),
    )
}

fn build(p: &mut Producer, d: usize, trace: Option<&Path>) {
    let x = v("x");
    let y = v("y");
    let z = v("z");
    let pair = vec![("x".into(), cube_type(d)), ("y".into(), cube_type(d))];
    let one = vec![("x".into(), cube_type(d))];
    for equal in [false, true] {
        let mut e = definition(equal, d, x.clone(), y.clone());
        e.proof = refl(e.right.clone());
        theorem(
            p,
            &format!(
                "TreeIndex{}Definition{d}",
                if equal { "Equal" } else { "Less" }
            ),
            &pair,
            e,
        );
    }
    let mut eqargs = pair.clone();
    eqargs.push((
        "equal".into(),
        eq(word_eq(d, x.clone(), y.clone()), literal(true)),
    ));
    let bits = if d == 0 {
        let e = definition(true, d, x.clone(), y.clone());
        let h = next(
            backwards(e.clone()),
            typed(bool_ty(), e.left, literal(true), v("equal")),
        );
        under(
            p,
            bool_eq(x.clone(), y.clone()),
            x.clone(),
            y.clone(),
            h.proof,
            apply(c(&name("IndexBoolEqualValue")), vec![x.clone(), y.clone()]),
        )
        .proof
    } else {
        let rest = (1..d)
            .map(|j| v(&format!("__byte_bit_{j}")))
            .collect::<Vec<_>>();
        let mut branches = vec![];
        for high in [false, true] {
            let h = equality_parts(p, d, x.clone(), y.clone(), v("equal"), high);
            let h = apply(
                equal_bits(
                    d - 1,
                    project(x.clone(), high),
                    project(y.clone(), high),
                    h.proof,
                ),
                rest.clone(),
            );
            branches.push((1..d).rev().fold(h, |body, j| {
                lambda(&format!("__byte_bit_{j}"), bool_ty(), body)
            }));
        }
        let mut qs = vec![v("__index_first")];
        qs.extend(rest);
        let mut goal = eq(apply(x.clone(), qs.clone()), apply(y.clone(), qs));
        for j in (1..d).rev() {
            goal = forall(&format!("__byte_bit_{j}"), bool_ty(), goal);
        }
        lambda(
            "__byte_bit_0",
            bool_ty(),
            apply(
                c("Std.Bool.cases"),
                vec![
                    lambda("__index_first", bool_ty(), goal),
                    branches[0].clone(),
                    branches[1].clone(),
                    v("__byte_bit_0"),
                ],
            ),
        )
    };
    let (ty, proof) = fold_binders(&eqargs, observations(d, x.clone(), y.clone()), bits);
    p.declare(&format!("TreeIndexEqualBits{d}"), ty, proof, true);
    let mut e = definition(true, d, x.clone(), x.clone());
    if d == 0 {
        let q = prove_with_checked_normalization(
            &eq(e.right.clone(), literal(true)),
            std::slice::from_ref(&x),
            &mut p.fresh,
        );
        let tail = typed(bool_ty(), e.right.clone(), literal(true), q);
        e = next(e, tail);
    } else {
        for h in [false, true] {
            e = change(p, e, refl_equal(d - 1, project(x.clone(), h)));
        }
        let q = normalize(e.right.clone(), p);
        e = next(e, q);
    }
    theorem(p, &format!("TreeIndexEqualReflexive{d}"), &one, e);
    let mut args = pair.clone();
    args.push(("same_bits".into(), observations(d, x.clone(), y.clone())));
    let e = if d == 0 {
        let h = typed(bool_ty(), x.clone(), y.clone(), v("same_bits"));
        let e = definition(true, 0, x.clone(), y.clone());
        let e = change(p, e, backwards(h));
        let tail = typed(
            bool_ty(),
            e.right.clone(),
            literal(true),
            refl_equal(0, x.clone()).proof,
        );
        next(e, tail)
    } else {
        let mut e = definition(true, d, x.clone(), y.clone());
        for high in [false, true] {
            e = change(
                p,
                e,
                equal_from_bits(
                    d - 1,
                    project(x.clone(), high),
                    project(y.clone(), high),
                    child_observation(d, v("same_bits"), high),
                ),
            );
        }
        let q = normalize(e.right.clone(), p);
        next(e, q)
    };
    theorem(p, &format!("TreeIndexEqualFromBits{d}"), &args, e);
    let mut e = definition(true, d, x.clone(), y.clone());
    if d == 0 {
        let proof = prove_with_checked_normalization(
            &eq(e.right.clone(), op_body(true, 0, y.clone(), x.clone())),
            &[x.clone(), y.clone()],
            &mut p.fresh,
        );
        let tail = typed(
            bool_ty(),
            e.right.clone(),
            op_body(true, 0, y.clone(), x.clone()),
            proof,
        );
        e = next(e, tail);
    } else {
        for high in [false, true] {
            e = change(
                p,
                e,
                sym_equal(d - 1, project(x.clone(), high), project(y.clone(), high)),
            );
        }
    }
    e = next(e, backwards(definition(true, d, y.clone(), x.clone())));
    theorem(p, &format!("TreeIndexEqualSymmetric{d}"), &pair, e);
    for equal in [false, true] {
        for right in [false, true] {
            let a = v("a");
            let b = v("b");
            let z = v("z");
            let mut args = vec![
                ("a".into(), cube_type(d)),
                ("b".into(), cube_type(d)),
                ("z".into(), cube_type(d)),
            ];
            args.push((
                "equal".into(),
                eq(word_eq(d, a.clone(), b.clone()), literal(true)),
            ));
            let mut e = if right {
                definition(equal, d, z.clone(), a.clone())
            } else {
                definition(equal, d, a.clone(), z.clone())
            };
            if d == 0 {
                let h = under(
                    p,
                    bool_eq(a.clone(), b.clone()),
                    a.clone(),
                    b.clone(),
                    next(
                        backwards(definition(true, 0, a.clone(), b.clone())),
                        typed(
                            bool_ty(),
                            word_eq(0, a.clone(), b.clone()),
                            literal(true),
                            v("equal"),
                        ),
                    )
                    .proof,
                    apply(c(&name("IndexBoolEqualValue")), vec![a.clone(), b.clone()]),
                );
                e = change(p, e, h);
            } else {
                for high in [false, true] {
                    let h = equality_parts(p, d, a.clone(), b.clone(), v("equal"), high);
                    e = change(
                        p,
                        e,
                        substitute_equal(
                            d - 1,
                            equal,
                            right,
                            project(a.clone(), high),
                            project(b.clone(), high),
                            project(z.clone(), high),
                            h.proof.clone(),
                        ),
                    );
                    if !equal && high {
                        e = change(
                            p,
                            e,
                            substitute_equal(
                                d - 1,
                                true,
                                right,
                                project(a.clone(), high),
                                project(b.clone(), high),
                                project(z.clone(), high),
                                h.proof,
                            ),
                        );
                    }
                }
            }
            e = next(
                e,
                backwards(if right {
                    definition(equal, d, z, b)
                } else {
                    definition(equal, d, b, z)
                }),
            );
            theorem(
                p,
                &format!(
                    "TreeIndex{}Substitute{}{d}",
                    if equal { "Equal" } else { "Less" },
                    if right { "Right" } else { "Left" }
                ),
                &args,
                e,
            );
        }
    }
    let e = if d == 0 {
        let e = definition(false, 0, x.clone(), y.clone());
        let h = next(
            backwards(definition(true, 0, x.clone(), y.clone())),
            typed(
                bool_ty(),
                word_eq(0, x.clone(), y.clone()),
                literal(true),
                v("equal"),
            ),
        );
        let bit = under(
            p,
            bool_eq(x.clone(), y.clone()),
            x.clone(),
            y.clone(),
            h.proof,
            apply(c(&name("IndexBoolEqualValue")), vec![x.clone(), y.clone()]),
        );
        let e = change(p, e, bit);
        let q = prove_with_checked_normalization(
            &eq(e.right.clone(), literal(false)),
            std::slice::from_ref(&y),
            &mut p.fresh,
        );
        let tail = typed(bool_ty(), e.right.clone(), literal(false), q);
        next(e, tail)
    } else {
        let hi = equality_parts(p, d, x.clone(), y.clone(), v("equal"), true);
        let lo = equality_parts(p, d, x.clone(), y.clone(), v("equal"), false);
        let e = selected_less(p, d, x.clone(), y.clone(), true, hi.proof);
        next(
            e,
            equal_not_less(
                d - 1,
                project(x.clone(), false),
                project(y.clone(), false),
                lo.proof,
            ),
        )
    };
    theorem(p, &format!("TreeIndexEqualNotLess{d}"), &eqargs, e);
    let mut ltargs = pair.clone();
    ltargs.push((
        "less".into(),
        eq(word_less(d, x.clone(), y.clone()), literal(true)),
    ));
    let target = word_eq(d, x.clone(), y.clone());
    let none = equal_not_less(d, x.clone(), y.clone(), v("__equal"));
    let conflict = next(
        backwards(none),
        typed(
            bool_ty(),
            word_less(d, x.clone(), y.clone()),
            literal(true),
            v("less"),
        ),
    );
    let absurd = impossible(bool_ty(), target.clone(), literal(false), conflict.proof);
    let e = typed(
        bool_ty(),
        target.clone(),
        literal(false),
        cases(
            target.clone(),
            "__equal",
            eq(target, literal(false)),
            v("__equal"),
            absurd.proof,
        ),
    );
    theorem(p, &format!("TreeIndexLessNotEqual{d}"), &ltargs, e);
    let e = if d == 0 {
        let goal = trichotomy_body(0, x.clone(), y.clone());
        let body = or(
            op_body(false, 0, x.clone(), y.clone()),
            or(
                op_body(true, 0, x.clone(), y.clone()),
                op_body(false, 0, y.clone(), x.clone()),
            ),
        );
        let q = prove_with_checked_normalization(
            &eq(body.clone(), literal(true)),
            &[x.clone(), y.clone()],
            &mut p.fresh,
        );
        typed(bool_ty(), goal, literal(true), q)
    } else {
        let xh = project(x.clone(), true);
        let yh = project(y.clone(), true);
        let goal = eq(trichotomy_body(d, x.clone(), y.clone()), literal(true));
        let mut branches = vec![];
        for equal in [false, true] {
            let mut e = reflexive_t(bool_ty(), trichotomy_body(d, x.clone(), y.clone()));
            let selected = selected_less(p, d, x.clone(), y.clone(), equal, v("__high_equal"));
            e = change(p, e, selected);
            let sym = sym_equal(d - 1, yh.clone(), xh.clone());
            let heq = next(
                sym,
                typed(
                    bool_ty(),
                    word_eq(d - 1, xh.clone(), yh.clone()),
                    literal(equal),
                    v("__high_equal"),
                ),
            );
            let selected = selected_less(p, d, y.clone(), x.clone(), equal, heq.proof);
            e = change(p, e, selected);
            e = change(p, e, definition(true, d, x.clone(), y.clone()));
            e = change(
                p,
                e,
                typed(
                    bool_ty(),
                    word_eq(d - 1, xh.clone(), yh.clone()),
                    literal(equal),
                    v("__high_equal"),
                ),
            );
            let q = normalize(e.right.clone(), p);
            e = next(e, q);
            let child = if equal {
                trichotomy(d - 1, project(x.clone(), false), project(y.clone(), false))
            } else {
                let mut t = trichotomy(d - 1, xh.clone(), yh.clone());
                let q = replace(
                    p,
                    bool_ty(),
                    t.left.clone(),
                    typed(
                        bool_ty(),
                        word_eq(d - 1, xh.clone(), yh.clone()),
                        literal(false),
                        v("__high_equal"),
                    ),
                );
                t = next(backwards(q.clone()), t);
                let simplified = normalize(q.right, p);
                next(backwards(simplified), t)
            };
            branches.push(next(e, child).proof);
        }
        typed(
            bool_ty(),
            trichotomy_body(d, x.clone(), y.clone()),
            literal(true),
            cases(
                word_eq(d - 1, xh, yh),
                "__high_equal",
                goal,
                branches[0].clone(),
                branches[1].clone(),
            ),
        )
    };
    theorem(p, &format!("TreeIndexTrichotomy{d}"), &pair, e);
    let mut targs = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("z".into(), cube_type(d)),
    ];
    targs.push((
        "xy".into(),
        eq(word_less(d, x.clone(), y.clone()), literal(true)),
    ));
    targs.push((
        "yz".into(),
        eq(word_less(d, y.clone(), z.clone()), literal(true)),
    ));
    let e = if d == 0 {
        let condition = and(
            op_body(false, 0, x.clone(), y.clone()),
            op_body(false, 0, y.clone(), z.clone()),
        );
        let left = choose(
            condition.clone(),
            op_body(false, 0, x.clone(), z.clone()),
            literal(true),
        );
        let q = prove_with_checked_normalization(
            &eq(left.clone(), literal(true)),
            &[x.clone(), y.clone(), z.clone()],
            &mut p.fresh,
        );
        let mut h = reflexive_t(bool_ty(), condition.clone());
        h = change(p, h, backwards(definition(false, 0, x.clone(), y.clone())));
        h = change(p, h, backwards(definition(false, 0, y.clone(), z.clone())));
        h = change(
            p,
            h,
            typed(
                bool_ty(),
                word_less(0, x.clone(), y.clone()),
                literal(true),
                v("xy"),
            ),
        );
        h = change(
            p,
            h,
            typed(
                bool_ty(),
                word_less(0, y.clone(), z.clone()),
                literal(true),
                v("yz"),
            ),
        );
        let normal = normalize(h.right.clone(), p);
        let h = next(h, normal);
        let mut e = under(
            p,
            condition,
            op_body(false, 0, x.clone(), z.clone()),
            literal(true),
            h.proof,
            q,
        );
        e.left = word_less(0, x.clone(), z.clone());
        e
    } else {
        transitive_proof(p, d)
    };
    theorem(p, &format!("TreeIndexTransitive{d}"), &targs, e);
    let mut args = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("z".into(), cube_type(d)),
    ];
    args.push((
        "xy".into(),
        eq(word_less(d, x.clone(), y.clone()), literal(true)),
    ));
    args.push((
        "zy".into(),
        eq(word_less(d, z.clone(), y.clone()), literal(false)),
    ));
    let target = word_less(d, x.clone(), z.clone());
    let goal = eq(target.clone(), literal(true));
    let reverse = reverse_less(p, d, x.clone(), z.clone(), v("__xz_less"), v("__xz_equal"));
    let bad = transitive(d, z.clone(), x.clone(), y.clone(), reverse.proof, v("xy"));
    let no = impossible(
        bool_ty(),
        target.clone(),
        literal(true),
        contradiction(bad, v("zy")),
    )
    .proof;
    let q = substitute_equal(
        d,
        false,
        false,
        x.clone(),
        z.clone(),
        y.clone(),
        v("__xz_equal"),
    );
    let bad = next(
        backwards(q.clone()),
        typed(bool_ty(), q.left, literal(true), v("xy")),
    );
    let yes = impossible(
        bool_ty(),
        target.clone(),
        literal(true),
        contradiction(bad, v("zy")),
    )
    .proof;
    let no = cases(
        word_eq(d, x.clone(), z.clone()),
        "__xz_equal",
        goal.clone(),
        no,
        yes,
    );
    let proof = cases(target.clone(), "__xz_less", goal, no, v("__xz_less"));
    theorem(
        p,
        &format!("TreeIndexLessLeTransitive{d}"),
        &args,
        typed(bool_ty(), target, literal(true), proof),
    );
    let mut args = pair.clone();
    args.push(("x_sign".into(), eq(sign_bit(d, x.clone()), literal(false))));
    args.push(("y_sign".into(), eq(sign_bit(d, y.clone()), literal(true))));
    let e = if d == 0 {
        let mut e = definition(false, 0, x.clone(), y.clone());
        e = change(
            p,
            e,
            typed(bool_ty(), x.clone(), literal(false), v("x_sign")),
        );
        e = change(
            p,
            e,
            typed(bool_ty(), y.clone(), literal(true), v("y_sign")),
        );
        let q = normalize(e.right.clone(), p);
        next(e, q)
    } else {
        let high = sign_less(
            d - 1,
            project(x.clone(), true),
            project(y.clone(), true),
            v("x_sign"),
            v("y_sign"),
        );
        let h = less_not_equal(
            d - 1,
            project(x.clone(), true),
            project(y.clone(), true),
            high.proof.clone(),
        );
        let e = selected_less(p, d, x.clone(), y.clone(), false, h.proof);
        next(e, high)
    };
    theorem(p, &format!("TreeIndexSignLess{d}"), &args, e);
    let e = definition(false, d, x.clone(), literal_word(d, 0));
    let mut e = unfold_literal(p, e, d, 0);
    if d > 0 {
        for high in [false, true] {
            e = change(p, e, less_zero(d - 1, project(x.clone(), high)));
        }
    }
    let q = normalize(e.right.clone(), p);
    e = next(e, q);
    theorem(p, &format!("TreeIndexLessZero{d}"), &one, e);
    let mut args = one.clone();
    args.push((
        "less_one".into(),
        eq(word_less(d, x.clone(), literal_word(d, 1)), literal(true)),
    ));
    let e = if d == 0 {
        let a = definition(false, 0, x.clone(), literal_word(0, 1));
        let q = normalize(a.right.clone(), p);
        let a = next(a, q);
        let h = next(
            backwards(a.clone()),
            typed(bool_ty(), a.left, literal(true), v("less_one")),
        );
        let b = definition(true, 0, x.clone(), literal_word(0, 0));
        let q = normalize(b.right.clone(), p);
        next(next(b, q), h)
    } else {
        let a = definition(false, d, x.clone(), literal_word(d, 1));
        let mut a = unfold_literal(p, a, d, 1);
        a = change(p, a, less_zero(d - 1, project(x.clone(), true)));
        let h = next(
            backwards(a.clone()),
            typed(bool_ty(), a.left, literal(true), v("less_one")),
        );
        let hi = word_eq(d - 1, project(x.clone(), true), literal_word(d - 1, 0));
        let lo = word_less(d - 1, project(x.clone(), false), literal_word(d - 1, 1));
        let hh = conjunction(p, hi.clone(), lo.clone(), h.proof.clone(), false);
        let hl = conjunction(p, hi, lo, h.proof, true);
        let low = less_one(d - 1, project(x.clone(), false), hl.proof);
        let b = definition(true, d, x.clone(), literal_word(d, 0));
        let mut b = unfold_literal(p, b, d, 0);
        b = change(p, b, hh);
        b = change(p, b, low);
        let q = normalize(b.right.clone(), p);
        next(b, q)
    };
    theorem(p, &format!("TreeIndexLessOneZero{d}"), &args, e);
    save_trace(p, trace, &format!("index-order-{d}"));
}

pub(super) fn generate(root: &Path, output: &Path, capacity: usize, trace: bool) {
    assert!([0, 1, 32, 256].contains(&capacity));
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.IndexBoundsCapacity{capacity}");
    primitives(&mut p);
    for d in 0..=WORD {
        word_library(&mut p, d);
        build(&mut p, d, trace.then_some(output));
    }
    coverage::build(&mut p, capacity, trace.then_some(output));
    if trace {
        fs::write(
            output.join("library.mpcert"),
            Builder::from(p.builder.c.clone()).finish(),
        )
        .unwrap();
    }
    coverage::emit(p.builder.c, output, capacity);
}
