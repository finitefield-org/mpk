use mpk_vc::go_practical_type_keys::{
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoAnalysisVendorArchiveError, GoAnalysisVendorSnapshotError,
    GoTypeSourceBuffer,
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

#[test]
fn shared_directory_and_archive_budget_matches_native_go_at_the_exact_limit() {
    let previous = evidence("analysis-namespace");
    let original: Value =
        serde_json::from_slice(&std::fs::read(previous.join("native.json")).unwrap()).unwrap();
    let mut sources = BTreeMap::new();
    for row in original["original_sources"].as_array().unwrap() {
        let bytes = std::fs::read(
            previous
                .join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &bytes)
                .unwrap();
        assert!(sources.insert(source.object().id.clone(), source).is_none());
    }
    assert_eq!(sources.len(), 213);
    let root = evidence("analysis-budget");
    let native: Value =
        serde_json::from_slice(&std::fs::read(root.join("native.json")).unwrap()).unwrap();
    assert_eq!(native["original_sources"], 213);
    assert_eq!(native["vendor_files"], 133);
    assert_eq!(native["original_root_authenticated"], false);
    let buffers = native["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let manifest = sources[original["manifest_source_id"].as_str().unwrap()].clone();
    let archives = original["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    let cases = native["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let directory_entries = case["directory_entries"].as_u64().unwrap();
        let archive_entries = case["archive_entries"].as_u64().unwrap();
        assert_eq!(archive_entries, 2403);
        if name == "exact-shared-limit" {
            assert_eq!(directory_entries + archive_entries, 131_072);
        }
        if name == "one-over-shared-limit" {
            assert_eq!(directory_entries + archive_entries, 131_073);
        }
        let wire = std::fs::read(root.join(case["snapshot_file"].as_str().unwrap())).unwrap();
        let snapshot = import_go_source_snapshot_inputs(&wire, &buffers).unwrap();
        let result = check_go_analysis_vendor_snapshot(
            snapshot,
            parse_go_analysis_vendor_manifest(manifest.clone()).unwrap(),
            archives.clone(),
        );
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
        if name == "one-over-shared-limit" {
            assert_eq!(
                result.unwrap_err(),
                GoAnalysisVendorSnapshotError::Archive(GoAnalysisVendorArchiveError::Limit)
            );
        } else {
            let owner = result.unwrap();
            assert_eq!(owner.snapshot().source_objects().len(), 213);
            assert_eq!(owner.correspondence().supplied_files().len(), 133);
            assert_eq!(
                owner
                    .correspondence()
                    .archives()
                    .values()
                    .map(|a| a.namespace_entries())
                    .sum::<usize>(),
                2403
            );
        }
    }
}
