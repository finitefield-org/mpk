//! Real full64 length bounds connect observable ranks and string equality.
//! Low address bits are used only after the actual strict bound is proved.
use super::*;

fn bounded(name: &str, args: Vec<Expr>) -> Expr {
    oc(&format!("Bounded.{name}"), args)
}
fn slot(s: Expr, at: Vec<Expr>) -> Expr {
    app(data(s), at)
}
fn present(s: Expr, at: Vec<Expr>) -> Expr {
    less(index(at), size(s))
}
fn rank_same(s: Expr, t: Expr) -> Expr {
    let at = address_bits(&POS);
    cube(&POS, eq(nat(), rank(s, at.clone()), rank(t, at)), false)
}
fn byte_same(x: Expr, y: Expr) -> Expr {
    let at = address_bits(&BITS[..3]);
    cube(
        &BITS[..3],
        eq(boolean(), app(x, at.clone()), app(y, at)),
        false,
    )
}
fn content(s: Expr, t: Expr) -> Expr {
    let at = address_bits(&POS);
    cube(
        &POS,
        pi(
            "inside",
            truth(present(s.clone(), at.clone())),
            byte_same(slot(s, at.clone()), slot(t, at)),
        ),
        false,
    )
}
fn rank_equal(s: Expr, t: Expr) -> Expr {
    deq(8, ranks(s), ranks(t))
}
fn string_equal(s: Expr, t: Expr) -> Expr {
    ec("Equal", vec![s, t])
}
fn order(name: &str, s: Expr, t: Expr) -> Expr {
    oc(name, vec![s, t])
}
fn at_args(mut args: Vec<Expr>, at: Vec<Expr>, witness: Option<Expr>) -> Vec<Expr> {
    args.extend(at);
    args.extend(witness);
    args
}
fn rank_case(s: Expr, at: Vec<Expr>, inside: bool, witness: Expr) -> Equality {
    Equality::natural(
        rank(s.clone(), at.clone()),
        if inside {
            succ(numeric_byte(slot(s.clone(), at.clone())))
        } else {
            zero()
        },
        oc(
            if inside { "RankInside" } else { "RankOutside" },
            at_args(vec![s], at, Some(witness)),
        ),
    )
}
fn length_address(s: Expr) -> Result<Vec<Expr>> {
    (0..8)
        .rev()
        .map(|i| word::at(length(s.clone()), 64, i))
        .collect()
}

