//! Inactive descriptor-relative C# runner for the successor staging fixture.
//!
//! No production command calls this module. The only entry point opens the
//! release root beside the already executing `bin/mpk`, validates both staged
//! registries, snapshots the selected bundle bytes, and launches that snapshot
//! through the existing hardened Linux sandbox.

use crate::frontend_protocol::FrontendProcessFacts;
use crate::frontend_registry::{BundleSnapshot, FrontendReleaseCode, StagedInstalledRelease};
use crate::frontend_sandbox::{
    launch_staged_csharp_frontend, prepare_release_sandbox, SandboxError,
};
use crate::successor_frontend_protocol::{
    validate_successor_frontend_process, AcceptedSuccessorFrontendEnvelope,
    SuccessorFrontendProtocolCode, SuccessorFrontendProtocolRequest,
};
use crate::successor_release_bundle::{
    validate_successor_release_registry, ResolvedSuccessorRelease,
    SuccessorReleaseSelectionRequest, ValidatedSuccessorReleaseRegistry, CSHARP_FRONTEND_BUNDLE_ID,
    CSHARP_FRONTEND_SHA256, CSHARP_HOST_PROFILE_ID, CSHARP_RUNTIME_LAYOUT_ID,
    CSHARP_STAGING_REGISTRY_SHA256, CSHARP_TOOLCHAIN_BUNDLE_ID, SUCCESSOR_RELEASE_REGISTRY_ID,
    SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    validate_inactive_semantic_profile_registry, validate_registry_selection_envelope,
    InactiveRegistryRevision, SelectionEnvelope, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{
    CapturedInput, ComponentIdentity, ExecutableRuntime, InputKind, ReleaseRegistryIdentity,
    SubordinateIdentity, SyntheticPermission, ToolchainComponent,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

const CSHARP_PROFILE_ENTRY_SHA256: &str =
    "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac";
const CSHARP_ARGUMENT_BYTES_MAX: usize = 131_072;
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
pub enum StagedCSharpRunCode {
    Release,
    Selection,
    Contract,
    Sandbox,
    Launch,
    Process,
    Protocol(SuccessorFrontendProtocolCode),
    Identity,
}

impl StagedCSharpRunCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "CSHARP_STAGED_RELEASE",
            Self::Selection => "CSHARP_STAGED_SELECTION",
            Self::Contract => "CSHARP_STAGED_CONTRACT",
            Self::Sandbox => "CSHARP_STAGED_SANDBOX",
            Self::Launch => "CSHARP_STAGED_LAUNCH",
            Self::Process => "CSHARP_STAGED_PROCESS",
            Self::Protocol(code) => code.as_str(),
            Self::Identity => "CSHARP_STAGED_IDENTITY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedCSharpRunError {
    code: StagedCSharpRunCode,
}

impl StagedCSharpRunError {
    pub const fn code(&self) -> StagedCSharpRunCode {
        self.code
    }
}

impl fmt::Display for StagedCSharpRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for StagedCSharpRunError {}

#[derive(Clone, Copy, Debug)]
pub struct StagedCSharpRunRequest<'a> {
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
pub struct AcceptedStagedCSharpRun {
    envelope: AcceptedSuccessorFrontendEnvelope,
    launcher: CSharpLauncherPlan,
    release_registry: ReleaseRegistryIdentity,
}

impl AcceptedStagedCSharpRun {
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

pub fn run_staged_installed_csharp_frontend(
    request: StagedCSharpRunRequest<'_>,
) -> Result<AcceptedStagedCSharpRun, StagedCSharpRunError> {
    let installed = StagedInstalledRelease::open().map_err(release_error)?;
    let semantic_registry = validate_inactive_semantic_profile_registry(
        &installed.semantic_registry_bytes,
        InactiveRegistryRevision::Revision2,
    )
    .map_err(|_| failure(StagedCSharpRunCode::Release))?;
    let registry =
        validate_successor_release_registry(&installed.registry_bytes, &semantic_registry)
            .map_err(|_| failure(StagedCSharpRunCode::Release))?;
    if registry.registry_sha256() != CSHARP_STAGING_REGISTRY_SHA256 {
        return Err(failure(StagedCSharpRunCode::Release));
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
        .map_err(|_| failure(StagedCSharpRunCode::Selection))?;
    validate_csharp_release(&semantic_registry, &resolved)?;
    let selection = validate_registry_selection_envelope(
        &semantic_registry,
        &resolved.semantic_context,
        request.selection,
    )
    .map_err(|_| failure(StagedCSharpRunCode::Selection))?;
    validate_captured_selection(&selection, request.captured_inputs)?;
    let launcher = csharp_launcher_plan(&resolved, &selection)?;

    let expected = BTreeMap::from([
        (
            resolved.frontend.bundle_id.clone(),
            &resolved.frontend.inventory,
        ),
        (
            resolved.toolchain.bundle_id.clone(),
            &resolved.toolchain.inventory,
        ),
    ]);
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
        .map_err(|_| failure(StagedCSharpRunCode::Sandbox))?;
    let output = launch_staged_csharp_frontend(
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
        return Err(failure(StagedCSharpRunCode::Protocol(
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
    .map_err(|error| failure(StagedCSharpRunCode::Protocol(error.code())))?;
    validate_emitted_release_identity(&envelope, &resolved)?;
    Ok(AcceptedStagedCSharpRun {
        envelope,
        launcher,
        release_registry,
    })
}

fn validate_csharp_registry_layout(
    registry: &ValidatedSuccessorReleaseRegistry,
) -> Result<(), StagedCSharpRunError> {
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
    if value.execution_host_profiles.len() != 1
        || value.native_runtime_layout_profiles.len() != 1
        || serde_json::to_value(&value.execution_host_profiles[0]).ok() != Some(expected_host)
        || serde_json::to_value(&value.native_runtime_layout_profiles[0]).ok()
            != Some(expected_layout)
    {
        return Err(failure(StagedCSharpRunCode::Identity));
    }
    Ok(())
}

fn validate_csharp_release(
    semantic_registry: &ValidatedSemanticProfileRegistry,
    resolved: &ResolvedSuccessorRelease<'_>,
) -> Result<(), StagedCSharpRunError> {
    let identity = semantic_registry.identity();
    if identity.revision() != 2
        || identity.registry_sha256()
            != "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75"
        || resolved.semantic_context.source_language() != "csharp"
        || resolved.semantic_context.semantic_profile() != "mpk.csharp.scalar.v0"
        || resolved.semantic_context.profile_entry_sha256() != CSHARP_PROFILE_ENTRY_SHA256
        || resolved.release_tuple.limit_profile_id != "mpk.vir.limits.v0"
        || resolved.frontend.bundle_id != CSHARP_FRONTEND_BUNDLE_ID
        || resolved.toolchain.bundle_id != CSHARP_TOOLCHAIN_BUNDLE_ID
        || resolved.toolchain.execution_host_profile_id != CSHARP_HOST_PROFILE_ID
    {
        return Err(failure(StagedCSharpRunCode::Contract));
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
        return Err(failure(StagedCSharpRunCode::Contract));
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
        return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
    }
    validate_toolchain_layout(resolved)?;
    Ok(())
}

fn validate_toolchain_layout(
    resolved: &ResolvedSuccessorRelease<'_>,
) -> Result<(), StagedCSharpRunError> {
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
        return Err(failure(StagedCSharpRunCode::Identity));
    }
    let dotnet = &resolved.toolchain.components[0];
    let ToolchainComponent::Executable {
        release,
        path,
        runtime,
        ..
    } = dotnet
    else {
        return Err(failure(StagedCSharpRunCode::Identity));
    };
    let ExecutableRuntime::Dynamic {
        interpreter_mount,
        libraries,
    } = runtime
    else {
        return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
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
            return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
    }
    Ok(())
}

fn validate_captured_selection(
    selection: &SelectionEnvelope,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), StagedCSharpRunError> {
    let value = selection
        .value()
        .as_object()
        .ok_or_else(|| failure(StagedCSharpRunCode::Selection))?;
    let sources = string_array(value.get("sources"))?;
    let contracts = string_array(value.get("contracts"))?;
    let mut expected = BTreeMap::new();
    for path in sources {
        if expected.insert(path, InputKind::Source).is_some() {
            return Err(failure(StagedCSharpRunCode::Selection));
        }
    }
    for path in contracts {
        if expected.insert(path, InputKind::Contract).is_some() {
            return Err(failure(StagedCSharpRunCode::Selection));
        }
    }
    if expected.len() != captured_inputs.len() {
        return Err(failure(StagedCSharpRunCode::Selection));
    }
    let mut observed = BTreeSet::new();
    for input in captured_inputs {
        if input.bytes.is_empty()
            || expected.get(input.normalized_path) != Some(&input.kind)
            || !observed.insert(input.normalized_path)
        {
            return Err(failure(StagedCSharpRunCode::Selection));
        }
    }
    Ok(())
}

fn csharp_launcher_plan(
    resolved: &ResolvedSuccessorRelease<'_>,
    selection: &SelectionEnvelope,
) -> Result<CSharpLauncherPlan, StagedCSharpRunError> {
    let value = selection
        .value()
        .as_object()
        .ok_or_else(|| failure(StagedCSharpRunCode::Selection))?;
    let compilation = value
        .get("compilation")
        .and_then(Value::as_str)
        .ok_or_else(|| failure(StagedCSharpRunCode::Selection))?;
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
        CSHARP_STAGING_REGISTRY_SHA256,
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
        return Err(failure(StagedCSharpRunCode::Launch));
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

fn string_array(value: Option<&Value>) -> Result<Vec<&str>, StagedCSharpRunError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| failure(StagedCSharpRunCode::Selection))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| failure(StagedCSharpRunCode::Selection))
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
) -> Result<(), StagedCSharpRunError> {
    let manifest = envelope
        .artifacts()
        .ok_or_else(|| failure(StagedCSharpRunCode::Identity))?
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
        || manifest.release_registry().registry_sha256 != CSHARP_STAGING_REGISTRY_SHA256
    {
        return Err(failure(StagedCSharpRunCode::Identity));
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
        return Err(failure(StagedCSharpRunCode::Identity));
    }
    Ok(())
}

fn selected_snapshot<'a>(
    snapshots: &'a BTreeMap<String, Arc<BundleSnapshot>>,
    bundle_id: &str,
) -> Result<&'a BundleSnapshot, StagedCSharpRunError> {
    snapshots
        .get(bundle_id)
        .map(Arc::as_ref)
        .ok_or_else(|| failure(StagedCSharpRunCode::Release))
}

fn release_error(error: crate::frontend_registry::FrontendReleaseError) -> StagedCSharpRunError {
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
    failure(StagedCSharpRunCode::Release)
}

fn sandbox_error(error: SandboxError) -> StagedCSharpRunError {
    match error {
        SandboxError::Unavailable => failure(StagedCSharpRunCode::Launch),
        SandboxError::Spawn | SandboxError::Killed => failure(StagedCSharpRunCode::Process),
    }
}

const fn failure(code: StagedCSharpRunCode) -> StagedCSharpRunError {
    StagedCSharpRunError { code }
}
