//! Small structural cube laws over arbitrary input bits, not operand cases.
use super::*;

impl Conversions<'_> {
    pub(super) fn fill(&mut self, d: usize) -> Result<()> {
        let b = var("bit");
        let args = [("bit", boolean())];
        let mut body = b.clone();
        for n in ADDRESSES[..d].iter().rev() {
            body = lam(n, boolean(), body);
        }
        self.emit(&format!("Fill{d}"), &args, cube(d), body, false)?;
        let o = if d == 0 {
            one()
        } else {
            add(ones(d - 1), mul(weight(d - 1), ones(d - 1)))
        };
        self.emit(&format!("Ones{d}"), &[], nat(), o.clone(), false)?;
        let balance = if d == 0 {
            self.polynomial(&[], add(o.clone(), one()), weight_body(d))?
        } else {
            let h = d - 1;
            let prior = known(
                &format!("OnesBalance{h}"),
                vec![],
                add(ones(h), one()),
                weight(h),
            );
            let e = self.polynomial(
                &[],
                add(o.clone(), one()),
                add(mul(weight(h), ones(h)), add(ones(h), one())),
            )?;
            let e = e.then(prior.clone().on_right(mul(weight(h), ones(h))))?;
            let p = self.polynomial(&[], e.right.clone(), mul(weight(h), add(ones(h), one())))?;
            e.then(p)?.then(mul_right(weight(h), prior))?
        };
        let start = Equality::natural(
            add(ones(d), one()),
            add(o.clone(), one()),
            refl(nat(), add(o.clone(), one())),
        );
        self.theorem(
            &format!("OnesBalance{d}"),
            &[],
            start.then(balance)?.then(weight_definition(d).reverse())?,
        )?;
        let x = fill(d, b.clone());
        let e = value_definition(d, x.clone());
        let e = if d == 0 {
            e.then(Equality::natural(
                bit_value(x),
                bit_value(b.clone()),
                refl(nat(), bit_value(b.clone())),
            ))?
        } else {
            let h = d - 1;
            let child = known(
                &format!("FillValue{h}"),
                vec![b.clone()],
                value(h, fill(h, b.clone())),
                mul(ones(h), bit_value(b.clone())),
            );
            let lo = Equality::natural(
                value(h, project(x.clone(), false)),
                child.left.clone(),
                refl(nat(), child.left.clone()),
            )
            .then(child.clone())?;
            let hi = Equality::natural(
                value(h, project(x, true)),
                child.left.clone(),
                refl(nat(), child.left.clone()),
            )
            .then(child)?;
            e.then(lo.clone().on_left(mul(weight(h), hi.left.clone())))?
                .then(mul_right(weight(h), hi).on_right(lo.right))?
        };
        let p = self.polynomial(&args, e.right.clone(), mul(o.clone(), bit_value(b.clone())))?;
        let odef = Equality::natural(ones(d), o.clone(), refl(nat(), o));
        self.theorem(
            &format!("FillValue{d}"),
            &args,
            e.then(p)?.then(mul_left(odef.reverse(), bit_value(b)))?,
        )
    }
    pub(super) fn extension(&mut self, s: usize, t: usize) -> Result<()> {
        let (x, b) = (var("x"), var("bit"));
        let args = [("x", cube(s)), ("bit", boolean())];
        let (body, m) = if s == t {
            (x.clone(), zero())
        } else {
            let h = t - 1;
            (
                lam(
                    "_extension_high",
                    boolean(),
                    choose(
                        var("_extension_high"),
                        fill(h, b.clone()),
                        extend(s, h, x.clone(), b.clone()),
                        cube(h),
                    ),
                ),
                add(mask(s, h), mul(weight(h), ones(h))),
            )
        };
        self.emit(&format!("Extend{s}To{t}"), &args, cube(t), body, false)?;
        self.emit(&format!("Mask{s}To{t}"), &[], nat(), m.clone(), false)?;
        let e = if s == t {
            self.polynomial(&[], add(weight(s), m.clone()), weight(t))?
        } else {
            let h = t - 1;
            let e = law(
                "AddAssoc",
                vec![weight(s), mask(s, h), mul(weight(h), ones(h))],
                add(add(weight(s), mask(s, h)), mul(weight(h), ones(h))),
                add(weight(s), m.clone()),
            )
            .reverse();
            let e = e.then(
                known(
                    &format!("MaskBalance{s}To{h}"),
                    vec![],
                    add(weight(s), mask(s, h)),
                    weight(h),
                )
                .on_left(mul(weight(h), ones(h))),
            )?;
            let p = self.polynomial(&[], e.right.clone(), mul(weight(h), add(ones(h), one())))?;
            e.then(p)?
                .then(mul_right(
                    weight(h),
                    known(
                        &format!("OnesBalance{h}"),
                        vec![],
                        add(ones(h), one()),
                        weight(h),
                    ),
                ))?
                .then(weight_definition(t).reverse())?
        };
        let start = Equality::natural(
            add(weight(s), mask(s, t)),
            e.left.clone(),
            refl(nat(), e.left.clone()),
        );
        self.theorem(&format!("MaskBalance{s}To{t}"), &[], start.then(e)?)?;
        let ext = extend(s, t, x.clone(), b.clone());
        let e = if s == t {
            Equality::natural(
                value(t, ext),
                value(s, x.clone()),
                refl(nat(), value(s, x.clone())),
            )
        } else {
            let h = t - 1;
            let e = value_definition(t, ext.clone());
            let lo = known(
                &format!("ExtensionValue{s}To{h}"),
                vec![x.clone(), b.clone()],
                value(h, extend(s, h, x.clone(), b.clone())),
                add(value(s, x.clone()), mul(mask(s, h), bit_value(b.clone()))),
            );
            let lo = Equality::natural(
                value(h, project(ext.clone(), false)),
                lo.left.clone(),
                refl(nat(), lo.left.clone()),
            )
            .then(lo)?;
            let hi = known(
                &format!("FillValue{h}"),
                vec![b.clone()],
                value(h, fill(h, b.clone())),
                mul(ones(h), bit_value(b.clone())),
            );
            let hi = Equality::natural(
                value(h, project(ext, true)),
                hi.left.clone(),
                refl(nat(), hi.left.clone()),
            )
            .then(hi)?;
            e.then(lo.clone().on_left(mul(weight(h), hi.left.clone())))?
                .then(mul_right(weight(h), hi).on_right(lo.right))?
        };
        let expanded = add(value(s, x.clone()), mul(m, bit_value(b.clone())));
        let target = add(value(s, x), mul(mask(s, t), bit_value(b)));
        let p = self.polynomial(&args, e.right.clone(), expanded.clone())?;
        let end = Equality::natural(target, expanded.clone(), refl(nat(), expanded)).reverse();
        self.theorem(
            &format!("ExtensionValue{s}To{t}"),
            &args,
            e.then(p)?.then(end)?,
        )
    }
    pub(super) fn truncation(&mut self, s: usize, t: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", cube(s))];
        let (l, q, r) = if s == t {
            (x.clone(), zero(), one())
        } else {
            let h = s - 1;
            (
                low(h, t, project(x.clone(), false)),
                add(
                    quotient(h, t, project(x.clone(), false)),
                    mul(ratio(h, t), value(h, project(x.clone(), true))),
                ),
                mul(weight(h), ratio(h, t)),
            )
        };
        self.emit(&format!("Low{s}To{t}"), &args, cube(t), l.clone(), false)?;
        self.emit(&format!("Quotient{s}To{t}"), &args, nat(), q.clone(), false)?;
        self.emit(&format!("Ratio{s}To{t}"), &[], nat(), r.clone(), false)?;
        let e = if s == t {
            self.polynomial(&[], mul(weight(t), r), weight(s))?
        } else {
            let h = s - 1;
            let e = self.polynomial(
                &[],
                mul(weight(t), r),
                mul(weight(h), mul(weight(t), ratio(h, t))),
            )?;
            e.then(mul_right(
                weight(h),
                known(
                    &format!("RatioBalance{h}To{t}"),
                    vec![],
                    mul(weight(t), ratio(h, t)),
                    weight(h),
                ),
            ))?
            .then(weight_definition(s).reverse())?
        };
        let start = Equality::natural(
            mul(weight(t), ratio(s, t)),
            e.left.clone(),
            refl(nat(), e.left.clone()),
        );
        self.theorem(&format!("RatioBalance{s}To{t}"), &[], start.then(e)?)?;
        let e = if s == t {
            Equality::reflexive(value(s, x.clone()))
        } else {
            let h = s - 1;
            let e = value_definition(s, x.clone());
            let xl = project(x.clone(), false);
            let xh = project(x.clone(), true);
            let child = known(
                &format!("TruncationBalance{h}To{t}"),
                vec![xl.clone()],
                value(h, xl.clone()),
                add(
                    value(t, low(h, t, xl.clone())),
                    mul(weight(t), quotient(h, t, xl)),
                ),
            );
            let weight_factor = known(
                &format!("RatioBalance{h}To{t}"),
                vec![],
                mul(weight(t), ratio(h, t)),
                weight(h),
            )
            .reverse();
            e.then(child.clone().on_left(mul(weight(h), value(h, xh.clone()))))?
                .then(mul_left(weight_factor, value(h, xh)).on_right(child.right))?
        };
        let expanded = add(value(t, l), mul(weight(t), q));
        let target = add(
            value(t, low(s, t, x.clone())),
            mul(weight(t), quotient(s, t, x)),
        );
        let p = self.polynomial(&args, e.right.clone(), expanded.clone())?;
        let end = Equality::natural(target, expanded.clone(), refl(nat(), expanded)).reverse();
        self.theorem(
            &format!("TruncationBalance{s}To{t}"),
            &args,
            e.then(p)?.then(end)?,
        )
    }
}
