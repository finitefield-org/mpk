//! Explicit balanced-Int projections before ordinary Nat algebra.
use super::*;

fn product(a: Equality, b: Equality) -> Result<Equality> {
    mul_left(a.clone(), b.left.clone()).then(mul_right(a.right, b))
}
impl Refinement<'_> {
    fn integer_part(&self, w: usize, negative: bool, x: Expr) -> Result<Equality> {
        let Expr::Apply(head, args) = &x else {
            return Err(GoOrdinaryError::Linkage);
        };
        let label = if negative { "Negative" } else { "Positive" };
        if **head == constant(&format!("{PREFIX}.W{w}.Signed")) {
            let [word] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            // A literal outer sign selects the whole actual word first. Mapping
            // that small equality avoids normalizing a large value recursor.
            let mut target = word.clone();
            let mut prefix = None;
            if let Expr::Apply(select, fields) = word {
                if **select == constant(&format!("Mpk.Go.Practical.Word.W{w}.Select")) {
                    if let [flag, yes, no] = fields.as_slice() {
                        if let Some(flag) = literal_flag(flag) {
                            target = if flag { yes.clone() } else { no.clone() };
                            let e = Equality {
                                ty: word_type(w)?,
                                left: word.clone(),
                                right: target.clone(),
                                proof: refl(word_type(w)?, target.clone()),
                            };
                            prefix = Some(e.mapped(
                                lam(
                                    "_selected_word",
                                    word_type(w)?,
                                    part(negative, signed(w, var("_selected_word"))),
                                ),
                                part(negative, x.clone()),
                                part(negative, signed(w, target.clone())),
                                nat(),
                            ));
                        }
                    }
                }
            }
            let rhs = if negative {
                mul(weight(depth(w)?), bit_value(sign(w, target.clone())?))
            } else {
                value(depth(w)?, target.clone())
            };
            let e = Equality::natural(
                part(negative, signed(w, target.clone())),
                rhs,
                numeric(&format!("W{w}.Signed{label}"), vec![target]),
            );
            return if let Some(prefix) = prefix {
                prefix.then(e)
            } else {
                Ok(e)
            };
        }
        if **head == constant(&format!("{SIGNED_DIVISION}.SignedNat")) {
            let [flag, amount] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let (yes, no) = if negative {
                (amount.clone(), zero())
            } else {
                (zero(), amount.clone())
            };
            return Equality::natural(
                part(negative, x.clone()),
                choose(flag.clone(), yes.clone(), no.clone(), nat()),
                sd(
                    &format!("SignedNat{label}"),
                    vec![flag.clone(), amount.clone()],
                ),
            )
            .then(natural_choice(flag.clone(), yes, no));
        }
        if **head == constant(&format!("{SIGNED_DIVISION}.W{w}.SelectionCorrection")) {
            let [word, flag] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let (yes, no) = if negative {
                (
                    bit_value(wc(w, "DivideDefined", vec![word.clone()])),
                    zero(),
                )
            } else {
                (
                    bit_value(sign(w, wc(w, "Neg", vec![word.clone()]))?),
                    bit_value(sign(w, word.clone())?),
                )
            };
            return Equality::natural(
                part(negative, x.clone()),
                choose(flag.clone(), yes.clone(), no.clone(), nat()),
                sd(
                    &format!("W{w}.SelectionCorrection{label}"),
                    vec![word.clone(), flag.clone()],
                ),
            )
            .then(natural_choice(flag.clone(), yes, no));
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
            let a = self.integer_part(w, negative, a.clone())?;
            let b = self.integer_part(w, negative, b.clone())?;
            return Equality::natural(
                part(negative, x.clone()),
                add(a.left.clone(), b.left.clone()),
                ic(
                    &format!("{label}OfAdd"),
                    vec![args[0].clone(), args[1].clone()],
                ),
            )
            .then(a.clone().on_left(b.left.clone()))?
            .then(b.on_right(a.right));
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
        width: usize,
        left: Expr,
        right: Expr,
    ) -> Result<(Equality, Equality)> {
        let lp = self.integer_part(width, false, left.clone())?;
        let ln = self.integer_part(width, true, left)?;
        let rp = self.integer_part(width, false, right.clone())?;
        let rn = self.integer_part(width, true, right)?;
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
