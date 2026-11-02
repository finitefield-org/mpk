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
fn replace_field(
    c: &mut mpk_cert::encode::Certificate,
    value: u32,
    len: usize,
    index: usize,
    new: u32,
) -> u32 {
    use mpk_cert::encode::TermNode;
    if len == 1 {
        assert_eq!(index, 0);
        return new;
    }
    let TermNode::Lam { ty, body } = c.term_table[value as usize].clone() else {
        panic!("actual frozen product constructor")
    };
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[body as usize].clone()
    else {
        panic!()
    };
    assert_eq!(arguments.len(), 4);
    let middle = len / 2;
    if index < middle {
        arguments[1] = replace_field(c, arguments[1], middle, index, new);
    } else {
        arguments[2] = replace_field(c, arguments[2], len - middle, index - middle, new);
    }
    let body = node(
        c,
        TermNode::App {
            function,
            arguments,
        },
    );
    node(c, TermNode::Lam { ty, body })
}
fn candidate(name: &str, bytes: &[u8], accepted: bool) {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_OPERATIONS_OUTPUT").map(PathBuf::from);
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
fn changed_definition(c:&mut mpk_cert::encode::Certificate,name:&str,value:u32) {
 use mpk_cert::encode::{DeclarationKind,DefinitionReducibility};
 let i=member(c,name);let DeclarationKind::Def{ty,..}=c.declarations[i].kind else {panic!()};
 c.declarations[i].kind=DeclarationKind::Def{ty,value,reducibility:DefinitionReducibility::Reducible};
}
fn copy_body(c:&mut mpk_cert::encode::Certificate,f:&GoSourceErrorOperationsFoundation,field:usize,value:u32) {
 use mpk_cert::encode::{DeclarationKind,TermNode};
 let name=format!("{}.Copy",f.module_name());let i=member(c,&name);let DeclarationKind::Def{value:old,..}=c.declarations[i].kind else {panic!()};
 let TermNode::Lam{ty,body}=c.term_table[old as usize].clone() else {panic!()};
 let body=replace_field(c,body,f.case_tags().len()+1,field,value);let body=node(c,TermNode::Lam{ty,body});changed_definition(c,&name,body);
}
fn reject(name:&str,domain:&GoSourceValueDomain,c:mpk_cert::encode::Certificate) {
 let bytes=encoded(c);assert_eq!(import_go_practical_source_error_operations_foundation(&bytes,domain),Err(GoOrdinaryError::Linkage));candidate(name,&bytes,false);
}
fn nil_body(c:&mut mpk_cert::encode::Certificate,f:&GoSourceErrorOperationsFoundation,field:usize,value:u32) {
 use mpk_cert::encode::DeclarationKind;
 let original=member(c,&format!("{}.T{}.Zero",f.source_layout_module_name(),f.source_error_type_id()));
 let DeclarationKind::Def{value:body,..}=c.declarations[original].kind else {panic!()};
 let body=replace_field(c,body,f.case_tags().len()+1,field,value);changed_definition(c,&format!("{}.Nil",f.module_name()),body);
}
fn false_proof(c:&mut mpk_cert::encode::Certificate,proof:u32)->u32 {
 use mpk_cert::encode::TermNode;
 if let TermNode::Lam{ty,body}=c.term_table[proof as usize].clone() {let body=false_proof(c,body);return node(c,TermNode::Lam{ty,body});}
 let function=constant(c,"Std.Eq.refl");let boolean=constant(c,"Std.Bool");let no=constant(c,"Std.Bool.false");node(c,TermNode::App{function,arguments:vec![boolean,no]})
}
#[test]
fn small_nil_copy_controls_and_exact_case_test_ownership() {
 use mpk_cert::encode::{DeclarationKind,TermNode};
 let domain=root_domain();let f=generate_go_practical_source_error_operations_foundation(&domain).unwrap();assert_eq!(f.case_tags().len(),4);
 let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-operations/shared-observation/certificates/all-root-shapes.mpcert");assert_eq!(f.certificate_bytes(),std::fs::read(original).unwrap());
 let fresh=||mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
 let source=|i:usize|format!("{}.T{}.Zero",f.source_layout_module_name(),f.case_tags()[i].0);
 let mut c=fresh();let tag=constant(&mut c,&format!("{}.C{}.Tag",f.boxing_module_name(),f.case_tags()[0].0));changed_definition(&mut c,&format!("{}.NilTag",f.module_name()),tag);reject("nil-tag-is-nonzero",&domain,c);
 let mut c=fresh();let function=constant(&mut c,&format!("{}.C{}.Box",f.boxing_module_name(),f.case_tags()[0].0));let zero=constant(&mut c,&source(0));let value=node(&mut c,TermNode::App{function,arguments:vec![zero]});changed_definition(&mut c,&format!("{}.Nil",f.module_name()),value);reject("nil-is-concrete",&domain,c);
 let keys=domain.types().map(|t|(t.id(),t)).collect::<std::collections::BTreeMap<_,_>>();
 let flag=f.case_tags().iter().position(|(id,_)|matches!(keys[domain.underlying(id).unwrap()],GoSourceTypeKey::Builtin{name,..}if name=="bool")).unwrap();
 let mut c=fresh();let yes=constant(&mut c,"Std.Bool.true");nil_body(&mut c,&f,flag+1,yes);reject("nil-inactive-bool",&domain,c);
 let reference=f.case_tags().iter().position(|(id,_)|matches!(keys[domain.underlying(id).unwrap()],GoSourceTypeKey::Struct{fields,..}if fields.iter().any(|field|matches!(keys[field.type_id.as_str()],GoSourceTypeKey::Slice{..})))).unwrap();
 let GoSourceTypeKey::Struct{fields,..}=keys[domain.underlying(&f.case_tags()[reference].0).unwrap()] else {panic!()};assert_eq!(fields.len(),1);
 let mut c=fresh();let i=member(&c,&format!("{}.T{}.Zero",f.source_layout_module_name(),fields[0].type_id));let DeclarationKind::Def{value,..}=c.declarations[i].kind else {panic!()};let no=constant(&mut c,"Std.Bool.false");let bad_slice=replace_field(&mut c,value,6,0,no);nil_body(&mut c,&f,reference+1,bad_slice);reject("nil-inactive-slice-nil",&domain,c);
 let mut c=fresh();let zero=constant(&mut c,&format!("{}.NilTag",f.module_name()));copy_body(&mut c,&f,0,zero);reject("copy-erases-tag",&domain,c);
 for i in 0..4 {let mut c=fresh();let zero=constant(&mut c,&source(i));copy_body(&mut c,&f,i+1,zero);reject(&format!("copy-erases-payload-{i}"),&domain,c);}
 // A constant-false case test has coherent nil/copy laws, but is not the
 // frozen tag comparison. Heal only its own mathematical laws explicitly.
 let mut c=fresh();let name=format!("{}.HasCase",f.module_name());let i=member(&c,&name);let DeclarationKind::Def{value,..}=c.declarations[i].kind else {panic!()};
 let TermNode::Lam{ty:case_ty,body}=c.term_table[value as usize].clone() else {panic!()};let TermNode::Lam{ty:error_ty,..}=c.term_table[body as usize].clone() else {panic!()};
 let no=constant(&mut c,"Std.Bool.false");let body=node(&mut c,TermNode::Lam{ty:error_ty,body:no});let value=node(&mut c,TermNode::Lam{ty:case_ty,body});changed_definition(&mut c,&name,value);
 let ids=c.declarations.iter().enumerate().filter(|(_,d)|{let n=&c.name_table[d.name as usize];n==&format!("{}.HasCaseOfCopy",f.module_name()) || (n.starts_with(&(f.module_name().to_string()+".C")) && n.ends_with(".NilHasNoCase")) || n==&format!("{}.DifferentTagHasNoCase",f.module_name())}).map(|(i,_)|i).collect::<Vec<_>>();
 for i in ids {let DeclarationKind::Theorem{ty,proof}=c.declarations[i].kind else {panic!()};let proof=false_proof(&mut c,proof);c.declarations[i].kind=DeclarationKind::Theorem{ty,proof};}
 let bytes=encoded(c);assert_eq!(import_go_practical_source_error_operations_foundation(&bytes,&domain),Err(GoOrdinaryError::Linkage));candidate("constant-false-case-test-is-only-math",&bytes,true);
 if let Some(out)=std::env::var_os("MPK_GO_SOURCE_ERROR_OPERATIONS_OUTPUT") {let meta=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-operations/shared-observation/certificates/all-root-shapes.sources.json");write(&PathBuf::from(out),"constant-false-case-test-is-only-math.sources.json",&std::fs::read(meta).unwrap());}
}
#[test]
fn large_copy_keeps_last_source_payload() {
 let selected=dispatch("seventy-cases");let domain=selected.domain();let f=generate_go_practical_source_error_operations_foundation(domain).unwrap();assert_eq!(f.case_tags().len(),70);
 let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-operations/first-large/certificates/source-operations-N70.mpcert");assert_eq!(f.certificate_bytes(),std::fs::read(original).unwrap());
 let mut c=mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();let zero=constant(&mut c,&format!("{}.T{}.Zero",f.source_layout_module_name(),f.case_tags()[69].0));copy_body(&mut c,&f,70,zero);reject("copy-erases-N70-last-payload",domain,c);
}
