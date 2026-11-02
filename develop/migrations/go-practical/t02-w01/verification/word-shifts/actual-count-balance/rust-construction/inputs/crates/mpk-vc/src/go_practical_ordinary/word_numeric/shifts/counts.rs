//! Full64 count bits, with the exact balanced high-bit tree of the source.
use super::*;
mod reflection;
fn digits(i: usize, x: Expr) -> Expr {
    numeric(&format!("W64.Digits{i}"), vec![x])
}
fn high(d: usize, x: Expr) -> Expr {
    sh(&format!("CountHigh{d}"), vec![x])
}
fn high_value(d: usize, x: Expr) -> Expr {
    sh(&format!("CountHighValue{d}"), vec![x])
}
fn tree(d: usize, a: usize, b: usize, x: Expr, numeric: bool) -> Result<Expr> {
    if b == a + 1 {
        let bit = at(x, 64, a)?;
        return Ok(if numeric {
            mul(coefficient(a - d), bit_value(bit))
        } else {
            bit
        });
    }
    let m = a + (b - a) / 2;
    let (l, r) = (
        tree(d, a, m, x.clone(), numeric)?,
        tree(d, m, b, x, numeric)?,
    );
    Ok(if numeric {
        add(l, r)
    } else {
        bc("Or", vec![l, r])
    })
}
impl Shifts<'_> {
    pub(super) fn counts(&mut self) -> Result<()> {
        self.count_combine()?;
        self.count_reflection_laws()?;
        let count = var("count");
        let args = [("count", word_type(64)?)];
        for d in 3..=6 {
            let bits = tree(d, d, 64, count.clone(), false)?;
            let sum = tree(d, d, 64, count.clone(), true)?;
            self.emit(
                &format!("CountHigh{d}"),
                &args,
                boolean(),
                bits.clone(),
                false,
            )?;
            self.emit(
                &format!("CountHighValue{d}"),
                &args,
                nat(),
                sum.clone(),
                false,
            )?;
            let balance = self.count_range(d, d, 64, count.clone())?;
            let sum_def = Equality::natural(
                high_value(d, count.clone()),
                sum.clone(),
                refl(nat(), sum.clone()),
            );
            let balance = balance.then(
                mul_right(coefficient(d), sum_def.reverse()).on_right(digits(d, count.clone())),
            )?;
            let full = Equality::natural(
                digits(64, count.clone()),
                value(6, count.clone()),
                numeric("W64.DigitsValue", vec![count.clone()]),
            );
            self.theorem(
                &format!("CountBalance{d}"),
                &args,
                full.reverse().then(balance)?,
            )?;
            let reflection = self.count_zero(d, d, 64, count.clone())?;
            let sum_def = Equality::natural(
                high_value(d, count.clone()),
                sum,
                refl(nat(), reflection.left.clone()), /* replaced below */
            );
            // Transport a small definition equality through Nat.Equal before
            // reducing its major; no computed-major conversion is assumed.
            let sum_def = Equality::natural(
                sum_def.left,
                sum_def.right.clone(),
                refl(nat(), sum_def.right),
            );
            let bits_def =
                Equality::boolean(high(d, count.clone()), bits.clone(), refl(boolean(), bits));
            let proof = equal_left(sum_def, zero())
                .then(reflection)?
                .then(bool_map("Not", bits_def.reverse()))?;
            self.theorem(&format!("CountHighZero{d}"), &args, proof)?;
        }
        Ok(())
    }
    fn count_combine(&mut self) -> Result<()> {
        let (a, m, z, p, l, r) = (
            var("before"),
            var("middle"),
            var("after"),
            var("power"),
            var("low"),
            var("high"),
        );
        let mut args = ["before", "middle", "after", "power", "low", "high"]
            .map(|n| (n, nat()))
            .to_vec();
        let le = add(a.clone(), mul(p.clone(), l.clone()));
        let re = add(m.clone(), mul(p.clone(), r.clone()));
        args.extend([
            ("first", eq(nat(), m.clone(), le.clone())),
            ("second", eq(nat(), z.clone(), re.clone())),
        ]);
        let e = Equality::natural(z, re, var("second"))
            .then(Equality::natural(m, le, var("first")).on_left(mul(p.clone(), r.clone())))?;
        let next = self.polynomial(&args, e.right.clone(), add(a, mul(p, add(l, r))))?;
        self.theorem("CountRangeCompose", &args, e.then(next)?)
    }
    fn count_range(&self, d: usize, a: usize, b: usize, x: Expr) -> Result<Equality> {
        let before = digits(a, x.clone());
        let after = digits(b, x.clone());
        let sum = tree(d, a, b, x.clone(), true)?;
        let target = add(before.clone(), mul(coefficient(d), sum));
        if b == a + 1 {
            let v = bit_value(at(x, 64, a)?);
            let body = add(before.clone(), mul(coefficient(a), v.clone()));
            return Equality::natural(after, body.clone(), refl(nat(), body))
                .then(mul_left(powers::split_power(d, a - d)?, v.clone()).on_right(before.clone()))?
                .then(
                    law(
                        "MulAssoc",
                        vec![coefficient(d), coefficient(a - d), v.clone()],
                        mul(mul(coefficient(d), coefficient(a - d)), v.clone()),
                        mul(coefficient(d), mul(coefficient(a - d), v)),
                    )
                    .on_right(before),
                );
        }
        let m = a + (b - a) / 2;
        let l = self.count_range(d, a, m, x.clone())?;
        let r = self.count_range(d, m, b, x.clone())?;
        Ok(known(
            "CountRangeCompose",
            vec![
                before,
                digits(m, x.clone()),
                after.clone(),
                coefficient(d),
                tree(d, a, m, x.clone(), true)?,
                tree(d, m, b, x, true)?,
                l.proof,
                r.proof,
            ],
            after,
            target,
        ))
    }
    fn count_zero(&self, d: usize, a: usize, b: usize, x: Expr) -> Result<Equality> {
        let sum = tree(d, a, b, x.clone(), true)?;
        let bits = tree(d, a, b, x.clone(), false)?;
        let target = bc("Not", vec![bits]);
        if b == a + 1 {
            return Ok(Equality::boolean(
                equal(sum, zero()),
                target,
                sh(
                    "WeightedBitZero",
                    vec![
                        coefficient(a - d),
                        at(x, 64, a)?,
                        sh("PowerPositive", vec![numeral(a - d)]),
                    ],
                ),
            ));
        }
        let m = a + (b - a) / 2;
        let l = self.count_zero(d, a, m, x.clone())?;
        let r = self.count_zero(d, m, b, x.clone())?;
        let (lv, rv) = (
            tree(d, a, m, x.clone(), true)?,
            tree(d, m, b, x.clone(), true)?,
        );
        let (lb, rb) = (tree(d, a, m, x.clone(), false)?, tree(d, m, b, x, false)?);
        Ok(Equality::boolean(
            equal(sum, zero()),
            target,
            sh("CountPairZero", vec![lv, rv, lb, rb, l.proof, r.proof]),
        ))
    }
}
