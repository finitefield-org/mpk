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
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_EXPORT_OUTPUT") {
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

fn original() -> (Value, GoSourceCaptureInputs) {
    let directory = root().join("export-inputs/native/native-originals/owned");
    let packet = read(&directory.join("complete-original-export-inputs.json"));
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection(&packet),
        snapshot(&packet, Some(&directory)),
    )
    .unwrap();
    (packet, capture)
}

fn original_modules() -> (Value, GoSourceExportInputs) {
    original_modules_at("modules-owned")
}
fn original_modules_at(name: &str) -> (Value, GoSourceExportInputs) {
    let directory = root()
        .join("module-observations/native/native-originals")
        .join(name);
    let packet = read(&directory.join("complete-original-module-observations.json"));
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        selection(&packet),
        snapshot(&packet, Some(&directory)),
    )
    .unwrap();
    let exports = import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap();
    (packet, exports)
}

#[test]
fn whole_original_versioned_replacement_keeps_cache_escaping_and_archive_sources() {
    let (packet, exports) = original_modules_at("versioned-owned");
    let input = import_go_source_module_observations(
        &unhex(packet["module_observations_hex"].as_str().unwrap()),
        exports,
    )
    .unwrap();
    let manifests = input.module_manifest_sources();
    assert_eq!(manifests.len(), 2);
    assert_eq!(
        manifests["example.com/Dep"].path,
        "cache/download/example.com/!dep/@v/v1.2.0.mod"
    );
    assert_eq!(
        input
            .export_inputs()
            .capture_inputs()
            .derive_contracts()
            .unwrap()
            .functions()
            .len(),
        1
    );
    let snapshot = input.export_inputs().capture_inputs().snapshot_inputs();
    let original_cache = snapshot
        .source_objects()
        .iter()
        .filter(|o| o.path.starts_with("cache/download/example.com/!dep/@v/"))
        .collect::<Vec<_>>();
    assert_eq!(original_cache.len(), 4);
    assert!(original_cache.iter().any(|o| o.path.ends_with(".zip")));
    output(
        "whole-original-versioned-module-linkage.json",
        &json!({"status":"passed","manifests":manifests,"whole_original_cache_sources":original_cache,"metadata_hex":hex(input.original_metadata()),"diagnostics_hex":hex(input.original_diagnostics()),"independent_archive_checksum_or_MVS_verification":false}),
    );
}

#[test]
fn whole_original_module_graph_binds_replacements_and_every_package_owner() {
    let (packet, exports) = original_modules();
    let objects = exports
        .capture_inputs()
        .snapshot_inputs()
        .source_objects()
        .len();
    let input = import_go_source_module_observations(
        &unhex(packet["module_observations_hex"].as_str().unwrap()),
        exports,
    )
    .unwrap();
    let manifests = input.module_manifest_sources();
    assert_eq!(manifests.len(), 3);
    assert_eq!(manifests["example.com/alias"].path, "local-a/go.mod");
    assert_eq!(manifests["example.com/shared"].path, "local-b/go.mod");
    let parsed = input.derive_module_manifests().unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed["example.com/policy"].requirements().len(), 2);
    assert_eq!(
        parsed["example.com/alias"].original_source().object(),
        &manifests["example.com/alias"]
    );
    assert_eq!(manifests["example.com/policy"].path, "go.mod");
    assert_eq!(
        input
            .export_inputs()
            .capture_inputs()
            .derive_contracts()
            .unwrap()
            .functions()
            .len(),
        1
    );
    assert!(input
        .export_inputs()
        .export_sources()
        .contains_key("net/http"));
    assert!(input
        .export_inputs()
        .export_sources()
        .contains_key("example.com/alias"));
    assert!(input
        .export_inputs()
        .export_sources()
        .contains_key("example.com/shared"));
    output(
        "whole-original-module-linkage.json",
        &json!({"status":"passed","original_objects":objects,"manifests":manifests,"original_selected_contract_functions":1,"metadata_hex":hex(input.original_metadata()),"diagnostics_hex":hex(input.original_diagnostics()),"independent_MVS_checksum_root_or_context_authority":false}),
    );
}

