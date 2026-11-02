//! Full actual shift/add multiplication with a computed discarded-value witness.
use super::digits::digits;
use super::shift::{discarded, left};
use super::*;

fn witness(width: usize, i: usize, x: Expr, y: Expr) -> Expr {
    numeric(&format!("W{width}.MulDiscard{i}"), vec![x, y])
}

impl Numeric<'_> {
    pub(super) fn product_combine(&mut self) -> Result<()> {
        let (x, d, p, b, a, l, s, n, m, q, t, c) = (
            var("x"),
            var("digits"),
            var("power"),
            var("bit"),
            var("acc"),
            var("part"),
            var("selected"),
            var("next"),
            var("modulus"),
            var("quotient"),
            var("discarded"),
            var("carry"),
        );
        let old_l = mul(x.clone(), d.clone());
        let old_r = add(a.clone(), mul(m.clone(), q.clone()));
        let shift_l = add(l.clone(), mul(m.clone(), t.clone()));
        let shift_r = mul(p.clone(), x.clone());
        let select_r = mul(b.clone(), l.clone());
        let add_l = add(a.clone(), s.clone());
        let add_r = add(n.clone(), mul(m.clone(), c.clone()));
        let mut args: Vec<Binder> = [
            "x",
            "digits",
            "power",
            "bit",
            "acc",
            "part",
            "selected",
            "next",
            "modulus",
            "quotient",
            "discarded",
            "carry",
        ]
        .into_iter()
        .map(|n| (n, nat()))
        .collect();
        args.extend([
            ("old", eq(nat(), old_l.clone(), old_r.clone())),
            ("shift", eq(nat(), shift_l.clone(), shift_r.clone())),
            ("select", eq(nat(), s.clone(), select_r.clone())),
            ("addition", eq(nat(), add_l.clone(), add_r.clone())),
        ]);
        let lhs = mul(x, add(d, mul(p, b.clone())));
        let e = self
            .polynomial(
                &args,
                lhs,
                add(old_l.clone(), mul(b.clone(), shift_r.clone())),
            )?
            .then(
                Equality::natural(old_l, old_r.clone(), var("old"))
                    .on_left(mul(b.clone(), shift_r.clone())),
            )?
            .then(
                mul_right(
                    b.clone(),
                    Equality::natural(shift_l, shift_r, var("shift")).reverse(),
                )
                .on_right(old_r),
            )?;
        let rest = mul(m.clone(), add(q.clone(), mul(b.clone(), t.clone())));
        let next = self.polynomial(
            &args,
            e.right.clone(),
            add(add(a.clone(), select_r.clone()), rest.clone()),
        )?;
        let e = e
            .then(next)?
            .then(
                Equality::natural(s, select_r, var("select"))
                    .reverse()
                    .on_right(a)
                    .on_left(rest.clone()),
            )?
            .then(Equality::natural(add_l, add_r, var("addition")).on_left(rest))?;
        let rhs = add(n, mul(m, add(add(q, mul(b, t)), c)));
        let next = self.polynomial(&args, e.right.clone(), rhs)?;
        self.theorem("ProductCombine", &args, e.then(next)?)
    }

    pub(super) fn product_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(width)?), ("y", word_type(width)?)];
        let vx = value(d, x.clone());
        let z = wc(width, "Zero", vec![]);
        self.emit(
            &format!("W{width}.MulDiscard0"),
            &args,
            nat(),
            zero(),
            false,
        )?;
        let lhs = mul_right(
            vx.clone(),
            Equality::natural(digits(width, 0, y.clone()), zero(), refl(nat(), zero())),
        )
        .then(Equality::natural(
            mul(vx.clone(), zero()),
            zero(),
            nc("MulZero", vec![vx.clone()]),
        ))?;
        let acc = wc(width, "Mul0", vec![x.clone(), y.clone()]);
        let word = Equality {
            ty: word_type(width)?,
            left: acc,
            right: z.clone(),
            proof: refl(word_type(width)?, z.clone()),
        };
        let rhs = map_value(d, word)
            .then(instance(
                &format!("W{width}.ZeroValue"),
                vec![],
                value(d, z),
                zero(),
            ))?
            .on_left(mul(weight(d), witness(width, 0, x.clone(), y.clone())))
            .then(
                mul_right(
                    weight(d),
                    Equality::natural(
                        witness(width, 0, x.clone(), y.clone()),
                        zero(),
                        refl(nat(), zero()),
                    ),
                )
                .on_right(zero()),
            )?
            .then(
                Equality::natural(
                    mul(weight(d), zero()),
                    zero(),
                    nc("MulZero", vec![weight(d)]),
                )
                .on_right(zero()),
            )?
            .then(Equality::natural(
                add(zero(), zero()),
                zero(),
                nc("ZeroAdd", vec![zero()]),
            ))?;
        self.theorem(
            &format!("W{width}.MulConservation0"),
            &args,
            lhs.then(rhs.reverse())?,
        )?;
        for i in 0..width {
            let old = wc(width, &format!("Mul{i}"), vec![x.clone(), y.clone()]);
            let part = left(width, i, x.clone());
            let flag = at(y.clone(), width, i)?;
            let bv = bit_value(flag.clone());
            let selected = wc(
                width,
                "Select",
                vec![flag.clone(), part.clone(), wc(width, "Zero", vec![])],
            );
            let sum = wc(width, "Add", vec![old.clone(), selected.clone()]);
            let next = wc(width, &format!("Mul{}", i + 1), vec![x.clone(), y.clone()]);
            let carry = bit_value(wc(
                width,
                &format!("Carry{width}"),
                vec![old.clone(), selected.clone(), bit(false)],
            ));
            let q = witness(width, i, x.clone(), y.clone());
            let shift = discarded(width, i, x.clone());
            let qbody = add(
                add(q.clone(), mul(bv.clone(), shift.clone())),
                carry.clone(),
            );
            self.emit(
                &format!("W{width}.MulDiscard{}", i + 1),
                &args,
                nat(),
                qbody.clone(),
                false,
            )?;
            let dpbody = add(digits(width, i, y.clone()), mul(coefficient(i), bv.clone()));
            let start = mul_right(
                vx.clone(),
                Equality::natural(
                    digits(width, i + 1, y.clone()),
                    dpbody.clone(),
                    refl(nat(), dpbody),
                ),
            );
            let proof = numeric(
                "ProductCombine",
                vec![
                    vx.clone(),
                    digits(width, i, y.clone()),
                    coefficient(i),
                    bv,
                    value(d, old.clone()),
                    value(d, part.clone()),
                    value(d, selected.clone()),
                    value(d, sum.clone()),
                    weight(d),
                    q,
                    shift,
                    carry,
                    numeric(
                        &format!("W{width}.MulConservation{i}"),
                        vec![x.clone(), y.clone()],
                    ),
                    unary(width, &format!("ShiftConservation{i}"), x.clone()),
                    numeric(&format!("W{width}.SelectZeroValue"), vec![part, flag]),
                    numeric(
                        &format!("W{width}.AddValueConservation"),
                        vec![old, selected],
                    ),
                ],
            );
            let end = add(value(d, sum.clone()), mul(weight(d), qbody.clone()));
            let first = start.right.clone();
            let e = start.then(Equality::natural(first, end, proof))?;
            let word = Equality {
                ty: word_type(width)?,
                left: next.clone(),
                right: sum.clone(),
                proof: refl(word_type(width)?, sum),
            };
            let e = e
                .then(
                    map_value(d, word)
                        .reverse()
                        .on_left(mul(weight(d), qbody.clone())),
                )?
                .then(
                    mul_right(
                        weight(d),
                        Equality::natural(
                            witness(width, i + 1, x.clone(), y.clone()),
                            qbody.clone(),
                            refl(nat(), qbody),
                        )
                        .reverse(),
                    )
                    .on_right(value(d, next)),
                )?;
            self.theorem(&format!("W{width}.MulConservation{}", i + 1), &args, e)?;
        }
        let digits = instance(
            &format!("W{width}.DigitsValue"),
            vec![y.clone()],
            digits(width, width, y.clone()),
            value(d, y.clone()),
        );
        let last = wc(width, &format!("Mul{width}"), vec![x.clone(), y.clone()]);
        let result = wc(width, "Mul", vec![x.clone(), y.clone()]);
        let q = witness(width, width, x.clone(), y.clone());
        let first = mul_right(vx, digits).reverse();
        let stage = instance(
            &format!("W{width}.MulConservation{width}"),
            vec![x.clone(), y.clone()],
            first.right.clone(),
            add(value(d, last.clone()), mul(weight(d), q.clone())),
        );
        let word = Equality {
            ty: word_type(width)?,
            left: result,
            right: last.clone(),
            proof: refl(word_type(width)?, last),
        };
        let e = first
            .then(stage)?
            .then(map_value(d, word).reverse().on_left(mul(weight(d), q)))?;
        self.theorem(&format!("W{width}.MulValueConservation"), &args, e.clone())?;
        let e = Equality {
            proof: numeric(&format!("W{width}.MulValueConservation"), vec![x, y]),
            ..e
        }
        .on_right(var("initial"));
        self.theorem(
            &format!("W{width}.MultiplicationConservation"),
            &[
                ("x", word_type(width)?),
                ("y", word_type(width)?),
                ("initial", nat()),
            ],
            e,
        )
    }
}
