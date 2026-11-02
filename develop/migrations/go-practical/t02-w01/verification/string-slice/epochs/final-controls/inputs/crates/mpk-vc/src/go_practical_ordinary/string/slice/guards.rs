//! Actual full-width signed-index guard; no low-bit proxy for either endpoint.
use super::*;

pub(super) fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Equality.{name}"), args)
}
pub(super) fn defined(s: Expr, a: Expr, b: Expr) -> Expr {
    sl("Defined", vec![s, a, b])
}
fn fields(s: Expr, a: Expr, b: Expr) -> Result<Vec<Expr>> {
    Ok(vec![
        bc("Not", vec![sign(a.clone())?]),
        bc("Not", vec![sign(b.clone())?]),
        bc("Not", vec![wc("UnsignedLess", vec![b.clone(), a])]),
        bc("Not", vec![wc("UnsignedLess", vec![length(s), b])]),
    ])
}
pub(super) fn conjunction(fields: &[Expr]) -> Expr {
    fields[1..].iter().fold(fields[0].clone(), |rest, flag| {
        bc("And", vec![flag.clone(), rest])
    })
}
pub(super) fn component(fields: &[Expr], mut proof: Expr, wanted: usize) -> Expr {
    for i in (1..fields.len()).rev() {
        let rest = conjunction(&fields[..i]);
        if i == wanted {
            return ec("AndLeft", vec![fields[i].clone(), rest, proof]);
        }
        proof = ec("AndRight", vec![fields[i].clone(), rest, proof]);
    }
    proof
}
pub(super) fn and_true(a: Equality, b: Equality) -> Result<Equality> {
    bool_left("And", a, b.left.clone())
        .then(bool_right("And", bit(true), b))?
        .then(Equality::boolean(
            bc("And", vec![bit(true), bit(true)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))
}
pub(super) fn not_false(e: Equality) -> Result<Equality> {
    not(e).then(Equality::boolean(
        bc("Not", vec![bit(false)]),
        bit(true),
        refl(boolean(), bit(true)),
    ))
}
pub(super) fn ordered(s: Expr, a: Expr, b: Expr, h: Expr) -> Expr {
    sl("DefinedOrdered", vec![s, a, b, h])
}
pub(super) fn high(s: Expr, a: Expr, b: Expr, h: Expr) -> Expr {
    sl("DefinedHigh", vec![s, a, b, h])
}

impl Slice<'_> {
    pub(super) fn guards(&mut self) -> Result<()> {
        let p = var("p");
        let e = Equality::boolean(
            bc("Not", vec![bc("Not", vec![p.clone()])]),
            p.clone(),
            bc("DoubleNot", vec![p.clone()]),
        )
        .reverse()
        .then(not(Equality::boolean(
            bc("Not", vec![p.clone()]),
            bit(true),
            var("h"),
        )))?
        .then(Equality::boolean(
            bc("Not", vec![bit(true)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        self.theorem(
            "NotTrueFalse",
            &[("p", boolean()), ("h", truth(bc("Not", vec![p]), true))],
            e,
        )?;
        let (s, a, b) = (var("s"), var("a"), var("b"));
        let base = [("s", string_type()), ("a", word64()), ("b", word64())];
        let fields = fields(s.clone(), a.clone(), b.clone())?;
        let body = conjunction(&fields);
        self.emit("Defined", &base, boolean(), body.clone(), false)?;
        self.theorem(
            "DefinedStep",
            &base,
            Equality::boolean(
                defined(s.clone(), a.clone(), b.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        let mut args = base.to_vec();
        args.push(("h", truth(defined(s.clone(), a.clone(), b.clone()), true)));
        let all = Equality::boolean(
            defined(s.clone(), a.clone(), b.clone()),
            body,
            sl("DefinedStep", vec![s.clone(), a.clone(), b.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            defined(s.clone(), a.clone(), b.clone()),
            bit(true),
            var("h"),
        ))?;
        for (i, name, flag) in [
            (0, "DefinedLowSign", sign(a.clone())?),
            (1, "DefinedHighSign", sign(b.clone())?),
            (
                2,
                "DefinedOrderedCircuit",
                wc("UnsignedLess", vec![b.clone(), a.clone()]),
            ),
            (
                3,
                "DefinedHighCircuit",
                wc("UnsignedLess", vec![length(s.clone()), b.clone()]),
            ),
        ] {
            self.theorem(
                name,
                &args,
                Equality::boolean(
                    flag.clone(),
                    bit(false),
                    sl(
                        "NotTrueFalse",
                        vec![flag, component(&fields, all.proof.clone(), i)],
                    ),
                ),
            )?;
        }
        for (name, circuit, x, y, n, m) in [
            (
                "DefinedOrdered",
                "DefinedOrderedCircuit",
                b.clone(),
                a.clone(),
                value(b.clone()),
                value(a.clone()),
            ),
            (
                "DefinedHigh",
                "DefinedHighCircuit",
                length(s.clone()),
                b.clone(),
                size(s.clone()),
                value(b.clone()),
            ),
        ] {
            let flag = wc("UnsignedLess", vec![x.clone(), y.clone()]);
            self.theorem(
                name,
                &args,
                Equality::boolean(
                    flag.clone(),
                    less(n.clone(), m.clone()),
                    ord("W64.UnsignedLessNatural", vec![x, y]),
                )
                .reverse()
                .then(Equality::boolean(
                    flag,
                    bit(false),
                    sl(circuit, vec![s.clone(), a.clone(), b.clone(), var("h")]),
                ))?,
            )?;
        }
        let mut args = base.to_vec();
        args.extend([
            ("low_clear", truth(sign(a.clone())?, false)),
            ("high_clear", truth(sign(b.clone())?, false)),
            (
                "ordered",
                truth(less(value(b.clone()), value(a.clone())), false),
            ),
            (
                "high",
                truth(less(size(s.clone()), value(b.clone())), false),
            ),
        ]);
        let mut proofs = vec![
            not_false(Equality::boolean(
                sign(a.clone())?,
                bit(false),
                var("low_clear"),
            ))?,
            not_false(Equality::boolean(
                sign(b.clone())?,
                bit(false),
                var("high_clear"),
            ))?,
        ];
        for (x, y, n, m, h) in [
            (
                b.clone(),
                a.clone(),
                value(b.clone()),
                value(a.clone()),
                var("ordered"),
            ),
            (
                length(s.clone()),
                b.clone(),
                size(s.clone()),
                value(b.clone()),
                var("high"),
            ),
        ] {
            proofs.push(not_false(
                Equality::boolean(
                    wc("UnsignedLess", vec![x.clone(), y.clone()]),
                    less(n.clone(), m.clone()),
                    ord("W64.UnsignedLessNatural", vec![x, y]),
                )
                .then(Equality::boolean(less(n, m), bit(false), h))?,
            )?);
        }
        let mut proof = proofs[0].clone();
        for next in &proofs[1..] {
            proof = and_true(next.clone(), proof)?;
        }
        self.theorem(
            "DefinedComplete",
            &args,
            Equality::boolean(
                defined(s.clone(), a.clone(), b.clone()),
                conjunction(&fields),
                sl("DefinedStep", vec![s, a, b]),
            )
            .then(proof)?,
        )
    }
}
