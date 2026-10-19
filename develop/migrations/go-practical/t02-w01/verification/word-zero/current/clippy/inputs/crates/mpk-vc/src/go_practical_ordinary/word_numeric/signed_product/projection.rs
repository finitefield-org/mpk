//! Explicit checked balanced-Int projections before Nat algebra.
use super::*;
pub(super) fn part(negative: bool, x: Expr) -> Expr {
    ic(if negative { "Negative" } else { "Positive" }, vec![x])
}
fn product(a: Equality, b: Equality) -> Result<Equality> {
    mul_left(a.clone(), b.left.clone()).then(mul_right(a.right, b))
}

impl SignedProduct<'_> {
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
                mul(weight(depth(w)?), bit_value(at(word.clone(), w, w - 1)?))
            } else {
                value(depth(w)?, word.clone())
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                rhs,
                numeric(&format!("W{w}.Signed{label}"), vec![word.clone()]),
            ));
        }
        if **head == constant(&format!("{SIGNED_PRODUCT}.W{w}.Correction")) {
            let [a, b] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let s = super::scalar::Scalars::word(w, a.clone(), b.clone())?;
            return Ok(instance(
                &format!("W{w}.Correction{label}"),
                vec![a.clone(), b.clone()],
                part(negative, x),
                if negative { s.negative() } else { s.positive() },
            ));
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
            let a_pos = self.integer_part(w, false, a.clone())?;
            let a_neg = self.integer_part(w, true, a.clone())?;
            let b_pos = self.integer_part(w, false, b.clone())?;
            let b_neg = self.integer_part(w, true, b.clone())?;
            let (left, right) = if negative {
                (product(a_pos, b_neg)?, product(a_neg, b_pos)?)
            } else {
                (product(a_pos, b_pos)?, product(a_neg, b_neg)?)
            };
            return Equality::natural(
                part(negative, x.clone()),
                add(left.left.clone(), right.left.clone()),
                ic(&format!("{label}OfMul"), vec![a.clone(), b.clone()]),
            )
            .then(left.clone().on_left(right.left.clone()))?
            .then(right.on_right(left.right));
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
