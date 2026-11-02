//! Exact discarded-bit conservation for the actual shift-and-insert circuit.
use super::*;

fn tree_push(d: usize, x: Expr, b: Expr) -> Expr {
    numeric(&format!("Push{d}"), vec![x, b])
}
fn top(d: usize, x: Expr) -> Expr {
    numeric(&format!("Top{d}"), vec![x])
}

impl Numeric<'_> {
    pub(super) fn push_laws(&mut self) -> Result<()> {
        let (w, lo, hi, lr, hr, b, mid, out) = (
            var("weight"),
            var("lo"),
            var("hi"),
            var("lr"),
            var("hr"),
            var("b"),
            var("mid"),
            var("out"),
        );
        let ll = add(lr.clone(), mul(w.clone(), mid.clone()));
        let rr = add(mul(two(), lo.clone()), b.clone());
        let hl = add(hr.clone(), mul(w.clone(), out.clone()));
        let hh = add(mul(two(), hi.clone()), mid.clone());
        let mut args: Vec<Binder> = ["weight", "lo", "hi", "lr", "hr", "b", "mid", "out"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect();
        args.extend([
            ("low", eq(nat(), ll.clone(), rr.clone())),
            ("high", eq(nat(), hl.clone(), hh.clone())),
        ]);
        let left = add(
            add(lr.clone(), mul(w.clone(), hr)),
            mul(mul(w.clone(), w.clone()), out),
        );
        let stage = add(lr, mul(w.clone(), hl.clone()));
        let e = self.polynomial(&args, left, stage)?.then(
            mul_right(w.clone(), Equality::natural(hl, hh, var("high"))).on_right(var("lr")),
        )?;
        let next = self.polynomial(
            &args,
            e.right.clone(),
            add(ll.clone(), mul(w.clone(), mul(two(), hi.clone()))),
        )?;
        let e = e.then(next)?.then(
            Equality::natural(ll, rr, var("low")).on_left(mul(w.clone(), mul(two(), hi.clone()))),
        )?;
        let right = add(mul(two(), add(lo, mul(w, hi))), b);
        let next = self.polynomial(&args, e.right.clone(), right)?;
        self.theorem("PushCombine", &args, e.then(next)?)?;
        for d in 0..=6 {
            let (x, b) = (var("x"), var("b"));
            let top_body = if d == 0 {
                x.clone()
            } else {
                top(d - 1, project(x.clone(), true))
            };
            self.emit(
                &format!("Top{d}"),
                &[("x", cube(d))],
                boolean(),
                top_body,
                false,
            )?;
            let pushed = if d == 0 {
                b.clone()
            } else {
                lam(
                    "_half",
                    boolean(),
                    choose(
                        var("_half"),
                        tree_push(
                            d - 1,
                            project(x.clone(), true),
                            top(d - 1, project(x.clone(), false)),
                        ),
                        tree_push(d - 1, project(x.clone(), false), b.clone()),
                        cube(d - 1),
                    ),
                )
            };
            self.emit(
                &format!("Push{d}"),
                &[("x", cube(d)), ("b", boolean())],
                cube(d),
                pushed,
                false,
            )?;
            let lhs = add(
                value(d, tree_push(d, x.clone(), b.clone())),
                mul(weight(d), bit_value(top(d, x.clone()))),
            );
            let rhs = add(mul(two(), value(d, x.clone())), bit_value(b.clone()));
            let e = if d == 0 {
                let first = value_definition(0, tree_push(0, x.clone(), b.clone()))
                    .on_left(mul(weight(0), bit_value(top(0, x.clone()))));
                let literal = Equality::natural(
                    first.right.clone(),
                    add(bit_value(b.clone()), mul(weight(0), bit_value(x.clone()))),
                    refl(
                        nat(),
                        add(bit_value(b.clone()), mul(weight(0), bit_value(x.clone()))),
                    ),
                );
                let e = first.then(literal)?.then(
                    mul_left(weight_definition(0), bit_value(x.clone()))
                        .on_right(bit_value(b.clone())),
                )?;
                let next = self.polynomial(
                    &[("x", boolean()), ("b", boolean())],
                    e.right.clone(),
                    add(mul(two(), bit_value(x.clone())), bit_value(b.clone())),
                )?;
                e.then(next)?.then(
                    mul_right(two(), value_definition(0, x.clone()).reverse())
                        .on_left(bit_value(b.clone())),
                )?
            } else {
                let h = d - 1;
                let (xl, xh) = (project(x.clone(), false), project(x.clone(), true));
                let lp = tree_push(h, xl.clone(), b.clone());
                let hp = tree_push(h, xh.clone(), top(h, xl.clone()));
                let split = add(value(h, lp.clone()), mul(weight(h), value(h, hp.clone())));
                let whole = tree_push(d, x.clone(), b.clone());
                let mut projections = vec![];
                for (high, child) in [(false, lp.clone()), (true, hp.clone())] {
                    let name = format!("PushProjection{d}{}", if high { "High" } else { "Low" });
                    let e = Equality {
                        ty: cube(h),
                        left: project(whole.clone(), high),
                        right: child.clone(),
                        proof: refl(cube(h), child),
                    };
                    self.theorem(&name, &[("x", cube(d)), ("b", boolean())], e.clone())?;
                    projections.push(map_value(
                        h,
                        Equality {
                            proof: numeric(&name, vec![x.clone(), b.clone()]),
                            ..e
                        },
                    ));
                }
                let low = projections[0].clone();
                let high = projections[1].clone();
                let e = value_definition(d, whole)
                    .then(low.on_left(mul(weight(h), high.left.clone())))?
                    .then(mul_right(weight(h), high).on_right(value(h, lp.clone())))?
                    .on_left(mul(weight(d), bit_value(top(d, x.clone()))));
                let selected_top = top(h, xh.clone());
                let top_e = map_bit(Equality::boolean(
                    top(d, x.clone()),
                    selected_top.clone(),
                    refl(boolean(), selected_top.clone()),
                ));
                let e = e.then(mul_right(weight(d), top_e).on_right(split.clone()))?;
                let expanded = add(
                    split.clone(),
                    mul(weight_body(d), bit_value(top(h, xh.clone()))),
                );
                let e = e.then(
                    mul_left(weight_definition(d), bit_value(selected_top)).on_right(split),
                )?;
                let body_right = add(mul(two(), value_body(d, x.clone())), bit_value(b.clone()));
                let proof = numeric(
                    "PushCombine",
                    vec![
                        weight(h),
                        value(h, xl.clone()),
                        value(h, xh.clone()),
                        value(h, lp),
                        value(h, hp),
                        bit_value(b.clone()),
                        bit_value(top(h, xl.clone())),
                        bit_value(top(h, xh.clone())),
                        numeric(&format!("PushConservation{h}"), vec![xl, b.clone()]),
                        numeric(
                            &format!("PushConservation{h}"),
                            vec![xh, top(h, project(x.clone(), false))],
                        ),
                    ],
                );
                e.then(Equality::natural(expanded, body_right, proof))?
                    .then(
                        mul_right(two(), value_definition(d, x.clone()).reverse())
                            .on_left(bit_value(b.clone())),
                    )?
            };
            if e.left != lhs || e.right != rhs {
                return Err(GoOrdinaryError::Linkage);
            }
            self.theorem(
                &format!("PushConservation{d}"),
                &[("x", cube(d)), ("b", boolean())],
                e,
            )?;
        }
        Ok(())
    }

    pub(super) fn push_family(&mut self, width: usize) -> Result<()> {
        let (x, b) = (var("x"), var("b"));
        let d = depth(width)?;
        let tree = tree_push(d, x.clone(), b.clone());
        let actual = wc(width, "PushLow", vec![x.clone(), b.clone()]);
        let names = &ADDRESSES[..d];
        let proposition = |env: &BTreeMap<&'static str, Expr>| {
            let args = names.iter().map(|n| env[n].clone()).collect::<Vec<_>>();
            eq(
                boolean(),
                app(tree.clone(), args.clone()),
                app(actual.clone(), args),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let args = names.iter().map(|n| env[n].clone()).collect::<Vec<_>>();
            Ok(refl(boolean(), app(actual.clone(), args)))
        };
        let proof = boolean_cases(names, &BTreeMap::new(), &proposition, &mut leaf)?;
        let name = format!("W{width}.PushPointwise");
        self.emit(
            &name,
            &[("x", word_type(width)?), ("b", boolean())],
            pointwise(width, tree.clone(), actual.clone())?,
            proof,
            true,
        )?;
        let values = instance(
            &format!("ValueCongruence{d}"),
            vec![
                tree.clone(),
                actual.clone(),
                numeric(&name, vec![x.clone(), b.clone()]),
            ],
            value(d, tree.clone()),
            value(d, actual.clone()),
        );
        let top_e = Equality::boolean(
            top(d, x.clone()),
            at(x.clone(), width, width - 1)?,
            refl(boolean(), at(x.clone(), width, width - 1)?),
        );
        let top_value = map_bit(top_e).reverse();
        let e = values
            .reverse()
            .on_left(mul(weight(d), top_value.left.clone()))
            .then(mul_right(weight(d), top_value).on_right(value(d, tree.clone())))?
            .then(instance(
                &format!("PushConservation{d}"),
                vec![x.clone(), b.clone()],
                add(value(d, tree), mul(weight(d), bit_value(top(d, x.clone())))),
                add(mul(two(), value(d, x.clone())), bit_value(b.clone())),
            ))?;
        self.theorem(
            &format!("W{width}.PushLowConservation"),
            &[("x", word_type(width)?), ("b", boolean())],
            e,
        )
    }
}
