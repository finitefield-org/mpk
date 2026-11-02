use mpk_vc::go_practical_type_keys::{
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoAnalysisVendorSnapshotError, GoSourceSnapshotInputs,
    GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/go-practical/t02-w01/verification/analysis-namespace/native",
    )
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn originals() -> (Value, BTreeMap<String, GoTypeSourceBuffer>) {
    let root = evidence();
    let packet: Value =
        serde_json::from_slice(&std::fs::read(root.join("native.json")).unwrap()).unwrap();
    let mut sources = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let raw = std::fs::read(
            root.join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        assert!(sources.insert(source.object().id.clone(), source).is_none());
    }
    assert_eq!(sources.len(), 213);
    (packet, sources)
}

fn snapshot(
    case: &Value,
    originals: &BTreeMap<String, GoTypeSourceBuffer>,
) -> GoSourceSnapshotInputs {
    let sources = case["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| originals[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    import_go_source_snapshot_inputs(&unhex(case["snapshot_hex"].as_str().unwrap()), &sources)
        .unwrap()
}

fn selected_inputs(
    packet: &Value,
    sources: &BTreeMap<String, GoTypeSourceBuffer>,
) -> (GoTypeSourceBuffer, Vec<GoTypeSourceBuffer>) {
    let manifest = sources[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect();
    (manifest, archives)
}

#[test]
fn complete_observed_namespace_matches_native_go_and_distinguishes_unread_entries() {
    let (packet, sources) = originals();
    let (manifest, archives) = selected_inputs(&packet, &sources);
    assert_eq!(archives.len(), 3);
    assert_eq!(packet["authenticated_original_root_claimed"], false);
    let cases = packet["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let snapshot = snapshot(case, &sources);
        let wire: Value = serde_json::from_slice(snapshot.canonical_bytes()).unwrap();
        assert_eq!(snapshot.snapshot_sha256(), wire["snapshot_sha256"]);
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
        if let Ok(owner) = &result {
            assert_eq!(
                owner.correspondence().supplied_files().len(),
                case["files"].as_u64().unwrap() as usize
            );
            assert_eq!(owner.correspondence().manifest().package_paths().len(), 29);
            assert_eq!(owner.correspondence().differences().len(), 1);
            assert_eq!(
                owner
                    .correspondence()
                    .archives()
                    .values()
                    .map(|archive| archive.members().len())
                    .sum::<usize>(),
                1751
            );
            if name == "original-complete-snapshot" {
                assert_eq!(owner.snapshot().source_objects().len(), 213);
            }
            if name == "coherently-pruned-snapshot" {
                // The internally consistent snapshot cannot authenticate an original filesystem.
                assert_eq!(owner.correspondence().supplied_files().len(), 132);
            }
        } else {
            let expected = match name {
                "unread-file" | "unread-directory" | "unknown-unread-file" => {
                    GoSourceSnapshotLookupError::NotCaptured
                }
                "symbolic-link" => GoSourceSnapshotLookupError::Type,
                _ => panic!("unexpected native control: {name}"),
            };
            assert_eq!(
                result.unwrap_err(),
                GoAnalysisVendorSnapshotError::Snapshot(expected),
                "{name}"
            );
        }
    }
}

#[test]
fn manifest_and_archives_must_be_the_sources_in_the_same_snapshot() {
    let (packet, sources) = originals();
    let (manifest, archives) = selected_inputs(&packet, &sources);
    let original = snapshot(&packet["cases"][0], &sources);
    let changed = GoTypeSourceBuffer::new(
        &manifest.object().capability,
        &manifest.object().path,
        &manifest.object().kind,
        &[manifest.bytes(), b"\n"].concat(),
    )
    .unwrap();
    assert_eq!(
        check_go_analysis_vendor_snapshot(
            original.clone(),
            parse_go_analysis_vendor_manifest(changed).unwrap(),
            archives.clone()
        )
        .unwrap_err(),
        GoAnalysisVendorSnapshotError::Mismatch
    );
    let mut crossed = archives.clone();
    let mut raw = crossed[0].bytes().to_vec();
    raw[0] ^= 1;
    crossed[0] = GoTypeSourceBuffer::new(
        &crossed[0].object().capability,
        &crossed[0].object().path,
        &crossed[0].object().kind,
        &raw,
    )
    .unwrap();
    assert_eq!(
        check_go_analysis_vendor_snapshot(
            original,
            parse_go_analysis_vendor_manifest(manifest).unwrap(),
            crossed
        )
        .unwrap_err(),
        GoAnalysisVendorSnapshotError::Mismatch
    );
}
