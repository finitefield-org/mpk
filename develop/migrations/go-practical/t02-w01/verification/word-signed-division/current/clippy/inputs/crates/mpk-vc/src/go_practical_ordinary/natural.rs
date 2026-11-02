//! Unbounded natural arithmetic. Computation is ordinary recursion; all laws
//! below are ordinary equality terms, including genuinely quantified induction.
use super::syntax::*;
use super::{Builder, Result};

fn lemma(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}
fn az(a: Expr) -> Equality {
    lemma("AddZero", vec![a.clone()], add(a.clone(), zero()), a)
}
fn za(a: Expr) -> Equality {
    lemma("ZeroAdd", vec![a.clone()], add(zero(), a.clone()), a)
}
fn asucc(a: Expr, b: Expr) -> Equality {
    lemma(
        "AddSucc",
        vec![a.clone(), b.clone()],
        add(a.clone(), succ(b.clone())),
        succ(add(a, b)),
    )
}
fn sa(a: Expr, b: Expr) -> Equality {
    lemma(
        "SuccAdd",
        vec![a.clone(), b.clone()],
        add(succ(a.clone()), b.clone()),
        succ(add(a, b)),
    )
}
fn assoc(a: Expr, b: Expr, c: Expr) -> Equality {
    lemma(
        "AddAssoc",
        vec![a.clone(), b.clone(), c.clone()],
        add(add(a.clone(), b.clone()), c.clone()),
        add(a, add(b, c)),
    )
}
fn comm(a: Expr, b: Expr) -> Equality {
    lemma(
        "AddComm",
        vec![a.clone(), b.clone()],
        add(a.clone(), b.clone()),
        add(b, a),
    )
}
fn mz(a: Expr) -> Equality {
    lemma("MulZero", vec![a.clone()], mul(a, zero()), zero())
}
fn ms(a: Expr, b: Expr) -> Equality {
    lemma(
        "MulSucc",
        vec![a.clone(), b.clone()],
        mul(a.clone(), succ(b.clone())),
        add(mul(a.clone(), b), a),
    )
}
fn sz(a: Expr) -> Equality {
    lemma("SubZero", vec![a.clone()], sub(a.clone(), zero()), a)
}
fn ss(a: Expr, b: Expr) -> Equality {
    lemma(
        "SubSucc",
        vec![a.clone(), b.clone()],
        sub(a.clone(), succ(b.clone())),
        pred(sub(a, b)),
    )
}
fn both(a: Expr, b: Expr) -> Equality {
    lemma(
        "SubSuccBoth",
        vec![a.clone(), b.clone()],
        sub(succ(a.clone()), succ(b.clone())),
        sub(a, b),
    )
}
fn step(property: Expr, proof: Expr) -> Expr {
    lam("_n", nat(), lam("_ih", property, proof))
}

struct Natural<'a>(&'a mut Builder);

