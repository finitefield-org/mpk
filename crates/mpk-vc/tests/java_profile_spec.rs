//! Specification-freeze checks, not a Java frontend or Java runtime validator.
//! Source, compiler-adapter, VIR/VC and isolation cases gain executable owners
//! in the later JAVA-03 tasks recorded by the traceability ledger.

use mpk_vc::semantic_profile_registry::{
    validate_semantic_registry_limit, CompiledSemanticProfile, SemanticRegistryLimit,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, sha256_raw_file_bytes,
    HashDomain, StrictJsonLimits, StrictJsonValue,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const PROFILE: &[u8] = include_bytes!("../../../develop/specs/vectors/java-profile-v0.json");
const REGISTRY: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v3.json");
const PREDECESSOR: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v2.json");
const ACTIVE: &[u8] = include_bytes!("../../../release/bundles/semantic-profile-registry.json");
const SPEC: &str = include_str!("../../../develop/specs/JAVA_PROFILE_V0.md");
const ENTRY_HASH: &str = "0d80d13f97c45557fa9978eccc2545ffdb3fc1b93a26856b365a9be200470301";
const REGISTRY_HASH: &str = "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557";
const FIELDS: [&str; 9] = [
    "ai",
    "evidence",
    "frontend",
    "manifest",
    "policy",
    "release",
    "source_map",
    "vc",
    "vir",
];
const LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(16 * 1024 * 1024, 2_000_000, 128, 4 * 1024 * 1024);

#[test]
fn java_registry_hashes_preserve_every_predecessor_entry() {
    let vectors = load(REGISTRY);
    exact_keys(
        &vectors,
        &[
            "schema",
            "owner_test",
            "mechanism_spec",
            "profile_spec",
            "predecessor",
            "java_entry",
            "registry",
            "hash_cases",
            "append_only_cases",
            "activation_cases",
            "mutation_cases",
        ],
    );
    assert_eq!(
        vectors["schema"],
        "mpk.semantic_profile.registry.conformance.v3"
    );
    assert_eq!(
        vectors["owner_test"],
        "crates/mpk-vc/tests/java_profile_spec.rs"
    );
    let predecessor = load(PREDECESSOR);
    assert_eq!(
        canonical(&vectors["predecessor"]),
        canonical(&predecessor["registry"])
    );
    let root = &vectors["registry"];
    exact_keys(
        root,
        &["schema", "id", "revision", "profiles", "registry_sha256"],
    );
    assert_eq!(root["schema"], "mpk.semantic_profile.registry.v1");
    assert_eq!(root["id"], root["schema"]);
    assert_eq!(root["revision"], 3);
    assert_eq!(root["registry_sha256"], REGISTRY_HASH);
    let entries = array(&root["profiles"]);
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries
            .iter()
            .map(|entry| text(&entry["source_language"]))
            .collect::<Vec<_>>(),
        ["csharp", "go", "java", "rust"]
    );
    for (old, new) in [(0, 0), (1, 1), (2, 3)] {
        assert_eq!(
            canonical(&predecessor["registry"]["profiles"][old]),
            canonical(&entries[new])
        );
    }
    assert_eq!(&entries[2], &vectors["java_entry"]);
    for entry in entries {
        exact_keys(
            entry,
            &[
                "schema",
                "source_language",
                "semantic_profile",
                "semantic_parameters_schema",
                "selection_schema",
                "contracts",
                "entry_sha256",
            ],
        );
        exact_keys(&entry["contracts"], &FIELDS);
        assert_eq!(
            hash_without("MPK-SEMANTIC-PROFILE-ENTRY-1.0", entry, "entry_sha256"),
            entry["entry_sha256"]
        );
    }
    assert_eq!(entries[2]["entry_sha256"], ENTRY_HASH);
    assert_eq!(entries[2]["semantic_profile"], "mpk.java.scalar.v0");
    assert_eq!(
        entries[2]["semantic_parameters_schema"],
        "mpk.semantic_parameters.java_scalar.v0"
    );
    assert_eq!(
        entries[2]["selection_schema"],
        "mpk.selection.java_methods.v0"
    );
    for field in FIELDS {
        assert_eq!(
            entries[2]["contracts"][field],
            format!("mpk.profile.{field}.java_scalar.v0")
        );
    }
    assert_eq!(
        hash_without("MPK-SEMANTIC-PROFILE-REGISTRY-1.0", root, "registry_sha256"),
        REGISTRY_HASH
    );
    for case in array(&vectors["hash_cases"]) {
        assert_hash_case(root, case);
    }
    for case in array(&vectors["append_only_cases"]) {
        match text(&case["id"]) {
            "append.exact_count" => {
                assert_eq!(entries.len() as u64, number(&case["expected_count"]))
            }
            "append.exact_order" => assert_eq!(
                entries
                    .iter()
                    .map(|e| e["source_language"].clone())
                    .collect::<Vec<_>>(),
                *array(&case["expected_languages"])
            ),
            "append.no_later_language" => {
                for absent in array(&case["absent_languages"]) {
                    assert!(!entries
                        .iter()
                        .any(|entry| &entry["source_language"] == absent));
                }
            }
            "append.csharp_bytes_unchanged"
            | "append.go_bytes_unchanged"
            | "append.rust_bytes_unchanged" => {
                let old = number(&case["predecessor_index"]) as usize;
                let new = number(&case["successor_index"]) as usize;
                assert_eq!(
                    canonical(&predecessor["registry"]["profiles"][old]),
                    canonical(&entries[new])
                );
            }
            other => panic!("unowned append-only case {other}"),
        }
    }
    assert_eq!(array(&vectors["append_only_cases"]).len(), 6);
    assert_eq!(array(&vectors["activation_cases"]).len(), 8);
    assert_eq!(vectors["activation_cases"][0]["result"], "inactive");
    assert_eq!(array(&vectors["mutation_cases"]).len(), 10);
    for case in array(&vectors["mutation_cases"]) {
        let mut changed = root.clone();
        apply_mutation(&mut changed, case);
        if case["repair_hashes"] == true {
            for entry in changed["profiles"].as_array_mut().unwrap() {
                entry["entry_sha256"] =
                    hash_without("MPK-SEMANTIC-PROFILE-ENTRY-1.0", entry, "entry_sha256").into();
            }
            changed["registry_sha256"] = hash_without(
                "MPK-SEMANTIC-PROFILE-REGISTRY-1.0",
                &changed,
                "registry_sha256",
            )
            .into();
        }
        assert_ne!(canonical(&changed), canonical(root));
        assert!(
            !registry_specimen_valid(&changed, root, &predecessor["registry"]),
            "{}",
            case["id"]
        );
    }
}

fn registry_specimen_valid(value: &Value, frozen: &Value, predecessor: &Value) -> bool {
    if !has_keys(
        value,
        &["schema", "id", "revision", "profiles", "registry_sha256"],
    ) || value["schema"] != "mpk.semantic_profile.registry.v1"
        || value["id"] != value["schema"]
        || value["revision"] != 3
    {
        return false;
    }
    let Some(entries) = value["profiles"].as_array() else {
        return false;
    };
    if entries.len() != 4 || entries[2] != frozen["profiles"][2] {
        return false;
    }
    for (old, new) in [(0, 0), (1, 1), (2, 3)] {
        if canonical(&entries[new]) != canonical(&predecessor["profiles"][old]) {
            return false;
        }
    }
    value["registry_sha256"]
        == hash_without(
            "MPK-SEMANTIC-PROFILE-REGISTRY-1.0",
            value,
            "registry_sha256",
        )
        && value["registry_sha256"] == REGISTRY_HASH
}

