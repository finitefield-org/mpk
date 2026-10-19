//! Opposite subtraction carries identify exact bitwise equality.
use super::*;

fn pair(width: usize, index: usize, x: Expr, y: Expr) -> Expr {
    let carry = |a: Expr, b: Expr| {
        if index == 0 {
            bit(true)
        } else {
            wc(
                width,
                &format!("Carry{index}"),
                vec![a, wc(width, "Not", vec![b]), bit(true)],
            )
        }
    };
    bc("And", vec![carry(x.clone(), y.clone()), carry(y, x)])
}
fn block(width: usize, d: usize, offset: usize, x: Expr, y: Expr) -> Expr {
    oc(&format!("W{width}.EqualBlock{d}At{offset}"), vec![x, y])
}
fn carry_step(width: usize, index: usize, x: Expr, y: Expr) -> Result<Equality> {
    let ny = wc(width, "Not", vec![y.clone()]);
    let old = if index == 0 {
        bit(true)
    } else {
        wc(
            width,
            &format!("Carry{index}"),
            vec![x.clone(), ny.clone(), bit(true)],
        )
    };
    let out = wc(
        width,
        &format!("Carry{}", index + 1),
        vec![x.clone(), ny.clone(), bit(true)],
    );
    let before = bc(
        "Carry",
        vec![
            at(x.clone(), width, index)?,
            at(ny.clone(), width, index)?,
            old.clone(),
        ],
    );
    let proof = numeric(
        &format!("W{width}.CarryStep{index}"),
        vec![x.clone(), ny.clone(), bit(true)],
    );
    let selected = bc("Not", vec![at(y, width, index)?]);
    let e = Equality::boolean(
        at(ny, width, index)?,
        selected.clone(),
        refl(boolean(), selected.clone()),
    );
    let end = bc(
        "Carry",
        vec![at(x.clone(), width, index)?, selected, old.clone()],
    );
    let mapped = e.mapped(
        lam(
            "_complement_bit",
            boolean(),
            bc(
                "Carry",
                vec![at(x, width, index)?, var("_complement_bit"), old],
            ),
        ),
        before.clone(),
        end,
        boolean(),
    );
    Equality::boolean(out, before, proof).then(mapped)
}

