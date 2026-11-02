//! Heterogeneous Sort0 products using the frozen dependent Bool eliminators.
//! All equality of arbitrary products is pointwise; no function extensionality.
use super::syntax::*;
use super::{Builder, Result};

const PREFIX: &str = "Mpk.Go.Practical.Product";
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn selected(a: Expr, b: Expr, tag: Expr) -> Expr {
    call("Std.Bool.recType", vec![a, b, tag])
}
fn product(a: Expr, b: Expr) -> Expr {
    pc("Type", vec![a, b])
}
fn make(a: Expr, b: Expr, x: Expr, y: Expr) -> Expr {
    pc("Make", vec![a, b, x, y])
}
fn projection(a: Expr, b: Expr, p: Expr, right: bool) -> Expr {
    pc(if right { "Right" } else { "Left" }, vec![a, b, p])
}
fn pointwise(a: Expr, b: Expr, p: Expr, q: Expr) -> Expr {
    pc("Pointwise", vec![a, b, p, q])
}
fn predicate(a: Expr) -> Expr {
    pi("_value", a, Expr::Sort0)
}
fn function(a: Expr, b: Expr) -> Expr {
    pi("_value", a, b)
}
fn valid(a: Expr, b: Expr, p: Expr, q: Expr, x: Expr) -> Expr {
    pc("Valid", vec![a, b, p, q, x])
}
fn update(a: Expr, b: Expr, p: Expr, x: Expr, right: bool) -> Expr {
    pc(if right { "SetRight" } else { "SetLeft" }, vec![a, b, p, x])
}
fn map(a: Expr, b: Expr, c: Expr, d: Expr, f: Expr, g: Expr, p: Expr) -> Expr {
    pc("Map", vec![a, b, c, d, f, g, p])
}
fn observed(a: Expr, b: Expr, p: Expr, q: Expr, tag: Expr) -> Expr {
    eq(
        selected(a, b, tag.clone()),
        app(p, vec![tag.clone()]),
        app(q, vec![tag]),
    )
}
fn by_components(a: Expr, b: Expr, p: Expr, q: Expr, l: Expr, r: Expr) -> Expr {
    pc("PointwiseOfProjections", vec![a, b, p, q, l, r])
}

