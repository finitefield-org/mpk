use rustc_abi::{FieldIdx, VariantIdx};
use rustc_index::IndexVec;
use rustc_middle::mir::{AggregateKind, Operand, Rvalue};
use rustc_middle::ty::GenericArgsRef;
use rustc_span::def_id::DefId;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StructAggregatePatternVector {
    pub aggregate_is_adt: bool,
    pub definition_is_local_named_struct: bool,
    pub variant_is_only_variant: bool,
    pub arguments_are_empty: bool,
    pub active_union_field_absent: bool,
    pub destination_matches: bool,
    pub arity_matches: bool,
    pub field_types_match: bool,
    pub within_limit: bool,
}

impl StructAggregatePatternVector {
    pub(crate) fn pinned() -> Self {
        Self {
            aggregate_is_adt: true,
            definition_is_local_named_struct: true,
            variant_is_only_variant: true,
            arguments_are_empty: true,
            active_union_field_absent: true,
            destination_matches: true,
            arity_matches: true,
            field_types_match: true,
            within_limit: true,
        }
    }
}

pub(crate) fn validate_struct_aggregate_pattern(
    vector: &StructAggregatePatternVector,
) -> Result<(), MirCode> {
    if vector == &StructAggregatePatternVector::pinned() {
        Ok(())
    } else {
        Err(MirCode::Rvalue)
    }
}

pub(super) struct StructAggregateRef<'a, 'tcx> {
    pub(super) def_id: DefId,
    pub(super) variant: VariantIdx,
    pub(super) arguments: GenericArgsRef<'tcx>,
    pub(super) active_field: Option<FieldIdx>,
    pub(super) operands: &'a IndexVec<FieldIdx, Operand<'tcx>>,
}

pub(super) fn struct_aggregate<'a, 'tcx>(
    rvalue: &'a Rvalue<'tcx>,
) -> Option<StructAggregateRef<'a, 'tcx>> {
    let Rvalue::Aggregate(kind, operands) = rvalue else {
        return None;
    };
    let AggregateKind::Adt(def_id, variant, arguments, _, active_field) = &**kind else {
        return None;
    };
    Some(StructAggregateRef {
        def_id: *def_id,
        variant: *variant,
        arguments,
        active_field: *active_field,
        operands,
    })
}
use super::mir_lower::MirCode;
