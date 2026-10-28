use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs,
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoTypeSourceBuffer,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn evidence(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
        .join(name)
        .join("native")
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn complete_frontend_build_input_list_matches_original_go_capture() {
    let original = evidence("analysis-namespace");
    let packet: Value =
        serde_json::from_slice(&std::fs::read(original.join("native.json")).unwrap()).unwrap();
    let mut sources = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let raw = std::fs::read(
            original
                .join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        assert!(sources.insert(source.object().id.clone(), source).is_none());
    }
    assert_eq!(sources.len(), 213);
    let dir = evidence("analysis-build-list");
    let native: Value =
        serde_json::from_slice(&std::fs::read(dir.join("native.json")).unwrap()).unwrap();
    assert_eq!(native["original_sources"], 213);
    assert_eq!(native["frontend_go_files"], 65);
    assert_eq!(native["vendor_files"], 133);
    assert_eq!(native["root_or_binary_authority_claimed"], false);
    let buffers = native["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(native["snapshot_hex"].as_str().unwrap()),
        &buffers,
    )
    .unwrap();
    let manifest = sources[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let vendor = check_go_analysis_vendor_snapshot(
        snapshot,
        parse_go_analysis_vendor_manifest(manifest).unwrap(),
        archives,
    )
    .unwrap();
    let module = check_go_analysis_module_inputs(vendor).unwrap();
    let owner = check_go_analysis_source_inputs(module).unwrap();
    let build = owner.build_sources();
    let expected = native["build_objects"].as_array().unwrap();
    assert_eq!(build.len(), 201);
    assert_eq!(expected.len(), 201);
    for (source, object) in build.iter().zip(expected) {
        assert_eq!(serde_json::to_value(source.object()).unwrap(), *object);
        assert_eq!(sources[&source.object().id].bytes(), source.bytes());
    }
    assert!(build
        .windows(2)
        .all(|pair| pair[0].object().path < pair[1].object().path));
    assert_eq!(owner.frontend_sources().len(), 65);
    assert_eq!(
        owner
            .module()
            .vendor()
            .correspondence()
            .supplied_files()
            .len(),
        133
    );
    assert_eq!(
        owner.module().vendor().snapshot().source_objects().len(),
        213
    );
    if let Some(path) = std::env::var_os("MPK_GO_ANALYSIS_BUILD_LIST_RUST_OUTPUT") {
        let result = json!({"status":"passed","build_objects":expected,"snapshot_sha256":owner.module().vendor().snapshot().snapshot_sha256(),"complete_original_snapshot_objects":213,"frontend_go_files":65,"vendor_files":133,"original_root_or_registered_binary_proved":false});
        std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}