#[test]
fn java_profile_context_and_nine_closed_payloads_are_consistent() {
    let profile = load(PROFILE);
    exact_keys(
        &profile,
        &[
            "schema",
            "owner_test",
            "spec_schema",
            "mechanism_schema",
            "profile_identity",
            "semantic_parameters",
            "semantic_context_fixture",
            "selection_fixture",
            "selection_sha256",
            "contract_fixture",
            "contract_sidecar_sha256",
            "normalized_contract_fixture",
            "toolchain_inputs",
            "profile_contracts",
            "case_harness",
            "shared_envelope_limits",
            "semantic_rows",
            "type_mappings",
            "operation_mappings",
            "conversion_rules",
            "accepted_cases",
            "rejected_cases",
            "precedence_cases",
            "limit_cases",
            "diagnostic_registry",
            "diagnostic_normalization",
            "source_map_cases",
            "upgrade_cases",
            "cfg_patterns",
            "compiler_session",
            "launcher_contract",
            "adapter_observations",
            "host_probe",
            "mutation_cases",
            "hash_cases",
            "isolation_cases",
        ],
    );
    assert_eq!(profile["schema"], "mpk.java.profile.conformance.v0");
    assert_eq!(
        profile["owner_test"],
        "crates/mpk-vc/tests/java_profile_spec.rs"
    );
    assert_eq!(profile["spec_schema"], "mpk.java.scalar.v0");
    assert_eq!(
        profile["mechanism_schema"],
        "mpk.semantic_profile.registry.v1"
    );
    let identity = &profile["profile_identity"];
    exact_keys(
        identity,
        &[
            "source_language",
            "semantic_profile",
            "semantic_parameters_schema",
            "selection_schema",
            "contract_schema",
            "profile_entry_sha256",
            "registry_revision",
            "registry_sha256",
        ],
    );
    assert_eq!(identity["source_language"], "java");
    assert_eq!(identity["semantic_profile"], "mpk.java.scalar.v0");
    assert_eq!(
        identity["semantic_parameters_schema"],
        "mpk.semantic_parameters.java_scalar.v0"
    );
    assert_eq!(
        identity["selection_schema"],
        "mpk.selection.java_methods.v0"
    );
    assert_eq!(identity["contract_schema"], "mpk.java.contract.v0");
    assert_eq!(identity["profile_entry_sha256"], ENTRY_HASH);
    assert_eq!(identity["registry_revision"], 3);
    assert_eq!(identity["registry_sha256"], REGISTRY_HASH);
    let context = expected_context();
    assert_eq!(
        profile["semantic_parameters"],
        context["semantic_parameters"]
    );
    assert_eq!(profile["semantic_context_fixture"], context);
    assert!(specimen_valid(
        &profile,
        "/selection_fixture",
        &profile["selection_fixture"]
    ));
    assert!(specimen_valid(
        &profile,
        "/contract_fixture",
        &profile["contract_fixture"]
    ));
    assert_eq!(
        profile["case_harness"]["baseline_selection"],
        profile["selection_fixture"]
    );
    let source = text(&profile["case_harness"]["baseline_files"]["src/demo/Policy.java"]);
    assert!(source.contains("public static boolean approved(long reserve, long debit)"));
    assert!(source.ends_with('\n'));
    assert!(!source.contains('\r'));
    let contracts = array(&profile["profile_contracts"]);
    assert_eq!(contracts.len(), 9);
    for (index, field) in FIELDS.iter().enumerate() {
        let contract = &contracts[index];
        exact_keys(contract, &["field", "envelope", "canonical_envelope_bytes"]);
        assert_eq!(contract["field"], *field);
        let expected = expected_payload(&profile, field);
        assert_eq!(contract["envelope"], expected, "{field} payload changed");
        assert_eq!(
            canonical(&expected).len() as u64,
            number(&contract["canonical_envelope_bytes"])
        );
        assert!(canonical(&expected).len() <= 1_048_576);
    }
    let normalized = &profile["normalized_contract_fixture"];
    exact_keys(
        normalized,
        &[
            "semantic_context",
            "unit_id",
            "function_id",
            "requires",
            "ensures",
            "modifies",
            "panic",
            "termination",
            "loops",
            "contract_hash",
        ],
    );
    assert_eq!(normalized["semantic_context"], expected_context());
    assert_eq!(normalized["unit_id"], "payment-policy");
    assert_eq!(
        normalized["function_id"],
        profile["contract_fixture"]["method"]
    );
    assert_eq!(
        normalized["requires"][0],
        json!({"op":"signed_ge", "lhs":{"var":"arg0"}, "rhs":{"var":"arg1"}})
    );
    assert_eq!(
        normalized["requires"][1],
        json!({"op":"signed_ge", "lhs":{"var":"arg0"}, "rhs":{"int":{"value":"0","width":64,"signed":true}}})
    );
    assert_eq!(
        normalized["ensures"],
        json!([{"op":"eq","lhs":{"result":0},"rhs":{"bool":true}}])
    );
    assert_eq!(normalized["panic"], "forbidden");
    assert_eq!(normalized["termination"], "total");
    assert_eq!(normalized["modifies"], json!([]));
    assert_eq!(normalized["loops"], json!([]));
    for case in array(&profile["hash_cases"]) {
        assert_hash_case(&profile, case);
    }
    assert_eq!(
        profile["selection_sha256"],
        hash("MPK-JAVA-SELECTION-0.1", &profile["selection_fixture"])
    );
    assert_eq!(
        profile["contract_sidecar_sha256"],
        hash(
            "MPK-JAVA-CONTRACT-SIDECAR-0.1",
            &profile["contract_fixture"]
        )
    );
    assert_eq!(
        normalized["contract_hash"],
        hash_without("MPK-CONTRACT-1.0", normalized, "contract_hash")
    );
    assert_eq!(
        profile["toolchain_inputs"]["toolchain_inputs_sha256"],
        hash_without(
            "MPK-JAVA-TOOLCHAIN-INPUTS-0.1",
            &profile["toolchain_inputs"],
            "toolchain_inputs_sha256"
        )
    );
}

#[test]
fn java_specification_mutations_are_owned_and_rejected_by_fixture_invariants() {
    let profile = load(PROFILE);
    let cases = array(&profile["mutation_cases"]);
    assert_eq!(cases.len(), 53);
    let mut ids = BTreeSet::new();
    for case in cases {
        assert!(ids.insert(text(&case["id"])));
        let fixture = text(&case["fixture"]);
        let baseline = profile.pointer(fixture).expect("owned fixture");
        assert!(
            specimen_valid(&profile, fixture, baseline),
            "baseline for {}",
            case["id"]
        );
        let mut changed = baseline.clone();
        apply_mutation(&mut changed, case);
        assert_ne!(&changed, baseline, "mutation must change its fixture");
        assert_eq!(case["expected"], "reject");
        assert!(
            !specimen_valid(&profile, fixture, &changed),
            "unrejected specimen mutation {}",
            case["id"]
        );
    }
    // Raw transport errors cannot be expressed by mutations on a parsed Value.
    for bytes in [
        br#"{"schema":"a","schema":"b"}"#.as_slice(),
        br#"{"preview":1.0}"#.as_slice(),
        br#"{"limit":9007199254740992}"#.as_slice(),
        br#"{"identifier":"\ud800"}"#.as_slice(),
    ] {
        assert!(parse_strict_json(bytes, LIMITS).is_err());
    }
}

