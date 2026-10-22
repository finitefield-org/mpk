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
use sha2::{Digest, Sha256};
fn check(name: &str, domain: &GoSourceValueDomain) {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_CASES_OUTPUT").map(PathBuf::from);
    let catalogue = generate_go_practical_source_error_case_catalogue(domain).unwrap();
    let prior = generate_go_practical_source_error_operations_foundation(domain).unwrap();
    assert_eq!(
        catalogue.prior_operations_certificate_bytes(),
        prior.certificate_bytes()
    );
    let epoch = if domain.error_cases().len() == 70 {
        "first-large"
    } else {
        "explicit-bit"
    };
    let old_name = if domain.error_cases().len() == 70 {
        "source-operations-N70"
    } else {
        name
    };
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-error-operations")
        .join(epoch)
        .join("certificates")
        .join(format!("{old_name}.mpcert"));
    assert_eq!(
        catalogue.prior_operations_certificate_bytes(),
        std::fs::read(&original).unwrap()
    );
    let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
    let before = dependencies::declarations(&old);
    assert_eq!(catalogue.case_tags(), prior.case_tags());
    let invalid = domain
        .types()
        .find(|t| !catalogue.case_tags().iter().any(|(id, _)| id == t.id()))
        .unwrap();
    assert_eq!(
        catalogue.generate_row(invalid.id()),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        catalogue.generate_row("not-an-original-case"),
        Err(GoOrdinaryError::Linkage)
    );
    if let Some(root) = &out {
        write(
            root,
            &format!("{name}.prior-operations.original-bytes"),
            catalogue.prior_operations_certificate_bytes(),
        );
        write(root,&format!("{name}.catalogue.json"),&serde_json::to_vec_pretty(&json!({"source_unit":name,"source_layout_module":catalogue.source_layout_module_name(),"boxing_module":catalogue.boxing_module_name(),"operations_module":catalogue.operations_module_name(),"case_tags":catalogue.case_tags(),"source_types":domain.types().collect::<Vec<_>>(),"original_complete_operations_stream":original,"prior_operations_bytes":prior.certificate_bytes().len(),"prior_operations_raw_sha256":format!("{:x}",Sha256::digest(prior.certificate_bytes())),"original_complete_operations_bytes_literal_equal":true})).unwrap());
    }
    for (index, (id, _)) in catalogue.case_tags().iter().enumerate() {
        let row = catalogue.generate_row(id).unwrap();
        let c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let after = dependencies::declarations(&c);
        for (name, identity) in &before {
            if name.starts_with(&(catalogue.source_layout_module_name().to_string() + ".")) {
                assert_eq!(
                    after.get(name),
                    Some(identity),
                    "every original source-specific definition/proof {name}"
                );
            }
        }
        for (name, identity) in &after {
            if let Some(original) = before.get(name) {
                assert_eq!(identity, original, "unchanged retained identity {name}");
            }
        }
        assert_eq!(row.case_tags(), catalogue.case_tags());
        assert_eq!(row.boxed_source_type_id(), id);
        assert!(c.imports.is_empty());
        assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
        assert_eq!(
            row,
            catalogue.import_row(row.certificate_bytes(), id).unwrap()
        );
        if catalogue.case_tags().len() > 1 {
            let other = &catalogue.case_tags()[(index + 1) % catalogue.case_tags().len()].0;
            assert_eq!(
                catalogue.import_row(row.certificate_bytes(), other),
                Err(GoOrdinaryError::Linkage)
            );
        }
        let file = format!("{name}-C{id}");
        if let Some(root) = &out {
            write(root, &format!("{file}.mpcert"), row.certificate_bytes());
            write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"source_unit":name,"module":row.module_name(),"source_layout_module":row.source_layout_module_name(),"boxing_module":row.boxing_module_name(),"operations_module":row.operations_module_name(),"boxed_source_type_id":row.boxed_source_type_id(),"case_tags":row.case_tags(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":c.declarations.len(),"terms":c.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_source_specific_named_identities_preserved":true})).unwrap());
        }
        let started = std::time::Instant::now();
        let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        println!(
            "{name} row {}/{}: {:?}",
            index + 1,
            catalogue.case_tags().len(),
            started.elapsed()
        );
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
#[test]
fn complete_case_rows_empty_single_grown_and_root_payloads() {
    for name in ["nil-only", "boxing-only", "entry"] {
        check(name, dispatch(name).domain());
    }
    check("all-root-shapes", &root_domain());
}
#[test]
fn all_seventy_case_rows_cover_all_4900_comparisons() {
    let selected = dispatch("seventy-cases");
    assert_eq!(selected.domain().error_cases().len(), 70);
    check("source-cases-N70", selected.domain());
}
