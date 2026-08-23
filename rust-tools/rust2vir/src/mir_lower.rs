use super::hir_check::{contract_type, HirFunction};
use super::mir_arithmetic::{ArithmeticOperation, ArithmeticPlan, DivRemOperation, ShiftOperation};
use rust2vir_internal::contract::ContractType;
use rust2vir_internal::driver_protocol::DriverRequest;
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceRangeError};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::mir_access::MIR_PROFILE_ID;
use rust2vir_internal::sha256::{hex, Sha256};
use rust2vir_internal::source_map::{raw_source_map, SourceMapEntry, SourceOrigin, VirReference};
use rust2vir_internal::stable_id::{block_names, breadth_first_order, DenseIds, StableIdError};
use rustc_index::Idx;
use rustc_middle::mir::{
    BasicBlock, BinOp, Body, Local, Operand, Place, Rvalue, StatementKind, TerminatorKind, UnOp,
    VarDebugInfoContents,
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::LocalDefId;
use rustc_span::{FileName, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const VIR_HASH_DOMAIN: &[u8] = b"MPK-VIR-0.1";
const MIR_BLOCKS_FUNCTION_MAX: usize = 1_024;
const MIR_BLOCKS_CLOSURE_MAX: usize = 8_192;
const MIR_STATEMENTS_FUNCTION_MAX: usize = 100_000;
const MIR_STATEMENTS_CLOSURE_MAX: usize = 250_000;
const VIR_PARAMETERS_MAX: usize = 256;
const VIR_LOCALS_MAX: usize = 65_536;
const VIR_BLOCK_PARAMETERS_MAX: usize = 4_096;
const VIR_INSTRUCTIONS_FUNCTION_MAX: usize = 100_000;
const VIR_INSTRUCTIONS_CLOSURE_MAX: usize = 250_000;
const VIR_CANONICAL_BYTES_MAX: usize = 192 * 1_024 * 1_024;
const SOURCE_MAP_CANONICAL_BYTES_MAX: usize = 32 * 1_024 * 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MirCode {
    Statement,
    Rvalue,
    Operand,
    Place,
    Projection,
    Terminator,
    Assertion,
    Call,
    Move,
    Cleanup,
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
            Self::Call => "RUST_MIR_CALL",
            Self::Move => "RUST_MIR_MOVE",
            Self::Cleanup => "RUST_MIR_CLEANUP",
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
            Self::Call => "MIR call is not implemented by the basic lowerer",
            Self::Move => "projected or dropping MIR move is not permitted",
            Self::Cleanup => "MIR cleanup, drop, or unwind flow is not permitted",
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

#[derive(Clone, Copy)]
enum Flow {
    Goto(usize),
    Branch {
        false_block: usize,
        true_block: usize,
    },
    Return,
}

type Reachability = (Vec<usize>, Vec<Option<Flow>>, ArithmeticPlan);

impl Flow {
    fn successors(self) -> Vec<usize> {
        match self {
            Self::Goto(target) => vec![target],
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
) -> Result<MirLowering, MirError> {
    let mut block_total = 0_usize;
    let mut statement_total = 0_usize;
    let mut instruction_total = 0_usize;
    for item in &lowered {
        block_total = block_total
            .checked_add(item.reachable_blocks)
            .ok_or_else(|| MirError::new(MirCode::BlockLimit, &item.function_id))?;
        statement_total = statement_total
            .checked_add(item.reachable_statements)
            .ok_or_else(|| MirError::new(MirCode::StatementLimit, &item.function_id))?;
        instruction_total = instruction_total
            .checked_add(item.instructions)
            .ok_or_else(|| MirError::new(MirCode::IrLimit, &item.function_id))?;
        if block_total > MIR_BLOCKS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::BlockLimit, &item.function_id));
        }
        if statement_total > MIR_STATEMENTS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::StatementLimit, &item.function_id));
        }
        if instruction_total > VIR_INSTRUCTIONS_CLOSURE_MAX {
            return Err(MirError::new(MirCode::IrLimit, &item.function_id));
        }
    }
    lowered.sort_by(|left, right| left.function_id.cmp(&right.function_id));

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
    let functions_json = lowered.iter().map(|item| item.value.clone()).collect();
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
                ("type_decls".to_owned(), JsonValue::Array(Vec::new())),
                ("const_decls".to_owned(), JsonValue::Array(Vec::new())),
                ("functions".to_owned(), JsonValue::Array(functions_json)),
            ]))]),
        ),
    ]));
    let vir_preimage =
        json::canonical(&vir).map_err(|_| MirError::new(MirCode::Rvalue, request.selection().2))?;
    let vir_hash = domain_hash(VIR_HASH_DOMAIN, &vir_preimage);
    vir.as_object_mut()
        .expect("constructed VIR object")
        .insert("vir_hash".to_owned(), string(&vir_hash));
    if json::canonical(&vir)
        .map_err(|_| MirError::new(MirCode::IrLimit, request.selection().2))?
        .len()
        > VIR_CANONICAL_BYTES_MAX
    {
        return Err(MirError::new(MirCode::IrLimit, request.selection().2));
    }

    let mut entries = Vec::new();
    for item in lowered {
        entries.extend(item.source_map);
    }
    let source_map = raw_source_map(&vir_hash, entries);
    if json::canonical(&source_map)
        .map_err(|_| MirError::new(MirCode::IrLimit, request.selection().2))?
        .len()
        > SOURCE_MAP_CANONICAL_BYTES_MAX
    {
        return Err(MirError::new(MirCode::IrLimit, request.selection().2));
    }
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
    contract: &JsonValue,
    loader: &SnapshotFileLoader,
) -> Result<LoweredFunction, MirError> {
    let function_id = function.function_id.as_str();
    if body.arg_count != function.parameter_types.len()
        || function.local_names.len() != function.local_types.len()
        || function.local_names.len() != function.local_spans.len()
        || function.parameter_types.len() > VIR_PARAMETERS_MAX
        || function.local_types.len() > VIR_LOCALS_MAX
        || !is_scalar(&function.result_type)
        || function.parameter_types.iter().any(|ty| !is_scalar(ty))
        || function.local_types.iter().any(|ty| !is_scalar(ty))
    {
        return Err(MirError::new(MirCode::Rvalue, function_id));
    }
    let local_kinds = map_locals(body, function)?;
    let (order, flows, mut arithmetic) = reachable_order(tcx, def_id, body, function, function_id)?;
    arithmetic
        .finish(body, &order)
        .map_err(|code| MirError::new(code, function_id))?;
    if order.len() > MIR_BLOCKS_FUNCTION_MAX {
        return Err(MirError::new(MirCode::BlockLimit, function_id));
    }
    let statement_count = order
        .iter()
        .try_fold(0_usize, |count, block| {
            count.checked_add(body.basic_blocks[BasicBlock::new(*block)].statements.len())
        })
        .ok_or_else(|| MirError::new(MirCode::StatementLimit, function_id))?;
    if statement_count > MIR_STATEMENTS_FUNCTION_MAX {
        return Err(MirError::new(MirCode::StatementLimit, function_id));
    }
    validate_storage(body, &order, &flows, &local_kinds, function_id)?;
    let (live_in, uses, definitions) = live_compiler_locals(
        tcx,
        def_id,
        body,
        &order,
        &flows,
        &local_kinds,
        &arithmetic,
        function_id,
    )?;
    let _ = (uses, definitions);
    if !live_in[0].is_empty() {
        return Err(MirError::new(MirCode::Operand, function_id));
    }

    let block_ids = block_names(&order);
    let incoming_origins = compiler_local_origins(body, &order, &flows, &local_kinds, function_id)?;
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
        origin: source_origin(tcx, loader, body.span, function_id)?,
    }];
    let mut blocks = Vec::with_capacity(order.len());
    let mut next_instruction = 0_usize;
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
                        function,
                        &local_kinds,
                        &function.local_spans,
                        &mut initialized_user_locals,
                        &mut environment,
                        &mut ids,
                        &mut next_instruction,
                        block_index,
                        unit_id,
                        function_id,
                        loader,
                        &mut instructions,
                        &mut source_entries,
                    )?;
                }
                StatementKind::StorageLive(_) | StatementKind::StorageDead(_) => {}
                StatementKind::Nop => {
                    if !span_contains(body.span, statement.source_info.span) {
                        return Err(MirError::new(MirCode::SourceMapExternal, function_id));
                    }
                    let _ = source_origin(tcx, loader, statement.source_info.span, function_id)?;
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
            Flow::Return => fallback_terminator_span,
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
            origin: source_origin(tcx, loader, terminator_span, function_id)?,
        });
        blocks.push(JsonValue::Object(BTreeMap::from([
            ("label".to_owned(), string(&block_ids[old_block])),
            ("parameters".to_owned(), JsonValue::Array(parameters)),
            ("instructions".to_owned(), JsonValue::Array(instructions)),
            ("terminator".to_owned(), terminator_json),
        ])));
        if next_instruction > VIR_INSTRUCTIONS_FUNCTION_MAX {
            return Err(MirError::new(MirCode::IrLimit, function_id));
        }
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
    if flows.iter().flatten().any(|flow| matches!(flow, Flow::Branch { false_block, true_block } if false_block != true_block)) {
        features.push(string("branch"));
    }
    if !function.local_types.is_empty() || instructions_contain_copy(&blocks) {
        features.push(string("mutable_local"));
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
            ("contracts".to_owned(), contract.clone()),
            ("features_used".to_owned(), JsonValue::Array(features)),
        ])),
        source_map: source_entries,
        reachable_blocks: order.len(),
        reachable_statements: statement_count,
        instructions: next_instruction,
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
    function_id: &str,
) -> Result<Reachability, MirError> {
    if body.basic_blocks.is_empty() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    let mut flows = vec![None; body.basic_blocks.len()];
    let mut arithmetic = ArithmeticPlan::default();
    let mut failure = None;
    let order = breadth_first_order(0, body.basic_blocks.len(), |index| {
        if failure.is_some() {
            return Vec::new();
        }
        let block = &body.basic_blocks[BasicBlock::new(index)];
        if block.is_cleanup {
            failure = Some(MirError::new(MirCode::Cleanup, function_id));
            return Vec::new();
        }
        match classify_flow(
            tcx,
            def_id,
            body,
            index,
            function,
            &mut arithmetic,
            function_id,
        ) {
            Ok(flow) => {
                flows[index] = Some(flow);
                flow.successors()
            }
            Err(error) => {
                failure = Some(error);
                Vec::new()
            }
        }
    })
    .map_err(|error| match error {
        StableIdError::EmptyGraph | StableIdError::UnknownSuccessor => {
            MirError::new(MirCode::Terminator, function_id)
        }
    })?;
    if let Some(error) = failure {
        return Err(error);
    }
    if topological_order(&order, &flows).is_none() {
        return Err(MirError::new(MirCode::Terminator, function_id));
    }
    Ok((order, flows, arithmetic))
}

