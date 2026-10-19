//! Unbounded zero reflection, proved by Nat induction and local Bool cases.
use super::*;
impl Shifts<'_> {
    pub(super) fn count_reflection_laws(&mut self) -> Result<()> {
        self.sum_zero()?;
        self.positive_zero()?;
        self.weighted_zero()?;
        let (a, b, fa, fb) = (var("a"), var("b"), var("first_flag"), var("second_flag"));
        let left = bc(
            "And",
            vec![bc("Not", vec![fa.clone()]), bc("Not", vec![fb.clone()])],
        );
        let right = bc("Not", vec![bc("Or", vec![fa.clone(), fb.clone()])]);
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                bc(
                    "And",
                    vec![
                        bc("Not", vec![e["first_flag"].clone()]),
                        bc("Not", vec![e["second_flag"].clone()]),
                    ],
                ),
                bc(
                    "Not",
                    vec![bc(
                        "Or",
                        vec![e["first_flag"].clone(), e["second_flag"].clone()],
                    )],
                ),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| {
            Ok(refl(
                boolean(),
                bit(e["first_flag"] == bit(false) && e["second_flag"] == bit(false)),
            ))
        };
        self.theorem(
            "CountDeMorgan",
            &[("first_flag", boolean()), ("second_flag", boolean())],
            Equality::boolean(
                left.clone(),
                right.clone(),
                boolean_cases(
                    &["first_flag", "second_flag"],
                    &BTreeMap::new(),
                    &prop,
                    &mut leaf,
                )?,
            ),
        )?;
        let args = [
            ("a", nat()),
            ("b", nat()),
            ("first_flag", boolean()),
            ("second_flag", boolean()),
            (
                "first",
                eq(
                    boolean(),
                    equal(a.clone(), zero()),
                    bc("Not", vec![fa.clone()]),
                ),
            ),
            (
                "second",
                eq(
                    boolean(),
                    equal(b.clone(), zero()),
                    bc("Not", vec![fb.clone()]),
                ),
            ),
        ];
        let e = Equality::boolean(
            equal(add(a.clone(), b.clone()), zero()),
            bc(
                "And",
                vec![equal(a.clone(), zero()), equal(b.clone(), zero())],
            ),
            sh("CountSumZero", vec![a.clone(), b.clone()]),
        );
        let e = e
            .then(bool_left(
                "And",
                Equality::boolean(equal(a, zero()), bc("Not", vec![fa.clone()]), var("first")),
                equal(b.clone(), zero()),
            ))?
            .then(bool_right(
                "And",
                bc("Not", vec![fa.clone()]),
                Equality::boolean(equal(b, zero()), bc("Not", vec![fb.clone()]), var("second")),
            ))?
            .then(Equality::boolean(
                left,
                right,
                sh("CountDeMorgan", vec![fa, fb]),
            ))?;
        self.theorem("CountPairZero", &args, e)
    }
    fn sum_zero(&mut self) -> Result<()> {
        let (a, b, n) = (var("a"), var("b"), var("_index"));
        let property = |b: Expr| {
            eq(
                boolean(),
                equal(add(a.clone(), b.clone()), zero()),
                bc("And", vec![equal(a.clone(), zero()), equal(b, zero())]),
            )
        };
        let base = equal_left(
            law(
                "AddZero",
                vec![a.clone()],
                add(a.clone(), zero()),
                a.clone(),
            ),
            zero(),
        );
        let right = bool_right(
            "And",
            equal(a.clone(), zero()),
            Equality::boolean(
                equal(zero(), zero()),
                bit(true),
                nc("EqualZeroZero", vec![]),
            ),
        )
        .then(Equality::boolean(
            bc("And", vec![equal(a.clone(), zero()), bit(true)]),
            equal(a.clone(), zero()),
            refl(boolean(), equal(a.clone(), zero())),
        ))?;
        let base = base.then(right.reverse())?;
        let step = equal_left(
            law(
                "AddSucc",
                vec![a.clone(), n.clone()],
                add(a.clone(), succ(n.clone())),
                succ(add(a.clone(), n.clone())),
            ),
            zero(),
        )
        .then(Equality::boolean(
            equal(succ(add(a.clone(), n.clone())), zero()),
            bit(false),
            nc("EqualSuccZero", vec![add(a.clone(), n.clone())]),
        ))?;
        let right = bool_right(
            "And",
            equal(a.clone(), zero()),
            Equality::boolean(
                equal(succ(n.clone()), zero()),
                bit(false),
                nc("EqualSuccZero", vec![n.clone()]),
            ),
        )
        .then(Equality::boolean(
            bc("And", vec![equal(a.clone(), zero()), bit(false)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        let step = step.then(right.reverse())?;
        self.theorem(
            "CountSumZero",
            &[("a", nat()), ("b", nat())],
            Equality::boolean(
                equal(add(a.clone(), b.clone()), zero()),
                bc(
                    "And",
                    vec![equal(a.clone(), zero()), equal(b.clone(), zero())],
                ),
                induct(
                    "_index",
                    property(n.clone()),
                    base.proof,
                    lam("_index", nat(), lam("_ignored", property(n), step.proof)),
                    b,
                ),
            ),
        )
    }
    fn positive_zero(&mut self) -> Result<()> {
        let (n, i) = (var("n"), var("_index"));
        let property = |n: Expr| {
            eq(
                boolean(),
                bc("Not", vec![equal(n.clone(), zero())]),
                less(zero(), n),
            )
        };
        let make = |succ_: bool| -> Result<Equality> {
            let n = if succ_ { succ(i.clone()) } else { zero() };
            let eq0 = Equality::boolean(
                equal(n.clone(), zero()),
                bit(!succ_),
                nc(
                    if succ_ {
                        "EqualSuccZero"
                    } else {
                        "EqualZeroZero"
                    },
                    if succ_ { vec![i.clone()] } else { vec![] },
                ),
            );
            let e = bool_map("Not", eq0).then(Equality::boolean(
                bc("Not", vec![bit(!succ_)]),
                bit(succ_),
                refl(boolean(), bit(succ_)),
            ))?;
            e.then(
                Equality::boolean(
                    less(zero(), n),
                    bit(succ_),
                    nc(
                        if succ_ { "LessZeroSucc" } else { "LessZero" },
                        if succ_ { vec![i.clone()] } else { vec![zero()] },
                    ),
                )
                .reverse(),
            )
        };
        let (base, step) = (make(false)?, make(true)?);
        self.theorem(
            "CountNonzeroPositive",
            &[("n", nat())],
            Equality::boolean(
                bc("Not", vec![equal(n.clone(), zero())]),
                less(zero(), n.clone()),
                induct(
                    "_index",
                    property(i.clone()),
                    base.proof,
                    lam("_index", nat(), lam("_ignored", property(i), step.proof)),
                    n,
                ),
            ),
        )
    }
    fn weighted_zero(&mut self) -> Result<()> {
        let (p, b) = (var("power"), var("bit"));
        let args = [
            ("power", nat()),
            ("bit", boolean()),
            (
                "positive",
                eq(boolean(), less(zero(), p.clone()), bit(true)),
            ),
        ];
        let nonzero = Equality::boolean(
            bc("Not", vec![equal(p.clone(), zero())]),
            less(zero(), p.clone()),
            sh("CountNonzeroPositive", vec![p.clone()]),
        )
        .then(Equality::boolean(
            less(zero(), p.clone()),
            bit(true),
            var("positive"),
        ))?;
        let flag = equal(p.clone(), zero());
        let notzero = Equality::boolean(
            bc("Not", vec![bc("Not", vec![flag.clone()])]),
            flag.clone(),
            bc("DoubleNot", vec![flag]),
        )
        .reverse()
        .then(bool_map("Not", nonzero))?
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        let property = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                equal(mul(p.clone(), bit_value(e["bit"].clone())), zero()),
                bc("Not", vec![e["bit"].clone()]),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["bit"] == bit(true);
            let product = mul_right(p.clone(), bit_literal(yes)).then(law(
                if yes { "MulOne" } else { "MulZero" },
                vec![p.clone()],
                mul(p.clone(), if yes { one() } else { zero() }),
                if yes { p.clone() } else { zero() },
            ))?;
            let end = if yes {
                notzero.clone()
            } else {
                Equality::boolean(
                    equal(zero(), zero()),
                    bit(true),
                    nc("EqualZeroZero", vec![]),
                )
            };
            let out = equal_left(product, zero()).then(end)?;
            Ok(out
                .then(
                    Equality::boolean(
                        bc("Not", vec![bit(yes)]),
                        bit(!yes),
                        refl(boolean(), bit(!yes)),
                    )
                    .reverse(),
                )?
                .proof)
        };
        self.theorem(
            "WeightedBitZero",
            &args,
            Equality::boolean(
                equal(mul(p.clone(), bit_value(b.clone())), zero()),
                bc("Not", vec![b]),
                boolean_cases(&["bit"], &BTreeMap::new(), &property, &mut leaf)?,
            ),
        )
    }
}
