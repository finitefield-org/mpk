use std::collections::BTreeSet;

use mpk_vc::import_vir_json;
use serde_json::Value;

const GO_PROFILE: &[u8] = include_bytes!("../../../develop/specs/vectors/go-vir-profile-v0.json");
const VIR_VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/vir-v0.json");

#[test]
fn every_go_profile_vector_has_a_unique_owned_outcome() {
    let vectors: Value = serde_json::from_slice(GO_PROFILE).expect("Go profile vectors");
    assert_eq!(vectors["schema"], "mpk.go.vir_profile.conformance.v0");
    assert_eq!(vectors["spec_profile"], "mpk.go.fixed.v0");
    let groups = [
        ("profile_cases", 6),
        ("capture_cases", 27),
        ("source_cases", 22),
        ("operation_cases", 17),
        ("contract_cases", 21),
        ("loop_call_cases", 10),
        ("diagnostic_cases", 7),
        ("limit_cases", 20),
    ];
    let mut visited = BTreeSet::new();
    for (group, count) in groups {
        let cases = vectors[group].as_array().expect("Go vector case group");
        assert_eq!(cases.len(), count, "{group}");
        for case in cases {
            let id = case["id"].as_str().expect("Go vector case ID");
            assert!(visited.insert(id), "duplicate Go vector case {id}");
            assert!(case["expect"]["outcome"].is_string(), "{id} has no outcome");
        }
    }
    assert_eq!(visited.len(), 130);
}

#[test]
fn shared_go_identity_is_accepted_by_the_rust_vir_importer() {
    let vectors: Value = serde_json::from_slice(VIR_VECTORS).expect("VIR vectors");
    let input = vectors["module_cases"]
        .as_array()
        .expect("VIR module cases")
        .iter()
        .find(|case| case["id"] == "module.valid_go_identity")
        .expect("Go identity VIR")["input"]
        .clone();
    import_vir_json(&serde_json::to_vec(&input).expect("serialize Go identity VIR"))
        .expect("Rust importer accepts Go identity VIR");
}
