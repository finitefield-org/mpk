//! Bind the structural equations to every actual source conversion circuit.
use super::*;

impl Conversions<'_> {
    fn literal_values(&self, w: usize, a: Expr, b: Expr) -> Result<Equality> {
        let mut args = vec![a.clone(), b.clone()];
        for i in 0..w {
            args.push(refl(boolean(), at(b.clone(), w, i)?));
        }
        Ok(Equality::natural(
            value(depth(w)?, a),
            value(depth(w)?, b),
            numeric(&format!("LiteralValueCongruence{w}"), args),
        ))
    }
    pub(super) fn family(&mut self, w: usize, t: usize, sg: bool) -> Result<()> {
        let (s, d) = (depth(w)?, depth(t)?);
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let name = format!("W{w}.{}To{t}", if sg { "Signed" } else { "Unsigned" });
        let result = converted(w, t, sg, x.clone());
        let (u, r) = (value(s, x.clone()), value(d, result.clone()));
        let top = if w >= t {
            at(x.clone(), w, t - 1)?
        } else if sg {
            sign(w, x.clone())?
        } else {
            bit(false)
        };
        self.theorem(
            &format!("{name}.Top"),
            &args,
            Equality::boolean(sign(t, result.clone())?, top.clone(), refl(boolean(), top)),
        )?;
        let e = if w >= t {
            let actual = self.literal_values(t, result.clone(), low(s, d, x.clone()))?;
            known(
                &format!("TruncationBalance{s}To{d}"),
                vec![x.clone()],
                u.clone(),
                add(
                    value(d, low(s, d, x.clone())),
                    mul(weight(d), quotient(s, d, x.clone())),
                ),
            )
            .then(
                actual
                    .reverse()
                    .on_left(mul(weight(d), quotient(s, d, x.clone()))),
            )?
        } else {
            let flag = if sg { sign(w, x.clone())? } else { bit(false) };
            let actual =
                self.literal_values(t, result.clone(), extend(s, d, x.clone(), flag.clone()))?;
            actual.then(known(
                &format!("ExtensionValue{s}To{d}"),
                vec![x.clone(), flag.clone()],
                value(d, extend(s, d, x.clone(), flag.clone())),
                add(u.clone(), mul(mask(s, d), bit_value(flag))),
            ))?
        };
        self.theorem(&format!("{name}.UnsignedConservation"), &args, e.clone())?;
        self.theorem(
            &format!("{name}.ResultBound"),
            &args,
            Equality::boolean(
                less(r.clone(), weight(d)),
                bit(true),
                call(
                    &format!("Mpk.Go.Practical.Word.Order.W{t}.ValueBound"),
                    vec![result.clone()],
                ),
            ),
        )?;
        for view_signed in [false, true] {
            self.modular(w, t, sg, view_signed, &name, &args, x.clone(), e.clone())?;
        }
        if w <= t && sg {
            self.exact_signed(w, t, &name, &args, x.clone(), e.clone())?;
        }
        if w <= t && !sg {
            let e = if w == t {
                identity_value(w, x.clone(), r.clone(), e)?.reverse()
            } else {
                let z = mul_right(mask(s, d), bit_literal(false)).on_right(u.clone());
                let e = e.then(z)?;
                let p = self.polynomial(&args, e.right.clone(), u.clone())?;
                e.then(p)?
            };
            self.theorem(&format!("{name}.UnsignedExact"), &args, e)?;
        }
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "All source/target views instantiate the same proved scalar law."
    )]
    fn modular(
        &mut self,
        w: usize,
        t: usize,
        sg: bool,
        view_signed: bool,
        name: &str,
        args: &[Binder],
        x: Expr,
        e: Equality,
    ) -> Result<()> {
        let (s, d) = (depth(w)?, depth(t)?);
        let result = converted(w, t, sg, x.clone());
        let (u, r) = (value(s, x.clone()), value(d, result.clone()));
        let sx = if sg {
            bit_value(sign(w, x.clone())?)
        } else {
            zero()
        };
        let sy = if view_signed {
            bit_value(sign(t, result.clone())?)
        } else {
            zero()
        };
        let q = if w >= t {
            quotient(s, d, x.clone())
        } else {
            zero()
        };
        let cp = if w >= t {
            add(q.clone(), sy.clone())
        } else {
            sy.clone()
        };
        let cn = if w >= t {
            mul(ratio(s, d), sx.clone())
        } else {
            sx.clone()
        };
        let ns = if sg {
            mul(weight(s), sx.clone())
        } else {
            zero()
        };
        let nt = if view_signed {
            mul(weight(d), sy.clone())
        } else {
            zero()
        };
        let gl = add(
            u.clone(),
            add(mul(weight(d), sy.clone()), mul(weight(d), cn.clone())),
        );
        let gr = add(
            add(r.clone(), mul(weight(d), cp.clone())),
            mul(weight(s), sx.clone()),
        );
        let left = if view_signed {
            Equality::reflexive(gl.clone())
        } else {
            law("MulZero", vec![weight(d)], mul(weight(d), zero()), zero())
                .on_left(mul(weight(d), cn.clone()))
                .on_right(u.clone())
        };
        let right = if sg {
            Equality::reflexive(gr.clone())
        } else {
            law("MulZero", vec![weight(s)], mul(weight(s), zero()), zero())
                .on_right(add(r.clone(), mul(weight(d), cp.clone())))
        };
        let proof = if w >= t {
            cv(
                "NarrowCross",
                vec![
                    u.clone(),
                    r.clone(),
                    weight(s),
                    weight(d),
                    q,
                    ratio(s, d),
                    sx.clone(),
                    sy,
                    e.proof,
                    cv(&format!("RatioBalance{s}To{d}"), vec![]),
                ],
            )
        } else {
            let e = if sg {
                e
            } else {
                e.then(mul_right(mask(s, d), bit_literal(false)).on_right(u.clone()))?
            };
            cv(
                "WidenCross",
                vec![
                    u.clone(),
                    r.clone(),
                    weight(s),
                    weight(d),
                    mask(s, d),
                    sx,
                    sy,
                    e.proof,
                    cv(&format!("MaskBalance{s}To{d}"), vec![]),
                ],
            )
        };
        let balanced = left
            .reverse()
            .then(Equality::natural(gl, gr, proof))?
            .then(right)?;
        let source = if sg {
            signed(w, x)
        } else {
            int("FromNat", vec![u.clone()])
        };
        let target = if view_signed {
            signed(t, result)
        } else {
            int("FromNat", vec![r.clone()])
        };
        let corrected = int(
            "Add",
            vec![
                target,
                int(
                    "Mul",
                    vec![
                        int("FromNat", vec![weight(d)]),
                        int("Make", vec![cp.clone(), cn.clone()]),
                    ],
                ),
            ],
        );
        self.emit(
            &format!(
                "{name}.{}Refinement",
                if view_signed { "Signed" } else { "Unsigned" }
            ),
            args,
            int("Equivalent", vec![source, corrected]),
            cv(
                "ScalarCorrection",
                vec![u, r, ns, nt, weight(d), cp, cn, balanced.proof],
            ),
            true,
        )
    }
    fn exact_signed(
        &mut self,
        w: usize,
        t: usize,
        name: &str,
        args: &[Binder],
        x: Expr,
        e: Equality,
    ) -> Result<()> {
        let (s, d) = (depth(w)?, depth(t)?);
        let result = converted(w, t, true, x.clone());
        let (u, r) = (value(s, x.clone()), value(d, result.clone()));
        let sx = bit_value(sign(w, x.clone())?);
        let (a, b) = (signed(w, x.clone()), signed(t, result.clone()));
        let (left, right) = integer::cross(a.clone(), b.clone())?;
        let ts = map_bit(Equality::boolean(
            sign(t, result.clone())?,
            sign(w, x.clone())?,
            cv(&format!("{name}.Top"), vec![x.clone()]),
        ));
        let lp = mul_right(weight(d), ts).on_right(u.clone());
        let back = if w == t {
            let e = identity_value(w, x.clone(), r.clone(), e)?.reverse();
            e.on_left(mul(weight(s), sx))
        } else {
            let e = e.on_left(mul(weight(s), sx.clone()));
            let p = self.polynomial(
                args,
                e.right.clone(),
                add(u.clone(), mul(add(weight(s), mask(s, d)), sx.clone())),
            )?;
            e.then(p)?.then(
                mul_left(
                    known(
                        &format!("MaskBalance{s}To{d}"),
                        vec![],
                        add(weight(s), mask(s, d)),
                        weight(d),
                    ),
                    sx,
                )
                .on_right(u),
            )?
        };
        let middle = lp.then(back.reverse())?;
        assert_eq!(middle.left, left.right);
        assert_eq!(middle.right, right.right);
        self.relation(&format!("{name}.SignedExact"), args, a, b, middle)
    }
    pub(super) fn count(&mut self, w: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let s = depth(w)?;
        let c = converted(w, 64, true, x.clone());
        let guard = wc(64, "SignedCountDefined", vec![c.clone()]);
        let top = sign(w, x.clone())?;
        let g = bc("Not", vec![top.clone()]);
        self.theorem(
            &format!("W{w}.SignedCountGuard"),
            &args,
            Equality::boolean(guard.clone(), g.clone(), refl(boolean(), g)),
        )?;
        let valid = [
            args[0].clone(),
            ("defined", eq(boolean(), guard, bit(true))),
        ];
        let negated = Equality::boolean(bc("Not", vec![top.clone()]), bit(true), var("defined"));
        let flag = Equality::boolean(
            bc("Not", vec![bc("Not", vec![top.clone()])]),
            top.clone(),
            bc("DoubleNot", vec![top.clone()]),
        )
        .reverse()
        .then(negated.mapped(
            lam("_flag", boolean(), bc("Not", vec![var("_flag")])),
            bc("Not", vec![bc("Not", vec![top])]),
            bc("Not", vec![bit(true)]),
            boolean(),
        ))?
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        let e = if w == 64 {
            let old = known(
                "W64.SignedTo64.UnsignedConservation",
                vec![x.clone()],
                value(6, x.clone()),
                add(
                    value(6, c.clone()),
                    mul(weight(6), quotient(6, 6, x.clone())),
                ),
            );
            identity_value(64, x.clone(), value(6, c.clone()), old)?.reverse()
        } else {
            let old = known(
                &format!("W{w}.SignedTo64.UnsignedConservation"),
                vec![x.clone()],
                value(6, c),
                add(
                    value(s, x.clone()),
                    mul(mask(s, 6), bit_value(sign(w, x.clone())?)),
                ),
            );
            let bit_zero = map_bit(flag).then(bit_literal(false))?;
            let e = old.then(mul_right(mask(s, 6), bit_zero).on_right(value(s, x.clone())))?;
            let p = self.polynomial(&valid, e.right.clone(), value(s, x.clone()))?;
            e.then(p)?
        };
        self.theorem(&format!("W{w}.SignedCountValue"), &valid, e)?;
        if w < 64 {
            let c = converted(w, 64, false, x);
            self.theorem(
                &format!("W{w}.UnsignedCountGuard"),
                &args,
                Equality::boolean(
                    wc(64, "SignedCountDefined", vec![c]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ),
            )?;
        }
        Ok(())
    }
}

fn identity_value(w: usize, x: Expr, r: Expr, e: Equality) -> Result<Equality> {
    let d = depth(w)?;
    let q = Equality::natural(quotient(d, d, x), zero(), refl(nat(), zero()));
    let e = e.then(mul_right(weight(d), q).on_right(r.clone()))?;
    let e = e.then(
        law("MulZero", vec![weight(d)], mul(weight(d), zero()), zero()).on_right(r.clone()),
    )?;
    e.then(law("AddZero", vec![r.clone()], add(r.clone(), zero()), r))
}
