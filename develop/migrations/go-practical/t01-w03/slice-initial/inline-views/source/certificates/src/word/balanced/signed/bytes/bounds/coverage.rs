//! Actual source-width bounds imply value-preserving finite index embedding.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Node {
    d: usize,
    p: usize,
}
fn symbol(base: &str, n: Node) -> String {
    format!("Index{base}{}_{}", n.d, n.p)
}
fn call(base: &str, n: Node, args: Vec<Expr>) -> Expr {
    apply(c(&name(&symbol(base, n))), args)
}
fn collect(n: Node, out: &mut BTreeSet<Node>) {
    if !out.insert(n) || n.p == 0 || n.p >= (1usize << n.d) {
        return;
    }
    let half = 1usize << (n.d - 1);
    if n.p != half {
        collect(
            Node {
                d: n.d - 1,
                p: if n.p < half { n.p } else { n.p - half },
            },
            out,
        );
    }
}
fn fits(n: Node, x: Expr) -> Expr {
    call("Fits", n, vec![x])
}
fn prefix(n: Node, x: Expr) -> Expr {
    call("Prefix", n, vec![x])
}
fn fits_body(n: Node, x: Expr) -> Expr {
    if n.p == 0 {
        word_eq(n.d, x, literal_word(n.d, 0))
    } else if n.p >= (1usize << n.d) {
        literal(true)
    } else {
        let half = 1usize << (n.d - 1);
        let high_zero = word_eq(n.d - 1, project(x.clone(), true), literal_word(n.d - 1, 0));
        if n.p < half {
            and(
                high_zero,
                fits(Node { d: n.d - 1, p: n.p }, project(x, false)),
            )
        } else if n.p == half {
            high_zero
        } else {
            fits(
                Node {
                    d: n.d - 1,
                    p: n.p - half,
                },
                project(x, true),
            )
        }
    }
}
fn prefix_body(n: Node, x: Expr) -> Expr {
    if n.p == 0 {
        literal_word(n.d, 0)
    } else if n.p >= (1usize << n.d) {
        canon(n.d, x)
    } else {
        let half = 1usize << (n.d - 1);
        if n.p < half {
            join(
                n.d,
                prefix(Node { d: n.d - 1, p: n.p }, project(x, false)),
                literal_word(n.d - 1, 0),
            )
        } else if n.p == half {
            join(
                n.d,
                canon(n.d - 1, project(x, false)),
                literal_word(n.d - 1, 0),
            )
        } else {
            join(
                n.d,
                canon(n.d - 1, project(x.clone(), false)),
                prefix(
                    Node {
                        d: n.d - 1,
                        p: n.p - half,
                    },
                    project(x, true),
                ),
            )
        }
    }
}
fn fit_definition(n: Node, x: Expr) -> Typed {
    typed(
        bool_ty(),
        fits(n, x.clone()),
        fits_body(n, x.clone()),
        call("FitsDefinition", n, vec![x]),
    )
}
fn prefix_projection(n: Node, x: Expr, high: bool) -> Typed {
    assert!(n.p > 0 && n.p < (1usize << n.d));
    let half = 1usize << (n.d - 1);
    let field = project(x.clone(), high);
    let right = if high {
        if n.p <= half {
            literal_word(n.d - 1, 0)
        } else {
            prefix(
                Node {
                    d: n.d - 1,
                    p: n.p - half,
                },
                field,
            )
        }
    } else if n.p >= half {
        canon(n.d - 1, field)
    } else {
        prefix(Node { d: n.d - 1, p: n.p }, field)
    };
    typed(
        cube_type(n.d - 1),
        project(prefix(n, x.clone()), high),
        right,
        call(if high { "PrefixHigh" } else { "PrefixLow" }, n, vec![x]),
    )
}
fn less_fit(n: Node, x: Expr, h: Expr) -> Typed {
    typed(
        bool_ty(),
        fits(n, x.clone()),
        literal(true),
        call("LessPowerFits", n, vec![x, h]),
    )
}
fn fit_bits(n: Node, x: Expr, h: Expr) -> Expr {
    call("FitsBits", n, vec![x, h])
}
fn addr(d: usize, position: usize) -> Vec<Expr> {
    (0..d)
        .rev()
        .map(|j| literal((position >> j) & 1 != 0))
        .collect()
}
fn point(n: Node, x: Expr, position: usize) -> Typed {
    let qs = addr(n.d, position);
    let right = if position < n.p {
        apply(x.clone(), qs.clone())
    } else {
        literal(false)
    };
    let left = apply(prefix(n, x.clone()), qs.clone());
    if n.p == 0 {
        return typed(bool_ty(), left, right, refl(literal(false)));
    }
    if n.p >= (1usize << n.d) {
        return typed(bool_ty(), left, right, apply(canon_bits(n.d, x), qs));
    }
    let half = 1usize << (n.d - 1);
    let high = position >= half;
    let rest = addr(n.d - 1, position % half);
    let q = prefix_projection(n, x.clone(), high);
    let a = apply(q.left.clone(), rest.clone());
    let b = apply(q.right.clone(), rest.clone());
    let q = mapped(
        q,
        bool_ty(),
        lambda(
            "__coverage_view",
            cube_type(n.d - 1),
            apply(v("__coverage_view"), rest.clone()),
        ),
        a,
        b,
    );
    let tail = if high && n.p <= half {
        typed(
            bool_ty(),
            q.right.clone(),
            literal(false),
            refl(literal(false)),
        )
    } else if !high && n.p >= half {
        typed(
            bool_ty(),
            q.right.clone(),
            apply(project(x.clone(), false), rest.clone()),
            apply(canon_bits(n.d - 1, project(x.clone(), false)), rest),
        )
    } else {
        point(
            Node {
                d: n.d - 1,
                p: if high { n.p - half } else { n.p },
            },
            project(x.clone(), high),
            position % half,
        )
    };
    let proof = next(q, tail).proof;
    typed(bool_ty(), left, right, proof)
}
fn address_cases(
    p: &mut Producer,
    goal: &Expr,
    majors: &[Expr],
    leaf: &mut dyn FnMut(&mut Producer, &Expr) -> Expr,
) -> Expr {
    if let Some((major, rest)) = majors.split_first() {
        let tag = format!("__coverage_case_{}", p.fresh);
        p.fresh += 1;
        let no = address_cases(p, &substitute(goal, major, &literal(false)), rest, leaf);
        let yes = address_cases(p, &substitute(goal, major, &literal(true)), rest, leaf);
        apply(
            c("Std.Bool.cases"),
            vec![
                lambda(&tag, bool_ty(), substitute(goal, major, &v(&tag))),
                no,
                yes,
                major.clone(),
            ],
        )
    } else {
        leaf(p, goal)
    }
}
fn literal_value(d: usize, value: u64, qs: Vec<Expr>) -> Typed {
    let pos = qs
        .iter()
        .fold(0usize, |n, q| (n << 1) + usize::from(*q == literal(true)));
    let bit = literal((value >> pos) & 1 != 0);
    typed(
        bool_ty(),
        apply(literal_word(d, value), qs),
        bit.clone(),
        refl(bit),
    )
}
fn compatible_literal(p: &mut Producer, capacity: usize) -> Typed {
    let a = constant_word(capacity);
    let b = literal_word(WORD, capacity as u64);
    let qs = addresses(WORD, "__coverage_literal_");
    let goal = eq(apply(a.clone(), qs.clone()), apply(b.clone(), qs.clone()));
    let proof = address_cases(p, &goal, &qs, &mut |p, goal| {
        let Expr::Apply(_, args) = goal else {
            panic!("literal goal")
        };
        let left = normalize(args[1].clone(), p);
        let Expr::Apply(_, query) = &args[2] else {
            panic!("literal application")
        };
        let right = literal_value(WORD, capacity as u64, query.clone());
        assert_eq!(left.right, right.right);
        next(left, backwards(right)).proof
    });
    let h = cube_lambda(WORD, "__coverage_literal_", proof);
    equal_from_bits(WORD, a, b, h)
}
fn embedding_bits(p: &mut Producer, n: Node, x: Expr) -> Expr {
    assert_eq!(n.d, WORD);
    let qs = addresses(WORD, "__coverage_embedding_");
    let embedded = index_word(&low_bits(x.clone(), n.p));
    let goal = eq(
        apply(prefix(n, x.clone()), qs.clone()),
        apply(embedded, qs.clone()),
    );
    let proof = address_cases(p, &goal, &qs, &mut |p, goal| {
        let Expr::Apply(_, args) = goal else {
            panic!("embedding equality")
        };
        let Expr::Apply(_, parts) = &args[1] else {
            panic!("prefix application")
        };
        let position = parts.iter().fold(0usize, |acc, b| {
            (acc << 1) + usize::from(*b == literal(true))
        });
        let a = point(n, x.clone(), position);
        let b = normalize(args[2].clone(), p);
        assert_eq!(a.right, b.right);
        next(a, backwards(b)).proof
    });
    cube_lambda(WORD, "__coverage_embedding_", proof)
}
fn build_node(p: &mut Producer, n: Node, trace: Option<&Path>) {
    let x = v("word");
    let args = vec![("word".into(), cube_type(n.d))];
    function_definition(
        p,
        &symbol("Fits", n),
        &args,
        bool_ty(),
        fits_body(n, x.clone()),
    );
    function_definition(
        p,
        &symbol("Prefix", n),
        &args,
        cube_type(n.d),
        prefix_body(n, x.clone()),
    );
    let mut e = fit_definition(n, x.clone());
    e.proof = refl(e.right.clone());
    theorem(p, &symbol("FitsDefinition", n), &args, e);
    if n.p > 0 && n.p < (1usize << n.d) {
        for high in [false, true] {
            let mut q = prefix_projection(n, x.clone(), high);
            q.proof = apply(c(REFL), vec![q.ty.clone(), q.right.clone()]);
            theorem(
                p,
                &symbol(if high { "PrefixHigh" } else { "PrefixLow" }, n),
                &args,
                q,
            );
        }
    }
    let mut largs = args.clone();
    largs.push((
        "less".into(),
        eq(
            word_less(n.d, x.clone(), literal_word(n.d, 1u64 << n.p)),
            literal(true),
        ),
    ));
    let e = if n.p == 0 {
        next(
            fit_definition(n, x.clone()),
            less_one(n.d, x.clone(), v("less")),
        )
    } else {
        let half = 1usize << (n.d - 1);
        let a = definition(false, n.d, x.clone(), literal_word(n.d, 1u64 << n.p));
        let mut a = unfold_literal(p, a, n.d, 1u64 << n.p);
        let mut e = fit_definition(n, x.clone());
        if n.p < half {
            a = change(p, a, less_zero(n.d - 1, project(x.clone(), true)));
            let h = next(
                backwards(a.clone()),
                typed(bool_ty(), a.left, literal(true), v("less")),
            );
            let high = word_eq(n.d - 1, project(x.clone(), true), literal_word(n.d - 1, 0));
            let low = word_less(
                n.d - 1,
                project(x.clone(), false),
                literal_word(n.d - 1, 1u64 << n.p),
            );
            let hh = conjunction(p, high.clone(), low.clone(), h.proof.clone(), false);
            let hl = conjunction(p, high, low, h.proof, true);
            let child = less_fit(
                Node { d: n.d - 1, p: n.p },
                project(x.clone(), false),
                hl.proof,
            );
            e = change(p, e, hh);
            e = change(p, e, child);
            let q = normalize(e.right.clone(), p);
            e = next(e, q);
        } else {
            a = change(p, a, less_zero(n.d - 1, project(x.clone(), false)));
            let h = next(
                backwards(a.clone()),
                typed(bool_ty(), a.left, literal(true), v("less")),
            );
            let cond = word_eq(
                n.d - 1,
                project(x.clone(), true),
                literal_word(n.d - 1, 1u64 << (n.p - half)),
            );
            let lt = word_less(
                n.d - 1,
                project(x.clone(), true),
                literal_word(n.d - 1, 1u64 << (n.p - half)),
            );
            let h = true_else(p, cond, lt, h.proof);
            let child = if n.p == half {
                less_one(n.d - 1, project(x.clone(), true), h.proof)
            } else {
                less_fit(
                    Node {
                        d: n.d - 1,
                        p: n.p - half,
                    },
                    project(x.clone(), true),
                    h.proof,
                )
            };
            e = next(e, child);
        }
        e
    };
    theorem(p, &symbol("LessPowerFits", n), &largs, e);
    let mut fargs = args.clone();
    fargs.push(("fits".into(), eq(fits(n, x.clone()), literal(true))));
    let h = next(
        backwards(fit_definition(n, x.clone())),
        typed(bool_ty(), fits(n, x.clone()), literal(true), v("fits")),
    );
    let proof = if n.p == 0 {
        equal_bits(n.d, x.clone(), literal_word(n.d, 0), h.proof)
    } else if n.p >= (1usize << n.d) {
        let qs = addresses(n.d, "__byte_bit_");
        let e = typed(
            bool_ty(),
            apply(canon(n.d, x.clone()), qs.clone()),
            apply(x.clone(), qs.clone()),
            apply(canon_bits(n.d, x.clone()), qs),
        );
        cube_lambda(n.d, "__byte_bit_", backwards(e).proof)
    } else {
        let half = 1usize << (n.d - 1);
        let rest = (1..n.d)
            .map(|j| v(&format!("__byte_bit_{j}")))
            .collect::<Vec<_>>();
        let mut branches = vec![];
        for high in [false, true] {
            let field = project(x.clone(), high);
            let q = prefix_projection(n, x.clone(), high);
            let q = mapped(
                q.clone(),
                bool_ty(),
                lambda(
                    "__coverage_bit_view",
                    cube_type(n.d - 1),
                    apply(v("__coverage_bit_view"), rest.clone()),
                ),
                apply(q.left, rest.clone()),
                apply(q.right, rest.clone()),
            );
            let tail = if high && n.p <= half {
                let hh = if n.p == half {
                    h.proof.clone()
                } else {
                    conjunction(
                        p,
                        word_eq(n.d - 1, project(x.clone(), true), literal_word(n.d - 1, 0)),
                        fits(Node { d: n.d - 1, p: n.p }, project(x.clone(), false)),
                        h.proof.clone(),
                        false,
                    )
                    .proof
                };
                typed(
                    bool_ty(),
                    apply(field.clone(), rest.clone()),
                    q.right.clone(),
                    apply(
                        equal_bits(n.d - 1, field, literal_word(n.d - 1, 0), hh),
                        rest.clone(),
                    ),
                )
            } else if !high && n.p >= half {
                backwards(typed(
                    bool_ty(),
                    q.right.clone(),
                    apply(field.clone(), rest.clone()),
                    apply(canon_bits(n.d - 1, field), rest.clone()),
                ))
            } else {
                let child = Node {
                    d: n.d - 1,
                    p: if high { n.p - half } else { n.p },
                };
                let hc = if high {
                    h.proof.clone()
                } else {
                    conjunction(
                        p,
                        word_eq(n.d - 1, project(x.clone(), true), literal_word(n.d - 1, 0)),
                        fits(child, field.clone()),
                        h.proof.clone(),
                        true,
                    )
                    .proof
                };
                typed(
                    bool_ty(),
                    apply(field.clone(), rest.clone()),
                    q.right.clone(),
                    apply(fit_bits(child, field, hc), rest.clone()),
                )
            };
            let e = next(tail, backwards(q));
            branches.push((1..n.d).rev().fold(e.proof, |body, j| {
                lambda(&format!("__byte_bit_{j}"), bool_ty(), body)
            }));
        }
        let mut qs = vec![v("__coverage_first")];
        qs.extend(rest);
        let mut goal = eq(
            apply(x.clone(), qs.clone()),
            apply(prefix(n, x.clone()), qs),
        );
        for j in (1..n.d).rev() {
            goal = forall(&format!("__byte_bit_{j}"), bool_ty(), goal);
        }
        lambda(
            "__byte_bit_0",
            bool_ty(),
            apply(
                c("Std.Bool.cases"),
                vec![
                    lambda("__coverage_first", bool_ty(), goal),
                    branches[0].clone(),
                    branches[1].clone(),
                    v("__byte_bit_0"),
                ],
            ),
        )
    };
    let (ty, proof) = fold_binders(&fargs, observations(n.d, x.clone(), prefix(n, x)), proof);
    p.declare(&symbol("FitsBits", n), ty, proof, true);
    save_trace(p, trace, &format!("index-fit-{}-{}", n.d, n.p));
}

