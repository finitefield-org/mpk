//! Actual length/data constructor, full-width reads and canonical padding.
use super::*;
pub(super) fn result(s: Expr, t: Expr) -> Expr {
    cc("Value", vec![s, t])
}
pub(super) fn right_word(s: Expr, at: Vec<Expr>) -> Expr {
    let mut args = vec![s];
    args.extend(at);
    cc("RightWord", args)
}
pub(super) fn slot(s: Expr, at: Vec<Expr>) -> Expr {
    app(data(s), at)
}
pub(super) fn call_at(name: &str, s: Expr, t: Expr, at: Vec<Expr>, tail: Vec<Expr>) -> Expr {
    let mut args = vec![s, t];
    args.extend(at);
    args.extend(tail);
    cc(name, args)
}
pub(super) fn size_result(s: Expr, t: Expr, h: Expr) -> Equality {
    Equality::natural(
        size(result(s.clone(), t.clone())),
        sum(s.clone(), t.clone()),
        cc("Size", vec![s, t, h]),
    )
}
pub(super) fn right_value(s: Expr, at: Vec<Expr>, after: Expr) -> Equality {
    let mut args = vec![s.clone()];
    args.extend(at.clone());
    args.push(after);
    Equality::natural(
        value(right_word(s.clone(), at.clone())),
        sub(index(at), size(s)),
        cc("RightValue", args),
    )
}
fn raw_right(i: Expr, s: Expr) -> Expr {
    wc("Sub", vec![i, length(s)])
}
impl Concat<'_> {
    pub(super) fn carrier(&mut self) -> Result<()> {
        self.geometry()?;
        self.definitions()?;
        self.validity()?;
        self.reads()
    }
    fn geometry(&mut self) -> Result<()> {
        let (a, b, i) = (var("a"), var("b"), var("i"));
        let d = sub(i.clone(), a.clone());
        let decomposition = Equality::natural(
            add(a.clone(), d.clone()),
            i.clone(),
            nc(
                "DifferenceDecomposition",
                vec![i.clone(), a.clone(), var("after")],
            ),
        );
        let e = nl(decomposition.reverse(), add(a.clone(), b.clone()))
            .then(nl(
                Equality::natural(
                    add(a.clone(), d.clone()),
                    add(d.clone(), a.clone()),
                    nc("AddComm", vec![a.clone(), d.clone()]),
                ),
                add(a.clone(), b.clone()),
            ))?
            .then(nr(
                add(d.clone(), a.clone()),
                Equality::natural(
                    add(a.clone(), b.clone()),
                    add(b.clone(), a.clone()),
                    nc("AddComm", vec![a.clone(), b.clone()]),
                ),
            ))?
            .then(Equality::boolean(
                less(add(d.clone(), a.clone()), add(b.clone(), a.clone())),
                less(d.clone(), b.clone()),
                nc("LessTranslate", vec![d.clone(), b.clone(), a.clone()]),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(i.clone(), add(a.clone(), b.clone())),
                bit(true),
                var("within"),
            ))?;
        self.theorem(
            "RightWithinNat",
            &[
                ("a", nat()),
                ("b", nat()),
                ("i", nat()),
                ("after", truth(less(i.clone(), a), false)),
                ("within", truth(less(i, add(var("a"), b)), true)),
            ],
            e,
        )?;
        let (s, t, i) = (var("s"), var("t"), var("i"));
        let at = address_bits(&SLOTS);
        let mut args = vec![("s", string_type())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let raw = raw_right(iw(at.clone()), s.clone());
        self.emit("RightWord", &args, word64(), raw.clone(), false)?;
        self.theorem(
            "RightWordStep",
            &args,
            Equality {
                ty: word64(),
                left: right_word(s.clone(), at.clone()),
                right: raw.clone(),
                proof: refl(word64(), raw.clone()),
            },
        )?;
        args.push((
            "after",
            truth(less(index(at.clone()), size(s.clone())), false),
        ));
        let address = Equality::natural(
            value(iw(at.clone())),
            index(at.clone()),
            sl("IndexValue", at.clone()),
        );
        let ordered = nl(address.clone(), size(s.clone())).then(Equality::boolean(
            less(index(at.clone()), size(s.clone())),
            bit(false),
            var("after"),
        ))?;
        let mut step_args = vec![s.clone()];
        step_args.extend(at.clone());
        let step = Equality {
            ty: word64(),
            left: right_word(s.clone(), at.clone()),
            right: raw.clone(),
            proof: cc("RightWordStep", step_args),
        }
        .mapped(
            lam("_word", word64(), value(var("_word"))),
            value(right_word(s.clone(), at.clone())),
            value(raw.clone()),
            nat(),
        );
        self.theorem(
            "RightValue",
            &args,
            step.then(Equality::natural(
                value(raw),
                sub(value(iw(at.clone())), size(s.clone())),
                sl(
                    "SubValue",
                    vec![iw(at.clone()), length(s.clone()), ordered.proof],
                ),
            ))?
            .then(address.mapped(
                lam("_n", nat(), sub(var("_n"), size(s.clone()))),
                sub(value(iw(at.clone())), size(s.clone())),
                sub(index(at.clone()), size(s.clone())),
                nat(),
            ))?,
        )?;
        let mut args = vec![("s", string_type()), ("t", string_type())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.extend([
            (
                "after",
                truth(less(index(at.clone()), size(s.clone())), false),
            ),
            (
                "within",
                truth(less(index(at.clone()), sum(s.clone(), t.clone())), true),
            ),
        ]);
        self.theorem(
            "RightWithin",
            &args,
            nl(
                right_value(s.clone(), at.clone(), var("after")),
                size(t.clone()),
            )
            .then(Equality::boolean(
                less(sub(index(at.clone()), size(s.clone())), size(t.clone())),
                bit(true),
                cc(
                    "RightWithinNat",
                    vec![
                        size(s.clone()),
                        size(t.clone()),
                        index(at.clone()),
                        var("after"),
                        var("within"),
                    ],
                ),
            ))?,
        )?;
        self.theorem(
            "RightValueFull",
            &[
                ("s", string_type()),
                ("i", word64()),
                (
                    "after",
                    truth(less(value(i.clone()), size(s.clone())), false),
                ),
            ],
            Equality::natural(
                value(raw_right(i.clone(), s.clone())),
                sub(value(i.clone()), size(s.clone())),
                sl("SubValue", vec![i.clone(), length(s.clone()), var("after")]),
            ),
        )?;
        let x = var("x");
        let y = var("y");
        self.theorem(
            "ByteIndexValue",
            &[
                ("s", string_type()),
                ("x", word64()),
                ("y", word64()),
                ("e", eq(nat(), value(x.clone()), value(y.clone()))),
            ],
            Equality::natural(value(x.clone()), value(y.clone()), var("e")).mapped(
                lam(
                    "_n",
                    nat(),
                    call(
                        "Mpk.Go.Practical.FiniteVector.D8.Lookup",
                        vec![byte(), data(s.clone()), var("_n")],
                    ),
                ),
                observed(s.clone(), x),
                observed(s, y),
                byte(),
            ),
        )
    }
    fn definitions(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let base = [("s", string_type()), ("t", string_type())];
        let at = address_bits(&SLOTS);
        let l = observed(s.clone(), iw(at.clone()));
        let r = observed(t.clone(), right_word(s.clone(), at.clone()));
        let left = less(index(at.clone()), size(s.clone()));
        let inside = less(index(at.clone()), sum(s.clone(), t.clone()));
        let content = word::choose(left.clone(), l.clone(), r.clone(), byte());
        let chosen = word::choose(inside.clone(), content.clone(), zero_word(8), byte());
        self.emit(
            "Data",
            &base,
            slots(),
            cube(&SLOTS, chosen.clone(), true),
            false,
        )?;
        let ds = cc("Data", vec![s.clone(), t.clone()]);
        let res = result(s.clone(), t.clone());
        let len = length_word(s.clone(), t.clone());
        self.emit(
            "Value",
            &base,
            string_type(),
            make(len.clone(), ds.clone()),
            false,
        )?;
        self.theorem(
            "Length",
            &base,
            Equality {
                ty: word64(),
                left: length(res.clone()),
                right: len.clone(),
                proof: st("MakeLength", vec![len.clone(), ds.clone()]),
            },
        )?;
        let raw = Equality {
            ty: word64(),
            left: length(res.clone()),
            right: len.clone(),
            proof: cc("Length", vec![s.clone(), t.clone()]),
        }
        .mapped(
            lam("_word", word64(), value(var("_word"))),
            size(res.clone()),
            value(len.clone()),
            nat(),
        );
        self.theorem("SizeRaw", &base, raw.clone())?;
        let mut fit = base.to_vec();
        fit.push(("fit", truth(fits(s.clone(), t.clone()), true)));
        self.theorem(
            "Size",
            &fit,
            raw.then(Equality::natural(
                value(len.clone()),
                sum(s.clone(), t.clone()),
                cc("LengthValue", vec![s.clone(), t.clone(), var("fit")]),
            ))?,
        )?;
        let mut args = base.to_vec();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let step = Equality {
            ty: slots(),
            left: data(res.clone()),
            right: ds.clone(),
            proof: st("MakeData", vec![len, ds]),
        }
        .mapped(
            lam("_data", slots(), app(var("_data"), at.clone())),
            slot(res.clone(), at.clone()),
            chosen.clone(),
            byte(),
        );
        self.theorem("SlotStep", &args, step.clone())?;
        for yes in [false, true] {
            let mut args = args.clone();
            args.push(("within", truth(inside.clone(), yes)));
            self.theorem(
                if yes { "Inside" } else { "Outside" },
                &args,
                step.clone().then(Equality {
                    ty: byte(),
                    left: chosen.clone(),
                    right: if yes { content.clone() } else { zero_word(8) },
                    proof: sc(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            byte(),
                            inside.clone(),
                            content.clone(),
                            zero_word(8),
                            var("within"),
                        ],
                    ),
                })?,
            )?;
        }
        for yes in [false, true] {
            let mut args = args.clone();
            args.extend([
                ("within", truth(inside.clone(), true)),
                ("branch", truth(left.clone(), yes)),
            ]);
            self.theorem(
                if yes { "Left" } else { "Right" },
                &args,
                Equality {
                    ty: byte(),
                    left: slot(res.clone(), at.clone()),
                    right: content.clone(),
                    proof: call_at(
                        "Inside",
                        s.clone(),
                        t.clone(),
                        at.clone(),
                        vec![var("within")],
                    ),
                }
                .then(Equality {
                    ty: byte(),
                    left: content.clone(),
                    right: if yes { l.clone() } else { r.clone() },
                    proof: sc(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![byte(), left.clone(), l.clone(), r.clone(), var("branch")],
                    ),
                })?,
            )?;
        }
        Ok(())
    }
    fn validity(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let res = result(s.clone(), t.clone());
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("fit", truth(fits(s.clone(), t.clone()), true)),
        ];
        let at = address_bits(&SLOTS);
        let outside = nr(
            index(at.clone()),
            size_result(s.clone(), t.clone(), var("fit")),
        )
        .reverse()
        .then(Equality::boolean(
            less(index(at.clone()), size(res.clone())),
            bit(false),
            var("_outside"),
        ))?;
        self.emit(
            "Padding",
            &args,
            padding(res.clone()),
            cube(
                &SLOTS,
                lam(
                    "_outside",
                    truth(less(index(at.clone()), size(res.clone())), false),
                    call_at("Outside", s.clone(), t.clone(), at, vec![outside.proof]),
                ),
                true,
            ),
            true,
        )?;
        self.theorem(
            "Upper",
            &args,
            nr(capacity(), size_result(s.clone(), t.clone(), var("fit"))).then(
                Equality::boolean(
                    less(capacity(), sum(s.clone(), t.clone())),
                    bit(false),
                    bound(s.clone(), t.clone(), var("fit")),
                ),
            )?,
        )?;
        self.emit(
            "Valid",
            &args,
            valid(res.clone()),
            pc(
                "Make",
                vec![
                    upper(res.clone()),
                    padding(res),
                    cc("Upper", vec![s.clone(), t.clone(), var("fit")]),
                    cc("Padding", vec![s, t, var("fit")]),
                ],
            ),
            true,
        )
    }
    fn reads(&mut self) -> Result<()> {
        let (s, t, i) = (var("s"), var("t"), var("i"));
        let res = result(s.clone(), t.clone());
        let at = (0..8)
            .rev()
            .map(|k| word::at(i.clone(), 64, k))
            .collect::<Result<Vec<_>>>()?;
        let base = vec![
            ("s", string_type()),
            ("t", string_type()),
            ("fit", truth(fits(s.clone(), t.clone()), true)),
            ("i", word64()),
            ("within", within(res.clone(), i.clone())),
        ];
        let full = Equality::natural(
            value(i.clone()),
            index(at.clone()),
            ix(
                "BoundedIndex",
                vec![
                    i.clone(),
                    st(
                        "WithinCapacity",
                        vec![
                            res.clone(),
                            i.clone(),
                            cc("Upper", vec![s.clone(), t.clone(), var("fit")]),
                            var("within"),
                        ],
                    ),
                ],
            ),
        );
        let total_inside = nr(
            value(i.clone()),
            size_result(s.clone(), t.clone(), var("fit")),
        )
        .reverse()
        .then(Equality::boolean(
            less(value(i.clone()), size(res.clone())),
            bit(true),
            var("within"),
        ))?;
        let slot_inside =
            nl(full.clone().reverse(), sum(s.clone(), t.clone())).then(total_inside.clone())?;
        for left in [true, false] {
            let mut args = base.clone();
            args.push((
                "branch",
                truth(less(value(i.clone()), size(s.clone())), left),
            ));
            let at_branch = nl(full.clone().reverse(), size(s.clone())).then(Equality::boolean(
                less(value(i.clone()), size(s.clone())),
                bit(left),
                var("branch"),
            ))?;
            let to_slot = Equality {
                ty: byte(),
                left: observed(res.clone(), i.clone()),
                right: slot(res.clone(), at.clone()),
                proof: st(
                    "ByteWithinSlot",
                    vec![
                        res.clone(),
                        i.clone(),
                        cc("Upper", vec![s.clone(), t.clone(), var("fit")]),
                        var("within"),
                    ],
                ),
            };
            let word = if left {
                iw(at.clone())
            } else {
                right_word(s.clone(), at.clone())
            };
            let output = if left { s.clone() } else { t.clone() };
            let want = if left {
                i.clone()
            } else {
                raw_right(i.clone(), s.clone())
            };
            let e = to_slot.then(Equality {
                ty: byte(),
                left: slot(res.clone(), at.clone()),
                right: observed(output.clone(), word.clone()),
                proof: call_at(
                    if left { "Left" } else { "Right" },
                    s.clone(),
                    t.clone(),
                    at.clone(),
                    vec![slot_inside.proof.clone(), at_branch.proof.clone()],
                ),
            })?;
            let values = if left {
                Equality::natural(
                    value(word.clone()),
                    index(at.clone()),
                    sl("IndexValue", at.clone()),
                )
                .then(full.clone().reverse())?
            } else {
                right_value(s.clone(), at.clone(), at_branch.proof)
                    .then(full.clone().reverse().mapped(
                        lam("_n", nat(), sub(var("_n"), size(s.clone()))),
                        sub(index(at.clone()), size(s.clone())),
                        sub(value(i.clone()), size(s.clone())),
                        nat(),
                    ))?
                    .then(
                        Equality::natural(
                            value(want.clone()),
                            sub(value(i.clone()), size(s.clone())),
                            cc("RightValueFull", vec![s.clone(), i.clone(), var("branch")]),
                        )
                        .reverse(),
                    )?
            };
            self.theorem(
                if left { "ReadLeft" } else { "ReadRight" },
                &args,
                e.then(Equality {
                    ty: byte(),
                    left: observed(output.clone(), word.clone()),
                    right: observed(output.clone(), want.clone()),
                    proof: cc("ByteIndexValue", vec![output, word, want, values.proof]),
                })?,
            )?;
        }
        let mut args = base;
        args.push((
            "after",
            truth(less(value(i.clone()), size(s.clone())), false),
        ));
        self.theorem(
            "RightWithinFull",
            &args,
            nl(
                Equality::natural(
                    value(raw_right(i.clone(), s.clone())),
                    sub(value(i.clone()), size(s.clone())),
                    cc("RightValueFull", vec![s.clone(), i.clone(), var("after")]),
                ),
                size(t.clone()),
            )
            .then(Equality::boolean(
                less(sub(value(i.clone()), size(s.clone())), size(t.clone())),
                bit(true),
                cc(
                    "RightWithinNat",
                    vec![size(s), size(t), value(i), var("after"), total_inside.proof],
                ),
            ))?,
        )
    }
}
