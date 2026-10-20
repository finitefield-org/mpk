use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_equality_foundation,
    generate_go_practical_word_signed_order_foundation,
    import_go_practical_word_signed_equality_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_signed_equality_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_EQUALITY_TEST_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(out.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let r: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let d: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}

#[test]
fn exact_component_ownership_is_separate_from_core_acceptance() {
    let component = generate_go_practical_word_signed_equality_foundation().unwrap();
    assert_eq!(
        component,
        import_go_practical_word_signed_equality_foundation(component.certificate_bytes()).unwrap()
    );
    assert_eq!(
        component,
        generate_go_practical_word_signed_equality_foundation().unwrap()
    );
    let different = controls::honest_different_component();
    inspect("honest-different-signed-equality", &different, true);
    assert_eq!(
        import_go_practical_word_signed_equality_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = component.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_word_signed_equality_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_signed_equality_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn signed_equality_rejects_changed_source_claims_and_real_premises() {
    controls::semantic_controls();
}
#[test]
fn actual_signed_equality_component_and_complete_old_identities() {
    let f = generate_go_practical_word_signed_equality_foundation().unwrap();
    let bytes = f.certificate_bytes();
    inspect("word-signed-equality", bytes, true);
    let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert!(
        c.declarations.len() <= 4096 && c.term_table.len() <= 131_072 && bytes.len() <= 2_097_152
    );
    assert!(c.imports.is_empty());
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    println!(
        "signed equality: declarations={} terms={} bytes={} zero axioms",
        c.declarations.len(),
        c.term_table.len(),
        bytes.len()
    );
    let old = generate_go_practical_word_signed_order_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "0f61e8eb6a89cac3e2fa786f6027901424e2edaeb730df5dee0c5e8db49e8587"
    );
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 3457);
    let names = dependencies::declarations(&c);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(names.get(&name), Some(&value), "{name}");
    }
    for width in [8, 16, 32, 64] {
        for member in [
            "SourceEqualPreservesSign",
            "ValueEqualPreservesSign",
            "EqualInteger",
            "NotEqualInteger",
            "EquivalentReflect",
            "EquivalentComplete",
        ] {
            assert!(names.contains_key(&format!(
                "Mpk.Go.Practical.Word.SignedEquality.W{width}.{member}"
            )));
        }
    }
}
