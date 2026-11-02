//! Unbounded Nat gaps and strict upper bounds, with explicit ordinary proofs.
use super::*;

impl Bounded<'_> {
    pub(super) fn natural_bounds(&mut self) -> Result<()> {
        self.strict_upper()?;
        self.strict_gap_balance()?;
        self.quotient_gap()?;
        self.strict_unequal()?;
        self.less_one_zero()
    }
    fn strict_upper(&mut self) -> Result<()> {
        let (x, y, g, h) = (var("x"), var("y"), var("gap"), var("h"));
        let args = [
            ("x", nat()),
            ("y", nat()),
            ("gap", nat()),
            ("h", nat()),
            (
                "ordered",
                eq(boolean(), less(x.clone(), y.clone()), bit(true)),
            ),
            ("balance", eq(nat(), add(y.clone(), g.clone()), h.clone())),
        ];
        let k = sub(y.clone(), succ(x.clone()));
        let start =
            Equality::natural(add(y.clone(), g.clone()), h.clone(), var("balance")).reverse();
        let split = Equality::natural(
            y.clone(),
            add(add(x.clone(), k.clone()), one()),
            prerequisite(
                "Unsigned",
                "StrictGap",
                vec![x.clone(), y.clone(), var("ordered")],
            ),
        );
        let e = start.then(split.on_left(g.clone()))?;
        let tail = add(k, g);
        let normal = self.polynomial(
            &args,
            e.right.clone(),
            add(x.clone(), add(tail.clone(), one())),
        )?;
        let e = e.then(normal)?.then(
            nlaw(
                "AddOne",
                vec![tail.clone()],
                add(tail.clone(), one()),
                succ(tail.clone()),
            )
            .on_right(x.clone()),
        )?;
        let e = less_right(x.clone(), e).then(less_left(
            nlaw(
                "ZeroAdd",
                vec![x.clone()],
                add(zero(), x.clone()),
                x.clone(),
            )
            .reverse(),
            add(x.clone(), succ(tail.clone())),
        ))?;
        let target = less(add(zero(), x.clone()), add(succ(tail.clone()), x.clone()));
        let swapped = nlaw(
            "AddComm",
            vec![x.clone(), succ(tail.clone())],
            add(x.clone(), succ(tail.clone())),
            add(succ(tail.clone()), x.clone()),
        );
        let e = e.then(less_right(add(zero(), x.clone()), swapped))?;
        if e.right != target {
            return Err(GoOrdinaryError::Linkage);
        }
        self.theorem(
            "StrictUpperFromBalance",
            &args,
            e.then(Equality::boolean(
                target,
                less(zero(), succ(tail.clone())),
                nc("LessTranslate", vec![zero(), succ(tail.clone()), x]),
            ))?
            .then(Equality::boolean(
                less(zero(), succ(tail.clone())),
                bit(true),
                nc("LessZeroSucc", vec![tail]),
            ))?,
        )
    }
    fn strict_gap_balance(&mut self) -> Result<()> {
        let (x, h) = (var("x"), var("h"));
        let k = sub(h.clone(), succ(x.clone()));
        let args = [
            ("x", nat()),
            ("h", nat()),
            (
                "ordered",
                eq(boolean(), less(x.clone(), h.clone()), bit(true)),
            ),
        ];
        let e = nlaw(
            "AddAssoc",
            vec![x.clone(), k.clone(), one()],
            add(add(x.clone(), k.clone()), one()),
            add(x.clone(), add(k, one())),
        )
        .reverse()
        .then(
            Equality::natural(
                h.clone(),
                add(add(x.clone(), sub(h.clone(), succ(x.clone()))), one()),
                prerequisite(
                    "Unsigned",
                    "StrictGap",
                    vec![x.clone(), h.clone(), var("ordered")],
                ),
            )
            .reverse(),
        )?;
        self.theorem("StrictGapBalance", &args, e)
    }
    fn quotient_gap(&mut self) -> Result<()> {
        let (a, d, q, r, g, h) = (var("a"), var("d"), var("q"), var("r"), var("gap"), var("h"));
        let args = [
            ("a", nat()),
            ("d", nat()),
            ("q", nat()),
            ("r", nat()),
            ("gap", nat()),
            ("h", nat()),
            (
                "positive",
                eq(boolean(), less(zero(), d.clone()), bit(true)),
            ),
            (
                "division",
                eq(nat(), a.clone(), add(mul(d.clone(), q.clone()), r.clone())),
            ),
            ("half", eq(nat(), add(a.clone(), g.clone()), h.clone())),
        ];
        let p = sub(d.clone(), one());
        let decomp = Equality::natural(
            d.clone(),
            add(one(), p),
            prerequisite(
                "Unsigned",
                "StrictDecomposition",
                vec![zero(), d.clone(), var("positive")],
            ),
        );
        let e = Equality::natural(add(a.clone(), g.clone()), h.clone(), var("half"))
            .reverse()
            .then(
                Equality::natural(
                    a,
                    add(mul(d.clone(), q.clone()), r.clone()),
                    var("division"),
                )
                .on_left(g.clone()),
            )?
            .then(
                mul_left(decomp, q.clone())
                    .on_left(r.clone())
                    .on_left(g.clone()),
            )?;
        let target = add(q.clone(), qgap(d, q, r, g));
        let normal = self.polynomial(&args, e.right.clone(), target)?;
        self.theorem("QuotientGapBalance", &args, e.then(normal)?.reverse())
    }
    fn strict_unequal(&mut self) -> Result<()> {
        let (x, h) = (var("x"), var("h"));
        let equal_flag = equal(x.clone(), h.clone());
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_literal",
                eq(boolean(), equal_flag.clone(), env["flag"].clone()),
                eq(boolean(), equal_flag.clone(), bit(false)),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["flag"].clone();
            let proof = if flag == bit(false) {
                var("_literal")
            } else {
                let same = Equality::natural(
                    x.clone(),
                    h.clone(),
                    nc("EqualReflect", vec![x.clone(), h.clone(), var("_literal")]),
                );
                let bad = less_left(same, h.clone())
                    .reverse()
                    .then(Equality::boolean(
                        less(x.clone(), h.clone()),
                        bit(true),
                        var("ordered"),
                    ))?;
                let contradiction = Equality::boolean(
                    less(h.clone(), h.clone()),
                    bit(false),
                    nc("LessIrreflexive", vec![h.clone()]),
                )
                .reverse()
                .then(bad)?;
                bc(
                    "FalseElim",
                    vec![
                        eq(boolean(), equal_flag.clone(), bit(false)),
                        contradiction.proof,
                    ],
                )
            };
            Ok(lam(
                "_literal",
                eq(boolean(), equal_flag.clone(), flag),
                proof,
            ))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.theorem(
            "StrictUnequal",
            &[
                ("x", nat()),
                ("h", nat()),
                ("ordered", eq(boolean(), less(x, h), bit(true))),
            ],
            Equality::boolean(
                equal_flag.clone(),
                bit(false),
                app(proof, vec![equal_flag.clone(), refl(boolean(), equal_flag)]),
            ),
        )
    }
    fn less_one_zero(&mut self) -> Result<()> {
        let (n, i) = (var("n"), var("_i"));
        let prop = |n: Expr| {
            pi(
                "_ordered",
                eq(boolean(), less(n.clone(), one()), bit(true)),
                eq(nat(), n, zero()),
            )
        };
        let base = lam(
            "_ordered",
            eq(boolean(), less(zero(), one()), bit(true)),
            refl(nat(), zero()),
        );
        let less_succ = Equality::boolean(
            less(succ(i.clone()), one()),
            less(i.clone(), zero()),
            nc("LessSuccBoth", vec![i.clone(), zero()]),
        )
        .then(Equality::boolean(
            less(i.clone(), zero()),
            bit(false),
            nc("LessZero", vec![i.clone()]),
        ))?;
        let contradiction = less_succ.reverse().then(Equality::boolean(
            less(succ(i.clone()), one()),
            bit(true),
            var("_ordered"),
        ))?;
        let step = lam(
            "_i",
            nat(),
            lam(
                "_ignored",
                prop(i.clone()),
                lam(
                    "_ordered",
                    eq(boolean(), less(succ(i.clone()), one()), bit(true)),
                    bc(
                        "FalseElim",
                        vec![eq(nat(), succ(i), zero()), contradiction.proof],
                    ),
                ),
            ),
        );
        self.emit(
            "LessOneZero",
            &[("n", nat())],
            prop(n.clone()),
            induct("_i", prop(var("_i")), base, step, n),
            true,
        )
    }
}
