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
    if name.starts_with("source-length-") {
        corpus("source-string-lengths")
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
    let catalogue =
        generate_go_practical_source_string_length_catalogue(selection.domain()).unwrap();
    assert_eq!(catalogue.strings().source_string_type_ids().len(), 4);
    assert!(!catalogue
        .strings()
        .source_types()
        .iter()
        .any(|t| matches!(t,GoSourceTypeKey::Builtin{name,..}if name=="uint8")));
    let int = catalogue.source_int_type_id().unwrap();
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_LENGTH_OUTPUT").map(PathBuf::from);
    for string in catalogue.strings().source_string_type_ids() {
        let prior = catalogue.strings().generate_row(string).unwrap();
        let before = dependencies::declarations(
            &mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap(),
        );
        let row = catalogue.generate_row(string).unwrap();
        assert_eq!(row.source_result_type_id(), int);
        let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let after = dependencies::declarations(&decoded);
        for (n, v) in &before {
            assert_eq!(
                after.get(n),
                Some(v),
                "all prior original string identities: {n}"
            );
        }
        assert_eq!(
            catalogue
                .import_row(row.certificate_bytes(), string)
                .unwrap(),
            row
        );
        assert_eq!(
            catalogue.import_row(row.certificate_bytes(), "wrong-string"),
            Err(GoOrdinaryError::Linkage)
        );
        let file = format!("{name}-{string}");
        if let Some(root) = &out {
            write(root, &format!("{file}.mpcert"), row.certificate_bytes());
            write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"source_layout_module":row.prior_string_row().source_layout_module_name(),"string_module":row.prior_string_row().module_name(),"source_string_type_id":string,"source_result_type_id":int,"cases":catalogue.strings().case_type_ids(),"types":catalogue.strings().source_types(),"underlying":catalogue.strings().source_types().iter().filter_map(|t|catalogue.strings().underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_string_identities_unchanged":true,"source_expression_or_context_authority":false})).unwrap());
        }
        let start = std::time::Instant::now();
        let r = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        println!("{file}: {:?} {}", start.elapsed(), r.accepted);
        if let Some(root) = &out {
            write(
                root,
                &format!("{file}.rust.json"),
                format!("{}\n", r.json).as_bytes(),
            );
        }
        assert!(r.accepted, "{}", r.json);
        let report: Value = serde_json::from_str(&r.json).unwrap();
        assert_eq!(report["axiom_count"], 0);
    }
}
#[test]
fn source_lengths_empty() {
    check("source-length-empty")
}
#[test]
fn source_lengths_small() {
    check("source-length-small")
}
#[test]
fn source_lengths_wide() {
    check("source-length-wide")
}
#[test]
fn source_lengths_no_fabricated_int() {
    let selection = dispatch("nil-only");
    let c = generate_go_practical_source_string_length_catalogue(selection.domain()).unwrap();
    assert_eq!(c.source_int_type_id(), None);
    assert_eq!(
        c.generate_row(&c.strings().source_string_type_ids()[0]),
        Err(GoOrdinaryError::Linkage)
    );
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



fn app(c:&mut mpk_cert::encode::Certificate,f:u32,args:Vec<u32>)->u32{node(c,mpk_cert::encode::TermNode::App{function:f,arguments:args})}
fn lambda_body(c:&mut mpk_cert::encode::Certificate,t:u32,value:u32)->u32{use mpk_cert::encode::TermNode;match c.term_table[t as usize].clone(){TermNode::Lam{ty,body}=>{let body=lambda_body(c,body,value);node(c,TermNode::Lam{ty,body})},_=>value}}
fn control(name:&str,catalogue:&GoSourceStringLengthCatalogue,row:&GoSourceStringLengthRow,c:mpk_cert::encode::Certificate,accepted:bool){
 let bytes=encoded(c);let id=row.prior_string_row().source_string_type_id();assert_eq!(catalogue.import_row(&bytes,id),Err(GoOrdinaryError::Linkage));
 let out=std::env::var_os("MPK_GO_SOURCE_STRING_LENGTH_OUTPUT").map(PathBuf::from);let file=format!("control-{name}");
 if let Some(root)=&out{write(root,&format!("{file}.mpcert"),&bytes);}
 let r=mpk_kernel::verify_certificate_bytes_json_output(&bytes);if let Some(root)=&out{write(root,&format!("{file}.rust.json"),format!("{}\n",r.json).as_bytes());}
 assert_eq!(r.accepted,accepted,"{name}: {}",r.json);let report:Value=serde_json::from_str(&r.json).unwrap();if accepted{assert_eq!(report["axiom_count"],0)}else{assert_eq!(report["error_code"],"KERNEL_CORE_CHECK");let detail:Value=serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();assert_eq!(detail["code"],"CORE_TYPE_MISMATCH");}
}
#[test]fn source_lengths_controls(){
 use mpk_cert::encode::{DeclarationKind,Declaration,TermNode};
 let selection=dispatch("source-length-small");let catalogue=generate_go_practical_source_string_length_catalogue(selection.domain()).unwrap();let string=&catalogue.strings().source_string_type_ids()[0];let row=catalogue.generate_row(string).unwrap();let original=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
 let mut c=original.clone();let name=format!("{}.Len",row.module_name());let DeclarationKind::Def{value,..}=c.declarations[member(&c,&name)].kind else{panic!()};let z=constant(&mut c,&format!("{}.T{}.Zero",row.prior_string_row().source_layout_module_name(),row.source_result_type_id()));let body=lambda_body(&mut c,value,z);define(&mut c,&name,body);control("constant-length",&catalogue,&row,c,false);
 let mut c=original.clone();let name=format!("{}.AsInteger",row.module_name());let DeclarationKind::Def{value,..}=c.declarations[member(&c,&name)].kind else{panic!()};let n=constant(&mut c,&format!("{}.Natural",row.module_name()));let v=node(&mut c,TermNode::Var(0));let n=app(&mut c,n,vec![v]);let from=constant(&mut c,"Mpk.Go.Practical.Int.FromNat");let n=app(&mut c,from,vec![n]);let value=lambda_body(&mut c,value,n);define(&mut c,&name,value);control("unsigned-integer-interpretation",&catalogue,&row,c,false);
 let mut c=original.clone();let eq=constant(&mut c,"Std.Eq");let boolean=constant(&mut c,"Std.Bool");let no=constant(&mut c,"Std.Bool.false");let truth=app(&mut c,eq,vec![boolean,no,no]);
 let law=member(&c,&format!("{}.FitsNonnegative",row.module_name()));let DeclarationKind::Theorem{ty,proof}=c.declarations[law].kind else{panic!()};
 fn domain(c:&mut mpk_cert::encode::Certificate,t:u32,depth:usize,truth:u32)->u32{match c.term_table[t as usize].clone(){TermNode::Pi{ty,body}=>{let (ty,body)=if depth==0{(truth,body)}else{(ty,domain(c,body,depth-1,truth))};node(c,TermNode::Pi{ty,body})},TermNode::Lam{ty,body}=>{let (ty,body)=if depth==0{(truth,body)}else{(ty,domain(c,body,depth-1,truth))};node(c,TermNode::Lam{ty,body})},_=>panic!()}}
 let ty=domain(&mut c,ty,1,truth);let proof=domain(&mut c,proof,1,truth);c.declarations[law].kind=DeclarationKind::Theorem{ty,proof};control("erased-real-length-bound",&catalogue,&row,c,false);
 let mut c=original;let boolean=constant(&mut c,"Std.Bool");let no=constant(&mut c,"Std.Bool.false");let eq=constant(&mut c,"Std.Eq");let ty=app(&mut c,eq,vec![boolean,no,no]);let refl=constant(&mut c,"Std.Eq.refl");let proof=app(&mut c,refl,vec![boolean,no]);let name=c.name_table.len() as u32;c.name_table.push(format!("{}.CallerExtra",row.module_name()));c.declarations.push(Declaration{name,kind:DeclarationKind::Theorem{ty,proof}});
 if let Some(root)=std::env::var_os("MPK_GO_SOURCE_STRING_LENGTH_OUTPUT").map(PathBuf::from){write(&root,"control-extra.sources.json",&serde_json::to_vec_pretty(&json!({"unit":"source-length-small","module":row.module_name(),"source_layout_module":row.prior_string_row().source_layout_module_name(),"string_module":row.prior_string_row().module_name(),"source_string_type_id":string,"source_result_type_id":row.source_result_type_id(),"cases":catalogue.strings().case_type_ids(),"types":catalogue.strings().source_types(),"underlying":catalogue.strings().source_types().iter().filter_map(|t|catalogue.strings().underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>()})).unwrap());}
 control("extra",&catalogue,&row,c,true);
}
