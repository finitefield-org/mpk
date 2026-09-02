//! Descriptor-relative C# runner for the active successor release.
//!
//! The entry point opens only the release root beside the already executing
//! `bin/mpk`, validates both build-pinned registries, snapshots the selected
//! bundle bytes, and launches that snapshot through the hardened Linux
//! sandbox.

use crate::frontend_protocol::FrontendProcessFacts;
use crate::frontend_registry::{
    BundleSnapshot, FrontendReleaseCode, InstalledSuccessorRelease, EXPECTED_REGISTRY_SHA256_HEX,
    EXPECTED_SEMANTIC_REGISTRY_SHA256,
};
use crate::frontend_sandbox::{
    launch_csharp_frontend, launch_release_frontend, prepare_release_sandbox, SandboxError,
};
use crate::java_frontend_runner::{JavaRunError, PreparedJavaRun};
use crate::successor_frontend_protocol::{
    validate_successor_frontend_process, AcceptedSuccessorFrontendEnvelope,
    SuccessorFrontendProtocolCode, SuccessorFrontendProtocolRequest,
};
use crate::successor_release_bundle::{
    validate_successor_release_registry, ResolvedSuccessorRelease,
    SuccessorReleaseSelectionRequest, ValidatedSuccessorReleaseRegistry, CSHARP_FRONTEND_BUNDLE_ID,
    CSHARP_FRONTEND_SHA256, CSHARP_HOST_PROFILE_ID, CSHARP_RUNTIME_LAYOUT_ID,
    CSHARP_TOOLCHAIN_BUNDLE_ID, GO_FRONTEND_BUNDLE_ID, GO_TOOLCHAIN_BUNDLE_ID,
    RUST_FRONTEND_BUNDLE_ID, RUST_TOOLCHAIN_BUNDLE_ID, SUCCESSOR_RELEASE_REGISTRY_ID,
    SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_semantic_profile_registry, RegistryRevision,
    SelectionEnvelope, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{
    BundleInventory, CapturedInput, ComponentIdentity, ExecutableRuntime, FrontendIdentity,
    InputKind, ReleaseRegistryIdentity, SourceReference, SubordinateIdentity, SyntheticPermission,
    ToolchainComponent, ToolchainIdentity,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

const CSHARP_PROFILE_ENTRY_SHA256: &str =
    "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac";
const CSHARP_ARGUMENT_BYTES_MAX: usize = 131_072;
const FRONTEND_ARGUMENT_BYTES_MAX: usize = 262_144;
const FRONTEND_CONTRACTS_MAX: usize = 128;
const DOTNET_PROGRAM: &str = "/mpk/toolchain/dotnet/dotnet";
const SOURCE_ROOT: &str = "/mpk/source";
const ROSLYN_CSHARP_SHA256: &str =
    "1af1de8a162d2312eb2f6b781f5edbe8cec7d5cd268c7e4de24396225e54260f";
const ROSLYN_COMMON_SHA256: &str =
    "42c9ce7891470f430267e2dc02d03571f9d046a7e7e121107754bee58d344613";
const DOTNET_RUNTIME_ARCHIVE_SHA256: &str =
    "7d847ecaa123efae40b114c5d45641e456b4cd65e5114b4612095d45d7c71a63";
const REFERENCE_INVENTORY_SHA256: &str =
    "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CSharpRunCode {
    Release,
    Selection,
    Contract,
    Sandbox,
    Launch,
    Process,
    Protocol(SuccessorFrontendProtocolCode),
    Identity,
}

impl CSharpRunCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "CSHARP_RELEASE",
            Self::Selection => "CSHARP_SELECTION",
            Self::Contract => "CSHARP_CONTRACT",
            Self::Sandbox => "CSHARP_SANDBOX",
            Self::Launch => "CSHARP_LAUNCH",
            Self::Process => "CSHARP_PROCESS",
            Self::Protocol(code) => code.as_str(),
            Self::Identity => "CSHARP_IDENTITY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSharpRunError {
    code: CSharpRunCode,
}

impl CSharpRunError {
    pub const fn code(&self) -> CSharpRunCode {
        self.code
    }
}

impl fmt::Display for CSharpRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for CSharpRunError {}

#[derive(Clone, Copy, Debug)]
pub struct CSharpRunRequest<'a> {
    pub semantic_context: &'a Value,
    pub selection: &'a Value,
    pub captured_inputs: &'a [CapturedInput<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSharpLauncherPlan {
    argv: Vec<String>,
    environment: BTreeMap<String, String>,
}

impl CSharpLauncherPlan {
    pub fn program(&self) -> &str {
        &self.argv[0]
    }

    pub fn working_directory(&self) -> &str {
        SOURCE_ROOT
    }

    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    pub fn environment(&self) -> &BTreeMap<String, String> {
        &self.environment
    }
}

#[derive(Clone, Debug)]
pub struct AcceptedCSharpRun {
    envelope: AcceptedSuccessorFrontendEnvelope,
    launcher: CSharpLauncherPlan,
    release_registry: ReleaseRegistryIdentity,
}

impl AcceptedCSharpRun {
    pub fn envelope(&self) -> &AcceptedSuccessorFrontendEnvelope {
        &self.envelope
    }

    pub fn launcher(&self) -> &CSharpLauncherPlan {
        &self.launcher
    }

    pub fn release_registry(&self) -> &ReleaseRegistryIdentity {
        &self.release_registry
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstalledFrontendRunRequest<'a> {
    pub semantic_context: &'a Value,
    pub selection: &'a Value,
    pub release_registry_id: &'a str,
    pub release_registry_sha256: &'a str,
    pub frontend_bundle_id: &'a str,
    pub toolchain_bundle_id: &'a str,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub synthetic_permissions: &'a [SyntheticPermission],
    pub staged_directories: &'a [&'a str],
    pub staged_placeholders: &'a [&'a str],
    pub contracts: &'a [String],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstalledFrontendRunCode {
    Release,
    Selection,
    Launch,
    Process,
    Protocol(SuccessorFrontendProtocolCode),
    Identity,
}

impl InstalledFrontendRunCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "FRONTEND_RELEASE",
            Self::Selection => "FRONTEND_SELECTION",
            Self::Launch => "FRONTEND_LAUNCH",
            Self::Process => "FRONTEND_PROCESS",
            Self::Protocol(code) => code.as_str(),
            Self::Identity => "FRONTEND_IDENTITY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledFrontendRunError {
    code: InstalledFrontendRunCode,
}

