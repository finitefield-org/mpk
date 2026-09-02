use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, sha256_raw_file_bytes,
    HashDomain, StrictJsonLimits, StrictJsonValue,
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const PROFILE_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/csharp-profile-v0.json");
const REGISTRY_V2_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v2.json");
const REGISTRY_V1_BYTES: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const SPEC: &str = include_str!("../../../develop/specs/CSHARP_PROFILE_V0.md");

const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(4 * 1024 * 1024, 1_000_000, 128, 2 * 1024 * 1024);
const ENTRY_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-ENTRY-1.0");
const REGISTRY_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-REGISTRY-1.0");
const REFERENCE_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-REFERENCE-INVENTORY-0.1");
const TOOLCHAIN_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-TOOLCHAIN-INPUTS-0.1");
const SELECTION_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-SELECTION-0.1");
const SIDECAR_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-CONTRACT-SIDECAR-0.1");
const CONTRACT_DOMAIN: HashDomain = HashDomain::new("MPK-CONTRACT-1.0");

const CSHARP_ENTRY_HASH: &str = "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac";
const REGISTRY_V2_HASH: &str = "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75";
const REGISTRY_V3_HASH: &str = "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557";
const REGISTRY_V2_TRANSPORT_HASH: &str =
    "d3ccae252f388c21fbb3c400b58454c45d28943ae7d681d385a1dd4c017c0952";
const TOOLCHAIN_HASH: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";
const REFERENCE_HASH: &str = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
const SELECTION_HASH: &str = "d5033138bd8c53eee3901d0d1852ed4c1b1a85686cf2a68f01effb0b8c70dfcd";
const SIDECAR_HASH: &str = "6684361a15dc454a8172d7e515dd6a3a49ec1ff8faae00bc12d958eae8982228";
const CONTRACT_HASH: &str = "b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab";

