#![cfg(target_os = "linux")]

#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
mod go_practical_source_roots;

use go_practical_source_roots::{GoSourceRootError, GoSourceRoots};
use mpk_vc::go_practical_type_keys::{import_go_source_snapshot_inputs, GoTypeSourceBuffer};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn registered_frontend_capture_matches_all_restored_original_root_bytes() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let native = std::env::var_os("MPK_GO_REGISTERED_ANALYSIS_INPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join("develop/migrations/go-practical/t02-w01/verification/registered-analysis/native-fixed")
        });
    let packet: Value =
        serde_json::from_slice(&std::fs::read(native.join("native.json")).unwrap()).unwrap();
    let sources = packet["original_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let raw = std::fs::read(
                native
                    .join("original-bytes")
                    .join(row["raw_file"].as_str().unwrap()),
            )
            .unwrap();
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 196);
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &sources,
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let restored = temp.path().join("registered-original");
    let status = std::process::Command::new("python3")
        .arg("-B")
        .arg(repo.join("develop/migrations/go-practical/t02-w01/verification/registered-analysis/restore_registered.py"))
        .arg(&restored)
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(status.success(), "all 269 original files and modes restore");
    let frontend = restored.join("go-tools/go2vir");
    let archives = repo.join("go-tools/go2vir/testdata/practical-analysis-cache");
    let bindings = || {
        vec![
            ("analysis-frontend".to_owned(), frontend.clone()),
            ("analysis-archives".to_owned(), archives.clone()),
        ]
    };
    let matched = GoSourceRoots::open(bindings())
        .unwrap()
        .compare(snapshot.clone())
        .expect("the complete historical capture matches the independently opened roots");
    assert_eq!(matched.snapshot().source_objects().len(), 196);
    assert_eq!(
        matched.snapshot().canonical_bytes(),
        snapshot.canonical_bytes()
    );
    drop(matched);

    let file = frontend.join("preflight.go");
    let original = std::fs::read(&file).unwrap();
    let mut changed = original.clone();
    changed[0] ^= 1;
    std::fs::write(&file, changed).unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings())
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Source)
    ));
    std::fs::write(file, original).unwrap();

    let extra = frontend.join("mpk-unread-original-control");
    std::fs::write(&extra, b"extra entry").unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings()).unwrap().compare(snapshot),
        Err(GoSourceRootError::Namespace)
    ));
    std::fs::remove_file(extra).unwrap();
}
