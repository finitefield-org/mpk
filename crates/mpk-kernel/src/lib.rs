//! Fast source-free verifier orchestration crate.

#![forbid(unsafe_code)]

pub mod verifier;

pub use verifier::{
    verify_certificate_bytes, VerificationError, VerificationErrorKind, VerificationReport,
};