fn source_guard(l: Expr, i: Expr) -> Expr {
    let i = canon(WORD, i);
    let si = sign_bit(WORD, i.clone());
    let sl = sign_bit(WORD, l.clone());
    and(
        not(si.clone()),
        choose(bool_eq(si.clone(), sl), word_less(WORD, i, l), si),
    )
}
fn bound_value(l: Expr, capacity: usize) -> Expr {
    not(word_less(WORD, constant_word(capacity), l))
}
fn length_nonnegative(p: &mut Producer, capacity: usize, l: Expr, h: Expr) -> Typed {
    let bad = word_less(WORD, constant_word(capacity), l.clone());
    let no = not_true(p, bad.clone(), h);
    let source = constant_word(capacity);
    let qs = vec![literal(true); WORD];
    let sign = normalize(apply(source.clone(), qs), p);
    assert_eq!(sign.right, literal(false));
    let less = sign_less(WORD, source, l.clone(), sign.proof, v("__length_sign"));
    let absurd = impossible(
        bool_ty(),
        sign_bit(WORD, l.clone()),
        literal(false),
        contradiction(less, no.proof),
    );
    let goal = eq(sign_bit(WORD, l.clone()), literal(false));
    typed(
        bool_ty(),
        sign_bit(WORD, l.clone()),
        literal(false),
        cases(
            sign_bit(WORD, l),
            "__length_sign",
            goal,
            v("__length_sign"),
            absurd.proof,
        ),
    )
}
fn guard_exact(p: &mut Producer, capacity: usize, l: Expr, i: Expr, h: Expr) -> Typed {
    let mut e = reflexive_t(bool_ty(), source_guard(l.clone(), i.clone()));
    let sign = length_nonnegative(p, capacity, l.clone(), h);
    e = change(p, e, sign);
    let si = sign_bit(WORD, canon(WORD, i.clone()));
    let lt = word_less(WORD, canon(WORD, i.clone()), l.clone());
    let q = typed(
        bool_ty(),
        e.right.clone(),
        active(i, l),
        apply(c(&name("IndexSignedGuard")), vec![si, lt]),
    );
    next(e, q)
}
fn coverage(
    p: &mut Producer,
    capacity: usize,
    l: Expr,
    i: Expr,
    hbound: Expr,
    hsafe: Expr,
) -> Typed {
    let bits = capacity.ilog2() as usize;
    let n = Node { d: WORD, p: bits };
    let x = canon(WORD, i.clone());
    let g = guard_exact(p, capacity, l.clone(), i.clone(), hbound.clone());
    let hactive = next(
        backwards(g.clone()),
        typed(bool_ty(), g.left, literal(true), hsafe),
    );
    let hlt = conjunction(
        p,
        not(sign_bit(WORD, x.clone())),
        word_less(WORD, x.clone(), l.clone()),
        hactive.proof,
        true,
    );
    let literal = compatible_literal(p, capacity);
    let sub = substitute_equal(
        WORD,
        false,
        false,
        constant_word(capacity),
        literal_word(WORD, capacity as u64),
        l.clone(),
        literal.proof,
    );
    let hfalse = not_true(
        p,
        word_less(WORD, constant_word(capacity), l.clone()),
        hbound,
    );
    let hfalse = next(backwards(sub), hfalse);
    let hi = mixed_transitive(
        WORD,
        x.clone(),
        l,
        literal_word(WORD, capacity as u64),
        hlt.proof,
        hfalse.proof,
    );
    let fit = less_fit(n, x.clone(), hi.proof);
    let hbits = fit_bits(n, x.clone(), fit.proof);
    let a = canon_congr(WORD, x.clone(), prefix(n, x.clone()), hbits);
    let a = next(backwards(canon_idem(WORD, i.clone())), a);
    let h = call("PrefixEmbeddingBits", n, vec![x.clone()]);
    let b = canon_congr(
        WORD,
        prefix(n, x.clone()),
        index_word(&low_bits(x.clone(), bits)),
        h,
    );
    let mut e = next(a, b);
    for bit in 0..bits {
        let qs = bit_address(bit);
        let q = typed(
            bool_ty(),
            apply(x.clone(), qs.clone()),
            apply(i.clone(), qs.clone()),
            apply(canon_bits(WORD, i.clone()), qs),
        );
        e = change(p, e, q);
    }
    e
}
fn empty_no_index(p: &mut Producer, l: Expr, i: Expr, hbound: Expr) -> Typed {
    let literal = compatible_literal(p, 0);
    let q = substitute_equal(
        WORD,
        false,
        false,
        constant_word(0),
        literal_word(WORD, 0),
        l.clone(),
        literal.proof,
    );
    let no = not_true(
        p,
        word_less(WORD, constant_word(0), l.clone()),
        hbound.clone(),
    );
    let no = next(backwards(q), no);
    let zero = literal_word(WORD, 0);
    let tri = trichotomy(WORD, zero.clone(), l.clone());
    let mut e = reflexive_t(bool_ty(), tri.left.clone());
    e = change(p, e, no);
    e = change(p, e, less_zero(WORD, l.clone()));
    let q = normalize(e.right.clone(), p);
    let h = next(backwards(next(e, q)), tri);
    let x = canon(WORD, i.clone());
    let q = backwards(substitute_equal(
        WORD,
        false,
        true,
        zero,
        l.clone(),
        x.clone(),
        h.proof,
    ));
    let q = next(q, less_zero(WORD, x));
    let mut e = guard_exact(p, 0, l, i, hbound);
    e = change(p, e, q);
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
pub(super) fn build(p: &mut Producer, capacity: usize, trace: Option<&Path>) {
    bool_local(
        p,
        "IndexSignedGuard",
        &["s", "lt"],
        and(
            not(v("s")),
            choose(bool_eq(v("s"), literal(false)), v("lt"), v("s")),
        ),
        and(not(v("s")), v("lt")),
    );
    if capacity > 0 {
        let n = Node {
            d: WORD,
            p: capacity.ilog2() as usize,
        };
        let mut nodes = BTreeSet::new();
        collect(n, &mut nodes);
        for child in nodes {
            build_node(p, child, trace);
        }
        let x = v("word");
        let proof = embedding_bits(p, n, x.clone());
        let (ty, proof) = fold_binders(
            &[("word".into(), word_ty())],
            observations(WORD, prefix(n, x.clone()), index_word(&low_bits(x, n.p))),
            proof,
        );
        p.declare(&symbol("PrefixEmbeddingBits", n), ty, proof, true);
    }
    let l = v("length");
    let i = v("index");
    let args = vec![
        ("length".into(), word_ty()),
        ("index".into(), word_ty()),
        (
            "bound".into(),
            eq(bound_value(l.clone(), capacity), literal(true)),
        ),
    ];
    let q = length_nonnegative(p, capacity, l.clone(), v("bound"));
    theorem(p, "IndexLengthNonnegative", &args, q);
    let q = guard_exact(p, capacity, l.clone(), i.clone(), v("bound"));
    theorem(p, "IndexGuardExact", &args, q);
    if capacity > 0 {
        let mut args = args;
        args.push((
            "safe".into(),
            eq(source_guard(l.clone(), i.clone()), literal(true)),
        ));
        let q = coverage(p, capacity, l, i, v("bound"), v("safe"));
        theorem(p, "IndexCoverage", &args, q);
    } else {
        let q = empty_no_index(p, l, i, v("bound"));
        theorem(p, "IndexEmptyNoIndex", &args, q);
    }
    save_trace(p, trace, "index-root");
}
fn premises(family: &str, mutation: usize) -> &str {
    match (family,mutation) {
    ("index-coverage",1)=>"length bound deliberately erased; actual source-shaped signed index guard holds; no fit oracle",
    ("index-coverage",2)=>"actual full-width length is within capacity; source guard deliberately erased; no fit oracle",
    ("empty-no-index",1)=>"zero-capacity length bound deliberately erased",
    _=>match family {
    "length-sign"|"guard-equivalence"=>"actual full-width length is within the selected capacity",
    "index-coverage"=>"actual full-width length is within capacity and actual source-shaped signed index guard holds; no fit oracle",
    _=>"actual full-width length is within zero capacity"
}}
}
pub(super) fn emit(base: Certificate, output: &Path, capacity: usize) {
    let mut cases = vec![];
    for family in [
        "length-sign",
        "guard-equivalence",
        "index-coverage",
        "empty-no-index",
    ] {
        if family == "index-coverage" && capacity == 0
            || family == "empty-no-index" && capacity != 0
        {
            continue;
        }
        for mutation in 0..if family == "index-coverage" { 3 } else { 2 } {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let l = v("length");
            let i = v("index");
            let bad_bound =
                mutation == 1 && (family == "index-coverage" || family == "empty-no-index");
            let mut args = vec![
                ("length".into(), word_ty()),
                ("index".into(), word_ty()),
                (
                    "bound".into(),
                    if bad_bound {
                        eq(literal(true), literal(true))
                    } else {
                        eq(bound_value(l.clone(), capacity), literal(true))
                    },
                ),
            ];
            let common = vec![l.clone(), i.clone(), v("bound")];
            let mut e = match family {
                "length-sign" => typed(
                    bool_ty(),
                    sign_bit(WORD, l.clone()),
                    literal(false),
                    apply(c(&name("IndexLengthNonnegative")), common),
                ),
                "guard-equivalence" => typed(
                    bool_ty(),
                    source_guard(l.clone(), i.clone()),
                    active(i.clone(), l.clone()),
                    apply(c(&name("IndexGuardExact")), common),
                ),
                "empty-no-index" => typed(
                    bool_ty(),
                    source_guard(l.clone(), i.clone()),
                    literal(false),
                    apply(c(&name("IndexEmptyNoIndex")), common),
                ),
                _ => {
                    args.push((
                        "safe".into(),
                        if mutation == 2 {
                            eq(literal(true), literal(true))
                        } else {
                            eq(source_guard(l.clone(), i.clone()), literal(true))
                        },
                    ));
                    let mut call = common;
                    call.push(v("safe"));
                    typed(
                        word_ty(),
                        canon(WORD, i.clone()),
                        canon(
                            WORD,
                            index_word(&low_bits(i.clone(), capacity.ilog2() as usize)),
                        ),
                        apply(c(&name("IndexCoverage")), call),
                    )
                }
            };
            if mutation == 1 {
                if family == "length-sign" {
                    e.right = literal(true);
                } else if family == "guard-equivalence" {
                    e.right = not(e.right);
                }
            }
            let suffix = format!("IndexBoundClaim{}", family.replace('-', ""));
            let ty = eq_ty(e.ty.clone(), e.left, e.right);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let label = if mutation == 0 {
                "valid"
            } else if family == "index-coverage" || family == "empty-no-index" {
                if mutation == 1 {
                    "erased-bound"
                } else {
                    "erased-guard"
                }
            } else {
                "altered-result"
            };
            let id = format!("index-bound-{capacity}-{family}-{label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":capacity,"capacity":capacity,"source_word_bits":64,"full_index_bits":64,"full_length_bits":64,"low_index_bits":if capacity==0 {0} else {capacity.ilog2()},"premises":premises(family,mutation),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"private full-width index coverage and source-shaped signed guard; production Go source/operator binding and remaining string/slice laws stay pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.index_bound_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
