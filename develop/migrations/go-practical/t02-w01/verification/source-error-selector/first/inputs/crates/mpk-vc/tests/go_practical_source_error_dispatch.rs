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
    let roots = input["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(&keys, &r["ref"], "FuncDecl"))
        .collect::<Vec<_>>();
    let entries = if matches!(name, "nil-only" | "entry" | "seventy-cases") {
        vec![GoErrorEntryConstraint::new(
            roots[0].clone(),
            0,
            vec![],
            cases,
        )]
    } else {
        vec![]
    };
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
fn root_domain() -> GoSourceValueDomain {
    let c = corpus("source-domains");
    let keys = keys(&c);
    let roots = ["Plain", "WeightGood", "Depth16", "Empty", "Scalar"]
        .iter()
        .map(|n| locate(&keys, &c["roots"][n], "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = ["Code", "Flag", "Unit", "Ref"]
        .iter()
        .map(|n| {
            locate(
                &keys,
                &c["declarations"][format!("example.com/policy.{n}")],
                "TypeSpec",
            )
        })
        .collect();
    let selection = derive_go_source_selection(&keys, &roots).unwrap();
    let domain = derive_go_source_value_domain(
        &selection,
        &[GoErrorEntryConstraint::new(
            roots[0].clone(),
            0,
            vec![],
            cases,
        )],
    )
    .unwrap();
    assert_eq!(domain.error_cases().len(), 4);
    domain
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
            catalogue.generate_row(&foreign, None),
            Err(GoOrdinaryError::Linkage)
        );
    }
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_DISPATCH_OUTPUT").map(PathBuf::from);
    for (index, call) in calls.iter().enumerate() {
        for boxed in std::iter::once(None).chain(
            catalogue
                .case_tags()
                .iter()
                .map(|(id, _)| Some(id.as_str())),
        ) {
            let row = catalogue.generate_row(call, boxed).unwrap();
            let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
            let prior = boxed.map(|id| cases.generate_row(id).unwrap());
            if let Some(old) = &prior {
                let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
                let before = dependencies::declarations(&old);
                let after = dependencies::declarations(&decoded);
                for (n, v) in before {
                    assert_eq!(
                        after.get(&n),
                        Some(&v),
                        "every complete prior case-row identity {n}"
                    );
                }
            }
            assert!(decoded.imports.is_empty());
            assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
            assert_eq!(
                row,
                catalogue
                    .import_row(row.certificate_bytes(), call, boxed)
                    .unwrap()
            );
            if let Some(other) = catalogue
                .case_tags()
                .iter()
                .map(|(id, _)| Some(id.as_str()))
                .find(|v| *v != boxed)
            {
                assert_eq!(
                    catalogue.import_row(row.certificate_bytes(), call, other),
                    Err(GoOrdinaryError::Linkage)
                );
            }
            if calls.len() > 1 {
                assert_eq!(
                    catalogue.import_row(
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
                write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"source_layout_module":row.source_layout_module_name(),"boxing_module":row.boxing_module_name(),"operations_module":row.operations_module_name(),"case_row_module":row.case_row_module_name(),"boxed_source_type_id":row.boxed_source_type_id(),"call":call_json(row.call()),"all_calls":calls.iter().map(call_json).collect::<Vec<_>>(),"targets":row.targets().iter().map(target_json).collect::<Vec<_>>(),"case_tags":row.case_tags(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_case_row_identities_unchanged":true,"expected_selected_ordinal":boxed.and_then(|id|catalogue.case_tags().iter().position(|(t,_)|t==id)).map_or(0,|i|i+1)})).unwrap());
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
fn source_dispatch_empty_single_context_expression_and_multiple_sites() {
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
fn all_seventy_source_dispatch_targets_and_nil_are_selected_exactly() {
    check("seventy-cases");
}
#[test]
fn current_case_producer_replays_every_old_complete_row() {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_DISPATCH_OUTPUT").map(PathBuf::from);
    let mut values = vec![];
    let domains = [
        ("nil-only", dispatch("nil-only").domain().clone()),
        ("boxing-only", dispatch("boxing-only").domain().clone()),
        ("entry", dispatch("entry").domain().clone()),
        ("all-root-shapes", root_domain()),
        (
            "source-cases-N70",
            dispatch("seventy-cases").domain().clone(),
        ),
    ];
    for (name, domain) in domains {
        let cases = generate_go_practical_source_error_case_catalogue(&domain).unwrap();
        let epoch = if cases.case_tags().len() == 70 {
            "first-large"
        } else {
            "first"
        };
        for (id, _) in cases.case_tags() {
            let row = cases.generate_row(id).unwrap();
            let file = format!("{name}-C{id}.mpcert");
            let old = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(
                    "../../develop/migrations/go-practical/t02-w01/verification/source-error-cases",
                )
                .join(epoch)
                .join("certificates")
                .join(file);
            assert_eq!(row.certificate_bytes(), std::fs::read(&old).unwrap());
            values.push(json!({"source_unit":name,"source_type_id":id,"original":old,"bytes":row.certificate_bytes().len(),"complete_current_original_bytes_literal_equal":true}));
        }
    }
    assert_eq!(values.len(), 78);
    if let Some(out) = out {
        write(
            &out,
            "case-row-literal-map.json",
            &serde_json::to_vec_pretty(&values).unwrap(),
        );
    }
}
