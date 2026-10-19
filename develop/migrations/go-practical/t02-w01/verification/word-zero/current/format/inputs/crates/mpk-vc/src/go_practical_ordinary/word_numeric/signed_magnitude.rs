//! Actual two's-complement negation and unsigned magnitude, with ordinary proofs.
use super::*;
mod magnitude;
mod natural;
mod negation;
mod projection;
mod signed_negation;

const MAGNITUDE: &str = "Mpk.Go.Practical.Word.SignedMagnitude";
fn mc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{MAGNITUDE}.{name}"), args)
}
fn order(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn shared(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.SignedProduct.{name}"), args)
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn integer() -> Expr {
    pi("_part", boolean(), nat())
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, mc(name, args))
}
fn unary(w: usize, name: &str, x: Expr) -> Expr {
    mc(&format!("W{w}.{name}"), vec![x])
}
fn sign(w: usize, x: Expr) -> Result<Expr> {
    at(x, w, w - 1)
}
fn neg(w: usize, x: Expr) -> Expr {
    wc(w, "Neg", vec![x])
}
fn signed(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}
fn map_value(d: usize, e: Equality) -> Equality {
    let (left, right) = (value(d, e.left.clone()), value(d, e.right.clone()));
    e.mapped(
        lam("_magnitude_word", cube(d), value(d, var("_magnitude_word"))),
        left,
        right,
        nat(),
    )
}
fn bool_map(name: &str, e: Equality) -> Equality {
    let (left, right) = (
        bc(name, vec![e.left.clone()]),
        bc(name, vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_magnitude_flag",
            boolean(),
            bc(name, vec![var("_magnitude_flag")]),
        ),
        left,
        right,
        boolean(),
    )
}
fn less_right(a: Expr, e: Equality) -> Equality {
    let (left, right) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(
        lam("_magnitude_n", nat(), less(a, var("_magnitude_n"))),
        left,
        right,
        boolean(),
    )
}
struct Magnitude<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Magnitude<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.builder,
            &format!("{MAGNITUDE}.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], left: Expr, right: Expr) -> Result<Equality> {
        polynomial::prove_named(self.builder, args, &mut self.serial, MAGNITUDE, left, right)
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut roots = builder
        .globals
        .keys()
        .filter(|n| {
            n.starts_with("Mpk.Go.Practical.Nat.") || n.starts_with("Mpk.Go.Practical.Bool.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for w in WIDTHS {
        for name in ["Neg", "AbsBits", "DivideDefined"] {
            roots.push(format!("Mpk.Go.Practical.Word.W{w}.{name}"));
        }
        for name in [
            "Complement",
            "ValueBound",
            "IsZeroNatural",
            "ZeroValue",
            "ZeroPointwise",
        ] {
            roots.push(format!("Mpk.Go.Practical.Word.Order.W{w}.{name}"));
        }
        roots.push(format!(
            "Mpk.Go.Practical.Word.Order.W{w}.EqualBitsBlock{}At0",
            depth(w)?
        ));
        for name in ["SignedPositive", "SignedNegative"] {
            roots.push(format!("{PREFIX}.W{w}.{name}"));
        }
        roots.push(format!("{PREFIX}.W{w}.Block{}At0", depth(w)?));
        roots.push(format!(
            "Mpk.Go.Practical.Word.SignedProduct.W{w}.ModulusPositive"
        ));
    }
    for name in [
        "ZeroSumLeft",
        "ZeroSumRight",
        "MultiplyCancelPositive",
        "ReflectionAgreement",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.SignedProduct.{name}"));
    }
    for name in [
        "Make",
        "Negate",
        "FromNat",
        "Mul",
        "Add",
        "Equivalent",
        "Equal",
        "EqualFromEquivalent",
        "EqualReflect",
        "PositiveOfMake",
        "NegativeOfMake",
        "PositiveOfFromNat",
        "NegativeOfFromNat",
        "PositiveOfNegate",
        "NegativeOfNegate",
        "PositiveOfAdd",
        "NegativeOfAdd",
        "PositiveOfMul",
        "NegativeOfMul",
    ] {
        roots.push(format!("Mpk.Go.Practical.Int.{name}"));
    }
    builder.retain_ordinary_dependencies(&roots)?;
    let mut m = Magnitude { builder, serial: 0 };
    m.natural_laws()?;
    for w in WIDTHS {
        m.negation(w)?;
        m.magnitude(w)?;
        m.signed_negation(w)?;
    }
    Ok(())
}
