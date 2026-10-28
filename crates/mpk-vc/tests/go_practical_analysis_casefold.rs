//! Inspect untrusted native PGO resolution while retaining all original inputs.
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs_for_pgo,
    check_go_analysis_source_inputs_for_pgo_resolution, check_go_analysis_vendor_snapshot,
    import_go_source_snapshot_inputs, parse_go_analysis_vendor_manifest, GoAnalysisModuleInputs,
    GoAnalysisPgoMode, GoAnalysisSourceError, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn evidence(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold")
        .join(name)
}
fn module(root: &Path) -> GoAnalysisModuleInputs {
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
    check_go_analysis_module_inputs(vendor).unwrap()
}

#[test]
fn original_profile_name_requires_explicit_untrusted_resolution() {
    let root = evidence("folded-final");
    let original = module(&root);
    assert_eq!(original.vendor().snapshot().source_objects().len(), 197);
    let owner = check_go_analysis_source_inputs_for_pgo_resolution(
        original.clone(),
        GoAnalysisPgoMode::Auto,
        Some("DEFAULT.PGO"),
    )
    .unwrap();
    assert_eq!(
        owner.pgo_lookup(),
        ("analysis-frontend", "default.pgo".to_owned())
    );
    assert_eq!(owner.build_sources().len(), 185);
    assert_eq!(owner.default_pgo().unwrap().object().path, "DEFAULT.PGO");
    assert_eq!(
        owner.default_pgo().unwrap().bytes(),
        std::fs::read(root.join("original-profile.raw")).unwrap()
    );
    let uncorroborated =
        check_go_analysis_source_inputs_for_pgo(original.clone(), GoAnalysisPgoMode::Auto).unwrap();
    assert!(uncorroborated.default_pgo().is_none());
    assert_eq!(uncorroborated.build_sources().len(), 184);
    assert!(matches!(
        check_go_analysis_source_inputs_for_pgo_resolution(
            original.clone(),
            GoAnalysisPgoMode::Auto,
            Some("not-observed.pgo")
        ),
        Err(GoAnalysisSourceError::Namespace)
    ));
    assert!(matches!(
        check_go_analysis_source_inputs_for_pgo_resolution(
            original,
            GoAnalysisPgoMode::Off,
            Some("DEFAULT.PGO")
        ),
        Err(GoAnalysisSourceError::Namespace)
    ));
}

#[test]
fn case_sensitive_ignored_entry_preserves_complete_original_inputs() {
    let original = module(&evidence("sensitive-final"));
    assert_eq!(original.vendor().snapshot().source_objects().len(), 196);
    let owner = check_go_analysis_source_inputs_for_pgo(original, GoAnalysisPgoMode::Auto).unwrap();
    assert!(owner.default_pgo().is_none());
    assert_eq!(owner.build_sources().len(), 184);
}
