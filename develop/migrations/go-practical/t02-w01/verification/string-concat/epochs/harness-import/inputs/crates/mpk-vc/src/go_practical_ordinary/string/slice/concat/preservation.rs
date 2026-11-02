//! Existing weak content equivalence suffices; tails are never assumed equal.
use super::carrier::{call_at, result, right_value, right_word, size_result, slot};
use super::*;
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    guards::ec(name, args)
}
fn same(s: Expr, t: Expr) -> Expr {
    ec("Equivalent", vec![s, t])
}
fn sizes(s: Expr, t: Expr, e: Expr) -> Equality {
    Equality::natural(
        size(s.clone()),
        size(t.clone()),
        ec("EquivalentLength", vec![s, t, e]),
    )
}
fn same_byte(x: Expr, y: Expr) -> Expr {
    let bits = address_bits(&BITS[..3]);
    cube(
        &BITS[..3],
        eq(boolean(), app(x, bits.clone()), app(y, bits)),
        false,
    )
}
fn at_bit(e: Equality, bits: Vec<Expr>) -> Equality {
    let (l, r) = (
        app(e.left.clone(), bits.clone()),
        app(e.right.clone(), bits.clone()),
    );
    e.mapped(
        lam("_byte", byte(), app(var("_byte"), bits)),
        l,
        r,
        boolean(),
    )
}
fn content(s: Expr, t: Expr) -> Expr {
    let at = address_bits(&SLOTS);
    cube(
        &SLOTS,
        pi(
            "_inside",
            truth(less(index(at.clone()), size(s.clone())), true),
            same_byte(slot(s, at.clone()), slot(t, at)),
        ),
        false,
    )
}
fn normalized(s: Expr) -> Expr {
    call("Mpk.Go.Practical.String.Normalization.Value", vec![s])
}
fn sum_eq(s: Expr, s2: Expr, t: Expr, t2: Expr, se: Expr, te: Expr) -> Equality {
    Equality::natural(
        sum(s.clone(), t.clone()),
        sum(s2.clone(), t2.clone()),
        cc("SumRespects", vec![s, s2, t, t2, se, te]),
    )
}
fn transfer(s: Expr, s2: Expr, t: Expr, t2: Expr, se: Expr, te: Expr, h: Expr) -> Expr {
    cc("FitTransfer", vec![s, s2, t, t2, se, te, h])
}

