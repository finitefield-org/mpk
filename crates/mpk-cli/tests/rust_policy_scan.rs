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
    validate_frontend_process, validate_frontend_process_from_staging, FrontendProcessFacts,
    FrontendProtocolRequest, FrontendStagingRequest,
};
use frontend_runner::{AcceptedFrontendRun, FrontendReleaseIdentity};
use mpk_vc::{
    canonical_json_bytes, input_set_hash, parse_strict_json, source_manifest_hash, source_map_hash,
    validate_release_registry, CompilerIdentity, InputKind, ReleaseSelectionRequest,
    SourceManifest, SourceMap, StrictJsonLimits, ValidatedReleaseRegistry,
};
use policy_scan::v1::tests::go_scan_argv as synthetic_go_scan_argv;
use policy_scan::v1::{
    capture_invocation_inputs, capture_invocation_staging, parse_policy_scan_v1_argv,
    run_policy_scan_v1_with, run_policy_scan_v1_with_staging, OwnedCapturedInput,
    PolicyScanV1Error,
};
use serde_json::{json, Value};
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_ID: &str = "mpk.release.registry.v0";
const REGISTRY_SHA256: &str = "bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98";
const RUST_FRONTEND: &str = "frontend.rust.rust2vir.candidate.v0";
const RUST_TOOLCHAIN: &str = "toolchain.rust.nightly-2025-06-01.candidate.v0";

#[test]
fn both_registered_rust_targets_use_the_generic_runner_and_are_deterministic() {
    for (artifact_target, target, contract, width, raw_contract_hash, contract_hash, vir_hash) in [
        (
            "i686",
            "i686-unknown-linux-gnu",
            "contracts/i686.json",
            32,
            "377667e7c4ccb31b654862ee153933c47f915bf66d0818bcbeb815dc79eabc60",
            "5670ba8d28b88c64829884cb2e0ced83ad9045e0a1cc80f21a37aa6b8356d405",
            "2324b451b37fdfceda56c45f080d2c9ec0eff2be180ec5a3423541e132a4991e",
        ),
        (
            "x86_64",
            "x86_64-unknown-linux-gnu",
            "contracts/x86_64.json",
            64,
            "a0dce8024c005326926622966963f1d4af29ef46717bd94e4a8a834a8e76a592",
            "f7daf9935899976031907b762c6015ecfa7cf6ed0a0e721ae1ce704320b8b947",
            "04484b915288b0bf51a9de286b00f51e2a188da8aa2779e61222eae037f0d45e",
        ),
    ] {
        let case = repo_root().join("fixtures/rust-basic/positive/usize-targets");
        let envelope_path = case
            .join("artifacts")
            .join(artifact_target)
            .join("frontend-envelope.json");
        let inputs = captured_case_inputs(&case, &envelope_path);
        let accepted = accepted_ready_run(&envelope_path, &inputs);
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        let launches = Cell::new(0);
        let argv = rust_scan_argv(target, "vector::usize_index", &[contract], "out/scan.json");

        let first = run_with_registered_release(
            &argv,
            temporary.path(),
            inputs.clone(),
            accepted.clone(),
            &launches,
        );
        assert_eq!(launches.get(), 1);
        let document = serde_json::to_value(first.scan.document()).unwrap();
        assert_eq!(document["readiness"], "ready");
        assert_eq!(document["semantic_parameters"]["pointer_width"], width);
        assert_eq!(document["semantic_parameters"]["target_id"], target);
        assert_eq!(
            document["selection"],
            json!({
                "package":"vector",
                "crate":"vector",
                "kind":"lib",
                "function":"vector::usize_index"
            })
        );
        assert_eq!(document["frontend"]["name"], "rust2vir");
        assert_eq!(
            document["frontend"]["subordinate_binaries"][0]["name"],
            "rust2vir-driver"
        );
        assert_eq!(
            document["frontend"]["subordinate_binaries"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            document["toolchain"]["bundle_id"],
            "toolchain.rust.nightly-2025-06-01.candidate.v0"
        );
        let artifacts = accepted.envelope.artifacts.as_ref().unwrap();
        let manifest = &accepted.envelope.value["source_manifest"];
        assert_eq!(document["source_language"], "rust");
        assert_eq!(document["semantic_profile"], "mpk.rust.checked.v0");
        assert_eq!(document["limit_profile"], manifest["limit_profile"]);
        assert_eq!(document["input_set_hash"], manifest["input_set_hash"]);
        assert_eq!(document["source_ir_schema"], "mpk.vir.v0");
        assert_eq!(document["release_registry"], manifest["release_registry"]);
        assert_eq!(document["frontend"], manifest["frontend"]);
        assert_eq!(document["toolchain"], manifest["toolchain"]);
        assert_eq!(
            document["frontend_source_manifest_hash"],
            artifacts.source_manifest.hash().as_str()
        );
        assert_eq!(
            document["source_map_hash"],
            artifacts.source_map.hash().as_str()
        );
        assert_eq!(document["source_ir_hash"], artifacts.vir.vir_hash.as_str());
        assert_eq!(
            document["helper_artifacts"],
            json!([
                {
                    "kind":"source",
                    "id":"source:src/lib.rs",
                    "normalized_path":"src/lib.rs",
                    "sha256":"4fb0ed4ec7a986a1afbfc498d46978ed456ccc7aee4567d68ac63b7f1949c03e"
                },
                {
                    "kind":"contract",
                    "id":"contract:vector::usize_index",
                    "normalized_path":contract,
                    "schema":"mpk.rust.contract.v0",
                    "raw_input_sha256":raw_contract_hash,
                    "function_id":"vector::usize_index",
                    "contract_hash":contract_hash
                },
                {
                    "kind":"verification_ir",
                    "id":"verification_ir",
                    "schema":"mpk.vir.v0",
                    "sha256":vir_hash
                }
            ])
        );
        for forbidden in ["strategy_profile", "checker_profile", "axiom_profile"] {
            assert!(document.get(forbidden).is_none());
        }
        assert!(
            serde_json::to_string(&artifacts.vir)
                .unwrap()
                .contains(r#""safety_checks":[{"kind":"index_in_bounds"}]"#),
            "a safety-bearing lowered function must remain scan-ready"
        );

        let mut repeated_argv = argv;
        replace_option(&mut repeated_argv, "--json-out", "out/scan-repeat.json");
        let repeated = run_with_registered_release(
            &repeated_argv,
            temporary.path(),
            inputs,
            accepted,
            &launches,
        );
        assert_eq!(launches.get(), 2);
        assert_eq!(
            first.scan.canonical_bytes(),
            repeated.scan.canonical_bytes()
        );
        assert_eq!(
            fs::read(temporary.path().join("out/scan.json")).unwrap(),
            fs::read(temporary.path().join("out/scan-repeat.json")).unwrap()
        );
    }
}

#[test]
fn arbitrary_rust_crate_root_filename_survives_protocol_and_scan_projection() {
    const CUSTOM_ROOT: &str = "src/Cargo.lock";
    const CUSTOM_CONTRACT: &str = "contracts/spec";

    let case = repo_root().join("fixtures/rust-basic/positive/usize-targets");
    let envelope_path = case.join("artifacts/x86_64/frontend-envelope.json");
    let mut inputs = captured_case_inputs(&case, &envelope_path);
    inputs
        .iter_mut()
        .find(|input| input.kind == InputKind::Source)
        .unwrap()
        .normalized_path = CUSTOM_ROOT.to_owned();
    inputs
        .iter_mut()
        .find(|input| input.kind == InputKind::Contract)
        .unwrap()
        .normalized_path = CUSTOM_CONTRACT.to_owned();

    let mut envelope = read_json(&envelope_path);
    let mut source_map_value = envelope["source_map"].clone();
    for entry in source_map_value["entries"].as_array_mut().unwrap() {
        if entry["origin"]["input_kind"] == "source" {
            entry["origin"]["normalized_path"] = Value::String(CUSTOM_ROOT.to_owned());
        }
    }
    let mut source_map: SourceMap = serde_json::from_value(source_map_value).unwrap();
    source_map.source_map_hash = source_map_hash(&source_map).unwrap().as_str().to_owned();
    envelope["source_map"] = serde_json::to_value(&source_map).unwrap();

    let mut source_manifest: SourceManifest =
        serde_json::from_value(envelope["source_manifest"].clone()).unwrap();
    source_manifest
        .inputs
        .iter_mut()
        .find(|input| input.kind == InputKind::Source)
        .unwrap()
        .normalized_path = CUSTOM_ROOT.to_owned();
    source_manifest
        .inputs
        .iter_mut()
        .find(|input| input.kind == InputKind::Contract)
        .unwrap()
        .normalized_path = CUSTOM_CONTRACT.to_owned();
    source_manifest.input_set_hash = input_set_hash(&source_manifest.inputs)
        .unwrap()
        .as_str()
        .to_owned();
    source_manifest.source_map_hash = source_map.source_map_hash.clone();
    source_manifest.source_manifest_hash = source_manifest_hash(&source_manifest)
        .unwrap()
        .as_str()
        .to_owned();
    envelope["source_manifest"] = serde_json::to_value(&source_manifest).unwrap();

    let bytes = canonical_transport(&envelope);
    let registry = tracked_registry();
    let captured = inputs
        .iter()
        .map(OwnedCapturedInput::as_ref)
        .collect::<Vec<_>>();
    let accepted = validate_frontend_process_from_staging(
        FrontendStagingRequest {
            source_language: "rust",
            semantic_profile: "mpk.rust.checked.v0",
            semantic_parameters: &envelope["semantic_parameters"],
            selection: &envelope["selection"],
            release_registry: Some(&registry),
            available_inputs: &captured,
        },
        FrontendProcessFacts {
            exit_code: Some(0),
            signaled: false,
            stdout: &bytes,
            stderr_observed_bytes: 0,
        },
    )
    .unwrap();
    let accepted = AcceptedFrontendRun {
        envelope: accepted,
        release: release_from_manifest(&envelope["source_manifest"]),
        registry,
    };

    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    let launches = Cell::new(0);
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::usize_index",
        &[CUSTOM_CONTRACT],
        "out/custom-root.json",
    );
    let output = run_with_registered_release(&argv, temporary.path(), inputs, accepted, &launches);
    assert_eq!(launches.get(), 1);
    assert!(output
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == CUSTOM_ROOT && input.kind == InputKind::Source));
    assert!(
        serde_json::to_value(output.scan.document()).unwrap()["helper_artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|helper| helper["id"] == "source:src/Cargo.lock"
                && helper["normalized_path"] == CUSTOM_ROOT)
    );
    assert!(
        serde_json::to_value(output.scan.document()).unwrap()["helper_artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|helper| helper["kind"] == "contract"
                && helper["normalized_path"] == CUSTOM_CONTRACT)
    );
}

