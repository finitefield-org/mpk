//! Unbounded Nat gaps and bounds used by restoring division.
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

impl Division<'_> {
    pub(super) fn natural_bounds(&mut self) -> Result<()> {
        self.strict_decomposition()?;
        let (a, b) = (var("a"), var("b"));
        let g = gap(a.clone(), b.clone());
        let e = instance(
            "StrictDecomposition",
            vec![a.clone(), b.clone(), var("ordered")],
            b.clone(),
            add(succ(a.clone()), g.clone()),
        )
        .then(nat_law(
            "SuccAdd",
            vec![a.clone(), g.clone()],
            add(succ(a.clone()), g.clone()),
            succ(add(a.clone(), g.clone())),
        ))?
        .then(
            nat_law(
                "AddOne",
                vec![add(a.clone(), g.clone())],
                add(add(a.clone(), g.clone()), one()),
                succ(add(a.clone(), g)),
            )
            .reverse(),
        )?;
        self.theorem(
            "StrictGap",
            &[("a", nat()), ("b", nat()), ("ordered", ordered(a, b))],
            e,
        )?;
        self.double_bit_bound()?;
        self.larger_divisor_sum()?;
        self.remove_divisor_bound()
    }
    fn strict_decomposition(&mut self) -> Result<()> {
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
            "StrictDecomposition",
            &[("a", nat()), ("b", nat())],
            with_order(a.clone(), b.clone()),
            app(induct("_outer", property, base, update, a), vec![b]),
            true,
        )
    }
    fn double_bit_bound(&mut self) -> Result<()> {
        let (r, d) = (var("r"), var("d"));
        let property = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_ordered",
                ordered(r.clone(), d.clone()),
                eq(
                    boolean(),
                    less(
                        add(mul(two(), r.clone()), bit_value(env["low"].clone())),
                        mul(two(), d.clone()),
                    ),
                    bit(true),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["low"] == bit(true);
            let g = gap(r.clone(), d.clone());
            let args = [
                ("r", nat()),
                ("d", nat()),
                ("_ordered", ordered(r.clone(), d.clone())),
            ];
            let n = if flag { one() } else { zero() };
            let a = add(mul(two(), r.clone()), n);
            let k = add(mul(two(), g.clone()), if flag { zero() } else { one() });
            let d_eq = instance(
                "StrictGap",
                vec![r.clone(), d.clone(), var("_ordered")],
                d.clone(),
                add(add(r.clone(), g), one()),
            );
            let e = mul_right(two(), d_eq);
            let next = self.polynomial(
                &args,
                e.right.clone(),
                add(a.clone(), add(k.clone(), one())),
            )?;
            let e = e.then(next)?.then(
                nat_law(
                    "AddOne",
                    vec![k.clone()],
                    add(k.clone(), one()),
                    succ(k.clone()),
                )
                .on_right(a.clone()),
            )?;
            let first = less_left(
                bit_literal(flag).on_right(mul(two(), r.clone())),
                mul(two(), d.clone()),
            );
            let proof = first
                .then(less_right(a.clone(), e))?
                .then(Equality::boolean(
                    less(a.clone(), add(a.clone(), succ(k.clone()))),
                    bit(true),
                    order("LessPositiveExtension", vec![a, k]),
                ))?;
            Ok(lam("_ordered", ordered(r.clone(), d.clone()), proof.proof))
        };
        let proof = boolean_cases(&["low"], &BTreeMap::new(), &property, &mut leaf)?;
        self.emit(
            "DoubleBitBound",
            &[("r", nat()), ("d", nat())],
            pi(
                "low",
                boolean(),
                property(&[("low", var("low"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn larger_divisor_sum(&mut self) -> Result<()> {
        let (d, m, t) = (var("d"), var("modulus"), var("t"));
        let args = [
            ("d", nat()),
            ("modulus", nat()),
            ("t", nat()),
            ("ordered", ordered(d.clone(), m.clone())),
        ];
        let g = gap(d.clone(), m.clone());
        let s = add(add(t.clone(), d.clone()), m.clone());
        let p = add(add(t, g.clone()), one());
        let twice = add(d.clone(), d.clone());
        let e = instance(
            "StrictGap",
            vec![d.clone(), m.clone(), var("ordered")],
            m.clone(),
            add(add(d.clone(), g), one()),
        )
        .on_right(add(var("t"), d.clone()));
        let next = self.polynomial(&args, e.right.clone(), add(p.clone(), twice.clone()))?;
        let e = e.then(next)?;
        let two_d = self.polynomial(&args, mul(two(), d.clone()), twice.clone())?;
        let result = less_left(e, mul(two(), d.clone()))
            .then(less_right(add(p.clone(), twice.clone()), two_d))?
            .then(less_right(
                add(p.clone(), twice.clone()),
                nat_law(
                    "ZeroAdd",
                    vec![twice.clone()],
                    add(zero(), twice.clone()),
                    twice.clone(),
                )
                .reverse(),
            ))?
            .then(Equality::boolean(
                less(add(p.clone(), twice.clone()), add(zero(), twice.clone())),
                less(p.clone(), zero()),
                nc("LessTranslate", vec![p.clone(), zero(), twice]),
            ))?
            .then(Equality::boolean(
                less(p.clone(), zero()),
                bit(false),
                nc("LessZero", vec![p]),
            ))?;
        assert_eq!(result.left, less(s, mul(two(), d)));
        self.theorem("LargerDivisorSum", &args, result)
    }
    fn remove_divisor_bound(&mut self) -> Result<()> {
        let (d, t, k) = (var("d"), var("t"), var("sum"));
        let args = [
            ("d", nat()),
            ("t", nat()),
            ("sum", nat()),
            ("equation", eq(nat(), add(t.clone(), d.clone()), k.clone())),
            ("bound", ordered(k.clone(), mul(two(), d.clone()))),
        ];
        let twice = add(d.clone(), d.clone());
        let result = Equality::boolean(
            less(add(t.clone(), d.clone()), twice.clone()),
            less(t.clone(), d.clone()),
            nc("LessTranslate", vec![t.clone(), d.clone(), d.clone()]),
        )
        .reverse()
        .then(less_left(
            Equality::natural(add(t, d.clone()), k.clone(), var("equation")),
            twice.clone(),
        ))?
        .then(less_right(
            k.clone(),
            self.polynomial(&args, twice, mul(two(), d.clone()))?,
        ))?
        .then(Equality::boolean(
            less(k, mul(two(), d)),
            bit(true),
            var("bound"),
        ))?;
        self.theorem("RemoveDivisorBound", &args, result)
    }
}
