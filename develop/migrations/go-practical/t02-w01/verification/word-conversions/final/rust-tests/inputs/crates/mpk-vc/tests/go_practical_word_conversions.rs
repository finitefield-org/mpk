use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation, generate_go_practical_word_conversions,
    generate_go_practical_word_order_foundation, import_go_practical_word_conversions,
    GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_conversions_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[path = "support/go_word_conversions_observations.rs"]
mod observations;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_CONVERSIONS_TEST_OUTPUT") {
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
fn actual_all_width_conversions_and_shift_count_normalization_exact_import() {
    let f = generate_go_practical_word_conversions().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "conversions declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-conversions", f.certificate_bytes(), true);
    assert!(
        c.declarations.len() <= 4096
            && c.term_table.len() <= 131072
            && f.certificate_bytes().len() <= 2_097_152
    );
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        (
            c.declarations.len(),
            c.term_table.len(),
            f.certificate_bytes().len()
        ),
        (1882, 31495, 483095)
    );
    assert_eq!(f, generate_go_practical_word_conversions().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_conversions(f.certificate_bytes()).unwrap()
    );
    let mut old = std::collections::BTreeMap::new();
    let order = generate_go_practical_word_order_foundation().unwrap();
    let arithmetic = generate_go_practical_word_arithmetic_foundation().unwrap();
    for (bytes, hash) in [
        (
            order.certificate_bytes(),
            "c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa",
        ),
        (
            arithmetic.certificate_bytes(),
            "2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), hash);
        for (name, value) in
            dependencies::declarations(&mpk_cert::decode_canonical_certificate(bytes).unwrap())
        {
            if let Some(prior) = old.insert(name, value.clone()) {
                assert_eq!(prior, value);
            }
        }
    }
    let actual = dependencies::declarations(&c);
    let mut retained = 0;
    for (name, value) in &actual {
        if !name.starts_with("Mpk.Go.Practical.Word.Conversions.") {
            assert_eq!(
                old.get(name),
                Some(value),
                "old conversion prerequisite changed: {name}"
            );
            retained += 1;
        }
    }
    println!("matched retained named conversion prerequisites={retained}");
    let different = controls::honest_different_component();
    emit("honest-different-conversions", &different, true);
    assert_eq!(
        import_go_practical_word_conversions(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_conversions(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_conversions(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
