use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    generate_go_practical_word_division_foundation,
    generate_go_practical_word_signed_product_foundation,
    import_go_practical_word_signed_product_foundation,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_PRODUCT_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", r.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{ext}")))
                .unwrap();
            file.write_all(&raw).unwrap();
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
fn actual_signed_product_exact_import_and_named_prerequisite_preservation() {
    let f = generate_go_practical_word_signed_product_foundation().unwrap();
    assert_eq!(
        f,
        generate_go_practical_word_signed_product_foundation().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_signed_product_foundation(f.certificate_bytes()).unwrap()
    );
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "signed-product declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit(
        "word-signed-product-foundation",
        f.certificate_bytes(),
        true,
    );
    assert!(c.axiom_report.entries.is_empty());
    let old = generate_go_practical_word_arithmetic_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c"
    );
    let old_c = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    let reference = dependencies::declarations(&old_c);
    let mut retained = 0;
    for (name, value) in dependencies::declarations(&c) {
        if !name.starts_with("Mpk.Go.Practical.Word.SignedProduct.") {
            assert_eq!(
                Some(&value),
                reference.get(&name),
                "retained named declaration changed: {name}"
            );
            retained += 1;
        }
    }
    println!("matched named prerequisites={retained}");
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(
                generate_go_practical_word_division_foundation()
                    .unwrap()
                    .certificate_bytes()
            )
        ),
        "64ef60d8a3517bede84945e89af06ab155ec04cc4346f1f828b20f8bbda4f462"
    );
}
