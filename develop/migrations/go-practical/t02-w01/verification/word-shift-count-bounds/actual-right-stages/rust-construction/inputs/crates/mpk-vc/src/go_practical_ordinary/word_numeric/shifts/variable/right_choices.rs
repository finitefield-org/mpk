//! Shared conditional power complements and filled-right scalar conservation.
use super::*;
impl Shifts<'_> {
    pub(super) fn variable_minus_choices(&mut self) -> Result<()> {
        let (n, s, f) = (var("exponent"), var("amount"), var("flag"));
        let args = [("exponent", nat()), ("amount", nat()), ("flag", boolean())];
        let end = |flag: Expr| {
            select(
                flag,
                add(minus(n.clone()), mul(power(n.clone()), minus(s.clone()))),
                minus(n.clone()),
            )
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                minus(add(
                    n.clone(),
                    mul(s.clone(), bit_value(env["_minus_flag"].clone())),
                )),
                end(env["_minus_flag"].clone()),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = env["_minus_flag"] == bit(true);
            let ex = mul_right(s.clone(), bit_literal(yes))
                .on_right(n.clone())
                .then(
                    law(
                        if yes { "MulOne" } else { "MulZero" },
                        vec![s.clone()],
                        mul(s.clone(), if yes { one() } else { zero() }),
                        if yes { s.clone() } else { zero() },
                    )
                    .on_right(n.clone()),
                )?;
            let e = if yes {
                map_minus(ex).then(known(
                    "PowerMinusAdd",
                    vec![n.clone(), s.clone()],
                    minus(add(n.clone(), s.clone())),
                    add(minus(n.clone()), mul(power(n.clone()), minus(s.clone()))),
                ))?
            } else {
                map_minus(ex.then(law(
                    "AddZero",
                    vec![n.clone()],
                    add(n.clone(), zero()),
                    n.clone(),
                ))?)
            };
            Ok(e.then(
                literal(
                    yes,
                    add(minus(n.clone()), mul(power(n.clone()), minus(s.clone()))),
                    minus(n.clone()),
                )
                .reverse(),
            )?
            .proof)
        };
        let proof = app(
            boolean_cases(&["_minus_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![f.clone()],
        );
        self.theorem(
            "ConditionalMinus",
            &args,
            Equality::natural(
                minus(add(n.clone(), mul(s.clone(), bit_value(f.clone())))),
                end(f),
                proof,
            ),
        )?;
        let c = var("count");
        let args = [("count", word_type(64)?)];
        for i in 0..6 {
            let n = digits(i, c.clone());
            let s = coefficient(i);
            let f = at(c.clone(), 64, i)?;
            let body = add(n.clone(), mul(s.clone(), bit_value(f.clone())));
            let e = map_minus(Equality::natural(
                digits(i + 1, c.clone()),
                body.clone(),
                refl(nat(), body.clone()),
            ));
            let target = select(
                f.clone(),
                add(minus(n.clone()), mul(power(n.clone()), minus(s.clone()))),
                minus(n.clone()),
            );
            let e = e.then(known(
                "ConditionalMinus",
                vec![n.clone(), s, f.clone()],
                minus(body),
                target,
            ))?;
            let bridge = map_minus(known(
                &format!("SmallPower{i}"),
                vec![],
                coefficient(i),
                numeral(1 << i),
            ));
            let e = e.then(choice_left(
                f,
                mul_right(power(n.clone()), bridge).on_right(minus(n.clone())),
                minus(n),
            ))?;
            self.theorem(&format!("CountPrefixMinusStep{i}"), &args, e)?;
        }
        Ok(())
    }
    pub(super) fn right_choice(&mut self) -> Result<()> {
        let (f, u, a, z, m, p, q, e, t, b, r, s) = (
            var("flag"),
            var("input"),
            var("old"),
            var("next"),
            var("modulus"),
            var("power"),
            var("step_power"),
            var("fill"),
            var("step_fill"),
            var("bit_value"),
            var("remainder"),
            var("step_remainder"),
        );
        let mut args = vec![("flag", boolean())];
        args.extend(
            [
                "input",
                "old",
                "next",
                "modulus",
                "power",
                "step_power",
                "fill",
                "step_fill",
                "bit_value",
                "remainder",
                "step_remainder",
            ]
            .map(|n| (n, nat())),
        );
        let ll = add(u.clone(), mul(mul(m.clone(), e.clone()), b.clone()));
        let lr = add(mul(p.clone(), a.clone()), r.clone());
        let hl = add(a.clone(), mul(mul(m.clone(), t.clone()), b.clone()));
        let hr = add(mul(q.clone(), z.clone()), s.clone());
        args.extend([
            ("before", eq(nat(), ll.clone(), lr.clone())),
            ("after", eq(nat(), hl, hr)),
        ]);
        let nf = add(e.clone(), mul(p.clone(), t.clone()));
        let np = mul(p.clone(), q.clone());
        let nr = add(r.clone(), mul(p.clone(), s.clone()));
        let endpoints = |flag: Expr| {
            (
                add(
                    u.clone(),
                    mul(
                        mul(m.clone(), select(flag.clone(), nf.clone(), e.clone())),
                        b.clone(),
                    ),
                ),
                add(
                    mul(
                        select(flag.clone(), np.clone(), p.clone()),
                        select(flag.clone(), z.clone(), a.clone()),
                    ),
                    select(flag, nr.clone(), r.clone()),
                ),
            )
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let (l, r) = endpoints(env["_right_flag"].clone());
            eq(nat(), l, r)
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = env["_right_flag"] == bit(true);
            let fill = literal(yes, nf.clone(), e.clone());
            let left = mul_left(mul_right(m.clone(), fill), b.clone()).on_right(u.clone());
            let power = literal(yes, np.clone(), p.clone());
            let output = literal(yes, z.clone(), a.clone());
            let rem = literal(yes, nr.clone(), r.clone());
            let pq = power.right.clone();
            let right = mul_left(power, output.left.clone())
                .on_left(rem.left.clone())
                .then(mul_right(pq.clone(), output.clone()).on_left(rem.left.clone()))?
                .then(rem.on_right(mul(pq, output.right)))?;
            let middle = if yes {
                known(
                    "RightCompose",
                    vec![
                        u.clone(),
                        a.clone(),
                        z.clone(),
                        m.clone(),
                        p.clone(),
                        q.clone(),
                        e.clone(),
                        t.clone(),
                        b.clone(),
                        r.clone(),
                        s.clone(),
                        var("before"),
                        var("after"),
                    ],
                    left.right.clone(),
                    right.right.clone(),
                )
            } else {
                Equality::natural(ll.clone(), lr.clone(), var("before"))
            };
            Ok(left.then(middle)?.then(right.reverse())?.proof)
        };
        let (l, r) = endpoints(f.clone());
        let proof = app(
            boolean_cases(&["_right_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![f],
        );
        self.theorem(
            "RightConditionalCompose",
            &args,
            Equality::natural(l, r, proof),
        )
    }
    pub(super) fn right_boolean_choices(&mut self) -> Result<()> {
        let (f, b) = (var("flag"), var("bit"));
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                choose(env["_same_flag"].clone(), b.clone(), b.clone(), boolean()),
                b.clone(),
            )
        };
        let mut leaf = |_: &BTreeMap<&'static str, Expr>| Ok(refl(boolean(), b.clone()));
        let proof = app(
            boolean_cases(&["_same_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![f.clone()],
        );
        self.theorem(
            "BoolChoiceSame",
            &[("flag", boolean()), ("bit", boolean())],
            Equality::boolean(choose(f, b.clone(), b.clone(), boolean()), b, proof),
        )
    }
}
