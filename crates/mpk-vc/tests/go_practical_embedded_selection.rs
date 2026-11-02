use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn unhex(raw: &str) -> Vec<u8> {
    raw.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
fn strings(row: &Value) -> Vec<String> {
    row.as_array().map_or(vec![], |r| {
        r.iter().map(|s| s.as_str().unwrap().into()).collect()
    })
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_EMBEDDED_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
fn original_bytes(path: &Path) -> Vec<u8> {
    if !path.is_file() {
        static RESTORE: std::sync::Once = std::sync::Once::new();
        RESTORE.call_once(|| {
            assert!(std::process::Command::new("python3")
                .arg("-B")
                .arg(root().join("snapshot-inputs/preserve-originals.py"))
                .arg("--restore")
                .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .status()
                .unwrap()
                .success())
        });
    }
    std::fs::read(path).unwrap()
}
fn buffers(packet: &Value, field: &str, directory: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(s) = r["raw_hex"].as_str() {
                unhex(s)
            } else {
                original_bytes(
                    &directory
                        .unwrap()
                        .join("original-source-objects")
                        .join(r["raw_file"].as_str().unwrap()),
                )
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
fn snapshot(packet: &Value, directory: Option<&Path>) -> GoSourceSnapshotInputs {
    import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(packet, "snapshot_sources", directory),
    )
    .unwrap()
}
fn pattern_hex(patterns: &[Vec<u8>]) -> Vec<String> {
    patterns.iter().map(|p| hex(p)).collect()
}
fn selection(packet: &Value) -> GoSourceSelectionInputs {
    import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(packet, "sources", None),
        &buffers(packet, "overlays", None),
    )
    .unwrap()
}

#[test]
fn original_resource_selection_matches_every_actual_native_case() {
    let corpus = read(&root().join(
        "embedded-selection/native/native-originals/matrix/complete-embedded-candidates.json",
    ));
    let mut checked = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let original = snapshot(&row["snapshot"], None);
        let package = derive_go_source_package_files(&original, "original", "").unwrap();
        assert_eq!(
            pattern_hex(package.embed_patterns()),
            strings(&row["native_patterns_hex"]),
            "{row}"
        );
        let derived = derive_go_source_embedded_files(&original, "original", "");
        assert_eq!(
            derived.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{}: {derived:?}",
            row["case"]
        );
        let result = match derived {
            Ok(files) => {
                assert_eq!(files.files(), strings(&row["native_selected_files"]));
                for (path, object) in files.files().iter().zip(files.source_objects()) {
                    assert_eq!(path, &object.path);
                    assert_eq!(original.source(&object.id).unwrap().object(), object);
                }
                json!({"files":files.files(),"objects":files.source_objects(),"patterns":files.pattern_sources()})
            }
            Err(error) => json!({"rejected":error.to_string(),"kind":format!("{:?}",error.kind())}),
        };
        checked.push(json!({"case":row["case"],"complete_original_objects":original.source_objects().len(),"source_pattern_hex":pattern_hex(package.embed_patterns()),"accepted":row["accepted"],"result":result}));
    }
    assert_eq!(checked.len(), 54);
    assert_eq!(checked.iter().filter(|r| r["accepted"] == true).count(), 27);
    assert_eq!(corpus["original_roots_removed"], true);
    output(
        "original-matrix-comparison.json",
        &json!({"status":"passed","cases":checked,"caller_patterns_or_source_adapter":false,"host_filesystem_fallback":false}),
    );
}

#[test]
fn original_directive_bytes_match_actual_go_build_inventory() {
    let corpus = read(&root().join(
        "embedded-selection/native/native-originals/headers/original-embedded-directives.json",
    ));
    let mut checked = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let original = snapshot(row, None);
        let actual = derive_go_source_package_files(&original, "original", "");
        if row["native_observation"]["diagnostic"]
            .as_str()
            .unwrap()
            .is_empty()
        {
            let actual = actual.unwrap_or_else(|e| panic!("{}: {e}", row["case"]));
            assert_eq!(
                pattern_hex(actual.embed_patterns()),
                strings(&row["native_patterns_hex"]),
                "{}",
                row["case"]
            );
            checked.push(
                json!({"case":row["case"],"pattern_hex":pattern_hex(actual.embed_patterns())}),
            );
        } else {
            checked.push(json!({"case":row["case"],"rejected":actual.unwrap_err().to_string()}));
        }
    }
    output(
        "original-pragma-comparison.json",
        &json!({"status":"passed","cases":checked,"native_build_inventory_only":true,"compiler_pragma_or_program_proof":false}),
    );
}

