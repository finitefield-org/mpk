//! Frozen full-word/byte-slot strings, with explicit length and padding facts.
//! These are ordinary data/proofs, not source evaluation or nominal identity.
use super::polynomial::mul_right;
use super::syntax::*;
use super::{word, Builder, Result};

const SLOTS: [&str; 8] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"];
const BITS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];
fn st(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn ix(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Index.D8.{name}"), args)
}
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Sequence.{name}"), args)
}
fn byte() -> Expr {
    word::word_type(8).expect("frozen byte width")
}
fn word64() -> Expr {
    word::word_type(64).expect("frozen length width")
}
fn cube(names: &[&str], mut element: Expr, lambda: bool) -> Expr {
    for name in names.iter().rev() {
        element = if lambda {
            lam(name, boolean(), element)
        } else {
            pi(name, boolean(), element)
        };
    }
    element
}
fn slots() -> Expr {
    cube(&SLOTS, byte(), false)
}
fn product(a: Expr, b: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call("Std.Bool.recType", vec![a, b, var("_tag")]),
    )
}
fn string_type() -> Expr {
    product(word64(), slots())
}
fn zero_word(width: usize) -> Expr {
    cube(
        &word::ADDRESSES[..word::depth(width).expect("frozen width")],
        bit(false),
        true,
    )
}
fn zero_slots() -> Expr {
    cube(&SLOTS, zero_word(8), true)
}
fn make(length: Expr, data: Expr) -> Expr {
    st("Make", vec![length, data])
}
fn length(s: Expr) -> Expr {
    st("Length", vec![s])
}
fn data(s: Expr) -> Expr {
    st("Data", vec![s])
}
fn value(w: Expr) -> Expr {
    call("Mpk.Go.Practical.Word.Numeric.Value6", vec![w])
}
fn numeric(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Numeric.{name}"), args)
}
fn size(s: Expr) -> Expr {
    st("Size", vec![s])
}
fn capacity() -> Expr {
    call("Mpk.Go.Practical.FiniteVector.D8.Capacity", vec![])
}
fn upper(s: Expr) -> Expr {
    eq(boolean(), less(capacity(), size(s)), bit(false))
}
fn within(s: Expr, i: Expr) -> Expr {
    eq(boolean(), less(value(i), size(s)), bit(true))
}
fn index(address: Vec<Expr>) -> Expr {
    call("Mpk.Go.Practical.FiniteVector.D8.Index", address)
}
fn address_bits(names: &[&str]) -> Vec<Expr> {
    names.iter().map(|name| var(name)).collect()
}
fn padding(s: Expr) -> Expr {
    let address = address_bits(&SLOTS);
    cube(
        &SLOTS,
        pi(
            "_outside",
            eq(
                boolean(),
                less(index(address.clone()), size(s.clone())),
                bit(false),
            ),
            eq(byte(), app(data(s), address), zero_word(8)),
        ),
        false,
    )
}
fn valid(s: Expr) -> Expr {
    st("Valid", vec![s])
}
fn sequence_type() -> Expr {
    product(nat(), pi("_index", nat(), byte()))
}
fn sequence(s: Expr) -> Expr {
    st("Sequence", vec![s])
}
fn observed(s: Expr, i: Expr) -> Expr {
    st("Byte", vec![s, i])
}
fn actual_slot(s: Expr, i: Expr) -> Result<Expr> {
    Ok(app(
        data(s),
        (0..8)
            .rev()
            .map(|bit| word::at(i.clone(), 64, bit))
            .collect::<Result<Vec<_>>>()?,
    ))
}
struct Strings<'a>(&'a mut Builder);
impl Strings<'_> {
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
            &format!("Mpk.Go.Practical.String.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        left: Expr,
        right: Expr,
        proof: Expr,
    ) -> Result<()> {
        self.emit(name, args, eq(ty, left, right), proof, true)
    }
    fn definitions(&mut self) -> Result<()> {
        for (name, ty) in [
            ("ByteType", byte()),
            ("LengthType", word64()),
            ("SlotsType", slots()),
            ("Type", string_type()),
        ] {
            self.emit(name, &[], Expr::Sort0, ty, false)?;
        }
        self.emit(
            "TypeAsFunction",
            &[("s", st("Type", vec![]))],
            string_type(),
            var("s"),
            false,
        )?;
        let input = [("length", word64()), ("data", slots())];
        self.emit(
            "Make",
            &input,
            string_type(),
            pc("Make", vec![word64(), slots(), var("length"), var("data")]),
            false,
        )?;
        let arg = [("s", string_type())];
        for (name, right, ty) in [("Length", false, word64()), ("Data", true, slots())] {
            self.emit(
                name,
                &arg,
                ty,
                pc(
                    if right { "Right" } else { "Left" },
                    vec![word64(), slots(), var("s")],
                ),
                false,
            )?;
        }
        self.emit("Size", &arg, nat(), value(length(var("s"))), false)?;
        self.emit(
            "Sequence",
            &arg,
            sequence_type(),
            ix("Sequence64", vec![byte(), data(var("s")), length(var("s"))]),
            false,
        )?;
        self.emit(
            "Byte",
            &[("s", string_type()), ("i", word64())],
            byte(),
            ix("Read64", vec![byte(), data(var("s")), var("i")]),
            false,
        )?;
        for (name, body) in [
            ("LengthFits", upper(var("s"))),
            ("Padding", padding(var("s"))),
            ("Valid", product(upper(var("s")), padding(var("s")))),
        ] {
            self.emit(name, &arg, Expr::Sort0, body, false)?;
        }
        self.emit(
            "PaddingAsFunction",
            &[
                ("s", string_type()),
                ("padding", st("Padding", vec![var("s")])),
            ],
            padding(var("s")),
            var("padding"),
            false,
        )?;
        let mut args = vec![("s", string_type())];
        args.extend(BITS.iter().map(|name| (*name, boolean())));
        self.emit(
            "LengthBit",
            &args,
            boolean(),
            app(length(var("s")), address_bits(&BITS)),
            false,
        )?;
        let mut args = vec![("s", string_type())];
        args.extend(SLOTS.iter().map(|name| (*name, boolean())));
        args.extend(BITS[..3].iter().map(|name| (*name, boolean())));
        self.emit(
            "SlotBit",
            &args,
            boolean(),
            app(
                app(data(var("s")), address_bits(&SLOTS)),
                address_bits(&BITS[..3]),
            ),
            false,
        )?;
        self.emit(
            "Zero",
            &[],
            string_type(),
            make(zero_word(64), zero_slots()),
            false,
        )
    }
    fn constructors(&mut self) -> Result<()> {
        let args = [("length", word64()), ("data", slots())];
        let made = make(var("length"), var("data"));
        for (name, right, ty, want) in [
            ("MakeLength", false, word64(), var("length")),
            ("MakeData", true, slots(), var("data")),
        ] {
            self.theorem(
                name,
                &args,
                ty,
                if right {
                    data(made.clone())
                } else {
                    length(made.clone())
                },
                want,
                pc(
                    if right { "RightOfMake" } else { "LeftOfMake" },
                    vec![word64(), slots(), var("length"), var("data")],
                ),
            )?;
        }
        self.theorem(
            "MakeSize",
            &args,
            nat(),
            size(made.clone()),
            value(var("length")),
            call(
                "Std.Eq.congr",
                vec![
                    word64(),
                    nat(),
                    lam("_length", word64(), value(var("_length"))),
                    length(made.clone()),
                    var("length"),
                    st("MakeLength", vec![var("length"), var("data")]),
                ],
            ),
        )?;
        let mut bit_args = args.to_vec();
        bit_args.extend(BITS.iter().map(|name| (*name, boolean())));
        let mut left_args = vec![made.clone()];
        left_args.extend(address_bits(&BITS));
        let e = Equality {
            ty: word64(),
            left: length(made.clone()),
            right: var("length"),
            proof: st("MakeLength", vec![var("length"), var("data")]),
        }
        .mapped(
            lam("_word", word64(), app(var("_word"), address_bits(&BITS))),
            st("LengthBit", left_args),
            app(var("length"), address_bits(&BITS)),
            boolean(),
        );
        self.emit(
            "MakeLengthBit",
            &bit_args,
            eq(e.ty, e.left, e.right),
            e.proof,
            true,
        )?;
        let mut bit_args = args.to_vec();
        bit_args.extend(SLOTS.iter().map(|name| (*name, boolean())));
        bit_args.extend(BITS[..3].iter().map(|name| (*name, boolean())));
        let mut left_args = vec![made.clone()];
        left_args.extend(address_bits(&SLOTS));
        left_args.extend(address_bits(&BITS[..3]));
        let access = |slots| app(app(slots, address_bits(&SLOTS)), address_bits(&BITS[..3]));
        let e = Equality {
            ty: slots(),
            left: data(made.clone()),
            right: var("data"),
            proof: st("MakeData", vec![var("length"), var("data")]),
        }
        .mapped(
            lam("_slots", slots(), access(var("_slots"))),
            st("SlotBit", left_args),
            access(var("data")),
            boolean(),
        );
        self.emit(
            "MakeSlotBit",
            &bit_args,
            eq(e.ty, e.left, e.right),
            e.proof,
            true,
        )?;
        let mut claims = args.to_vec();
        claims.extend([
            ("upper", upper(made.clone())),
            ("padding", padding(made.clone())),
        ]);
        self.emit(
            "MakeValid",
            &claims,
            valid(made.clone()),
            pc(
                "Make",
                vec![
                    upper(made.clone()),
                    padding(made),
                    var("upper"),
                    var("padding"),
                ],
            ),
            true,
        )?;
        let s = var("s");
        for (name, right, ty) in [
            ("ValidUpper", false, upper(s.clone())),
            ("ValidPadding", true, padding(s.clone())),
        ] {
            self.emit(
                name,
                &[("s", string_type()), ("valid", valid(s.clone()))],
                ty,
                pc(
                    if right { "Right" } else { "Left" },
                    vec![upper(s.clone()), padding(s.clone()), var("valid")],
                ),
                true,
            )?;
        }
        let mut args = vec![("s", string_type()), ("valid", valid(s.clone()))];
        args.extend(SLOTS.iter().map(|name| (*name, boolean())));
        args.push((
            "outside",
            eq(
                boolean(),
                less(index(address_bits(&SLOTS)), size(s.clone())),
                bit(false),
            ),
        ));
        let mut proof_args = address_bits(&SLOTS);
        proof_args.push(var("outside"));
        self.theorem(
            "OutsidePadding",
            &args,
            byte(),
            app(data(s.clone()), address_bits(&SLOTS)),
            zero_word(8),
            app(st("ValidPadding", vec![s, var("valid")]), proof_args),
        )
    }
    fn observations(&mut self) -> Result<()> {
        let (s, i) = (var("s"), var("i"));
        let args = [("s", string_type()), ("i", word64())];
        self.theorem(
            "SequenceLength",
            &args[..1],
            nat(),
            sc("Length", vec![byte(), sequence(s.clone())]),
            size(s.clone()),
            ix(
                "FullLength",
                vec![byte(), data(s.clone()), length(s.clone())],
            ),
        )?;
        self.theorem(
            "SequenceByte",
            &args,
            byte(),
            sc("At", vec![byte(), sequence(s.clone()), value(i.clone())]),
            observed(s.clone(), i.clone()),
            ix(
                "SequenceRead",
                vec![byte(), data(s.clone()), length(s.clone()), i.clone()],
            ),
        )?;
        let mut guarded = args.to_vec();
        guarded.extend([
            ("upper", upper(s.clone())),
            ("within", within(s.clone(), i.clone())),
        ]);
        self.emit(
            "WithinCapacity",
            &guarded,
            eq(boolean(), less(value(i.clone()), capacity()), bit(true)),
            call(
                "Mpk.Go.Practical.Word.Index.UpperBound",
                vec![
                    capacity(),
                    size(s.clone()),
                    value(i.clone()),
                    var("upper"),
                    var("within"),
                ],
            ),
            true,
        )?;
        for sequence_read in [false, true] {
            self.theorem(
                if sequence_read {
                    "SequenceWithinSlot"
                } else {
                    "ByteWithinSlot"
                },
                &guarded,
                byte(),
                if sequence_read {
                    sc("At", vec![byte(), sequence(s.clone()), value(i.clone())])
                } else {
                    observed(s.clone(), i.clone())
                },
                actual_slot(s.clone(), i.clone())?,
                ix(
                    if sequence_read {
                        "SequenceWithinSlot"
                    } else {
                        "ReadWithinLength"
                    },
                    vec![
                        byte(),
                        data(s.clone()),
                        length(s.clone()),
                        i.clone(),
                        var("upper"),
                        var("within"),
                    ],
                ),
            )?;
        }
        let mut guarded = args.to_vec();
        guarded.extend([
            ("valid", valid(s.clone())),
            ("within", within(s.clone(), i.clone())),
        ]);
        self.emit(
            "ValidWithinCapacity",
            &guarded,
            eq(boolean(), less(value(i.clone()), capacity()), bit(true)),
            st(
                "WithinCapacity",
                vec![
                    s.clone(),
                    i.clone(),
                    st("ValidUpper", vec![s.clone(), var("valid")]),
                    var("within"),
                ],
            ),
            true,
        )?;
        for sequence_read in [false, true] {
            self.theorem(
                if sequence_read {
                    "ValidSequenceWithinSlot"
                } else {
                    "ValidByteWithinSlot"
                },
                &guarded,
                byte(),
                if sequence_read {
                    sc("At", vec![byte(), sequence(s.clone()), value(i.clone())])
                } else {
                    observed(s.clone(), i.clone())
                },
                actual_slot(s.clone(), i.clone())?,
                st(
                    if sequence_read {
                        "SequenceWithinSlot"
                    } else {
                        "ByteWithinSlot"
                    },
                    vec![
                        s.clone(),
                        i.clone(),
                        st("ValidUpper", vec![s.clone(), var("valid")]),
                        var("within"),
                    ],
                ),
            )?;
        }
        Ok(())
    }
    fn zero(&mut self) -> Result<()> {
        let z = st("Zero", vec![]);
        for (name, right, ty, want) in [
            ("ZeroLength", false, word64(), zero_word(64)),
            ("ZeroData", true, slots(), zero_slots()),
        ] {
            self.theorem(
                name,
                &[],
                ty,
                if right {
                    data(z.clone())
                } else {
                    length(z.clone())
                },
                want,
                st(
                    if right { "MakeData" } else { "MakeLength" },
                    vec![zero_word(64), zero_slots()],
                ),
            )?;
        }
        let mut proof = refl(byte(), zero_word(8));
        proof = lam(
            "_outside",
            eq(
                boolean(),
                less(index(address_bits(&SLOTS)), size(z.clone())),
                bit(false),
            ),
            proof,
        );
        self.emit(
            "ZeroPadding",
            &[],
            padding(z.clone()),
            cube(&SLOTS, proof, true),
            true,
        )?;
        // Unfold one actual balanced numeric node at a time. An implicit
        // computation of a nested recursor major is not an ordinary proof.
        for depth in 0..=6 {
            let all_zero = cube(&word::ADDRESSES[..depth], bit(false), true);
            let call_value = |x| numeric(&format!("Value{depth}"), vec![x]);
            let e = if depth == 0 {
                let bit_value = numeric("Bit", vec![bit(false)]);
                Equality::natural(
                    call_value(all_zero.clone()),
                    bit_value.clone(),
                    numeric("ValueDefinition0", vec![all_zero]),
                )
                .then(Equality::natural(bit_value, zero(), refl(nat(), zero())))?
            } else {
                let (lo, hi) = (
                    app(all_zero.clone(), vec![bit(false)]),
                    app(all_zero.clone(), vec![bit(true)]),
                );
                let subvalue = |x| numeric(&format!("Value{}", depth - 1), vec![x]);
                let weight = numeric(&format!("Weight{}", depth - 1), vec![]);
                let lower = subvalue(lo.clone());
                let higher = subvalue(hi.clone());
                let left = Equality::natural(
                    lower.clone(),
                    zero(),
                    st(&format!("ZeroCubeValue{}", depth - 1), vec![]),
                );
                let right = Equality::natural(
                    higher.clone(),
                    zero(),
                    st(&format!("ZeroCubeValue{}", depth - 1), vec![]),
                );
                Equality::natural(
                    call_value(all_zero.clone()),
                    add(lower, mul(weight.clone(), higher.clone())),
                    numeric(&format!("ValueDefinition{depth}"), vec![all_zero]),
                )
                .then(left.on_left(mul(weight.clone(), higher)))?
                .then(mul_right(weight.clone(), right).on_right(zero()))?
                .then(
                    Equality::natural(
                        mul(weight.clone(), zero()),
                        zero(),
                        nc("MulZero", vec![weight]),
                    )
                    .on_right(zero()),
                )?
                .then(Equality::natural(
                    add(zero(), zero()),
                    zero(),
                    nc("AddZero", vec![zero()]),
                ))?
            };
            self.emit(
                &format!("ZeroCubeValue{depth}"),
                &[],
                eq(e.ty, e.left, e.right),
                e.proof,
                true,
            )?;
        }
        let e = Equality {
            ty: word64(),
            left: length(z.clone()),
            right: zero_word(64),
            proof: st("ZeroLength", vec![]),
        }
        .mapped(
            lam("_zero_word", word64(), value(var("_zero_word"))),
            size(z.clone()),
            value(zero_word(64)),
            nat(),
        )
        .then(Equality::natural(
            value(zero_word(64)),
            zero(),
            st("ZeroCubeValue6", vec![]),
        ))?;
        self.emit("ZeroValue", &[], eq(e.ty, e.left, e.right), e.proof, true)?;
        let e = Equality::natural(size(z.clone()), zero(), st("ZeroValue", vec![]))
            .mapped(
                lam("_size", nat(), less(capacity(), var("_size"))),
                less(capacity(), size(z.clone())),
                less(capacity(), zero()),
                boolean(),
            )
            .then(Equality::boolean(
                less(capacity(), zero()),
                bit(false),
                refl(boolean(), bit(false)),
            ))?;
        self.emit("ZeroFits", &[], upper(z.clone()), e.proof, true)?;
        self.emit(
            "ZeroValid",
            &[],
            valid(z.clone()),
            pc(
                "Make",
                vec![
                    upper(z.clone()),
                    padding(z),
                    st("ZeroFits", vec![]),
                    st("ZeroPadding", vec![]),
                ],
            ),
            true,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut s = Strings(builder);
    s.definitions()?;
    s.constructors()?;
    s.observations()?;
    s.zero()
}

// Address order agrees exactly with the frozen most-significant-head cubes.
// Homogeneous subtrees are collapsed so canonical zero is literally all false.
fn tree(values: &[Expr], names: &[&str], element: Expr) -> Expr {
    if values.iter().all(|v| v == &values[0]) {
        return values[0].clone();
    }
    let middle = values.len() / 2;
    word::choose(
        var(names[0]),
        tree(&values[middle..], &names[1..], element.clone()),
        tree(&values[..middle], &names[1..], element.clone()),
        element,
    )
}
fn encoded_word(width: usize, number: u64) -> Expr {
    let names = &word::ADDRESSES[..word::depth(width).expect("fixed word")];
    cube(
        names,
        tree(
            &(0..width)
                .map(|bit_index| bit(number & (1 << bit_index) != 0))
                .collect::<Vec<_>>(),
            names,
            boolean(),
        ),
        true,
    )
}
fn slot_address(index: usize) -> Vec<Expr> {
    (0..8)
        .rev()
        .map(|shift| bit(index & (1 << shift) != 0))
        .collect()
}
pub(super) fn append_value(builder: &mut Builder, bytes: &[u8]) -> Result<()> {
    let length_word = encoded_word(64, bytes.len() as u64);
    let words = (0..256)
        .map(|i| encoded_word(8, u64::from(bytes.get(i).copied().unwrap_or(0))))
        .collect::<Vec<_>>();
    let data = cube(&SLOTS, tree(&words, &SLOTS, byte()), true);
    let value_name = "Mpk.Go.Practical.String.Value";
    emit(
        builder,
        value_name,
        &[],
        string_type(),
        make(length_word, data),
        false,
    )?;
    let v = constant(value_name);
    // Each conclusion names its expected atomic Boolean independently of the
    // constructed function tree. Both cores check every2112 exact observations.
    for position in 0..64 {
        let expected = bit(((bytes.len() as u64) >> position) & 1 != 0);
        let mut args = vec![v.clone()];
        args.extend(word::address(64, position)?);
        emit(
            builder,
            &format!("{value_name}.LengthBit{position}"),
            &[],
            eq(boolean(), st("LengthBit", args), expected.clone()),
            refl(boolean(), expected),
            true,
        )?;
    }
    for position in 0..256 {
        for bit_index in 0..8 {
            let expected = bit((bytes.get(position).copied().unwrap_or(0) >> bit_index) & 1 != 0);
            let mut args = vec![v.clone()];
            args.extend(slot_address(position));
            args.extend(word::address(8, bit_index)?);
            emit(
                builder,
                &format!("{value_name}.Slot{position}Bit{bit_index}"),
                &[],
                eq(boolean(), st("SlotBit", args), expected.clone()),
                refl(boolean(), expected),
                true,
            )?;
        }
    }
    Ok(())
}