impl Natural<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        result: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.Nat.{name}"),
            args,
            result,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self) -> Result<()> {
        let (a, b) = (var("a"), var("b"));
        let ab = [("a", nat()), ("b", nat())];
        let nstep = |body| lam("_index", nat(), lam("_accumulator", nat(), body));
        self.emit(
            "Pred",
            &ab[..1],
            nat(),
            call("Std.Nat.rec", vec![zero(), nstep(var("_index")), a.clone()]),
            false,
        )?;
        for (name, base, update) in [
            ("Add", a.clone(), succ(var("_accumulator"))),
            ("Mul", zero(), add(var("_accumulator"), a.clone())),
            ("Sub", a.clone(), pred(var("_accumulator"))),
        ] {
            self.emit(
                name,
                &ab,
                nat(),
                call("Std.Nat.rec", vec![base, nstep(update), b.clone()]),
                false,
            )?;
        }
        let is_zero = induct(
            "_n",
            boolean(),
            bit(true),
            lam("_n", nat(), lam("_ignored", boolean(), bit(false))),
            a.clone(),
        );
        self.emit("IsZero", &ab[..1], boolean(), is_zero, false)?;
        let function = pi("_argument", nat(), boolean());
        let equal_inner = induct(
            "_m",
            boolean(),
            bit(false),
            lam(
                "_m",
                nat(),
                lam("_ignored", boolean(), app(var("_prior"), vec![var("_m")])),
            ),
            var("_other"),
        );
        let equal_step = lam(
            "_n",
            nat(),
            lam(
                "_prior",
                function.clone(),
                lam("_other", nat(), equal_inner),
            ),
        );
        let equal_base = lam("_other", nat(), nc("IsZero", vec![var("_other")]));
        self.emit(
            "Equal",
            &ab,
            boolean(),
            app(
                induct("_n", function.clone(), equal_base, equal_step, a.clone()),
                vec![b.clone()],
            ),
            false,
        )?;
        let less_inner = induct(
            "_m",
            boolean(),
            bit(true),
            lam(
                "_m",
                nat(),
                lam("_ignored", boolean(), app(var("_prior"), vec![var("_m")])),
            ),
            var("_other"),
        );
        let less_step = lam(
            "_n",
            nat(),
            lam("_prior", function.clone(), lam("_other", nat(), less_inner)),
        );
        let less_base = lam("_other", nat(), bit(false));
        self.emit(
            "Less",
            &ab,
            boolean(),
            app(
                induct("_n", function, less_base, less_step, b.clone()),
                vec![a.clone()],
            ),
            false,
        )?;
        self.emit(
            "LessEqual",
            &ab,
            boolean(),
            bc("Not", vec![less(b, a)]),
            false,
        )
    }
    fn addition(&mut self) -> Result<()> {
        let (a, b, c, d, n, ih) = (
            var("a"),
            var("b"),
            var("c"),
            var("d"),
            var("_n"),
            var("_ih"),
        );
        let a1 = [("a", nat())];
        let ab = [("a", nat()), ("b", nat())];
        let abc = [("a", nat()), ("b", nat()), ("c", nat())];
        self.theorem(
            "AddZero",
            &a1,
            Equality::natural(add(a.clone(), zero()), a.clone(), refl(nat(), a.clone())),
        )?;
        self.theorem(
            "AddSucc",
            &ab,
            Equality::natural(
                add(a.clone(), succ(b.clone())),
                succ(add(a.clone(), b.clone())),
                refl(nat(), succ(add(a.clone(), b.clone()))),
            ),
        )?;
        self.theorem(
            "PredZero",
            &[],
            Equality::natural(pred(zero()), zero(), refl(nat(), zero())),
        )?;
        self.theorem(
            "PredSucc",
            &a1,
            Equality::natural(pred(succ(a.clone())), a.clone(), refl(nat(), a.clone())),
        )?;
        let property = eq(nat(), add(zero(), n.clone()), n.clone());
        let e = Equality::natural(add(zero(), n.clone()), n.clone(), ih.clone()).on_succ();
        self.theorem(
            "ZeroAdd",
            &[("b", nat())],
            Equality::natural(
                add(zero(), b.clone()),
                b.clone(),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), zero()),
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let property = eq(
            nat(),
            add(succ(a.clone()), n.clone()),
            succ(add(a.clone(), n.clone())),
        );
        let e = Equality::natural(
            add(succ(a.clone()), n.clone()),
            succ(add(a.clone(), n.clone())),
            ih.clone(),
        )
        .on_succ();
        self.theorem(
            "SuccAdd",
            &ab,
            Equality::natural(
                add(succ(a.clone()), b.clone()),
                succ(add(a.clone(), b.clone())),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), succ(a.clone())),
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let left = add(add(a.clone(), b.clone()), n.clone());
        let right = add(a.clone(), add(b.clone(), n.clone()));
        let property = eq(nat(), left.clone(), right.clone());
        let e = asucc(add(a.clone(), b.clone()), n.clone())
            .then(Equality::natural(left, right, ih.clone()).on_succ())?
            .then(asucc(a.clone(), add(b.clone(), n.clone())).reverse())?
            .then(asucc(b.clone(), n.clone()).reverse().on_right(a.clone()))?;
        let base =
            az(add(a.clone(), b.clone())).then(az(b.clone()).on_right(a.clone()).reverse())?;
        self.theorem(
            "AddAssoc",
            &abc,
            Equality::natural(
                add(add(a.clone(), b.clone()), c.clone()),
                add(a.clone(), add(b.clone(), c.clone())),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    c.clone(),
                ),
            ),
        )?;
        let property = eq(nat(), add(a.clone(), n.clone()), add(n.clone(), a.clone()));
        let e = asucc(a.clone(), n.clone())
            .then(
                Equality::natural(add(a.clone(), n.clone()), add(n.clone(), a.clone()), ih)
                    .on_succ(),
            )?
            .then(sa(n.clone(), a.clone()).reverse())?;
        let base = az(a.clone()).then(za(a.clone()).reverse())?;
        self.theorem(
            "AddComm",
            &ab,
            Equality::natural(
                add(a.clone(), b.clone()),
                add(b.clone(), a.clone()),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        self.theorem(
            "AddOne",
            &a1,
            Equality::natural(
                add(a.clone(), one()),
                succ(a.clone()),
                refl(nat(), succ(a.clone())),
            ),
        )?;
        // This interchange is the only reordering required by distributivity.
        // Every step uses the checked associative/commutative laws above.
        let e = assoc(a.clone(), b.clone(), add(c.clone(), d.clone()))
            .then(
                assoc(b.clone(), c.clone(), d.clone())
                    .reverse()
                    .on_right(a.clone()),
            )?
            .then(
                comm(b.clone(), c.clone())
                    .on_left(d.clone())
                    .on_right(a.clone()),
            )?
            .then(assoc(c.clone(), b.clone(), d.clone()).on_right(a.clone()))?
            .then(assoc(a.clone(), c.clone(), add(b.clone(), d.clone())).reverse())?;
        self.theorem(
            "AddInterchange",
            &[("a", nat()), ("b", nat()), ("c", nat()), ("d", nat())],
            e,
        )
    }
    fn multiplication(&mut self) -> Result<()> {
        let (a, b, c, n, ih) = (var("a"), var("b"), var("c"), var("_n"), var("_ih"));
        let a1 = [("a", nat())];
        let ab = [("a", nat()), ("b", nat())];
        let abc = [("a", nat()), ("b", nat()), ("c", nat())];
        self.theorem(
            "MulZero",
            &a1,
            Equality::natural(mul(a.clone(), zero()), zero(), refl(nat(), zero())),
        )?;
        let result = add(mul(a.clone(), b.clone()), a.clone());
        self.theorem(
            "MulSucc",
            &ab,
            Equality::natural(
                mul(a.clone(), succ(b.clone())),
                result.clone(),
                refl(nat(), result),
            ),
        )?;
        let property = eq(nat(), mul(zero(), n.clone()), zero());
        let e = ms(zero(), n.clone())
            .then(az(mul(zero(), n.clone())))?
            .then(Equality::natural(
                mul(zero(), n.clone()),
                zero(),
                ih.clone(),
            ))?;
        self.theorem(
            "ZeroMul",
            &[("b", nat())],
            Equality::natural(
                mul(zero(), b.clone()),
                zero(),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), zero()),
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let property = eq(nat(), mul(one(), n.clone()), n.clone());
        let e = ms(one(), n.clone())
            .then(Equality::natural(mul(one(), n.clone()), n.clone(), ih.clone()).on_left(one()))?
            .then(lemma(
                "AddOne",
                vec![n.clone()],
                add(n.clone(), one()),
                succ(n.clone()),
            ))?;
        self.theorem(
            "OneMul",
            &[("b", nat())],
            Equality::natural(
                mul(one(), b.clone()),
                b.clone(),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), zero()),
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let e = ms(a.clone(), zero())
            .then(mz(a.clone()).on_left(a.clone()))?
            .then(za(a.clone()))?;
        self.theorem("MulOne", &a1, e)?;
        let left = mul(a.clone(), add(b.clone(), n.clone()));
        let right = add(mul(a.clone(), b.clone()), mul(a.clone(), n.clone()));
        let property = eq(nat(), left.clone(), right.clone());
        let e = asucc(b.clone(), n.clone())
            .mapped(
                lam("_sum", nat(), mul(a.clone(), var("_sum"))),
                mul(a.clone(), add(b.clone(), succ(n.clone()))),
                mul(a.clone(), succ(add(b.clone(), n.clone()))),
                nat(),
            )
            .then(ms(a.clone(), add(b.clone(), n.clone())))?
            .then(Equality::natural(left, right, ih.clone()).on_left(a.clone()))?
            .then(assoc(
                mul(a.clone(), b.clone()),
                mul(a.clone(), n.clone()),
                a.clone(),
            ))?
            .then(
                ms(a.clone(), n.clone())
                    .reverse()
                    .on_right(mul(a.clone(), b.clone())),
            )?;
        let base = az(b.clone())
            .mapped(
                lam("_sum", nat(), mul(a.clone(), var("_sum"))),
                mul(a.clone(), add(b.clone(), zero())),
                mul(a.clone(), b.clone()),
                nat(),
            )
            .then(az(mul(a.clone(), b.clone())).reverse())?
            .then(mz(a.clone()).reverse().on_right(mul(a.clone(), b.clone())))?;
        self.theorem(
            "MulAddRight",
            &abc,
            Equality::natural(
                mul(a.clone(), add(b.clone(), c.clone())),
                add(mul(a.clone(), b.clone()), mul(a.clone(), c.clone())),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    c.clone(),
                ),
            ),
        )?;
        let left = mul(add(a.clone(), b.clone()), n.clone());
        let right = add(mul(a.clone(), n.clone()), mul(b.clone(), n.clone()));
        let property = eq(nat(), left.clone(), right.clone());
        let reordered = add(
            add(mul(a.clone(), n.clone()), a.clone()),
            add(mul(b.clone(), n.clone()), b.clone()),
        );
        let e = ms(add(a.clone(), b.clone()), n.clone())
            .then(
                Equality::natural(left, right.clone(), ih.clone())
                    .on_left(add(a.clone(), b.clone())),
            )?
            .then(lemma(
                "AddInterchange",
                vec![
                    mul(a.clone(), n.clone()),
                    mul(b.clone(), n.clone()),
                    a.clone(),
                    b.clone(),
                ],
                add(right, add(a.clone(), b.clone())),
                reordered,
            ))?
            .then(
                ms(a.clone(), n.clone())
                    .reverse()
                    .on_left(add(mul(b.clone(), n.clone()), b.clone())),
            )?
            .then(
                ms(b.clone(), n.clone())
                    .reverse()
                    .on_right(mul(a.clone(), succ(n.clone()))),
            )?;
        let tail = mz(a.clone())
            .on_left(mul(b.clone(), zero()))
            .then(mz(b.clone()).on_right(zero()))?
            .then(az(zero()))?;
        let base = mz(add(a.clone(), b.clone())).then(tail.reverse())?;
        self.theorem(
            "MulAddLeft",
            &abc,
            Equality::natural(
                mul(add(a.clone(), b.clone()), c.clone()),
                add(mul(a.clone(), c.clone()), mul(b.clone(), c.clone())),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    c.clone(),
                ),
            ),
        )?;
        let e = lemma(
            "AddOne",
            vec![a.clone()],
            add(a.clone(), one()),
            succ(a.clone()),
        )
        .reverse()
        .mapped(
            lam("_sum", nat(), mul(var("_sum"), b.clone())),
            mul(succ(a.clone()), b.clone()),
            mul(add(a.clone(), one()), b.clone()),
            nat(),
        )
        .then(lemma(
            "MulAddLeft",
            vec![a.clone(), one(), b.clone()],
            mul(add(a.clone(), one()), b.clone()),
            add(mul(a.clone(), b.clone()), mul(one(), b.clone())),
        ))?
        .then(
            lemma("OneMul", vec![b.clone()], mul(one(), b.clone()), b.clone())
                .on_right(mul(a.clone(), b.clone())),
        )?;
        self.theorem("SuccMul", &ab, e)?;
        let property = eq(nat(), mul(a.clone(), n.clone()), mul(n.clone(), a.clone()));
        let e = ms(a.clone(), n.clone())
            .then(
                Equality::natural(
                    mul(a.clone(), n.clone()),
                    mul(n.clone(), a.clone()),
                    ih.clone(),
                )
                .on_left(a.clone()),
            )?
            .then(
                lemma(
                    "SuccMul",
                    vec![n.clone(), a.clone()],
                    mul(succ(n.clone()), a.clone()),
                    add(mul(n.clone(), a.clone()), a.clone()),
                )
                .reverse(),
            )?;
        let base = mz(a.clone())
            .then(lemma("ZeroMul", vec![a.clone()], mul(zero(), a.clone()), zero()).reverse())?;
        self.theorem(
            "MulComm",
            &ab,
            Equality::natural(
                mul(a.clone(), b.clone()),
                mul(b.clone(), a.clone()),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let left = mul(mul(a.clone(), b.clone()), n.clone());
        let right = mul(a.clone(), mul(b.clone(), n.clone()));
        let property = eq(nat(), left.clone(), right.clone());
        let e = ms(mul(a.clone(), b.clone()), n.clone())
            .then(Equality::natural(left, right, ih).on_left(mul(a.clone(), b.clone())))?
            .then(
                lemma(
                    "MulAddRight",
                    vec![a.clone(), mul(b.clone(), n.clone()), b.clone()],
                    mul(a.clone(), add(mul(b.clone(), n.clone()), b.clone())),
                    add(
                        mul(a.clone(), mul(b.clone(), n.clone())),
                        mul(a.clone(), b.clone()),
                    ),
                )
                .reverse(),
            )?
            .then(ms(b.clone(), n.clone()).reverse().mapped(
                lam("_product", nat(), mul(a.clone(), var("_product"))),
                mul(a.clone(), add(mul(b.clone(), n.clone()), b.clone())),
                mul(a.clone(), mul(b.clone(), succ(n))),
                nat(),
            ))?;
        let base = mz(mul(a.clone(), b.clone()))
            .then(mz(a.clone()).reverse())?
            .then(mz(b.clone()).reverse().mapped(
                lam("_product", nat(), mul(a.clone(), var("_product"))),
                mul(a.clone(), zero()),
                mul(a.clone(), mul(b.clone(), zero())),
                nat(),
            ))?;
        self.theorem(
            "MulAssoc",
            &abc,
            Equality::natural(
                mul(mul(a.clone(), b.clone()), c.clone()),
                mul(a, mul(b, c.clone())),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    c,
                ),
            ),
        )
    }

    fn equality_order(&mut self) -> Result<()> {
        let (a, b, c, n, ih) = (var("a"), var("b"), var("c"), var("_n"), var("_ih"));
        let a1 = [("a", nat())];
        let ab = [("a", nat()), ("b", nat())];
        let abc = [("a", nat()), ("b", nat()), ("c", nat())];
        for (name, args, left, right) in [
            ("EqualZeroZero", vec![], equal(zero(), zero()), bit(true)),
            (
                "EqualZeroSucc",
                vec![("b", nat())],
                equal(zero(), succ(b.clone())),
                bit(false),
            ),
            (
                "EqualSuccZero",
                a1.to_vec(),
                equal(succ(a.clone()), zero()),
                bit(false),
            ),
            (
                "EqualSuccBoth",
                ab.to_vec(),
                equal(succ(a.clone()), succ(b.clone())),
                equal(a.clone(), b.clone()),
            ),
            ("LessZero", a1.to_vec(), less(a.clone(), zero()), bit(false)),
            (
                "LessZeroSucc",
                vec![("b", nat())],
                less(zero(), succ(b.clone())),
                bit(true),
            ),
            (
                "LessSuccBoth",
                ab.to_vec(),
                less(succ(a.clone()), succ(b.clone())),
                less(a.clone(), b.clone()),
            ),
        ] {
            self.theorem(
                name,
                &args,
                Equality::boolean(left, right.clone(), refl(boolean(), right)),
            )?;
        }
        // Right cancellation uses successor injectivity derived by congruence
        // through Pred, not an axiom or a rule added to either kernel.
        let premise = |k: Expr| eq(nat(), add(a.clone(), k.clone()), add(b.clone(), k));
        let property = pi("_h", premise(n.clone()), eq(nat(), a.clone(), b.clone()));
        let e = asucc(a.clone(), n.clone())
            .reverse()
            .then(Equality::natural(
                add(a.clone(), succ(n.clone())),
                add(b.clone(), succ(n.clone())),
                var("_h"),
            ))?
            .then(asucc(b.clone(), n.clone()))?;
        let left = pred(e.left.clone());
        let right = pred(e.right.clone());
        let e = e.mapped(constant("Mpk.Go.Practical.Nat.Pred"), left, right, nat());
        let e = lemma(
            "PredSucc",
            vec![add(a.clone(), n.clone())],
            pred(succ(add(a.clone(), n.clone()))),
            add(a.clone(), n.clone()),
        )
        .reverse()
        .then(e)?
        .then(lemma(
            "PredSucc",
            vec![add(b.clone(), n.clone())],
            pred(succ(add(b.clone(), n.clone()))),
            add(b.clone(), n.clone()),
        ))?;
        let base = lam("_h", premise(zero()), var("_h"));
        let update = step(
            property.clone(),
            lam(
                "_h",
                premise(succ(n.clone())),
                app(ih.clone(), vec![e.proof]),
            ),
        );
        self.emit(
            "AddCancel",
            &abc,
            pi("_h", premise(c.clone()), eq(nat(), a.clone(), b.clone())),
            induct("_n", property, base, update, c.clone()),
            true,
        )?;
        let property = eq(boolean(), equal(n.clone(), n.clone()), bit(true));
        let e = Equality::boolean(
            equal(succ(n.clone()), succ(n.clone())),
            equal(n.clone(), n.clone()),
            nc("EqualSuccBoth", vec![n.clone(), n.clone()]),
        )
        .then(Equality::boolean(
            equal(n.clone(), n.clone()),
            bit(true),
            ih.clone(),
        ))?;
        self.theorem(
            "EqualReflexive",
            &a1,
            Equality::boolean(
                equal(a.clone(), a.clone()),
                bit(true),
                induct(
                    "_n",
                    property.clone(),
                    nc("EqualZeroZero", vec![]),
                    step(property, e.proof),
                    a.clone(),
                ),
            ),
        )?;
        let target = |t| eq(boolean(), equal(a.clone(), t), bit(true));
        self.emit(
            "EqualComplete",
            &[
                ("a", nat()),
                ("b", nat()),
                ("_same", eq(nat(), a.clone(), b.clone())),
            ],
            target(b.clone()),
            call(
                "Std.Eq.rewrite",
                vec![
                    nat(),
                    a.clone(),
                    b.clone(),
                    lam("_other", nat(), target(var("_other"))),
                    var("_same"),
                    nc("EqualReflexive", vec![a.clone()]),
                ],
            ),
            true,
        )?;
        self.equality_reflect()?;
        for operation in ["Less", "Equal"] {
            let compare = |x, y| nc(operation, vec![x, y]);
            let map_left = |e: Equality, right: Expr| {
                let left = compare(e.left.clone(), right.clone());
                let endpoint = compare(e.right.clone(), right.clone());
                e.mapped(
                    lam("_value", nat(), compare(var("_value"), right)),
                    left,
                    endpoint,
                    boolean(),
                )
            };
            let map_right = |left: Expr, e: Equality| {
                let start = compare(left.clone(), e.left.clone());
                let end = compare(left.clone(), e.right.clone());
                e.mapped(
                    lam("_value", nat(), compare(left, var("_value"))),
                    start,
                    end,
                    boolean(),
                )
            };
            let property = eq(
                boolean(),
                compare(add(a.clone(), n.clone()), add(b.clone(), n.clone())),
                compare(a.clone(), b.clone()),
            );
            let base = map_left(az(a.clone()), add(b.clone(), zero()))
                .then(map_right(a.clone(), az(b.clone())))?;
            let e = map_left(asucc(a.clone(), n.clone()), add(b.clone(), succ(n.clone())))
                .then(map_right(
                    succ(add(a.clone(), n.clone())),
                    asucc(b.clone(), n.clone()),
                ))?
                .then(Equality::boolean(
                    compare(
                        succ(add(a.clone(), n.clone())),
                        succ(add(b.clone(), n.clone())),
                    ),
                    compare(add(a.clone(), n.clone()), add(b.clone(), n.clone())),
                    nc(
                        &format!("{operation}SuccBoth"),
                        vec![add(a.clone(), n.clone()), add(b.clone(), n.clone())],
                    ),
                ))?
                .then(Equality::boolean(
                    compare(add(a.clone(), n.clone()), add(b.clone(), n.clone())),
                    compare(a.clone(), b.clone()),
                    ih.clone(),
                ))?;
            self.theorem(
                &format!("{operation}Translate"),
                &abc,
                Equality::boolean(
                    compare(add(a.clone(), c.clone()), add(b.clone(), c.clone())),
                    compare(a.clone(), b.clone()),
                    induct(
                        "_n",
                        property.clone(),
                        base.proof,
                        step(property, e.proof),
                        c.clone(),
                    ),
                ),
            )?;
        }
        let property = eq(boolean(), less(n.clone(), n.clone()), bit(false));
        let e = Equality::boolean(
            less(succ(n.clone()), succ(n.clone())),
            less(n.clone(), n.clone()),
            nc("LessSuccBoth", vec![n.clone(), n.clone()]),
        )
        .then(Equality::boolean(less(n.clone(), n), bit(false), ih))?;
        self.theorem(
            "LessIrreflexive",
            &a1,
            Equality::boolean(
                less(a.clone(), a.clone()),
                bit(false),
                induct(
                    "_n",
                    property.clone(),
                    nc("LessZero", vec![zero()]),
                    step(property, e.proof),
                    a,
                ),
            ),
        )
    }
    fn equality_reflect(&mut self) -> Result<()> {
        // Nested induction proves reflection for arbitrary naturals. Impossible
        // zero/successor cases require the actual false=true premise.
        let (a, b, k, m, outer, prior) = (
            var("a"),
            var("b"),
            var("_k"),
            var("_m"),
            var("_outer"),
            var("_prior"),
        );
        let result = |x: Expr, y: Expr| {
            pi(
                "_true",
                eq(boolean(), equal(x.clone(), y.clone()), bit(true)),
                eq(nat(), x, y),
            )
        };
        let impossible = |x: Expr, y: Expr| {
            call(
                "Mpk.Go.Practical.Bool.FalseElim",
                vec![eq(nat(), x, y), var("_true")],
            )
        };
        let base = lam(
            "_true",
            eq(boolean(), equal(zero(), zero()), bit(true)),
            refl(nat(), zero()),
        );
        let base_step = lam(
            "_k",
            nat(),
            lam(
                "_unused",
                result(zero(), k.clone()),
                lam(
                    "_true",
                    eq(boolean(), equal(zero(), succ(k.clone())), bit(true)),
                    impossible(zero(), succ(k.clone())),
                ),
            ),
        );
        let base_all = lam(
            "_m",
            nat(),
            induct("_k", result(zero(), k.clone()), base, base_step, m.clone()),
        );
        let property = pi("_m", nat(), result(outer.clone(), m.clone()));
        let successor_base = lam(
            "_true",
            eq(boolean(), equal(succ(outer.clone()), zero()), bit(true)),
            impossible(succ(outer.clone()), zero()),
        );
        let e = Equality::natural(
            outer.clone(),
            k.clone(),
            app(prior, vec![k.clone(), var("_true")]),
        )
        .on_succ();
        let successor_step = lam(
            "_k",
            nat(),
            lam(
                "_unused",
                result(succ(outer.clone()), k.clone()),
                lam(
                    "_true",
                    eq(
                        boolean(),
                        equal(succ(outer.clone()), succ(k.clone())),
                        bit(true),
                    ),
                    e.proof,
                ),
            ),
        );
        let update = lam(
            "_outer",
            nat(),
            lam(
                "_prior",
                property.clone(),
                lam(
                    "_m",
                    nat(),
                    induct(
                        "_k",
                        result(succ(outer.clone()), k),
                        successor_base,
                        successor_step,
                        m,
                    ),
                ),
            ),
        );
        self.emit(
            "EqualReflect",
            &[("a", nat()), ("b", nat())],
            result(a.clone(), b.clone()),
            app(induct("_outer", property, base_all, update, a), vec![b]),
            true,
        )
    }
    fn subtraction(&mut self) -> Result<()> {
        let (a, b, c, n, ih) = (var("a"), var("b"), var("c"), var("_n"), var("_ih"));
        let a1 = [("a", nat())];
        let ab = [("a", nat()), ("b", nat())];
        let abc = [("a", nat()), ("b", nat()), ("c", nat())];
        self.theorem(
            "SubZero",
            &a1,
            Equality::natural(sub(a.clone(), zero()), a.clone(), refl(nat(), a.clone())),
        )?;
        let value = pred(sub(a.clone(), b.clone()));
        self.theorem(
            "SubSucc",
            &ab,
            Equality::natural(
                sub(a.clone(), succ(b.clone())),
                value.clone(),
                refl(nat(), value),
            ),
        )?;
        let map_pred = |e: Equality| {
            let left = pred(e.left.clone());
            let right = pred(e.right.clone());
            e.mapped(constant("Mpk.Go.Practical.Nat.Pred"), left, right, nat())
        };
        let property = eq(
            nat(),
            pred(sub(a.clone(), n.clone())),
            sub(pred(a.clone()), n.clone()),
        );
        let base = map_pred(sz(a.clone())).then(sz(pred(a.clone())).reverse())?;
        let e = map_pred(ss(a.clone(), n.clone()))
            .then(map_pred(Equality::natural(
                pred(sub(a.clone(), n.clone())),
                sub(pred(a.clone()), n.clone()),
                ih.clone(),
            )))?
            .then(ss(pred(a.clone()), n.clone()).reverse())?;
        self.theorem(
            "PredSub",
            &ab,
            Equality::natural(
                pred(sub(a.clone(), b.clone())),
                sub(pred(a.clone()), b.clone()),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let e = ss(succ(a.clone()), b.clone())
            .then(lemma(
                "PredSub",
                vec![succ(a.clone()), b.clone()],
                pred(sub(succ(a.clone()), b.clone())),
                sub(pred(succ(a.clone())), b.clone()),
            ))?
            .then(
                lemma(
                    "PredSucc",
                    vec![a.clone()],
                    pred(succ(a.clone())),
                    a.clone(),
                )
                .mapped(
                    lam("_value", nat(), sub(var("_value"), b.clone())),
                    sub(pred(succ(a.clone())), b.clone()),
                    sub(a.clone(), b.clone()),
                    nat(),
                ),
            )?;
        self.theorem("SubSuccBoth", &ab, e)?;
        let property = eq(nat(), sub(zero(), n.clone()), zero());
        let e = ss(zero(), n.clone())
            .then(map_pred(Equality::natural(
                sub(zero(), n.clone()),
                zero(),
                ih.clone(),
            )))?
            .then(lemma("PredZero", vec![], pred(zero()), zero()))?;
        self.theorem(
            "ZeroSub",
            &[("b", nat())],
            Equality::natural(
                sub(zero(), b.clone()),
                zero(),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), zero()),
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let property = eq(nat(), sub(n.clone(), n.clone()), zero());
        let e = both(n.clone(), n.clone()).then(Equality::natural(
            sub(n.clone(), n.clone()),
            zero(),
            ih.clone(),
        ))?;
        self.theorem(
            "SubSame",
            &a1,
            Equality::natural(
                sub(a.clone(), a.clone()),
                zero(),
                induct(
                    "_n",
                    property.clone(),
                    refl(nat(), zero()),
                    step(property, e.proof),
                    a.clone(),
                ),
            ),
        )?;
        let property = eq(nat(), sub(add(a.clone(), n.clone()), n.clone()), a.clone());
        let base = sz(add(a.clone(), zero())).then(az(a.clone()))?;
        let e = asucc(a.clone(), n.clone())
            .mapped(
                lam("_value", nat(), sub(var("_value"), succ(n.clone()))),
                sub(add(a.clone(), succ(n.clone())), succ(n.clone())),
                sub(succ(add(a.clone(), n.clone())), succ(n.clone())),
                nat(),
            )
            .then(both(add(a.clone(), n.clone()), n.clone()))?
            .then(Equality::natural(
                sub(add(a.clone(), n.clone()), n.clone()),
                a.clone(),
                ih.clone(),
            ))?;
        self.theorem(
            "SubAddRight",
            &ab,
            Equality::natural(
                sub(add(a.clone(), b.clone()), b.clone()),
                a.clone(),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    b.clone(),
                ),
            ),
        )?;
        let property = eq(
            nat(),
            sub(add(a.clone(), n.clone()), add(b.clone(), n.clone())),
            sub(a.clone(), b.clone()),
        );
        let base = az(a.clone())
            .mapped(
                lam("_value", nat(), sub(var("_value"), add(b.clone(), zero()))),
                sub(add(a.clone(), zero()), add(b.clone(), zero())),
                sub(a.clone(), add(b.clone(), zero())),
                nat(),
            )
            .then(az(b.clone()).mapped(
                lam("_value", nat(), sub(a.clone(), var("_value"))),
                sub(a.clone(), add(b.clone(), zero())),
                sub(a.clone(), b.clone()),
                nat(),
            ))?;
        let e = asucc(a.clone(), n.clone())
            .mapped(
                lam(
                    "_value",
                    nat(),
                    sub(var("_value"), add(b.clone(), succ(n.clone()))),
                ),
                sub(
                    add(a.clone(), succ(n.clone())),
                    add(b.clone(), succ(n.clone())),
                ),
                sub(
                    succ(add(a.clone(), n.clone())),
                    add(b.clone(), succ(n.clone())),
                ),
                nat(),
            )
            .then(asucc(b.clone(), n.clone()).mapped(
                lam(
                    "_value",
                    nat(),
                    sub(succ(add(a.clone(), n.clone())), var("_value")),
                ),
                sub(
                    succ(add(a.clone(), n.clone())),
                    add(b.clone(), succ(n.clone())),
                ),
                sub(
                    succ(add(a.clone(), n.clone())),
                    succ(add(b.clone(), n.clone())),
                ),
                nat(),
            ))?
            .then(both(add(a.clone(), n.clone()), add(b.clone(), n.clone())))?
            .then(Equality::natural(
                sub(add(a.clone(), n.clone()), add(b.clone(), n.clone())),
                sub(a.clone(), b.clone()),
                ih.clone(),
            ))?;
        self.theorem(
            "SubTranslate",
            &abc,
            Equality::natural(
                sub(add(a.clone(), c.clone()), add(b.clone(), c.clone())),
                sub(a.clone(), b.clone()),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    c,
                ),
            ),
        )?;
        let property = eq(nat(), sub(a.clone(), add(a.clone(), n.clone())), zero());
        let e = asucc(a.clone(), n.clone())
            .mapped(
                lam("_value", nat(), sub(a.clone(), var("_value"))),
                sub(a.clone(), add(a.clone(), succ(n.clone()))),
                sub(a.clone(), succ(add(a.clone(), n.clone()))),
                nat(),
            )
            .then(ss(a.clone(), add(a.clone(), n.clone())))?
            .then(map_pred(Equality::natural(
                sub(a.clone(), add(a.clone(), n)),
                zero(),
                ih,
            )))?
            .then(lemma("PredZero", vec![], pred(zero()), zero()))?;
        let base = az(a.clone())
            .mapped(
                lam("_value", nat(), sub(a.clone(), var("_value"))),
                sub(a.clone(), add(a.clone(), zero())),
                sub(a.clone(), a.clone()),
                nat(),
            )
            .then(lemma(
                "SubSame",
                vec![a.clone()],
                sub(a.clone(), a.clone()),
                zero(),
            ))?;
        self.theorem(
            "SubExtra",
            &ab,
            Equality::natural(
                sub(a.clone(), add(a.clone(), b.clone())),
                zero(),
                induct(
                    "_n",
                    property.clone(),
                    base.proof,
                    step(property, e.proof),
                    b,
                ),
            ),
        )?;
        self.difference_decomposition()
    }
    fn difference_decomposition(&mut self) -> Result<()> {
        let (a, b, outer, m, k) = (var("a"), var("b"), var("_outer"), var("_m"), var("_k"));
        let result = |x: Expr, y: Expr| {
            pi(
                "_not_less",
                eq(boolean(), less(x.clone(), y.clone()), bit(false)),
                eq(nat(), add(y.clone(), sub(x.clone(), y)), x),
            )
        };
        let base_zero = |x: Expr| -> Result<Expr> {
            let e = sz(x.clone())
                .mapped(
                    lam("_value", nat(), add(zero(), var("_value"))),
                    add(zero(), sub(x.clone(), zero())),
                    add(zero(), x.clone()),
                    nat(),
                )
                .then(za(x.clone()))?;
            Ok(lam(
                "_not_less",
                eq(boolean(), less(x, zero()), bit(false)),
                e.proof,
            ))
        };
        let absurd = call(
            "Std.Eq.symm",
            vec![boolean(), bit(true), bit(false), var("_not_less")],
        );
        let impossible = call(
            "Mpk.Go.Practical.Bool.FalseElim",
            vec![
                eq(
                    nat(),
                    add(succ(k.clone()), sub(zero(), succ(k.clone()))),
                    zero(),
                ),
                absurd,
            ],
        );
        let zero_step = lam(
            "_k",
            nat(),
            lam(
                "_unused",
                result(zero(), k.clone()),
                lam(
                    "_not_less",
                    eq(boolean(), less(zero(), succ(k.clone())), bit(false)),
                    impossible,
                ),
            ),
        );
        let base_all = lam(
            "_m",
            nat(),
            induct(
                "_k",
                result(zero(), k.clone()),
                base_zero(zero())?,
                zero_step,
                m.clone(),
            ),
        );
        let property = pi("_m", nat(), result(outer.clone(), m.clone()));
        let e = both(outer.clone(), k.clone())
            .mapped(
                lam("_value", nat(), add(succ(k.clone()), var("_value"))),
                add(succ(k.clone()), sub(succ(outer.clone()), succ(k.clone()))),
                add(succ(k.clone()), sub(outer.clone(), k.clone())),
                nat(),
            )
            .then(sa(k.clone(), sub(outer.clone(), k.clone())))?
            .then(
                Equality::natural(
                    add(k.clone(), sub(outer.clone(), k.clone())),
                    outer.clone(),
                    app(var("_prior"), vec![k.clone(), var("_not_less")]),
                )
                .on_succ(),
            )?;
        let inner = lam(
            "_k",
            nat(),
            lam(
                "_unused",
                result(succ(outer.clone()), k.clone()),
                lam(
                    "_not_less",
                    eq(
                        boolean(),
                        less(succ(outer.clone()), succ(k.clone())),
                        bit(false),
                    ),
                    e.proof,
                ),
            ),
        );
        let update = lam(
            "_outer",
            nat(),
            lam(
                "_prior",
                property.clone(),
                lam(
                    "_m",
                    nat(),
                    induct(
                        "_k",
                        result(succ(outer.clone()), k),
                        base_zero(succ(outer))?,
                        inner,
                        m,
                    ),
                ),
            ),
        );
        self.emit(
            "DifferenceDecomposition",
            &[("a", nat()), ("b", nat())],
            result(a.clone(), b.clone()),
            app(
                induct("_outer", property, base_all, update, a.clone()),
                vec![b.clone()],
            ),
            true,
        )?;
        let e = lemma(
            "DifferenceDecomposition",
            vec![a.clone(), b.clone(), var("_not_less")],
            add(b.clone(), sub(a.clone(), b.clone())),
            a.clone(),
        )
        .reverse()
        .mapped(
            lam("_value", nat(), sub(b.clone(), var("_value"))),
            sub(b.clone(), a.clone()),
            sub(b.clone(), add(b.clone(), sub(a.clone(), b.clone()))),
            nat(),
        )
        .then(lemma(
            "SubExtra",
            vec![b.clone(), sub(a.clone(), b.clone())],
            sub(b.clone(), add(b.clone(), sub(a.clone(), b.clone()))),
            zero(),
        ))?;
        self.theorem(
            "EmptyLength",
            &[
                ("a", nat()),
                ("b", nat()),
                ("_not_less", eq(boolean(), less(a, b), bit(false))),
            ],
            e,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut natural = Natural(builder);
    natural.definitions()?;
    natural.addition()?;
    natural.multiplication()?;
    natural.equality_order()?;
    natural.subtraction()
}
