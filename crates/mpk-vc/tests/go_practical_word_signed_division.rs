use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_division_foundation, generate_go_practical_word_foundation,
    generate_go_practical_word_signed_bounds_foundation,
    generate_go_practical_word_signed_division_prerequisites,
    generate_go_practical_word_zero_foundation,
    import_go_practical_word_signed_division_prerequisites, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_signed_division_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[path = "support/go_word_signed_division_observations.rs"]
mod observations;
fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_DIVISION_TEST_OUTPUT") {
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
fn replacement(name: &str) -> Option<String> {
    for w in [8, 16, 32, 64] {
        for (old, n) in [
            ("Word.Order", "IsZeroNatural"),
            ("Word.Order", "DefinedDivisorPositive"),
            ("Word.SignedMagnitude", "ZeroSign"),
        ] {
            if name == format!("Mpk.Go.Practical.{old}.W{w}.{n}") {
                return Some(format!("Mpk.Go.Practical.Word.Zero.W{w}.{n}"));
            }
        }
    }
    None
}
fn mapped(name: &str) -> String {
    let n = replacement(name).unwrap_or_else(|| name.into());
    for (old, new) in [
        ("Word.Order", "Order"),
        ("Word.Division", "Unsigned"),
        ("Word.SignedMagnitude", "Magnitude"),
        ("Word.SignedBounds", "Bounds"),
        ("Word.SignedProduct", "Arithmetic"),
        ("Word.Zero", "Zero"),
    ] {
        if let Some(s) = n.strip_prefix(&format!("Mpk.Go.Practical.{old}.")) {
            return format!("Mpk.Go.Practical.Word.SignedDivision.Prerequisite.{new}.{s}");
        }
    }
    n
}
#[test]
fn complete_signed_division_prerequisites_exact_import_and_mapped_proof_identity() {
    let f = generate_go_practical_word_signed_division_prerequisites().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    println!(
        "signed-division prerequisites declarations={} terms={} bytes={}",
        c.declarations.len(),
        c.term_table.len(),
        f.certificate_bytes().len()
    );
    emit("signed-division-prerequisites", f.certificate_bytes(), true);
    assert_eq!(c.declarations.len(), 3483);
    assert_eq!(c.term_table.len(), 85883);
    assert_eq!(f.certificate_bytes().len(), 1255288);
    assert!(c.axiom_report.entries.is_empty());
    assert_eq!(
        f,
        generate_go_practical_word_signed_division_prerequisites().unwrap()
    );
    assert_eq!(
        f,
        import_go_practical_word_signed_division_prerequisites(f.certificate_bytes()).unwrap()
    );
    let inputs = [
        generate_go_practical_word_division_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_signed_bounds_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_zero_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
    ];
    for (i, hash) in [
        (
            0,
            "64ef60d8a3517bede84945e89af06ab155ec04cc4346f1f828b20f8bbda4f462",
        ),
        (
            1,
            "8895b359ea61393aaf241c82e26366498f06cd12a0645e5ba37e3cbe14c41a31",
        ),
        (
            3,
            "23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(&inputs[i])), hash);
    }
    let mut original = std::collections::BTreeMap::new();
    let mut reference = std::collections::BTreeMap::new();
    for raw in &inputs {
        let mut c = mpk_cert::decode_canonical_certificate(raw).unwrap();
        let names = c
            .declarations
            .iter()
            .map(|d| c.name_table[d.name as usize].clone())
            .collect::<Vec<_>>();
        for (n, v) in dependencies::declarations(&c) {
            if let Some(old) = original.insert(n.clone(), v.clone()) {
                assert_eq!(old, v, "inconsistent original: {n}");
            }
        }
        for (d, n) in c.declarations.iter_mut().zip(&names) {
            let id = c.name_table.len() as u32;
            c.name_table.push(mapped(n));
            d.name = id;
        }
        let renamed = dependencies::declarations(&c);
        for n in names {
            if replacement(&n).is_none() {
                let m = mapped(&n);
                let v = renamed[&m].clone();
                if let Some(old) = reference.insert(m.clone(), v.clone()) {
                    assert_eq!(old, v, "inconsistent renamed: {m}");
                }
            }
        }
    }
    for (name, value) in &original {
        if let Some(target) = replacement(name) {
            assert_eq!(
                value[1], original[&target][1],
                "replacement theorem interface: {name}"
            );
        }
    }
    for (name, value) in dependencies::declarations(&c) {
        assert_eq!(
            reference.get(&name),
            Some(&value),
            "mapped declaration changed: {name}"
        );
    }
    println!("every mapped declaration matches; actual substitutions=12");
    for w in [8, 16, 32, 64] {
        for (role, names) in [
            (
                "Unsigned",
                vec!["UnsignedDivisionConservation", "UnsignedRemainderBound"],
            ),
            (
                "Bounds",
                vec![
                    "MagnitudeHalfBound",
                    "MagnitudeHalfIsMinimum",
                    "MagnitudeSignIsMinimum",
                ],
            ),
            (
                "Magnitude",
                vec![
                    "SignedMagnitudeEquivalent",
                    "MagnitudeDivideDefined",
                    "NegationValueSum",
                ],
            ),
            (
                "Zero",
                vec!["IsZeroNatural", "DefinedDivisorPositive", "ZeroSign"],
            ),
        ] {
            for n in names {
                assert!(c.declarations.iter().any(|g| c.name_table[g.name as usize]
                    == format!(
                        "Mpk.Go.Practical.Word.SignedDivision.Prerequisite.{role}.W{w}.{n}"
                    )));
            }
        }
    }
    let mut altered = f.certificate_bytes().to_vec();
    altered.push(0);
    assert_eq!(
        import_go_practical_word_signed_division_prerequisites(&altered),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_division_prerequisites(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let different = controls::honest_different_component();
    emit("honest-different-division-prerequisites", &different, true);
    assert_eq!(
        import_go_practical_word_signed_division_prerequisites(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let changed = controls::honest_different_signed_div_source();
    emit("honest-different-signed-div-definition", &changed, true);
    assert_eq!(
        import_go_practical_word_signed_division_prerequisites(&changed),
        Err(GoOrdinaryError::Linkage)
    );
}
