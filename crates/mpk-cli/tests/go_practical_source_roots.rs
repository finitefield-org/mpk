#![cfg(target_os = "linux")]

#[path = "../src/go_practical_source_roots.rs"]
mod go_practical_source_roots;

use go_practical_source_roots::{GoSourceRootError, GoSourceRoots};
use mpk_vc::go_practical_type_keys::{
    import_go_source_snapshot_inputs, GoSourceSnapshotInputs, GoTypeSourceBuffer,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn original_snapshot(repo: &Path) -> GoSourceSnapshotInputs {
    let evidence = repo.join("develop/migrations/go-practical/t02-w01/verification");
    let original = evidence.join("analysis-namespace/native");
    let packet: Value =
        serde_json::from_slice(&fs::read(original.join("native.json")).unwrap()).unwrap();
    let sources: BTreeMap<_, _> = packet["original_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let bytes = fs::read(
                original
                    .join("original-bytes")
                    .join(row["raw_file"].as_str().unwrap()),
            )
            .unwrap();
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &bytes)
                    .unwrap();
            (source.object().id.clone(), source)
        })
        .collect();
    assert_eq!(sources.len(), 213);
    let native: Value = serde_json::from_slice(
        &fs::read(evidence.join("analysis-build-list/native/native.json")).unwrap(),
    )
    .unwrap();
    let buffers: Vec<_> = native["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| sources[id.as_str().unwrap()].clone())
        .collect();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(native["snapshot_hex"].as_str().unwrap()),
        &buffers,
    )
    .unwrap();
    assert_eq!(snapshot.source_objects().len(), 213);
    assert_eq!(snapshot.observed_directories().count(), 52);
    assert_eq!(
        snapshot
            .observed_directories()
            .map(|(_, _, entries)| entries.len())
            .sum::<usize>(),
        373
    );
    snapshot
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()));
        } else {
            assert!(kind.is_file());
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

fn bindings(root: &Path) -> Vec<(String, PathBuf)> {
    vec![
        (
            "analysis-archives".into(),
            root.join("testdata/practical-analysis-cache"),
        ),
        ("analysis-frontend".into(), root.to_path_buf()),
    ]
}

#[test]
fn complete_original_analysis_snapshot_matches_opened_roots_and_rejects_changes() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let snapshot = original_snapshot(&repo);
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("original-frontend");
    copy_tree(&repo.join("go-tools/go2vir"), &root);

    let owner = GoSourceRoots::open(bindings(&root))
        .unwrap()
        .compare(snapshot.clone())
        .expect("all original source bytes and directory entries match the opened roots");
    assert_eq!(
        owner.snapshot().canonical_bytes(),
        snapshot.canonical_bytes()
    );
    drop(owner);

    let source = root.join("practical_analysis.go");
    let original = fs::read(&source).unwrap();
    let mut changed = original.clone();
    changed[0] ^= 1;
    fs::write(&source, &changed).unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings(&root))
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Source)
    ));
    fs::write(&source, &original).unwrap();

    let extra = root.join("unobserved-original-control");
    fs::write(&extra, b"control").unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings(&root))
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Namespace)
    ));
    fs::remove_file(&extra).unwrap();

    let detached = temp.path().join("detached-original-source");
    fs::rename(&source, &detached).unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings(&root))
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Namespace)
    ));
    symlink(&detached, &source).unwrap();
    assert!(matches!(
        GoSourceRoots::open(bindings(&root))
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Namespace)
    ));
    fs::remove_file(&source).unwrap();
    fs::rename(&detached, &source).unwrap();

    let roots = GoSourceRoots::open(bindings(&root)).unwrap();
    let previous_mode = fs::metadata(&source).unwrap().permissions().mode();
    assert!(matches!(
        roots.compare_with_hook(snapshot.clone(), || {
            fs::set_permissions(&source, fs::Permissions::from_mode(previous_mode ^ 0o100))
                .unwrap();
        }),
        Err(GoSourceRootError::Changed)
    ));
    fs::set_permissions(&source, fs::Permissions::from_mode(previous_mode)).unwrap();

    let mut missing = bindings(&root);
    missing.pop();
    assert!(matches!(
        GoSourceRoots::open(missing)
            .unwrap()
            .compare(snapshot.clone()),
        Err(GoSourceRootError::Capabilities)
    ));
    let mut duplicate = bindings(&root);
    duplicate.push(duplicate[0].clone());
    assert!(matches!(
        GoSourceRoots::open(duplicate),
        Err(GoSourceRootError::Capabilities)
    ));
    assert!(matches!(
        GoSourceRoots::open(vec![("analysis-frontend".into(), root.join("."))]),
        Err(GoSourceRootError::Path)
    ));
    assert_eq!(fs::read(&source).unwrap(), original);
    assert!(GoSourceRoots::open(bindings(&root))
        .unwrap()
        .compare(snapshot)
        .is_ok());
}
