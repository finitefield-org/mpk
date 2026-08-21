use mpk_cli::policy_evidence::PolicyEvidenceReport;
use mpk_cli::policy_report::{render_policy_evidence_markdown, render_policy_evidence_v1_markdown};
use mpk_cli::policy_schema::{
    expected_reproduction_recipes, import_policy_evidence_v1_json, import_policy_scan_v1_json,
    render_posix_argv, PolicyAxiomReportV1, PolicyCheckedDeclaration, PolicyEvidenceLinkageContext,
    PolicyEvidenceV1, PolicyExpectedCertificateV1, PolicyExpectedMemberV1,
    PolicyExpectedPropertyV1, PolicyHelperArtifact, PolicyIssue, PolicyScanLinkageContext,
    PolicySelection, PolicySemanticParameters, PolicyTheoryCertificateEvidenceV1,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, FrontendIdentity, ReleaseRegistryIdentity,
    StrictJsonLimits, ToolchainIdentity,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[test]
fn accepted_evidence_json_renders_markdown_snapshot() {
    let report = parse_report(accepted_evidence_json());
    let first = render_policy_evidence_markdown(&report);
    let second = render_policy_evidence_markdown(&report);

    assert_eq!(first, second);
    assert_eq!(
        first,
        r#"# MPK Policy Evidence Report

## Target Function

- Package: `example.com/orderpolicy`
- Function: `example.com/orderpolicy.ApprovedReserveCents`
- Strategy profile: `payment-policy-alpha`
- Checker profile: `mvp-strict`
- Allowed axiom profiles: `zero-axiom`

## Verification Summary

- Verified: `1`
- Proof pending: `0`
- Helper only: `0`
- Unsupported: `0`

## Verified Properties

- `approved_reserve_nonnegative`: Approved reserve cents never goes negative. (status: `mpk_verified`)
  - Evidence:
    - checked declaration `ProofOps.OrderPolicy.approved_reserve_nonnegative` in certificate `cert:order-policy`
    - checked theory certificate `theory:int-linear-001` for obligation `vc:approved_reserve_nonnegative`

## Proof-Pending Properties

- None recorded.

## Helper-Only Properties

- None recorded.

## Unsupported Properties

- None recorded.

## Required Preconditions

- Not recorded in `mpk.policy.evidence.v0`; consult the source evidence JSON and scan JSON for contract preconditions.

## Hashes

### Trusted Evidence Hashes

- Certificate `cert:order-policy`
  - Module: `ProofOps.OrderPolicy`
  - Path: `proofs/policy/order_policy.mpcert`
  - Certificate hash: `37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94`
  - Export hash: `5e3396fad9702c2578204b2cb90c112e9653fdb57908ab455e3f77dd58b2e91e`
  - Axiom report hash: `0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5`
- Theory certificate `theory:int-linear-001`
  - Theory: `signed_int_linear`
  - Format: `mpk.linarith.v0`
  - Theory certificate hash: `a85d54f8d5c32dba5f414490120847013b7c727a3ce8b6ae2c3a44aae4edd7e1`
  - Checker profile: `mvp-strict`
- Axiom report
  - Axiom report hash: `0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5`
  - Total axiom count: `0`
  - Core axiom count: `0`
  - Builtin theory axiom count: `0`
  - Go semantics axiom count: `0`
  - External axiom count: `0`

### Helper Artifact Hashes

- Source root: `examples/order_policy`
- Source hash: `5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502`
- Source file `examples/order_policy/policy.go`: `4b8fab6e2f2d9e20dc77eee7f1b8813fc423acd858d1dab802259725f1801948`
- Contract path: `examples/order_policy/policy_contract.json`
- Contract schema: `mpk.go.contract.v0`
- Contract hash: `fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00`
- GIR hash: `83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950`
- VC hash: `2222222222222222222222222222222222222222222222222222222222222222`

## Checker Verdicts

- Rust fast-kernel: `accepted`
  - Command: `cargo run --quiet -p mpk-cli -- check proofs/policy/order_policy.mpcert`
  - Certificate ids: `cert:order-policy`
- Independent reference checker: `accepted`
  - Command: `go run ./cmd/mpk-checker-ref verify proofs/policy/order_policy.mpcert`
  - Certificate ids: `cert:order-policy`

## Helper Artifacts

- Go source text is helper evidence only.
- Contract JSON is helper evidence only.
- GIR is helper evidence only.
- VC JSON is helper evidence only.
- CI status is helper evidence only.
- Helper warnings: none recorded.

## Reproduction Commands

- scan

```sh
mpk policy scan examples/order_policy --function example.com/orderpolicy.ApprovedReserveCents --contract examples/order_policy/policy_contract.json --json-out scan.json
```

- check

```sh
mpk check proofs/policy/order_policy.mpcert
```

## Trust-Boundary Notes

- The `mpk.policy.evidence.v0` JSON is the source of truth; this Markdown is a derived view.
- A property is `mpk_verified` only when it references checked declaration evidence or checked theory-certificate evidence.
- GIR, VC JSON, source text, contract JSON, and CI status are helper artifacts and are not proof evidence.
"#
    );
}

#[test]
fn helper_only_evidence_json_renders_clear_non_verified_status() {
    let report = parse_report(helper_only_evidence_json());
    let markdown = render_policy_evidence_markdown(&report);

    assert!(markdown.contains("- Verified: `0`\n"));
    assert!(markdown.contains("- Helper only: `1`\n"));
    assert!(
        markdown
            .contains("- `approved_reserve_nonnegative`: Approved reserve cents never goes negative. (status: `helper_only`)")
    );
    assert!(markdown.contains("## Helper Artifacts\n\n- Go source text is helper evidence only."));
    assert!(markdown.contains("helper artifact `contract`: Contract has an ensures clause"));
    assert!(markdown.contains("- Rust fast-kernel: not recorded."));
}

#[test]
fn rejected_evidence_json_renders_clear_proof_pending_status() {
    let report = parse_report(rejected_evidence_json());
    let markdown = render_policy_evidence_markdown(&report);

    assert!(markdown.contains("- Verified: `0`\n"));
    assert!(markdown.contains("- Proof pending: `1`\n"));
    assert!(
        markdown.contains(
            "- `approved_reserve_nonnegative`: Approved reserve cents never goes negative. (status: `proof_pending`)"
        )
    );
    assert!(markdown.contains("- Rust fast-kernel: `rejected`"));
    assert!(markdown.contains("Rust checker rejected the current candidate certificate."));
}

#[test]
fn unsupported_evidence_json_renders_unsupported_section_and_warning() {
    let report = parse_report(unsupported_evidence_json());
    let markdown = render_policy_evidence_markdown(&report);

    assert!(markdown.contains("- Unsupported: `1`\n"));
    assert!(
        markdown.contains(
            "- `approved_reserve_nonnegative`: Approved reserve cents never goes negative. (status: `unsupported`)"
        )
    );
    assert!(markdown.contains(
        "unsupported feature `GO2GIR_REJECTED_MAPS`: Map operations are outside Go subset v0."
    ));
    assert!(
        markdown
            .contains("- `GO2GIR_REJECTED_MAPS` (go_source): go2gir rejected map operations in the policy function.")
    );
}

#[test]
fn renderer_uses_evidence_json_as_source_of_truth() {
    let mut value = serde_json::from_str::<Value>(accepted_evidence_json()).expect("valid JSON");
    value["target"]["function_id"] = json!("example.com/orderpolicy.OtherPolicy");
    value["properties"][0]["description"] = json!("Edited JSON description.");
    let report = PolicyEvidenceReport::from_json(&serde_json::to_string(&value).unwrap())
        .expect("edited JSON is still valid");
    let markdown = render_policy_evidence_markdown(&report);

    assert!(markdown.contains("- Function: `example.com/orderpolicy.OtherPolicy`"));
    assert!(markdown.contains("Edited JSON description."));
    assert!(
        markdown.contains(
            "The `mpk.policy.evidence.v0` JSON is the source of truth; this Markdown is a derived view."
        )
    );
}

#[test]
fn required_preconditions_section_is_present_without_promoting_markdown_to_source_of_truth() {
    let report = parse_report(accepted_evidence_json());
    let markdown = render_policy_evidence_markdown(&report);

    assert!(markdown.contains("## Required Preconditions\n\n"));
    assert!(
        markdown.contains(
            "Not recorded in `mpk.policy.evidence.v0`; consult the source evidence JSON and scan JSON for contract preconditions."
        )
    );
}

#[test]
fn validated_v1_evidence_renders_all_frozen_sections_deterministically() {
    let evidence = validated_rust_v1_evidence();
    let first = render_policy_evidence_v1_markdown(&evidence).expect("v1 report renders");
    let second = render_policy_evidence_v1_markdown(&evidence).expect("repeat render succeeds");
    assert_eq!(first, second);
    assert!(first.ends_with('\n'));
    assert!(!first.ends_with("\n\n"));
    assert!(!first.contains('\r'));
    assert!(first.lines().all(|line| !line.ends_with(' ')));
    assert!(!first.contains("/Users/"));
    assert!(!first.contains("checker command"));
    assert!(first.contains("- Untrusted helper `source:src/lib.rs`"));
    assert!(first.contains("Frontend source manifest SHA-256"));
    assert!(first.contains("Certificate source manifest SHA-256"));
    assert!(first.contains("Dependency `VC.Function."));
    assert!(first.contains("Only checker-accepted canonical certificate and theory-certificate bytes are trusted evidence."));
    assert!(first.contains("Policy JSON, source text, contracts, VIR, VC, AI analysis, CI status, and this Markdown report are not proof evidence."));

    let headings = first
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect::<Vec<_>>();
    assert_eq!(
        headings,
        [
            "Target and Profiles",
            "Source and Release Identities",
            "Verification Summary",
            "Properties",
            "Trusted Evidence",
            "Helper Artifacts",
            "Reproduction Recipes",
            "Trust-Boundary Notes",
        ]
    );
}

#[test]
fn rejected_v1_candidate_remains_reportable_but_not_verified() {
    let evidence = validated_rust_v1_evidence_with(|document| {
        document.trusted_evidence.checker_verdicts[0].verdict = "rejected".to_owned();
        document.properties[0].status = "proof_pending".to_owned();
        document.properties[0].members[0].status = "proof_pending".to_owned();
        document.properties[0].members[0].evidence = vec![
            mpk_cli::policy_schema::PolicyEvidenceReferenceV1::HelperArtifact {
                artifact_id: "vc".to_owned(),
            },
        ];
        document.reproduction_recipes = expected_reproduction_recipes(document);
    });
    let markdown = render_policy_evidence_v1_markdown(&evidence).expect("report renders");

    assert!(markdown.contains("- mpk_verified: `0`\n"));
    assert!(markdown.contains("- proof_pending: `1`\n"));
    assert!(markdown.contains("- Verdict: `rejected`\n"));
}

#[test]
fn policy_v1_posix_display_executes_every_render_vector() {
    let vectors = load_value("develop/specs/vectors/policy-recipes-v1.json");
    assert_eq!(
        vectors["owner_test"],
        "crates/mpk-cli/tests/policy_recipes_v1.rs"
    );
    let mut ids = std::collections::BTreeSet::new();
    for case in vectors["render_cases"].as_array().expect("render cases") {
        let id = case["id"].as_str().expect("render ID");
        assert!(ids.insert(id));
        let argv = serde_json::from_value::<Vec<String>>(case["argv"].clone()).unwrap();
        assert_eq!(render_posix_argv(&argv), case["expected_posix"], "{id}");
    }
    assert_eq!(ids.len(), vectors["render_cases"].as_array().unwrap().len());
}

#[test]
fn policy_v1_recipe_builder_executes_every_recipe_vector() {
    let recipes = load_value("develop/specs/vectors/policy-recipes-v1.json");
    let evidence = load_value("develop/specs/vectors/policy-evidence-v1.json");
    let mut ids = std::collections::BTreeSet::new();
    for case in recipes["recipe_cases"].as_array().expect("recipe cases") {
        let id = case["id"].as_str().unwrap();
        assert!(ids.insert(id));
        let fixture_id = match case["invocation"].as_str().unwrap() {
            "invocation.go_verify" => "evidence.go_identity_pending",
            "invocation.rust_verify_fixture_update" => "evidence.rust_call_pair_verified",
            other => panic!("unknown recipe invocation {other}"),
        };
        let mut document: PolicyEvidenceV1 =
            serde_json::from_value(find_id(&evidence["fixtures"], fixture_id)["input"].clone())
                .unwrap();
        let invocation = find_id(
            &recipes["invocations"],
            case["invocation"].as_str().unwrap(),
        );
        document.verification_options.strict = invocation["parsed"]["strict"].as_bool().unwrap();
        document.verification_options.update_fixtures =
            invocation["parsed"]["update_fixtures"].as_bool().unwrap();
        assert_eq!(
            serde_json::to_value(expected_reproduction_recipes(&document)).unwrap(),
            case["expect"]["recipes"],
            "{id}"
        );
    }
    assert_eq!(ids.len(), recipes["recipe_cases"].as_array().unwrap().len());
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportScanContext {
    id: String,
    frontend_status: String,
    frontend_phase: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: PolicySemanticParameters,
    selection: PolicySelection,
    release_registry: ReleaseRegistryIdentity,
    frontend: FrontendIdentity,
    toolchain: ToolchainIdentity,
    limit_profile: String,
    frontend_source_manifest_hash: String,
    input_set_hash: String,
    source_map_hash: String,
    source_ir_schema: String,
    source_ir_hash: String,
    helper_artifacts: Vec<PolicyHelperArtifact>,
    rejected_features: Vec<PolicyIssue>,
    diagnostics: Vec<PolicyIssue>,
}

fn validated_rust_v1_evidence() -> mpk_cli::policy_schema::ValidatedPolicyEvidenceV1 {
    validated_rust_v1_evidence_with(|_| {})
}

fn validated_rust_v1_evidence_with(
    mutate: impl FnOnce(&mut PolicyEvidenceV1),
) -> mpk_cli::policy_schema::ValidatedPolicyEvidenceV1 {
    let scan_vectors = load_value("develop/specs/vectors/policy-scan-v1.json");
    let evidence_vectors = load_value("develop/specs/vectors/policy-evidence-v1.json");
    let scan_fixture = find_id(&scan_vectors["fixtures"], "scan.rust_call_pair_ready");
    let scan_context_value = find_id(
        &scan_vectors["linkage_contexts"],
        scan_fixture["linkage_context"].as_str().unwrap(),
    );
    let scan_context: ReportScanContext =
        serde_json::from_value(scan_context_value.clone()).unwrap();
    assert_eq!(scan_context.id, "context.rust_call_pair_ready");
    let scan_linkage = PolicyScanLinkageContext {
        frontend_status: scan_context.frontend_status,
        frontend_phase: scan_context.frontend_phase,
        source_language: scan_context.source_language,
        semantic_profile: scan_context.semantic_profile,
        semantic_parameters: scan_context.semantic_parameters,
        selection: scan_context.selection,
        release_registry: scan_context.release_registry,
        frontend: scan_context.frontend,
        toolchain: scan_context.toolchain,
        rejected_features: scan_context.rejected_features,
        diagnostics: scan_context.diagnostics,
        limit_profile: Some(scan_context.limit_profile),
        frontend_source_manifest_hash: Some(scan_context.frontend_source_manifest_hash),
        input_set_hash: Some(scan_context.input_set_hash),
        source_map_hash: Some(scan_context.source_map_hash),
        source_ir_schema: Some(scan_context.source_ir_schema),
        source_ir_hash: Some(scan_context.source_ir_hash),
        helper_artifacts: Some(scan_context.helper_artifacts),
    };
    let scan_bytes = canonical_transport(&scan_fixture["input"]);
    let scan = import_policy_scan_v1_json(&scan_bytes, &scan_linkage).unwrap();

    let evidence_fixture = find_id(
        &evidence_vectors["fixtures"],
        "evidence.rust_call_pair_verified",
    );
    let mut document: PolicyEvidenceV1 =
        serde_json::from_value(evidence_fixture["input"].clone()).unwrap();
    mutate(&mut document);
    let context = find_id(
        &evidence_vectors["linkage_contexts"],
        evidence_fixture["linkage_context"].as_str().unwrap(),
    );
    let declarations =
        serde_json::from_value::<Vec<PolicyCheckedDeclaration>>(context["declarations"].clone())
            .unwrap();
    let expected_members = declarations
        .iter()
        .flat_map(|declaration| {
            declaration.member_ids.iter().map(|member_id| {
                let mut parts = member_id.rsplitn(3, '#');
                let _ordinal = parts.next().unwrap();
                let kind = parts.next().unwrap();
                PolicyExpectedMemberV1 {
                    member_id: member_id.clone(),
                    function_id: declaration.function_id.clone(),
                    kind: kind.to_owned(),
                    group_id: declaration.group_id.clone(),
                    declaration_name: declaration.name.clone(),
                    declaration_hash: declaration.declaration_hash.clone(),
                }
            })
        })
        .collect();
    let expected_certificate = PolicyExpectedCertificateV1 {
        module: document.trusted_evidence.certificates[0].module.clone(),
        certificate_hash: context["accepted_certificate_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
        export_hash: context["accepted_export_hash"].as_str().unwrap().to_owned(),
        axiom_report_hash: context["accepted_axiom_report_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
    };
    let expected_theory_certificates: Vec<PolicyTheoryCertificateEvidenceV1> =
        document.trusted_evidence.theory_certificates.clone();
    let expected_axiom_report: PolicyAxiomReportV1 = document.trusted_evidence.axiom_report.clone();
    let expected_checker_verdicts = document.trusted_evidence.checker_verdicts.clone();
    let expected_properties = document
        .properties
        .iter()
        .map(|property| PolicyExpectedPropertyV1 {
            id: property.id.clone(),
            description: property.description.clone(),
            member_ids: property
                .members
                .iter()
                .map(|member| member.member_id.clone())
                .collect(),
            notes: property.notes.clone(),
        })
        .collect();
    let linkage = PolicyEvidenceLinkageContext {
        scan: &scan,
        certificate_source_manifest_hash: context["certificate_source_manifest_hash"]
            .as_str()
            .unwrap()
            .to_owned(),
        source_vc_schema: context["source_vc_schema"].as_str().unwrap().to_owned(),
        vc_hash: context["vc_hash"].as_str().unwrap().to_owned(),
        verification_limit_profile: context["verification_limit_profile"]
            .as_str()
            .unwrap()
            .to_owned(),
        expected_members,
        expected_declarations: declarations,
        expected_certificate: Some(expected_certificate),
        expected_theory_certificates,
        expected_axiom_report,
        expected_checker_verdicts,
        expected_properties,
        expected_unsupported_codes: Vec::new(),
        expected_optional_helpers: Vec::new(),
    };
    let evidence_bytes = canonical_transport(&serde_json::to_value(document).unwrap());
    import_policy_evidence_v1_json(&evidence_bytes, &linkage).unwrap()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let serialized = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576),
    )
    .unwrap();
    let mut bytes = canonical_json_bytes(&strict).unwrap();
    bytes.push(b'\n');
    bytes
}

fn find_id<'a>(values: &'a Value, id: &str) -> &'a Value {
    values
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["id"] == id)
        .unwrap_or_else(|| panic!("missing vector ID {id}"))
}

