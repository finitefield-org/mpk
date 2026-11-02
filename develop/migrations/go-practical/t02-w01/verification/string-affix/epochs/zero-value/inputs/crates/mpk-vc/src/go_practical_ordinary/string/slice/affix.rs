//! Checked whole-string affixes; length guards precede all byte comparisons.
use super::*;
fn af(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Affix.{name}"), args)
}
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    guards::ec(name, args)
}
fn bound(s: Expr, p: Expr) -> Expr {
    af("Bound", vec![s, p])
}
fn bound_proof(s: Expr, p: Expr, h: Expr) -> Expr {
    af("BoundReflect", vec![s, p, h])
}
fn offset(s: Expr, p: Expr, k: Expr) -> Expr {
    af("Offset", vec![s, p, k])
}
fn offset_nat(s: Expr, p: Expr, k: Expr) -> Expr {
    af("OffsetNat", vec![s, p, k])
}
fn source(s: Expr, p: Expr, k: Expr, i: Expr) -> Expr {
    af("Index", vec![s, p, k, i])
}
fn offset_value(s: Expr, p: Expr, k: Expr, h: Expr) -> Equality {
    Equality::natural(
        value(offset(s.clone(), p.clone(), k.clone())),
        offset_nat(s.clone(), p.clone(), k.clone()),
        af("OffsetValue", vec![s, p, k, h]),
    )
}
fn index_value(s: Expr, p: Expr, k: Expr, h: Expr, i: Expr, inside: Expr) -> Equality {
    Equality::natural(
        value(source(s.clone(), p.clone(), k.clone(), i.clone())),
        add(
            offset_nat(s.clone(), p.clone(), k.clone()),
            value(i.clone()),
        ),
        af("IndexValue", vec![s, p, k, h, i, inside]),
    )
}
fn with_args(s: Expr, p: Expr, _k: Expr) -> Vec<Binder> {
    vec![
        ("s", string_type()),
        ("p", string_type()),
        ("k", boolean()),
        ("bound", truth(less(size(s), size(p)), false)),
    ]
}
struct Affix<'a>(&'a mut Builder);
impl Affix<'_> {
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
            &format!("Mpk.Go.Practical.String.Affix.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn bounds(&mut self) -> Result<()> {
        let (s, p) = (var("s"), var("p"));
        let args = [("s", string_type()), ("p", string_type())];
        let raw = wc("UnsignedLess", vec![length(s.clone()), length(p.clone())]);
        let body = bc("Not", vec![raw.clone()]);
        self.emit("Bound", &args, boolean(), body.clone(), false)?;
        self.theorem(
            "BoundStep",
            &args,
            Equality::boolean(
                bound(s.clone(), p.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        let mut reflect = args.to_vec();
        reflect.push(("h", truth(bound(s.clone(), p.clone()), true)));
        let not_less = Equality::boolean(
            bound(s.clone(), p.clone()),
            body.clone(),
            af("BoundStep", vec![s.clone(), p.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            bound(s.clone(), p.clone()),
            bit(true),
            var("h"),
        ))?;
        let flag = Equality::boolean(
            raw.clone(),
            less(size(s.clone()), size(p.clone())),
            ord(
                "W64.UnsignedLessNatural",
                vec![length(s.clone()), length(p.clone())],
            ),
        );
        self.theorem(
            "BoundReflect",
            &reflect,
            flag.clone().reverse().then(Equality::boolean(
                raw.clone(),
                bit(false),
                sl("NotTrueFalse", vec![raw.clone(), not_less.proof]),
            ))?,
        )?;
        let mut complete = args.to_vec();
        complete.push((
            "bound",
            truth(less(size(s.clone()), size(p.clone())), false),
        ));
        self.theorem(
            "BoundComplete",
            &complete,
            Equality::boolean(
                bound(s.clone(), p.clone()),
                body,
                af("BoundStep", vec![s.clone(), p.clone()]),
            )
            .then(guards::not_false(flag.then(Equality::boolean(
                less(size(s.clone()), size(p.clone())),
                bit(false),
                var("bound"),
            ))?)?)?,
        )?;
        Ok(())
    }
    fn offsets(&mut self) -> Result<()> {
        let (s, p, k) = (var("s"), var("p"), var("k"));
        let base = [("s", string_type()), ("p", string_type()), ("k", boolean())];
        let diff = sub(size(s.clone()), size(p.clone()));
        let raw = wc("Sub", vec![length(s.clone()), length(p.clone())]);
        let body = word::choose(k.clone(), raw.clone(), zero_word(64), word64());
        let natural = word::choose(k.clone(), diff.clone(), zero(), nat());
        self.emit("Offset", &base, word64(), body.clone(), false)?;
        self.emit("OffsetNat", &base, nat(), natural.clone(), false)?;
        self.theorem(
            "OffsetStep",
            &base,
            Equality {
                ty: word64(),
                left: offset(s.clone(), p.clone(), k.clone()),
                right: body.clone(),
                proof: refl(word64(), body.clone()),
            },
        )?;
        self.theorem(
            "OffsetNatStep",
            &base,
            Equality::natural(
                offset_nat(s.clone(), p.clone(), k.clone()),
                natural.clone(),
                refl(nat(), natural.clone()),
            ),
        )?;
        let args = with_args(s.clone(), p.clone(), k.clone());
        let mut branches = vec![];
        for yes in [false, true] {
            let selected = if yes { raw.clone() } else { zero_word(64) };
            let e = Equality {
                ty: word64(),
                left: offset(s.clone(), p.clone(), k.clone()),
                right: body.clone(),
                proof: af("OffsetStep", vec![s.clone(), p.clone(), k.clone()]),
            }
            .then(Equality {
                ty: word64(),
                left: body.clone(),
                right: selected.clone(),
                proof: sc(
                    if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                    vec![
                        word64(),
                        k.clone(),
                        raw.clone(),
                        zero_word(64),
                        var("_branch"),
                    ],
                ),
            })?
            .mapped(
                lam("_word", word64(), value(var("_word"))),
                value(offset(s.clone(), p.clone(), k.clone())),
                value(selected.clone()),
                nat(),
            );
            let e = if yes {
                e.then(Equality::natural(
                    value(raw.clone()),
                    diff.clone(),
                    sl(
                        "SubValue",
                        vec![length(s.clone()), length(p.clone()), var("bound")],
                    ),
                ))?
            } else {
                e.then(Equality::natural(
                    value(zero_word(64)),
                    zero(),
                    st("ZeroCubeValue6", vec![]),
                ))?
            };
            let selected_nat = if yes { diff.clone() } else { zero() };
            let out = Equality::natural(
                offset_nat(s.clone(), p.clone(), k.clone()),
                natural.clone(),
                af("OffsetNatStep", vec![s.clone(), p.clone(), k.clone()]),
            )
            .then(Equality::natural(
                natural.clone(),
                selected_nat,
                sc(
                    if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                    vec![nat(), k.clone(), diff.clone(), zero(), var("_branch")],
                ),
            ))?;
            branches.push(e.then(out.reverse())?.proof);
        }
        let target = eq(
            nat(),
            value(offset(s.clone(), p.clone(), k.clone())),
            offset_nat(s.clone(), p.clone(), k.clone()),
        );
        self.emit(
            "OffsetValue",
            &args,
            target.clone(),
            decide(k, target, branches[0].clone(), branches[1].clone()),
            true,
        )
    }
    fn positions(&mut self) -> Result<()> {
        let (s, p, k, i) = (var("s"), var("p"), var("k"), var("i"));
        let mut args = with_args(s.clone(), p.clone(), k.clone());
        args.extend([
            ("i", nat()),
            ("within", truth(less(i.clone(), size(p.clone())), true)),
        ]);
        let mut branches = vec![];
        for yes in [false, true] {
            let d = sub(size(s.clone()), size(p.clone()));
            let choose = word::choose(k.clone(), d.clone(), zero(), nat());
            let e = Equality::natural(
                offset_nat(s.clone(), p.clone(), k.clone()),
                choose.clone(),
                af("OffsetNatStep", vec![s.clone(), p.clone(), k.clone()]),
            )
            .then(Equality::natural(
                choose,
                if yes { d.clone() } else { zero() },
                sc(
                    if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                    vec![nat(), k.clone(), d.clone(), zero(), var("_branch")],
                ),
            ))?
            .on_left(i.clone());
            let e = nl(e, size(s.clone()));
            let e = if yes {
                let decomp = Equality::natural(
                    add(size(p.clone()), d.clone()),
                    size(s.clone()),
                    nc(
                        "DifferenceDecomposition",
                        vec![size(s.clone()), size(p.clone()), var("bound")],
                    ),
                );
                e.then(nr(add(d.clone(), i.clone()), decomp.reverse()))?
                    .then(nr(
                        add(d.clone(), i.clone()),
                        Equality::natural(
                            add(size(p.clone()), d.clone()),
                            add(d.clone(), size(p.clone())),
                            nc("AddComm", vec![size(p.clone()), d.clone()]),
                        ),
                    ))?
                    .then(nl(
                        Equality::natural(
                            add(d.clone(), i.clone()),
                            add(i.clone(), d.clone()),
                            nc("AddComm", vec![d.clone(), i.clone()]),
                        ),
                        add(d.clone(), size(p.clone())),
                    ))?
                    .then(nr(
                        add(i.clone(), d.clone()),
                        Equality::natural(
                            add(d.clone(), size(p.clone())),
                            add(size(p.clone()), d.clone()),
                            nc("AddComm", vec![d.clone(), size(p.clone())]),
                        ),
                    ))?
                    .then(Equality::boolean(
                        less(add(i.clone(), d.clone()), add(size(p.clone()), d.clone())),
                        less(i.clone(), size(p.clone())),
                        nc("LessTranslate", vec![i.clone(), size(p.clone()), d]),
                    ))?
                    .then(Equality::boolean(
                        less(i.clone(), size(p.clone())),
                        bit(true),
                        var("within"),
                    ))?
            } else {
                e.then(nl(
                    Equality::natural(
                        add(zero(), i.clone()),
                        i.clone(),
                        nc("ZeroAdd", vec![i.clone()]),
                    ),
                    size(s.clone()),
                ))?
                .then(Equality::boolean(
                    less(i.clone(), size(s.clone())),
                    bit(true),
                    call(
                        "Mpk.Go.Practical.Word.Index.UpperBound",
                        vec![
                            size(s.clone()),
                            size(p.clone()),
                            i.clone(),
                            var("bound"),
                            var("within"),
                        ],
                    ),
                ))?
            };
            branches.push(e.proof);
        }
        let flag = less(
            add(offset_nat(s.clone(), p.clone(), k.clone()), i.clone()),
            size(s.clone()),
        );
        self.theorem(
            "IndexBelow",
            &args,
            Equality::boolean(
                flag.clone(),
                bit(true),
                decide(
                    k.clone(),
                    truth(flag, true),
                    branches[0].clone(),
                    branches[1].clone(),
                ),
            ),
        )?;
        let base = [
            ("s", string_type()),
            ("p", string_type()),
            ("k", boolean()),
            ("i", word64()),
        ];
        let raw = wc(
            "Add",
            vec![offset(s.clone(), p.clone(), k.clone()), i.clone()],
        );
        self.emit("Index", &base, word64(), raw.clone(), false)?;
        self.theorem(
            "IndexStep",
            &base,
            Equality {
                ty: word64(),
                left: source(s.clone(), p.clone(), k.clone(), i.clone()),
                right: raw.clone(),
                proof: refl(word64(), raw.clone()),
            },
        )?;
        let mut args = with_args(s.clone(), p.clone(), k.clone());
        args.extend([("i", word64()), ("within", within(p.clone(), i.clone()))]);
        let off = offset_value(s.clone(), p.clone(), k.clone(), var("bound"));
        let sum = off.on_left(value(i.clone()));
        let small = call(
            "Mpk.Go.Practical.Word.Index.UpperBound",
            vec![
                modulus(),
                size(s.clone()),
                add(
                    offset_nat(s.clone(), p.clone(), k.clone()),
                    value(i.clone()),
                ),
                call(
                    "Mpk.Go.Practical.String.Order.NatAsym",
                    vec![
                        size(s.clone()),
                        modulus(),
                        ord("W64.ValueBound", vec![length(s.clone())]),
                    ],
                ),
                af(
                    "IndexBelow",
                    vec![
                        s.clone(),
                        p.clone(),
                        k.clone(),
                        var("bound"),
                        value(i.clone()),
                        var("within"),
                    ],
                ),
            ],
        );
        let small = nl(sum.clone(), modulus()).then(Equality::boolean(
            less(
                add(
                    offset_nat(s.clone(), p.clone(), k.clone()),
                    value(i.clone()),
                ),
                modulus(),
            ),
            bit(true),
            small,
        ))?;
        let step = Equality {
            ty: word64(),
            left: source(s.clone(), p.clone(), k.clone(), i.clone()),
            right: raw.clone(),
            proof: af(
                "IndexStep",
                vec![s.clone(), p.clone(), k.clone(), i.clone()],
            ),
        }
        .mapped(
            lam("_word", word64(), value(var("_word"))),
            value(source(s.clone(), p.clone(), k.clone(), i.clone())),
            value(raw.clone()),
            nat(),
        );
        self.theorem(
            "IndexValue",
            &args,
            step.then(Equality::natural(
                value(raw),
                add(
                    value(offset(s.clone(), p.clone(), k.clone())),
                    value(i.clone()),
                ),
                sl(
                    "AddValue",
                    vec![
                        offset(s.clone(), p.clone(), k.clone()),
                        i.clone(),
                        small.proof,
                    ],
                ),
            ))?
            .then(sum)?,
        )?;
        self.theorem(
            "IndexWithin",
            &args,
            nl(
                index_value(
                    s.clone(),
                    p.clone(),
                    k.clone(),
                    var("bound"),
                    i.clone(),
                    var("within"),
                ),
                size(s.clone()),
            )
            .then(Equality::boolean(
                less(
                    add(
                        offset_nat(s.clone(), p.clone(), k.clone()),
                        value(i.clone()),
                    ),
                    size(s.clone()),
                ),
                bit(true),
                af(
                    "IndexBelow",
                    vec![s, p, k, var("bound"), value(i), var("within")],
                ),
            ))?,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut a = Affix(builder);
    a.bounds()?;
    a.offsets()?;
    a.positions()
}
