use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-string-operations")
}
fn corpus() -> Value {
    serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn selection(name: &str) -> GoSourceErrorSelection {
    let c = corpus();
    let row = c["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == name)
        .unwrap();
    let input = &row["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys =
        import_go_type_key_components(&unhex(input["components_hex"].as_str().unwrap()), &sources)
            .unwrap();
    let scopes = build_go_source_scopes(&keys).unwrap();
    let locate = |r: &Value, kind: &str| {
        let source = r["source"].as_str().unwrap();
        let node = scopes
            .source(source)
            .unwrap()
            .nodes()
            .iter()
            .position(|n| {
                n.kind() == kind
                    && n.start() == r["start"].as_u64().unwrap() as usize
                    && n.end() == r["end"].as_u64().unwrap() as usize
            })
            .unwrap();
        GoSourceSelectionRef::new(source, node)
    };
    let roots = row["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = row["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "TypeSpec"))
        .collect::<Vec<_>>();
    let entries = if cases.is_empty() {
        vec![]
    } else {
        vec![GoErrorEntryConstraint::new(
            locate(&row["entry"], "FuncDecl"),
            0,
            vec![],
            cases,
        )]
    };
    derive_go_source_error_selection(&keys, &roots, &entries).unwrap()
}
fn write(name: &str, raw: &[u8]) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_STRING_OPERATION_OUTPUT") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn check(name: &str) {
    let s = selection(name);
    let c = generate_go_practical_source_string_operation_catalogue(s.domain()).unwrap();
    assert!(!c.sites().is_empty());
    assert_eq!(
        s.domain().error_cases().len(),
        if name == "wide" { 70 } else { 0 }
    );
    assert_eq!(
        s.domain()
            .types()
            .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="error")),
        name == "wide"
    );
    let mut count = 0;
    let mut constants = 0;
    for (ordinal, site) in c.sites().iter().enumerate() {
        if matches!(site.kind(), GoSourceStringOperationKind::ConstantLength(_)) {
            constants += 1;
            assert_eq!(
                c.generate_row(site.caller(), site.node()),
                Err(GoOrdinaryError::Foundation)
            );
            continue;
        }
        let row = c.generate_row(site.caller(), site.node()).unwrap();
        let prior = mpk_cert::decode_canonical_certificate(row.prior_certificate_bytes()).unwrap();
        let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let before = dependencies::declarations(&prior);
        let after = dependencies::declarations(&decoded);
        for (n, d) in &before {
            assert_eq!(after.get(n), Some(d), "complete prior identity {n}")
        }
        assert_eq!(
            c.import_row(row.certificate_bytes(), site.caller(), site.node())
                .unwrap(),
            row
        );
        assert_eq!(
            c.import_row(
                row.certificate_bytes(),
                &GoSourceSelectionRef::new("missing", 0),
                site.node()
            ),
            Err(GoOrdinaryError::Linkage)
        );
        let file = format!("{name}-{ordinal:03}");
        write(&format!("{file}.mpcert"), row.certificate_bytes());
        write(&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"prior_module":row.prior_module_name(),"source_module":row.source_layout_module_name(),"binding":row.source_binding_sha256(),"caller":[site.caller().source_object_id(),site.caller().node()],"node":site.node(),"start":site.start_byte(),"end":site.end_byte(),"kind":format!("{:?}",site.kind()),"operands":site.operand_nodes(),"operand_types":site.operand_type_ids(),"result":site.result_type_id(),"calls":site.operand_calls().iter().map(|c|json!([c.call_node(),c.callee().source_object_id(),c.callee().node()])).collect::<Vec<_>>(),"types":s.domain().types().collect::<Vec<_>>(),"underlying":s.domain().types().filter_map(|t|s.domain().underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"cases":s.domain().error_cases().iter().map(|e|e.value_type_id()).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_identities_unchanged":true,"source_expression_execution_or_context_authority":false})).unwrap());
        let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        write(
            &format!("{file}.rust.json"),
            format!("{}\n", result.json).as_bytes(),
        );
        assert!(result.accepted, "{file}: {}", result.json);
        let report: Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["axiom_count"], 0);
        count += 1;
    }
    assert!(count >= 5 && constants >= 1);
    write(&format!("{name}-inventory.json"),&serde_json::to_vec_pretty(&json!({"dynamic_rows":count,"constant_sites":constants,"sites":c.sites().iter().map(|site|json!({"source":site.caller().source_object_id(),"caller":site.caller().node(),"node":site.node(),"start":site.start_byte(),"end":site.end_byte(),"kind":format!("{:?}",site.kind()),"operand_types":site.operand_type_ids(),"result":site.result_type_id(),"calls":site.operand_calls().len()})).collect::<Vec<_>>()})).unwrap());
}
#[test]
fn source_operations_pure() {
    check("pure")
}
#[test]
fn source_operations_wide() {
    check("wide")
}

fn one(name:&str) {
 let s=selection(name);assert!(s.domain().error_cases().is_empty());assert!(!s.domain().types().any(|k|matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="error")));
 if name=="length-only" {assert!(!s.domain().types().any(|k|matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="uint8")))}
 if name=="index-only" {assert!(!s.domain().types().any(|k|matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="int")))}
 if name=="literal-only" {assert!(s.domain().types().any(|k|matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="string")))}
 let c=generate_go_practical_source_string_operation_catalogue(s.domain()).unwrap();assert_eq!(c.sites().len(),1);let site=&c.sites()[0];let row=c.generate_row(site.caller(),site.node()).unwrap();
 write(&format!("{name}.mpcert"),row.certificate_bytes());
 let r=mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());write(&format!("{name}.rust.json"),format!("{}\n",r.json).as_bytes());assert!(r.accepted,"{}",r.json);
}
#[test]fn source_operations_length_without_error_or_byte(){one("length-only")}
#[test]fn source_operations_index_without_error_or_int(){one("index-only")}
#[test]fn source_operations_literal_materializes_string(){one("literal-only")}