#[test]
fn csharp_profile_identity_toolchain_and_payloads_are_frozen() {
    let profile = load(PROFILE_BYTES, "C# profile");
    assert_exact_keys(
        &profile,
        &[
            "schema",
            "owner_test",
            "spec_schema",
            "mechanism_schema",
            "profile_identity",
            "semantic_parameters",
            "selection_fixture",
            "selection_sha256",
            "contract_fixture",
            "contract_sidecar_sha256",
            "normalized_contract_fixture",
            "toolchain_inputs",
            "compiler_session",
            "launcher_contract",
            "case_harness",
            "source_map_cases",
            "profile_contracts",
            "type_mappings",
            "operation_mappings",
            "roslyn_checked_state_cases",
            "conversion_rules",
            "semantic_rows",
            "accepted_cases",
            "rejected_cases",
            "precedence_cases",
            "diagnostic_registry",
            "diagnostic_normalization",
            "limit_cases",
            "hash_cases",
            "isolation_cases",
            "upgrade_cases",
        ],
    );
    assert_eq!(
        text(field(&profile, "schema")),
        "mpk.csharp.profile.conformance.v0"
    );
    assert_eq!(
        text(field(&profile, "owner_test")),
        "crates/mpk-vc/tests/csharp_profile_spec.rs"
    );
    assert_eq!(text(field(&profile, "spec_schema")), "mpk.csharp.scalar.v0");
    assert_eq!(
        text(field(&profile, "mechanism_schema")),
        "mpk.semantic_profile.registry.v1"
    );

    let identity = field(&profile, "profile_identity");
    assert_exact_keys(
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
    assert_eq!(text(field(identity, "source_language")), "csharp");
    assert_eq!(
        text(field(identity, "semantic_profile")),
        "mpk.csharp.scalar.v0"
    );
    assert_eq!(
        text(field(identity, "profile_entry_sha256")),
        CSHARP_ENTRY_HASH
    );
    assert_eq!(integer(field(identity, "registry_revision")), 3);
    assert_eq!(text(field(identity, "registry_sha256")), REGISTRY_V3_HASH);

    assert_semantic_parameters(field(&profile, "semantic_parameters"));
    assert_selection(field(&profile, "selection_fixture"));
    assert_eq!(text(field(&profile, "selection_sha256")), SELECTION_HASH);
    assert_contract_sidecar(field(&profile, "contract_fixture"));
    assert_eq!(
        text(field(&profile, "contract_sidecar_sha256")),
        SIDECAR_HASH
    );

    let case_harness = field(&profile, "case_harness");
    assert_exact_keys(
        case_harness,
        &[
            "baseline_files",
            "baseline_selection",
            "default_contract_template",
            "operation_projection",
            "accepted_case_materialization",
            "application_order",
        ],
    );
    let baseline_files = field(case_harness, "baseline_files");
    let source = text(field(baseline_files, "src/Policy.cs"));
    assert!(source.ends_with('\n'));
    assert!(!source.contains('\r'));
    assert!(!source.contains("\\n"));
    assert!(source.contains("return reserve >= debit;"));
    assert_eq!(
        field(baseline_files, "contracts/approved.json"),
        field(&profile, "contract_fixture")
    );
    let default_contract = field(case_harness, "default_contract_template");
    assert_eq!(
        text(field(default_contract, "schema")),
        "mpk.csharp.contract.v0"
    );
    assert!(array(field(default_contract, "requires")).is_empty());
    assert_eq!(array(field(default_contract, "ensures")).len(), 1);
    assert!(text(field(case_harness, "operation_projection"))
        .contains("expected_required_checks is exhaustive"));

    let toolchain = field(&profile, "toolchain_inputs");
    assert_exact_keys(
        toolchain,
        &[
            "schema",
            "id",
            "host",
            "roslyn_source",
            "archives",
            "package_graph",
            "managed_projection",
            "reference_projection",
            "toolchain_inputs_sha256",
        ],
    );
    assert_eq!(
        text(field(toolchain, "schema")),
        "mpk.csharp.toolchain_inputs.v0"
    );
    assert_eq!(
        text(field(toolchain, "toolchain_inputs_sha256")),
        TOOLCHAIN_HASH
    );
    let host = field(toolchain, "host");
    assert_exact_keys(
        host,
        &[
            "architecture",
            "os",
            "rid",
            "execution_host_profile_id",
            "runtime_layout_profile_id",
            "minimum_kernel_abi",
            "interpreter",
            "native_library_roots",
        ],
    );
    assert_eq!(
        text(field(host, "execution_host_profile_id")),
        "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
    );
    assert_eq!(
        text(field(host, "runtime_layout_profile_id")),
        "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
    );
    assert_eq!(text(field(host, "minimum_kernel_abi")), "6.4.0");
    assert_eq!(array(field(host, "native_library_roots")).len(), 2);
    assert_eq!(
        text(field(field(toolchain, "roslyn_source"), "commit")),
        "c0573ed0a7dc3e3b4d2e70da47f97cc51a35524f"
    );
    assert_archives(field(toolchain, "archives"));
    assert_eq!(array(field(toolchain, "package_graph")).len(), 4);
    assert_eq!(array(field(toolchain, "managed_projection")).len(), 2);
    assert_reference_projection(field(toolchain, "reference_projection"));
    assert_compiler_session(field(&profile, "compiler_session"));
    assert_launcher_contract(field(&profile, "launcher_contract"));
    assert_source_map_cases(field(&profile, "source_map_cases"));
    assert_profile_contracts(field(&profile, "profile_contracts"));
}

#[test]
fn csharp_semantic_partition_and_case_ownership_are_closed() {
    let profile = load(PROFILE_BYTES, "C# profile");

    let mappings = array(field(&profile, "type_mappings"));
    assert_eq!(mappings.len(), 5);
    let source_types = mappings
        .iter()
        .map(|mapping| text(field(mapping, "source_type")))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        source_types,
        BTreeSet::from(["bool", "int", "uint", "long", "ulong"])
    );
    for mapping in mappings {
        assert_exact_keys(mapping, &["source_type", "method_id_token", "vir_type"]);
        let source_type = text(field(mapping, "source_type"));
        let vir_type = field(mapping, "vir_type");
        if source_type == "bool" {
            assert_exact_keys(vir_type, &["kind"]);
            assert_eq!(text(field(vir_type, "kind")), "bool");
        } else {
            assert_exact_keys(vir_type, &["kind", "width", "signed"]);
            assert_eq!(text(field(vir_type, "kind")), "bv");
            assert!([32, 64].contains(&integer(field(vir_type, "width"))));
        }
    }

    let operations = array(field(&profile, "operation_mappings"));
    assert_eq!(operations.len(), 35);
    let operation_keys = operations
        .iter()
        .map(|operation| {
            format!(
                "{}\t{}\t{}\t{}",
                text(field(operation, "source")),
                text(field(operation, "context")),
                array(field(operation, "operand_types"))
                    .iter()
                    .map(text)
                    .collect::<Vec<_>>()
                    .join(","),
                array(field(operation, "vir"))
                    .iter()
                    .map(text)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<BTreeSet<_>>();
    let expected_operation_keys = BTreeSet::from([
        "!\tany\tbool\tbool_not",
        "==\tany\tbool,int,uint,long,ulong\teq",
        "!=\tany\tbool,int,uint,long,ulong\tnot_eq",
        "<\tany\tint,long\tsigned_lt",
        "<=\tany\tint,long\tsigned_le",
        ">\tany\tint,long\tsigned_gt",
        ">=\tany\tint,long\tsigned_ge",
        "<\tany\tuint,ulong\tunsigned_lt",
        "<=\tany\tuint,ulong\tunsigned_le",
        ">\tany\tuint,ulong\tunsigned_gt",
        ">=\tany\tuint,ulong\tunsigned_ge",
        "+\tchecked\tint,uint,long,ulong\tbv_add",
        "-\tchecked\tint,uint,long,ulong\tbv_sub",
        "*\tchecked\tint,uint,long,ulong\tbv_mul",
        "+\tunchecked\tint,uint,long,ulong\tbv_add",
        "-\tunchecked\tint,uint,long,ulong\tbv_sub",
        "*\tunchecked\tint,uint,long,ulong\tbv_mul",
        "unary-\tchecked\tint,long\tbv_neg",
        "unary-\tunchecked\tint,long\tbv_neg",
        "/\texplicit_checked_or_unchecked\tint,long\tbv_sdiv",
        "%\texplicit_checked_or_unchecked\tint,long\tbv_srem",
        "/\texplicit_checked_or_unchecked\tuint,ulong\tbv_udiv",
        "%\texplicit_checked_or_unchecked\tuint,ulong\tbv_urem",
        "~\tany\tint,uint,long,ulong\tbv_not",
        "&\tany\tint,uint,long,ulong\tbv_and",
        "|\tany\tint,uint,long,ulong\tbv_or",
        "^\tany\tint,uint,long,ulong\tbv_xor",
        "<<\tany\tint,uint\tbv_and(count,31),bv_shl",
        "<<\tany\tlong,ulong\tbv_and(count,63),bv_shl",
        ">>\tany\tint,long\tbv_and(count,width-1),bv_ashr",
        ">>\tany\tuint,ulong\tbv_and(count,width-1),bv_lshr",
        "&&\tany\tbool\tBranch,block_parameter",
        "||\tany\tbool\tBranch,block_parameter",
        "?:\tany\tbool_condition,identical_accepted_branch_type\tBranch,Jump,block_parameter",
        "direct_static_call\tany\texact_signature\tCallStatic",
    ])
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(operation_keys, expected_operation_keys);
    let allowed_checks = BTreeSet::from([
        "integer_no_overflow",
        "divisor_nonzero",
        "signed_divrem_representable",
        "callee_contract_hash",
    ]);
    for operation in operations {
        let source = text(field(operation, "source"));
        let context = text(field(operation, "context"));
        for check in array(field(operation, "checks")) {
            assert!(
                allowed_checks.contains(text(check)),
                "unknown check in {source}"
            );
        }
        if ["+", "-", "*", "unary-"].contains(&source) {
            assert!(matches!(context, "checked" | "unchecked"));
        }
        if ["/", "%"].contains(&source) {
            assert_eq!(context, "explicit_checked_or_unchecked");
            assert_eq!(
                text(
                    array(field(operation, "checks"))
                        .first()
                        .expect("division check")
                ),
                "divisor_nonzero"
            );
        }
    }
    assert!(operations.iter().any(|operation| {
        text(field(operation, "source")) == "/"
            && array(field(operation, "checks")).len() == 2
            && array(field(operation, "operand_types"))
                == [Value::String("int".into()), Value::String("long".into())]
    }));

    let checked_states = array(field(&profile, "roslyn_checked_state_cases"));
    assert_eq!(checked_states.len(), 12);
    let mut checked_state_ids = BTreeSet::new();
    for case in checked_states {
        assert_exact_keys(
            case,
            &[
                "id",
                "source",
                "operator_kind",
                "operand_types",
                "context",
                "expected_is_checked",
            ],
        );
        checked_state_ids.insert(text(field(case, "id")));
        let source = text(field(case, "source"));
        let expected_kind = match source {
            "+" => "Add",
            "-" => "Subtract",
            "*" => "Multiply",
            "unary-" => "Minus",
            "/" => "Divide",
            "%" => "Remainder",
            _ => panic!("unknown checked-state source operator {source}"),
        };
        assert_eq!(text(field(case, "operator_kind")), expected_kind);
        let context = text(field(case, "context"));
        assert!(matches!(context, "checked" | "unchecked"));
        assert_eq!(
            boolean(field(case, "expected_is_checked")),
            context == "checked" && source != "%"
        );
        let expected_types = if source == "unary-" {
            ["int", "long"].as_slice()
        } else {
            ["int", "uint", "long", "ulong"].as_slice()
        };
        assert_eq!(
            array(field(case, "operand_types"))
                .iter()
                .map(text)
                .collect::<Vec<_>>(),
            expected_types
        );
    }
    assert_eq!(
        checked_state_ids,
        BTreeSet::from([
            "checked.add",
            "checked.subtract",
            "checked.multiply",
            "checked.negate",
            "checked.divide",
            "checked.remainder",
            "unchecked.add",
            "unchecked.subtract",
            "unchecked.multiply",
            "unchecked.negate",
            "unchecked.divide",
            "unchecked.remainder",
        ])
    );

    let conversions = array(field(&profile, "conversion_rules"));
    assert_eq!(conversions.len(), 20);
    let mut pairs = BTreeSet::new();
    let mut forms = BTreeMap::<&str, usize>::new();
    for conversion in conversions {
        assert_exact_keys(
            conversion,
            &[
                "source_type",
                "destination_type",
                "source_form",
                "context",
                "vir",
                "checks",
            ],
        );
        assert!(pairs.insert(format!(
            "{}->{}:{}",
            text(field(conversion, "source_type")),
            text(field(conversion, "destination_type")),
            text(field(conversion, "source_form"))
        )));
        let form = text(field(conversion, "source_form"));
        *forms.entry(form).or_default() += 1;
        assert!(array(field(conversion, "checks")).is_empty());
        if form == "explicit" {
            assert_eq!(text(field(conversion, "context")), "unchecked");
            assert_eq!(
                array(field(conversion, "vir")),
                [Value::String("Convert".into())]
            );
        }
    }
    assert_eq!(
        pairs,
        BTreeSet::from([
            "bool->bool:identity".to_owned(),
            "int->int:identity".to_owned(),
            "int->uint:explicit".to_owned(),
            "int->long:implicit".to_owned(),
            "int->long:explicit".to_owned(),
            "int->ulong:explicit".to_owned(),
            "uint->int:explicit".to_owned(),
            "uint->uint:identity".to_owned(),
            "uint->long:implicit".to_owned(),
            "uint->long:explicit".to_owned(),
            "uint->ulong:implicit".to_owned(),
            "uint->ulong:explicit".to_owned(),
            "long->int:explicit".to_owned(),
            "long->uint:explicit".to_owned(),
            "long->long:identity".to_owned(),
            "long->ulong:explicit".to_owned(),
            "ulong->int:explicit".to_owned(),
            "ulong->uint:explicit".to_owned(),
            "ulong->long:explicit".to_owned(),
            "ulong->ulong:identity".to_owned(),
        ])
    );
    assert_eq!(
        forms,
        BTreeMap::from([("explicit", 12), ("identity", 5), ("implicit", 3)])
    );

    let rows = array(field(&profile, "semantic_rows"));
    assert_eq!(rows.len(), 34);
    let accepted_expected = BTreeSet::from([
        "M01", "M02", "M07", "M08", "M09", "M10", "M11", "M12", "M13", "M14", "M16", "M18", "M19",
        "M21", "M27", "M29", "M33", "M34",
    ]);
    let mut accepted_actual = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        assert_exact_keys(row, &["row", "disposition", "basis"]);
        assert_eq!(text(field(row, "row")), format!("M{:02}", index + 1));
        match text(field(row, "disposition")) {
            "accept_under_profile_restrictions" => {
                accepted_actual.insert(text(field(row, "row")));
            }
            "reject_before_vir" => {}
            disposition => panic!("unknown semantic disposition {disposition}"),
        }
    }
    assert_eq!(accepted_actual, accepted_expected);

    assert_case_and_diagnostic_closure(&profile);
    assert_diagnostic_normalization(field(&profile, "diagnostic_normalization"));
    assert_limit_cases(field(&profile, "limit_cases"));
    assert_eq!(array(field(&profile, "isolation_cases")).len(), 12);
    assert_eq!(array(field(&profile, "upgrade_cases")).len(), 12);
}

#[test]
fn csharp_profile_and_revision_two_hashes_recompute() {
    let profile = load(PROFILE_BYTES, "C# profile");
    for case in array(field(&profile, "hash_cases")) {
        assert_profile_hash_case(&profile, case);
    }
    assert_eq!(
        text(field(
            field(&profile, "toolchain_inputs"),
            "toolchain_inputs_sha256"
        )),
        TOOLCHAIN_HASH
    );
    assert_eq!(text(field(&profile, "selection_sha256")), SELECTION_HASH);
    assert_eq!(
        text(field(&profile, "contract_sidecar_sha256")),
        SIDECAR_HASH
    );
    assert_eq!(
        text(field(
            field(&profile, "normalized_contract_fixture"),
            "contract_hash"
        )),
        CONTRACT_HASH
    );

    let vectors = load(REGISTRY_V2_BYTES, "semantic profile registry revision 2");
    assert_exact_keys(
        &vectors,
        &[
            "schema",
            "owner_test",
            "mechanism_spec",
            "profile_spec",
            "predecessor",
            "csharp_entry",
            "registry",
            "hash_cases",
            "append_only_cases",
            "activation_cases",
        ],
    );
    assert_eq!(
        text(field(&vectors, "schema")),
        "mpk.semantic_profile.registry.conformance.v2"
    );
    assert_eq!(
        text(field(&vectors, "owner_test")),
        "crates/mpk-vc/tests/csharp_profile_spec.rs"
    );

    let revision_one_vectors = load(REGISTRY_V1_BYTES, "registry revision 1");
    let revision_one = field(field(&revision_one_vectors, "fixtures"), "base_registry");
    assert_eq!(
        canonical(field(&vectors, "predecessor")),
        canonical(revision_one)
    );

    let registry = field(&vectors, "registry");
    assert_exact_keys(
        registry,
        &["schema", "id", "revision", "profiles", "registry_sha256"],
    );
    assert_eq!(integer(field(registry, "revision")), 2);
    assert_eq!(text(field(registry, "registry_sha256")), REGISTRY_V2_HASH);
    let profiles = array(field(registry, "profiles"));
    assert_eq!(profiles.len(), 3);
    assert_eq!(&profiles[0], field(&vectors, "csharp_entry"));
    assert_eq!(
        profiles
            .iter()
            .map(|entry| text(field(entry, "source_language")))
            .collect::<Vec<_>>(),
        ["csharp", "go", "rust"]
    );
    assert_eq!(
        canonical(&profiles[1]),
        canonical(&array(field(revision_one, "profiles"))[0])
    );
    assert_eq!(
        canonical(&profiles[2]),
        canonical(&array(field(revision_one, "profiles"))[1])
    );
    assert_eq!(text(field(&profiles[0], "entry_sha256")), CSHARP_ENTRY_HASH);
    assert_eq!(
        object(field(&profiles[0], "contracts"))
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "ai",
            "evidence",
            "frontend",
            "manifest",
            "policy",
            "release",
            "source_map",
            "vc",
            "vir",
        ])
    );
    for absent in ["dart", "java", "typescript", "python"] {
        assert!(!profiles
            .iter()
            .any(|entry| text(field(entry, "source_language")) == absent));
    }
    for case in array(field(&vectors, "hash_cases")) {
        assert_registry_hash_case(registry, case);
    }
    assert_eq!(array(field(&vectors, "append_only_cases")).len(), 8);
    assert_eq!(array(field(&vectors, "activation_cases")).len(), 8);
    let membership = array(field(&vectors, "activation_cases"))
        .iter()
        .find(|case| text(field(case, "id")) == "activation.membership_is_inactive")
        .expect("membership activation case");
    assert!(!boolean(field(membership, "active")));
}

