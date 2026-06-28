//! Local AI proof API session crate.
//!
//! `mpk-api` is an untrusted helper layer for AI proof construction and repair
//! loops. API state is never an acceptance signal; certificates still have to
//! pass the canonical kernel and checker path.

#![forbid(unsafe_code)]

pub mod batch;
pub mod check_api;
pub mod diagnostics;
pub mod proof_api;
pub mod session;
pub mod term_api;

pub use batch::{
    BatchCandidate, BatchCheckMode, BatchCheckRequest, BatchCheckResponse, BatchCheckSummary,
    CandidateVerdict,
};
pub use check_api::{CheckNodeRequest, CheckNodeResponse};
pub use diagnostics::{RepairDiagnostic, RepairDiagnosticRequest, RepairDiagnosticResponse};
pub use proof_api::{
    ApiProofId, ApplyProofRequest, ConvProofRequest, ExactProofRequest, IntroProofRequest,
    ProofResponse, ReflProofRequest,
};
pub use session::{
    ApiError, ApiErrorCode, ApiService, ApiSession, ProofProfile, SessionId, SessionStatus,
    SessionSummary, StartSessionRequest, StartSessionResponse,
};
pub use term_api::{
    ApiTermId, AppTermRequest, BinderTermRequest, ConstTermRequest, LetTermRequest,
    SortTermRequest, TermResponse, VarTermRequest,
};
