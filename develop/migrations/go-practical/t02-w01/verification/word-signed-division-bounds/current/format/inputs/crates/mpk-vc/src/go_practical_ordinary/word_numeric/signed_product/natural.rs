//! Genuine unbounded Nat cancellation and positivity; no machine rule.
use super::*;

fn relation(a: Expr, b: Expr, m: Expr) -> Expr {
    pi(
        "_equation",
        eq(nat(), mul(m.clone(), a.clone()), mul(m, b.clone())),
        eq(nat(), a, b),
    )
}
fn numeral(n: usize) -> Expr {
    (0..n).fold(zero(), |n, _| succ(n))
}
fn power(n: Expr) -> Expr {
    numeric("PowerTwo", vec![n])
}
fn nlaw(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}

impl SignedProduct<'_> {
    pub(super) fn natural_laws(&mut self) -> Result<()> {
        let (a, n) = (var("A"), var("n"));
        let false_ = Equality::boolean(
            equal(succ(n.clone()), zero()),
            bit(false),
            nc("EqualSuccZero", vec![n.clone()]),
        );
        let contradiction = false_.reverse().then(Equality::boolean(
            equal(succ(n.clone()), zero()),
            bit(true),
            nc(
                "EqualComplete",
                vec![succ(n.clone()), zero(), var("equation")],
            ),
        ))?;
        self.emit(
            "SuccessorZeroElim",
            &[
                ("A", Expr::Sort0),
                ("n", nat()),
                ("equation", eq(nat(), succ(n), zero())),
            ],
            a.clone(),
            bc("FalseElim", vec![a, contradiction.proof]),
            true,
        )?;
        self.zero_sum()?;
        self.product_successor_zero()?;
        self.multiply_cancel()?;
        self.positive_powers()
    }
    fn zero_sum(&mut self) -> Result<()> {
        let (a, b, n) = (var("a"), var("b"), var("_n"));
        let prop = |x: Expr| {
            pi(
                "_equation",
                eq(nat(), add(a.clone(), x.clone()), zero()),
                eq(nat(), x, zero()),
            )
        };
        let e = nlaw(
            "AddSucc",
            vec![a.clone(), n.clone()],
            add(a.clone(), succ(n.clone())),
            succ(add(a.clone(), n.clone())),
        )
        .reverse()
        .then(Equality::natural(
            add(a.clone(), succ(n.clone())),
            zero(),
            var("_equation"),
        ))?;
        let step = lam(
            "_n",
            nat(),
            lam(
                "_ignored",
                prop(n.clone()),
                lam(
                    "_equation",
                    eq(nat(), add(a.clone(), succ(n.clone())), zero()),
                    sc(
                        "SuccessorZeroElim",
                        vec![
                            eq(nat(), succ(n.clone()), zero()),
                            add(a.clone(), n.clone()),
                            e.proof,
                        ],
                    ),
                ),
            ),
        );
        let proof = induct(
            "_n",
            prop(n.clone()),
            lam(
                "_equation",
                eq(nat(), add(a.clone(), zero()), zero()),
                refl(nat(), zero()),
            ),
            step,
            b.clone(),
        );
        self.emit(
            "ZeroSumRight",
            &[("a", nat()), ("b", nat())],
            prop(b.clone()),
            proof,
            true,
        )?;
        let e = nlaw(
            "AddComm",
            vec![b.clone(), a.clone()],
            add(b.clone(), a.clone()),
            add(a.clone(), b.clone()),
        )
        .then(Equality::natural(
            add(a.clone(), b.clone()),
            zero(),
            var("equation"),
        ))?;
        self.theorem(
            "ZeroSumLeft",
            &[
                ("a", nat()),
                ("b", nat()),
                ("equation", eq(nat(), add(a.clone(), b.clone()), zero())),
            ],
            instance("ZeroSumRight", vec![b, a.clone(), e.proof], a, zero()),
        )
    }
    fn product_successor_zero(&mut self) -> Result<()> {
        let (a, m, n) = (var("A"), var("modulus"), var("n"));
        let e = nlaw(
            "MulSucc",
            vec![m.clone(), n.clone()],
            mul(m.clone(), succ(n.clone())),
            add(mul(m.clone(), n.clone()), m.clone()),
        )
        .reverse()
        .then(Equality::natural(
            mul(m.clone(), succ(n.clone())),
            zero(),
            var("equation"),
        ))?;
        let mz = instance(
            "ZeroSumRight",
            vec![mul(m.clone(), n.clone()), m.clone(), e.proof],
            m.clone(),
            zero(),
        );
        let contradiction = less_right(zero(), mz)
            .then(Equality::boolean(
                less(zero(), zero()),
                bit(false),
                nc("LessZero", vec![zero()]),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(zero(), m.clone()),
                bit(true),
                var("positive"),
            ))?;
        self.emit(
            "ProductSuccessorZeroElim",
            &[
                ("A", Expr::Sort0),
                ("modulus", nat()),
                ("n", nat()),
                (
                    "positive",
                    eq(boolean(), less(zero(), m.clone()), bit(true)),
                ),
                ("equation", eq(nat(), mul(m, succ(n)), zero())),
            ],
            a.clone(),
            bc("FalseElim", vec![a, contradiction.proof]),
            true,
        )
    }
    fn multiply_cancel(&mut self) -> Result<()> {
        let (m, a, b, n, k, other) = (
            var("modulus"),
            var("a"),
            var("b"),
            var("_outer"),
            var("_inner"),
            var("_other"),
        );
        let zero_mul = nlaw("MulZero", vec![m.clone()], mul(m.clone(), zero()), zero());
        let e = Equality::natural(
            mul(m.clone(), zero()),
            mul(m.clone(), succ(k.clone())),
            var("_equation"),
        )
        .reverse()
        .then(zero_mul.clone())?;
        let base_step = lam(
            "_inner",
            nat(),
            lam(
                "_ignored",
                relation(zero(), k.clone(), m.clone()),
                lam(
                    "_equation",
                    eq(
                        nat(),
                        mul(m.clone(), zero()),
                        mul(m.clone(), succ(k.clone())),
                    ),
                    sc(
                        "ProductSuccessorZeroElim",
                        vec![
                            eq(nat(), zero(), succ(k.clone())),
                            m.clone(),
                            k.clone(),
                            var("positive"),
                            e.proof,
                        ],
                    ),
                ),
            ),
        );
        let base = lam(
            "_other",
            nat(),
            induct(
                "_inner",
                relation(zero(), k.clone(), m.clone()),
                lam(
                    "_equation",
                    eq(nat(), mul(m.clone(), zero()), mul(m.clone(), zero())),
                    refl(nat(), zero()),
                ),
                base_step,
                other.clone(),
            ),
        );
        let property = pi(
            "_other",
            nat(),
            relation(n.clone(), other.clone(), m.clone()),
        );
        let suc0 = Equality::natural(
            mul(m.clone(), succ(n.clone())),
            mul(m.clone(), zero()),
            var("_equation"),
        )
        .then(zero_mul)?;
        let inner_base = lam(
            "_equation",
            eq(
                nat(),
                mul(m.clone(), succ(n.clone())),
                mul(m.clone(), zero()),
            ),
            sc(
                "ProductSuccessorZeroElim",
                vec![
                    eq(nat(), succ(n.clone()), zero()),
                    m.clone(),
                    n.clone(),
                    var("positive"),
                    suc0.proof,
                ],
            ),
        );
        let e = nlaw(
            "MulSucc",
            vec![m.clone(), n.clone()],
            mul(m.clone(), succ(n.clone())),
            add(mul(m.clone(), n.clone()), m.clone()),
        )
        .reverse()
        .then(Equality::natural(
            mul(m.clone(), succ(n.clone())),
            mul(m.clone(), succ(k.clone())),
            var("_equation"),
        ))?
        .then(nlaw(
            "MulSucc",
            vec![m.clone(), k.clone()],
            mul(m.clone(), succ(k.clone())),
            add(mul(m.clone(), k.clone()), m.clone()),
        ))?;
        let cancelled = nc(
            "AddCancel",
            vec![
                mul(m.clone(), n.clone()),
                mul(m.clone(), k.clone()),
                m.clone(),
                e.proof,
            ],
        );
        let step = lam(
            "_inner",
            nat(),
            lam(
                "_ignored",
                relation(succ(n.clone()), k.clone(), m.clone()),
                lam(
                    "_equation",
                    eq(
                        nat(),
                        mul(m.clone(), succ(n.clone())),
                        mul(m.clone(), succ(k.clone())),
                    ),
                    Equality::natural(
                        n.clone(),
                        k.clone(),
                        app(var("_prior"), vec![k.clone(), cancelled]),
                    )
                    .on_succ()
                    .proof,
                ),
            ),
        );
        let update = lam(
            "_outer",
            nat(),
            lam(
                "_prior",
                property.clone(),
                lam(
                    "_other",
                    nat(),
                    induct(
                        "_inner",
                        relation(succ(n.clone()), k, m.clone()),
                        inner_base,
                        step,
                        other.clone(),
                    ),
                ),
            ),
        );
        self.emit(
            "MultiplyCancelPositive",
            &[
                ("modulus", nat()),
                (
                    "positive",
                    eq(boolean(), less(zero(), m.clone()), bit(true)),
                ),
                ("a", nat()),
                ("b", nat()),
            ],
            relation(a.clone(), b.clone(), m),
            app(induct("_outer", property, base, update, a), vec![b]),
            true,
        )
    }
    fn positive_powers(&mut self) -> Result<()> {
        let (a, n) = (var("a"), var("_n"));
        let property = |a: Expr| {
            pi(
                "_positive",
                eq(boolean(), less(zero(), a.clone()), bit(true)),
                eq(boolean(), less(zero(), mul(two(), a)), bit(true)),
            )
        };
        let contradiction = Equality::boolean(
            less(zero(), zero()),
            bit(false),
            nc("LessZero", vec![zero()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(zero(), zero()),
            bit(true),
            var("_positive"),
        ))?;
        let base = lam(
            "_positive",
            eq(boolean(), less(zero(), zero()), bit(true)),
            bc(
                "FalseElim",
                vec![
                    eq(boolean(), less(zero(), mul(two(), zero())), bit(true)),
                    contradiction.proof,
                ],
            ),
        );
        let p = self
            .polynomial(
                &[("_n", nat())],
                mul(two(), succ(n.clone())),
                add(succ(n.clone()), succ(n.clone())),
            )?
            .then(nlaw(
                "SuccAdd",
                vec![n.clone(), succ(n.clone())],
                add(succ(n.clone()), succ(n.clone())),
                succ(add(n.clone(), succ(n.clone()))),
            ))?;
        let p = less_right(zero(), p).then(Equality::boolean(
            less(zero(), succ(add(n.clone(), succ(n.clone())))),
            bit(true),
            nc("LessZeroSucc", vec![add(n.clone(), succ(n.clone()))]),
        ))?;
        let step = lam(
            "_n",
            nat(),
            lam(
                "_ignored",
                property(n.clone()),
                lam(
                    "_positive",
                    eq(boolean(), less(zero(), succ(n.clone())), bit(true)),
                    p.proof,
                ),
            ),
        );
        self.emit(
            "PositiveDouble",
            &[("a", nat())],
            property(a.clone()),
            induct("_n", property(n.clone()), base, step, a),
            true,
        )?;
        let prop = |n: Expr| eq(boolean(), less(zero(), power(n)), bit(true));
        let base = less_right(
            zero(),
            Equality::natural(power(zero()), one(), numeric("PowerTwoZero", vec![])),
        )
        .then(Equality::boolean(
            less(zero(), one()),
            bit(true),
            nc("LessZeroSucc", vec![zero()]),
        ))?;
        let step = less_right(
            zero(),
            Equality::natural(
                power(succ(n.clone())),
                mul(two(), power(n.clone())),
                numeric("PowerTwoSucc", vec![n.clone()]),
            ),
        )
        .then(Equality::boolean(
            less(zero(), mul(two(), power(n.clone()))),
            bit(true),
            sc("PositiveDouble", vec![power(n.clone()), var("_prior")]),
        ))?;
        self.emit(
            "PowerTwoPositive",
            &[("n", nat())],
            prop(var("n")),
            induct(
                "_n",
                prop(n.clone()),
                base.proof,
                lam("_n", nat(), lam("_prior", prop(n), step.proof)),
                var("n"),
            ),
            true,
        )?;
        for width in WIDTHS {
            let d = depth(width)?;
            let bridge = Equality::natural(
                power(numeral(width)),
                weight(d),
                numeric(&format!("PowerTwoWeight{d}"), vec![]),
            );
            let e = less_right(zero(), bridge.reverse()).then(Equality::boolean(
                less(zero(), power(numeral(width))),
                bit(true),
                sc("PowerTwoPositive", vec![numeral(width)]),
            ))?;
            self.theorem(&format!("W{width}.ModulusPositive"), &[], e)?;
        }
        Ok(())
    }
}
