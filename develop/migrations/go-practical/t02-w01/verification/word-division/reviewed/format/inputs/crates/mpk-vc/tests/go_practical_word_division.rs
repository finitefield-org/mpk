use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    generate_go_practical_word_division_foundation, generate_go_practical_word_order_foundation,
    import_go_practical_word_division_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};

#[path = "support/go_word_division_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[path = "support/go_word_division_observations.rs"]
mod observations;

#[test]
fn closed_prerequisite_declarations_match_prior_components_by_name() {
    let mut reference = std::collections::BTreeMap::new();
    for bytes in [
        generate_go_practical_word_order_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_arithmetic_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
    ] {
        let c = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        for (name, value) in dependencies::declarations(&c) {
            if let Some(prior) = reference.insert(name.clone(), value.clone()) {
                assert_eq!(prior, value, "conflicting reference declaration {name}");
            }
        }
    }
    let f = generate_go_practical_word_division_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let mut retained = 0;
    for (name, value) in dependencies::declarations(&c) {
        if !name.starts_with("Mpk.Go.Practical.Word.Division.") {
            assert_eq!(
                Some(&value),
                reference.get(&name),
                "retained named type/proof/body/metadata changed: {name}"
            );
            retained += 1;
        }
    }
    assert_eq!(retained, 2198);
    println!(
        "all {retained} retained prerequisites match prior named types/bodies/proofs/metadata"
    );
}

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_DIVISION_TEST_OUTPUT") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (name, raw) in [
            (&format!("{name}.mpcert"), bytes.to_vec()),
            (
                &format!("{name}.rust.json"),
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
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    if !accepted {
        let r: serde_json::Value = serde_json::from_str(&r.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}

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
    emit("word-division-foundation", f.certificate_bytes(), true);
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
    assert_eq!(c.declarations.len(), 3588);
    assert_eq!(c.term_table.len(), 82264);
    assert!(f.certificate_bytes().len() <= 2_097_152);
    assert!(!c.name_table.iter().any(
        |n| n.starts_with("Mpk.Go.Practical.Int.") || n.starts_with("Mpk.Go.Practical.Range.")
    ));
    for name in [
        "Std.Nat.induct",
        "Std.Nat.zero",
        "Std.Nat.succ",
        "Std.Bool.recType",
        "Mpk.Go.Practical.Word.Order.W64.EqualBitsBlock6At0",
    ] {
        assert!(c
            .declarations
            .iter()
            .any(|d| c.name_table[d.name as usize] == name));
    }
    for w in [8, 16, 32, 64] {
        for name in [
            "RestoreConservation".to_owned(),
            "RestoreBound".into(),
            "PrefixValue".into(),
            format!("StageConservation{w}"),
            format!("StageDividendFold{}", w - 1),
            "DefinedQuotientValue".into(),
            "DefinedRemainderValue".into(),
            "UnsignedDivisionConservation".into(),
            "UnsignedRemainderBound".into(),
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.Division.W{w}.{name}")));
        }
    }
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_division_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_division_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let other = controls::honest_different_component();
    emit("honest-different-division-component", &other, true);
    assert_eq!(
        import_go_practical_word_division_foundation(&other),
        Err(GoOrdinaryError::Linkage)
    );
}