#[test]
fn csharp_normative_spec_is_resolved_and_inactive() {
    assert!(!SPEC.contains("__"));
    for required in [
        "MLANG-01-T03",
        "mpk.csharp.scalar.v0",
        "LanguageVersion.CSharp14",
        "Microsoft.CodeAnalysis.CSharp",
        ".NET runtime execution closure",
        "Microsoft.NETCore.App.Ref",
        "CanonicalDecimal",
        "unreachable source method rejects",
        "Line and column values are never serialized",
        "UTF-16 code units since",
        CSHARP_ENTRY_HASH,
        REGISTRY_V2_HASH,
        TOOLCHAIN_HASH,
        "No current MPK",
        "C# remains inactive",
        "adds no executable",
    ] {
        assert!(SPEC.contains(required), "spec is missing {required:?}");
    }
}
fn assert_semantic_parameters(envelope: &Value) {
    assert_exact_keys(envelope, &["schema", "value"]);
    assert_eq!(
        text(field(envelope, "schema")),
        "mpk.semantic_parameters.csharp_scalar.v0"
    );
    let value = field(envelope, "value");
    assert_exact_keys(
        value,
        &[
            "check_overflow_default",
            "documentation_mode",
            "language_version",
            "nullable_context",
            "optimization",
            "platform",
            "pointer_width",
            "preprocessor_symbols",
            "source_kind",
            "target_framework",
            "target_id",
            "unsafe",
        ],
    );
    assert!(!boolean(field(value, "check_overflow_default")));
    assert_eq!(text(field(value, "language_version")), "14.0");
    assert_eq!(text(field(value, "target_framework")), "net10.0");
    assert_eq!(text(field(value, "target_id")), "linux-x64");
    assert_eq!(integer(field(value, "pointer_width")), 64);
    assert!(array(field(value, "preprocessor_symbols")).is_empty());
    assert!(!boolean(field(value, "unsafe")));
}

