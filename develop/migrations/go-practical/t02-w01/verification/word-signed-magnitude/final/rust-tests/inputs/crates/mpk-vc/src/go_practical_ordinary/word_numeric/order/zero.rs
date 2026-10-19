//! Source division definedness reflects actual nonzero natural word value.
use super::*;

fn zero_cube(d: usize) -> Expr {
    let mut body = bit(false);
    for name in ADDRESSES[..d].iter().rev() {
        body = lam(name, boolean(), body);
    }
    body
}

impl Order<'_> {
    pub(super) fn zero_values(&mut self) -> Result<()> {
        for d in 0..=6 {
            let x = zero_cube(d);
            let start = value_definition(d, x.clone());
            let e = if d == 0 {
                start.then(bit_literal(false))?
            } else {
                let h = d - 1;
                let pick = |high| -> Result<Equality> {
                    let z = zero_cube(h);
                    map_value(
                        h,
                        Equality {
                            ty: cube(h),
                            left: project(x.clone(), high),
                            right: z.clone(),
                            proof: refl(cube(h), z.clone()),
                        },
                    )
                    .then(instance(
                        &format!("ZeroValue{h}"),
                        vec![],
                        value(h, z),
                        zero(),
                    ))
                };
                let low = pick(false)?;
                let high = pick(true)?;
                start
                    .then(low.on_left(mul(weight(h), high.left.clone())))?
                    .then(mul_right(weight(h), high).on_right(zero()))?
                    .then(
                        Equality::natural(
                            mul(weight(h), zero()),
                            zero(),
                            nc("MulZero", vec![weight(h)]),
                        )
                        .on_right(zero()),
                    )?
                    .then(Equality::natural(
                        add(zero(), zero()),
                        zero(),
                        nc("ZeroAdd", vec![zero()]),
                    ))?
            };
            self.theorem(&format!("ZeroValue{d}"), &[], e)?;
        }
        Ok(())
    }
    pub(super) fn zero_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, z) = (var("x"), wc(width, "Zero", vec![]));
        let names = &ADDRESSES[..d];
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                app(z.clone(), names.iter().map(|n| env[n].clone()).collect()),
                bit(false),
            )
        };
        let mut leaf =
            |_: &BTreeMap<&'static str, Expr>| -> Result<Expr> { Ok(refl(boolean(), bit(false))) };
        let proof = boolean_cases(names, &BTreeMap::new(), &prop, &mut leaf)?;
        let mut ty = prop(&names.iter().map(|n| (*n, var(n))).collect());
        for name in names.iter().rev() {
            ty = pi(name, boolean(), ty);
        }
        self.emit(&format!("W{width}.ZeroPointwise"), &[], ty, proof, true)?;
        let e = Equality::natural(
            value(d, z.clone()),
            value(d, zero_cube(d)),
            numeric(
                &format!("ValueCongruence{d}"),
                vec![
                    z.clone(),
                    zero_cube(d),
                    oc(&format!("W{width}.ZeroPointwise"), vec![]),
                ],
            ),
        )
        .then(instance(
            &format!("ZeroValue{d}"),
            vec![],
            value(d, zero_cube(d)),
            zero(),
        ))?;
        self.theorem(&format!("W{width}.ZeroValue"), &[], e.clone())?;
        let vx = value(d, x.clone());
        let vz = value(d, z.clone());
        let mapped = e.mapped(
            lam("_zero", nat(), equal(vx.clone(), var("_zero"))),
            equal(vx.clone(), vz),
            equal(vx.clone(), zero()),
            boolean(),
        );
        let actual_eq = wc(width, "Equal", vec![x.clone(), z.clone()]);
        let e = Equality::boolean(
            wc(width, "IsZero", vec![x.clone()]),
            actual_eq.clone(),
            refl(boolean(), actual_eq),
        )
        .then(Equality::boolean(
            wc(width, "Equal", vec![x.clone(), z]),
            equal(vx.clone(), value(d, wc(width, "Zero", vec![]))),
            oc(
                &format!("W{width}.EqualNatural"),
                vec![x.clone(), wc(width, "Zero", vec![])],
            ),
        ))?
        .then(mapped)?;
        self.theorem(
            &format!("W{width}.IsZeroNatural"),
            &[("x", word_type(width)?)],
            e.clone(),
        )?;
        let positive = bool_map("Not", e).then(Equality::boolean(
            bc("Not", vec![equal(vx.clone(), zero())]),
            less(zero(), vx.clone()),
            oc("NaturalNonzeroPositive", vec![vx.clone()]),
        ))?;
        let positive = Equality {
            left: wc(width, "DivideDefined", vec![x.clone()]),
            ..positive
        };
        self.theorem(
            &format!("W{width}.DivideDefinedNatural"),
            &[("x", word_type(width)?)],
            positive.clone(),
        )?;
        let guard = eq(
            boolean(),
            wc(width, "DivideDefined", vec![x.clone()]),
            bit(true),
        );
        let e = positive.reverse().then(Equality::boolean(
            wc(width, "DivideDefined", vec![x.clone()]),
            bit(true),
            var("defined"),
        ))?;
        self.theorem(
            &format!("W{width}.DefinedDivisorPositive"),
            &[("x", word_type(width)?), ("defined", guard)],
            e,
        )
    }
}
