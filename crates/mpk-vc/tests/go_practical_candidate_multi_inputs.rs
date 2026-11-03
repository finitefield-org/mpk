//! Cross-language check against a complete fresh Go capture, supplied locally
//! by TestPracticalCandidateOriginalPackagesOwnWholeImage.
use mpk_vc::go_practical_context::GoPrivateCandidateRegistry;
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const LIMITS: StrictJsonLimits = StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216);

fn unhex(raw: &str) -> Vec<u8> {
    assert_eq!(raw.len() % 2, 0);
    raw.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}

fn bytes(packet: &Value, key: &str) -> Vec<u8> {
    unhex(packet[key].as_str().unwrap())
}

fn buffers(packet: &Value, key: &str, dir: &Path) -> Vec<GoTypeSourceBuffer> {
    packet[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let raw = if let Some(inline) = row["raw_hex"].as_str() {
                unhex(inline)
            } else {
                let name = row["raw_file"].as_str().unwrap();
                assert!(name.len() == 68 && name.ends_with(".raw"));
                std::fs::read(dir.join("original-source-objects").join(name)).unwrap()
            };
            GoTypeSourceBuffer::import(&serde_json::to_vec(&row["object"]).unwrap(), &raw).unwrap()
        })
        .collect()
}

fn seal(value: &mut Value, domain: &'static str) {
    value.as_object_mut().unwrap().remove("content_sha256");
    let strict = parse_strict_json(&serde_json::to_vec(value).unwrap(), LIMITS).unwrap();
    value["content_sha256"] = json!(hash_canonical_json(HashDomain::new(domain), &strict)
        .unwrap()
        .to_hex());
}

#[test]
#[ignore = "requires the complete fresh Go candidate capture in MPK_GO_CANDIDATE_MULTI_INPUTS"]
fn complete_go_multi_package_capture_binds_independently_in_rust() {
    let dir = PathBuf::from(std::env::var_os("MPK_GO_CANDIDATE_MULTI_INPUTS").unwrap());
    let packet: Value =
        serde_json::from_slice(&std::fs::read(dir.join("candidate-multi-inputs.json")).unwrap())
            .unwrap();
    assert_eq!(packet["snapshot_sources"].as_array().unwrap().len(), 5699);
    let request = GoPrivateCandidateRegistry::new()
        .unwrap()
        .import_request(
            &parse_strict_json(&bytes(&packet, "candidate_request_hex"), LIMITS).unwrap(),
        )
        .unwrap();
    let sources = buffers(&packet, "sources", &dir);
    let overlays = buffers(&packet, "overlays", &dir);
    let selection = import_go_source_selection_inputs(
        &bytes(&packet, "selection_hex"),
        &bytes(&packet, "envelope_hex"),
        &bytes(&packet, "components_hex"),
        &sources,
        &overlays,
    )
    .unwrap();
    assert_eq!(selection.roots().len(), 4);
    assert_eq!(
        selection.contract_inputs().requested_packages(),
        [
            "example.com/c",
            "example.com/policy",
            "example.com/policy/second"
        ]
    );
    assert_eq!(
        selection.contract_inputs().candidate_request_sha256(),
        Some(request.request_sha256())
    );
    bind_go_practical_source_selection(&request, &selection).unwrap();

    // Coherent changes to the transport are still not the original request.
    for control in ["package-set", "request-hash", "physical-root"] {
        let mut contract: Value = serde_json::from_slice(&bytes(&packet, "envelope_hex")).unwrap();
        match control {
            "package-set" => {
                contract["requested_packages"] = json!(["example.com/c", "example.com/policy"])
            }
            "request-hash" => contract["candidate_request_sha256"] = json!("a".repeat(64)),
            "physical-root" => {}
            _ => unreachable!(),
        }
        seal(&mut contract, "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0");
        let mut selected: Value = serde_json::from_slice(&bytes(&packet, "selection_hex")).unwrap();
        selected["source_contract_inputs_sha256"] = contract["content_sha256"].clone();
        if control == "physical-root" {
            let source = packet["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| {
                    row["object"]["capability"] == "workspace/main"
                        && row["object"]["path"] == "policy.go"
                })
                .unwrap()["object"]["id"]
                .as_str()
                .unwrap();
            let roots = selected["roots"].as_array_mut().unwrap();
            let at = roots
                .iter()
                .position(|root| root["source_object_id"] == source)
                .unwrap();
            roots.remove(at);
        }
        seal(&mut selected, "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0");
        let changed = import_go_source_selection_inputs(
            &serde_json::to_vec(&selected).unwrap(),
            &serde_json::to_vec(&contract).unwrap(),
            &bytes(&packet, "components_hex"),
            &sources,
            &overlays,
        );
        if control == "package-set" {
            assert!(changed.is_err(), "accepted {control}");
        } else {
            assert!(
                bind_go_practical_source_selection(&request, &changed.unwrap()).is_err(),
                "accepted {control}"
            );
        }
    }
    let snapshot = import_go_source_snapshot_inputs(
        &bytes(&packet, "snapshot_hex"),
        &buffers(&packet, "snapshot_sources", &dir),
    )
    .unwrap();
    let capture =
        import_go_source_capture_inputs(&bytes(&packet, "capture_hex"), selection, snapshot)
            .unwrap();
    let original: Value = serde_json::from_slice(&bytes(&packet, "export_inputs_hex")).unwrap();
    for control in [
        "package-set",
        "duplicate",
        "order",
        "request-hash",
        "selected-dependency",
        "extra-root",
    ] {
        let mut changed = original.clone();
        if control == "package-set" {
            changed["requested_packages"] = json!(["example.com/c", "example.com/policy"]);
        } else if control == "duplicate" {
            changed["requested_packages"] = json!(["example.com/c", "example.com/c"]);
        } else if control == "order" {
            changed["requested_packages"]
                .as_array_mut()
                .unwrap()
                .reverse();
        } else if control == "request-hash" {
            changed["candidate_request_sha256"] = json!("a".repeat(64));
        } else {
            let raw = unhex(changed["metadata_hex"].as_str().unwrap());
            let mut rows: Vec<Value> = serde_json::Deserializer::from_slice(&raw)
                .into_iter::<Value>()
                .map(Result::unwrap)
                .collect();
            let name = if control == "selected-dependency" {
                "example.com/policy/second"
            } else {
                "net/http"
            };
            let row = rows
                .iter_mut()
                .find(|row| row["ImportPath"] == name)
                .unwrap();
            if control == "selected-dependency" {
                row["DepOnly"] = json!(true);
                row.as_object_mut().unwrap().remove("Match");
            } else {
                row["DepOnly"] = json!(false);
                row["Match"] = json!([name]);
            }
            let mut rewritten = Vec::new();
            for row in rows {
                rewritten.extend(serde_json::to_vec(&row).unwrap());
                rewritten.push(b'\n');
            }
            changed["metadata_hex"] = json!(hex(&rewritten));
        }
        seal(&mut changed, "MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-2.0");
        assert!(
            import_go_source_export_inputs(&serde_json::to_vec(&changed).unwrap(), capture.clone())
                .is_err(),
            "accepted {control}"
        );
    }
    let exports =
        import_go_source_export_inputs(&bytes(&packet, "export_inputs_hex"), capture).unwrap();
    assert_eq!(exports.export_sources().len(), 182);
    let modules =
        import_go_source_module_observations(&bytes(&packet, "module_observations_hex"), exports)
            .unwrap();
    let graph = import_go_source_module_graph_inputs(
        &bytes(&packet, "module_graph_inputs_hex"),
        modules.verify_offline_module_data().unwrap(),
    )
    .unwrap();
    let selected = graph
        .module_data()
        .module_observations()
        .export_inputs()
        .capture_inputs()
        .selection_inputs();
    let bound = bind_go_practical_source_selection(&request, selected).unwrap();
    let bound = bind_go_practical_module_graph(bound, &graph).unwrap();
    let contracts = bound.derive_contracts().unwrap();
    assert_eq!(contracts.contracts().module().selected_roots().len(), 4);
}

