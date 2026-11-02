//! Exact fixed-left-shift quotients, selected addends and actual addition.
use super::*;

pub(super) fn left(width: usize, i: usize, x: Expr) -> Expr {
    wc(width, &format!("Left{i}"), vec![x])
}
pub(super) fn discarded(width: usize, i: usize, x: Expr) -> Expr {
    unary(width, &format!("ShiftDiscard{i}"), x)
}
fn zero_cube(d: usize) -> Expr {
    let mut body = bit(false);
    for name in ADDRESSES[..d].iter().rev() {
        body = lam(name, boolean(), body);
    }
    body
}

impl Numeric<'_> {
    pub(super) fn literal_correspondence(
        &mut self,
        width: usize,
        name: &str,
        args: &[Binder],
        from: Expr,
        to: Expr,
    ) -> Result<Equality> {
        let d = depth(width)?;
        let names = &ADDRESSES[..d];
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            let a = names.iter().map(|n| e[n].clone()).collect::<Vec<_>>();
            eq(boolean(), app(from.clone(), a.clone()), app(to.clone(), a))
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            Ok(refl(
                boolean(),
                app(to.clone(), names.iter().map(|n| e[n].clone()).collect()),
            ))
        };
        let proof = boolean_cases(names, &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            name,
            args,
            pointwise(width, from.clone(), to.clone())?,
            proof,
            true,
        )?;
        Ok(instance(
            &format!("ValueCongruence{d}"),
            vec![
                from.clone(),
                to.clone(),
                numeric(name, args.iter().map(|(n, _)| var(n)).collect()),
            ],
            value(d, from),
            value(d, to),
        ))
    }

    pub(super) fn shift_combine(&mut self) -> Result<()> {
        let (m, old, next, q, top, p, x) = (
            var("modulus"),
            var("old"),
            var("next"),
            var("quotient"),
            var("top"),
            var("power"),
            var("x"),
        );
        let sl = add(old.clone(), mul(m.clone(), q.clone()));
        let sr = mul(p.clone(), x.clone());
        let tl = add(next.clone(), mul(m.clone(), top.clone()));
        let tr = mul(two(), old);
        let mut args: Vec<Binder> = ["modulus", "old", "next", "quotient", "top", "power", "x"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect();
        args.extend([
            ("prior", eq(nat(), sl.clone(), sr.clone())),
            ("step", eq(nat(), tl.clone(), tr.clone())),
        ]);
        let lhs = add(next, mul(m.clone(), add(mul(two(), q.clone()), top)));
        let stage = add(tl.clone(), mul(two(), mul(m.clone(), q.clone())));
        let e = self
            .polynomial(&args, lhs, stage)?
            .then(Equality::natural(tl, tr, var("step")).on_left(mul(two(), mul(m, q))))?;
        let next = self.polynomial(&args, e.right.clone(), mul(two(), sl.clone()))?;
        let e = e
            .then(next)?
            .then(mul_right(two(), Equality::natural(sl, sr, var("prior"))))?;
        let next = self.polynomial(&args, e.right.clone(), mul(mul(two(), p), x))?;
        self.theorem("ShiftCombine", &args, e.then(next)?)
    }

    pub(super) fn zero_values(&mut self) -> Result<()> {
        for d in 0..=6 {
            let x = zero_cube(d);
            let e = value_definition(d, x.clone());
            let e = if d == 0 {
                e.then(bit_literal(false))?
            } else {
                let h = d - 1;
                let low = instance(
                    &format!("ZeroValue{h}"),
                    vec![],
                    value(h, project(x.clone(), false)),
                    zero(),
                );
                let high = instance(
                    &format!("ZeroValue{h}"),
                    vec![],
                    value(h, project(x, true)),
                    zero(),
                );
                e.then(low.on_left(mul(weight(h), high.left.clone())))?
                    .then(mul_right(weight(h), high).on_right(zero()))?
                    .then(
                        Equality::natural(
                            mul(weight(h), zero()),
                            zero(),
                            nc("MulZero", vec![weight(h)]),
                        )
                        .on_right(zero()),
                    )?
                    .then(Equality::natural(
                        add(zero(), zero()),
                        zero(),
                        nc("ZeroAdd", vec![zero()]),
                    ))?
            };
            self.theorem(&format!("ZeroValue{d}"), &[], e)?;
        }
        Ok(())
    }

    pub(super) fn shift_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let x = var("x");
        let args = [("x", word_type(width)?)];
        let z = wc(width, "Zero", vec![]);
        let e = self.literal_correspondence(
            width,
            &format!("W{width}.ZeroPointwise"),
            &[],
            z.clone(),
            zero_cube(d),
        )?;
        self.theorem(
            &format!("W{width}.ZeroValue"),
            &[],
            e.then(instance(
                &format!("ZeroValue{d}"),
                vec![],
                value(d, zero_cube(d)),
                zero(),
            ))?,
        )?;
        let flag = var("flag");
        let select = wc(width, "Select", vec![flag.clone(), x.clone(), z.clone()]);
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                value(
                    d,
                    wc(
                        width,
                        "Select",
                        vec![env["flag"].clone(), x.clone(), z.clone()],
                    ),
                ),
                mul(bit_value(env["flag"].clone()), value(d, x.clone())),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let b = env["flag"] == bit(true);
            let target = if b {
                value(d, x.clone())
            } else {
                value(d, z.clone())
            };
            let lhs = Equality::natural(
                value(d, wc(width, "Select", vec![bit(b), x.clone(), z.clone()])),
                target.clone(),
                refl(nat(), target),
            );
            let lhs = if b {
                lhs
            } else {
                lhs.then(instance(
                    &format!("W{width}.ZeroValue"),
                    vec![],
                    value(d, z.clone()),
                    zero(),
                ))?
            };
            let v = value(d, x.clone());
            let n = if b { one() } else { zero() };
            let rhs = mul_left(bit_literal(b), v.clone()).then(Equality::natural(
                mul(n, v.clone()),
                if b { v.clone() } else { zero() },
                nc(if b { "OneMul" } else { "ZeroMul" }, vec![v]),
            ))?;
            Ok(lhs.then(rhs.reverse())?.proof)
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            &format!("W{width}.SelectZeroValue"),
            &args,
            pi(
                "flag",
                boolean(),
                eq(
                    nat(),
                    value(d, select),
                    mul(bit_value(flag), value(d, x.clone())),
                ),
            ),
            proof,
            true,
        )?;
        self.addition_value(width)?;
        self.emit(
            &format!("W{width}.ShiftDiscard0"),
            &args,
            nat(),
            zero(),
            false,
        )?;
        let first = left(width, 0, x.clone());
        let correspondence = self.literal_correspondence(
            width,
            &format!("W{width}.LeftPointwise0"),
            &args,
            first.clone(),
            x.clone(),
        )?;
        let q = discarded(width, 0, x.clone());
        let e = mul_right(
            weight(d),
            Equality::natural(q.clone(), zero(), refl(nat(), zero())),
        )
        .on_right(value(d, first.clone()))
        .then(
            Equality::natural(
                mul(weight(d), zero()),
                zero(),
                nc("MulZero", vec![weight(d)]),
            )
            .on_right(value(d, first.clone())),
        )?
        .then(Equality::natural(
            add(value(d, first.clone()), zero()),
            value(d, first),
            nc("AddZero", vec![value(d, left(width, 0, x.clone()))]),
        ))?
        .then(correspondence)?;
        let rhs = mul_left(
            instance("PowerTwoZero", vec![], coefficient(0), one()),
            value(d, x.clone()),
        )
        .then(Equality::natural(
            mul(one(), value(d, x.clone())),
            value(d, x.clone()),
            nc("OneMul", vec![value(d, x.clone())]),
        ))?;
        self.theorem(
            &format!("W{width}.ShiftConservation0"),
            &args,
            e.then(rhs.reverse())?,
        )?;
        for i in 1..width {
            let prior = left(width, i - 1, x.clone());
            let next = left(width, i, x.clone());
            let top = bit_value(at(prior.clone(), width, width - 1)?);
            let pushed = wc(width, "PushLow", vec![prior.clone(), bit(false)]);
            let v = self.literal_correspondence(
                width,
                &format!("W{width}.LeftPointwise{i}"),
                &args,
                next.clone(),
                pushed.clone(),
            )?;
            let rhs = add(mul(two(), value(d, prior.clone())), bit_value(bit(false)));
            let e = v
                .on_left(mul(weight(d), top.clone()))
                .then(instance(
                    &format!("W{width}.PushLowConservation"),
                    vec![prior.clone(), bit(false)],
                    add(value(d, pushed), mul(weight(d), top.clone())),
                    rhs,
                ))?
                .then(bit_literal(false).on_right(mul(two(), value(d, prior.clone()))))?
                .then(Equality::natural(
                    add(mul(two(), value(d, prior.clone())), zero()),
                    mul(two(), value(d, prior.clone())),
                    nc("AddZero", vec![mul(two(), value(d, prior.clone()))]),
                ))?;
            self.theorem(&format!("W{width}.LeftStep{i}"), &args, e.clone())?;
            let oldq = discarded(width, i - 1, x.clone());
            let q = discarded(width, i, x.clone());
            let qbody = add(mul(two(), oldq.clone()), top.clone());
            self.emit(
                &format!("W{width}.ShiftDiscard{i}"),
                &args,
                nat(),
                qbody.clone(),
                false,
            )?;
            let e = Equality::natural(q, qbody.clone(), refl(nat(), qbody)).mapped(
                lam(
                    "_quotient",
                    nat(),
                    add(value(d, next.clone()), mul(weight(d), var("_quotient"))),
                ),
                add(
                    value(d, next.clone()),
                    mul(weight(d), discarded(width, i, x.clone())),
                ),
                add(
                    value(d, next.clone()),
                    mul(weight(d), add(mul(two(), oldq.clone()), top.clone())),
                ),
                nat(),
            );
            let rhs = mul(mul(two(), coefficient(i - 1)), value(d, x.clone()));
            let combine = instance(
                "ShiftCombine",
                vec![
                    weight(d),
                    value(d, prior),
                    value(d, next),
                    oldq,
                    top,
                    coefficient(i - 1),
                    value(d, x.clone()),
                    unary(width, &format!("ShiftConservation{}", i - 1), x.clone()),
                    unary(width, &format!("LeftStep{i}"), x.clone()),
                ],
                e.right.clone(),
                rhs,
            );
            let e = e.then(combine)?.then(mul_left(
                instance(
                    "PowerTwoSucc",
                    vec![numeral(i - 1)],
                    coefficient(i),
                    mul(two(), coefficient(i - 1)),
                )
                .reverse(),
                value(d, x.clone()),
            ))?;
            self.theorem(&format!("W{width}.ShiftConservation{i}"), &args, e)?;
        }
        Ok(())
    }

    fn addition_value(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, y) = (var("x"), var("y"));
        let sum = add(value(d, x.clone()), value(d, y.clone()));
        let old = wc(width, "AddCarry", vec![x.clone(), y.clone(), bit(false)]);
        let result = wc(width, "Add", vec![x.clone(), y.clone()]);
        let carry = bit_value(wc(
            width,
            &format!("Carry{width}"),
            vec![x.clone(), y.clone(), bit(false)],
        ));
        let padded = bit_literal(false)
            .on_right(sum.clone())
            .then(Equality::natural(
                add(sum.clone(), zero()),
                sum.clone(),
                nc("AddZero", vec![sum.clone()]),
            ))?;
        let root = instance(
            &format!("W{width}.Block{d}At0"),
            vec![x.clone(), y.clone(), bit(false)],
            add(sum, bit_value(bit(false))),
            add(value(d, old.clone()), mul(weight(d), carry.clone())),
        );
        let word = Equality {
            ty: word_type(width)?,
            left: result,
            right: old.clone(),
            proof: refl(word_type(width)?, old),
        };
        let e = padded
            .reverse()
            .then(root)?
            .then(map_value(d, word).reverse().on_left(mul(weight(d), carry)))?;
        self.theorem(
            &format!("W{width}.AddValueConservation"),
            &[("x", word_type(width)?), ("y", word_type(width)?)],
            e,
        )
    }
}
