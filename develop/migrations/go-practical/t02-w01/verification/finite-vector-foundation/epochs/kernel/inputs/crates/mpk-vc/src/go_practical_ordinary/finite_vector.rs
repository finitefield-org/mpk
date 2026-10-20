//! Finite Bool-indexed values observed through ordinary Nat-indexed sequences.
//! Depth zero has one slot, not the frozen empty-array dummy. Depth eight
//! preserves all 256 arbitrary byte slots when A is the full eight-bit type.
//! No word truncation, source access authority or nominal identity is inferred.
use super::syntax::*;
use super::{Builder, Result};

mod arithmetic;

const BITS: [&str; 8] = ["b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7"];
const SLOTS: [&str; 8] = [
    "_slot0", "_slot1", "_slot2", "_slot3", "_slot4", "_slot5", "_slot6", "_slot7",
];

fn fc(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Sequence.{name}"), args)
}
fn capacity(d: usize) -> Expr {
    fc(d, "Capacity", vec![])
}
fn cube(d: usize, mut a: Expr) -> Expr {
    for name in SLOTS[..d].iter().rev() {
        a = pi(name, boolean(), a);
    }
    a
}
fn bit_args(d: usize) -> Vec<Binder> {
    BITS[..d].iter().map(|name| (*name, boolean())).collect()
}
fn bits(d: usize) -> Vec<Expr> {
    BITS[..d].iter().map(|name| var(name)).collect()
}
fn index(d: usize, bits: &[Expr]) -> Expr {
    fc(d, "Index", bits.to_vec())
}
fn lookup(d: usize, a: Expr, data: Expr, i: Expr) -> Expr {
    fc(d, "Lookup", vec![a, data, i])
}
fn inside(i: Expr, n: Expr) -> Expr {
    eq(boolean(), less(i, n), bit(true))
}
fn choose(a: Expr, condition: Expr, yes: Expr, no: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_case", boolean(), a), no, yes, condition],
    )
}
fn all(d: usize, p: Expr, data: Expr) -> Expr {
    let mut body = app(
        p,
        vec![app(data, SLOTS[..d].iter().map(|name| var(name)).collect())],
    );
    for name in SLOTS[..d].iter().rev() {
        body = pi(name, boolean(), body);
    }
    body
}
fn fits(d: usize, n: Expr) -> Expr {
    pi(
        "_point",
        nat(),
        pi(
            "_in",
            inside(var("_point"), n),
            inside(var("_point"), capacity(d)),
        ),
    )
}
fn embed(d: usize, a: Expr, n: Expr, data: Expr) -> Expr {
    fc(d, "Embed", vec![a, n, data])
}
fn at(a: Expr, s: Expr, i: Expr) -> Expr {
    sc("At", vec![a, s, i])
}
fn length(a: Expr, s: Expr) -> Expr {
    sc("Length", vec![a, s])
}
fn seq(a: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call(
            "Std.Bool.recType",
            vec![nat(), pi("_point", nat(), a), var("_tag")],
        ),
    )
}
fn predicate(a: Expr) -> Expr {
    pi("_value", a, Expr::Sort0)
}

