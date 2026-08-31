//! T04: independent failure transport and the private pinned-JDK adapter executor.
//! No installed Java tuple or public source-processing route is enabled here.

use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
    SelectionEnvelope, SemanticContext, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{CapturedInput, ReleaseRegistryIdentity};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const PROFILE: &[u8] = include_bytes!("../../../develop/specs/vectors/java-profile-v0.json");
const REGISTRY: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v3.json");
const PHASES: [&str; 7] = [
    "capture",
    "source",
    "metadata",
    "typecheck",
    "subset",
    "lowering",
    "emission",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn profile() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_slice(PROFILE).unwrap())
}
fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}

struct Request {
    registry: ValidatedSemanticProfileRegistry,
    context: SemanticContext,
    selection: SelectionEnvelope,
    release: ReleaseRegistryIdentity,
}

impl Request {
    fn new(selection: &Value) -> Self {
        let vector: Value = serde_json::from_slice(REGISTRY).unwrap();
        let registry = validate_semantic_profile_registry(
            &canonical_registry_transport(&vector["registry"]).unwrap(),
            RegistryRevision::Revision3,
        )
        .unwrap();
        let context =
            validate_registry_semantic_context(&registry, &profile()["semantic_context_fixture"])
                .unwrap();
        let selection =
            validate_registry_selection_envelope(&registry, &context, selection).unwrap();
        Self {
            registry,
            context,
            selection,
            release: ReleaseRegistryIdentity {
                schema: mpk_vc::successor_source_artifacts::SUCCESSOR_RELEASE_REGISTRY_SCHEMA
                    .into(),
                id: mpk_vc::successor_source_artifacts::SUCCESSOR_RELEASE_REGISTRY_ID.into(),
                registry_sha256: "0".repeat(64),
            },
        }
    }
    fn validate(&self, bytes: &[u8], exit: i32, captured: &[CapturedInput<'_>]) -> bool {
        validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &self.registry,
                semantic_context: &self.context,
                selection: &self.selection,
                release_registry: &self.release,
                captured_inputs: captured,
                synthetic_permissions: &[],
            },
            FrontendProcessFacts {
                exit_code: Some(exit),
                signaled: false,
                stdout: bytes,
                stderr_observed_bytes: 0,
            },
        )
        .is_ok_and(|accepted| accepted.artifacts().is_none())
    }
}

fn envelope(definition: &Value, phase: &str) -> Value {
    let issue = json!({"code":definition["code"], "message":definition["message"]});
    let rejected = definition["status"] == "rejected";
    json!({
        "schema":"mpk.frontend.cli.v1", "status":definition["status"], "phase":phase,
        "semantic_context":profile()["semantic_context_fixture"],
        "selection":profile()["selection_fixture"],
        "rejected_features":if rejected {vec![issue.clone()]} else {vec![]},
        "diagnostics":if rejected {vec![]} else {vec![issue]}
    })
}

#[test]
fn every_java_diagnostic_has_exact_code_phase_status_message_and_exit() {
    let vectors = profile();
    let request = Request::new(&vectors["selection_fixture"]);
    for definition in vectors["diagnostic_registry"].as_array().unwrap() {
        let owner = definition["phase"].as_str().unwrap();
        let exit = definition["exit"].as_i64().unwrap() as i32;
        for phase in PHASES.into_iter().chain(["release", "unknown"]) {
            let value = envelope(definition, phase);
            let valid = PHASES.contains(&phase) && (owner == "started_phase" || phase == owner);
            assert_eq!(
                request.validate(&canonical_line(&value), exit, &[]),
                valid,
                "{} / {phase}",
                definition["code"]
            );
            if !valid {
                continue;
            }
            let branch = if definition["status"] == "rejected" {
                "rejected_features"
            } else {
                "diagnostics"
            };
            for (field, replacement) in [
                (
                    "code",
                    json!(format!("{}_UNKNOWN", definition["code"].as_str().unwrap())),
                ),
                ("code", json!("SOURCE_PARSE")),
                ("code", json!("CSHARP_SOURCE_PARSE")),
                (
                    "message",
                    json!("compiler prose, identifiers, and snippets are forbidden"),
                ),
                ("message", json!("Java source is invalid ")),
            ] {
                let mut changed = value.clone();
                changed[branch][0][field] = replacement;
                assert!(!request.validate(&canonical_line(&changed), exit, &[]));
            }
            for other_exit in [0, 1, 2, 3, 4, 5]
                .into_iter()
                .filter(|other| *other != exit)
            {
                assert!(!request.validate(&canonical_line(&value), other_exit, &[]));
            }
            for field in ["ir", "source_map", "source_manifest", "partial_artifacts"] {
                let mut changed = value.clone();
                changed[field] = json!({});
                assert!(!request.validate(&canonical_line(&changed), exit, &[]));
            }
        }
    }
}

