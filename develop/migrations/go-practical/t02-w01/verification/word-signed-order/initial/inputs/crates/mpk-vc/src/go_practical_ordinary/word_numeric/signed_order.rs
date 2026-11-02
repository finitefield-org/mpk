//! Actual signed word comparisons agree with balanced unbounded integers.
use super::*;

const SIGNED_ORDER: &str = "Mpk.Go.Practical.Word.SignedOrder";
fn sc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{SIGNED_ORDER}.{name}"), args)
}
fn oc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn signed(width: usize, x: Expr) -> Expr {
    numeric(&format!("W{width}.Signed"), vec![x])
}
fn less_left(e: Equality, b: Expr) -> Equality {
    let (left, right) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(
        lam("_signed_order_n", nat(), less(var("_signed_order_n"), b)),
        left,
        right,
        boolean(),
    )
}
fn less_right(a: Expr, e: Equality) -> Equality {
    let (left, right) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(
        lam("_signed_order_n", nat(), less(a, var("_signed_order_n"))),
        left,
        right,
        boolean(),
    )
}
fn negate(e: Equality) -> Equality {
    let (left, right) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam(
            "_signed_order_flag",
            boolean(),
            bc("Not", vec![var("_signed_order_flag")]),
        ),
        left,
        right,
        boolean(),
    )
}
fn branch(a: Expr, b: Expr, ordinary: Expr) -> Expr {
    choose(bc("Xor", vec![a.clone(), b]), a, ordinary, boolean())
}
fn nlaw(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}

