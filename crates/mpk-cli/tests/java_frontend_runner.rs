//! Java launcher and bundle invariants after the atomic Revision 3 cutover.
//! Native installed execution is owned by `successor_atomic_cutover`.

use mpk_cli::successor_release_bundle::{
    successor_release_registry_hash, validate_successor_bundle_candidate,
    validate_successor_release_registry, SuccessorReleaseSelectionRequest,
};
use mpk_vc::java_release;
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_semantic_profile_registry, RegistryRevision,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(root().join(relative)).expect("read repository file")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative)).expect("parse repository JSON")
}

fn line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("serialize JSON");
    bytes.push(b'\n');
    bytes
}

#[test]
fn active_java_candidate_and_launcher_are_exact() {
    let check = Command::new(root().join("scripts/build-java-candidate.sh"))
        .arg("--check")
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("JAVA_HOME", "/host/jdk")
        .env("CLASSPATH", "/host/classes")
        .env("JAVA_TOOL_OPTIONS", "-javaagent:/host/agent.jar")
        .output()
        .expect("run Java descriptor check");
    assert!(
        check.status.success() && check.stdout.is_empty() && check.stderr.is_empty(),
        "candidate reconstruction: {}",
        String::from_utf8_lossy(&check.stderr)
    );

    let vector = load("develop/specs/vectors/java-profile-v0.json");
    let semantic = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let candidate_bytes = read("release/bundles/candidates/java.json");
    let candidate = validate_successor_bundle_candidate(&candidate_bytes, &semantic)
        .expect("active Java candidate");
    let registry = validate_successor_release_registry(
        &read("release/bundles/bundle-registry.json"),
        &semantic,
    )
    .expect("active release registry");
    assert_eq!(candidate.candidate(), java_release::candidate());
    assert_eq!(
        candidate.candidate().toolchain_bundles[0]
            .inventory
            .files
            .len(),
        405
    );

    let resolved = registry
        .resolve(
            &semantic,
            SuccessorReleaseSelectionRequest {
                semantic_context: &vector["semantic_context_fixture"],
                frontend_bundle_id: java_release::FRONTEND_ID,
                toolchain_bundle_id: java_release::TOOLCHAIN_ID,
            },
        )
        .expect("resolve installed Java tuple");
    let selection = validate_registry_selection_envelope(
        &semantic,
        &resolved.semantic_context,
        &json!({"schema":"mpk.selection.java_methods.v0", "value": {
            "compilation":"vector", "sources":["src/vector/Case.java"],
            "contracts":["contracts/selected.json"],
            "methods":["vector.Case::f(int)->int"]}}),
    )
    .expect("Java selection");
    let launcher = java_release::launcher_plan(&registry, &resolved, &selection)
        .expect("registered Java launcher");
    assert_eq!(
        json!(java_release::ARGV_PREFIX),
        vector["launcher_contract"]["argv_prefix"]
    );
    assert_eq!(
        json!(launcher.environment()),
        vector["launcher_contract"]["environment"]
    );

    let template = vector["launcher_contract"]["frontend_argv_template"]
        .as_array()
        .expect("launcher template");
    let substitutions = [
        ("{selection.compilation}", "vector"),
        (
            "{each selection.sources in stored order}",
            "src/vector/Case.java",
        ),
        (
            "{each selection.contracts in stored order}",
            "contracts/selected.json",
        ),
        (
            "{each selection.methods in stored order}",
            "vector.Case::f(int)->int",
        ),
        ("{release.frontend_bundle_id}", java_release::FRONTEND_ID),
        (
            "{release.frontend_sha256}",
            resolved.frontend.main.binary_sha256.as_str(),
        ),
        ("{release.registry_sha256}", registry.registry_sha256()),
        ("{release.toolchain_bundle_id}", java_release::TOOLCHAIN_ID),
        (
            "{release.toolchain_distribution_sha256}",
            resolved.toolchain.distribution_sha256.as_str(),
        ),
    ];
    let mut expected = java_release::ARGV_PREFIX
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    for token in template {
        let value = token.as_str().expect("launcher token");
        expected.push(
            substitutions
                .iter()
                .find(|(key, _)| *key == value)
                .map_or(value, |(_, replacement)| *replacement)
                .to_owned(),
        );
    }
    assert_eq!(launcher.argv(), expected);
}

#[test]
fn java_release_bytes_cannot_be_crossed_or_caller_replaced() {
    let semantic = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let registry_bytes = read("release/bundles/bundle-registry.json");
    for (pointer, replacement) in [
        ("/frontend_bundles/2/main/path", json!("evil.jar")),
        (
            "/frontend_bundles/2/main/binary_sha256",
            json!("0".repeat(64)),
        ),
        (
            "/toolchain_bundles/2/components/0/path",
            json!("jdk/bin/javac"),
        ),
        (
            "/toolchain_bundles/2/distribution_sha256",
            json!("1".repeat(64)),
        ),
    ] {
        let mut changed: Value = serde_json::from_slice(&registry_bytes).expect("registry JSON");
        *changed.pointer_mut(pointer).expect("mutation pointer") = replacement;
        changed["registry_sha256"] =
            json!(successor_release_registry_hash(&changed).expect("repair outer hash"));
        assert!(
            validate_successor_release_registry(&line(&changed), &semantic).is_err(),
            "{pointer}"
        );
    }
    for removed in [
        "scripts/check-java-runner.sh",
        "scripts/check-java-rehearsal.sh",
        "release/build-inputs/java/bundle-registry.json",
    ] {
        assert!(
            !root().join(removed).exists(),
            "staging entrypoint remains: {removed}"
        );
    }
}
