//! Actual signed negation, computed modular correction and exact-result guard.
use super::projection::part;
use super::*;

impl Magnitude<'_> {
    pub(super) fn signed_negation(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let n = neg(w, x.clone());
        let m = weight(d);
        let cp = add(
            bit_value(sign(w, x.clone())?),
            bit_value(sign(w, n.clone())?),
        );
        let cn = bit_value(wc(w, "DivideDefined", vec![x.clone()]));
        let correction = unary(w, "NegationCorrection", x.clone());
        self.emit(
            &format!("W{w}.NegationCorrection"),
            &args,
            integer(),
            ic("Make", vec![cp.clone(), cn.clone()]),
            false,
        )?;
        for (negative, rhs) in [(false, cp.clone()), (true, cn.clone())] {
            self.theorem(
                &format!(
                    "W{w}.NegationCorrection{}",
                    if negative { "Negative" } else { "Positive" }
                ),
                &args,
                Equality::natural(
                    part(negative, correction.clone()),
                    rhs.clone(),
                    refl(nat(), rhs),
                ),
            )?;
        }
        let exact = unary(w, "NegationNoOverflow", x.clone());
        self.emit(
            &format!("W{w}.NegationNoOverflow"),
            &args,
            boolean(),
            equal(cp.clone(), cn.clone()),
            false,
        )?;
        let mathematical = ic("Negate", vec![signed(w, x.clone())]);
        let result = signed(w, n.clone());
        let residual = ic(
            "Add",
            vec![
                result.clone(),
                ic("Mul", vec![ic("FromNat", vec![m.clone()]), correction]),
            ],
        );
        let (lp, rp) = self.cross_parts(w, mathematical.clone(), residual.clone())?;
        let ux = value(d, x.clone());
        let un = value(d, n);
        let sum = instance(
            &format!("W{w}.NegationValueSum"),
            vec![x.clone()],
            add(ux.clone(), un.clone()),
            mul(m.clone(), cn.clone()),
        );
        let e = self
            .polynomial(
                &args,
                lp.right.clone(),
                add(mul(m.clone(), cp.clone()), mul(m.clone(), cn.clone())),
            )?
            .then(sum.clone().reverse().on_right(mul(m.clone(), cp.clone())))?;
        let tail = self.polynomial(&args, e.right.clone(), rp.right.clone())?;
        self.emit(
            &format!("W{w}.SignedNegationConservation"),
            &args,
            ic("Equivalent", vec![mathematical.clone(), residual]),
            lp.then(e)?.then(tail)?.then(rp.reverse())?.proof,
            true,
        )?;
        let (lp, rp) = self.cross_parts(w, mathematical.clone(), result.clone())?;
        let left = lp.then(self.polynomial(
            &args,
            add(
                mul(m.clone(), bit_value(sign(w, x.clone())?)),
                mul(m.clone(), bit_value(sign(w, neg(w, x.clone()))?)),
            ),
            mul(m.clone(), cp.clone()),
        )?)?;
        let right = rp
            .then(Equality::natural(
                add(un.clone(), ux.clone()),
                add(ux.clone(), un.clone()),
                nc("AddComm", vec![un, ux]),
            ))?
            .then(sum)?;
        let mut guarded = args.to_vec();
        guarded.push(("valid", eq(boolean(), exact.clone(), bit(true))));
        let correction_equal = Equality::natural(
            cp.clone(),
            cn.clone(),
            nc("EqualReflect", vec![cp.clone(), cn.clone(), var("valid")]),
        );
        let reflected = left
            .clone()
            .then(mul_right(m.clone(), correction_equal))?
            .then(right.clone().reverse())?;
        self.emit(
            &format!("W{w}.NegationNoOverflowReflect"),
            &guarded,
            ic("Equivalent", vec![mathematical.clone(), result.clone()]),
            reflected.proof,
            true,
        )?;
        let same = ic("Equivalent", vec![mathematical.clone(), result.clone()]);
        let mut complete = args.to_vec();
        complete.push(("same", same));
        let cross = Equality::natural(
            add(
                part(false, mathematical.clone()),
                part(true, result.clone()),
            ),
            add(
                part(false, result.clone()),
                part(true, mathematical.clone()),
            ),
            var("same"),
        );
        let multiplied = left.reverse().then(cross)?.then(right)?;
        let cancelled = shared(
            "MultiplyCancelPositive",
            vec![
                m,
                shared(&format!("W{w}.ModulusPositive"), vec![]),
                cp.clone(),
                cn.clone(),
                multiplied.proof,
            ],
        );
        self.theorem(
            &format!("W{w}.NegationNoOverflowComplete"),
            &complete,
            Equality::boolean(
                exact.clone(),
                bit(true),
                nc("EqualComplete", vec![cp, cn, cancelled]),
            ),
        )?;
        let integer_equal = ic("Equal", vec![mathematical.clone(), result.clone()]);
        let f = lam(
            "_valid",
            eq(boolean(), exact.clone(), bit(true)),
            ic(
                "EqualFromEquivalent",
                vec![
                    mathematical.clone(),
                    result.clone(),
                    mc(
                        &format!("W{w}.NegationNoOverflowReflect"),
                        vec![x.clone(), var("_valid")],
                    ),
                ],
            ),
        );
        let b = lam(
            "_same",
            eq(boolean(), integer_equal.clone(), bit(true)),
            mc(
                &format!("W{w}.NegationNoOverflowComplete"),
                vec![
                    x,
                    ic("EqualReflect", vec![mathematical, result, var("_same")]),
                ],
            ),
        );
        self.theorem(
            &format!("W{w}.NegationNoOverflowIntegerEquality"),
            &args,
            Equality::boolean(
                exact.clone(),
                integer_equal.clone(),
                shared("ReflectionAgreement", vec![exact, integer_equal, f, b]),
            ),
        )
    }
}
