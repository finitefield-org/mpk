//! Repair diagnostics for failed local AI proof API checks.

use mpk_core::{TermId, TermNode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    check_api::{diagnose_proof_node_in_session, ProofCheckFailure},
    proof_api::ApiProofId,
    session::{ApiError, ApiErrorCode, ApiService, ApiSession, SessionId},
    term_api::ApiTermId,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepairDiagnosticRequest {
    pub session_id: SessionId,
    pub proof_id: ApiProofId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepairDiagnosticResponse {
    pub session_id: SessionId,
    pub diagnostic: RepairDiagnostic,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepairDiagnostic {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub node_id: ApiProofId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_type_id: Option<ApiTermId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_type_id: Option<ApiTermId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_head: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_head: Option<String>,
    #[serde(default)]
    pub context_summary: Vec<ApiTermId>,
    #[serde(default)]
    pub repair_hints: Vec<String>,
}

impl ApiService {
    pub fn proof_repair_diagnostics(
        &mut self,
        request: RepairDiagnosticRequest,
    ) -> Result<RepairDiagnosticResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let diagnostic = match diagnose_proof_node_in_session(session, request.proof_id) {
            Ok(_) => RepairDiagnostic {
                ok: true,
                error_code: None,
                node_id: request.proof_id,
                expected_type_id: None,
                actual_type_id: None,
                expected_head: None,
                actual_head: None,
                context_summary: Vec::new(),
                repair_hints: Vec::new(),
            },
            Err(failure) => diagnostic_for_failure(session, failure),
        };

        Ok(RepairDiagnosticResponse {
            session_id,
            diagnostic,
        })
    }
}

fn diagnostic_for_failure(session: &ApiSession, failure: ProofCheckFailure) -> RepairDiagnostic {
    let parsed = ParsedDiagnosticDetail::parse(failure.error.detail.as_deref());
    let error_code = parsed
        .error_code
        .unwrap_or_else(|| failure.error.code.as_str().to_owned());
    let expected_type_id = parsed.expected_type_id;
    let actual_type_id = parsed.actual_type_id;
    let expected_head = expected_type_id.and_then(|term_id| term_head(session, term_id));
    let actual_head = actual_type_id.and_then(|term_id| term_head(session, term_id));
    let repair_hints = repair_hints(
        &failure.error.code,
        &error_code,
        parsed.kind.as_deref(),
        expected_head.as_deref(),
        actual_head.as_deref(),
    );

    RepairDiagnostic {
        ok: false,
        error_code: Some(error_code),
        node_id: failure.proof_id,
        expected_type_id,
        actual_type_id,
        expected_head,
        actual_head,
        context_summary: failure.context_summary,
        repair_hints,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ParsedDiagnosticDetail {
    error_code: Option<String>,
    kind: Option<String>,
    expected_type_id: Option<ApiTermId>,
    actual_type_id: Option<ApiTermId>,
}

impl ParsedDiagnosticDetail {
    fn parse(detail: Option<&str>) -> Self {
        let Some(detail) = detail else {
            return Self::default();
        };
        let Ok(value) = serde_json::from_str::<Value>(detail) else {
            return Self::default();
        };
        let details = value.get("details").unwrap_or(&value);

        Self {
            error_code: string_value(value.get("code")),
            kind: string_value(details.get("kind")),
            expected_type_id: first_api_term_id(
                details,
                &[
                    "expected_term_index",
                    "expected_domain_index",
                    "expected_type_index",
                    "expected_term",
                    "term",
                ],
            ),
            actual_type_id: first_api_term_id(
                details,
                &[
                    "inferred_term_index",
                    "actual_domain_index",
                    "actual_term_index",
                    "whnf_type_index",
                    "inferred_term",
                ],
            ),
        }
    }
}

fn string_value(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToOwned::to_owned)
}

fn first_api_term_id(details: &Value, keys: &[&str]) -> Option<ApiTermId> {
    keys.iter()
        .find_map(|key| u32_value(details.get(*key)).map(ApiTermId))
}

fn u32_value(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(number) => number.as_u64().and_then(|raw| u32::try_from(raw).ok()),
        Value::String(raw) => raw.parse().ok(),
        _ => None,
    }
}

fn term_head(session: &ApiSession, term_id: ApiTermId) -> Option<String> {
    let term = session
        .core_term_id(term_id)
        .or_else(|| core_term_by_raw_index(session, term_id.as_u32()))?;
    Some(term_head_name(session, term))
}

fn core_term_by_raw_index(session: &ApiSession, raw: u32) -> Option<TermId> {
    let raw = usize::try_from(raw).ok()?;
    session
        .terms()
        .iter_topological()
        .find_map(|(term, _)| (term.index() == raw).then_some(term))
}

fn term_head_name(session: &ApiSession, term: TermId) -> String {
    match session.terms().node(term) {
        TermNode::Sort(_) => "sort".to_owned(),
        TermNode::Var(index) => format!("var:{index}"),
        TermNode::Const { global, .. } => session
            .environment()
            .lookup(*global)
            .map(|declaration| declaration.name().as_str().to_owned())
            .unwrap_or_else(|| format!("global:{}", global.as_u32())),
        TermNode::App { function, .. } => term_head_name(session, *function),
        TermNode::Lam { .. } => "lam".to_owned(),
        TermNode::Pi { .. } => "pi".to_owned(),
        TermNode::Let { .. } => "let".to_owned(),
    }
}

fn repair_hints(
    api_code: &ApiErrorCode,
    error_code: &str,
    detail_kind: Option<&str>,
    expected_head: Option<&str>,
    actual_head: Option<&str>,
) -> Vec<String> {
    let hints: &[&str] = match (api_code, error_code, detail_kind) {
        (ApiErrorCode::UnknownProof, _, _) => &["select-existing-proof"],
        (ApiErrorCode::UnknownTerm, _, _) => &["select-existing-term"],
        (ApiErrorCode::UnsupportedProofNodeKind, _, _) => &["use-core-bootstrap-node"],
        (_, "CORE_NOT_A_FUNCTION", _) => &["apply", "rebuild-function"],
        (_, "CORE_UNBOUND_VARIABLE", _) => &["intro", "use-local"],
        (_, "CORE_TYPE_MISMATCH", Some("lambda_domain_mismatch")) => &["intro", "conv"],
        (_, "CORE_TYPE_MISMATCH", Some(kind)) if kind.ends_with("_component_not_sort") => {
            &["rebuild-type", "intro"]
        }
        (_, "CORE_TYPE_MISMATCH", _) if expected_head != actual_head => &["apply", "intro", "conv"],
        (_, "CORE_TYPE_MISMATCH", _) => &["exact", "conv"],
        _ => &["inspect-node"],
    };

    hints.iter().map(|hint| (*hint).to_owned()).collect()
}
