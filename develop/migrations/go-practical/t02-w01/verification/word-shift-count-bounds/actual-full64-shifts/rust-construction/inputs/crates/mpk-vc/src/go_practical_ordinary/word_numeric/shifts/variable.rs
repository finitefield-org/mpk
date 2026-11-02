//! Actual source variable stages, with computed numeric witnesses.
use super::counts::{digits, high};
use super::*;
mod choices;
mod left;
mod right;
mod right_bounds;
mod right_choices;
mod saturation;
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
fn map_minus(e: Equality) -> Equality {
    let (l, r) = (minus(e.left.clone()), minus(e.right.clone()));
    e.mapped(
        lam("_minus_value", nat(), minus(var("_minus_value"))),
        l,
        r,
        nat(),
    )
}
fn choice_right(f: Expr, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        select(f.clone(), a.clone(), e.left.clone()),
        select(f.clone(), a.clone(), e.right.clone()),
    );
    e.mapped(
        lam("_chosen_nat", nat(), select(f, a, var("_chosen_nat"))),
        l,
        r,
        nat(),
    )
}
fn boolean_choice_left(f: Expr, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        choose(f.clone(), e.left.clone(), b.clone(), boolean()),
        choose(f.clone(), e.right.clone(), b.clone(), boolean()),
    );
    e.mapped(
        lam(
            "_selected_bit",
            boolean(),
            choose(f, var("_selected_bit"), b, boolean()),
        ),
        l,
        r,
        boolean(),
    )
}
fn boolean_choice_right(f: Expr, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        choose(f.clone(), a.clone(), e.left.clone(), boolean()),
        choose(f.clone(), a.clone(), e.right.clone(), boolean()),
    );
    e.mapped(
        lam(
            "_selected_bit",
            boolean(),
            choose(f, a, var("_selected_bit"), boolean()),
        ),
        l,
        r,
        boolean(),
    )
}
fn right_kind(signed: bool) -> &'static str {
    if signed {
        "SignedRight"
    } else {
        "UnsignedRight"
    }
}
fn right_stage(w: usize, signed: bool, i: usize, x: Expr, c: Expr) -> Expr {
    wc(
        w,
        &format!(
            "{}{i}",
            if signed {
                "SignedShiftRight"
            } else {
                "ShiftRight"
            }
        ),
        vec![x, c],
    )
}
fn right_remainder(w: usize, signed: bool, i: usize, x: Expr, c: Expr) -> Expr {
    sh(
        &format!("W{w}.{}StageRemainder{i}", right_kind(signed)),
        vec![x, c],
    )
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
        self.variable_minus_choices()?;
        self.right_choice()?;
        self.variable_right_bounds()?;
        self.right_boolean_choices()?;
        for w in WIDTHS {
            self.variable_select_top(w)?;
            self.variable_right(w, false)?;
            self.variable_right(w, true)?;
        }
        self.right_saturation_scalar()?;
        self.right_saturation_bound()?;
        self.bit_select_value()?;
        for w in WIDTHS {
            self.right_fill(w)?;
            self.full_right(w, false)?;
            self.full_right(w, true)?;
        }
        Ok(())
    }
}
