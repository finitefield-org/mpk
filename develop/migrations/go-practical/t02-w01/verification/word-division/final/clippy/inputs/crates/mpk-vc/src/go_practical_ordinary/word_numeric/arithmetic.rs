//! Fresh arithmetic proof recipes over the retained complete numeric component.
use super::*;

mod bits;
mod digits;
mod powers;
mod product;
mod push;
mod shift;

fn numeral(n: usize) -> Expr {
    (0..n).fold(zero(), |n, _| succ(n))
}
fn power(n: Expr) -> Expr {
    numeric("PowerTwo", vec![n])
}
fn coefficient(i: usize) -> Expr {
    power(numeral(i))
}
fn unary(width: usize, name: &str, x: Expr) -> Expr {
    numeric(&format!("W{width}.{name}"), vec![x])
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, numeric(name, args))
}
fn pointwise(width: usize, x: Expr, y: Expr) -> Result<Expr> {
    let args = ADDRESSES[..depth(width)?]
        .iter()
        .map(|n| var(n))
        .collect::<Vec<_>>();
    let mut ty = eq(boolean(), app(x, args.clone()), app(y, args));
    for name in ADDRESSES[..depth(width)?].iter().rev() {
        ty = pi(name, boolean(), ty);
    }
    Ok(ty)
}
fn map_value(d: usize, e: Equality) -> Equality {
    let left = value(d, e.left.clone());
    let right = value(d, e.right.clone());
    e.mapped(
        lam("_numeric_word", cube(d), value(d, var("_numeric_word"))),
        left,
        right,
        nat(),
    )
}

pub(super) fn append(numeric: &mut Numeric<'_>) -> Result<()> {
    numeric.powers()?;
    numeric.push_laws()?;
    numeric.zero_values()?;
    numeric.shift_combine()?;
    numeric.digits_combine()?;
    numeric.product_combine()?;
    numeric.literal_value_laws()?;
    for width in WIDTHS {
        numeric.push_family(width)?;
        numeric.shift_family(width)?;
        numeric.digits_family(width)?;
        numeric.product_family(width)?;
    }
    Ok(())
}

pub(super) fn append_division_prerequisites(numeric: &mut Numeric<'_>) -> Result<()> {
    numeric.push_laws()?;
    for width in WIDTHS {
        numeric.push_family(width)?;
    }
    Ok(())
}
