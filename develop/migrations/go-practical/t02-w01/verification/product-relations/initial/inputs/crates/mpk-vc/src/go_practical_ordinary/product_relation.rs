//! Products of actual field-relation witnesses and reflected comparisons.
//! Field equivalence need not be equality of carrier representations.
use super::syntax::*;
use super::{Builder, Result};

const PREFIX: &str = "Mpk.Go.Practical.Product.Relation";
fn rc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn product(a: Expr, b: Expr) -> Expr {
    pc("Type", vec![a, b])
}
fn projection(a: Expr, b: Expr, p: Expr, right: bool) -> Expr {
    pc(if right { "Right" } else { "Left" }, vec![a, b, p])
}
fn relation(a: Expr) -> Expr {
    pi("_x", a.clone(), pi("_y", a, Expr::Sort0))
}
fn comparator(a: Expr) -> Expr {
    pi("_x", a.clone(), pi("_y", a, boolean()))
}
fn holds(r: Expr, x: Expr, y: Expr) -> Expr {
    app(r, vec![x, y])
}
fn truth(p: Expr) -> Expr {
    eq(boolean(), p, bit(true))
}
fn fields(a: Expr, b: Expr, r: Expr, s: Expr, p: Expr, q: Expr) -> (Expr, Expr) {
    (
        holds(
            r,
            projection(a.clone(), b.clone(), p.clone(), false),
            projection(a.clone(), b.clone(), q.clone(), false),
        ),
        holds(
            s,
            projection(a.clone(), b.clone(), p, true),
            projection(a, b, q, true),
        ),
    )
}
fn related(a: Expr, b: Expr, r: Expr, s: Expr, p: Expr, q: Expr) -> Expr {
    rc("Related", vec![a, b, r, s, p, q])
}
fn input() -> Vec<Binder> {
    vec![
        ("A", Expr::Sort0),
        ("B", Expr::Sort0),
        ("R", relation(var("A"))),
        ("S", relation(var("B"))),
        ("p", product(var("A"), var("B"))),
        ("q", product(var("A"), var("B"))),
    ]
}
fn parameters(p: Expr, q: Expr) -> Vec<Expr> {
    vec![var("A"), var("B"), var("R"), var("S"), p, q]
}
fn field(right: bool, value: Expr) -> Expr {
    projection(var("A"), var("B"), value, right)
}
fn witness(right: bool, p: Expr, q: Expr, same: Expr) -> Expr {
    let mut args = parameters(p, q);
    args.push(same);
    rc(if right { "RelatedRight" } else { "RelatedLeft" }, args)
}
fn make_witness(p: Expr, q: Expr, l: Expr, r: Expr) -> Expr {
    let mut args = parameters(p, q);
    args.extend([l, r]);
    rc("RelatedMake", args)
}

