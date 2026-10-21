//! Preservation of existing observable byte equivalence, including dirty tails.
use super::carrier::{call_at, difference, result, size_result, slot, source};
use super::guards::{defined, ec, high, ordered};
use super::*;
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
    let at = address_bits(&BITS[..3]);
    cube(
        &BITS[..3],
        eq(boolean(), app(x, at.clone()), app(y, at)),
        false,
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
fn at_bit(e: Equality, at: Vec<Expr>) -> Equality {
    let (l, r) = (
        app(e.left.clone(), at.clone()),
        app(e.right.clone(), at.clone()),
    );
    e.mapped(lam("_byte", byte(), app(var("_byte"), at)), l, r, boolean())
}
fn nz(name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("Mpk.Go.Practical.String.Normalization.{name}"),
        args,
    )
}

impl Slice<'_> {
    pub(super) fn preservation(&mut self) -> Result<()> {
        self.defined_preservation()?;
        self.byte_preservation()?;
        self.slice_preservation()
    }
    fn defined_preservation(&mut self) -> Result<()> {
        let (s, t, a, b) = (var("s"), var("t"), var("a"), var("b"));
        let base = [
            ("s", string_type()),
            ("t", string_type()),
            ("e", same(s.clone(), t.clone())),
            ("a", word64()),
            ("b", word64()),
        ];
        let mut args = base.to_vec();
        args.push(("h", truth(defined(s.clone(), a.clone(), b.clone()), true)));
        let upper = nl(
            sizes(s.clone(), t.clone(), var("e")).reverse(),
            value(b.clone()),
        )
        .then(Equality::boolean(
            less(size(s.clone()), value(b.clone())),
            bit(false),
            high(s.clone(), a.clone(), b.clone(), var("h")),
        ))?;
        self.theorem(
            "DefinedTransfer",
            &args,
            Equality::boolean(
                defined(t.clone(), a.clone(), b.clone()),
                bit(true),
                sl(
                    "DefinedComplete",
                    vec![
                        t.clone(),
                        a.clone(),
                        b.clone(),
                        sl(
                            "DefinedLowSign",
                            vec![s.clone(), a.clone(), b.clone(), var("h")],
                        ),
                        sl(
                            "DefinedHighSign",
                            vec![s.clone(), a.clone(), b.clone(), var("h")],
                        ),
                        ordered(s.clone(), a.clone(), b.clone(), var("h")),
                        upper.proof,
                    ],
                ),
            ),
        )?;
        self.theorem(
            "DefinedRespects",
            &base,
            Equality::boolean(
                defined(s.clone(), a.clone(), b.clone()),
                defined(t.clone(), a.clone(), b.clone()),
                ec(
                    "TruthIff",
                    vec![
                        defined(s.clone(), a.clone(), b.clone()),
                        defined(t.clone(), a.clone(), b.clone()),
                        lam(
                            "_h",
                            truth(defined(s.clone(), a.clone(), b.clone()), true),
                            sl(
                                "DefinedTransfer",
                                vec![
                                    s.clone(),
                                    t.clone(),
                                    var("e"),
                                    a.clone(),
                                    b.clone(),
                                    var("_h"),
                                ],
                            ),
                        ),
                        lam(
                            "_h",
                            truth(defined(t.clone(), a.clone(), b.clone()), true),
                            sl(
                                "DefinedTransfer",
                                vec![
                                    t.clone(),
                                    s.clone(),
                                    ec("EquivalentSym", vec![s, t, var("e")]),
                                    a,
                                    b,
                                    var("_h"),
                                ],
                            ),
                        ),
                    ],
                ),
            ),
        )
    }
    fn byte_preservation(&mut self) -> Result<()> {
        let (s, t, w) = (var("s"), var("t"), var("w"));
        let args = [
            ("s", string_type()),
            ("t", string_type()),
            ("e", same(s.clone(), t.clone())),
            ("w", word64()),
            ("upper", upper(s.clone())),
            ("within", within(s.clone(), w.clone())),
        ];
        let at = (0..8)
            .rev()
            .map(|k| word::at(w.clone(), 64, k))
            .collect::<Result<Vec<_>>>()?;
        let bits = address_bits(&BITS[..3]);
        let length = sizes(s.clone(), t.clone(), var("e"));
        let upper_t = nr(capacity(), length.clone().reverse()).then(Equality::boolean(
            less(capacity(), size(s.clone())),
            bit(false),
            var("upper"),
        ))?;
        let within_t = nr(value(w.clone()), length.reverse()).then(Equality::boolean(
            less(value(w.clone()), size(s.clone())),
            bit(true),
            var("within"),
        ))?;
        let bound = st(
            "WithinCapacity",
            vec![s.clone(), w.clone(), var("upper"), var("within")],
        );
        let full = Equality::natural(
            value(w.clone()),
            index(at.clone()),
            ix("BoundedIndex", vec![w.clone(), bound]),
        );
        let inside = nl(full.reverse(), size(s.clone())).then(Equality::boolean(
            less(value(w.clone()), size(s.clone())),
            bit(true),
            var("within"),
        ))?;
        let mut location = at.clone();
        location.push(inside.proof);
        location.extend(bits.clone());
        let middle = Equality::boolean(
            app(slot(s.clone(), at.clone()), bits.clone()),
            app(slot(t.clone(), at.clone()), bits.clone()),
            app(
                ec("EquivalentContents", vec![s.clone(), t.clone(), var("e")]),
                location,
            ),
        );
        let left = Equality {
            ty: byte(),
            left: observed(s.clone(), w.clone()),
            right: slot(s.clone(), at.clone()),
            proof: st(
                "ByteWithinSlot",
                vec![s.clone(), w.clone(), var("upper"), var("within")],
            ),
        };
        let right = Equality {
            ty: byte(),
            left: observed(t.clone(), w.clone()),
            right: slot(t.clone(), at),
            proof: st(
                "ByteWithinSlot",
                vec![t.clone(), w.clone(), upper_t.proof, within_t.proof],
            ),
        };
        self.emit(
            "ByteSameAt",
            &args,
            same_byte(observed(s, w.clone()), observed(t, w)),
            cube(
                &BITS[..3],
                at_bit(left, bits.clone())
                    .then(middle)?
                    .then(at_bit(right, bits).reverse())?
                    .proof,
                true,
            ),
            true,
        )
    }
    fn slice_preservation(&mut self) -> Result<()> {
        let (s, t, a, b) = (var("s"), var("t"), var("a"), var("b"));
        let base = vec![
            ("s", string_type()),
            ("t", string_type()),
            ("a", word64()),
            ("b", word64()),
            ("e", same(s.clone(), t.clone())),
            ("h", truth(defined(s.clone(), a.clone(), b.clone()), true)),
            ("upper", upper(s.clone())),
        ];
        let at = address_bits(&SLOTS);
        let bits = address_bits(&BITS[..3]);
        let (rs, rt) = (
            result(s.clone(), a.clone(), b.clone()),
            result(t.clone(), a.clone(), b.clone()),
        );
        let mut args = base.clone();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        args.push((
            "inside",
            truth(
                less(index(at.clone()), difference(a.clone(), b.clone())),
                true,
            ),
        ));
        let left = Equality {
            ty: byte(),
            left: slot(rs.clone(), at.clone()),
            right: observed(s.clone(), source(a.clone(), at.clone())),
            proof: call_at(
                "Inside",
                s.clone(),
                a.clone(),
                b.clone(),
                at.clone(),
                vec![var("inside")],
            ),
        };
        let right = Equality {
            ty: byte(),
            left: slot(rt.clone(), at.clone()),
            right: observed(t.clone(), source(a.clone(), at.clone())),
            proof: call_at(
                "Inside",
                t.clone(),
                a.clone(),
                b.clone(),
                at.clone(),
                vec![var("inside")],
            ),
        };
        let middle = Equality::boolean(
            app(left.right.clone(), bits.clone()),
            app(right.right.clone(), bits.clone()),
            app(
                sl(
                    "ByteSameAt",
                    vec![
                        s.clone(),
                        t.clone(),
                        var("e"),
                        source(a.clone(), at.clone()),
                        var("upper"),
                        call_at(
                            "SourceWithin",
                            s.clone(),
                            a.clone(),
                            b.clone(),
                            at.clone(),
                            vec![var("h"), var("inside")],
                        ),
                    ],
                ),
                bits.clone(),
            ),
        );
        self.emit(
            "RespectsInside",
            &args,
            same_byte(slot(rs.clone(), at.clone()), slot(rt.clone(), at.clone())),
            cube(
                &BITS[..3],
                at_bit(left, bits.clone())
                    .then(middle)?
                    .then(at_bit(right, bits).reverse())?
                    .proof,
                true,
            ),
            true,
        )?;
        let inside = nr(
            index(at.clone()),
            size_result(s.clone(), a.clone(), b.clone(), var("h")),
        )
        .reverse()
        .then(Equality::boolean(
            less(index(at.clone()), size(rs.clone())),
            bit(true),
            var("_inside"),
        ))?;
        let mut application = vec![
            s.clone(),
            t.clone(),
            a.clone(),
            b.clone(),
            var("e"),
            var("h"),
            var("upper"),
        ];
        application.extend(at.clone());
        application.push(inside.proof);
        let points = cube(
            &SLOTS,
            lam(
                "_inside",
                truth(less(index(at.clone()), size(rs.clone())), true),
                sl("RespectsInside", application),
            ),
            true,
        );
        let length = Equality::natural(
            size(rs.clone()),
            value(wc("Sub", vec![b.clone(), a.clone()])),
            sl("SizeRaw", vec![s.clone(), a.clone(), b.clone()]),
        )
        .then(
            Equality::natural(
                size(rt.clone()),
                value(wc("Sub", vec![b.clone(), a.clone()])),
                sl("SizeRaw", vec![t.clone(), a.clone(), b.clone()]),
            )
            .reverse(),
        )?;
        self.emit(
            "Respects",
            &base,
            same(rs.clone(), rt.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(rs), size(rt.clone())),
                    content(result(s.clone(), a.clone(), b.clone()), rt),
                    length.proof,
                    points,
                ],
            ),
            true,
        )?;
        let args = [
            ("s", string_type()),
            ("a", word64()),
            ("b", word64()),
            ("h", truth(defined(s.clone(), a.clone(), b.clone()), true)),
            ("upper", upper(s.clone())),
        ];
        let norm = nz("Value", vec![s.clone()]);
        let e = ec(
            "EquivalentSym",
            vec![
                norm.clone(),
                s.clone(),
                ec("NormalizationEquivalent", vec![s.clone()]),
            ],
        );
        self.emit(
            "InputNormalization",
            &args,
            same(
                result(norm.clone(), a.clone(), b.clone()),
                result(s.clone(), a.clone(), b.clone()),
            ),
            ec(
                "EquivalentSym",
                vec![
                    result(s.clone(), a.clone(), b.clone()),
                    result(norm, a.clone(), b.clone()),
                    sl(
                        "Respects",
                        vec![
                            s.clone(),
                            nz("Value", vec![s.clone()]),
                            a.clone(),
                            b.clone(),
                            e,
                            var("h"),
                            var("upper"),
                        ],
                    ),
                ],
            ),
            true,
        )?;
        let mut args = args.to_vec();
        args.extend(SLOTS.iter().map(|n| (*n, boolean())));
        let res = result(s.clone(), a.clone(), b.clone());
        let mut application = vec![res.clone(), sl("Padding", vec![s, a, b, var("h")])];
        application.extend(at.clone());
        self.theorem(
            "CanonicalSlot",
            &args,
            Equality {
                ty: byte(),
                left: slot(nz("Value", vec![res.clone()]), at.clone()),
                right: slot(res, at),
                proof: nz("StableSlot", application),
            },
        )
    }
}
