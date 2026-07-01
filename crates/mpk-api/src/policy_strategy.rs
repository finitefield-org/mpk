//! Product-level policy strategy profile metadata.
//!
//! Strategy profiles select helper workflow coverage for policy verification.
//! They are intentionally separate from checker `ProofProfile` values and from
//! axiom-policy allowlist profiles.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::theory_strategy::{TheoryStrategyCandidate, TheoryStrategyKind};

pub const PAYMENT_POLICY_ALPHA_PROFILE: &str = "payment-policy-alpha";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyStrategyProfile {
    PaymentPolicyAlpha,
}

impl PolicyStrategyProfile {
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::PaymentPolicyAlpha => PAYMENT_POLICY_ALPHA_PROFILE,
        }
    }

    pub fn metadata(self) -> PolicyStrategyMetadata {
        match self {
            Self::PaymentPolicyAlpha => PolicyStrategyMetadata {
                profile: self,
                allowed_obligation_patterns: vec![
                    PolicyObligationPattern::NonNegativeResult,
                    PolicyObligationPattern::ResultBoundedByInputAmount,
                    PolicyObligationPattern::RefundBoundedByPaidMinusAlreadyRefunded,
                    PolicyObligationPattern::DiscountOrFeeBoundedByConfiguredCaps,
                    PolicyObligationPattern::BranchResultEqualsSelectedInput,
                    PolicyObligationPattern::IntegerRuntimeSafety,
                ],
                candidate_theory_strategies: vec![
                    TheoryStrategyKind::Linarith,
                    TheoryStrategyKind::BitVecGround,
                    TheoryStrategyKind::BoolTautology,
                ],
            },
        }
    }
}

impl FromStr for PolicyStrategyProfile {
    type Err = PolicyStrategyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            PAYMENT_POLICY_ALPHA_PROFILE => Ok(Self::PaymentPolicyAlpha),
            _ => Err(PolicyStrategyError::unknown_strategy_profile(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyStrategyMetadata {
    pub profile: PolicyStrategyProfile,
    pub allowed_obligation_patterns: Vec<PolicyObligationPattern>,
    pub candidate_theory_strategies: Vec<TheoryStrategyKind>,
}

impl PolicyStrategyMetadata {
    pub fn for_profile(profile: PolicyStrategyProfile) -> Self {
        profile.metadata()
    }

    pub fn parse_profile(profile: &str) -> Result<Self, PolicyStrategyError> {
        Ok(profile.parse::<PolicyStrategyProfile>()?.metadata())
    }

    pub fn theory_candidates(&self) -> Vec<TheoryStrategyCandidate> {
        self.candidate_theory_strategies
            .iter()
            .copied()
            .map(|theory| TheoryStrategyCandidate { theory })
            .collect()
    }

    pub fn validate_obligation(
        &self,
        obligation: &PolicyObligationDescriptor,
    ) -> Result<(), PolicyStrategyError> {
        if self
            .allowed_obligation_patterns
            .contains(&obligation.pattern)
        {
            Ok(())
        } else {
            Err(PolicyStrategyError::obligation_outside_profile(
                self.profile,
                obligation,
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyObligationPattern {
    NonNegativeResult,
    ResultBoundedByInputAmount,
    RefundBoundedByPaidMinusAlreadyRefunded,
    DiscountOrFeeBoundedByConfiguredCaps,
    BranchResultEqualsSelectedInput,
    IntegerRuntimeSafety,
    ExternalStateInvariant,
}

impl PolicyObligationPattern {
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::NonNegativeResult => "non_negative_result",
            Self::ResultBoundedByInputAmount => "result_bounded_by_input_amount",
            Self::RefundBoundedByPaidMinusAlreadyRefunded => {
                "refund_bounded_by_paid_minus_already_refunded"
            }
            Self::DiscountOrFeeBoundedByConfiguredCaps => {
                "discount_or_fee_bounded_by_configured_caps"
            }
            Self::BranchResultEqualsSelectedInput => "branch_result_equals_selected_input",
            Self::IntegerRuntimeSafety => "integer_runtime_safety",
            Self::ExternalStateInvariant => "external_state_invariant",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyObligationDescriptor {
    pub obligation_id: String,
    pub pattern: PolicyObligationPattern,
}

impl PolicyObligationDescriptor {
    pub fn new(obligation_id: impl Into<String>, pattern: PolicyObligationPattern) -> Self {
        Self {
            obligation_id: obligation_id.into(),
            pattern,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyStrategyError {
    pub code: PolicyStrategyErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl PolicyStrategyError {
    fn unknown_strategy_profile(profile: &str) -> Self {
        Self {
            code: PolicyStrategyErrorCode::UnknownStrategyProfile,
            message: format!(
                "unknown policy strategy profile {profile:?}; expected one of: {PAYMENT_POLICY_ALPHA_PROFILE}"
            ),
            field: Some("strategy_profile".to_owned()),
            detail: Some(profile.to_owned()),
        }
    }

    fn obligation_outside_profile(
        profile: PolicyStrategyProfile,
        obligation: &PolicyObligationDescriptor,
    ) -> Self {
        let profile_name = profile.canonical_name();
        let pattern_name = obligation.pattern.canonical_name();
        Self {
            code: PolicyStrategyErrorCode::ObligationOutsideProfile,
            message: format!(
                "obligation {:?} with pattern {pattern_name:?} is outside policy strategy profile {profile_name:?}",
                obligation.obligation_id
            ),
            field: Some("obligation.pattern".to_owned()),
            detail: Some(format!(
                "profile={profile_name}; obligation_id={}; pattern={pattern_name}",
                obligation.obligation_id
            )),
        }
    }
}

impl fmt::Display for PolicyStrategyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for PolicyStrategyError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyStrategyErrorCode {
    UnknownStrategyProfile,
    ObligationOutsideProfile,
}

impl PolicyStrategyErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownStrategyProfile => "UNKNOWN_STRATEGY_PROFILE",
            Self::ObligationOutsideProfile => "OBLIGATION_OUTSIDE_PROFILE",
        }
    }
}
