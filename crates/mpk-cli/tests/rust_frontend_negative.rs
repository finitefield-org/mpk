#![allow(dead_code)]

#[path = "../src/frontend_protocol.rs"]
mod frontend_protocol;

mod policy_schema {
    pub use mpk_cli::policy_schema::*;
}

mod frontend_runner {
    use crate::frontend_protocol::AcceptedFrontendEnvelope;
    use mpk_vc::{
        CapturedInput, FrontendIdentity, ReleaseRegistryIdentity, ReleaseSelectionRequest,
        ToolchainIdentity, ValidatedReleaseRegistry,
    };
    use serde_json::Value;

    pub(crate) struct FrontendRunRequest<'a> {
        pub(crate) release: ReleaseSelectionRequest,
        pub(crate) semantic_parameters: &'a Value,
        pub(crate) selection: &'a Value,
        pub(crate) captured_inputs: &'a [CapturedInput<'a>],
        pub(crate) staged_directories: &'a [&'a str],
        pub(crate) staged_placeholders: &'a [&'a str],
        pub(crate) contracts: &'a [String],
    }

    #[derive(Clone, Debug)]
    pub(crate) struct FrontendReleaseIdentity {
        pub(crate) release_registry: ReleaseRegistryIdentity,
        pub(crate) frontend: FrontendIdentity,
        pub(crate) toolchain: ToolchainIdentity,
        pub(crate) limit_profile: String,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct AcceptedFrontendRun {
        pub(crate) envelope: AcceptedFrontendEnvelope,
        pub(crate) release: FrontendReleaseIdentity,
        pub(crate) registry: ValidatedReleaseRegistry,
    }

    #[derive(Debug)]
    pub(crate) struct PreparedFrontendRun;

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct FrontendRunCode;

    impl FrontendRunCode {
        pub(crate) const fn as_str(self) -> &'static str {
            "FRONTEND_SANDBOX_UNAVAILABLE"
        }
    }

    #[derive(Debug)]
    pub(crate) struct FrontendRunError;

    impl FrontendRunError {
        pub(crate) const fn code(&self) -> FrontendRunCode {
            FrontendRunCode
        }
    }

    pub(crate) fn prepare_installed_frontend(
        _release: &ReleaseSelectionRequest,
    ) -> Result<PreparedFrontendRun, FrontendRunError> {
        Err(FrontendRunError)
    }

    pub(crate) fn run_prepared_frontend(
        _prepared: PreparedFrontendRun,
        _request: FrontendRunRequest<'_>,
    ) -> Result<AcceptedFrontendRun, FrontendRunError> {
        Err(FrontendRunError)
    }

    pub(crate) fn rust_pointer_width(target: &str) -> Option<i64> {
        match target {
            "i686-unknown-linux-gnu" => Some(32),
            "x86_64-unknown-linux-gnu" => Some(64),
            _ => None,
        }
    }

    pub(crate) fn rust_package_name(value: &str) -> bool {
        if value.len() > 1_024 {
            return false;
        }
        let mut bytes = value.bytes();
        bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }

    fn rust_identifier(value: &str) -> bool {
        if value == "_" || value.len() > 255 || !value.is_ascii() {
            return false;
        }
        let mut bytes = value.bytes();
        bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    }

    pub(crate) fn rust_function_id(value: &str, crate_name: &str) -> bool {
        if value.len() > 1_024 {
            return false;
        }
        let mut segments = value.split("::");
        rust_identifier(crate_name)
            && segments.next() == Some(crate_name)
            && segments.next().is_some_and(rust_identifier)
            && segments.all(rust_identifier)
    }
}

#[path = "../src/policy_scan.rs"]
mod policy_scan;

