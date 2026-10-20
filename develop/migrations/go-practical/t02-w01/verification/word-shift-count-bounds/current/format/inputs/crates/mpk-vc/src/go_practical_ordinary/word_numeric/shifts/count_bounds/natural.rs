//! Unbounded natural count bounds and symbolic positive-tail decomposition.
use super::*;
impl Shifts<'_> {
    pub(super) fn count_bound_natural(&mut self) -> Result<()> {
        let b = var("bit");
        let not = call("Mpk.Go.Practical.Word.Order.NotCube0", vec![b.clone()]);
        let left = value_definition(0, b.clone());
        let right = value_definition(0, not.clone()).then(map_bit(Equality::boolean(
            not,
            bc("Not", vec![b.clone()]),
            refl(boolean(), bc("Not", vec![b.clone()])),
        )))?;
        let bits = add(bit_value(b.clone()), bit_value(bc("Not", vec![b.clone()])));
        let convert = left
            .clone()
            .on_left(right.left.clone())
            .then(right.on_right(left.right))?
            .on_left(one());
        let original_sum = convert.left.clone();
        let e = convert
            .reverse()
            .then(Equality::natural(
                original_sum,
                weight(0),
                call("Mpk.Go.Practical.Word.Order.Complement0", vec![b.clone()]),
            ))?
            .then(weight_definition(0))?;
        self.theorem(
            "CountBitComplement",
            &[("bit", boolean())],
            Equality::natural(
                bits.clone(),
                one(),
                nc("AddCancel", vec![bits, one(), one(), e.proof]),
            ),
        )?;
        self.prefix_step()?;
        self.complement_bound()?;
        self.no_less_extension()?;
        self.positive_decomposition()
    }
    fn prefix_step(&mut self) -> Result<()> {
        let (a, z, p, b, n) = (
            var("low"),
            var("complement"),
            var("power"),
            var("bit_value"),
            var("not_bit_value"),
        );
        let balance = add(add(a.clone(), z.clone()), one());
        let bits = add(b.clone(), n.clone());
        let mut args = ["low", "complement", "power", "bit_value", "not_bit_value"]
            .map(|n| (n, nat()))
            .to_vec();
        args.extend([
            ("balance", eq(nat(), balance.clone(), p.clone())),
            ("bits", eq(nat(), bits.clone(), one())),
        ]);
        let start = add(
            add(add(a, mul(p.clone(), b)), add(z, mul(p.clone(), n))),
            one(),
        );
        let target = add(balance.clone(), mul(p.clone(), bits.clone()));
        let e = self
            .polynomial(&args, start, target)?
            .then(
                Equality::natural(balance, p.clone(), var("balance"))
                    .on_left(mul(p.clone(), bits.clone())),
            )?
            .then(
                mul_right(p.clone(), Equality::natural(bits, one(), var("bits")))
                    .on_right(p.clone()),
            )?
            .then(
                law("MulOne", vec![p.clone()], mul(p.clone(), one()), p.clone())
                    .on_right(p.clone()),
            )?;
        let end = known(
            "TwoMul",
            vec![p.clone()],
            mul(two(), p.clone()),
            add(p.clone(), p),
        );
        self.theorem("CountPrefixComplementStep", &args, e.then(end.reverse())?)
    }
    fn complement_bound(&mut self) -> Result<()> {
        let (a, z, p) = (var("low"), var("complement"), var("power"));
        let balance = add(add(a.clone(), z.clone()), one());
        let args = [
            ("low", nat()),
            ("complement", nat()),
            ("power", nat()),
            ("balance", eq(nat(), balance.clone(), p.clone())),
        ];
        let right = Equality::natural(balance, p.clone(), var("balance"))
            .reverse()
            .then(law(
                "AddAssoc",
                vec![a.clone(), z.clone(), one()],
                add(add(a.clone(), z.clone()), one()),
                add(a.clone(), add(z.clone(), one())),
            ))?
            .then(
                law(
                    "AddOne",
                    vec![z.clone()],
                    add(z.clone(), one()),
                    succ(z.clone()),
                )
                .on_right(a.clone()),
            )?;
        let e = less_right(a.clone(), right).then(Equality::boolean(
            less(a.clone(), add(a.clone(), succ(z.clone()))),
            bit(true),
            call(
                "Mpk.Go.Practical.Word.Order.LessPositiveExtension",
                vec![a, z],
            ),
        ))?;
        self.theorem("CountComplementBound", &args, e)
    }
    fn no_less_extension(&mut self) -> Result<()> {
        let (p, e) = (var("power"), var("excess"));
        let args = [("power", nat()), ("excess", nat())];
        let first = less_left(
            law(
                "AddComm",
                vec![p.clone(), e.clone()],
                add(p.clone(), e.clone()),
                add(e.clone(), p.clone()),
            ),
            p.clone(),
        );
        let second = less_right(
            add(e.clone(), p.clone()),
            law(
                "ZeroAdd",
                vec![p.clone()],
                add(zero(), p.clone()),
                p.clone(),
            )
            .reverse(),
        );
        let e = first
            .then(second)?
            .then(Equality::boolean(
                less(add(e.clone(), p.clone()), add(zero(), p.clone())),
                less(e.clone(), zero()),
                nc("LessTranslate", vec![e.clone(), zero(), p]),
            ))?
            .then(Equality::boolean(
                less(e.clone(), zero()),
                bit(false),
                nc("LessZero", vec![e]),
            ))?;
        self.theorem("CountNoLessExtension", &args, e)
    }
    fn positive_decomposition(&mut self) -> Result<()> {
        let (n, i) = (var("n"), var("_index"));
        let result = |n: Expr| eq(nat(), n.clone(), add(one(), sub(n, one())));
        let property = |n: Expr| {
            pi(
                "_positive",
                eq(boolean(), less(zero(), n.clone()), bit(true)),
                result(n),
            )
        };
        let contradiction = Equality::boolean(
            less(zero(), zero()),
            bit(false),
            nc("LessZero", vec![zero()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(zero(), zero()),
            bit(true),
            var("_positive"),
        ))?;
        let base = lam(
            "_positive",
            eq(boolean(), less(zero(), zero()), bit(true)),
            bc("FalseElim", vec![result(zero()), contradiction.proof]),
        );
        let tail = law(
            "SubSuccBoth",
            vec![i.clone(), zero()],
            sub(succ(i.clone()), one()),
            sub(i.clone(), zero()),
        )
        .then(law(
            "SubZero",
            vec![i.clone()],
            sub(i.clone(), zero()),
            i.clone(),
        ))?;
        let right = tail
            .on_right(one())
            .then(law(
                "SuccAdd",
                vec![zero(), i.clone()],
                add(one(), i.clone()),
                succ(add(zero(), i.clone())),
            ))?
            .then(
                law(
                    "ZeroAdd",
                    vec![i.clone()],
                    add(zero(), i.clone()),
                    i.clone(),
                )
                .on_succ(),
            )?;
        let step = lam(
            "_index",
            nat(),
            lam(
                "_unused",
                property(i.clone()),
                lam(
                    "_positive",
                    eq(boolean(), less(zero(), succ(i.clone())), bit(true)),
                    right.reverse().proof,
                ),
            ),
        );
        let proof = app(
            induct("_index", property(i.clone()), base, step, n.clone()),
            vec![var("positive")],
        );
        self.theorem(
            "CountPositiveDecomposition",
            &[
                ("n", nat()),
                (
                    "positive",
                    eq(boolean(), less(zero(), n.clone()), bit(true)),
                ),
            ],
            Equality::natural(n.clone(), add(one(), sub(n, one())), proof),
        )
    }
}
