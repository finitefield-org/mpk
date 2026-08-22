use crate::cli::{LowerRequest, RustTarget};
use crate::driver_protocol::{self, DriverReleaseIdentity};
use crate::environment::{
    EvidenceEnvironment, ARGUMENT_PROFILE_ID, CARGO_PATH, DRIVER_OUTPUT_ROOT,
    ENVIRONMENT_PROFILE_ID, FRONTEND_ROOT, HOME_ROOT, INPUT_ROOT, NATIVE_RUNTIME_ROOT, TARGET_ROOT,
    TEMP_ROOT, TOOLCHAIN_ROOT, WORK_ROOT,
};
use crate::sha256::{digest, hex};
use crate::snapshot::Snapshot;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::sync::atomic::AtomicBool;
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::thread;

#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

pub const EXECUTION_HOST_PROFILE_ID: &str = "mpk.host.linux-x86_64-gnu.glibc2_27.v0";
pub const RUNTIME_LAYOUT_PROFILE_ID: &str = "mpk.runtime.linux-x86_64-gnu.glibc2_27.v0";
pub const FRONTEND_BUNDLE_ID: &str = "frontend.rust.rust2vir.candidate.v0";
pub const TOOLCHAIN_BUNDLE_ID: &str = "toolchain.rust.nightly-2025-06-01.candidate.v0";
pub const LIMIT_PROFILE_ID: &str = "mpk.vir.limits.v0";

pub const PROCESS_LIMIT: u64 = 256;
pub const OPEN_FILE_LIMIT: u64 = 1_024;
pub const VIRTUAL_MEMORY_LIMIT: u64 = 17_179_869_184;
pub const RESIDENT_MEMORY_LIMIT: u64 = 8_589_934_592;
pub const TEMP_BYTES_LIMIT: u64 = 4_294_967_296;
pub const TARGET_BYTES_LIMIT: u64 = 17_179_869_184;
pub const OUTPUT_FILES_LIMIT: u64 = 262_144;
pub const STDOUT_BYTES_LIMIT: usize = 67_108_864;
pub const STDERR_BYTES_LIMIT: usize = 2_097_152;
const CANDIDATE_FILE_COUNT_LIMIT: usize = 1_048_576;
const CANDIDATE_FILE_SIZE_LIMIT: u64 = 4_294_967_296;
const CANDIDATE_AGGREGATE_LIMIT: u64 = 34_359_738_368;
const CANDIDATE_PATH_LIMIT: usize = 1_024;

static NEXT_INVOCATION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryFile {
    path: String,
    executable: bool,
    size_bytes: u64,
    sha256: String,
}

