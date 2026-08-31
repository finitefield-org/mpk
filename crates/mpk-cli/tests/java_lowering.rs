//! T06: real Java output is imported by the independent, inactive revision-3 validators.

#[path = "support/java_lowering.rs"]
mod harness;

use serde_json::{json, Value};
use std::collections::BTreeSet;

#[test]
fn all_frozen_operations_cfgs_cases_and_emission_counters_have_private_fixtures() {
    harness::check_fixtures();
    let profile = harness::profile();
    assert_eq!(profile["accepted_cases"].as_array().unwrap().len(), 49);
    assert_eq!(profile["operation_mappings"].as_array().unwrap().len(), 27);
    assert_eq!(profile["cfg_patterns"].as_array().unwrap().len(), 6);
    let lowering: Vec<_> = profile["rejected_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["id"].as_str().unwrap().starts_with("lowering."))
        .collect();
    assert_eq!(lowering.len(), 7);
    for case in lowering {
        assert_eq!(case["expected_phase"], "lowering");
    }
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn pinned_lowering_emits_complete_deterministic_artifacts_and_preserves_java_evaluation() {
    let report = harness::run();
    let profile = harness::profile();
    let cases = report["cases"].as_array().unwrap();
    let actual: BTreeSet<_> = cases
        .iter()
        .filter_map(|case| case["id"].as_str().unwrap().strip_prefix("accepted/"))
        .collect();
    let expected: BTreeSet<_> = profile["accepted_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["id"].as_str().unwrap().starts_with("operation/"))
            .count(),
        27
    );
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["id"].as_str().unwrap().starts_with("cfg/"))
            .count(),
        6
    );
    assert!(report["evaluation_count"].as_u64().unwrap() >= 100);
    assert_eq!(report["counter_boundaries"].as_array().unwrap().len(), 9);
    let failures: BTreeSet<_> = report["mutations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect();
    for case in profile["rejected_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if id.starts_with("lowering.") {
            assert!(failures.contains(id));
        }
    }
    assert_eq!(
        harness::case(report, "precedence/contract_before_lowering")["phase"],
        "subset"
    );
    assert_eq!(
        harness::case(report, "limit/block-method")["code"],
        "JAVA_LIMIT_CFG_BLOCKS_PER_METHOD"
    );

    // Nested eager operands must use explicit block parameters, without new locals.
    let live = harness::envelope(harness::case(report, "extra/live-prefix"));
    let function = &live["ir"]["value"]["units"][0]["functions"][0];
    assert_eq!(function["locals"], json!([]));
    assert!(function["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|block| block["parameters"].as_array().unwrap().len() == 2));
    // javac's negative literal and an explicit parenthesized negation stay distinct.
    for (id, expected) in [
        ("accepted/int.minimum", vec!["Const"]),
        ("accepted/literal.parenthesized", vec!["Const", "UnaryOp"]),
        (
            "extra/constant-no-fold",
            vec!["Const", "Const", "Const", "BinOp", "BinOp"],
        ),
    ] {
        let value = harness::envelope(harness::case(report, id));
        let instructions = value["ir"]["value"]["units"][0]["functions"][0]["blocks"][0]
            ["instructions"]
            .as_array()
            .unwrap();
        assert_eq!(
            instructions
                .iter()
                .map(|instruction| instruction["kind"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn shared_validators_reject_rehashed_java_check_shift_and_scope_mutations() {
    let report = harness::run();
    for mutation in 0..5 {
        let id = match mutation {
            0 | 1 => "accepted/int.division",
            2 | 3 => "accepted/int.shift_unsigned_right",
            _ => "extra/live-prefix",
        };
        let case = harness::case(report, id);
        let mut envelope = harness::envelope(case);
        let request = harness::Request::new(&envelope["selection"]);
        let blocks = envelope["ir"]["value"]["units"][0]["functions"][0]["blocks"]
            .as_array_mut()
            .unwrap();
        match mutation {
            0 => blocks[0]["instructions"][0]["safety_checks"] = json!([]),
            1 => {
                blocks[0]["instructions"][0]["safety_checks"] =
                    json!([{"kind":"divisor_nonzero"}, {"kind":"signed_divrem_representable"}])
            }
            2 => blocks[0]["instructions"][0]["value"]["int"]["value"] = json!("63"),
            3 => {
                let shift = blocks[0]["instructions"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|instruction| instruction["op"] == "bv_lshr")
                    .unwrap();
                shift["rhs"] = json!({"var":"arg1"});
            }
            _ => {
                let external = blocks[0]["instructions"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()["id"]
                    .clone();
                let join = blocks
                    .iter_mut()
                    .find(|block| block["parameters"].as_array().unwrap().len() == 2)
                    .unwrap();
                join["instructions"][0]["lhs"] = json!({"var":external});
            }
        }
        harness::refresh(&mut envelope);
        assert!(
            request
                .validate(
                    &harness::canonical_line(&envelope),
                    0,
                    &harness::captured(case)
                )
                .is_err(),
            "mutation {mutation}"
        );
    }
    // Every producer-side rejection was separately accepted as artifact-free by harness::run.
    for case in report["mutations"].as_array().unwrap() {
        assert_eq!(case["published_bytes"], Value::from(0));
    }
}
