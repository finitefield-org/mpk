//! Product-level policy strategy profile metadata.
//!
//! Strategy profiles select helper workflow coverage for policy verification.
//! They are intentionally separate from checker `ProofProfile` values and from
//! axiom-policy allowlist profiles.

use std::fmt;
use std::str::FromStr;

use mpk_vc::{SemanticProfile, SourceLanguage};
use serde::{Deserialize, Serialize};

use crate::theory_strategy::{TheoryStrategyCandidate, TheoryStrategyKind};

pub const PAYMENT_POLICY_ALPHA_PROFILE: &str = "payment-policy-alpha";
pub const PAYMENT_POLICY_RUST_ALPHA_PROFILE: &str = "payment-policy-rust-alpha";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyStrategyProfile {
    PaymentPolicyAlpha,
    PaymentPolicyRustAlpha,
}

impl PolicyStrategyProfile {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::PaymentPolicyAlpha => PAYMENT_POLICY_ALPHA_PROFILE,
            Self::PaymentPolicyRustAlpha => PAYMENT_POLICY_RUST_ALPHA_PROFILE,
        }
    }

    pub const fn registration(self) -> PolicyStrategyRegistration {
        match self {
            Self::PaymentPolicyAlpha => PolicyStrategyRegistration {
                strategy_profile: Self::PaymentPolicyAlpha,
                source_language: SourceLanguage::Go,
                semantic_profile: SemanticProfile::GoFixedV0,
                axiom_profile: PolicyAxiomProfile::ZeroAxiom,
            },
            Self::PaymentPolicyRustAlpha => PolicyStrategyRegistration {
                strategy_profile: Self::PaymentPolicyRustAlpha,
                source_language: SourceLanguage::Rust,
                semantic_profile: SemanticProfile::RustCheckedV0,
                axiom_profile: PolicyAxiomProfile::MvpTheory,
            },
        }
    }

    pub const fn readiness_description(self, readiness: PolicyReadiness) -> &'static str {
        match (self, readiness) {
            (Self::PaymentPolicyAlpha, PolicyReadiness::Ready) => {
                "The selected Go function is ready for policy proof search."
            }
            (Self::PaymentPolicyAlpha, PolicyReadiness::Unsupported) => {
                "The selected Go source uses a feature outside the supported verification subset."
            }
            (Self::PaymentPolicyAlpha, PolicyReadiness::SourceError) => {
                "The selected Go source has a source error and is not ready for policy proof search."
            }
            (Self::PaymentPolicyAlpha, PolicyReadiness::FrontendError) => {
                "The registered Go frontend failed before producing a validated policy input."
            }
            (Self::PaymentPolicyRustAlpha, PolicyReadiness::Ready) => {
                "The selected Rust function is ready for policy proof search under checked arithmetic and abort-on-panic semantics."
            }
            (Self::PaymentPolicyRustAlpha, PolicyReadiness::Unsupported) => {
                "The selected Rust source uses a feature outside the supported verification subset."
            }
            (Self::PaymentPolicyRustAlpha, PolicyReadiness::SourceError) => {
                "The selected Rust source has a source error and is not ready for policy proof search."
            }
            (Self::PaymentPolicyRustAlpha, PolicyReadiness::FrontendError) => {
                "The registered Rust frontend failed before producing a validated policy input."
            }
        }
    }

    pub fn metadata(self) -> PolicyStrategyMetadata {
        match self {
            Self::PaymentPolicyAlpha | Self::PaymentPolicyRustAlpha => PolicyStrategyMetadata {
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
            PAYMENT_POLICY_RUST_ALPHA_PROFILE => Ok(Self::PaymentPolicyRustAlpha),
            _ => Err(PolicyStrategyError::unknown_strategy_profile(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyAxiomProfile {
    ZeroAxiom,
    CoreMvp,
    MvpTheory,
    GoArtifactAlpha,
    ExperimentalExternal,
}

impl PolicyAxiomProfile {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::ZeroAxiom => "zero-axiom",
            Self::CoreMvp => "core-mvp",
            Self::MvpTheory => "mvp-theory",
            Self::GoArtifactAlpha => "go-artifact-alpha",
            Self::ExperimentalExternal => "experimental-external",
        }
    }

    pub fn parse_registered(value: &str) -> Option<Self> {
        match value {
            "zero-axiom" => Some(Self::ZeroAxiom),
            "core-mvp" => Some(Self::CoreMvp),
            "mvp-theory" => Some(Self::MvpTheory),
            "go-artifact-alpha" => Some(Self::GoArtifactAlpha),
            "experimental-external" => Some(Self::ExperimentalExternal),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyStrategyRegistration {
    pub strategy_profile: PolicyStrategyProfile,
    pub source_language: SourceLanguage,
    pub semantic_profile: SemanticProfile,
    pub axiom_profile: PolicyAxiomProfile,
}

impl PolicyStrategyRegistration {
    pub const fn source_language_name(self) -> &'static str {
        match self.source_language {
            SourceLanguage::Go => "go",
            SourceLanguage::Rust => "rust",
        }
    }

    pub const fn semantic_profile_name(self) -> &'static str {
        match self.semantic_profile {
            SemanticProfile::GoFixedV0 => "mpk.go.fixed.v0",
            SemanticProfile::RustCheckedV0 => "mpk.rust.checked.v0",
        }
    }
}

pub const POLICY_STRATEGY_REGISTRY: [PolicyStrategyRegistration; 2] = [
    PolicyStrategyProfile::PaymentPolicyAlpha.registration(),
    PolicyStrategyProfile::PaymentPolicyRustAlpha.registration(),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReadiness {
    Ready,
    Unsupported,
    SourceError,
    FrontendError,
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
                "unknown policy strategy profile {profile:?}; expected one of: {PAYMENT_POLICY_ALPHA_PROFILE}, {PAYMENT_POLICY_RUST_ALPHA_PROFILE}"
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