impl InventoryFile {
    pub fn new(
        path: impl Into<String>,
        executable: bool,
        size_bytes: u64,
        sha256: impl Into<String>,
    ) -> Result<Self, SandboxError> {
        let value = Self {
            path: path.into(),
            executable,
            size_bytes,
            sha256: sha256.into(),
        };
        value.validate_shape()?;
        Ok(value)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn executable(&self) -> bool {
        self.executable
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    fn validate_shape(&self) -> Result<(), SandboxError> {
        validate_relative_path(&self.path)?;
        validate_sha256(&self.sha256)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetLibrary {
    target: RustTarget,
    pointer_width: u8,
    component_name: String,
    component_content_sha256: String,
    path_prefix: String,
}

impl TargetLibrary {
    pub fn new(
        target: RustTarget,
        pointer_width: u8,
        component_name: impl Into<String>,
        component_content_sha256: impl Into<String>,
        path_prefix: impl Into<String>,
    ) -> Result<Self, SandboxError> {
        let value = Self {
            target,
            pointer_width,
            component_name: component_name.into(),
            component_content_sha256: component_content_sha256.into(),
            path_prefix: path_prefix.into(),
        };
        if value.pointer_width != value.target.pointer_width()
            || value.component_name.is_empty()
            || !value
                .component_name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(SandboxError::ToolchainTarget);
        }
        validate_sha256(&value.component_content_sha256)?;
        validate_relative_path(value.path_prefix.trim_end_matches('/'))?;
        if !value.path_prefix.ends_with('/') {
            return Err(SandboxError::ToolchainTarget);
        }
        Ok(value)
    }

    pub fn target(&self) -> RustTarget {
        self.target
    }

    pub fn component_name(&self) -> &str {
        &self.component_name
    }

    pub fn component_content_sha256(&self) -> &str {
        &self.component_content_sha256
    }
}

#[derive(Clone, Debug)]
pub struct InjectedCandidate {
    frontend_bundle_id: String,
    frontend_bundle_sha256: String,
    frontend_root: PathBuf,
    frontend_inventory: Vec<InventoryFile>,
    toolchain_bundle_id: String,
    toolchain_distribution_sha256: String,
    toolchain_root: PathBuf,
    toolchain_inventory: Vec<InventoryFile>,
    target_libraries: Vec<TargetLibrary>,
    execution_host_profile_id: String,
    runtime_layout_profile_id: String,
    driver_release_identity: DriverReleaseIdentity,
}

#[derive(Clone, Debug)]
pub struct CandidateDefinition {
    pub frontend_bundle_id: String,
    pub frontend_bundle_sha256: String,
    pub frontend_root: PathBuf,
    pub frontend_inventory: Vec<InventoryFile>,
    pub toolchain_bundle_id: String,
    pub toolchain_distribution_sha256: String,
    pub toolchain_root: PathBuf,
    pub toolchain_inventory: Vec<InventoryFile>,
    pub target_libraries: Vec<TargetLibrary>,
    pub execution_host_profile_id: String,
    pub runtime_layout_profile_id: String,
    pub environment_profile_id: String,
    pub argument_profile_id: String,
    pub limit_profile_id: String,
    pub compiler_release: String,
    pub compiler_commit: String,
    pub native_runtime_component_root: String,
    pub native_runtime_content_sha256: String,
    pub driver_release_identity: DriverReleaseIdentity,
}

impl InjectedCandidate {
    pub fn from_definition(definition: CandidateDefinition) -> Result<Self, SandboxError> {
        validate_sha256(&definition.frontend_bundle_sha256)?;
        validate_sha256(&definition.toolchain_distribution_sha256)?;
        validate_inventory_shape(&definition.frontend_inventory)?;
        validate_inventory_shape(&definition.toolchain_inventory)?;
        let frontend_sha256 = frontend_bundle_sha256(
            &definition.frontend_bundle_id,
            &definition.frontend_inventory,
        );
        let distribution_sha256 = toolchain_distribution_sha256(
            &definition.toolchain_bundle_id,
            &definition.toolchain_inventory,
        );
        let native_runtime_sha256 = component_content_sha256(
            &definition.toolchain_bundle_id,
            "native-runtime",
            &definition.toolchain_inventory,
            "native-runtime/",
        );
        if definition.frontend_bundle_id != FRONTEND_BUNDLE_ID
            || definition.toolchain_bundle_id != TOOLCHAIN_BUNDLE_ID
            || definition.execution_host_profile_id != EXECUTION_HOST_PROFILE_ID
            || definition.runtime_layout_profile_id != RUNTIME_LAYOUT_PROFILE_ID
            || definition.environment_profile_id != ENVIRONMENT_PROFILE_ID
            || definition.argument_profile_id != ARGUMENT_PROFILE_ID
            || definition.limit_profile_id != LIMIT_PROFILE_ID
            || definition.compiler_release != crate::EXPECTED_RUSTC_RELEASE
            || definition.compiler_commit != crate::EXPECTED_RUSTC_COMMIT
            || definition.native_runtime_component_root != "native-runtime"
            || !definition
                .toolchain_inventory
                .iter()
                .any(|file| file.path.starts_with("native-runtime/"))
            || definition.frontend_bundle_sha256 != frontend_sha256
            || definition.toolchain_distribution_sha256 != distribution_sha256
            || definition.native_runtime_content_sha256 != native_runtime_sha256
            || definition.target_libraries.len() != 2
            || definition.target_libraries[0].target != RustTarget::I686UnknownLinuxGnu
            || definition.target_libraries[1].target != RustTarget::X86_64UnknownLinuxGnu
            || definition.target_libraries[0].component_name != "rust-target-i686"
            || definition.target_libraries[1].component_name != "rust-target-x86_64"
            || validate_driver_release_identity(&definition).is_err()
        {
            return Err(SandboxError::ToolchainComponent);
        }
        for target in &definition.target_libraries {
            if !definition
                .toolchain_inventory
                .iter()
                .any(|file| file.path.starts_with(&target.path_prefix))
                || target.component_content_sha256
                    != component_content_sha256(
                        &definition.toolchain_bundle_id,
                        &target.component_name,
                        &definition.toolchain_inventory,
                        &target.path_prefix,
                    )
            {
                return Err(SandboxError::ToolchainTarget);
            }
        }
        Ok(Self {
            frontend_bundle_id: definition.frontend_bundle_id,
            frontend_bundle_sha256: definition.frontend_bundle_sha256,
            frontend_root: definition.frontend_root,
            frontend_inventory: definition.frontend_inventory,
            toolchain_bundle_id: definition.toolchain_bundle_id,
            toolchain_distribution_sha256: definition.toolchain_distribution_sha256,
            toolchain_root: definition.toolchain_root,
            toolchain_inventory: definition.toolchain_inventory,
            target_libraries: definition.target_libraries,
            execution_host_profile_id: definition.execution_host_profile_id,
            runtime_layout_profile_id: definition.runtime_layout_profile_id,
            driver_release_identity: definition.driver_release_identity,
        })
    }

    pub fn frontend_root(&self) -> &Path {
        &self.frontend_root
    }

    pub fn toolchain_root(&self) -> &Path {
        &self.toolchain_root
    }

    pub fn native_runtime_root(&self) -> PathBuf {
        self.toolchain_root.join("native-runtime")
    }

    pub fn driver_release_identity(&self) -> &DriverReleaseIdentity {
        &self.driver_release_identity
    }

    pub fn validate_for(&self, request: &LowerRequest) -> Result<ValidatedCandidate, SandboxError> {
        if request.release.frontend_bundle_id != self.frontend_bundle_id
            || request.release.frontend_sha256 != self.frontend_bundle_sha256
            || request.release.toolchain_bundle_id != self.toolchain_bundle_id
            || request.release.toolchain_distribution_sha256 != self.toolchain_distribution_sha256
            || request.release.toolchain_root != self.toolchain_root
            || request.release.driver != self.frontend_root.join("bin/rust2vir-driver")
        {
            return Err(SandboxError::ToolchainComponent);
        }
        let target = self
            .target_libraries
            .iter()
            .find(|library| library.target == request.target)
            .ok_or(SandboxError::ToolchainTarget)?;
        if target.pointer_width != request.target.pointer_width()
            || !self
                .toolchain_inventory
                .iter()
                .any(|file| file.path.starts_with(&target.path_prefix))
        {
            return Err(SandboxError::ToolchainTarget);
        }

        let driver = required_file(&self.frontend_inventory, "bin/rust2vir-driver", true)?;
        if driver.sha256 != request.release.driver_sha256 {
            return Err(SandboxError::ToolchainComponent);
        }
        required_file(&self.frontend_inventory, "bin/rust2vir", true)?;
        required_file(&self.toolchain_inventory, "bin/cargo", true)?;
        required_file(&self.toolchain_inventory, "bin/rustc", true)?;
        required_file(
            &self.toolchain_inventory,
            "native-runtime/lib64/ld-linux-x86-64.so.2",
            true,
        )?;
        for runtime_file in [
            "native-runtime/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
            "native-runtime/lib/x86_64-linux-gnu/libc.so.6",
            "native-runtime/lib/x86_64-linux-gnu/libdl.so.2",
            "native-runtime/lib/x86_64-linux-gnu/libgcc_s.so.1",
            "native-runtime/lib/x86_64-linux-gnu/libm.so.6",
            "native-runtime/lib/x86_64-linux-gnu/libpthread.so.0",
            "native-runtime/lib/x86_64-linux-gnu/librt.so.1",
            "native-runtime/lib/x86_64-linux-gnu/libstdc++.so.6",
            "native-runtime/lib/x86_64-linux-gnu/libtinfo.so.5",
            "native-runtime/lib/x86_64-linux-gnu/libz.so.1",
        ] {
            required_file(&self.toolchain_inventory, runtime_file, false)?;
        }
        if !self
            .toolchain_inventory
            .iter()
            .any(|file| file.path.starts_with("lib/"))
            || !self
                .toolchain_inventory
                .iter()
                .any(|file| file.path.starts_with("native-runtime/"))
        {
            return Err(SandboxError::ToolchainComponent);
        }

        let frontend_handle =
            validate_inventory_root(&self.frontend_root, &self.frontend_inventory)?;
        let toolchain_handle =
            validate_inventory_root(&self.toolchain_root, &self.toolchain_inventory)?;
        validate_distinct_roots(&frontend_handle, &toolchain_handle)?;
        #[cfg(target_os = "linux")]
        let retained_native_runtime = PathBuf::from(format!(
            "/proc/self/fd/{}/native-runtime",
            toolchain_handle.as_raw_fd()
        ));
        #[cfg(not(target_os = "linux"))]
        let retained_native_runtime = self.native_runtime_root();
        let native_runtime_handle = open_directory_nofollow(&retained_native_runtime)?;
        Ok(ValidatedCandidate {
            frontend_root: self.frontend_root.clone(),
            toolchain_root: self.toolchain_root.clone(),
            native_runtime_root: self.native_runtime_root(),
            target: request.target,
            execution_host_profile_id: self.execution_host_profile_id.clone(),
            runtime_layout_profile_id: self.runtime_layout_profile_id.clone(),
            frontend_handle,
            toolchain_handle,
            native_runtime_handle,
        })
    }
}

fn validate_driver_release_identity(definition: &CandidateDefinition) -> Result<(), SandboxError> {
    let identity = &definition.driver_release_identity;
    if identity.frontend_bundle_id != definition.frontend_bundle_id
        || identity.toolchain_bundle_id != definition.toolchain_bundle_id
        || identity.toolchain_distribution_sha256 != definition.toolchain_distribution_sha256
        || identity.frontend_binary_sha256
            != required_file(&definition.frontend_inventory, "bin/rust2vir", true)?.sha256
        || identity.driver_binary_sha256
            != required_file(&definition.frontend_inventory, "bin/rust2vir-driver", true)?.sha256
        || identity.toolchain_components.is_empty()
        || identity
            .toolchain_components
            .iter()
            .map(|component| component.name.as_str())
            .ne([
                "cargo",
                "native-runtime",
                "rust-compiler-runtime",
                "rust-target-i686",
                "rust-target-x86_64",
                "rustc",
            ])
    {
        return Err(SandboxError::ToolchainComponent);
    }
    let mut previous = None;
    for component in &identity.toolchain_components {
        validate_sha256(&component.sha256)?;
        if component.name.is_empty()
            || component.release.is_empty()
            || previous
                .is_some_and(|previous: &str| previous.as_bytes() >= component.name.as_bytes())
        {
            return Err(SandboxError::ToolchainComponent);
        }
        previous = Some(component.name.as_str());
        match (component.kind.as_str(), component.name.as_str()) {
            ("executable", "cargo") => {
                if component.commit_hash.is_some()
                    || component.sha256
                        != required_file(&definition.toolchain_inventory, "bin/cargo", true)?.sha256
                {
                    return Err(SandboxError::ToolchainComponent);
                }
            }
            ("executable", "rustc") => {
                if component.release != definition.compiler_release
                    || component.commit_hash.as_deref() != Some(&definition.compiler_commit)
                    || component.sha256
                        != required_file(&definition.toolchain_inventory, "bin/rustc", true)?.sha256
                {
                    return Err(SandboxError::ToolchainComponent);
                }
            }
            (
                "content",
                "native-runtime"
                | "rust-compiler-runtime"
                | "rust-target-i686"
                | "rust-target-x86_64",
            ) if component.commit_hash.is_none() => {}
            _ => return Err(SandboxError::ToolchainComponent),
        }
    }
    for (name, hash) in [
        (
            "native-runtime",
            definition.native_runtime_content_sha256.as_str(),
        ),
        (
            "rust-target-i686",
            definition.target_libraries[0]
                .component_content_sha256
                .as_str(),
        ),
        (
            "rust-target-x86_64",
            definition.target_libraries[1]
                .component_content_sha256
                .as_str(),
        ),
    ] {
        if !identity.toolchain_components.iter().any(|component| {
            component.kind == "content" && component.name == name && component.sha256 == hash
        }) {
            return Err(SandboxError::ToolchainComponent);
        }
    }
    Ok(())
}

pub struct ValidatedCandidate {
    frontend_root: PathBuf,
    toolchain_root: PathBuf,
    native_runtime_root: PathBuf,
    target: RustTarget,
    execution_host_profile_id: String,
    runtime_layout_profile_id: String,
    frontend_handle: File,
    toolchain_handle: File,
    native_runtime_handle: File,
}

impl std::fmt::Debug for ValidatedCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedCandidate")
            .field("frontend_root", &self.frontend_root)
            .field("toolchain_root", &self.toolchain_root)
            .field("native_runtime_root", &self.native_runtime_root)
            .field("target", &self.target)
            .field("execution_host_profile_id", &self.execution_host_profile_id)
            .field("runtime_layout_profile_id", &self.runtime_layout_profile_id)
            .finish_non_exhaustive()
    }
}

impl ValidatedCandidate {
    pub fn frontend_root(&self) -> &Path {
        &self.frontend_root
    }

    pub fn toolchain_root(&self) -> &Path {
        &self.toolchain_root
    }

    pub fn native_runtime_root(&self) -> &Path {
        &self.native_runtime_root
    }

    pub fn target(&self) -> RustTarget {
        self.target
    }

    pub fn execution_host_profile_id(&self) -> &str {
        &self.execution_host_profile_id
    }

