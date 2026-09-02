use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::ReleaseRegistryIdentity;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROFILE_PATH: &str = "develop/specs/vectors/csharp-profile-v0.json";
const REGISTRY_PATH: &str = "release/bundles/semantic-profile-registry.json";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
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
fn aggregate_owner_is_pinned_and_the_active_release_registers_csharp() {
    let root = repository_root();
    let project = fs::read_to_string(root.join("csharp-tools/csharp2vir/csharp2vir.csproj"))
        .expect("read C# project");
    for input in [
        "FrontendDiagnostics.cs",
        "FrontendLimits.cs",
        "FrontendProtocol.cs",
    ] {
        assert!(project.contains(input), "missing project input {input}");
    }

    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read C# program");
    assert!(program.contains("CSharpFrontendFailureEmitter.Emit"));
    assert!(program.contains("FrontendLimits.ValidateArguments"));
    assert!(!program.contains("T11 owns successor protocol serialization"));

    let harness =
        fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_frontend_vectors_harness.cs"))
            .expect("read aggregate C# harness");
    for owner in [
        "ExecuteAccepted",
        "ExecuteRejected",
        "ValidateDiagnosticRegistry",
        "ValidateDiagnosticNormalization",
        "ValidateLimits",
        "ValidatePrecedence",
        "ValidateHashes",
        "ValidateSemanticRows",
        "ValidateRequiredChecks",
        "ValidateFailureEnvelope",
    ] {
        assert!(harness.contains(owner), "missing aggregate owner {owner}");
    }

    let build = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    for owner in [
        "validate_frontend_vector_implementation",
        "run_frontend_vector_tests=True",
        "csharp2vir-frontend-vector-tests.dll",
        "argv == [\"test-frontend-vectors\"]",
        "argv == [\"emit-frontend-vector-report\"]",
    ] {
        assert!(build.contains(owner), "missing build owner {owner}");
    }

    let manifest = load("develop/specs/vectors/manifest.json");
    let csharp = array(&manifest["vectors"])
        .iter()
        .find(|record| record["path"] == PROFILE_PATH)
        .expect("C# vector manifest record");
    assert!(array(&csharp["implementation_test_owners"])
        .iter()
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_frontend_vectors.rs"));

    let build_input_self_test = Command::new(root.join("scripts/build-csharp-frontend.sh"))
        .arg("--self-test")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute C# build-input mutation owner");
    assert_eq!(build_input_self_test.status.code(), Some(0));
    assert!(build_input_self_test.stdout.is_empty());
    assert!(build_input_self_test.stderr.is_empty());

    let active_registry = fs::read_to_string(root.join("release/bundles/bundle-registry.json"))
        .expect("read active bundle registry");
    assert!(active_registry.contains("csharp2vir"));
    assert!(active_registry.contains("mpk.csharp.scalar.v0"));

    let release_script = fs::read_to_string(root.join("scripts/build-release-bundles.sh"))
        .expect("read release assembler route");
    assert!(release_script.contains("2:--check:successor|"));
    assert!(release_script.contains("successor_release_bundles.py\" check"));
}

#[test]
fn pinned_candidate_executes_every_frontend_owned_vector() {
    let profile = load(PROFILE_PATH);
    assert_eq!(array(&profile["accepted_cases"]).len(), 30);
    assert_eq!(array(&profile["rejected_cases"]).len(), 88);
    assert_eq!(array(&profile["diagnostic_registry"]).len(), 44);
    assert_eq!(array(&profile["precedence_cases"]).len(), 12);
    assert_eq!(array(&profile["limit_cases"]).len(), 32);
    assert_eq!(array(&profile["hash_cases"]).len(), 5);
    assert_eq!(array(&profile["semantic_rows"]).len(), 34);
    if !cfg!(target_os = "linux") {
        return;
    }
    if !complete_archive_cache(&profile) {
        return;
    }

    let output = Command::new(repository_root().join("scripts/build-csharp-frontend.sh"))
        .arg("--emit-frontend-vector-report")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# aggregate harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));
    let report: Value = serde_json::from_slice(&output.stdout).expect("parse execution report");
    assert_eq!(
        text(&report["schema"]),
        "mpk.csharp.frontend_vector_execution.v0"
    );
    assert_lower_sha256(&report["normalization_sha256"], "normalization hash");

    assert_accepted_report(&profile, &report);
    assert_rejected_report(&profile, &report);
    assert_eq!(
        report["diagnostic_registry"],
        profile["diagnostic_registry"]
    );
    assert_limit_report(&profile, &report);
    assert_precedence_report(&profile, &report);
    assert_hash_report(&profile, &report);
    assert_semantic_row_report(&profile, &report);
    assert_failure_envelopes_pass_the_shared_protocol(&profile, &report);
}

fn assert_accepted_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["accepted_cases"]);
    let results = array(&report["accepted"]);
    assert_eq!(results.len(), vectors.len());
    let mut ids = BTreeSet::new();
    for (vector, result) in vectors.iter().zip(results) {
        assert_eq!(result["id"], vector["id"]);
        assert_eq!(result["status"], vector["expect"]["status"]);
        assert_eq!(result["phase"], vector["expect"]["phase"]);
        assert_eq!(result["code"], vector["expect"]["code"]);
        assert!(ids.insert(text(&result["id"])));
        for field in [
            "envelope_sha256",
            "vir_sha256",
            "source_map_sha256",
            "source_manifest_sha256",
        ] {
            assert_lower_sha256(&result[field], &format!("{} {field}", text(&result["id"])));
        }
    }
    assert_eq!(ids.len(), 30);
}

