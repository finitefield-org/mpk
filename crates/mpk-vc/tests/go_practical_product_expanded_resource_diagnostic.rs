//! Complete original expanded inputs; no over-cap, kernel or prefix proof authority.
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
    // Own complete candidate bytes before any diagnostic interpretation.
    let mut original_bytes = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join(format!("{name}.mpcert")))
        .unwrap();
    original_bytes.write_all(bytes).unwrap();
    let original = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/product-foundation/final/tests/certificates")
        .join(format!("{name}.mpcert"));
    let certificate = mpk_cert::decode_canonical_certificate(bytes).unwrap();
    assert!(bytes.len() <= 2_097_152 && certificate.declarations.len() <= 4096);
    let over_cap = certificate.term_table.len() > 131_072;
    if over_cap {
        assert!(!original.exists());
    } else {
        assert_eq!(std::fs::read(original).unwrap(), bytes);
    }
    let record = serde_json::json!({
        "name":name,"raw_sha256":format!("{:x}",Sha256::digest(bytes)),
        "bytes":bytes.len(),"declarations":certificate.declarations.len(),
        "terms":certificate.term_table.len(),
        "declared_axiom_count":certificate.axiom_report.summary.total_axiom_count,
        "outside_frozen_Go_producer_term_budget":over_cap,
        "original_complete_checked_bytes_equal":!over_cap,
        "kernel_executed":false,
        "scope":"Diagnostic complete original construction only. In-budget bytes equal the entire original actual checked input; no repeated unchanged kernel check. Out-of-budget streams stay excluded from Go producer admission. The original generator guard and full failed execution remain; raw construction does not widen its cap. No kernel reject/accept verdict, semantic negative, checked-axiom, prefix/source/program/context/application proof authority is claimed."
    });
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join(format!("{name}.diagnostic.json")))
        .unwrap();
    writeln!(file, "{}", serde_json::to_string_pretty(&record).unwrap()).unwrap();
}

#[test]
#[ignore = "Explicit complete original resource construction; no kernel or over-cap proof authority."]
fn retain_complete_original_expanded_product_candidates() {
    shapes::all_balanced_shapes();
}
