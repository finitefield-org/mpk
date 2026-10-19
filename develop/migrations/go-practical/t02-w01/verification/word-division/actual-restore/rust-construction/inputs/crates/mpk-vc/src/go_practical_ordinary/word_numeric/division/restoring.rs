//! The overflow bit is part of the restoring decision, including large divisors.
use super::*;

fn arguments(h: Expr, c: Expr) -> Vec<Binder> {
    let (r, d, m, s, t, b) = (
        var("r"),
        var("d"),
        var("modulus"),
        var("s"),
        var("t"),
        var("low"),
    );
    vec![
        ("r", nat()),
        ("d", nat()),
        ("modulus", nat()),
        ("s", nat()),
        ("t", nat()),
        ("low", boolean()),
        (
            "_r_bound",
            eq(boolean(), less(r.clone(), d.clone()), bit(true)),
        ),
        (
            "_d_bound",
            eq(boolean(), less(d.clone(), m.clone()), bit(true)),
        ),
        (
            "_s_bound",
            eq(boolean(), less(s.clone(), m.clone()), bit(true)),
        ),
        (
            "_push",
            eq(
                nat(),
                add(s.clone(), mul(m.clone(), bit_value(h))),
                add(mul(two(), r), bit_value(b)),
            ),
        ),
        (
            "_sub",
            eq(
                nat(),
                add(s, m.clone()),
                add(add(t, d), mul(m, bit_value(c))),
            ),
        ),
    ]
}
fn selected(h: Expr, c: Expr) -> Expr {
    choose(bc("Or", vec![h, c]), var("t"), var("s"), nat())
}
fn result(h: Expr, c: Expr, bound: bool) -> Expr {
    let select = selected(h.clone(), c.clone());
    if bound {
        eq(boolean(), less(select, var("d")), bit(true))
    } else {
        eq(
            nat(),
            add(select, mul(var("d"), bit_value(bc("Or", vec![h, c])))),
            add(mul(two(), var("r")), bit_value(var("low"))),
        )
    }
}
fn nat_bit_product(a: Expr, b: bool) -> Result<Equality> {
    let n = if b { one() } else { zero() };
    mul_right(a.clone(), bit_literal(b)).then(Equality::natural(
        mul(a.clone(), n),
        if b { a.clone() } else { zero() },
        nc(if b { "MulOne" } else { "MulZero" }, vec![a]),
    ))
}

