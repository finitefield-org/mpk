//! Full-width safety predicates are separate from storage and permissions.
use super::*;
fn wc(name: &str, args: Vec<Expr>) -> Expr {
    word::wc(64, name, args)
}
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Equality.{name}"), args)
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
fn conjunction(fields: &[Expr]) -> Expr {
    fields[1..].iter().fold(fields[0].clone(), |rest, flag| {
        bc("And", vec![flag.clone(), rest])
    })
}
fn component(fields: &[Expr], mut proof: Expr, wanted: usize) -> Expr {
    for i in (1..fields.len()).rev() {
        let rest = conjunction(&fields[..i]);
        if i == wanted {
            return ec("AndLeft", vec![fields[i].clone(), rest, proof]);
        }
        proof = ec("AndRight", vec![fields[i].clone(), rest, proof]);
    }
    proof
}
fn fold_equality(fields: &[Equality]) -> Result<Equality> {
    let mut result = fields[0].clone();
    for next in &fields[1..] {
        let start = result.clone().mapped(
            lam(
                "_b",
                boolean(),
                bc("And", vec![next.left.clone(), var("_b")]),
            ),
            bc("And", vec![next.left.clone(), result.left]),
            bc("And", vec![next.left.clone(), result.right.clone()]),
            boolean(),
        );
        let end = next.clone().mapped(
            lam(
                "_b",
                boolean(),
                bc("And", vec![var("_b"), result.right.clone()]),
            ),
            bc("And", vec![next.left.clone(), result.right.clone()]),
            bc("And", vec![next.right.clone(), result.right]),
            boolean(),
        );
        result = start.then(end)?;
    }
    Ok(result)
}
fn unsigned(a: Expr, b: Expr) -> Equality {
    Equality::boolean(
        wc("UnsignedLess", vec![a.clone(), b.clone()]),
        less(value(a.clone()), value(b.clone())),
        call(
            "Mpk.Go.Practical.Word.Order.W64.UnsignedLessNatural",
            vec![a, b],
        ),
    )
}
fn nonnegative(w: Expr) -> Result<Equality> {
    let sign = word::at(w.clone(), 64, 63)?;
    let negative = call(
        "Mpk.Go.Practical.Int.Less",
        vec![
            call("Mpk.Go.Practical.Word.Numeric.W64.Signed", vec![w.clone()]),
            call("Mpk.Go.Practical.Int.FromNat", vec![zero()]),
        ],
    );
    Ok(not(Equality::boolean(
        negative,
        sign,
        call("Mpk.Go.Practical.String.Slice.SignedNegative", vec![w]),
    )
    .reverse()))
}
impl Descriptors<'_> {
    pub(super) fn guards(&mut self) -> Result<()> {
        for family in ["Index", "Two", "Three"] {
            let names: &[&'static str] = match family {
                "Index" => &["i"],
                "Two" => &["low", "high"],
                _ => &["low", "high", "max"],
            };
            let mut args = vec![("s", descriptor())];
            args.extend(names.iter().map(|n| (*n, word_type(64))));
            let mut supplied = vec![var("s")];
            supplied.extend(names.iter().map(|n| var(n)));
            let comparisons = match family {
                "Index" => vec![unsigned(var("i"), getter(4, var("s")))],
                "Two" => vec![
                    not(unsigned(var("high"), var("low"))),
                    not(unsigned(getter(5, var("s")), var("high"))),
                ],
                _ => vec![
                    not(unsigned(var("high"), var("low"))),
                    not(unsigned(var("max"), var("high"))),
                    not(unsigned(getter(5, var("s")), var("max"))),
                ],
            };
            let name = format!("{family}Unsigned");
            let raw = comparisons
                .iter()
                .map(|e| e.left.clone())
                .collect::<Vec<_>>();
            let folded = conjunction(&raw);
            self.emit(&name, &args, boolean(), folded.clone(), false)?;
            self.theorem(
                &format!("{name}Step"),
                &args,
                Equality::boolean(
                    dc(&name, supplied.clone()),
                    folded.clone(),
                    refl(boolean(), folded),
                ),
            )?;
            let numeric = Equality::boolean(
                dc(&name, supplied.clone()),
                conjunction(&raw),
                dc(&format!("{name}Step"), supplied.clone()),
            )
            .then(fold_equality(&comparisons)?)?;
            self.theorem(&format!("{name}Numeric"), &args, numeric.clone())?;
            let mut proof_args = args.clone();
            proof_args.push(("h", truth(dc(&name, supplied.clone()), true)));
            for (i, field) in raw.iter().enumerate() {
                self.theorem(
                    &format!("{name}Field{i}"),
                    &proof_args,
                    Equality::boolean(field.clone(), bit(true), component(&raw, var("h"), i)),
                )?;
            }
            let mut signed = vec![numeric];
            for n in names {
                signed.push(nonnegative(var(n))?);
            }
            let name = format!("{family}Signed");
            let raw = signed.iter().map(|e| e.left.clone()).collect::<Vec<_>>();
            let folded = conjunction(&raw);
            self.emit(&name, &args, boolean(), folded.clone(), false)?;
            self.theorem(
                &format!("{name}Step"),
                &args,
                Equality::boolean(
                    dc(&name, supplied.clone()),
                    folded.clone(),
                    refl(boolean(), folded),
                ),
            )?;
            let e = Equality::boolean(
                dc(&name, supplied.clone()),
                conjunction(&raw),
                dc(&format!("{name}Step"), supplied.clone()),
            )
            .then(fold_equality(&signed)?)?;
            self.theorem(&format!("{name}Numeric"), &args, e)?;
            let mut proof_args = args.clone();
            proof_args.push(("h", truth(dc(&name, supplied.clone()), true)));
            for (i, field) in raw.iter().enumerate() {
                self.theorem(
                    &format!("{name}Field{i}"),
                    &proof_args,
                    Equality::boolean(field.clone(), bit(true), component(&raw, var("h"), i)),
                )?;
            }
            if family != "Index" {
                let mut changed = args.clone();
                changed.extend([("offset", word_type(64)), ("length", word_type(64))]);
                let other = dc(
                    "WithBounds",
                    vec![var("s"), var("offset"), var("length"), getter(5, var("s"))],
                );
                let mut new = supplied.clone();
                new[0] = other;
                for suffix in ["Unsigned", "Signed"] {
                    let n = format!("{family}{suffix}");
                    self.theorem(
                        &format!("{n}IgnoresLength"),
                        &changed,
                        Equality::boolean(
                            dc(&n, new.clone()),
                            dc(&n, supplied.clone()),
                            refl(boolean(), dc(&n, supplied.clone())),
                        ),
                    )?;
                }
            }
        }
        let args = [("s", descriptor()), ("i", word_type(64))];
        self.emit(
            "AbsoluteIndex",
            &args,
            nat(),
            add(number(3, var("s")), value(var("i"))),
            false,
        )?;
        self.theorem(
            "CopyAbsoluteIndex",
            &args,
            Equality::natural(
                dc("AbsoluteIndex", vec![dc("Copy", vec![var("s")]), var("i")]),
                dc("AbsoluteIndex", vec![var("s"), var("i")]),
                refl(nat(), dc("AbsoluteIndex", vec![var("s"), var("i")])),
            ),
        )
    }
}
