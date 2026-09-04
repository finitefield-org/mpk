//! Candidate-only successor release contract used by CSHARP-03-T02-W09.
//!
//! This module validates caller-injected descriptors. It neither reads the
//! installed release roots nor constructs a release candidate. T07 owns real
//! bundle materialization and T08 owns the atomic installed-image cutover.

use crate::csharp_practical_registry::{
    SuccessorProfileEntry, ValidatedSuccessorRegistry, FOUNDATION_DESCRIPTOR_CONTENT_SHA256,
    FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
};
use crate::{
    hash_domain_separated_raw, parse_strict_json, HashDomain, StrictJsonLimits,
    BUNDLE_DECLARED_BYTES_MAX, BUNDLE_FILE_BYTES_MAX, PORTABLE_PATH_BYTES_MAX,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const PRIVATE_RELEASE_WORK_ITEM: &str = "CSHARP-03-T02-W09";
pub const PRIVATE_RELEASE_REGISTRY_SCHEMA: &str = "mpk.release.bundle_registry.v2";
pub const PRIVATE_RELEASE_REGISTRY_ID: &str = "mpk.release.registry.v2";
pub const PRIVATE_FRONTEND_BUNDLE_SCHEMA: &str = "mpk.release.frontend_bundle.v2";
pub const PRIVATE_TOOLCHAIN_BUNDLE_SCHEMA: &str = "mpk.release.toolchain_bundle.v2";
pub const PRIVATE_BUNDLE_INVENTORY_SCHEMA: &str = "mpk.release.bundle_inventory.v1";
pub const PRIVATE_RELEASE_TUPLE_COUNT: usize = 5;

pub const PRIVATE_BUNDLE_CONTENT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-BUNDLE-CONTENT-1.0");
pub const PRIVATE_RELEASE_REGISTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-BUNDLE-REGISTRY-2.0");

const RELEASE_TRANSPORT_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(16_777_216, 16_777_216, 64, 1_048_576);
const BUNDLES_MAX: usize = 16;
const MEMBERS_PER_BUNDLE_MAX: usize = 4_096;

#[derive(Clone, Copy, Debug)]
pub struct PrivateReleaseMemberInput<'a> {
    pub path: &'a str,
    pub raw_sha256: &'a str,
    pub size_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct PrivateReleaseBundleInput<'a> {
    pub bundle_id: &'a str,
    pub semantic_profiles: &'a [&'a str],
    pub members: &'a [PrivateReleaseMemberInput<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryIdentity {
    schema: String,
    id: String,
    revision: u64,
    registry_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FoundationDescriptor {
    schema: String,
    id: String,
    content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleMember {
    path: String,
    raw_sha256: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleInventory {
    schema: String,
    members: Vec<BundleMember>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrontendBundle {
    schema: String,
    bundle_id: String,
    semantic_profiles: Vec<String>,
    profile_contracts: Vec<String>,
    inventory: BundleInventory,
    bundle_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ToolchainBundle {
    schema: String,
    bundle_id: String,
    semantic_profiles: Vec<String>,
    profile_contracts: Vec<String>,
    inventory: BundleInventory,
    distribution_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseTuple {
    source_language: String,
    semantic_profile: String,
    profile_entry_sha256: String,
    frontend_bundle_id: String,
    toolchain_bundle_id: String,
    foundation_descriptor: FoundationDescriptor,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseRegistry {
    schema: String,
    id: String,
    profile_registry: RegistryIdentity,
    foundation_descriptor: FoundationDescriptor,
    frontend_bundles: Vec<FrontendBundle>,
    toolchain_bundles: Vec<ToolchainBundle>,
    tuples: Vec<ReleaseTuple>,
    registry_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPrivateReleaseRegistry {
    canonical_bytes: Vec<u8>,
    registry_sha256: String,
    tuple_profiles: Vec<String>,
    member_count: usize,
}

impl ValidatedPrivateReleaseRegistry {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn registry_sha256(&self) -> &str {
        &self.registry_sha256
    }

    pub fn tuple_profiles(&self) -> &[String] {
        &self.tuple_profiles
    }

    pub const fn member_count(&self) -> usize {
        self.member_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateReleasePhase {
    Transport,
    Shape,
    Identity,
    Inventory,
    Linkage,
    Hash,
    Canonical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateReleaseCode {
    Json,
    Shape,
    Identity,
    Inventory,
    Linkage,
    Hash,
    Canonical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateReleaseError {
    phase: PrivateReleasePhase,
    code: PrivateReleaseCode,
}

impl PrivateReleaseError {
    pub const fn phase(&self) -> PrivateReleasePhase {
        self.phase
    }

    pub const fn code(&self) -> PrivateReleaseCode {
        self.code
    }
}

impl fmt::Display for PrivateReleaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "private successor release {:?} at {:?}",
            self.code, self.phase
        )
    }
}

impl Error for PrivateReleaseError {}

/// Builds a deterministic, non-installed descriptor for consumer tests.
///
/// Each semantic profile must be claimed by exactly one frontend and one
/// toolchain input. The result is immediately passed through the same strict
/// validator used by consumers.
pub fn build_private_successor_release_fixture(
    registry: &ValidatedSuccessorRegistry,
    frontend_inputs: &[PrivateReleaseBundleInput<'_>],
    toolchain_inputs: &[PrivateReleaseBundleInput<'_>],
) -> Result<ValidatedPrivateReleaseRegistry, PrivateReleaseError> {
    let foundation = expected_foundation();
    let frontends = build_frontends(registry, frontend_inputs)?;
    let toolchains = build_toolchains(registry, toolchain_inputs)?;
    let tuples = expected_tuples(registry, &frontends, &toolchains)?;
    let identity = registry.identity();
    let mut wire = ReleaseRegistry {
        schema: PRIVATE_RELEASE_REGISTRY_SCHEMA.to_owned(),
        id: PRIVATE_RELEASE_REGISTRY_ID.to_owned(),
        profile_registry: RegistryIdentity {
            schema: identity.schema().to_owned(),
            id: identity.id().to_owned(),
            revision: identity.revision(),
            registry_sha256: identity.registry_sha256().to_owned(),
        },
        foundation_descriptor: foundation,
        frontend_bundles: frontends,
        toolchain_bundles: toolchains,
        tuples,
        registry_sha256: String::new(),
    };
    wire.registry_sha256 = release_registry_hash(&wire)?;
    let transport = serde_json::to_vec(&wire)
        .map_err(|_| failure(PrivateReleasePhase::Transport, PrivateReleaseCode::Json))?;
    validate_private_successor_release_registry(&transport, registry)
}

/// Strictly consumes one test-injected successor release root.
pub fn validate_private_successor_release_registry(
    input: &[u8],
    registry: &ValidatedSuccessorRegistry,
) -> Result<ValidatedPrivateReleaseRegistry, PrivateReleaseError> {
    parse_strict_json(input, RELEASE_TRANSPORT_LIMITS)
        .map_err(|_| failure(PrivateReleasePhase::Transport, PrivateReleaseCode::Json))?;
    let wire: ReleaseRegistry = serde_json::from_slice(input)
        .map_err(|_| failure(PrivateReleasePhase::Shape, PrivateReleaseCode::Shape))?;
    let reencoded = serde_json::to_vec(&wire).map_err(|_| {
        failure(
            PrivateReleasePhase::Canonical,
            PrivateReleaseCode::Canonical,
        )
    })?;
    if reencoded != input {
        return Err(failure(
            PrivateReleasePhase::Canonical,
            PrivateReleaseCode::Canonical,
        ));
    }
    validate_registry_identity(&wire, registry)?;
    validate_bundles(registry, &wire.frontend_bundles, &wire.toolchain_bundles)?;
    let expected = expected_tuples(registry, &wire.frontend_bundles, &wire.toolchain_bundles)?;
    if wire.tuples != expected || wire.tuples.len() != PRIVATE_RELEASE_TUPLE_COUNT {
        return Err(failure(
            PrivateReleasePhase::Linkage,
            PrivateReleaseCode::Linkage,
        ));
    }
    if !valid_sha256(&wire.registry_sha256) || release_registry_hash(&wire)? != wire.registry_sha256
    {
        return Err(failure(PrivateReleasePhase::Hash, PrivateReleaseCode::Hash));
    }
    let tuple_profiles = wire
        .tuples
        .iter()
        .map(|tuple| tuple.semantic_profile.clone())
        .collect();
    let member_count = wire
        .frontend_bundles
        .iter()
        .map(|bundle| bundle.inventory.members.len())
        .sum::<usize>()
        + wire
            .toolchain_bundles
            .iter()
            .map(|bundle| bundle.inventory.members.len())
            .sum::<usize>();
    Ok(ValidatedPrivateReleaseRegistry {
        canonical_bytes: input.to_vec(),
        registry_sha256: wire.registry_sha256,
        tuple_profiles,
        member_count,
    })
}

fn build_frontends(
    registry: &ValidatedSuccessorRegistry,
    inputs: &[PrivateReleaseBundleInput<'_>],
) -> Result<Vec<FrontendBundle>, PrivateReleaseError> {
    let mut output = inputs
        .iter()
        .map(|input| {
            let profiles = normalize_profiles(registry, input.semantic_profiles)?;
            let contracts = contracts_for_profiles(registry, &profiles)?;
            let inventory = build_inventory(input.members)?;
            let mut bundle = FrontendBundle {
                schema: PRIVATE_FRONTEND_BUNDLE_SCHEMA.to_owned(),
                bundle_id: input.bundle_id.to_owned(),
                semantic_profiles: profiles,
                profile_contracts: contracts,
                inventory,
                bundle_sha256: String::new(),
            };
            validate_bundle_id(&bundle.bundle_id)?;
            bundle.bundle_sha256 = frontend_hash(&bundle)?;
            Ok(bundle)
        })
        .collect::<Result<Vec<_>, PrivateReleaseError>>()?;
    output.sort_by(|left, right| left.bundle_id.cmp(&right.bundle_id));
    require_unique(output.iter().map(|bundle| bundle.bundle_id.as_str()))?;
    Ok(output)
}

fn build_toolchains(
    registry: &ValidatedSuccessorRegistry,
    inputs: &[PrivateReleaseBundleInput<'_>],
) -> Result<Vec<ToolchainBundle>, PrivateReleaseError> {
    let mut output = inputs
        .iter()
        .map(|input| {
            let profiles = normalize_profiles(registry, input.semantic_profiles)?;
            let contracts = contracts_for_profiles(registry, &profiles)?;
            let inventory = build_inventory(input.members)?;
            let mut bundle = ToolchainBundle {
                schema: PRIVATE_TOOLCHAIN_BUNDLE_SCHEMA.to_owned(),
                bundle_id: input.bundle_id.to_owned(),
                semantic_profiles: profiles,
                profile_contracts: contracts,
                inventory,
                distribution_sha256: String::new(),
            };
            validate_bundle_id(&bundle.bundle_id)?;
            bundle.distribution_sha256 = toolchain_hash(&bundle)?;
            Ok(bundle)
        })
        .collect::<Result<Vec<_>, PrivateReleaseError>>()?;
    output.sort_by(|left, right| left.bundle_id.cmp(&right.bundle_id));
    require_unique(output.iter().map(|bundle| bundle.bundle_id.as_str()))?;
    Ok(output)
}

fn normalize_profiles(
    registry: &ValidatedSuccessorRegistry,
    profiles: &[&str],
) -> Result<Vec<String>, PrivateReleaseError> {
    let mut normalized = profiles
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    if normalized.len() != profiles.len()
        || normalized.is_empty()
        || normalized.iter().any(|profile| {
            registry
                .entries()
                .iter()
                .all(|entry| entry.semantic_profile() != profile)
        })
    {
        return Err(failure(
            PrivateReleasePhase::Linkage,
            PrivateReleaseCode::Linkage,
        ));
    }
    Ok(normalized)
}

fn contracts_for_profiles(
    registry: &ValidatedSuccessorRegistry,
    profiles: &[String],
) -> Result<Vec<String>, PrivateReleaseError> {
    let mut contracts = BTreeSet::new();
    for profile in profiles {
        let entry = registry
            .entries()
            .iter()
            .find(|entry| entry.semantic_profile() == profile)
            .ok_or_else(|| failure(PrivateReleasePhase::Linkage, PrivateReleaseCode::Linkage))?;
        contracts.extend(entry.contracts().iter().map(|(_, id)| id.to_owned()));
    }
    Ok(contracts.into_iter().collect())
}

fn build_inventory(
    inputs: &[PrivateReleaseMemberInput<'_>],
) -> Result<BundleInventory, PrivateReleaseError> {
    if inputs.is_empty() || inputs.len() > MEMBERS_PER_BUNDLE_MAX {
        return Err(failure(
            PrivateReleasePhase::Inventory,
            PrivateReleaseCode::Inventory,
        ));
    }
    let mut members = inputs
        .iter()
        .map(|input| BundleMember {
            path: input.path.to_owned(),
            raw_sha256: input.raw_sha256.to_owned(),
            size_bytes: input.size_bytes,
        })
        .collect::<Vec<_>>();
    members.sort_by(|left, right| left.path.cmp(&right.path));
    if !valid_members(&members) {
        return Err(failure(
            PrivateReleasePhase::Inventory,
            PrivateReleaseCode::Inventory,
        ));
    }
    Ok(BundleInventory {
        schema: PRIVATE_BUNDLE_INVENTORY_SCHEMA.to_owned(),
        members,
    })
}

fn validate_registry_identity(
    wire: &ReleaseRegistry,
    registry: &ValidatedSuccessorRegistry,
) -> Result<(), PrivateReleaseError> {
    let identity = registry.identity();
    if wire.schema != PRIVATE_RELEASE_REGISTRY_SCHEMA
        || wire.id != PRIVATE_RELEASE_REGISTRY_ID
        || wire.profile_registry.schema != identity.schema()
        || wire.profile_registry.id != identity.id()
        || wire.profile_registry.revision != identity.revision()
        || wire.profile_registry.registry_sha256 != identity.registry_sha256()
        || wire.foundation_descriptor != expected_foundation()
    {
        return Err(failure(
            PrivateReleasePhase::Identity,
            PrivateReleaseCode::Identity,
        ));
    }
    Ok(())
}

fn validate_bundles(
    registry: &ValidatedSuccessorRegistry,
    frontends: &[FrontendBundle],
    toolchains: &[ToolchainBundle],
) -> Result<(), PrivateReleaseError> {
    if frontends.is_empty()
        || toolchains.is_empty()
        || frontends.len() > BUNDLES_MAX
        || toolchains.len() > BUNDLES_MAX
        || !strictly_increasing(frontends.iter().map(|bundle| bundle.bundle_id.as_str()))
        || !strictly_increasing(toolchains.iter().map(|bundle| bundle.bundle_id.as_str()))
    {
        return Err(failure(
            PrivateReleasePhase::Inventory,
            PrivateReleaseCode::Inventory,
        ));
    }
    let expected_profiles = registry
        .entries()
        .iter()
        .map(|entry| entry.semantic_profile().to_owned())
        .collect::<BTreeSet<_>>();
    for bundle in frontends {
        if bundle.schema != PRIVATE_FRONTEND_BUNDLE_SCHEMA
            || !valid_bundle_id(&bundle.bundle_id)
            || !valid_profile_projection(
                registry,
                &bundle.semantic_profiles,
                &bundle.profile_contracts,
            )
            || !valid_inventory(&bundle.inventory)
            || !valid_sha256(&bundle.bundle_sha256)
            || frontend_hash(bundle)? != bundle.bundle_sha256
        {
            return Err(failure(PrivateReleasePhase::Hash, PrivateReleaseCode::Hash));
        }
    }
    for bundle in toolchains {
        if bundle.schema != PRIVATE_TOOLCHAIN_BUNDLE_SCHEMA
            || !valid_bundle_id(&bundle.bundle_id)
            || !valid_profile_projection(
                registry,
                &bundle.semantic_profiles,
                &bundle.profile_contracts,
            )
            || !valid_inventory(&bundle.inventory)
            || !valid_sha256(&bundle.distribution_sha256)
            || toolchain_hash(bundle)? != bundle.distribution_sha256
        {
            return Err(failure(PrivateReleasePhase::Hash, PrivateReleaseCode::Hash));
        }
    }
    let frontend_profiles = exact_claimed_profiles(
        frontends
            .iter()
            .flat_map(|bundle| bundle.semantic_profiles.iter()),
    )?;
    let toolchain_profiles = exact_claimed_profiles(
        toolchains
            .iter()
            .flat_map(|bundle| bundle.semantic_profiles.iter()),
    )?;
    if frontend_profiles != expected_profiles || toolchain_profiles != expected_profiles {
        return Err(failure(
            PrivateReleasePhase::Linkage,
            PrivateReleaseCode::Linkage,
        ));
    }
    Ok(())
}

fn expected_tuples(
    registry: &ValidatedSuccessorRegistry,
    frontends: &[FrontendBundle],
    toolchains: &[ToolchainBundle],
) -> Result<Vec<ReleaseTuple>, PrivateReleaseError> {
    let mut tuples = registry
        .entries()
        .iter()
        .map(|entry| {
            Ok(ReleaseTuple {
                source_language: entry.source_language().to_owned(),
                semantic_profile: entry.semantic_profile().to_owned(),
                profile_entry_sha256: entry.entry_sha256().to_owned(),
                frontend_bundle_id: sole_frontend(entry, frontends)?.bundle_id.clone(),
                toolchain_bundle_id: sole_toolchain(entry, toolchains)?.bundle_id.clone(),
                foundation_descriptor: expected_foundation(),
            })
        })
        .collect::<Result<Vec<_>, PrivateReleaseError>>()?;
    tuples.sort_by(|left, right| {
        (&left.source_language, &left.semantic_profile)
            .cmp(&(&right.source_language, &right.semantic_profile))
    });
    Ok(tuples)
}

fn sole_frontend<'a>(
    entry: &SuccessorProfileEntry,
    bundles: &'a [FrontendBundle],
) -> Result<&'a FrontendBundle, PrivateReleaseError> {
    sole_bundle(
        entry.semantic_profile(),
        bundles.iter().filter(|bundle| {
            bundle
                .semantic_profiles
                .iter()
                .any(|value| value == entry.semantic_profile())
        }),
    )
}

fn sole_toolchain<'a>(
    entry: &SuccessorProfileEntry,
    bundles: &'a [ToolchainBundle],
) -> Result<&'a ToolchainBundle, PrivateReleaseError> {
    sole_bundle(
        entry.semantic_profile(),
        bundles.iter().filter(|bundle| {
            bundle
                .semantic_profiles
                .iter()
                .any(|value| value == entry.semantic_profile())
        }),
    )
}

fn sole_bundle<'a, T>(
    _profile: &str,
    mut matches: impl Iterator<Item = &'a T>,
) -> Result<&'a T, PrivateReleaseError> {
    let first = matches
        .next()
        .ok_or_else(|| failure(PrivateReleasePhase::Linkage, PrivateReleaseCode::Linkage))?;
    if matches.next().is_some() {
        Err(failure(
            PrivateReleasePhase::Linkage,
            PrivateReleaseCode::Linkage,
        ))
    } else {
        Ok(first)
    }
}

fn valid_profile_projection(
    registry: &ValidatedSuccessorRegistry,
    profiles: &[String],
    contracts: &[String],
) -> bool {
    if profiles.is_empty()
        || !strictly_increasing(profiles.iter().map(String::as_str))
        || !strictly_increasing(contracts.iter().map(String::as_str))
    {
        return false;
    }
    contracts_for_profiles(registry, profiles).is_ok_and(|expected| expected == contracts)
}

fn valid_inventory(inventory: &BundleInventory) -> bool {
    inventory.schema == PRIVATE_BUNDLE_INVENTORY_SCHEMA
        && !inventory.members.is_empty()
        && inventory.members.len() <= MEMBERS_PER_BUNDLE_MAX
        && valid_members(&inventory.members)
}

fn valid_members(members: &[BundleMember]) -> bool {
    if !strictly_increasing(members.iter().map(|member| member.path.as_str())) {
        return false;
    }
    let mut folded_paths = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for member in members {
        if !valid_member_path(&member.path)
            || !folded_paths.insert(member.path.to_ascii_lowercase())
            || !valid_sha256(&member.raw_sha256)
            || member.size_bytes == 0
            || member.size_bytes > BUNDLE_FILE_BYTES_MAX
        {
            return false;
        }
        let Some(next_total) = total_bytes.checked_add(member.size_bytes) else {
            return false;
        };
        if next_total > BUNDLE_DECLARED_BYTES_MAX {
            return false;
        }
        total_bytes = next_total;
    }
    true
}

fn exact_claimed_profiles<'a>(
    profiles: impl Iterator<Item = &'a String>,
) -> Result<BTreeSet<String>, PrivateReleaseError> {
    let mut set = BTreeSet::new();
    for profile in profiles {
        if !set.insert(profile.clone()) {
            return Err(failure(
                PrivateReleasePhase::Linkage,
                PrivateReleaseCode::Linkage,
            ));
        }
    }
    Ok(set)
}

fn expected_foundation() -> FoundationDescriptor {
    FoundationDescriptor {
        schema: FOUNDATION_DESCRIPTOR_SCHEMA.to_owned(),
        id: FOUNDATION_DESCRIPTOR_ID.to_owned(),
        content_sha256: FOUNDATION_DESCRIPTOR_CONTENT_SHA256.to_owned(),
    }
}

fn frontend_hash(bundle: &FrontendBundle) -> Result<String, PrivateReleaseError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        bundle_id: &'a str,
        semantic_profiles: &'a [String],
        profile_contracts: &'a [String],
        inventory: &'a BundleInventory,
    }
    hash_serialized(
        PRIVATE_BUNDLE_CONTENT_HASH_DOMAIN,
        &Preimage {
            schema: &bundle.schema,
            bundle_id: &bundle.bundle_id,
            semantic_profiles: &bundle.semantic_profiles,
            profile_contracts: &bundle.profile_contracts,
            inventory: &bundle.inventory,
        },
    )
}

fn toolchain_hash(bundle: &ToolchainBundle) -> Result<String, PrivateReleaseError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        bundle_id: &'a str,
        semantic_profiles: &'a [String],
        profile_contracts: &'a [String],
        inventory: &'a BundleInventory,
    }
    hash_serialized(
        PRIVATE_BUNDLE_CONTENT_HASH_DOMAIN,
        &Preimage {
            schema: &bundle.schema,
            bundle_id: &bundle.bundle_id,
            semantic_profiles: &bundle.semantic_profiles,
            profile_contracts: &bundle.profile_contracts,
            inventory: &bundle.inventory,
        },
    )
}

fn release_registry_hash(wire: &ReleaseRegistry) -> Result<String, PrivateReleaseError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        id: &'a str,
        profile_registry: &'a RegistryIdentity,
        foundation_descriptor: &'a FoundationDescriptor,
        frontend_bundles: &'a [FrontendBundle],
        toolchain_bundles: &'a [ToolchainBundle],
        tuples: &'a [ReleaseTuple],
    }
    hash_serialized(
        PRIVATE_RELEASE_REGISTRY_HASH_DOMAIN,
        &Preimage {
            schema: &wire.schema,
            id: &wire.id,
            profile_registry: &wire.profile_registry,
            foundation_descriptor: &wire.foundation_descriptor,
            frontend_bundles: &wire.frontend_bundles,
            toolchain_bundles: &wire.toolchain_bundles,
            tuples: &wire.tuples,
        },
    )
}

