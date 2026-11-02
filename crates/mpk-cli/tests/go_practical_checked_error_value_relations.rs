#[path = "../src/go_practical_checked_error_value_relations.rs"]
#[allow(dead_code)]
mod checked;

use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn error_corpus(name: &str) -> Value {
    if name == "heterogeneous-payloads" {
        let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-representation/corpus-heterogeneous.json");
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    } else {
        corpus("error-dispatch")
    }
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
    let c = error_corpus(name);
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

fn check_original(name: &str, cases: usize) {
    let selected = dispatch(name);
    let domain = selected.domain();
    assert_eq!(domain.error_cases().len(), cases);
    let catalogue = generate_go_practical_source_value_relation_catalogue(domain).unwrap();
    let rows = checked::check_go_error_value_catalogue(&catalogue, domain).unwrap();
    assert_eq!(rows.receipts().len(), cases + 1);
    assert_eq!(rows.receipts()[0].boxed_source_type_id(), None);
    for (receipt, (type_id, _)) in rows.receipts()[1..].iter().zip(catalogue.case_tags()) {
        assert_eq!(receipt.boxed_source_type_id(), Some(type_id.as_str()));
    }
    let mut indices = vec![0];
    if cases != 0 {
        indices.push(cases);
    }
    for index in indices {
        let row = rows.reconstruct(&catalogue, index).unwrap();
        let receipt = &rows.receipts()[index];
        assert_eq!(row.module_name(), receipt.module());
        assert_eq!(
            mpk_cert::certificate_hash(row.certificate_bytes()),
            *receipt.certificate_hash()
        );
        if let Some(out) = std::env::var_os("MPK_GO_CHECKED_ERROR_VALUE_OUTPUT") {
            let out = PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            let label = if index == 0 { "nil" } else { "last" };
            std::fs::write(
                out.join(format!("{name}-{label}.mpcert")),
                row.certificate_bytes(),
            )
            .unwrap();
            std::fs::write(out.join(format!("{name}-{label}.rust.json")), serde_json::to_vec(&json!({
                "verdict":"accepted", "module":receipt.module(), "declaration_count":receipt.declarations(),
                "axiom_count":0, "hashes":{"export":mpk_cert::hash_hex(receipt.export_hash()),
                "axiom_report":mpk_cert::hash_hex(receipt.axiom_report_hash()),
                "certificate":mpk_cert::hash_hex(receipt.certificate_hash())}
            })).unwrap()).unwrap();
        }
    }
    assert!(matches!(
        rows.reconstruct(&catalogue, cases + 1),
        Err(checked::GoCheckedErrorValueRelationError::Coverage { .. })
    ));
    eprintln!(
        "{name}: checked {} complete original rows",
        rows.receipts().len()
    );
}

#[test]
fn checks_original_nil_and_small_error_domains() {
    check_original("nil-only", 0);
    check_original("entry", 3);
    let first = dispatch("nil-only");
    let other = dispatch("entry");
    let catalogue = generate_go_practical_source_value_relation_catalogue(first.domain()).unwrap();
    assert!(matches!(
        checked::check_go_error_value_catalogue(&catalogue, other.domain()),
        Err(checked::GoCheckedErrorValueRelationError::Coverage { .. })
    ));
}

#[test]
fn checks_all_seventy_original_error_cases_and_nil() {
    check_original("seventy-cases", 70);
}
