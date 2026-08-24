use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, scan_strict_json, serialize_json_bounded,
    BoundedJsonSerializeError, CanonicalJsonError, ComponentIdentity, FrontendIdentity,
    ReleaseRegistryIdentity, StrictJsonError, StrictJsonEvent, StrictJsonLimits,
    StrictJsonPathSegment, StrictJsonValue, ToolchainIdentity,
};
use serde::{Deserialize, Serialize};

use crate::policy_profile::{
    validate_policy_profile_selection, PolicyProfileErrorKind, PolicyProfileSelection,
};

pub const POLICY_SCAN_V1_SCHEMA: &str = "mpk.policy.scan.v1";
pub const POLICY_EVIDENCE_V1_SCHEMA: &str = "mpk.policy.evidence.v1";
pub const POLICY_JSON_TRANSPORT_BYTES_MAX: u64 = 268_435_456;
pub const POLICY_MARKDOWN_BYTES_MAX: u64 = 268_435_456;
pub const POLICY_JSON_NESTING_MAX: u64 = 256;
pub const POLICY_STRING_BYTES_MAX: u64 = 1_048_576;
pub const POLICY_COLLECTION_ELEMENTS_MAX: u64 = 262_144;
pub const POLICY_HELPER_ARTIFACTS_MAX: u64 = 65_536;
pub const POLICY_CERTIFICATES_MAX: u64 = 1;
pub const POLICY_REFERENCES_PER_MEMBER_MAX: u64 = 4_096;
pub const POLICY_RECIPE_ARGV_ELEMENTS_MAX: u64 = 65_536;

const STRICT_POLICY_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    POLICY_JSON_TRANSPORT_BYTES_MAX,
    67_108_865,
    POLICY_JSON_NESTING_MAX,
    POLICY_STRING_BYTES_MAX,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyValidationPhase {
    Transport,
    Shape,
    Scalar,
    Order,
    Profile,
    Release,
    SourceLinkage,
    ManifestLifecycle,
    VcLinkage,
    Helpers,
    Trusted,
    Properties,
    Dependencies,
    Recipes,
    CanonicalSize,
    CanonicalTransport,
    Report,
}

impl PolicyValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Order => "order",
            Self::Profile => "profile",
            Self::Release => "release",
            Self::SourceLinkage => "source_linkage",
            Self::ManifestLifecycle => "manifest_lifecycle",
            Self::VcLinkage => "vc_linkage",
            Self::Helpers => "helpers",
            Self::Trusted => "trusted",
            Self::Properties => "properties",
            Self::Dependencies => "dependencies",
            Self::Recipes => "recipes",
            Self::CanonicalSize => "canonical_size",
            Self::CanonicalTransport => "canonical_transport",
            Self::Report => "report",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyValidationError {
    phase: PolicyValidationPhase,
    code: &'static str,
    detail: String,
}

