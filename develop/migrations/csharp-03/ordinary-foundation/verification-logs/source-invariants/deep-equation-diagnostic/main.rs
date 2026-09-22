#[path = "/Users/kazuyoshitoshiya/mpk/crates/mpk-vc/src/csharp_practical_ordinary_test_eval.rs"]
#[allow(dead_code)]
mod core_eval;
fn main() {
    use core_eval::{bit, run, V};
    let root = "/Users/kazuyoshitoshiya/mpk/develop/migrations/csharp-03/ordinary-foundation/source-invariants/";
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(format!("{root}certificates.json")).unwrap()).unwrap();
    let row = manifest["sources"].as_array().unwrap().iter().find(|r| r["id"] == "boundary-map-compound").unwrap();
    let metadata = &row["metadata"];
    let id = "mpk.csharp.source.b6b592255570c29551edd02432ac059654faaeb1a0fb2816c9d37351f25ee41c";
    let def = metadata["definitions"].as_array().unwrap().iter().find(|r| r["invariant"]["type_id"] == id).unwrap();
    let domain = metadata["public_domains"].as_array().unwrap().iter().find(|r| r["carrier"]["type_id"] == id).unwrap();
    let hex = std::fs::read_to_string(format!("{root}boundary-map-compound.hex")).unwrap();
    let bytes = hex.trim().as_bytes().chunks_exact(2).map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap()).collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    for (kind, name) in [("domain", domain["valid_definition"].as_str().unwrap()), ("body",def["body_definition"].as_str().unwrap())] {
        let started = std::time::Instant::now();
        eprintln!("begin {kind}, depth 34, independently constructed all-zero cube");
        let result = bit(run(&cert, name, vec![V::UniformCube(false,34)]));
        eprintln!("end {kind}: {result}, {:?}",started.elapsed());
    }
}
