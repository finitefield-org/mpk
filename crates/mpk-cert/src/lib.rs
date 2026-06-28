//! Canonical certificate encoding, decoding, hashing, imports, exports, and axiom reports.

#![forbid(unsafe_code)]

pub mod axiom_report;
pub mod binary_tags;
pub mod canonical;
pub mod decode;
pub mod encode;
pub mod export;
pub mod hash;
pub mod imports;

#[cfg(test)]
mod cert_basic;

pub use axiom_report::{
    axiom_report_hash_for_report, build_axiom_report, encode_axiom_report, AxiomReportBuildError,
    AxiomReportBuildErrorKind,
};
pub use binary_tags::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};
pub use canonical::{
    decode_canonical_certificate, validate_canonical_certificate, CanonicalError,
    CanonicalErrorKind,
};
pub use decode::{decode_certificate, DecodeError, DecodeErrorKind};
pub use encode::{encode_certificate, encode_unsigned_varint, Certificate, CertificateHashes};
pub use export::{
    build_export_block, build_export_block_for_declarations, declaration_interface_hash,
    encode_export_block, export_block_hash, ExportBuildError, ExportBuildErrorKind,
};
pub use hash::{
    axiom_report_hash, certificate_hash, export_hash, hash_hex, hash_with_domain, level_hash,
    term_hash, HashDomain,
};
pub use imports::{
    sort_import_table, validate_certificate_imports, validate_import_table, ImportValidationError,
    ImportValidationErrorKind,
};