impl PolicyValidationError {
    fn new(phase: PolicyValidationPhase, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            phase,
            code,
            detail: detail.into(),
        }
    }

    pub const fn phase(&self) -> PolicyValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for PolicyValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} ({}): {}",
            self.code,
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for PolicyValidationError {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PolicySemanticParameters {
    Go(PolicyGoSemanticParameters),
    Rust(PolicyRustSemanticParameters),
}

impl PolicySemanticParameters {
    pub fn target_id(&self) -> &str {
        match self {
            Self::Go(value) => &value.target_id,
            Self::Rust(value) => &value.target_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyGoSemanticParameters {
    pub target_id: String,
    pub pointer_width: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRustSemanticParameters {
    pub target_id: String,
    pub pointer_width: i64,
    pub overflow_mode: String,
    pub panic_mode: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PolicySelection {
    Go(PolicyGoSelection),
    Rust(PolicyRustSelection),
}

impl PolicySelection {
    pub fn package(&self) -> &str {
        match self {
            Self::Go(value) => &value.package,
            Self::Rust(value) => &value.package,
        }
    }

    pub fn function(&self) -> &str {
        match self {
            Self::Go(value) => &value.function,
            Self::Rust(value) => &value.function,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyGoSelection {
    pub package: String,
    pub function: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRustSelection {
    pub package: String,
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub kind: String,
    pub function: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySourceSpan {
    pub normalized_path: String,
    pub start: i64,
    pub end: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyIssue {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<PolicySourceSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolicyHelperArtifact {
    Source {
        id: String,
        normalized_path: String,
        sha256: String,
    },
    Contract {
        id: String,
        normalized_path: String,
        schema: String,
        raw_input_sha256: String,
        function_id: String,
        contract_hash: String,
    },
    VerificationIr {
        id: String,
        schema: String,
        sha256: String,
    },
    Vc {
        id: String,
        schema: String,
        sha256: String,
    },
    AiAnalysis {
        id: String,
        schema: String,
        sha256: String,
    },
    CiStatus {
        id: String,
        system: String,
        check: String,
        status: String,
        subject_sha256: String,
    },
}

impl PolicyHelperArtifact {
    pub fn id(&self) -> &str {
        match self {
            Self::Source { id, .. }
            | Self::Contract { id, .. }
            | Self::VerificationIr { id, .. }
            | Self::Vc { id, .. }
            | Self::AiAnalysis { id, .. }
            | Self::CiStatus { id, .. } => id,
        }
    }

    fn kind_rank(&self) -> u8 {
        match self {
            Self::Source { .. } => 0,
            Self::Contract { .. } => 1,
            Self::VerificationIr { .. } => 2,
            Self::Vc { .. } => 3,
            Self::AiAnalysis { .. } => 4,
            Self::CiStatus { .. } => 5,
        }
    }

    fn contract_path(&self) -> Option<&str> {
        match self {
            Self::Contract {
                normalized_path, ..
            } => Some(normalized_path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanV1 {
    pub schema: String,
    pub frontend_status: String,
    pub frontend_phase: String,
    pub source_language: String,
    pub semantic_profile: String,
    pub semantic_parameters: PolicySemanticParameters,
    pub selection: PolicySelection,
    pub release_registry: ReleaseRegistryIdentity,
    pub frontend: FrontendIdentity,
    pub toolchain: ToolchainIdentity,
    pub readiness: String,
    pub rejected_features: Vec<PolicyIssue>,
    pub diagnostics: Vec<PolicyIssue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_source_manifest_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_set_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ir_schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ir_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helper_artifacts: Option<Vec<PolicyHelperArtifact>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyScanLinkageContext {
    pub frontend_status: String,
    pub frontend_phase: String,
    pub source_language: String,
    pub semantic_profile: String,
    pub semantic_parameters: PolicySemanticParameters,
    pub selection: PolicySelection,
    pub release_registry: ReleaseRegistryIdentity,
    pub frontend: FrontendIdentity,
    pub toolchain: ToolchainIdentity,
    pub rejected_features: Vec<PolicyIssue>,
    pub diagnostics: Vec<PolicyIssue>,
    pub limit_profile: Option<String>,
    pub frontend_source_manifest_hash: Option<String>,
    pub input_set_hash: Option<String>,
    pub source_map_hash: Option<String>,
    pub source_ir_schema: Option<String>,
    pub source_ir_hash: Option<String>,
    pub helper_artifacts: Option<Vec<PolicyHelperArtifact>>,
}

#[derive(Clone, Debug)]
pub struct ValidatedPolicyScanV1 {
    document: PolicyScanV1,
    canonical_bytes: Vec<u8>,
}

impl ValidatedPolicyScanV1 {
    pub fn document(&self) -> &PolicyScanV1 {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyVerificationOptions {
    pub strict: bool,
    pub update_fixtures: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDeclarationDependency {
    pub name: String,
    pub declaration_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCheckedDeclaration {
    pub name: String,
    pub declaration_hash: String,
    pub function_id: String,
    pub group_id: String,
    pub group_kind: String,
    pub member_ids: Vec<String>,
    pub dependencies: Vec<PolicyDeclarationDependency>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCertificateEvidenceV1 {
    pub id: String,
    pub module: String,
    pub certificate_hash: String,
    pub export_hash: String,
    pub axiom_report_hash: String,
    pub checked_declarations: Vec<PolicyCheckedDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTheoryCertificateEvidenceV1 {
    pub id: String,
    pub theory: String,
    pub format: String,
    pub theory_certificate_hash: String,
    pub checker_profile: String,
    pub checked_member_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolicyAxiomReportV1 {
    NotGenerated,
    Checked {
        axiom_report_hash: String,
        category_counts: PolicyAxiomCategoryCountsV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAxiomCategoryCountsV1 {
    pub total_axiom_count: i64,
    pub core_axiom_count: i64,
    pub builtin_theory_axiom_count: i64,
    pub go_semantics_axiom_count: i64,
    pub external_axiom_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCheckerVerdictV1 {
    pub checker: String,
    pub checker_profile: String,
    pub verdict: String,
    pub certificate_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTrustedEvidenceV1 {
    pub certificates: Vec<PolicyCertificateEvidenceV1>,
    pub theory_certificates: Vec<PolicyTheoryCertificateEvidenceV1>,
    pub axiom_report: PolicyAxiomReportV1,
    pub checker_verdicts: Vec<PolicyCheckerVerdictV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolicyEvidenceReferenceV1 {
    CheckedDeclaration { certificate_id: String },
    CheckedTheoryCertificate { theory_certificate_id: String },
    HelperArtifact { artifact_id: String },
    UnsupportedFeature { code: String },
}

impl PolicyEvidenceReferenceV1 {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CheckedDeclaration { .. } => "checked_declaration",
            Self::CheckedTheoryCertificate { .. } => "checked_theory_certificate",
            Self::HelperArtifact { .. } => "helper_artifact",
            Self::UnsupportedFeature { .. } => "unsupported_feature",
        }
    }

    fn order_key(&self) -> (u8, &str) {
        match self {
            Self::CheckedDeclaration { certificate_id } => (0, certificate_id),
            Self::CheckedTheoryCertificate {
                theory_certificate_id,
            } => (1, theory_certificate_id),
            Self::HelperArtifact { artifact_id } => (2, artifact_id),
            Self::UnsupportedFeature { code } => (3, code),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyMemberRowV1 {
    pub member_id: String,
    pub function_id: String,
    pub kind: String,
    pub group_id: String,
    pub declaration_name: String,
    pub declaration_hash: String,
    pub status: String,
    pub evidence: Vec<PolicyEvidenceReferenceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyPropertyV1 {
    pub id: String,
    pub description: String,
    pub status: String,
    pub members: Vec<PolicyMemberRowV1>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyReproductionRecipeV1 {
    pub label: String,
    pub working_directory_role: String,
    pub argv: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyEvidenceV1 {
    pub schema: String,
    pub source_language: String,
    pub semantic_profile: String,
    pub semantic_parameters: PolicySemanticParameters,
    pub selection: PolicySelection,
    pub limit_profile: String,
    pub release_registry: ReleaseRegistryIdentity,
    pub frontend: FrontendIdentity,
    pub toolchain: ToolchainIdentity,
    pub frontend_source_manifest_hash: String,
    pub input_set_hash: String,
    pub source_map_hash: String,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub certificate_source_manifest_hash: String,
    pub source_vc_schema: String,
    pub vc_hash: String,
    pub verification_limit_profile: String,
    pub strategy_profile: String,
    pub checker_profile: String,
    pub axiom_profile: String,
    pub verification_options: PolicyVerificationOptions,
    pub helper_artifacts: Vec<PolicyHelperArtifact>,
    pub trusted_evidence: PolicyTrustedEvidenceV1,
    pub properties: Vec<PolicyPropertyV1>,
    pub reproduction_recipes: Vec<PolicyReproductionRecipeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyExpectedMemberV1 {
    pub member_id: String,
    pub function_id: String,
    pub kind: String,
    pub group_id: String,
    pub declaration_name: String,
    pub declaration_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyExpectedCertificateV1 {
    pub module: String,
    pub certificate_hash: String,
    pub export_hash: String,
    pub axiom_report_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyExpectedPropertyV1 {
    pub id: String,
    pub description: String,
    pub member_ids: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PolicyEvidenceLinkageContext<'a> {
    pub scan: &'a ValidatedPolicyScanV1,
    pub certificate_source_manifest_hash: String,
    pub source_vc_schema: String,
    pub vc_hash: String,
    pub verification_limit_profile: String,
    pub expected_members: Vec<PolicyExpectedMemberV1>,
    pub expected_declarations: Vec<PolicyCheckedDeclaration>,
    pub expected_certificate: Option<PolicyExpectedCertificateV1>,
    pub expected_theory_certificates: Vec<PolicyTheoryCertificateEvidenceV1>,
    pub expected_axiom_report: PolicyAxiomReportV1,
    pub expected_checker_verdicts: Vec<PolicyCheckerVerdictV1>,
    pub expected_properties: Vec<PolicyExpectedPropertyV1>,
    pub expected_unsupported_codes: Vec<String>,
    pub expected_optional_helpers: Vec<PolicyHelperArtifact>,
}

#[derive(Clone, Debug)]
pub struct ValidatedPolicyEvidenceV1 {
    document: PolicyEvidenceV1,
    canonical_bytes: Vec<u8>,
}

impl ValidatedPolicyEvidenceV1 {
    pub fn document(&self) -> &PolicyEvidenceV1 {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

pub fn import_policy_scan_v1_json(
    input: &[u8],
    context: &PolicyScanLinkageContext,
) -> Result<ValidatedPolicyScanV1, PolicyValidationError> {
    let (strict, canonical_without_lf) = parse_policy_transport(input)?;
    require_schema(&strict, POLICY_SCAN_V1_SCHEMA, "POLICY_SCAN_SCHEMA")?;
    reject_null(&strict)?;
    let document: PolicyScanV1 =
        serde_json::from_slice(&canonical_without_lf).map_err(|error| {
            PolicyValidationError::new(
                PolicyValidationPhase::Shape,
                "POLICY_SHAPE",
                error.to_string(),
            )
        })?;

    validate_scan_collection_limits(&document)?;
    validate_scan_shape(&document)?;
    validate_scan_scalars(&document)?;
    validate_scan_order(&document)?;
    validate_language_profile(
        &document.source_language,
        &document.semantic_profile,
        &document.semantic_parameters,
        &document.selection,
    )?;
    validate_scan_release(&document, context)?;
    validate_scan_source_linkage(&document, context)?;
    validate_canonical_size(canonical_transport_size(&canonical_without_lf)?)?;
    validate_canonical_transport(input, canonical_without_lf)?;

    Ok(ValidatedPolicyScanV1 {
        document,
        canonical_bytes: input.to_vec(),
    })
}

pub fn import_policy_evidence_v1_json(
    input: &[u8],
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<ValidatedPolicyEvidenceV1, PolicyValidationError> {
    let (strict, canonical_without_lf) = parse_policy_transport(input)?;
    require_schema(&strict, POLICY_EVIDENCE_V1_SCHEMA, "POLICY_EVIDENCE_SCHEMA")?;
    reject_null(&strict)?;
    let document: PolicyEvidenceV1 =
        serde_json::from_slice(&canonical_without_lf).map_err(|error| {
            PolicyValidationError::new(
                PolicyValidationPhase::Shape,
                "POLICY_SHAPE",
                error.to_string(),
            )
        })?;

    validate_evidence_collection_limits(&document)?;
    validate_evidence_shape(&document)?;
    validate_evidence_scalars(&document)?;
    validate_evidence_order(&document)?;
    validate_evidence_profiles(&document)?;
    validate_evidence_release(&document, context)?;
    validate_evidence_source_linkage(&document, context)?;
    if document.certificate_source_manifest_hash != context.certificate_source_manifest_hash {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::ManifestLifecycle,
            "POLICY_MANIFEST_LIFECYCLE",
            "certificate-stage source manifest identity differs from the retained lifecycle",
        ));
    }
    validate_evidence_vc(&document, context)?;
    validate_evidence_helpers(&document, context)?;
    validate_trusted_evidence(&document, context)?;
    validate_properties(&document, context)?;
    validate_dependencies(&document, context)?;
    validate_recipes(&document)?;
    validate_canonical_size(canonical_transport_size(&canonical_without_lf)?)?;
    validate_canonical_transport(input, canonical_without_lf)?;

    Ok(ValidatedPolicyEvidenceV1 {
        document,
        canonical_bytes: input.to_vec(),
    })
}

/// Imports a standalone evidence report at a consumer boundary that has no
/// access to the source-side linkage context used while producing it.
///
/// This applies every validation that can be recomputed from the evidence
/// itself, including helper, trusted-evidence, member/status, and dependency
/// links. Rechecking the named external artifact bytes remains the
/// responsibility of the policy verifier that created the report.
pub fn import_policy_evidence_v1_for_consumer(
    input: &[u8],
) -> Result<ValidatedPolicyEvidenceV1, PolicyValidationError> {
    let (strict, canonical_without_lf) = parse_policy_transport(input)?;
    require_schema(&strict, POLICY_EVIDENCE_V1_SCHEMA, "POLICY_EVIDENCE_SCHEMA")?;
    reject_null(&strict)?;
    let document: PolicyEvidenceV1 =
        serde_json::from_slice(&canonical_without_lf).map_err(|error| {
            PolicyValidationError::new(
                PolicyValidationPhase::Shape,
                "POLICY_SHAPE",
                error.to_string(),
            )
        })?;

    validate_evidence_collection_limits(&document)?;
    validate_evidence_shape(&document)?;
    validate_evidence_scalars(&document)?;
    validate_evidence_order(&document)?;
    validate_evidence_profiles(&document)?;
    validate_consumer_evidence_linkage(&document)?;
    validate_recipes(&document)?;
    validate_canonical_size(canonical_transport_size(&canonical_without_lf)?)?;
    validate_canonical_transport(input, canonical_without_lf)?;

    Ok(ValidatedPolicyEvidenceV1 {
        document,
        canonical_bytes: input.to_vec(),
    })
}

fn validate_consumer_evidence_linkage(
    document: &PolicyEvidenceV1,
) -> Result<(), PolicyValidationError> {
    if document.release_registry.schema != "mpk.release.bundle_registry.v0"
        || document.release_registry.id != "mpk.release.registry.v0"
        || document.limit_profile != "mpk.vir.limits.v0"
        || document.source_ir_schema != "mpk.vir.v0"
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Release,
            "POLICY_RELEASE_LINKAGE",
            "standalone evidence does not name the active release and VIR profiles",
        ));
    }

    let scan_helpers = document
        .helper_artifacts
        .iter()
        .filter(|helper| {
            matches!(
                helper,
                PolicyHelperArtifact::Source { .. }
                    | PolicyHelperArtifact::Contract { .. }
                    | PolicyHelperArtifact::VerificationIr { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let optional_helpers = document
        .helper_artifacts
        .iter()
        .filter(|helper| {
            matches!(
                helper,
                PolicyHelperArtifact::AiAnalysis { .. } | PolicyHelperArtifact::CiStatus { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let scan = ValidatedPolicyScanV1 {
        document: PolicyScanV1 {
            schema: POLICY_SCAN_V1_SCHEMA.to_owned(),
            frontend_status: "ir-lowered".to_owned(),
            frontend_phase: "complete".to_owned(),
            source_language: document.source_language.clone(),
            semantic_profile: document.semantic_profile.clone(),
            semantic_parameters: document.semantic_parameters.clone(),
            selection: document.selection.clone(),
            release_registry: document.release_registry.clone(),
            frontend: document.frontend.clone(),
            toolchain: document.toolchain.clone(),
            readiness: "ready".to_owned(),
            rejected_features: Vec::new(),
            diagnostics: Vec::new(),
            limit_profile: Some(document.limit_profile.clone()),
            frontend_source_manifest_hash: Some(document.frontend_source_manifest_hash.clone()),
            input_set_hash: Some(document.input_set_hash.clone()),
            source_map_hash: Some(document.source_map_hash.clone()),
            source_ir_schema: Some(document.source_ir_schema.clone()),
            source_ir_hash: Some(document.source_ir_hash.clone()),
            helper_artifacts: Some(scan_helpers),
        },
        canonical_bytes: Vec::new(),
    };
    let expected_members = document
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .map(|member| PolicyExpectedMemberV1 {
            member_id: member.member_id.clone(),
            function_id: member.function_id.clone(),
            kind: member.kind.clone(),
            group_id: member.group_id.clone(),
            declaration_name: member.declaration_name.clone(),
            declaration_hash: member.declaration_hash.clone(),
        })
        .collect::<Vec<_>>();
    let expected_properties = document
        .properties
        .iter()
        .map(|property| PolicyExpectedPropertyV1 {
            id: property.id.clone(),
            description: property.description.clone(),
            member_ids: property
                .members
                .iter()
                .map(|member| member.member_id.clone())
                .collect(),
            notes: property.notes.clone(),
        })
        .collect::<Vec<_>>();
    let expected_unsupported_codes = document
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .flat_map(|member| &member.evidence)
        .filter_map(|reference| match reference {
            PolicyEvidenceReferenceV1::UnsupportedFeature { code } => Some(code.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_certificate = document
        .trusted_evidence
        .certificates
        .first()
        .map(|certificate| PolicyExpectedCertificateV1 {
            module: certificate.module.clone(),
            certificate_hash: certificate.certificate_hash.clone(),
            export_hash: certificate.export_hash.clone(),
            axiom_report_hash: certificate.axiom_report_hash.clone(),
        });
    let expected_declarations = document
        .trusted_evidence
        .certificates
        .first()
        .map(|certificate| certificate.checked_declarations.clone())
        .unwrap_or_default();
    let context = PolicyEvidenceLinkageContext {
        scan: &scan,
        certificate_source_manifest_hash: document.certificate_source_manifest_hash.clone(),
        source_vc_schema: document.source_vc_schema.clone(),
        vc_hash: document.vc_hash.clone(),
        verification_limit_profile: document.verification_limit_profile.clone(),
        expected_members,
        expected_declarations,
        expected_certificate,
        expected_theory_certificates: document.trusted_evidence.theory_certificates.clone(),
        expected_axiom_report: document.trusted_evidence.axiom_report.clone(),
        expected_checker_verdicts: document.trusted_evidence.checker_verdicts.clone(),
        expected_properties,
        expected_unsupported_codes,
        expected_optional_helpers: optional_helpers,
    };

    validate_evidence_release(document, &context)?;
    validate_evidence_source_linkage(document, &context)?;
    validate_evidence_vc(document, &context)?;
    validate_evidence_helpers(document, &context)?;
    validate_trusted_evidence(document, &context)?;
    validate_properties(document, &context)?;
    validate_dependencies(document, &context)
}

pub fn canonical_policy_scan_v1_json(
    document: &PolicyScanV1,
) -> Result<Vec<u8>, PolicyValidationError> {
    canonical_policy_document(document)
}

pub fn canonical_policy_evidence_v1_json(
    document: &PolicyEvidenceV1,
) -> Result<Vec<u8>, PolicyValidationError> {
    canonical_policy_document(document)
}

pub fn expected_reproduction_recipes(
    evidence: &PolicyEvidenceV1,
) -> Vec<PolicyReproductionRecipeV1> {
    let mut contracts = evidence
        .helper_artifacts
        .iter()
        .filter_map(PolicyHelperArtifact::contract_path)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    contracts.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));

    let mut prefix = vec![
        "mpk".to_owned(),
        "policy".to_owned(),
        "scan".to_owned(),
        ".".to_owned(),
        "--language".to_owned(),
        evidence.source_language.clone(),
        "--semantic-profile".to_owned(),
        evidence.semantic_profile.clone(),
        "--require-release-registry-id".to_owned(),
        evidence.release_registry.id.clone(),
        "--require-release-registry-sha256".to_owned(),
        evidence.release_registry.registry_sha256.clone(),
        "--frontend-bundle".to_owned(),
        evidence.frontend.bundle_id.clone(),
        "--toolchain-bundle".to_owned(),
        evidence.toolchain.bundle_id.clone(),
        "--target".to_owned(),
        evidence.semantic_parameters.target_id().to_owned(),
        "--package".to_owned(),
        evidence.selection.package().to_owned(),
        "--function".to_owned(),
        evidence.selection.function().to_owned(),
    ];
    for contract in contracts {
        prefix.push("--contract".to_owned());
        prefix.push(contract);
    }

    let mut scan_argv = prefix.clone();
    scan_argv.push("--json-out".to_owned());
    scan_argv.push("mpk-reproduction-scan.json".to_owned());

    prefix[2] = "verify".to_owned();
    prefix.extend([
        "--strategy-profile".to_owned(),
        evidence.strategy_profile.clone(),
        "--checker-profile".to_owned(),
        evidence.checker_profile.clone(),
        "--axiom-profile".to_owned(),
        evidence.axiom_profile.clone(),
        "--evidence-json".to_owned(),
        "mpk-reproduction-evidence.json".to_owned(),
        "--evidence-md".to_owned(),
        "mpk-reproduction-evidence.md".to_owned(),
    ]);
    if evidence.verification_options.strict {
        prefix.push("--strict".to_owned());
    }
    if evidence.verification_options.update_fixtures {
        prefix.push("--update-fixtures".to_owned());
    }

    vec![
        PolicyReproductionRecipeV1 {
            label: "scan".to_owned(),
            working_directory_role: "source_root".to_owned(),
            argv: scan_argv,
        },
        PolicyReproductionRecipeV1 {
            label: "verify".to_owned(),
            working_directory_role: "source_root".to_owned(),
            argv: prefix,
        },
    ]
}

pub fn render_posix_argv(argv: &[String]) -> String {
    argv.iter()
        .map(|argument| render_posix_argument(argument))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn render_posix_argument(argument: &str) -> String {
    if !argument.is_empty()
        && argument.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'@' | b'%' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'-'
                )
        })
    {
        argument.to_owned()
    } else if argument.is_empty() {
        "''".to_owned()
    } else {
        format!("'{}'", argument.replace('\'', "'\\''"))
    }
}

pub fn validate_policy_limit(counter: &str, count: u64) -> Result<(), PolicyValidationError> {
    let (maximum, phase, code) = match counter {
        "json_transport_bytes" => (
            POLICY_JSON_TRANSPORT_BYTES_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_JSON_BYTES",
        ),
        "markdown_bytes" => (
            POLICY_MARKDOWN_BYTES_MAX,
            PolicyValidationPhase::Report,
            "POLICY_LIMIT_MARKDOWN_BYTES",
        ),
        "json_nesting" => (
            POLICY_JSON_NESTING_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_JSON_INVALID",
        ),
        "string_bytes" => (
            POLICY_STRING_BYTES_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_JSON_INVALID",
        ),
        "helper_artifacts" => (
            POLICY_HELPER_ARTIFACTS_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
        ),
        "certificates" => (
            POLICY_CERTIFICATES_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
        ),
        "references_per_member" => (
            POLICY_REFERENCES_PER_MEMBER_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
        ),
        "recipe_argv_elements" => (
            POLICY_RECIPE_ARGV_ELEMENTS_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
        ),
        "array_elements_default"
        | "object_members_default"
        | "checked_declarations"
        | "theory_certificates"
        | "properties"
        | "member_rows" => (
            POLICY_COLLECTION_ELEMENTS_MAX,
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
        ),
        _ => {
            return Err(PolicyValidationError::new(
                PolicyValidationPhase::Transport,
                "POLICY_LIMIT_COLLECTION",
                format!("unknown policy counter {counter:?}"),
            ));
        }
    };
    if count > maximum {
        return Err(PolicyValidationError::new(
            phase,
            code,
            format!("{counter} count {count} exceeds inclusive maximum {maximum}"),
        ));
    }
    Ok(())
}

fn parse_policy_transport(
    input: &[u8],
) -> Result<(StrictJsonValue, Vec<u8>), PolicyValidationError> {
    scan_policy_transport_limits(input)?;
    let strict = parse_strict_json(input, STRICT_POLICY_LIMITS).map_err(map_strict_json_error)?;
    validate_collection_limits(&strict)?;
    validate_policy_field_limits(&strict)?;
    let canonical = canonical_json_bytes_bounded(&strict, policy_json_without_lf_maximum())
        .map_err(|error| map_policy_canonical_error(error, PolicyValidationPhase::Transport))?;
    Ok((strict, canonical))
}

fn scan_policy_transport_limits(input: &[u8]) -> Result<(), PolicyValidationError> {
    let mut member_rows = 0_u64;
    let mut recipe_arguments = 0_u64;
    let mut first_error: Option<StrictJsonError> = None;
    let mut observer = |event: StrictJsonEvent<'_>| -> Result<(), StrictJsonError> {
        let result = (|| {
            match event {
                StrictJsonEvent::ArrayElement { path, count } => {
                    observed_policy_max(
                        "array_elements_default",
                        count,
                        POLICY_COLLECTION_ELEMENTS_MAX,
                    )?;
                    if path_is(path, &["helper_artifacts"]) {
                        observed_policy_max(
                            "helper_artifacts",
                            count,
                            POLICY_HELPER_ARTIFACTS_MAX,
                        )?;
                    } else if path_is(path, &["trusted_evidence", "certificates"]) {
                        observed_policy_max("certificates", count, POLICY_CERTIFICATES_MAX)?;
                    } else if policy_member_rows_path(path) {
                        member_rows = member_rows.checked_add(1).ok_or(
                            StrictJsonError::ObservedCounterOverflow {
                                limit: "member_rows",
                            },
                        )?;
                        observed_policy_max(
                            "member_rows",
                            member_rows,
                            POLICY_COLLECTION_ELEMENTS_MAX,
                        )?;
                    } else if policy_member_references_path(path) {
                        observed_policy_max(
                            "references_per_member",
                            count,
                            POLICY_REFERENCES_PER_MEMBER_MAX,
                        )?;
                    } else if policy_recipe_argv_path(path) {
                        recipe_arguments = recipe_arguments.checked_add(1).ok_or(
                            StrictJsonError::ObservedCounterOverflow {
                                limit: "recipe_argv_elements",
                            },
                        )?;
                        observed_policy_max(
                            "recipe_argv_elements",
                            recipe_arguments,
                            POLICY_RECIPE_ARGV_ELEMENTS_MAX,
                        )?;
                    }
                }
                StrictJsonEvent::ObjectMember { count, .. } => {
                    observed_policy_max(
                        "object_members_default",
                        count,
                        POLICY_COLLECTION_ELEMENTS_MAX,
                    )?;
                }
                _ => {}
            }
            Ok(())
        })();
        if first_error.is_none() {
            first_error = result.err();
        }
        Ok(())
    };
    scan_strict_json(input, STRICT_POLICY_LIMITS, &mut observer).map_err(map_strict_json_error)?;
    if let Some(error) = first_error {
        return Err(map_strict_json_error(error));
    }
    Ok(())
}

fn observed_policy_max(
    limit: &'static str,
    actual: u64,
    maximum: u64,
) -> Result<(), StrictJsonError> {
    if actual > maximum {
        Err(StrictJsonError::ObservedLimitExceeded {
            limit,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}

fn path_is(path: &[StrictJsonPathSegment], keys: &[&str]) -> bool {
    if path.len() != keys.len() {
        return false;
    }
    path.iter().zip(keys).all(
        |(segment, key)| matches!(segment, StrictJsonPathSegment::Key(actual) if actual == key),
    )
}

fn policy_member_rows_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(properties),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(members),
        ] if properties == "properties" && members == "members"
    )
}

fn policy_member_references_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(properties),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(members),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(evidence),
        ] if properties == "properties" && members == "members" && evidence == "evidence"
    )
}

fn policy_recipe_argv_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(recipes),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(argv),
        ] if recipes == "reproduction_recipes" && argv == "argv"
    )
}

fn validate_policy_field_limits(value: &StrictJsonValue) -> Result<(), PolicyValidationError> {
    let Some(root) = value.as_object() else {
        return Ok(());
    };
    if let Some(helpers) =
        strict_object_field(root, "helper_artifacts").and_then(StrictJsonValue::as_array)
    {
        validate_policy_limit("helper_artifacts", helpers.len() as u64)?;
    }
    if let Some(components) = strict_object_field(root, "toolchain")
        .and_then(StrictJsonValue::as_object)
        .and_then(|toolchain| strict_object_field(toolchain, "components"))
        .and_then(StrictJsonValue::as_array)
    {
        validate_inherited_collection_limit("toolchain_components", components.len(), 8_192)?;
    }
    if let Some(subordinates) = strict_object_field(root, "frontend")
        .and_then(StrictJsonValue::as_object)
        .and_then(|frontend| strict_object_field(frontend, "subordinate_binaries"))
        .and_then(StrictJsonValue::as_array)
    {
        let maximum =
            match strict_object_field(root, "source_language").and_then(StrictJsonValue::as_str) {
                Some("go") => 0,
                _ => 1,
            };
        validate_inherited_collection_limit("frontend_subordinates", subordinates.len(), maximum)?;
    }
    let Some(trusted) =
        strict_object_field(root, "trusted_evidence").and_then(StrictJsonValue::as_object)
    else {
        return validate_property_and_recipe_limits(root);
    };
    if let Some(certificates) =
        strict_object_field(trusted, "certificates").and_then(StrictJsonValue::as_array)
    {
        validate_policy_limit("certificates", certificates.len() as u64)?;
        let checked_declarations = certificates
            .iter()
            .filter_map(StrictJsonValue::as_object)
            .filter_map(|certificate| strict_object_field(certificate, "checked_declarations"))
            .filter_map(StrictJsonValue::as_array)
            .map(|declarations| declarations.len() as u64)
            .try_fold(0u64, u64::checked_add)
            .unwrap_or(u64::MAX);
        validate_policy_limit("checked_declarations", checked_declarations)?;
    }
    if let Some(theory_certificates) =
        strict_object_field(trusted, "theory_certificates").and_then(StrictJsonValue::as_array)
    {
        validate_policy_limit("theory_certificates", theory_certificates.len() as u64)?;
    }
    validate_property_and_recipe_limits(root)
}

fn validate_property_and_recipe_limits(
    root: &[(String, StrictJsonValue)],
) -> Result<(), PolicyValidationError> {
    if let Some(properties) =
        strict_object_field(root, "properties").and_then(StrictJsonValue::as_array)
    {
        validate_policy_limit("properties", properties.len() as u64)?;
        let mut member_rows = 0u64;
        for property in properties.iter().filter_map(StrictJsonValue::as_object) {
            let Some(members) =
                strict_object_field(property, "members").and_then(StrictJsonValue::as_array)
            else {
                continue;
            };
            member_rows = member_rows
                .checked_add(u64::try_from(members.len()).unwrap_or(u64::MAX))
                .ok_or_else(|| {
                    PolicyValidationError::new(
                        PolicyValidationPhase::Transport,
                        "POLICY_LIMIT_COLLECTION",
                        "member_rows counter overflow",
                    )
                })?;
            for member in members.iter().filter_map(StrictJsonValue::as_object) {
                if let Some(references) =
                    strict_object_field(member, "evidence").and_then(StrictJsonValue::as_array)
                {
                    validate_policy_limit("references_per_member", references.len() as u64)?;
                }
            }
        }
        validate_policy_limit("member_rows", member_rows)?;
    }
    if let Some(recipes) =
        strict_object_field(root, "reproduction_recipes").and_then(StrictJsonValue::as_array)
    {
        let arguments = recipes
            .iter()
            .filter_map(StrictJsonValue::as_object)
            .filter_map(|recipe| strict_object_field(recipe, "argv"))
            .filter_map(StrictJsonValue::as_array)
            .map(|argv| argv.len() as u64)
            .try_fold(0u64, u64::checked_add)
            .unwrap_or(u64::MAX);
        validate_policy_limit("recipe_argv_elements", arguments)?;
    }
    Ok(())
}

fn strict_object_field<'a>(
    entries: &'a [(String, StrictJsonValue)],
    name: &str,
) -> Option<&'a StrictJsonValue> {
    entries
        .iter()
        .find_map(|(candidate, value)| (candidate == name).then_some(value))
}

fn validate_inherited_collection_limit(
    counter: &str,
    count: usize,
    maximum: usize,
) -> Result<(), PolicyValidationError> {
    if count > maximum {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Transport,
            "POLICY_LIMIT_COLLECTION",
            format!("{counter} count {count} exceeds inclusive maximum {maximum}"),
        ));
    }
    Ok(())
}

fn map_strict_json_error(error: StrictJsonError) -> PolicyValidationError {
    let code = match error {
        StrictJsonError::InputBytesExceeded { .. } => "POLICY_LIMIT_JSON_BYTES",
        StrictJsonError::DuplicateObjectName { .. } => "POLICY_JSON_DUPLICATE_KEY",
        StrictJsonError::ObservedLimitExceeded { .. }
        | StrictJsonError::ObservedCounterOverflow { .. } => "POLICY_LIMIT_COLLECTION",
        _ => "POLICY_JSON_INVALID",
    };
    PolicyValidationError::new(PolicyValidationPhase::Transport, code, error.to_string())
}

fn validate_collection_limits(value: &StrictJsonValue) -> Result<(), PolicyValidationError> {
    match value {
        StrictJsonValue::Array(values) => {
            validate_policy_limit(
                "array_elements_default",
                u64::try_from(values.len()).unwrap_or(u64::MAX),
            )?;
            for value in values {
                validate_collection_limits(value)?;
            }
        }
        StrictJsonValue::Object(entries) => {
            validate_policy_limit(
                "object_members_default",
                u64::try_from(entries.len()).unwrap_or(u64::MAX),
            )?;
            for (_, value) in entries {
                validate_collection_limits(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn require_schema(
    value: &StrictJsonValue,
    expected: &'static str,
    code: &'static str,
) -> Result<(), PolicyValidationError> {
    let schema = value
        .as_object()
        .and_then(|entries| entries.iter().find(|(name, _)| name == "schema"))
        .and_then(|(_, value)| value.as_str());
    if schema != Some(expected) {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Shape,
            code,
            format!("schema {schema:?} does not equal {expected:?}"),
        ));
    }
    Ok(())
}

fn reject_null(value: &StrictJsonValue) -> Result<(), PolicyValidationError> {
    match value {
        StrictJsonValue::Null => Err(PolicyValidationError::new(
            PolicyValidationPhase::Shape,
            "POLICY_SHAPE",
            "null is forbidden in policy documents",
        )),
        StrictJsonValue::Array(values) => {
            for value in values {
                reject_null(value)?;
            }
            Ok(())
        }
        StrictJsonValue::Object(entries) => {
            for (_, value) in entries {
                reject_null(value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_canonical_transport(
    input: &[u8],
    mut canonical_without_lf: Vec<u8>,
) -> Result<(), PolicyValidationError> {
    canonical_without_lf.push(b'\n');
    if input != canonical_without_lf {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::CanonicalTransport,
            "POLICY_CANONICAL_TRANSPORT",
            "transport is not exact JCS followed by one LF",
        ));
    }
    Ok(())
}

fn canonical_policy_document<T: Serialize>(document: &T) -> Result<Vec<u8>, PolicyValidationError> {
    let encoded =
        serialize_json_bounded(document, policy_json_without_lf_maximum()).map_err(|error| {
            match error {
                BoundedJsonSerializeError::OutputBytesExceeded { .. } => {
                    policy_canonical_size_error()
                }
                BoundedJsonSerializeError::Serialize(detail) => {
                    PolicyValidationError::new(PolicyValidationPhase::Shape, "POLICY_SHAPE", detail)
                }
            }
        })?;
    let strict = parse_strict_json(&encoded, STRICT_POLICY_LIMITS).map_err(|error| {
        let code = if matches!(error, StrictJsonError::InputBytesExceeded { .. }) {
            "POLICY_LIMIT_JSON_BYTES"
        } else {
            "POLICY_JSON_INVALID"
        };
        PolicyValidationError::new(
            PolicyValidationPhase::CanonicalSize,
            code,
            error.to_string(),
        )
    })?;
    let mut canonical = canonical_json_bytes_bounded(&strict, policy_json_without_lf_maximum())
        .map_err(|error| map_policy_canonical_error(error, PolicyValidationPhase::CanonicalSize))?;
    canonical.push(b'\n');
    validate_canonical_size(canonical.len())?;
    Ok(canonical)
}

fn policy_json_without_lf_maximum() -> usize {
    usize::try_from(POLICY_JSON_TRANSPORT_BYTES_MAX.saturating_sub(1)).unwrap_or(usize::MAX)
}

fn policy_canonical_size_error() -> PolicyValidationError {
    PolicyValidationError::new(
        PolicyValidationPhase::CanonicalSize,
        "POLICY_LIMIT_JSON_BYTES",
        "canonical policy JSON exceeds the 256 MiB limit",
    )
}

fn map_policy_canonical_error(
    error: CanonicalJsonError,
    phase: PolicyValidationPhase,
) -> PolicyValidationError {
    if matches!(error, CanonicalJsonError::OutputBytesExceeded { .. }) {
        PolicyValidationError::new(
            phase,
            "POLICY_LIMIT_JSON_BYTES",
            "canonical policy JSON exceeds the 256 MiB limit",
        )
    } else {
        PolicyValidationError::new(phase, "POLICY_JSON_INVALID", error.to_string())
    }
}

fn validate_canonical_size(bytes: usize) -> Result<(), PolicyValidationError> {
    if u64::try_from(bytes).unwrap_or(u64::MAX) > POLICY_JSON_TRANSPORT_BYTES_MAX {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::CanonicalSize,
            "POLICY_LIMIT_JSON_BYTES",
            "canonical policy JSON exceeds the 256 MiB limit",
        ));
    }
    Ok(())
}

fn canonical_transport_size(canonical_without_lf: &[u8]) -> Result<usize, PolicyValidationError> {
    canonical_without_lf.len().checked_add(1).ok_or_else(|| {
        PolicyValidationError::new(
            PolicyValidationPhase::CanonicalSize,
            "POLICY_LIMIT_JSON_BYTES",
            "canonical policy JSON byte counter overflow",
        )
    })
}

fn validate_scan_collection_limits(document: &PolicyScanV1) -> Result<(), PolicyValidationError> {
    if let Some(helpers) = &document.helper_artifacts {
        validate_policy_limit("helper_artifacts", helpers.len() as u64)?;
    }
    Ok(())
}

fn validate_evidence_collection_limits(
    document: &PolicyEvidenceV1,
) -> Result<(), PolicyValidationError> {
    validate_policy_limit("helper_artifacts", document.helper_artifacts.len() as u64)?;
    validate_policy_limit(
        "certificates",
        document.trusted_evidence.certificates.len() as u64,
    )?;
    let checked_declarations = document
        .trusted_evidence
        .certificates
        .iter()
        .map(|certificate| certificate.checked_declarations.len() as u64)
        .try_fold(0u64, u64::checked_add)
        .unwrap_or(u64::MAX);
    validate_policy_limit("checked_declarations", checked_declarations)?;
    validate_policy_limit(
        "theory_certificates",
        document.trusted_evidence.theory_certificates.len() as u64,
    )?;
    validate_policy_limit("properties", document.properties.len() as u64)?;
    let member_rows = document
        .properties
        .iter()
        .map(|property| property.members.len() as u64)
        .try_fold(0u64, u64::checked_add)
        .unwrap_or(u64::MAX);
    validate_policy_limit("member_rows", member_rows)?;
    for member in document
        .properties
        .iter()
        .flat_map(|property| &property.members)
    {
        validate_policy_limit("references_per_member", member.evidence.len() as u64)?;
    }
    let recipe_arguments = document
        .reproduction_recipes
        .iter()
        .map(|recipe| recipe.argv.len() as u64)
        .try_fold(0u64, u64::checked_add)
        .unwrap_or(u64::MAX);
    validate_policy_limit("recipe_argv_elements", recipe_arguments)?;
    Ok(())
}

fn validate_scan_shape(document: &PolicyScanV1) -> Result<(), PolicyValidationError> {
    validate_release_component_shape(&document.source_language, &document.toolchain)?;
    let (expected_readiness, success, phases, rejected_required, diagnostics_required) =
        match document.frontend_status.as_str() {
            "ir-lowered" => ("ready", true, &["emission"][..], false, false),
            "rejected" => (
                "unsupported",
                false,
                &[
                    "capture", "source", "metadata", "subset", "lowering", "emission",
                ][..],
                true,
                false,
            ),
            "source-error" => (
                "source_error",
                false,
                &["capture", "source", "metadata", "typecheck"][..],
                false,
                true,
            ),
            "frontend-error" => (
                "frontend_error",
                false,
                &[
                    "capture",
                    "source",
                    "metadata",
                    "typecheck",
                    "subset",
                    "lowering",
                    "emission",
                ][..],
                false,
                true,
            ),
            _ => return shape("unknown frontend_status"),
        };
    if document.readiness != expected_readiness
        || !phases.contains(&document.frontend_phase.as_str())
    {
        return shape("frontend status, phase, and readiness are incompatible");
    }
    if document.frontend_status == "ir-lowered" && !document.rejected_features.is_empty() {
        return shape("ir-lowered cannot contain rejected features");
    }
    if rejected_required && document.rejected_features.is_empty() && document.diagnostics.is_empty()
    {
        return shape("rejected status requires an issue");
    }
    if document.frontend_status == "source-error" && !document.rejected_features.is_empty() {
        return shape("source-error cannot contain rejected features");
    }
    if document.frontend_status == "frontend-error" && !document.rejected_features.is_empty() {
        return shape("frontend-error cannot contain rejected features");
    }
    if diagnostics_required && document.diagnostics.is_empty() {
        return shape("status requires a diagnostic");
    }

    let success_presence = [
        document.limit_profile.is_some(),
        document.frontend_source_manifest_hash.is_some(),
        document.input_set_hash.is_some(),
        document.source_map_hash.is_some(),
        document.source_ir_schema.is_some(),
        document.source_ir_hash.is_some(),
        document.helper_artifacts.is_some(),
    ];
    if success_presence.iter().any(|present| *present != success) {
        return shape("success-only scan fields have the wrong presence");
    }
    Ok(())
}

fn validate_evidence_shape(document: &PolicyEvidenceV1) -> Result<(), PolicyValidationError> {
    validate_release_component_shape(&document.source_language, &document.toolchain)?;
    if document.properties.is_empty() {
        return shape("properties must be nonempty");
    }
    if document.reproduction_recipes.len() != 2 {
        return shape("evidence has exactly two reproduction recipes");
    }
    if document.trusted_evidence.checker_verdicts.len() != 2 {
        return shape("trusted evidence has exactly two checker verdicts");
    }
    if document.trusted_evidence.certificates.len() > 1 {
        return shape("certificates is empty or a singleton");
    }
    for property in &document.properties {
        if property.members.is_empty() {
            return shape("every property has at least one member");
        }
    }
    Ok(())
}

fn validate_release_component_shape(
    source_language: &str,
    toolchain: &ToolchainIdentity,
) -> Result<(), PolicyValidationError> {
    for component in &toolchain.components {
        if let ComponentIdentity::Executable {
            name, commit_hash, ..
        } = component
        {
            let valid = match source_language {
                "rust" if name == "rustc" => commit_hash.is_some(),
                "go" | "rust" => commit_hash.is_none(),
                _ => true,
            };
            if !valid {
                return shape("commit_hash presence is invalid for the toolchain component");
            }
        }
    }
    Ok(())
}

fn shape<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Shape,
        "POLICY_SHAPE",
        detail,
    ))
}

fn validate_scan_scalars(document: &PolicyScanV1) -> Result<(), PolicyValidationError> {
    validate_common_scalars(
        &document.source_language,
        &document.semantic_profile,
        &document.semantic_parameters,
        &document.selection,
        &document.release_registry,
        &document.frontend,
        &document.toolchain,
    )?;
    validate_issues(&document.rejected_features)?;
    validate_issues(&document.diagnostics)?;
    let issue_count = document
        .rejected_features
        .len()
        .checked_add(document.diagnostics.len())
        .ok_or_else(|| scalar_error("combined frontend issue counter overflow"))?;
    let message_bytes = document
        .rejected_features
        .iter()
        .chain(&document.diagnostics)
        .try_fold(0usize, |total, issue| {
            total.checked_add(issue.message.len())
        })
        .ok_or_else(|| scalar_error("combined frontend message byte counter overflow"))?;
    if issue_count > 1_024 || message_bytes > 2_097_152 {
        return scalar("combined frontend issue budget exceeded");
    }
    if let Some(value) = &document.frontend_source_manifest_hash {
        require_sha256(value)?;
    }
    if let Some(value) = &document.input_set_hash {
        require_sha256(value)?;
    }
    if let Some(value) = &document.source_map_hash {
        require_sha256(value)?;
    }
    if let Some(value) = &document.source_ir_hash {
        require_sha256(value)?;
    }
    if let Some(value) = &document.limit_profile {
        require_profile_id(value)?;
    }
    if let Some(value) = &document.source_ir_schema {
        require_profile_id(value)?;
    }
    if let Some(helpers) = &document.helper_artifacts {
        validate_helper_scalars(helpers, &document.source_language)?;
    }
    Ok(())
}

fn validate_evidence_scalars(document: &PolicyEvidenceV1) -> Result<(), PolicyValidationError> {
    validate_common_scalars(
        &document.source_language,
        &document.semantic_profile,
        &document.semantic_parameters,
        &document.selection,
        &document.release_registry,
        &document.frontend,
        &document.toolchain,
    )?;
    for hash in [
        &document.frontend_source_manifest_hash,
        &document.input_set_hash,
        &document.source_map_hash,
        &document.source_ir_hash,
        &document.certificate_source_manifest_hash,
        &document.vc_hash,
    ] {
        require_sha256(hash)?;
    }
    require_profile_id(&document.strategy_profile)?;
    require_profile_id(&document.checker_profile)?;
    require_profile_id(&document.axiom_profile)?;
    require_profile_id(&document.limit_profile)?;
    require_profile_id(&document.source_ir_schema)?;
    require_profile_id(&document.source_vc_schema)?;
    require_profile_id(&document.verification_limit_profile)?;
    validate_helper_scalars(&document.helper_artifacts, &document.source_language)?;
    validate_trusted_scalars(&document.trusted_evidence)?;
    for property in &document.properties {
        require_artifact_id(&property.id)?;
        require_policy_text(&property.description)?;
        for note in &property.notes {
            require_policy_text(note)?;
        }
        for member in &property.members {
            require_identity(&member.member_id)?;
            require_identity(&member.function_id)?;
            require_profile_id(&member.kind)?;
            require_identity(&member.group_id)?;
            require_identity(&member.declaration_name)?;
            require_sha256(&member.declaration_hash)?;
            if !matches!(
                member.status.as_str(),
                "mpk_verified" | "proof_pending" | "helper_only" | "unsupported"
            ) {
                return scalar("unknown member status");
            }
            for reference in &member.evidence {
                match reference {
                    PolicyEvidenceReferenceV1::CheckedDeclaration { certificate_id } => {
                        require_artifact_id(certificate_id)?
                    }
                    PolicyEvidenceReferenceV1::CheckedTheoryCertificate {
                        theory_certificate_id,
                    } => require_artifact_id(theory_certificate_id)?,
                    PolicyEvidenceReferenceV1::HelperArtifact { artifact_id } => {
                        require_artifact_id(artifact_id)?
                    }
                    PolicyEvidenceReferenceV1::UnsupportedFeature { code } => {
                        require_issue_code(code)?
                    }
                }
            }
        }
        if !matches!(
            property.status.as_str(),
            "mpk_verified" | "proof_pending" | "helper_only" | "unsupported"
        ) {
            return scalar("unknown property status");
        }
    }
    for recipe in &document.reproduction_recipes {
        require_profile_id(&recipe.label)?;
        require_profile_id(&recipe.working_directory_role)?;
        if recipe
            .argv
            .iter()
            .any(|argument| argument.len() > POLICY_STRING_BYTES_MAX as usize)
        {
            return scalar("recipe argument exceeds string limit");
        }
    }
    Ok(())
}

fn validate_common_scalars(
    source_language: &str,
    semantic_profile: &str,
    semantic_parameters: &PolicySemanticParameters,
    selection: &PolicySelection,
    registry: &ReleaseRegistryIdentity,
    frontend: &FrontendIdentity,
    toolchain: &ToolchainIdentity,
) -> Result<(), PolicyValidationError> {
    if !matches!(source_language, "go" | "rust") {
        return scalar("source_language must be go or rust");
    }
    require_profile_id(semantic_profile)?;
    match semantic_parameters {
        PolicySemanticParameters::Go(parameters) => require_go_target_id(&parameters.target_id)?,
        PolicySemanticParameters::Rust(parameters) => {
            require_rust_target_id(&parameters.target_id)?
        }
    }
    require_identity(selection.package())?;
    require_identity(selection.function())?;
    match selection {
        PolicySelection::Go(selection) => validate_go_selection(selection)?,
        PolicySelection::Rust(selection) => validate_rust_selection(selection)?,
    }
    require_release_identifier(&registry.schema, 128, "registry schema")?;
    require_release_identifier(&registry.id, 128, "registry ID")?;
    require_sha256(&registry.registry_sha256)?;
    require_release_identifier(&frontend.bundle_id, 128, "frontend bundle ID")?;
    require_release_identifier(&frontend.name, 64, "frontend executable name")?;
    require_version(&frontend.version)?;
    require_sha256(&frontend.binary_sha256)?;
    for subordinate in &frontend.subordinate_binaries {
        require_release_identifier(&subordinate.name, 64, "subordinate executable name")?;
        require_version(&subordinate.version)?;
        require_sha256(&subordinate.binary_sha256)?;
    }
    require_release_identifier(&toolchain.bundle_id, 128, "toolchain bundle ID")?;
    require_sha256(&toolchain.distribution_sha256)?;
    for component in &toolchain.components {
        match component {
            ComponentIdentity::Executable {
                name,
                release,
                commit_hash,
                binary_sha256,
            } => {
                require_release_identifier(name, 64, "toolchain executable name")?;
                require_version(release)?;
                if let Some(hash) = commit_hash {
                    if hash.len() != 40
                        || !hash
                            .bytes()
                            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    {
                        return scalar("commit_hash must be 40 lowercase hexadecimal characters");
                    }
                }
                require_sha256(binary_sha256)?;
            }
            ComponentIdentity::Content {
                name,
                release,
                content_sha256,
            } => {
                require_release_identifier(name, 128, "toolchain component name")?;
                require_version(release)?;
                require_sha256(content_sha256)?;
            }
        }
    }
    Ok(())
}

fn validate_go_selection(selection: &PolicyGoSelection) -> Result<(), PolicyValidationError> {
    let valid_unit_segment = |segment: &str| {
        let bytes = segment.as_bytes();
        !bytes.is_empty()
            && !matches!(segment, "." | "..")
            && (bytes[0].is_ascii_alphanumeric() || bytes[0] == b'_')
            && bytes.iter().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-')
            })
    };
    if !selection.package.split('/').all(valid_unit_segment) {
        return scalar("invalid canonical Go package identity");
    }
    let Some(item_path) = selection
        .function
        .strip_prefix(&selection.package)
        .and_then(|suffix| suffix.strip_prefix('.'))
    else {
        return scalar("Go function does not belong to the selected package");
    };
    let items = item_path.split('.').collect::<Vec<_>>();
    if !(1..=2).contains(&items.len())
        || items.iter().any(|item| !valid_ascii_identifier(item, 255))
    {
        return scalar("invalid canonical Go function identity");
    }
    Ok(())
}

fn validate_rust_selection(selection: &PolicyRustSelection) -> Result<(), PolicyValidationError> {
    let mut package = selection.package.bytes();
    if !package
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic())
        || !package.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return scalar("invalid canonical Rust package identity");
    }
    require_ascii_identifier(&selection.crate_name, 255, "Rust crate name")?;
    let mut segments = selection.function.split("::");
    if segments.next() != Some(selection.crate_name.as_str())
        || segments.clone().next().is_none()
        || segments.any(|segment| !valid_ascii_identifier(segment, 255))
    {
        return scalar("invalid canonical Rust function identity");
    }
    Ok(())
}

fn validate_helper_scalars(
    helpers: &[PolicyHelperArtifact],
    source_language: &str,
) -> Result<(), PolicyValidationError> {
    for helper in helpers {
        require_artifact_id(helper.id())?;
        match helper {
            PolicyHelperArtifact::Source {
                normalized_path,
                sha256,
                ..
            } => {
                require_normalized_path(normalized_path)?;
                require_sha256(sha256)?;
                if helper.id() != format!("source:{normalized_path}") {
                    return scalar("source helper ID does not match its normalized path");
                }
            }
            PolicyHelperArtifact::Contract {
                normalized_path,
                schema,
                raw_input_sha256,
                function_id,
                contract_hash,
                ..
            } => {
                require_normalized_path(normalized_path)?;
                require_profile_id(schema)?;
                require_sha256(raw_input_sha256)?;
                require_identity(function_id)?;
                require_sha256(contract_hash)?;
                let expected_schema = match source_language {
                    "go" => "mpk.go.contract.v0",
                    "rust" => "mpk.rust.contract.v0",
                    _ => return scalar("unknown source language for contract helper"),
                };
                if schema != expected_schema || helper.id() != format!("contract:{function_id}") {
                    return scalar("contract helper identity or schema is inconsistent");
                }
            }
            PolicyHelperArtifact::VerificationIr { schema, sha256, .. } => {
                require_profile_id(schema)?;
                require_sha256(sha256)?;
                if helper.id() != "verification_ir" || schema != "mpk.vir.v0" {
                    return scalar("verification IR helper identity or schema is inconsistent");
                }
            }
            PolicyHelperArtifact::Vc { schema, sha256, .. } => {
                require_profile_id(schema)?;
                require_sha256(sha256)?;
                if helper.id() != "vc" || schema != "mpk.vc.v1" {
                    return scalar("VC helper identity or schema is inconsistent");
                }
            }
            PolicyHelperArtifact::AiAnalysis { schema, sha256, .. } => {
                require_profile_id(schema)?;
                require_sha256(sha256)?;
            }
            PolicyHelperArtifact::CiStatus {
                system,
                check,
                status,
                subject_sha256,
                ..
            } => {
                require_profile_id(system)?;
                require_profile_id(check)?;
                if !matches!(status.as_str(), "success" | "failure" | "pending") {
                    return scalar("unknown CI status");
                }
                require_sha256(subject_sha256)?;
            }
        }
    }
    Ok(())
}

fn validate_trusted_scalars(
    trusted: &PolicyTrustedEvidenceV1,
) -> Result<(), PolicyValidationError> {
    for certificate in &trusted.certificates {
        if certificate.id != "program" {
            return scalar("certificate id must be program");
        }
        require_identity(&certificate.module)?;
        require_sha256(&certificate.certificate_hash)?;
        require_sha256(&certificate.export_hash)?;
        require_sha256(&certificate.axiom_report_hash)?;
        for declaration in &certificate.checked_declarations {
            require_identity(&declaration.name)?;
            require_sha256(&declaration.declaration_hash)?;
            require_identity(&declaration.function_id)?;
            require_identity(&declaration.group_id)?;
            if !matches!(declaration.group_kind.as_str(), "contract" | "panic_free") {
                return scalar("unknown declaration group kind");
            }
            for member_id in &declaration.member_ids {
                require_identity(member_id)?;
            }
            for dependency in &declaration.dependencies {
                require_identity(&dependency.name)?;
                require_sha256(&dependency.declaration_hash)?;
            }
        }
    }
    for theory in &trusted.theory_certificates {
        require_artifact_id(&theory.id)?;
        require_profile_id(&theory.theory)?;
        require_profile_id(&theory.format)?;
        require_sha256(&theory.theory_certificate_hash)?;
        require_profile_id(&theory.checker_profile)?;
        if theory.checked_member_ids.is_empty() {
            return scalar("theory certificate member set must be nonempty");
        }
        for member_id in &theory.checked_member_ids {
            require_identity(member_id)?;
        }
    }
    if let PolicyAxiomReportV1::Checked {
        axiom_report_hash,
        category_counts,
    } = &trusted.axiom_report
    {
        require_sha256(axiom_report_hash)?;
        let counts = [
            category_counts.total_axiom_count,
            category_counts.core_axiom_count,
            category_counts.builtin_theory_axiom_count,
            category_counts.go_semantics_axiom_count,
            category_counts.external_axiom_count,
        ];
        if counts.iter().any(|count| *count < 0) {
            return scalar("axiom category counts must be nonnegative");
        }
        let category_sum = i128::from(category_counts.core_axiom_count)
            + i128::from(category_counts.builtin_theory_axiom_count)
            + i128::from(category_counts.go_semantics_axiom_count)
            + i128::from(category_counts.external_axiom_count);
        if i128::from(category_counts.total_axiom_count) != category_sum {
            return scalar("axiom category counts do not sum to total_axiom_count");
        }
    }
    for verdict in &trusted.checker_verdicts {
        if !matches!(
            verdict.checker.as_str(),
            "rust_fast_kernel" | "reference_checker"
        ) {
            return scalar("unknown checker identity");
        }
        require_profile_id(&verdict.checker_profile)?;
        if !matches!(
            verdict.verdict.as_str(),
            "accepted" | "rejected" | "not_run"
        ) {
            return scalar("unknown checker verdict");
        }
        for certificate_id in &verdict.certificate_ids {
            require_artifact_id(certificate_id)?;
        }
    }
    Ok(())
}

fn scalar<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(scalar_error(detail))
}

fn scalar_error(detail: impl Into<String>) -> PolicyValidationError {
    PolicyValidationError::new(PolicyValidationPhase::Scalar, "POLICY_SCALAR", detail)
}

fn require_sha256(value: &str) -> Result<(), PolicyValidationError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        scalar("Sha256 must be 64 lowercase hexadecimal ASCII characters")
    }
}

fn require_profile_id(value: &str) -> Result<(), PolicyValidationError> {
    require_release_identifier(value, 128, "profile identifier")
}

fn require_release_identifier(
    value: &str,
    maximum: usize,
    label: &str,
) -> Result<(), PolicyValidationError> {
    let bytes = value.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= maximum
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        && !bytes.windows(2).any(|pair| {
            matches!(pair[0], b'.' | b'_' | b'-') && matches!(pair[1], b'.' | b'_' | b'-')
        });
    if valid {
        Ok(())
    } else {
        scalar(format!("invalid {label} {value:?}"))
    }
}

fn require_version(value: &str) -> Result<(), PolicyValidationError> {
    let bytes = value.as_bytes();
    if !bytes.is_empty()
        && bytes.len() <= 128
        && !bytes[0].is_ascii_whitespace()
        && !bytes[bytes.len() - 1].is_ascii_whitespace()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
        && !bytes.iter().any(|byte| matches!(byte, b'/' | b'\\'))
        && !contains_machine_path(value)
    {
        Ok(())
    } else {
        scalar(format!("invalid release version {value:?}"))
    }
}

fn require_go_target_id(value: &str) -> Result<(), PolicyValidationError> {
    let mut segments = value.split('/');
    let valid_segment = |segment: &str| {
        !segment.is_empty()
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    };
    if segments.next().is_some_and(valid_segment)
        && segments.next().is_some_and(valid_segment)
        && segments.next().is_none()
    {
        Ok(())
    } else {
        scalar(format!("invalid Go target ID {value:?}"))
    }
}

fn require_rust_target_id(value: &str) -> Result<(), PolicyValidationError> {
    let bytes = value.as_bytes();
    if !bytes.is_empty()
        && bytes.len() <= 255
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && (bytes[bytes.len() - 1].is_ascii_lowercase() || bytes[bytes.len() - 1].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
    {
        Ok(())
    } else {
        scalar(format!("invalid Rust target ID {value:?}"))
    }
}

fn require_ascii_identifier(
    value: &str,
    maximum: usize,
    label: &str,
) -> Result<(), PolicyValidationError> {
    if valid_ascii_identifier(value, maximum) {
        Ok(())
    } else {
        scalar(format!("invalid {label} {value:?}"))
    }
}

fn valid_ascii_identifier(value: &str, maximum: usize) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= maximum
        && (bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        && value != "_"
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
}

fn require_artifact_id(value: &str) -> Result<(), PolicyValidationError> {
    let bytes = value.as_bytes();
    if (1..=1_033).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'~' | b':' | b'#' | b'/' | b'-')
        })
        && !contains_machine_path(value)
    {
        Ok(())
    } else {
        scalar(format!("invalid artifact identifier {value:?}"))
    }
}

fn require_identity(value: &str) -> Result<(), PolicyValidationError> {
    if value.is_empty()
        || value.len() > 1_024
        || value.chars().any(char::is_control)
        || value.chars().any(char::is_whitespace)
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'~' | b':' | b'#' | b'/' | b'-')
        })
        || contains_machine_path(value)
    {
        scalar(format!("invalid public identity {value:?}"))
    } else {
        Ok(())
    }
}

fn require_issue_code(value: &str) -> Result<(), PolicyValidationError> {
    let bytes = value.as_bytes();
    if (1..=128).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        Ok(())
    } else {
        scalar(format!("invalid issue code {value:?}"))
    }
}

fn require_policy_text(value: &str) -> Result<(), PolicyValidationError> {
    if value.is_empty()
        || value.len() > 4_096
        || value.chars().any(char::is_control)
        || contains_machine_path(value)
    {
        scalar("policy text is empty, oversized, contains control text, or leaks a machine path")
    } else {
        Ok(())
    }
}

fn require_normalized_path(value: &str) -> Result<(), PolicyValidationError> {
    mpk_vc::validate_manifest_normalized_path(value).map_err(|error| {
        PolicyValidationError::new(
            PolicyValidationPhase::Scalar,
            "POLICY_SCALAR",
            error.to_string(),
        )
    })
}

fn validate_issues(issues: &[PolicyIssue]) -> Result<(), PolicyValidationError> {
    let total = issues.len();
    if total > 1_024 {
        return scalar("issue count exceeds frontend protocol maximum");
    }
    let mut message_bytes = 0usize;
    for issue in issues {
        require_issue_code(&issue.code)?;
        if issue.message.is_empty()
            || issue.message.len() > 4_096
            || issue.message.chars().any(char::is_control)
            || contains_machine_path(&issue.message)
        {
            return scalar("invalid normalized issue message");
        }
        message_bytes = message_bytes
            .checked_add(issue.message.len())
            .ok_or_else(|| scalar_error("issue message byte counter overflow"))?;
        if let Some(function_id) = &issue.function_id {
            require_identity(function_id)?;
        }
        if let Some(span) = &issue.span {
            require_normalized_path(&span.normalized_path)?;
            if span.start < 0 || span.start >= span.end {
                return scalar("invalid source span range");
            }
        }
    }
    if message_bytes > 2_097_152 {
        return scalar("issue message budget exceeded");
    }
    Ok(())
}

fn contains_machine_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    for (index, _) in value.char_indices() {
        let boundary = index == 0
            || value[..index].chars().next_back().is_some_and(|previous| {
                !previous.is_alphanumeric() && !matches!(previous, '.' | '_' | '-' | '/' | '\\')
            });
        if !boundary {
            continue;
        }
        let tail = &bytes[index..];
        let file_locator = tail
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"file:"));
        let drive = tail.len() >= 3
            && tail[0].is_ascii_alphabetic()
            && tail[1] == b':'
            && matches!(tail[2], b'/' | b'\\');
        if tail.starts_with(b"/") || tail.starts_with(b"\\\\") || file_locator || drive {
            return true;
        }
    }
    false
}

fn validate_scan_order(document: &PolicyScanV1) -> Result<(), PolicyValidationError> {
    require_issue_order(&document.rejected_features)?;
    require_issue_order(&document.diagnostics)?;
    require_release_order(&document.frontend, &document.toolchain)?;
    if let Some(helpers) = &document.helper_artifacts {
        require_helper_order(helpers)?;
    }
    Ok(())
}

fn validate_evidence_order(document: &PolicyEvidenceV1) -> Result<(), PolicyValidationError> {
    require_release_order(&document.frontend, &document.toolchain)?;
    require_helper_order(&document.helper_artifacts)?;
    require_strict_order_by(
        &document.trusted_evidence.theory_certificates,
        |left, right| left.id.as_bytes() < right.id.as_bytes(),
        "theory certificate IDs",
    )?;
    for theory in &document.trusted_evidence.theory_certificates {
        require_strict_string_order(&theory.checked_member_ids, "theory checked member IDs")?;
    }
    for certificate in &document.trusted_evidence.certificates {
        let mut names = BTreeSet::new();
        for declaration in &certificate.checked_declarations {
            if !names.insert(&declaration.name) {
                return order("checked declaration names must be unique");
            }
            require_strict_string_order(&declaration.member_ids, "declaration member IDs")?;
            require_strict_order_by(
                &declaration.dependencies,
                |left, right| left.name.as_bytes() < right.name.as_bytes(),
                "declaration dependencies",
            )?;
        }
    }
    let verdicts = &document.trusted_evidence.checker_verdicts;
    if verdicts[0].checker != "rust_fast_kernel" || verdicts[1].checker != "reference_checker" {
        return order("checker verdict rows have the wrong order");
    }
    for verdict in verdicts {
        require_strict_string_order(&verdict.certificate_ids, "checker certificate IDs")?;
    }
    require_strict_order_by(
        &document.properties,
        |left, right| left.id.as_bytes() < right.id.as_bytes(),
        "properties",
    )?;
    let mut all_members = BTreeSet::new();
    for property in &document.properties {
        require_strict_order_by(
            &property.members,
            |left, right| left.member_id.as_bytes() < right.member_id.as_bytes(),
            "property members",
        )?;
        require_strict_string_order(&property.notes, "property notes")?;
        for member in &property.members {
            if !all_members.insert(&member.member_id) {
                return order("a member may appear in only one property");
            }
            require_strict_order_by(
                &member.evidence,
                |left, right| left.order_key() < right.order_key(),
                "member evidence",
            )?;
        }
    }
    Ok(())
}

fn require_release_order(
    frontend: &FrontendIdentity,
    toolchain: &ToolchainIdentity,
) -> Result<(), PolicyValidationError> {
    require_strict_order_by(
        &frontend.subordinate_binaries,
        |left, right| left.name.as_bytes() < right.name.as_bytes(),
        "subordinate binaries",
    )?;
    require_strict_order_by(
        &toolchain.components,
        |left, right| {
            let left_name = match left {
                ComponentIdentity::Executable { name, .. }
                | ComponentIdentity::Content { name, .. } => name.as_bytes(),
            };
            let right_name = match right {
                ComponentIdentity::Executable { name, .. }
                | ComponentIdentity::Content { name, .. } => name.as_bytes(),
            };
            left_name < right_name
        },
        "toolchain components",
    )
}

fn require_helper_order(helpers: &[PolicyHelperArtifact]) -> Result<(), PolicyValidationError> {
    require_strict_order_by(
        helpers,
        |left, right| {
            (left.kind_rank(), left.id().as_bytes()) < (right.kind_rank(), right.id().as_bytes())
        },
        "helper artifacts",
    )
}

fn require_issue_order(issues: &[PolicyIssue]) -> Result<(), PolicyValidationError> {
    for pair in issues.windows(2) {
        if issue_key(&pair[0]) > issue_key(&pair[1]) {
            return order("issues are not in frontend protocol order");
        }
    }
    Ok(())
}

type IssueKey<'a> = (&'a [u8], i64, &'a [u8], &'a [u8], &'a [u8], i64);

fn issue_key(issue: &PolicyIssue) -> IssueKey<'_> {
    let (path, start, end) = issue
        .span
        .as_ref()
        .map(|span| (span.normalized_path.as_bytes(), span.start, span.end))
        .unwrap_or((b"", 0, 0));
    (
        path,
        start,
        issue.code.as_bytes(),
        issue.message.as_bytes(),
        issue.function_id.as_deref().unwrap_or("").as_bytes(),
        end,
    )
}

fn require_strict_string_order(
    values: &[String],
    label: &str,
) -> Result<(), PolicyValidationError> {
    require_strict_order_by(
        values,
        |left, right| left.as_bytes() < right.as_bytes(),
        label,
    )
}

fn require_strict_order_by<T>(
    values: &[T],
    less: impl Fn(&T, &T) -> bool,
    label: &str,
) -> Result<(), PolicyValidationError> {
    for pair in values.windows(2) {
        if !less(&pair[0], &pair[1]) {
            return order(format!("{label} must be strictly increasing"));
        }
    }
    Ok(())
}

fn order<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Order,
        "POLICY_ORDER",
        detail,
    ))
}

fn validate_language_profile(
    language: &str,
    profile: &str,
    parameters: &PolicySemanticParameters,
    selection: &PolicySelection,
) -> Result<(), PolicyValidationError> {
    let known_profile = matches!(profile, "mpk.go.fixed.v0" | "mpk.rust.checked.v0");
    if !known_profile {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Profile,
            "POLICY_PROFILE_UNKNOWN",
            "semantic profile is not registered",
        ));
    }
    let valid = match (language, profile, parameters, selection) {
        (
            "go",
            "mpk.go.fixed.v0",
            PolicySemanticParameters::Go(parameters),
            PolicySelection::Go(selection),
        ) => {
            matches!(parameters.pointer_width, 32 | 64)
                && selection
                    .function
                    .starts_with(&format!("{}.", selection.package))
                && selection.function.len() > selection.package.len() + 1
        }
        (
            "rust",
            "mpk.rust.checked.v0",
            PolicySemanticParameters::Rust(parameters),
            PolicySelection::Rust(selection),
        ) => {
            matches!(parameters.pointer_width, 32 | 64)
                && parameters.overflow_mode == "checked"
                && parameters.panic_mode == "abort"
                && selection.kind == "lib"
                && selection
                    .function
                    .starts_with(&format!("{}::", selection.crate_name))
                && selection.function.len() > selection.crate_name.len() + 2
        }
        _ => false,
    };
    if !valid {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Profile,
            "POLICY_PROFILE_TUPLE",
            "language, semantic parameters, and selection form a crossed tuple",
        ));
    }
    Ok(())
}

fn validate_evidence_profiles(document: &PolicyEvidenceV1) -> Result<(), PolicyValidationError> {
    validate_language_profile(
        &document.source_language,
        &document.semantic_profile,
        &document.semantic_parameters,
        &document.selection,
    )?;
    validate_policy_profile_selection(PolicyProfileSelection {
        strategy_profile: &document.strategy_profile,
        checker_profile: &document.checker_profile,
        source_language: &document.source_language,
        semantic_profile: &document.semantic_profile,
        axiom_profile: &document.axiom_profile,
    })
    .map(|_| ())
    .map_err(|error| {
        let code = match error.kind() {
            PolicyProfileErrorKind::CrossedTuple => "POLICY_PROFILE_TUPLE",
            PolicyProfileErrorKind::Unknown | PolicyProfileErrorKind::PackageMismatch => {
                "POLICY_PROFILE_UNKNOWN"
            }
        };
        PolicyValidationError::new(PolicyValidationPhase::Profile, code, error.to_string())
    })
}

fn validate_scan_release(
    document: &PolicyScanV1,
    context: &PolicyScanLinkageContext,
) -> Result<(), PolicyValidationError> {
    if document.release_registry.schema != "mpk.release.bundle_registry.v0"
        || document.release_registry.id != "mpk.release.registry.v0"
        || document.release_registry != context.release_registry
        || document.frontend != context.frontend
        || document.toolchain != context.toolchain
        || document.limit_profile != context.limit_profile
        || (document.frontend_status == "ir-lowered"
            && document.limit_profile.as_deref() != Some("mpk.vir.limits.v0"))
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Release,
            "POLICY_RELEASE_LINKAGE",
            "scan release projection differs from the validated selected tuple",
        ));
    }
    Ok(())
}

fn validate_scan_source_linkage(
    document: &PolicyScanV1,
    context: &PolicyScanLinkageContext,
) -> Result<(), PolicyValidationError> {
    if document.frontend_status != context.frontend_status
        || document.frontend_phase != context.frontend_phase
        || document.source_language != context.source_language
        || document.semantic_profile != context.semantic_profile
        || document.semantic_parameters != context.semantic_parameters
        || document.selection != context.selection
        || document.rejected_features != context.rejected_features
        || document.diagnostics != context.diagnostics
        || document.frontend_source_manifest_hash != context.frontend_source_manifest_hash
        || document.input_set_hash != context.input_set_hash
        || document.source_map_hash != context.source_map_hash
        || document.source_ir_schema != context.source_ir_schema
        || document.source_ir_hash != context.source_ir_hash
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::SourceLinkage,
            "POLICY_SOURCE_LINKAGE",
            "scan source identity differs from the retained frontend result",
        ));
    }
    if document.frontend_status == "ir-lowered"
        && document.source_ir_schema.as_deref() != Some("mpk.vir.v0")
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::SourceLinkage,
            "POLICY_SOURCE_LINKAGE",
            "successful scan does not identify VIR v0 and its fixed limit profile",
        ));
    }
    if document.helper_artifacts != context.helper_artifacts {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::SourceLinkage,
            "POLICY_HELPER_LINKAGE",
            "scan helper projection differs from retained source, contract, or VIR identities",
        ));
    }
    if let Some(helpers) = &document.helper_artifacts {
        let verification_ir_count = helpers
            .iter()
            .filter(|helper| matches!(helper, PolicyHelperArtifact::VerificationIr { .. }))
            .count();
        if verification_ir_count != 1
            || helpers.iter().any(|helper| {
                matches!(
                    helper,
                    PolicyHelperArtifact::Vc { .. }
                        | PolicyHelperArtifact::AiAnalysis { .. }
                        | PolicyHelperArtifact::CiStatus { .. }
                )
            })
        {
            return Err(PolicyValidationError::new(
                PolicyValidationPhase::SourceLinkage,
                "POLICY_HELPER_LINKAGE",
                "successful scan helpers require exactly source, contract, and one VIR row",
            ));
        }
    }
    Ok(())
}

