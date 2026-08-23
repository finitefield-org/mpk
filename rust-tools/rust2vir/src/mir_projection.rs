use super::hir_check::{contract_type, HirFunction};
use super::mir_lower::MirCode;
use rust2vir_internal::contract::ContractType;
use rustc_index::Idx;
use rustc_middle::mir::{
    AssertKind, BasicBlock, BinOp, Body, Local, Operand, Place, ProjectionElem, Rvalue,
    StatementKind, TerminatorKind,
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::LocalDefId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexPatternVector {
    pub base_is_fixed_array: bool,
    pub index_is_target_usize: bool,
    pub element_is_copy: bool,
    pub predicate_matches: bool,
    pub message_matches: bool,
    pub length_matches: bool,
    pub operand_modes_match: bool,
    pub projection_is_copy: bool,
    pub expected_true: bool,
    pub condition_moved: bool,
    pub continuation_matches: bool,
    pub unwind_unreachable: bool,
    pub assertion_uses: usize,
    pub guard_uses: usize,
    pub index_uses: usize,
    pub projection_uses: usize,
}

impl IndexPatternVector {
    pub(crate) fn pinned() -> Self {
        Self {
            base_is_fixed_array: true,
            index_is_target_usize: true,
            element_is_copy: true,
            predicate_matches: true,
            message_matches: true,
            length_matches: true,
            operand_modes_match: true,
            projection_is_copy: true,
            expected_true: true,
            condition_moved: true,
            continuation_matches: true,
            unwind_unreachable: true,
            assertion_uses: 1,
            guard_uses: 1,
            index_uses: 3,
            projection_uses: 1,
        }
    }
}

pub(crate) fn validate_index_pattern(vector: &IndexPatternVector) -> Result<(), MirCode> {
    if !vector.base_is_fixed_array
        || !vector.index_is_target_usize
        || !vector.element_is_copy
        || !vector.predicate_matches
        || !vector.message_matches
        || !vector.length_matches
        || !vector.operand_modes_match
        || !vector.projection_is_copy
        || !vector.expected_true
        || !vector.condition_moved
        || !vector.continuation_matches
        || !vector.unwind_unreachable
        || vector.assertion_uses != 1
        || vector.guard_uses != 1
        || vector.index_uses != 3
        || vector.projection_uses != 1
    {
        Err(MirCode::Assertion)
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct PlannedIndex {
    base: Local,
    index: Local,
    base_ty: ContractType,
    element_ty: ContractType,
    index_ty: ContractType,
    length: u64,
    assertion_block: usize,
    guard: Local,
    vector: IndexPatternVector,
}

pub(super) struct PlannedIndexRef<'a> {
    pub(super) base: Local,
    pub(super) index: Local,
    pub(super) base_ty: &'a ContractType,
    pub(super) element_ty: &'a ContractType,
    pub(super) index_ty: &'a ContractType,
    pub(super) length: u64,
}

#[derive(Clone, Default)]
pub(super) struct ProjectionPlan {
    guards: BTreeSet<(usize, usize)>,
    indexes: BTreeMap<(usize, usize), PlannedIndex>,
    assertions: BTreeMap<usize, usize>,
}

impl ProjectionPlan {
    pub(super) fn recognize_assert<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        def_id: LocalDefId,
        body: &Body<'tcx>,
        block_index: usize,
        _function: &HirFunction,
    ) -> Result<usize, MirCode> {
        if let Some(target) = self.assertions.get(&block_index) {
            return Ok(*target);
        }
        let block = &body.basic_blocks[BasicBlock::new(block_index)];
        let TerminatorKind::Assert {
            cond,
            expected,
            msg,
            target,
            unwind,
        } = &block.terminator().kind
        else {
            return Err(MirCode::Assertion);
        };
        let AssertKind::BoundsCheck {
            len: message_length,
            index: message_index,
        } = &**msg
        else {
            return Err(MirCode::Assertion);
        };
        let (guard, condition_moved) = plain_operand_local(cond).ok_or(MirCode::Assertion)?;
        let (guard_statement, predicate_index, predicate_length) = block
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| match &statement.kind {
                StatementKind::Assign(assignment)
                    if assignment.0.projection.is_empty() && assignment.0.local == guard =>
                {
                    match &assignment.1 {
                        Rvalue::BinaryOp(BinOp::Lt, operands) => {
                            Some((index, &operands.0, &operands.1))
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .next()
            .ok_or(MirCode::Assertion)?;
        if block
            .statements
            .iter()
            .skip(guard_statement + 1)
            .any(|statement| matches!(statement.kind, StatementKind::Assign(_)))
        {
            return Err(MirCode::Assertion);
        }

        let (operation_statement, destination, projection, projection_is_copy) =
            index_operation_site(body, *target)?;
        let (base, index) = index_place(&projection).ok_or(MirCode::Assertion)?;
        let base_mir_ty = body.local_decls[base].ty;
        let ty::Array(element_mir_ty, _) = base_mir_ty.kind() else {
            return Err(MirCode::Assertion);
        };
        let base_ty = contract_type(tcx, def_id, base_mir_ty).map_err(|_| MirCode::Rvalue)?;
        let (element_ty, length) = match &base_ty {
            ContractType::Array { element, length } => ((**element).clone(), *length),
            _ => return Err(MirCode::Assertion),
        };
        let index_ty =
            contract_type(tcx, def_id, body.local_decls[index].ty).map_err(|_| MirCode::Rvalue)?;
        let typing_env = ty::TypingEnv::post_analysis(tcx, def_id);
        let target_width =
            u8::try_from(tcx.sess.target.pointer_width).map_err(|_| MirCode::Assertion)?;

        let mut vector = IndexPatternVector::pinned();
        vector.base_is_fixed_array = matches!(base_mir_ty.kind(), ty::Array(..));
        vector.index_is_target_usize = body.local_decls[index].ty == tcx.types.usize
            && index_ty
                == ContractType::BitVector {
                    width: target_width,
                    signed: false,
                };
        vector.element_is_copy = tcx.type_is_copy_modulo_regions(typing_env, *element_mir_ty);
        vector.predicate_matches = plain_copy_local(predicate_index) == Some(index);
        vector.message_matches = plain_copy_local(message_index) == Some(index);
        vector.length_matches = usize_constant(tcx, def_id, predicate_length, length)
            && usize_constant(tcx, def_id, message_length, length);
        vector.operand_modes_match = matches!(predicate_length, Operand::Constant(_))
            && matches!(message_length, Operand::Constant(_));
        vector.projection_is_copy = projection_is_copy;
        vector.expected_true = *expected;
        vector.condition_moved = condition_moved;
        vector.continuation_matches = destination.projection.is_empty()
            && body.local_decls[destination.local].ty == *element_mir_ty;
        vector.unwind_unreachable = matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable);
        validate_index_pattern(&vector)?;

        if !self.guards.insert((block_index, guard_statement))
            || self
                .indexes
                .insert(
                    (target.index(), operation_statement),
                    PlannedIndex {
                        base,
                        index,
                        base_ty,
                        element_ty,
                        index_ty,
                        length,
                        assertion_block: block_index,
                        guard,
                        vector,
                    },
                )
                .is_some()
            || self
                .assertions
                .insert(block_index, target.index())
                .is_some()
        {
            return Err(MirCode::Assertion);
        }
        Ok(target.index())
    }

    pub(super) fn finish(&mut self, body: &Body<'_>, order: &[usize]) -> Result<(), MirCode> {
        let reachable = order.iter().copied().collect::<BTreeSet<_>>();
        let mut predecessors = vec![0_usize; body.basic_blocks.len()];
        for block_index in order {
            for successor in body.basic_blocks[BasicBlock::new(*block_index)]
                .terminator()
                .successors()
            {
                if reachable.contains(&successor.index()) {
                    predecessors[successor.index()] += 1;
                }
            }
        }
        for ((block_index, statement_index), index) in &mut self.indexes {
            index.vector.assertion_uses =
                usize::from(self.assertions.contains_key(&index.assertion_block));
            index.vector.guard_uses = local_uses(body, order, index.guard);
            index.vector.index_uses = local_uses(body, order, index.index);
            index.vector.projection_uses =
                index_projection_count(body, order, index.base, index.index);
            index.vector.continuation_matches &= predecessors[*block_index] == 1
                && statement_destination_count(body, order, index.guard) == 1
                && statement_destination_count(body, order, index.index) == 1
                && matches!(
                    body.basic_blocks[BasicBlock::new(*block_index)].statements[*statement_index]
                        .kind,
                    StatementKind::Assign(_)
                );
            validate_index_pattern(&index.vector)?;
        }
        Ok(())
    }

    pub(super) fn is_guard(&self, block: usize, statement: usize) -> bool {
        self.guards.contains(&(block, statement))
    }

    pub(super) fn index(&self, block: usize, statement: usize) -> Option<PlannedIndexRef<'_>> {
        self.indexes
            .get(&(block, statement))
            .map(|planned| PlannedIndexRef {
                base: planned.base,
                index: planned.index,
                base_ty: &planned.base_ty,
                element_ty: &planned.element_ty,
                index_ty: &planned.index_ty,
                length: planned.length,
            })
    }

    pub(super) fn has_indexes(&self) -> bool {
        !self.indexes.is_empty()
    }
}

fn index_operation_site<'tcx>(
    body: &Body<'tcx>,
    block: BasicBlock,
) -> Result<(usize, Place<'tcx>, Place<'tcx>, bool), MirCode> {
    let (statement_index, statement) = body.basic_blocks[block]
        .statements
        .iter()
        .enumerate()
        .find(|(_, statement)| matches!(statement.kind, StatementKind::Assign(_)))
        .ok_or(MirCode::Assertion)?;
    let StatementKind::Assign(assignment) = &statement.kind else {
        unreachable!("selected assignment")
    };
    match &assignment.1 {
        Rvalue::Use(Operand::Copy(place)) => Ok((statement_index, assignment.0, *place, true)),
        Rvalue::Use(Operand::Move(place)) => Ok((statement_index, assignment.0, *place, false)),
        _ => Err(MirCode::Assertion),
    }
}

fn index_place(place: &Place<'_>) -> Option<(Local, Local)> {
    match &place.projection[..] {
        [ProjectionElem::Index(index)] => Some((place.local, *index)),
        _ => None,
    }
}

fn plain_copy_local(operand: &Operand<'_>) -> Option<Local> {
    match operand {
        Operand::Copy(place) if place.projection.is_empty() => Some(place.local),
        _ => None,
    }
}

fn plain_operand_local(operand: &Operand<'_>) -> Option<(Local, bool)> {
    match operand {
        Operand::Copy(place) if place.projection.is_empty() => Some((place.local, false)),
        Operand::Move(place) if place.projection.is_empty() => Some((place.local, true)),
        _ => None,
    }
}

fn usize_constant<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    operand: &Operand<'tcx>,
    expected: u64,
) -> bool {
    let Operand::Constant(constant) = operand else {
        return false;
    };
    constant.const_.ty() == tcx.types.usize
        && constant
            .const_
            .try_eval_target_usize(tcx, ty::TypingEnv::post_analysis(tcx, def_id))
            == Some(expected)
}

fn local_uses(body: &Body<'_>, order: &[usize], local: Local) -> usize {
    let mut count = 0;
    for block_index in order {
        let block = &body.basic_blocks[BasicBlock::new(*block_index)];
        for statement in &block.statements {
            if let StatementKind::Assign(assignment) = &statement.kind {
                count += rvalue_local_uses(&assignment.1, local);
            }
        }
        if let TerminatorKind::Assert { cond, msg, .. } = &block.terminator().kind {
            count += operand_local_uses(cond, local);
            if let AssertKind::BoundsCheck { len, index } = &**msg {
                count += operand_local_uses(len, local);
                count += operand_local_uses(index, local);
            }
        }
    }
    count
}

fn rvalue_local_uses(rvalue: &Rvalue<'_>, local: Local) -> usize {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) => operand_local_uses(operand, local),
        Rvalue::BinaryOp(_, operands) => {
            operand_local_uses(&operands.0, local) + operand_local_uses(&operands.1, local)
        }
        _ => 0,
    }
}

