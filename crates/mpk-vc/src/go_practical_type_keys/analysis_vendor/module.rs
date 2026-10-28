//! Module and checksum facts relative to the same untrusted analysis snapshot.
//! Cached .info metadata, source syntax and root/build authority remain separate.
use super::super::{
    derive_go_source_module_manifest, import_go_module_sums, module_observations::manifest_sum,
    GoModuleSums, GoSourceModuleDataError, GoSourceModuleManifest, GoSourceModuleManifestError,
    GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use super::archives::root_before;
use super::{go_space, GoAnalysisVendorSnapshotInputs};
use std::collections::{BTreeMap, BTreeSet};

const FRONTEND: &[u8] = b"github.com/finitefield-org/mpk/go-tools/go2vir";
const PINS: [(&str, &str, bool, &str); 3] = [
    (
        "golang.org/x/mod",
        "v0.27.0",
        false,
        "h1:rWI627Fq0DEoudcK+MBkNkCe0EetEaDSwJJkCcjpazc=",
    ),
    (
        "golang.org/x/sync",
        "v0.16.0",
        true,
        "h1:1dzgHSNfp02xaA81J2MS99Qcpr2w7fw1gpm99rleRqA=",
    ),
    (
        "golang.org/x/tools",
        "v0.36.0",
        false,
        "h1:WBDiHKJK8YgLHlcQPYQzNCkUxUypCaa5ZegCVutKm+s=",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoAnalysisModuleError {
    Snapshot(GoSourceSnapshotLookupError),
    Manifest(GoSourceModuleManifestError),
    Sums(GoSourceModuleDataError),
    Ownership,
    Pins,
    Checksum,
}

impl std::fmt::Display for GoAnalysisModuleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Go analysis module inputs: {self:?}")
    }
}
impl std::error::Error for GoAnalysisModuleError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisModuleInputs {
    vendor: GoAnalysisVendorSnapshotInputs,
    manifest: GoSourceModuleManifest,
    sums: GoModuleSums,
    cached_manifests: BTreeMap<String, GoSourceModuleManifest>,
    zip_hashes: BTreeMap<String, GoTypeSourceBuffer>,
}
impl GoAnalysisModuleInputs {
    pub fn vendor(&self) -> &GoAnalysisVendorSnapshotInputs {
        &self.vendor
    }
    pub fn manifest(&self) -> &GoSourceModuleManifest {
        &self.manifest
    }
    pub fn sums(&self) -> &GoModuleSums {
        &self.sums
    }
    pub fn cached_manifests(&self) -> &BTreeMap<String, GoSourceModuleManifest> {
        &self.cached_manifests
    }
    pub fn zip_hashes(&self) -> &BTreeMap<String, GoTypeSourceBuffer> {
        &self.zip_hashes
    }
}

/// Bind the original frontend go.mod/go.sum and cached .mod/.ziphash files to
/// the already checked vendor snapshot and original ZIPs. This establishes
/// consistency only: it does not authenticate the original filesystem, cached
/// .info records, source syntax, executable build or semantic context.
pub fn check_go_analysis_module_inputs(
    vendor: GoAnalysisVendorSnapshotInputs,
) -> Result<GoAnalysisModuleInputs, GoAnalysisModuleError> {
    use GoAnalysisModuleError as E;
    let snapshot = vendor.snapshot();
    let original = vendor.correspondence().manifest().original_source();
    let root = root_before(&original.object().path, "vendor/modules.txt").ok_or(E::Ownership)?;
    let frontend_path = |name: &str| {
        if root.is_empty() {
            name.to_owned()
        } else {
            format!("{root}/{name}")
        }
    };
    let capability = &original.object().capability;
    let manifest = derive_go_source_module_manifest(
        snapshot
            .read_source(capability, &frontend_path("go.mod"), "go_mod")
            .map_err(E::Snapshot)?
            .clone(),
        false,
    )
    .map_err(E::Manifest)?;
    let sums = import_go_module_sums(
        snapshot
            .read_source(capability, &frontend_path("go.sum"), "go_sum")
            .map_err(E::Snapshot)?
            .clone(),
    )
    .map_err(E::Sums)?;
    if manifest.module_path() != FRONTEND
        || manifest.go_requirement() != Some("1.23.0")
        || manifest.toolchain_suggestion().is_some()
        || manifest.requirements().len() != PINS.len()
        || !manifest.exclusions().is_empty()
        || !manifest.replacements().is_empty()
        || !manifest.retractions().is_empty()
    {
        return Err(E::Pins);
    }
    let mut seen = BTreeSet::new();
    for requirement in manifest.requirements() {
        let Some((name, version, indirect, _)) = PINS
            .iter()
            .find(|(name, _, _, _)| requirement.path == name.as_bytes())
        else {
            return Err(E::Pins);
        };
        if !seen.insert(*name)
            || requirement.version != *version
            || requirement.indirect != *indirect
        {
            return Err(E::Pins);
        }
    }
    let mut cached_manifests = BTreeMap::new();
    let mut zip_hashes = BTreeMap::new();
    for (name, version, _, pinned_mod_sum) in PINS {
        let archive = vendor
            .correspondence()
            .archives()
            .get(name)
            .ok_or(E::Ownership)?;
        if sums.checksum(name, version) != Some(archive.checksum())
            || sums.checksum(name, &format!("{version}/go.mod")) != Some(pinned_mod_sum)
        {
            return Err(E::Pins);
        }
        let source = archive.original_source();
        let prefix = source
            .object()
            .path
            .strip_suffix(".zip")
            .ok_or(E::Ownership)?;
        let cached = snapshot
            .read_source(
                &source.object().capability,
                &format!("{prefix}.mod"),
                "go_mod",
            )
            .map_err(E::Snapshot)?
            .clone();
        if manifest_sum(&cached) != pinned_mod_sum {
            return Err(E::Checksum);
        }
        if let Some(member) = archive.members().get("go.mod") {
            if member.raw_sha256 != cached.object().raw_sha256
                || member.bytes != cached.object().bytes
            {
                return Err(E::Checksum);
            }
        }
        let cached = derive_go_source_module_manifest(cached, false).map_err(E::Manifest)?;
        let zip_hash = snapshot
            .read_source(
                &source.object().capability,
                &format!("{prefix}.ziphash"),
                "metadata",
            )
            .map_err(E::Snapshot)?
            .clone();
        let text = std::str::from_utf8(zip_hash.bytes()).map_err(|_| E::Checksum)?;
        if text.trim_matches(go_space) != archive.checksum() {
            return Err(E::Checksum);
        }
        cached_manifests.insert(name.to_owned(), cached);
        zip_hashes.insert(name.to_owned(), zip_hash);
    }
    Ok(GoAnalysisModuleInputs {
        vendor,
        manifest,
        sums,
        cached_manifests,
        zip_hashes,
    })
}
