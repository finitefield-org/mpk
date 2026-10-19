//! Signed multiplication is exact modulo a computed balanced-Int correction.
use super::*;
mod family;
mod natural;
mod projection;
mod scalar;

const SIGNED_PRODUCT: &str = "Mpk.Go.Practical.Word.SignedProduct";
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{SIGNED_PRODUCT}.{name}"), args)
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn integer() -> Expr {
    pi("_part", boolean(), nat())
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, sc(name, args))
}
fn less_right(a: Expr, e: Equality) -> Equality {
    let (left, right) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(
        lam("_positive_value", nat(), less(a, var("_positive_value"))),
        left,
        right,
        boolean(),
    )
}
struct SignedProduct<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl SignedProduct<'_> {
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
            &format!("{SIGNED_PRODUCT}.{name}"),
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
        polynomial::prove_named(
            self.builder,
            args,
            &mut self.serial,
            SIGNED_PRODUCT,
            left,
            right,
        )
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
    for width in WIDTHS {
        for name in ["MulValueConservation", "SignedPositive", "SignedNegative"] {
            roots.push(format!("{PREFIX}.W{width}.{name}"));
        }
        roots.push(format!("{PREFIX}.PowerTwoWeight{}", depth(width)?));
    }
    for name in [
        "Make",
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
        "PositiveOfAdd",
        "NegativeOfAdd",
        "PositiveOfMul",
        "NegativeOfMul",
    ] {
        roots.push(format!("Mpk.Go.Practical.Int.{name}"));
    }
    builder.retain_ordinary_dependencies(&roots)?;
    let mut product = SignedProduct { builder, serial: 0 };
    product.natural_laws()?;
    product.scalar_laws()?;
    for width in WIDTHS {
        product.family(width)?;
    }
    Ok(())
}