fn validate_evidence_release(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    let scan = context.scan.document();
    if document.release_registry != scan.release_registry
        || document.frontend != scan.frontend
        || document.toolchain != scan.toolchain
        || document.limit_profile != scan.limit_profile.as_deref().unwrap_or("")
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Release,
            "POLICY_RELEASE_LINKAGE",
            "evidence release projection differs from the retained scan",
        ));
    }
    Ok(())
}

fn validate_evidence_source_linkage(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    let scan = context.scan.document();
    if scan.frontend_status != "ir-lowered"
        || scan.readiness != "ready"
        || document.source_language != scan.source_language
        || document.semantic_profile != scan.semantic_profile
        || document.semantic_parameters != scan.semantic_parameters
        || document.selection != scan.selection
        || document.frontend_source_manifest_hash
            != scan.frontend_source_manifest_hash.as_deref().unwrap_or("")
        || document.input_set_hash != scan.input_set_hash.as_deref().unwrap_or("")
        || document.source_map_hash != scan.source_map_hash.as_deref().unwrap_or("")
        || document.source_ir_schema != scan.source_ir_schema.as_deref().unwrap_or("")
        || document.source_ir_hash != scan.source_ir_hash.as_deref().unwrap_or("")
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::SourceLinkage,
            "POLICY_SOURCE_LINKAGE",
            "evidence does not repeat the exact retained ready scan",
        ));
    }
    Ok(())
}

