//! Bind every count bit to the true small/large source branch, without clipping.
use super::counts::{digits, high, high_value};
use super::*;
mod natural;
fn complement(i: usize, x: Expr) -> Expr {
    sh(&format!("CountLowComplement{i}"), vec![x])
}
fn excess(d: usize, x: Expr) -> Expr {
    sh(&format!("CountExcess{d}"), vec![x])
}
fn flag_arg(d: usize, x: Expr, b: bool) -> Binder {
    ("high_branch", eq(boolean(), high(d, x), bit(b)))
}
fn small_power(i: usize) -> Equality {
    known(
        &format!("SmallPower{i}"),
        vec![],
        coefficient(i),
        numeral(1 << i),
    )
}
fn count_balance(d: usize, x: Expr) -> Equality {
    known(
        &format!("CountBalance{d}"),
        vec![x.clone()],
        value(6, x.clone()),
        add(digits(d, x.clone()), mul(coefficient(d), high_value(d, x))),
    )
}
impl Shifts<'_> {
    pub(super) fn count_bounds(&mut self) -> Result<()> {
        self.count_bound_natural()?;
        self.count_prefixes()?;
        for d in 3..=6 {
            self.count_branches(d)?;
        }
        Ok(())
    }
    fn count_prefixes(&mut self) -> Result<()> {
        for i in 0..=6 {
            self.count_prefix(i)?;
        }
        Ok(())
    }
    pub(super) fn count_prefix(&mut self, i: usize) -> Result<()> {
        let x = var("count");
        let args = [("count", word_type(64)?)];
        let body = if i == 0 {
            zero()
        } else {
            add(
                complement(i - 1, x.clone()),
                mul(
                    coefficient(i - 1),
                    bit_value(bc("Not", vec![at(x.clone(), 64, i - 1)?])),
                ),
            )
        };
        self.emit(
            &format!("CountLowComplement{i}"),
            &args,
            nat(),
            body.clone(),
            false,
        )?;
        let cdef = Equality::natural(complement(i, x.clone()), body.clone(), refl(nat(), body));
        let balance = if i == 0 {
            Equality::natural(digits(0, x.clone()), zero(), refl(nat(), zero()))
                .on_left(complement(0, x.clone()))
                .then(cdef.on_right(zero()))?
                .on_left(one())
                .then(law("AddZero", vec![zero()], add(zero(), zero()), zero()).on_left(one()))?
                .then(law("ZeroAdd", vec![one()], add(zero(), one()), one()))?
                .then(
                    Equality::natural(coefficient(0), one(), numeric("PowerTwoZero", vec![]))
                        .reverse(),
                )?
        } else {
            let h = i - 1;
            let a = digits(h, x.clone());
            let z = complement(h, x.clone());
            let b = at(x.clone(), 64, h)?;
            let v = bit_value(b.clone());
            let n = bit_value(bc("Not", vec![b.clone()]));
            let p = coefficient(h);
            let db = add(a.clone(), mul(p.clone(), v.clone()));
            let start = Equality::natural(digits(i, x.clone()), db.clone(), refl(nat(), db))
                .on_left(complement(i, x.clone()))
                .then(cdef.on_right(add(a.clone(), mul(p.clone(), v.clone()))))?
                .on_left(one());
            let target = mul(two(), p.clone());
            let step = known(
                "CountPrefixComplementStep",
                vec![
                    a,
                    z,
                    p.clone(),
                    v,
                    n,
                    sh(&format!("CountLowBalance{h}"), vec![x.clone()]),
                    sh("CountBitComplement", vec![b]),
                ],
                start.right.clone(),
                target.clone(),
            );
            start.then(step)?.then(
                Equality::natural(
                    coefficient(i),
                    target,
                    numeric("PowerTwoSucc", vec![numeral(h)]),
                )
                .reverse(),
            )?
        };
        self.theorem(&format!("CountLowBalance{i}"), &args, balance.clone())?;
        self.theorem(
            &format!("CountLowBound{i}"),
            &args,
            Equality::boolean(
                less(digits(i, x.clone()), coefficient(i)),
                bit(true),
                sh(
                    "CountComplementBound",
                    vec![
                        digits(i, x.clone()),
                        complement(i, x.clone()),
                        coefficient(i),
                        sh(&format!("CountLowBalance{i}"), vec![x.clone()]),
                    ],
                ),
            ),
        )?;
        Ok(())
    }
    fn count_branches(&mut self, d: usize) -> Result<()> {
        let x = var("count");
        let args = [("count", word_type(64)?)];
        let (h, v, lo) = (
            high(d, x.clone()),
            high_value(d, x.clone()),
            digits(d, x.clone()),
        );
        let false_args = [args[0].clone(), flag_arg(d, x.clone(), false)];
        let true_args = [args[0].clone(), flag_arg(d, x.clone(), true)];
        let reflection = Equality::boolean(
            equal(v.clone(), zero()),
            bc("Not", vec![h.clone()]),
            sh(&format!("CountHighZero{d}"), vec![x.clone()]),
        );
        let hz = reflection
            .clone()
            .then(bool_map(
                "Not",
                Equality::boolean(h.clone(), bit(false), var("high_branch")),
            ))?
            .then(Equality::boolean(
                bc("Not", vec![bit(false)]),
                bit(true),
                refl(boolean(), bit(true)),
            ))?;
        let vz = Equality::natural(
            v.clone(),
            zero(),
            nc("EqualReflect", vec![v.clone(), zero(), hz.proof]),
        );
        let small = count_balance(d, x.clone())
            .then(mul_right(coefficient(d), vz).on_right(lo.clone()))?
            .then(
                law(
                    "MulZero",
                    vec![coefficient(d)],
                    mul(coefficient(d), zero()),
                    zero(),
                )
                .on_right(lo.clone()),
            )?
            .then(law(
                "AddZero",
                vec![lo.clone()],
                add(lo.clone(), zero()),
                lo.clone(),
            ))?;
        self.theorem(&format!("CountSmallValue{d}"), &false_args, small.clone())?;
        self.theorem(
            &format!("CountSmallPower{d}"),
            &false_args,
            map_power(small),
        )?;
        let positive = Equality::boolean(
            bc("Not", vec![equal(v.clone(), zero())]),
            less(zero(), v.clone()),
            sh("CountNonzeroPositive", vec![v.clone()]),
        )
        .reverse();
        let positive = positive
            .then(bool_map("Not", reflection))?
            .then(Equality::boolean(
                bc("Not", vec![bc("Not", vec![h.clone()])]),
                h.clone(),
                bc("DoubleNot", vec![h.clone()]),
            ))?
            .then(Equality::boolean(h.clone(), bit(true), var("high_branch")))?;
        self.theorem(
            &format!("CountHighPositive{d}"),
            &true_args,
            positive.clone(),
        )?;
        let subhigh = sub(v.clone(), one());
        let body = add(lo.clone(), mul(coefficient(d), subhigh.clone()));
        self.emit(
            &format!("CountExcess{d}"),
            &args,
            nat(),
            body.clone(),
            false,
        )?;
        let decomposition = known(
            "CountPositiveDecomposition",
            vec![v.clone(), positive.proof],
            v.clone(),
            add(one(), subhigh),
        );
        let large = count_balance(d, x.clone())
            .then(mul_right(coefficient(d), decomposition).on_right(lo))?;
        let target = add(coefficient(d), body.clone());
        let poly = self.polynomial(&true_args, large.right.clone(), target)?;
        let large = large.then(poly)?.then(
            Equality::natural(excess(d, x.clone()), body.clone(), refl(nat(), body))
                .reverse()
                .on_right(coefficient(d)),
        )?;
        self.theorem(&format!("CountLargeValue{d}"), &true_args, large.clone())?;
        let large_power = map_power(large).then(Equality::natural(
            power(add(coefficient(d), excess(d, x.clone()))),
            mul(power(coefficient(d)), power(excess(d, x.clone()))),
            numeric("PowerTwoAdd", vec![coefficient(d), excess(d, x.clone())]),
        ))?;
        let outer_power = map_power(small_power(d)).then(Equality::natural(
            coefficient(1 << d),
            weight(d),
            numeric(&format!("PowerTwoWeight{d}"), vec![]),
        ))?;
        self.theorem(
            &format!("CountLargePower{d}"),
            &true_args,
            large_power.then(mul_left(outer_power, power(excess(d, x.clone()))))?,
        )?;
        let property = |e: &BTreeMap<&'static str, Expr>| {
            pi(
                "_branch",
                eq(boolean(), h.clone(), e["_count_flag"].clone()),
                eq(
                    boolean(),
                    less(value(6, x.clone()), coefficient(d)),
                    bc("Not", vec![e["_count_flag"].clone()]),
                ),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_count_flag"] == bit(true);
            let value_eq = if yes {
                known(
                    &format!("CountLargeValue{d}"),
                    vec![x.clone(), var("_branch")],
                    value(6, x.clone()),
                    add(coefficient(d), excess(d, x.clone())),
                )
            } else {
                known(
                    &format!("CountSmallValue{d}"),
                    vec![x.clone(), var("_branch")],
                    value(6, x.clone()),
                    digits(d, x.clone()),
                )
            };
            let step = less_left(value_eq, coefficient(d));
            let finish = if yes {
                Equality::boolean(
                    step.right.clone(),
                    bit(false),
                    sh(
                        "CountNoLessExtension",
                        vec![coefficient(d), excess(d, x.clone())],
                    ),
                )
            } else {
                Equality::boolean(
                    step.right.clone(),
                    bit(true),
                    sh(&format!("CountLowBound{d}"), vec![x.clone()]),
                )
            };
            let proof = step.then(finish)?.then(
                Equality::boolean(
                    bc("Not", vec![bit(yes)]),
                    bit(!yes),
                    refl(boolean(), bit(!yes)),
                )
                .reverse(),
            )?;
            Ok(lam(
                "_branch",
                eq(boolean(), h.clone(), bit(yes)),
                proof.proof,
            ))
        };
        let cases = boolean_cases(&["_count_flag"], &BTreeMap::new(), &property, &mut leaf)?;
        let proof = Equality::boolean(
            less(value(6, x.clone()), coefficient(d)),
            bc("Not", vec![h.clone()]),
            app(cases, vec![h.clone(), refl(boolean(), h)]),
        );
        let width = less_right(value(6, x.clone()), small_power(d))
            .reverse()
            .then(proof)?;
        self.theorem(&format!("CountBelowWidth{d}"), &args, width)
    }
}