fn assert_rejected_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["rejected_cases"]);
    let results = array(&report["rejected"]);
    assert_eq!(results.len(), vectors.len());
    let mut ids = BTreeSet::new();
    for (vector, result) in vectors.iter().zip(results) {
        let expectation = &vector["expect"];
        assert_eq!(result["id"], vector["id"]);
        assert_eq!(result["status"], expectation["status"]);
        assert_eq!(result["phase"], expectation["phase"]);
        assert_eq!(result["code"], expectation["code"]);
        assert_eq!(result["artifact_free"], true);
        assert_eq!(
            integer(&result["exit"]),
            match text(&result["status"]) {
                "frontend-error" => 1,
                "rejected" => 3,
                "source-error" => 4,
                status => panic!("unexpected status {status}"),
            }
        );
        assert!(!text(&result["owner"]).is_empty());
        assert!(ids.insert(text(&result["id"])));
        let envelope = object(&result["envelope"]);
        for forbidden in ["ir", "source_map", "source_manifest"] {
            assert!(!envelope.contains_key(forbidden));
        }
        let mut transport = serde_json::to_vec(&result["envelope"]).unwrap();
        transport.push(b'\n');
        assert_eq!(sha256(&transport), text(&result["envelope_sha256"]));
    }
    assert_eq!(ids.len(), 88);
}

fn assert_limit_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["limit_cases"]);
    let results = array(&report["limits"]);
    assert_eq!(results.len(), vectors.len());
    for (vector, result) in vectors.iter().zip(results) {
        assert_eq!(result["id"], vector["id"]);
        assert_eq!(result["maximum"], vector["maximum"]);
        assert_eq!(result["code"], vector["code"]);
        let expected_status = if text(&vector["boundary_plus_one"]).starts_with("frontend_error_") {
            "frontend-error"
        } else {
            "rejected"
        };
        assert_eq!(text(&result["plus_one_status"]), expected_status);
    }
}

fn assert_precedence_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["precedence_cases"]);
    let results = array(&report["precedence"]);
    assert_eq!(results.len(), vectors.len());
    for (vector, result) in vectors.iter().zip(results) {
        assert_eq!(result["id"], vector["id"]);
        assert_eq!(result["winner"], vector["winner"]);
    }
}

fn assert_hash_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["hash_cases"]);
    let results = array(&report["hashes"]);
    assert_eq!(results.len(), vectors.len());
    for (vector, result) in vectors.iter().zip(results) {
        assert_eq!(result["id"], vector["id"]);
        assert_eq!(
            result["payload_utf8_length"],
            vector["expected_payload_utf8_length"]
        );
        assert_eq!(
            result["preimage_length"],
            vector["expected_preimage_length"]
        );
        assert_eq!(result["sha256"], vector["expected_sha256"]);
    }
}

fn assert_semantic_row_report(profile: &Value, report: &Value) {
    let vectors = array(&profile["semantic_rows"]);
    let results = array(&report["semantic_rows"]);
    assert_eq!(results.len(), vectors.len());
    for (vector, result) in vectors.iter().zip(results) {
        assert_eq!(result["row"], vector["row"]);
        assert_eq!(result["disposition"], vector["disposition"]);
    }
}

fn assert_failure_envelopes_pass_the_shared_protocol(profile: &Value, report: &Value) {
    let registry_vectors = load(REGISTRY_PATH);
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&registry_vectors["registry"])
            .expect("canonical revision-3 registry transport"),
        RegistryRevision::Revision3,
    )
    .expect("active revision-3 registry validates");
    let first = &report["rejected"][0]["envelope"];
    assert_eq!(
        first["selection"],
        profile["case_harness"]["baseline_selection"]
    );
    assert_eq!(
        first["semantic_context"]["semantic_parameters"],
        profile["semantic_parameters"]
    );
    let semantic_context =
        validate_registry_semantic_context(&registry, &first["semantic_context"])
            .expect("reported semantic context");
    let selection =
        validate_registry_selection_envelope(&registry, &semantic_context, &first["selection"])
            .expect("reported baseline selection");
    let release_registry: ReleaseRegistryIdentity = serde_json::from_value(json!({
        "schema": "mpk.release.registry.v1",
        "id": "mpk.release.registry.v1",
        "registry_sha256": "2222222222222222222222222222222222222222222222222222222222222222"
    }))
    .expect("release registry identity");
    let request = SuccessorFrontendProtocolRequest {
        registry: &registry,
        semantic_context: &semantic_context,
        selection: &selection,
        release_registry: &release_registry,
        captured_inputs: &[],
        synthetic_permissions: &[],
    };

    for result in array(&report["rejected"]) {
        assert_eq!(
            result["envelope"]["semantic_context"],
            first["semantic_context"]
        );
        assert_eq!(result["envelope"]["selection"], first["selection"]);
        let mut stdout = serde_json::to_vec(&result["envelope"]).unwrap();
        stdout.push(b'\n');
        let accepted = validate_successor_frontend_process(
            request,
            FrontendProcessFacts {
                exit_code: Some(integer(&result["exit"]) as i32),
                signaled: false,
                stdout: &stdout,
                stderr_observed_bytes: 0,
            },
        )
        .unwrap_or_else(|error| panic!("{}: {error}", text(&result["id"])));
        assert_eq!(accepted.status(), text(&result["status"]));
        assert_eq!(accepted.phase(), text(&result["phase"]));
        assert!(accepted.artifacts().is_none());
    }
}
