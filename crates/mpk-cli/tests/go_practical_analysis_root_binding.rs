#![cfg(target_os = "linux")]

#[path = "../src/go_practical_analysis_root_binding.rs"]
mod go_practical_analysis_root_binding;
#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
mod go_practical_source_roots;

use go_practical_analysis_root_binding::{
    bind_go_analysis_root_inputs, GoAnalysisRootBindingError,
};
use go_practical_source_roots::{GoSourceRoots, GoSourceRootsMatch};
use mpk_vc::go_practical_type_keys::{
    check_go_analysis_module_inputs, check_go_analysis_source_inputs,
    check_go_analysis_vendor_snapshot, import_go_source_snapshot_inputs,
    parse_go_analysis_vendor_manifest, GoAnalysisSourceInputs, GoSourceSnapshotInputs,
    GoTypeSourceBuffer,
};
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn sources(packet: &Value, directory: &Path) -> BTreeMap<String, GoTypeSourceBuffer> {
    packet["original_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let raw = if let Some(value) = row["raw_hex"].as_str() {
                unhex(value)
            } else {
                std::fs::read(
                    directory
                        .join("original-bytes")
                        .join(row["raw_file"].as_str().unwrap()),
                )
                .unwrap()
            };
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw)
                    .unwrap();
            (source.object().id.clone(), source)
        })
        .collect()
}
fn snapshot(
    packet: &Value,
    original: &BTreeMap<String, GoTypeSourceBuffer>,
) -> GoSourceSnapshotInputs {
    let buffers = packet["source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| original[id.as_str().unwrap()].clone())
        .collect::<Vec<_>>();
    import_go_source_snapshot_inputs(&unhex(packet["snapshot_hex"].as_str().unwrap()), &buffers)
        .unwrap()
}
fn analysis(packet: &Value, snapshot_packet: &Value, directory: &Path) -> GoAnalysisSourceInputs {
    let original = sources(packet, directory);
    let captured = snapshot(snapshot_packet, &original);
    let manifest = original[packet["manifest_source_id"].as_str().unwrap()].clone();
    let archives = packet["archive_source_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| original[id.as_str().unwrap()].clone())
        .collect();
    let vendor = check_go_analysis_vendor_snapshot(
        captured,
        parse_go_analysis_vendor_manifest(manifest).unwrap(),
        archives,
    )
    .unwrap();
    check_go_analysis_source_inputs(check_go_analysis_module_inputs(vendor).unwrap()).unwrap()
}
fn seal(value: &mut Value, field: &str, domain: &'static str) {
    value.as_object_mut().unwrap().remove(field);
    let parsed = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value[field] = json!(hash_canonical_json(HashDomain::new(domain), &parsed)
        .unwrap()
        .to_hex());
}
fn native_hash(value: &Value) -> String {
    let mut hash = Sha256::new();
    hash.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    fn number(hash: &mut Sha256, n: u64) {
        hash.update(n.to_le_bytes());
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
fn finish(packet: &mut Value, wire: &mut Value) {
    wire["objects"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().unwrap().cmp(b["id"].as_str().unwrap()));
    packet["source_ids"] = json!(wire["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|object| object["id"].as_str().unwrap())
        .collect::<Vec<_>>());
    wire["snapshot_sha256"] = json!(native_hash(wire));
    seal(
        wire,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0",
    );
    packet["snapshot_hex"] = json!(hex(&serde_json::to_vec(wire).unwrap()));
}
fn roots(frontend: &Path, archives: &Path, captured: GoSourceSnapshotInputs) -> GoSourceRootsMatch {
    GoSourceRoots::open(vec![
        ("analysis-frontend".into(), frontend.to_path_buf()),
        ("analysis-archives".into(), archives.to_path_buf()),
    ])
    .unwrap()
    .compare(captured)
    .expect("all original or coherently changed root entries and bytes match")
}

#[test]
fn independent_analysis_owner_binds_only_to_the_same_complete_roots() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let evidence = repo.join("develop/migrations/go-practical/t02-w01/verification");
    let registered = evidence.join("registered-analysis/native-fixed");
    let packet = read(&registered.join("native.json"));
    let original = sources(&packet, &registered);
    assert_eq!(original.len(), 196);
    let captured = snapshot(&packet, &original);
    let inputs = analysis(&packet, &packet, &registered);
    assert_eq!(inputs.build_sources().len(), 184);
    let temp = tempfile::tempdir().unwrap();
    let restored = temp.path().join("registered-original");
    let status = std::process::Command::new("python3")
        .arg("-B")
        .arg(evidence.join("registered-analysis/restore_registered.py"))
        .arg(&restored)
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "all 269 original registered source files restore"
    );
    let frontend = restored.join("go-tools/go2vir");
    let archives = repo.join("go-tools/go2vir/testdata/practical-analysis-cache");
    let accepted = bind_go_analysis_root_inputs(
        inputs.clone(),
        roots(&frontend, &archives, captured.clone()),
    )
    .expect("the complete historical analysis owner and roots must bind");
    assert_eq!(accepted.inputs().build_sources().len(), 184);
    assert_eq!(accepted.matched_snapshot().source_objects().len(), 196);
    drop(accepted);

    // The complete later checkout generation may validate independently but
    // cannot be paired with these historical root handles.
    let current_dir = evidence.join("analysis-namespace/native");
    let current = read(&current_dir.join("native.json"));
    let current_snapshot = read(&evidence.join("analysis-build-list/native/native.json"));
    let later = analysis(&current, &current_snapshot, &current_dir);
    assert_eq!(
        later.module().vendor().snapshot().source_objects().len(),
        213
    );
    assert_eq!(later.build_sources().len(), 201);
    assert!(matches!(
        bind_go_analysis_root_inputs(later, roots(&frontend, &archives, captured.clone())),
        Err(GoAnalysisRootBindingError::Snapshot)
    ));

    // A complete source snapshot with a changed byte and all identities
    // recomputed can match its changed root, but not the original parsed owner.
    let mut changed = packet.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
    let mut object = wire["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|object| {
            object["capability"] == "analysis-frontend" && object["path"] == "preflight.go"
        })
        .unwrap()
        .clone();
    let old_id = object["id"].as_str().unwrap().to_owned();
    let file = frontend.join("preflight.go");
    let original_bytes = std::fs::read(&file).unwrap();
    let mut modified = original_bytes.clone();
    modified[0] ^= 1;
    std::fs::write(&file, &modified).unwrap();
    object["raw_sha256"] = json!(format!("{:x}", Sha256::digest(&modified)));
    seal(&mut object, "id", "MPK-GO-SOURCE-OBJECT-1.0");
    let row = changed["original_sources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["object"]["id"] == old_id)
        .unwrap();
    row["object"] = object.clone();
    row.as_object_mut().unwrap().remove("raw_file");
    row["raw_hex"] = json!(hex(&modified));
    *wire["objects"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == old_id)
        .unwrap() = object;
    finish(&mut changed, &mut wire);
    let modified_sources = sources(&changed, &registered);
    let modified_snapshot = snapshot(&changed, &modified_sources);
    assert_eq!(modified_snapshot.source_objects().len(), 196);
    assert!(matches!(
        bind_go_analysis_root_inputs(
            inputs.clone(),
            roots(&frontend, &archives, modified_snapshot)
        ),
        Err(GoAnalysisRootBindingError::Snapshot)
    ));
    std::fs::write(&file, original_bytes).unwrap();

    // An extra real but unread directory entry and its new snapshot hash are
    // likewise compared against the entire original owner.
    let mut changed = packet.clone();
    let mut wire: Value =
        serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
    let name = b"mpk-analysis-unread-control";
    let extra = frontend.join(std::str::from_utf8(name).unwrap());
    std::fs::write(&extra, b"extra original-root control").unwrap();
    let entries = wire["directories"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|directory| directory["capability"] == "analysis-frontend" && directory["path"] == "")
        .unwrap()["entries"]
        .as_array_mut()
        .unwrap();
    entries.push(json!({"name_hex":hex(name),"type_bits":0}));
    entries.sort_by(|a, b| {
        a["name_hex"]
            .as_str()
            .unwrap()
            .cmp(b["name_hex"].as_str().unwrap())
    });
    finish(&mut changed, &mut wire);
    let changed_snapshot = snapshot(&changed, &original);
    assert!(matches!(
        bind_go_analysis_root_inputs(inputs, roots(&frontend, &archives, changed_snapshot)),
        Err(GoAnalysisRootBindingError::Snapshot)
    ));
    std::fs::remove_file(extra).unwrap();
    println!(
        "{}",
        json!({"status":"passed","registered_source_objects":196,"registered_build_objects":184,"current_generation_source_objects":213,"current_generation_build_objects":201,"cross_generation_rejected":true,"coherently_resealed_source_rejected":true,"coherently_resealed_namespace_rejected":true,"test_only_restored_roots":true,"launcher_authority_claimed":false,"registered_source_binary_authority_claimed":false,"native_Linux_release_acceptance":false})
    );
}
