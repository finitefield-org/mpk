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
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_COMPILER_INPUTS_OUTPUT") {
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
fn seal(value: &mut Value, domain: &'static str) {
    value.as_object_mut().unwrap().remove("content_sha256");
    let parsed = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value["content_sha256"] = json!(hash_canonical_json(HashDomain::new(domain), &parsed)
        .unwrap()
        .to_hex());
}
fn buffers(packet: &Value, field: &str, dir: Option<&Path>) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let raw = if let Some(raw) = r["raw_hex"].as_str() {
                unhex(raw)
            } else {
                std::fs::read(
                    dir.unwrap()
                        .join("original-source-objects")
                        .join(r["raw_file"].as_str().unwrap()),
                )
                .unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&r["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}
fn import_exports(packet: &Value, dir: &Path) -> GoSourceExportInputs {
    let select = import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(packet, "sources", None),
        &buffers(packet, "overlays", None),
    )
    .unwrap();
    let snapshot = import_go_source_snapshot_inputs(
        &unhex(packet["snapshot_hex"].as_str().unwrap()),
        &buffers(packet, "snapshot_sources", Some(dir)),
    )
    .unwrap();
    let capture = import_go_source_capture_inputs(
        &unhex(packet["capture_hex"].as_str().unwrap()),
        select,
        snapshot,
    )
    .unwrap();
    import_go_source_export_inputs(
        &unhex(packet["export_inputs_hex"].as_str().unwrap()),
        capture,
    )
    .unwrap()
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

fn original() -> (
    Value,
    GoSourceExportInputs,
    Value,
    Vec<GoCompilerFileBuffer>,
) {
    let dir = root().join("compiler-inputs/native/native-originals/versioned-owned");
    let packet = read(&dir.join("complete-original-module-graph-inputs.json"));
    let exports = import_exports(&packet, &dir);
    let wire =
        serde_json::from_slice(&unhex(packet["compiler_inputs_hex"].as_str().unwrap())).unwrap();
    let files = packet["compiler_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let raw = std::fs::read(
                dir.join("original-compiler-files")
                    .join(row["raw_file"].as_str().unwrap()),
            )
            .unwrap();
            let file = GoCompilerFileBuffer::import(
                row["entry"]["path"].as_str().unwrap(),
                row["entry"]["executable"].as_bool().unwrap(),
                &raw,
            )
            .unwrap();
            assert_eq!(serde_json::to_value(file.entry()).unwrap(), row["entry"]);
            file
        })
        .collect();
    (packet, exports, wire, files)
}
#[test]
fn whole_original_prepared_compiler_bytes_bind_every_sdk_source_and_component() {
    let (packet, exports, wire, files) = original();
    let raw = unhex(packet["compiler_inputs_hex"].as_str().unwrap());
    let input = import_go_source_compiler_inputs(&raw, exports, files).unwrap();
    assert_eq!(input.canonical_bytes(), raw);
    assert_eq!(
        input.files().len(),
        wire["inventory"]["files"].as_array().unwrap().len()
    );
    assert!(input.files().iter().any(|f| f.bytes().len() > 16777216));
    assert!(input.sdk_source_files().len() > 5000);
    assert_eq!(packet["snapshot_sources"].as_array().unwrap().len(), 5697);
    assert_eq!(input.executable_build_info().len(), 9);
    for (path, info) in input.executable_build_info() {
        info.validate_compiler_tool(path.rsplit('/').next().unwrap(), input.host())
            .unwrap();
    }
    let projection: Value =
        serde_json::from_slice(input.export_inputs().canonical_bytes()).unwrap();
    assert_eq!(projection["packages"].as_array().unwrap().len(), 182);
    let total = input.files().iter().map(|f| f.bytes().len()).sum::<usize>();
    output(
        "whole-original-compiler-import.json",
        &json!({"status":"passed","compiler_inputs":wire,"files":input.files().len(),"bytes":total,"files_larger_than_application_source_limit":input.files().iter().filter(|f|f.bytes().len()>16777216).count(),"sdk_sources":input.sdk_source_files().len(),"original_objects":5697,"original_packages":182,"host":input.host(),"compiler_file_hashes":input.files().iter().map(|f|f.entry()).collect::<Vec<_>>(),"independently_bound_executables":input.executable_build_info().len(),"installed_or_compiler_correctness_authority":false}),
    );
}

