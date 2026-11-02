//! Persistent value updates retain element validity and canonical padding.
use super::*;
fn updated(d: usize, a: Expr, f: Expr, i: Expr, x: Expr) -> Expr {
    ac(d, "Set", vec![a, f, i, x])
}
impl Arrays<'_> {
    pub(super) fn updates(&mut self, d: usize) -> Result<()> {
        let (a, p, z, n, f, i, x) = (
            var("A"),
            var("P"),
            var("z"),
            var("n"),
            var("f"),
            var("i"),
            var("x"),
        );
        let at = address(d);
        let test = equal(idx(d, at.clone()), value(i.clone()));
        let base = [
            ("A", Expr::Sort0),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("x", a.clone()),
        ];
        let out = updated(d, a.clone(), f.clone(), i.clone(), x.clone());
        self.emit(
            d,
            "Set",
            &base,
            cube(d, a.clone(), false),
            cube(
                d,
                choose(
                    a.clone(),
                    test.clone(),
                    x.clone(),
                    slot(f.clone(), at.clone()),
                ),
                true,
            ),
            false,
        )?;
        for yes in [false, true] {
            let mut args = base.to_vec();
            args.extend(BITS[..d].iter().map(|b| (*b, boolean())));
            args.push(("same", truth(test.clone(), yes)));
            self.theorem(
                d,
                if yes { "SetSameSlot" } else { "SetOtherSlot" },
                &args,
                Equality {
                    ty: a.clone(),
                    left: slot(out.clone(), at.clone()),
                    right: if yes {
                        x.clone()
                    } else {
                        slot(f.clone(), at.clone())
                    },
                    proof: seq(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            a.clone(),
                            test.clone(),
                            x.clone(),
                            slot(f.clone(), at.clone()),
                            var("same"),
                        ],
                    ),
                },
            )?;
        }
        let args = [
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("x", a.clone()),
            ("all", elements(d, p.clone(), f.clone())),
            ("hx", app(p.clone(), vec![x.clone()])),
        ];
        let side = |yes| Equality {
            ty: a.clone(),
            left: slot(out.clone(), at.clone()),
            right: if yes {
                x.clone()
            } else {
                slot(f.clone(), at.clone())
            },
            proof: seq(
                if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                vec![
                    a.clone(),
                    test.clone(),
                    x.clone(),
                    slot(f.clone(), at.clone()),
                    var("_case"),
                ],
            ),
        };
        self.emit(
            d,
            "SetElements",
            &args,
            elements(d, p.clone(), out.clone()),
            cube(
                d,
                decide(
                    test.clone(),
                    app(p.clone(), vec![slot(out.clone(), at.clone())]),
                    retype(
                        a.clone(),
                        p.clone(),
                        side(false).reverse(),
                        slot(var("all"), at.clone()),
                    ),
                    retype(a.clone(), p.clone(), side(true).reverse(), var("hx")),
                ),
                true,
            ),
            true,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("x", a.clone()),
            (
                "pad",
                padding(d, a.clone(), z.clone(), n.clone(), f.clone()),
            ),
            ("within", inside(value(i.clone()), n.clone())),
        ];
        let same = Equality::natural(
            idx(d, at.clone()),
            value(i.clone()),
            nc(
                "EqualReflect",
                vec![idx(d, at.clone()), value(i.clone()), var("_case")],
            ),
        );
        let contradiction = nl(same, n.clone())
            .reverse()
            .then(Equality::boolean(
                less(idx(d, at.clone()), n.clone()),
                bit(false),
                var("_out"),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(value(i.clone()), n.clone()),
                bit(true),
                var("within"),
            ))?;
        let impossible = bc(
            "FalseElim",
            vec![
                eq(a.clone(), slot(out.clone(), at.clone()), z.clone()),
                contradiction.proof,
            ],
        );
        let no = side(false).then(Equality {
            ty: a.clone(),
            left: slot(f.clone(), at.clone()),
            right: z.clone(),
            proof: app(slot(var("pad"), at.clone()), vec![var("_out")]),
        })?;
        self.emit(
            d,
            "SetPadding",
            &args,
            padding(d, a.clone(), z.clone(), n.clone(), out.clone()),
            cube(
                d,
                lam(
                    "_out",
                    truth(less(idx(d, at.clone()), n.clone()), false),
                    decide(
                        test,
                        eq(a.clone(), slot(out.clone(), at), z.clone()),
                        no.proof,
                        impossible,
                    ),
                ),
                true,
            ),
            true,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("x", a.clone()),
            (
                "h",
                valid(d, a.clone(), p.clone(), z.clone(), n.clone(), f.clone()),
            ),
            ("hx", app(p.clone(), vec![x.clone()])),
            ("within", inside(value(i.clone()), n.clone())),
        ];
        self.emit(
            d,
            "SetValid",
            &args,
            valid(d, a.clone(), p.clone(), z.clone(), n.clone(), out.clone()),
            pc(
                "Make",
                vec![
                    elements(d, p.clone(), out.clone()),
                    padding(d, a.clone(), z.clone(), n.clone(), out),
                    ac(
                        d,
                        "SetElements",
                        vec![
                            a.clone(),
                            p.clone(),
                            f.clone(),
                            i.clone(),
                            x.clone(),
                            ac(
                                d,
                                "ValidElements",
                                vec![
                                    a.clone(),
                                    p.clone(),
                                    z.clone(),
                                    n.clone(),
                                    f.clone(),
                                    var("h"),
                                ],
                            ),
                            var("hx"),
                        ],
                    ),
                    ac(
                        d,
                        "SetPadding",
                        vec![
                            a.clone(),
                            z.clone(),
                            n.clone(),
                            f.clone(),
                            i.clone(),
                            x.clone(),
                            ac(d, "ValidPadding", vec![a, p, z, n, f, var("h")]),
                            var("within"),
                        ],
                    ),
                ],
            ),
            true,
        )?;
        self.update_reads(d)
    }
    fn update_reads(&mut self, d: usize) -> Result<()> {
        let (a, n, f, i, j, x) = (var("A"), var("n"), var("f"), var("i"), var("j"), var("x"));
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("x", a.clone()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(i.clone()), n.clone())),
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
        )
        .reverse();
        let same = nc(
            "EqualComplete",
            vec![idx(d, at.clone()), value(i.clone()), e.proof],
        );
        let mut set_args = vec![a.clone(), f.clone(), i.clone(), x.clone()];
        set_args.extend(at.clone());
        set_args.push(same);
        self.theorem(
            d,
            "ReadSetSame",
            &args,
            Equality {
                ty: a.clone(),
                left: read(
                    d,
                    a.clone(),
                    updated(d, a.clone(), f.clone(), i.clone(), x.clone()),
                    i.clone(),
                ),
                right: slot(updated(d, a.clone(), f.clone(), i.clone(), x.clone()), at),
                proof: ac(
                    d,
                    "ReadWithinSlot",
                    vec![
                        a.clone(),
                        n.clone(),
                        updated(d, a.clone(), f.clone(), i.clone(), x.clone()),
                        i.clone(),
                        var("shape"),
                        var("within"),
                    ],
                ),
            }
            .then(Equality {
                ty: a.clone(),
                left: slot(
                    updated(d, a.clone(), f.clone(), i.clone(), x.clone()),
                    indices(i.clone(), d)?,
                ),
                right: x.clone(),
                proof: ac(d, "SetSameSlot", set_args),
            })?,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("j", word()),
            ("x", a.clone()),
            ("shape", shape(d, n.clone())),
            ("jin", inside(value(j.clone()), n.clone())),
            (
                "different",
                truth(equal(value(j.clone()), value(i.clone())), false),
            ),
        ];
        let at = indices(j.clone(), d)?;
        let e = Equality::natural(
            value(j.clone()),
            idx(d, at.clone()),
            ix(
                d,
                "BoundedIndex",
                vec![
                    j.clone(),
                    bounded(d, n.clone(), j.clone(), var("shape"), var("jin")),
                ],
            ),
        )
        .reverse();
        let changed = e
            .mapped(
                lam("_n", nat(), equal(var("_n"), value(i.clone()))),
                equal(idx(d, at.clone()), value(i.clone())),
                equal(value(j.clone()), value(i.clone())),
                boolean(),
            )
            .then(Equality::boolean(
                equal(value(j.clone()), value(i.clone())),
                bit(false),
                var("different"),
            ))?;
        let mut set_args = vec![a.clone(), f.clone(), i.clone(), x.clone()];
        set_args.extend(at.clone());
        set_args.push(changed.proof);
        let updated_f = updated(d, a.clone(), f.clone(), i.clone(), x);
        self.theorem(
            d,
            "ReadSetOther",
            &args,
            Equality {
                ty: a.clone(),
                left: read(d, a.clone(), updated_f.clone(), j.clone()),
                right: slot(updated_f.clone(), at.clone()),
                proof: ac(
                    d,
                    "ReadWithinSlot",
                    vec![
                        a.clone(),
                        n.clone(),
                        updated_f,
                        j.clone(),
                        var("shape"),
                        var("jin"),
                    ],
                ),
            }
            .then(Equality {
                ty: a.clone(),
                left: slot(updated(d, a.clone(), f.clone(), i, var("x")), at.clone()),
                right: slot(f.clone(), at.clone()),
                proof: ac(d, "SetOtherSlot", set_args),
            })?
            .then(
                Equality {
                    ty: a.clone(),
                    left: read(d, a.clone(), f.clone(), j.clone()),
                    right: slot(f.clone(), at),
                    proof: ac(
                        d,
                        "ReadWithinSlot",
                        vec![a, n, f, j, var("shape"), var("jin")],
                    ),
                }
                .reverse(),
            )?,
        )
    }
}