#[test]
fn patterns_keep_every_whole_original_package_directory() {
    let corpus=read(&root().join("embedded-selection/native/native-originals/originals/whole-original-embedded-patterns.json"));
    let mut checked = vec![];
    for unit in corpus["units"].as_array().unwrap() {
        let path = root()
            .join("snapshot-inputs")
            .join(unit["packet"].as_str().unwrap());
        let packet = read(&path);
        let original = snapshot(&packet, path.parent());
        assert_eq!(
            original.source_objects().len() as u64,
            unit["complete_original_objects"].as_u64().unwrap()
        );
        let mut packages = vec![];
        for row in unit["packages"].as_array().unwrap() {
            let actual = derive_go_source_package_files(
                &original,
                row["capability"].as_str().unwrap(),
                row["directory"].as_str().unwrap(),
            );
            if row["diagnostic"].as_str().unwrap().is_empty() {
                let actual = actual.unwrap_or_else(|e| panic!("{row}: {e}"));
                assert_eq!(
                    pattern_hex(actual.embed_patterns()),
                    strings(&row["native_patterns_hex"]),
                    "{row}"
                );
                assert_eq!(
                    pattern_hex(actual.test_embed_patterns()),
                    strings(&row["native_test_patterns_hex"]),
                    "{row}"
                );
                assert_eq!(
                    pattern_hex(actual.external_test_embed_patterns()),
                    strings(&row["native_external_test_patterns_hex"]),
                    "{row}"
                );
                packages.push(json!({"capability":row["capability"],"directory":row["directory"],"patterns_hex":pattern_hex(actual.embed_patterns()),"test_patterns_hex":pattern_hex(actual.test_embed_patterns()),"external_test_patterns_hex":pattern_hex(actual.external_test_embed_patterns())}));
            } else {
                packages.push(json!({"capability":row["capability"],"directory":row["directory"],"rejected":actual.unwrap_err().to_string()}));
            }
        }
        checked.push(json!({"unit":unit["unit"],"original_objects":original.source_objects().len(),"packages":packages}));
    }
    assert_eq!(
        checked
            .iter()
            .map(|u| u["packages"].as_array().unwrap().len())
            .collect::<Vec<_>>(),
        [668, 668, 667]
    );
    output(
        "whole-original-pattern-comparison.json",
        &json!({"status":"passed","units":checked,"original_roots_type_error_and_proof_domains_unchanged":true}),
    );
}

fn owner() -> (PathBuf, Value, GoSourceCaptureInputs) {
    let directory = root().join("embedded-inputs/native/native-originals/owned");
    let packet = read(&directory.join("complete-embedded-capture.json"));
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection(&packet),
        snapshot(&packet, Some(&directory)),
    )
    .unwrap();
    (directory, packet, capture)
}
#[test]
fn original_owned_capture_derives_every_embedded_resource_from_source() {
    let (directory, _, capture) = owner();
    assert_eq!(capture.snapshot_inputs().source_objects().len(), 5535);
    assert_eq!(capture.derive_contracts().unwrap().functions().len(), 1);
    let native = read(&directory.join("original-embedded-resources.json"));
    let cap = native["selected"][0]["object"]["capability"]
        .as_str()
        .unwrap();
    let actual = derive_go_source_embedded_files(capture.snapshot_inputs(), cap, "").unwrap();
    assert_eq!(actual.files(), strings(&native["files"]));
    let mut resources = vec![];
    for (object, row) in actual
        .source_objects()
        .iter()
        .zip(native["selected"].as_array().unwrap())
    {
        assert_eq!(serde_json::to_value(object).unwrap(), row["object"]);
        let source = capture.snapshot_inputs().source(&object.id).unwrap();
        assert_eq!(source.bytes(), unhex(row["raw_hex"].as_str().unwrap()));
        resources.push(json!({"object":object,"raw_hex":hex(source.bytes())}));
    }
    assert_eq!(resources.len(), 4);
    output(
        "whole-original-owned-resources.json",
        &json!({"status":"passed","original_objects":5535,"original_selected_contract_functions":1,"files":actual.files(),"pattern_sources":actual.pattern_sources(),"resources":resources,"independent_original_source_pattern_and_resource_selection":true,"source_or_context_program_proof":false}),
    );
}