    pub fn runtime_layout_profile_id(&self) -> &str {
        &self.runtime_layout_profile_id
    }

    #[cfg(target_os = "linux")]
    fn retained_descriptors(&self) -> [i32; 3] {
        [
            self.frontend_handle.as_raw_fd(),
            self.toolchain_handle.as_raw_fd(),
            self.native_runtime_handle.as_raw_fd(),
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoInvocationKind {
    Metadata,
    Check,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoInvocation {
    kind: CargoInvocationKind,
    arguments: Vec<String>,
}

impl CargoInvocation {
    pub fn new(kind: CargoInvocationKind, arguments: &[&str]) -> Result<Self, SandboxError> {
        if arguments.is_empty()
            || arguments
                .iter()
                .any(|argument| argument.is_empty() || argument.contains('\0'))
        {
            return Err(SandboxError::ToolchainArgument);
        }
        Ok(Self {
            kind,
            arguments: arguments
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
        })
    }

    pub fn kind(&self) -> CargoInvocationKind {
        self.kind
    }

    pub fn executable(&self) -> &'static str {
        CARGO_PATH
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessOutput {
    pub exit_code: Option<i32>,
    pub signaled: bool,
    pub stdout: Vec<u8>,
    pub stderr_observed_bytes: usize,
    pub stdout_limit_exceeded: bool,
    pub stderr_limit_exceeded: bool,
}

impl ProcessOutput {
    pub fn success(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            exit_code: Some(0),
            signaled: false,
            stdout: stdout.into(),
            stderr_observed_bytes: 0,
            stdout_limit_exceeded: false,
            stderr_limit_exceeded: false,
        }
    }

    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0) && !self.signaled
    }

    pub fn exceeded_stream_limit(&self) -> bool {
        self.stdout_limit_exceeded || self.stderr_limit_exceeded
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxLimits {
    pub processes: u64,
    pub open_files_per_process: u64,
    pub virtual_memory_bytes: u64,
    pub resident_memory_bytes: u64,
    pub temp_bytes: u64,
    pub target_bytes: u64,
    pub output_files: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

impl SandboxLimits {
    pub const FROZEN: Self = Self {
        processes: PROCESS_LIMIT,
        open_files_per_process: OPEN_FILE_LIMIT,
        virtual_memory_bytes: VIRTUAL_MEMORY_LIMIT,
        resident_memory_bytes: RESIDENT_MEMORY_LIMIT,
        temp_bytes: TEMP_BYTES_LIMIT,
        target_bytes: TARGET_BYTES_LIMIT,
        output_files: OUTPUT_FILES_LIMIT,
        stdout_bytes: STDOUT_BYTES_LIMIT,
        stderr_bytes: STDERR_BYTES_LIMIT,
    };
}

#[derive(Debug)]
pub struct SandboxContext<'a> {
    invocation_id: u64,
    snapshot_root: &'a Path,
    candidate: &'a ValidatedCandidate,
    workspace: &'a InvocationWorkspace,
    environment: &'a EvidenceEnvironment,
    limits: SandboxLimits,
    snapshot_handle: &'a File,
}

impl SandboxContext<'_> {
    pub fn invocation_id(&self) -> u64 {
        self.invocation_id
    }

    pub fn snapshot_root(&self) -> &Path {
        self.snapshot_root
    }

    pub fn candidate(&self) -> &ValidatedCandidate {
        self.candidate
    }

    pub fn environment(&self) -> &EvidenceEnvironment {
        self.environment
    }

    pub fn limits(&self) -> SandboxLimits {
        self.limits
    }

    pub fn writable_host_path(&self, sandbox_path: &str) -> Option<&Path> {
        self.workspace.writable_host_path(sandbox_path)
    }

    pub fn rootfs(&self) -> &Path {
        &self.workspace.rootfs
    }

    pub fn driver_request_host_path(&self) -> &Path {
        self.workspace.driver_request_host_path()
    }

    #[cfg(target_os = "linux")]
    fn retained_descriptors(&self) -> Vec<i32> {
        let [frontend, toolchain, native_runtime] = self.candidate.retained_descriptors();
        let mut descriptors = vec![
            self.snapshot_handle.as_raw_fd(),
            frontend,
            toolchain,
            native_runtime,
        ];
        descriptors.extend(self.workspace.retained_descriptors());
        descriptors
    }
}

pub trait SandboxExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxError {
    SandboxUnavailable,
    ToolchainComponent,
    ToolchainTarget,
    ToolchainArgument,
    ChildOutputLimit,
    Spawn,
    Killed,
    FilesystemLimit,
}

impl SandboxError {
    pub fn code(self) -> &'static str {
        match self {
            Self::SandboxUnavailable => "FRONTEND_SANDBOX_UNAVAILABLE",
            Self::ToolchainComponent => "RUST_TOOLCHAIN_COMPONENT",
            Self::ToolchainTarget => "RUST_TOOLCHAIN_TARGET",
            Self::ToolchainArgument => "RUST_TOOLCHAIN_ARGUMENT",
            Self::ChildOutputLimit | Self::FilesystemLimit => "RUST_FRONTEND_CHILD_OUTPUT_LIMIT",
            Self::Spawn | Self::Killed => "RUST_FRONTEND_COMPILER_CRASH",
        }
    }
}

pub struct PreparedSandbox<'a, E> {
    request: &'a LowerRequest,
    snapshot: &'a Snapshot,
    candidate_source: &'a InjectedCandidate,
    candidate: ValidatedCandidate,
    snapshot_handle: File,
    environment: EvidenceEnvironment,
    workspace: InvocationWorkspace,
    executor: E,
    observed_stdout_bytes: usize,
    observed_stderr_bytes: usize,
    driver_request: driver_protocol::DriverRequest,
}

impl<'a, E: SandboxExecutor> PreparedSandbox<'a, E> {
    pub fn prepare(
        request: &'a LowerRequest,
        snapshot: &'a Snapshot,
        candidate: &'a InjectedCandidate,
        private_parent: &Path,
        executor: E,
    ) -> Result<Self, SandboxError> {
        snapshot
            .validate()
            .map_err(|_| SandboxError::SandboxUnavailable)?;
        let validated = candidate.validate_for(request)?;
        let snapshot_handle = snapshot
            .try_clone_root()
            .map_err(|_| SandboxError::SandboxUnavailable)?;
        let environment = EvidenceEnvironment::frozen();
        environment
            .validate()
            .map_err(|_| SandboxError::ToolchainArgument)?;
        let driver_request = driver_protocol::construct_request(
            request,
            snapshot.inputs(),
            candidate.driver_release_identity(),
        )
        .map_err(|_| SandboxError::ToolchainComponent)?;
        let workspace = InvocationWorkspace::create(
            private_parent,
            snapshot.path(),
            &validated.frontend_root,
            &validated.toolchain_root,
            driver_request.transport(),
        )?;
        Ok(Self {
            request,
            snapshot,
            candidate_source: candidate,
            candidate: validated,
            snapshot_handle,
            environment,
            workspace,
            executor,
            observed_stdout_bytes: 0,
            observed_stderr_bytes: 0,
            driver_request,
        })
    }

    pub(crate) fn execute(
        &mut self,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        self.snapshot
            .validate()
            .map_err(|_| SandboxError::SandboxUnavailable)?;
        self.candidate = self.candidate_source.validate_for(self.request)?;
        self.workspace.validate(
            self.snapshot.path(),
            &self.candidate.frontend_root,
            &self.candidate.toolchain_root,
        )?;
        self.workspace.validate_before(invocation.kind())?;
        self.environment
            .validate()
            .map_err(|_| SandboxError::ToolchainArgument)?;
        let context = SandboxContext {
            invocation_id: self.workspace.invocation_id,
            snapshot_root: self.snapshot.path(),
            candidate: &self.candidate,
            workspace: &self.workspace,
            environment: &self.environment,
            limits: SandboxLimits::FROZEN,
            snapshot_handle: &self.snapshot_handle,
        };
        let output = self.executor.execute(&context, invocation)?;
        self.observed_stdout_bytes = self
            .observed_stdout_bytes
            .checked_add(output.stdout.len())
            .ok_or(SandboxError::ChildOutputLimit)?;
        self.observed_stderr_bytes = self
            .observed_stderr_bytes
            .checked_add(output.stderr_observed_bytes)
            .ok_or(SandboxError::ChildOutputLimit)?;
        if output.exceeded_stream_limit()
            || self.observed_stdout_bytes > STDOUT_BYTES_LIMIT
            || self.observed_stderr_bytes > STDERR_BYTES_LIMIT
        {
            return Err(SandboxError::ChildOutputLimit);
        }
        self.workspace.validate_after(invocation.kind())?;
        Ok(output)
    }

    pub fn environment(&self) -> &EvidenceEnvironment {
        &self.environment
    }

    pub fn invocation_id(&self) -> u64 {
        self.workspace.invocation_id
    }

    pub fn driver_request(&self) -> &driver_protocol::DriverRequest {
        &self.driver_request
    }