impl Division<'_> {
    pub(super) fn restoring_laws(&mut self) -> Result<()> {
        for high in [false, true] {
            for carry in [false, true] {
                self.restoring_branch(high, carry)?;
            }
        }
        for bound in [false, true] {
            let property = |e: &BTreeMap<&'static str, Expr>| {
                let mut ty = result(e["high"].clone(), e["carry"].clone(), bound);
                for (n, d) in arguments(e["high"].clone(), e["carry"].clone())[6..]
                    .iter()
                    .rev()
                {
                    ty = pi(n, d.clone(), ty);
                }
                ty
            };
            let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let h = e["high"] == bit(true);
                let c = e["carry"] == bit(true);
                let args = arguments(bit(h), bit(c));
                let name = format!(
                    "Restore{}{}{}",
                    u8::from(h),
                    u8::from(c),
                    if bound { "Bound" } else { "Conservation" }
                );
                let mut proof = dc(&name, args.iter().map(|(n, _)| var(n)).collect());
                for (n, d) in args[6..].iter().rev() {
                    proof = lam(n, d.clone(), proof);
                }
                Ok(proof)
            };
            let proof = boolean_cases(&["high", "carry"], &BTreeMap::new(), &property, &mut leaf)?;
            let ty = pi(
                "high",
                boolean(),
                pi(
                    "carry",
                    boolean(),
                    property(
                        &[("high", var("high")), ("carry", var("carry"))]
                            .into_iter()
                            .collect(),
                    ),
                ),
            );
            self.emit(
                if bound {
                    "RestoringBound"
                } else {
                    "RestoringConservation"
                },
                &arguments(var("high"), var("carry"))[..6],
                ty,
                proof,
                true,
            )?;
        }
        Ok(())
    }
    fn restoring_branch(&mut self, h: bool, c: bool) -> Result<()> {
        let args = arguments(bit(h), bit(c));
        let (r, d, m, s, t, b) = (
            var("r"),
            var("d"),
            var("modulus"),
            var("s"),
            var("t"),
            var("low"),
        );
        let k = add(mul(two(), r.clone()), bit_value(b.clone()));
        let sum = add(t.clone(), d.clone());
        let p = nat_bit_product(m.clone(), h)?.on_right(s.clone());
        let p = if h {
            p
        } else {
            p.then(Equality::natural(
                add(s.clone(), zero()),
                s.clone(),
                nc("AddZero", vec![s.clone()]),
            ))?
        };
        let push = p.reverse().then(Equality::natural(
            add(s.clone(), mul(m.clone(), bit_value(bit(h)))),
            k.clone(),
            var("_push"),
        ))?;
        let sub = Equality::natural(
            add(s.clone(), m.clone()),
            add(sum.clone(), mul(m.clone(), bit_value(bit(c)))),
            var("_sub"),
        )
        .then(nat_bit_product(m.clone(), c)?.on_right(sum.clone()))?;
        let sub = if c {
            sub
        } else {
            sub.then(Equality::natural(
                add(sum.clone(), zero()),
                sum.clone(),
                nc("AddZero", vec![sum.clone()]),
            ))?
        };
        let double = Equality::boolean(
            less(k.clone(), mul(two(), d.clone())),
            bit(true),
            dc(
                "DoubleBitBound",
                vec![r.clone(), d.clone(), b, var("_r_bound")],
            ),
        );
        let q = h || c;
        let qexpr = bc("Or", vec![bit(h), bit(c)]);
        let flag = Equality::boolean(qexpr.clone(), bit(q), refl(boolean(), bit(q)));
        let pick = flag
            .clone()
            .mapped(
                lam(
                    "_pick",
                    boolean(),
                    choose(var("_pick"), t.clone(), s.clone(), nat()),
                ),
                selected(bit(h), bit(c)),
                choose(bit(q), t.clone(), s.clone(), nat()),
                nat(),
            )
            .then(Equality::natural(
                choose(bit(q), t.clone(), s.clone(), nat()),
                if q { t.clone() } else { s.clone() },
                refl(nat(), if q { t.clone() } else { s.clone() }),
            ))?;
        let literal = map_bit(flag).then(bit_literal(q))?;
        let first = pick
            .clone()
            .on_left(mul(d.clone(), bit_value(qexpr)))
            .then(mul_right(d.clone(), literal).on_right(pick.right.clone()))?
            .then(
                Equality::natural(
                    mul(d.clone(), if q { one() } else { zero() }),
                    if q { d.clone() } else { zero() },
                    nc(if q { "MulOne" } else { "MulZero" }, vec![d.clone()]),
                )
                .on_right(pick.right.clone()),
            )?;
        let first = if q {
            first
        } else {
            first.then(Equality::natural(
                add(s.clone(), zero()),
                s.clone(),
                nc("AddZero", vec![s.clone()]),
            ))?
        };
        let (conservation, bound) = if h && c {
            let equation = sub.reverse().then(push)?;
            let contradiction = Equality::boolean(
                less(add(sum, m.clone()), mul(two(), d.clone())),
                bit(false),
                dc(
                    "LargerDivisorSum",
                    vec![d.clone(), m.clone(), t.clone(), var("_d_bound")],
                ),
            )
            .reverse()
            .then(less_left(equation, mul(two(), d.clone())))?
            .then(double)?;
            (
                Equality::natural(
                    first.right.clone(),
                    k.clone(),
                    bc(
                        "FalseElim",
                        vec![
                            eq(nat(), first.right.clone(), k),
                            contradiction.proof.clone(),
                        ],
                    ),
                ),
                Equality::boolean(
                    less(pick.right.clone(), d.clone()),
                    bit(true),
                    bc(
                        "FalseElim",
                        vec![
                            eq(boolean(), less(pick.right.clone(), d.clone()), bit(true)),
                            contradiction.proof,
                        ],
                    ),
                ),
            )
        } else if !h && !c {
            let cmp = Equality::boolean(
                bc("Not", vec![bit(false)]),
                less(s.clone(), d.clone()),
                order(
                    "SubtractionOrder",
                    vec![
                        s.clone(),
                        d.clone(),
                        t.clone(),
                        m.clone(),
                        bit(false),
                        var("_s_bound"),
                        var("_sub"),
                    ],
                ),
            )
            .reverse()
            .then(Equality::boolean(
                bc("Not", vec![bit(false)]),
                bit(true),
                refl(boolean(), bit(true)),
            ))?;
            (push, cmp)
        } else {
            let equation = if h {
                sub.reverse().then(push)?
            } else {
                let cancelled = Equality::natural(
                    s.clone(),
                    sum.clone(),
                    nc(
                        "AddCancel",
                        vec![s.clone(), sum.clone(), m.clone(), sub.proof],
                    ),
                );
                cancelled.reverse().then(push)?
            };
            let bound = Equality::boolean(
                less(t.clone(), d.clone()),
                bit(true),
                dc(
                    "RemoveDivisorBound",
                    vec![
                        d.clone(),
                        t,
                        k.clone(),
                        equation.proof.clone(),
                        double.proof,
                    ],
                ),
            );
            (equation, bound)
        };
        self.theorem(
            &format!("Restore{}{}Conservation", u8::from(h), u8::from(c)),
            &args,
            first.then(conservation)?,
        )?;
        self.theorem(
            &format!("Restore{}{}Bound", u8::from(h), u8::from(c)),
            &args,
            less_left(pick, d).then(bound)?,
        )
    }
}
