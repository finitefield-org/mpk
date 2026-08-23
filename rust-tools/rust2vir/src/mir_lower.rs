use super::const_lower::{self, HirConstant};
use super::hir_check::HirFunction;
use super::mir_aggregate::{
    array_operands, struct_aggregate, validate_array_aggregate_pattern,
    validate_struct_aggregate_pattern, ArrayAggregatePatternVector, StructAggregatePatternVector,
};
use super::mir_arithmetic::{ArithmeticOperation, ArithmeticPlan, DivRemOperation, ShiftOperation};
use super::mir_call::{
    validate_function_contract_context, validate_function_mir_signature, CallContext, CallPlan,
    PlannedCall,
};
use super::mir_projection::{direct_field_projection, FieldProjection, ProjectionPlan};
use super::type_lower::{contract_type, struct_declarations, vir_type, HirStructDecl};
use rust2vir_internal::call_closure::canonical_callee_first_order;
use rust2vir_internal::contract::{ContractSet, ContractType, NormalizedContract};
use rust2vir_internal::driver_protocol::DriverRequest;
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceRangeError};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::limits::RustLimitId;
use rust2vir_internal::mir_access::MIR_PROFILE_ID;
use rust2vir_internal::sha256::{hex, Sha256};
use rust2vir_internal::source_map::{raw_source_map, SourceMapEntry, SourceOrigin, VirReference};
use rust2vir_internal::stable_id::{block_names, DenseIds};
use rustc_index::Idx;
use rustc_middle::mir::{
    BasicBlock, BinOp, Body, Local, Operand, Place, Rvalue, StatementKind, TerminatorKind, UnOp,
    VarDebugInfoContents,
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::LocalDefId;
use rustc_span::{FileName, Span};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

const VIR_HASH_DOMAIN: &[u8] = b"MPK-VIR-0.1";
const MIR_BLOCKS_FUNCTION_MAX: usize = RustLimitId::MirBlocksFunction.maximum() as usize;
const MIR_BLOCKS_CLOSURE_MAX: usize = RustLimitId::MirBlocksClosure.maximum() as usize;
const MIR_STATEMENTS_FUNCTION_MAX: usize = RustLimitId::MirStatementsFunction.maximum() as usize;
const MIR_STATEMENTS_CLOSURE_MAX: usize = RustLimitId::MirStatementsClosure.maximum() as usize;
const VIR_PARAMETERS_MAX: usize = 256;
const VIR_LOCALS_MAX: usize = 65_536;
const VIR_BLOCK_PARAMETERS_MAX: usize = 4_096;
const VIR_INSTRUCTIONS_FUNCTION_MAX: usize = 100_000;
const VIR_INSTRUCTIONS_CLOSURE_MAX: usize = 250_000;
const VIR_CANONICAL_BYTES_MAX: usize = RustLimitId::VirJcs.maximum() as usize;
const SOURCE_MAP_CANONICAL_BYTES_MAX: usize = RustLimitId::SourceMapJcs.maximum() as usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MirCode {
    Statement,
    Rvalue,
    Operand,
    Place,
    Projection,
    Terminator,
    Assertion,
    CheckedPattern,
    Call,
    Move,
    Cleanup,
    SemanticsType,
    BlockLimit,
    StatementLimit,
    IrLimit,
    SourceMapExternal,
    SourceMapRange,
}

#[allow(dead_code)]
impl MirCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Statement => "RUST_MIR_STATEMENT",
            Self::Rvalue => "RUST_MIR_RVALUE",
            Self::Operand => "RUST_MIR_OPERAND",
            Self::Place => "RUST_MIR_PLACE",
            Self::Projection => "RUST_MIR_PROJECTION",
            Self::Terminator => "RUST_MIR_TERMINATOR",
            Self::Assertion => "RUST_MIR_ASSERTION",
            Self::CheckedPattern => "RUST_MIR_CHECKED_PATTERN",
            Self::Call => "RUST_MIR_CALL",
            Self::Move => "RUST_MIR_MOVE",
            Self::Cleanup => "RUST_MIR_CLEANUP",
            Self::SemanticsType => "RUST_SEMANTICS_TYPE",
            Self::BlockLimit => "RUST_LIMIT_MIR_BLOCKS",
            Self::StatementLimit => "RUST_LIMIT_MIR_STATEMENTS",
            Self::IrLimit => "RUST_LIMIT_IR",
            Self::SourceMapExternal => "RUST_FRONTEND_SOURCE_MAP_EXTERNAL",
            Self::SourceMapRange => "RUST_FRONTEND_SOURCE_MAP_RANGE",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Statement => "reachable MIR statement is outside the basic lowering dialect",
            Self::Rvalue => "reachable MIR rvalue is outside the basic lowering dialect",
            Self::Operand => "reachable MIR operand is outside the basic lowering dialect",
            Self::Place => "reachable MIR place is outside the basic lowering dialect",
            Self::Projection => "MIR place projection is not implemented by the basic lowerer",
            Self::Terminator => "reachable MIR terminator is outside the basic lowering dialect",
            Self::Assertion => {
                "MIR assertion does not match a supported pinned checked-operation pattern"
            }
            Self::CheckedPattern => {
                "MIR checked operation does not match the pinned consumption pattern"
            }
            Self::Call => "MIR call does not match the contracted direct-call pattern",
            Self::Move => "projected or dropping MIR move is not permitted",
            Self::Cleanup => "MIR cleanup, drop, or unwind flow is not permitted",
            Self::SemanticsType => {
                "lowered semantic type or contract binding does not match recomputation"
            }
            Self::BlockLimit => "reachable MIR block count exceeds the deterministic limit",
            Self::StatementLimit => "reachable MIR statement count exceeds the deterministic limit",
            Self::IrLimit => "lowered VIR exceeds the deterministic structural limit",
            Self::SourceMapExternal => "required MIR source origin is not a captured source span",
            Self::SourceMapRange => "required MIR source origin has an invalid UTF-8 byte range",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MirError {
    pub code: MirCode,
    pub function_id: String,
}

impl MirError {
    fn new(code: MirCode, function_id: &str) -> Self {
        Self {
            code,
            function_id: function_id.to_owned(),
        }
    }

    #[allow(dead_code)]
    pub fn is_frontend_error(&self) -> bool {
        matches!(
            self.code,
            MirCode::SourceMapExternal | MirCode::SourceMapRange
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MirLowering {
    pub raw_lowering: JsonValue,
    pub raw_source_map: JsonValue,
}

#[derive(Clone)]
pub(super) struct LoweredFunction {
    function_id: String,
    value: JsonValue,
    source_map: Vec<SourceMapEntry>,
    reachable_blocks: usize,
    reachable_statements: usize,
    instructions: usize,
}

#[derive(Default)]
pub(super) struct MirClosureBudget {
    blocks: usize,
    statements: usize,
    instructions: usize,
}

struct MirFunctionBudget {
    blocks: usize,
    statements: usize,
    maximum_blocks: usize,
    maximum_statements: usize,
}

struct InstructionBudget {
    next: usize,
    maximum: usize,
}

impl InstructionBudget {
    fn new(closure_remaining: usize) -> Self {
        Self {
            next: 0,
            maximum: VIR_INSTRUCTIONS_FUNCTION_MAX.min(closure_remaining),
        }
    }

    fn reserve(&mut self, function_id: &str) -> Result<usize, MirError> {
        if self.next >= self.maximum {
            return Err(MirError::new(MirCode::IrLimit, function_id));
        }
        let index = self.next;
        self.next += 1;
        Ok(index)
    }

    fn last_index(&self) -> usize {
        self.next
            .checked_sub(1)
            .expect("an instruction was reserved before it was emitted")
    }

    fn count(&self) -> usize {
        self.next
    }
}

impl MirFunctionBudget {
    fn new(closure_blocks_remaining: usize, closure_statements_remaining: usize) -> Self {
        Self {
            blocks: 0,
            statements: 0,
            maximum_blocks: MIR_BLOCKS_FUNCTION_MAX.min(closure_blocks_remaining),
            maximum_statements: MIR_STATEMENTS_FUNCTION_MAX.min(closure_statements_remaining),
        }
    }

    fn observe_block(&mut self, statements: usize, function_id: &str) -> Result<(), MirError> {
        let blocks = self
            .blocks
            .checked_add(1)
            .ok_or_else(|| MirError::new(MirCode::BlockLimit, function_id))?;
        if blocks > self.maximum_blocks {
            return Err(MirError::new(MirCode::BlockLimit, function_id));
        }
        let statements = self
            .statements
            .checked_add(statements)
            .ok_or_else(|| MirError::new(MirCode::StatementLimit, function_id))?;
        if statements > self.maximum_statements {
            return Err(MirError::new(MirCode::StatementLimit, function_id));
        }
        self.blocks = blocks;
        self.statements = statements;
        Ok(())
    }
}

impl MirClosureBudget {
    fn mir_remaining(&self, function_id: &str) -> Result<(usize, usize, usize), MirError> {
        let blocks = MIR_BLOCKS_CLOSURE_MAX
            .checked_sub(self.blocks)
            .ok_or_else(|| MirError::new(MirCode::BlockLimit, function_id))?;
        let statements = MIR_STATEMENTS_CLOSURE_MAX
            .checked_sub(self.statements)
            .ok_or_else(|| MirError::new(MirCode::StatementLimit, function_id))?;
        let instructions = VIR_INSTRUCTIONS_CLOSURE_MAX
            .checked_sub(self.instructions)
            .ok_or_else(|| MirError::new(MirCode::IrLimit, function_id))?;
        Ok((blocks, statements, instructions))
    }

    pub(super) fn observe(&mut self, item: &LoweredFunction) -> Result<(), MirError> {
        let blocks = self
            .blocks
            .checked_add(item.reachable_blocks)
            .ok_or_else(|| MirError::new(MirCode::BlockLimit, &item.function_id))?;
        let statements = self
            .statements
            .checked_add(item.reachable_statements)
            .ok_or_else(|| MirError::new(MirCode::StatementLimit, &item.function_id))?;
        let instructions = self
            .instructions
            .checked_add(item.instructions)
            .ok_or_else(|| MirError::new(MirCode::IrLimit, &item.function_id))?;
        if blocks > MIR_BLOCKS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::BlockLimit, &item.function_id));
        }
        if statements > MIR_STATEMENTS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::StatementLimit, &item.function_id));
        }
        if instructions > VIR_INSTRUCTIONS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::IrLimit, &item.function_id));
        }
        self.blocks = blocks;
        self.statements = statements;
        self.instructions = instructions;
        Ok(())
    }
}

pub(super) struct ModuleContext<'a> {
    call_closure: &'a [HirFunction],
    structs: &'a [HirStructDecl],
    contracts: &'a ContractSet,
    request: &'a DriverRequest,
    loader: &'a SnapshotFileLoader,
}

impl<'a> ModuleContext<'a> {
    pub(super) fn new(
        call_closure: &'a [HirFunction],
        structs: &'a [HirStructDecl],
        contracts: &'a ContractSet,
        request: &'a DriverRequest,
        loader: &'a SnapshotFileLoader,
    ) -> Self {
        Self {
            call_closure,
            structs,
            contracts,
            request,
            loader,
        }
    }
}

#[derive(Clone, Copy)]
enum Flow {
    Goto(usize),
    Call(usize),
    Branch {
        false_block: usize,
        true_block: usize,
    },
    Return,
}

type Reachability<'tcx> = (
    Vec<usize>,
    Vec<Option<Flow>>,
    ArithmeticPlan,
    ProjectionPlan,
    CallPlan<'tcx>,
    usize,
);

impl Flow {
    fn successors(self) -> Vec<usize> {
        match self {
            Self::Goto(target) | Self::Call(target) => vec![target],
            Self::Branch {
                false_block,
                true_block,
            } if false_block == true_block => vec![false_block],
            Self::Branch {
                false_block,
                true_block,
            } => vec![false_block, true_block],
            Self::Return => Vec::new(),
        }
    }
}

#[derive(Clone)]
struct Value {
    json: JsonValue,
    ty: ContractType,
    span: Span,
}

#[derive(Clone, Copy)]
enum LocalKind {
    Result,
    Argument(usize),
    User(usize),
    Temporary,
}

