use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
    SelectionEnvelope, SemanticContext, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{CapturedInput, InputKind, ReleaseRegistryIdentity};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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
    let mut command = Command::new(root().join("scripts/check-java-frontend.sh"));
    command
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
    let output = command().arg("--check-lowering-fixtures").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}
pub fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
pub fn captured(case: &Value) -> Vec<CapturedInput<'_>> {
    case["captured_inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| CapturedInput {
            kind: if input["kind"] == "source" {
                InputKind::Source
            } else {
                InputKind::Contract
            },
            normalized_path: input["path"].as_str().unwrap(),
            bytes: input["text"].as_str().unwrap().as_bytes(),
        })
        .collect()
}
pub struct Request {
    registry: ValidatedSemanticProfileRegistry,
    context: SemanticContext,
    selection: SelectionEnvelope,
    release: ReleaseRegistryIdentity,
}
impl Request {
    pub fn new(selection: &Value) -> Self {
        let vector: Value = serde_json::from_slice(include_bytes!(
            "../../../../develop/specs/vectors/semantic-profile-registry-v3.json"
        ))
        .unwrap();
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
    pub fn validate(
        &self,
        bytes: &[u8],
        exit: i32,
        inputs: &[CapturedInput<'_>],
    ) -> Result<bool, String> {
        validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &self.registry,
                semantic_context: &self.context,
                selection: &self.selection,
                release_registry: &self.release,
                captured_inputs: inputs,
                synthetic_permissions: &[],
            },
            FrontendProcessFacts {
                exit_code: Some(exit),
                signaled: false,
                stdout: bytes,
                stderr_observed_bytes: 0,
            },
        )
        .map(|result| result.artifacts().is_some())
        .map_err(|error| format!("{error:?}"))
    }
}
pub fn run() -> &'static Value {
    static REPORT: OnceLock<Value> = OnceLock::new();
    REPORT.get_or_init(|| {
        let output = command().arg("--run-lowering").output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(canonical_line(&report), output.stdout);
        assert_eq!(report["schema"], "mpk.java.lowering_tests.v0");
        let inventory: Value = serde_json::from_slice(
            &std::fs::read(root().join("release/build-inputs/java/candidate-inventory.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(report["candidate_inventory"], inventory);
        for case in report["cases"]
            .as_array()
            .unwrap()
            .iter()
            .chain(report["mutations"].as_array().unwrap())
        {
            let bytes = case["envelope"].as_str().unwrap().as_bytes();
            let envelope: Value = serde_json::from_slice(bytes).unwrap();
            assert_eq!(canonical_line(&envelope), bytes);
            let request = Request::new(&envelope["selection"]);
            let inputs = if case.get("captured_inputs").is_some() {
                captured(case)
            } else {
                vec![]
            };
            let exit = i32::try_from(case["exit"].as_i64().unwrap()).unwrap();
            let artifacts = request
                .validate(bytes, exit, &inputs)
                .unwrap_or_else(|error| panic!("{}: {error}", case["id"]));
            assert_eq!(artifacts, exit == 0);
        }
        report
    })
}
pub fn case<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
}
pub fn envelope(case: &Value) -> Value {
    serde_json::from_str(case["envelope"].as_str().unwrap()).unwrap()
}
fn rehash(value: &mut Value, field: &str, domain: &str) {
    value.as_object_mut().unwrap().remove(field);
    let mut digest = Sha256::new();
    digest.update(domain.as_bytes());
    digest.update([0]);
    digest.update(serde_json::to_vec(value).unwrap());
    value[field] = format!("{:x}", digest.finalize()).into();
}
/// Rebind every dependent hash, so mutation tests exercise semantic validation.
pub fn refresh(envelope: &mut Value) {
    rehash(&mut envelope["ir"]["value"], "vir_hash", "MPK-VIR-1.0");
    let vir_hash = envelope["ir"]["value"]["vir_hash"].clone();
    envelope["ir"]["sha256"] = vir_hash.clone();
    envelope["source_map"]["source_ir_hash"] = vir_hash.clone();
    rehash(
        &mut envelope["source_map"],
        "source_map_hash",
        "MPK-SOURCE-MAP-1.0",
    );
    envelope["source_manifest"]["vir_hash"] = vir_hash;
    envelope["source_manifest"]["source_map_hash"] =
        envelope["source_map"]["source_map_hash"].clone();
    rehash(
        &mut envelope["source_manifest"],
        "source_manifest_hash",
        "MPK-SOURCE-MANIFEST-1.0",
    );
}
