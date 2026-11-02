//! Complete frozen concatenation: capacity is a profile premise, never a panic.
use super::*;
fn cc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Concat.{name}"), args)
}
fn cap_word() -> Expr {
    cc("CapacityWord", vec![])
}
fn numeral(n: usize) -> Expr {
    (0..n).fold(zero(), |n, _| succ(n))
}
fn coefficient(n: usize) -> Expr {
    numeric("PowerTwo", vec![numeral(n)])
}
fn sum(s: Expr, t: Expr) -> Expr {
    add(size(s), size(t))
}
fn fits(s: Expr, t: Expr) -> Expr {
    cc("Fits", vec![s, t])
}
fn bound(s: Expr, t: Expr, h: Expr) -> Expr {
    cc("FitBound", vec![s, t, h])
}
fn length_word(s: Expr, t: Expr) -> Expr {
    cc("LengthWord", vec![s, t])
}
fn carry(s: Expr, t: Expr) -> Expr {
    cc("Carry", vec![s, t])
}

fn high_value(lo: usize, hi: usize) -> Result<Equality> {
    if hi == lo + 1 {
        let yes = lo == 8;
        let weight = coefficient(lo - 8);
        let flag = word::at(cap_word(), 64, lo)?;
        let e = mul_right(
            weight.clone(),
            bit_value(Equality::boolean(flag, bit(yes), refl(boolean(), bit(yes)))),
        )
        .then(mul_right(
            weight.clone(),
            Equality::natural(
                numeric("Bit", vec![bit(yes)]),
                if yes { one() } else { zero() },
                refl(nat(), if yes { one() } else { zero() }),
            ),
        ))?
        .then(Equality::natural(
            mul(weight.clone(), if yes { one() } else { zero() }),
            if yes { weight.clone() } else { zero() },
            nc(if yes { "MulOne" } else { "MulZero" }, vec![weight.clone()]),
        ))?;
        return if yes {
            e.then(Equality::natural(
                weight,
                one(),
                numeric("PowerTwoZero", vec![]),
            ))
        } else {
            Ok(e)
        };
    }
    let mid = lo + (hi - lo) / 2;
    let left = high_value(lo, mid)?;
    let right = high_value(mid, hi)?;
    let e = left
        .on_left(right.left.clone())
        .then(right.on_right(if lo == 8 { one() } else { zero() }))?;
    let target = if lo == 8 { one() } else { zero() };
    e.then(Equality::natural(
        e_end(lo),
        target.clone(),
        nc("AddZero", vec![target]),
    ))
}
fn e_end(lo: usize) -> Expr {
    add(if lo == 8 { one() } else { zero() }, zero())
}

