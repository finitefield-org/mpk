use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    generate_go_practical_word_numeric_foundation, generate_go_practical_word_order_foundation,
    import_go_practical_word_order_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};

#[path = "support/go_word_order_controls.rs"]
mod controls;
#[path = "support/go_word_order_observations.rs"]
mod observations;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_ORDER_TEST_OUTPUT") {
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
    if !accepted {
        let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}

#[test]
fn actual_order_component_exact_import_and_retained_numeric_arithmetic() {
    let f = generate_go_practical_word_order_foundation().unwrap();
    assert_eq!(f, generate_go_practical_word_order_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_order_foundation(f.certificate_bytes()).unwrap()
    );
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131_072);
    assert!(f.certificate_bytes().len() <= 2_097_152);
    println!(
        "order declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-order-foundation", f.certificate_bytes(), true);
    assert!(c.axiom_report.entries.is_empty());
    let old = generate_go_practical_word_numeric_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "fd48d9fa0f5d26634e489bb03ebd68208dd9e7f6e2620e9a3cdf966f47d112ea"
    );
    emit("retained-numeric-basis", old.certificate_bytes(), true);
    assert_eq!(
        import_go_practical_word_order_foundation(old.certificate_bytes()),
        Err(GoOrdinaryError::Linkage)
    );
    let old_c = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(&c.term_table[..old_c.term_table.len()], old_c.term_table);
    for (a, b) in old_c.declarations.iter().zip(&c.declarations) {
        assert_eq!(
            old_c.name_table[a.name as usize],
            c.name_table[b.name as usize]
        );
        assert_eq!(a.kind, b.kind);
    }
    let arithmetic = generate_go_practical_word_arithmetic_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(arithmetic.certificate_bytes())),
        "2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c"
    );
    for w in [8, 16, 32, 64] {
        for name in [
            "Complement",
            "ValueBound",
            "SubtractionConservation",
            "UnsignedLessNatural",
            "UnsignedLessEqualNatural",
            "CarryPairEquality",
            "EqualNatural",
            "NotEqualNatural",
            "ZeroValue",
            "IsZeroNatural",
            "DivideDefinedNatural",
            "DefinedDivisorPositive",
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.Order.W{w}.{name}")));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_order_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_order_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let different = controls::honest_different_component();
    emit("honest-different-order-component", &different, true);
    assert_eq!(
        import_go_practical_word_order_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
}
