use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_bounds_foundation,
    generate_go_practical_word_zero_foundation, import_go_practical_word_zero_foundation,
    GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_ZERO_TEST_OUTPUT") {
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
fn actual_direct_zero_exact_import_and_named_prerequisites() {
    let f = generate_go_practical_word_zero_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "direct-zero declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-zero-foundation", f.certificate_bytes(), true);
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(f, generate_go_practical_word_zero_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_zero_foundation(f.certificate_bytes()).unwrap()
    );
    let old = generate_go_practical_word_signed_bounds_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "8895b359ea61393aaf241c82e26366498f06cd12a0645e5ba37e3cbe14c41a31"
    );
    let reference = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    );
    let mut retained = 0;
    for (name, v) in dependencies::declarations(&c) {
        if !name.starts_with("Mpk.Go.Practical.Word.Zero.") {
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
            "IsZeroBits",
            "IsZeroNatural",
            "DivideDefinedNatural",
            "DefinedDivisorPositive",
            "ZeroSign",
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.Zero.W{w}.{name}")));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_zero_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_zero_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
