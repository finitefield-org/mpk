//! Inactive successor release-bundle models and validators.
//!
//! The active resolver continues to accept only `mpk.release.bundle_registry.v0`.
//! This module is reached solely by explicit C#/Go staging harnesses and never
//! discovers a repository candidate or an alternate release root.

use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_registry_semantic_context, ProfileContractField,
    ProfileRegistryIdentity, SemanticContext, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{
    canonical_json_bytes_bounded, hash_domain_separated_raw, parse_strict_json, BundleInventory,
    ExecutableRecord, ExecutableRuntime, ExecutionHostProfile, HashDomain, InventoryFile,
    InventoryScope, NativeRuntimeLayoutProfile, StrictJsonLimits, ToolchainComponent,
    BUNDLE_FILE_BYTES_MAX, PORTABLE_PATH_BYTES_MAX, REGISTRY_CANONICAL_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX, RELEASE_JSON_NESTING_MAX, RELEASE_STRING_BYTES_MAX,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const SUCCESSOR_RELEASE_REGISTRY_SCHEMA: &str = "mpk.release.bundle_registry.v1";
pub const SUCCESSOR_RELEASE_REGISTRY_ID: &str = "mpk.release.registry.v1";
pub const SUCCESSOR_FRONTEND_BUNDLE_SCHEMA: &str = "mpk.release.frontend_bundle.v1";
pub const SUCCESSOR_TOOLCHAIN_BUNDLE_SCHEMA: &str = "mpk.release.toolchain_bundle.v1";
pub const SUCCESSOR_BUNDLE_CANDIDATE_SCHEMA: &str = "mpk.release.bundle_candidate.v1";
pub const SUCCESSOR_RELEASE_REGISTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-BUNDLE-REGISTRY-1.0");
pub const CSHARP_STAGING_REGISTRY_SHA256: &str =
    "52824d9c4f6bbdce2e0d16675062f70292bdd86d903ced99278f30eaabbfc1bc";
pub const CSHARP_FRONTEND_BUNDLE_ID: &str = "frontend.csharp.csharp2vir.candidate.v1";
pub const CSHARP_TOOLCHAIN_BUNDLE_ID: &str =
    "toolchain.csharp.roslyn-5_6_0.dotnet-10_0_11.candidate.v1";
pub const CSHARP_FRONTEND_SHA256: &str =
    "0783dc269c152ad1b13e77f42f9eff6f6891002c65890bc1445f2fe1a1a0410d";
pub const CSHARP_HOST_PROFILE_ID: &str = "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0";
pub const CSHARP_RUNTIME_LAYOUT_ID: &str =
    "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0";
pub const GO_STAGING_FRONTEND_BUNDLE_ID: &str = "frontend.go.go2vir.candidate.v1";
pub const GO_STAGING_TOOLCHAIN_BUNDLE_ID: &str = "toolchain.go.go1.25.0.linux-amd64.candidate.v1";
pub const GO_STAGING_FRONTEND_SHA256: &str =
    "71f7c73b2796fd8caee6bc5e18a871e6dc1ca5639dc2a0840d6b1af4da32c0b9";
pub const RUST_STAGING_FRONTEND_BUNDLE_ID: &str = "frontend.rust.rust2vir.candidate.v1";
pub const RUST_STAGING_TOOLCHAIN_BUNDLE_ID: &str = "toolchain.rust.nightly-2025-06-01.candidate.v1";
pub const RUST_STAGING_FRONTEND_SHA256: &str =
    "b1897a991dad216b4299e618160efc6a68c87d44f2a5bc30b7ed37abff1bba9d";
pub const RUST_STAGING_DRIVER_SHA256: &str =
    "74cb253216dc7b9cbb0be61e541d8ff5eec3943daecdadab1291787d093d08d2";
pub const RUST_STAGING_TOOLCHAIN_DISTRIBUTION_SHA256: &str =
    "86dab73dadd3a3184064e7d7da7e878562eba4cfc4c8a969bc8f44a5e865c90a";

const RUST_STAGING_VERSION: &str = "0.1.0-profile-v1-staging";
const RUST_TOOLCHAIN_RELEASE: &str = "1.89.0-nightly";
const RUST_COMPONENT_RELEASE: &str = "nightly-2025-06-01";
const RUST_HOST_PROFILE_ID: &str = "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0";
const RUST_RUNTIME_LAYOUT_ID: &str = "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0";
const RUST_CARGO_SHA256: &str = "4ab49080934031ce3b87b1a8792e685f99819e8a3f537f110a339d7331f1dcea";
const RUST_RUSTC_SHA256: &str = "a7c2179d845e8f40305bace1657b903f10d149cc6d72b0c08ecef75487418922";
const RUST_NATIVE_RUNTIME_SHA256: &str =
    "6d8ebe276575c5019abdc97051baf78e166354249eca4d6b65f638c5fb171005";
const RUST_COMPILER_RUNTIME_SHA256: &str =
    "7698b22d00656113340f692fd9212a1494077fd470f924948945e690da401292";
const RUST_TARGET_I686_SHA256: &str =
    "8f606996b669eb0f4314309d145d93c6eeaad8b261791584387bcff46ccafb0a";
const RUST_TARGET_X86_64_SHA256: &str =
    "d8c45533753e17186cefde3e0830f7b358a8b4c818eb732d8814a31861335a15";

const CONTENT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-BUNDLE-CONTENT-0.1");
const TRANSPORT_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    REGISTRY_TRANSPORT_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX,
    RELEASE_JSON_NESTING_MAX,
    RELEASE_STRING_BYTES_MAX,
);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorProfileRegistryIdentity {
    pub schema: String,
    pub id: String,
    pub revision: u64,
    pub registry_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorBundleRegistry {
    pub schema: String,
    pub id: String,
    pub profile_registry: SuccessorProfileRegistryIdentity,
    pub execution_host_profiles: Vec<ExecutionHostProfile>,
    pub native_runtime_layout_profiles: Vec<NativeRuntimeLayoutProfile>,
    pub frontend_bundles: Vec<SuccessorFrontendBundle>,
    pub toolchain_bundles: Vec<SuccessorToolchainBundle>,
    pub tuples: Vec<SuccessorReleaseTuple>,
    pub registry_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorBundleCandidate {
    pub schema: String,
    pub profile_registry: SuccessorProfileRegistryIdentity,
    pub execution_host_profiles: Vec<ExecutionHostProfile>,
    pub native_runtime_layout_profiles: Vec<NativeRuntimeLayoutProfile>,
    pub frontend_bundles: Vec<SuccessorFrontendBundle>,
    pub toolchain_bundles: Vec<SuccessorToolchainBundle>,
    pub tuples: Vec<SuccessorReleaseTuple>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorFrontendBundle {
    pub schema: String,
    pub bundle_id: String,
    pub name: String,
    pub version: String,
    pub profile_contracts: Vec<Value>,
    pub main: ExecutableRecord,
    pub subordinate_binaries: Vec<ExecutableRecord>,
    pub inventory: BundleInventory,
    pub bundle_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorToolchainBundle {
    pub schema: String,
    pub bundle_id: String,
    pub execution_host_profile_id: String,
    pub profile_contracts: Vec<Value>,
    pub components: Vec<ToolchainComponent>,
    pub inventory: BundleInventory,
    pub distribution_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorReleaseTuple {
    pub semantic_context: Value,
    pub limit_profile_id: String,
    pub frontend_bundle_id: String,
    pub toolchain_bundle_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorReleaseValidationPhase {
    Transport,
    Shape,
    Identity,
    Order,
    Inventory,
    Contract,
    Linkage,
    Hash,
    CanonicalTransport,
}

impl SuccessorReleaseValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Identity => "identity",
            Self::Order => "order",
            Self::Inventory => "inventory",
            Self::Contract => "contract",
            Self::Linkage => "linkage",
            Self::Hash => "hash",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorReleaseErrorCode {
    Transport,
    Shape,
    Identity,
    Order,
    Inventory,
    Contract,
    Linkage,
    Hash,
    Canonical,
    Selection,
}

impl SuccessorReleaseErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "SUCCESSOR_RELEASE_TRANSPORT",
            Self::Shape => "SUCCESSOR_RELEASE_SHAPE",
            Self::Identity => "SUCCESSOR_RELEASE_IDENTITY",
            Self::Order => "SUCCESSOR_RELEASE_ORDER",
            Self::Inventory => "SUCCESSOR_RELEASE_INVENTORY",
            Self::Contract => "SUCCESSOR_RELEASE_CONTRACT",
            Self::Linkage => "SUCCESSOR_RELEASE_LINKAGE",
            Self::Hash => "SUCCESSOR_RELEASE_HASH",
            Self::Canonical => "SUCCESSOR_RELEASE_CANONICAL",
            Self::Selection => "SUCCESSOR_RELEASE_SELECTION",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorReleaseError {
    phase: SuccessorReleaseValidationPhase,
    code: SuccessorReleaseErrorCode,
}

impl SuccessorReleaseError {
    const fn new(phase: SuccessorReleaseValidationPhase, code: SuccessorReleaseErrorCode) -> Self {
        Self { phase, code }
    }

    pub const fn phase(&self) -> SuccessorReleaseValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorReleaseErrorCode {
        self.code
    }
}

impl fmt::Display for SuccessorReleaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at successor release phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for SuccessorReleaseError {}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorReleaseRegistry {
    registry: SuccessorBundleRegistry,
    semantic_contexts: Vec<SemanticContext>,
}

impl ValidatedSuccessorReleaseRegistry {
    pub fn registry(&self) -> &SuccessorBundleRegistry {
        &self.registry
    }

    pub fn registry_sha256(&self) -> &str {
        &self.registry.registry_sha256
    }

    pub fn release_identity(&self) -> mpk_vc::ReleaseRegistryIdentity {
        mpk_vc::ReleaseRegistryIdentity {
            schema: self.registry.schema.clone(),
            id: self.registry.id.clone(),
            registry_sha256: self.registry.registry_sha256.clone(),
        }
    }

    pub fn resolve<'a>(
        &'a self,
        semantic_registry: &ValidatedSemanticProfileRegistry,
        request: SuccessorReleaseSelectionRequest<'_>,
    ) -> Result<ResolvedSuccessorRelease<'a>, SuccessorReleaseError> {
        let context =
            validate_registry_semantic_context(semantic_registry, request.semantic_context)
                .map_err(|_| selection_failure())?;
        let index = self
            .registry
            .tuples
            .iter()
            .enumerate()
            .find_map(|(index, tuple)| {
                (self.semantic_contexts[index] == context
                    && tuple.frontend_bundle_id == request.frontend_bundle_id
                    && tuple.toolchain_bundle_id == request.toolchain_bundle_id)
                    .then_some(index)
            })
            .ok_or_else(selection_failure)?;
        let tuple = &self.registry.tuples[index];
        let frontend = self
            .registry
            .frontend_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == tuple.frontend_bundle_id)
            .ok_or_else(selection_failure)?;
        let toolchain = self
            .registry
            .toolchain_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == tuple.toolchain_bundle_id)
            .ok_or_else(selection_failure)?;
        Ok(ResolvedSuccessorRelease {
            release_tuple: tuple,
            semantic_context: context,
            frontend,
            toolchain,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SuccessorReleaseSelectionRequest<'a> {
    pub semantic_context: &'a Value,
    pub frontend_bundle_id: &'a str,
    pub toolchain_bundle_id: &'a str,
}

#[derive(Clone, Debug)]
pub struct ResolvedSuccessorRelease<'a> {
    pub release_tuple: &'a SuccessorReleaseTuple,
    pub semantic_context: SemanticContext,
    pub frontend: &'a SuccessorFrontendBundle,
    pub toolchain: &'a SuccessorToolchainBundle,
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorBundleCandidate {
    candidate: SuccessorBundleCandidate,
}

impl ValidatedSuccessorBundleCandidate {
    pub fn candidate(&self) -> &SuccessorBundleCandidate {
        &self.candidate
    }
}

pub fn validate_successor_release_registry(
    input: &[u8],
    semantic_registry: &ValidatedSemanticProfileRegistry,
) -> Result<ValidatedSuccessorReleaseRegistry, SuccessorReleaseError> {
    let (strict, canonical) = parse_transport(input)?;
    let registry: SuccessorBundleRegistry = serde_json::from_slice(&canonical).map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Shape,
            SuccessorReleaseErrorCode::Shape,
        )
    })?;
    validate_profile_registry_identity(&registry.profile_registry, semantic_registry.identity())?;
    if registry.schema != SUCCESSOR_RELEASE_REGISTRY_SCHEMA
        || registry.id != SUCCESSOR_RELEASE_REGISTRY_ID
        || !lower_sha256(&registry.registry_sha256)
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Identity,
            SuccessorReleaseErrorCode::Identity,
        ));
    }
    let contexts = validate_projection(
        &registry.execution_host_profiles,
        &registry.native_runtime_layout_profiles,
        &registry.frontend_bundles,
        &registry.toolchain_bundles,
        &registry.tuples,
        semantic_registry,
    )?;
    let expected = successor_release_registry_hash_from_strict(&strict)?;
    if registry.registry_sha256 != expected {
        return Err(failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        ));
    }
    require_canonical_transport(input, canonical)?;
    Ok(ValidatedSuccessorReleaseRegistry {
        registry,
        semantic_contexts: contexts,
    })
}

