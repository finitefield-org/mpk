//! Tagged observation view: a full Word64 tag and the whole payload product.
//! Adapting the exact frozen balanced carrier to this view is separate work,
//! as are the source universe, tag assignment and inactive-field validity.
use super::syntax::*;
use super::word::{at, fold_bits, wc, word_type, ADDRESSES};
use super::{Builder, Result};
use std::collections::BTreeMap;

const PREFIX: &str = "Mpk.Go.Practical.ErrorValue";
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn equal(a: Expr, b: Expr) -> Expr {
    wc(64, "Equal", vec![a, b])
}
fn tag_zero() -> Expr {
    wc(64, "Zero", vec![])
}
fn carrier(p: Expr) -> Expr {
    ec("Carrier", vec![p])
}
fn value(name: &str, e: Expr) -> Expr {
    ec(name, vec![var("P"), e])
}
fn made(tag: Expr, payload: Expr) -> Expr {
    ec("Make", vec![var("P"), tag, payload])
}
fn has_case(tag: Expr, e: Expr) -> Expr {
    ec("HasCase", vec![var("P"), tag, e])
}
fn truth(b: Expr, yes: bool) -> Expr {
    eq(boolean(), b, bit(yes))
}

struct Errors<'a>(&'a mut Builder);
impl Errors<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, body, law)
    }

    fn definitions(&mut self) -> Result<()> {
        let p = var("P");
        let word = word_type(64)?;
        self.emit(
            "Carrier",
            &[("P", Expr::Sort0)],
            Expr::Sort0,
            pc("Type", vec![word.clone(), p.clone()]),
            false,
        )?;
        let args = [
            ("P", Expr::Sort0),
            ("tag", word.clone()),
            ("payload", p.clone()),
        ];
        self.emit(
            "Make",
            &args,
            carrier(p.clone()),
            pc(
                "Make",
                vec![word.clone(), p.clone(), var("tag"), var("payload")],
            ),
            false,
        )?;
        self.emit(
            "Nil",
            &[("P", Expr::Sort0), ("zero", p.clone())],
            carrier(p.clone()),
            made(tag_zero(), var("zero")),
            false,
        )?;
        let input = [("P", Expr::Sort0), ("e", carrier(p.clone()))];
        for (name, projection, ty) in [
            ("Tag", "Left", word.clone()),
            ("Payload", "Right", p.clone()),
        ] {
            self.emit(
                name,
                &input,
                ty,
                pc(projection, vec![word.clone(), p.clone(), var("e")]),
                false,
            )?;
        }
        self.emit(
            "Copy",
            &input,
            carrier(p.clone()),
            made(value("Tag", var("e")), value("Payload", var("e"))),
            false,
        )?;
        self.emit(
            "IsNil",
            &input,
            boolean(),
            equal(value("Tag", var("e")), tag_zero()),
            false,
        )?;
        self.emit(
            "IsNonNil",
            &input,
            boolean(),
            bc("Not", vec![value("IsNil", var("e"))]),
            false,
        )?;
        self.emit(
            "HasCase",
            &[("P", Expr::Sort0), ("case", word), ("e", carrier(p))],
            boolean(),
            equal(var("case"), value("Tag", var("e"))),
            false,
        )
    }

    fn equal_reflexive(&mut self) -> Result<()> {
        // A balanced conjunction of all 64 actual bit comparisons. There is no
        // machine integer primitive, sampled tag set or function equality rule.
        fn proof(bits: &[Expr]) -> Expr {
            if let [b] = bits {
                return bc("EqualReflexive", vec![b.clone()]);
            }
            let m = bits.len() / 2;
            let comparisons = |part: &[Expr]| {
                let terms: Vec<_> = part
                    .iter()
                    .map(|b| bc("Equal", vec![b.clone(), b.clone()]))
                    .collect();
                fold_bits("And", &terms, true)
            };
            call(
                "Mpk.Go.Practical.Product.Relation.AndTrueIntro",
                vec![
                    comparisons(&bits[..m]),
                    comparisons(&bits[m..]),
                    proof(&bits[..m]),
                    proof(&bits[m..]),
                ],
            )
        }
        let bits = (0..64)
            .map(|i| at(var("tag"), 64, i))
            .collect::<Result<Vec<_>>>()?;
        self.emit(
            "TagEqualReflexive",
            &[("tag", word_type(64)?)],
            truth(equal(var("tag"), var("tag")), true),
            proof(&bits),
            true,
        )
    }

    fn different_bits(&mut self, recipes: &BTreeMap<String, Expr>) -> Result<()> {
        let a = var("a");
        let b = var("b");
        for right in [false, true] {
            let input = [
                ("a", boolean()), ("b", boolean()),
                ("different", truth(if right { b.clone() } else { a.clone() }, false)),
            ];
            let start = bc("And", vec![a.clone(), b.clone()]);
            let reduced = if right { bc("And", vec![a.clone(), bit(false)]) }
                          else { bc("And", vec![bit(false), b.clone()]) };
            let mapped = Equality::boolean(if right { b.clone() } else { a.clone() }, bit(false), var("different"))
                .mapped(if right { bc("And", vec![a.clone()]) }
                    else { lam("_bit", boolean(), bc("And", vec![var("_bit"), b.clone()])) },
                    start.clone(), reduced.clone(), boolean());
            let computed = if right { refl(boolean(), bit(false)) } else {
                call("Std.Bool.cases", vec![lam("_b", boolean(),
                    truth(bc("And", vec![bit(false), var("_b")]), false)),
                    refl(boolean(), bit(false)), refl(boolean(), bit(false)), b.clone()])
            };
            self.emit(if right { "AndFalseRight" } else { "AndFalseLeft" }, &input,
                truth(start, false), mapped.then(Equality::boolean(reduced, bit(false), computed))?.proof, true)?;
        }
        fn mismatch(comparisons: &[Expr], index: usize) -> Expr {
            if comparisons.len() == 1 { return var("different"); }
            let m = comparisons.len() / 2;
            let right = index >= m;
            ec(if right { "AndFalseRight" } else { "AndFalseLeft" }, vec![
                fold_bits("And", &comparisons[..m], true), fold_bits("And", &comparisons[m..], true),
                if right { mismatch(&comparisons[m..], index - m) } else { mismatch(&comparisons[..m], index) },
            ])
        }
        let comparisons = (0..64).map(|i| Ok(bc("Equal", vec![at(var("a"), 64, i)?, at(var("b"), 64, i)?])))
            .collect::<Result<Vec<_>>>()?;
        for i in 0..64 {
            self.emit(&format!("TagDifferentAt{i}"), &[("a", word_type(64)?), ("b", word_type(64)?),
                ("different", truth(comparisons[i].clone(), false))],
                truth(equal(var("a"), var("b")), false), mismatch(&comparisons, i), true)?;
        }
        let different = super::word_proofs::Evaluation::new(recipes).equation(
            bc("Equal", vec![at(wc(64, "One", vec![]), 64, 0)?, at(tag_zero(), 64, 0)?]), bit(false))?;
        self.emit("TagOneIsNonzero", &[], truth(equal(wc(64, "One", vec![]), tag_zero()), false),
            ec("TagDifferentAt0", vec![wc(64, "One", vec![]), tag_zero(), different.proof]), true)
    }

    fn laws(&mut self) -> Result<()> {
        let p = var("P");
        let word = word_type(64)?;
        let tag = var("tag");
        let payload = var("payload");
        let args = [
            ("P", Expr::Sort0),
            ("tag", word.clone()),
            ("payload", p.clone()),
        ];
        let make = made(tag.clone(), payload.clone());
        let input = [("P", Expr::Sort0), ("e", carrier(p.clone()))];
        self.emit("NonNilDefinition", &input,
            eq(boolean(), value("IsNonNil", var("e")), bc("Not", vec![value("IsNil", var("e"))])),
            refl(boolean(), bc("Not", vec![value("IsNil", var("e"))])), true)?;
        let nil_args = [("P", Expr::Sort0), ("zero", p.clone())];
        let nil = ec("Nil", vec![p.clone(), var("zero")]);
        let nil_make = made(tag_zero(), var("zero"));
        self.emit("NilDefinition", &nil_args,
            eq(carrier(p.clone()), nil.clone(), nil_make.clone()), refl(carrier(p.clone()), nil_make.clone()), true)?;
        let nil_definition = Equality {
            ty: carrier(p.clone()), left: nil.clone(), right: nil_make.clone(),
            proof: ec("NilDefinition", vec![p.clone(), var("zero")]),
        };
        for (name, projection, ty, expected) in [
            ("TagOfMake", "Tag", word.clone(), tag.clone()),
            ("PayloadOfMake", "Payload", p.clone(), payload.clone()),
        ] {
            self.emit(
                name,
                &args,
                eq(
                    ty.clone(),
                    value(projection, make.clone()),
                    expected.clone(),
                ),
                refl(ty, expected),
                true,
            )?;
        }
        let tag_of_make = Equality {
            ty: word.clone(),
            left: value("Tag", make.clone()),
            right: tag.clone(),
            proof: ec("TagOfMake", vec![p.clone(), tag.clone(), payload.clone()]),
        };
        self.emit(
            "IsNilOfMake",
            &args,
            eq(
                boolean(),
                value("IsNil", make.clone()),
                equal(tag.clone(), tag_zero()),
            ),
            tag_of_make.clone().mapped(
                lam("_tag", word.clone(), equal(var("_tag"), tag_zero())),
                value("IsNil", make.clone()), equal(tag.clone(), tag_zero()), boolean()).proof,
            true,
        )?;
        let mut case_args = args.to_vec();
        case_args.push(("case", word.clone()));
        self.emit(
            "HasCaseOfMake",
            &case_args,
            eq(
                boolean(),
                has_case(var("case"), make.clone()),
                equal(var("case"), tag.clone()),
            ),
            tag_of_make.mapped(
                lam("_tag", word.clone(), equal(var("case"), var("_tag"))),
                has_case(var("case"), make.clone()), equal(var("case"), tag.clone()), boolean()).proof,
            true,
        )?;
        self.emit(
            "MakeHasOwnCase",
            &args,
            truth(has_case(tag.clone(), make.clone()), true),
            Equality::boolean(has_case(tag.clone(), make.clone()), equal(tag.clone(), tag.clone()),
                ec("HasCaseOfMake", vec![p.clone(), tag.clone(), payload.clone(), tag.clone()]))
                .then(Equality::boolean(equal(tag.clone(), tag.clone()), bit(true),
                    ec("TagEqualReflexive", vec![tag.clone()])))?.proof,
            true,
        )?;
        let mut nonzero = args.to_vec();
        nonzero.push(("nonzero", truth(equal(tag.clone(), tag_zero()), false)));
        self.emit(
            "NonzeroMakeIsNotNil",
            &nonzero,
            truth(value("IsNil", make.clone()), false),
            Equality::boolean(value("IsNil", make.clone()), equal(tag.clone(), tag_zero()),
                ec("IsNilOfMake", vec![p.clone(), tag.clone(), payload.clone()]))
                .then(Equality::boolean(equal(tag.clone(), tag_zero()), bit(false), var("nonzero")))?.proof,
            true,
        )?;
        let h = Equality::boolean(value("IsNil", make.clone()), bit(false),
            ec("NonzeroMakeIsNotNil", vec![p.clone(), tag.clone(), payload.clone(), var("nonzero")]));
        self.emit(
            "NonzeroMakeIsNonNil",
            &nonzero,
            truth(value("IsNonNil", make.clone()), true),
            h.mapped(
                lam("_bit", boolean(), bc("Not", vec![var("_bit")])),
                bc("Not", vec![value("IsNil", make)]),
                bc("Not", vec![bit(false)]),
                boolean(),
            )
            .proof,
            true,
        )?;

        for (name, projection, ty, expected) in [
            ("NilTag", "Tag", word.clone(), tag_zero()),
            ("NilPayload", "Payload", p.clone(), var("zero")),
        ] {
            self.emit(
                name,
                &nil_args,
                eq(ty.clone(), value(projection, nil.clone()), expected.clone()),
                refl(ty, expected),
                true,
            )?;
        }
        self.emit("NilIsNil", &nil_args, truth(value("IsNil", nil.clone()), true),
            nil_definition.clone().mapped(ec("IsNil", vec![p.clone()]),
                value("IsNil", nil.clone()), value("IsNil", nil_make.clone()), boolean())
                .then(Equality::boolean(value("IsNil", nil_make.clone()), equal(tag_zero(), tag_zero()),
                    ec("IsNilOfMake", vec![p.clone(), tag_zero(), var("zero")])))?
                .then(Equality::boolean(equal(tag_zero(), tag_zero()), bit(true),
                    ec("TagEqualReflexive", vec![tag_zero()])))?.proof, true)?;
        self.emit("NilIsNotNonNil", &nil_args, truth(value("IsNonNil", nil.clone()), false),
            Equality::boolean(value("IsNonNil", nil.clone()), bc("Not", vec![value("IsNil", nil.clone())]),
                ec("NonNilDefinition", vec![p.clone(), nil.clone()]))
                .then(Equality::boolean(value("IsNil", nil.clone()), bit(true),
                ec("NilIsNil", vec![p.clone(), var("zero")]))
                .mapped(constant("Mpk.Go.Practical.Bool.Not"),
                    bc("Not", vec![value("IsNil", nil.clone())]), bc("Not", vec![bit(true)]), boolean()))?.proof, true)?;
        let mut no_case = nil_args.to_vec();
        no_case.extend([
            ("case", word.clone()),
            ("nonzero", truth(equal(var("case"), tag_zero()), false)),
        ]);
        self.emit(
            "NilHasNoNonzeroCase",
            &no_case,
            truth(has_case(var("case"), nil.clone()), false),
            nil_definition.mapped(ec("HasCase", vec![p.clone(), var("case")]),
                has_case(var("case"), nil), has_case(var("case"), nil_make.clone()), boolean())
                .then(Equality::boolean(has_case(var("case"), nil_make), equal(var("case"), tag_zero()),
                    ec("HasCaseOfMake", vec![p.clone(), tag_zero(), var("zero"), var("case")])))?
                .then(Equality::boolean(equal(var("case"), tag_zero()), bit(false), var("nonzero")))?.proof,
            true,
        )?;
        // Every payload, including the concrete zero, remains non-nil at tag 1.
        self.emit(
            "ConcreteZeroIsNonNil",
            &nil_args,
            truth(
                value("IsNonNil", made(wc(64, "One", vec![]), var("zero"))),
                true,
            ),
            ec("NonzeroMakeIsNonNil", vec![p.clone(), wc(64, "One", vec![]), var("zero"), ec("TagOneIsNonzero", vec![])]),
            true,
        )?;

        let copy = value("Copy", var("e"));
        for (name, projection, ty) in [
            ("CopyTag", "Tag", word.clone()),
            ("CopyPayload", "Payload", p.clone()),
        ] {
            let expected = value(projection, var("e"));
            self.emit(
                name,
                &input,
                eq(
                    ty.clone(),
                    value(projection, copy.clone()),
                    expected.clone(),
                ),
                refl(ty, expected),
                true,
            )?;
        }
        let copy_tag = Equality {
            ty: word.clone(), left: value("Tag", copy.clone()), right: value("Tag", var("e")),
            proof: ec("CopyTag", vec![p.clone(), var("e")]),
        };
        self.emit("CopyIsNil", &input,
            eq(boolean(), value("IsNil", copy.clone()), value("IsNil", var("e"))),
            copy_tag.clone().mapped(lam("_tag", word.clone(), equal(var("_tag"), tag_zero())),
                value("IsNil", copy.clone()), value("IsNil", var("e")), boolean()).proof, true)?;
        self.emit("CopyIsNonNil", &input,
            eq(boolean(), value("IsNonNil", copy.clone()), value("IsNonNil", var("e"))),
            Equality::boolean(value("IsNil", copy.clone()), value("IsNil", var("e")),
                ec("CopyIsNil", vec![p.clone(), var("e")]))
                .mapped(lam("_bit", boolean(), bc("Not", vec![var("_bit")])),
                    value("IsNonNil", copy.clone()), value("IsNonNil", var("e")), boolean()).proof, true)?;
        let mut case_input = input.to_vec();
        case_input.push(("case", word.clone()));
        let expected = has_case(var("case"), var("e"));
        self.emit(
            "CopyHasCase",
            &case_input,
            eq(
                boolean(),
                has_case(var("case"), copy.clone()),
                expected.clone(),
            ),
            copy_tag.mapped(lam("_tag", word.clone(), equal(var("case"), var("_tag"))),
                has_case(var("case"), copy.clone()), expected, boolean()).proof,
            true,
        )?;
        self.emit(
            "CopyPointwise",
            &input,
            pc(
                "Pointwise",
                vec![word.clone(), p.clone(), copy.clone(), var("e")],
            ),
            pc(
                "PointwiseOfProjections",
                vec![
                    word.clone(),
                    p.clone(),
                    copy.clone(),
                    var("e"),
                    refl(word.clone(), value("Tag", var("e"))),
                    refl(p.clone(), value("Payload", var("e"))),
                ],
            ),
            true,
        )?;
        // The address remains universally quantified; no truncation to a low bit.
        let mut addressed = input.to_vec();
        addressed.extend(ADDRESSES.iter().map(|&name| (name, boolean())));
        let indexes = ADDRESSES.iter().map(|name| var(name)).collect::<Vec<_>>();
        let expected = app(value("Tag", var("e")), indexes.clone());
        self.emit(
            "CopyTagAt",
            &addressed,
            eq(
                boolean(),
                app(value("Tag", copy), indexes),
                expected.clone(),
            ),
            refl(boolean(), expected),
            true,
        )?;
        Ok(())
    }
}

pub(super) fn append(builder: &mut Builder, recipes: &BTreeMap<String, Expr>) -> Result<()> {
    let mut errors = Errors(builder);
    errors.definitions()?;
    errors.equal_reflexive()?;
    errors.different_bits(recipes)?;
    errors.laws()
}