pub(super) fn finish_module(
    request: &DriverRequest,
    mut lowered: Vec<LoweredFunction>,
    structs: &[HirStructDecl],
    constants: &[HirConstant],
) -> Result<MirLowering, MirError> {
    let mut budget = MirClosureBudget::default();
    for item in &lowered {
        budget.observe(item)?;
    }
    let call_graph = lowered
        .iter()
        .map(|item| {
            Ok((
                item.function_id.clone(),
                emitted_callees(&item.value, &item.function_id)?,
            ))
        })
        .collect::<Result<Vec<_>, MirError>>()?;
    let function_order = canonical_callee_first_order(call_graph)
        .map_err(|_| MirError::new(MirCode::Call, request.selection().2))?;
    let mut functions_by_id = BTreeMap::new();
    for item in lowered {
        let function_id = item.function_id.clone();
        if functions_by_id.insert(function_id, item).is_some() {
            return Err(MirError::new(MirCode::Call, request.selection().2));
        }
    }
    lowered = function_order
        .into_iter()
        .map(|function_id| {
            functions_by_id
                .remove(&function_id)
                .ok_or_else(|| MirError::new(MirCode::Call, request.selection().2))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !functions_by_id.is_empty() {
        return Err(MirError::new(MirCode::Call, request.selection().2));
    }

    let unit_id = request.selection().1;
    let semantic_parameters = JsonValue::Object(BTreeMap::from([
        (
            "target_id".to_owned(),
            JsonValue::String(request.target().to_owned()),
        ),
        (
            "pointer_width".to_owned(),
            JsonValue::Number(request.pointer_width().to_string()),
        ),
        (
            "overflow_mode".to_owned(),
            JsonValue::String("checked".to_owned()),
        ),
        (
            "panic_mode".to_owned(),
            JsonValue::String("abort".to_owned()),
        ),
    ]));
    let mut functions_json = Vec::with_capacity(lowered.len());
    let mut entries = Vec::new();
    for item in lowered {
        functions_json.push(item.value);
        entries.extend(item.source_map);
    }
    let const_decls = const_lower::declarations(constants)
        .map_err(|_| MirError::new(MirCode::Rvalue, request.selection().2))?;
    let type_decls = struct_declarations(structs);
    let mut vir = JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string("mpk.vir.v0")),
        ("source_language".to_owned(), string("rust")),
        ("semantic_profile".to_owned(), string("mpk.rust.checked.v0")),
        ("semantic_parameters".to_owned(), semantic_parameters),
        (
            "units".to_owned(),
            JsonValue::Array(vec![JsonValue::Object(BTreeMap::from([
                ("id".to_owned(), string(unit_id)),
                ("name".to_owned(), string(request.selection().0)),
                ("type_decls".to_owned(), JsonValue::Array(type_decls)),
                ("const_decls".to_owned(), JsonValue::Array(const_decls)),
                ("functions".to_owned(), JsonValue::Array(functions_json)),
            ]))]),
        ),
    ]));
    let vir_preimage = json::canonical_bounded(&vir, VIR_CANONICAL_BYTES_MAX)
        .map_err(|_| MirError::new(MirCode::IrLimit, request.selection().2))?;
    let vir_hash = domain_hash(VIR_HASH_DOMAIN, &vir_preimage);
    vir.as_object_mut()
        .expect("constructed VIR object")
        .insert("vir_hash".to_owned(), string(&vir_hash));
    json::canonical_size(&vir, VIR_CANONICAL_BYTES_MAX)
        .map_err(|_| MirError::new(MirCode::IrLimit, request.selection().2))?;

    let source_map = raw_source_map(&vir_hash, entries);
    json::canonical_size(&source_map, SOURCE_MAP_CANONICAL_BYTES_MAX)
        .map_err(|_| MirError::new(MirCode::IrLimit, request.selection().2))?;
    Ok(MirLowering {
        raw_lowering: JsonValue::Object(BTreeMap::from([
            ("schema".to_owned(), string("mpk.rust.driver.lowering.v0")),
            ("mir_profile_id".to_owned(), string(MIR_PROFILE_ID)),
            ("vir".to_owned(), vir),
        ])),
        raw_source_map: source_map,
    })
}

pub(super) fn lower_function<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    function: &HirFunction,
    contract: &NormalizedContract,
    context: &ModuleContext<'_>,
    closure_budget: &MirClosureBudget,
) -> Result<LoweredFunction, MirError> {
    let function_id = function.function_id.as_str();
    let (closure_blocks_remaining, closure_statements_remaining, closure_instructions_remaining) =
        closure_budget.mir_remaining(function_id)?;
    validate_function_contract_context(function, contract, context.contracts, context.request)
        .map_err(|code| MirError::new(code, function_id))?;
    validate_function_mir_signature(tcx, def_id, body, function)
        .map_err(|code| MirError::new(code, function_id))?;
    if function.local_names.len() != function.local_types.len()
        || function.local_names.len() != function.local_spans.len()
        || function.parameter_types.len() > VIR_PARAMETERS_MAX
        || function.local_types.len() > VIR_LOCALS_MAX
        || !is_struct_value_type(&function.result_type)
        || function
            .parameter_types
            .iter()
            .any(|ty| !is_struct_value_type(ty))
        || function
            .local_types
            .iter()
            .any(|ty| !is_struct_value_type(ty))
    {
        return Err(MirError::new(MirCode::Rvalue, function_id));
    }
    let local_kinds = map_locals(body, function)?;
    let call_context = CallContext {
        caller: function,
        closure: context.call_closure,
        contracts: context.contracts,
        request: context.request,
    };
    let (order, flows, mut arithmetic, mut projection, calls, statement_count) = reachable_order(
        tcx,
        def_id,
        body,
        function,
        &call_context,
        function_id,
        closure_blocks_remaining,
        closure_statements_remaining,
    )?;
    arithmetic
        .finish(body, &order)
        .map_err(|code| MirError::new(code, function_id))?;
    projection
        .finish(body, &order)
        .map_err(|code| MirError::new(code, function_id))?;
    validate_storage(body, &order, &flows, &local_kinds, &calls, function_id)?;
    let (live_in, uses, definitions) = live_compiler_locals(
        tcx,
        def_id,
        body,
        &order,
        &flows,
        &local_kinds,
        &arithmetic,
        &projection,
        &calls,
        function_id,
    )?;
    let _ = (uses, definitions);
    if !live_in[0].is_empty() {
        return Err(MirError::new(MirCode::Operand, function_id));
    }

    let block_ids = block_names(&order);
    let incoming_origins =
        compiler_local_origins(body, &order, &flows, &local_kinds, &calls, function_id)?;
    let mut ids = DenseIds::default();
    let mut block_parameters = vec![BTreeMap::<usize, String>::new(); body.basic_blocks.len()];
    for block in order.iter().skip(1) {
        if live_in[*block].len() > VIR_BLOCK_PARAMETERS_MAX {
            return Err(MirError::new(MirCode::IrLimit, function_id));
        }
        for local in &live_in[*block] {
            block_parameters[*block].insert(*local, ids.block_parameter());
        }
    }

    let unit_id = function_id
        .split("::")
        .next()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    let mut source_entries = vec![SourceMapEntry {
        reference: VirReference::Function {
            unit_id: unit_id.to_owned(),
            function_id: function_id.to_owned(),
        },
        origin: source_origin(tcx, context.loader, body.span, function_id)?,
    }];
    let mut blocks = Vec::with_capacity(order.len());
    let mut next_instruction = InstructionBudget::new(closure_instructions_remaining);
    let mut initialized_user_locals = BTreeSet::new();
    for (block_index, old_block) in order.iter().enumerate() {
        let block = &body.basic_blocks[BasicBlock::new(*old_block)];
        let mut environment = BTreeMap::<usize, Value>::new();
        let mut parameters = Vec::new();
        for (local, id) in &block_parameters[*old_block] {
            let ty = arithmetic
                .scalar_local_type(Local::new(*local))
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| {
                    mir_contract_type(
                        tcx,
                        def_id,
                        body.local_decls[Local::new(*local)].ty,
                        function_id,
                    )
                })?;
            environment.insert(
                *local,
                Value {
                    json: variable(id),
                    ty: ty.clone(),
                    span: *incoming_origins[*old_block]
                        .get(local)
                        .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?,
                },
            );
            parameters.push(binding(id, &ty));
        }
        let mut instructions = Vec::new();
        for (statement_index, statement) in block.statements.iter().enumerate() {
            match &statement.kind {
                StatementKind::Assign(assignment) => {
                    let (destination, rvalue) = &**assignment;
                    lower_assignment(
                        tcx,
                        def_id,
                        body,
                        destination,
                        rvalue,
                        statement.source_info.span,
                        *old_block,
                        statement_index,
                        &arithmetic,
                        &projection,
                        function,
                        context.structs,
                        &local_kinds,
                        &function.local_spans,
                        &mut initialized_user_locals,
                        &mut environment,
                        &mut ids,
                        &mut next_instruction,
                        block_index,
                        unit_id,
                        function_id,
                        context.loader,
                        &mut instructions,
                        &mut source_entries,
                    )?;
                }
                StatementKind::StorageLive(_) | StatementKind::StorageDead(_) => {}
                StatementKind::Nop => {
                    if !span_contains(body.span, statement.source_info.span) {
                        return Err(MirError::new(MirCode::SourceMapExternal, function_id));
                    }
                    let _ = source_origin(
                        tcx,
                        context.loader,
                        statement.source_info.span,
                        function_id,
                    )?;
                }
                _ => return Err(MirError::new(MirCode::Statement, function_id)),
            }
        }
        let terminator = block.terminator();
        let flow =
            flows[*old_block].ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?;
        let (terminator_json, fallback_terminator_span) = match flow {
            Flow::Goto(target) => {
                let fallback = live_in[target]
                    .iter()
                    .filter_map(|local| environment.get(local).map(|value| value.span))
                    .min_by_key(|span| span_key(*span))
                    .or_else(|| {
                        block
                            .statements
                            .iter()
                            .rev()
                            .map(|statement| statement.source_info.span)
                            .find(|span| span.lo() < span.hi())
                    });
                (
                    jump(
                        &block_ids[&target],
                        edge_arguments(
                            target,
                            &live_in,
                            &environment,
                            &block_parameters,
                            function_id,
                        )?,
                    ),
                    fallback,
                )
            }
            Flow::Call(target) => {
                let call = calls
                    .get(*old_block)
                    .ok_or_else(|| MirError::new(MirCode::Call, function_id))?;
                lower_static_call(
                    tcx,
                    def_id,
                    body,
                    call,
                    &arithmetic,
                    function,
                    &local_kinds,
                    &mut initialized_user_locals,
                    &mut environment,
                    &mut ids,
                    &mut next_instruction,
                    block_index,
                    unit_id,
                    function_id,
                    context.loader,
                    &mut instructions,
                    &mut source_entries,
                )?;
                (
                    jump(
                        &block_ids[&target],
                        edge_arguments(
                            target,
                            &live_in,
                            &environment,
                            &block_parameters,
                            function_id,
                        )?,
                    ),
                    Some(call.span),
                )
            }
            Flow::Branch {
                false_block,
                true_block,
            } => {
                let TerminatorKind::SwitchInt { discr, .. } = &terminator.kind else {
                    return Err(MirError::new(MirCode::Terminator, function_id));
                };
                let condition = lower_operand(
                    tcx,
                    def_id,
                    body,
                    discr,
                    &local_kinds,
                    &environment,
                    &arithmetic,
                    function_id,
                )?;
                let fallback = Some(condition.span);
                if condition.ty != ContractType::Bool {
                    return Err(MirError::new(MirCode::Terminator, function_id));
                }
                let terminator = if false_block == true_block {
                    jump(
                        &block_ids[&false_block],
                        edge_arguments(
                            false_block,
                            &live_in,
                            &environment,
                            &block_parameters,
                            function_id,
                        )?,
                    )
                } else {
                    branch(
                        condition.json,
                        &block_ids[&true_block],
                        edge_arguments(
                            true_block,
                            &live_in,
                            &environment,
                            &block_parameters,
                            function_id,
                        )?,
                        &block_ids[&false_block],
                        edge_arguments(
                            false_block,
                            &live_in,
                            &environment,
                            &block_parameters,
                            function_id,
                        )?,
                    )
                };
                (terminator, fallback)
            }
            Flow::Return => {
                let result = resolve_local(
                    tcx,
                    def_id,
                    body,
                    Local::new(0),
                    &local_kinds,
                    &environment,
                    &arithmetic,
                    function_id,
                )?;
                if result.ty != function.result_type {
                    return Err(MirError::new(MirCode::Operand, function_id));
                }
                (
                    JsonValue::Object(BTreeMap::from([
                        ("kind".to_owned(), string("Return")),
                        ("values".to_owned(), JsonValue::Array(vec![result.json])),
                    ])),
                    Some(result.span),
                )
            }
        };
        let raw_terminator_span = match flow {
            Flow::Return | Flow::Call(_) => fallback_terminator_span,
            _ if terminator.source_info.span.lo() < terminator.source_info.span.hi() => {
                Some(terminator.source_info.span)
            }
            _ => fallback_terminator_span,
        };
        let terminator_span = match flow {
            Flow::Branch { .. } => raw_terminator_span
                .and_then(|span| enclosing_control_flow_span(span, &function.control_flow_spans))
                .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?,
            Flow::Goto(_) => raw_terminator_span
                .and_then(|span| enclosing_control_flow_span(span, &function.control_flow_spans))
                .or(raw_terminator_span)
                .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?,
            Flow::Call(_) => raw_terminator_span
                .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?,
            Flow::Return => raw_terminator_span
                .and_then(|span| enclosing_span(span, &function.return_value_spans))
                .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?,
        };
        source_entries.push(SourceMapEntry {
            reference: VirReference::Terminator {
                unit_id: unit_id.to_owned(),
                function_id: function_id.to_owned(),
                block_index,
            },
            origin: source_origin(tcx, context.loader, terminator_span, function_id)?,
        });
        blocks.push(JsonValue::Object(BTreeMap::from([
            ("label".to_owned(), string(&block_ids[old_block])),
            ("parameters".to_owned(), JsonValue::Array(parameters)),
            ("instructions".to_owned(), JsonValue::Array(instructions)),
            ("terminator".to_owned(), terminator_json),
        ])));
    }

    let params = function
        .parameter_types
        .iter()
        .enumerate()
        .map(|(index, ty)| binding(&format!("arg{index}"), ty))
        .collect();
    let locals = function
        .local_types
        .iter()
        .enumerate()
        .map(|(index, ty)| binding(&format!("local{index}"), ty))
        .collect();
    let mut features = Vec::new();
    if projection.has_indexes()
        || is_array_type(&function.result_type)
        || function.parameter_types.iter().any(is_array_type)
        || function.local_types.iter().any(is_array_type)
    {
        features.push(string("array"));
    }
    if flows.iter().flatten().any(|flow| matches!(flow, Flow::Branch { false_block, true_block } if false_block != true_block)) {
        features.push(string("branch"));
    }
    if blocks_contain_instruction_kind(&blocks, &["CallStatic"]) {
        features.push(string("call_static"));
    }
    if blocks_contain_constant_reference(&blocks) {
        features.push(string("constant_decl"));
    }
    if !function.local_types.is_empty() || instructions_contain_copy(&blocks) {
        features.push(string("mutable_local"));
    }
    if is_struct_type(&function.result_type)
        || function.parameter_types.iter().any(is_struct_type)
        || function.local_types.iter().any(is_struct_type)
        || blocks_contain_instruction_kind(&blocks, &["Field", "MakeStruct"])
    {
        features.push(string("struct"));
    }
    let name = function_id
        .rsplit("::")
        .next()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    Ok(LoweredFunction {
        function_id: function_id.to_owned(),
        value: JsonValue::Object(BTreeMap::from([
            ("id".to_owned(), string(function_id)),
            ("unit_id".to_owned(), string(unit_id)),
            ("name".to_owned(), string(name)),
            ("params".to_owned(), JsonValue::Array(params)),
            (
                "results".to_owned(),
                JsonValue::Array(vec![binding("result0", &function.result_type)]),
            ),
            ("locals".to_owned(), JsonValue::Array(locals)),
            ("blocks".to_owned(), JsonValue::Array(blocks)),
            ("contracts".to_owned(), contract.value.clone()),
            ("features_used".to_owned(), JsonValue::Array(features)),
        ])),
        source_map: source_entries,
        reachable_blocks: order.len(),
        reachable_statements: statement_count,
        instructions: next_instruction.count(),
    })
}