pub fn validate_successor_bundle_candidate(
    input: &[u8],
    semantic_registry: &ValidatedSemanticProfileRegistry,
) -> Result<ValidatedSuccessorBundleCandidate, SuccessorReleaseError> {
    let (_strict, canonical) = parse_transport(input)?;
    let candidate: SuccessorBundleCandidate = serde_json::from_slice(&canonical).map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Shape,
            SuccessorReleaseErrorCode::Shape,
        )
    })?;
    if candidate.schema != SUCCESSOR_BUNDLE_CANDIDATE_SCHEMA {
        return Err(failure(
            SuccessorReleaseValidationPhase::Identity,
            SuccessorReleaseErrorCode::Identity,
        ));
    }
    validate_profile_registry_identity(&candidate.profile_registry, semantic_registry.identity())?;
    validate_projection(
        &candidate.execution_host_profiles,
        &candidate.native_runtime_layout_profiles,
        &candidate.frontend_bundles,
        &candidate.toolchain_bundles,
        &candidate.tuples,
        semantic_registry,
    )?;
    require_canonical_transport(input, canonical)?;
    Ok(ValidatedSuccessorBundleCandidate { candidate })
}

pub fn successor_release_registry_hash(value: &Value) -> Result<String, SuccessorReleaseError> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        )
    })?;
    let strict = parse_strict_json(&bytes, TRANSPORT_LIMITS).map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        )
    })?;
    successor_release_registry_hash_from_strict(&strict)
}

