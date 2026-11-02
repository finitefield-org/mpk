//! Reusable balanced Bool-cube zero reflection and sign consequences.
use super::*;
impl Zero<'_> {
    pub(super) fn cube(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        let args = [("x", cube(d))];
        let body = if d == 0 {
            bc("Not", vec![x.clone()])
        } else {
            bc(
                "And",
                vec![
                    bits(d - 1, project(x.clone(), false)),
                    bits(d - 1, project(x.clone(), true)),
                ],
            )
        };
        self.emit(
            &format!("ZeroBits{d}"),
            &args,
            boolean(),
            body.clone(),
            false,
        )?;
        self.theorem(
            &format!("ZeroBitsDefinition{d}"),
            &args,
            Equality::boolean(
                bits(d, x.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        self.weight_positive(d)?;
        let literal_proof = if d == 0 {
            Equality::boolean(
                bits(d, x.clone()),
                literal(d, x.clone()),
                refl(boolean(), literal(d, x.clone())),
            )
        } else {
            let h = d - 1;
            let (lo, hi) = (project(x.clone(), false), project(x.clone(), true));
            let l = Equality::boolean(
                bits(h, lo.clone()),
                literal(h, lo.clone()),
                zc(&format!("ZeroBitsLiteral{h}"), vec![lo]),
            );
            let r = Equality::boolean(
                bits(h, hi.clone()),
                literal(h, hi.clone()),
                zc(&format!("ZeroBitsLiteral{h}"), vec![hi]),
            );
            let e = Equality::boolean(bits(d, x.clone()), body.clone(), refl(boolean(), body))
                .then(bool_left("And", l.clone(), r.left.clone()))?
                .then(bool_right("And", l.right, r))?;
            let end = literal(d, x.clone());
            let tail = Equality::boolean(e.right.clone(), end.clone(), refl(boolean(), end));
            e.then(tail)?
        };
        self.theorem(&format!("ZeroBitsLiteral{d}"), &args, literal_proof)?;
        self.cube_natural(d)?;
        self.cube_sign(d)
    }
    fn weight_positive(&mut self, d: usize) -> Result<()> {
        let mut zero_cube = bit(false);
        for name in ADDRESSES[..d].iter().rev() {
            zero_cube = lam(name, boolean(), zero_cube);
        }
        let sum = add(
            value(d, zero_cube.clone()),
            value(d, order(&format!("NotCube{d}"), vec![zero_cube.clone()])),
        );
        let e = Equality::natural(
            add(sum.clone(), one()),
            weight(d),
            order(&format!("Complement{d}"), vec![zero_cube]),
        )
        .reverse()
        .then(nlaw(
            "AddOne",
            vec![sum.clone()],
            add(sum.clone(), one()),
            succ(sum.clone()),
        ))?;
        self.theorem(
            &format!("WeightPositive{d}"),
            &[],
            less_right(e).then(Equality::boolean(
                less(zero(), succ(sum.clone())),
                bit(true),
                nc("LessZeroSucc", vec![sum]),
            ))?,
        )
    }
    fn cube_natural(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        if d == 0 {
            let prop = |env: &BTreeMap<&'static str, Expr>| {
                eq(
                    boolean(),
                    bits(0, env["x"].clone()),
                    equal(value(0, env["x"].clone()), zero()),
                )
            };
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let b = env["x"] == bit(true);
                let v = value_definition(0, bit(b)).then(bit_literal(b))?;
                let n = if b { one() } else { zero() };
                let equal_value = equal_left(v).then(Equality::boolean(
                    equal(n, zero()),
                    bit(!b),
                    nc(
                        if b { "EqualSuccZero" } else { "EqualZeroZero" },
                        if b { vec![zero()] } else { vec![] },
                    ),
                ))?;
                let z = Equality::boolean(bits(0, bit(b)), bit(!b), refl(boolean(), bit(!b)));
                Ok(z.then(equal_value.reverse())?.proof)
            };
            let proof = boolean_cases(&["x"], &BTreeMap::new(), &prop, &mut leaf)?;
            return self.emit(
                "ZeroBitsNatural0",
                &[],
                pi("x", boolean(), prop(&[("x", x)].into_iter().collect())),
                proof,
                true,
            );
        }
        let h = d - 1;
        let (lo, hi) = (project(x.clone(), false), project(x.clone(), true));
        let a = value(h, lo.clone());
        let b = value(h, hi.clone());
        let l = Equality::boolean(
            bits(h, lo.clone()),
            equal(a.clone(), zero()),
            zc(&format!("ZeroBitsNatural{h}"), vec![lo]),
        );
        let r = Equality::boolean(
            bits(h, hi.clone()),
            equal(b.clone(), zero()),
            zc(&format!("ZeroBitsNatural{h}"), vec![hi]),
        );
        let zdef = bc("And", vec![l.left.clone(), r.left.clone()]);
        let e = Equality::boolean(bits(d, x.clone()), zdef.clone(), refl(boolean(), zdef))
            .then(bool_left("And", l.clone(), r.left.clone()))?
            .then(bool_right("And", l.right.clone(), r))?;
        let product = Equality::boolean(
            equal(mul(weight(h), b.clone()), zero()),
            equal(b.clone(), zero()),
            zc(
                "PositiveProductZero",
                vec![
                    weight(h),
                    b.clone(),
                    zc(&format!("WeightPositive{h}"), vec![]),
                ],
            ),
        );
        let e = e
            .then(bool_right("And", l.right, product.reverse()))?
            .then(
                Equality::boolean(
                    equal(add(a.clone(), mul(weight(h), b.clone())), zero()),
                    bc(
                        "And",
                        vec![
                            equal(a.clone(), zero()),
                            equal(mul(weight(h), b.clone()), zero()),
                        ],
                    ),
                    zc("SumZero", vec![a, mul(weight(h), b)]),
                )
                .reverse(),
            )?
            .then(equal_left(value_definition(d, x.clone())).reverse())?;
        self.theorem(&format!("ZeroBitsNatural{d}"), &[("x", cube(d))], e)
    }
    fn cube_sign(&mut self, d: usize) -> Result<()> {
        let x = var("x");
        if d == 0 {
            let prop = |env: &BTreeMap<&'static str, Expr>| {
                pi(
                    "_zero",
                    eq(boolean(), bits(0, env["x"].clone()), bit(true)),
                    eq(boolean(), env["x"].clone(), bit(false)),
                )
            };
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let b = env["x"] == bit(true);
                let proof = if b {
                    bc(
                        "FalseElim",
                        vec![eq(boolean(), bit(true), bit(false)), var("_zero")],
                    )
                } else {
                    refl(boolean(), bit(false))
                };
                Ok(lam(
                    "_zero",
                    eq(boolean(), bits(0, bit(b)), bit(true)),
                    proof,
                ))
            };
            let proof = boolean_cases(&["x"], &BTreeMap::new(), &prop, &mut leaf)?;
            return self.emit(
                "ZeroBitsSign0",
                &[],
                pi("x", boolean(), prop(&[("x", x)].into_iter().collect())),
                proof,
                true,
            );
        }
        let h = d - 1;
        let hi = project(x.clone(), true);
        let right = bc(
            "AndRight",
            vec![
                bits(h, project(x.clone(), false)),
                bits(h, hi.clone()),
                var("zero"),
            ],
        );
        self.theorem(
            &format!("ZeroBitsSign{d}"),
            &[
                ("x", cube(d)),
                ("zero", eq(boolean(), bits(d, x.clone()), bit(true))),
            ],
            Equality::boolean(
                top(d, x),
                bit(false),
                zc(&format!("ZeroBitsSign{h}"), vec![hi, right]),
            ),
        )
    }
}
