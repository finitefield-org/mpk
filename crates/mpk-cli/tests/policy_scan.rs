use mpk_cli::policy_scan::{
    PolicyScanContract, PolicyScanContractStatus, PolicyScanEvidenceLabel, PolicyScanFeature,
    PolicyScanLocation, PolicyScanPrecondition, PolicyScanPreconditionSource, PolicyScanReadiness,
    PolicyScanReadinessStatus, PolicyScanReport, PolicyScanSource, PolicyScanSourceFile,
    PolicyScanTarget, POLICY_SCAN_SCHEMA,
};
use serde_json::{json, Value};

const ORDER_POLICY_FUNCTION: &str = "example.com/orderpolicy.ApprovedReserveCents";
const ORDER_POLICY_SOURCE_HASH: &str =
    "4b8fab6e2f2d9e20dc77eee7f1b8813fc423acd858d1dab802259725f1801948";
const ORDER_POLICY_CONTRACT_HASH: &str =
    "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00";
const ORDER_POLICY_GIR_HASH: &str =
    "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950";
const GO2GIR_FIXTURE_HASH: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";

#[test]
fn ready_order_policy_scan_snapshot_is_deterministic() {
    let report = ready_order_policy_report();
    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert_eq!(
        first,
        r#"{
  "schema": "mpk.policy.scan.v0",
  "target": {
    "package_path": "example.com/orderpolicy",
    "function_id": "example.com/orderpolicy.ApprovedReserveCents"
  },
  "source": {
    "root": "examples/order_policy",
    "go_toolchain": "go1.23",
    "go2gir_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    "source_sha256": "4b8fab6e2f2d9e20dc77eee7f1b8813fc423acd858d1dab802259725f1801948",
    "gir_sha256": "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950",
    "files": [
      {
        "path": "examples/order_policy/policy.go",
        "sha256": "4b8fab6e2f2d9e20dc77eee7f1b8813fc423acd858d1dab802259725f1801948"
      }
    ]
  },
  "contract": {
    "path": "examples/order_policy/policy_contract.json",
    "schema": "mpk.go.contract.v0",
    "sha256": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00",
    "status": "function_resolved",
    "function_id": "example.com/orderpolicy.ApprovedReserveCents"
  },
  "readiness": {
    "status": "ready",
    "summary": "function is within Go subset v0 and the contract resolves"
  },
  "supported_features": [
    {
      "code": "GO_SUBSET_TOP_LEVEL_FUNCTION",
      "message": "top-level pure function is supported",
      "source_path": "examples/order_policy/policy.go",
      "function_id": "example.com/orderpolicy.ApprovedReserveCents",
      "evidence_label": "helper_evidence",
      "location": {
        "line": 4,
        "column": 1
      }
    },
    {
      "code": "GO_SUBSET_IF_RETURN",
      "message": "if/return control flow is supported",
      "source_path": "examples/order_policy/policy.go",
      "function_id": "example.com/orderpolicy.ApprovedReserveCents",
      "evidence_label": "helper_evidence",
      "location": {
        "line": 6,
        "column": 2
      }
    }
  ],
  "rejected_features": [],
  "preconditions": [
    {
      "id": "requires[0]",
      "expression": "balanceCents >= 0",
      "source": "contract_requires",
      "source_path": "examples/order_policy/policy_contract.json",
      "function_id": "example.com/orderpolicy.ApprovedReserveCents",
      "evidence_label": "helper_evidence"
    },
    {
      "id": "requires[1]",
      "expression": "requestedCents >= 0",
      "source": "contract_requires",
      "source_path": "examples/order_policy/policy_contract.json",
      "function_id": "example.com/orderpolicy.ApprovedReserveCents",
      "evidence_label": "helper_evidence"
    }
  ]
}
"#
    );

    let reparsed = PolicyScanReport::from_json(&first).expect("valid schema parses");
    assert_eq!(reparsed, report);
}