fn assert_selection(selection: &Value) {
    assert_exact_keys(selection, &["schema", "value"]);
    assert_eq!(
        text(field(selection, "schema")),
        "mpk.selection.csharp_methods.v0"
    );
    let value = field(selection, "value");
    assert_exact_keys(value, &["compilation", "contracts", "methods", "sources"]);
    assert_eq!(text(field(value, "compilation")), "payment-policy");
    for name in ["sources", "contracts", "methods"] {
        assert_sorted_unique_strings(field(value, name));
    }
    assert_eq!(text(&array(field(value, "sources"))[0]), "src/Policy.cs");
    assert_eq!(
        text(&array(field(value, "contracts"))[0]),
        "contracts/approved.json"
    );
    assert_eq!(
        text(&array(field(value, "methods"))[0]),
        "Example.Payment.Policy::Approved(i64,i64)->bool"
    );
}

fn assert_contract_sidecar(contract: &Value) {
    assert_exact_keys(
        contract,
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
    );
    assert_eq!(text(field(contract, "schema")), "mpk.csharp.contract.v0");
    assert_eq!(
        text(field(contract, "semantic_profile")),
        "mpk.csharp.scalar.v0"
    );
    assert_eq!(array(field(contract, "requires")).len(), 2);
    assert_eq!(array(field(contract, "ensures")).len(), 1);
    assert!(array(field(contract, "modifies")).is_empty());
    assert_eq!(text(field(contract, "abrupt_completion")), "forbidden");
    assert_eq!(text(field(contract, "termination")), "total");
}

fn assert_archives(value: &Value) {
    let archives = array(value);
    assert_eq!(archives.len(), 6);
    let ids = archives
        .iter()
        .map(|archive| text(field(archive, "id")))
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        ids,
        [
            "dotnet-runtime-linux-x64",
            "dotnet-sdk-linux-x64",
            "microsoft-codeanalysis-analyzers",
            "microsoft-codeanalysis-common",
            "microsoft-codeanalysis-csharp",
            "microsoft-netcore-app-ref",
        ]
    );
    for archive in archives {
        let expected_keys: &[&str] = if text(field(archive, "kind")) == "tar.gz" {
            &[
                "id",
                "kind",
                "version",
                "url",
                "size_bytes",
                "sha256",
                "sha512",
            ]
        } else {
            &["id", "kind", "version", "url", "size_bytes", "sha256"]
        };
        assert_exact_keys(archive, expected_keys);
        assert!(integer(field(archive, "size_bytes")) > 0);
        assert_lower_hex(text(field(archive, "sha256")), 64);
        if let Some(sha512) = archive.get("sha512") {
            assert_lower_hex(text(sha512), 128);
        }
        assert!(text(field(archive, "url")).starts_with("https://"));
    }
}

fn assert_reference_projection(projection: &Value) {
    assert_exact_keys(
        projection,
        &[
            "package_id",
            "version",
            "selector",
            "install_root",
            "count",
            "total_bytes",
            "canonical_payload_bytes",
            "hash_domain",
            "inventory_sha256",
            "metadata",
            "inventory",
        ],
    );
    assert_eq!(integer(field(projection, "count")), 167);
    assert_eq!(
        text(field(projection, "install_root")),
        "/mpk/toolchain/reference-pack"
    );
    assert_eq!(integer(field(projection, "total_bytes")), 6_046_008);
    assert_eq!(
        integer(field(projection, "canonical_payload_bytes")),
        24_670
    );
    assert_eq!(text(field(projection, "inventory_sha256")), REFERENCE_HASH);
    assert_eq!(array(field(projection, "metadata")).len(), 3);

    let inventory = array(field(projection, "inventory"));
    assert_eq!(inventory.len(), 167);
    let mut previous = None;
    let mut total = 0_i64;
    for record in inventory {
        assert_exact_keys(record, &["path", "size_bytes", "sha256"]);
        let path = text(field(record, "path"));
        assert!(path.starts_with("ref/net10.0/"));
        assert!(path.ends_with(".dll"));
        assert_eq!(path.matches('/').count(), 2);
        if let Some(previous) = previous {
            assert!(
                previous < path,
                "reference inventory is not strictly sorted"
            );
        }
        previous = Some(path);
        total += integer(field(record, "size_bytes"));
        assert_lower_hex(text(field(record, "sha256")), 64);
    }
    assert_eq!(total, 6_046_008);
}

