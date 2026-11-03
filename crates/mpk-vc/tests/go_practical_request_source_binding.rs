use mpk_vc::go_practical_context::{GoPrivateCandidateRegistry, GoPrivateCandidateRequest};
use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
    StrictJsonValue,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const LIMITS: StrictJsonLimits = StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216);

fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/selection-inputs")
}

fn packet(path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(evidence().join(path)).unwrap()).unwrap()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn buffers(packet: &Value, name: &str) -> Vec<GoTypeSourceBuffer> {
    packet[name]
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

fn strict(value: &Value) -> StrictJsonValue {
    parse_strict_json(&serde_json::to_vec(value).unwrap(), LIMITS).unwrap()
}

fn seal(value: &mut Value, field: &str, domain: &'static str) {
    value.as_object_mut().unwrap().remove(field);
    value[field] = json!(hash_canonical_json(HashDomain::new(domain), &strict(value))
        .unwrap()
        .to_hex());
}

fn input(packet: &Value, wire: &Value) -> GoSourceSelectionInputs {
    import_go_source_selection_inputs(
        &serde_json::to_vec(wire).unwrap(),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(packet, "sources"),
        &buffers(packet, "overlays"),
    )
    .unwrap()
}

fn selection_wire(packet: &Value) -> Value {
    serde_json::from_slice(&unhex(packet["selection_hex"].as_str().unwrap())).unwrap()
}

fn root(function: &str, receiver: Option<&str>) -> Value {
    json!({"package":"example.com/policy", "function":function,
        "receiver":receiver.map(|name| json!({"package":"example.com/policy", "type_name":name}))})
}

fn request(mut roots: Vec<Value>) -> GoPrivateCandidateRequest {
    roots.sort_by_cached_key(|root| canonical_json_bytes(&strict(root)).unwrap());
    let mut wire: Value = serde_json::from_slice(include_bytes!(
        "fixtures/go-practical-candidate-request.json"
    ))
    .unwrap();
    wire["selection"]["value"]["roots"] = json!(roots);
    seal(
        &mut wire,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    GoPrivateCandidateRegistry::new()
        .unwrap()
        .import_request(&strict(&wire))
        .unwrap()
}

fn original_packet() -> Value {
    packet("native/native-originals/selection-owned/selection-inputs.json")
}

fn original_roots() -> Vec<Value> {
    vec![
        root("Root", None),
        root("Work", Some("Count")),
        root("Work", Some("Other")),
    ]
}

#[test]
fn candidate_contract_keeps_request_and_physical_selection_in_agreement() {
    let packet = original_packet();
    let request = request(original_roots());
    let sources = buffers(&packet, "sources");
    let overlays = buffers(&packet, "overlays");
    let components = unhex(packet["components_hex"].as_str().unwrap());
    let mut contract: Value =
        serde_json::from_slice(&unhex(packet["envelope_hex"].as_str().unwrap())).unwrap();
    contract
        .as_object_mut()
        .unwrap()
        .remove("requested_package");
    contract["schema"] = json!("mpk.go.practical.source_contract_inputs.v2");
    contract["requested_packages"] = json!(["example.com/policy"]);
    contract["candidate_request_sha256"] = json!(request.request_sha256());
    seal(
        &mut contract,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0",
    );
    let mut selection = selection_wire(&packet);
    selection["source_contract_inputs_sha256"] = contract["content_sha256"].clone();
    seal(
        &mut selection,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    let import = |selection: &Value, contract: &Value| {
        import_go_source_selection_inputs(
            &serde_json::to_vec(selection).unwrap(),
            &serde_json::to_vec(contract).unwrap(),
            &components,
            &sources,
            &overlays,
        )
    };
    let original = import(&selection, &contract).unwrap();
    bind_go_practical_source_selection(&request, &original).unwrap();

    let mut extra = contract.clone();
    extra["requested_packages"] = json!(["example.com/policy", "example.com/policy/helper"]);
    seal(
        &mut extra,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0",
    );
    let mut extra_selection = selection.clone();
    extra_selection["source_contract_inputs_sha256"] = extra["content_sha256"].clone();
    seal(
        &mut extra_selection,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    assert!(import(&extra_selection, &extra).is_err());

    let mut different = contract.clone();
    different["candidate_request_sha256"] = json!("a".repeat(64));
    seal(
        &mut different,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0",
    );
    let mut different_selection = selection.clone();
    different_selection["source_contract_inputs_sha256"] = different["content_sha256"].clone();
    seal(
        &mut different_selection,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    let different = import(&different_selection, &different).unwrap();
    assert!(bind_go_practical_source_selection(&request, &different).is_err());

    let mut missing = selection;
    missing["roots"].as_array_mut().unwrap().pop();
    seal(
        &mut missing,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    let missing = import(&missing, &contract).unwrap();
    assert!(bind_go_practical_source_selection(&request, &missing).is_err());
}

#[test]
fn binds_complete_original_named_roots_to_their_physical_declarations() {
    let packet = original_packet();
    let wire = selection_wire(&packet);
    let inputs = input(&packet, &wire);
    let original = inputs.canonical_bytes().to_vec();
    assert_eq!(inputs.roots().len(), 3);
    assert_eq!(inputs.contract_inputs().overlays().len(), 3);
    let request = request(original_roots());
    let binding = bind_go_practical_source_selection(&request, &inputs).unwrap();
    assert_eq!(binding.source_inputs().canonical_bytes(), original);
    assert_eq!(binding.source_inputs().roots().len(), 3);
    assert_eq!(
        binding.request().context().semantic_profile(),
        "mpk.go.practical.v1"
    );
    let expected = inputs.derive_contracts().unwrap();
    let contracts = binding.derive_contracts().unwrap();
    assert_eq!(contracts.module(), &expected);
    assert_eq!(contracts.module().selected_roots().len(), 3);
    assert_eq!(contracts.module().functions().len(), 4);
    assert_eq!(
        contracts
            .module()
            .functions()
            .iter()
            .filter(|function| function.overlay_object_id().is_some())
            .count(),
        2
    );
    let invocations = contracts.derive_invocation_catalogue().unwrap();
    let module = invocations.contracts().module();
    let sites = invocations.catalogue().sites();
    assert_eq!(sites.len(), 1);
    let site = &sites[0];
    assert!(module
        .functions()
        .iter()
        .any(|function| function.is_selected_root() && function.function() == site.caller()));
    assert_eq!(site.targets().len(), 1);
    assert!(module.functions().iter().any(|function| {
        !function.is_selected_root() && function.function() == site.targets()[0].callee()
    }));
    assert_eq!(site.input_type_ids().len(), 1);
    assert_eq!(site.result_type_ids().len(), 1);
    let row = invocations
        .catalogue()
        .generate_row(site.caller(), site.node())
        .unwrap();
    let checked = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
    assert!(checked.accepted, "{}", checked.json);
    let report: Value = serde_json::from_str(&checked.json).unwrap();
    assert_eq!(report["axiom_count"], 0);
    if let Some(dir) = std::env::var_os("MPK_GO_CANDIDATE_INVOCATION_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in [
            ("candidate-invocation.mpcert", row.certificate_bytes()),
            ("candidate-invocation.rust.json", checked.json.as_bytes()),
        ] {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(dir.join(name))
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
    }
}

#[test]
fn rejects_other_coherent_source_roots_and_missing_or_unknown_requests() {
    let packet = original_packet();
    let original = selection_wire(&packet);
    let mut wrong_name = original_roots();
    wrong_name[0] = root("Standalone", None);
    assert!(matches!(
        bind_go_practical_source_selection(&request(wrong_name), &input(&packet, &original)),
        Err(GoPracticalSourceBindingError::Mismatch { root: Some(0) })
    ));
    assert!(matches!(
        bind_go_practical_source_selection(
            &request(vec![root("Root", None)]),
            &input(&packet, &original)
        ),
        Err(GoPracticalSourceBindingError::Mismatch { root: None })
    ));
    assert!(matches!(
        bind_go_practical_source_selection(
            &request(vec![root("Missing", None)]),
            &input(&packet, &original)
        ),
        Err(GoPracticalSourceBindingError::Resolve { root: Some(0), .. })
    ));

    // Reseal a coherent physical selection of the actual original Standalone
    // function. It remains a valid independent owner, but not for Root.
    let observations =
        self::packet("native/native-originals/selection-owned/physical-root-observations.json");
    let physical = &observations["functions"]["example.com/policy.Standalone"];
    let mut changed = original.clone();
    let package_id = changed["roots"][0]["package_id"].clone();
    changed["roots"] = json!([{"package_id":package_id, "source_object_id":physical["source"],
        "start_byte":physical["start"], "end_byte":physical["end"]}]);
    seal(
        &mut changed,
        "content_sha256",
        "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0",
    );
    let independent = input(&packet, &changed);
    assert_eq!(independent.roots().len(), 1);
    assert!(matches!(
        bind_go_practical_source_selection(&request(vec![root("Root", None)]), &independent),
        Err(GoPracticalSourceBindingError::Mismatch { root: Some(0) })
    ));
}

#[test]
fn binds_ordinary_init_method_but_not_a_package_initializer() {
    let packet = packet("native-final/native-originals/method-init/selection-method-init.json");
    let inputs = input(&packet, &selection_wire(&packet));
    assert!(bind_go_practical_source_selection(
        &request(vec![root("init", Some("Count"))]),
        &inputs
    )
    .is_ok());
}

#[test]
fn retained_profile_request_cannot_be_used_as_practical_source_authority() {
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let compiled: Value = serde_json::from_slice(registry.canonical_bytes()).unwrap();
    let mut wire: Value = serde_json::from_slice(include_bytes!(
        "fixtures/go-practical-candidate-request.json"
    ))
    .unwrap();
    wire["semantic_context"]["profile_entry_sha256"] =
        compiled["profiles"][0]["entry_sha256"].clone();
    wire["semantic_context"]["semantic_profile"] = json!("mpk.go.fixed.v0");
    wire["semantic_context"]["semantic_parameters"] = json!({
        "schema":"mpk.semantic_parameters.go_fixed.v0", "value":{"target_id":"linux/amd64", "pointer_width":64}
    });
    wire["semantic_context"]["foundation_descriptor"] = Value::Null;
    wire["selection"] = json!({"schema":"mpk.selection.go_function.v0", "value":{"package":"quote", "function":"quote.Quote"}});
    seal(
        &mut wire,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    let request = registry.import_request(&strict(&wire)).unwrap();
    let packet = original_packet();
    assert!(matches!(
        bind_go_practical_source_selection(&request, &input(&packet, &selection_wire(&packet))),
        Err(GoPracticalSourceBindingError::Profile)
    ));
}

#[test]
fn binds_all_8192_original_roots_without_narrowing_the_source_set() {
    let packet = packet("native/native-originals/selection-owned/selection-boundary-inputs.json");
    let inputs = input(&packet, &selection_wire(&packet));
    assert_eq!(inputs.roots().len(), 8192);
    let names = (0..8192)
        .map(|i| root(&format!("Bound{i:04}"), None))
        .collect();
    let started = std::time::Instant::now();
    let request = request(names);
    let binding = bind_go_practical_source_selection(&request, &inputs).unwrap();
    assert_eq!(binding.source_inputs().roots().len(), 8192);
    eprintln!("bound all 8192 original roots in {:?}", started.elapsed());
    let started = std::time::Instant::now();
    let contracts = binding.derive_contracts().unwrap();
    let module = contracts.module();
    assert_eq!(module.selected_roots().len(), 8192);
    assert_eq!(module.functions().len(), 8192);
    assert_eq!(module.expression_nodes(), 8192);
    assert!(module.functions().iter().all(|function| {
        function.is_selected_root()
            && function.origin() == &GoSourceContractOrigin::Default
            && function.termination() == "partial"
    }));
    eprintln!(
        "derived all 8192 original contract candidates in {:?}",
        started.elapsed()
    );
}