use frontend_protocol::{
    validate_frontend_process, AcceptedFrontendEnvelope, FrontendProcessFacts,
    FrontendProtocolCode, FrontendProtocolRequest,
};
use frontend_runner::{AcceptedFrontendRun, FrontendReleaseIdentity};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, validate_release_registry, StrictJsonLimits,
    ValidatedReleaseRegistry,
};
use policy_scan::v1::run_policy_scan_v1_with;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_ID: &str = "mpk.release.registry.v0";
const REGISTRY_SHA256: &str = "226baa5e744f2966615a5fe03d6bfa0395db4b191e92bc099e63436fa9936aba";
const RUST_FRONTEND: &str = "frontend.rust.rust2vir.candidate.v0";
const RUST_TOOLCHAIN: &str = "toolchain.rust.nightly-2025-06-01.candidate.v0";

#[test]
fn every_normative_rust_rejection_has_an_accepted_exact_public_classification() {
    let vector = read_json(&repo_root().join("develop/specs/vectors/rust-subset-v0.json"));
    let cases = vector["rejected_cases"].as_array().unwrap();
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(ids.insert(id), "duplicate normative rejection {id}");
        let expected = &case["expect"];
        let status = expected["status"].as_str().unwrap();
        let phase = expected["phase"].as_str().unwrap();
        let code = expected["code"].as_str().unwrap();
        let exit = match status {
            "frontend-error" => 1,
            "rejected" => 3,
            "source-error" => 4,
            _ => panic!("{id}: unknown negative status {status}"),
        };
        let envelope = validate_rust_envelope(&rust_non_success_value(status, phase, code), exit)
            .unwrap_or_else(|error| panic!("{id}: invalid {status}/{phase}/{code}: {error:?}"));
        assert_eq!(envelope.status, status, "{id}");
        assert_eq!(envelope.phase, phase, "{id}");
        assert!(envelope.artifacts.is_none(), "{id}");
    }
    assert_eq!(ids.len(), 73, "the normative rejection inventory drifted");
}

#[test]
fn rust_non_success_envelopes_have_exact_protocol_and_never_become_ready() {
    for (status, phase, exit, code, readiness) in [
        ("rejected", "subset", 3, "RUST_SUBSET_MACRO", "unsupported"),
        (
            "source-error",
            "typecheck",
            4,
            "RUST_SOURCE_TYPE",
            "source_error",
        ),
        (
            "frontend-error",
            "lowering",
            1,
            "RUST_FRONTEND_DRIVER_PROTOCOL_SHAPE",
            "frontend_error",
        ),
        (
            "frontend-error",
            "metadata",
            1,
            "FRONTEND_SANDBOX_UNAVAILABLE",
            "frontend_error",
        ),
        (
            "rejected",
            "subset",
            3,
            "RUST_LIMIT_CONTRACT",
            "unsupported",
        ),
        ("rejected", "emission", 3, "RUST_LIMIT_IR", "unsupported"),
    ] {
        let value = rust_non_success_value(status, phase, code);
        let envelope = validate_rust_envelope(&value, exit).unwrap();
        assert_eq!(envelope.status, status);
        assert_eq!(envelope.phase, phase);
        assert!(envelope.artifacts.is_none());
        for artifact in ["ir", "source_manifest", "source_map"] {
            assert!(
                envelope.value.get(artifact).is_none(),
                "{status}: {artifact}"
            );
        }

        let wrong_exit = if exit == 1 { 3 } else { 1 };
        assert_eq!(
            validate_rust_envelope(&value, wrong_exit)
                .unwrap_err()
                .code(),
            FrontendProtocolCode::ProtocolStatusExit
        );

        let accepted = accepted_non_success_run(envelope);
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        let output = run_policy_scan_v1_with(
            &rust_scan_argv("out/scan.json"),
            temporary.path(),
            Vec::new(),
            |_| Ok(()),
            |(), _| Ok(accepted.clone()),
        )
        .unwrap()
        .unwrap();
        let document = output.scan.document();
        assert_eq!(document.frontend_status, status);
        assert_eq!(document.frontend_phase, phase);
        assert_eq!(document.readiness, readiness);
        assert_ne!(document.readiness, "ready");
        assert!(document.limit_profile.is_none());
        assert!(document.frontend_source_manifest_hash.is_none());
        assert!(document.input_set_hash.is_none());
        assert!(document.source_map_hash.is_none());
        assert!(document.source_ir_schema.is_none());
        assert!(document.source_ir_hash.is_none());
        assert!(document.helper_artifacts.is_none());
        assert!(output.frontend.envelope.artifacts.is_none());

        let projected = serde_json::to_value(document).unwrap();
        for success_only in [
            "limit_profile",
            "frontend_source_manifest_hash",
            "input_set_hash",
            "source_map_hash",
            "source_ir_schema",
            "source_ir_hash",
            "helper_artifacts",
        ] {
            assert!(
                projected.get(success_only).is_none(),
                "{status}: {success_only}"
            );
        }
    }
}

