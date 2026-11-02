use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_order_foundation, generate_go_practical_word_shift_foundation,
    generate_go_practical_word_signed_order_foundation,
};
use sha2::{Digest, Sha256};

#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let report = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(directory) = std::env::var_os("MPK_GO_WORD_SIGNED_ORDER_TEST_OUTPUT") {
        use std::io::Write;
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", report.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(report.accepted, accepted, "{name}: {}", report.json);
    if !accepted {
        let value: serde_json::Value = serde_json::from_str(&report.json).unwrap();
        assert_eq!(value["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(value["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}

#[test]
fn actual_signed_order_component_and_complete_old_identities() {
    let foundation = generate_go_practical_word_signed_order_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    inspect("word-signed-order", bytes, true);
    let certificate = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert!(certificate.declarations.len() <= 4096);
    assert!(certificate.term_table.len() <= 131_072);
    assert!(bytes.len() <= 2_097_152);
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert!(certificate.imports.is_empty());
    println!(
        "signed order: declarations={} terms={} bytes={} zero axioms",
        certificate.declarations.len(),
        certificate.term_table.len(),
        bytes.len()
    );
    let old_order = generate_go_practical_word_order_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old_order.certificate_bytes())),
        "c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa"
    );
    let old = mpk_cert::decode_canonical_certificate(old_order.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 3437);
    let retained = dependencies::declarations(&certificate);
    for (name, original) in dependencies::declarations(&old) {
        assert_eq!(retained.get(&name), Some(&original), "{name}");
    }
    let old_shifts = generate_go_practical_word_shift_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(old_shifts.certificate_bytes())),
        "119c384ddefd5afc6c42de2171df608d4b5b6329304912d3e9ce29860751e1b2"
    );
    for width in [8, 16, 32, 64] {
        for member in ["SignedLessInteger", "SignedLessEqualInteger"] {
            assert!(retained.contains_key(&format!(
                "Mpk.Go.Practical.Word.SignedOrder.W{width}.{member}"
            )));
        }
    }
}
