//! Genuine large-count algebra and source zero/sign-fill saturation.
use super::*;
mod scalar;
fn cv(n: &str, a: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Conversions.{n}"), a)
}
fn ones(d: usize) -> Expr {
    cv(&format!("Ones{d}"), vec![])
}
fn full_rem(w: usize, sg: bool, x: Expr, c: Expr) -> Expr {
    sh(&format!("W{w}.{}Remainder", right_kind(sg)), vec![x, c])
}
impl Shifts<'_> {
    pub(super) fn right_saturation_laws(&mut self) -> Result<()> {
        self.right_saturation_scalar()?;
        self.right_saturation_bound()
    }
    pub(super) fn bit_select_value(&mut self) -> Result<()> {
        let (m, b) = (var("value"), var("bit"));
        let args = [("value", nat()), ("bit", boolean())];
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                select(e["_value_flag"].clone(), m.clone(), zero()),
                mul(m.clone(), bit_value(e["_value_flag"].clone())),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_value_flag"] == bit(true);
            let rhs = mul_right(m.clone(), bit_literal(yes)).then(law(
                if yes { "MulOne" } else { "MulZero" },
                vec![m.clone()],
                mul(m.clone(), if yes { one() } else { zero() }),
                if yes { m.clone() } else { zero() },
            ))?;
            Ok(literal(yes, m.clone(), zero()).then(rhs.reverse())?.proof)
        };
        let proof = app(
            boolean_cases(&["_value_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![b.clone()],
        );
        self.theorem(
            "BitSelectValue",
            &args,
            Equality::natural(
                select(b.clone(), m.clone(), zero()),
                mul(m, bit_value(b)),
                proof,
            ),
        )
    }
    pub(super) fn right_fill(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let b = var("bit");
        let args = [("bit", boolean())];
        let word_ones = wc(w, "Ones", vec![]);
        let word_zero = wc(w, "Zero", vec![]);
        let filled = cv(&format!("Fill{d}"), vec![bit(true)]);
        let eq_ones = self
            .literal_values(d, word_ones.clone(), filled.clone())
            .then(Equality::natural(
                value(d, filled),
                mul(ones(d), bit_value(bit(true))),
                cv(&format!("FillValue{d}"), vec![bit(true)]),
            ))?
            .then(mul_right(ones(d), bit_literal(true)))?
            .then(law("MulOne", vec![ones(d)], mul(ones(d), one()), ones(d)))?;
        self.theorem(&format!("W{w}.OnesValue"), &[], eq_ones.clone())?;
        let filled = wc(
            w,
            "Select",
            vec![b.clone(), word_ones.clone(), word_zero.clone()],
        );
        let value_eq = known(
            &format!("SelectValue{w}"),
            vec![b.clone(), word_ones.clone(), word_zero.clone()],
            value(d, filled.clone()),
            select(
                b.clone(),
                value(d, word_ones.clone()),
                value(d, word_zero.clone()),
            ),
        );
        let value_eq = value_eq
            .then(choice_left(b.clone(), eq_ones, value(d, word_zero.clone())))?
            .then(choice_right(
                b.clone(),
                ones(d),
                Equality::natural(
                    value(d, word_zero),
                    zero(),
                    numeric(&format!("W{w}.ZeroValue"), vec![]),
                ),
            ))?
            .then(known(
                "BitSelectValue",
                vec![ones(d), b.clone()],
                select(b.clone(), ones(d), zero()),
                mul(ones(d), bit_value(b.clone())),
            ))?;
        self.theorem(&format!("W{w}.SignFillValue"), &args, value_eq)?;
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                at(
                    wc(
                        w,
                        "Select",
                        vec![
                            e["_fill_bit"].clone(),
                            word_ones.clone(),
                            wc(w, "Zero", vec![]),
                        ],
                    ),
                    w,
                    w - 1,
                )
                .expect("fixed source width"),
                e["_fill_bit"].clone(),
            )
        };
        let mut leaf =
            |e: &BTreeMap<&'static str, Expr>| Ok(refl(boolean(), e["_fill_bit"].clone()));
        let proof = app(
            boolean_cases(&["_fill_bit"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![b.clone()],
        );
        self.theorem(
            &format!("W{w}.SignFillTop"),
            &args,
            Equality::boolean(at(filled, w, w - 1)?, b, proof),
        )
    }
    pub(super) fn full_right(&mut self, w: usize, sg: bool) -> Result<()> {
        let d = depth(w)?;
        let kind = right_kind(sg);
        let (x, c) = (var("x"), var("count"));
        let args = [("x", word_type(w)?), ("count", word_type(64)?)];
        let b = if sg {
            at(x.clone(), w, w - 1)?
        } else {
            bit(false)
        };
        let bb = bit_value(b.clone());
        let h = high(d, c.clone());
        let tail = sh(&format!("CountExcess{d}"), vec![c.clone()]);
        let m = weight(d);
        let u = value(d, x.clone());
        let p = power(value(6, c.clone()));
        let pm = minus(value(6, c.clone()));
        let ex = minus(tail.clone());
        let large = add(u.clone(), mul(mul(m.clone(), ex.clone()), bb.clone()));
        let old_rem = right_remainder(w, sg, d, x.clone(), c.clone());
        let rem_body = select(h.clone(), large.clone(), old_rem.clone());
        self.emit(
            &format!("W{w}.{kind}Remainder"),
            &args,
            nat(),
            rem_body.clone(),
            false,
        )?;
        let last = right_stage(w, sg, d, x.clone(), c.clone());
        let fill = if sg {
            wc(
                w,
                "Select",
                vec![b.clone(), wc(w, "Ones", vec![]), wc(w, "Zero", vec![])],
            )
        } else {
            wc(w, "Zero", vec![])
        };
        let fill_eq = if sg {
            known(
                &format!("W{w}.SignFillValue"),
                vec![b.clone()],
                value(d, fill.clone()),
                mul(ones(d), bb.clone()),
            )
        } else {
            Equality::natural(
                value(d, fill.clone()),
                zero(),
                numeric(&format!("W{w}.ZeroValue"), vec![]),
            )
            .then(
                mul_right(ones(d), bit_literal(false))
                    .then(law("MulZero", vec![ones(d)], mul(ones(d), zero()), zero()))?
                    .reverse(),
            )?
        };
        let source = wc(
            w,
            if sg { "SignedShiftRight" } else { "ShiftRight" },
            vec![x.clone(), c.clone()],
        );
        let raw = counts::tree(d, d, 64, c.clone(), false)?;
        let raw_selected = wc(w, "Select", vec![raw.clone(), fill.clone(), last.clone()]);
        let named_selected = wc(w, "Select", vec![h.clone(), fill.clone(), last.clone()]);
        let raw_eq = Equality::boolean(raw.clone(), h.clone(), refl(boolean(), raw.clone()))
            .mapped(
                lam(
                    "_actual_high",
                    boolean(),
                    value(
                        d,
                        wc(
                            w,
                            "Select",
                            vec![var("_actual_high"), fill.clone(), last.clone()],
                        ),
                    ),
                ),
                value(d, raw_selected.clone()),
                value(d, named_selected.clone()),
                nat(),
            );
        let output = cube_equation(d, source.clone(), raw_selected)
            .then(raw_eq)?
            .then(known(
                &format!("SelectValue{w}"),
                vec![h.clone(), fill.clone(), last.clone()],
                value(d, named_selected),
                select(h.clone(), value(d, fill.clone()), value(d, last.clone())),
            ))?
            .then(choice_left(h.clone(), fill_eq, value(d, last.clone())))?;
        self.theorem(&format!("W{w}.{kind}SelectedValue"), &args, output.clone())?;
        let lhs = add(u.clone(), mul(mul(m.clone(), pm.clone()), bb.clone()));
        let rhs = add(
            mul(p.clone(), value(d, source.clone())),
            full_rem(w, sg, x.clone(), c.clone()),
        );
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            pi(
                "_actual_branch",
                eq(boolean(), h.clone(), e["_saturation_flag"].clone()),
                eq(nat(), lhs.clone(), rhs.clone()),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_saturation_flag"] == bit(true);
            let guard = Equality::boolean(h.clone(), bit(yes), var("_actual_branch"));
            let ov = output
                .clone()
                .then(choice_condition(
                    guard.clone(),
                    mul(ones(d), bb.clone()),
                    value(d, last.clone()),
                ))?
                .then(literal(
                    yes,
                    mul(ones(d), bb.clone()),
                    value(d, last.clone()),
                ))?;
            let rv = Equality::natural(
                full_rem(w, sg, x.clone(), c.clone()),
                rem_body.clone(),
                refl(nat(), rem_body.clone()),
            )
            .then(choice_condition(guard, large.clone(), old_rem.clone()))?
            .then(literal(yes, large.clone(), old_rem.clone()))?;
            let right = mul_right(p.clone(), ov).on_left(rv.left.clone());
            let product = if yes {
                mul(p.clone(), mul(ones(d), bb.clone()))
            } else {
                mul(p.clone(), value(d, last.clone()))
            };
            let right = right.then(rv.on_right(product))?;
            let left = if yes {
                let before = sh("PowerMinusBalance", vec![value(6, c.clone())]);
                let after = sh("PowerMinusBalance", vec![tail.clone()]);
                known(
                    "RightSaturatedConservation",
                    vec![
                        u.clone(),
                        m.clone(),
                        ones(d),
                        p.clone(),
                        pm.clone(),
                        power(tail.clone()),
                        ex.clone(),
                        bb.clone(),
                        cv(&format!("OnesBalance{d}"), vec![]),
                        before,
                        after,
                        sh(
                            &format!("CountLargePower{d}"),
                            vec![c.clone(), var("_actual_branch")],
                        ),
                    ],
                    lhs.clone(),
                    right.right.clone(),
                )
            } else {
                let small = known(
                    &format!("CountSmallValue{d}"),
                    vec![c.clone(), var("_actual_branch")],
                    value(6, c.clone()),
                    digits(d, c.clone()),
                );
                let left = mul_left(mul_right(m.clone(), map_minus(small)), bb.clone())
                    .on_right(u.clone());
                let target = add(
                    mul(power(digits(d, c.clone())), value(d, last.clone())),
                    old_rem.clone(),
                );
                let lright = left.right.clone();
                let left = left.then(known(
                    &format!("W{w}.{kind}StageConservation{d}"),
                    vec![x.clone(), c.clone()],
                    lright,
                    target,
                ))?;
                let pow = known(
                    &format!("CountSmallPower{d}"),
                    vec![c.clone(), var("_actual_branch")],
                    p.clone(),
                    power(digits(d, c.clone())),
                );
                left.then(mul_left(pow.reverse(), value(d, last.clone())).on_left(old_rem.clone()))?
            };
            Ok(lam(
                "_actual_branch",
                eq(boolean(), h.clone(), bit(yes)),
                left.then(right.reverse())?.proof,
            ))
        };
        let proof = app(
            boolean_cases(&["_saturation_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![h.clone(), refl(boolean(), h.clone())],
        );
        self.theorem(
            &format!("W{w}.{kind}FilledConservation"),
            &args,
            Equality::natural(lhs.clone(), rhs.clone(), proof),
        )?;
        if !sg {
            let e = super::right::zero_fill(u.clone(), m.clone(), pm.clone())?
                .reverse()
                .then(known(
                    &format!("W{w}.{kind}FilledConservation"),
                    vec![x.clone(), c.clone()],
                    lhs,
                    rhs,
                ))?;
            self.theorem(&format!("W{w}.UnsignedRightConservation"), &args, e)?;
        }
        self.full_right_bound(w, sg, x.clone(), c.clone(), b, &args)?;
        if sg {
            self.full_right_top(w, x, c, (raw, fill, last, source), &args)?;
        }
        Ok(())
    }
    fn full_right_bound(
        &mut self,
        w: usize,
        sg: bool,
        x: Expr,
        c: Expr,
        b: Expr,
        args: &[Binder],
    ) -> Result<()> {
        let d = depth(w)?;
        let kind = right_kind(sg);
        let h = high(d, c.clone());
        let ex = sh(&format!("CountExcess{d}"), vec![c.clone()]);
        let p = power(value(6, c.clone()));
        let m = weight(d);
        let u = value(d, x.clone());
        let bb = bit_value(b.clone());
        let large = add(
            u.clone(),
            mul(mul(m.clone(), minus(ex.clone())), bb.clone()),
        );
        let small = right_remainder(w, sg, d, x.clone(), c.clone());
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_actual_branch",
                eq(boolean(), h.clone(), env["_bound_high"].clone()),
                eq(
                    boolean(),
                    less(full_rem(w, sg, x.clone(), c.clone()), p.clone()),
                    bit(true),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = env["_bound_high"] == bit(true);
            let guard = Equality::boolean(h.clone(), bit(yes), var("_actual_branch"));
            let body = select(h.clone(), large.clone(), small.clone());
            let e = Equality::natural(
                full_rem(w, sg, x.clone(), c.clone()),
                body.clone(),
                refl(nat(), body),
            )
            .then(choice_condition(guard, large.clone(), small.clone()))?
            .then(literal(yes, large.clone(), small.clone()))?;
            let start = less_left(e, p.clone());
            let finish = if yes {
                let ncube = call(
                    &format!("Mpk.Go.Practical.Word.Order.NotCube{d}"),
                    vec![x.clone()],
                );
                let complement = value(d, ncube);
                Equality::boolean(
                    less(large.clone(), p.clone()),
                    bit(true),
                    sh(
                        "RightSaturatedRemainderBound",
                        vec![
                            u.clone(),
                            complement,
                            m.clone(),
                            minus(ex.clone()),
                            bb.clone(),
                            bit_value(bc("Not", vec![b.clone()])),
                            power(ex.clone()),
                            p.clone(),
                            call(
                                &format!("Mpk.Go.Practical.Word.Order.Complement{d}"),
                                vec![x.clone()],
                            ),
                            sh("CountBitComplement", vec![b.clone()]),
                            sh("PowerMinusBalance", vec![ex.clone()]),
                            sh(
                                &format!("CountLargePower{d}"),
                                vec![c.clone(), var("_actual_branch")],
                            ),
                        ],
                    ),
                )
            } else {
                less_right(
                    small.clone(),
                    known(
                        &format!("CountSmallPower{d}"),
                        vec![c.clone(), var("_actual_branch")],
                        p.clone(),
                        power(digits(d, c.clone())),
                    ),
                )
                .then(Equality::boolean(
                    less(small.clone(), power(digits(d, c.clone()))),
                    bit(true),
                    sh(
                        &format!("W{w}.{kind}StageBound{d}"),
                        vec![x.clone(), c.clone()],
                    ),
                ))?
            };
            Ok(lam(
                "_actual_branch",
                eq(boolean(), h.clone(), bit(yes)),
                start.then(finish)?.proof,
            ))
        };
        let proof = app(
            boolean_cases(&["_bound_high"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![h.clone(), refl(boolean(), h)],
        );
        self.theorem(
            &format!("W{w}.{kind}RemainderBound"),
            args,
            Equality::boolean(less(full_rem(w, sg, x, c), p), bit(true), proof),
        )
    }
    fn full_right_top(
        &mut self,
        w: usize,
        x: Expr,
        c: Expr,
        words: (Expr, Expr, Expr, Expr),
        args: &[Binder],
    ) -> Result<()> {
        let (raw, fill, last, source) = words;
        let top = at(x.clone(), w, w - 1)?;
        let selected = wc(w, "Select", vec![raw.clone(), fill.clone(), last.clone()]);
        let point = at(selected, w, w - 1)?;
        let fill_top = at(fill.clone(), w, w - 1)?;
        let stage_top = at(last.clone(), w, w - 1)?;
        let e = Equality::boolean(
            at(source, w, w - 1)?,
            point.clone(),
            refl(boolean(), point.clone()),
        )
        .then(Equality::boolean(
            point,
            choose(raw.clone(), fill_top.clone(), stage_top.clone(), boolean()),
            sh(&format!("SelectTop{w}"), vec![raw.clone(), fill, last]),
        ))?;
        let e = e
            .then(boolean_choice_left(
                raw.clone(),
                Equality::boolean(
                    fill_top,
                    top.clone(),
                    sh(&format!("W{w}.SignFillTop"), vec![top.clone()]),
                ),
                stage_top.clone(),
            ))?
            .then(boolean_choice_right(
                raw.clone(),
                top.clone(),
                Equality::boolean(
                    stage_top,
                    top.clone(),
                    sh(
                        &format!("W{w}.SignedRightStageTop{}", depth(w)?),
                        vec![x, c],
                    ),
                ),
            ))?
            .then(Equality::boolean(
                choose(raw.clone(), top.clone(), top.clone(), boolean()),
                top.clone(),
                sh("BoolChoiceSame", vec![raw, top]),
            ))?;
        self.theorem(&format!("W{w}.SignedRightTop"), args, e)
    }
}