struct Relations<'a>(&'a mut Builder);
impl Relations<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, body, theorem)
    }
    fn witnesses(&mut self) -> Result<()> {
        let args = input();
        let p = var("p");
        let q = var("q");
        let (l, r) = fields(var("A"), var("B"), var("R"), var("S"), p.clone(), q.clone());
        let claim = related(var("A"), var("B"), var("R"), var("S"), p.clone(), q.clone());
        self.emit(
            "Related",
            &args,
            Expr::Sort0,
            product(l.clone(), r.clone()),
            false,
        )?;
        let mut make = args.clone();
        make.extend([("left", l.clone()), ("right", r.clone())]);
        self.emit(
            "RelatedMake",
            &make,
            claim.clone(),
            pc(
                "Make",
                vec![l.clone(), r.clone(), var("left"), var("right")],
            ),
            true,
        )?;
        let mut observed = args.clone();
        observed.push(("same", claim));
        for right in [false, true] {
            self.emit(
                if right { "RelatedRight" } else { "RelatedLeft" },
                &observed,
                if right { r.clone() } else { l.clone() },
                projection(l.clone(), r.clone(), var("same"), right),
                true,
            )?;
        }
        let mut reflexive = args[..5].to_vec();
        for (name, ty, rel) in [("reflA", var("A"), var("R")), ("reflB", var("B"), var("S"))] {
            reflexive.push((name, pi("_x", ty, holds(rel, var("_x"), var("_x")))));
        }
        self.emit(
            "Reflexive",
            &reflexive,
            related(var("A"), var("B"), var("R"), var("S"), p.clone(), p.clone()),
            make_witness(
                p.clone(),
                p.clone(),
                app(var("reflA"), vec![field(false, p.clone())]),
                app(var("reflB"), vec![field(true, p.clone())]),
            ),
            true,
        )?;
        let mut symmetric = observed.clone();
        for (name, ty, rel) in [("symmA", var("A"), var("R")), ("symmB", var("B"), var("S"))] {
            symmetric.push((
                name,
                pi(
                    "_x",
                    ty.clone(),
                    pi(
                        "_y",
                        ty,
                        pi(
                            "_xy",
                            holds(rel.clone(), var("_x"), var("_y")),
                            holds(rel, var("_y"), var("_x")),
                        ),
                    ),
                ),
            ));
        }
        let reversed = |right, name| {
            app(
                var(name),
                vec![
                    field(right, p.clone()),
                    field(right, q.clone()),
                    witness(right, p.clone(), q.clone(), var("same")),
                ],
            )
        };
        self.emit(
            "Symmetric",
            &symmetric,
            related(var("A"), var("B"), var("R"), var("S"), q.clone(), p.clone()),
            make_witness(
                q.clone(),
                p.clone(),
                reversed(false, "symmA"),
                reversed(true, "symmB"),
            ),
            true,
        )?;
        let mut transitive = observed.clone();
        transitive.extend([
            ("z", product(var("A"), var("B"))),
            (
                "next",
                related(var("A"), var("B"), var("R"), var("S"), q.clone(), var("z")),
            ),
        ]);
        for (name, ty, rel) in [
            ("transA", var("A"), var("R")),
            ("transB", var("B"), var("S")),
        ] {
            transitive.push((
                name,
                pi(
                    "_x",
                    ty.clone(),
                    pi(
                        "_y",
                        ty.clone(),
                        pi(
                            "_z",
                            ty,
                            pi(
                                "_xy",
                                holds(rel.clone(), var("_x"), var("_y")),
                                pi(
                                    "_yz",
                                    holds(rel.clone(), var("_y"), var("_z")),
                                    holds(rel, var("_x"), var("_z")),
                                ),
                            ),
                        ),
                    ),
                ),
            ));
        }
        let composed = |right, name| {
            app(
                var(name),
                vec![
                    field(right, p.clone()),
                    field(right, q.clone()),
                    field(right, var("z")),
                    witness(right, p.clone(), q.clone(), var("same")),
                    witness(right, q.clone(), var("z"), var("next")),
                ],
            )
        };
        self.emit(
            "Transitive",
            &transitive,
            related(var("A"), var("B"), var("R"), var("S"), p.clone(), var("z")),
            make_witness(
                p.clone(),
                var("z"),
                composed(false, "transA"),
                composed(true, "transB"),
            ),
            true,
        )?;
        for right in [false, true] {
            let ty = if right { var("B") } else { var("A") };
            let mut updated = observed.clone();
            updated.extend([
                ("x", ty.clone()),
                ("y", ty),
                (
                    "values",
                    holds(if right { var("S") } else { var("R") }, var("x"), var("y")),
                ),
            ]);
            let update = |p, value| {
                pc(
                    if right { "SetRight" } else { "SetLeft" },
                    vec![var("A"), var("B"), p, value],
                )
            };
            let new_p = update(p.clone(), var("x"));
            let new_q = update(q.clone(), var("y"));
            let retained = witness(!right, p.clone(), q.clone(), var("same"));
            self.emit(
                if right {
                    "SetRightRelated"
                } else {
                    "SetLeftRelated"
                },
                &updated,
                related(
                    var("A"),
                    var("B"),
                    var("R"),
                    var("S"),
                    new_p.clone(),
                    new_q.clone(),
                ),
                make_witness(
                    new_p,
                    new_q,
                    if right {
                        retained.clone()
                    } else {
                        var("values")
                    },
                    if right { var("values") } else { retained },
                ),
                true,
            )?;
        }
        let mut mapped = observed;
        mapped.extend([
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
            ("T", relation(var("C"))),
            ("U", relation(var("D"))),
            ("f", pi("_x", var("A"), var("C"))),
            ("g", pi("_x", var("B"), var("D"))),
        ]);
        for (name, ty, rel, output, fun) in [
            ("preservesA", var("A"), var("R"), var("T"), var("f")),
            ("preservesB", var("B"), var("S"), var("U"), var("g")),
        ] {
            mapped.push((
                name,
                pi(
                    "_x",
                    ty.clone(),
                    pi(
                        "_y",
                        ty,
                        pi(
                            "_xy",
                            holds(rel, var("_x"), var("_y")),
                            holds(
                                output,
                                app(fun.clone(), vec![var("_x")]),
                                app(fun, vec![var("_y")]),
                            ),
                        ),
                    ),
                ),
            ));
        }
        let map = |value| {
            pc(
                "Map",
                vec![
                    var("A"),
                    var("B"),
                    var("C"),
                    var("D"),
                    var("f"),
                    var("g"),
                    value,
                ],
            )
        };
        let new_p = map(p.clone());
        let new_q = map(q.clone());
        let preserved = |right, name| {
            app(
                var(name),
                vec![
                    field(right, p.clone()),
                    field(right, q.clone()),
                    witness(right, p.clone(), q.clone(), var("same")),
                ],
            )
        };
        self.emit(
            "MapRelated",
            &mapped,
            related(
                var("C"),
                var("D"),
                var("T"),
                var("U"),
                new_p.clone(),
                new_q.clone(),
            ),
            rc(
                "RelatedMake",
                vec![
                    var("C"),
                    var("D"),
                    var("T"),
                    var("U"),
                    new_p,
                    new_q,
                    preserved(false, "preservesA"),
                    preserved(true, "preservesB"),
                ],
            ),
            true,
        )
    }
    fn boolean_truth(&mut self) -> Result<()> {
        for right in [false, true] {
            let proposition = |env: &std::collections::BTreeMap<_, _>| {
                pi(
                    "proof",
                    truth(bc("And", vec![env["a"].clone(), env["b"].clone()])),
                    truth(env[if right { "b" } else { "a" }].clone()),
                )
            };
            let proof = super::word_proofs::boolean_cases(
                &["a", "b"],
                &std::collections::BTreeMap::new(),
                &proposition,
                &mut |env| {
                    let claim = truth(env[if right { "b" } else { "a" }].clone());
                    let value = if env["a"] == bit(true) && env["b"] == bit(true) {
                        refl(boolean(), bit(true))
                    } else {
                        call("Mpk.Go.Practical.Bool.FalseElim", vec![claim, var("proof")])
                    };
                    Ok(lam(
                        "proof",
                        truth(bc("And", vec![env["a"].clone(), env["b"].clone()])),
                        value,
                    ))
                },
            )?;
            self.emit(
                if right { "AndTrueRight" } else { "AndTrueLeft" },
                &[],
                pi(
                    "a",
                    boolean(),
                    pi(
                        "b",
                        boolean(),
                        pi(
                            "proof",
                            truth(bc("And", vec![var("a"), var("b")])),
                            truth(var(if right { "b" } else { "a" })),
                        ),
                    ),
                ),
                proof,
                true,
            )?;
        }
        let a = var("a");
        let b = var("b");
        let first = Equality::boolean(a.clone(), bit(true), var("left")).mapped(
            lam("_v", boolean(), bc("And", vec![var("_v"), b.clone()])),
            bc("And", vec![a.clone(), b.clone()]),
            bc("And", vec![bit(true), b.clone()]),
            boolean(),
        );
        let second = Equality::boolean(b.clone(), bit(true), var("right")).mapped(
            lam("_v", boolean(), bc("And", vec![bit(true), var("_v")])),
            bc("And", vec![bit(true), b.clone()]),
            bc("And", vec![bit(true), bit(true)]),
            boolean(),
        );
        self.emit(
            "AndTrueIntro",
            &[
                ("a", boolean()),
                ("b", boolean()),
                ("left", truth(a.clone())),
                ("right", truth(b.clone())),
            ],
            truth(bc("And", vec![a, b])),
            first.then(second)?.proof,
            true,
        )
    }
    fn comparisons(&mut self) -> Result<()> {
        let mut args = input();
        args.extend([
            ("cmpA", comparator(var("A"))),
            ("cmpB", comparator(var("B"))),
        ]);
        let left = holds(var("cmpA"), field(false, var("p")), field(false, var("q")));
        let right = holds(var("cmpB"), field(true, var("p")), field(true, var("q")));
        // Comparators do not depend on the field relations.
        let compare_args = args
            .iter()
            .filter(|(name, _)| !["R", "S"].contains(name))
            .cloned()
            .collect::<Vec<_>>();
        self.emit(
            "EqualBy",
            &compare_args,
            boolean(),
            bc("And", vec![left.clone(), right.clone()]),
            false,
        )?;
        let compared = rc(
            "EqualBy",
            vec![
                var("A"),
                var("B"),
                var("p"),
                var("q"),
                var("cmpA"),
                var("cmpB"),
            ],
        );
        let same = related(var("A"), var("B"), var("R"), var("S"), var("p"), var("q"));
        for reflect in [true, false] {
            let mut proof_args = args.clone();
            for (name, ty, rel, cmp) in [
                ("reflectA", var("A"), var("R"), var("cmpA")),
                ("reflectB", var("B"), var("S"), var("cmpB")),
            ] {
                let witness = holds(rel, var("_x"), var("_y"));
                let boolean = truth(holds(cmp, var("_x"), var("_y")));
                let (premise, result) = if reflect {
                    (boolean, witness)
                } else {
                    (witness, boolean)
                };
                proof_args.push((
                    name,
                    pi(
                        "_x",
                        ty.clone(),
                        pi("_y", ty, pi("_proof", premise, result)),
                    ),
                ));
            }
            proof_args.push((
                "proof",
                if reflect {
                    truth(compared.clone())
                } else {
                    same.clone()
                },
            ));
            let projected = |right_side| {
                if reflect {
                    rc(
                        if right_side {
                            "AndTrueRight"
                        } else {
                            "AndTrueLeft"
                        },
                        vec![left.clone(), right.clone(), var("proof")],
                    )
                } else {
                    witness(right_side, var("p"), var("q"), var("proof"))
                }
            };
            let first = app(
                var("reflectA"),
                vec![
                    field(false, var("p")),
                    field(false, var("q")),
                    projected(false),
                ],
            );
            let second = app(
                var("reflectB"),
                vec![
                    field(true, var("p")),
                    field(true, var("q")),
                    projected(true),
                ],
            );
            self.emit(
                if reflect {
                    "EqualByReflect"
                } else {
                    "EqualByComplete"
                },
                &proof_args,
                if reflect {
                    same.clone()
                } else {
                    truth(compared.clone())
                },
                if reflect {
                    make_witness(var("p"), var("q"), first, second)
                } else {
                    rc(
                        "AndTrueIntro",
                        vec![left.clone(), right.clone(), first, second],
                    )
                },
                true,
            )?;
        }
        Ok(())
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut relations = Relations(builder);
    relations.witnesses()?;
    relations.boolean_truth()?;
    relations.comparisons()
}

