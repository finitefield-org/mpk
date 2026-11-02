//! Checked Go entry, call and value relations from one held root owner.
//!
//! This private owner does not establish source execution, a complete
//! foundation, installed launcher authority or an application theorem.
use crate::go_practical_capture_binding::{
    GoCandidateRootGraphBindingError, GoCompleteRootMatchedExportInputs,
    InstalledGoCompleteRootMatchedCompilerInputs,
};
use crate::go_practical_checked_components::{
    check_go_root_components, check_installed_go_root_components, GoCheckedComponentsError,
    GoCheckedRootComponents,
};
use crate::go_practical_checked_error_value_relations::{
    check_go_graph_error_value_relations, GoCheckedErrorValueRelationError,
    GoCheckedGraphErrorValueRelations,
};
use crate::go_practical_checked_value_relations::{
    check_go_graph_value_relations, GoCheckedGraphValueRelations, GoCheckedValueRelationError,
};
use mpk_vc::go_practical_context::GoPrivateCandidateRequest;
use mpk_vc::go_practical_ordinary::GoOrdinaryError;
use mpk_vc::go_practical_type_keys::{
    GoPracticalModuleGraphBinding, GoSourceContractDerivationError, GoSourceModuleGraphInputs,
    GoSourceTypeKey,
};

#[derive(Debug)]
pub(crate) enum GoCheckedRootValueError {
    Components(GoCheckedComponentsError),
    Binding(GoCandidateRootGraphBindingError),
    Contracts(GoSourceContractDerivationError),
    DivergentContracts,
    Catalogue(GoOrdinaryError),
    ErrorValues(GoCheckedErrorValueRelationError),
    ErrorFreeValues(GoCheckedValueRelationError),
}
impl std::fmt::Display for GoCheckedRootValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Components(error) => write!(f, "entry and call components: {error}"),
            Self::Binding(error) => write!(f, "candidate graph: {error}"),
            Self::Contracts(error) => write!(f, "source contracts: {error}"),
            Self::DivergentContracts => write!(f, "source contract derivations differ"),
            Self::Catalogue(error) => write!(f, "value relations: {error:?}"),
            Self::ErrorValues(error) => write!(f, "error value relations: {error}"),
            Self::ErrorFreeValues(error) => write!(f, "error-free value relations: {error}"),
        }
    }
}
impl std::error::Error for GoCheckedRootValueError {}

pub(crate) enum GoCheckedRootValues<'a> {
    ErrorFree(Box<GoCheckedGraphValueRelations<'a>>),
    Error(Box<GoCheckedGraphErrorValueRelations<'a>>),
}
pub(crate) struct GoCheckedRootValueComponents<'a> {
    components: GoCheckedRootComponents<'a>,
    values: GoCheckedRootValues<'a>,
}
impl GoCheckedRootValueComponents<'_> {
    pub(crate) fn components(&self) -> &GoCheckedRootComponents<'_> {
        &self.components
    }
    pub(crate) fn values(&self) -> &GoCheckedRootValues<'_> {
        &self.values
    }
}

// Only the two existing held-root owners can supply this private adapter.
trait RootValueOwner {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError>;
    fn components<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError>;
}
impl RootValueOwner for GoCompleteRootMatchedExportInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
    fn components<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError> {
        check_go_root_components(self, request, graph)
    }
}
impl RootValueOwner for InstalledGoCompleteRootMatchedCompilerInputs {
    fn bind<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoPracticalModuleGraphBinding<'a>, GoCandidateRootGraphBindingError> {
        self.bind_candidate_graph(request, graph)
    }
    fn components<'a>(
        &'a self,
        request: &'a GoPrivateCandidateRequest,
        graph: &'a GoSourceModuleGraphInputs,
    ) -> Result<GoCheckedRootComponents<'a>, GoCheckedComponentsError> {
        check_installed_go_root_components(self, request, graph)
    }
}

fn check_root_values<'a, O: RootValueOwner>(
    owner: &'a O,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError> {
    let components = owner
        .components(request, graph)
        .map_err(GoCheckedRootValueError::Components)?;
    let contracts = owner
        .bind(request, graph)
        .map_err(GoCheckedRootValueError::Binding)?
        .derive_contracts()
        .map_err(GoCheckedRootValueError::Contracts)?;
    if contracts.contracts().module()
        != components
            .entries()
            .entries()
            .contracts()
            .contracts()
            .module()
    {
        return Err(GoCheckedRootValueError::DivergentContracts);
    }
    let has_error = contracts
        .contracts()
        .module()
        .value_domain()
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Builtin { name, .. } if name == "error"));
    let values = if has_error {
        let binding = contracts
            .derive_error_value_relations()
            .map_err(GoCheckedRootValueError::Catalogue)?;
        GoCheckedRootValues::Error(Box::new(
            check_go_graph_error_value_relations(binding)
                .map_err(GoCheckedRootValueError::ErrorValues)?,
        ))
    } else {
        let binding = contracts
            .derive_error_free_value_relations()
            .map_err(GoCheckedRootValueError::Catalogue)?;
        GoCheckedRootValues::ErrorFree(Box::new(
            check_go_graph_value_relations(binding)
                .map_err(GoCheckedRootValueError::ErrorFreeValues)?,
        ))
    };
    Ok(GoCheckedRootValueComponents { components, values })
}

pub(crate) fn check_go_root_value_components<'a>(
    owner: &'a GoCompleteRootMatchedExportInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError> {
    check_root_values(owner, request, graph)
}

pub(crate) fn check_installed_go_root_value_components<'a>(
    owner: &'a InstalledGoCompleteRootMatchedCompilerInputs,
    request: &'a GoPrivateCandidateRequest,
    graph: &'a GoSourceModuleGraphInputs,
) -> Result<GoCheckedRootValueComponents<'a>, GoCheckedRootValueError> {
    check_root_values(owner, request, graph)
}