fn classify_flow<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    block_index: usize,
    function: &HirFunction,
    arithmetic: &mut ArithmeticPlan,
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
        TerminatorKind::Assert { .. } => arithmetic
            .recognize_assert(tcx, def_id, body, block_index, function)
            .map(Flow::Goto)
            .map_err(|code| MirError::new(code, function_id)),
        TerminatorKind::Call { unwind, .. }
            if !matches!(unwind, rustc_middle::mir::UnwindAction::Unreachable) =>
        {
            Err(MirError::new(MirCode::Cleanup, function_id))
        }
        TerminatorKind::Call { .. } => Err(MirError::new(MirCode::Call, function_id)),
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

fn compiler_local_origins(
    body: &Body<'_>,
    order: &[usize],
    flows: &[Option<Flow>],
    local_kinds: &[LocalKind],
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
                    if arithmetic.is_negation_guard(*block_index, statement_index)
                        || arithmetic.is_div_rem_guard(*block_index, statement_index)
                        || arithmetic.is_shift_guard(*block_index, statement_index)
                    {
                        // The guard is represented by the attached VIR safety check.
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
        if let TerminatorKind::SwitchInt { discr, .. } = &block.terminator().kind {
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

fn validate_storage(
    body: &Body<'_>,
    order: &[usize],
    flows: &[Option<Flow>],
    local_kinds: &[LocalKind],
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
        if let TerminatorKind::SwitchInt { discr, .. } = &block.terminator().kind {
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
        Operand::Constant(constant) => {
            literal_value(tcx, def_id, &constant.const_, function_id).map(drop)
        }
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
    function: &HirFunction,
    local_kinds: &[LocalKind],
    local_binding_spans: &[Span],
    initialized_user_locals: &mut BTreeSet<usize>,
    environment: &mut BTreeMap<usize, Value>,
    ids: &mut DenseIds,
    next_instruction: &mut usize,
    block_index: usize,
    unit_id: &str,
    function_id: &str,
    loader: &SnapshotFileLoader,
    instructions: &mut Vec<JsonValue>,
    source_entries: &mut Vec<SourceMapEntry>,
) -> Result<(), MirError> {
    validate_destination(destination, local_kinds, function_id)?;
    if arithmetic.is_negation_guard(mir_block, statement_index)
        || arithmetic.is_div_rem_guard(mir_block, statement_index)
        || arithmetic.is_shift_guard(mir_block, statement_index)
    {
        return Ok(());
    }
    if is_erasable_unit_assignment(body, destination, rvalue, local_kinds) {
        return Ok(());
    }
    let mut value = if let Some((operation, ty)) = arithmetic.binary(mir_block, statement_index) {
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
                    *next_instruction - 1,
                )?
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
                    *next_instruction - 1,
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
                    *next_instruction - 1,
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
                    *next_instruction - 1,
                )?
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
    match local_kinds[destination.local.index()] {
        LocalKind::Result | LocalKind::Temporary => {
            value.span = statement_span;
            environment.insert(destination.local.index(), value);
        }
        LocalKind::User(local_index) => {
            let copy_span = if initialized_user_locals.insert(local_index) {
                *local_binding_spans
                    .get(local_index)
                    .ok_or_else(|| MirError::new(MirCode::SourceMapRange, function_id))?
            } else {
                statement_span
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
                *next_instruction - 1,
            )?;
        }
        LocalKind::Argument(_) => return Err(MirError::new(MirCode::Place, function_id)),
    }
    Ok(())
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
    next_instruction: &mut usize,
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
        *next_instruction - 1,
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
    next_instruction: &mut usize,
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
        *next_instruction - 1,
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
    next_instruction: &mut usize,
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
        *next_instruction - 1,
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
    next_instruction: &mut usize,
    kind: &str,
    ty: &ContractType,
    function_id: &str,
) -> Result<BTreeMap<String, JsonValue>, MirError> {
    if *next_instruction >= VIR_INSTRUCTIONS_FUNCTION_MAX {
        return Err(MirError::new(MirCode::IrLimit, function_id));
    }
    let id = ids.temporary();
    debug_assert_eq!(id, format!("t{}", *next_instruction));
    *next_instruction += 1;
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
        Operand::Constant(constant) => literal_value(tcx, def_id, &constant.const_, function_id),
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

fn vir_type(ty: &ContractType) -> JsonValue {
    match ty {
        ContractType::Bool => {
            JsonValue::Object(BTreeMap::from([("kind".to_owned(), string("bool"))]))
        }
        ContractType::BitVector { width, signed } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), string("bv")),
            ("width".to_owned(), JsonValue::Number(width.to_string())),
            ("signed".to_owned(), JsonValue::Bool(*signed)),
        ])),
        ContractType::Array { element, length } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), string("array")),
            ("length".to_owned(), JsonValue::Number(length.to_string())),
            ("element".to_owned(), vir_type(element)),
        ])),
        ContractType::Struct { id } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), string("struct")),
            ("id".to_owned(), string(id)),
        ])),
    }
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
                        == Some("Copy")
                })
            })
    })
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
