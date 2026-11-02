//! Empty paths and disjoint observations compose without function extensionality.
use super::*;

fn frame(a: Expr, b: Expr, c: Expr, l: Expr, k: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_x",
            b.clone(),
            eq(
                c.clone(),
                get(
                    a.clone(),
                    c.clone(),
                    k.clone(),
                    put(a.clone(), b, l, var("_s"), var("_x")),
                ),
                get(a, c, k, var("_s")),
            ),
        ),
    )
}
impl Lenses<'_> {
    pub(super) fn frames(&mut self) -> Result<()> {
        self.identity_laws()?;
        self.inner_frame()?;
        self.outer_frame()
    }
    fn identity_laws(&mut self) -> Result<()> {
        let (a, r, p) = (var("A"), var("R"), var("P"));
        let identity = lc("Identity", vec![a.clone()]);
        let args = [("A", Expr::Sort0)];
        self.emit(
            "IdentityGetPut",
            &args,
            get_put(a.clone(), a.clone(), identity.clone()),
            lam(
                "_s",
                a.clone(),
                lam("_x", a.clone(), refl(a.clone(), var("_x"))),
            ),
            true,
        )?;
        let mut args = args.to_vec();
        args.push(("R", relation(a.clone())));
        self.emit(
            "IdentityGetRespect",
            &args,
            get_respect(a.clone(), a.clone(), r.clone(), r.clone(), identity.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_t",
                    a.clone(),
                    lam("_h", holds(r.clone(), var("_s"), var("_t")), var("_h")),
                ),
            ),
            true,
        )?;
        self.emit(
            "IdentityPutRespect",
            &args,
            put_respect(a.clone(), a.clone(), r.clone(), r.clone(), identity.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_x",
                    a.clone(),
                    lam(
                        "_y",
                        a.clone(),
                        lam("_h", holds(r.clone(), var("_x"), var("_y")), var("_h")),
                    ),
                ),
            ),
            true,
        )?;
        args.push((
            "refl",
            pi(
                "_value",
                a.clone(),
                holds(r.clone(), var("_value"), var("_value")),
            ),
        ));
        self.emit(
            "IdentityPutGet",
            &args,
            put_get(a.clone(), a.clone(), r.clone(), identity.clone()),
            lam("_s", a.clone(), app(var("refl"), vec![var("_s")])),
            true,
        )?;
        self.emit(
            "IdentityPutPut",
            &args,
            put_put(a.clone(), a.clone(), r, identity.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_x",
                    a.clone(),
                    lam("_y", a.clone(), app(var("refl"), vec![var("_y")])),
                ),
            ),
            true,
        )?;
        let args = [("A", Expr::Sort0), ("P", predicate(a.clone()))];
        self.emit(
            "IdentityGetValid",
            &args,
            get_valid(a.clone(), a.clone(), p.clone(), p.clone(), identity.clone()),
            lam(
                "_s",
                a.clone(),
                lam("_valid", app(p.clone(), vec![var("_s")]), var("_valid")),
            ),
            true,
        )?;
        self.emit(
            "IdentityPutValid",
            &args,
            put_valid(a.clone(), a.clone(), p.clone(), p.clone(), identity),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_x",
                    a,
                    lam(
                        "_valid",
                        app(p.clone(), vec![var("_s")]),
                        lam("_value_valid", app(p, vec![var("_x")]), var("_value_valid")),
                    ),
                ),
            ),
            true,
        )
    }
    fn inner_frame(&mut self) -> Result<()> {
        let (a, b, c, d) = (var("A"), var("B"), var("C"), var("D"));
        let (l, m, n) = (var("l"), var("m"), var("n"));
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("m", lens(b.clone(), c.clone())),
            ("n", lens(b.clone(), d.clone())),
            ("read_write", get_put(a.clone(), b.clone(), l.clone())),
            (
                "disjoint",
                frame(b.clone(), c.clone(), d.clone(), m.clone(), n.clone()),
            ),
        ];
        let write_view = compose(a.clone(), b.clone(), c.clone(), l.clone(), m.clone());
        let read_view = compose(a.clone(), b.clone(), d.clone(), l.clone(), n.clone());
        let parent = get(a.clone(), b.clone(), l.clone(), var("_s"));
        let updated = put(b.clone(), c.clone(), m, parent.clone(), var("_x"));
        let parent_after = get(
            a.clone(),
            b.clone(),
            l.clone(),
            put(a.clone(), b.clone(), l, var("_s"), updated.clone()),
        );
        let first = Equality {
            ty: b.clone(),
            left: parent_after,
            right: updated.clone(),
            proof: app(var("read_write"), vec![var("_s"), updated.clone()]),
        }
        .mapped(
            lam(
                "_child_root",
                b.clone(),
                get(b.clone(), d.clone(), n.clone(), var("_child_root")),
            ),
            get(
                a.clone(),
                d.clone(),
                read_view.clone(),
                put(
                    a.clone(),
                    c.clone(),
                    write_view.clone(),
                    var("_s"),
                    var("_x"),
                ),
            ),
            get(b.clone(), d.clone(), n.clone(), updated),
            d.clone(),
        );
        let last = Equality {
            ty: d.clone(),
            left: first.right.clone(),
            right: get(b, d.clone(), n, parent.clone()),
            proof: app(var("disjoint"), vec![parent, var("_x")]),
        };
        self.emit(
            "ComposeInnerFrame",
            &args,
            frame(a.clone(), c.clone(), d, write_view, read_view),
            lam("_s", a, lam("_x", c, first.then(last)?.proof)),
            true,
        )
    }
    fn outer_frame(&mut self) -> Result<()> {
        let (a, b, c, d, e) = (var("A"), var("B"), var("C"), var("D"), var("E"));
        let (l, k, m, n) = (var("l"), var("k"), var("m"), var("n"));
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
            ("E", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("k", lens(a.clone(), c.clone())),
            ("m", lens(b.clone(), d.clone())),
            ("n", lens(c.clone(), e.clone())),
            (
                "disjoint",
                frame(a.clone(), b.clone(), c.clone(), l.clone(), k.clone()),
            ),
        ];
        let write_view = compose(a.clone(), b.clone(), d.clone(), l.clone(), m.clone());
        let read_view = compose(a.clone(), c.clone(), e.clone(), k.clone(), n.clone());
        let update = put(
            b.clone(),
            d.clone(),
            m,
            get(a.clone(), b.clone(), l.clone(), var("_s")),
            var("_x"),
        );
        let left = get(
            a.clone(),
            c.clone(),
            k.clone(),
            put(a.clone(), b, l, var("_s"), update.clone()),
        );
        let right = get(a.clone(), c.clone(), k, var("_s"));
        let proof = Equality {
            ty: c.clone(),
            left,
            right,
            proof: app(var("disjoint"), vec![var("_s"), update]),
        }
        .mapped(
            lam(
                "_other_parent",
                c.clone(),
                get(c, e.clone(), n, var("_other_parent")),
            ),
            get(
                a.clone(),
                e.clone(),
                read_view.clone(),
                put(
                    a.clone(),
                    d.clone(),
                    write_view.clone(),
                    var("_s"),
                    var("_x"),
                ),
            ),
            get(a.clone(), e.clone(), read_view.clone(), var("_s")),
            e.clone(),
        );
        self.emit(
            "ComposeOuterFrame",
            &args,
            frame(a.clone(), d.clone(), e, write_view, read_view),
            lam("_s", a, lam("_x", d, proof.proof)),
            true,
        )
    }
}
