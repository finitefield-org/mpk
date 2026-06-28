//! Core-bootstrap proof construction endpoints for the local AI proof API.

use mpk_cert::encode::ProofNode;
use serde::{Deserialize, Serialize};

use crate::{
    session::{ApiError, ApiErrorCode, ApiService, ApiSession, SessionId},
    term_api::ApiTermId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ApiProofId(pub u32);

impl ApiProofId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExactProofRequest {
    pub session_id: SessionId,
    pub term: ApiTermId,
    pub expected_type: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyProofRequest {
    pub session_id: SessionId,
    pub function_proof: ApiProofId,
    #[serde(default)]
    pub argument_proofs: Vec<ApiProofId>,
    pub expected_type: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntroProofRequest {
    pub session_id: SessionId,
    pub domain_type: ApiTermId,
    pub body_proof: ApiProofId,
    pub expected_type: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReflProofRequest {
    pub session_id: SessionId,
    pub term: ApiTermId,
    pub expected_type: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConvProofRequest {
    pub session_id: SessionId,
    pub proof: ApiProofId,
    pub expected_type: ApiTermId,
    #[serde(default)]
    pub defeq_witness: Option<ApiTermId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProofResponse {
    pub session_id: SessionId,
    pub proof_id: ApiProofId,
}

impl ApiService {
    pub fn proof_exact(&mut self, request: ExactProofRequest) -> Result<ProofResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let term = cert_term_id(session, request.term, "term")?;
        let expected_type = cert_term_id(session, request.expected_type, "expected_type")?;
        response(
            session,
            session_id,
            ProofNode::Exact {
                term,
                expected_type,
            },
        )
    }

    pub fn proof_apply(&mut self, request: ApplyProofRequest) -> Result<ProofResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let function_proof = session.require_proof_id(request.function_proof, "function_proof")?;
        let argument_proofs = request
            .argument_proofs
            .iter()
            .enumerate()
            .map(|(index, proof_id)| {
                session.require_proof_id(*proof_id, format!("argument_proofs[{index}]"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let expected_type = cert_term_id(session, request.expected_type, "expected_type")?;
        response(
            session,
            session_id,
            ProofNode::Apply {
                function_proof,
                argument_proofs,
                expected_type,
            },
        )
    }

    pub fn proof_intro(&mut self, request: IntroProofRequest) -> Result<ProofResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let domain_type = cert_term_id(session, request.domain_type, "domain_type")?;
        let body_proof = session.require_proof_id(request.body_proof, "body_proof")?;
        let expected_type = cert_term_id(session, request.expected_type, "expected_type")?;
        response(
            session,
            session_id,
            ProofNode::Intro {
                domain_type,
                body_proof,
                expected_type,
            },
        )
    }

    pub fn proof_refl(&mut self, request: ReflProofRequest) -> Result<ProofResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let term = cert_term_id(session, request.term, "term")?;
        let expected_type = cert_term_id(session, request.expected_type, "expected_type")?;
        response(
            session,
            session_id,
            ProofNode::Refl {
                term,
                expected_type,
            },
        )
    }

    pub fn proof_conv(&mut self, request: ConvProofRequest) -> Result<ProofResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let proof = session.require_proof_id(request.proof, "proof")?;
        let expected_type = cert_term_id(session, request.expected_type, "expected_type")?;
        let defeq_witness = request
            .defeq_witness
            .map(|term_id| cert_term_id(session, term_id, "defeq_witness"))
            .transpose()?;
        response(
            session,
            session_id,
            ProofNode::Conv {
                proof,
                expected_type,
                defeq_witness,
            },
        )
    }
}

fn cert_term_id(
    session: &ApiSession,
    term_id: ApiTermId,
    field: impl Into<String>,
) -> Result<u32, ApiError> {
    let core = session.require_term_id(term_id, field)?;
    u32::try_from(core.index()).map_err(|_| {
        ApiError::new(
            ApiErrorCode::TermIdOverflow,
            "core term id exceeded certificate u32 term ids",
            None,
            Some(core.index().to_string()),
        )
    })
}

fn response(
    session: &mut ApiSession,
    session_id: SessionId,
    node: ProofNode,
) -> Result<ProofResponse, ApiError> {
    Ok(ProofResponse {
        session_id,
        proof_id: session.register_proof_node(node)?,
    })
}