#[test]
fn java_failure_transport_is_bounded_canonical_and_identity_linked() {
    let vectors = profile();
    let definition = vectors["diagnostic_registry"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "JAVA_SOURCE_PARSE")
        .unwrap();
    let request = Request::new(&vectors["selection_fixture"]);
    let value = envelope(definition, "source");
    assert!(request.validate(&canonical_line(&value), 4, &[]));
    let mut missing_lf = canonical_line(&value);
    missing_lf.pop();
    assert!(!request.validate(&missing_lf, 4, &[]));
    let mut extra_lf = canonical_line(&value);
    extra_lf.push(b'\n');
    assert!(!request.validate(&extra_lf, 4, &[]));
    for (pointer, replacement) in [
        ("/semantic_context/source_language", json!("go")),
        ("/semantic_context/profile_registry/revision", json!(2)),
        ("/selection/value/compilation", json!("changed")),
        ("/status", json!("ir-lowered")),
        ("/diagnostics", json!([])),
    ] {
        let mut changed = value.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(!request.validate(&canonical_line(&changed), 4, &[]));
    }
    let mut count = value.clone();
    count["diagnostics"] = json!(vec![value["diagnostics"][0].clone(); 1024]);
    assert!(request.validate(&canonical_line(&count), 4, &[]));
    count["diagnostics"]
        .as_array_mut()
        .unwrap()
        .push(value["diagnostics"][0].clone());
    assert!(!request.validate(&canonical_line(&count), 4, &[]));
    let mut wrong_order = value.clone();
    let mut late = value["diagnostics"][0].clone();
    late["span"] = json!({"normalized_path":"src/demo/Policy.java", "start":10, "end":11});
    wrong_order["diagnostics"] = json!([late, value["diagnostics"][0]]);
    assert!(!request.validate(&canonical_line(&wrong_order), 4, &[]));
}

fn command() -> Command {
    let mut command = Command::new(root().join("scripts/check-java-frontend.sh"));
    command
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("HOME", "/unselected")
        .env("TMPDIR", "/unselected")
        .env("JAVA_HOME", "/unselected/jdk")
        .env("CLASSPATH", "/unselected.jar")
        .env("JAVA_TOOL_OPTIONS", "-javaagent:/unselected.jar")
        .env("JDK_JAVA_OPTIONS", "--patch-module=java.base=/unselected")
        .env("JDK_JAVAC_OPTIONS", "-processor unselected.Processor")
        .env("_JAVA_OPTIONS", "-Xmx1m")
        .env("DOCKER_HOST", "tcp://127.0.0.1:1");
    command
}

#[test]
fn private_adapter_fixture_ownership_is_closed_and_requires_an_explicit_run() {
    let output = command().arg("--check-fixtures").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    for arguments in [
        vec![],
        vec!["--run", "--class-path", "/unselected"],
        vec!["--worker"],
    ] {
        let output = command().args(arguments).output().unwrap();
        assert_eq!(output.status.code(), Some(64));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"JAVA_FRONTEND_TEST_USAGE\n");
    }
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn pinned_java_capture_compiler_and_diagnostic_vectors_execute() {
    let output = command().arg("--run").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(canonical_line(&report), output.stdout);
    assert_eq!(report["schema"], "mpk.java.frontend_tests.v0");
    let inventory: Value = serde_json::from_slice(
        &std::fs::read(root().join("release/build-inputs/java/candidate-inventory.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["candidate_inventory"], inventory);
    assert_eq!(
        report["lowering_precedence"],
        json!({
            "contract_before_lowering":"T06: --run-lowering",
            "map_failure_prevents_partial_output":"T06: --run-lowering"
        })
    );
    assert_eq!(
        report["follow_on_precedence"],
        json!({"release_before_source":"T07"})
    );
    for case in report["failures"].as_array().unwrap() {
        let envelope: Value = serde_json::from_str(case["envelope"].as_str().unwrap()).unwrap();
        let request = Request::new(&envelope["selection"]);
        assert!(
            request.validate(
                case["envelope"].as_str().unwrap().as_bytes(),
                case["exit"].as_i64().unwrap() as i32,
                &[],
            ),
            "{}",
            case["id"]
        );
    }
}
