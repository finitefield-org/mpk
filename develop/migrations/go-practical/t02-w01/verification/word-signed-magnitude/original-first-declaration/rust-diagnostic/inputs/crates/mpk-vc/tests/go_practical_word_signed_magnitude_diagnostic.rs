//! A temporary diagnostic harness over the original failed full certificate.
#[test]
fn identify_original_first_failed_declaration_without_changing_the_checker() {
    let path = std::env::var("MPK_GO_WORD_SIGNED_MAGNITUDE_DIAGNOSTIC_INPUT").unwrap_or_else(|_| "develop/migrations/go-practical/t02-w01/verification/word-signed-magnitude/owned-bounds/rust-construction/certificates/word-signed-magnitude-foundation.mpcert".into());
    let raw = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&path),
    )
    .unwrap();
    let original = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    let mut low=original.declarations.iter().position(|d|original.name_table[d.name as usize].starts_with("Mpk.Go.Practical.Word.SignedMagnitude.")).unwrap();
    let mut high=original.declarations.len();
    let mut rows=vec![];
    while low + 1 < high {
        let middle = (low + high) / 2;
        let mut prefix = original.clone();
        prefix.declarations.truncate(middle);
        let observation = mpk_kernel::check_declarations(&prefix);
        let accepted = observation.is_ok();
        let detail = observation.err().map(|e| e.detail().to_owned());
        println!(
            "prefix={middle} accepted={accepted} last={} error={detail:?}",
            original.name_table[original.declarations[middle - 1].name as usize]
        );
        rows.push(serde_json::json!({"diagnostic_declaration_prefix":middle,"checked":accepted,"detail":detail}));
        if accepted {
            low = middle;
        } else {
            high = middle;
        }
    }
    let name = &original.name_table[original.declarations[high - 1].name as usize];
    println!("first_failed_index={} name={name}", high - 1);
    let report = serde_json::json!({"input":path,"first_failed_declaration_index":high-1,"first_failed_declaration_name":name,"rows":rows,"scope":"Diagnostic direct declaration checking of truncated in-memory prefixes. Complete original bytes and rejection remain authoritative; these prefix observations are not complete certificate acceptances."});
    let path = std::env::var_os("MPK_GO_WORD_SIGNED_MAGNITUDE_DIAGNOSTIC_OUTPUT").unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