struct SignedOrder<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl SignedOrder<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, proof: Expr) -> Result<()> {
        emit(
            self.builder,
            &format!("{SIGNED_ORDER}.{name}"),
            args,
            ty,
            proof,
            true,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof)
    }
    fn natural_extensions(&mut self) -> Result<()> {
        let (a, m, b) = (var("a"), var("modulus"), var("b"));
        let args = [
            ("a", nat()),
            ("modulus", nat()),
            ("b", nat()),
            (
                "bound",
                eq(boolean(), less(a.clone(), m.clone()), bit(true)),
            ),
        ];
        let g = sub(m.clone(), succ(a.clone()));
        let expanded = add(add(a.clone(), g.clone()), one());
        let start = Equality::natural(
            m.clone(),
            expanded,
            call(
                "Mpk.Go.Practical.Word.Shifts.RightStrictGap",
                vec![a.clone(), m.clone(), var("bound")],
            ),
        )
        .on_right(b.clone());
        let middle = add(a.clone(), add(add(b.clone(), g.clone()), one()));
        let poly = polynomial::prove_named(
            self.builder,
            &args,
            &mut self.serial,
            SIGNED_ORDER,
            start.right.clone(),
            middle,
        )?;
        let e = start.then(poly)?.then(
            nlaw(
                "AddOne",
                vec![add(b.clone(), g.clone())],
                add(add(b.clone(), g.clone()), one()),
                succ(add(b.clone(), g.clone())),
            )
            .on_right(a.clone()),
        )?;
        self.theorem("BelowExtensionBalance", &args, e.clone())?;
        let gap = add(b.clone(), g);
        let named = Equality::natural(
            add(b.clone(), m.clone()),
            add(a.clone(), succ(gap.clone())),
            sc(
                "BelowExtensionBalance",
                vec![a.clone(), m.clone(), b.clone(), var("bound")],
            ),
        );
        let bound = less_right(a.clone(), named.clone()).then(Equality::boolean(
            less(a.clone(), add(a.clone(), succ(gap.clone()))),
            bit(true),
            oc("LessPositiveExtension", vec![a.clone(), gap.clone()]),
        ))?;
        self.theorem("BelowExtendedRight", &args, bound)?;
        let left = nlaw(
            "AddComm",
            vec![a.clone(), succ(gap.clone())],
            add(a.clone(), succ(gap.clone())),
            add(succ(gap.clone()), a.clone()),
        );
        let right = nlaw(
            "ZeroAdd",
            vec![a.clone()],
            add(zero(), a.clone()),
            a.clone(),
        )
        .reverse();
        let e = less_left(named.then(left)?, a.clone())
            .then(less_right(add(succ(gap.clone()), a.clone()), right))?
            .then(Equality::boolean(
                less(add(succ(gap.clone()), a.clone()), add(zero(), a.clone())),
                less(succ(gap.clone()), zero()),
                nc("LessTranslate", vec![succ(gap.clone()), zero(), a]),
            ))?
            .then(Equality::boolean(
                less(succ(gap.clone()), zero()),
                bit(false),
                nc("LessZero", vec![succ(gap)]),
            ))?;
        self.theorem("ExtendedRightNotBelow", &args, e)
    }
    fn sign_product(m: Expr, flag: bool) -> Result<Equality> {
        mul_right(m.clone(), bit_literal(flag)).then(nlaw(
            if flag { "MulOne" } else { "MulZero" },
            vec![m.clone()],
            mul(m.clone(), if flag { one() } else { zero() }),
            if flag { m } else { zero() },
        ))
    }
    fn offset(a: Expr, m: Expr, flag: bool) -> Result<Equality> {
        let e = Self::sign_product(m, flag)?.on_right(a.clone());
        if flag {
            Ok(e)
        } else {
            e.then(nlaw("AddZero", vec![a.clone()], add(a.clone(), zero()), a))
        }
    }
    fn scalar(&mut self) -> Result<()> {
        let (a, b, m) = (var("a"), var("b"), var("modulus"));
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
        let property = |env: &BTreeMap<&'static str, Expr>| {
            eq(
                boolean(),
                branch(
                    env["sx"].clone(),
                    env["sy"].clone(),
                    less(a.clone(), b.clone()),
                ),
                less(
                    add(a.clone(), mul(m.clone(), bit_value(env["sy"].clone()))),
                    add(b.clone(), mul(m.clone(), bit_value(env["sx"].clone()))),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (sx, sy) = (env["sx"] == bit(true), env["sy"] == bit(true));
            let x = Self::offset(a.clone(), m.clone(), sy)?;
            let y = Self::offset(b.clone(), m.clone(), sx)?;
            let mut e = less_left(x.clone(), y.left.clone())
                .then(less_right(x.right.clone(), y.clone()))?;
            let expected = if sx == sy {
                if sx {
                    e = e.then(Equality::boolean(
                        less(x.right, y.right),
                        less(a.clone(), b.clone()),
                        nc("LessTranslate", vec![a.clone(), b.clone(), m.clone()]),
                    ))?;
                }
                less(a.clone(), b.clone())
            } else if sx {
                e = e.then(Equality::boolean(
                    less(x.right, y.right),
                    bit(true),
                    sc(
                        "BelowExtendedRight",
                        vec![a.clone(), m.clone(), b.clone(), var("a_bound")],
                    ),
                ))?;
                bit(true)
            } else {
                e = e.then(Equality::boolean(
                    less(x.right, y.right),
                    bit(false),
                    sc(
                        "ExtendedRightNotBelow",
                        vec![b.clone(), m.clone(), a.clone(), var("b_bound")],
                    ),
                ))?;
                bit(false)
            };
            Ok(Equality::boolean(
                branch(bit(sx), bit(sy), less(a.clone(), b.clone())),
                expected.clone(),
                refl(boolean(), expected),
            )
            .then(e.reverse())?
            .proof)
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
        self.emit("SignedLessScalar", &args, ty, proof)
    }
    fn mathematical_less(width: usize, x: Expr, y: Expr) -> Result<Equality> {
        let d = depth(width)?;
        let (sx, sy) = (signed(width, x.clone()), signed(width, y.clone()));
        let p = |s: Expr| ic("Positive", vec![s]);
        let n = |s: Expr| ic("Negative", vec![s]);
        let left = add(p(sx.clone()), n(sy.clone()));
        let right = add(p(sy.clone()), n(sx.clone()));
        let base = Equality::boolean(
            ic("Less", vec![sx.clone(), sy.clone()]),
            less(left.clone(), right.clone()),
            refl(boolean(), less(left, right.clone())),
        );
        let projection = |word: Expr, negative: bool| -> Result<Equality> {
            Ok(Equality::natural(
                if negative {
                    n(signed(width, word.clone()))
                } else {
                    p(signed(width, word.clone()))
                },
                if negative {
                    mul(weight(d), bit_value(at(word.clone(), width, width - 1)?))
                } else {
                    value(d, word.clone())
                },
                numeric(
                    &format!(
                        "W{width}.{}",
                        if negative {
                            "SignedNegative"
                        } else {
                            "SignedPositive"
                        }
                    ),
                    vec![word],
                ),
            ))
        };
        let px = projection(x.clone(), false)?;
        let ny = projection(y.clone(), true)?;
        let py = projection(y, false)?;
        let nx = projection(x, true)?;
        let a = px
            .clone()
            .on_left(ny.left.clone())
            .then(ny.on_right(px.right))?;
        let b = py
            .clone()
            .on_left(nx.left.clone())
            .then(nx.on_right(py.right))?;
        base.then(less_left(a.clone(), b.left.clone()))?
            .then(less_right(a.right, b))
    }
    fn family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(width)?), ("y", word_type(width)?)];
        let (sx, sy) = (
            at(x.clone(), width, width - 1)?,
            at(y.clone(), width, width - 1)?,
        );
        let actual = wc(width, "SignedLess", vec![x.clone(), y.clone()]);
        let word_lt = wc(width, "UnsignedLess", vec![x.clone(), y.clone()]);
        let natural_lt = less(value(d, x.clone()), value(d, y.clone()));
        let source_branch = branch(sx.clone(), sy.clone(), word_lt.clone());
        let natural_branch = branch(sx.clone(), sy.clone(), natural_lt.clone());
        let source = Equality::boolean(
            actual.clone(),
            source_branch.clone(),
            refl(boolean(), source_branch.clone()),
        );
        let ordinary = Equality::boolean(
            word_lt,
            natural_lt,
            oc(
                &format!("W{width}.UnsignedLessNatural"),
                vec![x.clone(), y.clone()],
            ),
        );
        let switched = ordinary.mapped(
            lam(
                "_unsigned",
                boolean(),
                branch(sx.clone(), sy.clone(), var("_unsigned")),
            ),
            source_branch,
            natural_branch.clone(),
            boolean(),
        );
        let mathematical = Self::mathematical_less(width, x.clone(), y.clone())?;
        let scalar = Equality::boolean(
            natural_branch,
            mathematical.right.clone(),
            sc(
                "SignedLessScalar",
                vec![
                    value(d, x.clone()),
                    value(d, y.clone()),
                    weight(d),
                    oc(&format!("W{width}.ValueBound"), vec![x.clone()]),
                    oc(&format!("W{width}.ValueBound"), vec![y.clone()]),
                    sx,
                    sy,
                ],
            ),
        );
        self.theorem(
            &format!("W{width}.SignedLessInteger"),
            &args,
            source
                .then(switched)?
                .then(scalar)?
                .then(mathematical.reverse())?,
        )?;
        let source = wc(width, "SignedLessEqual", vec![x.clone(), y.clone()]);
        let reversed = wc(width, "SignedLess", vec![y.clone(), x.clone()]);
        let target = ic(
            "Less",
            vec![signed(width, y.clone()), signed(width, x.clone())],
        );
        let inner = Equality::boolean(
            reversed.clone(),
            target,
            sc(
                &format!("W{width}.SignedLessInteger"),
                vec![y.clone(), x.clone()],
            ),
        );
        let e = Equality::boolean(
            source,
            bc("Not", vec![reversed.clone()]),
            refl(boolean(), bc("Not", vec![reversed])),
        )
        .then(negate(inner))?;
        let end = ic("LessEqual", vec![signed(width, x), signed(width, y)]);
        self.theorem(
            &format!("W{width}.SignedLessEqualInteger"),
            &args,
            e.clone().then(Equality::boolean(
                e.right,
                end.clone(),
                refl(boolean(), end),
            ))?,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut order = SignedOrder { builder, serial: 0 };
    order.natural_extensions()?;
    order.scalar()?;
    for width in WIDTHS {
        order.family(width)?;
    }
    Ok(())
}