struct Concat<'a>(&'a mut Builder);
impl Concat<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Concat.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn capacity(&mut self) -> Result<()> {
        self.emit("CapacityWord", &[], word64(), encoded_word(64, 256), false)?;
        let w = cap_word();
        let low = (0..8)
            .rev()
            .map(|i| word::at(w.clone(), 64, i))
            .collect::<Result<Vec<_>>>()?;
        let mut e = Equality::natural(
            numeric("W64.Digits8", vec![w.clone()]),
            index(low.clone()),
            ix("DigitsIndex", vec![w.clone()]),
        );
        let mut current = low;
        for i in 0..8 {
            let mut template = current.clone();
            template[i] = var("_bit");
            let mut next = current.clone();
            next[i] = bit(false);
            e = e.then(
                Equality::boolean(current[i].clone(), bit(false), refl(boolean(), bit(false)))
                    .mapped(
                        lam("_bit", boolean(), index(template)),
                        index(current),
                        index(next.clone()),
                        nat(),
                    ),
            )?;
            current = next;
        }
        e = e.then(Equality::natural(
            index(vec![bit(false); 8]),
            zero(),
            call("Mpk.Go.Practical.String.Validity.IndexZero8", vec![]),
        ))?;
        self.theorem("CapacityDigits", &[], e)?;
        let high = high_value(8, 64)?;
        self.theorem(
            "CapacityHighValue",
            &[],
            Equality::natural(
                call(
                    "Mpk.Go.Practical.Word.Shifts.CountHighValue8",
                    vec![w.clone()],
                ),
                high.left.clone(),
                refl(nat(), high.left.clone()),
            )
            .then(high)?,
        )?;
        let low = numeric("W64.Digits8", vec![w.clone()]);
        let hv = call(
            "Mpk.Go.Practical.Word.Shifts.CountHighValue8",
            vec![w.clone()],
        );
        let e = Equality::natural(
            value(w.clone()),
            add(low.clone(), mul(coefficient(8), hv.clone())),
            call(
                "Mpk.Go.Practical.Word.Shifts.CountBalance8",
                vec![w.clone()],
            ),
        )
        .then(
            Equality::natural(low, zero(), cc("CapacityDigits", vec![]))
                .on_left(mul(coefficient(8), hv.clone())),
        )?
        .then(
            mul_right(
                coefficient(8),
                Equality::natural(hv, one(), cc("CapacityHighValue", vec![])),
            )
            .on_right(zero()),
        )?
        .then(
            Equality::natural(
                mul(coefficient(8), one()),
                coefficient(8),
                nc("MulOne", vec![coefficient(8)]),
            )
            .on_right(zero()),
        )?
        .then(Equality::natural(
            add(zero(), coefficient(8)),
            coefficient(8),
            nc("ZeroAdd", vec![coefficient(8)]),
        ))?
        .then(Equality::natural(
            coefficient(8),
            capacity(),
            ix("CapacityPower", vec![]),
        ))?;
        self.theorem("CapacityValue", &[], e.clone())?;
        self.theorem(
            "CapacityBelowModulus",
            &[],
            nl(e.reverse(), modulus()).then(Equality::boolean(
                less(value(w.clone()), modulus()),
                bit(true),
                ord("W64.ValueBound", vec![w]),
            ))?,
        )
    }
    fn domain(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let args = [("s", string_type()), ("t", string_type())];
        let total = sum(s.clone(), t.clone());
        let flag = less(capacity(), total.clone());
        let body = bc("Not", vec![flag.clone()]);
        self.emit("Fits", &args, boolean(), body.clone(), false)?;
        self.theorem(
            "FitsStep",
            &args,
            Equality::boolean(
                fits(s.clone(), t.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        let mut guarded = args.to_vec();
        guarded.push(("fit", truth(fits(s.clone(), t.clone()), true)));
        let h = Equality::boolean(
            fits(s.clone(), t.clone()),
            body,
            cc("FitsStep", vec![s.clone(), t.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            fits(s.clone(), t.clone()),
            bit(true),
            var("fit"),
        ))?;
        self.theorem(
            "FitBound",
            &guarded,
            Equality::boolean(
                flag.clone(),
                bit(false),
                sl("NotTrueFalse", vec![flag.clone(), h.proof]),
            ),
        )?;
        let mut complete = args.to_vec();
        complete.push(("bound", truth(flag.clone(), false)));
        self.theorem(
            "FitsComplete",
            &complete,
            Equality::boolean(
                fits(s.clone(), t.clone()),
                bc("Not", vec![flag.clone()]),
                cc("FitsStep", vec![s.clone(), t.clone()]),
            )
            .then(guards::not_false(Equality::boolean(
                flag,
                bit(false),
                var("bound"),
            ))?)?,
        )?;
        let (n, c, m) = (var("n"), var("c"), var("m"));
        let target = truth(less(n.clone(), m.clone()), true);
        let absurd = Equality::boolean(less(c.clone(), n.clone()), bit(false), var("upper"))
            .reverse()
            .then(Equality::boolean(
                less(c.clone(), n.clone()),
                bit(true),
                call(
                    "Mpk.Go.Practical.Word.Index.UpperBound",
                    vec![
                        n.clone(),
                        m.clone(),
                        c.clone(),
                        var("_branch"),
                        var("small"),
                    ],
                ),
            ))?;
        self.theorem(
            "CappedSmall",
            &[
                ("n", nat()),
                ("c", nat()),
                ("m", nat()),
                ("upper", truth(less(c.clone(), n.clone()), false)),
                ("small", truth(less(c.clone(), m.clone()), true)),
            ],
            Equality::boolean(
                less(n.clone(), m.clone()),
                bit(true),
                decide(
                    less(n, m),
                    target.clone(),
                    bc("FalseElim", vec![target, absurd.proof]),
                    var("_branch"),
                ),
            ),
        )?;
        self.theorem(
            "SumSmall",
            &guarded,
            Equality::boolean(
                less(total.clone(), modulus()),
                bit(true),
                cc(
                    "CappedSmall",
                    vec![
                        total.clone(),
                        capacity(),
                        modulus(),
                        bound(s.clone(), t.clone(), var("fit")),
                        cc("CapacityBelowModulus", vec![]),
                    ],
                ),
            ),
        )?;
        let actual_carry = wc(
            "Carry64",
            vec![length(s.clone()), length(t.clone()), bit(false)],
        );
        self.emit("Carry", &args, boolean(), actual_carry.clone(), false)?;
        self.theorem(
            "CarryStep",
            &args,
            Equality::boolean(
                carry(s.clone(), t.clone()),
                actual_carry.clone(),
                refl(boolean(), actual_carry.clone()),
            ),
        )?;
        self.theorem(
            "CarryClear",
            &guarded,
            Equality::boolean(
                carry(s.clone(), t.clone()),
                actual_carry,
                cc("CarryStep", vec![s.clone(), t.clone()]),
            )
            .then(Equality::boolean(
                wc(
                    "Carry64",
                    vec![length(s.clone()), length(t.clone()), bit(false)],
                ),
                bit(false),
                sl(
                    "AddCarryClear",
                    vec![
                        length(s.clone()),
                        length(t.clone()),
                        cc("SumSmall", vec![s.clone(), t.clone(), var("fit")]),
                    ],
                ),
            ))?,
        )?;
        let raw = wc("Add", vec![length(s.clone()), length(t.clone())]);
        self.emit("LengthWord", &args, word64(), raw.clone(), false)?;
        self.theorem(
            "LengthWordStep",
            &args,
            Equality {
                ty: word64(),
                left: length_word(s.clone(), t.clone()),
                right: raw.clone(),
                proof: refl(word64(), raw.clone()),
            },
        )?;
        self.theorem(
            "LengthValue",
            &guarded,
            Equality {
                ty: word64(),
                left: length_word(s.clone(), t.clone()),
                right: raw.clone(),
                proof: cc("LengthWordStep", vec![s.clone(), t.clone()]),
            }
            .mapped(
                lam("_word", word64(), value(var("_word"))),
                value(length_word(s.clone(), t.clone())),
                value(raw.clone()),
                nat(),
            )
            .then(Equality::natural(
                value(raw),
                total.clone(),
                sl(
                    "AddValue",
                    vec![
                        length(s.clone()),
                        length(t.clone()),
                        cc("SumSmall", vec![s.clone(), t.clone(), var("fit")]),
                    ],
                ),
            ))?,
        )?;
        for right in [false, true] {
            let a = if right {
                size(t.clone())
            } else {
                size(s.clone())
            };
            let b = if right {
                size(s.clone())
            } else {
                size(t.clone())
            };
            let e = if right {
                Equality::natural(
                    total.clone(),
                    add(a.clone(), b.clone()),
                    nc("AddComm", vec![size(s.clone()), size(t.clone())]),
                )
            } else {
                Equality::reflexive(total.clone())
            };
            let above = nl(e, a.clone()).then(Equality::boolean(
                less(add(a.clone(), b.clone()), a.clone()),
                bit(false),
                call(
                    "Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",
                    vec![a.clone(), b],
                ),
            ))?;
            self.theorem(
                if right { "RightUpper" } else { "LeftUpper" },
                &guarded,
                Equality::boolean(
                    less(capacity(), a.clone()),
                    bit(false),
                    sl(
                        "UpperTrans",
                        vec![
                            a,
                            total.clone(),
                            capacity(),
                            above.proof,
                            bound(s.clone(), t.clone(), var("fit")),
                        ],
                    ),
                ),
            )?;
        }
        Ok(())
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut c = Concat(builder);
    c.capacity()?;
    c.domain()
}
