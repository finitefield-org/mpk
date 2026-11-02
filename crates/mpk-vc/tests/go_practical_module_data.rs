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
    if let Some(dir) = std::env::var_os("MPK_GO_MODULE_DATA_OUTPUT") {
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

fn oracle(name: &str) -> Value {
    read(
        &root()
            .join("module-data/native/native-originals/module-data-oracle")
            .join(name),
    )
}
fn original_modules(name: &str) -> (Value, GoSourceModuleObservations) {
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
    let observations = import_go_source_module_observations(
        &unhex(packet["module_observations_hex"].as_str().unwrap()),
        exports,
    )
    .unwrap();
    (packet, observations)
}
#[test]
fn every_original_go_sum_line_matches_the_pinned_native_parser() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let corpus = oracle("original-module-sums.json");
    let mut rows = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let raw = STANDARD.decode(row["raw"].as_str().unwrap()).unwrap();
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        let result = import_go_module_sums(source);
        assert_eq!(
            result.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{}",
            row["case"]
        );
        if let Ok(sums) = &result {
            assert_eq!(sums.original_source().bytes(), raw);
            let entries = sums
                .entries()
                .iter()
                .map(|((path, version), sum)| json!({"path":path,"version":version,"sum":sum}))
                .collect::<Vec<_>>();
            assert_eq!(json!(entries), row["entries"], "{}", row["case"]);
        }
        rows.push(json!({"case":row["case"],"accepted":result.is_ok(),"error":result.err().map(|e|e.to_string())}));
    }
    output(
        "all-original-sum-observations.json",
        &json!({"status":"passed","cases":rows,"whole_original_corpus_retained":true}),
    );
}
#[test]
fn every_whole_original_zip_matches_native_go_at_full_protocol_boundaries() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let corpora = [oracle("original-module-archives.json"), read(&root().join("module-data/native-compat/native-originals/module-data-compat/original-zip-compatibility.json"))];
    let mut rows = vec![];
    for corpus in corpora {
        for row in corpus["cases"].as_array().unwrap() {
            let raw = STANDARD.decode(row["raw"].as_str().unwrap()).unwrap();
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw)
                    .unwrap();
            let result = inspect_go_module_archive(
                row["path"].as_str().unwrap(),
                row["version"].as_str().unwrap(),
                &source,
                row["sum"].as_str().unwrap(),
                row["remaining"].as_u64().unwrap() as usize,
            );
            assert_eq!(
                result.is_ok(),
                row["accepted"].as_bool().unwrap(),
                "{} {:?}",
                row["case"],
                result
            );
            if let Ok(archive) = &result {
                assert_eq!(archive.original_source().bytes(), raw);
                assert_eq!(
                    archive.namespace_entries(),
                    row["namespace_entries"].as_u64().unwrap() as usize
                );
                let members = archive
                    .members()
                    .values()
                    .map(|m| json!({"Path":m.path,"Bytes":m.bytes,"RawSHA256":m.raw_sha256}))
                    .collect::<Vec<_>>();
                assert_eq!(json!(members), row["members"], "{}", row["case"]);
            }
            rows.push(json!({"case":row["case"],"accepted":result.is_ok(),"error":result.err().map(|e|e.to_string())}));
        }
    }
    output(
        "all-original-archive-observations.json",
        &json!({"status":"passed","cases":rows,"whole_original_corpus_retained":true,"full_expanded_protocol_bytes":500u64<<20,"full_namespace_bound":131072}),
    );
}
#[test]
fn actual_original_versioned_and_local_module_universes_verify_without_host_inputs() {
    let mut rows = vec![];
    for name in ["modules-owned", "versioned-owned"] {
        let (_, input) = original_modules(name);
        let source_count = input
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs()
            .source_objects()
            .len();
        let data = input.verify_offline_module_data().unwrap();
        let archives=data.archives().iter().map(|(path,a)|json!({"path":path,"object":a.original_source().object(),"sum":a.checksum(),"namespace_entries":a.namespace_entries(),"members":a.members().values().map(|m|json!({"path":m.path,"bytes":m.bytes,"raw_sha256":m.raw_sha256})).collect::<Vec<_>>()})).collect::<Vec<_>>();
        if name == "versioned-owned" {
            assert_eq!(archives.len(), 1);
            assert!(data.sums().is_some());
        } else {
            assert!(archives.is_empty());
            assert!(data.sums().is_none());
        }
        assert_eq!(
            data.module_observations()
                .export_inputs()
                .capture_inputs()
                .derive_contracts()
                .unwrap()
                .functions()
                .len(),
            1
        );
        rows.push(json!({"owner":name,"source_objects":source_count,"archives":archives,"sum_source":data.sums().map(|s|s.original_source().object()),"original_selected_Root_contracts":1}));
    }
    output(
        "whole-original-offline-module-data.json",
        &json!({"status":"passed","owners":rows,"general_MVS_root_context_or_source_program_authority":false}),
    );
}
#[test]
fn coherently_resealed_auxiliary_data_cannot_replace_original_module_contents() {
    let directory = root().join("module-observations/native/native-originals/versioned-owned");
    let (packet, original) = original_modules("versioned-owned");
    let all = buffers(&packet, "snapshot_sources", Some(&directory));
    let mut rows = vec![];
    for name in [
        "sum-conflict",
        "sum-missing-manifest",
        "sum-missing-archive",
        "sum-unrelated-malformed",
        "sum-duplicate-identical",
        "sum-unobserved-bytes",
        "ziphash",
        "info-version",
        "archive-encoding-variant",
        "archive-body",
        "archive-comment",
    ] {
        let suffix = if name.starts_with("sum-") {
            "go.sum"
        } else if name == "ziphash" {
            ".ziphash"
        } else if name == "info-version" {
            ".info"
        } else {
            ".zip"
        };
        let source = all
            .iter()
            .find(|s| {
                (s.object().capability
                    == if name.starts_with("sum-") {
                        "workspace/main"
                    } else {
                        "workspace/dep"
                    })
                    && s.object().path.ends_with(suffix)
            })
            .unwrap();
        let object = source.object();
        let mut raw = source.bytes().to_vec();
        match name {
            "sum-conflict" => {
                let line = String::from_utf8(raw.clone())
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap()
                    .to_string();
                let mut fields = line.split_whitespace().collect::<Vec<_>>();
                fields[2] = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
                raw.extend_from_slice(format!("{}\n", fields.join(" ")).as_bytes());
            }
            "sum-missing-manifest" | "sum-missing-archive" => {
                raw = String::from_utf8(raw)
                    .unwrap()
                    .lines()
                    .filter(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .is_some_and(|v| v.ends_with("/go.mod"))
                            == (name == "sum-missing-archive")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into_bytes();
            }
            "sum-unrelated-malformed" => raw.extend_from_slice(b"unrelated malformed input\n"),
            "sum-duplicate-identical" => {
                let extra = raw.clone();
                raw.extend(extra);
            }
            "sum-unobserved-bytes" => {}
            "ziphash" => raw[0] ^= 1,
            "info-version" => {
                let mut v: Value = serde_json::from_slice(&raw).unwrap();
                v["Version"] = json!("v1.2.1");
                raw = serde_json::to_vec(&v).unwrap();
            }
            "archive-encoding-variant" | "archive-body" => {
                let name_len = u16::from_le_bytes(raw[26..28].try_into().unwrap()) as usize;
                let extra_len = u16::from_le_bytes(raw[28..30].try_into().unwrap()) as usize;
                let offset = if name == "archive-body" {
                    let central = raw.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
                    u32::from_le_bytes(raw[central + 20..central + 24].try_into().unwrap()) as usize
                        / 2
                } else {
                    0
                };
                raw[30 + name_len + extra_len + offset] ^= 1;
            }
            "archive-comment" => {
                let end = raw.len() - 22;
                assert_eq!(&raw[end..end + 4], b"PK\x05\x06");
                raw[end + 20] = 1;
                raw.push(b'x');
            }
            _ => unreachable!(),
        }
        let changed =
            GoTypeSourceBuffer::new(&object.capability, &object.path, &object.kind, &raw).unwrap();
        let sources = all
            .iter()
            .filter_map(|s| {
                if s.object().id == object.id {
                    if name == "sum-unobserved-bytes" {
                        None
                    } else {
                        Some(changed.clone())
                    }
                } else {
                    Some(s.clone())
                }
            })
            .collect::<Vec<_>>();
        let mut wire: Value =
            serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
        wire["objects"] = json!(sources.iter().map(|s| s.object()).collect::<Vec<_>>());
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
            original
                .export_inputs()
                .capture_inputs()
                .selection_inputs()
                .clone(),
            snapshot,
        )
        .unwrap();
        let mut export: Value =
            serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
        export["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
        seal(&mut export, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
        let exports =
            import_go_source_export_inputs(&serde_json::to_vec(&export).unwrap(), captured)
                .unwrap();
        let mut module: Value =
            serde_json::from_slice(&unhex(packet["module_observations_hex"].as_str().unwrap()))
                .unwrap();
        module["source_export_inputs_sha256"] = export["content_sha256"].clone();
        seal(&mut module, "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0");
        let observations =
            import_go_source_module_observations(&serde_json::to_vec(&module).unwrap(), exports)
                .unwrap();
        assert_eq!(
            observations
                .export_inputs()
                .capture_inputs()
                .derive_contracts()
                .unwrap()
                .functions()
                .len(),
            1
        );
        let result = observations.verify_offline_module_data();
        let accepted = matches!(
            name,
            "sum-duplicate-identical" | "archive-comment" | "archive-encoding-variant"
        );
        let record = json!({"case":name,"accepted":accepted,"actual_accepted":result.is_ok(),"error":result.as_ref().err().map(|e|e.to_string()),"original_object":object,"changed_object":changed.object(),"complete_changed_raw_hex":hex(&raw),"changed_snapshot_hex":hex(&serde_json::to_vec(&wire).unwrap()),"changed_capture_hex":hex(&serde_json::to_vec(&capture).unwrap()),"changed_export_hex":hex(&serde_json::to_vec(&export).unwrap()),"changed_module_hex":hex(&serde_json::to_vec(&module).unwrap()),"same_original_source_selection_and_Root_contract":true,"reconstructs_original_failed_first_bit_control":name=="archive-encoding-variant"});
        output(&format!("control-{name}.json"), &record);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name} {:?}",
            result.as_ref().err()
        );
        rows.push(record);
    }
    output(
        "coherent-original-module-data-controls.json",
        &json!({"status":"passed","cases":rows,"full_original_source_and_proof_universes_preserved":true}),
    );
}

