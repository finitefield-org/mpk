//! Actual top bits reflect half thresholds; bounded magnitude sign is minimum.
use super::*;
impl Bounds<'_> {
    pub(super) fn threshold_laws(&mut self) -> Result<()> {
        let (a, h) = (var("a"), var("half"));
        let first = bc("Not", vec![less(a.clone(), h.clone())]);
        let second = bc("Not", vec![less(h.clone(), a.clone())]);
        let yes = Equality::boolean(second.clone(), bit(true), var("bound"));
        let mapped = yes.mapped(
            lam(
                "_flag",
                boolean(),
                bc("And", vec![first.clone(), var("_flag")]),
            ),
            bc("And", vec![first.clone(), second.clone()]),
            bc("And", vec![first.clone(), bit(true)]),
            boolean(),
        );
        let e = mapped
            .then(Equality::boolean(
                bc("And", vec![first.clone(), bit(true)]),
                first.clone(),
                bc("AndTrueRight", vec![first.clone()]),
            ))?
            .reverse()
            .then(Equality::boolean(
                bc("And", vec![first.clone(), second]),
                equal(a.clone(), h.clone()),
                order("NaturalTotalEquality", vec![a.clone(), h.clone()]),
            ))?;
        self.theorem(
            "AtMostThresholdEquals",
            &[
                ("a", nat()),
                ("half", nat()),
                ("bound", eq(boolean(), le(a, h), bit(true))),
            ],
            e,
        )
    }
    pub(super) fn magnitude_sign(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let (u, l, h) = (value(d, x.clone()), low(d, x.clone()), half(d));
        let s = at(x.clone(), w, w - 1)?;
        let args = [("x", word_type(w)?)];
        let split = instance(
            &format!("W{w}.NegativeValue"),
            vec![x.clone(), var("sign")],
            u.clone(),
            add(l.clone(), h.clone()),
        );
        let e = less_left(split, h.clone())
            .then(less_right(
                add(l.clone(), h.clone()),
                nlaw(
                    "ZeroAdd",
                    vec![h.clone()],
                    add(zero(), h.clone()),
                    h.clone(),
                )
                .reverse(),
            ))?
            .then(Equality::boolean(
                less(add(l.clone(), h.clone()), add(zero(), h.clone())),
                less(l.clone(), zero()),
                nc("LessTranslate", vec![l.clone(), zero(), h.clone()]),
            ))?
            .then(Equality::boolean(
                less(l.clone(), zero()),
                bit(false),
                nc("LessZero", vec![l]),
            ))?;
        let mut negative = args.to_vec();
        negative.push(("sign", eq(boolean(), s.clone(), bit(true))));
        self.theorem(&format!("W{w}.NegativeNotBelowHalf"), &negative, e)?;
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_literal",
                eq(boolean(), s.clone(), env["flag"].clone()),
                eq(
                    boolean(),
                    less(u.clone(), h.clone()),
                    bc("Not", vec![env["flag"].clone()]),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["flag"] == bit(true);
            let actual = Equality::boolean(
                less(u.clone(), h.clone()),
                bit(!flag),
                sb(
                    &format!(
                        "W{w}.{}",
                        if flag {
                            "NegativeNotBelowHalf"
                        } else {
                            "NonnegativeHalfBound"
                        }
                    ),
                    vec![x.clone(), var("_literal")],
                ),
            );
            let e = actual.then(
                Equality::boolean(
                    bc("Not", vec![bit(flag)]),
                    bit(!flag),
                    refl(boolean(), bit(!flag)),
                )
                .reverse(),
            )?;
            Ok(lam(
                "_literal",
                eq(boolean(), s.clone(), bit(flag)),
                e.proof,
            ))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.theorem(
            &format!("W{w}.UnsignedHalfLess"),
            &args,
            Equality::boolean(
                less(u.clone(), h.clone()),
                bc("Not", vec![s.clone()]),
                app(proof, vec![s.clone(), refl(boolean(), s)]),
            ),
        )?;
        let abs = wc(w, "AbsBits", vec![x.clone()]);
        let a = value(d, abs.clone());
        let abs_sign = at(abs.clone(), w, w - 1)?;
        let e = Equality::boolean(
            less(a.clone(), h.clone()),
            bc("Not", vec![abs_sign.clone()]),
            sb(&format!("W{w}.UnsignedHalfLess"), vec![abs]),
        );
        let e = bool_not(e)
            .then(Equality::boolean(
                bc("Not", vec![bc("Not", vec![abs_sign.clone()])]),
                abs_sign,
                bc(
                    "DoubleNot",
                    vec![at(wc(w, "AbsBits", vec![x.clone()]), w, w - 1)?],
                ),
            ))?
            .reverse()
            .then(Equality::boolean(
                bc("Not", vec![less(a.clone(), h.clone())]),
                equal(a.clone(), h.clone()),
                sb(
                    "AtMostThresholdEquals",
                    vec![
                        a.clone(),
                        h.clone(),
                        sb(&format!("W{w}.MagnitudeHalfBound"), vec![x.clone()]),
                    ],
                ),
            ))?
            .then(Equality::boolean(
                equal(a, h.clone()),
                equal(u, h),
                sb(&format!("W{w}.MagnitudeHalfIsMinimum"), vec![x]),
            ))?;
        self.theorem(&format!("W{w}.MagnitudeSignIsMinimum"), &args, e)
    }
}
