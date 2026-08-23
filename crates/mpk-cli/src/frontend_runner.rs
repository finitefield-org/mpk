use crate::frontend_protocol::{
    validate_frontend_process_from_staging, AcceptedFrontendEnvelope, FrontendProcessFacts,
    FrontendProtocolCode, FrontendProtocolError, FrontendStagingRequest,
};
use crate::frontend_registry::{
    assert_embedded_registry, FrontendReleaseCode, InstalledReleaseResolver,
    SelectedFrontendRelease,
};
use crate::frontend_sandbox::{
    launch_release_frontend, prepare_release_sandbox, PreparedSandbox, SandboxError,
};
use mpk_vc::{
    CapturedInput, CompilerIdentity, ComponentIdentity, FrontendIdentity, ReleaseRegistryIdentity,
    ReleaseSelectionRequest, SubordinateIdentity, ToolchainComponent, ToolchainIdentity,
    ValidatedReleaseRegistry,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const GO_FRONTEND_ARGUMENT_BYTES_MAX: usize = 262_144;
const GO_FRONTEND_CONTRACTS_MAX: usize = 128;

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
    /// Retained validated registry bytes and entries are needed to validate
    /// the same source-manifest lifecycle after VC generation.
    pub(crate) registry: ValidatedReleaseRegistry,
}

