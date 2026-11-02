//! Shared balanced half-weights and lower-bit definitions, never unary moduli.
use super::*;
impl Bounds<'_> {
    pub(super) fn definitions(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let h = if d == 0 {
            one()
        } else {
            mul(weight(d - 1), half(d - 1))
        };
        self.emit(&format!("Half{d}"), &[], nat(), h.clone(), false)?;
        self.theorem(
            &format!("HalfDefinition{d}"),
            &[],
            Equality::natural(half(d), h.clone(), refl(nat(), h)),
        )?;
        for c in [false, true] {
            let label = if c { "LowComplement" } else { "Low" };
            let body = low_body(d, x.clone(), c);
            self.emit(
                &format!("{label}{d}"),
                &[("x", cube(d))],
                nat(),
                body.clone(),
                false,
            )?;
            let lhs = if c {
                low_complement(d, x.clone())
            } else {
                low(d, x.clone())
            };
            self.theorem(
                &format!("{label}Definition{d}"),
                &[("x", cube(d))],
                Equality::natural(lhs, body.clone(), refl(nat(), body)),
            )?;
        }
        let bit = app(x.clone(), vec![bit(true); d]);
        self.emit(
            &format!("Top{d}"),
            &[("x", cube(d))],
            boolean(),
            bit.clone(),
            false,
        )?;
        self.theorem(
            &format!("TopDefinition{d}"),
            &[("x", cube(d))],
            Equality::boolean(top(d, x), bit.clone(), refl(boolean(), bit)),
        )
    }
    pub(super) fn half_double(&mut self, d: usize) -> Result<()> {
        let a = half_definition(d);
        let e = a
            .clone()
            .on_left(half(d))
            .then(a.clone().on_right(a.right.clone()))?;
        let e = if d == 0 {
            self.polynomial(&[], e.right.clone(), two())
                .and_then(|p| e.then(p))?
        } else {
            let h = d - 1;
            let p = self.polynomial(&[], e.right.clone(), mul(weight(h), add(half(h), half(h))))?;
            e.then(p)?.then(mul_right(
                weight(h),
                instance(
                    &format!("HalfDouble{h}"),
                    vec![],
                    add(half(h), half(h)),
                    weight(h),
                ),
            ))?
        };
        self.theorem(
            &format!("HalfDouble{d}"),
            &[],
            e.then(weight_definition(d).reverse())?,
        )
    }
    pub(super) fn half_positive(&mut self, d: usize) -> Result<()> {
        let mut z = bit(false);
        for name in ADDRESSES[..d].iter().rev() {
            z = lam(name, boolean(), z);
        }
        let sum = add(low(d, z.clone()), low_complement(d, z.clone()));
        let h = instance(
            &format!("LowComplementBalance{d}"),
            vec![z],
            add(sum.clone(), one()),
            half(d),
        )
        .reverse()
        .then(nlaw(
            "AddOne",
            vec![sum.clone()],
            add(sum.clone(), one()),
            succ(sum.clone()),
        ))?;
        self.theorem(
            &format!("HalfPositive{d}"),
            &[],
            less_right(zero(), h).then(Equality::boolean(
                less(zero(), succ(sum.clone())),
                bit(true),
                nc("LessZeroSucc", vec![sum]),
            ))?,
        )
    }
}
