use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Once;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/snapshot-inputs")
}
fn read(path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(path)).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_SNAPSHOT_INPUTS_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
fn small() -> Value {
    read("native/native-originals/namespace/snapshot-inputs.json")
}
fn wire(p: &Value) -> Value {
    serde_json::from_slice(&unhex(p["snapshot_hex"].as_str().unwrap())).unwrap()
}
fn buffers(p: &Value, field: &str, directory: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    p[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(hex) = r["raw_hex"].as_str() {
                unhex(hex)
            } else {
                let path = directory
                    .unwrap()
                    .join("original-source-objects")
                    .join(r["raw_file"].as_str().unwrap());
                if !path.is_file() {
                    static RESTORE: Once = Once::new();
                    RESTORE.call_once(|| {
                        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
                        assert!(std::process::Command::new("python3")
                            .arg("-B")
                            .arg(root().join("preserve-originals.py"))
                            .arg("--restore")
                            .current_dir(repo)
                            .status()
                            .unwrap()
                            .success());
                    });
                }
                std::fs::read(path).unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
fn import_small(p: &Value, v: &Value) -> Result<GoSourceSnapshotInputs, GoSourceSnapshotError> {
    import_go_source_snapshot_inputs(
        &serde_json::to_vec(v).unwrap(),
        &buffers(p, "snapshot_sources", None),
    )
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
    let mut h = Sha256::new();
    h.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    fn number(h: &mut Sha256, n: u64) {
        h.update(n.to_le_bytes())
    }
    fn text(h: &mut Sha256, b: &[u8]) {
        number(h, b.len() as u64);
        h.update(b)
    }
    let roots = v["capabilities"].as_array().unwrap();
    number(&mut h, roots.len() as u64);
    for c in roots {
        text(&mut h, c.as_str().unwrap().as_bytes())
    }
    let dirs = v["directories"].as_array().unwrap();
    number(&mut h, dirs.len() as u64);
    for d in dirs {
        text(&mut h, d["capability"].as_str().unwrap().as_bytes());
        text(&mut h, d["path"].as_str().unwrap().as_bytes());
        let entries = d["entries"].as_array().unwrap();
        number(&mut h, entries.len() as u64);
        for e in entries {
            text(&mut h, &unhex(e["name_hex"].as_str().unwrap()));
            number(&mut h, e["type_bits"].as_u64().unwrap())
        }
    }
    let objects = v["objects"].as_array().unwrap();
    number(&mut h, objects.len() as u64);
    for o in objects {
        text(&mut h, o["id"].as_str().unwrap().as_bytes())
    }
    format!("{:x}", h.finalize())
}
fn reseal(v: &mut Value) {
    v["snapshot_sha256"] = json!(native_hash(v));
    seal(v, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0")
}

#[test]
fn snapshot_inputs_reproduce_every_actual_native_lookup_and_source_byte() {
    let p = small();
    let source = buffers(&p, "snapshot_sources", None);
    let snapshot =
        import_go_source_snapshot_inputs(&unhex(p["snapshot_hex"].as_str().unwrap()), &source)
            .unwrap();
    assert_eq!(snapshot.source_objects().len(), 9);
    assert_eq!(
        snapshot.canonical_bytes(),
        unhex(p["snapshot_hex"].as_str().unwrap())
    );
    assert_eq!(snapshot.snapshot_sha256(), native_hash(&wire(&p)));
    for s in &source {
        assert_eq!(
            snapshot
                .read_source(&s.object().capability, &s.object().path, &s.object().kind)
                .unwrap(),
            s
        );
    }
    drop(source);
    let native = read("native/native-originals/namespace/namespace-lookups.json");
    let mut checked = vec![];
    for q in native["queries"].as_array().unwrap() {
        let cap = q["capability"].as_str().unwrap();
        let path = q["path"].as_str().unwrap();
        let result = if q["directory"].as_bool().unwrap() {
            snapshot.read_directory(cap, path).map(|_| ())
        } else {
            snapshot
                .read_source(cap, path, q["kind"].as_str().unwrap())
                .map(|_| ())
        };
        let outcome = match result {
            Ok(()) => "present",
            Err(GoSourceSnapshotLookupError::NotFound) => "absent",
            Err(GoSourceSnapshotLookupError::NotCaptured) => "uncaptured",
            Err(_) => "rejected",
        };
        assert_eq!(outcome, q["outcome"].as_str().unwrap(), "{}", q["name"]);
        checked.push(json!({"name":q["name"],"outcome":outcome}));
    }
    assert_eq!(checked.len(), 30);
    output(
        "native-lookup-comparison.json",
        &json!({"status":"passed","all_original_objects":9,"original_native_queries":checked,"no_filesystem_fallback":true}),
    );
}

#[test]
fn snapshot_inputs_preserve_invalid_utf8_names_without_json_replacement() {
    let p = read("native/native-originals/namespace/raw-native-name-unit.json");
    let snapshot = import_small(&p, &wire(&p)).unwrap();
    let names = snapshot
        .read_directory("root", "")
        .unwrap()
        .iter()
        .map(|e| e.name_bytes().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(names, vec![vec![0xef, 0xbf, 0xbd], vec![0xfe], vec![0xff]]);
    assert_eq!(snapshot.snapshot_sha256(), native_hash(&wire(&p)));
    output(
        "native-name-bytes.json",
        &json!({"status":"passed","three_raw_names_distinct":true,"native_capture_authority_claimed":false}),
    );
}

#[test]
fn snapshot_inputs_reject_noncanonical_incomplete_and_disconnected_data() {
    let p = small();
    let original = wire(&p);
    let mut cases = vec![];
    let mut v = original.clone();
    v["directories"][0]["entries"][0]["type_bits"] = json!(1);
    cases.push(("stale-hash", v));
    let mut v = original.clone();
    v["schema"] = json!("unknown");
    seal(&mut v, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
    cases.push(("schema", v));
    let mut v = original.clone();
    v["extra"] = json!(false);
    reseal(&mut v);
    cases.push(("extra", v));
    let mut v = original.clone();
    v["capabilities"] = json!([]);
    reseal(&mut v);
    cases.push(("empty-capabilities", v));
    let mut v = original.clone();
    v["capabilities"][1] = v["capabilities"][0].clone();
    reseal(&mut v);
    cases.push(("duplicate-capability", v));
    let mut v = original.clone();
    v["directories"].as_array_mut().unwrap().reverse();
    reseal(&mut v);
    cases.push(("directory-order", v));
    for (name, value) in [
        ("empty-name", ""),
        ("slash-name", "2f"),
        ("nul-name", "00"),
        ("dot-name", "2e"),
        ("parent-name", "2e2e"),
        ("uppercase-hex", "FF"),
        ("odd-hex", "f"),
    ] {
        let mut v = original.clone();
        v["directories"][0]["entries"][0]["name_hex"] = json!(value);
        if name == "odd-hex" {
            seal(&mut v, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
        } else {
            reseal(&mut v);
        }
        cases.push((name, v));
    }
    for (name, bits) in [
        ("permission-bits", 1u64),
        ("append-mode", 1 << 30),
        ("type-over-u32", 1u64 << 32),
    ] {
        let mut v = original.clone();
        v["directories"][0]["entries"][0]["type_bits"] = json!(bits);
        reseal(&mut v);
        cases.push((name, v));
    }
    let mut v = original.clone();
    v["directories"][0]["entries"][0]["type_bits"] = json!(false);
    seal(&mut v, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
    cases.push(("boolean-mode", v));
    let mut v = original.clone();
    let dirs = v["directories"].as_array_mut().unwrap();
    dirs.retain(|d| !(d["capability"] == "workspace/main" && d["path"] == ""));
    reseal(&mut v);
    cases.push(("disconnected-directory", v));
    let mut v = original.clone();
    v["objects"].as_array_mut().unwrap().pop();
    reseal(&mut v);
    cases.push(("omitted-object", v));
    let mut checked = vec![];
    for (name, v) in cases {
        let error = import_small(&p, &v).unwrap_err();
        checked.push(json!({"name":name,"error":error.to_string()}));
    }
    let mut missing = p.clone();
    missing["snapshot_sources"].as_array_mut().unwrap().pop();
    assert!(import_small(&missing, &original).is_err());
    output(
        "snapshot-controls.json",
        &json!({"status":"passed","rejected":checked,"missing_raw_source_rejected":true}),
    );
}

fn capture(directory: &Path, p: &Value) -> GoSourceCaptureInputs {
    let selection = import_go_source_selection_inputs(
        &unhex(p["selection_hex"].as_str().unwrap()),
        &unhex(p["envelope_hex"].as_str().unwrap()),
        &unhex(p["components_hex"].as_str().unwrap()),
        &buffers(p, "sources", None),
        &buffers(p, "overlays", None),
    )
    .unwrap();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(p["snapshot_hex"].as_str().unwrap()),
        &buffers(p, "snapshot_sources", Some(directory)),
    )
    .unwrap();
    import_go_source_capture_inputs(
        &unhex(p["capture_hex"].as_str().unwrap()),
        selection,
        snapshot,
    )
    .unwrap()
}

#[test]
fn capture_inputs_bind_the_whole_original_sdk_namespace_and_contract_packet() {
    let directory = root().join("native-final/native-originals/owned");
    let p = read("native-final/native-originals/owned/capture-inputs.json");
    let input = capture(&directory, &p);
    assert!(input.snapshot_inputs().source_objects().len() > 5000);
    let contracts = input.selection_inputs().contract_inputs();
    assert_eq!(
        input
            .snapshot_inputs()
            .source(contracts.main_source_object_id())
            .unwrap()
            .object()
            .kind,
        "go_mod"
    );
    assert!(input
        .snapshot_inputs()
        .source_objects()
        .iter()
        .any(|o| o.kind == "excluded_source"));
    let module = input.derive_contracts().unwrap();
    assert_eq!(module.functions().len(), 2);
    assert_eq!(
        input.canonical_bytes(),
        unhex(p["capture_hex"].as_str().unwrap())
    );
    let mut altered: Value = serde_json::from_slice(input.canonical_bytes()).unwrap();
    altered["snapshot_inputs_sha256"] = json!("0".repeat(64));
    seal(&mut altered, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
    assert!(import_go_source_capture_inputs(
        &serde_json::to_vec(&altered).unwrap(),
        input.selection_inputs().clone(),
        input.snapshot_inputs().clone()
    )
    .is_err());
    output(
        "actual-owned-capture.json",
        &json!({"status":"passed","complete_original_objects":input.snapshot_inputs().source_objects().len(),"complete_contract_functions":2,"main_module_retained":true,"crossed_snapshot_rejected":true,"installed_context_or_program_authority":false}),
    );
}

#[test]
fn capture_inputs_preserve_both_complete_original_units_and_every_original_root() {
    let old: Value = serde_json::from_slice(
        &std::fs::read(root().parent().unwrap().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap();
    let mut rows = vec![];
    for name in ["pure", "wide"] {
        let directory = root().join("native/native-originals").join(name);
        let p = read(&format!(
            "native/native-originals/{name}/complete-capture-inputs.json"
        ));
        let input = capture(&directory, &p);
        let prior = old["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["name"] == name)
            .unwrap();
        assert_eq!(p["components_hex"], prior["input"]["components_hex"]);
        assert_eq!(p["sources"], prior["input"]["sources"]);
        assert_eq!(input.selection_inputs().roots().len(), 43);
        rows.push(json!({"unit":name,"original_roots":43,"complete_original_objects":input.snapshot_inputs().source_objects().len(),"all_source_components_and_bytes_equal":true}));
    }
    output(
        "all-original-capture-units.json",
        &json!({"status":"passed","units":rows,"original_zero_70_error_universes_preserved_by_original_consumer":true,"default_contracts_substituted_for_original_cases":false}),
    );
}

#[test]
fn capture_inputs_reject_coherently_rebound_target_excluded_original_helpers() {
    let directory = root().join("native-final/native-originals/owned");
    let p = read("native-final/native-originals/owned/capture-inputs.json");
    let original_sources = buffers(&p, "sources", None);
    let original_snapshot_sources = buffers(&p, "snapshot_sources", Some(&directory));
    let old = original_sources
        .iter()
        .find(|s| s.object().path == "helper/helper.go")
        .unwrap();
    let mut checked = vec![];
    for (name, path, prefix) in [
        ("windows-filename", "helper/helper_windows.go", ""),
        ("arm64-filename", "helper/helper_arm64.go", ""),
        ("test-filename", "helper/helper_test.go", ""),
        (
            "false-target-header",
            "helper/helper.go",
            "//go:build windows\n\n",
        ),
    ] {
        let mut raw = prefix.as_bytes().to_vec();
        raw.extend_from_slice(old.bytes());
        let changed =
            GoTypeSourceBuffer::new(&old.object().capability, path, "go_file", &raw).unwrap();
        let replace = |sources: &[GoTypeSourceBuffer]| {
            sources
                .iter()
                .map(|s| {
                    if s.object().id == old.object().id {
                        changed.clone()
                    } else {
                        s.clone()
                    }
                })
                .collect::<Vec<_>>()
        };
        let mut components: Value =
            serde_json::from_slice(&unhex(p["components_hex"].as_str().unwrap())).unwrap();
        for object in components["source_objects"].as_array_mut().unwrap() {
            if object["id"] == old.object().id {
                *object = serde_json::to_value(changed.object()).unwrap();
            }
        }
        components["source_objects"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        let mut package_change = None;
        for package in components["packages"].as_array_mut().unwrap() {
            if package["source_objects"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| id == &json!(old.object().id))
            {
                let previous = package["id"].as_str().unwrap().to_string();
                for id in package["source_objects"].as_array_mut().unwrap() {
                    if id == &json!(old.object().id) {
                        *id = json!(changed.object().id);
                    }
                }
                package["source_objects"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
                let mut value = package.take();
                value.as_object_mut().unwrap().remove("id");
                let parsed = parse_strict_json(
                    &serde_json::to_vec(&value).unwrap(),
                    StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
                )
                .unwrap();
                value["id"] = json!(hash_canonical_json(
                    HashDomain::new("MPK-GO-TYPE-PACKAGE-2.0"),
                    &parsed
                )
                .unwrap()
                .to_hex());
                package_change = Some((previous, value["id"].clone()));
                *package = value;
            }
        }
        let (previous, next) = package_change.unwrap();
        for id in components["package_roots"].as_array_mut().unwrap() {
            if id == &json!(previous) {
                *id = next.clone();
            }
        }
        components["packages"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        components["package_roots"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        seal(&mut components, "MPK-GO-TYPE-COMPONENTS-2.0");
        let mut contracts: Value =
            serde_json::from_slice(&unhex(p["envelope_hex"].as_str().unwrap())).unwrap();
        contracts["source_components_sha256"] = components["content_sha256"].clone();
        seal(&mut contracts, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0");
        let mut selected: Value =
            serde_json::from_slice(&unhex(p["selection_hex"].as_str().unwrap())).unwrap();
        selected["source_contract_inputs_sha256"] = contracts["content_sha256"].clone();
        for r in selected["roots"].as_array_mut().unwrap() {
            if r["package_id"] == previous {
                r["package_id"] = next.clone();
            }
        }
        seal(&mut selected, "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0");
        let selection = import_go_source_selection_inputs(
            &serde_json::to_vec(&selected).unwrap(),
            &serde_json::to_vec(&contracts).unwrap(),
            &serde_json::to_vec(&components).unwrap(),
            &replace(&original_sources),
            &buffers(&p, "overlays", None),
        )
        .unwrap();
        assert_eq!(
            selection.derive_contracts().unwrap().functions().len(),
            2,
            "same complete call/contract candidates"
        );
        let mut snapshot = wire(&p);
        for object in snapshot["objects"].as_array_mut().unwrap() {
            if object["id"] == old.object().id {
                *object = serde_json::to_value(changed.object()).unwrap();
            }
        }
        snapshot["objects"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        if path != old.object().path {
            let dir = old.object().path.rsplit_once('/').unwrap().0;
            let before = old
                .object()
                .path
                .rsplit('/')
                .next()
                .unwrap()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let after = path
                .rsplit('/')
                .next()
                .unwrap()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let d = snapshot["directories"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|d| d["capability"] == old.object().capability && d["path"] == dir)
                .unwrap();
            for e in d["entries"].as_array_mut().unwrap() {
                if e["name_hex"] == before {
                    e["name_hex"] = json!(after);
                }
            }
            d["entries"]
                .as_array_mut()
                .unwrap()
                .sort_by(|a, b| a["name_hex"].as_str().cmp(&b["name_hex"].as_str()));
        }
        reseal(&mut snapshot);
        let snapshot = import_go_source_snapshot_inputs(
            &serde_json::to_vec(&snapshot).unwrap(),
            &replace(&original_snapshot_sources),
        )
        .unwrap();
        assert_eq!(
            snapshot.source_objects().len(),
            original_snapshot_sources.len()
        );
        let mut record: Value =
            serde_json::from_slice(&unhex(p["capture_hex"].as_str().unwrap())).unwrap();
        record["selection_inputs_sha256"] = selected["content_sha256"].clone();
        record["snapshot_inputs_sha256"] = json!(snapshot.content_sha256());
        seal(&mut record, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
        let GoSourceSnapshotError::Build(error) = import_go_source_capture_inputs(
            &serde_json::to_vec(&record).unwrap(),
            selection,
            snapshot,
        )
        .unwrap_err() else {
            panic!("actual target exclusion: {name}")
        };
        assert_eq!(error.kind(), GoSourceBuildErrorKind::Excluded);
        assert_eq!(error.source_object_id(), changed.object().id);
        checked.push(json!({"case":name,"original_objects":original_snapshot_sources.len(),"complete_rebound_contract_functions":2,"error":error.to_string()}));
    }
    output(
        "coherent-target-exclusion-controls.json",
        &json!({"status":"passed","controls":checked,"complete_original_capture_retained":true,"source_program_proof":false}),
    );
}

#[test]
fn capture_inputs_reject_coherent_omission_of_an_actual_unselected_original_file() {
    let directory = root()
        .parent()
        .unwrap()
        .join("package-files/native/native-originals/owned");
    let p: Value = serde_json::from_slice(
        &std::fs::read(directory.join("complete-package-inputs.json")).unwrap(),
    )
    .unwrap();
    let original = capture(&directory, &p);
    assert_eq!(original.derive_contracts().unwrap().functions().len(), 2);
    let sources = buffers(&p, "sources", None);
    let omitted = sources
        .iter()
        .find(|s| s.object().path == "unselected.go")
        .unwrap();
    let mut components: Value =
        serde_json::from_slice(&unhex(p["components_hex"].as_str().unwrap())).unwrap();
    components["source_objects"]
        .as_array_mut()
        .unwrap()
        .retain(|s| s["id"] != omitted.object().id);
    let package = components["packages"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| {
            p["source_objects"]
                .as_array()
                .unwrap()
                .contains(&json!(omitted.object().id))
        })
        .unwrap();
    let old_package = package["id"].as_str().unwrap().to_string();
    package["source_objects"]
        .as_array_mut()
        .unwrap()
        .retain(|s| s != &json!(omitted.object().id));
    package.as_object_mut().unwrap().remove("id");
    let value = parse_strict_json(
        &serde_json::to_vec(package).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    let new_package = hash_canonical_json(HashDomain::new("MPK-GO-TYPE-PACKAGE-2.0"), &value)
        .unwrap()
        .to_hex();
    package["id"] = json!(new_package);
    for id in components["package_roots"].as_array_mut().unwrap() {
        if id == &json!(old_package) {
            *id = json!(new_package);
        }
    }
    components["packages"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    components["package_roots"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    seal(&mut components, "MPK-GO-TYPE-COMPONENTS-2.0");
    let mut contracts: Value =
        serde_json::from_slice(&unhex(p["envelope_hex"].as_str().unwrap())).unwrap();
    contracts["source_components_sha256"] = components["content_sha256"].clone();
    seal(&mut contracts, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0");
    let mut selected: Value =
        serde_json::from_slice(&unhex(p["selection_hex"].as_str().unwrap())).unwrap();
    selected["source_contract_inputs_sha256"] = contracts["content_sha256"].clone();
    for root in selected["roots"].as_array_mut().unwrap() {
        if root["package_id"] == old_package {
            root["package_id"] = json!(new_package);
        }
    }
    selected["roots"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["package_id"].as_str().cmp(&b["package_id"].as_str()));
    seal(&mut selected, "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0");
    let remaining = sources
        .iter()
        .filter(|s| s.object().id != omitted.object().id)
        .cloned()
        .collect::<Vec<_>>();
    let selection = import_go_source_selection_inputs(
        &serde_json::to_vec(&selected).unwrap(),
        &serde_json::to_vec(&contracts).unwrap(),
        &serde_json::to_vec(&components).unwrap(),
        &remaining,
        &buffers(&p, "overlays", None),
    )
    .unwrap();
    assert_eq!(selection.derive_contracts().unwrap().functions().len(), 2);
    let complete_snapshot = original.snapshot_inputs().clone();
    assert!(complete_snapshot.source(&omitted.object().id).is_some());
    let mut record: Value =
        serde_json::from_slice(&unhex(p["capture_hex"].as_str().unwrap())).unwrap();
    record["selection_inputs_sha256"] = selected["content_sha256"].clone();
    seal(&mut record, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
    let GoSourceSnapshotError::Package(error) = import_go_source_capture_inputs(
        &serde_json::to_vec(&record).unwrap(),
        selection,
        complete_snapshot,
    )
    .unwrap_err() else {
        panic!("actual complete Go-file membership guard")
    };
    assert_eq!(error.kind(), GoSourcePackageErrorKind::Membership);
    output(
        "coherent-unselected-file-omission.json",
        &json!({"status":"passed","omitted_original_object":omitted.object(),"complete_original_snapshot_objects":original.snapshot_inputs().source_objects().len(),"original_and_rebound_selected_contract_functions":2,"original_snapshot_unchanged":true,"error":error.to_string(),"source_or_context_proof":false}),
    );
}

#[test]
fn capture_inputs_reject_invalid_directives_in_an_original_cgo_disabled_file() {
    let directory = root()
        .parent()
        .unwrap()
        .join("cgo-directives/native/native-originals/owned");
    let p: Value = serde_json::from_slice(
        &std::fs::read(directory.join("complete-cgo-capture.json")).unwrap(),
    )
    .unwrap();
    let original = capture(&directory, &p);
    assert_eq!(original.derive_contracts().unwrap().functions().len(), 2);
    let sources = buffers(&p, "snapshot_sources", Some(&directory));
    let old = sources
        .iter()
        .find(|s| s.object().path == "cgo.go")
        .unwrap();
    assert_eq!(old.object().kind, "excluded_source");
    let raw = std::str::from_utf8(old.bytes())
        .unwrap()
        .replace("CFLAGS", "UNKNOWN");
    let changed = GoTypeSourceBuffer::new(
        &old.object().capability,
        &old.object().path,
        &old.object().kind,
        raw.as_bytes(),
    )
    .unwrap();
    let mut wire = wire(&p);
    for object in wire["objects"].as_array_mut().unwrap() {
        if object["id"] == old.object().id {
            *object = serde_json::to_value(changed.object()).unwrap();
        }
    }
    wire["objects"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    reseal(&mut wire);
    let revised = sources
        .iter()
        .map(|s| {
            if s.object().id == old.object().id {
                changed.clone()
            } else {
                s.clone()
            }
        })
        .collect::<Vec<_>>();
    let snapshot =
        import_go_source_snapshot_inputs(&serde_json::to_vec(&wire).unwrap(), &revised).unwrap();
    assert_eq!(
        snapshot.source_objects().len(),
        original.snapshot_inputs().source_objects().len()
    );
    let mut record: Value =
        serde_json::from_slice(&unhex(p["capture_hex"].as_str().unwrap())).unwrap();
    record["snapshot_inputs_sha256"] = json!(snapshot.content_sha256());
    seal(&mut record, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
    let selection = original.selection_inputs().clone();
    assert_eq!(selection.derive_contracts().unwrap().functions().len(), 2);
    let GoSourceSnapshotError::Package(error) =
        import_go_source_capture_inputs(&serde_json::to_vec(&record).unwrap(), selection, snapshot)
            .unwrap_err()
    else {
        panic!("actual cgo directive rejection")
    };
    assert_eq!(error.kind(), GoSourcePackageErrorKind::CgoDirective);
    assert_eq!(error.path(), "cgo.go");
    output(
        "coherent-excluded-cgo-directive.json",
        &json!({"status":"passed","complete_original_objects":sources.len(),"same_original_source_selection_and_two_contract_candidates":true,"original_excluded_source":old.object(),"changed_excluded_source":changed.object(),"error":error.to_string(),"cgo_enabled":false,"source_or_context_proof":false}),
    );
}