    pub fn consume_driver_output(
        &self,
        driver_exit_code: i32,
        signaled: bool,
    ) -> Result<driver_protocol::DriverOutput, driver_protocol::DriverProtocolError> {
        let bytes = crate::driver_process::consume_result_from_open_directory(
            self.workspace.driver_output_handle(),
        )?;
        driver_protocol::parse_output_transport(
            &bytes,
            &self.driver_request,
            driver_exit_code,
            signaled,
        )
    }

    pub(crate) fn consume_driver_output_artifact(
        &self,
    ) -> Result<driver_protocol::DriverOutput, driver_protocol::DriverProtocolError> {
        let bytes = crate::driver_process::consume_result_from_open_directory(
            self.workspace.driver_output_handle(),
        )?;
        driver_protocol::parse_output_artifact(&bytes, &self.driver_request)
    }

    pub(crate) fn driver_output_is_empty(
        &self,
    ) -> Result<bool, driver_protocol::DriverProtocolError> {
        crate::driver_process::open_directory_is_empty(self.workspace.driver_output_handle())
    }

    pub(crate) fn target_id(&self) -> &'static str {
        self.candidate.target.id()
    }

    pub fn into_executor(self) -> E {
        self.executor
    }
}

#[derive(Default)]
pub struct LinuxNamespaceExecutor;

impl SandboxExecutor for LinuxNamespaceExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (context, invocation);
            Err(SandboxError::SandboxUnavailable)
        }
        #[cfg(target_os = "linux")]
        {
            execute_linux_namespace(context, invocation)
        }
    }
}

#[cfg(target_os = "linux")]
fn execute_linux_namespace(
    context: &SandboxContext<'_>,
    invocation: &CargoInvocation,
) -> Result<ProcessOutput, SandboxError> {
    use std::os::unix::process::{CommandExt, ExitStatusExt};

    let retained_descriptors = context.retained_descriptors();
    let retained_sources = [
        retained_fd_path(retained_descriptors[0]),
        retained_fd_path(retained_descriptors[2]),
        retained_fd_path(retained_descriptors[1]),
        retained_fd_path(retained_descriptors[3]),
        retained_fd_path(retained_descriptors[4]),
        retained_fd_path(retained_descriptors[5]),
        retained_fd_path(retained_descriptors[6]),
        retained_fd_path(retained_descriptors[7]),
        retained_fd_path(retained_descriptors[8]),
        retained_fd_path(retained_descriptors[9]),
        retained_fd_path(retained_descriptors[10]),
    ];
    if !context.rootfs().is_absolute() || context.rootfs().to_str().is_none() {
        return Err(SandboxError::SandboxUnavailable);
    }

    let mut arguments = vec![
        "__rust2vir_cargo_sandbox_v0".to_owned(),
        context
            .rootfs()
            .to_str()
            .ok_or(SandboxError::SandboxUnavailable)?
            .to_owned(),
    ];
    arguments.extend(retained_sources.iter().map(|path| path.to_owned()));
    arguments.push("--".to_owned());
    arguments.extend(invocation.arguments().iter().cloned());

    let bootstrap_loader = format!("{}/lib64/ld-linux-x86-64.so.2", retained_sources[3]);
    let bootstrap_library_path = format!("{}/lib/x86_64-linux-gnu", retained_sources[3]);
    let bootstrap_executable = format!("{}/bin/rust2vir", retained_sources[2]);
    let mut command = Command::new(bootstrap_loader);
    command
        .arg("--library-path")
        .arg(bootstrap_library_path)
        .arg(bootstrap_executable)
        .args(arguments)
        .env_clear()
        .envs(context.environment().entries())
        .current_dir(&context.workspace.root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    // SAFETY: the callback performs only fcntl on already-open descriptors. It neither
    // allocates nor touches shared application state between fork and exec.
    unsafe {
        command.pre_exec(move || {
            for descriptor in retained_descriptors.iter().copied() {
                linux::inherit_descriptor(descriptor)?;
            }
            Ok(())
        });
    }
    let mut child = command
        .spawn()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    let child_id = child.id();
    let stdout = child.stdout.take().ok_or(SandboxError::Spawn)?;
    let stderr = child.stderr.take().ok_or(SandboxError::Spawn)?;
    let overflow = Arc::new(AtomicBool::new(false));
    let read_failed = Arc::new(AtomicBool::new(false));
    let stdout_reader = bounded_reader(
        stdout,
        context.limits().stdout_bytes,
        true,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let stderr_reader = bounded_reader(
        stderr,
        context.limits().stderr_bytes,
        false,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            kill_process_group(child_id);
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::yield_now(),
            Err(_) => {
                kill_process_group(child_id);
                let _ = child.kill();
                let _ = child.wait();
                return Err(SandboxError::Killed);
            }
        }
    };
    kill_process_group(child_id);
    let (stdout, stdout_observed) = stdout_reader.join().map_err(|_| SandboxError::Killed)?;
    let (_, stderr_observed) = stderr_reader.join().map_err(|_| SandboxError::Killed)?;
    if read_failed.load(Ordering::Acquire) {
        return Err(SandboxError::Killed);
    }
    if status.code() == Some(125) {
        return Err(SandboxError::SandboxUnavailable);
    }
    if status.code() == Some(126) {
        return Err(SandboxError::SandboxUnavailable);
    }
    Ok(ProcessOutput {
        exit_code: status.code(),
        signaled: status.signal().is_some(),
        stdout,
        stderr_observed_bytes: stderr_observed,
        stdout_limit_exceeded: stdout_observed > context.limits().stdout_bytes,
        stderr_limit_exceeded: stderr_observed > context.limits().stderr_bytes,
    })
}

#[cfg(target_os = "linux")]
fn retained_fd_path(descriptor: i32) -> String {
    format!("/proc/self/fd/{descriptor}")
}

#[cfg(target_os = "linux")]
fn bounded_reader<R: Read + Send + 'static>(
    mut reader: R,
    maximum: usize,
    retain: bool,
    overflow: Arc<AtomicBool>,
    read_failed: Arc<AtomicBool>,
) -> thread::JoinHandle<(Vec<u8>, usize)> {
    thread::spawn(move || {
        let mut retained = Vec::new();
        let mut observed = 0_usize;
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Err(_) => {
                    read_failed.store(true, Ordering::Release);
                    break;
                }
                Ok(count) => {
                    observed = observed.saturating_add(count);
                    if observed > maximum {
                        overflow.store(true, Ordering::Release);
                    }
                    if retain && retained.len() <= maximum {
                        let remaining = maximum.saturating_add(1).saturating_sub(retained.len());
                        retained.extend_from_slice(&buffer[..count.min(remaining)]);
                    }
                }
            }
        }
        (retained, observed)
    })
}

#[cfg(target_os = "linux")]
fn kill_process_group(process: u32) {
    if let Ok(process) = i32::try_from(process) {
        // SAFETY: a negative, validated child process-group ID and SIGKILL contain no pointers.
        let _ = unsafe { linux::kill(-process, linux::SIGKILL) };
    }
}

pub fn run_bootstrap(arguments: &[String]) -> u8 {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = arguments;
        125
    }
    #[cfg(target_os = "linux")]
    {
        linux_bootstrap(arguments).unwrap_or_else(|code| code)
    }
}

#[cfg(target_os = "linux")]
fn linux_bootstrap(arguments: &[String]) -> Result<u8, u8> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::process::ExitStatusExt;

    if arguments.len() < 14 || arguments[12] != "--" {
        return Err(125);
    }
    let paths = arguments[..12]
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if paths
        .iter()
        .any(|path| !path.is_absolute() || path.as_os_str().as_bytes().contains(&0))
        || arguments[13..]
            .iter()
            .any(|argument| argument.is_empty() || argument.contains('\0'))
        || current_environment() != EvidenceEnvironment::frozen().entries().clone()
    {
        return Err(125);
    }
    let [rootfs, input, toolchain, frontend, native_runtime, work, home, cargo_home, temporary, target, driver_output, driver_request]: [&Path; 12] =
        paths.iter().map(PathBuf::as_path).collect::<Vec<_>>().try_into().map_err(|_| 125)?;

    linux::enter_namespaces()?;
    linux::bind_root(rootfs)?;
    for (source, destination, executable) in [
        (input, "mpk/input", false),
        (toolchain, "mpk/toolchain", true),
        (frontend, "mpk/frontend", true),
        (work, "mpk/work", false),
        (native_runtime, "mpk/native-runtime", true),
        (
            &native_runtime.join("lib/x86_64-linux-gnu"),
            "lib/x86_64-linux-gnu",
            true,
        ),
    ] {
        linux::bind_view(source, &rootfs.join(destination), true, executable)?;
    }
    linux::bind_view(
        &native_runtime.join("lib64/ld-linux-x86-64.so.2"),
        &rootfs.join("lib64/ld-linux-x86-64.so.2"),
        true,
        true,
    )?;
    linux::bind_view(
        driver_request,
        &rootfs.join("mpk/driver-request.json"),
        true,
        false,
    )?;
    for (source, destination) in [
        (home, "mpk/home"),
        (cargo_home, "mpk/cargo-home"),
        (temporary, "mpk/tmp"),
        (target, "mpk/target"),
        (driver_output, "mpk/driver-output"),
    ] {
        linux::bind_view(source, &rootfs.join(destination), false, false)?;
    }
    linux::seal_root(rootfs)?;
    linux::enter_root(rootfs)?;
    linux::apply_process_controls()?;
    for source in &paths[1..] {
        linux::close_retained_path(source)?;
    }

    let environment = EvidenceEnvironment::frozen();
    let status = Command::new(CARGO_PATH)
        .args(&arguments[13..])
        .env_clear()
        .envs(environment.entries())
        .current_dir(WORK_ROOT)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| 126)?;
    if status.signal().is_some() {
        return Err(125);
    }
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .ok_or(125)
}

