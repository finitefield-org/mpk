//! Connect independently matched roots to the complete analysis source owner.
//!
//! This private observation grants no launcher, compiler-to-source or
//! practical-profile authority.
use crate::go_practical_source_roots::GoSourceRootsMatch;
use mpk_vc::go_practical_type_keys::{GoAnalysisSourceInputs, GoSourceSnapshotInputs};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoAnalysisRootBindingError {
    Snapshot,
}

/// Retains both the independently parsed source owner and the matched root
/// descriptors. Neither validates the caller's choice of those roots.
pub(crate) struct GoRootMatchedAnalysisInputs {
    inputs: GoAnalysisSourceInputs,
    roots: GoSourceRootsMatch,
}

impl GoRootMatchedAnalysisInputs {
    pub(crate) fn inputs(&self) -> &GoAnalysisSourceInputs {
        &self.inputs
    }

    pub(crate) fn matched_snapshot(&self) -> &GoSourceSnapshotInputs {
        self.roots.snapshot()
    }
}

/// Require the whole independently parsed snapshot, including every capability,
/// native directory entry, source identity and byte buffer, to be the same
/// snapshot compared against the retained root handles.
pub(crate) fn bind_go_analysis_root_inputs(
    inputs: GoAnalysisSourceInputs,
    roots: GoSourceRootsMatch,
) -> Result<GoRootMatchedAnalysisInputs, GoAnalysisRootBindingError> {
    if inputs.module().vendor().snapshot() != roots.snapshot() {
        return Err(GoAnalysisRootBindingError::Snapshot);
    }
    Ok(GoRootMatchedAnalysisInputs { inputs, roots })
}