struct Vectors<'a>(&'a mut Builder);
impl Vectors<'_> {
    fn emit(
        &mut self,
        d: usize,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }

    fn definitions(&mut self, d: usize) -> Result<()> {
        let (a, data, i, n) = (var("A"), var("data"), var("i"), var("n"));
        self.emit(
            d,
            "Capacity",
            &[],
            nat(),
            if d == 0 {
                one()
            } else {
                add(capacity(d - 1), capacity(d - 1))
            },
            false,
        )?;
        self.emit(
            d,
            "Type",
            &[("A", Expr::Sort0)],
            Expr::Sort0,
            cube(d, a.clone()),
            false,
        )?;
        self.emit(
            d,
            "TypeAsFunction",
            &[("A", Expr::Sort0), ("data", fc(d, "Type", vec![a.clone()]))],
            cube(d, a.clone()),
            data.clone(),
            false,
        )?;
        let address = bits(d);
        self.emit(
            d,
            "Index",
            &bit_args(d),
            nat(),
            if d == 0 {
                zero()
            } else {
                let tail = index(d - 1, &address[1..]);
                choose(
                    nat(),
                    address[0].clone(),
                    add(capacity(d - 1), tail.clone()),
                    tail,
                )
            },
            false,
        )?;
        self.emit(
            d,
            "Lookup",
            &[
                ("A", Expr::Sort0),
                ("data", cube(d, a.clone())),
                ("i", nat()),
            ],
            a.clone(),
            if d == 0 {
                data.clone()
            } else {
                choose(
                    a.clone(),
                    less(i.clone(), capacity(d - 1)),
                    lookup(
                        d - 1,
                        a.clone(),
                        app(data.clone(), vec![bit(false)]),
                        i.clone(),
                    ),
                    lookup(
                        d - 1,
                        a.clone(),
                        app(data.clone(), vec![bit(true)]),
                        sub(i, capacity(d - 1)),
                    ),
                )
            },
            false,
        )?;
        let inputs = [
            ("A", Expr::Sort0),
            ("P", predicate(a.clone())),
            ("data", cube(d, a.clone())),
        ];
        self.emit(
            d,
            "All",
            &inputs,
            Expr::Sort0,
            all(d, var("P"), data.clone()),
            false,
        )?;
        let mut observed = inputs.to_vec();
        observed.push((
            "valid",
            fc(d, "All", vec![a.clone(), var("P"), data.clone()]),
        ));
        self.emit(
            d,
            "AllAsFunction",
            &observed,
            all(d, var("P"), data.clone()),
            var("valid"),
            false,
        )?;
        self.emit(
            d,
            "Fits",
            &[("n", nat())],
            Expr::Sort0,
            fits(d, n.clone()),
            false,
        )?;
        self.emit(
            d,
            "FitsAsFunction",
            &[("n", nat()), ("fits", fc(d, "Fits", vec![n.clone()]))],
            fits(d, n.clone()),
            var("fits"),
            false,
        )?;
        self.emit(
            d,
            "Embed",
            &[
                ("A", Expr::Sort0),
                ("n", nat()),
                ("data", cube(d, a.clone())),
            ],
            seq(a.clone()),
            sc(
                "Make",
                vec![
                    a.clone(),
                    n,
                    lam("_point", nat(), lookup(d, a, data, var("_point"))),
                ],
            ),
            false,
        )
    }

    fn lookup_index(&mut self, d: usize) -> Result<()> {
        let (a, data, address) = (var("A"), var("data"), bits(d));
        let property = |address: &[Expr]| {
            eq(
                a.clone(),
                lookup(d, a.clone(), data.clone(), index(d, address)),
                app(data.clone(), address.to_vec()),
            )
        };
        let proof = if d == 0 {
            refl(a.clone(), data.clone())
        } else {
            let small = index(d - 1, &address[1..]);
            let half = capacity(d - 1);
            let read = |value, i| lookup(d - 1, a.clone(), app(data.clone(), vec![bit(value)]), i);
            let previous = |value| {
                let mut args = vec![a.clone(), app(data.clone(), vec![bit(value)])];
                args.extend(address[1..].iter().cloned());
                fc(d - 1, "LookupIndex", args)
            };
            let low = read(false, small.clone());
            let high = read(true, sub(small.clone(), half.clone()));
            let false_proof = Equality {
                ty: a.clone(),
                left: choose(
                    a.clone(),
                    less(small.clone(), half.clone()),
                    low.clone(),
                    high.clone(),
                ),
                right: low.clone(),
                proof: sc(
                    "ChoiceTrue",
                    vec![
                        a.clone(),
                        less(small.clone(), half.clone()),
                        low.clone(),
                        high,
                        fc(d - 1, "IndexBound", address[1..].to_vec()),
                    ],
                ),
            }
            .then(Equality {
                ty: a.clone(),
                left: low,
                right: app(
                    data.clone(),
                    [vec![bit(false)], address[1..].to_vec()].concat(),
                ),
                proof: previous(false),
            })?
            .proof;
            let shifted = add(half.clone(), small.clone());
            let condition = less(shifted.clone(), half.clone());
            let low = read(false, shifted.clone());
            let high = read(true, sub(shifted.clone(), half.clone()));
            let subtraction = arithmetic::unshift(half.clone(), small.clone())?.mapped(
                lam("_index", nat(), read(true, var("_index"))),
                high.clone(),
                read(true, small.clone()),
                a.clone(),
            );
            let true_proof = Equality {
                ty: a.clone(),
                left: choose(a.clone(), condition.clone(), low.clone(), high.clone()),
                right: high.clone(),
                proof: sc(
                    "ChoiceFalse",
                    vec![
                        a.clone(),
                        condition,
                        low,
                        high,
                        arithmetic::not_below(half, small.clone())?.proof,
                    ],
                ),
            }
            .then(subtraction)?
            .then(Equality {
                ty: a.clone(),
                left: read(true, small),
                right: app(
                    data.clone(),
                    [vec![bit(true)], address[1..].to_vec()].concat(),
                ),
                proof: previous(true),
            })?
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
        let mut args = vec![("A", Expr::Sort0), ("data", cube(d, a.clone()))];
        args.extend(bit_args(d));
        self.emit(d, "LookupIndex", &args, property(&address), proof, true)
    }

    fn lookup_valid(&mut self, d: usize) -> Result<()> {
        let (a, p, data, i) = (var("A"), var("P"), var("data"), var("i"));
        let proof = if d == 0 {
            var("valid")
        } else {
            let half = capacity(d - 1);
            let condition = less(i.clone(), half.clone());
            let high_index = sub(i.clone(), half.clone());
            let read =
                |value, index| lookup(d - 1, a.clone(), app(data.clone(), vec![bit(value)]), index);
            let low = read(false, i.clone());
            let high = read(true, high_index.clone());
            let case = |value, index, bound| {
                fc(
                    d - 1,
                    "LookupValid",
                    vec![
                        a.clone(),
                        p.clone(),
                        app(data.clone(), vec![bit(value)]),
                        app(var("valid"), vec![bit(value)]),
                        index,
                        bound,
                    ],
                )
            };
            let motive = lam(
                "_flag",
                boolean(),
                pi(
                    "_branch",
                    eq(boolean(), condition.clone(), var("_flag")),
                    app(p.clone(), vec![choose(a.clone(), var("_flag"), low, high)]),
                ),
            );
            let right_bound = sc(
                "AppendRightBound",
                vec![half.clone(), i.clone(), half, var("bound"), var("_branch")],
            );
            let right = lam(
                "_branch",
                eq(boolean(), condition.clone(), bit(false)),
                case(true, high_index, right_bound),
            );
            let left = lam(
                "_branch",
                eq(boolean(), condition.clone(), bit(true)),
                case(false, i.clone(), var("_branch")),
            );
            app(
                call(
                    "Std.Bool.cases",
                    vec![motive, right, left, condition.clone()],
                ),
                vec![refl(boolean(), condition)],
            )
        };
        self.emit(
            d,
            "LookupValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate(a.clone())),
                ("data", cube(d, a.clone())),
                ("valid", all(d, p.clone(), data.clone())),
                ("i", nat()),
                ("bound", inside(i.clone(), capacity(d))),
            ],
            app(p, vec![lookup(d, a, data, i)]),
            proof,
            true,
        )
    }

    fn embeddings(&mut self, d: usize) -> Result<()> {
        let (a, p, data, n, i) = (var("A"), var("P"), var("data"), var("n"), var("i"));
        let value = embed(d, a.clone(), n.clone(), data.clone());
        let input = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("data", cube(d, a.clone())),
        ];
        self.emit(
            d,
            "EmbedLength",
            &input,
            eq(nat(), length(a.clone(), value.clone()), n.clone()),
            refl(nat(), n.clone()),
            true,
        )?;
        let mut indexed = input.to_vec();
        indexed.push(("i", nat()));
        let read = lookup(d, a.clone(), data.clone(), i.clone());
        self.emit(
            d,
            "EmbedAt",
            &indexed,
            eq(a.clone(), at(a.clone(), value.clone(), i), read.clone()),
            refl(a.clone(), read),
            true,
        )?;
        let address = bits(d);
        let natural_index = index(d, &address);
        let mut at_slot = input.to_vec();
        at_slot.extend(bit_args(d));
        let mut previous = vec![a.clone(), data.clone()];
        previous.extend(address.iter().cloned());
        self.emit(
            d,
            "EmbedIndex",
            &at_slot,
            eq(
                a.clone(),
                at(a.clone(), value.clone(), natural_index.clone()),
                app(data.clone(), address),
            ),
            fc(d, "LookupIndex", previous),
            true,
        )?;
        at_slot.push(("bound", inside(natural_index.clone(), n.clone())));
        self.emit(
            d,
            "EmbedIndexValid",
            &at_slot,
            sc("IndexValid", vec![a.clone(), value.clone(), natural_index]),
            var("bound"),
            true,
        )?;
        let result = pi(
            "_point",
            nat(),
            pi(
                "_in",
                inside(var("_point"), length(a.clone(), value.clone())),
                app(p.clone(), vec![at(a.clone(), value, var("_point"))]),
            ),
        );
        self.emit(
            d,
            "EmbedValid",
            &[
                ("A", Expr::Sort0),
                ("P", predicate(a.clone())),
                ("n", nat()),
                ("data", cube(d, a.clone())),
                ("fits", fits(d, n.clone())),
                ("valid", all(d, p.clone(), data.clone())),
            ],
            result,
            lam(
                "_point",
                nat(),
                lam(
                    "_in",
                    inside(var("_point"), n),
                    fc(
                        d,
                        "LookupValid",
                        vec![
                            a,
                            p,
                            data,
                            var("valid"),
                            var("_point"),
                            app(var("fits"), vec![var("_point"), var("_in")]),
                        ],
                    ),
                ),
            ),
            true,
        )?;
        self.emit(
            d,
            "FitsFull",
            &[],
            fits(d, capacity(d)),
            lam(
                "_point",
                nat(),
                lam("_in", inside(var("_point"), capacity(d)), var("_in")),
            ),
            true,
        )?;
        let impossible = Equality::boolean(
            less(var("_point"), zero()),
            bit(false),
            nc("LessZero", vec![var("_point")]),
        )
        .reverse()
        .then(Equality::boolean(
            less(var("_point"), zero()),
            bit(true),
            var("_in"),
        ))?
        .proof;
        self.emit(
            d,
            "FitsEmpty",
            &[],
            fits(d, zero()),
            lam(
                "_point",
                nat(),
                lam(
                    "_in",
                    inside(var("_point"), zero()),
                    bc(
                        "FalseElim",
                        vec![inside(var("_point"), capacity(d)), impossible],
                    ),
                ),
            ),
            true,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut vectors = Vectors(builder);
    for d in 0..=8 {
        vectors.definitions(d)?;
        arithmetic::bounds(&mut vectors, d)?;
        vectors.lookup_index(d)?;
        vectors.lookup_valid(d)?;
        vectors.embeddings(d)?;
    }
    Ok(())
}