#[test]
fn resealed_module_and_package_metadata_cannot_cross_original_manifest_owners() {
    let (packet, exports) = original_modules();
    let original: Value =
        serde_json::from_slice(&unhex(packet["module_observations_hex"].as_str().unwrap()))
            .unwrap();
    let original_export: Value =
        serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
    let mut observations = vec![];
    for name in [
        "unknown-field",
        "export-owner",
        "manifest-owner",
        "omitted-module",
        "duplicate-module",
        "unknown-native-field",
        "no-main",
        "main-name",
        "main-version",
        "source-directory",
        "source-manifest",
        "source-language",
        "replacement-path",
        "replacement-version",
        "replacement-chain",
        "replacement-wildcard",
        "package-module-path",
        "package-module-version",
        "package-module-directory",
        "package-module-main-type",
        "package-replacement-main",
        "package-replacement-path",
        "package-replacement-directory",
    ] {
        let mut wire = original.clone();
        let mut export_wire = original_export.clone();
        let mut rows = native_rows(&wire);
        let main = rows.iter().position(|r| r["Main"] == true).unwrap();
        let alias = rows
            .iter()
            .position(|r| r["Path"] == "example.com/alias")
            .unwrap();
        let selected = wire["modules"]
            .as_array()
            .unwrap()
            .iter()
            .position(|r| r["path"] == "example.com/alias")
            .unwrap();
        match name {
            "unknown-field" => wire["unexpected"] = json!(true),
            "export-owner" => wire["source_export_inputs_sha256"] = json!("f".repeat(64)),
            "manifest-owner" => {
                wire["modules"][selected]["manifest_source_object_id"] =
                    wire["modules"][2]["manifest_source_object_id"].clone()
            }
            "omitted-module" => {
                rows.remove(alias);
                wire["modules"].as_array_mut().unwrap().remove(selected);
            }
            "duplicate-module" => rows.push(rows[alias].clone()),
            "unknown-native-field" => rows[alias]["Unmodeled"] = json!(1),
            "no-main" => rows[main]["Main"] = json!(false),
            "main-name" => rows[main]["Path"] = json!("example.com/other"),
            "main-version" => rows[main]["Version"] = json!("v1.0.0"),
            "source-directory" => rows[alias]["Dir"] = json!("/another/source"),
            "source-manifest" => rows[alias]["GoMod"] = rows[main]["GoMod"].clone(),
            "source-language" => {
                rows[alias]["GoVersion"] = json!("1.24");
                rows[alias]["Replace"]["GoVersion"] = json!("1.24");
            }
            "replacement-path" | "replacement-wildcard" => {
                let path = if name == "replacement-wildcard" {
                    "./wildcard"
                } else {
                    "./local-b"
                };
                rows[alias]["Replace"]["Path"] = json!(path);
                wire["modules"][selected]["replacement_path"] = json!(path);
            }
            "replacement-version" => {
                rows[alias]["Replace"]["Version"] = json!("v1.0.0");
                wire["modules"][selected]["replacement_version"] = json!("v1.0.0");
            }
            "replacement-chain" => {
                rows[alias]["Replace"]["Replace"] = rows[alias]["Replace"].clone()
            }
            _ => {
                let mut packages = native_rows(&export_wire);
                let package = packages
                    .iter_mut()
                    .find(|r| r["ImportPath"] == "example.com/alias")
                    .unwrap();
                match name {
                    "package-module-path" => {
                        package["Module"]["Path"] = json!("example.com/shared")
                    }
                    "package-module-version" => package["Module"]["Version"] = json!("v1.1.0"),
                    "package-module-main-type" => package["Module"]["Main"] = json!(1),
                    "package-replacement-main" => {
                        package["Module"]["Replace"]["Main"] = json!(true)
                    }
                    "package-module-directory" => package["Module"]["Dir"] = json!("/another/root"),
                    "package-replacement-path" => {
                        package["Module"]["Replace"]["Path"] = json!("./wildcard")
                    }
                    "package-replacement-directory" => {
                        package["Module"]["Replace"]["Dir"] = json!("/another/root")
                    }
                    _ => unreachable!(),
                }
                set_rows(&mut export_wire, &packages);
                seal(&mut export_wire, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
                wire["source_export_inputs_sha256"] = export_wire["content_sha256"].clone();
            }
        }
        set_rows(&mut wire, &rows);
        seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0");
        let changed_exports = if export_wire == original_export {
            exports.clone()
        } else {
            import_go_source_export_inputs(
                &serde_json::to_vec(&export_wire).unwrap(),
                exports.capture_inputs().clone(),
            )
            .unwrap()
        };
        let error = import_go_source_module_observations(
            &serde_json::to_vec(&wire).unwrap(),
            changed_exports,
        )
        .unwrap_err();
        observations.push(json!({"case":name,"error":format!("{error:?}"),"complete_changed_module_observations_hex":hex(&serde_json::to_vec(&wire).unwrap()),"complete_export_inputs_hex":hex(&serde_json::to_vec(&export_wire).unwrap()),"original_capture_and_one_Root_contract_unchanged":true}));
    }
    // A JSON value decoder would erase this duplicate before projection.
    let mut wire = original;
    let raw = unhex(wire["metadata_hex"].as_str().unwrap());
    let start = raw.iter().position(|b| *b == b'{').unwrap();
    let mut duplicate = raw[..start + 1].to_vec();
    duplicate.extend_from_slice(b"\"Path\":\"discarded.module\",");
    duplicate.extend_from_slice(&raw[start + 1..]);
    wire["metadata_hex"] = json!(hex(&duplicate));
    seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0");
    assert_eq!(
        import_go_source_module_observations(&serde_json::to_vec(&wire).unwrap(), exports)
            .unwrap_err(),
        GoSourceModuleObservationError::Metadata
    );
    observations.push(json!({"case":"duplicate-native-field","complete_changed_module_observations_hex":hex(&serde_json::to_vec(&wire).unwrap()),"original_capture_and_one_Root_contract_unchanged":true}));
    output(
        "coherent-original-module-controls.json",
        &json!({"status":"passed","observations":observations}),
    );
}
#[test]
fn whole_original_compiler_metadata_and_exports_bind_to_every_source_package() {
    let (packet, capture) = original();
    let count = capture.snapshot_inputs().source_objects().len();
    let input = import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap();
    assert_eq!(
        input
            .capture_inputs()
            .derive_contracts()
            .unwrap()
            .functions()
            .len(),
        1
    );
    assert!(input.export_sources().contains_key("net/http"));
    assert!(input
        .export_sources()
        .keys()
        .any(|name| name.starts_with("vendor/")));
    assert!(!input.export_sources().contains_key("unsafe"));
    let rows=input.export_sources().iter().map(|(package,obj)|json!({"package":package,"object":obj,"bytes":input.capture_inputs().snapshot_inputs().source(&obj.id).unwrap().bytes().len()})).collect::<Vec<_>>();
    output(
        "whole-original-export-linkage.json",
        &json!({"status":"passed","original_objects":count,"original_selected_contract_functions":1,"exports":rows,"metadata_hex":hex(input.original_metadata()),"diagnostics_hex":hex(input.original_diagnostics()),"original_package_initialization_or_unused_http_body_executed":false,"source_or_context_program_proof":false}),
    );
}
fn native_rows(wire: &Value) -> Vec<Value> {
    serde_json::Deserializer::from_slice(&unhex(wire["metadata_hex"].as_str().unwrap()))
        .into_iter()
        .map(|r| r.unwrap())
        .collect()
}
fn set_rows(wire: &mut Value, rows: &[Value]) {
    let mut raw = vec![];
    for r in rows {
        raw.extend(serde_json::to_vec(r).unwrap());
        raw.push(b'\n');
    }
    wire["metadata_hex"] = json!(hex(&raw));
}
#[test]
fn resealed_native_metadata_cannot_replace_original_source_or_export_bindings() {
    let (packet, capture) = original();
    let original: Value =
        serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
    let mut checked = vec![];
    for name in [
        "unknown-field",
        "capture-owner",
        "image-owner",
        "compiler-owner",
        "selected-root",
        "source-name",
        "source-list",
        "compiled-source",
        "source-directory",
        "extra-root",
        "failed-package",
        "cgo-package",
        "import-closure",
        "import-remap",
        "resource-list",
        "resource-pattern",
        "archive-build-id",
        "export-buffer",
        "omitted-package",
        "duplicate-metadata",
    ] {
        let mut wire = original.clone();
        let mut rows = native_rows(&wire);
        let at = rows
            .iter()
            .position(|r| r["ImportPath"] == "example.com/policy")
            .unwrap();
        match name {
            "unknown-field" => {
                wire["unchecked_adapter"] = json!(true);
            }
            "capture-owner" => wire["source_capture_inputs_sha256"] = json!("00".repeat(32)),
            "image-owner" => wire["image_sha256"] = json!("00".repeat(32)),
            "compiler-owner" => wire["compiler_sha256"] = json!("00".repeat(32)),
            "selected-root" => wire["requested_package"] = json!("net/http"),
            "source-name" => rows[at]["Name"] = json!("replaced"),
            "source-list" => rows[at]["GoFiles"] = json!([]),
            "compiled-source" => rows[at]["CompiledGoFiles"] = json!(["generated.go"]),
            "source-directory" => rows[at]["Dir"] = json!("/replaced-original"),
            "extra-root" => {
                let dependency = rows
                    .iter_mut()
                    .find(|r| r["ImportPath"] == "net/http")
                    .unwrap();
                dependency["DepOnly"] = json!(false);
            }
            "failed-package" => rows[at]["Incomplete"] = json!(true),
            "cgo-package" => rows[at]["CgoFiles"] = json!(["generated.go"]),
            "import-closure" => rows[at]["Imports"] = json!([]),
            "import-remap" => rows[at]["ImportMap"] = json!({"embed":"net/http"}),
            "resource-list" => rows[at]["EmbedFiles"] = json!([]),
            "resource-pattern" => rows[at]["EmbedPatterns"] = json!(["missing"]),
            "archive-build-id" => rows[at]["BuildID"] = json!("crossed-original-build"),
            "export-buffer" => {
                wire["packages"][0]["export_source_object_id"] =
                    wire["packages"][1]["export_source_object_id"].clone()
            }
            "omitted-package" => {
                wire["packages"].as_array_mut().unwrap().pop();
                rows.pop();
            }
            "duplicate-metadata" => rows.push(rows[at].clone()),
            _ => unreachable!(),
        }
        set_rows(&mut wire, &rows);
        seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
        let error =
            import_go_source_export_inputs(&serde_json::to_vec(&wire).unwrap(), capture.clone())
                .unwrap_err();
        checked.push(json!({"case":name,"complete_changed_export_inputs_hex":hex(&serde_json::to_vec(&wire).unwrap()),"error":error.to_string()}));
    }
    // A duplicate native field must reject before typed JSON can erase it.
    let mut wire = original;
    let raw = unhex(wire["metadata_hex"].as_str().unwrap());
    let start = raw.iter().position(|b| *b == b'{').unwrap();
    let mut changed = raw[..start + 1].to_vec();
    changed.extend_from_slice(b"\"ImportPath\":\"duplicate\",");
    changed.extend_from_slice(&raw[start + 1..]);
    wire["metadata_hex"] = json!(hex(&changed));
    seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
    let error =
        import_go_source_export_inputs(&serde_json::to_vec(&wire).unwrap(), capture).unwrap_err();
    checked.push(json!({"case":"duplicate-native-field","complete_changed_export_inputs_hex":hex(&serde_json::to_vec(&wire).unwrap()),"error":error.to_string()}));
    output(
        "coherent-native-export-controls.json",
        &json!({"status":"passed","cases":checked,"original_input_unchanged":true,"source_or_context_program_proof":false}),
    );
}
#[test]
fn coherently_resealed_export_archives_cannot_change_target_build_id_or_bounds() {
    let directory = root().join("export-inputs/native/native-originals/owned");
    let (packet, original) = original();
    let all = buffers(&packet, "snapshot_sources", Some(&directory));
    let export_record: Value =
        serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
    let source_id = export_record["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["import_path"] == "example.com/policy")
        .unwrap()["export_source_object_id"]
        .as_str()
        .unwrap();
    let source = all.iter().find(|s| s.object().id == source_id).unwrap();
    let mut checked = vec![];
    for name in [
        "magic",
        "member-size",
        "compiler-target",
        "padding-or-trailer",
    ] {
        let mut raw = source.bytes().to_vec();
        match name {
            "magic" => raw[0] ^= 1,
            "member-size" => raw[8 + 48..8 + 58].fill(b'9'),
            "compiler-target" => {
                let at = raw
                    .windows(b"linux amd64".len())
                    .position(|s| s == b"linux amd64")
                    .unwrap();
                raw[at..at + 5].copy_from_slice(b"other");
            }
            "padding-or-trailer" => raw.push(1),
            _ => unreachable!(),
        }
        let object = source.object();
        let changed =
            GoTypeSourceBuffer::new(&object.capability, &object.path, &object.kind, &raw).unwrap();
        let sources = all
            .iter()
            .map(|s| {
                if s.object().id == object.id {
                    changed.clone()
                } else {
                    s.clone()
                }
            })
            .collect::<Vec<_>>();
        let mut wire: Value =
            serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
        for row in wire["objects"].as_array_mut().unwrap() {
            if row["id"] == object.id {
                *row = serde_json::to_value(changed.object()).unwrap();
            }
        }
        wire["objects"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        wire["snapshot_sha256"] = json!(native_hash(&wire));
        seal(&mut wire, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
        let snapshot =
            import_go_source_snapshot_inputs(&serde_json::to_vec(&wire).unwrap(), &sources)
                .unwrap();
        let mut capture: Value =
            serde_json::from_slice(&unhex(packet["capture_hex"].as_str().unwrap())).unwrap();
        capture["snapshot_inputs_sha256"] = json!(snapshot.content_sha256());
        seal(&mut capture, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
        let captured = import_go_source_capture_inputs(
            &serde_json::to_vec(&capture).unwrap(),
            original.selection_inputs().clone(),
            snapshot,
        )
        .unwrap();
        let mut exported = export_record.clone();
        exported["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
        for pkg in exported["packages"].as_array_mut().unwrap() {
            if pkg["export_source_object_id"] == object.id {
                pkg["export_source_object_id"] = json!(changed.object().id);
            }
        }
        seal(&mut exported, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
        let error =
            import_go_source_export_inputs(&serde_json::to_vec(&exported).unwrap(), captured)
                .unwrap_err();
        assert_eq!(error, GoSourceExportInputError::Archive);
        checked.push(json!({"case":name,"original_object":object,"changed_object":changed.object(),"complete_changed_archive_hex":hex(&raw),"changed_snapshot_hex":hex(&serde_json::to_vec(&wire).unwrap()),"changed_capture_hex":hex(&serde_json::to_vec(&capture).unwrap()),"changed_export_hex":hex(&serde_json::to_vec(&exported).unwrap()),"same_original_source_selection_and_root_contract":true,"error":error.to_string()}));
    }
    output(
        "coherent-original-archive-controls.json",
        &json!({"status":"passed","original_objects":all.len(),"cases":checked,"source_or_context_program_proof":false}),
    );
}