fn successor_release_registry_hash_from_strict(
    value: &mpk_vc::StrictJsonValue,
) -> Result<String, SuccessorReleaseError> {
    let mut payload = value.clone();
    let fields = payload.as_object_mut().ok_or_else(|| {
        failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        )
    })?;
    let before = fields.len();
    fields.retain(|(name, _)| name != "registry_sha256");
    if before != fields.len() + 1 {
        return Err(failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        ));
    }
    let canonical = canonical_json_bytes_bounded(
        &payload,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).unwrap_or(usize::MAX),
    )
    .map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        )
    })?;
    hash_domain_separated_raw(SUCCESSOR_RELEASE_REGISTRY_HASH_DOMAIN, &canonical)
        .map(|digest| digest.to_hex())
        .map_err(|_| {
            failure(
                SuccessorReleaseValidationPhase::Hash,
                SuccessorReleaseErrorCode::Hash,
            )
        })
}

fn parse_transport(
    input: &[u8],
) -> Result<(mpk_vc::StrictJsonValue, Vec<u8>), SuccessorReleaseError> {
    let strict = parse_strict_json(input, TRANSPORT_LIMITS).map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Transport,
            SuccessorReleaseErrorCode::Transport,
        )
    })?;
    let canonical = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).unwrap_or(usize::MAX),
    )
    .map_err(|_| {
        failure(
            SuccessorReleaseValidationPhase::Transport,
            SuccessorReleaseErrorCode::Transport,
        )
    })?;
    Ok((strict, canonical))
}

fn require_canonical_transport(
    input: &[u8],
    mut canonical: Vec<u8>,
) -> Result<(), SuccessorReleaseError> {
    canonical.push(b'\n');
    if input == canonical {
        Ok(())
    } else {
        Err(failure(
            SuccessorReleaseValidationPhase::CanonicalTransport,
            SuccessorReleaseErrorCode::Canonical,
        ))
    }
}

fn validate_profile_registry_identity(
    actual: &SuccessorProfileRegistryIdentity,
    expected: &ProfileRegistryIdentity,
) -> Result<(), SuccessorReleaseError> {
    if actual.schema == expected.schema()
        && actual.id == expected.id()
        && actual.revision == expected.revision()
        && actual.registry_sha256 == expected.registry_sha256()
    {
        Ok(())
    } else {
        Err(failure(
            SuccessorReleaseValidationPhase::Identity,
            SuccessorReleaseErrorCode::Identity,
        ))
    }
}

