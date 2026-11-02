//! The half quotient forces an actual minimum dividend and negative unit divisor.
use super::correction::generic_cross;
use super::family::{
    divisor_positive, half_balance, q_balance, q_correction, r_exact, source_guard,
};
use super::*;

fn variable_gap() -> Expr {
    qgap(var("d"), var("q"), var("r"), var("gap"))
}
impl Bounded<'_> {
    pub(super) fn exception_laws(&mut self) -> Result<()> {
        self.half_gap_zero()?;
        self.half_divisor_unit()?;
        self.half_input()?;
        self.unit_half_quotient()?;
        self.signed_unit_reflect()?;
        self.signed_nat_zero()?;
        self.xor_true_left()
    }
    fn half_gap_zero(&mut self) -> Result<()> {
        let (d, q, r, g, h) = (var("d"), var("q"), var("r"), var("gap"), var("h"));
        let total = variable_gap();
        let args = [
            ("d", nat()),
            ("q", nat()),
            ("r", nat()),
            ("gap", nat()),
            ("h", nat()),
            (
                "balance",
                eq(nat(), add(q.clone(), total.clone()), h.clone()),
            ),
            ("half", eq(nat(), q.clone(), h.clone())),
        ];
        let e = nlaw(
            "AddComm",
            vec![total.clone(), h.clone()],
            add(total.clone(), h.clone()),
            add(h.clone(), total.clone()),
        )
        .then(
            Equality::natural(q.clone(), h.clone(), var("half"))
                .on_left(total.clone())
                .reverse(),
        )?
        .then(Equality::natural(
            add(q, total.clone()),
            h.clone(),
            var("balance"),
        ))?
        .then(
            nlaw(
                "ZeroAdd",
                vec![h.clone()],
                add(zero(), h.clone()),
                h.clone(),
            )
            .reverse(),
        )?;
        self.theorem(
            "HalfQuotientGapZero",
            &args,
            Equality::natural(
                qgap(d, var("q"), r, g),
                zero(),
                nc("AddCancel", vec![total, zero(), h, e.proof]),
            ),
        )
    }
    fn half_divisor_unit(&mut self) -> Result<()> {
        let (d, q, r, g, h) = (var("d"), var("q"), var("r"), var("gap"), var("h"));
        let p = sub(d.clone(), one());
        let pq = mul(p.clone(), q.clone());
        let total = variable_gap();
        let args = [
            ("d", nat()),
            ("q", nat()),
            ("r", nat()),
            ("gap", nat()),
            ("h", nat()),
            (
                "positive_d",
                eq(boolean(), less(zero(), d.clone()), bit(true)),
            ),
            (
                "positive_h",
                eq(boolean(), less(zero(), h.clone()), bit(true)),
            ),
            ("gap_zero", eq(nat(), total, zero())),
            ("half", eq(nat(), q.clone(), h.clone())),
        ];
        let pair_zero = zero_sum(false, add(pq.clone(), r.clone()), g, var("gap_zero"));
        let product_zero = zero_sum(false, pq.clone(), r, pair_zero.proof);
        let hp = nlaw(
            "MulComm",
            vec![h.clone(), p.clone()],
            mul(h.clone(), p.clone()),
            mul(p.clone(), h.clone()),
        )
        .then(mul_right(p.clone(), Equality::natural(q, h.clone(), var("half"))).reverse())?
        .then(product_zero)?
        .then(nlaw("MulZero", vec![h.clone()], mul(h.clone(), zero()), zero()).reverse())?;
        let pz = Equality::natural(
            p.clone(),
            zero(),
            prerequisite(
                "Arithmetic",
                "MultiplyCancelPositive",
                vec![h, var("positive_h"), p.clone(), zero(), hp.proof],
            ),
        );
        let e = Equality::natural(
            d.clone(),
            add(one(), p),
            prerequisite(
                "Unsigned",
                "StrictDecomposition",
                vec![zero(), d.clone(), var("positive_d")],
            ),
        )
        .then(pz.on_right(one()))?
        .then(nlaw("AddZero", vec![one()], add(one(), zero()), one()))?;
        self.theorem("HalfQuotientDivisorUnit", &args, e)
    }
    fn half_input(&mut self) -> Result<()> {
        let (a, d, q, r, g, h) = (var("a"), var("d"), var("q"), var("r"), var("gap"), var("h"));
        let args = [
            ("a", nat()),
            ("d", nat()),
            ("q", nat()),
            ("r", nat()),
            ("gap", nat()),
            ("h", nat()),
            ("gap_zero", eq(nat(), variable_gap(), zero())),
            ("half", eq(nat(), add(a.clone(), g.clone()), h.clone())),
        ];
        let gz = zero_sum(
            true,
            add(mul(sub(d, one()), q), r),
            g.clone(),
            var("gap_zero"),
        );
        let e = nlaw(
            "AddZero",
            vec![a.clone()],
            add(a.clone(), zero()),
            a.clone(),
        )
        .reverse()
        .then(gz.on_right(a.clone()).reverse())?
        .then(Equality::natural(add(a, g), h, var("half")))?;
        self.theorem("HalfQuotientInput", &args, e)
    }
    fn unit_half_quotient(&mut self) -> Result<()> {
        let (a, d, q, r, h) = (var("a"), var("d"), var("q"), var("r"), var("h"));
        let args = [
            ("a", nat()),
            ("d", nat()),
            ("q", nat()),
            ("r", nat()),
            ("h", nat()),
            (
                "division",
                eq(nat(), a.clone(), add(mul(d.clone(), q.clone()), r.clone())),
            ),
            ("half", eq(nat(), a.clone(), h.clone())),
            ("unit", eq(nat(), d.clone(), one())),
            (
                "ordered",
                eq(boolean(), less(r.clone(), d.clone()), bit(true)),
            ),
        ];
        let unit = Equality::natural(d.clone(), one(), var("unit"));
        let lt = less_right(r.clone(), unit.clone())
            .reverse()
            .then(Equality::boolean(
                less(r.clone(), d.clone()),
                bit(true),
                var("ordered"),
            ))?;
        let rz = Equality::natural(
            r.clone(),
            zero(),
            db("LessOneZero", vec![r.clone(), lt.proof]),
        );
        let e = Equality::natural(
            a.clone(),
            add(mul(d, q.clone()), r.clone()),
            var("division"),
        )
        .then(mul_left(unit, q.clone()).on_left(r))?
        .then(rz.on_right(mul(one(), q.clone())))?
        .then(nlaw(
            "AddZero",
            vec![mul(one(), q.clone())],
            add(mul(one(), q.clone()), zero()),
            mul(one(), q.clone()),
        ))?
        .then(nlaw("OneMul", vec![q.clone()], mul(one(), q.clone()), q))?;
        self.theorem(
            "UnitDivisorHalfQuotient",
            &args,
            e.reverse().then(Equality::natural(a, h, var("half")))?,
        )
    }
    fn signed_unit_reflect(&mut self) -> Result<()> {
        let n = var("amount");
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            let f = env["flag"].clone();
            pi(
                "_same",
                equivalent(signed_nat(f.clone(), n.clone()), minus_one()),
                eq(
                    boolean(),
                    bc("And", vec![f, equal(n.clone(), one())]),
                    bit(true),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let f = env["flag"].clone();
            let left = signed_nat(f.clone(), n.clone());
            let right = minus_one();
            let (lp, rp) = generic_cross(left.clone(), right.clone())?;
            let reduced = lp
                .clone()
                .reverse()
                .then(Equality::natural(lp.left, rp.left.clone(), var("_same")))?
                .then(rp)?;
            let result = if f == bit(true) {
                let unit = nlaw("ZeroAdd", vec![one()], add(zero(), one()), one())
                    .reverse()
                    .then(reduced)?
                    .then(nlaw(
                        "ZeroAdd",
                        vec![n.clone()],
                        add(zero(), n.clone()),
                        n.clone(),
                    ))?
                    .reverse();
                both(
                    Equality::boolean(bit(true), bit(true), refl(boolean(), bit(true))),
                    Equality::boolean(
                        equal(n.clone(), one()),
                        bit(true),
                        nc("EqualComplete", vec![n.clone(), one(), unit.proof]),
                    ),
                )?
                .proof
            } else {
                let impossible = nlaw(
                    "AddOne",
                    vec![n.clone()],
                    add(n.clone(), one()),
                    succ(n.clone()),
                )
                .reverse()
                .then(reduced)?
                .then(nlaw(
                    "AddZero",
                    vec![zero()],
                    add(zero(), zero()),
                    zero(),
                ))?;
                let valid = Equality::boolean(
                    equal(succ(n.clone()), zero()),
                    bit(true),
                    nc(
                        "EqualComplete",
                        vec![succ(n.clone()), zero(), impossible.proof],
                    ),
                );
                let falsehood = Equality::boolean(
                    equal(succ(n.clone()), zero()),
                    bit(false),
                    nc("EqualSuccZero", vec![n.clone()]),
                )
                .reverse()
                .then(valid)?;
                bc(
                    "FalseElim",
                    vec![
                        eq(
                            boolean(),
                            bc("And", vec![f.clone(), equal(n.clone(), one())]),
                            bit(true),
                        ),
                        falsehood.proof,
                    ],
                )
            };
            Ok(lam("_same", equivalent(left, right), result))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "SignedNatNegativeUnitReflect",
            &[("amount", nat())],
            pi(
                "flag",
                boolean(),
                prop(&[("flag", var("flag"))].into_iter().collect()),
            ),
            proof,
            true,
        )
    }
    fn xor_true_left(&mut self) -> Result<()> {
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                bc("Xor", vec![bit(true), env["flag"].clone()]),
                bc("Not", vec![env["flag"].clone()]),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            Ok(refl(boolean(), bc("Not", vec![env["flag"].clone()])))
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "XorTrueLeft",
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
    fn signed_nat_zero(&mut self) -> Result<()> {
        let prop = |env: &BTreeMap<&'static str, Expr>| {
            equivalent(
                signed_nat(env["flag"].clone(), zero()),
                ic("FromNat", vec![zero()]),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (l, r) = generic_cross(
                signed_nat(env["flag"].clone(), zero()),
                ic("FromNat", vec![zero()]),
            )?;
            let same = Equality::reflexive(l.right.clone());
            Ok(l.then(same)?.then(r.reverse())?.proof)
        };
        let proof = boolean_cases(&["flag"], &BTreeMap::new(), &prop, &mut leaf)?;
        self.emit(
            "SignedNatZero",
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
}

struct MinimumFacts {
    q_half: Equality,
    r_zero: Equality,
    flag_false: Equality,
}
fn minimum_facts(w: usize, x: Expr, y: Expr, valid: Expr) -> Result<MinimumFacts> {
    let d = depth(w)?;
    let h = half(d);
    let sx = sign(w, x.clone())?;
    let sy = sign(w, y.clone())?;
    let a = value(d, wc(w, "AbsBits", vec![x.clone()]));
    let b = value(d, wc(w, "AbsBits", vec![y.clone()]));
    let q = value(d, amount_word(w, true, x.clone(), y.clone()));
    let r = value(d, amount_word(w, false, x.clone(), y.clone()));
    let min = equal(value(d, x.clone()), h.clone());
    let nu = negative_unit(w, y.clone());
    let body = Equality::boolean(
        minimum_case(w, x.clone(), y.clone()),
        bc("And", vec![min.clone(), nu.clone()]),
        field(w, "MinimumMinusOneDefinition", x.clone(), y.clone()),
    )
    .reverse()
    .then(Equality::boolean(
        minimum_case(w, x.clone(), y.clone()),
        bit(true),
        valid,
    ))?;
    let min_true = bc("AndLeft", vec![min, nu.clone(), body.proof.clone()]);
    let nu_true = bc(
        "AndRight",
        vec![equal(value(d, x.clone()), h.clone()), nu, body.proof],
    );
    let nu_body = Equality::boolean(
        negative_unit(w, y.clone()),
        bc("And", vec![sy.clone(), equal(b.clone(), one())]),
        db(&format!("W{w}.NegativeUnitDefinition"), vec![y.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        negative_unit(w, y.clone()),
        bit(true),
        nu_true,
    ))?;
    let sy_true = Equality::boolean(
        sy.clone(),
        bit(true),
        bc(
            "AndLeft",
            vec![sy.clone(), equal(b.clone(), one()), nu_body.proof.clone()],
        ),
    );
    let b_one = Equality::natural(
        b.clone(),
        one(),
        nc(
            "EqualReflect",
            vec![
                b.clone(),
                one(),
                bc("AndRight", vec![sy, equal(b.clone(), one()), nu_body.proof]),
            ],
        ),
    );
    let a_half = Equality::natural(
        a.clone(),
        h.clone(),
        nc(
            "EqualReflect",
            vec![
                a.clone(),
                h.clone(),
                prerequisite(
                    "Bounds",
                    &format!("W{w}.MagnitudeHalfComplete"),
                    vec![x.clone(), min_true],
                ),
            ],
        ),
    );
    let sx_true = Equality::boolean(
        sx.clone(),
        bit(true),
        prerequisite(
            "Bounds",
            &format!("W{w}.MagnitudeHalfForcesSign"),
            vec![x.clone(), a_half.proof.clone()],
        ),
    );
    let division = sd(
        &format!("W{w}.MagnitudeDivisionConservation"),
        vec![x.clone(), y.clone(), var("defined")],
    );
    let rbound = Equality::boolean(
        less(r.clone(), b.clone()),
        bit(true),
        sd(
            &format!("W{w}.MagnitudeRemainderBound"),
            vec![x.clone(), y.clone(), var("defined")],
        ),
    );
    let r_one = less_right(r.clone(), b_one.clone())
        .reverse()
        .then(rbound.clone())?;
    let rz = Equality::natural(
        r.clone(),
        zero(),
        db("LessOneZero", vec![r.clone(), r_one.proof]),
    );
    let qh = Equality::natural(
        q.clone(),
        h.clone(),
        db(
            "UnitDivisorHalfQuotient",
            vec![
                a,
                b,
                q,
                r,
                h,
                division,
                a_half.proof,
                b_one.proof,
                rbound.proof,
            ],
        ),
    );
    let flag = bool_binary("Xor", sx_true, sy_true)?.then(Equality::boolean(
        bc("Xor", vec![bit(true), bit(true)]),
        bit(false),
        refl(boolean(), bit(false)),
    ))?;
    Ok(MinimumFacts {
        q_half: qh,
        r_zero: rz,
        flag_false: flag,
    })
}

impl Bounded<'_> {
    fn negative_unit_laws(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let y = var("word");
        let sy = sign(w, y.clone())?;
        let a = value(d, wc(w, "AbsBits", vec![y.clone()]));
        let plain = [("word", word_type(w)?)];
        let body = bc("And", vec![sy.clone(), equal(a.clone(), one())]);
        self.emit(
            &format!("W{w}.NegativeUnit"),
            &plain,
            boolean(),
            body.clone(),
            false,
        )?;
        let def = Equality::boolean(
            negative_unit(w, y.clone()),
            body.clone(),
            refl(boolean(), body.clone()),
        );
        self.theorem(&format!("W{w}.NegativeUnitDefinition"), &plain, def.clone())?;
        let source = signed(w, y.clone());
        let mag = signed_nat(sy.clone(), a.clone());
        let input = || {
            Relation::new(
                source.clone(),
                mag.clone(),
                prerequisite(
                    "Magnitude",
                    &format!("W{w}.SignedMagnitudeEquivalent"),
                    vec![y.clone()],
                ),
            )
        };
        let mut args = plain.to_vec();
        args.push((
            "valid",
            eq(boolean(), negative_unit(w, y.clone()), bit(true)),
        ));
        let valid = def.clone().reverse().then(Equality::boolean(
            negative_unit(w, y.clone()),
            bit(true),
            var("valid"),
        ))?;
        let sy_true = Equality::boolean(
            sy.clone(),
            bit(true),
            bc(
                "AndLeft",
                vec![sy.clone(), equal(a.clone(), one()), valid.proof.clone()],
            ),
        );
        let unit = Equality::natural(
            a.clone(),
            one(),
            nc(
                "EqualReflect",
                vec![
                    a.clone(),
                    one(),
                    bc(
                        "AndRight",
                        vec![sy.clone(), equal(a.clone(), one()), valid.proof],
                    ),
                ],
            ),
        );
        let flag_eq = sy_true.mapped(
            lam(
                "_unit_sign",
                boolean(),
                signed_nat(var("_unit_sign"), a.clone()),
            ),
            mag.clone(),
            signed_nat(bit(true), a.clone()),
            integer(),
        );
        let value_eq = unit.mapped(
            lam(
                "_unit_amount",
                nat(),
                signed_nat(bit(true), var("_unit_amount")),
            ),
            signed_nat(bit(true), a.clone()),
            signed_nat(bit(true), one()),
            integer(),
        );
        let represented = flag_eq.then(value_eq)?.then(Equality {
            ty: integer(),
            left: signed_nat(bit(true), one()),
            right: minus_one(),
            proof: refl(integer(), minus_one()),
        })?;
        self.relation(
            &format!("W{w}.NegativeUnitSound"),
            &args,
            input().then(Relation::reflexive(minus_one()).replace_left(represented)?)?,
        )?;
        let mut complete = plain.to_vec();
        complete.push(("same", equivalent(source.clone(), minus_one())));
        let represented =
            input()
                .reverse()
                .then(Relation::new(source, minus_one(), var("same")))?;
        let complete_bool = Equality::boolean(
            body,
            bit(true),
            db(
                "SignedNatNegativeUnitReflect",
                vec![a, sy, represented.proof],
            ),
        );
        self.theorem(
            &format!("W{w}.NegativeUnitComplete"),
            &complete,
            def.then(complete_bool)?,
        )
    }
    pub(super) fn exception(&mut self, w: usize) -> Result<()> {
        self.negative_unit_laws(w)?;
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let h = half(d);
        let m = weight(d);
        let plain = [("x", word_type(w)?), ("y", word_type(w)?)];
        let mut args = plain.to_vec();
        args.push(("defined", source_guard(w, y.clone())));
        let sx = sign(w, x.clone())?;
        let sy = sign(w, y.clone())?;
        let flag = quotient_flag(w, x.clone(), y.clone())?;
        let qw = amount_word(w, true, x.clone(), y.clone());
        let q = value(d, qw.clone());
        let r = value(d, amount_word(w, false, x.clone(), y.clone()));
        let a = value(d, wc(w, "AbsBits", vec![x.clone()]));
        let b = value(d, wc(w, "AbsBits", vec![y.clone()]));
        let gx = magnitude_gap(w, x.clone());
        let case_body = bc(
            "And",
            vec![
                equal(value(d, x.clone()), h.clone()),
                negative_unit(w, y.clone()),
            ],
        );
        self.emit(
            &format!("W{w}.MinimumMinusOne"),
            &plain,
            boolean(),
            case_body.clone(),
            false,
        )?;
        let case_def = Equality::boolean(
            minimum_case(w, x.clone(), y.clone()),
            case_body.clone(),
            refl(boolean(), case_body),
        );
        self.theorem(
            &format!("W{w}.MinimumMinusOneDefinition"),
            &plain,
            case_def.clone(),
        )?;
        let ov_body = gate(flag.clone(), equal(q.clone(), h.clone()));
        let ov_def = Equality::boolean(
            overflow(w, x.clone(), y.clone()),
            ov_body.clone(),
            field(w, "QuotientOverflowDefinition", x.clone(), y.clone()),
        );
        let observed = ov_def.clone().reverse().then(Equality::boolean(
            overflow(w, x.clone(), y.clone()),
            bit(true),
            var("_overflow"),
        ))?;
        let flag_not = Equality::boolean(
            bc("Not", vec![flag.clone()]),
            bit(true),
            bc(
                "AndLeft",
                vec![
                    bc("Not", vec![flag.clone()]),
                    equal(q.clone(), h.clone()),
                    observed.proof.clone(),
                ],
            ),
        );
        let qh = Equality::natural(
            q.clone(),
            h.clone(),
            nc(
                "EqualReflect",
                vec![
                    q.clone(),
                    h.clone(),
                    bc(
                        "AndRight",
                        vec![
                            bc("Not", vec![flag.clone()]),
                            equal(q.clone(), h.clone()),
                            observed.proof,
                        ],
                    ),
                ],
            ),
        );
        let qg = Equality::natural(
            qgap_value(w, x.clone(), y.clone()),
            qgap(b.clone(), q.clone(), r.clone(), gx.clone()),
            field(w, "QuotientGapDefinition", x.clone(), y.clone()),
        );
        let gap_balance = qg.on_right(q.clone()).reverse().then(Equality::natural(
            add(q.clone(), qgap_value(w, x.clone(), y.clone())),
            h.clone(),
            q_balance(w, x.clone(), y.clone()),
        ))?;
        let gz = db(
            "HalfQuotientGapZero",
            vec![
                b.clone(),
                q.clone(),
                r.clone(),
                gx.clone(),
                h.clone(),
                gap_balance.proof,
                qh.proof.clone(),
            ],
        );
        let unit = Equality::natural(
            b.clone(),
            one(),
            db(
                "HalfQuotientDivisorUnit",
                vec![
                    b.clone(),
                    q.clone(),
                    r.clone(),
                    gx.clone(),
                    h.clone(),
                    divisor_positive(w, y.clone())?,
                    prerequisite("Bounds", &format!("HalfPositive{d}"), vec![]),
                    gz.clone(),
                    qh.proof,
                ],
            ),
        );
        let a_half = Equality::natural(
            a.clone(),
            h.clone(),
            db(
                "HalfQuotientInput",
                vec![
                    a.clone(),
                    b.clone(),
                    q.clone(),
                    r,
                    gx,
                    h.clone(),
                    gz,
                    half_balance(w, x.clone()),
                ],
            ),
        );
        let min_true = Equality::boolean(
            equal(value(d, x.clone()), h.clone()),
            bit(true),
            prerequisite(
                "Bounds",
                &format!("W{w}.MagnitudeHalfReflect"),
                vec![
                    x.clone(),
                    nc(
                        "EqualComplete",
                        vec![a.clone(), h.clone(), a_half.proof.clone()],
                    ),
                ],
            ),
        );
        let sx_true = Equality::boolean(
            sx.clone(),
            bit(true),
            prerequisite(
                "Bounds",
                &format!("W{w}.MagnitudeHalfForcesSign"),
                vec![x.clone(), a_half.proof],
            ),
        );
        let xor = bool_binary(
            "Xor",
            sx_true,
            Equality::boolean(sy.clone(), sy.clone(), refl(boolean(), sy.clone())),
        )?
        .then(Equality::boolean(
            bc("Xor", vec![bit(true), sy.clone()]),
            bc("Not", vec![sy.clone()]),
            db("XorTrueLeft", vec![sy.clone()]),
        ))?;
        let not_xor = bnot(xor).then(Equality::boolean(
            bc("Not", vec![bc("Not", vec![sy.clone()])]),
            sy.clone(),
            bc("DoubleNot", vec![sy.clone()]),
        ))?;
        let sy_true = not_xor.reverse().then(flag_not)?;
        let d_unit = Equality::boolean(
            equal(b.clone(), one()),
            bit(true),
            nc("EqualComplete", vec![b, one(), unit.proof]),
        );
        let nu_body = both(sy_true, d_unit)?;
        let nu_def = Equality::boolean(
            negative_unit(w, y.clone()),
            nu_body.left.clone(),
            db(&format!("W{w}.NegativeUnitDefinition"), vec![y.clone()]),
        );
        let forward = case_def
            .clone()
            .then(both(min_true, nu_def.then(nu_body)?)?)?;
        let forward = lam(
            "_overflow",
            eq(boolean(), overflow(w, x.clone(), y.clone()), bit(true)),
            forward.proof,
        );
        let facts = minimum_facts(w, x.clone(), y.clone(), var("_minimum"))?;
        let not_flag = bnot(facts.flag_false).then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        let q_half = Equality::boolean(
            equal(q.clone(), h.clone()),
            bit(true),
            nc(
                "EqualComplete",
                vec![q.clone(), h.clone(), facts.q_half.proof],
            ),
        );
        let backward = ov_def.then(both(not_flag, q_half)?)?;
        let backward = lam(
            "_minimum",
            eq(boolean(), minimum_case(w, x.clone(), y.clone()), bit(true)),
            backward.proof,
        );
        let eq_case = Equality::boolean(
            overflow(w, x.clone(), y.clone()),
            minimum_case(w, x.clone(), y.clone()),
            prerequisite(
                "Arithmetic",
                "ReflectionAgreement",
                vec![
                    overflow(w, x.clone(), y.clone()),
                    minimum_case(w, x.clone(), y.clone()),
                    forward,
                    backward,
                ],
            ),
        );
        self.theorem(
            &format!("W{w}.OverflowIsMinimumMinusOne"),
            &args,
            eq_case.clone(),
        )?;
        let cq = q_correction(w, x.clone(), y.clone())
            .replace_right(from_nat_equality(map_bit(eq_case)).reverse())?;
        self.relation(
            &format!("W{w}.QuotientCorrectionMinimum"),
            &args,
            Relation::new(cq.left.clone(), cq.right.clone(), cq.proof),
        )?;
        let cq = Relation::new(
            cq.left,
            cq.right,
            db(
                &format!("W{w}.QuotientCorrectionMinimum"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        let sq = signed(w, wc(w, "SignedDiv", vec![x.clone(), y.clone()]));
        let qm = old_field(w, "MathematicalQuotient", x.clone(), y.clone());
        let old_q = Relation::new(
            qm.clone(),
            ic(
                "Add",
                vec![
                    sq.clone(),
                    ic(
                        "Mul",
                        vec![
                            ic("FromNat", vec![m.clone()]),
                            old_field(w, "QuotientCorrection", x.clone(), y.clone()),
                        ],
                    ),
                ],
            ),
            old_field(w, "QuotientRefinement", x.clone(), y.clone()),
        );
        let classified = old_q.then(
            Relation::reflexive(sq.clone()).plus(cq.times_left(ic("FromNat", vec![m.clone()]))),
        )?;
        self.relation(
            &format!("W{w}.QuotientRefinementMinimum"),
            &args,
            Relation::new(
                classified.left.clone(),
                classified.right.clone(),
                classified.proof,
            ),
        )?;
        let classified = Relation::new(
            classified.left,
            classified.right,
            db(
                &format!("W{w}.QuotientRefinementMinimum"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        let mathematical = Relation::new(
            signed(w, x.clone()),
            ic(
                "Add",
                vec![
                    ic("Mul", vec![signed(w, y.clone()), qm.clone()]),
                    old_field(w, "MathematicalRemainder", x.clone(), y.clone()),
                ],
            ),
            sd(
                &format!("W{w}.MathematicalDivisionConservation"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        self.relation(
            &format!("W{w}.SignedDivisionMinimumConservation"),
            &args,
            mathematical.then(
                Relation::new(
                    classified.left.clone(),
                    classified.right.clone(),
                    classified.proof.clone(),
                )
                .times_left(signed(w, y.clone()))
                .plus(r_exact(w, x.clone(), y.clone())),
            )?,
        )?;
        self.exact_quotient(w, &plain, &args, classified)?;
        let mut minimum = args.clone();
        minimum.push((
            "minimum",
            eq(boolean(), minimum_case(w, x.clone(), y.clone()), bit(true)),
        ));
        let facts = minimum_facts(w, x.clone(), y.clone(), var("minimum"))?;
        self.theorem(
            &format!("W{w}.MinimumCaseQuotientHalf"),
            &minimum,
            facts.q_half.clone(),
        )?;
        self.theorem(
            &format!("W{w}.MinimumCaseRemainderZero"),
            &minimum,
            facts.r_zero.clone(),
        )?;
        let source = wc(w, "SignedDiv", vec![x.clone(), y.clone()]);
        let selected = selection(w, qw.clone(), flag.clone());
        let definition = Equality {
            ty: word_type(w)?,
            left: source.clone(),
            right: selected.clone(),
            proof: old_field(w, "SignedDivDefinition", x.clone(), y.clone()),
        };
        let false_selected = selection(w, qw.clone(), bit(false));
        let literal = facts.flag_false.mapped(
            lam(
                "_quotient_flag",
                boolean(),
                selection(w, qw.clone(), var("_quotient_flag")),
            ),
            selected,
            false_selected.clone(),
            word_type(w)?,
        );
        let word = definition.then(literal)?.then(Equality {
            ty: word_type(w)?,
            left: false_selected,
            right: qw.clone(),
            proof: refl(word_type(w)?, qw.clone()),
        })?;
        let value_eq = word
            .mapped(
                lam(
                    "_minimum_quotient",
                    word_type(w)?,
                    value(d, var("_minimum_quotient")),
                ),
                value(d, source),
                value(d, qw),
                nat(),
            )
            .then(facts.q_half)?;
        self.theorem(
            &format!("W{w}.MinimumSourceQuotientValue"),
            &minimum,
            value_eq,
        )?;
        let rm = old_field(w, "MathematicalRemainder", x.clone(), y.clone());
        let sx = sign(w, x.clone())?;
        let rv = value(d, amount_word(w, false, x.clone(), y.clone()));
        let rdef = Equality {
            ty: integer(),
            left: rm.clone(),
            right: signed_nat(sx.clone(), rv.clone()),
            proof: old_field(w, "MathematicalRemainderDefinition", x.clone(), y.clone()),
        };
        let rzero_eq = facts.r_zero.mapped(
            lam(
                "_zero_remainder",
                nat(),
                signed_nat(sx.clone(), var("_zero_remainder")),
            ),
            signed_nat(sx.clone(), rv),
            signed_nat(sx.clone(), zero()),
            integer(),
        );
        let represented = Relation::new(
            signed_nat(sx.clone(), zero()),
            ic("FromNat", vec![zero()]),
            db("SignedNatZero", vec![sx]),
        )
        .replace_left(rdef.then(rzero_eq)?)?;
        self.relation(
            &format!("W{w}.MinimumSourceRemainderZero"),
            &minimum,
            r_exact(w, x, y).reverse().then(represented)?,
        )
    }
    fn exact_quotient(
        &mut self,
        w: usize,
        plain: &[Binder],
        args: &[Binder],
        classified: Relation,
    ) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let d = depth(w)?;
        let c = minimum_case(w, x.clone(), y.clone());
        let guard = field(w, "QuotientNoOverflow", x.clone(), y.clone());
        self.emit(
            &format!("W{w}.QuotientNoOverflow"),
            plain,
            boolean(),
            bc("Not", vec![c.clone()]),
            false,
        )?;
        let mut valid = args.to_vec();
        valid.push(("valid", eq(boolean(), guard, bit(true))));
        let cf = not_true_false(c.clone(), var("valid"))?;
        let eqzero = from_nat_equality(map_bit(cf).then(bit_literal(false))?);
        let scalar_zero = Relation::reflexive(ic("FromNat", vec![zero()])).replace_left(eqzero)?;
        let sq = signed(w, wc(w, "SignedDiv", vec![x.clone(), y.clone()]));
        let e = classified.then(
            Relation::reflexive(sq.clone())
                .plus(scalar_zero.times_left(ic("FromNat", vec![weight(d)]))),
        )?;
        let remove = Relation::new(
            e.right.clone(),
            sq.clone(),
            db("RemoveZeroCorrection", vec![sq.clone(), weight(d)]),
        );
        let e = e.then(remove)?;
        self.relation(
            &format!("W{w}.QuotientRefinementExact"),
            &valid,
            Relation::new(e.left.clone(), e.right.clone(), e.proof),
        )?;
        let qexact = Relation::new(
            e.left,
            e.right,
            db(
                &format!("W{w}.QuotientRefinementExact"),
                vec![x.clone(), y.clone(), var("defined"), var("valid")],
            ),
        );
        let source = Relation::new(
            signed(w, x.clone()),
            ic(
                "Add",
                vec![
                    ic(
                        "Mul",
                        vec![
                            signed(w, y.clone()),
                            old_field(w, "MathematicalQuotient", x.clone(), y.clone()),
                        ],
                    ),
                    old_field(w, "MathematicalRemainder", x.clone(), y.clone()),
                ],
            ),
            sd(
                &format!("W{w}.MathematicalDivisionConservation"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        self.relation(
            &format!("W{w}.SignedDivisionExactConservation"),
            &valid,
            source.then(
                qexact
                    .times_left(signed(w, y.clone()))
                    .plus(r_exact(w, x, y)),
            )?,
        )
    }
}