#[test]
fn java_common_envelope_limits_count_schema_and_value() {
    let profile = load(PROFILE);
    let cases = array(&profile["shared_envelope_limits"]);
    assert_eq!(cases.len(), 3);
    for case in cases {
        exact_keys(
            case,
            &[
                "id",
                "maximum",
                "scope",
                "exact_boundary",
                "boundary_plus_one",
            ],
        );
        let limit = SemanticRegistryLimit::from_id(text(&case["id"])).expect("known common limit");
        let maximum = number(&case["maximum"]);
        assert_eq!(limit.inclusive_maximum(), maximum);
        assert_eq!(case["scope"], "complete_envelope");
        assert!(validate_semantic_registry_limit(limit, maximum).is_ok());
        assert!(validate_semantic_registry_limit(limit, maximum + 1).is_err());
        let base = json!({"schema":"mpk.test.v0", "value":{"padding":""}});
        let padding = maximum as usize - canonical(&base).len();
        let envelope = json!({"schema":"mpk.test.v0", "value":{"padding":"x".repeat(padding)}});
        assert_eq!(canonical(&envelope).len() as u64, maximum);
        assert!(canonical(&envelope["value"]).len() < maximum as usize);
        let overflow = json!({"schema":"mpk.test.v0", "value":{"padding":"x".repeat(padding + 1)}});
        assert_eq!(canonical(&overflow).len() as u64, maximum + 1);
        assert!(
            validate_semantic_registry_limit(limit, canonical(&overflow).len() as u64).is_err()
        );
    }
}

#[test]
fn java_semantic_partition_cases_checks_and_diagnostics_are_closed() {
    let profile = load(PROFILE);
    assert_eq!(
        profile["type_mappings"],
        json!([
            {"source":"boolean", "vir":{"kind":"bool"}},
            {"source":"int", "vir":{"kind":"bv","width":32,"signed":true}},
            {"source":"long", "vir":{"kind":"bv","width":64,"signed":true}}
        ])
    );
    let expected_accepts = BTreeSet::from([
        "M01", "M02", "M07", "M08", "M09", "M10", "M11", "M12", "M13", "M16", "M18", "M19", "M21",
        "M27", "M29", "M33", "M34",
    ]);
    let rows = array(&profile["semantic_rows"]);
    assert_eq!(rows.len(), 34);
    let mut rejected_rows = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        exact_keys(row, &["row", "disposition", "basis"]);
        assert_eq!(row["row"], format!("M{:02}", index + 1));
        let accepted = expected_accepts.contains(text(&row["row"]));
        assert_eq!(
            row["disposition"],
            if accepted {
                "accept_under_profile_restrictions"
            } else {
                "reject_before_vir"
            }
        );
        if !accepted {
            rejected_rows.insert(text(&row["row"]));
        }
        assert!(!text(&row["basis"]).is_empty());
    }
    assert_eq!(rejected_rows.len(), 17);

    let operations = array(&profile["operation_mappings"]);
    let mut operation_keys = BTreeSet::new();
    for mapping in operations {
        let source = text(&mapping["source"]);
        let rule = text(&mapping["operand_rule"]);
        assert!(operation_keys.insert((source, rule)));
        let checks = &mapping["required_checks"];
        if matches!(source, "/" | "%") {
            assert_eq!(checks, &json!(["divisor_nonzero"]));
        } else if source != "direct_static_call" {
            assert_eq!(
                checks,
                &json!([]),
                "Java arithmetic must not inherit C# overflow or shift-range checks"
            );
        }
        if matches!(source, "<<" | ">>" | ">>>") {
            assert_eq!(mapping["mask_width"], 32);
            assert_eq!(
                mapping["mask"],
                if rule.starts_with("signed-bv32") {
                    31
                } else {
                    63
                }
            );
            let lowering = array(&mapping["lowering"]);
            assert_eq!(lowering[0], "Const(mask)");
            assert_eq!(lowering[1], "bv_and");
            if source == ">>>" {
                assert_eq!(
                    mapping["lowering"],
                    json!([
                        "Const(mask)",
                        "bv_and",
                        "Convert(signed-to-unsigned)",
                        "bv_lshr",
                        "Convert(unsigned-to-signed)"
                    ])
                );
            }
        }
    }
    for operator in [
        "!",
        "==",
        "!=",
        "<",
        "<=",
        ">",
        ">=",
        "+",
        "-",
        "*",
        "unary -",
        "~",
        "&",
        "|",
        "^",
        "/",
        "%",
        "<<",
        ">>",
        ">>>",
        "&&",
        "||",
        "?:",
        "direct_static_call",
    ] {
        assert!(
            operation_keys.iter().any(|(source, _)| source == &operator),
            "missing {operator}"
        );
    }
    let conversions = array(&profile["conversion_rules"]);
    let mut conversion_keys = BTreeSet::new();
    assert_eq!(conversions.len(), 35);
    for conversion in conversions {
        exact_keys(
            conversion,
            &[
                "source",
                "target",
                "context",
                "accepted",
                "lowering",
                "required_checks",
            ],
        );
        let from = text(&conversion["source"]);
        let to = text(&conversion["target"]);
        let context = text(&conversion["context"]);
        assert!(conversion_keys.insert((from, to, context)));
        let accepted = from == to
            || context == "explicit_cast"
            || (from == "int"
                && to == "long"
                && matches!(context, "local_initializer" | "local_assignment" | "return"));
        assert_eq!(conversion["accepted"], accepted);
        assert_eq!(conversion["required_checks"], json!([]));
        if !accepted || from == to {
            assert_eq!(conversion["lowering"], json!([]));
        } else if from == "int" {
            assert_eq!(conversion["lowering"], json!(["Convert(sign-extend)"]));
        } else {
            assert_eq!(conversion["lowering"], json!(["Convert(low-32-bits)"]));
        }
    }

    let diagnostics = array(&profile["diagnostic_registry"]);
    let mut by_code = BTreeMap::new();
    for diagnostic in diagnostics {
        exact_keys(diagnostic, &["code", "status", "phase", "exit", "message"]);
        let code = text(&diagnostic["code"]);
        assert!(code.starts_with("JAVA_"));
        assert!(by_code.insert(code, diagnostic).is_none());
        assert!(!text(&diagnostic["message"]).is_empty());
        assert_eq!(
            diagnostic["exit"],
            match text(&diagnostic["status"]) {
                "source-error" => 4,
                "rejected" => 3,
                "frontend-error" => 1,
                other => panic!("unknown diagnostic status {other}"),
            }
        );
    }
    let accepted = array(&profile["accepted_cases"]);
    let mut ids = BTreeSet::new();
    let mut covered_accepted = BTreeSet::new();
    for case in accepted {
        exact_keys(
            case,
            &[
                "id",
                "rows",
                "sources",
                "methods",
                "contracts",
                "expected_profile_operations",
                "expected_required_checks",
                "evaluation_cases",
            ],
        );
        assert!(ids.insert(text(&case["id"])));
        for row in array(&case["rows"]) {
            assert!(expected_accepts.contains(text(row)));
            covered_accepted.insert(text(row));
        }
        for (path, source) in case["sources"].as_object().unwrap() {
            assert!(path.starts_with("src/") && path.ends_with(".java"));
            assert!(text(source).ends_with('\n'));
        }
        let methods = array(&case["methods"]);
        assert!(!methods.is_empty());
        assert!(methods
            .windows(2)
            .all(|pair| text(&pair[0]) < text(&pair[1])));
        let sidecars = array(&case["contracts"]);
        for method in methods {
            assert_eq!(
                sidecars
                    .iter()
                    .filter(|sidecar| &sidecar["method"] == method)
                    .count(),
                1
            );
        }
        for check in array(&case["expected_required_checks"]) {
            assert!(matches!(
                text(check),
                "divisor_nonzero" | "callee_contract_hash"
            ));
        }
        assert_numeric_evaluations(case);
    }
    assert_eq!(covered_accepted, expected_accepts);
    let mut covered_rejected = BTreeSet::new();
    for case in array(&profile["rejected_cases"]) {
        assert!(ids.insert(text(&case["id"])));
        assert!(
            accepted
                .iter()
                .any(|baseline| baseline["id"] == case["baseline"]),
            "missing rejection baseline {}",
            case["id"]
        );
        let diagnostic = by_code
            .get(text(&case["expected_code"]))
            .expect("registered rejection diagnostic");
        assert_eq!(
            case["expected_status"], diagnostic["status"],
            "{}",
            case["id"]
        );
        assert_diagnostic_phase(&case["expected_phase"], &diagnostic["phase"]);
        for row in array(&case["rows"]) {
            covered_rejected.insert(text(row));
        }
    }
    assert!(rejected_rows.is_subset(&covered_rejected));
    let expected_limits = BTreeMap::from([
        ("source_files", 256),
        ("source_file_bytes", 1_048_576),
        ("source_total_bytes", 16_777_216),
        ("contract_files", 128),
        ("contract_file_bytes", 1_048_576),
        ("contract_total_bytes", 8_388_608),
        ("snapshot_entries", 512),
        ("snapshot_total_bytes", 33_554_432),
        ("normalized_path_bytes", 1024),
        ("canonical_method_id_bytes", 1024),
        ("selected_methods", 32),
        ("method_closure", 128),
        ("syntax_nodes", 250_000),
        ("syntax_depth", 256),
        ("instructions_per_method", 100_000),
        ("instructions_per_closure", 250_000),
        ("cfg_blocks_per_method", 1024),
        ("cfg_blocks_per_closure", 8192),
        ("contract_clauses", 64),
        ("contract_nodes_per_method", 1024),
        ("contract_nodes_per_closure", 8192),
        ("contract_depth", 32),
        ("normalized_issues", 1024),
        ("diagnostic_message_bytes", 4096),
        ("diagnostic_total_message_bytes", 2_097_152),
        ("frontend_argument_bytes", 131_072),
        ("frontend_stdout", 268_435_456),
        ("frontend_stderr", 2_097_152),
        ("vir_canonical_bytes", 201_326_592),
        ("source_map_canonical_bytes", 33_554_432),
        ("source_manifest_canonical_bytes", 4_194_304),
        ("parameter_slots", 255),
    ]);
    let mut actual_limits = BTreeMap::new();
    for case in array(&profile["limit_cases"]) {
        let maximum = number(&case["limit"]);
        assert!(actual_limits.insert(text(&case["id"]), maximum).is_none());
        assert_eq!(number(&case["boundary"]), maximum);
        assert_eq!(number(&case["overflow"]), maximum + 1);
        assert_eq!(case["expected_boundary"], "accept_at_counter");
        let diagnostic = by_code
            .get(text(&case["expected_overflow_code"]))
            .expect("registered limit diagnostic");
        assert_eq!(
            case["expected_overflow_status"], diagnostic["status"],
            "{}",
            case["id"]
        );
        assert_diagnostic_phase(&case["expected_overflow_phase"], &diagnostic["phase"]);
    }
    assert_eq!(actual_limits, expected_limits);
}

