//! Validate a real Go compiler profile without replacing any original source.
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs_for_pgo,
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, read_go_executable_build_info, GoAnalysisPgoMode,
    GoAnalysisSourceError, GoAnalysisSourceInputs, GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn evidence(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/analysis-pgo")
        .join(name)
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn analysis(
    directory: &Path,
    packet: &Value,
    mode: GoAnalysisPgoMode,
) -> Result<GoAnalysisSourceInputs, GoAnalysisSourceError> {
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
        .collect();
    let vendor = check_go_analysis_vendor_snapshot(
        snapshot,
        parse_go_analysis_vendor_manifest(manifest).unwrap(),
        archives,
    )
    .unwrap();
    check_go_analysis_source_inputs_for_pgo(check_go_analysis_module_inputs(vendor).unwrap(), mode)
}

#[test]
fn observed_but_uncaptured_default_profile_rejects() {
    let root = evidence("native-before-fix");
    let packet = read(&root.join("native.json"));
    assert_eq!(packet["original_sources"].as_array().unwrap().len(), 196);
    assert_eq!(packet["build_objects"].as_array().unwrap().len(), 184);
    assert_eq!(packet["native_auto_plan_uses_profile"], true);
    assert_eq!(packet["native_off_plan_uses_profile"], false);
    let off = analysis(&root, &packet, GoAnalysisPgoMode::Off).unwrap();
    assert_eq!(off.pgo_mode(), GoAnalysisPgoMode::Off);
    assert_eq!(off.build_sources().len(), 184);
    assert!(off.default_pgo().is_none());
    assert_eq!(off.module().vendor().snapshot().source_objects().len(), 196);
    let result = analysis(&root, &packet, GoAnalysisPgoMode::Auto);
    assert!(
        matches!(
            result,
            Err(GoAnalysisSourceError::Snapshot(
                GoSourceSnapshotLookupError::NotCaptured
            ))
        ),
        "an observed default profile without captured bytes must reject"
    );
}

#[test]
fn exact_default_profile_joins_every_original_build_input() {
    let root = evidence("native-mode-aware");
    let packet = read(&root.join("native.json"));
    assert_eq!(packet["original_sources"].as_array().unwrap().len(), 197);
    assert_eq!(packet["build_objects"].as_array().unwrap().len(), 185);
    assert_eq!(packet["native_auto_plan_uses_profile"], true);
    assert_eq!(packet["native_off_plan_uses_profile"], false);
    let owner = analysis(&root, &packet, GoAnalysisPgoMode::Auto)
        .expect("the full original source and exact PGO profile must be owned");
    assert_eq!(owner.pgo_mode(), GoAnalysisPgoMode::Auto);
    let build = owner.build_sources();
    assert_eq!(build.len(), 185);
    let profile = build
        .iter()
        .find(|source| source.object().path == "default.pgo")
        .expect("profile in original build inputs");
    assert_eq!(profile.object().kind, "auxiliary");
    assert_eq!(
        profile.object().raw_sha256,
        packet["profile_sha256"].as_str().unwrap()
    );
    assert_eq!(
        profile.bytes(),
        std::fs::read(root.join("original-profile.raw")).unwrap()
    );
    assert_eq!(
        owner.module().vendor().snapshot().source_objects().len(),
        197
    );
    let off = analysis(&root, &packet, GoAnalysisPgoMode::Off).unwrap();
    assert_eq!(off.pgo_mode(), GoAnalysisPgoMode::Off);
    assert!(off.default_pgo().is_none());
    assert_eq!(off.build_sources().len(), 184);
    assert_eq!(off.module().vendor().snapshot().source_objects().len(), 197);
    let prior = read(&evidence("native-before-fix").join("native.json"));
    let old_ids = prior["source_ids"].as_array().unwrap();
    let new_ids = packet["source_ids"].as_array().unwrap();
    assert_eq!(old_ids.len(), 196);
    assert!(
        old_ids.iter().all(|id| new_ids.contains(id)),
        "every original source object remains"
    );
}

#[test]
fn disabled_pgo_keeps_original_inputs_for_ignored_entries() {
    for name in ["directory", "symlink", "oversized"] {
        let root = evidence("native-mode-aware").join("controls").join(name);
        let packet = read(&root.join("native.json"));
        let off = analysis(&root, &packet, GoAnalysisPgoMode::Off).unwrap();
        assert_eq!(off.pgo_mode(), GoAnalysisPgoMode::Off, "{name}");
        assert_eq!(off.build_sources().len(), 184, "{name}");
        assert_eq!(
            off.module().vendor().snapshot().source_objects().len(),
            196,
            "{name}"
        );
        assert!(off.default_pgo().is_none(), "{name}");
        assert!(
            analysis(&root, &packet, GoAnalysisPgoMode::Auto).is_err(),
            "{name}"
        );
    }
}

#[test]
fn automatic_pgo_with_no_profile_keeps_original_build_inputs() {
    let root = evidence("../registered-analysis/native-fixed");
    let packet = read(&root.join("native.json"));
    let owner = analysis(&root, &packet, GoAnalysisPgoMode::Auto).unwrap();
    assert_eq!(owner.pgo_mode(), GoAnalysisPgoMode::Auto);
    assert!(owner.default_pgo().is_none());
    assert_eq!(owner.build_sources().len(), 184);
    assert_eq!(
        owner.module().vendor().snapshot().source_objects().len(),
        196
    );
}

#[test]
fn actual_profile_build_info_distinguishes_registered_frontend() {
    let root = evidence("actual-build-original");
    for (name, accepted) in [("auto", false), ("off", true)] {
        let raw = unhex(&std::fs::read_to_string(root.join(format!("{name}.elf.hex"))).unwrap());
        let info = read_go_executable_build_info(&raw).unwrap();
        assert_eq!(
            info.validate_analysis_frontend("linux_amd64").is_ok(),
            accepted
        );
    }
}
