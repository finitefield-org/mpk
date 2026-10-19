//! Both actual sign branches refine to a mathematical value plus a correction.
use super::*;

impl Refinement<'_> {
    pub(super) fn selection(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, flag) = (var("word"), var("flag"));
        let args = [("word", word_type(w)?), ("flag", boolean())];
        let nx = wc(w, "Neg", vec![x.clone()]);
        let positive = choose(
            flag.clone(),
            bit_value(sign(w, nx.clone())?),
            bit_value(sign(w, x.clone())?),
            nat(),
        );
        let negative = choose(
            flag.clone(),
            bit_value(wc(w, "DivideDefined", vec![x.clone()])),
            zero(),
            nat(),
        );
        self.emit(
            &format!("W{w}.SelectionCorrection"),
            &args,
            integer(),
            ic("Make", vec![positive.clone(), negative.clone()]),
            false,
        )?;
        for (is_negative, label, rhs) in
            [(false, "Positive", positive), (true, "Negative", negative)]
        {
            self.theorem(
                &format!("W{w}.SelectionCorrection{label}"),
                &args,
                Equality::natural(
                    part(is_negative, correction(w, x.clone(), flag.clone())),
                    rhs.clone(),
                    refl(nat(), rhs),
                ),
            )?;
        }
        let modulus = weight(d);
        let u = value(d, x.clone());
        let n = value(d, nx.clone());
        let expressions = |flag: Expr| {
            (
                signed_nat(flag.clone(), u.clone()),
                ic(
                    "Add",
                    vec![
                        signed(w, selection(w, x.clone(), flag.clone())),
                        ic(
                            "Mul",
                            vec![
                                ic("FromNat", vec![modulus.clone()]),
                                correction(w, x.clone(), flag),
                            ],
                        ),
                    ],
                ),
            )
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let (left, right) = expressions(env["flag"].clone());
            equivalent(left, right)
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let is_negative = env["flag"] == bit(true);
            let (left, right) = expressions(env["flag"].clone());
            let (lp, rp) = self.cross_parts(w, left, right)?;
            let plain = &args[..1];
            let middle = if is_negative {
                let sign_part = mul(modulus.clone(), bit_value(sign(w, nx.clone())?));
                let actual_nonzero = mul(
                    modulus.clone(),
                    bit_value(wc(w, "DivideDefined", vec![x.clone()])),
                );
                let l = self.polynomial(
                    plain,
                    lp.right.clone(),
                    add(sign_part.clone(), actual_nonzero.clone()),
                )?;
                let balance = Equality::natural(
                    add(u.clone(), n.clone()),
                    actual_nonzero,
                    prerequisite(
                        "Magnitude",
                        &format!("W{w}.NegationValueSum"),
                        vec![x.clone()],
                    ),
                );
                let e = l.then(balance.reverse().on_right(sign_part))?;
                let tail = self.polynomial(plain, e.right.clone(), rp.right.clone())?;
                e.then(tail)?
            } else {
                self.polynomial(plain, lp.right.clone(), rp.right.clone())?
            };
            Ok(lp.then(middle)?.then(rp.reverse())?.proof)
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            &format!("W{w}.SignedSelectionRefinement"),
            &args[..1],
            pi(
                "flag",
                boolean(),
                prop(&[("flag", flag)].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
}
