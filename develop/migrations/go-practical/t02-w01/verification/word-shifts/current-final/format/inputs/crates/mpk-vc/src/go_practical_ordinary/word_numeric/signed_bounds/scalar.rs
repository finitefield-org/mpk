//! Ordinary arbitrary-Nat bounds and the computed magnitude gap.
use super::*;

impl Bounds<'_> {
    pub(super) fn gap_less_equal(&mut self) -> Result<()> {
        let (a, g) = (var("a"), var("gap"));
        let e = less_left(
            nlaw(
                "AddComm",
                vec![a.clone(), g.clone()],
                add(a.clone(), g.clone()),
                add(g.clone(), a.clone()),
            ),
            a.clone(),
        )
        .then(less_right(
            add(g.clone(), a.clone()),
            nlaw(
                "ZeroAdd",
                vec![a.clone()],
                add(zero(), a.clone()),
                a.clone(),
            )
            .reverse(),
        ))?
        .then(Equality::boolean(
            less(add(g.clone(), a.clone()), add(zero(), a.clone())),
            less(g.clone(), zero()),
            nc("LessTranslate", vec![g.clone(), zero(), a.clone()]),
        ))?
        .then(Equality::boolean(
            less(g.clone(), zero()),
            bit(false),
            nc("LessZero", vec![g.clone()]),
        ))?;
        let e = bool_not(e).then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        self.theorem(
            "GapLessEqual",
            &[("a", nat()), ("gap", nat())],
            Equality {
                left: le(a.clone(), add(a, g)),
                ..e
            },
        )
    }
    pub(super) fn scalar_balance(&mut self) -> Result<()> {
        let (u, n, l, c, h, m) = (
            var("ux"),
            var("un"),
            var("low"),
            var("complement"),
            var("half"),
            var("modulus"),
        );
        let args = ["ux", "un", "low", "complement", "half", "modulus"]
            .into_iter()
            .map(|s| (s, nat()))
            .collect::<Vec<_>>();
        let guards = |flag: Expr| -> Vec<Binder> {
            vec![
                (
                    "_split",
                    eq(
                        nat(),
                        u.clone(),
                        add(l.clone(), mul(h.clone(), bit_value(flag.clone()))),
                    ),
                ),
                (
                    "_complement",
                    eq(nat(), add(add(l.clone(), c.clone()), one()), h.clone()),
                ),
                ("_double", eq(nat(), add(h.clone(), h.clone()), m.clone())),
                (
                    "_negative",
                    pi(
                        "_sign",
                        eq(boolean(), flag, bit(true)),
                        eq(nat(), add(u.clone(), n.clone()), m.clone()),
                    ),
                ),
            ]
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let flag = env["sign"].clone();
            let mut p = eq(
                nat(),
                add(
                    choose(flag.clone(), n.clone(), u.clone(), nat()),
                    choose(flag.clone(), l.clone(), add(c.clone(), one()), nat()),
                ),
                h.clone(),
            );
            for (name, ty) in guards(flag).into_iter().rev() {
                p = pi(name, ty, p);
            }
            p
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = env["sign"] == bit(true);
            let sign = bit(flag);
            let mut local = args.clone();
            local.extend(guards(sign.clone()));
            let selected = if flag { n.clone() } else { u.clone() };
            let gap = if flag {
                l.clone()
            } else {
                add(c.clone(), one())
            };
            let abs = Equality::natural(
                choose(sign.clone(), n.clone(), u.clone(), nat()),
                selected.clone(),
                refl(nat(), selected.clone()),
            );
            let gap = Equality::natural(
                choose(sign.clone(), l.clone(), add(c.clone(), one()), nat()),
                gap.clone(),
                refl(nat(), gap),
            );
            let first = abs
                .clone()
                .on_left(gap.left.clone())
                .then(gap.on_right(abs.right))?;
            let split = Equality::natural(
                u.clone(),
                add(l.clone(), mul(h.clone(), bit_value(sign.clone()))),
                var("_split"),
            )
            .then(mul_right(h.clone(), bit_literal(flag)).on_right(l.clone()))?;
            let target = if flag {
                add(l.clone(), h.clone())
            } else {
                l.clone()
            };
            let tail = self.polynomial(&local, split.right.clone(), target)?;
            let split = split.then(tail)?;
            let proof = if flag {
                let pair = add(n.clone(), l.clone());
                let e = self
                    .polynomial(
                        &local,
                        add(pair.clone(), h.clone()),
                        add(add(l.clone(), h.clone()), n.clone()),
                    )?
                    .then(split.reverse().on_left(n.clone()))?
                    .then(Equality::natural(
                        add(u.clone(), n.clone()),
                        m.clone(),
                        app(var("_negative"), vec![refl(boolean(), bit(true))]),
                    ))?
                    .then(
                        Equality::natural(add(h.clone(), h.clone()), m.clone(), var("_double"))
                            .reverse(),
                    )?;
                let cancelled = Equality::natural(
                    pair.clone(),
                    h.clone(),
                    nc("AddCancel", vec![pair, h.clone(), h.clone(), e.proof]),
                );
                first.then(cancelled)?
            } else {
                let e = first.then(split.on_left(add(c.clone(), one())))?;
                let tail = self.polynomial(
                    &local,
                    e.right.clone(),
                    add(add(l.clone(), c.clone()), one()),
                )?;
                e.then(tail)?.then(Equality::natural(
                    add(add(l.clone(), c.clone()), one()),
                    h.clone(),
                    var("_complement"),
                ))?
            };
            let mut body = proof.proof;
            for (name, ty) in guards(sign).into_iter().rev() {
                body = lam(name, ty, body);
            }
            Ok(body)
        };
        let proof = boolean_cases(&["sign"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "ScalarMagnitudeGapBalance",
            &args,
            pi(
                "sign",
                boolean(),
                prop(&[("sign", var("sign"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
}
