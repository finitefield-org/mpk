use std::panic::{catch_unwind, AssertUnwindSafe};

use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block,
    encode::{
        AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
        ProofNode, TermNode, TheoryCertificate,
    },
    encode_certificate, export_block_hash,
};
use mpk_kernel::verify_certificate_bytes;
use mpk_theory::{ARRAY_CERT_FORMAT, BOOL_CERT_FORMAT, LINARITH_CERT_FORMAT};

const BITVEC_CERT_FORMAT: &str = "mpk.bitvec-ground.v0";

#[test]
fn malformed_theory_certificate_payloads_reject_deterministically_without_panics() {
    for (name, format, payload) in malformed_theory_payload_cases() {
        let bytes = encode_certificate(&theory_certificate(format, payload));
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            (
                verify_certificate_bytes(&bytes),
                verify_certificate_bytes(&bytes),
            )
        }));

        assert!(outcome.is_ok(), "case `{name}` panicked");
        let (first, second) = outcome.expect("checked above");
        assert_eq!(first, second, "case `{name}` is nondeterministic");
        assert!(first.is_err(), "case `{name}` unexpectedly verified");
    }
}

fn malformed_theory_payload_cases() -> Vec<(String, &'static str, Vec<u8>)> {
    [
        (BOOL_CERT_FORMAT, "bool", b"MPKBOOL0".as_slice()),
        (BITVEC_CERT_FORMAT, "bitvec", b"MPKBVGC0".as_slice()),
        (LINARITH_CERT_FORMAT, "linarith", b"MPKLINR0".as_slice()),
        (ARRAY_CERT_FORMAT, "array", b"MPKARRY0".as_slice()),
    ]
    .into_iter()
    .flat_map(|(format, label, magic)| {
        [
            (format!("{label}-empty"), Vec::new()),
            (format!("{label}-single-zero"), vec![0]),
            (format!("{label}-single-ff"), vec![0xff]),
            (format!("{label}-magic-only"), magic.to_vec()),
            (
                format!("{label}-varint-overflow-tail"),
                [magic, &[0xff; 16]].concat(),
            ),
            (
                format!("{label}-trailing-noise"),
                [magic, &[0, 1, 2, 3, 0xff]].concat(),
            ),
        ]
        .map(|(name, payload)| (name, format, payload))
    })
    .collect()
}

fn theory_certificate(format: &str, payload: Vec<u8>) -> Certificate {
    finalize_certificate(Certificate {
        module: "Fuzz.TheoryCertificateSmoke".to_owned(),
        imports: Vec::new(),
        name_table: vec!["Fuzz.TheoryCertificateSmoke.x".to_owned()],
        level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
        term_table: vec![
            TermNode::Sort(0),
            TermNode::Sort(1),
            TermNode::Const {
                global: 0,
                levels: Vec::new(),
            },
        ],
        proof_node_table: vec![ProofNode::Theory {
            theory_certificate: 0,
            expected_type: 0,
        }],
        declarations: vec![Declaration {
            name: 0,
            kind: DeclarationKind::Axiom { ty: 0 },
        }],
        theory_certificates: vec![TheoryCertificate {
            format: format.to_owned(),
            payload,
        }],
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: None,
        hashes: CertificateHashes::default(),
    })
}

fn finalize_certificate(mut certificate: Certificate) -> Certificate {
    certificate.export_block = build_export_block(&certificate).expect("export block builds");
    certificate.axiom_report = build_axiom_report(&certificate).expect("axiom report builds");
    certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    certificate
}