fn validate_projection(
    hosts: &[ExecutionHostProfile],
    layouts: &[NativeRuntimeLayoutProfile],
    frontends: &[SuccessorFrontendBundle],
    toolchains: &[SuccessorToolchainBundle],
    tuples: &[SuccessorReleaseTuple],
    semantic_registry: &ValidatedSemanticProfileRegistry,
) -> Result<Vec<SemanticContext>, SuccessorReleaseError> {
    if hosts.is_empty()
        || frontends.is_empty()
        || toolchains.is_empty()
        || tuples.is_empty()
        || !strictly_increasing(hosts.iter().map(|value| value.id.as_str()))
        || !strictly_increasing(layouts.iter().map(|value| value.id.as_str()))
        || !strictly_increasing(frontends.iter().map(|value| value.bundle_id.as_str()))
        || !strictly_increasing(toolchains.iter().map(|value| value.bundle_id.as_str()))
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Order,
            SuccessorReleaseErrorCode::Order,
        ));
    }
    if frontends.iter().any(|frontend| {
        toolchains
            .iter()
            .any(|toolchain| toolchain.bundle_id == frontend.bundle_id)
    }) {
        return Err(failure(
            SuccessorReleaseValidationPhase::Order,
            SuccessorReleaseErrorCode::Order,
        ));
    }
    validate_host_and_layouts(hosts, layouts)?;
    for frontend in frontends {
        validate_frontend(frontend, semantic_registry)?;
    }
    for toolchain in toolchains {
        validate_toolchain(toolchain, hosts, layouts, semantic_registry)?;
    }

    let frontend_by_id: BTreeMap<_, _> = frontends
        .iter()
        .map(|bundle| (bundle.bundle_id.as_str(), bundle))
        .collect();
    let toolchain_by_id: BTreeMap<_, _> = toolchains
        .iter()
        .map(|bundle| (bundle.bundle_id.as_str(), bundle))
        .collect();
    let mut contexts = Vec::with_capacity(tuples.len());
    let mut tuple_keys = BTreeSet::new();
    let mut selected_frontends = BTreeSet::new();
    let mut selected_toolchains = BTreeSet::new();
    for tuple in tuples {
        let context =
            validate_registry_semantic_context(semantic_registry, &tuple.semantic_context)
                .map_err(|_| {
                    failure(
                        SuccessorReleaseValidationPhase::Linkage,
                        SuccessorReleaseErrorCode::Linkage,
                    )
                })?;
        let frontend = frontend_by_id
            .get(tuple.frontend_bundle_id.as_str())
            .ok_or_else(|| {
                failure(
                    SuccessorReleaseValidationPhase::Linkage,
                    SuccessorReleaseErrorCode::Linkage,
                )
            })?;
        let toolchain = toolchain_by_id
            .get(tuple.toolchain_bundle_id.as_str())
            .ok_or_else(|| {
                failure(
                    SuccessorReleaseValidationPhase::Linkage,
                    SuccessorReleaseErrorCode::Linkage,
                )
            })?;
        let frontend_contract = profile_contract(
            frontend.profile_contracts.as_slice(),
            &context,
            semantic_registry,
            ProfileContractField::Frontend,
        )?;
        let release_contract = profile_contract(
            toolchain.profile_contracts.as_slice(),
            &context,
            semantic_registry,
            ProfileContractField::Release,
        )?;
        // C# has a private frontend limit in addition to the shared VIR
        // artifact limit. Go's existing frontend limit is the shared one.
        let expected_frontend_limit = match context.semantic_profile() {
            "mpk.csharp.scalar.v0" => "mpk.csharp.limits.v0",
            "mpk.go.fixed.v0" | "mpk.rust.checked.v0" => "mpk.vir.limits.v0",
            _ => return Err(contract_failure()),
        };
        if frontend_contract
            .get("limit_profile_id")
            .and_then(Value::as_str)
            != Some(expected_frontend_limit)
            || tuple.limit_profile_id != "mpk.vir.limits.v0"
            || release_contract
                .get("execution_host_profile_id")
                .and_then(Value::as_str)
                != Some(toolchain.execution_host_profile_id.as_str())
        {
            return Err(failure(
                SuccessorReleaseValidationPhase::Linkage,
                SuccessorReleaseErrorCode::Linkage,
            ));
        }
        if context.semantic_profile() == "mpk.go.fixed.v0" {
            if frontend.bundle_id != GO_STAGING_FRONTEND_BUNDLE_ID {
                return Err(linkage_failure());
            }
            validate_go_release_contract(toolchain, release_contract)?;
        }
        if context.semantic_profile() == "mpk.rust.checked.v0" {
            if frontend.bundle_id != RUST_STAGING_FRONTEND_BUNDLE_ID {
                return Err(linkage_failure());
            }
            validate_rust_release_contract(toolchain, release_contract)?;
            validate_rust_runtime_linkage(frontend, toolchain, layouts)?;
        }
        let context_key = serde_json::to_string(&tuple.semantic_context).map_err(|_| {
            failure(
                SuccessorReleaseValidationPhase::Order,
                SuccessorReleaseErrorCode::Order,
            )
        })?;
        let key = (
            context_key,
            tuple.limit_profile_id.clone(),
            tuple.frontend_bundle_id.clone(),
            tuple.toolchain_bundle_id.clone(),
        );
        if !tuple_keys.insert(key) {
            return Err(failure(
                SuccessorReleaseValidationPhase::Order,
                SuccessorReleaseErrorCode::Order,
            ));
        }
        selected_frontends.insert(tuple.frontend_bundle_id.as_str());
        selected_toolchains.insert(tuple.toolchain_bundle_id.as_str());
        contexts.push(context);
    }
    if selected_frontends.len() != frontends.len() || selected_toolchains.len() != toolchains.len()
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Linkage,
            SuccessorReleaseErrorCode::Linkage,
        ));
    }
    Ok(contexts)
}

fn validate_host_and_layouts(
    hosts: &[ExecutionHostProfile],
    layouts: &[NativeRuntimeLayoutProfile],
) -> Result<(), SuccessorReleaseError> {
    let host_ids: BTreeSet<_> = hosts.iter().map(|host| host.id.as_str()).collect();
    for host in hosts {
        if !valid_identifier(&host.id)
            || host.required_primitives.is_empty()
            || !strictly_increasing(host.required_primitives.iter().map(String::as_str))
        {
            return Err(failure(
                SuccessorReleaseValidationPhase::Identity,
                SuccessorReleaseErrorCode::Identity,
            ));
        }
    }
    for layout in layouts {
        if !valid_identifier(&layout.id)
            || !host_ids.contains(layout.execution_host_profile_id.as_str())
            || layout.runtime_root != "/mpk/native-runtime"
            || layout.forbidden_host_roots != ["/lib", "/lib64", "/usr/lib"]
        {
            return Err(failure(
                SuccessorReleaseValidationPhase::Linkage,
                SuccessorReleaseErrorCode::Linkage,
            ));
        }
    }
    Ok(())
}

