//! Active successor AI-explanation integration.
//!
//! This module accepts only a validated successor policy
//! evidence document and its profile-owned `ai` contract. It prepares a
//! deterministic, redacted provider request and assembles an untrusted local
//! report. It has no proof-acceptance role.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, CompiledProfileEnvelope, CompiledSemanticProfile,
    ProfileContractField, SemanticContext, SemanticParametersEnvelope,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, serialize_json_bounded, StrictJsonLimits,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::ai_explain::{
    parse_provider_response_v0, prompt_template_sha256_v1, ModelExplanationResponseV0,
    VertexContentV1, VertexEnumStringSchemaV1, VertexGenerateRequestV1, VertexGenerationConfigV1,
    VertexPartV1, VertexPropertyExplanationPropertiesV1, VertexPropertyExplanationSchemaV1,
    VertexPropertyExplanationsSchemaV1, VertexResponseFormatV1, VertexResponseSchemaPropertiesV1,
    VertexResponseSchemaV1, VertexStringSchemaV1, VertexTextListSchemaV1,
    VertexTextResponseFormatV1, VertexThinkingConfigV1, AI_EXPLANATION_RESPONSE_SCHEMA_V0,
    MINIMAL_REDACTION_PROFILE_V1, PROMPT_TEMPLATE_ID_V1, SYSTEM_INSTRUCTION_V1,
    TRUST_CLASSIFICATION, TRUST_DISCLAIMER, USER_TEMPLATE_V1, VERTEX_AI_PROVIDER,
};
pub use crate::ai_explain::{ExplainLanguageV1, DEFAULT_GEMINI_MODEL};
use crate::policy_schema::{
    PolicyAxiomReportV1, PolicyEvidenceReferenceV1, PolicyHelperArtifact, PolicyPropertyV1,
    PolicyTrustedEvidenceV1,
};
use crate::successor_policy::{
    ValidatedSuccessorPolicyEvidenceV2, SUCCESSOR_POLICY_EVIDENCE_SCHEMA,
    SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE,
};

pub const SUCCESSOR_AI_EXPLAIN_REQUEST_SCHEMA: &str = "mpk.ai.explain.request.v2";
pub const SUCCESSOR_AI_EXPLANATION_SCHEMA: &str = "mpk.ai.explanation.v2";

const PROMPT_PLACEHOLDER: &str = "{{SANITIZED_PAYLOAD_JSON}}";
const MAX_EVIDENCE_BYTES: usize = 2 * 1024 * 1024;
const MAX_SANITIZED_REQUEST_BYTES: usize = 64 * 1024;
const MAX_VERTEX_REQUEST_BYTES: usize = 96 * 1024;
const MAX_EXPLANATION_BYTES: usize = 2 * 1024 * 1024;
const MAX_PROVIDER_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_RETAINED_IDENTIFIER_BYTES: usize = 4 * 1024;
const MAX_PROPERTIES: usize = 32;
const MAX_OVERVIEW_BYTES: usize = 2_000;
const MAX_GENERATED_ITEM_BYTES: usize = 500;
const MAX_GENERATED_LIST_ITEMS: usize = 10;
const MAX_TOTAL_AI_TEXT_BYTES: usize = 32 * 1024;
const MAX_JSON_NESTING: u64 = 128;

const RECOGNIZED_CATEGORIES: &[&str] = &[
    "non_negative_result",
    "result_bounded_by_input",
    "refund_bounded_by_available_paid_amount",
    "fee_or_discount_bounded_by_cap",
    "selected_branch_result_equals_input",
    "integer_runtime_safety",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorAiPhase {
    EvidenceLinkage,
    ProfileContract,
    RedactionProjection,
    ProviderResponse,
    ReportAssembly,
    Transport,
    CanonicalTransport,
}

impl SuccessorAiPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceLinkage => "evidence_linkage",
            Self::ProfileContract => "profile_contract",
            Self::RedactionProjection => "redaction_projection",
            Self::ProviderResponse => "provider_response",
            Self::ReportAssembly => "report_assembly",
            Self::Transport => "transport",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorAiCode {
    EvidenceLinkage,
    ProfileContract,
    Projection,
    ResponseInvalid,
    ReportInvalid,
    Json,
    CanonicalTransport,
}

impl SuccessorAiCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceLinkage => "SUCCESSOR_AI_EVIDENCE_LINKAGE",
            Self::ProfileContract => "SUCCESSOR_AI_PROFILE_CONTRACT",
            Self::Projection => "SUCCESSOR_AI_PROJECTION",
            Self::ResponseInvalid => "SUCCESSOR_AI_RESPONSE_INVALID",
            Self::ReportInvalid => "SUCCESSOR_AI_REPORT_INVALID",
            Self::Json => "SUCCESSOR_AI_JSON",
            Self::CanonicalTransport => "SUCCESSOR_AI_CANONICAL_TRANSPORT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorAiError {
    phase: SuccessorAiPhase,
    code: SuccessorAiCode,
    detail: &'static str,
}

impl SuccessorAiError {
    pub const fn phase(&self) -> SuccessorAiPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorAiCode {
        self.code
    }

    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl fmt::Display for SuccessorAiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} during {}: {}",
            self.code.as_str(),
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for SuccessorAiError {}

#[derive(Clone, Copy)]
pub struct SuccessorAiSource<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub evidence: &'a ValidatedSuccessorPolicyEvidenceV2,
    pub ai_contract: &'a Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuccessorAiRegistration {
    profile: CompiledSemanticProfile,
    display_language: &'static str,
    projection_profile_id: &'static str,
    redaction_profile_id: &'static str,
    strategy_profile: &'static str,
    checker_profile: &'static str,
    axiom_profile: &'static str,
}

impl SuccessorAiRegistration {
    pub const fn profile(self) -> CompiledSemanticProfile {
        self.profile
    }

    pub const fn display_language(self) -> &'static str {
        self.display_language
    }

    pub const fn projection_profile_id(self) -> &'static str {
        self.projection_profile_id
    }

    pub const fn redaction_profile_id(self) -> &'static str {
        self.redaction_profile_id
    }

    pub const fn strategy_profile(self) -> &'static str {
        self.strategy_profile
    }

    pub const fn checker_profile(self) -> &'static str {
        self.checker_profile
    }

    pub const fn axiom_profile(self) -> &'static str {
        self.axiom_profile
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SourcePropertyStatusV2 {
    MpkVerified,
    ProofPending,
    HelperOnly,
    Unsupported,
}

