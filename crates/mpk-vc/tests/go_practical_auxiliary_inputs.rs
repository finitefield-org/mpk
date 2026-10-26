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
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
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
                .success());
        });
    }
    std::fs::read(path).unwrap()
}
fn buffers(p: &Value, directory: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    p["snapshot_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(h) = r["raw_hex"].as_str() {
                unhex(h)
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
fn snapshot(p: &Value, directory: Option<&Path>) -> GoSourceSnapshotInputs {
    import_go_source_snapshot_inputs(
        &unhex(p["snapshot_hex"].as_str().unwrap()),
        &buffers(p, directory),
    )
    .unwrap()
}
fn output(name: &str, v: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_AUXILIARY_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(v).unwrap())
            .unwrap();
    }
}
fn names(s: &[GoTypeSourceObject]) -> Vec<&str> {
    s.iter()
        .map(|s| s.path.rsplit('/').next().unwrap())
        .collect()
}
fn expected(v: &Value) -> Vec<&str> {
    v.as_array()
        .map(|a| a.iter().map(|s| s.as_str().unwrap()).collect())
        .unwrap_or_default()
}
fn compare(snapshot: &GoSourceSnapshotInputs, row: &Value) -> Value {
    let cap = row["capability"].as_str().unwrap_or("original");
    let dir = row["directory"].as_str().unwrap_or("");
    assert!(
        row["diagnostic"].as_str().unwrap().is_empty(),
        "native baseline {cap}/{dir}: {}",
        row["diagnostic"]
    );
    let files = derive_go_source_package_files(snapshot, cap, dir)
        .unwrap_or_else(|e| panic!("{cap}/{dir}: {e}"));
    let kinds = [
        ("c", GoSourceAuxiliaryKind::C),
        ("cxx", GoSourceAuxiliaryKind::Cxx),
        ("objective_c", GoSourceAuxiliaryKind::ObjectiveC),
        ("headers", GoSourceAuxiliaryKind::Header),
        ("fortran", GoSourceAuxiliaryKind::Fortran),
        ("assembly", GoSourceAuxiliaryKind::Assembly),
        ("swig", GoSourceAuxiliaryKind::Swig),
        ("swig_cxx", GoSourceAuxiliaryKind::SwigCxx),
        ("system_objects", GoSourceAuxiliaryKind::SystemObject),
    ];
    let mut selected = serde_json::Map::new();
    for (field, kind) in kinds {
        let actual = names(files.auxiliary_sources(kind));
        assert_eq!(actual, expected(&row[field]), "{cap}/{dir} {field}");
        selected.insert(field.into(), json!(actual));
    }
    assert_eq!(
        names(files.ignored_auxiliary_sources()),
        expected(&row["ignored_auxiliary"]),
        "{cap}/{dir} ignored auxiliary"
    );
    assert_eq!(names(files.active_sources()), expected(&row["go_files"]));
    assert_eq!(files.package_name(), row["name"].as_str().unwrap());
    json!({"capability":cap,"directory":dir,"selected":selected,"ignored_auxiliary":names(files.ignored_auxiliary_sources()),"complete_original_objects":snapshot.source_objects().len()})
}
#[test]
fn auxiliary_inputs_match_every_original_native_selection_case() {
    let corpus =
        read(&root().join("auxiliary-inputs/native/native-originals/cases/auxiliary-corpus.json"));
    let mut rows = vec![];
    for p in corpus["cases"].as_array().unwrap() {
        let original = snapshot(p, None);
        let mut result = compare(&original, &p["expected"]);
        result["case"] = p["name"].clone();
        rows.push(result);
    }
    assert!(rows.len() > 170);
    output(
        "native-case-comparison.json",
        &json!({"status":"passed","cases":rows,"original_bytes_unchanged":true,"additional_program_language_semantics":false}),
    );
}
#[test]
fn auxiliary_inputs_match_every_package_in_all_complete_original_universes() {
    let corpus =
        read(&root().join(
            "auxiliary-inputs/native/native-originals/originals/original-auxiliary-files.json",
        ));
    let mut units = vec![];
    for unit in corpus["units"].as_array().unwrap() {
        let file = root()
            .join("snapshot-inputs")
            .join(unit["packet"].as_str().unwrap());
        let p = read(&file);
        let original = snapshot(&p, file.parent());
        let expected = original
            .source_objects()
            .iter()
            .filter(|s| s.kind == "go_file")
            .map(|s| {
                (
                    s.capability.as_str(),
                    s.path.rsplit_once('/').map_or("", |(d, _)| d),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        let mut seen = std::collections::BTreeSet::new();
        let mut rows = vec![];
        for row in unit["packages"].as_array().unwrap() {
            assert!(seen.insert((
                row["capability"].as_str().unwrap(),
                row["directory"].as_str().unwrap()
            )));
            rows.push(compare(&original, row));
        }
        assert_eq!(expected, seen);
        units.push(json!({"unit":unit["unit"],"complete_original_objects":original.source_objects().len(),"packages":rows}));
    }
    assert_eq!(units.len(), 3);
    output(
        "complete-original-universe-comparison.json",
        &json!({"status":"passed","units":units,"original_source_root_type_error_and_proof_scope_preserved":true}),
    );
}
fn reseal(w: &mut Value) {
    let mut h = Sha256::new();
    h.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    fn number(h: &mut Sha256, n: usize) {
        h.update((n as u64).to_le_bytes())
    }
    fn text(h: &mut Sha256, s: &[u8]) {
        number(h, s.len());
        h.update(s)
    }
    let caps = w["capabilities"].as_array().unwrap();
    number(&mut h, caps.len());
    for c in caps {
        text(&mut h, c.as_str().unwrap().as_bytes());
    }
    let dirs = w["directories"].as_array().unwrap();
    number(&mut h, dirs.len());
    for d in dirs {
        text(&mut h, d["capability"].as_str().unwrap().as_bytes());
        text(&mut h, d["path"].as_str().unwrap().as_bytes());
        let entries = d["entries"].as_array().unwrap();
        number(&mut h, entries.len());
        for e in entries {
            text(&mut h, &unhex(e["name_hex"].as_str().unwrap()));
            h.update(e["type_bits"].as_u64().unwrap().to_le_bytes());
        }
    }
    let objects = w["objects"].as_array().unwrap();
    number(&mut h, objects.len());
    for o in objects {
        text(&mut h, o["id"].as_str().unwrap().as_bytes());
    }
    w["snapshot_sha256"] = json!(format!("{:x}", h.finalize()));
    w.as_object_mut().unwrap().remove("content_sha256");
    let value = parse_strict_json(
        &serde_json::to_vec(w).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    w["content_sha256"] = json!(hash_canonical_json(
        HashDomain::new("MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0"),
        &value
    )
    .unwrap()
    .to_hex());
}
#[test]
fn auxiliary_inputs_reject_missing_and_reclassified_original_buffers() {
    let corpus =
        read(&root().join("auxiliary-inputs/native/native-originals/cases/auxiliary-corpus.json"));
    let p = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "extension-s")
        .unwrap();
    let original = buffers(p, None);
    let object = original
        .iter()
        .find(|s| s.object().path == "original.s")
        .unwrap();
    let wire: Value = serde_json::from_slice(&unhex(p["snapshot_hex"].as_str().unwrap())).unwrap();
    let mut rows = vec![];
    for reclassify in [false, true] {
        let mut w = wire.clone();
        let changed = GoTypeSourceBuffer::new(
            &object.object().capability,
            &object.object().path,
            "metadata",
            object.bytes(),
        )
        .unwrap();
        if reclassify {
            for o in w["objects"].as_array_mut().unwrap() {
                if o["id"] == object.object().id {
                    *o = serde_json::to_value(changed.object()).unwrap();
                }
            }
        } else {
            w["objects"]
                .as_array_mut()
                .unwrap()
                .retain(|o| o["id"] != object.object().id);
        }
        w["objects"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        reseal(&mut w);
        let sources = original
            .iter()
            .filter_map(|s| {
                if s.object().id != object.object().id {
                    Some(s.clone())
                } else if reclassify {
                    Some(changed.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let snapshot =
            import_go_source_snapshot_inputs(&serde_json::to_vec(&w).unwrap(), &sources).unwrap();
        let err = derive_go_source_package_files(&snapshot, "original", "").unwrap_err();
        assert_eq!(err.kind(), GoSourcePackageErrorKind::Read);
        assert_eq!(err.path(), "original.s");
        rows.push(json!({"reclassified":reclassify,"original_auxiliary":object.object(),"error":err.to_string()}));
    }
    output(
        "closed-original-buffer-controls.json",
        &json!({"status":"passed","controls":rows,"all_original_names_preserved":true,"fallback_filesystem_or_source_adapter":false}),
    );
}
