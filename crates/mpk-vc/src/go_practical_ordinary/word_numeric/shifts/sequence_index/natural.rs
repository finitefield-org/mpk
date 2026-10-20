//! Small universal arithmetic/Boolean lemmas; no source operands are listed.
use super::*;

fn theorem(builder: &mut Builder, name: &str, args: &[Binder], e: Equality) -> Result<()> {
    emit(
        builder,
        &format!("Mpk.Go.Practical.Word.Index.{name}"),
        args,
        eq(e.ty, e.left, e.right),
        e.proof,
        true,
    )
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let (lo, p, t) = (var("low"), var("power"), var("tail"));
    let e = law(
        "MulAddRight",
        vec![p.clone(), one(), t.clone()],
        mul(p.clone(), add(one(), t.clone())),
        add(mul(p.clone(), one()), mul(p.clone(), t.clone())),
    )
    .on_right(lo.clone())
    .then(
        law("MulOne", vec![p.clone()], mul(p.clone(), one()), p.clone())
            .on_left(mul(p.clone(), t.clone()))
            .on_right(lo.clone()),
    )?
    .then(
        law(
            "AddAssoc",
            vec![lo.clone(), p.clone(), mul(p.clone(), t.clone())],
            add(add(lo.clone(), p.clone()), mul(p.clone(), t.clone())),
            add(lo.clone(), add(p.clone(), mul(p.clone(), t.clone()))),
        )
        .reverse(),
    )?
    .then(
        law(
            "AddComm",
            vec![lo.clone(), p.clone()],
            add(lo.clone(), p.clone()),
            add(p.clone(), lo.clone()),
        )
        .on_left(mul(p.clone(), t.clone())),
    )?
    .then(law(
        "AddAssoc",
        vec![p.clone(), lo.clone(), mul(p.clone(), t.clone())],
        add(add(p.clone(), lo.clone()), mul(p.clone(), t.clone())),
        add(p.clone(), add(lo.clone(), mul(p.clone(), t.clone()))),
    ))?;
    theorem(
        builder,
        "RebalanceLarge",
        &[("low", nat()), ("power", nat()), ("tail", nat())],
        e,
    )?;
    let b = var("flag");
    let property = |b: Expr| {
        pi(
            "_not",
            eq(boolean(), bc("Not", vec![b.clone()]), bit(true)),
            eq(boolean(), b, bit(false)),
        )
    };
    let yes = lam(
        "_not",
        eq(boolean(), bc("Not", vec![bit(true)]), bit(true)),
        bc(
            "FalseElim",
            vec![eq(boolean(), bit(true), bit(false)), var("_not")],
        ),
    );
    let no = lam(
        "_not",
        eq(boolean(), bc("Not", vec![bit(false)]), bit(true)),
        refl(boolean(), bit(false)),
    );
    emit(
        builder,
        "Mpk.Go.Practical.Word.Index.NotTrueReflect",
        &[("flag", boolean())],
        property(b.clone()),
        call(
            "Std.Bool.cases",
            vec![lam("_flag", boolean(), property(var("_flag"))), no, yes, b],
        ),
        true,
    )?;
    let (lo, p, b) = (var("low"), var("power"), var("flag"));
    let expression = |b: Expr| add(lo.clone(), mul(p.clone(), bit_value(b)));
    let result = |b: Expr| choose(b, add(p.clone(), lo.clone()), lo.clone(), nat());
    let left = mul_right(p.clone(), bit_literal(false))
        .on_right(lo.clone())
        .then(law("MulZero", vec![p.clone()], mul(p.clone(), zero()), zero()).on_right(lo.clone()))?
        .then(law(
            "AddZero",
            vec![lo.clone()],
            add(lo.clone(), zero()),
            lo.clone(),
        ))?;
    let right = mul_right(p.clone(), bit_literal(true))
        .on_right(lo.clone())
        .then(
            law("MulOne", vec![p.clone()], mul(p.clone(), one()), p.clone()).on_right(lo.clone()),
        )?
        .then(law(
            "AddComm",
            vec![lo.clone(), p.clone()],
            add(lo.clone(), p.clone()),
            add(p.clone(), lo.clone()),
        ))?;
    theorem(
        builder,
        "DigitIndexStep",
        &[("low", nat()), ("power", nat()), ("flag", boolean())],
        Equality::natural(
            expression(b.clone()),
            result(b.clone()),
            call(
                "Std.Bool.cases",
                vec![
                    lam(
                        "_flag",
                        boolean(),
                        eq(nat(), expression(var("_flag")), result(var("_flag"))),
                    ),
                    left.proof,
                    right.proof,
                    b,
                ],
            ),
        ),
    )
}
