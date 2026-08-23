use super::hir_check::HirFunction;
use super::mir_lower::MirCode;
use super::type_lower::contract_type;
use rust2vir_internal::contract::ContractType;
use rustc_index::Idx;
use rustc_middle::mir::{
    AssertKind, BasicBlock, BinOp, Body, CastKind, Local, Operand, Place, ProjectionElem, Rvalue,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DivRemOperation {
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShiftOperation {
    Shl,
    Shr,
}

impl ShiftOperation {
    pub(super) fn vir_name(self, lhs_signed: bool) -> &'static str {
        match (self, lhs_signed) {
            (Self::Shl, _) => "bv_shl",
            (Self::Shr, true) => "bv_ashr",
            (Self::Shr, false) => "bv_lshr",
        }
    }
}

impl DivRemOperation {
    pub(super) fn safety_name(self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Rem => "rem",
        }
    }

    pub(super) fn vir_name(self, signed: bool) -> &'static str {
        match (self, signed) {
            (Self::Div, true) => "bv_sdiv",
            (Self::Div, false) => "bv_udiv",
            (Self::Rem, true) => "bv_srem",
            (Self::Rem, false) => "bv_urem",
        }
    }
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
        Err(MirCode::CheckedPattern)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DivRemPatternVector {
    pub operation: DivRemOperation,
    pub value_operation: DivRemOperation,
    pub signed: bool,
    pub operands_match: bool,
    pub operand_modes_match: bool,
    pub type_matches: bool,
    pub zero_guard_matches: bool,
    pub zero_message_matches: bool,
    pub representability_guard_matches: bool,
    pub representability_message_matches: bool,
    pub guard_order_matches: bool,
    pub expected_false: bool,
    pub conditions_moved: bool,
    pub unwind_unreachable: bool,
    pub continuation_matches: bool,
    pub assertion_uses: usize,
    pub guard_uses_match: bool,
}

impl DivRemPatternVector {
    pub(crate) fn pinned(operation: DivRemOperation, signed: bool) -> Self {
        Self {
            operation,
            value_operation: operation,
            signed,
            operands_match: true,
            operand_modes_match: true,
            type_matches: true,
            zero_guard_matches: true,
            zero_message_matches: true,
            representability_guard_matches: signed,
            representability_message_matches: signed,
            guard_order_matches: true,
            expected_false: true,
            conditions_moved: true,
            unwind_unreachable: true,
            continuation_matches: true,
            assertion_uses: if signed { 2 } else { 1 },
            guard_uses_match: true,
        }
    }
}

