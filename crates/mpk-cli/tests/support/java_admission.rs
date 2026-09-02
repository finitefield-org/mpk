use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::ReleaseRegistryIdentity;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn profile() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../../develop/specs/vectors/java-profile-v0.json"
    ))
    .unwrap()
}

fn command() -> Command {
    let mut command = Command::new("/usr/bin/python3");
    command
        .args(["-I", "-S", "-B"])
        .arg(root().join("scripts/java_frontend_tests.py"))
        .current_dir(root())
        .env("JAVA_HOME", "/unselected/jdk")
        .env("CLASSPATH", "/unselected/classes")
        .env("JAVA_TOOL_OPTIONS", "-javaagent:/unselected.jar")
        .env("JDK_JAVA_OPTIONS", "--patch-module=java.base=/unselected")
        .env("JDK_JAVAC_OPTIONS", "-processor unselected.Processor")
        .env("_JAVA_OPTIONS", "-Xmx1m");
    command
}

pub fn check_fixtures() {
    let output = command().arg("check-admission-fixtures").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

pub fn run() -> Value {
    let output = command().arg("run-admission").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut canonical = serde_json::to_vec(&report).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, output.stdout);
    assert_eq!(report["schema"], "mpk.java.admission_tests.v0");
    let inventory: Value = serde_json::from_slice(
        &std::fs::read(root().join("release/build-inputs/java/candidate-inventory.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["candidate_inventory"], inventory);

    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../../../develop/specs/vectors/semantic-profile-registry-v3.json"
    ))
    .unwrap();
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&vectors["registry"]).unwrap(),
        RegistryRevision::Revision3,
    )
    .unwrap();
    let context =
        validate_registry_semantic_context(&registry, &profile()["semantic_context_fixture"])
            .unwrap();
    let release = ReleaseRegistryIdentity {
        schema: mpk_vc::successor_source_artifacts::SUCCESSOR_RELEASE_REGISTRY_SCHEMA.into(),
        id: mpk_vc::successor_source_artifacts::SUCCESSOR_RELEASE_REGISTRY_ID.into(),
        registry_sha256: "0".repeat(64),
    };
    for case in report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case.get("envelope").is_some())
        .chain(std::iter::once(&report["link_failure"]))
    {
        let bytes = case["envelope"].as_str().unwrap().as_bytes();
        let envelope: Value = serde_json::from_slice(bytes).unwrap();
        let selection =
            validate_registry_selection_envelope(&registry, &context, &envelope["selection"])
                .unwrap_or_else(|error| panic!("Java fixture selection {}: {error:?}", case["id"]));
        let accepted = validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &registry,
                semantic_context: &context,
                selection: &selection,
                release_registry: &release,
                captured_inputs: &[],
                synthetic_permissions: &[],
            },
            FrontendProcessFacts {
                exit_code: Some(case["exit"].as_i64().unwrap().try_into().unwrap()),
                signaled: false,
                stdout: bytes,
                stderr_observed_bytes: 0,
            },
        )
        .unwrap_or_else(|error| panic!("Java rejection {}: {error:?}", case["id"]));
        assert!(accepted.artifacts().is_none());
    }
    report
}