#[cfg(target_os = "linux")]
fn current_environment() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{OPEN_FILE_LIMIT, PROCESS_LIMIT, RESIDENT_MEMORY_LIMIT, VIRTUAL_MEMORY_LIMIT};
    use std::ffi::{c_char, c_int, c_ulong, c_void, CString};
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    pub(super) const SIGKILL: c_int = 9;
    const CLONE_NEWNS: c_int = 0x0002_0000;
    const CLONE_NEWUTS: c_int = 0x0400_0000;
    const CLONE_NEWIPC: c_int = 0x0800_0000;
    const CLONE_NEWUSER: c_int = 0x1000_0000;
    const CLONE_NEWPID: c_int = 0x2000_0000;
    const CLONE_NEWNET: c_int = 0x4000_0000;
    const MS_RDONLY: c_ulong = 1;
    const MS_NOSUID: c_ulong = 2;
    const MS_NODEV: c_ulong = 4;
    const MS_NOEXEC: c_ulong = 8;
    const MS_REMOUNT: c_ulong = 32;
    const MS_BIND: c_ulong = 4_096;
    const MS_REC: c_ulong = 16_384;
    const MS_PRIVATE: c_ulong = 1 << 18;
    const PR_SET_NO_NEW_PRIVS: c_int = 38;
    const RLIMIT_FSIZE: c_int = 1;
    const RLIMIT_CORE: c_int = 4;
    const RLIMIT_RSS: c_int = 5;
    const RLIMIT_NPROC: c_int = 6;
    const RLIMIT_NOFILE: c_int = 7;
    const RLIMIT_AS: c_int = 9;

    #[repr(C)]
    struct RLimit {
        current: u64,
        maximum: u64,
    }

    unsafe extern "C" {
        fn chroot(path: *const c_char) -> c_int;
        fn close(descriptor: c_int) -> c_int;
        fn fcntl(fd: c_int, command: c_int, ...) -> c_int;
        fn getgid() -> u32;
        fn getuid() -> u32;
        pub(super) fn kill(pid: c_int, signal: c_int) -> c_int;
        fn mount(
            source: *const c_char,
            target: *const c_char,
            filesystem_type: *const c_char,
            flags: c_ulong,
            data: *const c_void,
        ) -> c_int;
        fn prctl(option: c_int, ...) -> c_int;
        fn sethostname(name: *const c_char, length: usize) -> c_int;
        fn setrlimit(resource: c_int, limit: *const RLimit) -> c_int;
        fn umask(mask: u32) -> u32;
        fn unshare(flags: c_int) -> c_int;
    }

    pub(super) fn inherit_descriptor(descriptor: c_int) -> std::io::Result<()> {
        const F_SETFD: c_int = 2;
        // SAFETY: descriptor is retained by SandboxContext and F_SETFD with value zero clears
        // only its close-on-exec flag in the post-fork child.
        if unsafe { fcntl(descriptor, F_SETFD, 0) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    pub(super) fn close_retained_path(path: &Path) -> Result<(), u8> {
        let descriptor = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse::<c_int>().ok())
            .ok_or(125)?;
        // SAFETY: the descriptor number came from the trusted retained /proc/self/fd path and
        // is closed only after all bind mounts have completed.
        if unsafe { close(descriptor) } != 0 {
            return Err(125);
        }
        Ok(())
    }

    pub(super) fn enter_namespaces() -> Result<(), u8> {
        // SAFETY: unshare receives a constant flag mask and changes only the calling process.
        if unsafe { unshare(CLONE_NEWUSER) } != 0 {
            return Err(125);
        }
        // SAFETY: getuid/getgid take no arguments and have no memory preconditions.
        let uid = unsafe { getuid() };
        let gid = unsafe { getgid() };
        fs::write("/proc/self/setgroups", b"deny\n").map_err(|_| 125)?;
        fs::write("/proc/self/uid_map", format!("0 {uid} 1\n")).map_err(|_| 125)?;
        fs::write("/proc/self/gid_map", format!("0 {gid} 1\n")).map_err(|_| 125)?;
        let flags = CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWIPC | CLONE_NEWUTS | CLONE_NEWPID;
        // SAFETY: unshare receives the closed namespace mask and changes only this process.
        if unsafe { unshare(flags) } != 0 {
            return Err(125);
        }
        mount_call(None, Path::new("/"), MS_REC | MS_PRIVATE)?;
        let hostname = b"mpk-rust";
        // SAFETY: the byte slice is valid for its supplied length.
        if unsafe { sethostname(hostname.as_ptr().cast(), hostname.len()) } != 0 {
            return Err(125);
        }
        Ok(())
    }

    pub(super) fn bind_root(root: &Path) -> Result<(), u8> {
        mount_call(Some(root), root, MS_BIND | MS_REC)
    }

    pub(super) fn bind_view(
        source: &Path,
        target: &Path,
        read_only: bool,
        executable: bool,
    ) -> Result<(), u8> {
        mount_call(Some(source), target, MS_BIND | MS_REC)?;
        let mut flags = MS_BIND | MS_REMOUNT | MS_NOSUID | MS_NODEV;
        if read_only {
            flags |= MS_RDONLY;
        }
        if !executable {
            flags |= MS_NOEXEC;
        }
        mount_call(None, target, flags)
    }

    pub(super) fn seal_root(root: &Path) -> Result<(), u8> {
        mount_call(
            None,
            root,
            MS_BIND | MS_REMOUNT | MS_RDONLY | MS_NOSUID | MS_NODEV | MS_NOEXEC,
        )
    }

    pub(super) fn enter_root(root: &Path) -> Result<(), u8> {
        let root = path_cstring(root)?;
        // SAFETY: root is a NUL-terminated absolute path prepared by the trusted parent.
        if unsafe { chroot(root.as_ptr()) } != 0 {
            return Err(125);
        }
        std::env::set_current_dir("/").map_err(|_| 125)
    }

    pub(super) fn apply_process_controls() -> Result<(), u8> {
        for (resource, value) in [
            (RLIMIT_CORE, 0),
            (RLIMIT_FSIZE, VIRTUAL_MEMORY_LIMIT),
            (RLIMIT_RSS, RESIDENT_MEMORY_LIMIT),
            (RLIMIT_NPROC, PROCESS_LIMIT),
            (RLIMIT_NOFILE, OPEN_FILE_LIMIT),
            (RLIMIT_AS, VIRTUAL_MEMORY_LIMIT),
        ] {
            let limit = RLimit {
                current: value,
                maximum: value,
            };
            // SAFETY: limit points to a fully initialized Linux rlimit structure.
            if unsafe { setrlimit(resource, &limit) } != 0 {
                return Err(125);
            }
        }
        // SAFETY: PR_SET_NO_NEW_PRIVS has the fixed integer arguments required by Linux.
        if unsafe { prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
            return Err(125);
        }
        // SAFETY: umask accepts only permission bits and has no pointer arguments.
        unsafe { umask(0o077) };
        Ok(())
    }

    fn mount_call(source: Option<&Path>, target: &Path, flags: c_ulong) -> Result<(), u8> {
        let source = source.map(path_cstring).transpose()?;
        let target = path_cstring(target)?;
        let source_pointer = source
            .as_ref()
            .map_or(std::ptr::null(), |path| path.as_ptr());
        // SAFETY: optional source and required target are live NUL-terminated strings; no
        // filesystem type or data is used for bind/remount/propagation operations.
        if unsafe {
            mount(
                source_pointer,
                target.as_ptr(),
                std::ptr::null(),
                flags,
                std::ptr::null(),
            )
        } != 0
        {
            return Err(125);
        }
        Ok(())
    }

    fn path_cstring(path: &Path) -> Result<CString, u8> {
        CString::new(path.as_os_str().as_bytes()).map_err(|_| 125)
    }
}

#[derive(Debug)]
struct InvocationWorkspace {
    invocation_id: u64,
    root: PathBuf,
    rootfs: PathBuf,
    rootfs_handle: File,
    writable: BTreeMap<&'static str, PathBuf>,
    work: PathBuf,
    work_handle: File,
    writable_handles: BTreeMap<&'static str, File>,
    driver_request: PathBuf,
    driver_request_handle: File,
    driver_request_bytes: Vec<u8>,
}

