use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn original_bytes(path: &Path) -> Vec<u8> {
    if !path.is_file() {
        static RESTORE: std::sync::Once = std::sync::Once::new();
        RESTORE.call_once(|| {
            let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            assert!(std::process::Command::new("python3")
                .arg("-B")
                .arg(evidence().join("snapshot-inputs/preserve-originals.py"))
                .arg("--restore")
                .current_dir(repo)
                .status()
                .unwrap()
                .success());
        });
    }
    std::fs::read(path).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_SOURCE_BUILD_OUTPUT") {
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
fn compare(row: &Value, raw: &[u8]) -> Value {
    let source =
        GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), raw).unwrap();
    let result = derive_go_source_build_match(&source);
    if row["diagnostic"].as_str().unwrap().is_empty() {
        let actual = result.unwrap_or_else(|e| panic!("{}: {e}", source.object().path));
        assert_eq!(
            actual.matches_target(),
            row["matched"].as_bool().unwrap(),
            "{}",
            source.object().path
        );
        assert_eq!(
            actual.is_binary_only(),
            row["binary_only"].as_bool().unwrap(),
            "{}",
            source.object().path
        );
        assert_eq!(
            actual.is_test_file(),
            source.object().path.ends_with("_test.go")
        );
        json!({"source_object_id":source.object().id,"matched":actual.matches_target(),"binary_only":actual.is_binary_only(),"test":actual.is_test_file(),"production_candidate":actual.is_production_candidate()})
    } else {
        let err = result.unwrap_err();
        assert_eq!(err.kind(), GoSourceBuildErrorKind::Constraint);
        json!({"source_object_id":source.object().id,"rejected":err.to_string()})
    }
}
#[test]
fn build_constraints_match_every_original_fixed_go_observation() {
    let root = evidence().join("build-constraints/native/native-originals/probe/build-corpus.json");
    let packet = read(&root);
    let mut rows = vec![];
    for row in packet["cases"].as_array().unwrap() {
        let mut actual = compare(row, &unhex(row["raw_hex"].as_str().unwrap()));
        actual["name"] = row["name"].clone();
        rows.push(actual);
    }
    assert!(rows.len() > 180);
    output(
        "native-case-comparison.json",
        &json!({"status":"passed","cases":rows,"fixed_target":"Go1.25.0/linux/amd64/GOAMD64=v1/default experiments","caller_supplied_tags":false,"full_import_cgo_or_context_authority":false}),
    );
}
#[test]
fn build_constraints_match_every_go_candidate_in_all_complete_original_captures() {
    let packet = read(&evidence().join(
        "build-constraints/native/native-originals/originals/original-build-candidates.json",
    ));
    let mut units = vec![];
    for unit in packet["units"].as_array().unwrap() {
        let original = evidence()
            .join("snapshot-inputs")
            .join(unit["packet"].as_str().unwrap());
        let original_packet = read(&original);
        assert_eq!(
            unit["complete_original_objects"].as_u64().unwrap(),
            original_packet["snapshot_sources"]
                .as_array()
                .unwrap()
                .len() as u64
        );
        let expected = original_packet["snapshot_sources"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                matches!(
                    r["object"]["kind"].as_str(),
                    Some("go_file" | "excluded_source")
                )
            })
            .map(|r| r["object"]["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let mut actual = std::collections::BTreeSet::new();
        let mut rows = vec![];
        for row in unit["candidates"].as_array().unwrap() {
            let raw = original_bytes(
                &original
                    .parent()
                    .unwrap()
                    .join("original-source-objects")
                    .join(row["raw_file"].as_str().unwrap()),
            );
            rows.push(compare(row, &raw));
            assert!(actual.insert(row["object"]["id"].as_str().unwrap()));
        }
        assert_eq!(expected, actual);
        units.push(json!({"unit":unit["unit"],"complete_original_objects":unit["complete_original_objects"],"all_original_go_candidates":rows}));
    }
    assert_eq!(units.len(), 3);
    output(
        "original-candidate-comparison.json",
        &json!({"status":"passed","units":units,"original_source_bodies_and_scope_unchanged":true}),
    );
}
