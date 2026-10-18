//! Exact historical input access; no current-source proof or activation claim.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn manifest(epoch: &str) -> &'static Value {
    static PUBLISHED: OnceLock<Value> = OnceLock::new();
    static PRIVATE: OnceLock<Value> = OnceLock::new();
    static ENTRY: OnceLock<Value> = OnceLock::new();
    let (slot, commit) = match epoch {
        "published" => (&PUBLISHED, "16946c6ae2a82004be03f49ba52726001dd3317f"),
        "private" => (&PRIVATE, "40469d298f1d69558713d02ea0b553a1dd583bf5"),
        "entry" => (&ENTRY, "1c34ed5701df2e53fd515d49ad4d158cc728cb4b"),
        _ => panic!("unknown retained epoch"),
    };
    slot.get_or_init(|| {
        let path = root().join(format!(
            "develop/migrations/go-practical/t01-w05/retained-csharp/{epoch}-epoch.json"
        ));
        let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(
            value["schema"],
            "mpk.go_practical.retained_csharp_source_epoch.v0"
        );
        assert_eq!(value["epoch"], epoch);
        assert_eq!(value["commit"], commit);
        value
    })
}

fn read(epoch: &str, path: &str) -> Vec<u8> {
    let manifest = manifest(epoch);
    let row = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == path)
        .unwrap_or_else(|| panic!("unrecorded {epoch} input {path}"));
    let object = format!("{}:{path}", manifest["commit"].as_str().unwrap());
    let result = Command::new("/usr/bin/git")
        .args(["show", &object])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(result.status.success() && result.stderr.is_empty());
    assert_eq!(row["bytes"], result.stdout.len());
    assert_eq!(
        row["sha256"],
        format!("{:x}", Sha256::digest(&result.stdout))
    );
    result.stdout
}

pub fn published(path: &str) -> Vec<u8> {
    read("published", path)
}

pub fn private(path: &str) -> Vec<u8> {
    read("private", path)
}

pub fn published_document(path: &str) -> Value {
    serde_json::from_slice(&published(path)).unwrap()
}

pub fn replay(epoch: &str) {
    let result = Command::new("python3")
        .arg(root().join("develop/probes/go-practical/retained_csharp_epoch.py"))
        .arg(epoch)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["epoch"], epoch);
    assert_eq!(value["historical_inputs_changed"], false);
    assert_eq!(value["current_application_proofs"], false);
    assert_eq!(value["profile_activated"], false);
    assert_eq!(value["commit"], manifest(epoch)["commit"]);
}

pub fn assert_publication_retained_and_inactive() {
    for path in [
        "develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md",
        "develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md",
        "develop/specs/vectors/csharp-practical-profile-v1.json",
    ] {
        assert!(
            fs::read(root().join(path)).unwrap() == read("entry", path),
            "retained entry bytes changed for {path}"
        );
    }
    let vector = "develop/specs/vectors/csharp-practical-profile-v1.json";
    assert!(
        fs::read(root().join(vector)).unwrap() == published(vector),
        "original inactive CSharp vector payload changed"
    );
    for path in [
        "release/bundles/semantic-profile-registry.json",
        "release/bundles/bundle-registry.json",
    ] {
        let active = fs::read_to_string(root().join(path)).unwrap();
        assert!(!active.contains("mpk.csharp.practical"));
        assert!(!active.contains("csharp-practical"));
    }
}
