//! Heterogeneous element relations avoid equality or extensionality assumptions.
use super::normalization::normalized;
use super::*;
fn related(d: usize, a: Expr, b: Expr, r: Expr, n: Expr, f: Expr, g: Expr) -> Expr {
    ac(d, "Related", vec![a, b, r, n, f, g])
}
fn relation(a: Expr, b: Expr) -> Expr {
    pi("_x", a, pi("_y", b, Expr::Sort0))
}
fn holds(r: Expr, x: Expr, y: Expr) -> Expr {
    app(r, vec![x, y])
}
fn exposed(d: usize, r: Expr, n: Expr, f: Expr, g: Expr) -> Expr {
    cube(
        d,
        pi(
            "_in",
            inside(idx(d, address(d)), n),
            holds(r, slot(f, address(d)), slot(g, address(d))),
        ),
        false,
    )
}
fn points(d: usize, n: Expr, body: Expr) -> Expr {
    cube(d, lam("_in", inside(idx(d, address(d)), n), body), true)
}
fn transfer(a: Expr, b: Expr, r: Expr, x: Equality, y: Equality, h: Expr) -> Expr {
    let left = retype(
        a.clone(),
        lam(
            "_x",
            a.clone(),
            holds(r.clone(), var("_x"), y.right.clone()),
        ),
        x.clone().reverse(),
        h,
    );
    retype(
        b.clone(),
        lam("_y", b, holds(r, x.left, var("_y"))),
        y.reverse(),
        left,
    )
}
impl Arrays<'_> {
    pub(super) fn relations(&mut self, d: usize) -> Result<()> {
        let (a, b, r, n, f, g) = (var("A"), var("B"), var("R"), var("n"), var("f"), var("g"));
        let base = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("R", relation(a.clone(), b.clone())),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("g", cube(d, b.clone(), false)),
        ];
        self.emit(
            d,
            "Related",
            &base,
            Expr::Sort0,
            exposed(d, r.clone(), n.clone(), f.clone(), g.clone()),
            false,
        )?;
        let mut args = base.to_vec();
        args.push((
            "h",
            related(
                d,
                a.clone(),
                b.clone(),
                r.clone(),
                n.clone(),
                f.clone(),
                g.clone(),
            ),
        ));
        self.emit(
            d,
            "RelatedAsFunction",
            &args,
            exposed(d, r.clone(), n.clone(), f.clone(), g.clone()),
            var("h"),
            false,
        )?;
        let mut slot_args = args.clone();
        slot_args.extend(BITS[..d].iter().map(|name| (*name, boolean())));
        slot_args.push(("within", inside(idx(d, address(d)), n.clone())));
        self.emit(
            d,
            "RelatedSlot",
            &slot_args,
            holds(
                r.clone(),
                slot(f.clone(), address(d)),
                slot(g.clone(), address(d)),
            ),
            app(
                slot(
                    ac(
                        d,
                        "RelatedAsFunction",
                        vec![
                            a.clone(),
                            b.clone(),
                            r.clone(),
                            n.clone(),
                            f.clone(),
                            g.clone(),
                            var("h"),
                        ],
                    ),
                    address(d),
                ),
                vec![var("within")],
            ),
            true,
        )?;
        let mut read_args = args.clone();
        read_args.extend([
            ("i", word()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(var("i")), n.clone())),
        ]);
        let i = var("i");
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
        let hin = nl(e.reverse(), n.clone()).then(Equality::boolean(
            less(value(i.clone()), n.clone()),
            bit(true),
            var("within"),
        ))?;
        let mut supplied = vec![
            a.clone(),
            b.clone(),
            r.clone(),
            n.clone(),
            f.clone(),
            g.clone(),
            var("h"),
        ];
        supplied.extend(at.clone());
        supplied.push(hin.proof);
        let read_eq = |ty: Expr, data: Expr| Equality {
            ty: ty.clone(),
            left: read(d, ty.clone(), data.clone(), i.clone()),
            right: slot(data.clone(), at.clone()),
            proof: ac(
                d,
                "ReadWithinSlot",
                vec![ty, n.clone(), data, i.clone(), var("shape"), var("within")],
            ),
        };
        self.emit(
            d,
            "RelatedRead",
            &read_args,
            holds(
                r.clone(),
                read(d, a.clone(), f.clone(), i.clone()),
                read(d, b.clone(), g.clone(), i.clone()),
            ),
            transfer(
                a.clone(),
                b.clone(),
                r.clone(),
                read_eq(a.clone(), f.clone()),
                read_eq(b.clone(), g.clone()),
                ac(d, "RelatedSlot", supplied),
            ),
            true,
        )?;
        self.related_updates(d, &base, &args)?;
        let mut norm_args = args;
        norm_args.extend([("za", a.clone()), ("zb", b.clone())]);
        let at = address(d);
        let norm_eq = |ty: Expr, z: Expr, data: Expr| {
            let mut arg = vec![ty.clone(), z.clone(), n.clone(), data.clone()];
            arg.extend(at.clone());
            arg.push(var("_in"));
            Equality {
                ty: ty.clone(),
                left: slot(normalized(d, ty, z, n.clone(), data.clone()), at.clone()),
                right: slot(data, at.clone()),
                proof: ac(d, "NormalizeInside", arg),
            }
        };
        let mut witness = vec![
            a.clone(),
            b.clone(),
            r.clone(),
            n.clone(),
            f.clone(),
            g.clone(),
            var("h"),
        ];
        witness.extend(at);
        witness.push(var("_in"));
        self.emit(
            d,
            "RelatedNormalize",
            &norm_args,
            related(
                d,
                a.clone(),
                b.clone(),
                r.clone(),
                n.clone(),
                normalized(d, a.clone(), var("za"), n.clone(), f.clone()),
                normalized(d, b.clone(), var("zb"), n.clone(), g.clone()),
            ),
            points(
                d,
                n.clone(),
                transfer(
                    a.clone(),
                    b.clone(),
                    r,
                    norm_eq(a, var("za"), f),
                    norm_eq(b, var("zb"), g),
                    ac(d, "RelatedSlot", witness),
                ),
            ),
            true,
        )
    }
    fn related_updates(&mut self, d: usize, base: &[Binder], observed: &[Binder]) -> Result<()> {
        let (a, b, r, n, f, g, i, x, y) = (
            var("A"),
            var("B"),
            var("R"),
            var("n"),
            var("f"),
            var("g"),
            var("i"),
            var("x"),
            var("y"),
        );
        let mut args = observed.to_vec();
        args.extend([
            ("i", word()),
            ("x", a.clone()),
            ("y", b.clone()),
            ("xy", holds(r.clone(), x.clone(), y.clone())),
        ]);
        let at = address(d);
        let flag = equal(idx(d, at.clone()), value(i.clone()));
        let changed = |ty: Expr, data: Expr, v: Expr| ac(d, "Set", vec![ty, data, i.clone(), v]);
        let choose_eq = |ty: Expr, data: Expr, v: Expr, yes| Equality {
            ty: ty.clone(),
            left: slot(changed(ty.clone(), data.clone(), v.clone()), at.clone()),
            right: if yes {
                v.clone()
            } else {
                slot(data.clone(), at.clone())
            },
            proof: seq(
                if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                vec![ty, flag.clone(), v, slot(data, at.clone()), var("_case")],
            ),
        };
        let mut supplied = base.iter().map(|(name, _)| var(name)).collect::<Vec<_>>();
        supplied.push(var("h"));
        supplied.extend(at.clone());
        supplied.push(var("_in"));
        let no = transfer(
            a.clone(),
            b.clone(),
            r.clone(),
            choose_eq(a.clone(), f.clone(), x.clone(), false),
            choose_eq(b.clone(), g.clone(), y.clone(), false),
            ac(d, "RelatedSlot", supplied),
        );
        let yes = transfer(
            a.clone(),
            b.clone(),
            r.clone(),
            choose_eq(a.clone(), f.clone(), x.clone(), true),
            choose_eq(b.clone(), g.clone(), y.clone(), true),
            var("xy"),
        );
        let fs = changed(a.clone(), f, x);
        let gs = changed(b.clone(), g, y);
        self.emit(
            d,
            "RelatedSet",
            &args,
            related(d, a, b, r.clone(), n.clone(), fs.clone(), gs.clone()),
            points(
                d,
                n,
                decide(flag, holds(r, slot(fs, at.clone()), slot(gs, at)), no, yes),
            ),
            true,
        )
    }
}
