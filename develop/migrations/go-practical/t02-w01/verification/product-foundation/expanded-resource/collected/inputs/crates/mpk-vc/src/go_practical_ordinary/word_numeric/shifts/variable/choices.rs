//! Local Bool elimination and explicit Nat/Value congruences.
use super::*;
impl Shifts<'_> {
    pub(super) fn variable_power_choices(&mut self) -> Result<()> {
        let (n, s, f) = (var("exponent"), var("amount"), var("flag"));
        let args = [("exponent", nat()), ("amount", nat()), ("flag", boolean())];
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                power(add(
                    n.clone(),
                    mul(s.clone(), bit_value(e["_power_flag"].clone())),
                )),
                select(
                    e["_power_flag"].clone(),
                    mul(power(n.clone()), power(s.clone())),
                    power(n.clone()),
                ),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_power_flag"] == bit(true);
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
            let proof = if yes {
                map_power(ex).then(Equality::natural(
                    power(add(n.clone(), s.clone())),
                    mul(power(n.clone()), power(s.clone())),
                    numeric("PowerTwoAdd", vec![n.clone(), s.clone()]),
                ))?
            } else {
                map_power(ex.then(law(
                    "AddZero",
                    vec![n.clone()],
                    add(n.clone(), zero()),
                    n.clone(),
                ))?)
            };
            Ok(proof
                .then(
                    literal(
                        yes,
                        mul(power(n.clone()), power(s.clone())),
                        power(n.clone()),
                    )
                    .reverse(),
                )?
                .proof)
        };
        self.theorem(
            "ConditionalPower",
            &args,
            Equality::natural(
                power(add(n.clone(), mul(s.clone(), bit_value(f.clone())))),
                select(
                    f.clone(),
                    mul(power(n.clone()), power(s.clone())),
                    power(n.clone()),
                ),
                app(
                    boolean_cases(&["_power_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
                    vec![f],
                ),
            ),
        )?;
        let count = var("count");
        let args = [("count", word_type(64)?)];
        for i in 0..6 {
            let f = at(count.clone(), 64, i)?;
            let n = digits(i, count.clone());
            let s = coefficient(i);
            let body = add(n.clone(), mul(s.clone(), bit_value(f.clone())));
            let e = map_power(Equality::natural(
                digits(i + 1, count.clone()),
                body,
                refl(nat(), add(n.clone(), mul(s.clone(), bit_value(f.clone())))),
            ));
            let target = select(
                f.clone(),
                mul(power(n.clone()), power(s.clone())),
                power(n.clone()),
            );
            let previous_power = e.right.clone();
            let e = e.then(known(
                "ConditionalPower",
                vec![n.clone(), s, f.clone()],
                previous_power,
                target,
            ))?;
            let sp = map_power(known(
                &format!("SmallPower{i}"),
                vec![],
                coefficient(i),
                numeral(1 << i),
            ));
            let e = e.then(choice_left(f, mul_right(power(n.clone()), sp), power(n)))?;
            self.theorem(&format!("CountPrefixPowerStep{i}"), &args, e)?;
        }
        Ok(())
    }
    pub(super) fn variable_select_value(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (f, a, b) = (var("flag"), var("first_word"), var("second_word"));
        let args = [
            ("flag", boolean()),
            ("first_word", word_type(w)?),
            ("second_word", word_type(w)?),
        ];
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                value(
                    d,
                    wc(
                        w,
                        "Select",
                        vec![e["_select_flag"].clone(), a.clone(), b.clone()],
                    ),
                ),
                select(
                    e["_select_flag"].clone(),
                    value(d, a.clone()),
                    value(d, b.clone()),
                ),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_select_flag"] == bit(true);
            let source = wc(w, "Select", vec![bit(yes), a.clone(), b.clone()]);
            let target = if yes { a.clone() } else { b.clone() };
            Ok(self
                .literal_values(d, source, target)
                .then(literal(yes, value(d, a.clone()), value(d, b.clone())).reverse())?
                .proof)
        };
        let proof = app(
            boolean_cases(&["_select_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
            vec![f.clone()],
        );
        self.theorem(
            &format!("SelectValue{w}"),
            &args,
            Equality::natural(
                value(d, wc(w, "Select", vec![f.clone(), a.clone(), b.clone()])),
                select(f.clone(), value(d, a.clone()), value(d, b.clone())),
                proof,
            ),
        )
    }
    pub(super) fn left_compose(&mut self) -> Result<()> {
        let (u, a, b, m, p, q, d, r) = (
            var("input"),
            var("old"),
            var("next"),
            var("modulus"),
            var("power"),
            var("step_power"),
            var("discard"),
            var("step_discard"),
        );
        let mut args = [
            "input",
            "old",
            "next",
            "modulus",
            "power",
            "step_power",
            "discard",
            "step_discard",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        let before = add(a.clone(), mul(m.clone(), d.clone()));
        let after = add(b.clone(), mul(m.clone(), r.clone()));
        args.extend([
            (
                "before",
                eq(nat(), before.clone(), mul(p.clone(), u.clone())),
            ),
            ("after", eq(nat(), after.clone(), mul(q.clone(), a.clone()))),
        ]);
        let lhs = add(b, mul(m.clone(), add(mul(q.clone(), d.clone()), r)));
        let first = add(after.clone(), mul(m.clone(), mul(q.clone(), d)));
        let e = self.polynomial(&args, lhs, first)?.then(
            Equality::natural(after, mul(q.clone(), a), var("after"))
                .on_left(mul(m, mul(q.clone(), var("discard")))),
        )?;
        let target = mul(q.clone(), before.clone());
        let poly = self.polynomial(&args, e.right.clone(), target)?;
        let e = e.then(poly)?.then(mul_right(
            q.clone(),
            Equality::natural(before, mul(p.clone(), u.clone()), var("before")),
        ))?;
        let target = mul(mul(p, q), u);
        let poly = self.polynomial(&args, e.right.clone(), target)?;
        self.theorem("LeftCompose", &args, e.then(poly)?)
    }
    pub(super) fn left_choice(&mut self) -> Result<()> {
        let (f, u, a, b, m, p, q, d, r) = (
            var("flag"),
            var("input"),
            var("old"),
            var("next"),
            var("modulus"),
            var("power"),
            var("step_power"),
            var("discard"),
            var("step_discard"),
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
                "discard",
                "step_discard",
            ]
            .map(|n| (n, nat())),
        );
        let before = add(a.clone(), mul(m.clone(), d.clone()));
        let after = add(b.clone(), mul(m.clone(), r.clone()));
        args.extend([
            (
                "before",
                eq(nat(), before.clone(), mul(p.clone(), u.clone())),
            ),
            ("after", eq(nat(), after.clone(), mul(q.clone(), a.clone()))),
        ]);
        let nd = add(mul(q.clone(), d.clone()), r.clone());
        let np = mul(p.clone(), q.clone());
        let endpoints = |flag: Expr| {
            (
                add(
                    select(flag.clone(), b.clone(), a.clone()),
                    mul(m.clone(), select(flag.clone(), nd.clone(), d.clone())),
                ),
                mul(select(flag, np.clone(), p.clone()), u.clone()),
            )
        };
        let prop = |e: &BTreeMap<&'static str, Expr>| {
            let (l, r) = endpoints(e["_left_flag"].clone());
            eq(nat(), l, r)
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = e["_left_flag"] == bit(true);
            let output = literal(yes, b.clone(), a.clone());
            let discard = literal(yes, nd.clone(), d.clone());
            let left = output
                .clone()
                .on_left(mul(m.clone(), discard.left.clone()))
                .then(mul_right(m.clone(), discard).on_right(output.right))?;
            let middle = if yes {
                known(
                    "LeftCompose",
                    vec![
                        u.clone(),
                        a.clone(),
                        b.clone(),
                        m.clone(),
                        p.clone(),
                        q.clone(),
                        d.clone(),
                        r.clone(),
                        var("before"),
                        var("after"),
                    ],
                    left.right.clone(),
                    mul(np.clone(), u.clone()),
                )
            } else {
                Equality::natural(before.clone(), mul(p.clone(), u.clone()), var("before"))
            };
            Ok(left
                .then(middle)?
                .then(mul_left(literal(yes, np.clone(), p.clone()), u.clone()).reverse())?
                .proof)
        };
        let (l, r) = endpoints(f.clone());
        self.theorem(
            "LeftConditionalCompose",
            &args,
            Equality::natural(
                l,
                r,
                app(
                    boolean_cases(&["_left_flag"], &BTreeMap::new(), &prop, &mut leaf)?,
                    vec![f],
                ),
            ),
        )
    }
}
