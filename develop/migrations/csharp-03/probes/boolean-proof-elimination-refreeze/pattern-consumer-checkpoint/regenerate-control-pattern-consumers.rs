//! Exact original eighteen-source replay for current pattern consumer producers.
use mpk_vc::csharp_practical_vir_model::*;
use mpk_vc::csharp_practical_registry::{SUCCESSOR_CANDIDATE_REVISION,FOUNDATION_DESCRIPTOR_CONTENT_SHA256};
use mpk_vc::csharp_practical_vc_model::validate_csharp_practical_certificate_structure;
use serde_json::{json,Value};
use std::{fs,path::Path};
#[allow(dead_code)]
#[path="/private/tmp/mpk-w09-refreeze-metadata-integration/crates/mpk-cli/tests/support/csharp_practical_data_context.rs"]
mod support;
fn read(root:&Path,p:&str)->Value {serde_json::from_slice(&fs::read(root.join(p)).unwrap()).unwrap()}
fn main(){
 let args=std::env::args().collect::<Vec<_>>();assert_eq!(args.len(),3);let root=Path::new(&args[1]);let out=Path::new(&args[2]);fs::create_dir(out).unwrap();
 let bundle=validate_registered_foundation_bundle(registered_foundation_descriptor_transport(),registered_foundation_definitions_transport()).unwrap();assert_eq!(SUCCESSOR_CANDIDATE_REVISION,5);assert_eq!(FOUNDATION_DESCRIPTOR_CONTENT_SHA256,"99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844");
 let mut requests=read(root,"develop/migrations/csharp-03/control-vc/loop-requests.json");let mut responses=read(root,"develop/migrations/csharp-03/control-emission/loop-responses.json");
 requests.as_array_mut().unwrap().extend(read(root,"develop/migrations/csharp-03/control-vc/measure-requests.json").as_array().unwrap().iter().filter(|r|r["id"]=="total_variable").cloned());
 responses.as_array_mut().unwrap().extend(read(root,"develop/migrations/csharp-03/control-vc/measure-responses.json").as_array().unwrap().iter().filter(|r|r["id"]=="total_variable").cloned());
 let patterns=read(root,"develop/migrations/csharp-03/control-emission/source-cases.json");let mut rows=vec![];
 for id in ["count_fill","while","for","short_circuit","switch","is_binding","guard_order","guard_throw","total_variable","index_update","foreach_string","foreach_string_var","foreach_array","foreach_array_var","lookup","governing_throw","type","string_property"] {
  let (context,captures,facts)=if let Some(request)=requests.as_array().unwrap().iter().find(|r|r["id"]==id) {
   let response=responses.as_array().unwrap().iter().find(|r|r["id"]==id).unwrap();let (context,captures)=support::replay_context(&bundle,request);(context,captures,response["facts"].clone())
  }else{
   let row=patterns.as_array().unwrap().iter().find(|r|r["stage"]=="patterns" && r["source_case"]["id"]==id).unwrap();assert_eq!(row["accepted"],true);let source=&row["source_case"];let (context,captures)=support::context(&bundle,source["root"].as_str().unwrap(),source["source"].as_str().unwrap().as_bytes());(context,captures,row["data"].clone())
  };
  let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();let vir=emitted.vir();
  let current:Value=read(root,&format!("develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures/{id}.json"));let old_source_ir=current["source_ir_sha256"].clone();
  macro_rules! owner {($variant:expr,$generate:ident,$import:ident)=>{{
   let p=$generate(vir).unwrap();let metadata=p.canonical_bytes();let bytes=p.certificate_bytes().to_vec();let restored=$import(&metadata,&bytes,vir).unwrap();assert_eq!(restored.canonical_bytes(),metadata);assert_eq!(restored.certificate_bytes(),bytes);
   let cert=mpk_cert::decode_canonical_certificate(&bytes).unwrap();validate_csharp_practical_certificate_structure(&cert).unwrap();assert!(cert.imports.is_empty() && cert.proof_node_table.is_empty() && cert.theory_certificates.is_empty() && cert.axiom_report.entries.is_empty() && cert.source_manifest.is_none());
   let parsed:Value=serde_json::from_slice(&metadata).unwrap();assert_eq!(parsed["source_ir_sha256"],vir.hash());assert_eq!(parsed["certificate_sha256"],mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)));let dest=out.join($variant);fs::create_dir_all(&dest).unwrap();fs::write(dest.join(format!("{id}.json")),metadata).unwrap();fs::write(dest.join(format!("{id}.hex")),bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()+"\n").unwrap();
   rows.push(json!({"owner":$variant,"id":id,"old_source_ir_sha256":old_source_ir,"source_ir_sha256":vir.hash(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"strict_import":"passed"}));println!("{id}: {} generated and strictly imported",$variant);
  }}};
  owner!("with-pattern-primitives",generate_csharp_practical_ordinary_control_predicates_with_pattern_observations,import_csharp_practical_ordinary_control_predicates_with_pattern_observations);
  owner!("with-pattern-routes",generate_csharp_practical_ordinary_control_predicates_with_pattern_routes,import_csharp_practical_ordinary_control_predicates_with_pattern_routes);
  owner!("with-packed-pattern-proof-types",generate_csharp_practical_ordinary_control_predicates_with_packed_pattern_proof_types,import_csharp_practical_ordinary_control_predicates_with_packed_pattern_proof_types);
  let control=generate_csharp_practical_control_vcs_with_pattern_routes(vir).unwrap();fs::write(out.join("with-pattern-routes").join(format!("{id}.control.json")),control.canonical_bytes()).unwrap();fs::write(out.join("generation.json"),serde_json::to_vec_pretty(&json!({"status":"running","programs":rows})).unwrap()).unwrap();
 }
 assert_eq!(rows.len(),54);fs::write(out.join("generation.json"),serde_json::to_vec_pretty(&json!({"status":"passed_actual_generation_strict_import_and_structure","programs":rows})).unwrap()).unwrap();
}
