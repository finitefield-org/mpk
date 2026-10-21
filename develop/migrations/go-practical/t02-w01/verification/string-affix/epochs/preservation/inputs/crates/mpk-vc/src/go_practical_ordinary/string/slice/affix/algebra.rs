//! Empty, self and concatenation laws use the actual full-word byte equations.
use super::matching::{
    at_bit, byte_value, content, matches, relation, relation_content, same_byte,
};
use super::*;
fn cc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Concat.{name}"), args)
}
fn concat(s: Expr, t: Expr) -> Expr {
    cc("Value", vec![s, t])
}
fn offset_literal(s: Expr, p: Expr, suffix: bool) -> Result<Equality> {
    let k = bit(suffix);
    let d = sub(size(s.clone()), size(p.clone()));
    let choose = word::choose(k.clone(), d.clone(), zero(), nat());
    Equality::natural(
        offset_nat(s.clone(), p.clone(), k.clone()),
        choose.clone(),
        af("OffsetNatStep", vec![s, p, k.clone()]),
    )
    .then(Equality::natural(
        choose,
        if suffix { d.clone() } else { zero() },
        sc(
            if suffix { "ChoiceTrue" } else { "ChoiceFalse" },
            vec![nat(), k.clone(), d, zero(), refl(boolean(), k)],
        ),
    ))
}
impl Affix<'_> {
    pub(super) fn algebra(&mut self) -> Result<()> {
        self.empty_and_long()?;
        self.self_match()?;
        self.observations_api()?;
        self.concatenation()
    }
    fn empty_and_long(&mut self) -> Result<()> {
        let (s, p, k) = (var("s"), var("p"), var("k"));
        let base = vec![("s", string_type()), ("p", string_type()), ("k", boolean())];
        let mut args = base.clone();
        args.push(("long", truth(less(size(s.clone()), size(p.clone())), true)));
        let raw = wc("UnsignedLess", vec![length(s.clone()), length(p.clone())]);
        let negative = not(Equality::boolean(
            raw.clone(),
            less(size(s.clone()), size(p.clone())),
            ord(
                "W64.UnsignedLessNatural",
                vec![length(s.clone()), length(p.clone())],
            ),
        )
        .then(Equality::boolean(
            less(size(s.clone()), size(p.clone())),
            bit(true),
            var("long"),
        ))?)
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        let bound_false = Equality::boolean(
            bound(s.clone(), p.clone()),
            bc("Not", vec![raw]),
            af("BoundStep", vec![s.clone(), p.clone()]),
        )
        .then(negative)?;
        let all = ec(
            "D8.All",
            vec![matching::flags(s.clone(), p.clone(), k.clone())],
        );
        self.theorem(
            "TooLong",
            &args,
            Equality::boolean(
                matches(s.clone(), p.clone(), k.clone()),
                bc("And", vec![all.clone(), bound(s.clone(), p.clone())]),
                af("MatchStep", vec![s.clone(), p.clone(), k.clone()]),
            )
            .then(bool_right("And", all.clone(), bound_false))?
            .then(Equality::boolean(
                bc("And", vec![all, bit(false)]),
                bit(false),
                refl(boolean(), bit(false)),
            ))?,
        )?;
        let mut args = base;
        args.push(("empty", eq(nat(), size(p.clone()), zero())));
        let empty = Equality::natural(size(p.clone()), zero(), var("empty"));
        let b = nr(size(s.clone()), empty.clone()).then(Equality::boolean(
            less(size(s.clone()), zero()),
            bit(false),
            nc("LessZero", vec![size(s.clone())]),
        ))?;
        let i = var("_i");
        let impossible = nr(value(i.clone()), empty)
            .then(Equality::boolean(
                less(value(i.clone()), zero()),
                bit(false),
                nc("LessZero", vec![value(i.clone())]),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(value(i.clone()), size(p.clone())),
                bit(true),
                var("_within"),
            ))?;
        let target = same_byte(
            observed(
                s.clone(),
                source(s.clone(), p.clone(), k.clone(), i.clone()),
            ),
            observed(p.clone(), i.clone()),
        );
        let points = lam(
            "_i",
            word64(),
            lam(
                "_within",
                within(p.clone(), i),
                bc("FalseElim", vec![target, impossible.proof]),
            ),
        );
        let r = pc(
            "Make",
            vec![
                truth(less(size(s.clone()), size(p.clone())), false),
                content(s.clone(), p.clone(), k.clone()),
                b.proof,
                points,
            ],
        );
        self.theorem(
            "Empty",
            &args,
            Equality::boolean(
                matches(s.clone(), p.clone(), k.clone()),
                bit(true),
                af("Complete", vec![s, p, k, r]),
            ),
        )
    }
    fn self_match(&mut self) -> Result<()> {
        let (s, k) = (var("s"), var("k"));
        let base = [("s", string_type()), ("k", boolean())];
        let mut branches = vec![];
        for yes in [false, true] {
            let diff = sub(size(s.clone()), size(s.clone()));
            let choice = word::choose(k.clone(), diff.clone(), zero(), nat());
            let e = Equality::natural(
                offset_nat(s.clone(), s.clone(), k.clone()),
                choice.clone(),
                af("OffsetNatStep", vec![s.clone(), s.clone(), k.clone()]),
            )
            .then(Equality::natural(
                choice,
                if yes { diff.clone() } else { zero() },
                sc(
                    if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                    vec![nat(), k.clone(), diff.clone(), zero(), var("_branch")],
                ),
            ))?;
            branches.push(if yes {
                e.then(Equality::natural(
                    diff,
                    zero(),
                    nc("SubSame", vec![size(s.clone())]),
                ))?
                .proof
            } else {
                e.proof
            });
        }
        let target = eq(nat(), offset_nat(s.clone(), s.clone(), k.clone()), zero());
        self.emit(
            "SelfOffset",
            &base,
            target.clone(),
            decide(k.clone(), target, branches[0].clone(), branches[1].clone()),
            true,
        )?;
        let i = var("_i");
        let bits = address_bits(&BITS[..3]);
        let b = nc("LessIrreflexive", vec![size(s.clone())]);
        let values = index_value(
            s.clone(),
            s.clone(),
            k.clone(),
            b.clone(),
            i.clone(),
            var("_within"),
        )
        .then(
            Equality::natural(
                offset_nat(s.clone(), s.clone(), k.clone()),
                zero(),
                af("SelfOffset", vec![s.clone(), k.clone()]),
            )
            .on_left(value(i.clone())),
        )?
        .then(Equality::natural(
            add(zero(), value(i.clone())),
            value(i.clone()),
            nc("ZeroAdd", vec![value(i.clone())]),
        ))?;
        let bytes = byte_value(
            s.clone(),
            source(s.clone(), s.clone(), k.clone(), i.clone()),
            i.clone(),
            values,
        );
        let points = lam(
            "_i",
            word64(),
            lam(
                "_within",
                within(s.clone(), i),
                cube(&BITS[..3], at_bit(bytes, bits).proof, true),
            ),
        );
        let r = pc(
            "Make",
            vec![
                truth(less(size(s.clone()), size(s.clone())), false),
                content(s.clone(), s.clone(), k.clone()),
                b,
                points,
            ],
        );
        self.theorem(
            "Reflexive",
            &base,
            Equality::boolean(
                matches(s.clone(), s.clone(), k.clone()),
                bit(true),
                af("Complete", vec![s.clone(), s, k, r]),
            ),
        )
    }
    fn observations_api(&mut self) -> Result<()> {
        let (s, p, i) = (var("s"), var("p"), var("i"));
        for suffix in [false, true] {
            let prefix = if suffix { "Suffix" } else { "Prefix" };
            let name = if suffix { "HasSuffix" } else { "HasPrefix" };
            let k = bit(suffix);
            let base = vec![("s", string_type()), ("p", string_type())];
            self.theorem(
                &format!("{prefix}Offset"),
                &base,
                offset_literal(s.clone(), p.clone(), suffix)?,
            )?;
            let mut args = base.clone();
            args.extend([
                ("h", truth(af(name, vec![s.clone(), p.clone()]), true)),
                ("i", word64()),
                ("within", within(p.clone(), i.clone())),
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
            let bound = af(
                "MatchedBound",
                vec![s.clone(), p.clone(), k.clone(), h.proof],
            );
            let position = index_value(
                s.clone(),
                p.clone(),
                k.clone(),
                bound,
                i.clone(),
                var("within"),
            )
            .then(
                Equality::natural(
                    offset_nat(s.clone(), p.clone(), k.clone()),
                    if suffix {
                        sub(size(s.clone()), size(p.clone()))
                    } else {
                        zero()
                    },
                    af(&format!("{prefix}Offset"), vec![s.clone(), p.clone()]),
                )
                .on_left(value(i.clone())),
            )?;
            let position = if suffix {
                position
            } else {
                position.then(Equality::natural(
                    add(zero(), value(i.clone())),
                    value(i.clone()),
                    nc("ZeroAdd", vec![value(i.clone())]),
                ))?
            };
            self.theorem(&format!("{prefix}Position"), &args, position)?;
            let mut args = base.clone();
            args.extend([
                ("upper", upper(p.clone())),
                ("h", truth(af(name, vec![s.clone(), p.clone()]), true)),
                ("i", word64()),
                ("within", within(p.clone(), i.clone())),
            ]);
            let r = af(
                &format!("{prefix}Reflect"),
                vec![s.clone(), p.clone(), var("upper"), var("h")],
            );
            self.emit(
                &format!("{prefix}Bytes"),
                &args,
                same_byte(
                    observed(
                        s.clone(),
                        source(s.clone(), p.clone(), k.clone(), i.clone()),
                    ),
                    observed(p.clone(), i.clone()),
                ),
                relation_content(s.clone(), p.clone(), k, r, i.clone(), var("within")),
                true,
            )?;
        }
        Ok(())
    }
    fn concatenation(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let r = concat(s.clone(), t.clone());
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("fit", truth(cc("Fits", vec![s.clone(), t.clone()]), true)),
        ];
        let sum = add(size(s.clone()), size(t.clone()));
        let size_eq = Equality::natural(
            size(r.clone()),
            sum.clone(),
            cc("Size", vec![s.clone(), t.clone(), var("fit")]),
        );
        for suffix in [false, true] {
            let p = if suffix { t.clone() } else { s.clone() };
            let other = if suffix { s.clone() } else { t.clone() };
            let k = bit(suffix);
            let b = nl(size_eq.clone(), size(p.clone()));
            let b = if suffix {
                b.then(nl(
                    Equality::natural(
                        sum.clone(),
                        add(size(t.clone()), size(s.clone())),
                        nc("AddComm", vec![size(s.clone()), size(t.clone())]),
                    ),
                    size(p.clone()),
                ))?
            } else {
                b
            };
            let b = b.then(Equality::boolean(
                less(add(size(p.clone()), size(other.clone())), size(p.clone())),
                bit(false),
                call(
                    "Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",
                    vec![size(p.clone()), size(other)],
                ),
            ))?;
            let off = offset_literal(r.clone(), p.clone(), suffix)?;
            let off = if suffix {
                off.then(size_eq.clone().mapped(
                    lam("_n", nat(), sub(var("_n"), size(t.clone()))),
                    sub(size(r.clone()), size(t.clone())),
                    sub(sum.clone(), size(t.clone())),
                    nat(),
                ))?
                .then(Equality::natural(
                    sub(sum.clone(), size(t.clone())),
                    size(s.clone()),
                    nc("SubAddRight", vec![size(s.clone()), size(t.clone())]),
                ))?
            } else {
                off
            };
            let i = var("_i");
            let bits = address_bits(&BITS[..3]);
            let w = source(r.clone(), p.clone(), k.clone(), i.clone());
            let values = index_value(
                r.clone(),
                p.clone(),
                k.clone(),
                b.proof.clone(),
                i.clone(),
                var("_within"),
            )
            .then(off.on_left(value(i.clone())))?;
            let inside = af(
                "IndexWithin",
                vec![
                    r.clone(),
                    p.clone(),
                    k.clone(),
                    b.proof.clone(),
                    i.clone(),
                    var("_within"),
                ],
            );
            let bytes = if suffix {
                let after = nl(values.clone(), size(s.clone())).then(Equality::boolean(
                    less(add(size(s.clone()), value(i.clone())), size(s.clone())),
                    bit(false),
                    call(
                        "Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",
                        vec![size(s.clone()), value(i.clone())],
                    ),
                ))?;
                let subword = wc("Sub", vec![w.clone(), length(s.clone())]);
                let subvalue = Equality::natural(
                    value(subword.clone()),
                    sub(value(w.clone()), size(s.clone())),
                    sl(
                        "SubValue",
                        vec![w.clone(), length(s.clone()), after.proof.clone()],
                    ),
                )
                .then(values.mapped(
                    lam("_n", nat(), sub(var("_n"), size(s.clone()))),
                    sub(value(w.clone()), size(s.clone())),
                    sub(add(size(s.clone()), value(i.clone())), size(s.clone())),
                    nat(),
                ))?
                .then(
                    Equality::natural(
                        add(size(s.clone()), value(i.clone())),
                        add(value(i.clone()), size(s.clone())),
                        nc("AddComm", vec![size(s.clone()), value(i.clone())]),
                    )
                    .mapped(
                        lam("_n", nat(), sub(var("_n"), size(s.clone()))),
                        sub(add(size(s.clone()), value(i.clone())), size(s.clone())),
                        sub(add(value(i.clone()), size(s.clone())), size(s.clone())),
                        nat(),
                    ),
                )?
                .then(Equality::natural(
                    sub(add(value(i.clone()), size(s.clone())), size(s.clone())),
                    value(i.clone()),
                    nc("SubAddRight", vec![value(i.clone()), size(s.clone())]),
                ))?;
                Equality {
                    ty: byte(),
                    left: observed(r.clone(), w.clone()),
                    right: observed(t.clone(), subword.clone()),
                    proof: cc(
                        "ReadRight",
                        vec![s.clone(), t.clone(), var("fit"), w, inside, after.proof],
                    ),
                }
                .then(byte_value(t.clone(), subword, i.clone(), subvalue))?
            } else {
                let values = values.then(Equality::natural(
                    add(zero(), value(i.clone())),
                    value(i.clone()),
                    nc("ZeroAdd", vec![value(i.clone())]),
                ))?;
                let inside_i = nl(values.clone().reverse(), size(r.clone())).then(
                    Equality::boolean(less(value(w.clone()), size(r.clone())), bit(true), inside),
                )?;
                byte_value(r.clone(), w, i.clone(), values).then(Equality {
                    ty: byte(),
                    left: observed(r.clone(), i.clone()),
                    right: observed(s.clone(), i.clone()),
                    proof: cc(
                        "ReadLeft",
                        vec![
                            s.clone(),
                            t.clone(),
                            var("fit"),
                            i.clone(),
                            inside_i.proof,
                            var("_within"),
                        ],
                    ),
                })?
            };
            let points = lam(
                "_i",
                word64(),
                lam(
                    "_within",
                    within(p.clone(), i),
                    cube(&BITS[..3], at_bit(bytes, bits).proof, true),
                ),
            );
            let rel = pc(
                "Make",
                vec![
                    truth(less(size(r.clone()), size(p.clone())), false),
                    content(r.clone(), p.clone(), k.clone()),
                    b.proof,
                    points,
                ],
            );
            let complete = af("Complete", vec![r.clone(), p.clone(), k.clone(), rel]);
            let name = if suffix { "HasSuffix" } else { "HasPrefix" };
            let step = if suffix { "SuffixStep" } else { "PrefixStep" };
            self.theorem(
                if suffix {
                    "SuffixOfConcat"
                } else {
                    "PrefixOfConcat"
                },
                &args,
                Equality::boolean(
                    af(name, vec![r.clone(), p.clone()]),
                    matches(r.clone(), p.clone(), k),
                    af(step, vec![r.clone(), p]),
                )
                .then(Equality::boolean(
                    matches(
                        r.clone(),
                        if suffix { t.clone() } else { s.clone() },
                        bit(suffix),
                    ),
                    bit(true),
                    complete,
                ))?,
            )?;
        }
        Ok(())
    }
}