fn validate_frontend(
    frontend: &SuccessorFrontendBundle,
    semantic_registry: &ValidatedSemanticProfileRegistry,
) -> Result<(), SuccessorReleaseError> {
    if frontend.schema != SUCCESSOR_FRONTEND_BUNDLE_SCHEMA
        || !valid_identifier(&frontend.bundle_id)
        || frontend.name.is_empty()
        || frontend.version.is_empty()
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Identity,
            SuccessorReleaseErrorCode::Identity,
        ));
    }
    validate_profile_contracts(
        &frontend.profile_contracts,
        semantic_registry,
        ProfileContractField::Frontend,
    )?;
    validate_inventory(
        &frontend.inventory,
        InventoryScope::FrontendBundle {
            bundle_id: frontend.bundle_id.clone(),
        },
    )?;
    if inventory_hash(&frontend.inventory)? != frontend.bundle_sha256 {
        return Err(failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        ));
    }
    let files = inventory_files(&frontend.inventory);
    validate_executable_record(&frontend.main, &files, false)?;
    if !strictly_increasing(
        frontend
            .subordinate_binaries
            .iter()
            .map(|binary| binary.name.as_str()),
    ) {
        return Err(failure(
            SuccessorReleaseValidationPhase::Order,
            SuccessorReleaseErrorCode::Order,
        ));
    }
    for binary in &frontend.subordinate_binaries {
        validate_executable_record(binary, &files, false)?;
    }
    if frontend.bundle_id == CSHARP_FRONTEND_BUNDLE_ID
        && (frontend.name != "csharp2vir"
            || frontend.version != "0.1.0"
            || frontend.main.path != "csharp2vir.dll"
            || frontend.main.binary_sha256 != CSHARP_FRONTEND_SHA256
            || !matches!(frontend.main.runtime, ExecutableRuntime::Static)
            || frontend
                .subordinate_binaries
                .iter()
                .map(|binary| binary.name.as_str())
                .collect::<Vec<_>>()
                != [
                    "Microsoft.CodeAnalysis.CSharp.dll",
                    "Microsoft.CodeAnalysis.dll",
                ])
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Linkage,
            SuccessorReleaseErrorCode::Linkage,
        ));
    }
    if frontend.bundle_id == GO_STAGING_FRONTEND_BUNDLE_ID
        && (frontend.name != "go2vir"
            || frontend.version != "go1.25.0-profile-v1-staging"
            || frontend.main.name != "go2vir"
            || frontend.main.version != "go1.25.0-profile-v1-staging"
            || frontend.main.path != "bin/go2vir"
            || frontend.main.binary_sha256 != GO_STAGING_FRONTEND_SHA256
            || !matches!(frontend.main.runtime, ExecutableRuntime::Static)
            || !frontend.subordinate_binaries.is_empty())
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Linkage,
            SuccessorReleaseErrorCode::Linkage,
        ));
    }
    if frontend.bundle_id == GO_STAGING_FRONTEND_BUNDLE_ID {
        validate_executable_record(&frontend.main, &files, true)?;
    }
    if frontend.bundle_id == RUST_STAGING_FRONTEND_BUNDLE_ID {
        let [driver] = frontend.subordinate_binaries.as_slice() else {
            return Err(linkage_failure());
        };
        if frontend.name != "rust2vir"
            || frontend.version != RUST_STAGING_VERSION
            || frontend.main.name != "rust2vir"
            || frontend.main.version != RUST_STAGING_VERSION
            || frontend.main.path != "bin/rust2vir"
            || frontend.main.binary_sha256 != RUST_STAGING_FRONTEND_SHA256
            || !matches!(frontend.main.runtime, ExecutableRuntime::Dynamic { .. })
            || driver.name != "rust2vir-driver"
            || driver.version != RUST_STAGING_VERSION
            || driver.path != "bin/rust2vir-driver"
            || driver.binary_sha256 != RUST_STAGING_DRIVER_SHA256
            || !matches!(driver.runtime, ExecutableRuntime::Dynamic { .. })
            || files.len() != 2
        {
            return Err(linkage_failure());
        }
        validate_executable_record(&frontend.main, &files, true)?;
        validate_executable_record(driver, &files, true)?;
    }
    Ok(())
}

fn validate_toolchain(
    toolchain: &SuccessorToolchainBundle,
    hosts: &[ExecutionHostProfile],
    layouts: &[NativeRuntimeLayoutProfile],
    semantic_registry: &ValidatedSemanticProfileRegistry,
) -> Result<(), SuccessorReleaseError> {
    if toolchain.schema != SUCCESSOR_TOOLCHAIN_BUNDLE_SCHEMA
        || !valid_identifier(&toolchain.bundle_id)
        || !hosts
            .iter()
            .any(|host| host.id == toolchain.execution_host_profile_id)
        || !strictly_increasing(toolchain.components.iter().map(ToolchainComponent::name))
    {
        return Err(failure(
            SuccessorReleaseValidationPhase::Identity,
            SuccessorReleaseErrorCode::Identity,
        ));
    }
    validate_profile_contracts(
        &toolchain.profile_contracts,
        semantic_registry,
        ProfileContractField::Release,
    )?;
    validate_inventory(
        &toolchain.inventory,
        InventoryScope::ToolchainBundle {
            bundle_id: toolchain.bundle_id.clone(),
        },
    )?;
    if inventory_hash(&toolchain.inventory)? != toolchain.distribution_sha256 {
        return Err(failure(
            SuccessorReleaseValidationPhase::Hash,
            SuccessorReleaseErrorCode::Hash,
        ));
    }
    let root_files = inventory_files(&toolchain.inventory);
    let mut covered = BTreeSet::new();
    for component in &toolchain.components {
        match component {
            ToolchainComponent::Executable {
                path,
                binary_sha256,
                runtime,
                ..
            } => {
                let record = root_files
                    .get(path.as_str())
                    .ok_or_else(inventory_failure)?;
                if !record.executable || &record.sha256 != binary_sha256 {
                    return Err(inventory_failure());
                }
                validate_runtime(runtime)?;
                covered.insert(path.as_str());
            }
            ToolchainComponent::Content {
                name,
                inventory,
                content_sha256,
                ..
            } => {
                validate_inventory(
                    inventory,
                    InventoryScope::Component {
                        bundle_id: toolchain.bundle_id.clone(),
                        component_name: name.clone(),
                    },
                )?;
                if inventory_hash(inventory)? != *content_sha256 {
                    return Err(failure(
                        SuccessorReleaseValidationPhase::Hash,
                        SuccessorReleaseErrorCode::Hash,
                    ));
                }
                for record in &inventory.files {
                    if root_files.get(record.path.as_str()).copied() != Some(record)
                        || !covered.insert(record.path.as_str())
                    {
                        return Err(inventory_failure());
                    }
                }
            }
        }
    }
    if covered.len() != root_files.len() {
        return Err(inventory_failure());
    }
    if toolchain.bundle_id == CSHARP_TOOLCHAIN_BUNDLE_ID {
        let names = toolchain
            .components
            .iter()
            .map(ToolchainComponent::name)
            .collect::<Vec<_>>();
        if toolchain.execution_host_profile_id != CSHARP_HOST_PROFILE_ID
            || names
                != [
                    "dotnet",
                    "dotnet-runtime",
                    "native-runtime",
                    "reference-pack",
                ]
        {
            return Err(failure(
                SuccessorReleaseValidationPhase::Linkage,
                SuccessorReleaseErrorCode::Linkage,
            ));
        }
        validate_csharp_runtime_linkage(toolchain, layouts, &root_files)?;
    }
    if toolchain.bundle_id == RUST_STAGING_TOOLCHAIN_BUNDLE_ID {
        let names = toolchain
            .components
            .iter()
            .map(ToolchainComponent::name)
            .collect::<Vec<_>>();
        if toolchain.execution_host_profile_id != RUST_HOST_PROFILE_ID
            || toolchain.distribution_sha256 != RUST_STAGING_TOOLCHAIN_DISTRIBUTION_SHA256
            || names
                != [
                    "cargo",
                    "native-runtime",
                    "rust-compiler-runtime",
                    "rust-target-i686",
                    "rust-target-x86_64",
                    "rustc",
                ]
            || !rust_toolchain_components_are_exact(&toolchain.components)
        {
            return Err(linkage_failure());
        }
    }
    Ok(())
}

