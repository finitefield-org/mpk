//! Exercise the unchanged private boundary from an actual installed bin/mpk.
//! The outer driver materializes every registered Go/Rust bundle byte.
#[path = "../src/frontend_registry.rs"]
#[allow(unused_imports)]
mod frontend_registry;
#[path = "../src/go_practical_installed_compiler.rs"]
#[allow(dead_code)]
mod go_practical_installed_compiler;

use go_practical_installed_compiler::{
    InstalledGoCompiler, InstalledGoCompilerError, InstalledGoRelease,
};
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::release_bundle_v1::{
    validate_successor_release_registry, GO_FRONTEND_BUNDLE_ID, GO_TOOLCHAIN_BUNDLE_ID,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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
    let selection: Value =
        serde_json::from_slice(&unhex(packet["selection_hex"].as_str().unwrap())).unwrap();
    let roots = selection["roots"].as_array().unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(packet["overlays"].as_array().unwrap().len(), 0);
    let selected_source = packet["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["object"]["id"] == roots[0]["source_object_id"])
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

    // Both snapshots are independently sealed from the entire registered
    // frontend and SDK inventories. This test does not impersonate the
    // installed root or establish that the frontend produced the capture.
    let frontend = registry
        .registry()
        .frontend_bundles
        .iter()
        .find(|bundle| bundle.bundle_id == GO_FRONTEND_BUNDLE_ID)
        .unwrap();
    assert_eq!(frontend.inventory.files.len(), 1);
    let frontend_entry = &frontend.inventory.files[0];
    assert_eq!(frontend_entry.path, frontend.main.path);
    let frontend_hex = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../develop/migrations/go-practical/t02-w01/verification/registered-analysis/rebuild-original/frontend-0.elf.hex",
        ),
    )
    .unwrap();
    let frontend_bytes = unhex(&frontend_hex);
    assert_eq!(frontend_bytes.len(), 9_004_378);
    let frontend_snapshot = Arc::new(
        frontend_registry::test_snapshot(
            &frontend.inventory,
            vec![(
                frontend_entry.path.clone(),
                frontend_bytes.clone(),
                frontend_entry.executable,
            )],
        )
        .unwrap(),
    );
    let release = InstalledGoRelease::test_original_snapshots(
        &registry,
        frontend_snapshot.clone(),
        snapshot.clone(),
    )
    .expect("the complete registered frontend and SDK snapshots bind");
    assert_eq!(release.frontend().bundle_id(), GO_FRONTEND_BUNDLE_ID);
    assert_eq!(release.frontend().main_path(), frontend.main.path);
    assert_eq!(release.frontend().inventory(), &frontend.inventory);
    assert_eq!(release.compiler().inventory(), inventory);
    let release_inputs = release
        .bind_original_inputs(&record, original_exports.clone())
        .expect("both original snapshots remain with the original compiler input");
    assert_eq!(
        release_inputs
            .frontend()
            .snapshot()
            .file(release_inputs.frontend().main_path())
            .unwrap()
            .bytes,
        frontend_bytes
    );
    assert_eq!(release_inputs.frontend().inventory(), &frontend.inventory);
    assert_eq!(
        release_inputs.compiler().release_registry(),
        &registry.release_identity()
    );
    assert_eq!(
        release_inputs
            .compiler()
            .compiler_inputs()
            .canonical_bytes(),
        record
    );
    assert_eq!(
        release_inputs.compiler().compiler_inputs().files().len(),
        5577
    );
    drop(release_inputs);

    // A complete, coherently resealed different frontend is not the frontend
    // pinned by this registry, even when the original SDK remains unchanged.
    let mut altered_frontend = frontend_bytes.clone();
    let last = altered_frontend.len() - 1;
    altered_frontend[last] ^= 1;
    let mut alternate_inventory = frontend.inventory.clone();
    alternate_inventory.files[0].sha256 = format!("{:x}", Sha256::digest(&altered_frontend));
    let alternate = Arc::new(
        frontend_registry::test_snapshot(
            &alternate_inventory,
            vec![(
                frontend_entry.path.clone(),
                altered_frontend,
                frontend_entry.executable,
            )],
        )
        .unwrap(),
    );
    assert!(matches!(
        InstalledGoRelease::test_original_snapshots(&registry, alternate, snapshot.clone()),
        Err(InstalledGoCompilerError::Identity)
    ));

    let mut wrong_mode_inventory = frontend.inventory.clone();
    wrong_mode_inventory.files[0].executable = !frontend_entry.executable;
    let wrong_mode = Arc::new(
        frontend_registry::test_snapshot(
            &wrong_mode_inventory,
            vec![(
                frontend_entry.path.clone(),
                frontend_bytes,
                !frontend_entry.executable,
            )],
        )
        .unwrap(),
    );
    assert!(matches!(
        InstalledGoRelease::test_original_snapshots(&registry, wrong_mode, snapshot.clone()),
        Err(InstalledGoCompilerError::Identity)
    ));

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
    let other_selection: Value =
        serde_json::from_slice(&unhex(other["selection_hex"].as_str().unwrap())).unwrap();
    assert_eq!(other_selection["roots"], selection["roots"]);
    assert_eq!(other["overlays"], packet["overlays"]);
    let other_source = other["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["object"]["id"] == roots[0]["source_object_id"])
        .unwrap();
    assert_eq!(other_source, selected_source);
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
        json!({"status":"passed", "method":"test-only-sealed-snapshot", "installed_root_exercised":false, "registered_frontend_files":1, "registered_frontend_bytes":9004378, "registered_frontend_and_sdk_retained":true, "coherently_resealed_frontend_rejected":true, "coherently_resealed_frontend_mode_rejected":true, "registered_sdk_files":5577, "registered_sdk_bytes":166354960, "sdk_sources":5480, "sdk_source_directories":source_directories, "original_objects":packet["snapshot_sources"].as_array().unwrap().len(), "original_packages":182, "root_selections":roots.len(), "contract_overlays":packet["overlays"].as_array().unwrap().len(), "same_selected_original_source_and_root":true, "coherently_rehashed_host_rejected":true, "complete_darwin_owner_crossed_with_linux_snapshot_rejected":true})
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

// Test-only diagnostic: expose the existing loader's rejection detail without
// adding a production path override or changing its error mapping.
fn diagnose_installed() {
    let installed = match frontend_registry::InstalledSuccessorRelease::open() {
        Ok(value) => value,
        Err(error) => {
            println!(
                "{}",
                json!({"phase":"descriptors", "error":error.to_string()})
            );
            return;
        }
    };
    let semantic = validate_semantic_profile_registry(
        &installed.semantic_registry_bytes,
        RegistryRevision::Revision4,
    )
    .unwrap();
    let registry = validate_successor_release_registry(&installed.registry_bytes, &semantic)
        .expect("installed registry validates");
    assert_eq!(
        semantic.identity().registry_sha256(),
        frontend_registry::EXPECTED_SEMANTIC_REGISTRY_SHA256
    );
    assert_eq!(
        registry.registry_sha256(),
        frontend_registry::EXPECTED_REGISTRY_SHA256_HEX
    );
    let expected = registry
        .registry()
        .frontend_bundles
        .iter()
        .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory))
        .chain(
            registry
                .registry()
                .toolchain_bundles
                .iter()
                .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory)),
        )
        .collect::<BTreeMap<_, _>>();
    match installed.snapshot_selected_bundles(
        &expected,
        mpk_vc::release_bundle_v1::GO_FRONTEND_BUNDLE_ID,
        GO_TOOLCHAIN_BUNDLE_ID,
    ) {
        Ok(_) => println!("{}", json!({"phase":"snapshot", "status":"accepted"})),
        Err(error) => println!("{}", json!({"phase":"snapshot", "error":error.to_string()})),
    }
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
        assert!(
            matches!(
                InstalledGoRelease::open(),
                Err(InstalledGoCompilerError::Release)
            ),
            "uninstalled executable acquired installed frontend authority"
        );
        run_whole_snapshot();
        return;
    }
    if args.as_slice() == ["--whole-snapshot"] {
        run_whole_snapshot();
        return;
    }
    if args.as_slice() == ["--diagnose-installed"] {
        diagnose_installed();
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
    if matches!(
        args.first().map(String::as_str),
        Some("--release-snapshot" | "--release-bind")
    ) {
        let release = match InstalledGoRelease::open() {
            Ok(release) => release,
            Err(error) => {
                println!(
                    "{}",
                    json!({"status":"rejected", "phase":"installed", "error":format!("{error:?}")})
                );
                return;
            }
        };
        let frontend = release.frontend();
        let file = frontend.snapshot().file(frontend.main_path()).unwrap();
        let frontend_bytes = file.bytes.len();
        let frontend_hash = file.sha256.clone();
        let frontend_files = frontend.inventory().files.len();
        let frontend_id = frontend.bundle_id().to_owned();
        let result = match args.as_slice() {
            [mode] if mode == "--release-snapshot" => json!({
                "status":"accepted", "phase":"installed-release",
                "frontend_id":frontend_id, "frontend_files":frontend_files,
                "frontend_bytes":frontend_bytes, "frontend_sha256":frontend_hash,
                "sdk_files":release.compiler().inventory().files.len()
            }),
            [mode, directory, record] if mode == "--release-bind" => {
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
                match release.bind_original_inputs(&record, inputs) {
                    Ok(bound) => {
                        let compiler = bound.compiler().compiler_inputs();
                        let file = bound
                            .frontend()
                            .snapshot()
                            .file(bound.frontend().main_path())
                            .unwrap();
                        assert_eq!(file.bytes.len(), frontend_bytes);
                        assert_eq!(file.sha256, frontend_hash);
                        json!({"status":"accepted", "phase":"release-bound",
                            "frontend_id":frontend_id, "frontend_files":frontend_files,
                            "frontend_bytes":frontend_bytes, "frontend_sha256":frontend_hash,
                            "release_registry":bound.compiler().release_registry(),
                            "files":compiler.files().len(),
                            "bytes":compiler.files().iter().map(|f| f.bytes().len()).sum::<usize>(),
                            "original_objects":packet["snapshot_sources"].as_array().unwrap().len(),
                            "original_packages":serde_json::from_slice::<Value>(compiler.export_inputs().canonical_bytes()).unwrap()["packages"].as_array().unwrap().len()})
                    }
                    Err(error) => {
                        json!({"status":"rejected", "phase":"binding", "error":format!("{error:?}")})
                    }
                }
            }
            _ => panic!("unexpected installed release test arguments"),
        };
        println!("{result}");
        return;
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
