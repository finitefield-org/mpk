use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_GRAPH_MANIFEST_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
fn facts(input: &GoSourceGraphManifest) -> Value {
    json!({"module_path_hex":input.module_path().map(hex),"go_requirement":input.go_requirement(),"requirements":input.requirements().iter().map(|r|json!({"path_hex":hex(&r.path),"version":r.version,"indirect":r.indirect})).collect::<Vec<_>>(),"retractions":input.retractions().iter().map(|r|json!({"low_hex":hex(&r.low),"high_hex":hex(&r.high)})).collect::<Vec<_>>(),"ignored_paths_hex":input.ignored_paths().iter().map(|s|hex(s)).collect::<Vec<_>>(),"pruned":input.supports_graph_pruning(),"implicit_go_requirement":input.implicit_go_requirement(),"requires_newer_compiler":input.requires_newer_compiler()})
}
fn graph_requirements(
    parsed: &GoSourceGraphManifest,
    main: &GoSourceModuleManifest,
    native_names: bool,
) -> Vec<Value> {
    let mut requires = parsed
        .requirements()
        .iter()
        .filter(|r| {
            !main
                .exclusions()
                .iter()
                .any(|e| e.path == r.path && e.version == r.version)
        })
        .map(|r| {
            (
                String::from_utf8(r.path.clone()).unwrap(),
                r.version.clone(),
            )
        })
        .collect::<Vec<_>>();
    if let Some(go) = parsed.implicit_go_requirement() {
        requires.push(("go".into(), go.into()));
    }
    requires
        .into_iter()
        .map(|(path, version)| {
            if native_names {
                json!({"Path":path,"Version":version})
            } else {
                json!({"path":path,"version":version})
            }
        })
        .collect()
}
#[test]
fn complete_original_dependency_parser_matches_native_lax_facts() {
    let corpus=read(&root().join("graph-manifests/native/native-originals/graph-manifests/complete-original-graph-manifests.json"));
    let mut observations = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let raw = unhex(row["raw_hex"].as_str().unwrap());
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        let result = derive_go_source_graph_manifest(source);
        assert_eq!(
            result.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{} origin={} error={:?}",
            row["case"],
            row["origin_main"],
            result.as_ref().err()
        );
        let actual = result.as_ref().ok().map(facts);
        if let Ok(parsed) = &result {
            assert_eq!(parsed.original_source().bytes(), raw);
            assert_eq!(
                actual.as_ref().unwrap(),
                &row["facts"],
                "{} origin={}",
                row["case"],
                row["origin_main"]
            );
        }
        observations.push(json!({"case":row["case"],"origin_main":row["origin_main"],"object":row["object"],"accepted":result.is_ok(),"error":result.err().map(|e|e.to_string()),"facts":actual}));
    }
    output(
        "complete-original-graph-manifest-observations.json",
        &json!({"status":"passed","observations":observations,"whole_original_bytes_retained":true,"independent_MVS_or_source_program_proof":false}),
    );
}

