use mpk_api::{theory_strategy_certificate_evidence, TheoryStrategyKind};
use mpk_cli::policy_evidence::{
    PolicyAxiomCategoryCounts, PolicyAxiomReportEvidence, PolicyCertificateEvidence,
    PolicyCheckerVerdictEvidence, PolicyContractArtifact, PolicyEvidenceReport,
    PolicyEvidenceReproductionCommand, PolicyEvidenceTarget, PolicyHelperArtifactKind,
    PolicyHelperArtifacts, PolicyHelperWarning, PolicyPropertyEvidence, PolicyPropertyEvidenceRef,
    PolicyPropertyEvidenceStatus, PolicySourceArtifact, PolicySourceFileHash,
    PolicyTheoryCertificateEvidence, PolicyTrustedEvidence, POLICY_EVIDENCE_SCHEMA,
};
use serde_json::{json, Value};

const ORDER_POLICY_FUNCTION: &str = "example.com/orderpolicy.ApprovedReserveCents";
const RESERVE_POLICY_FUNCTION: &str = "example.com/payment/reserve.ApprovedReserveCents";
const RESERVE_EVIDENCE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/payment_policies/reserve/evidence_alpha.json"
);
const SOURCE_HASH: &str = "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502";
const SOURCE_FILE_HASH: &str = "4b8fab6e2f2d9e20dc77eee7f1b8813fc423acd858d1dab802259725f1801948";
const CONTRACT_HASH: &str = "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00";
const GIR_HASH: &str = "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950";
const VC_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const CERTIFICATE_HASH: &str = "37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94";
const EXPORT_HASH: &str = "5e3396fad9702c2578204b2cb90c112e9653fdb57908ab455e3f77dd58b2e91e";
const AXIOM_REPORT_HASH: &str = "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5";
const LINARITH_THEORY_CERT_FORMAT: &str = "mpk.linarith.v0";
const LINARITH_THEORY_CERT_HASH: &str =
    "a85d54f8d5c32dba5f414490120847013b7c727a3ce8b6ae2c3a44aae4edd7e1";

#[test]
fn accepted_policy_evidence_snapshot_is_deterministic() {
    let report = accepted_evidence_report();
    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert_eq!(
        first,
        r#"{
  "schema": "mpk.policy.evidence.v0",
  "target": {
    "package_path": "example.com/orderpolicy",
    "function_id": "example.com/orderpolicy.ApprovedReserveCents"
  },
  "strategy_profile": "payment-policy-alpha",
  "checker_profile": "mvp-strict",
  "allowed_axiom_profiles": [
    "zero-axiom"
  ],
  "trusted_evidence": {
    "certificates": [
      {
        "id": "cert:order-policy",
        "module": "ProofOps.OrderPolicy",
        "path": "proofs/policy/order_policy.mpcert",
        "certificate_hash": "37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94",
        "export_hash": "5e3396fad9702c2578204b2cb90c112e9653fdb57908ab455e3f77dd58b2e91e",
        "axiom_report_hash": "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
        "checked_declarations": [
          "ProofOps.OrderPolicy.approved_reserve_nonnegative"
        ]
      }
    ],
    "theory_certificates": [
      {
        "id": "theory:int-linear-001",
        "theory": "signed_int_linear",
        "format": "mpk.linarith.v0",
        "theory_certificate_hash": "a85d54f8d5c32dba5f414490120847013b7c727a3ce8b6ae2c3a44aae4edd7e1",
        "checker_profile": "mvp-strict",
        "checked_obligations": [
          "vc:approved_reserve_nonnegative"
        ]
      }
    ],
    "axiom_report": {
      "axiom_report_hash": "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
      "category_counts": {
        "total_axiom_count": 0,
        "core_axiom_count": 0,
        "builtin_theory_axiom_count": 0,
        "go_semantics_axiom_count": 0,
        "external_axiom_count": 0
      }
    },
    "rust_checker": {
      "verdict": "accepted",
      "command": "cargo run --quiet -p mpk-cli -- check proofs/policy/order_policy.mpcert",
      "certificate_ids": [
        "cert:order-policy"
      ]
    },
    "reference_checker": {
      "verdict": "accepted",
      "command": "go run ./cmd/mpk-checker-ref verify proofs/policy/order_policy.mpcert",
      "certificate_ids": [
        "cert:order-policy"
      ]
    }
  },
  "helper_artifacts": {
    "source": {
      "root": "examples/order_policy",
      "source_hash": "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502",
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
      "contract_hash": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00"
    },
    "gir_hash": "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950",
    "vc_hash": "2222222222222222222222222222222222222222222222222222222222222222",
    "warnings": []
  },
  "properties": [
    {
      "id": "approved_reserve_nonnegative",
      "description": "Approved reserve cents never goes negative.",
      "status": "mpk_verified",
      "evidence": [
        {
          "kind": "checked_declaration",
          "certificate_id": "cert:order-policy",
          "declaration_id": "ProofOps.OrderPolicy.approved_reserve_nonnegative"
        },
        {
          "kind": "checked_theory_certificate",
          "theory_certificate_id": "theory:int-linear-001",
          "obligation_id": "vc:approved_reserve_nonnegative"
        }
      ],
      "notes": []
    }
  ],
  "reproduction_commands": [
    {
      "label": "scan",
      "command": "mpk policy scan examples/order_policy --function example.com/orderpolicy.ApprovedReserveCents --contract examples/order_policy/policy_contract.json --json-out scan.json"
    },
    {
      "label": "check",
      "command": "mpk check proofs/policy/order_policy.mpcert"
    }
  ]
}
"#
    );

    let reparsed = PolicyEvidenceReport::from_json(&first).expect("valid schema parses");
    assert_eq!(reparsed, report);
}