impl InvocationWorkspace {
    fn create(
        parent: &Path,
        snapshot: &Path,
        frontend: &Path,
        toolchain: &Path,
        driver_request_bytes: &[u8],
    ) -> Result<Self, SandboxError> {
        let invocation_id = NEXT_INVOCATION.fetch_add(1, Ordering::Relaxed);
        let root = parent.join(format!(
            "rust2vir-sandbox-{}-{invocation_id}",
            std::process::id()
        ));
        fs::create_dir(&root).map_err(|_| SandboxError::SandboxUnavailable)?;
        set_mode(&root, 0o700)?;
        let result = (|| {
            let rootfs = root.join("rootfs");
            let work = root.join("work");
            fs::create_dir(&rootfs).map_err(|_| SandboxError::SandboxUnavailable)?;
            fs::create_dir(&work).map_err(|_| SandboxError::SandboxUnavailable)?;
            set_mode(&rootfs, 0o700)?;
            set_mode(&work, 0o500)?;
            let mut writable = BTreeMap::new();
            for (virtual_path, name) in [
                (HOME_ROOT, "home"),
                (crate::environment::CARGO_HOME_ROOT, "cargo-home"),
                (TEMP_ROOT, "tmp"),
                (TARGET_ROOT, "target"),
                (DRIVER_OUTPUT_ROOT, "driver-output"),
            ] {
                let path = root.join(name);
                fs::create_dir(&path).map_err(|_| SandboxError::SandboxUnavailable)?;
                set_mode(&path, 0o700)?;
                writable.insert(virtual_path, path);
            }
            let (driver_request, driver_request_handle) =
                create_driver_request(&root, driver_request_bytes)?;
            create_rootfs_mountpoints(&rootfs)?;
            let rootfs_handle = open_workspace_directory(&rootfs)?;
            let work_handle = open_workspace_directory(&work)?;
            let writable_handles = writable
                .iter()
                .map(|(sandbox_path, host_path)| {
                    Ok((*sandbox_path, open_workspace_directory(host_path)?))
                })
                .collect::<Result<BTreeMap<_, _>, SandboxError>>()?;
            let workspace = Self {
                invocation_id,
                root: root.clone(),
                rootfs,
                rootfs_handle,
                writable,
                work,
                work_handle,
                writable_handles,
                driver_request,
                driver_request_handle,
                driver_request_bytes: driver_request_bytes.to_vec(),
            };
            workspace.validate(snapshot, frontend, toolchain)?;
            Ok(workspace)
        })();
        if result.is_err() {
            let _ = make_tree_private(&root);
            let _ = fs::remove_dir_all(&root);
        }
        result
    }

    fn writable_host_path(&self, sandbox_path: &str) -> Option<&Path> {
        self.writable.get(sandbox_path).map(PathBuf::as_path)
    }

    fn driver_request_host_path(&self) -> &Path {
        &self.driver_request
    }

    fn driver_output_handle(&self) -> &File {
        &self.writable_handles[DRIVER_OUTPUT_ROOT]
    }

    #[cfg(target_os = "linux")]
    fn retained_descriptors(&self) -> [i32; 7] {
        [
            self.work_handle.as_raw_fd(),
            self.writable_handles[HOME_ROOT].as_raw_fd(),
            self.writable_handles[crate::environment::CARGO_HOME_ROOT].as_raw_fd(),
            self.writable_handles[TEMP_ROOT].as_raw_fd(),
            self.writable_handles[TARGET_ROOT].as_raw_fd(),
            self.writable_handles[DRIVER_OUTPUT_ROOT].as_raw_fd(),
            self.driver_request_handle.as_raw_fd(),
        ]
    }

    fn validate(
        &self,
        snapshot: &Path,
        frontend: &Path,
        toolchain: &Path,
    ) -> Result<(), SandboxError> {
        let roots = [
            snapshot,
            frontend,
            toolchain,
            self.rootfs.as_path(),
            self.work.as_path(),
            self.writable[HOME_ROOT].as_path(),
            self.writable[crate::environment::CARGO_HOME_ROOT].as_path(),
            self.writable[TEMP_ROOT].as_path(),
            self.writable[TARGET_ROOT].as_path(),
            self.writable[DRIVER_OUTPUT_ROOT].as_path(),
        ];
        let mut identities = BTreeSet::new();
        for path in roots {
            let metadata =
                fs::symlink_metadata(path).map_err(|_| SandboxError::SandboxUnavailable)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(SandboxError::SandboxUnavailable);
            }
            let identity = directory_identity(&metadata)?;
            if !identities.insert(identity) {
                return Err(SandboxError::SandboxUnavailable);
            }
        }
        let mut retained = vec![
            (self.rootfs.as_path(), &self.rootfs_handle),
            (self.work.as_path(), &self.work_handle),
        ];
        retained.extend(self.writable.iter().map(|(sandbox_path, host_path)| {
            (host_path.as_path(), &self.writable_handles[sandbox_path])
        }));
        for (path, handle) in retained {
            let named = fs::symlink_metadata(path).map_err(|_| SandboxError::SandboxUnavailable)?;
            let opened = handle
                .metadata()
                .map_err(|_| SandboxError::SandboxUnavailable)?;
            if directory_identity(&named)? != directory_identity(&opened)? {
                return Err(SandboxError::SandboxUnavailable);
            }
        }
        if fs::read_dir(&self.work)
            .map_err(|_| SandboxError::SandboxUnavailable)?
            .next()
            .is_some()
        {
            return Err(SandboxError::SandboxUnavailable);
        }
        for path in self.writable.values() {
            let metadata =
                fs::symlink_metadata(path).map_err(|_| SandboxError::SandboxUnavailable)?;
            if metadata_mode(&metadata)? & 0o7777 != 0o700 {
                return Err(SandboxError::SandboxUnavailable);
            }
        }
        if metadata_mode(
            &self
                .work_handle
                .metadata()
                .map_err(|_| SandboxError::SandboxUnavailable)?,
        )? & 0o7777
            != 0o500
        {
            return Err(SandboxError::SandboxUnavailable);
        }
        let rootfs_metadata = self
            .rootfs_handle
            .metadata()
            .map_err(|_| SandboxError::SandboxUnavailable)?;
        if metadata_mode(&rootfs_metadata)? & 0o7777 != 0o700 {
            return Err(SandboxError::SandboxUnavailable);
        }
        validate_rootfs_scaffold(&self.rootfs)?;
        validate_driver_request(
            &self.driver_request,
            &self.driver_request_handle,
            &self.driver_request_bytes,
        )?;
        Ok(())
    }

    fn validate_usage(&self) -> Result<(), SandboxError> {
        let (home_files, _) = tree_usage(&self.writable[HOME_ROOT])?;
        let (cargo_home_files, _) =
            tree_usage(&self.writable[crate::environment::CARGO_HOME_ROOT])?;
        let (temp_files, temp_bytes) = tree_usage(&self.writable[TEMP_ROOT])?;
        let (target_files, target_bytes) = tree_usage(&self.writable[TARGET_ROOT])?;
        let (driver_files, _) = tree_usage(&self.writable[DRIVER_OUTPUT_ROOT])?;
        if temp_bytes > TEMP_BYTES_LIMIT
            || target_bytes > TARGET_BYTES_LIMIT
            || home_files
                .checked_add(cargo_home_files)
                .and_then(|count| count.checked_add(temp_files))
                .and_then(|count| count.checked_add(target_files))
                .and_then(|count| count.checked_add(driver_files))
                .is_none_or(|count| count > OUTPUT_FILES_LIMIT)
        {
            return Err(SandboxError::FilesystemLimit);
        }
        Ok(())
    }

    fn validate_before(&self, _kind: CargoInvocationKind) -> Result<(), SandboxError> {
        if tree_usage(&self.writable[TARGET_ROOT])? != (0, 0)
            || tree_usage(&self.writable[DRIVER_OUTPUT_ROOT])? != (0, 0)
        {
            return Err(SandboxError::SandboxUnavailable);
        }
        Ok(())
    }

    fn validate_after(&self, kind: CargoInvocationKind) -> Result<(), SandboxError> {
        self.validate_usage()?;
        if kind == CargoInvocationKind::Metadata
            && (tree_usage(&self.writable[TARGET_ROOT])? != (0, 0)
                || tree_usage(&self.writable[DRIVER_OUTPUT_ROOT])? != (0, 0))
        {
            return Err(SandboxError::SandboxUnavailable);
        }
        Ok(())
    }
}