#[test]
fn java_frozen_toolchain_inventory_and_host_closure_are_bound() {
    let profile = load(PROFILE);
    let inputs = &profile["toolchain_inputs"];
    exact_keys(
        inputs,
        &[
            "schema",
            "id",
            "compiler_profile_id",
            "runtime_profile_id",
            "system_modules_profile_id",
            "execution_host_profile_id",
            "runtime_layout_profile_id",
            "archive",
            "release_metadata",
            "jdk_inventory",
            "native_image",
            "native_inventory",
            "runtime_linkage",
            "host",
            "system_module_inventory",
            "archive_policy",
            "toolchain_inputs_sha256",
        ],
    );
    assert_eq!(inputs["schema"], "mpk.java.toolchain_inputs.v0");
    assert_eq!(
        inputs["toolchain_inputs_sha256"],
        "a75175ba0cce86d97a8e056d4dda7a0826bb6676ba551c454bd65e5d44d23fc4"
    );
    assert_eq!(inputs["archive"]["bytes"], 141_329_719);
    assert_eq!(
        inputs["archive"]["sha256"],
        "dbb698396d478e7fa2b1e50f4103324b2a99b90569ee27c33f2261f9215cf41e"
    );
    assert_eq!(
        inputs["release_metadata"]["JAVA_RUNTIME_VERSION"],
        "25.0.4.1+1-LTS"
    );
    assert_eq!(inputs["release_metadata"]["OS_ARCH"], "x86_64");
    let inventory = array(&inputs["jdk_inventory"]);
    assert_eq!(inventory.len(), 486);
    let mut previous = "";
    for entry in inventory {
        let path = text(&entry["path"]);
        assert!(path > previous && !path.starts_with('/') && !path.contains('\\'));
        assert!(path == "." || !path.split('/').any(|part| matches!(part, "" | "." | "..")));
        previous = path;
        assert!(matches!(
            text(&entry["mode"]),
            "0444" | "0644" | "0755" | "0777"
        ));
        match text(&entry["kind"]) {
            "regular" => {
                assert!(entry["bytes"].as_u64().is_some());
                assert_sha256(&entry["sha256"]);
            }
            "directory" => {}
            "symlink" => assert!(entry.get("target").and_then(Value::as_str).is_some()),
            other => panic!("unexpected archive entry kind {other}"),
        }
    }
    for reference in array(&inputs["system_module_inventory"]) {
        assert!(
            inventory.contains(reference),
            "system modules must be pinned JDK entries"
        );
    }
    assert_eq!(array(&inputs["system_module_inventory"]).len(), 3);
    let native = array(&inputs["native_inventory"]);
    assert_eq!(native.len(), 6);
    for library in native {
        exact_keys(
            library,
            &["path", "source_path", "bytes", "sha256", "mode", "elf"],
        );
        assert_sha256(&library["sha256"]);
        assert!(number(&library["bytes"]) > 0);
    }
    let host = &inputs["host"];
    assert_eq!(host["architecture"], "x86_64");
    assert_eq!(host["glibc"], "2.36");
    assert_eq!(host["memory_max"], 1_073_741_824_u64);
    assert_eq!(host["memory_swap_max"], 0);
    assert_eq!(host["address_space_bytes"], 17_179_869_184_u64);
    assert_eq!(host["pids_max"], 128);
    assert_eq!(host["tmpfs_bytes"], 67_108_864);
    assert_eq!(host["timeout_seconds"], 120);
    assert_eq!(
        host["tmpfs_mount_flags"],
        json!(["nosuid", "nodev", "noexec", "noswap"])
    );
    assert_eq!(host["proc"], "readonly_private_pid_namespace");
    let linkage = &inputs["runtime_linkage"];
    assert_eq!(
        linkage["jdk_runtime_files"],
        json!([
            "bin/java",
            "lib/libjava.so",
            "lib/libjimage.so",
            "lib/libjli.so",
            "lib/libnet.so",
            "lib/libnio.so",
            "lib/libzip.so",
            "lib/server/libjvm.so",
        ])
    );
    let mut runtime_files = BTreeMap::new();
    for path in array(&linkage["jdk_runtime_files"]) {
        let entry = inventory
            .iter()
            .find(|item| &item["path"] == path)
            .expect("pinned runtime ELF");
        runtime_files.insert(format!("/mpk/toolchain/jdk/{}", text(path)), entry);
    }
    for entry in native {
        runtime_files.insert(format!("/{}", text(&entry["path"])), entry);
    }
    let expected_dependencies = runtime_files
        .iter()
        .flat_map(|(path, entry)| {
            array(&entry["elf"]["needed"])
                .iter()
                .map(move |needed| (path.clone(), text(needed).to_owned()))
        })
        .collect::<BTreeSet<_>>();
    let mut actual_dependencies = BTreeSet::new();
    for edge in array(&linkage["resolved_needed"]) {
        exact_keys(edge, &["from", "needed", "to"]);
        let from = text(&edge["from"]);
        let needed = text(&edge["needed"]);
        let to = text(&edge["to"]);
        assert!(runtime_files.contains_key(from) && runtime_files.contains_key(to));
        assert_eq!(to.rsplit('/').next(), Some(needed));
        assert!(actual_dependencies.insert((from.to_owned(), needed.to_owned())));
    }
    assert_eq!(actual_dependencies, expected_dependencies);
    assert_eq!(
        runtime_files["/mpk/toolchain/jdk/bin/java"]["elf"]["interpreter"],
        host["interpreter"]
    );
}