fn map_locals(body: &Body<'_>, function: &HirFunction) -> Result<Vec<LocalKind>, MirError> {
    let function_id = function.function_id.as_str();
    if body.local_decls.len() < body.arg_count + 1 {
        return Err(MirError::new(MirCode::Place, function_id));
    }
    let mut kinds = vec![LocalKind::Temporary; body.local_decls.len()];
    kinds[0] = LocalKind::Result;
    for index in 0..body.arg_count {
        kinds[index + 1] = LocalKind::Argument(index);
    }
    let mut by_name = BTreeMap::<String, usize>::new();
    for debug in &body.var_debug_info {
        if debug.argument_index.is_some() || debug.composite.is_some() {
            continue;
        }
        let VarDebugInfoContents::Place(place) = debug.value else {
            continue;
        };
        if !place.projection.is_empty() {
            continue;
        }
        let index = place.local.index();
        if index <= body.arg_count || index >= kinds.len() {
            continue;
        }
        if by_name
            .insert(debug.name.as_str().to_owned(), index)
            .is_some()
        {
            return Err(MirError::new(MirCode::Place, function_id));
        }
    }
    let mut claimed = BTreeSet::new();
    for (source_index, name) in function.local_names.iter().enumerate() {
        let local = *by_name
            .get(name)
            .ok_or_else(|| MirError::new(MirCode::Place, function_id))?;
        if !claimed.insert(local) {
            return Err(MirError::new(MirCode::Place, function_id));
        }
        kinds[local] = LocalKind::User(source_index);
    }
    Ok(kinds)
}

fn reachable_order<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    function: &HirFunction,
    call_context: &CallContext<'_>,
    function_id: &str,
    closure_blocks_remaining: usize,
    closure_statements_remaining: usize,
) -> Result<Reachability<'tcx>, MirError> {
    if body.basic_blocks.is_empty() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    let mut flows_by_block = BTreeMap::new();
    let mut arithmetic = ArithmeticPlan::default();
    let mut projection = ProjectionPlan::default();
    let mut calls = CallPlan::default();
    let mut budget = MirFunctionBudget::new(closure_blocks_remaining, closure_statements_remaining);
    budget.observe_block(
        body.basic_blocks[BasicBlock::new(0)].statements.len(),
        function_id,
    )?;
    let mut discovered = BTreeSet::from([0_usize]);
    let mut pending = VecDeque::from([0_usize]);
    let mut order = Vec::new();
    while let Some(index) = pending.pop_front() {
        let block = &body.basic_blocks[BasicBlock::new(index)];
        if block.is_cleanup {
            return Err(MirError::new(MirCode::Cleanup, function_id));
        }
        let flow = classify_flow(
            tcx,
            def_id,
            body,
            index,
            function,
            &mut arithmetic,
            &mut projection,
            &mut calls,
            call_context,
            function_id,
        )?;
        flows_by_block.insert(index, flow);
        order.push(index);
        for successor in flow.successors() {
            if successor >= body.basic_blocks.len() {
                return Err(MirError::new(MirCode::Terminator, function_id));
            }
            if !discovered.contains(&successor) {
                budget.observe_block(
                    body.basic_blocks[BasicBlock::new(successor)]
                        .statements
                        .len(),
                    function_id,
                )?;
                discovered.insert(successor);
                pending.push_back(successor);
            }
        }
    }
    let mut flows = vec![None; body.basic_blocks.len()];
    for (index, flow) in flows_by_block {
        flows[index] = Some(flow);
    }
    if topological_order(&order, &flows).is_none() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    Ok((
        order,
        flows,
        arithmetic,
        projection,
        calls,
        budget.statements,
    ))
}

#[cfg(test)]
mod limit_tests {
    use super::*;

    #[test]
    fn function_budget_accepts_exact_boundaries_without_retaining_excess() {
        let mut blocks = MirFunctionBudget::new(MIR_BLOCKS_FUNCTION_MAX, usize::MAX);
        blocks.blocks = MIR_BLOCKS_FUNCTION_MAX - 1;
        blocks.observe_block(0, "vector::f").unwrap();
        assert_eq!(blocks.blocks, MIR_BLOCKS_FUNCTION_MAX);
        assert_eq!(
            blocks.observe_block(0, "vector::f").unwrap_err().code,
            MirCode::BlockLimit
        );
        assert_eq!(blocks.blocks, MIR_BLOCKS_FUNCTION_MAX);

        let mut statements = MirFunctionBudget::new(usize::MAX, MIR_STATEMENTS_FUNCTION_MAX);
        statements.statements = MIR_STATEMENTS_FUNCTION_MAX;
        assert_eq!(
            statements.observe_block(1, "vector::f").unwrap_err().code,
            MirCode::StatementLimit
        );
        assert_eq!(statements.blocks, 0);
        assert_eq!(statements.statements, MIR_STATEMENTS_FUNCTION_MAX);
    }

    #[test]
    fn closure_remaining_budget_rejects_before_retaining_the_next_block() {
        let closure = MirClosureBudget {
            blocks: MIR_BLOCKS_CLOSURE_MAX,
            statements: MIR_STATEMENTS_CLOSURE_MAX,
            instructions: 0,
        };
        assert_eq!(
            closure.mir_remaining("vector::next"),
            Ok((0, 0, VIR_INSTRUCTIONS_CLOSURE_MAX))
        );

        let mut blocks = MirFunctionBudget::new(0, usize::MAX);
        assert_eq!(
            blocks.observe_block(0, "vector::next").unwrap_err().code,
            MirCode::BlockLimit
        );
        assert_eq!((blocks.blocks, blocks.statements), (0, 0));

        let mut statements = MirFunctionBudget::new(1, 0);
        assert_eq!(
            statements
                .observe_block(1, "vector::next")
                .unwrap_err()
                .code,
            MirCode::StatementLimit
        );
        assert_eq!((statements.blocks, statements.statements), (0, 0));
    }

