use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::{validate_release_registry, ReleaseRegistryIdentity};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROFILE_PATH: &str = "develop/specs/vectors/csharp-profile-v0.json";
const PROFILE_SHA256: &str = "8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8";
const SEMANTIC_V1_PATH: &str = "develop/specs/vectors/semantic-profile-registry-v1.json";
const SEMANTIC_V3_PATH: &str = "develop/specs/vectors/semantic-profile-registry-v3.json";
const STAGED_SEMANTIC_PATH: &str = "release/bundles/semantic-profile-registry.json";
const FUZZ_MANIFEST_PATH: &str = "csharp-tools/csharp2vir/fuzz/seed-manifest.json";

const TOP_LEVEL_FIELDS: [&str; 31] = [
    "accepted_cases",
    "case_harness",
    "compiler_session",
    "contract_fixture",
    "contract_sidecar_sha256",
    "conversion_rules",
    "diagnostic_normalization",
    "diagnostic_registry",
    "hash_cases",
    "isolation_cases",
    "launcher_contract",
    "limit_cases",
    "mechanism_schema",
    "normalized_contract_fixture",
    "operation_mappings",
    "owner_test",
    "precedence_cases",
    "profile_contracts",
    "profile_identity",
    "rejected_cases",
    "roslyn_checked_state_cases",
    "schema",
    "selection_fixture",
    "selection_sha256",
    "semantic_parameters",
    "semantic_rows",
    "source_map_cases",
    "spec_schema",
    "toolchain_inputs",
    "type_mappings",
    "upgrade_cases",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative)).expect("read repository input")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative)).expect("parse repository JSON")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("JSON object")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn integer(value: &Value) -> u64 {
    value.as_u64().expect("nonnegative JSON integer")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("serialize canonical JSON");
    bytes.push(b'\n');
    bytes
}

fn assert_lower_sha256(value: &Value, label: &str) {
    let value = text(value);
    assert_eq!(value.len(), 64, "{label}");
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{label}"
    );
}

fn complete_archive_cache(profile: &Value) -> bool {
    let hash = text(&profile["toolchain_inputs"]["toolchain_inputs_sha256"]);
    let cache = repository_root()
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = array(&profile["toolchain_inputs"]["archives"]);
    let present = archives
        .iter()
        .filter(|archive| {
            let suffix = match text(&archive["kind"]) {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!("{}{}", text(&archive["id"]), suffix))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial C# archive cache"
    );
    present == archives.len()
}

#[test]
fn every_frozen_top_level_field_has_an_aggregate_executor() {
    let bytes = read(PROFILE_PATH);
    assert_eq!(sha256(&bytes), PROFILE_SHA256);
    let profile: Value = serde_json::from_slice(&bytes).expect("C# profile JSON");
    let observed = object(&profile)
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(observed, TOP_LEVEL_FIELDS);

    assert_eq!(profile["schema"], "mpk.csharp.profile.conformance.v0");
    assert_eq!(profile["spec_schema"], "mpk.csharp.scalar.v0");
    assert_eq!(
        profile["mechanism_schema"],
        "mpk.semantic_profile.registry.v1"
    );
    assert_eq!(
        profile["owner_test"],
        "crates/mpk-vc/tests/csharp_profile_spec.rs"
    );
    for field in ["contract_sidecar_sha256", "selection_sha256"] {
        assert_lower_sha256(&profile[field], field);
    }
    for (field, expected) in [
        ("accepted_cases", 30),
        ("conversion_rules", 20),
        ("diagnostic_registry", 44),
        ("hash_cases", 5),
        ("isolation_cases", 12),
        ("limit_cases", 32),
        ("operation_mappings", 35),
        ("precedence_cases", 12),
        ("profile_contracts", 9),
        ("rejected_cases", 88),
        ("roslyn_checked_state_cases", 12),
        ("semantic_rows", 34),
        ("source_map_cases", 6),
        ("type_mappings", 5),
        ("upgrade_cases", 12),
    ] {
        assert_eq!(array(&profile[field]).len(), expected, "{field}");
    }
    for field in [
        "case_harness",
        "compiler_session",
        "contract_fixture",
        "diagnostic_normalization",
        "launcher_contract",
        "normalized_contract_fixture",
        "profile_identity",
        "selection_fixture",
        "semantic_parameters",
        "toolchain_inputs",
    ] {
        assert!(!object(&profile[field]).is_empty(), "{field}");
    }

    let registry_vectors = load(SEMANTIC_V3_PATH);
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&registry_vectors["registry"])
            .expect("canonical revision-3 registry"),
        RegistryRevision::Revision3,
    )
    .expect("revision-3 registry validates");
    let semantic_context = json!({
        "profile_entry_sha256": profile["profile_identity"]["profile_entry_sha256"],
        "profile_registry": {
            "id": "mpk.semantic_profile.registry.v1",
            "registry_sha256": profile["profile_identity"]["registry_sha256"],
            "revision": profile["profile_identity"]["registry_revision"],
            "schema": "mpk.semantic_profile.registry.v1"
        },
        "semantic_parameters": profile["semantic_parameters"],
        "semantic_profile": profile["profile_identity"]["semantic_profile"],
        "source_language": profile["profile_identity"]["source_language"]
    });
    let context = validate_registry_semantic_context(&registry, &semantic_context)
        .expect("C# semantic context");
    validate_registry_selection_envelope(&registry, &context, &profile["selection_fixture"])
        .expect("C# selection fixture");

    let manifest = load("develop/specs/vectors/manifest.json");
    let record = array(&manifest["vectors"])
        .iter()
        .find(|record| record["path"] == PROFILE_PATH)
        .expect("C# manifest record");
    assert_eq!(record["sha256"], PROFILE_SHA256);
    assert_eq!(record["owning_spec"], "develop/specs/CSHARP_PROFILE_V0.md");
    let owners = array(&record["implementation_test_owners"]);
    assert_eq!(
        owners.last(),
        Some(&Value::String(
            "crates/mpk-cli/tests/csharp_profile_vectors.rs".into()
        ))
    );
    assert_eq!(owners.len(), 14);
}

