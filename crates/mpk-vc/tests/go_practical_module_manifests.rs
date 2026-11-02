use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn read(p: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_MODULE_MANIFEST_OUTPUT") {
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
fn facts(input: &GoSourceModuleManifest) -> Value {
    let require = |r: &GoModuleRequirement| json!({"path_hex":hex(&r.path),"version":r.version,"indirect":r.indirect});
    json!({"module_path_hex":hex(input.module_path()),"go_requirement":input.go_requirement(),"language_version":input.language_version(),"toolchain_suggestion_hex":input.toolchain_suggestion().map(hex),"requirements":input.requirements().iter().map(require).collect::<Vec<_>>(),"exclusions":input.exclusions().iter().map(require).collect::<Vec<_>>(),"replacements":input.replacements().iter().map(|r|json!({"old_path_hex":hex(&r.old_path),"old_version":r.old_version,"new_path_hex":hex(&r.new_path),"new_version":r.new_version})).collect::<Vec<_>>(),"retractions":input.retractions().iter().map(|r|json!({"low_hex":hex(&r.low),"high_hex":hex(&r.high)})).collect::<Vec<_>>()})
}
#[test]
fn every_complete_original_manifest_matches_pinned_go_directive_and_version_facts() {
    let corpus=read(&root().join("module-manifests/native-toolchain/native-originals/module-manifests/complete-original-module-manifests.json"));
    let mut observed = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let raw = unhex(row["raw_hex"].as_str().unwrap());
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap();
        let input = derive_go_source_module_manifest(source, row["main"].as_bool().unwrap());
        assert_eq!(
            input.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "case {} main={} error={:?}",
            row["case"],
            row["main"],
            input.as_ref().err()
        );
        let actual = input.as_ref().ok().map(facts);
        if let Ok(parsed) = input.as_ref() {
            assert_eq!(parsed.original_source().bytes(), raw);
            assert_eq!(
                actual.as_ref().unwrap(),
                &row["facts"],
                "case {} main={}",
                row["case"],
                row["main"]
            );
        }
        observed.push(json!({"case":row["case"],"main":row["main"],"object":row["object"],"accepted":input.is_ok(),"error":input.err().map(|e|e.to_string()),"facts":actual}));
    }
    output(
        "complete-original-module-manifest-observations.json",
        &json!({"status":"passed","observations":observed,"whole_original_source_bytes_preserved":true,"MVS_selected_or_profile_activated":false}),
    );
}
#[test]
fn every_original_selected_module_keeps_its_owned_manifest_and_all_dependency_rows() {
    let mut observations = vec![];
    for owner in ["modules-owned", "versioned-owned"] {
        let directory = root()
            .join("module-observations/native/native-originals")
            .join(owner);
        let packet = read(&directory.join("complete-original-module-observations.json"));
        let wire: Value =
            serde_json::from_slice(&unhex(packet["module_observations_hex"].as_str().unwrap()))
                .unwrap();
        let exports: Value =
            serde_json::from_slice(&unhex(packet["export_inputs_hex"].as_str().unwrap())).unwrap();
        let mut rows = vec![];
        for selected in wire["modules"].as_array().unwrap() {
            let id = selected["manifest_source_object_id"].as_str().unwrap();
            let original = packet["snapshot_sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["object"]["id"] == id)
                .unwrap();
            let raw = std::fs::read(
                directory
                    .join("original-source-objects")
                    .join(original["raw_file"].as_str().unwrap()),
            )
            .unwrap();
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&original["object"]).unwrap(), &raw)
                    .unwrap();
            let main = selected["path"] == "example.com/policy";
            let parsed = derive_go_source_module_manifest(source, main).unwrap();
            assert_eq!(parsed.original_source().bytes(), raw);
            rows.push(json!({"path":selected["path"],"original_object":original["object"],"facts":facts(&parsed)}));
        }
        assert_eq!(rows.len(), if owner == "modules-owned" { 3 } else { 2 });
        let main = &rows
            .iter()
            .find(|r| r["path"] == "example.com/policy")
            .unwrap()["facts"];
        assert_eq!(
            main["requirements"].as_array().unwrap().len(),
            if owner == "modules-owned" { 2 } else { 1 }
        );
        observations.push(json!({"owner":owner,"original_snapshot_objects":packet["snapshot_sources"].as_array().unwrap().len(),"original_packages":exports["packages"].as_array().unwrap().len(),"modules":rows,"original_source_or_package_universe_reduced":false}));
    }
    output(
        "whole-original-selected-module-manifests.json",
        &json!({"status":"passed","observations":observations}),
    );
}
