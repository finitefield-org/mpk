//! Verify binding against the complete retained Go capture and compiler bytes.
//! Restored test roots and the test-only SDK snapshot are not launcher authority.
#![cfg(target_os = "linux")]
#[path = "../src/frontend_registry.rs"]
#[allow(unused_imports)]
mod frontend_registry;
#[path = "../src/go_practical_capture_binding.rs"]
#[allow(dead_code)]
mod go_practical_capture_binding;
#[path = "../src/go_practical_installed_compiler.rs"]
#[allow(dead_code)]
mod go_practical_installed_compiler;
#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
mod go_practical_source_roots;

use go_practical_capture_binding::{
    bind_installed_go_root_matched_inputs, source_capabilities_match,
};
use go_practical_installed_compiler::{InstalledGoCompiler, InstalledGoCompilerError};
use go_practical_source_roots::{GoSourceRoots, GoSourceRootsMatch};
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::release_bundle_v1::{validate_successor_release_registry, GO_TOOLCHAIN_BUNDLE_ID};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|b| format!("{b:02x}")).collect()
}

fn buffers(packet: &Value, field: &str, directory: &Path) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let bytes = if let Some(raw) = row["raw_hex"].as_str() {
                unhex(raw)
            } else {
                let name = row["raw_file"].as_str().unwrap();
                assert!(
                    name.len() == 68
                        && name.ends_with(".raw")
                        && name[..64].bytes().all(|b| b.is_ascii_hexdigit())
                );
                std::fs::read(directory.join("original-source-objects").join(name)).unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &bytes)
                .unwrap()
        })
        .collect()
}

fn snapshot(packet: &Value, directory: &Path) -> GoSourceSnapshotInputs {
    import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(packet, "snapshot_sources", directory),
    )
    .unwrap()
}

fn exports(packet: &Value, directory: &Path) -> GoSourceExportInputs {
    let selection = import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(packet, "sources", directory),
        &buffers(packet, "overlays", directory),
    )
    .unwrap();
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection,
        snapshot(packet, directory),
    )
    .unwrap();
    import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap()
}

fn seal(value: &mut Value, field: &str, domain: &'static str) {
    value.as_object_mut().unwrap().remove(field);
    let strict = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value[field] = json!(hash_canonical_json(HashDomain::new(domain), &strict)
        .unwrap()
        .to_hex());
}

