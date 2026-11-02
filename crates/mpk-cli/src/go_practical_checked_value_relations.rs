//! Kernel-checked, error-free value relations for one bound Go graph.
//!
//! These are whole-carrier mathematical relations. They do not prove Go
//! equality, source execution, a complete foundation or launcher authority.
use mpk_kernel::{verify_certificate_bytes, VerificationError, VerificationReport};
use mpk_vc::go_practical_ordinary::GoPracticalGraphValueRelationBinding;

#[derive(Debug)]
pub(crate) enum GoCheckedValueRelationError {
    Kernel(VerificationError),
    Axioms(u64),
}
impl std::fmt::Display for GoCheckedValueRelationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kernel(error) => write!(
                f,
                "value relations: {}: {}",
                error.kind().code(),
                error.detail()
            ),
            Self::Axioms(count) => write!(f, "value relations contain {count} axioms"),
        }
    }
}
impl std::error::Error for GoCheckedValueRelationError {}

pub(crate) struct GoCheckedGraphValueRelations<'a> {
    binding: GoPracticalGraphValueRelationBinding<'a>,
    report: VerificationReport,
}
impl GoCheckedGraphValueRelations<'_> {
    pub(crate) fn binding(&self) -> &GoPracticalGraphValueRelationBinding<'_> {
        &self.binding
    }
    pub(crate) fn report(&self) -> &VerificationReport {
        &self.report
    }
}

/// Retain the original candidate and graph through the kernel check. The
/// caller cannot supply a replacement certificate or verification report.
pub(crate) fn check_go_graph_value_relations(
    binding: GoPracticalGraphValueRelationBinding<'_>,
) -> Result<GoCheckedGraphValueRelations<'_>, GoCheckedValueRelationError> {
    let report = verify_certificate_bytes(binding.relations().certificate_bytes())
        .map_err(GoCheckedValueRelationError::Kernel)?;
    if report.axiom_count != 0 {
        return Err(GoCheckedValueRelationError::Axioms(report.axiom_count));
    }
    Ok(GoCheckedGraphValueRelations { binding, report })
}
