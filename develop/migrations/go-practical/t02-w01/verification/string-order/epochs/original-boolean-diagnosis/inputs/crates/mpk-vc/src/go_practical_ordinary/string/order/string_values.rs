//! End markers and real unsigned byte values, followed by four comparisons.
use super::*;

fn slot(s: Expr, at: Vec<Expr>) -> Expr {
    app(data(s), at)
}
fn present(s: Expr, at: Vec<Expr>) -> Expr {
    less(index(at), size(s))
}
fn choice(flag: Expr, yes: Expr, no: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_flag", boolean(), nat()), no, yes, flag],
    )
}
fn with_address(mut args: Vec<Expr>, at: Vec<Expr>, witness: Option<Expr>) -> Vec<Expr> {
    args.extend(at);
    args.extend(witness);
    args
}
fn rank_case(s: Expr, at: Vec<Expr>, inside: bool, witness: Expr) -> Equality {
    Equality::natural(
        rank(s.clone(), at.clone()),
        if inside {
            succ(numeric_byte(slot(s.clone(), at.clone())))
        } else {
            zero()
        },
        oc(
            if inside { "RankInside" } else { "RankOutside" },
            with_address(vec![s], at, Some(witness)),
        ),
    )
}
fn string_less(s: Expr, t: Expr) -> Expr {
    oc("Less", vec![s, t])
}
fn rank_prefix(s: Expr, t: Expr, at: Vec<Expr>) -> Expr {
    let q = address_bits(&QUERY);
    cube(
        &QUERY,
        pi(
            "_before",
            truth(less(index(q.clone()), index(at))),
            eq(nat(), rank(s, q.clone()), rank(t, q)),
        ),
        false,
    )
}
fn rank_handler(s: Expr, t: Expr, result: Expr) -> Expr {
    let at = address_bits(&POS);
    cube(
        &POS,
        pi(
            "_prior",
            rank_prefix(s.clone(), t.clone(), at.clone()),
            pi(
                "_strict",
                truth(less(rank(s, at.clone()), rank(t, at))),
                result,
            ),
        ),
        false,
    )
}