impl Drop for InvocationWorkspace {
    fn drop(&mut self) {
        let _ = make_tree_private(&self.root);
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_rootfs_mountpoints(root: &Path) -> Result<(), SandboxError> {
    for path in ROOTFS_DIRECTORIES {
        fs::create_dir_all(root.join(path)).map_err(|_| SandboxError::SandboxUnavailable)?;
    }
    let interpreter = root.join(ROOTFS_INTERPRETER);
    File::create(&interpreter).map_err(|_| SandboxError::SandboxUnavailable)?;
    set_mode(&interpreter, 0o444)?;
    let request = root.join(ROOTFS_DRIVER_REQUEST);
    File::create(&request).map_err(|_| SandboxError::SandboxUnavailable)?;
    set_mode(&request, 0o444)?;
    for path in ROOTFS_DIRECTORIES {
        set_mode(&root.join(path), 0o555)?;
    }
    validate_rootfs_scaffold(root)
}

const ROOTFS_DIRECTORIES: [&str; 14] = [
    "lib",
    "lib/x86_64-linux-gnu",
    "lib64",
    "mpk",
    "mpk/cargo-home",
    "mpk/driver-output",
    "mpk/frontend",
    "mpk/home",
    "mpk/input",
    "mpk/native-runtime",
    "mpk/target",
    "mpk/tmp",
    "mpk/toolchain",
    "mpk/work",
];
const ROOTFS_INTERPRETER: &str = "lib64/ld-linux-x86-64.so.2";
const ROOTFS_DRIVER_REQUEST: &str = "mpk/driver-request.json";

fn validate_rootfs_scaffold(root: &Path) -> Result<(), SandboxError> {
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|_| SandboxError::SandboxUnavailable)? {
            let entry = entry.map_err(|_| SandboxError::SandboxUnavailable)?;
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| SandboxError::SandboxUnavailable)?;
            if metadata.file_type().is_symlink() {
                return Err(SandboxError::SandboxUnavailable);
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| SandboxError::SandboxUnavailable)?
                .to_str()
                .ok_or(SandboxError::SandboxUnavailable)?
                .to_owned();
            if metadata.is_dir() {
                if metadata_mode(&metadata)? & 0o7777 != 0o555 || !directories.insert(relative) {
                    return Err(SandboxError::SandboxUnavailable);
                }
                pending.push(path);
            } else if metadata.is_file() {
                if metadata_mode(&metadata)? & 0o7777 != 0o444
                    || metadata.len() != 0
                    || metadata_link_count(&metadata)? != 1
                    || !files.insert(relative)
                {
                    return Err(SandboxError::SandboxUnavailable);
                }
            } else {
                return Err(SandboxError::SandboxUnavailable);
            }
        }
    }
    let expected_directories = ROOTFS_DIRECTORIES.into_iter().map(str::to_owned).collect();
    let expected_files = [
        ROOTFS_INTERPRETER.to_owned(),
        ROOTFS_DRIVER_REQUEST.to_owned(),
    ]
    .into_iter()
    .collect();
    if directories != expected_directories || files != expected_files {
        return Err(SandboxError::SandboxUnavailable);
    }
    Ok(())
}

fn create_driver_request(root: &Path, bytes: &[u8]) -> Result<(PathBuf, File), SandboxError> {
    if bytes.len() > driver_protocol::REQUEST_TRANSPORT_MAX {
        return Err(SandboxError::SandboxUnavailable);
    }
    let path = root.join("driver-request.json");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(target_os = "linux")]
    options.mode(0o600).custom_flags(0o400_000 | 0o2_000_000);
    let mut writer = options
        .open(&path)
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    writer
        .write_all(bytes)
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    writer
        .flush()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    writer
        .sync_all()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    set_mode(&path, 0o400)?;
    writer
        .sync_all()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    drop(writer);
    File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    let handle = open_workspace_regular(&path)?;
    validate_driver_request(&path, &handle, bytes)?;
    Ok((path, handle))
}

fn validate_driver_request(
    path: &Path,
    handle: &File,
    expected: &[u8],
) -> Result<(), SandboxError> {
    let named = fs::symlink_metadata(path).map_err(|_| SandboxError::SandboxUnavailable)?;
    let opened = handle
        .metadata()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    if !named.is_file()
        || named.file_type().is_symlink()
        || metadata_mode(&named)? & 0o7777 != 0o400
        || metadata_link_count(&named)? != 1
        || regular_file_identity(&named)? != regular_file_identity(&opened)?
        || named.len() != expected.len() as u64
    {
        return Err(SandboxError::SandboxUnavailable);
    }
    let mut reader = handle
        .try_clone()
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    let mut bytes = Vec::with_capacity(expected.len());
    reader
        .take(expected.len().saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| SandboxError::SandboxUnavailable)?;
    if bytes != expected
        || regular_file_identity(
            &fs::symlink_metadata(path).map_err(|_| SandboxError::SandboxUnavailable)?,
        )? != regular_file_identity(
            &handle
                .metadata()
                .map_err(|_| SandboxError::SandboxUnavailable)?,
        )?
    {
        return Err(SandboxError::SandboxUnavailable);
    }
    Ok(())
}

fn required_file<'a>(
    inventory: &'a [InventoryFile],
    path: &str,
    executable: bool,
) -> Result<&'a InventoryFile, SandboxError> {
    let file = inventory
        .binary_search_by_key(&path, |file| file.path.as_str())
        .ok()
        .map(|index| &inventory[index])
        .ok_or(SandboxError::ToolchainComponent)?;
    if file.executable != executable {
        return Err(SandboxError::ToolchainComponent);
    }
    Ok(file)
}

fn validate_inventory_shape(inventory: &[InventoryFile]) -> Result<(), SandboxError> {
    if inventory.is_empty() || inventory.len() > CANDIDATE_FILE_COUNT_LIMIT {
        return Err(SandboxError::ToolchainComponent);
    }
    let mut previous: Option<&str> = None;
    let mut casefolded = BTreeSet::new();
    let mut aggregate = 0_u64;
    for file in inventory {
        file.validate_shape()?;
        aggregate = aggregate
            .checked_add(file.size_bytes)
            .ok_or(SandboxError::ToolchainComponent)?;
        if file.size_bytes > CANDIDATE_FILE_SIZE_LIMIT
            || aggregate > CANDIDATE_AGGREGATE_LIMIT
            || previous.is_some_and(|path| path >= file.path.as_str())
            || !casefolded.insert(file.path.to_ascii_lowercase())
        {
            return Err(SandboxError::ToolchainComponent);
        }
        previous = Some(&file.path);
    }
    Ok(())
}

fn validate_inventory_root(root: &Path, inventory: &[InventoryFile]) -> Result<File, SandboxError> {
    let root_metadata = fs::symlink_metadata(root).map_err(|_| SandboxError::ToolchainComponent)?;
    if !root_metadata.is_dir()
        || root_metadata.file_type().is_symlink()
        || metadata_mode(&root_metadata)? & 0o7777 != 0o555
    {
        return Err(SandboxError::ToolchainComponent);
    }
    let handle = open_directory_nofollow(root)?;
    let retained_metadata = handle
        .metadata()
        .map_err(|_| SandboxError::ToolchainComponent)?;
    if directory_identity(&root_metadata)? != directory_identity(&retained_metadata)? {
        return Err(SandboxError::ToolchainComponent);
    }

    #[cfg(target_os = "linux")]
    let retained_root = PathBuf::from(format!("/proc/self/fd/{}", handle.as_raw_fd()));
    #[cfg(not(target_os = "linux"))]
    let retained_root = root.to_path_buf();

    let mut observed = Vec::new();
    collect_regular_files(
        &retained_root,
        &retained_root,
        metadata_device(&retained_metadata)?,
        &mut observed,
    )?;
    observed.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    if observed.len() != inventory.len()
        || observed
            .iter()
            .map(String::as_str)
            .ne(inventory.iter().map(|file| file.path.as_str()))
    {
        return Err(SandboxError::ToolchainComponent);
    }
    for expected in inventory {
        let path = retained_root.join(&expected.path);
        let metadata = fs::symlink_metadata(&path).map_err(|_| SandboxError::ToolchainComponent)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() != expected.size_bytes
            || metadata_link_count(&metadata)? != 1
            || metadata_mode(&metadata)? & 0o7777 != if expected.executable { 0o555 } else { 0o444 }
        {
            return Err(SandboxError::ToolchainComponent);
        }
        let identity = regular_file_identity(&metadata)?;
        let file = open_regular_nofollow(&path)?;
        if regular_file_identity(
            &file
                .metadata()
                .map_err(|_| SandboxError::ToolchainComponent)?,
        )? != identity
            || regular_file_identity(
                &fs::symlink_metadata(&path).map_err(|_| SandboxError::ToolchainComponent)?,
            )? != identity
        {
            return Err(SandboxError::ToolchainComponent);
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(expected.size_bytes).map_err(|_| SandboxError::ToolchainComponent)?,
        );
        file.take(expected.size_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| SandboxError::ToolchainComponent)?;
        if bytes.len() as u64 != expected.size_bytes || hex(&digest(&bytes)) != expected.sha256 {
            return Err(SandboxError::ToolchainComponent);
        }
    }
    Ok(handle)
}

