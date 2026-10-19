//! Ordinary value congruence from all literal-address bit equalities. A single
//! generic address-elimination proof is shared by every concrete fixed shift.
use super::*;

impl Numeric<'_> {
    pub(super) fn literal_value_laws(&mut self) -> Result<()> {
        for width in WIDTHS {
            let d = depth(width)?;
            let (x, y) = (var("left_word"), var("right_word"));
            let names = &ADDRESSES[..d];
            let prop = |e: &BTreeMap<&'static str, Expr>| {
                let a = names.iter().map(|n| e[n].clone()).collect::<Vec<_>>();
                eq(boolean(), app(x.clone(), a.clone()), app(y.clone(), a))
            };
            let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let mut index = 0;
                for name in names {
                    index = 2 * index + usize::from(e[name] == bit(true));
                }
                Ok(var(&format!("_literal_equality{index}")))
            };
            let pointwise = boolean_cases(names, &BTreeMap::new(), &prop, &mut leaf)?;
            let mut proof = numeric(
                &format!("ValueCongruence{d}"),
                vec![x.clone(), y.clone(), pointwise],
            );
            let mut ty = eq(nat(), value(d, x.clone()), value(d, y.clone()));
            for i in (0..width).rev() {
                let name = format!("_literal_equality{i}");
                let p = eq(
                    boolean(),
                    at(x.clone(), width, i)?,
                    at(y.clone(), width, i)?,
                );
                ty = pi(&name, p.clone(), ty);
                proof = lam(&name, p, proof);
            }
            self.emit(
                &format!("LiteralValueCongruence{width}"),
                &[
                    ("left_word", word_type(width)?),
                    ("right_word", word_type(width)?),
                ],
                ty,
                proof,
                true,
            )?;
        }
        Ok(())
    }

    pub(super) fn literal_values(&self, width: usize, from: Expr, to: Expr) -> Result<Equality> {
        let d = depth(width)?;
        let mut arguments = vec![from.clone(), to.clone()];
        for i in 0..width {
            arguments.push(refl(boolean(), at(to.clone(), width, i)?));
        }
        Ok(instance(
            &format!("LiteralValueCongruence{width}"),
            arguments,
            value(d, from),
            value(d, to),
        ))
    }
}
