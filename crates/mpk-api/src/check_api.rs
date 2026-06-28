//! Proof-node checking endpoints for the local AI proof API.

use mpk_cert::encode::ProofNode;
use mpk_core::{check, infer, LocalContext, TermId, TermNode};
use serde::{Deserialize, Serialize};

use crate::{
    proof_api::ApiProofId,
    session::{ApiError, ApiErrorCode, ApiService, ApiSession, SessionId},
    term_api::ApiTermId,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckNodeRequest {
    pub session_id: SessionId,
    pub proof_id: ApiProofId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckNodeResponse {
    pub session_id: SessionId,
    pub proof_id: ApiProofId,
    pub ok: bool,
    pub term_id: ApiTermId,
}

impl ApiService {
    pub fn proof_check_node(
        &mut self,
        request: CheckNodeRequest,
    ) -> Result<CheckNodeResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let term_id = check_proof_node_in_session(session, request.proof_id)?;

        Ok(CheckNodeResponse {
            session_id,
            proof_id: request.proof_id,
            ok: true,
            term_id,
        })
    }
}

pub(crate) fn check_proof_node_in_session(
    session: &mut ApiSession,
    proof_id: ApiProofId,
) -> Result<ApiTermId, ApiError> {
    session.require_proof_id(proof_id, "proof_id")?;
    let term = CheckNodeDriver { session }.check_node(proof_id, &LocalContext::new())?;
    session.register_term_id(term)
}

struct CheckNodeDriver<'session> {
    session: &'session mut ApiSession,
}

impl CheckNodeDriver<'_> {
    fn check_node(
        &mut self,
        proof_id: ApiProofId,
        context: &LocalContext,
    ) -> Result<TermId, ApiError> {
        let node = self
            .session
            .proof_node(proof_id)
            .cloned()
            .ok_or_else(|| unknown_proof(proof_id, "proof_id"))?;

        match node {
            ProofNode::Exact {
                term,
                expected_type,
            } => {
                let term = self.term_id(proof_id, term, "term")?;
                let expected_type = self.expected_type(proof_id, expected_type, context)?;
                self.check_term(proof_id, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Apply {
                function_proof,
                argument_proofs,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_id, expected_type, context)?;
                let function = self.check_node(ApiProofId(function_proof), context)?;
                let arguments = argument_proofs
                    .into_iter()
                    .map(|argument| self.check_node(ApiProofId(argument), context))
                    .collect::<Result<Vec<_>, _>>()?;
                let term = self.session.terms_mut().app(function, arguments);
                self.check_term(proof_id, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Intro {
                domain_type,
                body_proof,
                expected_type,
            } => {
                let domain_type = self.term_id(proof_id, domain_type, "domain_type")?;
                self.expect_type_is_sort(proof_id, context, "domain_type", domain_type)?;
                let expected_type = self.expected_type(proof_id, expected_type, context)?;

                let mut body_context = context.clone();
                body_context.push_binder(domain_type);
                let body = self.check_node(ApiProofId(body_proof), &body_context)?;
                let term = self.session.terms_mut().lam(domain_type, body);
                self.check_term(proof_id, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Refl {
                term,
                expected_type,
            } => {
                let term = self.term_id(proof_id, term, "term")?;
                let expected_type = self.expected_type(proof_id, expected_type, context)?;
                self.check_term(proof_id, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Conv {
                proof,
                expected_type,
                defeq_witness,
            } => {
                if let Some(defeq_witness) = defeq_witness {
                    let _ = self.term_id(proof_id, defeq_witness, "defeq_witness")?;
                }
                let expected_type = self.expected_type(proof_id, expected_type, context)?;
                let term = self.check_node(ApiProofId(proof), context)?;
                self.check_term(proof_id, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::LetProof { .. }
            | ProofNode::Rewrite { .. }
            | ProofNode::EqRec { .. }
            | ProofNode::Constructor { .. }
            | ProofNode::Recursor { .. }
            | ProofNode::Theory { .. } => Err(ApiError::new(
                ApiErrorCode::UnsupportedProofNodeKind,
                format!(
                    "proof node {} is not supported by the check-node endpoint",
                    proof_id.as_u32()
                ),
                Some("proof_id".to_owned()),
                Some(proof_id.as_u32().to_string()),
            )),
        }
    }

    fn expected_type(
        &mut self,
        proof_id: ApiProofId,
        term: u32,
        context: &LocalContext,
    ) -> Result<TermId, ApiError> {
        let expected_type = self.term_id(proof_id, term, "expected_type")?;
        self.expect_type_is_sort(proof_id, context, "expected_type", expected_type)?;
        Ok(expected_type)
    }

    fn term_id(
        &self,
        proof_id: ApiProofId,
        term: u32,
        field: impl Into<String>,
    ) -> Result<TermId, ApiError> {
        self.session
            .require_term_id(ApiTermId(term), field)
            .map_err(|error| proof_error_with_node(proof_id, error))
    }

    fn infer_term(
        &mut self,
        proof_id: ApiProofId,
        context: &LocalContext,
        term: TermId,
    ) -> Result<TermId, ApiError> {
        let (levels, terms, env) = self.session.core_parts_mut();
        infer(levels, terms, context, env, term).map_err(|error| {
            proof_check_failed(
                proof_id,
                "core inference failed while checking proof node",
                error.to_deterministic_json(),
            )
        })
    }

    fn check_term(
        &mut self,
        proof_id: ApiProofId,
        context: &LocalContext,
        term: TermId,
        expected_type: TermId,
    ) -> Result<(), ApiError> {
        let (levels, terms, env) = self.session.core_parts_mut();
        check(levels, terms, context, env, term, expected_type).map_err(|error| {
            proof_check_failed(
                proof_id,
                "core checking failed while checking proof node",
                error.to_deterministic_json(),
            )
        })
    }

    fn expect_type_is_sort(
        &mut self,
        proof_id: ApiProofId,
        context: &LocalContext,
        field: &'static str,
        term: TermId,
    ) -> Result<(), ApiError> {
        let inferred = self.infer_term(proof_id, context, term)?;
        if matches!(self.session.terms().node(inferred), TermNode::Sort(_)) {
            return Ok(());
        }

        Err(proof_check_failed(
            proof_id,
            format!("{field} must infer to a sort"),
            format!(
                "{{\"field\":\"{field}\",\"inferred_term\":{},\"term\":{}}}",
                inferred.index(),
                term.index()
            ),
        ))
    }
}

fn unknown_proof(proof_id: ApiProofId, field: impl Into<String>) -> ApiError {
    ApiError::new(
        ApiErrorCode::UnknownProof,
        format!(
            "proof id {} is not registered in this API session",
            proof_id.as_u32()
        ),
        Some(field.into()),
        None,
    )
}

fn proof_error_with_node(proof_id: ApiProofId, mut error: ApiError) -> ApiError {
    error.detail = Some(match error.detail {
        Some(detail) => format!("proof_id={}; {detail}", proof_id.as_u32()),
        None => format!("proof_id={}", proof_id.as_u32()),
    });
    error
}

fn proof_check_failed(
    proof_id: ApiProofId,
    message: impl Into<String>,
    detail: impl Into<String>,
) -> ApiError {
    ApiError::new(
        ApiErrorCode::ProofCheckFailed,
        format!("proof id {} failed: {}", proof_id.as_u32(), message.into()),
        Some("proof_id".to_owned()),
        Some(detail.into()),
    )
}