impl SourcePropertyStatusV2 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MpkVerified => "mpk_verified",
            Self::ProofPending => "proof_pending",
            Self::HelperOnly => "helper_only",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SanitizedEvidenceKindV2 {
    CheckedDeclaration,
    CheckedTheoryCertificate,
    HelperArtifact,
    UnsupportedFeature,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SanitizedHelperKindV2 {
    Source,
    Contract,
    VerificationIr,
    Vc,
    AiAnalysis,
    CiStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SanitizedSuccessorAiRequestV2 {
    schema: String,
    language: ExplainLanguageV1,
    display_language: String,
    projection_profile_id: String,
    redaction_profile_id: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: SemanticParametersEnvelope,
    policy: SanitizedPolicyV2,
    summary: SanitizedSummaryV2,
    trusted_evidence_summary: SanitizedTrustedEvidenceSummaryV2,
    properties: Vec<SanitizedPropertyV2>,
    helper_artifact_summary: Vec<SanitizedHelperSummaryV2>,
}

impl SanitizedSuccessorAiRequestV2 {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn display_language(&self) -> &str {
        &self.display_language
    }

    pub fn source_language(&self) -> &str {
        &self.source_language
    }

    pub fn semantic_profile(&self) -> &str {
        &self.semantic_profile
    }

    pub fn property_count(&self) -> usize {
        self.properties.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedPolicyV2 {
    strategy_profile: String,
    checker_profile: String,
    axiom_profile: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedSummaryV2 {
    total: u32,
    mpk_verified: u32,
    proof_pending: u32,
    helper_only: u32,
    unsupported: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedTrustedEvidenceSummaryV2 {
    certificate_candidates: u32,
    checked_theory_certificates: u32,
    theory_formats: Vec<String>,
    rust_fast_kernel: String,
    reference_checker: String,
    axiom_counts: Option<SanitizedAxiomCountsV2>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedAxiomCountsV2 {
    total_axiom_count: i64,
    core_axiom_count: i64,
    builtin_theory_axiom_count: i64,
    go_semantics_axiom_count: i64,
    external_axiom_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedPropertyV2 {
    #[serde(rename = "ref")]
    property_ref: String,
    category: String,
    status: SourcePropertyStatusV2,
    evidence_kinds: Vec<SanitizedEvidenceKindV2>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SanitizedHelperSummaryV2 {
    artifact: SanitizedHelperKindV2,
    count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PropertyAliasV2 {
    property_ref: String,
    original_id: String,
    original_status: SourcePropertyStatusV2,
    original_index: usize,
}

struct ProjectedPropertyV2 {
    original_index: usize,
    original_id: String,
    category: String,
    status: SourcePropertyStatusV2,
    evidence_kinds: Vec<SanitizedEvidenceKindV2>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedSuccessorAiRequestV2 {
    registration: SuccessorAiRegistration,
    payload: SanitizedSuccessorAiRequestV2,
    canonical_request: Vec<u8>,
    request_body: Vec<u8>,
    evidence_sha256: String,
    selection_sha256: String,
    prompt_template_sha256: String,
    response_schema_sha256: String,
    sanitized_payload_sha256: String,
    request_body_sha256: String,
    alias_map: Vec<PropertyAliasV2>,
    semantic_context: SemanticContext,
    policy_contract: CompiledProfileEnvelope,
    ai_contract: CompiledProfileEnvelope,
}

impl PreparedSuccessorAiRequestV2 {
    pub const fn registration(&self) -> SuccessorAiRegistration {
        self.registration
    }

    pub fn document(&self) -> &SanitizedSuccessorAiRequestV2 {
        &self.payload
    }

    pub fn canonical_request_bytes(&self) -> &[u8] {
        &self.canonical_request
    }

    pub fn request_body(&self) -> &[u8] {
        &self.request_body
    }

    pub fn evidence_sha256(&self) -> &str {
        &self.evidence_sha256
    }

    pub fn selection_sha256(&self) -> &str {
        &self.selection_sha256
    }

    pub fn import_request_json(&self, input: &[u8]) -> Result<(), SuccessorAiError> {
        validate_exact_document(
            input,
            &self.payload,
            MAX_SANITIZED_REQUEST_BYTES,
            "successor AI request did not exactly regenerate",
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorAiReportRequest {
    pub project: String,
    pub location: String,
    pub requested_model: String,
    pub language: ExplainLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorAiProviderProvenance {
    pub model_version: String,
    pub response_id: String,
    pub create_time: String,
    pub finish_reason: String,
    pub attempts: u8,
    pub prompt_tokens: Option<u64>,
    pub thinking_tokens: Option<u64>,
    pub response_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorAiExplanationV2 {
    schema: String,
    generator: GeneratorMetadataV2,
    trust: TrustLabelV2,
    source_evidence: SourceEvidenceReferenceV2,
    semantic_context: SemanticContext,
    selection_reference: SelectionReferenceV2,
    policy_contract: CompiledProfileEnvelope,
    ai_contract: CompiledProfileEnvelope,
    request: ExplainOutputRequestV2,
    provider_response: ProviderProvenanceV2,
    local_summary: LocalSummaryV2,
    ai_analysis: AiAnalysisV2,
}

impl SuccessorAiExplanationV2 {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn policy_contract(&self) -> &CompiledProfileEnvelope {
        &self.policy_contract
    }

    pub fn ai_contract(&self) -> &CompiledProfileEnvelope {
        &self.ai_contract
    }

    pub const fn proof_evidence(&self) -> bool {
        self.trust.proof_evidence
    }

    pub fn trust_classification(&self) -> &str {
        &self.trust.classification
    }

    pub fn property_explanations(&self) -> impl Iterator<Item = (&str, &str, &str)> {
        self.ai_analysis
            .property_explanations
            .iter()
            .map(|property| {
                (
                    property.property_id.as_str(),
                    property.source_status.as_str(),
                    property.explanation.as_str(),
                )
            })
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorAiExplanationV2 {
    document: SuccessorAiExplanationV2,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorAiExplanationV2 {
    pub fn document(&self) -> &SuccessorAiExplanationV2 {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn import_json(&self, input: &[u8]) -> Result<(), SuccessorAiError> {
        validate_exact_document(
            input,
            &self.document,
            MAX_EXPLANATION_BYTES,
            "successor AI report did not exactly regenerate",
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct GeneratorMetadataV2 {
    name: String,
    version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct TrustLabelV2 {
    classification: String,
    proof_evidence: bool,
    disclaimer: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SourceEvidenceReferenceV2 {
    schema: String,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SelectionReferenceV2 {
    schema: String,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ExplainOutputRequestV2 {
    provider: String,
    project: String,
    location: String,
    requested_model: String,
    language: ExplainLanguageV1,
    display_language: String,
    projection_profile_id: String,
    redaction_profile_id: String,
    prompt_template: String,
    prompt_template_sha256: String,
    response_schema: String,
    response_schema_sha256: String,
    sanitized_payload_sha256: String,
    request_body_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProviderProvenanceV2 {
    model_version: String,
    response_id: String,
    create_time: String,
    finish_reason: String,
    attempts: u8,
    prompt_tokens: Option<u64>,
    thinking_tokens: Option<u64>,
    response_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct LocalSummaryV2 {
    source_language: String,
    semantic_profile: String,
    strategy_profile: String,
    checker_profile: String,
    axiom_profile: String,
    total: u32,
    mpk_verified: u32,
    proof_pending: u32,
    helper_only: u32,
    unsupported: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AiAnalysisV2 {
    overview: String,
    property_explanations: Vec<AiPropertyExplanationV2>,
    limitations: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AiPropertyExplanationV2 {
    property_id: String,
    source_status: SourcePropertyStatusV2,
    explanation: String,
}

/// Prepares the deterministic, redacted successor provider request. The
/// returned bytes are inert staging output and are not sent anywhere.
pub fn prepare_successor_ai_explanation(
    source: SuccessorAiSource<'_>,
    language: ExplainLanguageV1,
) -> Result<PreparedSuccessorAiRequestV2, SuccessorAiError> {
    let evidence_bytes = source.evidence.canonical_bytes();
    let evidence = source.evidence.document();
    if evidence_bytes.len() > MAX_EVIDENCE_BYTES
        || evidence.schema() != SUCCESSOR_POLICY_EVIDENCE_SCHEMA
        || evidence.program_certificate_profile() != SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE
    {
        return Err(failure(
            SuccessorAiPhase::EvidenceLinkage,
            SuccessorAiCode::EvidenceLinkage,
            "successor policy evidence identity is invalid",
        ));
    }

    let context = evidence.semantic_context();
    if context.profile_registry() != source.registry.identity() {
        return Err(failure(
            SuccessorAiPhase::EvidenceLinkage,
            SuccessorAiCode::EvidenceLinkage,
            "semantic registry identity does not match evidence",
        ));
    }
    let entry = source
        .registry
        .lookup(context.source_language(), context.semantic_profile())
        .filter(|entry| {
            entry.entry_sha256() == context.profile_entry_sha256()
                && entry.semantic_parameters_schema() == context.semantic_parameters().schema()
                && entry.selection_schema() == evidence.selection().schema()
        })
        .ok_or_else(|| {
            failure(
                SuccessorAiPhase::EvidenceLinkage,
                SuccessorAiCode::EvidenceLinkage,
                "semantic context or selection is not registered",
            )
        })?;
    for (field, envelope) in [
        (ProfileContractField::Policy, evidence.policy_contract()),
        (ProfileContractField::Evidence, evidence.evidence_contract()),
    ] {
        if envelope.profile_entry_sha256() != entry.entry_sha256()
            || envelope.contract_id() != entry.contracts().contract_id(field)
        {
            return Err(failure(
                SuccessorAiPhase::EvidenceLinkage,
                SuccessorAiCode::EvidenceLinkage,
                "validated evidence carries a crossed profile contract",
            ));
        }
    }

    let registration = registration(entry.compiled_profile())?;
    validate_policy_contract(evidence.policy_contract(), registration)?;
    let ai_contract = validate_compiled_profile_envelope(
        source.registry,
        source.ai_contract,
        ProfileContractField::Ai,
    )
    .map_err(|_| {
        failure(
            SuccessorAiPhase::ProfileContract,
            SuccessorAiCode::ProfileContract,
            "AI profile contract is invalid",
        )
    })?;
    if ai_contract.profile_entry_sha256() != entry.entry_sha256()
        || ai_contract.contract_id() != entry.contracts().contract_id(ProfileContractField::Ai)
    {
        return Err(failure(
            SuccessorAiPhase::ProfileContract,
            SuccessorAiCode::ProfileContract,
            "AI profile contract is crossed",
        ));
    }

    let (properties, alias_map) = project_properties(evidence.properties())?;
    let summary = summarize_statuses(evidence.properties())?;
    let trusted_evidence_summary =
        summarize_trusted_evidence(evidence.trusted_evidence(), registration)?;
    let helper_artifact_summary = summarize_helpers(evidence.helper_artifacts())?;
    let payload = SanitizedSuccessorAiRequestV2 {
        schema: SUCCESSOR_AI_EXPLAIN_REQUEST_SCHEMA.to_owned(),
        language,
        display_language: registration.display_language.to_owned(),
        projection_profile_id: registration.projection_profile_id.to_owned(),
        redaction_profile_id: registration.redaction_profile_id.to_owned(),
        source_language: context.source_language().to_owned(),
        semantic_profile: context.semantic_profile().to_owned(),
        semantic_parameters: context.semantic_parameters().clone(),
        policy: SanitizedPolicyV2 {
            strategy_profile: registration.strategy_profile.to_owned(),
            checker_profile: registration.checker_profile.to_owned(),
            axiom_profile: registration.axiom_profile.to_owned(),
        },
        summary,
        trusted_evidence_summary,
        properties,
        helper_artifact_summary,
    };
    let canonical_request = canonical_document(
        &payload,
        MAX_SANITIZED_REQUEST_BYTES,
        SuccessorAiPhase::RedactionProjection,
        SuccessorAiCode::Projection,
        "sanitized request exceeds its closed transport",
    )?;
    let payload_json = String::from_utf8(canonical_request.clone()).map_err(|_| {
        failure(
            SuccessorAiPhase::RedactionProjection,
            SuccessorAiCode::Projection,
            "sanitized request is not UTF-8",
        )
    })?;
    let response_schema = build_response_schema(&alias_map)?;
    let response_schema_bytes = serde_json::to_vec(&response_schema).map_err(|_| {
        failure(
            SuccessorAiPhase::RedactionProjection,
            SuccessorAiCode::Projection,
            "provider response schema could not serialize",
        )
    })?;
    let user_text = replace_prompt_payload(&payload_json)?;
    let request = VertexGenerateRequestV1 {
        system_instruction: VertexContentV1 {
            role: None,
            parts: vec![VertexPartV1 {
                text: SYSTEM_INSTRUCTION_V1.to_owned(),
            }],
        },
        contents: vec![VertexContentV1 {
            role: Some("user".to_owned()),
            parts: vec![VertexPartV1 { text: user_text }],
        }],
        generation_config: VertexGenerationConfigV1 {
            candidate_count: 1,
            temperature: 0.0,
            max_output_tokens: 8192,
            response_format: vec![VertexResponseFormatV1 {
                text: VertexTextResponseFormatV1 {
                    mime_type: "APPLICATION_JSON",
                    schema: response_schema,
                },
            }],
            thinking_config: VertexThinkingConfigV1 {
                thinking_level: "MINIMAL",
                include_thoughts: false,
            },
        },
    };
    let mut request_body = serde_json::to_vec_pretty(&request).map_err(|_| {
        failure(
            SuccessorAiPhase::RedactionProjection,
            SuccessorAiCode::Projection,
            "provider request could not serialize",
        )
    })?;
    request_body.push(b'\n');
    if request_body.len() > MAX_VERTEX_REQUEST_BYTES {
        return Err(failure(
            SuccessorAiPhase::RedactionProjection,
            SuccessorAiCode::Projection,
            "provider request exceeds its closed transport",
        ));
    }
    let selection_bytes = canonical_document(
        evidence.selection(),
        MAX_SANITIZED_REQUEST_BYTES,
        SuccessorAiPhase::EvidenceLinkage,
        SuccessorAiCode::EvidenceLinkage,
        "selection reference could not be canonicalized",
    )?;

    Ok(PreparedSuccessorAiRequestV2 {
        registration,
        payload,
        canonical_request: canonical_request.clone(),
        request_body_sha256: sha256_hex(&request_body),
        request_body,
        evidence_sha256: sha256_hex(evidence_bytes),
        selection_sha256: sha256_hex(&selection_bytes),
        prompt_template_sha256: prompt_template_sha256_v1(),
        response_schema_sha256: sha256_hex(&response_schema_bytes),
        sanitized_payload_sha256: sha256_hex(&canonical_request),
        alias_map,
        semantic_context: context.clone(),
        policy_contract: evidence.policy_contract().clone(),
        ai_contract,
    })
}

/// Imports a canonical successor request by regenerating it from the complete
/// validated evidence and profile-contract boundary. Predecessor, crossed,
/// widened, noncanonical, or otherwise mutated request bytes cannot be
/// imported as a successor request.
pub fn import_successor_ai_request_json(
    input: &[u8],
    source: SuccessorAiSource<'_>,
    language: ExplainLanguageV1,
) -> Result<PreparedSuccessorAiRequestV2, SuccessorAiError> {
    let prepared = prepare_successor_ai_explanation(source, language)?;
    prepared.import_request_json(input)?;
    Ok(prepared)
}

/// Assembles a canonical local report from a provider's prose-only response.
/// All evidence identifiers, statuses, profile bindings, and trust labels are
/// restored from the prepared local request rather than accepted from the
/// provider response.
pub fn build_successor_ai_explanation(
    prepared: &PreparedSuccessorAiRequestV2,
    request: &SuccessorAiReportRequest,
    provenance: &SuccessorAiProviderProvenance,
    provider_text: &[u8],
) -> Result<ValidatedSuccessorAiExplanationV2, SuccessorAiError> {
    if request.requested_model != DEFAULT_GEMINI_MODEL
        || !valid_project_id(&request.project)
        || !valid_location(&request.location)
        || request.language != prepared.payload.language
    {
        return Err(response_failure("AI report request metadata is invalid"));
    }
    validate_provider_provenance(provenance)?;
    if provider_text.len() > MAX_PROVIDER_RESPONSE_BYTES {
        return Err(response_failure("provider response is too large"));
    }
    let model_response = parse_provider_response_v0(provider_text)
        .map_err(|_| response_failure("provider response does not match the prose-only schema"))?;
    validate_model_response(&model_response, &prepared.alias_map)?;

    let generated = model_response
        .property_explanations
        .iter()
        .map(|property| {
            (
                property.property_ref.as_str(),
                property.explanation.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut aliases = prepared.alias_map.clone();
    aliases.sort_by_key(|alias| alias.original_index);
    let property_explanations = aliases
        .into_iter()
        .map(|alias| AiPropertyExplanationV2 {
            property_id: alias.original_id,
            source_status: alias.original_status,
            explanation: generated[alias.property_ref.as_str()].to_owned(),
        })
        .collect();
    let summary = &prepared.payload.summary;
    let registration = prepared.registration;
    let document = SuccessorAiExplanationV2 {
        schema: SUCCESSOR_AI_EXPLANATION_SCHEMA.to_owned(),
        generator: GeneratorMetadataV2 {
            name: "mpk".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        trust: TrustLabelV2 {
            classification: TRUST_CLASSIFICATION.to_owned(),
            proof_evidence: false,
            disclaimer: TRUST_DISCLAIMER.to_owned(),
        },
        source_evidence: SourceEvidenceReferenceV2 {
            schema: SUCCESSOR_POLICY_EVIDENCE_SCHEMA.to_owned(),
            sha256: prepared.evidence_sha256.clone(),
        },
        semantic_context: prepared.semantic_context.clone(),
        selection_reference: SelectionReferenceV2 {
            schema: prepared.semantic_context_source_selection_schema(),
            sha256: prepared.selection_sha256.clone(),
        },
        policy_contract: prepared.policy_contract.clone(),
        ai_contract: prepared.ai_contract.clone(),
        request: ExplainOutputRequestV2 {
            provider: VERTEX_AI_PROVIDER.to_owned(),
            project: request.project.clone(),
            location: request.location.clone(),
            requested_model: request.requested_model.clone(),
            language: request.language,
            display_language: registration.display_language.to_owned(),
            projection_profile_id: registration.projection_profile_id.to_owned(),
            redaction_profile_id: registration.redaction_profile_id.to_owned(),
            prompt_template: PROMPT_TEMPLATE_ID_V1.to_owned(),
            prompt_template_sha256: prepared.prompt_template_sha256.clone(),
            response_schema: AI_EXPLANATION_RESPONSE_SCHEMA_V0.to_owned(),
            response_schema_sha256: prepared.response_schema_sha256.clone(),
            sanitized_payload_sha256: prepared.sanitized_payload_sha256.clone(),
            request_body_sha256: prepared.request_body_sha256.clone(),
        },
        provider_response: ProviderProvenanceV2 {
            model_version: provenance.model_version.clone(),
            response_id: provenance.response_id.clone(),
            create_time: provenance.create_time.clone(),
            finish_reason: provenance.finish_reason.clone(),
            attempts: provenance.attempts,
            prompt_tokens: provenance.prompt_tokens,
            thinking_tokens: provenance.thinking_tokens,
            response_tokens: provenance.response_tokens,
            total_tokens: provenance.total_tokens,
        },
        local_summary: LocalSummaryV2 {
            source_language: prepared.payload.source_language.clone(),
            semantic_profile: prepared.payload.semantic_profile.clone(),
            strategy_profile: registration.strategy_profile.to_owned(),
            checker_profile: registration.checker_profile.to_owned(),
            axiom_profile: registration.axiom_profile.to_owned(),
            total: summary.total,
            mpk_verified: summary.mpk_verified,
            proof_pending: summary.proof_pending,
            helper_only: summary.helper_only,
            unsupported: summary.unsupported,
        },
        ai_analysis: AiAnalysisV2 {
            overview: model_response.overview,
            property_explanations,
            limitations: model_response.limitations,
            next_steps: model_response.next_steps,
        },
    };
    let canonical_bytes = canonical_document(
        &document,
        MAX_EXPLANATION_BYTES,
        SuccessorAiPhase::ReportAssembly,
        SuccessorAiCode::ReportInvalid,
        "successor AI report exceeds its closed transport",
    )?;
    Ok(ValidatedSuccessorAiExplanationV2 {
        document,
        canonical_bytes,
    })
}

/// Imports a canonical successor explanation by regenerating every trusted
/// field from the prepared local request and the separately supplied provider
/// prose/provenance. The provider response is never treated as an authority
/// for evidence, status, semantic context, profile contracts, or trust labels.
pub fn import_successor_ai_explanation_json(
    input: &[u8],
    prepared: &PreparedSuccessorAiRequestV2,
    request: &SuccessorAiReportRequest,
    provenance: &SuccessorAiProviderProvenance,
    provider_text: &[u8],
) -> Result<ValidatedSuccessorAiExplanationV2, SuccessorAiError> {
    let mut explanation =
        build_successor_ai_explanation(prepared, request, provenance, provider_text)?;
    explanation.import_json(input)?;
    explanation.canonical_bytes = input.to_vec();
    Ok(explanation)
}

impl PreparedSuccessorAiRequestV2 {
    fn semantic_context_source_selection_schema(&self) -> String {
        self.registration.profile.selection_schema().to_owned()
    }
}

fn registration(
    profile: CompiledSemanticProfile,
) -> Result<SuccessorAiRegistration, SuccessorAiError> {
    Ok(match profile {
        CompiledSemanticProfile::GoFixedV0 => SuccessorAiRegistration {
            profile,
            display_language: "Go",
            projection_profile_id: "mpk.go.ai_projection.v0",
            redaction_profile_id: MINIMAL_REDACTION_PROFILE_V1,
            strategy_profile: "payment-policy-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "zero-axiom",
        },
        CompiledSemanticProfile::RustCheckedV0 => SuccessorAiRegistration {
            profile,
            display_language: "Rust",
            projection_profile_id: "mpk.rust.ai_projection.v0",
            redaction_profile_id: MINIMAL_REDACTION_PROFILE_V1,
            strategy_profile: "payment-policy-rust-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "mvp-theory",
        },
        CompiledSemanticProfile::CSharpScalarV0 => SuccessorAiRegistration {
            profile,
            display_language: "C#",
            projection_profile_id: "mpk.csharp.ai_projection.v0",
            redaction_profile_id: MINIMAL_REDACTION_PROFILE_V1,
            strategy_profile: "payment-policy-csharp-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "mvp-theory",
        },
        CompiledSemanticProfile::JavaScalarV0 => SuccessorAiRegistration {
            profile,
            display_language: "Java",
            projection_profile_id: "mpk.java.ai_projection.v0",
            redaction_profile_id: MINIMAL_REDACTION_PROFILE_V1,
            strategy_profile: "payment-policy-java-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "mvp-theory",
        },
    })
}

fn validate_policy_contract(
    contract: &CompiledProfileEnvelope,
    registration: SuccessorAiRegistration,
) -> Result<(), SuccessorAiError> {
    let value = contract.value();
    let exact = value.as_object().is_some_and(|object| {
        object.len() == 3
            && value.get("strategy_profile").and_then(Value::as_str)
                == Some(registration.strategy_profile)
            && value.get("checker_profile").and_then(Value::as_str)
                == Some(registration.checker_profile)
            && value.get("axiom_profile").and_then(Value::as_str)
                == Some(registration.axiom_profile)
    });
    if exact {
        Ok(())
    } else {
        Err(failure(
            SuccessorAiPhase::ProfileContract,
            SuccessorAiCode::ProfileContract,
            "policy strategy tuple is crossed",
        ))
    }
}

fn project_properties(
    properties: &[PolicyPropertyV1],
) -> Result<(Vec<SanitizedPropertyV2>, Vec<PropertyAliasV2>), SuccessorAiError> {
    if properties.is_empty() || properties.len() > MAX_PROPERTIES {
        return Err(projection_failure(
            "property count is outside the closed limit",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut projected = Vec::with_capacity(properties.len());
    for (original_index, property) in properties.iter().enumerate() {
        if property.id.is_empty()
            || property.id.len() > MAX_RETAINED_IDENTIFIER_BYTES
            || property.id.chars().any(is_forbidden_character)
            || !ids.insert(property.id.as_str())
        {
            return Err(projection_failure("property identity is invalid"));
        }
        let mut evidence_kinds = Vec::new();
        for reference in property.members.iter().flat_map(|member| &member.evidence) {
            let kind = match reference {
                PolicyEvidenceReferenceV1::CheckedDeclaration { .. } => {
                    SanitizedEvidenceKindV2::CheckedDeclaration
                }
                PolicyEvidenceReferenceV1::CheckedTheoryCertificate { .. } => {
                    SanitizedEvidenceKindV2::CheckedTheoryCertificate
                }
                PolicyEvidenceReferenceV1::HelperArtifact { .. } => {
                    SanitizedEvidenceKindV2::HelperArtifact
                }
                PolicyEvidenceReferenceV1::UnsupportedFeature { .. } => {
                    SanitizedEvidenceKindV2::UnsupportedFeature
                }
            };
            if !evidence_kinds.contains(&kind) {
                evidence_kinds.push(kind);
            }
        }
        evidence_kinds.sort_by_key(|kind| evidence_kind_rank(*kind));
        projected.push(ProjectedPropertyV2 {
            original_index,
            original_id: property.id.clone(),
            category: extract_category(&property.description),
            status: parse_status(&property.status)?,
            evidence_kinds,
        });
    }
    projected.sort_by(|left, right| {
        status_rank(left.status)
            .cmp(&status_rank(right.status))
            .then_with(|| category_rank(&left.category).cmp(&category_rank(&right.category)))
            .then_with(|| {
                evidence_bitset(&left.evidence_kinds).cmp(&evidence_bitset(&right.evidence_kinds))
            })
            .then_with(|| left.original_index.cmp(&right.original_index))
    });

    let mut sanitized = Vec::with_capacity(projected.len());
    let mut aliases = Vec::with_capacity(projected.len());
    for (index, property) in projected.into_iter().enumerate() {
        let property_ref = format!("property-{:04}", index + 1);
        aliases.push(PropertyAliasV2 {
            property_ref: property_ref.clone(),
            original_id: property.original_id,
            original_status: property.status,
            original_index: property.original_index,
        });
        sanitized.push(SanitizedPropertyV2 {
            property_ref,
            category: property.category,
            status: property.status,
            evidence_kinds: property.evidence_kinds,
        });
    }
    Ok((sanitized, aliases))
}

fn summarize_statuses(
    properties: &[PolicyPropertyV1],
) -> Result<SanitizedSummaryV2, SuccessorAiError> {
    let mut summary = SanitizedSummaryV2 {
        total: u32::try_from(properties.len())
            .map_err(|_| projection_failure("property count overflowed"))?,
        mpk_verified: 0,
        proof_pending: 0,
        helper_only: 0,
        unsupported: 0,
    };
    for property in properties {
        match parse_status(&property.status)? {
            SourcePropertyStatusV2::MpkVerified => summary.mpk_verified += 1,
            SourcePropertyStatusV2::ProofPending => summary.proof_pending += 1,
            SourcePropertyStatusV2::HelperOnly => summary.helper_only += 1,
            SourcePropertyStatusV2::Unsupported => summary.unsupported += 1,
        }
    }
    Ok(summary)
}

fn summarize_trusted_evidence(
    evidence: &PolicyTrustedEvidenceV1,
    registration: SuccessorAiRegistration,
) -> Result<SanitizedTrustedEvidenceSummaryV2, SuccessorAiError> {
    let mut theory_formats = evidence
        .theory_certificates
        .iter()
        .map(|certificate| map_theory_format(&certificate.format).to_owned())
        .collect::<Vec<_>>();
    theory_formats.sort_by_key(|format| theory_format_rank(format));
    theory_formats.dedup();
    let checker = |name: &str| {
        let rows = evidence
            .checker_verdicts
            .iter()
            .filter(|row| row.checker == name)
            .collect::<Vec<_>>();
        if rows.len() != 1
            || rows[0].checker_profile != registration.checker_profile
            || !matches!(
                rows[0].verdict.as_str(),
                "accepted" | "rejected" | "not_run"
            )
        {
            return Err(projection_failure("checker summary is invalid"));
        }
        Ok(rows[0].verdict.clone())
    };
    let axiom_counts = match &evidence.axiom_report {
        PolicyAxiomReportV1::NotGenerated => None,
        PolicyAxiomReportV1::Checked {
            category_counts, ..
        } => Some(SanitizedAxiomCountsV2 {
            total_axiom_count: category_counts.total_axiom_count,
            core_axiom_count: category_counts.core_axiom_count,
            builtin_theory_axiom_count: category_counts.builtin_theory_axiom_count,
            go_semantics_axiom_count: category_counts.go_semantics_axiom_count,
            external_axiom_count: category_counts.external_axiom_count,
        }),
    };
    Ok(SanitizedTrustedEvidenceSummaryV2 {
        certificate_candidates: u32::try_from(evidence.certificates.len())
            .map_err(|_| projection_failure("certificate count overflowed"))?,
        checked_theory_certificates: u32::try_from(evidence.theory_certificates.len())
            .map_err(|_| projection_failure("theory certificate count overflowed"))?,
        theory_formats,
        rust_fast_kernel: checker("rust_fast_kernel")?,
        reference_checker: checker("reference_checker")?,
        axiom_counts,
    })
}

fn summarize_helpers(
    artifacts: &[PolicyHelperArtifact],
) -> Result<Vec<SanitizedHelperSummaryV2>, SuccessorAiError> {
    let kinds = [
        SanitizedHelperKindV2::Source,
        SanitizedHelperKindV2::Contract,
        SanitizedHelperKindV2::VerificationIr,
        SanitizedHelperKindV2::Vc,
        SanitizedHelperKindV2::AiAnalysis,
        SanitizedHelperKindV2::CiStatus,
    ];
    let mut output = Vec::new();
    for kind in kinds {
        let count = artifacts
            .iter()
            .filter(|artifact| helper_kind(artifact) == kind)
            .count();
        if count > 0 {
            output.push(SanitizedHelperSummaryV2 {
                artifact: kind,
                count: u32::try_from(count)
                    .map_err(|_| projection_failure("helper count overflowed"))?,
            });
        }
    }
    Ok(output)
}

fn build_response_schema(
    aliases: &[PropertyAliasV2],
) -> Result<VertexResponseSchemaV1, SuccessorAiError> {
    let count = u32::try_from(aliases.len())
        .map_err(|_| projection_failure("response alias count overflowed"))?;
    let string_item = || VertexStringSchemaV1 {
        schema_type: "string",
        min_length: Some(1),
        max_length: MAX_GENERATED_ITEM_BYTES as u32,
    };
    Ok(VertexResponseSchemaV1 {
        schema_type: "object",
        properties: VertexResponseSchemaPropertiesV1 {
            overview: VertexStringSchemaV1 {
                schema_type: "string",
                min_length: Some(1),
                max_length: MAX_OVERVIEW_BYTES as u32,
            },
            property_explanations: VertexPropertyExplanationsSchemaV1 {
                schema_type: "array",
                min_items: count,
                max_items: count,
                items: VertexPropertyExplanationSchemaV1 {
                    schema_type: "object",
                    properties: VertexPropertyExplanationPropertiesV1 {
                        property_ref: VertexEnumStringSchemaV1 {
                            schema_type: "string",
                            r#enum: aliases
                                .iter()
                                .map(|alias| alias.property_ref.clone())
                                .collect(),
                        },
                        explanation: string_item(),
                    },
                    required: vec!["property_ref", "explanation"],
                    additional_properties: false,
                },
            },
            limitations: VertexTextListSchemaV1 {
                schema_type: "array",
                max_items: MAX_GENERATED_LIST_ITEMS as u32,
                items: string_item(),
            },
            next_steps: VertexTextListSchemaV1 {
                schema_type: "array",
                max_items: MAX_GENERATED_LIST_ITEMS as u32,
                items: string_item(),
            },
        },
        required: vec![
            "overview",
            "property_explanations",
            "limitations",
            "next_steps",
        ],
        additional_properties: false,
    })
}

fn replace_prompt_payload(payload: &str) -> Result<String, SuccessorAiError> {
    if USER_TEMPLATE_V1.matches(PROMPT_PLACEHOLDER).count() != 1 {
        return Err(projection_failure("prompt template is not closed"));
    }
    Ok(USER_TEMPLATE_V1.replacen(PROMPT_PLACEHOLDER, payload, 1))
}

fn validate_model_response(
    response: &ModelExplanationResponseV0,
    aliases: &[PropertyAliasV2],
) -> Result<(), SuccessorAiError> {
    let mut total = 0;
    validate_generated_text(&response.overview, MAX_OVERVIEW_BYTES, &mut total)?;
    if response.property_explanations.len() != aliases.len()
        || response.limitations.len() > MAX_GENERATED_LIST_ITEMS
        || response.next_steps.len() > MAX_GENERATED_LIST_ITEMS
    {
        return Err(response_failure("provider response cardinality is invalid"));
    }
    let allowed = aliases
        .iter()
        .map(|alias| alias.property_ref.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for property in &response.property_explanations {
        if !allowed.contains(property.property_ref.as_str())
            || !seen.insert(property.property_ref.as_str())
        {
            return Err(response_failure("provider property aliases are invalid"));
        }
        validate_generated_text(&property.explanation, MAX_GENERATED_ITEM_BYTES, &mut total)?;
    }
    for item in response
        .limitations
        .iter()
        .chain(response.next_steps.iter())
    {
        validate_generated_text(item, MAX_GENERATED_ITEM_BYTES, &mut total)?;
    }
    Ok(())
}

fn validate_generated_text(
    value: &str,
    maximum: usize,
    total: &mut usize,
) -> Result<(), SuccessorAiError> {
    if value.trim().is_empty()
        || value.len() > maximum
        || value
            .chars()
            .any(|character| (character.is_control() && character != '\n') || is_bidi(character))
    {
        return Err(response_failure("provider prose is invalid"));
    }
    *total = total.saturating_add(value.len());
    if *total > MAX_TOTAL_AI_TEXT_BYTES {
        return Err(response_failure("provider prose exceeds its closed limit"));
    }
    Ok(())
}

fn validate_provider_provenance(
    provenance: &SuccessorAiProviderProvenance,
) -> Result<(), SuccessorAiError> {
    if !validate_model_version(&provenance.model_version)
        || !validate_token68(provenance.response_id.as_bytes(), 256)
        || !validate_create_time(&provenance.create_time)
        || provenance.finish_reason != "STOP"
        || !(1..=3).contains(&provenance.attempts)
        || [
            provenance.prompt_tokens,
            provenance.thinking_tokens,
            provenance.response_tokens,
            provenance.total_tokens,
        ]
        .into_iter()
        .flatten()
        .any(|count| count > 10_000_000)
    {
        return Err(response_failure("provider provenance is invalid"));
    }
    Ok(())
}

fn valid_project_id(project: &str) -> bool {
    (6..=30).contains(&project.len())
        && project
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
        && project
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && project
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_location(location: &str) -> bool {
    !location.is_empty()
        && location.len() + "-aiplatform".len() <= 63
        && location
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && location
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && location
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_token68(value: &[u8], maximum: usize) -> bool {
    if value.is_empty() || value.len() > maximum {
        return false;
    }
    let mut has_base_character = false;
    let mut padding_started = false;
    for &byte in value {
        if byte == b'=' {
            padding_started = true;
        } else if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
        {
            if padding_started {
                return false;
            }
            has_base_character = true;
        } else {
            return false;
        }
    }
    has_base_character
}

fn validate_model_version(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
}

fn validate_create_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() > 35 || bytes.len() < 20 || !bytes.is_ascii() {
        return false;
    }
    if [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')]
        .into_iter()
        .any(|(index, expected)| bytes.get(index).copied() != Some(expected))
        || ![0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
            .into_iter()
            .all(|range| bytes[range].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    let mut timezone_start = 19;
    if bytes.get(timezone_start) == Some(&b'.') {
        let fraction_start = timezone_start + 1;
        timezone_start = fraction_start;
        while bytes.get(timezone_start).is_some_and(u8::is_ascii_digit) {
            timezone_start += 1;
        }
        if ![3, 6, 9].contains(&(timezone_start - fraction_start)) {
            return false;
        }
    }
    match bytes.get(timezone_start..) {
        Some(timezone) if timezone == b"Z" => true,
        Some(timezone)
            if timezone.len() == 6
                && matches!(timezone[0], b'+' | b'-')
                && timezone[3] == b':'
                && timezone[1..3].iter().all(u8::is_ascii_digit)
                && timezone[4..6].iter().all(u8::is_ascii_digit) =>
        {
            true
        }
        _ => false,
    }
}

fn parse_status(value: &str) -> Result<SourcePropertyStatusV2, SuccessorAiError> {
    match value {
        "mpk_verified" => Ok(SourcePropertyStatusV2::MpkVerified),
        "proof_pending" => Ok(SourcePropertyStatusV2::ProofPending),
        "helper_only" => Ok(SourcePropertyStatusV2::HelperOnly),
        "unsupported" => Ok(SourcePropertyStatusV2::Unsupported),
        _ => Err(projection_failure("property status is invalid")),
    }
}

fn extract_category(description: &str) -> String {
    let Some(token) = description
        .strip_prefix("Payment policy obligation classified as ")
        .and_then(|value| value.strip_suffix('.'))
    else {
        return "unrecognized".to_owned();
    };
    let valid = !token.is_empty()
        && token.len() <= 64
        && token.as_bytes()[0].is_ascii_lowercase()
        && token
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if valid && RECOGNIZED_CATEGORIES.contains(&token) {
        token.to_owned()
    } else {
        "unrecognized".to_owned()
    }
}

fn map_theory_format(value: &str) -> &'static str {
    match value {
        "mpk.bool-normalize.v0" | "mpk.theory.bool.v0" => "bool",
        "mpk.bitvec-ground.v0" => "bitvec",
        "mpk.linarith.v0" => "linarith",
        "mpk.array-read-write.v0" => "array",
        _ => "unrecognized",
    }
}

fn theory_format_rank(value: &str) -> u8 {
    match value {
        "bool" => 0,
        "bitvec" => 1,
        "linarith" => 2,
        "array" => 3,
        _ => 4,
    }
}

fn category_rank(category: &str) -> usize {
    RECOGNIZED_CATEGORIES
        .iter()
        .position(|candidate| *candidate == category)
        .unwrap_or(RECOGNIZED_CATEGORIES.len())
}

fn status_rank(status: SourcePropertyStatusV2) -> u8 {
    match status {
        SourcePropertyStatusV2::MpkVerified => 0,
        SourcePropertyStatusV2::ProofPending => 1,
        SourcePropertyStatusV2::HelperOnly => 2,
        SourcePropertyStatusV2::Unsupported => 3,
    }
}

fn evidence_kind_rank(kind: SanitizedEvidenceKindV2) -> u8 {
    match kind {
        SanitizedEvidenceKindV2::CheckedDeclaration => 0,
        SanitizedEvidenceKindV2::CheckedTheoryCertificate => 1,
        SanitizedEvidenceKindV2::HelperArtifact => 2,
        SanitizedEvidenceKindV2::UnsupportedFeature => 3,
    }
}

fn evidence_bitset(kinds: &[SanitizedEvidenceKindV2]) -> u8 {
    kinds
        .iter()
        .fold(0, |bits, kind| bits | (1_u8 << evidence_kind_rank(*kind)))
}

fn helper_kind(artifact: &PolicyHelperArtifact) -> SanitizedHelperKindV2 {
    match artifact {
        PolicyHelperArtifact::Source { .. } => SanitizedHelperKindV2::Source,
        PolicyHelperArtifact::Contract { .. } => SanitizedHelperKindV2::Contract,
        PolicyHelperArtifact::VerificationIr { .. } => SanitizedHelperKindV2::VerificationIr,
        PolicyHelperArtifact::Vc { .. } => SanitizedHelperKindV2::Vc,
        PolicyHelperArtifact::AiAnalysis { .. } => SanitizedHelperKindV2::AiAnalysis,
        PolicyHelperArtifact::CiStatus { .. } => SanitizedHelperKindV2::CiStatus,
    }
}

fn is_forbidden_character(character: char) -> bool {
    character.is_control() || is_bidi(character)
}

fn is_bidi(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_document<T: Serialize>(
    value: &T,
    maximum: usize,
    phase: SuccessorAiPhase,
    code: SuccessorAiCode,
    detail: &'static str,
) -> Result<Vec<u8>, SuccessorAiError> {
    let maximum_u64 = u64::try_from(maximum).map_err(|_| failure(phase, code, detail))?;
    let serialized =
        serialize_json_bounded(value, maximum).map_err(|_| failure(phase, code, detail))?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(maximum_u64, maximum_u64, MAX_JSON_NESTING, maximum_u64),
    )
    .map_err(|_| failure(phase, code, detail))?;
    canonical_json_bytes_bounded(&strict, maximum).map_err(|_| failure(phase, code, detail))
}

fn validate_exact_document<T: Serialize>(
    input: &[u8],
    expected: &T,
    maximum: usize,
    detail: &'static str,
) -> Result<(), SuccessorAiError> {
    let maximum_u64 = u64::try_from(maximum)
        .map_err(|_| failure(SuccessorAiPhase::Transport, SuccessorAiCode::Json, detail))?;
    let strict = parse_strict_json(
        input,
        StrictJsonLimits::new(maximum_u64, maximum_u64, MAX_JSON_NESTING, maximum_u64),
    )
    .map_err(|_| failure(SuccessorAiPhase::Transport, SuccessorAiCode::Json, detail))?;
    let canonical = canonical_json_bytes_bounded(&strict, maximum).map_err(|_| {
        failure(
            SuccessorAiPhase::CanonicalTransport,
            SuccessorAiCode::CanonicalTransport,
            detail,
        )
    })?;
    let expected = canonical_document(
        expected,
        maximum,
        SuccessorAiPhase::CanonicalTransport,
        SuccessorAiCode::CanonicalTransport,
        detail,
    )?;
    if input != canonical || canonical != expected {
        return Err(failure(
            SuccessorAiPhase::CanonicalTransport,
            SuccessorAiCode::CanonicalTransport,
            detail,
        ));
    }
    Ok(())
}

fn projection_failure(detail: &'static str) -> SuccessorAiError {
    failure(
        SuccessorAiPhase::RedactionProjection,
        SuccessorAiCode::Projection,
        detail,
    )
}

fn response_failure(detail: &'static str) -> SuccessorAiError {
    failure(
        SuccessorAiPhase::ProviderResponse,
        SuccessorAiCode::ResponseInvalid,
        detail,
    )
}

const fn failure(
    phase: SuccessorAiPhase,
    code: SuccessorAiCode,
    detail: &'static str,
) -> SuccessorAiError {
    SuccessorAiError {
        phase,
        code,
        detail,
    }
}

// Private v3 linkage-only explanation documents for CSHARP-03-T02-W09.
// They never invoke a provider and are not reachable from the installed CLI.
pub(crate) const PRIVATE_AI_REQUEST_SCHEMA: &str = "mpk.ai.explain.request.v3";
pub(crate) const PRIVATE_AI_EXPLANATION_SCHEMA: &str = "mpk.ai.explanation.v3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PrivateAiArtifacts {
    request: Vec<u8>,
    explanation: Vec<u8>,
    request_sha256: String,
    explanation_sha256: String,
}

impl PrivateAiArtifacts {
    pub(crate) fn request(&self) -> &[u8] {
        &self.request
    }

    pub(crate) fn explanation(&self) -> &[u8] {
        &self.explanation
    }

    pub(crate) fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub(crate) fn explanation_sha256(&self) -> &str {
        &self.explanation_sha256
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PrivateAiRequest {
    schema: String,
    semantic_context: Value,
    policy_evidence_sha256: String,
    policy_receipt_sha256: String,
    redaction: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PrivateAiExplanation {
    schema: String,
    semantic_context: Value,
    request_sha256: String,
    status: String,
    proof_authority: String,
}

pub(crate) fn build_private_successor_ai_artifacts(
    semantic_context: &Value,
    policy_evidence_sha256: &str,
    policy_receipt_sha256: &str,
) -> Result<PrivateAiArtifacts, &'static str> {
    if !private_ai_context(semantic_context)
        || !private_ai_sha256(policy_evidence_sha256)
        || !private_ai_sha256(policy_receipt_sha256)
    {
        return Err("private AI linkage");
    }
    let request = PrivateAiRequest {
        schema: PRIVATE_AI_REQUEST_SCHEMA.to_owned(),
        semantic_context: semantic_context.clone(),
        policy_evidence_sha256: policy_evidence_sha256.to_owned(),
        policy_receipt_sha256: policy_receipt_sha256.to_owned(),
        redaction: "closed_sanitized_projection".to_owned(),
    };
    let request = private_ai_encode(&request)?;
    let request_sha256 = mpk_vc::sha256_raw_file_bytes(&request).to_hex();
    let explanation = PrivateAiExplanation {
        schema: PRIVATE_AI_EXPLANATION_SCHEMA.to_owned(),
        semantic_context: semantic_context.clone(),
        request_sha256: request_sha256.clone(),
        status: "private_local_summary".to_owned(),
        proof_authority: "none".to_owned(),
    };
    let explanation = private_ai_encode(&explanation)?;
    let explanation_sha256 = mpk_vc::sha256_raw_file_bytes(&explanation).to_hex();
    validate_private_successor_ai_artifacts(&request, &explanation)?;
    Ok(PrivateAiArtifacts {
        request,
        explanation,
        request_sha256,
        explanation_sha256,
    })
}

pub(crate) fn validate_private_successor_ai_artifacts(
    request: &[u8],
    explanation: &[u8],
) -> Result<(), &'static str> {
    let request: PrivateAiRequest = private_ai_decode(request)?;
    let explanation: PrivateAiExplanation = private_ai_decode(explanation)?;
    if request.schema != PRIVATE_AI_REQUEST_SCHEMA
        || explanation.schema != PRIVATE_AI_EXPLANATION_SCHEMA
        || !private_ai_context(&request.semantic_context)
        || request.semantic_context != explanation.semantic_context
        || !private_ai_sha256(&request.policy_evidence_sha256)
        || !private_ai_sha256(&request.policy_receipt_sha256)
        || explanation.request_sha256
            != mpk_vc::sha256_raw_file_bytes(&private_ai_encode(&request)?).to_hex()
        || request.redaction != "closed_sanitized_projection"
        || explanation.status != "private_local_summary"
        || explanation.proof_authority != "none"
    {
        return Err("private AI document linkage");
    }
    Ok(())
}

fn private_ai_encode<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json::to_vec(value).map_err(|_| "private AI transport")
}

fn private_ai_context(value: &Value) -> bool {
    serde_json::to_vec(value).is_ok_and(|transport| {
        mpk_vc::csharp_practical_registry::validate_successor_registry_document(
            mpk_vc::csharp_practical_registry::SuccessorRegistryDocumentKind::SemanticContext,
            &transport,
        )
        .is_ok()
    })
}

fn private_ai_decode<T: for<'de> Deserialize<'de> + Serialize>(
    input: &[u8],
) -> Result<T, &'static str> {
    parse_strict_json(
        input,
        StrictJsonLimits::new(1_048_576, 1_048_576, 32, 1_048_576),
    )
    .map_err(|_| "private AI transport")?;
    let value = serde_json::from_slice(input).map_err(|_| "private AI transport")?;
    if private_ai_encode(&value)? != input {
        return Err("private AI canonical transport");
    }
    Ok(value)
}

fn private_ai_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
