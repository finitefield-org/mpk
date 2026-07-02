use mpk_cert::encode::TheoryCertificate;
use mpk_kernel::proof_theory::check_theory_certificate;
use mpk_theory::{
    encode_bool_certificate, encode_linarith_certificate, BoolCertificate, BoolCertificateRow,
    BoolExpr, FarkasMultiplier, LinarithCertificate, LinearInequality, LinearTerm,
    BOOL_CERT_FORMAT, LINARITH_CERT_FORMAT,
};

#[test]
fn bool_theory_certificate_encoder_payload_is_accepted_by_kernel() {
    let certificate = BoolCertificate {
        variable_count: 0,
        root: BoolExpr::Const(true),
        rows: vec![BoolCertificateRow {
            assignment: Vec::new(),
            normalized_value: true,
        }],
    };

    let checked = check_theory_certificate(&TheoryCertificate {
        format: BOOL_CERT_FORMAT.to_owned(),
        payload: encode_bool_certificate(&certificate),
    })
    .expect("encoded bool certificate is accepted by kernel");

    assert_eq!(checked.format, BOOL_CERT_FORMAT);
}

#[test]
fn linarith_theory_certificate_encoder_payload_is_accepted_by_kernel() {
    let certificate = LinarithCertificate {
        premises: vec![LinearInequality::new(vec![LinearTerm::new(0, -1)], 0)],
        goal: LinearInequality::new(vec![LinearTerm::new(0, -1)], 0),
        combination: vec![FarkasMultiplier::new(0, 1)],
    };

    let checked = check_theory_certificate(&TheoryCertificate {
        format: LINARITH_CERT_FORMAT.to_owned(),
        payload: encode_linarith_certificate(&certificate),
    })
    .expect("encoded linarith certificate is accepted by kernel");

    assert_eq!(checked.format, LINARITH_CERT_FORMAT);
}