fn assert_compiler_session(session: &Value) {
    assert_exact_keys(
        session,
        &[
            "source_text",
            "parse_options",
            "syntax_tree_order",
            "cfg_creation",
            "semantic_api_options",
            "compilation_options",
            "public_api_families",
            "diagnostics",
        ],
    );
    let source_text = field(session, "source_text");
    assert_exact_keys(
        source_text,
        &[
            "decode",
            "encoding",
            "checksum_algorithm",
            "source_text_overload",
            "parse_text_overload",
        ],
    );
    assert_eq!(
        text(field(source_text, "encoding")),
        "new UTF8Encoding(false,true)"
    );
    assert_eq!(text(field(source_text, "checksum_algorithm")), "Sha256");
    let parse = field(session, "parse_options");
    assert_eq!(text(field(parse, "language_version_enum")), "CSharp14");
    assert_eq!(text(field(parse, "source_kind")), "Regular");
    assert!(array(field(parse, "preprocessor_symbols")).is_empty());
    assert_eq!(
        text(field(session, "syntax_tree_order")),
        "selection.value.sources stored order"
    );
    let cfg = field(session, "cfg_creation");
    assert_exact_keys(cfg, &["operation_root", "overload", "cancellation_token"]);
    assert_eq!(
        text(field(cfg, "operation_root")),
        "IMethodBodyOperation from exact MethodDeclarationSyntax"
    );
    assert_eq!(text(field(cfg, "cancellation_token")), "None");
    let semantic_api = field(session, "semantic_api_options");
    assert_exact_keys(
        semantic_api,
        &[
            "cancellation_token",
            "ignore_accessibility",
            "speculative_models",
        ],
    );
    assert_eq!(text(field(semantic_api, "cancellation_token")), "None");
    assert!(!boolean(field(semantic_api, "ignore_accessibility")));
    assert!(!boolean(field(semantic_api, "speculative_models")));

    let compilation = field(session, "compilation_options");
    assert!(!boolean(field(compilation, "check_overflow")));
    assert!(!boolean(field(compilation, "allow_unsafe")));
    assert!(boolean(field(compilation, "deterministic")));
    assert!(!boolean(field(compilation, "concurrent_build")));
    assert_eq!(
        text(field(compilation, "general_diagnostic_option")),
        "Error"
    );
    assert_eq!(integer(field(compilation, "warning_level")), 4);
    assert!(!boolean(field(
        compilation,
        "report_suppressed_diagnostics"
    )));
    assert!(!boolean(field(
        compilation,
        "references_supersede_lower_versions"
    )));
    assert_eq!(
        text(field(compilation, "assembly_identity_comparer")),
        "Default"
    );
    let reference = field(compilation, "metadata_reference_properties");
    assert_exact_keys(
        reference,
        &[
            "kind",
            "aliases",
            "embed_interop_types",
            "documentation_provider",
        ],
    );
    assert_eq!(text(field(reference, "kind")), "Assembly");
    assert!(array(field(reference, "aliases")).is_empty());
    assert!(!boolean(field(reference, "embed_interop_types")));
    let public_apis = array(field(session, "public_api_families"));
    assert_eq!(public_apis.len(), 13);
    assert!(public_apis
        .iter()
        .any(|api| text(api) == "SyntaxTree.GetDiagnostics"));
    let diagnostics = field(session, "diagnostics");
    assert_exact_keys(
        diagnostics,
        &[
            "warnings",
            "errors",
            "syntax_tree_active_owner",
            "compilation_active_owner",
            "compilation_diagnostics_after_clean_syntax_only",
            "informational_and_hidden",
        ],
    );
    assert_eq!(
        text(field(diagnostics, "syntax_tree_active_owner")),
        "source/CSHARP_SOURCE_PARSE"
    );
    assert_eq!(
        text(field(diagnostics, "compilation_active_owner")),
        "metadata/CSHARP_SOURCE_DIAGNOSTIC"
    );
    assert!(boolean(field(
        diagnostics,
        "compilation_diagnostics_after_clean_syntax_only"
    )));
}
fn assert_launcher_contract(launcher: &Value) {
    assert_exact_keys(
        launcher,
        &[
            "profile_id",
            "program",
            "working_directory",
            "stdin",
            "stdout",
            "stderr",
            "runtime_config",
            "argv_prefix",
            "frontend_argv_template",
            "repeated_argument_expansion",
            "environment",
            "inherited_environment",
        ],
    );
    assert_eq!(
        text(field(launcher, "profile_id")),
        "mpk.csharp.dotnet_launcher.v0"
    );
    assert_eq!(
        text(field(launcher, "program")),
        "/mpk/toolchain/dotnet/dotnet"
    );
    assert_eq!(text(field(launcher, "working_directory")), "/mpk/source");
    assert_eq!(text(field(launcher, "stdin")), "null");
    assert!(array(field(launcher, "inherited_environment")).is_empty());

    let runtime = field(launcher, "runtime_config");
    assert_exact_keys(
        runtime,
        &["tfm", "framework_name", "framework_version", "roll_forward"],
    );
    assert_eq!(text(field(runtime, "tfm")), "net10.0");
    assert_eq!(text(field(runtime, "framework_version")), "10.0.11");
    assert_eq!(text(field(runtime, "roll_forward")), "Disable");

    let prefix = array(field(launcher, "argv_prefix"));
    assert_eq!(
        prefix.first().map(text),
        Some("/mpk/toolchain/dotnet/dotnet")
    );
    assert_eq!(
        prefix.last().map(text),
        Some("/mpk/frontend/csharp2vir.dll")
    );
    let arguments = array(field(launcher, "frontend_argv_template"))
        .iter()
        .map(text)
        .collect::<Vec<_>>();
    for required in [
        "--compilation",
        "--source",
        "--contract",
        "--method",
        "--profile-registry-id",
        "--profile-registry-revision",
        "--profile-registry-sha256",
        "--profile-entry-sha256",
        "--toolchain-distribution-sha256",
    ] {
        assert!(arguments.contains(&required));
    }
    assert!(arguments.contains(&REGISTRY_V3_HASH));
    assert!(arguments.contains(&CSHARP_ENTRY_HASH));

    let environment = object(field(launcher, "environment"));
    assert_eq!(environment.len(), 18);
    assert_eq!(
        text(environment.get("DOTNET_ROOT").expect("DOTNET_ROOT")),
        "/mpk/toolchain/dotnet"
    );
    assert_eq!(
        text(
            environment
                .get("DOTNET_MULTILEVEL_LOOKUP")
                .expect("multilevel")
        ),
        "0"
    );
    assert_eq!(text(environment.get("PATH").expect("PATH")), "/nonexistent");
}

