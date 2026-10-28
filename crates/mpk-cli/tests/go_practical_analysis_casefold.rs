//! Match Go's default-PGO lookup on both original filesystem behaviors.
#![cfg(target_os = "linux")]

#[path = "../src/go_practical_analysis_root_binding.rs"]
mod go_practical_analysis_root_binding;
#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
mod go_practical_source_roots;

use go_practical_analysis_root_binding::{
    bind_go_analysis_root_inputs, GoAnalysisRootBindingError,
};
use go_practical_source_roots::GoSourceRoots;
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs_for_pgo,
    check_go_analysis_source_inputs_for_pgo_resolution, check_go_analysis_vendor_snapshot,
    import_go_source_snapshot_inputs, parse_go_analysis_vendor_manifest, GoAnalysisPgoMode,
    GoAnalysisSourceInputs, GoSourceSnapshotInputs, GoTypeSourceBuffer,
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

fn inputs(
    root: &Path,
) -> (
    Value,
    GoSourceSnapshotInputs,
    BTreeMap<String, GoTypeSourceBuffer>,
) {
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
    (packet, snapshot, sources)
}

fn analysis(
    packet: &Value,
    snapshot: GoSourceSnapshotInputs,
    sources: &BTreeMap<String, GoTypeSourceBuffer>,
    name: Option<&str>,
) -> GoAnalysisSourceInputs {
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
    let module = check_go_analysis_module_inputs(vendor).unwrap();
    match name {
        Some(name) => check_go_analysis_source_inputs_for_pgo_resolution(
            module,
            GoAnalysisPgoMode::Auto,
            Some(name),
        )
        .unwrap(),
        None => check_go_analysis_source_inputs_for_pgo(module, GoAnalysisPgoMode::Auto).unwrap(),
    }
}

fn restored(repo: &Path, directory: &Path, profile: &[u8]) -> PathBuf {
    let dest = directory.join("complete-registered-original");
    let status = std::process::Command::new("python3").arg("-B").arg(repo.join("develop/migrations/go-practical/t02-w01/verification/registered-analysis/restore_registered.py")).arg(&dest).current_dir(repo).status().unwrap();
    assert!(
        status.success(),
        "all 269 original source files and modes restore"
    );
    let frontend = dest.join("go-tools/go2vir");
    std::fs::write(frontend.join("DEFAULT.PGO"), profile).unwrap();
    frontend
}

fn roots(repo: &Path, frontend: &Path) -> GoSourceRoots {
    GoSourceRoots::open(vec![
        ("analysis-frontend".into(), frontend.to_path_buf()),
        (
            "analysis-archives".into(),
            repo.join("go-tools/go2vir/testdata/practical-analysis-cache"),
        ),
    ])
    .unwrap()
}

