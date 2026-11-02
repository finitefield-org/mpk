//! Unbounded power complement, addition and positivity, without huge literals.
use super::*;
pub(super) fn numeral_add(a: usize, b: usize) -> Result<Equality> {
    if b == 0 {
        return Ok(law(
            "AddZero",
            vec![numeral(a)],
            add(numeral(a), zero()),
            numeral(a),
        ));
    }
    law(
        "AddSucc",
        vec![numeral(a), numeral(b - 1)],
        add(numeral(a), numeral(b)),
        succ(add(numeral(a), numeral(b - 1))),
    )
    .then(numeral_add(a, b - 1)?.on_succ())
}
pub(super) fn split_power(a: usize, b: usize) -> Result<Equality> {
    map_power(numeral_add(a, b)?)
        .reverse()
        .then(Equality::natural(
            power(add(numeral(a), numeral(b))),
            mul(coefficient(a), coefficient(b)),
            numeric("PowerTwoAdd", vec![numeral(a), numeral(b)]),
        ))
}
impl Shifts<'_> {
    pub(super) fn powers(&mut self) -> Result<()> {
        let n = var("n");
        self.emit(
            "PowerMinusOne",
            &[("n", nat())],
            nat(),
            call(
                "Std.Nat.rec",
                vec![
                    zero(),
                    lam(
                        "_index",
                        nat(),
                        lam("_previous", nat(), add(mul(two(), var("_previous")), one())),
                    ),
                    n.clone(),
                ],
            ),
            false,
        )?;
        self.theorem(
            "PowerMinusZero",
            &[],
            Equality::natural(minus(zero()), zero(), refl(nat(), zero())),
        )?;
        self.theorem(
            "PowerMinusSucc",
            &[("n", nat())],
            Equality::natural(
                minus(succ(n.clone())),
                add(mul(two(), minus(n.clone())), one()),
                refl(nat(), add(mul(two(), minus(n)), one())),
            ),
        )?;
        let i = var("_index");
        let property = eq(nat(), add(minus(i.clone()), one()), power(i.clone()));
        let base = known("PowerMinusZero", vec![], minus(zero()), zero())
            .on_left(one())
            .then(law("ZeroAdd", vec![one()], add(zero(), one()), one()))?
            .then(
                Equality::natural(power(zero()), one(), numeric("PowerTwoZero", vec![])).reverse(),
            )?;
        let e = known(
            "PowerMinusSucc",
            vec![i.clone()],
            minus(succ(i.clone())),
            add(mul(two(), minus(i.clone())), one()),
        )
        .on_left(one());
        let args = [("_index", nat()), ("_prior", property.clone())];
        let p = self.polynomial(
            &args,
            e.right.clone(),
            mul(two(), add(minus(i.clone()), one())),
        )?;
        let e = e
            .then(p)?
            .then(mul_right(
                two(),
                Equality::natural(
                    add(minus(i.clone()), one()),
                    power(i.clone()),
                    var("_prior"),
                ),
            ))?
            .then(
                Equality::natural(
                    power(succ(i.clone())),
                    mul(two(), power(i.clone())),
                    numeric("PowerTwoSucc", vec![i.clone()]),
                )
                .reverse(),
            )?;
        let step = lam("_index", nat(), lam("_prior", property.clone(), e.proof));
        self.theorem(
            "PowerMinusBalance",
            &[("n", nat())],
            Equality::natural(
                add(minus(var("n")), one()),
                power(var("n")),
                induct("_index", property, base.proof, step, var("n")),
            ),
        )?;
        self.power_fill_add()?;
        self.power_positive()?;
        self.small_powers()
    }
    fn power_fill_add(&mut self) -> Result<()> {
        let (m, n) = (var("m"), var("n"));
        let args = [("m", nat()), ("n", nat())];
        let sum = add(minus(m.clone()), mul(power(m.clone()), minus(n.clone())));
        let e = self.polynomial(
            &args,
            add(sum.clone(), one()),
            add(
                add(minus(m.clone()), one()),
                mul(power(m.clone()), minus(n.clone())),
            ),
        )?;
        let e = e.then(
            known(
                "PowerMinusBalance",
                vec![m.clone()],
                add(minus(m.clone()), one()),
                power(m.clone()),
            )
            .on_left(mul(power(m.clone()), minus(n.clone()))),
        )?;
        let p = self.polynomial(
            &args,
            e.right.clone(),
            mul(power(m.clone()), add(minus(n.clone()), one())),
        )?;
        let e = e
            .then(p)?
            .then(mul_right(
                power(m.clone()),
                known(
                    "PowerMinusBalance",
                    vec![n.clone()],
                    add(minus(n.clone()), one()),
                    power(n.clone()),
                ),
            ))?
            .then(
                Equality::natural(
                    power(add(m.clone(), n.clone())),
                    mul(power(m.clone()), power(n.clone())),
                    numeric("PowerTwoAdd", vec![m.clone(), n.clone()]),
                )
                .reverse(),
            )?
            .then(
                known(
                    "PowerMinusBalance",
                    vec![add(m.clone(), n.clone())],
                    add(minus(add(m.clone(), n.clone())), one()),
                    power(add(m.clone(), n.clone())),
                )
                .reverse(),
            )?;
        let target = minus(add(m, n));
        self.theorem(
            "PowerMinusAdd",
            &args,
            Equality::natural(
                sum.clone(),
                target.clone(),
                nc("AddCancel", vec![sum, target, one(), e.proof]),
            )
            .reverse(),
        )
    }
    fn power_positive(&mut self) -> Result<()> {
        let n = var("n");
        let e = known(
            "PowerMinusBalance",
            vec![n.clone()],
            add(minus(n.clone()), one()),
            power(n.clone()),
        )
        .reverse()
        .then(law(
            "AddOne",
            vec![minus(n.clone())],
            add(minus(n.clone()), one()),
            succ(minus(n.clone())),
        ))?;
        self.theorem(
            "PowerPositive",
            &[("n", nat())],
            less_right(zero(), e).then(Equality::boolean(
                less(zero(), succ(minus(n.clone()))),
                bit(true),
                nc("LessZeroSucc", vec![minus(n)]),
            ))?,
        )
    }
    fn small_powers(&mut self) -> Result<()> {
        let n = var("n");
        let e = law(
            "MulAddLeft",
            vec![one(), one(), n.clone()],
            mul(two(), n.clone()),
            add(mul(one(), n.clone()), mul(one(), n.clone())),
        )
        .then(
            law("OneMul", vec![n.clone()], mul(one(), n.clone()), n.clone())
                .on_left(mul(one(), n.clone())),
        )?
        .then(
            law("OneMul", vec![n.clone()], mul(one(), n.clone()), n.clone()).on_right(n.clone()),
        )?;
        self.theorem("TwoMul", &[("n", nat())], e)?;
        self.theorem(
            "SmallPower0",
            &[],
            Equality::natural(coefficient(0), one(), numeric("PowerTwoZero", vec![])),
        )?;
        for i in 1..=6 {
            let k = 1 << (i - 1);
            let e = Equality::natural(
                coefficient(i),
                mul(two(), coefficient(i - 1)),
                numeric("PowerTwoSucc", vec![numeral(i - 1)]),
            );
            let e = e
                .then(mul_right(
                    two(),
                    known(
                        &format!("SmallPower{}", i - 1),
                        vec![],
                        coefficient(i - 1),
                        numeral(k),
                    ),
                ))?
                .then(known(
                    "TwoMul",
                    vec![numeral(k)],
                    mul(two(), numeral(k)),
                    add(numeral(k), numeral(k)),
                ))?
                .then(numeral_add(k, k)?)?;
            self.theorem(&format!("SmallPower{i}"), &[], e)?;
        }
        Ok(())
    }
}
