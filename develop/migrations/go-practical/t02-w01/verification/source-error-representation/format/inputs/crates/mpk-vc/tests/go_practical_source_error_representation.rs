use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn unhex(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn keys(corpus: &Value) -> GoTypeKeyComponents {
    let input = &corpus["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(&s["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    import_go_type_key_components(&unhex(&input["components_hex"]), &sources).unwrap()
}
fn locate(keys: &GoTypeKeyComponents, row: &Value, kind: &str) -> GoSourceSelectionRef {
    let source = row["source"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == kind
                && n.start() == row["start"].as_u64().unwrap() as usize
                && n.end() == row["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn dispatch(name: &str) -> GoSourceErrorSelection {
    let c = corpus("error-dispatch");
    let keys = keys(&c);
    let row = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["input"]["name"] == name)
        .unwrap();
    let input = &row["input"];
    let cases = input["entry_cases"]
        .as_array()
        .map(|cs| {
            cs.iter()
                .map(|n| {
                    locate(
                        &keys,
                        &c["declarations"][format!("example.com/policy.{}", n.as_str().unwrap())],
                        "TypeSpec",
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut roots = vec![];
    let mut entries = vec![];
    for row in input["roots"].as_array().unwrap() {
        let r = locate(&keys, &row["ref"], "FuncDecl");
        let name = row["name"].as_str().unwrap();
        roots.push(r.clone());
        if matches!(
            name,
            "Entry" | "Expr" | "Paren" | "Guarded" | "Branch" | "Imported" | "OtherRoot"
        ) {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![],
                if name == "OtherRoot" {
                    vec![]
                } else {
                    cases.clone()
                },
            ));
        }
        if name == "Nested" {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![GoErrorEntryStep::Field(0)],
                cases.clone(),
            ));
            entries.push(GoErrorEntryConstraint::new(
                r,
                0,
                vec![GoErrorEntryStep::Field(1), GoErrorEntryStep::Elements],
                vec![],
            ));
        }
    }
    derive_go_source_error_selection(&keys, &roots, &entries).unwrap()
}
fn write(root: &Path, name: &str, raw: &[u8]) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(name))
        .unwrap()
        .write_all(raw)
        .unwrap();
}
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn call_json(c: &GoSourceErrorCall) -> Value {
    json!({"source":c.caller().source_object_id(),"caller_node":c.caller().node(),"call_node":c.call_node(),"receiver_node":c.receiver_node(),"method_expression":c.method_expression()})
}
fn target_json(t: &GoErrorImplementation) -> Value {
    let m = t.method();
    json!({"type_id":t.value_type_id(),"source":m.declaration_source_object_id(),"declaration_node":m.declaration_node(),"body_node":m.body_node(),"name":m.name(),"receiver_type_id":m.receiver_type_id(),"receiver_path":m.receiver_path(),"parameter_type_ids":m.parameter_type_ids(),"result_type_ids":m.result_type_ids()})
}
fn span(s: &GoSourceSelection, source: &str, node: usize) -> Value {
    let n = &s.source(source).unwrap().nodes()[node];
    json!({"source":source,"start":n.start(),"end":n.end()})
}
fn native_dispatch(s: &GoSourceErrorSelection, c: &GoSourceErrorCall) -> Value {
    let selected = s.selection();
    let source = c.caller().source_object_id();
    let p = selected.source(source).unwrap();
    let n = &p.nodes()[c.call_node()];
    let receiver = &p.nodes()[c.receiver_node()];
    let d = s.dispatches().iter().find(|d| d.call() == c).unwrap();
    json!({"caller":span(selected,source,c.caller().node()),"source":source,"start":n.start(),"end":n.end(),"receiver_start":receiver.start(),"receiver_end":receiver.end(),"method_expression":c.method_expression(),"targets":d.targets().iter().map(|t|{let m=t.method();json!({"type_id":t.value_type_id(),"callee":span(selected,m.declaration_source_object_id().unwrap(),m.declaration_node().unwrap())})}).collect::<Vec<_>>()})
}
fn check(name: &str) {
    let selection = dispatch(name);
    let domain = selection.domain();
    let catalogue = generate_go_practical_source_error_dispatch_catalogue(&selection).unwrap();
    let cases = generate_go_practical_source_error_case_catalogue(domain).unwrap();
    assert_eq!(catalogue.case_tags(), cases.case_tags());
    assert_eq!(catalogue.targets(), domain.error_cases());
    assert_eq!(
        catalogue.prior_operations_certificate_bytes(),
        cases.prior_operations_certificate_bytes()
    );
    let calls = selection
        .dispatches()
        .iter()
        .map(|d| d.call().clone())
        .collect::<Vec<_>>();
    assert_eq!(catalogue.calls(), calls);
    let foreign = dispatch("expression").dispatches()[0].call().clone();
    if !calls.contains(&foreign) {
        assert_eq!(
            catalogue.generate_representation_row(&foreign, None),
            Err(GoOrdinaryError::Linkage)
        );
    }
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_REPRESENTATION_OUTPUT").map(PathBuf::from);
    let original_corpus = corpus("error-dispatch");
    let expected = &original_corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["input"]["name"] == name)
        .unwrap()["native"]["dispatches"];
    assert_eq!(calls.len(), expected.as_array().unwrap().len());
    for (index, call) in calls.iter().enumerate() {
        let original_dispatch = native_dispatch(&selection, call);
        assert!(expected.as_array().unwrap().contains(&original_dispatch));
        for boxed in std::iter::once(None).chain(
            catalogue
                .case_tags()
                .iter()
                .map(|(id, _)| Some(id.as_str())),
        ) {
            let row = catalogue.generate_representation_row(call, boxed).unwrap();
            let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
            let previous = row.prior_receiver_row();
            let old = previous.prior_dispatch_row();
            let before = mpk_cert::decode_canonical_certificate(previous.certificate_bytes()).unwrap();
            let before = dependencies::declarations(&before);
            let after = dependencies::declarations(&decoded);
            for (n, v) in before {
                assert_eq!(
                    after.get(&n),
                    Some(&v),
                    "every prior full selector identity {n}"
                );
            }
            let original_name = format!("{name}-call{index}-{}.mpcert", boxed.unwrap_or("nil"));
            let epoch = if name == "seventy-cases" {
                "first-large"
            } else {
                "entries"
            };
            let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-receiver").join(epoch).join("certificates").join(original_name);
            assert_eq!(
                previous.certificate_bytes(),
                std::fs::read(&original).unwrap(),
                "full original selector bytes literal"
            );
            assert!(decoded.imports.is_empty());
            assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
            assert_eq!(
                row,
                catalogue
                    .import_representation_row(row.certificate_bytes(), call, boxed)
                    .unwrap()
            );
            if let Some(other) = catalogue
                .case_tags()
                .iter()
                .map(|(id, _)| Some(id.as_str()))
                .find(|v| *v != boxed)
            {
                assert_eq!(
                    catalogue.import_representation_row(row.certificate_bytes(), call, other),
                    Err(GoOrdinaryError::Linkage)
                );
            }
            if calls.len() > 1 {
                assert_eq!(
                    catalogue.import_representation_row(
                        row.certificate_bytes(),
                        &calls[(index + 1) % calls.len()],
                        boxed
                    ),
                    Err(GoOrdinaryError::Linkage)
                );
            }
            let file = format!("{name}-call{index}-{}", boxed.unwrap_or("nil"));
            if let Some(root) = &out {
                write(root, &format!("{file}.mpcert"), row.certificate_bytes());
                write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"receiver_module":previous.module_name(),"selector_module":old.module_name(),"source_layout_module":old.source_layout_module_name(),"boxing_module":old.boxing_module_name(),"operations_module":old.operations_module_name(),"case_row_module":old.case_row_module_name(),"boxed_source_type_id":old.boxed_source_type_id(),"call":call_json(old.call()),"all_calls":calls.iter().map(call_json).collect::<Vec<_>>(),"original_dispatch":original_dispatch,"targets":old.targets().iter().map(target_json).collect::<Vec<_>>(),"case_tags":old.case_tags(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_receiver_identities_unchanged":true,"prior_receiver_original":original,"prior_receiver_complete_bytes_literal_equal":true,"expected_selected_ordinal":boxed.and_then(|id|catalogue.case_tags().iter().position(|(t,_)|t==id)).map_or(0,|i|i+1)})).unwrap());
            }
            let start = std::time::Instant::now();
            let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
            println!("{file}: {:?}", start.elapsed());
            if let Some(root) = &out {
                write(
                    root,
                    &format!("{file}.rust.json"),
                    format!("{}\n", result.json).as_bytes(),
                );
            }
            assert!(result.accepted, "{file}: {}", result.json);
        }
    }
    if let Some(root) = &out {
        write(root,&format!("{name}.catalogue.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"cases":catalogue.case_tags().len(),"sites":calls.len(),"proof_rows":calls.len()*(catalogue.case_tags().len()+1),"calls":calls.iter().map(call_json).collect::<Vec<_>>(),"targets":catalogue.targets().iter().map(target_json).collect::<Vec<_>>(),"empty_target_table_does_not_prove_nil_safety":true})).unwrap());
    }
}
#[test]
fn source_representation_empty_expression_sites_with_all_inactive_zeros() {
    for name in [
        "nil-only",
        "entry",
        "expression",
        "receiver-call",
        "multiple-roots",
        "boxing-only",
    ] {
        check(name);
    }
}
#[test]
fn all_seventy_representations_keep_complete_inactive_payloads_and_copy() {
    check("seventy-cases");
}
#[test]
fn current_receiver_producer_replays_every_old_complete_row() {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_REPRESENTATION_OUTPUT").map(PathBuf::from);
    let mut values = vec![];
    for name in [
        "nil-only",
        "entry",
        "expression",
        "receiver-call",
        "multiple-roots",
        "boxing-only",
        "seventy-cases",
    ] {
        let selection = dispatch(name);
        let catalogue = generate_go_practical_source_error_dispatch_catalogue(&selection).unwrap();
        for (i, call) in catalogue.calls().iter().enumerate() {
            for boxed in std::iter::once(None).chain(
                catalogue
                    .case_tags()
                    .iter()
                    .map(|(id, _)| Some(id.as_str())),
            ) {
                let row = catalogue.generate_receiver_row(call, boxed).unwrap();
                let epoch = if name == "seventy-cases" {
                    "first-large"
                } else {
                    "entries"
                };
                let filename = format!("{name}-call{i}-{}.mpcert", boxed.unwrap_or("nil"));
                let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-receiver").join(epoch).join("certificates").join(filename);
                assert_eq!(row.certificate_bytes(), std::fs::read(&original).unwrap());
                values.push(json!({"unit":name,"call_index":i,"boxed_type":boxed,"original":original,"bytes":row.certificate_bytes().len(),"complete_current_original_bytes_literal_equal":true}));
            }
        }
    }
    assert_eq!(values.len(), 92);
    if let Some(out) = out {
        write(
            &out,
            "receiver-row-literal-map.json",
            &serde_json::to_vec_pretty(&values).unwrap(),
        );
    }
}
