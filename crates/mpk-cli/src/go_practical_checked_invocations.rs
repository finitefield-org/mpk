//! Private checked mathematical interfaces for all original Go call sites.
//!
//! These checks do not establish callee execution, effects or source/IR proof.
use mpk_cert::encode::HashBytes;
use mpk_kernel::{verify_certificate_bytes, VerificationError};
use mpk_vc::go_practical_ordinary::{
    GoOrdinaryError, GoPracticalGraphInvocationBinding, GoPracticalInvocationBinding,
    GoSourceInvocationCatalogue, GoSourceInvocationRow,
};
use mpk_vc::go_practical_type_keys::GoSourceCallSite;

#[derive(Debug)]
pub(crate) enum GoCheckedInvocationError {
    Construction {
        site: usize,
        error: GoOrdinaryError,
    },
    Coverage {
        site: usize,
    },
    Kernel {
        site: usize,
        error: VerificationError,
    },
    Axioms {
        site: usize,
        count: u64,
    },
}
impl std::fmt::Display for GoCheckedInvocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Construction { site, error } => {
                write!(f, "call interface {site} construction: {error:?}")
            }
            Self::Coverage { site } => write!(f, "call interface {site} crossed its source site"),
            Self::Kernel { site, error } => write!(
                f,
                "call interface {site}: {}: {}",
                error.kind().code(),
                error.detail()
            ),
            Self::Axioms { site, count } => {
                write!(f, "call interface {site} contains {count} axioms")
            }
        }
    }
}
impl std::error::Error for GoCheckedInvocationError {}

/// A compact receipt for a kernel-checked, zero-axiom source call. It does
/// not retain the large certificate or its full axiom report in memory.
pub(crate) struct GoCheckedInvocationReceipt {
    site: GoSourceCallSite,
    module: String,
    declarations: usize,
    export_hash: HashBytes,
    axiom_report_hash: HashBytes,
    certificate_hash: HashBytes,
}
impl GoCheckedInvocationReceipt {
    pub(crate) fn site(&self) -> &GoSourceCallSite {
        &self.site
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

pub(crate) struct GoCheckedInvocationRows {
    receipts: Vec<GoCheckedInvocationReceipt>,
}
impl GoCheckedInvocationRows {
    pub(crate) fn receipts(&self) -> &[GoCheckedInvocationReceipt] {
        &self.receipts
    }

    fn reconstruct(
        &self,
        catalogue: &GoSourceInvocationCatalogue,
        index: usize,
    ) -> Result<GoSourceInvocationRow, GoCheckedInvocationError> {
        let receipt = self
            .receipts
            .get(index)
            .ok_or(GoCheckedInvocationError::Coverage { site: index })?;
        let row = catalogue
            .generate_row(receipt.site.caller(), receipt.site.node())
            .map_err(|error| GoCheckedInvocationError::Construction { site: index, error })?;
        if row.site() != &receipt.site
            || row.module_name() != receipt.module
            || mpk_cert::certificate_hash(row.certificate_bytes()) != receipt.certificate_hash
        {
            return Err(GoCheckedInvocationError::Coverage { site: index });
        }
        Ok(row)
    }
}

/// Check every original site in catalogue order. Empty call closures are
/// valid; callers cannot supply certificates or choose a smaller site set.
pub(crate) fn check_go_invocation_catalogue(
    catalogue: &GoSourceInvocationCatalogue,
) -> Result<GoCheckedInvocationRows, GoCheckedInvocationError> {
    let mut receipts = Vec::with_capacity(catalogue.sites().len());
    for (index, site) in catalogue.sites().iter().enumerate() {
        let row = catalogue
            .generate_row(site.caller(), site.node())
            .map_err(|error| GoCheckedInvocationError::Construction { site: index, error })?;
        if row.site() != site {
            return Err(GoCheckedInvocationError::Coverage { site: index });
        }
        let report = verify_certificate_bytes(row.certificate_bytes())
            .map_err(|error| GoCheckedInvocationError::Kernel { site: index, error })?;
        if report.axiom_count != 0 {
            return Err(GoCheckedInvocationError::Axioms {
                site: index,
                count: report.axiom_count,
            });
        }
        receipts.push(GoCheckedInvocationReceipt {
            site: site.clone(),
            module: report.module,
            declarations: report.declaration_count,
            export_hash: report.export_hash,
            axiom_report_hash: report.axiom_report_hash,
            certificate_hash: report.certificate_hash,
        });
    }
    Ok(GoCheckedInvocationRows { receipts })
}

pub(crate) struct GoCheckedInvocations<'a> {
    invocations: GoPracticalInvocationBinding<'a>,
    checked: GoCheckedInvocationRows,
}
impl GoCheckedInvocations<'_> {
    pub(crate) fn invocations(&self) -> &GoPracticalInvocationBinding<'_> {
        &self.invocations
    }
    pub(crate) fn checked(&self) -> &GoCheckedInvocationRows {
        &self.checked
    }
    /// Rebuild one certificate and require the same identity that the kernel
    /// already checked; only the requested certificate is held in memory.
    pub(crate) fn reconstruct(
        &self,
        index: usize,
    ) -> Result<GoSourceInvocationRow, GoCheckedInvocationError> {
        self.checked
            .reconstruct(self.invocations.catalogue(), index)
    }
}
pub(crate) fn check_go_invocations(
    invocations: GoPracticalInvocationBinding<'_>,
) -> Result<GoCheckedInvocations<'_>, GoCheckedInvocationError> {
    let checked = check_go_invocation_catalogue(invocations.catalogue())?;
    Ok(GoCheckedInvocations {
        invocations,
        checked,
    })
}

pub(crate) struct GoCheckedGraphInvocations<'a> {
    invocations: GoPracticalGraphInvocationBinding<'a>,
    checked: GoCheckedInvocationRows,
}
impl GoCheckedGraphInvocations<'_> {
    pub(crate) fn invocations(&self) -> &GoPracticalGraphInvocationBinding<'_> {
        &self.invocations
    }
    pub(crate) fn checked(&self) -> &GoCheckedInvocationRows {
        &self.checked
    }
    pub(crate) fn reconstruct(
        &self,
        index: usize,
    ) -> Result<GoSourceInvocationRow, GoCheckedInvocationError> {
        self.checked
            .reconstruct(self.invocations.catalogue(), index)
    }
}
pub(crate) fn check_go_graph_invocations(
    invocations: GoPracticalGraphInvocationBinding<'_>,
) -> Result<GoCheckedGraphInvocations<'_>, GoCheckedInvocationError> {
    let checked = check_go_invocation_catalogue(invocations.catalogue())?;
    Ok(GoCheckedGraphInvocations {
        invocations,
        checked,
    })
}
