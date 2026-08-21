//! AI API v1 route and transport boundary.
//!
//! This is the active source-program API. It accepts validated VIR artifacts;
//! the removed predecessor source-import route has no compatibility adapter.

use std::fmt;

use mpk_vc::{canonical_json_bytes, parse_strict_json, StrictJsonLimits};
use serde::{de::DeserializeOwned, Serialize};

pub const AI_API_V1_PROFILE: &str = "mpk.ai.api.v1";
pub const API_REJECTION_MESSAGE: &str = "AI API v1 request rejected";

const API_ENVELOPE_OVERHEAD: u64 = 1_048_576;
const API_WRAPPER_LEVELS: u64 = 257;
const API_STRING_BYTES_MAX: u64 = 1_048_576;
const API_TRANSPORT_BYTES_MAX: u64 = 269_484_032;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V1ValidationPhase {
    Route,
    Transport,
    Shape,
    Scalar,
    Session,
    Artifact,
    Context,
    CanonicalTransport,
}

impl V1ValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Route => "route",
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Session => "session",
            Self::Artifact => "artifact",
            Self::Context => "context",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum V1ErrorCode {
    #[serde(rename = "AI_API_ROUTE_UNKNOWN")]
    RouteUnknown,
    #[serde(rename = "AI_API_JSON_INVALID")]
    JsonInvalid,
    #[serde(rename = "AI_API_SHAPE")]
    Shape,
    #[serde(rename = "AI_API_SCALAR")]
    Scalar,
    #[serde(rename = "UNKNOWN_SESSION")]
    UnknownSession,
    #[serde(rename = "AI_API_SESSION_STATE")]
    SessionState,
    #[serde(rename = "AI_API_VIR_SCHEMA")]
    VirSchema,
    #[serde(rename = "AI_API_VIR_INVALID")]
    VirInvalid,
    #[serde(rename = "AI_API_VIR_HASH")]
    VirHash,
    #[serde(rename = "AI_API_SOURCE_CONTEXT_UNKNOWN")]
    SourceContextUnknown,
    #[serde(rename = "AI_API_SOURCE_MANIFEST_SCHEMA")]
    SourceManifestSchema,
    #[serde(rename = "AI_API_SOURCE_MANIFEST_INVALID")]
    SourceManifestInvalid,
    #[serde(rename = "AI_API_SOURCE_MANIFEST_HASH")]
    SourceManifestHash,
    #[serde(rename = "AI_API_VC_INVALID")]
    VcInvalid,
    #[serde(rename = "AI_API_CONTEXT_MISMATCH")]
    ContextMismatch,
    #[serde(rename = "AI_API_TARGET_UNKNOWN")]
    TargetUnknown,
    #[serde(rename = "UNKNOWN_PROOF")]
    UnknownProof,
    #[serde(rename = "AI_API_CANDIDATE_UNKNOWN")]
    CandidateUnknown,
    #[serde(rename = "AI_API_CANONICAL_TRANSPORT")]
    CanonicalTransport,
}

impl V1ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteUnknown => "AI_API_ROUTE_UNKNOWN",
            Self::JsonInvalid => "AI_API_JSON_INVALID",
            Self::Shape => "AI_API_SHAPE",
            Self::Scalar => "AI_API_SCALAR",
            Self::UnknownSession => "UNKNOWN_SESSION",
            Self::SessionState => "AI_API_SESSION_STATE",
            Self::VirSchema => "AI_API_VIR_SCHEMA",
            Self::VirInvalid => "AI_API_VIR_INVALID",
            Self::VirHash => "AI_API_VIR_HASH",
            Self::SourceContextUnknown => "AI_API_SOURCE_CONTEXT_UNKNOWN",
            Self::SourceManifestSchema => "AI_API_SOURCE_MANIFEST_SCHEMA",
            Self::SourceManifestInvalid => "AI_API_SOURCE_MANIFEST_INVALID",
            Self::SourceManifestHash => "AI_API_SOURCE_MANIFEST_HASH",
            Self::VcInvalid => "AI_API_VC_INVALID",
            Self::ContextMismatch => "AI_API_CONTEXT_MISMATCH",
            Self::TargetUnknown => "AI_API_TARGET_UNKNOWN",
            Self::UnknownProof => "UNKNOWN_PROOF",
            Self::CandidateUnknown => "AI_API_CANDIDATE_UNKNOWN",
            Self::CanonicalTransport => "AI_API_CANONICAL_TRANSPORT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct V1ApiError {
    pub code: V1ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip)]
    phase: V1ValidationPhase,
}

