//! Full64 Nat values, exact high bits and finite-data observations.
//! A small-range proof is consumed before replacing the complete value with
//! a slot address. Lengths always retain the original full64 denotation.
use super::counts::{digits, high, high_value};
use super::*;

mod natural;
mod upper;

const CELLS: [&str; 8] = [
    "_cell0", "_cell1", "_cell2", "_cell3", "_cell4", "_cell5", "_cell6", "_cell7",
];
fn ic(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Index.D{d}.{name}"), args)
}
fn common(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Index.{name}"), args)
}
fn fv(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Sequence.{name}"), args)
}
fn cap(d: usize) -> Expr {
    fv(d, "Capacity", vec![])
}
fn inside(i: Expr, n: Expr) -> Expr {
    eq(boolean(), less(i, n), bit(true))
}
fn upper(n: Expr, c: Expr) -> Expr {
    eq(boolean(), less(c, n), bit(false))
}
fn function(domain: Expr, result: Expr) -> Expr {
    pi("_argument", domain, result)
}
fn data_type(d: usize, mut a: Expr) -> Expr {
    for name in CELLS[..d].iter().rev() {
        a = pi(name, boolean(), a);
    }
    a
}
fn slots_valid(d: usize, p: Expr, data: Expr) -> Expr {
    let mut ty = app(
        p,
        vec![app(data, CELLS[..d].iter().map(|n| var(n)).collect())],
    );
    for name in CELLS[..d].iter().rev() {
        ty = pi(name, boolean(), ty);
    }
    ty
}
fn sequence(a: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call(
            "Std.Bool.recType",
            vec![nat(), function(nat(), a), var("_tag")],
        ),
    )
}
fn slot_address(d: usize, word: Expr) -> Result<Vec<Expr>> {
    (0..d).rev().map(|i| at(word.clone(), 64, i)).collect()
}
fn slot_index(d: usize, word: Expr) -> Result<Expr> {
    Ok(fv(d, "Index", slot_address(d, word)?))
}
fn excess(d: usize, word: Expr) -> Expr {
    ic(d, "Excess", vec![word])
}
fn full_read(d: usize, a: Expr, data: Expr, word: Expr) -> Expr {
    ic(d, "Read64", vec![a, data, word])
}
fn full_sequence(d: usize, a: Expr, data: Expr, word: Expr) -> Expr {
    ic(d, "Sequence64", vec![a, data, word])
}
fn read_body(d: usize, a: Expr, data: Expr) -> Expr {
    lam(
        "_word",
        cube(6),
        fv(d, "Lookup", vec![a, data, value(6, var("_word"))]),
    )
}
fn sequence_body(d: usize, a: Expr, data: Expr) -> Expr {
    lam(
        "_word",
        cube(6),
        fv(d, "Embed", vec![a, value(6, var("_word")), data]),
    )
}
fn definition(d: usize, name: &str, a: Expr, data: Expr, body: Expr, result: Expr) -> Equality {
    Equality {
        ty: function(cube(6), result),
        left: ic(d, name, vec![a.clone(), data.clone()]),
        right: body,
        proof: ic(d, &format!("{name}Definition"), vec![a, data]),
    }
}
fn read_equation(d: usize, a: Expr, data: Expr, word: Expr) -> Equality {
    definition(
        d,
        "Read64",
        a.clone(),
        data.clone(),
        read_body(d, a.clone(), data.clone()),
        a.clone(),
    )
    .mapped(
        lam(
            "_read",
            function(cube(6), a.clone()),
            app(var("_read"), vec![word.clone()]),
        ),
        full_read(d, a.clone(), data.clone(), word.clone()),
        fv(d, "Lookup", vec![a.clone(), data, value(6, word)]),
        a,
    )
}
fn sequence_equation(d: usize, a: Expr, data: Expr, word: Expr) -> Equality {
    definition(
        d,
        "Sequence64",
        a.clone(),
        data.clone(),
        sequence_body(d, a.clone(), data.clone()),
        sequence(a.clone()),
    )
    .mapped(
        lam(
            "_sequence",
            function(cube(6), sequence(a.clone())),
            app(var("_sequence"), vec![word.clone()]),
        ),
        full_sequence(d, a.clone(), data.clone(), word.clone()),
        fv(d, "Embed", vec![a.clone(), value(6, word), data]),
        sequence(a),
    )
}

