//! Fast source-free verifier orchestration crate.

#![forbid(unsafe_code)]

pub mod cache;
pub mod decl_driver;
pub mod json_output;
pub mod profile;
pub mod proof_check;
pub mod proof_structural;
pub mod proof_theory;
pub mod verifier;

pub use cache::{
    CheckerCache, CheckerCacheMetrics, CheckerCacheOperationMetrics, CheckerCacheStats,
};
pub use decl_driver::{
    check_declarations, DeclarationCheckError, DeclarationCheckErrorKind, DeclarationCheckReport,
};
pub use json_output::{
    render_axiom_report_json, render_verification_error_json, render_verification_report_json,
    verify_certificate_bytes_axiom_report_json_output, verify_certificate_bytes_json,
    verify_certificate_bytes_json_output, VerificationJsonOutput,
};
pub use profile::{
    profile_certificate_bytes, KernelProfileError, KernelProfileHotspot, KernelProfileReport,
    KernelProfileStage, KernelProfileTableCounts, KernelProfileTimings,
};
pub use proof_check::{
    check_proof_nodes, check_proof_nodes_with_profile, ProofCheckError, ProofCheckErrorKind,
    ProofCheckProfile, ProofCheckReport,
};
pub use verifier::{
    verify_certificate_bytes, VerificationError, VerificationErrorKind, VerificationReport,
};
