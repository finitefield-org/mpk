use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_PACKAGE_FILES_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
fn original_bytes(path: &Path) -> Vec<u8> {
    if !path.is_file() {
        static RESTORE: std::sync::Once = std::sync::Once::new();
        RESTORE.call_once(|| {
            assert!(std::process::Command::new("python3")
                .arg("-B")
                .arg(root().join("snapshot-inputs/preserve-originals.py"))
                .arg("--restore")
                .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .status()
                .unwrap()
                .success());
        });
    }
    std::fs::read(path).unwrap()
}
fn sources(packet: &Value, directory: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    packet["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(hex) = r["raw_hex"].as_str() {
                unhex(hex)
            } else {
                original_bytes(
                    &directory
                        .unwrap()
                        .join("original-source-objects")
                        .join(r["raw_file"].as_str().unwrap()),
                )
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
fn snapshot(packet: &Value, directory: Option<&Path>) -> GoSourceSnapshotInputs {
    import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &sources(packet, directory),
    )
    .unwrap()
}
fn names(objects: &[GoTypeSourceObject]) -> Vec<String> {
    objects
        .iter()
        .map(|o| o.path.rsplit('/').next().unwrap().into())
        .collect()
}
fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|v| v.iter().map(|s| s.as_str().unwrap().into()).collect())
        .unwrap_or_default()
}
fn compare(snapshot: &GoSourceSnapshotInputs, row: &Value) -> Value {
    let capability = row["capability"].as_str().unwrap_or("original");
    let directory = row["directory"].as_str().unwrap_or("");
    let actual = derive_go_source_package_files(snapshot, capability, directory);
    if !row["diagnostic"].as_str().unwrap().is_empty() {
        let error = actual.unwrap_err();
        return json!({"capability":capability,"directory":directory,"rejected":error.to_string()});
    }
    let actual = actual.unwrap_or_else(|e| panic!("{capability}/{directory}: {e}"));
    assert_eq!(actual.package_name(), row["name"].as_str().unwrap());
    assert_eq!(names(actual.active_sources()), strings(&row["go_files"]));
    assert_eq!(
        names(actual.excluded_sources()),
        strings(&row["ignored_go_files"])
    );
    assert_eq!(names(actual.test_sources()), strings(&row["test_go_files"]));
    assert_eq!(
        names(actual.external_test_sources()),
        strings(&row["external_test_go_files"])
    );
    assert_eq!(actual.imports(), strings(&row["imports"]));
    assert_eq!(actual.test_imports(), strings(&row["test_imports"]));
    assert_eq!(
        actual.external_test_imports(),
        strings(&row["external_test_imports"])
    );
    assert_eq!(
        actual.is_binary_only(),
        row["binary_only"].as_bool().unwrap()
    );
    json!({"capability":capability,"directory":directory,"name":actual.package_name(),"active":names(actual.active_sources()),"excluded":names(actual.excluded_sources()),"tests":names(actual.test_sources()),"external_tests":names(actual.external_test_sources()),"cgo_ignored":names(actual.cgo_ignored_sources()),"imports":actual.imports(),"test_imports":actual.test_imports(),"external_test_imports":actual.external_test_imports(),"binary_only":actual.is_binary_only()})
}
#[test]
fn package_files_match_all_actual_original_header_observations() {
    let packet = read(
        &root()
            .join("package-files/native-final/native-originals/headers/package-header-corpus.json"),
    );
    let mut checked = vec![];
    for row in packet["cases"].as_array().unwrap() {
        let original = snapshot(row, None);
        let mut actual = compare(&original, &row["expected"]);
        actual["case"] = row["name"].clone();
        checked.push(actual);
        assert_eq!(
            original.source_objects().len(),
            row["snapshot_sources"].as_array().unwrap().len()
        );
    }
    assert!(checked.len() > 30);
    output(
        "header-comparison.json",
        &json!({"status":"passed","cases":checked,"complete_original_bodies_preserved":true,"source_or_context_proof":false}),
    );
}
#[test]
fn package_files_match_every_directory_in_all_complete_original_universes() {
    let corpus = read(
        &root().join("package-files/native/native-originals/originals/original-package-files.json"),
    );
    let mut units = vec![];
    for unit in corpus["units"].as_array().unwrap() {
        let path = root()
            .join("snapshot-inputs")
            .join(unit["packet"].as_str().unwrap());
        let packet = read(&path);
        let original = snapshot(&packet, path.parent());
        assert_eq!(
            original.source_objects().len() as u64,
            unit["complete_original_objects"].as_u64().unwrap()
        );
        let expected = original
            .source_objects()
            .iter()
            .filter(|s| s.kind == "go_file")
            .map(|s| {
                (
                    s.capability.as_str(),
                    s.path.rsplit_once('/').map_or("", |(d, _)| d),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        let mut locations = std::collections::BTreeSet::new();
        let mut rows = vec![];
        for row in unit["packages"].as_array().unwrap() {
            assert!(locations.insert((
                row["capability"].as_str().unwrap(),
                row["directory"].as_str().unwrap()
            )));
            rows.push(compare(&original, row));
        }
        assert_eq!(locations, expected);
        units.push(json!({"unit":unit["unit"],"complete_original_objects":original.source_objects().len(),"packages":rows}));
    }
    assert_eq!(units.len(), 3);
    output(
        "original-universe-comparison.json",
        &json!({"status":"passed","units":units,"original_scope_not_reduced":true}),
    );
}