fn validate_evidence_vc(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    if document.source_vc_schema != context.source_vc_schema
        || document.vc_hash != context.vc_hash
        || document.verification_limit_profile != context.verification_limit_profile
        || document.source_vc_schema != "mpk.vc.v1"
        || document.verification_limit_profile != "mpk.verify.limits.v0"
    {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::VcLinkage,
            "POLICY_VC_LINKAGE",
            "VC or verification-limit identity differs from the validated VC",
        ));
    }
    Ok(())
}

fn validate_evidence_helpers(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    let mut expected = context
        .scan
        .document()
        .helper_artifacts
        .clone()
        .unwrap_or_default();
    expected.push(PolicyHelperArtifact::Vc {
        id: "vc".to_owned(),
        schema: context.source_vc_schema.clone(),
        sha256: context.vc_hash.clone(),
    });
    expected.extend(context.expected_optional_helpers.clone());
    if document.helper_artifacts != expected {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Helpers,
            "POLICY_HELPER_LINKAGE",
            "evidence helper projection differs from scan helpers plus the validated VC",
        ));
    }
    Ok(())
}

fn validate_trusted_evidence(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    let trusted = &document.trusted_evidence;
    if trusted.theory_certificates != context.expected_theory_certificates {
        return trusted_error("theory certificate projection differs from checked theory bytes");
    }
    if trusted.axiom_report != context.expected_axiom_report {
        return trusted_error("axiom report projection differs from the recomputed report");
    }
    if trusted.checker_verdicts != context.expected_checker_verdicts {
        return trusted_error("checker verdict projection differs from retained checker results");
    }
    let certificate_ids = trusted
        .certificates
        .iter()
        .map(|certificate| certificate.id.as_str())
        .collect::<Vec<_>>();
    for verdict in &trusted.checker_verdicts {
        if verdict.checker_profile != document.checker_profile {
            return trusted_error("checker verdict profile differs from the evidence root");
        }
        match verdict.verdict.as_str() {
            "accepted" | "rejected"
                if !certificate_ids.is_empty()
                    && verdict
                        .certificate_ids
                        .iter()
                        .map(String::as_str)
                        .eq(certificate_ids.iter().copied()) => {}
            "not_run" if verdict.certificate_ids.is_empty() => {}
            _ => return trusted_error("checker verdict does not name the complete candidate set"),
        }
    }
    if trusted
        .theory_certificates
        .iter()
        .any(|certificate| certificate.checker_profile != document.checker_profile)
    {
        return trusted_error("theory certificate profile differs from the evidence root");
    }
    match &context.expected_certificate {
        None => {
            if !trusted.certificates.is_empty()
                || !trusted.theory_certificates.is_empty()
                || !matches!(trusted.axiom_report, PolicyAxiomReportV1::NotGenerated)
                || trusted
                    .checker_verdicts
                    .iter()
                    .any(|verdict| verdict.verdict != "not_run")
            {
                return trusted_error("proof-pending context contains a trusted-evidence claim");
            }
        }
        Some(expected_certificate) => {
            let Some(certificate) = trusted.certificates.first() else {
                return trusted_error("candidate certificate is missing");
            };
            if trusted.certificates.len() != 1
                || certificate.module != expected_certificate.module
                || certificate.certificate_hash != expected_certificate.certificate_hash
                || certificate.export_hash != expected_certificate.export_hash
                || certificate.axiom_report_hash != expected_certificate.axiom_report_hash
                || certificate.checked_declarations.len() != context.expected_declarations.len()
            {
                return trusted_error(
                    "candidate certificate projection differs from checked bytes",
                );
            }
            for (actual, expected) in certificate
                .checked_declarations
                .iter()
                .zip(&context.expected_declarations)
            {
                if actual.name != expected.name
                    || actual.declaration_hash != expected.declaration_hash
                    || actual.function_id != expected.function_id
                    || actual.group_id != expected.group_id
                    || actual.group_kind != expected.group_kind
                    || actual.member_ids != expected.member_ids
                {
                    return trusted_error(
                        "checked declaration set or non-dependency fields differ",
                    );
                }
            }
            match &trusted.axiom_report {
                PolicyAxiomReportV1::Checked {
                    axiom_report_hash, ..
                } if axiom_report_hash == &expected_certificate.axiom_report_hash => {}
                _ => return trusted_error("candidate lacks its checked axiom report"),
            }
            if trusted
                .checker_verdicts
                .iter()
                .all(|verdict| verdict.verdict == "not_run")
            {
                return trusted_error("candidate certificate was not submitted to either checker");
            }
        }
    }

    Ok(())
}

