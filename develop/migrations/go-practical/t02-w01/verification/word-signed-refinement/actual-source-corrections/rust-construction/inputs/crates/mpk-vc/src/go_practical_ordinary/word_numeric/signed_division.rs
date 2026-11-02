//! Actual signed quotient/remainder and explicit, computed modular corrections.
use super::*;
mod family;
mod projection;
mod scalar;
mod selection;

const SIGNED_DIVISION: &str = "Mpk.Go.Practical.Word.SignedDivision";
fn sd(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{SIGNED_DIVISION}.{name}"), args)
}
fn prerequisite(role: &str, name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("{SIGNED_DIVISION}.Prerequisite.{role}.{name}"),
        args,
    )
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn integer() -> Expr {
    pi("_part", boolean(), nat())
}
fn part(negative: bool, x: Expr) -> Expr {
    ic(if negative { "Negative" } else { "Positive" }, vec![x])
}
fn equivalent(a: Expr, b: Expr) -> Expr {
    ic("Equivalent", vec![a, b])
}
fn signed_nat(flag: Expr, amount: Expr) -> Expr {
    sd("SignedNat", vec![flag, amount])
}
fn signed(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}
fn sign(w: usize, x: Expr) -> Result<Expr> {
    at(x, w, w - 1)
}
fn selection(w: usize, x: Expr, flag: Expr) -> Expr {
    wc(w, "Select", vec![flag, wc(w, "Neg", vec![x.clone()]), x])
}
fn correction(w: usize, x: Expr, flag: Expr) -> Expr {
    sd(&format!("W{w}.SelectionCorrection"), vec![x, flag])
}
fn cross(a: Expr, b: Expr) -> (Expr, Expr) {
    (
        add(part(false, a.clone()), part(true, b.clone())),
        add(part(false, b), part(true, a)),
    )
}
fn literal_flag(flag: &Expr) -> Option<bool> {
    if *flag == bit(true) {
        return Some(true);
    }
    if *flag == bit(false) {
        return Some(false);
    }
    let Expr::Apply(head, args) = flag else {
        return None;
    };
    if **head != constant("Mpk.Go.Practical.Bool.Xor") {
        return None;
    }
    let [a, b] = args.as_slice() else {
        return None;
    };
    Some(literal_flag(a)? != literal_flag(b)?)
}
fn natural_choice(flag: Expr, yes: Expr, no: Expr) -> Equality {
    let left = choose(flag.clone(), yes.clone(), no.clone(), nat());
    let right = match literal_flag(&flag) {
        Some(true) => yes,
        Some(false) => no,
        None => return Equality::reflexive(left),
    };
    Equality::natural(left, right.clone(), refl(nat(), right))
}

struct Relation {
    left: Expr,
    right: Expr,
    proof: Expr,
}
impl Relation {
    fn new(left: Expr, right: Expr, proof: Expr) -> Self {
        Self { left, right, proof }
    }
    fn reflexive(value: Expr) -> Self {
        Self::new(value.clone(), value.clone(), ic("Reflexive", vec![value]))
    }
    fn reverse(self) -> Self {
        Self::new(
            self.right.clone(),
            self.left.clone(),
            ic("Symmetric", vec![self.left, self.right, self.proof]),
        )
    }
    fn then(self, other: Self) -> Result<Self> {
        if self.right != other.left {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(Self::new(
            self.left.clone(),
            other.right.clone(),
            ic(
                "Transitive",
                vec![self.left, self.right, other.right, self.proof, other.proof],
            ),
        ))
    }
    fn times_right(self, right: Expr) -> Self {
        Self::new(
            ic("Mul", vec![self.left.clone(), right.clone()]),
            ic("Mul", vec![self.right.clone(), right.clone()]),
            ic(
                "MulRespectsLeft",
                vec![self.left, self.right, right, self.proof],
            ),
        )
    }
    fn times_left(self, left: Expr) -> Self {
        Self::new(
            ic("Mul", vec![left.clone(), self.left.clone()]),
            ic("Mul", vec![left.clone(), self.right.clone()]),
            ic(
                "MulRespectsRight",
                vec![self.left, self.right, left, self.proof],
            ),
        )
    }
    fn plus(self, other: Self) -> Self {
        Self::new(
            ic("Add", vec![self.left.clone(), other.left.clone()]),
            ic("Add", vec![self.right.clone(), other.right.clone()]),
            ic(
                "AddRespects",
                vec![
                    self.left,
                    other.left,
                    self.right,
                    other.right,
                    self.proof,
                    other.proof,
                ],
            ),
        )
    }
    fn replace_left(self, e: Equality) -> Result<Self> {
        if e.ty != integer() || e.right != self.left {
            return Err(GoOrdinaryError::Linkage);
        }
        let (new_left, new_right) = cross(e.left.clone(), self.right.clone());
        let (old_left, old_right) = cross(self.left.clone(), self.right.clone());
        let prefix = e.clone().mapped(
            lam(
                "_related_left",
                integer(),
                add(
                    part(false, var("_related_left")),
                    part(true, self.right.clone()),
                ),
            ),
            new_left,
            old_left.clone(),
            nat(),
        );
        let suffix = e.clone().mapped(
            lam(
                "_related_left",
                integer(),
                add(
                    part(false, self.right.clone()),
                    part(true, var("_related_left")),
                ),
            ),
            new_right,
            old_right.clone(),
            nat(),
        );
        let proof = prefix
            .then(Equality::natural(old_left, old_right, self.proof))?
            .then(suffix.reverse())?
            .proof;
        Ok(Self::new(e.left, self.right, proof))
    }
    fn replace_right(self, e: Equality) -> Result<Self> {
        if e.ty != integer() || e.right != self.right {
            return Err(GoOrdinaryError::Linkage);
        }
        let (new_left, new_right) = cross(self.left.clone(), e.left.clone());
        let (old_left, old_right) = cross(self.left.clone(), self.right.clone());
        let prefix = e.clone().mapped(
            lam(
                "_related_right",
                integer(),
                add(
                    part(false, self.left.clone()),
                    part(true, var("_related_right")),
                ),
            ),
            new_left,
            old_left.clone(),
            nat(),
        );
        let suffix = e.clone().mapped(
            lam(
                "_related_right",
                integer(),
                add(
                    part(false, var("_related_right")),
                    part(true, self.left.clone()),
                ),
            ),
            new_right,
            old_right.clone(),
            nat(),
        );
        let proof = prefix
            .then(Equality::natural(old_left, old_right, self.proof))?
            .then(suffix.reverse())?
            .proof;
        Ok(Self::new(self.left, e.left, proof))
    }
}

struct Refinement<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Refinement<'_> {
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
            &format!("{SIGNED_DIVISION}.{name}"),
            args,
            result,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn relation(&mut self, name: &str, args: &[Binder], e: Relation) -> Result<()> {
        self.emit(name, args, equivalent(e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], left: Expr, right: Expr) -> Result<Equality> {
        polynomial::prove_named(
            self.builder,
            args,
            &mut self.serial,
            SIGNED_DIVISION,
            left,
            right,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut r = Refinement { builder, serial: 0 };
    r.signed_nat()?;
    r.scalar_division()?;
    for width in WIDTHS {
        r.selection(width)?;
        r.family(width)?;
    }
    Ok(())
}