struct Products<'a>(&'a mut Builder);
impl Products<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        value: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, value, proof)
    }
    fn definitions(&mut self) -> Result<()> {
        let (a, b, p) = (var("A"), var("B"), var("p"));
        let types = [("A", Expr::Sort0), ("B", Expr::Sort0)];
        self.emit(
            "Type",
            &types,
            Expr::Sort0,
            pi("tag", boolean(), selected(a.clone(), b.clone(), var("tag"))),
            false,
        )?;
        let values = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("x", a.clone()),
            ("y", b.clone()),
        ];
        self.emit(
            "Make",
            &values,
            product(a.clone(), b.clone()),
            lam(
                "tag",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(
                            "_tag",
                            boolean(),
                            selected(a.clone(), b.clone(), var("_tag")),
                        ),
                        var("x"),
                        var("y"),
                        var("tag"),
                    ],
                ),
            ),
            false,
        )?;
        let input = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("p", product(a.clone(), b.clone())),
        ];
        for right in [false, true] {
            self.emit(
                if right { "Right" } else { "Left" },
                &input,
                if right { b.clone() } else { a.clone() },
                app(p.clone(), vec![bit(right)]),
                false,
            )?;
            let made = make(a.clone(), b.clone(), var("x"), var("y"));
            let (ty, value) = if right {
                (b.clone(), var("y"))
            } else {
                (a.clone(), var("x"))
            };
            self.emit(
                if right { "RightOfMake" } else { "LeftOfMake" },
                &values,
                eq(
                    ty.clone(),
                    projection(a.clone(), b.clone(), made, right),
                    value.clone(),
                ),
                refl(ty, value),
                true,
            )?;
            let mut args = input.to_vec();
            args.push(("value", if right { b.clone() } else { a.clone() }));
            self.emit(
                if right { "SetRight" } else { "SetLeft" },
                &args,
                product(a.clone(), b.clone()),
                make(
                    a.clone(),
                    b.clone(),
                    if right {
                        projection(a.clone(), b.clone(), p.clone(), false)
                    } else {
                        var("value")
                    },
                    if right {
                        var("value")
                    } else {
                        projection(a.clone(), b.clone(), p.clone(), true)
                    },
                ),
                false,
            )?;
        }
        let mut relation = input.to_vec();
        relation.push(("q", product(a.clone(), b.clone())));
        self.emit(
            "Pointwise",
            &relation,
            Expr::Sort0,
            pi(
                "tag",
                boolean(),
                observed(a.clone(), b.clone(), p, var("q"), var("tag")),
            ),
            false,
        )
    }
    fn observations(&mut self) -> Result<()> {
        let (a, b, p, q, r) = (var("A"), var("B"), var("p"), var("q"), var("r"));
        let input = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("p", product(a.clone(), b.clone())),
        ];
        self.emit(
            "PointwiseReflexive",
            &input,
            pointwise(a.clone(), b.clone(), p.clone(), p.clone()),
            lam(
                "tag",
                boolean(),
                refl(
                    selected(a.clone(), b.clone(), var("tag")),
                    app(p.clone(), vec![var("tag")]),
                ),
            ),
            true,
        )?;
        let mut args = input.to_vec();
        args.push(("q", product(a.clone(), b.clone())));
        let pair_args = args.clone();
        args.push((
            "left",
            eq(
                a.clone(),
                projection(a.clone(), b.clone(), p.clone(), false),
                projection(a.clone(), b.clone(), q.clone(), false),
            ),
        ));
        args.push((
            "right",
            eq(
                b.clone(),
                projection(a.clone(), b.clone(), p.clone(), true),
                projection(a.clone(), b.clone(), q.clone(), true),
            ),
        ));
        self.emit(
            "PointwiseOfProjections",
            &args,
            pointwise(a.clone(), b.clone(), p.clone(), q.clone()),
            lam(
                "tag",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(
                            "_tag",
                            boolean(),
                            observed(a.clone(), b.clone(), p.clone(), q.clone(), var("_tag")),
                        ),
                        var("left"),
                        var("right"),
                        var("tag"),
                    ],
                ),
            ),
            true,
        )?;
        let mut related = pair_args;
        related.push((
            "same",
            pointwise(a.clone(), b.clone(), p.clone(), q.clone()),
        ));
        for right in [false, true] {
            let ty = if right { b.clone() } else { a.clone() };
            self.emit(
                if right {
                    "PointwiseRight"
                } else {
                    "PointwiseLeft"
                },
                &related,
                eq(
                    ty,
                    projection(a.clone(), b.clone(), p.clone(), right),
                    projection(a.clone(), b.clone(), q.clone(), right),
                ),
                app(var("same"), vec![bit(right)]),
                true,
            )?;
        }
        let tag = var("tag");
        self.emit(
            "PointwiseSymmetric",
            &related,
            pointwise(a.clone(), b.clone(), q.clone(), p.clone()),
            lam(
                "tag",
                boolean(),
                call(
                    "Std.Eq.symm",
                    vec![
                        selected(a.clone(), b.clone(), tag.clone()),
                        app(p.clone(), vec![tag.clone()]),
                        app(q.clone(), vec![tag.clone()]),
                        app(var("same"), vec![tag.clone()]),
                    ],
                ),
            ),
            true,
        )?;
        let mut trans = input.to_vec();
        trans.push(("q", product(a.clone(), b.clone())));
        trans.push(("r", product(a.clone(), b.clone())));
        trans.push(("pq", pointwise(a.clone(), b.clone(), p.clone(), q.clone())));
        trans.push(("qr", pointwise(a.clone(), b.clone(), q.clone(), r.clone())));
        self.emit(
            "PointwiseTransitive",
            &trans,
            pointwise(a.clone(), b.clone(), p.clone(), r.clone()),
            lam(
                "tag",
                boolean(),
                call(
                    "Std.Eq.trans",
                    vec![
                        selected(a.clone(), b.clone(), tag.clone()),
                        app(p.clone(), vec![tag.clone()]),
                        app(q, vec![tag.clone()]),
                        app(r, vec![tag.clone()]),
                        app(var("pq"), vec![tag.clone()]),
                        app(var("qr"), vec![tag]),
                    ],
                ),
            ),
            true,
        )?;
        let left = projection(a.clone(), b.clone(), p.clone(), false);
        let right = projection(a.clone(), b.clone(), p.clone(), true);
        let rebuilt = make(a.clone(), b.clone(), left.clone(), right.clone());
        self.emit(
            "Reconstruct",
            &input,
            pointwise(a.clone(), b.clone(), rebuilt.clone(), p.clone()),
            by_components(
                a.clone(),
                b.clone(),
                rebuilt,
                p,
                refl(a, left),
                refl(b, right),
            ),
            true,
        )
    }
    fn updates(&mut self) -> Result<()> {
        let (a, b, p) = (var("A"), var("B"), var("p"));
        let input = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("p", product(a.clone(), b.clone())),
        ];
        for right in [false, true] {
            let side = if right { b.clone() } else { a.clone() };
            let mut args = input.to_vec();
            args.push(("value", side.clone()));
            let changed = update(a.clone(), b.clone(), p.clone(), var("value"), right);
            for observed_right in [false, true] {
                let ty = if observed_right { b.clone() } else { a.clone() };
                let expected = if observed_right == right {
                    var("value")
                } else {
                    projection(a.clone(), b.clone(), p.clone(), observed_right)
                };
                self.emit(
                    &format!(
                        "{}OfSet{}",
                        if observed_right { "Right" } else { "Left" },
                        if right { "Right" } else { "Left" }
                    ),
                    &args,
                    eq(
                        ty.clone(),
                        projection(a.clone(), b.clone(), changed.clone(), observed_right),
                        expected.clone(),
                    ),
                    refl(ty, expected),
                    true,
                )?;
            }
            args.push(("next", side));
            let twice = update(a.clone(), b.clone(), changed, var("next"), right);
            let once = update(a.clone(), b.clone(), p.clone(), var("next"), right);
            let left = if right {
                projection(a.clone(), b.clone(), p.clone(), false)
            } else {
                var("next")
            };
            let end = if right {
                var("next")
            } else {
                projection(a.clone(), b.clone(), p.clone(), true)
            };
            self.emit(
                if right {
                    "OverwriteRight"
                } else {
                    "OverwriteLeft"
                },
                &args,
                pointwise(a.clone(), b.clone(), twice.clone(), once.clone()),
                by_components(
                    a.clone(),
                    b.clone(),
                    twice,
                    once,
                    refl(a.clone(), left),
                    refl(b.clone(), end),
                ),
                true,
            )?;
        }
        let mut args = input.to_vec();
        args.push(("x", a.clone()));
        args.push(("y", b.clone()));
        let first = update(
            a.clone(),
            b.clone(),
            update(a.clone(), b.clone(), p.clone(), var("y"), true),
            var("x"),
            false,
        );
        let second = update(
            a.clone(),
            b.clone(),
            update(a.clone(), b.clone(), p, var("x"), false),
            var("y"),
            true,
        );
        self.emit(
            "CommuteUpdates",
            &args,
            pointwise(a.clone(), b.clone(), first.clone(), second.clone()),
            by_components(
                a.clone(),
                b.clone(),
                first,
                second,
                refl(a, var("x")),
                refl(b, var("y")),
            ),
            true,
        )
    }
    fn mapping(&mut self) -> Result<()> {
        let (a, b, c, d, p, f, g) = (
            var("A"),
            var("B"),
            var("C"),
            var("D"),
            var("p"),
            var("f"),
            var("g"),
        );
        let types = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
        ];
        let mut args = types.to_vec();
        args.push(("f", function(a.clone(), c.clone())));
        args.push(("g", function(b.clone(), d.clone())));
        args.push(("p", product(a.clone(), b.clone())));
        let left = app(
            f.clone(),
            vec![projection(a.clone(), b.clone(), p.clone(), false)],
        );
        let right = app(
            g.clone(),
            vec![projection(a.clone(), b.clone(), p.clone(), true)],
        );
        let mapped = map(
            a.clone(),
            b.clone(),
            c.clone(),
            d.clone(),
            f.clone(),
            g.clone(),
            p.clone(),
        );
        self.emit(
            "Map",
            &args,
            product(c.clone(), d.clone()),
            make(c.clone(), d.clone(), left.clone(), right.clone()),
            false,
        )?;
        for observed_right in [false, true] {
            let ty = if observed_right { d.clone() } else { c.clone() };
            let value = if observed_right {
                right.clone()
            } else {
                left.clone()
            };
            self.emit(
                if observed_right {
                    "RightOfMap"
                } else {
                    "LeftOfMap"
                },
                &args,
                eq(
                    ty.clone(),
                    projection(c.clone(), d.clone(), mapped.clone(), observed_right),
                    value.clone(),
                ),
                refl(ty, value),
                true,
            )?;
        }
        let input = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("p", product(a.clone(), b.clone())),
        ];
        let identity = map(
            a.clone(),
            b.clone(),
            a.clone(),
            b.clone(),
            lam("_x", a.clone(), var("_x")),
            lam("_y", b.clone(), var("_y")),
            p.clone(),
        );
        self.emit(
            "MapIdentity",
            &input,
            pointwise(a.clone(), b.clone(), identity.clone(), p.clone()),
            by_components(
                a.clone(),
                b.clone(),
                identity,
                p.clone(),
                refl(
                    a.clone(),
                    projection(a.clone(), b.clone(), p.clone(), false),
                ),
                refl(b.clone(), projection(a.clone(), b.clone(), p.clone(), true)),
            ),
            true,
        )?;
        let mut related = args.clone();
        related.push(("q", product(a.clone(), b.clone())));
        related.push(("same", pointwise(a.clone(), b.clone(), p.clone(), var("q"))));
        let mapped_q = map(
            a.clone(),
            b.clone(),
            c.clone(),
            d.clone(),
            f.clone(),
            g.clone(),
            var("q"),
        );
        let congruence = |r: bool| {
            call(
                "Std.Eq.congr",
                vec![
                    if r { b.clone() } else { a.clone() },
                    if r { d.clone() } else { c.clone() },
                    if r { g.clone() } else { f.clone() },
                    projection(a.clone(), b.clone(), p.clone(), r),
                    projection(a.clone(), b.clone(), var("q"), r),
                    pc(
                        if r { "PointwiseRight" } else { "PointwiseLeft" },
                        vec![a.clone(), b.clone(), p.clone(), var("q"), var("same")],
                    ),
                ],
            )
        };
        self.emit(
            "MapPointwise",
            &related,
            pointwise(c.clone(), d.clone(), mapped.clone(), mapped_q.clone()),
            by_components(
                c.clone(),
                d.clone(),
                mapped,
                mapped_q,
                congruence(false),
                congruence(true),
            ),
            true,
        )?;
        let (e, h, i, j) = (var("E"), var("F"), var("h"), var("i"));
        let mut compose = types.to_vec();
        compose.push(("E", Expr::Sort0));
        compose.push(("F", Expr::Sort0));
        compose.push(("f", function(a.clone(), c.clone())));
        compose.push(("g", function(b.clone(), d.clone())));
        compose.push(("h", function(c.clone(), e.clone())));
        compose.push(("i", function(d.clone(), h.clone())));
        compose.push(("p", product(a.clone(), b.clone())));
        let first = map(
            c.clone(),
            d.clone(),
            e.clone(),
            h.clone(),
            i.clone(),
            j.clone(),
            map(a.clone(), b.clone(), c, d, f.clone(), g.clone(), p.clone()),
        );
        let second = map(
            a.clone(),
            b.clone(),
            e.clone(),
            h.clone(),
            lam("_x", a, app(i.clone(), vec![app(f, vec![var("_x")])])),
            lam("_y", b, app(j.clone(), vec![app(g, vec![var("_y")])])),
            p,
        );
        let end_left = app(i, vec![left]);
        let end_right = app(j, vec![right]);
        self.emit(
            "MapComposition",
            &compose,
            pointwise(e.clone(), h.clone(), first.clone(), second.clone()),
            by_components(
                e.clone(),
                h.clone(),
                first,
                second,
                refl(e, end_left),
                refl(h, end_right),
            ),
            true,
        )
    }
    fn validity(&mut self) -> Result<()> {
        let (a, b, p, q, x) = (var("A"), var("B"), var("P"), var("Q"), var("p"));
        let predicates = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("P", predicate(a.clone())),
            ("Q", predicate(b.clone())),
        ];
        let mut input = predicates.to_vec();
        input.push(("p", product(a.clone(), b.clone())));
        let left = projection(a.clone(), b.clone(), x.clone(), false);
        let right = projection(a.clone(), b.clone(), x.clone(), true);
        let pl = app(p.clone(), vec![left]);
        let qr = app(q.clone(), vec![right]);
        let validity = valid(a.clone(), b.clone(), p.clone(), q.clone(), x.clone());
        self.emit(
            "Valid",
            &input,
            Expr::Sort0,
            product(pl.clone(), qr.clone()),
            false,
        )?;
        let mut assumed = input.clone();
        assumed.push(("valid", validity));
        for r in [false, true] {
            self.emit(
                if r { "ValidRight" } else { "ValidLeft" },
                &assumed,
                if r { qr.clone() } else { pl.clone() },
                projection(pl.clone(), qr.clone(), var("valid"), r),
                true,
            )?;
        }
        let mut values = predicates.to_vec();
        values.push(("x", a.clone()));
        values.push(("y", b.clone()));
        let px = app(p.clone(), vec![var("x")]);
        let qy = app(q.clone(), vec![var("y")]);
        values.push(("valid_x", px.clone()));
        values.push(("valid_y", qy.clone()));
        self.emit(
            "ValidMake",
            &values,
            valid(
                a.clone(),
                b.clone(),
                p.clone(),
                q.clone(),
                make(a.clone(), b.clone(), var("x"), var("y")),
            ),
            make(px, qy, var("valid_x"), var("valid_y")),
            true,
        )?;
        for r in [false, true] {
            let mut args = assumed.clone();
            args.push(("value", if r { b.clone() } else { a.clone() }));
            args.push((
                "valid_value",
                app(if r { q.clone() } else { p.clone() }, vec![var("value")]),
            ));
            let changed = update(a.clone(), b.clone(), x.clone(), var("value"), r);
            let unchanged = pc(
                if r { "ValidLeft" } else { "ValidRight" },
                vec![
                    a.clone(),
                    b.clone(),
                    p.clone(),
                    q.clone(),
                    x.clone(),
                    var("valid"),
                ],
            );
            let l = if r {
                projection(a.clone(), b.clone(), x.clone(), false)
            } else {
                var("value")
            };
            let end = if r {
                var("value")
            } else {
                projection(a.clone(), b.clone(), x.clone(), true)
            };
            let proof = pc(
                "ValidMake",
                vec![
                    a.clone(),
                    b.clone(),
                    p.clone(),
                    q.clone(),
                    l,
                    end,
                    if r {
                        unchanged.clone()
                    } else {
                        var("valid_value")
                    },
                    if r { var("valid_value") } else { unchanged },
                ],
            );
            self.emit(
                if r { "SetRightValid" } else { "SetLeftValid" },
                &args,
                valid(a.clone(), b.clone(), p.clone(), q.clone(), changed),
                proof,
                true,
            )?;
        }
        let (c, d, f, g, rc, rd) = (var("C"), var("D"), var("f"), var("g"), var("R"), var("S"));
        let mut args = vec![
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
            ("P", predicate(a.clone())),
            ("Q", predicate(b.clone())),
            ("R", predicate(c.clone())),
            ("S", predicate(d.clone())),
            ("f", function(a.clone(), c.clone())),
            ("g", function(b.clone(), d.clone())),
        ];
        args.push((
            "preserve_f",
            pi(
                "_x",
                a.clone(),
                pi(
                    "_valid",
                    app(p.clone(), vec![var("_x")]),
                    app(rc.clone(), vec![app(f.clone(), vec![var("_x")])]),
                ),
            ),
        ));
        args.push((
            "preserve_g",
            pi(
                "_y",
                b.clone(),
                pi(
                    "_valid",
                    app(q.clone(), vec![var("_y")]),
                    app(rd.clone(), vec![app(g.clone(), vec![var("_y")])]),
                ),
            ),
        ));
        args.extend(assumed[4..].iter().cloned());
        let l = projection(a.clone(), b.clone(), x.clone(), false);
        let r = projection(a.clone(), b.clone(), x.clone(), true);
        let vl = pc(
            "ValidLeft",
            vec![
                a.clone(),
                b.clone(),
                p.clone(),
                q.clone(),
                x.clone(),
                var("valid"),
            ],
        );
        let vr = pc(
            "ValidRight",
            vec![a.clone(), b.clone(), p, q, x.clone(), var("valid")],
        );
        let fl = app(f.clone(), vec![l.clone()]);
        let gr = app(g.clone(), vec![r.clone()]);
        let result = valid(
            c.clone(),
            d.clone(),
            rc.clone(),
            rd.clone(),
            map(a, b, c.clone(), d.clone(), f, g, x),
        );
        self.emit(
            "MapValid",
            &args,
            result,
            pc(
                "ValidMake",
                vec![
                    c,
                    d,
                    rc,
                    rd,
                    fl,
                    gr,
                    app(var("preserve_f"), vec![l, vl]),
                    app(var("preserve_g"), vec![r, vr]),
                ],
            ),
            true,
        )
    }
    fn empty(&mut self) -> Result<()> {
        self.emit("Unit", &[], Expr::Sort0, boolean(), false)?;
        self.emit("UnitZero", &[], pc("Unit", vec![]), bit(false), false)?;
        self.emit(
            "UnitValid",
            &[("unit", pc("Unit", vec![]))],
            Expr::Sort0,
            eq(boolean(), var("unit"), bit(false)),
            false,
        )?;
        self.emit(
            "UnitZeroValid",
            &[],
            pc("UnitValid", vec![pc("UnitZero", vec![])]),
            refl(boolean(), bit(false)),
            true,
        )?;
        self.emit(
            "UnitValidUnique",
            &[
                ("unit", pc("Unit", vec![])),
                ("valid", pc("UnitValid", vec![var("unit")])),
            ],
            eq(boolean(), var("unit"), pc("UnitZero", vec![])),
            var("valid"),
            true,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut products = Products(builder);
    products.definitions()?;
    products.observations()?;
    products.updates()?;
    products.mapping()?;
    products.validity()?;
    products.empty()
}
