//! Genuine unbounded natural exponentiation and its ordinary addition law.
use super::*;

fn power_succ(n: Expr) -> Equality {
    instance(
        "PowerTwoSucc",
        vec![n.clone()],
        power(succ(n.clone())),
        mul(two(), power(n)),
    )
}
fn map_power(e: Equality) -> Equality {
    let left = power(e.left.clone());
    let right = power(e.right.clone());
    e.mapped(
        lam("_exponent", nat(), power(var("_exponent"))),
        left,
        right,
        nat(),
    )
}

impl Numeric<'_> {
    pub(super) fn powers(&mut self) -> Result<()> {
        let n = var("n");
        self.emit(
            "PowerTwo",
            &[("n", nat())],
            nat(),
            call(
                "Std.Nat.rec",
                vec![
                    one(),
                    lam(
                        "_index",
                        nat(),
                        lam("_power", nat(), mul(two(), var("_power"))),
                    ),
                    n.clone(),
                ],
            ),
            false,
        )?;
        let p0 = instance("PowerTwoZero", vec![], power(zero()), one());
        self.theorem(
            "PowerTwoZero",
            &[],
            Equality::natural(p0.left.clone(), one(), refl(nat(), one())),
        )?;
        self.theorem(
            "PowerTwoSucc",
            &[("n", nat())],
            Equality::natural(
                power(succ(n.clone())),
                mul(two(), power(n.clone())),
                refl(nat(), mul(two(), power(n))),
            ),
        )?;
        let (m, n, i) = (var("m"), var("n"), var("_index"));
        let pm = power(m.clone());
        let property = eq(
            nat(),
            power(add(m.clone(), i.clone())),
            mul(pm.clone(), power(i.clone())),
        );
        let base = map_power(Equality::natural(
            add(m.clone(), zero()),
            m.clone(),
            nc("AddZero", vec![m.clone()]),
        ));
        let rhs = mul_right(pm.clone(), p0.clone()).then(Equality::natural(
            mul(pm.clone(), one()),
            pm.clone(),
            nc("MulOne", vec![pm.clone()]),
        ))?;
        let base = base.then(rhs.reverse())?;
        let e = map_power(Equality::natural(
            add(m.clone(), succ(i.clone())),
            succ(add(m.clone(), i.clone())),
            nc("AddSucc", vec![m.clone(), i.clone()]),
        ))
        .then(power_succ(add(m.clone(), i.clone())))?
        .then(mul_right(
            two(),
            Equality::natural(
                power(add(m.clone(), i.clone())),
                mul(pm.clone(), power(i.clone())),
                var("_prior"),
            ),
        ))?;
        let args = [("m", nat()), ("_index", nat())];
        let next = self.polynomial(
            &args,
            e.right.clone(),
            mul(pm.clone(), mul(two(), power(i.clone()))),
        )?;
        let e = e
            .then(next)?
            .then(mul_right(pm.clone(), power_succ(i.clone()).reverse()))?;
        let step = lam("_index", nat(), lam("_prior", property.clone(), e.proof));
        let proof = induct("_index", property, base.proof, step, n.clone());
        self.theorem(
            "PowerTwoAdd",
            &[("m", nat()), ("n", nat())],
            Equality::natural(
                power(add(m.clone(), n.clone())),
                mul(power(m), power(n)),
                proof,
            ),
        )?;
        for d in 0..=6 {
            let e = if d == 0 {
                power_succ(zero())
                    .then(mul_right(two(), p0.clone()))?
                    .then(Equality::natural(
                        mul(two(), one()),
                        two(),
                        nc("MulOne", vec![two()]),
                    ))?
            } else {
                let half = numeral(1 << (d - 1));
                let p = power(half.clone());
                let e = instance(
                    "PowerTwoAdd",
                    vec![half.clone(), half],
                    power(numeral(1 << d)),
                    mul(p.clone(), p.clone()),
                );
                let prev = instance(
                    &format!("PowerTwoWeight{}", d - 1),
                    vec![],
                    p.clone(),
                    weight(d - 1),
                );
                e.then(mul_left(prev.clone(), p))?
                    .then(mul_right(weight(d - 1), prev))?
            };
            self.theorem(
                &format!("PowerTwoWeight{d}"),
                &[],
                e.then(weight_definition(d).reverse())?,
            )?;
        }
        Ok(())
    }
}
