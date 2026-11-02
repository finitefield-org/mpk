//! A concrete complement is the ordinary strict upper-bound witness.
use super::*;

fn complement(d: usize, x: Expr) -> Expr {
    oc(&format!("NotCube{d}"), vec![x])
}
fn complement_body(d: usize, x: Expr) -> Expr {
    let mut body = bc(
        "Not",
        vec![app(x, ADDRESSES[..d].iter().map(|n| var(n)).collect())],
    );
    for name in ADDRESSES[..d].iter().rev() {
        body = lam(name, boolean(), body);
    }
    body
}

impl Order<'_> {
    pub(super) fn complement_laws(&mut self) -> Result<()> {
        let (w, a, b, ah, bh) = (
            var("weight"),
            var("low"),
            var("not_low"),
            var("high"),
            var("not_high"),
        );
        let low = add(add(a.clone(), b.clone()), one());
        let high = add(add(ah.clone(), bh.clone()), one());
        let mut args: Vec<Binder> = ["weight", "low", "not_low", "high", "not_high"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect();
        args.extend([
            ("low_equation", eq(nat(), low.clone(), w.clone())),
            ("high_equation", eq(nat(), high.clone(), w.clone())),
        ]);
        let lhs = add(
            add(
                add(a, mul(w.clone(), ah.clone())),
                add(b, mul(w.clone(), bh.clone())),
            ),
            one(),
        );
        let stage = add(low.clone(), mul(w.clone(), add(ah, bh)));
        let e = self.polynomial(&args, lhs, stage.clone())?;
        let tail = polynomial::split(&stage, "Add")
            .ok_or(GoOrdinaryError::Linkage)?
            .1;
        let e = e.then(Equality::natural(low, w.clone(), var("low_equation")).on_left(tail))?;
        let next = self.polynomial(&args, e.right.clone(), mul(w.clone(), high.clone()))?;
        let e = e.then(next)?.then(mul_right(
            w.clone(),
            Equality::natural(high, w.clone(), var("high_equation")),
        ))?;
        self.theorem("ComplementCombine", &args, e)?;
        for d in 0..=6 {
            let x = var("x");
            self.emit(
                &format!("NotCube{d}"),
                &[("x", cube(d))],
                cube(d),
                complement_body(d, x.clone()),
                false,
            )?;
            if d == 0 {
                self.complement_bit()?;
                continue;
            }
            let h = d - 1;
            let not = complement(d, x.clone());
            let mut projections = vec![];
            for high in [false, true] {
                let child = complement(h, project(x.clone(), high));
                let name = format!("NotProjection{d}{}", if high { "High" } else { "Low" });
                let e = Equality {
                    ty: cube(h),
                    left: project(not.clone(), high),
                    right: child.clone(),
                    proof: refl(cube(h), child),
                };
                self.theorem(&name, &[("x", cube(d))], e.clone())?;
                projections.push(map_value(
                    h,
                    Equality {
                        proof: oc(&name, vec![x.clone()]),
                        ..e
                    },
                ));
            }
            let lo = projections[0].clone();
            let hi = projections[1].clone();
            let vnot = value_definition(d, not.clone())
                .then(lo.clone().on_left(mul(weight(h), hi.left.clone())))?
                .then(mul_right(weight(h), hi.clone()).on_right(lo.right.clone()))?;
            let vx = value_definition(d, x.clone());
            let start = vx
                .clone()
                .on_left(value(d, not))
                .then(vnot.on_right(vx.right))?
                .on_left(one());
            let (xl, xh) = (project(x.clone(), false), project(x.clone(), true));
            let proof = oc(
                "ComplementCombine",
                vec![
                    weight(h),
                    value(h, xl.clone()),
                    lo.right,
                    value(h, xh.clone()),
                    hi.right,
                    oc(&format!("Complement{h}"), vec![xl]),
                    oc(&format!("Complement{h}"), vec![xh]),
                ],
            );
            let end = mul(weight(h), weight(h));
            let first = start.right.clone();
            let e = start
                .then(Equality::natural(first, end, proof))?
                .then(weight_definition(d).reverse())?;
            self.theorem(&format!("Complement{d}"), &[("x", cube(d))], e)?;
        }
        self.positive_extension()
    }
    fn complement_bit(&mut self) -> Result<()> {
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                add(
                    add(
                        value(0, e["x"].clone()),
                        value(0, complement(0, e["x"].clone())),
                    ),
                    one(),
                ),
                weight(0),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let b = e["x"] == bit(true);
            let x = bit(b);
            let not = complement(0, x.clone());
            let left = value_definition(0, x.clone()).then(bit_literal(b))?;
            let notbit = Equality::boolean(not.clone(), bit(!b), refl(boolean(), bit(!b)));
            let right = value_definition(0, not)
                .then(map_bit(notbit))?
                .then(bit_literal(!b))?;
            let e = left
                .clone()
                .on_left(right.left.clone())
                .then(right.on_right(left.right))?
                .on_left(one());
            let next = self.polynomial(&[], e.right.clone(), two())?;
            Ok(e.then(next)?.then(weight_definition(0).reverse())?.proof)
        };
        let proof = boolean_cases(&["x"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "Complement0",
            &[],
            pi(
                "x",
                boolean(),
                prop(&[("x", var("x"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn positive_extension(&mut self) -> Result<()> {
        let (a, k) = (var("a"), var("k"));
        let e = Equality::boolean(
            less(add(zero(), a.clone()), add(succ(k.clone()), a.clone())),
            less(zero(), succ(k.clone())),
            nc("LessTranslate", vec![zero(), succ(k.clone()), a.clone()]),
        );
        let p = less_left(
            Equality::natural(
                add(zero(), a.clone()),
                a.clone(),
                nc("ZeroAdd", vec![a.clone()]),
            ),
            add(succ(k.clone()), a.clone()),
        );
        let e = p.reverse().then(e)?.then(Equality::boolean(
            less(zero(), succ(k.clone())),
            bit(true),
            nc("LessZeroSucc", vec![k.clone()]),
        ))?;
        let right = add(a.clone(), succ(k.clone()));
        let swap = Equality::natural(
            add(succ(k.clone()), a.clone()),
            right.clone(),
            nc("AddComm", vec![succ(k.clone()), a.clone()]),
        );
        let e = less_right(a.clone(), swap).reverse().then(e)?;
        self.theorem("LessPositiveExtension", &[("a", nat()), ("k", nat())], e)
    }
    pub(super) fn bounds_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let x = var("x");
        let actual = wc(width, "Not", vec![x.clone()]);
        let tree = complement(d, x.clone());
        let word = Equality {
            ty: word_type(width)?,
            left: actual.clone(),
            right: tree.clone(),
            proof: refl(word_type(width)?, tree.clone()),
        };
        let e = map_value(d, word)
            .on_right(value(d, x.clone()))
            .on_left(one())
            .then(instance(
                &format!("Complement{d}"),
                vec![x.clone()],
                add(add(value(d, x.clone()), value(d, tree)), one()),
                weight(d),
            ))?;
        self.theorem(
            &format!("W{width}.Complement"),
            &[("x", word_type(width)?)],
            e.clone(),
        )?;
        let gap = value(d, actual);
        let v = value(d, x.clone());
        let successor = Equality::natural(
            add(gap.clone(), one()),
            succ(gap.clone()),
            nc("AddOne", vec![gap.clone()]),
        );
        let assoc = Equality::natural(
            add(add(v.clone(), gap.clone()), one()),
            add(v.clone(), add(gap.clone(), one())),
            nc("AddAssoc", vec![v.clone(), gap.clone(), one()]),
        );
        let combine = e
            .reverse()
            .then(assoc)?
            .then(successor.on_right(v.clone()))?;
        let bound = less_right(v.clone(), combine).then(Equality::boolean(
            less(v.clone(), add(v.clone(), succ(gap.clone()))),
            bit(true),
            oc("LessPositiveExtension", vec![v, gap]),
        ))?;
        self.theorem(
            &format!("W{width}.ValueBound"),
            &[("x", word_type(width)?)],
            bound,
        )
    }
}
