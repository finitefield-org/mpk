use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    generate_go_practical_word_division_foundation, generate_go_practical_word_order_foundation,
    import_go_practical_word_division_foundation,
};
use sha2::{Digest, Sha256};

#[test]
fn actual_division_closed_dependency_assembly_preserves_prior_component_bytes() {
    let f = generate_go_practical_word_division_foundation().unwrap();
    assert_eq!(f, generate_go_practical_word_division_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_division_foundation(f.certificate_bytes()).unwrap()
    );
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "division declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    let r = mpk_kernel::verify_certificate_bytes_json_output(f.certificate_bytes());
    if let Some(out) = std::env::var_os("MPK_GO_WORD_DIVISION_TEST_OUTPUT") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (name, raw) in [
            (
                "word-division-foundation.mpcert",
                f.certificate_bytes().to_vec(),
            ),
            (
                "word-division-foundation.rust.json",
                format!("{}\n", r.json).into_bytes(),
            ),
        ] {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(name))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert!(r.accepted, "{}", r.json);
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(
                generate_go_practical_word_order_foundation()
                    .unwrap()
                    .certificate_bytes()
            )
        ),
        "c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(
                generate_go_practical_word_arithmetic_foundation()
                    .unwrap()
                    .certificate_bytes()
            )
        ),
        "2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c"
    );
}
