//! The actual bit-prefix expansion equals the balanced unbounded Nat value.
use super::*;

pub(super) fn digits(width: usize, i: usize, x: Expr) -> Expr {
    unary(width, &format!("Digits{i}"), x)
}

impl Numeric<'_> {
    pub(super) fn digits_combine(&mut self) -> Result<()> {
        let (initial, p, lo, w, hi) = (
            var("initial"),
            var("power"),
            var("low"),
            var("weight"),
            var("high"),
        );
        let args = [
            ("initial", nat()),
            ("power", nat()),
            ("low", nat()),
            ("weight", nat()),
            ("high", nat()),
        ];
        let e = self.polynomial(
            &args,
            add(
                add(initial.clone(), mul(p.clone(), lo.clone())),
                mul(mul(p.clone(), w.clone()), hi.clone()),
            ),
            add(initial, mul(p, add(lo, mul(w, hi)))),
        )?;
        self.theorem("DigitsCombine", &args, e)
    }
    pub(super) fn digits_family(&mut self, width: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", word_type(width)?)];
        self.emit(&format!("W{width}.Digits0"), &args, nat(), zero(), false)?;
        for i in 0..width {
            let body = add(
                digits(width, i, x.clone()),
                mul(coefficient(i), bit_value(at(x.clone(), width, i)?)),
            );
            self.emit(
                &format!("W{width}.Digits{}", i + 1),
                &args,
                nat(),
                body,
                false,
            )?;
        }
        self.digits_block(width, depth(width)?, 0)?;
        let d = depth(width)?;
        let v = value(d, x.clone());
        let e = instance(
            &format!("W{width}.DigitsBlock{d}At0"),
            vec![x.clone()],
            digits(width, width, x.clone()),
            add(digits(width, 0, x.clone()), mul(coefficient(0), v.clone())),
        )
        .then(
            Equality::natural(digits(width, 0, x.clone()), zero(), refl(nat(), zero()))
                .on_left(mul(coefficient(0), v.clone())),
        )?
        .then(
            mul_left(
                instance("PowerTwoZero", vec![], coefficient(0), one()),
                v.clone(),
            )
            .on_right(zero()),
        )?
        .then(
            Equality::natural(
                mul(one(), v.clone()),
                v.clone(),
                nc("OneMul", vec![v.clone()]),
            )
            .on_right(zero()),
        )?
        .then(Equality::natural(
            add(zero(), v.clone()),
            v.clone(),
            nc("ZeroAdd", vec![v]),
        ))?;
        self.theorem(&format!("W{width}.DigitsValue"), &args, e)
    }
    fn digits_block(&mut self, width: usize, d: usize, offset: usize) -> Result<()> {
        let size = 1 << d;
        if d > 0 {
            self.digits_block(width, d - 1, offset)?;
            self.digits_block(width, d - 1, offset + size / 2)?;
        }
        let x = var("x");
        let args = [("x", word_type(width)?)];
        let sub = app(
            x.clone(),
            address(width, offset)?[..depth(width)? - d].to_vec(),
        );
        let before = digits(width, offset, x.clone());
        let after = digits(width, offset + size, x.clone());
        let p = coefficient(offset);
        let e = if d == 0 {
            let body = add(
                before.clone(),
                mul(p.clone(), bit_value(at(x.clone(), width, offset)?)),
            );
            let e = Equality::natural(after, body.clone(), refl(nat(), body));
            let projected = Equality::natural(
                bit_value(at(x.clone(), width, offset)?),
                bit_value(sub.clone()),
                refl(nat(), bit_value(sub.clone())),
            );
            e.then(mul_right(p.clone(), projected).on_right(before.clone()))?
                .then(mul_right(p, value_definition(0, sub).reverse()).on_right(before))?
        } else {
            let half = size / 2;
            let h = d - 1;
            let (lo, hi) = (
                value(h, project(sub.clone(), false)),
                value(h, project(sub.clone(), true)),
            );
            let mid = digits(width, offset + half, x.clone());
            let hp = coefficient(offset + half);
            let low = instance(
                &format!("W{width}.DigitsBlock{h}At{offset}"),
                vec![x.clone()],
                mid.clone(),
                add(before.clone(), mul(p.clone(), lo.clone())),
            );
            let high = instance(
                &format!("W{width}.DigitsBlock{h}At{}", offset + half),
                vec![x.clone()],
                after,
                add(mid, mul(hp.clone(), hi.clone())),
            );
            let pow = instance(
                "PowerTwoAdd",
                vec![numeral(offset), numeral(half)],
                hp,
                mul(p.clone(), coefficient(half)),
            )
            .then(mul_right(
                p.clone(),
                instance(
                    &format!("PowerTwoWeight{h}"),
                    vec![],
                    coefficient(half),
                    weight(h),
                ),
            ))?;
            let e = high
                .then(low.on_left(mul(coefficient(offset + half), hi.clone())))?
                .then(
                    mul_left(pow, hi.clone())
                        .on_right(add(before.clone(), mul(p.clone(), lo.clone()))),
                )?;
            let finish = add(before.clone(), mul(p.clone(), value_body(d, sub.clone())));
            let next = instance(
                "DigitsCombine",
                vec![before.clone(), p.clone(), lo, weight(h), hi],
                e.right.clone(),
                finish,
            );
            e.then(next)?
                .then(mul_right(p, value_definition(d, sub).reverse()).on_right(before))?
        };
        self.theorem(&format!("W{width}.DigitsBlock{d}At{offset}"), &args, e)
    }
}
