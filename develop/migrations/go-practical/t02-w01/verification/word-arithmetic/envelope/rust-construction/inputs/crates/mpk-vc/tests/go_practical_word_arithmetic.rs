use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    generate_go_practical_word_numeric_foundation, import_go_practical_word_arithmetic_foundation,
    GoOrdinaryError,
};
use sha2::{Digest, Sha256};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_WORD_ARITHMETIC_TEST_OUTPUT") {
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
        let r: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}

#[test]
fn actual_arithmetic_component_exact_import_and_retained_numeric_basis() {
    let f = generate_go_practical_word_arithmetic_foundation().unwrap();
    assert_eq!(
        f,
        generate_go_practical_word_arithmetic_foundation().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_arithmetic_foundation(f.certificate_bytes()).unwrap()
    );
    emit("word-arithmetic-foundation", f.certificate_bytes(), true);
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    assert!(c.axiom_report.entries.is_empty());
    println!(
        "arithmetic declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    let old = generate_go_practical_word_numeric_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "fd48d9fa0f5d26634e489bb03ebd68208dd9e7f6e2620e9a3cdf966f47d112ea"
    );
    emit("retained-numeric-basis", old.certificate_bytes(), true);
    let old_c = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(&c.term_table[..old_c.term_table.len()], old_c.term_table);
    for (before, after) in old_c.declarations.iter().zip(&c.declarations) {
        assert_eq!(
            old_c.name_table[before.name as usize],
            c.name_table[after.name as usize]
        );
        assert_eq!(before.kind, after.kind);
    }
    assert_eq!(
        import_go_practical_word_arithmetic_foundation(old.certificate_bytes()),
        Err(GoOrdinaryError::Linkage)
    );
    for width in [8, 16, 32, 64] {
        assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
            == format!("Mpk.Go.Practical.Word.Numeric.W{width}.PushLowConservation")));
    }
}
