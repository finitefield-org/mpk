import hashlib,json,pathlib,subprocess
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');plan=json.loads((b/'wrapper-program-plan.json').read_bytes())
s='''//! Temporary source-bound lineage tool; all original proof IDs remain pending.
use mpk_vc::csharp_practical_vir_model::*;
use mpk_vc::csharp_practical_registry::{SUCCESSOR_CANDIDATE_REVISION, FOUNDATION_DESCRIPTOR_CONTENT_SHA256};
use mpk_vc::csharp_practical_vc_model::validate_csharp_practical_certificate_structure;
use serde_json::{json,Value};
use std::{collections::BTreeMap,fs,path::Path};
#[allow(dead_code)]
#[path = "/private/tmp/mpk-w09-refreeze-metadata-integration/crates/mpk-cli/tests/support/csharp_practical_data_context.rs"]
mod support;
fn read(p:impl AsRef<Path>)->Value {serde_json::from_slice(&fs::read(p).unwrap()).unwrap()}
fn generate(schema:&str,emitted:&EmittedDataPhase)->(Vec<u8>,Vec<u8>) {
 let vir=emitted.vir();
 macro_rules! program {($generate:ident,$import:ident,$ctx:expr)=>{{
  let p=$generate($ctx).unwrap();let m=p.canonical_bytes();let c=p.certificate_bytes().to_vec();
  let restored=$import(&m,&c,$ctx).unwrap();assert_eq!(restored.canonical_bytes(),m);assert_eq!(restored.certificate_bytes(),c);
  (m,c)
 }}}
 match schema {
'''
for name,ctx in sorted(plan['producers'].items()):s+=f'  "mpk.csharp.ordinary_{name}.v1" => program!(generate_csharp_practical_ordinary_{name},import_csharp_practical_ordinary_{name},{ctx}),\n'
s+='''  _=>panic!("unknown producer {schema}"),
 }
}
fn main(){
 let args=std::env::args().collect::<Vec<_>>();assert_eq!(args.len(),4);
 let plan=read(&args[1]);let root=Path::new(&args[2]);let out=Path::new(&args[3]);fs::create_dir(out).unwrap();
 let bundle=validate_registered_foundation_bundle(registered_foundation_descriptor_transport(),registered_foundation_definitions_transport()).unwrap();
 assert_eq!(SUCCESSOR_CANDIDATE_REVISION,5);assert_eq!(FOUNDATION_DESCRIPTOR_CONTENT_SHA256,"99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844");
 let mut groups=BTreeMap::<String,Vec<&Value>>::new();for target in plan["targets"].as_array().unwrap(){groups.entry(target["old_source_ir_sha256"].as_str().unwrap().into()).or_default().push(target);}
 let mut cache=BTreeMap::<String,Value>::new();let mut rows=vec![];let mut state=json!({"status":"running","revision":SUCCESSOR_CANDIDATE_REVISION,"foundation_sha256":FOUNDATION_DESCRIPTOR_CONTENT_SHA256});
 for (old,targets) in groups {
  let source=&targets[0]["source"];let req=source["request_path"].as_str().unwrap();let resp=source["response_path"].as_str().unwrap();
  for p in [req,resp]{cache.entry(p.to_owned()).or_insert_with(||read(root.join(p)));}
  let id=&source["id"];let request=cache[req].as_array().unwrap().iter().find(|r|r["id"]==*id).unwrap();let response=cache[resp].as_array().unwrap().iter().find(|r|r["id"]==*id).unwrap();
  let (context,captures)=support::replay_context(&bundle,request);let source_data=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&response["facts"]).unwrap()).unwrap();
  let emitted=emit_data_phase(&bundle,&context,&captures,&source_data).unwrap();assert_eq!(emitted.vir().hash(),source["new_source_ir_sha256"].as_str().unwrap());
  for target in targets {
   assert_eq!(target["source"],*source);let schema=target["schema"].as_str().unwrap();let label=target["label"].as_str().unwrap();
   let (metadata,bytes)=generate(schema,&emitted);let parsed:Value=serde_json::from_slice(&metadata).unwrap();assert_eq!(parsed["schema"],schema);let parsed_source=parsed.get("source_ir_sha256").unwrap_or(&parsed["parsers"]["source_ir_sha256"]);assert_eq!(*parsed_source,emitted.vir().hash());
   let certificate=mpk_cert::decode_canonical_certificate(&bytes).unwrap();validate_csharp_practical_certificate_structure(&certificate).unwrap();
   assert!(certificate.imports.is_empty() && certificate.proof_node_table.is_empty() && certificate.theory_certificates.is_empty() && certificate.axiom_report.entries.is_empty() && certificate.source_manifest.is_none());
   let certificate_sha256=mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes));if let Some(hash)=parsed.get("certificate_sha256"){assert_eq!(*hash,certificate_sha256);}
   fs::write(out.join(format!("{label}.program.json")),&metadata).unwrap();let hex=bytes.iter().map(|x|format!("{x:02x}")).collect::<String>()+"\\n";fs::write(out.join(format!("{label}.hex")),hex).unwrap();
   rows.push(json!({"label":label,"schema":schema,"old_source_ir_sha256":old,"source_ir_sha256":emitted.vir().hash(),"source":source,"certificate_sha256":certificate_sha256,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"strict_import":"passed"}));
   println!("generated {label}");
  }
  state["programs"]=json!(rows);fs::write(out.join("generation.json"),serde_json::to_vec_pretty(&state).unwrap()).unwrap();
 }
 assert_eq!(rows.len(),plan["targets"].as_array().unwrap().len());state["status"]=json!("passed_actual_generation_and_strict_import");state["programs"]=json!(rows);fs::write(out.join("generation.json"),serde_json::to_vec_pretty(&state).unwrap()).unwrap();
}
'''
(b/'regenerate-wrapper-programs.rs').write_text(s)
cmd=json.loads((b/'vir-hash-new-compile-command.json').read_bytes());cmd[2]=str(b/'regenerate-wrapper-programs.rs');cmd[-1]=str(b/'regenerate-wrapper-programs');cmd[-2:-2]=['--extern','mpk_cert=/private/tmp/mpk-w09-bool-cases-target/debug/deps/libmpk_cert-73d78a7ad244656d.rlib']
(b/'wrapper-program-compile-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (b/'wrapper-program-compile.log.txt').open('wb') as log:code=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT).returncode
print(json.dumps({'compile_exit_code':code,'source_sha256':hashlib.sha256(s.encode()).hexdigest()}));raise SystemExit(code)
