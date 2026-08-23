//! Shared policy-profile registry and package/release profile gate.
//!
//! Strategy, checker, semantic, and axiom profiles are independent typed
//! selections. A strategy row binds only the exact language, semantic, and
//! axiom tuple registered for that strategy.

use std::collections::BTreeSet;
use std::fmt;

use mpk_api::{
    PolicyAxiomProfile, PolicyStrategyProfile, PolicyStrategyRegistration, ProofProfile,
    POLICY_STRATEGY_REGISTRY,
};
use mpk_cert::encode::AxiomCategory;
use mpk_vc::{SemanticProfile, SourceLanguage};

pub const POLICY_CHECKER_REGISTRY: [ProofProfile; 3] = [
    ProofProfile::CoreBootstrap,
    ProofProfile::MvpStructural,
    ProofProfile::MvpStrict,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyProfileSelection<'a> {
    pub strategy_profile: &'a str,
    pub checker_profile: &'a str,
    pub source_language: &'a str,
    pub semantic_profile: &'a str,
    pub axiom_profile: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedPolicyProfileSelection {
    pub strategy_profile: PolicyStrategyProfile,
    pub checker_profile: ProofProfile,
    pub source_language: SourceLanguage,
    pub semantic_profile: SemanticProfile,
    pub axiom_profile: PolicyAxiomProfile,
}

impl ValidatedPolicyProfileSelection {
    pub const fn registration(self) -> PolicyStrategyRegistration {
        self.strategy_profile.registration()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyProfileRecognition {
    Recognized(ValidatedPolicyProfileSelection),
    UnrecognizedStrategy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyProfileErrorKind {
    Unknown,
    CrossedTuple,
    PackageMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyProfileField {
    StrategyProfile,
    CheckerProfile,
    SourceLanguage,
    SemanticProfile,
    AxiomProfile,
    PackagePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyProfileError {
    kind: PolicyProfileErrorKind,
    field: PolicyProfileField,
    detail: String,
}

impl PolicyProfileError {
    pub const fn kind(&self) -> PolicyProfileErrorKind {
        self.kind
    }

    pub const fn field(&self) -> PolicyProfileField {
        self.field
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn unknown(field: PolicyProfileField, value: &str) -> Self {
        Self {
            kind: PolicyProfileErrorKind::Unknown,
            field,
            detail: format!("unregistered {} {value:?}", field.canonical_name()),
        }
    }

    fn crossed() -> Self {
        Self {
            kind: PolicyProfileErrorKind::CrossedTuple,
            field: PolicyProfileField::StrategyProfile,
            detail: "strategy, language, semantic profile, and axiom profile form a crossed tuple"
                .to_owned(),
        }
    }

    fn package(detail: impl Into<String>) -> Self {
        Self {
            kind: PolicyProfileErrorKind::PackageMismatch,
            field: PolicyProfileField::PackagePolicy,
            detail: detail.into(),
        }
    }
}

impl PolicyProfileField {
    const fn canonical_name(self) -> &'static str {
        match self {
            Self::StrategyProfile => "strategy profile",
            Self::CheckerProfile => "checker profile",
            Self::SourceLanguage => "source language",
            Self::SemanticProfile => "semantic profile",
            Self::AxiomProfile => "axiom profile",
            Self::PackagePolicy => "package policy",
        }
    }
}

impl fmt::Display for PolicyProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for PolicyProfileError {}

pub fn strategy_registry() -> &'static [PolicyStrategyRegistration] {
    &POLICY_STRATEGY_REGISTRY
}

pub fn lookup_strategy_registration(strategy_profile: &str) -> Option<PolicyStrategyRegistration> {
    let strategy_profile = strategy_profile.parse::<PolicyStrategyProfile>().ok()?;
    Some(strategy_profile.registration())
}

pub fn validate_policy_profile_selection(
    selection: PolicyProfileSelection<'_>,
) -> Result<ValidatedPolicyProfileSelection, PolicyProfileError> {
    let strategy_profile = selection
        .strategy_profile
        .parse::<PolicyStrategyProfile>()
        .map_err(|_| {
            PolicyProfileError::unknown(
                PolicyProfileField::StrategyProfile,
                selection.strategy_profile,
            )
        })?;
    let checker_profile = parse_checker_profile(selection.checker_profile)?;
    let source_language = parse_source_language(selection.source_language)?;
    let semantic_profile = parse_semantic_profile(selection.semantic_profile)?;
    let axiom_profile = parse_evidence_axiom_profile(selection.axiom_profile)?;

    let registration = strategy_profile.registration();
    if registration.source_language != source_language
        || registration.semantic_profile != semantic_profile
        || registration.axiom_profile != axiom_profile
    {
        return Err(PolicyProfileError::crossed());
    }

    Ok(ValidatedPolicyProfileSelection {
        strategy_profile,
        checker_profile,
        source_language,
        semantic_profile,
        axiom_profile,
    })
}

pub fn validate_explainer_profile_selection(
    selection: PolicyProfileSelection<'_>,
    allow_unrecognized_strategy: bool,
) -> Result<PolicyProfileRecognition, PolicyProfileError> {
    if lookup_strategy_registration(selection.strategy_profile).is_some() {
        return validate_policy_profile_selection(selection)
            .map(PolicyProfileRecognition::Recognized);
    }
    if !allow_unrecognized_strategy {
        return Err(PolicyProfileError::unknown(
            PolicyProfileField::StrategyProfile,
            selection.strategy_profile,
        ));
    }

    parse_checker_profile(selection.checker_profile)?;
    let source_language = parse_source_language(selection.source_language)?;
    let semantic_profile = parse_semantic_profile(selection.semantic_profile)?;
    parse_evidence_axiom_profile(selection.axiom_profile)?;
    if semantic_profile.source_language() != source_language {
        return Err(PolicyProfileError::crossed());
    }
    Ok(PolicyProfileRecognition::UnrecognizedStrategy)
}

pub fn validate_package_release_profiles(
    evidence: PolicyProfileSelection<'_>,
    package_checker_profile: &str,
    package_allowed_axiom_profiles: &[String],
    active: PolicyProfileSelection<'_>,
) -> Result<ValidatedPolicyProfileSelection, PolicyProfileError> {
    let evidence = validate_policy_profile_selection(evidence)?;
    let active = validate_policy_profile_selection(active)?;
    let package_checker = parse_checker_profile(package_checker_profile)?;
    let package_axioms = validate_package_axiom_profiles(package_allowed_axiom_profiles)?;

    if active != evidence
        || package_checker != evidence.checker_profile
        || !package_axioms.contains(&evidence.axiom_profile)
    {
        return Err(PolicyProfileError::package(
            "active release, package, and evidence profiles differ or are not permitted",
        ));
    }
    Ok(evidence)
}

pub fn validate_package_axiom_profiles(
    profiles: &[String],
) -> Result<BTreeSet<PolicyAxiomProfile>, PolicyProfileError> {
    if profiles.is_empty() {
        return Err(PolicyProfileError::package(
            "package axiom profile allowlist must not be empty",
        ));
    }
    let mut validated = BTreeSet::new();
    for profile in profiles {
        let parsed = PolicyAxiomProfile::parse_registered(profile).ok_or_else(|| {
            PolicyProfileError::unknown(PolicyProfileField::AxiomProfile, profile)
        })?;
        if !validated.insert(parsed) {
            return Err(PolicyProfileError::package(
                "package axiom profile allowlist contains a duplicate",
            ));
        }
    }
    Ok(validated)
}

fn parse_checker_profile(value: &str) -> Result<ProofProfile, PolicyProfileError> {
    POLICY_CHECKER_REGISTRY
        .into_iter()
        .find(|profile| profile.canonical_name() == value)
        .ok_or_else(|| PolicyProfileError::unknown(PolicyProfileField::CheckerProfile, value))
}

fn parse_source_language(value: &str) -> Result<SourceLanguage, PolicyProfileError> {
    match value {
        "go" => Ok(SourceLanguage::Go),
        "rust" => Ok(SourceLanguage::Rust),
        _ => Err(PolicyProfileError::unknown(
            PolicyProfileField::SourceLanguage,
            value,
        )),
    }
}

fn parse_semantic_profile(value: &str) -> Result<SemanticProfile, PolicyProfileError> {
    match value {
        "mpk.go.fixed.v0" => Ok(SemanticProfile::GoFixedV0),
        "mpk.rust.checked.v0" => Ok(SemanticProfile::RustCheckedV0),
        _ => Err(PolicyProfileError::unknown(
            PolicyProfileField::SemanticProfile,
            value,
        )),
    }
}

fn parse_evidence_axiom_profile(value: &str) -> Result<PolicyAxiomProfile, PolicyProfileError> {
    match value {
        "zero-axiom" => Ok(PolicyAxiomProfile::ZeroAxiom),
        "mvp-theory" => Ok(PolicyAxiomProfile::MvpTheory),
        _ => Err(PolicyProfileError::unknown(
            PolicyProfileField::AxiomProfile,
            value,
        )),
    }
}

/// The full identity required before an axiom can be release-approved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedAxiomIdentity {
    pub category: AxiomCategory,
    pub name: &'static str,
    pub origin_module: &'static str,
    pub type_hash: &'static str,
    pub declaration_hash: &'static str,
    pub export_hash: &'static str,
    pub certificate_hash: Option<&'static str>,
}

/// No concrete axiom identity is currently release-approved for `mvp-theory`.
/// Checked theory certificates remain available because they are proof evidence,
/// not axiom identities.
pub const MVP_THEORY_APPROVED_AXIOMS: &[ApprovedAxiomIdentity] = &[];

pub const fn approved_axioms(profile: PolicyAxiomProfile) -> &'static [ApprovedAxiomIdentity] {
    match profile {
        PolicyAxiomProfile::MvpTheory => MVP_THEORY_APPROVED_AXIOMS,
        PolicyAxiomProfile::ZeroAxiom
        | PolicyAxiomProfile::CoreMvp
        | PolicyAxiomProfile::GoArtifactAlpha
        | PolicyAxiomProfile::ExperimentalExternal => &[],
    }
}

/// A summary-only report can authorize only the empty observed set. Any
/// nonempty report needs full identities to be checked against the concrete
/// allowlist; category counts can never supply that proof.
pub const fn summary_only_axiom_report_is_permitted(
    _profile: PolicyAxiomProfile,
    total_axiom_count: i64,
) -> bool {
    total_axiom_count == 0
}
