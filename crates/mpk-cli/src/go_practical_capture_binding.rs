//! Connect independently matched source roots to the installed compiler owner.
//!
//! The distinct export-cache root and launcher authorization remain separate;
//! this private owner cannot publish a practical context or source theorem.
use crate::go_practical_installed_compiler::{
    InstalledGoCompiler, InstalledGoCompilerError, InstalledGoSourceCompilerInputs,
};
use crate::go_practical_source_roots::GoSourceRootsMatch;
use mpk_vc::go_practical_type_keys::{
    GoSourceCompilerInputs, GoSourceExportInputs, GoSourceSnapshotInputs,
};
use mpk_vc::ReleaseRegistryIdentity;

const EXPORT_CAPABILITY: &str = "original-go-package-exports";

/// Owns both the validated compiler inputs and every independently matched
/// source-root handle. This is not launcher or export-cache-root authority.
pub(crate) struct InstalledGoRootMatchedCompilerInputs {
    compiler: InstalledGoSourceCompilerInputs,
    roots: GoSourceRootsMatch,
}

impl InstalledGoRootMatchedCompilerInputs {
    pub(crate) fn compiler_inputs(&self) -> &GoSourceCompilerInputs {
        self.compiler.compiler_inputs()
    }

    pub(crate) fn release_registry(&self) -> &ReleaseRegistryIdentity {
        self.compiler.release_registry()
    }

    pub(crate) fn matched_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.roots.snapshot()
    }
}

/// Bind all non-export capabilities, their complete observed namespace and raw
/// bytes to the same original export/compiler owner. The additional export root
/// is retained unchanged; unless explicitly matched it remains unauthenticated.
pub(crate) fn bind_installed_go_root_matched_inputs(
    compiler: InstalledGoCompiler,
    record: &[u8],
    exports: GoSourceExportInputs,
    roots: GoSourceRootsMatch,
) -> Result<InstalledGoRootMatchedCompilerInputs, InstalledGoCompilerError> {
    if !source_capabilities_match(exports.capture_inputs().snapshot_inputs(), roots.snapshot()) {
        return Err(InstalledGoCompilerError::Inputs);
    }
    let compiler = compiler.bind_original_inputs(record, exports)?;
    Ok(InstalledGoRootMatchedCompilerInputs { compiler, roots })
}

/// An unmatched export cache is exempt only when its captured objects really
/// are export data. Every other capability must be matched in full.
pub(crate) fn source_capabilities_match(
    full: &GoSourceSnapshotInputs,
    matched: &GoSourceSnapshotInputs,
) -> bool {
    !full
        .capabilities()
        .iter()
        .filter(|capability| capability.as_str() != EXPORT_CAPABILITY)
        .any(|capability| matched.capabilities().binary_search(capability).is_err())
        && (matched
            .capabilities()
            .iter()
            .any(|capability| capability == EXPORT_CAPABILITY)
            || !full.source_objects().iter().any(|object| {
                object.capability == EXPORT_CAPABILITY && object.kind != "export_data"
            }))
        && full.contains_exact_capabilities(matched)
}
