//! Ordinary unbounded Nat denotation of the actual full-width word circuits.
//! Balanced decomposition shares weights; no unary machine-width literal,
//! operand enumeration, arithmetic opcode or extensionality axiom is used.
use super::polynomial::{self, mul_left, mul_right};
use super::syntax::*;
use super::word::{address, at, choose, depth, wc, word_type, ADDRESSES, WIDTHS};
use super::word_proofs::{boolean_cases, Evaluation};
use super::{Builder, GoOrdinaryError, Result};
use std::collections::BTreeMap;

mod arithmetic;
mod division;
mod order;
mod signed;
mod signed_product;

const PREFIX: &str = "Mpk.Go.Practical.Word.Numeric";

fn numeric(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn cube(d: usize) -> Expr {
    let mut ty = boolean();
    for name in ADDRESSES[..d].iter().rev() {
        ty = pi(name, boolean(), ty);
    }
    ty
}
fn bit_value(b: Expr) -> Expr {
    numeric("Bit", vec![b])
}
fn weight(d: usize) -> Expr {
    numeric(&format!("Weight{d}"), vec![])
}
fn value(d: usize, x: Expr) -> Expr {
    numeric(&format!("Value{d}"), vec![x])
}
fn project(x: Expr, high: bool) -> Expr {
    app(x, vec![bit(high)])
}
fn two() -> Expr {
    add(one(), one())
}
fn weight_body(d: usize) -> Expr {
    if d == 0 {
        two()
    } else {
        mul(weight(d - 1), weight(d - 1))
    }
}
fn value_body(d: usize, x: Expr) -> Expr {
    if d == 0 {
        bit_value(x)
    } else {
        add(
            value(d - 1, project(x.clone(), false)),
            mul(weight(d - 1), value(d - 1, project(x, true))),
        )
    }
}
fn weight_definition(d: usize) -> Equality {
    Equality::natural(
        weight(d),
        weight_body(d),
        numeric(&format!("WeightDefinition{d}"), vec![]),
    )
}
fn value_definition(d: usize, x: Expr) -> Equality {
    Equality::natural(
        value(d, x.clone()),
        value_body(d, x.clone()),
        numeric(&format!("ValueDefinition{d}"), vec![x]),
    )
}
fn bit_literal(b: bool) -> Equality {
    let n = if b { one() } else { zero() };
    Equality::natural(bit_value(bit(b)), n.clone(), refl(nat(), n))
}
fn map_bit(e: Equality) -> Equality {
    let left = bit_value(e.left.clone());
    let right = bit_value(e.right.clone());
    e.mapped(
        lam("_numeric_bit", boolean(), bit_value(var("_numeric_bit"))),
        left,
        right,
        nat(),
    )
}
fn bit_addition(a: Expr, b: Expr, c: Expr) -> (Expr, Expr) {
    (
        add(
            add(bit_value(a.clone()), bit_value(b.clone())),
            bit_value(c.clone()),
        ),
        add(
            bit_value(bc("Sum", vec![a.clone(), b.clone(), c.clone()])),
            mul(two(), bit_value(bc("Carry", vec![a, b, c]))),
        ),
    )
}

struct Numeric<'a> {
    builder: &'a mut Builder,
    recipes: &'a BTreeMap<String, Expr>,
    serial: usize,
}
impl Numeric<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.builder,
            &format!("{PREFIX}.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn polynomial(&mut self, args: &[Binder], left: Expr, right: Expr) -> Result<Equality> {
        polynomial::prove_named(self.builder, args, &mut self.serial, PREFIX, left, right)
    }
    fn definitions(&mut self) -> Result<()> {
        self.emit(
            "Bit",
            &[("b", boolean())],
            nat(),
            choose(var("b"), one(), zero(), nat()),
            false,
        )?;
        for d in 0..=6 {
            let w = weight_body(d);
            self.emit(&format!("Weight{d}"), &[], nat(), w.clone(), false)?;
            self.theorem(
                &format!("WeightDefinition{d}"),
                &[],
                Equality::natural(weight(d), w.clone(), refl(nat(), w)),
            )?;
            let x = var("x");
            let body = value_body(d, x.clone());
            self.emit(
                &format!("Value{d}"),
                &[("x", cube(d))],
                nat(),
                body.clone(),
                false,
            )?;
            self.theorem(
                &format!("ValueDefinition{d}"),
                &[("x", cube(d))],
                Equality::natural(value(d, x), body.clone(), refl(nat(), body)),
            )?;
        }
        Ok(())
    }
    fn local_addition(&mut self) -> Result<()> {
        let names = ["a", "b", "c"];
        let proposition = |e: &BTreeMap<&'static str, Expr>| {
            let (left, right) = bit_addition(e["a"].clone(), e["b"].clone(), e["c"].clone());
            eq(nat(), left, right)
        };
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (a, b, c) = (e["a"].clone(), e["b"].clone(), e["c"].clone());
            let (aa, bb, cc) = (a == bit(true), b == bit(true), c == bit(true));
            let s = aa ^ bb ^ cc;
            let carry = (aa && bb) || ((aa ^ bb) && cc);
            let mut evaluator = Evaluation::new(self.recipes);
            let sum = map_bit(
                evaluator.equation(bc("Sum", vec![a.clone(), b.clone(), c.clone()]), bit(s))?,
            )
            .then(bit_literal(s))?;
            let out = map_bit(evaluator.equation(bc("Carry", vec![a, b, c]), bit(carry))?)
                .then(bit_literal(carry))?;
            let ca = if aa { one() } else { zero() };
            let cb = if bb { one() } else { zero() };
            let (left, _) = bit_addition(bit(aa), bit(bb), bit(cc));
            let p = bit_literal(aa)
                .on_left(bit_value(bit(bb)))
                .then(bit_literal(bb).on_right(ca.clone()))?
                .on_left(bit_value(bit(cc)))
                .then(bit_literal(cc).on_right(add(ca, cb)))?;
            if p.left != left {
                return Err(GoOrdinaryError::Linkage);
            }
            let q = sum
                .clone()
                .on_left(mul(two(), out.left.clone()))
                .then(mul_right(two(), out).on_right(sum.right))?;
            let middle = self.polynomial(&[], p.right.clone(), q.right.clone())?;
            Ok(p.then(middle)?.then(q.reverse())?.proof)
        };
        let proof = boolean_cases(&names, &BTreeMap::new(), &proposition, &mut leaf)?;
        let mut ty = proposition(&names.iter().map(|n| (*n, var(n))).collect());
        for n in names.iter().rev() {
            ty = pi(n, boolean(), ty);
        }
        self.emit("BitAddition", &[], ty, proof, true)
    }
    fn value_congruence(&mut self) -> Result<()> {
        for d in 0..=6 {
            let (x, y) = (var("x"), var("y"));
            let indices: Vec<_> = ADDRESSES[..d].iter().map(|n| var(n)).collect();
            let mut premise = eq(
                boolean(),
                app(x.clone(), indices.clone()),
                app(y.clone(), indices),
            );
            for n in ADDRESSES[..d].iter().rev() {
                premise = pi(n, boolean(), premise);
            }
            let args = [("x", cube(d)), ("y", cube(d)), ("pointwise", premise)];
            let mut p = value_definition(d, x.clone());
            if d == 0 {
                p = p.then(map_bit(Equality::boolean(
                    x.clone(),
                    y.clone(),
                    var("pointwise"),
                )))?;
            } else {
                let child = |high| {
                    let xx = project(x.clone(), high);
                    let yy = project(y.clone(), high);
                    Equality::natural(
                        value(d - 1, xx.clone()),
                        value(d - 1, yy.clone()),
                        numeric(
                            &format!("ValueCongruence{}", d - 1),
                            vec![xx, yy, app(var("pointwise"), vec![bit(high)])],
                        ),
                    )
                };
                p = p
                    .then(
                        child(false)
                            .on_left(mul(weight(d - 1), value(d - 1, project(x.clone(), true)))),
                    )?
                    .then(
                        mul_right(weight(d - 1), child(true))
                            .on_right(value(d - 1, project(y.clone(), false))),
                    )?;
            }
            p = p.then(value_definition(d, y).reverse())?;
            self.theorem(&format!("ValueCongruence{d}"), &args, p)?;
        }
        Ok(())
    }
    fn combine(&mut self) -> Result<()> {
        let w = var("weight");
        let (a, b, ah, bh, lo, hi) = (
            var("a"),
            var("b"),
            var("ah"),
            var("bh"),
            var("lo"),
            var("hi"),
        );
        let (ci, cm, co) = (var("ci"), var("cm"), var("co"));
        let low_left = add(add(a.clone(), b.clone()), ci.clone());
        let low_right = add(lo.clone(), mul(w.clone(), cm.clone()));
        let high_left = add(add(ah.clone(), bh.clone()), cm.clone());
        let high_right = add(hi.clone(), mul(w.clone(), co.clone()));
        let mut args: Vec<Binder> = ["weight", "a", "b", "ah", "bh", "lo", "hi", "ci", "cm", "co"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect();
        args.extend([
            ("low", eq(nat(), low_left.clone(), low_right.clone())),
            ("high", eq(nat(), high_left.clone(), high_right.clone())),
        ]);
        let left = add(
            add(
                add(a, mul(w.clone(), ah.clone())),
                add(b, mul(w.clone(), bh.clone())),
            ),
            ci,
        );
        let stage = add(low_left.clone(), mul(w.clone(), add(ah, bh)));
        let p = self.polynomial(&args, left, stage.clone())?;
        let low = Equality::natural(low_left, low_right, var("low"));
        let high = Equality::natural(high_left.clone(), high_right, var("high"));
        let weighted_high = polynomial::split(&stage, "Add")
            .ok_or(GoOrdinaryError::Linkage)?
            .1;
        let p = p.then(low.on_left(weighted_high))?;
        let regrouped = add(lo.clone(), mul(w.clone(), high_left));
        let next = self.polynomial(&args, p.right.clone(), regrouped)?;
        let p = p
            .then(next)?
            .then(mul_right(w.clone(), high).on_right(lo.clone()))?;
        let right = add(add(lo, mul(w.clone(), hi)), mul(mul(w.clone(), w), co));
        let next = self.polynomial(&args, p.right.clone(), right)?;
        self.theorem("Combine", &args, p.then(next)?)
    }
    fn block(&mut self, width: usize, d: usize, offset: usize) -> Result<()> {
        let size = 1 << d;
        if d > 0 {
            self.block(width, d - 1, offset)?;
            self.block(width, d - 1, offset + size / 2)?;
        }
        let (x, y, incoming) = (var("x"), var("y"), var("incoming"));
        let args = [
            ("x", word_type(width)?),
            ("y", word_type(width)?),
            ("incoming", boolean()),
        ];
        let result = wc(
            width,
            "AddCarry",
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let sub = |word: Expr, at: usize, d: usize| -> Result<Expr> {
            Ok(app(word, address(width, at)?[..depth(width)? - d].to_vec()))
        };
        let xs = sub(x.clone(), offset, d)?;
        let ys = sub(y.clone(), offset, d)?;
        let rs = sub(result, offset, d)?;
        let carry = |i: usize| {
            if i == 0 {
                incoming.clone()
            } else {
                wc(
                    width,
                    &format!("Carry{i}"),
                    vec![x.clone(), y.clone(), incoming.clone()],
                )
            }
        };
        let ci = bit_value(carry(offset));
        let co = bit_value(carry(offset + size));
        let mut p = value_definition(d, xs.clone())
            .on_left(value(d, ys.clone()))
            .then(value_definition(d, ys.clone()).on_right(value_body(d, xs.clone())))?
            .on_left(ci.clone());
        if d == 0 {
            let sum = bc("Sum", vec![xs.clone(), ys.clone(), carry(offset)]);
            let out = bc("Carry", vec![xs.clone(), ys.clone(), carry(offset)]);
            let sum_name = format!("W{width}.SumBit{offset}");
            let carry_name = format!("W{width}.CarryStep{offset}");
            let sum_e = Equality::boolean(rs.clone(), sum.clone(), refl(boolean(), sum.clone()));
            let carry_e =
                Equality::boolean(carry(offset + 1), out.clone(), refl(boolean(), out.clone()));
            self.theorem(&sum_name, &args, sum_e.clone())?;
            self.theorem(&carry_name, &args, carry_e.clone())?;
            let (left, right) = bit_addition(xs.clone(), ys.clone(), carry(offset));
            p = p.then(Equality::natural(
                left,
                right,
                numeric("BitAddition", vec![xs, ys, carry(offset)]),
            ))?;
            let sum_e = Equality {
                proof: numeric(&sum_name, vec![x.clone(), y.clone(), incoming.clone()]),
                ..sum_e
            };
            let carry_e = Equality {
                proof: numeric(&carry_name, vec![x, y, incoming]),
                ..carry_e
            };
            let r = map_bit(sum_e.reverse()).then(value_definition(0, rs.clone()).reverse())?;
            p = p.then(r.on_left(mul(two(), bit_value(out))))?.then(
                mul_right(two(), map_bit(carry_e.reverse())).on_right(value(0, rs.clone())),
            )?;
        } else {
            let half = size / 2;
            let (xl, xh) = (project(xs.clone(), false), project(xs, true));
            let (yl, yh) = (project(ys.clone(), false), project(ys, true));
            let (rl, rh) = (project(rs.clone(), false), project(rs.clone(), true));
            let a = value(d - 1, xl);
            let b = value(d - 1, yl);
            let ah = value(d - 1, xh);
            let bh = value(d - 1, yh);
            let lo = value(d - 1, rl);
            let hi = value(d - 1, rh);
            let proof = numeric(
                "Combine",
                vec![
                    weight(d - 1),
                    a,
                    b,
                    ah,
                    bh,
                    lo.clone(),
                    hi.clone(),
                    ci,
                    bit_value(carry(offset + half)),
                    co.clone(),
                    numeric(
                        &format!("W{width}.Block{}At{offset}", d - 1),
                        vec![x.clone(), y.clone(), incoming.clone()],
                    ),
                    numeric(
                        &format!("W{width}.Block{}At{}", d - 1, offset + half),
                        vec![x, y, incoming],
                    ),
                ],
            );
            let end = add(
                add(lo, mul(weight(d - 1), hi)),
                mul(weight_body(d), co.clone()),
            );
            let start = p.right.clone();
            p = p.then(Equality::natural(start, end, proof))?;
            p = p.then(
                value_definition(d, rs.clone())
                    .reverse()
                    .on_left(mul(weight_body(d), co.clone())),
            )?;
        }
        p = p.then(mul_left(weight_definition(d).reverse(), co).on_right(value(d, rs)))?;
        self.theorem(&format!("W{width}.Block{d}At{offset}"), &args, p)
    }
    fn family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        let (x, y, incoming, initial) = (var("x"), var("y"), var("incoming"), var("initial"));
        let name = |suffix: &str| format!("W{width}.{suffix}");
        self.emit(
            &name("Unsigned"),
            &[("x", word_type(width)?)],
            nat(),
            value(d, x.clone()),
            false,
        )?;
        self.emit(&name("Modulus"), &[], nat(), weight(d), false)?;
        self.emit(
            &name("Sign"),
            &[("x", word_type(width)?)],
            boolean(),
            at(x.clone(), width, width - 1)?,
            false,
        )?;
        self.emit(
            &name("Signed"),
            &[("x", word_type(width)?)],
            pi("_part", boolean(), nat()),
            call(
                "Mpk.Go.Practical.Int.Make",
                vec![
                    numeric(&name("Unsigned"), vec![x.clone()]),
                    mul(
                        numeric(&name("Modulus"), vec![]),
                        bit_value(numeric(&name("Sign"), vec![x.clone()])),
                    ),
                ],
            ),
            false,
        )?;
        let ci = wc(
            width,
            &format!("Carry{}", width - 1),
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let co = wc(
            width,
            &format!("Carry{width}"),
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        self.emit(
            &name("NoOverflow"),
            &[
                ("x", word_type(width)?),
                ("y", word_type(width)?),
                ("incoming", boolean()),
            ],
            boolean(),
            bc("Equal", vec![ci, co]),
            false,
        )?;
        self.block(width, d, 0)?;
        let result = wc(
            width,
            "AddCarry",
            vec![x.clone(), y.clone(), incoming.clone()],
        );
        let left = add(
            add(value(d, x.clone()), value(d, y.clone())),
            bit_value(incoming.clone()),
        );
        let right = add(
            value(d, result),
            mul(
                weight(d),
                bit_value(wc(
                    width,
                    &format!("Carry{width}"),
                    vec![x.clone(), y.clone(), incoming.clone()],
                )),
            ),
        );
        let p = Equality::natural(
            left,
            right,
            numeric(&format!("W{width}.Block{d}At0"), vec![x, y, incoming]),
        )
        .on_right(initial);
        self.theorem(
            &name("AdditionConservation"),
            &[
                ("x", word_type(width)?),
                ("y", word_type(width)?),
                ("incoming", boolean()),
                ("initial", nat()),
            ],
            p,
        )?;
        self.signed_family(width)
    }
}

pub(super) fn append(builder: &mut Builder, recipes: &BTreeMap<String, Expr>) -> Result<()> {
    let mut numeric = Numeric {
        builder,
        recipes,
        serial: 0,
    };
    append_base(&mut numeric)
}

fn append_base(numeric: &mut Numeric<'_>) -> Result<()> {
    numeric.definitions()?;
    numeric.value_congruence()?;
    numeric.local_addition()?;
    numeric.combine()?;
    numeric.sign_addition()?;
    numeric.signed_combine()?;
    for width in WIDTHS {
        numeric.family(width)?;
    }
    Ok(())
}

pub(super) fn append_arithmetic(
    builder: &mut Builder,
    recipes: &BTreeMap<String, Expr>,
) -> Result<()> {
    let mut numeric = Numeric {
        builder,
        recipes,
        serial: 0,
    };
    append_base(&mut numeric)?;
    arithmetic::append(&mut numeric)
}

pub(super) fn append_order(builder: &mut Builder, recipes: &BTreeMap<String, Expr>) -> Result<()> {
    let mut numeric = Numeric {
        builder,
        recipes,
        serial: 0,
    };
    append_base(&mut numeric)?;
    order::append(numeric.builder, recipes)
}

pub(super) fn append_division(
    builder: &mut Builder,
    recipes: &BTreeMap<String, Expr>,
) -> Result<()> {
    let mut numeric = Numeric {
        builder,
        recipes,
        serial: 0,
    };
    append_base(&mut numeric)?;
    arithmetic::append_division_prerequisites(&mut numeric)?;
    order::append(numeric.builder, recipes)?;
    division::append(numeric.builder, recipes)
}

pub(super) fn append_signed_product(
    builder: &mut Builder,
    recipes: &BTreeMap<String, Expr>,
) -> Result<()> {
    let mut numeric = Numeric {
        builder,
        recipes,
        serial: 0,
    };
    append_base(&mut numeric)?;
    arithmetic::append(&mut numeric)?;
    signed_product::append(numeric.builder)
}
