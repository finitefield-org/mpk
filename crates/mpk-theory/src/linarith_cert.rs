//! Simple Farkas-style checker for non-strict integer linear inequalities.
//!
//! Inequalities are represented in canonical `linear_expr <= 0` form. A
//! certificate is accepted only when a non-negative integer combination of
//! premises recomputes a value that is pointwise at least as strong as the goal.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const LINARITH_CERT_FORMAT: &str = "mpk.linarith.v0";
pub const MAX_LINARITH_PREMISES: usize = 64;
pub const MAX_LINARITH_TERMS_PER_INEQUALITY: usize = 64;
pub const MAX_LINARITH_COMBINATION_TERMS: usize = 64;
pub const MAX_LINARITH_VARIABLES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinarithCertificate {
    pub premises: Vec<LinearInequality>,
    pub goal: LinearInequality,
    pub combination: Vec<FarkasMultiplier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearInequality {
    pub terms: Vec<LinearTerm>,
    pub constant: i128,
}

impl LinearInequality {
    pub fn new(terms: Vec<LinearTerm>, constant: i128) -> Self {
        Self { terms, constant }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LinearTerm {
    pub variable: u32,
    pub coefficient: i128,
}

impl LinearTerm {
    pub fn new(variable: u32, coefficient: i128) -> Self {
        Self {
            variable,
            coefficient,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FarkasMultiplier {
    pub premise_index: usize,
    pub multiplier: u64,
}

impl FarkasMultiplier {
    pub fn new(premise_index: usize, multiplier: u64) -> Self {
        Self {
            premise_index,
            multiplier,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinarithCertificateSummary {
    pub premise_count: usize,
    pub premises_used: usize,
    pub combination_terms: usize,
    pub variable_count: usize,
    pub slack: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinarithCertError {
    kind: LinarithCertErrorKind,
    detail: String,
}

impl LinarithCertError {
    pub fn kind(&self) -> LinarithCertErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: LinarithCertErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for LinarithCertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for LinarithCertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum LinarithCertErrorKind {
    TooManyPremises,
    TooManyTerms,
    TooManyCombinationTerms,
    TooManyVariables,
    InvalidPremiseIndex,
    ZeroMultiplier,
    ArithmeticOverflow,
    CombinationDoesNotProveGoal,
}

impl LinarithCertErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TooManyPremises => "LINARITH_TOO_MANY_PREMISES",
            Self::TooManyTerms => "LINARITH_TOO_MANY_TERMS",
            Self::TooManyCombinationTerms => "LINARITH_TOO_MANY_COMBINATION_TERMS",
            Self::TooManyVariables => "LINARITH_TOO_MANY_VARIABLES",
            Self::InvalidPremiseIndex => "LINARITH_INVALID_PREMISE_INDEX",
            Self::ZeroMultiplier => "LINARITH_ZERO_MULTIPLIER",
            Self::ArithmeticOverflow => "LINARITH_ARITHMETIC_OVERFLOW",
            Self::CombinationDoesNotProveGoal => "LINARITH_COMBINATION_DOES_NOT_PROVE_GOAL",
        }
    }
}

pub fn check_linarith_certificate(
    certificate: &LinarithCertificate,
) -> Result<LinarithCertificateSummary, LinarithCertError> {
    if certificate.premises.len() > MAX_LINARITH_PREMISES {
        return Err(LinarithCertError::new(
            LinarithCertErrorKind::TooManyPremises,
            format!(
                "premises={}; max={MAX_LINARITH_PREMISES}",
                certificate.premises.len()
            ),
        ));
    }
    if certificate.combination.len() > MAX_LINARITH_COMBINATION_TERMS {
        return Err(LinarithCertError::new(
            LinarithCertErrorKind::TooManyCombinationTerms,
            format!(
                "combination_terms={}; max={MAX_LINARITH_COMBINATION_TERMS}",
                certificate.combination.len()
            ),
        ));
    }

    let premises = certificate
        .premises
        .iter()
        .enumerate()
        .map(|(index, premise)| NormalizedLinearExpr::from_inequality(premise, "premise", index))
        .collect::<Result<Vec<_>, _>>()?;
    let goal = NormalizedLinearExpr::from_inequality(&certificate.goal, "goal", 0)?;

    let variable_count = count_variables(&premises, &goal)?;
    let mut combined = NormalizedLinearExpr::zero();
    let mut used_premises = BTreeSet::new();
    for (row_index, row) in certificate.combination.iter().enumerate() {
        if row.multiplier == 0 {
            return Err(LinarithCertError::new(
                LinarithCertErrorKind::ZeroMultiplier,
                format!("row={row_index}; premise_index={}", row.premise_index),
            ));
        }
        let premise = premises.get(row.premise_index).ok_or_else(|| {
            LinarithCertError::new(
                LinarithCertErrorKind::InvalidPremiseIndex,
                format!(
                    "row={row_index}; premise_index={}; premise_count={}",
                    row.premise_index,
                    premises.len()
                ),
            )
        })?;
        combined.checked_scaled_add(premise, row.multiplier, row_index)?;
        used_premises.insert(row.premise_index);
    }

    let residual = combined.checked_sub(&goal)?;
    if let Some((variable, coefficient)) = residual.terms.iter().next() {
        return Err(LinarithCertError::new(
            LinarithCertErrorKind::CombinationDoesNotProveGoal,
            format!(
                "residual variable term remains; variable={variable}; coefficient={coefficient}"
            ),
        ));
    }
    if residual.constant < 0 {
        return Err(LinarithCertError::new(
            LinarithCertErrorKind::CombinationDoesNotProveGoal,
            format!("negative constant slack={}", residual.constant),
        ));
    }

    Ok(LinarithCertificateSummary {
        premise_count: certificate.premises.len(),
        premises_used: used_premises.len(),
        combination_terms: certificate.combination.len(),
        variable_count,
        slack: residual.constant,
    })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct NormalizedLinearExpr {
    terms: BTreeMap<u32, i128>,
    constant: i128,
}

impl NormalizedLinearExpr {
    fn zero() -> Self {
        Self::default()
    }

    fn from_inequality(
        input: &LinearInequality,
        label: &'static str,
        index: usize,
    ) -> Result<Self, LinarithCertError> {
        if input.terms.len() > MAX_LINARITH_TERMS_PER_INEQUALITY {
            return Err(LinarithCertError::new(
                LinarithCertErrorKind::TooManyTerms,
                format!(
                    "{label}={index}; terms={}; max={MAX_LINARITH_TERMS_PER_INEQUALITY}",
                    input.terms.len()
                ),
            ));
        }

        let mut normalized = Self {
            terms: BTreeMap::new(),
            constant: input.constant,
        };
        for term in &input.terms {
            normalized.checked_add_term(
                term.variable,
                term.coefficient,
                format!("{label}={index}; variable={}", term.variable),
            )?;
        }
        Ok(normalized)
    }

    fn checked_scaled_add(
        &mut self,
        other: &Self,
        multiplier: u64,
        row_index: usize,
    ) -> Result<(), LinarithCertError> {
        let multiplier = i128::from(multiplier);
        let scaled_constant = checked_mul(
            other.constant,
            multiplier,
            format!("row={row_index}; field=constant"),
        )?;
        self.constant = checked_add(
            self.constant,
            scaled_constant,
            format!("row={row_index}; field=constant"),
        )?;
        for (variable, coefficient) in &other.terms {
            let scaled = checked_mul(
                *coefficient,
                multiplier,
                format!("row={row_index}; variable={variable}"),
            )?;
            self.checked_add_term(
                *variable,
                scaled,
                format!("row={row_index}; variable={variable}"),
            )?;
        }
        Ok(())
    }

    fn checked_sub(&self, rhs: &Self) -> Result<Self, LinarithCertError> {
        let mut result = self.clone();
        result.constant = checked_sub(result.constant, rhs.constant, "field=constant")?;
        for (variable, coefficient) in &rhs.terms {
            let negated = checked_neg(*coefficient, format!("variable={variable}"))?;
            result.checked_add_term(*variable, negated, format!("variable={variable}"))?;
        }
        Ok(result)
    }

    fn checked_add_term(
        &mut self,
        variable: u32,
        coefficient: i128,
        context: impl Into<String>,
    ) -> Result<(), LinarithCertError> {
        if coefficient == 0 {
            return Ok(());
        }
        let current = *self.terms.get(&variable).unwrap_or(&0);
        let next = checked_add(current, coefficient, context)?;
        if next == 0 {
            self.terms.remove(&variable);
        } else {
            self.terms.insert(variable, next);
        }
        Ok(())
    }
}

fn count_variables(
    premises: &[NormalizedLinearExpr],
    goal: &NormalizedLinearExpr,
) -> Result<usize, LinarithCertError> {
    let mut variables = BTreeSet::new();
    for premise in premises {
        variables.extend(premise.terms.keys().copied());
    }
    variables.extend(goal.terms.keys().copied());
    if variables.len() > MAX_LINARITH_VARIABLES {
        return Err(LinarithCertError::new(
            LinarithCertErrorKind::TooManyVariables,
            format!(
                "variables={}; max={MAX_LINARITH_VARIABLES}",
                variables.len()
            ),
        ));
    }
    Ok(variables.len())
}

fn checked_add(
    lhs: i128,
    rhs: i128,
    context: impl Into<String>,
) -> Result<i128, LinarithCertError> {
    lhs.checked_add(rhs).ok_or_else(|| {
        LinarithCertError::new(
            LinarithCertErrorKind::ArithmeticOverflow,
            format!("addition overflow; {}", context.into()),
        )
    })
}

fn checked_sub(
    lhs: i128,
    rhs: i128,
    context: impl Into<String>,
) -> Result<i128, LinarithCertError> {
    lhs.checked_sub(rhs).ok_or_else(|| {
        LinarithCertError::new(
            LinarithCertErrorKind::ArithmeticOverflow,
            format!("subtraction overflow; {}", context.into()),
        )
    })
}

fn checked_mul(
    lhs: i128,
    rhs: i128,
    context: impl Into<String>,
) -> Result<i128, LinarithCertError> {
    lhs.checked_mul(rhs).ok_or_else(|| {
        LinarithCertError::new(
            LinarithCertErrorKind::ArithmeticOverflow,
            format!("multiplication overflow; {}", context.into()),
        )
    })
}

fn checked_neg(value: i128, context: impl Into<String>) -> Result<i128, LinarithCertError> {
    value.checked_neg().ok_or_else(|| {
        LinarithCertError::new(
            LinarithCertErrorKind::ArithmeticOverflow,
            format!("negation overflow; {}", context.into()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(variable: u32, coefficient: i128) -> LinearTerm {
        LinearTerm::new(variable, coefficient)
    }

    fn ineq(terms: Vec<LinearTerm>, constant: i128) -> LinearInequality {
        LinearInequality::new(terms, constant)
    }

    fn row(premise_index: usize, multiplier: u64) -> FarkasMultiplier {
        FarkasMultiplier::new(premise_index, multiplier)
    }

    #[test]
    fn checks_transitive_non_strict_linear_fixture() {
        let certificate = LinarithCertificate {
            premises: vec![
                ineq(vec![term(0, 1), term(1, -1)], 0),
                ineq(vec![term(1, 1), term(2, -1)], 0),
            ],
            goal: ineq(vec![term(0, 1), term(2, -1)], 0),
            combination: vec![row(0, 1), row(1, 1)],
        };

        let summary = check_linarith_certificate(&certificate).expect("certificate checks");

        assert_eq!(
            summary,
            LinarithCertificateSummary {
                premise_count: 2,
                premises_used: 2,
                combination_terms: 2,
                variable_count: 3,
                slack: 0,
            }
        );
    }

    #[test]
    fn checks_scaled_premise_fixture() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], 0)],
            goal: ineq(vec![term(0, 2)], 0),
            combination: vec![row(0, 2)],
        };

        let summary = check_linarith_certificate(&certificate).expect("scaled premise checks");

        assert_eq!(summary.slack, 0);
        assert_eq!(summary.premises_used, 1);
    }

    #[test]
    fn checks_constant_slack_fixture() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], 0)],
            goal: ineq(vec![term(0, 1)], -1),
            combination: vec![row(0, 1)],
        };

        let summary = check_linarith_certificate(&certificate).expect("slack checks");

        assert_eq!(summary.slack, 1);
    }

    #[test]
    fn checks_trivial_constant_goal_without_premises() {
        let certificate = LinarithCertificate {
            premises: Vec::new(),
            goal: ineq(Vec::new(), -1),
            combination: Vec::new(),
        };

        let summary = check_linarith_certificate(&certificate).expect("constant goal checks");

        assert_eq!(summary.premise_count, 0);
        assert_eq!(summary.slack, 1);
    }

    #[test]
    fn normalizes_duplicate_terms() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1), term(0, 1)], 0)],
            goal: ineq(vec![term(0, 2)], 0),
            combination: vec![row(0, 1)],
        };

        let summary = check_linarith_certificate(&certificate).expect("duplicates normalize");

        assert_eq!(summary.variable_count, 1);
        assert_eq!(summary.slack, 0);
    }

    #[test]
    fn rejects_unproven_variable_residual() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], 0)],
            goal: ineq(vec![term(1, 1)], 0),
            combination: vec![row(0, 1)],
        };

        let error = check_linarith_certificate(&certificate).expect_err("residual rejects");

        assert_eq!(
            error.kind(),
            LinarithCertErrorKind::CombinationDoesNotProveGoal
        );
        assert_eq!(
            error.detail(),
            "residual variable term remains; variable=0; coefficient=1"
        );
    }

    #[test]
    fn rejects_negative_constant_slack() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], -1)],
            goal: ineq(vec![term(0, 1)], 0),
            combination: vec![row(0, 1)],
        };

        let error = check_linarith_certificate(&certificate).expect_err("negative slack rejects");

        assert_eq!(
            error.kind(),
            LinarithCertErrorKind::CombinationDoesNotProveGoal
        );
        assert_eq!(error.detail(), "negative constant slack=-1");
    }

    #[test]
    fn rejects_invalid_premise_index() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], 0)],
            goal: ineq(vec![term(0, 1)], 0),
            combination: vec![row(1, 1)],
        };

        let error = check_linarith_certificate(&certificate).expect_err("bad index rejects");

        assert_eq!(error.kind(), LinarithCertErrorKind::InvalidPremiseIndex);
        assert_eq!(error.detail(), "row=0; premise_index=1; premise_count=1");
    }

    #[test]
    fn rejects_zero_multiplier() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, 1)], 0)],
            goal: ineq(vec![term(0, 1)], 0),
            combination: vec![row(0, 0)],
        };

        let error = check_linarith_certificate(&certificate).expect_err("zero rejects");

        assert_eq!(error.kind(), LinarithCertErrorKind::ZeroMultiplier);
        assert_eq!(error.detail(), "row=0; premise_index=0");
    }

    #[test]
    fn rejects_arithmetic_overflow() {
        let certificate = LinarithCertificate {
            premises: vec![ineq(vec![term(0, i128::MAX)], 0)],
            goal: ineq(vec![term(0, 1)], 0),
            combination: vec![row(0, 2)],
        };

        let error = check_linarith_certificate(&certificate).expect_err("overflow rejects");

        assert_eq!(error.kind(), LinarithCertErrorKind::ArithmeticOverflow);
        assert_eq!(error.detail(), "multiplication overflow; row=0; variable=0");
    }
}