impl Concat<'_> {
    pub(super) fn preservation(&mut self) -> Result<()> {
        self.conservation()?;
        self.domain_preservation()?;
        self.content_preservation()
    }
    fn conservation(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let args = [("s", string_type()), ("t", string_type())];
        let raw = wc("Add", vec![length(s.clone()), length(t.clone())]);
        let rc = wc(
            "Carry64",
            vec![length(s.clone()), length(t.clone()), bit(false)],
        );
        let word = Equality {
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
        );
        let flag = Equality::boolean(
            carry(s.clone(), t.clone()),
            rc.clone(),
            cc("CarryStep", vec![s.clone(), t.clone()]),
        );
        let e = Equality::natural(
            sum(s.clone(), t.clone()),
            add(value(raw), mul(modulus(), numeric("Bit", vec![rc.clone()]))),
            sl(
                "AddConservation",
                vec![length(s.clone()), length(t.clone())],
            ),
        )
        .then(
            word.reverse()
                .on_left(mul(modulus(), numeric("Bit", vec![rc]))),
        )?
        .then(mul_right(modulus(), bit_value(flag.reverse())).on_right(value(length_word(s, t))))?;
        self.theorem("LengthConservation", &args, e)
    }
    fn domain_preservation(&mut self) -> Result<()> {
        let (s, s2, t, t2) = (var("s"), var("u"), var("t"), var("v"));
        let base = [
            ("s", string_type()),
            ("u", string_type()),
            ("t", string_type()),
            ("v", string_type()),
            ("se", same(s.clone(), s2.clone())),
            ("te", same(t.clone(), t2.clone())),
        ];
        let e = sizes(s.clone(), s2.clone(), var("se"))
            .on_left(size(t.clone()))
            .then(sizes(t.clone(), t2.clone(), var("te")).on_right(size(s2.clone())))?;
        self.theorem("SumRespects", &base, e.clone())?;
        self.theorem(
            "FitsRespects",
            &base,
            Equality::boolean(
                fits(s.clone(), t.clone()),
                bc("Not", vec![less(capacity(), sum(s.clone(), t.clone()))]),
                cc("FitsStep", vec![s.clone(), t.clone()]),
            )
            .then(not(nr(capacity(), e)))?
            .then(
                Equality::boolean(
                    fits(s2.clone(), t2.clone()),
                    bc("Not", vec![less(capacity(), sum(s2.clone(), t2.clone()))]),
                    cc("FitsStep", vec![s2.clone(), t2.clone()]),
                )
                .reverse(),
            )?,
        )?;
        let mut args = base.to_vec();
        args.push(("fit", truth(fits(s.clone(), t.clone()), true)));
        self.theorem(
            "FitTransfer",
            &args,
            Equality::boolean(
                fits(s.clone(), t.clone()),
                fits(s2.clone(), t2.clone()),
                cc(
                    "FitsRespects",
                    vec![
                        s.clone(),
                        s2.clone(),
                        t.clone(),
                        t2.clone(),
                        var("se"),
                        var("te"),
                    ],
                ),
            )
            .reverse()
            .then(Equality::boolean(
                fits(s.clone(), t.clone()),
                bit(true),
                var("fit"),
            ))?,
        )?;
        let args = [("s", string_type()), ("t", string_type())];
        self.theorem(
            "FitsNormalization",
            &args,
            Equality::boolean(
                fits(normalized(s.clone()), normalized(t.clone())),
                fits(s.clone(), t.clone()),
                cc(
                    "FitsRespects",
                    vec![
                        normalized(s.clone()),
                        s.clone(),
                        normalized(t.clone()),
                        t.clone(),
                        ec("NormalizationEquivalent", vec![s]),
                        ec("NormalizationEquivalent", vec![t]),
                    ],
                ),
            ),
        )
    }
    fn content_preservation(&mut self) -> Result<()> {
        let (s, s2, t, t2) = (var("s"), var("u"), var("t"), var("v"));
        let base = vec![
            ("s", string_type()),
            ("u", string_type()),
            ("t", string_type()),
            ("v", string_type()),
            ("se", same(s.clone(), s2.clone())),
            ("te", same(t.clone(), t2.clone())),
            ("fit", truth(fits(s.clone(), t.clone()), true)),
        ];
        let at = address_bits(&SLOTS);
        let bits = address_bits(&BITS[..3]);
        let i = index(at.clone());
        let old = result(s.clone(), t.clone());
        let new = result(s2.clone(), t2.clone());
        let e = sum_eq(
            s.clone(),
            s2.clone(),
            t.clone(),
            t2.clone(),
            var("se"),
            var("te"),
        );
        let within2 = nr(i.clone(), e.clone().reverse()).then(Equality::boolean(
            less(i.clone(), sum(s.clone(), t.clone())),
            bit(true),
            var("within"),
        ))?;
        let length = sizes(s.clone(), s2.clone(), var("se"));
        let mut args = base.clone();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.push((
            "within",
            truth(less(i.clone(), sum(s.clone(), t.clone())), true),
        ));
        let mut branches = vec![];
        for left in [false, true] {
            let branch2 = nr(i.clone(), length.clone().reverse()).then(Equality::boolean(
                less(i.clone(), size(s.clone())),
                bit(left),
                var("_branch"),
            ))?;
            let word = if left {
                iw(at.clone())
            } else {
                right_word(s.clone(), at.clone())
            };
            let other = if left {
                iw(at.clone())
            } else {
                right_word(s2.clone(), at.clone())
            };
            let (x, y, rel) = if left {
                (s.clone(), s2.clone(), var("se"))
            } else {
                (t.clone(), t2.clone(), var("te"))
            };
            let within = if left {
                nl(
                    Equality::natural(value(word.clone()), i.clone(), sl("IndexValue", at.clone())),
                    size(s.clone()),
                )
                .then(Equality::boolean(
                    less(i.clone(), size(s.clone())),
                    bit(true),
                    var("_branch"),
                ))?
                .proof
            } else {
                call_at(
                    "RightWithin",
                    s.clone(),
                    t.clone(),
                    at.clone(),
                    vec![var("_branch"), var("within")],
                )
            };
            let byte_same = app(
                sl(
                    "ByteSameAt",
                    vec![
                        x.clone(),
                        y.clone(),
                        rel,
                        word.clone(),
                        cc(
                            if left { "LeftUpper" } else { "RightUpper" },
                            vec![s.clone(), t.clone(), var("fit")],
                        ),
                        within,
                    ],
                ),
                bits.clone(),
            );
            let middle = Equality::boolean(
                app(observed(x.clone(), word.clone()), bits.clone()),
                app(observed(y.clone(), word.clone()), bits.clone()),
                byte_same,
            );
            let middle = if left {
                middle
            } else {
                let values = right_value(s.clone(), at.clone(), var("_branch"))
                    .then(length.clone().mapped(
                        lam("_n", nat(), sub(i.clone(), var("_n"))),
                        sub(i.clone(), size(s.clone())),
                        sub(i.clone(), size(s2.clone())),
                        nat(),
                    ))?
                    .then(right_value(s2.clone(), at.clone(), branch2.proof.clone()).reverse())?;
                middle.then(at_bit(
                    Equality {
                        ty: byte(),
                        left: observed(y.clone(), word.clone()),
                        right: observed(y.clone(), other.clone()),
                        proof: cc(
                            "ByteIndexValue",
                            vec![y.clone(), word.clone(), other.clone(), values.proof],
                        ),
                    },
                    bits.clone(),
                ))?
            };
            let before = Equality {
                ty: byte(),
                left: slot(old.clone(), at.clone()),
                right: observed(x, word),
                proof: call_at(
                    if left { "Left" } else { "Right" },
                    s.clone(),
                    t.clone(),
                    at.clone(),
                    vec![var("within"), var("_branch")],
                ),
            };
            let after = Equality {
                ty: byte(),
                left: slot(new.clone(), at.clone()),
                right: observed(y, other),
                proof: call_at(
                    if left { "Left" } else { "Right" },
                    s2.clone(),
                    t2.clone(),
                    at.clone(),
                    vec![within2.proof.clone(), branch2.proof],
                ),
            };
            let mut branch_args = args.clone();
            branch_args.push(("_branch", truth(less(i.clone(), size(s.clone())), left)));
            let name = if left {
                "RespectsLeft"
            } else {
                "RespectsRight"
            };
            self.emit(
                name,
                &branch_args,
                same_byte(slot(old.clone(), at.clone()), slot(new.clone(), at.clone())),
                cube(
                    &BITS[..3],
                    at_bit(before, bits.clone())
                        .then(middle)?
                        .then(at_bit(after, bits.clone()).reverse())?
                        .proof,
                    true,
                ),
                true,
            )?;
            let mut actual = vec![
                s.clone(),
                s2.clone(),
                t.clone(),
                t2.clone(),
                var("se"),
                var("te"),
                var("fit"),
            ];
            actual.extend(at.clone());
            actual.extend([var("within"), var("_branch")]);
            branches.push(cc(name, actual));
        }
        let target = same_byte(slot(old.clone(), at.clone()), slot(new.clone(), at.clone()));
        self.emit(
            "RespectsInside",
            &args,
            target.clone(),
            decide(
                less(i.clone(), size(s.clone())),
                target,
                branches[0].clone(),
                branches[1].clone(),
            ),
            true,
        )?;
        let fit2 = transfer(
            s.clone(),
            s2.clone(),
            t.clone(),
            t2.clone(),
            var("se"),
            var("te"),
            var("fit"),
        );
        let lengths = size_result(s.clone(), t.clone(), var("fit"))
            .then(e)?
            .then(size_result(s2.clone(), t2.clone(), fit2).reverse())?;
        let inside = nr(i.clone(), size_result(s.clone(), t.clone(), var("fit")))
            .reverse()
            .then(Equality::boolean(
                less(i.clone(), size(old.clone())),
                bit(true),
                var("_inside"),
            ))?;
        let mut application = vec![
            s.clone(),
            s2.clone(),
            t.clone(),
            t2.clone(),
            var("se"),
            var("te"),
            var("fit"),
        ];
        application.extend(at.clone());
        application.push(inside.proof);
        let points = cube(
            &SLOTS,
            lam(
                "_inside",
                truth(less(i, size(old.clone())), true),
                cc("RespectsInside", application),
            ),
            true,
        );
        self.emit(
            "Respects",
            &base,
            same(old.clone(), new.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(old), size(new.clone())),
                    content(result(s.clone(), t.clone()), new),
                    lengths.proof,
                    points,
                ],
            ),
            true,
        )?;
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("fit", truth(fits(s.clone(), t.clone()), true)),
        ];
        let se = ec(
            "EquivalentSym",
            vec![
                normalized(s.clone()),
                s.clone(),
                ec("NormalizationEquivalent", vec![s.clone()]),
            ],
        );
        let te = ec(
            "EquivalentSym",
            vec![
                normalized(t.clone()),
                t.clone(),
                ec("NormalizationEquivalent", vec![t.clone()]),
            ],
        );
        self.emit(
            "InputNormalization",
            &args,
            same(
                result(normalized(s.clone()), normalized(t.clone())),
                result(s.clone(), t.clone()),
            ),
            ec(
                "EquivalentSym",
                vec![
                    result(s.clone(), t.clone()),
                    result(normalized(s.clone()), normalized(t.clone())),
                    cc(
                        "Respects",
                        vec![
                            s.clone(),
                            normalized(s.clone()),
                            t.clone(),
                            normalized(t.clone()),
                            se,
                            te,
                            var("fit"),
                        ],
                    ),
                ],
            ),
            true,
        )?;
        let mut args = args.to_vec();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let res = result(s.clone(), t.clone());
        let mut application = vec![res.clone(), cc("Padding", vec![s, t, var("fit")])];
        application.extend(at.clone());
        self.theorem(
            "CanonicalSlot",
            &args,
            Equality {
                ty: byte(),
                left: slot(normalized(res.clone()), at.clone()),
                right: slot(res, at),
                proof: call(
                    "Mpk.Go.Practical.String.Normalization.StableSlot",
                    application,
                ),
            },
        )
    }
}
