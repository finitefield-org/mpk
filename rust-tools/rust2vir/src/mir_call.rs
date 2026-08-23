use super::hir_check::HirFunction;
use super::mir_lower::MirCode;
use super::type_lower::contract_type;
use rust2vir_internal::contract::{
    ContractSet, ContractType, NormalizedContract, RUST_SEMANTIC_PROFILE,
};
use rust2vir_internal::driver_protocol::DriverRequest;
use rust2vir_internal::json::JsonValue;
use rustc_hir::def::DefKind;
use rustc_index::Idx;
use rustc_middle::mir::{
    BasicBlock, Body, CallSource, Local, Operand, TerminatorKind, UnwindAction,
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::LocalDefId;
use rustc_span::Span;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectCallPatternVector {
    pub call_source_is_normal: bool,
    pub callee_is_constant_fn_def: bool,
    pub callee_is_local_free_function: bool,
    pub generic_arguments_are_empty: bool,
    pub callee_is_in_hir_closure: bool,
    pub callee_differs_from_caller: bool,
    pub hir_call_site_matches: bool,
    pub signature_matches_hir: bool,
    pub argument_modes_are_supported: bool,
    pub argument_types_match: bool,
    pub destination_is_plain: bool,
    pub destination_type_matches: bool,
    pub normal_target_is_present: bool,
    pub unwind_is_unreachable: bool,
    pub caller_contract_matches: bool,
    pub callee_contract_matches: bool,
    pub contract_hash_matches: bool,
    pub semantic_context_matches: bool,
    pub unit_identity_matches: bool,
}

impl DirectCallPatternVector {
    pub(crate) fn pinned() -> Self {
        Self {
            call_source_is_normal: true,
            callee_is_constant_fn_def: true,
            callee_is_local_free_function: true,
            generic_arguments_are_empty: true,
            callee_is_in_hir_closure: true,
            callee_differs_from_caller: true,
            hir_call_site_matches: true,
            signature_matches_hir: true,
            argument_modes_are_supported: true,
            argument_types_match: true,
            destination_is_plain: true,
            destination_type_matches: true,
            normal_target_is_present: true,
            unwind_is_unreachable: true,
            caller_contract_matches: true,
            callee_contract_matches: true,
            contract_hash_matches: true,
            semantic_context_matches: true,
            unit_identity_matches: true,
        }
    }
}

pub(crate) fn validate_direct_call_pattern(
    vector: &DirectCallPatternVector,
) -> Result<(), MirCode> {
    if !vector.unwind_is_unreachable {
        return Err(MirCode::Cleanup);
    }
    let mut structural = vector.clone();
    structural.contract_hash_matches = true;
    if structural != DirectCallPatternVector::pinned() {
        return Err(MirCode::Call);
    }
    vector
        .contract_hash_matches
        .then_some(())
        .ok_or(MirCode::SemanticsType)
}

#[derive(Clone)]
pub(super) struct PlannedCall<'tcx> {
    pub(super) callee: String,
    pub(super) contract_hash: String,
    pub(super) arguments: Vec<Operand<'tcx>>,
    pub(super) parameter_types: Vec<ContractType>,
    pub(super) destination: Local,
    pub(super) result_type: ContractType,
    pub(super) target: usize,
    pub(super) span: Span,
}

#[derive(Clone, Default)]
pub(super) struct CallPlan<'tcx> {
    calls: BTreeMap<usize, PlannedCall<'tcx>>,
    consumed_hir_call_sites: BTreeSet<usize>,
}

pub(super) struct CallContext<'a> {
    pub(super) caller: &'a HirFunction,
    pub(super) closure: &'a [HirFunction],
    pub(super) contracts: &'a ContractSet,
    pub(super) request: &'a DriverRequest,
}

