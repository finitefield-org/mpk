//! Batch candidate checking for the local AI proof API.

use serde::{Deserialize, Serialize};

use crate::{
    check_api::check_proof_node_in_session,
    proof_api::ApiProofId,
    session::{ApiError, ApiService, SessionId},
    term_api::ApiTermId,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCheckRequest {
    pub session_id: SessionId,
    #[serde(default)]
    pub mode: BatchCheckMode,
    pub candidates: Vec<BatchCandidate>,
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum BatchCheckMode {
    #[default]
    FailFastPerCandidate,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCandidate {
    pub candidate_id: String,
    pub proof_id: ApiProofId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCheckResponse {
    pub session_id: SessionId,
    pub mode: BatchCheckMode,
    pub summary: BatchCheckSummary,
    pub verdicts: Vec<CandidateVerdict>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCheckSummary {
    pub total: usize,
    pub accepted: usize,
    pub rejected: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateVerdict {
    pub candidate_id: String,
    pub proof_id: ApiProofId,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term_id: Option<ApiTermId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl ApiService {
    pub fn vc_check_candidates(
        &mut self,
        request: BatchCheckRequest,
    ) -> Result<BatchCheckResponse, ApiError> {
        let session_id = request.session_id;
        let mode = request.mode;
        let session = self.require_session_mut(&session_id)?;
        let mut verdicts = Vec::with_capacity(request.candidates.len());
        let mut accepted = 0usize;

        for candidate in request.candidates {
            let result = match mode {
                BatchCheckMode::FailFastPerCandidate => {
                    check_proof_node_in_session(session, candidate.proof_id)
                }
            };
            let verdict = match result {
                Ok(term_id) => {
                    accepted += 1;
                    CandidateVerdict {
                        candidate_id: candidate.candidate_id,
                        proof_id: candidate.proof_id,
                        ok: true,
                        term_id: Some(term_id),
                        error: None,
                    }
                }
                Err(error) => CandidateVerdict {
                    candidate_id: candidate.candidate_id,
                    proof_id: candidate.proof_id,
                    ok: false,
                    term_id: None,
                    error: Some(error),
                },
            };
            verdicts.push(verdict);
        }

        let total = verdicts.len();
        Ok(BatchCheckResponse {
            session_id,
            mode,
            summary: BatchCheckSummary {
                total,
                accepted,
                rejected: total - accepted,
            },
            verdicts,
        })
    }
}
