//! Checked entry and call mathematics derived from one held original owner.
//!
//! This does not authorize a launcher or establish source/program correctness.
use crate::go_practical_capture_binding::{
    GoCandidateRootGraphBindingError, GoCompleteRootMatchedExportInputs,
    InstalledGoCompleteRootMatchedCompilerInputs,
};
use crate::go_practical_checked_entries::{
    check_go_graph_entries, GoCheckedEntryError, GoCheckedGraphEntries,
};
use crate::go_practical_checked_invocations::{
    check_go_graph_invocations, GoCheckedGraphInvocations, GoCheckedInvocationError,
};
use mpk_vc::go_practical_context::GoPrivateCandidateRequest;
use mpk_vc::go_practical_ordinary::GoOrdinaryError;
use mpk_vc::go_practical_type_keys::{
    GoPracticalModuleGraphBinding, GoSourceContractDerivationError, GoSourceModuleGraphInputs,
};

#[derive(Debug)]
pub(crate) enum GoCheckedComponentsError {
    Binding(GoCandidateRootGraphBindingError),
    Contracts(GoSourceContractDerivationError),
    DivergentContracts,
    Catalogue(GoOrdinaryError),
    Entries(GoCheckedEntryError),
    Invocations(GoCheckedInvocationError),
}
impl std::fmt::Display for GoCheckedComponentsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(error) => write!(f, "candidate graph: {error}"),
            Self::Contracts(error) => write!(f, "source contracts: {error}"),
            Self::DivergentContracts => write!(f, "source contract derivations differ"),
            Self::Catalogue(error) => write!(f, "source catalogue: {error:?}"),
            Self::Entries(error) => write!(f, "entry equations: {error}"),
            Self::Invocations(error) => write!(f, "call interfaces: {error}"),
        }
    }
}
impl std::error::Error for GoCheckedComponentsError {}

pub(crate) struct GoCheckedRootComponents<'a> {
    entries: GoCheckedGraphEntries<'a>,
    invocations: GoCheckedGraphInvocations<'a>,
}
impl GoCheckedRootComponents<'_> {
    pub(crate) fn entries(&self) -> &GoCheckedGraphEntries<'_> {
        &self.entries
    }
    pub(crate) fn invocations(&self) -> &GoCheckedGraphInvocations<'_> {
        &self.invocations
    }
}

// Only the two existing root-held owners implement this private adapter.
// The caller cannot supply independently built entry and invocation owners.
trait RootGraphOwner {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError>;
}
impl RootGraphOwner for GoCompleteRootMatchedExportInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
}
impl RootGraphOwner for InstalledGoCompleteRootMatchedCompilerInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
}

fn check_components<'a, O: RootGraphOwner>(
    owner: &'a O,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError> {
    let entries = owner
        .bind(request, graph)
        .map_err(GoCheckedComponentsError::Binding)?
        .derive_contracts()
        .map_err(GoCheckedComponentsError::Contracts)?;
    let invocations = owner
        .bind(request, graph)
        .map_err(GoCheckedComponentsError::Binding)?
        .derive_contracts()
        .map_err(GoCheckedComponentsError::Contracts)?;
    if entries.contracts().module() != invocations.contracts().module() {
        return Err(GoCheckedComponentsError::DivergentContracts);
    }
    let entries = entries
        .derive_entry_catalogue()
        .map_err(GoCheckedComponentsError::Catalogue)?;
    let invocations = invocations
        .derive_invocation_catalogue()
        .map_err(GoCheckedComponentsError::Catalogue)?;
    Ok(GoCheckedRootComponents {
        entries: check_go_graph_entries(entries).map_err(GoCheckedComponentsError::Entries)?,
        invocations: check_go_graph_invocations(invocations)
            .map_err(GoCheckedComponentsError::Invocations)?,
    })
}

pub(crate) fn check_go_root_components<'a>(
    owner: &'a GoCompleteRootMatchedExportInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError> {
    check_components(owner, request, graph)
}

pub(crate) fn check_installed_go_root_components<'a>(
    owner: &'a InstalledGoCompleteRootMatchedCompilerInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError> {
    check_components(owner, request, graph)
}
