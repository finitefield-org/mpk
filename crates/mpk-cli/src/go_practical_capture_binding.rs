//! Connect independently matched source roots to the installed compiler owner.
//!
//! The distinct export-cache root and launcher authorization remain separate;
//! this private owner cannot publish a practical context or source theorem.
use crate::go_practical_installed_compiler::{
    InstalledGoCompiler, InstalledGoCompilerError, InstalledGoFrontend, InstalledGoRelease,
    InstalledGoSourceCompilerInputs,
};
use crate::go_practical_source_roots::{GoSourceRootError, GoSourceRoots, GoSourceRootsMatch};
use mpk_vc::go_practical_context::GoPrivateCandidateRequest;
use mpk_vc::go_practical_type_keys::{
    bind_go_practical_module_graph, bind_go_practical_source_selection,
    GoPracticalModuleGraphBinding, GoPracticalModuleGraphBindingError, GoPracticalSourceBinding,
    GoPracticalSourceBindingError, GoSourceCompilerInputs, GoSourceExportInputs,
    GoSourceModuleGraphInputs, GoSourceSnapshotInputs,
};
use mpk_vc::ReleaseRegistryIdentity;
use std::collections::BTreeSet;
use std::path::PathBuf;

const EXPORT_CAPABILITY: &str = "original-go-package-exports";

#[derive(Debug)]
pub(crate) enum GoCandidateRootGraphBindingError {
    Export,
    Source(GoPracticalSourceBindingError),
    Graph(GoPracticalModuleGraphBindingError),
}
impl std::fmt::Display for GoCandidateRootGraphBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Export => write!(f, "candidate graph crossed its held export owner"),
            Self::Source(error) => write!(f, "{error}"),
            Self::Graph(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for GoCandidateRootGraphBindingError {}

fn bind_candidate_root_graph<'a>(
    exports: &'a GoSourceExportInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
    if graph.module_data().module_observations().export_inputs() != exports {
        return Err(GoCandidateRootGraphBindingError::Export);
    }
    let source =
        bind_go_practical_source_selection(request, exports.capture_inputs().selection_inputs())
            .map_err(GoCandidateRootGraphBindingError::Source)?;
    bind_go_practical_module_graph(source, graph).map_err(GoCandidateRootGraphBindingError::Graph)
}

/// Capture-time root ownership, before the generated export cache exists.
/// This enforces observation order, but does not authorize caller-selected
/// paths or prove that an external process read from the retained handles.
pub(crate) struct GoCaptureSourceRoots {
    source: GoSourceRoots,
}

/// Both root sets are open, but neither snapshot has yet been matched.
pub(crate) struct GoCapturePendingRoots {
    source: GoSourceRoots,
    export: GoSourceRoots,
}

/// The export snapshot was matched while its generated cache was live.
pub(crate) struct GoCapturePendingSource {
    source: GoSourceRoots,
    export: GoSourceRootsMatch,
}

/// Both independently matched root sets are retained until binding.
pub(crate) struct GoCaptureMatchedRoots {
    source: GoSourceRootsMatch,
    export: GoSourceRootsMatch,
}

impl GoCaptureSourceRoots {
    pub(crate) fn open(bindings: Vec<(String, PathBuf)>) -> Result<Self, GoSourceRootError> {
        let mut names = BTreeSet::new();
        if bindings.is_empty()
            || bindings.iter().any(|(name, _)| {
                name.eq_ignore_ascii_case(EXPORT_CAPABILITY)
                    || !names.insert(name.to_ascii_lowercase())
            })
        {
            return Err(GoSourceRootError::Capabilities);
        }
        Ok(Self {
            source: GoSourceRoots::open(bindings)?,
        })
    }

    /// Open the one reserved export capability before capturing its bytes.
    pub(crate) fn open_export(
        self,
        path: PathBuf,
    ) -> Result<GoCapturePendingRoots, GoSourceRootError> {
        let export = GoSourceRoots::open(vec![(EXPORT_CAPABILITY.to_owned(), path)])?;
        Ok(GoCapturePendingRoots {
            source: self.source,
            export,
        })
    }
}

impl GoCapturePendingRoots {
    pub(crate) fn match_export(
        self,
        snapshot: GoSourceSnapshotInputs,
    ) -> Result<GoCapturePendingSource, GoSourceRootError> {
        Ok(GoCapturePendingSource {
            source: self.source,
            export: self.export.compare(snapshot)?,
        })
    }
}

impl GoCapturePendingSource {
    pub(crate) fn match_source(
        self,
        snapshot: GoSourceSnapshotInputs,
    ) -> Result<GoCaptureMatchedRoots, GoSourceRootError> {
        Ok(GoCaptureMatchedRoots {
            source: self.source.compare(snapshot)?,
            export: self.export,
        })
    }
}

impl GoCaptureMatchedRoots {
    pub(crate) fn bind(
        self,
        exports: GoSourceExportInputs,
    ) -> Result<GoCompleteRootMatchedExportInputs, InstalledGoCompilerError> {
        bind_complete_go_root_matched_exports(exports, self.source, self.export)
    }
}

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

    /// The binding borrows the exact source owner covered by both held roots.
    pub(crate) fn bind_candidate_request<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
    ) -> Result<GoPracticalSourceBinding<'a>, GoPracticalSourceBindingError> {
        bind_go_practical_source_selection(
            request,
            self.exports.capture_inputs().selection_inputs(),
        )
    }

    /// Retain both matched roots with the exact complete request/graph owner.
    pub(crate) fn bind_candidate_graph<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        bind_candidate_root_graph(&self.exports, request, graph)
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

fn bind_complete_go_root_matched_exports(
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

    /// Keep installed compiler and both root owners borrowed for the complete
    /// request/source match; this does not authorize launcher-selected paths.
    pub(crate) fn bind_candidate_request<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
    ) -> Result<GoPracticalSourceBinding<'a>, GoPracticalSourceBindingError> {
        bind_go_practical_source_selection(
            request,
            self.compiler_inputs()
                .export_inputs()
                .capture_inputs()
                .selection_inputs(),
        )
    }

    /// Retain installed compiler and both matched roots with the same graph.
    pub(crate) fn bind_candidate_graph<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        bind_candidate_root_graph(self.compiler_inputs().export_inputs(), request, graph)
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

/// Keep the exact registered frontend with the SDK and both independently
/// matched root sets. The frontend has not been shown to execute the capture.
pub(crate) struct InstalledGoCompleteRootMatchedReleaseInputs {
    compiler: InstalledGoCompleteRootMatchedCompilerInputs,
    frontend: InstalledGoFrontend,
}
impl InstalledGoCompleteRootMatchedReleaseInputs {
    pub(crate) fn compiler(&self) -> &InstalledGoCompleteRootMatchedCompilerInputs {
        &self.compiler
    }

    pub(crate) fn frontend(&self) -> &InstalledGoFrontend {
        &self.frontend
    }
}

pub(crate) fn bind_installed_go_complete_root_matched_release_inputs(
    release: InstalledGoRelease,
    record: &[u8],
    matched: GoCompleteRootMatchedExportInputs,
) -> Result<InstalledGoCompleteRootMatchedReleaseInputs, InstalledGoCompilerError> {
    let (compiler, frontend) = release.into_parts();
    let compiler = bind_installed_go_complete_root_matched_inputs(compiler, record, matched)?;
    Ok(InstalledGoCompleteRootMatchedReleaseInputs { compiler, frontend })
}
