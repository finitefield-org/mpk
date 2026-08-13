//! Foundation types for the optional Vertex AI evidence explainer.
//!
//! This module intentionally contains schemas and trust-boundary types only.
//! Input projection, authentication, transport, model-response validation, and
//! output-file orchestration belong to later implementation tasks.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const AI_EXPLAIN_REQUEST_SCHEMA: &str = "mpk.ai.explain.request.v0";
pub const AI_EXPLANATION_SCHEMA: &str = "mpk.ai.explanation.v0";
pub const AI_EXPLANATION_RESPONSE_SCHEMA: &str = "mpk.ai.explanation.response.v0";
pub const MINIMAL_REDACTION_PROFILE: &str = "minimal-v0";
pub const PROMPT_TEMPLATE_ID: &str = "mpk.evidence-explainer.v0";
pub const VERTEX_AI_PROVIDER: &str = "vertex-ai";
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.5-flash";
pub const TRUST_CLASSIFICATION: &str = "untrusted_helper_analysis";
pub const TRUST_DISCLAIMER: &str =
    "AI-generated explanation. Verification status is determined only by MPK evidence and checkers.";

/// The provider selector is deliberately an enum: adding a provider requires
/// an explicit review instead of accepting an arbitrary provider string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplainProvider {
    #[serde(rename = "vertex-ai")]
    VertexAi,
}

impl ExplainProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VertexAi => VERTEX_AI_PROVIDER,
        }
    }
}

/// Supported explanation languages are fixed for the v0 contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplainLanguage {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
}

impl ExplainLanguage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Japanese => "ja",
        }
    }
}

/// The local command request contains configuration and paths only. It has no
/// credential, token, endpoint override, or model-controlled proof field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainRequest {
    pub evidence_path: PathBuf,
    pub provider: ExplainProvider,
    pub project: String,
    pub location: String,
    pub model: String,
    pub language: ExplainLanguage,
    pub output_json: PathBuf,
    pub output_markdown: PathBuf,
    pub overwrite: bool,
}

/// The typed shape of a future Vertex request. Building and sending this
/// request are intentionally deferred to GEMINI-AUX-02 and GEMINI-AUX-03.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VertexGenerateRequest {
    #[serde(rename = "systemInstruction")]
    pub system_instruction: VertexContent,
    pub contents: Vec<VertexContent>,
    #[serde(rename = "generationConfig")]
    pub generation_config: VertexGenerationConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VertexContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<VertexPart>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VertexPart {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VertexGenerationConfig {
    #[serde(rename = "candidateCount")]
    pub candidate_count: u8,
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
    #[serde(rename = "responseMimeType")]
    pub response_mime_type: String,
    #[serde(rename = "responseSchema")]
    pub response_schema: serde_json::Value,
}

/// Provider envelopes remain forward-compatible with fields added by Google;
/// the later transport/parser task extracts only the fields in this type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexGenerateResponse {
    pub candidates: Vec<VertexCandidate>,
    pub usage_metadata: Option<VertexUsageMetadata>,
    pub response_id: Option<String>,
    pub model_version: Option<String>,
    pub create_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexCandidate {
    pub content: Option<VertexResponseContent>,
    pub finish_reason: Option<String>,
    pub index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexResponseContent {
    pub role: Option<String>,
    pub parts: Vec<VertexResponsePart>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VertexResponsePart {
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexUsageMetadata {
    pub prompt_token_count: Option<u64>,
    pub thoughts_token_count: Option<u64>,
    pub candidates_token_count: Option<u64>,
    pub total_token_count: Option<u64>,
}

/// This is the only shape accepted from the model. In particular, it has no
/// status, verdict, certificate, hash, or trusted-evidence field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelExplanationResponse {
    pub overview: String,
    pub property_explanations: Vec<ModelPropertyExplanation>,
    pub limitations: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelPropertyExplanation {
    pub property_ref: String,
    pub explanation: String,
}

/// A closed trust classification. There is no trusted or proof-evidence
/// variant in the assistant output type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrustClassification {
    #[serde(rename = "untrusted_helper_analysis")]
    UntrustedHelperAnalysis,
}

impl TrustClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UntrustedHelperAnalysis => TRUST_CLASSIFICATION,
        }
    }
}

/// Constructing this type always sets `proof_evidence` to false. It is not
/// deserializable, so a model response cannot manufacture or alter the trust
/// label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrustLabel {
    classification: TrustClassification,
    proof_evidence: bool,
    disclaimer: &'static str,
}

