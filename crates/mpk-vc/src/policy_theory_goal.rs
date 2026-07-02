//! Neutral theory-goal extraction for payment-policy obligations.
//!
//! This module does not build trusted certificates. It only turns classified VC
//! obligations into small, deterministic goal descriptions that later closure
//! code can check and encode.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::expr_encode::{MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_NOT, STD_BOOL_OR, STD_EQ};
use crate::policy_obligation::{
    PaymentPolicyClassificationOutcome, PaymentPolicyObligationClassification,
    PaymentPolicyObligationPattern,
};
use crate::vc::VcObligation;

pub const MAX_POLICY_LINEAR_VARIABLES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTheoryGoal {
    pub obligation_id: String,
    pub function_id: String,
    pub pattern: PaymentPolicyObligationPattern,
    pub kind: PolicyTheoryGoalKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", content = "goal", rename_all = "snake_case")]
pub enum PolicyTheoryGoalKind {
    Linear(PolicyLinearGoal),
    BoolTautology(PolicyBoolGoal),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLinearGoal {
    pub variables: Vec<String>,
    pub premises: Vec<PolicyLinearInequality>,
    pub goal: PolicyLinearInequality,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLinearInequality {
    pub terms: Vec<PolicyLinearTerm>,
    pub constant: i128,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLinearTerm {
    pub variable: u32,
    pub coefficient: i128,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBoolGoal {
    pub reason: PolicyBoolTautologyReason,
    pub tautology: PolicyBoolTautology,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyBoolTautologyReason {
    ReflexiveSelectedBranchDisjunct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyBoolTautology {
    TrueOrOpaque,
    OpaqueOrTrue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyTheoryGoalError {
    kind: PolicyTheoryGoalErrorKind,
    detail: String,
}

impl PolicyTheoryGoalError {
    pub fn kind(&self) -> PolicyTheoryGoalErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: PolicyTheoryGoalErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for PolicyTheoryGoalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for PolicyTheoryGoalError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PolicyTheoryGoalErrorKind {
    ClassificationMismatch,
    MissingSupportedPattern,
    UnsupportedLinearConclusion,
    InvalidLiteral,
    InvalidVariable,
    TooManyVariables,
    ArithmeticOverflow,
}

impl PolicyTheoryGoalErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClassificationMismatch => "POLICY_THEORY_GOAL_CLASSIFICATION_MISMATCH",
            Self::MissingSupportedPattern => "POLICY_THEORY_GOAL_MISSING_SUPPORTED_PATTERN",
            Self::UnsupportedLinearConclusion => "POLICY_THEORY_GOAL_UNSUPPORTED_LINEAR_CONCLUSION",
            Self::InvalidLiteral => "POLICY_THEORY_GOAL_INVALID_LITERAL",
            Self::InvalidVariable => "POLICY_THEORY_GOAL_INVALID_VARIABLE",
            Self::TooManyVariables => "POLICY_THEORY_GOAL_TOO_MANY_VARIABLES",
            Self::ArithmeticOverflow => "POLICY_THEORY_GOAL_ARITHMETIC_OVERFLOW",
        }
    }
}

pub fn policy_theory_goal_from_obligation(
    obligation: &VcObligation,
    classification: &PaymentPolicyObligationClassification,
) -> Result<Option<PolicyTheoryGoal>, PolicyTheoryGoalError> {
    validate_classification_target(obligation, classification)?;
    if classification.outcome != PaymentPolicyClassificationOutcome::SupportedProperty {
        return Ok(None);
    }

    let pattern = classification.pattern.ok_or_else(|| {
        PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::MissingSupportedPattern,
            format!(
                "classification for obligation {:?} is supported but has no pattern",
                obligation.id
            ),
        )
    })?;
    if pattern == PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput {
        return Ok(bool_goal_from_branch_equality(obligation, pattern));
    }
    if is_linear_pattern(pattern) {
        return linear_goal_from_obligation(obligation, pattern);
    }

    Ok(None)
}

fn linear_goal_from_obligation(
    obligation: &VcObligation,
    pattern: PaymentPolicyObligationPattern,
) -> Result<Option<PolicyTheoryGoal>, PolicyTheoryGoalError> {
    let raw_goal = normalize_direct_linear_comparison(&obligation.conclusion)?.ok_or_else(|| {
        PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::UnsupportedLinearConclusion,
            format!(
                "classification pattern {:?} requires a supported linear conclusion for obligation {:?}",
                pattern, obligation.id
            ),
        )
    })?;
    let mut raw_premises = Vec::new();
    for assumption in &obligation.assumptions {
        if let Some(premise) = normalize_premise_linear_comparison(assumption)? {
            raw_premises.push(premise);
        }
    }

    let variables = collect_linear_variables(&raw_premises, &raw_goal)?;
    let premise_terms = raw_premises
        .iter()
        .map(|premise| materialize_linear_inequality(premise, &variables))
        .collect::<Result<Vec<_>, _>>()?;
    let goal = materialize_linear_inequality(&raw_goal, &variables)?;

    Ok(Some(PolicyTheoryGoal {
        obligation_id: obligation.id.clone(),
        function_id: obligation.function_id.clone(),
        pattern,
        kind: PolicyTheoryGoalKind::Linear(PolicyLinearGoal {
            variables,
            premises: premise_terms,
            goal,
        }),
    }))
}

fn bool_goal_from_branch_equality(
    obligation: &VcObligation,
    pattern: PaymentPolicyObligationPattern,
) -> Option<PolicyTheoryGoal> {
    let MpkExprTerm::Apply { function, args } = &obligation.conclusion else {
        return None;
    };
    if function != STD_BOOL_OR {
        return None;
    }
    let [lhs, rhs] = args.as_slice() else {
        return None;
    };

    let tautology = if is_reflexive_equality(lhs) {
        PolicyBoolTautology::TrueOrOpaque
    } else if is_reflexive_equality(rhs) {
        PolicyBoolTautology::OpaqueOrTrue
    } else {
        return None;
    };

    Some(PolicyTheoryGoal {
        obligation_id: obligation.id.clone(),
        function_id: obligation.function_id.clone(),
        pattern,
        kind: PolicyTheoryGoalKind::BoolTautology(PolicyBoolGoal {
            reason: PolicyBoolTautologyReason::ReflexiveSelectedBranchDisjunct,
            tautology,
        }),
    })
}

fn is_reflexive_equality(term: &MpkExprTerm) -> bool {
    let MpkExprTerm::Apply { function, args } = term else {
        return false;
    };
    let [lhs, rhs] = args.as_slice() else {
        return false;
    };
    function == STD_EQ && lhs == rhs
}

fn validate_classification_target(
    obligation: &VcObligation,
    classification: &PaymentPolicyObligationClassification,
) -> Result<(), PolicyTheoryGoalError> {
    if classification.obligation_id != obligation.id {
        return Err(PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::ClassificationMismatch,
            format!(
                "classification obligation_id {:?} does not match obligation {:?}",
                classification.obligation_id, obligation.id
            ),
        ));
    }
    if classification.function_id != obligation.function_id {
        return Err(PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::ClassificationMismatch,
            format!(
                "classification function_id {:?} does not match obligation function_id {:?}",
                classification.function_id, obligation.function_id
            ),
        ));
    }
    Ok(())
}

fn is_linear_pattern(pattern: PaymentPolicyObligationPattern) -> bool {
    matches!(
        pattern,
        PaymentPolicyObligationPattern::NonNegativeResult
            | PaymentPolicyObligationPattern::ResultBoundedByInput
            | PaymentPolicyObligationPattern::RefundBoundedByAvailablePaidAmount
            | PaymentPolicyObligationPattern::FeeOrDiscountBoundedByCap
    )
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RawLinearInequality {
    terms: BTreeMap<String, i128>,
    constant: i128,
}

impl RawLinearInequality {
    fn add_term(
        &mut self,
        variable: String,
        coefficient: i128,
    ) -> Result<(), PolicyTheoryGoalError> {
        let updated = checked_add(
            self.terms.get(&variable).copied().unwrap_or(0),
            coefficient,
            format!("coefficient accumulation for {variable:?}"),
        )?;
        if updated == 0 {
            self.terms.remove(&variable);
        } else {
            self.terms.insert(variable, updated);
        }
        Ok(())
    }

    fn add_constant(&mut self, value: i128) -> Result<(), PolicyTheoryGoalError> {
        self.constant = checked_add(self.constant, value, "constant accumulation")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinearComparisonOp {
    Sge,
    Sle,
    Sgt,
    Slt,
}

impl LinearComparisonOp {
    fn negated(self) -> Self {
        match self {
            Self::Sge => Self::Slt,
            Self::Sle => Self::Sgt,
            Self::Sgt => Self::Sle,
            Self::Slt => Self::Sge,
        }
    }
}

enum LinearOperand {
    Variable(String),
    Literal(i128),
}

fn normalize_direct_linear_comparison(
    term: &MpkExprTerm,
) -> Result<Option<RawLinearInequality>, PolicyTheoryGoalError> {
    let Some((op, lhs, rhs)) = linear_comparison_parts(term) else {
        return Ok(None);
    };
    normalize_linear_comparison(op, lhs, rhs)
}

fn normalize_premise_linear_comparison(
    term: &MpkExprTerm,
) -> Result<Option<RawLinearInequality>, PolicyTheoryGoalError> {
    if let Some((op, lhs, rhs)) = linear_comparison_parts(term) {
        return normalize_linear_comparison(op, lhs, rhs);
    }

    let MpkExprTerm::Apply { function, args } = term else {
        return Ok(None);
    };
    if function != STD_BOOL_NOT || args.len() != 1 {
        return Ok(None);
    }
    let Some((op, lhs, rhs)) = linear_comparison_parts(&args[0]) else {
        return Ok(None);
    };
    normalize_linear_comparison(op.negated(), lhs, rhs)
}

fn linear_comparison_parts(
    term: &MpkExprTerm,
) -> Option<(LinearComparisonOp, &MpkExprTerm, &MpkExprTerm)> {
    let MpkExprTerm::Apply { function, args } = term else {
        return None;
    };
    let op = linear_comparison_op(function)?;
    let [lhs, rhs] = args.as_slice() else {
        return None;
    };
    Some((op, lhs, rhs))
}

fn linear_comparison_op(function: &str) -> Option<LinearComparisonOp> {
    let (width, suffix) = bitvec_function_suffix(function)?;
    if width != 64 {
        return None;
    }
    match suffix {
        "sge" => Some(LinearComparisonOp::Sge),
        "sle" => Some(LinearComparisonOp::Sle),
        "sgt" => Some(LinearComparisonOp::Sgt),
        "slt" => Some(LinearComparisonOp::Slt),
        _ => None,
    }
}

fn bitvec_function_suffix(function: &str) -> Option<(u32, &str)> {
    let rest = function
        .strip_prefix(STD_BITVEC_MODULE)?
        .strip_prefix(".BV")?;
    let (width, suffix) = rest.split_once('.')?;
    let width = width.parse::<u32>().ok()?;
    Some((width, suffix))
}

fn normalize_linear_comparison(
    op: LinearComparisonOp,
    lhs: &MpkExprTerm,
    rhs: &MpkExprTerm,
) -> Result<Option<RawLinearInequality>, PolicyTheoryGoalError> {
    let Some(lhs) = linear_operand(lhs)? else {
        return Ok(None);
    };
    let Some(rhs) = linear_operand(rhs)? else {
        return Ok(None);
    };

    let mut inequality = RawLinearInequality::default();
    match op {
        LinearComparisonOp::Sge => {
            add_operand(&mut inequality, &rhs, 1)?;
            add_operand(&mut inequality, &lhs, -1)?;
        }
        LinearComparisonOp::Sle => {
            add_operand(&mut inequality, &lhs, 1)?;
            add_operand(&mut inequality, &rhs, -1)?;
        }
        LinearComparisonOp::Sgt => {
            add_operand(&mut inequality, &rhs, 1)?;
            add_operand(&mut inequality, &lhs, -1)?;
            inequality.add_constant(1)?;
        }
        LinearComparisonOp::Slt => {
            add_operand(&mut inequality, &lhs, 1)?;
            add_operand(&mut inequality, &rhs, -1)?;
            inequality.add_constant(1)?;
        }
    }
    Ok(Some(inequality))
}

fn linear_operand(term: &MpkExprTerm) -> Result<Option<LinearOperand>, PolicyTheoryGoalError> {
    match term {
        MpkExprTerm::Var { name } => {
            if name.is_empty() {
                return Err(PolicyTheoryGoalError::new(
                    PolicyTheoryGoalErrorKind::InvalidVariable,
                    "empty variable name",
                ));
            }
            Ok(Some(LinearOperand::Variable(format!("var:{name}"))))
        }
        MpkExprTerm::Result { index } => {
            Ok(Some(LinearOperand::Variable(format!("result:{index}"))))
        }
        MpkExprTerm::BitVecLiteral {
            value,
            width: 64,
            signed: true,
        } => Ok(Some(LinearOperand::Literal(parse_signed_bv64(value)?))),
        MpkExprTerm::BitVecLiteral { .. }
        | MpkExprTerm::Apply { .. }
        | MpkExprTerm::Convert { .. }
        | MpkExprTerm::Constant { .. } => Ok(None),
    }
}

fn parse_signed_bv64(value: &str) -> Result<i128, PolicyTheoryGoalError> {
    let parsed = value.parse::<i128>().map_err(|_| {
        PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::InvalidLiteral,
            format!("signed BV64 literal {value:?} cannot be parsed"),
        )
    })?;
    let min = -(1_i128 << 63);
    let max = (1_i128 << 63) - 1;
    if parsed < min || parsed > max {
        return Err(PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::InvalidLiteral,
            format!("signed BV64 literal {value:?} is outside i64 range"),
        ));
    }
    Ok(parsed)
}

fn add_operand(
    inequality: &mut RawLinearInequality,
    operand: &LinearOperand,
    coefficient: i128,
) -> Result<(), PolicyTheoryGoalError> {
    match operand {
        LinearOperand::Variable(variable) => inequality.add_term(variable.clone(), coefficient),
        LinearOperand::Literal(value) => {
            inequality.add_constant(checked_mul(*value, coefficient, "literal coefficient")?)
        }
    }
}

fn collect_linear_variables(
    premises: &[RawLinearInequality],
    goal: &RawLinearInequality,
) -> Result<Vec<String>, PolicyTheoryGoalError> {
    let mut variables = BTreeMap::<String, ()>::new();
    for variable in premises
        .iter()
        .flat_map(|premise| premise.terms.keys())
        .chain(goal.terms.keys())
    {
        variables.insert(variable.clone(), ());
    }
    if variables.len() > MAX_POLICY_LINEAR_VARIABLES {
        return Err(PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::TooManyVariables,
            format!(
                "linear goal uses {} variables; max={MAX_POLICY_LINEAR_VARIABLES}",
                variables.len()
            ),
        ));
    }
    Ok(variables.into_keys().collect())
}

fn materialize_linear_inequality(
    raw: &RawLinearInequality,
    variables: &[String],
) -> Result<PolicyLinearInequality, PolicyTheoryGoalError> {
    let ids = variables
        .iter()
        .enumerate()
        .map(|(index, variable)| {
            Ok((
                variable.as_str(),
                u32::try_from(index).map_err(|_| {
                    PolicyTheoryGoalError::new(
                        PolicyTheoryGoalErrorKind::TooManyVariables,
                        "linear variable index exceeds u32",
                    )
                })?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, PolicyTheoryGoalError>>()?;
    let terms = raw
        .terms
        .iter()
        .map(|(variable, coefficient)| {
            Ok(PolicyLinearTerm {
                variable: *ids.get(variable.as_str()).ok_or_else(|| {
                    PolicyTheoryGoalError::new(
                        PolicyTheoryGoalErrorKind::InvalidVariable,
                        format!("variable {variable:?} was not assigned an id"),
                    )
                })?,
                coefficient: *coefficient,
            })
        })
        .collect::<Result<Vec<_>, PolicyTheoryGoalError>>()?;
    Ok(PolicyLinearInequality {
        terms,
        constant: raw.constant,
    })
}

fn checked_add(
    lhs: i128,
    rhs: i128,
    context: impl Into<String>,
) -> Result<i128, PolicyTheoryGoalError> {
    lhs.checked_add(rhs).ok_or_else(|| {
        PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::ArithmeticOverflow,
            format!("addition overflow; {}", context.into()),
        )
    })
}

fn checked_mul(
    lhs: i128,
    rhs: i128,
    context: impl Into<String>,
) -> Result<i128, PolicyTheoryGoalError> {
    lhs.checked_mul(rhs).ok_or_else(|| {
        PolicyTheoryGoalError::new(
            PolicyTheoryGoalErrorKind::ArithmeticOverflow,
            format!("multiplication overflow; {}", context.into()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr_encode::{STD_BOOL_OR, STD_EQ};
    use crate::policy_obligation::classify_payment_policy_obligation;
    use crate::vc::VcObligationKind;

    fn obligation(conclusion: MpkExprTerm) -> VcObligation {
        VcObligation {
            id: "example.Policy.post0".to_owned(),
            function_id: "example.Policy".to_owned(),
            kind: VcObligationKind::Postcondition,
            assumptions: Vec::new(),
            conclusion,
        }
    }

    fn obligation_with_assumptions(
        assumptions: Vec<MpkExprTerm>,
        conclusion: MpkExprTerm,
    ) -> VcObligation {
        VcObligation {
            assumptions,
            ..obligation(conclusion)
        }
    }

    fn classification(obligation: &VcObligation) -> PaymentPolicyObligationClassification {
        classify_payment_policy_obligation(obligation)
    }

    fn apply(function: impl Into<String>, args: Vec<MpkExprTerm>) -> MpkExprTerm {
        MpkExprTerm::apply(function, args)
    }

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn result(index: u32) -> MpkExprTerm {
        MpkExprTerm::Result { index }
    }

    fn int64(value: &str) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_owned(),
            width: 64,
            signed: true,
        }
    }

    fn unsigned_int64(value: &str) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_owned(),
            width: 64,
            signed: false,
        }
    }

    fn sge(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sge"), vec![lhs, rhs])
    }

    fn sle(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sle"), vec![lhs, rhs])
    }

    fn sgt(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sgt"), vec![lhs, rhs])
    }

    fn slt(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.slt"), vec![lhs, rhs])
    }

    fn not(value: MpkExprTerm) -> MpkExprTerm {
        apply(STD_BOOL_NOT, vec![value])
    }

    fn add(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.add"), vec![lhs, rhs])
    }

    fn eq(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(STD_EQ, vec![lhs, rhs])
    }

    fn linear_goal(goal: Option<PolicyTheoryGoal>) -> PolicyLinearGoal {
        let Some(PolicyTheoryGoal {
            kind: PolicyTheoryGoalKind::Linear(goal),
            ..
        }) = goal
        else {
            panic!("expected linear goal");
        };
        goal
    }

    fn bool_goal(goal: Option<PolicyTheoryGoal>) -> PolicyBoolGoal {
        let Some(PolicyTheoryGoal {
            kind: PolicyTheoryGoalKind::BoolTautology(goal),
            ..
        }) = goal
        else {
            panic!("expected bool tautology goal");
        };
        goal
    }

    #[test]
    fn policy_theory_goal_extracts_exact_premise_linear_candidate() {
        let predicate = sge(result(0), int64("0"));
        let obligation = obligation_with_assumptions(vec![predicate.clone()], predicate);
        let goal = linear_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts linear goal"),
        );

        assert_eq!(goal.variables, vec!["result:0"]);
        assert_eq!(goal.premises, vec![goal.goal.clone()]);
        assert_eq!(
            goal.goal,
            PolicyLinearInequality {
                terms: vec![PolicyLinearTerm {
                    variable: 0,
                    coefficient: -1,
                }],
                constant: 0,
            }
        );
    }

    #[test]
    fn policy_theory_goal_extracts_identity_goal_without_terms() {
        let obligation = obligation(sle(var("approved"), var("approved")));
        let goal = linear_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts identity goal"),
        );

        assert!(goal.variables.is_empty());
        assert_eq!(
            goal.goal,
            PolicyLinearInequality {
                terms: Vec::new(),
                constant: 0,
            }
        );
    }

    #[test]
    fn policy_theory_goal_extracts_strict_branch_premise_candidate() {
        let obligation = obligation_with_assumptions(
            vec![sgt(var("requested"), var("balance"))],
            sle(var("balance"), var("requested")),
        );
        let goal = linear_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts strict branch premise"),
        );

        assert_eq!(goal.variables, vec!["var:balance", "var:requested"]);
        assert_eq!(
            goal.premises,
            vec![PolicyLinearInequality {
                terms: vec![
                    PolicyLinearTerm {
                        variable: 0,
                        coefficient: 1,
                    },
                    PolicyLinearTerm {
                        variable: 1,
                        coefficient: -1,
                    },
                ],
                constant: 1,
            }]
        );
        assert_eq!(
            goal.goal,
            PolicyLinearInequality {
                terms: vec![
                    PolicyLinearTerm {
                        variable: 0,
                        coefficient: 1,
                    },
                    PolicyLinearTerm {
                        variable: 1,
                        coefficient: -1,
                    },
                ],
                constant: 0,
            }
        );
    }

    #[test]
    fn policy_theory_goal_extracts_negated_strict_else_branch_premise_candidate() {
        let obligation = obligation_with_assumptions(
            vec![not(sgt(var("requested"), var("balance")))],
            sle(var("requested"), var("balance")),
        );
        let goal = linear_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts negated strict branch premise"),
        );

        assert_eq!(
            goal.premises,
            vec![PolicyLinearInequality {
                terms: vec![
                    PolicyLinearTerm {
                        variable: 0,
                        coefficient: -1,
                    },
                    PolicyLinearTerm {
                        variable: 1,
                        coefficient: 1,
                    },
                ],
                constant: 0,
            }]
        );
        assert_eq!(goal.premises[0], goal.goal);
    }

    #[test]
    fn policy_theory_goal_extracts_signed_bv64_literals() {
        let obligation = obligation(sle(result(0), int64("-5")));
        let mut classification = classification(&obligation);
        classification.outcome = PaymentPolicyClassificationOutcome::SupportedProperty;
        classification.pattern = Some(PaymentPolicyObligationPattern::ResultBoundedByInput);
        let goal = linear_goal(
            policy_theory_goal_from_obligation(&obligation, &classification)
                .expect("extracts literal comparison"),
        );

        assert_eq!(goal.variables, vec!["result:0"]);
        assert_eq!(
            goal.goal,
            PolicyLinearInequality {
                terms: vec![PolicyLinearTerm {
                    variable: 0,
                    coefficient: 1,
                }],
                constant: 5,
            }
        );
    }

    #[test]
    fn policy_theory_goal_returns_none_for_unsupported_arithmetic() {
        let obligation = obligation(sle(add(var("a"), var("b")), var("cap")));
        let goal = policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
            .expect("unsupported arithmetic is non-applicable");

        assert_eq!(goal, None);
    }

    #[test]
    fn policy_theory_goal_extracts_true_or_opaque_bool_branch_goal() {
        let obligation = obligation(apply(
            STD_BOOL_OR,
            vec![
                eq(var("selected"), var("selected")),
                eq(var("selected"), var("other")),
            ],
        ));
        let goal = bool_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts reflexive bool branch goal"),
        );

        assert_eq!(
            goal,
            PolicyBoolGoal {
                reason: PolicyBoolTautologyReason::ReflexiveSelectedBranchDisjunct,
                tautology: PolicyBoolTautology::TrueOrOpaque,
            }
        );
    }

    #[test]
    fn policy_theory_goal_extracts_opaque_or_true_bool_branch_goal() {
        let obligation = obligation(apply(
            STD_BOOL_OR,
            vec![
                eq(var("selected"), var("other")),
                eq(var("selected"), var("selected")),
            ],
        ));
        let goal = bool_goal(
            policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
                .expect("extracts reflexive bool branch goal"),
        );

        assert_eq!(
            goal,
            PolicyBoolGoal {
                reason: PolicyBoolTautologyReason::ReflexiveSelectedBranchDisjunct,
                tautology: PolicyBoolTautology::OpaqueOrTrue,
            }
        );
    }

    #[test]
    fn policy_theory_goal_returns_none_for_non_reflexive_bool_branch_goal() {
        let obligation = obligation(apply(
            STD_BOOL_OR,
            vec![eq(var("selected"), var("a")), eq(var("selected"), var("b"))],
        ));
        let goal = policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
            .expect("branch equality is non-applicable");

        assert_eq!(goal, None);
    }

    #[test]
    fn policy_theory_goal_returns_none_for_non_or_bool_branch_goal() {
        let obligation = obligation(eq(var("selected"), var("selected")));
        let mut classification = classification(&obligation);
        classification.outcome = PaymentPolicyClassificationOutcome::SupportedProperty;
        classification.pattern =
            Some(PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput);

        let goal = policy_theory_goal_from_obligation(&obligation, &classification)
            .expect("non-or branch equality is non-applicable");

        assert_eq!(goal, None);
    }

    #[test]
    fn policy_theory_goal_returns_none_for_wrong_arity_bool_branch_goal() {
        let obligation = obligation(apply(
            STD_BOOL_OR,
            vec![eq(var("selected"), var("selected"))],
        ));
        let mut classification = classification(&obligation);
        classification.outcome = PaymentPolicyClassificationOutcome::SupportedProperty;
        classification.pattern =
            Some(PaymentPolicyObligationPattern::SelectedBranchResultEqualsInput);

        let goal = policy_theory_goal_from_obligation(&obligation, &classification)
            .expect("wrong arity branch equality is non-applicable");

        assert_eq!(goal, None);
    }

    #[test]
    fn policy_theory_goal_rejects_too_many_linear_variables() {
        let mut assumptions = (0..MAX_POLICY_LINEAR_VARIABLES)
            .map(|index| sge(var(&format!("v{index:02}")), int64("0")))
            .collect::<Vec<_>>();
        assumptions.push(slt(var("overflow"), int64("10")));
        let obligation = obligation_with_assumptions(assumptions, sge(result(0), int64("0")));

        let error = policy_theory_goal_from_obligation(&obligation, &classification(&obligation))
            .expect_err("too many variables reject");

        assert_eq!(error.kind(), PolicyTheoryGoalErrorKind::TooManyVariables);
    }

    #[test]
    fn policy_theory_goal_rejects_supported_linear_classification_with_bad_conclusion() {
        let obligation = obligation(sle(var("approved"), unsigned_int64("0")));
        let mut classification = classification(&obligation);
        classification.outcome = PaymentPolicyClassificationOutcome::SupportedProperty;
        classification.pattern = Some(PaymentPolicyObligationPattern::ResultBoundedByInput);

        let error = policy_theory_goal_from_obligation(&obligation, &classification)
            .expect_err("contradictory supported classification rejects");

        assert_eq!(
            error.kind(),
            PolicyTheoryGoalErrorKind::UnsupportedLinearConclusion
        );
    }
}