#[test]
fn held_roots_corroborate_actual_case_sensitive_and_folded_lookup() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let evidence =
        repo.join("develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold");
    let folded = evidence.join("folded-final");
    let sensitive = evidence.join("sensitive-final");
    let profile = std::fs::read(folded.join("original-profile.raw")).unwrap();
    assert_eq!(
        profile,
        std::fs::read(sensitive.join("original-profile.raw")).unwrap()
    );
    let (folded_packet, folded_snapshot, folded_sources) = inputs(&folded);
    let (sensitive_packet, sensitive_snapshot, sensitive_sources) = inputs(&sensitive);
    assert_eq!(folded_snapshot.source_objects().len(), 197);
    assert_eq!(sensitive_snapshot.source_objects().len(), 196);
    assert_eq!(folded_packet["go_lookup_resolves"], true);
    assert_eq!(sensitive_packet["go_lookup_resolves"], false);

    let sensitive_temp = tempfile::tempdir().unwrap();
    let sensitive_frontend = restored(&repo, sensitive_temp.path(), &profile);
    assert!(!sensitive_frontend.join("default.pgo").exists());
    let matched = roots(&repo, &sensitive_frontend)
        .compare(sensitive_snapshot.clone())
        .unwrap();
    let owner = analysis(
        &sensitive_packet,
        sensitive_snapshot.clone(),
        &sensitive_sources,
        None,
    );
    let accepted = bind_go_analysis_root_inputs(owner, matched).unwrap();
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 196);
    assert_eq!(accepted.inputs().build_sources().len(), 184);
    assert!(accepted.inputs().default_pgo().is_none());
    drop(accepted);

    let case_dir = std::env::var_os("MPK_GO_CASEFOLD_TEST_ROOT")
        .expect("original case-insensitive filesystem mount");
    let folded_temp = tempfile::Builder::new()
        .prefix("go-pgo-casefold-")
        .tempdir_in(case_dir)
        .unwrap();
    let folded_frontend = restored(&repo, folded_temp.path(), &profile);
    assert!(folded_frontend.join("default.pgo").exists());
    let matched = roots(&repo, &folded_frontend)
        .compare(folded_snapshot.clone())
        .unwrap();
    let owner = analysis(
        &folded_packet,
        folded_snapshot,
        &folded_sources,
        Some("DEFAULT.PGO"),
    );
    let accepted = bind_go_analysis_root_inputs(owner, matched).unwrap();
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 197);
    assert_eq!(accepted.inputs().build_sources().len(), 185);
    let actual = accepted.inputs().default_pgo().unwrap();
    assert_eq!(actual.object().path, "DEFAULT.PGO");
    assert_eq!(actual.bytes(), profile);
    drop(accepted);

    // The same names and original 196 source objects match the physical root,
    // but falsely claiming the resolved profile is absent must still reject.
    let matched = roots(&repo, &folded_frontend)
        .compare(sensitive_snapshot.clone())
        .unwrap();
    let owner = analysis(
        &sensitive_packet,
        sensitive_snapshot,
        &sensitive_sources,
        None,
    );
    assert!(matches!(
        bind_go_analysis_root_inputs(owner, matched),
        Err(GoAnalysisRootBindingError::PgoLookup)
    ));
    println!(
        "{}",
        serde_json::json!({"status":"passed","complete_original_objects":196,"casefold_objects":197,"exact_profile_bytes":profile.len(),"case_sensitive_ignored_entry_accepted":true,"casefold_profile_matched":true,"casefold_false_absence_rejected":true,"launcher_authority_claimed":false,"native_Linux_release_acceptance":false})
    );
}

#[test]
fn held_roots_reject_a_different_file_with_identical_profile_bytes() {
    use std::os::unix::fs::MetadataExt;
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let evidence = repo.join(
        "develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold/crossed-final",
    );
    let profile = std::fs::read(evidence.join("original-profile.raw")).unwrap();
    let (packet, snapshot, sources) = inputs(&evidence);
    assert_eq!(snapshot.source_objects().len(), 198);
    let case_dir = std::env::var_os("MPK_GO_CASEFOLD_TEST_ROOT")
        .expect("original case-insensitive filesystem mount");
    let temp = tempfile::Builder::new()
        .prefix("go-pgo-crossed-")
        .tempdir_in(case_dir)
        .unwrap();
    let frontend = restored(&repo, temp.path(), &profile);
    std::fs::write(frontend.join("other.pgo"), &profile).unwrap();
    assert_ne!(
        std::fs::metadata(frontend.join("default.pgo"))
            .unwrap()
            .ino(),
        std::fs::metadata(frontend.join("other.pgo")).unwrap().ino()
    );
    let matched = roots(&repo, &frontend).compare(snapshot.clone()).unwrap();
    let wrong = analysis(&packet, snapshot.clone(), &sources, Some("other.pgo"));
    assert_eq!(wrong.default_pgo().unwrap().bytes(), profile);
    assert!(matches!(
        bind_go_analysis_root_inputs(wrong, matched),
        Err(GoAnalysisRootBindingError::PgoLookup)
    ));
    let matched = roots(&repo, &frontend).compare(snapshot.clone()).unwrap();
    let correct = analysis(&packet, snapshot, &sources, Some("DEFAULT.PGO"));
    let accepted = bind_go_analysis_root_inputs(correct, matched).unwrap();
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 198);
    assert_eq!(accepted.inputs().build_sources().len(), 185);
    assert_eq!(
        accepted.inputs().default_pgo().unwrap().object().path,
        "DEFAULT.PGO"
    );
}
