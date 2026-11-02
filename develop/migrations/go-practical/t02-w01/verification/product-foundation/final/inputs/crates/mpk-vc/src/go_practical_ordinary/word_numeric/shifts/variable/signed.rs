//! Actual signed interpretations: modular left shift and floor right shift.
use super::*;
fn int(n: &str, a: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{n}"), a)
}
fn signed_value(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}
impl Shifts<'_> {
    pub(super) fn signed_shift_balances(&mut self) -> Result<()> {
        self.left_signed_balance()?;
        self.right_signed_balance()
    }
    fn left_signed_balance(&mut self) -> Result<()> {
        let (u, v, m, p, s, t, d) = (
            var("input"),
            var("output"),
            var("modulus"),
            var("power"),
            var("source_bit"),
            var("target_bit"),
            var("discard"),
        );
        let mut args = [
            "input",
            "output",
            "modulus",
            "power",
            "source_bit",
            "target_bit",
            "discard",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let l = add(v.clone(), mul(m.clone(), d.clone()));
        let r = mul(p.clone(), u.clone());
        args.push(("conservation", eq(nat(), l.clone(), r.clone())));
        let prefix = add(
            mul(m.clone(), t.clone()),
            mul(m.clone(), mul(p.clone(), s.clone())),
        );
        let start = Equality::natural(l, r, var("conservation"))
            .reverse()
            .on_left(prefix);
        let target = add(add(v, mul(m.clone(), add(d, t))), mul(p, mul(m, s)));
        let step = self.polynomial(&args, start.right.clone(), target)?;
        self.theorem("LeftSignedBalance", &args, start.then(step)?)
    }
    fn right_signed_balance(&mut self) -> Result<()> {
        let (u, v, m, p, e, b, r) = (
            var("input"),
            var("output"),
            var("modulus"),
            var("power"),
            var("power_minus"),
            var("bit_value"),
            var("remainder"),
        );
        let mut args = [
            "input",
            "output",
            "modulus",
            "power",
            "power_minus",
            "bit_value",
            "remainder",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let lhs = add(u.clone(), mul(mul(m.clone(), e.clone()), b.clone()));
        let rhs = add(mul(p.clone(), v.clone()), r.clone());
        args.extend([
            ("power_balance", eq(nat(), add(e.clone(), one()), p.clone())),
            ("conservation", eq(nat(), lhs.clone(), rhs.clone())),
        ]);
        let padded =
            Equality::natural(lhs, rhs, var("conservation")).on_left(mul(m.clone(), b.clone()));
        let to_power = self
            .polynomial(
                &args,
                padded.left.clone(),
                add(
                    u.clone(),
                    mul(mul(m.clone(), add(e.clone(), one())), b.clone()),
                ),
            )?
            .then(
                mul_left(
                    mul_right(
                        m.clone(),
                        Equality::natural(add(e, one()), p, var("power_balance")),
                    ),
                    b,
                )
                .on_right(u),
            )?;
        self.theorem(
            "RightSignedBalance",
            &args,
            to_power.reverse().then(padded)?,
        )
    }
    pub(super) fn signed_shift_views(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, c) = (var("x"), var("count"));
        let args = [("x", word_type(w)?), ("count", word_type(64)?)];
        let m = weight(d);
        let p = power(value(6, c.clone()));
        let u = value(d, x.clone());
        let sx = bit_value(at(x.clone(), w, w - 1)?);
        let out = wc(w, "ShiftLeft", vec![x.clone(), c.clone()]);
        let v = value(d, out.clone());
        let sy = bit_value(at(out.clone(), w, w - 1)?);
        let discard = full_discard(w, x.clone(), c.clone());
        let desired = int(
            "Mul",
            vec![int("FromNat", vec![p.clone()]), signed_value(w, x.clone())],
        );
        let corrected = int(
            "Add",
            vec![
                signed_value(w, out.clone()),
                int(
                    "Mul",
                    vec![
                        int("FromNat", vec![m.clone()]),
                        int(
                            "Make",
                            vec![add(discard.clone(), sy.clone()), mul(p.clone(), sx.clone())],
                        ),
                    ],
                ),
            ],
        );
        let (left, right) =
            super::super::super::conversions::projection_cross(desired.clone(), corrected.clone())?;
        let target_left = add(
            mul(p.clone(), u.clone()),
            add(
                mul(m.clone(), sy.clone()),
                mul(m.clone(), mul(p.clone(), sx.clone())),
            ),
        );
        let target_right = add(
            add(v.clone(), mul(m.clone(), add(discard.clone(), sy.clone()))),
            mul(p.clone(), mul(m.clone(), sx.clone())),
        );
        let left_norm = self.polynomial(&args, left.right.clone(), target_left.clone())?;
        let right_norm = self.polynomial(&args, right.right.clone(), target_right.clone())?;
        let balance = known(
            "LeftSignedBalance",
            vec![
                u.clone(),
                v,
                m.clone(),
                p.clone(),
                sx.clone(),
                sy,
                discard,
                sh(
                    &format!("W{w}.UnsignedLeftConservation"),
                    vec![x.clone(), c.clone()],
                ),
            ],
            target_left,
            target_right,
        );
        let proof = left
            .then(left_norm)?
            .then(balance)?
            .then(right_norm.reverse())?
            .then(right.reverse())?;
        self.emit(
            &format!("W{w}.SignedLeftRefinement"),
            &args,
            int("Equivalent", vec![desired, corrected]),
            proof.proof,
            true,
        )?;
        let out = wc(w, "SignedShiftRight", vec![x.clone(), c.clone()]);
        let v = value(d, out.clone());
        let sy = bit_value(at(out.clone(), w, w - 1)?);
        let rem = sh(
            &format!("W{w}.SignedRightRemainder"),
            vec![x.clone(), c.clone()],
        );
        let e = minus(value(6, c.clone()));
        let source = signed_value(w, x.clone());
        let result = int(
            "Add",
            vec![
                int(
                    "Mul",
                    vec![
                        int("FromNat", vec![p.clone()]),
                        signed_value(w, out.clone()),
                    ],
                ),
                int("FromNat", vec![rem.clone()]),
            ],
        );
        let (left, right) =
            super::super::super::conversions::projection_cross(source.clone(), result.clone())?;
        let target_left = add(u.clone(), mul(mul(m.clone(), p.clone()), sy.clone()));
        let target_left_source = add(u.clone(), mul(mul(m.clone(), p.clone()), sx.clone()));
        let target_right = add(
            add(mul(p.clone(), v.clone()), rem.clone()),
            mul(m.clone(), sx.clone()),
        );
        let left_norm = self.polynomial(&args, left.right.clone(), target_left)?;
        let right_norm = self.polynomial(&args, right.right.clone(), target_right.clone())?;
        let top = map_bit(Equality::boolean(
            at(out, w, w - 1)?,
            at(x.clone(), w, w - 1)?,
            sh(&format!("W{w}.SignedRightTop"), vec![x.clone(), c.clone()]),
        ));
        let bridge = mul_right(mul(m.clone(), p.clone()), top).on_right(u.clone());
        let balance = known(
            "RightSignedBalance",
            vec![
                u,
                v,
                m,
                p,
                e,
                sx,
                rem,
                sh("PowerMinusBalance", vec![value(6, c.clone())]),
                sh(&format!("W{w}.SignedRightFilledConservation"), vec![x, c]),
            ],
            target_left_source,
            target_right,
        );
        let proof = left
            .then(left_norm)?
            .then(bridge)?
            .then(balance)?
            .then(right_norm.reverse())?
            .then(right.reverse())?;
        self.emit(
            &format!("W{w}.SignedRightFloorConservation"),
            &args,
            int("Equivalent", vec![source, result]),
            proof.proof,
            true,
        )
    }
}
