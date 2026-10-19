//! Classify the actual correction from bounded word signs, with real Int laws.
use super::*;

pub(super) fn generic_part(negative: bool, x: Expr) -> Result<Equality> {
    let Expr::Apply(head, args) = &x else {
        return Ok(Equality::reflexive(part(negative, x)));
    };
    let label = if negative { "Negative" } else { "Positive" };
    if **head == constant("Mpk.Go.Practical.Int.Make") {
        let [p, n] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        return Ok(make_projection(negative, p.clone(), n.clone()));
    }
    if **head == constant("Mpk.Go.Practical.Int.FromNat") {
        let [n] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        return Ok(Equality::natural(
            part(negative, x.clone()),
            if negative { zero() } else { n.clone() },
            ic(&format!("{label}OfFromNat"), vec![n.clone()]),
        ));
    }
    if **head == constant(&format!("{SIGNED_DIVISION}.SignedNat")) {
        let [f, n] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        let (yes, no) = if negative {
            (n.clone(), zero())
        } else {
            (zero(), n.clone())
        };
        return Equality::natural(
            part(negative, x.clone()),
            choose(f.clone(), yes.clone(), no.clone(), nat()),
            sd(&format!("SignedNat{label}"), vec![f.clone(), n.clone()]),
        )
        .then(natural_choice(f.clone(), yes, no)?);
    }
    if **head == constant("Mpk.Go.Practical.Int.Add") {
        let [a, b] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        let pa = generic_part(negative, a.clone())?;
        let pb = generic_part(negative, b.clone())?;
        return Equality::natural(
            part(negative, x.clone()),
            add(pa.left.clone(), pb.left.clone()),
            ic(&format!("{label}OfAdd"), vec![a.clone(), b.clone()]),
        )
        .then(pa.clone().on_left(pb.left.clone()))?
        .then(pb.on_right(pa.right));
    }
    if **head == constant("Mpk.Go.Practical.Int.Mul") {
        let [a, b] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        let ap = generic_part(false, a.clone())?;
        let an = generic_part(true, a.clone())?;
        let bp = generic_part(false, b.clone())?;
        let bn = generic_part(true, b.clone())?;
        let prod = |a: Equality, b: Equality| {
            mul_left(a.clone(), b.left.clone()).then(mul_right(a.right, b))
        };
        let (l, r) = if negative {
            (prod(ap, bn)?, prod(an, bp)?)
        } else {
            (prod(ap, bp)?, prod(an, bn)?)
        };
        return Equality::natural(
            part(negative, x.clone()),
            add(l.left.clone(), r.left.clone()),
            ic(&format!("{label}OfMul"), vec![a.clone(), b.clone()]),
        )
        .then(l.clone().on_left(r.left.clone()))?
        .then(r.on_right(l.right));
    }
    Ok(Equality::reflexive(part(negative, x)))
}
pub(super) fn generic_cross(left: Expr, right: Expr) -> Result<(Equality, Equality)> {
    let lp = generic_part(false, left.clone())?;
    let ln = generic_part(true, left)?;
    let rp = generic_part(false, right.clone())?;
    let rn = generic_part(true, right)?;
    Ok((
        lp.clone()
            .on_left(rn.left.clone())
            .then(rn.on_right(lp.right))?,
        rp.clone()
            .on_left(ln.left.clone())
            .then(ln.on_right(rp.right))?,
    ))
}