fn rust_toolchain_components_are_exact(components: &[ToolchainComponent]) -> bool {
    components.iter().all(|component| match component {
        ToolchainComponent::Executable {
            name,
            release,
            path,
            binary_sha256,
            runtime,
        } if name == "cargo" => {
            release == RUST_TOOLCHAIN_RELEASE
                && path == "bin/cargo"
                && binary_sha256 == RUST_CARGO_SHA256
                && matches!(runtime, ExecutableRuntime::Dynamic { .. })
        }
        ToolchainComponent::Executable {
            name,
            release,
            path,
            binary_sha256,
            runtime,
        } if name == "rustc" => {
            release == RUST_TOOLCHAIN_RELEASE
                && path == "bin/rustc"
                && binary_sha256 == RUST_RUSTC_SHA256
                && matches!(runtime, ExecutableRuntime::Dynamic { .. })
        }
        ToolchainComponent::Content {
            name,
            release,
            content_sha256,
            ..
        } if name == "native-runtime" => {
            release == RUST_COMPONENT_RELEASE && content_sha256 == RUST_NATIVE_RUNTIME_SHA256
        }
        ToolchainComponent::Content {
            name,
            release,
            content_sha256,
            ..
        } if name == "rust-compiler-runtime" => {
            release == RUST_COMPONENT_RELEASE && content_sha256 == RUST_COMPILER_RUNTIME_SHA256
        }
        ToolchainComponent::Content {
            name,
            release,
            content_sha256,
            ..
        } if name == "rust-target-i686" => {
            release == RUST_COMPONENT_RELEASE && content_sha256 == RUST_TARGET_I686_SHA256
        }
        ToolchainComponent::Content {
            name,
            release,
            content_sha256,
            ..
        } if name == "rust-target-x86_64" => {
            release == RUST_COMPONENT_RELEASE && content_sha256 == RUST_TARGET_X86_64_SHA256
        }
        _ => false,
    })
}

fn validate_csharp_runtime_linkage(
    toolchain: &SuccessorToolchainBundle,
    layouts: &[NativeRuntimeLayoutProfile],
    root_files: &BTreeMap<&str, &InventoryFile>,
) -> Result<(), SuccessorReleaseError> {
    let layout = layouts
        .iter()
        .find(|layout| layout.id == CSHARP_RUNTIME_LAYOUT_ID)
        .ok_or_else(inventory_failure)?;
    let native_inventory = toolchain
        .components
        .iter()
        .find_map(|component| match component {
            ToolchainComponent::Content {
                name, inventory, ..
            } if name == "native-runtime" => Some(inventory),
            _ => None,
        })
        .ok_or_else(inventory_failure)?;
    let native_files = inventory_files(native_inventory);
    let runtime = toolchain
        .components
        .iter()
        .find_map(|component| match component {
            ToolchainComponent::Executable { name, runtime, .. } if name == "dotnet" => {
                Some(runtime)
            }
            _ => None,
        })
        .ok_or_else(inventory_failure)?;
    validate_dynamic_runtime_linkage(runtime, layout, &native_files, root_files)
}

