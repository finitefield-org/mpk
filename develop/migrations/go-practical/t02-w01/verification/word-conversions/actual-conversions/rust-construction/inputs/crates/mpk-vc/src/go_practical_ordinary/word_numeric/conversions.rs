//! Actual truncation, zero/sign extension and signed count normalization.
use super::*;
mod cubes;
mod family;
mod integer;

const CONVERSIONS: &str = "Mpk.Go.Practical.Word.Conversions";
fn cv(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{CONVERSIONS}.{name}"), args)
}
fn fill(d: usize, b: Expr) -> Expr {
    cv(&format!("Fill{d}"), vec![b])
}
fn ones(d: usize) -> Expr {
    cv(&format!("Ones{d}"), vec![])
}
fn mask(s: usize, t: usize) -> Expr {
    cv(&format!("Mask{s}To{t}"), vec![])
}
fn ratio(s: usize, t: usize) -> Expr {
    cv(&format!("Ratio{s}To{t}"), vec![])
}
fn low(s: usize, t: usize, x: Expr) -> Expr {
    cv(&format!("Low{s}To{t}"), vec![x])
}
fn quotient(s: usize, t: usize, x: Expr) -> Expr {
    cv(&format!("Quotient{s}To{t}"), vec![x])
}
fn extend(s: usize, t: usize, x: Expr, b: Expr) -> Expr {
    cv(&format!("Extend{s}To{t}"), vec![x, b])
}
fn converted(w: usize, t: usize, signed: bool, x: Expr) -> Expr {
    wc(
        w,
        &format!("{}To{t}", if signed { "Signed" } else { "Unsigned" }),
        vec![x],
    )
}
fn sign(w: usize, x: Expr) -> Result<Expr> {
    at(x, w, w - 1)
}
fn int(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn signed(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}
fn part(negative: bool, x: Expr) -> Expr {
    int(if negative { "Negative" } else { "Positive" }, vec![x])
}
fn law(name: &str, args: Vec<Expr>, a: Expr, b: Expr) -> Equality {
    Equality::natural(a, b, nc(name, args))
}
fn known(name: &str, args: Vec<Expr>, a: Expr, b: Expr) -> Equality {
    Equality::natural(a, b, cv(name, args))
}
struct Conversions<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Conversions<'_> {
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
            &format!("{CONVERSIONS}.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], a: Expr, b: Expr) -> Result<Equality> {
        polynomial::prove_named(self.builder, args, &mut self.serial, CONVERSIONS, a, b)
    }
    fn relation(
        &mut self,
        name: &str,
        args: &[Binder],
        a: Expr,
        b: Expr,
        middle: Equality,
    ) -> Result<()> {
        let (left, right) = integer::cross(a.clone(), b.clone())?;
        let proof = left.then(middle)?.then(right.reverse())?.proof;
        self.emit(name, args, int("Equivalent", vec![a, b]), proof, true)
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut roots = builder
        .globals
        .keys()
        .filter(|n| {
            n.starts_with("Mpk.Go.Practical.Nat.")
                || n.starts_with("Mpk.Go.Practical.Bool.")
                || n.starts_with("Mpk.Go.Practical.Int.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for w in WIDTHS {
        for t in WIDTHS {
            for signed in [false, true] {
                roots.push(format!(
                    "Mpk.Go.Practical.Word.W{w}.{}To{t}",
                    if signed { "Signed" } else { "Unsigned" }
                ));
            }
        }
        roots.push(format!("Mpk.Go.Practical.Word.W{w}.SignedCountDefined"));
        roots.push(format!("Mpk.Go.Practical.Word.Order.W{w}.ValueBound"));
        roots.push(format!("{PREFIX}.LiteralValueCongruence{w}"));
        for n in ["SignedPositive", "SignedNegative"] {
            roots.push(format!("{PREFIX}.W{w}.{n}"));
        }
    }
    builder.retain_ordinary_dependencies(&roots)?;
    let mut c = Conversions { builder, serial: 0 };
    for d in 0..=6 {
        c.fill(d)?;
    }
    for s in 3..=6 {
        for t in s..=6 {
            c.extension(s, t)?;
        }
        for t in 3..=s {
            c.truncation(s, t)?;
        }
    }
    for w in WIDTHS {
        for t in WIDTHS {
            for sg in [false, true] {
                c.family(w, t, sg)?;
            }
        }
        c.count(w)?;
    }
    Ok(())
}
