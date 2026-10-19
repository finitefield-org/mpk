//! Direct balanced zero reflection, avoiding general word-order proof closure.
use super::*;
mod cube;
mod family;
mod natural;
const ZERO: &str = "Mpk.Go.Practical.Word.Zero";
fn zc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{ZERO}.{name}"), args)
}
fn shared(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.SignedProduct.{name}"), args)
}
fn order(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn bits(d: usize, x: Expr) -> Expr {
    zc(&format!("ZeroBits{d}"), vec![x])
}
fn nlaw(name: &str, args: Vec<Expr>, l: Expr, r: Expr) -> Equality {
    Equality::natural(l, r, nc(name, args))
}
fn bool_map(name: &str, e: Equality) -> Equality {
    let (l, r) = (
        bc(name, vec![e.left.clone()]),
        bc(name, vec![e.right.clone()]),
    );
    e.mapped(
        lam("_zero_bit", boolean(), bc(name, vec![var("_zero_bit")])),
        l,
        r,
        boolean(),
    )
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_zero_bit", boolean(), bc(name, vec![var("_zero_bit"), b])),
        l,
        r,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_zero_bit", boolean(), bc(name, vec![a, var("_zero_bit")])),
        l,
        r,
        boolean(),
    )
}
fn equal_left(e: Equality) -> Equality {
    let (l, r) = (
        equal(e.left.clone(), zero()),
        equal(e.right.clone(), zero()),
    );
    e.mapped(
        lam("_zero_value", nat(), equal(var("_zero_value"), zero())),
        l,
        r,
        boolean(),
    )
}
fn less_right(e: Equality) -> Equality {
    let (l, r) = (less(zero(), e.left.clone()), less(zero(), e.right.clone()));
    e.mapped(
        lam("_zero_value", nat(), less(zero(), var("_zero_value"))),
        l,
        r,
        boolean(),
    )
}
fn top(d: usize, x: Expr) -> Expr {
    app(x, vec![bit(true); d])
}
fn literal(d: usize, x: Expr) -> Expr {
    fn join(parts: &[Expr]) -> Expr {
        if parts.len() == 1 {
            return parts[0].clone();
        }
        let m = parts.len() / 2;
        bc("And", vec![join(&parts[..m]), join(&parts[m..])])
    }
    let values = (0..(1 << d))
        .map(|i| {
            bc(
                "Not",
                vec![app(
                    x.clone(),
                    (0..d).rev().map(|j| bit(i & (1 << j) != 0)).collect(),
                )],
            )
        })
        .collect::<Vec<_>>();
    join(&values)
}
struct Zero<'a> {
    builder: &'a mut Builder,
}
impl Zero<'_> {
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
            &format!("{ZERO}.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
}
pub(super) fn append(builder: &mut Builder, extra_roots: &[String]) -> Result<()> {
    let mut roots = builder
        .globals
        .keys()
        .filter(|n| {
            n.starts_with("Mpk.Go.Practical.Nat.") || n.starts_with("Mpk.Go.Practical.Bool.")
        })
        .cloned()
        .collect::<Vec<_>>();
    roots.extend_from_slice(extra_roots);
    for d in 0..=6 {
        roots.push(format!("Mpk.Go.Practical.Word.Order.Complement{d}"));
        roots.push(format!("{PREFIX}.ValueDefinition{d}"));
    }
    for w in WIDTHS {
        for n in ["IsZero", "DivideDefined"] {
            roots.push(format!("Mpk.Go.Practical.Word.W{w}.{n}"));
        }
    }
    for n in [
        "ZeroSumLeft",
        "ZeroSumRight",
        "MultiplyCancelPositive",
        "ReflectionAgreement",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.SignedProduct.{n}"));
    }
    builder.retain_ordinary_dependencies(&roots)?;
    let mut z = Zero { builder };
    z.natural_laws()?;
    for d in 0..=6 {
        z.cube(d)?;
    }
    for w in WIDTHS {
        z.family(w)?;
    }
    Ok(())
}
