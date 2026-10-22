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
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_REPRESENTATION_OUTPUT").map(PathBuf::from);
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
fn target_name(t: &GoErrorImplementation) -> String {
    let m = t.method();
    format!(
        "Target.C{}.S{}.N{}",
        t.value_type_id(),
        m.declaration_source_object_id().unwrap(),
        m.declaration_node().unwrap()
    )
}


fn reject(name:&str,catalogue:&GoSourceErrorDispatchCatalogue,row:&GoSourceErrorRepresentationRow,c:mpk_cert::encode::Certificate){
 let bytes=encoded(c);let p=row.prior_receiver_row().prior_dispatch_row();
 assert_eq!(catalogue.import_representation_row(&bytes,p.call(),p.boxed_source_type_id()),Err(GoOrdinaryError::Linkage));candidate(name,&bytes,false);
}
fn nonzero8(c:&mut mpk_cert::encode::Certificate)->u32{
 use mpk_cert::encode::TermNode;
 let boolean=constant(c,"Std.Bool");let mut value=constant(c,"Std.Bool.true");
 for _ in 0..3{value=node(c,TermNode::Lam{ty:boolean,body:value});}value
}
#[test]
fn representation_tag_inactive_copy_and_construction_controls(){
 use mpk_cert::encode::{DeclarationKind,TermNode};
 let selected=dispatch("entry");let catalogue=generate_go_practical_source_error_dispatch_catalogue(&selected).unwrap();
 let site=&catalogue.calls()[0];let id=&catalogue.case_tags()[0].0;let inactive=&catalogue.case_tags()[1].0;
 let row=catalogue.generate_representation_row(site,Some(id)).unwrap();let receiver=row.prior_receiver_row();let prior=receiver.prior_dispatch_row();let module=row.module_name();
 let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-representation/entries/certificates").join(format!("entry-call0-{id}.mpcert"));
 assert_eq!(row.certificate_bytes(),std::fs::read(&original).unwrap());
 let fresh=||mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
 let mut c=fresh();let i=member(&c,&format!("{module}.RepresentedCase"));
 let DeclarationKind::Def{value,..}=c.declarations[i].kind else{panic!()};
 let TermNode::Lam{ty,..}=c.term_table[value as usize].clone() else{panic!()};
 let predicate=constant(&mut c,&format!("{}.ReceiverTag",receiver.module_name()));let e=node(&mut c,TermNode::Var(0));
 let body=node(&mut c,TermNode::App{function:predicate,arguments:vec![e]});let value=node(&mut c,TermNode::Lam{ty,body});
 define(&mut c,&format!("{module}.RepresentedCase"),value);reject("erased-inactive-predicate",&catalogue,&row,c);
 let mut c=fresh();let wrong=nonzero8(&mut c);change_rhs(&mut c,&format!("{module}.ConstructedInactiveZero.C{inactive}"),wrong);reject("nonzero-inactive-construction",&catalogue,&row,c);
 let mut c=fresh();let wrong=nonzero8(&mut c);change_rhs(&mut c,&format!("{module}.CopyInactiveZero.C{inactive}"),wrong);reject("nonzero-inactive-copy",&catalogue,&row,c);
 let mut c=fresh();let i=member(&c,&format!("{module}.ConstructedRepresentation"));
 let DeclarationKind::Theorem{ty,proof}=c.declarations[i].kind else{panic!()};let TermNode::Lam{ty:domain,..}=c.term_table[proof as usize].clone() else{panic!()};
 let old=constant(&mut c,&format!("{}.ConstructedReceiverTag",receiver.module_name()));let v=node(&mut c,TermNode::Var(0));let body=node(&mut c,TermNode::App{function:old,arguments:vec![v]});let proof=node(&mut c,TermNode::Lam{ty:domain,body});c.declarations[i].kind=DeclarationKind::Theorem{ty,proof};reject("missing-inactive-witnesses",&catalogue,&row,c);
 let mut c=fresh();let no=constant(&mut c,&format!("{}.NoTarget",prior.module_name()));change_rhs(&mut c,&format!("{module}.SelectedFromRepresentation"),no);reject("wrong-represented-target",&catalogue,&row,c);
 let nil=catalogue.generate_representation_row(site,None).unwrap();let mut c=mpk_cert::decode_canonical_certificate(nil.certificate_bytes()).unwrap();let yes=constant(&mut c,"Std.Bool.true");change_rhs(&mut c,&format!("{}.IsNonNilFromRepresentation",nil.module_name()),yes);reject("nil-representation-is-non-nil",&catalogue,&nil,c);
 // The forged receiver has the correct original tag and active value, but a
 // full inactive uint8 payload of255. It cannot reuse the Box witness.
 let mut c=fresh();let i=member(&c,&format!("{module}.ConstructedRepresentation"));
 let DeclarationKind::Theorem{ty,proof}=c.declarations[i].kind else{panic!()};let TermNode::Pi{ty:domain,body}=c.term_table[ty as usize].clone() else{panic!()};let TermNode::App{function,arguments}=c.term_table[body as usize].clone() else{panic!()};assert_eq!(arguments.len(),1);
 let make=constant(&mut c,&format!("{}.Make",prior.boxing_module_name()));let tag=constant(&mut c,&format!("{}.C{id}.Tag",prior.boxing_module_name()));let mut values=vec![tag];
 for (index,(case,_)) in catalogue.case_tags().iter().enumerate(){values.push(if index==0{node(&mut c,TermNode::Var(0))}else if index==1{nonzero8(&mut c)}else{constant(&mut c,&format!("{}.T{case}.Zero",prior.source_layout_module_name()))});}
 let junk=node(&mut c,TermNode::App{function:make,arguments:values});let body=node(&mut c,TermNode::App{function,arguments:vec![junk]});let ty=node(&mut c,TermNode::Pi{ty:domain,body});c.declarations[i].kind=DeclarationKind::Theorem{ty,proof};reject("forged-tagged-junk-representation",&catalogue,&row,c);
 let mut c=fresh();let zero=constant(&mut c,"Std.Nat.zero");let succ=constant(&mut c,"Std.Nat.succ");let one=node(&mut c,TermNode::App{function:succ,arguments:vec![zero]});let two=node(&mut c,TermNode::App{function:succ,arguments:vec![one]});
 define(&mut c,&format!("{}.{}",prior.module_name(),target_name(&catalogue.targets()[0])),two);let bytes=encoded(c);assert_eq!(catalogue.import_representation_row(&bytes,site,Some(id)),Err(GoOrdinaryError::Linkage));candidate("wrong-ordinal-is-only-math",&bytes,true);
 if let Some(out)=std::env::var_os("MPK_GO_SOURCE_ERROR_REPRESENTATION_OUTPUT"){write(&PathBuf::from(out),"wrong-ordinal-is-only-math.sources.json",&std::fs::read(original.with_extension("sources.json")).unwrap());}
}
#[test]
fn last_large_representation_keeps_each_inactive_zero(){
 let selected=dispatch("seventy-cases");let catalogue=generate_go_practical_source_error_dispatch_catalogue(&selected).unwrap();let site=&catalogue.calls()[0];let id=&catalogue.case_tags()[69].0;let inactive=&catalogue.case_tags()[0].0;
 let row=catalogue.generate_representation_row(site,Some(id)).unwrap();let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-representation/first-large/certificates").join(format!("seventy-cases-call0-{id}.mpcert"));assert_eq!(row.certificate_bytes(),std::fs::read(original).unwrap());
 let mut c=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();let wrong=nonzero8(&mut c);change_rhs(&mut c,&format!("{}.ConstructedInactiveZero.C{inactive}",row.module_name()),wrong);reject("last-large-inactive-is-nonzero",&catalogue,&row,c);
}
fn change_rhs(c: &mut mpk_cert::encode::Certificate, name: &str, result: u32) {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    fn rebuild(c: &mut mpk_cert::encode::Certificate, ty: u32, result: u32) -> u32 {
        match c.term_table[ty as usize].clone() {
            TermNode::Pi { ty, body } => {
                let body = rebuild(c, body, result);
                node(c, TermNode::Pi { ty, body })
            }
            TermNode::App {
                function,
                mut arguments,
            } => {
                assert_eq!(arguments.len(), 3);
                arguments[2] = result;
                node(
                    c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                )
            }
            _ => panic!("equation conclusion"),
        }
    }
    let i = member(c, name);
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
        panic!()
    };
    let ty = rebuild(c, ty, result);
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
}
