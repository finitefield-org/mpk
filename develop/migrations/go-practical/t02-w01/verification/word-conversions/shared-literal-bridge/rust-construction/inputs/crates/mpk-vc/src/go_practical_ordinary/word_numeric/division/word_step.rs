//! Universal step laws use the same PushLow/Sub/Select/overflow decision.
use super::*;

pub(super) fn decision(width: usize, r: Expr, y: Expr, b: Expr) -> Expr {
    dc(&format!("W{width}.Decision"), vec![r, y, b])
}
pub(super) fn next(width: usize, r: Expr, y: Expr, b: Expr) -> Expr {
    dc(&format!("W{width}.NextRemainder"), vec![r, y, b])
}

impl Division<'_> {
    pub(super) fn word_step(&mut self, width: usize) -> Result<()> {
        let depth = depth(width)?;
        let (r, y, b) = (var("r"), var("y"), var("low"));
        let args = [
            ("r", word_type(width)?),
            ("y", word_type(width)?),
            ("low", boolean()),
        ];
        let pushed = wc(width, "PushLow", vec![r.clone(), b.clone()]);
        let sub = wc(width, "Sub", vec![pushed.clone(), y.clone()]);
        let high = at(r.clone(), width, width - 1)?;
        let word_less = wc(width, "UnsignedLess", vec![pushed.clone(), y.clone()]);
        let decide = bc("Or", vec![high.clone(), bc("Not", vec![word_less.clone()])]);
        self.emit(
            &format!("W{width}.Decision"),
            &args,
            boolean(),
            decide,
            false,
        )?;
        let q = decision(width, r.clone(), y.clone(), b.clone());
        let selected = wc(
            width,
            "Select",
            vec![q.clone(), sub.clone(), pushed.clone()],
        );
        self.emit(
            &format!("W{width}.NextRemainder"),
            &args,
            word_type(width)?,
            selected.clone(),
            false,
        )?;
        self.select_value(width)?;
        let carry = wc(
            width,
            &format!("Carry{width}"),
            vec![pushed.clone(), wc(width, "Not", vec![y.clone()]), bit(true)],
        );
        let unfold = Equality::boolean(
            word_less,
            bc("Not", vec![carry.clone()]),
            refl(boolean(), bc("Not", vec![carry.clone()])),
        );
        let flag = bool_map("Not", unfold).then(Equality::boolean(
            bc("Not", vec![bc("Not", vec![carry.clone()])]),
            carry.clone(),
            bc("DoubleNot", vec![carry.clone()]),
        ))?;
        let flag = Equality {
            left: q.clone(),
            ..bool_right("Or", high.clone(), flag)
        };
        self.theorem(&format!("W{width}.DecisionCarry"), &args, flag.clone())?;
        let result = next(width, r.clone(), y.clone(), b.clone());
        let chosen = choose(
            q.clone(),
            value(depth, sub.clone()),
            value(depth, pushed.clone()),
            nat(),
        );
        let value_selected = map_value(
            depth,
            Equality {
                ty: word_type(width)?,
                left: result.clone(),
                right: selected.clone(),
                proof: refl(word_type(width)?, selected),
            },
        )
        .then(instance(
            &format!("W{width}.SelectValue"),
            vec![sub.clone(), pushed.clone(), q.clone()],
            value(
                depth,
                wc(
                    width,
                    "Select",
                    vec![q.clone(), sub.clone(), pushed.clone()],
                ),
            ),
            chosen.clone(),
        ))?;
        let q2 = flag.right.clone();
        let chosen2 = choose(
            q2.clone(),
            value(depth, sub.clone()),
            value(depth, pushed.clone()),
            nat(),
        );
        let values = value_selected.then(flag.clone().mapped(
            lam(
                "_q",
                boolean(),
                choose(
                    var("_q"),
                    value(depth, sub.clone()),
                    value(depth, pushed.clone()),
                    nat(),
                ),
            ),
            chosen,
            chosen2,
            nat(),
        ))?;
        let bound = eq(
            boolean(),
            less(value(depth, r.clone()), value(depth, y.clone())),
            bit(true),
        );
        let mut args = args.to_vec();
        args.push(("remainder_bound", bound));
        let mut common = vec![
            value(depth, r.clone()),
            value(depth, y.clone()),
            weight(depth),
            value(depth, pushed.clone()),
            value(depth, sub.clone()),
            b.clone(),
            high,
            carry,
            var("remainder_bound"),
            order(&format!("W{width}.ValueBound"), vec![y.clone()]),
            order(&format!("W{width}.ValueBound"), vec![sub.clone()]),
            numeric(&format!("W{width}.PushLowConservation"), vec![r.clone(), b]),
            order(
                &format!("W{width}.SubtractionConservation"),
                vec![pushed, y.clone()],
            ),
        ];
        let d = value(depth, y);
        let first = values
            .clone()
            .on_left(mul(d.clone(), bit_value(q.clone())))
            .then(mul_right(d.clone(), map_bit(flag)).on_right(values.right.clone()))?;
        let equation = Equality::natural(
            first.right.clone(),
            add(mul(two(), value(depth, r)), bit_value(var("low"))),
            dc("RestoringConservation", common.clone()),
        );
        self.theorem(
            &format!("W{width}.RestoreConservation"),
            &args,
            first.then(equation)?,
        )?;
        let second = less_left(values, d.clone());
        let proof = dc("RestoringBound", std::mem::take(&mut common));
        self.theorem(
            &format!("W{width}.RestoreBound"),
            &args,
            second
                .clone()
                .then(Equality::boolean(second.right, bit(true), proof))?,
        )
    }
    fn select_value(&mut self, width: usize) -> Result<()> {
        let (yes, no) = (var("yes"), var("no"));
        let d = depth(width)?;
        let property = |e: &BTreeMap<&'static str, Expr>| {
            eq(
                nat(),
                value(
                    d,
                    wc(
                        width,
                        "Select",
                        vec![e["flag"].clone(), yes.clone(), no.clone()],
                    ),
                ),
                choose(
                    e["flag"].clone(),
                    value(d, yes.clone()),
                    value(d, no.clone()),
                    nat(),
                ),
            )
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let flag = e["flag"] == bit(true);
            let target = if flag { yes.clone() } else { no.clone() };
            let e = map_value(
                d,
                Equality {
                    ty: word_type(width)?,
                    left: wc(width, "Select", vec![bit(flag), yes.clone(), no.clone()]),
                    right: target.clone(),
                    proof: refl(word_type(width)?, target.clone()),
                },
            );
            Ok(e.then(
                Equality::natural(
                    choose(
                        bit(flag),
                        value(d, yes.clone()),
                        value(d, no.clone()),
                        nat(),
                    ),
                    value(d, target.clone()),
                    refl(nat(), value(d, target)),
                )
                .reverse(),
            )?
            .proof)
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &property, &mut leaf)?;
        self.emit(
            &format!("W{width}.SelectValue"),
            &[("yes", word_type(width)?), ("no", word_type(width)?)],
            pi(
                "flag",
                boolean(),
                property(&[("flag", var("flag"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
}
