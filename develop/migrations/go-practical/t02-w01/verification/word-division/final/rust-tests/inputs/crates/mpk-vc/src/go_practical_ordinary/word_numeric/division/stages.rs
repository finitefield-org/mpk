//! Every actual division stage, from a real nonzero divisor to Euclidean laws.
use super::super::super::word::vector;
use super::horner::prefix;
use super::word_step::{decision, next};
use super::*;

fn remainder(w: usize, i: usize, x: Expr, y: Expr) -> Expr {
    wc(w, &format!("Remainder{i}"), vec![x, y])
}
fn quotient(w: usize, i: usize, x: Expr, y: Expr) -> Expr {
    wc(w, &format!("QuotientBit{i}"), vec![x, y])
}
fn raw(w: usize, x: Expr, y: Expr) -> Expr {
    dc(&format!("W{w}.QuotientWord"), vec![x, y])
}
fn guard(w: usize, y: Expr) -> Expr {
    eq(boolean(), wc(w, "DivideDefined", vec![y]), bit(true))
}

impl Division<'_> {
    pub(super) fn stage_combine(&mut self) -> Result<()> {
        let (r, d, q, b, rn, qn, p) = (
            var("r"),
            var("d"),
            var("q"),
            var("low"),
            var("next"),
            var("digit"),
            var("prefix"),
        );
        let mut args = ["r", "d", "q", "low", "next", "digit", "prefix"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect::<Vec<Binder>>();
        args.extend([
            (
                "prior",
                eq(nat(), add(r.clone(), mul(d.clone(), q.clone())), p.clone()),
            ),
            (
                "step",
                eq(
                    nat(),
                    add(rn.clone(), mul(d.clone(), qn.clone())),
                    add(mul(two(), r.clone()), b.clone()),
                ),
            ),
        ]);
        let e = self.polynomial(
            &args,
            add(rn, mul(d.clone(), add(mul(two(), q.clone()), qn.clone()))),
            add(
                add(var("next"), mul(d.clone(), qn.clone())),
                mul(two(), mul(d.clone(), q.clone())),
            ),
        )?;
        let e = e.then(
            Equality::natural(
                add(var("next"), mul(d.clone(), qn)),
                add(mul(two(), r.clone()), b.clone()),
                var("step"),
            )
            .on_left(mul(two(), mul(d.clone(), q.clone()))),
        )?;
        let next = self.polynomial(
            &args,
            e.right.clone(),
            add(
                mul(two(), add(r.clone(), mul(d.clone(), q.clone()))),
                b.clone(),
            ),
        )?;
        let e = e.then(next)?.then(
            mul_right(two(), Equality::natural(add(r, mul(d, q)), p, var("prior"))).on_left(b),
        )?;
        self.theorem("StageCombine", &args, e)
    }
    pub(super) fn stages(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let plain = [("x", word_type(w)?), ("y", word_type(w)?)];
        let mut args = plain.to_vec();
        args.push(("defined", guard(w, y.clone())));
        let bits = (0..w)
            .map(|i| quotient(w, w - 1 - i, x.clone(), y.clone()))
            .collect();
        self.emit(
            &format!("W{w}.QuotientWord"),
            &plain,
            word_type(w)?,
            vector(w, bits)?,
            false,
        )?;
        let qword = raw(w, x.clone(), y.clone());
        let modulus = value(d, y.clone());
        let r0 = remainder(w, 0, x.clone(), y.clone());
        let zero_word = wc(w, "Zero", vec![]);
        let zero_value = map_value(
            d,
            Equality {
                ty: word_type(w)?,
                left: r0.clone(),
                right: zero_word.clone(),
                proof: refl(word_type(w)?, zero_word.clone()),
            },
        )
        .then(Equality::natural(
            value(d, zero_word),
            zero(),
            order(&format!("W{w}.ZeroValue"), vec![]),
        ))?;
        self.theorem(
            &format!("W{w}.RemainderZeroValue"),
            &plain,
            zero_value.clone(),
        )?;
        let initial_bound =
            less_left(zero_value.clone(), modulus.clone()).then(Equality::boolean(
                less(zero(), modulus.clone()),
                bit(true),
                order(
                    &format!("W{w}.DefinedDivisorPositive"),
                    vec![y.clone(), var("defined")],
                ),
            ))?;
        self.theorem(&format!("W{w}.RemainderBound0"), &args, initial_bound)?;
        let e = zero_value
            .on_left(mul(modulus.clone(), prefix(w, 0, qword.clone())))
            .then(
                mul_right(
                    modulus.clone(),
                    Equality::natural(prefix(w, 0, qword.clone()), zero(), refl(nat(), zero())),
                )
                .on_right(zero()),
            )?
            .then(
                Equality::natural(
                    mul(modulus.clone(), zero()),
                    zero(),
                    nc("MulZero", vec![modulus.clone()]),
                )
                .on_right(zero()),
            )?
            .then(Equality::natural(
                add(zero(), zero()),
                zero(),
                nc("ZeroAdd", vec![zero()]),
            ))?
            .then(
                Equality::natural(prefix(w, 0, x.clone()), zero(), refl(nat(), zero())).reverse(),
            )?;
        self.theorem(&format!("W{w}.StageConservation0"), &args, e)?;
        for i in 0..w {
            self.stage(w, i)?;
        }
        self.finish_division(w)
    }
    fn stage(&mut self, w: usize, i: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let plain = [("x", word_type(w)?), ("y", word_type(w)?)];
        let mut args = plain.to_vec();
        args.push(("defined", guard(w, y.clone())));
        let r = remainder(w, i, x.clone(), y.clone());
        let rn = remainder(w, i + 1, x.clone(), y.clone());
        let b = at(x.clone(), w, w - 1 - i)?;
        let q = quotient(w, i, x.clone(), y.clone());
        let chosen = decision(w, r.clone(), y.clone(), b.clone());
        let next = next(w, r.clone(), y.clone(), b.clone());
        let shifted = wc(w, &format!("DividendStep{i}"), vec![x.clone(), y.clone()]);
        let pushed = wc(w, "PushLow", vec![r.clone(), b.clone()]);
        let shift_fold = Equality {
            ty: word_type(w)?,
            left: shifted.clone(),
            right: pushed.clone(),
            proof: refl(word_type(w)?, pushed.clone()),
        };
        self.theorem(
            &format!("W{w}.StageDividendFold{i}"),
            &plain,
            shift_fold.clone(),
        )?;
        let shift_fold = Equality {
            proof: dc(
                &format!("W{w}.StageDividendFold{i}"),
                vec![x.clone(), y.clone()],
            ),
            ..shift_fold
        };
        let top = at(r.clone(), w, w - 1)?;
        let q_at = |s: Expr| {
            bc(
                "Or",
                vec![
                    top.clone(),
                    bc("Not", vec![wc(w, "UnsignedLess", vec![s, y.clone()])]),
                ],
            )
        };
        let q_source = q_at(shifted.clone());
        let q_shifted = q_at(pushed.clone());
        let qfold = Equality::boolean(
            q.clone(),
            q_source.clone(),
            refl(boolean(), q_source.clone()),
        )
        .then(shift_fold.clone().mapped(
            lam("_shifted", word_type(w)?, q_at(var("_shifted"))),
            q_source,
            q_shifted.clone(),
            boolean(),
        ))?
        .then(
            Equality::boolean(
                chosen.clone(),
                q_shifted.clone(),
                refl(boolean(), q_shifted),
            )
            .reverse(),
        )?;
        self.theorem(&format!("W{w}.StageDecisionFold{i}"), &plain, qfold.clone())?;
        let qfold = Equality {
            proof: dc(
                &format!("W{w}.StageDecisionFold{i}"),
                vec![x.clone(), y.clone()],
            ),
            ..qfold
        };
        let select_at = |flag: Expr, s: Expr| {
            wc(
                w,
                "Select",
                vec![flag, wc(w, "Sub", vec![s.clone(), y.clone()]), s],
            )
        };
        let source = select_at(q.clone(), shifted.clone());
        let pushed_select = select_at(q.clone(), pushed.clone());
        let chosen_select = select_at(chosen.clone(), pushed.clone());
        let fold = Equality {
            ty: word_type(w)?,
            left: rn.clone(),
            right: source.clone(),
            proof: refl(word_type(w)?, source.clone()),
        }
        .then(shift_fold.mapped(
            lam(
                "_shifted",
                word_type(w)?,
                select_at(q.clone(), var("_shifted")),
            ),
            source,
            pushed_select.clone(),
            word_type(w)?,
        ))?
        .then(qfold.clone().mapped(
            lam(
                "_decision",
                boolean(),
                select_at(var("_decision"), pushed.clone()),
            ),
            pushed_select,
            chosen_select.clone(),
            word_type(w)?,
        ))?
        .then(
            Equality {
                ty: word_type(w)?,
                left: next.clone(),
                right: chosen_select.clone(),
                proof: refl(word_type(w)?, chosen_select),
            }
            .reverse(),
        )?;
        self.theorem(&format!("W{w}.StageRemainderFold{i}"), &plain, fold.clone())?;
        let fold = map_value(
            d,
            Equality {
                proof: dc(
                    &format!("W{w}.StageRemainderFold{i}"),
                    vec![x.clone(), y.clone()],
                ),
                ..fold
            },
        );
        let previous_bound = dc(
            &format!("W{w}.RemainderBound{i}"),
            vec![x.clone(), y.clone(), var("defined")],
        );
        let step = fold
            .clone()
            .on_left(mul(value(d, y.clone()), bit_value(q.clone())))
            .then(mul_right(value(d, y.clone()), map_bit(qfold)).on_right(value(d, next.clone())))?
            .then(instance(
                &format!("W{w}.RestoreConservation"),
                vec![r.clone(), y.clone(), b.clone(), previous_bound.clone()],
                add(
                    value(d, next.clone()),
                    mul(value(d, y.clone()), bit_value(chosen)),
                ),
                add(mul(two(), value(d, r.clone())), bit_value(b.clone())),
            ))?;
        self.theorem(&format!("W{w}.StepConservation{i}"), &args, step.clone())?;
        let bound = less_left(fold, value(d, y.clone())).then(Equality::boolean(
            less(value(d, next), value(d, y.clone())),
            bit(true),
            dc(
                &format!("W{w}.RestoreBound"),
                vec![r.clone(), y.clone(), b.clone(), previous_bound],
            ),
        ))?;
        self.theorem(&format!("W{w}.RemainderBound{}", i + 1), &args, bound)?;
        let qw = raw(w, x.clone(), y.clone());
        let before = prefix(w, i, qw.clone());
        let after = prefix(w, i + 1, qw.clone());
        let atq = at(qw.clone(), w, w - 1 - i)?;
        let qeq = instance(
            &format!("W{w}.PrefixDefinition{}", i + 1),
            vec![qw.clone()],
            after,
            add(mul(two(), before.clone()), bit_value(atq.clone())),
        )
        .then(
            map_bit(Equality::boolean(
                atq,
                q.clone(),
                refl(boolean(), q.clone()),
            ))
            .on_right(mul(two(), before.clone())),
        )?;
        let nextprefix = add(mul(two(), prefix(w, i, x.clone())), bit_value(b.clone()));
        let e = mul_right(value(d, y.clone()), qeq).on_right(value(d, rn.clone()));
        let combine = instance(
            "StageCombine",
            vec![
                value(d, r.clone()),
                value(d, y.clone()),
                before,
                bit_value(b),
                value(d, rn),
                bit_value(q),
                prefix(w, i, x.clone()),
                dc(
                    &format!("W{w}.StageConservation{i}"),
                    vec![x.clone(), y.clone(), var("defined")],
                ),
                step.proof,
            ],
            e.right.clone(),
            nextprefix.clone(),
        );
        let e = e.then(combine)?.then(
            instance(
                &format!("W{w}.PrefixDefinition{}", i + 1),
                vec![x.clone()],
                prefix(w, i + 1, x.clone()),
                nextprefix,
            )
            .reverse(),
        )?;
        self.theorem(&format!("W{w}.StageConservation{}", i + 1), &args, e)
    }
    fn defined_values(&mut self, w: usize) -> Result<(Equality, Equality)> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let args = [
            ("x", word_type(w)?),
            ("y", word_type(w)?),
            ("defined", guard(w, y.clone())),
        ];
        let iz = wc(w, "IsZero", vec![y.clone()]);
        let defined = Equality::boolean(bc("Not", vec![iz.clone()]), bit(true), var("defined"));
        let zero = Equality::boolean(
            bc("Not", vec![bc("Not", vec![iz.clone()])]),
            iz.clone(),
            bc("DoubleNot", vec![iz.clone()]),
        )
        .reverse()
        .then(bool_map("Not", defined))?
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        let mut values = vec![];
        for (op, target, suffix) in [
            (
                "UnsignedDiv",
                raw(w, x.clone(), y.clone()),
                "DefinedQuotientValue",
            ),
            (
                "UnsignedRem",
                remainder(w, w, x.clone(), y.clone()),
                "DefinedRemainderValue",
            ),
        ] {
            let actual = wc(w, op, vec![x.clone(), y.clone()]);
            let selected = wc(
                w,
                "Select",
                vec![iz.clone(), wc(w, "Zero", vec![]), target.clone()],
            );
            let first = Equality {
                ty: word_type(w)?,
                left: actual,
                right: selected.clone(),
                proof: refl(word_type(w)?, selected.clone()),
            };
            let selected_false = wc(
                w,
                "Select",
                vec![bit(false), wc(w, "Zero", vec![]), target.clone()],
            );
            let mapped = zero.clone().mapped(
                lam(
                    "_zero_guard",
                    boolean(),
                    wc(
                        w,
                        "Select",
                        vec![var("_zero_guard"), wc(w, "Zero", vec![]), target.clone()],
                    ),
                ),
                selected,
                selected_false.clone(),
                word_type(w)?,
            );
            let e = map_value(
                d,
                first.then(mapped)?.then(Equality {
                    ty: word_type(w)?,
                    left: selected_false,
                    right: target.clone(),
                    proof: refl(word_type(w)?, target),
                })?,
            );
            self.theorem(&format!("W{w}.{suffix}"), &args, e.clone())?;
            values.push(e);
        }
        Ok((values.remove(0), values.remove(0)))
    }
    fn finish_division(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let args = [
            ("x", word_type(w)?),
            ("y", word_type(w)?),
            ("defined", guard(w, y.clone())),
        ];
        let (q, r) = self.defined_values(w)?;
        let qw = raw(w, x.clone(), y.clone());
        let rw = remainder(w, w, x.clone(), y.clone());
        let divisor = value(d, y.clone());
        let previous = instance(
            &format!("W{w}.StageConservation{w}"),
            vec![x.clone(), y.clone(), var("defined")],
            add(
                value(d, rw.clone()),
                mul(divisor.clone(), prefix(w, w, qw.clone())),
            ),
            prefix(w, w, x.clone()),
        );
        let qvalue = instance(
            &format!("W{w}.PrefixValue"),
            vec![qw.clone()],
            prefix(w, w, qw.clone()),
            value(d, qw),
        );
        let xvalue = instance(
            &format!("W{w}.PrefixValue"),
            vec![x.clone()],
            prefix(w, w, x.clone()),
            value(d, x),
        );
        let e = previous
            .reverse()
            .then(mul_right(divisor.clone(), qvalue).on_right(value(d, rw.clone())))?;
        let e = xvalue
            .reverse()
            .then(e)?
            .then(
                r.clone()
                    .reverse()
                    .on_left(mul(divisor.clone(), q.right.clone())),
            )?
            .then(mul_right(divisor.clone(), q.clone().reverse()).on_right(r.left.clone()))?
            .then(Equality::natural(
                add(r.left.clone(), mul(divisor.clone(), q.left.clone())),
                add(mul(divisor.clone(), q.left.clone()), r.left.clone()),
                nc(
                    "AddComm",
                    vec![r.left.clone(), mul(divisor.clone(), q.left.clone())],
                ),
            ))?;
        self.theorem(&format!("W{w}.UnsignedDivisionConservation"), &args, e)?;
        let bound = less_left(r, divisor.clone()).then(Equality::boolean(
            less(value(d, rw), divisor),
            bit(true),
            dc(
                &format!("W{w}.RemainderBound{w}"),
                vec![var("x"), y, var("defined")],
            ),
        ))?;
        self.theorem(&format!("W{w}.UnsignedRemainderBound"), &args, bound)
    }
}
