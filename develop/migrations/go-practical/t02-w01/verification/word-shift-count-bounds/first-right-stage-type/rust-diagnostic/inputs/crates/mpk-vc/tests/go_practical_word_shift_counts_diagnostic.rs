//! Direct suffix diagnosis; the original full core verdict remains authoritative.
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[test]
#[ignore = "Explicit one-off full original failed-candidate prefix diagnostic; no prefix proof authority."]
fn identify_first_new_right_stage_failure_preserving_the_full_original() {
    let p = std::env::var("MPK_GO_WORD_SHIFT_COUNTS_DIAGNOSTIC_INPUT").unwrap();
    let raw = std::fs::read(&p).unwrap();
    let original = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    let old_raw=include_bytes!("../../../develop/migrations/go-practical/t02-w01/verification/word-shift-count-bounds/actual-variable-left/rust-construction/certificates/word-shift-counts.mpcert");
    let old = mpk_cert::decode_canonical_certificate(old_raw).unwrap();
    let a = dependencies::declarations(&old);
    let b = dependencies::declarations(&original);
    assert_eq!(old.declarations.len(), 3352);
    for (n, v) in a {
        assert_eq!(b.get(&n), Some(&v), "retained prefix changed: {n}");
    }
    for (i, d) in old.declarations.iter().enumerate() {
        assert_eq!(
            old.name_table[d.name as usize],
            original.name_table[original.declarations[i].name as usize]
        );
    }
    let (mut low, mut high) = (old.declarations.len(), original.declarations.len());
    let mut rows = vec![];
    while low + 1 < high {
        let mid = (low + high) / 2;
        let mut prefix = original.clone();
        prefix.declarations.truncate(mid);
        let checked = mpk_kernel::check_declarations(&prefix);
        let ok = checked.is_ok();
        let detail = checked.err().map(|e| e.detail().to_owned());
        println!(
            "prefix={mid} checked={ok} last={} detail={detail:?}",
            original.name_table[original.declarations[mid - 1].name as usize]
        );
        rows.push(serde_json::json!({"prefix":mid,"checked":ok,"detail":detail}));
        if ok {
            low = mid
        } else {
            high = mid
        }
    }
    let name = &original.name_table[original.declarations[high - 1].name as usize];
    println!("first_failed_index={} name={name}", high - 1);
    let record = serde_json::json!({"original_input":p,"known_exact_original_prefix":3352,"first_failed_declaration_index":high-1,"first_failed_declaration_name":name,"rows":rows,"scope":"In-memory direct declaration prefix observations only; no encoded-prefix acceptance or semantic negative counted. Both unchanged cores check complete original bytes separately. No checker/fuel/bounds changes."});
    let out = std::env::var_os("MPK_GO_WORD_SHIFT_COUNTS_DIAGNOSTIC_OUTPUT").unwrap();
    let mut bytes = serde_json::to_vec_pretty(&record).unwrap();
    bytes.push(b'\n');
    std::fs::write(out, bytes).unwrap();
}
