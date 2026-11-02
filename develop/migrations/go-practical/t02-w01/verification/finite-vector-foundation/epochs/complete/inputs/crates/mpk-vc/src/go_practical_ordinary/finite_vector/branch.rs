//! Abstract one-level observations; fixed-depth laws compose their equalities.
//! Symbolic sizes keep complete 256-slot proofs within the unchanged fuel cap.
use super::*;

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let (a, half, lo, hi, k, head) = (
        var("A"),
        var("half"),
        var("lo"),
        var("hi"),
        var("k"),
        var("head"),
    );
    let read = |f: Expr, i| app(f, vec![i]);
    let branch = |i| {
        gc(
            "Branch",
            vec![a.clone(), half.clone(), lo.clone(), hi.clone(), i],
        )
    };
    let args = [
        ("A", Expr::Sort0),
        ("half", nat()),
        ("lo", function(a.clone())),
        ("hi", function(a.clone())),
        ("i", nat()),
    ];
    emit(
        builder,
        "Mpk.Go.Practical.FiniteVector.Branch",
        &args,
        a.clone(),
        choose(
            a.clone(),
            less(var("i"), half.clone()),
            read(lo.clone(), var("i")),
            read(hi.clone(), sub(var("i"), half.clone())),
        ),
        false,
    )?;
    let shifted = add(half.clone(), k.clone());
    let no_proof = sc(
        "ChoiceTrue",
        vec![
            a.clone(),
            less(k.clone(), half.clone()),
            read(lo.clone(), k.clone()),
            read(hi.clone(), sub(k.clone(), half.clone())),
            var("bound"),
        ],
    );
    let high = read(hi.clone(), sub(shifted.clone(), half.clone()));
    let yes_proof = Equality {
        ty: a.clone(),
        left: branch(shifted.clone()),
        right: high.clone(),
        proof: sc(
            "ChoiceFalse",
            vec![
                a.clone(),
                less(shifted.clone(), half.clone()),
                read(lo.clone(), shifted.clone()),
                high.clone(),
                arithmetic::not_below(half.clone(), k.clone())?.proof,
            ],
        ),
    }
    .then(arithmetic::unshift(half.clone(), k.clone())?.mapped(
        lam("_index", nat(), read(hi.clone(), var("_index"))),
        high,
        read(hi.clone(), k.clone()),
        a.clone(),
    ))?
    .proof;
    let statement = |head: Expr| {
        eq(
            a.clone(),
            branch(choose(nat(), head.clone(), shifted.clone(), k.clone())),
            choose(
                a.clone(),
                head,
                read(hi.clone(), k.clone()),
                read(lo.clone(), k.clone()),
            ),
        )
    };
    emit(
        builder,
        "Mpk.Go.Practical.FiniteVector.BranchIndex",
        &[
            ("A", Expr::Sort0),
            ("half", nat()),
            ("lo", function(a.clone())),
            ("hi", function(a.clone())),
            ("k", nat()),
            ("head", boolean()),
            ("bound", inside(k.clone(), half.clone())),
        ],
        statement(head.clone()),
        call(
            "Std.Bool.cases",
            vec![
                lam("_head", boolean(), statement(var("_head"))),
                no_proof,
                yes_proof,
                head.clone(),
            ],
        ),
        true,
    )?;

    let p = var("P");
    let valid = |f: Expr| {
        pi(
            "_point",
            nat(),
            pi(
                "_in",
                inside(var("_point"), half.clone()),
                app(p.clone(), vec![read(f, var("_point"))]),
            ),
        )
    };
    let i = var("i");
    let condition = less(i.clone(), half.clone());
    let right_index = sub(i.clone(), half.clone());
    let motive = lam(
        "_flag",
        boolean(),
        pi(
            "_branch",
            eq(boolean(), condition.clone(), var("_flag")),
            app(
                p.clone(),
                vec![choose(
                    a.clone(),
                    var("_flag"),
                    read(lo.clone(), i.clone()),
                    read(hi.clone(), right_index.clone()),
                )],
            ),
        ),
    );
    let right_bound = sc(
        "AppendRightBound",
        vec![
            half.clone(),
            i.clone(),
            half.clone(),
            var("bound"),
            var("_branch"),
        ],
    );
    let right = lam(
        "_branch",
        eq(boolean(), condition.clone(), bit(false)),
        app(var("right"), vec![right_index, right_bound]),
    );
    let left = lam(
        "_branch",
        eq(boolean(), condition.clone(), bit(true)),
        app(var("left"), vec![i.clone(), var("_branch")]),
    );
    let proof = app(
        call(
            "Std.Bool.cases",
            vec![motive, right, left, condition.clone()],
        ),
        vec![refl(boolean(), condition)],
    );
    emit(
        builder,
        "Mpk.Go.Practical.FiniteVector.BranchValid",
        &[
            ("A", Expr::Sort0),
            ("P", predicate(a.clone())),
            ("half", nat()),
            ("lo", function(a.clone())),
            ("hi", function(a.clone())),
            ("left", valid(lo.clone())),
            ("right", valid(hi.clone())),
            ("i", nat()),
            ("bound", inside(i.clone(), add(half.clone(), half.clone()))),
        ],
        app(p, vec![branch(i)]),
        proof,
        true,
    )?;

    let f = var("f");
    let selection = |b: Expr| {
        eq(
            a.clone(),
            choose(
                a.clone(),
                b.clone(),
                app(f.clone(), vec![bit(true)]),
                app(f.clone(), vec![bit(false)]),
            ),
            app(f.clone(), vec![b]),
        )
    };
    emit(
        builder,
        "Mpk.Go.Practical.FiniteVector.SelectAt",
        &[
            ("A", Expr::Sort0),
            ("f", pi("_head", boolean(), a.clone())),
            ("head", boolean()),
        ],
        selection(head.clone()),
        call(
            "Std.Bool.cases",
            vec![
                lam("_head", boolean(), selection(var("_head"))),
                refl(a.clone(), app(f.clone(), vec![bit(false)])),
                refl(a, app(f, vec![bit(true)])),
                head,
            ],
        ),
        true,
    )?;
    arithmetic::padded_step(builder)
}
