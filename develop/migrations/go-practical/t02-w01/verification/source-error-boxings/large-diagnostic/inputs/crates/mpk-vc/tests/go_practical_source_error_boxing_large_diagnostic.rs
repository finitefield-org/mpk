use mpk_cert::encode::{Certificate,DeclarationKind,TermNode};
use std::collections::{BTreeSet,BTreeMap};
use std::path::{Path,PathBuf};
use std::io::Write;
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
fn isolate_original_large_fuel_boundary_without_unrelated_prefix_rechecks() {
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-boxings");
 let raw=std::fs::read(root.join("shared-large/certificates/source-boxing-N70.mpcert")).unwrap();let c=mpk_cert::decode_canonical_certificate(&raw).unwrap();
 let roots=c.declarations.iter().enumerate().filter(|(_,d)|c.name_table[d.name as usize].starts_with(&(c.module.clone()+"."))).map(|(i,_)|i).collect::<Vec<_>>();assert_eq!(roots.len(),854);
 let out=PathBuf::from(std::env::var_os("MPK_GO_SOURCE_ERROR_BOXINGS_OUTPUT").unwrap());std::fs::create_dir_all(&out).unwrap();
 let mut lo=0;let mut hi=roots.len();
 while lo+1<hi {
  let mid=(lo+hi)/2;let prefix=closed(&c,&roots[..mid]);let count=prefix.declarations.len();let bytes=encoded(prefix);let name=format!("closed-{mid}");
  std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.mpcert"))).unwrap().write_all(&bytes).unwrap();
  let r=mpk_kernel::verify_certificate_bytes_json_output(&bytes);
  std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.rust.json"))).unwrap().write_all(format!("{}\n",r.json).as_bytes()).unwrap();
  println!("closed roots={mid} declarations={count} accepted={} last={}",r.accepted,c.name_table[c.declarations[roots[mid-1]].name as usize]);
  if r.accepted {lo=mid;}else{hi=mid;}
 }
 let name=&c.name_table[c.declarations[roots[hi-1]].name as usize];std::fs::write(out.join("boundary.json"),serde_json::to_vec_pretty(&serde_json::json!({"original":"shared-large/certificates/source-boxing-N70.mpcert","last_accepted_new_roots":lo,"first_rejected_new_roots":hi,"first_rejected_declaration":name,"complete_named_types_bodies_proofs_and_family_metadata_retained_for_every_selected_dependency":true,"scope":"Only diagnostic closed dependencies of the original failed full stream. This is not a passing full70-case stream or a substitute for its final validation."})).unwrap()).unwrap();println!("first rejected {name}");
}