fn trusted_error<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Trusted,
        "POLICY_TRUSTED_EVIDENCE",
        detail,
    ))
}

fn validate_properties(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    if document.properties.len() != context.expected_properties.len() {
        return member_linkage("property set differs from the registered strategy classification");
    }
    for (property, expected) in document.properties.iter().zip(&context.expected_properties) {
        let member_ids = property
            .members
            .iter()
            .map(|member| member.member_id.as_str())
            .collect::<Vec<_>>();
        if property.id != expected.id
            || property.description != expected.description
            || property.notes != expected.notes
            || !member_ids
                .iter()
                .copied()
                .eq(expected.member_ids.iter().map(String::as_str))
        {
            return member_linkage(
                "property identity, prose, notes, or member grouping differs from the registered strategy classification",
            );
        }
    }
    let members = context
        .expected_members
        .iter()
        .map(|member| (member.member_id.as_str(), member))
        .collect::<BTreeMap<_, _>>();
    let helper_ids = document
        .helper_artifacts
        .iter()
        .map(PolicyHelperArtifact::id)
        .collect::<BTreeSet<_>>();
    let theory = document
        .trusted_evidence
        .theory_certificates
        .iter()
        .map(|certificate| (certificate.id.as_str(), certificate))
        .collect::<BTreeMap<_, _>>();
    let expected_unsupported_codes = context
        .expected_unsupported_codes
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let both_checkers_accepted = document
        .trusted_evidence
        .checker_verdicts
        .iter()
        .all(|verdict| verdict.verdict == "accepted" && verdict.certificate_ids == ["program"]);
    let checked_axiom_report = matches!(
        document.trusted_evidence.axiom_report,
        PolicyAxiomReportV1::Checked { .. }
    );

    for property in &document.properties {
        for row in &property.members {
            let Some(expected) = members.get(row.member_id.as_str()) else {
                return member_linkage("property references a member outside the validated VC");
            };
            if row.function_id != expected.function_id
                || row.kind != expected.kind
                || row.group_id != expected.group_id
                || row.declaration_name != expected.declaration_name
                || row.declaration_hash != expected.declaration_hash
            {
                return member_linkage("member/group/declaration projection differs from the validated VC and skeleton");
            }

            let mut checked_declarations = 0usize;
            let mut checked_theories = BTreeSet::new();
            let mut helpers = 0usize;
            let mut unsupported = 0usize;
            for reference in &row.evidence {
                match reference {
                    PolicyEvidenceReferenceV1::CheckedDeclaration { certificate_id } => {
                        checked_declarations += 1;
                        let Some(certificate) = document.trusted_evidence.certificates.first()
                        else {
                            return property_status(
                                "checked declaration reference has no certificate",
                            );
                        };
                        if certificate_id != &certificate.id
                            || !certificate.checked_declarations.iter().any(|declaration| {
                                declaration.name == row.declaration_name
                                    && declaration.declaration_hash == row.declaration_hash
                                    && declaration.group_id == row.group_id
                                    && declaration.member_ids.contains(&row.member_id)
                            })
                        {
                            return property_status(
                                "checked declaration reference does not contain the member",
                            );
                        }
                    }
                    PolicyEvidenceReferenceV1::CheckedTheoryCertificate {
                        theory_certificate_id,
                    } => {
                        let Some(certificate) = theory.get(theory_certificate_id.as_str()) else {
                            return property_status("checked theory reference is unresolved");
                        };
                        if !certificate.checked_member_ids.contains(&row.member_id) {
                            return property_status(
                                "checked theory reference does not contain the member",
                            );
                        }
                        checked_theories.insert(theory_certificate_id.as_str());
                    }
                    PolicyEvidenceReferenceV1::HelperArtifact { artifact_id } => {
                        helpers += 1;
                        if !helper_ids.contains(artifact_id.as_str()) {
                            return property_status("helper reference is unresolved");
                        }
                    }
                    PolicyEvidenceReferenceV1::UnsupportedFeature { code } => {
                        unsupported += 1;
                        if !expected_unsupported_codes.contains(code.as_str()) {
                            return property_status(
                                "unsupported reference is not a retained verification classification",
                            );
                        }
                    }
                }
            }
            let expected_theories = theory
                .values()
                .filter(|certificate| certificate.checked_member_ids.contains(&row.member_id))
                .map(|certificate| certificate.id.as_str())
                .collect::<BTreeSet<_>>();
            let expected_checked_declarations = document
                .trusted_evidence
                .certificates
                .iter()
                .flat_map(|certificate| &certificate.checked_declarations)
                .filter(|declaration| declaration.member_ids.contains(&row.member_id))
                .count();
            let status_valid = match row.status.as_str() {
                "mpk_verified" => {
                    checked_declarations == 1
                        && expected_checked_declarations == 1
                        && checked_theories == expected_theories
                        && helpers == 0
                        && unsupported == 0
                        && both_checkers_accepted
                        && checked_axiom_report
                }
                "proof_pending" => checked_declarations == 0
                    && checked_theories.is_empty()
                    && unsupported == 0
                    && row.evidence.iter().any(|reference| matches!(reference, PolicyEvidenceReferenceV1::HelperArtifact { artifact_id } if artifact_id == "vc")),
                "helper_only" => {
                    checked_declarations == 0
                        && checked_theories.is_empty()
                        && unsupported == 0
                        && helpers > 0
                }
                "unsupported" => {
                    checked_declarations == 0
                        && checked_theories.is_empty()
                        && unsupported > 0
                }
                _ => false,
            };
            if !status_valid {
                return property_status(
                    "member status does not match its closed evidence-reference set",
                );
            }
        }
        let derived = if property
            .members
            .iter()
            .any(|member| member.status == "unsupported")
        {
            "unsupported"
        } else if property
            .members
            .iter()
            .any(|member| member.status == "proof_pending")
        {
            "proof_pending"
        } else if property
            .members
            .iter()
            .any(|member| member.status == "helper_only")
        {
            "helper_only"
        } else {
            "mpk_verified"
        };
        if property.status != derived {
            return property_status("property status is not the aggregate of member statuses");
        }
    }
    Ok(())
}

