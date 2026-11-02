use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_bounds_foundation,
    generate_go_practical_word_signed_magnitude_foundation,
    import_go_practical_word_signed_bounds_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_BOUNDS_TEST_OUTPUT") {
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
fn actual_signed_half_bounds_exact_import_and_all_prior_named_declarations() {
    let f = generate_go_practical_word_signed_bounds_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "signed-bounds declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("word-signed-bounds-foundation", f.certificate_bytes(), true);
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        f,
        generate_go_practical_word_signed_bounds_foundation().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_signed_bounds_foundation(f.certificate_bytes()).unwrap()
    );
    let old = generate_go_practical_word_signed_magnitude_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "5f3d83784114bfb471f9a8c305628c15c04e3f3b1398a566d4721c754912c976"
    );
    let reference = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    );
    let mut retained = 0;
    for (name, v) in dependencies::declarations(&c) {
        if !name.starts_with("Mpk.Go.Practical.Word.SignedBounds.") {
            assert_eq!(
                reference.get(&name),
                Some(&v),
                "retained named declaration changed: {name}"
            );
            retained += 1;
        }
    }
    assert_eq!(retained, 2163);
    println!("matched named prerequisites={retained}");
    for d in 0..=6 {
        for name in [
            "HalfDouble",
            "HalfPositive",
            "ValueSplit",
            "LowComplementBalance",
            "LowStrictHalf",
        ] {
            assert!(c.declarations.iter().any(|g| c.name_table[g.name as usize]
                == format!("Mpk.Go.Practical.Word.SignedBounds.{name}{d}")));
        }
    }
    for w in [8, 16, 32, 64] {
        for name in [
            "ValueSplit",
            "MagnitudeGap",
            "MagnitudeHalfBalance",
            "MagnitudeHalfBound",
            "NonnegativeHalfBound",
            "NonnegativeMagnitudeHalfBound",
            "NegativeMagnitudeLowBalance",
            "MagnitudeHalfForcesSign",
            "ValueHalfForcesSign",
            "MagnitudeHalfReflect",
            "MagnitudeHalfComplete",
            "MagnitudeHalfIsMinimum",
        ] {
            assert!(c.declarations.iter().any(|g| c.name_table[g.name as usize]
                == format!("Mpk.Go.Practical.Word.SignedBounds.W{w}.{name}")));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_signed_bounds_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_bounds_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}