fn validate_dynamic_runtime_linkage(
    runtime: &ExecutableRuntime,
    layout: &NativeRuntimeLayoutProfile,
    native_files: &BTreeMap<&str, &InventoryFile>,
    root_files: &BTreeMap<&str, &InventoryFile>,
) -> Result<(), SuccessorReleaseError> {
    let ExecutableRuntime::Dynamic {
        interpreter_mount,
        libraries,
    } = runtime
    else {
        return Err(inventory_failure());
    };
    let interpreter = layout
        .interpreter_mounts
        .iter()
        .find(|mount| mount.sandbox_path == *interpreter_mount)
        .ok_or_else(inventory_failure)?;
    let interpreter_path = format!("native-runtime/{}", interpreter.component_path);
    if native_files.get(interpreter_path.as_str()) != root_files.get(interpreter_path.as_str()) {
        return Err(inventory_failure());
    }
    for library in libraries {
        if library.component_path.rsplit('/').next() != Some(library.soname.as_str()) {
            return Err(inventory_failure());
        }
        let path = format!("native-runtime/{}", library.component_path);
        let file = native_files
            .get(path.as_str())
            .copied()
            .ok_or_else(inventory_failure)?;
        if root_files.get(path.as_str()).copied() != Some(file)
            || file.sha256 != library.sha256
            || !layout.library_mounts.iter().any(|mount| {
                library.component_path == mount.component_path
                    || library
                        .component_path
                        .strip_prefix(mount.component_path.as_str())
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
        {
            return Err(inventory_failure());
        }
    }
    Ok(())
}

fn validate_rust_runtime_linkage(
    frontend: &SuccessorFrontendBundle,
    toolchain: &SuccessorToolchainBundle,
    layouts: &[NativeRuntimeLayoutProfile],
) -> Result<(), SuccessorReleaseError> {
    let layout = layouts
        .iter()
        .find(|layout| layout.id == RUST_RUNTIME_LAYOUT_ID)
        .ok_or_else(inventory_failure)?;
    let native_inventory = toolchain
        .components
        .iter()
        .find_map(|component| match component {
            ToolchainComponent::Content {
                name, inventory, ..
            } if name == "native-runtime" => Some(inventory),
            _ => None,
        })
        .ok_or_else(inventory_failure)?;
    let native_files = inventory_files(native_inventory);
    let root_files = inventory_files(&toolchain.inventory);

    validate_dynamic_runtime_linkage(&frontend.main.runtime, layout, &native_files, &root_files)?;
    for binary in &frontend.subordinate_binaries {
        validate_dynamic_runtime_linkage(&binary.runtime, layout, &native_files, &root_files)?;
    }
    for component in &toolchain.components {
        if let ToolchainComponent::Executable { runtime, .. } = component {
            validate_dynamic_runtime_linkage(runtime, layout, &native_files, &root_files)?;
        }
    }
    Ok(())
}

fn validate_profile_contracts(
    contracts: &[Value],
    semantic_registry: &ValidatedSemanticProfileRegistry,
    field: ProfileContractField,
) -> Result<(), SuccessorReleaseError> {
    if contracts.is_empty() {
        return Err(contract_failure());
    }
    let mut previous: Option<String> = None;
    for contract in contracts {
        let validated = validate_compiled_profile_envelope(semantic_registry, contract, field)
            .map_err(|_| contract_failure())?;
        if previous
            .as_deref()
            .is_some_and(|value| value >= validated.profile_entry_sha256())
        {
            return Err(failure(
                SuccessorReleaseValidationPhase::Order,
                SuccessorReleaseErrorCode::Order,
            ));
        }
        previous = Some(validated.profile_entry_sha256().to_owned());
    }
    Ok(())
}

fn profile_contract<'a>(
    contracts: &'a [Value],
    context: &SemanticContext,
    semantic_registry: &ValidatedSemanticProfileRegistry,
    field: ProfileContractField,
) -> Result<&'a serde_json::Map<String, Value>, SuccessorReleaseError> {
    let expected_id = semantic_registry
        .lookup(context.source_language(), context.semantic_profile())
        .ok_or_else(contract_failure)?
        .contracts()
        .contract_id(field);
    let mut matching = contracts.iter().filter(|contract| {
        contract.get("profile_entry_sha256").and_then(Value::as_str)
            == Some(context.profile_entry_sha256())
    });
    let contract = matching.next().ok_or_else(contract_failure)?;
    if matching.next().is_some()
        || contract.get("contract_id").and_then(Value::as_str) != Some(expected_id)
    {
        return Err(contract_failure());
    }
    contract
        .get("value")
        .and_then(Value::as_object)
        .ok_or_else(contract_failure)
}

fn validate_go_release_contract(
    toolchain: &SuccessorToolchainBundle,
    contract: &serde_json::Map<String, Value>,
) -> Result<(), SuccessorReleaseError> {
    let compiler = contract
        .get("compiler")
        .and_then(Value::as_object)
        .ok_or_else(linkage_failure)?;
    let native_runtime = contract
        .get("native_runtime")
        .and_then(Value::as_object)
        .ok_or_else(linkage_failure)?;
    if toolchain.bundle_id != GO_STAGING_TOOLCHAIN_BUNDLE_ID
        || compiler.get("kind").and_then(Value::as_str) != Some("go")
        || compiler.get("release").and_then(Value::as_str) != Some("go1.25.0")
        || native_runtime.get("kind").and_then(Value::as_str) != Some("none")
    {
        return Err(linkage_failure());
    }
    let Some(targets) = contract.get("target_libraries").and_then(Value::as_array) else {
        return Err(linkage_failure());
    };
    if targets.len() != 1
        || targets[0].get("target_id").and_then(Value::as_str) != Some("linux/amd64")
        || targets[0].get("pointer_width").and_then(Value::as_u64) != Some(64)
    {
        return Err(linkage_failure());
    }
    let component_name = targets[0]
        .get("component_name")
        .and_then(Value::as_str)
        .ok_or_else(linkage_failure)?;
    let content_sha256 = targets[0]
        .get("content_sha256")
        .and_then(Value::as_str)
        .ok_or_else(linkage_failure)?;
    let mut found_go = false;
    let mut found_target = false;
    for component in &toolchain.components {
        match component {
            ToolchainComponent::Executable {
                name,
                release,
                runtime,
                ..
            } => {
                if release != "go1.25.0" || !matches!(runtime, ExecutableRuntime::Static) {
                    return Err(linkage_failure());
                }
                if name == "go" {
                    found_go = true;
                }
            }
            ToolchainComponent::Content {
                name,
                release,
                content_sha256: actual,
                ..
            } if name == component_name => {
                found_target = release == "go1.25.0" && actual == content_sha256;
            }
            ToolchainComponent::Content { release, .. } => {
                if release != "go1.25.0" {
                    return Err(linkage_failure());
                }
            }
        }
    }
    if !found_go || !found_target {
        return Err(linkage_failure());
    }
    Ok(())
}

