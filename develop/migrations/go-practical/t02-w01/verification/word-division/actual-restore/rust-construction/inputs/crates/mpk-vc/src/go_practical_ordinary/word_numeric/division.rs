//! Ordinary restoring division over the actual circuits and their Nat values.
use super::*;

mod natural;
mod restoring;
mod word_step;

const DIVISION: &str = "Mpk.Go.Practical.Word.Division";
fn dc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{DIVISION}.{name}"), args)
}
fn order(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, dc(name, args))
}
fn map_value(d: usize, e: Equality) -> Equality {
    let (left, right) = (value(d, e.left.clone()), value(d, e.right.clone()));
    e.mapped(
        lam("_division_word", cube(d), value(d, var("_division_word"))),
        left,
        right,
        nat(),
    )
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (left, right) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_division_n", nat(), less(var("_division_n"), b)),
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
        lam("_division_n", nat(), less(a, var("_division_n"))),
        left,
        right,
        boolean(),
    )
}
fn bool_map(name: &str, e: Equality) -> Equality {
    let (left, right) = (
        bc(name, vec![e.left.clone()]),
        bc(name, vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_division_flag",
            boolean(),
            bc(name, vec![var("_division_flag")]),
        ),
        left,
        right,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (left, right) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam(
            "_division_flag",
            boolean(),
            bc(name, vec![a, var("_division_flag")]),
        ),
        left,
        right,
        boolean(),
    )
}
struct Division<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Division<'_> {
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
            &format!("{DIVISION}.{name}"),
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
        polynomial::prove_named(self.builder, args, &mut self.serial, DIVISION, left, right)
    }
}

pub(super) fn append(builder: &mut Builder, _recipes: &BTreeMap<String, Expr>) -> Result<()> {
    let mut roots = builder
        .globals
        .keys()
        .filter(|n| {
            n.starts_with("Mpk.Go.Practical.Nat.") || n.starts_with("Mpk.Go.Practical.Bool.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for width in WIDTHS {
        for name in [
            "ValueBound",
            "SubtractionConservation",
            "UnsignedLessNatural",
            "DefinedDivisorPositive",
            "ZeroValue",
        ] {
            roots.push(format!("Mpk.Go.Practical.Word.Order.W{width}.{name}"));
        }
        roots.push(format!("{PREFIX}.W{width}.PushLowConservation"));
        for name in ["UnsignedDiv", "UnsignedRem"] {
            roots.push(format!("Mpk.Go.Practical.Word.W{width}.{name}"));
        }
    }
    // Exact closed dependencies are assembled before adding the new laws.
    // This is untrusted certificate construction, not application closure or
    // an alternative to checking every retained proof in both cores.
    builder.retain_ordinary_dependencies(&roots)?;
    let mut division = Division { builder, serial: 0 };
    division.natural_bounds()?;
    division.restoring_laws()?;
    for width in WIDTHS {
        division.word_step(width)?;
    }
    Ok(())
}
