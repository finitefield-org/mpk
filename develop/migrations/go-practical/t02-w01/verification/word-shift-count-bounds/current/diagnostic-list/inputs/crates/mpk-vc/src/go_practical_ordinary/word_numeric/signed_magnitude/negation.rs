//! Connect the actual complement-plus-one circuit, carry and source guards.
use super::*;

fn carry(w: usize, x: Expr) -> Expr {
    wc(
        w,
        &format!("Carry{w}"),
        vec![wc(w, "Not", vec![x]), wc(w, "Zero", vec![]), bit(true)],
    )
}
impl Magnitude<'_> {
    pub(super) fn negation(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let (nx, z) = (wc(w, "Not", vec![x.clone()]), wc(w, "Zero", vec![]));
        let actual = neg(w, x.clone());
        let source = wc(w, "AddCarry", vec![nx.clone(), z.clone(), bit(true)]);
        let definition = Equality {
            ty: word_type(w)?,
            left: actual.clone(),
            right: source.clone(),
            proof: refl(word_type(w)?, source.clone()),
        };
        self.theorem(
            &format!("W{w}.NegationDefinition"),
            &args,
            definition.clone(),
        )?;
        let (ux, un, m, c) = (
            value(d, x.clone()),
            value(d, actual.clone()),
            weight(d),
            carry(w, x.clone()),
        );
        let uz = Equality::natural(
            value(d, z.clone()),
            zero(),
            order(&format!("W{w}.ZeroValue"), vec![]),
        );
        let outgoing = mul(m.clone(), bit_value(c.clone()));
        let lhs = add(
            add(value(d, nx.clone()), value(d, z.clone())),
            bit_value(bit(true)),
        );
        let cleaned = uz
            .on_right(value(d, nx.clone()))
            .on_left(bit_value(bit(true)))
            .then(bit_literal(true).on_right(add(value(d, nx.clone()), zero())))?;
        let adder = cleaned
            .reverse()
            .then(Equality::natural(
                lhs,
                add(value(d, source), outgoing.clone()),
                numeric(&format!("W{w}.Block{d}At0"), vec![nx.clone(), z, bit(true)]),
            ))?
            .then(map_value(d, definition.reverse()).on_left(outgoing.clone()))?;
        let left = add(add(ux.clone(), un.clone()), outgoing.clone());
        let e = self
            .polynomial(
                &args,
                left.clone(),
                add(ux.clone(), add(un.clone(), outgoing)),
            )?
            .then(adder.reverse().on_right(ux.clone()))?;
        let next = self.polynomial(
            &args,
            e.right.clone(),
            add(add(ux.clone(), value(d, nx)), one()),
        )?;
        let e = e.then(next)?.then(Equality::natural(
            add(
                add(ux.clone(), value(d, wc(w, "Not", vec![x.clone()]))),
                one(),
            ),
            m.clone(),
            order(&format!("W{w}.Complement"), vec![x.clone()]),
        ))?;
        self.theorem(&format!("W{w}.NegationCarryBalance"), &args, e.clone())?;
        let iszero = wc(w, "IsZero", vec![x.clone()]);
        let carry_zero = Equality::boolean(
            c.clone(),
            equal(ux.clone(), zero()),
            mc(
                "SumCarryIsZero",
                vec![
                    ux.clone(),
                    un.clone(),
                    m.clone(),
                    order(&format!("W{w}.ValueBound"), vec![actual.clone()]),
                    c.clone(),
                    e.proof.clone(),
                ],
            ),
        )
        .then(
            Equality::boolean(
                iszero.clone(),
                equal(ux.clone(), zero()),
                order(&format!("W{w}.IsZeroNatural"), vec![x.clone()]),
            )
            .reverse(),
        )?;
        self.theorem(
            &format!("W{w}.NegationCarryIsZero"),
            &args,
            carry_zero.clone(),
        )?;
        let swapped = self
            .polynomial(
                &args,
                add(
                    add(un.clone(), ux.clone()),
                    mul(m.clone(), bit_value(c.clone())),
                ),
                left,
            )?
            .then(e.clone())?;
        let other = Equality::boolean(
            c.clone(),
            equal(un.clone(), zero()),
            mc(
                "SumCarryIsZero",
                vec![
                    un.clone(),
                    ux.clone(),
                    m.clone(),
                    order(&format!("W{w}.ValueBound"), vec![x.clone()]),
                    c.clone(),
                    swapped.proof,
                ],
            ),
        );
        let same_zero = Equality::boolean(
            wc(w, "IsZero", vec![actual.clone()]),
            equal(un.clone(), zero()),
            order(&format!("W{w}.IsZeroNatural"), vec![actual.clone()]),
        )
        .then(other.reverse())?
        .then(carry_zero.clone())?;
        self.theorem(&format!("W{w}.NegationIsZero"), &args, same_zero.clone())?;
        self.theorem(
            &format!("W{w}.NegationDivideDefined"),
            &args,
            Equality {
                left: wc(w, "DivideDefined", vec![actual]),
                right: wc(w, "DivideDefined", vec![x.clone()]),
                ..bool_map("Not", same_zero)
            },
        )?;
        let complement = instance(
            "SumCarryComplement",
            vec![ux.clone(), un.clone(), m.clone(), c.clone(), e.proof],
            add(ux.clone(), un.clone()),
            mul(m.clone(), bit_value(bc("Not", vec![c]))),
        );
        let nonzero = map_bit(Equality {
            right: wc(w, "DivideDefined", vec![x.clone()]),
            ..bool_map("Not", carry_zero)
        });
        self.theorem(
            &format!("W{w}.NegationValueSum"),
            &args,
            complement.then(mul_right(m.clone(), nonzero))?,
        )?;
        self.zero_sign(w)?;
        let mut negative = args.to_vec();
        negative.push(("negative", eq(boolean(), sign(w, x.clone())?, bit(true))));
        let nonzero = Equality::boolean(
            wc(w, "DivideDefined", vec![x.clone()]),
            bit(true),
            mc(
                "SignNonzero",
                vec![
                    sign(w, x.clone())?,
                    iszero,
                    lam(
                        "_zero",
                        eq(boolean(), wc(w, "IsZero", vec![x.clone()]), bit(true)),
                        mc(&format!("W{w}.ZeroSign"), vec![x.clone(), var("_zero")]),
                    ),
                    var("negative"),
                ],
            ),
        );
        self.theorem(
            &format!("W{w}.SignDivideDefined"),
            &negative,
            nonzero.clone(),
        )?;
        let balanced = instance(
            &format!("W{w}.NegationValueSum"),
            vec![x.clone()],
            add(ux, un),
            mul(
                m.clone(),
                bit_value(wc(w, "DivideDefined", vec![x.clone()])),
            ),
        )
        .then(mul_right(
            m.clone(),
            map_bit(nonzero).then(bit_literal(true))?,
        ))?;
        let next = self.polynomial(&negative, balanced.right.clone(), m)?;
        self.theorem(
            &format!("W{w}.NegativeNegationBalance"),
            &negative,
            balanced.then(next)?,
        )
    }
    fn zero_sign(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let z = wc(w, "Zero", vec![]);
        let args = [
            ("x", word_type(w)?),
            (
                "zero",
                eq(boolean(), wc(w, "IsZero", vec![x.clone()]), bit(true)),
            ),
        ];
        let block = |level: usize, offset: usize| {
            order(
                &format!("W{w}.EqualBlock{level}At{offset}"),
                vec![x.clone(), z.clone()],
            )
        };
        let root = Equality::boolean(
            block(d, 0),
            wc(w, "IsZero", vec![x.clone()]),
            order(
                &format!("W{w}.EqualBitsBlock{d}At0"),
                vec![x.clone(), z.clone()],
            ),
        )
        .then(Equality::boolean(
            wc(w, "IsZero", vec![x.clone()]),
            bit(true),
            var("zero"),
        ))?;
        self.theorem(&format!("W{w}.ZeroTopBlock{d}"), &args, root)?;
        let mut offset = 0;
        for level in (0..d).rev() {
            let right = offset + (1 << level);
            let prior = mc(
                &format!("W{w}.ZeroTopBlock{}", level + 1),
                vec![x.clone(), var("zero")],
            );
            let e = Equality::boolean(
                block(level, right),
                bit(true),
                bc(
                    "AndRight",
                    vec![block(level, offset), block(level, right), prior],
                ),
            );
            self.theorem(&format!("W{w}.ZeroTopBlock{level}"), &args, e)?;
            offset = right;
        }
        let top = sign(w, x.clone())?;
        let topz = sign(w, z.clone())?;
        let zero_bit = order(&format!("W{w}.ZeroPointwise"), vec![bit(true); d]);
        let zero_sign = Equality::boolean(
            top.clone(),
            topz.clone(),
            bc(
                "EqualReflect",
                vec![
                    top,
                    topz.clone(),
                    mc(&format!("W{w}.ZeroTopBlock0"), vec![x, var("zero")]),
                ],
            ),
        )
        .then(Equality::boolean(topz, bit(false), zero_bit))?;
        self.theorem(&format!("W{w}.ZeroSign"), &args, zero_sign)
    }
}
