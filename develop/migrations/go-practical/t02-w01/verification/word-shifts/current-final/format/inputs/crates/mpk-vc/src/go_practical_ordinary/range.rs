//! Ordinary logical folds over [lower, upper). Bodies are total mathematical
//! functions; source evaluation/definedness is deliberately not inferred here.
use super::syntax::*;
use super::{Builder, Result};

fn rc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Range.{name}"), args)
}
fn ic(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn integer() -> Expr {
    pi("_part", boolean(), nat())
}

#[derive(Clone, Copy)]
enum Kind {
    Forall,
    Exists,
    NatSum,
    IntSum,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Forall => "Forall",
            Self::Exists => "Exists",
            Self::NatSum => "NatSum",
            Self::IntSum => "IntSum",
        }
    }
    fn ty(self) -> Expr {
        match self {
            Self::Forall | Self::Exists => boolean(),
            Self::NatSum => nat(),
            Self::IntSum => integer(),
        }
    }
    fn initial(self) -> Expr {
        match self {
            Self::Forall => bit(true),
            Self::Exists => bit(false),
            Self::NatSum => zero(),
            Self::IntSum => ic("Zero", vec![]),
        }
    }
    fn combine(self, a: Expr, b: Expr) -> Expr {
        match self {
            Self::Forall => bc("And", vec![a, b]),
            Self::Exists => bc("Or", vec![a, b]),
            Self::NatSum => add(a, b),
            Self::IntSum => ic("Add", vec![a, b]),
        }
    }
    fn relation(self, a: Expr, b: Expr) -> Expr {
        if matches!(self, Self::IntSum) {
            ic("Equivalent", vec![a, b])
        } else {
            eq(self.ty(), a, b)
        }
    }
    fn compare(self, a: Expr, b: Expr) -> Expr {
        match self {
            Self::Forall | Self::Exists => bc("Equal", vec![a, b]),
            Self::NatSum => equal(a, b),
            Self::IntSum => ic("Equal", vec![a, b]),
        }
    }
    fn reflect(self, a: Expr, b: Expr, proof: Expr) -> Expr {
        match self {
            Self::Forall | Self::Exists => bc("EqualReflect", vec![a, b, proof]),
            Self::NatSum => nc("EqualReflect", vec![a, b, proof]),
            Self::IntSum => ic("EqualReflect", vec![a, b, proof]),
        }
    }
    fn fold(self, lower: Expr, count: Expr, body: Expr) -> Expr {
        rc(&format!("Fold{}", self.name()), vec![lower, count, body])
    }
    fn range(self, lower: Expr, upper: Expr, body: Expr) -> Expr {
        rc(self.name(), vec![lower, upper, body])
    }
    fn base_proof(self) -> Expr {
        if matches!(self, Self::IntSum) {
            ic("Reflexive", vec![self.initial()])
        } else {
            refl(self.ty(), self.initial())
        }
    }
    fn congruence(
        self,
        old_left: Expr,
        old_right: Expr,
        value_left: Expr,
        value_right: Expr,
        previous: Expr,
        current: Expr,
    ) -> Result<Expr> {
        if matches!(self, Self::IntSum) {
            return Ok(ic(
                "AddRespects",
                vec![
                    old_left,
                    value_left,
                    old_right,
                    value_right,
                    previous,
                    current,
                ],
            ));
        }
        let p = Equality {
            ty: self.ty(),
            left: old_left.clone(),
            right: old_right.clone(),
            proof: previous,
        };
        let q = Equality {
            ty: self.ty(),
            left: value_left.clone(),
            right: value_right.clone(),
            proof: current,
        };
        let p = p.mapped(
            lam(
                "_left",
                self.ty(),
                self.combine(var("_left"), value_left.clone()),
            ),
            self.combine(old_left, value_left.clone()),
            self.combine(old_right.clone(), value_left.clone()),
            self.ty(),
        );
        let q = q.mapped(
            lam(
                "_right",
                self.ty(),
                self.combine(old_right.clone(), var("_right")),
            ),
            self.combine(old_right.clone(), value_left),
            self.combine(old_right, value_right),
            self.ty(),
        );
        Ok(p.then(q)?.proof)
    }
}

struct Ranges<'a>(&'a mut Builder);

