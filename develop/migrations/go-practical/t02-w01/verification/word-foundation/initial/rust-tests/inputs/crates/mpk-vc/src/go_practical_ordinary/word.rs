//! Full bit-vector values and ordinary circuits; no machine arithmetic rule.
use super::syntax::*;
use super::{word_proofs, Builder, GoOrdinaryError, Result};
use std::collections::BTreeMap;

pub(super) const WIDTHS: [usize; 4] = [8, 16, 32, 64];
pub(super) const ADDRESSES: [&str; 6] = [
    "_address0",
    "_address1",
    "_address2",
    "_address3",
    "_address4",
    "_address5",
];
pub(super) fn depth(width: usize) -> Result<usize> {
    if !WIDTHS.contains(&width) {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(width.trailing_zeros() as usize)
}
pub(super) fn word_type(width: usize) -> Result<Expr> {
    let mut ty = boolean();
    for name in ADDRESSES[..depth(width)?].iter().rev() {
        ty = pi(name, boolean(), ty);
    }
    Ok(ty)
}
pub(super) fn address(width: usize, index: usize) -> Result<Vec<Expr>> {
    if index >= width {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok((0..depth(width)?)
        .rev()
        .map(|i| bit(index & (1 << i) != 0))
        .collect())
}
pub(super) fn at(value: Expr, width: usize, index: usize) -> Result<Expr> {
    Ok(app(value, address(width, index)?))
}
pub(super) fn wc(width: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.W{width}.{name}"), args)
}
pub(super) fn choose(condition: Expr, yes: Expr, no: Expr, ty: Expr) -> Expr {
    call(
        "Std.Bool.cases",
        vec![lam("_case", boolean(), ty), no, yes, condition],
    )
}
fn vector(width: usize, bits: Vec<Expr>) -> Result<Expr> {
    if bits.len() != width {
        return Err(GoOrdinaryError::Linkage);
    }
    fn tree(bits: &[Expr], d: usize) -> Expr {
        if bits.len() == 1 {
            return bits[0].clone();
        }
        let middle = bits.len() / 2;
        choose(
            var(ADDRESSES[d]),
            tree(&bits[middle..], d + 1),
            tree(&bits[..middle], d + 1),
            boolean(),
        )
    }
    let mut body = tree(&bits, 0);
    for name in ADDRESSES[..depth(width)?].iter().rev() {
        body = lam(name, boolean(), body);
    }
    Ok(body)
}
fn literal(width: usize, value: u64) -> Result<Expr> {
    vector(
        width,
        (0..width).map(|i| bit(value & (1u64 << i) != 0)).collect(),
    )
}
fn fold_bits(name: &str, values: &[Expr], empty: bool) -> Expr {
    match values {
        [] => bit(empty),
        [a] => a.clone(),
        _ => {
            let m = values.len() / 2;
            bc(
                name,
                vec![
                    fold_bits(name, &values[..m], empty),
                    fold_bits(name, &values[m..], empty),
                ],
            )
        }
    }
}

pub(super) struct Words<'a> {
    pub builder: &'a mut Builder,
    pub definitions: BTreeMap<String, Expr>,
}
impl Words<'_> {
    pub fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        value: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(self.builder, name, args, ty, value.clone(), theorem)?;
        if !theorem {
            let mut body = value;
            for (name, ty) in args.iter().rev() {
                body = lam(name, ty.clone(), body);
            }
            self.definitions.insert(name.into(), body);
        }
        Ok(())
    }
    pub fn local(
        &mut self,
        width: usize,
        name: &str,
        args: &[Binder],
        ty: Expr,
        value: Expr,
        theorem: bool,
    ) -> Result<()> {
        self.emit(
            &format!("Mpk.Go.Practical.Word.W{width}.{name}"),
            args,
            ty,
            value,
            theorem,
        )
    }
    fn boolean_recipes(&mut self) -> Result<()> {
        // Proof-search recipes for the actual preceding ordinary definitions.
        // They supply no authority: every computed equality checks against the
        // original named definition in both cores.
        let (a, b) = (var("_core_a"), var("_core_b"));
        for (name, arity, value) in [
            (
                "Not",
                1,
                call("Std.Bool.rec", vec![bit(true), bit(false), a.clone()]),
            ),
            (
                "And",
                2,
                call("Std.Bool.rec", vec![bit(false), a.clone(), b.clone()]),
            ),
            (
                "Or",
                2,
                call("Std.Bool.rec", vec![a.clone(), bit(true), b.clone()]),
            ),
            (
                "Xor",
                2,
                call(
                    "Std.Bool.rec",
                    vec![a.clone(), bc("Not", vec![a.clone()]), b.clone()],
                ),
            ),
            (
                "Equal",
                2,
                call(
                    "Std.Bool.rec",
                    vec![bc("Not", vec![a.clone()]), a.clone(), b.clone()],
                ),
            ),
        ] {
            let mut body = value;
            if arity == 2 {
                body = lam("_core_b", boolean(), body);
            }
            body = lam("_core_a", boolean(), body);
            self.definitions
                .insert(format!("Mpk.Go.Practical.Bool.{name}"), body);
        }
        let (a, b, c) = (var("a"), var("b"), var("c"));
        let args = [("a", boolean()), ("b", boolean()), ("c", boolean())];
        self.emit(
            "Mpk.Go.Practical.Bool.Sum",
            &args,
            boolean(),
            bc(
                "Xor",
                vec![bc("Xor", vec![a.clone(), b.clone()]), c.clone()],
            ),
            false,
        )?;
        self.emit(
            "Mpk.Go.Practical.Bool.Carry",
            &args,
            boolean(),
            bc(
                "Or",
                vec![
                    bc("And", vec![a.clone(), b.clone()]),
                    bc("And", vec![bc("Xor", vec![a, b]), c]),
                ],
            ),
            false,
        )
    }
    fn family(&mut self, width: usize) -> Result<()> {
        let ty = word_type(width)?;
        let count_type = word_type(64)?;
        let d = depth(width)?;
        let (x, y, c) = (var("x"), var("y"), var("carry"));
        let xy = [("x", ty.clone()), ("y", ty.clone())];
        let args = [("x", ty.clone()), ("y", ty.clone()), ("carry", boolean())];
        self.local(width, "Carrier", &[], Expr::Sort0, ty.clone(), false)?;
        for (name, value) in [("Zero", 0), ("One", 1), ("Ones", u64::MAX)] {
            self.local(width, name, &[], ty.clone(), literal(width, value)?, false)?;
        }
        let indices: Vec<Binder> = ADDRESSES[..d].iter().map(|&n| (n, boolean())).collect();
        let av: Vec<_> = indices.iter().map(|(n, _)| var(n)).collect();
        let point = |op: &str, unary: bool| {
            let a = app(x.clone(), av.clone());
            let mut value = if unary {
                bc(op, vec![a])
            } else {
                bc(op, vec![a, app(y.clone(), av.clone())])
            };
            for (name, ty) in indices.iter().rev() {
                value = lam(name, ty.clone(), value);
            }
            value
        };
        self.local(
            width,
            "Not",
            &xy[..1],
            ty.clone(),
            point("Not", true),
            false,
        )?;
        for name in ["And", "Or", "Xor"] {
            self.local(width, name, &xy, ty.clone(), point(name, false), false)?;
        }
        self.local(
            width,
            "AndNot",
            &xy,
            ty.clone(),
            wc(
                width,
                "And",
                vec![x.clone(), wc(width, "Not", vec![y.clone()])],
            ),
            false,
        )?;
        self.local(
            width,
            "Select",
            &[
                ("condition", boolean()),
                ("x", ty.clone()),
                ("y", ty.clone()),
            ],
            ty.clone(),
            choose(var("condition"), x.clone(), y.clone(), ty.clone()),
            false,
        )?;
        self.local(width, "Carry0", &args, boolean(), c.clone(), false)?;
        for i in 0..width {
            let value = bc(
                "Carry",
                vec![
                    at(x.clone(), width, i)?,
                    at(y.clone(), width, i)?,
                    wc(
                        width,
                        &format!("Carry{i}"),
                        vec![x.clone(), y.clone(), c.clone()],
                    ),
                ],
            );
            self.local(
                width,
                &format!("Carry{}", i + 1),
                &args,
                boolean(),
                value,
                false,
            )?;
        }
        let bits = (0..width)
            .map(|i| {
                Ok(bc(
                    "Sum",
                    vec![
                        at(x.clone(), width, i)?,
                        at(y.clone(), width, i)?,
                        wc(
                            width,
                            &format!("Carry{i}"),
                            vec![x.clone(), y.clone(), c.clone()],
                        ),
                    ],
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        self.local(
            width,
            "AddCarry",
            &args,
            ty.clone(),
            vector(width, bits)?,
            false,
        )?;
        self.local(
            width,
            "Add",
            &xy,
            ty.clone(),
            wc(width, "AddCarry", vec![x.clone(), y.clone(), bit(false)]),
            false,
        )?;
        self.local(
            width,
            "Neg",
            &xy[..1],
            ty.clone(),
            wc(
                width,
                "AddCarry",
                vec![
                    wc(width, "Not", vec![x.clone()]),
                    wc(width, "Zero", vec![]),
                    bit(true),
                ],
            ),
            false,
        )?;
        self.local(
            width,
            "Sub",
            &xy,
            ty.clone(),
            wc(
                width,
                "AddCarry",
                vec![x.clone(), wc(width, "Not", vec![y.clone()]), bit(true)],
            ),
            false,
        )?;
        let equal = (0..width)
            .map(|i| {
                Ok(bc(
                    "Equal",
                    vec![at(x.clone(), width, i)?, at(y.clone(), width, i)?],
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        self.local(
            width,
            "Equal",
            &xy,
            boolean(),
            fold_bits("And", &equal, true),
            false,
        )?;
        self.local(
            width,
            "NotEqual",
            &xy,
            boolean(),
            bc("Not", vec![wc(width, "Equal", vec![x.clone(), y.clone()])]),
            false,
        )?;
        self.local(
            width,
            "IsZero",
            &xy[..1],
            boolean(),
            wc(width, "Equal", vec![x.clone(), wc(width, "Zero", vec![])]),
            false,
        )?;
        self.local(
            width,
            "UnsignedLess",
            &xy,
            boolean(),
            bc(
                "Not",
                vec![wc(
                    width,
                    &format!("Carry{width}"),
                    vec![x.clone(), wc(width, "Not", vec![y.clone()]), bit(true)],
                )],
            ),
            false,
        )?;
        self.local(
            width,
            "UnsignedLessEqual",
            &xy,
            boolean(),
            bc(
                "Not",
                vec![wc(width, "UnsignedLess", vec![y.clone(), x.clone()])],
            ),
            false,
        )?;
        self.local(
            width,
            "SignedLess",
            &xy,
            boolean(),
            choose(
                bc(
                    "Xor",
                    vec![
                        at(x.clone(), width, width - 1)?,
                        at(y.clone(), width, width - 1)?,
                    ],
                ),
                at(x.clone(), width, width - 1)?,
                wc(width, "UnsignedLess", vec![x.clone(), y.clone()]),
                boolean(),
            ),
            false,
        )?;
        self.local(
            width,
            "SignedLessEqual",
            &xy,
            boolean(),
            bc(
                "Not",
                vec![wc(width, "SignedLess", vec![y.clone(), x.clone()])],
            ),
            false,
        )?;
        for n in 0..width {
            let bits = (0..width)
                .map(|i| {
                    if i < n {
                        Ok(bit(false))
                    } else {
                        at(x.clone(), width, i - n)
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            self.local(
                width,
                &format!("Left{n}"),
                &xy[..1],
                ty.clone(),
                vector(width, bits)?,
                false,
            )?;
        }
        for n in (0..d).map(|i| 1 << i) {
            let bits = (0..width)
                .map(|i| {
                    if i + n < width {
                        at(x.clone(), width, i + n)
                    } else {
                        Ok(bit(false))
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            self.local(
                width,
                &format!("Right{n}"),
                &xy[..1],
                ty.clone(),
                vector(width, bits)?,
                false,
            )?;
            let bits = (0..width)
                .map(|i| at(x.clone(), width, (i + n).min(width - 1)))
                .collect::<Result<Vec<_>>>()?;
            self.local(
                width,
                &format!("SignedRight{n}"),
                &xy[..1],
                ty.clone(),
                vector(width, bits)?,
                false,
            )?;
        }
        let count = var("count");
        let shift_args = [("x", ty.clone()), ("count", count_type.clone())];
        let high = (d..64)
            .map(|i| at(count.clone(), 64, i))
            .collect::<Result<Vec<_>>>()?;
        let high = fold_bits("Or", &high, false);
        for (name, shift) in [
            ("ShiftLeft", "Left"),
            ("ShiftRight", "Right"),
            ("SignedShiftRight", "SignedRight"),
        ] {
            self.local(
                width,
                &format!("{name}0"),
                &shift_args,
                ty.clone(),
                x.clone(),
                false,
            )?;
            for i in 0..d {
                let previous = wc(width, &format!("{name}{i}"), vec![x.clone(), count.clone()]);
                let value = wc(
                    width,
                    "Select",
                    vec![
                        at(count.clone(), 64, i)?,
                        wc(width, &format!("{shift}{}", 1 << i), vec![previous.clone()]),
                        previous,
                    ],
                );
                self.local(
                    width,
                    &format!("{name}{}", i + 1),
                    &shift_args,
                    ty.clone(),
                    value,
                    false,
                )?;
            }
            let fill = if name == "SignedShiftRight" {
                wc(
                    width,
                    "Select",
                    vec![
                        at(x.clone(), width, width - 1)?,
                        wc(width, "Ones", vec![]),
                        wc(width, "Zero", vec![]),
                    ],
                )
            } else {
                wc(width, "Zero", vec![])
            };
            self.local(
                width,
                name,
                &shift_args,
                ty.clone(),
                wc(
                    width,
                    "Select",
                    vec![
                        high.clone(),
                        fill,
                        wc(width, &format!("{name}{d}"), vec![x.clone(), count.clone()]),
                    ],
                ),
                false,
            )?;
        }
        self.local(
            width,
            "SignedCountDefined",
            &[("count", count_type)],
            boolean(),
            bc("Not", vec![at(count, 64, 63)?]),
            false,
        )?;
        self.local(
            width,
            "Mul0",
            &xy,
            ty.clone(),
            wc(width, "Zero", vec![]),
            false,
        )?;
        for i in 0..width {
            self.local(
                width,
                &format!("Mul{}", i + 1),
                &xy,
                ty.clone(),
                wc(
                    width,
                    "Add",
                    vec![
                        wc(width, &format!("Mul{i}"), vec![x.clone(), y.clone()]),
                        wc(
                            width,
                            "Select",
                            vec![
                                at(y.clone(), width, i)?,
                                wc(width, &format!("Left{i}"), vec![x.clone()]),
                                wc(width, "Zero", vec![]),
                            ],
                        ),
                    ],
                ),
                false,
            )?;
        }
        self.local(
            width,
            "Mul",
            &xy,
            ty.clone(),
            wc(width, &format!("Mul{width}"), vec![x.clone(), y.clone()]),
            false,
        )?;
        let bits = (0..width)
            .map(|i| {
                if i == 0 {
                    Ok(var("low"))
                } else {
                    at(x.clone(), width, i - 1)
                }
            })
            .collect::<Result<Vec<_>>>()?;
        self.local(
            width,
            "PushLow",
            &[("x", ty.clone()), ("low", boolean())],
            ty.clone(),
            vector(width, bits)?,
            false,
        )?;
        self.local(
            width,
            "Remainder0",
            &xy,
            ty.clone(),
            wc(width, "Zero", vec![]),
            false,
        )?;
        let mut quotient = vec![];
        for i in 0..width {
            let previous = wc(width, &format!("Remainder{i}"), vec![x.clone(), y.clone()]);
            self.local(
                width,
                &format!("DividendStep{i}"),
                &xy,
                ty.clone(),
                wc(
                    width,
                    "PushLow",
                    vec![previous.clone(), at(x.clone(), width, width - 1 - i)?],
                ),
                false,
            )?;
            let current = wc(
                width,
                &format!("DividendStep{i}"),
                vec![x.clone(), y.clone()],
            );
            let q = bc(
                "Or",
                vec![
                    at(previous, width, width - 1)?,
                    bc(
                        "Not",
                        vec![wc(width, "UnsignedLess", vec![current.clone(), y.clone()])],
                    ),
                ],
            );
            self.local(width, &format!("QuotientBit{i}"), &xy, boolean(), q, false)?;
            let q = wc(
                width,
                &format!("QuotientBit{i}"),
                vec![x.clone(), y.clone()],
            );
            quotient.push(q.clone());
            self.local(
                width,
                &format!("Remainder{}", i + 1),
                &xy,
                ty.clone(),
                wc(
                    width,
                    "Select",
                    vec![
                        q,
                        wc(width, "Sub", vec![current.clone(), y.clone()]),
                        current,
                    ],
                ),
                false,
            )?;
        }
        self.local(
            width,
            "DivideDefined",
            &[("y", ty.clone())],
            boolean(),
            bc("Not", vec![wc(width, "IsZero", vec![y.clone()])]),
            false,
        )?;
        quotient.reverse();
        self.local(
            width,
            "UnsignedDiv",
            &xy,
            ty.clone(),
            wc(
                width,
                "Select",
                vec![
                    wc(width, "IsZero", vec![y.clone()]),
                    wc(width, "Zero", vec![]),
                    vector(width, quotient)?,
                ],
            ),
            false,
        )?;
        self.local(
            width,
            "UnsignedRem",
            &xy,
            ty.clone(),
            wc(
                width,
                "Select",
                vec![
                    wc(width, "IsZero", vec![y.clone()]),
                    wc(width, "Zero", vec![]),
                    wc(
                        width,
                        &format!("Remainder{width}"),
                        vec![x.clone(), y.clone()],
                    ),
                ],
            ),
            false,
        )?;
        self.local(
            width,
            "AbsBits",
            &xy[..1],
            ty.clone(),
            wc(
                width,
                "Select",
                vec![
                    at(x.clone(), width, width - 1)?,
                    wc(width, "Neg", vec![x.clone()]),
                    x.clone(),
                ],
            ),
            false,
        )?;
        let magnitude = |name| {
            wc(
                width,
                name,
                vec![
                    wc(width, "AbsBits", vec![x.clone()]),
                    wc(width, "AbsBits", vec![y.clone()]),
                ],
            )
        };
        let uq = magnitude("UnsignedDiv");
        let ur = magnitude("UnsignedRem");
        self.local(
            width,
            "SignedDiv",
            &xy,
            ty.clone(),
            wc(
                width,
                "Select",
                vec![
                    bc(
                        "Xor",
                        vec![
                            at(x.clone(), width, width - 1)?,
                            at(y.clone(), width, width - 1)?,
                        ],
                    ),
                    wc(width, "Neg", vec![uq.clone()]),
                    uq,
                ],
            ),
            false,
        )?;
        self.local(
            width,
            "SignedRem",
            &xy,
            ty.clone(),
            wc(
                width,
                "Select",
                vec![
                    at(x.clone(), width, width - 1)?,
                    wc(width, "Neg", vec![ur.clone()]),
                    ur,
                ],
            ),
            false,
        )?;
        for target in WIDTHS {
            for signed in [false, true] {
                let bits = (0..target)
                    .map(|i| {
                        if i < width {
                            at(x.clone(), width, i)
                        } else if signed {
                            at(x.clone(), width, width - 1)
                        } else {
                            Ok(bit(false))
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.local(
                    width,
                    &format!("{}To{target}", if signed { "Signed" } else { "Unsigned" }),
                    &xy[..1],
                    word_type(target)?,
                    vector(target, bits)?,
                    false,
                )?;
            }
        }
        Ok(())
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut words = Words {
        builder,
        definitions: BTreeMap::new(),
    };
    words.boolean_recipes()?;
    word_proofs::boolean_laws(&mut words)?;
    for width in WIDTHS {
        words.family(width)?;
        word_proofs::word_laws(&mut words, width)?;
    }
    Ok(())
}
