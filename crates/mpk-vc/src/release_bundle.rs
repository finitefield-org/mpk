//! Side-effect-free release-registry validation and tuple resolution.
//!
//! Filesystem discovery, installed-tree snapshots, and process execution belong
//! to the outer runner. This module accepts only bounded registry bytes and
//! returns immutable validated descriptors.

use crate::{
    canonical_json_bytes, hash_canonical_inventory, hash_canonical_json, parse_strict_json,
    HashDomain, Sha256Digest, StrictJsonError, StrictJsonLimits, StrictJsonValue,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const RELEASE_REGISTRY_SCHEMA: &str = "mpk.release.bundle_registry.v0";
pub const RELEASE_REGISTRY_ID: &str = "mpk.release.registry.v0";
pub const BUNDLE_INVENTORY_SCHEMA: &str = "mpk.release.bundle_inventory.v0";
pub const FRONTEND_BUNDLE_SCHEMA: &str = "mpk.release.frontend_bundle.v0";
pub const TOOLCHAIN_BUNDLE_SCHEMA: &str = "mpk.release.toolchain_bundle.v0";

pub const BUNDLE_REGISTRY_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-BUNDLE-REGISTRY-0.1");
pub const BUNDLE_CONTENT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-BUNDLE-CONTENT-0.1");

pub const REGISTRY_CANONICAL_BYTES_MAX: u64 = 67_108_864;
pub const REGISTRY_TRANSPORT_BYTES_MAX: u64 = 67_108_865;
pub const RELEASE_JSON_NESTING_MAX: u64 = 256;
pub const RELEASE_STRING_BYTES_MAX: u64 = 1_048_576;
pub const BUNDLE_DESCRIPTORS_MAX: u64 = 1_024;
pub const RELEASE_TUPLES_MAX: u64 = 4_096;
pub const EXECUTION_HOST_PROFILES_MAX: u64 = 256;
pub const NATIVE_RUNTIME_LAYOUT_PROFILES_MAX: u64 = 256;
pub const TOOLCHAIN_COMPONENTS_MAX: u64 = 8_192;
pub const SERIALIZED_INVENTORY_ENTRIES_MAX: u64 = 262_144;
pub const UNIQUE_BUNDLE_FILES_MAX: u64 = 262_144;
pub const PORTABLE_PATH_BYTES_MAX: u64 = 1_024;
pub const BUNDLE_FILE_BYTES_MAX: u64 = 4_294_967_296;
pub const BUNDLE_DECLARED_BYTES_MAX: u64 = 34_359_738_368;

const REGISTRY_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    REGISTRY_TRANSPORT_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX,
    RELEASE_JSON_NESTING_MAX,
    RELEASE_STRING_BYTES_MAX,
);

const REQUIRED_PRIMITIVES: [&str; 10] = [
    "filesystem.atomic_no_replace",
    "filesystem.immutable_handle",
    "filesystem.no_follow_open",
    "isolation.mount_namespace",
    "isolation.network_namespace",
    "isolation.user_namespace",
    "mount.no_exec",
    "mount.read_only",
    "process.closed_environment",
    "process.no_new_privileges",
];

