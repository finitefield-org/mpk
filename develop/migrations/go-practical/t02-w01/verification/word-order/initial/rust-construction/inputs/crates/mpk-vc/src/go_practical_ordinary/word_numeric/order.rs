//! Actual unsigned word bounds/order, using genuine Nat and actual carries.
//! These extensions have a disjoint namespace from the arithmetic component.
use super::*;

mod bounds;
mod compare;

const ORDER: &str = "Mpk.Go.Practical.Word.Order";
fn oc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{ORDER}.{name}"), args)
}
fn instance(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, oc(name, args))
}
fn map_value(d: usize, e: Equality) -> Equality {
    let (left, right) = (value(d, e.left.clone()), value(d, e.right.clone()));
    e.mapped(
        lam("_order_word", cube(d), value(d, var("_order_word"))),
        left,
        right,
        nat(),
    )
}
fn bool_map(name: &str, e: Equality) -> Equality {
    let (left, right) = (
        bc(name, vec![e.left.clone()]),
        bc(name, vec![e.right.clone()]),
    );
    e.mapped(
        lam("_order_flag", boolean(), bc(name, vec![var("_order_flag")])),
        left,
        right,
        boolean(),
    )
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (left, right) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam(
            "_order_flag",
            boolean(),
            bc(name, vec![var("_order_flag"), b]),
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
            "_order_flag",
            boolean(),
            bc(name, vec![a, var("_order_flag")]),
        ),
        left,
        right,
        boolean(),
    )
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (left, right) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_order_n", nat(), less(var("_order_n"), b)),
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
        lam("_order_n", nat(), less(a, var("_order_n"))),
        left,
        right,
        boolean(),
    )
}

struct Order<'a> {
    builder: &'a mut Builder,
    recipes: &'a BTreeMap<String, Expr>,
    serial: usize,
}
impl Order<'_> {
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
            &format!("{ORDER}.{name}"),
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
        polynomial::prove_named(self.builder, args, &mut self.serial, ORDER, left, right)
    }
}
pub(super) fn append(builder: &mut Builder, recipes: &BTreeMap<String, Expr>) -> Result<()> {
    let mut order = Order {
        builder,
        recipes,
        serial: 0,
    };
    order.complement_laws()?;
    order.compare_laws()?;
    for width in WIDTHS {
        order.bounds_family(width)?;
        order.compare_family(width)?;
    }
    Ok(())
}