impl Bounded<'_> {
    pub(super) fn correction_laws(&mut self) -> Result<()> {
        self.gate_laws()?;
        self.scalar_correction()?;
        self.remove_zero_correction()
    }
    fn gate_laws(&mut self) -> Result<()> {
        for flag in [false, true] {
            let prop = |env: &BTreeMap<&'static str, Expr>| {
                eq(
                    boolean(),
                    gate(bit(flag), env["threshold"].clone()),
                    if flag {
                        bit(false)
                    } else {
                        env["threshold"].clone()
                    },
                )
            };
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let b = env["threshold"].clone();
                let not = Equality::boolean(
                    bc("Not", vec![bit(flag)]),
                    bit(!flag),
                    refl(boolean(), bit(!flag)),
                );
                let e = bool_binary(
                    "And",
                    not,
                    Equality::boolean(b.clone(), b.clone(), refl(boolean(), b.clone())),
                )?;
                let target = if flag { bit(false) } else { b };
                Ok(e.clone()
                    .then(Equality::boolean(
                        e.right,
                        target.clone(),
                        refl(boolean(), target),
                    ))?
                    .proof)
            };
            let proof = boolean_cases(&["threshold"], &BTreeMap::new(), &prop, &mut leaf)?;
            self.emit(
                if flag {
                    "GateLiteralTrue"
                } else {
                    "GateLiteralFalse"
                },
                &[],
                pi(
                    "threshold",
                    boolean(),
                    prop(&[("threshold", var("threshold"))].into_iter().collect()),
                ),
                proof,
                true,
            )?;
        }
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(boolean(), gate(env["flag"].clone(), bit(false)), bit(false))
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            Ok(db(
                if env["flag"] == bit(true) {
                    "GateLiteralTrue"
                } else {
                    "GateLiteralFalse"
                },
                vec![bit(false)],
            ))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "GateZero",
            &[],
            pi(
                "flag",
                boolean(),
                prop(&[("flag", var("flag"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn scalar_correction(&mut self) -> Result<()> {
        let (b, z) = (var("threshold"), var("nonzero"));
        let exprs = |f: Expr| {
            let p = choose(f.clone(), bit_value(z.clone()), bit_value(b.clone()), nat());
            let n = choose(f.clone(), bit_value(z.clone()), zero(), nat());
            (p, n, ic("FromNat", vec![bit_value(gate(f, b.clone()))]))
        };
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let (p, n, t) = exprs(env["flag"].clone());
            equivalent(ic("Make", vec![p, n]), t)
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let f = env["flag"].clone();
            let flag = f == bit(true);
            let (p, n, t) = exprs(f.clone());
            let cp = make_projection(false, p.clone(), n.clone()).then(natural_choice(
                f.clone(),
                bit_value(z.clone()),
                bit_value(b.clone()),
            )?)?;
            let cn = make_projection(true, p, n).then(natural_choice(
                f.clone(),
                bit_value(z.clone()),
                zero(),
            )?)?;
            let mut positive = Equality::natural(
                part(false, t.clone()),
                bit_value(gate(f, b.clone())),
                ic(
                    "PositiveOfFromNat",
                    vec![bit_value(gate(bit(flag), b.clone()))],
                ),
            )
            .then(map_bit(Equality::boolean(
                gate(bit(flag), b.clone()),
                if flag { bit(false) } else { b.clone() },
                db(
                    if flag {
                        "GateLiteralTrue"
                    } else {
                        "GateLiteralFalse"
                    },
                    vec![b.clone()],
                ),
            )))?;
            if flag {
                positive = positive.then(bit_literal(false))?;
            }
            let negative = Equality::natural(
                part(true, t),
                zero(),
                ic(
                    "NegativeOfFromNat",
                    vec![bit_value(gate(bit(flag), b.clone()))],
                ),
            );
            let lp = cp
                .clone()
                .on_left(negative.left.clone())
                .then(negative.on_right(cp.right.clone()))?;
            let rp = positive
                .clone()
                .on_left(cn.left.clone())
                .then(cn.on_right(positive.right))?;
            let middle = if flag {
                nlaw(
                    "AddZero",
                    vec![bit_value(z.clone())],
                    add(bit_value(z.clone()), zero()),
                    bit_value(z.clone()),
                )
                .then(
                    nlaw(
                        "ZeroAdd",
                        vec![bit_value(z.clone())],
                        add(zero(), bit_value(z.clone())),
                        bit_value(z.clone()),
                    )
                    .reverse(),
                )?
            } else {
                Equality::reflexive(lp.right.clone())
            };
            Ok(lp.then(middle)?.then(rp.reverse())?.proof)
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "ScalarSelectionCorrection",
            &[("threshold", boolean()), ("nonzero", boolean())],
            pi(
                "flag",
                boolean(),
                prop(&[("flag", var("flag"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn remove_zero_correction(&mut self) -> Result<()> {
        let (x, m) = (var("value"), var("modulus"));
        let args = [("value", integer()), ("modulus", nat())];
        let left = ic(
            "Add",
            vec![
                x.clone(),
                ic(
                    "Mul",
                    vec![ic("FromNat", vec![m.clone()]), ic("FromNat", vec![zero()])],
                ),
            ],
        );
        let (lp, rp) = generic_cross(left.clone(), x.clone())?;
        let middle = self.polynomial(&args, lp.right.clone(), rp.right.clone())?;
        self.relation(
            "RemoveZeroCorrection",
            &args,
            Relation::new(left, x, lp.then(middle)?.then(rp.reverse())?.proof),
        )
    }
    pub(super) fn word_correction(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("word");
        let g = var("gap");
        let f = var("flag");
        let u = value(d, x.clone());
        let h = half(d);
        let theta = equal(u.clone(), h.clone());
        let nz = wc(w, "DivideDefined", vec![x.clone()]);
        let s = sign(w, x.clone())?;
        let ns = sign(w, wc(w, "Neg", vec![x.clone()]))?;
        let args = [
            ("word", word_type(w)?),
            ("gap", nat()),
            ("balance", eq(nat(), add(u, g.clone()), h)),
            ("flag", boolean()),
        ];
        let ps = map_bit(Equality::boolean(
            s.clone(),
            theta.clone(),
            db(
                &format!("W{w}.WordSignIsHalf"),
                vec![x.clone(), g.clone(), var("balance")],
            ),
        ));
        let pn = map_bit(Equality::boolean(
            ns.clone(),
            nz.clone(),
            db(
                &format!("W{w}.NegationSignIsNonzero"),
                vec![x.clone(), g, var("balance")],
            ),
        ));
        let old_p = choose(
            f.clone(),
            bit_value(ns.clone()),
            bit_value(s.clone()),
            nat(),
        );
        let old_n = choose(f.clone(), bit_value(nz.clone()), zero(), nat());
        let mid_p = choose(
            f.clone(),
            bit_value(nz.clone()),
            bit_value(s.clone()),
            nat(),
        );
        let new_p = choose(
            f.clone(),
            bit_value(nz.clone()),
            bit_value(theta.clone()),
            nat(),
        );
        let p = pn
            .mapped(
                lam(
                    "_neg_bit",
                    nat(),
                    choose(f.clone(), var("_neg_bit"), bit_value(s), nat()),
                ),
                old_p.clone(),
                mid_p.clone(),
                nat(),
            )
            .then(ps.mapped(
                lam(
                    "_sign_bit",
                    nat(),
                    choose(f.clone(), bit_value(nz.clone()), var("_sign_bit"), nat()),
                ),
                mid_p,
                new_p.clone(),
                nat(),
            ))?;
        let body = ic("Make", vec![old_p, old_n.clone()]);
        let definition = Equality {
            ty: integer(),
            left: correction(w, x.clone(), f.clone()),
            right: body.clone(),
            proof: refl(integer(), body),
        };
        let changed = definition.then(make_equality(p, Equality::reflexive(old_n.clone()))?)?;
        let target = ic("FromNat", vec![bit_value(gate(f.clone(), theta.clone()))]);
        let relation = Relation::new(
            ic("Make", vec![new_p, old_n]),
            target,
            db("ScalarSelectionCorrection", vec![theta, nz, f]),
        );
        self.relation(
            &format!("W{w}.BoundedSelectionCorrection"),
            &args,
            relation.replace_left(changed)?,
        )
    }
}
