//! Unsigned byte ranks and real first-difference lexicographic witnesses.
//! Fixed address depth supplies structure; no operand table or source oracle.
use super::*;

mod string_values;

const POS: [&str; 8] = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"];
const QUERY: [&str; 8] = ["j0", "j1", "j2", "j3", "j4", "j5", "j6", "j7"];
fn oc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Order.{name}"), args)
}
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Equality.{name}"), args)
}
fn norm(s: Expr) -> Expr {
    call("Mpk.Go.Practical.String.Normalization.Value", vec![s])
}
fn fv(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn idx(d: usize, a: Vec<Expr>) -> Expr {
    fv(d, "Index", a)
}
fn vector(d: usize) -> Expr {
    cube(&POS[..d], nat(), false)
}
fn truth(b: Expr) -> Expr {
    eq(boolean(), b, bit(true))
}
fn pointwise(d: usize, f: Expr, g: Expr) -> Expr {
    let a = address_bits(&POS[..d]);
    cube(&POS[..d], eq(nat(), app(f, a.clone()), app(g, a)), false)
}
fn prefix(d: usize, f: Expr, g: Expr, at: Vec<Expr>) -> Expr {
    let a = address_bits(&QUERY[..d]);
    cube(
        &QUERY[..d],
        pi(
            "_before",
            truth(less(idx(d, a.clone()), idx(d, at))),
            eq(nat(), app(f, a.clone()), app(g, a)),
        ),
        false,
    )
}
fn handler(d: usize, f: Expr, g: Expr, result: Expr) -> Expr {
    let at = address_bits(&POS[..d]);
    cube(
        &POS[..d],
        pi(
            "_prior",
            prefix(d, f.clone(), g.clone(), at.clone()),
            pi(
                "_strict",
                truth(less(app(f, at.clone()), app(g, at))),
                result,
            ),
        ),
        false,
    )
}
fn deq(d: usize, f: Expr, g: Expr) -> Expr {
    oc(&format!("D{d}.Equal"), vec![f, g])
}
fn dlt(d: usize, f: Expr, g: Expr) -> Expr {
    oc(&format!("D{d}.Less"), vec![f, g])
}
fn half(f: Expr, high: bool) -> Expr {
    app(f, vec![bit(high)])
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (a, z) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_flag", boolean(), bc(name, vec![var("_flag"), b])),
        a,
        z,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_flag", boolean(), bc(name, vec![a, var("_flag")])),
        l,
        r,
        boolean(),
    )
}
fn neg(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_flag", boolean(), bc("Not", vec![var("_flag")])),
        l,
        r,
        boolean(),
    )
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(lam("_n", nat(), less(var("_n"), b)), l, r, boolean())
}
fn less_right(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(lam("_n", nat(), less(a, var("_n"))), l, r, boolean())
}
fn numeric_byte(x: Expr) -> Expr {
    numeric("Value3", vec![x])
}
fn rank(s: Expr, a: Vec<Expr>) -> Expr {
    let mut args = vec![s];
    args.extend(a);
    oc("Rank", args)
}
fn ranks(s: Expr) -> Expr {
    oc("Ranks", vec![s])
}
fn decide(flag: Expr, result: Expr, low: Expr, high: Expr) -> Expr {
    app(
        call(
            "Std.Bool.cases",
            vec![
                lam(
                    "_chosen",
                    boolean(),
                    pi(
                        "_branch",
                        eq(boolean(), flag.clone(), var("_chosen")),
                        result.clone(),
                    ),
                ),
                lam("_branch", eq(boolean(), flag.clone(), bit(false)), low),
                lam("_branch", eq(boolean(), flag.clone(), bit(true)), high),
                flag.clone(),
            ],
        ),
        vec![refl(boolean(), flag)],
    )
}
fn comparison(d: usize, name: &str, a: Vec<Expr>, b: Vec<Expr>, high: bool) -> Equality {
    let mut x = vec![bit(high)];
    x.extend(a.clone());
    let mut y = vec![bit(high)];
    y.extend(b.clone());
    let mut args = a.clone();
    args.extend(b.clone());
    Equality::boolean(
        less(idx(d, x), idx(d, y)),
        less(idx(d - 1, a), idx(d - 1, b)),
        oc(&format!("D{d}.{name}Compare"), args),
    )
}
fn first_type(d: usize, f: Expr, g: Expr) -> Expr {
    pi(
        "R",
        Expr::Sort0,
        pi("on_first", handler(d, f, g, var("R")), var("R")),
    )
}
fn pair_step(d: usize, op: &str, f: Expr, g: Expr) -> Expr {
    if d == 0 {
        return if op == "Equal" {
            equal(f, g)
        } else {
            less(f, g)
        };
    }
    let lo = if op == "Equal" {
        deq(d - 1, half(f.clone(), false), half(g.clone(), false))
    } else {
        dlt(d - 1, half(f.clone(), false), half(g.clone(), false))
    };
    let hi = if op == "Equal" {
        deq(d - 1, half(f, true), half(g, true))
    } else {
        bc(
            "And",
            vec![
                deq(d - 1, half(f.clone(), false), half(g.clone(), false)),
                dlt(d - 1, half(f, true), half(g, true)),
            ],
        )
    };
    bc(if op == "Equal" { "And" } else { "Or" }, vec![lo, hi])
}

