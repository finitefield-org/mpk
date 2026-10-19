//! Genuine unbounded Nat zero-sum and positive-multiplier reflection.
use super::*;
impl Zero<'_> {
    pub(super) fn natural_laws(&mut self) -> Result<()> {
        let (a, b, m) = (var("a"), var("b"), var("modulus"));
        for product in [false, true] {
            let name = if product {
                "PositiveProductZero"
            } else {
                "SumZero"
            };
            let left_value = if product {
                mul(m.clone(), b.clone())
            } else {
                add(a.clone(), b.clone())
            };
            let f = equal(left_value.clone(), zero());
            let g = if product {
                equal(b.clone(), zero())
            } else {
                bc(
                    "And",
                    vec![equal(a.clone(), zero()), equal(b.clone(), zero())],
                )
            };
            let equation = Equality::natural(
                left_value.clone(),
                zero(),
                nc(
                    "EqualReflect",
                    vec![left_value.clone(), zero(), var("_valid")],
                ),
            );
            let forward = if product {
                let p = equation.then(
                    nlaw("MulZero", vec![m.clone()], mul(m.clone(), zero()), zero()).reverse(),
                )?;
                let e = shared(
                    "MultiplyCancelPositive",
                    vec![m.clone(), var("positive"), b.clone(), zero(), p.proof],
                );
                nc("EqualComplete", vec![b.clone(), zero(), e])
            } else {
                let az = shared(
                    "ZeroSumLeft",
                    vec![a.clone(), b.clone(), equation.proof.clone()],
                );
                let bz = shared("ZeroSumRight", vec![a.clone(), b.clone(), equation.proof]);
                let ea = Equality::boolean(
                    equal(a.clone(), zero()),
                    bit(true),
                    nc("EqualComplete", vec![a.clone(), zero(), az]),
                );
                let eb = Equality::boolean(
                    equal(b.clone(), zero()),
                    bit(true),
                    nc("EqualComplete", vec![b.clone(), zero(), bz]),
                );
                bool_left("And", ea.clone(), eb.left.clone())
                    .then(bool_right("And", ea.right, eb))?
                    .then(Equality::boolean(
                        bc("And", vec![bit(true), bit(true)]),
                        bit(true),
                        refl(boolean(), bit(true)),
                    ))?
                    .proof
            };
            let beq = |n: Expr, proof: Expr| {
                Equality::natural(
                    n.clone(),
                    zero(),
                    nc("EqualReflect", vec![n, zero(), proof]),
                )
            };
            let backward = if product {
                mul_right(m.clone(), beq(b.clone(), var("_valid"))).then(nlaw(
                    "MulZero",
                    vec![m.clone()],
                    mul(m.clone(), zero()),
                    zero(),
                ))?
            } else {
                let ea = beq(
                    a.clone(),
                    bc(
                        "AndLeft",
                        vec![
                            equal(a.clone(), zero()),
                            equal(b.clone(), zero()),
                            var("_valid"),
                        ],
                    ),
                );
                let eb = beq(
                    b.clone(),
                    bc(
                        "AndRight",
                        vec![
                            equal(a.clone(), zero()),
                            equal(b.clone(), zero()),
                            var("_valid"),
                        ],
                    ),
                );
                ea.clone()
                    .on_left(eb.left.clone())
                    .then(eb.on_right(ea.right))?
                    .then(nlaw("AddZero", vec![zero()], add(zero(), zero()), zero()))?
            };
            let backward = nc("EqualComplete", vec![left_value, zero(), backward.proof]);
            let fproof = lam("_valid", eq(boolean(), f.clone(), bit(true)), forward);
            let bproof = lam("_valid", eq(boolean(), g.clone(), bit(true)), backward);
            let args = if product {
                vec![
                    ("modulus", nat()),
                    ("b", nat()),
                    (
                        "positive",
                        eq(boolean(), less(zero(), m.clone()), bit(true)),
                    ),
                ]
            } else {
                vec![("a", nat()), ("b", nat())]
            };
            self.theorem(
                name,
                &args,
                Equality::boolean(
                    f.clone(),
                    g.clone(),
                    shared("ReflectionAgreement", vec![f, g, fproof, bproof]),
                ),
            )?;
        }
        self.nonzero_positive()
    }
    fn nonzero_positive(&mut self) -> Result<()> {
        let (n, i) = (var("n"), var("_i"));
        let prop = |n: Expr| {
            eq(
                boolean(),
                bc("Not", vec![equal(n.clone(), zero())]),
                less(zero(), n),
            )
        };
        let base = bool_map(
            "Not",
            Equality::boolean(
                equal(zero(), zero()),
                bit(true),
                nc("EqualZeroZero", vec![]),
            ),
        )
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?
        .then(
            Equality::boolean(
                less(zero(), zero()),
                bit(false),
                nc("LessZero", vec![zero()]),
            )
            .reverse(),
        )?;
        let step = bool_map(
            "Not",
            Equality::boolean(
                equal(succ(i.clone()), zero()),
                bit(false),
                nc("EqualSuccZero", vec![i.clone()]),
            ),
        )
        .then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?
        .then(
            Equality::boolean(
                less(zero(), succ(i.clone())),
                bit(true),
                nc("LessZeroSucc", vec![i.clone()]),
            )
            .reverse(),
        )?;
        self.theorem(
            "NaturalNonzeroPositive",
            &[("n", nat())],
            Equality::boolean(
                bc("Not", vec![equal(n.clone(), zero())]),
                less(zero(), n.clone()),
                induct(
                    "_i",
                    prop(i.clone()),
                    base.proof,
                    lam("_i", nat(), lam("_ignored", prop(i), step.proof)),
                    n,
                ),
            ),
        )
    }
}
