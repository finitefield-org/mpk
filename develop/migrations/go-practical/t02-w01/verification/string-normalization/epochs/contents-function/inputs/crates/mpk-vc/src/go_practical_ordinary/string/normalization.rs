//! Canonical padding and complete-word guarded observations of arbitrary data.
//! Logical content equivalence deliberately omits unobservable tail bytes.
use super::*;

fn nz(name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("Mpk.Go.Practical.String.Normalization.{name}"),
        args,
    )
}
fn normalized(s: Expr) -> Expr {
    nz("Value", vec![s])
}
fn normalized_data(s: Expr) -> Expr {
    nz("Data", vec![s])
}
fn slot(s: Expr, addr: Vec<Expr>) -> Expr {
    app(data(s), addr)
}
fn choice(flag: Expr, yes: Expr, no: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_flag", boolean(), byte()), no, yes, flag],
    )
}
fn fit(i: Expr, n: Expr, yes: bool) -> Expr {
    eq(boolean(), less(i, n), bit(yes))
}
fn same_length(s: Expr, t: Expr) -> Expr {
    nz("SameLength", vec![s, t])
}
fn same_contents(s: Expr, t: Expr) -> Expr {
    nz("SameContents", vec![s, t])
}
fn equivalent(s: Expr, t: Expr) -> Expr {
    nz("Equivalent", vec![s, t])
}
fn contents(s: Expr, t: Expr) -> Expr {
    let addr = address_bits(&SLOTS);
    cube(
        &SLOTS,
        pi(
            "_inside",
            fit(index(addr.clone()), size(s.clone()), true),
            eq(byte(), slot(s, addr.clone()), slot(t, addr)),
        ),
        false,
    )
}
fn points(s: Expr, proof: Expr) -> Expr {
    cube(
        &SLOTS,
        lam(
            "_inside",
            fit(index(address_bits(&SLOTS)), size(s), true),
            proof,
        ),
        true,
    )
}
fn lengths(s: Expr, t: Expr, e: Expr) -> Equality {
    Equality::natural(
        size(s.clone()),
        size(t.clone()),
        nz("EquivalentLength", vec![s, t, e]),
    )
}
fn at_slot(s: Expr, t: Expr, e: Expr, addr: Vec<Expr>, inside: Expr) -> Equality {
    let mut args = vec![s.clone(), t.clone(), e];
    args.extend(addr.clone());
    args.push(inside);
    Equality {
        ty: byte(),
        left: slot(s, addr.clone()),
        right: slot(t, addr),
        proof: nz("EquivalentSlot", args),
    }
}
fn less_left(e: Equality, n: Expr) -> Equality {
    e.clone().mapped(
        lam("_n", nat(), less(var("_n"), n.clone())),
        less(e.left, n.clone()),
        less(e.right, n),
        boolean(),
    )
}
fn less_right(i: Expr, e: Equality) -> Equality {
    e.clone().mapped(
        lam("_n", nat(), less(i.clone(), var("_n"))),
        less(i.clone(), e.left),
        less(i, e.right),
        boolean(),
    )
}
fn flag_claim(i: Expr, n: Expr, yes: bool, proof: Expr) -> Equality {
    Equality::boolean(less(i, n), bit(yes), proof)
}
fn slot_call(name: &str, s: Expr, addr: Vec<Expr>, witness: Option<Expr>, right: Expr) -> Equality {
    let mut args = vec![s.clone()];
    args.extend(addr.clone());
    args.extend(witness);
    Equality {
        ty: byte(),
        left: slot(normalized(s), addr),
        right,
        proof: nz(name, args),
    }
}
struct Normalization<'a>(&'a mut Builder);
impl Normalization<'_> {
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
            &format!("Mpk.Go.Practical.String.Normalization.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let addr = address_bits(&SLOTS);
        self.emit(
            "Data",
            &[("s", string_type())],
            slots(),
            cube(
                &SLOTS,
                choice(
                    less(index(addr.clone()), size(s.clone())),
                    slot(s.clone(), addr),
                    zero_word(8),
                ),
                true,
            ),
            false,
        )?;
        self.emit(
            "Value",
            &[("s", string_type())],
            string_type(),
            make(length(s.clone()), normalized_data(s.clone())),
            false,
        )?;
        let args = [("s", string_type()), ("t", string_type())];
        self.emit(
            "SameLength",
            &args,
            Expr::Sort0,
            eq(nat(), size(s.clone()), size(t.clone())),
            false,
        )?;
        self.emit(
            "SameContents",
            &args,
            Expr::Sort0,
            contents(s.clone(), t.clone()),
            false,
        )?;
        self.emit(
            "ContentsAsFunction",
            &[
                args[0].clone(),
                args[1].clone(),
                ("related", same_contents(s.clone(), t.clone())),
            ],
            contents(s.clone(), t.clone()),
            var("related"),
            false,
        )?;
        self.emit(
            "Equivalent",
            &args,
            Expr::Sort0,
            product(same_length(s.clone(), t.clone()), same_contents(s, t)),
            false,
        )
    }
    fn equivalence(&mut self) -> Result<()> {
        let (s, t, u, e, f) = (var("s"), var("t"), var("u"), var("e"), var("f"));
        let pair = [
            ("s", string_type()),
            ("t", string_type()),
            ("e", equivalent(s.clone(), t.clone())),
        ];
        for (name, right, ty) in [
            ("EquivalentLength", false, same_length(s.clone(), t.clone())),
            (
                "EquivalentContents",
                true,
                same_contents(s.clone(), t.clone()),
            ),
        ] {
            self.emit(
                name,
                &pair,
                ty,
                pc(
                    if right { "Right" } else { "Left" },
                    vec![
                        same_length(s.clone(), t.clone()),
                        same_contents(s.clone(), t.clone()),
                        e.clone(),
                    ],
                ),
                true,
            )?;
        }
        let addr = address_bits(&SLOTS);
        let i = index(addr.clone());
        let mut args = pair.to_vec();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.push(("inside", fit(i.clone(), size(s.clone()), true)));
        let mut application = addr.clone();
        application.push(var("inside"));
        self.theorem(
            "EquivalentSlot",
            &args,
            Equality {
                ty: byte(),
                left: slot(s.clone(), addr.clone()),
                right: slot(t.clone(), addr.clone()),
                proof: app(
                    nz(
                        "ContentsAsFunction",
                        vec![
                            s.clone(),
                            t.clone(),
                            nz("EquivalentContents", vec![s.clone(), t.clone(), e.clone()]),
                        ],
                    ),
                    application,
                ),
            },
        )?;
        self.emit(
            "EquivalentRefl",
            &[("s", string_type())],
            equivalent(s.clone(), s.clone()),
            pc(
                "Make",
                vec![
                    same_length(s.clone(), s.clone()),
                    same_contents(s.clone(), s.clone()),
                    refl(nat(), size(s.clone())),
                    points(s.clone(), refl(byte(), slot(s.clone(), addr.clone()))),
                ],
            ),
            true,
        )?;
        let len = lengths(s.clone(), t.clone(), e.clone());
        let inside_s = less_right(i.clone(), len.clone()).then(flag_claim(
            i.clone(),
            size(t.clone()),
            true,
            var("_inside"),
        ))?;
        let symmetric = at_slot(
            s.clone(),
            t.clone(),
            e.clone(),
            addr.clone(),
            inside_s.proof,
        )
        .reverse();
        self.emit(
            "EquivalentSym",
            &pair,
            equivalent(t.clone(), s.clone()),
            pc(
                "Make",
                vec![
                    same_length(t.clone(), s.clone()),
                    same_contents(t.clone(), s.clone()),
                    len.clone().reverse().proof,
                    points(t.clone(), symmetric.proof),
                ],
            ),
            true,
        )?;
        let chain = [
            pair[0].clone(),
            pair[1].clone(),
            ("u", string_type()),
            pair[2].clone(),
            ("f", equivalent(t.clone(), u.clone())),
        ];
        let inside_t = less_right(i.clone(), len.clone().reverse()).then(flag_claim(
            i.clone(),
            size(s.clone()),
            true,
            var("_inside"),
        ))?;
        let joined = at_slot(
            s.clone(),
            t.clone(),
            e.clone(),
            addr.clone(),
            var("_inside"),
        )
        .then(at_slot(
            t.clone(),
            u.clone(),
            f.clone(),
            addr,
            inside_t.proof,
        ))?;
        self.emit(
            "EquivalentTrans",
            &chain,
            equivalent(s.clone(), u.clone()),
            pc(
                "Make",
                vec![
                    same_length(s.clone(), u.clone()),
                    same_contents(s.clone(), u.clone()),
                    len.clone().then(lengths(t.clone(), u, f))?.proof,
                    points(s.clone(), joined.proof),
                ],
            ),
            true,
        )?;
        let mut args = pair.to_vec();
        args.push(("upper", upper(s.clone())));
        self.theorem(
            "EquivalentUpper",
            &args,
            less_right(capacity(), len.clone().reverse()).then(flag_claim(
                capacity(),
                size(s.clone()),
                false,
                var("upper"),
            ))?,
        )?;
        let mut args = pair.to_vec();
        args.extend([("i", word64()), ("within", within(s.clone(), var("i")))]);
        self.theorem(
            "EquivalentWithin",
            &args,
            less_right(value(var("i")), len.reverse()).then(flag_claim(
                value(var("i")),
                size(s),
                true,
                var("within"),
            ))?,
        )
    }
    fn read_equivalence(&mut self) -> Result<()> {
        let (s, t, e, i) = (var("s"), var("t"), var("e"), var("i"));
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("e", equivalent(s.clone(), t.clone())),
            ("i", word64()),
            ("upper", upper(s.clone())),
            ("within", within(s.clone(), i.clone())),
        ];
        let address = (0..8)
            .rev()
            .map(|bit| word::at(i.clone(), 64, bit))
            .collect::<Result<Vec<_>>>()?;
        let bound = st(
            "WithinCapacity",
            vec![s.clone(), i.clone(), var("upper"), var("within")],
        );
        let full = Equality::natural(
            value(i.clone()),
            index(address.clone()),
            ix("BoundedIndex", vec![i.clone(), bound]),
        );
        let slot_inside = less_left(full.reverse(), size(s.clone())).then(flag_claim(
            value(i.clone()),
            size(s.clone()),
            true,
            var("within"),
        ))?;
        let left = Equality {
            ty: byte(),
            left: observed(s.clone(), i.clone()),
            right: slot(s.clone(), address.clone()),
            proof: st(
                "ByteWithinSlot",
                vec![s.clone(), i.clone(), var("upper"), var("within")],
            ),
        };
        let right = Equality {
            ty: byte(),
            left: observed(t.clone(), i.clone()),
            right: slot(t.clone(), address.clone()),
            proof: st(
                "ByteWithinSlot",
                vec![
                    t.clone(),
                    i.clone(),
                    nz(
                        "EquivalentUpper",
                        vec![s.clone(), t.clone(), e.clone(), var("upper")],
                    ),
                    nz(
                        "EquivalentWithin",
                        vec![s.clone(), t.clone(), e.clone(), i.clone(), var("within")],
                    ),
                ],
            ),
        };
        self.theorem(
            "EquivalentRead",
            &args,
            left.then(at_slot(s, t, e, address, slot_inside.proof))?
                .then(right.reverse())?,
        )
    }
    fn projections(&mut self) -> Result<()> {
        let s = var("s");
        let made = normalized(s.clone());
        let fields = vec![length(s.clone()), normalized_data(s.clone())];
        self.theorem(
            "Length",
            &[("s", string_type())],
            Equality {
                ty: word64(),
                left: length(made.clone()),
                right: length(s.clone()),
                proof: st("MakeLength", fields.clone()),
            },
        )?;
        self.theorem(
            "Size",
            &[("s", string_type())],
            Equality::natural(
                size(made.clone()),
                size(s.clone()),
                st("MakeSize", fields.clone()),
            ),
        )?;
        let addr = address_bits(&SLOTS);
        let flag = less(index(addr.clone()), size(s.clone()));
        let result = choice(flag.clone(), slot(s.clone(), addr.clone()), zero_word(8));
        let e = Equality {
            ty: slots(),
            left: data(made.clone()),
            right: normalized_data(s.clone()),
            proof: st("MakeData", fields),
        }
        .mapped(
            lam("_data", slots(), app(var("_data"), addr.clone())),
            slot(made, addr.clone()),
            result.clone(),
            byte(),
        );
        let mut args = vec![("s", string_type())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        self.theorem("Slot", &args, e)?;
        for yes in [true, false] {
            let mut args = args.clone();
            args.push(("branch", eq(boolean(), flag.clone(), bit(yes))));
            let e = slot_call("Slot", s.clone(), addr.clone(), None, result.clone()).then(
                Equality {
                    ty: byte(),
                    left: result.clone(),
                    right: if yes {
                        slot(s.clone(), addr.clone())
                    } else {
                        zero_word(8)
                    },
                    proof: sc(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            byte(),
                            flag.clone(),
                            slot(s.clone(), addr.clone()),
                            zero_word(8),
                            var("branch"),
                        ],
                    ),
                },
            )?;
            self.theorem(if yes { "Inside" } else { "Outside" }, &args, e)?;
        }
        Ok(())
    }
    fn validity(&mut self) -> Result<()> {
        let s = var("s");
        let n = normalized(s.clone());
        let addr = address_bits(&SLOTS);
        let i = index(addr.clone());
        let len = Equality::natural(
            size(n.clone()),
            size(s.clone()),
            nz("Size", vec![s.clone()]),
        );
        self.theorem(
            "Upper",
            &[("s", string_type()), ("upper", upper(s.clone()))],
            less_right(capacity(), len.clone()).then(flag_claim(
                capacity(),
                size(s.clone()),
                false,
                var("upper"),
            ))?,
        )?;
        let outside = less_right(i.clone(), len.clone())
            .reverse()
            .then(flag_claim(
                i.clone(),
                size(n.clone()),
                false,
                var("_outside"),
            ))?;
        let proof = slot_call(
            "Outside",
            s.clone(),
            addr.clone(),
            Some(outside.proof),
            zero_word(8),
        );
        self.emit(
            "Padding",
            &[("s", string_type())],
            padding(n.clone()),
            cube(
                &SLOTS,
                lam(
                    "_outside",
                    fit(i.clone(), size(n.clone()), false),
                    proof.proof,
                ),
                true,
            ),
            true,
        )?;
        self.emit(
            "Valid",
            &[("s", string_type()), ("upper", upper(s.clone()))],
            valid(n.clone()),
            pc(
                "Make",
                vec![
                    upper(n.clone()),
                    padding(n.clone()),
                    nz("Upper", vec![s.clone(), var("upper")]),
                    nz("Padding", vec![s.clone()]),
                ],
            ),
            true,
        )?;
        let inside = less_right(i.clone(), len.clone())
            .reverse()
            .then(flag_claim(i, size(n.clone()), true, var("_inside")))?;
        self.emit(
            "ResultEquivalent",
            &[("s", string_type())],
            equivalent(n.clone(), s.clone()),
            pc(
                "Make",
                vec![
                    same_length(n.clone(), s.clone()),
                    same_contents(n.clone(), s.clone()),
                    len.proof,
                    points(
                        n,
                        slot_call(
                            "Inside",
                            s.clone(),
                            addr.clone(),
                            Some(inside.proof),
                            slot(s, addr),
                        )
                        .proof,
                    ),
                ],
            ),
            true,
        )
    }
    fn observations(&mut self) -> Result<()> {
        let s = var("s");
        let n = normalized(s.clone());
        let i = var("i");
        let args = [
            ("s", string_type()),
            ("i", word64()),
            ("upper", upper(s.clone())),
            ("within", within(s.clone(), i.clone())),
        ];
        let inside = less_right(
            value(i.clone()),
            Equality::natural(
                size(n.clone()),
                size(s.clone()),
                nz("Size", vec![s.clone()]),
            ),
        )
        .then(flag_claim(
            value(i.clone()),
            size(s.clone()),
            true,
            var("within"),
        ))?;
        let proof = nz(
            "EquivalentRead",
            vec![
                n.clone(),
                s.clone(),
                nz("ResultEquivalent", vec![s.clone()]),
                i.clone(),
                nz("Upper", vec![s.clone(), var("upper")]),
                inside.proof,
            ],
        );
        self.theorem(
            "Read",
            &args,
            Equality {
                ty: byte(),
                left: observed(n.clone(), i.clone()),
                right: observed(s.clone(), i.clone()),
                proof,
            },
        )?;
        self.theorem(
            "SequenceRead",
            &args,
            Equality {
                ty: byte(),
                left: sc("At", vec![byte(), sequence(n.clone()), value(i.clone())]),
                right: observed(n.clone(), i.clone()),
                proof: st("SequenceByte", vec![n, i.clone()]),
            }
            .then(Equality {
                ty: byte(),
                left: observed(normalized(s.clone()), i.clone()),
                right: observed(s.clone(), i.clone()),
                proof: nz(
                    "Read",
                    vec![s.clone(), i.clone(), var("upper"), var("within")],
                ),
            })?
            .then(
                Equality {
                    ty: byte(),
                    left: sc("At", vec![byte(), sequence(s.clone()), value(i.clone())]),
                    right: observed(s.clone(), i.clone()),
                    proof: st("SequenceByte", vec![s, i]),
                }
                .reverse(),
            )?,
        )
    }
    fn stable(&mut self) -> Result<()> {
        let s = var("s");
        let addr = address_bits(&SLOTS);
        let flag = less(index(addr.clone()), size(s.clone()));
        let goal = eq(
            byte(),
            slot(normalized(s.clone()), addr.clone()),
            slot(s.clone(), addr.clone()),
        );
        let branch = |yes| eq(boolean(), flag.clone(), bit(yes));
        let inside = slot_call(
            "Inside",
            s.clone(),
            addr.clone(),
            Some(var("_branch")),
            slot(s.clone(), addr.clone()),
        );
        let mut padding_args = addr.clone();
        padding_args.push(var("_branch"));
        let outside = slot_call(
            "Outside",
            s.clone(),
            addr.clone(),
            Some(var("_branch")),
            zero_word(8),
        )
        .then(
            Equality {
                ty: byte(),
                left: slot(s.clone(), addr.clone()),
                right: zero_word(8),
                proof: app(var("padding"), padding_args),
            }
            .reverse(),
        )?;
        let motive = lam(
            "_flag",
            boolean(),
            pi(
                "_branch",
                eq(boolean(), flag.clone(), var("_flag")),
                goal.clone(),
            ),
        );
        let proof = app(
            call(
                "Std.Bool.cases",
                vec![
                    motive,
                    lam("_branch", branch(false), outside.proof),
                    lam("_branch", branch(true), inside.proof),
                    flag.clone(),
                ],
            ),
            vec![refl(boolean(), flag)],
        );
        let mut args = vec![("s", string_type()), ("padding", padding(s.clone()))];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        self.emit("StableSlot", &args, goal, proof, true)?;
        let mut args = vec![("s", string_type())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let mut proof_args = vec![normalized(s.clone()), nz("Padding", vec![s.clone()])];
        proof_args.extend(addr.clone());
        self.theorem(
            "IdempotentSlot",
            &args,
            Equality {
                ty: byte(),
                left: slot(normalized(normalized(s.clone())), addr.clone()),
                right: slot(normalized(s.clone()), addr.clone()),
                proof: nz("StableSlot", proof_args),
            },
        )?;
        for full in [false, true] {
            let mut args = vec![
                ("s", string_type()),
                (
                    "size",
                    eq(
                        nat(),
                        size(s.clone()),
                        if full { capacity() } else { zero() },
                    ),
                ),
            ];
            args.extend(SLOTS.iter().map(|n| (*n, boolean())));
            let i = index(addr.clone());
            let end = if full { capacity() } else { zero() };
            let e = less_right(
                i.clone(),
                Equality::natural(size(s.clone()), end.clone(), var("size")),
            )
            .then(flag_claim(
                i.clone(),
                end,
                full,
                if full {
                    call("Mpk.Go.Practical.FiniteVector.D8.IndexBound", addr.clone())
                } else {
                    nc("LessZero", vec![i])
                },
            ))?;
            self.theorem(
                if full { "FullSlot" } else { "EmptySlot" },
                &args,
                slot_call(
                    if full { "Inside" } else { "Outside" },
                    s.clone(),
                    addr.clone(),
                    Some(e.proof),
                    if full {
                        slot(s.clone(), addr.clone())
                    } else {
                        zero_word(8)
                    },
                ),
            )?;
        }
        let t = var("t");
        let e = var("e");
        self.emit(
            "EquivalentNormalized",
            &[
                ("s", string_type()),
                ("t", string_type()),
                ("e", equivalent(s.clone(), t.clone())),
            ],
            equivalent(normalized(s.clone()), normalized(t.clone())),
            nz(
                "EquivalentTrans",
                vec![
                    normalized(s.clone()),
                    t.clone(),
                    normalized(t.clone()),
                    nz(
                        "EquivalentTrans",
                        vec![
                            normalized(s.clone()),
                            s.clone(),
                            t.clone(),
                            nz("ResultEquivalent", vec![s]),
                            e,
                        ],
                    ),
                    nz(
                        "EquivalentSym",
                        vec![
                            normalized(t.clone()),
                            t.clone(),
                            nz("ResultEquivalent", vec![t]),
                        ],
                    ),
                ],
            ),
            true,
        )
    }
    fn bits(&mut self) -> Result<()> {
        let s = var("s");
        let mut args = vec![("s", string_type())];
        args.extend(BITS.iter().map(|n| (*n, boolean())));
        let e = Equality {
            ty: word64(),
            left: length(normalized(s.clone())),
            right: length(s.clone()),
            proof: nz("Length", vec![s.clone()]),
        }
        .mapped(
            lam("_word", word64(), app(var("_word"), address_bits(&BITS))),
            app(length(normalized(s.clone())), address_bits(&BITS)),
            app(length(s.clone()), address_bits(&BITS)),
            boolean(),
        );
        self.theorem("LengthBit", &args, e)?;
        let addr = address_bits(&SLOTS);
        let bitaddr = address_bits(&BITS[..3]);
        for inside in [true, false] {
            let mut args = vec![("s", string_type())];
            args.extend(SLOTS.iter().map(|n| (*n, boolean())));
            args.push(("branch", fit(index(addr.clone()), size(s.clone()), inside)));
            args.extend(BITS[..3].iter().map(|n| (*n, boolean())));
            let right = if inside {
                slot(s.clone(), addr.clone())
            } else {
                zero_word(8)
            };
            let e = slot_call(
                if inside { "Inside" } else { "Outside" },
                s.clone(),
                addr.clone(),
                Some(var("branch")),
                right.clone(),
            )
            .mapped(
                lam("_byte", byte(), app(var("_byte"), bitaddr.clone())),
                app(slot(normalized(s.clone()), addr.clone()), bitaddr.clone()),
                app(right, bitaddr.clone()),
                boolean(),
            );
            self.theorem(if inside { "InsideBit" } else { "OutsideBit" }, &args, e)?;
        }
        Ok(())
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut n = Normalization(builder);
    n.definitions()?;
    n.equivalence()?;
    n.read_equivalence()?;
    n.projections()?;
    n.validity()?;
    n.observations()?;
    n.stable()?;
    n.bits()
}
