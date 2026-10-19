//! Generic Nat algebra for the actual signed-product correction.
use super::*;

pub(super) struct Scalars {
    pub x: Expr,
    pub y: Expr,
    pub r: Expr,
    pub sx: Expr,
    pub sy: Expr,
    pub sr: Expr,
    pub m: Expr,
    pub q: Expr,
}
impl Scalars {
    fn variables() -> Self {
        Self {
            x: var("ux"),
            y: var("uy"),
            r: var("ur"),
            sx: var("sx"),
            sy: var("sy"),
            sr: var("sr"),
            m: var("modulus"),
            q: var("discarded"),
        }
    }
    pub(super) fn word(w: usize, x: Expr, y: Expr) -> Result<Self> {
        let d = depth(w)?;
        let r = wc(w, "Mul", vec![x.clone(), y.clone()]);
        Ok(Self {
            x: value(d, x.clone()),
            y: value(d, y.clone()),
            r: value(d, r.clone()),
            sx: bit_value(at(x.clone(), w, w - 1)?),
            sy: bit_value(at(y.clone(), w, w - 1)?),
            sr: bit_value(at(r, w, w - 1)?),
            m: weight(d),
            q: numeric(&format!("W{w}.MulDiscard{w}"), vec![x, y]),
        })
    }
    pub(super) fn values(&self) -> Vec<Expr> {
        vec![
            self.x.clone(),
            self.y.clone(),
            self.r.clone(),
            self.sx.clone(),
            self.sy.clone(),
            self.sr.clone(),
            self.m.clone(),
            self.q.clone(),
        ]
    }
    pub(super) fn positive(&self) -> Expr {
        add(
            add(self.q.clone(), self.sr.clone()),
            mul(self.m.clone(), mul(self.sx.clone(), self.sy.clone())),
        )
    }
    pub(super) fn negative(&self) -> Expr {
        add(
            mul(self.x.clone(), self.sy.clone()),
            mul(self.y.clone(), self.sx.clone()),
        )
    }
    pub(super) fn product_positive(&self) -> Expr {
        add(
            mul(self.x.clone(), self.y.clone()),
            mul(
                mul(self.m.clone(), self.sx.clone()),
                mul(self.m.clone(), self.sy.clone()),
            ),
        )
    }
    pub(super) fn product_negative(&self) -> Expr {
        add(
            mul(self.x.clone(), mul(self.m.clone(), self.sy.clone())),
            mul(mul(self.m.clone(), self.sx.clone()), self.y.clone()),
        )
    }
    pub(super) fn left(&self) -> Expr {
        add(
            self.product_positive(),
            mul(self.m.clone(), self.sr.clone()),
        )
    }
    pub(super) fn right(&self) -> Expr {
        add(self.r.clone(), self.product_negative())
    }
    pub(super) fn residual_left(&self) -> Expr {
        add(
            self.product_positive(),
            add(
                mul(self.m.clone(), self.sr.clone()),
                add(
                    mul(self.m.clone(), self.negative()),
                    mul(zero(), self.positive()),
                ),
            ),
        )
    }
    pub(super) fn residual_right(&self) -> Expr {
        add(
            add(
                self.r.clone(),
                add(
                    mul(self.m.clone(), self.positive()),
                    mul(zero(), self.negative()),
                ),
            ),
            self.product_negative(),
        )
    }
    fn args(&self) -> Vec<Binder> {
        let mut args = ["ux", "uy", "ur", "sx", "sy", "sr", "modulus", "discarded"]
            .into_iter()
            .map(|n| (n, nat()))
            .collect::<Vec<_>>();
        args.push((
            "_unsigned",
            eq(
                nat(),
                mul(self.x.clone(), self.y.clone()),
                add(self.r.clone(), mul(self.m.clone(), self.q.clone())),
            ),
        ));
        args
    }
}

