//! Checked source strings alongside entry, call and value relations from one
//! held Go root. This does not establish source execution or launcher authority.
use crate::go_practical_capture_binding::{
    GoCandidateRootGraphBindingError, GoCompleteRootMatchedExportInputs,
    InstalledGoCompleteRootMatchedCompilerInputs,
};
use crate::go_practical_checked_root_values::{
    check_go_root_value_components, check_installed_go_root_value_components,
    GoCheckedRootValueComponents, GoCheckedRootValueError,
};
use crate::go_practical_checked_strings::{
    check_go_graph_strings, GoCheckedGraphStrings, GoCheckedStringError,
};
use mpk_vc::go_practical_context::GoPrivateCandidateRequest;
use mpk_vc::go_practical_ordinary::GoOrdinaryError;
use mpk_vc::go_practical_type_keys::{
    GoPracticalModuleGraphBinding, GoSourceContractDerivationError, GoSourceModuleGraphInputs,
};

#[derive(Debug)]
pub(crate) enum GoCheckedRootStringError {
    Values(GoCheckedRootValueError),
    Binding(GoCandidateRootGraphBindingError),
    Contracts(GoSourceContractDerivationError),
    DivergentContracts,
    Catalogue(GoOrdinaryError),
    Strings(GoCheckedStringError),
}
impl std::fmt::Display for GoCheckedRootStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Values(error) => write!(f, "entry, call and value components: {error}"),
            Self::Binding(error) => write!(f, "candidate graph: {error}"),
            Self::Contracts(error) => write!(f, "source contracts: {error}"),
            Self::DivergentContracts => write!(f, "source contract derivations differ"),
            Self::Catalogue(error) => write!(f, "source strings: {error:?}"),
            Self::Strings(error) => write!(f, "source strings: {error}"),
        }
    }
}
impl std::error::Error for GoCheckedRootStringError {}

pub(crate) struct GoCheckedRootStringComponents<'a> {
    values: GoCheckedRootValueComponents<'a>,
    strings: GoCheckedGraphStrings<'a>,
}
impl GoCheckedRootStringComponents<'_> {
    pub(crate) fn values(&self) -> &GoCheckedRootValueComponents<'_> {
        &self.values
    }
    pub(crate) fn strings(&self) -> &GoCheckedGraphStrings<'_> {
        &self.strings
    }
}

trait RootStringOwner {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError>;
    fn values<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError>;
}
impl RootStringOwner for GoCompleteRootMatchedExportInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
    fn values<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError> {
        check_go_root_value_components(self, request, graph)
    }
}
impl RootStringOwner for InstalledGoCompleteRootMatchedCompilerInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
    fn values<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError> {
        check_installed_go_root_value_components(self, request, graph)
    }
}

fn check_root_strings<'a, O: RootStringOwner>(
    owner: &'a O,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootStringComponents<'a>, GoCheckedRootStringError> {
    let values = owner
        .values(request, graph)
        .map_err(GoCheckedRootStringError::Values)?;
    let contracts = owner
        .bind(request, graph)
        .map_err(GoCheckedRootStringError::Binding)?
        .derive_contracts()
        .map_err(GoCheckedRootStringError::Contracts)?;
    if contracts.contracts().module()
        != values
            .components()
            .entries()
            .entries()
            .contracts()
            .contracts()
            .module()
    {
        return Err(GoCheckedRootStringError::DivergentContracts);
    }
    let binding = contracts
        .derive_source_strings()
        .map_err(GoCheckedRootStringError::Catalogue)?;
    let strings = check_go_graph_strings(binding).map_err(GoCheckedRootStringError::Strings)?;
    Ok(GoCheckedRootStringComponents { values, strings })
}

pub(crate) fn check_go_root_string_components<'a>(
    owner: &'a GoCompleteRootMatchedExportInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootStringComponents<'a>, GoCheckedRootStringError> {
    check_root_strings(owner, request, graph)
}

pub(crate) fn check_installed_go_root_string_components<'a>(
    owner: &'a InstalledGoCompleteRootMatchedCompilerInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootStringComponents<'a>, GoCheckedRootStringError> {
    check_root_strings(owner, request, graph)
}
