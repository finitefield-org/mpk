//! Total-order equality and positivity are genuine unbounded Nat inductions.
use super::*;

fn total(a: Expr, b: Expr) -> Expr {
    bc(
        "And",
        vec![
            bc("Not", vec![less(a.clone(), b.clone())]),
            bc("Not", vec![less(b, a)]),
        ],
    )
}
fn property(a: Expr, b: Expr) -> Expr {
    eq(boolean(), total(a.clone(), b.clone()), equal(a, b))
}

impl Order<'_> {
    pub(super) fn natural_order(&mut self) -> Result<()> {
        let (a, b, n, k, m) = (
            var("a"),
            var("b"),
            var("_outer"),
            var("_inner"),
            var("_other"),
        );
        let base = lam(
            "_other",
            nat(),
            induct(
                "_inner",
                property(zero(), k.clone()),
                Self::total_literal(false, false, zero())?,
                lam(
                    "_inner",
                    nat(),
                    lam(
                        "_ignored",
                        property(zero(), k.clone()),
                        Self::total_literal(false, true, k.clone())?,
                    ),
                ),
                m.clone(),
            ),
        );
        let outer = pi("_other", nat(), property(n.clone(), m.clone()));
        let low = nc("LessSuccBoth", vec![n.clone(), k.clone()]);
        let high = nc("LessSuccBoth", vec![k.clone(), n.clone()]);
        let left = bool_map(
            "Not",
            Equality::boolean(
                less(succ(n.clone()), succ(k.clone())),
                less(n.clone(), k.clone()),
                low,
            ),
        );
        let right = bool_map(
            "Not",
            Equality::boolean(
                less(succ(k.clone()), succ(n.clone())),
                less(k.clone(), n.clone()),
                high,
            ),
        );
        let e = bool_left("And", left.clone(), right.left.clone())
            .then(bool_right("And", left.right, right))?
            .then(Equality::boolean(
                total(n.clone(), k.clone()),
                equal(n.clone(), k.clone()),
                app(var("_prior"), vec![k.clone()]),
            ))?
            .then(
                Equality::boolean(
                    equal(succ(n.clone()), succ(k.clone())),
                    equal(n.clone(), k.clone()),
                    nc("EqualSuccBoth", vec![n.clone(), k.clone()]),
                )
                .reverse(),
            )?;
        let inner = induct(
            "_inner",
            property(succ(n.clone()), k.clone()),
            Self::total_literal(true, false, n.clone())?,
            lam(
                "_inner",
                nat(),
                lam("_ignored", property(succ(n.clone()), k.clone()), e.proof),
            ),
            m.clone(),
        );
        let step = lam(
            "_outer",
            nat(),
            lam("_prior", outer.clone(), lam("_other", nat(), inner)),
        );
        let proof = app(
            induct("_outer", outer, base, step, a.clone()),
            vec![b.clone()],
        );
        self.theorem(
            "NaturalTotalEquality",
            &[("a", nat()), ("b", nat())],
            Equality::boolean(total(a.clone(), b.clone()), equal(a, b), proof),
        )?;
        let n = var("n");
        let i = var("_index");
        let pos = |x: Expr| {
            eq(
                boolean(),
                bc("Not", vec![equal(x.clone(), zero())]),
                less(zero(), x),
            )
        };
        let p0 = bool_map(
            "Not",
            Equality::boolean(
                equal(zero(), zero()),
                bit(true),
                nc("EqualZeroZero", vec![]),
            ),
        )
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?
        .then(
            Equality::boolean(
                less(zero(), zero()),
                bit(false),
                nc("LessZero", vec![zero()]),
            )
            .reverse(),
        )?;
        let ps = bool_map(
            "Not",
            Equality::boolean(
                equal(succ(i.clone()), zero()),
                bit(false),
                nc("EqualSuccZero", vec![i.clone()]),
            ),
        )
        .then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?
        .then(
            Equality::boolean(
                less(zero(), succ(i.clone())),
                bit(true),
                nc("LessZeroSucc", vec![i.clone()]),
            )
            .reverse(),
        )?;
        let proof = induct(
            "_index",
            pos(i.clone()),
            p0.proof,
            lam("_index", nat(), lam("_ignored", pos(i), ps.proof)),
            n.clone(),
        );
        self.theorem(
            "NaturalNonzeroPositive",
            &[("n", nat())],
            Equality::boolean(
                bc("Not", vec![equal(n.clone(), zero())]),
                less(zero(), n),
                proof,
            ),
        )
    }
    fn total_literal(succ_left: bool, succ_right: bool, index: Expr) -> Result<Expr> {
        let a = if succ_left {
            succ(index.clone())
        } else {
            zero()
        };
        let b = if succ_right {
            succ(index.clone())
        } else {
            zero()
        };
        let ab = if succ_right {
            Equality::boolean(
                less(a.clone(), b.clone()),
                bit(true),
                nc("LessZeroSucc", vec![index.clone()]),
            )
        } else {
            Equality::boolean(
                less(a.clone(), b.clone()),
                bit(false),
                nc("LessZero", vec![a.clone()]),
            )
        };
        let ba = if succ_left {
            Equality::boolean(
                less(b.clone(), a.clone()),
                bit(true),
                nc("LessZeroSucc", vec![index.clone()]),
            )
        } else {
            Equality::boolean(
                less(b.clone(), a.clone()),
                bit(false),
                nc("LessZero", vec![b.clone()]),
            )
        };
        let eqproof = if succ_left {
            nc("EqualSuccZero", vec![index.clone()])
        } else if succ_right {
            nc("EqualZeroSucc", vec![index])
        } else {
            nc("EqualZeroZero", vec![])
        };
        let eqvalue = bit(!succ_left && !succ_right);
        let left = bool_map("Not", ab).then(Equality::boolean(
            bc("Not", vec![bit(succ_right)]),
            bit(!succ_right),
            refl(boolean(), bit(!succ_right)),
        ))?;
        let right = bool_map("Not", ba).then(Equality::boolean(
            bc("Not", vec![bit(succ_left)]),
            bit(!succ_left),
            refl(boolean(), bit(!succ_left)),
        ))?;
        let result = bool_left("And", left.clone(), right.left.clone())
            .then(bool_right("And", left.right, right))?;
        let last = Equality::boolean(
            result.right.clone(),
            eqvalue.clone(),
            refl(boolean(), eqvalue.clone()),
        );
        Ok(result
            .then(last)?
            .then(Equality::boolean(equal(a, b), eqvalue, eqproof).reverse())?
            .proof)
    }
}