struct Order<'a>(&'a mut Builder);
impl Order<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Order.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn boolean(&mut self) -> Result<()> {
        for name in ["OrLeftTrue", "OrRightTrue", "OrFalseLeft"] {
            let statement = |a: Expr, b: Expr| {
                if name == "OrFalseLeft" {
                    pi(
                        "_false",
                        eq(boolean(), a.clone(), bit(false)),
                        pi("_joined", truth(bc("Or", vec![a, b.clone()])), truth(b)),
                    )
                } else {
                    pi(
                        "_yes",
                        truth(if name == "OrLeftTrue" {
                            a.clone()
                        } else {
                            b.clone()
                        }),
                        truth(bc("Or", vec![a, b])),
                    )
                }
            };
            let leaf = |a: bool, b: bool| {
                if name == "OrFalseLeft" {
                    let proof = if b {
                        refl(boolean(), bit(true))
                    } else if !a {
                        var("_joined")
                    } else {
                        bc(
                            "FalseElim",
                            vec![
                                truth(bit(b)),
                                call(
                                    "Std.Eq.symm",
                                    vec![boolean(), bit(true), bit(false), var("_false")],
                                ),
                            ],
                        )
                    };
                    lam(
                        "_false",
                        eq(boolean(), bit(a), bit(false)),
                        lam("_joined", truth(bc("Or", vec![bit(a), bit(b)])), proof),
                    )
                } else {
                    let premise = if name == "OrLeftTrue" { a } else { b };
                    let proof = if a || b {
                        refl(boolean(), bit(true))
                    } else {
                        bc(
                            "FalseElim",
                            vec![truth(bc("Or", vec![bit(a), bit(b)])), var("_yes")],
                        )
                    };
                    lam("_yes", truth(bit(premise)), proof)
                }
            };
            let branch = |a: bool| {
                lam(
                    "b",
                    boolean(),
                    call(
                        "Std.Bool.cases",
                        vec![
                            lam("b", boolean(), statement(bit(a), var("b"))),
                            leaf(a, false),
                            leaf(a, true),
                            var("b"),
                        ],
                    ),
                )
            };
            self.emit(
                name,
                &[("a", boolean()), ("b", boolean())],
                statement(var("a"), var("b")),
                app(
                    call(
                        "Std.Bool.cases",
                        vec![
                            lam(
                                "a",
                                boolean(),
                                pi("b", boolean(), statement(var("a"), var("b"))),
                            ),
                            branch(false),
                            branch(true),
                            var("a"),
                        ],
                    ),
                    vec![var("b")],
                ),
                true,
            )?;
        }
        Ok(())
    }
    fn vectors(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let args = [("f", vector(d)), ("g", vector(d))];
        for op in ["Equal", "Less"] {
            let body = pair_step(d, op, f.clone(), g.clone());
            let value = oc(&format!("D{d}.{op}"), vec![f.clone(), g.clone()]);
            self.emit(&format!("D{d}.{op}"), &args, boolean(), body.clone(), false)?;
            self.theorem(
                &format!("D{d}.{op}Step"),
                &args,
                Equality::boolean(value, body.clone(), refl(boolean(), body)),
            )?;
        }
        let whole = deq(d, f.clone(), g.clone());
        let body = pair_step(d, "Equal", f.clone(), g.clone());
        let h = var("h");
        let reflect = if d == 0 {
            nc("EqualReflect", vec![f.clone(), g.clone(), h.clone()])
        } else {
            let e = Equality::boolean(
                whole.clone(),
                body.clone(),
                oc(&format!("D{d}.EqualStep"), vec![f.clone(), g.clone()]),
            )
            .reverse()
            .then(Equality::boolean(whole.clone(), bit(true), h.clone()))?;
            let low = deq(d - 1, half(f.clone(), false), half(g.clone(), false));
            let high = deq(d - 1, half(f.clone(), true), half(g.clone(), true));
            let branch = |yes| {
                oc(
                    &format!("D{}.EqualReflect", d - 1),
                    vec![
                        half(f.clone(), yes),
                        half(g.clone(), yes),
                        ec(
                            if yes { "AndRight" } else { "AndLeft" },
                            vec![low.clone(), high.clone(), e.proof.clone()],
                        ),
                    ],
                )
            };
            lam(
                POS[0],
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(
                            POS[0],
                            boolean(),
                            cube(
                                &POS[1..d],
                                eq(
                                    nat(),
                                    app(f.clone(), address_bits(&POS[..d])),
                                    app(g.clone(), address_bits(&POS[..d])),
                                ),
                                false,
                            ),
                        ),
                        branch(false),
                        branch(true),
                        var(POS[0]),
                    ],
                ),
            )
        };
        self.emit(
            &format!("D{d}.EqualReflect"),
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", truth(whole.clone())),
            ],
            pointwise(d, f.clone(), g.clone()),
            reflect,
            true,
        )?;
        let complete = if d == 0 {
            nc("EqualComplete", vec![f.clone(), g.clone(), h.clone()])
        } else {
            let low = deq(d - 1, half(f.clone(), false), half(g.clone(), false));
            let high = deq(d - 1, half(f.clone(), true), half(g.clone(), true));
            Equality::boolean(
                whole.clone(),
                body,
                oc(&format!("D{d}.EqualStep"), vec![f.clone(), g.clone()]),
            )
            .then(Equality::boolean(
                bc("And", vec![low.clone(), high.clone()]),
                bit(true),
                ec(
                    "AndComplete",
                    vec![
                        low,
                        high,
                        oc(
                            &format!("D{}.EqualComplete", d - 1),
                            vec![
                                half(f.clone(), false),
                                half(g.clone(), false),
                                half(h.clone(), false),
                            ],
                        ),
                        oc(
                            &format!("D{}.EqualComplete", d - 1),
                            vec![half(f.clone(), true), half(g.clone(), true), half(h, true)],
                        ),
                    ],
                ),
            ))?
            .proof
        };
        self.emit(
            &format!("D{d}.EqualComplete"),
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", pointwise(d, f.clone(), g.clone())),
            ],
            truth(whole),
            complete,
            true,
        )?;
        self.vector_respects(d)?;
        self.first_difference(d)
    }
    fn vector_respects(&mut self, d: usize) -> Result<()> {
        let (f, g, f2, g2) = (var("f"), var("g"), var("f2"), var("g2"));
        let args = [
            ("f", vector(d)),
            ("g", vector(d)),
            ("f2", vector(d)),
            ("g2", vector(d)),
            ("left", pointwise(d, f.clone(), f2.clone())),
            ("right", pointwise(d, g.clone(), g2.clone())),
        ];
        for op in ["Equal", "Less"] {
            let e = if d == 0 {
                let a = Equality::natural(f.clone(), f2.clone(), var("left"));
                let b = Equality::natural(g.clone(), g2.clone(), var("right"));
                let name = if op == "Equal" { "Equal" } else { "Less" };
                let (start, middle, end) = (
                    nc(name, vec![f.clone(), g.clone()]),
                    nc(name, vec![f2.clone(), g.clone()]),
                    nc(name, vec![f2.clone(), g2.clone()]),
                );
                a.mapped(
                    lam("_n", nat(), nc(name, vec![var("_n"), g.clone()])),
                    start,
                    middle.clone(),
                    boolean(),
                )
                .then(b.mapped(
                    lam("_n", nat(), nc(name, vec![f2.clone(), var("_n")])),
                    middle,
                    end,
                    boolean(),
                ))?
            } else {
                let e = |op: &str, high| {
                    Equality::boolean(
                        oc(
                            &format!("D{}.{op}", d - 1),
                            vec![half(f.clone(), high), half(g.clone(), high)],
                        ),
                        oc(
                            &format!("D{}.{op}", d - 1),
                            vec![half(f2.clone(), high), half(g2.clone(), high)],
                        ),
                        oc(
                            &format!("D{}.{op}Respects", d - 1),
                            vec![
                                half(f.clone(), high),
                                half(g.clone(), high),
                                half(f2.clone(), high),
                                half(g2.clone(), high),
                                half(var("left"), high),
                                half(var("right"), high),
                            ],
                        ),
                    )
                };
                if op == "Equal" {
                    let a = e("Equal", false);
                    let b = e("Equal", true);
                    bool_left("And", a, b.left.clone()).then(bool_right(
                        "And",
                        deq(d - 1, half(f2.clone(), false), half(g2.clone(), false)),
                        b,
                    ))?
                } else {
                    let a = e("Less", false);
                    let b = e("Equal", false);
                    let c = e("Less", true);
                    let rhs = bool_left("And", b, c.left.clone()).then(bool_right(
                        "And",
                        deq(d - 1, half(f2.clone(), false), half(g2.clone(), false)),
                        c,
                    ))?;
                    bool_left("Or", a, rhs.left.clone()).then(bool_right(
                        "Or",
                        dlt(d - 1, half(f2.clone(), false), half(g2.clone(), false)),
                        rhs,
                    ))?
                }
            };
            let e = Equality::boolean(
                oc(&format!("D{d}.{op}"), vec![f.clone(), g.clone()]),
                pair_step(d, op, f.clone(), g.clone()),
                oc(&format!("D{d}.{op}Step"), vec![f.clone(), g.clone()]),
            )
            .then(e)?
            .then(
                Equality::boolean(
                    oc(&format!("D{d}.{op}"), vec![f2.clone(), g2.clone()]),
                    pair_step(d, op, f2.clone(), g2.clone()),
                    oc(&format!("D{d}.{op}Step"), vec![f2.clone(), g2.clone()]),
                )
                .reverse(),
            )?;
            self.theorem(&format!("D{d}.{op}Respects"), &args, e)?;
        }
        Ok(())
    }
    fn lifted_prefix(
        &self,
        d: usize,
        f: Expr,
        g: Expr,
        at: Vec<Expr>,
        high: bool,
        prior: Expr,
        low_equal: Option<Expr>,
    ) -> Result<Expr> {
        let h = d - 1;
        let q = address_bits(&QUERY[1..d]);
        let mut witness = vec![bit(high)];
        witness.extend(at.clone());
        let branch = |query_high: bool| -> Result<Expr> {
            let mut query = vec![bit(query_high)];
            query.extend(q.clone());
            let ty = truth(less(idx(d, query.clone()), idx(d, witness.clone())));
            let result = eq(nat(), app(f.clone(), query.clone()), app(g.clone(), query));
            let proof = if query_high == high {
                let e = comparison(
                    d,
                    if high { "High" } else { "Low" },
                    q.clone(),
                    at.clone(),
                    high,
                )
                .reverse()
                .then(Equality::boolean(
                    less(
                        idx(d, {
                            let mut x = vec![bit(high)];
                            x.extend(q.clone());
                            x
                        }),
                        idx(d, witness.clone()),
                    ),
                    bit(true),
                    var("_before"),
                ))?;
                let mut args = q.clone();
                args.push(e.proof);
                app(prior.clone(), args)
            } else if high {
                let mut args = vec![
                    half(f.clone(), false),
                    half(g.clone(), false),
                    low_equal
                        .clone()
                        .expect("high prefix has real low equality"),
                ];
                args.extend(q.clone());
                oc(&format!("D{h}.EqualReflect"), args)
            } else {
                let mut args = at.clone();
                args.extend(q.clone());
                let mut before = vec![bit(true)];
                before.extend(q.clone());
                let contradiction = Equality::boolean(
                    less(idx(d, before), idx(d, witness.clone())),
                    bit(false),
                    oc(&format!("D{d}.ReverseCompare"), args),
                )
                .reverse()
                .then(Equality::boolean(
                    less(
                        idx(d, {
                            let mut x = vec![bit(true)];
                            x.extend(q.clone());
                            x
                        }),
                        idx(d, witness.clone()),
                    ),
                    bit(true),
                    var("_before"),
                ))?;
                bc("FalseElim", vec![result, contradiction.proof])
            };
            Ok(cube(&QUERY[1..d], lam("_before", ty, proof), true))
        };
        let mut query = vec![var(QUERY[0])];
        query.extend(q.clone());
        Ok(lam(
            QUERY[0],
            boolean(),
            call(
                "Std.Bool.cases",
                vec![
                    lam(
                        QUERY[0],
                        boolean(),
                        cube(
                            &QUERY[1..d],
                            pi(
                                "_before",
                                truth(less(idx(d, query.clone()), idx(d, witness.clone()))),
                                eq(nat(), app(f.clone(), query.clone()), app(g.clone(), query)),
                            ),
                            false,
                        ),
                    ),
                    branch(false)?,
                    branch(true)?,
                    var(QUERY[0]),
                ],
            ),
        ))
    }
    fn first_difference(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let lt = dlt(d, f.clone(), g.clone());
        let reflect = if d == 0 {
            let contradiction = Equality::boolean(
                less(idx(0, vec![]), idx(0, vec![])),
                bit(false),
                nc("LessIrreflexive", vec![idx(0, vec![])]),
            )
            .reverse()
            .then(Equality::boolean(
                less(idx(0, vec![]), idx(0, vec![])),
                bit(true),
                var("_before"),
            ))?;
            let prior = lam(
                "_before",
                truth(less(idx(0, vec![]), idx(0, vec![]))),
                bc(
                    "FalseElim",
                    vec![eq(nat(), f.clone(), g.clone()), contradiction.proof],
                ),
            );
            let strict = Equality::boolean(
                lt.clone(),
                less(f.clone(), g.clone()),
                oc("D0.LessStep", vec![f.clone(), g.clone()]),
            )
            .reverse()
            .then(Equality::boolean(lt.clone(), bit(true), var("h")))?;
            app(var("on_first"), vec![prior, strict.proof])
        } else {
            let h = d - 1;
            let a = dlt(h, half(f.clone(), false), half(g.clone(), false));
            let e = deq(h, half(f.clone(), false), half(g.clone(), false));
            let b = dlt(h, half(f.clone(), true), half(g.clone(), true));
            let joined = bc("And", vec![e.clone(), b.clone()]);
            let total = Equality::boolean(
                lt.clone(),
                pair_step(d, "Less", f.clone(), g.clone()),
                oc(&format!("D{d}.LessStep"), vec![f.clone(), g.clone()]),
            )
            .reverse()
            .then(Equality::boolean(lt.clone(), bit(true), var("h")))?;
            let and_true = oc(
                "OrFalseLeft",
                vec![a.clone(), joined.clone(), var("_branch"), total.proof],
            );
            let low_equal = ec("AndLeft", vec![e.clone(), b.clone(), and_true.clone()]);
            let high_less = ec("AndRight", vec![e, b, and_true]);
            let lift = |high, low_equal| -> Result<Expr> {
                let at = address_bits(&POS[..h]);
                let strict = truth(less(
                    app(half(f.clone(), high), at.clone()),
                    app(half(g.clone(), high), at.clone()),
                ));
                let global = self.lifted_prefix(
                    d,
                    f.clone(),
                    g.clone(),
                    at.clone(),
                    high,
                    var("_prior"),
                    low_equal,
                )?;
                let mut args = vec![bit(high)];
                args.extend(at.clone());
                args.extend([global, var("_strict")]);
                Ok(cube(
                    &POS[..h],
                    lam(
                        "_prior",
                        prefix(h, half(f.clone(), high), half(g.clone(), high), at),
                        lam("_strict", strict, app(var("on_first"), args)),
                    ),
                    true,
                ))
            };
            let low = oc(
                &format!("D{h}.LessReflect"),
                vec![
                    half(f.clone(), false),
                    half(g.clone(), false),
                    var("_branch"),
                    var("R"),
                    lift(false, None)?,
                ],
            );
            let high = oc(
                &format!("D{h}.LessReflect"),
                vec![
                    half(f.clone(), true),
                    half(g.clone(), true),
                    high_less,
                    var("R"),
                    lift(true, Some(low_equal))?,
                ],
            );
            decide(a, var("R"), high, low)
        };
        self.emit(
            &format!("D{d}.LessReflect"),
            &[("f", vector(d)), ("g", vector(d)), ("h", truth(lt.clone()))],
            first_type(d, f.clone(), g.clone()),
            lam(
                "R",
                Expr::Sort0,
                lam(
                    "on_first",
                    handler(d, f.clone(), g.clone(), var("R")),
                    reflect,
                ),
            ),
            true,
        )?;
        let at = address_bits(&POS[..d]);
        let complete = if d == 0 {
            Equality::boolean(
                lt.clone(),
                less(f.clone(), g.clone()),
                oc("D0.LessStep", vec![f.clone(), g.clone()]),
            )
            .then(Equality::boolean(
                less(f.clone(), g.clone()),
                bit(true),
                var("strict"),
            ))?
            .proof
        } else {
            let h = d - 1;
            let rest = address_bits(&POS[1..d]);
            let q = address_bits(&QUERY[..h]);
            let branch = |high: bool| -> Result<Expr> {
                let mut witness = vec![bit(high)];
                witness.extend(rest.clone());
                let fq = half(f.clone(), high);
                let gq = half(g.clone(), high);
                let comparison = comparison(
                    d,
                    if high { "High" } else { "Low" },
                    q.clone(),
                    rest.clone(),
                    high,
                )
                .then(Equality::boolean(
                    less(idx(h, q.clone()), idx(h, rest.clone())),
                    bit(true),
                    var("_before"),
                ))?;
                let mut query = vec![bit(high)];
                query.extend(q.clone());
                let mut args = query;
                args.push(comparison.proof);
                let lower_prefix = cube(
                    &QUERY[..h],
                    lam(
                        "_before",
                        truth(less(idx(h, q.clone()), idx(h, rest.clone()))),
                        app(var("prior"), args),
                    ),
                    true,
                );
                let mut args = vec![fq.clone(), gq.clone()];
                args.extend(rest.clone());
                args.extend([lower_prefix, var("strict")]);
                let lower = oc(&format!("D{h}.LessComplete"), args);
                let a = dlt(h, half(f.clone(), false), half(g.clone(), false));
                let e = deq(h, half(f.clone(), false), half(g.clone(), false));
                let b = dlt(h, half(f.clone(), true), half(g.clone(), true));
                let joined = bc("And", vec![e.clone(), b.clone()]);
                let proof = if !high {
                    oc("OrLeftTrue", vec![a.clone(), joined.clone(), lower])
                } else {
                    let mut crosses = q.clone();
                    crosses.extend(rest.clone());
                    let cross = oc(&format!("D{d}.CrossCompare"), crosses);
                    let mut query = vec![bit(false)];
                    query.extend(q.clone());
                    query.push(cross);
                    let low_points = cube(&QUERY[..h], app(var("prior"), query), true);
                    let low = oc(
                        &format!("D{h}.EqualComplete"),
                        vec![half(f.clone(), false), half(g.clone(), false), low_points],
                    );
                    oc(
                        "OrRightTrue",
                        vec![
                            a.clone(),
                            joined.clone(),
                            ec("AndComplete", vec![e, b, low, lower]),
                        ],
                    )
                };
                let body = bc("Or", vec![a, joined]);
                let mut dargs = vec![f.clone(), g.clone()];
                let e = Equality::boolean(
                    dlt(d, f.clone(), g.clone()),
                    pair_step(d, "Less", f.clone(), g.clone()),
                    oc(&format!("D{d}.LessStep"), std::mem::take(&mut dargs)),
                )
                .then(Equality::boolean(body, bit(true), proof))?;
                let mut specific = vec![bit(high)];
                specific.extend(rest.clone());
                Ok(cube(
                    &POS[1..d],
                    lam(
                        "prior",
                        prefix(d, f.clone(), g.clone(), witness),
                        lam(
                            "strict",
                            truth(less(
                                app(f.clone(), specific.clone()),
                                app(g.clone(), specific),
                            )),
                            e.proof,
                        ),
                    ),
                    true,
                ))
            };
            let output = cube(
                &POS[1..d],
                pi(
                    "prior",
                    prefix(d, f.clone(), g.clone(), at.clone()),
                    pi(
                        "strict",
                        truth(less(app(f.clone(), at.clone()), app(g.clone(), at.clone()))),
                        truth(lt.clone()),
                    ),
                ),
                false,
            );
            lam(
                POS[0],
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(POS[0], boolean(), output),
                        branch(false)?,
                        branch(true)?,
                        var(POS[0]),
                    ],
                ),
            )
        };
        if d == 0 {
            self.emit(
                "D0.LessComplete",
                &[
                    ("f", vector(0)),
                    ("g", vector(0)),
                    ("prior", prefix(0, f.clone(), g.clone(), vec![])),
                    ("strict", truth(less(f, g))),
                ],
                truth(lt),
                complete,
                true,
            )
        } else {
            self.emit(
                &format!("D{d}.LessComplete"),
                &[("f", vector(d)), ("g", vector(d))],
                {
                    let mut ty = truth(lt);
                    ty = pi(
                        "strict",
                        truth(less(app(f.clone(), at.clone()), app(g.clone(), at.clone()))),
                        ty,
                    );
                    ty = pi("prior", prefix(d, f, g, at), ty);
                    cube(&POS[..d], ty, false)
                },
                complete,
                true,
            )
        }
    }
    fn asymmetry(&mut self) -> Result<()> {
        let (a, b, n, m) = (var("a"), var("b"), var("_n"), var("_m"));
        let p = |a: Expr, b: Expr| {
            pi(
                "_h",
                truth(less(a.clone(), b.clone())),
                eq(boolean(), less(b, a), bit(false)),
            )
        };
        let base = lam(
            "b",
            nat(),
            lam(
                "_h",
                truth(less(zero(), b.clone())),
                nc("LessZero", vec![b.clone()]),
            ),
        );
        let empty = Equality::boolean(
            less(succ(n.clone()), zero()),
            bit(false),
            nc("LessZero", vec![succ(n.clone())]),
        )
        .reverse()
        .then(Equality::boolean(
            less(succ(n.clone()), zero()),
            bit(true),
            var("_h"),
        ))?;
        let inner_base = lam(
            "_h",
            truth(less(succ(n.clone()), zero())),
            bc(
                "FalseElim",
                vec![
                    eq(boolean(), less(zero(), succ(n.clone())), bit(false)),
                    empty.proof,
                ],
            ),
        );
        let previous = Equality::boolean(
            less(succ(n.clone()), succ(m.clone())),
            less(n.clone(), m.clone()),
            nc("LessSuccBoth", vec![n.clone(), m.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(succ(n.clone()), succ(m.clone())),
            bit(true),
            var("_h"),
        ))?;
        let result = Equality::boolean(
            less(succ(m.clone()), succ(n.clone())),
            less(m.clone(), n.clone()),
            nc("LessSuccBoth", vec![m.clone(), n.clone()]),
        )
        .then(Equality::boolean(
            less(m.clone(), n.clone()),
            bit(false),
            app(var("_prior"), vec![m.clone(), previous.proof]),
        ))?;
        let inner_step = lam(
            "_m",
            nat(),
            lam(
                "_unused",
                p(succ(n.clone()), m.clone()),
                lam(
                    "_h",
                    truth(less(succ(n.clone()), succ(m.clone()))),
                    result.proof,
                ),
            ),
        );
        let step = lam(
            "_n",
            nat(),
            lam(
                "_prior",
                pi("b", nat(), p(n.clone(), b.clone())),
                lam(
                    "b",
                    nat(),
                    induct(
                        "_m",
                        p(succ(n.clone()), m),
                        inner_base,
                        inner_step,
                        b.clone(),
                    ),
                ),
            ),
        );
        let proof = app(
            induct(
                "_n",
                pi("b", nat(), p(n.clone(), b.clone())),
                base,
                step,
                a.clone(),
            ),
            vec![b.clone()],
        );
        self.emit(
            "NatAsym",
            &[("a", nat()), ("b", nat())],
            p(a, b),
            proof,
            true,
        )
    }
    fn index_comparisons(&mut self, d: usize) -> Result<()> {
        let h = d - 1;
        let a = address_bits(&POS[..h]);
        let b = address_bits(&QUERY[..h]);
        let mut args = POS[..h].iter().map(|n| (*n, boolean())).collect::<Vec<_>>();
        args.extend(QUERY[..h].iter().map(|n| (*n, boolean())));
        for high in [false, true] {
            let mut x = vec![bit(high)];
            x.extend(a.clone());
            let mut y = vec![bit(high)];
            y.extend(b.clone());
            let ai = idx(h, a.clone());
            let bi = idx(h, b.clone());
            let cap = fv(h, "Capacity", vec![]);
            let ax = if high {
                add(cap.clone(), ai.clone())
            } else {
                ai.clone()
            };
            let by = if high {
                add(cap.clone(), bi.clone())
            } else {
                bi.clone()
            };
            let name = if high { "High" } else { "Low" };
            let mut e = less_left(
                Equality::natural(
                    idx(d, x),
                    ax.clone(),
                    call(
                        &format!("Mpk.Go.Practical.String.Validity.Index{name}{d}"),
                        a.clone(),
                    ),
                ),
                idx(d, y.clone()),
            )
            .then(less_right(
                ax.clone(),
                Equality::natural(
                    idx(d, y),
                    by.clone(),
                    call(
                        &format!("Mpk.Go.Practical.String.Validity.Index{name}{d}"),
                        b.clone(),
                    ),
                ),
            ))?;
            if high {
                e = e
                    .then(less_left(
                        Equality::natural(
                            ax,
                            add(ai.clone(), cap.clone()),
                            nc("AddComm", vec![cap.clone(), ai.clone()]),
                        ),
                        by.clone(),
                    ))?
                    .then(less_right(
                        add(ai.clone(), cap.clone()),
                        Equality::natural(
                            by,
                            add(bi.clone(), cap.clone()),
                            nc("AddComm", vec![cap.clone(), bi.clone()]),
                        ),
                    ))?
                    .then(Equality::boolean(
                        less(add(ai.clone(), cap.clone()), add(bi.clone(), cap.clone())),
                        less(ai, bi),
                        nc(
                            "LessTranslate",
                            vec![idx(h, a.clone()), idx(h, b.clone()), cap],
                        ),
                    ))?;
            }
            self.theorem(&format!("D{d}.{name}Compare"), &args, e)?;
        }
        let mut low = vec![bit(false)];
        low.extend(a.clone());
        let mut high = vec![bit(true)];
        high.extend(b.clone());
        let mut tail = a.clone();
        tail.extend(b.clone());
        self.theorem(
            &format!("D{d}.CrossCompare"),
            &args,
            Equality::boolean(
                less(idx(d, low.clone()), idx(d, high.clone())),
                bit(true),
                call(
                    &format!("Mpk.Go.Practical.String.Validity.LessCross{d}"),
                    tail,
                ),
            ),
        )?;
        self.theorem(
            &format!("D{d}.ReverseCompare"),
            &args,
            Equality::boolean(
                less(idx(d, high.clone()), idx(d, low.clone())),
                bit(false),
                oc(
                    "NatAsym",
                    vec![idx(d, low), idx(d, high), {
                        let mut args = a;
                        args.extend(b);
                        oc(&format!("D{d}.CrossCompare"), args)
                    }],
                ),
            ),
        )
    }
    fn bytes(&mut self) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let args = [("x", byte()), ("y", byte())];
        for (name, op, bridge, natural) in [
            (
                "ByteLess",
                "UnsignedLess",
                "UnsignedLessNatural",
                less(numeric_byte(x.clone()), numeric_byte(y.clone())),
            ),
            (
                "ByteLessEqual",
                "UnsignedLessEqual",
                "UnsignedLessEqualNatural",
                nc(
                    "LessEqual",
                    vec![numeric_byte(x.clone()), numeric_byte(y.clone())],
                ),
            ),
        ] {
            self.emit(
                name,
                &args,
                boolean(),
                word::wc(8, op, vec![x.clone(), y.clone()]),
                false,
            )?;
            self.theorem(
                &format!("{name}Natural"),
                &args,
                Equality::boolean(
                    oc(name, vec![x.clone(), y.clone()]),
                    natural,
                    call(
                        &format!("Mpk.Go.Practical.Word.Order.W8.{bridge}"),
                        vec![x.clone(), y.clone()],
                    ),
                ),
            )?;
        }
        self.theorem(
            "ByteEqualCircuit",
            &args,
            Equality::boolean(
                ec("ByteEqual", vec![x.clone(), y.clone()]),
                word::wc(8, "Equal", vec![x.clone(), y.clone()]),
                refl(boolean(), word::wc(8, "Equal", vec![x.clone(), y.clone()])),
            ),
        )?;
        self.theorem(
            "ByteEqualNatural",
            &args,
            Equality::boolean(
                ec("ByteEqual", vec![x.clone(), y.clone()]),
                word::wc(8, "Equal", vec![x.clone(), y.clone()]),
                oc("ByteEqualCircuit", vec![x.clone(), y.clone()]),
            )
            .then(Equality::boolean(
                word::wc(8, "Equal", vec![x.clone(), y.clone()]),
                equal(numeric_byte(x.clone()), numeric_byte(y.clone())),
                call("Mpk.Go.Practical.Word.Order.W8.EqualNatural", vec![x, y]),
            ))?,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut order = Order(builder);
    order.boolean()?;
    order.asymmetry()?;
    for d in 1..=8 {
        order.index_comparisons(d)?;
    }
    order.bytes()?;
    for d in 0..=8 {
        order.vectors(d)?;
    }
    order.string_values()
}
