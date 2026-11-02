//! Kernel-checked source-string representations for one complete Go graph.
//!
//! These are representation relations, not source execution, string-operation
//! semantics, a complete foundation or launcher authority.
use mpk_cert::encode::HashBytes;
use mpk_kernel::{verify_certificate_bytes, VerificationError};
use mpk_vc::go_practical_ordinary::{
    GoOrdinaryError, GoPracticalGraphStringBinding, GoSourceStringCatalogue, GoSourceStringRow,
};
use mpk_vc::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(crate) enum GoCheckedStringError {
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
impl std::fmt::Display for GoCheckedStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coverage { row } => write!(f, "source string {row} crossed its domain"),
            Self::Construction { row, error } => {
                write!(f, "source string {row} construction: {error:?}")
            }
            Self::Kernel { row, error } => write!(
                f,
                "source string {row}: {}: {}",
                error.kind().code(),
                error.detail()
            ),
            Self::Axioms { row, count } => {
                write!(f, "source string {row} contains {count} axioms")
            }
        }
    }
}
impl std::error::Error for GoCheckedStringError {}

pub(crate) struct GoCheckedStringReceipt {
    source_type: String,
    module: String,
    declarations: usize,
    export_hash: HashBytes,
    axiom_report_hash: HashBytes,
    certificate_hash: HashBytes,
}
impl GoCheckedStringReceipt {
    pub(crate) fn source_type_id(&self) -> &str {
        &self.source_type
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
pub(crate) struct GoCheckedStringRows {
    receipts: Vec<GoCheckedStringReceipt>,
}
impl GoCheckedStringRows {
    pub(crate) fn receipts(&self) -> &[GoCheckedStringReceipt] {
        &self.receipts
    }
    pub(crate) fn reconstruct(
        &self,
        catalogue: &GoSourceStringCatalogue,
        index: usize,
    ) -> Result<GoSourceStringRow, GoCheckedStringError> {
        let receipt = self
            .receipts
            .get(index)
            .ok_or(GoCheckedStringError::Coverage { row: index })?;
        let row = catalogue
            .generate_row(&receipt.source_type)
            .map_err(|error| GoCheckedStringError::Construction { row: index, error })?;
        if row.source_string_type_id() != receipt.source_type
            || row.source_layout_module_name() != catalogue.source_layout_module_name()
            || row.module_name() != receipt.module
            || mpk_cert::certificate_hash(row.certificate_bytes()) != receipt.certificate_hash
        {
            return Err(GoCheckedStringError::Coverage { row: index });
        }
        Ok(row)
    }
}

/// Check all original types, their underlying identities and the complete
/// error universe before deriving and checking every source-string row.
pub(crate) fn check_go_string_catalogue(
    catalogue: &GoSourceStringCatalogue,
    domain: &GoSourceValueDomain,
) -> Result<GoCheckedStringRows, GoCheckedStringError> {
    let types = domain.types().collect::<Vec<_>>();
    let keys = types
        .iter()
        .map(|key| (key.id(), *key))
        .collect::<BTreeMap<_, _>>();
    if keys.len() != types.len()
        || catalogue.source_types().iter().collect::<Vec<_>>() != types
        || types
            .iter()
            .any(|key| catalogue.underlying_type_id(key.id()) != domain.underlying(key.id()))
        || !catalogue
            .case_type_ids()
            .iter()
            .map(String::as_str)
            .eq(domain.error_cases().iter().map(|case| case.value_type_id()))
    {
        return Err(GoCheckedStringError::Coverage { row: 0 });
    }
    let mut strings = Vec::new();
    for id in keys.keys() {
        let mut current = *id;
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                return Err(GoCheckedStringError::Coverage { row: 0 });
            }
            match keys.get(current) {
                Some(GoSourceTypeKey::Nominal { .. }) => {
                    current = domain
                        .underlying(current)
                        .ok_or(GoCheckedStringError::Coverage { row: 0 })?;
                }
                Some(GoSourceTypeKey::Builtin { name, .. }) if name == "string" => {
                    strings.push(*id);
                    break;
                }
                Some(_) => break,
                None => return Err(GoCheckedStringError::Coverage { row: 0 }),
            }
        }
    }
    if !catalogue
        .source_string_type_ids()
        .iter()
        .map(String::as_str)
        .eq(strings.iter().copied())
    {
        return Err(GoCheckedStringError::Coverage { row: 0 });
    }
    let mut receipts = Vec::with_capacity(strings.len());
    for (index, id) in strings.into_iter().enumerate() {
        let row = catalogue
            .generate_row(id)
            .map_err(|error| GoCheckedStringError::Construction { row: index, error })?;
        if row.source_string_type_id() != id
            || row.source_layout_module_name() != catalogue.source_layout_module_name()
        {
            return Err(GoCheckedStringError::Coverage { row: index });
        }
        let report = verify_certificate_bytes(row.certificate_bytes())
            .map_err(|error| GoCheckedStringError::Kernel { row: index, error })?;
        if report.axiom_count != 0 {
            return Err(GoCheckedStringError::Axioms {
                row: index,
                count: report.axiom_count,
            });
        }
        if report.module != row.module_name()
            || report.certificate_hash != mpk_cert::certificate_hash(row.certificate_bytes())
        {
            return Err(GoCheckedStringError::Coverage { row: index });
        }
        receipts.push(GoCheckedStringReceipt {
            source_type: id.to_string(),
            module: report.module,
            declarations: report.declaration_count,
            export_hash: report.export_hash,
            axiom_report_hash: report.axiom_report_hash,
            certificate_hash: report.certificate_hash,
        });
    }
    Ok(GoCheckedStringRows { receipts })
}

pub(crate) struct GoCheckedGraphStrings<'a> {
    binding: GoPracticalGraphStringBinding<'a>,
    checked: GoCheckedStringRows,
}
impl GoCheckedGraphStrings<'_> {
    pub(crate) fn binding(&self) -> &GoPracticalGraphStringBinding<'_> {
        &self.binding
    }
    pub(crate) fn checked(&self) -> &GoCheckedStringRows {
        &self.checked
    }
    pub(crate) fn reconstruct(
        &self,
        index: usize,
    ) -> Result<GoSourceStringRow, GoCheckedStringError> {
        self.checked.reconstruct(self.binding.catalogue(), index)
    }
}
pub(crate) fn check_go_graph_strings(
    binding: GoPracticalGraphStringBinding<'_>,
) -> Result<GoCheckedGraphStrings<'_>, GoCheckedStringError> {
    let checked = check_go_string_catalogue(
        binding.catalogue(),
        binding.contracts().contracts().module().value_domain(),
    )?;
    Ok(GoCheckedGraphStrings { binding, checked })
}
