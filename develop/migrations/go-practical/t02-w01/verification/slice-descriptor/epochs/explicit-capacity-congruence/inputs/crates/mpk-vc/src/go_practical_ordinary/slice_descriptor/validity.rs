//! Header validity has real bounds and nil metadata; no live-store predicate.
use super::*;
impl Descriptors<'_> {
    pub(super) fn validity(&mut self) -> Result<()> {
        let s = var("s");
        let nil = dc("Nil", vec![]);
        self.emit(
            "CanonicalNil",
            &[("s", descriptor())],
            Expr::Sort0,
            same_fields(s.clone(), nil.clone()),
            false,
        )?;
        let bound = truth(
            word::wc(
                64,
                "UnsignedLess",
                vec![getter(5, s.clone()), getter(4, s.clone())],
            ),
            false,
        );
        self.emit(
            "HeaderBounds",
            &[("s", descriptor())],
            Expr::Sort0,
            bound.clone(),
            false,
        )?;
        let conditional = pi(
            "_nil",
            truth(getter(0, s.clone()), true),
            dc("CanonicalNil", vec![s.clone()]),
        );
        self.emit(
            "HeaderValid",
            &[("s", descriptor())],
            Expr::Sort0,
            product(&[bound.clone(), conditional.clone()]),
            false,
        )?;
        let valid = [
            ("s", descriptor()),
            ("h", dc("HeaderValid", vec![s.clone()])),
        ];
        self.emit(
            "HeaderBoundsOfValid",
            &valid,
            bound.clone(),
            project(&[bound.clone(), conditional.clone()], var("h"), 0),
            true,
        )?;
        let mut nil_valid = valid.to_vec();
        nil_valid.push(("is_nil", truth(getter(0, s.clone()), true)));
        self.emit(
            "HeaderNilCanonical",
            &nil_valid,
            dc("CanonicalNil", vec![s.clone()]),
            app(
                project(&[bound, conditional], var("h"), 1),
                vec![var("is_nil")],
            ),
            true,
        )?;
        let w = var("w");
        let e = Equality::boolean(
            word::wc(64, "UnsignedLess", vec![w.clone(), zero_word(64)]),
            less(value(w.clone()), value(zero_word(64))),
            call(
                "Mpk.Go.Practical.Word.Order.W64.UnsignedLessNatural",
                vec![w.clone(), zero_word(64)],
            ),
        );
        let change = Equality::natural(
            value(zero_word(64)),
            zero(),
            call("Mpk.Go.Practical.String.ZeroCubeValue6", vec![]),
        )
        .mapped(
            lam("_n", nat(), less(value(w.clone()), var("_n"))),
            less(value(w.clone()), value(zero_word(64))),
            less(value(w.clone()), zero()),
            boolean(),
        );
        self.theorem(
            "WordLessZero",
            &[("w", word_type(64))],
            e.then(change)?.then(Equality::boolean(
                less(value(w.clone()), zero()),
                bit(false),
                nc("LessZero", vec![value(w)]),
            ))?,
        )?;
        for ctor in ["Nil", "EmptyNonNilRaw"] {
            let s = dc(ctor, vec![]);
            let bound = truth(
                word::wc(
                    64,
                    "UnsignedLess",
                    vec![getter(5, s.clone()), getter(4, s.clone())],
                ),
                false,
            );
            let conditional = pi(
                "_nil",
                truth(getter(0, s.clone()), true),
                dc("CanonicalNil", vec![s.clone()]),
            );
            let canonical = if ctor == "Nil" {
                dc("SameFieldsReflexive", vec![s.clone()])
            } else {
                bc(
                    "FalseElim",
                    vec![dc("CanonicalNil", vec![s.clone()]), var("_nil")],
                )
            };
            let cap = getter(5, s.clone());
            let len = getter(4, s.clone());
            let z = zero_word(64);
            let bound_proof = Equality {
                ty: word_type(64),
                left: cap.clone(),
                right: z.clone(),
                proof: dc(&format!("{ctor}Capacity"), vec![]),
            }
            .mapped(
                lam(
                    "_word",
                    word_type(64),
                    word::wc(64, "UnsignedLess", vec![var("_word"), len.clone()]),
                ),
                word::wc(64, "UnsignedLess", vec![cap, len.clone()]),
                word::wc(64, "UnsignedLess", vec![z.clone(), len.clone()]),
                boolean(),
            )
            .then(
                Equality {
                    ty: word_type(64),
                    left: len.clone(),
                    right: z.clone(),
                    proof: dc(&format!("{ctor}Length"), vec![]),
                }
                .mapped(
                    lam(
                        "_word",
                        word_type(64),
                        word::wc(64, "UnsignedLess", vec![z.clone(), var("_word")]),
                    ),
                    word::wc(64, "UnsignedLess", vec![z.clone(), len]),
                    word::wc(64, "UnsignedLess", vec![z.clone(), z.clone()]),
                    boolean(),
                ),
            )?
            .then(Equality::boolean(
                word::wc(64, "UnsignedLess", vec![z.clone(), z.clone()]),
                bit(false),
                dc("WordLessZero", vec![z]),
            ))?;
            self.emit(
                &format!("{ctor}HeaderValid"),
                &[],
                dc("HeaderValid", vec![s]),
                make_product(
                    &[bound, conditional],
                    &[
                        bound_proof.proof,
                        lam("_nil", truth(getter(0, dc(ctor, vec![])), true), canonical),
                    ],
                ),
                true,
            )?;
        }
        for (i, ty) in fields().into_iter().enumerate() {
            let middle = getter(i, nil.clone());
            let right = zero_values(true)[i].clone();
            let e = Equality {
                ty: ty.clone(),
                left: getter(i, s.clone()),
                right: middle.clone(),
                proof: dc(
                    &format!("SameFields{}", GETTERS[i]),
                    vec![
                        s.clone(),
                        nil.clone(),
                        dc(
                            "HeaderNilCanonical",
                            vec![s.clone(), var("h"), var("is_nil")],
                        ),
                    ],
                ),
            }
            .then(Equality {
                ty,
                left: middle,
                right: right.clone(),
                proof: dc(&format!("Nil{}", GETTERS[i]), vec![]),
            })?;
            self.theorem(&format!("ValidNil{}", GETTERS[i]), &nil_valid, e)?;
        }
        self.emit(
            "CopyHeaderValid",
            &valid,
            dc("HeaderValid", vec![dc("Copy", vec![s.clone()])]),
            var("h"),
            true,
        )?;
        let args = [
            ("s", descriptor()),
            (
                "length_zero",
                eq(word_type(64), getter(4, s.clone()), zero_word(64)),
            ),
            ("i", word_type(64)),
        ];
        let x = var("i");
        let e = Equality {
            ty: word_type(64),
            left: getter(4, s.clone()),
            right: zero_word(64),
            proof: var("length_zero"),
        }
        .mapped(
            lam(
                "_length",
                word_type(64),
                word::wc(64, "UnsignedLess", vec![x.clone(), var("_length")]),
            ),
            dc("IndexUnsigned", vec![s.clone(), x.clone()]),
            word::wc(64, "UnsignedLess", vec![x.clone(), zero_word(64)]),
            boolean(),
        )
        .then(Equality::boolean(
            word::wc(64, "UnsignedLess", vec![x.clone(), zero_word(64)]),
            bit(false),
            dc("WordLessZero", vec![x]),
        ))?;
        self.theorem("ZeroLengthHasNoIndex", &args, e)?;
        let mut args = nil_valid;
        args.push(("i", word_type(64)));
        self.theorem(
            "ValidNilHasNoIndex",
            &args,
            Equality::boolean(
                dc("IndexUnsigned", vec![s.clone(), var("i")]),
                bit(false),
                dc(
                    "ZeroLengthHasNoIndex",
                    vec![
                        s.clone(),
                        dc("ValidNilLength", vec![s.clone(), var("h"), var("is_nil")]),
                        var("i"),
                    ],
                ),
            ),
        )
    }
}
