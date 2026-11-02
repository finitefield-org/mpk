//! Genuine signed reconstruction from an ordinary Nat quotient/remainder law.
use super::*;

impl Refinement<'_> {
    pub(super) fn signed_nat(&mut self) -> Result<()> {
        let (flag, amount) = (var("flag"), var("amount"));
        let args = [("flag", boolean()), ("amount", nat())];
        let positive = choose(flag.clone(), zero(), amount.clone(), nat());
        let negative = choose(flag.clone(), amount.clone(), zero(), nat());
        self.emit(
            "SignedNat",
            &args,
            integer(),
            ic("Make", vec![positive.clone(), negative.clone()]),
            false,
        )?;
        for (is_negative, label, rhs) in
            [(false, "Positive", positive), (true, "Negative", negative)]
        {
            self.theorem(
                &format!("SignedNat{label}"),
                &args,
                Equality::natural(
                    part(is_negative, signed_nat(flag.clone(), amount.clone())),
                    rhs.clone(),
                    refl(nat(), rhs),
                ),
            )?;
        }
        Ok(())
    }
    pub(super) fn scalar_division(&mut self) -> Result<()> {
        let (a, b, q, r) = (var("a"), var("b"), var("q"), var("r"));
        let sum = add(mul(b.clone(), q.clone()), r.clone());
        let args = [
            ("a", nat()),
            ("b", nat()),
            ("q", nat()),
            ("r", nat()),
            ("balance", eq(nat(), a.clone(), sum.clone())),
        ];
        let expressions = |sx: Expr, sy: Expr| {
            (
                signed_nat(sx.clone(), a.clone()),
                ic(
                    "Add",
                    vec![
                        ic(
                            "Mul",
                            vec![
                                signed_nat(sy.clone(), b.clone()),
                                signed_nat(bc("Xor", vec![sx.clone(), sy]), q.clone()),
                            ],
                        ),
                        signed_nat(sx, r.clone()),
                    ],
                ),
            )
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let (left, right) = expressions(env["sx"].clone(), env["sy"].clone());
            equivalent(left, right)
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let negative = env["sx"] == bit(true);
            let (left, right) = expressions(env["sx"].clone(), env["sy"].clone());
            // No word operation is involved in this generic scalar theorem.
            let (lp, rp) = self.cross_parts(8, left, right)?;
            let (lt, rt) = if negative {
                (sum.clone(), a.clone())
            } else {
                (a.clone(), sum.clone())
            };
            let l = self.polynomial(&args, lp.right.clone(), lt.clone())?;
            let r = self.polynomial(&args, rp.right.clone(), rt.clone())?;
            let balance = Equality::natural(a.clone(), sum.clone(), var("balance"));
            let middle = if negative { balance.reverse() } else { balance };
            Ok(lp
                .then(l)?
                .then(middle)?
                .then(r.reverse())?
                .then(rp.reverse())?
                .proof)
        };
        let proof = boolean_cases(&["sx", "sy"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "ScalarSignedDivision",
            &args,
            pi(
                "sx",
                boolean(),
                pi(
                    "sy",
                    boolean(),
                    prop(&[("sx", var("sx")), ("sy", var("sy"))].into_iter().collect()),
                ),
            ),
            proof,
            true,
        )
    }
}
