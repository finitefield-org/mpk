//! Actual AbsBits preserves zero/definedness and represents signed magnitude.
use super::projection::part;
use super::*;

impl Magnitude<'_> {
    pub(super) fn magnitude(&mut self, w: usize) -> Result<()> {
        self.select_observations(w)?;
        let d = depth(w)?;
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let s = sign(w, x.clone())?;
        let n = neg(w, x.clone());
        let abs = wc(w, "AbsBits", vec![x.clone()]);
        let selected = wc(w, "Select", vec![s.clone(), n.clone(), x.clone()]);
        let definition = Equality {
            ty: word_type(w)?,
            left: abs.clone(),
            right: selected.clone(),
            proof: refl(word_type(w)?, selected),
        };
        self.theorem(
            &format!("W{w}.MagnitudeDefinition"),
            &args,
            definition.clone(),
        )?;
        let absvalue = map_value(d, definition.clone()).then(instance(
            &format!("W{w}.SelectValue"),
            vec![n.clone(), x.clone(), s.clone()],
            value(d, wc(w, "Select", vec![s.clone(), n.clone(), x.clone()])),
            choose(s.clone(), value(d, n.clone()), value(d, x.clone()), nat()),
        ))?;
        self.theorem(&format!("W{w}.MagnitudeValue"), &args, absvalue.clone())?;
        let iz = |word: Expr| wc(w, "IsZero", vec![word]);
        let z = iz(x.clone());
        let mapped = definition.mapped(
            lam("_abs_word", word_type(w)?, iz(var("_abs_word"))),
            iz(abs.clone()),
            iz(wc(w, "Select", vec![s.clone(), n.clone(), x.clone()])),
            boolean(),
        );
        let selected_zero = Equality::boolean(
            iz(wc(w, "Select", vec![s.clone(), n.clone(), x.clone()])),
            choose(s.clone(), iz(n.clone()), z.clone(), boolean()),
            mc(
                &format!("W{w}.SelectIsZero"),
                vec![n.clone(), x.clone(), s.clone()],
            ),
        );
        let nz = Equality::boolean(
            iz(n.clone()),
            z.clone(),
            unary(w, "NegationIsZero", x.clone()),
        );
        let replaced = nz.mapped(
            lam(
                "_neg_zero",
                boolean(),
                choose(s.clone(), var("_neg_zero"), z.clone(), boolean()),
            ),
            choose(s.clone(), iz(n.clone()), z.clone(), boolean()),
            choose(s.clone(), z.clone(), z.clone(), boolean()),
            boolean(),
        );
        let same = Equality::boolean(
            choose(s.clone(), z.clone(), z.clone(), boolean()),
            z.clone(),
            mc(&format!("W{w}.ChooseZeroSame"), vec![z, s.clone()]),
        );
        let zeros = mapped.then(selected_zero)?.then(replaced)?.then(same)?;
        self.theorem(&format!("W{w}.MagnitudeIsZero"), &args, zeros.clone())?;
        self.theorem(
            &format!("W{w}.MagnitudeDivideDefined"),
            &args,
            Equality {
                left: wc(w, "DivideDefined", vec![abs.clone()]),
                right: wc(w, "DivideDefined", vec![x.clone()]),
                ..bool_map("Not", zeros)
            },
        )?;
        let av = value(d, abs);
        let positive = choose(s.clone(), zero(), av.clone(), nat());
        let negative = choose(s.clone(), av.clone(), zero(), nat());
        let represented = unary(w, "SignedMagnitude", x.clone());
        self.emit(
            &format!("W{w}.SignedMagnitude"),
            &args,
            integer(),
            ic("Make", vec![positive.clone(), negative.clone()]),
            false,
        )?;
        for (isnegative, rhs) in [(false, positive.clone()), (true, negative.clone())] {
            self.theorem(
                &format!(
                    "W{w}.Magnitude{}",
                    if isnegative { "Negative" } else { "Positive" }
                ),
                &args,
                Equality::natural(
                    part(isnegative, represented.clone()),
                    rhs.clone(),
                    refl(nat(), rhs),
                ),
            )?;
        }
        let (lp, rp) = self.cross_parts(w, signed(w, x.clone()), represented.clone())?;
        let vx = value(d, x.clone());
        let map_negative = absvalue.clone().mapped(
            lam(
                "_abs_n",
                nat(),
                choose(s.clone(), var("_abs_n"), zero(), nat()),
            ),
            negative,
            choose(s.clone(), absvalue.right.clone(), zero(), nat()),
            nat(),
        );
        let map_positive = absvalue.clone().mapped(
            lam(
                "_abs_n",
                nat(),
                choose(s.clone(), zero(), var("_abs_n"), nat()),
            ),
            positive,
            choose(s.clone(), zero(), absvalue.right.clone(), nat()),
            nat(),
        );
        let left = lp.then(map_negative.on_right(vx.clone()))?;
        let right = rp.then(map_positive.on_left(mul(weight(d), bit_value(s.clone()))))?;
        let balance = lam(
            "_negative",
            eq(boolean(), s.clone(), bit(true)),
            mc(
                &format!("W{w}.NegativeNegationBalance"),
                vec![x.clone(), var("_negative")],
            ),
        );
        let lhs = left.right.clone();
        let rhs = right.right.clone();
        let cross = left.then(instance(
            "MagnitudeCross",
            vec![vx, value(d, n), weight(d), s, balance],
            lhs,
            rhs,
        ))?;
        self.emit(
            &format!("W{w}.SignedMagnitudeEquivalent"),
            &args,
            ic("Equivalent", vec![signed(w, x), represented]),
            cross.then(right.reverse())?.proof,
            true,
        )
    }
    fn select_observations(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (yes, no) = (var("yes"), var("no"));
        for zero in [false, true] {
            let observation = |word: Expr| {
                if zero {
                    wc(w, "IsZero", vec![word])
                } else {
                    value(d, word)
                }
            };
            let ty = if zero { boolean() } else { nat() };
            let prop = |env: &BTreeMap<&'static str, Expr>| {
                eq(
                    ty.clone(),
                    observation(wc(
                        w,
                        "Select",
                        vec![env["flag"].clone(), yes.clone(), no.clone()],
                    )),
                    choose(
                        env["flag"].clone(),
                        observation(yes.clone()),
                        observation(no.clone()),
                        ty.clone(),
                    ),
                )
            };
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let flag = env["flag"].clone();
                let target = if flag == bit(true) {
                    yes.clone()
                } else {
                    no.clone()
                };
                let selected = wc(w, "Select", vec![flag.clone(), yes.clone(), no.clone()]);
                let word = Equality {
                    ty: word_type(w)?,
                    left: selected.clone(),
                    right: target.clone(),
                    proof: refl(word_type(w)?, target.clone()),
                };
                let mapped = word.mapped(
                    lam(
                        "_select_word",
                        word_type(w)?,
                        observation(var("_select_word")),
                    ),
                    observation(selected),
                    observation(target.clone()),
                    ty.clone(),
                );
                let end = Equality {
                    ty: ty.clone(),
                    left: choose(
                        flag,
                        observation(yes.clone()),
                        observation(no.clone()),
                        ty.clone(),
                    ),
                    right: observation(target.clone()),
                    proof: refl(ty.clone(), observation(target)),
                };
                Ok(mapped.then(end.reverse())?.proof)
            };
            let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
            self.emit(
                &format!("W{w}.Select{}", if zero { "IsZero" } else { "Value" }),
                &[("yes", word_type(w)?), ("no", word_type(w)?)],
                pi(
                    "flag",
                    boolean(),
                    prop(&[("flag", var("flag"))].into_iter().collect()),
                ),
                proof,
                true,
            )?;
        }
        let z = var("zero_flag");
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                choose(env["flag"].clone(), z.clone(), z.clone(), boolean()),
                z.clone(),
            )
        };
        let mut leaf =
            |_: &BTreeMap<&'static str, Expr>| -> Result<Expr> { Ok(refl(boolean(), z.clone())) };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            &format!("W{w}.ChooseZeroSame"),
            &[("zero_flag", boolean())],
            pi(
                "flag",
                boolean(),
                prop(&[("flag", var("flag"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
}