#[test]
fn java_compiler_and_launcher_observations_are_scoped_and_closed() {
    let profile = load(PROFILE);
    let session = &profile["compiler_session"];
    exact_keys(
        session,
        &[
            "jdk_runtime_version",
            "java_vendor",
            "compiler_provider",
            "options",
            "locale",
            "source_encoding",
            "phases",
            "generate_called",
            "compilation_task_call_called",
            "fresh_task_per_case",
            "max_retained_diagnostics",
        ],
    );
    assert_eq!(session["jdk_runtime_version"], "25.0.4.1+1-LTS");
    assert_eq!(
        session["options"],
        json!([
            "--release",
            "25",
            "-encoding",
            "UTF-8",
            "-proc:none",
            "-implicit:none",
            "-Xlint:none",
            "-Xmaxerrs",
            "1025",
            "-Xmaxwarns",
            "1025"
        ])
    );
    assert_eq!(session["phases"], json!(["parse", "analyze"]));
    assert_eq!(session["generate_called"], false);
    assert_eq!(session["compilation_task_call_called"], false);
    assert_eq!(session["max_retained_diagnostics"], 1024);
    let launcher = &profile["launcher_contract"];
    exact_keys(
        launcher,
        &[
            "profile_id",
            "program",
            "working_directory",
            "stdin",
            "stdout",
            "stderr",
            "argv_prefix",
            "frontend_argv_template",
            "repeated_argument_expansion",
            "environment",
            "inherited_environment",
        ],
    );
    assert_eq!(launcher["profile_id"], "mpk.java.jvm_launcher.v0");
    assert_eq!(launcher["program"], "/mpk/toolchain/jdk/bin/java");
    assert_eq!(
        launcher["argv_prefix"],
        json!([
            "/mpk/toolchain/jdk/bin/java",
            "-Xint",
            "-Xshare:off",
            "-XX:+UseSerialGC",
            "-XX:ActiveProcessorCount=1",
            "-XX:+DisableAttachMechanism",
            "-XX:-UsePerfData",
            "-Xms32m",
            "-Xmx512m",
            "-Xss1m",
            "-Dfile.encoding=UTF-8",
            "-Duser.language=en",
            "-Duser.country=US",
            "-Duser.timezone=UTC",
            "-Djava.io.tmpdir=/mpk/tmp",
            "-Duser.home=/mpk/empty-home",
            "-Djava.library.path=/nonexistent",
            "-XX:ErrorFile=/mpk/tmp/hs_err.log",
            "-XX:-CreateCoredumpOnCrash",
            "-XX:-HeapDumpOnOutOfMemoryError",
            "--limit-modules",
            "java.base,java.compiler,jdk.compiler,jdk.zipfs",
            "--add-modules",
            "java.compiler,jdk.compiler,jdk.zipfs",
            "-cp",
            "/mpk/frontend/java2vir.jar",
            "mpk.java2vir.Main"
        ])
    );
    assert_eq!(
        launcher["environment"],
        json!({
            "HOME":"/mpk/empty-home", "TMPDIR":"/mpk/tmp", "PATH":"/nonexistent", "LANG":"C.UTF-8", "LC_ALL":"C.UTF-8", "TZ":"UTC"
        })
    );
    assert_eq!(launcher["inherited_environment"], json!([]));
    let arguments = array(&launcher["frontend_argv_template"]);
    for (option, value) in [
        ("--profile-registry-revision", "3"),
        ("--profile-registry-sha256", REGISTRY_HASH),
        ("--profile-entry-sha256", ENTRY_HASH),
    ] {
        let index = arguments.iter().position(|arg| arg == option).unwrap();
        assert_eq!(arguments[index + 1], value);
    }

    let observations = &profile["adapter_observations"];
    let normalization = &profile["diagnostic_normalization"];
    let allowed_codes = array(&normalization["compiler_code_allowlist"])
        .iter()
        .map(text)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        allowed_codes,
        BTreeSet::from([
            "compiler.err.cant.resolve.location",
            "compiler.err.doesnt.exist",
            "compiler.err.int.number.too.large",
            "compiler.err.premature.eof",
            "compiler.err.prob.found.req",
            "compiler.err.var.might.not.have.been.initialized",
        ])
    );
    for code in &allowed_codes {
        assert_eq!(
            normalization["compiler_kind_allowlist"][*code],
            json!(["ERROR"])
        );
    }
    assert_eq!(
        observations["boot_modules"],
        json!([
            "java.base",
            "java.compiler",
            "jdk.compiler",
            "jdk.internal.opt",
            "jdk.zipfs"
        ])
    );
    assert_eq!(observations["planted_processor_executed"], false);
    for result in observations["file_manager_boundary_checks"]
        .as_object()
        .unwrap()
        .values()
    {
        assert_eq!(result, true);
    }
    let cases = array(&observations["cases"]);
    assert_eq!(cases.len(), 20);
    let mut observed_codes = BTreeSet::new();
    for case in cases {
        assert_eq!(case["output_attempts"], 0);
        assert_eq!(case["writer_characters"], 0);
        assert_eq!(case["manager_closed"], true);
        assert_eq!(case["after_close"], "java.lang.IllegalStateException");
        for diagnostic in array(&case["diagnostics"]) {
            let code = text(&diagnostic["code"]);
            assert!(allowed_codes.contains(code));
            assert_eq!(diagnostic["kind"], "ERROR");
            observed_codes.insert(code);
        }
    }
    assert_eq!(
        observed_codes, allowed_codes,
        "every allowed compiler code needs a pinned observation"
    );
    let literals = cases
        .iter()
        .find(|case| case["id"] == "negative-literals")
        .unwrap();
    assert_eq!(literals["public_tree_inventory_unchanged"], true);
    for minimum in ["-2147483648", "-9223372036854775808"] {
        assert!(array(&literals["before_analysis"])
            .iter()
            .any(|tree| tree["literal_value"] == minimum));
    }
    // Known excluded source parents are rejected before accepted-subtree
    // adapter comparisons; javac's generated descendants do not change that
    // outcome to frontend-error. Attribution diagnostics still precede both.
    for (id, source_kind, generated_kind, precedence_id, rejection_code) in [
        (
            "excluded-class-default-constructor",
            "CLASS",
            "METHOD",
            "excluded_class_before_accepted_tree_compare",
            "JAVA_SUBSET_DECLARATION",
        ),
        (
            "excluded-var-inferred-type",
            "VARIABLE",
            "PRIMITIVE_TYPE",
            "excluded_var_before_accepted_tree_compare",
            "JAVA_SUBSET_TYPE",
        ),
    ] {
        let case = cases
            .iter()
            .find(|case| case["id"] == id)
            .expect("excluded-source adapter observation");
        assert_eq!(case["diagnostics_seen"], 0);
        assert_eq!(case["public_tree_inventory_unchanged"], false);
        assert!(array(&case["before_analysis"])
            .iter()
            .any(|tree| tree["kind"] == source_kind));
        assert!(array(&case["after_analysis"])
            .iter()
            .any(|tree| tree["kind"] == generated_kind && tree["end_utf16"] == -1));
        let precedence = array(&profile["precedence_cases"])
            .iter()
            .find(|case| case["id"] == precedence_id)
            .expect("excluded-parent precedence vector");
        assert_eq!(precedence["expected_status"], "rejected");
        assert_eq!(precedence["expected_phase"], "subset");
        assert_eq!(precedence["expected_code"], rejection_code);
    }
    let inferred = cases
        .iter()
        .find(|case| case["id"] == "excluded-var-inferred-type")
        .unwrap();
    assert!(array(&inferred["after_analysis"])
        .iter()
        .any(|tree| tree["kind"] == "VARIABLE"
            && tree["spelling"] == "var y = x;"
            && tree["type"] == "int"));
    let abort = cases
        .iter()
        .find(|case| case["id"] == "diagnostic-listener-abort-1025")
        .unwrap();
    assert_eq!(abort["diagnostics_seen"], 1025);
    assert_eq!(array(&abort["diagnostics"]).len(), 1024);
    assert!(abort.get("thrown").is_some());
    let slots = cases
        .iter()
        .find(|case| case["id"] == "parameter-slots-256")
        .unwrap();
    assert_eq!(
        slots["diagnostics_seen"], 0,
        "manual profile slot limit is required despite clean analyze"
    );
    assert_eq!(slots["analyze_called"], true);

    let host = &profile["host_probe"];
    assert!(text(&host["scope"]).contains("not the production runner"));
    assert_eq!(host["execution_architecture"], "linux/amd64");
    assert_eq!(host["outer_host_architecture"], "aarch64");
    assert_eq!(
        host["unmeasured"],
        json!([
            "integrated_user_namespace_bootstrap",
            "native_x86_64_syscall_and_clone3_trace",
            "production_seccomp_policy",
            "resource_exhaustion_and_timeout_cleanup",
            "complete_installed_runner_release_gate",
        ])
    );
    assert_eq!(
        host["outer_seccomp"],
        "unconfined_for_disposable_mount_setup"
    );
    assert_eq!(
        host["harness_differences"]["selected_source_execution"],
        false
    );
    for result in host["runtime"]["checks"].as_object().unwrap().values() {
        assert_eq!(result, true);
    }
    let inventories = &profile["toolchain_inputs"];
    for path in array(&observations["native_mapped_paths"])
        .iter()
        .chain(array(&host["runtime"]["loaded_native_files"]))
    {
        let path = text(path);
        let recorded = if let Some(relative) = path.strip_prefix("/mpk/toolchain/jdk/") {
            array(&inventories["jdk_inventory"])
                .iter()
                .any(|item| item["path"] == relative && item["kind"] == "regular")
        } else {
            array(&inventories["native_inventory"])
                .iter()
                .any(|item| item["path"] == path.trim_start_matches('/'))
        };
        assert!(recorded, "unrecorded measured native input {path}");
    }
    let isolation = array(&profile["isolation_cases"]);
    assert_eq!(isolation.len(), 15);
    for case in isolation {
        exact_keys(
            case,
            &[
                "id",
                "mutation",
                "expect",
                "implementation_owner",
                "implementation_task",
            ],
        );
        assert_eq!(case["implementation_task"], "JAVA-03-T07");
        assert_eq!(
            case["implementation_owner"],
            "crates/mpk-cli/tests/java_frontend_runner.rs"
        );
        assert!(matches!(
            text(&case["expect"]),
            "reject" | "reject_or_unavailable"
        ));
    }
}

