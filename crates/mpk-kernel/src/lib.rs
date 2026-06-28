//! Fast source-free verifier orchestration crate.

#![forbid(unsafe_code)]

pub mod decl_driver;
pub mod verifier;

pub use decl_driver::{
    check_declarations, DeclarationCheckError, DeclarationCheckErrorKind, DeclarationCheckReport,
};
pub use verifier::{
    verify_certificate_bytes, VerificationError, VerificationErrorKind, VerificationReport,
};
