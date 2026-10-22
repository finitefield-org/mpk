//! Frozen error-value shape: a full Word64 tag and the whole payload product.
//! P is the specialized product of every case, including inactive zero fields.
//! The source universe, tag assignment and payload validity are separate proofs.
use super::syntax::*;
use super::word::{at, fold_bits, wc, word_type, ADDRESSES};
use super::{Builder, Result};

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
        self.emit("Carrier", &[("P", Expr::Sort0)], Expr::Sort0,
                  pc("Type", vec![word.clone(), p.clone()]), false)?;
        let args = [("P", Expr::Sort0), ("tag", word.clone()), ("payload", p.clone())];
        self.emit("Make", &args, carrier(p.clone()),
                  pc("Make", vec![word.clone(), p.clone(), var("tag"), var("payload")]), false)?;
        self.emit("Nil", &[("P", Expr::Sort0), ("zero", p.clone())], carrier(p.clone()),
                  made(tag_zero(), var("zero")), false)?;
        let input = [("P", Expr::Sort0), ("e", carrier(p.clone()))];
        for (name, projection, ty) in [("Tag", "Left", word.clone()), ("Payload", "Right", p.clone())] {
            self.emit(name, &input, ty,
                      pc(projection, vec![word.clone(), p.clone(), var("e")]), false)?;
        }
        self.emit("Copy", &input, carrier(p.clone()),
                  made(value("Tag", var("e")), value("Payload", var("e"))), false)?;
        self.emit("IsNil", &input, boolean(), equal(value("Tag", var("e")), tag_zero()), false)?;
        self.emit("IsNonNil", &input, boolean(), bc("Not", vec![value("IsNil", var("e"))]), false)?;
        self.emit("HasCase", &[("P", Expr::Sort0), ("case", word), ("e", carrier(p))],
                  boolean(), equal(var("case"), value("Tag", var("e"))), false)
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
                let terms: Vec<_> = part.iter().map(|b| bc("Equal", vec![b.clone(), b.clone()])).collect();
                fold_bits("And", &terms, true)
            };
            call("Mpk.Go.Practical.Product.Relation.AndTrueIntro",
                 vec![comparisons(&bits[..m]), comparisons(&bits[m..]), proof(&bits[..m]), proof(&bits[m..])])
        }
        let bits = (0..64).map(|i| at(var("tag"), 64, i)).collect::<Result<Vec<_>>>()?;
        self.emit("TagEqualReflexive", &[("tag", word_type(64)?)],
                  truth(equal(var("tag"), var("tag")), true), proof(&bits), true)
    }

    fn laws(&mut self) -> Result<()> {
        let p = var("P");
        let word = word_type(64)?;
        let tag = var("tag");
        let payload = var("payload");
        let args = [("P", Expr::Sort0), ("tag", word.clone()), ("payload", p.clone())];
        let make = made(tag.clone(), payload.clone());
        for (name, projection, ty, expected) in [
            ("TagOfMake", "Tag", word.clone(), tag.clone()),
            ("PayloadOfMake", "Payload", p.clone(), payload.clone()),
        ] {
            self.emit(name, &args, eq(ty.clone(), value(projection, make.clone()), expected.clone()),
                      refl(ty, expected), true)?;
        }
        self.emit("IsNilOfMake", &args,
                  eq(boolean(), value("IsNil", make.clone()), equal(tag.clone(), tag_zero())),
                  refl(boolean(), equal(tag.clone(), tag_zero())), true)?;
        let mut case_args = args.to_vec();
        case_args.push(("case", word.clone()));
        self.emit("HasCaseOfMake", &case_args,
                  eq(boolean(), has_case(var("case"), make.clone()), equal(var("case"), tag.clone())),
                  refl(boolean(), equal(var("case"), tag.clone())), true)?;
        self.emit("MakeHasOwnCase", &args, truth(has_case(tag.clone(), make.clone()), true),
                  ec("TagEqualReflexive", vec![tag.clone()]), true)?;
        let mut nonzero = args.to_vec();
        nonzero.push(("nonzero", truth(equal(tag.clone(), tag_zero()), false)));
        self.emit("NonzeroMakeIsNotNil", &nonzero, truth(value("IsNil", make.clone()), false), var("nonzero"), true)?;
        let h = Equality::boolean(equal(tag.clone(), tag_zero()), bit(false), var("nonzero"));
        self.emit("NonzeroMakeIsNonNil", &nonzero, truth(value("IsNonNil", make), true),
                  h.mapped(lam("_bit", boolean(), bc("Not", vec![var("_bit")])),
                           bc("Not", vec![equal(tag.clone(), tag_zero())]), bc("Not", vec![bit(false)]), boolean()).proof, true)?;

        let nil_args = [("P", Expr::Sort0), ("zero", p.clone())];
        let nil = ec("Nil", vec![p.clone(), var("zero")]);
        for (name, projection, ty, expected) in [
            ("NilTag", "Tag", word.clone(), tag_zero()),
            ("NilPayload", "Payload", p.clone(), var("zero")),
            ("NilIsNil", "IsNil", boolean(), bit(true)),
            ("NilIsNotNonNil", "IsNonNil", boolean(), bit(false)),
        ] {
            self.emit(name, &nil_args, eq(ty.clone(), value(projection, nil.clone()), expected.clone()),
                      refl(ty, expected), true)?;
        }
        let mut no_case = nil_args.to_vec();
        no_case.extend([("case", word.clone()), ("nonzero", truth(equal(var("case"), tag_zero()), false))]);
        self.emit("NilHasNoNonzeroCase", &no_case, truth(has_case(var("case"), nil), false), var("nonzero"), true)?;
        // Every payload, including the concrete zero, remains non-nil at tag 1.
        self.emit("ConcreteZeroIsNonNil", &nil_args,
                  truth(value("IsNonNil", made(wc(64, "One", vec![]), var("zero"))), true),
                  refl(boolean(), bit(true)), true)?;

        let input = [("P", Expr::Sort0), ("e", carrier(p.clone()))];
        let copy = value("Copy", var("e"));
        for (name, projection, ty) in [("CopyTag", "Tag", word.clone()), ("CopyPayload", "Payload", p.clone()),
                                     ("CopyIsNil", "IsNil", boolean()), ("CopyIsNonNil", "IsNonNil", boolean())] {
            let expected = value(projection, var("e"));
            self.emit(name, &input, eq(ty.clone(), value(projection, copy.clone()), expected.clone()),
                      refl(ty, expected), true)?;
        }
        let mut case_input = input.to_vec();
        case_input.push(("case", word.clone()));
        let expected = has_case(var("case"), var("e"));
        self.emit("CopyHasCase", &case_input,
                  eq(boolean(), has_case(var("case"), copy.clone()), expected.clone()), refl(boolean(), expected), true)?;
        self.emit("CopyPointwise", &input,
                  pc("Pointwise", vec![word.clone(), p.clone(), copy.clone(), var("e")]),
                  pc("PointwiseOfProjections", vec![word.clone(), p.clone(), copy.clone(), var("e"),
                      refl(word.clone(), value("Tag", var("e"))), refl(p.clone(), value("Payload", var("e")))]), true)?;
        // The address remains universally quantified; no truncation to a low bit.
        let mut addressed = input.to_vec();
        addressed.extend(ADDRESSES.iter().map(|&name| (name, boolean())));
        let indexes = ADDRESSES.iter().map(|name| var(name)).collect::<Vec<_>>();
        let expected = app(value("Tag", var("e")), indexes.clone());
        self.emit("CopyTagAt", &addressed,
                  eq(boolean(), app(value("Tag", copy), indexes), expected.clone()), refl(boolean(), expected), true)?;
        Ok(())
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut errors = Errors(builder);
    errors.definitions()?;
    errors.equal_reflexive()?;
    errors.laws()
}
