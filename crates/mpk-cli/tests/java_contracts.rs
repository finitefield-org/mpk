//! T05 owns strict sidecar parsing, complete attachment, typing and normalization.

#[path = "support/java_admission.rs"]
mod harness;

use mpk_vc::successor_source_artifacts::successor_contract_hash_value;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn hash(domain: &str, value: &Value) -> String {
    let mut digest = Sha256::new();
    digest.update(domain.as_bytes());
    digest.update([0]);
    digest.update(serde_json::to_vec(value).unwrap());
    format!("{:x}", digest.finalize())
}

#[test]
fn strict_contract_fixtures_cover_frozen_hashes_and_executable_boundaries() {
    harness::check_fixtures();
    let profile = harness::profile();
    assert_eq!(
        hash(
            "MPK-JAVA-CONTRACT-SIDECAR-0.1",
            &profile["contract_fixture"]
        ),
        profile["contract_sidecar_sha256"]
    );
    assert_eq!(
        successor_contract_hash_value(&profile["normalized_contract_fixture"])
            .unwrap()
            .as_str(),
        profile["normalized_contract_fixture"]["contract_hash"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&profile["contract_fixture"])
            .unwrap()
            .len(),
        430
    );
    let limits: BTreeMap<_, _> = profile["limit_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["id"].as_str().unwrap(), row["limit"].as_u64().unwrap()))
        .collect();
    for (key, maximum) in [
        ("contract_clauses", 64),
        ("contract_nodes_per_method", 1024),
        ("contract_nodes_per_closure", 8192),
        ("contract_depth", 32),
    ] {
        assert_eq!(limits[key], maximum);
    }
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn pinned_contract_executor_matches_independent_normalized_hashes_and_all_refusals() {
    let report = harness::run();
    let profile = harness::profile();
    let cases: BTreeMap<_, _> = report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| (case["id"].as_str().unwrap(), case))
        .collect();
    for id in report["owned_contract_cases"].as_array().unwrap() {
        let actual = cases[id.as_str().unwrap()];
        let expected = profile["rejected_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == *id)
            .unwrap();
        assert_eq!(actual["code"], expected["expected_code"]);
        assert_eq!(actual["phase"], "subset");
        assert_eq!(actual["status"], "rejected");
    }
    for case in cases.values().filter(|case| case["status"] == "admitted") {
        for attached in case["contracts"].as_array().unwrap() {
            let normalized = &attached["normalized"];
            assert_eq!(
                successor_contract_hash_value(normalized).unwrap().as_str(),
                normalized["contract_hash"].as_str().unwrap()
            );
            assert_eq!(
                hash("MPK-JAVA-CONTRACT-SIDECAR-0.1", &attached["sidecar"]),
                attached["sidecar_sha256"]
            );
            assert_eq!(
                normalized["semantic_context"],
                profile["semantic_context_fixture"]
            );
            assert_eq!(normalized["modifies"], json!([]));
            assert_eq!(normalized["loops"], json!([]));
            assert_eq!(normalized["panic"], "forbidden");
            assert_eq!(normalized["termination"], "total");
        }
    }
    assert_eq!(
        cases["contract-fixture"]["contracts"][0]["normalized"],
        profile["normalized_contract_fixture"]
    );
    assert_eq!(
        cases["contract-fixture"]["selection_sha256"],
        profile["selection_sha256"]
    );
    for (key, boundary) in [
        ("contract_clauses", 64),
        ("contract_nodes_per_method", 1024),
        ("contract_nodes_per_closure", 8192),
        ("contract_depth", 32),
    ] {
        let accepted = cases[format!("limit/{key}/{boundary}").as_str()];
        assert_eq!(accepted["status"], "admitted");
        assert_eq!(
            cases[format!("limit/{key}/{}", boundary + 1).as_str()]["code"],
            format!("JAVA_LIMIT_{}", key.to_uppercase())
        );
        if key.contains("nodes") {
            assert_eq!(accepted["contract_nodes"], boundary);
        }
    }
    assert_eq!(report["link_failure"]["code"], "JAVA_CONTRACT_HASH");
}