impl InstalledFrontendRunError {
    pub const fn code(&self) -> InstalledFrontendRunCode {
        self.code
    }
}

impl fmt::Display for InstalledFrontendRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for InstalledFrontendRunError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendLauncherPlan {
    program: String,
    argv: Vec<String>,
    environment: BTreeMap<String, String>,
}

impl FrontendLauncherPlan {
    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    pub fn environment(&self) -> &BTreeMap<String, String> {
        &self.environment
    }
}

#[derive(Clone, Debug)]
pub struct AcceptedInstalledFrontendRun {
    envelope: AcceptedSuccessorFrontendEnvelope,
    launcher: FrontendLauncherPlan,
    release_registry: ReleaseRegistryIdentity,
}

impl AcceptedInstalledFrontendRun {
    pub fn envelope(&self) -> &AcceptedSuccessorFrontendEnvelope {
        &self.envelope
    }

    pub fn launcher(&self) -> &FrontendLauncherPlan {
        &self.launcher
    }

    pub fn release_registry(&self) -> &ReleaseRegistryIdentity {
        &self.release_registry
    }
}

/// Runs exactly one registry-selected frontend from the installed successor
/// image. The caller supplies assertions, never paths; all four languages
/// pass through the same registry/context/selection/protocol chain.
pub fn run_installed_frontend(
    request: InstalledFrontendRunRequest<'_>,
) -> Result<AcceptedInstalledFrontendRun, InstalledFrontendRunError> {
    if request
        .semantic_context
        .get("source_language")
        .and_then(Value::as_str)
        == Some("java")
    {
        if request.release_registry_id != SUCCESSOR_RELEASE_REGISTRY_ID
            || request.release_registry_sha256 != EXPECTED_REGISTRY_SHA256_HEX
            || request.frontend_bundle_id != mpk_vc::java_release::FRONTEND_ID
            || request.toolchain_bundle_id != mpk_vc::java_release::TOOLCHAIN_ID
            || request.semantic_context
                != &mpk_vc::java_release::candidate().tuples[0].semantic_context
            || !request.synthetic_permissions.is_empty()
            || !request.staged_directories.is_empty()
            || !request.staged_placeholders.is_empty()
            || !request.contracts.is_empty()
        {
            return Err(installed_failure(InstalledFrontendRunCode::Selection));
        }
        let accepted = PreparedJavaRun::open()
            .map_err(map_java_error)?
            .run(request.selection, request.captured_inputs)
            .map_err(map_java_error)?;
        return Ok(AcceptedInstalledFrontendRun {
            envelope: accepted.envelope,
            launcher: FrontendLauncherPlan {
                program: mpk_vc::java_release::PROGRAM.to_owned(),
                argv: accepted.launcher.argv().to_vec(),
                environment: accepted.launcher.environment().clone(),
            },
            release_registry: accepted.release_registry,
        });
    }

    if request
        .semantic_context
        .get("source_language")
        .and_then(Value::as_str)
        == Some("csharp")
    {
        if request.release_registry_id != SUCCESSOR_RELEASE_REGISTRY_ID
            || request.release_registry_sha256 != EXPECTED_REGISTRY_SHA256_HEX
            || request.frontend_bundle_id != CSHARP_FRONTEND_BUNDLE_ID
            || request.toolchain_bundle_id != CSHARP_TOOLCHAIN_BUNDLE_ID
            || !request.synthetic_permissions.is_empty()
            || !request.staged_directories.is_empty()
            || !request.staged_placeholders.is_empty()
            || !request.contracts.is_empty()
        {
            return Err(installed_failure(InstalledFrontendRunCode::Selection));
        }
        let accepted = run_installed_csharp_frontend(CSharpRunRequest {
            semantic_context: request.semantic_context,
            selection: request.selection,
            captured_inputs: request.captured_inputs,
        })
        .map_err(|error| installed_failure(map_csharp_error(error.code())))?;
        let launcher = FrontendLauncherPlan {
            program: accepted.launcher.program().to_owned(),
            argv: accepted.launcher.argv().to_vec(),
            environment: accepted.launcher.environment().clone(),
        };
        return Ok(AcceptedInstalledFrontendRun {
            envelope: accepted.envelope,
            launcher,
            release_registry: accepted.release_registry,
        });
    }

    if !request.synthetic_permissions.is_empty() {
        return Err(installed_failure(InstalledFrontendRunCode::Selection));
    }

    run_installed_native_frontend(request)
}

pub fn run_installed_csharp_frontend(
    request: CSharpRunRequest<'_>,
) -> Result<AcceptedCSharpRun, CSharpRunError> {
    let installed = InstalledSuccessorRelease::open().map_err(release_error)?;
    let semantic_registry = validate_semantic_profile_registry(
        &installed.semantic_registry_bytes,
        RegistryRevision::Revision3,
    )
    .map_err(|_| failure(CSharpRunCode::Release))?;
    let registry =
        validate_successor_release_registry(&installed.registry_bytes, &semantic_registry)
            .map_err(|_| failure(CSharpRunCode::Release))?;
    if semantic_registry.identity().registry_sha256() != EXPECTED_SEMANTIC_REGISTRY_SHA256
        || registry.registry_sha256() != EXPECTED_REGISTRY_SHA256_HEX
    {
        return Err(failure(CSharpRunCode::Release));
    }
    validate_csharp_registry_layout(&registry)?;
    let resolved = registry
        .resolve(
            &semantic_registry,
            SuccessorReleaseSelectionRequest {
                semantic_context: request.semantic_context,
                frontend_bundle_id: CSHARP_FRONTEND_BUNDLE_ID,
                toolchain_bundle_id: CSHARP_TOOLCHAIN_BUNDLE_ID,
            },
        )
        .map_err(|_| failure(CSharpRunCode::Selection))?;
    validate_csharp_release(&semantic_registry, &resolved)?;
    let selection = validate_registry_selection_envelope(
        &semantic_registry,
        &resolved.semantic_context,
        request.selection,
    )
    .map_err(|_| failure(CSharpRunCode::Selection))?;
    validate_captured_selection(&selection, request.captured_inputs)?;
    let launcher = csharp_launcher_plan(&resolved, &selection)?;

    let expected = complete_inventory_set(&registry);
    let snapshots = installed
        .snapshot_selected_bundles(
            &expected,
            &resolved.frontend.bundle_id,
            &resolved.toolchain.bundle_id,
        )
        .map_err(release_error)?;
    let frontend = selected_snapshot(&snapshots, &resolved.frontend.bundle_id)?;
    let toolchain = selected_snapshot(&snapshots, &resolved.toolchain.bundle_id)?;
    let sandbox = prepare_release_sandbox(&resolved.toolchain.execution_host_profile_id)
        .map_err(|_| failure(CSharpRunCode::Sandbox))?;
    let output = launch_csharp_frontend(
        sandbox,
        frontend,
        toolchain,
        &launcher.argv[1..],
        &launcher.environment,
        request.captured_inputs,
        &[],
        &[],
    )
    .map_err(sandbox_error)?;
    if output.stream_limit_exceeded {
        return Err(failure(CSharpRunCode::Protocol(
            SuccessorFrontendProtocolCode::ProtocolLimit,
        )));
    }
    let release_registry = registry.release_identity();
    let envelope = validate_successor_frontend_process(
        SuccessorFrontendProtocolRequest {
            registry: &semantic_registry,
            semantic_context: &resolved.semantic_context,
            selection: &selection,
            release_registry: &release_registry,
            captured_inputs: request.captured_inputs,
            synthetic_permissions: &[] as &[SyntheticPermission],
        },
        FrontendProcessFacts {
            exit_code: output.exit_code,
            signaled: output.signaled,
            stdout: &output.stdout,
            stderr_observed_bytes: output.stderr_observed_bytes,
        },
    )
    .map_err(|error| failure(CSharpRunCode::Protocol(error.code())))?;
    validate_emitted_release_identity(&envelope, &resolved)?;
    Ok(AcceptedCSharpRun {
        envelope,
        launcher,
        release_registry,
    })
}