pub(super) fn append_word_pairs(builder: &mut Builder) -> Result<()> {
    for left in [8, 16, 32, 64] {
        for right in [8, 16, 32, 64] {
            let a = super::word::word_type(left)?;
            let b = super::word::word_type(right)?;
            let numeric = |width, ty: Expr| {
                lam(
                    "_x",
                    ty.clone(),
                    lam(
                        "_y",
                        ty,
                        call(
                            "Mpk.Go.Practical.Int.Equivalent",
                            vec![
                                call(
                                    &format!("Mpk.Go.Practical.Word.Numeric.W{width}.Signed"),
                                    vec![var("_x")],
                                ),
                                call(
                                    &format!("Mpk.Go.Practical.Word.Numeric.W{width}.Signed"),
                                    vec![var("_y")],
                                ),
                            ],
                        ),
                    ),
                )
            };
            let r = numeric(left, a.clone());
            let s = numeric(right, b.clone());
            let cmp_a = constant(&format!("Mpk.Go.Practical.Word.W{left}.Equal"));
            let cmp_b = constant(&format!("Mpk.Go.Practical.Word.W{right}.Equal"));
            let compared = rc(
                "EqualBy",
                vec![
                    a.clone(),
                    b.clone(),
                    var("p"),
                    var("q"),
                    cmp_a.clone(),
                    cmp_b.clone(),
                ],
            );
            let same = related(
                a.clone(),
                b.clone(),
                r.clone(),
                s.clone(),
                var("p"),
                var("q"),
            );
            for reflect in [true, false] {
                let member = if reflect {
                    "EquivalentReflect"
                } else {
                    "EquivalentComplete"
                };
                let args = [
                    ("p", product(a.clone(), b.clone())),
                    ("q", product(a.clone(), b.clone())),
                    (
                        "proof",
                        if reflect {
                            truth(compared.clone())
                        } else {
                            same.clone()
                        },
                    ),
                ];
                let proof = rc(
                    if reflect {
                        "EqualByReflect"
                    } else {
                        "EqualByComplete"
                    },
                    vec![
                        a.clone(),
                        b.clone(),
                        r.clone(),
                        s.clone(),
                        var("p"),
                        var("q"),
                        cmp_a.clone(),
                        cmp_b.clone(),
                        constant(&format!(
                            "Mpk.Go.Practical.Word.SignedEquality.W{left}.{member}"
                        )),
                        constant(&format!(
                            "Mpk.Go.Practical.Word.SignedEquality.W{right}.{member}"
                        )),
                        var("proof"),
                    ],
                );
                emit(
                    builder,
                    &format!("Mpk.Go.Practical.Word.ProductEquality.W{left}W{right}.{member}"),
                    &args,
                    if reflect {
                        same.clone()
                    } else {
                        truth(compared.clone())
                    },
                    proof,
                    true,
                )?;
            }
        }
    }
    let integer = pi("_part", boolean(), nat());
    let integer_make =
        |positive, negative| call("Mpk.Go.Practical.Int.Make", vec![positive, negative]);
    let old_left = integer_make(var("lp"), var("ln"));
    let old_right = integer_make(var("rp"), var("rn"));
    let new_left = integer_make(add(var("lp"), var("lk")), add(var("ln"), var("lk")));
    let new_right = integer_make(add(var("rp"), var("rk")), add(var("rn"), var("rk")));
    let pair = |left, right| pc("Make", vec![integer.clone(), integer.clone(), left, right]);
    let p = pair(old_left, old_right);
    let q = pair(new_left, new_right);
    let relation = constant("Mpk.Go.Practical.Int.Equivalent");
    let comparator = constant("Mpk.Go.Practical.Int.Equal");
    let same = rc(
        "RelatedMake",
        vec![
            integer.clone(),
            integer.clone(),
            relation.clone(),
            relation.clone(),
            p.clone(),
            q.clone(),
            call(
                "Mpk.Go.Practical.Int.RepresentationShift",
                vec![var("lp"), var("ln"), var("lk")],
            ),
            call(
                "Mpk.Go.Practical.Int.RepresentationShift",
                vec![var("rp"), var("rn"), var("rk")],
            ),
        ],
    );
    emit(
        builder,
        "Mpk.Go.Practical.Int.ProductEquality.RepresentationShift",
        &[
            ("lp", nat()),
            ("ln", nat()),
            ("lk", nat()),
            ("rp", nat()),
            ("rn", nat()),
            ("rk", nat()),
        ],
        truth(rc(
            "EqualBy",
            vec![
                integer.clone(),
                integer.clone(),
                p.clone(),
                q.clone(),
                comparator.clone(),
                comparator.clone(),
            ],
        )),
        rc(
            "EqualByComplete",
            vec![
                integer.clone(),
                integer,
                relation.clone(),
                relation,
                p,
                q,
                comparator.clone(),
                comparator,
                constant("Mpk.Go.Practical.Int.EqualFromEquivalent"),
                constant("Mpk.Go.Practical.Int.EqualFromEquivalent"),
                same,
            ],
        ),
        true,
    )
}
