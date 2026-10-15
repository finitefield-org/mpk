use mpk_vc::csharp_practical_registry::{
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, SUCCESSOR_CANDIDATE_REGISTRY_SHA256,
    SUCCESSOR_CANDIDATE_REVISION,
};
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::{env, fs, path::Path};

#[allow(dead_code)]
#[path = "/private/tmp/mpk-w09-context-application-integration/crates/mpk-cli/tests/support/csharp_practical_data_context.rs"]
mod support;

fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 5);
    let plan = read(Path::new(&args[1]));
    let bundle = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"kind":"compiled_context", "revision":SUCCESSOR_CANDIDATE_REVISION,
            "registry_sha256":SUCCESSOR_CANDIDATE_REGISTRY_SHA256,
            "foundation_sha256":FOUNDATION_DESCRIPTOR_CONTENT_SHA256})
    );
    for corpus in plan.as_array().unwrap() {
        let requests = read(&Path::new(&args[2]).join(corpus["request_path"].as_str().unwrap()));
        let responses = read(&Path::new(&args[3]).join(corpus["response_path"].as_str().unwrap()));
        assert_eq!(
            requests.as_array().unwrap().len(),
            responses.as_array().unwrap().len()
        );
        for (index, response) in responses.as_array().unwrap().iter().enumerate() {
            let id = response["id"].as_str().unwrap();
            let request = requests
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap();
            let mut record = json!({"kind":"replay", "corpus":corpus["request_path"], "index":index,
                "id":id});
            if !response["facts"].is_object() {
                record["result"] = json!("native_source_rejection");
                record["native_response"] = response.clone();
            } else {
                let (ctx, captures) = support::replay_context(&bundle, request);
                let facts = serde_json::to_vec(&response["facts"]).unwrap();
                match ValidatedDataSource::import_captured_facts(&bundle, &ctx, &captures, &facts) {
                    Err(e) => {
                        record["result"] = json!("import_rejected");
                        record["error"] = json!(format!("{e:?}"));
                    }
                    Ok(source) => match emit_data_phase(&bundle, &ctx, &captures, &source) {
                        Err(e) => {
                            record["result"] = json!("emission_rejected");
                            record["error"] = json!(format!("{e:?}"));
                        }
                        Ok(emitted) => {
                            record["result"] = json!("emitted");
                            record["source_ir_sha256"] = json!(emitted.vir().hash());
                            record["contract_expressions"] =
                                json!(emitted.vir().contract_expressions());
                            record["type_routes"] = json!(emitted.routes());
                            record["transition_count"] = json!(emitted.transitions().len());
                            record["boundary_count"] = json!(emitted.boundaries().len());
                        }
                    },
                }
            }
            println!("{record}");
        }
    }
    fs::write(&args[4], b"completed\n").unwrap();
}
