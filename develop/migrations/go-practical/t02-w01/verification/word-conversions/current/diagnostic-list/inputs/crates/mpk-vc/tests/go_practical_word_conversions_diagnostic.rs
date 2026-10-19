//! Direct prefix diagnostics; only the complete original bytes give a verdict.
#[test]
#[ignore = "One-off original failed-certificate prefix diagnostic; explicitly select with --ignored."]
fn identify_original_first_failed_declaration_without_changing_the_checker() {
    let path = std::env::var("MPK_GO_WORD_CONVERSIONS_DIAGNOSTIC_INPUT").unwrap();
    let raw = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&path),
    )
    .unwrap();
    let original = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    let mut low = original
        .declarations
        .iter()
        .position(|d| {
            original.name_table[d.name as usize] == "Mpk.Go.Practical.Word.Conversions.Fill0"
        })
        .unwrap();
    let mut high = original.declarations.len();
    let mut rows = vec![];
    while low + 1 < high {
        let middle = (low + high) / 2;
        let mut prefix = original.clone();
        prefix.declarations.truncate(middle);
        let observation = mpk_kernel::check_declarations(&prefix);
        let accepted = observation.is_ok();
        let detail = observation.err().map(|e| e.detail().to_owned());
        println!(
            "prefix={middle} checked={accepted} last={} error={detail:?}",
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
    let report = serde_json::json!({"input":path,"first_failed_declaration_index":high-1,"first_failed_declaration_name":name,"rows":rows,"scope":"Diagnostic direct declaration checking over in-memory prefixes only. The complete original certificate/source/raw core rejection stays authoritative. No complete prefix acceptance, semantic negative or resource acceptance is claimed."});
    let out = std::env::var_os("MPK_GO_WORD_CONVERSIONS_DIAGNOSTIC_OUTPUT").unwrap();
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(out, bytes).unwrap();
}
