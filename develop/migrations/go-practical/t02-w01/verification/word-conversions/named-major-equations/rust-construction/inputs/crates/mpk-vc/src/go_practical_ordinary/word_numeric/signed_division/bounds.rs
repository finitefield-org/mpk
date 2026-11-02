//! Actual quotient/half gaps, exact remainder and the negative-unit exception.
use super::*;
mod correction;
mod exception;
mod family;
mod natural;
mod signs;

const BOUNDED: &str = "Mpk.Go.Practical.Word.SignedDivision.Bounds";
fn db(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{BOUNDED}.{name}"), args)
}
fn half(d: usize) -> Expr {
    prerequisite("Bounds", &format!("Half{d}"), vec![])
}
fn magnitude_gap(w: usize, x: Expr) -> Expr {
    prerequisite("Bounds", &format!("W{w}.MagnitudeGap"), vec![x])
}
fn amount_word(w: usize, quotient: bool, x: Expr, y: Expr) -> Expr {
    wc(
        w,
        if quotient {
            "UnsignedDiv"
        } else {
            "UnsignedRem"
        },
        vec![wc(w, "AbsBits", vec![x]), wc(w, "AbsBits", vec![y])],
    )
}
fn qgap(d: Expr, q: Expr, r: Expr, gap: Expr) -> Expr {
    add(add(mul(sub(d, one()), q), r), gap)
}
fn strict_gap(x: Expr, h: Expr) -> Expr {
    add(sub(h, succ(x)), one())
}
fn gate(flag: Expr, threshold: Expr) -> Expr {
    bc("And", vec![bc("Not", vec![flag]), threshold])
}
fn le(a: Expr, b: Expr) -> Expr {
    nc("LessEqual", vec![a, b])
}
fn minus_one() -> Expr {
    ic("Make", vec![zero(), one()])
}
fn field(w: usize, name: &str, x: Expr, y: Expr) -> Expr {
    db(&format!("W{w}.{name}"), vec![x, y])
}
fn old_field(w: usize, name: &str, x: Expr, y: Expr) -> Expr {
    sd(&format!("W{w}.{name}"), vec![x, y])
}
fn quotient_flag(w: usize, x: Expr, y: Expr) -> Result<Expr> {
    Ok(bc("Xor", vec![sign(w, x)?, sign(w, y)?]))
}
fn negative_unit(w: usize, x: Expr) -> Expr {
    db(&format!("W{w}.NegativeUnit"), vec![x])
}
fn overflow(w: usize, x: Expr, y: Expr) -> Expr {
    field(w, "QuotientOverflow", x, y)
}
fn minimum_case(w: usize, x: Expr, y: Expr) -> Expr {
    field(w, "MinimumMinusOne", x, y)
}
fn qgap_value(w: usize, x: Expr, y: Expr) -> Expr {
    field(w, "QuotientGap", x, y)
}
fn from_nat_equality(e: Equality) -> Equality {
    let (l, r) = (
        ic("FromNat", vec![e.left.clone()]),
        ic("FromNat", vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_bound_amount",
            nat(),
            ic("FromNat", vec![var("_bound_amount")]),
        ),
        l,
        r,
        integer(),
    )
}
fn nlaw(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_bound_n", nat(), less(var("_bound_n"), b)),
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
        lam("_bound_n", nat(), less(a, var("_bound_n"))),
        l,
        r,
        boolean(),
    )
}
fn bnot(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_bound_flag",
            boolean(),
            bc("Not", vec![var("_bound_flag")]),
        ),
        l,
        r,
        boolean(),
    )
}
fn bool_binary(name: &str, a: Equality, b: Equality) -> Result<Equality> {
    let first = a.clone().mapped(
        lam(
            "_left_flag",
            boolean(),
            bc(name, vec![var("_left_flag"), b.left.clone()]),
        ),
        bc(name, vec![a.left.clone(), b.left.clone()]),
        bc(name, vec![a.right.clone(), b.left.clone()]),
        boolean(),
    );
    first.then(b.clone().mapped(
        lam(
            "_right_flag",
            boolean(),
            bc(name, vec![a.right.clone(), var("_right_flag")]),
        ),
        bc(name, vec![a.right.clone(), b.left]),
        bc(name, vec![a.right, b.right]),
        boolean(),
    ))
}
fn both(a: Equality, b: Equality) -> Result<Equality> {
    if a.right != bit(true) || b.right != bit(true) {
        return Err(GoOrdinaryError::Linkage);
    }
    bool_binary("And", a, b)?.then(Equality::boolean(
        bc("And", vec![bit(true), bit(true)]),
        bit(true),
        refl(boolean(), bit(true)),
    ))
}
fn not_true_false(x: Expr, proof: Expr) -> Result<Equality> {
    Equality::boolean(
        bc("Not", vec![bc("Not", vec![x.clone()])]),
        x.clone(),
        bc("DoubleNot", vec![x.clone()]),
    )
    .reverse()
    .then(bnot(Equality::boolean(
        bc("Not", vec![x]),
        bit(true),
        proof,
    )))?
    .then(Equality::boolean(
        bc("Not", vec![bit(true)]),
        bit(false),
        refl(boolean(), bit(false)),
    ))
}
fn make_projection(negative: bool, p: Expr, n: Expr) -> Equality {
    let rhs = if negative { n.clone() } else { p.clone() };
    Equality::natural(
        part(negative, ic("Make", vec![p.clone(), n.clone()])),
        rhs,
        ic(
            if negative {
                "NegativeOfMake"
            } else {
                "PositiveOfMake"
            },
            vec![p, n],
        ),
    )
}
fn make_equality(p: Equality, n: Equality) -> Result<Equality> {
    let first = p.clone().mapped(
        lam(
            "_positive",
            nat(),
            ic("Make", vec![var("_positive"), n.left.clone()]),
        ),
        ic("Make", vec![p.left.clone(), n.left.clone()]),
        ic("Make", vec![p.right.clone(), n.left.clone()]),
        integer(),
    );
    first.then(n.clone().mapped(
        lam(
            "_negative",
            nat(),
            ic("Make", vec![p.right.clone(), var("_negative")]),
        ),
        ic("Make", vec![p.right.clone(), n.left]),
        ic("Make", vec![p.right, n.right]),
        integer(),
    ))
}
fn zero_sum(negative: bool, a: Expr, b: Expr, proof: Expr) -> Equality {
    Equality::natural(
        if negative { b.clone() } else { a.clone() },
        zero(),
        prerequisite(
            "Arithmetic",
            if negative {
                "ZeroSumRight"
            } else {
                "ZeroSumLeft"
            },
            vec![a, b, proof],
        ),
    )
}

struct Bounded<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Bounded<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        result: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.builder,
            &format!("{BOUNDED}.{name}"),
            args,
            result,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn relation(&mut self, name: &str, args: &[Binder], r: Relation) -> Result<()> {
        self.emit(name, args, equivalent(r.left, r.right), r.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], l: Expr, r: Expr) -> Result<Equality> {
        polynomial::prove_named(self.builder, args, &mut self.serial, BOUNDED, l, r)
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut b = Bounded { builder, serial: 0 };
    b.natural_bounds()?;
    b.signs()?;
    b.correction_laws()?;
    b.exception_laws()?;
    for w in WIDTHS {
        b.bounded_word(w)?;
        b.word_correction(w)?;
        b.family(w)?;
        b.exception(w)?;
    }
    Ok(())
}