fn load_value(relative: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn parse_report(json: &str) -> PolicyEvidenceReport {
    PolicyEvidenceReport::from_json(json).expect("evidence JSON fixture parses")
}

fn accepted_evidence_json() -> &'static str {
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
}

fn helper_only_evidence_json() -> &'static str {
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
    "certificates": [],
    "theory_certificates": [],
    "axiom_report": null,
    "rust_checker": null,
    "reference_checker": null
  },
  "helper_artifacts": {
    "source": {
      "root": "examples/order_policy",
      "source_hash": "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502",
      "files": []
    },
    "contract": {
      "path": "examples/order_policy/policy_contract.json",
      "schema": "mpk.go.contract.v0",
      "contract_hash": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00"
    },
    "gir_hash": "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950",
    "vc_hash": null,
    "warnings": []
  },
  "properties": [
    {
      "id": "approved_reserve_nonnegative",
      "description": "Approved reserve cents never goes negative.",
      "status": "helper_only",
      "evidence": [
        {
          "kind": "helper_artifact",
          "artifact": "contract",
          "summary": "Contract has an ensures clause, but no checked certificate is available."
        }
      ],
      "notes": [
        "Use proof generation before treating this as verified."
      ]
    }
  ],
  "reproduction_commands": []
}
"#
}

