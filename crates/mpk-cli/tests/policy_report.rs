use mpk_cli::policy_evidence::PolicyEvidenceReport;
use mpk_cli::policy_report::render_policy_evidence_markdown;
use serde_json::{json, Value};

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
  - Theory certificate hash: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
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
        "theory_certificate_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