#[test]
fn unsupported_policy_scan_snapshot_is_deterministic() {
    let report = unsupported_policy_report();
    let json = report.to_deterministic_json().expect("serializes");

    assert_eq!(
        json,
        r#"{
  "schema": "mpk.policy.scan.v0",
  "target": {
    "package_path": "example.com/orderpolicy",
    "function_id": "example.com/orderpolicy.ApprovedReserveFromLedger"
  },
  "source": {
    "root": "examples/order_policy",
    "go_toolchain": "go1.23",
    "go2gir_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    "source_sha256": "2222222222222222222222222222222222222222222222222222222222222222",
    "gir_sha256": null,
    "files": [
      {
        "path": "examples/order_policy/unsupported_policy.go",
        "sha256": "2222222222222222222222222222222222222222222222222222222222222222"
      }
    ]
  },
  "contract": {
    "path": null,
    "schema": null,
    "sha256": null,
    "status": "not_provided",
    "function_id": null
  },
  "readiness": {
    "status": "unsupported",
    "summary": "function uses map iteration, which is outside Go subset v0"
  },
  "supported_features": [],
  "rejected_features": [
    {
      "code": "GO_SUBSET_MAP",
      "message": "maps are not supported in Go subset v0",
      "source_path": "examples/order_policy/unsupported_policy.go",
      "function_id": "example.com/orderpolicy.ApprovedReserveFromLedger",
      "evidence_label": "helper_evidence",
      "location": {
        "line": 5,
        "column": 18
      }
    }
  ],
  "preconditions": []
}
"#
    );
}

#[test]
fn policy_scan_schema_status_values_are_exact() {
    assert_eq!(POLICY_SCAN_SCHEMA, "mpk.policy.scan.v0");
    assert_eq!(
        serde_json::to_value([
            PolicyScanReadinessStatus::Ready,
            PolicyScanReadinessStatus::NeedsRefactor,
            PolicyScanReadinessStatus::Unsupported,
        ])
        .expect("serializes"),
        json!(["ready", "needs_refactor", "unsupported"])
    );
    assert!(serde_json::from_value::<PolicyScanReadinessStatus>(json!("blocked")).is_err());
}

#[test]
fn scan_output_has_no_proof_acceptance_fields() {
    let json = ready_order_policy_report()
        .to_deterministic_json()
        .expect("serializes");

    assert!(!json.contains("proof_acceptance"));
    assert!(!json.contains("verified_properties"));
    assert!(!json.contains("\"verified\""));
    assert!(!json.contains("\"accepted\""));
}

#[test]
fn unknown_fields_reject_for_scan_report_deserialization() {
    assert_unknown_top_level_field_rejects("proof_acceptance", json!(false));
    assert_unknown_top_level_field_rejects("verified_properties", json!([]));
    assert_unknown_nested_object_field_rejects(["source"].as_slice(), "proof_acceptance");
    assert_unknown_nested_object_field_rejects(["contract"].as_slice(), "proof_acceptance");
    assert_unknown_nested_object_field_rejects(["readiness"].as_slice(), "proof_acceptance");

    let mut value = serde_json::to_value(ready_order_policy_report()).expect("value serializes");
    value
        .get_mut("supported_features")
        .and_then(Value::as_array_mut)
        .and_then(|features| features.first_mut())
        .and_then(Value::as_object_mut)
        .expect("feature object")
        .insert("proof_acceptance".to_owned(), json!(false));

    assert!(serde_json::from_value::<PolicyScanReport>(value).is_err());

    let mut value = serde_json::to_value(ready_order_policy_report()).expect("value serializes");
    value
        .get_mut("preconditions")
        .and_then(Value::as_array_mut)
        .and_then(|preconditions| preconditions.first_mut())
        .and_then(Value::as_object_mut)
        .expect("precondition object")
        .insert("proof_acceptance".to_owned(), json!(false));

    assert!(serde_json::from_value::<PolicyScanReport>(value).is_err());
}

#[test]
fn schema_mismatch_rejects_for_policy_scan_report() {
    let mut value = serde_json::to_value(ready_order_policy_report()).expect("value serializes");
    value
        .as_object_mut()
        .expect("report object")
        .insert("schema".to_owned(), json!("mpk.policy.scan.v1"));
    let json = serde_json::to_string(&value).expect("value serializes");

    let error = PolicyScanReport::from_json(&json).expect_err("schema mismatch rejects");
    assert!(error.to_string().contains("schema mismatch"));
}