impl V1ApiError {
    pub fn new(
        phase: V1ValidationPhase,
        code: V1ErrorCode,
        field: Option<&'static str>,
        detail: Option<impl Into<String>>,
    ) -> Self {
        Self {
            code,
            message: API_REJECTION_MESSAGE.to_owned(),
            field,
            detail: detail.map(Into::into),
            phase,
        }
    }

    pub fn inherited(
        phase: V1ValidationPhase,
        code: V1ErrorCode,
        message: impl Into<String>,
        field: &'static str,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            field: Some(field),
            detail: None,
            phase,
        }
    }

    pub const fn phase(&self) -> V1ValidationPhase {
        self.phase
    }
}

impl fmt::Display for V1ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} during {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl std::error::Error for V1ApiError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum V1Handler {
    ModuleNew,
    ModuleImport,
    ModuleFreeze,
    ModuleExportCertificate,
    TermSort,
    TermVar,
    TermConst,
    TermApp,
    TermLam,
    TermPi,
    TermLet,
    TermCheck,
    TermInfer,
    TermDefeq,
    ProofExact,
    ProofApply,
    ProofIntro,
    ProofRefl,
    ProofLet,
    ProofRewrite,
    ProofEqRec,
    ProofConstructor,
    ProofRecursor,
    ProofConv,
    ProofTheory,
    ProofCheckNode,
    ProofCheckDecl,
    VirImport,
    VcGenerate,
    VcList,
    VcStartProof,
    VcAttachCandidate,
    VcCheckCandidate,
}

impl V1Handler {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModuleNew => "module_new",
            Self::ModuleImport => "module_import",
            Self::ModuleFreeze => "module_freeze",
            Self::ModuleExportCertificate => "module_export_certificate",
            Self::TermSort => "term_sort",
            Self::TermVar => "term_var",
            Self::TermConst => "term_const",
            Self::TermApp => "term_app",
            Self::TermLam => "term_lam",
            Self::TermPi => "term_pi",
            Self::TermLet => "term_let",
            Self::TermCheck => "term_check",
            Self::TermInfer => "term_infer",
            Self::TermDefeq => "term_defeq",
            Self::ProofExact => "proof_exact",
            Self::ProofApply => "proof_apply",
            Self::ProofIntro => "proof_intro",
            Self::ProofRefl => "proof_refl",
            Self::ProofLet => "proof_let",
            Self::ProofRewrite => "proof_rewrite",
            Self::ProofEqRec => "proof_eq_rec",
            Self::ProofConstructor => "proof_constructor",
            Self::ProofRecursor => "proof_recursor",
            Self::ProofConv => "proof_conv",
            Self::ProofTheory => "proof_theory",
            Self::ProofCheckNode => "proof_check_node",
            Self::ProofCheckDecl => "proof_check_decl",
            Self::VirImport => "vir_import",
            Self::VcGenerate => "vc_generate",
            Self::VcList => "vc_list",
            Self::VcStartProof => "vc_start_proof",
            Self::VcAttachCandidate => "vc_attach_candidate",
            Self::VcCheckCandidate => "vc_check_candidate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct V1Route {
    pub method: &'static str,
    pub path: &'static str,
    pub handler: V1Handler,
}

pub const V1_ROUTES: &[V1Route] = &[
    route("POST", "/module/new", V1Handler::ModuleNew),
    route("POST", "/module/import", V1Handler::ModuleImport),
    route("POST", "/module/freeze", V1Handler::ModuleFreeze),
    route(
        "POST",
        "/module/export-certificate",
        V1Handler::ModuleExportCertificate,
    ),
    route("POST", "/term/sort", V1Handler::TermSort),
    route("POST", "/term/var", V1Handler::TermVar),
    route("POST", "/term/const", V1Handler::TermConst),
    route("POST", "/term/app", V1Handler::TermApp),
    route("POST", "/term/lam", V1Handler::TermLam),
    route("POST", "/term/pi", V1Handler::TermPi),
    route("POST", "/term/let", V1Handler::TermLet),
    route("POST", "/term/check", V1Handler::TermCheck),
    route("POST", "/term/infer", V1Handler::TermInfer),
    route("POST", "/term/defeq", V1Handler::TermDefeq),
    route("POST", "/proof/exact", V1Handler::ProofExact),
    route("POST", "/proof/apply", V1Handler::ProofApply),
    route("POST", "/proof/intro", V1Handler::ProofIntro),
    route("POST", "/proof/refl", V1Handler::ProofRefl),
    route("POST", "/proof/let", V1Handler::ProofLet),
    route("POST", "/proof/rewrite", V1Handler::ProofRewrite),
    route("POST", "/proof/eq-rec", V1Handler::ProofEqRec),
    route("POST", "/proof/constructor", V1Handler::ProofConstructor),
    route("POST", "/proof/recursor", V1Handler::ProofRecursor),
    route("POST", "/proof/conv", V1Handler::ProofConv),
    route("POST", "/proof/theory", V1Handler::ProofTheory),
    route("POST", "/proof/check-node", V1Handler::ProofCheckNode),
    route("POST", "/proof/check-decl", V1Handler::ProofCheckDecl),
    route("POST", "/vir/import", V1Handler::VirImport),
    route("POST", "/vc/generate", V1Handler::VcGenerate),
    route("GET", "/vc/list", V1Handler::VcList),
    route("POST", "/vc/start-proof", V1Handler::VcStartProof),
    route("POST", "/vc/attach-candidate", V1Handler::VcAttachCandidate),
    route("POST", "/vc/check-candidate", V1Handler::VcCheckCandidate),
];

const fn route(method: &'static str, path: &'static str, handler: V1Handler) -> V1Route {
    V1Route {
        method,
        path,
        handler,
    }
}

pub fn resolve_route(method: &str, path: &str) -> Result<V1Handler, V1ApiError> {
    V1_ROUTES
        .iter()
        .find(|route| route.method == method && route.path == path)
        .map(|route| route.handler)
        .ok_or_else(|| {
            V1ApiError::new(
                V1ValidationPhase::Route,
                V1ErrorCode::RouteUnknown,
                None,
                None::<String>,
            )
        })
}

pub struct ParsedRequest<T> {
    pub value: T,
    canonical_transport: bool,
}

impl<T> ParsedRequest<T> {
    pub fn require_canonical(&self) -> Result<(), V1ApiError> {
        if self.canonical_transport {
            Ok(())
        } else {
            Err(V1ApiError::new(
                V1ValidationPhase::CanonicalTransport,
                V1ErrorCode::CanonicalTransport,
                None,
                None::<String>,
            ))
        }
    }
}

pub fn parse_request<T: DeserializeOwned>(
    input: &[u8],
    embedded_bytes_max: u64,
) -> Result<ParsedRequest<T>, V1ApiError> {
    let transport_max = embedded_bytes_max
        .checked_add(API_ENVELOPE_OVERHEAD)
        .ok_or_else(json_invalid)?;
    if u64::try_from(input.len()).unwrap_or(u64::MAX) > transport_max
        || !input.ends_with(b"\n")
        || input.starts_with(&[0xef, 0xbb, 0xbf])
    {
        return Err(json_invalid());
    }
    let body = &input[..input.len() - 1];
    let strict = parse_strict_json(
        body,
        StrictJsonLimits::new(
            transport_max,
            transport_max,
            API_WRAPPER_LEVELS,
            API_STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| json_invalid())?;
    let canonical = canonical_json_bytes(&strict).map_err(|_| json_invalid())?;
    let value = serde_json::from_slice(&canonical).map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Shape,
            V1ErrorCode::Shape,
            None,
            None::<String>,
        )
    })?;
    let canonical_transport = input.len() == canonical.len() + 1
        && input[..canonical.len()] == canonical
        && input[canonical.len()] == b'\n';
    Ok(ParsedRequest {
        value,
        canonical_transport,
    })
}

