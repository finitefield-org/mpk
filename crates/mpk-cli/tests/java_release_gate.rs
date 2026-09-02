//! JAVA-03-T09 owner: deterministic differential/fuzz/upgrade/private release rehearsal.

#![cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64")),
    allow(unused_imports, dead_code)
)]

#[path = "support/java_lowering.rs"]
#[allow(dead_code)]
mod java_lowering;
#[path = "support/successor_policy.rs"]
mod successor_policy_support;

use mpk_cert::encode::AxiomCategory;
use mpk_cli::policy_profile::lookup_strategy_registration;
use mpk_cli::successor_release_bundle::validate_successor_bundle_candidate;
use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_registry_semantic_context, ProfileContractField,
};
use mpk_vc::sha256_raw_file_bytes;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;

use successor_policy_support::{
    candidate_registry, checked_in_json, profile_contract, registry, repository_path,
};

const CORPUS_PATH: &str = "develop/specs/vectors/java-t09-corpus.json";
const OWNER: &str = "crates/mpk-cli/tests/java_release_gate.rs";

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("canonical JSON");
    bytes.push(b'\n');
    bytes
}

fn states(seed: &str, iterations: usize) -> (Vec<u64>, String) {
    let mut value = u64::from_str_radix(seed, 16).expect("fuzz seed");
    let mut digest = Sha256::new();
    let mut values = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        digest.update(value.to_be_bytes());
        values.push(value);
    }
    (values, format!("{:x}", digest.finalize()))
}

