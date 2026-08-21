//! Local AI proof API session crate.
//!
//! `mpk-api` is an untrusted helper layer for AI proof construction and repair
//! loops. API state is never an acceptance signal; certificates still have to
//! pass the canonical kernel and checker path.

#![forbid(unsafe_code)]

pub mod batch;
pub mod check_api;
pub mod diagnostics;
pub mod jsonl;
pub mod policy_strategy;
pub mod proof_api;
pub mod session;
pub mod strategies;
pub mod term_api;
pub mod theory_strategy;

// AI API v1 stays deliberately unreachable from released builds until the
// atomic cutover publishes these modules in GO-VIR-02-T12.
#[cfg(test)]
mod v1_router;
#[cfg(test)]
mod v1_tests;
#[cfg(test)]
mod vc_api;
#[cfg(test)]
mod vir_api;

pub use batch::{
    BatchCandidate, BatchCheckMode, BatchCheckRequest, BatchCheckResponse, BatchCheckSummary,
    CandidateVerdict,
};
pub use check_api::{CheckNodeRequest, CheckNodeResponse};
pub use diagnostics::{RepairDiagnostic, RepairDiagnosticRequest, RepairDiagnosticResponse};
pub use jsonl::{
    export_batch_candidates_jsonl, import_batch_candidates_jsonl, JsonlExportRequest,
    JsonlExportResponse, JsonlImportRequest, JsonlImportResponse,
};
pub use policy_strategy::{
    PolicyObligationDescriptor, PolicyObligationPattern, PolicyStrategyError,
    PolicyStrategyErrorCode, PolicyStrategyMetadata, PolicyStrategyProfile,
    PAYMENT_POLICY_ALPHA_PROFILE,
};
pub use proof_api::{
    ApiProofId, ApplyProofRequest, ConvProofRequest, ExactProofRequest, IntroProofRequest,
    ProofResponse, ReflProofRequest,
};
pub use session::{
    ApiError, ApiErrorCode, ApiService, ApiSession, ProofProfile, SessionId, SessionStatus,
    SessionSummary, StartSessionRequest, StartSessionResponse,
};
pub use strategies::{
    ApplyStrategyCandidate, StrategyAttempt, StrategyKind, StrategyProveRequest,
    StrategyProveResponse,
};
pub use term_api::{
    ApiTermId, AppTermRequest, BinderTermRequest, ConstTermRequest, LetTermRequest,
    SortTermRequest, TermResponse, VarTermRequest,
};
pub use theory_strategy::{
    theory_strategy_certificate, theory_strategy_certificate_evidence, TheoryStrategyCandidate,
    TheoryStrategyCertificateEvidence, TheoryStrategyKind,
};
