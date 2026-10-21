//! Composed views use actual read/write, relation and validity witnesses.
use super::*;
impl Lenses<'_> {
    pub(super) fn algebra(&mut self) -> Result<()> {
        let (a, b, c, l, m, r, s, t) = (
            var("A"),
            var("B"),
            var("C"),
            var("l"),
            var("m"),
            var("R"),
            var("S"),
            var("T"),
        );
        let base = vec![
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("m", lens(b.clone(), c.clone())),
        ];
        let composed = compose(a.clone(), b.clone(), c.clone(), l.clone(), m.clone());
        let root = var("_s");
        let x = var("_x");
        let y = var("_y");
        let parent = get(a.clone(), b.clone(), l.clone(), root.clone());
        let child = put(b.clone(), c.clone(), m.clone(), parent.clone(), x.clone());
        let mut args = base.clone();
        args.extend([
            ("gs1", get_put(a.clone(), b.clone(), l.clone())),
            ("gs2", get_put(b.clone(), c.clone(), m.clone())),
        ]);
        let first = Equality {
            ty: b.clone(),
            left: get(
                a.clone(),
                b.clone(),
                l.clone(),
                put(a.clone(), b.clone(), l.clone(), root.clone(), child.clone()),
            ),
            right: child.clone(),
            proof: app(var("gs1"), vec![root.clone(), child.clone()]),
        }
        .mapped(
            lam(
                "_b",
                b.clone(),
                get(b.clone(), c.clone(), m.clone(), var("_b")),
            ),
            get(
                a.clone(),
                c.clone(),
                composed.clone(),
                put(
                    a.clone(),
                    c.clone(),
                    composed.clone(),
                    root.clone(),
                    x.clone(),
                ),
            ),
            get(b.clone(), c.clone(), m.clone(), child.clone()),
            c.clone(),
        );
        let end = Equality {
            ty: c.clone(),
            left: get(b.clone(), c.clone(), m.clone(), child.clone()),
            right: x.clone(),
            proof: app(var("gs2"), vec![parent.clone(), x.clone()]),
        };
        self.emit(
            "ComposeGetPut",
            &args,
            get_put(a.clone(), c.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                lam("_x", c.clone(), first.then(end)?.proof),
            ),
            true,
        )?;
        let mut related = base.clone();
        related.extend([("R", relation(a.clone())), ("S", relation(b.clone()))]);
        let mut args = related.clone();
        args.extend([
            ("trans", transitive(a.clone(), r.clone())),
            (
                "respect1",
                put_respect(a.clone(), b.clone(), r.clone(), s.clone(), l.clone()),
            ),
            ("pg1", put_get(a.clone(), b.clone(), r.clone(), l.clone())),
            ("pg2", put_get(b.clone(), c.clone(), s.clone(), m.clone())),
        ]);
        let current = get(b.clone(), c.clone(), m.clone(), parent.clone());
        let restored = put(b.clone(), c.clone(), m.clone(), parent.clone(), current);
        let start = put(
            a.clone(),
            b.clone(),
            l.clone(),
            root.clone(),
            restored.clone(),
        );
        let middle = put(
            a.clone(),
            b.clone(),
            l.clone(),
            root.clone(),
            parent.clone(),
        );
        let h = app(
            var("respect1"),
            vec![
                root.clone(),
                restored,
                parent.clone(),
                app(var("pg2"), vec![parent.clone()]),
            ],
        );
        self.emit(
            "ComposePutGet",
            &args,
            put_get(a.clone(), c.clone(), r.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                app(
                    var("trans"),
                    vec![
                        start,
                        middle,
                        root.clone(),
                        h,
                        app(var("pg1"), vec![root.clone()]),
                    ],
                ),
            ),
            true,
        )?;
        let mut args = related.clone();
        args.extend([
            ("trans", transitive(a.clone(), r.clone())),
            ("gs1", get_put(a.clone(), b.clone(), l.clone())),
            ("pp1", put_put(a.clone(), b.clone(), r.clone(), l.clone())),
            (
                "respect1",
                put_respect(a.clone(), b.clone(), r.clone(), s.clone(), l.clone()),
            ),
            ("pp2", put_put(b.clone(), c.clone(), s.clone(), m.clone())),
        ]);
        let next_root = put(a.clone(), b.clone(), l.clone(), root.clone(), child.clone());
        let read_after = get(a.clone(), b.clone(), l.clone(), next_root.clone());
        let second = put(
            b.clone(),
            c.clone(),
            m.clone(),
            read_after.clone(),
            y.clone(),
        );
        let final_child = put(b.clone(), c.clone(), m.clone(), parent.clone(), y.clone());
        let e = Equality {
            ty: b.clone(),
            left: read_after,
            right: child.clone(),
            proof: app(var("gs1"), vec![root.clone(), child.clone()]),
        }
        .mapped(
            lam(
                "_b",
                b.clone(),
                put(b.clone(), c.clone(), m.clone(), var("_b"), y.clone()),
            ),
            second.clone(),
            put(b.clone(), c.clone(), m.clone(), child.clone(), y.clone()),
            b.clone(),
        );
        let unchanged = Equality {
            ty: b.clone(),
            left: final_child.clone(),
            right: final_child.clone(),
            proof: refl(b.clone(), final_child.clone()),
        };
        let child_relation = transfer(
            b.clone(),
            s.clone(),
            e,
            unchanged,
            app(var("pp2"), vec![parent.clone(), x.clone(), y.clone()]),
        );
        let h1 = app(
            var("pp1"),
            vec![root.clone(), child.clone(), second.clone()],
        );
        let h2 = app(
            var("respect1"),
            vec![
                root.clone(),
                second.clone(),
                final_child.clone(),
                child_relation,
            ],
        );
        let h = app(
            var("trans"),
            vec![
                put(a.clone(), b.clone(), l.clone(), next_root, second.clone()),
                put(a.clone(), b.clone(), l.clone(), root.clone(), second),
                put(a.clone(), b.clone(), l.clone(), root.clone(), final_child),
                h1,
                h2,
            ],
        );
        self.emit(
            "ComposePutPut",
            &args,
            put_put(a.clone(), c.clone(), r.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                lam("_x", c.clone(), lam("_y", c.clone(), h)),
            ),
            true,
        )?;
        let mut triple = related;
        triple.push(("T", relation(c.clone())));
        let mut args = triple.clone();
        args.extend([
            (
                "gr1",
                get_respect(a.clone(), b.clone(), r.clone(), s.clone(), l.clone()),
            ),
            (
                "gr2",
                get_respect(b.clone(), c.clone(), s.clone(), t.clone(), m.clone()),
            ),
        ]);
        let other = var("_t");
        let gother = get(a.clone(), b.clone(), l.clone(), other.clone());
        self.emit(
            "ComposeGetRespect",
            &args,
            get_respect(a.clone(), c.clone(), r.clone(), t.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_t",
                    a.clone(),
                    lam(
                        "_h",
                        holds(r.clone(), root.clone(), other.clone()),
                        app(
                            var("gr2"),
                            vec![
                                parent.clone(),
                                gother,
                                app(var("gr1"), vec![root.clone(), other, var("_h")]),
                            ],
                        ),
                    ),
                ),
            ),
            true,
        )?;
        let mut args = triple;
        args.extend([
            (
                "pr1",
                put_respect(a.clone(), b.clone(), r.clone(), s.clone(), l.clone()),
            ),
            (
                "pr2",
                put_respect(b.clone(), c.clone(), s.clone(), t.clone(), m.clone()),
            ),
        ]);
        self.emit(
            "ComposePutRespect",
            &args,
            put_respect(a.clone(), c.clone(), r, t.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_x",
                    c.clone(),
                    lam(
                        "_y",
                        c.clone(),
                        lam(
                            "_h",
                            holds(t, x.clone(), y.clone()),
                            app(
                                var("pr1"),
                                vec![
                                    root.clone(),
                                    child,
                                    put(b.clone(), c.clone(), m.clone(), parent.clone(), y.clone()),
                                    app(var("pr2"), vec![parent.clone(), x.clone(), y, var("_h")]),
                                ],
                            ),
                        ),
                    ),
                ),
            ),
            true,
        )?;
        let (p, q, u) = (var("P"), var("Q"), var("U"));
        let mut valid = base;
        valid.extend([
            ("P", predicate(a.clone())),
            ("Q", predicate(b.clone())),
            ("U", predicate(c.clone())),
        ]);
        let mut args = valid.clone();
        args.extend([
            (
                "vg1",
                get_valid(a.clone(), b.clone(), p.clone(), q.clone(), l.clone()),
            ),
            (
                "vg2",
                get_valid(b.clone(), c.clone(), q.clone(), u.clone(), m.clone()),
            ),
        ]);
        self.emit(
            "ComposeGetValid",
            &args,
            get_valid(a.clone(), c.clone(), p.clone(), u.clone(), composed.clone()),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_valid",
                    app(p.clone(), vec![root.clone()]),
                    app(
                        var("vg2"),
                        vec![
                            parent.clone(),
                            app(var("vg1"), vec![root.clone(), var("_valid")]),
                        ],
                    ),
                ),
            ),
            true,
        )?;
        let mut args = valid;
        args.extend([
            (
                "vg1",
                get_valid(a.clone(), b.clone(), p.clone(), q.clone(), l.clone()),
            ),
            (
                "vp1",
                put_valid(a.clone(), b.clone(), p.clone(), q.clone(), l.clone()),
            ),
            (
                "vp2",
                put_valid(b.clone(), c.clone(), q, u.clone(), m.clone()),
            ),
        ]);
        let leaf = put(b, c.clone(), m, parent.clone(), x.clone());
        let h = app(
            var("vp1"),
            vec![
                root.clone(),
                leaf,
                var("_valid"),
                app(
                    var("vp2"),
                    vec![
                        parent,
                        x.clone(),
                        app(var("vg1"), vec![root, var("_valid")]),
                        var("_value_valid"),
                    ],
                ),
            ],
        );
        self.emit(
            "ComposePutValid",
            &args,
            put_valid(a.clone(), c.clone(), p.clone(), u.clone(), composed),
            lam(
                "_s",
                a.clone(),
                lam(
                    "_x",
                    c.clone(),
                    lam(
                        "_valid",
                        app(p, vec![var("_s")]),
                        lam("_value_valid", app(u, vec![x]), h),
                    ),
                ),
            ),
            true,
        )?;
        self.identity_and_associativity()
    }
    fn identity_and_associativity(&mut self) -> Result<()> {
        let (a, b, c, d) = (var("A"), var("B"), var("C"), var("D"));
        let (l, m, n) = (var("l"), var("m"), var("n"));
        let base = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("s", a.clone()),
            ("x", b.clone()),
        ];
        for (name, view) in [
            (
                "IdentityLeft",
                compose(
                    a.clone(),
                    a.clone(),
                    b.clone(),
                    lc("Identity", vec![a.clone()]),
                    l.clone(),
                ),
            ),
            (
                "IdentityRight",
                compose(
                    a.clone(),
                    b.clone(),
                    b.clone(),
                    l.clone(),
                    lc("Identity", vec![b.clone()]),
                ),
            ),
        ] {
            let g = get(a.clone(), b.clone(), l.clone(), var("s"));
            self.theorem(
                &format!("{name}Get"),
                &base[..4],
                Equality {
                    ty: b.clone(),
                    left: get(a.clone(), b.clone(), view.clone(), var("s")),
                    right: g.clone(),
                    proof: refl(b.clone(), g),
                },
            )?;
            let p = put(a.clone(), b.clone(), l.clone(), var("s"), var("x"));
            self.theorem(
                &format!("{name}Put"),
                &base,
                Equality {
                    ty: a.clone(),
                    left: put(a.clone(), b.clone(), view, var("s"), var("x")),
                    right: p.clone(),
                    proof: refl(a.clone(), p),
                },
            )?;
        }
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("D", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("m", lens(b.clone(), c.clone())),
            ("n", lens(c.clone(), d.clone())),
            ("s", a.clone()),
            ("x", d.clone()),
        ];
        let left = compose(
            a.clone(),
            c.clone(),
            d.clone(),
            compose(a.clone(), b.clone(), c.clone(), l.clone(), m.clone()),
            n.clone(),
        );
        let right = compose(
            a.clone(),
            b.clone(),
            d.clone(),
            l,
            compose(b, c, d.clone(), m, n),
        );
        let g = get(a.clone(), d.clone(), right.clone(), var("s"));
        self.theorem(
            "AssociativeGet",
            &args[..8],
            Equality {
                ty: d.clone(),
                left: get(a.clone(), d.clone(), left.clone(), var("s")),
                right: g.clone(),
                proof: refl(d.clone(), g),
            },
        )?;
        let p = put(a.clone(), d.clone(), right, var("s"), var("x"));
        self.theorem(
            "AssociativePut",
            &args,
            Equality {
                ty: a.clone(),
                left: put(a.clone(), d, left, var("s"), var("x")),
                right: p.clone(),
                proof: refl(a, p),
            },
        )
    }
}