impl SignedProduct<'_> {
    pub(super) fn scalar_laws(&mut self) -> Result<()> {
        let s = Scalars::variables();
        let args = s.args();
        let cp = s.positive();
        let cn = s.negative();
        let l = s.left();
        let r = s.right();
        let left = add(l.clone(), mul(s.m.clone(), cn.clone()));
        let right = add(r.clone(), mul(s.m.clone(), cp.clone()));
        let pad = add(
            mul(
                mul(s.m.clone(), s.sx.clone()),
                mul(s.m.clone(), s.sy.clone()),
            ),
            add(mul(s.m.clone(), s.sr.clone()), mul(s.m.clone(), cn.clone())),
        );
        let e = self
            .polynomial(
                &args,
                left.clone(),
                add(mul(s.x.clone(), s.y.clone()), pad.clone()),
            )?
            .then(
                Equality::natural(
                    mul(s.x.clone(), s.y.clone()),
                    add(s.r.clone(), mul(s.m.clone(), s.q.clone())),
                    var("_unsigned"),
                )
                .on_left(pad),
            )?;
        let next = self.polynomial(&args, e.right.clone(), right.clone())?;
        self.theorem("CorrectionBalance", &args, e.then(next)?)?;
        let mut zero_args = args.clone();
        zero_args.push(("_correction", eq(nat(), cp.clone(), cn.clone())));
        let mut call_args = s.values();
        call_args.push(var("_unsigned"));
        let e = instance(
            "CorrectionBalance",
            call_args.clone(),
            left.clone(),
            right.clone(),
        )
        .then(
            mul_right(
                s.m.clone(),
                Equality::natural(cp.clone(), cn.clone(), var("_correction")),
            )
            .on_right(r.clone()),
        )?;
        self.theorem(
            "ExactCrossFromCorrection",
            &zero_args,
            Equality::natural(
                l.clone(),
                r.clone(),
                nc(
                    "AddCancel",
                    vec![l.clone(), r.clone(), mul(s.m.clone(), cn.clone()), e.proof],
                ),
            ),
        )?;
        let mut exact_args = args.clone();
        exact_args.extend([
            (
                "_positive",
                eq(boolean(), less(zero(), s.m.clone()), bit(true)),
            ),
            ("_exact", eq(nat(), l.clone(), r.clone())),
        ]);
        let e = Equality::natural(l.clone(), r.clone(), var("_exact"))
            .on_left(mul(s.m.clone(), cn.clone()))
            .reverse()
            .then(instance(
                "CorrectionBalance",
                call_args.clone(),
                left.clone(),
                right.clone(),
            ))?;
        let commute = Equality::natural(
            add(mul(s.m.clone(), cn.clone()), r.clone()),
            add(r.clone(), mul(s.m.clone(), cn.clone())),
            nc("AddComm", vec![mul(s.m.clone(), cn.clone()), r.clone()]),
        )
        .then(e)?
        .then(Equality::natural(
            add(r.clone(), mul(s.m.clone(), cp.clone())),
            add(mul(s.m.clone(), cp.clone()), r.clone()),
            nc("AddComm", vec![r.clone(), mul(s.m.clone(), cp.clone())]),
        ))?;
        let same = nc(
            "AddCancel",
            vec![
                mul(s.m.clone(), cn.clone()),
                mul(s.m.clone(), cp.clone()),
                r.clone(),
                commute.proof,
            ],
        );
        self.theorem(
            "CorrectionFromExactCross",
            &exact_args,
            instance(
                "MultiplyCancelPositive",
                vec![s.m.clone(), var("_positive"), cn.clone(), cp.clone(), same],
                cn,
                cp,
            )
            .reverse(),
        )?;
        let e = self
            .polynomial(&args, s.residual_left(), left.clone())?
            .then(instance("CorrectionBalance", call_args, left, right))?;
        let next = self.polynomial(&args, e.right.clone(), s.residual_right())?;
        self.theorem("ResidualCross", &args, e.then(next)?)?;
        self.reflection_agreement()
    }
    fn reflection_agreement(&mut self) -> Result<()> {
        let property = |env: &BTreeMap<&'static str, Expr>| {
            let (a, b) = (env["a"].clone(), env["b"].clone());
            pi(
                "_forward",
                pi(
                    "_h",
                    eq(boolean(), a.clone(), bit(true)),
                    eq(boolean(), b.clone(), bit(true)),
                ),
                pi(
                    "_backward",
                    pi(
                        "_h",
                        eq(boolean(), b.clone(), bit(true)),
                        eq(boolean(), a.clone(), bit(true)),
                    ),
                    eq(boolean(), a, b),
                ),
            )
        };
        let mut leaf = |env: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (a, b) = (env["a"] == bit(true), env["b"] == bit(true));
            let proof = if a == b {
                refl(boolean(), bit(a))
            } else {
                bc(
                    "FalseElim",
                    vec![
                        eq(boolean(), bit(a), bit(b)),
                        app(
                            var(if a { "_forward" } else { "_backward" }),
                            vec![refl(boolean(), bit(true))],
                        ),
                    ],
                )
            };
            Ok(lam(
                "_forward",
                pi(
                    "_h",
                    eq(boolean(), bit(a), bit(true)),
                    eq(boolean(), bit(b), bit(true)),
                ),
                lam(
                    "_backward",
                    pi(
                        "_h",
                        eq(boolean(), bit(b), bit(true)),
                        eq(boolean(), bit(a), bit(true)),
                    ),
                    proof,
                ),
            ))
        };
        let proof = boolean_cases(&["a", "b"], &BTreeMap::new(), &property, &mut leaf)?;
        self.emit(
            "ReflectionAgreement",
            &[],
            pi(
                "a",
                boolean(),
                pi(
                    "b",
                    boolean(),
                    property(&[("a", var("a")), ("b", var("b"))].into_iter().collect()),
                ),
            ),
            proof,
            true,
        )
    }
}
