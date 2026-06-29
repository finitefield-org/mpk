//! Minimal checked proof strategies for local AI proof repair loops.

use mpk_core::TermNode;
use serde::{Deserialize, Serialize};

use crate::{
    check_api::CheckNodeRequest,
    proof_api::{
        ApiProofId, ApplyProofRequest, ExactProofRequest, IntroProofRequest, ProofResponse,
        ReflProofRequest,
    },
    session::{ApiError, ApiErrorCode, ApiService, SessionId},
    term_api::ApiTermId,
    theory_strategy::TheoryStrategyCandidate,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProveRequest {
    pub session_id: SessionId,
    pub expected_type: ApiTermId,
    #[serde(default)]
    pub exact_terms: Vec<ApiTermId>,
    #[serde(default)]
    pub refl_terms: Vec<ApiTermId>,
    #[serde(default)]
    pub split: bool,
    #[serde(default)]
    pub apply: Vec<ApplyStrategyCandidate>,
    #[serde(default)]
    pub theory: Vec<TheoryStrategyCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyStrategyCandidate {
    pub function_proof: ApiProofId,
    #[serde(default)]
    pub argument_proofs: Vec<ApiProofId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyProveResponse {
    pub session_id: SessionId,
    pub expected_type: ApiTermId,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_id: Option<ApiProofId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term_id: Option<ApiTermId>,
    pub attempts: Vec<StrategyAttempt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyAttempt {
    pub strategy: StrategyKind,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_id: Option<ApiProofId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term_id: Option<ApiTermId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    Exact,
    Refl,
    Split,
    Apply,
    Theory,
}

impl ApiService {
    pub fn proof_try_strategies(
        &mut self,
        request: StrategyProveRequest,
    ) -> Result<StrategyProveResponse, ApiError> {
        let session_id = request.session_id;
        {
            let session = self.require_session_mut(&session_id)?;
            session.require_term_id(request.expected_type, "expected_type")?;
        }

        let mut attempts = Vec::new();
        for term in request.exact_terms {
            let attempt = self.try_exact_strategy(&session_id, request.expected_type, term);
            if let Some(response) =
                record_attempt(&session_id, request.expected_type, &mut attempts, attempt)
            {
                return Ok(response);
            }
        }

        for term in request.refl_terms {
            let attempt = self.try_refl_strategy(&session_id, request.expected_type, term);
            if let Some(response) =
                record_attempt(&session_id, request.expected_type, &mut attempts, attempt)
            {
                return Ok(response);
            }
        }

        if request.split {
            let attempt = self.try_split_strategy(&session_id, request.expected_type);
            if let Some(response) =
                record_attempt(&session_id, request.expected_type, &mut attempts, attempt)
            {
                return Ok(response);
            }
        }

        for candidate in request.apply {
            let attempt = self.try_apply_strategy(&session_id, request.expected_type, candidate);
            if let Some(response) =
                record_attempt(&session_id, request.expected_type, &mut attempts, attempt)
            {
                return Ok(response);
            }
        }

        for candidate in request.theory {
            let attempt =
                self.try_theory_strategy_candidate(&session_id, request.expected_type, candidate);
            if let Some(response) =
                record_attempt(&session_id, request.expected_type, &mut attempts, attempt)
            {
                return Ok(response);
            }
        }

        Ok(StrategyProveResponse {
            session_id,
            expected_type: request.expected_type,
            ok: false,
            proof_id: None,
            term_id: None,
            attempts,
        })
    }

    fn try_exact_strategy(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
        term: ApiTermId,
    ) -> StrategyAttempt {
        match self.proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term,
            expected_type,
        }) {
            Ok(proof) => self.check_strategy_proof(session_id, StrategyKind::Exact, proof),
            Err(error) => StrategyAttempt::failed(StrategyKind::Exact, None, error),
        }
    }

    fn try_refl_strategy(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
        term: ApiTermId,
    ) -> StrategyAttempt {
        match self.proof_refl(ReflProofRequest {
            session_id: session_id.clone(),
            term,
            expected_type,
        }) {
            Ok(proof) => self.check_strategy_proof(session_id, StrategyKind::Refl, proof),
            Err(error) => StrategyAttempt::failed(StrategyKind::Refl, None, error),
        }
    }

    fn try_split_strategy(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
    ) -> StrategyAttempt {
        match self.construct_split_proof(session_id, expected_type) {
            Ok(proof) => self.check_strategy_proof(session_id, StrategyKind::Split, proof),
            Err(error) => StrategyAttempt::failed(StrategyKind::Split, None, error),
        }
    }

    fn try_apply_strategy(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
        candidate: ApplyStrategyCandidate,
    ) -> StrategyAttempt {
        match self.proof_apply(ApplyProofRequest {
            session_id: session_id.clone(),
            function_proof: candidate.function_proof,
            argument_proofs: candidate.argument_proofs,
            expected_type,
        }) {
            Ok(proof) => self.check_strategy_proof(session_id, StrategyKind::Apply, proof),
            Err(error) => StrategyAttempt::failed(StrategyKind::Apply, None, error),
        }
    }

    fn try_theory_strategy_candidate(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
        candidate: TheoryStrategyCandidate,
    ) -> StrategyAttempt {
        match self.try_theory_strategy(session_id, expected_type, candidate) {
            Ok(proof) => self.check_strategy_proof(session_id, StrategyKind::Theory, proof),
            Err(error) => StrategyAttempt::failed(StrategyKind::Theory, None, error),
        }
    }

    fn construct_split_proof(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
    ) -> Result<ProofResponse, ApiError> {
        let (domain_type, body_type, local_var) = {
            let session = self.require_session_mut(session_id)?;
            let expected = session.require_term_id(expected_type, "expected_type")?;
            let TermNode::Pi { ty, body } = session.terms().node(expected).clone() else {
                return Err(strategy_not_applicable(
                    StrategyKind::Split,
                    "expected_type is not a Pi term",
                ));
            };
            let domain_type = session.register_term_id(ty)?;
            let body_type = session.register_term_id(body)?;
            let local_var = session.terms_mut().var(0);
            let local_var = session.register_term_id(local_var)?;
            (domain_type, body_type, local_var)
        };
        let body_proof = self.proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: local_var,
            expected_type: body_type,
        })?;
        self.proof_intro(IntroProofRequest {
            session_id: session_id.clone(),
            domain_type,
            body_proof: body_proof.proof_id,
            expected_type,
        })
    }

    fn check_strategy_proof(
        &mut self,
        session_id: &SessionId,
        strategy: StrategyKind,
        proof: ProofResponse,
    ) -> StrategyAttempt {
        match self.proof_check_node(CheckNodeRequest {
            session_id: session_id.clone(),
            proof_id: proof.proof_id,
        }) {
            Ok(checked) => StrategyAttempt {
                strategy,
                ok: true,
                proof_id: Some(proof.proof_id),
                term_id: Some(checked.term_id),
                error: None,
            },
            Err(error) => StrategyAttempt::failed(strategy, Some(proof.proof_id), error),
        }
    }
}

