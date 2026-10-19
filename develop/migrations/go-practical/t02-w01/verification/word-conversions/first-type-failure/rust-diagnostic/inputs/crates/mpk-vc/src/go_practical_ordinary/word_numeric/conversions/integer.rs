//! Expand only actual established balanced-Int projection equations.
use super::*;
fn product(a: Equality, b: Equality) -> Result<Equality> {
    mul_left(a.clone(), b.left.clone()).then(mul_right(a.right, b))
}
fn projection(negative: bool, x: Expr) -> Result<Equality> {
    let Expr::Apply(head, args) = &x else {
        return Err(GoOrdinaryError::Linkage);
    };
    let label = if negative { "Negative" } else { "Positive" };
    for w in WIDTHS {
        if **head == constant(&format!("{PREFIX}.W{w}.Signed")) {
            let [word] = args.as_slice() else {
                return Err(GoOrdinaryError::Linkage);
            };
            return Ok(Equality::natural(
                part(negative, x.clone()),
                if negative {
                    mul(weight(depth(w)?), bit_value(sign(w, word.clone())?))
                } else {
                    value(depth(w)?, word.clone())
                },
                numeric(&format!("W{w}.Signed{label}"), vec![word.clone()]),
            ));
        }
    }
    if **head == constant("Mpk.Go.Practical.Int.FromNat") {
        let [n] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        return Ok(Equality::natural(
            part(negative, x.clone()),
            if negative { zero() } else { n.clone() },
            int(&format!("{label}OfFromNat"), vec![n.clone()]),
        ));
    }
    if **head == constant("Mpk.Go.Practical.Int.Make") {
        let [p, n] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        return Ok(Equality::natural(
            part(negative, x.clone()),
            if negative { n.clone() } else { p.clone() },
            int(&format!("{label}OfMake"), vec![p.clone(), n.clone()]),
        ));
    }
    if **head == constant("Mpk.Go.Practical.Int.Add") {
        let [a, b] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        let pa = projection(negative, a.clone())?;
        let pb = projection(negative, b.clone())?;
        return Equality::natural(
            part(negative, x.clone()),
            add(pa.left.clone(), pb.left.clone()),
            int(&format!("{label}OfAdd"), vec![a.clone(), b.clone()]),
        )
        .then(pa.clone().on_left(pb.left.clone()))?
        .then(pb.on_right(pa.right));
    }
    if **head == constant("Mpk.Go.Practical.Int.Mul") {
        let [a, b] = args.as_slice() else {
            return Err(GoOrdinaryError::Linkage);
        };
        let ap = projection(false, a.clone())?;
        let an = projection(true, a.clone())?;
        let bp = projection(false, b.clone())?;
        let bn = projection(true, b.clone())?;
        let (l, r) = if negative {
            (product(ap, bn)?, product(an, bp)?)
        } else {
            (product(ap, bp)?, product(an, bn)?)
        };
        return Equality::natural(
            part(negative, x.clone()),
            add(l.left.clone(), r.left.clone()),
            int(&format!("{label}OfMul"), vec![a.clone(), b.clone()]),
        )
        .then(l.clone().on_left(r.left.clone()))?
        .then(r.on_right(l.right));
    }
    Err(GoOrdinaryError::Linkage)
}
pub(super) fn cross(a: Expr, b: Expr) -> Result<(Equality, Equality)> {
    let ap = projection(false, a.clone())?;
    let an = projection(true, a)?;
    let bp = projection(false, b.clone())?;
    let bn = projection(true, b)?;
    Ok((
        ap.clone()
            .on_left(bn.left.clone())
            .then(bn.on_right(ap.right))?,
        bp.clone()
            .on_left(an.left.clone())
            .then(an.on_right(bp.right))?,
    ))
}