#[test]
fn pinned_report_is_two_run_identical_and_complete() {
    let profile = load(PROFILE_PATH);
    if !cfg!(target_os = "linux") {
        return;
    }
    if !complete_archive_cache(&profile) {
        return;
    }

    let execute = || {
        Command::new(repository_root().join("scripts/build-csharp-frontend.sh"))
            .arg("--emit-frontend-vector-report")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("execute C# hardening harness")
    };
    let first = execute();
    assert!(
        first.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let second = execute();
    assert!(
        second.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(first.stderr.is_empty() && second.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout, "two C# reports differ");
    assert!(first.stdout.ends_with(b"\n"));

    let report: Value = serde_json::from_slice(&first.stdout).expect("hardening report JSON");
    assert_eq!(report["schema"], "mpk.csharp.frontend_vector_execution.v0");
    assert_eq!(array(&report["accepted"]).len(), 30);
    assert_eq!(array(&report["rejected"]).len(), 88);
    assert_eq!(array(&report["diagnostic_registry"]).len(), 44);
    assert_eq!(array(&report["hashes"]).len(), 5);
    assert_eq!(array(&report["limits"]).len(), 32);
    assert_eq!(array(&report["precedence"]).len(), 12);
    assert_eq!(array(&report["semantic_rows"]).len(), 34);

    let differential = &report["differential"];
    assert_eq!(differential["runtime_version"], "10.0.11");
    let vectors = array(&differential["vectors"]);
    assert_eq!(vectors.len(), 30);
    let case_count = integer(&differential["case_count"]);
    assert!((120..=480).contains(&case_count));
    assert_eq!(
        vectors
            .iter()
            .map(|entry| integer(&entry["case_count"]))
            .sum::<u64>(),
        case_count
    );
    for entry in vectors {
        assert_lower_sha256(&entry["outcomes_sha256"], "differential outcome hash");
    }

    let fuzz = array(&report["fuzz"]);
    assert_eq!(
        fuzz.iter()
            .map(|entry| text(&entry["id"]))
            .collect::<Vec<_>>(),
        [
            "compiler_output",
            "contract",
            "parser",
            "protocol",
            "resource"
        ]
    );
    for target in fuzz {
        assert_eq!(target["seed_count"], 2);
        assert_eq!(target["mutation_count"], 12);
        assert_lower_sha256(&target["outcomes_sha256"], "fuzz outcome hash");
    }

    let transport = String::from_utf8(first.stdout).expect("UTF-8 report");
    let lower = transport.to_ascii_lowercase();
    for forbidden in [
        "/root/",
        "/tmp/",
        "http://",
        "https://",
        "authorization",
        "api_key",
        "password",
        "secret",
    ] {
        assert!(
            !lower.contains(forbidden),
            "hardening report leaked {forbidden}"
        );
    }
}

#[test]
fn checked_in_regressions_are_closed_and_protocol_mutations_fail_safely() {
    let manifest_bytes = read(FUZZ_MANIFEST_PATH);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("fuzz manifest JSON");
    assert_eq!(canonical_line(&manifest), manifest_bytes);
    assert_eq!(manifest["schema"], "mpk.csharp.fuzz_seeds.v0");
    let targets = object(&manifest["targets"]);
    assert_eq!(
        targets.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "compiler_output",
            "contract",
            "parser",
            "protocol",
            "resource"
        ]
    );
    for (target, records) in targets {
        assert_eq!(array(records).len(), 2, "{target}");
        let mut previous = "";
        for record in array(records) {
            let relative = text(&record["path"]);
            assert!(relative > previous && !relative.contains(['/', '\\']));
            previous = relative;
            let path = format!("csharp-tools/csharp2vir/fuzz/seeds/{target}/{relative}");
            let bytes = read(&path);
            assert_eq!(bytes.len() as u64, integer(&record["size_bytes"]));
            assert_eq!(sha256(&bytes), text(&record["sha256"]));
        }
    }

    let registry = validate_semantic_profile_registry(
        &read(STAGED_SEMANTIC_PATH),
        RegistryRevision::Revision3,
    )
    .expect("staged semantic registry");
    let valid = read("csharp-tools/csharp2vir/fuzz/seeds/protocol/rejected.json");
    let envelope: Value = serde_json::from_slice(&valid).expect("valid protocol seed");
    let context = validate_registry_semantic_context(&registry, &envelope["semantic_context"])
        .expect("seed semantic context");
    let selection =
        validate_registry_selection_envelope(&registry, &context, &envelope["selection"])
            .expect("seed selection");
    let release_registry: ReleaseRegistryIdentity = serde_json::from_value(json!({
        "schema": "mpk.release.registry.v1",
        "id": "mpk.release.registry.v1",
        "registry_sha256": "2222222222222222222222222222222222222222222222222222222222222222"
    }))
    .expect("release registry identity");
    let request = SuccessorFrontendProtocolRequest {
        registry: &registry,
        semantic_context: &context,
        selection: &selection,
        release_registry: &release_registry,
        captured_inputs: &[],
        synthetic_permissions: &[],
    };
    let accepted = validate_successor_frontend_process(
        request,
        FrontendProcessFacts {
            exit_code: Some(3),
            signaled: false,
            stdout: &valid,
            stderr_observed_bytes: 0,
        },
    )
    .expect("valid checked-in protocol seed");
    assert_eq!(accepted.status(), "rejected");

    for seed in array(&targets["protocol"]) {
        let path = format!(
            "csharp-tools/csharp2vir/fuzz/seeds/protocol/{}",
            text(&seed["path"])
        );
        for mutation in deterministic_mutations(&read(&path)) {
            for exit_code in [1, 3, 4] {
                let _ = validate_successor_frontend_process(
                    request,
                    FrontendProcessFacts {
                        exit_code: Some(exit_code),
                        signaled: false,
                        stdout: &mutation,
                        stderr_observed_bytes: 0,
                    },
                );
            }
        }
    }
}

fn deterministic_mutations(seed: &[u8]) -> Vec<Vec<u8>> {
    let mut appended = seed.to_vec();
    appended.push(0);
    let mut flipped = seed.to_vec();
    if !flipped.is_empty() {
        let middle = flipped.len() / 2;
        flipped[middle] ^= 0x80;
    }
    let mut duplicated = seed.to_vec();
    duplicated.extend_from_slice(seed);
    vec![
        seed.to_vec(),
        Vec::new(),
        seed[..seed.len() / 2].to_vec(),
        appended,
        flipped,
        duplicated,
    ]
}

fn semantic_registry() -> mpk_vc::semantic_profile_registry::ValidatedSemanticProfileRegistry {
    validate_semantic_profile_registry(&read(STAGED_SEMANTIC_PATH), RegistryRevision::Revision3)
        .expect("active revision-3 semantic registry")
}

#[test]
fn all_four_profiles_reject_predecessor_and_crossed_release_bytes() {
    let semantic = semantic_registry();
    let candidates = [
        "release/bundles/candidates/go.json",
        "release/bundles/candidates/rust.json",
        "release/bundles/candidates/csharp.json",
        "release/bundles/candidates/java.json",
    ]
    .map(load);
    let registries = [
        "release/bundles/bundle-registry.json",
        "release/bundles/bundle-registry.json",
        "release/bundles/bundle-registry.json",
        "release/bundles/bundle-registry.json",
    ];
    for (index, candidate) in candidates.iter().enumerate() {
        let validated = validate_successor_bundle_candidate(&canonical_line(candidate), &semantic)
            .expect("successor candidate");
        assert_eq!(validated.candidate().tuples.len(), [1, 2, 1, 1][index]);
    }
    for path in registries {
        let bytes = read(path);
        validate_successor_release_registry(&bytes, &semantic).expect("successor registry");
        assert!(validate_release_registry(&bytes).is_err());
    }

    for candidate_index in 0..candidates.len() {
        for context_index in 0..candidates.len() {
            if candidate_index == context_index {
                continue;
            }
            let mut crossed = candidates[candidate_index].clone();
            crossed["tuples"][0]["semantic_context"] =
                candidates[context_index]["tuples"][0]["semantic_context"].clone();
            assert!(
                validate_successor_bundle_candidate(&canonical_line(&crossed), &semantic).is_err(),
                "candidate {candidate_index} accepted context {context_index}"
            );
        }
    }

    let v1_vectors = load(SEMANTIC_V1_PATH);
    let v1_bytes = canonical_registry_transport(&v1_vectors["fixtures"]["base_registry"])
        .expect("canonical revision-1 registry");
    validate_semantic_profile_registry(&v1_bytes, RegistryRevision::Revision1)
        .expect("revision-1 registry");
    assert!(validate_semantic_profile_registry(&v1_bytes, RegistryRevision::Revision2).is_err());
    assert!(validate_semantic_profile_registry(
        &read(STAGED_SEMANTIC_PATH),
        RegistryRevision::Revision1
    )
    .is_err());
}

#[test]
fn every_upgrade_class_requires_a_new_identity() {
    let profile = load(PROFILE_PATH);
    let upgrades = array(&profile["upgrade_cases"]);
    let ids = upgrades
        .iter()
        .map(|case| text(&case["id"]))
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "upgrade.sdk",
            "upgrade.runtime",
            "upgrade.roslyn",
            "upgrade.references",
            "upgrade.language",
            "upgrade.compilation_option",
            "upgrade.api",
            "upgrade.operation",
            "upgrade.cfg",
            "upgrade.diagnostic",
            "upgrade.host",
            "upgrade.payload",
        ]
    );
    for case in upgrades {
        assert!(
            case["requires_new_profile"] == true || case["requires_new_entry"] == true,
            "{}",
            text(&case["id"])
        );
        assert!(!text(&case["changed"]).is_empty());
    }

    let semantic = semantic_registry();
    let candidate = load("release/bundles/candidates/csharp.json");
    for id in ids {
        let mut changed = candidate.clone();
        apply_upgrade_mutation(id, &mut changed);
        assert_ne!(changed, candidate, "{id}");
        assert!(
            validate_successor_bundle_candidate(&canonical_line(&changed), &semantic).is_err(),
            "{id} did not require a new identity"
        );
    }
}

