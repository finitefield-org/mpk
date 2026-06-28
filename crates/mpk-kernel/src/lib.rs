//! Fast source-free verifier orchestration crate.

#![forbid(unsafe_code)]

pub mod cache;
pub mod decl_driver;
pub mod proof_check;
pub mod verifier;

pub use cache::{CheckerCache, CheckerCacheStats};
pub use decl_driver::{
    check_declarations, DeclarationCheckError, DeclarationCheckErrorKind, DeclarationCheckReport,
};
pub use proof_check::{
    check_proof_nodes, check_proof_nodes_with_profile, ProofCheckError, ProofCheckErrorKind,
    ProofCheckProfile, ProofCheckReport,
};
pub use verifier::{
    verify_certificate_bytes, VerificationError, VerificationErrorKind, VerificationReport,
};