fn run_installed_native_frontend(
    request: InstalledFrontendRunRequest<'_>,
) -> Result<AcceptedInstalledFrontendRun, InstalledFrontendRunError> {
    let installed = InstalledSuccessorRelease::open().map_err(installed_release_error)?;
    let semantic_registry = validate_semantic_profile_registry(
        &installed.semantic_registry_bytes,
        RegistryRevision::Revision3,
    )
    .map_err(|_| installed_failure(InstalledFrontendRunCode::Release))?;
    let registry =
        validate_successor_release_registry(&installed.registry_bytes, &semantic_registry)
            .map_err(|_| installed_failure(InstalledFrontendRunCode::Release))?;
    if semantic_registry.identity().registry_sha256() != EXPECTED_SEMANTIC_REGISTRY_SHA256
        || registry.registry_sha256() != EXPECTED_REGISTRY_SHA256_HEX
        || request.release_registry_id != SUCCESSOR_RELEASE_REGISTRY_ID
        || request.release_registry_sha256 != EXPECTED_REGISTRY_SHA256_HEX
    {
        return Err(installed_failure(InstalledFrontendRunCode::Release));
    }
    let resolved = registry
        .resolve(
            &semantic_registry,
            SuccessorReleaseSelectionRequest {
                semantic_context: request.semantic_context,
                frontend_bundle_id: request.frontend_bundle_id,
                toolchain_bundle_id: request.toolchain_bundle_id,
            },
        )
        .map_err(|_| installed_failure(InstalledFrontendRunCode::Selection))?;
    let selection = validate_registry_selection_envelope(
        &semantic_registry,
        &resolved.semantic_context,
        request.selection,
    )
    .map_err(|_| installed_failure(InstalledFrontendRunCode::Selection))?;
    if !matches!(resolved.semantic_context.source_language(), "go" | "rust") {
        return Err(installed_failure(InstalledFrontendRunCode::Selection));
    }
    let launcher = native_launcher_plan(&resolved, &selection, request.contracts)?;
    let expected = complete_inventory_set(&registry);
    let snapshots = installed
        .snapshot_selected_bundles(
            &expected,
            &resolved.frontend.bundle_id,
            &resolved.toolchain.bundle_id,
        )
        .map_err(installed_release_error)?;
    let frontend = snapshots
        .get(&resolved.frontend.bundle_id)
        .map(Arc::as_ref)
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Release))?;
    let toolchain = snapshots
        .get(&resolved.toolchain.bundle_id)
        .map(Arc::as_ref)
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Release))?;
    let sandbox = prepare_release_sandbox(&resolved.toolchain.execution_host_profile_id)
        .map_err(|_| installed_failure(InstalledFrontendRunCode::Launch))?;
    let output = launch_release_frontend(
        sandbox,
        frontend,
        toolchain,
        &resolved.frontend.main.path,
        &launcher.argv,
        &launcher.environment,
        request.captured_inputs,
        request.staged_directories,
        request.staged_placeholders,
    )
    .map_err(installed_sandbox_error)?;
    if output.stream_limit_exceeded {
        return Err(installed_failure(InstalledFrontendRunCode::Protocol(
            SuccessorFrontendProtocolCode::ProtocolLimit,
        )));
    }
    let synthetic_permissions = installed_synthetic_permissions(
        resolved.semantic_context.source_language(),
        &output.stdout,
    )?;
    let release_registry = registry.release_identity();
    let envelope = validate_successor_frontend_process(
        SuccessorFrontendProtocolRequest {
            registry: &semantic_registry,
            semantic_context: &resolved.semantic_context,
            selection: &selection,
            release_registry: &release_registry,
            captured_inputs: request.captured_inputs,
            synthetic_permissions: &synthetic_permissions,
        },
        FrontendProcessFacts {
            exit_code: output.exit_code,
            signaled: output.signaled,
            stdout: &output.stdout,
            stderr_observed_bytes: output.stderr_observed_bytes,
        },
    )
    .map_err(|error| installed_failure(InstalledFrontendRunCode::Protocol(error.code())))?;
    validate_native_emitted_identity(&envelope, &resolved, &selection)?;
    Ok(AcceptedInstalledFrontendRun {
        envelope,
        launcher,
        release_registry,
    })
}

