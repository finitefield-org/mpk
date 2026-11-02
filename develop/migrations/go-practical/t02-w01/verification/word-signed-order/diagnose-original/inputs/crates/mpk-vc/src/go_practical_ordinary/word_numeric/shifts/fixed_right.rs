//! Bind doubled right-fill steps and actual low-block remainders to real words.
use super::pop::{lowest, pop};
use super::*;
fn low(s: usize, t: usize, x: Expr) -> Expr {
    sh(&format!("Low{s}To{t}"), vec![x])
}
fn filled(w: usize, i: usize, x: Expr, b: Expr) -> Expr {
    sh(&format!("W{w}.FilledRight{i}"), vec![x, b])
}
fn remainder(w: usize, i: usize, x: Expr) -> Expr {
    sh(&format!("W{w}.Remainder{i}"), vec![x])
}
fn r_def(w: usize, i: usize, x: Expr) -> Result<Equality> {
    let (s, j) = (depth(w)?, i.trailing_zeros() as usize);
    let rhs = value(j, low(s, j, x.clone()));
    Ok(known(
        &format!("W{w}.RemainderDefinition{i}"),
        vec![x.clone()],
        remainder(w, i, x),
        rhs,
    ))
}
fn filled_law(w: usize, i: usize, x: Expr, b: Expr) -> Result<Equality> {
    let d = depth(w)?;
    Ok(known(
        &format!("W{w}.FilledRightConservation{i}"),
        vec![x.clone(), b.clone()],
        add(
            value(d, x.clone()),
            mul(mul(weight(d), minus(numeral(i))), bit_value(b.clone())),
        ),
        add(
            mul(coefficient(i), value(d, filled(w, i, x.clone(), b))),
            remainder(w, i, x),
        ),
    ))
}
fn one_minus() -> Result<Equality> {
    let e = known(
        "PowerMinusBalance",
        vec![one()],
        add(minus(one()), one()),
        power(one()),
    )
    .then(known("SmallPower1", vec![], coefficient(1), numeral(2)))?
    .then(powers::numeral_add(1, 1)?.reverse())?;
    Ok(Equality::natural(
        minus(one()),
        one(),
        nc("AddCancel", vec![minus(one()), one(), one(), e.proof]),
    ))
}
fn split_minus(a: usize, b: usize) -> Result<Equality> {
    let e = powers::numeral_add(a, b)?;
    let (l, r) = (minus(e.left.clone()), minus(e.right.clone()));
    e.mapped(
        lam("_minus_exponent", nat(), minus(var("_minus_exponent"))),
        l,
        r,
        nat(),
    )
    .reverse()
    .then(known(
        "PowerMinusAdd",
        vec![numeral(a), numeral(b)],
        minus(add(numeral(a), numeral(b))),
        add(minus(numeral(a)), mul(coefficient(a), minus(numeral(b)))),
    ))
}
impl Shifts<'_> {
    pub(super) fn right_combine(&mut self) -> Result<()> {
        let (u, old, next, m, p, q, e, f, b, ri, rj) = (
            var("input"),
            var("old"),
            var("next"),
            var("modulus"),
            var("first_power"),
            var("second_power"),
            var("first_fill"),
            var("second_fill"),
            var("bit"),
            var("first_remainder"),
            var("second_remainder"),
        );
        let mut args = [
            "input",
            "old",
            "next",
            "modulus",
            "first_power",
            "second_power",
            "first_fill",
            "second_fill",
            "bit",
            "first_remainder",
            "second_remainder",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let ll = add(u.clone(), mul(mul(m.clone(), e.clone()), b.clone()));
        let lr = add(mul(p.clone(), old.clone()), ri.clone());
        let hl = add(old, mul(mul(m.clone(), f.clone()), b.clone()));
        let hr = add(mul(q.clone(), next.clone()), rj.clone());
        args.extend([
            ("first", eq(nat(), ll.clone(), lr.clone())),
            ("second", eq(nat(), hl.clone(), hr.clone())),
        ]);
        let initial = add(
            u,
            mul(mul(m.clone(), add(e, mul(p.clone(), f.clone()))), b.clone()),
        );
        let pad = mul(p.clone(), mul(mul(m, b.clone()), f));
        let proof = self.polynomial(&args, initial, add(ll.clone(), pad.clone()))?;
        let proof = proof.then(Equality::natural(ll, lr, var("first")).on_left(pad))?;
        let step = self.polynomial(
            &args,
            proof.right.clone(),
            add(mul(p.clone(), hl.clone()), ri.clone()),
        )?;
        let proof = proof.then(step)?.then(
            mul_right(p.clone(), Equality::natural(hl, hr, var("second"))).on_left(ri.clone()),
        )?;
        let target = add(mul(mul(p.clone(), q), next), add(ri, mul(p, rj)));
        let step = self.polynomial(&args, proof.right.clone(), target)?;
        self.theorem("RightCompose", &args, proof.then(step)?)
    }
    pub(super) fn fixed_right(&mut self, w: usize) -> Result<()> {
        let s = depth(w)?;
        let x = var("x");
        let b = var("fill");
        let args = [("x", word_type(w)?), ("fill", boolean())];
        for j in 0..s {
            self.emit(
                &format!("Low{s}To{j}"),
                &args[..1],
                cube(j),
                app(x.clone(), vec![bit(false); s - j]),
                false,
            )?;
        }
        for j in 0..s {
            let i = 1 << j;
            let body = value(j, low(s, j, x.clone()));
            self.emit(
                &format!("W{w}.Remainder{i}"),
                &args[..1],
                nat(),
                body.clone(),
                false,
            )?;
            self.theorem(
                &format!("W{w}.RemainderDefinition{i}"),
                &args[..1],
                Equality::natural(remainder(w, i, x.clone()), body.clone(), refl(nat(), body)),
            )?;
            let bits = (0..w)
                .map(|k| {
                    if k + i < w {
                        at(x.clone(), w, k + i)
                    } else {
                        Ok(b.clone())
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            self.emit(
                &format!("W{w}.FilledRight{i}"),
                &args,
                word_type(w)?,
                super::super::super::word::vector(w, bits)?,
                false,
            )?;
            self.theorem(
                &format!("W{w}.FilledRightTop{i}"),
                &args,
                Equality::boolean(
                    at(filled(w, i, x.clone(), b.clone()), w, w - 1)?,
                    b.clone(),
                    refl(boolean(), b.clone()),
                ),
            )?;
            let conservation = if i == 1 {
                self.right_one(w, x.clone(), b.clone())?
            } else {
                self.right_double(w, i, x.clone(), b.clone(), &args)?
            };
            self.theorem(
                &format!("W{w}.FilledRightConservation{i}"),
                &args,
                conservation,
            )?;
            self.bind_right(w, i, x.clone())?;
            let r = r_def(w, i, x.clone())?;
            let low_value = r.right.clone();
            let e = less_left(r, coefficient(i))
                .then(less_right(
                    low_value.clone(),
                    Equality::natural(
                        coefficient(i),
                        weight(j),
                        numeric(&format!("PowerTwoWeight{j}"), vec![]),
                    ),
                ))?
                .then(Equality::boolean(
                    less(low_value, weight(j)),
                    bit(true),
                    sh(&format!("CubeValueBound{j}"), vec![low(s, j, x.clone())]),
                ))?;
            self.theorem(&format!("W{w}.RemainderBound{i}"), &args[..1], e)?;
        }
        Ok(())
    }
    fn right_one(&self, w: usize, x: Expr, b: Expr) -> Result<Equality> {
        let d = depth(w)?;
        let u = value(d, x.clone());
        let bitb = bit_value(b.clone());
        let factor = mul_right(weight(d), one_minus()?).then(law(
            "MulOne",
            vec![weight(d)],
            mul(weight(d), one()),
            weight(d),
        ))?;
        let first = mul_left(factor, bitb.clone())
            .on_right(u.clone())
            .then(known(
                &format!("PopConservation{d}"),
                vec![x.clone(), b.clone()],
                add(u, mul(weight(d), bitb)),
                add(
                    mul(two(), value(d, pop(d, x.clone(), b.clone()))),
                    bit_value(lowest(d, x.clone())),
                ),
            ))?;
        let actual = filled(w, 1, x.clone(), b.clone());
        let popped = pop(d, x.clone(), b);
        let correspond = self.literal_values(d, actual, popped.clone());
        let power1 = known("SmallPower1", vec![], coefficient(1), numeral(2))
            .then(powers::numeral_add(1, 1)?.reverse())?;
        let loweq = map_bit(Equality::boolean(
            low(d, 0, x.clone()),
            lowest(d, x.clone()),
            refl(boolean(), lowest(d, x.clone())),
        ));
        let rd = r_def(w, 1, x.clone())?
            .then(value_definition(0, low(d, 0, x.clone())))?
            .then(loweq)?;
        let right = mul_right(coefficient(1), correspond)
            .on_left(remainder(w, 1, x.clone()))
            .then(mul_left(power1, value(d, popped.clone())).on_left(remainder(w, 1, x)))?;
        let right = right.then(rd.on_right(mul(two(), value(d, popped))))?;
        first.then(right.reverse())
    }
    fn right_double(
        &mut self,
        w: usize,
        i: usize,
        x: Expr,
        b: Expr,
        args: &[Binder],
    ) -> Result<Equality> {
        let s = depth(w)?;
        let j = i.trailing_zeros() as usize;
        let h = i / 2;
        let first = filled(w, h, x.clone(), b.clone());
        let second = filled(w, h, first.clone(), b.clone());
        let lower = low(s, j, x.clone());
        let lv = low(s, j - 1, x.clone());
        let hv = low(s, j - 1, first.clone());
        let lo = self.literal_values(j - 1, project(lower.clone(), false), lv.clone());
        let hi = self.literal_values(j - 1, project(lower.clone(), true), hv.clone());
        let split = r_def(w, i, x.clone())?
            .then(value_definition(j, lower))?
            .then(lo.clone().on_left(mul(weight(j - 1), hi.left.clone())))?
            .then(mul_right(weight(j - 1), hi).on_right(lo.right))?;
        let p = Equality::natural(
            coefficient(h),
            weight(j - 1),
            numeric(&format!("PowerTwoWeight{}", j - 1), vec![]),
        )
        .reverse();
        let split = split
            .then(mul_left(p, value(j - 1, hv.clone())).on_right(value(j - 1, lv)))?
            .then(
                r_def(w, h, x.clone())?
                    .reverse()
                    .on_left(mul(coefficient(h), value(j - 1, hv))),
            )?
            .then(
                mul_right(coefficient(h), r_def(w, h, first.clone())?.reverse())
                    .on_right(remainder(w, h, x.clone())),
            )?;
        self.theorem(&format!("W{w}.RemainderSplit{i}"), args, split.clone())?;
        let fill_split = split_minus(h, h)?;
        let left = mul_left(mul_right(weight(s), fill_split), bit_value(b.clone()))
            .on_right(value(s, x.clone()));
        let combined = known(
            "RightCompose",
            vec![
                value(s, x.clone()),
                value(s, first.clone()),
                value(s, second.clone()),
                weight(s),
                coefficient(h),
                coefficient(h),
                minus(numeral(h)),
                minus(numeral(h)),
                bit_value(b.clone()),
                remainder(w, h, x.clone()),
                remainder(w, h, first.clone()),
                filled_law(w, h, x.clone(), b.clone())?.proof,
                filled_law(w, h, first, b.clone())?.proof,
            ],
            left.right.clone(),
            add(
                mul(
                    mul(coefficient(h), coefficient(h)),
                    value(s, second.clone()),
                ),
                split.right.clone(),
            ),
        );
        let e = left.then(combined)?.then(
            mul_left(
                powers::split_power(h, h)?.reverse(),
                value(s, second.clone()),
            )
            .on_left(split.right.clone()),
        )?;
        let actual = filled(w, i, x.clone(), b);
        let actual_value = value(s, actual.clone());
        let v = self.literal_values(s, actual, second).reverse();
        e.then(mul_right(coefficient(i), v).on_left(split.right.clone()))?
            .then(split.reverse().on_right(mul(coefficient(i), actual_value)))
    }
    fn bind_right(&mut self, w: usize, i: usize, x: Expr) -> Result<()> {
        let d = depth(w)?;
        let args = [("x", word_type(w)?)];
        let bit_zero = bit_literal(false);
        let left = mul_right(mul(weight(d), minus(numeral(i))), bit_zero)
            .on_right(value(d, x.clone()))
            .then(
                law(
                    "MulZero",
                    vec![mul(weight(d), minus(numeral(i)))],
                    mul(mul(weight(d), minus(numeral(i))), zero()),
                    zero(),
                )
                .on_right(value(d, x.clone())),
            )?
            .then(law(
                "AddZero",
                vec![value(d, x.clone())],
                add(value(d, x.clone()), zero()),
                value(d, x.clone()),
            ))?;
        let e = left
            .reverse()
            .then(filled_law(w, i, x.clone(), bit(false))?)?;
        let actual = wc(w, &format!("Right{i}"), vec![x.clone()]);
        let bind = self
            .literal_values(d, actual.clone(), filled(w, i, x.clone(), bit(false)))
            .reverse();
        self.theorem(
            &format!("W{w}.UnsignedRightConservation{i}"),
            &args,
            e.then(mul_right(coefficient(i), bind).on_left(remainder(w, i, x.clone())))?,
        )?;
        let top = at(x.clone(), w, w - 1)?;
        let actual = wc(w, &format!("SignedRight{i}"), vec![x.clone()]);
        let bind = self
            .literal_values(d, actual.clone(), filled(w, i, x.clone(), top.clone()))
            .reverse();
        let e = filled_law(w, i, x.clone(), top.clone())?
            .then(mul_right(coefficient(i), bind).on_left(remainder(w, i, x.clone())))?;
        self.theorem(&format!("W{w}.SignedRightConservation{i}"), &args, e)?;
        self.theorem(
            &format!("W{w}.SignedRightTop{i}"),
            &args,
            Equality::boolean(at(actual, w, w - 1)?, top.clone(), refl(boolean(), top)),
        )
    }
}
