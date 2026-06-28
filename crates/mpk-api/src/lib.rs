//! Local AI proof API session crate.
//!
//! `mpk-api` is an untrusted helper layer for AI proof construction and repair
//! loops. API state is never an acceptance signal; certificates still have to
//! pass the canonical kernel and checker path.

#![forbid(unsafe_code)]

pub mod session;
pub mod term_api;

pub use session::{
    ApiError, ApiErrorCode, ApiService, ApiSession, ProofProfile, SessionId, SessionStatus,
    SessionSummary, StartSessionRequest, StartSessionResponse,
};
pub use term_api::{
    ApiTermId, AppTermRequest, BinderTermRequest, ConstTermRequest, LetTermRequest,
    SortTermRequest, TermResponse, VarTermRequest,
};
