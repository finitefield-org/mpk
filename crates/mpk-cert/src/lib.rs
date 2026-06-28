//! Canonical certificate encoding, decoding, hashing, imports, exports, and axiom reports.

#![forbid(unsafe_code)]

pub mod binary_tags;

pub use binary_tags::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};