#[test]
fn checked_linarith_evidence_matches_strategy_certificate_hash() {
    let evidence = theory_strategy_certificate_evidence(TheoryStrategyKind::Linarith);

    assert_eq!(evidence.format, LINARITH_THEORY_CERT_FORMAT);
    assert_eq!(evidence.theory_certificate_hash, LINARITH_THEORY_CERT_HASH);
}

#[test]
fn reserve_policy_evidence_fixture_marks_all_properties_verified() {
    let json = std::fs::read_to_string(RESERVE_EVIDENCE_FIXTURE)
        .expect("reserve evidence fixture is readable");
    let report = PolicyEvidenceReport::from_json(&json).expect("reserve evidence fixture parses");

    assert_eq!(report.target.package_path, "example.com/payment/reserve");
    assert_eq!(report.target.function_id, RESERVE_POLICY_FUNCTION);
    assert_eq!(report.strategy_profile, "payment-policy-alpha");
    assert_eq!(report.checker_profile, "mvp-strict");
    assert!(report.trusted_evidence.certificates.is_empty());
    assert_eq!(report.trusted_evidence.theory_certificates.len(), 8);

    assert_eq!(report.properties.len(), 8);
    assert!(report
        .properties
        .iter()
        .all(|property| property.status == PolicyPropertyEvidenceStatus::MpkVerified));

    let theory_by_id = report
        .trusted_evidence
        .theory_certificates
        .iter()
        .map(|theory| (theory.id.as_str(), theory))
        .collect::<std::collections::BTreeMap<_, _>>();

    for index in 1..=6 {
        let id = format!("theory:policy-linarith-{index:04}");
        let theory = theory_by_id
            .get(id.as_str())
            .unwrap_or_else(|| panic!("missing linarith theory certificate {id}"));
        assert_eq!(theory.theory, "linarith");
        assert_eq!(theory.format, LINARITH_THEORY_CERT_FORMAT);
        assert_eq!(theory.checker_profile, "mvp-strict");
        assert_eq!(theory.theory_certificate_hash.len(), 64);
        assert_eq!(theory.checked_obligations.len(), 1);
    }

    for index in 1..=2 {
        let id = format!("theory:policy-bool-tautology-{index:04}");
        let theory = theory_by_id
            .get(id.as_str())
            .unwrap_or_else(|| panic!("missing bool theory certificate {id}"));
        assert_eq!(theory.theory, "bool_tautology");
        assert_eq!(theory.format, "mpk.bool-normalize.v0");
        assert_eq!(theory.checker_profile, "mvp-strict");
        assert_eq!(theory.theory_certificate_hash.len(), 64);
        assert_eq!(theory.checked_obligations.len(), 1);
    }

    for property in &report.properties {
        let [PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id,
            obligation_id,
        }] = property.evidence.as_slice()
        else {
            panic!("reserve property should reference exactly one checked theory certificate");
        };
        assert_eq!(obligation_id, &property.id);
        let theory = theory_by_id
            .get(theory_certificate_id.as_str())
            .unwrap_or_else(|| panic!("missing theory certificate {theory_certificate_id}"));
        assert_eq!(
            theory.checked_obligations[0].as_str(),
            obligation_id.as_str()
        );
    }

    assert!(!json.contains("\"ai_analysis\""));
    assert!(!json.contains("solver"));
}

