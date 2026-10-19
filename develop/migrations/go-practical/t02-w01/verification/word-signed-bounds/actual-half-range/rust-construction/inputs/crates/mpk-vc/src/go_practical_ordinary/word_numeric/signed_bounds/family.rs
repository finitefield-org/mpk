//! Actual source sign, computed magnitude gap and machine half-range bounds.
use super::*;

fn unary(w: usize, name: &str, x: Expr) -> Expr {
    sb(&format!("W{w}.{name}"), vec![x])
}
fn sign(w: usize, x: Expr) -> Result<Expr> {
    at(x, w, w - 1)
}
fn magnitude(w: usize, x: Expr) -> Result<Expr> {
    Ok(value(depth(w)?, wc(w, "AbsBits", vec![x])))
}
fn gap(w: usize, x: Expr) -> Expr {
    unary(w, "MagnitudeGap", x)
}
impl Bounds<'_> {
    pub(super) fn family(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let (u, n, l, c, h, m) = (
            value(d, x.clone()),
            value(d, wc(w, "Neg", vec![x.clone()])),
            low(d, x.clone()),
            low_complement(d, x.clone()),
            half(d),
            weight(d),
        );
        let s = sign(w, x.clone())?;
        let top_sign = Equality::boolean(
            top(d, x.clone()),
            s.clone(),
            sb(&format!("TopDefinition{d}"), vec![x.clone()]),
        );
        self.theorem(&format!("W{w}.TopSign"), &args, top_sign.clone())?;
        let split = instance(
            &format!("ValueSplit{d}"),
            vec![x.clone()],
            u.clone(),
            add(l.clone(), mul(h.clone(), bit_value(top(d, x.clone())))),
        )
        .then(mul_right(h.clone(), map_bit(top_sign)).on_right(l.clone()))?;
        self.theorem(&format!("W{w}.ValueSplit"), &args, split.clone())?;
        let gap_body = choose(s.clone(), l.clone(), add(c.clone(), one()), nat());
        self.emit(
            &format!("W{w}.MagnitudeGap"),
            &args,
            nat(),
            gap_body.clone(),
            false,
        )?;
        let gap_def = Equality::natural(
            gap(w, x.clone()),
            gap_body.clone(),
            refl(nat(), gap_body.clone()),
        );
        self.theorem(
            &format!("W{w}.MagnitudeGapDefinition"),
            &args,
            gap_def.clone(),
        )?;
        let av = magnitude(w, x.clone())?;
        let av_choice = choose(s.clone(), n.clone(), u.clone(), nat());
        let observed = Equality::natural(
            av.clone(),
            av_choice.clone(),
            mg(&format!("W{w}.MagnitudeValue"), vec![x.clone()]),
        );
        let e = observed
            .clone()
            .on_left(gap(w, x.clone()))
            .then(gap_def.clone().on_right(observed.right))?;
        let negative = lam(
            "_negative",
            eq(boolean(), s.clone(), bit(true)),
            mg(
                &format!("W{w}.NegativeNegationBalance"),
                vec![x.clone(), var("_negative")],
            ),
        );
        let balance = e.then(instance(
            "ScalarMagnitudeGapBalance",
            vec![
                u.clone(),
                n.clone(),
                l.clone(),
                c.clone(),
                h.clone(),
                m,
                s.clone(),
                split.proof.clone(),
                sb(&format!("LowComplementBalance{d}"), vec![x.clone()]),
                sb(&format!("HalfDouble{d}"), vec![]),
                negative,
            ],
            add(av_choice, gap_body),
            h.clone(),
        ))?;
        self.theorem(
            &format!("W{w}.MagnitudeHalfBalance"),
            &args,
            balance.clone(),
        )?;
        let mapped = balance.clone().reverse().mapped(
            lam("_bound", nat(), le(av.clone(), var("_bound"))),
            le(av.clone(), h.clone()),
            le(av.clone(), add(av.clone(), gap(w, x.clone()))),
            boolean(),
        );
        self.theorem(
            &format!("W{w}.MagnitudeHalfBound"),
            &args,
            mapped.then(Equality::boolean(
                le(av.clone(), add(av.clone(), gap(w, x.clone()))),
                bit(true),
                sb("GapLessEqual", vec![av.clone(), gap(w, x.clone())]),
            ))?,
        )?;
        for negative in [false, true] {
            let mut guarded = args.to_vec();
            guarded.push(("sign", eq(boolean(), s.clone(), bit(negative))));
            let literal = Equality::boolean(s.clone(), bit(negative), var("sign"));
            let e = split.clone().then(
                mul_right(
                    h.clone(),
                    map_bit(literal.clone()).then(bit_literal(negative))?,
                )
                .on_right(l.clone()),
            )?;
            let target = if negative {
                add(l.clone(), h.clone())
            } else {
                l.clone()
            };
            let tail = self.polynomial(&guarded, e.right.clone(), target)?;
            let actual = e.then(tail)?;
            self.theorem(
                &format!(
                    "W{w}.{}Value",
                    if negative { "Negative" } else { "Nonnegative" }
                ),
                &guarded,
                actual.clone(),
            )?;
            if negative {
                let selected = literal.mapped(
                    lam(
                        "_flag",
                        boolean(),
                        choose(var("_flag"), l.clone(), add(c.clone(), one()), nat()),
                    ),
                    gap_def.right.clone(),
                    choose(bit(true), l.clone(), add(c.clone(), one()), nat()),
                    nat(),
                );
                let g = gap_def.clone().then(selected)?.then(Equality::natural(
                    choose(bit(true), l.clone(), add(c.clone(), one()), nat()),
                    l.clone(),
                    refl(nat(), l.clone()),
                ))?;
                self.theorem(
                    &format!("W{w}.NegativeMagnitudeLowBalance"),
                    &guarded,
                    g.on_right(av.clone()).reverse().then(balance.clone())?,
                )?;
            } else {
                self.theorem(
                    &format!("W{w}.NonnegativeHalfBound"),
                    &guarded,
                    less_left(actual.clone(), h.clone()).then(Equality::boolean(
                        less(l.clone(), h.clone()),
                        bit(true),
                        sb(&format!("LowStrictHalf{d}"), vec![x.clone()]),
                    ))?,
                )?;
                let selected = literal.mapped(
                    lam(
                        "_flag",
                        boolean(),
                        choose(var("_flag"), n.clone(), u.clone(), nat()),
                    ),
                    observed.right.clone(),
                    choose(bit(false), n.clone(), u.clone(), nat()),
                    nat(),
                );
                let a = observed
                    .clone()
                    .then(selected)?
                    .then(Equality::natural(
                        choose(bit(false), n.clone(), u.clone(), nat()),
                        u.clone(),
                        refl(nat(), u.clone()),
                    ))?
                    .then(actual)?;
                self.theorem(
                    &format!("W{w}.NonnegativeMagnitudeHalfBound"),
                    &guarded,
                    less_left(a, h.clone()).then(Equality::boolean(
                        less(l.clone(), h.clone()),
                        bit(true),
                        sb(&format!("LowStrictHalf{d}"), vec![x.clone()]),
                    ))?,
                )?;
            }
        }
        self.minimum_equivalence(w)
    }
}