fn seal(v: &mut Value, domain: &'static str) {
    v.as_object_mut().unwrap().remove("content_sha256");
    let parsed = parse_strict_json(
        &serde_json::to_vec(v).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    v["content_sha256"] = json!(hash_canonical_json(HashDomain::new(domain), &parsed)
        .unwrap()
        .to_hex());
}
fn native_hash(v: &Value) -> String {
    fn n(h: &mut Sha256, n: u64) {
        h.update(n.to_le_bytes())
    }
    fn t(h: &mut Sha256, b: &[u8]) {
        n(h, b.len() as u64);
        h.update(b)
    }
    let mut h = Sha256::new();
    h.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    let roots = v["capabilities"].as_array().unwrap();
    n(&mut h, roots.len() as u64);
    for r in roots {
        t(&mut h, r.as_str().unwrap().as_bytes())
    }
    let dirs = v["directories"].as_array().unwrap();
    n(&mut h, dirs.len() as u64);
    for d in dirs {
        t(&mut h, d["capability"].as_str().unwrap().as_bytes());
        t(&mut h, d["path"].as_str().unwrap().as_bytes());
        let es = d["entries"].as_array().unwrap();
        n(&mut h, es.len() as u64);
        for e in es {
            t(&mut h, &unhex(e["name_hex"].as_str().unwrap()));
            n(&mut h, e["type_bits"].as_u64().unwrap());
        }
    }
    let os = v["objects"].as_array().unwrap();
    n(&mut h, os.len() as u64);
    for o in os {
        t(&mut h, o["id"].as_str().unwrap().as_bytes())
    }
    format!("{:x}", h.finalize())
}
#[test]
fn complete_capture_rejects_coherently_omitted_original_resource_inputs() {
    let (directory, packet, original) = owner();
    let all = buffers(&packet, "snapshot_sources", Some(&directory));
    let selected = original.selection_inputs().clone();
    assert_eq!(selected.derive_contracts().unwrap().functions().len(), 1);
    let asset = all
        .iter()
        .find(|s| s.object().path == "assets/binary.dat")
        .unwrap();
    let cap = asset.object().capability.clone();
    let mut checked = vec![];
    for (name, path, missing_dir, kind) in [
        (
            "buffer",
            "assets/binary.dat",
            None,
            GoSourcePackageErrorKind::EmbedSource,
        ),
        (
            "directory",
            "assets/empty/.ignored",
            Some("assets/empty"),
            GoSourcePackageErrorKind::EmbedNamespace,
        ),
    ] {
        let omitted = all
            .iter()
            .find(|s| s.object().capability == cap && s.object().path == path)
            .unwrap();
        let kept = all
            .iter()
            .filter(|s| s.object().id != omitted.object().id)
            .cloned()
            .collect::<Vec<_>>();
        let mut wire: Value =
            serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
        wire["objects"]
            .as_array_mut()
            .unwrap()
            .retain(|o| o["id"] != omitted.object().id);
        if let Some(path) = missing_dir {
            wire["directories"]
                .as_array_mut()
                .unwrap()
                .retain(|d| !(d["capability"] == cap && d["path"] == path));
        }
        wire["snapshot_sha256"] = json!(native_hash(&wire));
        seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
        let changed =
            import_go_source_snapshot_inputs(&serde_json::to_vec(&wire).unwrap(), &kept).unwrap();
        let mut capture: Value =
            serde_json::from_slice(&unhex(packet["capture_hex"].as_str().unwrap())).unwrap();
        capture["snapshot_inputs_sha256"] = json!(changed.content_sha256());
        seal(&mut capture, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
        let GoSourceSnapshotError::Package(error) = import_go_source_capture_inputs(
            &serde_json::to_vec(&capture).unwrap(),
            selected.clone(),
            changed,
        )
        .unwrap_err() else {
            panic!("original resource guard")
        };
        assert_eq!(error.kind(), kind);
        assert_eq!(error.path(), missing_dir.unwrap_or(path));
        checked.push(json!({"case":name,"omitted_original_object":omitted.object(),"changed_snapshot_hex":hex(&serde_json::to_vec(&wire).unwrap()),"changed_capture_hex":hex(&serde_json::to_vec(&capture).unwrap()),"same_original_source_selection_and_contract":true,"error":error.to_string(),"kind":format!("{:?}",error.kind())}));
    }
    output(
        "coherent-original-resource-omissions.json",
        &json!({"status":"passed","original_objects":5535,"same_original_selected_contract_functions":1,"cases":checked,"original_capture_unchanged":true,"source_or_context_program_proof":false}),
    );
}

#[test]
fn original_non_test_resources_keep_internal_and_external_test_patterns_separate() {
    let corpus=read(&root().join("embedded-selection/native-groups/native-originals/groups/original-test-pattern-groups.json"));
    let mut checked = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let original = snapshot(&row["snapshot"], None);
        let package = derive_go_source_package_files(&original, "original", "").unwrap();
        assert_eq!(
            pattern_hex(package.embed_patterns()),
            strings(&row["native_patterns_hex"])
        );
        assert_eq!(
            pattern_hex(package.test_embed_patterns()),
            strings(&row["native_test_patterns_hex"])
        );
        assert_eq!(
            pattern_hex(package.external_test_embed_patterns()),
            strings(&row["native_external_test_patterns_hex"])
        );
        assert!(
            !package.test_embed_patterns().is_empty()
                || !package.external_test_embed_patterns().is_empty()
        );
        let resources = derive_go_source_embedded_files(&original, "original", "").unwrap();
        assert_eq!(resources.files(), strings(&row["native_selected_files"]));
        assert_eq!(resources.files(), ["assets/prod.txt"]);
        checked.push(json!({"case":row["case"],"original_objects":original.source_objects().len(),"production_pattern_hex":pattern_hex(package.embed_patterns()),"internal_test_pattern_hex":pattern_hex(package.test_embed_patterns()),"external_test_pattern_hex":pattern_hex(package.external_test_embed_patterns()),"production_files":resources.files(),"production_objects":resources.source_objects()}));
    }
    assert_eq!(checked.len(), 3);
    assert!(corpus["original_roots_removed"].as_bool().unwrap());
    output(
        "original-test-pattern-groups.json",
        &json!({"status":"passed","cases":checked,"original_test_patterns_retained_without_production_resource_reads":true,"source_or_context_program_proof":false}),
    );
}
