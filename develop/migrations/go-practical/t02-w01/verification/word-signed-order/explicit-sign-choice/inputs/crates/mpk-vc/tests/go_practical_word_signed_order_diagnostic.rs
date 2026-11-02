//! Direct diagnosis only; complete original checker verdicts remain authoritative.
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

#[test]
#[ignore = "Explicit full original failed-candidate prefix diagnostic; no prefix proof authority."]
fn identify_first_signed_order_type_failure() {
    let raw = include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/word-signed-order/initial/construction/certificates/word-signed-order.mpcert");
    let old_raw = include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/word-order/corrected/rust-tests/certificates/word-order-foundation.mpcert");
    let original = mpk_cert::decode_canonical_certificate(raw).unwrap();
    let old = mpk_cert::decode_canonical_certificate(old_raw).unwrap();
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(raw)),
        "d33c2efb0fa90673aa114831d3671d11ecb1d3d3ba4bcb2415d35ca765859fbf"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(old_raw)),
        "c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa"
    );
    assert_eq!(old.declarations.len(), 3437);
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
    let (mut low, mut high) = (3437, original.declarations.len());
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
    let record = serde_json::json!({"original_raw_sha256":format!("{:x}", Sha256::digest(raw)),"known_original_prefix":3437,"first_failed_declaration_index":high-1,"first_failed_declaration_name":name,"rows":rows,"scope":"Direct in-memory declaration prefix observations only. Full original bytes are checked separately in both unchanged cores; no encoded-prefix acceptance, semantic negative or source/program/profile authority is claimed."});
    let mut bytes = serde_json::to_vec_pretty(&record).unwrap();
    bytes.push(b'\n');
    std::fs::write(
        std::env::var_os("MPK_GO_WORD_SIGNED_ORDER_DIAGNOSTIC_OUTPUT").unwrap(),
        bytes,
    )
    .unwrap();
}