pub fn encode_response<T: Serialize>(response: &T) -> Result<Vec<u8>, V1ApiError> {
    let serialized = serde_json::to_vec(response).map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Artifact,
            V1ErrorCode::VcInvalid,
            None,
            None::<String>,
        )
    })?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(
            API_TRANSPORT_BYTES_MAX,
            API_TRANSPORT_BYTES_MAX,
            API_WRAPPER_LEVELS,
            API_STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| json_invalid())?;
    let mut canonical = canonical_json_bytes(&strict).map_err(|_| json_invalid())?;
    canonical.push(b'\n');
    Ok(canonical)
}

pub fn validate_session_id(value: &str) -> Result<(), V1ApiError> {
    let valid = value.strip_prefix('s').is_some_and(|digits| {
        !digits.is_empty()
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok_and(|number| number > 0)
    });
    if valid {
        Ok(())
    } else {
        Err(V1ApiError::new(
            V1ValidationPhase::Scalar,
            V1ErrorCode::Scalar,
            Some("session_id"),
            None::<String>,
        ))
    }
}

pub fn validate_sha256(value: &str, field: &'static str) -> Result<(), V1ApiError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(V1ApiError::new(
            V1ValidationPhase::Scalar,
            V1ErrorCode::Scalar,
            Some(field),
            None::<String>,
        ))
    }
}

fn json_invalid() -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Transport,
        V1ErrorCode::JsonInvalid,
        None,
        None::<String>,
    )
}
