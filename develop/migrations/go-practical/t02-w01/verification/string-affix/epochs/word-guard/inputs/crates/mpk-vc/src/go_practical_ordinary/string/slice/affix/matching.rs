//! Actual bounded fold, reflected against every complete64-bit index.
use super::*;
pub(super) fn matches(s: Expr, p: Expr, k: Expr) -> Expr {
    af("Match", vec![s, p, k])
}
pub(super) fn flags(s: Expr, p: Expr, k: Expr) -> Expr {
    af("Flags", vec![s, p, k])
}
fn all(s: Expr, p: Expr, k: Expr) -> Expr {
    ec("D8.All", vec![flags(s, p, k)])
}
pub(super) fn same_byte(x: Expr, y: Expr) -> Expr {
    let bits = address_bits(&BITS[..3]);
    cube(
        &BITS[..3],
        eq(boolean(), app(x, bits.clone()), app(y, bits)),
        false,
    )
}
pub(super) fn at_bit(e: Equality, bits: Vec<Expr>) -> Equality {
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
pub(super) fn byte_value(s: Expr, x: Expr, y: Expr, e: Equality) -> Equality {
    Equality {
        ty: byte(),
        left: observed(s.clone(), x.clone()),
        right: observed(s.clone(), y.clone()),
        proof: call(
            "Mpk.Go.Practical.String.Concat.ByteIndexValue",
            vec![s, x, y, e.proof],
        ),
    }
}
pub(super) fn content(s: Expr, p: Expr, k: Expr) -> Expr {
    let i = var("_i");
    pi(
        "_i",
        word64(),
        pi(
            "_within",
            within(p.clone(), i.clone()),
            same_byte(
                observed(s.clone(), source(s, p.clone(), k, i.clone())),
                observed(p, i),
            ),
        ),
    )
}
pub(super) fn relation(s: Expr, p: Expr, k: Expr) -> Expr {
    af("Relation", vec![s, p, k])
}
pub(super) fn relation_bound(s: Expr, p: Expr, k: Expr, r: Expr) -> Expr {
    af("RelationBound", vec![s, p, k, r])
}
pub(super) fn relation_content(s: Expr, p: Expr, k: Expr, r: Expr, i: Expr, inside: Expr) -> Expr {
    app(af("RelationContent", vec![s, p, k, r]), vec![i, inside])
}
fn at_call(name: &str, s: Expr, p: Expr, k: Expr, at: Vec<Expr>, tail: Vec<Expr>) -> Expr {
    let mut args = vec![s, p, k];
    args.extend(at);
    args.extend(tail);
    af(name, args)
}
impl Affix<'_> {
    pub(super) fn matching(&mut self) -> Result<()> {
        self.slot_read()?;
        self.match_definitions()?;
        self.reflect()?;
        self.complete()?;
        self.api()
    }
    fn slot_read(&mut self) -> Result<()> {
        let s = var("s");
        let at = address_bits(&SLOTS);
        let mut args = vec![("s", string_type())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let selected = Equality::natural(
            value(iw(at.clone())),
            index(at.clone()),
            sl("IndexValue", at.clone()),
        );
        let lookup = |i| {
            call(
                "Mpk.Go.Practical.FiniteVector.D8.Lookup",
                vec![byte(), data(s.clone()), i],
            )
        };
        let mut lookup_args = vec![byte(), data(s.clone())];
        lookup_args.extend(at.clone());
        self.theorem(
            "SlotRead",
            &args,
            selected
                .mapped(
                    lam("_n", nat(), lookup(var("_n"))),
                    observed(s.clone(), iw(at.clone())),
                    lookup(index(at.clone())),
                    byte(),
                )
                .then(Equality {
                    ty: byte(),
                    left: lookup(index(at.clone())),
                    right: app(data(s), at),
                    proof: call("Mpk.Go.Practical.FiniteVector.D8.LookupIndex", lookup_args),
                })?,
        )
    }
    fn match_definitions(&mut self) -> Result<()> {
        let (s, p, k) = (var("s"), var("p"), var("k"));
        let base = [("s", string_type()), ("p", string_type()), ("k", boolean())];
        let at = address_bits(&SLOTS);
        let flag = less(index(at.clone()), size(p.clone()));
        let x = observed(
            s.clone(),
            source(s.clone(), p.clone(), k.clone(), iw(at.clone())),
        );
        let y = app(data(p.clone()), at.clone());
        let equal = ec("ByteEqual", vec![x.clone(), y.clone()]);
        let choice = word::choose(flag.clone(), equal.clone(), bit(true), boolean());
        let full_guard = wc("UnsignedLess", vec![iw(at.clone()), length(p.clone())]);
        let actual_choice = word::choose(full_guard.clone(), equal.clone(), bit(true), boolean());
        self.emit(
            "Flags",
            &base,
            cube(&SLOTS, boolean(), false),
            cube(&SLOTS, actual_choice, true),
            false,
        )?;
        let mut args = base.to_vec();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        self.theorem(
            "FlagsStep",
            &args,
            Equality::boolean(
                app(flags(s.clone(), p.clone(), k.clone()), at.clone()),
                choice.clone(),
                Equality::boolean(
                    full_guard,
                    less(value(iw(at.clone())), size(p.clone())),
                    ord(
                        "W64.UnsignedLessNatural",
                        vec![iw(at.clone()), length(p.clone())],
                    ),
                )
                .then(nl(
                    Equality::natural(
                        value(iw(at.clone())),
                        index(at.clone()),
                        sl("IndexValue", at.clone()),
                    ),
                    size(p.clone()),
                ))?
                .mapped(
                    lam(
                        "_guard",
                        boolean(),
                        word::choose(var("_guard"), equal.clone(), bit(true), boolean()),
                    ),
                    app(flags(s.clone(), p.clone(), k.clone()), at.clone()),
                    choice.clone(),
                    boolean(),
                )
                .proof,
            ),
        )?;
        for yes in [false, true] {
            let mut args = args.clone();
            args.push(("inside", truth(flag.clone(), yes)));
            self.theorem(
                if yes { "FlagInside" } else { "FlagOutside" },
                &args,
                Equality::boolean(
                    app(flags(s.clone(), p.clone(), k.clone()), at.clone()),
                    choice.clone(),
                    at_call(
                        "FlagsStep",
                        s.clone(),
                        p.clone(),
                        k.clone(),
                        at.clone(),
                        vec![],
                    ),
                )
                .then(Equality::boolean(
                    choice.clone(),
                    if yes { equal.clone() } else { bit(true) },
                    sc(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            boolean(),
                            flag.clone(),
                            equal.clone(),
                            bit(true),
                            var("inside"),
                        ],
                    ),
                ))?,
            )?;
        }
        let body = bc(
            "And",
            vec![
                all(s.clone(), p.clone(), k.clone()),
                bound(s.clone(), p.clone()),
            ],
        );
        self.emit("Match", &base, boolean(), body.clone(), false)?;
        self.theorem(
            "MatchStep",
            &base,
            Equality::boolean(
                matches(s.clone(), p.clone(), k.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        let mut args = base.to_vec();
        args.push(("h", truth(matches(s.clone(), p.clone(), k.clone()), true)));
        let h = Equality::boolean(
            matches(s.clone(), p.clone(), k.clone()),
            body,
            af("MatchStep", vec![s.clone(), p.clone(), k.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            matches(s.clone(), p.clone(), k.clone()),
            bit(true),
            var("h"),
        ))?;
        let bound_h = ec(
            "AndRight",
            vec![
                all(s.clone(), p.clone(), k.clone()),
                bound(s.clone(), p.clone()),
                h.proof.clone(),
            ],
        );
        self.theorem(
            "MatchedBound",
            &args,
            Equality::boolean(
                less(size(s.clone()), size(p.clone())),
                bit(false),
                bound_proof(s.clone(), p.clone(), bound_h),
            ),
        )?;
        self.theorem(
            "MatchedAll",
            &args,
            Equality::boolean(
                all(s.clone(), p.clone(), k.clone()),
                bit(true),
                ec(
                    "AndLeft",
                    vec![
                        all(s.clone(), p.clone(), k.clone()),
                        bound(s.clone(), p.clone()),
                        h.proof,
                    ],
                ),
            ),
        )?;
        let bound_ty = truth(less(size(s.clone()), size(p.clone())), false);
        self.emit(
            "Relation",
            &base,
            Expr::Sort0,
            product(bound_ty.clone(), content(s.clone(), p.clone(), k.clone())),
            false,
        )?;
        let mut args = base.to_vec();
        args.push(("r", relation(s.clone(), p.clone(), k.clone())));
        self.emit(
            "RelationBound",
            &args,
            bound_ty.clone(),
            pc(
                "Left",
                vec![
                    bound_ty.clone(),
                    content(s.clone(), p.clone(), k.clone()),
                    var("r"),
                ],
            ),
            true,
        )?;
        self.emit(
            "RelationContent",
            &args,
            content(s.clone(), p.clone(), k.clone()),
            pc(
                "Right",
                vec![bound_ty, content(s.clone(), p.clone(), k.clone()), var("r")],
            ),
            true,
        )?;
        let x = var("x");
        let y = var("y");
        let mut args = with_args(s.clone(), p.clone(), k.clone());
        args.extend([
            ("x", word64()),
            ("y", word64()),
            ("xin", within(p.clone(), x.clone())),
            ("yin", within(p.clone(), y.clone())),
            ("e", eq(nat(), value(x.clone()), value(y.clone()))),
        ]);
        let values =
            index_value(
                s.clone(),
                p.clone(),
                k.clone(),
                var("bound"),
                x.clone(),
                var("xin"),
            )
            .then(
                Equality::natural(value(x.clone()), value(y.clone()), var("e"))
                    .on_right(offset_nat(s.clone(), p.clone(), k.clone())),
            )?
            .then(
                index_value(
                    s.clone(),
                    p.clone(),
                    k.clone(),
                    var("bound"),
                    y.clone(),
                    var("yin"),
                )
                .reverse(),
            )?;
        self.theorem(
            "IndexReadRespectsValue",
            &args,
            byte_value(
                s.clone(),
                source(s.clone(), p.clone(), k.clone(), x),
                source(s, p, k, y),
                values,
            ),
        )?;
        Ok(())
    }
    fn reflect(&mut self) -> Result<()> {
        let (s, p, k) = (var("s"), var("p"), var("k"));
        let i = var("_i");
        let args = [
            ("s", string_type()),
            ("p", string_type()),
            ("k", boolean()),
            ("upper", upper(p.clone())),
            ("h", truth(matches(s.clone(), p.clone(), k.clone()), true)),
        ];
        let bounded = af(
            "MatchedBound",
            vec![s.clone(), p.clone(), k.clone(), var("h")],
        );
        let at = (0..8)
            .rev()
            .map(|j| word::at(i.clone(), 64, j))
            .collect::<Result<Vec<_>>>()?;
        let bits = address_bits(&BITS[..3]);
        let cap = st(
            "WithinCapacity",
            vec![p.clone(), i.clone(), var("upper"), var("_within")],
        );
        let full = Equality::natural(
            value(i.clone()),
            index(at.clone()),
            ix("BoundedIndex", vec![i.clone(), cap]),
        );
        let selected = Equality::natural(
            value(iw(at.clone())),
            index(at.clone()),
            sl("IndexValue", at.clone()),
        );
        let equal_values = full.clone().then(selected.reverse())?;
        let slot_inside = nl(full.reverse(), size(p.clone())).then(Equality::boolean(
            less(value(i.clone()), size(p.clone())),
            bit(true),
            var("_within"),
        ))?;
        let word_inside =
            nl(equal_values.clone().reverse(), size(p.clone())).then(Equality::boolean(
                less(value(i.clone()), size(p.clone())),
                bit(true),
                var("_within"),
            ))?;
        let x = observed(
            s.clone(),
            source(s.clone(), p.clone(), k.clone(), iw(at.clone())),
        );
        let y = app(data(p.clone()), at.clone());
        let folded = app(
            ec(
                "D8.Reflect",
                vec![
                    flags(s.clone(), p.clone(), k.clone()),
                    af(
                        "MatchedAll",
                        vec![s.clone(), p.clone(), k.clone(), var("h")],
                    ),
                ],
            ),
            at.clone(),
        );
        let byte_equal = Equality::boolean(
            app(flags(s.clone(), p.clone(), k.clone()), at.clone()),
            ec("ByteEqual", vec![x.clone(), y.clone()]),
            at_call(
                "FlagInside",
                s.clone(),
                p.clone(),
                k.clone(),
                at.clone(),
                vec![slot_inside.proof],
            ),
        )
        .reverse()
        .then(Equality::boolean(
            app(flags(s.clone(), p.clone(), k.clone()), at.clone()),
            bit(true),
            folded,
        ))?;
        let middle = Equality::boolean(
            app(x.clone(), bits.clone()),
            app(y.clone(), bits.clone()),
            app(
                ec("ByteReflect", vec![x, y, byte_equal.proof]),
                bits.clone(),
            ),
        );
        let left = Equality {
            ty: byte(),
            left: observed(
                s.clone(),
                source(s.clone(), p.clone(), k.clone(), i.clone()),
            ),
            right: observed(
                s.clone(),
                source(s.clone(), p.clone(), k.clone(), iw(at.clone())),
            ),
            proof: af(
                "IndexReadRespectsValue",
                vec![
                    s.clone(),
                    p.clone(),
                    k.clone(),
                    bounded.clone(),
                    i.clone(),
                    iw(at.clone()),
                    var("_within"),
                    word_inside.proof,
                    equal_values.proof.clone(),
                ],
            ),
        };
        let right = Equality {
            ty: byte(),
            left: observed(p.clone(), i.clone()),
            right: app(data(p.clone()), at),
            proof: st(
                "ByteWithinSlot",
                vec![p.clone(), i.clone(), var("upper"), var("_within")],
            ),
        }
        .reverse();
        let points = lam(
            "_i",
            word64(),
            lam(
                "_within",
                within(p.clone(), i),
                cube(
                    &BITS[..3],
                    at_bit(left, bits.clone())
                        .then(middle)?
                        .then(at_bit(right, bits))?
                        .proof,
                    true,
                ),
            ),
        );
        self.emit(
            "Reflect",
            &args,
            relation(s.clone(), p.clone(), k.clone()),
            pc(
                "Make",
                vec![
                    truth(less(size(s.clone()), size(p.clone())), false),
                    content(s, p, k),
                    bounded,
                    points,
                ],
            ),
            true,
        )
    }
    fn complete(&mut self) -> Result<()> {
        let (s, p, k) = (var("s"), var("p"), var("k"));
        let base = [
            ("s", string_type()),
            ("p", string_type()),
            ("k", boolean()),
            ("r", relation(s.clone(), p.clone(), k.clone())),
        ];
        let at = address_bits(&SLOTS);
        let flag = less(index(at.clone()), size(p.clone()));
        let f = app(flags(s.clone(), p.clone(), k.clone()), at.clone());
        let no = at_call(
            "FlagOutside",
            s.clone(),
            p.clone(),
            k.clone(),
            at.clone(),
            vec![var("_branch")],
        );
        let inside = nl(
            Equality::natural(
                value(iw(at.clone())),
                index(at.clone()),
                sl("IndexValue", at.clone()),
            ),
            size(p.clone()),
        )
        .then(Equality::boolean(flag.clone(), bit(true), var("_branch")))?;
        let x = observed(
            s.clone(),
            source(s.clone(), p.clone(), k.clone(), iw(at.clone())),
        );
        let original_y = observed(p.clone(), iw(at.clone()));
        let y = app(data(p.clone()), at.clone());
        let mut slot_args = vec![p.clone()];
        slot_args.extend(at.clone());
        let bits = address_bits(&BITS[..3]);
        let points = Equality::boolean(
            app(x.clone(), bits.clone()),
            app(original_y.clone(), bits.clone()),
            app(
                relation_content(
                    s.clone(),
                    p.clone(),
                    k.clone(),
                    var("r"),
                    iw(at.clone()),
                    inside.proof,
                ),
                bits.clone(),
            ),
        )
        .then(at_bit(
            Equality {
                ty: byte(),
                left: original_y,
                right: y.clone(),
                proof: af("SlotRead", slot_args),
            },
            bits,
        ))?;
        let yes = Equality::boolean(
            f.clone(),
            ec("ByteEqual", vec![x.clone(), y.clone()]),
            at_call(
                "FlagInside",
                s.clone(),
                p.clone(),
                k.clone(),
                at.clone(),
                vec![var("_branch")],
            ),
        )
        .then(Equality::boolean(
            ec("ByteEqual", vec![x.clone(), y.clone()]),
            bit(true),
            ec(
                "ByteComplete",
                vec![x, y, cube(&BITS[..3], points.proof, true)],
            ),
        ))?;
        let points = cube(&SLOTS, decide(flag, truth(f, true), no, yes.proof), true);
        let all_h = ec(
            "D8.Complete",
            vec![flags(s.clone(), p.clone(), k.clone()), points],
        );
        let bound_h = af(
            "BoundComplete",
            vec![
                s.clone(),
                p.clone(),
                relation_bound(s.clone(), p.clone(), k.clone(), var("r")),
            ],
        );
        self.theorem(
            "Complete",
            &base,
            Equality::boolean(
                matches(s.clone(), p.clone(), k.clone()),
                bc(
                    "And",
                    vec![
                        all(s.clone(), p.clone(), k.clone()),
                        bound(s.clone(), p.clone()),
                    ],
                ),
                af("MatchStep", vec![s.clone(), p.clone(), k.clone()]),
            )
            .then(Equality::boolean(
                bc(
                    "And",
                    vec![
                        all(s.clone(), p.clone(), k.clone()),
                        bound(s.clone(), p.clone()),
                    ],
                ),
                bit(true),
                ec(
                    "AndComplete",
                    vec![
                        all(s.clone(), p.clone(), k.clone()),
                        bound(s, p),
                        all_h,
                        bound_h,
                    ],
                ),
            ))?,
        )
    }
    fn api(&mut self) -> Result<()> {
        let (s, p) = (var("s"), var("p"));
        let base = [("s", string_type()), ("p", string_type())];
        for suffix in [false, true] {
            let name = if suffix { "HasSuffix" } else { "HasPrefix" };
            let prefix = if suffix { "Suffix" } else { "Prefix" };
            let k = bit(suffix);
            self.emit(
                name,
                &base,
                boolean(),
                matches(s.clone(), p.clone(), k.clone()),
                false,
            )?;
            self.theorem(
                &format!("{prefix}Step"),
                &base,
                Equality::boolean(
                    af(name, vec![s.clone(), p.clone()]),
                    matches(s.clone(), p.clone(), k.clone()),
                    refl(boolean(), matches(s.clone(), p.clone(), k.clone())),
                ),
            )?;
            let mut args = base.to_vec();
            args.extend([
                ("upper", upper(p.clone())),
                ("h", truth(af(name, vec![s.clone(), p.clone()]), true)),
            ]);
            let h = Equality::boolean(
                af(name, vec![s.clone(), p.clone()]),
                matches(s.clone(), p.clone(), k.clone()),
                af(&format!("{prefix}Step"), vec![s.clone(), p.clone()]),
            )
            .reverse()
            .then(Equality::boolean(
                af(name, vec![s.clone(), p.clone()]),
                bit(true),
                var("h"),
            ))?;
            self.emit(
                &format!("{prefix}Reflect"),
                &args,
                relation(s.clone(), p.clone(), k.clone()),
                af(
                    "Reflect",
                    vec![s.clone(), p.clone(), k.clone(), var("upper"), h.proof],
                ),
                true,
            )?;
            let mut args = base.to_vec();
            args.push(("r", relation(s.clone(), p.clone(), k.clone())));
            self.theorem(
                &format!("{prefix}Complete"),
                &args,
                Equality::boolean(
                    af(name, vec![s.clone(), p.clone()]),
                    matches(s.clone(), p.clone(), k.clone()),
                    af(&format!("{prefix}Step"), vec![s.clone(), p.clone()]),
                )
                .then(Equality::boolean(
                    matches(s.clone(), p.clone(), k.clone()),
                    bit(true),
                    af("Complete", vec![s.clone(), p.clone(), k, var("r")]),
                ))?,
            )?;
        }
        Ok(())
    }
}
