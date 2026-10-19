use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_division_bounds,
    generate_go_practical_word_signed_division_refinement,
    import_go_practical_word_signed_division_bounds, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_DIVISION_BOUNDS_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{ext}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
}
#[test]
fn actual_division_bounds_minimum_classification_and_exact_import() {
    let f = generate_go_practical_word_signed_division_bounds().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "division bounds declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-division-bounds", f.certificate_bytes(), true);
    assert!(
        c.declarations.len() <= 4096
            && c.term_table.len() <= 131072
            && f.certificate_bytes().len() <= 2_097_152
    );
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        f,
        generate_go_practical_word_signed_division_bounds().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_signed_division_bounds(f.certificate_bytes()).unwrap()
    );
    let old = generate_go_practical_word_signed_division_refinement().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "a38c1903fb7add8bf7156b484d747612b52a985899227453a772f3a3b6498487"
    );
    let reference = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    );
    let actual = dependencies::declarations(&c);
    assert_eq!(reference.len(), 3897);
    for (name, value) in &reference {
        assert_eq!(
            actual.get(name),
            Some(value),
            "old refinement declaration changed: {name}"
        );
    }
    println!(
        "matched complete prior named refinement={}",
        reference.len()
    );
    for w in [8, 16, 32, 64] {
        for name in [
            "QuotientGapBalance",
            "QuotientHalfBound",
            "RemainderStrictHalfBound",
            "NegativeUnitSound",
            "NegativeUnitComplete",
            "OverflowIsMinimumMinusOne",
            "QuotientCorrectionMinimum",
            "RemainderCorrectionZero",
            "QuotientRefinementMinimum",
            "QuotientRefinementExact",
            "RemainderRefinementExact",
            "SignedDivisionMinimumConservation",
            "SignedDivisionExactConservation",
            "MinimumSourceQuotientValue",
            "MinimumSourceRemainderZero",
        ] {
            assert!(actual.contains_key(&format!(
                "Mpk.Go.Practical.Word.SignedDivision.Bounds.W{w}.{name}"
            )));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_signed_division_bounds(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_division_bounds(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
