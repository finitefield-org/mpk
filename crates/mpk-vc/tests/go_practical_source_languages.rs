use mpk_vc::go_practical_context::{GoPrivateCandidateRegistry, GoPrivateCandidateRequest};
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn root() -> PathBuf {
    evidence().join("source-languages")
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
        .collect()
}
fn buffers(v: &Value) -> Vec<GoTypeSourceBuffer> {
    v["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(row["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect()
}
fn inputs(v: &Value) -> GoSourceInputs {
    import_go_source_inputs(&unhex(v["components_hex"].as_str().unwrap()), &buffers(v)).unwrap()
}
fn corpus() -> Value {
    serde_json::from_slice(
        &std::fs::read(root().join("native-final/native-originals/probe/language-corpus.json"))
            .unwrap(),
    )
    .unwrap()
}
fn output(name: &str, v: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_SOURCE_LANGUAGES_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .unwrap();
        f.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
    }
}

#[test]
fn source_languages_match_every_original_pinned_go_observation() {
    let c = corpus();
    let mut rows = vec![];
    for row in c["cases"].as_array().unwrap() {
        let input = inputs(row);
        let actual = input.languages();
        if row["metadata_accepted"].as_bool().unwrap() {
            let langs = actual.unwrap_or_else(|e| panic!("{}: {e}", row["name"]));
            assert_eq!(langs.files().count(), 1);
            let language = langs
                .source(row["go_source_object_id"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                language.module_source_object_id(),
                row["module_source_object_id"].as_str().unwrap()
            );
            assert_eq!(
                language.module_requirement(),
                row["module_requirement"].as_str(),
                "{} module",
                row["name"]
            );
            assert_eq!(
                language.build_requirement(),
                row["build_requirement"].as_str(),
                "{} constraint",
                row["name"]
            );
            assert_eq!(
                language.effective_version(),
                row["effective_version"].as_str().unwrap(),
                "{} effective",
                row["name"]
            );
            assert_eq!(
                language.language_version(),
                row["language_version"].as_str().unwrap(),
                "{} minor",
                row["name"]
            );
            assert_eq!(
                language.in_practical_range(),
                matches!(
                    language.language_version(),
                    "go1.22" | "go1.23" | "go1.24" | "go1.25"
                )
            );
            rows.push(json!({"name":row["name"],"module":language.module_requirement(),"build":language.build_requirement(),"effective":language.effective_version(),"language":language.language_version(),"in_practical_range":language.in_practical_range()}));
        } else {
            let err = actual.unwrap_err();
            rows.push(json!({"name":row["name"],"error":err.to_string(),"source":err.source_object_id(),"offset":err.offset()}));
        }
    }
    assert!(rows.len() > 120);
    output(
        "original-language-comparison.json",
        &json!({"status":"passed","rows":rows,"native_version_facts_enter_constructor":false,"source_or_context_proof":false}),
    );
}

fn selector(name: &str) -> GoSourceFunctionSelector {
    serde_json::from_value(json!({"package":"example.com/policy","function":name,"receiver":null}))
        .unwrap()
}

fn seal(value: &mut Value, field: &str, domain: &'static str) {
    value.as_object_mut().unwrap().remove(field);
    let strict = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value[field] = json!(hash_canonical_json(HashDomain::new(domain), &strict)
        .unwrap()
        .to_hex());
}

fn request(name: &str) -> GoPrivateCandidateRequest {
    let mut wire: Value = serde_json::from_slice(include_bytes!(
        "fixtures/go-practical-candidate-request.json"
    ))
    .unwrap();
    wire["selection"]["value"]["roots"] =
        json!([{"package":"example.com/policy","function":name,"receiver":null}]);
    seal(
        &mut wire,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    GoPrivateCandidateRegistry::new()
        .unwrap()
        .import_request(
            &parse_strict_json(
                &serde_json::to_vec(&wire).unwrap(),
                StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
            )
            .unwrap(),
        )
        .unwrap()
}

fn physical_selection(
    packet: &Value,
    unit: &GoSourceContractInputs,
    original: &[GoTypeSourceBuffer],
    name: &str,
) -> GoSourceSelectionInputs {
    // Derive the original declaration only to construct the existing physical
    // selection record. The candidate-bound path below performs its own checks.
    let module = derive_go_source_contract_module(
        unit.source_inputs().components(),
        &[selector(name)],
        unit.overlays(),
    )
    .unwrap();
    let root = &module.selected_roots()[0];
    let node = &module
        .selection()
        .source(root.source_object_id())
        .unwrap()
        .nodes()[root.node()];
    let package = module
        .selection()
        .packages()
        .iter()
        .find(|package| {
            package
                .source_objects
                .iter()
                .any(|id| id == root.source_object_id())
        })
        .unwrap();
    let mut wire = json!({
        "schema":"mpk.go.practical.source_selection_inputs.v1",
        "source_contract_inputs_sha256":unit.content_sha256(),
        "roots":[{"package_id":package.id,"source_object_id":root.source_object_id(),
                  "start_byte":node.start(),"end_byte":node.end()}],
    });
    seal(
        &mut wire,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    import_go_source_selection_inputs(
        &serde_json::to_vec(&wire).unwrap(),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        original,
        &[],
    )
    .unwrap()
}

#[test]
fn candidate_bound_contracts_enforce_original_selected_body_languages() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(root().join("native/native-originals/owned/language-owned-inputs.json"))
            .unwrap(),
    )
    .unwrap();
    let original = buffers(&packet);
    let unit = import_go_source_contract_inputs(
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &original,
        &[],
    )
    .unwrap();
    assert_eq!(unit.source_inputs().languages().unwrap().files().count(), 3);
    let inputs = physical_selection(&packet, &unit, &original, "Root");
    let root_request = request("Root");
    let binding = bind_go_practical_source_selection(&root_request, &inputs).unwrap();
    let contracts = binding.derive_contracts().unwrap();
    assert_eq!(contracts.module().functions().len(), 2);
    assert_eq!(contracts.source().request(), &root_request);
    for name in ["Legacy", "CallsLegacy"] {
        let inputs = physical_selection(&packet, &unit, &original, name);
        let request = request(name);
        let binding = bind_go_practical_source_selection(&request, &inputs).unwrap();
        let error = match binding.derive_contracts() {
            Err(GoSourceContractDerivationError::Language(error)) => error,
            _ => panic!("candidate must reject the original selected old-language body"),
        };
        assert_eq!(error.kind(), GoSourceLanguageErrorKind::SelectedLanguage);
        assert_eq!(error.version(), Some("go1.21"));
        assert!(error.offset() > 0);
        assert_eq!(
            original
                .iter()
                .find(|source| source.object().id == error.source_object_id())
                .unwrap()
                .object()
                .path,
            "legacy.go"
        );
    }
}
#[test]
fn owned_contracts_check_every_reached_body_and_keep_unselected_old_files() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(root().join("native/native-originals/owned/language-owned-inputs.json"))
            .unwrap(),
    )
    .unwrap();
    let p = buffers(&packet);
    let unit = import_go_source_contract_inputs(
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &p,
        &[],
    )
    .unwrap();
    let versions: Value = serde_json::from_slice(
        &std::fs::read(root().join("native/native-originals/owned/language-owned-versions.json"))
            .unwrap(),
    )
    .unwrap();
    let languages = unit.source_inputs().languages().unwrap();
    assert_eq!(languages.files().count(), 3);
    for row in languages.files() {
        assert_eq!(
            row.effective_version(),
            versions["files"][row.source_object_id()].as_str().unwrap()
        );
    }
    let module = unit.derive_contracts(&[selector("Root")]).unwrap();
    assert_eq!(module.functions().len(), 2);
    assert_eq!(
        languages
            .files()
            .filter(|r| !r.in_practical_range())
            .count(),
        1
    );
    let mut rejected = vec![];
    for name in ["Legacy", "CallsLegacy"] {
        let GoSourceContractDerivationError::Language(error) =
            unit.derive_contracts(&[selector(name)]).unwrap_err()
        else {
            panic!("actual file-language rejection")
        };
        assert_eq!(error.kind(), GoSourceLanguageErrorKind::SelectedLanguage);
        assert_eq!(error.version(), Some("go1.21"));
        assert!(error.offset() > 0);
        assert_eq!(
            p.iter()
                .find(|s| s.object().id == error.source_object_id())
                .unwrap()
                .object()
                .path,
            "legacy.go"
        );
        rejected.push(json!({"selected":name,"error":error.to_string(),"origin":error.source_object_id(),"offset":error.offset()}));
    }
    let c = corpus();
    let foreign = inputs(&c["cases"][0]).languages().unwrap();
    assert_eq!(
        foreign
            .check_selection(module.selection())
            .unwrap_err()
            .kind(),
        GoSourceLanguageErrorKind::Source
    );
    output(
        "owned-language-selection.json",
        &json!({"status":"passed","all_original_files":3,"checked_root_and_transitive_bodies":2,"unselected_old_file_retained":true,"old_root_and_callee_rejected":rejected,"foreign_package_context_rejected":true}),
    );
}

#[test]
fn source_languages_preserve_both_complete_original_source_units() {
    let v: Value = serde_json::from_slice(
        &std::fs::read(evidence().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap();
    let mut units = vec![];
    for unit in v["units"].as_array().unwrap() {
        let p = &unit["input"];
        let input = inputs(p);
        let languages = input.languages().unwrap();
        let files = buffers(p)
            .into_iter()
            .filter(|s| s.object().kind == "go_file")
            .count();
        assert_eq!(languages.files().count(), files);
        for row in languages.files() {
            assert_eq!(row.module_requirement(), Some("go1.25.0"));
            assert_eq!(row.effective_version(), "go1.25.0");
            assert_eq!(row.language_version(), "go1.25");
            assert!(row.in_practical_range());
        }
        units.push(json!({"name":unit["name"],"complete_original_files":files,"source_component_bytes_equal":input.components().canonical_bytes()==unhex(p["components_hex"].as_str().unwrap()),"file_language":"go1.25"}));
    }
    output(
        "original-unit-languages.json",
        &json!({"status":"passed","units":units}),
    );
}
