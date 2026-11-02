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

fn check(name: &str, domain: &GoSourceValueDomain) {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_BOXINGS_OUTPUT").map(PathBuf::from);
    let generated = generate_go_practical_source_error_boxing_foundation(domain);
    if let (Some(root), Err(error)) = (&out, &generated) {
        write(root,&format!("{name}.generation-error.json"),&serde_json::to_vec(&json!({"error":format!("{error:?}"),"cases":domain.error_cases().len(),"types":domain.types().count()})).unwrap());
    }
    let artifact = generated.unwrap();
    let certificate = mpk_cert::decode_canonical_certificate(artifact.certificate_bytes()).unwrap();
    let prior = generate_go_practical_source_error_layout_foundation(domain).unwrap();
    let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
    let before = dependencies::declarations(&old);
    let after = dependencies::declarations(&certificate);
    for (name, identity) in before {
        assert_eq!(after.get(&name), Some(&identity), "prior identity {name}");
    }
    assert_eq!(
        artifact
            .case_tags()
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        domain
            .error_cases()
            .iter()
            .map(|c| c.value_type_id())
            .collect::<Vec<_>>()
    );
    for (i, (_, tag)) in artifact.case_tags().iter().enumerate() {
        assert_eq!(*tag, i as u64 + 1);
    }
    assert!(certificate.imports.is_empty());
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert_eq!(certificate.module, artifact.module_name());
    assert_eq!(
        artifact,
        import_go_practical_source_error_boxing_foundation(artifact.certificate_bytes(), domain)
            .unwrap()
    );
    if let Some(root) = &out {
        write(
            root,
            &format!("{name}.mpcert"),
            artifact.certificate_bytes(),
        );
        write(root,&format!("{name}.sources.json"),&serde_json::to_vec_pretty(&json!({"module":artifact.module_name(),"source_layout_module":artifact.source_layout_module_name(),"case_tags":artifact.case_tags(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"prior_source_module":prior.module_name(),"prior_source_declarations":old.declarations.len(),"declarations":certificate.declarations.len(),"terms":certificate.term_table.len(),"bytes":artifact.certificate_bytes().len()})).unwrap());
    }
    let started = std::time::Instant::now();
    let result = mpk_kernel::verify_certificate_bytes_json_output(artifact.certificate_bytes());
    println!("{name}: {:?}", started.elapsed());
    if let Some(root) = out {
        write(
            &root,
            &format!("{name}.rust.json"),
            format!("{}\n", result.json).as_bytes(),
        );
    }
    assert!(result.accepted, "{name}: {}", result.json);
}
#[test]
fn complete_source_boxing_empty_single_grown_and_root_payloads() {
    for name in ["nil-only", "boxing-only", "entry"] {
        check(name, dispatch(name).domain());
    }
    check("all-root-shapes", &root_domain());
}
#[test]
fn all_seventy_source_cases_fit_and_preserve_every_payload() {
    let unit = dispatch("seventy-cases");
    assert_eq!(unit.domain().error_cases().len(), 70);
    check("source-boxing-N70", unit.domain());
}
