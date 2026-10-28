//! Analysis vendor enumeration relative to one complete observed snapshot.
//! A caller-supplied snapshot is not proof of an authenticated original root.
use super::super::{GoSourceSnapshotInputs, GoSourceSnapshotLookupError, GoTypeSourceBuffer};
use super::archives::root_before;
use super::{
    check_go_analysis_vendor_archive_bytes, GoAnalysisVendorArchiveError,
    GoAnalysisVendorArchiveInputs, GoAnalysisVendorManifest,
};

const DIRECTORY_TYPE: u32 = 1 << 31;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoAnalysisVendorSnapshotError {
    Snapshot(GoSourceSnapshotLookupError),
    Namespace,
    Mismatch,
    Archive(GoAnalysisVendorArchiveError),
}

impl std::fmt::Display for GoAnalysisVendorSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Go analysis vendor snapshot inputs: {self:?}")
    }
}

impl std::error::Error for GoAnalysisVendorSnapshotError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisVendorSnapshotInputs {
    snapshot: GoSourceSnapshotInputs,
    correspondence: GoAnalysisVendorArchiveInputs,
}

impl GoAnalysisVendorSnapshotInputs {
    pub fn snapshot(&self) -> &GoSourceSnapshotInputs {
        &self.snapshot
    }
    pub fn correspondence(&self) -> &GoAnalysisVendorArchiveInputs {
        &self.correspondence
    }
}

/// Derive every vendor file from recorded directory entries. The snapshot
/// remains an untrusted input; a coherently forged smaller namespace can only
/// be excluded by a separately authenticated capture of the original roots.
pub fn check_go_analysis_vendor_snapshot(
    snapshot: GoSourceSnapshotInputs,
    manifest: GoAnalysisVendorManifest,
    archives: Vec<GoTypeSourceBuffer>,
) -> Result<GoAnalysisVendorSnapshotInputs, GoAnalysisVendorSnapshotError> {
    use GoAnalysisVendorSnapshotError as E;
    let original = manifest.original_source();
    let capability = original.object().capability.clone();
    let manifest_path = original.object().path.clone();
    let root = root_before(&manifest_path, "vendor/modules.txt")
        .ok_or(E::Namespace)?
        .to_owned();
    let observed = snapshot
        .read_source(&capability, &manifest_path, "metadata")
        .map_err(E::Snapshot)?;
    if observed != original {
        return Err(E::Mismatch);
    }
    for archive in &archives {
        let observed = snapshot
            .read_source(
                &archive.object().capability,
                &archive.object().path,
                "auxiliary",
            )
            .map_err(E::Snapshot)?;
        if observed != archive {
            return Err(E::Mismatch);
        }
    }
    let vendor_root = if root.is_empty() {
        "vendor".to_owned()
    } else {
        format!("{root}/vendor")
    };
    let mut directories = vec![vendor_root];
    let mut files = Vec::new();
    while let Some(directory) = directories.pop() {
        let entries = snapshot
            .read_directory(&capability, &directory)
            .map_err(E::Snapshot)?;
        for entry in entries {
            let name = std::str::from_utf8(entry.name_bytes()).map_err(|_| E::Namespace)?;
            let path = format!("{directory}/{name}");
            if entry.type_bits() == DIRECTORY_TYPE {
                directories.push(path);
                continue;
            }
            if entry.type_bits() != 0 {
                return Err(E::Snapshot(GoSourceSnapshotLookupError::Type));
            }
            if path == manifest_path {
                continue;
            }
            let kind = if name.ends_with(".go") {
                "go_file"
            } else if matches!(name, "LICENSE" | "PATENTS" | "TODO") {
                "auxiliary"
            } else {
                return Err(E::Archive(GoAnalysisVendorArchiveError::Source));
            };
            files.push(
                snapshot
                    .read_source(&capability, &path, kind)
                    .map_err(E::Snapshot)?
                    .clone(),
            );
        }
    }
    let correspondence =
        check_go_analysis_vendor_archive_bytes(manifest, archives, files).map_err(E::Archive)?;
    Ok(GoAnalysisVendorSnapshotInputs {
        snapshot,
        correspondence,
    })
}