impl TrustLabel {
    pub const fn untrusted_helper_analysis() -> Self {
        Self {
            classification: TrustClassification::UntrustedHelperAnalysis,
            proof_evidence: false,
            disclaimer: TRUST_DISCLAIMER,
        }
    }

    pub const fn classification(&self) -> TrustClassification {
        self.classification
    }

    pub const fn proof_evidence(&self) -> bool {
        self.proof_evidence
    }

    pub const fn disclaimer(&self) -> &'static str {
        self.disclaimer
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorMetadata {
    pub name: String,
    pub version: String,
}

impl GeneratorMetadata {
    pub fn current() -> Self {
        Self {
            name: "mpk".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceEvidenceReference {
    pub schema: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExplainOutputRequest {
    pub provider: ExplainProvider,
    pub project: String,
    pub location: String,
    pub requested_model: String,
    pub language: ExplainLanguage,
    pub redaction_profile: String,
    pub prompt_template: String,
    pub prompt_template_sha256: String,
    pub response_schema: String,
    pub response_schema_sha256: String,
    pub sanitized_payload_sha256: String,
    pub request_body_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProviderFinishReason {
    #[serde(rename = "STOP")]
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct AttemptCount(u8);

impl AttemptCount {
    pub fn new(value: u8) -> Result<Self, AiExplainError> {
        if (1..=3).contains(&value) {
            Ok(Self(value))
        } else {
            Err(AiExplainError::new(
                AiExplainErrorCode::AiExplainResponseInvalid,
                "attempt count must be between 1 and 3",
            ))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderUsage {
    pub prompt_tokens: Option<u64>,
    pub thinking_tokens: Option<u64>,
    pub response_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl ProviderUsage {
    pub const fn empty() -> Self {
        Self {
            prompt_tokens: None,
            thinking_tokens: None,
            response_tokens: None,
            total_tokens: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderProvenance {
    pub model_version: String,
    pub response_id: String,
    pub create_time: String,
    pub finish_reason: ProviderFinishReason,
    pub attempts: AttemptCount,
    #[serde(flatten)]
    pub usage: ProviderUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePropertyStatus {
    MpkVerified,
    ProofPending,
    HelperOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalSummary {
    pub strategy_profile: String,
    pub checker_profile: String,
    pub allowed_axiom_profiles: Vec<String>,
    pub total: u32,
    pub mpk_verified: u32,
    pub proof_pending: u32,
    pub helper_only: u32,
    pub unsupported: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiPropertyExplanation {
    pub property_id: String,
    pub source_status: SourcePropertyStatus,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiAnalysis {
    pub overview: String,
    pub property_explanations: Vec<AiPropertyExplanation>,
    pub limitations: Vec<String>,
    pub next_steps: Vec<String>,
}

/// The final output is constructed locally from validated evidence and a
/// separately validated model response. The model response type above cannot
/// be converted into this report without an explicit local mapping step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiExplanationReport {
    pub schema: String,
    pub generator: GeneratorMetadata,
    pub trust: TrustLabel,
    pub source_evidence: SourceEvidenceReference,
    pub request: ExplainOutputRequest,
    pub provider_response: ProviderProvenance,
    pub local_summary: LocalSummary,
    pub ai_analysis: AiAnalysis,
}

impl AiExplanationReport {
    pub fn new(
        source_evidence: SourceEvidenceReference,
        request: ExplainOutputRequest,
        provider_response: ProviderProvenance,
        local_summary: LocalSummary,
        ai_analysis: AiAnalysis,
    ) -> Self {
        Self {
            schema: AI_EXPLANATION_SCHEMA.to_owned(),
            generator: GeneratorMetadata::current(),
            trust: TrustLabel::untrusted_helper_analysis(),
            source_evidence,
            request,
            provider_response,
            local_summary,
            ai_analysis,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AiExplainErrorCode {
    #[serde(rename = "AI_EXPLAIN_INPUT_UNAVAILABLE")]
    AiExplainInputUnavailable,
    #[serde(rename = "AI_EXPLAIN_INPUT_TOO_LARGE")]
    AiExplainInputTooLarge,
    #[serde(rename = "AI_EXPLAIN_NO_PROPERTIES")]
    AiExplainNoProperties,
    #[serde(rename = "AI_EXPLAIN_TOO_MANY_PROPERTIES")]
    AiExplainTooManyProperties,
    #[serde(rename = "AI_EXPLAIN_INVALID_EVIDENCE")]
    AiExplainInvalidEvidence,
    #[serde(rename = "AI_EXPLAIN_PAYLOAD_TOO_LARGE")]
    AiExplainPayloadTooLarge,
    #[serde(rename = "VERTEX_CONFIG_INVALID")]
    VertexConfigInvalid,
    #[serde(rename = "VERTEX_AUTH_UNAVAILABLE")]
    VertexAuthUnavailable,
    #[serde(rename = "VERTEX_AUTH_FAILED")]
    VertexAuthFailed,
    #[serde(rename = "VERTEX_PERMISSION_DENIED")]
    VertexPermissionDenied,
    #[serde(rename = "VERTEX_NOT_FOUND")]
    VertexNotFound,
    #[serde(rename = "VERTEX_REQUEST_FAILED")]
    VertexRequestFailed,
    #[serde(rename = "VERTEX_RATE_LIMITED")]
    VertexRateLimited,
    #[serde(rename = "VERTEX_TIMEOUT")]
    VertexTimeout,
    #[serde(rename = "VERTEX_TRANSPORT_FAILED")]
    VertexTransportFailed,
    #[serde(rename = "VERTEX_UNAVAILABLE")]
    VertexUnavailable,
    #[serde(rename = "VERTEX_RESPONSE_BLOCKED")]
    VertexResponseBlocked,
    #[serde(rename = "VERTEX_PROTOCOL_ERROR")]
    VertexProtocolError,
    #[serde(rename = "AI_EXPLAIN_RESPONSE_INVALID")]
    AiExplainResponseInvalid,
    #[serde(rename = "AI_EXPLAIN_OUTPUT_FAILED")]
    AiExplainOutputFailed,
}

impl AiExplainErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AiExplainInputUnavailable => "AI_EXPLAIN_INPUT_UNAVAILABLE",
            Self::AiExplainInputTooLarge => "AI_EXPLAIN_INPUT_TOO_LARGE",
            Self::AiExplainNoProperties => "AI_EXPLAIN_NO_PROPERTIES",
            Self::AiExplainTooManyProperties => "AI_EXPLAIN_TOO_MANY_PROPERTIES",
            Self::AiExplainInvalidEvidence => "AI_EXPLAIN_INVALID_EVIDENCE",
            Self::AiExplainPayloadTooLarge => "AI_EXPLAIN_PAYLOAD_TOO_LARGE",
            Self::VertexConfigInvalid => "VERTEX_CONFIG_INVALID",
            Self::VertexAuthUnavailable => "VERTEX_AUTH_UNAVAILABLE",
            Self::VertexAuthFailed => "VERTEX_AUTH_FAILED",
            Self::VertexPermissionDenied => "VERTEX_PERMISSION_DENIED",
            Self::VertexNotFound => "VERTEX_NOT_FOUND",
            Self::VertexRequestFailed => "VERTEX_REQUEST_FAILED",
            Self::VertexRateLimited => "VERTEX_RATE_LIMITED",
            Self::VertexTimeout => "VERTEX_TIMEOUT",
            Self::VertexTransportFailed => "VERTEX_TRANSPORT_FAILED",
            Self::VertexUnavailable => "VERTEX_UNAVAILABLE",
            Self::VertexResponseBlocked => "VERTEX_RESPONSE_BLOCKED",
            Self::VertexProtocolError => "VERTEX_PROTOCOL_ERROR",
            Self::AiExplainResponseInvalid => "AI_EXPLAIN_RESPONSE_INVALID",
            Self::AiExplainOutputFailed => "AI_EXPLAIN_OUTPUT_FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiExplainError {
    code: AiExplainErrorCode,
    detail: String,
}

impl AiExplainError {
    pub fn new(code: AiExplainErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub const fn code(&self) -> AiExplainErrorCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for AiExplainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl Error for AiExplainError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_and_policy_identifiers_are_pinned() {
        assert_eq!(AI_EXPLAIN_REQUEST_SCHEMA, "mpk.ai.explain.request.v0");
        assert_eq!(AI_EXPLANATION_SCHEMA, "mpk.ai.explanation.v0");
        assert_eq!(
            AI_EXPLANATION_RESPONSE_SCHEMA,
            "mpk.ai.explanation.response.v0"
        );
        assert_eq!(MINIMAL_REDACTION_PROFILE, "minimal-v0");
        assert_eq!(PROMPT_TEMPLATE_ID, "mpk.evidence-explainer.v0");
        assert_eq!(VERTEX_AI_PROVIDER, "vertex-ai");
    }

    #[test]
    fn trust_label_is_always_untrusted_and_not_proof_evidence() {
        let label = TrustLabel::untrusted_helper_analysis();
        assert_eq!(label.classification().as_str(), TRUST_CLASSIFICATION);
        assert!(!label.proof_evidence());

        let value = serde_json::to_value(&label).expect("trust label serializes");
        assert_eq!(
            value,
            json!({
                "classification": "untrusted_helper_analysis",
                "proof_evidence": false,
                "disclaimer": TRUST_DISCLAIMER
            })
        );
    }

    #[test]
    fn model_response_rejects_status_and_trusted_evidence_fields() {
        let response = json!({
            "overview": "safe helper text",
            "property_explanations": [{
                "property_ref": "property-0001",
                "explanation": "safe helper text"
            }],
            "limitations": [],
            "next_steps": [],
            "status": "mpk_verified",
            "trusted_evidence": []
        });

        assert!(serde_json::from_value::<ModelExplanationResponse>(response).is_err());
    }

    #[test]
    fn output_report_injects_local_trust_label() {
        let report = AiExplanationReport::new(
            SourceEvidenceReference {
                schema: "mpk.policy.evidence.v0".to_owned(),
                sha256: "a".repeat(64),
            },
            ExplainOutputRequest {
                provider: ExplainProvider::VertexAi,
                project: "example-project".to_owned(),
                location: "global".to_owned(),
                requested_model: DEFAULT_GEMINI_MODEL.to_owned(),
                language: ExplainLanguage::English,
                redaction_profile: MINIMAL_REDACTION_PROFILE.to_owned(),
                prompt_template: PROMPT_TEMPLATE_ID.to_owned(),
                prompt_template_sha256: "b".repeat(64),
                response_schema: AI_EXPLANATION_RESPONSE_SCHEMA.to_owned(),
                response_schema_sha256: "c".repeat(64),
                sanitized_payload_sha256: "d".repeat(64),
                request_body_sha256: "e".repeat(64),
            },
            ProviderProvenance {
                model_version: "gemini-3.5-flash-001".to_owned(),
                response_id: "response-1".to_owned(),
                create_time: "2026-08-14T00:00:00Z".to_owned(),
                finish_reason: ProviderFinishReason::Stop,
                attempts: AttemptCount::new(1).expect("valid attempt count"),
                usage: ProviderUsage::empty(),
            },
            LocalSummary {
                strategy_profile: "payment-policy-alpha".to_owned(),
                checker_profile: "mvp-strict".to_owned(),
                allowed_axiom_profiles: vec!["zero-axiom".to_owned()],
                total: 1,
                mpk_verified: 1,
                proof_pending: 0,
                helper_only: 0,
                unsupported: 0,
            },
            AiAnalysis {
                overview: "helper".to_owned(),
                property_explanations: vec![AiPropertyExplanation {
                    property_id: "local-property".to_owned(),
                    source_status: SourcePropertyStatus::MpkVerified,
                    explanation: "helper".to_owned(),
                }],
                limitations: Vec::new(),
                next_steps: Vec::new(),
            },
        );

        let value = serde_json::to_value(report).expect("report serializes");
        assert_eq!(value["trust"]["classification"], TRUST_CLASSIFICATION);
        assert_eq!(value["trust"]["proof_evidence"], false);
        assert_eq!(value["provider_response"]["finish_reason"], "STOP");
    }

    #[test]
    fn attempt_count_is_bounded() {
        assert!(AttemptCount::new(0).is_err());
        assert_eq!(AttemptCount::new(1).expect("one attempt").get(), 1);
        assert_eq!(AttemptCount::new(3).expect("three attempts").get(), 3);
        assert!(AttemptCount::new(4).is_err());
    }

    #[test]
    fn error_code_strings_are_stable() {
        assert_eq!(
            AiExplainErrorCode::VertexAuthFailed.as_str(),
            "VERTEX_AUTH_FAILED"
        );
        assert_eq!(
            AiExplainErrorCode::AiExplainOutputFailed.as_str(),
            "AI_EXPLAIN_OUTPUT_FAILED"
        );
        let error = AiExplainError::new(
            AiExplainErrorCode::VertexProtocolError,
            "provider response was incomplete",
        );
        assert_eq!(
            error.to_string(),
            "VERTEX_PROTOCOL_ERROR: provider response was incomplete"
        );
    }
}