fn native_hash(value: &Value) -> String {
    let mut hash = Sha256::new();
    hash.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    fn number(hash: &mut Sha256, number: u64) {
        hash.update(number.to_le_bytes());
    }
    fn text(hash: &mut Sha256, bytes: &[u8]) {
        number(hash, bytes.len() as u64);
        hash.update(bytes);
    }
    let capabilities = value["capabilities"].as_array().unwrap();
    number(&mut hash, capabilities.len() as u64);
    for capability in capabilities {
        text(&mut hash, capability.as_str().unwrap().as_bytes());
    }
    let directories = value["directories"].as_array().unwrap();
    number(&mut hash, directories.len() as u64);
    for directory in directories {
        text(
            &mut hash,
            directory["capability"].as_str().unwrap().as_bytes(),
        );
        text(&mut hash, directory["path"].as_str().unwrap().as_bytes());
        let entries = directory["entries"].as_array().unwrap();
        number(&mut hash, entries.len() as u64);
        for entry in entries {
            text(&mut hash, &unhex(entry["name_hex"].as_str().unwrap()));
            number(&mut hash, entry["type_bits"].as_u64().unwrap());
        }
    }
    let objects = value["objects"].as_array().unwrap();
    number(&mut hash, objects.len() as u64);
    for object in objects {
        text(&mut hash, object["id"].as_str().unwrap().as_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn finish_snapshot(packet: &mut Value, wire: &mut Value) {
    wire["objects"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().unwrap().cmp(b["id"].as_str().unwrap()));
    wire["snapshot_sha256"] = json!(native_hash(wire));
    seal(
        wire,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0",
    );
    packet["snapshot_hex"] = json!(hex(&serde_json::to_vec(wire).unwrap()));
}

fn replace_object(
    packet: &mut Value,
    wire: &mut Value,
    old_id: &str,
    object: Value,
    raw: Option<&[u8]>,
) {
    let row = packet["snapshot_sources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["object"]["id"] == old_id)
        .unwrap();
    row["object"] = object.clone();
    if let Some(raw) = raw {
        row.as_object_mut().unwrap().remove("raw_file");
        row["raw_hex"] = json!(hex(raw));
    }
    *wire["objects"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|o| o["id"] == old_id)
        .unwrap() = object;
}

fn roots(base: &Path, capabilities: &[&str]) -> GoSourceRoots {
    GoSourceRoots::open(
        capabilities
            .iter()
            .map(|capability| {
                let name = match *capability {
                    "workspace/main" => "main",
                    "workspace/dep" => "dep",
                    "original-go-sdk" => "sdk",
                    _ => panic!("unexpected capability"),
                };
                (capability.to_string(), base.join("roots").join(name))
            })
            .collect(),
    )
    .unwrap()
}

fn matched(
    base: &Path,
    packet: &Value,
    directory: &Path,
    capabilities: &[&str],
) -> GoSourceRootsMatch {
    roots(base, capabilities)
        .compare(snapshot(packet, directory))
        .expect("the complete coherent capture must match its restored roots")
}

// Give a restored SDK writable directories before TempDir cleanup.
struct RestoredFixture {
    _temp: tempfile::TempDir,
    base: PathBuf,
}
impl Drop for RestoredFixture {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        fn writable(path: &Path) {
            if let Ok(metadata) = std::fs::symlink_metadata(path) {
                if metadata.file_type().is_dir() {
                    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.flatten() {
                            writable(&entry.path());
                        }
                    }
                }
            }
        }
        writable(&self.base);
    }
}

fn restore_fixture() -> RestoredFixture {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().join("fixture");
    let status = std::process::Command::new("python3").arg("-B")
        .arg(repo.join("develop/migrations/go-practical/t02-w01/verification/capture-binding/restore_fixture.py"))
        .arg(&base).current_dir(&repo).status().unwrap();
    assert!(
        status.success(),
        "restore all original evidence and source roots"
    );
    RestoredFixture { _temp: temp, base }
}

#[test]
fn complete_capture_binding_rejects_coherent_mismatches() {
    let fixture = restore_fixture();
    let base = &fixture.base;
    let info: Value =
        serde_json::from_slice(&std::fs::read(base.join("fixture.json")).unwrap()).unwrap();
    assert_eq!(info["original_application_entries"], 74);
    assert_eq!(info["registered_sdk_files"], 5577);
    assert_eq!(info["registered_sdk_bytes"], 166354960);
    assert_eq!(info["all_original_evidence_files"], 16880);
    let directory = base.join("saved/originals/versioned-owned");
    let packet: Value = serde_json::from_slice(
        &std::fs::read(directory.join("complete-original-module-graph-inputs.json")).unwrap(),
    )
    .unwrap();
    let live: Value = serde_json::from_slice(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-root-live/live-original/coord/snapshot-ready.json")).unwrap()).unwrap();
    let semantic = validate_semantic_profile_registry(
        include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision4,
    )
    .unwrap();
    let registry = validate_successor_release_registry(
        include_bytes!("../../../release/bundles/bundle-registry.json"),
        &semantic,
    )
    .unwrap();
    let inventory = &registry
        .registry()
        .toolchain_bundles
        .iter()
        .find(|b| b.bundle_id == GO_TOOLCHAIN_BUNDLE_ID)
        .unwrap()
        .inventory;
    let rows = packet["compiler_files"].as_array().unwrap();
    assert_eq!(rows.len(), 5577);
    let objects = rows
        .iter()
        .zip(&inventory.files)
        .map(|(row, entry)| {
            assert_eq!(row["entry"], serde_json::to_value(entry).unwrap());
            assert_eq!(row["raw_file"], format!("{}.raw", entry.sha256));
            (
                entry.path.clone(),
                std::fs::read(
                    directory
                        .join("original-compiler-files")
                        .join(row["raw_file"].as_str().unwrap()),
                )
                .unwrap(),
                entry.executable,
            )
        })
        .collect();
    let sdk = Arc::new(frontend_registry::test_snapshot(inventory, objects).unwrap());
    let original_exports = exports(&packet, &directory);
    let record = unhex(packet["compiler_inputs_hex"].as_str().unwrap());
    let capabilities = ["original-go-sdk", "workspace/dep", "workspace/main"];
    let original_roots = matched(base, &live, &directory, &capabilities);
    let full = original_exports.capture_inputs().snapshot_inputs();
    assert_eq!(full.source_objects().len(), 5697);
    assert_eq!(original_roots.snapshot().source_objects().len(), 5516);
    assert!(source_capabilities_match(full, original_roots.snapshot()));
    let accepted = bind_installed_go_root_matched_inputs(
        InstalledGoCompiler::test_original_snapshot(&registry, sdk.clone()).unwrap(),
        &record,
        original_exports.clone(),
        original_roots,
    )
    .expect("the whole compiler owner and the same three complete source roots bind");
    assert_eq!(accepted.release_registry(), &registry.release_identity());
    assert_eq!(accepted.compiler_inputs().canonical_bytes(), record);
    assert_eq!(accepted.compiler_inputs().files().len(), 5577);
    assert_eq!(
        accepted
            .compiler_inputs()
            .files()
            .iter()
            .map(|f| f.bytes().len())
            .sum::<usize>(),
        166354960
    );
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 5516);
    assert_eq!(
        serde_json::from_slice::<Value>(original_exports.canonical_bytes()).unwrap()["packages"]
            .as_array()
            .unwrap()
            .len(),
        182
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&unhex(packet["selection_hex"].as_str().unwrap())).unwrap()
            ["roots"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(packet["overlays"].as_array().unwrap().len(), 0);
    drop(accepted);

    // A changed application file and its source identity are coherently resealed.
    // Root comparison accepts that modified copy; the original full owner must not.
    let mut changed = live.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(live["snapshot_hex"].as_str().unwrap())).unwrap();
    let original = wire["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["capability"] == "workspace/main" && o["path"] == "policy.go")
        .unwrap()
        .clone();
    let old_id = original["id"].as_str().unwrap().to_owned();
    let path = base.join("roots/main/policy.go");
    let original_bytes = std::fs::read(&path).unwrap();
    let mut modified = original_bytes.clone();
    modified[0] ^= 1;
    std::fs::write(&path, &modified).unwrap();
    let mut object = original;
    object["raw_sha256"] = json!(format!("{:x}", Sha256::digest(&modified)));
    seal(&mut object, "id", "MPK-GO-SOURCE-OBJECT-1.0");
    replace_object(&mut changed, &mut wire, &old_id, object, Some(&modified));
    finish_snapshot(&mut changed, &mut wire);
    let changed_roots = matched(base, &changed, &directory, &capabilities);
    assert!(matches!(
        bind_installed_go_root_matched_inputs(
            InstalledGoCompiler::test_original_snapshot(&registry, sdk.clone()).unwrap(),
            &record,
            original_exports.clone(),
            changed_roots
        ),
        Err(InstalledGoCompilerError::Inputs)
    ));
    std::fs::write(&path, &original_bytes).unwrap();

    // Add a real unread name, reseal the complete namespace, and compare roots.
    let mut changed = live.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(live["snapshot_hex"].as_str().unwrap())).unwrap();
    let name = b"mpk-unread-control";
    let control = base
        .join("roots/main")
        .join(std::str::from_utf8(name).unwrap());
    std::fs::write(&control, b"independent extra entry").unwrap();
    let entries = wire["directories"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|d| d["capability"] == "workspace/main" && d["path"] == "")
        .unwrap()["entries"]
        .as_array_mut()
        .unwrap();
    entries.push(json!({"name_hex":hex(name), "type_bits":0}));
    entries.sort_by(|a, b| {
        a["name_hex"]
            .as_str()
            .unwrap()
            .cmp(b["name_hex"].as_str().unwrap())
    });
    finish_snapshot(&mut changed, &mut wire);
    let changed_roots = matched(base, &changed, &directory, &capabilities);
    assert!(matches!(
        bind_installed_go_root_matched_inputs(
            InstalledGoCompiler::test_original_snapshot(&registry, sdk.clone()).unwrap(),
            &record,
            original_exports.clone(),
            changed_roots
        ),
        Err(InstalledGoCompilerError::Inputs)
    ));
    std::fs::remove_file(control).unwrap();

    // A coherently resealed partial attack can match two roots, but cannot bind
    // while the original full capture also requires the application root.
    let mut changed = live.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(live["snapshot_hex"].as_str().unwrap())).unwrap();
    wire["capabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c != "workspace/main");
    wire["directories"]
        .as_array_mut()
        .unwrap()
        .retain(|d| d["capability"] != "workspace/main");
    wire["objects"]
        .as_array_mut()
        .unwrap()
        .retain(|o| o["capability"] != "workspace/main");
    changed["snapshot_sources"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["object"]["capability"] != "workspace/main");
    finish_snapshot(&mut changed, &mut wire);
    let partial = matched(
        base,
        &changed,
        &directory,
        &["original-go-sdk", "workspace/dep"],
    );
    assert!(matches!(
        bind_installed_go_root_matched_inputs(
            InstalledGoCompiler::test_original_snapshot(&registry, sdk).unwrap(),
            &record,
            original_exports.clone(),
            partial
        ),
        Err(InstalledGoCompilerError::Inputs)
    ));

    // Reclassifying an actual export object does not make an unmatched source
    // root exempt: all 5,697 original objects and their bytes are retained.
    let mut altered_full = packet.clone();
    let mut full_wire: Value =
        serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
    let mut object = full_wire["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["capability"] == "original-go-package-exports" && o["kind"] == "export_data")
        .unwrap()
        .clone();
    let old_id = object["id"].as_str().unwrap().to_owned();
    object["kind"] = json!("go_file");
    seal(&mut object, "id", "MPK-GO-SOURCE-OBJECT-1.0");
    replace_object(&mut altered_full, &mut full_wire, &old_id, object, None);
    finish_snapshot(&mut altered_full, &mut full_wire);
    let modified_full = snapshot(&altered_full, &directory);
    assert_eq!(modified_full.source_objects().len(), 5697);
    let original_roots = matched(base, &live, &directory, &capabilities);
    assert!(!source_capabilities_match(
        &modified_full,
        original_roots.snapshot()
    ));
    println!(
        "{}",
        json!({"status":"passed", "original_application_entries":74, "registered_sdk_files":5577, "registered_sdk_bytes":166354960, "all_original_evidence_files":16880, "matched_objects":5516, "complete_objects":5697, "original_packages":182, "root_selections":1, "contract_overlays":0, "coherently_resealed_source_rejected":true, "coherently_resealed_namespace_rejected":true, "missing_source_capability_rejected":true, "reclassified_unmatched_export_rejected":true, "test_only_restored_roots":true, "launcher_authority_claimed":false, "export_root_authenticated":false, "native_Linux_release_acceptance":false})
    );
}