#[test]
fn original_application_module_info_matches_native_go_with_complete_capture() {
    let directory = root().join("module-observations/native/native-originals/versioned-owned");
    let (packet, original) = original_modules("versioned-owned");
    let all = buffers(&packet, "snapshot_sources", Some(&directory));
    let old = all
        .iter()
        .find(|source| {
            source.object().capability == "workspace/dep"
                && source.object().path == "cache/download/example.com/!dep/@v/v1.2.0.info"
        })
        .unwrap();
    let evidence = root().join("module-info/native");
    let native = read(&evidence.join("native.json"));
    assert_eq!(native["original_source_id"], old.object().id);
    assert_eq!(
        native["original_source_objects"].as_u64().unwrap() as usize,
        all.len()
    );
    assert_eq!(native["original_root_or_vcs_authority"], false);
    let cases = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 14);
    let mut checked = Vec::new();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let row = &case["source"];
        let raw = std::fs::read(
            evidence
                .join("observed-bytes")
                .join(row["raw_file"].as_str().unwrap()),
        )
        .unwrap();
        let changed =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        assert_eq!(
            (
                &changed.object().capability,
                &changed.object().path,
                &changed.object().kind
            ),
            (
                &old.object().capability,
                &old.object().path,
                &old.object().kind
            )
        );
        let accepted = case["accepted"].as_bool().unwrap();
        assert_eq!(
            validate_go_module_cache_info(&changed, "v1.2.0").is_ok(),
            accepted,
            "direct native metadata {name}"
        );
        let sources = all
            .iter()
            .map(|source| {
                if source.object().id == old.object().id {
                    changed.clone()
                } else {
                    source.clone()
                }
            })
            .collect::<Vec<_>>();
        let mut wire: Value =
            serde_json::from_slice(&unhex(packet["snapshot_hex"].as_str().unwrap())).unwrap();
        wire["objects"] = json!(sources
            .iter()
            .map(|source| source.object())
            .collect::<Vec<_>>());
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
            original
                .export_inputs()
                .capture_inputs()
                .selection_inputs()
                .clone(),
            snapshot,
        )
        .unwrap();
        assert_eq!(captured.derive_contracts().unwrap().functions().len(), 1);
        let mut export: Value =
            serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
        export["source_capture_inputs_sha256"] = capture["content_sha256"].clone();
        seal(&mut export, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
        let exports =
            import_go_source_export_inputs(&serde_json::to_vec(&export).unwrap(), captured)
                .unwrap();
        let mut module: Value =
            serde_json::from_slice(&unhex(packet["module_observations_hex"].as_str().unwrap()))
                .unwrap();
        module["source_export_inputs_sha256"] = export["content_sha256"].clone();
        seal(&mut module, "MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0");
        let observations =
            import_go_source_module_observations(&serde_json::to_vec(&module).unwrap(), exports)
                .unwrap();
        let result = observations.verify_offline_module_data();
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.as_ref().err()
        );
        if !accepted {
            assert_eq!(
                result.unwrap_err(),
                GoSourceModuleDataError::Metadata,
                "{name}"
            );
        }
        checked.push(json!({"case":name,"accepted":accepted,"original_info_id":old.object().id,"changed_info":changed.object(),"snapshot_sha256":wire["snapshot_sha256"],"source_capture_sha256":capture["content_sha256"],"export_sha256":export["content_sha256"],"module_sha256":module["content_sha256"],"original_source_objects":all.len(),"original_root_contracts":1}));
    }
    output(
        "original-application-module-info-controls.json",
        &json!({"status":"passed","cases":checked,"original_application_and_sdk_sources_preserved":true,"root_or_vcs_authority":false}),
    );
}
