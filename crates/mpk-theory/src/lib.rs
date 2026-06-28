//! Independently checkable MVP theory certificate helpers.

#![forbid(unsafe_code)]

pub mod bitvec_eval;
pub mod bool_cert;
pub mod linarith_cert;

pub use bitvec_eval::{
    evaluate_bitvec_expr, BitVecBinaryOp, BitVecComparisonOp, BitVecEvalError, BitVecEvalErrorKind,
    BitVecEvalResult, BitVecExpr, BitVecUnaryOp, BitVecValue, MAX_BITVEC_EXPR_NODES,
    SUPPORTED_BITVEC_WIDTHS,
};
pub use bool_cert::{
    check_bool_certificate, check_bool_certificate_payload, decode_bool_certificate, BoolCertError,
    BoolCertErrorKind, BoolCertificate, BoolCertificateRow, BoolCertificateSummary, BoolExpr,
    BOOL_CERT_FORMAT, MAX_BOOL_EXPR_NODES, MAX_BOOL_VARIABLES,
};
pub use linarith_cert::{
    check_linarith_certificate, FarkasMultiplier, LinarithCertError, LinarithCertErrorKind,
    LinarithCertificate, LinarithCertificateSummary, LinearInequality, LinearTerm,
    LINARITH_CERT_FORMAT, MAX_LINARITH_COMBINATION_TERMS, MAX_LINARITH_PREMISES,
    MAX_LINARITH_TERMS_PER_INEQUALITY, MAX_LINARITH_VARIABLES,
};
