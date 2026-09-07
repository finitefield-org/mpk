//! Source-bound control handoff validation shared by emission and import.
use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExceptionDeclaration {
    type_id: String,
    sealed_type: bool,
    direct_base_type_id: String,
    payload_member_names: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    facts: Value,
    normalized_syntax_sha256: String,
    normalized_syntax_utf8: String,
    sequence_handoff: Value,
    functions: Vec<LoopControlFunction>,
    handlers: Vec<HandlerFunction>,
    total_getters: Vec<String>,
    exception_definitions: Vec<ExceptionDeclaration>,
}
#[derive(Clone, Debug)]
pub struct ValidatedControlSource {
    functions: Vec<LoopControlFunction>,
    handlers: Vec<PreparedHandlerFunction>,
    definitions: Vec<SourceExceptionDefinition>,
    universe: ClosedExceptionUniverse,
    facts: Value,
    total_getters: Vec<String>,
}
impl ValidatedControlSource {
    pub fn functions(&self) -> &[LoopControlFunction] {
        &self.functions
    }
    pub fn handlers(&self) -> &[PreparedHandlerFunction] {
        &self.handlers
    }
    pub fn definitions(&self) -> &[SourceExceptionDefinition] {
        &self.definitions
    }
    pub fn universe(&self) -> &ClosedExceptionUniverse {
        &self.universe
    }
    pub fn facts(&self) -> &Value {
        &self.facts
    }
    pub fn total_getters(&self) -> &[String] {
        &self.total_getters
    }
}

