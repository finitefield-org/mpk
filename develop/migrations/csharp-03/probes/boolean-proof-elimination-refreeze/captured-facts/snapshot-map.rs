use mpk_vc::csharp_practical_registry::{
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, SUCCESSOR_CANDIDATE_REVISION,
};
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::{env, fs, path::Path};
#[allow(dead_code)]
#[path = "/private/tmp/mpk-w09-context-application-integration/crates/mpk-cli/tests/support/csharp_practical_data_context.rs"]
mod support;
fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3);
    let bundle = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"kind":"compiled_context","revision":SUCCESSOR_CANDIDATE_REVISION,"foundation_sha256":FOUNDATION_DESCRIPTOR_CONTENT_SHA256})
    );
    let plan: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    for row in plan.as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        let requests: Value =
            serde_json::from_slice(&fs::read(Path::new(&args[2]).join(path)).unwrap()).unwrap();
        for (index, request) in requests.as_array().unwrap().iter().enumerate() {
            let (_, captures) = support::replay_context(&bundle, request);
            println!(
                "{}",
                json!({"request_path":path,"row_index":index,"original_id":request["id"],"snapshot_sha256":captures.snapshot_sha256()})
            );
        }
    }
}
