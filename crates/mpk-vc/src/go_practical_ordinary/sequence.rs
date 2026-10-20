//! Ordinary indexed sequences with explicit lengths and prefix observations.
//! Data outside the observed prefix grants no source access. Full Go carrier,
//! word-index, domain, source, invocation and storage correspondence remain due.
use super::syntax::*;
use super::{Builder, Result};

mod arithmetic;

fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Sequence.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn function(a: Expr, b: Expr) -> Expr {
    pi("_argument", a, b)
}
fn data_type(a: Expr) -> Expr {
    function(nat(), a)
}
fn sequence(a: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call("Std.Bool.recType", vec![nat(), data_type(a), var("_tag")]),
    )
}
fn make(a: Expr, n: Expr, f: Expr) -> Expr {
    sc("Make", vec![a, n, f])
}
fn length(a: Expr, s: Expr) -> Expr {
    sc("Length", vec![a, s])
}
fn at(a: Expr, s: Expr, i: Expr) -> Expr {
    sc("At", vec![a, s, i])
}
fn inside(i: Expr, n: Expr) -> Expr {
    eq(boolean(), less(i, n), bit(true))
}
fn prefix(a: Expr, n: Expr, s: Expr, t: Expr) -> Expr {
    pi(
        "_point",
        nat(),
        pi(
            "_in",
            inside(var("_point"), n),
            eq(
                a.clone(),
                at(a.clone(), s, var("_point")),
                at(a, t, var("_point")),
            ),
        ),
    )
}
fn validity(a: Expr, p: Expr, s: Expr) -> Expr {
    pi(
        "_point",
        nat(),
        pi(
            "_in",
            inside(var("_point"), length(a.clone(), s.clone())),
            app(p, vec![at(a, s, var("_point"))]),
        ),
    )
}
fn within(a: Expr, s: Expr, lower: Expr, count: Expr) -> Expr {
    pi(
        "_point",
        nat(),
        pi(
            "_in",
            inside(var("_point"), count),
            inside(add(lower, var("_point")), length(a, s)),
        ),
    )
}
fn choose(a: Expr, b: Expr, yes: Expr, no: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_case", boolean(), a), no, yes, b],
    )
}
fn mapped(a: Expr, b: Expr, f: Expr, s: Expr) -> Expr {
    sc("Map", vec![a, b, f, s])
}
fn set(a: Expr, s: Expr, index: Expr, value: Expr) -> Expr {
    sc("Set", vec![a, s, index, value])
}
fn offset(a: Expr, s: Expr, lower: Expr, count: Expr) -> Expr {
    sc("Offset", vec![a, s, lower, count])
}
fn appended(a: Expr, s: Expr, t: Expr) -> Expr {
    sc("Append", vec![a, s, t])
}
fn points(n: Expr, body: Expr) -> Expr {
    lam("_point", nat(), lam("_in", inside(var("_point"), n), body))
}