fn assert_sha256(value: &Value) {
    let text = text(value);
    assert_eq!(text.len(), 64);
    assert!(text
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
}

#[test]
fn java_cfg_goldens_have_explicit_forward_edges_and_join_arguments() {
    let profile = load(PROFILE);
    let patterns = array(&profile["cfg_patterns"]);
    assert_eq!(patterns.len(), 6);
    for pattern in patterns {
        let blocks = array(&pattern["blocks"]);
        for (index, block) in blocks.iter().enumerate() {
            assert_eq!(block["label"], format!("bb{index}"));
            let terminator = &block["terminator"];
            let destinations: Vec<&str> = match text(&terminator["kind"]) {
                "Branch" => vec![
                    text(&terminator["else_label"]),
                    text(&terminator["then_label"]),
                ],
                "Jump" => vec![text(&terminator["label"])],
                "Return" => {
                    assert_eq!(array(&terminator["values"]).len(), 1);
                    vec![]
                }
                other => panic!("unknown CFG terminator {other}"),
            };
            for destination in destinations {
                let target = destination
                    .strip_prefix("bb")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                assert!(target > index && target < blocks.len());
                if terminator["kind"] == "Jump" {
                    assert_eq!(
                        array(&terminator["args"]).len(),
                        array(&blocks[target]["parameters"]).len()
                    );
                }
            }
        }
    }
    let ternary = patterns.iter().find(|p| p["id"] == "ternary").unwrap();
    assert_eq!(ternary["blocks"][0]["terminator"]["else_label"], "bb1");
    assert_eq!(ternary["blocks"][0]["terminator"]["then_label"], "bb2");
    assert_eq!(
        ternary["blocks"][3]["parameters"],
        json!([{"id":"p0","type":"int"}])
    );
    let calls = patterns
        .iter()
        .find(|p| p["id"] == "nested_call_arguments")
        .unwrap();
    let calls = array(&calls["blocks"][0]["instructions"]);
    assert_eq!(
        calls
            .iter()
            .map(|c| text(&c["function"]))
            .collect::<Vec<_>>(),
        [
            "vector.Case::left(int)->int",
            "vector.Case::right(int)->int",
            "vector.Case::add(int,int)->int"
        ]
    );
    assert_eq!(calls[2]["args"], json!(["t0", "t1"]));
}

#[test]
fn java_source_map_goldens_use_original_utf8_boundaries() {
    let profile = load(PROFILE);
    for case in array(&profile["source_map_cases"]) {
        let source = text(&case["source"]);
        let start = number(&case["utf16_start"]);
        let end = number(&case["utf16_end"]);
        let mut boundaries = BTreeMap::from([(0, 0)]);
        let mut utf16 = 0_u64;
        for (byte, character) in source.char_indices() {
            utf16 += character.len_utf16() as u64;
            boundaries.insert(utf16, (byte + character.len_utf8()) as u64);
        }
        let mapped = if start < end {
            boundaries
                .get(&start)
                .zip(boundaries.get(&end))
                .map(|(start, end)| json!([start, end]))
        } else {
            None
        };
        if case["expected_status"] == "accept" {
            assert_eq!(
                mapped,
                Some(case["expected_utf8_range"].clone()),
                "{}",
                case["id"]
            );
        } else {
            assert!(
                mapped.is_none(),
                "invalid source boundary accepted: {}",
                case["id"]
            );
        }
    }
}

fn assert_diagnostic_phase(actual: &Value, rule: &Value) {
    if rule == "started_phase" {
        assert!(matches!(
            text(actual),
            "capture" | "source" | "metadata" | "typecheck" | "subset" | "lowering" | "emission"
        ));
    } else {
        assert_eq!(actual, rule);
    }
}

// Independently check the arithmetic goldens with a wider host carrier. No
// source parsing or frontend execution is implied by this numeric-vector test.
fn assert_numeric_evaluations(case: &Value) {
    let Some((kind, operation)) = text(&case["id"]).split_once('.') else {
        return;
    };
    let width = match kind {
        "int" => 32,
        "long" => 64,
        _ => return,
    };
    let modulus = 1_i128 << width;
    let sign = 1_i128 << (width - 1);
    for evaluation in array(&case["evaluation_cases"]) {
        let arguments = array(&evaluation["arguments"])
            .iter()
            .map(|v| text(v).parse::<i128>().unwrap())
            .collect::<Vec<_>>();
        let lhs = arguments.first().copied().unwrap_or(0);
        let rhs = arguments.get(1).copied().unwrap_or(0);
        let actual = match operation {
            "identity" => lhs,
            "minimum" => -sign,
            "wrap_add" => lhs + rhs,
            "wrap_sub" => lhs - rhs,
            "wrap_mul" => lhs * rhs,
            "bitand" => lhs & rhs,
            "bitor" => lhs | rhs,
            "bitxor" => lhs ^ rhs,
            "negate" => -lhs,
            "bitnot" => !lhs,
            "division" => {
                assert_ne!(rhs, 0);
                lhs / rhs
            }
            "remainder" => {
                assert_ne!(rhs, 0);
                lhs % rhs
            }
            "shift_left" => lhs << (rhs & (width - 1)),
            "shift_right" => lhs >> (rhs & (width - 1)),
            "shift_unsigned_right" => lhs.rem_euclid(modulus) >> (rhs & (width - 1)),
            other => panic!("numeric golden owner absent for {other}"),
        };
        let bits = actual.rem_euclid(modulus);
        let signed = if bits >= sign { bits - modulus } else { bits };
        assert_eq!(
            signed.to_string(),
            evaluation["result"],
            "{} {arguments:?}",
            case["id"]
        );
    }
}

#[test]
fn java_freeze_does_not_activate_a_production_frontend() {
    // T03 compiles Java validators; installed membership remains revision 2.
    assert!(CompiledSemanticProfile::from_identity("java", "mpk.java.scalar.v0").is_some());
    let active = mpk_vc::semantic_profile_registry::validate_semantic_profile_registry(
        ACTIVE,
        mpk_vc::semantic_profile_registry::RegistryRevision::Revision2,
    )
    .unwrap();
    assert!(active.lookup("java", "mpk.java.scalar.v0").is_none());
    let mut active_transport = canonical(&load(PREDECESSOR)["registry"]);
    active_transport.push(b'\n');
    assert_eq!(ACTIVE, active_transport);
    for required in [
        "JAVA-03-T01",
        "mpk.java.scalar.v0",
        ENTRY_HASH,
        REGISTRY_HASH,
    ] {
        assert!(
            SPEC.contains(required),
            "normative document is missing {required}"
        );
    }
}

fn expected_context() -> Value {
    json!({
        "profile_registry":{"schema":"mpk.semantic_profile.registry.v1", "id":"mpk.semantic_profile.registry.v1", "revision":3, "registry_sha256":REGISTRY_HASH},
        "profile_entry_sha256":ENTRY_HASH, "source_language":"java", "semantic_profile":"mpk.java.scalar.v0",
        "semantic_parameters":{"schema":"mpk.semantic_parameters.java_scalar.v0", "value":{
            "language_version":"25", "release":"25", "preview":false, "encoding":"UTF-8", "annotation_processing":"none", "target_id":"linux-x64"
        }}
    })
}

fn expected_payload(profile: &Value, field: &str) -> Value {
    let value = match field {
        "ai" => {
            json!({"display_language":"Java", "projection_profile_id":"mpk.java.ai_projection.v0", "proof_authority":false, "redaction_profile_id":"minimal-v1", "source_access":false})
        }
        "evidence" => {
            json!({"proof_authority":"certificate_only", "recipe_profile_id":"mpk.java.evidence_recipe.v0", "require_reference_checker":true, "require_source_free_check":true})
        }
        "frontend" => {
            json!({"argument_profile_id":"mpk.java.frontend_arguments.v0", "environment_profile_id":"mpk.java.frontend_environment.v0", "launcher_profile_id":"mpk.java.jvm_launcher.v0", "limit_profile_id":"mpk.java.limits.v0", "private_driver":"none"})
        }
        "manifest" => {
            json!({"input_kinds":["contract","source"], "source_extension":".java", "unit_kind":"compilation"})
        }
        "policy" => {
            json!({"axiom_profile":"mvp-theory", "checker_profile":"mvp-strict", "strategy_profile":"payment-policy-java-alpha"})
        }
        "release" => json!({
            "compiler_profile_id":"mpk.java.javac_25_0_4_1_1.v0",
            "execution_host_profile_id":"mpk.host.linux-x86_64-gnu.java25.v0",
            "runtime_layout_profile_id":"mpk.runtime.linux-x86_64-gnu.java25.v0",
            "runtime_profile_id":"mpk.java.hotspot_25_0_4_1_1.v0",
            "system_modules_profile_id":"mpk.java.system_modules_25.v0",
            "toolchain_inputs_sha256":profile["toolchain_inputs"]["toolchain_inputs_sha256"]
        }),
        "source_map" => {
            json!({"encoding":"utf-8", "offset_unit":"utf8-byte", "synthetic_reasons":[]})
        }
        "vc" => {
            json!({"contract_profile_id":"mpk.java.contract.v0", "required_check_profile_id":"mpk.java.required_checks.v0", "verification_limit_profile_id":"mpk.verify.limits.v0"})
        }
        "vir" => {
            json!({"operation_profile_id":"mpk.java.operations.v0", "source_map_profile_id":"mpk.java.source_map.v0", "vir_limit_profile_id":"mpk.vir.limits.v0"})
        }
        other => panic!("unknown contract field {other}"),
    };
    json!({"profile_entry_sha256":ENTRY_HASH, "contract_id":format!("mpk.profile.{field}.java_scalar.v0"), "value":value})
}

// A finite oracle for these frozen fixtures. It is never called by production
// code and does not claim to replace later frontend/consumer mutation tests.
fn specimen_valid(profile: &Value, fixture: &str, value: &Value) -> bool {
    match fixture {
        "/semantic_parameters" => value == &expected_context()["semantic_parameters"],
        "/semantic_context_fixture" => value == &expected_context(),
        "/selection_fixture" => selection_specimen_valid(value),
        "/contract_fixture" => contract_specimen_valid(value),
        path if path.starts_with("/profile_contracts/") => {
            let index: usize = path.split('/').nth(2).unwrap().parse().unwrap();
            value == &expected_payload(profile, FIELDS[index])
        }
        other => panic!("unknown fixture owner {other}"),
    }
}

fn selection_specimen_valid(value: &Value) -> bool {
    if !has_keys(value, &["schema", "value"])
        || value["schema"] != "mpk.selection.java_methods.v0"
        || !has_keys(
            &value["value"],
            &["compilation", "contracts", "methods", "sources"],
        )
        || value["value"]["compilation"] != "payment-policy"
    {
        return false;
    }
    for (field, prefix, suffix, max) in [
        ("sources", "src/", ".java", 256),
        ("contracts", "contracts/", ".json", 128),
    ] {
        let Some(paths) = value["value"][field].as_array() else {
            return false;
        };
        if paths.is_empty() || paths.len() > max {
            return false;
        }
        let mut previous = "";
        for path in paths {
            let Some(path) = path.as_str() else {
                return false;
            };
            if path <= previous
                || !path.starts_with(prefix)
                || !path.ends_with(suffix)
                || !path.is_ascii()
                || path.len() > 1024
                || path.contains('\\')
                || path.split('/').any(|part| matches!(part, "" | "." | ".."))
            {
                return false;
            }
            previous = path;
        }
    }
    // This fixture has one selected method; the complete method-ID grammar is
    // exercised by the frontend-owned cases, not implemented in this oracle.
    value["value"]["methods"] == json!(["demo.Policy::approved(long,long)->boolean"])
        && canonical(value).len() <= 65_536
}

fn contract_specimen_valid(value: &Value) -> bool {
    if !has_keys(
        value,
        &[
            "schema",
            "semantic_profile",
            "method",
            "requires",
            "ensures",
            "modifies",
            "abrupt_completion",
            "termination",
        ],
    ) || value["schema"] != "mpk.java.contract.v0"
        || value["semantic_profile"] != "mpk.java.scalar.v0"
        || value["method"] != "demo.Policy::approved(long,long)->boolean"
        || value["modifies"] != json!([])
        || value["abrupt_completion"] != "forbidden"
        || value["termination"] != "total"
    {
        return false;
    }
    let (Some(requires), Some(ensures)) =
        (value["requires"].as_array(), value["ensures"].as_array())
    else {
        return false;
    };
    !ensures.is_empty()
        && requires.len() + ensures.len() <= 64
        && requires
            .iter()
            .all(|e| expression_type(e, false) == Some("boolean"))
        && ensures
            .iter()
            .all(|e| expression_type(e, true) == Some("boolean"))
}

fn expression_type(value: &Value, result_allowed: bool) -> Option<&'static str> {
    if has_keys(value, &["parameter"]) {
        return matches!(value["parameter"].as_str(), Some("reserve" | "debit")).then_some("i64");
    }
    if has_keys(value, &["result"]) {
        return (result_allowed && value["result"] == 0).then_some("boolean");
    }
    if has_keys(value, &["bool"]) {
        return value["bool"].is_boolean().then_some("boolean");
    }
    if has_keys(value, &["int"]) && has_keys(&value["int"], &["decimal", "type"]) {
        let decimal = value["int"]["decimal"].as_str()?;
        let parsed: i64 = decimal.parse().ok()?;
        if parsed.to_string() != decimal {
            return None;
        }
        return match value["int"]["type"].as_str()? {
            "i32" if i32::try_from(parsed).is_ok() => Some("i32"),
            "i64" => Some("i64"),
            _ => None,
        };
    }
    if !has_keys(value, &["op", "args"]) {
        return None;
    }
    let op = value["op"].as_str()?;
    let args = value["args"].as_array()?;
    let types = args
        .iter()
        .map(|e| expression_type(e, result_allowed))
        .collect::<Option<Vec<_>>>()?;
    let first = *types.first()?;
    if !types.iter().all(|ty| *ty == first) {
        return None;
    }
    match (op, types.len(), first) {
        ("not", 1, "boolean") => Some("boolean"),
        ("bv_neg" | "bv_not", 1, "i32" | "i64") => Some(first),
        ("and" | "or", 2..=64, "boolean") => Some("boolean"),
        ("eq" | "not_eq", 2, _) => Some("boolean"),
        ("signed_lt" | "signed_le" | "signed_gt" | "signed_ge", 2, "i32" | "i64") => {
            Some("boolean")
        }
        ("bv_add" | "bv_sub" | "bv_mul" | "bv_and" | "bv_or" | "bv_xor", 2, "i32" | "i64") => {
            Some(first)
        }
        _ => None,
    }
}