fn rejected_evidence_json() -> &'static str {
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
    "certificates": [],
    "theory_certificates": [],
    "axiom_report": null,
    "rust_checker": {
      "verdict": "rejected",
      "command": "cargo run --quiet -p mpk-cli -- check proofs/policy/order_policy.mpcert",
      "certificate_ids": [
        "cert:order-policy"
      ]
    },
    "reference_checker": null
  },
  "helper_artifacts": {
    "source": {
      "root": "examples/order_policy",
      "source_hash": "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502",
      "files": []
    },
    "contract": {
      "path": "examples/order_policy/policy_contract.json",
      "schema": "mpk.go.contract.v0",
      "contract_hash": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00"
    },
    "gir_hash": "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950",
    "vc_hash": null,
    "warnings": []
  },
  "properties": [
    {
      "id": "approved_reserve_nonnegative",
      "description": "Approved reserve cents never goes negative.",
      "status": "proof_pending",
      "evidence": [
        {
          "kind": "helper_artifact",
          "artifact": "vc",
          "summary": "VC was generated, but no checked proof was accepted."
        }
      ],
      "notes": [
        "Rust checker rejected the current candidate certificate."
      ]
    }
  ],
  "reproduction_commands": []
}
"#
}

fn unsupported_evidence_json() -> &'static str {
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
    "certificates": [],
    "theory_certificates": [],
    "axiom_report": null,
    "rust_checker": null,
    "reference_checker": null
  },
  "helper_artifacts": {
    "source": {
      "root": "examples/order_policy",
      "source_hash": "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502",
      "files": []
    },
    "contract": {
      "path": "examples/order_policy/policy_contract.json",
      "schema": "mpk.go.contract.v0",
      "contract_hash": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00"
    },
    "gir_hash": null,
    "vc_hash": null,
    "warnings": [
      {
        "code": "GO2GIR_REJECTED_MAPS",
        "message": "go2gir rejected map operations in the policy function.",
        "artifact": "go_source"
      }
    ]
  },
  "properties": [
    {
      "id": "approved_reserve_nonnegative",
      "description": "Approved reserve cents never goes negative.",
      "status": "unsupported",
      "evidence": [
        {
          "kind": "unsupported_feature",
          "code": "GO2GIR_REJECTED_MAPS",
          "message": "Map operations are outside Go subset v0."
        }
      ],
      "notes": []
    }
  ],
  "reproduction_commands": []
}
"#
}
