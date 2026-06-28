#![no_main]

use libfuzzer_sys::fuzz_target;

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
const MAX_FUZZ_PAYLOAD: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let Some((selector, payload)) = data.split_first() else {
        return;
    };
    let payload = &payload[..payload.len().min(MAX_FUZZ_PAYLOAD)];
    let certificate = theory_certificate(format_for_selector(*selector), payload.to_vec());
    let bytes = encode_certificate(&certificate);

    let first = verify_certificate_bytes(&bytes);
    let second = verify_certificate_bytes(&bytes);
    assert_eq!(first, second);
});

fn format_for_selector(selector: u8) -> &'static str {
    match selector % 4 {
        0 => BOOL_CERT_FORMAT,
        1 => BITVEC_CERT_FORMAT,
        2 => LINARITH_CERT_FORMAT,
        _ => ARRAY_CERT_FORMAT,
    }
}

fn theory_certificate(format: &str, payload: Vec<u8>) -> Certificate {
    finalize_certificate(Certificate {
        module: "Fuzz.TheoryCertificate".to_owned(),
        imports: Vec::new(),
        name_table: vec!["Fuzz.TheoryCertificate.x".to_owned()],
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