impl<'tcx> CallPlan<'tcx> {
    pub(super) fn recognize(
        &mut self,
        tcx: TyCtxt<'tcx>,
        caller_def_id: LocalDefId,
        body: &Body<'tcx>,
        block_index: usize,
        context: &CallContext<'_>,
    ) -> Result<usize, MirCode> {
        if let Some(call) = self.calls.get(&block_index) {
            return Ok(call.target);
        }
        let block = &body.basic_blocks[BasicBlock::new(block_index)];
        let TerminatorKind::Call {
            func,
            args,
            destination,
            target,
            unwind,
            call_source,
            fn_span,
        } = &block.terminator().kind
        else {
            return Err(MirCode::Call);
        };
        let mut vector = DirectCallPatternVector::pinned();
        vector.call_source_is_normal = matches!(call_source, CallSource::Normal);
        vector.destination_is_plain =
            destination.projection.is_empty() && destination.local.index() < body.local_decls.len();
        vector.normal_target_is_present = target.is_some();
        vector.unwind_is_unreachable = matches!(unwind, UnwindAction::Unreachable);

        let (callee_def_id, generic_arguments) = match func {
            Operand::Constant(constant) => match constant.const_.ty().kind() {
                ty::FnDef(def_id, arguments) => (*def_id, *arguments),
                _ => {
                    vector.callee_is_constant_fn_def = false;
                    validate_direct_call_pattern(&vector)?;
                    unreachable!("non-function constant rejected")
                }
            },
            _ => {
                vector.callee_is_constant_fn_def = false;
                validate_direct_call_pattern(&vector)?;
                unreachable!("function-value call rejected")
            }
        };
        let callee_local = callee_def_id.as_local();
        vector.callee_is_local_free_function =
            callee_local.is_some() && matches!(tcx.def_kind(callee_def_id), DefKind::Fn);
        vector.generic_arguments_are_empty = generic_arguments.is_empty();
        let callee = callee_local.and_then(|def_id| {
            context
                .closure
                .iter()
                .find(|function| function.def_id == def_id)
        });
        vector.callee_is_in_hir_closure = callee.is_some();
        let Some(callee) = callee else {
            validate_direct_call_pattern(&vector)?;
            unreachable!("callee outside HIR closure rejected")
        };
        vector.callee_differs_from_caller = callee.def_id != context.caller.def_id;

        let call_site = context
            .caller
            .call_sites
            .iter()
            .enumerate()
            .filter(|(_, site)| {
                site.callee_def_id == callee.def_id
                    && site.callee_id == callee.function_id
                    && (span_contains(site.span, *fn_span)
                        || span_contains(site.span, block.terminator().source_info.span))
            })
            .min_by_key(|(_, site)| {
                (
                    site.span.hi().0 - site.span.lo().0,
                    site.span.lo().0,
                    site.span.hi().0,
                )
            });
        vector.hir_call_site_matches = call_site
            .as_ref()
            .is_some_and(|(index, _)| !self.consumed_hir_call_sites.contains(index));

        let Some(callee_local) = callee_local else {
            validate_direct_call_pattern(&vector)?;
            unreachable!("external callee rejected")
        };
        let signature = tcx
            .fn_sig(callee_def_id)
            .instantiate_identity()
            .skip_binder();
        let mir_parameter_types = signature
            .inputs()
            .iter()
            .map(|ty| contract_type(tcx, callee_local, *ty))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| MirCode::Call)?;
        let mir_result_type =
            contract_type(tcx, callee_local, signature.output()).map_err(|_| MirCode::Call)?;
        vector.signature_matches_hir =
            mir_parameter_types == callee.parameter_types && mir_result_type == callee.result_type;

        let mut arguments = Vec::with_capacity(args.len());
        let mut argument_types = Vec::with_capacity(args.len());
        for argument in args {
            let operand = &argument.node;
            let (supported, ty) = call_operand_type(tcx, caller_def_id, body, operand);
            vector.argument_modes_are_supported &= supported;
            if let Some(ty) = ty {
                argument_types.push(ty);
            }
            arguments.push(operand.clone());
        }
        vector.argument_types_match =
            argument_types.len() == arguments.len() && argument_types == callee.parameter_types;
        let destination_type = vector
            .destination_is_plain
            .then(|| contract_type(tcx, caller_def_id, body.local_decls[destination.local].ty))
            .transpose()
            .map_err(|_| MirCode::Call)?;
        vector.destination_type_matches = destination_type.as_ref() == Some(&callee.result_type);

        let caller_contract = context.contracts.get(&context.caller.function_id);
        let callee_contract = context.contracts.get(&callee.function_id);
        vector.caller_contract_matches = caller_contract.is_some_and(|contract| {
            contract_binding_matches(context.caller, contract, context.request)
        });
        vector.callee_contract_matches = callee_contract
            .is_some_and(|contract| contract_binding_matches(callee, contract, context.request));
        vector.contract_hash_matches = callee_contract.is_some_and(|contract| {
            contract
                .value
                .as_object()
                .and_then(|value| value.get("contract_hash"))
                .and_then(JsonValue::as_str)
                == Some(contract.contract_hash.as_str())
        });
        vector.semantic_context_matches = match (caller_contract, callee_contract) {
            (Some(caller), Some(callee)) => {
                contract_member(&caller.value, "semantic_profile")
                    == contract_member(&callee.value, "semantic_profile")
                    && caller
                        .value
                        .as_object()
                        .and_then(|value| value.get("semantic_parameters"))
                        == callee
                            .value
                            .as_object()
                            .and_then(|value| value.get("semantic_parameters"))
            }
            _ => false,
        };
        vector.unit_identity_matches = match (caller_contract, callee_contract) {
            (Some(caller), Some(callee)) => {
                contract_member(&caller.value, "unit_id") == Some(context.request.selection().1)
                    && contract_member(&callee.value, "unit_id")
                        == Some(context.request.selection().1)
            }
            _ => false,
        };
        validate_direct_call_pattern(&vector)?;

