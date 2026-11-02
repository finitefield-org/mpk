//! Actual magnitude operands supply every range gap and correction premise.
use super::*;

pub(super) fn source_guard(w: usize, y: Expr) -> Expr {
    eq(boolean(), wc(w, "DivideDefined", vec![y]), bit(true))
}
pub(super) fn divisor_positive(w: usize, y: Expr) -> Result<Expr> {
    let ay = wc(w, "AbsBits", vec![y.clone()]);
    let e = Equality::boolean(
        wc(w, "DivideDefined", vec![ay.clone()]),
        wc(w, "DivideDefined", vec![y.clone()]),
        prerequisite(
            "Magnitude",
            &format!("W{w}.MagnitudeDivideDefined"),
            vec![y.clone()],
        ),
    )
    .then(Equality::boolean(
        wc(w, "DivideDefined", vec![y]),
        bit(true),
        var("defined"),
    ))?;
    Ok(prerequisite(
        "Zero",
        &format!("W{w}.DefinedDivisorPositive"),
        vec![ay, e.proof],
    ))
}
pub(super) fn half_balance(w: usize, x: Expr) -> Expr {
    prerequisite("Bounds", &format!("W{w}.MagnitudeHalfBalance"), vec![x])
}
pub(super) fn q_balance(w: usize, x: Expr, y: Expr) -> Expr {
    db(
        &format!("W{w}.QuotientGapBalance"),
        vec![x, y, var("defined")],
    )
}
pub(super) fn q_correction(w: usize, x: Expr, y: Expr) -> Relation {
    Relation::new(
        old_field(w, "QuotientCorrection", x.clone(), y.clone()),
        ic(
            "FromNat",
            vec![bit_value(overflow(w, x.clone(), y.clone()))],
        ),
        db(
            &format!("W{w}.QuotientCorrectionClassified"),
            vec![x, y, var("defined")],
        ),
    )
}
pub(super) fn r_exact(w: usize, x: Expr, y: Expr) -> Relation {
    Relation::new(
        old_field(w, "MathematicalRemainder", x.clone(), y.clone()),
        signed(w, wc(w, "SignedRem", vec![x.clone(), y.clone()])),
        db(
            &format!("W{w}.RemainderRefinementExact"),
            vec![x, y, var("defined")],
        ),
    )
}

