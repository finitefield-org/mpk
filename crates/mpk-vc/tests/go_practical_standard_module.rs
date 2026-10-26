use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/standard-module")
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn facts(input: &GoSourceStrictModuleManifest) -> Value {
    let require = |r: &GoModuleRequirement| json!({"path_hex":hex(&r.path),"version":r.version,"indirect":r.indirect});
    json!({"module_path_hex":input.module_path().map(hex),"go_requirement":input.go_requirement(),"toolchain_suggestion_hex":input.toolchain_suggestion().map(hex),"requirements":input.requirements().iter().map(require).collect::<Vec<_>>(),"exclusions":input.exclusions().iter().map(require).collect::<Vec<_>>(),"replacements":input.replacements().iter().map(|r|json!({"old_path_hex":hex(&r.old_path),"old_version":r.old_version,"new_path_hex":hex(&r.new_path),"new_version":r.new_version})).collect::<Vec<_>>(),"retractions":input.retractions().iter().map(|r|json!({"low_hex":hex(&r.low),"high_hex":hex(&r.high)})).collect::<Vec<_>>(),"debug_settings":input.debug_settings().iter().map(|r|json!({"key_hex":hex(&r.key),"value_hex":hex(&r.value)})).collect::<Vec<_>>(),"tools_hex":input.tools().iter().map(|s|hex(s)).collect::<Vec<_>>(),"ignored_paths_hex":input.ignored_paths().iter().map(|s|hex(s)).collect::<Vec<_>>()})
}
#[test]
fn complete_original_strict_module_facts_and_standard_binding_match_pinned_go() {
    let native: Value = serde_json::from_slice(
        &std::fs::read(root().join("native/observations/complete-original-strict-modules.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(native["go_version"], "go1.25.0");
    assert_eq!(native["prior_original_cases"], 656);
    let mut observations = vec![];
    let mut long = 0;
    let mut sdk = 0;
    for row in native["cases"].as_array().unwrap() {
        let raw = unhex(row["raw_hex"].as_str().unwrap());
        let source =
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["source_object"]).unwrap(), &raw)
                .unwrap();
        let parsed = derive_go_source_strict_module_manifest(source);
        assert_eq!(
            parsed.is_ok(),
            row["parse_accepted"].as_bool().unwrap(),
            "{}: {parsed:?}; native={}",
            row["name"],
            row["parse_error"]
        );
        let standard = parsed
            .as_ref()
            .is_ok_and(|p| p.standard_library_go_requirement().is_ok());
        assert_eq!(
            standard,
            row["standard_accepted"].as_bool().unwrap(),
            "{}",
            row["name"]
        );
        let actual = parsed.as_ref().ok().map(facts);
        if let Ok(p) = &parsed {
            assert_eq!(p.original_source().bytes(), raw);
            assert_eq!(actual.as_ref().unwrap(), &row["facts"], "{}", row["name"]);
            if row["name"] == "long-complete-directives" {
                assert_eq!(p.debug_settings().len(), 40000);
                assert!(p
                    .original_source()
                    .bytes()
                    .ends_with(b"// original suffix\n"));
                long += 1;
            }
        }
        if row["name"].as_str().unwrap().starts_with("sdk-") {
            sdk += 1;
        }
        observations.push(json!({"name":row["name"],"source_object":row["source_object"],"parse_accepted":parsed.is_ok(),"parse_error":parsed.err().map(|e|e.to_string()),"standard_accepted":standard,"facts":actual}));
    }
    assert_eq!(long, 1);
    assert!(sdk >= 14);
    if let Some(dir) = std::env::var_os("MPK_GO_STANDARD_MODULE_RUST_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("complete-original-strict-module-comparison.json"),serde_json::to_vec_pretty(&json!({"status":"passed","observations":observations,"prior_original_cases":656,"sdk_manifests":sdk,"whole_original_source_bytes_preserved":true,"public_profile_activated":false})).unwrap()).unwrap();
    }
}
#[test]
fn general_strict_facts_do_not_change_application_admission_or_dependency_lax_rules() {
    for raw in [
        b"module std\ngo 1.25\n".as_slice(),
        b"module example.com/policy\ngo 1.25\ngodebug panicnil=1\n",
        b"module example.com/policy\ngo 1.25\ntool example.com/tool\n",
        b"module example.com/policy\ngo 1.25\nignore assets\n",
    ] {
        let source = GoTypeSourceBuffer::new("strict-facts", "go.mod", "go_mod", raw).unwrap();
        assert!(derive_go_source_strict_module_manifest(source.clone()).is_ok());
        assert!(derive_go_source_module_manifest(source, true).is_err());
    }
    let bad = GoTypeSourceBuffer::new(
        "strict-facts",
        "go.mod",
        "go_mod",
        b"module std\ngo 1.25garbage\n",
    )
    .unwrap();
    assert_eq!(
        derive_go_source_graph_manifest(bad.clone())
            .unwrap()
            .go_requirement(),
        Some("1.25")
    );
    assert!(derive_go_source_strict_module_manifest(bad).is_err());
    let suggestion = GoTypeSourceBuffer::new(
        "strict-facts",
        "go.mod",
        "go_mod",
        b"module std\ngo 1.25\ntoolchain go1.99.0\n",
    )
    .unwrap();
    assert!(derive_go_source_strict_module_manifest(suggestion)
        .unwrap()
        .standard_library_go_requirement()
        .is_ok());
    let wrong = GoTypeSourceBuffer::new(
        "strict-facts",
        "go.mod",
        "auxiliary",
        b"module std\ngo 1.25\n",
    )
    .unwrap();
    assert!(derive_go_source_strict_module_manifest(wrong).is_err());
}
