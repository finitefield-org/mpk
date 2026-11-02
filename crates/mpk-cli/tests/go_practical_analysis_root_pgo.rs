#![cfg(target_os = "linux")]

#[path = "../src/go_practical_analysis_root_binding.rs"]
mod go_practical_analysis_root_binding;
#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
mod go_practical_source_roots;

use go_practical_analysis_root_binding::bind_go_analysis_root_inputs;
use go_practical_source_roots::GoSourceRoots;
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs_for_pgo,
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoAnalysisPgoMode, GoAnalysisSourceError,
    GoAnalysisSourceInputs, GoSourceSnapshotInputs, GoSourceSnapshotLookupError,
    GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn inputs(
    directory: &Path,
) -> (
    Value,
    GoSourceSnapshotInputs,
    BTreeMap<String, GoTypeSourceBuffer>,
) {
    let packet: Value =
        serde_json::from_slice(&std::fs::read(directory.join("native.json")).unwrap()).unwrap();
    let mut sources = BTreeMap::new();
    for row in packet["original_sources"].as_array().unwrap() {
        let bytes = std::fs::read(
            directory
                .join("original-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &bytes)
                .unwrap();
        assert!(sources.insert(source.object().id.clone(), source).is_none());
    }
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
    (packet, snapshot, sources)
}
fn analysis(
    packet: &Value,
    snapshot: GoSourceSnapshotInputs,
    sources: &BTreeMap<String, GoTypeSourceBuffer>,
    mode: GoAnalysisPgoMode,
) -> Result<GoAnalysisSourceInputs, GoAnalysisSourceError> {
    let manifest = sources[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect();
    let vendor = check_go_analysis_vendor_snapshot(
        snapshot,
        parse_go_analysis_vendor_manifest(manifest).unwrap(),
        archives,
    )
    .unwrap();
    check_go_analysis_source_inputs_for_pgo(check_go_analysis_module_inputs(vendor).unwrap(), mode)
}
fn roots(frontend: &Path, archives: &Path) -> GoSourceRoots {
    GoSourceRoots::open(vec![
        ("analysis-frontend".into(), frontend.to_path_buf()),
        ("analysis-archives".into(), archives.to_path_buf()),
    ])
    .unwrap()
}

#[test]
fn root_matched_analysis_retains_the_real_default_profile() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let evidence = repo.join("develop/migrations/go-practical/t02-w01/verification");
    let before = evidence.join("analysis-pgo/native-before-fix");
    let after = evidence.join("analysis-pgo/native-mode-aware");
    let profile = std::fs::read(after.join("original-profile.raw")).unwrap();
    assert_eq!(
        profile,
        std::fs::read(before.join("original-profile.raw")).unwrap()
    );
    let temp = tempfile::tempdir().unwrap();
    let restored = temp.path().join("registered-original");
    let status = std::process::Command::new("python3")
        .arg("-B")
        .arg(evidence.join("registered-analysis/restore_registered.py"))
        .arg(&restored)
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "all 269 original source files and modes restore"
    );
    let frontend = restored.join("go-tools/go2vir");
    std::fs::write(frontend.join("default.pgo"), &profile).unwrap();
    let archives = repo.join("go-tools/go2vir/testdata/practical-analysis-cache");

    // The early snapshot matches observed root entries but did not capture the
    // profile bytes. The independent analysis importer must reject that gap.
    let (before_packet, before_snapshot, before_sources) = inputs(&before);
    assert_eq!(before_snapshot.source_objects().len(), 196);
    let before_roots = roots(&frontend, &archives)
        .compare(before_snapshot.clone())
        .unwrap();
    let off = analysis(
        &before_packet,
        before_snapshot.clone(),
        &before_sources,
        GoAnalysisPgoMode::Off,
    )
    .unwrap();
    let off_binding = bind_go_analysis_root_inputs(off, before_roots).unwrap();
    assert_eq!(off_binding.inputs().pgo_mode(), GoAnalysisPgoMode::Off);
    assert_eq!(off_binding.inputs().build_sources().len(), 184);
    assert!(off_binding.inputs().default_pgo().is_none());
    drop(off_binding);
    assert!(matches!(
        analysis(
            &before_packet,
            before_snapshot,
            &before_sources,
            GoAnalysisPgoMode::Auto
        ),
        Err(GoAnalysisSourceError::Snapshot(
            GoSourceSnapshotLookupError::NotCaptured
        ))
    ));
    let (packet, snapshot, sources) = inputs(&after);
    assert_eq!(snapshot.source_objects().len(), 197);
    let matched = roots(&frontend, &archives)
        .compare(snapshot.clone())
        .unwrap();
    let owner = analysis(&packet, snapshot, &sources, GoAnalysisPgoMode::Auto).unwrap();
    let accepted = bind_go_analysis_root_inputs(owner, matched).unwrap();
    assert_eq!(accepted.inputs().pgo_mode(), GoAnalysisPgoMode::Auto);
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 197);
    assert_eq!(accepted.inputs().build_sources().len(), 185);
    let captured = accepted
        .inputs()
        .default_pgo()
        .expect("actual Go profile retained");
    assert_eq!(captured.bytes(), profile);
    assert_eq!(captured.object().kind, "auxiliary");
    println!(
        "{}",
        serde_json::json!({"status":"passed","original_registered_objects":196,"complete_profile_objects":197,"complete_build_inputs":185,"real_profile_bytes":profile.len(),"uncaptured_profile_rejected":true,"matched_profile_bytes_equal":true,"test_only_restored_roots":true,"launcher_authority_claimed":false,"native_Linux_release_acceptance":false})
    );
}