impl Ranges<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.Range.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn boolean_observations(&mut self) -> Result<()> {
        for name in ["AndLeft", "AndRight", "EqualReflect"] {
            let proposition = |a: Expr, b: Expr| {
                let operation = if name == "EqualReflect" {
                    "Equal"
                } else {
                    "And"
                };
                let result = match name {
                    "AndLeft" => eq(boolean(), a.clone(), bit(true)),
                    "AndRight" => eq(boolean(), b.clone(), bit(true)),
                    _ => eq(boolean(), a.clone(), b.clone()),
                };
                pi(
                    "_h",
                    eq(boolean(), bc(operation, vec![a, b]), bit(true)),
                    result,
                )
            };
            let leaf = |a: bool, b: bool| {
                let premise = eq(
                    boolean(),
                    bc(
                        if name == "EqualReflect" {
                            "Equal"
                        } else {
                            "And"
                        },
                        vec![bit(a), bit(b)],
                    ),
                    bit(true),
                );
                let proof = match name {
                    "AndLeft" if a => refl(boolean(), bit(true)),
                    "AndRight" if b => refl(boolean(), bit(true)),
                    "AndLeft" | "AndRight" => var("_h"),
                    _ if a == b => refl(boolean(), bit(a)),
                    _ if !a => var("_h"),
                    _ => call(
                        "Std.Eq.symm",
                        vec![boolean(), bit(false), bit(true), var("_h")],
                    ),
                };
                lam("_h", premise, proof)
            };
            let branch = |a: bool| {
                let motive = lam("b", boolean(), proposition(bit(a), var("b")));
                lam(
                    "b",
                    boolean(),
                    call(
                        "Std.Bool.cases",
                        vec![motive, leaf(a, false), leaf(a, true), var("b")],
                    ),
                )
            };
            let ty = pi(
                "a",
                boolean(),
                pi("b", boolean(), proposition(var("a"), var("b"))),
            );
            let motive = lam(
                "a",
                boolean(),
                pi("b", boolean(), proposition(var("a"), var("b"))),
            );
            let proof = lam(
                "a",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![motive, branch(false), branch(true), var("a")],
                ),
            );
            emit(
                self.0,
                &format!("Mpk.Go.Practical.Bool.{name}"),
                &[],
                ty,
                proof,
                true,
            )?;
        }
        Ok(())
    }
    fn family(&mut self, kind: Kind) -> Result<()> {
        let name = kind.name();
        let ty = kind.ty();
        let initial = kind.initial();
        let (lower, upper, count, body, other, index) = (
            var("lower"),
            var("upper"),
            var("count"),
            var("body"),
            var("other"),
            var("_index"),
        );
        let function = pi("_argument", nat(), ty.clone());
        let args = [
            ("lower", nat()),
            ("count", nat()),
            ("body", function.clone()),
        ];
        let step = lam(
            "_index",
            nat(),
            lam(
                "_acc",
                ty.clone(),
                kind.combine(
                    var("_acc"),
                    app(body.clone(), vec![add(lower.clone(), index.clone())]),
                ),
            ),
        );
        self.emit(
            &format!("Fold{name}"),
            &args,
            ty.clone(),
            induct("_index", ty.clone(), initial.clone(), step, count.clone()),
            false,
        )?;
        self.emit(
            name,
            &[
                ("lower", nat()),
                ("upper", nat()),
                ("body", function.clone()),
            ],
            ty.clone(),
            kind.fold(
                lower.clone(),
                sub(upper.clone(), lower.clone()),
                body.clone(),
            ),
            false,
        )?;
        let zero_fold = kind.fold(lower.clone(), zero(), body.clone());
        self.emit(
            &format!("{name}Zero"),
            &[("lower", nat()), ("body", function.clone())],
            eq(ty.clone(), zero_fold.clone(), initial.clone()),
            refl(ty.clone(), initial.clone()),
            true,
        )?;
        let folded = kind.fold(lower.clone(), count.clone(), body.clone());
        let next = kind.fold(lower.clone(), succ(count.clone()), body.clone());
        let value = kind.combine(
            folded.clone(),
            app(body.clone(), vec![add(lower.clone(), count.clone())]),
        );
        self.emit(
            &format!("{name}Step"),
            &args,
            eq(ty.clone(), next, value.clone()),
            refl(ty.clone(), value),
            true,
        )?;
        let collapse = |length: Expr, proof: Expr| {
            let p = Equality::natural(length.clone(), zero(), proof).mapped(
                lam(
                    "_count",
                    nat(),
                    kind.fold(lower.clone(), var("_count"), body.clone()),
                ),
                kind.fold(lower.clone(), length, body.clone()),
                zero_fold.clone(),
                ty.clone(),
            );
            let q = Equality {
                ty: ty.clone(),
                left: zero_fold.clone(),
                right: initial.clone(),
                proof: rc(&format!("{name}Zero"), vec![lower.clone(), body.clone()]),
            };
            p.then(q)
        };
        let same = collapse(
            sub(lower.clone(), lower.clone()),
            nc("SubSame", vec![lower.clone()]),
        )?;
        self.emit(
            &format!("{name}SameBounds"),
            &[("lower", nat()), ("body", function.clone())],
            eq(
                ty.clone(),
                kind.range(lower.clone(), lower.clone(), body.clone()),
                initial.clone(),
            ),
            same.proof,
            true,
        )?;
        let empty = eq(boolean(), less(lower.clone(), upper.clone()), bit(false));
        let p = collapse(
            sub(upper.clone(), lower.clone()),
            nc(
                "EmptyLength",
                vec![lower.clone(), upper.clone(), var("_empty")],
            ),
        )?;
        self.emit(
            &format!("{name}Empty"),
            &[
                ("lower", nat()),
                ("upper", nat()),
                ("body", function.clone()),
                ("_empty", empty),
            ],
            eq(
                ty.clone(),
                kind.range(lower.clone(), upper.clone(), body.clone()),
                initial,
            ),
            p.proof,
            true,
        )?;

        let point = kind.relation(
            app(body.clone(), vec![index.clone()]),
            app(other.clone(), vec![index.clone()]),
        );
        let point_type = pi("_index", nat(), point);
        let left = kind.fold(lower.clone(), count.clone(), body.clone());
        let right = kind.fold(lower.clone(), count.clone(), other.clone());
        let at_n = |f| kind.fold(lower.clone(), index.clone(), f);
        let position = add(lower.clone(), index.clone());
        let cf = app(body.clone(), vec![position.clone()]);
        let cg = app(other.clone(), vec![position.clone()]);
        let property = kind.relation(at_n(body.clone()), at_n(other.clone()));
        let step = kind.congruence(
            at_n(body.clone()),
            at_n(other.clone()),
            cf.clone(),
            cg.clone(),
            var("_ih"),
            app(var("_pointwise"), vec![position.clone()]),
        )?;
        let proof = induct(
            "_index",
            property.clone(),
            kind.base_proof(),
            lam("_index", nat(), lam("_ih", property, step)),
            count.clone(),
        );
        self.emit(
            &format!("{name}Respects"),
            &[
                ("lower", nat()),
                ("count", nat()),
                ("body", function.clone()),
                ("other", function.clone()),
                ("_pointwise", point_type),
            ],
            kind.relation(left.clone(), right.clone()),
            proof,
            true,
        )?;

        // The actual visited prefix supplies the full equality premise. No
        // equality at an unvisited point is required by the bounded theorem.
        let predicate = lam(
            "_element",
            nat(),
            kind.compare(
                app(body.clone(), vec![var("_element")]),
                app(other.clone(), vec![var("_element")]),
            ),
        );
        let all_equal = |n| rc("FoldForall", vec![lower.clone(), n, predicate.clone()]);
        let property = pi(
            "_bounded",
            eq(boolean(), all_equal(index.clone()), bit(true)),
            kind.relation(at_n(body.clone()), at_n(other.clone())),
        );
        let current = kind.compare(cf.clone(), cg.clone());
        let previous_true = bc(
            "AndLeft",
            vec![all_equal(index.clone()), current.clone(), var("_bounded")],
        );
        let current_true = bc(
            "AndRight",
            vec![all_equal(index.clone()), current, var("_bounded")],
        );
        let reflected = kind.reflect(cf.clone(), cg.clone(), current_true);
        let past = app(var("_previous"), vec![previous_true]);
        let step_value = kind.congruence(
            at_n(body.clone()),
            at_n(other.clone()),
            cf,
            cg,
            past,
            reflected,
        )?;
        let base = lam(
            "_bounded",
            eq(boolean(), all_equal(zero()), bit(true)),
            kind.base_proof(),
        );
        let step = lam(
            "_index",
            nat(),
            lam(
                "_previous",
                property.clone(),
                lam(
                    "_bounded",
                    eq(boolean(), all_equal(succ(index)), bit(true)),
                    step_value,
                ),
            ),
        );
        let proof = induct("_index", property, base, step, count.clone());
        let result = pi(
            "_bounded",
            eq(boolean(), all_equal(count), bit(true)),
            kind.relation(left, right),
        );
        self.emit(
            &format!("{name}BoundedRespects"),
            &[
                ("lower", nat()),
                ("count", nat()),
                ("body", function.clone()),
                ("other", function.clone()),
            ],
            result,
            proof,
            true,
        )?;
        let bounded = eq(
            boolean(),
            rc("Forall", vec![lower.clone(), upper.clone(), predicate]),
            bit(true),
        );
        let result = kind.relation(
            kind.range(lower.clone(), upper.clone(), body.clone()),
            kind.range(lower.clone(), upper.clone(), other.clone()),
        );
        let proof = rc(
            &format!("{name}BoundedRespects"),
            vec![
                lower.clone(),
                sub(upper.clone(), lower.clone()),
                body,
                other,
                var("_bounded"),
            ],
        );
        self.emit(
            &format!("{name}IntervalRespects"),
            &[
                ("lower", nat()),
                ("upper", nat()),
                ("body", function.clone()),
                ("other", function),
                ("_bounded", bounded),
            ],
            result,
            proof,
            true,
        )
    }
}

pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut ranges = Ranges(builder);
    ranges.boolean_observations()?;
    for kind in [Kind::Forall, Kind::Exists, Kind::NatSum, Kind::IntSum] {
        ranges.family(kind)?;
    }
    Ok(())
}