struct Bounded<'a>(&'a mut Builder);
impl Bounded<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Order.Bounded.{name}"),
            args,
            ty,
            body,
            true,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof)
    }
    fn lengths(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let address = length_address(s.clone())?;
        self.theorem(
            "LengthAddress",
            &[
                ("s", string_type()),
                ("small", truth(less(size(s.clone()), capacity()))),
            ],
            Equality::natural(
                size(s.clone()),
                index(address.clone()),
                ix("BoundedIndex", vec![length(s.clone()), var("small")]),
            ),
        )?;
        let ns = size(s.clone());
        let nt = size(t.clone());
        let lt = less(ns.clone(), nt.clone());
        let result = eq(boolean(), lt.clone(), bit(false));
        let small = call(
            "Mpk.Go.Practical.Word.Index.UpperBound",
            vec![
                capacity(),
                nt.clone(),
                ns.clone(),
                var("upper_t"),
                var("_branch"),
            ],
        );
        let i = index(address.clone());
        let index_eq = Equality::natural(
            ns.clone(),
            i.clone(),
            bounded("LengthAddress", vec![s.clone(), small]),
        )
        .reverse();
        let outside = less_left(index_eq.clone(), ns.clone()).then(Equality::boolean(
            less(ns.clone(), ns.clone()),
            bit(false),
            nc("LessIrreflexive", vec![ns.clone()]),
        ))?;
        let inside = less_left(index_eq, nt.clone()).then(Equality::boolean(
            lt.clone(),
            bit(true),
            var("_branch"),
        ))?;
        let x = rank_case(s.clone(), address.clone(), false, outside.proof);
        let y = rank_case(t.clone(), address.clone(), true, inside.proof);
        let equality = Equality::natural(
            rank(s.clone(), address.clone()),
            rank(t.clone(), address.clone()),
            app(var("same"), address.clone()),
        );
        let impossible = x.reverse().then(equality)?.then(y)?;
        let v = numeric_byte(slot(t.clone(), address));
        let contradiction = Equality::boolean(
            equal(zero(), succ(v.clone())),
            bit(false),
            nc("EqualZeroSucc", vec![v.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            equal(zero(), succ(v.clone())),
            bit(true),
            nc("EqualComplete", vec![zero(), succ(v), impossible.proof]),
        ))?;
        let proof = decide(
            lt.clone(),
            result.clone(),
            var("_branch"),
            bc("FalseElim", vec![result, contradiction.proof]),
        );
        self.theorem(
            "LengthNotLess",
            &[
                ("s", string_type()),
                ("t", string_type()),
                ("upper_t", upper(t.clone())),
                ("same", rank_same(s.clone(), t.clone())),
            ],
            Equality::boolean(lt, bit(false), proof),
        )?;
        let at = address_bits(&POS);
        let reverse = cube(
            &POS,
            call(
                "Std.Eq.symm",
                vec![
                    nat(),
                    rank(s.clone(), at.clone()),
                    rank(t.clone(), at.clone()),
                    app(var("same"), at),
                ],
            ),
            true,
        );
        let left = Equality::boolean(
            less(ns.clone(), nt.clone()),
            bit(false),
            bounded(
                "LengthNotLess",
                vec![s.clone(), t.clone(), var("upper_t"), var("same")],
            ),
        );
        let right = Equality::boolean(
            less(nt.clone(), ns.clone()),
            bit(false),
            bounded(
                "LengthNotLess",
                vec![t.clone(), s.clone(), var("upper_s"), reverse],
            ),
        );
        let left = neg(left).then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        let right = neg(right).then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        let total = flag_total(less(ns.clone(), nt.clone()), less(nt.clone(), ns.clone()));
        let truth_total = ec(
            "AndComplete",
            vec![left.left, right.left, left.proof, right.proof],
        );
        let natural = Equality::boolean(
            total.clone(),
            equal(ns.clone(), nt.clone()),
            call(
                "Mpk.Go.Practical.Word.Order.NaturalTotalEquality",
                vec![ns.clone(), nt.clone()],
            ),
        )
        .reverse()
        .then(Equality::boolean(total, bit(true), truth_total))?;
        self.theorem(
            "LengthEqual",
            &[
                ("s", string_type()),
                ("t", string_type()),
                ("upper_s", upper(s)),
                ("upper_t", upper(t)),
                ("same", rank_same(var("s"), var("t"))),
            ],
            Equality::natural(
                ns.clone(),
                nt.clone(),
                nc("EqualReflect", vec![ns, nt, natural.proof]),
            ),
        )
    }
    fn byte_contents(&mut self) -> Result<()> {
        let (a, b) = (var("a"), var("b"));
        let e = Equality::natural(succ(a.clone()), succ(b.clone()), var("same")).mapped(
            constant("Mpk.Go.Practical.Nat.Pred"),
            pred(succ(a.clone())),
            pred(succ(b.clone())),
            nat(),
        );
        let e = Equality::natural(
            pred(succ(a.clone())),
            a.clone(),
            nc("PredSucc", vec![a.clone()]),
        )
        .reverse()
        .then(e)?
        .then(Equality::natural(
            pred(succ(b.clone())),
            b.clone(),
            nc("PredSucc", vec![b.clone()]),
        ))?;
        self.theorem(
            "SuccCancel",
            &[
                ("a", nat()),
                ("b", nat()),
                ("same", eq(nat(), succ(a), succ(b))),
            ],
            e,
        )?;
        let (s, t) = (var("s"), var("t"));
        let at = address_bits(&POS);
        let i = index(at.clone());
        let lengths = Equality::natural(size(s.clone()), size(t.clone()), var("lengths"));
        let right_guard = less_right(i, lengths.reverse()).then(Equality::boolean(
            present(s.clone(), at.clone()),
            bit(true),
            var("inside"),
        ))?;
        let left = rank_case(s.clone(), at.clone(), true, var("inside"));
        let right = rank_case(t.clone(), at.clone(), true, right_guard.proof);
        let values = left
            .reverse()
            .then(Equality::natural(
                rank(s.clone(), at.clone()),
                rank(t.clone(), at.clone()),
                var("same"),
            ))?
            .then(right)?;
        let av = numeric_byte(slot(s.clone(), at.clone()));
        let bv = numeric_byte(slot(t.clone(), at.clone()));
        let number = bounded("SuccCancel", vec![av.clone(), bv.clone(), values.proof]);
        let beq = ec(
            "ByteEqual",
            vec![slot(s.clone(), at.clone()), slot(t.clone(), at.clone())],
        );
        let bool_eq = Equality::boolean(
            beq.clone(),
            equal(av.clone(), bv.clone()),
            oc(
                "ByteEqualNatural",
                vec![slot(s.clone(), at.clone()), slot(t.clone(), at.clone())],
            ),
        )
        .then(Equality::boolean(
            equal(av.clone(), bv.clone()),
            bit(true),
            nc("EqualComplete", vec![av, bv, number]),
        ))?;
        let mut args = vec![
            ("s", string_type()),
            ("t", string_type()),
            ("lengths", eq(nat(), size(s.clone()), size(t.clone()))),
        ];
        args.extend(POS.iter().map(|name| (*name, boolean())));
        args.extend([
            ("inside", truth(present(s.clone(), at.clone()))),
            (
                "same",
                eq(
                    nat(),
                    rank(s.clone(), at.clone()),
                    rank(t.clone(), at.clone()),
                ),
            ),
        ]);
        self.emit(
            "RankByteEqual",
            &args,
            byte_same(slot(s.clone(), at.clone()), slot(t.clone(), at.clone())),
            ec(
                "ByteReflect",
                vec![slot(s, at.clone()), slot(t, at), bool_eq.proof],
            ),
        )
    }
    fn equivalence(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let pair = [("s", string_type()), ("t", string_type())];
        self.emit(
            "RanksReflect",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("h", truth(rank_equal(s.clone(), t.clone()))),
            ],
            rank_same(s.clone(), t.clone()),
            oc(
                "D8.EqualReflect",
                vec![ranks(s.clone()), ranks(t.clone()), var("h")],
            ),
        )?;
        self.theorem(
            "RanksComplete",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("same", rank_same(s.clone(), t.clone())),
            ],
            Equality::boolean(
                rank_equal(s.clone(), t.clone()),
                bit(true),
                oc(
                    "D8.EqualComplete",
                    vec![ranks(s.clone()), ranks(t.clone()), var("same")],
                ),
            ),
        )?;
        let bounded_pair = [
            pair[0].clone(),
            pair[1].clone(),
            ("upper_s", upper(s.clone())),
            ("upper_t", upper(t.clone())),
        ];
        let at = address_bits(&POS);
        let length_eq = bounded(
            "LengthEqual",
            vec![
                s.clone(),
                t.clone(),
                var("upper_s"),
                var("upper_t"),
                var("same"),
            ],
        );
        let mut data_args = vec![s.clone(), t.clone(), length_eq.clone()];
        data_args.extend(at.clone());
        data_args.extend([var("inside"), app(var("same"), at.clone())]);
        let contents = cube(
            &POS,
            lam(
                "inside",
                truth(present(s.clone(), at.clone())),
                bounded("RankByteEqual", data_args),
            ),
            true,
        );
        let mut args = bounded_pair.to_vec();
        args.push(("same", rank_same(s.clone(), t.clone())));
        self.emit(
            "FromPoints",
            &args,
            ec("Equivalent", vec![s.clone(), t.clone()]),
            pc(
                "Make",
                vec![
                    eq(nat(), size(s.clone()), size(t.clone())),
                    content(s.clone(), t.clone()),
                    length_eq,
                    contents,
                ],
            ),
        )?;
        let mut args = bounded_pair.to_vec();
        args.push(("h", truth(rank_equal(s.clone(), t.clone()))));
        self.emit(
            "FromRanks",
            &args,
            ec("Equivalent", vec![s.clone(), t.clone()]),
            bounded(
                "FromPoints",
                vec![
                    s.clone(),
                    t.clone(),
                    var("upper_s"),
                    var("upper_t"),
                    bounded("RanksReflect", vec![s.clone(), t.clone(), var("h")]),
                ],
            ),
        )?;
        let mut witness = vec![s.clone(), t.clone(), var("related")];
        witness.extend(at);
        self.theorem(
            "ToRanks",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("related", ec("Equivalent", vec![s.clone(), t.clone()])),
            ],
            Equality::boolean(
                rank_equal(s.clone(), t.clone()),
                bit(true),
                bounded(
                    "RanksComplete",
                    vec![
                        s.clone(),
                        t.clone(),
                        cube(&POS, oc("RankRespects", witness), true),
                    ],
                ),
            ),
        )?;
        let forward = lam(
            "h",
            truth(rank_equal(s.clone(), t.clone())),
            ec(
                "EqualComplete",
                vec![
                    s.clone(),
                    t.clone(),
                    bounded(
                        "FromRanks",
                        vec![
                            s.clone(),
                            t.clone(),
                            var("upper_s"),
                            var("upper_t"),
                            var("h"),
                        ],
                    ),
                ],
            ),
        );
        let backward = lam(
            "h",
            truth(string_equal(s.clone(), t.clone())),
            bounded(
                "ToRanks",
                vec![
                    s.clone(),
                    t.clone(),
                    ec("EqualReflect", vec![s.clone(), t.clone(), var("h")]),
                ],
            ),
        );
        self.theorem(
            "EqualRanks",
            &bounded_pair,
            Equality::boolean(
                rank_equal(s.clone(), t.clone()),
                string_equal(s.clone(), t.clone()),
                ec(
                    "TruthIff",
                    vec![
                        rank_equal(s.clone(), t.clone()),
                        string_equal(s, t),
                        forward,
                        backward,
                    ],
                ),
            ),
        )
    }
    fn comparisons(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let pair = [("s", string_type()), ("t", string_type())];
        let bounds = [
            pair[0].clone(),
            pair[1].clone(),
            ("upper_s", upper(s.clone())),
            ("upper_t", upper(t.clone())),
        ];
        let ranks = Equality::boolean(
            flag_total(
                order("Less", s.clone(), t.clone()),
                order("Less", t.clone(), s.clone()),
            ),
            rank_equal(s.clone(), t.clone()),
            oc("TotalRanksEqual", vec![s.clone(), t.clone()]),
        );
        self.theorem(
            "TotalEquality",
            &bounds,
            ranks.then(Equality::boolean(
                rank_equal(s.clone(), t.clone()),
                string_equal(s.clone(), t.clone()),
                bounded(
                    "EqualRanks",
                    vec![s.clone(), t.clone(), var("upper_s"), var("upper_t")],
                ),
            ))?,
        )?;
        let weak = Equality::boolean(
            order("LessEqual", s.clone(), t.clone()),
            bc("Not", vec![order("Greater", s.clone(), t.clone())]),
            oc("LessEqualComplement", vec![s.clone(), t.clone()]),
        )
        .then(neg(Equality::boolean(
            order("Greater", s.clone(), t.clone()),
            order("Less", t.clone(), s.clone()),
            oc("GreaterReverse", vec![s.clone(), t.clone()]),
        )))?;
        let greater = Equality::boolean(
            order("GreaterEqual", s.clone(), t.clone()),
            bc("Not", vec![order("Less", s.clone(), t.clone())]),
            oc("GreaterEqualComplement", vec![s.clone(), t.clone()]),
        );
        let e = bool_left("And", weak.clone(), greater.left.clone())
            .then(bool_right("And", weak.right, greater))?
            .then(Equality::boolean(
                flag_total(
                    order("Less", t.clone(), s.clone()),
                    order("Less", s.clone(), t.clone()),
                ),
                string_equal(t.clone(), s.clone()),
                bounded(
                    "TotalEquality",
                    vec![t.clone(), s.clone(), var("upper_t"), var("upper_s")],
                ),
            ))?
            .then(Equality::boolean(
                string_equal(t.clone(), s.clone()),
                string_equal(s.clone(), t.clone()),
                ec("EqualSym", vec![t.clone(), s.clone()]),
            ))?;
        self.theorem("EqualBounds", &bounds, e)?;
        self.theorem(
            "GreaterEqualReverse",
            &pair,
            Equality::boolean(
                order("GreaterEqual", s.clone(), t.clone()),
                order("LessEqual", t.clone(), s.clone()),
                refl(boolean(), order("LessEqual", t.clone(), s.clone())),
            ),
        )?;
        let mut args = bounds.to_vec();
        args.extend([
            ("forward", truth(order("LessEqual", s.clone(), t.clone()))),
            ("backward", truth(order("LessEqual", t.clone(), s.clone()))),
        ]);
        let right = Equality::boolean(
            order("GreaterEqual", s.clone(), t.clone()),
            order("LessEqual", t.clone(), s.clone()),
            bounded("GreaterEqualReverse", vec![s.clone(), t.clone()]),
        )
        .then(Equality::boolean(
            order("LessEqual", t.clone(), s.clone()),
            bit(true),
            var("backward"),
        ))?;
        let joined = ec(
            "AndComplete",
            vec![
                order("LessEqual", s.clone(), t.clone()),
                order("GreaterEqual", s.clone(), t.clone()),
                var("forward"),
                right.proof,
            ],
        );
        let e = Equality::boolean(
            bc(
                "And",
                vec![
                    order("LessEqual", s.clone(), t.clone()),
                    order("GreaterEqual", s.clone(), t.clone()),
                ],
            ),
            string_equal(s.clone(), t.clone()),
            bounded(
                "EqualBounds",
                vec![s.clone(), t.clone(), var("upper_s"), var("upper_t")],
            ),
        )
        .reverse()
        .then(Equality::boolean(
            bc(
                "And",
                vec![
                    order("LessEqual", s.clone(), t.clone()),
                    order("GreaterEqual", s.clone(), t.clone()),
                ],
            ),
            bit(true),
            joined,
        ))?;
        self.theorem("Antisym", &args, e)?;
        let related = ec("EqualReflect", vec![s.clone(), t.clone(), var("same")]);
        let reverse = ec("EquivalentSym", vec![s.clone(), t.clone(), related]);
        for (name, op) in [("EqualNoLess", "Less"), ("EqualNoGreater", "Greater")] {
            let e = Equality::boolean(
                order(op, s.clone(), t.clone()),
                order(op, s.clone(), s.clone()),
                oc(
                    &format!("{op}Respects"),
                    vec![
                        s.clone(),
                        t.clone(),
                        s.clone(),
                        s.clone(),
                        ec("EquivalentRefl", vec![s.clone()]),
                        reverse.clone(),
                    ],
                ),
            )
            .then(Equality::boolean(
                order(op, s.clone(), s.clone()),
                bit(false),
                oc(&format!("{op}Refl"), vec![s.clone()]),
            ))?;
            self.theorem(
                name,
                &[
                    pair[0].clone(),
                    pair[1].clone(),
                    ("same", truth(string_equal(s.clone(), t.clone()))),
                ],
                e,
            )?;
        }
        Ok(())
    }
    fn alternatives(&mut self) -> Result<()> {
        let formula = |a: Expr, b: Expr| {
            bc(
                "Or",
                vec![a.clone(), bc("Or", vec![flag_total(a, b.clone()), b])],
            )
        };
        let leaf = |a: bool, b: bool| -> Result<Expr> {
            let (flag, proof) = literal_boolean(formula(bit(a), bit(b)))?;
            if !flag {
                return Err(GoOrdinaryError::Linkage);
            }
            Ok(proof.proof)
        };
        let branch = |a: bool| -> Result<Expr> {
            Ok(lam(
                "b",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam("b", boolean(), truth(formula(bit(a), var("b")))),
                        leaf(a, false)?,
                        leaf(a, true)?,
                        var("b"),
                    ],
                ),
            ))
        };
        self.theorem(
            "ChoiceTotal",
            &[("a", boolean()), ("b", boolean())],
            Equality::boolean(
                formula(var("a"), var("b")),
                bit(true),
                app(
                    call(
                        "Std.Bool.cases",
                        vec![
                            lam(
                                "a",
                                boolean(),
                                pi("b", boolean(), truth(formula(var("a"), var("b")))),
                            ),
                            branch(false)?,
                            branch(true)?,
                            var("a"),
                        ],
                    ),
                    vec![var("b")],
                ),
            ),
        )?;
        let (s, t) = (var("s"), var("t"));
        let pair = [("s", string_type()), ("t", string_type())];
        let a = order("Less", s.clone(), t.clone());
        let b = order("Less", t.clone(), s.clone());
        let total = Equality::boolean(
            flag_total(a.clone(), b.clone()),
            string_equal(s.clone(), t.clone()),
            bounded(
                "TotalEquality",
                vec![s.clone(), t.clone(), var("upper_s"), var("upper_t")],
            ),
        );
        let e = bool_right("Or", a.clone(), bool_left("Or", total.reverse(), b.clone())).then(
            Equality::boolean(
                formula(a.clone(), b.clone()),
                bit(true),
                bounded("ChoiceTotal", vec![a.clone(), b.clone()]),
            ),
        )?;
        // Greater is the reversed strict comparison, by the original law.
        let reverse = Equality::boolean(
            order("Greater", s.clone(), t.clone()),
            b.clone(),
            oc("GreaterReverse", vec![s.clone(), t.clone()]),
        );
        let inner = bool_right("Or", string_equal(s.clone(), t.clone()), reverse);
        let e = bool_right("Or", a, inner).then(e)?;
        self.theorem(
            "Trichotomy",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("upper_s", upper(s.clone())),
                ("upper_t", upper(t.clone())),
            ],
            e,
        )?;
        for (name, op, missing) in [
            ("EqualLessEqual", "LessEqual", "EqualNoGreater"),
            ("EqualGreaterEqual", "GreaterEqual", "EqualNoLess"),
        ] {
            let strict = if op == "LessEqual" { "Greater" } else { "Less" };
            let no = Equality::boolean(
                order(strict, s.clone(), t.clone()),
                bit(false),
                bounded(missing, vec![s.clone(), t.clone(), var("same")]),
            );
            let complement = Equality::boolean(
                order(op, s.clone(), t.clone()),
                bc("Not", vec![order(strict, s.clone(), t.clone())]),
                oc(&format!("{op}Complement"), vec![s.clone(), t.clone()]),
            );
            self.theorem(
                name,
                &[
                    pair[0].clone(),
                    pair[1].clone(),
                    ("same", truth(string_equal(s.clone(), t.clone()))),
                ],
                complement.then(neg(no))?.then(Equality::boolean(
                    bc("Not", vec![bit(false)]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ))?,
            )?;
        }
        for (name, op, missing) in [
            ("LessNotEqual", "Less", "EqualNoLess"),
            ("GreaterNotEqual", "Greater", "EqualNoGreater"),
        ] {
            let flag = string_equal(s.clone(), t.clone());
            let result = eq(boolean(), flag.clone(), bit(false));
            let contradiction = Equality::boolean(
                order(op, s.clone(), t.clone()),
                bit(false),
                bounded(missing, vec![s.clone(), t.clone(), var("_branch")]),
            )
            .reverse()
            .then(Equality::boolean(
                order(op, s.clone(), t.clone()),
                bit(true),
                var("strict"),
            ))?;
            self.theorem(
                name,
                &[
                    pair[0].clone(),
                    pair[1].clone(),
                    ("strict", truth(order(op, s.clone(), t.clone()))),
                ],
                Equality::boolean(
                    flag.clone(),
                    bit(false),
                    decide(
                        flag,
                        result.clone(),
                        var("_branch"),
                        bc("FalseElim", vec![result, contradiction.proof]),
                    ),
                ),
            )?;
        }
        Ok(())
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut b = Bounded(builder);
    b.lengths()?;
    b.byte_contents()?;
    b.equivalence()?;
    b.comparisons()?;
    b.alternatives()
}
