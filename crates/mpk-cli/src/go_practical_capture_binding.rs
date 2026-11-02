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

/// Owns both independently matched root sets and the exact original export
/// inputs they cover. Matching records observation at capture time; it does
/// not authorize launcher paths or establish a compiler/source theorem.
pub(crate) struct GoCompleteRootMatchedExportInputs {
    exports: GoSourceExportInputs,
    source_roots: GoSourceRootsMatch,
    export_roots: GoSourceRootsMatch,
}

impl GoCompleteRootMatchedExportInputs {
    pub(crate) fn export_inputs(&self) -> &GoSourceExportInputs {
        &self.exports
    }

    pub(crate) fn source_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.source_roots.snapshot()
    }

    pub(crate) fn export_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.export_roots.snapshot()
    }
}

/// Require the two disjoint root sets to cover every original capability,
/// including every captured object and each observed directory entry.
pub(crate) fn complete_capabilities_match(
    full: &GoSourceSnapshotInputs,
    source: &GoSourceSnapshotInputs,
    export: &GoSourceSnapshotInputs,
) -> bool {
    export.capabilities().len() == 1
        && export.capabilities()[0] == EXPORT_CAPABILITY
        && !source.capabilities().iter().any(|c| c == EXPORT_CAPABILITY)
        && full.capabilities().len() == source.capabilities().len() + 1
        && full.contains_exact_capabilities(source)
        && full.contains_exact_capabilities(export)
}

pub(crate) fn bind_complete_go_root_matched_exports(
    exports: GoSourceExportInputs,
    source_roots: GoSourceRootsMatch,
    export_roots: GoSourceRootsMatch,
) -> Result<GoCompleteRootMatchedExportInputs, InstalledGoCompilerError> {
    if !complete_capabilities_match(
        exports.capture_inputs().snapshot_inputs(),
        source_roots.snapshot(),
        export_roots.snapshot(),
    ) {
        return Err(InstalledGoCompilerError::Inputs);
    }
    Ok(GoCompleteRootMatchedExportInputs {
        exports,
        source_roots,
        export_roots,
    })
}

/// Retain the independently verified installed compiler and both matched root
/// sets with the same complete export owner. This remains a private input
/// owner, not a registered context or application verification result.
pub(crate) struct InstalledGoCompleteRootMatchedCompilerInputs {
    compiler: InstalledGoSourceCompilerInputs,
    source_roots: GoSourceRootsMatch,
    export_roots: GoSourceRootsMatch,
}

impl InstalledGoCompleteRootMatchedCompilerInputs {
    pub(crate) fn compiler_inputs(&self) -> &GoSourceCompilerInputs {
        self.compiler.compiler_inputs()
    }

    pub(crate) fn release_registry(&self) -> &ReleaseRegistryIdentity {
        self.compiler.release_registry()
    }

    pub(crate) fn source_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.source_roots.snapshot()
    }

    pub(crate) fn export_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.export_roots.snapshot()
    }
}

pub(crate) fn bind_installed_go_complete_root_matched_inputs(
    compiler: InstalledGoCompiler,
    record: &[u8],
    matched: GoCompleteRootMatchedExportInputs,
) -> Result<InstalledGoCompleteRootMatchedCompilerInputs, InstalledGoCompilerError> {
    let GoCompleteRootMatchedExportInputs {
        exports,
        source_roots,
        export_roots,
    } = matched;
    let compiler = compiler.bind_original_inputs(record, exports)?;
    Ok(InstalledGoCompleteRootMatchedCompilerInputs {
        compiler,
        source_roots,
        export_roots,
    })
}
