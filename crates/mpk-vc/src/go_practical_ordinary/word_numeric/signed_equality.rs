//! Actual all-bit equality, actual sign coherence and balanced Int equality.
use super::super::word::fold_bits;
use super::*;

const EQUALITY: &str = "Mpk.Go.Practical.Word.SignedEquality";
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{EQUALITY}.{name}"), args)
}
fn oc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.SignedOrder.{name}"), args)
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn signed(w: usize, x: Expr) -> Expr {
    numeric(&format!("W{w}.Signed"), vec![x])
}
fn negate(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_eq_flag", boolean(), bc("Not", vec![var("_eq_flag")])),
        l,
        r,
        boolean(),
    )
}
fn bool_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        bc("And", vec![e.left.clone(), b.clone()]),
        bc("And", vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_eq_flag", boolean(), bc("And", vec![var("_eq_flag"), b])),
        l,
        r,
        boolean(),
    )
}
fn bool_right(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc("And", vec![a.clone(), e.left.clone()]),
        bc("And", vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_eq_flag", boolean(), bc("And", vec![a, var("_eq_flag")])),
        l,
        r,
        boolean(),
    )
}
fn equal_left(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        equal(e.left.clone(), b.clone()),
        equal(e.right.clone(), b.clone()),
    );
    e.mapped(lam("_eq_n", nat(), equal(var("_eq_n"), b)), l, r, boolean())
}
fn equal_right(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        equal(a.clone(), e.left.clone()),
        equal(a.clone(), e.right.clone()),
    );
    e.mapped(lam("_eq_n", nat(), equal(a, var("_eq_n"))), l, r, boolean())
}
fn total(a: Expr, b: Expr) -> Expr {
    bc(
        "And",
        vec![
            bc("Not", vec![less(a.clone(), b.clone())]),
            bc("Not", vec![less(b, a)]),
        ],
    )
}
fn nlaw(name: &str, args: Vec<Expr>, a: Expr, b: Expr) -> Equality {
    Equality::natural(a, b, nc(name, args))
}

