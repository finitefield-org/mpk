//! The object protocol validates capabilities separately from carrier types.
//! Its edges are the ordinary VIR control edges; there is no second CFG.
use super::*;
use crate::csharp_practical_vir_model::{
    object_construction_can_finalize, object_construction_constructor_member,
    object_construction_root, object_construction_signature,
    object_constructor_execution_signature, ObjectConstructionOperation as Op, ValidatedDataSource,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveObject {
    owner: String,
    value: String,
    must: u32,
    may: u32,
}
type Live = BTreeMap<String, LiveObject>;

/// Validate a private handoff before it is allowed to publish any source value.
/// Ordinary VIR validation remains required as well (types, dominance, checks,
/// exception routes and the complete source call graph).
pub fn validate_object_construction_protocol(
    function: &PracticalVirFunction,
    source: &ValidatedDataSource,
    foundation: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    operations: &BTreeMap<String, ClosedOperationSignature>,
) -> Result<(), PracticalVirImportError> {
    let callable = source
        .callables()
        .iter()
        .find(|c| c.id() == function.id)
        .ok_or_else(ownership_failure)?;
    let owner = callable.identity()["owner"]
        .as_str()
        .ok_or_else(ownership_failure)?;
    let constructor = callable.identity()["kind"] == "constructor"
        && object_construction_root(roots, owner).is_ok();
    let plans = callable
        .initialization_plans()
        .iter()
        .filter(|p| object_construction_root(roots, &p.type_id).is_ok())
        .collect::<Vec<_>>();
    let has_private = function
        .blocks
        .iter()
        .filter_map(|b| b.invocation.as_ref())
        .any(|i| {
            i.operation_id.starts_with("object.")
                || operations
                    .get(&i.operation_id)
                    .is_some_and(|s| s.tag == ClosedOperationTag::ConstructorExecute)
        });
    if !constructor && plans.is_empty() {
        return if function.object_protocol.is_none() && !has_private {
            Ok(())
        } else {
            Err(ownership_failure())
        };
    }
    let protocol = function
        .object_protocol
        .as_ref()
        .ok_or_else(ownership_failure)?;
    if protocol.constructor_owner.as_deref() != constructor.then_some(owner)
        || protocol
            .initializations
            .iter()
            .map(|i| i.source_node_ordinal)
            .ne(plans.iter().map(|p| p.node_ordinal))
    {
        return Err(ownership_failure());
    }
    let nodes = function
        .blocks
        .iter()
        .map(|b| (b.node.id.as_str(), b))
        .collect::<BTreeMap<_, _>>();
    if nodes.len() != function.blocks.len()
        || function.blocks.is_empty()
        || !function.loops.is_empty()
        || !function.exception_regions.is_empty()
    {
        return Err(ownership_failure());
    }
    let predecessors = control_predecessors(function, &nodes)?;
    let reachable = validate_control_reachability(function, &nodes)?;
    let dominators = compute_dominators(function, &predecessors, &reachable)?;
    let invocation = |id: &str| {
        nodes
            .get(id)
            .and_then(|b| b.invocation.as_ref())
            .ok_or_else(ownership_failure)
    };
    let mut bound_nodes = BTreeSet::new();
    let mut bound_origins = BTreeMap::new();
    for (record, plan) in protocol.initializations.iter().zip(&plans) {
        let begin = invocation(&record.begin_node_id)?;
        let call = invocation(&record.constructor_node_id)?;
        let finalize = invocation(&record.finalize_node_id)?;
        if begin.operation_id != format!("object.begin.{}", plan.type_id)
            || call.operation_id != plan.constructor_id
            || call.operands.first() != Some(&begin.result)
            || finalize.operation_id != format!("object.finalize.{}", plan.type_id)
        {
            return Err(ownership_failure());
        }
        let assignments = plan
            .steps
            .iter()
            .filter(|s| s.kind == "Assign")
            .collect::<Vec<_>>();
        if assignments.len() != record.assignment_node_ids.len() {
            return Err(ownership_failure());
        }
        for (node, assignment) in record.assignment_node_ids.iter().zip(assignments) {
            let operation = Op::from_id(roots, &invocation(node)?.operation_id)
                .map_err(|_| ownership_failure())?;
            if !matches!(operation,Op::Write {owner,ordinal,..} if owner==plan.type_id && plan.member_order.get(ordinal)==Some(&assignment.target))
            {
                return Err(ownership_failure());
            }
        }
        let ordered = std::iter::once(&record.begin_node_id)
            .chain(std::iter::once(&record.constructor_node_id))
            .chain(&record.assignment_node_ids)
            .chain(std::iter::once(&record.finalize_node_id))
            .collect::<Vec<_>>();
        for node in ordered.iter().skip(1) {
            bound_origins.insert(node.as_str(), begin.result.id.as_str());
        }
        for node in &ordered {
            if !bound_nodes.insert(node.as_str()) {
                return Err(ownership_failure());
            }
        }
        for pair in ordered.windows(2) {
            if !dominators
                .get(pair[1].as_str())
                .is_some_and(|d| d.contains(pair[0].as_str()))
            {
                return Err(ownership_failure());
            }
        }
    }
    let mut cleanup = BTreeMap::new();
    let mut previous = None;
    for row in &protocol.exceptional_discards {
        if previous.is_some_and(|p: &str| p >= row.exit_node_id.as_str())
            || nodes
                .get(row.exit_node_id.as_str())
                .is_none_or(|b| b.node.tag != ControlNodeTag::Exit)
            || row.origin_value_ids.is_empty()
            || row.origin_value_ids.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(ownership_failure());
        }
        previous = Some(row.exit_node_id.as_str());
        cleanup.insert(
            row.exit_node_id.as_str(),
            row.origin_value_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
        );
    }
    let mut initial = Live::new();
    let mut tainted = BTreeSet::new();
    if constructor {
        let signature = operations.get(&function.id).ok_or_else(ownership_failure)?;
        let mut expected =
            object_constructor_execution_signature(foundation, roots, closed, callable)
                .map_err(|_| ownership_failure())?;
        expected.ordered_checks = signature.ordered_checks.clone();
        if *signature != expected {
            return Err(ownership_failure());
        }
        let value = function
            .parameter_values
            .first()
            .ok_or_else(ownership_failure)?;
        if value.type_id != expected.normal_result_type_id {
            return Err(ownership_failure());
        }
        initial.insert(
            value.id.clone(),
            LiveObject {
                owner: owner.into(),
                value: value.id.clone(),
                must: 0,
                may: 0,
            },
        );
        tainted.insert(value.id.clone());
    }
    for block in &function.blocks {
        if let Some(i) = &block.invocation {
            let signature = operations
                .get(&i.operation_id)
                .ok_or_else(ownership_failure)?;
            if signature.tag == ClosedOperationTag::ConstructorExecute
                || matches!(
                    Op::from_id(roots, &i.operation_id),
                    Ok(Op::Begin { .. } | Op::Write { .. })
                )
            {
                tainted.insert(i.result.id.clone());
            }
        }
    }
    loop {
        let before = tainted.len();
        for phi in function.blocks.iter().flat_map(|b| &b.phi_values) {
            if phi.incoming.iter().any(|v| tainted.contains(&v.value_id)) {
                tainted.insert(phi.value.id.clone());
            }
        }
        if before == tainted.len() {
            break;
        }
    }
    let mut edges = BTreeMap::<(String, String), Live>::new();
    let mut processed = BTreeSet::new();
    let mut seen_origins = BTreeSet::new();
    let delegation = source
        .body(&function.id)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|p| p[0].kind() == "ConstructorInitializer")
        .map(|p| p[1].symbol().to_owned())
        .collect::<Vec<_>>();
    let mut used_delegation = false;
    let mut returned: Option<(u32, u32)> = None;
    while processed.len() != nodes.len() {
        let mut progress = false;
        for block in &function.blocks {
            let id = block.node.id.as_str();
            if processed.contains(id)
                || predecessors[id]
                    .iter()
                    .any(|p| !edges.contains_key(&(p.to_string(), id.into())))
            {
                continue;
            }
            let mut states = Vec::new();
            let mut discarded = BTreeSet::new();
            for predecessor in &predecessors[id] {
                let mut state = edges[&(predecessor.to_string(), id.into())].clone();
                if block.node.tag == ControlNodeTag::Exit {
                    if !state.is_empty()
                        && nodes[predecessor]
                            .node
                            .normal_successor_ids
                            .iter()
                            .any(|n| n == id)
                    {
                        return Err(ownership_failure());
                    }
                    discarded.extend(state.keys().cloned());
                    for origin in cleanup.get(id).into_iter().flatten() {
                        state.remove(origin);
                    }
                }
                for phi in &block.phi_values {
                    if !tainted.contains(&phi.value.id) {
                        continue;
                    }
                    let incoming = phi
                        .incoming
                        .iter()
                        .find(|v| v.predecessor_node_id == *predecessor)
                        .ok_or_else(ownership_failure)?;
                    let object = state
                        .values_mut()
                        .find(|v| v.value == incoming.value_id)
                        .ok_or_else(ownership_failure)?;
                    object.value = phi.value.id.clone();
                }
                states.push(state);
            }
            if block.node.tag == ControlNodeTag::Exit
                && cleanup.get(id).cloned().unwrap_or_default() != discarded
            {
                return Err(ownership_failure());
            }
            let mut live = if block.node.tag == ControlNodeTag::Entry {
                initial.clone()
            } else {
                states.first().cloned().unwrap_or_default()
            };
            for state in states.iter().skip(1) {
                if live.keys().ne(state.keys()) {
                    return Err(ownership_failure());
                }
                for (origin, incoming) in state {
                    let prior = live.get_mut(origin).ok_or_else(ownership_failure)?;
                    if prior.owner != incoming.owner || prior.value != incoming.value {
                        return Err(ownership_failure());
                    }
                    prior.must &= incoming.must;
                    prior.may |= incoming.may;
                }
            }
            let mut exceptional = live.clone();
            if let Some(i) = &block.invocation {
                let signature = operations
                    .get(&i.operation_id)
                    .ok_or_else(ownership_failure)?;
                if i.operation_id.starts_with("object.") {
                    let expected = object_construction_signature(roots, closed, &i.operation_id)
                        .map_err(|_| ownership_failure())?;
                    if *signature != expected
                        || i.operands
                            .iter()
                            .map(|v| &v.type_id)
                            .ne(expected.argument_type_ids.iter())
                        || i.result.type_id != expected.normal_result_type_id
                    {
                        return Err(ownership_failure());
                    }
                    match Op::from_id(roots, &i.operation_id).map_err(|_| ownership_failure())? {
                        Op::Begin { owner } => {
                            if !bound_nodes.contains(id)
                                || !seen_origins.insert(i.result.id.clone())
                            {
                                return Err(ownership_failure());
                            }
                            live.insert(
                                i.result.id.clone(),
                                LiveObject {
                                    owner,
                                    value: i.result.id.clone(),
                                    must: 0,
                                    may: 0,
                                },
                            );
                        }
                        operation => {
                            let receiver = i.operands.first().ok_or_else(ownership_failure)?;
                            let origin = live
                                .iter()
                                .find(|(_, v)| {
                                    v.value == receiver.id && v.owner == operation.owner()
                                })
                                .map(|(id, _)| id.clone())
                                .ok_or_else(ownership_failure)?;
                            if bound_origins
                                .get(id)
                                .is_some_and(|expected| *expected != origin)
                            {
                                return Err(ownership_failure());
                            }
                            if i.operands.iter().skip(1).any(|v| tainted.contains(&v.id)) {
                                return Err(ownership_failure());
                            }
                            let state = live.get_mut(&origin).ok_or_else(ownership_failure)?;
                            let own_receiver =
                                constructor && origin == function.parameter_values[0].id;
                            match operation {
                                Op::Read { ordinal, .. } => {
                                    if !own_receiver || state.must & (1u32 << ordinal) == 0 {
                                        return Err(ownership_failure());
                                    }
                                }
                                Op::Write { ordinal, .. } => {
                                    let bit = 1u32 << ordinal;
                                    if state.may & bit != 0
                                        || !own_receiver && !bound_nodes.contains(id)
                                    {
                                        return Err(ownership_failure());
                                    }
                                    if own_receiver {
                                        object_construction_constructor_member(
                                            roots,
                                            &state.owner,
                                            ordinal,
                                        )
                                        .map_err(|_| ownership_failure())?;
                                    }
                                    state.must |= bit;
                                    state.may |= bit;
                                    state.value = i.result.id.clone();
                                }
                                Op::Finalize { .. } => {
                                    if !bound_nodes.contains(id) {
                                        return Err(ownership_failure());
                                    }
                                    object_construction_can_finalize(
                                        foundation,
                                        roots,
                                        closed,
                                        &state.owner,
                                        state.must,
                                    )
                                    .map_err(|_| ownership_failure())?;
                                    live.remove(&origin);
                                }
                                Op::Begin { .. } => unreachable!(),
                            }
                        }
                    }
                } else if signature.tag == ClosedOperationTag::ConstructorExecute {
                    let callee = source
                        .callables()
                        .iter()
                        .find(|c| c.id() == i.operation_id)
                        .ok_or_else(ownership_failure)?;
                    let mut expected =
                        object_constructor_execution_signature(foundation, roots, closed, callee)
                            .map_err(|_| ownership_failure())?;
                    expected.ordered_checks = signature.ordered_checks.clone();
                    if *signature != expected
                        || i.operands
                            .iter()
                            .map(|v| &v.type_id)
                            .ne(expected.argument_type_ids.iter())
                        || i.result.type_id != expected.normal_result_type_id
                    {
                        return Err(ownership_failure());
                    }
                    let receiver = i.operands.first().ok_or_else(ownership_failure)?;
                    let callee_owner = callee.identity()["owner"]
                        .as_str()
                        .ok_or_else(ownership_failure)?;
                    let origin = live
                        .iter()
                        .find(|(_, v)| v.value == receiver.id && v.owner == callee_owner)
                        .map(|(id, _)| id.clone())
                        .ok_or_else(ownership_failure)?;
                    if bound_origins
                        .get(id)
                        .is_some_and(|expected| *expected != origin)
                    {
                        return Err(ownership_failure());
                    }
                    let state = live.get_mut(&origin).ok_or_else(ownership_failure)?;
                    if !bound_nodes.contains(id) {
                        if !constructor
                            || origin != function.parameter_values[0].id
                            || delegation != [i.operation_id.clone()]
                            || used_delegation
                        {
                            return Err(ownership_failure());
                        }
                        used_delegation = true;
                    }
                    if state.may != 0 || i.operands.iter().skip(1).any(|v| tainted.contains(&v.id))
                    {
                        return Err(ownership_failure());
                    }
                    let assignment = source
                        .constructor_assignment(callee.id())
                        .ok_or_else(ownership_failure)?;
                    state.must = assignment.definitely_assigned;
                    state.may = assignment.possibly_assigned;
                    state.value = i.result.id.clone();
                    // Ownership moves to the callee. On failure its exit
                    // disposes the capability; the caller cannot discard twice.
                    exceptional.remove(&origin);
                } else if i.operands.iter().any(|v| tainted.contains(&v.id)) {
                    return Err(ownership_failure());
                }
            }
            if block
                .condition_value_id
                .as_ref()
                .is_some_and(|v| tainted.contains(v))
                || block
                    .abrupt_value_id
                    .as_ref()
                    .is_some_and(|v| tainted.contains(v))
            {
                return Err(ownership_failure());
            }
            if block.node.tag == ControlNodeTag::Return {
                if constructor {
                    let origin = &function.parameter_values[0].id;
                    let value = live.remove(origin).ok_or_else(ownership_failure)?;
                    let expected = source
                        .constructor_assignment(&function.id)
                        .ok_or_else(ownership_failure)?;
                    if block.return_value_ids != [value.value]
                        || value.must & expected.definitely_assigned != expected.definitely_assigned
                        || value.may & !expected.possibly_assigned != 0
                    {
                        return Err(ownership_failure());
                    }
                    returned = Some(returned.map_or((value.must, value.may), |(must, may)| {
                        (must & value.must, may | value.may)
                    }));
                } else if block.return_value_ids.iter().any(|v| tainted.contains(v)) {
                    return Err(ownership_failure());
                }
                if !live.is_empty() {
                    return Err(ownership_failure());
                }
            }
            if block.node.tag == ControlNodeTag::Exit && !live.is_empty() {
                return Err(ownership_failure());
            }
            if block
                .construction_actions
                .iter()
                .filter_map(PracticalConstructionAction::used_value)
                .any(|v| tainted.contains(&v.id))
            {
                return Err(ownership_failure());
            }
            for target in &block.node.normal_successor_ids {
                if edges
                    .insert((id.into(), target.clone()), live.clone())
                    .is_some_and(|prior| prior != live)
                {
                    return Err(ownership_failure());
                }
            }
            for edge in &block.node.exceptional_successors {
                if edges
                    .insert((id.into(), edge.target_id.clone()), exceptional.clone())
                    .is_some_and(|prior| prior != exceptional)
                {
                    return Err(ownership_failure());
                }
            }
            processed.insert(id);
            progress = true;
        }
        if !progress {
            return Err(ownership_failure());
        }
    }
    if constructor {
        let summary = source
            .constructor_assignment(&function.id)
            .ok_or_else(ownership_failure)?;
        if returned != Some((summary.definitely_assigned, summary.possibly_assigned))
            || used_delegation != !delegation.is_empty()
        {
            return Err(ownership_failure());
        }
    }
    Ok(())
}
