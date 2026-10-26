use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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
    if let Some(dir) = std::env::var_os("MPK_GO_GRAPH_INPUTS_OUTPUT") {
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
fn seal(value: &mut Value, domain: &'static str) {
    value.as_object_mut().unwrap().remove("content_sha256");
    let parsed = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value["content_sha256"] = json!(hash_canonical_json(HashDomain::new(domain), &parsed)
        .unwrap()
        .to_hex());
}
fn buffers(packet: &Value, field: &str, dir: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(raw) = r["raw_hex"].as_str() {
                unhex(raw)
            } else {
                std::fs::read(
                    dir.unwrap()
                        .join("original-source-objects")
                        .join(r["raw_file"].as_str().unwrap()),
                )
                .unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
fn import_data(packet: &Value, dir: &Path) -> GoSourceModuleData {
    let select = import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(packet, "sources", None),
        &buffers(packet, "overlays", None),
    )
    .unwrap();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(packet, "snapshot_sources", Some(dir)),
    )
    .unwrap();
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        select,
        snapshot,
    )
    .unwrap();
    let exports = import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap();
    import_go_source_module_observations(
        &unhex(packet["module_observations_hex"].as_str().unwrap()),
        exports,
    )
    .unwrap()
    .verify_offline_module_data()
    .unwrap()
}
fn original(versioned: bool) -> (Value, GoSourceModuleData, Value) {
    let dir = if versioned {
        root().join("graph-import/native/native-originals/versioned-owned")
    } else {
        root().join("graph-inputs/native/native-originals/graph-owned")
    };
    let packet = read(&dir.join("complete-original-module-graph-inputs.json"));
    let data = import_data(&packet, &dir);
    let wire = serde_json::from_slice(&unhex(packet["module_graph_inputs_hex"].as_str().unwrap()))
        .unwrap();
    (packet, data, wire)
}
#[test]
fn whole_original_local_graph_imports_non_utf8_versions_without_host_sources() {
    let (packet, data, wire) = original(false);
    let raw = unhex(packet["module_graph_inputs_hex"].as_str().unwrap());
    let input = import_go_source_module_graph_inputs(&raw, data).unwrap();
    assert_eq!(input.canonical_bytes(), raw);
    assert_eq!(
        input
            .module_data()
            .module_observations()
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs()
            .source_objects()
            .len(),
        5667
    );
    let mut suggestion = b"go1.25.0-original-".to_vec();
    suggestion.push(255);
    assert!(input
        .original_metadata()
        .windows(suggestion.len())
        .any(|b| b == suggestion));
    assert_eq!(input.initial_manifests().len(), 2);
    assert_eq!(input.final_manifests().len(), 2);
    assert_eq!(input.observed_selected_versions().len(), 3);
    let exports: Value = serde_json::from_slice(
        input
            .module_data()
            .module_observations()
            .export_inputs()
            .canonical_bytes(),
    )
    .unwrap();
    assert_eq!(exports["packages"].as_array().unwrap().len(), 181);
    assert!(exports["packages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["import_path"] == "net/http"));
    output(
        "whole-original-local-graph-import.json",
        &json!({"status":"passed","graph":wire,"initial_sources":input.initial_manifests().iter().map(|(m,f)|json!({"path":m.path,"version_hex":hex(&m.version),"source":f.original_source().object()})).collect::<Vec<_>>(),"original_objects":5667,"original_packages":181,"unchanged_original_source_universe":true,"independent_root_expansion_or_source_program_proof":false}),
    );
}
#[test]
fn whole_original_versioned_graph_keeps_lower_empty_cycles_and_pruning() {
    let (packet, data, _) = original(true);
    let input = import_go_source_module_graph_inputs(
        &unhex(packet["module_graph_inputs_hex"].as_str().unwrap()),
        data,
    )
    .unwrap();
    let old = GoSourceGraphModule {
        path: "example.com/c".into(),
        version: b"v1.0.0".to_vec(),
    };
    let leaf = GoSourceGraphModule {
        path: "example.com/leaf".into(),
        version: b"v1.0.0".to_vec(),
    };
    assert!(
        input.initial_manifests().contains_key(&old) && input.final_manifests().contains_key(&old)
    );
    assert!(input.final_manifests()[&leaf].requirements().is_empty());
    assert!(!input.final_manifests().contains_key(&GoSourceGraphModule {
        path: "example.com/q".into(),
        version: b"v1.0.0".to_vec()
    }));
    assert_eq!(
        input.observed_selected_versions()["example.com/c"],
        b"v1.1.0"
    );
    let exports: Value = serde_json::from_slice(
        input
            .module_data()
            .module_observations()
            .export_inputs()
            .canonical_bytes(),
    )
    .unwrap();
    assert!(exports["packages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["import_path"] == "net/http"));
    output(
        "whole-original-versioned-graph-import.json",
        &json!({"status":"passed","initial_sources":input.initial_manifests().iter().map(|(m,f)|json!({"path":m.path,"version_hex":hex(&m.version),"source":f.original_source().object()})).collect::<Vec<_>>(),"final_sources":input.final_manifests().iter().map(|(m,f)|json!({"path":m.path,"version_hex":hex(&m.version),"source":f.original_source().object()})).collect::<Vec<_>>(),"original_objects":packet["snapshot_sources"].as_array().unwrap().len(),"original_packages":exports["packages"].as_array().unwrap().len(),"metadata_hex":hex(input.original_metadata()),"diagnostics_hex":hex(input.original_diagnostics()),"independent_root_expansion_or_source_program_proof":false}),
    );
}
#[test]
fn coherently_resealed_graph_claims_cannot_replace_original_dependencies() {
    let (packet, data, original) = original(true);
    let original_count = packet["snapshot_sources"].as_array().unwrap().len();
    let mut observed = vec![];
    for case in [
        "schema",
        "missing-cache",
        "unknown-field",
        "module-link",
        "cache-path",
        "cache-capability",
        "cache-escape",
        "missing-empty-leaf",
        "changed-source",
        "replacement",
        "go-version",
        "pruning",
        "requirement-order",
        "deduplicate",
        "missing-implicit-go",
        "virtual-source",
        "changed-raw-edge",
        "disconnected-raw-edge",
        "truncated-raw-output",
        "missing-null",
    ] {
        let mut candidate = original.clone();
        let index = |value: &Value, path: &str, version: &[u8]| {
            value["final_summaries"]
                .as_array()
                .unwrap()
                .iter()
                .position(|r| {
                    r["module"]["path"] == path && r["module"]["version_hex"] == hex(version)
                })
                .unwrap()
        };
        match case {
            "schema" => {
                candidate["schema"] = json!("mpk.go.practical.source_module_graph_inputs.v1")
            }
            "missing-cache" => {
                candidate.as_object_mut().unwrap().remove("module_cache");
            }
            "unknown-field" => candidate["caller_graph"] = json!([]),
            "module-link" => candidate["source_module_observations_sha256"] = json!("0".repeat(64)),
            "cache-path" => candidate["module_cache"]["original_path"] = json!("/unobserved/cache"),
            "cache-capability" => candidate["module_cache"]["capability"] = json!("workspace/main"),
            "cache-escape" => candidate["module_cache"]["directory"] = json!(".."),
            "missing-empty-leaf" => {
                let at = index(&candidate, "example.com/leaf", b"v1.0.0");
                candidate["final_summaries"]
                    .as_array_mut()
                    .unwrap()
                    .remove(at);
            }
            "changed-source" => {
                let at = index(&candidate, "example.com/c", b"v1.0.0");
                candidate["final_summaries"][at]["manifest_source_object_id"] = candidate
                    ["final_summaries"][index(&candidate, "example.com/c", b"v1.1.0")]
                    ["manifest_source_object_id"]
                    .clone();
            }
            "replacement" => {
                let at = index(&candidate, "example.com/alias", b"v1.0.0");
                candidate["final_summaries"][at]["replacement_path_hex"] =
                    json!(hex(b"example.com/Other"));
            }
            "go-version" => {
                let at = index(&candidate, "example.com/c", b"v1.0.0");
                candidate["final_summaries"][at]["go_version"] = json!("1.25.0");
            }
            "pruning" => {
                let at = index(&candidate, "example.com/c", b"v1.0.0");
                candidate["final_summaries"][at]["pruned"] = json!(true);
            }
            "requirement-order" => {
                let at = index(&candidate, "example.com/b", b"v1.0.0");
                candidate["final_summaries"][at]["requirements"]
                    .as_array_mut()
                    .unwrap()
                    .reverse();
            }
            "deduplicate" => {
                let at = index(&candidate, "example.com/a", b"v1.0.0");
                candidate["final_summaries"][at]["requirements"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "missing-implicit-go" => {
                let at = index(&candidate, "example.com/c", b"v1.1.0");
                candidate["final_summaries"][at]["requirements"] = json!([]);
            }
            "virtual-source" => {
                let at = index(&candidate, "go", b"1.25.0");
                candidate["final_summaries"][at]["manifest_source_object_id"] =
                    candidate["final_summaries"][0]["manifest_source_object_id"].clone();
            }
            "changed-raw-edge" => {
                let raw = String::from_utf8(unhex(candidate["metadata_hex"].as_str().unwrap()))
                    .unwrap()
                    .replace(
                        "example.com/a@v1.0.0 example.com/c@v1.0.0\n",
                        "example.com/a@v1.0.0 example.com/c@v1.1.0\n",
                    );
                candidate["metadata_hex"] = json!(hex(raw.as_bytes()));
            }
            "disconnected-raw-edge" => {
                let mut raw = unhex(candidate["metadata_hex"].as_str().unwrap());
                raw.extend_from_slice(b"example.com/q@v1.0.0 go@1.22\n");
                candidate["metadata_hex"] = json!(hex(&raw));
            }
            "truncated-raw-output" => {
                let mut raw = unhex(candidate["metadata_hex"].as_str().unwrap());
                raw.pop();
                candidate["metadata_hex"] = json!(hex(&raw));
            }
            "missing-null" => {
                let at = index(&candidate, "go", b"1.25.0");
                candidate["final_summaries"][at]
                    .as_object_mut()
                    .unwrap()
                    .remove("manifest_source_object_id");
            }
            _ => unreachable!(),
        }
        seal(
            &mut candidate,
            "MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0",
        );
        let error = import_go_source_module_graph_inputs(
            &serde_json::to_vec(&candidate).unwrap(),
            data.clone(),
        )
        .expect_err(case);
        assert_eq!(
            data.module_observations()
                .export_inputs()
                .capture_inputs()
                .snapshot_inputs()
                .source_objects()
                .len(),
            original_count
        );
        observed.push(json!({"case":case,"complete_changed_graph":candidate,"error":error.to_string(),"original_object_count":original_count,"original_capture_and_Root_contract_unchanged":true}));
    }
    output(
        "coherent-original-graph-controls.json",
        &json!({"status":"passed","cases":observed}),
    );
}

fn native_hash(v: &Value) -> String {
    fn n(h: &mut Sha256, n: u64) {
        h.update(n.to_le_bytes())
    }
    fn t(h: &mut Sha256, b: &[u8]) {
        n(h, b.len() as u64);
        h.update(b)
    }
    let mut h = Sha256::new();
    h.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    let roots = v["capabilities"].as_array().unwrap();
    n(&mut h, roots.len() as u64);
    for r in roots {
        t(&mut h, r.as_str().unwrap().as_bytes())
    }
    let dirs = v["directories"].as_array().unwrap();
    n(&mut h, dirs.len() as u64);
    for d in dirs {
        t(&mut h, d["capability"].as_str().unwrap().as_bytes());
        t(&mut h, d["path"].as_str().unwrap().as_bytes());
        let es = d["entries"].as_array().unwrap();
        n(&mut h, es.len() as u64);
        for e in es {
            t(&mut h, &unhex(e["name_hex"].as_str().unwrap()));
            n(&mut h, e["type_bits"].as_u64().unwrap());
        }
    }
    let os = v["objects"].as_array().unwrap();
    n(&mut h, os.len() as u64);
    for o in os {
        t(&mut h, o["id"].as_str().unwrap().as_bytes())
    }
    format!("{:x}", h.finalize())
}

#[test]
fn coherently_changed_unselected_mod_bytes_still_require_original_go_sum() {
    let dir = root().join("graph-import/native/native-originals/versioned-owned");
    let mut packet = read(&dir.join("complete-original-module-graph-inputs.json"));
    let count = packet["snapshot_sources"].as_array().unwrap().len();
    let at = packet["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["object"]["path"] == "cache/download/example.com/c/@v/v1.0.0.mod")
        .unwrap();
    let row = packet["snapshot_sources"][at].clone();
    let mut raw = std::fs::read(
        dir.join("original-source-objects")
            .join(row["raw_file"].as_str().unwrap()),
    )
    .unwrap();
    let old_id = row["object"]["id"].as_str().unwrap().to_string();
    raw.extend_from_slice(b"\n// changed original graph-only manifest content\n");
    let mut object = row["object"].clone();
    object["bytes"] = json!(raw.len());
    object["raw_sha256"] = json!(format!("{:x}", Sha256::digest(&raw)));
    object.as_object_mut().unwrap().remove("id");
    let strict = parse_strict_json(
        &serde_json::to_vec(&object).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    let new_id = hash_canonical_json(HashDomain::new("MPK-GO-SOURCE-OBJECT-1.0"), &strict)
        .unwrap()
        .to_hex();
    object["id"] = json!(new_id);
    packet["snapshot_sources"][at]["object"] = object.clone();
    packet["snapshot_sources"][at]["raw_hex"] = json!(hex(&raw));
    packet["snapshot_sources"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["object"]["id"].as_str().cmp(&b["object"]["id"].as_str()));
    let mut snapshot: Value =
        serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
    for obj in snapshot["objects"].as_array_mut().unwrap() {
        if obj["id"] == old_id {
            *obj = object.clone()
        }
    }
    snapshot["objects"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    snapshot["snapshot_sha256"] = json!(native_hash(&snapshot));
    seal(&mut snapshot, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
    packet["snapshot_hex"] = json!(hex(&serde_json::to_vec(&snapshot).unwrap()));
    let mut capture: Value =
        serde_json::from_slice(&unhex(packet["capture_hex"].as_str().unwrap())).unwrap();
    capture["snapshot_inputs_sha256"] = snapshot["content_sha256"].clone();
    seal(&mut capture, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
    packet["capture_hex"] = json!(hex(&serde_json::to_vec(&capture).unwrap()));
    let mut export: Value =
        serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
    export["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
    seal(&mut export, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
    packet["export_inputs_hex"] = json!(hex(&serde_json::to_vec(&export).unwrap()));
    let mut modules: Value =
        serde_json::from_slice(&unhex(packet["module_observations_hex"].as_str().unwrap()))
            .unwrap();
    modules["source_export_inputs_sha256"] = export["content_sha256"].clone();
    seal(
        &mut modules,
        "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0",
    );
    packet["module_observations_hex"] = json!(hex(&serde_json::to_vec(&modules).unwrap()));
    let mut graph: Value =
        serde_json::from_slice(&unhex(packet["module_graph_inputs_hex"].as_str().unwrap()))
            .unwrap();
    graph["source_module_observations_sha256"] = modules["content_sha256"].clone();
    for phase in ["initial_summaries", "final_summaries"] {
        for row in graph[phase].as_array_mut().unwrap() {
            if row["manifest_source_object_id"] == old_id {
                row["manifest_source_object_id"] = json!(new_id)
            }
        }
    }
    seal(&mut graph, "MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0");
    let data = import_data(&packet, &dir);
    assert_eq!(
        data.module_observations()
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs()
            .source_objects()
            .len(),
        count
    );
    assert_eq!(
        data.module_observations()
            .export_inputs()
            .capture_inputs()
            .derive_contracts()
            .unwrap()
            .functions()
            .len(),
        1
    );
    let error = import_go_source_module_graph_inputs(&serde_json::to_vec(&graph).unwrap(), data)
        .expect_err("original graph-only go.mod h1 must still match");
    assert_eq!(error, GoSourceModuleGraphError::Checksum);
    output(
        "coherent-original-unselected-manifest-control.json",
        &json!({"status":"passed","changed_object":object,"complete_changed_raw_hex":hex(&raw),"changed_snapshot_hex":packet["snapshot_hex"],"changed_capture_hex":packet["capture_hex"],"changed_export_hex":packet["export_inputs_hex"],"changed_module_hex":packet["module_observations_hex"],"changed_graph":graph,"original_object_count":count,"same_original_package_source_and_Root_contract_universe":true,"error":error.to_string()}),
    );
}