impl Order<'_> {
    fn rank_definitions(&mut self) -> Result<()> {
        let s = var("s");
        let at = address_bits(&POS);
        let yes = succ(numeric_byte(slot(s.clone(), at.clone())));
        let flag = present(s.clone(), at.clone());
        let mut args = vec![("s", string_type())];
        args.extend(POS.iter().map(|name| (*name, boolean())));
        let body = choice(flag.clone(), yes.clone(), zero());
        self.emit("Rank", &args, nat(), body.clone(), false)?;
        self.theorem(
            "RankStep",
            &args,
            Equality::natural(
                rank(s.clone(), at.clone()),
                body.clone(),
                refl(nat(), body.clone()),
            ),
        )?;
        for inside in [false, true] {
            let mut args = args.clone();
            args.push(("branch", eq(boolean(), flag.clone(), bit(inside))));
            let value = if inside { yes.clone() } else { zero() };
            let e = Equality::natural(
                rank(s.clone(), at.clone()),
                body.clone(),
                oc("RankStep", with_address(vec![s.clone()], at.clone(), None)),
            )
            .then(Equality::natural(
                body.clone(),
                value,
                sc(
                    if inside { "ChoiceTrue" } else { "ChoiceFalse" },
                    vec![nat(), flag.clone(), yes.clone(), zero(), var("branch")],
                ),
            ))?;
            self.theorem(if inside { "RankInside" } else { "RankOutside" }, &args, e)?;
        }
        self.emit(
            "Ranks",
            &[("s", string_type())],
            vector(8),
            cube(&POS, rank(s.clone(), at.clone()), true),
            false,
        )?;
        self.theorem(
            "RanksAt",
            &args,
            Equality::natural(
                app(ranks(s.clone()), at.clone()),
                rank(s, at.clone()),
                refl(nat(), rank(var("s"), at)),
            ),
        )
    }
    fn byte_rank_order(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let at = address_bits(&POS);
        let mut pair = vec![("s", string_type()), ("t", string_type())];
        pair.extend(POS.iter().map(|name| (*name, boolean())));
        for (a, b, name) in [
            (true, true, "RankLessInside"),
            (false, true, "RankEndBefore"),
            (true, false, "RankEndAfter"),
            (false, false, "RankEndEqual"),
        ] {
            let mut args = pair.clone();
            args.extend([
                (
                    "left",
                    eq(boolean(), present(s.clone(), at.clone()), bit(a)),
                ),
                (
                    "right",
                    eq(boolean(), present(t.clone(), at.clone()), bit(b)),
                ),
            ]);
            let x = rank_case(s.clone(), at.clone(), a, var("left"));
            let y = rank_case(t.clone(), at.clone(), b, var("right"));
            if !a && !b {
                self.theorem(name, &args, x.then(y.reverse())?)?;
                continue;
            }
            let e = less_left(x.clone(), y.left.clone())
                .then(less_right(x.right.clone(), y.clone()))?;
            let e = if a && b {
                let av = numeric_byte(slot(s.clone(), at.clone()));
                let bv = numeric_byte(slot(t.clone(), at.clone()));
                e.then(Equality::boolean(
                    less(succ(av.clone()), succ(bv.clone())),
                    less(av.clone(), bv.clone()),
                    nc("LessSuccBoth", vec![av.clone(), bv.clone()]),
                ))?
                .then(
                    Equality::boolean(
                        oc(
                            "ByteLess",
                            vec![slot(s.clone(), at.clone()), slot(t.clone(), at.clone())],
                        ),
                        less(av, bv),
                        oc(
                            "ByteLessNatural",
                            vec![slot(s.clone(), at.clone()), slot(t.clone(), at.clone())],
                        ),
                    )
                    .reverse(),
                )?
            } else if !a {
                e.then(Equality::boolean(
                    less(zero(), y.right.clone()),
                    bit(true),
                    nc(
                        "LessZeroSucc",
                        vec![numeric_byte(slot(t.clone(), at.clone()))],
                    ),
                ))?
            } else {
                e.then(Equality::boolean(
                    less(x.right.clone(), zero()),
                    bit(false),
                    nc("LessZero", vec![x.right]),
                ))?
            };
            self.theorem(name, &args, e)?;
        }
        Ok(())
    }
    fn rank_preservation(&mut self) -> Result<()> {
        let (s, t, e) = (var("s"), var("t"), var("related"));
        let at = address_bits(&POS);
        let i = index(at.clone());
        let mut args = vec![
            ("s", string_type()),
            ("t", string_type()),
            ("related", ec("Equivalent", vec![s.clone(), t.clone()])),
        ];
        args.extend(POS.iter().map(|name| (*name, boolean())));
        let sizes = Equality::natural(
            size(s.clone()),
            size(t.clone()),
            ec("EquivalentLength", vec![s.clone(), t.clone(), e.clone()]),
        );
        let guard = less_right(i, sizes);
        let branch = |inside: bool| -> Result<Expr> {
            let rhs = guard.clone().reverse().then(Equality::boolean(
                present(s.clone(), at.clone()),
                bit(inside),
                var("_branch"),
            ))?;
            let left = rank_case(s.clone(), at.clone(), inside, var("_branch"));
            let right = rank_case(t.clone(), at.clone(), inside, rhs.proof);
            if !inside {
                return Ok(left.then(right.reverse())?.proof);
            }
            let mut bits = at.clone();
            bits.push(var("_branch"));
            let byte_eq = app(
                ec("EquivalentContents", vec![s.clone(), t.clone(), e.clone()]),
                bits,
            );
            let number = Equality::natural(
                numeric_byte(slot(s.clone(), at.clone())),
                numeric_byte(slot(t.clone(), at.clone())),
                numeric(
                    "ValueCongruence3",
                    vec![
                        slot(s.clone(), at.clone()),
                        slot(t.clone(), at.clone()),
                        byte_eq,
                    ],
                ),
            );
            Ok(left.then(number.on_succ())?.then(right.reverse())?.proof)
        };
        let proof = decide(
            present(s.clone(), at.clone()),
            eq(
                nat(),
                rank(s.clone(), at.clone()),
                rank(t.clone(), at.clone()),
            ),
            branch(false)?,
            branch(true)?,
        );
        self.theorem(
            "RankRespects",
            &args,
            Equality::natural(rank(s.clone(), at.clone()), rank(t, at.clone()), proof),
        )?;
        let mut args = vec![("s", string_type())];
        args.extend(POS.iter().map(|name| (*name, boolean())));
        self.theorem(
            "RankNormalized",
            &args,
            Equality::natural(
                rank(norm(s.clone()), at.clone()),
                rank(s.clone(), at.clone()),
                oc(
                    "RankRespects",
                    with_address(
                        vec![
                            norm(s.clone()),
                            s.clone(),
                            ec("NormalizationEquivalent", vec![s]),
                        ],
                        at,
                        None,
                    ),
                ),
            ),
        )
    }
    fn comparison_definitions(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let args = [("s", string_type()), ("t", string_type())];
        for (name, body) in [
            ("Less", dlt(8, ranks(s.clone()), ranks(t.clone()))),
            ("Greater", dlt(8, ranks(t.clone()), ranks(s.clone()))),
            (
                "LessEqual",
                bc("Not", vec![dlt(8, ranks(t.clone()), ranks(s.clone()))]),
            ),
            (
                "GreaterEqual",
                bc("Not", vec![dlt(8, ranks(s.clone()), ranks(t.clone()))]),
            ),
        ] {
            self.emit(name, &args, boolean(), body.clone(), false)?;
            self.theorem(
                &format!("{name}Step"),
                &args,
                Equality::boolean(
                    oc(name, vec![s.clone(), t.clone()]),
                    body.clone(),
                    refl(boolean(), body),
                ),
            )?;
        }
        for (name, reverse) in [("Less", false), ("Greater", true)] {
            let (a, b) = if reverse {
                (t.clone(), s.clone())
            } else {
                (s.clone(), t.clone())
            };
            let cmp = oc(name, vec![s.clone(), t.clone()]);
            self.emit(
                &format!("{name}Reflect"),
                &[args[0].clone(), args[1].clone(), ("h", truth(cmp.clone()))],
                pi(
                    "R",
                    Expr::Sort0,
                    pi(
                        "on_first",
                        rank_handler(a.clone(), b.clone(), var("R")),
                        var("R"),
                    ),
                ),
                oc(
                    "D8.LessReflect",
                    vec![ranks(a.clone()), ranks(b.clone()), var("h")],
                ),
                true,
            )?;
            let at = address_bits(&POS);
            let mut binders = args.to_vec();
            binders.extend(POS.iter().map(|name| (*name, boolean())));
            binders.extend([
                ("prior", rank_prefix(a.clone(), b.clone(), at.clone())),
                (
                    "strict",
                    truth(less(
                        rank(a.clone(), at.clone()),
                        rank(b.clone(), at.clone()),
                    )),
                ),
            ]);
            self.emit(
                &format!("{name}Complete"),
                &binders,
                truth(cmp),
                oc(
                    "D8.LessComplete",
                    with_address(vec![ranks(a), ranks(b)], at, Some(var("prior")))
                        .into_iter()
                        .chain([var("strict")])
                        .collect(),
                ),
                true,
            )?;
        }
        Ok(())
    }
    fn comparisons_preserved(&mut self) -> Result<()> {
        let (s, t, s2, t2) = (var("s"), var("t"), var("s2"), var("t2"));
        let at = address_bits(&POS);
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("s2", string_type()),
            ("t2", string_type()),
            ("left", ec("Equivalent", vec![s.clone(), s2.clone()])),
            ("right", ec("Equivalent", vec![t.clone(), t2.clone()])),
        ];
        for (name, reverse, negative) in [
            ("Less", false, false),
            ("Greater", true, false),
            ("LessEqual", true, true),
            ("GreaterEqual", false, true),
        ] {
            let (a, b, a2, b2, ea, eb) = if reverse {
                (
                    t.clone(),
                    s.clone(),
                    t2.clone(),
                    s2.clone(),
                    var("right"),
                    var("left"),
                )
            } else {
                (
                    s.clone(),
                    t.clone(),
                    s2.clone(),
                    t2.clone(),
                    var("left"),
                    var("right"),
                )
            };
            let e = Equality::boolean(
                dlt(8, ranks(a.clone()), ranks(b.clone())),
                dlt(8, ranks(a2.clone()), ranks(b2.clone())),
                oc(
                    "D8.LessRespects",
                    vec![
                        ranks(a.clone()),
                        ranks(b.clone()),
                        ranks(a2.clone()),
                        ranks(b2.clone()),
                        cube(
                            &POS,
                            oc(
                                "RankRespects",
                                with_address(vec![a, a2, ea], at.clone(), None),
                            ),
                            true,
                        ),
                        cube(
                            &POS,
                            oc(
                                "RankRespects",
                                with_address(vec![b, b2, eb], at.clone(), None),
                            ),
                            true,
                        ),
                    ],
                ),
            );
            let e = if negative { neg(e) } else { e };
            let result = Equality::boolean(
                oc(name, vec![s.clone(), t.clone()]),
                e.left.clone(),
                oc(&format!("{name}Step"), vec![s.clone(), t.clone()]),
            )
            .then(e.clone())?
            .then(
                Equality::boolean(
                    oc(name, vec![s2.clone(), t2.clone()]),
                    e.right,
                    oc(&format!("{name}Step"), vec![s2.clone(), t2.clone()]),
                )
                .reverse(),
            )?;
            self.theorem(&format!("{name}Respects"), &args, result)?;
            self.theorem(
                &format!("{name}Normalized"),
                &[args[0].clone(), args[1].clone()],
                Equality::boolean(
                    oc(name, vec![norm(s.clone()), norm(t.clone())]),
                    oc(name, vec![s.clone(), t.clone()]),
                    oc(
                        &format!("{name}Respects"),
                        vec![
                            norm(s.clone()),
                            norm(t.clone()),
                            s.clone(),
                            t.clone(),
                            ec("NormalizationEquivalent", vec![s.clone()]),
                            ec("NormalizationEquivalent", vec![t.clone()]),
                        ],
                    ),
                ),
            )?;
        }
        // Explicit complement/reversal facts are ordinary Boolean equalities.
        for (name, right) in [
            ("GreaterReverse", string_less(t.clone(), s.clone())),
            (
                "LessEqualComplement",
                bc("Not", vec![oc("Greater", vec![s.clone(), t.clone()])]),
            ),
            (
                "GreaterEqualComplement",
                bc("Not", vec![string_less(s.clone(), t.clone())]),
            ),
        ] {
            let op = if name.starts_with("GreaterReverse") {
                "Greater"
            } else if name.starts_with("LessEqual") {
                "LessEqual"
            } else {
                "GreaterEqual"
            };
            self.theorem(
                name,
                &[args[0].clone(), args[1].clone()],
                Equality::boolean(
                    oc(op, vec![s.clone(), t.clone()]),
                    right.clone(),
                    refl(boolean(), right),
                ),
            )?;
        }
        let ranks_eq = deq(8, ranks(s.clone()), ranks(t.clone()));
        self.theorem(
            "TotalRanksEqual",
            &[args[0].clone(), args[1].clone()],
            Equality::boolean(
                flag_total(
                    string_less(s.clone(), t.clone()),
                    string_less(t.clone(), s.clone()),
                ),
                ranks_eq,
                oc("D8.TotalEquality", vec![ranks(s.clone()), ranks(t.clone())]),
            ),
        )?;
        for (name, yes) in [
            ("Less", false),
            ("Greater", false),
            ("LessEqual", true),
            ("GreaterEqual", true),
        ] {
            let raw = Equality::boolean(
                dlt(8, ranks(s.clone()), ranks(s.clone())),
                bit(false),
                oc("D8.LessRefl", vec![ranks(s.clone())]),
            );
            let body = if yes {
                neg(raw).then(Equality::boolean(
                    bc("Not", vec![bit(false)]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ))?
            } else {
                raw
            };
            self.theorem(
                &format!("{name}Refl"),
                &[args[0].clone()],
                Equality::boolean(oc(name, vec![s.clone(), s.clone()]), bit(yes), body.proof),
            )?;
        }
        Ok(())
    }
    pub(super) fn string_values(&mut self) -> Result<()> {
        self.rank_definitions()?;
        self.byte_rank_order()?;
        self.rank_preservation()?;
        self.comparison_definitions()?;
        self.comparisons_preserved()
    }
}
