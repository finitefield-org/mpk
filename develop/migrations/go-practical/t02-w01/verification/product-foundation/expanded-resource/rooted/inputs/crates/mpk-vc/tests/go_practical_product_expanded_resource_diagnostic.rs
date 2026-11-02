//! Complete original expanded inputs only; no over-cap or prefix proof authority.
#[path = "support/go_product_alias_codec.rs"]
mod controls;
#[path = "support/go_product_expanded_resource.rs"]
mod shapes;

fn inspect(name: &str, bytes: &[u8], _accepted: bool) {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    let out = std::path::PathBuf::from(
        std::env::var_os("MPK_GO_PRODUCT_RESOURCE_DIAGNOSTIC_OUTPUT").unwrap(),
    );
    std::fs::create_dir_all(&out).unwrap();
    let original = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../develop/migrations/go-practical/t02-w01/verification/product-foundation/final/tests/certificates")
    .join(format!("{name}.mpcert"));
    let certificate = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert!(bytes.len() <= 2_097_152 && certificate.declarations.len() <= 4096);
    let over_cap = certificate.term_table.len() > 131_072;
    let checked = if over_cap {
        assert!(!original.exists());
        let report = mpk_kernel::verify_certificate_bytes_json_output(bytes);
        assert!(!report.accepted);
        Some(report.json)
    } else {
        assert_eq!(std::fs::read(original).unwrap(), bytes);
        None
    };
    let record = serde_json::json!({
        "name":name,"raw_sha256":format!("{:x}",Sha256::digest(bytes)),
        "bytes":bytes.len(),"declarations":certificate.declarations.len(),
        "terms":certificate.term_table.len(),"axioms":certificate.axiom_report.summary.total_axiom_count,
        "over_frozen_term_cap":over_cap,"original_complete_bytes_equal":!over_cap,
        "scope":"Diagnostic complete original construction. In-cap bytes equal the entire original actual checked input; no repeated unchanged kernel check. Over-cap streams are rejected by the unchanged actual Rust production verifier, not counted as semantic negatives. No kernel cap/fuel/axiom/import alteration or prefix/source/program/context/application proof authority."
    });
    for (extension, raw) in [
        ("mpcert", bytes.to_vec()),
        (
            "diagnostic.json",
            format!("{}\n", serde_json::to_string_pretty(&record).unwrap()).into_bytes(),
        ),
    ] {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(format!("{name}.{extension}")))
            .unwrap();
        file.write_all(&raw).unwrap();
    }
    if let Some(report) = checked {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(format!("{name}.rust.json")))
            .unwrap();
        writeln!(file, "{report}").unwrap();
    }
}

#[test]
#[ignore = "Explicit complete original resource diagnosis; no over-cap or prefix proof authority."]
fn retain_complete_original_expanded_product_candidates() {
    shapes::all_balanced_shapes();
}
