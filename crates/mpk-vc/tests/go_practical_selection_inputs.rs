use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn root() -> PathBuf {
    evidence().join("selection-inputs")
}
fn read(path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(path)).unwrap()).unwrap()
}
fn packet(name: &str) -> Value {
    read(&format!("native/native-originals/selection-owned/{name}"))
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn buffers(p: &Value, field: &str) -> Vec<GoTypeSourceBuffer> {
    p[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&r["object"]).unwrap(),
                &unhex(r["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect()
}
fn wire(p: &Value) -> Value {
    serde_json::from_slice(&unhex(p["selection_hex"].as_str().unwrap())).unwrap()
}
fn import(p: &Value, v: &Value) -> Result<GoSourceSelectionInputs, GoSourceSelectionInputError> {
    import_go_source_selection_inputs(
        &serde_json::to_vec(v).unwrap(),
        &unhex(p["envelope_hex"].as_str().unwrap()),
        &unhex(p["components_hex"].as_str().unwrap()),
        &buffers(p, "sources"),
        &buffers(p, "overlays"),
    )
}
fn seal(v: &mut Value) {
    v.as_object_mut().unwrap().remove("content_sha256");
    let parsed = parse_strict_json(
        &serde_json::to_vec(v).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    v["content_sha256"] = json!(hash_canonical_json(
        HashDomain::new("MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0"),
        &parsed
    )
    .unwrap()
    .to_hex());
}
fn output(name: &str, v: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_SELECTION_INPUTS_OUTPUT") {
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
fn selection_inputs_derive_only_the_bound_original_functions_and_methods() {
    let p = packet("selection-inputs.json");
    let input = import(&p, &wire(&p)).unwrap();
    assert_eq!(
        input.canonical_bytes(),
        unhex(p["selection_hex"].as_str().unwrap())
    );
    assert_eq!(
        input.contract_inputs().canonical_bytes(),
        unhex(p["envelope_hex"].as_str().unwrap())
    );
    assert_eq!(
        input
            .contract_inputs()
            .source_inputs()
            .components()
            .canonical_bytes(),
        unhex(p["components_hex"].as_str().unwrap())
    );
    assert_eq!(input.roots().len(), 3);
    assert_eq!(input.contract_inputs().overlays().len(), 3);
    let module = input.derive_contracts().unwrap();
    assert_eq!(module.selected_roots().len(), 3);
    assert_eq!(module.functions().len(), 4);
    let selected = module.selection();
    for root in input.roots() {
        let matches = selected
            .roots()
            .iter()
            .filter(|r| {
                let node = &selected.source(r.source_object_id()).unwrap().nodes()[r.node()];
                r.source_object_id() == root.source_object_id()
                    && node.start() as u64 == root.start_byte()
                    && node.end() as u64 == root.end_byte()
            })
            .count();
        assert_eq!(matches, 1);
    }
    assert_eq!(
        module
            .functions()
            .iter()
            .filter(|f| f.overlay_object_id().is_some())
            .count(),
        2
    );
    assert!(selected
        .declarations()
        .iter()
        .filter(|d| d.names().iter().any(|n| n.starts_with("Bound")))
        .all(|d| d.roles().collect::<Vec<_>>() == vec![GoSourceSelectionRole::TypeCheckOnly]));
    assert_eq!(
        selected
            .declarations()
            .iter()
            .filter(|d| d.names().iter().any(|n| n.starts_with("Bound")))
            .count(),
        8192
    );
    output(
        "owned-physical-selection.json",
        &json!({"status":"passed","selected_roots":input.roots(),"checked_functions":4,"selected_same_name_receiver_methods":2,"captured_raw_sidecars":3,"unselected_original_boundary_functions":8192,"native_name_or_type_hints_used":false}),
    );
}

#[test]
fn selection_inputs_distinguish_ordinary_init_methods_from_package_initializers() {
    let p = read("native-final/native-originals/method-init/selection-method-init.json");
    let native = read("native-final/native-originals/method-init/method-init-observations.json");
    let input = import(&p, &wire(&p)).unwrap();
    let module = input.derive_contracts().unwrap();
    assert_eq!(module.functions().len(), 1);
    let function = &module.functions()[0];
    assert_eq!(function.signature().name(), Some("init"));
    assert!(function
        .signature()
        .slots()
        .iter()
        .any(|s| s.role() == GoSourceSignatureRole::Receiver));
    let selector=serde_json::from_value(json!({"package":"example.com/policy","function":"init","receiver":{"package":"example.com/policy","type_name":"Count"}})).unwrap();
    assert_eq!(
        input
            .contract_inputs()
            .derive_contracts(&[selector])
            .unwrap(),
        module
    );
    let mut rejected = wire(&p);
    rejected["roots"][0]["start_byte"] = native["package_initializer"]["start"].clone();
    rejected["roots"][0]["end_byte"] = native["package_initializer"]["end"].clone();
    seal(&mut rejected);
    assert_eq!(
        import(&p, &rejected).unwrap_err(),
        GoSourceSelectionInputError::Root
    );
    output(
        "ordinary-init-method.json",
        &json!({"status":"passed","actual_original_source":native,"selected_method_named_init":true,"package_initializer_rejected":true,"physical_and_named_contract_derivations_equal":true}),
    );
}

#[test]
fn selection_inputs_reject_changed_crossed_and_non_function_roots() {
    let p = packet("selection-inputs.json");
    let original = wire(&p);
    let observations = packet("physical-root-observations.json");
    let mut controls: Vec<(&str, Value)> = vec![];
    let mut v = original.clone();
    v["roots"][0]["start_byte"] = json!(v["roots"][0]["start_byte"].as_u64().unwrap() + 1);
    controls.push(("stale-hash", v));
    let mut v = original.clone();
    v["schema"] = json!("unknown");
    seal(&mut v);
    controls.push(("schema", v));
    let mut v = original.clone();
    v["extra"] = json!(false);
    seal(&mut v);
    controls.push(("unknown-member", v));
    let mut v = original.clone();
    v["source_contract_inputs_sha256"] = json!("0".repeat(64));
    seal(&mut v);
    controls.push(("crossed-contract-set", v));
    let mut v = original.clone();
    v["roots"] = json!([]);
    seal(&mut v);
    controls.push(("empty", v));
    let mut v = original.clone();
    v["roots"] = json!([v["roots"][0].clone(), v["roots"][0].clone()]);
    seal(&mut v);
    controls.push(("duplicate", v));
    let mut v = original.clone();
    v["roots"].as_array_mut().unwrap().reverse();
    seal(&mut v);
    controls.push(("order", v));
    let mut v = original.clone();
    v["roots"] = json!(vec![v["roots"][0].clone(); 8193]);
    seal(&mut v);
    controls.push(("count-over", v));
    for (name, field, value) in [
        ("crossed-package", "package_id", json!("0".repeat(64))),
        ("crossed-source", "source_object_id", json!("0".repeat(64))),
        (
            "interior-start",
            "start_byte",
            json!(original["roots"][0]["start_byte"].as_u64().unwrap() + 1),
        ),
        ("boolean-offset", "start_byte", json!(true)),
        ("supplied-node", "node", json!(0)),
    ] {
        let mut v = original.clone();
        v["roots"] = json!([v["roots"][0].clone()]);
        v["roots"][0][field] = value;
        seal(&mut v);
        controls.push((name, v));
    }
    for (name, key) in [
        ("initializer", "example.com/policy.init"),
        ("blank", "example.com/policy._"),
    ] {
        let physical = &observations["functions"][key];
        let mut v = original.clone();
        let package_id = v["roots"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["source_object_id"] == physical["source"])
            .unwrap()["package_id"]
            .clone();
        v["roots"] = json!([{"package_id":package_id,"source_object_id":physical["source"],"start_byte":physical["start"],"end_byte":physical["end"]}]);
        seal(&mut v);
        controls.push((name, v));
    }
    let mut checked = vec![];
    for (name, v) in controls {
        let error = import(&p, &v).unwrap_err();
        checked.push(json!({"control":name,"error":error.to_string()}));
    }
    let mut missing = p.clone();
    missing["sources"].as_array_mut().unwrap().pop();
    assert!(import(&missing, &original).is_err());
    let mut missing = p.clone();
    missing["overlays"].as_array_mut().unwrap().pop();
    assert!(import(&missing, &original).is_err());
    // A coherent new root is a different candidate, not authentication of intent.
    let physical = &observations["functions"]["example.com/policy.Standalone"];
    let mut replacement = original.clone();
    let package_id = original["roots"][0]["package_id"].clone();
    replacement["roots"] = json!([{"package_id":package_id,"source_object_id":physical["source"],"start_byte":physical["start"],"end_byte":physical["end"]}]);
    seal(&mut replacement);
    let candidate = import(&p, &replacement).unwrap();
    assert_ne!(
        candidate.content_sha256(),
        original["content_sha256"].as_str().unwrap()
    );
    assert_eq!(candidate.derive_contracts().unwrap().functions().len(), 1);
    output(
        "physical-selection-controls.json",
        &json!({"status":"passed","rejections":checked,"missing_original_source_and_sidecar_rejected":true,"coherent_other_root_is_new_untrusted_candidate":true,"hash_is_launcher_authentication":false}),
    );
}

#[test]
fn selection_inputs_keep_the_exact_original_8192_root_boundary() {
    let p = packet("selection-boundary-inputs.json");
    let input = import(&p, &wire(&p)).unwrap();
    assert_eq!(input.roots().len(), 8192);
    let module = input.derive_contracts().unwrap();
    assert_eq!(module.selected_roots().len(), 8192);
    assert_eq!(module.functions().len(), 8192);
    assert!(module
        .functions()
        .iter()
        .all(|f| f.is_selected_root() && f.overlay_object_id().is_none()));
    assert_eq!(module.expression_nodes(), 8192);
    output(
        "physical-selection-boundary.json",
        &json!({"status":"passed","original_roots":8192,"derived_functions":8192,"expression_nodes":8192,"complete_program_proof":false}),
    );
}

#[test]
fn selection_inputs_bind_all_roots_of_both_complete_original_source_units() {
    let old: Value = serde_json::from_slice(
        &std::fs::read(evidence().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap();
    let prior: Value = serde_json::from_slice(
        &std::fs::read(evidence().join("source-invocations/corpus.json")).unwrap(),
    )
    .unwrap();
    let mut rows = vec![];
    for name in ["pure", "wide"] {
        let p = read(&format!(
            "native/native-originals/{name}/physical-selection-inputs.json"
        ));
        let input = import(&p, &wire(&p)).unwrap();
        let original = old["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .unwrap();
        assert_eq!(p["components_hex"], original["input"]["components_hex"]);
        assert_eq!(p["sources"], original["input"]["sources"]);
        let unit = prior["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .unwrap();
        let expected = unit["roots"].as_array().unwrap();
        assert_eq!(input.roots().len(), expected.len());
        for r in expected {
            assert_eq!(
                input
                    .roots()
                    .iter()
                    .filter(|p| p.source_object_id() == r["source"].as_str().unwrap()
                        && p.start_byte() == r["start"].as_u64().unwrap()
                        && p.end_byte() == r["end"].as_u64().unwrap())
                    .count(),
                1
            );
        }
        rows.push(json!({"name":name,"complete_original_roots":input.roots().len(),"source_component_and_all_original_bytes_equal":true,"original_error_cases_preserved_in_prior_consumer":if name=="wide"{70}else{0}}));
    }
    output(
        "all-original-root-inputs.json",
        &json!({"status":"passed","units":rows,"original_error_universes_replaced_by_defaults":false,"scope":"Complete original physical-root input binding only; the unchanged source-input consumer separately replays all original zero/70-case call proofs. No default-contract derivation substitutes for the original entry cases."}),
    );
}