pub(crate) struct PreparedFrontendRun {
    selected: SelectedFrontendRelease,
    release: ReleaseSelectionRequest,
    sandbox: PreparedSandbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrontendRunCode {
    Release(FrontendReleaseCode),
    ProcessSpawn,
    ProcessKilled,
    Protocol(FrontendProtocolCode),
}

impl FrontendRunCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Release(code) => code.as_str(),
            Self::ProcessSpawn => "FRONTEND_PROCESS_SPAWN",
            Self::ProcessKilled => "FRONTEND_PROCESS_KILLED",
            Self::Protocol(code) => code.as_str(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct FrontendRunError {
    code: FrontendRunCode,
}

impl FrontendRunError {
    pub(crate) const fn code(&self) -> FrontendRunCode {
        self.code
    }
}

impl fmt::Display for FrontendRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for FrontendRunError {}

pub(crate) fn prepare_installed_frontend(
    release: &ReleaseSelectionRequest,
) -> Result<PreparedFrontendRun, FrontendRunError> {
    assert_embedded_registry(release).map_err(|error| FrontendRunError {
        code: FrontendRunCode::Release(error.code()),
    })?;
    let resolver = InstalledReleaseResolver::open().map_err(|error| FrontendRunError {
        code: FrontendRunCode::Release(error.code()),
    })?;
    let selected = resolver
        .resolve(release)
        .map_err(|error| FrontendRunError {
            code: FrontendRunCode::Release(error.code()),
        })?;
    let sandbox = prepare_release_sandbox().map_err(|_| FrontendRunError {
        code: FrontendRunCode::Release(FrontendReleaseCode::SandboxUnavailable),
    })?;
    Ok(PreparedFrontendRun {
        selected,
        release: release.clone(),
        sandbox,
    })
}

pub(crate) fn run_prepared_frontend(
    prepared: PreparedFrontendRun,
    request: FrontendRunRequest<'_>,
) -> Result<AcceptedFrontendRun, FrontendRunError> {
    let PreparedFrontendRun {
        selected,
        release,
        sandbox,
    } = prepared;
    if request.release != release {
        return Err(FrontendRunError {
            code: FrontendRunCode::Release(FrontendReleaseCode::BundleIncompatible),
        });
    }
    let environment = registered_environment(&selected, &request.release)?;
    let args = registered_arguments(&selected, &request)?;
    let release = release_identity(&selected);
    let output = launch_release_frontend(
        sandbox,
        &selected.frontend_snapshot,
        &selected.toolchain_snapshot,
        &selected.frontend.main.path,
        &args,
        &environment,
        request.captured_inputs,
        request.staged_directories,
        request.staged_placeholders,
    )
    .map_err(|error| FrontendRunError {
        code: match error {
            SandboxError::Unavailable => {
                FrontendRunCode::Release(FrontendReleaseCode::SandboxUnavailable)
            }
            SandboxError::Spawn => FrontendRunCode::ProcessSpawn,
            SandboxError::Killed => FrontendRunCode::ProcessKilled,
        },
    })?;
    if output.stream_limit_exceeded {
        return Err(FrontendRunError {
            code: FrontendRunCode::Protocol(FrontendProtocolCode::ProtocolLimit),
        });
    }
    let envelope = validate_frontend_process_from_staging(
        FrontendStagingRequest {
            source_language: &request.release.source_language,
            semantic_profile: &request.release.semantic_profile,
            semantic_parameters: request.semantic_parameters,
            selection: request.selection,
            release_registry: Some(&selected.registry),
            available_inputs: request.captured_inputs,
        },
        FrontendProcessFacts {
            exit_code: output.exit_code,
            signaled: output.signaled,
            stdout: &output.stdout,
            stderr_observed_bytes: output.stderr_observed_bytes,
        },
    )
    .map_err(protocol_error)?;
    Ok(AcceptedFrontendRun {
        envelope,
        release,
        registry: selected.registry,
    })
}

fn registered_arguments(
    selected: &SelectedFrontendRelease,
    request: &FrontendRunRequest<'_>,
) -> Result<Vec<String>, FrontendRunError> {
    match request.release.source_language.as_str() {
        "go" => registered_go_arguments(selected, request),
        "rust" => registered_rust_arguments(selected, request),
        _ => Err(bundle_incompatible()),
    }
}

fn registered_go_arguments(
    selected: &SelectedFrontendRelease,
    request: &FrontendRunRequest<'_>,
) -> Result<Vec<String>, FrontendRunError> {
    if selected.frontend.source_language != "go"
        || selected.toolchain.source_language != "go"
        || selected.pointer_width != 64
        || selected.limit_profile_id != "mpk.vir.limits.v0"
        || selected.frontend.limit_profile_id != "mpk.vir.limits.v0"
        || selected.frontend.argument_profile_id != "mpk.go.frontend_arguments.v0"
        || !selected.frontend.subordinate_binaries.is_empty()
        || request.release.source_language != "go"
        || request.release.semantic_profile != "mpk.go.fixed.v0"
        || request.release.target_id != "linux/amd64"
        || !exact_go_semantic_parameters(request.semantic_parameters)
    {
        return Err(bundle_incompatible());
    }
    let selection = request
        .selection
        .as_object()
        .ok_or_else(bundle_incompatible)?;
    if selection.len() != 2
        || !selection.contains_key("package")
        || !selection.contains_key("function")
    {
        return Err(bundle_incompatible());
    }
    let package = request
        .selection
        .get("package")
        .and_then(Value::as_str)
        .ok_or_else(bundle_incompatible)?;
    let function = request
        .selection
        .get("function")
        .and_then(Value::as_str)
        .ok_or_else(bundle_incompatible)?;
    let contracts = normalized_contract_arguments(request.contracts)?;
    let mut arguments = vec![
        "lower".to_owned(),
        "/mpk/source".to_owned(),
        "--package".to_owned(),
        package.to_owned(),
        "--semantic-profile".to_owned(),
        request.release.semantic_profile.clone(),
        "--target".to_owned(),
        request.release.target_id.clone(),
        "--function".to_owned(),
        function.to_owned(),
        "--frontend-bundle-id".to_owned(),
        selected.frontend.bundle_id.clone(),
        "--frontend-sha256".to_owned(),
        selected.frontend.main.binary_sha256.clone(),
        "--release-registry-id".to_owned(),
        selected.registry_id.clone(),
        "--release-registry-sha256".to_owned(),
        selected.registry_sha256.clone(),
        "--toolchain-bundle-id".to_owned(),
        selected.toolchain.bundle_id.clone(),
        "--toolchain-root".to_owned(),
        "/mpk/toolchain".to_owned(),
        "--toolchain-distribution-sha256".to_owned(),
        selected.toolchain.distribution_sha256.clone(),
    ];
    for contract in contracts {
        arguments.push("--contract".to_owned());
        arguments.push(contract);
    }
    let argument_bytes = arguments.iter().try_fold(0usize, |total, argument| {
        total.checked_add(argument.len() + 1)
    });
    if argument_bytes.is_none_or(|bytes| bytes > GO_FRONTEND_ARGUMENT_BYTES_MAX) {
        return Err(bundle_incompatible());
    }
    Ok(arguments)
}

fn registered_rust_arguments(
    selected: &SelectedFrontendRelease,
    request: &FrontendRunRequest<'_>,
) -> Result<Vec<String>, FrontendRunError> {
    if selected.frontend.source_language != "rust"
        || selected.toolchain.source_language != "rust"
        || rust_pointer_width(&request.release.target_id) != Some(selected.pointer_width)
        || selected.limit_profile_id != "mpk.vir.limits.v0"
        || selected.frontend.limit_profile_id != "mpk.vir.limits.v0"
        || selected.frontend.argument_profile_id != "mpk.rust.frontend_arguments.v0"
        || selected.frontend.subordinate_binaries.len() != 1
        || selected.frontend.subordinate_binaries[0].name != "rust2vir-driver"
        || selected.frontend.subordinate_binaries[0].path != "bin/rust2vir-driver"
        || request.release.semantic_profile != "mpk.rust.checked.v0"
        || !exact_rust_semantic_parameters(request.semantic_parameters, &request.release)
    {
        return Err(bundle_incompatible());
    }
    let selection = request
        .selection
        .as_object()
        .ok_or_else(bundle_incompatible)?;
    if selection.len() != 4 || selection.get("kind").and_then(Value::as_str) != Some("lib") {
        return Err(bundle_incompatible());
    }
    let package = selection
        .get("package")
        .and_then(Value::as_str)
        .ok_or_else(bundle_incompatible)?;
    let crate_name = selection
        .get("crate")
        .and_then(Value::as_str)
        .ok_or_else(bundle_incompatible)?;
    let function = selection
        .get("function")
        .and_then(Value::as_str)
        .ok_or_else(bundle_incompatible)?;
    if !rust_package_name(package)
        || !rust_identifier(crate_name)
        || !rust_function_id(function, crate_name)
    {
        return Err(bundle_incompatible());
    }
    let contracts = normalized_contract_arguments(request.contracts)?;
    let driver = &selected.frontend.subordinate_binaries[0];
    let mut arguments = vec![
        "lower".to_owned(),
        "/mpk/source".to_owned(),
        "--manifest-path".to_owned(),
        "Cargo.toml".to_owned(),
        "--package".to_owned(),
        package.to_owned(),
        "--semantic-profile".to_owned(),
        request.release.semantic_profile.clone(),
        "--target".to_owned(),
        request.release.target_id.clone(),
        "--function".to_owned(),
        function.to_owned(),
        "--frontend-bundle-id".to_owned(),
        selected.frontend.bundle_id.clone(),
        "--frontend-sha256".to_owned(),
        selected.frontend.main.binary_sha256.clone(),
        "--release-registry-id".to_owned(),
        selected.registry_id.clone(),
        "--release-registry-sha256".to_owned(),
        selected.registry_sha256.clone(),
        "--toolchain-bundle-id".to_owned(),
        selected.toolchain.bundle_id.clone(),
        "--toolchain-root".to_owned(),
        "/mpk/toolchain".to_owned(),
        "--toolchain-distribution-sha256".to_owned(),
        selected.toolchain.distribution_sha256.clone(),
        "--driver".to_owned(),
        "/mpk/frontend/bin/rust2vir-driver".to_owned(),
        "--driver-sha256".to_owned(),
        driver.binary_sha256.clone(),
    ];
    for contract in contracts {
        arguments.push("--contract".to_owned());
        arguments.push(contract);
    }
    let argument_bytes = arguments.iter().try_fold(0usize, |total, argument| {
        total.checked_add(argument.len() + 1)
    });
    if argument_bytes.is_none_or(|bytes| bytes > GO_FRONTEND_ARGUMENT_BYTES_MAX) {
        return Err(bundle_incompatible());
    }
    Ok(arguments)
}

fn exact_go_semantic_parameters(parameters: &Value) -> bool {
    parameters.as_object().is_some_and(|object| {
        object.len() == 2
            && object.get("target_id").and_then(Value::as_str) == Some("linux/amd64")
            && object.get("pointer_width").and_then(Value::as_i64) == Some(64)
    })
}

fn exact_rust_semantic_parameters(parameters: &Value, release: &ReleaseSelectionRequest) -> bool {
    let Some(pointer_width) = rust_pointer_width(&release.target_id) else {
        return false;
    };
    parameters.as_object().is_some_and(|object| {
        object.len() == 4
            && object.get("target_id").and_then(Value::as_str) == Some(release.target_id.as_str())
            && object.get("pointer_width").and_then(Value::as_i64) == Some(pointer_width)
            && object.get("overflow_mode").and_then(Value::as_str) == Some("checked")
            && object.get("panic_mode").and_then(Value::as_str) == Some("abort")
    })
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

fn normalized_contract_arguments(contracts: &[String]) -> Result<Vec<String>, FrontendRunError> {
    if contracts.len() > GO_FRONTEND_CONTRACTS_MAX {
        return Err(bundle_incompatible());
    }
    let mut folded = BTreeSet::new();
    for contract in contracts {
        if mpk_vc::validate_manifest_normalized_path(contract).is_err()
            || !folded.insert(contract.to_ascii_lowercase())
        {
            return Err(bundle_incompatible());
        }
    }
    let mut normalized = contracts.to_vec();
    normalized.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(normalized)
}

fn bundle_incompatible() -> FrontendRunError {
    FrontendRunError {
        code: FrontendRunCode::Release(FrontendReleaseCode::BundleIncompatible),
    }
}

fn release_identity(selected: &SelectedFrontendRelease) -> FrontendReleaseIdentity {
    let rustc_commit = match &selected.toolchain.compiler {
        CompilerIdentity::Rust { rustc_commit, .. } => Some(rustc_commit.as_str()),
        CompilerIdentity::Go { .. } => None,
    };
    let components = selected
        .toolchain
        .components
        .iter()
        .map(|component| match component {
            ToolchainComponent::Executable {
                name,
                release,
                binary_sha256,
                ..
            } => ComponentIdentity::Executable {
                name: name.clone(),
                release: release.clone(),
                commit_hash: (name == "rustc")
                    .then(|| rustc_commit.map(str::to_owned))
                    .flatten(),
                binary_sha256: binary_sha256.clone(),
            },
            ToolchainComponent::Content {
                name,
                release,
                content_sha256,
                ..
            } => ComponentIdentity::Content {
                name: name.clone(),
                release: release.clone(),
                content_sha256: content_sha256.clone(),
            },
        })
        .collect();
    FrontendReleaseIdentity {
        release_registry: ReleaseRegistryIdentity {
            schema: selected.registry.registry().schema.clone(),
            id: selected.registry_id.clone(),
            registry_sha256: selected.registry_sha256.clone(),
        },
        frontend: FrontendIdentity {
            bundle_id: selected.frontend.bundle_id.clone(),
            name: selected.frontend.name.clone(),
            version: selected.frontend.version.clone(),
            binary_sha256: selected.frontend.main.binary_sha256.clone(),
            subordinate_binaries: selected
                .frontend
                .subordinate_binaries
                .iter()
                .map(|binary| SubordinateIdentity {
                    name: binary.name.clone(),
                    version: binary.version.clone(),
                    binary_sha256: binary.binary_sha256.clone(),
                })
                .collect(),
        },
        toolchain: ToolchainIdentity {
            bundle_id: selected.toolchain.bundle_id.clone(),
            distribution_sha256: selected.toolchain.distribution_sha256.clone(),
            components,
        },
        limit_profile: selected.limit_profile_id.clone(),
    }
}

fn registered_environment(
    selected: &SelectedFrontendRelease,
    request: &ReleaseSelectionRequest,
) -> Result<BTreeMap<String, String>, FrontendRunError> {
    if selected.frontend.source_language == "rust"
        && selected.frontend.environment_profile_id == "mpk.rust.frontend_environment.v0"
        && matches!(
            request.target_id.as_str(),
            "i686-unknown-linux-gnu" | "x86_64-unknown-linux-gnu"
        )
    {
        return Ok([
            ("HOME", "/mpk/empty/home"),
            ("LANG", "C"),
            ("LC_ALL", "C"),
            ("PATH", "/mpk/toolchain/bin"),
            ("TMPDIR", "/mpk/tmp"),
            ("TZ", "UTC"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect());
    }
    if selected.frontend.source_language != "go"
        || selected.frontend.environment_profile_id != "mpk.go.frontend_environment.v0"
        || request.target_id != "linux/amd64"
    {
        return Err(FrontendRunError {
            code: FrontendRunCode::Release(FrontendReleaseCode::BundleIncompatible),
        });
    }
    Ok([
        ("CGO_ENABLED", "0"),
        ("GO111MODULE", "on"),
        ("GOAMD64", "v1"),
        ("GOARCH", "amd64"),
        ("GOCACHE", "/mpk/cache/go-build"),
        ("GODEBUG", ""),
        ("GOENV", "off"),
        ("GOEXPERIMENT", ""),
        ("GOFLAGS", ""),
        ("GOMAXPROCS", "1"),
        ("GOMODCACHE", "/mpk/cache/go-mod"),
        ("GONOPROXY", ""),
        ("GONOSUMDB", ""),
        ("GOOS", "linux"),
        ("GOPACKAGESDRIVER", "off"),
        ("GOPATH", "/mpk/gopath"),
        ("GOPRIVATE", ""),
        ("GOPROXY", "off"),
        ("GOROOT", "/mpk/toolchain/go"),
        ("GOSUMDB", "off"),
        ("GOTELEMETRY", "off"),
        ("GOTOOLCHAIN", "local"),
        ("GOVCS", "*:off"),
        ("GOWORK", "off"),
        ("HOME", "/mpk/empty/home"),
        ("LANG", "C"),
        ("LC_ALL", "C"),
        ("PATH", "/mpk/toolchain/go/bin"),
        ("TMPDIR", "/mpk/tmp"),
        ("TZ", "UTC"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect())
}

fn protocol_error(error: FrontendProtocolError) -> FrontendRunError {
    FrontendRunError {
        code: FrontendRunCode::Protocol(error.code()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend_registry::SnapshotFile;
    use crate::frontend_sandbox::launch_snapshot_for_test;
    use sha2::{Digest, Sha256};

    #[test]
    fn validated_executable_bytes_are_snapshotted_before_launch() {
        let original = b"#!/bin/sh\nprintf '%s' snapshot-original\n".to_vec();
        let digest = Sha256::digest(&original)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let snapshot = SnapshotFile {
            bytes: original,
            executable: true,
            sha256: digest,
        };
        let mut replaced_path_bytes = b"#!/bin/sh\nprintf '%s' replacement\n".to_vec();
        replaced_path_bytes.fill(b'x');
        let output =
            launch_snapshot_for_test(&snapshot, &[], &BTreeMap::new()).expect("snapshot launches");
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.stdout, b"snapshot-original");
    }

    #[test]
    fn registered_contract_arguments_are_sorted_and_closed() {
        let contracts = vec!["contracts/z.json".to_owned(), "contracts/a.json".to_owned()];
        assert_eq!(
            normalized_contract_arguments(&contracts).expect("registered contract arguments"),
            ["contracts/a.json", "contracts/z.json"]
        );

        let collision = vec!["contracts/a.json".to_owned(), "contracts/A.json".to_owned()];
        assert_eq!(
            normalized_contract_arguments(&collision)
                .expect_err("ASCII case-fold collisions reject")
                .code(),
            FrontendRunCode::Release(FrontendReleaseCode::BundleIncompatible)
        );

        let too_many = (0..=GO_FRONTEND_CONTRACTS_MAX)
            .map(|index| format!("contracts/c{index}.json"))
            .collect::<Vec<_>>();
        assert!(normalized_contract_arguments(&too_many).is_err());
    }

    #[test]
    fn registered_rust_selection_uses_the_frontend_identifier_grammar() {
        assert_eq!(rust_pointer_width("i686-unknown-linux-gnu"), Some(32));
        assert_eq!(rust_pointer_width("x86_64-unknown-linux-gnu"), Some(64));
        assert_eq!(rust_pointer_width("aarch64-unknown-linux-gnu"), None);
        for package in ["vector", "vector-core", "vector_2"] {
            assert!(rust_package_name(package));
        }
        for package in ["", "2vector", "vector.core"] {
            assert!(!rust_package_name(package));
        }
        assert!(!rust_package_name(&"a".repeat(1_025)));
        for identifier in ["vector", "_vector", "vector_2"] {
            assert!(rust_identifier(identifier));
        }
        for identifier in ["", "_", "2vector", "vector-core", "véctor"] {
            assert!(!rust_identifier(identifier));
        }
        assert!(!rust_identifier(&"a".repeat(256)));
        assert!(rust_function_id("vector::identity", "vector"));
        assert!(rust_function_id("vector::module_2::identity", "vector"));
        assert!(!rust_function_id(
            &format!("vector::{}", vec!["a"; 510].join("::")),
            "vector"
        ));
        for function in [
            "vector",
            "::identity",
            "other::identity",
            "vector::",
            "vector::2identity",
        ] {
            assert!(!rust_function_id(function, "vector"));
        }
    }

    #[test]
    fn registry_assertions_reject_before_installed_release_access() {
        let request = ReleaseSelectionRequest {
            registry_id: "mpk.release.registry.v0".to_owned(),
            registry_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_owned(),
            source_language: "rust".to_owned(),
            semantic_profile: "mpk.rust.checked.v0".to_owned(),
            target_id: "x86_64-unknown-linux-gnu".to_owned(),
            frontend_bundle_id: Some("frontend.rust.rust2vir.candidate.v0".to_owned()),
            toolchain_bundle_id: Some("toolchain.rust.nightly-2025-06-01.candidate.v0".to_owned()),
        };
        let error = match prepare_installed_frontend(&request) {
            Ok(_) => panic!("wrong embedded assertion reached resolver access"),
            Err(error) => error,
        };
        assert_eq!(
            error.code(),
            FrontendRunCode::Release(FrontendReleaseCode::RegistryAssertion)
        );
    }
}