fn apply_upgrade_mutation(id: &str, candidate: &mut Value) {
    match id {
        "upgrade.sdk" => {
            candidate["toolchain_bundles"][0]["profile_contracts"][0]["value"]
                ["compiler_profile_id"] = json!("mpk.csharp.roslyn_5_7_0.v0");
        }
        "upgrade.runtime" => {
            candidate["toolchain_bundles"][0]["profile_contracts"][0]["value"]
                ["runtime_profile_id"] = json!("mpk.dotnet.runtime_10_0_12.linux_x64.v0");
        }
        "upgrade.roslyn" => {
            candidate["frontend_bundles"][0]["subordinate_binaries"][0]["binary_sha256"] =
                json!("0".repeat(64));
        }
        "upgrade.references" => {
            candidate["toolchain_bundles"][0]["profile_contracts"][0]["value"]
                ["reference_profile_id"] = json!("mpk.dotnet.netcore_ref_10_0_12.v0");
        }
        "upgrade.language" => {
            candidate["tuples"][0]["semantic_context"]["semantic_parameters"]["value"]
                ["language_version"] = json!("15.0");
        }
        "upgrade.compilation_option" => {
            candidate["tuples"][0]["semantic_context"]["semantic_parameters"]["value"]
                ["check_overflow_default"] = json!(true);
        }
        "upgrade.api" => {
            candidate["frontend_bundles"][0]["schema"] = json!("mpk.release.frontend_bundle.v2");
        }
        "upgrade.operation" => {
            candidate["tuples"][0]["semantic_context"]["semantic_profile"] =
                json!("mpk.csharp.scalar.v1");
        }
        "upgrade.cfg" => {
            candidate["tuples"][0]["semantic_context"]["semantic_parameters"]["value"]
                ["source_kind"] = json!("script");
        }
        "upgrade.diagnostic" => {
            candidate["frontend_bundles"][0]["profile_contracts"][0]["value"]["limit_profile_id"] =
                json!("mpk.csharp.limits.v1");
        }
        "upgrade.host" => {
            candidate["toolchain_bundles"][0]["execution_host_profile_id"] =
                json!("mpk.host.linux-x86_64-gnu.v1");
        }
        "upgrade.payload" => {
            candidate["tuples"][0]["semantic_context"]["profile_entry_sha256"] =
                json!("f".repeat(64));
        }
        _ => panic!("unexpected upgrade case {id}"),
    }
}

