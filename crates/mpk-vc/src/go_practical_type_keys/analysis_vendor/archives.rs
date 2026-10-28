//! Content checks over supplied original analysis archives and vendor files.
//! Closed namespace, source syntax, binary reproduction and context authority
//! are separate requirements; this type does not assert them.
use super::super::{inspect_go_module_archive, GoModuleArchiveInventory, GoTypeSourceBuffer};
use super::GoAnalysisVendorManifest;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const DIRECTORY_ENTRIES_MAX: usize = 131_072;
const CAPTURE_FILES_MAX: usize = 32_768;
const PINS: [(&str, &str, &str); 3] = [
    (
        "golang.org/x/mod",
        "v0.27.0",
        "h1:kb+q2PyFnEADO2IEF935ehFUXlWiNjJWtRNgBLSfbxQ=",
    ),
    (
        "golang.org/x/sync",
        "v0.16.0",
        "h1:ycBJEhp9p4vXvUZNszeOq0kGTPghopOL8q0fq3vstxw=",
    ),
    (
        "golang.org/x/tools",
        "v0.36.0",
        "h1:kWS0uv/zsvHEle1LbV5LE8QujrxB3wfQyxHfhOk0Qkg=",
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoAnalysisVendorArchiveError {
    Source,
    Archive,
    Ownership,
    Content,
    MissingPackage,
    Limit,
}

impl std::fmt::Display for GoAnalysisVendorArchiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Go analysis vendor archive inputs: {self:?}")
    }
}

impl std::error::Error for GoAnalysisVendorArchiveError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisVendorAssetDifference {
    pub source_id: String,
    pub path: String,
    pub source_sha256: String,
    pub source_bytes: u64,
    pub archive_sha256: String,
    pub archive_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisVendorArchiveInputs {
    manifest: GoAnalysisVendorManifest,
    archives: BTreeMap<String, GoModuleArchiveInventory>,
    files: BTreeMap<String, GoTypeSourceBuffer>,
    differences: Vec<GoAnalysisVendorAssetDifference>,
}

impl GoAnalysisVendorArchiveInputs {
    pub fn manifest(&self) -> &GoAnalysisVendorManifest {
        &self.manifest
    }
    pub fn archives(&self) -> &BTreeMap<String, GoModuleArchiveInventory> {
        &self.archives
    }
    pub fn supplied_files(&self) -> &BTreeMap<String, GoTypeSourceBuffer> {
        &self.files
    }
    pub fn differences(&self) -> &[GoAnalysisVendorAssetDifference] {
        &self.differences
    }
}

// Roots are logical captured paths and may have a nonempty original directory.
pub(super) fn root_before<'a>(path: &'a str, suffix: &str) -> Option<&'a str> {
    if path == suffix {
        Some("")
    } else {
        path.strip_suffix(suffix)?.strip_suffix('/')
    }
}

fn beneath<'a>(path: &'a str, root: &str) -> Option<&'a str> {
    if root.is_empty() {
        Some(path)
    } else {
        path.strip_prefix(root)?.strip_prefix('/')
    }
}

/// Check full archive bytes and every supplied vendor file against the pinned
/// original module identities. This verifies only supplied-file correspondence:
/// it does not prove a complete filesystem namespace or Go source syntax.
pub fn check_go_analysis_vendor_archive_bytes(
    manifest: GoAnalysisVendorManifest,
    archive_sources: Vec<GoTypeSourceBuffer>,
    vendor_files: Vec<GoTypeSourceBuffer>,
) -> Result<GoAnalysisVendorArchiveInputs, GoAnalysisVendorArchiveError> {
    use GoAnalysisVendorArchiveError as E;
    if archive_sources.len() != PINS.len() {
        return Err(E::Archive);
    }
    if vendor_files.len() > CAPTURE_FILES_MAX {
        return Err(E::Limit);
    }
    let manifest_source = manifest.original_source();
    let frontend_root =
        root_before(&manifest_source.object().path, "vendor/modules.txt").ok_or(E::Source)?;
    let frontend_capability = &manifest_source.object().capability;
    let mut archives = BTreeMap::new();
    let mut archive_root: Option<&str> = None;
    let mut archive_capability: Option<&str> = None;
    let mut remaining = DIRECTORY_ENTRIES_MAX;
    for (name, version, checksum) in PINS {
        let suffix = format!("{name}/@v/{version}.zip");
        let mut matching = archive_sources.iter().filter_map(|source| {
            root_before(&source.object().path, &suffix).map(|root| (source, root))
        });
        let (source, root) = matching.next().ok_or(E::Archive)?;
        if matching.next().is_some()
            || archive_root.is_some_and(|old| old != root)
            || archive_capability.is_some_and(|old| old != source.object().capability)
        {
            return Err(E::Archive);
        }
        archive_root = Some(root);
        archive_capability = Some(&source.object().capability);
        let archive = inspect_go_module_archive(name, version, source, checksum, remaining)
            .map_err(|_| E::Archive)?;
        remaining = remaining
            .checked_sub(archive.namespace_entries())
            .ok_or(E::Limit)?;
        archives.insert(name.to_owned(), archive);
    }
    let mut files = BTreeMap::new();
    let mut seen_packages = BTreeSet::new();
    let mut differences = Vec::new();
    for source in vendor_files {
        if &source.object().capability != frontend_capability {
            return Err(E::Source);
        }
        let relative = beneath(&source.object().path, frontend_root)
            .and_then(|path| path.strip_prefix("vendor/"))
            .ok_or(E::Ownership)?;
        let (name, member_path) = PINS
            .iter()
            .find_map(|(name, _, _)| {
                relative
                    .strip_prefix(name)
                    .and_then(|tail| tail.strip_prefix('/'))
                    .map(|tail| (*name, tail))
            })
            .ok_or(E::Ownership)?;
        let filename = member_path.rsplit('/').next().ok_or(E::Ownership)?;
        let is_go = filename.ends_with(".go");
        if (is_go && (source.object().kind != "go_file" || filename.ends_with("_test.go")))
            || (!is_go
                && (source.object().kind != "auxiliary"
                    || !matches!(filename, "LICENSE" | "PATENTS" | "TODO")))
        {
            return Err(E::Source);
        }
        if is_go {
            let package = relative.rsplit_once('/').ok_or(E::Ownership)?.0;
            if !manifest.package_paths().contains(package) {
                return Err(E::Ownership);
            }
            seen_packages.insert(package.to_owned());
        }
        let member = archives
            .get(name)
            .and_then(|archive| archive.members().get(member_path))
            .ok_or(E::Ownership)?;
        if source.object().bytes != member.bytes || source.object().raw_sha256 != member.raw_sha256
        {
            if filename != "TODO" {
                return Err(E::Content);
            }
            differences.push(GoAnalysisVendorAssetDifference {
                source_id: source.object().id.clone(),
                path: source.object().path.clone(),
                source_sha256: source.object().raw_sha256.clone(),
                source_bytes: source.object().bytes,
                archive_sha256: member.raw_sha256.clone(),
                archive_bytes: member.bytes,
            });
        }
        if files.insert(source.object().path.clone(), source).is_some() {
            return Err(E::Ownership);
        }
    }
    if &seen_packages != manifest.package_paths() {
        return Err(E::MissingPackage);
    }
    Ok(GoAnalysisVendorArchiveInputs {
        manifest,
        archives,
        files,
        differences,
    })
}