fn operand_local_uses(operand: &Operand<'_>, local: Local) -> usize {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => place_local_uses(place, local),
        Operand::Constant(_) => 0,
    }
}

fn place_local_uses(place: &Place<'_>, local: Local) -> usize {
    usize::from(place.local == local)
        + place
            .projection
            .iter()
            .filter(
                |projection| matches!(projection, ProjectionElem::Index(index) if *index == local),
            )
            .count()
}

fn index_projection_count(body: &Body<'_>, order: &[usize], base: Local, index: Local) -> usize {
    order
        .iter()
        .flat_map(|block| &body.basic_blocks[BasicBlock::new(*block)].statements)
        .filter_map(|statement| match &statement.kind {
            StatementKind::Assign(assignment) => match &assignment.1 {
                Rvalue::Use(Operand::Copy(place)) => Some(place),
                _ => None,
            },
            _ => None,
        })
        .filter(|place| index_place(place) == Some((base, index)))
        .count()
}

fn statement_destination_count(body: &Body<'_>, order: &[usize], local: Local) -> usize {
    order
        .iter()
        .flat_map(|block| &body.basic_blocks[BasicBlock::new(*block)].statements)
        .filter(|statement| {
            matches!(&statement.kind, StatementKind::Assign(assignment) if assignment.0.local == local)
        })
        .count()
}