#[test]
fn multiple_rust_contracts_populate_exact_sorted_helpers() {
    let case = repo_root().join("fixtures/rust-basic/positive/module-calls");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let inputs = captured_case_inputs(&case, &envelope_path);
    let accepted = accepted_ready_run(&envelope_path, &inputs);
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    let launches = Cell::new(0);
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::cross_module",
        &[
            "contracts/selected.json",
            "contracts/public.json",
            "contracts/private.json",
        ],
        "out/module-calls.json",
    );
    let output = run_with_registered_release(
        &argv,
        temporary.path(),
        inputs.clone(),
        accepted.clone(),
        &launches,
    );
    assert_eq!(launches.get(), 1);
    assert_eq!(
        output.invocation.contracts,
        [
            "contracts/private.json",
            "contracts/public.json",
            "contracts/selected.json",
        ]
    );
    let document = serde_json::to_value(output.scan.document()).unwrap();
    assert_eq!(
        document["helper_artifacts"],
        json!([
            {
                "kind":"source",
                "id":"source:src/lib.rs",
                "normalized_path":"src/lib.rs",
                "sha256":"21c3d0ecf917ae461c20b2d59f9452009026c9d72c80683f4b36ace130ab8240"
            },
            {
                "kind":"contract",
                "id":"contract:vector::cross_module",
                "normalized_path":"contracts/selected.json",
                "schema":"mpk.rust.contract.v0",
                "raw_input_sha256":"509b3cf4ece61efaba58e40b6b94fd44c90137d72da0efc6f48d9df9f0d1b098",
                "function_id":"vector::cross_module",
                "contract_hash":"4e2632912b632e1189f9ae0fa6366ecb79434b04a5b459277e564c19d6eb37dc"
            },
            {
                "kind":"contract",
                "id":"contract:vector::helpers::private_leaf",
                "normalized_path":"contracts/private.json",
                "schema":"mpk.rust.contract.v0",
                "raw_input_sha256":"3e7933a16b2ca2d9ff70e87fc57041bc6d0e04d2c3dcd09dad4e1f507582bbad",
                "function_id":"vector::helpers::private_leaf",
                "contract_hash":"b2c018ce3dc63da607afaab0879891937adf93f934a83f9cb1f4694bd64802bf"
            },
            {
                "kind":"contract",
                "id":"contract:vector::helpers::public_helper",
                "normalized_path":"contracts/public.json",
                "schema":"mpk.rust.contract.v0",
                "raw_input_sha256":"9d296ca7b2848ed24c6f4080c50974399f9580f3574ca0ca4002d4ba1728221a",
                "function_id":"vector::helpers::public_helper",
                "contract_hash":"a11ea90dc61981acb26ca5e1f34d11572c7b2f3f6355d7f2ad841b264129881b"
            },
            {
                "kind":"verification_ir",
                "id":"verification_ir",
                "schema":"mpk.vir.v0",
                "sha256":"be1afcdefa9c3b9336fbeb490afd93743c2f8ae65554a205a4478a9e1b515dc0"
            }
        ])
    );

    let repeated_argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::cross_module",
        &[
            "contracts/private.json",
            "contracts/selected.json",
            "contracts/public.json",
        ],
        "out/module-calls-repeat.json",
    );
    let repeated = run_with_registered_release(
        &repeated_argv,
        temporary.path(),
        inputs,
        accepted,
        &launches,
    );
    assert_eq!(launches.get(), 2);
    assert_eq!(
        output.scan.canonical_bytes(),
        repeated.scan.canonical_bytes()
    );
}

