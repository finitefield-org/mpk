//! Exact full64-bit signed/unsigned index conditions, before array reads.
use super::*;
fn gc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Array.Index.{name}"), args)
}
fn wc(name: &str, args: Vec<Expr>) -> Expr {
    super::super::word::wc(64, name, args)
}
fn sign(i: Expr) -> Result<Expr> {
    super::super::word::at(i, 64, 63)
}
fn ord(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.W64.{name}"), args)
}
fn negative(i: Expr) -> Expr {
    call(
        "Mpk.Go.Practical.Int.Less",
        vec![
            call("Mpk.Go.Practical.Word.Numeric.W64.Signed", vec![i]),
            call("Mpk.Go.Practical.Int.FromNat", vec![zero()]),
        ],
    )
}
fn map_not(e: Equality) -> Equality {
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
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let args = [("length", word()), ("i", word())];
    let (n, i) = (var("length"), var("i"));
    let unsigned = wc("UnsignedLess", vec![i.clone(), n.clone()]);
    let raw_signed = bc(
        "And",
        vec![
            gc("Unsigned", vec![n.clone(), i.clone()]),
            bc("Not", vec![sign(i.clone())?]),
        ],
    );
    for (name, body) in [("Unsigned", unsigned.clone()), ("Signed", raw_signed)] {
        emit(
            builder,
            &format!("Mpk.Go.Practical.Array.Index.{name}"),
            &args,
            boolean(),
            body,
            false,
        )?;
    }
    let e = Equality::boolean(
        gc("Unsigned", vec![n.clone(), i.clone()]),
        less(value(i.clone()), value(n.clone())),
        ord("UnsignedLessNatural", vec![i.clone(), n.clone()]),
    );
    emit(
        builder,
        "Mpk.Go.Practical.Array.Index.UnsignedNatural",
        &args,
        eq(boolean(), e.left.clone(), e.right.clone()),
        e.proof.clone(),
        true,
    )?;
    let left = bc(
        "And",
        vec![e.left.clone(), bc("Not", vec![sign(i.clone())?])],
    );
    let middle = bc(
        "And",
        vec![e.right.clone(), bc("Not", vec![sign(i.clone())?])],
    );
    let changed = e.clone().mapped(
        lam(
            "_b",
            boolean(),
            bc("And", vec![var("_b"), bc("Not", vec![sign(i.clone())?])]),
        ),
        left,
        middle.clone(),
        boolean(),
    );
    let negative_eq = Equality::boolean(
        negative(i.clone()),
        sign(i.clone())?,
        call(
            "Mpk.Go.Practical.String.Slice.SignedNegative",
            vec![i.clone()],
        ),
    );
    let from = map_not(negative_eq).reverse();
    let target = bc(
        "And",
        vec![e.right.clone(), bc("Not", vec![negative(i.clone())])],
    );
    let exact = changed.then(from.mapped(
        lam("_b", boolean(), bc("And", vec![e.right.clone(), var("_b")])),
        middle,
        target.clone(),
        boolean(),
    ))?;
    emit(
        builder,
        "Mpk.Go.Practical.Array.Index.SignedBounds",
        &args,
        eq(boolean(), gc("Signed", vec![n.clone(), i.clone()]), target),
        exact.proof,
        true,
    )?;
    let mut args = args.to_vec();
    args.push(("h", truth(gc("Signed", vec![n.clone(), i.clone()]), true)));
    let within = call(
        "Mpk.Go.Practical.String.Equality.AndLeft",
        vec![
            gc("Unsigned", vec![n.clone(), i.clone()]),
            bc("Not", vec![sign(i.clone())?]),
            var("h"),
        ],
    );
    let within = e.reverse().then(Equality::boolean(
        gc("Unsigned", vec![n, i.clone()]),
        bit(true),
        within,
    ))?;
    emit(
        builder,
        "Mpk.Go.Practical.Array.Index.SignedWithin",
        &args,
        eq(boolean(), within.left, within.right),
        within.proof,
        true,
    )
}