#[test]
#[ignore = "requires the complete fresh 8192-root Go capture in MPK_GO_CANDIDATE_MULTI_INPUTS"]
fn complete_8192_root_multi_package_capture_binds_independently_in_rust() {
    let dir = PathBuf::from(std::env::var_os("MPK_GO_CANDIDATE_MULTI_INPUTS").unwrap());
    let packet: Value = serde_json::from_slice(
        &std::fs::read(dir.join("candidate-multi-8192-inputs.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(packet["snapshot_sources"].as_array().unwrap().len(), 5699);
    let request = GoPrivateCandidateRegistry::new()
        .unwrap()
        .import_request(
            &parse_strict_json(&bytes(&packet, "candidate_request_hex"), LIMITS).unwrap(),
        )
        .unwrap();
    let selection = import_go_source_selection_inputs(
        &bytes(&packet, "selection_hex"),
        &bytes(&packet, "envelope_hex"),
        &bytes(&packet, "components_hex"),
        &buffers(&packet, "sources", &dir),
        &buffers(&packet, "overlays", &dir),
    )
    .unwrap();
    assert_eq!(selection.roots().len(), 8192);
    assert_eq!(
        selection.contract_inputs().requested_packages(),
        [
            "example.com/c",
            "example.com/policy",
            "example.com/policy/second"
        ]
    );
    assert_eq!(
        selection.contract_inputs().candidate_request_sha256(),
        Some(request.request_sha256())
    );
    let snapshot = import_go_source_snapshot_inputs(
        &bytes(&packet, "snapshot_hex"),
        &buffers(&packet, "snapshot_sources", &dir),
    )
    .unwrap();
    let capture =
        import_go_source_capture_inputs(&bytes(&packet, "capture_hex"), selection, snapshot)
            .unwrap();
    let exports =
        import_go_source_export_inputs(&bytes(&packet, "export_inputs_hex"), capture).unwrap();
    assert_eq!(exports.export_sources().len(), 182);
    let modules =
        import_go_source_module_observations(&bytes(&packet, "module_observations_hex"), exports)
            .unwrap();
    let graph = import_go_source_module_graph_inputs(
        &bytes(&packet, "module_graph_inputs_hex"),
        modules.verify_offline_module_data().unwrap(),
    )
    .unwrap();
    let selected = graph
        .module_data()
        .module_observations()
        .export_inputs()
        .capture_inputs()
        .selection_inputs();
    let start = std::time::Instant::now();
    let bound = bind_go_practical_source_selection(&request, selected).unwrap();
    eprintln!("bound 8192 original roots in {:?}", start.elapsed());
    let bound = bind_go_practical_module_graph(bound, &graph).unwrap();
    let start = std::time::Instant::now();
    let contracts = bound.derive_contracts().unwrap();
    let module = contracts.contracts().module();
    assert_eq!(module.selected_roots().len(), 8192);
    assert_eq!(module.functions().len(), 8192);
    assert_eq!(module.expression_nodes(), 8192);
    eprintln!(
        "derived 8192 original contract candidates in {:?}",
        start.elapsed()
    );
}