struct Indices<'a>(&'a mut Builder);
impl Indices<'_> {
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
            &format!("Mpk.Go.Practical.Word.Index.D{d}.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn theorem(&mut self, d: usize, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(d, name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn capacity(&mut self, d: usize) -> Result<()> {
        let e = if d == 0 {
            Equality::natural(coefficient(0), one(), numeric("PowerTwoZero", vec![]))
        } else {
            Equality::natural(
                coefficient(d),
                mul(two(), coefficient(d - 1)),
                numeric("PowerTwoSucc", vec![numeral(d - 1)]),
            )
            .then(mul_right(
                two(),
                Equality::natural(
                    coefficient(d - 1),
                    cap(d - 1),
                    ic(d - 1, "CapacityPower", vec![]),
                ),
            ))?
            .then(known(
                "TwoMul",
                vec![cap(d - 1)],
                mul(two(), cap(d - 1)),
                add(cap(d - 1), cap(d - 1)),
            ))?
        };
        let body = if d == 0 {
            one()
        } else {
            add(cap(d - 1), cap(d - 1))
        };
        self.theorem(
            d,
            "CapacityPower",
            &[],
            e.then(Equality::natural(cap(d), body, fv(d, "CapacityDefinition", vec![])).reverse())?,
        )
    }
    fn branches(&mut self, d: usize) -> Result<()> {
        let word = var("word");
        let args = [("word", cube(6))];
        let (h, v, lo, p) = (
            high(d, word.clone()),
            high_value(d, word.clone()),
            digits(d, word.clone()),
            coefficient(d),
        );
        let balance = || {
            Equality::natural(
                value(6, word.clone()),
                add(lo.clone(), mul(p.clone(), v.clone())),
                sh(&format!("CountBalance{d}"), vec![word.clone()]),
            )
        };
        let reflection = Equality::boolean(
            equal(v.clone(), zero()),
            bc("Not", vec![h.clone()]),
            sh(&format!("CountHighZero{d}"), vec![word.clone()]),
        );
        let flag_false = eq(boolean(), h.clone(), bit(false));
        let flag_true = eq(boolean(), h.clone(), bit(true));
        let hz = reflection
            .clone()
            .then(bool_map(
                "Not",
                Equality::boolean(h.clone(), bit(false), var("branch")),
            ))?
            .then(Equality::boolean(
                bc("Not", vec![bit(false)]),
                bit(true),
                refl(boolean(), bit(true)),
            ))?;
        let zero_high = Equality::natural(
            v.clone(),
            zero(),
            nc("EqualReflect", vec![v.clone(), zero(), hz.proof]),
        );
        let small = balance()
            .then(mul_right(p.clone(), zero_high).on_right(lo.clone()))?
            .then(
                law("MulZero", vec![p.clone()], mul(p.clone(), zero()), zero())
                    .on_right(lo.clone()),
            )?
            .then(law(
                "AddZero",
                vec![lo.clone()],
                add(lo.clone(), zero()),
                lo.clone(),
            ))?;
        self.theorem(
            d,
            "SmallValue",
            &[args[0].clone(), ("branch", flag_false.clone())],
            small,
        )?;
        let positive = Equality::boolean(
            bc("Not", vec![equal(v.clone(), zero())]),
            less(zero(), v.clone()),
            sh("CountNonzeroPositive", vec![v.clone()]),
        )
        .reverse()
        .then(bool_map("Not", reflection))?
        .then(Equality::boolean(
            bc("Not", vec![bc("Not", vec![h.clone()])]),
            h.clone(),
            bc("DoubleNot", vec![h.clone()]),
        ))?
        .then(Equality::boolean(h.clone(), bit(true), var("branch")))?;
        let tail = sub(v.clone(), one());
        let body = add(lo.clone(), mul(p.clone(), tail.clone()));
        self.emit(d, "Excess", &args, nat(), body.clone(), false)?;
        let large = balance().then(
            mul_right(
                p.clone(),
                known(
                    "CountPositiveDecomposition",
                    vec![v.clone(), positive.proof],
                    v,
                    add(one(), tail.clone()),
                ),
            )
            .on_right(lo.clone()),
        )?;
        let left = large.right.clone();
        let large = large
            .then(Equality::natural(
                left,
                add(p.clone(), body.clone()),
                common("RebalanceLarge", vec![lo.clone(), p.clone(), tail]),
            ))?
            .then(
                Equality::natural(excess(d, word.clone()), body.clone(), refl(nat(), body))
                    .reverse()
                    .on_right(p.clone()),
            )?;
        self.theorem(
            d,
            "LargeValue",
            &[args[0].clone(), ("branch", flag_true)],
            large,
        )?;
        let property = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "_branch",
                eq(boolean(), h.clone(), env["_flag"].clone()),
                eq(
                    boolean(),
                    less(value(6, word.clone()), p.clone()),
                    bc("Not", vec![env["_flag"].clone()]),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let yes = env["_flag"] == bit(true);
            let selected = Equality::natural(
                value(6, word.clone()),
                if yes {
                    add(p.clone(), excess(d, word.clone()))
                } else {
                    lo.clone()
                },
                ic(
                    d,
                    if yes { "LargeValue" } else { "SmallValue" },
                    vec![word.clone(), var("_branch")],
                ),
            );
            let e = less_left(selected, p.clone());
            let from = e.right.clone();
            let proof = if yes {
                sh(
                    "CountNoLessExtension",
                    vec![p.clone(), excess(d, word.clone())],
                )
            } else {
                sh(&format!("CountLowBound{d}"), vec![word.clone()])
            };
            let e = e.then(Equality::boolean(from, bit(!yes), proof))?.then(
                Equality::boolean(
                    bc("Not", vec![bit(yes)]),
                    bit(!yes),
                    refl(boolean(), bit(!yes)),
                )
                .reverse(),
            )?;
            Ok(lam("_branch", eq(boolean(), h.clone(), bit(yes)), e.proof))
        };
        let cases = boolean_cases(&["_flag"], &BTreeMap::new(), &property, &mut leaf)?;
        let proof = Equality::boolean(
            less(value(6, word.clone()), p),
            bc("Not", vec![h.clone()]),
            app(cases, vec![h.clone(), refl(boolean(), h.clone())]),
        );
        let capacity = Equality::natural(coefficient(d), cap(d), ic(d, "CapacityPower", vec![]));
        self.theorem(
            d,
            "BelowCapacity",
            &args,
            less_right(value(6, word.clone()), capacity)
                .reverse()
                .then(proof)?,
        )?;
        let bounded = [
            args[0].clone(),
            ("bound", inside(value(6, word.clone()), cap(d))),
        ];
        let not_high = Equality::boolean(
            less(value(6, word.clone()), cap(d)),
            bc("Not", vec![h.clone()]),
            ic(d, "BelowCapacity", vec![word.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(value(6, word.clone()), cap(d)),
            bit(true),
            var("bound"),
        ))?;
        self.theorem(
            d,
            "HighClear",
            &bounded,
            Equality::boolean(
                h,
                bit(false),
                common(
                    "NotTrueReflect",
                    vec![high(d, word.clone()), not_high.proof],
                ),
            ),
        )?;
        self.theorem(
            d,
            "BoundedValue",
            &bounded,
            Equality::natural(
                value(6, word.clone()),
                lo,
                ic(
                    d,
                    "SmallValue",
                    vec![word.clone(), ic(d, "HighClear", vec![word, var("bound")])],
                ),
            ),
        )
    }
    fn digits(&mut self, d: usize) -> Result<()> {
        let w = var("word");
        let target = slot_index(d, w.clone())?;
        let e = if d == 0 {
            Equality::natural(digits(0, w.clone()), zero(), refl(nat(), zero())).then(
                Equality::natural(
                    fv(0, "Index", vec![]),
                    zero(),
                    fv(0, "IndexDefinition", vec![]),
                )
                .reverse(),
            )?
        } else {
            let (h, b) = (d - 1, at(w.clone(), 64, d - 1)?);
            let low = slot_index(h, w.clone())?;
            let body = add(
                digits(h, w.clone()),
                mul(coefficient(h), bit_value(b.clone())),
            );
            let e = Equality::natural(digits(d, w.clone()), body.clone(), refl(nat(), body))
                .then(
                    Equality::natural(
                        digits(h, w.clone()),
                        low.clone(),
                        ic(h, "DigitsIndex", vec![w.clone()]),
                    )
                    .on_left(mul(coefficient(h), bit_value(b.clone()))),
                )?
                .then(
                    mul_left(
                        Equality::natural(coefficient(h), cap(h), ic(h, "CapacityPower", vec![])),
                        bit_value(b.clone()),
                    )
                    .on_right(low.clone()),
                )?;
            let next = choose(b.clone(), add(cap(h), low.clone()), low.clone(), nat());
            let e = e.then(Equality::natural(
                add(low.clone(), mul(cap(h), bit_value(b.clone()))),
                next.clone(),
                common("DigitIndexStep", vec![low, cap(h), b]),
            ))?;
            e.then(
                Equality::natural(
                    target.clone(),
                    next,
                    fv(d, "IndexDefinition", slot_address(d, w.clone())?),
                )
                .reverse(),
            )?
        };
        self.theorem(d, "DigitsIndex", &[("word", cube(6))], e)?;
        self.theorem(
            d,
            "BoundedIndex",
            &[
                ("word", cube(6)),
                ("bound", inside(value(6, w.clone()), cap(d))),
            ],
            Equality::natural(
                value(6, w.clone()),
                digits(d, w.clone()),
                ic(d, "BoundedValue", vec![w.clone(), var("bound")]),
            )
            .then(Equality::natural(
                digits(d, w.clone()),
                target,
                ic(d, "DigitsIndex", vec![w]),
            ))?,
        )
    }
    fn definitions(&mut self, d: usize) -> Result<()> {
        let (a, data) = (var("A"), var("data"));
        let args = [("A", Expr::Sort0), ("data", data_type(d, a.clone()))];
        for (name, body, ty) in [
            ("Read64", read_body(d, a.clone(), data.clone()), a.clone()),
            (
                "Sequence64",
                sequence_body(d, a.clone(), data.clone()),
                sequence(a.clone()),
            ),
        ] {
            self.emit(
                d,
                name,
                &args,
                function(cube(6), ty.clone()),
                body.clone(),
                false,
            )?;
            self.theorem(
                d,
                &format!("{name}Definition"),
                &args,
                Equality {
                    ty: function(cube(6), ty),
                    left: ic(d, name, vec![a.clone(), data.clone()]),
                    right: body.clone(),
                    proof: refl(
                        function(
                            cube(6),
                            if name == "Read64" {
                                a.clone()
                            } else {
                                sequence(a.clone())
                            },
                        ),
                        body,
                    ),
                },
            )?;
        }
        Ok(())
    }
    fn observations(&mut self, d: usize) -> Result<()> {
        let (a, data, w) = (var("A"), var("data"), var("word"));
        let args = [
            ("A", Expr::Sort0),
            ("data", data_type(d, a.clone())),
            ("word", cube(6)),
        ];
        let index = slot_index(d, w.clone())?;
        let original = fv(
            d,
            "Lookup",
            vec![a.clone(), data.clone(), value(6, w.clone())],
        );
        let indexed = fv(d, "Lookup", vec![a.clone(), data.clone(), index.clone()]);
        let actual = app(data.clone(), slot_address(d, w.clone())?);
        let index_equation = Equality::natural(
            value(6, w.clone()),
            index.clone(),
            ic(d, "BoundedIndex", vec![w.clone(), var("bound")]),
        )
        .mapped(
            lam(
                "_index",
                nat(),
                fv(d, "Lookup", vec![a.clone(), data.clone(), var("_index")]),
            ),
            original,
            indexed.clone(),
            a.clone(),
        );
        let mut slot_args = vec![a.clone(), data.clone()];
        slot_args.extend(slot_address(d, w.clone())?);
        let e = read_equation(d, a.clone(), data.clone(), w.clone())
            .then(index_equation)?
            .then(Equality {
                ty: a.clone(),
                left: indexed,
                right: actual,
                proof: fv(d, "LookupIndex", slot_args),
            })?;
        let mut bounded = args.to_vec();
        bounded.push(("bound", inside(value(6, w.clone()), cap(d))));
        self.theorem(d, "BoundedRead", &bounded, e)?;
        let sequence_value = full_sequence(d, a.clone(), data.clone(), w.clone());
        let embedded = fv(
            d,
            "Embed",
            vec![a.clone(), value(6, w.clone()), data.clone()],
        );
        let length = sequence_equation(d, a.clone(), data.clone(), w.clone())
            .mapped(
                lam(
                    "_sequence",
                    sequence(a.clone()),
                    sc("Length", vec![a.clone(), var("_sequence")]),
                ),
                sc("Length", vec![a.clone(), sequence_value]),
                sc("Length", vec![a.clone(), embedded.clone()]),
                nat(),
            )
            .then(Equality::natural(
                sc("Length", vec![a.clone(), embedded]),
                value(6, w.clone()),
                fv(d, "EmbedLength", vec![a, value(6, w), data]),
            ))?;
        self.theorem(d, "FullLength", &args, length)
    }

    fn lengths(&mut self, d: usize) -> Result<()> {
        let (a, p, data, len, index) =
            (var("A"), var("P"), var("data"), var("length"), var("index"));
        let n = value(6, len.clone());
        let i = value(6, index.clone());
        let fits = pi(
            "_point",
            nat(),
            pi(
                "_in",
                inside(var("_point"), n.clone()),
                inside(var("_point"), cap(d)),
            ),
        );
        let fits_proof = lam(
            "_point",
            nat(),
            lam(
                "_in",
                inside(var("_point"), n.clone()),
                common(
                    "UpperBound",
                    vec![cap(d), n.clone(), var("_point"), var("upper"), var("_in")],
                ),
            ),
        );
        self.emit(
            d,
            "FullLengthFits",
            &[("length", cube(6)), ("upper", upper(n.clone(), cap(d)))],
            fits,
            fits_proof,
            true,
        )?;
        let input = [
            ("A", Expr::Sort0),
            ("data", data_type(d, a.clone())),
            ("length", cube(6)),
            ("index", cube(6)),
        ];
        let actual_sequence = full_sequence(d, a.clone(), data.clone(), len.clone());
        let embedding = fv(d, "Embed", vec![a.clone(), n.clone(), data.clone()]);
        let observed = sc("At", vec![a.clone(), actual_sequence.clone(), i.clone()]);
        let embedded = sc("At", vec![a.clone(), embedding, i.clone()]);
        let mathematical = fv(d, "Lookup", vec![a.clone(), data.clone(), i.clone()]);
        let e = sequence_equation(d, a.clone(), data.clone(), len.clone())
            .mapped(
                lam(
                    "_sequence",
                    sequence(a.clone()),
                    sc("At", vec![a.clone(), var("_sequence"), i.clone()]),
                ),
                observed,
                embedded.clone(),
                a.clone(),
            )
            .then(Equality {
                ty: a.clone(),
                left: embedded,
                right: mathematical,
                proof: fv(
                    d,
                    "EmbedAt",
                    vec![a.clone(), n.clone(), data.clone(), i.clone()],
                ),
            })?
            .then(read_equation(d, a.clone(), data.clone(), index.clone()).reverse())?;
        self.theorem(d, "SequenceRead", &input, e)?;
        let bound = inside(i.clone(), n.clone());
        let mut guarded = input.to_vec();
        guarded.push(("within", bound.clone()));
        let length = sc("Length", vec![a.clone(), actual_sequence.clone()]);
        let e = less_right(
            i.clone(),
            Equality::natural(
                length,
                n.clone(),
                ic(d, "FullLength", vec![a.clone(), data.clone(), len.clone()]),
            ),
        )
        .then(Equality::boolean(
            less(i.clone(), n.clone()),
            bit(true),
            var("within"),
        ))?;
        self.emit(
            d,
            "WithinLengthValid",
            &guarded,
            sc(
                "IndexValid",
                vec![a.clone(), actual_sequence.clone(), i.clone()],
            ),
            e.proof,
            true,
        )?;
        let mut bounded = input.to_vec();
        bounded.extend([("upper", upper(n.clone(), cap(d))), ("within", bound)]);
        let cap_bound = common(
            "UpperBound",
            vec![cap(d), n.clone(), i.clone(), var("upper"), var("within")],
        );
        let observed = app(data.clone(), slot_address(d, index.clone())?);
        self.theorem(
            d,
            "ReadWithinLength",
            &bounded,
            Equality {
                ty: a.clone(),
                left: full_read(d, a.clone(), data.clone(), index.clone()),
                right: observed.clone(),
                proof: ic(
                    d,
                    "BoundedRead",
                    vec![a.clone(), data.clone(), index.clone(), cap_bound],
                ),
            },
        )?;
        let within_slot = Equality {
            ty: a.clone(),
            left: sc("At", vec![a.clone(), actual_sequence.clone(), i]),
            right: full_read(d, a.clone(), data.clone(), index.clone()),
            proof: ic(
                d,
                "SequenceRead",
                vec![a.clone(), data.clone(), len.clone(), index.clone()],
            ),
        }
        .then(Equality {
            ty: a.clone(),
            left: full_read(d, a.clone(), data.clone(), index.clone()),
            right: observed,
            proof: ic(
                d,
                "ReadWithinLength",
                vec![
                    a.clone(),
                    data.clone(),
                    len.clone(),
                    index,
                    var("upper"),
                    var("within"),
                ],
            ),
        })?;
        self.theorem(d, "SequenceWithinSlot", &bounded, within_slot)?;
        let eq = sequence_equation(d, a.clone(), data.clone(), len.clone()).reverse();
        let element_valid = fv(
            d,
            "EmbedValid",
            vec![
                a.clone(),
                p.clone(),
                n.clone(),
                data.clone(),
                ic(d, "FullLengthFits", vec![len.clone(), var("upper")]),
                var("valid"),
            ],
        );
        let proof = call(
            "Std.Eq.rewrite",
            vec![
                sequence(a.clone()),
                eq.left,
                eq.right,
                lam(
                    "_sequence",
                    sequence(a.clone()),
                    sc("Valid", vec![a.clone(), p.clone(), var("_sequence")]),
                ),
                eq.proof,
                element_valid,
            ],
        );
        self.emit(
            d,
            "FullSequenceValid",
            &[
                ("A", Expr::Sort0),
                ("P", function(a.clone(), Expr::Sort0)),
                ("data", data_type(d, a.clone())),
                ("length", cube(6)),
                ("upper", upper(n, cap(d))),
                ("valid", slots_valid(d, p.clone(), data)),
            ],
            sc("Valid", vec![a, p, actual_sequence]),
            proof,
            true,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    natural::append(builder)?;
    let mut indices = Indices(builder);
    for d in 0..=8 {
        indices.capacity(d)?;
        indices.branches(d)?;
        indices.digits(d)?;
        indices.definitions(d)?;
        indices.observations(d)?;
    }
    upper::append(indices.0)?;
    for d in 0..=8 {
        indices.lengths(d)?;
    }
    Ok(())
}
