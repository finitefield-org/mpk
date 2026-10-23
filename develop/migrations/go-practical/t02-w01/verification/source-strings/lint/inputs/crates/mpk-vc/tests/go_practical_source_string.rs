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
fn error_corpus(name: &str) -> Value {
    if name == "source-strings" {
        corpus("source-strings")
    } else if name == "heterogeneous-payloads" {
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

fn check(name: &str) {
    let selection = dispatch(name);
    let domain = selection.domain();
    let catalogue = generate_go_practical_source_string_catalogue(domain).unwrap();
    assert_eq!(
        catalogue.source_types(),
        domain.types().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        catalogue.case_type_ids(),
        domain
            .error_cases()
            .iter()
            .map(|c| c.value_type_id().to_string())
            .collect::<Vec<_>>()
    );
    if name == "source-strings" {
        assert_eq!(catalogue.source_string_type_ids().len(), 4);
    }
    let prior =
        mpk_cert::decode_canonical_certificate(catalogue.prior_layout_certificate_bytes()).unwrap();
    let generic =
        mpk_cert::decode_canonical_certificate(catalogue.generic_string_certificate_bytes())
            .unwrap();
    let before = dependencies::declarations(&prior);
    let generic_before = dependencies::declarations(&generic);
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_OUTPUT").map(PathBuf::from);
    assert_eq!(
        catalogue.generate_row("not-a-source-string").unwrap_err(),
        GoOrdinaryError::Linkage
    );
    for id in catalogue.source_string_type_ids() {
        let row = catalogue.generate_row(id).unwrap();
        let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let after = dependencies::declarations(&decoded);
        for (n, v) in before
            .iter()
            .filter(|(n, _)| n.starts_with(&(row.source_layout_module_name().to_string() + ".")))
        {
            assert_eq!(
                after.get(n),
                Some(v),
                "complete original source type/zero/proof: {n}"
            );
        }
        for (n, v) in generic_before
            .iter()
            .filter(|(n, _)| n.starts_with("Mpk.Go.Practical.String."))
        {
            assert_eq!(
                after.get(n),
                Some(v),
                "unchanged checked generic string identity: {n}"
            );
        }
        assert!(decoded.imports.is_empty());
        assert!(decoded.theory_certificates.is_empty());
        assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
        assert_eq!(
            catalogue.import_row(row.certificate_bytes(), id).unwrap(),
            row
        );
        assert_eq!(
            catalogue.import_row(row.certificate_bytes(), "not-a-source-string"),
            Err(GoOrdinaryError::Linkage)
        );
        if let Some(other) = catalogue.source_string_type_ids().iter().find(|s| *s != id) {
            assert_eq!(
                catalogue.import_row(row.certificate_bytes(), other),
                Err(GoOrdinaryError::Linkage)
            );
        }
        let file = format!("{name}-{id}");
        if let Some(root) = &out {
            write(root, &format!("{file}.mpcert"), row.certificate_bytes());
            write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"source_layout_module":row.source_layout_module_name(),"source_string_type_id":id,"cases":catalogue.case_type_ids(),"types":catalogue.source_types(),"underlying":catalogue.source_types().iter().filter_map(|t|catalogue.underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"all_original_source_and_generic_string_identities_unchanged":true,"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"source_expression_or_context_authority":false})).unwrap());
        }
        let start = std::time::Instant::now();
        let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        println!("{file}: {:?} {}", start.elapsed(), result.verdict);
        if let Some(root) = &out {
            write(
                root,
                &format!("{file}.rust.json"),
                &serde_json::to_vec_pretty(&result).unwrap(),
            );
        }
        assert_eq!(result.verdict, "accepted", "{result:?}");
        assert_eq!(result.axiom_count, Some(0));
    }
}
#[test]
fn source_strings() {
    for name in [
        "nil-only",
        "entry",
        "heterogeneous-payloads",
        "seventy-cases",
        "source-strings",
    ] {
        check(name);
    }
}
