use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_vendor_snapshot,
    import_go_source_snapshot_inputs, parse_go_analysis_vendor_manifest, GoAnalysisModuleError,
    GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use serde_json::Value;
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
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn frontend_module_and_original_cached_checksums_match_native_go() {
    let previous = evidence("analysis-namespace");
    let packet: Value =
        serde_json::from_slice(&std::fs::read(previous.join("native.json")).unwrap()).unwrap();
    let mut originals = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let raw = std::fs::read(
            previous
                .join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        assert!(originals
            .insert(source.object().id.clone(), source)
            .is_none());
    }
    assert_eq!(originals.len(), 213);
    let vendor_manifest = originals[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| originals[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let root = evidence("analysis-module");
    let native: Value =
        serde_json::from_slice(&std::fs::read(root.join("native.json")).unwrap()).unwrap();
    assert_eq!(native["original_sources"], 213);
    assert_eq!(native["vendor_files"], 133);
    assert_eq!(native["info_or_root_authority_claimed"], false);
    let cases = native["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let mut sources = originals.clone();
        if let Some(changed) = case["changed"].as_object() {
            let raw = std::fs::read(
                root.join("changed-bytes")
                    .join(changed["raw_file"].as_str().unwrap()),
            )
            .unwrap();
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&changed["object"]).unwrap(), &raw)
                    .unwrap();
            sources.insert(source.object().id.clone(), source);
        }
        let buffers = case["source_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| sources[id.as_str().unwrap()].clone())
            .collect::<Vec<_>>();
        let snapshot = import_go_source_snapshot_inputs(
            &unhex(case["snapshot_hex"].as_str().unwrap()),
            &buffers,
        )
        .unwrap();
        let vendor = check_go_analysis_vendor_snapshot(
            snapshot,
            parse_go_analysis_vendor_manifest(vendor_manifest.clone()).unwrap(),
            archives.clone(),
        )
        .unwrap();
        let result = check_go_analysis_module_inputs(vendor);
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
        if let Ok(owner) = &result {
            assert_eq!(owner.vendor().correspondence().supplied_files().len(), 133);
            assert_eq!(owner.manifest().requirements().len(), 3);
            assert_eq!(owner.sums().entries().len(), 8);
            assert_eq!(owner.cached_manifests().len(), 3);
            assert_eq!(owner.zip_hashes().len(), 3);
            assert_eq!(owner.vendor().snapshot().source_objects().len(), 213);
        }
        if matches!(name, "unread-sums" | "unread-cached-mod") {
            assert_eq!(
                result.unwrap_err(),
                GoAnalysisModuleError::Snapshot(GoSourceSnapshotLookupError::NotCaptured),
                "{name}"
            );
        }
    }
}