fn member_linkage<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Properties,
        "POLICY_MEMBER_LINKAGE",
        detail,
    ))
}

fn property_status<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Properties,
        "POLICY_PROPERTY_STATUS",
        detail,
    ))
}

fn validate_dependencies(
    document: &PolicyEvidenceV1,
    context: &PolicyEvidenceLinkageContext<'_>,
) -> Result<(), PolicyValidationError> {
    let Some(certificate) = document.trusted_evidence.certificates.first() else {
        return Ok(());
    };
    let positions = certificate
        .checked_declarations
        .iter()
        .enumerate()
        .map(|(index, declaration)| (declaration.name.as_str(), (index, declaration)))
        .collect::<BTreeMap<_, _>>();
    for (actual, expected) in certificate
        .checked_declarations
        .iter()
        .zip(&context.expected_declarations)
    {
        if actual.dependencies != expected.dependencies {
            return dependency_error("direct generated declaration dependency set differs");
        }
        let current = positions[actual.name.as_str()].0;
        for dependency in &actual.dependencies {
            let Some((position, declaration)) = positions.get(dependency.name.as_str()) else {
                return dependency_error("generated dependency declaration is missing");
            };
            if *position >= current || declaration.declaration_hash != dependency.declaration_hash {
                return dependency_error(
                    "generated dependency is not an earlier exact declaration",
                );
            }
        }
    }

    for member in document
        .properties
        .iter()
        .flat_map(|property| &property.members)
        .filter(|member| member.status == "mpk_verified")
    {
        let Some((_, containing)) = positions.get(member.declaration_name.as_str()) else {
            return dependency_error("verified member containing declaration is missing");
        };
        let mut pending = containing.dependencies.clone();
        let mut visited = BTreeSet::new();
        while let Some(dependency) = pending.pop() {
            if !visited.insert(dependency.name.clone()) {
                continue;
            }
            let Some((_, declaration)) = positions.get(dependency.name.as_str()) else {
                return dependency_error("verified member dependency closure is incomplete");
            };
            if declaration.declaration_hash != dependency.declaration_hash {
                return dependency_error("verified member dependency closure hash differs");
            }
            pending.extend(declaration.dependencies.clone());
        }
    }
    Ok(())
}

fn dependency_error<T>(detail: impl Into<String>) -> Result<T, PolicyValidationError> {
    Err(PolicyValidationError::new(
        PolicyValidationPhase::Dependencies,
        "POLICY_DEPENDENCY_CLOSURE",
        detail,
    ))
}

fn validate_recipes(document: &PolicyEvidenceV1) -> Result<(), PolicyValidationError> {
    let expected = expected_reproduction_recipes(document);
    if document.reproduction_recipes != expected {
        return Err(PolicyValidationError::new(
            PolicyValidationPhase::Recipes,
            "POLICY_RECIPE",
            "reproduction recipes differ from the exact normalized invocation",
        ));
    }
    Ok(())
}