/// The installed Go producer is content-addressed and closed by the active
/// release tuple, so its three frozen synthetic-origin reasons are compiled
/// into this runner. Generic protocol imports still require exact explicit
/// permissions; only this installed-bundle path derives their exact
/// references from the candidate output before strict protocol validation.
fn installed_synthetic_permissions(
    language: &str,
    stdout: &[u8],
) -> Result<Vec<SyntheticPermission>, InstalledFrontendRunError> {
    if language != "go" {
        return Ok(Vec::new());
    }
    let envelope: Value = serde_json::from_slice(stdout).map_err(|_| {
        installed_failure(InstalledFrontendRunCode::Protocol(
            SuccessorFrontendProtocolCode::ProtocolMalformed,
        ))
    })?;
    let Some(entries) = envelope
        .get("source_map")
        .and_then(|source_map| source_map.get("entries"))
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };
    entries
        .iter()
        .filter(|entry| entry["origin"]["kind"] == "synthetic")
        .map(|entry| {
            let reason = entry["origin"]["reason"].as_str().ok_or_else(|| {
                installed_failure(InstalledFrontendRunCode::Protocol(
                    SuccessorFrontendProtocolCode::ProtocolShape,
                ))
            })?;
            if !matches!(
                reason,
                "go.control_flow_join" | "go.loop_backedge" | "go.implicit_return"
            ) {
                return Err(installed_failure(InstalledFrontendRunCode::Protocol(
                    SuccessorFrontendProtocolCode::ProtocolArtifactMismatch,
                )));
            }
            let reference = serde_json::from_value::<SourceReference>(entry["reference"].clone())
                .map_err(|_| {
                installed_failure(InstalledFrontendRunCode::Protocol(
                    SuccessorFrontendProtocolCode::ProtocolShape,
                ))
            })?;
            Ok(SyntheticPermission {
                reference,
                reason: reason.to_owned(),
            })
        })
        .collect()
}

fn native_launcher_plan(
    resolved: &ResolvedSuccessorRelease<'_>,
    selection: &SelectionEnvelope,
    contracts: &[String],
) -> Result<FrontendLauncherPlan, InstalledFrontendRunError> {
    let contracts = normalized_contract_arguments(contracts)?;
    let context = &resolved.semantic_context;
    let parameters = context
        .semantic_parameters()
        .value()
        .as_object()
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Selection))?;
    let target = parameters
        .get("target_id")
        .and_then(Value::as_str)
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Selection))?;
    let value = selection
        .value()
        .as_object()
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Selection))?;
    let mut argv = match context.source_language() {
        "go" if resolved.frontend.bundle_id == GO_FRONTEND_BUNDLE_ID
            && resolved.toolchain.bundle_id == GO_TOOLCHAIN_BUNDLE_ID =>
        {
            vec![
                "lower".to_owned(),
                SOURCE_ROOT.to_owned(),
                "--package".to_owned(),
                required_string(value, "package")?.to_owned(),
                "--semantic-profile".to_owned(),
                context.semantic_profile().to_owned(),
                "--target".to_owned(),
                target.to_owned(),
                "--function".to_owned(),
                required_string(value, "function")?.to_owned(),
            ]
        }
        "rust"
            if resolved.frontend.bundle_id == RUST_FRONTEND_BUNDLE_ID
                && resolved.toolchain.bundle_id == RUST_TOOLCHAIN_BUNDLE_ID =>
        {
            vec![
                "lower".to_owned(),
                SOURCE_ROOT.to_owned(),
                "--manifest-path".to_owned(),
                "Cargo.toml".to_owned(),
                "--package".to_owned(),
                required_string(value, "package")?.to_owned(),
                "--semantic-profile".to_owned(),
                context.semantic_profile().to_owned(),
                "--target".to_owned(),
                target.to_owned(),
                "--function".to_owned(),
                required_string(value, "function")?.to_owned(),
            ]
        }
        _ => return Err(installed_failure(InstalledFrontendRunCode::Selection)),
    };
    let identity = context.profile_registry();
    append_option(&mut argv, "--profile-registry-id", identity.id());
    append_option(
        &mut argv,
        "--profile-registry-revision",
        &identity.revision().to_string(),
    );
    append_option(
        &mut argv,
        "--profile-registry-sha256",
        identity.registry_sha256(),
    );
    append_option(
        &mut argv,
        "--profile-entry-sha256",
        context.profile_entry_sha256(),
    );
    append_option(
        &mut argv,
        "--frontend-bundle-id",
        &resolved.frontend.bundle_id,
    );
    append_option(
        &mut argv,
        "--frontend-sha256",
        &resolved.frontend.main.binary_sha256,
    );
    append_option(
        &mut argv,
        "--release-registry-id",
        SUCCESSOR_RELEASE_REGISTRY_ID,
    );
    append_option(
        &mut argv,
        "--release-registry-sha256",
        EXPECTED_REGISTRY_SHA256_HEX,
    );
    append_option(
        &mut argv,
        "--toolchain-bundle-id",
        &resolved.toolchain.bundle_id,
    );
    append_option(&mut argv, "--toolchain-root", "/mpk/toolchain");
    append_option(
        &mut argv,
        "--toolchain-distribution-sha256",
        &resolved.toolchain.distribution_sha256,
    );
    if context.source_language() == "rust" {
        let driver = resolved
            .frontend
            .subordinate_binaries
            .iter()
            .find(|binary| binary.name == "rust2vir-driver")
            .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Identity))?;
        append_option(&mut argv, "--driver", "/mpk/frontend/bin/rust2vir-driver");
        append_option(&mut argv, "--driver-sha256", &driver.binary_sha256);
    }
    for contract in contracts {
        append_option(&mut argv, "--contract", &contract);
    }
    let argument_bytes = argv.iter().try_fold(0_usize, |total, argument| {
        total.checked_add(argument.len().checked_add(1)?)
    });
    if argument_bytes.is_none_or(|bytes| bytes > FRONTEND_ARGUMENT_BYTES_MAX)
        || argv.iter().any(|argument| argument.as_bytes().contains(&0))
    {
        return Err(installed_failure(InstalledFrontendRunCode::Launch));
    }
    Ok(FrontendLauncherPlan {
        program: resolved.frontend.main.path.clone(),
        argv,
        environment: native_environment(context.source_language(), target)?,
    })
}

fn required_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, InstalledFrontendRunError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Selection))
}

fn normalized_contract_arguments(
    contracts: &[String],
) -> Result<Vec<String>, InstalledFrontendRunError> {
    if contracts.len() > FRONTEND_CONTRACTS_MAX {
        return Err(installed_failure(InstalledFrontendRunCode::Selection));
    }
    let mut folded = BTreeSet::new();
    for contract in contracts {
        if mpk_vc::validate_manifest_normalized_path(contract).is_err()
            || !folded.insert(contract.to_ascii_lowercase())
        {
            return Err(installed_failure(InstalledFrontendRunCode::Selection));
        }
    }
    let mut normalized = contracts.to_vec();
    normalized.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(normalized)
}