pub(crate) fn validate_div_rem_pattern(vector: &DivRemPatternVector) -> Result<(), MirCode> {
    let expected_assertions = if vector.signed { 2 } else { 1 };
    if vector.operation != vector.value_operation
        || !vector.operands_match
        || !vector.operand_modes_match
        || !vector.type_matches
        || !vector.zero_guard_matches
        || !vector.zero_message_matches
        || vector.representability_guard_matches != vector.signed
        || vector.representability_message_matches != vector.signed
        || !vector.guard_order_matches
        || !vector.expected_false
        || !vector.conditions_moved
        || !vector.unwind_unreachable
        || !vector.continuation_matches
        || vector.assertion_uses != expected_assertions
        || !vector.guard_uses_match
    {
        Err(MirCode::CheckedPattern)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShiftPatternVector {
    pub operation: ShiftOperation,
    pub value_operation: ShiftOperation,
    pub signed_rhs: bool,
    pub operands_match: bool,
    pub operand_modes_match: bool,
    pub lhs_type_matches: bool,
    pub rhs_type_matches: bool,
    pub cast_matches: bool,
    pub predicate_matches: bool,
    pub threshold_matches: bool,
    pub message_matches: bool,
    pub expected_true: bool,
    pub condition_moved: bool,
    pub unwind_unreachable: bool,
    pub continuation_matches: bool,
    pub assertion_uses: usize,
    pub guard_uses_match: bool,
}

impl ShiftPatternVector {
    pub(crate) fn pinned(operation: ShiftOperation, signed_rhs: bool) -> Self {
        Self {
            operation,
            value_operation: operation,
            signed_rhs,
            operands_match: true,
            operand_modes_match: true,
            lhs_type_matches: true,
            rhs_type_matches: true,
            cast_matches: signed_rhs,
            predicate_matches: true,
            threshold_matches: true,
            message_matches: true,
            expected_true: true,
            condition_moved: true,
            unwind_unreachable: true,
            continuation_matches: true,
            assertion_uses: 1,
            guard_uses_match: true,
        }
    }
}

pub(crate) fn validate_shift_pattern(vector: &ShiftPatternVector) -> Result<(), MirCode> {
    if vector.operation != vector.value_operation
        || !vector.operands_match
        || !vector.operand_modes_match
        || !vector.lhs_type_matches
        || !vector.rhs_type_matches
        || vector.cast_matches != vector.signed_rhs
        || !vector.predicate_matches
        || !vector.threshold_matches
        || !vector.message_matches
        || !vector.expected_true
        || !vector.condition_moved
        || !vector.unwind_unreachable
        || !vector.continuation_matches
        || vector.assertion_uses != 1
        || !vector.guard_uses_match
    {
        Err(MirCode::CheckedPattern)
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

#[derive(Clone)]
struct PlannedDivRem {
    operation: DivRemOperation,
    ty: ContractType,
    zero_block: usize,
    overflow_block: Option<usize>,
    guards: Vec<Local>,
    vector: DivRemPatternVector,
}

#[derive(Clone)]
struct PlannedShift {
    operation: ShiftOperation,
    lhs_ty: ContractType,
    rhs_ty: ContractType,
    assertion_block: usize,
    guards: Vec<Local>,
    vector: ShiftPatternVector,
}

struct DivRemOperationSite<'a, 'tcx> {
    block: BasicBlock,
    statement: usize,
    destination: Place<'tcx>,
    operation: DivRemOperation,
    lhs: &'a Operand<'tcx>,
    rhs: &'a Operand<'tcx>,
}

struct SignedDivRemSite<'a, 'tcx> {
    block: BasicBlock,
    target: BasicBlock,
    statements: [usize; 3],
    guards: [Local; 3],
    rhs_minus_one: &'a Operand<'tcx>,
    lhs_minimum: &'a Operand<'tcx>,
    message_lhs: &'a Operand<'tcx>,
    message_rhs: &'a Operand<'tcx>,
    operation: DivRemOperation,
    expected_false: bool,
    conditions_moved: bool,
    unwind_unreachable: bool,
}

struct ShiftOperationSite<'a, 'tcx> {
    block: BasicBlock,
    statement: usize,
    destination: Place<'tcx>,
    operation: ShiftOperation,
    lhs: &'a Operand<'tcx>,
    rhs: &'a Operand<'tcx>,
}

#[derive(Clone, Default)]
pub(super) struct ArithmeticPlan {
    binaries: BTreeMap<(usize, usize), PlannedBinary>,
    negation_guards: BTreeSet<(usize, usize)>,
    negations: BTreeMap<(usize, usize), PlannedNegation>,
    div_rem_guards: BTreeSet<(usize, usize)>,
    div_rems: BTreeMap<(usize, usize), PlannedDivRem>,
    shift_guards: BTreeSet<(usize, usize)>,
    shifts: BTreeMap<(usize, usize), PlannedShift>,
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
        let unwind_unreachable = matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable);
        if !unwind_unreachable {
            return Err(MirCode::Cleanup);
        }

        match &**msg {
            AssertKind::DivisionByZero(message_lhs) => {
                return self.recognize_div_rem(
                    tcx,
                    def_id,
                    body,
                    block_index,
                    DivRemOperation::Div,
                    message_lhs,
                );
            }
            AssertKind::RemainderByZero(message_lhs) => {
                return self.recognize_div_rem(
                    tcx,
                    def_id,
                    body,
                    block_index,
                    DivRemOperation::Rem,
                    message_lhs,
                );
            }
            AssertKind::Overflow(message_op, message_lhs, message_rhs)
                if primitive_shift(*message_op).is_some() =>
            {
                return self.recognize_shift(
                    tcx,
                    def_id,
                    body,
                    block_index,
                    primitive_shift(*message_op).expect("guarded shift operation"),
                    message_lhs,
                    message_rhs,
                );
            }
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

    fn recognize_div_rem<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        def_id: LocalDefId,
        body: &Body<'tcx>,
        zero_block_index: usize,
        operation: DivRemOperation,
        zero_message_lhs: &Operand<'tcx>,
    ) -> Result<usize, MirCode> {
        let zero_block = &body.basic_blocks[BasicBlock::new(zero_block_index)];
        let TerminatorKind::Assert {
            cond,
            expected,
            target: zero_target,
            unwind,
            ..
        } = &zero_block.terminator().kind
        else {
            return Err(MirCode::Assertion);
        };
        let (zero_guard, zero_condition_moved) =
            plain_operand_local(cond).ok_or(MirCode::Assertion)?;
        let (zero_statement, zero_rhs, zero_constant_operand) = zero_block
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| match &statement.kind {
                StatementKind::Assign(assignment)
                    if assignment.0.projection.is_empty() && assignment.0.local == zero_guard =>
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
        if zero_block
            .statements
            .iter()
            .skip(zero_statement + 1)
            .any(|statement| matches!(statement.kind, StatementKind::Assign(_)))
        {
            return Err(MirCode::Assertion);
        }
        let provisional_type = operand_contract_type(tcx, def_id, body, zero_rhs)?;
        let signed = matches!(
            provisional_type,
            ContractType::BitVector { signed: true, .. }
        );
        let signed_site = if signed {
            Some(signed_div_rem_site(body, *zero_target, operation)?)
        } else {
            None
        };
        let operation_target = signed_site
            .as_ref()
            .map_or(*zero_target, |site| site.target);
        let operation_site = div_rem_operation_site(body, operation_target)?;
        let ty = operand_contract_type(tcx, def_id, body, operation_site.lhs)?;
        let rhs_ty = operand_contract_type(tcx, def_id, body, operation_site.rhs)?;
        let mut vector = DivRemPatternVector::pinned(operation, signed);
        vector.value_operation = operation_site.operation;
        vector.operands_match = same_value_operand(zero_rhs, operation_site.rhs)
            && same_value_operand(zero_message_lhs, operation_site.lhs);
        vector.operand_modes_match = guard_operand_mode(zero_rhs)
            && guard_operand_mode(zero_message_lhs)
            && operation_operand_mode(operation_site.lhs)
            && operation_operand_mode(operation_site.rhs);
        vector.type_matches = ty == rhs_ty
            && ty == provisional_type
            && operand_contract_type(tcx, def_id, body, zero_message_lhs)? == ty
            && supported_integer(&ty);
        vector.zero_guard_matches = body.local_decls[zero_guard].ty.is_bool()
            && integer_constant(
                tcx,
                def_id,
                zero_constant_operand,
                &ty,
                IntegerBoundary::Zero,
            );
        vector.zero_message_matches = operation_site.operation == operation;
        vector.expected_false = !*expected;
        vector.conditions_moved = zero_condition_moved;
        vector.unwind_unreachable = matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable);
        vector.continuation_matches = operation_site.destination.projection.is_empty();

        let mut guards = vec![zero_guard];
        let mut overflow_block = None;
        if let Some(site) = &signed_site {
            vector.operands_match &= same_value_operand(site.rhs_minus_one, operation_site.rhs)
                && same_value_operand(site.lhs_minimum, operation_site.lhs)
                && same_value_operand(site.message_lhs, operation_site.lhs)
                && same_value_operand(site.message_rhs, operation_site.rhs);
            vector.operand_modes_match &= guard_operand_mode(site.rhs_minus_one)
                && guard_operand_mode(site.lhs_minimum)
                && guard_operand_mode(site.message_lhs)
                && guard_operand_mode(site.message_rhs);
            vector.type_matches &= operand_contract_type(tcx, def_id, body, site.rhs_minus_one)?
                == ty
                && operand_contract_type(tcx, def_id, body, site.lhs_minimum)? == ty
                && operand_contract_type(tcx, def_id, body, site.message_lhs)? == ty
                && operand_contract_type(tcx, def_id, body, site.message_rhs)? == ty;
            vector.representability_guard_matches = integer_constant(
                tcx,
                def_id,
                signed_guard_constant(body, site.block, site.statements[0])?,
                &ty,
                IntegerBoundary::NegativeOne,
            ) && integer_constant(
                tcx,
                def_id,
                signed_guard_constant(body, site.block, site.statements[1])?,
                &ty,
                IntegerBoundary::Minimum,
            );
            vector.representability_message_matches = site.operation == operation;
            vector.guard_order_matches = signed_guard_order(body, site);
            vector.expected_false &= site.expected_false;
            vector.conditions_moved &= site.conditions_moved;
            vector.unwind_unreachable &= site.unwind_unreachable;
            guards.extend(site.guards);
            overflow_block = Some(site.block.index());
        }
        validate_div_rem_pattern(&vector)?;

        if !self
            .div_rem_guards
            .insert((zero_block_index, zero_statement))
        {
            return Err(MirCode::Assertion);
        }
        if let Some(site) = &signed_site {
            for statement in site.statements {
                if !self.div_rem_guards.insert((site.block.index(), statement)) {
                    return Err(MirCode::Assertion);
                }
            }
        }
        if self
            .assertions
            .insert(zero_block_index, zero_target.index())
            .is_some()
        {
            return Err(MirCode::Assertion);
        }
        if let Some(site) = &signed_site {
            if self
                .assertions
                .insert(site.block.index(), site.target.index())
                .is_some()
            {
                return Err(MirCode::Assertion);
            }
        }
        if self
            .div_rems
            .insert(
                (operation_site.block.index(), operation_site.statement),
                PlannedDivRem {
                    operation,
                    ty,
                    zero_block: zero_block_index,
                    overflow_block,
                    guards,
                    vector,
                },
            )
            .is_some()
        {
            return Err(MirCode::Assertion);
        }
        Ok(zero_target.index())
    }

    #[allow(clippy::too_many_arguments)]
    fn recognize_shift<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        def_id: LocalDefId,
        body: &Body<'tcx>,
        assertion_block_index: usize,
        operation: ShiftOperation,
        message_lhs: &Operand<'tcx>,
        message_rhs: &Operand<'tcx>,
    ) -> Result<usize, MirCode> {
        let block = &body.basic_blocks[BasicBlock::new(assertion_block_index)];
        let TerminatorKind::Assert {
            cond,
            expected,
            target,
            unwind,
            ..
        } = &block.terminator().kind
        else {
            return Err(MirCode::Assertion);
        };
        let (predicate_guard, condition_moved) =
            plain_operand_local(cond).ok_or(MirCode::Assertion)?;
        let (predicate_statement, predicate_count, threshold) = block
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| match &statement.kind {
                StatementKind::Assign(assignment)
                    if assignment.0.projection.is_empty()
                        && assignment.0.local == predicate_guard =>
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
            .skip(predicate_statement + 1)
            .any(|statement| matches!(statement.kind, StatementKind::Assign(_)))
        {
            return Err(MirCode::Assertion);
        }

        let lhs_ty = operand_contract_type(tcx, def_id, body, message_lhs)?;
        let rhs_ty = operand_contract_type(tcx, def_id, body, message_rhs)?;
        let signed_rhs = matches!(rhs_ty, ContractType::BitVector { signed: true, .. });
        let mut guard_locations = vec![(assertion_block_index, predicate_statement)];
        let mut guards = vec![predicate_guard];
        let (original_guard_count, cast_matches, predicate_mode_matches) = if signed_rhs {
            let (cast_local, cast_value_moved) =
                plain_operand_local(predicate_count).ok_or(MirCode::Assertion)?;
            let (cast_statement, cast_source, cast_type, cast_kind) = block
                .statements
                .iter()
                .enumerate()
                .take(predicate_statement)
                .rev()
                .find_map(|(index, statement)| match &statement.kind {
                    StatementKind::Assign(assignment)
                        if assignment.0.projection.is_empty()
                            && assignment.0.local == cast_local =>
                    {
                        match &assignment.1 {
                            Rvalue::Cast(kind, source, ty) => Some((index, source, *ty, *kind)),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .ok_or(MirCode::Assertion)?;
            let no_intervening_assignment = block
                .statements
                .iter()
                .enumerate()
                .skip(cast_statement + 1)
                .take(predicate_statement - cast_statement - 1)
                .all(|(_, statement)| !matches!(statement.kind, StatementKind::Assign(_)));
            let expected_cast_type = unsigned_equivalent(&rhs_ty).ok_or(MirCode::Assertion)?;
            let actual_cast_type = contract_type(tcx, def_id, cast_type).ok();
            guard_locations.push((assertion_block_index, cast_statement));
            guards.push(cast_local);
            (
                cast_source,
                matches!(cast_kind, CastKind::IntToInt)
                    && actual_cast_type.as_ref() == Some(&expected_cast_type)
                    && contract_type(tcx, def_id, body.local_decls[cast_local].ty)
                        .ok()
                        .as_ref()
                        == Some(&expected_cast_type)
                    && no_intervening_assignment,
                cast_value_moved && matches!(cast_source, Operand::Copy(_)),
            )
        } else {
            (
                predicate_count,
                false,
                matches!(predicate_count, Operand::Copy(_)),
            )
        };

        let operation_site = shift_operation_site(body, *target)?;
        let operation_lhs_ty = operand_contract_type(tcx, def_id, body, operation_site.lhs)?;
        let operation_rhs_ty = operand_contract_type(tcx, def_id, body, operation_site.rhs)?;
        let mut vector = ShiftPatternVector::pinned(operation, signed_rhs);
        vector.value_operation = operation_site.operation;
        vector.operands_match = same_value_operand(message_lhs, operation_site.lhs)
            && same_value_operand(message_rhs, operation_site.rhs)
            && same_value_operand(original_guard_count, message_rhs);
        vector.operand_modes_match = checked_binary_operand_mode(message_lhs, operation_site.lhs)
            && checked_binary_operand_mode(message_rhs, operation_site.rhs)
            && predicate_mode_matches;
        vector.lhs_type_matches = supported_integer(&lhs_ty) && operation_lhs_ty == lhs_ty;
        vector.rhs_type_matches = supported_integer(&rhs_ty) && operation_rhs_ty == rhs_ty;
        vector.cast_matches = cast_matches;
        vector.predicate_matches = body.local_decls[predicate_guard].ty.is_bool();
        vector.threshold_matches = integer_width_constant(
            tcx,
            def_id,
            threshold,
            unsigned_equivalent(&rhs_ty)
                .as_ref()
                .ok_or(MirCode::Assertion)?,
            lhs_ty
                .as_bit_vector()
                .map(|(width, _)| u128::from(width))
                .ok_or(MirCode::Assertion)?,
        );
        vector.message_matches = operation_site.operation == operation;
        vector.expected_true = *expected;
        vector.condition_moved = condition_moved;
        vector.unwind_unreachable = matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable);
        vector.continuation_matches = operation_site.destination.projection.is_empty();
        validate_shift_pattern(&vector)?;

        for location in guard_locations {
            if !self.shift_guards.insert(location) {
                return Err(MirCode::Assertion);
            }
        }
        if self
            .assertions
            .insert(assertion_block_index, target.index())
            .is_some()
        {
            return Err(MirCode::Assertion);
        }
        if self
            .shifts
            .insert(
                (operation_site.block.index(), operation_site.statement),
                PlannedShift {
                    operation,
                    lhs_ty,
                    rhs_ty,
                    assertion_block: assertion_block_index,
                    guards,
                    vector,
                },
            )
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
        for ((block_index, statement_index), div_rem) in &mut self.div_rems {
            let mut assertion_blocks = vec![div_rem.zero_block];
            if let Some(overflow_block) = div_rem.overflow_block {
                assertion_blocks.push(overflow_block);
            }
            div_rem.vector.assertion_uses = assertion_blocks
                .iter()
                .filter(|block| self.assertions.contains_key(block))
                .count();
            let unique_guards = div_rem.guards.iter().copied().collect::<BTreeSet<_>>();
            div_rem.vector.guard_uses_match = unique_guards.len() == div_rem.guards.len()
                && div_rem.guards.iter().all(|guard| {
                    plain_local_uses(body, order, *guard) == 1
                        && statement_destination_count(body, order, *guard) == 1
                });
            let first_target = div_rem.overflow_block.unwrap_or(*block_index);
            div_rem.vector.continuation_matches &= predecessors[first_target] == 1
                && predecessors[*block_index] == 1
                && matches!(
                    body.basic_blocks[BasicBlock::new(*block_index)].statements[*statement_index]
                        .kind,
                    StatementKind::Assign(_)
                );
            validate_div_rem_pattern(&div_rem.vector)?;
        }
        for ((block_index, statement_index), shift) in &mut self.shifts {
            shift.vector.assertion_uses =
                usize::from(self.assertions.contains_key(&shift.assertion_block));
            let unique_guards = shift.guards.iter().copied().collect::<BTreeSet<_>>();
            shift.vector.guard_uses_match = unique_guards.len() == shift.guards.len()
                && shift.guards.iter().all(|guard| {
                    plain_local_uses(body, order, *guard) == 1
                        && statement_destination_count(body, order, *guard) == 1
                });
            shift.vector.continuation_matches &= predecessors[*block_index] == 1
                && matches!(
                    body.basic_blocks[BasicBlock::new(*block_index)].statements[*statement_index]
                        .kind,
                    StatementKind::Assign(_)
                );
            validate_shift_pattern(&shift.vector)?;
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
                        .contains_key(&(*block_index, statement_index))
                    || self
                        .div_rem_guards
                        .contains(&(*block_index, statement_index))
                    || self.div_rems.contains_key(&(*block_index, statement_index))
                    || self.shift_guards.contains(&(*block_index, statement_index))
                    || self.shifts.contains_key(&(*block_index, statement_index));
                if matches!(
                    assignment.1,
                    Rvalue::BinaryOp(
                        BinOp::AddWithOverflow | BinOp::SubWithOverflow | BinOp::MulWithOverflow,
                        _
                    ) | Rvalue::BinaryOp(BinOp::Div | BinOp::Rem, _)
                        | Rvalue::BinaryOp(BinOp::Shl | BinOp::Shr, _)
                        | Rvalue::UnaryOp(UnOp::Neg, _)
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

    pub(super) fn is_div_rem_guard(&self, block: usize, statement: usize) -> bool {
        self.div_rem_guards.contains(&(block, statement))
    }

    pub(super) fn div_rem(
        &self,
        block: usize,
        statement: usize,
    ) -> Option<(DivRemOperation, &ContractType)> {
        self.div_rems
            .get(&(block, statement))
            .map(|planned| (planned.operation, &planned.ty))
    }

    pub(super) fn is_shift_guard(&self, block: usize, statement: usize) -> bool {
        self.shift_guards.contains(&(block, statement))
    }

    pub(super) fn shift(
        &self,
        block: usize,
        statement: usize,
    ) -> Option<(ShiftOperation, &ContractType, &ContractType)> {
        self.shifts
            .get(&(block, statement))
            .map(|planned| (planned.operation, &planned.lhs_ty, &planned.rhs_ty))
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

fn shift_operation_site<'a, 'tcx>(
    body: &'a Body<'tcx>,
    block: BasicBlock,
) -> Result<ShiftOperationSite<'a, 'tcx>, MirCode> {
    let (statement, first_assignment) = body.basic_blocks[block]
        .statements
        .iter()
        .enumerate()
        .find(|(_, statement)| matches!(statement.kind, StatementKind::Assign(_)))
        .ok_or(MirCode::Assertion)?;
    let StatementKind::Assign(assignment) = &first_assignment.kind else {
        unreachable!("selected assignment")
    };
    let Rvalue::BinaryOp(operation, operands) = &assignment.1 else {
        return Err(MirCode::Assertion);
    };
    let operation = primitive_shift(*operation).ok_or(MirCode::Assertion)?;
    Ok(ShiftOperationSite {
        block,
        statement,
        destination: assignment.0,
        operation,
        lhs: &operands.0,
        rhs: &operands.1,
    })
}

fn div_rem_operation_site<'a, 'tcx>(
    body: &'a Body<'tcx>,
    block: BasicBlock,
) -> Result<DivRemOperationSite<'a, 'tcx>, MirCode> {
    let (statement, first_assignment) = body.basic_blocks[block]
        .statements
        .iter()
        .enumerate()
        .find(|(_, statement)| matches!(statement.kind, StatementKind::Assign(_)))
        .ok_or(MirCode::Assertion)?;
    let StatementKind::Assign(assignment) = &first_assignment.kind else {
        unreachable!("selected assignment")
    };
    let Rvalue::BinaryOp(operation, operands) = &assignment.1 else {
        return Err(MirCode::Assertion);
    };
    let operation = primitive_div_rem(*operation).ok_or(MirCode::Assertion)?;
    Ok(DivRemOperationSite {
        block,
        statement,
        destination: assignment.0,
        operation,
        lhs: &operands.0,
        rhs: &operands.1,
    })
}

fn signed_div_rem_site<'a, 'tcx>(
    body: &'a Body<'tcx>,
    block: BasicBlock,
    expected_operation: DivRemOperation,
) -> Result<SignedDivRemSite<'a, 'tcx>, MirCode> {
    let data = &body.basic_blocks[block];
    let assignments = data
        .statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement.kind, StatementKind::Assign(_)).then_some(index)
        })
        .collect::<Vec<_>>();
    let [minus_one_statement, minimum_statement, and_statement] = assignments.as_slice() else {
        return Err(MirCode::Assertion);
    };
    let StatementKind::Assign(minus_one_assignment) = &data.statements[*minus_one_statement].kind
    else {
        unreachable!("selected assignment")
    };
    let StatementKind::Assign(minimum_assignment) = &data.statements[*minimum_statement].kind
    else {
        unreachable!("selected assignment")
    };
    let StatementKind::Assign(and_assignment) = &data.statements[*and_statement].kind else {
        unreachable!("selected assignment")
    };
    let Rvalue::BinaryOp(BinOp::Eq, minus_one_operands) = &minus_one_assignment.1 else {
        return Err(MirCode::Assertion);
    };
    let Rvalue::BinaryOp(BinOp::Eq, minimum_operands) = &minimum_assignment.1 else {
        return Err(MirCode::Assertion);
    };
    let Rvalue::BinaryOp(BinOp::BitAnd, and_operands) = &and_assignment.1 else {
        return Err(MirCode::Assertion);
    };
    let TerminatorKind::Assert {
        cond,
        expected,
        msg,
        target,
        unwind,
    } = &data.terminator().kind
    else {
        return Err(MirCode::Assertion);
    };
    let AssertKind::Overflow(message_operation, message_lhs, message_rhs) = &**msg else {
        return Err(MirCode::Assertion);
    };
    let operation = message_div_rem(*message_operation).ok_or(MirCode::Assertion)?;
    let (condition_guard, condition_moved) = plain_operand_local(cond).ok_or(MirCode::Assertion)?;
    let minus_one_guard = minus_one_assignment.0.local;
    let minimum_guard = minimum_assignment.0.local;
    let and_guard = and_assignment.0.local;
    let and_lhs = plain_operand_local(&and_operands.0).ok_or(MirCode::Assertion)?;
    let and_rhs = plain_operand_local(&and_operands.1).ok_or(MirCode::Assertion)?;
    let destinations_plain = minus_one_assignment.0.projection.is_empty()
        && minimum_assignment.0.projection.is_empty()
        && and_assignment.0.projection.is_empty();
    let guards_are_bool = [minus_one_guard, minimum_guard, and_guard]
        .into_iter()
        .all(|guard| body.local_decls[guard].ty.is_bool());
    if operation != expected_operation
        || !destinations_plain
        || !guards_are_bool
        || and_lhs.0 != minus_one_guard
        || and_rhs.0 != minimum_guard
        || condition_guard != and_guard
    {
        return Err(MirCode::Assertion);
    }
    Ok(SignedDivRemSite {
        block,
        target: *target,
        statements: [*minus_one_statement, *minimum_statement, *and_statement],
        guards: [minus_one_guard, minimum_guard, and_guard],
        rhs_minus_one: &minus_one_operands.0,
        lhs_minimum: &minimum_operands.0,
        message_lhs,
        message_rhs,
        operation,
        expected_false: !*expected,
        conditions_moved: condition_moved && and_lhs.1 && and_rhs.1,
        unwind_unreachable: matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable),
    })
}

fn signed_guard_constant<'a, 'tcx>(
    body: &'a Body<'tcx>,
    block: BasicBlock,
    statement: usize,
) -> Result<&'a Operand<'tcx>, MirCode> {
    let StatementKind::Assign(assignment) = &body.basic_blocks[block].statements[statement].kind
    else {
        return Err(MirCode::Assertion);
    };
    let Rvalue::BinaryOp(BinOp::Eq, operands) = &assignment.1 else {
        return Err(MirCode::Assertion);
    };
    Ok(&operands.1)
}

fn signed_guard_order(body: &Body<'_>, site: &SignedDivRemSite<'_, '_>) -> bool {
    site.statements[0] < site.statements[1]
        && site.statements[1] < site.statements[2]
        && site
            .guards
            .into_iter()
            .all(|guard| body.local_decls[guard].ty.is_bool())
}

#[derive(Clone, Copy)]
enum IntegerBoundary {
    Zero,
    NegativeOne,
    Minimum,
}

fn integer_constant<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    operand: &Operand<'tcx>,
    ty: &ContractType,
    boundary: IntegerBoundary,
) -> bool {
    let Operand::Constant(constant) = operand else {
        return false;
    };
    let ContractType::BitVector { width, signed } = ty else {
        return false;
    };
    if !constant_type_matches(tcx, def_id, constant.const_.ty(), ty) {
        return false;
    }
    let expected = match boundary {
        IntegerBoundary::Zero => 0,
        IntegerBoundary::NegativeOne if *signed => (1_u128 << *width) - 1,
        IntegerBoundary::Minimum if *signed => 1_u128 << (*width - 1),
        IntegerBoundary::NegativeOne | IntegerBoundary::Minimum => return false,
    };
    constant
        .const_
        .try_eval_bits(tcx, ty::TypingEnv::post_analysis(tcx, def_id))
        == Some(expected)
}

fn integer_width_constant<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    operand: &Operand<'tcx>,
    ty: &ContractType,
    expected: u128,
) -> bool {
    let Operand::Constant(constant) = operand else {
        return false;
    };
    constant_type_matches(tcx, def_id, constant.const_.ty(), ty)
        && constant
            .const_
            .try_eval_bits(tcx, ty::TypingEnv::post_analysis(tcx, def_id))
            == Some(expected)
}

fn unsigned_equivalent(ty: &ContractType) -> Option<ContractType> {
    let ContractType::BitVector { width, .. } = ty else {
        return None;
    };
    Some(ContractType::BitVector {
        width: *width,
        signed: false,
    })
}

fn constant_type_matches<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    ty: ty::Ty<'tcx>,
    expected: &ContractType,
) -> bool {
    contract_type(tcx, def_id, ty).ok().as_ref() == Some(expected)
}

fn guard_operand_mode(operand: &Operand<'_>) -> bool {
    matches!(operand, Operand::Copy(_) | Operand::Constant(_))
}

fn operation_operand_mode(operand: &Operand<'_>) -> bool {
    matches!(operand, Operand::Move(_) | Operand::Constant(_))
}

fn primitive_div_rem(operation: BinOp) -> Option<DivRemOperation> {
    match operation {
        BinOp::Div => Some(DivRemOperation::Div),
        BinOp::Rem => Some(DivRemOperation::Rem),
        _ => None,
    }
}

fn primitive_shift(operation: BinOp) -> Option<ShiftOperation> {
    match operation {
        BinOp::Shl => Some(ShiftOperation::Shl),
        BinOp::Shr => Some(ShiftOperation::Shr),
        _ => None,
    }
}

fn message_div_rem(operation: BinOp) -> Option<DivRemOperation> {
    primitive_div_rem(operation)
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
