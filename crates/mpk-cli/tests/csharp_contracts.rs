use serde_json::Value;
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

#[test]
fn frozen_contract_vector_semantic_row_and_limits_are_owned_exactly() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    assert_eq!(
        profile["contract_sidecar_sha256"],
        "6684361a15dc454a8172d7e515dd6a3a49ec1ff8faae00bc12d958eae8982228"
    );
    assert_eq!(
        profile["normalized_contract_fixture"]["contract_hash"],
        "b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab"
    );
    assert_eq!(
        profile["contract_fixture"]["schema"],
        "mpk.csharp.contract.v0"
    );
    assert_eq!(
        profile["contract_fixture"]["semantic_profile"],
        "mpk.csharp.scalar.v0"
    );
    assert_eq!(
        profile["contract_fixture"]["modifies"],
        serde_json::json!([])
    );
    assert_eq!(
        profile["contract_fixture"]["abrupt_completion"],
        "forbidden"
    );
    assert_eq!(profile["contract_fixture"]["termination"], "total");

    let hash_cases = profile["hash_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| {
            matches!(
                case["id"].as_str().unwrap(),
                "hash.contract_sidecar" | "hash.normalized_contract"
            )
        })
        .map(|case| {
            (
                case["id"].as_str().unwrap(),
                (
                    case["domain"].as_str().unwrap(),
                    case["expected_payload_utf8_length"].as_u64().unwrap(),
                    case["expected_preimage_length"].as_u64().unwrap(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        hash_cases,
        BTreeMap::from([
            (
                "hash.contract_sidecar",
                ("MPK-CSHARP-CONTRACT-SIDECAR-0.1", 440, 472),
            ),
            ("hash.normalized_contract", ("MPK-CONTRACT-1.0", 1151, 1168),),
        ])
    );

    let limits = profile["limit_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["id"].as_str().unwrap().starts_with("contract_"))
        .map(|case| {
            (
                case["id"].as_str().unwrap(),
                (
                    case["maximum"].as_u64().unwrap(),
                    case["code"].as_str().unwrap(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(limits.len(), 7);
    for (id, maximum, code) in [
        ("contract_files", 128, "CSHARP_LIMIT_CONTRACT_FILES"),
        (
            "contract_file_bytes",
            1_048_576,
            "CSHARP_LIMIT_CONTRACT_FILE_BYTES",
        ),
        (
            "contract_total_bytes",
            8_388_608,
            "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES",
        ),
        ("contract_clauses", 64, "CSHARP_LIMIT_CONTRACT_CLAUSES"),
        (
            "contract_nodes_per_method",
            1024,
            "CSHARP_LIMIT_CONTRACT_NODES_PER_METHOD",
        ),
        (
            "contract_nodes_per_closure",
            8192,
            "CSHARP_LIMIT_CONTRACT_NODES_PER_CLOSURE",
        ),
        ("contract_depth", 32, "CSHARP_LIMIT_CONTRACT_DEPTH"),
    ] {
        assert_eq!(limits[id], (maximum, code));
    }

    let row = profile["semantic_rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["row"] == "M34")
        .unwrap();
    assert_eq!(row["disposition"], "accept_under_profile_restrictions");
    assert_eq!(row["basis"], "P05");
}

#[test]
fn parser_attachment_typing_and_hashing_are_private_and_closed() {
    let root = repository_root();
    let parser = fs::read_to_string(root.join("csharp-tools/csharp2vir/ContractParser.cs"))
        .expect("read contract parser");
    let attachment = fs::read_to_string(root.join("csharp-tools/csharp2vir/ContractAttachment.cs"))
        .expect("read contract attachment");
    let canonical = fs::read_to_string(root.join("csharp-tools/csharp2vir/ContractCanonical.cs"))
        .expect("read contract canonicalizer");
    let model = fs::read_to_string(root.join("csharp-tools/csharp2vir/ContractModel.cs"))
        .expect("read contract model");
    let limits = fs::read_to_string(root.join("csharp-tools/csharp2vir/FrontendLimits.cs"))
        .expect("read frontend limits");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive frontend");
    let combined = format!("{parser}\n{attachment}\n{canonical}\n{model}\n{limits}");

    for required in [
        "Utf8JsonReader",
        "AllowTrailingCommas = false",
        "JsonCommentHandling.Disallow",
        "CSHARP_CONTRACT_JSON",
        "CSHARP_CONTRACT_SHAPE",
        "CSHARP_CONTRACT_IDENTITY",
        "CSHARP_CONTRACT_DUPLICATE",
        "CSHARP_CONTRACT_MISSING",
        "CSHARP_CONTRACT_UNUSED",
        "CSHARP_CONTRACT_TYPE",
        "CSHARP_CONTRACT_OPERATOR",
        "CSHARP_CONTRACT_HASH",
        "ContractClausesMaximum = 64",
        "ContractNodesPerMethodMaximum = 1_024",
        "ContractNodesPerClosureMaximum = 8_192",
        "ContractDepthMaximum = 32",
        "NumberStyles.AllowLeadingSign",
        "ValueSpan.SequenceEqual(\"0\"u8)",
        "MPK-CSHARP-CONTRACT-SIDECAR-0.1",
        "MPK-CONTRACT-1.0",
        "MPK-CSHARP-SELECTION-0.1",
        "CryptographicOperations.FixedTimeEquals",
        "panic",
        "forbidden",
        "termination",
        "total",
        "modifies",
        "WriteSemanticContext",
        "RawInputSha256",
        "SidecarSha256",
        "ContractHash",
    ] {
        assert!(
            combined.contains(required),
            "missing contract owner {required}"
        );
    }

    for operation in [
        "not",
        "bv_neg",
        "bv_not",
        "and",
        "or",
        "eq",
        "not_eq",
        "signed_lt",
        "signed_le",
        "signed_gt",
        "signed_ge",
        "unsigned_lt",
        "unsigned_le",
        "unsigned_gt",
        "unsigned_ge",
        "bv_add",
        "bv_sub",
        "bv_mul",
        "bv_and",
        "bv_or",
        "bv_xor",
        "bv_shl",
        "bv_ashr",
        "bv_lshr",
    ] {
        assert!(
            combined.contains(&format!("\"{operation}\"")),
            "missing {operation}"
        );
    }
    for forbidden in [
        "GetAttributes(",
        "GetDocumentationComment",
        "AttributeData",
        "DocumentationCommentTriviaSyntax",
        "CodeContract",
        "OpenStandardOutput",
    ] {
        assert!(
            !combined.contains(forbidden),
            "forbidden contract source {forbidden}"
        );
    }

    let method_counter = parser
        .split_once("internal void AddNode(uint depth)")
        .expect("method counter")
        .1;
    assert!(
        method_counter.find("NodesPerMethodMaximum").unwrap()
            < method_counter.find("closure.AddNode()").unwrap()
    );
    assert!(
        method_counter.find("closure.AddNode()").unwrap() < method_counter.find("nodes++").unwrap()
    );
    let closure_counter = parser
        .split_once("internal sealed class ContractClosureCounter")
        .expect("closure counter")
        .1;
    assert!(
        closure_counter.find("NodesPerClosureMaximum").unwrap()
            < closure_counter.find("nodes++").unwrap()
    );

    let subset = program.find("CSharpSubset.Validate").unwrap();
    let contracts = program.find("CSharpContracts.Attach").unwrap();
    let lowering = program.find("CSharpLowering.Lower").unwrap();
    assert!(subset < contracts && contracts < lowering);
    assert!(program.contains("phase = \"subset\""));
}

#[test]
fn executable_harness_build_gate_and_vector_manifest_own_t08() {
    let root = repository_root();
    let harness = fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_contracts_harness.cs"))
        .expect("read contract harness");
    for owner in [
        "FrozenContractVectorNormalizesExactly",
        "StrictJsonShapeIdentityAndRawClaimsAreClosed",
        "AttachmentIsExactAndSelectionBound",
        "SuccessorExpressionTypingIsExact",
        "ContractLimitsRejectBeforeExcessRetention",
        "SemanticRowM34IsOwned",
        "6684361a15dc454a8172d7e515dd6a3a49ec1ff8faae00bc12d958eae8982228",
        "b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab",
        "1_151",
        "1_024",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }

    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    assert!(script.contains("validate_contract_implementation"));
    assert!(script.contains("run_contract_tests=True"));
    assert!(script.contains("csharp2vir-contract-tests.dll"));
    assert!(script.contains("argv == [\"test-contracts\"]"));
    let assembly = fs::read_to_string(root.join("csharp-tools/csharp2vir/AssemblyInfo.cs"))
        .expect("read assembly metadata");
    assert!(assembly.contains("InternalsVisibleTo(\"csharp2vir-contract-tests\")"));

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
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_contracts.rs"));
}

#[test]
fn provisioned_offline_closure_executes_the_contract_harness() {
    if !cfg!(target_os = "linux") {
        return;
    }

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
        .arg("--test-contracts")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# contract harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