#[test]
fn active_release_and_frontend_sources_remain_plugin_free() {
    let active = String::from_utf8(read("release/bundles/bundle-registry.json"))
        .expect("active registry UTF-8");
    assert!(active.contains("csharp"));
    assert!(active.contains("csharp2vir"));

    let root = repository_root();
    let mut sources = fs::read_dir(root.join("csharp-tools/csharp2vir"))
        .expect("read C# candidate sources")
        .map(|entry| entry.expect("C# candidate source entry").path())
        .filter(|path| {
            path.is_file()
                && matches!(
                    path.extension().and_then(|extension| extension.to_str()),
                    Some("cs" | "csproj")
                )
        })
        .collect::<Vec<_>>();
    collect_rust_sources(&root.join("crates/mpk-cli/src"), &mut sources);
    sources.sort();
    assert!(sources.len() > 30, "closed production source inventory");

    for path in sources {
        let source = fs::read_to_string(&path).expect("candidate or production source UTF-8");
        let lower = source.to_ascii_lowercase();
        for forbidden in [
            "loadplugin",
            "pluginpath",
            "plugin_uri",
            "assemblyloadcontext",
            "assembly.load(",
            "assembly.loadfrom",
            "assembly.loadfile",
            "nativelibrary.load",
            "libloading",
            "dlopen(",
            "loadlibrary(",
        ] {
            assert!(
                !lower.contains(forbidden),
                "{} contains {forbidden}",
                path.display()
            );
        }
    }
}

fn collect_rust_sources(directory: &Path, output: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .expect("read Rust production source directory")
        .map(|entry| entry.expect("Rust production source entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        let metadata = fs::symlink_metadata(&path).expect("Rust production source metadata");
        assert!(!metadata.file_type().is_symlink(), "{}", path.display());
        if metadata.is_dir() {
            collect_rust_sources(&path, output);
        } else if metadata.is_file() && path.extension().is_some_and(|value| value == "rs") {
            output.push(path);
        }
    }
}