impl StrategyAttempt {
    fn failed(strategy: StrategyKind, proof_id: Option<ApiProofId>, error: ApiError) -> Self {
        Self {
            strategy,
            ok: false,
            proof_id,
            term_id: None,
            error: Some(error),
        }
    }

    fn success(&self) -> Option<(ApiProofId, ApiTermId)> {
        Some((self.proof_id?, self.term_id?)).filter(|_| self.ok)
    }
}

fn record_attempt(
    session_id: &SessionId,
    expected_type: ApiTermId,
    attempts: &mut Vec<StrategyAttempt>,
    attempt: StrategyAttempt,
) -> Option<StrategyProveResponse> {
    let success = attempt.success();
    attempts.push(attempt);
    success.map(|(proof_id, term_id)| StrategyProveResponse {
        session_id: session_id.clone(),
        expected_type,
        ok: true,
        proof_id: Some(proof_id),
        term_id: Some(term_id),
        attempts: attempts.clone(),
    })
}

fn strategy_not_applicable(strategy: StrategyKind, reason: &'static str) -> ApiError {
    ApiError::new(
        ApiErrorCode::StrategyNotApplicable,
        format!("{strategy:?} strategy is not applicable: {reason}"),
        Some("strategy".to_owned()),
        Some(reason.to_owned()),
    )
}