fn apply_mutation(value: &mut Value, case: &Value) {
    let pointer = text(&case["pointer"]);
    match text(&case["operation"]) {
        "replace" => {
            *value
                .pointer_mut(pointer)
                .expect("existing replacement target") = case["value"].clone()
        }
        "add" | "remove" => {
            let (parent, key) = pointer.rsplit_once('/').expect("member pointer");
            let object = value
                .pointer_mut(parent)
                .expect("existing parent")
                .as_object_mut()
                .expect("object parent");
            if case["operation"] == "add" {
                assert!(object
                    .insert(key.to_owned(), case["value"].clone())
                    .is_none());
            } else {
                assert!(object.remove(key).is_some());
            }
        }
        other => panic!("unknown mutation {other}"),
    }
}

fn assert_hash_case(root: &Value, case: &Value) {
    let source = root
        .pointer(text(&case["source_pointer"]))
        .expect("hash source pointer");
    let mut payload = source.clone();
    if let Some(excluded) = case["excluded_field"].as_str() {
        assert!(payload.as_object_mut().unwrap().remove(excluded).is_some());
    }
    let domain = text(&case["domain"]);
    let bytes = canonical(&payload);
    assert_eq!(
        bytes.len() as u64,
        number(&case["expected_payload_utf8_length"])
    );
    assert_eq!(
        (domain.len() + 1 + bytes.len()) as u64,
        number(&case["expected_preimage_length"])
    );
    assert_eq!(hash(domain, &payload), case["expected_sha256"]);
    if let Some(complete) = case.get("expected_complete_jcs_utf8_length") {
        assert_eq!(canonical(source).len() as u64, number(complete));
    }
    if let Some(transport) = case.get("expected_transport_utf8_length") {
        let mut bytes = canonical(source);
        bytes.push(b'\n');
        assert_eq!(bytes.len() as u64, number(transport));
        assert_eq!(
            sha256_raw_file_bytes(&bytes).to_hex(),
            case["expected_transport_sha256"]
        );
    }
}

