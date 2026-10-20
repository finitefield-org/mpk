//! Genuine unbounded strict gaps for the computed right remainders.
use super::*;
fn gap(a: Expr, b: Expr) -> Expr {
    sub(b, succ(a))
}
fn decomposition(a: Expr, b: Expr) -> Expr {
    eq(nat(), b.clone(), add(succ(a.clone()), gap(a, b)))
}
fn ordered(a: Expr, b: Expr) -> Expr {
    eq(boolean(), less(a, b), bit(true))
}
fn with_order(a: Expr, b: Expr) -> Expr {
    pi(
        "_ordered",
        ordered(a.clone(), b.clone()),
        decomposition(a, b),
    )
}
fn nat_law(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}

impl Shifts<'_> {
    fn right_strict_decomposition(&mut self) -> Result<()> {
        let (a, b, n, k, m) = (
            var("a"),
            var("b"),
            var("_outer"),
            var("_inner"),
            var("_other"),
        );
        let impossible = |x: Expr, y: Expr| -> Expr {
            let bad = Equality::boolean(
                less(x.clone(), y.clone()),
                bit(false),
                nc("LessZero", vec![x.clone()]),
            )
            .reverse()
            .then(Equality::boolean(
                less(x.clone(), y.clone()),
                bit(true),
                var("_ordered"),
            ))
            .expect("same less endpoint");
            bc("FalseElim", vec![decomposition(x, y), bad.proof])
        };
        let base0 = lam(
            "_ordered",
            ordered(zero(), zero()),
            impossible(zero(), zero()),
        );
        let subzero = nat_law(
            "SubSuccBoth",
            vec![k.clone(), zero()],
            sub(succ(k.clone()), succ(zero())),
            sub(k.clone(), zero()),
        )
        .then(nat_law(
            "SubZero",
            vec![k.clone()],
            sub(k.clone(), zero()),
            k.clone(),
        ))?;
        let right = subzero
            .on_right(succ(zero()))
            .then(nat_law(
                "SuccAdd",
                vec![zero(), k.clone()],
                add(succ(zero()), k.clone()),
                succ(add(zero(), k.clone())),
            ))?
            .then(
                nat_law(
                    "ZeroAdd",
                    vec![k.clone()],
                    add(zero(), k.clone()),
                    k.clone(),
                )
                .on_succ(),
            )?;
        let base_step = lam(
            "_inner",
            nat(),
            lam(
                "_ignored",
                with_order(zero(), k.clone()),
                lam(
                    "_ordered",
                    ordered(zero(), succ(k.clone())),
                    right.reverse().proof,
                ),
            ),
        );
        let base = lam(
            "_other",
            nat(),
            induct(
                "_inner",
                with_order(zero(), k.clone()),
                base0,
                base_step,
                m.clone(),
            ),
        );
        let property = pi("_other", nat(), with_order(n.clone(), m.clone()));
        let less_step = Equality::boolean(
            less(succ(n.clone()), succ(k.clone())),
            less(n.clone(), k.clone()),
            nc("LessSuccBoth", vec![n.clone(), k.clone()]),
        );
        let prior_order = less_step.reverse().then(Equality::boolean(
            less(succ(n.clone()), succ(k.clone())),
            bit(true),
            var("_ordered"),
        ))?;
        let g = gap(n.clone(), k.clone());
        let decrease = nat_law(
            "SubSuccBoth",
            vec![k.clone(), succ(n.clone())],
            sub(succ(k.clone()), succ(succ(n.clone()))),
            g.clone(),
        );
        let right = decrease
            .on_right(succ(succ(n.clone())))
            .then(nat_law(
                "SuccAdd",
                vec![succ(n.clone()), g.clone()],
                add(succ(succ(n.clone())), g.clone()),
                succ(add(succ(n.clone()), g.clone())),
            ))?
            .then(
                Equality::natural(
                    k.clone(),
                    add(succ(n.clone()), g),
                    app(var("_prior"), vec![k.clone(), prior_order.proof]),
                )
                .reverse()
                .on_succ(),
            )?;
        let step = lam(
            "_inner",
            nat(),
            lam(
                "_ignored",
                with_order(succ(n.clone()), k.clone()),
                lam(
                    "_ordered",
                    ordered(succ(n.clone()), succ(k.clone())),
                    right.reverse().proof,
                ),
            ),
        );
        let inner = induct(
            "_inner",
            with_order(succ(n.clone()), k.clone()),
            lam(
                "_ordered",
                ordered(succ(n.clone()), zero()),
                impossible(succ(n.clone()), zero()),
            ),
            step,
            m.clone(),
        );
        let update = lam(
            "_outer",
            nat(),
            lam("_prior", property.clone(), lam("_other", nat(), inner)),
        );
        self.emit(
            "RightStrictDecomposition",
            &[("a", nat()), ("b", nat())],
            with_order(a.clone(), b.clone()),
            app(induct("_outer", property, base, update, a), vec![b]),
            true,
        )
    }
}
impl Shifts<'_> {
    pub(super) fn variable_right_bounds(&mut self) -> Result<()> {
        self.right_strict_decomposition()?;
        let (a, b) = (var("a"), var("b"));
        let g = gap(a.clone(), b.clone());
        let e = known(
            "RightStrictDecomposition",
            vec![a.clone(), b.clone(), var("ordered")],
            b.clone(),
            add(succ(a.clone()), g.clone()),
        )
        .then(law(
            "SuccAdd",
            vec![a.clone(), g.clone()],
            add(succ(a.clone()), g.clone()),
            succ(add(a.clone(), g.clone())),
        ))?
        .then(
            law(
                "AddOne",
                vec![add(a.clone(), g.clone())],
                add(add(a.clone(), g), one()),
                succ(add(a.clone(), gap(a.clone(), b.clone()))),
            )
            .reverse(),
        )?;
        self.theorem(
            "RightStrictGap",
            &[("a", nat()), ("b", nat()), ("ordered", ordered(a, b))],
            e,
        )?;
        self.right_bound_compose()?;
        self.right_bound_choice()
    }
    fn right_bound_compose(&mut self) -> Result<()> {
        let (r, s, p, q) = (
            var("remainder"),
            var("step_remainder"),
            var("power"),
            var("step_power"),
        );
        let args = [
            ("remainder", nat()),
            ("step_remainder", nat()),
            ("power", nat()),
            ("step_power", nat()),
            ("first_bound", ordered(r.clone(), p.clone())),
            ("second_bound", ordered(s.clone(), q.clone())),
        ];
        let (g, h) = (gap(r.clone(), p.clone()), gap(s.clone(), q.clone()));
        let second = known(
            "RightStrictGap",
            vec![s.clone(), q.clone(), var("second_bound")],
            q.clone(),
            add(add(s.clone(), h.clone()), one()),
        );
        let proof = mul_right(p.clone(), second);
        let middle = add(
            add(mul(p.clone(), s.clone()), mul(p.clone(), h.clone())),
            p.clone(),
        );
        let poly = self.polynomial(&args, proof.right.clone(), middle)?;
        let proof = proof.then(poly)?.then(
            known(
                "RightStrictGap",
                vec![r.clone(), p.clone(), var("first_bound")],
                p.clone(),
                add(add(r.clone(), g.clone()), one()),
            )
            .on_right(add(mul(p.clone(), s.clone()), mul(p.clone(), h.clone()))),
        )?;
        let rem = add(r.clone(), mul(p.clone(), s.clone()));
        let witness = add(g, mul(p.clone(), h));
        let target = add(add(rem.clone(), witness.clone()), one());
        let poly = self.polynomial(&args, proof.right.clone(), target)?;
        let balance = proof.then(poly)?.reverse();
        self.theorem(
            "RightRemainderComposeBound",
            &args,
            Equality::boolean(
                less(rem.clone(), mul(p.clone(), q.clone())),
                bit(true),
                sh(
                    "CountComplementBound",
                    vec![rem, witness, mul(p, q), balance.proof],
                ),
            ),
        )
    }
    fn right_bound_choice(&mut self) -> Result<()> {
        let (f, r, s, p, q) = (
            var("flag"),
            var("remainder"),
            var("step_remainder"),
            var("power"),
            var("step_power"),
        );
        let args = [
            ("flag", boolean()),
            ("remainder", nat()),
            ("step_remainder", nat()),
            ("power", nat()),
            ("step_power", nat()),
            ("first_bound", ordered(r.clone(), p.clone())),
            ("second_bound", ordered(s.clone(), q.clone())),
        ];
        let rem = add(r.clone(), mul(p.clone(), s.clone()));
        let pow = mul(p.clone(), q.clone());
        let endpoints = |flag: Expr| {
            less(
                select(flag.clone(), rem.clone(), r.clone()),
                select(flag, pow.clone(), p.clone()),
            )
        };
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(boolean(), endpoints(e["_bound_flag"].clone()), bit(true))
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_bound_flag"] == bit(true);
            let rr = literal(yes, rem.clone(), r.clone());
            let pp = literal(yes, pow.clone(), p.clone());
            let raw = rr.right.clone();
            let first = less_left(rr, pp.left.clone()).then(less_right(raw, pp))?;
            let known = if yes {
                Equality::boolean(
                    less(rem.clone(), pow.clone()),
                    bit(true),
                    sh(
                        "RightRemainderComposeBound",
                        vec![
                            r.clone(),
                            s.clone(),
                            p.clone(),
                            q.clone(),
                            var("first_bound"),
                            var("second_bound"),
                        ],
                    ),
                )
            } else {
                Equality::boolean(less(r.clone(), p.clone()), bit(true), var("first_bound"))
            };
            Ok(first.then(known)?.proof)
        };
        let proof = app(
            boolean_cases(&["_bound_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![f.clone()],
        );
        self.theorem(
            "RightConditionalRemainderBound",
            &args,
            Equality::boolean(endpoints(f), bit(true), proof),
        )
    }
}