#[test]
fn rust_issue_policy_rejects_unknown_caller_owned_and_misclassified_codes() {
    let mut mutations = Vec::new();

    let mut unknown = rust_non_success_value("rejected", "subset", "RUST_SUBSET_MACRO");
    unknown["rejected_features"][0]["code"] = json!("RUST_UNKNOWN_CASE");
    mutations.push(("unknown Rust code", unknown, 3));

    let obsolete = rust_non_success_value("rejected", "subset", "RUST_CONTRACT_LIMIT");
    mutations.push(("obsolete Rust contract limit code", obsolete, 3));

    let mut caller_owned = rust_non_success_value("rejected", "subset", "RUST_SUBSET_MACRO");
    caller_owned["rejected_features"][0]["code"] = json!("FRONTEND_PROTOCOL_SHAPE");
    mutations.push(("caller-owned code", caller_owned, 3));

    mutations.push((
        "wrong status",
        rust_non_success_value("frontend-error", "subset", "RUST_SUBSET_MACRO"),
        1,
    ));
    mutations.push((
        "wrong phase",
        rust_non_success_value("source-error", "source", "RUST_SOURCE_TYPE"),
        4,
    ));
    mutations.push((
        "toolchain commit at non-owning emission phase",
        rust_non_success_value("frontend-error", "emission", "RUST_TOOLCHAIN_COMMIT"),
        1,
    ));

    let issue = json!({
        "code":"RUST_SUBSET_MACRO",
        "message":"stable Rust diagnostic",
        "function_id":"vector::identity"
    });
    let mut wrong_channel = rust_non_success_value("rejected", "subset", "RUST_SUBSET_MACRO");
    wrong_channel["rejected_features"] = json!([]);
    wrong_channel["diagnostics"] = json!([issue]);
    mutations.push(("wrong channel", wrong_channel, 3));

    let mut cross_crate = rust_non_success_value("rejected", "subset", "RUST_SUBSET_MACRO");
    cross_crate["rejected_features"][0]["function_id"] = json!("other::identity");
    mutations.push(("cross-crate function", cross_crate, 3));

    let mut malformed = rust_non_success_value("rejected", "subset", "RUST_SUBSET_MACRO");
    malformed["rejected_features"][0]["function_id"] = json!("vector::bad-name");
    mutations.push(("malformed function", malformed, 3));

    for (case, value, exit) in mutations {
        assert_eq!(
            validate_rust_envelope(&value, exit).unwrap_err().code(),
            FrontendProtocolCode::ProtocolShape,
            "{case}"
        );
    }

    let go_parameters = json!({"target_id":"linux/amd64","pointer_width":64});
    let go_selection = json!({
        "package":"example.com/mpk/vector",
        "function":"example.com/mpk/vector.Identity"
    });
    let go_value = json!({
        "schema":"mpk.frontend.cli.v0",
        "status":"rejected",
        "phase":"subset",
        "source_language":"go",
        "semantic_profile":"mpk.go.fixed.v0",
        "semantic_parameters":go_parameters,
        "selection":go_selection,
        "rejected_features":[{
            "code":"GO_CUSTOM_STABLE",
            "message":"unchanged Go protocol behavior",
            "function_id":"example.com/mpk/vector.Identity"
        }],
        "diagnostics":[]
    });
    let go_bytes = canonical_transport(&go_value);
    assert!(validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "go",
            semantic_profile: "mpk.go.fixed.v0",
            semantic_parameters: &go_parameters,
            selection: &go_selection,
            release_registry: None,
            captured_inputs: &[],
        },
        FrontendProcessFacts {
            exit_code: Some(3),
            signaled: false,
            stdout: &go_bytes,
            stderr_observed_bytes: 0,
        },
    )
    .is_ok());
}

