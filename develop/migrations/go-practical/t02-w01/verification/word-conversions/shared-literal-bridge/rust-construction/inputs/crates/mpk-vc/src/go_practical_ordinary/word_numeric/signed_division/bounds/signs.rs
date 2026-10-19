//! Bounded actual words expose their half equality and negation/nonzero sign.
use super::*;

impl Bounded<'_> {
    pub(super) fn signs(&mut self) -> Result<()> {
        let (u, n, g, h, m) = (var("u"), var("n"), var("gap"), var("half"), var("modulus"));
        let args = [
            ("u", nat()),
            ("n", nat()),
            ("gap", nat()),
            ("half", nat()),
            ("modulus", nat()),
            (
                "positive",
                eq(boolean(), less(zero(), h.clone()), bit(true)),
            ),
            ("balance", eq(nat(), add(u.clone(), g.clone()), h.clone())),
            ("double", eq(nat(), add(h.clone(), h.clone()), m.clone())),
        ];
        let guards = |sign: Expr, nonzero: Expr| -> Vec<Binder> {
            vec![
                (
                    "_sum",
                    eq(
                        nat(),
                        add(u.clone(), n.clone()),
                        mul(m.clone(), bit_value(nonzero)),
                    ),
                ),
                (
                    "_threshold",
                    eq(boolean(), less(n.clone(), h.clone()), bc("Not", vec![sign])),
                ),
            ]
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let s = env["sign"].clone();
            let z = env["nonzero"].clone();
            let mut out = eq(boolean(), s.clone(), z.clone());
            for (name, ty) in guards(s, z).into_iter().rev() {
                out = pi(name, ty, out);
            }
            out
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let s = env["sign"] == bit(true);
            let z = env["nonzero"] == bit(true);
            let proof = if s == z {
                refl(boolean(), bit(s))
            } else {
                let sum = Equality::natural(
                    add(u.clone(), n.clone()),
                    mul(m.clone(), bit_value(bit(z))),
                    var("_sum"),
                )
                .then(mul_right(m.clone(), bit_literal(z)))?
                .then(nlaw(
                    if z { "MulOne" } else { "MulZero" },
                    vec![m.clone()],
                    mul(m.clone(), if z { one() } else { zero() }),
                    if z { m.clone() } else { zero() },
                ))?;
                let threshold = Equality::boolean(
                    less(n.clone(), h.clone()),
                    bc("Not", vec![bit(s)]),
                    var("_threshold"),
                )
                .then(Equality::boolean(
                    bc("Not", vec![bit(s)]),
                    bit(!s),
                    refl(boolean(), bit(!s)),
                ))?;
                let contradiction = if !z {
                    let nz = zero_sum(true, u.clone(), n.clone(), sum.proof);
                    let positive = less_left(nz, h.clone()).then(Equality::boolean(
                        less(zero(), h.clone()),
                        bit(true),
                        var("positive"),
                    ))?;
                    threshold.reverse().then(positive)?
                } else {
                    let translated = Equality::boolean(
                        less(add(n.clone(), u.clone()), add(h.clone(), u.clone())),
                        less(n.clone(), h.clone()),
                        nc("LessTranslate", vec![n.clone(), h.clone(), u.clone()]),
                    )
                    .then(threshold)?;
                    let upper = nlaw(
                        "AddAssoc",
                        vec![h.clone(), u.clone(), g.clone()],
                        add(add(h.clone(), u.clone()), g.clone()),
                        add(h.clone(), add(u.clone(), g.clone())),
                    )
                    .then(
                        Equality::natural(add(u.clone(), g.clone()), h.clone(), var("balance"))
                            .on_right(h.clone()),
                    )?
                    .then(Equality::natural(
                        add(h.clone(), h.clone()),
                        m.clone(),
                        var("double"),
                    ))?;
                    let bound = Equality::boolean(
                        less(add(n.clone(), u.clone()), m.clone()),
                        bit(true),
                        db(
                            "StrictUpperFromBalance",
                            vec![
                                add(n.clone(), u.clone()),
                                add(h.clone(), u.clone()),
                                g.clone(),
                                m.clone(),
                                translated.proof,
                                upper.proof,
                            ],
                        ),
                    );
                    let total = nlaw(
                        "AddComm",
                        vec![n.clone(), u.clone()],
                        add(n.clone(), u.clone()),
                        add(u.clone(), n.clone()),
                    )
                    .then(sum)?;
                    let bad = less_left(total, m.clone()).reverse().then(bound)?;
                    Equality::boolean(
                        less(m.clone(), m.clone()),
                        bit(false),
                        nc("LessIrreflexive", vec![m.clone()]),
                    )
                    .reverse()
                    .then(bad)?
                };
                bc(
                    "FalseElim",
                    vec![eq(boolean(), bit(s), bit(z)), contradiction.proof],
                )
            };
            let mut body = proof;
            for (name, ty) in guards(bit(s), bit(z)).into_iter().rev() {
                body = lam(name, ty, body);
            }
            Ok(body)
        };
        let proof = boolean_cases(&["sign", "nonzero"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "NegationSignFromHalfGap",
            &args,
            pi(
                "sign",
                boolean(),
                pi(
                    "nonzero",
                    boolean(),
                    prop(
                        &[("sign", var("sign")), ("nonzero", var("nonzero"))]
                            .into_iter()
                            .collect(),
                    ),
                ),
            ),
            proof,
            true,
        )
    }
    pub(super) fn bounded_word(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("word");
        let g = var("gap");
        let h = half(d);
        let u = value(d, x.clone());
        let s = sign(w, x.clone())?;
        let args = [
            ("word", word_type(w)?),
            ("gap", nat()),
            ("balance", eq(nat(), add(u.clone(), g.clone()), h.clone())),
        ];
        let balance = Equality::natural(add(u.clone(), g.clone()), h.clone(), var("balance"));
        let at_most = balance
            .clone()
            .reverse()
            .mapped(
                lam("_half", nat(), le(u.clone(), var("_half"))),
                le(u.clone(), h.clone()),
                le(u.clone(), add(u.clone(), g.clone())),
                boolean(),
            )
            .then(Equality::boolean(
                le(u.clone(), add(u.clone(), g.clone())),
                bit(true),
                prerequisite("Bounds", "GapLessEqual", vec![u.clone(), g.clone()]),
            ))?;
        self.theorem(&format!("W{w}.WordHalfBound"), &args, at_most.clone())?;
        let threshold = Equality::boolean(
            less(u.clone(), h.clone()),
            bc("Not", vec![s.clone()]),
            prerequisite("Bounds", &format!("W{w}.UnsignedHalfLess"), vec![x.clone()]),
        );
        let same = bnot(threshold)
            .then(Equality::boolean(
                bc("Not", vec![bc("Not", vec![s.clone()])]),
                s.clone(),
                bc("DoubleNot", vec![s.clone()]),
            ))?
            .reverse()
            .then(Equality::boolean(
                bc("Not", vec![less(u.clone(), h.clone())]),
                equal(u.clone(), h.clone()),
                prerequisite(
                    "Bounds",
                    "AtMostThresholdEquals",
                    vec![u.clone(), h.clone(), at_most.proof],
                ),
            ))?;
        self.theorem(&format!("W{w}.WordSignIsHalf"), &args, same)?;
        let nx = wc(w, "Neg", vec![x.clone()]);
        let n = value(d, nx.clone());
        let ns = sign(w, nx.clone())?;
        let nz = wc(w, "DivideDefined", vec![x.clone()]);
        self.theorem(
            &format!("W{w}.NegationSignIsNonzero"),
            &args,
            Equality::boolean(
                ns.clone(),
                nz.clone(),
                db(
                    "NegationSignFromHalfGap",
                    vec![
                        u,
                        n,
                        g,
                        h.clone(),
                        weight(d),
                        prerequisite("Bounds", &format!("HalfPositive{d}"), vec![]),
                        var("balance"),
                        prerequisite("Bounds", &format!("HalfDouble{d}"), vec![]),
                        ns,
                        nz,
                        prerequisite("Magnitude", &format!("W{w}.NegationValueSum"), vec![x]),
                        prerequisite("Bounds", &format!("W{w}.UnsignedHalfLess"), vec![nx]),
                    ],
                ),
            ),
        )
    }
}