fn assert_source_map_cases(value: &Value) {
    let cases = array(value);
    assert_eq!(cases.len(), 6);
    let mut ids = BTreeSet::new();
    for case in cases {
        assert_exact_keys(
            case,
            &["id", "source", "utf16_start", "utf16_end", "expect"],
        );
        assert!(ids.insert(text(field(case, "id"))));
        let source = text(field(case, "source"));
        assert!(source.ends_with('\n'));
        let start = integer(field(case, "utf16_start"));
        let end = integer(field(case, "utf16_end"));
        assert!(start >= 0 && end >= start);
        let expectation = field(case, "expect");
        match text(field(expectation, "outcome")) {
            "accept" => {
                assert_exact_keys(
                    expectation,
                    &[
                        "outcome",
                        "utf8_start",
                        "utf8_end",
                        "line_start",
                        "column_start_utf16",
                        "line_end",
                        "column_end_utf16",
                    ],
                );
                assert!(
                    integer(field(expectation, "utf8_end"))
                        > integer(field(expectation, "utf8_start"))
                );
                let (start_byte, start_line, start_column) = utf16_position(source, start);
                let (end_byte, end_line, end_column) = utf16_position(source, end);
                assert_eq!(integer(field(expectation, "utf8_start")), start_byte);
                assert_eq!(integer(field(expectation, "utf8_end")), end_byte);
                assert_eq!(integer(field(expectation, "line_start")), start_line);
                assert_eq!(integer(field(expectation, "line_end")), end_line);
                assert_eq!(
                    integer(field(expectation, "column_start_utf16")),
                    start_column
                );
                assert_eq!(integer(field(expectation, "column_end_utf16")), end_column);
            }
            "reject" => {
                assert_exact_keys(expectation, &["outcome", "code"]);
                assert!(matches!(
                    text(field(expectation, "code")),
                    "CSHARP_SOURCE_MAP_UTF16" | "CSHARP_SOURCE_MAP_RANGE"
                ));
            }
            outcome => panic!("unknown source-map outcome {outcome}"),
        }
    }
    assert_eq!(
        ids,
        BTreeSet::from([
            "map.ascii",
            "map.bmp",
            "map.surrogate_pair",
            "map.reject_surrogate_split",
            "map.reject_zero_length",
            "map.reject_out_of_range",
        ])
    );
    let surrogate = cases
        .iter()
        .find(|case| text(field(case, "id")) == "map.surrogate_pair")
        .expect("surrogate mapping case");
    assert_eq!(integer(field(field(surrogate, "expect"), "utf8_end")), 7);
}

fn utf16_position(source: &str, offset: i64) -> (i64, i64, i64) {
    let mut units = 0_i64;
    let mut bytes = 0_i64;
    let mut line = 0_i64;
    let mut column = 0_i64;
    if offset == 0 {
        return (bytes, line, column);
    }
    for character in source.chars() {
        let width = character.len_utf16() as i64;
        assert!(units + width <= offset, "offset splits a surrogate pair");
        units += width;
        bytes += character.len_utf8() as i64;
        if character == '\n' {
            line += 1;
            column = 0;
        } else {
            column += width;
        }
        if units == offset {
            return (bytes, line, column);
        }
    }
    panic!("UTF-16 offset is outside source");
}

fn assert_profile_contracts(value: &Value) {
    let contracts = array(value);
    assert_eq!(contracts.len(), 9);
    let expected = [
        ("ai", "mpk.profile.ai.csharp_scalar.v0"),
        ("evidence", "mpk.profile.evidence.csharp_scalar.v0"),
        ("frontend", "mpk.profile.frontend.csharp_scalar.v0"),
        ("manifest", "mpk.profile.manifest.csharp_scalar.v0"),
        ("policy", "mpk.profile.policy.csharp_scalar.v0"),
        ("release", "mpk.profile.release.csharp_scalar.v0"),
        ("source_map", "mpk.profile.source_map.csharp_scalar.v0"),
        ("vc", "mpk.profile.vc.csharp_scalar.v0"),
        ("vir", "mpk.profile.vir.csharp_scalar.v0"),
    ];
    for (record, (field_name, contract_id)) in contracts.iter().zip(expected) {
        assert_exact_keys(record, &["field", "envelope"]);
        assert_eq!(text(field(record, "field")), field_name);
        let envelope = field(record, "envelope");
        assert_exact_keys(envelope, &["profile_entry_sha256", "contract_id", "value"]);
        assert_eq!(
            text(field(envelope, "profile_entry_sha256")),
            CSHARP_ENTRY_HASH
        );
        assert_eq!(text(field(envelope, "contract_id")), contract_id);
        assert!(field(envelope, "value").is_object());
        assert_no_executable_fields(field(envelope, "value"));
    }
    let release = field(field(&contracts[5], "envelope"), "value");
    assert_exact_keys(
        release,
        &[
            "compiler_profile_id",
            "execution_host_profile_id",
            "reference_profile_id",
            "runtime_layout_profile_id",
            "runtime_profile_id",
            "toolchain_inputs_sha256",
        ],
    );
    assert_eq!(
        text(field(release, "execution_host_profile_id")),
        "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
    );
    assert_eq!(
        text(field(release, "runtime_layout_profile_id")),
        "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
    );
    assert_eq!(
        text(field(release, "toolchain_inputs_sha256")),
        TOOLCHAIN_HASH
    );
}

fn assert_no_executable_fields(value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                assert!(
                    ![
                        "validator",
                        "callback",
                        "plugin",
                        "executable",
                        "command",
                        "library",
                        "uri",
                        "url",
                        "code",
                    ]
                    .contains(&key.as_str()),
                    "compiled payload contains executable field {key}"
                );
                assert_no_executable_fields(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_no_executable_fields(child);
            }
        }
        _ => {}
    }
}

