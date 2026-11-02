//! Shared unbounded Nat identities; every premise is a genuine source equation.
use super::*;
impl Shifts<'_> {
    pub(super) fn right_saturation_scalar(&mut self) -> Result<()> {
        let (m, o, p, e, q, f) = (
            var("modulus"),
            var("ones"),
            var("full_power"),
            var("full_minus"),
            var("excess_power"),
            var("excess_minus"),
        );
        let natural = [
            "modulus",
            "ones",
            "full_power",
            "full_minus",
            "excess_power",
            "excess_minus",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let premises = vec![
            ("ones_balance", eq(nat(), add(o.clone(), one()), m.clone())),
            ("full_balance", eq(nat(), add(e.clone(), one()), p.clone())),
            (
                "excess_balance",
                eq(nat(), add(f.clone(), one()), q.clone()),
            ),
            (
                "power_factor",
                eq(nat(), p.clone(), mul(m.clone(), q.clone())),
            ),
        ];
        let mut args = natural.clone();
        args.extend(premises.clone());
        let l = mul(m.clone(), e.clone());
        let r = add(mul(p.clone(), o.clone()), mul(m.clone(), f.clone()));
        let left = self
            .polynomial(
                &args,
                add(l.clone(), m.clone()),
                mul(m.clone(), add(e.clone(), one())),
            )?
            .then(mul_right(
                m.clone(),
                Equality::natural(add(e.clone(), one()), p.clone(), var("full_balance")),
            ))?;
        let right = self
            .polynomial(
                &args,
                add(r.clone(), m.clone()),
                add(
                    mul(p.clone(), o.clone()),
                    mul(m.clone(), add(f.clone(), one())),
                ),
            )?
            .then(
                mul_right(
                    m.clone(),
                    Equality::natural(add(f.clone(), one()), q.clone(), var("excess_balance")),
                )
                .on_right(mul(p.clone(), o.clone())),
            )?
            .then(
                Equality::natural(p.clone(), mul(m.clone(), q.clone()), var("power_factor"))
                    .reverse()
                    .on_right(mul(p.clone(), o.clone())),
            )?;
        let poly = self.polynomial(
            &args,
            right.right.clone(),
            mul(p.clone(), add(o.clone(), one())),
        )?;
        let right = right
            .then(poly)?
            .then(mul_right(
                p.clone(),
                Equality::natural(add(o.clone(), one()), m.clone(), var("ones_balance")),
            ))?
            .then(law(
                "MulComm",
                vec![p.clone(), m.clone()],
                mul(p.clone(), m.clone()),
                mul(m.clone(), p.clone()),
            ))?;
        let cancel = left.then(right.reverse())?;
        self.theorem(
            "RightSaturationCoefficient",
            &args,
            Equality::natural(
                l.clone(),
                r.clone(),
                nc("AddCancel", vec![l, r, m.clone(), cancel.proof]),
            ),
        )?;
        let (u, b) = (var("input"), var("bit_value"));
        let mut args = vec![("input", nat())];
        args.extend(natural);
        args.push(("bit_value", nat()));
        args.extend(premises);
        let coeff = known(
            "RightSaturationCoefficient",
            vec![
                m.clone(),
                o.clone(),
                p.clone(),
                e.clone(),
                q,
                f.clone(),
                var("ones_balance"),
                var("full_balance"),
                var("excess_balance"),
                var("power_factor"),
            ],
            mul(m.clone(), e.clone()),
            add(mul(p.clone(), o.clone()), mul(m.clone(), f.clone())),
        );
        let start = mul_left(coeff, b.clone()).on_right(u.clone());
        let target = add(mul(p, mul(o, b.clone())), add(u, mul(mul(m, f), b)));
        let poly = self.polynomial(&args, start.right.clone(), target)?;
        self.theorem("RightSaturatedConservation", &args, start.then(poly)?)
    }
    pub(super) fn right_saturation_bound(&mut self) -> Result<()> {
        let (u, n, m, e, b, c, q, p) = (
            var("input"),
            var("not_input"),
            var("modulus"),
            var("excess_minus"),
            var("bit_value"),
            var("not_bit_value"),
            var("excess_power"),
            var("full_power"),
        );
        let ib = add(add(u.clone(), n.clone()), one());
        let bits = add(b.clone(), c.clone());
        let mut args = [
            "input",
            "not_input",
            "modulus",
            "excess_minus",
            "bit_value",
            "not_bit_value",
            "excess_power",
            "full_power",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        args.extend([
            ("input_balance", eq(nat(), ib.clone(), m.clone())),
            ("bit_balance", eq(nat(), bits.clone(), one())),
            (
                "excess_balance",
                eq(nat(), add(e.clone(), one()), q.clone()),
            ),
            (
                "power_factor",
                eq(nat(), p.clone(), mul(m.clone(), q.clone())),
            ),
        ]);
        let rem = add(u, mul(mul(m.clone(), e.clone()), b));
        let gap = add(n, mul(mul(m.clone(), e.clone()), c));
        let lhs = add(add(rem.clone(), gap.clone()), one());
        let proof = self
            .polynomial(
                &args,
                lhs,
                add(ib.clone(), mul(mul(m.clone(), e.clone()), bits.clone())),
            )?
            .then(
                Equality::natural(ib, m.clone(), var("input_balance"))
                    .on_left(mul(mul(m.clone(), e.clone()), bits.clone())),
            )?
            .then(
                mul_right(
                    mul(m.clone(), e.clone()),
                    Equality::natural(bits, one(), var("bit_balance")),
                )
                .on_right(m.clone()),
            )?
            .then(
                law(
                    "MulOne",
                    vec![mul(m.clone(), e.clone())],
                    mul(mul(m.clone(), e.clone()), one()),
                    mul(m.clone(), e.clone()),
                )
                .on_right(m.clone()),
            )?;
        let poly = self.polynomial(
            &args,
            proof.right.clone(),
            mul(m.clone(), add(e.clone(), one())),
        )?;
        let proof = proof
            .then(poly)?
            .then(mul_right(
                m.clone(),
                Equality::natural(add(e, one()), q.clone(), var("excess_balance")),
            ))?
            .then(Equality::natural(p.clone(), mul(m, q), var("power_factor")).reverse())?;
        self.theorem(
            "RightSaturatedRemainderBound",
            &args,
            Equality::boolean(
                less(rem.clone(), p.clone()),
                bit(true),
                sh("CountComplementBound", vec![rem, gap, p, proof.proof]),
            ),
        )
    }
}