#[test]
fn broad_staging_projects_the_exact_rust_module_closure() {
    let case = repo_root().join("fixtures/rust-basic/positive/multi-file-closure");
    let envelope_path = case.join("artifacts/frontend-envelope.json");
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join(".git")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir_all(source.join("src/target")).unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    for path in [
        "contracts/private.json",
        "contracts/public.json",
        "contracts/selected.json",
        "src/helpers.rs",
        "src/lib.rs",
        "src/unrelated.rs",
    ] {
        fs::copy(case.join("source").join(path), source.join(path)).unwrap();
    }
    fs::write(
        source.join(".git/lib.rs"),
        b"pub fn staged_dot_git_crate() {}\n",
    )
    .unwrap();
    fs::write(
        source.join("src/target/mod.rs"),
        b"pub fn staged_target_module() {}\n",
    )
    .unwrap();
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::multi_file",
        &[
            "contracts/selected.json",
            "contracts/public.json",
            "contracts/private.json",
        ],
        "out/scan.json",
    );
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
    let staged = capture_invocation_inputs(&invocation, temporary.path()).unwrap();
    assert!(staged
        .iter()
        .any(|input| input.normalized_path == "src/unrelated.rs"));
    assert!(staged
        .iter()
        .any(|input| input.normalized_path == "src/target/mod.rs"));
    assert!(staged
        .iter()
        .any(|input| input.normalized_path == ".git/lib.rs"));

    assert_eq!(
        validate_ready_run_exact(&envelope_path, &staged)
            .unwrap_err()
            .code()
            .as_str(),
        "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH"
    );
    let accepted = validate_ready_run(&envelope_path, &staged).unwrap();
    let launches = Cell::new(0);
    let output =
        run_with_registered_release(&argv, temporary.path(), staged.clone(), accepted, &launches);
    assert_eq!(launches.get(), 1);
    assert_eq!(output.scan.document().readiness, "ready");
    assert!(!output
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == "src/unrelated.rs"));
    assert!(!output
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == "src/target/mod.rs"));
    assert!(!output
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == ".git/lib.rs"));
    assert_eq!(output.captured_inputs.len(), 7);
    let helpers = serde_json::to_value(output.scan.document())
        .unwrap()
        .get("helper_artifacts")
        .and_then(Value::as_array)
        .unwrap()
        .clone();
    assert_eq!(
        Value::Array(helpers[..2].to_vec()),
        json!([
            {
                "kind":"source",
                "id":"source:src/helpers.rs",
                "normalized_path":"src/helpers.rs",
                "sha256":"47177c1fe114862931fd070803228eb6a8fdde6a57dc87160ee7538d87c6f39c"
            },
            {
                "kind":"source",
                "id":"source:src/lib.rs",
                "normalized_path":"src/lib.rs",
                "sha256":"3df7fce3623f39b6ceb10cf7bc17109a790e64ff833b73de083252562b6e5596"
            }
        ])
    );

    let mut duplicate_available = staged.clone();
    duplicate_available.push(
        staged
            .iter()
            .find(|input| input.normalized_path == "src/unrelated.rs")
            .unwrap()
            .clone(),
    );
    let mut case_colliding_available = staged.clone();
    case_colliding_available.push(OwnedCapturedInput {
        kind: InputKind::Source,
        normalized_path: "src/UNRELATED.rs".to_owned(),
        bytes: b"pub fn ignored() {}\n".to_vec(),
    });
    let mut nonportable_available = staged.clone();
    nonportable_available.push(OwnedCapturedInput {
        kind: InputKind::Source,
        normalized_path: "../ignored.rs".to_owned(),
        bytes: b"pub fn ignored() {}\n".to_vec(),
    });
    for invalid_available in [
        duplicate_available,
        case_colliding_available,
        nonportable_available,
    ] {
        assert_eq!(
            validate_ready_run(&envelope_path, &invalid_available)
                .unwrap_err()
                .code()
                .as_str(),
            "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH"
        );
    }

    let missing = staged
        .iter()
        .filter(|input| input.normalized_path != "src/helpers.rs")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        validate_ready_run(&envelope_path, &missing)
            .unwrap_err()
            .code()
            .as_str(),
        "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH"
    );
    let mut wrong_kind = staged.clone();
    wrong_kind
        .iter_mut()
        .find(|input| input.normalized_path == "src/helpers.rs")
        .unwrap()
        .kind = InputKind::Contract;
    assert_eq!(
        validate_ready_run(&envelope_path, &wrong_kind)
            .unwrap_err()
            .code()
            .as_str(),
        "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH"
    );
    let mut mutated = staged;
    mutated
        .iter_mut()
        .find(|input| input.normalized_path == "src/helpers.rs")
        .unwrap()
        .bytes[0] ^= 1;
    assert_eq!(
        validate_ready_run(&envelope_path, &mutated)
            .unwrap_err()
            .code()
            .as_str(),
        "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH"
    );
}

