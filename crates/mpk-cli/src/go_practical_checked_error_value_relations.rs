//! Kernel-checked value relations for every original Go error case and nil.
//!
//! These mathematical rows do not prove Go equality, source execution,
//! application properties, a complete foundation or launcher authority.
use mpk_cert::encode::HashBytes;
use mpk_kernel::{verify_certificate_bytes, VerificationError};
use mpk_vc::go_practical_ordinary::{
    GoOrdinaryError, GoPracticalGraphErrorValueRelationBinding, GoSourceValueRelationCatalogue,
    GoSourceValueRelationRow,
};
use mpk_vc::go_practical_type_keys::GoSourceValueDomain;

#[derive(Debug)]
pub(crate) enum GoCheckedErrorValueRelationError {
    Coverage {
        row: usize,
    },
    Construction {
        row: usize,
        error: GoOrdinaryError,
    },
    Kernel {
        row: usize,
        error: VerificationError,
    },
    Axioms {
        row: usize,
        count: u64,
    },
}
impl std::fmt::Display for GoCheckedErrorValueRelationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coverage { row } => write!(f, "error value relation {row} crossed its domain"),
            Self::Construction { row, error } => {
                write!(f, "error value relation {row} construction: {error:?}")
            }
            Self::Kernel { row, error } => write!(
                f,
                "error value relation {row}: {}: {}",
                error.kind().code(),
                error.detail()
            ),
            Self::Axioms { row, count } => {
                write!(f, "error value relation {row} contains {count} axioms")
            }
        }
    }
}
impl std::error::Error for GoCheckedErrorValueRelationError {}

pub(crate) struct GoCheckedErrorValueReceipt {
    boxed: Option<String>,
    module: String,
    declarations: usize,
    export_hash: HashBytes,
    axiom_report_hash: HashBytes,
    certificate_hash: HashBytes,
}
impl GoCheckedErrorValueReceipt {
    pub(crate) fn boxed_source_type_id(&self) -> Option<&str> {
        self.boxed.as_deref()
    }
    pub(crate) fn module(&self) -> &str {
        &self.module
    }
    pub(crate) fn declarations(&self) -> usize {
        self.declarations
    }
    pub(crate) fn export_hash(&self) -> &HashBytes {
        &self.export_hash
    }
    pub(crate) fn axiom_report_hash(&self) -> &HashBytes {
        &self.axiom_report_hash
    }
    pub(crate) fn certificate_hash(&self) -> &HashBytes {
        &self.certificate_hash
    }
}
pub(crate) struct GoCheckedErrorValueRows {
    receipts: Vec<GoCheckedErrorValueReceipt>,
}
impl GoCheckedErrorValueRows {
    pub(crate) fn receipts(&self) -> &[GoCheckedErrorValueReceipt] {
        &self.receipts
    }
    pub(crate) fn reconstruct(
        &self,
        catalogue: &GoSourceValueRelationCatalogue,
        index: usize,
    ) -> Result<GoSourceValueRelationRow, GoCheckedErrorValueRelationError> {
        let receipt = self
            .receipts
            .get(index)
            .ok_or(GoCheckedErrorValueRelationError::Coverage { row: index })?;
        let row = catalogue
            .generate_row(receipt.boxed.as_deref())
            .map_err(|error| GoCheckedErrorValueRelationError::Construction {
                row: index,
                error,
            })?;
        if row.boxed_source_type_id() != receipt.boxed.as_deref()
            || row.module_name() != receipt.module
            || mpk_cert::certificate_hash(row.certificate_bytes()) != receipt.certificate_hash
        {
            return Err(GoCheckedErrorValueRelationError::Coverage { row: index });
        }
        Ok(row)
    }
}

/// Verify the exact complete original type and case domain before generating
/// every row in order. Neither rows nor certificates are caller-selected.
pub(crate) fn check_go_error_value_catalogue(
    catalogue: &GoSourceValueRelationCatalogue,
    domain: &GoSourceValueDomain,
) -> Result<GoCheckedErrorValueRows, GoCheckedErrorValueRelationError> {
    let types = domain.types().collect::<Vec<_>>();
    if catalogue.source_types().iter().collect::<Vec<_>>() != types
        || types
            .iter()
            .any(|key| catalogue.underlying_type_id(key.id()) != domain.underlying(key.id()))
        || catalogue.case_tags().len() != domain.error_cases().len()
        || catalogue
            .case_tags()
            .iter()
            .zip(domain.error_cases())
            .enumerate()
            .any(|(index, ((id, tag), case))| {
                id != case.value_type_id()
                    || u64::try_from(index).ok().and_then(|n| n.checked_add(1)) != Some(*tag)
            })
    {
        return Err(GoCheckedErrorValueRelationError::Coverage { row: 0 });
    }
    let mut receipts = Vec::with_capacity(catalogue.case_tags().len() + 1);
    for (index, boxed) in std::iter::once(None)
        .chain(
            catalogue
                .case_tags()
                .iter()
                .map(|(id, _)| Some(id.as_str())),
        )
        .enumerate()
    {
        let row = catalogue.generate_row(boxed).map_err(|error| {
            GoCheckedErrorValueRelationError::Construction { row: index, error }
        })?;
        if row.boxed_source_type_id() != boxed
            || row.source_layout_module_name() != catalogue.source_layout_module_name()
            || row.relation_module_name() != catalogue.relation_module_name()
            || row.case_tags() != catalogue.case_tags()
        {
            return Err(GoCheckedErrorValueRelationError::Coverage { row: index });
        }
        let report = verify_certificate_bytes(row.certificate_bytes())
            .map_err(|error| GoCheckedErrorValueRelationError::Kernel { row: index, error })?;
        if report.axiom_count != 0 {
            return Err(GoCheckedErrorValueRelationError::Axioms {
                row: index,
                count: report.axiom_count,
            });
        }
        receipts.push(GoCheckedErrorValueReceipt {
            boxed: boxed.map(String::from),
            module: report.module,
            declarations: report.declaration_count,
            export_hash: report.export_hash,
            axiom_report_hash: report.axiom_report_hash,
            certificate_hash: report.certificate_hash,
        });
    }
    Ok(GoCheckedErrorValueRows { receipts })
}

pub(crate) struct GoCheckedGraphErrorValueRelations<'a> {
    binding: GoPracticalGraphErrorValueRelationBinding<'a>,
    checked: GoCheckedErrorValueRows,
}
impl GoCheckedGraphErrorValueRelations<'_> {
    pub(crate) fn binding(&self) -> &GoPracticalGraphErrorValueRelationBinding<'_> {
        &self.binding
    }
    pub(crate) fn checked(&self) -> &GoCheckedErrorValueRows {
        &self.checked
    }
    pub(crate) fn reconstruct(
        &self,
        index: usize,
    ) -> Result<GoSourceValueRelationRow, GoCheckedErrorValueRelationError> {
        self.checked.reconstruct(self.binding.catalogue(), index)
    }
}
pub(crate) fn check_go_graph_error_value_relations(
    binding: GoPracticalGraphErrorValueRelationBinding<'_>,
) -> Result<GoCheckedGraphErrorValueRelations<'_>, GoCheckedErrorValueRelationError> {
    let checked = check_go_error_value_catalogue(
        binding.catalogue(),
        binding.contracts().contracts().module().value_domain(),
    )?;
    Ok(GoCheckedGraphErrorValueRelations { binding, checked })
}
