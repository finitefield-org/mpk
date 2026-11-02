use std::path::{Path,PathBuf};
use std::io::Write;
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
#[test]
fn isolate_original_conversion_fuel_boundary() {
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-boxings");
 let raw=std::fs::read(root.join("first/certificates/boxing-only.mpcert")).unwrap();
 let full=mpk_cert::decode_canonical_certificate(&raw).unwrap();
 let source:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("first/certificates/boxing-only.sources.json")).unwrap()).unwrap();
 let start=source["prior_source_declarations"].as_u64().unwrap() as usize;
 let out=PathBuf::from(std::env::var_os("MPK_GO_SOURCE_ERROR_BOXINGS_OUTPUT").unwrap());std::fs::create_dir_all(&out).unwrap();
 // Each complete prefix is closed because declarations are dependency ordered.
 let mut lo=start;let mut hi=full.declarations.len();
 while lo+1<hi {
  let mid=(lo+hi)/2;let mut c=full.clone();c.declarations.truncate(mid);let bytes=encoded(c);
  let name=format!("prefix-{mid}");
  std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.mpcert"))).unwrap().write_all(&bytes).unwrap();
  let report=mpk_kernel::verify_certificate_bytes_json_output(&bytes);
  std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(format!("{name}.rust.json"))).unwrap().write_all(format!("{}\n",report.json).as_bytes()).unwrap();
  println!("prefix{mid} accepted={} last={}",report.accepted,full.name_table[full.declarations[mid-1].name as usize]);
  if report.accepted {lo=mid;}else{hi=mid;}
 }
 let name=&full.name_table[full.declarations[hi-1].name as usize];
 std::fs::write(out.join("boundary.json"),serde_json::to_vec_pretty(&serde_json::json!({"original_failed_stream":"first/certificates/boxing-only.mpcert","last_accepted_prefix":lo,"first_rejected_prefix":hi,"first_rejected_declaration":name,"scope":"Fuel diagnosis of the retained original full stream; no semantic counterexample or profile claim."})).unwrap()).unwrap();
 println!("first rejected declaration {name}");
}
