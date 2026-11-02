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
    let unit = if matches!(name, "length-only" | "index-only" | "literal-only") {
        "pure"
    } else {
        name
    };
    let row = if name == "retained-error" {
        let p=root().parent().unwrap().join("source-string-lengths/corpus.json");
        let old:Value=serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
        let case=&old["cases"][0]["input"];
        json!({"input":old["input"],"roots":case["roots"].as_array().unwrap().iter().map(|r|r["ref"].clone()).collect::<Vec<_>>(),"cases":[],"entry":case["roots"].as_array().unwrap().iter().find(|r|r["name"]=="Entry").unwrap()["ref"]})
    } else {c["units"].as_array().unwrap().iter().find(|r|r["name"]==unit).unwrap().clone()};
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
        .filter(|r| {
            let expected = match name {
                "length-only" => Some("Length"),
                "index-only" => Some("At"),
                "literal-only" => Some("Literal"),
                _ => None,
            };
            expected.is_none_or(|name| {
                derive_go_source_signature(&scopes, r.source_object_id(), r.node())
                    .unwrap()
                    .name()
                    == Some(name)
            })
        })
        .collect::<Vec<_>>();
    let cases = row["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "TypeSpec"))
        .collect::<Vec<_>>();
    let entries = if row["entry"].is_null() {
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
        write(&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"prior_module":row.prior_module_name(),"source_module":row.source_layout_module_name(),"binding":row.source_binding_sha256(),"caller":[site.caller().source_object_id(),site.caller().node()],"node":site.node(),"start":site.start_byte(),"end":site.end_byte(),"caller_span":[s.selection().source(site.caller().source_object_id()).unwrap().nodes()[site.caller().node()].start(),s.selection().source(site.caller().source_object_id()).unwrap().nodes()[site.caller().node()].end()],"operand_spans":site.operand_nodes().iter().map(|n|{let a=&s.selection().source(site.caller().source_object_id()).unwrap().nodes()[*n];json!([a.start(),a.end()])}).collect::<Vec<_>>(),"kind":format!("{:?}",site.kind()),"operands":site.operand_nodes(),"operand_types":site.operand_type_ids(),"result":site.result_type_id(),"calls":site.operand_calls().iter().map(|c|json!([c.call_node(),c.callee().source_object_id(),c.callee().node()])).collect::<Vec<_>>(),"types":s.domain().types().collect::<Vec<_>>(),"underlying":s.domain().types().filter_map(|t|s.domain().underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"cases":s.domain().error_cases().iter().map(|e|e.value_type_id()).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_identities_unchanged":true,"source_expression_execution_or_context_authority":false})).unwrap());
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
    assert_eq!((count, constants), (11, 1));
    write(&format!("{name}-inventory.json"),&serde_json::to_vec_pretty(&json!({"dynamic_rows":count,"constant_sites":constants,"sites":c.sites().iter().map(|site|json!({"source":site.caller().source_object_id(),"caller":site.caller().node(),"node":site.node(),"start":site.start_byte(),"end":site.end_byte(),"caller_span":[s.selection().source(site.caller().source_object_id()).unwrap().nodes()[site.caller().node()].start(),s.selection().source(site.caller().source_object_id()).unwrap().nodes()[site.caller().node()].end()],"operand_spans":site.operand_nodes().iter().map(|n|{let a=&s.selection().source(site.caller().source_object_id()).unwrap().nodes()[*n];json!([a.start(),a.end()])}).collect::<Vec<_>>(),"kind":format!("{:?}",site.kind()),"operand_types":site.operand_type_ids(),"result":site.result_type_id(),"calls":site.operand_calls().len()})).collect::<Vec<_>>()})).unwrap());
}
#[test]
fn source_operations_pure() {
    check("pure")
}
#[test]
fn source_operations_wide() {
    check("wide")
}

fn one(name: &str) {
    let s = selection(name);
    assert!(s.domain().error_cases().is_empty());
    assert!(!s
        .domain()
        .types()
        .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="error")));
    if name == "length-only" {
        assert!(!s
            .domain()
            .types()
            .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="uint8")))
    }
    if name == "index-only" {
        assert!(!s
            .domain()
            .types()
            .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="int")))
    }
    if name == "literal-only" {
        assert!(s
            .domain()
            .types()
            .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..} if name=="string")))
    }
    let c = generate_go_practical_source_string_operation_catalogue(s.domain()).unwrap();
    assert_eq!(c.sites().len(), 1);
    let site = &c.sites()[0];
    let row = c.generate_row(site.caller(), site.node()).unwrap();
    write(&format!("{name}.mpcert"), row.certificate_bytes());
    let r = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
    write(
        &format!("{name}.rust.json"),
        format!("{}\n", r.json).as_bytes(),
    );
    assert!(r.accepted, "{}", r.json);
}
#[test]
fn source_operations_length_without_error_or_byte() {
    one("length-only")
}
#[test]
fn source_operations_index_without_error_or_int() {
    one("index-only")
}
#[test]
fn source_operations_literal_materializes_string() {
    one("literal-only")
}