fn native_environment(
    language: &str,
    target: &str,
) -> Result<BTreeMap<String, String>, InstalledFrontendRunError> {
    let values: &[(&str, &str)] = match (language, target) {
        ("rust", "i686-unknown-linux-gnu" | "x86_64-unknown-linux-gnu") => &[
            ("HOME", "/mpk/empty/home"),
            ("LANG", "C"),
            ("LC_ALL", "C"),
            ("PATH", "/mpk/toolchain/bin"),
            ("TMPDIR", "/mpk/tmp"),
            ("TZ", "UTC"),
        ],
        ("go", "linux/amd64") => &[
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
        ],
        _ => return Err(installed_failure(InstalledFrontendRunCode::Selection)),
    };
    Ok(values
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect())
}

fn validate_native_emitted_identity(
    envelope: &AcceptedSuccessorFrontendEnvelope,
    resolved: &ResolvedSuccessorRelease<'_>,
    selection: &SelectionEnvelope,
) -> Result<(), InstalledFrontendRunError> {
    let manifest = envelope
        .artifacts()
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Identity))?
        .source_manifest()
        .manifest();
    let expected_frontend = FrontendIdentity {
        bundle_id: resolved.frontend.bundle_id.clone(),
        name: resolved.frontend.name.clone(),
        version: resolved.frontend.version.clone(),
        binary_sha256: resolved.frontend.main.binary_sha256.clone(),
        subordinate_binaries: resolved
            .frontend
            .subordinate_binaries
            .iter()
            .map(|binary| SubordinateIdentity {
                name: binary.name.clone(),
                version: binary.version.clone(),
                binary_sha256: binary.binary_sha256.clone(),
            })
            .collect(),
    };
    let rust = resolved.semantic_context.source_language() == "rust";
    let expected_toolchain = ToolchainIdentity {
        bundle_id: resolved.toolchain.bundle_id.clone(),
        distribution_sha256: resolved.toolchain.distribution_sha256.clone(),
        components: resolved
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
                    commit_hash: (rust && name == "rustc")
                        .then(|| "4d08223c054cf5a56d9761ca925fd46ffebe7115".to_owned()),
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
            .collect(),
    };
    let parameters = resolved
        .semantic_context
        .semantic_parameters()
        .value()
        .as_object()
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Identity))?;
    let target = required_string(parameters, "target_id")?;
    let pointer_width = parameters
        .get("pointer_width")
        .and_then(Value::as_u64)
        .ok_or_else(|| installed_failure(InstalledFrontendRunCode::Identity))?;
    if manifest.semantic_context() != &resolved.semantic_context
        || manifest.selection() != selection
        || manifest.release_registry().schema != SUCCESSOR_RELEASE_REGISTRY_SCHEMA
        || manifest.release_registry().id != SUCCESSOR_RELEASE_REGISTRY_ID
        || manifest.release_registry().registry_sha256 != EXPECTED_REGISTRY_SHA256_HEX
        || manifest.frontend() != &expected_frontend
        || manifest.toolchain() != &expected_toolchain
        || manifest.limit_profile() != resolved.release_tuple.limit_profile_id
        || manifest.target().id() != target
        || u64::from(manifest.target().pointer_width().bits()) != pointer_width
    {
        return Err(installed_failure(InstalledFrontendRunCode::Identity));
    }
    Ok(())
}

fn complete_inventory_set(
    registry: &ValidatedSuccessorReleaseRegistry,
) -> BTreeMap<String, &BundleInventory> {
    registry
        .registry()
        .frontend_bundles
        .iter()
        .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory))
        .chain(
            registry
                .registry()
                .toolchain_bundles
                .iter()
                .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory)),
        )
        .collect()
}

fn map_csharp_error(code: CSharpRunCode) -> InstalledFrontendRunCode {
    match code {
        CSharpRunCode::Release | CSharpRunCode::Sandbox => InstalledFrontendRunCode::Release,
        CSharpRunCode::Selection | CSharpRunCode::Contract => InstalledFrontendRunCode::Selection,
        CSharpRunCode::Launch => InstalledFrontendRunCode::Launch,
        CSharpRunCode::Process => InstalledFrontendRunCode::Process,
        CSharpRunCode::Protocol(code) => InstalledFrontendRunCode::Protocol(code),
        CSharpRunCode::Identity => InstalledFrontendRunCode::Identity,
    }
}

fn map_java_error(error: JavaRunError) -> InstalledFrontendRunError {
    installed_failure(match error {
        JavaRunError::Release | JavaRunError::Sandbox => InstalledFrontendRunCode::Release,
        JavaRunError::Selection => InstalledFrontendRunCode::Selection,
        JavaRunError::Process => InstalledFrontendRunCode::Process,
        JavaRunError::Protocol => {
            InstalledFrontendRunCode::Protocol(SuccessorFrontendProtocolCode::ProtocolMalformed)
        }
        JavaRunError::Identity => InstalledFrontendRunCode::Identity,
    })
}

fn installed_release_error(
    _error: crate::frontend_registry::FrontendReleaseError,
) -> InstalledFrontendRunError {
    installed_failure(InstalledFrontendRunCode::Release)
}

fn installed_sandbox_error(error: SandboxError) -> InstalledFrontendRunError {
    installed_failure(match error {
        SandboxError::Unavailable => InstalledFrontendRunCode::Launch,
        SandboxError::Spawn | SandboxError::Killed => InstalledFrontendRunCode::Process,
    })
}

const fn installed_failure(code: InstalledFrontendRunCode) -> InstalledFrontendRunError {
    InstalledFrontendRunError { code }
}

