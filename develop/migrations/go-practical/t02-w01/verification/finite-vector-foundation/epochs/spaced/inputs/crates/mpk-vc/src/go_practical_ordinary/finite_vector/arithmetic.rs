//! Universal address bounds and cancellation, using the existing Nat laws.
use super::*;

fn commute(a: Expr, b: Expr) -> Equality {
    Equality::natural(
        add(a.clone(), b.clone()),
        add(b.clone(), a.clone()),
        nc("AddComm", vec![a, b]),
    )
}
fn associate(a: Expr, b: Expr, c: Expr) -> Equality {
    Equality::natural(
        add(add(a.clone(), b.clone()), c.clone()),
        add(a.clone(), add(b.clone(), c.clone())),
        nc("AddAssoc", vec![a, b, c]),
    )
}
fn add_zero(a: Expr) -> Equality {
    Equality::natural(add(a.clone(), zero()), a.clone(), nc("AddZero", vec![a]))
}
fn less_left(e: Equality, right: Expr) -> Equality {
    e.clone().mapped(
        lam("_value", nat(), less(var("_value"), right.clone())),
        less(e.left, right.clone()),
        less(e.right, right),
        boolean(),
    )
}
fn less_right(left: Expr, e: Equality) -> Equality {
    e.clone().mapped(
        lam("_value", nat(), less(left.clone(), var("_value"))),
        less(left.clone(), e.left),
        less(left, e.right),
        boolean(),
    )
}
pub(super) fn unshift(half: Expr, small: Expr) -> Result<Equality> {
    let from = sub(add(half.clone(), small.clone()), half.clone());
    let middle = sub(add(small.clone(), half.clone()), half.clone());
    commute(half.clone(), small.clone())
        .mapped(
            lam("_value", nat(), sub(var("_value"), half.clone())),
            from,
            middle.clone(),
            nat(),
        )
        .then(Equality::natural(
            middle,
            small.clone(),
            nc("SubAddRight", vec![small, half]),
        ))
}
pub(super) fn not_below(half: Expr, small: Expr) -> Result<Equality> {
    let zero_plus = commute(zero(), half.clone()).then(add_zero(half.clone()))?;
    less_left(commute(half.clone(), small.clone()), half.clone())
        .then(less_right(
            add(small.clone(), half.clone()),
            zero_plus.reverse(),
        ))?
        .then(Equality::boolean(
            less(add(small.clone(), half.clone()), add(zero(), half.clone())),
            less(small.clone(), zero()),
            nc("LessTranslate", vec![small.clone(), zero(), half]),
        ))?
        .then(Equality::boolean(
            less(small.clone(), zero()),
            bit(false),
            nc("LessZero", vec![small]),
        ))
}

pub(super) fn bounds(vectors: &mut Vectors<'_>, d: usize) -> Result<()> {
    let (address, padding) = (bits(d), var("padding"));
    let property = |address: &[Expr]| inside(index(d, address), add(capacity(d), padding.clone()));
    let proof = if d == 0 {
        let spacing = var("spacing");
        let successor = Equality::natural(
            add(succ(spacing.clone()), padding.clone()),
            succ(add(spacing.clone(), padding.clone())),
            nc("SuccAdd", vec![spacing.clone(), padding.clone()]),
        );
        less_right(zero(), successor)
            .then(Equality::boolean(
                less(zero(), succ(add(spacing.clone(), padding.clone()))),
                bit(true),
                nc("LessZeroSucc", vec![add(spacing, padding.clone())]),
            ))?
            .proof
    } else {
        let (half, small) = (capacity(d - 1), index(d - 1, &address[1..]));
        let previous = |padding: Expr| {
            let mut args = address[1..].to_vec();
            args.push(padding);
            fc(d - 1, "IndexPaddedBound", args)
        };
        let false_proof = less_right(
            small.clone(),
            associate(half.clone(), half.clone(), padding.clone()),
        )
        .then(Equality::boolean(
            less(
                small.clone(),
                add(half.clone(), add(half.clone(), padding.clone())),
            ),
            bit(true),
            previous(add(half.clone(), padding.clone())),
        ))?
        .proof;
        let true_proof = less_left(
            commute(half.clone(), small.clone()),
            add(add(half.clone(), half.clone()), padding.clone()),
        )
        .then(less_right(
            add(small.clone(), half.clone()),
            associate(half.clone(), half.clone(), padding.clone()),
        ))?
        .then(less_right(
            add(small.clone(), half.clone()),
            commute(half.clone(), add(half.clone(), padding.clone())),
        ))?
        .then(Equality::boolean(
            less(
                add(small.clone(), half.clone()),
                add(add(half.clone(), padding.clone()), half.clone()),
            ),
            less(small.clone(), add(half.clone(), padding.clone())),
            nc(
                "LessTranslate",
                vec![small.clone(), add(half.clone(), padding.clone()), half],
            ),
        ))?
        .then(Equality::boolean(
            less(small, add(capacity(d - 1), padding.clone())),
            bit(true),
            previous(padding.clone()),
        ))?
        .proof;
        let mut branch = address.clone();
        branch[0] = var("_head");
        call(
            "Std.Bool.cases",
            vec![
                lam("_head", boolean(), property(&branch)),
                false_proof,
                true_proof,
                address[0].clone(),
            ],
        )
    };
    let mut args = bit_args(d);
    args.push(("padding", nat()));
    vectors.emit(
        d,
        "IndexPaddedBound",
        &args,
        property(&address),
        proof,
        true,
    )?;
    let mut padded = address.clone();
    padded.push(zero());
    let proof = less_right(index(d, &address), add_zero(capacity(d)))
        .reverse()
        .then(Equality::boolean(
            less(index(d, &address), add(capacity(d), zero())),
            bit(true),
            fc(d, "IndexPaddedBound", padded),
        ))?
        .proof;
    vectors.emit(
        d,
        "IndexBound",
        &bit_args(d),
        inside(index(d, &address), capacity(d)),
        proof,
        true,
    )
}
