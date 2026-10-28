use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_vendor_snapshot,
    import_go_source_snapshot_inputs, parse_go_analysis_vendor_manifest,
    validate_go_analysis_module_info, GoAnalysisModuleError, GoAnalysisModuleInfoError,
    GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
        .join(name)
        .join("native")
}
fn source(row: &Value, folder: &Path) -> GoTypeSourceBuffer {
    let raw = std::fs::read(folder.join(row["raw_file"].as_str().unwrap())).unwrap();
    GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap()
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn every_native_info_observation_matches_pinned_go() {
    let dir = root("analysis-info");
    let native: Value =
        serde_json::from_slice(&std::fs::read(dir.join("native.json")).unwrap()).unwrap();
    let observations = native["observations"].as_array().unwrap();
    assert!(observations.len() >= 30);
    for case in observations {
        let name = case["name"].as_str().unwrap();
        let input = source(&case["source"], &dir.join("observed-bytes"));
        let result = validate_go_analysis_module_info(&input, case["version"].as_str().unwrap());
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
    }
    assert_eq!(native["original_sources"], 213);
    assert_eq!(native["vendor_files"], 133);
    assert_eq!(native["root_or_vcs_authority_claimed"], false);
}

#[test]
fn original_module_owner_checks_every_same_snapshot_info() {
    let previous = root("analysis-namespace");
    let packet: Value =
        serde_json::from_slice(&std::fs::read(previous.join("native.json")).unwrap()).unwrap();
    let mut originals = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let input = source(row, &previous.join("original-bytes"));
        assert!(originals.insert(input.object().id.clone(), input).is_none());
    }
    assert_eq!(originals.len(), 213);
    let manifest = originals[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| originals[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let dir = root("analysis-info");
    let native: Value =
        serde_json::from_slice(&std::fs::read(dir.join("native.json")).unwrap()).unwrap();
    let cases = native["snapshot_controls"].as_array().unwrap();
    assert_eq!(cases.len(), 5);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let mut sources = originals.clone();
        if case["changed"].is_object() {
            let input = source(&case["changed"], &dir.join("observed-bytes"));
            sources.insert(input.object().id.clone(), input);
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
            parse_go_analysis_vendor_manifest(manifest.clone()).unwrap(),
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
            assert_eq!(owner.cached_infos().len(), 3);
            assert_eq!(owner.vendor().snapshot().source_objects().len(), 213);
            assert_eq!(owner.vendor().correspondence().supplied_files().len(), 133);
        }
        let expected = match name {
            "wrong-version" => Some(GoAnalysisModuleError::Info(
                GoAnalysisModuleInfoError::Version,
            )),
            "escaped-time" => Some(GoAnalysisModuleError::Info(GoAnalysisModuleInfoError::Time)),
            "unread-info" => Some(GoAnalysisModuleError::Snapshot(
                GoSourceSnapshotLookupError::NotCaptured,
            )),
            _ => None,
        };
        if let Some(expected) = expected {
            assert_eq!(result.unwrap_err(), expected, "{name}");
        }
    }
}
