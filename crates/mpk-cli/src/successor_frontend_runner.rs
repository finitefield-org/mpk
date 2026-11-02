//! Descriptor-relative Go/Rust runner for the active successor release.
//!
//! The entry point opens only the release root beside the already executing
//! `bin/mpk`, validates both build-pinned registries, snapshots the selected
//! bundle bytes, and launches that snapshot through the hardened Linux
//! sandbox.

use crate::frontend_protocol::FrontendProcessFacts;
use crate::frontend_registry::{
    InstalledSuccessorRelease, EXPECTED_REGISTRY_SHA256_HEX, EXPECTED_SEMANTIC_REGISTRY_SHA256,
};
use crate::frontend_sandbox::{launch_release_frontend, prepare_release_sandbox, SandboxError};
use crate::successor_frontend_protocol::{
    validate_successor_frontend_process, AcceptedSuccessorFrontendEnvelope,
    SuccessorFrontendProtocolCode, SuccessorFrontendProtocolRequest,
};
use crate::successor_release_bundle::{
    validate_successor_release_registry, ResolvedSuccessorRelease,
    SuccessorReleaseSelectionRequest, ValidatedSuccessorReleaseRegistry, GO_FRONTEND_BUNDLE_ID,
    GO_TOOLCHAIN_BUNDLE_ID, RUST_FRONTEND_BUNDLE_ID, RUST_TOOLCHAIN_BUNDLE_ID,
    SUCCESSOR_RELEASE_REGISTRY_ID, SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_semantic_profile_registry, RegistryRevision,
    SelectionEnvelope,
};
use mpk_vc::{
    BundleInventory, CapturedInput, ComponentIdentity, FrontendIdentity, ReleaseRegistryIdentity,
    SourceReference, SubordinateIdentity, SyntheticPermission, ToolchainComponent,
    ToolchainIdentity,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

const FRONTEND_ARGUMENT_BYTES_MAX: usize = 262_144;
const FRONTEND_CONTRACTS_MAX: usize = 128;
const SOURCE_ROOT: &str = "/mpk/source";

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
/// image. The caller supplies assertions, never paths; Go and Rust
/// pass through the same registry/context/selection/protocol chain.
pub fn run_installed_frontend(
    request: InstalledFrontendRunRequest<'_>,
) -> Result<AcceptedInstalledFrontendRun, InstalledFrontendRunError> {
    if !request.synthetic_permissions.is_empty() {
        return Err(installed_failure(InstalledFrontendRunCode::Selection));
    }

    run_installed_native_frontend(request)
}

fn run_installed_native_frontend(
    request: InstalledFrontendRunRequest<'_>,
) -> Result<AcceptedInstalledFrontendRun, InstalledFrontendRunError> {
    let installed = InstalledSuccessorRelease::open().map_err(installed_release_error)?;
    let semantic_registry = validate_semantic_profile_registry(
        &installed.semantic_registry_bytes,
        RegistryRevision::Revision4,
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

fn append_option(arguments: &mut Vec<String>, option: &str, value: &str) {
    arguments.push(option.to_owned());
    arguments.push(value.to_owned());
}
