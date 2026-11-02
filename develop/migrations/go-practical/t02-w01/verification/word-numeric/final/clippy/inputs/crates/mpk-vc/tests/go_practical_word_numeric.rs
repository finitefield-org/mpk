use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_foundation, generate_go_practical_word_numeric_foundation,
    import_go_practical_word_numeric_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};

#[path = "support/go_word_numeric_controls.rs"]
mod controls;
#[path = "support/go_word_numeric_observations.rs"]
mod observations;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_WORD_NUMERIC_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, bytes) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&bytes).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}

#[test]
fn fresh_actual_addition_numeric_component_and_exact_import() {
    let foundation = generate_go_practical_word_numeric_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    assert_eq!(
        foundation,
        generate_go_practical_word_numeric_foundation().unwrap()
    );
    assert_eq!(
        foundation,
        import_go_practical_word_numeric_foundation(bytes).unwrap()
    );
    emit("word-numeric-foundation", bytes, true);
    let certificate = mpk_cert::decode_certificate(bytes).unwrap();
    assert!(certificate.axiom_report.entries.is_empty());
    println!(
        "numeric declarations={} terms={} bytes={}",
        certificate.declarations.len(),
        certificate.term_table.len(),
        bytes.len()
    );
    let old = generate_go_practical_word_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c"
    );
    emit("retained-word-basis", old.certificate_bytes(), true);
    assert_eq!(
        import_go_practical_word_numeric_foundation(old.certificate_bytes()),
        Err(GoOrdinaryError::Linkage)
    );
    let old = mpk_cert::decode_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(
        &certificate.term_table[..old.term_table.len()],
        old.term_table
    );
    for (before, after) in old.declarations.iter().zip(&certificate.declarations) {
        assert_eq!(
            old.name_table[before.name as usize],
            certificate.name_table[after.name as usize]
        );
        assert_eq!(before.kind, after.kind);
    }
    for width in [8, 16, 32, 64] {
        for suffix in [
            "Unsigned",
            "Signed",
            "Modulus",
            "Sign",
            "NoOverflow",
            "AdditionConservation",
            "SignedCrossSum",
            "SignedAdditionConservation",
            "NoOverflowReflect",
            "SignedPositive",
            "SignedNegative",
            "SignedAddition",
        ] {
            let name = format!("Mpk.Go.Practical.Word.Numeric.W{width}.{suffix}");
            assert!(
                certificate
                    .declarations
                    .iter()
                    .any(|d| certificate.name_table[d.name as usize] == name),
                "{name}"
            );
        }
    }
    let mut changed = bytes.to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_numeric_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_numeric_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let different = controls::honest_different_component();
    emit("honest-different-numeric-component", &different, true);
    assert_eq!(
        import_go_practical_word_numeric_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
}
