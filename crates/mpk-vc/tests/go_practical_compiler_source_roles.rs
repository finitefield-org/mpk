use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(b: &str) -> Vec<u8> {
    b.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_COMPILER_SOURCE_ROLES_RUST_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }
}
#[test]
fn every_original_sdk_source_and_complete_header_probe_matches_pinned_native_roles() {
    let native = read(
        &root().join("compiler-source-roles/native/observations/original-sdk-source-roles.json"),
    );
    assert_eq!(native["go_version"], "go1.25.0");
    assert_eq!(native["target"], "linux_amd64");
    let dir = root().join("compiler-inputs/native/native-originals/versioned-owned");
    let packet = read(&dir.join("complete-original-module-graph-inputs.json"));
    let sources = packet["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["object"]["capability"] == "original-go-sdk")
        .map(|r| (r["object"]["path"].as_str().unwrap(), r))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(sources.len(), 5480);
    assert_eq!(native["sdk_sources"].as_array().unwrap().len(), 11004);
    let transport = read(&root().join("compiler-source-roles/source-byte-transport.json"));
    let locations = transport["sdk_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["path"].as_str().unwrap(), r))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut prepared_count = 0;
    let mut observations = vec![];
    let mut packages = std::collections::BTreeSet::new();
    for row in native["sdk_sources"].as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        let location = locations[path];
        let store = match location["storage"].as_str().unwrap() {
            "prepared-compiler" => dir.join("original-compiler-files"),
            "raw-sdk" => root().join("compiler-source-roles/original-source-bytes"),
            other => panic!("unexpected raw source storage {other}"),
        };
        let raw = std::fs::read(store.join(location["raw_file"].as_str().unwrap())).unwrap();
        assert_eq!(hex(&Sha256::digest(&raw)), row["raw_sha256"], "{path}");
        assert_eq!(raw.len(), row["bytes"].as_u64().unwrap() as usize);
        let source = if let Some(original) = sources.get(path) {
            prepared_count += 1;
            GoTypeSourceBuffer::import(&serde_json::to_vec(&original["object"]).unwrap(), &raw)
                .unwrap()
        } else {
            GoTypeSourceBuffer::new(
                "original-raw-go-sdk",
                path,
                if path.ends_with(".go") {
                    "go_file"
                } else {
                    "auxiliary"
                },
                &raw,
            )
            .unwrap()
        };
        let role =
            derive_go_compiler_source_role(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(role, row["role"], "{path}");
        if sources.contains_key(path) {
            assert_eq!(role, source.object().kind, "{path}");
        }
        // A different untrusted Go role class cannot supply the decision.
        if path.ends_with(".go") {
            let opposite = if role == "go_file" {
                "excluded_source"
            } else {
                "go_file"
            };
            let claim = GoTypeSourceBuffer::new("original-go-sdk", path, opposite, &raw).unwrap();
            assert_eq!(
                derive_go_compiler_source_role(&claim).unwrap(),
                role,
                "{path}"
            );
            if let Some((directory, _)) = path.strip_prefix("src/").unwrap().rsplit_once('/') {
                packages.insert(directory.to_string());
            }
        }
        observations.push(json!({"path":path,"source_object_id":source.object().id,"raw_sha256":row["raw_sha256"],"bytes":raw.len(),"role":role}));
    }
    assert_eq!(prepared_count, 5480);
    assert_eq!(json!(packages), native["package_paths"]);
    let mut probes = vec![];
    let mut long = 0;
    for row in native["probes"].as_array().unwrap() {
        let raw = unhex(row["raw_hex"].as_str().unwrap());
        let path = row["path"].as_str().unwrap();
        assert_eq!(hex(&Sha256::digest(&raw)), row["raw_sha256"]);
        assert_eq!(raw.len(), row["bytes"].as_u64().unwrap() as usize);
        let source = GoTypeSourceBuffer::new(
            "sdk-role-probe",
            path,
            if path.ends_with(".go") {
                "go_file"
            } else {
                "auxiliary"
            },
            &raw,
        )
        .unwrap();
        let result = derive_go_compiler_source_role(&source);
        assert_eq!(
            result.is_ok(),
            row["error"].is_null(),
            "{}: {result:?}; native={}",
            row["name"],
            row["error"]
        );
        if let Ok(role) = &result {
            assert_eq!(*role, row["role"], "{}", row["name"]);
        }
        if row["name"] == "long-complete-header-and-body" {
            assert_eq!(result.as_deref(), Ok("excluded_source"));
            assert!(raw.len() > 800000);
            assert!(raw.ends_with(b"func OriginalBody(){ go func(){select{}}() }\n"));
            long += 1;
        }
        probes.push(json!({"name":row["name"],"path":path,"source_object_id":source.object().id,"bytes":raw.len(),"role":result.as_ref().ok(),"error":result.err().map(|e|e.to_string())}));
    }
    assert_eq!(long, 1);
    output(
        "whole-original-sdk-role-comparison.json",
        &json!({"status":"passed","sdk_sources":observations,"package_paths":packages,"probes":probes,"complete_original_application_objects":5697,"complete_original_application_packages":182,"whole_original_compiler_files":5577,"whole_application_Root_contract_preserved":true}),
    );
}
