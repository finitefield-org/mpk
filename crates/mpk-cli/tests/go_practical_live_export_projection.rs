//! Bind the complete live export observation to the independently imported original owner.
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
use go_practical_capture_binding::complete_capabilities_match;
use go_practical_installed_compiler::InstalledGoCompiler;
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

#[test]
fn live_export_projection_matches_complete_original_compiler_owner() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let evidence = temp.path().join("evidence");
    let restored = std::process::Command::new("python3")
        .arg("-B")
        .arg(repo.join("develop/migrations/go-practical/t02-w01/verification/export-root-live/restore_evidence.py"))
        .arg("live-original")
        .arg(&evidence)
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(
        restored.success(),
        "restore every original evidence file and mode"
    );
    let directory = evidence.join("originals/versioned-owned");
    let packet: Value = serde_json::from_slice(
        &std::fs::read(directory.join("complete-original-module-graph-inputs.json")).unwrap(),
    )
    .unwrap();
    let live:Value=serde_json::from_slice(&std::fs::read(repo.join("develop/migrations/go-practical/t02-w01/verification/export-root-live/live-original/coord/snapshot-ready.json")).unwrap()).unwrap();
    assert_eq!(live["copied_input_mutation_probe"], true);
    let observed = snapshot(&live, &directory);
    assert_eq!(
        observed.capabilities(),
        &["original-go-package-exports".to_owned()]
    );
    assert_eq!(observed.source_objects().len(), 181);
    let original_exports = exports(&packet, &directory);
    let full = original_exports.capture_inputs().snapshot_inputs();
    assert_eq!(full.source_objects().len(), 5697);
    assert!(full.contains_exact_capabilities(&observed));
    // This earlier live observation is an independent capture with the same
    // complete source namespace and bytes. Its original handles have closed;
    // this comparison does not claim a simultaneous held-root owner.
    let source_packet: Value = serde_json::from_slice(
        &std::fs::read(repo.join("develop/migrations/go-practical/t02-w01/verification/source-root-live/live-original/coord/snapshot-ready.json")).unwrap(),
    )
    .unwrap();
    let source_observed = snapshot(&source_packet, &directory);
    assert_eq!(source_observed.source_objects().len(), 5516);
    assert!(complete_capabilities_match(
        full,
        &source_observed,
        &observed
    ));
    assert!(!complete_capabilities_match(
        full,
        &observed,
        &source_observed
    ));
    assert!(!complete_capabilities_match(full, full, &observed));
    assert!(!complete_capabilities_match(
        full,
        &source_observed,
        &source_observed
    ));
    // Retain the complete original owner above. A separate coherent partial
    // observation must not become an owner when the SDK root is missing.
    let mut missing_sdk = source_packet.clone();
    let mut partial_wire: Value =
        serde_json::from_slice(&unhex(source_packet["snapshot_hex"].as_str().unwrap())).unwrap();
    partial_wire["capabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|capability| capability != "original-go-sdk");
    partial_wire["directories"]
        .as_array_mut()
        .unwrap()
        .retain(|directory| directory["capability"] != "original-go-sdk");
    partial_wire["objects"]
        .as_array_mut()
        .unwrap()
        .retain(|object| object["capability"] != "original-go-sdk");
    missing_sdk["snapshot_sources"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["object"]["capability"] != "original-go-sdk");
    finish_snapshot(&mut missing_sdk, &mut partial_wire);
    let partial_source = snapshot(&missing_sdk, &directory);
    assert!(!partial_source.source_objects().is_empty());
    assert!(!complete_capabilities_match(
        full,
        &partial_source,
        &observed
    ));
    assert_eq!(
        serde_json::from_slice::<Value>(original_exports.canonical_bytes()).unwrap()["packages"]
            .as_array()
            .unwrap()
            .len(),
        182
    );
    assert_eq!(packet["overlays"].as_array().unwrap().len(), 0);
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
    let files = rows
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
    let sdk = Arc::new(frontend_registry::test_snapshot(inventory, files).unwrap());
    let compiler = InstalledGoCompiler::test_original_snapshot(&registry, sdk).unwrap();
    let record = unhex(packet["compiler_inputs_hex"].as_str().unwrap());
    let owner = compiler
        .bind_original_inputs(&record, original_exports)
        .unwrap();
    assert_eq!(owner.compiler_inputs().canonical_bytes(), record);
    assert_eq!(owner.compiler_inputs().files().len(), 5577);
    assert_eq!(
        owner
            .compiler_inputs()
            .files()
            .iter()
            .map(|f| f.bytes().len())
            .sum::<usize>(),
        166354960
    );

    let mut changed = live.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(live["snapshot_hex"].as_str().unwrap())).unwrap();
    let mut object = wire["objects"].as_array().unwrap().first().unwrap().clone();
    let old_id = object["id"].as_str().unwrap().to_owned();
    let row = live["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["object"]["id"] == old_id)
        .unwrap();
    let mut altered = std::fs::read(
        directory
            .join("original-source-objects")
            .join(row["raw_file"].as_str().unwrap()),
    )
    .unwrap();
    assert!(!altered.is_empty());
    altered[0] ^= 1;
    object["raw_sha256"] = json!(format!("{:x}", Sha256::digest(&altered)));
    seal(&mut object, "id", "MPK-GO-SOURCE-OBJECT-1.0");
    replace_object(&mut changed, &mut wire, &old_id, object, Some(&altered));
    finish_snapshot(&mut changed, &mut wire);
    let altered_snapshot = snapshot(&changed, &directory);
    assert_eq!(altered_snapshot.source_objects().len(), 181);
    assert!(!complete_capabilities_match(
        owner
            .compiler_inputs()
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs(),
        &source_observed,
        &altered_snapshot
    ));
    assert!(!owner
        .compiler_inputs()
        .export_inputs()
        .capture_inputs()
        .snapshot_inputs()
        .contains_exact_capabilities(&altered_snapshot));

    let mut namespace = live.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(live["snapshot_hex"].as_str().unwrap())).unwrap();
    let entries = wire["directories"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|d| d["path"] == "")
        .unwrap()["entries"]
        .as_array_mut()
        .unwrap();
    let added = hex(b"mpk-unread-export-control");
    assert!(!entries.iter().any(|e| e["name_hex"] == added));
    entries.push(json!({"name_hex":added,"type_bits":0}));
    entries.sort_by(|a, b| {
        a["name_hex"]
            .as_str()
            .unwrap()
            .cmp(b["name_hex"].as_str().unwrap())
    });
    finish_snapshot(&mut namespace, &mut wire);
    let changed_namespace = snapshot(&namespace, &directory);
    assert_eq!(changed_namespace.source_objects().len(), 181);
    assert!(!complete_capabilities_match(
        owner
            .compiler_inputs()
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs(),
        &source_observed,
        &changed_namespace
    ));
    assert!(!owner
        .compiler_inputs()
        .export_inputs()
        .capture_inputs()
        .snapshot_inputs()
        .contains_exact_capabilities(&changed_namespace));
    if let Some(out) = std::env::var_os("MPK_GO_LIVE_EXPORT_BINDING_OUTPUT") {
        let root = PathBuf::from(out);
        std::fs::write(
            root.join("changed-export.json"),
            serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("changed-namespace.json"),
            serde_json::to_vec_pretty(&namespace).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("source-missing-sdk.json"),
            serde_json::to_vec_pretty(&missing_sdk).unwrap(),
        )
        .unwrap();
    }
    println!(
        "{}",
        json!({"status":"passed","live_export_objects":181,"complete_original_objects":5697,"compiler_files":5577,"compiler_bytes":166354960,"original_packages":182,"coherent_changed_export_rejected":true,"coherent_changed_namespace_rejected":true,"coherent_missing_source_rejected":true,"launcher_authority_claimed":false,"application_proof_claimed":false,"native_Linux_release_acceptance":false})
    );
}
