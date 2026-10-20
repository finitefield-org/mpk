//! Actual signed product, computed correction and exact-result guard.
use super::projection::part;
use super::scalar::Scalars;
use super::*;

fn named(w: usize, name: &str, x: Expr, y: Expr) -> Expr {
    sc(&format!("W{w}.{name}"), vec![x, y])
}
fn signed(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}

impl SignedProduct<'_> {
    pub(super) fn family(&mut self, w: usize) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(w)?), ("y", word_type(w)?)];
        let s = Scalars::word(w, x.clone(), y.clone())?;
        let cp = named(w, "PositiveCorrection", x.clone(), y.clone());
        let cn = named(w, "NegativeCorrection", x.clone(), y.clone());
        let pdef = Equality::natural(cp.clone(), s.positive(), refl(nat(), s.positive()));
        let ndef = Equality::natural(cn.clone(), s.negative(), refl(nat(), s.negative()));
        self.emit(
            &format!("W{w}.PositiveCorrection"),
            &args,
            nat(),
            s.positive(),
            false,
        )?;
        self.emit(
            &format!("W{w}.NegativeCorrection"),
            &args,
            nat(),
            s.negative(),
            false,
        )?;
        self.theorem(
            &format!("W{w}.PositiveCorrectionDefinition"),
            &args,
            pdef.clone(),
        )?;
        self.theorem(
            &format!("W{w}.NegativeCorrectionDefinition"),
            &args,
            ndef.clone(),
        )?;
        let correction = named(w, "Correction", x.clone(), y.clone());
        self.emit(
            &format!("W{w}.Correction"),
            &args,
            integer(),
            ic("Make", vec![cp.clone(), cn.clone()]),
            false,
        )?;
        self.theorem(
            &format!("W{w}.CorrectionPositive"),
            &args,
            Equality::natural(
                part(false, correction.clone()),
                s.positive(),
                refl(nat(), s.positive()),
            ),
        )?;
        self.theorem(
            &format!("W{w}.CorrectionNegative"),
            &args,
            Equality::natural(
                part(true, correction.clone()),
                s.negative(),
                refl(nat(), s.negative()),
            ),
        )?;
        let exact = named(w, "NoOverflow", x.clone(), y.clone());
        self.emit(
            &format!("W{w}.NoOverflow"),
            &args,
            boolean(),
            equal(cp.clone(), cn.clone()),
            false,
        )?;
        let mathematical = ic("Mul", vec![signed(w, x.clone()), signed(w, y.clone())]);
        let result = signed(w, wc(w, "Mul", vec![x.clone(), y.clone()]));
        let residual = ic(
            "Add",
            vec![
                result.clone(),
                ic("Mul", vec![ic("FromNat", vec![s.m.clone()]), correction]),
            ],
        );
        let (lp, rp) = self.cross_parts(w, mathematical.clone(), residual.clone())?;
        let mut scalar = s.values();
        scalar.push(numeric(
            &format!("W{w}.MulValueConservation"),
            vec![x.clone(), y.clone()],
        ));
        let cross = lp
            .then(instance(
                "ResidualCross",
                scalar.clone(),
                s.residual_left(),
                s.residual_right(),
            ))?
            .then(rp.reverse())?;
        self.emit(
            &format!("W{w}.SignedProductConservation"),
            &args,
            ic("Equivalent", vec![mathematical.clone(), residual]),
            cross.proof,
            true,
        )?;
        let (lp, rp) = self.cross_parts(w, mathematical.clone(), result.clone())?;
        let equal_correction = Equality::natural(
            cp.clone(),
            cn.clone(),
            nc("EqualReflect", vec![cp.clone(), cn.clone(), var("valid")]),
        );
        let expanded = pdef
            .clone()
            .reverse()
            .then(equal_correction)?
            .then(ndef.clone())?;
        let mut inputs = scalar.clone();
        inputs.push(expanded.proof);
        let proof = lp
            .clone()
            .then(instance(
                "ExactCrossFromCorrection",
                inputs,
                s.left(),
                s.right(),
            ))?
            .then(rp.clone().reverse())?;
        let mut guarded = args.to_vec();
        guarded.push(("valid", eq(boolean(), exact.clone(), bit(true))));
        self.emit(
            &format!("W{w}.NoOverflowReflect"),
            &guarded,
            ic("Equivalent", vec![mathematical.clone(), result.clone()]),
            proof.proof,
            true,
        )?;
        let same = ic("Equivalent", vec![mathematical.clone(), result.clone()]);
        let exact_cross = lp
            .reverse()
            .then(Equality::natural(
                add(
                    part(false, mathematical.clone()),
                    part(true, result.clone()),
                ),
                add(
                    part(false, result.clone()),
                    part(true, mathematical.clone()),
                ),
                var("same"),
            ))?
            .then(rp)?;
        let mut inputs = scalar;
        inputs.extend([
            sc(&format!("W{w}.ModulusPositive"), vec![]),
            exact_cross.proof,
        ]);
        let e = pdef
            .then(instance(
                "CorrectionFromExactCross",
                inputs,
                s.positive(),
                s.negative(),
            ))?
            .then(ndef.reverse())?;
        let complete = Equality::boolean(
            exact.clone(),
            bit(true),
            nc("EqualComplete", vec![cp, cn, e.proof]),
        );
        let mut complete_args = args.to_vec();
        complete_args.push(("same", same));
        self.theorem(
            &format!("W{w}.NoOverflowComplete"),
            &complete_args,
            complete,
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
                    sc(
                        &format!("W{w}.NoOverflowReflect"),
                        vec![x.clone(), y.clone(), var("_valid")],
                    ),
                ],
            ),
        );
        let b = lam(
            "_same",
            eq(boolean(), integer_equal.clone(), bit(true)),
            sc(
                &format!("W{w}.NoOverflowComplete"),
                vec![
                    x,
                    y,
                    ic("EqualReflect", vec![mathematical, result, var("_same")]),
                ],
            ),
        );
        self.theorem(
            &format!("W{w}.NoOverflowIntegerEquality"),
            &args,
            Equality::boolean(
                exact.clone(),
                integer_equal.clone(),
                sc("ReflectionAgreement", vec![exact, integer_equal, f, b]),
            ),
        )
    }
}