const FORBIDDEN_HOST_ROOTS: [&str; 3] = ["/lib", "/lib64", "/usr/lib"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseValidationPhase {
    Transport,
    Shape,
    Scalar,
    Order,
    Invariant,
    ContentHash,
    RegistryHash,
    CanonicalTransport,
}

impl ReleaseValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Order => "order",
            Self::Invariant => "invariant",
            Self::ContentHash => "content_hash",
            Self::RegistryHash => "registry_hash",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseRegistryErrorCode {
    Invalid,
    Limit,
}

impl ReleaseRegistryErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "FRONTEND_REGISTRY_INVALID",
            Self::Limit => "FRONTEND_REGISTRY_LIMIT",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseRegistryError {
    phase: ReleaseValidationPhase,
    code: ReleaseRegistryErrorCode,
    detail: String,
}

impl ReleaseRegistryError {
    pub const fn phase(&self) -> ReleaseValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> ReleaseRegistryErrorCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for ReleaseRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} ({}): {}",
            self.code.as_str(),
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for ReleaseRegistryError {}

fn invalid(phase: ReleaseValidationPhase, detail: impl Into<String>) -> ReleaseRegistryError {
    ReleaseRegistryError {
        phase,
        code: ReleaseRegistryErrorCode::Invalid,
        detail: detail.into(),
    }
}

fn limit(phase: ReleaseValidationPhase, detail: impl Into<String>) -> ReleaseRegistryError {
    ReleaseRegistryError {
        phase,
        code: ReleaseRegistryErrorCode::Limit,
        detail: detail.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleRegistry {
    pub schema: String,
    pub id: String,
    pub execution_host_profiles: Vec<ExecutionHostProfile>,
    pub native_runtime_layout_profiles: Vec<NativeRuntimeLayoutProfile>,
    pub frontend_bundles: Vec<FrontendBundle>,
    pub toolchain_bundles: Vec<ToolchainBundle>,
    pub tuples: Vec<ReleaseTuple>,
    pub registry_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleInventory {
    pub schema: String,
    pub scope: InventoryScope,
    pub files: Vec<InventoryFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InventoryScope {
    FrontendBundle {
        bundle_id: String,
    },
    ToolchainBundle {
        bundle_id: String,
    },
    Component {
        bundle_id: String,
        component_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryFile {
    pub path: String,
    pub executable: bool,
    pub size_bytes: i64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendBundle {
    pub schema: String,
    pub bundle_id: String,
    pub source_language: String,
    pub name: String,
    pub version: String,
    pub limit_profile_id: String,
    pub environment_profile_id: String,
    pub argument_profile_id: String,
    pub main: ExecutableRecord,
    pub subordinate_binaries: Vec<ExecutableRecord>,
    pub inventory: BundleInventory,
    pub bundle_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutableRecord {
    pub name: String,
    pub version: String,
    pub path: String,
    pub binary_sha256: String,
    pub runtime: ExecutableRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutableRuntime {
    Static,
    Dynamic {
        interpreter_mount: String,
        libraries: Vec<RuntimeLibrary>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLibrary {
    pub soname: String,
    pub component_path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainBundle {
    pub schema: String,
    pub bundle_id: String,
    pub source_language: String,
    pub compiler: CompilerIdentity,
    pub execution_host_profile_id: String,
    pub native_runtime: NativeRuntimeSelection,
    pub components: Vec<ToolchainComponent>,
    pub target_libraries: Vec<TargetLibrary>,
    pub inventory: BundleInventory,
    pub distribution_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CompilerIdentity {
    Go {
        release: String,
    },
    Rust {
        release: String,
        rustc_commit: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ToolchainComponent {
    Executable {
        name: String,
        release: String,
        path: String,
        binary_sha256: String,
        runtime: ExecutableRuntime,
    },
    Content {
        name: String,
        release: String,
        inventory: BundleInventory,
        content_sha256: String,
    },
}

impl ToolchainComponent {
    pub fn name(&self) -> &str {
        match self {
            Self::Executable { name, .. } | Self::Content { name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLibrary {
    pub target_id: String,
    pub pointer_width: i64,
    pub component_name: String,
    pub content_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeRuntimeSelection {
    None,
    Component {
        component_name: String,
        component_root: String,
        layout_profile_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionHostProfile {
    pub id: String,
    pub os: String,
    pub architecture: String,
    pub abi: String,
    pub minimum_kernel_abi: String,
    pub probe_profile_id: String,
    pub required_primitives: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRuntimeLayoutProfile {
    pub id: String,
    pub execution_host_profile_id: String,
    pub runtime_root: String,
    pub interpreter_mounts: Vec<InterpreterMount>,
    pub library_mounts: Vec<LibraryMount>,
    pub loader_search_paths: Vec<String>,
    pub forbidden_host_roots: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpreterMount {
    pub component_path: String,
    pub sandbox_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryMount {
    pub component_path: String,
    pub sandbox_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseTuple {
    pub source_language: String,
    pub semantic_profile: String,
    pub target_id: String,
    pub pointer_width: i64,
    pub limit_profile_id: String,
    pub frontend_bundle_id: String,
    pub toolchain_bundle_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedReleaseRegistry {
    registry: BundleRegistry,
    registry_digest: Sha256Digest,
}

impl ValidatedReleaseRegistry {
    pub fn registry(&self) -> &BundleRegistry {
        &self.registry
    }

    pub fn registry_digest(&self) -> Sha256Digest {
        self.registry_digest
    }

    pub fn frontend_bundle(&self, bundle_id: &str) -> Option<&FrontendBundle> {
        self.registry
            .frontend_bundles
            .binary_search_by(|bundle| bundle.bundle_id.as_str().cmp(bundle_id))
            .ok()
            .map(|index| &self.registry.frontend_bundles[index])
    }

    pub fn toolchain_bundle(&self, bundle_id: &str) -> Option<&ToolchainBundle> {
        self.registry
            .toolchain_bundles
            .binary_search_by(|bundle| bundle.bundle_id.as_str().cmp(bundle_id))
            .ok()
            .map(|index| &self.registry.toolchain_bundles[index])
    }

    pub fn resolve(
        &self,
        request: &ReleaseSelectionRequest,
    ) -> Result<ResolvedRelease<'_>, ReleaseSelectionError> {
        if request.registry_id != self.registry.id
            || decode_sha256(&request.registry_sha256).ok()
                != Some(*self.registry_digest.as_bytes())
        {
            return Err(ReleaseSelectionError::RegistryAssertion);
        }

        let Some(frontend_id) = request.frontend_bundle_id.as_deref() else {
            return Err(ReleaseSelectionError::BundleUnknown);
        };
        let Some(toolchain_id) = request.toolchain_bundle_id.as_deref() else {
            return Err(ReleaseSelectionError::BundleUnknown);
        };
        let Some(frontend) = self.frontend_bundle(frontend_id) else {
            return Err(ReleaseSelectionError::BundleUnknown);
        };
        let Some(toolchain) = self.toolchain_bundle(toolchain_id) else {
            return Err(ReleaseSelectionError::BundleUnknown);
        };

        let tuple = self.registry.tuples.iter().find(|tuple| {
            tuple.source_language == request.source_language
                && tuple.semantic_profile == request.semantic_profile
                && tuple.target_id == request.target_id
                && tuple.frontend_bundle_id == frontend_id
                && tuple.toolchain_bundle_id == toolchain_id
        });
        let Some(release_tuple) = tuple else {
            return Err(ReleaseSelectionError::BundleIncompatible);
        };
        Ok(ResolvedRelease {
            release_tuple,
            frontend,
            toolchain,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseSelectionRequest {
    pub registry_id: String,
    pub registry_sha256: String,
    pub source_language: String,
    pub semantic_profile: String,
    pub target_id: String,
    pub frontend_bundle_id: Option<String>,
    pub toolchain_bundle_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseSelectionError {
    RegistryAssertion,
    BundleUnknown,
    BundleIncompatible,
}

impl ReleaseSelectionError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::RegistryAssertion => "FRONTEND_REGISTRY_ASSERTION",
            Self::BundleUnknown => "FRONTEND_BUNDLE_UNKNOWN",
            Self::BundleIncompatible => "FRONTEND_BUNDLE_INCOMPATIBLE",
        }
    }
}

impl fmt::Display for ReleaseSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl Error for ReleaseSelectionError {}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedRelease<'a> {
    pub release_tuple: &'a ReleaseTuple,
    pub frontend: &'a FrontendBundle,
    pub toolchain: &'a ToolchainBundle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryBuildConstants {
    pub id: String,
    pub registry_sha256: [u8; 32],
}

/// Validates the tracked registry and returns only the two values a build
/// script is permitted to embed. Registry bytes and paths are intentionally
/// absent from the result.
pub fn registry_build_constants(
    input: &[u8],
) -> Result<RegistryBuildConstants, ReleaseRegistryError> {
    let validated = validate_release_registry(input)?;
    Ok(RegistryBuildConstants {
        id: validated.registry.id.clone(),
        registry_sha256: *validated.registry_digest.as_bytes(),
    })
}

/// Validates one exact JCS-plus-LF release registry without filesystem access.
pub fn validate_release_registry(
    input: &[u8],
) -> Result<ValidatedReleaseRegistry, ReleaseRegistryError> {
    let value = parse_registry_transport(input)?;
    let canonical = canonical_json_bytes(&value)
        .map_err(|error| invalid(ReleaseValidationPhase::Transport, error.to_string()))?;
    let registry: BundleRegistry = serde_json::from_slice(&canonical)
        .map_err(|error| invalid(ReleaseValidationPhase::Shape, error.to_string()))?;
    validate_shape(&registry)?;
    validate_scalar(&registry)?;
    validate_order_and_references(&registry)?;
    validate_invariants(&registry)?;
    validate_content_hashes(&registry, &value)?;
    if u64::try_from(canonical.len()).unwrap_or(u64::MAX) > REGISTRY_CANONICAL_BYTES_MAX {
        return Err(limit(
            ReleaseValidationPhase::RegistryHash,
            "registry canonical byte limit exceeded",
        ));
    }
    let registry_digest = validate_registry_hash(&registry, &value)?;

    let mut expected_transport = canonical;
    expected_transport.push(b'\n');
    if input != expected_transport {
        return Err(invalid(
            ReleaseValidationPhase::CanonicalTransport,
            "registry transport is not exact JCS plus one LF",
        ));
    }

    Ok(ValidatedReleaseRegistry {
        registry,
        registry_digest,
    })
}

fn parse_registry_transport(input: &[u8]) -> Result<StrictJsonValue, ReleaseRegistryError> {
    parse_strict_json(input, REGISTRY_LIMITS).map_err(|error| {
        let code = match error {
            StrictJsonError::InputBytesExceeded { .. }
            | StrictJsonError::NodeLimitExceeded { .. }
            | StrictJsonError::UnsupportedDepthLimit { .. }
            | StrictJsonError::DepthLimitExceeded { .. }
            | StrictJsonError::StringBytesExceeded { .. } => ReleaseRegistryErrorCode::Limit,
            _ => ReleaseRegistryErrorCode::Invalid,
        };
        ReleaseRegistryError {
            phase: ReleaseValidationPhase::Transport,
            code,
            detail: error.to_string(),
        }
    })
}

fn validate_shape(registry: &BundleRegistry) -> Result<(), ReleaseRegistryError> {
    shape_eq(&registry.schema, RELEASE_REGISTRY_SCHEMA, "registry schema")?;
    for frontend in &registry.frontend_bundles {
        shape_eq(&frontend.schema, FRONTEND_BUNDLE_SCHEMA, "frontend schema")?;
        validate_inventory_shape(&frontend.inventory)?;
    }
    for toolchain in &registry.toolchain_bundles {
        shape_eq(
            &toolchain.schema,
            TOOLCHAIN_BUNDLE_SCHEMA,
            "toolchain schema",
        )?;
        validate_inventory_shape(&toolchain.inventory)?;
        for component in &toolchain.components {
            if let ToolchainComponent::Content { inventory, .. } = component {
                validate_inventory_shape(inventory)?;
            }
        }
    }
    Ok(())
}

fn validate_inventory_shape(inventory: &BundleInventory) -> Result<(), ReleaseRegistryError> {
    shape_eq(
        &inventory.schema,
        BUNDLE_INVENTORY_SCHEMA,
        "inventory schema",
    )
}

fn shape_eq(actual: &str, expected: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(
            ReleaseValidationPhase::Shape,
            format!("{label} must be {expected:?}"),
        ))
    }
}

fn validate_scalar(registry: &BundleRegistry) -> Result<(), ReleaseRegistryError> {
    scalar_require(registry.id == RELEASE_REGISTRY_ID, "invalid registry ID")?;
    validate_sha256(&registry.registry_sha256, "registry_sha256")?;

    check_count(
        u64_len(registry.frontend_bundles.len())?
            .checked_add(u64_len(registry.toolchain_bundles.len())?)
            .ok_or_else(|| scalar_limit("bundle descriptor count overflow"))?,
        BUNDLE_DESCRIPTORS_MAX,
        "bundle descriptors",
    )?;
    check_count(
        u64_len(registry.tuples.len())?,
        RELEASE_TUPLES_MAX,
        "release tuples",
    )?;
    check_count(
        u64_len(registry.execution_host_profiles.len())?,
        EXECUTION_HOST_PROFILES_MAX,
        "execution host profiles",
    )?;
    check_count(
        u64_len(registry.native_runtime_layout_profiles.len())?,
        NATIVE_RUNTIME_LAYOUT_PROFILES_MAX,
        "native runtime layout profiles",
    )?;

    for profile in &registry.execution_host_profiles {
        validate_profile_id(&profile.id, "execution host profile ID")?;
        scalar_require(profile.os == "linux", "host OS must be linux")?;
        scalar_require(
            profile.architecture == "x86_64",
            "host architecture must be x86_64",
        )?;
        scalar_require(profile.abi == "gnu", "host ABI must be gnu")?;
        validate_kernel_abi(&profile.minimum_kernel_abi)?;
        scalar_require(
            profile.probe_profile_id == "mpk.release.probe.linux_namespaces.v0",
            "invalid host probe profile",
        )?;
    }
    for profile in &registry.native_runtime_layout_profiles {
        validate_profile_id(&profile.id, "runtime layout profile ID")?;
        validate_profile_id(
            &profile.execution_host_profile_id,
            "runtime host profile reference",
        )?;
        validate_absolute_path(&profile.runtime_root, "runtime_root")?;
        scalar_require(
            profile.runtime_root == "/mpk/native-runtime",
            "runtime_root must be /mpk/native-runtime",
        )?;
        scalar_require(
            !profile.interpreter_mounts.is_empty(),
            "interpreter_mounts must be nonempty",
        )?;
        scalar_require(
            !profile.library_mounts.is_empty(),
            "library_mounts must be nonempty",
        )?;
        scalar_require(
            !profile.loader_search_paths.is_empty(),
            "loader_search_paths must be nonempty",
        )?;
        for mount in &profile.interpreter_mounts {
            validate_portable_path(&mount.component_path, "interpreter component path")?;
            validate_absolute_path(&mount.sandbox_path, "interpreter sandbox path")?;
        }
        for mount in &profile.library_mounts {
            validate_portable_path(&mount.component_path, "library component path")?;
            validate_absolute_path(&mount.sandbox_path, "library sandbox path")?;
        }
        for path in &profile.loader_search_paths {
            validate_absolute_path(path, "loader search path")?;
        }
        for path in &profile.forbidden_host_roots {
            validate_absolute_path(path, "forbidden host root")?;
        }
    }

    let mut component_count = 0_u64;
    let mut serialized_entries = 0_u64;
    let mut unique_root_files = 0_u64;
    for frontend in &registry.frontend_bundles {
        validate_frontend_scalar(frontend)?;
        add_inventory_counts(
            &frontend.inventory,
            &mut serialized_entries,
            &mut unique_root_files,
        )?;
    }
    for toolchain in &registry.toolchain_bundles {
        validate_toolchain_scalar(toolchain)?;
        component_count = component_count
            .checked_add(u64_len(toolchain.components.len())?)
            .ok_or_else(|| scalar_limit("component count overflow"))?;
        add_inventory_counts(
            &toolchain.inventory,
            &mut serialized_entries,
            &mut unique_root_files,
        )?;
        for component in &toolchain.components {
            if let ToolchainComponent::Content { inventory, .. } = component {
                serialized_entries = serialized_entries
                    .checked_add(u64_len(inventory.files.len())?)
                    .ok_or_else(|| scalar_limit("serialized inventory count overflow"))?;
            }
        }
    }
    check_count(component_count, TOOLCHAIN_COMPONENTS_MAX, "components")?;
    check_count(
        serialized_entries,
        SERIALIZED_INVENTORY_ENTRIES_MAX,
        "serialized inventory entries",
    )?;
    check_count(
        unique_root_files,
        UNIQUE_BUNDLE_FILES_MAX,
        "unique bundle files",
    )?;

    for tuple in &registry.tuples {
        validate_language(&tuple.source_language)?;
        validate_profile_id(&tuple.semantic_profile, "semantic profile")?;
        validate_target(&tuple.source_language, &tuple.target_id)?;
        validate_pointer_width(tuple.pointer_width)?;
        validate_profile_id(&tuple.limit_profile_id, "tuple limit profile")?;
        validate_bundle_id(&tuple.frontend_bundle_id, "tuple frontend bundle ID")?;
        validate_bundle_id(&tuple.toolchain_bundle_id, "tuple toolchain bundle ID")?;
    }
    Ok(())
}

fn add_inventory_counts(
    inventory: &BundleInventory,
    serialized: &mut u64,
    unique: &mut u64,
) -> Result<(), ReleaseRegistryError> {
    let count = u64_len(inventory.files.len())?;
    *serialized = serialized
        .checked_add(count)
        .ok_or_else(|| scalar_limit("serialized inventory count overflow"))?;
    *unique = unique
        .checked_add(count)
        .ok_or_else(|| scalar_limit("root inventory count overflow"))?;
    Ok(())
}

fn validate_frontend_scalar(frontend: &FrontendBundle) -> Result<(), ReleaseRegistryError> {
    validate_bundle_id(&frontend.bundle_id, "frontend bundle ID")?;
    validate_language(&frontend.source_language)?;
    validate_executable_name(&frontend.name, "frontend name")?;
    validate_version(&frontend.version, "frontend version")?;
    validate_profile_id(&frontend.limit_profile_id, "limit profile ID")?;
    validate_profile_id(&frontend.environment_profile_id, "environment profile ID")?;
    validate_profile_id(&frontend.argument_profile_id, "argument profile ID")?;
    validate_executable_record(&frontend.main)?;
    for executable in &frontend.subordinate_binaries {
        validate_executable_record(executable)?;
    }
    validate_inventory_scalar(&frontend.inventory)?;
    validate_sha256(&frontend.bundle_sha256, "frontend bundle SHA-256")
}

fn validate_toolchain_scalar(toolchain: &ToolchainBundle) -> Result<(), ReleaseRegistryError> {
    validate_bundle_id(&toolchain.bundle_id, "toolchain bundle ID")?;
    validate_language(&toolchain.source_language)?;
    match &toolchain.compiler {
        CompilerIdentity::Go { release } => validate_version(release, "Go compiler release")?,
        CompilerIdentity::Rust {
            release,
            rustc_commit,
        } => {
            validate_version(release, "Rust compiler release")?;
            scalar_require(
                rustc_commit.len() == 40
                    && rustc_commit
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
                "rustc_commit must be 40 lowercase hexadecimal characters",
            )?;
        }
    }
    validate_profile_id(
        &toolchain.execution_host_profile_id,
        "toolchain host profile ID",
    )?;
    match &toolchain.native_runtime {
        NativeRuntimeSelection::None => {}
        NativeRuntimeSelection::Component {
            component_name,
            component_root,
            layout_profile_id,
        } => {
            validate_component_name(component_name, "native runtime component name")?;
            validate_portable_path(component_root, "native runtime component root")?;
            validate_profile_id(layout_profile_id, "native runtime layout profile ID")?;
        }
    }
    scalar_require(
        !toolchain.components.is_empty(),
        "toolchain components must be nonempty",
    )?;
    for component in &toolchain.components {
        match component {
            ToolchainComponent::Executable {
                name,
                release,
                path,
                binary_sha256,
                runtime,
            } => {
                validate_component_name(name, "executable component name")?;
                validate_version(release, "executable component release")?;
                validate_portable_path(path, "executable component path")?;
                validate_sha256(binary_sha256, "executable component SHA-256")?;
                validate_runtime(runtime)?;
            }
            ToolchainComponent::Content {
                name,
                release,
                inventory,
                content_sha256,
            } => {
                validate_component_name(name, "content component name")?;
                validate_version(release, "content component release")?;
                validate_inventory_scalar(inventory)?;
                validate_sha256(content_sha256, "content component SHA-256")?;
            }
        }
    }
    scalar_require(
        !toolchain.target_libraries.is_empty(),
        "target_libraries must be nonempty",
    )?;
    for target in &toolchain.target_libraries {
        validate_target(&toolchain.source_language, &target.target_id)?;
        validate_pointer_width(target.pointer_width)?;
        validate_component_name(&target.component_name, "target component name")?;
        validate_sha256(&target.content_sha256, "target content SHA-256")?;
    }
    validate_inventory_scalar(&toolchain.inventory)?;
    validate_sha256(
        &toolchain.distribution_sha256,
        "toolchain distribution SHA-256",
    )
}

fn validate_inventory_scalar(inventory: &BundleInventory) -> Result<(), ReleaseRegistryError> {
    match &inventory.scope {
        InventoryScope::FrontendBundle { bundle_id }
        | InventoryScope::ToolchainBundle { bundle_id } => {
            validate_bundle_id(bundle_id, "inventory scope bundle ID")?;
        }
        InventoryScope::Component {
            bundle_id,
            component_name,
        } => {
            validate_bundle_id(bundle_id, "inventory scope bundle ID")?;
            validate_component_name(component_name, "inventory scope component name")?;
        }
    }
    scalar_require(
        !inventory.files.is_empty(),
        "inventory files must be nonempty",
    )?;
    let mut declared_bytes = 0_u64;
    for file in &inventory.files {
        validate_portable_path(&file.path, "inventory path")?;
        let size_bytes = u64::try_from(file.size_bytes)
            .map_err(|_| invalid(ReleaseValidationPhase::Scalar, "negative bundle file size"))?;
        check_count(size_bytes, BUNDLE_FILE_BYTES_MAX, "bundle file bytes")?;
        declared_bytes = declared_bytes
            .checked_add(size_bytes)
            .ok_or_else(|| scalar_limit("bundle declared-byte sum overflow"))?;
        validate_sha256(&file.sha256, "inventory file SHA-256")?;
    }
    check_count(
        declared_bytes,
        BUNDLE_DECLARED_BYTES_MAX,
        "bundle declared bytes",
    )
}

fn validate_executable_record(record: &ExecutableRecord) -> Result<(), ReleaseRegistryError> {
    validate_executable_name(&record.name, "executable name")?;
    validate_version(&record.version, "executable version")?;
    validate_portable_path(&record.path, "executable path")?;
    validate_sha256(&record.binary_sha256, "executable binary SHA-256")?;
    validate_runtime(&record.runtime)
}

fn validate_runtime(runtime: &ExecutableRuntime) -> Result<(), ReleaseRegistryError> {
    if let ExecutableRuntime::Dynamic {
        interpreter_mount,
        libraries,
    } = runtime
    {
        validate_absolute_path(interpreter_mount, "dynamic interpreter mount")?;
        scalar_require(!libraries.is_empty(), "dynamic libraries must be nonempty")?;
        for library in libraries {
            validate_soname(&library.soname)?;
            validate_portable_path(&library.component_path, "runtime library path")?;
            validate_sha256(&library.sha256, "runtime library SHA-256")?;
        }
    }
    Ok(())
}

fn validate_order_and_references(registry: &BundleRegistry) -> Result<(), ReleaseRegistryError> {
    require_sorted_unique_by(
        &registry.execution_host_profiles,
        |profile| profile.id.as_str(),
        "execution host profiles",
    )?;
    require_sorted_unique_by(
        &registry.native_runtime_layout_profiles,
        |profile| profile.id.as_str(),
        "native runtime layout profiles",
    )?;
    require_sorted_unique_by(
        &registry.frontend_bundles,
        |bundle| bundle.bundle_id.as_str(),
        "frontend bundles",
    )?;
    require_sorted_unique_by(
        &registry.toolchain_bundles,
        |bundle| bundle.bundle_id.as_str(),
        "toolchain bundles",
    )?;

    let frontend_ids: BTreeSet<_> = registry
        .frontend_bundles
        .iter()
        .map(|bundle| bundle.bundle_id.as_str())
        .collect();
    let toolchain_ids: BTreeSet<_> = registry
        .toolchain_bundles
        .iter()
        .map(|bundle| bundle.bundle_id.as_str())
        .collect();
    order_require(
        frontend_ids.is_disjoint(&toolchain_ids),
        "frontend and toolchain bundle IDs must be disjoint",
    )?;

    for frontend in &registry.frontend_bundles {
        validate_inventory_order(&frontend.inventory)?;
        require_strict_order(
            &frontend.subordinate_binaries,
            |left, right| {
                (left.name.as_bytes(), left.path.as_bytes())
                    .cmp(&(right.name.as_bytes(), right.path.as_bytes()))
            },
            "frontend subordinate binaries",
        )?;
        require_unique_by(
            &frontend.subordinate_binaries,
            |record| record.name.as_str(),
            "subordinate executable names",
        )?;
        require_unique_by(
            &frontend.subordinate_binaries,
            |record| record.path.as_str(),
            "subordinate executable paths",
        )?;
        validate_runtime_order(&frontend.main.runtime)?;
        for executable in &frontend.subordinate_binaries {
            validate_runtime_order(&executable.runtime)?;
        }
    }
    for toolchain in &registry.toolchain_bundles {
        validate_inventory_order(&toolchain.inventory)?;
        require_sorted_unique_by(
            &toolchain.components,
            ToolchainComponent::name,
            "toolchain components",
        )?;
        require_sorted_unique_by(
            &toolchain.target_libraries,
            |target| target.target_id.as_str(),
            "target libraries",
        )?;
        for component in &toolchain.components {
            match component {
                ToolchainComponent::Executable { runtime, .. } => {
                    validate_runtime_order(runtime)?;
                }
                ToolchainComponent::Content { inventory, .. } => {
                    validate_inventory_order(inventory)?;
                }
            }
        }
    }
    for profile in &registry.native_runtime_layout_profiles {
        require_sorted_unique_by(
            &profile.interpreter_mounts,
            |mount| mount.sandbox_path.as_str(),
            "interpreter mounts",
        )?;
        require_unique_by(
            &profile.interpreter_mounts,
            |mount| mount.component_path.as_str(),
            "interpreter component paths",
        )?;
        require_sorted_unique_by(
            &profile.library_mounts,
            |mount| mount.sandbox_path.as_str(),
            "library mounts",
        )?;
        require_unique_by(
            &profile.library_mounts,
            |mount| mount.component_path.as_str(),
            "library component paths",
        )?;
        require_unique_by(
            &profile.loader_search_paths,
            String::as_str,
            "loader search paths",
        )?;
    }

    require_strict_order(&registry.tuples, compare_tuple_key, "release tuples")?;
    let mut selection_keys = BTreeSet::new();
    for tuple in &registry.tuples {
        order_require(
            selection_keys.insert((
                tuple.source_language.as_str(),
                tuple.semantic_profile.as_str(),
                tuple.target_id.as_str(),
                tuple.frontend_bundle_id.as_str(),
                tuple.toolchain_bundle_id.as_str(),
            )),
            "caller selection keys must be unique",
        )?;
    }

    let host_ids: BTreeSet<_> = registry
        .execution_host_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect();
    let layout_ids: BTreeSet<_> = registry
        .native_runtime_layout_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect();
    for profile in &registry.native_runtime_layout_profiles {
        order_require(
            host_ids.contains(profile.execution_host_profile_id.as_str()),
            "runtime layout references an unknown host profile",
        )?;
    }
    for toolchain in &registry.toolchain_bundles {
        order_require(
            host_ids.contains(toolchain.execution_host_profile_id.as_str()),
            "toolchain references an unknown host profile",
        )?;
        if let NativeRuntimeSelection::Component {
            component_name,
            layout_profile_id,
            ..
        } = &toolchain.native_runtime
        {
            order_require(
                layout_ids.contains(layout_profile_id.as_str()),
                "toolchain references an unknown runtime layout profile",
            )?;
            order_require(
                toolchain
                    .components
                    .iter()
                    .any(|component| component.name() == component_name),
                "toolchain references an unknown runtime component",
            )?;
        }
        for target in &toolchain.target_libraries {
            order_require(
                toolchain
                    .components
                    .iter()
                    .any(|component| component.name() == target.component_name),
                "target library references an unknown component",
            )?;
        }
    }
    for tuple in &registry.tuples {
        order_require(
            frontend_ids.contains(tuple.frontend_bundle_id.as_str()),
            "tuple references an unknown frontend bundle",
        )?;
        order_require(
            toolchain_ids.contains(tuple.toolchain_bundle_id.as_str()),
            "tuple references an unknown toolchain bundle",
        )?;
    }
    Ok(())
}

fn validate_inventory_order(inventory: &BundleInventory) -> Result<(), ReleaseRegistryError> {
    require_sorted_unique_by(
        &inventory.files,
        |file| file.path.as_str(),
        "inventory files",
    )?;
    let mut folded = BTreeSet::new();
    for file in &inventory.files {
        order_require(
            folded.insert(file.path.to_ascii_lowercase()),
            "inventory paths collide under ASCII case folding",
        )?;
    }
    Ok(())
}

fn validate_runtime_order(runtime: &ExecutableRuntime) -> Result<(), ReleaseRegistryError> {
    let ExecutableRuntime::Dynamic { libraries, .. } = runtime else {
        return Ok(());
    };
    require_strict_order(
        libraries,
        |left, right| {
            (left.soname.as_bytes(), left.component_path.as_bytes())
                .cmp(&(right.soname.as_bytes(), right.component_path.as_bytes()))
        },
        "dynamic libraries",
    )?;
    require_unique_by(
        libraries,
        |library| library.soname.as_str(),
        "dynamic library SONAMEs",
    )?;
    require_unique_by(
        libraries,
        |library| library.component_path.as_str(),
        "dynamic library component paths",
    )
}

fn validate_invariants(registry: &BundleRegistry) -> Result<(), ReleaseRegistryError> {
    let all_empty = registry.execution_host_profiles.is_empty()
        && registry.native_runtime_layout_profiles.is_empty()
        && registry.frontend_bundles.is_empty()
        && registry.toolchain_bundles.is_empty()
        && registry.tuples.is_empty();
    let any_empty = registry.execution_host_profiles.is_empty()
        || registry.native_runtime_layout_profiles.is_empty()
        || registry.frontend_bundles.is_empty()
        || registry.toolchain_bundles.is_empty()
        || registry.tuples.is_empty();
    invariant_require(
        all_empty || !any_empty,
        "bootstrap arrays must be empty together",
    )?;
    if all_empty {
        return Ok(());
    }

    for profile in &registry.execution_host_profiles {
        invariant_require(
            profile
                .required_primitives
                .iter()
                .map(String::as_str)
                .eq(REQUIRED_PRIMITIVES),
            "host required_primitives do not match the closed profile",
        )?;
    }
    for profile in &registry.native_runtime_layout_profiles {
        validate_layout_invariants(profile)?;
    }

    for frontend in &registry.frontend_bundles {
        validate_frontend_invariants(frontend)?;
    }
    for toolchain in &registry.toolchain_bundles {
        validate_toolchain_invariants(toolchain)?;
    }

    let frontends: BTreeMap<_, _> = registry
        .frontend_bundles
        .iter()
        .map(|bundle| (bundle.bundle_id.as_str(), bundle))
        .collect();
    let toolchains: BTreeMap<_, _> = registry
        .toolchain_bundles
        .iter()
        .map(|bundle| (bundle.bundle_id.as_str(), bundle))
        .collect();
    let layouts: BTreeMap<_, _> = registry
        .native_runtime_layout_profiles
        .iter()
        .map(|profile| (profile.id.as_str(), profile))
        .collect();

    let mut selected_frontends = BTreeSet::new();
    let mut selected_toolchains = BTreeSet::new();
    let mut selected_targets = BTreeSet::new();
    let mut paired_frontends: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for tuple in &registry.tuples {
        let Some(frontend) = frontends.get(tuple.frontend_bundle_id.as_str()) else {
            return Err(invariant("tuple references an unknown frontend bundle"));
        };
        let Some(toolchain) = toolchains.get(tuple.toolchain_bundle_id.as_str()) else {
            return Err(invariant("tuple references an unknown toolchain bundle"));
        };
        validate_tuple_invariants(tuple, frontend, toolchain)?;
        selected_frontends.insert(frontend.bundle_id.as_str());
        selected_toolchains.insert(toolchain.bundle_id.as_str());
        selected_targets.insert((toolchain.bundle_id.as_str(), tuple.target_id.as_str()));
        paired_frontends
            .entry(toolchain.bundle_id.as_str())
            .or_default()
            .insert(frontend.bundle_id.as_str());
    }
    invariant_require(
        selected_frontends.len() == registry.frontend_bundles.len(),
        "every frontend descriptor must be selected by a tuple",
    )?;
    invariant_require(
        selected_toolchains.len() == registry.toolchain_bundles.len(),
        "every toolchain descriptor must be selected by a tuple",
    )?;
    for toolchain in &registry.toolchain_bundles {
        let Some(frontend_ids) = paired_frontends.get(toolchain.bundle_id.as_str()) else {
            return Err(invariant("toolchain has no paired frontend"));
        };
        let paired: Vec<_> = frontend_ids
            .iter()
            .filter_map(|id| frontends.get(id).copied())
            .collect();
        invariant_require(
            paired.len() == frontend_ids.len(),
            "toolchain pairing references an unknown frontend",
        )?;
        validate_toolchain_runtime(&paired, toolchain, &layouts)?;
    }
    for toolchain in &registry.toolchain_bundles {
        for target in &toolchain.target_libraries {
            invariant_require(
                selected_targets
                    .contains(&(toolchain.bundle_id.as_str(), target.target_id.as_str())),
                "every target library must be selected by a tuple",
            )?;
        }
    }

    let selected_hosts: BTreeSet<_> = registry
        .toolchain_bundles
        .iter()
        .map(|bundle| bundle.execution_host_profile_id.as_str())
        .collect();
    invariant_require(
        selected_hosts.len() == registry.execution_host_profiles.len(),
        "every host profile must be selected by a toolchain",
    )?;
    let selected_layouts: BTreeSet<_> = registry
        .toolchain_bundles
        .iter()
        .filter_map(|bundle| match &bundle.native_runtime {
            NativeRuntimeSelection::Component {
                layout_profile_id, ..
            } => Some(layout_profile_id.as_str()),
            NativeRuntimeSelection::None => None,
        })
        .collect();
    invariant_require(
        selected_layouts.len() == registry.native_runtime_layout_profiles.len(),
        "every runtime layout profile must be selected by a toolchain",
    )
}

fn validate_layout_invariants(
    profile: &NativeRuntimeLayoutProfile,
) -> Result<(), ReleaseRegistryError> {
    invariant_require(
        profile
            .forbidden_host_roots
            .iter()
            .map(String::as_str)
            .eq(FORBIDDEN_HOST_ROOTS),
        "forbidden_host_roots do not match the closed profile",
    )?;
    invariant_require(
        profile.loader_search_paths.len() == profile.library_mounts.len()
            && profile
                .loader_search_paths
                .iter()
                .zip(&profile.library_mounts)
                .all(|(path, mount)| path == &mount.sandbox_path),
        "loader_search_paths must exactly match library mount destinations",
    )?;
    for (index, mount) in profile.library_mounts.iter().enumerate() {
        for other in profile.library_mounts.iter().skip(index + 1) {
            invariant_require(
                !paths_overlap(&mount.component_path, &other.component_path)
                    && !paths_overlap(&mount.sandbox_path, &other.sandbox_path),
                "library mount directory prefixes overlap",
            )?;
        }
        for interpreter in &profile.interpreter_mounts {
            invariant_require(
                !paths_overlap(&interpreter.component_path, &mount.component_path)
                    && !paths_overlap(&interpreter.sandbox_path, &mount.sandbox_path),
                "library mount overlaps an interpreter file",
            )?;
        }
    }
    Ok(())
}

fn validate_frontend_invariants(frontend: &FrontendBundle) -> Result<(), ReleaseRegistryError> {
    let (limit_profile, environment_profile, argument_profile, subordinate_name) =
        match frontend.source_language.as_str() {
            "go" => (
                "mpk.vir.limits.v0",
                "mpk.go.frontend_environment.v0",
                "mpk.go.frontend_arguments.v0",
                None,
            ),
            "rust" => (
                "mpk.vir.limits.v0",
                "mpk.rust.frontend_environment.v0",
                "mpk.rust.frontend_arguments.v0",
                Some("rust2vir-driver"),
            ),
            _ => return Err(invariant("frontend has an invalid source language")),
        };
    invariant_require(
        frontend.limit_profile_id == limit_profile
            && frontend.environment_profile_id == environment_profile
            && frontend.argument_profile_id == argument_profile,
        "frontend profile IDs are incompatible with its language",
    )?;
    invariant_require(
        frontend.main.name == frontend.name,
        "frontend main name must equal descriptor name",
    )?;
    match subordinate_name {
        None => invariant_require(
            frontend.subordinate_binaries.is_empty(),
            "Go frontend must not declare subordinate binaries",
        )?,
        Some(name) => invariant_require(
            frontend.subordinate_binaries.len() == 1
                && frontend.subordinate_binaries[0].name == name,
            "Rust frontend must declare exactly rust2vir-driver",
        )?,
    }
    match &frontend.inventory.scope {
        InventoryScope::FrontendBundle { bundle_id } => invariant_require(
            bundle_id == &frontend.bundle_id,
            "frontend inventory scope does not match descriptor",
        )?,
        _ => return Err(invariant("frontend inventory has the wrong scope kind")),
    }

    let files: BTreeMap<_, _> = frontend
        .inventory
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut named_paths = BTreeSet::new();
    let mut named_names = BTreeSet::new();
    for record in std::iter::once(&frontend.main).chain(&frontend.subordinate_binaries) {
        invariant_require(
            named_paths.insert(record.path.as_str()) && named_names.insert(record.name.as_str()),
            "frontend executable names and paths must be unique",
        )?;
        let Some(file) = files.get(record.path.as_str()) else {
            return Err(invariant("frontend executable is absent from inventory"));
        };
        invariant_require(
            file.executable && digest_equal(&file.sha256, &record.binary_sha256),
            "frontend executable record does not match its inventory entry",
        )?;
    }
    let executable_paths: BTreeSet<_> = frontend
        .inventory
        .files
        .iter()
        .filter(|file| file.executable)
        .map(|file| file.path.as_str())
        .collect();
    invariant_require(
        executable_paths == named_paths,
        "every frontend executable inventory entry must be named exactly once",
    )
}

fn validate_toolchain_invariants(toolchain: &ToolchainBundle) -> Result<(), ReleaseRegistryError> {
    invariant_require(
        matches!(
            (&toolchain.source_language[..], &toolchain.compiler),
            ("go", CompilerIdentity::Go { .. }) | ("rust", CompilerIdentity::Rust { .. })
        ),
        "compiler identity kind does not match toolchain language",
    )?;
    match &toolchain.inventory.scope {
        InventoryScope::ToolchainBundle { bundle_id } => invariant_require(
            bundle_id == &toolchain.bundle_id,
            "toolchain inventory scope does not match descriptor",
        )?,
        _ => return Err(invariant("toolchain inventory has the wrong scope kind")),
    }

    let root_files: BTreeMap<_, _> = toolchain
        .inventory
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut assigned = BTreeSet::new();
    let components: BTreeMap<_, _> = toolchain
        .components
        .iter()
        .map(|component| (component.name(), component))
        .collect();
    let native_component_name = match &toolchain.native_runtime {
        NativeRuntimeSelection::None => None,
        NativeRuntimeSelection::Component { component_name, .. } => Some(component_name.as_str()),
    };
    for component in &toolchain.components {
        match component {
            ToolchainComponent::Executable {
                path,
                binary_sha256,
                ..
            } => {
                let Some(root) = root_files.get(path.as_str()) else {
                    return Err(invariant(
                        "executable component is absent from root inventory",
                    ));
                };
                invariant_require(
                    root.executable && digest_equal(&root.sha256, binary_sha256),
                    "executable component does not match its root inventory entry",
                )?;
                invariant_require(
                    assigned.insert(path.as_str()),
                    "toolchain root file belongs to multiple components",
                )?;
            }
            ToolchainComponent::Content {
                name, inventory, ..
            } => {
                match &inventory.scope {
                    InventoryScope::Component {
                        bundle_id,
                        component_name,
                    } => invariant_require(
                        bundle_id == &toolchain.bundle_id && component_name == name,
                        "component inventory scope does not match its component",
                    )?,
                    _ => return Err(invariant("content component has the wrong scope kind")),
                }
                if native_component_name != Some(name.as_str()) {
                    invariant_require(
                        inventory.files.iter().all(|file| !file.executable),
                        "executable content is permitted only in the native runtime component",
                    )?;
                }
                for file in &inventory.files {
                    let Some(root) = root_files.get(file.path.as_str()) else {
                        return Err(invariant("component file is absent from root inventory"));
                    };
                    invariant_require(
                        *root == file,
                        "component inventory entry differs from root inventory",
                    )?;
                    invariant_require(
                        assigned.insert(file.path.as_str()),
                        "toolchain root file belongs to multiple components",
                    )?;
                }
            }
        }
    }
    invariant_require(
        assigned.len() == root_files.len(),
        "toolchain components must partition the root inventory",
    )?;

    for target in &toolchain.target_libraries {
        let Some(component) = components.get(target.component_name.as_str()) else {
            return Err(invariant("target library references an unknown component"));
        };
        let ToolchainComponent::Content { content_sha256, .. } = component else {
            return Err(invariant(
                "target library must reference a content component",
            ));
        };
        invariant_require(
            digest_equal(content_sha256, &target.content_sha256),
            "target library digest does not match its content component",
        )?;
    }
    Ok(())
}

fn validate_tuple_invariants(
    tuple: &ReleaseTuple,
    frontend: &FrontendBundle,
    toolchain: &ToolchainBundle,
) -> Result<(), ReleaseRegistryError> {
    invariant_require(
        tuple.source_language == frontend.source_language
            && tuple.source_language == toolchain.source_language,
        "tuple language does not match both descriptors",
    )?;
    let expected_profile = match tuple.source_language.as_str() {
        "go" => "mpk.go.fixed.v0",
        "rust" => "mpk.rust.checked.v0",
        _ => return Err(invariant("tuple has an invalid source language")),
    };
    invariant_require(
        tuple.semantic_profile == expected_profile,
        "unsupported language and semantic-profile pairing",
    )?;
    invariant_require(
        tuple.limit_profile_id == frontend.limit_profile_id,
        "tuple limit profile does not match frontend",
    )?;
    let matching_targets: Vec<_> = toolchain
        .target_libraries
        .iter()
        .filter(|target| target.target_id == tuple.target_id)
        .collect();
    invariant_require(
        matching_targets.len() == 1 && matching_targets[0].pointer_width == tuple.pointer_width,
        "tuple target and pointer width do not resolve one target library",
    )
}

fn validate_toolchain_runtime(
    frontends: &[&FrontendBundle],
    toolchain: &ToolchainBundle,
    layouts: &BTreeMap<&str, &NativeRuntimeLayoutProfile>,
) -> Result<(), ReleaseRegistryError> {
    let mut runtime_list = Vec::new();
    for frontend in frontends {
        runtime_list.push(&frontend.main.runtime);
        runtime_list.extend(
            frontend
                .subordinate_binaries
                .iter()
                .map(|record| &record.runtime),
        );
    }
    runtime_list.extend(toolchain.components.iter().filter_map(|component| {
        if let ToolchainComponent::Executable { runtime, .. } = component {
            Some(runtime)
        } else {
            None
        }
    }));
    let has_dynamic = runtime_list
        .iter()
        .any(|runtime| matches!(runtime, ExecutableRuntime::Dynamic { .. }));

    match &toolchain.native_runtime {
        NativeRuntimeSelection::None => {
            invariant_require(!has_dynamic, "dynamic executable requires native runtime")?;
            invariant_require(
                !toolchain
                    .components
                    .iter()
                    .any(|component| component.name() == "native-runtime"),
                "native-runtime component is forbidden with native_runtime none",
            )
        }
        NativeRuntimeSelection::Component {
            component_name,
            component_root,
            layout_profile_id,
        } => {
            invariant_require(has_dynamic, "native runtime component is unreferenced")?;
            invariant_require(
                component_name == "native-runtime" && component_root == "native-runtime",
                "native runtime component name and root must be literal native-runtime",
            )?;
            let Some(layout) = layouts.get(layout_profile_id.as_str()).copied() else {
                return Err(invariant(
                    "toolchain references an unknown runtime layout profile",
                ));
            };
            invariant_require(
                layout.execution_host_profile_id == toolchain.execution_host_profile_id,
                "runtime layout host does not match toolchain host",
            )?;
            let Some(component) = toolchain
                .components
                .iter()
                .find(|candidate| candidate.name() == component_name)
            else {
                return Err(invariant(
                    "toolchain references an unknown runtime component",
                ));
            };
            let ToolchainComponent::Content { inventory, .. } = component else {
                return Err(invariant("native runtime must be a content component"));
            };
            validate_runtime_inventory(inventory, layout, &runtime_list)
        }
    }
}

fn validate_runtime_inventory(
    inventory: &BundleInventory,
    layout: &NativeRuntimeLayoutProfile,
    runtimes: &[&ExecutableRuntime],
) -> Result<(), ReleaseRegistryError> {
    let files: BTreeMap<_, _> = inventory
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let prefixed = |path: &str| format!("native-runtime/{path}");

    for file in &inventory.files {
        invariant_require(
            file.path.starts_with("native-runtime/"),
            "native runtime inventory path is outside native-runtime/",
        )?;
        let relative = &file.path["native-runtime/".len()..];
        let covered = layout
            .interpreter_mounts
            .iter()
            .any(|mount| mount.component_path == relative)
            || layout
                .library_mounts
                .iter()
                .any(|mount| path_is_beneath(relative, &mount.component_path));
        invariant_require(covered, "native runtime inventory exceeds layout union")?;
    }
    let mut used_files = BTreeSet::new();
    for mount in &layout.interpreter_mounts {
        let path = prefixed(&mount.component_path);
        let Some(file) = files.get(path.as_str()) else {
            return Err(invariant(
                "runtime layout interpreter is absent from component",
            ));
        };
        invariant_require(
            file.executable,
            "runtime layout interpreter must have executable class",
        )?;
        used_files.insert(path);
    }
    for mount in &layout.library_mounts {
        invariant_require(
            inventory.files.iter().any(|file| {
                let relative = file.path.strip_prefix("native-runtime/").unwrap_or("");
                path_is_beneath(relative, &mount.component_path)
            }),
            "runtime layout library directory is empty",
        )?;
    }
    for runtime in runtimes {
        let ExecutableRuntime::Dynamic {
            interpreter_mount,
            libraries,
        } = runtime
        else {
            continue;
        };
        let Some(interpreter) = layout
            .interpreter_mounts
            .iter()
            .find(|mount| mount.sandbox_path == *interpreter_mount)
        else {
            return Err(invariant(
                "dynamic interpreter is absent from runtime layout",
            ));
        };
        let interpreter_path = prefixed(&interpreter.component_path);
        let Some(interpreter_file) = files.get(interpreter_path.as_str()) else {
            return Err(invariant(
                "runtime layout interpreter is absent from component",
            ));
        };
        invariant_require(
            interpreter_file.executable,
            "runtime interpreter must have executable class",
        )?;
        used_files.insert(interpreter_path);

        for library in libraries {
            invariant_require(
                layout
                    .library_mounts
                    .iter()
                    .any(|mount| path_is_beneath(&library.component_path, &mount.component_path)),
                "dynamic library is outside runtime layout library mounts",
            )?;
            let path = prefixed(&library.component_path);
            let Some(file) = files.get(path.as_str()) else {
                return Err(invariant(
                    "dynamic library is absent from runtime component",
                ));
            };
            invariant_require(
                digest_equal(&file.sha256, &library.sha256),
                "dynamic library does not match runtime inventory",
            )?;
            used_files.insert(path);
        }
    }
    invariant_require(
        inventory
            .files
            .iter()
            .filter(|file| file.executable)
            .all(|file| used_files.contains(&file.path)),
        "executable runtime content is not named by a dynamic closure",
    )
}

fn validate_content_hashes(
    registry: &BundleRegistry,
    root: &StrictJsonValue,
) -> Result<(), ReleaseRegistryError> {
    let frontend_values = strict_array(strict_field(root, "frontend_bundles")?)?;
    content_shape_require(
        frontend_values.len() == registry.frontend_bundles.len(),
        "frontend value count changed after shape validation",
    )?;
    for (frontend, frontend_value) in registry.frontend_bundles.iter().zip(frontend_values) {
        let inventory = strict_field(frontend_value, "inventory")?;
        require_content_digest(inventory, &frontend.bundle_sha256, "frontend inventory")?;
    }
    let toolchain_values = strict_array(strict_field(root, "toolchain_bundles")?)?;
    content_shape_require(
        toolchain_values.len() == registry.toolchain_bundles.len(),
        "toolchain value count changed after shape validation",
    )?;
    for (toolchain, toolchain_value) in registry.toolchain_bundles.iter().zip(toolchain_values) {
        let inventory = strict_field(toolchain_value, "inventory")?;
        require_content_digest(
            inventory,
            &toolchain.distribution_sha256,
            "toolchain inventory",
        )?;
        let component_values = strict_array(strict_field(toolchain_value, "components")?)?;
        content_shape_require(
            component_values.len() == toolchain.components.len(),
            "component value count changed after shape validation",
        )?;
        for (component, component_value) in toolchain.components.iter().zip(component_values) {
            if let ToolchainComponent::Content { content_sha256, .. } = component {
                let component_inventory = strict_field(component_value, "inventory")?;
                require_content_digest(component_inventory, content_sha256, "component inventory")?;
            }
        }
    }
    Ok(())
}

fn content_shape_require(
    condition: bool,
    detail: impl Into<String>,
) -> Result<(), ReleaseRegistryError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(ReleaseValidationPhase::ContentHash, detail))
    }
}

fn require_content_digest(
    inventory: &StrictJsonValue,
    declared: &str,
    label: &str,
) -> Result<(), ReleaseRegistryError> {
    let actual = hash_canonical_inventory(BUNDLE_CONTENT_HASH_DOMAIN, inventory)
        .map_err(|error| invalid(ReleaseValidationPhase::ContentHash, error.to_string()))?;
    if decode_sha256(declared).ok() == Some(*actual.as_bytes()) {
        Ok(())
    } else {
        Err(invalid(
            ReleaseValidationPhase::ContentHash,
            format!("{label} content hash mismatch"),
        ))
    }
}

fn validate_registry_hash(
    registry: &BundleRegistry,
    root: &StrictJsonValue,
) -> Result<Sha256Digest, ReleaseRegistryError> {
    let preimage = root
        .clone_without_fields(&["registry_sha256"])
        .map_err(|error| invalid(ReleaseValidationPhase::RegistryHash, error.to_string()))?;
    let actual = hash_canonical_json(BUNDLE_REGISTRY_HASH_DOMAIN, &preimage)
        .map_err(|error| invalid(ReleaseValidationPhase::RegistryHash, error.to_string()))?;
    if decode_sha256(&registry.registry_sha256).ok() == Some(*actual.as_bytes()) {
        Ok(actual)
    } else {
        Err(invalid(
            ReleaseValidationPhase::RegistryHash,
            "registry SHA-256 mismatch",
        ))
    }
}

/// Applies one named release-limit counter. This is also used by conformance
/// tests whose synthetic boundary values do not construct a complete registry.
pub fn validate_release_limit(kind: &str, value: u64) -> Result<(), ReleaseRegistryError> {
    let (maximum, phase) = match kind {
        "registry_canonical_bytes" => (
            REGISTRY_CANONICAL_BYTES_MAX,
            ReleaseValidationPhase::Transport,
        ),
        "registry_transport_bytes" => (
            REGISTRY_TRANSPORT_BYTES_MAX,
            ReleaseValidationPhase::Transport,
        ),
        "json_nesting" => (RELEASE_JSON_NESTING_MAX, ReleaseValidationPhase::Transport),
        "string_bytes" => (RELEASE_STRING_BYTES_MAX, ReleaseValidationPhase::Transport),
        "bundle_descriptors" => (BUNDLE_DESCRIPTORS_MAX, ReleaseValidationPhase::Scalar),
        "tuples" => (RELEASE_TUPLES_MAX, ReleaseValidationPhase::Scalar),
        "execution_host_profiles" => (EXECUTION_HOST_PROFILES_MAX, ReleaseValidationPhase::Scalar),
        "native_runtime_layout_profiles" => (
            NATIVE_RUNTIME_LAYOUT_PROFILES_MAX,
            ReleaseValidationPhase::Scalar,
        ),
        "components" => (TOOLCHAIN_COMPONENTS_MAX, ReleaseValidationPhase::Scalar),
        "serialized_inventory_entries" => (
            SERIALIZED_INVENTORY_ENTRIES_MAX,
            ReleaseValidationPhase::Scalar,
        ),
        "unique_bundle_files" => (UNIQUE_BUNDLE_FILES_MAX, ReleaseValidationPhase::Scalar),
        "portable_path_bytes" => (PORTABLE_PATH_BYTES_MAX, ReleaseValidationPhase::Scalar),
        "bundle_file_bytes" => (BUNDLE_FILE_BYTES_MAX, ReleaseValidationPhase::Scalar),
        "bundle_declared_bytes" => (BUNDLE_DECLARED_BYTES_MAX, ReleaseValidationPhase::Scalar),
        _ => {
            return Err(invalid(
                ReleaseValidationPhase::Scalar,
                format!("unknown release limit {kind:?}"),
            ));
        }
    };
    if value <= maximum {
        Ok(())
    } else {
        Err(limit(
            phase,
            format!("release limit {kind} exceeded: {value} > {maximum}"),
        ))
    }
}

fn validate_language(language: &str) -> Result<(), ReleaseRegistryError> {
    scalar_require(matches!(language, "go" | "rust"), "invalid source language")
}

fn validate_pointer_width(width: i64) -> Result<(), ReleaseRegistryError> {
    scalar_require(matches!(width, 32 | 64), "pointer width must be 32 or 64")
}

fn validate_bundle_id(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    validate_identifier(value, 128, label)
}

fn validate_profile_id(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    validate_identifier(value, 128, label)
}

fn validate_component_name(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    validate_identifier(value, 128, label)
}

fn validate_executable_name(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    validate_identifier(value, 64, label)
}

fn validate_identifier(
    value: &str,
    maximum: usize,
    label: &str,
) -> Result<(), ReleaseRegistryError> {
    let bytes = value.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= maximum
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        && !bytes.windows(2).any(|pair| {
            matches!(pair[0], b'.' | b'_' | b'-') && matches!(pair[1], b'.' | b'_' | b'-')
        });
    scalar_require(valid, format!("invalid {label}"))
}

fn validate_version(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    let bytes = value.as_bytes();
    scalar_require(
        !bytes.is_empty()
            && bytes.len() <= 128
            && bytes.iter().all(|byte| (0x20..=0x7e).contains(byte))
            && !bytes[0].is_ascii_whitespace()
            && !bytes[bytes.len() - 1].is_ascii_whitespace()
            && !bytes.iter().any(|byte| matches!(byte, b'/' | b'\\')),
        format!("invalid {label}"),
    )
}

fn validate_target(language: &str, target: &str) -> Result<(), ReleaseRegistryError> {
    let bytes = target.as_bytes();
    let valid = match language {
        "go" => {
            let mut parts = target.split('/');
            let valid_part = |part: &str| {
                !part.is_empty()
                    && part.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            };
            matches!((parts.next(), parts.next(), parts.next()), (Some(left), Some(right), None) if valid_part(left) && valid_part(right))
        }
        "rust" => {
            !bytes.is_empty()
                && bytes.len() <= 255
                && bytes[0].is_ascii_alphanumeric()
                && bytes[bytes.len() - 1].is_ascii_alphanumeric()
                && bytes.iter().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'.' | b'-')
                })
        }
        _ => false,
    };
    scalar_require(valid, "invalid target ID")
}

fn validate_kernel_abi(value: &str) -> Result<(), ReleaseRegistryError> {
    let parts: Vec<_> = value.split('.').collect();
    let valid = parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part == &"0" || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u32>().is_ok()
        });
    scalar_require(valid, "invalid minimum kernel ABI")
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    scalar_require(decode_sha256(value).is_ok(), format!("invalid {label}"))
}

fn decode_sha256(value: &str) -> Result<[u8; 32], ()> {
    if value.len() != 64 {
        return Err(());
    }
    let mut output = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (decode_hex(pair[0])? << 4) | decode_hex(pair[1])?;
    }
    Ok(output)
}

fn decode_hex(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(()),
    }
}

fn digest_equal(left: &str, right: &str) -> bool {
    match (decode_sha256(left), decode_sha256(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn validate_portable_path(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    let bytes = value.as_bytes();
    let valid = !bytes.is_empty()
        && u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= PORTABLE_PATH_BYTES_MAX
        && bytes.is_ascii()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains('\\')
        && value.split('/').all(|component| {
            valid_path_component(component)
                && !matches!(
                    component,
                    "candidates" | "build-inputs" | "build-input-cache"
                )
        });
    scalar_require(valid, format!("invalid {label}"))
}

fn validate_absolute_path(value: &str, label: &str) -> Result<(), ReleaseRegistryError> {
    let valid = value.starts_with('/')
        && value != "/"
        && !value.ends_with('/')
        && u64::try_from(value.len()).unwrap_or(u64::MAX) <= PORTABLE_PATH_BYTES_MAX
        && value.as_bytes().is_ascii()
        && value[1..].split('/').all(valid_path_component);
    scalar_require(valid, format!("invalid {label}"))
}

fn valid_path_component(component: &str) -> bool {
    let bytes = component.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 255
        || matches!(component, "." | "..")
        || component.ends_with('.')
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return false;
    }
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.to_ascii_uppercase();
    !matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

fn validate_soname(value: &str) -> Result<(), ReleaseRegistryError> {
    let bytes = value.as_bytes();
    scalar_require(
        !bytes.is_empty()
            && bytes.len() <= 255
            && bytes.iter().all(|byte| (0x20..=0x7e).contains(byte))
            && !bytes[0].is_ascii_whitespace()
            && !bytes[bytes.len() - 1].is_ascii_whitespace()
            && !bytes.iter().any(|byte| matches!(byte, b'/' | b'\\')),
        "invalid dynamic library SONAME",
    )
}

fn require_sorted_unique_by<'a, T, F>(
    values: &'a [T],
    key: F,
    label: &str,
) -> Result<(), ReleaseRegistryError>
where
    F: Fn(&'a T) -> &'a str,
{
    order_require(
        values
            .windows(2)
            .all(|pair| key(&pair[0]).as_bytes() < key(&pair[1]).as_bytes()),
        format!("{label} are not strictly ordered and unique"),
    )
}

fn require_strict_order<T, F>(
    values: &[T],
    compare: F,
    label: &str,
) -> Result<(), ReleaseRegistryError>
where
    F: Fn(&T, &T) -> Ordering,
{
    order_require(
        values
            .windows(2)
            .all(|pair| compare(&pair[0], &pair[1]) == Ordering::Less),
        format!("{label} are not strictly ordered and unique"),
    )
}

fn require_unique_by<'a, T, F>(
    values: &'a [T],
    key: F,
    label: &str,
) -> Result<(), ReleaseRegistryError>
where
    F: Fn(&'a T) -> &'a str,
{
    let mut seen = BTreeSet::new();
    order_require(
        values.iter().all(|value| seen.insert(key(value))),
        format!("{label} are not unique"),
    )
}

fn compare_tuple_key(left: &ReleaseTuple, right: &ReleaseTuple) -> Ordering {
    (
        left.source_language.as_bytes(),
        left.semantic_profile.as_bytes(),
        left.target_id.as_bytes(),
        left.pointer_width,
        left.limit_profile_id.as_bytes(),
        left.frontend_bundle_id.as_bytes(),
        left.toolchain_bundle_id.as_bytes(),
    )
        .cmp(&(
            right.source_language.as_bytes(),
            right.semantic_profile.as_bytes(),
            right.target_id.as_bytes(),
            right.pointer_width,
            right.limit_profile_id.as_bytes(),
            right.frontend_bundle_id.as_bytes(),
            right.toolchain_bundle_id.as_bytes(),
        ))
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right || path_is_beneath(left, right) || path_is_beneath(right, left)
}

fn path_is_beneath(path: &str, directory: &str) -> bool {
    path.strip_prefix(directory)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn strict_field<'a>(
    value: &'a StrictJsonValue,
    name: &str,
) -> Result<&'a StrictJsonValue, ReleaseRegistryError> {
    value.get(name).ok_or_else(|| {
        invalid(
            ReleaseValidationPhase::Shape,
            format!("missing JSON field {name:?}"),
        )
    })
}

fn strict_array(value: &StrictJsonValue) -> Result<&[StrictJsonValue], ReleaseRegistryError> {
    value
        .as_array()
        .ok_or_else(|| invalid(ReleaseValidationPhase::Shape, "JSON field is not an array"))
}

fn scalar_require(condition: bool, detail: impl Into<String>) -> Result<(), ReleaseRegistryError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(ReleaseValidationPhase::Scalar, detail))
    }
}

fn order_require(condition: bool, detail: impl Into<String>) -> Result<(), ReleaseRegistryError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(ReleaseValidationPhase::Order, detail))
    }
}

fn invariant(detail: impl Into<String>) -> ReleaseRegistryError {
    invalid(ReleaseValidationPhase::Invariant, detail)
}

fn invariant_require(
    condition: bool,
    detail: impl Into<String>,
) -> Result<(), ReleaseRegistryError> {
    if condition {
        Ok(())
    } else {
        Err(invariant(detail))
    }
}

fn scalar_limit(detail: impl Into<String>) -> ReleaseRegistryError {
    limit(ReleaseValidationPhase::Scalar, detail)
}

fn check_count(value: u64, maximum: u64, label: &str) -> Result<(), ReleaseRegistryError> {
    if value <= maximum {
        Ok(())
    } else {
        Err(scalar_limit(format!(
            "{label} exceed inclusive limit {maximum}"
        )))
    }
}

fn u64_len(value: usize) -> Result<u64, ReleaseRegistryError> {
    u64::try_from(value).map_err(|_| scalar_limit("collection length does not fit u64"))
}
