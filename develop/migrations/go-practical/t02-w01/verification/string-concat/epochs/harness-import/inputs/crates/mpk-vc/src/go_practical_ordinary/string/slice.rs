//! Full-width substring arithmetic and the frozen complete output carrier.
//! Signed guards precede access; bytes outside the result are canonical zero.
use super::*;
mod carrier;
mod concat;
pub(super) fn append_concat(builder: &mut Builder) -> Result<()> {
    concat::append(builder)
}
mod guards;
mod preservation;
mod signed;

fn sl(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Slice.{name}"), args)
}
fn ord(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn wc(name: &str, args: Vec<Expr>) -> Expr {
    word::wc(64, name, args)
}
fn modulus() -> Expr {
    numeric("Weight6", vec![])
}
fn truth(flag: Expr, yes: bool) -> Expr {
    eq(boolean(), flag, bit(yes))
}
fn sign(x: Expr) -> Result<Expr> {
    word::at(x, 64, 63)
}
fn nl(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(lam("_n", nat(), less(var("_n"), b)), l, r, boolean())
}
fn nr(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(lam("_n", nat(), less(a, var("_n"))), l, r, boolean())
}
fn not(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc("Not", vec![var("_b")])),
        l,
        r,
        boolean(),
    )
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc(name, vec![var("_b"), b])),
        l,
        r,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc(name, vec![a, var("_b")])),
        l,
        r,
        boolean(),
    )
}
fn bit_value(e: Equality) -> Equality {
    let (l, r) = (
        numeric("Bit", vec![e.left.clone()]),
        numeric("Bit", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), numeric("Bit", vec![var("_b")])),
        l,
        r,
        nat(),
    )
}
fn decide(flag: Expr, target: Expr, no: Expr, yes: Expr) -> Expr {
    app(
        call(
            "Std.Bool.cases",
            vec![
                lam(
                    "_tag",
                    boolean(),
                    pi("_branch", eq(boolean(), flag.clone(), var("_tag")), target),
                ),
                lam("_branch", truth(flag.clone(), false), no),
                lam("_branch", truth(flag.clone(), true), yes),
                flag.clone(),
            ],
        ),
        vec![refl(boolean(), flag)],
    )
}
fn iw(at: Vec<Expr>) -> Expr {
    sl("IndexWord", at)
}
fn hs(w: Expr, lo: usize, hi: usize) -> Result<Expr> {
    if hi == lo + 1 {
        return word::at(w, 64, lo);
    }
    let m = lo + (hi - lo) / 2;
    Ok(bc("Or", vec![hs(w.clone(), lo, m)?, hs(w, m, hi)?]))
}
fn high_clear(w: Expr, lo: usize, hi: usize) -> Result<Equality> {
    if hi == lo + 1 {
        return Ok(Equality::boolean(
            word::at(w, 64, lo)?,
            bit(false),
            refl(boolean(), bit(false)),
        ));
    }
    let m = lo + (hi - lo) / 2;
    let l = high_clear(w.clone(), lo, m)?;
    let r = high_clear(w, m, hi)?;
    bool_left("Or", l, r.left.clone())
        .then(bool_right("Or", bit(false), r))?
        .then(Equality::boolean(
            bc("Or", vec![bit(false), bit(false)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))
}

struct Slice<'a>(&'a mut Builder);
impl Slice<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Slice.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn sub_exact(&mut self) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let a = value(x.clone());
        let b = value(y.clone());
        let r = value(wc("Sub", vec![x.clone(), y.clone()]));
        let c = wc(
            "Carry64",
            vec![x.clone(), wc("Not", vec![y.clone()]), bit(true)],
        );
        let carry = Equality::boolean(
            bc("Not", vec![c.clone()]),
            less(a.clone(), b.clone()),
            ord("W64.UnsignedLessNatural", vec![x.clone(), y.clone()]),
        )
        .then(Equality::boolean(
            less(a.clone(), b.clone()),
            bit(false),
            var("ordered"),
        ))?;
        let carry = Equality::boolean(
            bc("Not", vec![bc("Not", vec![c.clone()])]),
            c.clone(),
            bc("DoubleNot", vec![c.clone()]),
        )
        .reverse()
        .then(not(carry))?
        .then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        let unit = bit_value(carry).then(Equality::natural(
            numeric("Bit", vec![bit(true)]),
            one(),
            refl(nat(), one()),
        ))?;
        let weighted =
            super::super::polynomial::mul_right(modulus(), unit).then(Equality::natural(
                mul(modulus(), one()),
                modulus(),
                nc("MulOne", vec![modulus()]),
            ))?;
        let sum = add(r.clone(), b.clone());
        let conserve = Equality::natural(
            add(a.clone(), modulus()),
            add(sum.clone(), mul(modulus(), numeric("Bit", vec![c]))),
            ord("W64.SubtractionConservation", vec![x.clone(), y.clone()]),
        )
        .then(weighted.on_right(sum.clone()))?;
        let cancellation = Equality::natural(
            a.clone(),
            sum.clone(),
            nc(
                "AddCancel",
                vec![a.clone(), sum.clone(), modulus(), conserve.proof],
            ),
        );
        let e = cancellation
            .mapped(
                lam("_n", nat(), sub(var("_n"), b.clone())),
                sub(a.clone(), b.clone()),
                sub(sum, b.clone()),
                nat(),
            )
            .then(Equality::natural(
                sub(add(r.clone(), b.clone()), b.clone()),
                r.clone(),
                nc("SubAddRight", vec![r, b.clone()]),
            ))?
            .reverse();
        self.theorem(
            "SubValue",
            &[
                ("x", word64()),
                ("y", word64()),
                ("ordered", truth(less(a, b), false)),
            ],
            e,
        )
    }
    fn indices(&mut self) -> Result<()> {
        let at = address_bits(&SLOTS);
        let mut bits = at.iter().rev().cloned().collect::<Vec<_>>();
        bits.extend(vec![bit(false); 56]);
        let args = SLOTS
            .iter()
            .map(|name| (*name, boolean()))
            .collect::<Vec<_>>();
        let w = iw(at.clone());
        self.emit(
            "IndexWord",
            &args,
            word64(),
            cube(
                &word::ADDRESSES[..6],
                tree(&bits, &word::ADDRESSES[..6], boolean()),
                true,
            ),
            false,
        )?;
        self.theorem(
            "IndexSign",
            &args,
            Equality::boolean(sign(w.clone())?, bit(false), refl(boolean(), bit(false))),
        )?;
        let flag = call("Mpk.Go.Practical.Word.Shifts.CountHigh8", vec![w.clone()]);
        let body = hs(w.clone(), 8, 64)?;
        self.theorem(
            "IndexHigh",
            &args,
            Equality::boolean(flag.clone(), body.clone(), refl(boolean(), body))
                .then(high_clear(w.clone(), 8, 64)?)?,
        )?;
        let digits = numeric("W64.Digits8", vec![w.clone()]);
        let low = (0..8)
            .rev()
            .map(|i| word::at(w.clone(), 64, i))
            .collect::<Result<Vec<_>>>()?;
        let mut e = Equality::natural(
            value(w.clone()),
            digits.clone(),
            ix("SmallValue", vec![w.clone(), sl("IndexHigh", at.clone())]),
        )
        .then(Equality::natural(
            digits,
            index(low.clone()),
            ix("DigitsIndex", vec![w]),
        ))?;
        let mut current = low;
        for i in 0..8 {
            let mut next = current.clone();
            next[i] = at[i].clone();
            let mut template = current.clone();
            template[i] = var("_new_bit");
            e = e.then(
                Equality::boolean(
                    current[i].clone(),
                    at[i].clone(),
                    refl(boolean(), at[i].clone()),
                )
                .mapped(
                    lam("_new_bit", boolean(), index(template)),
                    index(current),
                    index(next.clone()),
                    nat(),
                ),
            )?;
            current = next;
        }
        self.theorem("IndexValue", &args, e)
    }
    fn addition(&mut self) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let (a, b) = (value(x.clone()), value(y.clone()));
        let sum = add(a.clone(), b.clone());
        let raw = wc("AddCarry", vec![x.clone(), y.clone(), bit(false)]);
        let out = wc("Add", vec![x.clone(), y.clone()]);
        let c = wc("Carry64", vec![x.clone(), y.clone(), bit(false)]);
        let left = add(sum.clone(), numeric("Bit", vec![bit(false)]));
        let right = add(
            value(raw.clone()),
            mul(modulus(), numeric("Bit", vec![c.clone()])),
        );
        let core = Equality::natural(
            add(zero(), left.clone()),
            add(zero(), right.clone()),
            numeric(
                "W64.AdditionConservation",
                vec![x.clone(), y.clone(), bit(false), zero()],
            ),
        );
        let core = Equality::natural(
            add(zero(), left.clone()),
            left.clone(),
            nc("ZeroAdd", vec![left.clone()]),
        )
        .reverse()
        .then(core)?
        .then(Equality::natural(
            add(zero(), right.clone()),
            right.clone(),
            nc("ZeroAdd", vec![right]),
        ))?;
        let l = Equality::natural(
            numeric("Bit", vec![bit(false)]),
            zero(),
            refl(nat(), zero()),
        )
        .on_right(sum.clone())
        .then(Equality::natural(
            add(sum.clone(), zero()),
            sum.clone(),
            nc("AddZero", vec![sum.clone()]),
        ))?;
        let representation = Equality {
            ty: word64(),
            left: out.clone(),
            right: raw.clone(),
            proof: refl(word64(), raw.clone()),
        }
        .mapped(
            lam("_word", word64(), value(var("_word"))),
            value(out.clone()),
            value(raw),
            nat(),
        );
        let e = l.reverse().then(core)?.then(
            representation
                .reverse()
                .on_left(mul(modulus(), numeric("Bit", vec![c.clone()]))),
        )?;
        self.theorem("AddConservation", &[("x", word64()), ("y", word64())], e)?;
        let r = value(out.clone());
        let unit = bit_value(Equality::boolean(c.clone(), bit(true), var("_branch"))).then(
            Equality::natural(numeric("Bit", vec![bit(true)]), one(), refl(nat(), one())),
        )?;
        let weighted = mul_right(modulus(), unit).then(Equality::natural(
            mul(modulus(), one()),
            modulus(),
            nc("MulOne", vec![modulus()]),
        ))?;
        let e = Equality::natural(
            sum.clone(),
            add(r.clone(), mul(modulus(), numeric("Bit", vec![c.clone()]))),
            sl("AddConservation", vec![x.clone(), y.clone()]),
        )
        .then(weighted.on_right(r.clone()))?;
        let impossible = nl(e, modulus())
            .then(nl(
                Equality::natural(
                    add(r.clone(), modulus()),
                    add(modulus(), r.clone()),
                    nc("AddComm", vec![r.clone(), modulus()]),
                ),
                modulus(),
            ))?
            .then(Equality::boolean(
                less(add(modulus(), r.clone()), modulus()),
                bit(false),
                call(
                    "Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",
                    vec![modulus(), r.clone()],
                ),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(sum.clone(), modulus()),
                bit(true),
                var("small"),
            ))?;
        let target = truth(c.clone(), false);
        let proof = decide(
            c.clone(),
            target.clone(),
            var("_branch"),
            bc("FalseElim", vec![target, impossible.proof]),
        );
        let args = [
            ("x", word64()),
            ("y", word64()),
            ("small", truth(less(sum.clone(), modulus()), true)),
        ];
        self.theorem(
            "AddCarryClear",
            &args,
            Equality::boolean(c.clone(), bit(false), proof),
        )?;
        let clear = bit_value(Equality::boolean(
            c.clone(),
            bit(false),
            sl("AddCarryClear", vec![x.clone(), y.clone(), var("small")]),
        ))
        .then(Equality::natural(
            numeric("Bit", vec![bit(false)]),
            zero(),
            refl(nat(), zero()),
        ))?;
        let weighted = mul_right(modulus(), clear).then(Equality::natural(
            mul(modulus(), zero()),
            zero(),
            nc("MulZero", vec![modulus()]),
        ))?;
        let e = Equality::natural(
            sum.clone(),
            add(r.clone(), mul(modulus(), numeric("Bit", vec![c]))),
            sl("AddConservation", vec![x, y]),
        )
        .then(weighted.on_right(r.clone()))?
        .then(Equality::natural(
            add(r.clone(), zero()),
            r.clone(),
            nc("AddZero", vec![r]),
        ))?
        .reverse();
        self.theorem("AddValue", &args, e)
    }
    fn geometry(&mut self) -> Result<()> {
        let (a, b, i, n, r) = (var("a"), var("b"), var("i"), var("n"), var("r"));
        let difference = sub(b.clone(), a.clone());
        let decomposition = Equality::natural(
            add(a.clone(), difference.clone()),
            b.clone(),
            nc(
                "DifferenceDecomposition",
                vec![b.clone(), a.clone(), var("ordered")],
            ),
        );
        let e = nl(decomposition.clone().reverse(), difference.clone())
            .then(nl(
                Equality::natural(
                    add(a.clone(), difference.clone()),
                    add(difference.clone(), a.clone()),
                    nc("AddComm", vec![a.clone(), difference.clone()]),
                ),
                difference.clone(),
            ))?
            .then(Equality::boolean(
                less(add(difference.clone(), a.clone()), difference.clone()),
                bit(false),
                call(
                    "Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",
                    vec![difference.clone(), a.clone()],
                ),
            ))?;
        self.theorem(
            "DifferenceUpper",
            &[
                ("a", nat()),
                ("b", nat()),
                ("ordered", truth(less(b.clone(), a.clone()), false)),
            ],
            e,
        )?;
        let target = truth(less(n.clone(), r.clone()), false);
        let smaller = call(
            "Mpk.Go.Practical.Word.Index.UpperBound",
            vec![
                b.clone(),
                r.clone(),
                n.clone(),
                var("first"),
                var("_branch"),
            ],
        );
        let contradiction =
            Equality::boolean(less(n.clone(), b.clone()), bit(false), var("second"))
                .reverse()
                .then(Equality::boolean(
                    less(n.clone(), b.clone()),
                    bit(true),
                    smaller,
                ))?;
        self.theorem(
            "UpperTrans",
            &[
                ("r", nat()),
                ("b", nat()),
                ("n", nat()),
                ("first", truth(less(b.clone(), r.clone()), false)),
                ("second", truth(less(n.clone(), b.clone()), false)),
            ],
            Equality::boolean(
                less(n.clone(), r),
                bit(false),
                decide(
                    less(n.clone(), var("r")),
                    target.clone(),
                    var("_branch"),
                    bc("FalseElim", vec![target, contradiction.proof]),
                ),
            ),
        )?;
        let e = nr(add(a.clone(), i.clone()), decomposition.reverse())
            .then(nl(
                Equality::natural(
                    add(a.clone(), i.clone()),
                    add(i.clone(), a.clone()),
                    nc("AddComm", vec![a.clone(), i.clone()]),
                ),
                add(a.clone(), difference.clone()),
            ))?
            .then(nr(
                add(i.clone(), a.clone()),
                Equality::natural(
                    add(a.clone(), difference.clone()),
                    add(difference.clone(), a.clone()),
                    nc("AddComm", vec![a.clone(), difference.clone()]),
                ),
            ))?
            .then(Equality::boolean(
                less(
                    add(i.clone(), a.clone()),
                    add(difference.clone(), a.clone()),
                ),
                less(i.clone(), difference.clone()),
                nc(
                    "LessTranslate",
                    vec![i.clone(), difference.clone(), a.clone()],
                ),
            ))?
            .then(Equality::boolean(
                less(i.clone(), difference),
                bit(true),
                var("within"),
            ))?;
        self.theorem(
            "IndexBeforeHigh",
            &[
                ("a", nat()),
                ("b", nat()),
                ("i", nat()),
                ("ordered", truth(less(b.clone(), a.clone()), false)),
                (
                    "within",
                    truth(less(i.clone(), sub(b.clone(), a.clone())), true),
                ),
            ],
            e,
        )?;
        self.theorem(
            "IndexWithinSource",
            &[
                ("a", nat()),
                ("b", nat()),
                ("n", nat()),
                ("i", nat()),
                ("ordered", truth(less(b.clone(), a.clone()), false)),
                ("high", truth(less(n.clone(), b.clone()), false)),
                (
                    "within",
                    truth(less(i.clone(), sub(b.clone(), a.clone())), true),
                ),
            ],
            Equality::boolean(
                less(add(a.clone(), i.clone()), n.clone()),
                bit(true),
                call(
                    "Mpk.Go.Practical.Word.Index.UpperBound",
                    vec![
                        n,
                        b.clone(),
                        add(a.clone(), i.clone()),
                        var("high"),
                        sl(
                            "IndexBeforeHigh",
                            vec![a, b, i, var("ordered"), var("within")],
                        ),
                    ],
                ),
            ),
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut s = Slice(builder);
    s.sub_exact()?;
    s.indices()?;
    s.addition()?;
    s.geometry()?;
    s.guards()?;
    s.signed_bounds()?;
    s.carrier()?;
    s.preservation()
}
