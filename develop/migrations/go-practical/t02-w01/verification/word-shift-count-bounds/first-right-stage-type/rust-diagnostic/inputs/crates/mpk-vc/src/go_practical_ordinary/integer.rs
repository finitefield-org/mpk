//! Balanced mathematical integers. Representatives are Bool -> Nat; integer
//! equality is a Nat cross-sum relation, never equality of those functions.
use super::polynomial::{self, mul_left, mul_right, split};
use super::syntax::*;
use super::{Builder, GoOrdinaryError, Result};

fn integer() -> Expr {
    pi("_part", boolean(), nat())
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn pos(x: Expr) -> Expr {
    ic("Positive", vec![x])
}
fn neg(x: Expr) -> Expr {
    ic("Negative", vec![x])
}
fn cross(x: Expr, y: Expr) -> (Expr, Expr) {
    (add(pos(x.clone()), neg(y.clone())), add(pos(y), neg(x)))
}
fn equivalent(x: Expr, y: Expr) -> Expr {
    ic("Equivalent", vec![x, y])
}

struct Integers<'a> {
    builder: &'a mut Builder,
    serial: usize,
    args: Vec<Binder>,
}

impl Integers<'_> {
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
            &format!("Mpk.Go.Practical.Int.{name}"),
            args,
            result,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], result: Expr, proof: Expr) -> Result<()> {
        self.emit(name, args, result, proof, true)
    }
    fn save(&mut self, e: Equality) -> Result<Equality> {
        polynomial::checkpoint(self.builder, &self.args, &mut self.serial, e)
    }
    fn polynomial(&mut self, left: Expr, right: Expr) -> Result<Equality> {
        polynomial::prove(self.builder, &self.args, &mut self.serial, left, right)
    }
    fn definitions(&mut self) -> Result<()> {
        let (p, n, x, y) = (var("positive"), var("negative"), var("x"), var("y"));
        let x1 = [("x", integer())];
        let xy = [("x", integer()), ("y", integer())];
        let make = lam(
            "_tag",
            boolean(),
            call(
                "Std.Bool.cases",
                vec![
                    lam("_case", boolean(), nat()),
                    p.clone(),
                    n.clone(),
                    var("_tag"),
                ],
            ),
        );
        self.emit(
            "Make",
            &[("positive", nat()), ("negative", nat())],
            integer(),
            make,
            false,
        )?;
        self.emit(
            "Positive",
            &x1,
            nat(),
            app(x.clone(), vec![bit(false)]),
            false,
        )?;
        self.emit(
            "Negative",
            &x1,
            nat(),
            app(x.clone(), vec![bit(true)]),
            false,
        )?;
        self.emit(
            "Zero",
            &[],
            integer(),
            ic("Make", vec![zero(), zero()]),
            false,
        )?;
        self.emit(
            "FromNat",
            &[("positive", nat())],
            integer(),
            ic("Make", vec![p.clone(), zero()]),
            false,
        )?;
        self.emit(
            "Negate",
            &x1,
            integer(),
            ic("Make", vec![neg(x.clone()), pos(x.clone())]),
            false,
        )?;
        self.emit(
            "Add",
            &xy,
            integer(),
            ic(
                "Make",
                vec![
                    add(pos(x.clone()), pos(y.clone())),
                    add(neg(x.clone()), neg(y.clone())),
                ],
            ),
            false,
        )?;
        self.emit(
            "Sub",
            &xy,
            integer(),
            ic("Add", vec![x.clone(), ic("Negate", vec![y.clone()])]),
            false,
        )?;
        self.emit(
            "Mul",
            &xy,
            integer(),
            ic(
                "Make",
                vec![
                    add(
                        mul(pos(x.clone()), pos(y.clone())),
                        mul(neg(x.clone()), neg(y.clone())),
                    ),
                    add(
                        mul(pos(x.clone()), neg(y.clone())),
                        mul(neg(x.clone()), pos(y.clone())),
                    ),
                ],
            ),
            false,
        )?;
        let (left, right) = cross(x.clone(), y.clone());
        self.emit(
            "Equivalent",
            &xy,
            Expr::Sort0,
            eq(nat(), left.clone(), right.clone()),
            false,
        )?;
        self.emit(
            "Equal",
            &xy,
            boolean(),
            equal(left.clone(), right.clone()),
            false,
        )?;
        self.emit("Less", &xy, boolean(), less(left, right), false)?;
        self.emit(
            "LessEqual",
            &xy,
            boolean(),
            bc("Not", vec![ic("Less", vec![y.clone(), x.clone()])]),
            false,
        )?;
        for (name, negative) in [("Positive", false), ("Negative", true)] {
            let pick = |a| if negative { neg(a) } else { pos(a) };
            let value = if negative { n.clone() } else { p.clone() };
            self.theorem(
                &format!("{name}OfMake"),
                &[("positive", nat()), ("negative", nat())],
                eq(
                    nat(),
                    pick(ic("Make", vec![p.clone(), n.clone()])),
                    value.clone(),
                ),
                refl(nat(), value),
            )?;
            self.theorem(
                &format!("{name}OfZero"),
                &[],
                eq(nat(), pick(ic("Zero", vec![])), zero()),
                refl(nat(), zero()),
            )?;
            let value = if negative { zero() } else { p.clone() };
            self.theorem(
                &format!("{name}OfFromNat"),
                &[("positive", nat())],
                eq(nat(), pick(ic("FromNat", vec![p.clone()])), value.clone()),
                refl(nat(), value),
            )?;
            let value = if negative {
                pos(x.clone())
            } else {
                neg(x.clone())
            };
            self.theorem(
                &format!("{name}OfNegate"),
                &x1,
                eq(nat(), pick(ic("Negate", vec![x.clone()])), value.clone()),
                refl(nat(), value),
            )?;
            let value = add(pick(x.clone()), pick(y.clone()));
            self.theorem(
                &format!("{name}OfAdd"),
                &xy,
                eq(
                    nat(),
                    pick(ic("Add", vec![x.clone(), y.clone()])),
                    value.clone(),
                ),
                refl(nat(), value),
            )?;
            let value = if negative {
                add(neg(x.clone()), pos(y.clone()))
            } else {
                add(pos(x.clone()), neg(y.clone()))
            };
            self.theorem(
                &format!("{name}OfSub"),
                &xy,
                eq(
                    nat(),
                    pick(ic("Sub", vec![x.clone(), y.clone()])),
                    value.clone(),
                ),
                refl(nat(), value),
            )?;
            let value = if negative {
                add(
                    mul(pos(x.clone()), neg(y.clone())),
                    mul(neg(x.clone()), pos(y.clone())),
                )
            } else {
                add(
                    mul(pos(x.clone()), pos(y.clone())),
                    mul(neg(x.clone()), neg(y.clone())),
                )
            };
            self.theorem(
                &format!("{name}OfMul"),
                &xy,
                eq(
                    nat(),
                    pick(ic("Mul", vec![x.clone(), y.clone()])),
                    value.clone(),
                ),
                refl(nat(), value),
            )?;
        }
        Ok(())
    }
    fn project(&mut self, node: Expr) -> Result<Equality> {
        if let Expr::Apply(function, args) = &node {
            let part = if **function == constant("Mpk.Go.Practical.Int.Positive") {
                Some(false)
            } else if **function == constant("Mpk.Go.Practical.Int.Negative") {
                Some(true)
            } else {
                None
            };
            if let Some(negative) = part {
                let [value] = args.as_slice() else {
                    return Err(GoOrdinaryError::Linkage);
                };
                let (head, args) = match value {
                    Expr::Apply(f, args) => (f.as_ref(), args.as_slice()),
                    node => (node, &[][..]),
                };
                if let Expr::Constant(name) = head {
                    if let Some(op) = name.strip_prefix("Mpk.Go.Practical.Int.") {
                        let pick = |a| if negative { neg(a) } else { pos(a) };
                        let target = match (op, args) {
                            ("Make", [p, n]) => Some(if negative { n.clone() } else { p.clone() }),
                            ("Zero", []) => Some(zero()),
                            ("FromNat", [n]) => Some(if negative { zero() } else { n.clone() }),
                            ("Negate", [a]) => Some(if negative {
                                pos(a.clone())
                            } else {
                                neg(a.clone())
                            }),
                            ("Add", [a, b]) => Some(add(pick(a.clone()), pick(b.clone()))),
                            ("Sub", [a, b]) => Some(if negative {
                                add(neg(a.clone()), pos(b.clone()))
                            } else {
                                add(pos(a.clone()), neg(b.clone()))
                            }),
                            ("Mul", [a, b]) => Some(if negative {
                                add(
                                    mul(pos(a.clone()), neg(b.clone())),
                                    mul(neg(a.clone()), pos(b.clone())),
                                )
                            } else {
                                add(
                                    mul(pos(a.clone()), pos(b.clone())),
                                    mul(neg(a.clone()), neg(b.clone())),
                                )
                            }),
                            ("Make" | "Zero" | "FromNat" | "Negate" | "Add" | "Sub" | "Mul", _) => {
                                return Err(GoOrdinaryError::Linkage)
                            }
                            _ => None,
                        };
                        if let Some(target) = target {
                            let part = if negative { "Negative" } else { "Positive" };
                            let e = Equality::natural(
                                node.clone(),
                                target.clone(),
                                ic(&format!("{part}Of{op}"), args.to_vec()),
                            );
                            return e.then(self.project(target)?);
                        }
                    }
                }
            }
        }
        if let Some((a, b)) = split(&node, "Add") {
            let p = self.project(a)?;
            let q = self.project(b.clone())?;
            let pa = p.right.clone();
            return p.on_left(b).then(q.on_right(pa));
        }
        if let Some((a, b)) = split(&node, "Mul") {
            let p = self.project(a)?;
            let q = self.project(b.clone())?;
            let pa = p.right.clone();
            return mul_left(p, b).then(mul_right(pa, q));
        }
        Ok(Equality::reflexive(node))
    }
    fn relation_with<F>(
        &mut self,
        name: &str,
        args: &[Binder],
        left: Expr,
        right: Expr,
        middle: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut Self, Expr, Expr) -> Result<Equality>,
    {
        self.args = args.to_vec();
        let (l, r) = cross(left.clone(), right.clone());
        let lp = self.project(l)?;
        let lp = self.save(lp)?;
        let rp = self.project(r)?;
        let rp = self.save(rp)?;
        let mid = middle(self, lp.right.clone(), rp.right.clone())?;
        let e = lp.then(mid)?.then(rp.reverse())?;
        self.theorem(name, args, equivalent(left, right), e.proof)
    }
    fn relation(&mut self, name: &str, args: &[Binder], left: Expr, right: Expr) -> Result<()> {
        self.relation_with(name, args, left, right, |s, l, r| s.polynomial(l, r))
    }
    fn algebra(&mut self) -> Result<()> {
        let (x, y, z) = (var("x"), var("y"), var("z"));
        let x1 = vec![("x", integer())];
        let xy = vec![("x", integer()), ("y", integer())];
        let xyz = vec![("x", integer()), ("y", integer()), ("z", integer())];
        self.relation("Reflexive", &x1, x.clone(), x.clone())?;
        let (a, b) = cross(x.clone(), y.clone());
        self.theorem(
            "Symmetric",
            &[
                ("x", integer()),
                ("y", integer()),
                ("_same", equivalent(x.clone(), y.clone())),
            ],
            equivalent(y.clone(), x.clone()),
            call("Std.Eq.symm", vec![nat(), a, b, var("_same")]),
        )?;
        for (name, args, left, right) in [
            (
                "AddZero",
                x1.clone(),
                ic("Add", vec![x.clone(), ic("Zero", vec![])]),
                x.clone(),
            ),
            (
                "AddComm",
                xy.clone(),
                ic("Add", vec![x.clone(), y.clone()]),
                ic("Add", vec![y.clone(), x.clone()]),
            ),
            (
                "AddAssoc",
                xyz.clone(),
                ic(
                    "Add",
                    vec![ic("Add", vec![x.clone(), y.clone()]), z.clone()],
                ),
                ic(
                    "Add",
                    vec![x.clone(), ic("Add", vec![y.clone(), z.clone()])],
                ),
            ),
            (
                "NegateTwice",
                x1.clone(),
                ic("Negate", vec![ic("Negate", vec![x.clone()])]),
                x.clone(),
            ),
            (
                "AddInverse",
                x1.clone(),
                ic("Add", vec![x.clone(), ic("Negate", vec![x.clone()])]),
                ic("Zero", vec![]),
            ),
            (
                "SubtractSelf",
                x1.clone(),
                ic("Sub", vec![x.clone(), x.clone()]),
                ic("Zero", vec![]),
            ),
            (
                "MulZero",
                x1.clone(),
                ic("Mul", vec![x.clone(), ic("Zero", vec![])]),
                ic("Zero", vec![]),
            ),
            (
                "MulOne",
                x1.clone(),
                ic("Mul", vec![x.clone(), ic("FromNat", vec![one()])]),
                x.clone(),
            ),
            (
                "MulComm",
                xy.clone(),
                ic("Mul", vec![x.clone(), y.clone()]),
                ic("Mul", vec![y.clone(), x.clone()]),
            ),
            (
                "MulAddLeft",
                xyz.clone(),
                ic(
                    "Mul",
                    vec![ic("Add", vec![x.clone(), y.clone()]), z.clone()],
                ),
                ic(
                    "Add",
                    vec![
                        ic("Mul", vec![x.clone(), z.clone()]),
                        ic("Mul", vec![y.clone(), z.clone()]),
                    ],
                ),
            ),
            (
                "MulAddRight",
                xyz.clone(),
                ic(
                    "Mul",
                    vec![x.clone(), ic("Add", vec![y.clone(), z.clone()])],
                ),
                ic(
                    "Add",
                    vec![
                        ic("Mul", vec![x.clone(), y.clone()]),
                        ic("Mul", vec![x.clone(), z.clone()]),
                    ],
                ),
            ),
            (
                "MulAssoc",
                xyz,
                ic(
                    "Mul",
                    vec![ic("Mul", vec![x.clone(), y.clone()]), z.clone()],
                ),
                ic(
                    "Mul",
                    vec![x.clone(), ic("Mul", vec![y.clone(), z.clone()])],
                ),
            ),
        ] {
            self.relation(name, &args, left, right)?;
        }
        let (p, n, k) = (var("p"), var("n"), var("k"));
        self.relation(
            "RepresentationShift",
            &[("p", nat()), ("n", nat()), ("k", nat())],
            ic("Make", vec![p.clone(), n.clone()]),
            ic("Make", vec![add(p, k.clone()), add(n, k)]),
        )
    }
    fn equivalence(&mut self) -> Result<()> {
        let (x, y, z, u, v) = (var("x"), var("y"), var("z"), var("u"), var("v"));
        let (px, py, pz, nx, ny, nz) = (
            pos(x.clone()),
            pos(y.clone()),
            pos(z.clone()),
            neg(x.clone()),
            neg(y.clone()),
            neg(z.clone()),
        );
        let args = vec![
            ("x", integer()),
            ("y", integer()),
            ("z", integer()),
            ("_xy", equivalent(x.clone(), y.clone())),
            ("_yz", equivalent(y.clone(), z.clone())),
        ];
        self.args = args.clone();
        let (l, r) = cross(x.clone(), z.clone());
        let padding = add(py.clone(), ny.clone());
        let start = add(l.clone(), padding.clone());
        let finish = add(r.clone(), padding.clone());
        let first = add(add(px.clone(), ny.clone()), add(py.clone(), nz.clone()));
        let e = self.polynomial(start, first)?;
        let hxy = Equality::natural(
            add(px.clone(), ny.clone()),
            add(py.clone(), nx.clone()),
            var("_xy"),
        );
        let hyz = Equality::natural(
            add(py.clone(), nz.clone()),
            add(pz.clone(), ny.clone()),
            var("_yz"),
        );
        let e = e
            .then(hxy.on_left(add(py.clone(), nz.clone())))?
            .then(hyz.on_right(add(py.clone(), nx.clone())))?;
        let tail = self.polynomial(e.right.clone(), finish)?;
        let e = e.then(tail)?;
        self.theorem(
            "Transitive",
            &args,
            equivalent(x.clone(), z.clone()),
            nc("AddCancel", vec![l, r, padding, e.proof]),
        )?;
        let (pu, pv, nu, nv) = (
            pos(u.clone()),
            pos(v.clone()),
            neg(u.clone()),
            neg(v.clone()),
        );
        let args = vec![
            ("x", integer()),
            ("y", integer()),
            ("u", integer()),
            ("v", integer()),
            ("_xu", equivalent(x.clone(), u.clone())),
            ("_yv", equivalent(y.clone(), v.clone())),
        ];
        self.relation_with(
            "AddRespects",
            &args,
            ic("Add", vec![x.clone(), y.clone()]),
            ic("Add", vec![u.clone(), v.clone()]),
            |s, l, r| {
                let first = add(add(px.clone(), nu.clone()), add(py.clone(), nv.clone()));
                let e = s.polynomial(l, first)?;
                let e = e
                    .then(
                        Equality::natural(
                            add(px.clone(), nu.clone()),
                            add(pu.clone(), nx.clone()),
                            var("_xu"),
                        )
                        .on_left(add(py.clone(), nv.clone())),
                    )?
                    .then(
                        Equality::natural(
                            add(py.clone(), nv.clone()),
                            add(pv.clone(), ny.clone()),
                            var("_yv"),
                        )
                        .on_right(add(pu.clone(), nx.clone())),
                    )?;
                let tail = s.polynomial(e.right.clone(), r)?;
                e.then(tail)
            },
        )?;
        let xu = vec![
            ("x", integer()),
            ("u", integer()),
            ("_xu", equivalent(x.clone(), u.clone())),
        ];
        self.relation_with(
            "NegateRespects",
            &xu,
            ic("Negate", vec![x.clone()]),
            ic("Negate", vec![u.clone()]),
            |s, l, r| {
                let e = s.polynomial(l, add(pu.clone(), nx.clone()))?.then(
                    Equality::natural(
                        add(px.clone(), nu.clone()),
                        add(pu.clone(), nx.clone()),
                        var("_xu"),
                    )
                    .reverse(),
                )?;
                let tail = s.polynomial(e.right.clone(), r)?;
                e.then(tail)
            },
        )?;
        let xuz = vec![
            ("x", integer()),
            ("u", integer()),
            ("z", integer()),
            ("_xu", equivalent(x.clone(), u.clone())),
        ];
        for side in ["Left", "Right"] {
            let (left, right) = if side == "Left" {
                (
                    ic("Mul", vec![x.clone(), z.clone()]),
                    ic("Mul", vec![u.clone(), z.clone()]),
                )
            } else {
                (
                    ic("Mul", vec![z.clone(), x.clone()]),
                    ic("Mul", vec![z.clone(), u.clone()]),
                )
            };
            self.relation_with(
                &format!("MulRespects{side}"),
                &xuz,
                left,
                right,
                |s, l, r| {
                    let a = add(px.clone(), nu.clone());
                    let b = add(pu.clone(), nx.clone());
                    let e = s.polynomial(
                        l,
                        add(mul(a.clone(), pz.clone()), mul(b.clone(), nz.clone())),
                    )?;
                    let h = Equality::natural(a, b.clone(), var("_xu"));
                    let e = e
                        .then(mul_left(h.clone(), pz.clone()).on_left(mul(b.clone(), nz.clone())))?
                        .then(mul_left(h.reverse(), nz.clone()).on_right(mul(b, pz.clone())))?;
                    let tail = s.polynomial(e.right.clone(), r)?;
                    e.then(tail)
                },
            )?;
        }
        let p = ic(
            "MulRespectsLeft",
            vec![x.clone(), u.clone(), y.clone(), var("_xu")],
        );
        let q = ic(
            "MulRespectsRight",
            vec![y.clone(), v.clone(), u.clone(), var("_yv")],
        );
        self.theorem(
            "MulRespects",
            &args,
            equivalent(
                ic("Mul", vec![x.clone(), y.clone()]),
                ic("Mul", vec![u.clone(), v.clone()]),
            ),
            ic(
                "Transitive",
                vec![
                    ic("Mul", vec![x.clone(), y.clone()]),
                    ic("Mul", vec![u.clone(), y.clone()]),
                    ic("Mul", vec![u.clone(), v.clone()]),
                    p,
                    q,
                ],
            ),
        )?;
        let negative = ic("NegateRespects", vec![y.clone(), v.clone(), var("_yv")]);
        self.theorem(
            "SubRespects",
            &args,
            equivalent(
                ic("Sub", vec![x.clone(), y.clone()]),
                ic("Sub", vec![u.clone(), v.clone()]),
            ),
            ic(
                "AddRespects",
                vec![
                    x,
                    ic("Negate", vec![y]),
                    u,
                    ic("Negate", vec![v]),
                    var("_xu"),
                    negative,
                ],
            ),
        )
    }
    fn observations(&mut self) -> Result<()> {
        let (x, y, u, v) = (var("x"), var("y"), var("u"), var("v"));
        let (l, r) = cross(x.clone(), y.clone());
        let condition = eq(
            boolean(),
            ic("Equal", vec![x.clone(), y.clone()]),
            bit(true),
        );
        self.theorem(
            "EqualFromEquivalent",
            &[
                ("x", integer()),
                ("y", integer()),
                ("_same", equivalent(x.clone(), y.clone())),
            ],
            condition.clone(),
            nc("EqualComplete", vec![l.clone(), r.clone(), var("_same")]),
        )?;
        self.theorem(
            "EqualReflect",
            &[("x", integer()), ("y", integer()), ("_true", condition)],
            equivalent(x.clone(), y.clone()),
            nc("EqualReflect", vec![l, r, var("_true")]),
        )?;
        let (a, b) = cross(x.clone(), y.clone());
        let (aa, bb) = cross(u.clone(), v.clone());
        let padding = add(neg(u.clone()), neg(v.clone()));
        let other = add(neg(x.clone()), neg(y.clone()));
        let args = vec![
            ("x", integer()),
            ("y", integer()),
            ("u", integer()),
            ("v", integer()),
            ("_xu", equivalent(x.clone(), u.clone())),
            ("_yv", equivalent(y.clone(), v.clone())),
        ];
        self.args = args.clone();
        let pa = self.polynomial(
            add(a.clone(), padding.clone()),
            add(
                add(pos(x.clone()), neg(u.clone())),
                add(neg(y.clone()), neg(v.clone())),
            ),
        )?;
        let h = Equality::natural(
            add(pos(x.clone()), neg(u.clone())),
            add(pos(u.clone()), neg(x.clone())),
            var("_xu"),
        );
        let pa = pa.then(h.on_left(add(neg(y.clone()), neg(v.clone()))))?;
        let tail = self.polynomial(pa.right.clone(), add(aa.clone(), other.clone()))?;
        let pa = pa.then(tail)?;
        let pb = self.polynomial(
            add(b.clone(), padding.clone()),
            add(
                add(pos(y.clone()), neg(v.clone())),
                add(neg(x.clone()), neg(u.clone())),
            ),
        )?;
        let h = Equality::natural(
            add(pos(y.clone()), neg(v.clone())),
            add(pos(v.clone()), neg(y.clone())),
            var("_yv"),
        );
        let pb = pb.then(h.on_left(add(neg(x.clone()), neg(u.clone()))))?;
        let tail = self.polynomial(pb.right.clone(), add(bb.clone(), other.clone()))?;
        let pb = pb.then(tail)?;
        for name in ["Equal", "Less"] {
            let left = nc(name, vec![a.clone(), b.clone()]);
            let right = nc(name, vec![aa.clone(), bb.clone()]);
            let translated = nc(
                name,
                vec![
                    add(a.clone(), padding.clone()),
                    add(b.clone(), padding.clone()),
                ],
            );
            let e = Equality::boolean(
                translated.clone(),
                left,
                nc(
                    &format!("{name}Translate"),
                    vec![a.clone(), b.clone(), padding.clone()],
                ),
            )
            .reverse()
            .then(pa.clone().mapped(
                lam(
                    "_value",
                    nat(),
                    nc(name, vec![var("_value"), add(b.clone(), padding.clone())]),
                ),
                translated,
                nc(
                    name,
                    vec![
                        add(aa.clone(), other.clone()),
                        add(b.clone(), padding.clone()),
                    ],
                ),
                boolean(),
            ))?
            .then(pb.clone().mapped(
                lam(
                    "_value",
                    nat(),
                    nc(name, vec![add(aa.clone(), other.clone()), var("_value")]),
                ),
                nc(
                    name,
                    vec![
                        add(aa.clone(), other.clone()),
                        add(b.clone(), padding.clone()),
                    ],
                ),
                nc(
                    name,
                    vec![
                        add(aa.clone(), other.clone()),
                        add(bb.clone(), other.clone()),
                    ],
                ),
                boolean(),
            ))?
            .then(Equality::boolean(
                nc(
                    name,
                    vec![
                        add(aa.clone(), other.clone()),
                        add(bb.clone(), other.clone()),
                    ],
                ),
                right,
                nc(
                    &format!("{name}Translate"),
                    vec![aa.clone(), bb.clone(), other.clone()],
                ),
            ))?;
            self.theorem(
                &format!("{name}Respects"),
                &args,
                eq(
                    boolean(),
                    ic(name, vec![x.clone(), y.clone()]),
                    ic(name, vec![u.clone(), v.clone()]),
                ),
                e.proof,
            )?;
        }
        let e = Equality::boolean(
            ic("Less", vec![y.clone(), x.clone()]),
            ic("Less", vec![v.clone(), u.clone()]),
            ic(
                "LessRespects",
                vec![
                    y.clone(),
                    x.clone(),
                    v.clone(),
                    u.clone(),
                    var("_yv"),
                    var("_xu"),
                ],
            ),
        )
        .mapped(
            constant("Mpk.Go.Practical.Bool.Not"),
            ic("LessEqual", vec![x.clone(), y.clone()]),
            ic("LessEqual", vec![u.clone(), v.clone()]),
            boolean(),
        );
        self.theorem(
            "LessEqualRespects",
            &args,
            eq(e.ty, e.left, e.right),
            e.proof,
        )?;
        let (a, b) = (var("a"), var("b"));
        let args = [
            ("a", nat()),
            ("b", nat()),
            (
                "_same",
                equivalent(
                    ic("FromNat", vec![a.clone()]),
                    ic("FromNat", vec![b.clone()]),
                ),
            ),
        ];
        self.args = args.to_vec();
        let (l, r) = cross(
            ic("FromNat", vec![a.clone()]),
            ic("FromNat", vec![b.clone()]),
        );
        // Expose both zero projections through checked computation rules.
        // The existing core does not normalize a computed recursor major.
        let lp = self.project(l.clone())?;
        let tail = self.polynomial(lp.right.clone(), a.clone())?;
        let lp = self.save(lp.then(tail)?)?;
        let rp = self.project(r.clone())?;
        let tail = self.polynomial(rp.right.clone(), b.clone())?;
        let rp = self.save(rp.then(tail)?)?;
        let proof = lp
            .reverse()
            .then(Equality::natural(l, r, var("_same")))?
            .then(rp)?;
        self.theorem("FromNatInjective", &args, eq(nat(), a, b), proof.proof)
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut integers = Integers {
        builder,
        serial: 0,
        args: vec![],
    };
    integers.definitions()?;
    integers.algebra()?;
    integers.equivalence()?;
    integers.observations()
}
