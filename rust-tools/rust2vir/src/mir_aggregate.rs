use rustc_abi::FieldIdx;
use rustc_index::IndexVec;
use rustc_middle::mir::{AggregateKind, Operand, Rvalue};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArrayAggregatePatternVector {
    pub aggregate_is_array: bool,
    pub destination_is_array: bool,
    pub arity_matches: bool,
    pub element_types_match: bool,
    pub within_limit: bool,
}

impl ArrayAggregatePatternVector {
    pub(crate) fn pinned() -> Self {
        Self {
            aggregate_is_array: true,
            destination_is_array: true,
            arity_matches: true,
            element_types_match: true,
            within_limit: true,
        }
    }
}

pub(crate) fn validate_array_aggregate_pattern(
    vector: &ArrayAggregatePatternVector,
) -> Result<(), MirCode> {
    if vector == &ArrayAggregatePatternVector::pinned() {
        Ok(())
    } else {
        Err(MirCode::Rvalue)
    }
}

pub(super) fn array_operands<'a, 'tcx>(
    rvalue: &'a Rvalue<'tcx>,
) -> Option<&'a IndexVec<FieldIdx, Operand<'tcx>>> {
    let Rvalue::Aggregate(kind, operands) = rvalue else {
        return None;
    };
    matches!(&**kind, AggregateKind::Array(_)).then_some(operands)
}
use super::mir_lower::MirCode;