fn validate_csharp_registry_layout(
    registry: &ValidatedSuccessorReleaseRegistry,
) -> Result<(), CSharpRunError> {
    let value = registry.registry();
    let expected_host = json!({
        "abi": "gnu",
        "architecture": "x86_64",
        "id": CSHARP_HOST_PROFILE_ID,
        "minimum_kernel_abi": "6.4.0",
        "os": "linux",
        "probe_profile_id": "mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0",
        "required_primitives": [
            "filesystem.atomic_no_replace",
            "filesystem.immutable_handle",
            "filesystem.no_follow_open",
            "filesystem.tmpfs_allocated_blocks",
            "filesystem.tmpfs_inode_limit",
            "isolation.cgroup_v2",
            "isolation.mount_namespace",
            "isolation.network_namespace",
            "isolation.user_namespace",
            "memory.cgroup_accounting",
            "mount.no_exec",
            "mount.read_only",
            "mount.tmpfs_noswap",
            "process.cgroup_tasks",
            "process.closed_environment",
            "process.no_new_privileges",
            "process.rlimit_address_space",
            "process.rlimit_open_files",
            "process.task_tree_kill"
        ]
    });
    let expected_layout = json!({
        "execution_host_profile_id": CSHARP_HOST_PROFILE_ID,
        "forbidden_host_roots": ["/lib", "/lib64", "/usr/lib"],
        "id": CSHARP_RUNTIME_LAYOUT_ID,
        "interpreter_mounts": [{
            "component_path": "lib64/ld-linux-x86-64.so.2",
            "sandbox_path": "/lib64/ld-linux-x86-64.so.2"
        }],
        "library_mounts": [{
            "component_path": "lib/x86_64-linux-gnu",
            "sandbox_path": "/lib/x86_64-linux-gnu"
        }],
        "loader_search_paths": ["/lib/x86_64-linux-gnu"],
        "runtime_root": "/mpk/native-runtime"
    });
    let host = value
        .execution_host_profiles
        .iter()
        .find(|profile| profile.id == CSHARP_HOST_PROFILE_ID);
    let layout = value
        .native_runtime_layout_profiles
        .iter()
        .find(|profile| profile.id == CSHARP_RUNTIME_LAYOUT_ID);
    if value.execution_host_profiles.len() != 2
        || value.native_runtime_layout_profiles.len() != 1
        || host.and_then(|profile| serde_json::to_value(profile).ok()) != Some(expected_host)
        || layout.and_then(|profile| serde_json::to_value(profile).ok()) != Some(expected_layout)
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    Ok(())
}

fn validate_csharp_release(
    semantic_registry: &ValidatedSemanticProfileRegistry,
    resolved: &ResolvedSuccessorRelease<'_>,
) -> Result<(), CSharpRunError> {
    let identity = semantic_registry.identity();
    if identity.revision() != 3
        || identity.registry_sha256()
            != "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557"
        || resolved.semantic_context.source_language() != "csharp"
        || resolved.semantic_context.semantic_profile() != "mpk.csharp.scalar.v0"
        || resolved.semantic_context.profile_entry_sha256() != CSHARP_PROFILE_ENTRY_SHA256
        || resolved.release_tuple.limit_profile_id != "mpk.vir.limits.v0"
        || resolved.frontend.bundle_id != CSHARP_FRONTEND_BUNDLE_ID
        || resolved.toolchain.bundle_id != CSHARP_TOOLCHAIN_BUNDLE_ID
        || resolved.toolchain.execution_host_profile_id != CSHARP_HOST_PROFILE_ID
    {
        return Err(failure(CSharpRunCode::Contract));
    }
    let frontend_contract = json!({
        "contract_id": "mpk.profile.frontend.csharp_scalar.v0",
        "profile_entry_sha256": CSHARP_PROFILE_ENTRY_SHA256,
        "value": {
            "argument_profile_id": "mpk.csharp.frontend_arguments.v0",
            "environment_profile_id": "mpk.csharp.frontend_environment.v0",
            "launcher_profile_id": "mpk.csharp.dotnet_launcher.v0",
            "limit_profile_id": "mpk.csharp.limits.v0",
            "private_driver": "none"
        }
    });
    let release_contract = json!({
        "contract_id": "mpk.profile.release.csharp_scalar.v0",
        "profile_entry_sha256": CSHARP_PROFILE_ENTRY_SHA256,
        "value": {
            "compiler_profile_id": "mpk.csharp.roslyn_5_6_0.v0",
            "execution_host_profile_id": CSHARP_HOST_PROFILE_ID,
            "reference_profile_id": "mpk.dotnet.netcore_ref_10_0_11.v0",
            "runtime_layout_profile_id": CSHARP_RUNTIME_LAYOUT_ID,
            "runtime_profile_id": "mpk.dotnet.runtime_10_0_11.linux_x64.v0",
            "toolchain_inputs_sha256": "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f"
        }
    });
    if resolved.frontend.profile_contracts != [frontend_contract]
        || resolved.toolchain.profile_contracts != [release_contract]
    {
        return Err(failure(CSharpRunCode::Contract));
    }

    let host = resolved.toolchain.execution_host_profile_id.as_str();
    if host != CSHARP_HOST_PROFILE_ID
        || resolved.frontend.name != "csharp2vir"
        || resolved.frontend.version != "0.1.0"
        || resolved.frontend.main.path != "csharp2vir.dll"
        || resolved.frontend.main.binary_sha256 != CSHARP_FRONTEND_SHA256
        || !matches!(resolved.frontend.main.runtime, ExecutableRuntime::Static)
        || resolved.frontend.inventory.files.len() != 18
        || resolved.toolchain.inventory.files.len() != 373
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let subordinate = resolved
        .frontend
        .subordinate_binaries
        .iter()
        .map(|binary| {
            (
                binary.name.as_str(),
                binary.version.as_str(),
                binary.path.as_str(),
                binary.binary_sha256.as_str(),
                matches!(binary.runtime, ExecutableRuntime::Static),
            )
        })
        .collect::<Vec<_>>();
    if subordinate
        != [
            (
                "Microsoft.CodeAnalysis.CSharp.dll",
                "5.6.0",
                "Microsoft.CodeAnalysis.CSharp.dll",
                ROSLYN_CSHARP_SHA256,
                true,
            ),
            (
                "Microsoft.CodeAnalysis.dll",
                "5.6.0",
                "Microsoft.CodeAnalysis.dll",
                ROSLYN_COMMON_SHA256,
                true,
            ),
        ]
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let managed_frontend_files = resolved
        .frontend
        .inventory
        .files
        .iter()
        .filter_map(|file| file.path.ends_with(".dll").then_some(file.path.as_str()))
        .collect::<BTreeSet<_>>();
    if managed_frontend_files
        != BTreeSet::from([
            "Microsoft.CodeAnalysis.CSharp.dll",
            "Microsoft.CodeAnalysis.dll",
            "csharp2vir.dll",
        ])
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    validate_toolchain_layout(resolved)?;
    Ok(())
}

