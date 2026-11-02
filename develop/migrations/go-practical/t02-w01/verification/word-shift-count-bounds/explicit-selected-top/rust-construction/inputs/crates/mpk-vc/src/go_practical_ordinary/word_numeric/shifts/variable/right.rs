//! Actual unsigned/signed variable right stages and their true low remainders.
use super::*;
fn fixed_rem(w: usize, k: usize, x: Expr) -> Expr {
    sh(&format!("W{w}.Remainder{k}"), vec![x])
}
fn prefix_power_zero(c: Expr) -> Result<Equality> {
    map_power(Equality::natural(digits(0, c), zero(), refl(nat(), zero()))).then(Equality::natural(
        power(zero()),
        one(),
        numeric("PowerTwoZero", vec![]),
    ))
}
fn zero_fill(u: Expr, m: Expr, e: Expr) -> Result<Equality> {
    mul_right(mul(m.clone(), e.clone()), bit_literal(false))
        .on_right(u.clone())
        .then(
            law(
                "MulZero",
                vec![mul(m.clone(), e.clone())],
                mul(mul(m, e), zero()),
                zero(),
            )
            .on_right(u.clone()),
        )?
        .then(law("AddZero", vec![u.clone()], add(u.clone(), zero()), u))
}
impl Shifts<'_> {
    pub(super) fn variable_right(&mut self, w: usize, sg: bool) -> Result<()> {
        let d = depth(w)?;
        let kind = right_kind(sg);
        let (x, c) = (var("x"), var("count"));
        let args = [("x", word_type(w)?), ("count", word_type(64)?)];
        let m = weight(d);
        let u = value(d, x.clone());
        let fill = if sg {
            at(x.clone(), w, w - 1)?
        } else {
            bit(false)
        };
        let bb = bit_value(fill.clone());
        self.emit(
            &format!("W{w}.{kind}StageRemainder0"),
            &args,
            nat(),
            zero(),
            false,
        )?;
        let word = right_stage(w, sg, 0, x.clone(), c.clone());
        let pm = map_minus(Equality::natural(
            digits(0, c.clone()),
            zero(),
            refl(nat(), zero()),
        ))
        .then(known("PowerMinusZero", vec![], minus(zero()), zero()))?;
        let pad = mul_left(
            mul_right(m.clone(), pm).then(law(
                "MulZero",
                vec![m.clone()],
                mul(m.clone(), zero()),
                zero(),
            ))?,
            bb.clone(),
        )
        .then(law(
            "ZeroMul",
            vec![bb.clone()],
            mul(zero(), bb.clone()),
            zero(),
        ))?
        .on_right(u.clone())
        .then(law(
            "AddZero",
            vec![u.clone()],
            add(u.clone(), zero()),
            u.clone(),
        ))?;
        let result = mul_left(prefix_power_zero(c.clone())?, value(d, word.clone()))
            .on_left(right_remainder(w, sg, 0, x.clone(), c.clone()))
            .then(
                mul_right(one(), cube_equation(d, word.clone(), x.clone()))
                    .on_left(right_remainder(w, sg, 0, x.clone(), c.clone())),
            )?
            .then(
                law("OneMul", vec![u.clone()], mul(one(), u.clone()), u.clone())
                    .on_left(right_remainder(w, sg, 0, x.clone(), c.clone())),
            )?
            .then(
                Equality::natural(
                    right_remainder(w, sg, 0, x.clone(), c.clone()),
                    zero(),
                    refl(nat(), zero()),
                )
                .on_right(u.clone()),
            )?
            .then(law(
                "AddZero",
                vec![u.clone()],
                add(u.clone(), zero()),
                u.clone(),
            ))?;
        self.theorem(
            &format!("W{w}.{kind}StageConservation0"),
            &args,
            pad.then(result.reverse())?,
        )?;
        let bound = less_left(
            Equality::natural(
                right_remainder(w, sg, 0, x.clone(), c.clone()),
                zero(),
                refl(nat(), zero()),
            ),
            power(digits(0, c.clone())),
        )
        .then(less_right(zero(), prefix_power_zero(c.clone())?))?
        .then(Equality::boolean(
            less(zero(), one()),
            bit(true),
            nc("LessZeroSucc", vec![zero()]),
        ))?;
        self.theorem(&format!("W{w}.{kind}StageBound0"), &args, bound)?;
        if sg {
            self.theorem(
                &format!("W{w}.SignedRightStageTop0"),
                &args,
                Equality::boolean(
                    at(word, w, w - 1)?,
                    fill.clone(),
                    refl(boolean(), fill.clone()),
                ),
            )?;
        }
        for i in 0..d {
            self.right_stage_step(w, sg, i, x.clone(), c.clone(), &args)?;
        }
        Ok(())
    }
    fn right_stage_step(
        &mut self,
        w: usize,
        sg: bool,
        i: usize,
        x: Expr,
        c: Expr,
        args: &[Binder],
    ) -> Result<()> {
        let fill = if sg {
            at(x.clone(), w, w - 1)?
        } else {
            bit(false)
        };
        let d = depth(w)?;
        let kind = right_kind(sg);
        let k = 1 << i;
        let u = value(d, x.clone());
        let m = weight(d);
        let bb = bit_value(fill.clone());
        let flag = at(c.clone(), 64, i)?;
        let prev = right_stage(w, sg, i, x.clone(), c.clone());
        let next = right_stage(w, sg, i + 1, x.clone(), c.clone());
        let fixed = wc(
            w,
            &format!("{}{k}", if sg { "SignedRight" } else { "Right" }),
            vec![prev.clone()],
        );
        let r = right_remainder(w, sg, i, x.clone(), c.clone());
        let s = fixed_rem(w, k, prev.clone());
        let p = power(digits(i, c.clone()));
        let q = coefficient(k);
        let rembody = select(
            flag.clone(),
            add(r.clone(), mul(p.clone(), s.clone())),
            r.clone(),
        );
        self.emit(
            &format!("W{w}.{kind}StageRemainder{}", i + 1),
            args,
            nat(),
            rembody.clone(),
            false,
        )?;
        let selected = wc(w, "Select", vec![flag.clone(), fixed.clone(), prev.clone()]);
        let output = cube_equation(d, next.clone(), selected.clone()).then(known(
            &format!("SelectValue{w}"),
            vec![flag.clone(), fixed.clone(), prev.clone()],
            value(d, selected),
            select(
                flag.clone(),
                value(d, fixed.clone()),
                value(d, prev.clone()),
            ),
        ))?;
        let a = value(d, prev.clone());
        let b = value(d, fixed.clone());
        let fe = minus(digits(i, c.clone()));
        let ff = minus(numeral(k));
        let fix_lhs = add(a.clone(), mul(mul(m.clone(), ff.clone()), bb.clone()));
        let fix_rhs = add(mul(q.clone(), b.clone()), s.clone());
        let fixed_law = if sg {
            let source_top = at(prev.clone(), w, w - 1)?;
            let top = Equality::boolean(
                source_top.clone(),
                fill.clone(),
                sh(
                    &format!("W{w}.SignedRightStageTop{i}"),
                    vec![x.clone(), c.clone()],
                ),
            );
            let change = mul_right(mul(m.clone(), ff.clone()), map_bit(top)).on_right(a.clone());
            change.reverse().then(known(
                &format!("W{w}.SignedRightConservation{k}"),
                vec![prev.clone()],
                add(
                    a.clone(),
                    mul(mul(m.clone(), ff.clone()), bit_value(source_top)),
                ),
                fix_rhs.clone(),
            ))?
        } else {
            zero_fill(a.clone(), m.clone(), ff.clone())?.then(known(
                &format!("W{w}.UnsignedRightConservation{k}"),
                vec![prev.clone()],
                a.clone(),
                fix_rhs.clone(),
            ))?
        };
        assert_eq!(fixed_law.left, fix_lhs);
        let step_pm = known(
            &format!("CountPrefixMinusStep{i}"),
            vec![c.clone()],
            minus(digits(i + 1, c.clone())),
            select(
                flag.clone(),
                add(fe.clone(), mul(p.clone(), ff.clone())),
                fe.clone(),
            ),
        );
        let prefix = mul_left(mul_right(m.clone(), step_pm), bb.clone()).on_right(u.clone());
        let target = add(
            mul(
                select(flag.clone(), mul(p.clone(), q.clone()), p.clone()),
                select(flag.clone(), b.clone(), a.clone()),
            ),
            rembody.clone(),
        );
        let prefix_right = prefix.right.clone();
        let proof = prefix.then(known(
            "RightConditionalCompose",
            vec![
                flag.clone(),
                u.clone(),
                a,
                b,
                m.clone(),
                p.clone(),
                q.clone(),
                fe,
                ff,
                bb,
                r.clone(),
                s.clone(),
                sh(
                    &format!("W{w}.{kind}StageConservation{i}"),
                    vec![x.clone(), c.clone()],
                ),
                fixed_law.proof,
            ],
            prefix_right,
            target,
        ))?;
        let powers = known(
            &format!("CountPrefixPowerStep{i}"),
            vec![c.clone()],
            power(digits(i + 1, c.clone())),
            select(flag.clone(), mul(p.clone(), q.clone()), p.clone()),
        );
        let proof = proof
            .then(
                mul_left(powers.clone().reverse(), output.right.clone()).on_left(rembody.clone()),
            )?
            .then(
                mul_right(power(digits(i + 1, c.clone())), output.reverse())
                    .on_left(rembody.clone()),
            )?
            .then(
                Equality::natural(
                    right_remainder(w, sg, i + 1, x.clone(), c.clone()),
                    rembody.clone(),
                    refl(nat(), rembody),
                )
                .reverse()
                .on_right(mul(power(digits(i + 1, c.clone())), value(d, next.clone()))),
            )?;
        self.theorem(
            &format!("W{w}.{kind}StageConservation{}", i + 1),
            args,
            proof,
        )?;
        let remdef = Equality::natural(
            right_remainder(w, sg, i + 1, x.clone(), c.clone()),
            select(
                flag.clone(),
                add(r.clone(), mul(p.clone(), s.clone())),
                r.clone(),
            ),
            refl(
                nat(),
                select(
                    flag.clone(),
                    add(r.clone(), mul(p.clone(), s.clone())),
                    r.clone(),
                ),
            ),
        );
        let bound = less_left(remdef, power(digits(i + 1, c.clone())))
            .then(less_right(
                select(
                    flag.clone(),
                    add(r.clone(), mul(p.clone(), s.clone())),
                    r.clone(),
                ),
                powers,
            ))?
            .then(Equality::boolean(
                less(
                    select(
                        flag.clone(),
                        add(r.clone(), mul(p.clone(), s.clone())),
                        r.clone(),
                    ),
                    select(flag.clone(), mul(p.clone(), q.clone()), p.clone()),
                ),
                bit(true),
                sh(
                    "RightConditionalRemainderBound",
                    vec![
                        flag.clone(),
                        r,
                        s,
                        p,
                        q,
                        sh(
                            &format!("W{w}.{kind}StageBound{i}"),
                            vec![x.clone(), c.clone()],
                        ),
                        sh(&format!("W{w}.RemainderBound{k}"), vec![prev.clone()]),
                    ],
                ),
            ))?;
        self.theorem(&format!("W{w}.{kind}StageBound{}", i + 1), args, bound)?;
        if sg {
            let old_top = at(prev.clone(), w, w - 1)?;
            let fixed_top = at(fixed.clone(), w, w - 1)?;
            let selected_top = choose(flag.clone(), fixed_top.clone(), old_top.clone(), boolean());
            let selected_word = wc(w, "Select", vec![flag.clone(), fixed.clone(), prev.clone()]);
            let selected_point = at(selected_word, w, w - 1)?;
            let raw = Equality::boolean(
                at(next, w, w - 1)?,
                selected_point.clone(),
                refl(boolean(), selected_point.clone()),
            )
            .then(Equality::boolean(
                selected_point,
                selected_top,
                sh(
                    &format!("SelectTop{w}"),
                    vec![flag.clone(), fixed, prev.clone()],
                ),
            ))?;
            let top_fixed = Equality::boolean(
                fixed_top,
                old_top.clone(),
                sh(&format!("W{w}.SignedRightTop{k}"), vec![prev]),
            );
            let top = raw
                .then(boolean_choice_left(
                    flag.clone(),
                    top_fixed,
                    old_top.clone(),
                ))?
                .then(Equality::boolean(
                    choose(flag.clone(), old_top.clone(), old_top.clone(), boolean()),
                    old_top.clone(),
                    sh("BoolChoiceSame", vec![flag, old_top.clone()]),
                ))?
                .then(Equality::boolean(
                    old_top,
                    fill,
                    sh(&format!("W{w}.SignedRightStageTop{i}"), vec![x, c]),
                ))?;
            self.theorem(&format!("W{w}.SignedRightStageTop{}", i + 1), args, top)?;
        }
        Ok(())
    }
}
