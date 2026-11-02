//! Actual Abs/SignedDiv/SignedRem supply every scalar value, sign and premise.
use super::*;

fn field(w: usize, name: &str, x: Expr, y: Expr) -> Expr {
    sd(&format!("W{w}.{name}"), vec![x, y])
}
fn unsigned_word(w: usize, name: &str, x: Expr, y: Expr) -> Expr {
    wc(
        w,
        name,
        vec![wc(w, "AbsBits", vec![x]), wc(w, "AbsBits", vec![y])],
    )
}
fn binary_equality(name: &str, a: Equality, b: Equality) -> Result<Equality> {
    let first = a.clone().mapped(
        lam(
            "_left_int",
            integer(),
            ic(name, vec![var("_left_int"), b.left.clone()]),
        ),
        ic(name, vec![a.left.clone(), b.left.clone()]),
        ic(name, vec![a.right.clone(), b.left.clone()]),
        integer(),
    );
    first.then(b.clone().mapped(
        lam(
            "_right_int",
            integer(),
            ic(name, vec![a.right.clone(), var("_right_int")]),
        ),
        ic(name, vec![a.right.clone(), b.left]),
        ic(name, vec![a.right, b.right]),
        integer(),
    ))
}
fn int_reflexive(value: Expr) -> Equality {
    Equality {
        ty: integer(),
        left: value.clone(),
        right: value.clone(),
        proof: refl(integer(), value),
    }
}
struct Operator {
    mathematical: Equality,
    refinement: Relation,
}
impl Refinement<'_> {
    fn integer_definition(
        &mut self,
        w: usize,
        label: &str,
        args: &[Binder],
        body: Expr,
    ) -> Result<Equality> {
        self.emit(
            &format!("W{w}.{label}"),
            args,
            integer(),
            body.clone(),
            false,
        )?;
        let e = Equality {
            ty: integer(),
            left: field(w, label, var("x"), var("y")),
            right: body.clone(),
            proof: refl(integer(), body),
        };
        self.theorem(&format!("W{w}.{label}Definition"), args, e.clone())?;
        Ok(Equality {
            proof: field(w, &format!("{label}Definition"), var("x"), var("y")),
            ..e
        })
    }
    fn operator(&mut self, w: usize, quotient: bool) -> Result<Operator> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let args = [("x", word_type(w)?), ("y", word_type(w)?)];
        let label = if quotient { "Quotient" } else { "Remainder" };
        let source_name = if quotient { "SignedDiv" } else { "SignedRem" };
        let unsigned_name = if quotient {
            "UnsignedDiv"
        } else {
            "UnsignedRem"
        };
        let word = unsigned_word(w, unsigned_name, x.clone(), y.clone());
        let flag = if quotient {
            bc("Xor", vec![sign(w, x.clone())?, sign(w, y.clone())?])
        } else {
            sign(w, x.clone())?
        };
        let amount = value(d, word.clone());
        let mathematical = self.integer_definition(
            w,
            &format!("Mathematical{label}"),
            &args,
            signed_nat(flag.clone(), amount.clone()),
        )?;
        let correction = self.integer_definition(
            w,
            &format!("{label}Correction"),
            &args,
            correction(w, word.clone(), flag.clone()),
        )?;
        let source = wc(w, source_name, vec![x.clone(), y.clone()]);
        let selected = selection(w, word.clone(), flag.clone());
        let definition = Equality {
            ty: word_type(w)?,
            left: source.clone(),
            right: selected.clone(),
            proof: refl(word_type(w)?, selected.clone()),
        };
        self.theorem(
            &format!("W{w}.{source_name}Definition"),
            &args,
            definition.clone(),
        )?;
        let definition = Equality {
            proof: field(w, &format!("{source_name}Definition"), x.clone(), y.clone()),
            ..definition
        };
        let signed_definition = definition.mapped(
            lam(
                "_source_word",
                word_type(w)?,
                signed(w, var("_source_word")),
            ),
            signed(w, source),
            signed(w, selected.clone()),
            integer(),
        );
        let scaled_correction = binary_equality(
            "Mul",
            int_reflexive(ic("FromNat", vec![weight(d)])),
            correction,
        )?;
        let residual = binary_equality("Add", signed_definition, scaled_correction)?;
        let selection = Relation::new(
            signed_nat(flag.clone(), amount),
            residual.right.clone(),
            sd(&format!("W{w}.SignedSelectionRefinement"), vec![word, flag]),
        );
        let refinement = selection
            .replace_left(mathematical.clone())?
            .replace_right(residual)?;
        self.relation(
            &format!("W{w}.{label}Refinement"),
            &args,
            Relation::new(
                refinement.left.clone(),
                refinement.right.clone(),
                refinement.proof,
            ),
        )?;
        let refinement = Relation::new(
            refinement.left,
            refinement.right,
            field(w, &format!("{label}Refinement"), x, y),
        );
        Ok(Operator {
            mathematical,
            refinement,
        })
    }
    pub(super) fn family(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let (x, y) = (var("x"), var("y"));
        let mut args = vec![("x", word_type(w)?), ("y", word_type(w)?)];
        let guard = wc(w, "DivideDefined", vec![y.clone()]);
        args.push(("defined", eq(boolean(), guard.clone(), bit(true))));
        let ax = wc(w, "AbsBits", vec![x.clone()]);
        let ay = wc(w, "AbsBits", vec![y.clone()]);
        let abs_guard = Equality::boolean(
            wc(w, "DivideDefined", vec![ay.clone()]),
            guard.clone(),
            prerequisite(
                "Magnitude",
                &format!("W{w}.MagnitudeDivideDefined"),
                vec![y.clone()],
            ),
        )
        .then(Equality::boolean(guard, bit(true), var("defined")))?;
        let uq = unsigned_word(w, "UnsignedDiv", x.clone(), y.clone());
        let ur = unsigned_word(w, "UnsignedRem", x.clone(), y.clone());
        let (a, b, q, r) = (
            value(d, ax.clone()),
            value(d, ay.clone()),
            value(d, uq),
            value(d, ur),
        );
        let sum = add(mul(b.clone(), q.clone()), r.clone());
        let balance = Equality::natural(
            a.clone(),
            sum.clone(),
            prerequisite(
                "Unsigned",
                &format!("W{w}.UnsignedDivisionConservation"),
                vec![ax.clone(), ay.clone(), abs_guard.proof.clone()],
            ),
        );
        self.theorem(
            &format!("W{w}.MagnitudeDivisionConservation"),
            &args,
            balance,
        )?;
        self.theorem(
            &format!("W{w}.MagnitudeRemainderBound"),
            &args,
            Equality::boolean(
                less(r.clone(), b.clone()),
                bit(true),
                prerequisite(
                    "Unsigned",
                    &format!("W{w}.UnsignedRemainderBound"),
                    vec![ax, ay, abs_guard.proof],
                ),
            ),
        )?;
        let quotient = self.operator(w, true)?;
        let remainder = self.operator(w, false)?;
        let sx = sign(w, x.clone())?;
        let sy = sign(w, y.clone())?;
        let mx = signed_nat(sx.clone(), a.clone());
        let my = signed_nat(sy.clone(), b.clone());
        let x_relation = Relation::new(
            signed(w, x.clone()),
            mx.clone(),
            prerequisite(
                "Magnitude",
                &format!("W{w}.SignedMagnitudeEquivalent"),
                vec![x.clone()],
            ),
        );
        let y_relation = Relation::new(
            signed(w, y.clone()),
            my.clone(),
            prerequisite(
                "Magnitude",
                &format!("W{w}.SignedMagnitudeEquivalent"),
                vec![y.clone()],
            ),
        );
        let scalar = Relation::new(
            mx,
            ic(
                "Add",
                vec![
                    ic("Mul", vec![my, quotient.mathematical.right.clone()]),
                    remainder.mathematical.right.clone(),
                ],
            ),
            sd(
                "ScalarSignedDivision",
                vec![
                    a,
                    b,
                    q,
                    r,
                    sd(
                        &format!("W{w}.MagnitudeDivisionConservation"),
                        vec![x.clone(), y.clone(), var("defined")],
                    ),
                    sx,
                    sy,
                ],
            ),
        );
        let mathematical = x_relation.then(scalar)?.then(
            y_relation
                .reverse()
                .times_right(quotient.mathematical.right.clone())
                .plus(Relation::reflexive(remainder.mathematical.right.clone())),
        )?;
        let aliases = binary_equality(
            "Add",
            binary_equality(
                "Mul",
                int_reflexive(signed(w, y.clone())),
                quotient.mathematical,
            )?,
            remainder.mathematical,
        )?;
        let mathematical = mathematical.replace_right(aliases)?;
        self.relation(
            &format!("W{w}.MathematicalDivisionConservation"),
            &args,
            Relation::new(
                mathematical.left.clone(),
                mathematical.right.clone(),
                mathematical.proof,
            ),
        )?;
        let mathematical = Relation::new(
            mathematical.left,
            mathematical.right,
            sd(
                &format!("W{w}.MathematicalDivisionConservation"),
                vec![x.clone(), y.clone(), var("defined")],
            ),
        );
        self.relation(
            &format!("W{w}.SignedDivisionConservation"),
            &args,
            mathematical.then(
                quotient
                    .refinement
                    .times_left(signed(w, y))
                    .plus(remainder.refinement),
            )?,
        )
    }
}
