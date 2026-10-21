//! Complete 256-slot output, full-width length and checked source positions.
use super::guards::{defined, high, ordered};
use super::*;

pub(super) fn result(s: Expr, a: Expr, b: Expr) -> Expr {
    sl("Value", vec![s, a, b])
}
pub(super) fn difference(a: Expr, b: Expr) -> Expr {
    sub(value(b), value(a))
}
pub(super) fn source(a: Expr, at: Vec<Expr>) -> Expr {
    let mut args = vec![a];
    args.extend(at);
    sl("SourceWord", args)
}
pub(super) fn slot(s: Expr, at: Vec<Expr>) -> Expr {
    app(data(s), at)
}
fn contents(s: Expr, a: Expr, b: Expr) -> Expr {
    sl("Data", vec![s, a, b])
}
pub(super) fn call_at(
    name: &str,
    s: Expr,
    a: Expr,
    b: Expr,
    at: Vec<Expr>,
    tail: Vec<Expr>,
) -> Expr {
    let mut args = vec![s, a, b];
    args.extend(at);
    args.extend(tail);
    sl(name, args)
}
pub(super) fn source_value(a: Expr, b: Expr, at: Vec<Expr>, order: Expr, inside: Expr) -> Equality {
    let mut args = vec![a.clone(), b];
    args.extend(at.clone());
    args.extend([order, inside]);
    Equality::natural(
        value(source(a.clone(), at.clone())),
        add(value(a), index(at)),
        sl("SourceIndexValue", args),
    )
}
pub(super) fn size_result(s: Expr, a: Expr, b: Expr, h: Expr) -> Equality {
    Equality::natural(
        size(result(s.clone(), a.clone(), b.clone())),
        difference(a.clone(), b.clone()),
        sl("Size", vec![s, a, b, h]),
    )
}
impl Slice<'_> {
    pub(super) fn carrier(&mut self) -> Result<()> {
        let (s, a, b) = (var("s"), var("a"), var("b"));
        let base = [("s", string_type()), ("a", word64()), ("b", word64())];
        let at = address_bits(&SLOTS);
        let mut source_args = vec![("a", word64())];
        source_args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        self.emit(
            "SourceWord",
            &source_args,
            word64(),
            wc("Add", vec![a.clone(), iw(at.clone())]),
            false,
        )?;
        let diff = difference(a.clone(), b.clone());
        let flag = less(index(at.clone()), diff.clone());
        let read = observed(s.clone(), source(a.clone(), at.clone()));
        let chosen = word::choose(flag.clone(), read.clone(), zero_word(8), byte());
        self.emit(
            "Data",
            &base,
            slots(),
            cube(&SLOTS, chosen.clone(), true),
            false,
        )?;
        let len = wc("Sub", vec![b.clone(), a.clone()]);
        let res = result(s.clone(), a.clone(), b.clone());
        let ds = contents(s.clone(), a.clone(), b.clone());
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
        self.theorem(
            "SizeRaw",
            &base,
            Equality {
                ty: word64(),
                left: length(res.clone()),
                right: len.clone(),
                proof: sl("Length", vec![s.clone(), a.clone(), b.clone()]),
            }
            .mapped(
                lam("_length", word64(), value(var("_length"))),
                size(res.clone()),
                value(len.clone()),
                nat(),
            ),
        )?;
        let mut guarded = base.to_vec();
        guarded.push(("h", truth(defined(s.clone(), a.clone(), b.clone()), true)));
        self.theorem(
            "Size",
            &guarded,
            Equality::natural(
                size(res.clone()),
                value(len.clone()),
                sl("SizeRaw", vec![s.clone(), a.clone(), b.clone()]),
            )
            .then(Equality::natural(
                value(len.clone()),
                diff.clone(),
                sl(
                    "SubValue",
                    vec![
                        b.clone(),
                        a.clone(),
                        ordered(s.clone(), a.clone(), b.clone(), var("h")),
                    ],
                ),
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
            let mut bound = args.clone();
            bound.push(("branch", truth(flag.clone(), yes)));
            let want = if yes { read.clone() } else { zero_word(8) };
            self.theorem(
                if yes { "Inside" } else { "Outside" },
                &bound,
                step.clone().then(Equality {
                    ty: byte(),
                    left: chosen.clone(),
                    right: want,
                    proof: sc(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            byte(),
                            flag.clone(),
                            read.clone(),
                            zero_word(8),
                            var("branch"),
                        ],
                    ),
                })?,
            )?;
        }
        self.source_geometry()?;
        self.valid_output()?;
        self.read_output()
    }
    fn source_geometry(&mut self) -> Result<()> {
        let (a, b, s) = (var("a"), var("b"), var("s"));
        let at = address_bits(&SLOTS);
        let (av, bv, i) = (value(a.clone()), value(b.clone()), index(at.clone()));
        let mut args = vec![("a", word64()), ("b", word64())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.extend([
            ("ordered", truth(less(bv.clone(), av.clone()), false)),
            (
                "within",
                truth(less(i.clone(), difference(a.clone(), b.clone())), true),
            ),
        ]);
        let upper = call(
            "Mpk.Go.Practical.String.Order.NatAsym",
            vec![
                bv.clone(),
                modulus(),
                ord("W64.ValueBound", vec![b.clone()]),
            ],
        );
        let small = call(
            "Mpk.Go.Practical.Word.Index.UpperBound",
            vec![
                modulus(),
                bv.clone(),
                add(av.clone(), i.clone()),
                upper,
                sl(
                    "IndexBeforeHigh",
                    vec![
                        av.clone(),
                        bv.clone(),
                        i.clone(),
                        var("ordered"),
                        var("within"),
                    ],
                ),
            ],
        );
        let sum = Equality::natural(
            value(iw(at.clone())),
            i.clone(),
            sl("IndexValue", at.clone()),
        )
        .on_right(av.clone());
        let actual_small = nl(sum.clone(), modulus()).then(Equality::boolean(
            less(add(av.clone(), i.clone()), modulus()),
            bit(true),
            small,
        ))?;
        self.theorem(
            "SourceIndexValue",
            &args,
            Equality::natural(
                value(source(a.clone(), at.clone())),
                add(av.clone(), value(iw(at.clone()))),
                sl(
                    "AddValue",
                    vec![a.clone(), iw(at.clone()), actual_small.proof],
                ),
            )
            .then(sum)?,
        )?;
        let mut args = vec![("s", string_type()), ("a", word64()), ("b", word64())];
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.extend([
            ("h", truth(defined(s.clone(), a.clone(), b.clone()), true)),
            (
                "within",
                truth(less(i.clone(), difference(a.clone(), b.clone())), true),
            ),
        ]);
        let order = ordered(s.clone(), a.clone(), b.clone(), var("h"));
        self.theorem(
            "SourceWithin",
            &args,
            nl(
                source_value(a.clone(), b.clone(), at, order.clone(), var("within")),
                size(s.clone()),
            )
            .then(Equality::boolean(
                less(add(av.clone(), i.clone()), size(s.clone())),
                bit(true),
                sl(
                    "IndexWithinSource",
                    vec![
                        av,
                        bv,
                        size(s.clone()),
                        i,
                        order,
                        high(s, a, b, var("h")),
                        var("within"),
                    ],
                ),
            ))?,
        )
    }
    fn valid_output(&mut self) -> Result<()> {
        let (s, a, b) = (var("s"), var("a"), var("b"));
        let res = result(s.clone(), a.clone(), b.clone());
        let diff = difference(a.clone(), b.clone());
        let mut args = vec![
            ("s", string_type()),
            ("a", word64()),
            ("b", word64()),
            ("h", truth(defined(s.clone(), a.clone(), b.clone()), true)),
        ];
        let at = address_bits(&SLOTS);
        let outside = nr(
            index(at.clone()),
            size_result(s.clone(), a.clone(), b.clone(), var("h")),
        )
        .reverse()
        .then(Equality::boolean(
            less(index(at.clone()), size(res.clone())),
            bit(false),
            var("_outside"),
        ))?;
        let proof = cube(
            &SLOTS,
            lam(
                "_outside",
                truth(less(index(at.clone()), size(res.clone())), false),
                call_at(
                    "Outside",
                    s.clone(),
                    a.clone(),
                    b.clone(),
                    at,
                    vec![outside.proof],
                ),
            ),
            true,
        );
        self.emit("Padding", &args, padding(res.clone()), proof, true)?;
        args.push(("upper", upper(s.clone())));
        let d_upper = sl(
            "DifferenceUpper",
            vec![
                value(a.clone()),
                value(b.clone()),
                ordered(s.clone(), a.clone(), b.clone(), var("h")),
            ],
        );
        let source_upper = sl(
            "UpperTrans",
            vec![
                diff.clone(),
                value(b.clone()),
                size(s.clone()),
                d_upper,
                high(s.clone(), a.clone(), b.clone(), var("h")),
            ],
        );
        let cap_upper = sl(
            "UpperTrans",
            vec![
                diff.clone(),
                size(s.clone()),
                capacity(),
                source_upper,
                var("upper"),
            ],
        );
        self.theorem(
            "Upper",
            &args,
            nr(
                capacity(),
                size_result(s.clone(), a.clone(), b.clone(), var("h")),
            )
            .then(Equality::boolean(
                less(capacity(), diff),
                bit(false),
                cap_upper,
            ))?,
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
                    sl(
                        "Upper",
                        vec![s.clone(), a.clone(), b.clone(), var("h"), var("upper")],
                    ),
                    sl("Padding", vec![s, a, b, var("h")]),
                ],
            ),
            true,
        )
    }
    fn read_output(&mut self) -> Result<()> {
        let (s, a, b, i) = (var("s"), var("a"), var("b"), var("i"));
        let res = result(s.clone(), a.clone(), b.clone());
        let at = (0..8)
            .rev()
            .map(|k| word::at(i.clone(), 64, k))
            .collect::<Result<Vec<_>>>()?;
        let args = [
            ("s", string_type()),
            ("a", word64()),
            ("b", word64()),
            ("h", truth(defined(s.clone(), a.clone(), b.clone()), true)),
            ("upper", upper(s.clone())),
            ("i", word64()),
            ("within", within(res.clone(), i.clone())),
        ];
        let cap = st(
            "WithinCapacity",
            vec![
                res.clone(),
                i.clone(),
                sl(
                    "Upper",
                    vec![s.clone(), a.clone(), b.clone(), var("h"), var("upper")],
                ),
                var("within"),
            ],
        );
        let address = Equality::natural(
            value(i.clone()),
            index(at.clone()),
            ix("BoundedIndex", vec![i.clone(), cap]),
        );
        let inside = nr(
            value(i.clone()),
            size_result(s.clone(), a.clone(), b.clone(), var("h")),
        )
        .reverse()
        .then(Equality::boolean(
            less(value(i.clone()), size(res.clone())),
            bit(true),
            var("within"),
        ))?;
        let slot_inside =
            nl(address.clone().reverse(), difference(a.clone(), b.clone())).then(inside)?;
        self.theorem(
            "Read",
            &args,
            Equality {
                ty: byte(),
                left: observed(res.clone(), i.clone()),
                right: slot(res.clone(), at.clone()),
                proof: st(
                    "ByteWithinSlot",
                    vec![
                        res.clone(),
                        i.clone(),
                        sl(
                            "Upper",
                            vec![s.clone(), a.clone(), b.clone(), var("h"), var("upper")],
                        ),
                        var("within"),
                    ],
                ),
            }
            .then(Equality {
                ty: byte(),
                left: slot(res, at.clone()),
                right: observed(s.clone(), source(a.clone(), at.clone())),
                proof: call_at(
                    "Inside",
                    s.clone(),
                    a.clone(),
                    b.clone(),
                    at.clone(),
                    vec![slot_inside.proof.clone()],
                ),
            })?,
        )?;
        self.theorem(
            "ReadIndexValue",
            &args,
            source_value(
                a.clone(),
                b.clone(),
                at,
                ordered(s, a.clone(), b, var("h")),
                slot_inside.proof,
            )
            .then(address.reverse().on_right(value(a)))?,
        )
    }
}