#[test]
fn original_initial_and_final_graph_manifests_derive_without_graph_projections() {
    let base = root().join("module-graphs");
    let mut observations = vec![];
    for epoch in ["edges-final", "replacements-final", "toolchains-final"] {
        let directory = base.join(epoch).join("native-originals");
        let mut files = std::fs::read_dir(&directory)
            .unwrap()
            .map(|r| r.unwrap().path())
            .filter(|p| {
                let name = p.file_name().unwrap().to_str().unwrap();
                name.ends_with(".graph-whole-original-observation.json")
                    || name.ends_with(".graph-virtual-observation.json")
                    || name == "graph-rejected-native-root-expansion.json"
            })
            .collect::<Vec<_>>();
        files.sort();
        for path in files {
            let packet = read(&path);
            for phase in ["initial", "final"] {
                for row in packet[phase].as_array().into_iter().flatten() {
                    let source = &row["source"];
                    if source.is_null() {
                        assert!(matches!(
                            row["identity"]["Path"].as_str().unwrap(),
                            "go" | "toolchain"
                        ));
                        continue;
                    }
                    // Reopen the raw source via its complete saved original
                    // snapshot, never decode the producer's parser facts.
                    let mut source_raw = None;
                    let mut main_source = None;
                    for candidate in std::fs::read_dir(&directory).unwrap() {
                        let p = candidate.unwrap().path();
                        if p.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .ends_with("-inputs.json")
                            && p.file_name().unwrap().to_str().unwrap().contains(".graph-")
                        {
                            let original = read(&p);
                            if let Some(obj) = original["objects"]
                                .as_array()
                                .and_then(|r| r.iter().find(|o| o["id"] == source["object"]["id"]))
                            {
                                assert_eq!(obj, &source["object"]);
                                use base64::Engine;
                                source_raw = Some(
                                    base64::engine::general_purpose::STANDARD
                                        .decode(
                                            original["raw"][obj["id"].as_str().unwrap()]
                                                .as_str()
                                                .unwrap(),
                                        )
                                        .unwrap(),
                                );
                                let main = original["objects"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .find(|o| {
                                        o["capability"] == "workspace/main"
                                            && o["path"] == "go.mod"
                                            && o["kind"] == "go_mod"
                                    })
                                    .unwrap();
                                let data = base64::engine::general_purpose::STANDARD
                                    .decode(
                                        original["raw"][main["id"].as_str().unwrap()]
                                            .as_str()
                                            .unwrap(),
                                    )
                                    .unwrap();
                                main_source = Some(
                                    GoTypeSourceBuffer::import(
                                        &serde_json::to_vec(main).unwrap(),
                                        &data,
                                    )
                                    .unwrap(),
                                );
                                break;
                            }
                        }
                    }
                    let raw = source_raw.expect("whole original manifest snapshot missing");
                    let parsed = derive_go_source_graph_manifest(
                        GoTypeSourceBuffer::import(
                            &serde_json::to_vec(&source["object"]).unwrap(),
                            &raw,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                    assert_eq!(
                        parsed.go_requirement().unwrap_or(""),
                        row["go_version"].as_str().unwrap()
                    );
                    assert_eq!(
                        parsed.supports_graph_pruning(),
                        row["pruned"].as_bool().unwrap()
                    );
                    assert!(!parsed.requires_newer_compiler());
                    let main =
                        derive_go_source_module_manifest(main_source.unwrap(), true).unwrap();
                    let expected = graph_requirements(&parsed, &main, true);
                    assert_eq!(
                        expected,
                        row["requirements"].as_array().cloned().unwrap_or_default()
                    );
                    observations.push(json!({"epoch":epoch,"observation":path.file_name().unwrap().to_str().unwrap(),"phase":phase,"module":row["identity"],"original_object":source["object"],"facts":facts(&parsed),"graph_requirements":expected}));
                }
            }
        }
    }
    assert!(observations
        .iter()
        .any(|r| r["module"]["Path"] == "example.com/leaf"
            && r["module"]["Version"] == "v1.0.0"
            && r["facts"]["go_requirement"].is_null()));
    output(
        "whole-original-initial-final-graph-manifests.json",
        &json!({"status":"passed","observations":observations,"native_root_authority_or_MVS_claimed":false}),
    );
}

fn buffers(packet: &Value, field: &str, directory: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(s) = r["raw_hex"].as_str() {
                unhex(s)
            } else {
                std::fs::read(
                    directory
                        .unwrap()
                        .join("original-source-objects")
                        .join(r["raw_file"].as_str().unwrap()),
                )
                .unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
#[test]
fn complete_original_graph_owner_retains_all_source_export_and_module_inputs() {
    let dir = root().join("module-graphs/native-initial/native-originals/graph-owned");
    let packet = read(&dir.join("complete-original-module-graph-inputs.json"));
    let selection = import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(&packet, "sources", None),
        &buffers(&packet, "overlays", None),
    )
    .unwrap();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(&packet, "snapshot_sources", Some(&dir)),
    )
    .unwrap();
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection,
        snapshot,
    )
    .unwrap();
    let exports = import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap();
    let modules = import_go_source_module_observations(
        &unhex(packet["module_observations_hex"].as_str().unwrap()),
        exports,
    )
    .unwrap();
    let graph: Value =
        serde_json::from_slice(&unhex(packet["module_graph_inputs_hex"].as_str().unwrap()))
            .unwrap();
    let original = modules.export_inputs().capture_inputs().snapshot_inputs();
    assert_eq!(original.source_objects().len(), 5667);
    let main_id = modules
        .export_inputs()
        .capture_inputs()
        .selection_inputs()
        .contract_inputs()
        .main_source_object_id();
    let main =
        derive_go_source_module_manifest(original.source(main_id).unwrap().clone(), true).unwrap();
    let mut observed = vec![];
    for phase in ["initial_summaries", "final_summaries"] {
        for row in graph[phase].as_array().unwrap() {
            if let Some(id) = row["manifest_source_object_id"].as_str() {
                let source = original.source(id).unwrap().clone();
                let parsed = derive_go_source_graph_manifest(source).unwrap();
                assert_eq!(
                    parsed.supports_graph_pruning(),
                    row["pruned"].as_bool().unwrap()
                );
                assert_eq!(
                    parsed.go_requirement().unwrap_or(""),
                    row["go_version"].as_str().unwrap()
                );
                let expected = graph_requirements(&parsed, &main, false);
                assert_eq!(expected, row["requirements"].as_array().unwrap().clone());
                observed.push(json!({"phase":phase,"module":row["module"],"original_object":parsed.original_source().object(),"facts":facts(&parsed),"graph_requirements":expected}));
            }
        }
    }
    let export: Value = serde_json::from_slice(modules.export_inputs().canonical_bytes()).unwrap();
    assert_eq!(export["packages"].as_array().unwrap().len(), 181);
    assert!(export["packages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["import_path"] == "net/http"));
    output(
        "complete-original-graph-owner-manifests.json",
        &json!({"status":"passed","observations":observed,"original_objects":original.source_objects().len(),"original_packages":181,"whole_source_or_package_universe_reduced":false,"graph_transport_import_or_independent_MVS_claimed":false}),
    );
}
