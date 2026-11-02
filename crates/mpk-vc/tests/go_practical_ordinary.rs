use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_boolean_foundation, import_go_practical_boolean_foundation,
    GoOrdinaryError,
};

fn save_observed_certificate(name: &str, bytes: &[u8]) {
    if let Some(directory) = std::env::var_os("MPK_GO_ORDINARY_TEST_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(name), bytes).unwrap();
    }
}

fn constant(certificate: &mut Certificate, name: &str) -> u32 {
    let global = certificate
        .declarations
        .iter()
        .position(|d| certificate.name_table[d.name as usize] == name)
        .unwrap() as u32;
    let id = certificate.term_table.len() as u32;
    certificate.term_table.push(TermNode::Const {
        global,
        levels: vec![],
    });
    id
}

fn application(certificate: &mut Certificate, function: u32, arguments: Vec<u32>) -> u32 {
    let id = certificate.term_table.len() as u32;
    certificate.term_table.push(TermNode::App {
        function,
        arguments,
    });
    id
}

#[test]
fn ordinary_boolean_foundation_is_closed_and_zero_axiom() {
    let foundation = generate_go_practical_boolean_foundation().unwrap();
    let decoded = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    assert!(decoded.imports.is_empty() && decoded.theory_certificates.is_empty());
    assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
    let checked = mpk_kernel::verify_certificate_bytes_json_output(foundation.certificate_bytes());
    assert!(checked.accepted, "{}", checked.json);
    save_observed_certificate("boolean-foundation.mpcert", foundation.certificate_bytes());
    println!("{}", checked.json);
    println!(
        "bytes={} terms={} declarations={}",
        foundation.certificate_bytes().len(),
        decoded.term_table.len(),
        decoded.declarations.len()
    );
}

#[test]
fn exact_component_import_rejects_changed_certificate() {
    let first = generate_go_practical_boolean_foundation().unwrap();
    let second = generate_go_practical_boolean_foundation().unwrap();
    assert_eq!(first.certificate_bytes(), second.certificate_bytes());
    assert_eq!(
        import_go_practical_boolean_foundation(first.certificate_bytes()).unwrap(),
        first
    );
    let mut changed = first.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_boolean_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn an_incorrect_ordinary_boolean_claim_is_rejected() {
    let foundation = generate_go_practical_boolean_foundation().unwrap();
    let mut certificate =
        mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    let boolean = constant(&mut certificate, "Std.Bool");
    let no = constant(&mut certificate, "Std.Bool.false");
    let yes = constant(&mut certificate, "Std.Bool.true");
    let eq = constant(&mut certificate, "Std.Eq");
    let refl = constant(&mut certificate, "Std.Eq.refl");
    let ty = application(&mut certificate, eq, vec![boolean, no, yes]);
    let proof = application(&mut certificate, refl, vec![boolean, no]);
    let theorem = certificate
        .declarations
        .iter_mut()
        .find(|d| certificate.name_table[d.name as usize] == "Mpk.Go.Practical.Bool.DoubleNot")
        .unwrap();
    theorem.kind = DeclarationKind::Theorem { ty, proof };
    certificate.export_block = mpk_cert::build_export_block(&certificate).unwrap();
    certificate.axiom_report = mpk_cert::build_axiom_report(&certificate).unwrap();
    certificate.hashes.export_hash = mpk_cert::export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash =
        mpk_cert::axiom_report_hash_for_report(&certificate.axiom_report);
    let bytes = mpk_cert::encode_certificate_bounded(&certificate, 2_097_152).unwrap();
    let decoded = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
    let checked = mpk_kernel::verify_certificate_bytes_json_output(&bytes);
    assert!(
        !checked.accepted,
        "incorrect Eq Bool false true was accepted"
    );
    assert!(
        checked.json.contains("CORE_TYPE_MISMATCH"),
        "{}",
        checked.json
    );
    assert_eq!(
        import_go_practical_boolean_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    save_observed_certificate("incorrect-boolean-claim.mpcert", &bytes);
    println!("{}", checked.json);
}
