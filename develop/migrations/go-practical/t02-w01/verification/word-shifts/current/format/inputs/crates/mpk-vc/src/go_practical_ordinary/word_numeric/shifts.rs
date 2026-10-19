//! Actual full-count shifts, with ordinary numeric witnesses and source binding.
use super::*;
mod counts;
mod fixed_right;
mod pop;
mod powers;
const SHIFTS: &str = "Mpk.Go.Practical.Word.Shifts";
fn sh(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{SHIFTS}.{name}"), args)
}
fn bool_map(n: &str, e: Equality) -> Equality {
    let (l, r) = (bc(n, vec![e.left.clone()]), bc(n, vec![e.right.clone()]));
    e.mapped(
        lam("_shift_flag", boolean(), bc(n, vec![var("_shift_flag")])),
        l,
        r,
        boolean(),
    )
}
fn bool_left(n: &str, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        bc(n, vec![e.left.clone(), b.clone()]),
        bc(n, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_shift_flag", boolean(), bc(n, vec![var("_shift_flag"), b])),
        l,
        r,
        boolean(),
    )
}
fn bool_right(n: &str, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc(n, vec![a.clone(), e.left.clone()]),
        bc(n, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_shift_flag", boolean(), bc(n, vec![a, var("_shift_flag")])),
        l,
        r,
        boolean(),
    )
}
fn equal_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        equal(e.left.clone(), b.clone()),
        equal(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_shift_nat", nat(), equal(var("_shift_nat"), b)),
        l,
        r,
        boolean(),
    )
}
fn power(n: Expr) -> Expr {
    numeric("PowerTwo", vec![n])
}
fn numeral(n: usize) -> Expr {
    (0..n).fold(zero(), |v, _| succ(v))
}
fn coefficient(n: usize) -> Expr {
    power(numeral(n))
}
fn minus(n: Expr) -> Expr {
    sh("PowerMinusOne", vec![n])
}
fn law(n: &str, args: Vec<Expr>, a: Expr, b: Expr) -> Equality {
    Equality::natural(a, b, nc(n, args))
}
fn known(n: &str, args: Vec<Expr>, a: Expr, b: Expr) -> Equality {
    Equality::natural(a, b, sh(n, args))
}
fn map_power(e: Equality) -> Equality {
    let (l, r) = (power(e.left.clone()), power(e.right.clone()));
    e.mapped(
        lam("_shift_exponent", nat(), power(var("_shift_exponent"))),
        l,
        r,
        nat(),
    )
}
fn map_cube(d: usize, e: Equality) -> Equality {
    let (l, r) = (value(d, e.left.clone()), value(d, e.right.clone()));
    e.mapped(
        lam("_shift_cube", cube(d), value(d, var("_shift_cube"))),
        l,
        r,
        nat(),
    )
}
fn cube_equation(d: usize, a: Expr, b: Expr) -> Equality {
    map_cube(
        d,
        Equality {
            ty: cube(d),
            left: a,
            right: b.clone(),
            proof: refl(cube(d), b),
        },
    )
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_shift_left", nat(), less(var("_shift_left"), b)),
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
        lam("_shift_right", nat(), less(a, var("_shift_right"))),
        l,
        r,
        boolean(),
    )
}
fn cube_address(d: usize, i: usize) -> Vec<Expr> {
    (0..d).rev().map(|j| bit(i & (1 << j) != 0)).collect()
}
fn cube_at(d: usize, x: Expr, i: usize) -> Expr {
    app(x, cube_address(d, i))
}
struct Shifts<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Shifts<'_> {
    fn emit(&mut self, n: &str, args: &[Binder], ty: Expr, p: Expr, theorem: bool) -> Result<()> {
        emit(self.builder, &format!("{SHIFTS}.{n}"), args, ty, p, theorem)
    }
    fn theorem(&mut self, n: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(n, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], a: Expr, b: Expr) -> Result<Equality> {
        if a == b {
            return Ok(Equality::reflexive(a));
        }
        polynomial::prove_named(self.builder, args, &mut self.serial, SHIFTS, a, b)
    }
    fn literal_values(&self, d: usize, a: Expr, b: Expr) -> Equality {
        let mut args = vec![a.clone(), b.clone()];
        for i in 0..1 << d {
            args.push(refl(boolean(), cube_at(d, b.clone(), i)));
        }
        Equality::natural(
            value(d, a),
            value(d, b),
            if d >= 3 {
                numeric(&format!("LiteralValueCongruence{}", 1 << d), args)
            } else {
                sh(&format!("LiteralCubeValue{d}"), args)
            },
        )
    }
    fn literal_laws(&mut self) -> Result<()> {
        for d in 0..3 {
            let (x, y) = (var("left_word"), var("right_word"));
            let names = &ADDRESSES[..d];
            let prop = |e: &BTreeMap<&'static str, Expr>| {
                let a = names.iter().map(|n| e[n].clone()).collect::<Vec<_>>();
                eq(boolean(), app(x.clone(), a.clone()), app(y.clone(), a))
            };
            let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let mut i = 0;
                for n in names {
                    i = 2 * i + usize::from(e[n] == bit(true));
                }
                Ok(var(&format!("_literal_equality{i}")))
            };
            let point = boolean_cases(names, &BTreeMap::new(), &prop, &mut leaf)?;
            let mut proof = numeric(
                &format!("ValueCongruence{d}"),
                vec![x.clone(), y.clone(), point],
            );
            let mut ty = eq(nat(), value(d, x.clone()), value(d, y.clone()));
            for i in (0..1 << d).rev() {
                let n = format!("_literal_equality{i}");
                let p = eq(
                    boolean(),
                    cube_at(d, x.clone(), i),
                    cube_at(d, y.clone(), i),
                );
                ty = pi(&n, p.clone(), ty);
                proof = lam(&n, p, proof);
            }
            self.emit(
                &format!("LiteralCubeValue{d}"),
                &[("left_word", cube(d)), ("right_word", cube(d))],
                ty,
                proof,
                true,
            )?;
        }
        Ok(())
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut s = Shifts { builder, serial: 0 };
    s.literal_laws()?;
    s.powers()?;
    s.pop_combine()?;
    for d in 0..=6 {
        s.pop(d)?;
    }
    s.right_combine()?;
    for w in WIDTHS {
        s.fixed_right(w)?;
    }
    s.counts()?;
    Ok(())
}