    #[test]
    fn instruction_budget_rejects_before_allocating_the_next_instruction() {
        let mut budget = InstructionBudget::new(2);
        assert_eq!(budget.reserve("vector::f"), Ok(0));
        assert_eq!(budget.reserve("vector::f"), Ok(1));
        assert_eq!(budget.count(), 2);
        assert_eq!(
            budget.reserve("vector::f").unwrap_err().code,
            MirCode::IrLimit
        );
        assert_eq!(budget.count(), 2);

        let mut exhausted = InstructionBudget::new(0);
        assert_eq!(
            exhausted.reserve("vector::next").unwrap_err().code,
            MirCode::IrLimit
        );
        assert_eq!(exhausted.count(), 0);
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_flow<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    block_index: usize,
    function: &HirFunction,
    arithmetic: &mut ArithmeticPlan,
    projection: &mut ProjectionPlan,
    calls: &mut CallPlan<'tcx>,
    call_context: &CallContext<'_>,
    function_id: &str,
) -> Result<Flow, MirError> {
    let block = &body.basic_blocks[BasicBlock::new(block_index)];
    match &block.terminator().kind {
        TerminatorKind::Goto { target } => Ok(Flow::Goto(target.index())),
        TerminatorKind::SwitchInt { discr, targets } => {
            let Some((value, explicit, otherwise)) = targets.as_static_if() else {
                return Err(MirError::new(MirCode::Terminator, function_id));
            };
            if value != 0 {
                return Err(MirError::new(MirCode::Terminator, function_id));
            }
            if let Some(condition) =
                constant_bool_discriminant(tcx, def_id, discr, &block.statements, function_id)?
            {
                return Ok(Flow::Goto(
                    if condition { otherwise } else { explicit }.index(),
                ));
            }
            Ok(Flow::Branch {
                false_block: explicit.index(),
                true_block: otherwise.index(),
            })
        }
        TerminatorKind::Return => Ok(Flow::Return),
        TerminatorKind::Assert { msg, .. } => {
            let recognized = if matches!(&**msg, rustc_middle::mir::AssertKind::BoundsCheck { .. })
            {
                projection.recognize_assert(tcx, def_id, body, block_index, function)
            } else {
                arithmetic.recognize_assert(tcx, def_id, body, block_index, function)
            };
            recognized
                .map(Flow::Goto)
                .map_err(|code| MirError::new(code, function_id))
        }
        TerminatorKind::Call { .. } => calls
            .recognize(tcx, def_id, body, block_index, call_context)
            .map(Flow::Call)
            .map_err(|code| MirError::new(code, function_id)),
        TerminatorKind::TailCall { .. } => Err(MirError::new(MirCode::Call, function_id)),
        TerminatorKind::Drop { .. }
        | TerminatorKind::UnwindResume
        | TerminatorKind::UnwindTerminate(_)
        | TerminatorKind::CoroutineDrop => Err(MirError::new(MirCode::Cleanup, function_id)),
        _ => Err(MirError::new(MirCode::Terminator, function_id)),
    }
}

fn constant_bool_discriminant<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    discriminant: &Operand<'tcx>,
    statements: &[rustc_middle::mir::Statement<'tcx>],
    function_id: &str,
) -> Result<Option<bool>, MirError> {
    let constant = match discriminant {
        Operand::Constant(constant) => Some(&constant.const_),
        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => statements
            .iter()
            .rev()
            .find_map(|statement| match &statement.kind {
                StatementKind::Assign(assignment) if assignment.0.local == place.local => {
                    match &assignment.1 {
                        Rvalue::Use(Operand::Constant(constant))
                            if assignment.0.projection.is_empty() =>
                        {
                            Some(Some(&constant.const_))
                        }
                        _ => Some(None),
                    }
                }
                _ => None,
            })
            .flatten(),
        _ => None,
    };
    let Some(constant) = constant else {
        return Ok(None);
    };
    if !constant.ty().is_bool() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    let typing_env = ty::TypingEnv::post_analysis(tcx, def_id);
    constant
        .try_eval_bool(tcx, typing_env)
        .map(Some)
        .ok_or_else(|| MirError::new(MirCode::Operand, function_id))
}

fn topological_order(order: &[usize], flows: &[Option<Flow>]) -> Option<Vec<usize>> {
    let reachable = order.iter().copied().collect::<BTreeSet<_>>();
    let mut indegree = BTreeMap::new();
    for block in order {
        indegree.insert(*block, 0_usize);
    }
    for block in order {
        for successor in flows[*block]?.successors() {
            if reachable.contains(&successor) {
                *indegree.get_mut(&successor)? += 1;
            }
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(block, count)| (*count == 0).then_some(*block))
        .collect::<BTreeSet<_>>();
    let mut result = Vec::with_capacity(order.len());
    while let Some(block) = ready.pop_first() {
        result.push(block);
        for successor in flows[block]?.successors() {
            let count = indegree.get_mut(&successor)?;
            *count = count.checked_sub(1)?;
            if *count == 0 {
                ready.insert(successor);
            }
        }
    }
    (result.len() == order.len()).then_some(result)
}

type LocalSets = Vec<BTreeSet<usize>>;

fn compiler_local_origins<'tcx>(
    body: &Body<'tcx>,
    order: &[usize],
    flows: &[Option<Flow>],
    local_kinds: &[LocalKind],
    calls: &CallPlan<'tcx>,
    function_id: &str,
) -> Result<Vec<BTreeMap<usize, Span>>, MirError> {
    let topological = topological_order(order, flows)
        .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?;
    let mut predecessors = vec![Vec::new(); body.basic_blocks.len()];
    for block in order {
        for successor in flows[*block]
            .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?
            .successors()
        {
            predecessors[successor].push(*block);
        }
    }
    let mut incoming = vec![BTreeMap::new(); body.basic_blocks.len()];
    let mut outgoing = vec![BTreeMap::new(); body.basic_blocks.len()];
    for block_index in topological {
        let mut state = BTreeMap::new();
        for predecessor in &predecessors[block_index] {
            for (local, span) in &outgoing[*predecessor] {
                state
                    .entry(*local)
                    .and_modify(|current: &mut Span| {
                        if span_key(*span) < span_key(*current) {
                            *current = *span;
                        }
                    })
                    .or_insert(*span);
            }
        }
        incoming[block_index] = state.clone();
        for statement in &body.basic_blocks[BasicBlock::new(block_index)].statements {
            if let StatementKind::Assign(assignment) = &statement.kind {
                let (destination, _) = &**assignment;
                let local = destination.local.index();
                if is_modeled_compiler_local(body, local_kinds, local) {
                    state.insert(local, statement.source_info.span);
                }
            }
        }
        if let Some(call) = calls.get(block_index) {
            let local = call.destination.index();
            if is_modeled_compiler_local(body, local_kinds, local) {
                state.insert(local, call.span);
            }
        }
        outgoing[block_index] = state;
    }
    Ok(incoming)
}

fn span_key(span: Span) -> (u32, u32) {
    (span.lo().0, span.hi().0)
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.lo() < outer.hi()
        && inner.lo() < inner.hi()
        && outer.lo() <= inner.lo()
        && inner.hi() <= outer.hi()
}

fn enclosing_control_flow_span(span: Span, control_flow_spans: &[Span]) -> Option<Span> {
    enclosing_span(span, control_flow_spans)
}

fn enclosing_span(span: Span, candidates: &[Span]) -> Option<Span> {
    candidates
        .iter()
        .copied()
        .filter(|candidate| span_contains(*candidate, span))
        .min_by_key(|candidate| {
            (
                candidate.hi().0 - candidate.lo().0,
                candidate.lo().0,
                candidate.hi().0,
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn live_compiler_locals<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    order: &[usize],
    flows: &[Option<Flow>],
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    projection: &ProjectionPlan,
    calls: &CallPlan<'tcx>,
    function_id: &str,
) -> Result<(LocalSets, LocalSets, LocalSets), MirError> {
    let mut uses = vec![BTreeSet::new(); body.basic_blocks.len()];
    let mut definitions = vec![BTreeSet::new(); body.basic_blocks.len()];
    for block_index in order {
        let block = &body.basic_blocks[BasicBlock::new(*block_index)];
        for (statement_index, statement) in block.statements.iter().enumerate() {
            match &statement.kind {
                StatementKind::Assign(assignment) => {
                    let (place, rvalue) = &**assignment;
                    validate_destination(place, local_kinds, function_id)?;
                    let mut reads = Vec::new();
                    if projection.is_guard(*block_index, statement_index)
                        || arithmetic.is_negation_guard(*block_index, statement_index)
                        || arithmetic.is_div_rem_guard(*block_index, statement_index)
                        || arithmetic.is_shift_guard(*block_index, statement_index)
                    {
                        // The guard is represented by the attached VIR safety check.
                    } else if let Some(index) = projection.index(*block_index, statement_index) {
                        let Rvalue::Use(Operand::Copy(projected)) = rvalue else {
                            return Err(MirError::new(MirCode::Assertion, function_id));
                        };
                        if projected.local != index.base
                            || !matches!(
                                &projected.projection[..],
                                [rustc_middle::mir::ProjectionElem::Index(local)]
                                    if *local == index.index
                            )
                        {
                            return Err(MirError::new(MirCode::Assertion, function_id));
                        }
                        reads.extend([index.base, index.index]);
                    } else if let Some((_, ty)) = arithmetic.binary(*block_index, statement_index) {
                        validate_checked_binary_rvalue(
                            tcx,
                            def_id,
                            body,
                            rvalue,
                            ty,
                            local_kinds,
                            arithmetic,
                            function_id,
                            &mut reads,
                        )?;
                    } else if let Some(ty) = arithmetic.negation(*block_index, statement_index) {
                        validate_checked_negation_rvalue(
                            tcx,
                            def_id,
                            body,
                            rvalue,
                            ty,
                            local_kinds,
                            arithmetic,
                            function_id,
                            &mut reads,
                        )?;
                    } else if let Some((operation, ty)) =
                        arithmetic.div_rem(*block_index, statement_index)
                    {
                        validate_div_rem_rvalue(
                            tcx,
                            def_id,
                            body,
                            rvalue,
                            operation,
                            ty,
                            local_kinds,
                            arithmetic,
                            function_id,
                            &mut reads,
                        )?;
                    } else if let Some((operation, lhs_ty, rhs_ty)) =
                        arithmetic.shift(*block_index, statement_index)
                    {
                        validate_shift_rvalue(
                            tcx,
                            def_id,
                            body,
                            rvalue,
                            operation,
                            lhs_ty,
                            rhs_ty,
                            local_kinds,
                            arithmetic,
                            function_id,
                            &mut reads,
                        )?;
                    } else if is_erasable_unit_assignment(body, place, rvalue, local_kinds) {
                        collect_operand_locals(rvalue, &mut reads);
                    } else {
                        validate_rvalue(
                            tcx,
                            def_id,
                            body,
                            rvalue,
                            local_kinds,
                            arithmetic,
                            function_id,
                            &mut reads,
                        )?;
                    }
                    for local in reads {
                        let index = local.index();
                        if is_modeled_compiler_local(body, local_kinds, index)
                            && !definitions[*block_index].contains(&index)
                        {
                            uses[*block_index].insert(index);
                        }
                    }
                    let destination = place.local.index();
                    if is_modeled_compiler_local(body, local_kinds, destination) {
                        definitions[*block_index].insert(destination);
                    }
                }
                StatementKind::StorageLive(_) | StatementKind::StorageDead(_) => {}
                StatementKind::Nop => {}
                _ => return Err(MirError::new(MirCode::Statement, function_id)),
            }
        }
        if let Some(call) = calls.get(*block_index) {
            let destination = call.destination.index();
            match local_kinds.get(destination) {
                Some(LocalKind::Argument(_)) | None => {
                    return Err(MirError::new(MirCode::Place, function_id));
                }
                Some(_) => {}
            }
            let mut reads = Vec::new();
            for argument in &call.arguments {
                validate_operand(
                    tcx,
                    def_id,
                    body,
                    argument,
                    local_kinds,
                    arithmetic,
                    function_id,
                    &mut reads,
                )?;
            }
            for local in reads {
                let index = local.index();
                if is_modeled_compiler_local(body, local_kinds, index)
                    && !definitions[*block_index].contains(&index)
                {
                    uses[*block_index].insert(index);
                }
            }
            if is_modeled_compiler_local(body, local_kinds, destination) {
                definitions[*block_index].insert(destination);
            }
        } else if let TerminatorKind::SwitchInt { discr, .. } = &block.terminator().kind {
            let mut reads = Vec::new();
            validate_operand(
                tcx,
                def_id,
                body,
                discr,
                local_kinds,
                arithmetic,
                function_id,
                &mut reads,
            )?;
            for local in reads {
                let index = local.index();
                if is_modeled_compiler_local(body, local_kinds, index)
                    && !definitions[*block_index].contains(&index)
                {
                    uses[*block_index].insert(index);
                }
            }
        } else if matches!(block.terminator().kind, TerminatorKind::Return)
            && !definitions[*block_index].contains(&0)
        {
            uses[*block_index].insert(0);
        }
    }

    let topological = topological_order(order, flows)
        .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?;
    let mut live_in = vec![BTreeSet::new(); body.basic_blocks.len()];
    for block in topological.into_iter().rev() {
        let mut live_out = BTreeSet::new();
        for successor in flows[block]
            .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?
            .successors()
        {
            live_out.extend(live_in[successor].iter().copied());
        }
        live_out.retain(|local| !definitions[block].contains(local));
        live_out.extend(uses[block].iter().copied());
        live_in[block] = live_out;
    }
    Ok((live_in, uses, definitions))
}

fn validate_storage<'tcx>(
    body: &Body<'tcx>,
    order: &[usize],
    flows: &[Option<Flow>],
    local_kinds: &[LocalKind],
    calls: &CallPlan<'tcx>,
    function_id: &str,
) -> Result<(), MirError> {
    let mut has_marker = vec![false; body.local_decls.len()];
    for block_index in order {
        for statement in &body.basic_blocks[BasicBlock::new(*block_index)].statements {
            if let StatementKind::StorageLive(local) | StatementKind::StorageDead(local) =
                statement.kind
            {
                if local.index() >= has_marker.len() {
                    return Err(MirError::new(MirCode::Statement, function_id));
                }
                has_marker[local.index()] = true;
            }
        }
    }
    let initial = has_marker
        .iter()
        .enumerate()
        .filter_map(|(local, marked)| (!marked).then_some(local))
        .collect::<BTreeSet<_>>();
    let topological = topological_order(order, flows)
        .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?;
    let mut predecessors = vec![Vec::new(); body.basic_blocks.len()];
    for block in order {
        for successor in flows[*block]
            .ok_or_else(|| MirError::new(MirCode::Terminator, function_id))?
            .successors()
        {
            predecessors[successor].push(*block);
        }
    }
    let mut outputs = vec![None::<BTreeSet<usize>>; body.basic_blocks.len()];
    for block_index in topological {
        let mut live = if block_index == 0 {
            initial.clone()
        } else {
            let mut incoming = predecessors[block_index]
                .iter()
                .map(|predecessor| outputs[*predecessor].as_ref())
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| MirError::new(MirCode::Statement, function_id))?;
            let first = incoming
                .pop()
                .ok_or_else(|| MirError::new(MirCode::Statement, function_id))?;
            if incoming.iter().any(|state| *state != first) {
                return Err(MirError::new(MirCode::Statement, function_id));
            }
            first.clone()
        };
        let block = &body.basic_blocks[BasicBlock::new(block_index)];
        for statement in &block.statements {
            match &statement.kind {
                StatementKind::StorageLive(local) => {
                    if live.contains(&local.index()) {
                        return Err(MirError::new(MirCode::Statement, function_id));
                    }
                    live.insert(local.index());
                }
                StatementKind::StorageDead(local) => {
                    if !live.remove(&local.index()) {
                        return Err(MirError::new(MirCode::Statement, function_id));
                    }
                }
                StatementKind::Assign(assignment) => {
                    let (place, rvalue) = &**assignment;
                    if !live.contains(&place.local.index()) {
                        return Err(MirError::new(MirCode::Statement, function_id));
                    }
                    let mut reads = Vec::new();
                    collect_operand_locals(rvalue, &mut reads);
                    if reads.iter().any(|local| {
                        local.index() >= local_kinds.len() || !live.contains(&local.index())
                    }) {
                        return Err(MirError::new(MirCode::Statement, function_id));
                    }
                }
                StatementKind::Nop => {}
                _ => return Err(MirError::new(MirCode::Statement, function_id)),
            }
        }
        if let Some(call) = calls.get(block_index) {
            if !live.contains(&call.destination.index())
                || call.arguments.iter().any(|argument| {
                    operand_place(argument).is_some_and(|local| !live.contains(&local.index()))
                })
            {
                return Err(MirError::new(MirCode::Statement, function_id));
            }
        } else if let TerminatorKind::SwitchInt { discr, .. } = &block.terminator().kind {
            if operand_place(discr).is_some_and(|local| !live.contains(&local.index())) {
                return Err(MirError::new(MirCode::Statement, function_id));
            }
        } else if matches!(block.terminator().kind, TerminatorKind::Return) && !live.contains(&0) {
            return Err(MirError::new(MirCode::Statement, function_id));
        }
        outputs[block_index] = Some(live);
    }
    Ok(())
}

fn validate_destination(
    place: &Place<'_>,
    local_kinds: &[LocalKind],
    function_id: &str,
) -> Result<(), MirError> {
    if !place.projection.is_empty() {
        return Err(MirError::new(MirCode::Projection, function_id));
    }
    match local_kinds.get(place.local.index()) {
        Some(LocalKind::Argument(_)) | None => Err(MirError::new(MirCode::Place, function_id)),
        Some(_) => Ok(()),
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    match rvalue {
        Rvalue::Use(Operand::Copy(place)) if !place.projection.is_empty() => {
            let field = direct_field_projection(tcx, def_id, body, place)
                .map_err(|code| MirError::new(code, function_id))?
                .ok_or_else(|| MirError::new(MirCode::Projection, function_id))?;
            if field.base.index() >= local_kinds.len() {
                return Err(MirError::new(MirCode::Place, function_id));
            }
            reads.push(field.base);
            Ok(())
        }
        Rvalue::Use(operand) => validate_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            arithmetic,
            function_id,
            reads,
        ),
        Rvalue::UnaryOp(UnOp::Not, operand) => {
            validate_operand(
                tcx,
                def_id,
                body,
                operand,
                local_kinds,
                arithmetic,
                function_id,
                reads,
            )?;
            let ty = operand_type(tcx, def_id, body, operand, arithmetic, function_id)?;
            if ty != ContractType::Bool && ty.as_bit_vector().is_none() {
                return Err(MirError::new(MirCode::Rvalue, function_id));
            }
            Ok(())
        }
        Rvalue::BinaryOp(BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor, operands) => {
            for operand in [&operands.0, &operands.1] {
                validate_operand(
                    tcx,
                    def_id,
                    body,
                    operand,
                    local_kinds,
                    arithmetic,
                    function_id,
                    reads,
                )?;
            }
            let left = operand_type(tcx, def_id, body, &operands.0, arithmetic, function_id)?;
            let right = operand_type(tcx, def_id, body, &operands.1, arithmetic, function_id)?;
            if left != right || left.as_bit_vector().is_none() {
                return Err(MirError::new(MirCode::Rvalue, function_id));
            }
            Ok(())
        }
        Rvalue::BinaryOp(
            op @ (BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge),
            operands,
        ) => {
            validate_operand(
                tcx,
                def_id,
                body,
                &operands.0,
                local_kinds,
                arithmetic,
                function_id,
                reads,
            )?;
            validate_operand(
                tcx,
                def_id,
                body,
                &operands.1,
                local_kinds,
                arithmetic,
                function_id,
                reads,
            )?;
            let left = operand_type(tcx, def_id, body, &operands.0, arithmetic, function_id)?;
            let right = operand_type(tcx, def_id, body, &operands.1, arithmetic, function_id)?;
            if left != right
                || (!matches!(op, BinOp::Eq | BinOp::Ne) && left.as_bit_vector().is_none())
                || (matches!(op, BinOp::Eq | BinOp::Ne)
                    && left != ContractType::Bool
                    && left.as_bit_vector().is_none())
            {
                return Err(MirError::new(MirCode::Rvalue, function_id));
            }
            Ok(())
        }
        Rvalue::Aggregate(..) => {
            let (operands, maximum) = if let Some(operands) = array_operands(rvalue) {
                (operands, 256)
            } else if let Some(aggregate) = struct_aggregate(rvalue) {
                let adt = tcx.adt_def(aggregate.def_id);
                let mut pattern = StructAggregatePatternVector::pinned();
                pattern.definition_is_local_named_struct =
                    aggregate.def_id.is_local() && adt.is_struct();
                pattern.variant_is_only_variant = adt.is_struct() && aggregate.variant.index() == 0;
                pattern.arguments_are_empty = aggregate.arguments.is_empty();
                pattern.active_union_field_absent = aggregate.active_field.is_none();
                pattern.arity_matches = adt.is_struct()
                    && aggregate.operands.len() == adt.non_enum_variant().fields.len();
                pattern.within_limit = aggregate.operands.len() <= 64;
                validate_struct_aggregate_pattern(&pattern)
                    .map_err(|code| MirError::new(code, function_id))?;
                (aggregate.operands, 64)
            } else {
                return Err(MirError::new(MirCode::Rvalue, function_id));
            };
            if operands.len() > maximum {
                return Err(MirError::new(MirCode::IrLimit, function_id));
            }
            let array = array_operands(rvalue).is_some();
            let mut element_type = None;
            for operand in operands {
                validate_operand(
                    tcx,
                    def_id,
                    body,
                    operand,
                    local_kinds,
                    arithmetic,
                    function_id,
                    reads,
                )?;
                let ty = operand_type(tcx, def_id, body, operand, arithmetic, function_id)?;
                if array
                    && element_type
                        .as_ref()
                        .is_some_and(|expected| expected != &ty)
                {
                    return Err(MirError::new(MirCode::Rvalue, function_id));
                }
                element_type = Some(ty);
            }
            Ok(())
        }
        _ => Err(MirError::new(MirCode::Rvalue, function_id)),
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_checked_binary_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    expected_type: &ContractType,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    let Rvalue::BinaryOp(
        BinOp::AddWithOverflow | BinOp::SubWithOverflow | BinOp::MulWithOverflow,
        operands,
    ) = rvalue
    else {
        return Err(MirError::new(MirCode::Assertion, function_id));
    };
    for operand in [&operands.0, &operands.1] {
        validate_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            arithmetic,
            function_id,
            reads,
        )?;
        if operand_type(tcx, def_id, body, operand, arithmetic, function_id)? != *expected_type {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_checked_negation_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    expected_type: &ContractType,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    let Rvalue::UnaryOp(UnOp::Neg, operand) = rvalue else {
        return Err(MirError::new(MirCode::Assertion, function_id));
    };
    validate_operand(
        tcx,
        def_id,
        body,
        operand,
        local_kinds,
        arithmetic,
        function_id,
        reads,
    )?;
    if operand_type(tcx, def_id, body, operand, arithmetic, function_id)? != *expected_type {
        return Err(MirError::new(MirCode::Assertion, function_id));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_div_rem_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    expected_operation: DivRemOperation,
    expected_type: &ContractType,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    let Rvalue::BinaryOp(operation @ (BinOp::Div | BinOp::Rem), operands) = rvalue else {
        return Err(MirError::new(MirCode::Assertion, function_id));
    };
    let operation = match operation {
        BinOp::Div => DivRemOperation::Div,
        BinOp::Rem => DivRemOperation::Rem,
        _ => unreachable!("matched division or remainder"),
    };
    if operation != expected_operation {
        return Err(MirError::new(MirCode::Assertion, function_id));
    }
    for operand in [&operands.0, &operands.1] {
        validate_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            arithmetic,
            function_id,
            reads,
        )?;
        if operand_type(tcx, def_id, body, operand, arithmetic, function_id)? != *expected_type {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_shift_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    expected_operation: ShiftOperation,
    expected_lhs_type: &ContractType,
    expected_rhs_type: &ContractType,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    let Rvalue::BinaryOp(operation @ (BinOp::Shl | BinOp::Shr), operands) = rvalue else {
        return Err(MirError::new(MirCode::Assertion, function_id));
    };
    let operation = match operation {
        BinOp::Shl => ShiftOperation::Shl,
        BinOp::Shr => ShiftOperation::Shr,
        _ => unreachable!("matched shift"),
    };
    if operation != expected_operation {
        return Err(MirError::new(MirCode::Assertion, function_id));
    }
    for (operand, expected_type) in [
        (&operands.0, expected_lhs_type),
        (&operands.1, expected_rhs_type),
    ] {
        validate_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            arithmetic,
            function_id,
            reads,
        )?;
        if operand_type(tcx, def_id, body, operand, arithmetic, function_id)? != *expected_type {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_operand<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    operand: &Operand<'tcx>,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
    reads: &mut Vec<Local>,
) -> Result<(), MirError> {
    match operand {
        Operand::Copy(place) => {
            validate_read_place(place, local_kinds, arithmetic, function_id)?;
            reads.push(place.local);
            Ok(())
        }
        Operand::Move(place) => {
            if !place.projection.is_empty() && arithmetic.projected_type(place).is_none() {
                return Err(MirError::new(MirCode::Move, function_id));
            }
            validate_read_place(place, local_kinds, arithmetic, function_id)?;
            let ty = body.local_decls[place.local].ty;
            let typing_env = ty::TypingEnv::post_analysis(tcx, def_id);
            if ty.needs_drop(tcx, typing_env) {
                return Err(MirError::new(MirCode::Move, function_id));
            }
            reads.push(place.local);
            Ok(())
        }
        Operand::Constant(constant) => named_constant_value(tcx, &constant.const_)
            .map(|_| ())
            .map(Ok)
            .unwrap_or_else(|| literal_value(tcx, def_id, &constant.const_, function_id).map(drop)),
    }
}

fn validate_read_place(
    place: &Place<'_>,
    local_kinds: &[LocalKind],
    arithmetic: &ArithmeticPlan,
    function_id: &str,
) -> Result<(), MirError> {
    if !place.projection.is_empty() && arithmetic.projected_type(place).is_none() {
        return Err(MirError::new(MirCode::Projection, function_id));
    }
    if place.local.index() >= local_kinds.len() {
        return Err(MirError::new(MirCode::Place, function_id));
    }
    Ok(())
}

fn operand_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    operand: &Operand<'tcx>,
    arithmetic: &ArithmeticPlan,
    function_id: &str,
) -> Result<ContractType, MirError> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            if let Some(ty) = arithmetic.projected_type(place) {
                return Ok(ty.clone());
            }
            if !place.projection.is_empty() {
                return Err(MirError::new(MirCode::Projection, function_id));
            }
            mir_contract_type(tcx, def_id, body.local_decls[place.local].ty, function_id)
        }
        Operand::Constant(constant) => {
            mir_contract_type(tcx, def_id, constant.const_.ty(), function_id)
        }
    }
}

fn collect_operand_locals(rvalue: &Rvalue<'_>, reads: &mut Vec<Local>) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp(_, operand) => {
            if let Some(local) = operand_place(operand) {
                reads.push(local);
            }
        }
        Rvalue::BinaryOp(_, operands) => {
            if let Some(local) = operand_place(&operands.0) {
                reads.push(local);
            }
            if let Some(local) = operand_place(&operands.1) {
                reads.push(local);
            }
        }
        Rvalue::Aggregate(..) => {
            let Rvalue::Aggregate(_, operands) = rvalue else {
                unreachable!("matched aggregate")
            };
            for operand in operands {
                if let Some(local) = operand_place(operand) {
                    reads.push(local);
                }
            }
        }
        _ => {}
    }
}

fn operand_place(operand: &Operand<'_>) -> Option<Local> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => Some(place.local),
        Operand::Constant(_) => None,
    }
}

