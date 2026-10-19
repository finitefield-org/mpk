use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_conversions, generate_go_practical_word_shift_foundation,
    generate_go_practical_word_signed_division_bounds, import_go_practical_word_shift_foundation,
    GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SHIFTS_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", r.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
}
#[test]
fn ordinary_shift_foundation_exact_import_and_original_preservation() {
    let f = generate_go_practical_word_shift_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "shift foundation declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-shifts", f.certificate_bytes(), true);
    assert!(
        c.declarations.len() <= 4096
            && c.term_table.len() <= 131072
            && f.certificate_bytes().len() <= 2097152
    );
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(f, generate_go_practical_word_shift_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_shift_foundation(f.certificate_bytes()).unwrap()
    );
    let old = generate_go_practical_word_conversions().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "d8453a91807c0add99575b7b76a18cb9bf3e65d78895b5c8d4a5f75c1dcc09a1"
    );
    let reference = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    );
    let current = dependencies::declarations(&c);
    assert_eq!(reference.len(), 1882);
    for (name, value) in reference {
        assert_eq!(
            current.get(&name),
            Some(&value),
            "old conversion declaration changed: {name}"
        );
    }
    let division = generate_go_practical_word_signed_division_bounds().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(division.certificate_bytes())),
        "a0158c8449e9f44c36279dd735166fe84f5b30f1d30a8caa337d32e256927e56"
    );
    println!(
        "matched all1882 prior conversion declarations and original signed-division composer bytes"
    );
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_shift_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_shift_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}