fn rust_non_success_value(status: &str, phase: &str, code: &str) -> Value {
    let issue = json!({
        "code":code,
        "message":"stable Rust diagnostic",
        "function_id":"vector::identity"
    });
    json!({
        "schema":"mpk.frontend.cli.v0",
        "status":status,
        "phase":phase,
        "source_language":"rust",
        "semantic_profile":"mpk.rust.checked.v0",
        "semantic_parameters":rust_parameters(),
        "selection":rust_selection(),
        "rejected_features":if status == "rejected" { json!([issue]) } else { json!([]) },
        "diagnostics":if status == "rejected" { json!([]) } else { json!([issue]) },
    })
}

fn validate_rust_envelope(
    value: &Value,
    exit: i32,
) -> Result<AcceptedFrontendEnvelope, frontend_protocol::FrontendProtocolError> {
    let bytes = canonical_transport(value);
    validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "rust",
            semantic_profile: "mpk.rust.checked.v0",
            semantic_parameters: &value["semantic_parameters"],
            selection: &value["selection"],
            release_registry: None,
            captured_inputs: &[],
        },
        FrontendProcessFacts {
            exit_code: Some(exit),
            signaled: false,
            stdout: &bytes,
            stderr_observed_bytes: 0,
        },
    )
}

fn accepted_non_success_run(envelope: AcceptedFrontendEnvelope) -> AcceptedFrontendRun {
    let fixture = read_json(&repo_root().join(
        "fixtures/rust-basic/positive/usize-targets/artifacts/x86_64/frontend-envelope.json",
    ));
    AcceptedFrontendRun {
        envelope,
        release: release_from_manifest(&fixture["source_manifest"]),
        registry: tracked_registry(),
    }
}

fn release_from_manifest(manifest: &Value) -> FrontendReleaseIdentity {
    FrontendReleaseIdentity {
        release_registry: serde_json::from_value(manifest["release_registry"].clone()).unwrap(),
        frontend: serde_json::from_value(manifest["frontend"].clone()).unwrap(),
        toolchain: serde_json::from_value(manifest["toolchain"].clone()).unwrap(),
        limit_profile: manifest["limit_profile"].as_str().unwrap().to_owned(),
    }
}

fn tracked_registry() -> ValidatedReleaseRegistry {
    validate_release_registry(include_bytes!(
        "../../../release/bundles/bundle-registry.json"
    ))
    .unwrap()
}

fn rust_parameters() -> Value {
    json!({
        "target_id":"x86_64-unknown-linux-gnu",
        "pointer_width":64,
        "overflow_mode":"checked",
        "panic_mode":"abort"
    })
}

fn rust_selection() -> Value {
    json!({
        "package":"vector",
        "crate":"vector",
        "kind":"lib",
        "function":"vector::identity"
    })
}

fn rust_scan_argv(output: &str) -> Vec<String> {
    [
        "mpk",
        "policy",
        "scan",
        "source",
        "--language",
        "rust",
        "--semantic-profile",
        "mpk.rust.checked.v0",
        "--require-release-registry-id",
        REGISTRY_ID,
        "--require-release-registry-sha256",
        REGISTRY_SHA256,
        "--frontend-bundle",
        RUST_FRONTEND,
        "--toolchain-bundle",
        RUST_TOOLCHAIN,
        "--target",
        "x86_64-unknown-linux-gnu",
        "--package",
        "vector",
        "--function",
        "vector::identity",
        "--contract",
        "contracts/vector.json",
        "--json-out",
        output,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let raw = serde_json::to_vec(value).unwrap();
    let strict = parse_strict_json(
        &raw,
        StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576),
    )
    .unwrap();
    let mut bytes = canonical_json_bytes(&strict).unwrap();
    bytes.push(b'\n');
    bytes
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