fn is_compiler_local(kind: LocalKind) -> bool {
    matches!(kind, LocalKind::Result | LocalKind::Temporary)
}

fn is_modeled_compiler_local(body: &Body<'_>, kinds: &[LocalKind], local: usize) -> bool {
    is_compiler_local(kinds[local]) && !body.local_decls[Local::new(local)].ty.is_unit()
}

fn is_erasable_unit_assignment(
    body: &Body<'_>,
    destination: &Place<'_>,
    rvalue: &Rvalue<'_>,
    kinds: &[LocalKind],
) -> bool {
    if !destination.projection.is_empty()
        || !matches!(kinds[destination.local.index()], LocalKind::Temporary)
        || !body.local_decls[destination.local].ty.is_unit()
    {
        return false;
    }
    match rvalue {
        Rvalue::Use(Operand::Constant(constant)) => constant.const_.ty().is_unit(),
        Rvalue::Use(Operand::Copy(place) | Operand::Move(place)) => {
            place.projection.is_empty() && body.local_decls[place.local].ty.is_unit()
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_assignment<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    destination: &Place<'tcx>,
    rvalue: &Rvalue<'tcx>,
    statement_span: Span,
    mir_block: usize,
    statement_index: usize,
    arithmetic: &ArithmeticPlan,
    projection: &ProjectionPlan,
    function: &HirFunction,
    structs: &[HirStructDecl],
    local_kinds: &[LocalKind],
    local_binding_spans: &[Span],
    initialized_user_locals: &mut BTreeSet<usize>,
    environment: &mut BTreeMap<usize, Value>,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    loader: &SnapshotFileLoader,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<(), MirError> {
    validate_destination(destination, local_kinds, function_id)?;
    if projection.is_guard(mir_block, statement_index)
        || arithmetic.is_negation_guard(mir_block, statement_index)
        || arithmetic.is_div_rem_guard(mir_block, statement_index)
        || arithmetic.is_shift_guard(mir_block, statement_index)
    {
        return Ok(());
    }
    if is_erasable_unit_assignment(body, destination, rvalue, local_kinds) {
        return Ok(());
    }
    let direct_field = match rvalue {
        Rvalue::Use(Operand::Copy(place)) => direct_field_projection(tcx, def_id, body, place)
            .map_err(|code| MirError::new(code, function_id))?,
        _ => None,
    };
    let value = if let Some(field) = direct_field {
        let base = resolve_local(
            tcx,
            def_id,
            body,
            field.base,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if base.ty != field.base_ty {
            return Err(MirError::new(MirCode::Projection, function_id));
        }
        let field_span = enclosing_span(statement_span, &function.field_spans)
            .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?;
        emit_field(
            &field,
            base,
            field_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else if let Some(index) = projection.index(mir_block, statement_index) {
        let base = resolve_local(
            tcx,
            def_id,
            body,
            index.base,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        let index_value = resolve_local(
            tcx,
            def_id,
            body,
            index.index,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if base.ty != *index.base_ty || index_value.ty != *index.index_ty {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        let ContractType::Array { element, length } = &base.ty else {
            return Err(MirError::new(MirCode::Assertion, function_id));
        };
        if element.as_ref() != index.element_ty || *length != index.length {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        emit_index(
            index.element_ty,
            base,
            index_value,
            statement_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else if let Some((operation, ty)) = arithmetic.binary(mir_block, statement_index) {
        let Rvalue::BinaryOp(_, operands) = rvalue else {
            return Err(MirError::new(MirCode::Assertion, function_id));
        };
        let left = lower_operand(
            tcx,
            def_id,
            body,
            &operands.0,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        let right = lower_operand(
            tcx,
            def_id,
            body,
            &operands.1,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if left.ty != *ty || right.ty != *ty {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        emit_arithmetic(
            operation,
            ty,
            Some(left),
            right,
            statement_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else if let Some(ty) = arithmetic.negation(mir_block, statement_index) {
        let Rvalue::UnaryOp(UnOp::Neg, operand) = rvalue else {
            return Err(MirError::new(MirCode::Assertion, function_id));
        };
        let operand = lower_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if operand.ty != *ty {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        emit_arithmetic(
            ArithmeticOperation::Neg,
            ty,
            None,
            operand,
            statement_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else if let Some((operation, ty)) = arithmetic.div_rem(mir_block, statement_index) {
        let Rvalue::BinaryOp(_, operands) = rvalue else {
            return Err(MirError::new(MirCode::Assertion, function_id));
        };
        let left = lower_operand(
            tcx,
            def_id,
            body,
            &operands.0,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        let right = lower_operand(
            tcx,
            def_id,
            body,
            &operands.1,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if left.ty != *ty || right.ty != *ty {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        emit_div_rem(
            operation,
            ty,
            left,
            right,
            statement_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else if let Some((operation, lhs_ty, rhs_ty)) = arithmetic.shift(mir_block, statement_index) {
        let Rvalue::BinaryOp(_, operands) = rvalue else {
            return Err(MirError::new(MirCode::Assertion, function_id));
        };
        let left = lower_operand(
            tcx,
            def_id,
            body,
            &operands.0,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        let right = lower_operand(
            tcx,
            def_id,
            body,
            &operands.1,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if left.ty != *lhs_ty || right.ty != *rhs_ty {
            return Err(MirError::new(MirCode::Assertion, function_id));
        }
        emit_shift(
            operation,
            lhs_ty,
            rhs_ty,
            left,
            right,
            statement_span,
            tcx,
            loader,
            ids,
            next_instruction,
            block_index,
            unit_id,
            function_id,
            instructions,
            source_entries,
        )?
    } else {
        match rvalue {
            Rvalue::Use(Operand::Constant(constant)) => {
                if let Some(mut reference) = named_constant_value(tcx, &constant.const_) {
                    reference.span = statement_span;
                    reference
                } else {
                    let literal = literal_value(tcx, def_id, &constant.const_, function_id)?;
                    let literal_span = negative_literal_span(
                        tcx,
                        def_id,
                        &constant.const_,
                        constant.span,
                        statement_span,
                        function,
                        function_id,
                    )?;
                    let mut instruction =
                        instruction_base(ids, next_instruction, "Const", &literal.ty, function_id)?;
                    instruction.insert("value".to_owned(), literal.json);
                    emitted_value(
                        instruction,
                        literal.ty,
                        literal_span,
                        tcx,
                        loader,
                        block_index,
                        unit_id,
                        function_id,
                        instructions,
                        source_entries,
                        next_instruction.last_index(),
                    )?
                }
            }
            Rvalue::Use(operand) => lower_operand(
                tcx,
                def_id,
                body,
                operand,
                local_kinds,
                environment,
                arithmetic,
                function_id,
            )?,
            Rvalue::UnaryOp(UnOp::Not, operand) => {
                let operand = lower_operand(
                    tcx,
                    def_id,
                    body,
                    operand,
                    local_kinds,
                    environment,
                    arithmetic,
                    function_id,
                )?;
                let (operation, result_type) = if operand.ty == ContractType::Bool {
                    ("not", ContractType::Bool)
                } else if operand.ty.as_bit_vector().is_some() {
                    ("bv_not", operand.ty.clone())
                } else {
                    return Err(MirError::new(MirCode::Rvalue, function_id));
                };
                let mut instruction =
                    instruction_base(ids, next_instruction, "UnaryOp", &result_type, function_id)?;
                instruction.insert("op".to_owned(), string(operation));
                instruction.insert("value".to_owned(), operand.json);
                emitted_value(
                    instruction,
                    result_type,
                    statement_span,
                    tcx,
                    loader,
                    block_index,
                    unit_id,
                    function_id,
                    instructions,
                    source_entries,
                    next_instruction.last_index(),
                )?
            }
            Rvalue::BinaryOp(
                operation @ (BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor),
                operands,
            ) => {
                let left = lower_operand(
                    tcx,
                    def_id,
                    body,
                    &operands.0,
                    local_kinds,
                    environment,
                    arithmetic,
                    function_id,
                )?;
                let right = lower_operand(
                    tcx,
                    def_id,
                    body,
                    &operands.1,
                    local_kinds,
                    environment,
                    arithmetic,
                    function_id,
                )?;
                if left.ty != right.ty || left.ty.as_bit_vector().is_none() {
                    return Err(MirError::new(MirCode::Rvalue, function_id));
                }
                let operation = match operation {
                    BinOp::BitAnd => "bv_and",
                    BinOp::BitOr => "bv_or",
                    BinOp::BitXor => "bv_xor",
                    _ => unreachable!("matched bitwise operation"),
                };
                let mut instruction =
                    instruction_base(ids, next_instruction, "BinOp", &left.ty, function_id)?;
                instruction.insert("op".to_owned(), string(operation));
                instruction.insert("lhs".to_owned(), left.json);
                instruction.insert("rhs".to_owned(), right.json);
                emitted_value(
                    instruction,
                    left.ty,
                    statement_span,
                    tcx,
                    loader,
                    block_index,
                    unit_id,
                    function_id,
                    instructions,
                    source_entries,
                    next_instruction.last_index(),
                )?
            }
            Rvalue::BinaryOp(
                op @ (BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge),
                operands,
            ) => {
                let left = lower_operand(
                    tcx,
                    def_id,
                    body,
                    &operands.0,
                    local_kinds,
                    environment,
                    arithmetic,
                    function_id,
                )?;
                let right = lower_operand(
                    tcx,
                    def_id,
                    body,
                    &operands.1,
                    local_kinds,
                    environment,
                    arithmetic,
                    function_id,
                )?;
                let operation = comparison_operation(*op, &left.ty, &right.ty, function_id)?;
                let mut instruction = instruction_base(
                    ids,
                    next_instruction,
                    "BinOp",
                    &ContractType::Bool,
                    function_id,
                )?;
                instruction.insert("op".to_owned(), string(operation));
                instruction.insert("lhs".to_owned(), left.json);
                instruction.insert("rhs".to_owned(), right.json);
                emitted_value(
                    instruction,
                    ContractType::Bool,
                    statement_span,
                    tcx,
                    loader,
                    block_index,
                    unit_id,
                    function_id,
                    instructions,
                    source_entries,
                    next_instruction.last_index(),
                )?
            }
            Rvalue::Aggregate(..) => {
                let destination_type = mir_contract_type(
                    tcx,
                    def_id,
                    body.local_decls[destination.local].ty,
                    function_id,
                )?;
                if let Some(operands) = array_operands(rvalue) {
                    let ContractType::Array { element, length } = &destination_type else {
                        return Err(MirError::new(MirCode::Rvalue, function_id));
                    };
                    let mut pattern = ArrayAggregatePatternVector::pinned();
                    pattern.within_limit = operands.len() <= 256;
                    pattern.arity_matches = u64::try_from(operands.len()).ok() == Some(*length);
                    let mut elements = Vec::with_capacity(operands.len());
                    for operand in operands {
                        let value = lower_operand(
                            tcx,
                            def_id,
                            body,
                            operand,
                            local_kinds,
                            environment,
                            arithmetic,
                            function_id,
                        )?;
                        if &value.ty != element.as_ref() {
                            pattern.element_types_match = false;
                        }
                        elements.push(value.json);
                    }
                    validate_array_aggregate_pattern(&pattern)
                        .map_err(|code| MirError::new(code, function_id))?;
                    let mut instruction = instruction_base(
                        ids,
                        next_instruction,
                        "MakeArray",
                        &destination_type,
                        function_id,
                    )?;
                    instruction.insert("elements".to_owned(), JsonValue::Array(elements));
                    let array_span = enclosing_span(statement_span, &function.array_spans)
                        .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?;
                    emitted_value(
                        instruction,
                        destination_type,
                        array_span,
                        tcx,
                        loader,
                        block_index,
                        unit_id,
                        function_id,
                        instructions,
                        source_entries,
                        next_instruction.last_index(),
                    )?
                } else {
                    lower_struct_aggregate(
                        tcx,
                        def_id,
                        body,
                        rvalue,
                        destination_type,
                        statement_span,
                        function,
                        structs,
                        local_kinds,
                        environment,
                        arithmetic,
                        loader,
                        ids,
                        next_instruction,
                        block_index,
                        unit_id,
                        function_id,
                        instructions,
                        source_entries,
                    )?
                }
            }
            _ => return Err(MirError::new(MirCode::Rvalue, function_id)),
        }
    };
    let destination_type = arithmetic
        .scalar_local_type(destination.local)
        .cloned()
        .map(Ok)
        .unwrap_or_else(|| {
            mir_contract_type(
                tcx,
                def_id,
                body.local_decls[destination.local].ty,
                function_id,
            )
        })?;
    if destination_type != value.ty {
        return Err(MirError::new(MirCode::Rvalue, function_id));
    }
    store_destination_value(
        destination.local,
        value,
        statement_span,
        local_kinds,
        local_binding_spans,
        initialized_user_locals,
        environment,
        tcx,
        loader,
        ids,
        next_instruction,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_static_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    call: &PlannedCall<'tcx>,
    arithmetic: &ArithmeticPlan,
    function: &HirFunction,
    local_kinds: &[LocalKind],
    initialized_user_locals: &mut BTreeSet<usize>,
    environment: &mut BTreeMap<usize, Value>,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    loader: &SnapshotFileLoader,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<(), MirError> {
    let mut arguments = Vec::with_capacity(call.arguments.len());
    for (operand, expected_type) in call.arguments.iter().zip(&call.parameter_types) {
        let value = lower_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if &value.ty != expected_type {
            return Err(MirError::new(MirCode::Call, function_id));
        }
        arguments.push(value.json);
    }
    if arguments.len() != call.arguments.len() || call.arguments.len() != call.parameter_types.len()
    {
        return Err(MirError::new(MirCode::Call, function_id));
    }
    let mut instruction = instruction_base(
        ids,
        next_instruction,
        "CallStatic",
        &call.result_type,
        function_id,
    )?;
    instruction.insert("function".to_owned(), string(&call.callee));
    instruction.insert("contract_hash".to_owned(), string(&call.contract_hash));
    instruction.insert("args".to_owned(), JsonValue::Array(arguments));
    let value = emitted_value(
        instruction,
        call.result_type.clone(),
        call.span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )?;
    let destination_type = mir_contract_type(
        tcx,
        def_id,
        body.local_decls[call.destination].ty,
        function_id,
    )?;
    if destination_type != call.result_type || value.ty != call.result_type {
        return Err(MirError::new(MirCode::Call, function_id));
    }
    store_destination_value(
        call.destination,
        value,
        call.span,
        local_kinds,
        &function.local_spans,
        initialized_user_locals,
        environment,
        tcx,
        loader,
        ids,
        next_instruction,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
    )
}

#[allow(clippy::too_many_arguments)]
fn store_destination_value(
    destination: Local,
    mut value: Value,
    assignment_span: Span,
    local_kinds: &[LocalKind],
    local_binding_spans: &[Span],
    initialized_user_locals: &mut BTreeSet<usize>,
    environment: &mut BTreeMap<usize, Value>,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<(), MirError> {
    match local_kinds.get(destination.index()) {
        Some(LocalKind::Result | LocalKind::Temporary) => {
            value.span = assignment_span;
            environment.insert(destination.index(), value);
        }
        Some(LocalKind::User(local_index)) => {
            let local_index = *local_index;
            let copy_span = if initialized_user_locals.insert(local_index) {
                *local_binding_spans
                    .get(local_index)
                    .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?
            } else {
                assignment_span
            };
            let mut instruction =
                instruction_base(ids, next_instruction, "Copy", &value.ty, function_id)?;
            instruction.insert("target".to_owned(), string(&format!("local{local_index}")));
            instruction.insert("value".to_owned(), value.json);
            let _ = emitted_value(
                instruction,
                value.ty,
                copy_span,
                tcx,
                loader,
                block_index,
                unit_id,
                function_id,
                instructions,
                source_entries,
                next_instruction.last_index(),
            )?;
        }
        Some(LocalKind::Argument(_)) | None => {
            return Err(MirError::new(MirCode::Place, function_id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn lower_struct_aggregate<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    destination_type: ContractType,
    statement_span: Span,
    function: &HirFunction,
    structs: &[HirStructDecl],
    local_kinds: &[LocalKind],
    environment: &BTreeMap<usize, Value>,
    arithmetic: &ArithmeticPlan,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let aggregate =
        struct_aggregate(rvalue).ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    let adt = tcx.adt_def(aggregate.def_id);
    let declaration = aggregate.def_id.as_local().and_then(|aggregate_id| {
        structs
            .iter()
            .find(|declaration| declaration.def_id == aggregate_id)
    });
    let mut pattern = StructAggregatePatternVector::pinned();
    pattern.definition_is_local_named_struct =
        aggregate.def_id.is_local() && adt.is_struct() && declaration.is_some();
    pattern.variant_is_only_variant = adt.is_struct() && aggregate.variant.index() == 0;
    pattern.arguments_are_empty = aggregate.arguments.is_empty();
    pattern.active_union_field_absent = aggregate.active_field.is_none();
    let Some(declaration) = declaration else {
        validate_struct_aggregate_pattern(&pattern)
            .map_err(|code| MirError::new(code, function_id))?;
        unreachable!("missing struct declaration rejected")
    };
    pattern.destination_matches = matches!(
        &destination_type,
        ContractType::Struct { id } if id == &declaration.id
    );
    pattern.arity_matches = aggregate.operands.len() == declaration.fields.len();
    pattern.within_limit = aggregate.operands.len() <= 64;

    let mut fields = Vec::with_capacity(aggregate.operands.len());
    for (operand, field) in aggregate.operands.iter().zip(&declaration.fields) {
        let value = lower_operand(
            tcx,
            def_id,
            body,
            operand,
            local_kinds,
            environment,
            arithmetic,
            function_id,
        )?;
        if value.ty != field.ty {
            pattern.field_types_match = false;
        }
        fields.push(JsonValue::Object(BTreeMap::from([
            ("name".to_owned(), string(&field.name)),
            ("value".to_owned(), value.json),
        ])));
    }
    validate_struct_aggregate_pattern(&pattern).map_err(|code| MirError::new(code, function_id))?;
    let mut instruction = instruction_base(
        ids,
        next_instruction,
        "MakeStruct",
        &destination_type,
        function_id,
    )?;
    instruction.insert("fields".to_owned(), JsonValue::Array(fields));
    let struct_span = enclosing_span(statement_span, &function.struct_spans)
        .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?;
    emitted_value(
        instruction,
        destination_type,
        struct_span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_field(
    field: &FieldProjection,
    base: Value,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let mut instruction =
        instruction_base(ids, next_instruction, "Field", &field.field_ty, function_id)?;
    instruction.insert("base".to_owned(), base.json);
    instruction.insert("field".to_owned(), string(&field.field));
    emitted_value(
        instruction,
        field.field_ty.clone(),
        span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_arithmetic(
    operation: ArithmeticOperation,
    ty: &ContractType,
    left: Option<Value>,
    right: Value,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let kind = if operation == ArithmeticOperation::Neg {
        "UnaryOp"
    } else {
        "BinOp"
    };
    let mut instruction = instruction_base(ids, next_instruction, kind, ty, function_id)?;
    instruction.insert("op".to_owned(), string(operation.vir_name()));
    if let Some(left) = left {
        instruction.insert("lhs".to_owned(), left.json);
        instruction.insert("rhs".to_owned(), right.json);
    } else {
        instruction.insert("value".to_owned(), right.json);
    }
    instruction.insert(
        "safety_checks".to_owned(),
        JsonValue::Array(vec![integer_no_overflow(operation, ty, function_id)?]),
    );
    emitted_value(
        instruction,
        ty.clone(),
        span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_index(
    element_ty: &ContractType,
    base: Value,
    index: Value,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let mut instruction =
        instruction_base(ids, next_instruction, "Index", element_ty, function_id)?;
    instruction.insert("base".to_owned(), base.json);
    instruction.insert("index".to_owned(), index.json);
    instruction.insert(
        "safety_checks".to_owned(),
        JsonValue::Array(vec![JsonValue::Object(BTreeMap::from([(
            "kind".to_owned(),
            string("index_in_bounds"),
        )]))]),
    );
    emitted_value(
        instruction,
        element_ty.clone(),
        span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_div_rem(
    operation: DivRemOperation,
    ty: &ContractType,
    left: Value,
    right: Value,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let (_, signed) = ty
        .as_bit_vector()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    let mut instruction = instruction_base(ids, next_instruction, "BinOp", ty, function_id)?;
    instruction.insert("op".to_owned(), string(operation.vir_name(signed)));
    instruction.insert("lhs".to_owned(), left.json);
    instruction.insert("rhs".to_owned(), right.json);
    let mut safety_checks = vec![JsonValue::Object(BTreeMap::from([(
        "kind".to_owned(),
        string("divisor_nonzero"),
    )]))];
    if signed {
        safety_checks.push(JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), string("signed_divrem_representable")),
            ("operation".to_owned(), string(operation.safety_name())),
        ])));
    }
    instruction.insert("safety_checks".to_owned(), JsonValue::Array(safety_checks));
    emitted_value(
        instruction,
        ty.clone(),
        span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_shift(
    operation: ShiftOperation,
    lhs_ty: &ContractType,
    rhs_ty: &ContractType,
    left: Value,
    right: Value,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<Value, MirError> {
    let (_, lhs_signed) = lhs_ty
        .as_bit_vector()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    let (_, rhs_signed) = rhs_ty
        .as_bit_vector()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    let mut instruction = instruction_base(ids, next_instruction, "BinOp", lhs_ty, function_id)?;
    instruction.insert("op".to_owned(), string(operation.vir_name(lhs_signed)));
    instruction.insert("lhs".to_owned(), left.json);
    instruction.insert("rhs".to_owned(), right.json);
    let mut checks = Vec::new();
    if rhs_signed {
        checks.push(JsonValue::Object(BTreeMap::from([(
            "kind".to_owned(),
            string("shift_count_nonnegative"),
        )])));
    }
    checks.push(JsonValue::Object(BTreeMap::from([(
        "kind".to_owned(),
        string("shift_count_less_than_width"),
    )])));
    instruction.insert("safety_checks".to_owned(), JsonValue::Array(checks));
    emitted_value(
        instruction,
        lhs_ty.clone(),
        span,
        tcx,
        loader,
        block_index,
        unit_id,
        function_id,
        instructions,
        source_entries,
        next_instruction.last_index(),
    )
}

fn integer_no_overflow(
    operation: ArithmeticOperation,
    ty: &ContractType,
    function_id: &str,
) -> Result<JsonValue, MirError> {
    let (_, signed) = ty
        .as_bit_vector()
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
    Ok(JsonValue::Object(BTreeMap::from([
        ("kind".to_owned(), string("integer_no_overflow")),
        ("operation".to_owned(), string(operation.safety_name())),
        ("signed".to_owned(), JsonValue::Bool(signed)),
    ])))
}

fn negative_literal_span<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    constant: &rustc_middle::mir::Const<'tcx>,
    constant_span: Span,
    statement_span: Span,
    function: &HirFunction,
    function_id: &str,
) -> Result<Span, MirError> {
    let ContractType::BitVector {
        width,
        signed: true,
    } = mir_contract_type(tcx, def_id, constant.ty(), function_id)?
    else {
        return Ok(constant_span);
    };
    let bits = constant
        .try_eval_bits(tcx, ty::TypingEnv::post_analysis(tcx, def_id))
        .ok_or_else(|| MirError::new(MirCode::Operand, function_id))?;
    if bits & (1_u128 << (width - 1)) == 0 {
        return Ok(constant_span);
    }
    function
        .negative_literal_spans
        .iter()
        .copied()
        .filter(|candidate| {
            span_contains(*candidate, constant_span) || span_contains(*candidate, statement_span)
        })
        .min_by_key(|candidate| {
            (
                candidate.hi().0 - candidate.lo().0,
                candidate.lo().0,
                candidate.hi().0,
            )
        })
        .ok_or_else(|| MirError::new(MirCode::Assertion, function_id))
}

#[allow(clippy::too_many_arguments)]
fn emitted_value(
    instruction: BTreeMap<String, JsonValue>,
    ty: ContractType,
    span: Span,
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
    instruction_index: usize,
) -> Result<Value, MirError> {
    let id = instruction
        .get("id")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?
        .to_owned();
    instructions.push(JsonValue::Object(instruction));
    source_entries.push(SourceMapEntry {
        reference: VirReference::Instruction {
            unit_id: unit_id.to_owned(),
            function_id: function_id.to_owned(),
            block_index,
            instruction_index,
        },
        origin: source_origin(tcx, loader, span, function_id)?,
    });
    Ok(Value {
        json: variable(&id),
        ty,
        span,
    })
}

fn instruction_base(
    ids: &mut DenseIds,
    next_instruction: &mut InstructionBudget,
    kind: &str,
    ty: &ContractType,
    function_id: &str,
) -> Result<BTreeMap<String, JsonValue>, MirError> {
    let instruction_index = next_instruction.reserve(function_id)?;
    let id = ids.temporary();
    debug_assert_eq!(id, format!("t{instruction_index}"));
    Ok(BTreeMap::from([
        ("id".to_owned(), string(&id)),
        ("kind".to_owned(), string(kind)),
        ("type".to_owned(), vir_type(ty)),
        ("safety_checks".to_owned(), JsonValue::Array(Vec::new())),
    ]))
}

#[allow(clippy::too_many_arguments)]
fn lower_operand<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    operand: &Operand<'tcx>,
    local_kinds: &[LocalKind],
    environment: &BTreeMap<usize, Value>,
    arithmetic: &ArithmeticPlan,
    function_id: &str,
) -> Result<Value, MirError> {
    match operand {
        Operand::Copy(place) => {
            validate_read_place(place, local_kinds, arithmetic, function_id)?;
            resolve_local(
                tcx,
                def_id,
                body,
                place.local,
                local_kinds,
                environment,
                arithmetic,
                function_id,
            )
        }
        Operand::Move(place) => {
            if !place.projection.is_empty() && arithmetic.projected_type(place).is_none() {
                return Err(MirError::new(MirCode::Move, function_id));
            }
            validate_read_place(place, local_kinds, arithmetic, function_id)?;
            let mir_ty = body.local_decls[place.local].ty;
            let typing_env = ty::TypingEnv::post_analysis(tcx, def_id);
            if mir_ty.needs_drop(tcx, typing_env) {
                return Err(MirError::new(MirCode::Move, function_id));
            }
            resolve_local(
                tcx,
                def_id,
                body,
                place.local,
                local_kinds,
                environment,
                arithmetic,
                function_id,
            )
        }
        Operand::Constant(constant) => named_constant_value(tcx, &constant.const_)
            .map(Ok)
            .unwrap_or_else(|| literal_value(tcx, def_id, &constant.const_, function_id)),
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_local<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    local: Local,
    local_kinds: &[LocalKind],
    environment: &BTreeMap<usize, Value>,
    arithmetic: &ArithmeticPlan,
    function_id: &str,
) -> Result<Value, MirError> {
    match local_kinds
        .get(local.index())
        .copied()
        .ok_or_else(|| MirError::new(MirCode::Place, function_id))?
    {
        LocalKind::Argument(index) => Ok(Value {
            json: variable(&format!("arg{index}")),
            ty: arithmetic
                .scalar_local_type(local)
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| {
                    mir_contract_type(tcx, def_id, body.local_decls[local].ty, function_id)
                })?,
            span: body.span,
        }),
        LocalKind::User(index) => Ok(Value {
            json: variable(&format!("local{index}")),
            ty: arithmetic
                .scalar_local_type(local)
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| {
                    mir_contract_type(tcx, def_id, body.local_decls[local].ty, function_id)
                })?,
            span: body.span,
        }),
        LocalKind::Result | LocalKind::Temporary => environment
            .get(&local.index())
            .cloned()
            .ok_or_else(|| MirError::new(MirCode::Operand, function_id)),
    }
}

fn literal_value<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    constant: &rustc_middle::mir::Const<'tcx>,
    function_id: &str,
) -> Result<Value, MirError> {
    match constant {
        rustc_middle::mir::Const::Unevaluated(..) => {
            return Err(MirError::new(MirCode::Operand, function_id));
        }
        rustc_middle::mir::Const::Ty(_, value)
            if !matches!(value.kind(), ty::ConstKind::Value(_)) =>
        {
            return Err(MirError::new(MirCode::Operand, function_id));
        }
        _ => {}
    }
    let ty = mir_contract_type(tcx, def_id, constant.ty(), function_id)?;
    let typing_env = ty::TypingEnv::post_analysis(tcx, def_id);
    let json = match ty {
        ContractType::Bool => JsonValue::Object(BTreeMap::from([(
            "bool".to_owned(),
            JsonValue::Bool(
                constant
                    .try_eval_bool(tcx, typing_env)
                    .ok_or_else(|| MirError::new(MirCode::Operand, function_id))?,
            ),
        )])),
        ContractType::BitVector { width, signed } => {
            let bits = constant
                .try_eval_bits(tcx, typing_env)
                .ok_or_else(|| MirError::new(MirCode::Operand, function_id))?;
            let decimal = if signed && bits & (1_u128 << (width - 1)) != 0 {
                (i128::try_from(bits).map_err(|_| MirError::new(MirCode::Operand, function_id))?
                    - (1_i128 << width))
                    .to_string()
            } else {
                bits.to_string()
            };
            JsonValue::Object(BTreeMap::from([(
                "int".to_owned(),
                JsonValue::Object(BTreeMap::from([
                    ("value".to_owned(), string(&decimal)),
                    ("width".to_owned(), JsonValue::Number(width.to_string())),
                    ("signed".to_owned(), JsonValue::Bool(signed)),
                ])),
            )]))
        }
        ContractType::Array { .. } | ContractType::Struct { .. } => {
            return Err(MirError::new(MirCode::Rvalue, function_id));
        }
    };
    Ok(Value {
        json,
        ty,
        span: rustc_span::DUMMY_SP,
    })
}

fn named_constant_value<'tcx>(
    tcx: TyCtxt<'tcx>,
    constant: &rustc_middle::mir::Const<'tcx>,
) -> Option<Value> {
    let (id, ty) = const_lower::reference(tcx, constant)?;
    Some(Value {
        json: JsonValue::Object(BTreeMap::from([("const".to_owned(), string(&id))])),
        ty,
        span: rustc_span::DUMMY_SP,
    })
}

fn comparison_operation(
    operation: BinOp,
    left: &ContractType,
    right: &ContractType,
    function_id: &str,
) -> Result<&'static str, MirError> {
    if left != right {
        return Err(MirError::new(MirCode::Rvalue, function_id));
    }
    match operation {
        BinOp::Eq if left == &ContractType::Bool || left.as_bit_vector().is_some() => Ok("eq"),
        BinOp::Ne if left == &ContractType::Bool || left.as_bit_vector().is_some() => Ok("not_eq"),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let (_, signed) = left
                .as_bit_vector()
                .ok_or_else(|| MirError::new(MirCode::Rvalue, function_id))?;
            Ok(match (signed, operation) {
                (true, BinOp::Lt) => "signed_lt",
                (true, BinOp::Le) => "signed_le",
                (true, BinOp::Gt) => "signed_gt",
                (true, BinOp::Ge) => "signed_ge",
                (false, BinOp::Lt) => "unsigned_lt",
                (false, BinOp::Le) => "unsigned_le",
                (false, BinOp::Gt) => "unsigned_gt",
                (false, BinOp::Ge) => "unsigned_ge",
                _ => unreachable!("closed comparison operation"),
            })
        }
        _ => Err(MirError::new(MirCode::Rvalue, function_id)),
    }
}

fn edge_arguments(
    target: usize,
    live_in: &[BTreeSet<usize>],
    environment: &BTreeMap<usize, Value>,
    block_parameters: &[BTreeMap<usize, String>],
    function_id: &str,
) -> Result<Vec<JsonValue>, MirError> {
    if live_in.get(target).is_none() || block_parameters.get(target).is_none() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    live_in[target]
        .iter()
        .map(|local| {
            if !block_parameters[target].contains_key(local) {
                return Err(MirError::new(MirCode::Operand, function_id));
            }
            environment
                .get(local)
                .map(|value| value.json.clone())
                .ok_or_else(|| MirError::new(MirCode::Operand, function_id))
        })
        .collect()
}

fn jump(label: &str, arguments: Vec<JsonValue>) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("kind".to_owned(), string("Jump")),
        ("label".to_owned(), string(label)),
        ("args".to_owned(), JsonValue::Array(arguments)),
    ]))
}

fn branch(
    condition: JsonValue,
    then_label: &str,
    then_arguments: Vec<JsonValue>,
    else_label: &str,
    else_arguments: Vec<JsonValue>,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("kind".to_owned(), string("Branch")),
        ("cond".to_owned(), condition),
        ("then_label".to_owned(), string(then_label)),
        ("then_args".to_owned(), JsonValue::Array(then_arguments)),
        ("else_label".to_owned(), string(else_label)),
        ("else_args".to_owned(), JsonValue::Array(else_arguments)),
    ]))
}

fn binding(id: &str, ty: &ContractType) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_owned(), string(id)),
        ("type".to_owned(), vir_type(ty)),
    ]))
}

fn variable(id: &str) -> JsonValue {
    JsonValue::Object(BTreeMap::from([("var".to_owned(), string(id))]))
}

fn mir_contract_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    ty: rustc_middle::ty::Ty<'tcx>,
    function_id: &str,
) -> Result<ContractType, MirError> {
    contract_type(tcx, def_id, ty).map_err(|_| MirError::new(MirCode::Rvalue, function_id))
}

fn is_scalar(ty: &ContractType) -> bool {
    matches!(ty, ContractType::Bool | ContractType::BitVector { .. })
}

fn is_struct_value_type(ty: &ContractType) -> bool {
    is_scalar(ty)
        || matches!(
            ty,
            ContractType::Array { element, .. } if is_struct_value_type(element)
        )
        || matches!(ty, ContractType::Struct { .. })
}

fn is_array_type(ty: &ContractType) -> bool {
    matches!(ty, ContractType::Array { .. })
}

fn is_struct_type(ty: &ContractType) -> bool {
    match ty {
        ContractType::Array { element, .. } => is_struct_type(element),
        ContractType::Struct { .. } => true,
        ContractType::Bool | ContractType::BitVector { .. } => false,
    }
}

fn source_origin(
    tcx: TyCtxt<'_>,
    loader: &SnapshotFileLoader,
    span: Span,
    function_id: &str,
) -> Result<SourceOrigin, MirError> {
    if span.from_expansion() {
        return Err(MirError::new(MirCode::SourceMapExternal, function_id));
    }
    if span.is_dummy() || span.lo() >= span.hi() {
        return Err(MirError::new(MirCode::SourceMapRange, function_id));
    }
    let source_map = tcx.sess.source_map();
    let low = source_map.lookup_byte_offset(span.lo());
    let high = source_map.lookup_byte_offset(span.hi());
    if low.sf.start_pos != high.sf.start_pos || low.sf.name != high.sf.name {
        return Err(MirError::new(MirCode::SourceMapRange, function_id));
    }
    if !matches!(low.sf.name, FileName::Real(_)) {
        return Err(MirError::new(MirCode::SourceMapExternal, function_id));
    }
    let path = low.sf.name.prefer_local().to_string_lossy();
    let captured = loader
        .captured_source_range(
            Path::new(path.as_ref()),
            u64::from(low.pos.0),
            u64::from(high.pos.0),
        )
        .map_err(|error| {
            MirError::new(
                match error {
                    SourceRangeError::External => MirCode::SourceMapExternal,
                    SourceRangeError::Range => MirCode::SourceMapRange,
                },
                function_id,
            )
        })?;
    Ok(SourceOrigin {
        normalized_path: captured.normalized_path,
        start: captured.start,
        end: captured.end,
    })
}

fn instructions_contain_copy(blocks: &[JsonValue]) -> bool {
    blocks_contain_instruction_kind(blocks, &["Copy"])
}

fn blocks_contain_instruction_kind(blocks: &[JsonValue], kinds: &[&str]) -> bool {
    blocks.iter().any(|block| {
        block
            .as_object()
            .and_then(|block| block.get("instructions"))
            .and_then(JsonValue::as_array)
            .is_some_and(|instructions| {
                instructions.iter().any(|instruction| {
                    instruction
                        .as_object()
                        .and_then(|instruction| instruction.get("kind"))
                        .and_then(JsonValue::as_str)
                        .is_some_and(|kind| kinds.contains(&kind))
                })
            })
    })
}

fn emitted_callees(value: &JsonValue, function_id: &str) -> Result<Vec<String>, MirError> {
    let blocks = value
        .as_object()
        .and_then(|function| function.get("blocks"))
        .and_then(JsonValue::as_array)
        .ok_or_else(|| MirError::new(MirCode::Call, function_id))?;
    let mut callees = BTreeSet::new();
    for block in blocks {
        let instructions = block
            .as_object()
            .and_then(|block| block.get("instructions"))
            .and_then(JsonValue::as_array)
            .ok_or_else(|| MirError::new(MirCode::Call, function_id))?;
        for instruction in instructions {
            let instruction = instruction
                .as_object()
                .ok_or_else(|| MirError::new(MirCode::Call, function_id))?;
            if instruction.get("kind").and_then(JsonValue::as_str) != Some("CallStatic") {
                continue;
            }
            let callee = instruction
                .get("function")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| MirError::new(MirCode::Call, function_id))?;
            callees.insert(callee.to_owned());
        }
    }
    Ok(callees.into_iter().collect())
}

fn blocks_contain_constant_reference(blocks: &[JsonValue]) -> bool {
    blocks.iter().any(contains_constant_reference)
}

fn contains_constant_reference(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(object) => {
            (object.len() == 1
                && object
                    .get("const")
                    .is_some_and(|value| value.as_str().is_some()))
                || object.values().any(contains_constant_reference)
        }
        JsonValue::Array(values) => values.iter().any(contains_constant_reference),
        _ => false,
    }
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}

fn domain_hash(domain: &[u8], payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(payload);
    hex(&hasher.finish())
}
