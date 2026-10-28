//! Exercise the unchanged private boundary from an actual installed bin/mpk.
//! The outer driver materializes every registered Go/Rust bundle byte.
#[path = "../src/frontend_registry.rs"]
#[allow(unused_imports)]
mod frontend_registry;
#[path = "../src/go_practical_installed_compiler.rs"]
#[allow(dead_code)]
mod go_practical_installed_compiler;

use go_practical_installed_compiler::{InstalledGoCompiler, InstalledGoCompilerError};
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::release_bundle_v1::{validate_successor_release_registry, GO_TOOLCHAIN_BUNDLE_ID};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}

fn buffers(packet: &Value, field: &str, directory: &Path) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let raw = if let Some(raw) = row["raw_hex"].as_str() {
                unhex(raw)
            } else {
                let file = row["raw_file"].as_str().unwrap();
                assert!(
                    file.len() == 68
                        && file.ends_with(".raw")
                        && file[..64].bytes().all(|b| b.is_ascii_hexdigit())
                );
                std::fs::read(directory.join("original-source-objects").join(file)).unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
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
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(packet, "snapshot_sources", directory),
    )
    .unwrap();
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection,
        snapshot,
    )
    .unwrap();
    import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap()
}

// This checks the complete actual registered SDK bytes using the test-only
// snapshot sealer. It does not exercise or impersonate the installed root.
fn whole_snapshot_binding(linux: &Path, darwin: &Path) {
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
        .find(|bundle| bundle.bundle_id == GO_TOOLCHAIN_BUNDLE_ID)
        .unwrap()
        .inventory;
    let packet: Value = serde_json::from_slice(
        &std::fs::read(linux.join("complete-original-module-graph-inputs.json")).unwrap(),
    )
    .unwrap();
    let rows = packet["compiler_files"].as_array().unwrap();
    assert_eq!(rows.len(), inventory.files.len());
    let objects = rows
        .iter()
        .zip(&inventory.files)
        .map(|(row, entry)| {
            assert_eq!(row["entry"], serde_json::to_value(entry).unwrap());
            let raw_file = row["raw_file"].as_str().unwrap();
            assert_eq!(raw_file, format!("{}.raw", entry.sha256));
            (
                entry.path.clone(),
                std::fs::read(linux.join("original-compiler-files").join(raw_file)).unwrap(),
                entry.executable,
            )
        })
        .collect();
    let snapshot = Arc::new(frontend_registry::test_snapshot(inventory, objects).unwrap());
    let original_exports = exports(&packet, linux);
    let record = unhex(packet["compiler_inputs_hex"].as_str().unwrap());
    let accepted = InstalledGoCompiler::test_original_snapshot(&registry, snapshot.clone())
        .unwrap()
        .bind_original_inputs(&record, original_exports.clone())
        .expect("complete registered Linux SDK and original export inputs bind");
    let compiler = accepted.compiler_inputs();
    assert_eq!(compiler.canonical_bytes(), record);
    assert_eq!(compiler.inventory(), inventory);
    assert_eq!(compiler.host(), "linux_amd64");
    assert_eq!(compiler.files().len(), 5577);
    assert_eq!(
        compiler
            .files()
            .iter()
            .map(|file| file.bytes().len())
            .sum::<usize>(),
        166354960
    );
    assert_eq!(compiler.sdk_source_files().len(), 5480);
    assert_eq!(packet["snapshot_sources"].as_array().unwrap().len(), 5697);
    assert_eq!(
        serde_json::from_slice::<Value>(original_exports.canonical_bytes()).unwrap()["packages"]
            .as_array()
            .unwrap()
            .len(),
        182
    );
    assert_eq!(accepted.release_registry(), &registry.release_identity());
    let source_directories = compiler.source_index().package_paths().len();
    drop(accepted);

    // Rehash a conflicting host claim, preserving the complete Linux record,
    // SDK snapshot, package graph, all original sources and the Root selection.
    let mut altered: Value = serde_json::from_slice(&record).unwrap();
    altered["host"] = json!("darwin_arm64");
    altered.as_object_mut().unwrap().remove("content_sha256");
    let strict = parse_strict_json(
        &serde_json::to_vec(&altered).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    altered["content_sha256"] = json!(hash_canonical_json(
        HashDomain::new("MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0"),
        &strict,
    )
    .unwrap()
    .to_hex());
    let changed_host = InstalledGoCompiler::test_original_snapshot(&registry, snapshot.clone())
        .unwrap()
        .bind_original_inputs(&serde_json::to_vec(&altered).unwrap(), original_exports);
    assert!(matches!(
        changed_host,
        Err(InstalledGoCompilerError::Inputs)
    ));

    // A separately captured whole Darwin owner cannot be paired with the
    // registered Linux snapshot, even when its own records/exports agree.
    let other: Value = serde_json::from_slice(
        &std::fs::read(darwin.join("complete-original-module-graph-inputs.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(other["snapshot_sources"].as_array().unwrap().len(), 5697);
    let other_record = unhex(other["compiler_inputs_hex"].as_str().unwrap());
    let other_wire: Value = serde_json::from_slice(&other_record).unwrap();
    assert_eq!(other_wire["host"], "darwin_arm64");
    let other_exports = exports(&other, darwin);
    let crossed = InstalledGoCompiler::test_original_snapshot(&registry, snapshot)
        .unwrap()
        .bind_original_inputs(&other_record, other_exports);
    assert!(matches!(crossed, Err(InstalledGoCompilerError::Inputs)));
    println!(
        "{}",
        json!({"status":"passed", "method":"test-only-sealed-snapshot", "installed_root_exercised":false, "registered_sdk_files":5577, "registered_sdk_bytes":166354960, "sdk_sources":5480, "sdk_source_directories":source_directories, "original_objects":5697, "original_packages":182, "root_selections":1, "contract_overlays":0, "coherently_rehashed_host_rejected":true, "complete_darwin_owner_crossed_with_linux_snapshot_rejected":true})
    );
}

fn run_whole_snapshot() {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    whole_snapshot_binding(
        &evidence.join("installed-compiler/linux-owner-serial/native-originals/versioned-owned"),
        &evidence.join("compiler-inputs/native/native-originals/versioned-owned"),
    );
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        assert!(
            matches!(
                InstalledGoCompiler::open(),
                Err(InstalledGoCompilerError::Release)
            ),
            "uninstalled executable acquired installed authority"
        );
        run_whole_snapshot();
        return;
    }
    if args.as_slice() == ["--whole-snapshot"] {
        run_whole_snapshot();
        return;
    }
    if let [mode, path] = args.as_slice() {
        if mode == "--validate-registry" {
            let semantic = validate_semantic_profile_registry(
                include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
                RegistryRevision::Revision4,
            )
            .unwrap();
            let registry =
                validate_successor_release_registry(&std::fs::read(path).unwrap(), &semantic)
                    .expect("alternate registry must validate independently");
            println!(
                "{}",
                json!({"status":"accepted", "phase":"registry-validation", "registry_sha256":registry.registry_sha256()})
            );
            return;
        }
    }
    let authority = match InstalledGoCompiler::open() {
        Ok(authority) => authority,
        Err(error) => {
            println!(
                "{}",
                json!({"status":"rejected", "phase":"installed", "error":format!("{error:?}")})
            );
            return;
        }
    };
    let result = match args.as_slice() {
        [mode] if mode == "--snapshot" => {
            json!({"status":"accepted", "phase":"installed", "toolchain":authority.identity(), "inventory":authority.inventory()})
        }
        [mode, directory, record] if mode == "--bind" => {
            let directory = Path::new(directory);
            let packet: Value = serde_json::from_slice(
                &std::fs::read(directory.join("complete-original-module-graph-inputs.json"))
                    .unwrap(),
            )
            .unwrap();
            let inputs = exports(&packet, directory);
            let record = if record == "original" {
                unhex(packet["compiler_inputs_hex"].as_str().unwrap())
            } else {
                std::fs::read(record).unwrap()
            };
            match authority.bind_original_inputs(&record, inputs) {
                Ok(bound) => {
                    let compiler = bound.compiler_inputs();
                    json!({"status":"accepted", "phase":"bound", "release_registry":bound.release_registry(), "compiler_record":serde_json::from_slice::<Value>(compiler.canonical_bytes()).unwrap(), "files":compiler.files().len(), "bytes":compiler.files().iter().map(|f| f.bytes().len()).sum::<usize>(), "sdk_sources":compiler.sdk_source_files().len(), "sdk_source_directories":compiler.source_index().package_paths().len(), "original_objects":packet["snapshot_sources"].as_array().unwrap().len(), "original_packages":serde_json::from_slice::<Value>(compiler.export_inputs().canonical_bytes()).unwrap()["packages"].as_array().unwrap().len()})
                }
                Err(error) => {
                    json!({"status":"rejected", "phase":"binding", "error":format!("{error:?}")})
                }
            }
        }
        _ => panic!("unexpected installed compiler test arguments"),
    };
    println!("{result}");
}
