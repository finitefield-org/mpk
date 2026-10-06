use mpk_cert::encode::{Certificate, DeclarationKind as D, LevelNode as L, TermNode as T};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, BTreeSet}, env, fs};
fn digest(v: &Value) -> String { format!("{:x}", Sha256::digest(serde_json::to_vec(v).unwrap())) }
fn read_hex(path: &str) -> Vec<u8> { let s=fs::read_to_string(path).unwrap(); let s=s.trim(); assert_eq!(s.len()%2,0); (0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect() }
fn name(s:&str,map:&BTreeMap<String,String>)->String { let mut n=s.to_owned();for(a,b) in map { if n.contains(a) {n=n.replace(a,b);} }n }
fn fingerprint(c:&Certificate,map:&BTreeMap<String,String>)->Value {
 assert!(c.imports.is_empty() && c.proof_node_table.is_empty() && c.theory_certificates.is_empty() && c.axiom_report.entries.is_empty() && c.source_manifest.is_none());
 let rebound_names=c.name_table.iter().map(|s|name(s,map)).collect::<Vec<_>>();
 let global=|i:u32| rebound_names[c.declarations[i as usize].name as usize].clone();
 let mut levels:Vec<String>=vec![];
 for l in &c.level_table { let v=match l {L::Zero=>json!(["zero"]),L::Succ(a)=>json!(["succ",levels[*a as usize]]),L::Max(a,b)=>json!(["max",levels[*a as usize],levels[*b as usize]]),L::Param(a)=>json!(["param",rebound_names[*a as usize].clone()])};levels.push(digest(&v)); }
 let mut terms:Vec<String>=vec![];
 for t in &c.term_table {
  let v=match t { T::Sort(a)=>json!(["sort",levels[*a as usize]]),T::Var(a)=>json!(["var",a]),T::Const{global:g,levels:ls}=>json!(["const",global(*g),ls.iter().map(|i|levels[*i as usize].clone()).collect::<Vec<_>>()]),T::App{function,arguments}=>json!(["app",terms[*function as usize],arguments.iter().map(|i|terms[*i as usize].clone()).collect::<Vec<_>>()]),T::Lam{ty,body}=>json!(["lam",terms[*ty as usize],terms[*body as usize]]),T::Pi{ty,body}=>json!(["pi",terms[*ty as usize],terms[*body as usize]]),T::Let{ty,value,body}=>json!(["let",terms[*ty as usize],terms[*value as usize],terms[*body as usize]])};terms.push(digest(&v));
 }
 let mut declarations=BTreeMap::new();
 for d in &c.declarations {
  let v=match &d.kind {D::Axiom{..}|D::TheoryPrimitive{..}=>panic!("ordinary zero-axiom scope violated"),D::Def{ty,value,reducibility}=>json!(["def",terms[*ty as usize],terms[*value as usize],format!("{reducibility:?}")]),D::Theorem{ty,proof}=>json!(["theorem",terms[*ty as usize],terms[*proof as usize]]),D::Inductive{ty}=>json!(["inductive",terms[*ty as usize]]),D::Constructor{ty,inductive,generated}=>json!(["constructor",terms[*ty as usize],global(*inductive),generated]),D::Recursor{ty,inductive,generated}=>json!(["recursor",terms[*ty as usize],global(*inductive),generated])};assert!(declarations.insert(rebound_names[d.name as usize].clone(),v).is_none());
 }
 let names=rebound_names.iter().cloned().collect::<BTreeSet<_>>();
 assert_eq!(names.len(),c.name_table.len());
 let mut exports=c.export_block.iter().map(|e|json!([rebound_names[e.name as usize].clone(),global(e.declaration)])).collect::<Vec<_>>();
 exports.sort_by_key(|x|serde_json::to_vec(x).unwrap());
 terms.sort();let mut levels_sorted=levels;levels_sorted.sort();
 json!({"module":c.module,"names":names,"levels":levels_sorted,"terms":terms,"declarations":declarations,"exports":exports,"axiom_summary":format!("{:?}",c.axiom_report.summary)})
}
fn main(){let args=env::args().collect::<Vec<_>>();assert_eq!(args.len(),4);let map:BTreeMap<String,String>=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();let a=read_hex(&args[2]);let b=read_hex(&args[3]);let old=mpk_cert::decode_canonical_certificate(&a).unwrap();let new=mpk_cert::decode_canonical_certificate(&b).unwrap();let x=fingerprint(&old,&map);let y=fingerprint(&new,&BTreeMap::new());let same=x==y; if !same { for key in ["names","declarations"] { let a=x[key].clone(); let b=y[key].clone(); let left=if key=="names" {a.as_array().unwrap().iter().map(|v|v.as_str().unwrap().to_owned()).collect::<BTreeSet<_>>()} else {a.as_object().unwrap().keys().cloned().collect::<BTreeSet<_>>()}; let right=if key=="names" {b.as_array().unwrap().iter().map(|v|v.as_str().unwrap().to_owned()).collect::<BTreeSet<_>>()} else {b.as_object().unwrap().keys().cloned().collect::<BTreeSet<_>>()}; eprintln!("{}",json!({"section":key,"only_old":left.difference(&right).collect::<Vec<_>>(),"only_new":right.difference(&left).collect::<Vec<_>>() })); } }let differing=x.as_object().unwrap().keys().filter(|k|x[*k]!=y[*k]).cloned().collect::<Vec<_>>();println!("{}",json!({"status":if same{"passed_exact_term_declaration_graph_after_actual_context_name_rebinding"}else{"failed_graph_comparison"},"certificate_bytes_equal":a==b,"old_raw_bytes":a.len(),"new_raw_bytes":b.len(),"old_terms":old.term_table.len(),"new_terms":new.term_table.len(),"old_declarations":old.declarations.len(),"new_declarations":new.declarations.len(),"old_graph_sha256":digest(&x),"new_graph_sha256":digest(&y),"differing_sections":differing,"name_mapping_entries":map.len()}));assert!(same);}
