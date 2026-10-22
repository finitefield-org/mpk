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
fn check(name: &str, domain: &GoSourceValueDomain) {
    let artifact = generate_go_practical_source_error_layout_foundation(domain).unwrap();
    assert_eq!(
        artifact.case_type_ids(),
        domain
            .error_cases()
            .iter()
            .map(|c| c.value_type_id().to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        artifact,
        import_go_practical_source_error_layout_foundation(artifact.certificate_bytes(), domain)
            .unwrap()
    );
    let certificate = mpk_cert::decode_canonical_certificate(artifact.certificate_bytes()).unwrap();
    assert!(certificate.imports.is_empty());
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert_eq!(certificate.module, artifact.module_name());
    let prefix = format!("{}.T", artifact.module_name());
    assert_eq!(
        certificate
            .declarations
            .iter()
            .filter(|d| certificate.name_table[d.name as usize].starts_with(&prefix))
            .count(),
        3 * domain.types().count()
    );
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_LAYOUTS_OUTPUT").map(PathBuf::from);
    if let Some(root) = &out {
        write(
            root,
            &format!("{name}.mpcert"),
            artifact.certificate_bytes(),
        );
        let source = json!({"module":artifact.module_name(),"source_error_type":artifact.source_error_type_id(),"cases":artifact.case_type_ids(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>()});
        write(
            root,
            &format!("{name}.sources.json"),
            &serde_json::to_vec_pretty(&source).unwrap(),
        );
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
fn source_derived_empty_single_and_dispatch_grown_universes() {
    for name in ["nil-only", "boxing-only", "entry"] {
        check(name, dispatch(name).domain());
    }
}
#[test]
fn source_roots_keep_plain_root_string_array_and_zero_size_dependencies() {
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
    check("all-root-shapes", &domain);
}
#[test]
fn source_large_universe_keeps_every_case() {
    let selected = dispatch("seventy-cases");
    assert_eq!(selected.domain().error_cases().len(), 70);
    check("source-N70", selected.domain());
}
