//! Unsigned comparisons follow from actual subtraction and its result bound.
use super::*;

fn conserve(a: Expr, b: Expr, r: Expr, m: Expr, c: Expr) -> Expr {
    eq(
        nat(),
        add(a, m.clone()),
        add(add(r, b), mul(m, bit_value(c))),
    )
}

impl Order<'_> {
    pub(super) fn compare_laws(&mut self) -> Result<()> {
        let (a, b, nb, r, m, c) = (
            var("a"),
            var("b"),
            var("not_b"),
            var("r"),
            var("modulus"),
            var("carry"),
        );
        let cmp = add(add(b.clone(), nb.clone()), one());
        let adder = add(add(a.clone(), nb.clone()), one());
        let result = add(r.clone(), mul(m.clone(), c.clone()));
        let mut args: Vec<Binder> = ["a", "b", "not_b", "r", "modulus", "carry"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect();
        args.extend([
            ("complement", eq(nat(), cmp.clone(), m.clone())),
            ("addition", eq(nat(), adder.clone(), result.clone())),
        ]);
        let e = Equality::natural(cmp, m.clone(), var("complement"))
            .reverse()
            .on_right(a.clone());
        let next = self.polynomial(&args, e.right.clone(), add(adder.clone(), b.clone()))?;
        let e = e
            .then(next)?
            .then(Equality::natural(adder, result, var("addition")).on_left(b.clone()))?;
        let next = self.polynomial(&args, e.right.clone(), add(add(r, b.clone()), mul(m, c)))?;
        self.theorem("SubtractionFromComplement", &args, e.then(next)?)?;
        self.subtraction_order()
    }

    fn subtraction_order(&mut self) -> Result<()> {
        let (a, b, r, m) = (var("a"), var("b"), var("r"), var("modulus"));
        let bound = eq(boolean(), less(r.clone(), m.clone()), bit(true));
        let property = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_bound",
                bound.clone(),
                pi(
                    "_conservation",
                    conserve(
                        a.clone(),
                        b.clone(),
                        r.clone(),
                        m.clone(),
                        env["carry"].clone(),
                    ),
                    eq(
                        boolean(),
                        bc("Not", vec![env["carry"].clone()]),
                        less(a.clone(), b.clone()),
                    ),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let c = env["carry"] == bit(true);
            let sum = add(r.clone(), b.clone());
            let e = Equality::natural(
                add(a.clone(), m.clone()),
                add(sum.clone(), mul(m.clone(), bit_value(bit(c)))),
                var("_conservation"),
            )
            .then(mul_right(m.clone(), bit_literal(c)).on_right(sum.clone()))?
            .then(
                Equality::natural(
                    mul(m.clone(), if c { one() } else { zero() }),
                    if c { m.clone() } else { zero() },
                    nc(if c { "MulOne" } else { "MulZero" }, vec![m.clone()]),
                )
                .on_right(sum.clone()),
            )?;
            let natural = if c {
                let cancelled = Equality::natural(
                    a.clone(),
                    sum.clone(),
                    nc(
                        "AddCancel",
                        vec![a.clone(), sum.clone(), m.clone(), e.proof],
                    ),
                );
                less_left(cancelled, b.clone())
                    .then(less_right(
                        sum.clone(),
                        Equality::natural(
                            add(zero(), b.clone()),
                            b.clone(),
                            nc("ZeroAdd", vec![b.clone()]),
                        )
                        .reverse(),
                    ))?
                    .then(Equality::boolean(
                        less(sum, add(zero(), b.clone())),
                        less(r.clone(), zero()),
                        nc("LessTranslate", vec![r.clone(), zero(), b.clone()]),
                    ))?
                    .then(Equality::boolean(
                        less(r.clone(), zero()),
                        bit(false),
                        nc("LessZero", vec![r.clone()]),
                    ))?
            } else {
                let e = e.then(Equality::natural(
                    add(sum.clone(), zero()),
                    sum.clone(),
                    nc("AddZero", vec![sum.clone()]),
                ))?;
                Equality::boolean(
                    less(add(a.clone(), m.clone()), add(b.clone(), m.clone())),
                    less(a.clone(), b.clone()),
                    nc("LessTranslate", vec![a.clone(), b.clone(), m.clone()]),
                )
                .reverse()
                .then(less_left(e, add(b.clone(), m.clone())))?
                .then(less_right(
                    sum.clone(),
                    Equality::natural(
                        add(b.clone(), m.clone()),
                        add(m.clone(), b.clone()),
                        nc("AddComm", vec![b.clone(), m.clone()]),
                    ),
                ))?
                .then(Equality::boolean(
                    less(sum, add(m.clone(), b.clone())),
                    less(r.clone(), m.clone()),
                    nc("LessTranslate", vec![r.clone(), m.clone(), b.clone()]),
                ))?
                .then(Equality::boolean(
                    less(r.clone(), m.clone()),
                    bit(true),
                    var("_bound"),
                ))?
            };
            let not = Equality::boolean(bc("Not", vec![bit(c)]), bit(!c), refl(boolean(), bit(!c)));
            let e = not.then(natural.reverse())?;
            Ok(lam(
                "_bound",
                bound.clone(),
                lam(
                    "_conservation",
                    conserve(a.clone(), b.clone(), r.clone(), m.clone(), bit(c)),
                    e.proof,
                ),
            ))
        };
        let proof = boolean_cases(&["carry"], &BTreeMap::new(), &property, &mut leaf)?;
        let ty = pi(
            "carry",
            boolean(),
            property(&[("carry", var("carry"))].into_iter().collect()),
        );
        self.emit(
            "SubtractionOrder",
            &[("a", nat()), ("b", nat()), ("r", nat()), ("modulus", nat())],
            ty,
            proof,
            true,
        )
    }

    pub(super) fn compare_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(width)?), ("y", word_type(width)?)];
        let ny = wc(width, "Not", vec![y.clone()]);
        let c = wc(
            width,
            &format!("Carry{width}"),
            vec![x.clone(), ny.clone(), bit(true)],
        );
        let raw = wc(width, "AddCarry", vec![x.clone(), ny.clone(), bit(true)]);
        let subword = wc(width, "Sub", vec![x.clone(), y.clone()]);
        let sum = add(value(d, x.clone()), value(d, ny.clone()));
        let p = bit_literal(true)
            .on_right(sum.clone())
            .reverse()
            .then(instance_numeric(
                &format!("W{width}.Block{d}At0"),
                vec![x.clone(), ny.clone(), bit(true)],
                add(sum, bit_value(bit(true))),
                add(value(d, raw.clone()), mul(weight(d), bit_value(c.clone()))),
            ))?;
        let word = Equality {
            ty: word_type(width)?,
            left: subword.clone(),
            right: raw.clone(),
            proof: refl(word_type(width)?, raw),
        };
        let p = p.then(
            map_value(d, word)
                .reverse()
                .on_left(mul(weight(d), bit_value(c.clone()))),
        )?;
        let lhs = add(value(d, x.clone()), weight(d));
        let rhs = add(
            add(value(d, subword.clone()), value(d, y.clone())),
            mul(weight(d), bit_value(c.clone())),
        );
        let e = Equality::natural(
            lhs.clone(),
            rhs.clone(),
            oc(
                "SubtractionFromComplement",
                vec![
                    value(d, x.clone()),
                    value(d, y.clone()),
                    value(d, ny),
                    value(d, subword.clone()),
                    weight(d),
                    bit_value(c.clone()),
                    oc(&format!("W{width}.Complement"), vec![y.clone()]),
                    p.proof,
                ],
            ),
        );
        self.theorem(&format!("W{width}.SubtractionConservation"), &args, e)?;
        let proof = oc(
            "SubtractionOrder",
            vec![
                value(d, x.clone()),
                value(d, y.clone()),
                value(d, subword.clone()),
                weight(d),
                c.clone(),
                oc(&format!("W{width}.ValueBound"), vec![subword]),
                oc(
                    &format!("W{width}.SubtractionConservation"),
                    vec![x.clone(), y.clone()],
                ),
            ],
        );
        let actual = wc(width, "UnsignedLess", vec![x.clone(), y.clone()]);
        let nl = less(value(d, x.clone()), value(d, y.clone()));
        let e = Equality::boolean(
            actual,
            bc("Not", vec![c.clone()]),
            refl(boolean(), bc("Not", vec![c])),
        )
        .then(Equality::boolean(
            bc(
                "Not",
                vec![wc(
                    width,
                    &format!("Carry{width}"),
                    vec![x.clone(), wc(width, "Not", vec![y.clone()]), bit(true)],
                )],
            ),
            nl,
            proof,
        ))?;
        self.theorem(&format!("W{width}.UnsignedLessNatural"), &args, e)?;
        let reversed = Equality::boolean(
            wc(width, "UnsignedLess", vec![y.clone(), x.clone()]),
            less(value(d, y.clone()), value(d, x.clone())),
            oc(
                &format!("W{width}.UnsignedLessNatural"),
                vec![y.clone(), x.clone()],
            ),
        );
        let target = nc("LessEqual", vec![value(d, x.clone()), value(d, y.clone())]);
        let e = Equality::boolean(
            wc(width, "UnsignedLessEqual", vec![x.clone(), y.clone()]),
            bc("Not", vec![reversed.left.clone()]),
            refl(boolean(), bc("Not", vec![reversed.left.clone()])),
        )
        .then(bool_map("Not", reversed))?;
        self.theorem(
            &format!("W{width}.UnsignedLessEqualNatural"),
            &args,
            Equality { right: target, ..e },
        )
    }
}

fn instance_numeric(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, numeric(name, args))
}
