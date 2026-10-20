//! Actual source variable stages, with computed numeric witnesses.
use super::counts::{digits, high};
use super::*;
mod choices;
mod left;
fn select(f: Expr, a: Expr, b: Expr) -> Expr {
    choose(f, a, b, nat())
}
fn literal(b: bool, a: Expr, c: Expr) -> Equality {
    let target = if b { a.clone() } else { c.clone() };
    Equality::natural(select(bit(b), a, c), target.clone(), refl(nat(), target))
}
fn choice_left(f: Expr, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        select(f.clone(), e.left.clone(), b.clone()),
        select(f.clone(), e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_chosen_nat", nat(), select(f, var("_chosen_nat"), b)),
        l,
        r,
        nat(),
    )
}
fn choice_condition(e: Equality, a: Expr, b: Expr) -> Equality {
    let (l, r) = (
        select(e.left.clone(), a.clone(), b.clone()),
        select(e.right.clone(), a.clone(), b.clone()),
    );
    e.mapped(
        lam("_choice_flag", boolean(), select(var("_choice_flag"), a, b)),
        l,
        r,
        nat(),
    )
}
fn stage(w: usize, i: usize, x: Expr, c: Expr) -> Expr {
    wc(w, &format!("ShiftLeft{i}"), vec![x, c])
}
fn discarded(w: usize, i: usize, x: Expr, c: Expr) -> Expr {
    sh(&format!("W{w}.LeftStageDiscard{i}"), vec![x, c])
}
fn full_discard(w: usize, x: Expr, c: Expr) -> Expr {
    sh(&format!("W{w}.LeftDiscard"), vec![x, c])
}
impl Shifts<'_> {
    pub(super) fn variable(&mut self) -> Result<()> {
        self.variable_power_choices()?;
        self.left_compose()?;
        self.left_choice()?;
        for w in WIDTHS {
            self.variable_select_value(w)?;
            self.variable_left(w)?;
        }
        Ok(())
    }
}
