use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_division_prerequisites,
    generate_go_practical_word_signed_division_refinement,
    import_go_practical_word_signed_division_refinement, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_REFINEMENT_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
}

#[test]
fn actual_signed_division_refinement_exact_import_and_all_prerequisites() {
    let foundation = generate_go_practical_word_signed_division_refinement().unwrap();
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    println!(
        "signed-refinement declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        foundation.certificate_bytes().len()
    );
    emit(
        "word-signed-refinement",
        foundation.certificate_bytes(),
        true,
    );
    assert!(c.declarations.len() <= 4096);
    assert!(c.term_table.len() <= 131072);
    assert!(foundation.certificate_bytes().len() <= 2_097_152);
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        foundation,
        generate_go_practical_word_signed_division_refinement().unwrap()
    );
    assert_eq!(
        foundation,
        import_go_practical_word_signed_division_refinement(foundation.certificate_bytes())
            .unwrap()
    );
    let original = generate_go_practical_word_signed_division_prerequisites().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(original.certificate_bytes())),
        "2d23c58db425fc11d653c8c7966db69a0f71495d614712cd7b8abc0f2246e11d"
    );
    let reference = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(original.certificate_bytes()).unwrap(),
    );
    let current = dependencies::declarations(&c);
    for (name, value) in &reference {
        assert_eq!(
            current.get(name),
            Some(value),
            "original prerequisite changed: {name}"
        );
    }
    println!("matched complete named prerequisites={}", reference.len());
    assert_eq!(reference.len(), 3483);
    for width in [8, 16, 32, 64] {
        for name in [
            "SignedSelectionRefinement",
            "MagnitudeDivisionConservation",
            "MagnitudeRemainderBound",
            "QuotientRefinement",
            "RemainderRefinement",
            "MathematicalDivisionConservation",
            "SignedDivisionConservation",
        ] {
            assert!(current.contains_key(&format!(
                "Mpk.Go.Practical.Word.SignedDivision.W{width}.{name}"
            )));
        }
    }
    let mut altered = foundation.certificate_bytes().to_vec();
    altered.push(0);
    assert_eq!(
        import_go_practical_word_signed_division_refinement(&altered),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_division_refinement(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
