//! Independently derive the complete build inputs from the retained registered
//! Go frontend source epoch and its full native source/namespace capture.
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs,
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn registered_frontend_source_capture_has_the_exact_original_build_inputs() {
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/registered-analysis/native-fixed");
    let directory = std::env::var_os("MPK_GO_REGISTERED_ANALYSIS_INPUT")
        .map(PathBuf::from)
        .unwrap_or(fallback);
    let packet: Value =
        serde_json::from_slice(&std::fs::read(directory.join("native.json")).unwrap()).unwrap();
    assert_eq!(packet["original_registered_tree_files"], 242);
    assert_eq!(packet["original_full_source_epoch_files"], 269);
    assert_eq!(packet["original_frontend_go_files"], 48);
    assert_eq!(packet["original_vendor_files"], 133);
    assert_eq!(packet["registered_source_binary_proved"], false);
    assert_eq!(packet["launcher_authority_claimed"], false);
    let mut sources = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let raw = std::fs::read(
            directory
                .join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        assert!(sources.insert(source.object().id.clone(), source).is_none());
    }
    assert_eq!(sources.len(), 196);
    let buffers = packet["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
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
    let expected = packet["build_objects"].as_array().unwrap();
    assert_eq!(build.len(), 184);
    assert_eq!(expected.len(), 184);
    for (source, object) in build.iter().zip(expected) {
        assert_eq!(serde_json::to_value(source.object()).unwrap(), *object);
        assert_eq!(sources[&source.object().id].bytes(), source.bytes());
    }
    assert!(build
        .windows(2)
        .all(|pair| pair[0].object().path < pair[1].object().path));
    assert_eq!(owner.frontend_sources().len(), 48);
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
        196
    );
}