fn validate_toolchain_layout(
    resolved: &ResolvedSuccessorRelease<'_>,
) -> Result<(), CSharpRunError> {
    let names = resolved
        .toolchain
        .components
        .iter()
        .map(ToolchainComponent::name)
        .collect::<Vec<_>>();
    if names
        != [
            "dotnet",
            "dotnet-runtime",
            "native-runtime",
            "reference-pack",
        ]
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let dotnet = &resolved.toolchain.components[0];
    let ToolchainComponent::Executable {
        release,
        path,
        runtime,
        ..
    } = dotnet
    else {
        return Err(failure(CSharpRunCode::Identity));
    };
    let ExecutableRuntime::Dynamic {
        interpreter_mount,
        libraries,
    } = runtime
    else {
        return Err(failure(CSharpRunCode::Identity));
    };
    let direct_libraries = libraries
        .iter()
        .map(|library| library.soname.as_str())
        .collect::<Vec<_>>();
    if release != "10.0.11"
        || path != "dotnet/dotnet"
        || interpreter_mount != "/lib64/ld-linux-x86-64.so.2"
        || direct_libraries
            != [
                "ld-linux-x86-64.so.2",
                "libc.so.6",
                "libdl.so.2",
                "libgcc_s.so.1",
                "libm.so.6",
                "libpthread.so.0",
                "librt.so.1",
                "libstdc++.so.6",
            ]
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let content_counts = resolved
        .toolchain
        .components
        .iter()
        .filter_map(|component| match component {
            ToolchainComponent::Content {
                name, inventory, ..
            } => Some((name.as_str(), inventory.files.len())),
            ToolchainComponent::Executable { .. } => None,
        })
        .collect::<Vec<_>>();
    if content_counts
        != [
            ("dotnet-runtime", 192),
            ("native-runtime", 13),
            ("reference-pack", 167),
        ]
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let paths = resolved
        .toolchain
        .inventory
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    for required in [
        "dotnet/dotnet",
        "native-runtime/lib64/ld-linux-x86-64.so.2",
        "native-runtime/lib/x86_64-linux-gnu/libc.so",
        "native-runtime/lib/x86_64-linux-gnu/libc.so.6",
        "native-runtime/lib/x86_64-linux-gnu/libcrypto.so.1.1",
        "native-runtime/lib/x86_64-linux-gnu/libssl.so.1.1",
        "native-runtime/lib/x86_64-linux-gnu/libz.so.1",
        "reference-pack/ref/net10.0/System.Runtime.dll",
    ] {
        if !paths.contains(required) {
            return Err(failure(CSharpRunCode::Identity));
        }
    }
    if paths.iter().any(|path| {
        let lower = path.to_ascii_lowercase();
        [
            "analyzer",
            "csc.dll",
            "generator",
            "msbuild.dll",
            "nuget",
            "vbc.dll",
            "vstest",
        ]
        .iter()
        .any(|forbidden| lower.contains(forbidden))
    }) {
        return Err(failure(CSharpRunCode::Identity));
    }
    Ok(())
}

fn validate_captured_selection(
    selection: &SelectionEnvelope,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), CSharpRunError> {
    let value = selection
        .value()
        .as_object()
        .ok_or_else(|| failure(CSharpRunCode::Selection))?;
    let sources = string_array(value.get("sources"))?;
    let contracts = string_array(value.get("contracts"))?;
    let mut expected = BTreeMap::new();
    for path in sources {
        if expected.insert(path, InputKind::Source).is_some() {
            return Err(failure(CSharpRunCode::Selection));
        }
    }
    for path in contracts {
        if expected.insert(path, InputKind::Contract).is_some() {
            return Err(failure(CSharpRunCode::Selection));
        }
    }
    if expected.len() != captured_inputs.len() {
        return Err(failure(CSharpRunCode::Selection));
    }
    let mut observed = BTreeSet::new();
    for input in captured_inputs {
        if input.bytes.is_empty()
            || expected.get(input.normalized_path) != Some(&input.kind)
            || !observed.insert(input.normalized_path)
        {
            return Err(failure(CSharpRunCode::Selection));
        }
    }
    Ok(())
}

fn csharp_launcher_plan(
    resolved: &ResolvedSuccessorRelease<'_>,
    selection: &SelectionEnvelope,
) -> Result<CSharpLauncherPlan, CSharpRunError> {
    let value = selection
        .value()
        .as_object()
        .ok_or_else(|| failure(CSharpRunCode::Selection))?;
    let compilation = value
        .get("compilation")
        .and_then(Value::as_str)
        .ok_or_else(|| failure(CSharpRunCode::Selection))?;
    let sources = string_array(value.get("sources"))?;
    let contracts = string_array(value.get("contracts"))?;
    let methods = string_array(value.get("methods"))?;
    let identity = resolved.semantic_context.profile_registry();
    let mut argv = [
        DOTNET_PROGRAM,
        "exec",
        "--depsfile",
        "/mpk/frontend/csharp2vir.deps.json",
        "--runtimeconfig",
        "/mpk/frontend/csharp2vir.runtimeconfig.json",
        "--fx-version",
        "10.0.11",
        "/mpk/frontend/csharp2vir.dll",
        "lower",
        SOURCE_ROOT,
        "--semantic-profile",
        "mpk.csharp.scalar.v0",
        "--target",
        "linux-x64",
        "--compilation",
        compilation,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    append_pairs(&mut argv, "--source", &sources);
    append_pairs(&mut argv, "--contract", &contracts);
    append_pairs(&mut argv, "--method", &methods);
    append_option(&mut argv, "--profile-registry-id", identity.id());
    append_option(
        &mut argv,
        "--profile-registry-revision",
        &identity.revision().to_string(),
    );
    append_option(
        &mut argv,
        "--profile-registry-sha256",
        identity.registry_sha256(),
    );
    append_option(
        &mut argv,
        "--profile-entry-sha256",
        resolved.semantic_context.profile_entry_sha256(),
    );
    append_option(
        &mut argv,
        "--frontend-bundle-id",
        &resolved.frontend.bundle_id,
    );
    append_option(
        &mut argv,
        "--frontend-sha256",
        &resolved.frontend.main.binary_sha256,
    );
    append_option(
        &mut argv,
        "--release-registry-id",
        SUCCESSOR_RELEASE_REGISTRY_ID,
    );
    append_option(
        &mut argv,
        "--release-registry-sha256",
        EXPECTED_REGISTRY_SHA256_HEX,
    );
    append_option(
        &mut argv,
        "--toolchain-bundle-id",
        &resolved.toolchain.bundle_id,
    );
    append_option(&mut argv, "--toolchain-root", "/mpk/toolchain");
    append_option(
        &mut argv,
        "--toolchain-distribution-sha256",
        &resolved.toolchain.distribution_sha256,
    );
    let argument_bytes = argv.iter().try_fold(0_usize, |total, argument| {
        total.checked_add(argument.len().checked_add(1)?)
    });
    if argument_bytes.is_none_or(|bytes| bytes > CSHARP_ARGUMENT_BYTES_MAX)
        || argv.iter().any(|argument| argument.as_bytes().contains(&0))
    {
        return Err(failure(CSharpRunCode::Launch));
    }
    let environment = [
        ("COMPlus_ReadyToRun", "0"),
        ("DOTNET_CLI_TELEMETRY_OPTOUT", "1"),
        ("DOTNET_MULTILEVEL_LOOKUP", "0"),
        ("DOTNET_NOLOGO", "1"),
        ("DOTNET_ROOT", "/mpk/toolchain/dotnet"),
        ("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1"),
        ("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT", "1"),
        ("DOTNET_TieredCompilation", "0"),
        ("DOTNET_TieredPGO", "0"),
        ("HOME", "/mpk/empty-home"),
        ("LANG", "C.UTF-8"),
        ("LC_ALL", "C.UTF-8"),
        ("NUGET_HTTP_CACHE_PATH", "/mpk/empty-nuget-http"),
        ("NUGET_PACKAGES", "/mpk/empty-nuget"),
        ("NUGET_PLUGINS_CACHE_PATH", "/mpk/empty-nuget-plugins"),
        ("PATH", "/nonexistent"),
        ("TMPDIR", "/mpk/tmp"),
        ("TZ", "UTC"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    Ok(CSharpLauncherPlan { argv, environment })
}

fn string_array(value: Option<&Value>) -> Result<Vec<&str>, CSharpRunError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| failure(CSharpRunCode::Selection))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| failure(CSharpRunCode::Selection))
        })
        .collect()
}

