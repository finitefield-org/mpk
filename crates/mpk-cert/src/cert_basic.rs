use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate, encode_certificate, export_block_hash, hash_hex,
};

use crate::encode::{
    AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode, TermNode,
};

const CERT_BASIC_FIXTURE_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-basic");

#[derive(Debug)]
struct HashRecord {
    export_hash: String,
    axiom_report_hash: String,
    certificate_hash: String,
}

fn finalize_certificate(mut certificate: Certificate) -> Certificate {
    certificate.export_block = build_export_block(&certificate).expect("export block builds");
    certificate.axiom_report = build_axiom_report(&certificate).expect("axiom report builds");
    certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    certificate
}

fn zero_axiom_certificate() -> Certificate {
    finalize_certificate(Certificate {
        module: "Example.Basic.ZeroAxiom".to_owned(),
        imports: Vec::new(),
        name_table: Vec::new(),
        level_table: Vec::new(),
        term_table: Vec::new(),
        proof_node_table: Vec::new(),
        declarations: Vec::new(),
        theory_certificates: Vec::new(),
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: None,
        hashes: CertificateHashes::default(),
    })
}

fn one_theorem_certificate() -> Certificate {
    finalize_certificate(Certificate {
        module: "Example.Basic.OneTheorem".to_owned(),
        imports: Vec::new(),
        name_table: vec!["Example.Basic.OneTheorem.sort0IsSort1".to_owned()],
        level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
        term_table: vec![TermNode::Sort(0), TermNode::Sort(1)],
        proof_node_table: Vec::new(),
        declarations: vec![Declaration {
            name: 0,
            kind: DeclarationKind::Theorem { ty: 1, proof: 0 },
        }],
        theory_certificates: Vec::new(),
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: None,
        hashes: CertificateHashes::default(),
    })
}

fn expected_certificates() -> Vec<(&'static str, Certificate)> {
    vec![
        ("zero-axiom", zero_axiom_certificate()),
        ("one-theorem", one_theorem_certificate()),
    ]
}

fn decode_hex_fixture(path: &Path) -> Vec<u8> {
    let contents = fs::read_to_string(path).expect("hex fixture is readable");
    let hex = contents
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert_eq!(hex.len() % 2, 0, "fixture hex must use full bytes");

    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk).expect("fixture hex is utf8");
            u8::from_str_radix(byte, 16).expect("fixture hex byte is valid")
        })
        .collect()
}

fn read_hash_records() -> BTreeMap<String, HashRecord> {
    let path = Path::new(CERT_BASIC_FIXTURE_DIR).join("hashes.csv");
    let contents = fs::read_to_string(path).expect("hash fixture is readable");
    let mut records = BTreeMap::new();

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line_index == 0 {
            assert_eq!(
                line,
                "fixture,export_hash,axiom_report_hash,certificate_hash"
            );
            continue;
        }

        let fields = line.split(',').collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            4,
            "hash fixture line {} must be csv",
            line_index + 1
        );
        records.insert(
            fields[0].to_owned(),
            HashRecord {
                export_hash: fields[1].to_owned(),
                axiom_report_hash: fields[2].to_owned(),
                certificate_hash: fields[3].to_owned(),
            },
        );
    }

    records
}

#[test]
fn cert_basic_fixtures_decode_and_hash() {
    let hash_records = read_hash_records();
    let expected = expected_certificates();
    assert_eq!(hash_records.len(), expected.len());

    for (name, expected_certificate) in expected {
        let path = Path::new(CERT_BASIC_FIXTURE_DIR).join(format!("{name}.hex"));
        let bytes = decode_hex_fixture(&path);
        let decoded =
            decode_canonical_certificate(&bytes).expect("basic certificate fixture decodes");

        assert_eq!(
            decoded, expected_certificate,
            "{name} fixture shape drifted"
        );
        assert_eq!(encode_certificate(&decoded), bytes, "{name} re-encodes");

        let rebuilt_export_block =
            build_export_block(&decoded).expect("fixture export block rebuilds");
        assert_eq!(
            rebuilt_export_block, decoded.export_block,
            "{name} export block is recomputable"
        );
        let rebuilt_axiom_report =
            build_axiom_report(&decoded).expect("fixture axiom report rebuilds");
        assert_eq!(
            rebuilt_axiom_report, decoded.axiom_report,
            "{name} axiom report is recomputable"
        );

        let recomputed_export_hash = export_block_hash(&decoded.export_block);
        let recomputed_axiom_report_hash = axiom_report_hash_for_report(&decoded.axiom_report);
        assert_eq!(
            decoded.hashes.export_hash, recomputed_export_hash,
            "{name} embedded export hash matches"
        );
        assert_eq!(
            decoded.hashes.axiom_report_hash, recomputed_axiom_report_hash,
            "{name} embedded axiom report hash matches"
        );

        let hash_record = hash_records.get(name).expect("hash record exists");
        assert_eq!(hash_hex(&recomputed_export_hash), hash_record.export_hash);
        assert_eq!(
            hash_hex(&recomputed_axiom_report_hash),
            hash_record.axiom_report_hash
        );
        assert_eq!(
            hash_hex(&certificate_hash(&bytes)),
            hash_record.certificate_hash
        );
    }
}

#[test]
fn cert_basic_fixture_shapes_are_minimal() {
    let zero_axiom = decode_canonical_certificate(&decode_hex_fixture(
        &Path::new(CERT_BASIC_FIXTURE_DIR).join("zero-axiom.hex"),
    ))
    .expect("zero-axiom fixture decodes");
    assert_eq!(zero_axiom.module, "Example.Basic.ZeroAxiom");
    assert!(zero_axiom.declarations.is_empty());
    assert_eq!(zero_axiom.axiom_report.summary.total_axiom_count, 0);

    let one_theorem = decode_canonical_certificate(&decode_hex_fixture(
        &Path::new(CERT_BASIC_FIXTURE_DIR).join("one-theorem.hex"),
    ))
    .expect("one-theorem fixture decodes");
    assert_eq!(one_theorem.module, "Example.Basic.OneTheorem");
    assert_eq!(one_theorem.declarations.len(), 1);
    assert!(matches!(
        one_theorem.declarations[0].kind,
        DeclarationKind::Theorem { ty: 1, proof: 0 }
    ));
    assert_eq!(one_theorem.axiom_report.summary.total_axiom_count, 0);
}