impl Bounded<'_> {
    pub(super) fn family(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let plain = [("x", word_type(w)?), ("y", word_type(w)?)];
        let mut args = plain.to_vec();
        args.push(("defined", source_guard(w, y.clone())));
        let ax = wc(w, "AbsBits", vec![x.clone()]);
        let ay = wc(w, "AbsBits", vec![y.clone()]);
        let qw = amount_word(w, true, x.clone(), y.clone());
        let rw = amount_word(w, false, x.clone(), y.clone());
        let (a, b, q, r) = (
            value(d, ax),
            value(d, ay),
            value(d, qw.clone()),
            value(d, rw.clone()),
        );
        let h = half(d);
        let gx = magnitude_gap(w, x.clone());
        let gy = magnitude_gap(w, y.clone());
        let qg = qgap_value(w, x.clone(), y.clone());
        let body = qgap(b.clone(), q.clone(), r.clone(), gx.clone());
        self.emit(
            &format!("W{w}.QuotientGap"),
            &plain,
            nat(),
            body.clone(),
            false,
        )?;
        let gap_def = Equality::natural(qg.clone(), body.clone(), refl(nat(), body.clone()));
        self.theorem(
            &format!("W{w}.QuotientGapDefinition"),
            &plain,
            gap_def.clone(),
        )?;
        let balance = gap_def.on_right(q.clone()).then(Equality::natural(
            add(q.clone(), body),
            h.clone(),
            db(
                "QuotientGapBalance",
                vec![
                    a,
                    b.clone(),
                    q.clone(),
                    r.clone(),
                    gx,
                    h.clone(),
                    divisor_positive(w, y.clone())?,
                    sd(
                        &format!("W{w}.MagnitudeDivisionConservation"),
                        vec![x.clone(), y.clone(), var("defined")],
                    ),
                    half_balance(w, x.clone()),
                ],
            ),
        ))?;
        self.theorem(&format!("W{w}.QuotientGapBalance"), &args, balance)?;
        self.theorem(
            &format!("W{w}.QuotientHalfBound"),
            &args,
            Equality::boolean(
                le(q.clone(), h.clone()),
                bit(true),
                db(
                    &format!("W{w}.WordHalfBound"),
                    vec![qw.clone(), qg.clone(), q_balance(w, x.clone(), y.clone())],
                ),
            ),
        )?;
        let rbound = Equality::boolean(
            less(r.clone(), h.clone()),
            bit(true),
            db(
                "StrictUpperFromBalance",
                vec![
                    r.clone(),
                    b,
                    gy,
                    h.clone(),
                    sd(
                        &format!("W{w}.MagnitudeRemainderBound"),
                        vec![x.clone(), y.clone(), var("defined")],
                    ),
                    half_balance(w, y.clone()),
                ],
            ),
        );
        self.theorem(&format!("W{w}.RemainderStrictHalfBound"), &args, rbound)?;
        let flag = quotient_flag(w, x.clone(), y.clone())?;
        let ov_body = gate(flag.clone(), equal(q.clone(), h.clone()));
        self.emit(
            &format!("W{w}.QuotientOverflow"),
            &plain,
            boolean(),
            ov_body.clone(),
            false,
        )?;
        let ov_def = Equality::boolean(
            overflow(w, x.clone(), y.clone()),
            ov_body.clone(),
            refl(boolean(), ov_body.clone()),
        );
        self.theorem(
            &format!("W{w}.QuotientOverflowDefinition"),
            &plain,
            ov_def.clone(),
        )?;
        let qcorr_generic = Relation::new(
            correction(w, qw.clone(), flag.clone()),
            ic("FromNat", vec![bit_value(ov_body)]),
            db(
                &format!("W{w}.BoundedSelectionCorrection"),
                vec![qw, qg, q_balance(w, x.clone(), y.clone()), flag],
            ),
        );
        let qc = old_field(w, "QuotientCorrection", x.clone(), y.clone());
        let qcdef = Equality {
            ty: integer(),
            left: qc,
            right: qcorr_generic.left.clone(),
            proof: old_field(w, "QuotientCorrectionDefinition", x.clone(), y.clone()),
        };
        let qcorr = qcorr_generic
            .replace_left(qcdef)?
            .replace_right(from_nat_equality(map_bit(ov_def)))?;
        self.relation(&format!("W{w}.QuotientCorrectionClassified"), &args, qcorr)?;
        let sr = sign(w, x.clone())?;
        let rg = strict_gap(r.clone(), h.clone());
        let rb = db(
            &format!("W{w}.RemainderStrictHalfBound"),
            vec![x.clone(), y.clone(), var("defined")],
        );
        let rg_balance = db("StrictGapBalance", vec![r.clone(), h.clone(), rb.clone()]);
        let eq_false = Equality::boolean(
            equal(r.clone(), h.clone()),
            bit(false),
            db("StrictUnequal", vec![r, h, rb]),
        );
        let gate_zero = bool_binary(
            "And",
            Equality::boolean(
                bc("Not", vec![sr.clone()]),
                bc("Not", vec![sr.clone()]),
                refl(boolean(), bc("Not", vec![sr.clone()])),
            ),
            eq_false,
        )?
        .then(Equality::boolean(
            gate(sr.clone(), bit(false)),
            bit(false),
            db("GateZero", vec![sr.clone()]),
        ))?;
        let rz = map_bit(gate_zero).then(bit_literal(false))?;
        let r_target = ic(
            "FromNat",
            vec![bit_value(gate(
                sr.clone(),
                equal(value(d, rw.clone()), half(d)),
            ))],
        );
        let rcor = Relation::new(
            correction(w, rw.clone(), sr.clone()),
            r_target,
            db(
                &format!("W{w}.BoundedSelectionCorrection"),
                vec![rw, rg, rg_balance, sr],
            ),
        );
        let rcdef = Equality {
            ty: integer(),
            left: old_field(w, "RemainderCorrection", x.clone(), y.clone()),
            right: rcor.left.clone(),
            proof: old_field(w, "RemainderCorrectionDefinition", x.clone(), y.clone()),
        };
        self.relation(
            &format!("W{w}.RemainderCorrectionZero"),
            &args,
            rcor.replace_left(rcdef)?
                .replace_right(from_nat_equality(rz).reverse())?,
        )?;
        let rzero = Relation::new(
            old_field(w, "RemainderCorrection", x.clone(), y.clone()),
            ic("FromNat", vec![zero()]),
            db(
                &format!("W{w}.RemainderCorrectionZero"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        let actual_r = signed(w, wc(w, "SignedRem", vec![x.clone(), y.clone()]));
        let old_r = Relation::new(
            old_field(w, "MathematicalRemainder", x.clone(), y.clone()),
            ic(
                "Add",
                vec![
                    actual_r.clone(),
                    ic(
                        "Mul",
                        vec![
                            ic("FromNat", vec![weight(d)]),
                            old_field(w, "RemainderCorrection", x.clone(), y.clone()),
                        ],
                    ),
                ],
            ),
            old_field(w, "RemainderRefinement", x.clone(), y.clone()),
        );
        let r = old_r.then(
            Relation::reflexive(actual_r.clone())
                .plus(rzero.times_left(ic("FromNat", vec![weight(d)]))),
        )?;
        let remove = Relation::new(
            r.right.clone(),
            actual_r.clone(),
            db("RemoveZeroCorrection", vec![actual_r, weight(d)]),
        );
        self.relation(
            &format!("W{w}.RemainderRefinementExact"),
            &args,
            r.then(remove)?,
        )
    }
}