fn assert_case_and_diagnostic_closure(profile: &Value) {
    let diagnostics = array(field(profile, "diagnostic_registry"));
    assert_eq!(diagnostics.len(), 44);
    let diagnostic_owners = diagnostics
        .iter()
        .map(|diagnostic| {
            (
                text(field(diagnostic, "code")),
                (
                    text(field(diagnostic, "status")),
                    text(field(diagnostic, "phase")),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let diagnostic_codes = diagnostics
        .iter()
        .map(|diagnostic| {
            assert_exact_keys(diagnostic, &["code", "status", "phase"]);
            text(field(diagnostic, "code"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(diagnostic_codes.len(), 44);

    let accepted = array(field(profile, "accepted_cases"));
    let rejected = array(field(profile, "rejected_cases"));
    assert_eq!(accepted.len(), 30);
    assert_eq!(rejected.len(), 88);
    let mut ids = BTreeSet::new();
    let mut directly_covered_diagnostics = BTreeSet::new();
    for case in accepted {
        assert_exact_keys(
            case,
            &[
                "id",
                "stage",
                "construction",
                "expect",
                "source",
                "method",
                "expected_profile_operations",
                "expected_required_checks",
            ],
        );
        assert!(ids.insert(text(field(case, "id"))));
        let source = text(field(case, "source"));
        assert!(source.ends_with('\n'));
        assert!(!source.contains('\r'));
        assert!(text(field(case, "method")).contains("::"));
        assert!(field(case, "expected_profile_operations").is_array());
        assert!(field(case, "expected_required_checks").is_array());
        let expectation = field(case, "expect");
        assert_exact_keys(expectation, &["status", "phase", "code"]);
        assert_eq!(text(field(expectation, "status")), "ir-lowered");
        assert_eq!(text(field(expectation, "phase")), "complete");
        assert_eq!(text(field(expectation, "code")), "");
    }
    for case in rejected {
        assert_exact_keys(case, &["id", "stage", "mutation", "expect"]);
        assert!(ids.insert(text(field(case, "id"))));
        let expectation = field(case, "expect");
        assert_exact_keys(expectation, &["status", "phase", "code"]);
        assert_ne!(text(field(expectation, "status")), "ir-lowered");
        assert_eq!(
            text(field(expectation, "phase")),
            text(field(case, "stage"))
        );
        let code = text(field(expectation, "code"));
        assert!(diagnostic_codes.contains(code));
        directly_covered_diagnostics.insert(code);
        let (owner_status, owner_phase) = diagnostic_owners
            .get(code)
            .unwrap_or_else(|| panic!("missing diagnostic owner for {code}"));
        assert_eq!(text(field(expectation, "status")), *owner_status);
        if *owner_phase != "owner-phase" {
            assert_eq!(text(field(expectation, "phase")), *owner_phase);
        }
    }
    assert_eq!(ids.len(), 118);
    assert_eq!(directly_covered_diagnostics, diagnostic_codes);

    let precedence = array(field(profile, "precedence_cases"));
    assert_eq!(precedence.len(), 12);
    for case in precedence {
        assert_exact_keys(case, &["id", "coexisting", "winner"]);
        let winner = text(field(case, "winner"));
        assert!(array(field(case, "coexisting"))
            .iter()
            .any(|code| text(code) == winner));
        assert!(diagnostic_codes.contains(winner));
    }
}

fn assert_diagnostic_normalization(value: &Value) {
    assert_exact_keys(
        value,
        &[
            "public_code",
            "public_message_template",
            "roslyn_id_pattern",
            "status_messages",
            "limit_message",
            "roslyn_sort",
            "roslyn_severity_order",
            "absent_roslyn_location_sort_key",
            "public_span_policy",
            "public_issue_sort",
            "omitted",
        ],
    );
    assert_eq!(
        text(field(value, "public_code")),
        "CSHARP_SOURCE_DIAGNOSTIC"
    );
    assert_eq!(
        text(field(value, "public_message_template")),
        "C# compiler diagnostic CSNNNN"
    );
    assert_eq!(text(field(value, "roslyn_id_pattern")), "CS[0-9]{4}");
    let status_messages = field(value, "status_messages");
    assert_exact_keys(
        status_messages,
        &["source-error", "rejected", "frontend-error"],
    );
    assert_eq!(
        text(field(status_messages, "source-error")),
        "C# source is invalid"
    );
    assert_eq!(
        text(field(status_messages, "rejected")),
        "C# source is outside the frozen profile"
    );
    assert_eq!(
        text(field(status_messages, "frontend-error")),
        "C# frontend failed closed"
    );
    assert_eq!(
        text(field(value, "limit_message")),
        "C# profile limit exceeded"
    );
    assert_eq!(
        array(field(value, "roslyn_sort"))
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        [
            "normalized_path",
            "utf8_start",
            "utf8_end",
            "roslyn_id",
            "severity",
            "message_bytes",
        ]
    );
    assert_eq!(
        array(field(value, "roslyn_severity_order"))
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        ["Hidden", "Info", "Warning", "Error"]
    );
    let absent = field(value, "absent_roslyn_location_sort_key");
    assert_exact_keys(absent, &["normalized_path", "utf8_start", "utf8_end"]);
    assert_eq!(text(field(absent, "normalized_path")), "");
    assert_eq!(integer(field(absent, "utf8_start")), 0);
    assert_eq!(integer(field(absent, "utf8_end")), 0);
    let span = field(value, "public_span_policy");
    assert_exact_keys(
        span,
        &[
            "captured_nonempty_mappable",
            "zero_length_absent_external_or_unmappable",
        ],
    );
    assert_eq!(
        array(field(value, "public_issue_sort"))
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        [
            "span.normalized_path_or_empty",
            "span.start_or_zero",
            "code",
            "message",
            "function_id_or_empty",
            "span.end_or_zero",
        ]
    );
}

fn assert_limit_cases(value: &Value) {
    let cases = array(value);
    assert_eq!(cases.len(), 32);
    let expected = BTreeMap::from([
        ("source_files", 256),
        ("source_file_bytes", 1_048_576),
        ("source_total_bytes", 16_777_216),
        ("contract_files", 128),
        ("contract_file_bytes", 1_048_576),
        ("contract_total_bytes", 8_388_608),
        ("snapshot_entries", 512),
        ("snapshot_total_bytes", 33_554_432),
        ("normalized_path_bytes", 1_024),
        ("canonical_method_id_bytes", 1_024),
        ("selected_methods", 32),
        ("method_closure", 128),
        ("syntax_nodes", 250_000),
        ("operations_per_method", 100_000),
        ("operations_per_closure", 250_000),
        ("cfg_blocks_per_method", 1_024),
        ("cfg_blocks_per_closure", 8_192),
        ("contract_clauses", 64),
        ("contract_nodes_per_method", 1_024),
        ("contract_nodes_per_closure", 8_192),
        ("contract_depth", 32),
        ("normalized_issues", 1_024),
        ("diagnostic_message_bytes_each", 4_096),
        ("diagnostic_message_bytes_total", 2_097_152),
        ("frontend_argument_bytes", 131_072),
        ("private_runtime_stdout", 268_435_456),
        ("private_runtime_stderr", 2_097_152),
        ("vir_canonical_bytes", 201_326_592),
        ("source_map_canonical_bytes", 33_554_432),
        ("source_manifest_canonical_bytes", 4_194_304),
        ("frontend_stdout", 268_435_456),
        ("frontend_stderr", 2_097_152),
    ]);
    let diagnostic_budget_limits = BTreeSet::from([
        "normalized_issues",
        "diagnostic_message_bytes_each",
        "diagnostic_message_bytes_total",
    ]);
    let output_limits = BTreeSet::from([
        "private_runtime_stdout",
        "private_runtime_stderr",
        "frontend_stdout",
        "frontend_stderr",
    ]);
    let mut actual = BTreeMap::new();
    for case in cases {
        assert_exact_keys(
            case,
            &[
                "id",
                "maximum",
                "scope",
                "exact_boundary",
                "boundary_plus_one",
                "code",
            ],
        );
        assert_eq!(text(field(case, "exact_boundary")), "accept");
        let id = text(field(case, "id"));
        if diagnostic_budget_limits.contains(id) {
            assert_eq!(
                text(field(case, "boundary_plus_one")),
                "frontend_error_before_retaining_excess"
            );
            assert_eq!(
                text(field(case, "code")),
                "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET"
            );
        } else if output_limits.contains(id) {
            assert_eq!(
                text(field(case, "boundary_plus_one")),
                "frontend_error_before_retaining_excess"
            );
            assert_eq!(text(field(case, "code")), "CSHARP_FRONTEND_OUTPUT_LIMIT");
        } else {
            assert_eq!(
                text(field(case, "boundary_plus_one")),
                "reject_before_retaining_excess"
            );
            assert!(text(field(case, "code")).starts_with("CSHARP_LIMIT_"));
        }
        assert!(actual.insert(id, integer(field(case, "maximum"))).is_none());
    }
    assert_eq!(actual, expected);
}
fn assert_profile_hash_case(profile: &Value, case: &Value) {
    assert_exact_keys(
        case,
        &[
            "id",
            "source_pointer",
            "domain",
            "excluded_field",
            "expected_payload_utf8_length",
            "expected_preimage_length",
            "expected_sha256",
        ],
    );
    let pointer = text(field(case, "source_pointer"));
    let source = profile
        .pointer(pointer)
        .unwrap_or_else(|| panic!("missing profile hash pointer {pointer}"));
    let domain = domain(text(field(case, "domain")));
    let payload = payload_without(source, field(case, "excluded_field").as_str());
    let canonical_payload = canonical(&payload);
    assert_eq!(
        canonical_payload.len() as i64,
        integer(field(case, "expected_payload_utf8_length")),
        "{}",
        text(field(case, "id"))
    );
    assert_eq!(
        (domain.as_str().len() + 1 + canonical_payload.len()) as i64,
        integer(field(case, "expected_preimage_length")),
        "{}",
        text(field(case, "id"))
    );
    assert_eq!(
        hash_canonical_json(domain, &strict_value(&payload))
            .expect("C# profile payload hashes")
            .to_hex(),
        text(field(case, "expected_sha256")),
        "{}",
        text(field(case, "id"))
    );
}

fn assert_registry_hash_case(registry: &Value, case: &Value) {
    let expected_keys: &[&str] = if case.get("expected_transport_utf8_length").is_some() {
        &[
            "id",
            "source_pointer",
            "domain",
            "excluded_field",
            "expected_payload_utf8_length",
            "expected_preimage_length",
            "expected_complete_jcs_utf8_length",
            "expected_transport_utf8_length",
            "expected_transport_sha256",
            "expected_sha256",
        ]
    } else {
        &[
            "id",
            "source_pointer",
            "domain",
            "excluded_field",
            "expected_payload_utf8_length",
            "expected_preimage_length",
            "expected_complete_jcs_utf8_length",
            "expected_sha256",
        ]
    };
    assert_exact_keys(case, expected_keys);
    let pointer = text(field(case, "source_pointer"));
    let complete = if pointer.is_empty() {
        registry
    } else {
        registry
            .pointer(pointer)
            .unwrap_or_else(|| panic!("missing registry hash pointer {pointer}"))
    };
    let canonical_complete = canonical(complete);
    assert_eq!(
        canonical_complete.len() as i64,
        integer(field(case, "expected_complete_jcs_utf8_length"))
    );

    let payload = payload_without(complete, field(case, "excluded_field").as_str());
    let domain = domain(text(field(case, "domain")));
    let canonical_payload = canonical(&payload);
    assert_eq!(
        canonical_payload.len() as i64,
        integer(field(case, "expected_payload_utf8_length"))
    );
    assert_eq!(
        (domain.as_str().len() + 1 + canonical_payload.len()) as i64,
        integer(field(case, "expected_preimage_length"))
    );
    assert_eq!(
        hash_canonical_json(domain, &strict_value(&payload))
            .expect("registry payload hashes")
            .to_hex(),
        text(field(case, "expected_sha256"))
    );

    if let Some(expected_length) = case.get("expected_transport_utf8_length") {
        let mut transport = canonical_complete;
        transport.push(b'\n');
        assert_eq!(transport.len() as i64, integer(expected_length));
        assert_eq!(
            sha256_raw_file_bytes(&transport).to_hex(),
            text(field(case, "expected_transport_sha256"))
        );
        assert_eq!(
            text(field(case, "expected_transport_sha256")),
            REGISTRY_V2_TRANSPORT_HASH
        );
    }
}

fn payload_without(value: &Value, excluded: Option<&str>) -> Value {
    let mut payload = value.clone();
    if let Some(excluded) = excluded {
        object_mut(&mut payload)
            .remove(excluded)
            .unwrap_or_else(|| panic!("missing excluded field {excluded}"));
    }
    payload
}

fn domain(value: &str) -> HashDomain {
    match value {
        "MPK-SEMANTIC-PROFILE-ENTRY-1.0" => ENTRY_DOMAIN,
        "MPK-SEMANTIC-PROFILE-REGISTRY-1.0" => REGISTRY_DOMAIN,
        "MPK-CSHARP-REFERENCE-INVENTORY-0.1" => REFERENCE_DOMAIN,
        "MPK-CSHARP-TOOLCHAIN-INPUTS-0.1" => TOOLCHAIN_DOMAIN,
        "MPK-CSHARP-SELECTION-0.1" => SELECTION_DOMAIN,
        "MPK-CSHARP-CONTRACT-SIDECAR-0.1" => SIDECAR_DOMAIN,
        "MPK-CONTRACT-1.0" => CONTRACT_DOMAIN,
        _ => panic!("unknown C# profile hash domain {value}"),
    }
}

fn load(bytes: &[u8], label: &str) -> Value {
    parse_strict_json(bytes, TEST_LIMITS)
        .unwrap_or_else(|error| panic!("strict {label} vectors parse: {error}"));
    serde_json::from_slice(bytes).unwrap_or_else(|error| panic!("{label} JSON parses: {error}"))
}

fn strict_value(value: &Value) -> StrictJsonValue {
    let bytes = serde_json::to_vec(value).expect("serialize test JSON value");
    parse_strict_json(&bytes, TEST_LIMITS).expect("strict test JSON value parses")
}

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&strict_value(value)).expect("test JSON value canonicalizes")
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name:?}"))
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("value is an array")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("value is an object")
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("value is an object")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("value is a string")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().expect("value is an integer")
}

fn boolean(value: &Value) -> bool {
    value.as_bool().expect("value is a Boolean")
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let actual = object(value)
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "closed object keys differ");
}

fn assert_sorted_unique_strings(value: &Value) {
    let values = array(value).iter().map(text).collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
}

fn assert_lower_hex(value: &str, length: usize) {
    assert_eq!(value.len(), length);
    assert!(value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}
