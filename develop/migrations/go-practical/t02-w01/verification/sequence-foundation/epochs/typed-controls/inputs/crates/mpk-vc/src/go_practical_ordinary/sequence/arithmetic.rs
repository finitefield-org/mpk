//! The actual right-operand bound for an unbounded indexed concatenation.
//! This is Nat induction, with no operand enumeration or source truth flag.
use super::*;

fn zero_add(value: Expr) -> Result<Equality> {
    Equality::natural(
        add(zero(), value.clone()),
        add(value.clone(), zero()),
        nc("AddComm", vec![zero(), value.clone()]),
    )
    .then(Equality::natural(
        add(value.clone(), zero()),
        value.clone(),
        nc("AddZero", vec![value]),
    ))
}
fn successor_add(left: Expr, right: Expr) -> Result<Equality> {
    Equality::natural(
        add(succ(left.clone()), right.clone()),
        add(right.clone(), succ(left.clone())),
        nc("AddComm", vec![succ(left.clone()), right.clone()]),
    )
    .then(Equality::natural(
        add(right.clone(), succ(left.clone())),
        succ(add(right.clone(), left.clone())),
        nc("AddSucc", vec![right.clone(), left.clone()]),
    ))?
    .then(
        Equality::natural(
            add(right.clone(), left.clone()),
            add(left.clone(), right.clone()),
            nc("AddComm", vec![right, left]),
        )
        .on_succ(),
    )
}
fn less_left(relation: Equality, right: Expr) -> Equality {
    let from = less(relation.left.clone(), right.clone());
    let to = less(relation.right.clone(), right.clone());
    relation.mapped(
        lam("_value", nat(), less(var("_value"), right)),
        from,
        to,
        boolean(),
    )
}
fn less_right(left: Expr, relation: Equality) -> Equality {
    let from = less(left.clone(), relation.left.clone());
    let to = less(left.clone(), relation.right.clone());
    relation.mapped(
        lam("_value", nat(), less(left, var("_value"))),
        from,
        to,
        boolean(),
    )
}
fn statement(n: Expr, i: Expr, m: Expr) -> Expr {
    pi(
        "total",
        inside(i.clone(), add(n.clone(), m.clone())),
        pi(
            "right",
            eq(boolean(), less(i.clone(), n.clone()), bit(false)),
            inside(sub(i, n), m),
        ),
    )
}
fn at_index(n: Expr, i: Expr) -> Expr {
    pi("m", nat(), statement(n, i, var("m")))
}
fn property(n: Expr) -> Expr {
    pi("i", nat(), at_index(n, var("i")))
}
fn assumptions(n: Expr, i: Expr, m: Expr, body: Expr) -> Expr {
    lam(
        "total",
        inside(i.clone(), add(n.clone(), m)),
        lam("right", eq(boolean(), less(i, n), bit(false)), body),
    )
}

pub(super) fn append(sequences: &mut Sequences<'_>) -> Result<()> {
    let (n, i, m) = (var("_n"), var("_i"), var("m"));
    let start = less_right(var("i"), zero_add(m.clone())?)
        .reverse()
        .then(Equality::boolean(
            less(var("i"), add(zero(), m.clone())),
            bit(true),
            var("total"),
        ))?;
    let sub_zero = Equality::natural(
        sub(var("i"), zero()),
        var("i"),
        nc("SubZero", vec![var("i")]),
    );
    let base_proof = less_left(sub_zero, m.clone()).then(start)?.proof;
    let base = lam(
        "i",
        nat(),
        lam(
            "m",
            nat(),
            assumptions(zero(), var("i"), m.clone(), base_proof),
        ),
    );

    let zero_compare = less(zero(), succ(n.clone()));
    let contradiction = Equality::boolean(
        zero_compare.clone(),
        bit(true),
        nc("LessZeroSucc", vec![n.clone()]),
    )
    .reverse()
    .then(Equality::boolean(zero_compare, bit(false), var("right")))?
    .reverse()
    .proof;
    let impossible_result = inside(sub(zero(), succ(n.clone())), m.clone());
    let index_base = lam(
        "m",
        nat(),
        assumptions(
            succ(n.clone()),
            zero(),
            m.clone(),
            bc("FalseElim", vec![impossible_result, contradiction]),
        ),
    );

    let total_compare = less_right(succ(i.clone()), successor_add(n.clone(), m.clone())?).then(
        Equality::boolean(
            less(succ(i.clone()), succ(add(n.clone(), m.clone()))),
            less(i.clone(), add(n.clone(), m.clone())),
            nc("LessSuccBoth", vec![i.clone(), add(n.clone(), m.clone())]),
        ),
    )?;
    let total = total_compare
        .reverse()
        .then(Equality::boolean(
            less(succ(i.clone()), add(succ(n.clone()), m.clone())),
            bit(true),
            var("total"),
        ))?
        .proof;
    let branch = Equality::boolean(
        less(succ(i.clone()), succ(n.clone())),
        less(i.clone(), n.clone()),
        nc("LessSuccBoth", vec![i.clone(), n.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        less(succ(i.clone()), succ(n.clone())),
        bit(false),
        var("right"),
    ))?
    .proof;
    let previous = app(var("_outer"), vec![i.clone(), m.clone(), total, branch]);
    let subtraction = Equality::natural(
        sub(succ(i.clone()), succ(n.clone())),
        sub(i.clone(), n.clone()),
        nc("SubSuccBoth", vec![i.clone(), n.clone()]),
    );
    let step_proof = less_left(subtraction, m.clone())
        .then(Equality::boolean(
            less(sub(i.clone(), n.clone()), m.clone()),
            bit(true),
            previous,
        ))?
        .proof;
    let index_step = lam(
        "_i",
        nat(),
        lam(
            "_unused",
            at_index(succ(n.clone()), i),
            lam(
                "m",
                nat(),
                assumptions(succ(n.clone()), succ(var("_i")), m, step_proof),
            ),
        ),
    );
    let step = lam(
        "_n",
        nat(),
        lam(
            "_outer",
            property(n.clone()),
            lam(
                "i",
                nat(),
                induct(
                    "_i",
                    at_index(succ(n), var("_i")),
                    index_base,
                    index_step,
                    var("i"),
                ),
            ),
        ),
    );
    sequences.emit(
        "AppendRightBound",
        &[("n", nat())],
        property(var("n")),
        induct("_n", property(var("_n")), base, step, var("n")),
        true,
    )
}
