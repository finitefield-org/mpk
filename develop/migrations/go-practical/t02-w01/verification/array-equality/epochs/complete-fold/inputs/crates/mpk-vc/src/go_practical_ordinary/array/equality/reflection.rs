//! Every visible slot is related iff its actual comparator fold is true.
use super::*;
impl Comparisons<'_> {
    pub(super) fn reflection(&mut self, d: usize) -> Result<()> {
        let (a, p, r, f, g, c) = (var("A"), var("P"), var("R"), var("f"), var("g"), var("cmp"));
        let mut args = vec![
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("R", relation(a.clone())),
        ];
        args.extend(actual(d).into_iter().skip(1));
        args.extend([
            ("fp", elements(d, p.clone(), f.clone())),
            ("gp", elements(d, p.clone(), g.clone())),
        ]);
        let at = address(d);
        let flag = less(idx(d, at.clone()), var("n"));
        let x = slot(f.clone(), at.clone());
        let y = slot(g.clone(), at.clone());
        let mut slots = value_args(f.clone(), g.clone());
        slots.extend(at.clone());
        slots.push(var("_in"));
        let unfold = Equality::boolean(
            slot(flags(d, f.clone(), g.clone()), at.clone()),
            compared(c.clone(), x.clone(), y.clone()),
            qc(d, "FlagInside", slots),
        );
        let mut reflect_args = args.clone();
        reflect_args.extend([
            (
                "reflect",
                element_law(a.clone(), p.clone(), r.clone(), c.clone(), true),
            ),
            ("h", truth(equal_by(d, f.clone(), g.clone()), true)),
        ]);
        let folded = slot(
            ec(d, "Reflect", vec![flags(d, f.clone(), g.clone()), var("h")]),
            at.clone(),
        );
        let proof = app(
            var("reflect"),
            vec![
                x.clone(),
                y.clone(),
                slot(var("fp"), at.clone()),
                slot(var("gp"), at.clone()),
                unfold
                    .clone()
                    .reverse()
                    .then(Equality::boolean(
                        slot(flags(d, f.clone(), g.clone()), at.clone()),
                        bit(true),
                        folded,
                    ))?
                    .proof,
            ],
        );
        self.emit(
            d,
            "Reflect",
            &reflect_args,
            related(d, r.clone(), f.clone(), g.clone()),
            cube(d, lam("_in", truth(flag.clone(), true), proof), true),
            true,
        )?;
        let mut complete_args = args;
        complete_args.extend([
            (
                "complete",
                element_law(a.clone(), p, r.clone(), c.clone(), false),
            ),
            ("h", related(d, r.clone(), f.clone(), g.clone())),
        ]);
        let mut held = vec![
            a.clone(),
            a,
            r.clone(),
            var("n"),
            f.clone(),
            g.clone(),
            var("h"),
        ];
        held.extend(at.clone());
        held.push(var("_case"));
        let complete = app(
            var("complete"),
            vec![
                x,
                y,
                slot(var("fp"), at.clone()),
                slot(var("gp"), at.clone()),
                ac(d, "RelatedSlot", held),
            ],
        );
        let mut selected = value_args(f.clone(), g.clone());
        selected.extend(at.clone());
        selected.push(var("_case"));
        let yes = Equality::boolean(
            slot(flags(d, f.clone(), g.clone()), at.clone()),
            compared(c, slot(f.clone(), at.clone()), slot(g.clone(), at.clone())),
            qc(d, "FlagInside", selected.clone()),
        )
        .then(Equality::boolean(
            compared(
                var("cmp"),
                slot(f.clone(), at.clone()),
                slot(g.clone(), at.clone()),
            ),
            bit(true),
            complete,
        ))?;
        let no = qc(d, "FlagOutside", selected);
        self.theorem(
            d,
            "Complete",
            &complete_args,
            Equality::boolean(
                equal_by(d, f.clone(), g.clone()),
                bit(true),
                ec(
                    d,
                    "Complete",
                    vec![
                        flags(d, f.clone(), g.clone()),
                        cube(
                            d,
                            decide(flag, truth(slot(flags(d, f, g), at), true), no, yes.proof),
                            true,
                        ),
                    ],
                ),
            ),
        )
    }
}
