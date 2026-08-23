use super::hir_check::{contract_type, HirFunction};
use super::mir_lower::MirCode;
use rust2vir_internal::contract::ContractType;
use rustc_index::Idx;
use rustc_middle::mir::{
    AssertKind, BasicBlock, BinOp, Body, Local, Operand, Place, ProjectionElem, Rvalue,
    StatementKind, TerminatorKind, UnOp,
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::LocalDefId;
use rustc_span::Span;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArithmeticOperation {
    Add,
    Sub,
    Mul,
    Neg,
}

impl ArithmeticOperation {
    pub(super) fn vir_name(self) -> &'static str {
        match self {
            Self::Add => "bv_add",
            Self::Sub => "bv_sub",
            Self::Mul => "bv_mul",
            Self::Neg => "bv_neg",
        }
    }

    pub(super) fn safety_name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Neg => "neg",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PatternVector {
    pub operation: ArithmeticOperation,
    pub value_operation: ArithmeticOperation,
    pub lhs_matches: bool,
    pub rhs_matches: bool,
    pub type_matches: bool,
    pub tuple_matches: bool,
    pub operand_modes_match: bool,
    pub flag_field: u32,
    pub flag_moved: bool,
    pub result_field: u32,
    pub result_moved: bool,
    pub expected: bool,
    pub message_matches: bool,
    pub continuation_matches: bool,
    pub unwind_unreachable: bool,
    pub assertion_uses: usize,
    pub flag_uses: usize,
    pub result_uses: usize,
}

impl PatternVector {
    pub(crate) fn pinned(operation: ArithmeticOperation) -> Self {
        Self {
            operation,
            value_operation: operation,
            lhs_matches: true,
            rhs_matches: true,
            type_matches: true,
            tuple_matches: true,
            operand_modes_match: true,
            flag_field: 1,
            flag_moved: true,
            result_field: 0,
            result_moved: true,
            expected: false,
            message_matches: true,
            continuation_matches: true,
            unwind_unreachable: true,
            assertion_uses: 1,
            flag_uses: 1,
            result_uses: 1,
        }
    }
}

pub(crate) fn validate_pattern_vector(vector: &PatternVector) -> Result<(), MirCode> {
    if vector.operation != vector.value_operation
        || !vector.lhs_matches
        || !vector.rhs_matches
        || !vector.type_matches
        || !vector.tuple_matches
        || !vector.operand_modes_match
        || vector.flag_field != 1
        || !vector.flag_moved
        || vector.result_field != 0
        || !vector.result_moved
        || vector.expected
        || !vector.message_matches
        || !vector.continuation_matches
        || !vector.unwind_unreachable
        || vector.assertion_uses != 1
        || vector.flag_uses != 1
        || vector.result_uses != 1
    {
        Err(MirCode::Assertion)
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct PlannedBinary {
    operation: ArithmeticOperation,
    ty: ContractType,
    tuple: Local,
    vector: PatternVector,
}

#[derive(Clone)]
struct PlannedNegation {
    ty: ContractType,
    guard: Local,
    vector: PatternVector,
}

#[derive(Clone, Default)]
pub(super) struct ArithmeticPlan {
    binaries: BTreeMap<(usize, usize), PlannedBinary>,
    negation_guards: BTreeSet<(usize, usize)>,
    negations: BTreeMap<(usize, usize), PlannedNegation>,
    assertions: BTreeMap<usize, usize>,
    scalar_locals: BTreeMap<usize, ContractType>,
}

impl ArithmeticPlan {
    pub(super) fn recognize_assert<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        def_id: LocalDefId,
        body: &Body<'tcx>,
        block_index: usize,
        function: &HirFunction,
    ) -> Result<usize, MirCode> {
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
        let unwind_unreachable = matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable);
        if !unwind_unreachable {
            return Err(MirCode::Cleanup);
        }

        match &**msg {
            AssertKind::Overflow(message_op, message_lhs, message_rhs) => {
                let operation = message_operation(*message_op).ok_or(MirCode::Assertion)?;
                let (statement_index, destination, value_op, value_lhs, value_rhs) = block
                    .statements
                    .iter()
                    .enumerate()
                    .filter_map(|(index, statement)| match &statement.kind {
                        StatementKind::Assign(assignment) => match &assignment.1 {
                            Rvalue::BinaryOp(value_op, operands)
                                if checked_operation(*value_op).is_some() =>
                            {
                                Some((index, assignment.0, *value_op, &operands.0, &operands.1))
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .next()
                    .ok_or(MirCode::Assertion)?;
                if block
                    .statements
                    .iter()
                    .skip(statement_index + 1)
                    .any(|statement| matches!(statement.kind, StatementKind::Assign(_)))
                {
                    return Err(MirCode::Assertion);
                }
                let value_operation = checked_operation(value_op).ok_or(MirCode::Assertion)?;
                let tuple = destination.local;
                let (flag_local, flag_field, flag_moved) =
                    projected_local(cond).ok_or(MirCode::Assertion)?;
                let ty = operand_contract_type(tcx, def_id, body, value_lhs)?;
                let rhs_ty = operand_contract_type(tcx, def_id, body, value_rhs)?;
                let tuple_matches = destination.projection.is_empty()
                    && tuple.index() > body.arg_count
                    && flag_local == tuple
                    && checked_tuple_type(tcx, def_id, body.local_decls[tuple].ty, &ty);
                let (result_place, result_moved) = continuation_result_place(body, *target, tuple)?;
                let (_, result_field) = place_field(&result_place).ok_or(MirCode::Assertion)?;
                let mut vector = PatternVector::pinned(operation);
                vector.value_operation = value_operation;
                vector.lhs_matches = same_value_operand(value_lhs, message_lhs);
                vector.rhs_matches = same_value_operand(value_rhs, message_rhs);
                vector.operand_modes_match = checked_binary_operand_mode(value_lhs, message_lhs)
                    && checked_binary_operand_mode(value_rhs, message_rhs);
                vector.type_matches = ty == rhs_ty
                    && operand_contract_type(tcx, def_id, body, message_lhs)? == ty
                    && operand_contract_type(tcx, def_id, body, message_rhs)? == ty
                    && supported_integer(&ty);
                vector.tuple_matches = tuple_matches;
                vector.flag_field = flag_field;
                vector.flag_moved = flag_moved;
                vector.result_field = result_field;
                vector.result_moved = result_moved;
                vector.expected = *expected;
                vector.message_matches = operation == value_operation;
                vector.continuation_matches = true;
                vector.unwind_unreachable = unwind_unreachable;
                validate_pattern_vector(&vector)?;
                if self
                    .scalar_locals
                    .insert(tuple.index(), ty.clone())
                    .is_some()
                {
                    return Err(MirCode::Assertion);
                }
                self.binaries.insert(
                    (block_index, statement_index),
                    PlannedBinary {
                        operation,
                        ty,
                        tuple,
                        vector,
                    },
                );
            }
            AssertKind::OverflowNeg(message_operand) => {
                let (guard, flag_moved) = plain_operand_local(cond).ok_or(MirCode::Assertion)?;
                let (guard_statement, eq_lhs, eq_rhs) = block
                    .statements
                    .iter()
                    .enumerate()
                    .filter_map(|(index, statement)| match &statement.kind {
                        StatementKind::Assign(assignment)
                            if assignment.0.projection.is_empty()
                                && assignment.0.local == guard =>
                        {
                            match &assignment.1 {
                                Rvalue::BinaryOp(BinOp::Eq, operands) => {
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
                let ty = operand_contract_type(tcx, def_id, body, message_operand)?;
                let (neg_block, neg_statement, neg_operand, result_moved) =
                    continuation_negation(body, *target)?;
                let mut vector = PatternVector::pinned(ArithmeticOperation::Neg);
                vector.lhs_matches = same_value_operand(eq_lhs, message_operand)
                    && same_value_operand(neg_operand, message_operand);
                vector.rhs_matches = minimum_constant(tcx, def_id, eq_rhs, &ty);
                vector.operand_modes_match = matches!(eq_lhs, Operand::Copy(_))
                    && matches!(message_operand, Operand::Copy(_));
                vector.type_matches = supported_signed_integer(&ty)
                    && operand_contract_type(tcx, def_id, body, eq_lhs)? == ty
                    && operand_contract_type(tcx, def_id, body, eq_rhs)? == ty
                    && operand_contract_type(tcx, def_id, body, neg_operand)? == ty;
                vector.tuple_matches = body.local_decls[guard].ty.is_bool();
                vector.flag_moved = flag_moved;
                vector.result_moved = result_moved;
                vector.expected = *expected;
                vector.unwind_unreachable = unwind_unreachable;
                vector.continuation_matches = function.checked_negation_spans.iter().any(|span| {
                    span_contains(
                        *span,
                        body.basic_blocks[neg_block].statements[neg_statement]
                            .source_info
                            .span,
                    )
                });
                validate_pattern_vector(&vector)?;
                if !self.negation_guards.insert((block_index, guard_statement))
                    || self
                        .negations
                        .insert(
                            (neg_block.index(), neg_statement),
                            PlannedNegation { ty, guard, vector },
                        )
                        .is_some()
                {
                    return Err(MirCode::Assertion);
                }
            }
            _ => return Err(MirCode::Assertion),
        }
        if self
            .assertions
            .insert(block_index, target.index())
            .is_some()
        {
            return Err(MirCode::Assertion);
        }
        Ok(target.index())
    }

    pub(super) fn finish<'tcx>(
        &mut self,
        body: &Body<'tcx>,
        order: &[usize],
    ) -> Result<(), MirCode> {
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
        for ((block_index, statement_index), binary) in &mut self.binaries {
            let (flag_uses, result_uses, other_uses) =
                local_projection_uses(body, order, binary.tuple);
            binary.vector.assertion_uses = self
                .assertions
                .keys()
                .filter(|assertion| **assertion == *block_index)
                .count();
            binary.vector.flag_uses = flag_uses;
            binary.vector.result_uses = result_uses;
            binary.vector.continuation_matches &= other_uses == 0
                && predecessors[self.assertions[block_index]] == 1
                && statement_destination_count(body, order, binary.tuple) == 1
                && matches!(
                    body.basic_blocks[BasicBlock::new(*block_index)].statements[*statement_index]
                        .kind,
                    StatementKind::Assign(_)
                )
                && result_uses == 1;
            validate_pattern_vector(&binary.vector)?;
        }
        for ((block_index, statement_index), negation) in &mut self.negations {
            let guard_uses = plain_local_uses(body, order, negation.guard);
            negation.vector.flag_uses = guard_uses;
            negation.vector.result_uses = 1;
            negation.vector.assertion_uses = self
                .assertions
                .iter()
                .filter(|(_, target)| **target == *block_index)
                .count();
            negation.vector.continuation_matches &= predecessors[*block_index] == 1
                && statement_destination_count(body, order, negation.guard) == 1
                && matches!(
                    body.basic_blocks[BasicBlock::new(*block_index)].statements[*statement_index]
                        .kind,
                    StatementKind::Assign(_)
                );
            validate_pattern_vector(&negation.vector)?;
        }
        for block_index in order {
            for (statement_index, statement) in body.basic_blocks[BasicBlock::new(*block_index)]
                .statements
                .iter()
                .enumerate()
            {
                let StatementKind::Assign(assignment) = &statement.kind else {
                    continue;
                };
                let planned = self.binaries.contains_key(&(*block_index, statement_index))
                    || self
                        .negation_guards
                        .contains(&(*block_index, statement_index))
                    || self
                        .negations
                        .contains_key(&(*block_index, statement_index));
                if matches!(
                    assignment.1,
                    Rvalue::BinaryOp(
                        BinOp::AddWithOverflow | BinOp::SubWithOverflow | BinOp::MulWithOverflow,
                        _
                    ) | Rvalue::UnaryOp(UnOp::Neg, _)
                ) && !planned
                {
                    return Err(MirCode::Assertion);
                }
            }
        }
        Ok(())
    }

    pub(super) fn binary(
        &self,
        block: usize,
        statement: usize,
    ) -> Option<(ArithmeticOperation, &ContractType)> {
        self.binaries
            .get(&(block, statement))
            .map(|planned| (planned.operation, &planned.ty))
    }

    pub(super) fn is_negation_guard(&self, block: usize, statement: usize) -> bool {
        self.negation_guards.contains(&(block, statement))
    }

    pub(super) fn negation(&self, block: usize, statement: usize) -> Option<&ContractType> {
        self.negations
            .get(&(block, statement))
            .map(|planned| &planned.ty)
    }

    pub(super) fn projected_type(&self, place: &Place<'_>) -> Option<&ContractType> {
        let (local, field) = place_field(place)?;
        (field == 0)
            .then(|| self.scalar_locals.get(&local.index()))
            .flatten()
    }

    pub(super) fn scalar_local_type(&self, local: Local) -> Option<&ContractType> {
        self.scalar_locals.get(&local.index())
    }
}

fn message_operation(operation: BinOp) -> Option<ArithmeticOperation> {
    match operation {
        BinOp::Add => Some(ArithmeticOperation::Add),
        BinOp::Sub => Some(ArithmeticOperation::Sub),
        BinOp::Mul => Some(ArithmeticOperation::Mul),
        _ => None,
    }
}

fn checked_operation(operation: BinOp) -> Option<ArithmeticOperation> {
    match operation {
        BinOp::AddWithOverflow => Some(ArithmeticOperation::Add),
        BinOp::SubWithOverflow => Some(ArithmeticOperation::Sub),
        BinOp::MulWithOverflow => Some(ArithmeticOperation::Mul),
        _ => None,
    }
}

fn operand_contract_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    operand: &Operand<'tcx>,
) -> Result<ContractType, MirCode> {
    let ty = match operand {
        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => {
            body.local_decls[place.local].ty
        }
        Operand::Constant(constant) => constant.const_.ty(),
        _ => return Err(MirCode::Assertion),
    };
    contract_type(tcx, def_id, ty).map_err(|_| MirCode::Rvalue)
}

fn supported_integer(ty: &ContractType) -> bool {
    matches!(
        ty,
        ContractType::BitVector {
            width: 8 | 16 | 32 | 64,
            ..
        }
    )
}

fn supported_signed_integer(ty: &ContractType) -> bool {
    matches!(
        ty,
        ContractType::BitVector {
            width: 8 | 16 | 32 | 64,
            signed: true
        }
    )
}

fn checked_tuple_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    ty: ty::Ty<'tcx>,
    scalar: &ContractType,
) -> bool {
    if !matches!(ty.kind(), ty::Tuple(_)) {
        return false;
    }
    let fields = ty.tuple_fields();
    fields.len() == 2
        && fields[1].is_bool()
        && contract_type(tcx, def_id, fields[0]).ok().as_ref() == Some(scalar)
}

fn same_value_operand<'tcx>(left: &Operand<'tcx>, right: &Operand<'tcx>) -> bool {
    match (left, right) {
        (
            Operand::Copy(left) | Operand::Move(left),
            Operand::Copy(right) | Operand::Move(right),
        ) => left == right,
        (Operand::Constant(left), Operand::Constant(right)) => left.const_ == right.const_,
        _ => false,
    }
}

fn checked_binary_operand_mode(left: &Operand<'_>, right: &Operand<'_>) -> bool {
    matches!((left, right), (Operand::Copy(_), Operand::Move(_)))
        || matches!((left, right), (Operand::Constant(_), Operand::Constant(_)))
}

fn projected_local(operand: &Operand<'_>) -> Option<(Local, u32, bool)> {
    match operand {
        Operand::Copy(place) => place_field(place).map(|(local, field)| (local, field, false)),
        Operand::Move(place) => place_field(place).map(|(local, field)| (local, field, true)),
        Operand::Constant(_) => None,
    }
}

fn plain_operand_local(operand: &Operand<'_>) -> Option<(Local, bool)> {
    match operand {
        Operand::Copy(place) if place.projection.is_empty() => Some((place.local, false)),
        Operand::Move(place) if place.projection.is_empty() => Some((place.local, true)),
        _ => None,
    }
}

fn place_field(place: &Place<'_>) -> Option<(Local, u32)> {
    match &place.projection[..] {
        [ProjectionElem::Field(field, _)] => Some((place.local, field.as_u32())),
        _ => None,
    }
}

fn place_is_field(place: Place<'_>, local: Local, field: u32) -> bool {
    place_field(&place) == Some((local, field))
}

fn continuation_result_place<'tcx>(
    body: &Body<'tcx>,
    target: BasicBlock,
    tuple: Local,
) -> Result<(Place<'tcx>, bool), MirCode> {
    let first_assignment = body.basic_blocks[target]
        .statements
        .iter()
        .find(|statement| matches!(statement.kind, StatementKind::Assign(_)))
        .ok_or(MirCode::Assertion)?;
    let StatementKind::Assign(assignment) = &first_assignment.kind else {
        unreachable!("selected assignment")
    };
    match &assignment.1 {
        Rvalue::Use(Operand::Copy(place)) if place_is_field(*place, tuple, 0) => {
            Ok((*place, false))
        }
        Rvalue::Use(Operand::Move(place)) if place_is_field(*place, tuple, 0) => Ok((*place, true)),
        _ => Err(MirCode::Assertion),
    }
}

fn continuation_negation<'a, 'tcx>(
    body: &'a Body<'tcx>,
    target: BasicBlock,
) -> Result<(BasicBlock, usize, &'a Operand<'tcx>, bool), MirCode> {
    let (index, first_assignment) = body.basic_blocks[target]
        .statements
        .iter()
        .enumerate()
        .find(|(_, statement)| matches!(statement.kind, StatementKind::Assign(_)))
        .ok_or(MirCode::Assertion)?;
    let StatementKind::Assign(assignment) = &first_assignment.kind else {
        unreachable!("selected assignment")
    };
    match &assignment.1 {
        Rvalue::UnaryOp(UnOp::Neg, operand) => {
            Ok((target, index, operand, matches!(operand, Operand::Move(_))))
        }
        _ => Err(MirCode::Assertion),
    }
}

fn minimum_constant<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    operand: &Operand<'tcx>,
    ty: &ContractType,
) -> bool {
    let Operand::Constant(constant) = operand else {
        return false;
    };
    let ContractType::BitVector {
        width,
        signed: true,
    } = ty
    else {
        return false;
    };
    constant
        .const_
        .try_eval_bits(tcx, ty::TypingEnv::post_analysis(tcx, def_id))
        == Some(1_u128 << (width - 1))
}

fn local_projection_uses(body: &Body<'_>, order: &[usize], local: Local) -> (usize, usize, usize) {
    let mut flag = 0;
    let mut result = 0;
    let mut other = 0;
    for block_index in order {
        let block = &body.basic_blocks[BasicBlock::new(*block_index)];
        for statement in &block.statements {
            if let StatementKind::Assign(assignment) = &statement.kind {
                for place in rvalue_places(&assignment.1) {
                    count_projection(place, local, &mut flag, &mut result, &mut other);
                }
            }
        }
        match &block.terminator().kind {
            TerminatorKind::Assert { cond, msg, .. } => {
                if let Some(place) = operand_place(cond) {
                    count_projection(place, local, &mut flag, &mut result, &mut other);
                }
                match &**msg {
                    AssertKind::Overflow(_, lhs, rhs) => {
                        for operand in [lhs, rhs] {
                            if let Some(place) = operand_place(operand) {
                                count_projection(place, local, &mut flag, &mut result, &mut other);
                            }
                        }
                    }
                    AssertKind::OverflowNeg(operand) => {
                        if let Some(place) = operand_place(operand) {
                            count_projection(place, local, &mut flag, &mut result, &mut other);
                        }
                    }
                    _ => {}
                }
            }
            TerminatorKind::SwitchInt { discr, .. } => {
                if let Some(place) = operand_place(discr) {
                    count_projection(place, local, &mut flag, &mut result, &mut other);
                }
            }
            _ => {}
        }
    }
    (flag, result, other)
}

fn count_projection(
    place: &Place<'_>,
    local: Local,
    flag: &mut usize,
    result: &mut usize,
    other: &mut usize,
) {
    if place.local != local {
        return;
    }
    match place_field(place) {
        Some((_, 1)) => *flag += 1,
        Some((_, 0)) => *result += 1,
        _ => *other += 1,
    }
}

fn plain_local_uses(body: &Body<'_>, order: &[usize], local: Local) -> usize {
    let mut count = 0;
    for block_index in order {
        let block = &body.basic_blocks[BasicBlock::new(*block_index)];
        for statement in &block.statements {
            if let StatementKind::Assign(assignment) = &statement.kind {
                count += rvalue_places(&assignment.1)
                    .into_iter()
                    .filter(|place| place.local == local)
                    .count();
            }
        }
        if let TerminatorKind::Assert { cond, .. } = &block.terminator().kind {
            count += usize::from(operand_place(cond).is_some_and(|place| place.local == local));
        }
    }
    count
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

fn rvalue_places<'a, 'tcx>(rvalue: &'a Rvalue<'tcx>) -> Vec<&'a Place<'tcx>> {
    let operands: Vec<&Operand<'tcx>> = match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) => vec![operand],
        Rvalue::BinaryOp(_, operands) => vec![&operands.0, &operands.1],
        _ => Vec::new(),
    };
    operands.into_iter().filter_map(operand_place).collect()
}

fn operand_place<'a, 'tcx>(operand: &'a Operand<'tcx>) -> Option<&'a Place<'tcx>> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => Some(place),
        Operand::Constant(_) => None,
    }
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.lo() < outer.hi()
        && inner.lo() < inner.hi()
        && outer.lo() <= inner.lo()
        && inner.hi() <= outer.hi()
}
