//! Arbitrary-Nat complement and carry consequences, plus local flag cases.
use super::*;

fn sum(a: Expr, b: Expr, m: Expr, c: Expr) -> Expr {
    add(add(a, b), mul(m, bit_value(c)))
}
fn nlaw(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}
impl Magnitude<'_> {
    pub(super) fn natural_laws(&mut self) -> Result<()> {
        self.positive_complement_nonzero()?;
        self.carry_laws()?;
        self.sign_nonzero()?;
        self.magnitude_cross()
    }
    fn positive_complement_nonzero(&mut self) -> Result<()> {
        let (a, b, m, n) = (var("a"), var("b"), var("modulus"), var("_n"));
        let property = |a: Expr| {
            pi(
                "_equation",
                eq(nat(), add(a.clone(), b.clone()), m.clone()),
                eq(boolean(), equal(a, zero()), bit(false)),
            )
        };
        let bm = nlaw(
            "ZeroAdd",
            vec![b.clone()],
            add(zero(), b.clone()),
            b.clone(),
        )
        .reverse()
        .then(Equality::natural(
            add(zero(), b.clone()),
            m.clone(),
            var("_equation"),
        ))?;
        let bad = less_right(b.clone(), bm.reverse())
            .reverse()
            .then(Equality::boolean(
                less(b.clone(), m.clone()),
                bit(true),
                var("bound"),
            ))?;
        let contradiction = Equality::boolean(
            less(b.clone(), b.clone()),
            bit(false),
            nc("LessIrreflexive", vec![b.clone()]),
        )
        .reverse()
        .then(bad)?;
        let base = lam(
            "_equation",
            eq(nat(), add(zero(), b.clone()), m.clone()),
            bc(
                "FalseElim",
                vec![
                    eq(boolean(), equal(zero(), zero()), bit(false)),
                    contradiction.proof,
                ],
            ),
        );
        let step = lam(
            "_n",
            nat(),
            lam(
                "_ignored",
                property(n.clone()),
                lam(
                    "_equation",
                    eq(nat(), add(succ(n.clone()), b.clone()), m.clone()),
                    nc("EqualSuccZero", vec![n.clone()]),
                ),
            ),
        );
        self.emit(
            "PositiveComplementNonzero",
            &[
                ("a", nat()),
                ("b", nat()),
                ("modulus", nat()),
                (
                    "bound",
                    eq(boolean(), less(b.clone(), m.clone()), bit(true)),
                ),
            ],
            property(a.clone()),
            induct("_n", property(n), base, step, a),
            true,
        )
    }
    fn carry_literal(
        &mut self,
        args: &[Binder],
        a: Expr,
        b: Expr,
        m: Expr,
        flag: bool,
    ) -> Result<Equality> {
        let t = add(a.clone(), b.clone());
        let first = mul_right(m.clone(), bit_literal(flag)).on_right(t.clone());
        let next = self.polynomial(args, first.right.clone(), if flag { add(t, m) } else { t })?;
        first.then(next)
    }
    fn carry_zero(&mut self, args: &[Binder], a: Expr, b: Expr, m: Expr) -> Result<Equality> {
        let t = add(a.clone(), b.clone());
        let e = self
            .carry_literal(args, a, b, m.clone(), true)?
            .reverse()
            .then(Equality::natural(
                sum(var("a"), var("b"), m.clone(), bit(true)),
                m.clone(),
                var("_equation"),
            ))?
            .then(
                nlaw(
                    "ZeroAdd",
                    vec![m.clone()],
                    add(zero(), m.clone()),
                    m.clone(),
                )
                .reverse(),
            )?;
        Ok(Equality::natural(
            t.clone(),
            zero(),
            nc("AddCancel", vec![t, zero(), m, e.proof]),
        ))
    }
    fn carry_laws(&mut self) -> Result<()> {
        let (a, b, m) = (var("a"), var("b"), var("modulus"));
        for name in ["SumCarryIsZero", "SumCarryComplement"] {
            let is_zero = name == "SumCarryIsZero";
            let prop = |env: &BTreeMap<&'static str, Expr>| {
                let c = env["carry"].clone();
                pi(
                    "_equation",
                    eq(
                        nat(),
                        sum(a.clone(), b.clone(), m.clone(), c.clone()),
                        m.clone(),
                    ),
                    if is_zero {
                        eq(boolean(), c, equal(a.clone(), zero()))
                    } else {
                        eq(
                            nat(),
                            add(a.clone(), b.clone()),
                            mul(m.clone(), bit_value(bc("Not", vec![c]))),
                        )
                    },
                )
            };
            let mut args = vec![("a", nat()), ("b", nat()), ("modulus", nat())];
            if is_zero {
                args.push((
                    "bound",
                    eq(boolean(), less(b.clone(), m.clone()), bit(true)),
                ));
            }
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let flag = env["carry"] == bit(true);
                let mut local = args.clone();
                local.push((
                    "_equation",
                    eq(
                        nat(),
                        sum(a.clone(), b.clone(), m.clone(), bit(flag)),
                        m.clone(),
                    ),
                ));
                let e = if flag {
                    self.carry_zero(&local, a.clone(), b.clone(), m.clone())?
                } else {
                    self.carry_literal(&local, a.clone(), b.clone(), m.clone(), false)?
                        .reverse()
                        .then(Equality::natural(
                            sum(a.clone(), b.clone(), m.clone(), bit(false)),
                            m.clone(),
                            var("_equation"),
                        ))?
                };
                let proof = if is_zero {
                    if flag {
                        let az = shared("ZeroSumLeft", vec![a.clone(), b.clone(), e.proof]);
                        Equality::boolean(
                            equal(a.clone(), zero()),
                            bit(true),
                            nc("EqualComplete", vec![a.clone(), zero(), az]),
                        )
                        .reverse()
                        .proof
                    } else {
                        Equality::boolean(
                            equal(a.clone(), zero()),
                            bit(false),
                            mc(
                                "PositiveComplementNonzero",
                                vec![a.clone(), b.clone(), m.clone(), var("bound"), e.proof],
                            ),
                        )
                        .reverse()
                        .proof
                    }
                } else {
                    let n = if flag { zero() } else { one() };
                    let normalized = mul_right(
                        m.clone(),
                        Equality::natural(
                            bit_value(bc("Not", vec![bit(flag)])),
                            n.clone(),
                            refl(nat(), n),
                        ),
                    );
                    let tail =
                        self.polynomial(&local, normalized.right.clone(), e.right.clone())?;
                    e.then(normalized.then(tail)?.reverse())?.proof
                };
                Ok(lam(
                    "_equation",
                    eq(
                        nat(),
                        sum(a.clone(), b.clone(), m.clone(), bit(flag)),
                        m.clone(),
                    ),
                    proof,
                ))
            };
            let proof = boolean_cases(&["carry"], &BTreeMap::new(), &prop, &mut leaf)?;
            self.emit(
                name,
                &args,
                pi(
                    "carry",
                    boolean(),
                    prop(&[("carry", var("carry"))].into_iter().collect()),
                ),
                proof,
                true,
            )?;
        }
        Ok(())
    }
    fn sign_nonzero(&mut self) -> Result<()> {
        let s = var("sign");
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let z = env["zero"].clone();
            pi(
                "_zero_sign",
                pi(
                    "_zero",
                    eq(boolean(), z.clone(), bit(true)),
                    eq(boolean(), s.clone(), bit(false)),
                ),
                pi(
                    "_negative",
                    eq(boolean(), s.clone(), bit(true)),
                    eq(boolean(), bc("Not", vec![z]), bit(true)),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let z = env["zero"].clone();
            let result = eq(boolean(), bc("Not", vec![z.clone()]), bit(true));
            let body = if z == bit(false) {
                refl(boolean(), bit(true))
            } else {
                let contradiction = Equality::boolean(
                    s.clone(),
                    bit(false),
                    app(var("_zero_sign"), vec![refl(boolean(), bit(true))]),
                )
                .reverse()
                .then(Equality::boolean(s.clone(), bit(true), var("_negative")))?;
                bc("FalseElim", vec![result, contradiction.proof])
            };
            Ok(lam(
                "_zero_sign",
                pi(
                    "_zero",
                    eq(boolean(), z.clone(), bit(true)),
                    eq(boolean(), s.clone(), bit(false)),
                ),
                lam("_negative", eq(boolean(), s.clone(), bit(true)), body),
            ))
        };
        let proof = boolean_cases(&["zero"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "SignNonzero",
            &[("sign", boolean())],
            pi(
                "zero",
                boolean(),
                prop(&[("zero", var("zero"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn magnitude_cross(&mut self) -> Result<()> {
        let (x, n, m) = (var("ux"), var("un"), var("modulus"));
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let s = env["sign"].clone();
            let abs = choose(s.clone(), n.clone(), x.clone(), nat());
            pi(
                "_negative_balance",
                pi(
                    "_negative",
                    eq(boolean(), s.clone(), bit(true)),
                    eq(nat(), add(x.clone(), n.clone()), m.clone()),
                ),
                eq(
                    nat(),
                    add(x.clone(), choose(s.clone(), abs.clone(), zero(), nat())),
                    add(
                        choose(s.clone(), zero(), abs, nat()),
                        mul(m.clone(), bit_value(s)),
                    ),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["sign"] == bit(true);
            let s = bit(flag);
            let abs = choose(s.clone(), n.clone(), x.clone(), nat());
            let left = add(x.clone(), choose(s.clone(), abs.clone(), zero(), nat()));
            let right = add(
                choose(s.clone(), zero(), abs, nat()),
                mul(m.clone(), bit_value(s.clone())),
            );
            let l = if flag {
                add(x.clone(), n.clone())
            } else {
                add(x.clone(), zero())
            };
            let r = if flag {
                add(zero(), mul(m.clone(), one()))
            } else {
                add(x.clone(), mul(m.clone(), zero()))
            };
            let mut args = vec![("ux", nat()), ("un", nat()), ("modulus", nat())];
            args.push((
                "_negative_balance",
                pi(
                    "_negative",
                    eq(boolean(), s.clone(), bit(true)),
                    eq(nat(), add(x.clone(), n.clone()), m.clone()),
                ),
            ));
            let e = Equality::natural(left, l.clone(), refl(nat(), l.clone()));
            let e = if flag {
                e.then(Equality::natural(
                    l,
                    m.clone(),
                    app(var("_negative_balance"), vec![refl(boolean(), bit(true))]),
                ))?
            } else {
                e
            };
            let tail = self.polynomial(&args, e.right.clone(), r.clone())?;
            Ok(lam(
                "_negative_balance",
                pi(
                    "_negative",
                    eq(boolean(), s, bit(true)),
                    eq(nat(), add(x.clone(), n.clone()), m.clone()),
                ),
                e.then(tail)?
                    .then(Equality::natural(right, r.clone(), refl(nat(), r)).reverse())?
                    .proof,
            ))
        };
        let proof = boolean_cases(&["sign"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "MagnitudeCross",
            &[("ux", nat()), ("un", nat()), ("modulus", nat())],
            pi(
                "sign",
                boolean(),
                prop(&[("sign", var("sign"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
}
