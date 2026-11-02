use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation, generate_go_practical_word_order_foundation,
    generate_go_practical_word_signed_magnitude_foundation,
    generate_go_practical_word_signed_product_foundation,
    import_go_practical_word_signed_magnitude_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_MAGNITUDE_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", r.json).into_bytes()),
        ] {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{ext}")))
                .unwrap();
            f.write_all(&raw).unwrap();
        }
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    if !accepted {
        let r: serde_json::Value = serde_json::from_str(&r.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let d: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}
#[test]
fn actual_signed_magnitude_exact_import_and_named_prerequisite_preservation() {
    let f = generate_go_practical_word_signed_magnitude_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "signed-magnitude declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit(
        "word-signed-magnitude-foundation",
        f.certificate_bytes(),
        true,
    );
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        f,
        generate_go_practical_word_signed_magnitude_foundation().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_signed_magnitude_foundation(f.certificate_bytes()).unwrap()
    );
    let refs = [
        generate_go_practical_word_arithmetic_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_order_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_signed_product_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
    ];
    assert_eq!(
        format!("{:x}", Sha256::digest(&refs[0])),
        "2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&refs[2])),
        "5b0d67641b9a4c5de86a32411a29f00d6ebdd5cdb049f1b21839e79526a66eda"
    );
    let mut reference = std::collections::BTreeMap::new();
    for raw in &refs {
        let c = mpk_cert::decode_canonical_certificate(raw).unwrap();
        for (name, v) in dependencies::declarations(&c) {
            if let Some(old) = reference.insert(name.clone(), v.clone()) {
                assert_eq!(old, v, "inconsistent prior named reference: {name}");
            }
        }
    }
    let mut retained = 0;
    for (name, v) in dependencies::declarations(&c) {
        if !name.starts_with("Mpk.Go.Practical.Word.SignedMagnitude.") {
            assert_eq!(
                reference.get(&name),
                Some(&v),
                "retained named declaration changed: {name}"
            );
            retained += 1;
        }
    }
    println!("matched named prerequisites={retained}");
    for w in [8, 16, 32, 64] {
        for name in [
            "NegationCarryBalance",
            "NegationValueSum",
            "NegationIsZero",
            "NegationDivideDefined",
            "SignDivideDefined",
            "MagnitudeIsZero",
            "MagnitudeDivideDefined",
            "SignedMagnitudeEquivalent",
            "SignedNegationConservation",
            "NegationNoOverflowReflect",
            "NegationNoOverflowComplete",
            "NegationNoOverflowIntegerEquality",
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.SignedMagnitude.W{w}.{name}")));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_signed_magnitude_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_magnitude_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