impl Order<'_> {
    pub(super) fn equality_laws(&mut self) -> Result<()> {
        for label in ["OppositeCarry", "AndJoin"] {
            let names: &[&'static str] = if label == "OppositeCarry" {
                &["a", "b", "c", "d"]
            } else {
                &["a", "b", "c"]
            };
            let endpoints = |e: &BTreeMap<&'static str, Expr>| {
                let (a, b, c) = (e["a"].clone(), e["b"].clone(), e["c"].clone());
                if label == "OppositeCarry" {
                    let d = e["d"].clone();
                    (
                        bc(
                            "And",
                            vec![
                                bc(
                                    "Carry",
                                    vec![a.clone(), bc("Not", vec![b.clone()]), c.clone()],
                                ),
                                bc(
                                    "Carry",
                                    vec![b.clone(), bc("Not", vec![a.clone()]), d.clone()],
                                ),
                            ],
                        ),
                        bc("And", vec![bc("Equal", vec![a, b]), bc("And", vec![c, d])]),
                    )
                } else {
                    (
                        bc(
                            "And",
                            vec![b.clone(), bc("And", vec![a.clone(), c.clone()])],
                        ),
                        bc("And", vec![bc("And", vec![a, b]), c]),
                    )
                }
            };
            let prop = |e: &BTreeMap<&'static str, Expr>| {
                let (a, b) = endpoints(e);
                eq(boolean(), a, b)
            };
            let mut evaluator = Evaluation::new(self.recipes);
            let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
                let (a, b) = endpoints(e);
                Ok(evaluator.equation(a, b)?.proof)
            };
            let proof = boolean_cases(names, &BTreeMap::new(), &prop, &mut leaf)?;
            let env = names.iter().map(|n| (*n, var(n))).collect();
            let mut ty = prop(&env);
            for name in names.iter().rev() {
                ty = pi(name, boolean(), ty);
            }
            self.emit(label, &[], ty, proof, true)?;
        }
        Ok(())
    }
    fn equal_block(&mut self, width: usize, d: usize, offset: usize) -> Result<()> {
        let size = 1 << d;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(width)?), ("y", word_type(width)?)];
        if d > 0 {
            self.equal_block(width, d - 1, offset)?;
            self.equal_block(width, d - 1, offset + size / 2)?;
        }
        let body = if d == 0 {
            bc(
                "Equal",
                vec![at(x.clone(), width, offset)?, at(y.clone(), width, offset)?],
            )
        } else {
            bc(
                "And",
                vec![
                    block(width, d - 1, offset, x.clone(), y.clone()),
                    block(width, d - 1, offset + size / 2, x.clone(), y.clone()),
                ],
            )
        };
        self.emit(
            &format!("W{width}.EqualBlock{d}At{offset}"),
            &args,
            boolean(),
            body.clone(),
            false,
        )?;
        let lhs = pair(width, offset + size, x.clone(), y.clone());
        let e = if d == 0 {
            let a = carry_step(width, offset, x.clone(), y.clone())?;
            let b = carry_step(width, offset, y.clone(), x.clone())?;
            let start =
                bool_left("And", a.clone(), b.left.clone()).then(bool_right("And", a.right, b))?;
            let c1 = if offset == 0 {
                bit(true)
            } else {
                wc(
                    width,
                    &format!("Carry{offset}"),
                    vec![x.clone(), wc(width, "Not", vec![y.clone()]), bit(true)],
                )
            };
            let c2 = if offset == 0 {
                bit(true)
            } else {
                wc(
                    width,
                    &format!("Carry{offset}"),
                    vec![y.clone(), wc(width, "Not", vec![x.clone()]), bit(true)],
                )
            };
            let end = bc("And", vec![body, pair(width, offset, x.clone(), y.clone())]);
            let first = start.right.clone();
            start.then(Equality::boolean(
                first,
                end,
                oc(
                    "OppositeCarry",
                    vec![
                        at(x.clone(), width, offset)?,
                        at(y.clone(), width, offset)?,
                        c1,
                        c2,
                    ],
                ),
            ))?
        } else {
            let h = d - 1;
            let mid = offset + size / 2;
            let lo = block(width, h, offset, x.clone(), y.clone());
            let hi = block(width, h, mid, x.clone(), y.clone());
            let incoming = pair(width, offset, x.clone(), y.clone());
            let high = Equality::boolean(
                lhs.clone(),
                bc(
                    "And",
                    vec![hi.clone(), pair(width, mid, x.clone(), y.clone())],
                ),
                oc(
                    &format!("W{width}.EqualCarryBlock{h}At{mid}"),
                    vec![x.clone(), y.clone()],
                ),
            );
            let low = Equality::boolean(
                pair(width, mid, x.clone(), y.clone()),
                bc("And", vec![lo.clone(), incoming.clone()]),
                oc(
                    &format!("W{width}.EqualCarryBlock{h}At{offset}"),
                    vec![x.clone(), y.clone()],
                ),
            );
            let e = high.then(bool_right("And", hi.clone(), low))?;
            let end = bc("And", vec![body, incoming.clone()]);
            let first = e.right.clone();
            e.then(Equality::boolean(
                first,
                end,
                oc("AndJoin", vec![lo, hi, incoming]),
            ))?
        };
        let right = bc(
            "And",
            vec![
                block(width, d, offset, x.clone(), y.clone()),
                pair(width, offset, x.clone(), y.clone()),
            ],
        );
        self.theorem(
            &format!("W{width}.EqualCarryBlock{d}At{offset}"),
            &args,
            Equality {
                left: lhs,
                right,
                ..e
            },
        )
    }
    pub(super) fn equality_family(&mut self, width: usize) -> Result<()> {
        let d = depth(width)?;
        self.equal_block(width, d, 0)?;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(width)?), ("y", word_type(width)?)];
        let root = block(width, d, 0, x.clone(), y.clone());
        let first = Equality::boolean(
            pair(width, width, x.clone(), y.clone()),
            bc(
                "And",
                vec![root.clone(), bc("And", vec![bit(true), bit(true)])],
            ),
            oc(
                &format!("W{width}.EqualCarryBlock{d}At0"),
                vec![x.clone(), y.clone()],
            ),
        );
        let e = first
            .then(bool_right(
                "And",
                root.clone(),
                Equality::boolean(
                    bc("And", vec![bit(true), bit(true)]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ),
            ))?
            .then(Equality::boolean(
                bc("And", vec![root.clone(), bit(true)]),
                root.clone(),
                refl(boolean(), root.clone()),
            ))?;
        let actual = wc(width, "Equal", vec![x.clone(), y.clone()]);
        let e = e.then(Equality::boolean(
            root,
            actual.clone(),
            refl(boolean(), actual.clone()),
        ))?;
        self.theorem(&format!("W{width}.CarryPairEquality"), &args, e.clone())?;
        let carry = |a: Expr, b: Expr| {
            wc(
                width,
                &format!("Carry{width}"),
                vec![a, wc(width, "Not", vec![b]), bit(true)],
            )
        };
        let reflected = |a: Expr, b: Expr| {
            let c = carry(a.clone(), b.clone());
            let word_less = wc(width, "UnsignedLess", vec![a.clone(), b.clone()]);
            let unfold = Equality::boolean(
                word_less.clone(),
                bc("Not", vec![c.clone()]),
                refl(boolean(), bc("Not", vec![c.clone()])),
            );
            let involution = bool_map("Not", unfold).then(Equality::boolean(
                bc("Not", vec![bc("Not", vec![c.clone()])]),
                c.clone(),
                bc("DoubleNot", vec![c]),
            ))?;
            involution.reverse().then(bool_map(
                "Not",
                Equality::boolean(
                    word_less,
                    less(value(d, a.clone()), value(d, b.clone())),
                    oc(&format!("W{width}.UnsignedLessNatural"), vec![a, b]),
                ),
            ))
        };
        let a = reflected(x.clone(), y.clone())?;
        let b = reflected(y.clone(), x.clone())?;
        let e = e
            .reverse()
            .then(bool_left("And", a.clone(), b.left.clone()))?
            .then(bool_right("And", a.right, b))?
            .then(Equality::boolean(
                bc(
                    "And",
                    vec![
                        bc("Not", vec![less(value(d, x.clone()), value(d, y.clone()))]),
                        bc("Not", vec![less(value(d, y.clone()), value(d, x.clone()))]),
                    ],
                ),
                equal(value(d, x.clone()), value(d, y.clone())),
                oc(
                    "NaturalTotalEquality",
                    vec![value(d, x.clone()), value(d, y.clone())],
                ),
            ))?;
        self.theorem(&format!("W{width}.EqualNatural"), &args, e.clone())?;
        let not = bool_map("Not", e);
        self.theorem(
            &format!("W{width}.NotEqualNatural"),
            &args,
            Equality {
                left: wc(width, "NotEqual", vec![x, y]),
                ..not
            },
        )
    }
}
