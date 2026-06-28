//! Canonical certificate encoding, decoding, hashing, imports, exports, and axiom reports.

#![forbid(unsafe_code)]

pub mod binary_tags;
pub mod canonical;
pub mod decode;
pub mod encode;
pub mod hash;
pub mod imports;

pub use binary_tags::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};
pub use canonical::{
    decode_canonical_certificate, validate_canonical_certificate, CanonicalError,
    CanonicalErrorKind,
};
pub use decode::{decode_certificate, DecodeError, DecodeErrorKind};
pub use encode::{encode_certificate, encode_unsigned_varint, Certificate, CertificateHashes};
pub use hash::{
    axiom_report_hash, certificate_hash, export_hash, hash_hex, hash_with_domain, level_hash,
    term_hash, HashDomain,
};
pub use imports::{
    sort_import_table, validate_certificate_imports, validate_import_table, ImportValidationError,
    ImportValidationErrorKind,
};
