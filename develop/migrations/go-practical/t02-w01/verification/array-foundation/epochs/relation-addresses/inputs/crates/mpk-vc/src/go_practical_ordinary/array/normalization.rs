//! Preserve visible elements while canonicalizing every padding slot.
use super::*;
pub(super) fn normalized(d: usize, a: Expr, z: Expr, n: Expr, f: Expr) -> Expr {
    ac(d, "Normalize", vec![a, z, n, f])
}
pub(super) fn equivalent(d: usize, a: Expr, n: Expr, f: Expr, g: Expr) -> Expr {
    ac(d, "Equivalent", vec![a, n, f, g])
}
fn observed_equal(d: usize, a: Expr, n: Expr, f: Expr, g: Expr) -> Expr {
    cube(
        d,
        pi(
            "_in",
            inside(idx(d, address(d)), n),
            eq(a, slot(f, address(d)), slot(g, address(d))),
        ),
        false,
    )
}
fn pointwise(d: usize, n: Expr, body: Expr) -> Expr {
    cube(d, lam("_in", inside(idx(d, address(d)), n), body), true)
}
impl Arrays<'_> {
    pub(super) fn normalization(&mut self, d: usize) -> Result<()> {
        let (a, p, z, n, f, g) = (var("A"), var("P"), var("z"), var("n"), var("f"), var("g"));
        let base = [
            ("A", Expr::Sort0),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
        ];
        let at = address(d);
        let flag = less(idx(d, at.clone()), n.clone());
        let normalized_f = normalized(d, a.clone(), z.clone(), n.clone(), f.clone());
        let choice = choose(
            a.clone(),
            flag.clone(),
            slot(f.clone(), at.clone()),
            z.clone(),
        );
        self.emit(
            d,
            "Normalize",
            &base,
            cube(d, a.clone(), false),
            cube(d, choice.clone(), true),
            false,
        )?;
        for yes in [false, true] {
            let mut args = base.to_vec();
            args.extend(BITS[..d].iter().map(|b| (*b, boolean())));
            args.push(("within", truth(flag.clone(), yes)));
            self.theorem(
                d,
                if yes {
                    "NormalizeInside"
                } else {
                    "NormalizeOutside"
                },
                &args,
                Equality {
                    ty: a.clone(),
                    left: slot(normalized_f.clone(), at.clone()),
                    right: if yes {
                        slot(f.clone(), at.clone())
                    } else {
                        z.clone()
                    },
                    proof: seq(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            a.clone(),
                            flag.clone(),
                            slot(f.clone(), at.clone()),
                            z.clone(),
                            var("within"),
                        ],
                    ),
                },
            )?;
        }
        let mut args = vec![
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("all", elements(d, p.clone(), f.clone())),
            ("hz", app(p.clone(), vec![z.clone()])),
        ];
        let pchoice = |yes| Equality {
            ty: a.clone(),
            left: slot(normalized_f.clone(), at.clone()),
            right: if yes {
                slot(f.clone(), at.clone())
            } else {
                z.clone()
            },
            proof: seq(
                if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                vec![
                    a.clone(),
                    flag.clone(),
                    slot(f.clone(), at.clone()),
                    z.clone(),
                    var("_case"),
                ],
            ),
        };
        let no = retype(a.clone(), p.clone(), pchoice(false).reverse(), var("hz"));
        let yes = retype(
            a.clone(),
            p.clone(),
            pchoice(true).reverse(),
            slot(var("all"), at.clone()),
        );
        self.emit(
            d,
            "NormalizeElements",
            &args,
            elements(d, p.clone(), normalized_f.clone()),
            cube(
                d,
                decide(
                    flag.clone(),
                    app(p.clone(), vec![slot(normalized_f.clone(), at.clone())]),
                    no,
                    yes,
                ),
                true,
            ),
            true,
        )?;
        let mut outside_args = vec![a.clone(), z.clone(), n.clone(), f.clone()];
        outside_args.extend(at.clone());
        outside_args.push(var("_out"));
        self.emit(
            d,
            "NormalizePadding",
            &base,
            padding(d, a.clone(), z.clone(), n.clone(), normalized_f.clone()),
            cube(
                d,
                lam(
                    "_out",
                    truth(flag.clone(), false),
                    ac(d, "NormalizeOutside", outside_args),
                ),
                true,
            ),
            true,
        )?;
        self.emit(
            d,
            "NormalizeValid",
            &args,
            valid(
                d,
                a.clone(),
                p.clone(),
                z.clone(),
                n.clone(),
                normalized_f.clone(),
            ),
            pc(
                "Make",
                vec![
                    elements(d, p.clone(), normalized_f.clone()),
                    padding(d, a.clone(), z.clone(), n.clone(), normalized_f.clone()),
                    ac(
                        d,
                        "NormalizeElements",
                        vec![
                            a.clone(),
                            p.clone(),
                            z.clone(),
                            n.clone(),
                            f.clone(),
                            var("all"),
                            var("hz"),
                        ],
                    ),
                    ac(
                        d,
                        "NormalizePadding",
                        vec![a.clone(), z.clone(), n.clone(), f.clone()],
                    ),
                ],
            ),
            true,
        )?;
        args = vec![
            ("A", Expr::Sort0),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("g", cube(d, a.clone(), false)),
        ];
        self.emit(
            d,
            "Equivalent",
            &args,
            Expr::Sort0,
            observed_equal(d, a.clone(), n.clone(), f.clone(), g.clone()),
            false,
        )?;
        let mut exposed_args = args.clone();
        exposed_args.push((
            "h",
            equivalent(d, a.clone(), n.clone(), f.clone(), g.clone()),
        ));
        self.emit(
            d,
            "EquivalentAsFunction",
            &exposed_args,
            observed_equal(d, a.clone(), n.clone(), f.clone(), g.clone()),
            var("h"),
            false,
        )?;
        self.emit(
            d,
            "EquivalentRefl",
            &args[..3],
            equivalent(d, a.clone(), n.clone(), f.clone(), f.clone()),
            pointwise(d, n.clone(), refl(a.clone(), slot(f.clone(), at.clone()))),
            true,
        )?;
        args.push((
            "h",
            equivalent(d, a.clone(), n.clone(), f.clone(), g.clone()),
        ));
        self.emit(
            d,
            "EquivalentSymm",
            &args,
            equivalent(d, a.clone(), n.clone(), g.clone(), f.clone()),
            pointwise(
                d,
                n.clone(),
                Equality {
                    ty: a.clone(),
                    left: slot(f.clone(), at.clone()),
                    right: slot(g.clone(), at.clone()),
                    proof: app(
                        slot(
                            ac(
                                d,
                                "EquivalentAsFunction",
                                vec![a.clone(), n.clone(), f.clone(), g.clone(), var("h")],
                            ),
                            at.clone(),
                        ),
                        vec![var("_in")],
                    ),
                }
                .reverse()
                .proof,
            ),
            true,
        )?;
        let norm_eq_args = base.to_vec();
        let mut inside_args = vec![a.clone(), z.clone(), n.clone(), f.clone()];
        inside_args.extend(at.clone());
        inside_args.push(var("_in"));
        self.emit(
            d,
            "NormalizeEquivalent",
            &norm_eq_args,
            equivalent(d, a.clone(), n.clone(), normalized_f.clone(), f.clone()),
            pointwise(d, n.clone(), ac(d, "NormalizeInside", inside_args)),
            true,
        )?;
        let mut id_args = vec![a.clone(), z.clone(), n.clone(), normalized_f.clone()];
        id_args.extend(at.clone());
        id_args.push(var("_case"));
        let outer = slot(
            normalized(d, a.clone(), z.clone(), n.clone(), normalized_f.clone()),
            at.clone(),
        );
        let no = Equality {
            ty: a.clone(),
            left: outer.clone(),
            right: z.clone(),
            proof: ac(d, "NormalizeOutside", id_args.clone()),
        }
        .then(
            Equality {
                ty: a.clone(),
                left: slot(normalized_f.clone(), at.clone()),
                right: z.clone(),
                proof: seq(
                    "ChoiceFalse",
                    vec![
                        a.clone(),
                        flag.clone(),
                        slot(f.clone(), at.clone()),
                        z.clone(),
                        var("_case"),
                    ],
                ),
            }
            .reverse(),
        )?;
        let yes = ac(d, "NormalizeInside", id_args);
        self.emit(
            d,
            "NormalizeIdempotent",
            &base,
            cube(
                d,
                eq(
                    a.clone(),
                    outer.clone(),
                    slot(normalized_f.clone(), at.clone()),
                ),
                false,
            ),
            cube(
                d,
                decide(
                    flag,
                    eq(a.clone(), outer, slot(normalized_f.clone(), at)),
                    no.proof,
                    yes,
                ),
                true,
            ),
            true,
        )?;
        self.equivalent_reads(d)
    }
    fn equivalent_reads(&mut self, d: usize) -> Result<()> {
        let (a, n, f, g, i) = (var("A"), var("n"), var("f"), var("g"), var("i"));
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("g", cube(d, a.clone(), false)),
            ("i", word()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(i.clone()), n.clone())),
            (
                "h",
                equivalent(d, a.clone(), n.clone(), f.clone(), g.clone()),
            ),
        ];
        let at = indices(i.clone(), d)?;
        let e = Equality::natural(
            value(i.clone()),
            idx(d, at.clone()),
            ix(
                d,
                "BoundedIndex",
                vec![
                    i.clone(),
                    bounded(d, n.clone(), i.clone(), var("shape"), var("within")),
                ],
            ),
        );
        let slot_within = nl(e.reverse(), n.clone()).then(Equality::boolean(
            less(value(i.clone()), n.clone()),
            bit(true),
            var("within"),
        ))?;
        let eq = Equality {
            ty: a.clone(),
            left: read(d, a.clone(), f.clone(), i.clone()),
            right: slot(f.clone(), at.clone()),
            proof: ac(
                d,
                "ReadWithinSlot",
                vec![
                    a.clone(),
                    n.clone(),
                    f.clone(),
                    i.clone(),
                    var("shape"),
                    var("within"),
                ],
            ),
        }
        .then(Equality {
            ty: a.clone(),
            left: slot(f.clone(), at.clone()),
            right: slot(g.clone(), at.clone()),
            proof: app(
                slot(
                    ac(
                        d,
                        "EquivalentAsFunction",
                        vec![a.clone(), n.clone(), f.clone(), g.clone(), var("h")],
                    ),
                    at.clone(),
                ),
                vec![slot_within.proof],
            ),
        })?
        .then(
            Equality {
                ty: a.clone(),
                left: read(d, a.clone(), g.clone(), i.clone()),
                right: slot(g.clone(), at),
                proof: ac(
                    d,
                    "ReadWithinSlot",
                    vec![
                        a.clone(),
                        n.clone(),
                        g.clone(),
                        i.clone(),
                        var("shape"),
                        var("within"),
                    ],
                ),
            }
            .reverse(),
        )?;
        self.theorem(d, "EquivalentRead", &args, eq)?;
        let args = [
            ("A", Expr::Sort0),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(i.clone()), n.clone())),
        ];
        self.theorem(
            d,
            "ReadNormalize",
            &args,
            Equality {
                ty: a.clone(),
                left: read(
                    d,
                    a.clone(),
                    normalized(d, a.clone(), var("z"), n.clone(), f.clone()),
                    i.clone(),
                ),
                right: read(d, a.clone(), f.clone(), i.clone()),
                proof: ac(
                    d,
                    "EquivalentRead",
                    vec![
                        a.clone(),
                        n.clone(),
                        normalized(d, a.clone(), var("z"), n.clone(), f.clone()),
                        f.clone(),
                        i.clone(),
                        var("shape"),
                        var("within"),
                        ac(
                            d,
                            "NormalizeEquivalent",
                            vec![a.clone(), var("z"), n.clone(), f.clone()],
                        ),
                    ],
                ),
            },
        )?;
        self.theorem(
            d,
            "ReadCopy",
            &[
                ("A", Expr::Sort0),
                ("f", cube(d, a.clone(), false)),
                ("i", word()),
            ],
            Equality {
                ty: a.clone(),
                left: read(
                    d,
                    a.clone(),
                    ac(d, "Copy", vec![a.clone(), f.clone()]),
                    i.clone(),
                ),
                right: read(d, a.clone(), f.clone(), i.clone()),
                proof: Equality {
                    ty: cube(d, a.clone(), false),
                    left: ac(d, "Copy", vec![a.clone(), f.clone()]),
                    right: f.clone(),
                    proof: ac(d, "CopyIdentity", vec![a.clone(), f.clone()]),
                }
                .mapped(
                    lam(
                        "_data",
                        cube(d, a.clone(), false),
                        read(d, a.clone(), var("_data"), i.clone()),
                    ),
                    read(
                        d,
                        a.clone(),
                        ac(d, "Copy", vec![a.clone(), f.clone()]),
                        i.clone(),
                    ),
                    read(d, a.clone(), f, i),
                    a,
                )
                .proof,
            },
        )
    }
}