fn ready_order_policy_report() -> PolicyScanReport {
    let mut source = PolicyScanSource::new(
        "examples/order_policy",
        "go1.23",
        GO2GIR_FIXTURE_HASH,
        ORDER_POLICY_SOURCE_HASH,
    );
    source.gir_sha256 = Some(ORDER_POLICY_GIR_HASH.to_owned());
    source.files.push(PolicyScanSourceFile::new(
        "examples/order_policy/policy.go",
        ORDER_POLICY_SOURCE_HASH,
    ));

    let mut report = PolicyScanReport::new(
        PolicyScanTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        source,
        PolicyScanContract::resolved(
            "examples/order_policy/policy_contract.json",
            "mpk.go.contract.v0",
            ORDER_POLICY_CONTRACT_HASH,
            ORDER_POLICY_FUNCTION,
        ),
        PolicyScanReadiness::new(
            PolicyScanReadinessStatus::Ready,
            "function is within Go subset v0 and the contract resolves",
        ),
    );

    let mut top_level_function = PolicyScanFeature::helper(
        "GO_SUBSET_TOP_LEVEL_FUNCTION",
        "top-level pure function is supported",
        Some("examples/order_policy/policy.go".to_owned()),
        Some(ORDER_POLICY_FUNCTION.to_owned()),
    );
    top_level_function.location = Some(PolicyScanLocation::new(4, 1));
    report.supported_features.push(top_level_function);

    let mut if_return = PolicyScanFeature::helper(
        "GO_SUBSET_IF_RETURN",
        "if/return control flow is supported",
        Some("examples/order_policy/policy.go".to_owned()),
        Some(ORDER_POLICY_FUNCTION.to_owned()),
    );
    if_return.location = Some(PolicyScanLocation::new(6, 2));
    report.supported_features.push(if_return);

    report.preconditions.push(PolicyScanPrecondition::helper(
        "requires[0]",
        "balanceCents >= 0",
        PolicyScanPreconditionSource::ContractRequires,
        Some("examples/order_policy/policy_contract.json".to_owned()),
        Some(ORDER_POLICY_FUNCTION.to_owned()),
    ));
    report.preconditions.push(PolicyScanPrecondition::helper(
        "requires[1]",
        "requestedCents >= 0",
        PolicyScanPreconditionSource::ContractRequires,
        Some("examples/order_policy/policy_contract.json".to_owned()),
        Some(ORDER_POLICY_FUNCTION.to_owned()),
    ));

    report
}

fn unsupported_policy_report() -> PolicyScanReport {
    let unsupported_function = "example.com/orderpolicy.ApprovedReserveFromLedger";
    let source_hash = "2222222222222222222222222222222222222222222222222222222222222222";
    let mut source = PolicyScanSource::new(
        "examples/order_policy",
        "go1.23",
        GO2GIR_FIXTURE_HASH,
        source_hash,
    );
    source.files.push(PolicyScanSourceFile::new(
        "examples/order_policy/unsupported_policy.go",
        source_hash,
    ));

    let mut report = PolicyScanReport::new(
        PolicyScanTarget::new("example.com/orderpolicy", unsupported_function),
        source,
        PolicyScanContract {
            path: None,
            schema: None,
            sha256: None,
            status: PolicyScanContractStatus::NotProvided,
            function_id: None,
        },
        PolicyScanReadiness::new(
            PolicyScanReadinessStatus::Unsupported,
            "function uses map iteration, which is outside Go subset v0",
        ),
    );

    let mut map_feature = PolicyScanFeature::helper(
        "GO_SUBSET_MAP",
        "maps are not supported in Go subset v0",
        Some("examples/order_policy/unsupported_policy.go".to_owned()),
        Some(unsupported_function.to_owned()),
    );
    map_feature.location = Some(PolicyScanLocation::new(5, 18));
    report.rejected_features.push(map_feature);

    report
}

fn assert_unknown_top_level_field_rejects(field: &str, field_value: Value) {
    let mut value = serde_json::to_value(ready_order_policy_report()).expect("value serializes");
    value
        .as_object_mut()
        .expect("report object")
        .insert(field.to_owned(), field_value);

    let error = serde_json::from_value::<PolicyScanReport>(value)
        .expect_err("unknown top-level field rejects");
    assert!(error.to_string().contains("unknown field"));
}

fn assert_unknown_nested_object_field_rejects(path: &[&str], field: &str) {
    let mut value = serde_json::to_value(ready_order_policy_report()).expect("value serializes");
    let mut current = &mut value;
    for key in path {
        current = current.get_mut(*key).expect("path component exists");
    }
    current
        .as_object_mut()
        .expect("nested object")
        .insert(field.to_owned(), json!(false));

    let error = serde_json::from_value::<PolicyScanReport>(value)
        .expect_err("unknown nested field rejects");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn feature_reports_are_labeled_helper_evidence() {
    let reports = [ready_order_policy_report(), unsupported_policy_report()];

    for report in reports {
        for feature in report
            .supported_features
            .iter()
            .chain(report.rejected_features.iter())
        {
            assert_eq!(
                feature.evidence_label,
                PolicyScanEvidenceLabel::HelperEvidence
            );
            assert!(feature.source_path.is_some());
            assert!(feature.function_id.is_some());
        }
    }
}