fn validate_rust_release_contract(
    toolchain: &SuccessorToolchainBundle,
    contract: &serde_json::Map<String, Value>,
) -> Result<(), SuccessorReleaseError> {
    let compiler = contract
        .get("compiler")
        .and_then(Value::as_object)
        .ok_or_else(linkage_failure)?;
    let native_runtime = contract
        .get("native_runtime")
        .and_then(Value::as_object)
        .ok_or_else(linkage_failure)?;
    if toolchain.bundle_id != RUST_STAGING_TOOLCHAIN_BUNDLE_ID
        || compiler.get("kind").and_then(Value::as_str) != Some("rust")
        || compiler.get("release").and_then(Value::as_str) != Some(RUST_TOOLCHAIN_RELEASE)
        || compiler.get("rustc_commit").and_then(Value::as_str)
            != Some("4d08223c054cf5a56d9761ca925fd46ffebe7115")
        || native_runtime.get("kind").and_then(Value::as_str) != Some("component")
        || native_runtime.get("component_name").and_then(Value::as_str) != Some("native-runtime")
        || native_runtime.get("component_root").and_then(Value::as_str) != Some("native-runtime")
        || native_runtime
            .get("layout_profile_id")
            .and_then(Value::as_str)
            != Some(RUST_RUNTIME_LAYOUT_ID)
    {
        return Err(linkage_failure());
    }

    let Some(targets) = contract.get("target_libraries").and_then(Value::as_array) else {
        return Err(linkage_failure());
    };
    let expected = [
        (
            "rust-target-i686",
            RUST_TARGET_I686_SHA256,
            "i686-unknown-linux-gnu",
            32,
        ),
        (
            "rust-target-x86_64",
            RUST_TARGET_X86_64_SHA256,
            "x86_64-unknown-linux-gnu",
            64,
        ),
    ];
    if targets.len() != expected.len() {
        return Err(linkage_failure());
    }
    for (target, (component_name, content_sha256, target_id, pointer_width)) in
        targets.iter().zip(expected)
    {
        if target.get("component_name").and_then(Value::as_str) != Some(component_name)
            || target.get("content_sha256").and_then(Value::as_str) != Some(content_sha256)
            || target.get("target_id").and_then(Value::as_str) != Some(target_id)
            || target.get("pointer_width").and_then(Value::as_u64) != Some(pointer_width)
            || !toolchain.components.iter().any(|component| {
                matches!(
                    component,
                    ToolchainComponent::Content {
                        name,
                        content_sha256: actual,
                        ..
                    } if name == component_name && actual == content_sha256
                )
            })
        {
            return Err(linkage_failure());
        }
    }
    Ok(())
}

fn validate_inventory(
    inventory: &BundleInventory,
    expected_scope: InventoryScope,
) -> Result<(), SuccessorReleaseError> {
    if inventory.schema != "mpk.release.bundle_inventory.v0" || inventory.scope != expected_scope {
        return Err(inventory_failure());
    }
    let mut previous = None;
    let mut folded = BTreeSet::new();
    let mut total = 0_u64;
    for file in &inventory.files {
        if !portable_path(&file.path)
            || !lower_sha256(&file.sha256)
            || file.size_bytes < 0
            || u64::try_from(file.size_bytes).unwrap_or(u64::MAX) > BUNDLE_FILE_BYTES_MAX
            || previous.is_some_and(|value: &str| value >= file.path.as_str())
            || !folded.insert(file.path.to_ascii_lowercase())
        {
            return Err(inventory_failure());
        }
        total = total
            .checked_add(u64::try_from(file.size_bytes).map_err(|_| inventory_failure())?)
            .ok_or_else(inventory_failure)?;
        previous = Some(file.path.as_str());
    }
    if inventory.files.is_empty() || total == 0 {
        return Err(inventory_failure());
    }
    Ok(())
}

fn inventory_files(inventory: &BundleInventory) -> BTreeMap<&str, &InventoryFile> {
    inventory
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect()
}

fn inventory_hash(inventory: &BundleInventory) -> Result<String, SuccessorReleaseError> {
    let bytes = serde_json::to_vec(inventory).map_err(|_| inventory_failure())?;
    let strict = parse_strict_json(&bytes, TRANSPORT_LIMITS).map_err(|_| inventory_failure())?;
    let canonical = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).unwrap_or(usize::MAX),
    )
    .map_err(|_| inventory_failure())?;
    hash_domain_separated_raw(CONTENT_HASH_DOMAIN, &canonical)
        .map(|digest| digest.to_hex())
        .map_err(|_| inventory_failure())
}

fn validate_executable_record(
    executable: &ExecutableRecord,
    files: &BTreeMap<&str, &InventoryFile>,
    require_executable: bool,
) -> Result<(), SuccessorReleaseError> {
    let file = files
        .get(executable.path.as_str())
        .ok_or_else(inventory_failure)?;
    if (require_executable && !file.executable)
        || file.sha256 != executable.binary_sha256
        || !lower_sha256(&executable.binary_sha256)
        || executable.name.is_empty()
        || executable.version.is_empty()
    {
        return Err(inventory_failure());
    }
    validate_runtime(&executable.runtime)
}

fn validate_runtime(runtime: &ExecutableRuntime) -> Result<(), SuccessorReleaseError> {
    if let ExecutableRuntime::Dynamic {
        interpreter_mount,
        libraries,
    } = runtime
    {
        if interpreter_mount != "/lib64/ld-linux-x86-64.so.2"
            || libraries.is_empty()
            || !strictly_increasing(libraries.iter().map(|library| library.soname.as_str()))
            || libraries.iter().any(|library| {
                library.soname.is_empty()
                    || !portable_path(&library.component_path)
                    || !lower_sha256(&library.sha256)
            })
        {
            return Err(inventory_failure());
        }
    }
    Ok(())
}

fn portable_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= PORTABLE_PATH_BYTES_MAX as usize
        && value.is_ascii()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("//")
        && !value.contains('\\')
        && value
            .split('/')
            .all(|component| !matches!(component, "" | "." | ".."))
}

fn valid_identifier(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let mut separator = true;
    for byte in value.bytes() {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            separator = false;
        } else if matches!(byte, b'.' | b'_' | b'-') && !separator {
            separator = true;
        } else {
            return false;
        }
    }
    !separator
}

fn lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn strictly_increasing<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|prior: &str| prior.as_bytes() >= value.as_bytes()) {
            return false;
        }
        previous = Some(value);
    }
    true
}

const fn failure(
    phase: SuccessorReleaseValidationPhase,
    code: SuccessorReleaseErrorCode,
) -> SuccessorReleaseError {
    SuccessorReleaseError::new(phase, code)
}

fn inventory_failure() -> SuccessorReleaseError {
    failure(
        SuccessorReleaseValidationPhase::Inventory,
        SuccessorReleaseErrorCode::Inventory,
    )
}

fn contract_failure() -> SuccessorReleaseError {
    failure(
        SuccessorReleaseValidationPhase::Contract,
        SuccessorReleaseErrorCode::Contract,
    )
}

fn linkage_failure() -> SuccessorReleaseError {
    failure(
        SuccessorReleaseValidationPhase::Linkage,
        SuccessorReleaseErrorCode::Linkage,
    )
}

fn selection_failure() -> SuccessorReleaseError {
    failure(
        SuccessorReleaseValidationPhase::Linkage,
        SuccessorReleaseErrorCode::Selection,
    )
}
