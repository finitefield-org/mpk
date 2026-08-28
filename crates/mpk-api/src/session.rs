//! Session lifecycle for the local AI proof API.

use std::collections::BTreeMap;
use std::fmt;

use crate::proof_api::ApiProofId;
use crate::term_api::ApiTermId;
use mpk_cert::encode::{ProofNode, TheoryCertificate};
use mpk_core::{Environment, LevelArena, Name, TermArena, TermId};
use mpk_kernel::ProofCheckProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct ApiService {
    next_session_index: u64,
    sessions: BTreeMap<SessionId, ApiSession>,
}

impl ApiService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_session(
        &mut self,
        request: StartSessionRequest,
    ) -> Result<StartSessionResponse, ApiError> {
        let response = self.preview_start_session(&request)?;
        let next_session_index = self.next_session_index()?;
        self.next_session_index = next_session_index;
        let session_id = response.session_id.clone();
        let session = ApiSession {
            id: session_id.clone(),
            created_index: next_session_index,
            module_name: request.module_name,
            proof_profile: request.proof_profile,
            status: SessionStatus::Active,
            levels: LevelArena::new(),
            terms: TermArena::new(),
            term_ids: BTreeMap::new(),
            proof_nodes: Vec::new(),
            theory_certificates: Vec::new(),
            environment: Environment::new(),
        };
        self.sessions.insert(session_id, session);
        Ok(response)
    }

    pub(crate) fn preview_start_session(
        &self,
        request: &StartSessionRequest,
    ) -> Result<StartSessionResponse, ApiError> {
        validate_module_name(&request.module_name)?;
        let next_session_index = self.next_session_index()?;
        Ok(SessionSummary {
            session_id: SessionId(format!("s{next_session_index}")),
            module_name: request.module_name.clone(),
            proof_profile: request.proof_profile,
            status: SessionStatus::Active,
        }
        .into_start_response())
    }

    fn next_session_index(&self) -> Result<u64, ApiError> {
        self.next_session_index
            .checked_add(1)
            .ok_or_else(ApiError::session_limit_exceeded)
    }

    pub fn session(&self, session_id: &SessionId) -> Option<&ApiSession> {
        self.sessions.get(session_id)
    }

    pub fn session_mut(&mut self, session_id: &SessionId) -> Option<&mut ApiSession> {
        self.sessions.get_mut(session_id)
    }

    pub(crate) fn require_session_mut(
        &mut self,
        session_id: &SessionId,
    ) -> Result<&mut ApiSession, ApiError> {
        self.session_mut(session_id)
            .ok_or_else(|| ApiError::unknown_session(session_id))
    }

    pub fn session_summary(&self, session_id: &SessionId) -> Result<SessionSummary, ApiError> {
        self.session(session_id)
            .map(ApiSession::summary)
            .ok_or_else(|| ApiError::unknown_session(session_id))
    }

    pub fn list_sessions(&self) -> Vec<SessionSummary> {
        let mut sessions = self
            .sessions
            .values()
            .map(|session| (session.created_index, session.summary()))
            .collect::<Vec<_>>();
        sessions.sort_by_key(|(created_index, _)| *created_index);
        sessions.into_iter().map(|(_, summary)| summary).collect()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[derive(Debug)]
pub struct ApiSession {
    id: SessionId,
    created_index: u64,
    module_name: String,
    proof_profile: ProofProfile,
    status: SessionStatus,
    levels: LevelArena,
    terms: TermArena,
    term_ids: BTreeMap<ApiTermId, TermId>,
    proof_nodes: Vec<ProofNode>,
    theory_certificates: Vec<TheoryCertificate>,
    environment: Environment,
}

impl ApiSession {
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn proof_profile(&self) -> ProofProfile {
        self.proof_profile
    }

    pub fn kernel_proof_profile(&self) -> ProofCheckProfile {
        self.proof_profile.into()
    }

    pub fn status(&self) -> SessionStatus {
        self.status
    }

    pub fn levels(&self) -> &LevelArena {
        &self.levels
    }

    pub fn levels_mut(&mut self) -> &mut LevelArena {
        &mut self.levels
    }

    pub fn terms(&self) -> &TermArena {
        &self.terms
    }

    pub fn terms_mut(&mut self) -> &mut TermArena {
        &mut self.terms
    }

    pub fn core_term_id(&self, term_id: ApiTermId) -> Option<TermId> {
        self.term_ids.get(&term_id).copied()
    }

    pub(crate) fn register_term_id(&mut self, term_id: TermId) -> Result<ApiTermId, ApiError> {
        let api_term_id = ApiTermId(u32::try_from(term_id.index()).map_err(|_| {
            ApiError::new(
                ApiErrorCode::TermIdOverflow,
                "core term id exceeded API u32 term ids",
                None,
                Some(term_id.index().to_string()),
            )
        })?);
        self.term_ids.insert(api_term_id, term_id);
        Ok(api_term_id)
    }

    pub(crate) fn require_term_id(
        &self,
        term_id: ApiTermId,
        field: impl Into<String>,
    ) -> Result<TermId, ApiError> {
        self.core_term_id(term_id).ok_or_else(|| {
            ApiError::new(
                ApiErrorCode::UnknownTerm,
                format!(
                    "term id {} is not interned in this API session",
                    term_id.as_u32()
                ),
                Some(field.into()),
                None,
            )
        })
    }

    pub fn proof_node(&self, proof_id: ApiProofId) -> Option<&ProofNode> {
        self.proof_nodes
            .get(usize::try_from(proof_id.as_u32()).expect("u32 id fits in usize"))
    }

    pub fn proof_node_count(&self) -> usize {
        self.proof_nodes.len()
    }

    pub(crate) fn register_proof_node(&mut self, node: ProofNode) -> Result<ApiProofId, ApiError> {
        let proof_id = ApiProofId(u32::try_from(self.proof_nodes.len()).map_err(|_| {
            ApiError::new(
                ApiErrorCode::ProofIdOverflow,
                "API proof node table exceeded u32 ids",
                None,
                Some(self.proof_nodes.len().to_string()),
            )
        })?);
        self.proof_nodes.push(node);
        Ok(proof_id)
    }

    pub fn theory_certificate(&self, index: u32) -> Option<&TheoryCertificate> {
        self.theory_certificates
            .get(usize::try_from(index).expect("u32 id fits in usize"))
    }

    pub fn theory_certificate_count(&self) -> usize {
        self.theory_certificates.len()
    }

    pub(crate) fn register_theory_certificate(
        &mut self,
        certificate: TheoryCertificate,
    ) -> Result<u32, ApiError> {
        let index = u32::try_from(self.theory_certificates.len()).map_err(|_| {
            ApiError::new(
                ApiErrorCode::ProofIdOverflow,
                "API theory certificate table exceeded u32 ids",
                None,
                Some(self.theory_certificates.len().to_string()),
            )
        })?;
        self.theory_certificates.push(certificate);
        Ok(index)
    }

    pub(crate) fn require_proof_id(
        &self,
        proof_id: ApiProofId,
        field: impl Into<String>,
    ) -> Result<u32, ApiError> {
        if self.proof_node(proof_id).is_some() {
            return Ok(proof_id.as_u32());
        }

        Err(ApiError::new(
            ApiErrorCode::UnknownProof,
            format!(
                "proof id {} is not registered in this API session",
                proof_id.as_u32()
            ),
            Some(field.into()),
            None,
        ))
    }

    pub(crate) fn core_parts_mut(&mut self) -> (&mut LevelArena, &mut TermArena, &Environment) {
        (&mut self.levels, &mut self.terms, &self.environment)
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    pub fn summary(&self) -> SessionSummary {
        SessionSummary {
            session_id: self.id.clone(),
            module_name: self.module_name.clone(),
            proof_profile: self.proof_profile,
            status: self.status,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StartSessionRequest {
    pub module_name: String,
    #[serde(default)]
    pub proof_profile: ProofProfile,
}

impl StartSessionRequest {
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            proof_profile: ProofProfile::default(),
        }
    }

    pub fn with_proof_profile(mut self, proof_profile: ProofProfile) -> Self {
        self.proof_profile = proof_profile;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StartSessionResponse {
    pub session_id: SessionId,
    pub module_name: String,
    pub proof_profile: ProofProfile,
    pub status: SessionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub module_name: String,
    pub proof_profile: ProofProfile,
    pub status: SessionStatus,
}

impl SessionSummary {
    fn into_start_response(self) -> StartSessionResponse {
        StartSessionResponse {
            session_id: self.session_id,
            module_name: self.module_name,
            proof_profile: self.proof_profile,
            status: self.status,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ProofProfile {
    #[default]
    CoreBootstrap,
    MvpStructural,
    MvpStrict,
}

impl ProofProfile {
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::CoreBootstrap => "core-bootstrap",
            Self::MvpStructural => "mvp-structural",
            Self::MvpStrict => "mvp-strict",
        }
    }
}

impl From<ProofProfile> for ProofCheckProfile {
    fn from(value: ProofProfile) -> Self {
        match value {
            ProofProfile::CoreBootstrap => Self::CoreBootstrap,
            ProofProfile::MvpStructural => Self::MvpStructural,
            ProofProfile::MvpStrict => Self::MvpStrict,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ApiError {
    pub(crate) fn new(
        code: ApiErrorCode,
        message: impl Into<String>,
        field: Option<String>,
        detail: Option<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            field,
            detail,
        }
    }

    fn invalid_module_name(message: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::InvalidModuleName,
            message: message.into(),
            field: Some("module_name".to_owned()),
            detail: Some(detail.into()),
        }
    }

    pub(crate) fn unknown_session(session_id: &SessionId) -> Self {
        Self {
            code: ApiErrorCode::UnknownSession,
            message: format!("API session {session_id} does not exist"),
            field: Some("session_id".to_owned()),
            detail: None,
        }
    }

    fn session_limit_exceeded() -> Self {
        Self {
            code: ApiErrorCode::SessionLimitExceeded,
            message: "API session id counter exceeded u64".to_owned(),
            field: None,
            detail: None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    InvalidModuleName,
    InvalidJsonl,
    InvalidGlobalName,
    ProofIdOverflow,
    ProofCheckFailed,
    StrategyNotApplicable,
    SessionLimitExceeded,
    TermIdOverflow,
    UnsupportedProofNodeKind,
    UnknownProof,
    UnknownSession,
    UnknownGlobal,
    UnknownTerm,
    UnsupportedUniverseLevel,
}

impl ApiErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidModuleName => "INVALID_MODULE_NAME",
            Self::InvalidJsonl => "INVALID_JSONL",
            Self::InvalidGlobalName => "INVALID_GLOBAL_NAME",
            Self::ProofIdOverflow => "PROOF_ID_OVERFLOW",
            Self::ProofCheckFailed => "PROOF_CHECK_FAILED",
            Self::StrategyNotApplicable => "STRATEGY_NOT_APPLICABLE",
            Self::SessionLimitExceeded => "SESSION_LIMIT_EXCEEDED",
            Self::TermIdOverflow => "TERM_ID_OVERFLOW",
            Self::UnsupportedProofNodeKind => "UNSUPPORTED_PROOF_NODE_KIND",
            Self::UnknownProof => "UNKNOWN_PROOF",
            Self::UnknownSession => "UNKNOWN_SESSION",
            Self::UnknownGlobal => "UNKNOWN_GLOBAL",
            Self::UnknownTerm => "UNKNOWN_TERM",
            Self::UnsupportedUniverseLevel => "UNSUPPORTED_UNIVERSE_LEVEL",
        }
    }
}

fn validate_module_name(module_name: &str) -> Result<(), ApiError> {
    if module_name.trim() != module_name {
        return Err(ApiError::invalid_module_name(
            "module_name must not contain leading or trailing whitespace",
            "WHITESPACE",
        ));
    }
    Name::parse(module_name).map_err(|error| {
        ApiError::invalid_module_name(
            format!("module_name {module_name:?} is not a valid core name"),
            error.code(),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_session_with_default_profile() {
        let mut service = ApiService::new();

        let response = service
            .start_session(StartSessionRequest::new("Example.Api.Session"))
            .expect("session starts");

        assert_eq!(response.session_id.as_str(), "s1");
        assert_eq!(response.module_name, "Example.Api.Session");
        assert_eq!(response.proof_profile, ProofProfile::CoreBootstrap);
        assert_eq!(response.status, SessionStatus::Active);
        assert_eq!(service.session_count(), 1);

        let session = service
            .session(&response.session_id)
            .expect("session is registered");
        assert_eq!(session.module_name(), "Example.Api.Session");
        assert_eq!(session.status(), SessionStatus::Active);
        assert_eq!(session.levels().len(), 1);
        assert!(session.terms().is_empty());
        assert!(session.environment().is_empty());
    }

    #[test]
    fn starts_multiple_sessions_with_deterministic_ids() {
        let mut service = ApiService::new();

        let first = service
            .start_session(StartSessionRequest::new("Example.Api.First"))
            .expect("first session starts");
        let second = service
            .start_session(
                StartSessionRequest::new("Example.Api.Second")
                    .with_proof_profile(ProofProfile::MvpStructural),
            )
            .expect("second session starts");

        assert_eq!(first.session_id.as_str(), "s1");
        assert_eq!(second.session_id.as_str(), "s2");
        assert_eq!(second.proof_profile, ProofProfile::MvpStructural);
        assert_eq!(
            service.list_sessions(),
            vec![
                SessionSummary {
                    session_id: first.session_id,
                    module_name: "Example.Api.First".to_owned(),
                    proof_profile: ProofProfile::CoreBootstrap,
                    status: SessionStatus::Active,
                },
                SessionSummary {
                    session_id: second.session_id,
                    module_name: "Example.Api.Second".to_owned(),
                    proof_profile: ProofProfile::MvpStructural,
                    status: SessionStatus::Active,
                },
            ]
        );
    }

    #[test]
    fn list_sessions_uses_creation_order_after_double_digit_ids() {
        let mut service = ApiService::new();
        for index in 1..=11 {
            service
                .start_session(StartSessionRequest::new(format!(
                    "Example.Api.Session{index}"
                )))
                .expect("session starts");
        }

        let sessions = service.list_sessions();

        assert_eq!(sessions[1].session_id.as_str(), "s2");
        assert_eq!(sessions[9].session_id.as_str(), "s10");
        assert_eq!(sessions[10].session_id.as_str(), "s11");
    }

    #[test]
    fn maps_profiles_to_kernel_profiles() {
        let mut service = ApiService::new();
        let response = service
            .start_session(
                StartSessionRequest::new("Example.Api.Structural")
                    .with_proof_profile(ProofProfile::MvpStructural),
            )
            .expect("session starts");

        let session = service
            .session(&response.session_id)
            .expect("session exists");

        assert_eq!(
            session.kernel_proof_profile(),
            ProofCheckProfile::MvpStructural
        );
        assert_eq!(
            ProofProfile::MvpStructural.canonical_name(),
            ProofCheckProfile::MvpStructural.canonical_name()
        );

        let strict = service
            .start_session(
                StartSessionRequest::new("Example.Api.Strict")
                    .with_proof_profile(ProofProfile::MvpStrict),
            )
            .expect("strict session starts");
        let strict = service
            .session(&strict.session_id)
            .expect("strict session exists");
        assert_eq!(strict.kernel_proof_profile(), ProofCheckProfile::MvpStrict);
        assert_eq!(
            ProofProfile::MvpStrict.canonical_name(),
            ProofCheckProfile::MvpStrict.canonical_name()
        );
    }

    #[test]
    fn rejects_invalid_module_names() {
        let mut service = ApiService::new();

        let error = service
            .start_session(StartSessionRequest::new("Example.Bad-Name"))
            .expect_err("invalid module name rejects");

        assert_eq!(error.code, ApiErrorCode::InvalidModuleName);
        assert_eq!(error.field.as_deref(), Some("module_name"));
        assert_eq!(error.detail.as_deref(), Some("INVALID_COMPONENT_CHAR"));
        assert!(service.is_empty());
    }

    #[test]
    fn rejects_module_names_with_outer_whitespace() {
        let mut service = ApiService::new();

        let error = service
            .start_session(StartSessionRequest::new(" Example.Api.Session"))
            .expect_err("whitespace rejects");

        assert_eq!(error.code, ApiErrorCode::InvalidModuleName);
        assert_eq!(error.detail.as_deref(), Some("WHITESPACE"));
        assert!(service.is_empty());
    }

    #[test]
    fn unknown_session_returns_structured_error() {
        let service = ApiService::new();

        let error = service
            .session_summary(&SessionId("s404".to_owned()))
            .expect_err("unknown session rejects");

        assert_eq!(error.code, ApiErrorCode::UnknownSession);
        assert_eq!(error.field.as_deref(), Some("session_id"));
    }

    #[test]
    fn session_counter_overflow_returns_structured_error() {
        let mut service = ApiService {
            next_session_index: u64::MAX,
            sessions: BTreeMap::new(),
        };

        let error = service
            .start_session(StartSessionRequest::new("Example.Api.Overflow"))
            .expect_err("session counter overflow rejects");

        assert_eq!(error.code, ApiErrorCode::SessionLimitExceeded);
        assert!(service.is_empty());
    }

    #[test]
    fn start_session_response_serializes_stably() {
        let mut service = ApiService::new();
        let response = service
            .start_session(StartSessionRequest::new("Example.Api.Json"))
            .expect("session starts");

        let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

        assert_eq!(
            encoded,
            r#"{
  "session_id": "s1",
  "module_name": "Example.Api.Json",
  "proof_profile": "core-bootstrap",
  "status": "active"
}"#
        );
    }
}
