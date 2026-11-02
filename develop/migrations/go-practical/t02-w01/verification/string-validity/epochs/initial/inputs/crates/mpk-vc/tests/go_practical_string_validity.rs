use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_foundation, generate_go_practical_string_validity_foundation,
    generate_go_practical_string_value, generate_go_practical_valid_string_value,
    import_go_practical_string_validity_foundation, import_go_practical_valid_string_value,
    GoStringValueError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_VALIDITY_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(format!("{name}.{extension}")))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    retain(name, "mpcert", bytes);
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    retain(name, "rust.json", format!("{}\n", result.json).as_bytes());
    if accepted && !result.accepted {
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lo, mut hi) = (1, c.declarations.len());
        while lo < hi {
            let end = (lo + hi) / 2;
            let mut prefix = c.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lo = end + 1;
            } else {
                hi = end;
            }
        }
        panic!(
            "{name}: first declaration type-check failure {}: {}",
            c.name_table[c.declarations[lo - 1].name as usize],
            result.json
        );
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if accepted {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(v["axiom_count"], 0);
        assert!(v["axiom_report"]["entries"].as_array().unwrap().is_empty());
    } else {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        let d: serde_json::Value =
            serde_json::from_str(v["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}
fn decode(bytes: &[u8]) -> Certificate {
    mpk_cert::decode_canonical_certificate(bytes).unwrap()
}

#[test]
fn generic_validity_preserves_every_original_string_identity() {
    let new = generate_go_practical_string_validity_foundation().unwrap();
    inspect("validity-foundation", new.certificate_bytes(), true);
    assert_eq!(
        new,
        import_go_practical_string_validity_foundation(new.certificate_bytes()).unwrap()
    );
    let c = decode(new.certificate_bytes());
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_foundation().unwrap();
    let golden=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-foundation/epochs/native-controls/selected/candidates/string-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    for (name, value) in dependencies::declarations(&decode(old.certificate_bytes())) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    println!(
        "string validity: declarations={} terms={} bytes={} all1053 old identities exact",
        c.declarations.len(),
        c.term_table.len(),
        new.certificate_bytes().len()
    );
}

#[test]
fn literal_empty_partial_and_full_capacity_have_actual_validity_witnesses() {
    for (name, bytes) in [
        ("empty", vec![]),
        ("partial", vec![0, 255, 128]),
        ("full", (0..=255u8).collect::<Vec<_>>()),
    ] {
        let value = generate_go_practical_valid_string_value(&bytes).unwrap();
        inspect(&format!("valid-{name}"), value.certificate_bytes(), true);
        assert_eq!(value.native_bytes(), bytes);
        let c = decode(value.certificate_bytes());
        let old = generate_go_practical_string_value(&bytes).unwrap();
        let all = dependencies::declarations(&c);
        for (name, term) in dependencies::declarations(&decode(old.certificate_bytes())) {
            assert_eq!(all.get(&name), Some(&term), "{name}");
        }
        let proof = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == "Mpk.Go.Practical.String.Value.Valid")
            .unwrap();
        assert!(matches!(proof.kind, DeclarationKind::Theorem { .. }));
        assert_eq!(
            value,
            import_go_practical_valid_string_value(value.certificate_bytes(), &bytes).unwrap()
        );
        println!(
            "{name} valid: length={} declarations={} terms={} bytes={}",
            bytes.len(),
            c.declarations.len(),
            c.term_table.len(),
            value.certificate_bytes().len()
        );
    }
    assert_eq!(
        generate_go_practical_valid_string_value(&[0; 257]),
        Err(GoStringValueError::Domain { bytes: 257 })
    );
}