struct SignedEquality<'a> {
    builder: &'a mut Builder,
}
impl SignedEquality<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, proof: Expr) -> Result<()> {
        emit(
            self.builder,
            &format!("{EQUALITY}.{name}"),
            args,
            ty,
            proof,
            true,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof)
    }
    fn boolean_laws(&mut self) -> Result<()> {
        for left in [true, false] {
            let property = |env: &BTreeMap<&'static str, Expr>| {
                let (a, b) = (env["p"].clone(), env["q"].clone());
                pi(
                    "truth",
                    eq(boolean(), bc("And", vec![a.clone(), b.clone()]), bit(true)),
                    eq(boolean(), if left { a } else { b }, bit(true)),
                )
            };
            let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let (a, b) = (env["p"].clone(), env["q"].clone());
                let claim = eq(
                    boolean(),
                    if left { a.clone() } else { b.clone() },
                    bit(true),
                );
                let and = bc("And", vec![a.clone(), b.clone()]);
                let proof = if a == bit(true) && b == bit(true) {
                    refl(boolean(), bit(true))
                } else {
                    let bad =
                        Equality::boolean(and.clone(), bit(false), refl(boolean(), bit(false)))
                            .reverse()
                            .then(Equality::boolean(and.clone(), bit(true), var("truth")))?;
                    bc("FalseElim", vec![claim, bad.proof])
                };
                Ok(lam("truth", eq(boolean(), and, bit(true)), proof))
            };
            let proof = boolean_cases(&["p", "q"], &BTreeMap::new(), &property, &mut leaf)?;
            let ty = pi(
                "p",
                boolean(),
                pi(
                    "q",
                    boolean(),
                    property(&[("p", var("p")), ("q", var("q"))].into_iter().collect()),
                ),
            );
            self.emit(
                if left { "AndTrueLeft" } else { "AndTrueRight" },
                &[],
                ty,
                proof,
            )?;
        }
        for left in [true, false] {
            let property = |env: &BTreeMap<&'static str, Expr>| {
                eq(
                    boolean(),
                    bc(
                        "And",
                        if left {
                            vec![bit(false), env["p"].clone()]
                        } else {
                            vec![env["p"].clone(), bit(false)]
                        },
                    ),
                    bit(false),
                )
            };
            let mut leaf = |_: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                Ok(refl(boolean(), bit(false)))
            };
            let proof = boolean_cases(&["p"], &BTreeMap::new(), &property, &mut leaf)?;
            self.emit(
                if left {
                    "AndFalseLeft"
                } else {
                    "AndFalseRight"
                },
                &[],
                pi(
                    "p",
                    boolean(),
                    property(&[("p", var("p"))].into_iter().collect()),
                ),
                proof,
            )?;
        }
        let property = |env: &BTreeMap<&'static str, Expr>| {
            let p = env["p"].clone();
            pi(
                "refute",
                pi(
                    "_true",
                    eq(boolean(), p.clone(), bit(true)),
                    eq(boolean(), bit(false), bit(true)),
                ),
                eq(boolean(), p, bit(false)),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let p = env["p"].clone();
            let refute = pi(
                "_true",
                eq(boolean(), p.clone(), bit(true)),
                eq(boolean(), bit(false), bit(true)),
            );
            let proof = if p == bit(false) {
                refl(boolean(), bit(false))
            } else {
                bc(
                    "FalseElim",
                    vec![
                        eq(boolean(), p, bit(false)),
                        app(var("refute"), vec![refl(boolean(), bit(true))]),
                    ],
                )
            };
            Ok(lam("refute", refute, proof))
        };
        let proof = boolean_cases(&["p"], &BTreeMap::new(), &property, &mut leaf)?;
        self.emit(
            "RefutationFalse",
            &[],
            pi(
                "p",
                boolean(),
                property(&[("p", var("p"))].into_iter().collect()),
            ),
            proof,
        )
    }
    fn strict_difference(&mut self) -> Result<()> {
        let (a, b) = (var("a"), var("b"));
        for reverse in [false, true] {
            let condition = if reverse {
                less(b.clone(), a.clone())
            } else {
                less(a.clone(), b.clone())
            };
            let args = [
                ("a", nat()),
                ("b", nat()),
                ("ordered", eq(boolean(), condition.clone(), bit(true))),
            ];
            let not = negate(Equality::boolean(condition, bit(true), var("ordered"))).then(
                Equality::boolean(
                    bc("Not", vec![bit(true)]),
                    bit(false),
                    refl(boolean(), bit(false)),
                ),
            )?;
            let first = bc("Not", vec![less(a.clone(), b.clone())]);
            let second = bc("Not", vec![less(b.clone(), a.clone())]);
            let step = if reverse {
                bool_right(first.clone(), not)
            } else {
                bool_left(not, second.clone())
            };
            let end = if reverse {
                bc("And", vec![first.clone(), bit(false)])
            } else {
                bc("And", vec![bit(false), second.clone()])
            };
            let proof = Equality::boolean(
                total(a.clone(), b.clone()),
                equal(a.clone(), b.clone()),
                oc("NaturalTotalEquality", vec![a.clone(), b.clone()]),
            )
            .reverse()
            .then(step)?
            .then(Equality::boolean(
                end,
                bit(false),
                ec(
                    if reverse {
                        "AndFalseRight"
                    } else {
                        "AndFalseLeft"
                    },
                    vec![if reverse { first } else { second }],
                ),
            ))?;
            self.theorem(
                if reverse {
                    "StrictNotEqualRight"
                } else {
                    "StrictNotEqualLeft"
                },
                &args,
                proof,
            )?;
        }
        Ok(())
    }
    fn offset(a: Expr, m: Expr, flag: bool) -> Result<Equality> {
        let product = mul_right(m.clone(), bit_literal(flag)).then(nlaw(
            if flag { "MulOne" } else { "MulZero" },
            vec![m.clone()],
            mul(m.clone(), if flag { one() } else { zero() }),
            if flag { m } else { zero() },
        ))?;
        let e = product.on_right(a.clone());
        if flag {
            Ok(e)
        } else {
            e.then(nlaw("AddZero", vec![a.clone()], add(a.clone(), zero()), a))
        }
    }
    fn scalar(&mut self) -> Result<()> {
        let (a, b, m) = (var("a"), var("b"), var("modulus"));
        let original = equal(a.clone(), b.clone());
        let args = [
            ("a", nat()),
            ("b", nat()),
            ("modulus", nat()),
            (
                "a_bound",
                eq(boolean(), less(a.clone(), m.clone()), bit(true)),
            ),
            (
                "b_bound",
                eq(boolean(), less(b.clone(), m.clone()), bit(true)),
            ),
        ];
        let coherent = |sx: Expr, sy: Expr| {
            pi(
                "_same",
                eq(boolean(), original.clone(), bit(true)),
                eq(boolean(), sx, sy),
            )
        };
        let property = |env: &BTreeMap<&'static str, Expr>| {
            pi(
                "same_sign",
                coherent(env["sx"].clone(), env["sy"].clone()),
                eq(
                    boolean(),
                    original.clone(),
                    equal(
                        add(a.clone(), mul(m.clone(), bit_value(env["sy"].clone()))),
                        add(b.clone(), mul(m.clone(), bit_value(env["sx"].clone()))),
                    ),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (sx, sy) = (env["sx"] == bit(true), env["sy"] == bit(true));
            let x = Self::offset(a.clone(), m.clone(), sy)?;
            let y = Self::offset(b.clone(), m.clone(), sx)?;
            let mut cross = equal_left(x.clone(), y.left.clone())
                .then(equal_right(x.right.clone(), y.clone()))?;
            let e = if sx == sy {
                if sx {
                    cross = cross.then(Equality::boolean(
                        equal(x.right, y.right),
                        original.clone(),
                        nc("EqualTranslate", vec![a.clone(), b.clone(), m.clone()]),
                    ))?;
                }
                cross.reverse()
            } else {
                let less_proof = if sx {
                    sc(
                        "BelowExtendedRight",
                        vec![a.clone(), m.clone(), b.clone(), var("a_bound")],
                    )
                } else {
                    sc(
                        "BelowExtendedRight",
                        vec![b.clone(), m.clone(), a.clone(), var("b_bound")],
                    )
                };
                cross = cross.then(Equality::boolean(
                    equal(x.right.clone(), y.right.clone()),
                    bit(false),
                    ec(
                        if sx {
                            "StrictNotEqualLeft"
                        } else {
                            "StrictNotEqualRight"
                        },
                        vec![x.right, y.right, less_proof],
                    ),
                ))?;
                let signs =
                    Equality::boolean(bit(sx), bit(sy), app(var("same_sign"), vec![var("_same")]));
                let bad = if sx { signs.reverse() } else { signs };
                let refute = lam(
                    "_same",
                    eq(boolean(), original.clone(), bit(true)),
                    bad.proof,
                );
                Equality::boolean(
                    original.clone(),
                    bit(false),
                    ec("RefutationFalse", vec![original.clone(), refute]),
                )
                .then(cross.reverse())?
            };
            Ok(lam("same_sign", coherent(bit(sx), bit(sy)), e.proof))
        };
        let proof = boolean_cases(&["sx", "sy"], &BTreeMap::new(), &property, &mut leaf)?;
        let ty = pi(
            "sx",
            boolean(),
            pi(
                "sy",
                boolean(),
                property(&[("sx", var("sx")), ("sy", var("sy"))].into_iter().collect()),
            ),
        );
        self.emit("SignedEqualityScalar", &args, ty, proof)
    }
    fn sign_coherence(&mut self, w: usize) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let mut args = vec![("x", word_type(w)?), ("y", word_type(w)?)];
        let source = wc(w, "Equal", vec![x.clone(), y.clone()]);
        args.push(("source_equal", eq(boolean(), source.clone(), bit(true))));
        let flags = (0..w)
            .map(|i| {
                Ok(bc(
                    "Equal",
                    vec![at(x.clone(), w, i)?, at(y.clone(), w, i)?],
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let body = fold_bits("And", &flags, true);
        let mut truth = Equality::boolean(source.clone(), body.clone(), refl(boolean(), body))
            .reverse()
            .then(Equality::boolean(source, bit(true), var("source_equal")))?;
        let mut values = flags.as_slice();
        while values.len() > 1 {
            let mid = values.len() / 2;
            let a = fold_bits("And", &values[..mid], true);
            let b = fold_bits("And", &values[mid..], true);
            truth = Equality::boolean(
                b.clone(),
                bit(true),
                ec("AndTrueRight", vec![a, b, truth.proof]),
            );
            values = &values[mid..];
        }
        let sx = at(x.clone(), w, w - 1)?;
        let sy = at(y.clone(), w, w - 1)?;
        self.theorem(
            &format!("W{w}.SourceEqualPreservesSign"),
            &args,
            Equality::boolean(
                sx.clone(),
                sy.clone(),
                bc("EqualReflect", vec![sx.clone(), sy.clone(), truth.proof]),
            ),
        )?;
        args.pop();
        let natural = equal(value(depth(w)?, x.clone()), value(depth(w)?, y.clone()));
        args.push(("same_value", eq(boolean(), natural.clone(), bit(true))));
        let actual = wc(w, "Equal", vec![x.clone(), y.clone()]);
        let proof = Equality::boolean(
            actual.clone(),
            natural.clone(),
            oc(&format!("W{w}.EqualNatural"), vec![x.clone(), y.clone()]),
        )
        .then(Equality::boolean(natural, bit(true), var("same_value")))?;
        self.theorem(
            &format!("W{w}.ValueEqualPreservesSign"),
            &args,
            Equality::boolean(
                sx,
                sy,
                ec(
                    &format!("W{w}.SourceEqualPreservesSign"),
                    vec![x, y, proof.proof],
                ),
            ),
        )
    }
    fn mathematical_equal(w: usize, x: Expr, y: Expr) -> Result<Equality> {
        let d = depth(w)?;
        let (sx, sy) = (signed(w, x.clone()), signed(w, y.clone()));
        let p = |s| ic("Positive", vec![s]);
        let n = |s| ic("Negative", vec![s]);
        let left = add(p(sx.clone()), n(sy.clone()));
        let right = add(p(sy.clone()), n(sx.clone()));
        let base = Equality::boolean(
            ic("Equal", vec![sx, sy]),
            equal(left.clone(), right.clone()),
            refl(boolean(), equal(left, right.clone())),
        );
        let part = |z: Expr, negative: bool| -> Result<Equality> {
            Ok(Equality::natural(
                if negative {
                    n(signed(w, z.clone()))
                } else {
                    p(signed(w, z.clone()))
                },
                if negative {
                    mul(weight(d), bit_value(at(z.clone(), w, w - 1)?))
                } else {
                    value(d, z.clone())
                },
                numeric(
                    &format!(
                        "W{w}.{}",
                        if negative {
                            "SignedNegative"
                        } else {
                            "SignedPositive"
                        }
                    ),
                    vec![z],
                ),
            ))
        };
        let px = part(x.clone(), false)?;
        let ny = part(y.clone(), true)?;
        let py = part(y, false)?;
        let nx = part(x, true)?;
        let a = px
            .clone()
            .on_left(ny.left.clone())
            .then(ny.on_right(px.right))?;
        let b = py
            .clone()
            .on_left(nx.left.clone())
            .then(nx.on_right(py.right))?;
        base.then(equal_left(a.clone(), b.left.clone()))?
            .then(equal_right(a.right, b))
    }
    fn family(&mut self, w: usize) -> Result<()> {
        self.sign_coherence(w)?;
        let (x, y) = (var("x"), var("y"));
        let d = depth(w)?;
        let args = [("x", word_type(w)?), ("y", word_type(w)?)];
        let source = wc(w, "Equal", vec![x.clone(), y.clone()]);
        let natural = equal(value(d, x.clone()), value(d, y.clone()));
        let start = Equality::boolean(
            source.clone(),
            natural.clone(),
            oc(&format!("W{w}.EqualNatural"), vec![x.clone(), y.clone()]),
        );
        let math = Self::mathematical_equal(w, x.clone(), y.clone())?;
        let signs = lam(
            "_same",
            eq(boolean(), natural.clone(), bit(true)),
            ec(
                &format!("W{w}.ValueEqualPreservesSign"),
                vec![x.clone(), y.clone(), var("_same")],
            ),
        );
        let scalar = Equality::boolean(
            natural,
            math.right.clone(),
            ec(
                "SignedEqualityScalar",
                vec![
                    value(d, x.clone()),
                    value(d, y.clone()),
                    weight(d),
                    oc(&format!("W{w}.ValueBound"), vec![x.clone()]),
                    oc(&format!("W{w}.ValueBound"), vec![y.clone()]),
                    at(x.clone(), w, w - 1)?,
                    at(y.clone(), w, w - 1)?,
                    signs,
                ],
            ),
        );
        let e = start.then(scalar)?.then(math.reverse())?;
        self.theorem(&format!("W{w}.EqualInteger"), &args, e.clone())?;
        let not = wc(w, "NotEqual", vec![x.clone(), y.clone()]);
        let reflected = Equality::boolean(
            not,
            bc("Not", vec![source.clone()]),
            refl(boolean(), bc("Not", vec![source.clone()])),
        )
        .then(negate(e.clone()))?;
        self.theorem(&format!("W{w}.NotEqualInteger"), &args, reflected)?;
        let (sx, sy) = (signed(w, x.clone()), signed(w, y.clone()));
        let mut reflect = args.to_vec();
        reflect.push(("source_equal", eq(boolean(), source.clone(), bit(true))));
        let integer_truth = e.clone().reverse().then(Equality::boolean(
            source.clone(),
            bit(true),
            var("source_equal"),
        ))?;
        self.emit(
            &format!("W{w}.EquivalentReflect"),
            &reflect,
            ic("Equivalent", vec![sx.clone(), sy.clone()]),
            ic(
                "EqualReflect",
                vec![sx.clone(), sy.clone(), integer_truth.proof],
            ),
        )?;
        let mut complete = args.to_vec();
        complete.push((
            "same_integer",
            ic("Equivalent", vec![sx.clone(), sy.clone()]),
        ));
        let truth = Equality::boolean(
            ic("Equal", vec![sx.clone(), sy.clone()]),
            bit(true),
            ic("EqualFromEquivalent", vec![sx, sy, var("same_integer")]),
        );
        self.theorem(
            &format!("W{w}.EquivalentComplete"),
            &complete,
            e.then(truth)?,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut equality = SignedEquality { builder };
    equality.boolean_laws()?;
    equality.strict_difference()?;
    equality.scalar()?;
    for width in WIDTHS {
        equality.family(width)?;
    }
    Ok(())
}
