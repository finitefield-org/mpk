//! Checked theory-certificate strategy builders.

use mpk_cert::encode::{ProofNode, TheoryCertificate};
use mpk_kernel::proof_theory::BITVEC_CERT_FORMAT;
use mpk_theory::{ARRAY_CERT_FORMAT, BOOL_CERT_FORMAT, LINARITH_CERT_FORMAT};
use serde::{Deserialize, Serialize};

use crate::{
    proof_api::ProofResponse,
    session::{ApiError, ApiErrorCode, ApiService, ApiSession, ProofProfile, SessionId},
    term_api::ApiTermId,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TheoryStrategyCandidate {
    pub theory: TheoryStrategyKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TheoryStrategyKind {
    BoolTautology,
    BitVecGround,
    Linarith,
    ArrayReadWrite,
}

impl ApiService {
    pub(crate) fn try_theory_strategy(
        &mut self,
        session_id: &SessionId,
        expected_type: ApiTermId,
        candidate: TheoryStrategyCandidate,
    ) -> Result<ProofResponse, ApiError> {
        let certificate = build_theory_certificate(candidate.theory);
        let session = self.require_session_mut(session_id)?;
        if session.proof_profile() != ProofProfile::MvpStrict {
            return Err(theory_not_applicable(
                "theory strategies require the mvp-strict proof profile",
            ));
        }
        let expected_type = cert_term_id(session, expected_type)?;
        let theory_certificate = session.register_theory_certificate(certificate)?;
        let proof_id = session.register_proof_node(ProofNode::Theory {
            theory_certificate,
            expected_type,
        })?;

        Ok(ProofResponse {
            session_id: session_id.clone(),
            proof_id,
        })
    }
}

fn build_theory_certificate(theory: TheoryStrategyKind) -> TheoryCertificate {
    match theory {
        TheoryStrategyKind::BoolTautology => TheoryCertificate {
            format: BOOL_CERT_FORMAT.to_owned(),
            payload: bool_tautology_payload(),
        },
        TheoryStrategyKind::BitVecGround => TheoryCertificate {
            format: BITVEC_CERT_FORMAT.to_owned(),
            payload: bitvec_ground_payload(),
        },
        TheoryStrategyKind::Linarith => TheoryCertificate {
            format: LINARITH_CERT_FORMAT.to_owned(),
            payload: linarith_payload(),
        },
        TheoryStrategyKind::ArrayReadWrite => TheoryCertificate {
            format: ARRAY_CERT_FORMAT.to_owned(),
            payload: array_read_write_payload(),
        },
    }
}

fn cert_term_id(session: &ApiSession, term_id: ApiTermId) -> Result<u32, ApiError> {
    let core = session.require_term_id(term_id, "expected_type")?;
    u32::try_from(core.index()).map_err(|_| {
        ApiError::new(
            ApiErrorCode::TermIdOverflow,
            "core term id exceeded certificate u32 term ids",
            Some("expected_type".to_owned()),
            Some(core.index().to_string()),
        )
    })
}

fn theory_not_applicable(reason: &'static str) -> ApiError {
    ApiError::new(
        ApiErrorCode::StrategyNotApplicable,
        format!("Theory strategy is not applicable: {reason}"),
        Some("strategy".to_owned()),
        Some(reason.to_owned()),
    )
}

fn bool_tautology_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"MPKBOOL0");
    payload.push(0);
    payload.push(0x01);
    payload.extend_from_slice(&1u16.to_be_bytes());
    payload.push(0);
    payload.push(1);
    payload
}

fn bitvec_ground_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"MPKBVGC0");
    payload.push(0);
    payload.extend_from_slice(&[0x02, 0x03, 0x00, 0x08, 0x01, 0x00, 0x08, 0x01]);
    payload.push(3);

    payload.push(0);
    payload.push(1);
    payload.push(0x02);
    payload.push(0x00);
    payload.push(0x00);
    payload.push(0);
    push_bitvec_result(&mut payload, 0x08, 1);

    payload.push(1);
    payload.push(1);
    payload.push(0x03);
    payload.push(0x00);
    payload.push(0x00);
    payload.push(0);
    push_bitvec_result(&mut payload, 0x08, 1);

    payload.push(2);
    payload.push(0);
    payload.push(0x02);
    payload.push(0x03);
    payload.push(2);
    push_bitvec_result(&mut payload, 0x08, 1);
    push_bitvec_result(&mut payload, 0x08, 1);
    push_bitvec_result(&mut payload, 0x08, 2);

    push_bitvec_result(&mut payload, 0x08, 2);
    payload
}

fn push_bitvec_result(payload: &mut Vec<u8>, width_tag: u8, bits: u64) {
    payload.push(0x00);
    payload.push(width_tag);
    let len = usize::from(width_tag / 8);
    payload.extend_from_slice(&bits.to_be_bytes()[8 - len..]);
}

fn linarith_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"MPKLINR0");
    payload.push(0);
    payload.push(0);
    payload.extend_from_slice(&0i128.to_be_bytes());
    payload.push(0);
    payload
}

fn array_read_write_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"MPKARRY0");
    payload.push(1);
    payload.extend_from_slice(&0u32.to_be_bytes());
    payload.extend_from_slice(&2u32.to_be_bytes());
    payload.push(1);
    payload.push(0x00);
    payload.push(0x01);
    payload.push(0x00);
    payload.extend_from_slice(&0u32.to_be_bytes());
    payload.extend_from_slice(&1u32.to_be_bytes());
    payload.push(0x00);
    payload.extend_from_slice(&7u32.to_be_bytes());
    payload.extend_from_slice(&1u32.to_be_bytes());
    payload.push(0x00);
    payload.push(0x00);
    payload.extend_from_slice(&7u32.to_be_bytes());
    payload
}
