//! Unbounded upper-limit preservation, proved by nested Nat induction.
use super::*;
fn assumptions(c: Expr, n: Expr, i: Expr, body: Expr) -> Expr {
    lam(
        "_upper",
        upper(n.clone(), c.clone()),
        lam("_within", inside(i, n), body),
    )
}
fn statement(c: Expr, n: Expr, i: Expr) -> Expr {
    pi(
        "_upper",
        upper(n.clone(), c.clone()),
        pi("_within", inside(i.clone(), n), inside(i, c)),
    )
}
fn at_count(c: Expr, n: Expr) -> Expr {
    pi("i", nat(), statement(c, n, var("i")))
}
fn at_capacity(c: Expr) -> Expr {
    pi("n", nat(), at_count(c, var("n")))
}
fn empty(c: Expr) -> Result<Expr> {
    let i = var("i");
    let contradiction = Equality::boolean(
        less(i.clone(), zero()),
        bit(false),
        nc("LessZero", vec![i.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        less(i.clone(), zero()),
        bit(true),
        var("_within"),
    ))?;
    Ok(lam(
        "i",
        nat(),
        assumptions(
            c.clone(),
            zero(),
            i.clone(),
            bc("FalseElim", vec![inside(i, c), contradiction.proof]),
        ),
    ))
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let (c, n, i) = (var("_capacity"), var("_count"), var("_index"));
    let contradiction = Equality::boolean(
        less(zero(), succ(n.clone())),
        bit(true),
        nc("LessZeroSucc", vec![n.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        less(zero(), succ(n.clone())),
        bit(false),
        var("_upper"),
    ))?
    .reverse();
    let base_step = lam(
        "_count",
        nat(),
        lam(
            "_ignored",
            at_count(zero(), n.clone()),
            lam(
                "i",
                nat(),
                assumptions(
                    zero(),
                    succ(n.clone()),
                    var("i"),
                    bc(
                        "FalseElim",
                        vec![inside(var("i"), zero()), contradiction.proof],
                    ),
                ),
            ),
        ),
    );
    let base = lam(
        "n",
        nat(),
        induct(
            "_count",
            at_count(zero(), n.clone()),
            empty(zero())?,
            base_step,
            var("n"),
        ),
    );
    let previous_upper = Equality::boolean(
        less(succ(c.clone()), succ(n.clone())),
        less(c.clone(), n.clone()),
        nc("LessSuccBoth", vec![c.clone(), n.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        less(succ(c.clone()), succ(n.clone())),
        bit(false),
        var("_upper"),
    ))?;
    let previous_within = Equality::boolean(
        less(succ(i.clone()), succ(n.clone())),
        less(i.clone(), n.clone()),
        nc("LessSuccBoth", vec![i.clone(), n.clone()]),
    )
    .reverse()
    .then(Equality::boolean(
        less(succ(i.clone()), succ(n.clone())),
        bit(true),
        var("_within"),
    ))?;
    let previous = app(
        var("_prior"),
        vec![
            n.clone(),
            i.clone(),
            previous_upper.proof,
            previous_within.proof,
        ],
    );
    let result = Equality::boolean(
        less(succ(i.clone()), succ(c.clone())),
        less(i.clone(), c.clone()),
        nc("LessSuccBoth", vec![i.clone(), c.clone()]),
    )
    .then(Equality::boolean(
        less(i.clone(), c.clone()),
        bit(true),
        previous,
    ))?;
    let index_step = lam(
        "_index",
        nat(),
        lam(
            "_unused",
            statement(succ(c.clone()), succ(n.clone()), i.clone()),
            assumptions(
                succ(c.clone()),
                succ(n.clone()),
                succ(i.clone()),
                result.proof,
            ),
        ),
    );
    let index_base = assumptions(
        succ(c.clone()),
        succ(n.clone()),
        zero(),
        nc("LessZeroSucc", vec![c.clone()]),
    );
    let count_step = lam(
        "_count",
        nat(),
        lam(
            "_unused_count",
            at_count(succ(c.clone()), n.clone()),
            lam(
                "i",
                nat(),
                induct(
                    "_index",
                    statement(succ(c.clone()), succ(n.clone()), i.clone()),
                    index_base,
                    index_step,
                    var("i"),
                ),
            ),
        ),
    );
    let step = lam(
        "_capacity",
        nat(),
        lam(
            "_prior",
            at_capacity(c.clone()),
            lam(
                "n",
                nat(),
                induct(
                    "_count",
                    at_count(succ(c.clone()), n.clone()),
                    empty(succ(c))?,
                    count_step,
                    var("n"),
                ),
            ),
        ),
    );
    emit(
        builder,
        "Mpk.Go.Practical.Word.Index.UpperBound",
        &[("capacity", nat())],
        at_capacity(var("capacity")),
        induct(
            "_capacity",
            at_capacity(var("_capacity")),
            base,
            step,
            var("capacity"),
        ),
        true,
    )
}
