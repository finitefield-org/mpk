//! Half magnitude characterizes the actual minimum signed input word.
use super::*;

impl Bounds<'_> {
    fn half_forces_sign(&mut self, w: usize, absolute: bool) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let h = half(d);
        let a = value(
            d,
            if absolute {
                wc(w, "AbsBits", vec![x.clone()])
            } else {
                x.clone()
            },
        );
        let s = at(x.clone(), w, w - 1)?;
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_literal",
                eq(boolean(), s.clone(), env["flag"].clone()),
                pi(
                    "_half",
                    eq(nat(), a.clone(), h.clone()),
                    eq(boolean(), s.clone(), bit(true)),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["flag"].clone();
            let body = if flag == bit(true) {
                var("_literal")
            } else {
                let a_to_h = Equality::natural(a.clone(), h.clone(), var("_half"));
                let bad = less_left(a_to_h, h.clone())
                    .reverse()
                    .then(Equality::boolean(
                        less(a.clone(), h.clone()),
                        bit(true),
                        sb(
                            &format!(
                                "W{w}.Nonnegative{}HalfBound",
                                if absolute { "Magnitude" } else { "" }
                            ),
                            vec![x.clone(), var("_literal")],
                        ),
                    ))?;
                let contradiction = Equality::boolean(
                    less(h.clone(), h.clone()),
                    bit(false),
                    nc("LessIrreflexive", vec![h.clone()]),
                )
                .reverse()
                .then(bad)?;
                bc(
                    "FalseElim",
                    vec![eq(boolean(), s.clone(), bit(true)), contradiction.proof],
                )
            };
            Ok(lam(
                "_literal",
                eq(boolean(), s.clone(), flag),
                lam("_half", eq(nat(), a.clone(), h.clone()), body),
            ))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.theorem(
            &format!(
                "W{w}.{}HalfForcesSign",
                if absolute { "Magnitude" } else { "Value" }
            ),
            &[("x", word_type(w)?), ("half", eq(nat(), a, h))],
            Equality::boolean(
                s.clone(),
                bit(true),
                app(proof, vec![s.clone(), refl(boolean(), s), var("half")]),
            ),
        )
    }
    pub(super) fn minimum_equivalence(&mut self, w: usize) -> Result<()> {
        self.half_forces_sign(w, false)?;
        self.half_forces_sign(w, true)?;
        let d = depth(w)?;
        let x = var("x");
        let (u, a, l, h) = (
            value(d, x.clone()),
            value(d, wc(w, "AbsBits", vec![x.clone()])),
            low(d, x.clone()),
            half(d),
        );
        for from_magnitude in [true, false] {
            let input = if from_magnitude { a.clone() } else { u.clone() };
            let output = if from_magnitude { u.clone() } else { a.clone() };
            let sign = sb(
                &format!(
                    "W{w}.{}HalfForcesSign",
                    if from_magnitude { "Magnitude" } else { "Value" }
                ),
                vec![x.clone(), var("half")],
            );
            let balance = instance(
                &format!("W{w}.NegativeMagnitudeLowBalance"),
                vec![x.clone(), sign.clone()],
                add(a.clone(), l.clone()),
                h.clone(),
            );
            let split = instance(
                &format!("W{w}.NegativeValue"),
                vec![x.clone(), sign],
                u.clone(),
                add(l.clone(), h.clone()),
            );
            let input_eq = Equality::natural(input.clone(), h.clone(), var("half"));
            let zero_left = if from_magnitude {
                nlaw(
                    "AddComm",
                    vec![l.clone(), h.clone()],
                    add(l.clone(), h.clone()),
                    add(h.clone(), l.clone()),
                )
                .then(
                    input_eq
                        .on_left(l.clone())
                        .reverse()
                        .then(balance.clone())?,
                )?
            } else {
                split.clone().reverse().then(input_eq)?
            };
            let cancel = zero_left.then(
                nlaw(
                    "ZeroAdd",
                    vec![h.clone()],
                    add(zero(), h.clone()),
                    h.clone(),
                )
                .reverse(),
            )?;
            let low_zero = Equality::natural(
                l.clone(),
                zero(),
                nc(
                    "AddCancel",
                    vec![l.clone(), zero(), h.clone(), cancel.proof],
                ),
            );
            let e = if from_magnitude {
                split.then(low_zero.on_left(h.clone()))?.then(nlaw(
                    "ZeroAdd",
                    vec![h.clone()],
                    add(zero(), h.clone()),
                    h.clone(),
                ))?
            } else {
                low_zero
                    .on_right(a.clone())
                    .then(nlaw(
                        "AddZero",
                        vec![a.clone()],
                        add(a.clone(), zero()),
                        a.clone(),
                    ))?
                    .reverse()
                    .then(balance)?
            };
            if e.left != output {
                return Err(GoOrdinaryError::Linkage);
            }
            self.theorem(
                &format!(
                    "W{w}.{}Half{}",
                    if from_magnitude { "Magnitude" } else { "Value" },
                    if from_magnitude { "Value" } else { "Magnitude" }
                ),
                &[("x", word_type(w)?), ("half", eq(nat(), input, h.clone()))],
                e,
            )?;
        }
        for forward in [true, false] {
            let input = if forward { a.clone() } else { u.clone() };
            let output = if forward { u.clone() } else { a.clone() };
            let same = nc("EqualReflect", vec![input.clone(), h.clone(), var("valid")]);
            let converted = sb(
                &format!(
                    "W{w}.{}Half{}",
                    if forward { "Magnitude" } else { "Value" },
                    if forward { "Value" } else { "Magnitude" }
                ),
                vec![x.clone(), same],
            );
            self.theorem(
                &format!(
                    "W{w}.MagnitudeHalf{}",
                    if forward { "Reflect" } else { "Complete" }
                ),
                &[
                    ("x", word_type(w)?),
                    ("valid", eq(boolean(), equal(input, h.clone()), bit(true))),
                ],
                Equality::boolean(
                    equal(output.clone(), h.clone()),
                    bit(true),
                    nc("EqualComplete", vec![output, h.clone(), converted]),
                ),
            )?;
        }
        let (am, um) = (equal(a, h.clone()), equal(u, h));
        let f = lam(
            "_magnitude",
            eq(boolean(), am.clone(), bit(true)),
            sb(
                &format!("W{w}.MagnitudeHalfReflect"),
                vec![x.clone(), var("_magnitude")],
            ),
        );
        let b = lam(
            "_minimum",
            eq(boolean(), um.clone(), bit(true)),
            sb(
                &format!("W{w}.MagnitudeHalfComplete"),
                vec![x.clone(), var("_minimum")],
            ),
        );
        self.theorem(
            &format!("W{w}.MagnitudeHalfIsMinimum"),
            &[("x", word_type(w)?)],
            Equality::boolean(
                am.clone(),
                um.clone(),
                call(
                    "Mpk.Go.Practical.Word.SignedProduct.ReflectionAgreement",
                    vec![am, um, f, b],
                ),
            ),
        )
    }
}
