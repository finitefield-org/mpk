use mpk_vc::go_practical_type_keys::{
    read_go_executable_build_info, GoExecutableBuildInfo, GoExecutableModule,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/analysis-build-info")
}

fn bytes(value: &Value, key: &str) -> Vec<u8> {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .as_bytes()
        .to_vec()
}

fn module(value: &Value) -> GoExecutableModule {
    assert!(value.is_object());
    GoExecutableModule {
        path: bytes(value, "Path"),
        version: bytes(value, "Version"),
        sum: bytes(value, "Sum"),
        replacement: value
            .get("Replace")
            .filter(|value| !value.is_null())
            .map(|value| Box::new(module(value))),
    }
}

fn info(value: &Value) -> Option<GoExecutableBuildInfo> {
    let dependencies = value["Deps"].as_array().expect("native dependency array");
    if dependencies.iter().any(Value::is_null) {
        return None;
    }
    Some(GoExecutableBuildInfo {
        go_version: bytes(value, "GoVersion"),
        path: bytes(value, "Path"),
        main: module(&value["Main"]),
        dependencies: dependencies.iter().map(module).collect(),
        settings: value["Settings"]
            .as_array()
            .expect("native settings array")
            .iter()
            .map(|setting| (bytes(setting, "Key"), bytes(setting, "Value")))
            .collect(),
    })
}

#[test]
fn original_frontend_and_complete_native_controls_match_go() {
    let root = evidence();
    let rows: Value =
        serde_json::from_slice(&std::fs::read(root.join("native/native.json")).unwrap()).unwrap();
    let rows = rows.as_array().unwrap();
    assert!(rows.len() >= 38);
    let raw = std::fs::read(root.join("registered-go2vir.elf")).unwrap();
    assert_eq!(raw.len(), 9_004_378);
    assert_eq!(
        format!("{:x}", Sha256::digest(&raw)),
        "ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538"
    );
    let actual = read_go_executable_build_info(&raw).expect("actual registered Go frontend");
    let mut names = BTreeSet::new();
    for row in rows {
        let name = row["name"].as_str().unwrap();
        assert!(names.insert(name));
        let expected = row["accepted"].as_bool().unwrap();
        let decoded = info(&row["info"]);
        if decoded.is_none() {
            assert_eq!(name, "nil-dependency");
            assert!(!expected);
            continue;
        }
        let decoded = decoded.unwrap();
        assert_eq!(
            decoded
                .validate_analysis_frontend(row["host"].as_str().unwrap())
                .is_ok(),
            expected,
            "{name}"
        );
        if name == "original-registered-frontend" {
            assert!(expected);
            assert_eq!(
                actual, decoded,
                "independent facts from the exact original executable"
            );
        }
    }
    assert!(names.contains("original-registered-frontend"));
    assert!(names.contains("valid-checksums-order-and-empty-flag"));
    assert!(names.contains("valid-darwin-host"));
}