fn domain(value: &str) -> HashDomain {
    HashDomain::new(match value {
        "MPK-SEMANTIC-PROFILE-ENTRY-1.0" => "MPK-SEMANTIC-PROFILE-ENTRY-1.0",
        "MPK-SEMANTIC-PROFILE-REGISTRY-1.0" => "MPK-SEMANTIC-PROFILE-REGISTRY-1.0",
        "MPK-JAVA-SELECTION-0.1" => "MPK-JAVA-SELECTION-0.1",
        "MPK-JAVA-CONTRACT-SIDECAR-0.1" => "MPK-JAVA-CONTRACT-SIDECAR-0.1",
        "MPK-JAVA-TOOLCHAIN-INPUTS-0.1" => "MPK-JAVA-TOOLCHAIN-INPUTS-0.1",
        "MPK-CONTRACT-1.0" => "MPK-CONTRACT-1.0",
        other => panic!("unknown domain {other}"),
    })
}

fn hash(domain_name: &str, value: &Value) -> String {
    hash_canonical_json(domain(domain_name), &strict(value))
        .unwrap()
        .to_hex()
}
fn hash_without(domain: &str, value: &Value, field: &str) -> String {
    let mut payload = value.clone();
    assert!(payload.as_object_mut().unwrap().remove(field).is_some());
    hash(domain, &payload)
}
fn strict(value: &Value) -> StrictJsonValue {
    parse_strict_json(&serde_json::to_vec(value).unwrap(), LIMITS).unwrap()
}
fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&strict(value)).unwrap()
}
fn load(bytes: &[u8]) -> Value {
    parse_strict_json(bytes, LIMITS).expect("strict specification vector transport");
    serde_json::from_slice(bytes).unwrap()
}
fn array(value: &Value) -> &Vec<Value> {
    value.as_array().expect("array")
}
fn text(value: &Value) -> &str {
    value.as_str().expect("string")
}
fn number(value: &Value) -> u64 {
    value.as_u64().expect("unsigned integer")
}
fn has_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.keys().map(String::as_str).collect::<BTreeSet<_>>() == keys.iter().copied().collect()
    })
}
fn exact_keys(value: &Value, keys: &[&str]) {
    assert!(has_keys(value, keys), "unexpected object fields: {value}");
}