fn append_pairs(arguments: &mut Vec<String>, option: &str, values: &[&str]) {
    for value in values {
        append_option(arguments, option, value);
    }
}

fn append_option(arguments: &mut Vec<String>, option: &str, value: &str) {
    arguments.push(option.to_owned());
    arguments.push(value.to_owned());
}

fn validate_emitted_release_identity(
    envelope: &AcceptedSuccessorFrontendEnvelope,
    resolved: &ResolvedSuccessorRelease<'_>,
) -> Result<(), CSharpRunError> {
    let manifest = envelope
        .artifacts()
        .ok_or_else(|| failure(CSharpRunCode::Identity))?
        .source_manifest()
        .manifest();
    let frontend = manifest.frontend();
    let expected_subordinates = vec![
        SubordinateIdentity {
            name: "Microsoft.CodeAnalysis.CSharp.dll".to_owned(),
            version: "5.6.0".to_owned(),
            binary_sha256: ROSLYN_CSHARP_SHA256.to_owned(),
        },
        SubordinateIdentity {
            name: "Microsoft.CodeAnalysis.dll".to_owned(),
            version: "5.6.0".to_owned(),
            binary_sha256: ROSLYN_COMMON_SHA256.to_owned(),
        },
    ];
    if frontend.bundle_id != resolved.frontend.bundle_id
        || frontend.name != resolved.frontend.name
        || frontend.version != resolved.frontend.version
        || frontend.binary_sha256 != resolved.frontend.main.binary_sha256
        || frontend.subordinate_binaries != expected_subordinates
        || manifest.limit_profile() != resolved.release_tuple.limit_profile_id
        || manifest.target().id() != "linux-x64"
        || manifest.target().pointer_width().bits() != 64
        || manifest.release_registry().schema != SUCCESSOR_RELEASE_REGISTRY_SCHEMA
        || manifest.release_registry().id != SUCCESSOR_RELEASE_REGISTRY_ID
        || manifest.release_registry().registry_sha256 != EXPECTED_REGISTRY_SHA256_HEX
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    let expected_components = vec![
        ComponentIdentity::Content {
            name: "dotnet-runtime".to_owned(),
            release: "10.0.11".to_owned(),
            content_sha256: DOTNET_RUNTIME_ARCHIVE_SHA256.to_owned(),
        },
        ComponentIdentity::Content {
            name: "microsoft-codeanalysis-common".to_owned(),
            release: "5.6.0".to_owned(),
            content_sha256: ROSLYN_COMMON_SHA256.to_owned(),
        },
        ComponentIdentity::Content {
            name: "microsoft-codeanalysis-csharp".to_owned(),
            release: "5.6.0".to_owned(),
            content_sha256: ROSLYN_CSHARP_SHA256.to_owned(),
        },
        ComponentIdentity::Content {
            name: "reference-pack".to_owned(),
            release: "10.0.11".to_owned(),
            content_sha256: REFERENCE_INVENTORY_SHA256.to_owned(),
        },
    ];
    let toolchain = manifest.toolchain();
    if toolchain.bundle_id != resolved.toolchain.bundle_id
        || toolchain.distribution_sha256 != resolved.toolchain.distribution_sha256
        || toolchain.components != expected_components
    {
        return Err(failure(CSharpRunCode::Identity));
    }
    Ok(())
}

fn selected_snapshot<'a>(
    snapshots: &'a BTreeMap<String, Arc<BundleSnapshot>>,
    bundle_id: &str,
) -> Result<&'a BundleSnapshot, CSharpRunError> {
    snapshots
        .get(bundle_id)
        .map(Arc::as_ref)
        .ok_or_else(|| failure(CSharpRunCode::Release))
}

fn release_error(error: crate::frontend_registry::FrontendReleaseError) -> CSharpRunError {
    match error.code() {
        FrontendReleaseCode::RegistryMissing
        | FrontendReleaseCode::RegistryLimit
        | FrontendReleaseCode::RegistryInvalid
        | FrontendReleaseCode::RegistryMismatch
        | FrontendReleaseCode::BundleInvalid
        | FrontendReleaseCode::BundleUnknown
        | FrontendReleaseCode::BundleIncompatible
        | FrontendReleaseCode::RegistryAssertion
        | FrontendReleaseCode::SandboxUnavailable => (),
    }
    failure(CSharpRunCode::Release)
}

fn sandbox_error(error: SandboxError) -> CSharpRunError {
    match error {
        SandboxError::Unavailable => failure(CSharpRunCode::Launch),
        SandboxError::Spawn | SandboxError::Killed => failure(CSharpRunCode::Process),
    }
}

const fn failure(code: CSharpRunCode) -> CSharpRunError {
    CSharpRunError { code }
}
