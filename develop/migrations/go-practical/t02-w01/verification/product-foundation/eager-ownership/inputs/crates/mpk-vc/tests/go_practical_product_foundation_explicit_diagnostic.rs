//! Direct diagnosis only; complete original checker verdicts remain authoritative.
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

#[test]
#[ignore = "Explicit full original failed-candidate prefix diagnostic; no prefix proof authority."]
fn identify_first_explicit_product_type_failure() {
    let raw = include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/product-foundation/explicit-function-types/construction/certificates/product-foundation.mpcert");
    let old_raw = include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/core-interfaces/final/rust-tests/certificates/core-foundation.mpcert");
    let original = mpk_cert::decode_canonical_certificate(raw).unwrap();
    let old = mpk_cert::decode_canonical_certificate(old_raw).unwrap();
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(raw)),
        "082fb8eeba6cb22fa208a3670a009f33e23952eee7f64d5184e1169d55677870"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(old_raw)),
        "5b3b9c6e3eafa8f3c5652967cbad1f6d7eaa786d1ca23da894884a8a1b7ff373"
    );
    assert_eq!(old.declarations.len(), 35);
    let actual = dependencies::declarations(&original);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(actual.get(&name), Some(&value), "{name}");
    }
    for (index, declaration) in old.declarations.iter().enumerate() {
        assert_eq!(
            old.name_table[declaration.name as usize],
            original.name_table[original.declarations[index].name as usize]
        );
    }
    let (mut low, mut high) = (35, original.declarations.len());
    let mut rows = vec![];
    while low + 1 < high {
        let mid = (low + high) / 2;
        let mut prefix = original.clone();
        prefix.declarations.truncate(mid);
        let checked = mpk_kernel::check_declarations(&prefix);
        let ok = checked.is_ok();
        let detail = checked.err().map(|e| e.detail().to_owned());
        println!("prefix={mid} checked={ok} detail={detail:?}");
        rows.push(serde_json::json!({"prefix":mid,"checked":ok,"detail":detail}));
        if ok {
            low = mid
        } else {
            high = mid
        }
    }
    let name = &original.name_table[original.declarations[high - 1].name as usize];
    println!("first_failed_index={} name={name}", high - 1);
    let record = serde_json::json!({"original_raw_sha256":format!("{:x}", Sha256::digest(raw)),"known_original_prefix":35,"first_failed_declaration_index":high-1,"first_failed_declaration_name":name,"rows":rows,"scope":"Direct in-memory declaration prefix observations only. Full original bytes are checked separately in both unchanged cores; no encoded-prefix acceptance, semantic negative or source/program/profile authority is claimed."});
    let mut bytes = serde_json::to_vec_pretty(&record).unwrap();
    bytes.push(b'\n');
    std::fs::write(
        std::env::var_os("MPK_GO_PRODUCT_FOUNDATION_DIAGNOSTIC_OUTPUT").unwrap(),
        bytes,
    )
    .unwrap();
}
