//! Frozen fixed arrays: the empty Bool dummy and least power-of-two data cubes.
//! Full-word reads, canonical padding and value updates grant no source authority.
use super::syntax::*;
use super::{Builder, Result};
mod normalization;
mod update;
const BITS: [&str; 8] = ["_a0", "_a1", "_a2", "_a3", "_a4", "_a5", "_a6", "_a7"];
fn ac(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Array.D{d}.{name}"), args)
}
fn fc(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn ix(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Index.D{d}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn seq(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Sequence.{name}"), args)
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
fn indices(w: Expr, d: usize) -> Result<Vec<Expr>> {
    (0..d)
        .rev()
        .map(|i| super::word::at(w.clone(), 64, i))
        .collect()
}
fn word() -> Expr {
    (0..6).fold(boolean(), |b, _| pi("_bit", boolean(), b))
}
fn value(w: Expr) -> Expr {
    call("Mpk.Go.Practical.Word.Numeric.Value6", vec![w])
}
fn idx(d: usize, at: Vec<Expr>) -> Expr {
    fc(d, "Index", at)
}
fn cap(d: usize) -> Expr {
    fc(d, "Capacity", vec![])
}
fn truth(flag: Expr, yes: bool) -> Expr {
    eq(boolean(), flag, bit(yes))
}
fn inside(i: Expr, n: Expr) -> Expr {
    truth(less(i, n), true)
}
fn pred(a: Expr) -> Expr {
    pi("_element", a, Expr::Sort0)
}
fn product(a: Expr, b: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call("Std.Bool.recType", vec![a, b, var("_tag")]),
    )
}
fn choose(a: Expr, flag: Expr, yes: Expr, no: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_tag", boolean(), a), no, yes, flag],
    )
}
fn slot(f: Expr, at: Vec<Expr>) -> Expr {
    app(f, at)
}
fn read(d: usize, a: Expr, f: Expr, i: Expr) -> Expr {
    ac(d, "Read", vec![a, f, i])
}
fn padding(d: usize, a: Expr, z: Expr, n: Expr, f: Expr) -> Expr {
    cube(
        d,
        pi(
            "_out",
            truth(less(idx(d, address(d)), n), false),
            eq(a, slot(f, address(d)), z),
        ),
        false,
    )
}
fn elements(d: usize, p: Expr, f: Expr) -> Expr {
    cube(d, app(p, vec![slot(f, address(d))]), false)
}
fn valid(d: usize, a: Expr, p: Expr, z: Expr, n: Expr, f: Expr) -> Expr {
    ac(d, "Valid", vec![a, p, z, n, f])
}
fn shape(d: usize, n: Expr) -> Expr {
    ac(d, "Shape", vec![n])
}
fn shape_upper(d: usize, n: Expr, h: Expr) -> Expr {
    ac(d, "ShapeUpper", vec![n, h])
}
fn bounded(d: usize, n: Expr, i: Expr, h: Expr, within: Expr) -> Expr {
    call(
        "Mpk.Go.Practical.Word.Index.UpperBound",
        vec![cap(d), n.clone(), value(i), shape_upper(d, n, h), within],
    )
}
fn nl(e: Equality, n: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), n.clone()),
        less(e.right.clone(), n.clone()),
    );
    e.mapped(lam("_n", nat(), less(var("_n"), n)), l, r, boolean())
}
fn retype(a: Expr, p: Expr, e: Equality, h: Expr) -> Expr {
    call("Std.Eq.rewrite", vec![a, e.left, e.right, p, e.proof, h])
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
                lam("_case", truth(flag.clone(), false), no),
                lam("_case", truth(flag.clone(), true), yes),
                flag.clone(),
            ],
        ),
        vec![refl(boolean(), flag)],
    )
}
struct Arrays<'a>(&'a mut Builder);
impl Arrays<'_> {
    fn emit(
        &mut self,
        d: usize,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        law: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.Array.D{d}.{name}"),
            args,
            ty,
            body,
            law,
        )
    }
    fn theorem(&mut self, d: usize, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(d, name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self, d: usize) -> Result<()> {
        let (a, p, z, n, f) = (var("A"), var("P"), var("z"), var("n"), var("f"));
        self.emit(
            d,
            "Carrier",
            &[("A", Expr::Sort0)],
            Expr::Sort0,
            cube(d, a.clone(), false),
            false,
        )?;
        let lower = if d == 0 { one() } else { cap(d - 1) };
        let lower_bound = if d == 0 {
            inside(zero(), n.clone())
        } else {
            inside(lower, n.clone())
        };
        let upper = truth(less(cap(d), n.clone()), false);
        self.emit(
            d,
            "Shape",
            &[("n", nat())],
            Expr::Sort0,
            product(lower_bound.clone(), upper.clone()),
            false,
        )?;
        let shape_args = [("n", nat()), ("h", shape(d, n.clone()))];
        for (name, right, ty) in [
            ("ShapeLower", false, lower_bound.clone()),
            ("ShapeUpper", true, upper.clone()),
        ] {
            self.emit(
                d,
                name,
                &shape_args,
                ty,
                pc(
                    if right { "Right" } else { "Left" },
                    vec![lower_bound.clone(), upper.clone(), var("h")],
                ),
                true,
            )?;
        }
        let base = [
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("z", a.clone()),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
        ];
        self.emit(
            d,
            "Elements",
            &[base[0].clone(), base[1].clone(), base[4].clone()],
            Expr::Sort0,
            elements(d, p.clone(), f.clone()),
            false,
        )?;
        self.emit(
            d,
            "Padding",
            &[
                base[0].clone(),
                base[2].clone(),
                base[3].clone(),
                base[4].clone(),
            ],
            Expr::Sort0,
            padding(d, a.clone(), z.clone(), n.clone(), f.clone()),
            false,
        )?;
        self.emit(
            d,
            "Valid",
            &base,
            Expr::Sort0,
            product(
                elements(d, p.clone(), f.clone()),
                padding(d, a.clone(), z.clone(), n.clone(), f.clone()),
            ),
            false,
        )?;
        let mut args = base.to_vec();
        args.push((
            "h",
            valid(d, a.clone(), p.clone(), z.clone(), n.clone(), f.clone()),
        ));
        for (name, right, ty) in [
            ("ValidElements", false, elements(d, p.clone(), f.clone())),
            (
                "ValidPadding",
                true,
                padding(d, a.clone(), z.clone(), n.clone(), f.clone()),
            ),
        ] {
            self.emit(
                d,
                name,
                &args,
                ty,
                pc(
                    if right { "Right" } else { "Left" },
                    vec![
                        elements(d, p.clone(), f.clone()),
                        padding(d, a.clone(), z.clone(), n.clone(), f.clone()),
                        var("h"),
                    ],
                ),
                true,
            )?;
        }
        self.emit(
            d,
            "Zero",
            &[("A", Expr::Sort0), ("z", a.clone())],
            cube(d, a.clone(), false),
            cube(d, z.clone(), true),
            false,
        )?;
        self.emit(
            d,
            "Copy",
            &[("A", Expr::Sort0), ("f", cube(d, a.clone(), false))],
            cube(d, a.clone(), false),
            f.clone(),
            false,
        )?;
        let mut copied = args.clone();
        copied.retain(|(name, _)| *name != "h");
        copied.push((
            "h",
            valid(d, a.clone(), p.clone(), z.clone(), n.clone(), f.clone()),
        ));
        self.emit(
            d,
            "CopyValid",
            &copied,
            valid(
                d,
                a.clone(),
                p.clone(),
                z.clone(),
                n.clone(),
                ac(d, "Copy", vec![a.clone(), f.clone()]),
            ),
            var("h"),
            true,
        )?;
        self.emit(
            d,
            "Read",
            &[
                ("A", Expr::Sort0),
                ("f", cube(d, a.clone(), false)),
                ("i", word()),
            ],
            a.clone(),
            ix(d, "Read64", vec![a.clone(), f.clone(), var("i")]),
            false,
        )?;
        let zeroed = ac(d, "Zero", vec![a.clone(), z.clone()]);
        self.emit(
            d,
            "ZeroValid",
            &[
                ("A", Expr::Sort0),
                ("P", pred(a.clone())),
                ("z", a.clone()),
                ("n", nat()),
                ("hz", app(p.clone(), vec![z.clone()])),
            ],
            valid(d, a.clone(), p.clone(), z.clone(), n.clone(), zeroed),
            pc(
                "Make",
                vec![
                    elements(d, p.clone(), ac(d, "Zero", vec![a.clone(), z.clone()])),
                    padding(
                        d,
                        a.clone(),
                        z.clone(),
                        n,
                        ac(d, "Zero", vec![a.clone(), z.clone()]),
                    ),
                    cube(d, var("hz"), true),
                    cube(
                        d,
                        lam(
                            "_out",
                            truth(less(idx(d, address(d)), var("n")), false),
                            refl(a, z),
                        ),
                        true,
                    ),
                ],
            ),
            true,
        )
    }
    fn reads(&mut self, d: usize) -> Result<()> {
        let (a, p, z, n, f, i) = (var("A"), var("P"), var("z"), var("n"), var("f"), var("i"));
        let args = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("f", cube(d, a.clone(), false)),
            ("i", word()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(i.clone()), n.clone())),
        ];
        let cap_h = bounded(d, n.clone(), i.clone(), var("shape"), var("within"));
        self.theorem(
            d,
            "ReadWithinSlot",
            &args,
            Equality {
                ty: a.clone(),
                left: read(d, a.clone(), f.clone(), i.clone()),
                right: slot(f.clone(), indices(i.clone(), d)?),
                proof: ix(
                    d,
                    "BoundedRead",
                    vec![a.clone(), f.clone(), i.clone(), cap_h.clone()],
                ),
            },
        )?;
        let mut args = args.to_vec();
        args.insert(1, ("P", pred(a.clone())));
        args.insert(2, ("z", a.clone()));
        args.push((
            "h",
            valid(d, a.clone(), p.clone(), z.clone(), n.clone(), f.clone()),
        ));
        self.emit(
            d,
            "ReadValid",
            &args,
            app(p.clone(), vec![read(d, a.clone(), f.clone(), i.clone())]),
            fc(
                d,
                "LookupValid",
                vec![
                    a.clone(),
                    p.clone(),
                    f.clone(),
                    ac(
                        d,
                        "ValidElements",
                        vec![a.clone(), p, z, n.clone(), f, var("h")],
                    ),
                    value(i),
                    cap_h,
                ],
            ),
            true,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("z", a.clone()),
            ("n", nat()),
            ("i", word()),
            ("shape", shape(d, n.clone())),
            ("within", inside(value(var("i")), n.clone())),
        ];
        self.theorem(
            d,
            "ReadZero",
            &args,
            Equality {
                ty: a.clone(),
                left: read(
                    d,
                    a.clone(),
                    ac(d, "Zero", vec![a.clone(), var("z")]),
                    var("i"),
                ),
                right: var("z"),
                proof: ac(
                    d,
                    "ReadWithinSlot",
                    vec![
                        a.clone(),
                        n,
                        ac(d, "Zero", vec![a, var("z")]),
                        var("i"),
                        var("shape"),
                        var("within"),
                    ],
                ),
            },
        )
    }
}
fn empty(builder: &mut Builder) -> Result<()> {
    let emit_empty =
        |builder: &mut Builder, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool| {
            emit(
                builder,
                &format!("Mpk.Go.Practical.Array.Empty.{name}"),
                args,
                ty,
                body,
                law,
            )
        };
    emit_empty(builder, "Carrier", &[], Expr::Sort0, boolean(), false)?;
    emit_empty(builder, "Zero", &[], boolean(), bit(false), false)?;
    emit_empty(
        builder,
        "Valid",
        &[("dummy", boolean())],
        Expr::Sort0,
        truth(var("dummy"), false),
        false,
    )?;
    emit_empty(
        builder,
        "ZeroValid",
        &[],
        truth(call("Mpk.Go.Practical.Array.Empty.Zero", vec![]), false),
        refl(boolean(), bit(false)),
        true,
    )?;
    emit_empty(
        builder,
        "NoIndex",
        &[("i", word())],
        truth(less(value(var("i")), zero()), false),
        nc("LessZero", vec![value(var("i"))]),
        true,
    )?;
    emit_empty(
        builder,
        "Canonical",
        &[
            ("dummy", boolean()),
            (
                "h",
                call("Mpk.Go.Practical.Array.Empty.Valid", vec![var("dummy")]),
            ),
        ],
        truth(var("dummy"), false),
        var("h"),
        true,
    )
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    empty(builder)?;
    let mut arrays = Arrays(builder);
    for d in 0..=8 {
        arrays.definitions(d)?;
        arrays.reads(d)?;
        arrays.normalization(d)?;
        arrays.updates(d)?;
    }
    Ok(())
}
