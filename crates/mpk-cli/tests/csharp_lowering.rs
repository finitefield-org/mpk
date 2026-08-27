use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
}

fn joined(record: &Value, field: &str) -> String {
    record[field]
        .as_array()
        .unwrap_or_else(|| panic!("{field} array"))
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

fn owned<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}

#[test]
fn frozen_scalar_checked_conversion_and_operation_vectors_are_exact() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    assert_eq!(
        profile["type_mappings"],
        json!([
            {"source_type":"bool","method_id_token":"bool","vir_type":{"kind":"bool"}},
            {"source_type":"int","method_id_token":"i32","vir_type":{"kind":"bv","width":32,"signed":true}},
            {"source_type":"uint","method_id_token":"u32","vir_type":{"kind":"bv","width":32,"signed":false}},
            {"source_type":"long","method_id_token":"i64","vir_type":{"kind":"bv","width":64,"signed":true}},
            {"source_type":"ulong","method_id_token":"u64","vir_type":{"kind":"bv","width":64,"signed":false}}
        ])
    );

    let checked = profile["roslyn_checked_state_cases"]
        .as_array()
        .expect("checked-state cases")
        .iter()
        .map(|case| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                case["id"].as_str().unwrap(),
                case["source"].as_str().unwrap(),
                case["operator_kind"].as_str().unwrap(),
                joined(case, "operand_types"),
                case["context"].as_str().unwrap(),
                case["expected_is_checked"].as_bool().unwrap()
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        checked,
        owned([
            "checked.add|+|Add|int,uint,long,ulong|checked|true",
            "checked.subtract|-|Subtract|int,uint,long,ulong|checked|true",
            "checked.multiply|*|Multiply|int,uint,long,ulong|checked|true",
            "checked.negate|unary-|Minus|int,long|checked|true",
            "checked.divide|/|Divide|int,uint,long,ulong|checked|true",
            "checked.remainder|%|Remainder|int,uint,long,ulong|checked|false",
            "unchecked.add|+|Add|int,uint,long,ulong|unchecked|false",
            "unchecked.subtract|-|Subtract|int,uint,long,ulong|unchecked|false",
            "unchecked.multiply|*|Multiply|int,uint,long,ulong|unchecked|false",
            "unchecked.negate|unary-|Minus|int,long|unchecked|false",
            "unchecked.divide|/|Divide|int,uint,long,ulong|unchecked|false",
            "unchecked.remainder|%|Remainder|int,uint,long,ulong|unchecked|false",
        ])
    );

    let conversions = profile["conversion_rules"]
        .as_array()
        .expect("conversion rules")
        .iter()
        .map(|rule| {
            format!(
                "{}>{}|{}|{}|{}|{}",
                rule["source_type"].as_str().unwrap(),
                rule["destination_type"].as_str().unwrap(),
                rule["source_form"].as_str().unwrap(),
                rule["context"].as_str().unwrap(),
                joined(rule, "vir"),
                joined(rule, "checks")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        conversions,
        owned([
            "bool>bool|identity|any||",
            "int>int|identity|any||",
            "int>uint|explicit|unchecked|Convert|",
            "int>long|implicit|any|Convert|",
            "int>long|explicit|unchecked|Convert|",
            "int>ulong|explicit|unchecked|Convert|",
            "uint>int|explicit|unchecked|Convert|",
            "uint>uint|identity|any||",
            "uint>long|implicit|any|Convert|",
            "uint>long|explicit|unchecked|Convert|",
            "uint>ulong|implicit|any|Convert|",
            "uint>ulong|explicit|unchecked|Convert|",
            "long>int|explicit|unchecked|Convert|",
            "long>uint|explicit|unchecked|Convert|",
            "long>long|identity|any||",
            "long>ulong|explicit|unchecked|Convert|",
            "ulong>int|explicit|unchecked|Convert|",
            "ulong>uint|explicit|unchecked|Convert|",
            "ulong>long|explicit|unchecked|Convert|",
            "ulong>ulong|identity|any||",
        ])
    );

    let mappings = profile["operation_mappings"]
        .as_array()
        .expect("operation mappings");
    assert_eq!(mappings.len(), 35);
    let non_call = mappings[..34]
        .iter()
        .map(|mapping| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                mapping["source"].as_str().unwrap(),
                joined(mapping, "operand_types"),
                mapping
                    .get("count_type")
                    .and_then(Value::as_str)
                    .unwrap_or("-"),
                mapping["context"].as_str().unwrap(),
                joined(mapping, "vir"),
                joined(mapping, "checks")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        non_call,
        owned([
            "!|bool|-|any|bool_not|",
            "==|bool,int,uint,long,ulong|-|any|eq|",
            "!=|bool,int,uint,long,ulong|-|any|not_eq|",
            "<|int,long|-|any|signed_lt|",
            "<=|int,long|-|any|signed_le|",
            ">|int,long|-|any|signed_gt|",
            ">=|int,long|-|any|signed_ge|",
            "<|uint,ulong|-|any|unsigned_lt|",
            "<=|uint,ulong|-|any|unsigned_le|",
            ">|uint,ulong|-|any|unsigned_gt|",
            ">=|uint,ulong|-|any|unsigned_ge|",
            "+|int,uint,long,ulong|-|checked|bv_add|integer_no_overflow",
            "-|int,uint,long,ulong|-|checked|bv_sub|integer_no_overflow",
            "*|int,uint,long,ulong|-|checked|bv_mul|integer_no_overflow",
            "+|int,uint,long,ulong|-|unchecked|bv_add|",
            "-|int,uint,long,ulong|-|unchecked|bv_sub|",
            "*|int,uint,long,ulong|-|unchecked|bv_mul|",
            "unary-|int,long|-|checked|bv_neg|integer_no_overflow",
            "unary-|int,long|-|unchecked|bv_neg|",
            "/|int,long|-|explicit_checked_or_unchecked|bv_sdiv|divisor_nonzero,signed_divrem_representable",
            "%|int,long|-|explicit_checked_or_unchecked|bv_srem|divisor_nonzero,signed_divrem_representable",
            "/|uint,ulong|-|explicit_checked_or_unchecked|bv_udiv|divisor_nonzero",
            "%|uint,ulong|-|explicit_checked_or_unchecked|bv_urem|divisor_nonzero",
            "~|int,uint,long,ulong|-|any|bv_not|",
            "&|int,uint,long,ulong|-|any|bv_and|",
            "||int,uint,long,ulong|-|any|bv_or|",
            "^|int,uint,long,ulong|-|any|bv_xor|",
            "<<|int,uint|int|any|bv_and(count,31),bv_shl|",
            "<<|long,ulong|int|any|bv_and(count,63),bv_shl|",
            ">>|int,long|int|any|bv_and(count,width-1),bv_ashr|",
            ">>|uint,ulong|int|any|bv_and(count,width-1),bv_lshr|",
            "&&|bool|-|any|Branch,block_parameter|",
            "|||bool|-|any|Branch,block_parameter|",
            "?:|bool_condition,identical_accepted_branch_type|-|any|Branch,Jump,block_parameter|",
        ])
    );
    assert_eq!(
        mappings[34],
        json!({
            "source":"direct_static_call",
            "operand_types":["exact_signature"],
            "context":"any",
            "vir":["CallStatic"],
            "checks":["callee_contract_hash"]
        })
    );
}

#[test]
fn lowering_diagnostics_and_semantic_rows_are_owned_exactly() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let diagnostics = profile["diagnostic_registry"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["phase"] == "lowering")
        .map(|entry| {
            (
                entry["code"].as_str().unwrap(),
                entry["status"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostics,
        [
            ("CSHARP_LOWERING_OPERATION", "rejected"),
            ("CSHARP_LOWERING_CFG", "rejected"),
            ("CSHARP_LOWERING_CHECK_MISSING", "rejected"),
            ("CSHARP_LOWERING_CHECK_EXTRA", "rejected"),
            ("CSHARP_LOWERING_CHECK_ORDER", "rejected"),
        ]
    );

    let owned_rows = [
        "M01", "M02", "M07", "M08", "M09", "M10", "M11", "M12", "M13", "M14", "M16", "M18", "M19",
        "M21", "M29",
    ];
    let rows = profile["semantic_rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| owned_rows.contains(&row["row"].as_str().unwrap()))
        .map(|row| {
            (
                row["row"].as_str().unwrap(),
                (
                    row["disposition"].as_str().unwrap(),
                    row["basis"].as_str().unwrap(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        rows,
        BTreeMap::from([
            ("M01", ("accept_under_profile_restrictions", "P02")),
            ("M02", ("accept_under_profile_restrictions", "P02")),
            ("M07", ("accept_under_profile_restrictions", "P04")),
            ("M08", ("accept_under_profile_restrictions", "P02")),
            ("M09", ("accept_under_profile_restrictions", "P02")),
            ("M10", ("accept_under_profile_restrictions", "P04")),
            ("M11", ("accept_under_profile_restrictions", "P04")),
            ("M12", ("accept_under_profile_restrictions", "P04")),
            ("M13", ("accept_under_profile_restrictions", "P02")),
            ("M14", ("accept_under_profile_restrictions", "P02")),
            ("M16", ("accept_under_profile_restrictions", "P02")),
            ("M18", ("accept_under_profile_restrictions", "P02")),
            ("M19", ("accept_under_profile_restrictions", "P02")),
            ("M21", ("accept_under_profile_restrictions", "P02")),
            ("M29", ("accept_under_profile_restrictions", "P04")),
        ])
    );
}

#[test]
fn lowering_is_private_complete_and_separated_from_emission() {
    let root = repository_root();
    let model = fs::read_to_string(root.join("csharp-tools/csharp2vir/LoweringModel.cs"))
        .expect("read lowering model");
    let builder = fs::read_to_string(root.join("csharp-tools/csharp2vir/LoweringBuilder.cs"))
        .expect("read lowering builder");
    let validation = fs::read_to_string(root.join("csharp-tools/csharp2vir/LoweringValidation.cs"))
        .expect("read lowering validation");
    let subset = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetOperations.cs"))
        .expect("read subset operations");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive frontend");
    let combined = format!("{model}\n{builder}\n{validation}");

    for required in [
        "LoweredValueKind",
        "LoweredInstructionKind",
        "LoweredTerminatorKind",
        "LoweredConversionForm",
        "LoweredSafetyCheckKind",
        "LoweredRequiredCheck",
        "CanonicalRequiredChecks",
        "CanonicalBlockOrder",
        "LowerShortCircuit",
        "MergeFlows",
        "ApplyContextConversion",
        "IntegerNoOverflow",
        "DivisorNonzero",
        "SignedDivremRepresentable",
        "CSHARP_LOWERING_OPERATION",
        "CSHARP_LOWERING_CFG",
        "CSHARP_LOWERING_CHECK_MISSING",
        "CSHARP_LOWERING_CHECK_EXTRA",
        "CSHARP_LOWERING_CHECK_ORDER",
    ] {
        assert!(
            combined.contains(required),
            "missing lowering owner {required}"
        );
    }
    for forbidden in [
        "Utf8JsonWriter",
        "FrontendEnvelope",
        "OpenStandardOutput",
        "MPK-VIR",
        ".Emit(",
    ] {
        assert!(
            !combined.contains(forbidden),
            "later emission surface {forbidden}"
        );
    }
    assert!(model.contains("CallStatic"));
    assert!(subset.contains("(!classified.IsExplicit && !classified.IsImplicit)"));

    let contracts = program.find("CSharpContracts.Attach").unwrap();
    let lowering = program.find("CSharpLowering.Lower").unwrap();
    let emission = program.find("CSharpFrontendSuccessEmitter.Emit").unwrap();
    let output = program.find("Console.OpenStandardOutput").unwrap();
    assert!(contracts < lowering && lowering < emission && emission < output);
    assert!(program.contains("phase = \"lowering\""));
    assert!(program.contains("phase = \"emission\""));
}

#[test]
fn executable_harness_build_gate_and_vector_manifest_own_t09() {
    let root = repository_root();
    let harness = fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_lowering_harness.cs"))
        .expect("read lowering harness");
    for owner in [
        "TypeMappingsAreExact",
        "ConstantsAreExplicitAndBounded",
        "NonCallOperationMappingsAreComplete",
        "RoslynCheckedStatesAreExact",
        "ConversionRulesAreExact",
        "ControlFlowAndEvaluationAreDeterministic",
        "RequiredChecksAreExactAndClosed",
        "SemanticRowsAreOwned",
        "CallStaticIsT10Owned",
        "CHECK_CANONICAL_ORDER",
        "CALLSTATIC_T10_FEATURE",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }

    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    for owner in [
        "validate_lowering_implementation",
        "run_lowering_tests=True",
        "csharp2vir-lowering-tests.dll",
        "argv == [\"test-lowering\"]",
    ] {
        assert!(script.contains(owner), "missing build owner {owner}");
    }
    let shell = fs::read_to_string(root.join("scripts/build-csharp-frontend.sh"))
        .expect("read C# build entrypoint");
    assert!(shell.contains("--test-lowering"));
    let assembly = fs::read_to_string(root.join("csharp-tools/csharp2vir/AssemblyInfo.cs"))
        .expect("read assembly metadata");
    assert!(assembly.contains("InternalsVisibleTo(\"csharp2vir-lowering-tests\")"));
    let project = fs::read_to_string(root.join("csharp-tools/csharp2vir/csharp2vir.csproj"))
        .expect("read C# project");
    for input in [
        "LoweringBuilder.cs",
        "LoweringModel.cs",
        "LoweringValidation.cs",
    ] {
        assert!(project.contains(input), "missing project input {input}");
    }

    let manifest = load("develop/specs/vectors/manifest.json");
    let record = manifest["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == "develop/specs/vectors/csharp-profile-v0.json")
        .expect("C# vector record");
    assert!(record["implementation_test_owners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_lowering.rs"));
}

#[test]
fn provisioned_offline_closure_executes_the_lowering_harness() {
    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let hash = profile["toolchain_inputs"]["toolchain_inputs_sha256"]
        .as_str()
        .unwrap();
    let cache = root
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = profile["toolchain_inputs"]["archives"].as_array().unwrap();
    let present = archives
        .iter()
        .filter(|record| {
            let suffix = match record["kind"].as_str().unwrap() {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!("{}{}", record["id"].as_str().unwrap(), suffix))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial C# archive cache"
    );
    if present == 0 {
        return;
    }

    let output = Command::new(root.join("scripts/build-csharp-frontend.sh"))
        .arg("--test-lowering")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# lowering harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
