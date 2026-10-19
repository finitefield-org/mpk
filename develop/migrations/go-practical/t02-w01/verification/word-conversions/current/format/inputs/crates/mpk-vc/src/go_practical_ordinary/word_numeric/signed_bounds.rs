//! Actual signed magnitudes have a computed gap to half the word modulus.
use super::*;
mod family;
mod half;
mod minimum;
mod scalar;
mod split;
mod threshold;

const BOUNDS: &str = "Mpk.Go.Practical.Word.SignedBounds";
fn sb(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{BOUNDS}.{name}"), args)
}
fn mg(name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("Mpk.Go.Practical.Word.SignedMagnitude.{name}"),
        args,
    )
}
fn order(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn half(d: usize) -> Expr {
    sb(&format!("Half{d}"), vec![])
}
fn low(d: usize, x: Expr) -> Expr {
    sb(&format!("Low{d}"), vec![x])
}
fn low_complement(d: usize, x: Expr) -> Expr {
    sb(&format!("LowComplement{d}"), vec![x])
}
fn top(d: usize, x: Expr) -> Expr {
    sb(&format!("Top{d}"), vec![x])
}
fn not(d: usize, x: Expr) -> Expr {
    order(&format!("NotCube{d}"), vec![x])
}
fn nlaw(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, sb(name, args))
}
fn half_definition(d: usize) -> Equality {
    let rhs = if d == 0 {
        one()
    } else {
        mul(weight(d - 1), half(d - 1))
    };
    instance(&format!("HalfDefinition{d}"), vec![], half(d), rhs)
}
fn low_body(d: usize, x: Expr, complement: bool) -> Expr {
    if d == 0 {
        return zero();
    }
    let h = d - 1;
    let lo = project(x.clone(), false);
    let lo = if complement { not(h, lo) } else { lo };
    let hi = if complement {
        low_complement(h, project(x, true))
    } else {
        low(h, project(x, true))
    };
    add(value(h, lo), mul(weight(h), hi))
}
fn low_definition(d: usize, x: Expr, complement: bool) -> Equality {
    let label = if complement { "LowComplement" } else { "Low" };
    let lhs = if complement {
        low_complement(d, x.clone())
    } else {
        low(d, x.clone())
    };
    instance(
        &format!("{label}Definition{d}"),
        vec![x.clone()],
        lhs,
        low_body(d, x, complement),
    )
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_bounds_n", nat(), less(var("_bounds_n"), b)),
        l,
        r,
        boolean(),
    )
}
fn less_right(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(
        lam("_bounds_n", nat(), less(a, var("_bounds_n"))),
        l,
        r,
        boolean(),
    )
}
fn bool_not(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_bounds_flag",
            boolean(),
            bc("Not", vec![var("_bounds_flag")]),
        ),
        l,
        r,
        boolean(),
    )
}
fn le(a: Expr, b: Expr) -> Expr {
    nc("LessEqual", vec![a, b])
}

struct Bounds<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Bounds<'_> {
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
            &format!("{BOUNDS}.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], l: Expr, r: Expr) -> Result<Equality> {
        polynomial::prove_named(self.builder, args, &mut self.serial, BOUNDS, l, r)
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut b = Bounds { builder, serial: 0 };
    b.gap_less_equal()?;
    b.threshold_laws()?;
    b.scalar_balance()?;
    for d in 0..=6 {
        b.definitions(d)?;
        b.half_double(d)?;
        b.value_split(d)?;
        b.low_balance(d)?;
        b.low_bound(d)?;
        b.half_positive(d)?;
    }
    for w in WIDTHS {
        b.family(w)?;
    }
    Ok(())
}