#[test]
fn helper_only_policy_evidence_serializes_and_reparses() {
    let report = helper_only_evidence_report();
    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert!(first.contains("\"status\": \"helper_only\""));
    assert!(first.contains("\"artifact\": \"contract\""));
    let reparsed = PolicyEvidenceReport::from_json(&first).expect("valid schema parses");
    assert_eq!(reparsed, report);
}

#[test]
fn unsupported_policy_evidence_serializes_and_reparses() {
    let report = unsupported_evidence_report();
    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert!(first.contains("\"status\": \"unsupported\""));
    assert!(first.contains("\"kind\": \"unsupported_feature\""));
    let reparsed = PolicyEvidenceReport::from_json(&first).expect("valid schema parses");
    assert_eq!(reparsed, report);
}

#[test]
fn rejected_checker_evidence_serializes_and_reparses() {
    let mut report = helper_only_evidence_report();
    report.trusted_evidence.rust_checker = Some(PolicyCheckerVerdictEvidence::rejected(
        "cargo run --quiet -p mpk-cli -- check proofs/policy/order_policy.mpcert",
        vec!["cert:order-policy".to_owned()],
    ));
    report.properties[0].status = PolicyPropertyEvidenceStatus::ProofPending;
    report.properties[0]
        .notes
        .push("Rust checker rejected the current candidate certificate.".to_owned());

    let first = report.to_deterministic_json().expect("serializes");
    let second = report.to_deterministic_json().expect("serializes");

    assert_eq!(first, second);
    assert!(first.contains("\"verdict\": \"rejected\""));
    assert!(first.contains("\"status\": \"proof_pending\""));
    let reparsed = PolicyEvidenceReport::from_json(&first).expect("valid schema parses");
    assert_eq!(reparsed, report);
}

#[test]
fn evidence_sections_keep_trust_boundary_hashes_separate() {
    let report = accepted_evidence_report();
    let json = report.to_deterministic_json().expect("serializes");
    let value = serde_json::from_str::<Value>(&json).expect("valid JSON");

    let mut without_helper = value.clone();
    without_helper
        .as_object_mut()
        .expect("top-level object")
        .remove("helper_artifacts");
    assert_no_key(&without_helper, "gir_hash");
    assert_no_key(&without_helper, "vc_hash");

    let mut without_trusted = value.clone();
    without_trusted
        .as_object_mut()
        .expect("top-level object")
        .remove("trusted_evidence");
    assert_no_key(&without_trusted, "certificate_hash");
    assert_no_key(&without_trusted, "export_hash");
    assert_no_key(&without_trusted, "axiom_report_hash");
    assert_no_key(&without_trusted, "rust_checker");
    assert_no_key(&without_trusted, "reference_checker");
}

#[test]
fn evidence_records_strategy_checker_and_axiom_profiles_separately() {
    let report = accepted_evidence_report();
    assert_eq!(report.strategy_profile, "payment-policy-alpha");
    assert_eq!(report.checker_profile, "mvp-strict");
    assert_eq!(report.allowed_axiom_profiles, vec!["zero-axiom".to_owned()]);
    assert_ne!(report.strategy_profile, report.checker_profile);
    assert!(!report
        .allowed_axiom_profiles
        .iter()
        .any(|profile| profile == &report.strategy_profile || profile == &report.checker_profile));

    let json = report.to_deterministic_json().expect("serializes");
    let value = serde_json::from_str::<Value>(&json).expect("valid JSON");
    assert_eq!(value["strategy_profile"], json!("payment-policy-alpha"));
    assert_eq!(value["checker_profile"], json!("mvp-strict"));
    assert_eq!(value["allowed_axiom_profiles"], json!(["zero-axiom"]));

    let reparsed = PolicyEvidenceReport::from_json(&json).expect("valid schema parses");
    assert_eq!(reparsed.strategy_profile, "payment-policy-alpha");
    assert_eq!(reparsed.checker_profile, "mvp-strict");
    assert_eq!(
        reparsed.allowed_axiom_profiles,
        vec!["zero-axiom".to_owned()]
    );
}

