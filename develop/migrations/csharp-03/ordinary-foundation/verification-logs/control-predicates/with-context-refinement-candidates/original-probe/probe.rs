
#[test]
fn csharp_03_t06_w09_refinement_generation_original_probe() {
    std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        let mut visited = 0;
        each_context(|id, vir| {
            if id != "is_binding" { return; }
            visited += 1;
            let p = generate_csharp_practical_ordinary_control_predicates_with_packed_pattern_proof_types(vir).unwrap();
            let required = csharp_practical_ordinary_pattern_refinement_candidate_theorems(&p).unwrap();
            let outcome = generate_csharp_practical_ordinary_pattern_refinement_candidate(&p);
            let report = match outcome {
                Ok(candidate) => {
                    let checked = mpk_kernel::verify_certificate_bytes(candidate.certificate_bytes()).unwrap();
                    serde_json::json!({"status":"generated_and_rust_checked", "context":id, "required_theorems":required.len(), "axiom_count":checked.axiom_count, "proof_check_pending":candidate.proof_check_pending(), "application_scope_pending":true})
                },
                Err(error) => serde_json::json!({"status":"original_proofs_pending", "context":id, "required_theorems":required.len(), "error":format!("{error:?}"), "application_scope_pending":true}),
            };
            std::fs::write("/tmp/mpk-w09-context-refinement/original-probe/outcome.json", serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("Original source capability: {report}");
        });
        assert_eq!(visited,1);
    }).unwrap().join().unwrap();
}