        let (call_site_index, call_site) = call_site.expect("validated HIR call site");
        if !self.consumed_hir_call_sites.insert(call_site_index) {
            return Err(MirCode::Call);
        }
        let call = PlannedCall {
            callee: callee.function_id.clone(),
            contract_hash: callee_contract
                .expect("validated callee contract")
                .contract_hash
                .clone(),
            arguments,
            parameter_types: callee.parameter_types.clone(),
            destination: destination.local,
            result_type: callee.result_type.clone(),
            target: target.expect("validated normal target").index(),
            span: call_site.span,
        };
        let target = call.target;
        if self.calls.insert(block_index, call).is_some() {
            return Err(MirCode::Call);
        }
        Ok(target)
    }

    pub(super) fn get(&self, block_index: usize) -> Option<&PlannedCall<'tcx>> {
        self.calls.get(&block_index)
    }
}

pub(super) fn validate_function_contract_context(
    function: &HirFunction,
    contract: &NormalizedContract,
    contracts: &ContractSet,
    request: &DriverRequest,
) -> Result<(), MirCode> {
    (contracts.get(&function.function_id) == Some(contract)
        && contract_binding_matches(function, contract, request))
    .then_some(())
    .ok_or(MirCode::Call)
}

pub(super) fn validate_function_mir_signature<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    body: &Body<'tcx>,
    function: &HirFunction,
) -> Result<(), MirCode> {
    if function.def_id != def_id
        || body.arg_count != function.parameter_types.len()
        || body.local_decls.len() < body.arg_count + 1
    {
        return Err(MirCode::Call);
    }
    let signature = tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    let signature_parameters = signature
        .inputs()
        .iter()
        .map(|ty| contract_type(tcx, def_id, *ty))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| MirCode::Call)?;
    let signature_result =
        contract_type(tcx, def_id, signature.output()).map_err(|_| MirCode::Call)?;
    let body_parameters = (0..body.arg_count)
        .map(|index| contract_type(tcx, def_id, body.local_decls[Local::new(index + 1)].ty))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| MirCode::Call)?;
    let body_result = contract_type(tcx, def_id, body.local_decls[Local::new(0)].ty)
        .map_err(|_| MirCode::Call)?;
    if signature_parameters != function.parameter_types
        || signature_result != function.result_type
        || body_parameters != function.parameter_types
        || body_result != function.result_type
    {
        return Err(MirCode::Call);
    }
    Ok(())
}

fn contract_binding_matches(
    function: &HirFunction,
    contract: &NormalizedContract,
    request: &DriverRequest,
) -> bool {
    let unit_id = request.selection().1;
    let expected_parameters = expected_semantic_parameters(request);
    contract.function_id == function.function_id
        && function
            .function_id
            .strip_prefix(unit_id)
            .is_some_and(|suffix| suffix.starts_with("::"))
        && contract_member(&contract.value, "unit_id") == Some(unit_id)
        && contract_member(&contract.value, "function_id") == Some(&function.function_id)
        && contract_member(&contract.value, "semantic_profile") == Some(RUST_SEMANTIC_PROFILE)
        && contract_member(&contract.value, "contract_hash") == Some(&contract.contract_hash)
        && contract
            .value
            .as_object()
            .and_then(|value| value.get("semantic_parameters"))
            == Some(&expected_parameters)
}

fn expected_semantic_parameters(request: &DriverRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
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
    ]))
}

fn contract_member<'a>(contract: &'a JsonValue, field: &str) -> Option<&'a str> {
    contract
        .as_object()
        .and_then(|value| value.get(field))
        .and_then(JsonValue::as_str)
}

fn call_operand_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    body: &Body<'tcx>,
    operand: &Operand<'tcx>,
) -> (bool, Option<ContractType>) {
    let typing_env = ty::TypingEnv::post_analysis(tcx, owner);
    let mir_type = match operand {
        Operand::Copy(place) if place.projection.is_empty() => body
            .local_decls
            .get(place.local)
            .map(|declaration| declaration.ty),
        Operand::Move(place) if place.projection.is_empty() => body
            .local_decls
            .get(place.local)
            .filter(|declaration| !declaration.ty.needs_drop(tcx, typing_env))
            .map(|declaration| declaration.ty),
        Operand::Constant(constant) => Some(constant.const_.ty()),
        _ => None,
    };
    let Some(mir_type) = mir_type else {
        return (false, None);
    };
    (true, contract_type(tcx, owner, mir_type).ok())
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.lo() < outer.hi()
        && inner.lo() < inner.hi()
        && outer.lo() <= inner.lo()
        && inner.hi() <= outer.hi()
}
