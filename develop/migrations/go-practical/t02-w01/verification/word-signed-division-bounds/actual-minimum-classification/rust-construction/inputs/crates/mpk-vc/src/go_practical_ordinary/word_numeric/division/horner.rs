//! A shared MSB-first Nat prefix evaluates to the same balanced word Value.
use super::*;

pub(super) fn prefix(width: usize, i: usize, z: Expr) -> Expr {
    dc(&format!("W{width}.Prefix{i}"), vec![z])
}
fn chunk(width: usize, d: usize, offset: usize, z: Expr) -> Result<Expr> {
    let physical = width - offset - (1 << d);
    Ok(app(
        z,
        address(width, physical)?[..depth(width)? - d].to_vec(),
    ))
}

impl Division<'_> {
    pub(super) fn horner_laws(&mut self) -> Result<()> {
        let (w, a, hi, lo) = (var("weight"), var("initial"), var("high"), var("low"));
        let args = [
            ("weight", nat()),
            ("initial", nat()),
            ("high", nat()),
            ("low", nat()),
        ];
        let e = self.polynomial(
            &args,
            add(
                mul(w.clone(), add(mul(w.clone(), a.clone()), hi.clone())),
                lo.clone(),
            ),
            add(mul(mul(w.clone(), w.clone()), a), add(lo, mul(w, hi))),
        )?;
        self.theorem("HornerCombine", &args, e)
    }
    pub(super) fn horner_family(&mut self, width: usize) -> Result<()> {
        let z = var("z");
        let args = [("z", word_type(width)?)];
        self.emit(&format!("W{width}.Prefix0"), &args, nat(), zero(), false)?;
        for i in 0..width {
            let body = add(
                mul(two(), prefix(width, i, z.clone())),
                bit_value(at(z.clone(), width, width - 1 - i)?),
            );
            self.emit(
                &format!("W{width}.Prefix{}", i + 1),
                &args,
                nat(),
                body.clone(),
                false,
            )?;
            self.theorem(
                &format!("W{width}.PrefixDefinition{}", i + 1),
                &args,
                Equality::natural(
                    prefix(width, i + 1, z.clone()),
                    body.clone(),
                    refl(nat(), body),
                ),
            )?;
        }
        let d = depth(width)?;
        self.horner_block(width, d, 0)?;
        let v = value(d, z.clone());
        let e = instance(
            &format!("W{width}.HornerBlock{d}At0"),
            vec![z.clone()],
            prefix(width, width, z.clone()),
            add(mul(weight(d), prefix(width, 0, z.clone())), v.clone()),
        )
        .then(
            mul_right(
                weight(d),
                Equality::natural(prefix(width, 0, z.clone()), zero(), refl(nat(), zero())),
            )
            .on_left(v.clone()),
        )?
        .then(
            Equality::natural(
                mul(weight(d), zero()),
                zero(),
                nc("MulZero", vec![weight(d)]),
            )
            .on_left(v.clone()),
        )?
        .then(Equality::natural(
            add(zero(), v.clone()),
            v.clone(),
            nc("ZeroAdd", vec![v]),
        ))?;
        self.theorem(&format!("W{width}.PrefixValue"), &args, e)
    }
    fn horner_block(&mut self, width: usize, d: usize, offset: usize) -> Result<()> {
        let size = 1 << d;
        if d > 0 {
            self.horner_block(width, d - 1, offset)?;
            self.horner_block(width, d - 1, offset + size / 2)?;
        }
        let z = var("z");
        let args = [("z", word_type(width)?)];
        let sub = chunk(width, d, offset, z.clone())?;
        let before = prefix(width, offset, z.clone());
        let after = prefix(width, offset + size, z.clone());
        let e = if d == 0 {
            let at = at(z.clone(), width, width - 1 - offset)?;
            let body = add(mul(two(), before.clone()), bit_value(at.clone()));
            instance(
                &format!("W{width}.PrefixDefinition{}", offset + 1),
                vec![z.clone()],
                after,
                body,
            )
            .then(
                map_bit(Equality::boolean(
                    at,
                    sub.clone(),
                    refl(boolean(), sub.clone()),
                ))
                .on_right(mul(two(), before.clone())),
            )?
            .then(
                value_definition(0, sub)
                    .reverse()
                    .on_right(mul(two(), before.clone())),
            )?
            .then(
                mul_left(weight_definition(0).reverse(), before)
                    .on_left(value(0, chunk(width, 0, offset, z)?)),
            )?
        } else {
            let h = d - 1;
            let half = size / 2;
            let mid = prefix(width, offset + half, z.clone());
            let (high_word, low_word) = (
                chunk(width, h, offset, z.clone())?,
                chunk(width, h, offset + half, z.clone())?,
            );
            let hi = value(h, project(sub.clone(), true));
            let lo = value(h, project(sub.clone(), false));
            let high = instance(
                &format!("W{width}.HornerBlock{h}At{offset}"),
                vec![z.clone()],
                mid.clone(),
                add(mul(weight(h), before.clone()), value(h, high_word.clone())),
            )
            .then(
                map_value(
                    h,
                    Equality {
                        ty: cube(h),
                        left: high_word,
                        right: project(sub.clone(), true),
                        proof: refl(cube(h), project(sub.clone(), true)),
                    },
                )
                .on_right(mul(weight(h), before.clone())),
            )?;
            let low = instance(
                &format!("W{width}.HornerBlock{h}At{}", offset + half),
                vec![z.clone()],
                after,
                add(mul(weight(h), mid), value(h, low_word.clone())),
            )
            .then(
                map_value(
                    h,
                    Equality {
                        ty: cube(h),
                        left: low_word,
                        right: project(sub.clone(), false),
                        proof: refl(cube(h), project(sub.clone(), false)),
                    },
                )
                .on_right(mul(weight(h), prefix(width, offset + half, z.clone()))),
            )?;
            let first = low.then(mul_right(weight(h), high).on_left(lo.clone()))?;
            let next = instance(
                "HornerCombine",
                vec![weight(h), before.clone(), hi, lo],
                first.right.clone(),
                add(
                    mul(weight_body(d), before.clone()),
                    value_body(d, sub.clone()),
                ),
            );
            first
                .then(next)?
                .then(
                    mul_left(weight_definition(d).reverse(), before)
                        .on_left(value_body(d, sub.clone())),
                )?
                .then(
                    value_definition(d, sub)
                        .reverse()
                        .on_right(mul(weight(d), prefix(width, offset, z))),
                )?
        };
        self.theorem(&format!("W{width}.HornerBlock{d}At{offset}"), &args, e)
    }
}
