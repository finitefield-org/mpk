//! Actual highest bit and every lower bit account for the unsigned value.
use super::*;
impl Bounds<'_> {
    pub(super) fn value_split(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", cube(d))];
        let rhs = add(
            low(d, x.clone()),
            mul(half(d), bit_value(top(d, x.clone()))),
        );
        let lo = low_definition(d, x.clone(), false);
        let hd = half_definition(d);
        let expanded = lo
            .clone()
            .on_left(mul(half(d), bit_value(top(d, x.clone()))))
            .then(mul_left(hd.clone(), bit_value(top(d, x.clone()))).on_right(lo.right))?;
        let left = if d == 0 {
            value_definition(d, x.clone())
        } else {
            let h = d - 1;
            let hi = project(x.clone(), true);
            let projection = Equality::boolean(
                top(d, x.clone()),
                top(h, hi.clone()),
                refl(boolean(), top(h, hi.clone())),
            );
            self.theorem(&format!("TopProjection{d}"), &args, projection.clone())?;
            let child = instance(
                &format!("ValueSplit{h}"),
                vec![hi.clone()],
                value(h, hi.clone()),
                add(
                    low(h, hi.clone()),
                    mul(half(h), bit_value(top(h, hi.clone()))),
                ),
            )
            .then(mul_right(half(h), map_bit(projection.reverse())).on_right(low(h, hi)))?;
            value_definition(d, x.clone())
                .then(mul_right(weight(h), child).on_right(value(h, project(x.clone(), false))))?
        };
        let expanded = if d == 0 {
            expanded.then(
                mul_right(
                    hd.right,
                    map_bit(Equality::boolean(
                        top(0, x.clone()),
                        x.clone(),
                        sb("TopDefinition0", vec![x.clone()]),
                    )),
                )
                .on_right(zero()),
            )?
        } else {
            expanded
        };
        let p = self.polynomial(&args, left.right.clone(), expanded.right.clone())?;
        let proof = left.then(p)?.then(expanded.reverse())?;
        if proof.right != rhs {
            return Err(GoOrdinaryError::Linkage);
        }
        self.theorem(&format!("ValueSplit{d}"), &args, proof)
    }
    pub(super) fn low_balance(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", cube(d))];
        let lo = low_definition(d, x.clone(), false);
        let co = low_definition(d, x.clone(), true);
        let e = lo
            .clone()
            .on_left(co.left.clone())
            .then(co.on_right(lo.right))?
            .on_left(one());
        let e = if d == 0 {
            let p = self.polynomial(&args, e.right.clone(), one())?;
            e.then(p)?
        } else {
            let h = d - 1;
            let (xl, xh) = (project(x.clone(), false), project(x.clone(), true));
            let low_sum = add(
                add(value(h, xl.clone()), value(h, not(h, xl.clone()))),
                one(),
            );
            let high_sum = add(low(h, xh.clone()), low_complement(h, xh.clone()));
            let p = self.polynomial(
                &args,
                e.right.clone(),
                add(low_sum.clone(), mul(weight(h), high_sum.clone())),
            )?;
            let e = e.then(p)?.then(
                Equality::natural(
                    low_sum,
                    weight(h),
                    order(&format!("Complement{h}"), vec![xl]),
                )
                .on_left(mul(weight(h), high_sum.clone())),
            )?;
            let p = self.polynomial(
                &args,
                e.right.clone(),
                mul(weight(h), add(high_sum.clone(), one())),
            )?;
            e.then(p)?.then(mul_right(
                weight(h),
                instance(
                    &format!("LowComplementBalance{h}"),
                    vec![xh],
                    add(high_sum, one()),
                    half(h),
                ),
            ))?
        };
        self.theorem(
            &format!("LowComplementBalance{d}"),
            &args,
            e.then(half_definition(d).reverse())?,
        )
    }
    pub(super) fn low_bound(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let l = low(d, x.clone());
        let c = low_complement(d, x.clone());
        let e = instance(
            &format!("LowComplementBalance{d}"),
            vec![x.clone()],
            add(add(l.clone(), c.clone()), one()),
            half(d),
        )
        .reverse()
        .then(nlaw(
            "AddOne",
            vec![add(l.clone(), c.clone())],
            add(add(l.clone(), c.clone()), one()),
            succ(add(l.clone(), c.clone())),
        ))?
        .then(
            nlaw(
                "AddSucc",
                vec![l.clone(), c.clone()],
                add(l.clone(), succ(c.clone())),
                succ(add(l.clone(), c.clone())),
            )
            .reverse(),
        )?;
        self.theorem(
            &format!("LowStrictHalf{d}"),
            &[("x", cube(d))],
            less_right(l.clone(), e).then(Equality::boolean(
                less(l.clone(), add(l.clone(), succ(c.clone()))),
                bit(true),
                order("LessPositiveExtension", vec![l, c]),
            ))?,
        )
    }
}
