//! Full64 actual variable-left stages and actual large-count zero saturation.
use super::*;
impl Shifts<'_> {
    pub(super) fn variable_left(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, c) = (var("x"), var("count"));
        let args = [("x", word_type(w)?), ("count", word_type(64)?)];
        let u = value(d, x.clone());
        let m = weight(d);
        self.emit(
            &format!("W{w}.LeftStageDiscard0"),
            &args,
            nat(),
            zero(),
            false,
        )?;
        let s = stage(w, 0, x.clone(), c.clone());
        let first = cube_equation(d, s, x.clone())
            .on_left(mul(m.clone(), discarded(w, 0, x.clone(), c.clone())))
            .then(
                mul_right(
                    m.clone(),
                    Equality::natural(
                        discarded(w, 0, x.clone(), c.clone()),
                        zero(),
                        refl(nat(), zero()),
                    ),
                )
                .on_right(u.clone()),
            )?
            .then(
                law("MulZero", vec![m.clone()], mul(m.clone(), zero()), zero()).on_right(u.clone()),
            )?
            .then(law(
                "AddZero",
                vec![u.clone()],
                add(u.clone(), zero()),
                u.clone(),
            ))?;
        let p = map_power(Equality::natural(
            digits(0, c.clone()),
            zero(),
            refl(nat(), zero()),
        ))
        .then(Equality::natural(
            power(zero()),
            one(),
            numeric("PowerTwoZero", vec![]),
        ))?;
        let right = mul_left(p, u.clone()).then(law(
            "OneMul",
            vec![u.clone()],
            mul(one(), u.clone()),
            u.clone(),
        ))?;
        self.theorem(
            &format!("W{w}.LeftStageConservation0"),
            &args,
            first.then(right.reverse())?,
        )?;
        for i in 0..d {
            let k = 1 << i;
            let flag = at(c.clone(), 64, i)?;
            let previous = stage(w, i, x.clone(), c.clone());
            let shifted = wc(w, &format!("Left{k}"), vec![previous.clone()]);
            let result = stage(w, i + 1, x.clone(), c.clone());
            let old_discard = discarded(w, i, x.clone(), c.clone());
            let step_discard = numeric(&format!("W{w}.ShiftDiscard{k}"), vec![previous.clone()]);
            let body = select(
                flag.clone(),
                add(
                    mul(coefficient(k), old_discard.clone()),
                    step_discard.clone(),
                ),
                old_discard.clone(),
            );
            self.emit(
                &format!("W{w}.LeftStageDiscard{}", i + 1),
                &args,
                nat(),
                body.clone(),
                false,
            )?;
            let selected = wc(
                w,
                "Select",
                vec![flag.clone(), shifted.clone(), previous.clone()],
            );
            let output = cube_equation(d, result, selected.clone()).then(known(
                &format!("SelectValue{w}"),
                vec![flag.clone(), shifted.clone(), previous.clone()],
                value(d, selected),
                select(
                    flag.clone(),
                    value(d, shifted.clone()),
                    value(d, previous.clone()),
                ),
            ))?;
            let output_right = output.right.clone();
            let left = output
                .on_left(mul(m.clone(), discarded(w, i + 1, x.clone(), c.clone())))
                .then(
                    mul_right(
                        m.clone(),
                        Equality::natural(
                            discarded(w, i + 1, x.clone(), c.clone()),
                            body.clone(),
                            refl(nat(), body),
                        ),
                    )
                    .on_right(output_right),
                )?;
            let a = value(d, previous.clone());
            let b = value(d, shifted);
            let p = power(digits(i, c.clone()));
            let q = coefficient(k);
            let before = sh(
                &format!("W{w}.LeftStageConservation{i}"),
                vec![x.clone(), c.clone()],
            );
            let after = numeric(&format!("W{w}.ShiftConservation{k}"), vec![previous]);
            let target = mul(
                select(flag.clone(), mul(p.clone(), q.clone()), p.clone()),
                u.clone(),
            );
            let e = left.clone().then(known(
                "LeftConditionalCompose",
                vec![
                    flag,
                    u.clone(),
                    a,
                    b,
                    m.clone(),
                    p,
                    q,
                    old_discard,
                    step_discard,
                    before,
                    after,
                ],
                left.right,
                target,
            ))?;
            let power_step = known(
                &format!("CountPrefixPowerStep{i}"),
                vec![c.clone()],
                power(digits(i + 1, c.clone())),
                select(
                    at(c.clone(), 64, i)?,
                    mul(power(digits(i, c.clone())), coefficient(k)),
                    power(digits(i, c.clone())),
                ),
            );
            self.theorem(
                &format!("W{w}.LeftStageConservation{}", i + 1),
                &args,
                e.then(mul_left(power_step, u.clone()).reverse())?,
            )?;
        }
        self.left_saturation(w, x, c, &args)
    }
    fn left_saturation(&mut self, w: usize, x: Expr, c: Expr, args: &[Binder]) -> Result<()> {
        let d = depth(w)?;
        let h = high(d, c.clone());
        let e = sh(&format!("CountExcess{d}"), vec![c.clone()]);
        let u = value(d, x.clone());
        let m = weight(d);
        let last = stage(w, d, x.clone(), c.clone());
        let discard = discarded(w, d, x.clone(), c.clone());
        let body = select(h.clone(), mul(power(e.clone()), u.clone()), discard.clone());
        self.emit(
            &format!("W{w}.LeftDiscard"),
            args,
            nat(),
            body.clone(),
            false,
        )?;
        let actual = wc(w, "ShiftLeft", vec![x.clone(), c.clone()]);
        let zero_word = wc(w, "Zero", vec![]);
        let raw = counts::tree(d, d, 64, c.clone(), false)?;
        let raw_selected = wc(
            w,
            "Select",
            vec![raw.clone(), zero_word.clone(), last.clone()],
        );
        let output = cube_equation(d, actual.clone(), raw_selected.clone());
        let named_selected = wc(
            w,
            "Select",
            vec![h.clone(), zero_word.clone(), last.clone()],
        );
        let cond = Equality::boolean(raw.clone(), h.clone(), refl(boolean(), raw)).mapped(
            lam(
                "_source_high",
                boolean(),
                value(
                    d,
                    wc(
                        w,
                        "Select",
                        vec![var("_source_high"), zero_word.clone(), last.clone()],
                    ),
                ),
            ),
            value(d, raw_selected),
            value(d, named_selected.clone()),
            nat(),
        );
        let output = output.then(cond)?.then(known(
            &format!("SelectValue{w}"),
            vec![h.clone(), zero_word.clone(), last.clone()],
            value(d, named_selected),
            select(
                h.clone(),
                value(d, zero_word.clone()),
                value(d, last.clone()),
            ),
        ))?;
        let output = output.then(choice_left(
            h.clone(),
            Equality::natural(
                value(d, zero_word),
                zero(),
                numeric(&format!("W{w}.ZeroValue"), vec![]),
            ),
            value(d, last.clone()),
        ))?;
        self.theorem(&format!("W{w}.LeftSelectedValue"), args, output.clone())?;
        let goal_left = add(
            value(d, actual),
            mul(m.clone(), full_discard(w, x.clone(), c.clone())),
        );
        let goal_right = mul(power(value(6, c.clone())), u.clone());
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_saturation_branch",
                eq(boolean(), h.clone(), env["_saturation_flag"].clone()),
                eq(nat(), goal_left.clone(), goal_right.clone()),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = env["_saturation_flag"] == bit(true);
            let guard = Equality::boolean(h.clone(), bit(yes), var("_saturation_branch"));
            let v = output
                .clone()
                .then(choice_condition(
                    guard.clone(),
                    zero(),
                    value(d, last.clone()),
                ))?
                .then(literal(yes, zero(), value(d, last.clone())))?;
            let dv = Equality::natural(
                full_discard(w, x.clone(), c.clone()),
                body.clone(),
                refl(nat(), body.clone()),
            )
            .then(choice_condition(
                guard,
                mul(power(e.clone()), u.clone()),
                discard.clone(),
            ))?
            .then(literal(
                yes,
                mul(power(e.clone()), u.clone()),
                discard.clone(),
            ))?;
            let end = v.right.clone();
            let proof = v
                .on_left(mul(m.clone(), dv.left.clone()))
                .then(mul_right(m.clone(), dv).on_right(end))?;
            let finish = if yes {
                let proof = proof
                    .then(law(
                        "ZeroAdd",
                        vec![mul(m.clone(), mul(power(e.clone()), u.clone()))],
                        add(zero(), mul(m.clone(), mul(power(e.clone()), u.clone()))),
                        mul(m.clone(), mul(power(e.clone()), u.clone())),
                    ))?
                    .then(
                        law(
                            "MulAssoc",
                            vec![m.clone(), power(e.clone()), u.clone()],
                            mul(mul(m.clone(), power(e.clone())), u.clone()),
                            mul(m.clone(), mul(power(e.clone()), u.clone())),
                        )
                        .reverse(),
                    )?;
                proof.then(
                    mul_left(
                        known(
                            &format!("CountLargePower{d}"),
                            vec![c.clone(), var("_saturation_branch")],
                            power(value(6, c.clone())),
                            mul(m.clone(), power(e.clone())),
                        ),
                        u.clone(),
                    )
                    .reverse(),
                )?
            } else {
                proof
                    .then(known(
                        &format!("W{w}.LeftStageConservation{d}"),
                        vec![x.clone(), c.clone()],
                        add(value(d, last.clone()), mul(m.clone(), discard.clone())),
                        mul(power(digits(d, c.clone())), u.clone()),
                    ))?
                    .then(
                        mul_left(
                            known(
                                &format!("CountSmallPower{d}"),
                                vec![c.clone(), var("_saturation_branch")],
                                power(value(6, c.clone())),
                                power(digits(d, c.clone())),
                            ),
                            u.clone(),
                        )
                        .reverse(),
                    )?
            };
            Ok(lam(
                "_saturation_branch",
                eq(boolean(), h.clone(), bit(yes)),
                finish.proof,
            ))
        };
        let cases = boolean_cases(&["_saturation_flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.theorem(
            &format!("W{w}.UnsignedLeftConservation"),
            args,
            Equality::natural(
                goal_left,
                goal_right,
                app(cases, vec![h.clone(), refl(boolean(), h)]),
            ),
        )
    }
}