#[test]
fn t09_owns_exact_upgrade_corpus_and_keeps_public_activation_closed() {
    let corpus = checked_in_json(CORPUS_PATH);
    let java = checked_in_json("develop/specs/vectors/java-profile-v0.json");
    let manifest = checked_in_json("develop/specs/vectors/manifest.json");
    assert_eq!(corpus["schema"], "mpk.java.t09.corpus.v0");
    assert_eq!(corpus["owner_test"], OWNER);
    assert_eq!(corpus["task"], "JAVA-03-T09");
    assert_eq!(
        corpus["candidate_upgrade"],
        json!({
            "from_frontend_bundle_id":"frontend.java.java2vir.candidate.v1",
            "from_jar_sha256":"333a050128cddc206474c9bdcca244276c08b246f2a5ba11f55983537cf7cd75",
            "to_frontend_bundle_id":"frontend.java.java2vir.candidate.v2",
            "to_jar_sha256":"aeddb537d396bc7374390d5d01c4dc576c1975e2244dcf7a64de5757fd921558",
            "unchanged_toolchain_bundle_id":"toolchain.java.temurin-25_0_4_1_1.candidate.v1",
            "parent_snapshot_boundary":"complete_descriptor_inventory_before_source",
            "removed_child_rehash_bytes":175883627,
            "unchanged_launcher_mode":"-Xint",
            "unchanged_timeout_seconds":120
        })
    );
    assert_eq!(
        corpus["differential_supplements"].as_array().unwrap().len(),
        5
    );
    assert_eq!(
        corpus["differential_supplements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "int.division.zero",
            "int.remainder.zero",
            "call.dead_branch.normal",
            "call.multiple_entrypoints.caller",
            "call.multiple_entrypoints.callee",
        ]
    );
    for row in corpus["differential_supplements"].as_array().unwrap() {
        let object = row.as_object().unwrap();
        let outcome = match (object.contains_key("result"), object.contains_key("trap")) {
            (true, false) => "result",
            (false, true) => "trap",
            _ => panic!("supplement must have exactly one outcome"),
        };
        assert_eq!(
            object.keys().map(String::as_str).collect::<Vec<_>>(),
            ["arguments", "case_id", "id", "method", outcome]
        );
        assert!(row["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .all(Value::is_string));
        assert!(row[outcome].is_string());
    }
    assert_eq!(
        corpus["fuzz_profiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (
                row["id"].as_str().unwrap(),
                row["executor"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        [
            ("source_decoder_parser", "pinned_jdk_private_adapter"),
            ("contract_parser", "pinned_jdk_private_adapter"),
            ("diagnostic_normalizer", "pinned_jdk_private_adapter"),
            ("frontend_protocol", "rust_parent_validator"),
            ("resource_capture", "pinned_jdk_private_adapter"),
        ]
    );
    assert_eq!(
        corpus["upgrade_case_ids"],
        Value::Array(
            java["upgrade_cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|case| case["id"].clone())
                .collect()
        )
    );
    assert!(java["upgrade_cases"]
        .as_array()
        .unwrap()
        .iter()
        .all(|case| {
            case["expected"] == "reject_until_new_reviewed_identity_and_complete_offline_gate"
        }));
    for profile in corpus["fuzz_profiles"].as_array().unwrap() {
        let (_, hash) = states(
            profile["seed"].as_str().unwrap(),
            usize::try_from(profile["iterations"].as_u64().unwrap()).unwrap(),
        );
        assert_eq!(profile["iterations"], 32);
        assert_eq!(profile["sequence_sha256"], hash);
    }

    let record = manifest["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == CORPUS_PATH)
        .expect("T09 corpus manifest record");
    let corpus_bytes = fs::read(repository_path(CORPUS_PATH)).expect("T09 corpus bytes");
    assert_eq!(
        record["sha256"],
        sha256_raw_file_bytes(&corpus_bytes).to_hex()
    );
    assert!(record["implementation_test_owners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|owner| owner == OWNER));
    for path in [
        "develop/specs/vectors/java-profile-v0.json",
        "develop/specs/vectors/semantic-profile-registry-v3.json",
    ] {
        let record = manifest["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["path"] == path)
            .unwrap();
        assert!(record["implementation_test_owners"]
            .as_array()
            .unwrap()
            .iter()
            .any(|owner| owner == OWNER));
    }

    let installed = registry();
    let candidate = candidate_registry();
    assert!(installed.lookup("java", "mpk.java.scalar.v0").is_none());
    assert!(candidate.lookup("java", "mpk.java.scalar.v0").is_some());
    assert!(lookup_strategy_registration("payment-policy-java-alpha").is_none());
    assert!(!repository_path("release/bundles/candidates/java.json").exists());

    let revision2 = checked_in_json("develop/specs/vectors/semantic-profile-registry-v2.json");
    let revision3 = checked_in_json("develop/specs/vectors/semantic-profile-registry-v3.json");
    let current = revision3["registry"]["profiles"].as_array().unwrap();
    for predecessor in revision2["registry"]["profiles"].as_array().unwrap() {
        assert!(current.contains(predecessor), "revision-2 entry changed");
    }
    assert_eq!(
        current.len(),
        revision2["registry"]["profiles"].as_array().unwrap().len() + 1
    );

    for (id, field, pointer, replacement) in [
        (
            "jdk",
            ProfileContractField::Release,
            "/value/toolchain_inputs_sha256",
            json!("0".repeat(64)),
        ),
        (
            "compiler",
            ProfileContractField::Release,
            "/value/compiler_profile_id",
            json!("unreviewed"),
        ),
        (
            "reference",
            ProfileContractField::Release,
            "/value/system_modules_profile_id",
            json!("unreviewed"),
        ),
        (
            "launcher",
            ProfileContractField::Frontend,
            "/value/launcher_profile_id",
            json!("unreviewed"),
        ),
        (
            "environment",
            ProfileContractField::Frontend,
            "/value/environment_profile_id",
            json!("unreviewed"),
        ),
        (
            "native",
            ProfileContractField::Release,
            "/value/execution_host_profile_id",
            json!("unreviewed"),
        ),
        (
            "public_api",
            ProfileContractField::Frontend,
            "/value/private_driver",
            json!("raw"),
        ),
        (
            "semantics",
            ProfileContractField::Vir,
            "/value/operation_profile_id",
            json!("unreviewed"),
        ),
        (
            "axioms",
            ProfileContractField::Policy,
            "/value/axiom_profile",
            json!("java-source"),
        ),
    ] {
        let mut envelope = profile_contract("java", field.as_str());
        *envelope.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            validate_compiled_profile_envelope(&candidate, &envelope, field).is_err(),
            "unreviewed {id} upgrade admitted"
        );
    }

    let mut diagnostic_change = java.clone();
    diagnostic_change["diagnostic_normalization"]["compiler_code_allowlist"]
        .as_array_mut()
        .unwrap()
        .push(json!("compiler.err.unreviewed"));
    assert_ne!(
        sha256_raw_file_bytes(&serde_json::to_vec(&diagnostic_change).unwrap()).to_hex(),
        manifest["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["path"] == "develop/specs/vectors/java-profile-v0.json")
            .unwrap()["sha256"]
    );
    assert!(
        validate_registry_semantic_context(&installed, &java["semantic_context_fixture"]).is_err()
    );
    let private_candidate = fs::read(repository_path(
        "release/build-inputs/java/bundle-candidate.json",
    ))
    .expect("private Java candidate");
    let private_candidate_value: Value = serde_json::from_slice(&private_candidate).unwrap();
    assert_eq!(
        private_candidate_value["frontend_bundles"][0]["bundle_id"],
        corpus["candidate_upgrade"]["to_frontend_bundle_id"]
    );
    assert_eq!(
        private_candidate_value["frontend_bundles"][0]["main"]["binary_sha256"],
        corpus["candidate_upgrade"]["to_jar_sha256"]
    );
    assert_ne!(
        private_candidate_value["frontend_bundles"][0]["bundle_id"],
        corpus["candidate_upgrade"]["from_frontend_bundle_id"]
    );
    let toolchain_files = private_candidate_value["toolchain_bundles"][0]["inventory"]["files"]
        .as_array()
        .unwrap();
    let parent_verified_large_bytes = ["jdk/lib/modules", "jdk/lib/server/libjvm.so"]
        .iter()
        .map(|path| {
            toolchain_files
                .iter()
                .find(|record| record["path"] == *path)
                .unwrap()["size_bytes"]
                .as_u64()
                .unwrap()
        })
        .sum::<u64>();
    assert_eq!(
        parent_verified_large_bytes,
        corpus["candidate_upgrade"]["removed_child_rehash_bytes"]
            .as_u64()
            .unwrap()
    );
    let runtime_preflight = fs::read_to_string(repository_path(
        "java-tools/java2vir/src/mpk/java2vir/RuntimePreflight.java",
    ))
    .unwrap();
    assert!(!runtime_preflight.contains("Path.of(\"/mpk/toolchain/jdk/lib/modules\")"));
    assert!(!runtime_preflight.contains("Path.of(\"/mpk/toolchain/jdk/lib/server/libjvm.so\")"));
    validate_successor_bundle_candidate(&private_candidate, &candidate)
        .expect("candidate under revision 3");
    assert!(validate_successor_bundle_candidate(&private_candidate, &installed).is_err());

    let categories = [
        AxiomCategory::CoreAxiom,
        AxiomCategory::BuiltinTheoryAxiom,
        AxiomCategory::GoSemanticsAxiom,
        AxiomCategory::ExternalAxiom,
    ]
    .map(AxiomCategory::canonical_name);
    assert_eq!(
        categories,
        [
            "CoreAxiom",
            "BuiltinTheoryAxiom",
            "GoSemanticsAxiom",
            "ExternalAxiom"
        ]
    );
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs two isolated offline builds"]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn pinned_t09_release_rehearsal_builds_and_runs_twice() {
    fn run() -> Vec<u8> {
        let output = Command::new(repository_path("scripts/check-java-frontend.sh"))
            .arg("--run-release")
            .current_dir(repository_path(""))
            .env("JAVA_HOME", "/unselected/jdk")
            .env("CLASSPATH", "/unselected/classes")
            .env("JAVA_TOOL_OPTIONS", "-javaagent:/unselected.jar")
            .env("JDK_JAVA_OPTIONS", "--patch-module=java.base=/unselected")
            .env("JDK_JAVAC_OPTIONS", "-processor unselected.Processor")
            .env("_JAVA_OPTIONS", "-Xmx1m")
            .output()
            .expect("run private T09 release rehearsal");
        assert!(
            output.status.success(),
            "T09 release rehearsal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).expect("T09 canonical report");
        assert_eq!(canonical_line(&value), output.stdout);
        output.stdout
    }

    let first = run();
    let second = run();
    assert_eq!(first, second, "two isolated build/run reports differ");
    let report: Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(report["schema"], "mpk.java.t09.private_run.v0");
    assert_eq!(
        report["candidate_inventory"]["frontend_files"][0]["sha256"],
        "aeddb537d396bc7374390d5d01c4dc576c1975e2244dcf7a64de5757fd921558"
    );
    assert_eq!(
        report["rehearsal"]["differential"]
            .as_array()
            .unwrap()
            .len(),
        102
    );
    assert_eq!(
        report["rehearsal"]["upgrade_case_ids"]
            .as_array()
            .unwrap()
            .len(),
        12
    );

    let corpus = checked_in_json(CORPUS_PATH);
    let protocol = corpus["fuzz_profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["id"] == "frontend_protocol")
        .unwrap();
    let (states, hash) = states(protocol["seed"].as_str().unwrap(), 32);
    assert_eq!(protocol["sequence_sha256"], hash);
    let case = java_lowering::case(&report["lowering"], "accepted/int.identity");
    let envelope = java_lowering::envelope(case);
    let request = java_lowering::Request::new(&envelope["selection"]);
    let captured = java_lowering::captured(case);
    for (index, state) in states.into_iter().enumerate() {
        let mut bytes = case["envelope"].as_str().unwrap().as_bytes().to_vec();
        match state % 4 {
            0 => {
                bytes.pop();
            }
            1 => {
                let offset = bytes.len() / 2;
                bytes[offset] ^= 1;
            }
            2 => {
                let mut changed = envelope.clone();
                changed["unreviewed"] = json!(index);
                bytes = canonical_line(&changed);
            }
            _ => {
                let mut changed = envelope.clone();
                changed["semantic_context"]["profile_registry"]["registry_sha256"] =
                    json!("0".repeat(64));
                bytes = canonical_line(&changed);
            }
        }
        assert!(
            request.validate(&bytes, 0, &captured).is_err(),
            "protocol fuzz case {index}"
        );
    }

    println!(
        "JAVA_T09_RECEIPT {}",
        serde_json::to_string(&json!({
            "schema":"mpk.java.t09.receipt.v0",
            "private_report_sha256":sha256_raw_file_bytes(&first).to_hex(),
            "candidate_jar_sha256":report["candidate_inventory"]["frontend_files"][0]["sha256"],
            "builds":2,
            "runs":2,
            "differential_cases":102,
            "vir_evaluations":report["lowering"]["evaluation_count"],
            "fuzz_cases":160,
            "upgrade_cases":12,
            "axiom_categories":4,
            "java_axioms":0,
            "public_activation":false,
            "native_rehearsal":"separate_official_gate_required"
        }))
        .unwrap()
    );
}
