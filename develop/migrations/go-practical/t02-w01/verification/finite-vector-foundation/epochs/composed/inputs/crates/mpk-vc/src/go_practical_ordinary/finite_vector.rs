//! Finite Bool-indexed values observed through ordinary Nat-indexed sequences.
//! Depth zero has one slot, not the frozen empty-array dummy. Depth eight
//! preserves all 256 arbitrary byte slots when A is the full eight-bit type.
//! No word truncation, source access authority or nominal identity is inferred.
use super::syntax::*;
use super::{Builder, Result};

mod arithmetic;
mod branch;

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
fn gc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.{name}"), args)
}
fn function(a: Expr) -> Expr {
    pi("_point", nat(), a)
}
fn lookup_function(d: usize, a: Expr, data: Expr) -> Expr {
    fc(d, "Lookup", vec![a, data])
}
fn lookup_body(d: usize, a: Expr, data: Expr) -> Expr {
    if d == 0 {
        lam("_point", nat(), data)
    } else {
        gc(
            "Branch",
            vec![
                a.clone(),
                capacity(d - 1),
                lookup_function(d - 1, a.clone(), app(data.clone(), vec![bit(false)])),
                lookup_function(d - 1, a, app(data, vec![bit(true)])),
            ],
        )
    }
}
fn index_body(d: usize, address: &[Expr]) -> Expr {
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
    }
}
fn capacity_body(d: usize) -> Expr {
    if d == 0 {
        one()
    } else {
        add(capacity(d - 1), capacity(d - 1))
    }
}
fn definition(
    d: usize,
    name: &str,
    args: Vec<Expr>,
    ty: Expr,
    left: Expr,
    right: Expr,
) -> Equality {
    Equality {
        ty,
        left,
        right,
        proof: fc(d, &format!("{name}Definition"), args),
    }
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

fn embedded_read(d: usize, a: Expr, n: Expr, data: Expr, i: Expr) -> Result<Equality> {
    let value = embed(d, a.clone(), n.clone(), data.clone());
    let f = lookup_function(d, a.clone(), data.clone());
    let made = sc("Make", vec![a.clone(), n.clone(), f.clone()]);
    let observation = at(a.clone(), made.clone(), i.clone());
    definition(
        d,
        "Embed",
        vec![a.clone(), n.clone(), data.clone()],
        seq(a.clone()),
        value.clone(),
        made,
    )
    .mapped(
        lam(
            "_sequence",
            seq(a.clone()),
            at(a.clone(), var("_sequence"), i.clone()),
        ),
        at(a.clone(), value, i.clone()),
        observation.clone(),
        a.clone(),
    )
    .then(Equality {
        ty: a.clone(),
        left: observation,
        right: lookup(d, a.clone(), data, i.clone()),
        proof: sc("MakeAt", vec![a, n, f, i]),
    })
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
        let (a, data, n) = (var("A"), var("data"), var("n"));
        self.emit(d, "Capacity", &[], nat(), capacity_body(d), false)?;
        self.emit(
            d,
            "CapacityDefinition",
            &[],
            eq(nat(), capacity(d), capacity_body(d)),
            refl(nat(), capacity_body(d)),
            true,
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
            index_body(d, &address),
            false,
        )?;
        self.emit(
            d,
            "IndexDefinition",
            &bit_args(d),
            eq(nat(), index(d, &address), index_body(d, &address)),
            refl(nat(), index_body(d, &address)),
            true,
        )?;
        let input = [("A", Expr::Sort0), ("data", cube(d, a.clone()))];
        self.emit(
            d,
            "Lookup",
            &input,
            function(a.clone()),
            lookup_body(d, a.clone(), data.clone()),
            false,
        )?;
        self.emit(
            d,
            "LookupDefinition",
            &input,
            eq(
                function(a.clone()),
                lookup_function(d, a.clone(), data.clone()),
                lookup_body(d, a.clone(), data.clone()),
            ),
            refl(function(a.clone()), lookup_body(d, a.clone(), data.clone())),
            true,
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
        let input = [
            ("A", Expr::Sort0),
            ("n", nat()),
            ("data", cube(d, a.clone())),
        ];
        let body = sc(
            "Make",
            vec![
                a.clone(),
                n.clone(),
                lookup_function(d, a.clone(), data.clone()),
            ],
        );
        self.emit(d, "Embed", &input, seq(a.clone()), body.clone(), false)?;
        self.emit(
            d,
            "EmbedDefinition",
            &input,
            eq(seq(a.clone()), embed(d, a.clone(), n, data), body.clone()),
            refl(seq(a), body),
            true,
        )
    }

    fn lookup_index(&mut self, d: usize) -> Result<()> {
        let (a, data, address) = (var("A"), var("data"), bits(d));
        let natural_index = index(d, &address);
        let observed = app(data.clone(), address.clone());
        let proof = if d == 0 {
            refl(a.clone(), data.clone())
        } else {
            let half = capacity(d - 1);
            let small = index(d - 1, &address[1..]);
            let low = lookup_function(d - 1, a.clone(), app(data.clone(), vec![bit(false)]));
            let high = lookup_function(d - 1, a.clone(), app(data.clone(), vec![bit(true)]));
            let branch = |i| {
                gc(
                    "Branch",
                    vec![a.clone(), half.clone(), low.clone(), high.clone(), i],
                )
            };
            let before = lookup(d, a.clone(), data.clone(), natural_index.clone());
            let middle = branch(natural_index.clone());
            let unfolded = definition(
                d,
                "Lookup",
                vec![a.clone(), data.clone()],
                function(a.clone()),
                lookup_function(d, a.clone(), data.clone()),
                lookup_body(d, a.clone(), data.clone()),
            )
            .mapped(
                lam(
                    "_read",
                    function(a.clone()),
                    app(var("_read"), vec![natural_index.clone()]),
                ),
                before,
                middle.clone(),
                a.clone(),
            );
            let explicit_index = index_body(d, &address);
            let shifted = branch(explicit_index.clone());
            let change_index = definition(
                d,
                "Index",
                address.clone(),
                nat(),
                natural_index.clone(),
                explicit_index,
            )
            .mapped(
                lam("_index", nat(), branch(var("_index"))),
                middle,
                shifted.clone(),
                a.clone(),
            );
            let low_read = lookup(
                d - 1,
                a.clone(),
                app(data.clone(), vec![bit(false)]),
                small.clone(),
            );
            let high_read = lookup(
                d - 1,
                a.clone(),
                app(data.clone(), vec![bit(true)]),
                small.clone(),
            );
            let selected = choose(
                a.clone(),
                address[0].clone(),
                high_read.clone(),
                low_read.clone(),
            );
            let split = Equality {
                ty: a.clone(),
                left: shifted,
                right: selected.clone(),
                proof: gc(
                    "BranchIndex",
                    vec![
                        a.clone(),
                        half,
                        low,
                        high,
                        small,
                        address[0].clone(),
                        fc(d - 1, "IndexBound", address[1..].to_vec()),
                    ],
                ),
            };
            let slot = |head| {
                app(
                    data.clone(),
                    [vec![bit(head)], address[1..].to_vec()].concat(),
                )
            };
            let previous = |head| {
                let mut args = vec![a.clone(), app(data.clone(), vec![bit(head)])];
                args.extend(address[1..].to_vec());
                fc(d - 1, "LookupIndex", args)
            };
            let low_changed = choose(
                a.clone(),
                address[0].clone(),
                high_read.clone(),
                slot(false),
            );
            let low = Equality {
                ty: a.clone(),
                left: low_read,
                right: slot(false),
                proof: previous(false),
            }
            .mapped(
                lam(
                    "_value",
                    a.clone(),
                    choose(
                        a.clone(),
                        address[0].clone(),
                        high_read.clone(),
                        var("_value"),
                    ),
                ),
                selected,
                low_changed.clone(),
                a.clone(),
            );
            let both_changed = choose(a.clone(), address[0].clone(), slot(true), slot(false));
            let high = Equality {
                ty: a.clone(),
                left: high_read,
                right: slot(true),
                proof: previous(true),
            }
            .mapped(
                lam(
                    "_value",
                    a.clone(),
                    choose(a.clone(), address[0].clone(), var("_value"), slot(false)),
                ),
                low_changed,
                both_changed.clone(),
                a.clone(),
            );
            let mut selected_bits = address.clone();
            selected_bits[0] = var("_head");
            let finish = Equality {
                ty: a.clone(),
                left: both_changed,
                right: observed.clone(),
                proof: gc(
                    "SelectAt",
                    vec![
                        a.clone(),
                        lam("_head", boolean(), app(data.clone(), selected_bits)),
                        address[0].clone(),
                    ],
                ),
            };
            unfolded
                .then(change_index)?
                .then(split)?
                .then(low)?
                .then(high)?
                .then(finish)?
                .proof
        };
        let mut args = vec![("A", Expr::Sort0), ("data", cube(d, a.clone()))];
        args.extend(bit_args(d));
        self.emit(
            d,
            "LookupIndex",
            &args,
            eq(a.clone(), lookup(d, a, data, natural_index), observed),
            proof,
            true,
        )
    }

    fn lookup_valid(&mut self, d: usize) -> Result<()> {
        let (a, p, data, i) = (var("A"), var("P"), var("data"), var("i"));
        let proof = if d == 0 {
            var("valid")
        } else {
            let half = capacity(d - 1);
            let child =
                |head| lookup_function(d - 1, a.clone(), app(data.clone(), vec![bit(head)]));
            let child_valid = |head| {
                lam(
                    "_point",
                    nat(),
                    lam(
                        "_in",
                        inside(var("_point"), half.clone()),
                        fc(
                            d - 1,
                            "LookupValid",
                            vec![
                                a.clone(),
                                p.clone(),
                                app(data.clone(), vec![bit(head)]),
                                app(var("valid"), vec![bit(head)]),
                                var("_point"),
                                var("_in"),
                            ],
                        ),
                    ),
                )
            };
            let branch_valid = gc(
                "BranchValid",
                vec![
                    a.clone(),
                    p.clone(),
                    half,
                    child(false),
                    child(true),
                    child_valid(false),
                    child_valid(true),
                    i.clone(),
                    var("bound"),
                ],
            );
            let equation = definition(
                d,
                "Lookup",
                vec![a.clone(), data.clone()],
                function(a.clone()),
                lookup_function(d, a.clone(), data.clone()),
                lookup_body(d, a.clone(), data.clone()),
            )
            .reverse();
            call(
                "Std.Eq.rewrite",
                vec![
                    function(a.clone()),
                    equation.left,
                    equation.right,
                    lam(
                        "_read",
                        function(a.clone()),
                        app(p.clone(), vec![app(var("_read"), vec![i.clone()])]),
                    ),
                    equation.proof,
                    branch_valid,
                ],
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
            eq(
                a.clone(),
                at(a.clone(), value.clone(), i.clone()),
                read.clone(),
            ),
            embedded_read(d, a.clone(), n.clone(), data.clone(), i)?.proof,
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
            embedded_read(d, a.clone(), n.clone(), data.clone(), natural_index.clone())?
                .then(Equality {
                    ty: a.clone(),
                    left: lookup(d, a.clone(), data.clone(), natural_index.clone()),
                    right: app(data.clone(), bits(d)),
                    proof: fc(d, "LookupIndex", previous),
                })?
                .proof,
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
                app(p.clone(), vec![at(a.clone(), value.clone(), var("_point"))]),
            ),
        );
        let equation =
            embedded_read(d, a.clone(), n.clone(), data.clone(), var("_point"))?.reverse();
        let valid = fc(
            d,
            "LookupValid",
            vec![
                a.clone(),
                p.clone(),
                data.clone(),
                var("valid"),
                var("_point"),
                app(var("fits"), vec![var("_point"), var("_in")]),
            ],
        );
        let proof = call(
            "Std.Eq.rewrite",
            vec![
                a.clone(),
                equation.left,
                equation.right,
                p.clone(),
                equation.proof,
                valid,
            ],
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
                ("valid", all(d, p, data)),
            ],
            result,
            lam("_point", nat(), lam("_in", inside(var("_point"), n), proof)),
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
    branch::append(builder)?;
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
