//! Algebra requires actual conditional laws of the element comparator.
use super::*;
fn evidence(d: usize, f: Expr, g: Expr, h: &str) -> Result<Equality> {
    let at = address(d);
    let folded = slot(
        ec(d, "Reflect", vec![flags(d, f.clone(), g.clone()), var(h)]),
        at.clone(),
    );
    let mut args = value_args(f.clone(), g.clone());
    args.extend(at.clone());
    args.push(var("_case"));
    Equality::boolean(
        slot(flags(d, f.clone(), g.clone()), at.clone()),
        compared(
            var("cmp"),
            slot(f.clone(), at.clone()),
            slot(g.clone(), at.clone()),
        ),
        qc(d, "FlagInside", args),
    )
    .reverse()
    .then(Equality::boolean(
        slot(flags(d, f, g), at),
        bit(true),
        folded,
    ))
}
impl Comparisons<'_> {
    pub(super) fn algebra(&mut self, d: usize) -> Result<()> {
        let (a, p, f, g, u) = (var("A"), var("P"), var("f"), var("g"), var("u"));
        let at = address(d);
        let condition = less(idx(d, at.clone()), var("n"));
        let mut args = vec![
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("cmp", comparator(a.clone())),
            ("fp", elements(d, p.clone(), f.clone())),
        ];
        args.push((
            "refl",
            pi(
                "_x",
                a.clone(),
                pi(
                    "_px",
                    app(p.clone(), vec![var("_x")]),
                    truth(compared(var("cmp"), var("_x"), var("_x")), true),
                ),
            ),
        ));
        let ff = flags(d, f.clone(), f.clone());
        let mut selected = value_args(f.clone(), f.clone());
        selected.extend(at.clone());
        selected.push(var("_case"));
        let cmp = compared(
            var("cmp"),
            slot(f.clone(), at.clone()),
            slot(f.clone(), at.clone()),
        );
        let yes = Equality::boolean(
            slot(ff.clone(), at.clone()),
            cmp.clone(),
            qc(d, "FlagInside", selected.clone()),
        )
        .then(Equality::boolean(
            cmp,
            bit(true),
            app(
                var("refl"),
                vec![slot(f.clone(), at.clone()), slot(var("fp"), at.clone())],
            ),
        ))?;
        let no = qc(d, "FlagOutside", selected);
        self.theorem(
            d,
            "Reflexive",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), f.clone()),
                bit(true),
                ec(
                    d,
                    "Complete",
                    vec![
                        ff.clone(),
                        cube(
                            d,
                            decide(
                                condition.clone(),
                                truth(slot(ff, at.clone()), true),
                                no,
                                yes.proof,
                            ),
                            true,
                        ),
                    ],
                ),
            ),
        )?;
        args = vec![("A", Expr::Sort0), ("P", pred(a.clone()))];
        args.extend(actual(d).into_iter().skip(1));
        args.extend([
            ("fp", elements(d, p.clone(), f.clone())),
            ("gp", elements(d, p.clone(), g.clone())),
        ]);
        let symmetry = pi(
            "_x",
            a.clone(),
            pi(
                "_y",
                a.clone(),
                pi(
                    "_px",
                    app(p.clone(), vec![var("_x")]),
                    pi(
                        "_py",
                        app(p.clone(), vec![var("_y")]),
                        eq(
                            boolean(),
                            compared(var("cmp"), var("_x"), var("_y")),
                            compared(var("cmp"), var("_y"), var("_x")),
                        ),
                    ),
                ),
            ),
        );
        args.push(("symm", symmetry));
        let h = cube(
            d,
            lam(
                "_in",
                inside(idx(d, at.clone()), var("n")),
                app(
                    var("symm"),
                    vec![
                        slot(f.clone(), at.clone()),
                        slot(g.clone(), at.clone()),
                        slot(var("fp"), at.clone()),
                        slot(var("gp"), at.clone()),
                    ],
                ),
            ),
            true,
        );
        let mut supplied = value_args(f.clone(), g.clone());
        supplied.extend([g.clone(), f.clone(), h]);
        self.theorem(
            d,
            "Symmetric",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), g.clone()),
                equal_by(d, g.clone(), f.clone()),
                qc(d, "Pointwise", supplied),
            ),
        )?;
        let mut args = vec![("A", Expr::Sort0), ("P", pred(a.clone()))];
        args.extend(actual(d).into_iter().skip(1));
        args.extend([
            ("u", cube(d, a.clone(), false)),
            ("fp", elements(d, p.clone(), f.clone())),
            ("gp", elements(d, p.clone(), g.clone())),
            ("up", elements(d, p.clone(), u.clone())),
        ]);
        let (x, y, z) = (var("_x"), var("_y"), var("_z"));
        let trans = pi(
            "_x",
            a.clone(),
            pi(
                "_y",
                a.clone(),
                pi(
                    "_z",
                    a,
                    pi(
                        "_px",
                        app(p.clone(), vec![x.clone()]),
                        pi(
                            "_py",
                            app(p.clone(), vec![y.clone()]),
                            pi(
                                "_pz",
                                app(p, vec![z.clone()]),
                                pi(
                                    "_xy",
                                    truth(compared(var("cmp"), x.clone(), y.clone()), true),
                                    pi(
                                        "_yz",
                                        truth(compared(var("cmp"), y, z.clone()), true),
                                        truth(compared(var("cmp"), x, z), true),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        args.extend([
            ("trans", trans),
            ("fg", truth(equal_by(d, f.clone(), g.clone()), true)),
            ("gu", truth(equal_by(d, g.clone(), u.clone()), true)),
        ]);
        let combined = app(
            var("trans"),
            vec![
                slot(f.clone(), at.clone()),
                slot(g.clone(), at.clone()),
                slot(u.clone(), at.clone()),
                slot(var("fp"), at.clone()),
                slot(var("gp"), at.clone()),
                slot(var("up"), at.clone()),
                evidence(d, f.clone(), g, "fg")?.proof,
                evidence(d, var("g"), u.clone(), "gu")?.proof,
            ],
        );
        let cmp = compared(
            var("cmp"),
            slot(f.clone(), at.clone()),
            slot(u.clone(), at.clone()),
        );
        let fs = flags(d, f.clone(), u.clone());
        let mut selected = value_args(f.clone(), u.clone());
        selected.extend(at.clone());
        selected.push(var("_case"));
        let yes = Equality::boolean(
            slot(fs.clone(), at.clone()),
            cmp.clone(),
            qc(d, "FlagInside", selected.clone()),
        )
        .then(Equality::boolean(cmp, bit(true), combined))?;
        let no = qc(d, "FlagOutside", selected);
        self.theorem(
            d,
            "Transitive",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), u),
                bit(true),
                ec(
                    d,
                    "Complete",
                    vec![
                        fs.clone(),
                        cube(
                            d,
                            decide(condition, truth(slot(fs, at), true), no, yes.proof),
                            true,
                        ),
                    ],
                ),
            ),
        )
    }
}
