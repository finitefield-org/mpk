//! Inequality is precisely logical refutation of the full visible relation.
use super::*;
fn false_proof() -> Expr {
    truth(bit(false), true)
}
fn reflect_arguments(f: Expr, g: Expr, h: Expr) -> Vec<Expr> {
    vec![
        var("A"),
        var("P"),
        var("R"),
        var("n"),
        f,
        g,
        var("cmp"),
        var("fp"),
        var("gp"),
        var("reflect"),
        h,
    ]
}
impl Comparisons<'_> {
    pub(super) fn negative(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let yes = equal_by(d, f.clone(), g.clone());
        let no = unequal_by(d, f.clone(), g.clone());
        for flag in [false, true] {
            let mut args = actual(d);
            args.push(("h", truth(yes.clone(), flag)));
            let changed = Equality::boolean(yes.clone(), bit(flag), var("h"));
            let target = bc("Not", vec![bit(flag)]);
            self.theorem(
                d,
                if flag {
                    "EqualTrueNotFalse"
                } else {
                    "EqualFalseNotTrue"
                },
                &args,
                not(changed).then(Equality::boolean(
                    target,
                    bit(!flag),
                    refl(boolean(), bit(!flag)),
                ))?,
            )?;
        }
        let mut args = actual(d);
        args.push(("h", truth(no.clone(), true)));
        self.theorem(
            d,
            "NotTrueEqualFalse",
            &args,
            Equality::boolean(
                yes.clone(),
                bit(false),
                call(
                    "Mpk.Go.Practical.String.Slice.NotTrueFalse",
                    vec![yes.clone(), var("h")],
                ),
            ),
        )?;
        args = actual(d);
        args.push(("h", truth(no.clone(), false)));
        let double = Equality::boolean(
            bc("Not", vec![bc("Not", vec![yes.clone()])]),
            yes.clone(),
            bc("DoubleNot", vec![yes.clone()]),
        );
        self.theorem(
            d,
            "NotFalseEqualTrue",
            &args,
            double
                .reverse()
                .then(not(Equality::boolean(no, bit(false), var("h"))))?
                .then(Equality::boolean(
                    bc("Not", vec![bit(false)]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ))?,
        )?;
        let a = var("A");
        let p = var("P");
        let r = var("R");
        let c = var("cmp");
        let mut common = vec![
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("R", relation(a.clone())),
        ];
        common.extend(actual(d).into_iter().skip(1));
        common.extend([
            ("fp", elements(d, p.clone(), f.clone())),
            ("gp", elements(d, p.clone(), g.clone())),
        ]);
        let related = related(d, r.clone(), f.clone(), g.clone());
        args = common.clone();
        args.extend([
            (
                "complete",
                element_law(a.clone(), p.clone(), r.clone(), c.clone(), false),
            ),
            ("h", truth(unequal_by(d, f.clone(), g.clone()), true)),
        ]);
        let mut complete = reflect_arguments(f.clone(), g.clone(), var("_same"));
        complete[9] = var("complete");
        let contrad = Equality::boolean(
            yes.clone(),
            bit(false),
            qc(d, "NotTrueEqualFalse", {
                let mut args = value_args(f.clone(), g.clone());
                args.push(var("h"));
                args
            }),
        )
        .reverse()
        .then(Equality::boolean(
            yes.clone(),
            bit(true),
            qc(d, "Complete", complete),
        ))?;
        self.emit(
            d,
            "NotRefutes",
            &args,
            pi("_same", related.clone(), false_proof()),
            lam("_same", related.clone(), contrad.proof),
            true,
        )?;
        args = common.clone();
        args.extend([
            (
                "reflect",
                element_law(a.clone(), p.clone(), r.clone(), c, false),
            ),
            ("bad", pi("_same", related.clone(), false_proof())),
        ]);
        // This proof consumes the forward comparator law, not completeness.
        let at = args.len() - 2;
        args[at].1 = element_law(a.clone(), p, r.clone(), var("cmp"), true);
        let from_false = qc(d, "EqualFalseNotTrue", {
            let mut args = value_args(f.clone(), g.clone());
            args.push(var("_case"));
            args
        });
        let contradiction = app(
            var("bad"),
            vec![qc(
                d,
                "Reflect",
                reflect_arguments(f.clone(), g.clone(), var("_case")),
            )],
        );
        let target = truth(unequal_by(d, f.clone(), g.clone()), true);
        self.theorem(
            d,
            "NotFromRefutation",
            &args,
            Equality::boolean(
                unequal_by(d, f.clone(), g.clone()),
                bit(true),
                decide(
                    yes,
                    target.clone(),
                    from_false,
                    bc("FalseElim", vec![target, contradiction]),
                ),
            ),
        )?;
        args = common;
        args.extend([
            (
                "reflect",
                element_law(a.clone(), var("P"), r.clone(), var("cmp"), true),
            ),
            ("h", truth(equal_by(d, f.clone(), g.clone()), true)),
            ("i", word()),
            ("shape", shape(d, var("n"))),
            ("within", inside(value(var("i")), var("n"))),
        ]);
        self.emit(
            d,
            "ReadReflect",
            &args,
            app(
                r.clone(),
                vec![
                    read(d, a.clone(), f.clone(), var("i")),
                    read(d, a.clone(), g.clone(), var("i")),
                ],
            ),
            ac(
                d,
                "RelatedRead",
                vec![
                    a.clone(),
                    a,
                    r,
                    var("n"),
                    f.clone(),
                    g.clone(),
                    qc(d, "Reflect", reflect_arguments(f, g, var("h"))),
                    var("i"),
                    var("shape"),
                    var("within"),
                ],
            ),
            true,
        )
    }
}
fn not(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc("Not", vec![var("_b")])),
        l,
        r,
        boolean(),
    )
}
pub(super) fn empty(builder: &mut Builder) -> Result<()> {
    let args = [("a", boolean()), ("b", boolean())];
    for (name, yes) in [("Equal", true), ("NotEqual", false)] {
        let full = format!("Mpk.Go.Practical.Array.Equality.Empty.{name}");
        emit(builder, &full, &args, boolean(), bit(yes), false)?;
        emit(
            builder,
            &format!("{full}Value"),
            &args,
            truth(call(&full, vec![var("a"), var("b")]), yes),
            refl(boolean(), bit(yes)),
            true,
        )?;
    }
    let mut valid = args.to_vec();
    valid.extend([
        (
            "av",
            call("Mpk.Go.Practical.Array.Empty.Valid", vec![var("a")]),
        ),
        (
            "bv",
            call("Mpk.Go.Practical.Array.Empty.Valid", vec![var("b")]),
        ),
    ]);
    let canonical = Equality::boolean(var("a"), bit(false), var("av"))
        .then(Equality::boolean(var("b"), bit(false), var("bv")).reverse())?;
    emit(
        builder,
        "Mpk.Go.Practical.Array.Equality.Empty.CanonicalEquality",
        &valid,
        eq(boolean(), canonical.left, canonical.right),
        canonical.proof,
        true,
    )
}
