use mpk_vc::go_practical_ordinary::{
    generate_go_practical_range_foundation, generate_go_practical_word_foundation,
    import_go_practical_word_foundation, GoOrdinaryError,
};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_WORD_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", r.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{ext}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
}

#[test]
fn actual_full_width_word_circuits_and_universal_basic_laws_check_without_axioms() {
    let f = generate_go_practical_word_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.source_manifest.is_none()
            && c.proof_node_table.is_empty()
            && c.theory_certificates.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for width in [8, 16, 32, 64] {
        for member in [
            "Carrier",
            "Add",
            "Sub",
            "Neg",
            "Mul",
            "SignedDiv",
            "UnsignedDiv",
            "SignedRem",
            "UnsignedRem",
            "ShiftLeft",
            "ShiftRight",
            "SignedShiftRight",
            "DivideDefined",
            "SignedCountDefined",
            "AddZeroAt",
            "AndOnesAt",
            "XorZeroAt",
            "NotNotAt",
            "SignedTo64",
            "UnsignedTo8",
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.W{width}.{member}")));
        }
    }
    emit("word-foundation", f.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        f.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_word_import_is_deterministic_and_preserves_complete_range_bytes() {
    let f = generate_go_practical_word_foundation().unwrap();
    assert_eq!(f, generate_go_practical_word_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_foundation(f.certificate_bytes()).unwrap()
    );
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let old = generate_go_practical_range_foundation().unwrap();
    let r = mpk_kernel::verify_certificate_bytes_json_output(old.certificate_bytes());
    assert!(
        r.accepted
            && r.json
                .contains("ddb538b929f4ef0e04d6ac74f119d561e2df132cdba409aa61b0259b99179a70")
    );
    emit("retained-range-basis", old.certificate_bytes(), true);
}