fn collect_regular_files(
    root: &Path,
    directory: &Path,
    root_device: u64,
    output: &mut Vec<String>,
) -> Result<(), SandboxError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|_| SandboxError::ToolchainComponent)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SandboxError::ToolchainComponent)?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|_| SandboxError::ToolchainComponent)?;
        if metadata.file_type().is_symlink() || metadata_device(&metadata)? != root_device {
            return Err(SandboxError::ToolchainComponent);
        }
        if metadata.is_dir() {
            if metadata_mode(&metadata)? & 0o7777 != 0o555 {
                return Err(SandboxError::ToolchainComponent);
            }
            let file_count = output.len();
            collect_regular_files(root, &path, root_device, output)?;
            if output.len() == file_count {
                return Err(SandboxError::ToolchainComponent);
            }
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| SandboxError::ToolchainComponent)?
                .to_str()
                .ok_or(SandboxError::ToolchainComponent)?
                .to_owned();
            validate_relative_path(&relative)?;
            output.push(relative);
        } else {
            return Err(SandboxError::ToolchainComponent);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_regular_nofollow(path: &Path) -> Result<File, SandboxError> {
    const O_NOFOLLOW: i32 = 0o400_000;
    const O_CLOEXEC: i32 = 0o2_000_000;
    OpenOptions::new()
        .read(true)
        .custom_flags(O_NOFOLLOW | O_CLOEXEC)
        .open(path)
        .map_err(|_| SandboxError::ToolchainComponent)
}

#[cfg(not(target_os = "linux"))]
fn open_regular_nofollow(path: &Path) -> Result<File, SandboxError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| SandboxError::ToolchainComponent)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(SandboxError::ToolchainComponent);
    }
    File::open(path).map_err(|_| SandboxError::ToolchainComponent)
}

fn validate_distinct_roots(left: &File, right: &File) -> Result<(), SandboxError> {
    let left = left
        .metadata()
        .map_err(|_| SandboxError::ToolchainComponent)?;
    let right = right
        .metadata()
        .map_err(|_| SandboxError::ToolchainComponent)?;
    if directory_identity(&left)? == directory_identity(&right)? {
        return Err(SandboxError::ToolchainComponent);
    }
    Ok(())
}

fn open_workspace_directory(path: &Path) -> Result<File, SandboxError> {
    open_directory_nofollow(path).map_err(|_| SandboxError::SandboxUnavailable)
}

fn open_workspace_regular(path: &Path) -> Result<File, SandboxError> {
    open_regular_nofollow(path).map_err(|_| SandboxError::SandboxUnavailable)
}

#[cfg(target_os = "linux")]
fn open_directory_nofollow(path: &Path) -> Result<File, SandboxError> {
    const O_DIRECTORY: i32 = 0o200_000;
    const O_NOFOLLOW: i32 = 0o400_000;
    const O_CLOEXEC: i32 = 0o2_000_000;
    OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC)
        .open(path)
        .map_err(|_| SandboxError::ToolchainComponent)
}

#[cfg(not(target_os = "linux"))]
fn open_directory_nofollow(path: &Path) -> Result<File, SandboxError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| SandboxError::ToolchainComponent)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(SandboxError::ToolchainComponent);
    }
    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|_| SandboxError::ToolchainComponent)
}

fn validate_relative_path(path: &str) -> Result<(), SandboxError> {
    let candidate = Path::new(path);
    if path.is_empty()
        || path.len() > CANDIDATE_PATH_LIMIT
        || path.contains(['\\', '\0'])
        || !path.is_ascii()
        || !candidate
            .components()
            .all(|component| matches!(component, Component::Normal(value) if !value.is_empty()))
    {
        return Err(SandboxError::ToolchainComponent);
    }
    Ok(())
}

fn validate_sha256(value: &str) -> Result<(), SandboxError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SandboxError::ToolchainComponent);
    }
    Ok(())
}

pub fn toolchain_distribution_sha256(bundle_id: &str, inventory: &[InventoryFile]) -> String {
    inventory_content_sha256(bundle_id, "toolchain_bundle", None, inventory)
}

pub fn frontend_bundle_sha256(bundle_id: &str, inventory: &[InventoryFile]) -> String {
    inventory_content_sha256(bundle_id, "frontend_bundle", None, inventory)
}

pub fn component_content_sha256(
    bundle_id: &str,
    component_name: &str,
    inventory: &[InventoryFile],
    path_prefix: &str,
) -> String {
    let files = inventory
        .iter()
        .filter(|file| file.path.starts_with(path_prefix))
        .cloned()
        .collect::<Vec<_>>();
    inventory_content_sha256(bundle_id, "component", Some(component_name), &files)
}

fn inventory_content_sha256(
    bundle_id: &str,
    scope_kind: &str,
    component_name: Option<&str>,
    inventory: &[InventoryFile],
) -> String {
    let mut payload = String::from("{\"files\":[");
    for (index, file) in inventory.iter().enumerate() {
        if index != 0 {
            payload.push(',');
        }
        payload.push_str("{\"executable\":");
        payload.push_str(if file.executable { "true" } else { "false" });
        payload.push_str(",\"path\":");
        push_json_string(&mut payload, &file.path);
        payload.push_str(",\"sha256\":");
        push_json_string(&mut payload, &file.sha256);
        payload.push_str(",\"size_bytes\":");
        payload.push_str(&file.size_bytes.to_string());
        payload.push('}');
    }
    payload.push_str("],\"schema\":\"mpk.release.bundle_inventory.v0\",\"scope\":{");
    payload.push_str("\"bundle_id\":");
    push_json_string(&mut payload, bundle_id);
    if let Some(component_name) = component_name {
        payload.push_str(",\"component_name\":");
        push_json_string(&mut payload, component_name);
    }
    payload.push_str(",\"kind\":");
    push_json_string(&mut payload, scope_kind);
    payload.push('}');
    payload.push('}');
    let mut preimage = b"MPK-BUNDLE-CONTENT-0.1\0".to_vec();
    preimage.extend_from_slice(payload.as_bytes());
    hex(&digest(&preimage))
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{8}' => output.push_str("\\b"),
            '\u{c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0}'..='\u{1f}' => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", u32::from(character));
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn tree_usage(root: &Path) -> Result<(u64, u64), SandboxError> {
    let mut files = 0_u64;
    let mut bytes = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|_| SandboxError::FilesystemLimit)? {
            let entry = entry.map_err(|_| SandboxError::FilesystemLimit)?;
            let metadata =
                fs::symlink_metadata(entry.path()).map_err(|_| SandboxError::FilesystemLimit)?;
            if metadata.file_type().is_symlink() {
                return Err(SandboxError::FilesystemLimit);
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                files = files.checked_add(1).ok_or(SandboxError::FilesystemLimit)?;
                bytes = bytes
                    .checked_add(metadata.len())
                    .ok_or(SandboxError::FilesystemLimit)?;
            } else {
                return Err(SandboxError::FilesystemLimit);
            }
        }
    }
    Ok((files, bytes))
}

fn make_tree_private(root: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    if metadata.is_dir() {
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        for entry in fs::read_dir(root)? {
            make_tree_private(&entry?.path())?;
        }
    } else if metadata.is_file() {
        fs::set_permissions(root, fs::Permissions::from_mode(0o600))?;
    } else {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), SandboxError> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|_| SandboxError::SandboxUnavailable)
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

#[cfg(unix)]
fn directory_identity(metadata: &Metadata) -> Result<(u64, u64), SandboxError> {
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn directory_identity(_metadata: &Metadata) -> Result<(u64, u64), SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

#[cfg(unix)]
fn metadata_device(metadata: &Metadata) -> Result<u64, SandboxError> {
    Ok(metadata.dev())
}

#[cfg(not(unix))]
fn metadata_device(_metadata: &Metadata) -> Result<u64, SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RegularFileIdentity {
    device: u64,
    inode: u64,
    mode: u32,
    links: u64,
    length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[cfg(unix)]
fn regular_file_identity(metadata: &Metadata) -> Result<RegularFileIdentity, SandboxError> {
    Ok(RegularFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.mode(),
        links: metadata.nlink(),
        length: metadata.len(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    })
}

#[cfg(not(unix))]
fn regular_file_identity(_metadata: &Metadata) -> Result<RegularFileIdentity, SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

#[cfg(unix)]
fn metadata_mode(metadata: &Metadata) -> Result<u32, SandboxError> {
    Ok(metadata.mode())
}

#[cfg(not(unix))]
fn metadata_mode(_metadata: &Metadata) -> Result<u32, SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

#[cfg(unix)]
fn metadata_link_count(metadata: &Metadata) -> Result<u64, SandboxError> {
    Ok(metadata.nlink())
}

#[cfg(not(unix))]
fn metadata_link_count(_metadata: &Metadata) -> Result<u64, SandboxError> {
    Err(SandboxError::SandboxUnavailable)
}

pub fn fixed_read_only_views() -> [&'static str; 6] {
    [
        INPUT_ROOT,
        TOOLCHAIN_ROOT,
        FRONTEND_ROOT,
        WORK_ROOT,
        NATIVE_RUNTIME_ROOT,
        driver_protocol::REQUEST_PATH,
    ]
}

pub fn fixed_writable_views() -> [&'static str; 5] {
    [
        HOME_ROOT,
        crate::environment::CARGO_HOME_ROOT,
        TEMP_ROOT,
        TARGET_ROOT,
        DRIVER_OUTPUT_ROOT,
    ]
}