fn hash_serialized<T: Serialize>(
    domain: HashDomain,
    value: &T,
) -> Result<String, PrivateReleaseError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| failure(PrivateReleasePhase::Hash, PrivateReleaseCode::Hash))?;
    hash_domain_separated_raw(domain, &bytes)
        .map(|hash| hash.to_hex())
        .map_err(|_| failure(PrivateReleasePhase::Hash, PrivateReleaseCode::Hash))
}

fn validate_bundle_id(value: &str) -> Result<(), PrivateReleaseError> {
    if valid_bundle_id(value) {
        Ok(())
    } else {
        Err(failure(
            PrivateReleasePhase::Identity,
            PrivateReleaseCode::Identity,
        ))
    }
}

fn valid_bundle_id(value: &str) -> bool {
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

fn valid_member_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= PORTABLE_PATH_BYTES_MAX as usize
        && value.is_ascii()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), PrivateReleaseError> {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|prior| prior >= value) {
            return Err(failure(
                PrivateReleasePhase::Inventory,
                PrivateReleaseCode::Inventory,
            ));
        }
        previous = Some(value);
    }
    Ok(())
}

fn strictly_increasing<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let Some(mut previous) = values.next() else {
        return true;
    };
    for value in values {
        if previous >= value {
            return false;
        }
        previous = value;
    }
    true
}

const fn failure(phase: PrivateReleasePhase, code: PrivateReleaseCode) -> PrivateReleaseError {
    PrivateReleaseError { phase, code }
}
