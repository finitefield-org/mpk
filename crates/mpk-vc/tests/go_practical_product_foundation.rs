use mpk_vc::go_practical_ordinary::{
    generate_go_practical_core_foundation, generate_go_practical_product_foundation,
    import_go_practical_product_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_product_foundation_controls.rs"]
mod controls;
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(out) = std::env::var_os("MPK_GO_PRODUCT_FOUNDATION_TEST_OUTPUT") {
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
fn actual_heterogeneous_product_component_and_complete_old_identities() {
    let f = generate_go_practical_product_foundation().unwrap();
    let bytes = f.certificate_bytes();
    inspect("product-foundation", bytes, true);
    let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert!(
        c.declarations.len() <= 4096 && c.term_table.len() <= 131_072 && bytes.len() <= 2_097_152
    );
    assert!(c.imports.is_empty());
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    println!(
        "products: declarations={} terms={} bytes={} zero axioms",
        c.declarations.len(),
        c.term_table.len(),
        bytes.len()
    );
    let old = generate_go_practical_core_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old.certificate_bytes())),
        "5b3b9c6e3eafa8f3c5652967cbad1f6d7eaa786d1ca23da894884a8a1b7ff373"
    );
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 35);
    let all = dependencies::declarations(&c);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
}

#[test]
fn exact_component_ownership_is_separate_from_core_acceptance() {
    let component = generate_go_practical_product_foundation().unwrap();
    assert_eq!(
        component,
        import_go_practical_product_foundation(component.certificate_bytes()).unwrap()
    );
    assert_eq!(
        component,
        generate_go_practical_product_foundation().unwrap()
    );
    let different = controls::honest_different_component();
    inspect("honest-different-product", &different, true);
    assert_eq!(
        import_go_practical_product_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = component.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_product_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_product_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn products_reject_changed_operations_claims_and_validity_premises() {
    controls::semantic_controls();
}
