use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::Value;
use std::io::Write;
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
fn node(c: &mut mpk_cert::encode::Certificate, n: mpk_cert::encode::TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(n);
    id
}
fn constant(c: &mut mpk_cert::encode::Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    node(
        c,
        mpk_cert::encode::TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn candidate(name: &str, bytes: &[u8], accepted: bool) {
    let out = std::env::var_os("MPK_GO_SOURCE_VALUE_RELATIONS_OUTPUT").map(PathBuf::from);
    if let Some(root) = &out {
        write(root, &format!("{name}.mpcert"), bytes);
    }
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = out {
        write(
            &root,
            &format!("{name}.rust.json"),
            format!("{}\n", r.json).as_bytes(),
        );
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    let report: Value = serde_json::from_str(&r.json).unwrap();
    if accepted {
        assert_eq!(report["axiom_count"], 0);
    } else {
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: Value = serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn member(c: &mpk_cert::encode::Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn define(c: &mut mpk_cert::encode::Certificate, name: &str, value: u32) {
    use mpk_cert::encode::{DeclarationKind, DefinitionReducibility};
    let i = member(c, name);
    let DeclarationKind::Def { ty, .. } = c.declarations[i].kind else {
        panic!()
    };
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility: DefinitionReducibility::Reducible,
    };
}

fn reject(name: &str, catalogue: &GoSourceValueRelationCatalogue, row: &GoSourceValueRelationRow, c: mpk_cert::encode::Certificate) {
    let bytes=encoded(c);
    assert_eq!(catalogue.import_row(&bytes,row.boxed_source_type_id()),Err(GoOrdinaryError::Linkage));
    candidate(name,&bytes,false);
}
fn nonzero8(c: &mut mpk_cert::encode::Certificate) -> u32 {
    use mpk_cert::encode::TermNode;
    let ty=constant(c,"Std.Bool");
    let mut body=constant(c,"Std.Bool.true");
    for _ in 0..3 {body=node(c,TermNode::Lam{ty,body});}
    body
}
fn relation_rhs(c: &mut mpk_cert::encode::Certificate, name: &str, right:u32) {
    use mpk_cert::encode::{TermNode,DeclarationKind};
    fn rebuild(c:&mut mpk_cert::encode::Certificate,ty:u32,right:u32)->u32 {
        match c.term_table[ty as usize].clone() {
            TermNode::Pi{ty,body}=>{let body=rebuild(c,body,right);node(c,TermNode::Pi{ty,body})},
            TermNode::App{function,mut arguments}=>{assert_eq!(arguments.len(),2);arguments[1]=right;node(c,TermNode::App{function,arguments})},
            _=>panic!("closed relation conclusion")
        }
    }
    let i=member(c,name);
    let DeclarationKind::Theorem{ty,proof}=c.declarations[i].kind else {panic!()};
    let ty=rebuild(c,ty,right);
    c.declarations[i].kind=DeclarationKind::Theorem{ty,proof};
}
fn builtin(catalogue:&GoSourceValueRelationCatalogue,name:&str)->String {
    catalogue.source_types().iter().find(|t|matches!(t,GoSourceTypeKey::Builtin{name:n,..}if n==name)).unwrap().id().to_string()
}
#[test]
fn complete_relation_zero_copy_box_and_source_identity_controls() {
    use mpk_cert::encode::{TermNode,DeclarationKind};
    let selected=dispatch("entry");
    let catalogue=generate_go_practical_source_value_relation_catalogue(selected.domain()).unwrap();
    let id=&catalogue.case_tags()[0].0;
    let row=catalogue.generate_row(Some(id)).unwrap();
    let file=format!("entry-{id}");
    let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-value-relations/projections/certificates").join(format!("{file}.mpcert"));
    assert_eq!(row.certificate_bytes(),std::fs::read(&original).unwrap());
    let fresh=||mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let mut c=fresh();
    let wrong=nonzero8(&mut c);
    let byte=builtin(&catalogue,"uint8");
    relation_rhs(&mut c,&format!("{}.T{byte}.ZeroRelated",row.relation_module_name()),wrong);
    reject("nonzero-source-byte-related-to-zero",&catalogue,&row,c);
    let mut c=fresh();
    let nil=constant(&mut c,&format!("{}.Nil",row.operations_module_name()));
    relation_rhs(&mut c,&format!("{}.CopyRelated",row.module_name()),nil);
    reject("arbitrary-copy-related-to-nil",&catalogue,&row,c);
    let mut c=fresh();
    let nil=constant(&mut c,&format!("{}.Nil",row.operations_module_name()));
    relation_rhs(&mut c,&format!("{}.BoxRelated",row.module_name()),nil);
    reject("related-box-is-nil",&catalogue,&row,c);
    let mut c=fresh();
    let wrong=nonzero8(&mut c);
    relation_rhs(&mut c,&format!("{}.ActiveRelated",row.module_name()),wrong);
    reject("boxed-relation-forces-wrong-active-value",&catalogue,&row,c);
    // A coherent constant View still supports mathematics. It must fail the
    // source factory's exact import and the independent identity-body oracle.
    let mut c=fresh();
    let name=format!("{}.T{byte}.View",row.relation_module_name());
    let DeclarationKind::Def{value,..}=c.declarations[member(&c,&name)].kind else {panic!()};
    let TermNode::Lam{ty,..}=c.term_table[value as usize].clone() else {panic!()};
    let body=constant(&mut c,&format!("{}.T{byte}.Zero",row.source_layout_module_name()));
    let value=node(&mut c,TermNode::Lam{ty,body});define(&mut c,&name,value);
    let bytes=encoded(c);
    assert_eq!(catalogue.import_row(&bytes,Some(id)),Err(GoOrdinaryError::Linkage));
    candidate("constant-byte-view-is-only-math",&bytes,true);
    if let Some(out)=std::env::var_os("MPK_GO_SOURCE_VALUE_RELATIONS_OUTPUT") {
        write(&PathBuf::from(out),"constant-byte-view-is-only-math.sources.json",&std::fs::read(original.with_extension("sources.json")).unwrap());
    }
}
#[test]
fn heterogeneous_relation_observes_string_bytes_slice_owner_and_bool() {
    use mpk_cert::encode::{TermNode,DeclarationKind};
    let selected=dispatch("heterogeneous-payloads");
    let catalogue=generate_go_practical_source_value_relation_catalogue(selected.domain()).unwrap();
    let row=catalogue.generate_row(None).unwrap();
    let fresh=||mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let mut c=fresh();
    let yes=constant(&mut c,"Std.Bool.true");
    let id=builtin(&catalogue,"bool");
    relation_rhs(&mut c,&format!("{}.T{id}.ZeroRelated",row.relation_module_name()),yes);
    reject("source-bool-zero-is-true",&catalogue,&row,c);
    let mut c=fresh();
    let id=builtin(&catalogue,"string");
    let name=format!("{}.T{id}.Related",row.relation_module_name());
    let DeclarationKind::Def{value,..}=c.declarations[member(&c,&name)].kind else {panic!()};
    let TermNode::Lam{ty:x,body}=c.term_table[value as usize].clone() else {panic!()};
    let TermNode::Lam{ty:y,body}=c.term_table[body as usize].clone() else {panic!()};
    let TermNode::Pi{ty,body}=c.term_table[body as usize].clone() else {panic!()};
    let TermNode::App{function,mut arguments}=c.term_table[body as usize].clone() else {panic!()};
    assert_eq!(arguments.len(),3);arguments[1]=arguments[0];
    let body=node(&mut c,TermNode::App{function,arguments});
    let body=node(&mut c,TermNode::Pi{ty,body});
    let body=node(&mut c,TermNode::Lam{ty:y,body});
    let value=node(&mut c,TermNode::Lam{ty:x,body});define(&mut c,&name,value);
    reject("string-bytes-replaced-by-length-observation",&catalogue,&row,c);
    let mut c=fresh();
    let id=catalogue.source_types().iter().find(|t|matches!(t,GoSourceTypeKey::Slice{..})).unwrap().id();
    let name=format!("{}.T{id}.Related",row.relation_module_name());
    let DeclarationKind::Def{value,..}=c.declarations[member(&c,&name)].kind else {panic!()};
    let zero=constant(&mut c,"Std.Nat.zero");let mut changed=false;
    fn erase_owner(c:&mut mpk_cert::encode::Certificate,i:u32,zero:u32,changed:&mut bool)->u32 {
        if *changed{return i;}
        match c.term_table[i as usize].clone() {
            TermNode::App{function,mut arguments}=>{
                if let TermNode::Const{global,..}=c.term_table[function as usize] {
                    if c.name_table[c.declarations[global as usize].name as usize]=="Std.Eq" && arguments.len()==3 {
                        if let TermNode::Const{global,..}=c.term_table[arguments[0] as usize] {
                            if c.name_table[c.declarations[global as usize].name as usize]=="Std.Nat" {
                                arguments[1]=zero;arguments[2]=zero;*changed=true;
                                return node(c,TermNode::App{function,arguments});
                            }
                        }
                    }
                }
                let function=erase_owner(c,function,zero,changed);
                for a in &mut arguments {*a=erase_owner(c,*a,zero,changed);}
                node(c,TermNode::App{function,arguments})
            },
            TermNode::Lam{ty,body}=>{let body=erase_owner(c,body,zero,changed);node(c,TermNode::Lam{ty,body})},
            TermNode::Pi{ty,body}=>{let body=erase_owner(c,body,zero,changed);node(c,TermNode::Pi{ty,body})},
            _=>i
        }
    }
    let value=erase_owner(&mut c,value,zero,&mut changed);assert!(changed);define(&mut c,&name,value);
    reject("slice-owner-observation-erased",&catalogue,&row,c);
}
#[test]
fn last_large_relation_keeps_first_inactive_payload() {
    let selected=dispatch("seventy-cases");
    let catalogue=generate_go_practical_source_value_relation_catalogue(selected.domain()).unwrap();
    let id=&catalogue.case_tags()[69].0;let inactive=&catalogue.case_tags()[0].0;
    let row=catalogue.generate_row(Some(id)).unwrap();
    let mut c=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let wrong=nonzero8(&mut c);
    relation_rhs(&mut c,&format!("{}.CopyFieldRelated.C{inactive}",row.module_name()),wrong);
    reject("last-large-wrong-inactive-copy-relation",&catalogue,&row,c);
}