struct Sequences<'a>(&'a mut Builder);
impl Sequences<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.Sequence.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn definitions(&mut self) -> Result<()> {
        let (a, s, t, n, i) = (var("A"), var("s"), var("t"), var("n"), var("i"));
        self.emit(
            "Type",
            &[("A", Expr::Sort0)],
            Expr::Sort0,
            sequence(a.clone()),
            false,
        )?;
        self.emit(
            "TypeAsFunction",
            &[("A", Expr::Sort0), ("s", sc("Type", vec![a.clone()]))],
            sequence(a.clone()),
            s.clone(),
            false,
        )?;
        self.emit(
            "Make",
            &[
                ("A", Expr::Sort0),
                ("n", nat()),
                ("f", data_type(a.clone())),
            ],
            sequence(a.clone()),
            pc(
                "Make",
                vec![nat(), data_type(a.clone()), n.clone(), var("f")],
            ),
            false,
        )?;
        let input = [("A", Expr::Sort0), ("s", sequence(a.clone()))];
        self.emit(
            "Length",
            &input,
            nat(),
            pc("Left", vec![nat(), data_type(a.clone()), s.clone()]),
            false,
        )?;
        let mut indexed = input.to_vec();
        indexed.push(("i", nat()));
        self.emit(
            "At",
            &indexed,
            a.clone(),
            app(
                pc("Right", vec![nat(), data_type(a.clone()), s.clone()]),
                vec![i.clone()],
            ),
            false,
        )?;
        self.emit(
            "IndexValid",
            &indexed,
            Expr::Sort0,
            inside(i.clone(), length(a.clone(), s.clone())),
            false,
        )?;
        let related = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("s", sequence(a.clone())),
            ("t", sequence(a.clone())),
        ];
        self.emit(
            "SamePrefix",
            &related,
            Expr::Sort0,
            prefix(a.clone(), n.clone(), s.clone(), t.clone()),
            false,
        )?;
        let mut observed = related.to_vec();
        observed.push((
            "h",
            sc(
                "SamePrefix",
                vec![a.clone(), n.clone(), s.clone(), t.clone()],
            ),
        ));
        self.emit(
            "SamePrefixAsFunction",
            &observed,
            prefix(a.clone(), n.clone(), s.clone(), t.clone()),
            var("h"),
            false,
        )?;
        let valid = [
            ("A", Expr::Sort0),
            ("P", function(a.clone(), Expr::Sort0)),
            ("s", sequence(a.clone())),
        ];
        self.emit(
            "Valid",
            &valid,
            Expr::Sort0,
            validity(a.clone(), var("P"), s.clone()),
            false,
        )?;
        let mut exposed = valid.to_vec();
        exposed.push(("h", sc("Valid", vec![a.clone(), var("P"), s.clone()])));
        self.emit(
            "ValidAsFunction",
            &exposed,
            validity(a.clone(), var("P"), s.clone()),
            var("h"),
            false,
        )?;
        self.emit(
            "Constant",
            &[("A", Expr::Sort0), ("n", nat()), ("value", a.clone())],
            sequence(a.clone()),
            make(a.clone(), n.clone(), lam("_index", nat(), var("value"))),
            false,
        )?;
        self.emit(
            "Map",
            &[
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("f", function(a.clone(), var("B"))),
                ("s", sequence(a.clone())),
            ],
            sequence(var("B")),
            make(
                var("B"),
                length(a.clone(), s.clone()),
                lam(
                    "_index",
                    nat(),
                    app(var("f"), vec![at(a.clone(), s.clone(), var("_index"))]),
                ),
            ),
            false,
        )?;
        self.emit(
            "Set",
            &[
                ("A", Expr::Sort0),
                ("s", sequence(a.clone())),
                ("index", nat()),
                ("value", a.clone()),
            ],
            sequence(a.clone()),
            make(
                a.clone(),
                length(a.clone(), s.clone()),
                lam(
                    "_index",
                    nat(),
                    choose(
                        a.clone(),
                        equal(var("_index"), var("index")),
                        var("value"),
                        at(a.clone(), s.clone(), var("_index")),
                    ),
                ),
            ),
            false,
        )?;
        let sliced = [
            ("A", Expr::Sort0),
            ("s", sequence(a.clone())),
            ("lower", nat()),
            ("count", nat()),
        ];
        self.emit(
            "Offset",
            &sliced,
            sequence(a.clone()),
            make(
                a.clone(),
                var("count"),
                lam(
                    "_index",
                    nat(),
                    at(a.clone(), s.clone(), add(var("lower"), var("_index"))),
                ),
            ),
            false,
        )?;
        self.emit(
            "Within",
            &sliced,
            Expr::Sort0,
            within(a.clone(), s.clone(), var("lower"), var("count")),
            false,
        )?;
        self.emit(
            "Append",
            &[
                ("A", Expr::Sort0),
                ("s", sequence(a.clone())),
                ("t", sequence(a.clone())),
            ],
            sequence(a.clone()),
            make(
                a.clone(),
                add(length(a.clone(), s.clone()), length(a.clone(), t.clone())),
                lam(
                    "_index",
                    nat(),
                    choose(
                        a.clone(),
                        less(var("_index"), length(a.clone(), s.clone())),
                        at(a.clone(), s.clone(), var("_index")),
                        at(a, t, sub(var("_index"), length(var("A"), s))),
                    ),
                ),
            ),
            false,
        )
    }

    fn choices(&mut self) -> Result<()> {
        let (a, c, yes, no) = (var("A"), var("c"), var("yes"), var("no"));
        for branch in [false, true] {
            let result = if branch { yes.clone() } else { no.clone() };
            let args = [
                ("A", Expr::Sort0),
                ("c", boolean()),
                ("yes", a.clone()),
                ("no", a.clone()),
                ("h", eq(boolean(), c.clone(), bit(branch))),
            ];
            let proof = Equality::boolean(c.clone(), bit(branch), var("h"))
                .mapped(
                    lam(
                        "_choice",
                        boolean(),
                        choose(a.clone(), var("_choice"), yes.clone(), no.clone()),
                    ),
                    choose(a.clone(), c.clone(), yes.clone(), no.clone()),
                    result.clone(),
                    a.clone(),
                )
                .proof;
            self.emit(
                if branch { "ChoiceTrue" } else { "ChoiceFalse" },
                &args,
                eq(
                    a.clone(),
                    choose(a.clone(), c.clone(), yes.clone(), no.clone()),
                    result,
                ),
                proof,
                true,
            )?;
        }
        Ok(())
    }

    fn observations(&mut self) -> Result<()> {
        let (a, s, t, i, n) = (var("A"), var("s"), var("t"), var("i"), var("n"));
        let mut equations: Vec<(&str, Vec<Binder>, Expr, Expr, Expr)> = vec![];
        let base = vec![("A", Expr::Sort0), ("s", sequence(a.clone()))];
        let mut add_case = |name, args: Vec<Binder>, value, len, observed| {
            equations.push((name, args, value, len, observed));
        };
        add_case(
            "Make",
            vec![
                ("A", Expr::Sort0),
                ("n", nat()),
                ("f", data_type(a.clone())),
            ],
            make(a.clone(), n.clone(), var("f")),
            n.clone(),
            app(var("f"), vec![i.clone()]),
        );
        add_case(
            "Constant",
            vec![("A", Expr::Sort0), ("n", nat()), ("value", a.clone())],
            sc("Constant", vec![a.clone(), n.clone(), var("value")]),
            n.clone(),
            var("value"),
        );
        add_case(
            "Map",
            vec![
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("f", function(var("B"), a.clone())),
                ("s", sequence(var("B"))),
            ],
            mapped(var("B"), a.clone(), var("f"), s.clone()),
            length(var("B"), s.clone()),
            app(var("f"), vec![at(var("B"), s.clone(), i.clone())]),
        );
        let mut changed = base.clone();
        changed.extend([("index", nat()), ("value", a.clone())]);
        add_case(
            "Set",
            changed,
            set(a.clone(), s.clone(), var("index"), var("value")),
            length(a.clone(), s.clone()),
            choose(
                a.clone(),
                equal(i.clone(), var("index")),
                var("value"),
                at(a.clone(), s.clone(), i.clone()),
            ),
        );
        let mut sliced = base.clone();
        sliced.extend([("lower", nat()), ("count", nat())]);
        add_case(
            "Offset",
            sliced,
            offset(a.clone(), s.clone(), var("lower"), var("count")),
            var("count"),
            at(a.clone(), s.clone(), add(var("lower"), i.clone())),
        );
        let mut combined = base;
        combined.push(("t", sequence(a.clone())));
        add_case(
            "Append",
            combined,
            appended(a.clone(), s.clone(), t.clone()),
            add(length(a.clone(), s.clone()), length(a.clone(), t.clone())),
            choose(
                a.clone(),
                less(i.clone(), length(a.clone(), s.clone())),
                at(a.clone(), s.clone(), i.clone()),
                at(a.clone(), t, sub(i.clone(), length(a.clone(), s.clone()))),
            ),
        );
        for (name, mut args, value, len, observed) in equations {
            self.emit(
                &format!("{name}Length"),
                &args,
                eq(nat(), length(a.clone(), value.clone()), len.clone()),
                refl(nat(), len),
                true,
            )?;
            args.push(("i", nat()));
            self.emit(
                &format!("{name}At"),
                &args,
                eq(a.clone(), at(a.clone(), value, i.clone()), observed.clone()),
                refl(a.clone(), observed),
                true,
            )?;
        }
        Ok(())
    }

    fn relations(&mut self) -> Result<()> {
        let (a, n, s, t, u, i) = (
            var("A"),
            var("n"),
            var("s"),
            var("t"),
            var("u"),
            var("_point"),
        );
        let args = [("A", Expr::Sort0), ("n", nat()), ("s", sequence(a.clone()))];
        self.emit(
            "PrefixReflexive",
            &args,
            prefix(a.clone(), n.clone(), s.clone(), s.clone()),
            points(
                n.clone(),
                refl(a.clone(), at(a.clone(), s.clone(), i.clone())),
            ),
            true,
        )?;
        let mut symmetric = args.to_vec();
        symmetric.extend([
            ("t", sequence(a.clone())),
            ("h", prefix(a.clone(), n.clone(), s.clone(), t.clone())),
        ]);
        let observation = app(var("h"), vec![i.clone(), var("_in")]);
        self.emit(
            "PrefixSymmetric",
            &symmetric,
            prefix(a.clone(), n.clone(), t.clone(), s.clone()),
            points(
                n.clone(),
                call(
                    "Std.Eq.symm",
                    vec![
                        a.clone(),
                        at(a.clone(), s.clone(), i.clone()),
                        at(a.clone(), t.clone(), i.clone()),
                        observation,
                    ],
                ),
            ),
            true,
        )?;
        let mut transitive = args.to_vec();
        transitive.extend([
            ("t", sequence(a.clone())),
            ("u", sequence(a.clone())),
            ("left", prefix(a.clone(), n.clone(), s.clone(), t.clone())),
            ("right", prefix(a.clone(), n.clone(), t.clone(), u.clone())),
        ]);
        self.emit(
            "PrefixTransitive",
            &transitive,
            prefix(a.clone(), n.clone(), s.clone(), u.clone()),
            points(
                n.clone(),
                call(
                    "Std.Eq.trans",
                    vec![
                        a.clone(),
                        at(a.clone(), s.clone(), i.clone()),
                        at(a.clone(), t.clone(), i.clone()),
                        at(a.clone(), u, i.clone()),
                        app(var("left"), vec![i.clone(), var("_in")]),
                        app(var("right"), vec![i.clone(), var("_in")]),
                    ],
                ),
            ),
            true,
        )?;
        let mut restrict = args.to_vec();
        restrict.extend([
            ("m", nat()),
            ("t", sequence(a.clone())),
            ("h", prefix(a.clone(), n.clone(), s.clone(), t.clone())),
            (
                "bound",
                pi(
                    "_point",
                    nat(),
                    pi(
                        "_in",
                        inside(i.clone(), var("m")),
                        inside(i.clone(), n.clone()),
                    ),
                ),
            ),
        ]);
        self.emit(
            "PrefixRestrict",
            &restrict,
            prefix(a.clone(), var("m"), s, t),
            points(
                var("m"),
                app(
                    var("h"),
                    vec![i.clone(), app(var("bound"), vec![i, var("_in")])],
                ),
            ),
            true,
        )
    }

    fn maps(&mut self) -> Result<()> {
        let (a, b, c, s, t, i) = (
            var("A"),
            var("B"),
            var("C"),
            var("s"),
            var("t"),
            var("_point"),
        );
        let n = length(a.clone(), s.clone());
        let identity = mapped(
            a.clone(),
            a.clone(),
            lam("_value", a.clone(), var("_value")),
            s.clone(),
        );
        self.emit(
            "MapIdentity",
            &[("A", Expr::Sort0), ("s", sequence(a.clone()))],
            prefix(a.clone(), n.clone(), identity, s.clone()),
            points(
                n.clone(),
                refl(a.clone(), at(a.clone(), s.clone(), i.clone())),
            ),
            true,
        )?;
        let mapped_twice = mapped(
            b.clone(),
            c.clone(),
            var("g"),
            mapped(a.clone(), b.clone(), var("f"), s.clone()),
        );
        let composed = mapped(
            a.clone(),
            c.clone(),
            lam(
                "_value",
                a.clone(),
                app(var("g"), vec![app(var("f"), vec![var("_value")])]),
            ),
            s.clone(),
        );
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("f", function(a.clone(), b.clone())),
            ("g", function(b.clone(), c.clone())),
            ("s", sequence(a.clone())),
        ];
        let mut empty = args[..1].to_vec();
        empty.extend([("s", sequence(a.clone())), ("t", sequence(a.clone()))]);
        let contradiction = Equality::boolean(
            less(i.clone(), zero()),
            bit(false),
            nc("LessZero", vec![i.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(i.clone(), zero()),
            bit(true),
            var("_in"),
        ))?
        .proof;
        let goal = eq(
            a.clone(),
            at(a.clone(), s.clone(), i.clone()),
            at(a.clone(), t.clone(), i.clone()),
        );
        self.emit(
            "PrefixEmpty",
            &empty,
            prefix(a.clone(), zero(), s.clone(), t.clone()),
            points(zero(), bc("FalseElim", vec![goal, contradiction])),
            true,
        )?;
        self.emit(
            "MapCompose",
            &args,
            prefix(c.clone(), n.clone(), mapped_twice, composed),
            points(
                n,
                refl(
                    c,
                    app(
                        var("g"),
                        vec![app(var("f"), vec![at(a.clone(), s.clone(), i.clone())])],
                    ),
                ),
            ),
            true,
        )?;
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("f", function(a.clone(), b.clone())),
            ("n", nat()),
            ("s", sequence(a.clone())),
            ("t", sequence(a.clone())),
            ("h", prefix(a.clone(), var("n"), s.clone(), t.clone())),
        ];
        self.emit(
            "MapRespects",
            &args,
            prefix(
                b.clone(),
                var("n"),
                mapped(a.clone(), b.clone(), var("f"), s.clone()),
                mapped(a.clone(), b.clone(), var("f"), t.clone()),
            ),
            points(
                var("n"),
                call(
                    "Std.Eq.congr",
                    vec![
                        a.clone(),
                        b,
                        var("f"),
                        at(a.clone(), s, i.clone()),
                        at(a, t, i.clone()),
                        app(var("h"), vec![i, var("_in")]),
                    ],
                ),
            ),
            true,
        )
    }

    fn updates(&mut self) -> Result<()> {
        let (a, s, index, value, i) = (var("A"), var("s"), var("index"), var("value"), var("i"));
        let args = [
            ("A", Expr::Sort0),
            ("s", sequence(a.clone())),
            ("index", nat()),
            ("value", a.clone()),
        ];
        self.emit(
            "SetSame",
            &args,
            eq(
                a.clone(),
                at(
                    a.clone(),
                    set(a.clone(), s.clone(), index.clone(), value.clone()),
                    index.clone(),
                ),
                value.clone(),
            ),
            sc(
                "ChoiceTrue",
                vec![
                    a.clone(),
                    equal(index.clone(), index.clone()),
                    value.clone(),
                    at(a.clone(), s.clone(), index.clone()),
                    nc("EqualReflexive", vec![index.clone()]),
                ],
            ),
            true,
        )?;
        let mut other = args.to_vec();
        other.extend([
            ("i", nat()),
            (
                "different",
                eq(boolean(), equal(i.clone(), index.clone()), bit(false)),
            ),
        ]);
        self.emit(
            "SetOther",
            &other,
            eq(
                a.clone(),
                at(
                    a.clone(),
                    set(a.clone(), s.clone(), index.clone(), value.clone()),
                    i.clone(),
                ),
                at(a.clone(), s.clone(), i.clone()),
            ),
            sc(
                "ChoiceFalse",
                vec![
                    a.clone(),
                    equal(i, index.clone()),
                    value.clone(),
                    at(a.clone(), s.clone(), var("i")),
                    var("different"),
                ],
            ),
            true,
        )?;
        let mut overwrite = args.to_vec();
        overwrite.push(("new", a.clone()));
        let old_at = at(a.clone(), s.clone(), var("_point"));
        let at_flag = |flag: Expr| {
            choose(
                a.clone(),
                flag.clone(),
                var("new"),
                choose(a.clone(), flag, value.clone(), old_at.clone()),
            )
        };
        let expected = |flag| choose(a.clone(), flag, var("new"), old_at.clone());
        let motive = lam(
            "_flag",
            boolean(),
            eq(a.clone(), at_flag(var("_flag")), expected(var("_flag"))),
        );
        let body = call(
            "Std.Bool.cases",
            vec![
                motive,
                refl(a.clone(), old_at),
                refl(a.clone(), var("new")),
                equal(var("_point"), index.clone()),
            ],
        );
        self.emit(
            "SetOverwrite",
            &overwrite,
            prefix(
                a.clone(),
                length(a.clone(), s.clone()),
                set(
                    a.clone(),
                    set(a.clone(), s.clone(), index.clone(), value),
                    index.clone(),
                    var("new"),
                ),
                set(a.clone(), s.clone(), index, var("new")),
            ),
            points(length(a, s), body),
            true,
        )
    }

    fn offsets(&mut self) -> Result<()> {
        let (a, s, i) = (var("A"), var("s"), var("_point"));
        let zero_add = Equality::natural(
            add(zero(), i.clone()),
            add(i.clone(), zero()),
            nc("AddComm", vec![zero(), i.clone()]),
        )
        .then(Equality::natural(
            add(i.clone(), zero()),
            i.clone(),
            nc("AddZero", vec![i.clone()]),
        ))?;
        let proof = zero_add
            .mapped(
                lam(
                    "_position",
                    nat(),
                    at(a.clone(), s.clone(), var("_position")),
                ),
                at(a.clone(), s.clone(), add(zero(), i.clone())),
                at(a.clone(), s.clone(), i.clone()),
                a.clone(),
            )
            .proof;
        self.emit(
            "OffsetZero",
            &[
                ("A", Expr::Sort0),
                ("s", sequence(a.clone())),
                ("count", nat()),
            ],
            prefix(
                a.clone(),
                var("count"),
                offset(a.clone(), s.clone(), zero(), var("count")),
                s.clone(),
            ),
            points(var("count"), proof),
            true,
        )?;
        let (lower, start, count) = (var("lower"), var("start"), var("count"));
        let indices = Equality::natural(
            add(add(lower.clone(), start.clone()), i.clone()),
            add(lower.clone(), add(start.clone(), i.clone())),
            nc("AddAssoc", vec![lower.clone(), start.clone(), i.clone()]),
        )
        .reverse();
        let proof = indices
            .mapped(
                lam(
                    "_position",
                    nat(),
                    at(a.clone(), s.clone(), var("_position")),
                ),
                at(
                    a.clone(),
                    s.clone(),
                    add(lower.clone(), add(start.clone(), i.clone())),
                ),
                at(
                    a.clone(),
                    s.clone(),
                    add(add(lower.clone(), start.clone()), i),
                ),
                a.clone(),
            )
            .proof;
        self.emit(
            "OffsetCompose",
            &[
                ("A", Expr::Sort0),
                ("s", sequence(a.clone())),
                ("lower", nat()),
                ("length", nat()),
                ("start", nat()),
                ("count", nat()),
            ],
            prefix(
                a.clone(),
                count.clone(),
                offset(
                    a.clone(),
                    offset(a.clone(), s.clone(), lower.clone(), var("length")),
                    start.clone(),
                    count.clone(),
                ),
                offset(a, s, add(lower, start), count.clone()),
            ),
            points(count, proof),
            true,
        )
    }

    fn append_observations(&mut self) -> Result<()> {
        let (a, s, t, i) = (var("A"), var("s"), var("t"), var("i"));
        let condition = less(i.clone(), length(a.clone(), s.clone()));
        let left = at(a.clone(), s.clone(), i.clone());
        let right = at(
            a.clone(),
            t.clone(),
            sub(i.clone(), length(a.clone(), s.clone())),
        );
        for take_left in [true, false] {
            let args = [
                ("A", Expr::Sort0),
                ("s", sequence(a.clone())),
                ("t", sequence(a.clone())),
                ("i", nat()),
                ("branch", eq(boolean(), condition.clone(), bit(take_left))),
            ];
            self.emit(
                if take_left {
                    "AppendLeft"
                } else {
                    "AppendRight"
                },
                &args,
                eq(
                    a.clone(),
                    at(
                        a.clone(),
                        appended(a.clone(), s.clone(), t.clone()),
                        i.clone(),
                    ),
                    if take_left {
                        left.clone()
                    } else {
                        right.clone()
                    },
                ),
                sc(
                    if take_left {
                        "ChoiceTrue"
                    } else {
                        "ChoiceFalse"
                    },
                    vec![
                        a.clone(),
                        condition.clone(),
                        left.clone(),
                        right.clone(),
                        var("branch"),
                    ],
                ),
                true,
            )?;
        }
        Ok(())
    }

    fn validity(&mut self) -> Result<()> {
        let (a, b, p, q, s, i) = (
            var("A"),
            var("B"),
            var("P"),
            var("Q"),
            var("s"),
            var("_point"),
        );
        let predicate = function(a.clone(), Expr::Sort0);
        let empty_length = Equality::natural(length(a.clone(), s.clone()), zero(), var("empty"));
        let length_compare = empty_length.mapped(
            lam("_length", nat(), less(i.clone(), var("_length"))),
            less(i.clone(), length(a.clone(), s.clone())),
            less(i.clone(), zero()),
            boolean(),
        );
        let impossible = length_compare
            .then(Equality::boolean(
                less(i.clone(), zero()),
                bit(false),
                nc("LessZero", vec![i.clone()]),
            ))?
            .reverse()
            .then(Equality::boolean(
                less(i.clone(), length(a.clone(), s.clone())),
                bit(true),
                var("_in"),
            ))?
            .proof;
        self.emit(
            "EmptyValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate.clone()),
                ("s", sequence(a.clone())),
                ("empty", eq(nat(), length(a.clone(), s.clone()), zero())),
            ],
            validity(a.clone(), p.clone(), s.clone()),
            points(
                length(a.clone(), s.clone()),
                bc(
                    "FalseElim",
                    vec![
                        app(p.clone(), vec![at(a.clone(), s.clone(), i.clone())]),
                        impossible,
                    ],
                ),
            ),
            true,
        )?;
        self.emit(
            "ConstantValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate.clone()),
                ("n", nat()),
                ("value", a.clone()),
                ("valid", app(p.clone(), vec![var("value")])),
            ],
            validity(
                a.clone(),
                p.clone(),
                sc("Constant", vec![a.clone(), var("n"), var("value")]),
            ),
            points(var("n"), var("valid")),
            true,
        )?;
        let preservation = pi(
            "_value",
            a.clone(),
            pi(
                "_valid",
                app(p.clone(), vec![var("_value")]),
                app(q.clone(), vec![app(var("f"), vec![var("_value")])]),
            ),
        );
        self.emit(
            "MapValid",
            &[
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("P", predicate.clone()),
                ("Q", function(b.clone(), Expr::Sort0)),
                ("f", function(a.clone(), b.clone())),
                ("preserves", preservation),
                ("s", sequence(a.clone())),
                ("valid", validity(a.clone(), p.clone(), s.clone())),
            ],
            validity(b.clone(), q, mapped(a.clone(), b, var("f"), s.clone())),
            points(
                length(a.clone(), s.clone()),
                app(
                    var("preserves"),
                    vec![
                        at(a.clone(), s.clone(), i.clone()),
                        app(var("valid"), vec![i.clone(), var("_in")]),
                    ],
                ),
            ),
            true,
        )?;
        let old = at(a.clone(), s.clone(), i.clone());
        let motive = lam(
            "_flag",
            boolean(),
            app(
                p.clone(),
                vec![choose(a.clone(), var("_flag"), var("value"), old)],
            ),
        );
        let proof = call(
            "Std.Bool.cases",
            vec![
                motive,
                app(var("valid"), vec![i.clone(), var("_in")]),
                var("replacement"),
                equal(i.clone(), var("index")),
            ],
        );
        self.emit(
            "SetValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate.clone()),
                ("s", sequence(a.clone())),
                ("index", nat()),
                ("value", a.clone()),
                ("valid", validity(a.clone(), p.clone(), s.clone())),
                ("replacement", app(p.clone(), vec![var("value")])),
            ],
            validity(
                a.clone(),
                p.clone(),
                set(a.clone(), s.clone(), var("index"), var("value")),
            ),
            points(length(a.clone(), s.clone()), proof),
            true,
        )?;
        self.emit(
            "OffsetValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate),
                ("s", sequence(a.clone())),
                ("lower", nat()),
                ("count", nat()),
                ("valid", validity(a.clone(), p.clone(), s.clone())),
                (
                    "within",
                    within(a.clone(), s.clone(), var("lower"), var("count")),
                ),
            ],
            validity(
                a.clone(),
                p,
                offset(a.clone(), s, var("lower"), var("count")),
            ),
            points(
                var("count"),
                app(
                    var("valid"),
                    vec![
                        add(var("lower"), i.clone()),
                        app(var("within"), vec![i, var("_in")]),
                    ],
                ),
            ),
            true,
        )?;
        let t = var("t");
        let left_count = length(a.clone(), var("s"));
        let right_count = length(a.clone(), t.clone());
        let condition = less(var("_point"), left_count.clone());
        let left_value = at(a.clone(), var("s"), var("_point"));
        let right_index = sub(var("_point"), left_count.clone());
        let right_value = at(a.clone(), t.clone(), right_index.clone());
        let motive = lam(
            "_flag",
            boolean(),
            pi(
                "_branch",
                eq(boolean(), condition.clone(), var("_flag")),
                app(
                    var("P"),
                    vec![choose(a.clone(), var("_flag"), left_value, right_value)],
                ),
            ),
        );
        let right_bound = sc(
            "AppendRightBound",
            vec![
                left_count.clone(),
                var("_point"),
                right_count.clone(),
                var("_in"),
                var("_branch"),
            ],
        );
        let right_case = lam(
            "_branch",
            eq(boolean(), condition.clone(), bit(false)),
            app(var("right"), vec![right_index, right_bound]),
        );
        let left_case = lam(
            "_branch",
            eq(boolean(), condition.clone(), bit(true)),
            app(var("left"), vec![var("_point"), var("_branch")]),
        );
        let proof = app(
            call(
                "Std.Bool.cases",
                vec![motive, right_case, left_case, condition.clone()],
            ),
            vec![refl(boolean(), condition)],
        );
        self.emit(
            "AppendValid",
            &[
                ("A", Expr::Sort0),
                ("P", function(a.clone(), Expr::Sort0)),
                ("s", sequence(a.clone())),
                ("t", sequence(a.clone())),
                ("left", validity(a.clone(), var("P"), var("s"))),
                ("right", validity(a.clone(), var("P"), t.clone())),
            ],
            validity(a.clone(), var("P"), appended(a, var("s"), t)),
            points(add(left_count, right_count), proof),
            true,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut sequences = Sequences(builder);
    sequences.definitions()?;
    sequences.choices()?;
    arithmetic::append(&mut sequences)?;
    sequences.observations()?;
    sequences.relations()?;
    sequences.maps()?;
    sequences.updates()?;
    sequences.offsets()?;
    sequences.append_observations()?;
    sequences.validity()
}