#[test]
fn forbidden_rust_configuration_authorities_reach_frontend_staging() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join(".cargo")).unwrap();
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
    fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
    for path in [
        "rust-toolchain",
        "rust-toolchain.toml",
        ".cargo/config",
        ".cargo/config.toml",
    ] {
        fs::write(source.join(path), b"forbidden\n").unwrap();
    }

    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
    let staged = capture_invocation_inputs(&invocation, temporary.path()).unwrap();
    let expected = [
        ".cargo/config",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
    ];
    for path in expected {
        assert!(staged.iter().any(|input| {
            input.normalized_path == path && input.kind == InputKind::BuildManifest
        }));
    }

    let accepted =
        accepted_non_success_run("rejected", "capture", 3, "RUST_PREFLIGHT_TOOLCHAIN_FILE");
    let output = run_policy_scan_v1_with(
        &argv,
        temporary.path(),
        staged,
        |_| Ok(()),
        |(), request| {
            for path in expected {
                assert!(request.captured_inputs.iter().any(|input| {
                    input.normalized_path == path && input.kind == InputKind::BuildManifest
                }));
            }
            Ok(accepted.clone())
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(output.scan.document().readiness, "unsupported");
}

#[cfg(target_os = "linux")]
#[test]
fn rust_staging_rejects_symlinks_and_hard_link_aliases() {
    use std::os::unix::fs::symlink;

    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
    fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();

    fs::write(
        temporary.path().join("outside.rs"),
        b"pub fn escaped() {}\n",
    )
    .unwrap();
    symlink(
        temporary.path().join("outside.rs"),
        source.join("src/escape.rs"),
    )
    .unwrap();
    assert_eq!(
        capture_invocation_inputs(&invocation, temporary.path())
            .unwrap_err()
            .code(),
        "POLICY_CLI_INPUT"
    );
    fs::remove_file(source.join("src/escape.rs")).unwrap();

    fs::hard_link(source.join("src/lib.rs"), source.join("src/alias.rs")).unwrap();
    assert_eq!(
        capture_invocation_inputs(&invocation, temporary.path())
            .unwrap_err()
            .code(),
        "POLICY_CLI_INPUT"
    );
    fs::remove_file(source.join("src/alias.rs")).unwrap();

    let oversized = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(source.join("src/oversized.rs"))
        .unwrap();
    oversized.set_len(33_554_433).unwrap();
    assert_eq!(
        capture_invocation_inputs(&invocation, temporary.path())
            .unwrap_err()
            .code(),
        "POLICY_CLI_INPUT"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn staging_preserves_profile_candidates_and_go_hard_links_remain_path_semantic() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
    fs::write(source.join("src/.rs"), b"pub fn dot_only_rust() {}\n").unwrap();
    fs::write(source.join("src/root.custom"), b"pub fn custom_root() {}\n").unwrap();
    fs::write(
        source.join("src/Cargo.lock"),
        b"pub fn reserved_basename_root() {}\n",
    )
    .unwrap();
    fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let rust_invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();

    fs::write(source.join("ignored.go"), b"package vector\n").unwrap();
    let rust_staged = capture_invocation_inputs(&rust_invocation, temporary.path()).unwrap();
    assert!(rust_staged
        .iter()
        .any(|input| input.normalized_path == "src/.rs" && input.kind == InputKind::Source));
    assert!(
        rust_staged
            .iter()
            .any(|input| input.normalized_path == "src/root.custom"
                && input.kind == InputKind::Source)
    );
    assert!(rust_staged
        .iter()
        .any(|input| input.normalized_path == "src/Cargo.lock" && input.kind == InputKind::Source));
    assert!(rust_staged
        .iter()
        .any(|input| input.normalized_path == "ignored.go" && input.kind == InputKind::Source));

    fs::write(source.join("go.mod"), b"module example.com/vector\n").unwrap();
    fs::write(source.join("go.sum"), b"").unwrap();
    fs::write(source.join(".go"), b"package vector\n").unwrap();
    fs::write(
        source.join("main.go"),
        b"package vector\nfunc Identity() {}\n",
    )
    .unwrap();
    fs::hard_link(source.join("main.go"), source.join("alias.go")).unwrap();
    let ignored_test = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(source.join("ignored_test.go"))
        .unwrap();
    ignored_test.set_len(16_777_217).unwrap();
    drop(ignored_test);
    let auxiliary_suffixes = [
        ".c", ".cc", ".cpp", ".cxx", ".m", ".h", ".hh", ".hpp", ".hxx", ".f", ".F", ".for", ".f90",
        ".s", ".S", ".sx", ".swig", ".swigcxx", ".syso",
    ];
    for (index, suffix) in auxiliary_suffixes.iter().enumerate() {
        fs::write(
            source.join(format!("native{index}{suffix}")),
            b"forbidden\n",
        )
        .unwrap();
    }
    for (name, size) in [
        ("candidate_at_profile_max.c", 16_777_216_u64),
        ("candidate_above_profile_max.c", 16_777_217_u64),
    ] {
        let candidate = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(source.join(name))
            .unwrap();
        candidate.set_len(size).unwrap();
    }
    fs::write(source.join("orphan_contract.json"), b"{}\n").unwrap();
    let oversized_rust = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(source.join("src/ignored.rs"))
        .unwrap();
    oversized_rust.set_len(16_777_217).unwrap();
    drop(oversized_rust);
    fs::hard_link(
        source.join("src/ignored.rs"),
        source.join("src/ignored_alias.rs"),
    )
    .unwrap();

    let mut go_invocation = rust_invocation;
    go_invocation.source_language = "go".to_owned();
    let go_staged = capture_invocation_inputs(&go_invocation, temporary.path()).unwrap();
    assert!(go_staged
        .iter()
        .any(|input| input.normalized_path == "main.go"));
    assert!(go_staged
        .iter()
        .any(|input| input.normalized_path == "alias.go"));
    assert!(go_staged
        .iter()
        .any(|input| input.normalized_path == ".go" && input.kind == InputKind::Source));
    assert!(!go_staged
        .iter()
        .any(|input| input.normalized_path == "ignored_test.go"));
    for (index, suffix) in auxiliary_suffixes.iter().enumerate() {
        let path = format!("native{index}{suffix}");
        assert!(go_staged
            .iter()
            .any(|input| { input.normalized_path == path && input.kind == InputKind::Source }));
    }
    assert!(go_staged.iter().any(|input| {
        input.normalized_path == "orphan_contract.json" && input.kind == InputKind::Contract
    }));
    for (name, size) in [
        ("candidate_at_profile_max.c", 16_777_216_usize),
        ("candidate_above_profile_max.c", 16_777_217_usize),
    ] {
        assert!(go_staged.iter().any(|input| {
            input.normalized_path == name
                && input.kind == InputKind::Source
                && input.bytes.len() == size
        }));
    }
    assert!(!go_staged
        .iter()
        .any(|input| input.normalized_path.ends_with(".rs")));
    assert!(!go_staged
        .iter()
        .any(|input| input.normalized_path == "src/root.custom"));
    assert!(!go_staged
        .iter()
        .any(|input| input.normalized_path.starts_with("Cargo.")));
}

#[cfg(target_os = "linux")]
#[test]
fn private_namespace_reaches_the_runner_with_complete_entry_names_and_kinds() {
    for populated_vendor in [false, true] {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        fs::create_dir_all(source.join("contracts")).unwrap();
        fs::create_dir(source.join("vendor")).unwrap();
        fs::create_dir(source.join("native.c")).unwrap();
        fs::create_dir(source.join("misc")).unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        fs::write(
            source.join("go.mod"),
            b"module example.com/vector\ngo 1.23\n",
        )
        .unwrap();
        fs::write(source.join("go.sum"), b"").unwrap();
        fs::write(
            source.join("main.go"),
            b"package vector\nfunc Identity() {}\n",
        )
        .unwrap();
        fs::write(source.join("ignored_test.go"), b"package vector\n").unwrap();
        fs::write(source.join("notes.txt"), b"private bytes are not staged\n").unwrap();
        fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
        if populated_vendor {
            fs::write(source.join("vendor/modules.txt"), b"not opened\n").unwrap();
        }

        let mut argv = rust_scan_argv(
            "x86_64-unknown-linux-gnu",
            "vector::identity",
            &["contracts/vector.json"],
            "out/scan.json",
        );
        replace_option(&mut argv, "--language", "go");
        replace_option(&mut argv, "--semantic-profile", "mpk.go.fixed.v0");
        replace_option(&mut argv, "--frontend-bundle", "frontend.go.go2vir.v0");
        replace_option(
            &mut argv,
            "--toolchain-bundle",
            "toolchain.go.go1.25.0.linux-amd64.v0",
        );
        replace_option(&mut argv, "--target", "linux/amd64");
        replace_option(&mut argv, "--package", "example.com/vector");
        replace_option(&mut argv, "--function", "example.com/vector.Identity");
        let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
        let staging = capture_invocation_staging(&invocation, temporary.path()).unwrap();
        assert!(staging
            .staged_directories
            .iter()
            .any(|path| path == "vendor"));
        assert!(staging
            .staged_directories
            .iter()
            .any(|path| path == "native.c"));
        assert!(staging.staged_directories.iter().any(|path| path == "misc"));
        assert!(staging
            .staged_placeholders
            .iter()
            .any(|path| path == "ignored_test.go"));
        assert!(staging
            .staged_placeholders
            .iter()
            .any(|path| path == "notes.txt"));
        assert_eq!(
            staging
                .staged_placeholders
                .iter()
                .any(|path| path == "vendor/modules.txt"),
            populated_vendor
        );
        let launches = Cell::new(0);
        let error = run_policy_scan_v1_with_staging(
            &argv,
            temporary.path(),
            staging,
            |_| Ok(()),
            |(), request| {
                launches.set(launches.get() + 1);
                assert!(request.staged_directories.contains(&"vendor"));
                assert!(request.staged_directories.contains(&"native.c"));
                assert!(request.staged_placeholders.contains(&"ignored_test.go"));
                assert_eq!(
                    request.staged_placeholders.contains(&"vendor/modules.txt"),
                    populated_vendor
                );
                Err(PolicyScanV1Error::new(
                    "TEST_STOP",
                    "namespace reached the generic runner",
                ))
            },
        )
        .unwrap_err();
        assert_eq!(error.code(), "TEST_STOP");
        assert_eq!(launches.get(), 1);
    }

    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir(source.join("rust-toolchain")).unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
    fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
    let staging = capture_invocation_staging(&invocation, temporary.path()).unwrap();
    assert!(staging
        .staged_directories
        .iter()
        .any(|path| path == "rust-toolchain"));
    let error = run_policy_scan_v1_with_staging(
        &argv,
        temporary.path(),
        staging,
        |_| Ok(()),
        |(), request| {
            assert!(request.staged_directories.contains(&"rust-toolchain"));
            Err(PolicyScanV1Error::new(
                "TEST_STOP",
                "directory kind reached the generic runner",
            ))
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEST_STOP");
}

#[cfg(target_os = "linux")]
#[test]
fn missing_and_directory_contracts_reach_frontend_status_projection() {
    for (case, directory_contract) in [("missing", false), ("directory", true)] {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        fs::create_dir_all(source.join("contracts")).unwrap();
        fs::create_dir_all(source.join("src")).unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        for path in ["Cargo.toml", "Cargo.lock"] {
            fs::copy(
                repo_root().join("fixtures/rust-basic").join(path),
                source.join(path),
            )
            .unwrap();
        }
        fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
        let contract = format!("contracts/{case}");
        if directory_contract {
            fs::create_dir(source.join(&contract)).unwrap();
        }
        let argv = rust_scan_argv(
            "x86_64-unknown-linux-gnu",
            "vector::identity",
            &[&contract],
            "out/scan.json",
        );
        let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
        let staging = capture_invocation_staging(&invocation, temporary.path()).unwrap();
        assert!(!staging
            .captured_inputs
            .iter()
            .any(|input| input.normalized_path == contract));
        assert_eq!(
            staging
                .staged_directories
                .iter()
                .any(|path| path == &contract),
            directory_contract
        );

        let accepted =
            accepted_non_success_run("rejected", "capture", 3, "RUST_PREFLIGHT_FILE_TYPE");
        let launches = Cell::new(0);
        let output = run_policy_scan_v1_with_staging(
            &argv,
            temporary.path(),
            staging,
            |_| Ok(()),
            |(), request| {
                launches.set(launches.get() + 1);
                assert_eq!(request.contracts, [contract.as_str()]);
                assert!(!request
                    .captured_inputs
                    .iter()
                    .any(|input| input.normalized_path == contract));
                assert_eq!(
                    request.staged_directories.contains(&contract.as_str()),
                    directory_contract
                );
                Ok(accepted.clone())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(launches.get(), 1);
        assert_eq!(output.scan.document().frontend_status, "rejected");
        assert_eq!(output.scan.document().readiness, "unsupported");
        assert_eq!(
            output.scan.document().rejected_features[0].code,
            "RUST_PREFLIGHT_FILE_TYPE"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn invalid_go_explicit_contract_names_keep_filename_roles_until_frontend_rejection() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    fs::write(
        source.join("go.mod"),
        b"module example.com/mpk/vector\ngo 1.23\n",
    )
    .unwrap();
    fs::write(source.join("go.sum"), b"").unwrap();
    fs::write(
        source.join("main.go"),
        b"package vector\nfunc Identity() {}\n",
    )
    .unwrap();
    fs::write(source.join("notes.txt"), b"not a contract candidate\n").unwrap();
    let ignored_test = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(source.join("ignored_test.go"))
        .unwrap();
    ignored_test.set_len(33_554_433).unwrap();
    drop(ignored_test);

    let mut argv = synthetic_go_scan_argv();
    argv[3] = "source".to_owned();
    replace_option(&mut argv, "--contract", "ignored_test.go");
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
    let staging = capture_invocation_staging(&invocation, temporary.path()).unwrap();
    assert!(staging
        .staged_placeholders
        .iter()
        .any(|path| path == "ignored_test.go"));
    assert!(!staging
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == "ignored_test.go"));

    let mut role_probe = invocation.clone();
    role_probe.contracts = vec![
        "go.mod".to_owned(),
        "main.go".to_owned(),
        "notes.txt".to_owned(),
    ];
    let role_staging = capture_invocation_staging(&role_probe, temporary.path()).unwrap();
    assert!(role_staging.captured_inputs.iter().any(|input| {
        input.normalized_path == "go.mod" && input.kind == InputKind::BuildManifest
    }));
    assert!(role_staging
        .captured_inputs
        .iter()
        .any(|input| input.normalized_path == "main.go" && input.kind == InputKind::Source));
    assert!(role_staging
        .staged_placeholders
        .iter()
        .any(|path| path == "notes.txt"));

    let accepted = accepted_go_non_success_run("GO_CONTRACT_FUNCTION");
    let launches = Cell::new(0);
    let output = run_policy_scan_v1_with_staging(
        &argv,
        temporary.path(),
        staging,
        |_| Ok(()),
        |(), request| {
            launches.set(launches.get() + 1);
            assert!(request.staged_placeholders.contains(&"ignored_test.go"));
            assert!(!request
                .captured_inputs
                .iter()
                .any(|input| input.normalized_path == "ignored_test.go"));
            Ok(accepted.clone())
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(launches.get(), 1);
    assert_eq!(output.scan.document().frontend_status, "rejected");
    assert_eq!(output.scan.document().readiness, "unsupported");
    assert_eq!(
        output.scan.document().rejected_features[0].code,
        "GO_CONTRACT_FUNCTION"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn rust_profile_file_maximum_and_plus_one_reach_the_frontend_seam() {
    for (case, size) in [("at-max", 1_048_576_u64), ("above-max", 1_048_577_u64)] {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        fs::create_dir_all(source.join("contracts")).unwrap();
        fs::create_dir_all(source.join("src")).unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        for path in ["Cargo.toml", "Cargo.lock"] {
            fs::copy(
                repo_root().join("fixtures/rust-basic").join(path),
                source.join(path),
            )
            .unwrap();
        }
        let candidate = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(source.join("src/lib.rs"))
            .unwrap();
        candidate.set_len(size).unwrap();
        drop(candidate);
        fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
        let argv = rust_scan_argv(
            "x86_64-unknown-linux-gnu",
            "vector::identity",
            &["contracts/vector.json"],
            "out/scan.json",
        );
        let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
        let staged = capture_invocation_inputs(&invocation, temporary.path()).unwrap();
        assert!(staged.iter().any(|input| {
            input.normalized_path == "src/lib.rs" && input.bytes.len() as u64 == size
        }));
        let launches = Cell::new(0);
        let accepted = accepted_non_success_run("rejected", "capture", 3, "RUST_LIMIT_INPUT_BYTES");
        let output = run_policy_scan_v1_with(
            &argv,
            temporary.path(),
            staged,
            |_| Ok(()),
            |(), request| {
                launches.set(launches.get() + 1);
                assert!(request.captured_inputs.iter().any(|input| {
                    input.normalized_path == "src/lib.rs" && input.bytes.len() as u64 == size
                }));
                Ok(accepted.clone())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(launches.get(), 1, "{case}");
        assert_eq!(output.scan.document().readiness, "unsupported", "{case}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn rust_staging_bounds_descriptors_for_wide_and_deep_trees() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    fs::create_dir_all(source.join("contracts")).unwrap();
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir(source.join("wide")).unwrap();
    for path in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(
            repo_root().join("fixtures/rust-basic").join(path),
            source.join(path),
        )
        .unwrap();
    }
    fs::write(source.join("src/lib.rs"), b"pub fn identity() {}\n").unwrap();
    fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();
    for index in 0..1_050 {
        fs::create_dir(source.join("wide").join(format!("d{index:04}"))).unwrap();
    }
    let mut deep = source.join("deep");
    fs::create_dir(&deep).unwrap();
    for _ in 0..510 {
        deep.push("d");
        fs::create_dir(&deep).unwrap();
    }
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let invocation = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();

    let staged = capture_invocation_inputs(&invocation, temporary.path()).unwrap();
    assert!(staged
        .iter()
        .any(|input| input.normalized_path == "src/lib.rs"));
}

#[test]
fn rust_non_success_statuses_preserve_diagnostics_and_readiness() {
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
    ] {
        let accepted = accepted_non_success_run(status, phase, exit, code);
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        let argv = rust_scan_argv(
            "x86_64-unknown-linux-gnu",
            "vector::identity",
            &["contracts/vector.json"],
            "out/scan.json",
        );
        let output = run_policy_scan_v1_with(
            &argv,
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
        assert!(document.helper_artifacts.is_none());
        let issues = if status == "rejected" {
            &document.rejected_features
        } else {
            &document.diagnostics
        };
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, code);
        assert_eq!(issues[0].message, "stable Rust diagnostic");
    }
}

#[test]
fn registry_bundle_and_raw_locator_failures_never_launch() {
    let registry = tracked_registry();
    let launches = Cell::new(0);
    let base = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    for (option, value, expected) in [
        (
            "--require-release-registry-id",
            "mpk.release.registry.future",
            "FRONTEND_REGISTRY_ASSERTION",
        ),
        (
            "--require-release-registry-sha256",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "FRONTEND_REGISTRY_ASSERTION",
        ),
        (
            "--frontend-bundle",
            "frontend.go.go2vir.v0",
            "FRONTEND_BUNDLE_INCOMPATIBLE",
        ),
        (
            "--toolchain-bundle",
            "toolchain.go.go1.25.0.linux-amd64.v0",
            "FRONTEND_BUNDLE_INCOMPATIBLE",
        ),
    ] {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("out")).unwrap();
        let mut argv = base.clone();
        replace_option(&mut argv, option, value);
        let error = run_policy_scan_v1_with(
            &argv,
            temporary.path(),
            Vec::new(),
            |release| {
                registry.resolve(release).map(|_| ()).map_err(|error| {
                    PolicyScanV1Error::new(error.code(), "registered tuple selection failed")
                })
            },
            |(), _| {
                launches.set(launches.get() + 1);
                unreachable!("release selection failure must prevent launch")
            },
        )
        .unwrap_err();
        assert_eq!(error.code(), expected);
        assert!(!temporary.path().join("out/scan.json").exists());
    }
    assert_eq!(launches.get(), 0);

    for locator in [
        "--frontend",
        "--frontend-helper",
        "--driver",
        "--removed-frontend",
        "--toolchain-root",
        "--toolchain-path",
        "--registry",
        "--registry-path",
        "--release-registry-path",
    ] {
        let mut argv = base.clone();
        argv.extend([locator.to_owned(), "/tmp/private".to_owned()]);
        assert_eq!(
            parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
            "POLICY_CLI_FORBIDDEN_LOCATOR",
            "{locator}"
        );
    }

    for package in ["2vector", "vector.core", "vector/core"] {
        let mut argv = base.clone();
        replace_option(&mut argv, "--package", package);
        assert_eq!(
            parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
            "POLICY_CLI_SCALAR",
            "{package:?}"
        );
    }

    for function in [
        "::identity".to_owned(),
        format!("{}::identity", "a".repeat(256)),
    ] {
        let mut invalid = base.clone();
        replace_option(&mut invalid, "--function", &function);
        assert_eq!(
            parse_policy_scan_v1_argv(&invalid).unwrap_err().code(),
            "POLICY_CLI_SCALAR",
            "{function:?}"
        );
    }
}

#[test]
fn outer_protocol_failure_writes_no_scan() {
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir(temporary.path().join("out")).unwrap();
    let argv = rust_scan_argv(
        "x86_64-unknown-linux-gnu",
        "vector::identity",
        &["contracts/vector.json"],
        "out/scan.json",
    );
    let error = run_policy_scan_v1_with(
        &argv,
        temporary.path(),
        Vec::new(),
        |_| Ok(()),
        |(), request| {
            let protocol = validate_frontend_process(
                FrontendProtocolRequest {
                    source_language: &request.release.source_language,
                    semantic_profile: &request.release.semantic_profile,
                    semantic_parameters: request.semantic_parameters,
                    selection: request.selection,
                    release_registry: None,
                    captured_inputs: request.captured_inputs,
                },
                FrontendProcessFacts {
                    exit_code: Some(0),
                    signaled: false,
                    stdout: b"{}\n",
                    stderr_observed_bytes: 0,
                },
            )
            .unwrap_err();
            Err(PolicyScanV1Error::new(
                protocol.code().as_str(),
                "frontend protocol rejected",
            ))
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "FRONTEND_PROTOCOL_SHAPE");
    assert!(!temporary.path().join("out/scan.json").exists());
}

fn run_with_registered_release(
    argv: &[String],
    working_directory: &Path,
    inputs: Vec<OwnedCapturedInput>,
    accepted: AcceptedFrontendRun,
    launches: &Cell<usize>,
) -> policy_scan::v1::PolicyScanV1RunOutput {
    let registry = accepted.registry.clone();
    run_policy_scan_v1_with(
        argv,
        working_directory,
        inputs,
        |request| {
            let resolved = registry.resolve(request).map_err(|error| {
                PolicyScanV1Error::new(error.code(), "registered tuple selection failed")
            })?;
            assert_eq!(resolved.frontend.name, "rust2vir");
            assert_eq!(resolved.frontend.main.path, "bin/rust2vir");
            assert_eq!(resolved.frontend.subordinate_binaries.len(), 1);
            assert_eq!(
                resolved.frontend.subordinate_binaries[0].name,
                "rust2vir-driver"
            );
            assert_eq!(
                resolved.frontend.subordinate_binaries[0].path,
                "bin/rust2vir-driver"
            );
            assert!(matches!(
                &resolved.toolchain.compiler,
                CompilerIdentity::Rust { release, rustc_commit }
                    if release == "1.89.0-nightly"
                        && rustc_commit == "4d08223c054cf5a56d9761ca925fd46ffebe7115"
            ));
            Ok(request.clone())
        },
        |prepared: ReleaseSelectionRequest, request| {
            launches.set(launches.get() + 1);
            assert_eq!(prepared, request.release);
            assert_eq!(
                request.contracts,
                request
                    .contracts
                    .iter()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                request.semantic_parameters,
                accepted.envelope.value.get("semantic_parameters").unwrap()
            );
            assert_eq!(
                request.selection,
                accepted.envelope.value.get("selection").unwrap()
            );
            Ok(accepted.clone())
        },
    )
    .unwrap()
    .unwrap()
}

fn accepted_ready_run(envelope_path: &Path, inputs: &[OwnedCapturedInput]) -> AcceptedFrontendRun {
    validate_ready_run(envelope_path, inputs).unwrap()
}

fn validate_ready_run(
    envelope_path: &Path,
    inputs: &[OwnedCapturedInput],
) -> Result<AcceptedFrontendRun, frontend_protocol::FrontendProtocolError> {
    validate_ready_run_at_boundary(envelope_path, inputs, true)
}

fn validate_ready_run_exact(
    envelope_path: &Path,
    inputs: &[OwnedCapturedInput],
) -> Result<AcceptedFrontendRun, frontend_protocol::FrontendProtocolError> {
    validate_ready_run_at_boundary(envelope_path, inputs, false)
}

fn validate_ready_run_at_boundary(
    envelope_path: &Path,
    inputs: &[OwnedCapturedInput],
    staging: bool,
) -> Result<AcceptedFrontendRun, frontend_protocol::FrontendProtocolError> {
    let bytes = fs::read(envelope_path).unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let registry = tracked_registry();
    let captured = inputs
        .iter()
        .map(OwnedCapturedInput::as_ref)
        .collect::<Vec<_>>();
    let process = FrontendProcessFacts {
        exit_code: Some(0),
        signaled: false,
        stdout: &bytes,
        stderr_observed_bytes: 0,
    };
    let accepted = if staging {
        validate_frontend_process_from_staging(
            FrontendStagingRequest {
                source_language: "rust",
                semantic_profile: "mpk.rust.checked.v0",
                semantic_parameters: &value["semantic_parameters"],
                selection: &value["selection"],
                release_registry: Some(&registry),
                available_inputs: &captured,
            },
            process,
        )
    } else {
        validate_frontend_process(
            FrontendProtocolRequest {
                source_language: "rust",
                semantic_profile: "mpk.rust.checked.v0",
                semantic_parameters: &value["semantic_parameters"],
                selection: &value["selection"],
                release_registry: Some(&registry),
                captured_inputs: &captured,
            },
            process,
        )
    }?;
    Ok(AcceptedFrontendRun {
        envelope: accepted,
        release: release_from_manifest(&value["source_manifest"]),
        registry,
    })
}

fn accepted_go_non_success_run(code: &str) -> AcceptedFrontendRun {
    let parameters = json!({"target_id":"linux/amd64","pointer_width":64});
    let selection = json!({
        "package":"example.com/mpk/vector",
        "function":"example.com/mpk/vector.Identity"
    });
    let value = json!({
        "schema":"mpk.frontend.cli.v0",
        "status":"rejected",
        "phase":"subset",
        "source_language":"go",
        "semantic_profile":"mpk.go.fixed.v0",
        "semantic_parameters":parameters,
        "selection":selection,
        "rejected_features":[{
            "code":code,
            "message":"explicit contract paths do not equal discovered candidates",
            "function_id":"example.com/mpk/vector.Identity"
        }],
        "diagnostics":[],
    });
    let bytes = canonical_transport(&value);
    let envelope = validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "go",
            semantic_profile: "mpk.go.fixed.v0",
            semantic_parameters: &parameters,
            selection: &selection,
            release_registry: None,
            captured_inputs: &[],
        },
        FrontendProcessFacts {
            exit_code: Some(3),
            signaled: false,
            stdout: &bytes,
            stderr_observed_bytes: 0,
        },
    )
    .unwrap();
    let scan_vectors = read_json(&repo_root().join("develop/specs/vectors/policy-scan-v1.json"));
    let context = scan_vectors["linkage_contexts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|context| context["id"] == "context.go_identity_ready")
        .unwrap();
    AcceptedFrontendRun {
        envelope,
        release: FrontendReleaseIdentity {
            release_registry: serde_json::from_value(context["release_registry"].clone()).unwrap(),
            frontend: serde_json::from_value(context["frontend"].clone()).unwrap(),
            toolchain: serde_json::from_value(context["toolchain"].clone()).unwrap(),
            limit_profile: "mpk.vir.limits.v0".to_owned(),
        },
        registry: synthetic_registry(),
    }
}

fn accepted_non_success_run(
    status: &str,
    phase: &str,
    exit: i32,
    code: &str,
) -> AcceptedFrontendRun {
    let parameters = json!({
        "target_id":"x86_64-unknown-linux-gnu",
        "pointer_width":64,
        "overflow_mode":"checked",
        "panic_mode":"abort"
    });
    let selection = json!({
        "package":"vector",
        "crate":"vector",
        "kind":"lib",
        "function":"vector::identity"
    });
    let issue = json!({
        "code":code,
        "message":"stable Rust diagnostic",
        "function_id":"vector::identity"
    });
    let value = json!({
        "schema":"mpk.frontend.cli.v0",
        "status":status,
        "phase":phase,
        "source_language":"rust",
        "semantic_profile":"mpk.rust.checked.v0",
        "semantic_parameters":parameters,
        "selection":selection,
        "rejected_features":if status == "rejected" { json!([issue]) } else { json!([]) },
        "diagnostics":if status == "rejected" { json!([]) } else { json!([issue]) },
    });
    let bytes = canonical_transport(&value);
    let accepted = validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "rust",
            semantic_profile: "mpk.rust.checked.v0",
            semantic_parameters: &parameters,
            selection: &selection,
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
    .unwrap();
    let identity_fixture = read_json(&repo_root().join(
        "fixtures/rust-basic/positive/usize-targets/artifacts/x86_64/frontend-envelope.json",
    ));
    AcceptedFrontendRun {
        envelope: accepted,
        release: release_from_manifest(&identity_fixture["source_manifest"]),
        registry: tracked_registry(),
    }
}

fn captured_case_inputs(case: &Path, envelope_path: &Path) -> Vec<OwnedCapturedInput> {
    let envelope = read_json(envelope_path);
    envelope["source_manifest"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| {
            let normalized_path = input["normalized_path"].as_str().unwrap();
            let source = match normalized_path {
                "Cargo.toml" | "Cargo.lock" => repo_root()
                    .join("fixtures/rust-basic")
                    .join(normalized_path),
                _ => case.join("source").join(normalized_path),
            };
            OwnedCapturedInput {
                kind: match input["kind"].as_str().unwrap() {
                    "source" => InputKind::Source,
                    "contract" => InputKind::Contract,
                    "build_manifest" => InputKind::BuildManifest,
                    "lockfile" => InputKind::Lockfile,
                    kind => panic!("unknown fixture input kind {kind}"),
                },
                normalized_path: normalized_path.to_owned(),
                bytes: fs::read(source).unwrap(),
            }
        })
        .collect()
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

fn synthetic_registry() -> ValidatedReleaseRegistry {
    let vectors = read_json(&repo_root().join("develop/specs/vectors/release-bundles-v0.json"));
    validate_release_registry(&canonical_transport(&vectors["fixtures"]["valid_registry"])).unwrap()
}

fn rust_scan_argv(target: &str, function: &str, contracts: &[&str], output: &str) -> Vec<String> {
    let mut argv = vec![
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
        target,
        "--package",
        "vector",
        "--function",
        function,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for contract in contracts {
        argv.extend(["--contract".to_owned(), (*contract).to_owned()]);
    }
    argv.extend(["--json-out".to_owned(), output.to_owned()]);
    argv
}

fn replace_option(argv: &mut [String], option: &str, value: &str) {
    let position = argv.iter().position(|argument| argument == option).unwrap();
    argv[position + 1] = value.to_owned();
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
