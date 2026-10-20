//! Genuine right shift with one supplied highest bit, using balanced cubes.
use super::*;
pub(super) fn pop(d: usize, x: Expr, b: Expr) -> Expr {
    sh(&format!("Pop{d}"), vec![x, b])
}
pub(super) fn lowest(d: usize, x: Expr) -> Expr {
    sh(&format!("Lowest{d}"), vec![x])
}
impl Shifts<'_> {
    pub(super) fn pop_combine(&mut self) -> Result<()> {
        let (w, lo, hi, lr, hr, b, mid, out) = (
            var("weight"),
            var("low"),
            var("high"),
            var("low_result"),
            var("high_result"),
            var("fill"),
            var("middle"),
            var("lowest"),
        );
        let mut args = [
            "weight",
            "low",
            "high",
            "low_result",
            "high_result",
            "fill",
            "middle",
            "lowest",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let ll = add(lo.clone(), mul(w.clone(), mid.clone()));
        let lright = add(mul(two(), lr.clone()), out.clone());
        let hl = add(hi.clone(), mul(w.clone(), b.clone()));
        let hright = add(mul(two(), hr.clone()), mid.clone());
        args.extend([
            ("low_equation", eq(nat(), ll.clone(), lright.clone())),
            ("high_equation", eq(nat(), hl.clone(), hright.clone())),
        ]);
        let target = add(mul(two(), add(lr, mul(w.clone(), hr.clone()))), out);
        let e = self.polynomial(
            &args,
            add(
                add(lo.clone(), mul(w.clone(), hi)),
                mul(mul(w.clone(), w.clone()), b),
            ),
            add(lo.clone(), mul(w.clone(), hl.clone())),
        )?;
        let e = e.then(
            mul_right(
                w.clone(),
                Equality::natural(hl, hright, var("high_equation")),
            )
            .on_right(lo),
        )?;
        let p = self.polynomial(
            &args,
            e.right.clone(),
            add(ll.clone(), mul(two(), mul(w.clone(), hr.clone()))),
        )?;
        let e = e.then(p)?.then(
            Equality::natural(ll, lright, var("low_equation")).on_left(mul(two(), mul(w, hr))),
        )?;
        let p = self.polynomial(&args, e.right.clone(), target)?;
        self.theorem("PopCombine", &args, e.then(p)?)
    }
    pub(super) fn pop(&mut self, d: usize) -> Result<()> {
        let (x, b) = (var("x"), var("fill"));
        let args = [("x", cube(d)), ("fill", boolean())];
        let low_body = if d == 0 {
            x.clone()
        } else {
            lowest(d - 1, project(x.clone(), false))
        };
        self.emit(
            &format!("Lowest{d}"),
            &args[..1],
            boolean(),
            low_body,
            false,
        )?;
        let body = if d == 0 {
            b.clone()
        } else {
            let h = d - 1;
            lam(
                "_pop_high_half",
                boolean(),
                choose(
                    var("_pop_high_half"),
                    pop(h, project(x.clone(), true), b.clone()),
                    pop(
                        h,
                        project(x.clone(), false),
                        lowest(h, project(x.clone(), true)),
                    ),
                    cube(h),
                ),
            )
        };
        self.emit(&format!("Pop{d}"), &args, cube(d), body, false)?;
        let r = pop(d, x.clone(), b.clone());
        self.theorem(
            &format!("PopTop{d}"),
            &args,
            Equality::boolean(
                cube_at(d, r.clone(), (1 << d) - 1),
                b.clone(),
                refl(boolean(), b.clone()),
            ),
        )?;
        let (left, right) = if d == 0 {
            let left = value_definition(0, x.clone())
                .on_left(mul(weight(0), bit_value(b.clone())))
                .then(
                    mul_left(weight_definition(0), bit_value(b.clone()))
                        .on_right(bit_value(x.clone())),
                )?;
            let rval =
                cube_equation(0, r.clone(), b.clone()).then(value_definition(0, b.clone()))?;
            let low = map_bit(Equality::boolean(
                lowest(0, x.clone()),
                x.clone(),
                refl(boolean(), x.clone()),
            ));
            let right = mul_right(two(), rval)
                .on_left(bit_value(lowest(0, x.clone())))
                .then(low.on_right(mul(two(), bit_value(b.clone()))))?;
            (left, right)
        } else {
            let h = d - 1;
            let (xl, xh) = (project(x.clone(), false), project(x.clone(), true));
            let lp = pop(h, xl.clone(), lowest(h, xh.clone()));
            let hp = pop(h, xh.clone(), b.clone());
            let (a, ah, lr, hr, bb, mid, out) = (
                value(h, xl.clone()),
                value(h, xh.clone()),
                value(h, lp.clone()),
                value(h, hp.clone()),
                bit_value(b.clone()),
                bit_value(lowest(h, xh.clone())),
                bit_value(lowest(h, xl.clone())),
            );
            let e = value_definition(d, x.clone())
                .on_left(mul(weight(d), bb.clone()))
                .then(
                    mul_left(weight_definition(d), bb.clone()).on_right(value_body(d, x.clone())),
                )?;
            let target = add(
                mul(two(), add(lr.clone(), mul(weight(h), hr.clone()))),
                out.clone(),
            );
            let combined = known(
                "PopCombine",
                vec![
                    weight(h),
                    a.clone(),
                    ah.clone(),
                    lr.clone(),
                    hr.clone(),
                    bb.clone(),
                    mid.clone(),
                    out.clone(),
                    sh(
                        &format!("PopConservation{h}"),
                        vec![xl.clone(), lowest(h, xh.clone())],
                    ),
                    sh(&format!("PopConservation{h}"), vec![xh, b.clone()]),
                ],
                add(
                    add(a, mul(weight(h), ah)),
                    mul(mul(weight(h), weight(h)), bb),
                ),
                target,
            );
            let left = e.then(combined)?;
            let lo = cube_equation(h, project(r.clone(), false), lp);
            let hi = cube_equation(h, project(r.clone(), true), hp);
            let result = value_definition(d, r.clone())
                .then(lo.on_left(mul(weight(h), hi.left.clone())))?
                .then(mul_right(weight(h), hi).on_right(lr.clone()))?;
            let low = map_bit(Equality::boolean(
                lowest(d, x.clone()),
                lowest(h, xl.clone()),
                refl(boolean(), lowest(h, xl)),
            ));
            let right = mul_right(two(), result)
                .on_left(bit_value(lowest(d, x.clone())))
                .then(low.on_right(mul(two(), add(lr, mul(weight(h), hr)))))?;
            (left, right)
        };
        let p = self.polynomial(&args, left.right.clone(), right.right.clone())?;
        self.theorem(
            &format!("PopConservation{d}"),
            &args,
            left.then(p)?.then(right.reverse())?,
        )?;
        self.value_bound(d)
    }
    fn value_bound(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let u = value(d, x.clone());
        let not = call(
            &format!("Mpk.Go.Practical.Word.Order.NotCube{d}"),
            vec![x.clone()],
        );
        let g = value(d, not);
        let balance = Equality::natural(
            add(add(u.clone(), g.clone()), one()),
            weight(d),
            call(
                &format!("Mpk.Go.Practical.Word.Order.Complement{d}"),
                vec![x.clone()],
            ),
        )
        .reverse();
        let balance = balance
            .then(law(
                "AddAssoc",
                vec![u.clone(), g.clone(), one()],
                add(add(u.clone(), g.clone()), one()),
                add(u.clone(), add(g.clone(), one())),
            ))?
            .then(
                law(
                    "AddOne",
                    vec![g.clone()],
                    add(g.clone(), one()),
                    succ(g.clone()),
                )
                .on_right(u.clone()),
            )?;
        let e = less_right(u.clone(), balance).then(Equality::boolean(
            less(u.clone(), add(u.clone(), succ(g.clone()))),
            bit(true),
            call(
                "Mpk.Go.Practical.Word.Order.LessPositiveExtension",
                vec![u, g],
            ),
        ))?;
        self.theorem(&format!("CubeValueBound{d}"), &[("x", cube(d))], e)
    }
}
