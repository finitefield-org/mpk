use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_shift_foundation, import_go_practical_word_shift_foundation,
    GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let report = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SHIFT_COUNTS_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", report.json).into_bytes()),
        ] {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{extension}")))
                .unwrap();
            f.write_all(&raw).unwrap();
        }
    }
    assert_eq!(report.accepted, accepted, "{name}: {}", report.json);
}
#[test]
fn full64_count_bounds_preserve_all_previous_proofs_and_exact_ownership() {
    let f = generate_go_practical_word_shift_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "count foundation declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-shift-counts", f.certificate_bytes(), true);
    assert!(
        c.declarations.len() <= 4096
            && c.term_table.len() <= 131072
            && f.certificate_bytes().len() <= 2097152
            && c.axiom_report.entries.is_empty()
    );
    let previous=include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/word-shifts/metadata-corrected/rust-tests/certificates/word-shifts.mpcert");
    assert_eq!(
        format!("{:x}", Sha256::digest(previous)),
        "a723bc68d2f95b74592a942c2737631526d6bd7a92b26e5142af812550aac8e9"
    );
    let old =
        dependencies::declarations(&mpk_cert::decode_canonical_certificate(previous).unwrap());
    let current = dependencies::declarations(&c);
    assert_eq!(old.len(), 3105);
    for (n, v) in old {
        assert_eq!(
            current.get(&n),
            Some(&v),
            "old shift declaration changed: {n}"
        );
    }
    println!("all3105 previous named proof/definition/level/family identities match");
    assert_eq!(f, generate_go_practical_word_shift_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_shift_foundation(f.certificate_bytes()).unwrap()
    );
    let mut altered = f.certificate_bytes().to_vec();
    altered.push(0);
    assert_eq!(
        import_go_practical_word_shift_foundation(&altered),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_shift_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}