pub fn validate_control_source(
    bundle: &ValidatedFoundationBundle,
    source: &ValidatedDataSource,
) -> Result<ValidatedControlSource, DataPhaseError> {
    let fail = DataPhaseError::Source;
    let value = source.control_lowering().ok_or(fail.clone())?;
    let wire: Wire = serde_json::from_value(value.clone()).map_err(|_| fail.clone())?;
    if wire.schema != "mpk.csharp_practical.t04_w06.control_lowering.v1"
        || wire.normalized_syntax_utf8.len() > 32 * 1024 * 1024
        || wire.functions.len() > 128
        || wire.exception_definitions.len() > 32
        || wire.functions.iter().map(|f| f.nodes.len()).sum::<usize>() > 8192
        || wire.total_getters.windows(2).any(|w| w[0] >= w[1])
        || sha256_raw_file_bytes(wire.normalized_syntax_utf8.as_bytes()).to_hex()
            != wire.normalized_syntax_sha256
        || wire.sequence_handoff["source"] != wire.normalized_syntax_sha256
    {
        return Err(fail);
    }
    let captured: Value =
        serde_json::from_slice(source.captured_facts()).map_err(|_| fail.clone())?;
    if wire.facts["compilation_id"] != captured["compilation_id"]
        || wire.facts["selected_root_ids"] != captured["selected_root_ids"]
    {
        return Err(fail);
    }
    let expected_sources = captured["sources"]
        .as_array()
        .ok_or(fail.clone())?
        .iter()
        .map(|s| json!({"path":s["path"],"raw_sha256":s["raw_sha256"]}))
        .collect::<Vec<_>>();
    if wire.facts["sources"] != json!(expected_sources) {
        return Err(fail);
    }
    let syntax: Value =
        serde_json::from_str(&wire.normalized_syntax_utf8).map_err(|_| fail.clone())?;
    let callables = syntax["callables"].as_array().ok_or(fail.clone())?;
    let methods = wire.facts["methods"].as_array().ok_or(fail.clone())?;
    let ids = wire
        .functions
        .iter()
        .map(|f| f.callable_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != wire.functions.len()
        || wire.functions.len() != wire.handlers.len()
        || ids
            != methods
                .iter()
                .map(|m| m["callable_id"].as_str().ok_or(fail.clone()))
                .collect::<Result<BTreeSet<_>, _>>()?
        || wire.total_getters.iter().any(|id| {
            !source
                .callables()
                .iter()
                .any(|c| c.id() == id && c.is_property_getter())
        })
    {
        return Err(fail);
    }
    let roots = source.source_roots();
    let closed = derive_closed_instances(bundle, roots).map_err(|_| fail.clone())?;
    let definitions = wire
        .exception_definitions
        .iter()
        .map(|d| {
            let ty = roots.source_types.get(&d.type_id).ok_or(fail.clone())?;
            if ty
                .members
                .iter()
                .map(|m| m.name.as_str())
                .ne(d.payload_member_names.iter().map(String::as_str))
            {
                return Err(fail.clone());
            }
            Ok(SourceExceptionDefinition {
                type_id: d.type_id.clone(),
                sealed: d.sealed_type,
                direct_base_type_id: d.direct_base_type_id.clone(),
                payload_member_ids: ty.members.iter().map(|m| m.id.clone()).collect(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let universe =
        derive_closed_exception_universe(roots, &closed, &definitions).map_err(|_| fail.clone())?;
    let mut handlers = Vec::new();
    for (function, graph) in wire.functions.iter().zip(wire.handlers) {
        let callable = source
            .callables()
            .iter()
            .find(|c| c.id() == function.callable_id)
            .ok_or(fail.clone())?;
        let normalized = callables
            .iter()
            .find(|c| c["id"] == function.callable_id)
            .ok_or(fail.clone())?;
        let method = methods
            .iter()
            .find(|m| m["callable_id"] == function.callable_id)
            .ok_or(fail.clone())?;
        let locations = function.operation_locations.as_ref().ok_or(fail.clone())?;
        let source_file = captured["input_files"]
            .as_array()
            .ok_or(fail.clone())?
            .iter()
            .find(|file| file["kind"] == "source" && file["path"] == method["source_path"])
            .ok_or(fail.clone())?;
        let source_size = source_file["size_bytes"].as_u64().ok_or(fail.clone())?;
        if locations.len() != function.operations.len()
            || locations
                .iter()
                .any(|l| l.start_byte > l.end_byte || l.end_byte as u64 > source_size)
        {
            return Err(fail);
        }
        let mut parameters = Vec::new();
        if !callable.is_static() {
            parameters.push(json!({"id":"this", "type_id":callable.identity()["owner"]}));
        }
        for (ordinal, ty) in callable.identity()["parameter_type_ids"]
            .as_array()
            .ok_or(fail.clone())?
            .iter()
            .enumerate()
        {
            parameters.push(json!({"id":format!("parameter:{ordinal}"), "type_id":ty}));
        }
        let result_type = if callable.identity()["kind"] == "constructor" {
            json!("mpk.csharp.value.unit.v1")
        } else {
            callable.identity()["result_type_id"].clone()
        };
        if method["parameters"] != json!(parameters) || method["result_type_id"] != result_type {
            return Err(fail);
        }
        if normalized["body_sha256"] != callable.body_sha256()
            || normalized["body"].as_str().is_none_or(|body| {
                sha256_raw_file_bytes(body.as_bytes()).to_hex() != callable.body_sha256()
            })
            || serde_json::from_str::<Vec<LoopSourceOperation>>(
                normalized["body"].as_str().ok_or(fail.clone())?,
            )
            .map_err(|_| fail.clone())?
                != function.operations
        {
            return Err(fail);
        }
        let prepared = prepare_handler_function(function, graph, &universe)
            .map_err(DataPhaseError::ControlHandler)?;
        let graph = handler_control_graph(function, &prepared, &universe)?;
        loop_lowering::validate_function(&graph, true, Some(&universe), true)
            .map_err(DataPhaseError::ControlGraph)?;
        let parameters = parameters
            .iter()
            .map(|p| {
                let slot = p["id"].as_str().expect("derived parameter");
                (
                    slot.to_owned(),
                    format!("{}.parameter.{slot}", function.callable_id),
                )
            })
            .collect();
        let caught = prepared
            .graph()
            .regions
            .iter()
            .flat_map(|r| &r.catches)
            .filter(|c| !c.local.is_empty())
            .flat_map(|c| {
                std::iter::once(&c.entry)
                    .chain(c.filter.as_ref())
                    .map(|id| (id.clone(), (c.local.clone(), format!("{id}.exception"))))
            })
            .collect();
        derive_control_ssa(&graph, &parameters, &caught).map_err(DataPhaseError::ControlGraph)?;
        handlers.push(prepared);
    }
    Ok(ValidatedControlSource {
        functions: wire.functions,
        handlers,
        definitions,
        universe,
        facts: wire.facts,
        total_getters: wire.total_getters,
    })
}

/// Project the continuation table onto feasible intra-function CFG edges.
/// The staged resume list is an inventory, not an edge from every finally to
/// every catch. Search still retains the ordered protocol for execution.
pub fn handler_control_graph(
    function: &LoopControlFunction,
    handler: &PreparedHandlerFunction,
    universe: &ClosedExceptionUniverse,
) -> Result<LoopControlFunction, DataPhaseError> {
    let graph = handler.graph();
    let mut result = function.clone();
    let mut resumptions = BTreeMap::<String, BTreeSet<String>>::new();
    let mut filter_true = BTreeMap::<String, BTreeSet<String>>::new();
    let mut filter_false = BTreeMap::<String, BTreeSet<String>>::new();
    let first = |finalies: &[String], target: Option<&str>| -> Option<String> {
        finalies
            .first()
            .cloned()
            .or_else(|| target.map(str::to_owned))
    };
    let mut suffix = |finalies: &[String], target: Option<&str>| {
        for (i, entry) in finalies.iter().enumerate() {
            if let Some(next) = first(&finalies[i + 1..], target) {
                resumptions.entry(entry.clone()).or_default().insert(next);
            }
        }
    };
    for transfer in &graph.transfers {
        suffix(&transfer.finally_entries, transfer.target.as_deref());
        for candidate in &transfer.candidates {
            suffix(&candidate.finally_entries, Some(&candidate.entry));
            if candidate.filter.is_some() {
                filter_true
                    .entry(candidate.catch_id.clone())
                    .or_default()
                    .extend(first(&candidate.finally_entries, Some(&candidate.entry)));
            }
        }
        for (index, candidate) in transfer
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.filter.is_some())
        {
            // Closed catch ancestry makes the finite search subjects explicit.
            for arm in universe.arms() {
                if !universe.catch_is_ancestor(&candidate.type_id, &arm.type_id) {
                    continue;
                }
                let destination = transfer.candidates[index + 1..]
                    .iter()
                    .find(|c| universe.catch_is_ancestor(&c.type_id, &arm.type_id))
                    .and_then(|c| {
                        c.filter
                            .clone()
                            .or_else(|| first(&c.finally_entries, Some(&c.entry)))
                    })
                    .or_else(|| first(&transfer.finally_entries, None));
                filter_false
                    .entry(candidate.catch_id.clone())
                    .or_default()
                    .extend(destination);
            }
        }
    }
    for node in &mut result.nodes {
        match node.kind.as_str() {
            "handler_resume" => {
                let context = graph
                    .contexts
                    .iter()
                    .find(|c| c.node == node.id)
                    .ok_or(DataPhaseError::Source)?;
                let frame = context
                    .frames
                    .last()
                    .filter(|f| f.zone == "finally")
                    .ok_or(DataPhaseError::Source)?;
                let entry = graph
                    .regions
                    .iter()
                    .find(|r| r.id == frame.region)
                    .and_then(|r| r.finally_entry.as_ref())
                    .ok_or(DataPhaseError::Source)?;
                node.successors = resumptions
                    .get(entry)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .collect();
            }
            "handler_filter_result" => {
                node.successors = filter_true
                    .get(&node.slot)
                    .into_iter()
                    .flatten()
                    .chain(filter_false.get(&node.slot).into_iter().flatten())
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            }
            "handler_search" => {
                let transfer = graph
                    .transfers
                    .iter()
                    .find(|t| t.node == node.id)
                    .ok_or(DataPhaseError::Source)?;
                if let Some(catch) = &transfer.filter_catch {
                    node.successors = filter_false
                        .get(catch)
                        .into_iter()
                        .flatten()
                        .cloned()
                        .collect();
                }
            }
            _ => {}
        }
    }
    Ok(result)
}

/// Propagate termination claims over the captured, finite source-call closure.
/// A claim remains a proof obligation; this does not discharge any decreases.
/// Missing claims inherit partialness from callees, while explicit total claims
/// may not hide a partial descendant behind an intermediate helper.
pub fn derive_control_termination(
    source: &ValidatedDataSource,
    claims: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, DataPhaseError> {
    let ids = source
        .callables()
        .iter()
        .map(|c| c.id().to_owned())
        .collect::<BTreeSet<_>>();
    if claims
        .iter()
        .any(|(id, mode)| !ids.contains(id) || !matches!(mode.as_str(), "total" | "partial"))
    {
        return Err(DataPhaseError::Contract);
    }
    let getters = source
        .callables()
        .iter()
        .filter(|c| c.is_property_getter())
        .map(|c| {
            let name = c.identity()["name"]
                .as_str()
                .and_then(|n| n.strip_prefix("get_"))
                .ok_or(DataPhaseError::Source)?;
            let owner = c.identity()["owner"]
                .as_str()
                .ok_or(DataPhaseError::Source)?;
            Ok((format!("{owner}.{name}"), c.id().to_owned()))
        })
        .collect::<Result<BTreeMap<_, _>, DataPhaseError>>()?;
    let mut calls = BTreeMap::new();
    for id in &ids {
        let mut targets = BTreeSet::new();
        for operation in source.body(id).ok_or(DataPhaseError::Source)? {
            if ids.contains(operation.symbol())
                && matches!(operation.kind(), "Invocation" | "ObjectCreation")
            {
                targets.insert(operation.symbol().to_owned());
            } else if operation.kind() == "PropertyReference" {
                targets.extend(getters.get(operation.symbol()).cloned());
            }
        }
        calls.insert(id.clone(), targets);
    }
    let mut partial = claims
        .iter()
        .filter(|(_, mode)| *mode == "partial")
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    loop {
        let mut changed = false;
        for (id, callees) in &calls {
            if callees.iter().any(|callee| partial.contains(callee)) {
                changed |= partial.insert(id.clone());
            }
        }
        if !changed {
            break;
        }
    }
    if claims
        .iter()
        .any(|(id, mode)| mode == "total" && partial.contains(id))
    {
        return Err(DataPhaseError::Contract);
    }
    Ok(ids
        .into_iter()
        .map(|id| {
            let mode = if partial.contains(&id) {
                "partial"
            } else {
                "total"
            };
            (id, mode.to_owned())
        })
        .collect())
}
