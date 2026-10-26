use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(p: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
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
fn snapshot(p: &Value, directory: Option<&Path>) -> GoSourceSnapshotInputs {
    let sources = p["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(h) = r["raw_hex"].as_str() {
                unhex(h)
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
        .collect::<Vec<_>>();
    import_go_source_snapshot_inputs(&unhex(p["snapshot_hex"].as_str().unwrap()), &sources).unwrap()
}
fn names(s: &[GoTypeSourceObject]) -> Vec<&str> {
    s.iter()
        .map(|s| s.path.rsplit('/').next().unwrap())
        .collect()
}
fn expected(v: &Value) -> Vec<&str> {
    v.as_array()
        .map(|a| a.iter().map(|s| s.as_str().unwrap()).collect())
        .unwrap_or_default()
}
fn output(name: &str, v: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_CGO_DIRECTIVES_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(v).unwrap())
            .unwrap();
    }
}
fn compare(snapshot: &GoSourceSnapshotInputs, row: &Value) -> Value {
    let cap = row["capability"].as_str().unwrap_or("original");
    let dir = row["directory"].as_str().unwrap_or("");
    let result = derive_go_source_package_files(snapshot, cap, dir);
    if !row["diagnostic"].as_str().unwrap().is_empty() {
        let error = result.unwrap_err();
        assert_eq!(error.kind(), GoSourcePackageErrorKind::CgoDirective);
        json!({"capability":cap,"directory":dir,"error":error.to_string()})
    } else {
        let result = result.unwrap_or_else(|e| panic!("{cap}/{dir}: {e}"));
        assert_eq!(result.package_name(), row["name"].as_str().unwrap());
        assert_eq!(names(result.active_sources()), expected(&row["go_files"]));
        assert_eq!(
            names(result.excluded_sources()),
            expected(&row["ignored_go_files"])
        );
        assert_eq!(
            names(result.test_sources()),
            expected(&row["test_go_files"])
        );
        assert_eq!(
            names(result.external_test_sources()),
            expected(&row["external_test_go_files"])
        );
        assert_eq!(result.imports(), expected(&row["imports"]));
        json!({"capability":cap,"directory":dir,"name":result.package_name(),"cgo_ignored":names(result.cgo_ignored_sources()),"active":names(result.active_sources()),"excluded":names(result.excluded_sources())})
    }
}
#[test]
fn cgo_directives_match_every_original_native_comment_and_argument_case() {
    let p = read(
        &root().join("cgo-directives/native/native-originals/headers/cgo-directive-corpus.json"),
    );
    let mut rows = vec![];
    for case in p["cases"].as_array().unwrap() {
        let snapshot = snapshot(case, None);
        let mut actual = compare(&snapshot, &case["expected"]);
        actual["case"] = case["name"].clone();
        rows.push(actual);
    }
    assert!(rows.len() > 80);
    output(
        "native-cgo-comparison.json",
        &json!({"status":"passed","cases":rows,"cgo_enabled":false,"compiler_or_build_hook_executed":false}),
    );
}
#[test]
fn cgo_directives_keep_every_original_package_and_captured_byte() {
    let corpus =
        read(&root().join(
            "cgo-directives/native/native-originals/originals/original-cgo-package-files.json",
        ));
    let mut units = vec![];
    for unit in corpus["units"].as_array().unwrap() {
        let path = root()
            .join("snapshot-inputs")
            .join(unit["packet"].as_str().unwrap());
        let p = read(&path);
        let snapshot = snapshot(&p, path.parent());
        let expected = snapshot
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
        let mut seen = std::collections::BTreeSet::new();
        let mut rows = vec![];
        for row in unit["packages"].as_array().unwrap() {
            assert!(seen.insert((
                row["capability"].as_str().unwrap(),
                row["directory"].as_str().unwrap()
            )));
            rows.push(compare(&snapshot, row));
        }
        assert_eq!(seen, expected);
        units.push(json!({"unit":unit["unit"],"complete_original_objects":snapshot.source_objects().len(),"packages":rows}));
    }
    assert_eq!(units.len(), 3);
    output(
        "original-cgo-universes.json",
        &json!({"status":"passed","units":units,"complete_original_scope_unchanged":true}),
    );
}
