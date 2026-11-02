//! All frozen depths retain the full index and actual shape/range witnesses.
use super::*;

mod relations;
mod restoration;

const BITS: [&str; 8] = ["_a0", "_a1", "_a2", "_a3", "_a4", "_a5", "_a6", "_a7"];
fn ac(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Array.D{d}.{name}"), args)
}
fn fc(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn name(d: usize, suffix: &str) -> String { format!("ArrayD{d}{suffix}") }
fn cube(d: usize, body: Expr, lambdas: bool) -> Expr {
    BITS[..d].iter().rev().fold(body, |body, b| {
        if lambdas { lam(b, boolean(), body) } else { pi(b, boolean(), body) }
    })
}
fn address(d: usize) -> Vec<Expr> { BITS[..d].iter().map(|b| var(b)).collect() }
fn word() -> Expr { (0..6).fold(boolean(), |b, _| pi("_bit", boolean(), b)) }
fn value(w: Expr) -> Expr { call("Mpk.Go.Practical.Word.Numeric.Value6", vec![w]) }
fn inside(i: Expr, n: Expr) -> Expr { eq(boolean(), less(i, n), bit(true)) }
fn element(d: usize, a: Expr, i: Expr) -> Expr { lc(&name(d, "Element"), vec![a, i]) }
fn read(d: usize, a: Expr, s: Expr, i: Expr) -> Expr { ac(d, "Read", vec![a, s, i]) }
fn set(d: usize, a: Expr, s: Expr, i: Expr, x: Expr) -> Expr { ac(d, "Set", vec![a, s, i, x]) }
fn related(d: usize, a: Expr, r: Expr, n: Expr, s: Expr, t: Expr) -> Expr {
    ac(d, "Related", vec![a.clone(), a, r, n, s, t])
}
fn root_relation(d: usize, a: Expr, r: Expr, n: Expr) -> Expr {
    lam("_left", cube(d, a.clone(), false), lam("_right", cube(d, a.clone(), false),
        related(d, a, r, n, var("_left"), var("_right"))))
}
fn root_validity(d: usize, a: Expr, p: Expr, z: Expr, n: Expr) -> Expr {
    lam("_root", cube(d, a.clone(), false), ac(d, "Valid", vec![a, p, z, n, var("_root")]))
}
fn reflexive(a: Expr, r: Expr) -> Expr { pi("_value", a, holds(r, var("_value"), var("_value"))) }
fn points(d: usize, n: Expr, body: Expr) -> Expr {
    cube(d, lam("_in", inside(fc(d, "Index", address(d)), n), body), true)
}
fn decide(flag: Expr, target: Expr, no: Expr, yes: Expr) -> Expr {
    app(call("Std.Bool.cases", vec![
        lam("_tag", boolean(), pi("_case", eq(boolean(), flag.clone(), var("_tag")), target)),
        lam("_case", eq(boolean(), flag.clone(), bit(false)), no),
        lam("_case", eq(boolean(), flag.clone(), bit(true)), yes), flag.clone(),
    ]), vec![refl(boolean(), flag)])
}

impl Lenses<'_> {
    pub(super) fn arrays(&mut self) -> Result<()> {
        for d in 0..=8 {
            self.array_definitions(d)?;
            self.array_relations(d)?;
            self.array_restoration(d)?;
            self.array_validity(d)?;
        }
        Ok(())
    }
    fn array_definitions(&mut self, d: usize) -> Result<()> {
        let (a, i, n) = (var("A"), var("i"), var("n"));
        let root_ty = cube(d, a.clone(), false);
        let view = element(d, a.clone(), i.clone());
        let base = [("A", Expr::Sort0), ("i", word())];
        self.emit(&name(d, "Element"), &base, lens(root_ty.clone(), a.clone()),
            make(root_ty.clone(), a.clone(),
                lam("_s", root_ty.clone(), read(d, a.clone(), var("_s"), i.clone())),
                lam("_s", root_ty.clone(), lam("_x", a.clone(), set(d, a.clone(), var("_s"), i.clone(), var("_x"))))), false)?;
        let mut args = base.to_vec(); args.push(("s", root_ty.clone()));
        let g = read(d, a.clone(), var("s"), i.clone());
        self.theorem(&name(d, "ElementGet"), &args, Equality {
            ty: a.clone(), left: get(root_ty.clone(), a.clone(), view.clone(), var("s")),
            right: g.clone(), proof: refl(a.clone(), g),
        })?;
        args.push(("x", a.clone()));
        let p = set(d, a.clone(), var("s"), i.clone(), var("x"));
        self.theorem(&name(d, "ElementPut"), &args, Equality {
            ty: root_ty.clone(), left: put(root_ty.clone(), a.clone(), view.clone(), var("s"), var("x")),
            right: p.clone(), proof: refl(root_ty.clone(), p),
        })?;
        let args = [("A", Expr::Sort0), ("n", nat()), ("i", word()),
            ("shape", ac(d, "Shape", vec![n.clone()])), ("within", inside(value(i.clone()), n.clone()))];
        self.emit(&name(d, "ElementGetPut"), &args, get_put(root_ty.clone(), a.clone(), view.clone()),
            lam("_s", root_ty.clone(), lam("_x", a.clone(), ac(d, "ReadSetSame", vec![
                a.clone(), n.clone(), var("_s"), i.clone(), var("_x"), var("shape"), var("within"),
            ]))), true)?;
        let args = [("A", Expr::Sort0), ("n", nat()), ("i", word()), ("j", word()),
            ("shape", ac(d, "Shape", vec![n.clone()])), ("within", inside(value(var("j")), n.clone())),
            ("different", eq(boolean(), equal(value(var("j")), value(i.clone())), bit(false))),
            ("s", root_ty.clone()), ("x", a.clone())];
        let other = element(d, a.clone(), var("j"));
        self.theorem(&name(d, "ElementSiblingFrame"), &args, Equality {
            ty: a.clone(),
            left: get(root_ty.clone(), a.clone(), other.clone(), put(root_ty.clone(), a.clone(), view, var("s"), var("x"))),
            right: get(root_ty, a.clone(), other, var("s")),
            proof: ac(d, "ReadSetOther", vec![a, n, var("s"), i, var("j"), var("x"), var("shape"), var("within"), var("different")]),
        })
    }
    fn array_validity(&mut self, d: usize) -> Result<()> {
        let (a, p, z, n, i) = (var("A"), var("P"), var("z"), var("n"), var("i"));
        let root_ty = cube(d, a.clone(), false);
        let view = element(d, a.clone(), i.clone());
        let root_pred = root_validity(d, a.clone(), p.clone(), z.clone(), n.clone());
        let args = [("A", Expr::Sort0), ("P", predicate(a.clone())), ("z", a.clone()), ("n", nat()),
            ("i", word()), ("shape", ac(d, "Shape", vec![n.clone()])), ("within", inside(value(i.clone()), n.clone()))];
        self.emit(&name(d, "ElementGetValid"), &args, get_valid(root_ty.clone(), a.clone(), root_pred.clone(), p.clone(), view.clone()),
            lam("_s", root_ty.clone(), lam("_valid", app(root_pred.clone(), vec![var("_s")]),
                ac(d, "ReadValid", vec![a.clone(), p.clone(), z.clone(), n.clone(), var("_s"), i.clone(), var("shape"), var("within"), var("_valid")]))), true)?;
        // SetValid needs the actual length bound; the raw view supplies none.
        let args = [args[..5].to_vec(), vec![("within", inside(value(i.clone()), n.clone()))]].concat();
        self.emit(&name(d, "ElementPutValid"), &args, put_valid(root_ty.clone(), a.clone(), root_pred.clone(), p.clone(), view),
            lam("_s", root_ty, lam("_x", a.clone(), lam("_valid", app(root_pred, vec![var("_s")]),
                lam("_value_valid", app(p.clone(), vec![var("_x")]), ac(d, "SetValid", vec![
                    a, p, z, n, var("_s"), i, var("_x"), var("_valid"), var("_value_valid"), var("within"),
                ]))))), true)
    }
}
