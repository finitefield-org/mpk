//! Conditional signed conservation and its actual balanced-Int interpretation.
//! The premise is equality of the actual sign-stage carries, with a separately
//! checked reflection from the computed NoOverflow Boolean.
use super::*;

fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn part(negative: bool, x: Expr) -> Expr {
    ic(if negative { "Negative" } else { "Positive" }, vec![x])
}
fn sign(width: usize, x: Expr) -> Result<Expr> {
    Ok(bit_value(at(x, width, width - 1)?))
}

impl Numeric<'_> {
    pub(super) fn sign_addition(&mut self) -> Result<()> {
        let (a, b, c) = (var("a"), var("b"), var("c"));
        let out = bc("Carry", vec![a.clone(), b.clone(), c.clone()]);
        let sum = bc("Sum", vec![a.clone(), b.clone(), c.clone()]);
        let args = [
            ("a", boolean()),
            ("b", boolean()),
            ("c", boolean()),
            ("guard", eq(boolean(), c.clone(), out.clone())),
        ];
        let ab = add(bit_value(a.clone()), bit_value(b.clone()));
        let carry = bit_value(out.clone());
        let sum = bit_value(sum);
        let rewrite = map_bit(Equality::boolean(c.clone(), out, var("guard")))
            .on_right(ab.clone())
            .reverse();
        let (left, right) = bit_addition(a.clone(), b.clone(), c.clone());
        let p = rewrite.then(Equality::natural(
            left,
            right,
            numeric("BitAddition", vec![a, b, c]),
        ))?;
        let cancelled_right = add(sum.clone(), carry.clone());
        let regrouped = add(cancelled_right.clone(), carry.clone());
        let next = self.polynomial(&args, p.right.clone(), regrouped)?;
        let p = p.then(next)?;
        let cancelled = Equality::natural(
            ab.clone(),
            cancelled_right,
            nc(
                "AddCancel",
                vec![
                    ab.clone(),
                    add(sum.clone(), carry.clone()),
                    carry.clone(),
                    p.proof,
                ],
            ),
        );
        let order = Equality::natural(
            add(carry.clone(), sum.clone()),
            add(sum.clone(), carry.clone()),
            nc("AddComm", vec![carry, sum]),
        );
        self.theorem("SignAddition", &args, order.then(cancelled.reverse())?)
    }

    pub(super) fn signed_combine(&mut self) -> Result<()> {
        let (x, y, r, c, w, carry) = (
            var("ux"),
            var("uy"),
            var("ur"),
            var("incoming"),
            var("weight"),
            var("carry"),
        );
        let (sx, sy, sr) = (var("sx"), var("sy"), var("sr"));
        let sum = add(add(x, y), c);
        let unsigned_right = add(r.clone(), mul(w.clone(), carry.clone()));
        let sign_left = add(carry, sr.clone());
        let sign_right = add(sx, sy);
        let mut args: Vec<Binder> = [
            "ux", "uy", "ur", "incoming", "weight", "carry", "sx", "sy", "sr",
        ]
        .into_iter()
        .map(|n| (n, nat()))
        .collect();
        args.extend([
            ("unsigned", eq(nat(), sum.clone(), unsigned_right.clone())),
            ("sign", eq(nat(), sign_left.clone(), sign_right.clone())),
        ]);
        let p = Equality::natural(sum, unsigned_right, var("unsigned")).on_left(mul(w.clone(), sr));
        let regrouped = add(r.clone(), mul(w.clone(), sign_left.clone()));
        let next = self.polynomial(&args, p.right.clone(), regrouped)?;
        let p = p.then(next)?.then(
            mul_right(w, Equality::natural(sign_left, sign_right, var("sign"))).on_right(r),
        )?;
        self.theorem("SignedCombine", &args, p)
    }

    fn integer_part(&self, width: usize, negative: bool, x: Expr) -> Result<Equality> {
        let Expr::Apply(head, arguments) = &x else {
            return Err(GoOrdinaryError::Linkage);
        };
        let label = if negative { "Negative" } else { "Positive" };
        if **head == constant("Mpk.Go.Practical.Int.Add") {
            let [a, b] = arguments.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let pa = self.integer_part(width, negative, a.clone())?;
            let pb = self.integer_part(width, negative, b.clone())?;
            let p = Equality::natural(
                part(negative, x.clone()),
                add(part(negative, a.clone()), part(negative, b.clone())),
                ic(&format!("{label}OfAdd"), vec![a.clone(), b.clone()]),
            );
            return p
                .then(pa.clone().on_left(part(negative, b.clone())))?
                .then(pb.on_right(pa.right));
        }
        if **head == constant("Mpk.Go.Practical.Int.FromNat") {
            let [n] = arguments.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                if negative { zero() } else { n.clone() },
                ic(&format!("{label}OfFromNat"), vec![n.clone()]),
            ));
        }
        if **head == constant(&format!("{PREFIX}.W{width}.Signed")) {
            let [word] = arguments.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            let result = if negative {
                mul(weight(depth(width)?), sign(width, word.clone())?)
            } else {
                value(depth(width)?, word.clone())
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                result,
                numeric(&format!("W{width}.Signed{label}"), vec![word.clone()]),
            ));
        }
        Err(GoOrdinaryError::Linkage)
    }

    pub(super) fn signed_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let name = |suffix: &str| format!("W{width}.{suffix}");
        let (x, y, incoming) = (var("x"), var("y"), var("incoming"));
        let result = wc(
            width,
            "AddCarry",
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let carry = |i| {
            wc(
                width,
                &format!("Carry{i}"),
                vec![x.clone(), y.clone(), incoming.clone()],
            )
        };
        let (ci, co) = (carry(width - 1), carry(width));
        let (a, b) = (
            at(x.clone(), width, width - 1)?,
            at(y.clone(), width, width - 1)?,
        );
        let sum = bc("Sum", vec![a.clone(), b.clone(), ci.clone()]);
        let out = bc("Carry", vec![a.clone(), b.clone(), ci.clone()]);
        let step = Equality::boolean(
            co.clone(),
            out.clone(),
            numeric(
                &name(&format!("CarryStep{}", width - 1)),
                vec![x.clone(), y.clone(), incoming.clone()],
            ),
        );
        let projected = Equality::boolean(
            at(result.clone(), width, width - 1)?,
            sum.clone(),
            numeric(
                &name(&format!("SumBit{}", width - 1)),
                vec![x.clone(), y.clone(), incoming.clone()],
            ),
        );
        let mut args = vec![
            ("x", word_type(width)?),
            ("y", word_type(width)?),
            ("incoming", boolean()),
        ];
        let guard = eq(boolean(), ci.clone(), co.clone());
        args.push(("guard", guard.clone()));
        let actual_guard =
            Equality::boolean(ci.clone(), co.clone(), var("guard")).then(step.clone())?;
        let actual_sign = map_bit(step.reverse())
            .on_left(bit_value(sum.clone()))
            .then(map_bit(projected.reverse()).on_right(bit_value(co.clone())))?
            .reverse()
            .then(Equality::natural(
                add(bit_value(out), bit_value(sum)),
                add(bit_value(a.clone()), bit_value(b.clone())),
                numeric("SignAddition", vec![a, b, ci.clone(), actual_guard.proof]),
            ))?;
        let unsigned = numeric(
            &format!("W{width}.Block{d}At0"),
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let proof = numeric(
            "SignedCombine",
            vec![
                value(d, x.clone()),
                value(d, y.clone()),
                value(d, result.clone()),
                bit_value(incoming.clone()),
                weight(d),
                bit_value(co.clone()),
                sign(width, x.clone())?,
                sign(width, y.clone())?,
                sign(width, result.clone())?,
                unsigned,
                actual_sign.proof,
            ],
        );
        let left = add(
            add(
                add(value(d, x.clone()), value(d, y.clone())),
                bit_value(incoming.clone()),
            ),
            mul(weight(d), sign(width, result.clone())?),
        );
        let right = add(
            value(d, result.clone()),
            mul(
                weight(d),
                add(sign(width, x.clone())?, sign(width, y.clone())?),
            ),
        );
        let cross = Equality::natural(left, right, proof);
        self.theorem(&name("SignedCrossSum"), &args, cross.clone())?;
        let mut with_initial = args[..3].to_vec();
        with_initial.extend([("initial", nat()), ("guard", guard)]);
        self.theorem(
            &name("SignedAdditionConservation"),
            &with_initial,
            Equality {
                proof: numeric(
                    &name("SignedCrossSum"),
                    vec![x.clone(), y.clone(), incoming.clone(), var("guard")],
                ),
                ..cross.clone()
            }
            .on_right(var("initial")),
        )?;
        let no_overflow = numeric(
            &name("NoOverflow"),
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let mut boolean_guard = args[..3].to_vec();
        boolean_guard.push(("valid", eq(boolean(), no_overflow, bit(true))));
        self.theorem(
            &name("NoOverflowReflect"),
            &boolean_guard,
            Equality::boolean(
                ci.clone(),
                co.clone(),
                bc("EqualReflect", vec![ci, co, var("valid")]),
            ),
        )?;
        for negative in [false, true] {
            let signed = numeric(&name("Signed"), vec![x.clone()]);
            let expanded = if negative {
                mul(weight(d), sign(width, x.clone())?)
            } else {
                value(d, x.clone())
            };
            self.theorem(
                &name(if negative {
                    "SignedNegative"
                } else {
                    "SignedPositive"
                }),
                &[("x", word_type(width)?)],
                Equality::natural(
                    part(negative, signed),
                    expanded.clone(),
                    refl(nat(), expanded),
                ),
            )?;
        }
        let signed = |word| numeric(&name("Signed"), vec![word]);
        let left_integer = ic(
            "Add",
            vec![
                ic("Add", vec![signed(x.clone()), signed(y.clone())]),
                ic("FromNat", vec![bit_value(incoming.clone())]),
            ],
        );
        let right_integer = signed(result.clone());
        let lp = self.integer_part(width, false, left_integer.clone())?;
        let rn = self.integer_part(width, true, right_integer.clone())?;
        let rp = self.integer_part(width, false, right_integer.clone())?;
        let ln = self.integer_part(width, true, left_integer.clone())?;
        let lhs = lp
            .clone()
            .on_left(rn.left.clone())
            .then(rn.on_right(lp.right))?;
        let rhs = rp
            .clone()
            .on_left(ln.left.clone())
            .then(ln.on_right(rp.right))?;
        let ux = mul(weight(d), sign(width, x.clone())?);
        let uy = mul(weight(d), sign(width, y.clone())?);
        let distribute = Equality::natural(
            mul(
                weight(d),
                add(sign(width, x.clone())?, sign(width, y.clone())?),
            ),
            add(ux.clone(), uy.clone()),
            nc(
                "MulAddRight",
                vec![weight(d), sign(width, x.clone())?, sign(width, y.clone())?],
            ),
        );
        let padded = Equality::natural(
            add(add(ux.clone(), uy.clone()), zero()),
            add(ux, uy),
            nc(
                "AddZero",
                vec![add(
                    mul(weight(d), sign(width, x.clone())?),
                    mul(weight(d), sign(width, y.clone())?),
                )],
            ),
        )
        .reverse();
        let reflected = numeric(
            &name("NoOverflowReflect"),
            vec![x.clone(), y.clone(), incoming.clone(), var("valid")],
        );
        let cross = Equality {
            proof: numeric(&name("SignedCrossSum"), vec![x, y, incoming, reflected]),
            ..cross
        };
        let p = lhs
            .then(cross)?
            .then(distribute.on_right(value(d, result.clone())))?
            .then(padded.on_right(value(d, result)))?
            .then(rhs.reverse())?;
        self.emit(
            &name("SignedAddition"),
            &boolean_guard,
            ic("Equivalent", vec![left_integer, right_integer]),
            p.proof,
            true,
        )
    }
}
