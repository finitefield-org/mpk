//! Private checked entry-equation parts for a bound Go candidate.
//!
//! These kernel checks establish only the generated entry mathematics. They do
//! not establish source execution, a complete foundation or launcher authority.
use mpk_kernel::{verify_certificate_bytes, VerificationError, VerificationReport};
use mpk_vc::go_practical_ordinary::{
    GoOrdinaryError, GoPracticalGraphEntryBinding, GoSourceEntryAssemblies, GoSourceEntryCatalogue,
};
use mpk_vc::go_practical_type_keys::GoSourceContractModule;

#[derive(Debug)]
pub(crate) enum GoCheckedEntryError {
    Construction(GoOrdinaryError),
    Empty,
    Coverage,
    Kernel {
        part: usize,
        error: VerificationError,
    },
    Axioms {
        part: usize,
        count: u64,
    },
}

impl std::fmt::Display for GoCheckedEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Construction(error) => write!(f, "entry assembly construction: {error:?}"),
            Self::Empty => write!(f, "entry assembly has no functions"),
            Self::Coverage => write!(f, "entry assemblies do not cover the original layouts"),
            Self::Kernel { part, error } => write!(
                f,
                "entry assembly {part}: {}: {}",
                error.kind().code(),
                error.detail()
            ),
            Self::Axioms { part, count } => {
                write!(f, "entry assembly {part} contains {count} axioms")
            }
        }
    }
}
impl std::error::Error for GoCheckedEntryError {}

/// All complete parts and their independent kernel reports. Packing snapshots
/// remain incomplete construction data and never receive a report here.
pub(crate) struct GoCheckedEntryParts {
    assemblies: GoSourceEntryAssemblies,
    reports: Vec<VerificationReport>,
}
impl GoCheckedEntryParts {
    pub(crate) fn assemblies(&self) -> &GoSourceEntryAssemblies {
        &self.assemblies
    }
    pub(crate) fn reports(&self) -> &[VerificationReport] {
        &self.reports
    }
}

/// Generate every original layout, check its complete parts in order and
/// retain a report for each. No caller supplies a subset or certificate bytes.
pub(crate) fn check_go_entry_catalogue(
    catalogue: &GoSourceEntryCatalogue,
) -> Result<GoCheckedEntryParts, GoCheckedEntryError> {
    let assemblies = catalogue
        .generate_assemblies()
        .map_err(GoCheckedEntryError::Construction)?;
    if catalogue.layouts().is_empty() || assemblies.parts().is_empty() {
        return Err(GoCheckedEntryError::Empty);
    }
    let expected = catalogue.layouts().iter().map(|layout| layout.function());
    let actual = assemblies
        .parts()
        .iter()
        .flat_map(|part| part.function_modules().iter().map(|(function, _)| function));
    if !expected.eq(actual) {
        return Err(GoCheckedEntryError::Coverage);
    }
    let mut reports = Vec::with_capacity(assemblies.parts().len());
    for (part, assembly) in assemblies.parts().iter().enumerate() {
        let report = verify_certificate_bytes(assembly.certificate_bytes())
            .map_err(|error| GoCheckedEntryError::Kernel { part, error })?;
        if report.axiom_count != 0 {
            return Err(GoCheckedEntryError::Axioms {
                part,
                count: report.axiom_count,
            });
        }
        reports.push(report);
    }
    Ok(GoCheckedEntryParts {
        assemblies,
        reports,
    })
}

/// Retain the validated graph and original contracts alongside every checked
/// source-entry part. This owner does not grant program or context authority.
pub(crate) struct GoCheckedGraphEntries<'a> {
    entries: GoPracticalGraphEntryBinding<'a>,
    checked: GoCheckedEntryParts,
}
impl GoCheckedGraphEntries<'_> {
    pub(crate) fn entries(&self) -> &GoPracticalGraphEntryBinding<'_> {
        &self.entries
    }
    pub(crate) fn checked(&self) -> &GoCheckedEntryParts {
        &self.checked
    }
}

pub(crate) fn check_go_graph_entries(
    entries: GoPracticalGraphEntryBinding<'_>,
) -> Result<GoCheckedGraphEntries<'_>, GoCheckedEntryError> {
    if !catalogue_covers_contracts(
        entries.contracts().contracts().module(),
        entries.catalogue(),
    ) {
        return Err(GoCheckedEntryError::Coverage);
    }
    let checked = check_go_entry_catalogue(entries.catalogue())?;
    Ok(GoCheckedGraphEntries { entries, checked })
}

/// Compare the original selected and transitive contract bodies with the
/// catalogue before checking its packed parts. Both sequences are canonical.
pub(crate) fn catalogue_covers_contracts(
    contracts: &GoSourceContractModule,
    catalogue: &GoSourceEntryCatalogue,
) -> bool {
    contracts
        .functions()
        .iter()
        .map(|function| function.function())
        .eq(catalogue.layouts().iter().map(|layout| layout.function()))
}
