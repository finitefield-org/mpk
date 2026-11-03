use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/contract-inputs")
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn seal(v: &mut Value) {
    seal_domain(v, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0");
}
fn seal_domain(v: &mut Value, domain: &'static str) {
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

#[test]
fn candidate_contract_inputs_preserve_all_selected_packages_and_request_identity() {
    let p = packet();
    let overlays = buffers(&p, "overlays");
    let mut candidate = envelope(&p);
    candidate
        .as_object_mut()
        .unwrap()
        .remove("requested_package");
    candidate["schema"] = json!("mpk.go.practical.source_contract_inputs.v2");
    candidate["requested_packages"] = json!(["example.com/policy", "example.com/policy/helper"]);
    candidate["candidate_request_sha256"] = json!("a".repeat(64));
    seal_domain(&mut candidate, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0");
    let imported = import(&p, &candidate, &overlays).unwrap();
    assert_eq!(imported.requested_package(), None);
    assert_eq!(
        imported.requested_packages(),
        ["example.com/policy", "example.com/policy/helper"]
    );
    assert_eq!(
        imported.candidate_request_sha256(),
        Some("a".repeat(64).as_str())
    );

    for control in [
        "empty",
        "duplicate",
        "order",
        "absent",
        "hash",
        "mixed",
        "null",
    ] {
        let mut changed = candidate.clone();
        match control {
            "empty" => changed["requested_packages"] = json!([]),
            "duplicate" => {
                changed["requested_packages"] = json!(["example.com/policy", "example.com/policy"])
            }
            "order" => {
                changed["requested_packages"] =
                    json!(["example.com/policy/helper", "example.com/policy"])
            }
            "absent" => {
                changed["requested_packages"] = json!(["example.com/absent", "example.com/policy"])
            }
            "hash" => changed["candidate_request_sha256"] = json!("A".repeat(64)),
            "mixed" => changed["requested_package"] = json!("example.com/policy"),
            "null" => changed["requested_package"] = Value::Null,
            _ => unreachable!(),
        }
        seal_domain(&mut changed, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0");
        assert!(
            import(&p, &changed, &overlays).is_err(),
            "accepted {control}"
        );
    }
}
fn packet() -> Value {
    serde_json::from_slice(
        &std::fs::read(root().join("native-final/native-originals/owned/contract-inputs.json"))
            .unwrap(),
    )
    .unwrap()
}
fn buffers(p: &Value, key: &str) -> Vec<GoTypeSourceBuffer> {
    p[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect()
}
fn envelope(p: &Value) -> Value {
    serde_json::from_slice(&unhex(p["envelope_hex"].as_str().unwrap())).unwrap()
}
fn import(
    p: &Value,
    v: &Value,
    o: &[GoTypeSourceBuffer],
) -> Result<GoSourceContractInputs, GoSourceContractInputError> {
    import_go_source_contract_inputs(
        &serde_json::to_vec(v).unwrap(),
        &unhex(p["components_hex"].as_str().unwrap()),
        &buffers(p, "sources"),
        o,
    )
}
fn output(name: &str, v: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_CONTRACT_INPUTS_OUTPUT") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p)
            .unwrap();
        use std::io::Write;
        f.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
    }
}

#[test]
fn contract_inputs_bind_actual_owned_sources_and_every_sidecar() {
    let p = packet();
    let e = envelope(&p);
    let mut overlays = buffers(&p, "overlays");
    overlays.reverse(); // Raw-buffer transport order confers no identity.
    let input = import(&p, &e, &overlays).unwrap();
    assert_eq!(
        input.canonical_bytes(),
        unhex(p["envelope_hex"].as_str().unwrap())
    );
    assert_eq!(
        input.source_inputs().components().canonical_bytes(),
        unhex(p["components_hex"].as_str().unwrap())
    );
    assert_eq!(input.overlays().len(), 3);
    assert_eq!(input.requested_package(), Some("example.com/policy"));
    for (got, key) in [
        (input.image_sha256(), "image_sha256"),
        (input.compiler_sha256(), "compiler_sha256"),
        (input.main_source_object_id(), "main_source_object_id"),
        (input.content_sha256(), "content_sha256"),
    ] {
        assert_eq!(got, e[key].as_str().unwrap());
    }
    let roots = serde_json::from_value::<Vec<GoSourceFunctionSelector>>(
        json!([{"package":"example.com/policy","function":"Root","receiver":null}]),
    )
    .unwrap();
    let module = input.derive_contracts(&roots).unwrap();
    assert_eq!(module.functions().len(), 2);
    assert_eq!(module.captured_overlays().len(), 3);
    assert_eq!(
        module
            .captured_overlays()
            .iter()
            .filter(|o| !o.in_body_closure())
            .count(),
        1
    );
    assert!(module
        .functions()
        .iter()
        .all(|f| matches!(f.origin(), GoSourceContractOrigin::SourceOverlay { .. })));
    let stored = input.clone();
    drop(input);
    drop(overlays);
    drop(p);
    assert_eq!(stored.derive_contracts(&roots).unwrap(), module);
    output(
        "actual-original-contracts.json",
        &json!({"status":"passed","functions":module.functions().iter().map(|f|json!({"source":f.function().source_object_id(),"node":f.function().node(),"selected":f.is_selected_root(),"overlay":f.overlay_object_id()})).collect::<Vec<_>>(),"captured_overlays":3,"unselected_overlays":1,"source_components_byte_identical":true,"all_returned_inputs_independently_owned":true,"registered_context_or_source_program_proof":false}),
    );
}

#[test]
fn contract_inputs_reject_crossed_missing_extra_and_changed_originals() {
    let p = packet();
    let original = envelope(&p);
    let overlays = buffers(&p, "overlays");
    let mut results = vec![];
    for control in [
        "image-hash",
        "source-components",
        "requested-package",
        "missing-field",
        "unknown-field",
        "reordered",
        "duplicate",
        "missing-raw",
        "extra-raw",
        "changed-raw",
        "coherent-invalid-overlay",
        "case-path-collision",
    ] {
        let mut e = original.clone();
        let mut o = overlays.clone();
        match control {
            "image-hash" => e["image_sha256"] = json!("A".repeat(64)),
            "source-components" => e["source_components_sha256"] = json!("a".repeat(64)),
            "requested-package" => e["requested_package"] = json!("example.com/absent"),
            "missing-field" => {
                e.as_object_mut().unwrap().remove("compiler_sha256");
            }
            "unknown-field" => e["supplied_adapter"] = json!(true),
            "reordered" => {
                e["overlays"].as_array_mut().unwrap().reverse();
            }
            "duplicate" => {
                let x = e["overlays"][0].clone();
                e["overlays"].as_array_mut().unwrap().push(x);
                o.push(o[0].clone());
            }
            "missing-raw" => {
                o.pop();
            }
            "extra-raw" => o.push(
                GoTypeSourceBuffer::new("application", "extra.json", "overlay", b"{}").unwrap(),
            ),
            "changed-raw" => {
                let object = o[0].object();
                o[0] = GoTypeSourceBuffer::new(&object.capability, &object.path, "overlay", b"{}")
                    .unwrap();
            }
            "coherent-invalid-overlay" => {
                let object = o[0].object();
                o[0] = GoTypeSourceBuffer::new(&object.capability, &object.path, "overlay", b"{}")
                    .unwrap();
                e["overlays"] =
                    serde_json::to_value(o.iter().map(|s| s.object()).collect::<Vec<_>>()).unwrap();
                e["overlays"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(|v| v["id"].as_str().unwrap().to_string());
            }
            "case-path-collision" => {
                let object = o[0].object();
                o.push(
                    GoTypeSourceBuffer::new(
                        &object.capability,
                        &object.path.to_ascii_uppercase(),
                        "overlay",
                        o[0].bytes(),
                    )
                    .unwrap(),
                );
                e["overlays"] =
                    serde_json::to_value(o.iter().map(|s| s.object()).collect::<Vec<_>>()).unwrap();
                e["overlays"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(|v| v["id"].as_str().unwrap().to_string());
            }
            _ => unreachable!(),
        }
        seal(&mut e);
        let error = import(&p, &e, &o).unwrap_err();
        results.push(json!({"control":control,"error":error.to_string()}));
    }
    let raw = serde_json::to_string(&original).unwrap();
    let duplicate = format!("{{\"schema\":\"duplicate\",{}", &raw[1..]);
    assert!(import_go_source_contract_inputs(
        duplicate.as_bytes(),
        &unhex(p["components_hex"].as_str().unwrap()),
        &buffers(&p, "sources"),
        &overlays
    )
    .is_err());
    output(
        "contract-input-controls.json",
        &json!({"status":"passed","controls":results,"duplicate_json_key_rejected":true}),
    );
}

#[test]
fn contract_inputs_keep_frozen_aggregate_contract_byte_limit() {
    let p = packet();
    let mut e = envelope(&p);
    let mut o = vec![];
    let original = &buffers(&p, "overlays")[0];
    let mut raw = original.bytes().to_vec();
    raw.resize(1048576, b' ');
    for i in 0..8 {
        o.push(
            GoTypeSourceBuffer::new("application", &format!("limit-{i}.json"), "overlay", &raw)
                .unwrap(),
        );
    }
    e["overlays"] = serde_json::to_value(o.iter().map(|s| s.object()).collect::<Vec<_>>()).unwrap();
    e["overlays"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|v| v["id"].as_str().unwrap().to_string());
    seal(&mut e);
    assert_eq!(import(&p, &e, &o).unwrap().overlays().len(), 8);
    o.push(GoTypeSourceBuffer::new("application", "one-more.json", "overlay", b" ").unwrap());
    e["overlays"] = serde_json::to_value(o.iter().map(|s| s.object()).collect::<Vec<_>>()).unwrap();
    e["overlays"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|v| v["id"].as_str().unwrap().to_string());
    seal(&mut e);
    assert_eq!(
        import(&p, &e, &o),
        Err(GoSourceContractInputError::Input(GoTypeKeyError::Limit))
    );
    output(
        "contract-input-byte-boundary.json",
        &json!({"status":"passed","original_bound":8388608,"exact_bound_accepted":true,"one_byte_over_rejected":true}),
    );
}

#[test]
fn contract_inputs_without_sidecars_use_actual_default_contracts() {
    let p: Value = serde_json::from_slice(
        &std::fs::read(
            root().join("native-final/native-originals/empty/contract-inputs-empty.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let e = envelope(&p);
    let input = import(&p, &e, &[]).unwrap();
    let roots: Vec<GoSourceFunctionSelector> = serde_json::from_value(
        json!([{"package":"example.com/policy","function":"Root","receiver":null}]),
    )
    .unwrap();
    let module = input.derive_contracts(&roots).unwrap();
    assert_eq!(module.functions().len(), 1);
    assert!(module.captured_overlays().is_empty());
    assert_eq!(
        module.functions()[0].origin(),
        &GoSourceContractOrigin::Default
    );
    let mut other = e.clone();
    other["image_sha256"] = json!("a".repeat(64));
    seal(&mut other);
    assert!(
        import(&p, &other, &[]).is_ok(),
        "self-hashed provenance is input linkage, never launcher authentication"
    );
    output(
        "default-contract-inputs.json",
        &json!({"status":"passed","captured_overlays":0,"default_contract":true,"fabricated_sidecar":false,"self_hash_is_not_launcher_authentication":true}),
    );
}
