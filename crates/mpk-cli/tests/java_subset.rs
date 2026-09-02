//! T05 owns source admission; the later CFG/lowering and T10 activation owners
//! exercise this frozen boundary separately.

#[path = "support/java_admission.rs"]
mod harness;

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn frozen_source_matrix_and_all_conversion_contexts_have_executable_fixtures() {
    harness::check_fixtures();
    let profile = harness::profile();
    assert_eq!(
        profile["type_mappings"],
        json!([
            {"source":"boolean", "vir":{"kind":"bool"}},
            {"source":"int", "vir":{"kind":"bv", "width":32, "signed":true}},
            {"source":"long", "vir":{"kind":"bv", "width":64, "signed":true}}
        ])
    );
    let rules = profile["conversion_rules"].as_array().unwrap();
    assert_eq!(rules.len(), 35);
    let contexts: BTreeSet<_> = rules
        .iter()
        .map(|row| row["context"].as_str().unwrap())
        .collect();
    assert_eq!(
        contexts,
        BTreeSet::from([
            "explicit_cast",
            "local_initializer",
            "local_assignment",
            "return",
            "call_argument",
            "binary_operand",
            "conditional_arm"
        ])
    );
    let coverage: BTreeSet<_> = profile["accepted_cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(profile["rejected_cases"].as_array().unwrap())
        .flat_map(|case| case["rows"].as_array().unwrap())
        .map(|row| row.as_str().unwrap())
        .collect();
    assert_eq!(
        coverage,
        (1..=34)
            .map(|row| format!("M{row:02}"))
            .collect::<BTreeSet<_>>()
            .iter()
            .map(String::as_str)
            .collect()
    );
    let limits: BTreeMap<_, _> = profile["limit_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["id"].as_str().unwrap(), row["limit"].as_u64().unwrap()))
        .collect();
    assert_eq!(limits["parameter_slots"], 255);
    assert_eq!(limits["method_closure"], 128);
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn pinned_source_admission_executes_every_owned_case_and_preserves_full_closure() {
    let report = harness::run();
    let profile = harness::profile();
    let cases: BTreeMap<_, _> = report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| (case["id"].as_str().unwrap(), case))
        .collect();
    for id in report["owned_subset_cases"].as_array().unwrap() {
        let case = cases[id.as_str().unwrap()];
        let expected = profile["rejected_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == *id)
            .unwrap();
        assert_eq!(case["code"], expected["expected_code"]);
        assert_eq!(case["phase"], expected["expected_phase"]);
        assert_eq!(case["status"], expected["expected_status"]);
    }
    for expected in profile["accepted_cases"].as_array().unwrap() {
        let id = format!("accepted/{}", expected["id"].as_str().unwrap());
        let case = cases[id.as_str()];
        assert_eq!(case["status"], "admitted");
        let selected: BTreeSet<_> = expected["methods"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();
        let actual: BTreeSet<_> = case["methods"]
            .as_array()
            .unwrap()
            .iter()
            .map(|method| method["id"].as_str().unwrap())
            .collect();
        assert!(selected.is_subset(&actual));
        assert_eq!(
            actual.len(),
            expected["contracts"].as_array().unwrap().len()
        );
    }
    for (id, expected) in [
        (
            "accepted/call.direct",
            vec!["vector.Case::g(int)->int", "vector.Case::f(int)->int"],
        ),
        (
            "accepted/call.dead_branch",
            vec!["vector.Case::g(int)->int", "vector.Case::f(int)->int"],
        ),
        (
            "accepted/call.ordered_arguments",
            vec![
                "vector.Case::add(int,int)->int",
                "vector.Case::left(int)->int",
                "vector.Case::right(int)->int",
                "vector.Case::f(int)->int",
            ],
        ),
    ] {
        let actual: Vec<_> = cases[id]["methods"]
            .as_array()
            .unwrap()
            .iter()
            .map(|method| method["id"].as_str().unwrap())
            .collect();
        assert_eq!(actual, expected);
    }
    for (id, constant) in [
        ("int.minimum", "-2147483648"),
        ("long.minimum", "-9223372036854775808"),
    ] {
        assert_eq!(
            cases[format!("accepted/{id}").as_str()]["methods"][0]["integer_literals"],
            json!([constant])
        );
    }
    for (limit, boundary) in [("method_closure", 128), ("parameter_slots", 255)] {
        assert_eq!(
            cases[format!("limit/{limit}/{boundary}").as_str()]["status"],
            "admitted"
        );
        assert_eq!(
            cases[format!("limit/{limit}/{}", boundary + 1).as_str()]["code"],
            format!("JAVA_LIMIT_{}", limit.to_uppercase())
        );
    }
    let rows: Vec<_> = profile["semantic_rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["row"].clone())
        .collect();
    assert_eq!(report["owned_semantic_rows"], Value::Array(rows));
}