fn encoded(mut c: mpk_cert::encode::Certificate) -> Vec<u8> {
    use mpk_cert::encode::LevelNode;
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    assert!(c.imports.is_empty() && c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn push(c:&mut mpk_cert::encode::Certificate,t:mpk_cert::encode::TermNode)->u32 {let n=c.term_table.len() as u32;c.term_table.push(t);n}
fn constant(c:&mut mpk_cert::encode::Certificate,name:&str)->u32 {let id=c.declarations.iter().position(|d|c.name_table[d.name as usize]==name).unwrap();push(c,mpk_cert::encode::TermNode::Const{global:id as u32,levels:vec![]})}
fn body(c:&mut mpk_cert::encode::Certificate,t:u32,value:u32)->u32 {match c.term_table[t as usize].clone(){mpk_cert::encode::TermNode::Lam{ty,body:b}=>{let b=body(c,b,value);push(c,mpk_cert::encode::TermNode::Lam{ty,body:b})},_=>value}}
#[test]
fn source_operations_controls() {
 use mpk_cert::encode::{Declaration,DeclarationKind,TermNode};
 let s=selection("pure");let c=generate_go_practical_source_string_operation_catalogue(s.domain()).unwrap();
 let lengths=c.sites().iter().filter(|s|s.kind()==&GoSourceStringOperationKind::Length).collect::<Vec<_>>();
 let a=lengths.iter().find(|a|lengths.iter().any(|b|a.node()!=b.node() && a.operand_type_ids()==b.operand_type_ids())).unwrap();
 let b=lengths.iter().find(|b|a.node()!=b.node() && a.operand_type_ids()==b.operand_type_ids()).unwrap();
 let row=c.generate_row(a.caller(),a.node()).unwrap();assert_eq!(c.import_row(row.certificate_bytes(),b.caller(),b.node()),Err(GoOrdinaryError::Linkage));
 let original=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
 for (name,domain) in [("zero-result",false),("erased-domain",true),("extra-theorem",false)] {
  let mut candidate=original.clone();
  let no=constant(&mut candidate,"Std.Bool.false");let boolean=constant(&mut candidate,"Std.Bool");let eq=constant(&mut candidate,"Std.Eq");let truth=push(&mut candidate,TermNode::App{function:eq,arguments:vec![boolean,no,no]});
  if name=="extra-theorem" {
   let refl=constant(&mut candidate,"Std.Eq.refl");let proof=push(&mut candidate,TermNode::App{function:refl,arguments:vec![boolean,no]});let name=candidate.name_table.len() as u32;candidate.name_table.push(format!("{}.CallerExtra",row.module_name()));candidate.declarations.push(Declaration{name,kind:DeclarationKind::Theorem{ty:truth,proof}});
  }else{
   let target=format!("{}.{}",row.module_name(),if domain{"Domain"}else{"Result"});let id=candidate.declarations.iter().position(|d|candidate.name_table[d.name as usize]==target).unwrap();
   let DeclarationKind::Def{ty,value,reducibility}=candidate.declarations[id].kind else{panic!()};
   let v=if domain{truth}else{constant(&mut candidate,&format!("{}.T{}.Zero",row.source_layout_module_name(),a.result_type_id().unwrap()))};let value=body(&mut candidate,value,v);candidate.declarations[id].kind=DeclarationKind::Def{ty,value,reducibility};
  }
  let raw=encoded(candidate);assert_eq!(c.import_row(&raw,a.caller(),a.node()),Err(GoOrdinaryError::Linkage));write(&format!("control-{name}.mpcert"),&raw);
  let r=mpk_kernel::verify_certificate_bytes_json_output(&raw);write(&format!("control-{name}.rust.json"),format!("{}\n",r.json).as_bytes());assert_eq!(r.accepted,name=="extra-theorem","{}",r.json);let report:Value=serde_json::from_str(&r.json).unwrap();if r.accepted{assert_eq!(report["axiom_count"],0)}else{assert_eq!(report["error_code"],"KERNEL_CORE_CHECK");let detail:Value=serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();assert_eq!(detail["code"],"CORE_TYPE_MISMATCH")}
 }
}

#[test]
fn source_operations_preserve_error_bearing_predecessor_bytes() {
 let selected=selection("retained-error");
 let catalogue=generate_go_practical_source_string_length_catalogue(selected.domain()).unwrap();
 let string=&catalogue.strings().source_string_type_ids()[0];let row=catalogue.generate_row(string).unwrap();
 let old=root().parent().unwrap().join(format!("source-string-lengths/rust/certificates/source-length-empty-{string}.mpcert"));
 assert_eq!(row.certificate_bytes(),std::fs::read(&old).unwrap());
 write("retained-error-bytes.json",&serde_json::to_vec_pretty(&json!({"status":"identical","prior":old,"module":row.module_name(),"bytes":row.certificate_bytes().len(),"source_error_type_preserved":true,"both_original_checker_receipts_retained":true})).unwrap());
}
