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
use mpk_cert::encode::{Certificate,DeclarationKind,TermNode};
use std::collections::{BTreeSet,BTreeMap};
#[path="support/go_word_division_dependencies.rs"]
mod dependencies;
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
fn closed(original:&Certificate,roots:&[usize])->Certificate {
 let mut globals=BTreeSet::new();let mut terms=BTreeSet::new();
 let mut decl_work=roots.to_vec();
 for (i,d) in original.declarations.iter().enumerate() {if matches!(d.kind,DeclarationKind::Inductive{..}|DeclarationKind::Constructor{..}|DeclarationKind::Recursor{..}) {decl_work.push(i);}}
 let mut term_work=vec![];
 while !decl_work.is_empty() || !term_work.is_empty() {
  if let Some(i)=decl_work.pop() {
   if !globals.insert(i) {continue;}
   match original.declarations[i].kind {
    DeclarationKind::Def{ty,value,..}=>term_work.extend([ty,value]),
    DeclarationKind::Theorem{ty,proof}=>term_work.extend([ty,proof]),
    DeclarationKind::Inductive{ty}=>term_work.push(ty),
    DeclarationKind::Constructor{ty,inductive,..}|DeclarationKind::Recursor{ty,inductive,..}=>{term_work.push(ty);decl_work.push(inductive as usize);},
    _=>panic!(),
   }
  }else if let Some(i)=term_work.pop() {
   if !terms.insert(i) {continue;}
   match &original.term_table[i as usize] {
    TermNode::Const{global,..}=>decl_work.push(*global as usize),
    TermNode::App{function,arguments}=>{term_work.push(*function);term_work.extend(arguments);},
    TermNode::Lam{ty,body}|TermNode::Pi{ty,body}=>term_work.extend([*ty,*body]),
    TermNode::Let{ty,value,body}=>term_work.extend([*ty,*value,*body]),
    _=>{},
   }
  }
 }
 let gm=globals.iter().enumerate().map(|(new,old)|(*old,new as u32)).collect::<BTreeMap<_,_>>();
 let tm=terms.iter().enumerate().map(|(new,old)|(*old,new as u32)).collect::<BTreeMap<_,_>>();
 let mut c=original.clone();c.term_table=terms.iter().map(|i|match &original.term_table[*i as usize] {
  TermNode::Const{global,levels}=>TermNode::Const{global:gm[&(*global as usize)],levels:levels.clone()},
  TermNode::App{function,arguments}=>TermNode::App{function:tm[function],arguments:arguments.iter().map(|a|tm[a]).collect()},
  TermNode::Lam{ty,body}=>TermNode::Lam{ty:tm[ty],body:tm[body]},
  TermNode::Pi{ty,body}=>TermNode::Pi{ty:tm[ty],body:tm[body]},
  TermNode::Let{ty,value,body}=>TermNode::Let{ty:tm[ty],value:tm[value],body:tm[body]},
  other=>other.clone(),
 }).collect();
 c.declarations=globals.iter().map(|i| {let mut d=original.declarations[*i].clone();d.kind=match d.kind {
  DeclarationKind::Def{ty,value,reducibility}=>DeclarationKind::Def{ty:tm[&ty],value:tm[&value],reducibility},
  DeclarationKind::Theorem{ty,proof}=>DeclarationKind::Theorem{ty:tm[&ty],proof:tm[&proof]},
  DeclarationKind::Inductive{ty}=>DeclarationKind::Inductive{ty:tm[&ty]},
  DeclarationKind::Constructor{ty,inductive,generated}=>DeclarationKind::Constructor{ty:tm[&ty],inductive:gm[&(inductive as usize)],generated},
  DeclarationKind::Recursor{ty,inductive,generated}=>DeclarationKind::Recursor{ty:tm[&ty],inductive:gm[&(inductive as usize)],generated},
  _=>panic!(),
 };d}).collect();
 let expected=dependencies::declarations(original);
 for (name,identity) in dependencies::declarations(&c) {assert_eq!(expected.get(&name),Some(&identity));}
 c
}
#[test]
fn current_complete_case_proofs_in_closed_diagnostic() {
 let unit=dispatch("seventy-cases");let domain=unit.domain();let generated=generate_go_practical_source_error_boxing_foundation(domain);
 let out=PathBuf::from(std::env::var_os("MPK_GO_SOURCE_ERROR_BOXINGS_OUTPUT").unwrap());std::fs::create_dir_all(&out).unwrap();
 if let Err(e)=&generated {write(&out,"generation-error.json",&serde_json::to_vec(&json!({"error":format!("{e:?}")})).unwrap());}
 let f=generated.unwrap();write(&out,"complete-unverified-N70.mpcert",f.certificate_bytes());
 let c=mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
 let roots=c.declarations.iter().enumerate().filter(|(_,d)|c.name_table[d.name as usize].starts_with(&(c.module.clone()+"."))).map(|(i,_)|i).collect::<Vec<_>>();assert_eq!(roots.len(),857);
 let component=closed(&c,&roots);let count=component.declarations.len();let terms=component.term_table.len();let bytes=encoded(component);
 write(&out,"closed-current-N70.mpcert",&bytes);let r=mpk_kernel::verify_certificate_bytes_json_output(&bytes);write(&out,"closed-current-N70.rust.json",format!("{}\n",r.json).as_bytes());
 write(&out,"scope.json",&serde_json::to_vec_pretty(&json!({"complete_stream_declarations":c.declarations.len(),"complete_stream_terms":c.term_table.len(),"complete_stream_bytes":f.certificate_bytes().len(),"all_new_roots":roots.len(),"closed_declarations":count,"closed_terms":terms,"closed_verdict_accepted":r.accepted,"scope":"All current new case-proof dependencies preserve complete original named identities. Diagnostic only: the complete full-library70-case stream still requires both cores and original byte comparison."})).unwrap());
 assert!(r.accepted,"{}",r.json);
}