#[test]
fn coherent_original_executable_changes_require_actual_build_identity() {
    let (original_packet, _, original_wire, original_files) = original();
    let dir = root().join("compiler-inputs/native/native-originals/versioned-owned");
    let index = original_files
        .iter()
        .position(|f| f.entry().path == "go/bin/gofmt")
        .unwrap();
    let hash = |value: &Value| {
        let parsed = parse_strict_json(
            &serde_json::to_vec(value).unwrap(),
            StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
        )
        .unwrap();
        hash_canonical_json(HashDomain::new("MPK-BUNDLE-CONTENT-0.1"), &parsed)
            .unwrap()
            .to_hex()
    };
    for (name, before, after) in [
        ("release", b"go1.25.0".as_slice(), b"go1.24.0".as_slice()),
        ("tool", b"path\tcmd/gofmt\n", b"path\tcmd/wrong\n"),
        ("cgo", b"build\tCGO_ENABLED=0\n", b"build\tCGO_ENABLED=1\n"),
    ] {
        let mut raw = original_files[index].bytes().to_vec();
        let build_info = raw
            .windows(14)
            .position(|s| s == b"\xff Go buildinf:")
            .unwrap();
        let offset = if name == "release" {
            build_info + 33
        } else {
            // Runtime moduledata also embeds a copy in rodata. Mutate the
            // actual build-info record, keeping the whole original executable.
            build_info
                + raw[build_info..]
                    .windows(before.len())
                    .position(|s| s == before)
                    .unwrap()
        };
        assert_eq!(&raw[offset..offset + before.len()], before);
        raw[offset..offset + after.len()].copy_from_slice(after);
        let changed = GoCompilerFileBuffer::import("go/bin/gofmt", true, &raw).unwrap();
        let info = read_go_executable_build_info(&raw).unwrap();
        assert!(info
            .validate_compiler_tool("gofmt", original_wire["host"].as_str().unwrap())
            .is_err());
        let mut files = original_files.clone();
        files[index] = changed.clone();
        let mut wire = original_wire.clone();
        let entry = serde_json::to_value(changed.entry()).unwrap();
        wire["inventory"]["files"][index] = entry.clone();
        let component = wire["components"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["identity"]["name"] == "gofmt")
            .unwrap();
        component["inventory"]["files"][0] = entry;
        component["identity"]["binary_sha256"] = json!(changed.entry().sha256);
        let identity = component["identity"].clone();
        *wire["toolchain"]["components"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["name"] == "gofmt")
            .unwrap() = identity;
        wire["toolchain"]["distribution_sha256"] = json!(hash(&wire["inventory"]));
        let mut packet = original_packet.clone();
        let mut envelope: Value =
            serde_json::from_slice(&unhex(packet["envelope_hex"].as_str().unwrap())).unwrap();
        envelope["compiler_sha256"] = wire["toolchain"]["distribution_sha256"].clone();
        seal(&mut envelope, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0");
        packet["envelope_hex"] = json!(hex(&serde_json::to_vec(&envelope).unwrap()));
        let mut selection: Value =
            serde_json::from_slice(&unhex(packet["selection_hex"].as_str().unwrap())).unwrap();
        selection["source_contract_inputs_sha256"] = envelope["content_sha256"].clone();
        seal(&mut selection, "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0");
        packet["selection_hex"] = json!(hex(&serde_json::to_vec(&selection).unwrap()));
        let mut capture: Value =
            serde_json::from_slice(&unhex(packet["capture_hex"].as_str().unwrap())).unwrap();
        capture["selection_inputs_sha256"] = selection["content_sha256"].clone();
        seal(&mut capture, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
        packet["capture_hex"] = json!(hex(&serde_json::to_vec(&capture).unwrap()));
        let mut export: Value =
            serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
        export["compiler_sha256"] = wire["toolchain"]["distribution_sha256"].clone();
        export["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
        seal(&mut export, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
        packet["export_inputs_hex"] = json!(hex(&serde_json::to_vec(&export).unwrap()));
        wire["source_export_inputs_sha256"] = export["content_sha256"].clone();
        seal(&mut wire, "MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0");
        // Every original source, dependency, unused body/init and Root contract
        // remains byte-for-byte present. All metadata links and compiler file/
        // component/distribution hashes match the actual changed executable.
        for field in [
            "sources",
            "overlays",
            "components_hex",
            "snapshot_hex",
            "snapshot_sources",
        ] {
            assert_eq!(packet[field], original_packet[field]);
        }
        let exports = import_exports(&packet, &dir);
        let error =
            import_go_source_compiler_inputs(&serde_json::to_vec(&wire).unwrap(), exports, files)
                .unwrap_err();
        assert_eq!(error, GoSourceCompilerInputError::BuildInfo, "{name}");
        if let Some(out) = std::env::var_os("MPK_GO_COMPILER_INPUTS_OUTPUT") {
            let out = PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join(format!("changed-{name}.raw")), &raw).unwrap();
        }
        output(
            &format!("coherent-executable-{name}.json"),
            &json!({"status":"rejected","error":format!("{error:?}"),"changed_entry":changed.entry(),"byte_offset":offset,"before_hex":hex(before),"after_hex":hex(after),"original_objects":5697,"original_packages":182,"whole_original_source_and_Root_contract_unchanged":true,"envelope":envelope,"selection":selection,"capture":capture,"exports":export,"compiler_inputs":wire}),
        );
    }
}
#[test]
fn coherently_sealed_compiler_claims_require_whole_actual_bundle_membership() {
    let (_, exports, original, files) = original();
    let mut observed = vec![];
    for case in [
        "schema",
        "unknown",
        "export-link",
        "sdk-capability",
        "original-root",
        "host",
        "root-scope",
        "inventory-order",
        "inventory-case-collision",
        "inventory-escape",
        "inventory-negative-size",
        "inventory-oversized-file",
        "inventory-mode",
        "distribution",
        "component-scope",
        "component-release",
        "component-commit",
        "component-binary",
        "content-digest",
        "missing-component",
        "component-overlap",
        "component-missing-file",
        "projection-order",
        "projection-missing",
        "missing-file-buffer",
        "extra-file-buffer",
        "duplicate-file-buffer",
        "reordered-file-buffers",
        "changed-file-bytes",
        "changed-file-mode",
    ] {
        let mut wire = original.clone();
        let mut provided = files.clone();
        let executable = wire["components"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c["identity"]["kind"] == "executable")
            .unwrap();
        let content = wire["components"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c["identity"]["kind"] == "content")
            .unwrap();
        match case {
            "schema" => wire["schema"] = json!("mpk.go.practical.caller_compiler.v1"),
            "unknown" => wire["caller_authority"] = json!(true),
            "export-link" => wire["source_export_inputs_sha256"] = json!("0".repeat(64)),
            "sdk-capability" => wire["sdk_capability"] = json!("workspace/main"),
            "original-root" => wire["original_root"] = json!("/wrong/compiler/root"),
            "host" => {
                wire["host"] = json!(if original["host"] == "linux_amd64" {
                    "darwin_arm64"
                } else {
                    "linux_amd64"
                })
            }
            "root-scope" => wire["inventory"]["scope"]["bundle_id"] = json!("caller.bundle"),
            "inventory-order" => wire["inventory"]["files"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            "inventory-case-collision" => {
                wire["inventory"]["files"][1]["path"] = json!(wire["inventory"]["files"][0]["path"]
                    .as_str()
                    .unwrap()
                    .to_ascii_uppercase())
            }
            "inventory-escape" => wire["inventory"]["files"][0]["path"] = json!("go/../outside"),
            "inventory-negative-size" => wire["inventory"]["files"][0]["size_bytes"] = json!(-1),
            "inventory-oversized-file" => {
                wire["inventory"]["files"][0]["size_bytes"] = json!(4294967297u64)
            }
            "inventory-mode" => wire["inventory"]["files"][0]["executable"] = json!(true),
            "distribution" => wire["toolchain"]["distribution_sha256"] = json!("0".repeat(64)),
            "component-scope" => {
                wire["components"][content]["inventory"]["scope"]["component_name"] =
                    json!("wrong.component")
            }
            "component-release" => {
                wire["components"][executable]["identity"]["release"] = json!("go1.25.1")
            }
            "component-commit" => {
                wire["components"][executable]["identity"]["commit_hash"] = json!("caller")
            }
            "component-binary" => {
                wire["components"][executable]["identity"]["binary_sha256"] = json!("0".repeat(64))
            }
            "content-digest" => {
                wire["components"][content]["identity"]["content_sha256"] = json!("0".repeat(64))
            }
            "missing-component" => {
                wire["components"].as_array_mut().unwrap().remove(content);
            }
            "component-overlap" => {
                let extra = wire["components"][executable]["inventory"]["files"][0].clone();
                wire["components"][content]["inventory"]["files"]
                    .as_array_mut()
                    .unwrap()
                    .push(extra);
            }
            "component-missing-file" => {
                wire["components"][content]["inventory"]["files"]
                    .as_array_mut()
                    .unwrap()
                    .remove(0);
                // Correct both derived content references, retaining the actual
                // full distribution and all provided original file buffers.
                let inventory = wire["components"][content]["inventory"].clone();
                let parsed = parse_strict_json(
                    &serde_json::to_vec(&inventory).unwrap(),
                    StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
                )
                .unwrap();
                let digest =
                    hash_canonical_json(HashDomain::new("MPK-BUNDLE-CONTENT-0.1"), &parsed)
                        .unwrap()
                        .to_hex();
                wire["components"][content]["identity"]["content_sha256"] = json!(digest);
                let name = wire["components"][content]["identity"]["name"].clone();
                let index = wire["toolchain"]["components"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|c| c["name"] == name)
                    .unwrap();
                wire["toolchain"]["components"][index] =
                    wire["components"][content]["identity"].clone();
            }
            "projection-order" => wire["toolchain"]["components"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            "projection-missing" => {
                wire["toolchain"]["components"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "missing-file-buffer" => {
                provided.pop();
            }
            "extra-file-buffer" => provided.push(
                GoCompilerFileBuffer::import("go/unowned", false, b"original extra bytes").unwrap(),
            ),
            "duplicate-file-buffer" => provided.push(provided[0].clone()),
            "reordered-file-buffers" => provided.swap(0, 1),
            "changed-file-bytes" | "changed-file-mode" => {
                let index = provided
                    .iter()
                    .position(|f| f.entry().path == "go/bin/go")
                    .unwrap();
                let old = &provided[index];
                let mut bytes = old.bytes().to_vec();
                if case == "changed-file-bytes" {
                    bytes[0] ^= 1;
                }
                provided[index] = GoCompilerFileBuffer::import(
                    &old.entry().path,
                    if case == "changed-file-mode" {
                        !old.entry().executable
                    } else {
                        old.entry().executable
                    },
                    &bytes,
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        seal(&mut wire, "MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0");
        let error = match import_go_source_compiler_inputs(
            &serde_json::to_vec(&wire).unwrap(),
            exports.clone(),
            provided,
        ) {
            Err(e) => e,
            Ok(_) => panic!("invalid compiler accepted: {case}"),
        };
        observed.push(json!({"case":case,"complete_changed_record":wire,"error":error.to_string(),"whole_original_capture_and_Root_contract_preserved":true,"original_compiler_files":files.len()}));
    }
    output(
        "coherent-original-compiler-controls.json",
        &json!({"status":"passed","cases":observed}),
    );
}
#[test]
fn coherently_changed_unused_sdk_source_must_match_original_compiler_bytes() {
    let (mut packet, _, mut wire, files) = original();
    let dir = root().join("compiler-inputs/native/native-originals/versioned-owned");
    let row = packet["snapshot_sources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| {
            r["object"]["capability"] == "original-go-sdk"
                && r["object"]["path"] == "src/cmd/compile/internal/ssa/poset.go"
        })
        .unwrap();
    let old = row["object"].clone();
    let mut changed = std::fs::read(
        dir.join("original-source-objects")
            .join(row["raw_file"].as_str().unwrap()),
    )
    .unwrap();
    changed.extend_from_slice(b"\n// exact independent compiler-source content control\n");
    row["raw_hex"] = json!(hex(&changed));
    let mut object = old.clone();
    object["bytes"] = json!(changed.len());
    object["raw_sha256"] = json!(format!("{:x}", Sha256::digest(&changed)));
    object.as_object_mut().unwrap().remove("id");
    let strict = parse_strict_json(
        &serde_json::to_vec(&object).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    object["id"] = json!(
        hash_canonical_json(HashDomain::new("MPK-GO-SOURCE-OBJECT-1.0"), &strict)
            .unwrap()
            .to_hex()
    );
    row["object"] = object.clone();
    let mut snapshot: Value =
        serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
    for current in snapshot["objects"].as_array_mut().unwrap() {
        if current["id"] == old["id"] {
            *current = object.clone();
        }
    }
    snapshot["objects"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|o| o["id"].as_str().unwrap().to_string());
    snapshot["snapshot_sha256"] = json!(native_hash(&snapshot));
    seal(&mut snapshot, "MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
    packet["snapshot_hex"] = json!(hex(&serde_json::to_vec(&snapshot).unwrap()));
    let mut capture: Value =
        serde_json::from_slice(&unhex(packet["capture_hex"].as_str().unwrap())).unwrap();
    capture["snapshot_inputs_sha256"] = snapshot["content_sha256"].clone();
    seal(&mut capture, "MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
    packet["capture_hex"] = json!(hex(&serde_json::to_vec(&capture).unwrap()));
    let mut export: Value =
        serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
    export["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
    seal(&mut export, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
    packet["export_inputs_hex"] = json!(hex(&serde_json::to_vec(&export).unwrap()));
    let exports = import_exports(&packet, &dir);
    wire["source_export_inputs_sha256"] = export["content_sha256"].clone();
    seal(&mut wire, "MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0");
    let error =
        match import_go_source_compiler_inputs(&serde_json::to_vec(&wire).unwrap(), exports, files)
        {
            Err(e) => e,
            Ok(_) => panic!("changed unselected SDK bytes accepted"),
        };
    assert!(matches!(error, GoSourceCompilerInputError::Source));
    output(
        "coherent-original-sdk-content-control.json",
        &json!({"status":"passed","changed_raw_hex":hex(&changed),"changed_object":object,"changed_snapshot_hex":packet["snapshot_hex"],"changed_capture_hex":packet["capture_hex"],"changed_export_hex":packet["export_inputs_hex"],"changed_compiler_record":wire,"error":error.to_string(),"original_application_source_and_Root_contract_unchanged":true,"original_objects":packet["snapshot_sources"].as_array().unwrap().len(),"sdk_source_was_unused_by_application_packages":true}),
    );
}
#[test]
fn compiler_file_buffers_own_bytes_and_keep_existing_portable_bundle_paths() {
    let mut raw = vec![0, 255, 42];
    let file = GoCompilerFileBuffer::import("go/original.bytes", false, &raw).unwrap();
    raw.fill(9);
    assert_eq!(file.bytes(), [0, 255, 42]);
    for bad in [
        "",
        "/go/a",
        "go/../a",
        "go//a",
        "go/a/",
        "go/AUX.txt",
        "go/COM1",
        "go/a.",
        "go/a:b",
        "go/é",
        "go/a b",
    ] {
        assert!(
            GoCompilerFileBuffer::import(bad, false, b"").is_err(),
            "{bad}"
        );
    }
    assert!(GoCompilerFileBuffer::import("go/convalid", false, b"").is_ok());
}
