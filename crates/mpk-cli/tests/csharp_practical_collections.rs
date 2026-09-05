//! CSHARP-03-T03-W07: array source ownership and frozen value limits.
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORK_ITEM: &str = "CSHARP-03-T03-W07";
const OWNER: &str = "crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W07";
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(path: &str) -> Vec<u8> {
    fs::read(root().join(path)).unwrap()
}
fn json_file(path: &str) -> Value {
    serde_json::from_slice(&read(path)).unwrap()
}

#[test]
fn csharp_03_t03_w07_exact_private_inputs() {
    let path = "develop/migrations/csharp-03/arrays/arrays-inputs.json";
    let manifest = json_file(path);
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    assert_eq!(manifest["work_item"], WORK_ITEM);
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w07.arrays_inputs.v1"
    );
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_arrays_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ];
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), expected.len());
    for (record, path) in files.iter().zip(expected) {
        let bytes = read(path);
        assert_eq!(record["path"], path);
        assert_eq!(record["size_bytes"], bytes.len());
        assert_eq!(record["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!String::from_utf8(read(path))
            .unwrap()
            .contains("PracticalArrays"));
    }
}

#[test]
fn csharp_03_t03_w07_frozen_array_limit_and_shared_equality() {
    let bundle = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap();
    let descriptor = json!({"kind":"instance","template":"bounded_sequence","arguments":[{"kind":"primitive","id":"i32"}]});
    let roots = canonical_closed_root_set_transport(
        &bundle,
        &json!([{"origin":"contract","provenance_id":"array.source","type":descriptor}]),
        &json!({}),
    )
    .unwrap();
    let roots = validate_closed_root_set(&bundle, &roots).unwrap();
    let closed = derive_closed_instances(&bundle, &roots).unwrap();
    let id = csharp_practical_closed_instance_id(&bundle, &descriptor).unwrap();
    let program = generate_structural_program(&bundle, &roots, &closed, &id).unwrap();
    let package = json_file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let vectors: Vec<_> = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == WORK_ITEM)
        .collect();
    assert_eq!(vectors.len(), 3);
    for vector in vectors {
        assert_eq!(vector["production_test_owner"], OWNER);
        let length = vector["inputs"]["value"].as_u64().unwrap() as usize;
        let value = MonomorphicValue::Array {
            type_id: id.clone(),
            elements: vec![
                MonomorphicValue::Signed {
                    type_id: "mpk.csharp.value.i32.v1".to_owned(),
                    value: "0".to_owned()
                };
                length
            ],
        };
        let result = validate_monomorphic_value(&bundle, &roots, &closed, &value);
        assert_eq!(
            result.is_ok(),
            vector["expected"]["accept"] == true,
            "{}",
            vector["id"]
        );
        if result.is_ok() {
            assert!(program.structural_equal(&value, &value).unwrap());
            assert_eq!(
                program.canonical_compare(&value, &value).unwrap(),
                std::cmp::Ordering::Equal
            );
        }
    }
}

#[test]
fn csharp_03_t03_w07_pinned_source_harness_when_available() {
    let package = json_file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    // Validate the manifest shape on every host before the Linux-only runner.
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache=root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            let suffix = match archive["kind"].as_str().unwrap() {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("{kind}"),
            };
            cache
                .join(format!("{}{suffix}", archive["id"].as_str().unwrap()))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned cache"
    );
    if count == 0 {
        return;
    }
    let output = Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg("--test-arrays")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[path = "support/csharp_practical_sequences.rs"]
mod sequences;