#[test]
fn unknown_top_level_field_rejects() {
    let mut value =
        serde_json::from_str::<Value>(&accepted_evidence_report().to_deterministic_json().unwrap())
            .expect("valid JSON");
    value
        .as_object_mut()
        .expect("top-level object")
        .insert("proof_acceptance".to_owned(), json!("accepted"));

    let error = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect_err("unknown top-level field rejects");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn unknown_evidence_ref_field_rejects() {
    let mut value =
        serde_json::from_str::<Value>(&accepted_evidence_report().to_deterministic_json().unwrap())
            .expect("valid JSON");
    value["properties"][0]["evidence"][0]["extra"] = json!("not allowed");

    let error = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect_err("unknown evidence ref field rejects");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn unknown_property_status_rejects() {
    let mut value =
        serde_json::from_str::<Value>(&accepted_evidence_report().to_deterministic_json().unwrap())
            .expect("valid JSON");
    value["properties"][0]["status"] = json!("source_claimed_verified");

    let error = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect_err("unknown property status rejects");
    assert!(error.to_string().contains("unknown variant"));
}

#[test]
fn mpk_verified_property_must_reference_checked_declaration_or_theory_certificate() {
    let mut report = helper_only_evidence_report();
    report.properties[0].status = PolicyPropertyEvidenceStatus::MpkVerified;

    let error = report
        .to_deterministic_json()
        .expect_err("mpk_verified without trusted evidence rejects");
    assert!(error
        .to_string()
        .contains("mpk_verified without checked declaration"));
}

#[test]
fn schema_mismatch_rejects_for_policy_evidence_report() {
    let mut value =
        serde_json::from_str::<Value>(&accepted_evidence_report().to_deterministic_json().unwrap())
            .expect("valid JSON");
    value["schema"] = json!("mpk.policy.evidence.v1");

    let error = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect_err("schema mismatch rejects");
    assert!(error
        .to_string()
        .contains("policy evidence schema = \"mpk.policy.evidence.v1\""));
}

#[test]
fn policy_evidence_schema_constant_is_stable() {
    assert_eq!(POLICY_EVIDENCE_SCHEMA, "mpk.policy.evidence.v0");
}

fn accepted_evidence_report() -> PolicyEvidenceReport {
    let mut trusted = PolicyTrustedEvidence::empty();
    trusted.certificates.push(PolicyCertificateEvidence::new(
        "cert:order-policy",
        "ProofOps.OrderPolicy",
        "proofs/policy/order_policy.mpcert",
        CERTIFICATE_HASH,
        EXPORT_HASH,
        AXIOM_REPORT_HASH,
        vec!["ProofOps.OrderPolicy.approved_reserve_nonnegative".to_owned()],
    ));
    trusted
        .theory_certificates
        .push(PolicyTheoryCertificateEvidence::new(
            "theory:int-linear-001",
            "signed_int_linear",
            LINARITH_THEORY_CERT_FORMAT,
            LINARITH_THEORY_CERT_HASH,
            "mvp-strict",
            vec!["vc:approved_reserve_nonnegative".to_owned()],
        ));
    trusted.axiom_report = Some(PolicyAxiomReportEvidence::new(
        AXIOM_REPORT_HASH,
        PolicyAxiomCategoryCounts {
            total_axiom_count: 0,
            core_axiom_count: 0,
            builtin_theory_axiom_count: 0,
            go_semantics_axiom_count: 0,
            external_axiom_count: 0,
        },
    ));
    trusted.rust_checker = Some(PolicyCheckerVerdictEvidence::accepted(
        "cargo run --quiet -p mpk-cli -- check proofs/policy/order_policy.mpcert",
        vec!["cert:order-policy".to_owned()],
    ));
    trusted.reference_checker = Some(PolicyCheckerVerdictEvidence::accepted(
        "go run ./cmd/mpk-checker-ref verify proofs/policy/order_policy.mpcert",
        vec!["cert:order-policy".to_owned()],
    ));

    let mut report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        trusted,
        helper_artifacts(),
    );
    let mut property = PolicyPropertyEvidence::new(
        "approved_reserve_nonnegative",
        "Approved reserve cents never goes negative.",
        PolicyPropertyEvidenceStatus::MpkVerified,
    );
    property
        .evidence
        .push(PolicyPropertyEvidenceRef::CheckedDeclaration {
            certificate_id: "cert:order-policy".to_owned(),
            declaration_id: "ProofOps.OrderPolicy.approved_reserve_nonnegative".to_owned(),
        });
    property
        .evidence
        .push(PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id: "theory:int-linear-001".to_owned(),
            obligation_id: "vc:approved_reserve_nonnegative".to_owned(),
        });
    report.properties.push(property);
    report
        .reproduction_commands
        .push(PolicyEvidenceReproductionCommand::new(
            "scan",
            "mpk policy scan examples/order_policy --function example.com/orderpolicy.ApprovedReserveCents --contract examples/order_policy/policy_contract.json --json-out scan.json",
        ));
    report
        .reproduction_commands
        .push(PolicyEvidenceReproductionCommand::new(
            "check",
            "mpk check proofs/policy/order_policy.mpcert",
        ));
    report
}

