//! Independently checkable MVP theory certificate helpers.

#![forbid(unsafe_code)]

pub mod bool_cert;

pub use bool_cert::{
    check_bool_certificate, check_bool_certificate_payload, decode_bool_certificate, BoolCertError,
    BoolCertErrorKind, BoolCertificate, BoolCertificateRow, BoolCertificateSummary, BoolExpr,
    BOOL_CERT_FORMAT, MAX_BOOL_EXPR_NODES, MAX_BOOL_VARIABLES,
};
