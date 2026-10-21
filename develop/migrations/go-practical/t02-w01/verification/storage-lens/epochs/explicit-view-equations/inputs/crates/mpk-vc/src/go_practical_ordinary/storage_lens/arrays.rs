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
fn name(d: usize, suffix: &str) -> String {
    format!("ArrayD{d}{suffix}")
}
fn cube(d: usize, body: Expr, lambdas: bool) -> Expr {
    BITS[..d].iter().rev().fold(body, |body, b| {
        if lambdas {
            lam(b, boolean(), body)
        } else {
            pi(b, boolean(), body)
        }
    })
}
fn address(d: usize) -> Vec<Expr> {
    BITS[..d].iter().map(|b| var(b)).collect()
}
fn word() -> Expr {
    (0..6).fold(boolean(), |b, _| pi("_bit", boolean(), b))
}
fn value(w: Expr) -> Expr {
    call("Mpk.Go.Practical.Word.Numeric.Value6", vec![w])
}
fn inside(i: Expr, n: Expr) -> Expr {
    eq(boolean(), less(i, n), bit(true))
}
fn element(d: usize, a: Expr, i: Expr) -> Expr {
    lc(&name(d, "Element"), vec![a, i])
}
fn read(d: usize, a: Expr, s: Expr, i: Expr) -> Expr {
    ac(d, "Read", vec![a, s, i])
}
fn set(d: usize, a: Expr, s: Expr, i: Expr, x: Expr) -> Expr {
    ac(d, "Set", vec![a, s, i, x])
}
fn related(d: usize, a: Expr, r: Expr, n: Expr, s: Expr, t: Expr) -> Expr {
    ac(d, "Related", vec![a.clone(), a, r, n, s, t])
}
fn root_relation(d: usize, a: Expr, r: Expr, n: Expr) -> Expr {
    lam(
        "_left",
        cube(d, a.clone(), false),
        lam(
            "_right",
            cube(d, a.clone(), false),
            related(d, a, r, n, var("_left"), var("_right")),
        ),
    )
}
fn root_validity(d: usize, a: Expr, p: Expr, z: Expr, n: Expr) -> Expr {
    lam(
        "_root",
        cube(d, a.clone(), false),
        ac(d, "Valid", vec![a, p, z, n, var("_root")]),
    )
}
fn reflexive(a: Expr, r: Expr) -> Expr {
    pi("_value", a, holds(r, var("_value"), var("_value")))
}
fn points(d: usize, n: Expr, body: Expr) -> Expr {
    cube(
        d,
        lam("_in", inside(fc(d, "Index", address(d)), n), body),
        true,
    )
}
fn get_equation(d: usize, a: Expr, i: Expr, s: Expr) -> Equality {
    Equality {
        ty: a.clone(),
        left: get(cube(d, a.clone(), false), a.clone(), element(d, a.clone(), i.clone()), s.clone()),
        right: read(d, a.clone(), s.clone(), i.clone()),
        proof: lc(&name(d, "ElementGet"), vec![a, i, s]),
    }
}
fn put_equation(d: usize, a: Expr, i: Expr, s: Expr, x: Expr) -> Equality {
    Equality {
        ty: cube(d, a.clone(), false),
        left: put(cube(d, a.clone(), false), a.clone(), element(d, a.clone(), i.clone()), s.clone(), x.clone()),
        right: set(d, a.clone(), s.clone(), i.clone(), x.clone()),
        proof: lc(&name(d, "ElementPut"), vec![a, i, s, x]),
    }
}
fn read_after_put(d: usize, a: Expr, i: Expr, j: Expr, s: Expr, x: Expr) -> Result<Equality> {
    let changed = put_equation(d, a.clone(), i, s, x);
    let first = get_equation(d, a.clone(), j.clone(), changed.left.clone());
    let second = changed.clone().mapped(
        lam("_changed_array", cube(d, a.clone(), false), read(d, a.clone(), var("_changed_array"), j.clone())),
        read(d, a.clone(), changed.left, j.clone()), read(d, a.clone(), changed.right, j), a,
    );
    first.then(second)
}
fn decide(flag: Expr, target: Expr, no: Expr, yes: Expr) -> Expr {
    app(
        call(
            "Std.Bool.cases",
            vec![
                lam(
                    "_tag",
                    boolean(),
                    pi("_case", eq(boolean(), flag.clone(), var("_tag")), target),
                ),
                lam("_case", eq(boolean(), flag.clone(), bit(false)), no),
                lam("_case", eq(boolean(), flag.clone(), bit(true)), yes),
                flag.clone(),
            ],
        ),
        vec![refl(boolean(), flag)],
    )
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
        self.emit(
            &name(d, "Element"),
            &base,
            lens(root_ty.clone(), a.clone()),
            make(
                root_ty.clone(),
                a.clone(),
                lam(
                    "_s",
                    root_ty.clone(),
                    read(d, a.clone(), var("_s"), i.clone()),
                ),
                lam(
                    "_s",
                    root_ty.clone(),
                    lam(
                        "_x",
                        a.clone(),
                        set(d, a.clone(), var("_s"), i.clone(), var("_x")),
                    ),
                ),
            ),
            false,
        )?;
        let mut args = base.to_vec();
        args.push(("s", root_ty.clone()));
        let g = read(d, a.clone(), var("s"), i.clone());
        self.theorem(
            &name(d, "ElementGet"),
            &args,
            Equality {
                ty: a.clone(),
                left: get(root_ty.clone(), a.clone(), view.clone(), var("s")),
                right: g.clone(),
                proof: refl(a.clone(), g),
            },
        )?;
        args.push(("x", a.clone()));
        let p = set(d, a.clone(), var("s"), i.clone(), var("x"));
        self.theorem(
            &name(d, "ElementPut"),
            &args,
            Equality {
                ty: root_ty.clone(),
                left: put(root_ty.clone(), a.clone(), view.clone(), var("s"), var("x")),
                right: p.clone(),
                proof: refl(root_ty.clone(), p),
            },
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("i", word()),
            ("shape", ac(d, "Shape", vec![n.clone()])),
            ("within", inside(value(i.clone()), n.clone())),
        ];
        let e = read_after_put(d, a.clone(), i.clone(), i.clone(), var("_s"), var("_x"))?
            .then(Equality {
                ty: a.clone(),
                left: read(d, a.clone(), set(d, a.clone(), var("_s"), i.clone(), var("_x")), i.clone()),
                right: var("_x"),
                proof: ac(d, "ReadSetSame", vec![a.clone(), n.clone(), var("_s"), i.clone(), var("_x"), var("shape"), var("within")]),
            })?;
        self.emit(
            &name(d, "ElementGetPut"),
            &args,
            get_put(root_ty.clone(), a.clone(), view.clone()),
            lam(
                "_s",
                root_ty.clone(),
                lam(
                    "_x",
                    a.clone(),
                    e.proof,
                ),
            ),
            true,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("i", word()),
            ("j", word()),
            ("shape", ac(d, "Shape", vec![n.clone()])),
            ("within", inside(value(var("j")), n.clone())),
            (
                "different",
                eq(
                    boolean(),
                    equal(value(var("j")), value(i.clone())),
                    bit(false),
                ),
            ),
            ("s", root_ty.clone()),
            ("x", a.clone()),
        ];
        let e = read_after_put(d, a.clone(), i.clone(), var("j"), var("s"), var("x"))?
            .then(Equality {
                ty: a.clone(),
                left: read(d, a.clone(), set(d, a.clone(), var("s"), i.clone(), var("x")), var("j")),
                right: read(d, a.clone(), var("s"), var("j")),
                proof: ac(d, "ReadSetOther", vec![a.clone(), n, var("s"), i.clone(), var("j"), var("x"), var("shape"), var("within"), var("different")]),
            })?.then(get_equation(d, a, var("j"), var("s")).reverse())?;
        self.theorem(
            &name(d, "ElementSiblingFrame"),
            &args,
            e,
        )
    }
    fn array_validity(&mut self, d: usize) -> Result<()> {
        let (a,p,z,n,i)=(var("A"),var("P"),var("z"),var("n"),var("i"));
        let root_ty=cube(d,a.clone(),false);
        let view=element(d,a.clone(),i.clone());
        let root_pred=root_validity(d,a.clone(),p.clone(),z.clone(),n.clone());
        let args=[("A",Expr::Sort0),("P",predicate(a.clone())),("z",a.clone()),("n",nat()),("i",word()),
            ("shape",ac(d,"Shape",vec![n.clone()])),("within",inside(value(i.clone()),n.clone()))];
        let read_valid=ac(d,"ReadValid",vec![a.clone(),p.clone(),z.clone(),n.clone(),var("_s"),i.clone(),var("shape"),var("within"),var("_valid")]);
        let proof=retype(a.clone(),p.clone(),get_equation(d,a.clone(),i.clone(),var("_s")).reverse(),read_valid);
        self.emit(&name(d,"ElementGetValid"),&args,get_valid(root_ty.clone(),a.clone(),root_pred.clone(),p.clone(),view.clone()),
            lam("_s",root_ty.clone(),lam("_valid",app(root_pred.clone(),vec![var("_s")]),proof)),true)?;
        let args=[args[..5].to_vec(),vec![("within",inside(value(i.clone()),n.clone()))]].concat();
        let write_valid=ac(d,"SetValid",vec![a.clone(),p.clone(),z,n,var("_s"),i.clone(),var("_x"),var("_valid"),var("_value_valid"),var("within")]);
        let proof=retype(root_ty.clone(),root_pred.clone(),put_equation(d,a.clone(),i,var("_s"),var("_x")).reverse(),write_valid);
        self.emit(&name(d,"ElementPutValid"),&args,put_valid(root_ty.clone(),a.clone(),root_pred.clone(),p.clone(),view),
            lam("_s",root_ty,lam("_x",a,lam("_valid",app(root_pred,vec![var("_s")]),lam("_value_valid",app(p,vec![var("_x")]),proof)))),true)
    }
}