fn helper_only_evidence_report() -> PolicyEvidenceReport {
    let mut report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        PolicyTrustedEvidence::empty(),
        helper_artifacts(),
    );
    let mut property = PolicyPropertyEvidence::new(
        "approved_reserve_nonnegative",
        "Approved reserve cents never goes negative.",
        PolicyPropertyEvidenceStatus::HelperOnly,
    );
    property
        .evidence
        .push(PolicyPropertyEvidenceRef::HelperArtifact {
            artifact: PolicyHelperArtifactKind::Contract,
            summary: "Contract has an ensures clause, but no checked certificate is available."
                .to_owned(),
        });
    property
        .notes
        .push("Use proof generation before treating this as verified.".to_owned());
    report.properties.push(property);
    report
}

fn unsupported_evidence_report() -> PolicyEvidenceReport {
    let mut helper_artifacts = helper_artifacts();
    helper_artifacts.warnings.push(PolicyHelperWarning::new(
        "GO2GIR_REJECTED_MAPS",
        "go2gir rejected map operations in the policy function.",
        PolicyHelperArtifactKind::GoSource,
    ));
    let mut report = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new("example.com/orderpolicy", ORDER_POLICY_FUNCTION),
        "payment-policy-alpha",
        "mvp-strict",
        vec!["zero-axiom".to_owned()],
        PolicyTrustedEvidence::empty(),
        helper_artifacts,
    );
    let mut property = PolicyPropertyEvidence::new(
        "approved_reserve_nonnegative",
        "Approved reserve cents never goes negative.",
        PolicyPropertyEvidenceStatus::Unsupported,
    );
    property
        .evidence
        .push(PolicyPropertyEvidenceRef::UnsupportedFeature {
            code: "GO2GIR_REJECTED_MAPS".to_owned(),
            message: "Map operations are outside Go subset v0.".to_owned(),
        });
    report.properties.push(property);
    report
}

fn helper_artifacts() -> PolicyHelperArtifacts {
    let mut artifacts = PolicyHelperArtifacts::new(
        PolicySourceArtifact::new(
            "examples/order_policy",
            SOURCE_HASH,
            vec![PolicySourceFileHash::new(
                "examples/order_policy/policy.go",
                SOURCE_FILE_HASH,
            )],
        ),
        PolicyContractArtifact::new(
            "examples/order_policy/policy_contract.json",
            "mpk.go.contract.v0",
            CONTRACT_HASH,
        ),
    );
    artifacts.gir_hash = Some(GIR_HASH.to_owned());
    artifacts.vc_hash = Some(VC_HASH.to_owned());
    artifacts
}

fn assert_no_key(value: &Value, key: &str) {
    match value {
        Value::Object(object) => {
            assert!(!object.contains_key(key), "unexpected key {key} in {value}");
            for nested in object.values() {
                assert_no_key(nested, key);
            }
        }
        Value::Array(array) => {
            for nested in array {
                assert_no_key(nested, key);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
