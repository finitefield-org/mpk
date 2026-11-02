//! Pointwise comparator equality preserves the fold without function extensionality.
use super::*;
impl Comparisons<'_> {
    pub(super) fn preservation(&mut self, d: usize) -> Result<()> {
        let (f, g, u, v) = (var("f"), var("g"), var("u"), var("v"));
        let a = var("A");
        let at = address(d);
        let inside = less(idx(d, at.clone()), var("n"));
        let left = compared(
            var("cmp"),
            slot(f.clone(), at.clone()),
            slot(g.clone(), at.clone()),
        );
        let right = compared(
            var("cmp"),
            slot(u.clone(), at.clone()),
            slot(v.clone(), at.clone()),
        );
        let mut args = actual(d);
        args.extend([
            ("u", cube(d, a.clone(), false)),
            ("v", cube(d, a, false)),
            (
                "h",
                cube(
                    d,
                    pi(
                        "_in",
                        truth(inside.clone(), true),
                        eq(boolean(), left.clone(), right.clone()),
                    ),
                    false,
                ),
            ),
        ]);
        let mut largs = value_args(f.clone(), g.clone());
        largs.extend(at.clone());
        largs.push(var("_case"));
        let mut rargs = value_args(u.clone(), v.clone());
        rargs.extend(at.clone());
        rargs.push(var("_case"));
        let fl = slot(flags(d, f.clone(), g.clone()), at.clone());
        let fr = slot(flags(d, u.clone(), v.clone()), at.clone());
        let branch = |yes| -> Result<Expr> {
            let l = Equality::boolean(
                fl.clone(),
                if yes { left.clone() } else { bit(true) },
                qc(
                    d,
                    if yes { "FlagInside" } else { "FlagOutside" },
                    largs.clone(),
                ),
            );
            let r = Equality::boolean(
                fr.clone(),
                if yes { right.clone() } else { bit(true) },
                qc(
                    d,
                    if yes { "FlagInside" } else { "FlagOutside" },
                    rargs.clone(),
                ),
            )
            .reverse();
            Ok(if yes {
                l.then(Equality::boolean(
                    left.clone(),
                    right.clone(),
                    app(slot(var("h"), at.clone()), vec![var("_case")]),
                ))?
                .then(r)?
                .proof
            } else {
                l.then(r)?.proof
            })
        };
        self.theorem(
            d,
            "Pointwise",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), g.clone()),
                equal_by(d, u.clone(), v.clone()),
                qc(
                    d,
                    "FoldCongruence",
                    vec![
                        flags(d, f, g),
                        flags(d, u, v),
                        cube(
                            d,
                            decide(inside, eq(boolean(), fl, fr), branch(false)?, branch(true)?),
                            true,
                        ),
                    ],
                ),
            ),
        )
    }
}
