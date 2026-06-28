//! Canonical certificate encoding, decoding, hashing, imports, exports, and axiom reports.

#![forbid(unsafe_code)]

pub mod binary_tags;
pub mod encode;

pub use binary_tags::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};
pub use encode::{encode_certificate, encode_unsigned_varint, Certificate, CertificateHashes};
