//! Explicit ordinary balanced-Int projections before Nat algebra.
use super::*;
pub(super) fn part(negative: bool, x: Expr) -> Expr {
    ic(if negative { "Negative" } else { "Positive" }, vec![x])
}
fn product(a: Equality, b: Equality) -> Result<Equality> {
    mul_left(a.clone(), b.left.clone()).then(mul_right(a.right, b))
}
impl Magnitude<'_> {
    fn integer_part(&self, w: usize, negative: bool, x: Expr) -> Result<Equality> {
        let Expr::Apply(head, args) = &x else {
            return Err(GoOrdinaryError::Linkage);
        };
        let label = if negative { "Negative" } else { "Positive" };
        if **head == constant(&format!("{PREFIX}.W{w}.Signed")) {
            let [word] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let rhs = if negative {
                mul(weight(depth(w)?), bit_value(sign(w, word.clone())?))
            } else {
                value(depth(w)?, word.clone())
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                rhs,
                numeric(&format!("W{w}.Signed{label}"), vec![word.clone()]),
            ));
        }
        for name in ["NegationCorrection", "SignedMagnitude"] {
            if **head == constant(&format!("{MAGNITUDE}.W{w}.{name}")) {
                let [word] = args.as_slice() else {
                    return Err(GoOrdinaryError::Linkage);
                };
                let rhs = if name == "NegationCorrection" {
                    if negative {
                        bit_value(wc(w, "DivideDefined", vec![word.clone()]))
                    } else {
                        add(
                            bit_value(sign(w, word.clone())?),
                            bit_value(sign(w, neg(w, word.clone()))?),
                        )
                    }
                } else {
                    let av = value(depth(w)?, wc(w, "AbsBits", vec![word.clone()]));
                    if negative {
                        choose(sign(w, word.clone())?, av, zero(), nat())
                    } else {
                        choose(sign(w, word.clone())?, zero(), av, nat())
                    }
                };
                let suffix = if name == "NegationCorrection" {
                    format!("NegationCorrection{label}")
                } else {
                    format!("Magnitude{label}")
                };
                return Ok(instance(
                    &format!("W{w}.{suffix}"),
                    vec![word.clone()],
                    part(negative, x),
                    rhs,
                ));
            }
        }
        if **head == constant("Mpk.Go.Practical.Int.FromNat") {
            let [n] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                if negative { zero() } else { n.clone() },
                ic(&format!("{label}OfFromNat"), vec![n.clone()]),
            ));
        }
        if **head == constant("Mpk.Go.Practical.Int.Negate") {
            let [n] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            return Equality::natural(
                part(negative, x.clone()),
                part(!negative, n.clone()),
                ic(&format!("{label}OfNegate"), vec![n.clone()]),
            )
            .then(self.integer_part(w, !negative, n.clone())?);
        }
        let [a, b] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        if **head == constant("Mpk.Go.Practical.Int.Add") {
            let pa = self.integer_part(w, negative, a.clone())?;
            let pb = self.integer_part(w, negative, b.clone())?;
            return Equality::natural(
                part(negative, x.clone()),
                add(pa.left.clone(), pb.left.clone()),
                ic(&format!("{label}OfAdd"), vec![a.clone(), b.clone()]),
            )
            .then(pa.clone().on_left(pb.left.clone()))?
            .then(pb.on_right(pa.right));
        }
        if **head == constant("Mpk.Go.Practical.Int.Mul") {
            let ap = self.integer_part(w, false, a.clone())?;
            let an = self.integer_part(w, true, a.clone())?;
            let bp = self.integer_part(w, false, b.clone())?;
            let bn = self.integer_part(w, true, b.clone())?;
            let (l, r) = if negative {
                (product(ap, bn)?, product(an, bp)?)
            } else {
                (product(ap, bp)?, product(an, bn)?)
            };
            return Equality::natural(
                part(negative, x.clone()),
                add(l.left.clone(), r.left.clone()),
                ic(&format!("{label}OfMul"), vec![a.clone(), b.clone()]),
            )
            .then(l.clone().on_left(r.left.clone()))?
            .then(r.on_right(l.right));
        }
        Err(GoOrdinaryError::Linkage)
    }
    pub(super) fn cross_parts(
        &self,
        w: usize,
        left: Expr,
        right: Expr,
    ) -> Result<(Equality, Equality)> {
        let lp = self.integer_part(w, false, left.clone())?;
        let ln = self.integer_part(w, true, left)?;
        let rp = self.integer_part(w, false, right.clone())?;
        let rn = self.integer_part(w, true, right)?;
        Ok((
            lp.clone()
                .on_left(rn.left.clone())
                .then(rn.on_right(lp.right))?,
            rp.clone()
                .on_left(ln.left.clone())
                .then(ln.on_right(rp.right))?,
        ))
    }
}
