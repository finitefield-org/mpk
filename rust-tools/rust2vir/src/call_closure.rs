use std::collections::{BTreeMap, BTreeSet};

use crate::limits::RustLimitId;

pub const MAX_CALL_CLOSURE_FUNCTIONS: usize = RustLimitId::ClosureFunctions.maximum() as usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallClosureError {
    DuplicateFunction,
    UnknownRoot,
    UnknownCallee,
    Cycle,
    Limit,
}

/// Resolves the exact graph reachable from `root` in deterministic function-ID order.
///
/// The input graph is expected to come from compiler-resolved HIR. Keeping this graph
/// operation separate makes cycle and limit handling independent of MIR reachability or DCE.
pub fn resolve_call_closure<I, S>(
    functions: I,
    root: &str,
    maximum: usize,
) -> Result<Vec<String>, CallClosureError>
where
    I: IntoIterator<Item = (S, Vec<S>)>,
    S: Into<String>,
{
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for (function, callees) in functions {
        let function = function.into();
        let callees = callees.into_iter().map(Into::into).collect();
        if graph.insert(function, callees).is_some() {
            return Err(CallClosureError::DuplicateFunction);
        }
    }
    if !graph.contains_key(root) {
        return Err(CallClosureError::UnknownRoot);
    }

    let mut reachable = BTreeSet::new();
    let mut pending = vec![root.to_owned()];
    while let Some(function) = pending.pop() {
        if !reachable.insert(function.clone()) {
            continue;
        }
        if reachable.len() > maximum {
            return Err(CallClosureError::Limit);
        }
        let callees = graph
            .get(&function)
            .ok_or(CallClosureError::UnknownCallee)?;
        for callee in callees.iter().rev() {
            if !graph.contains_key(callee) {
                return Err(CallClosureError::UnknownCallee);
            }
            pending.push(callee.clone());
        }
    }

    let mut state = BTreeMap::<String, VisitState>::new();
    for function in &reachable {
        reject_cycles(function, &graph, &reachable, &mut state)?;
    }
    Ok(reachable.into_iter().collect())
}

/// Returns the canonical callee-first order for a complete emitted call graph.
///
/// Unlike [`resolve_call_closure`], this operates on every supplied function and is
/// intended for the graph reconstructed from reachable `CallStatic` instructions.
/// Functions without an emitted edge remain in the result and are ordered by their
/// canonical function ID whenever more than one node is ready.
pub fn canonical_callee_first_order<I, S>(functions: I) -> Result<Vec<String>, CallClosureError>
where
    I: IntoIterator<Item = (S, Vec<S>)>,
    S: Into<String>,
{
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for (function, callees) in functions {
        let function = function.into();
        let callees = callees.into_iter().map(Into::into).collect();
        if graph.insert(function, callees).is_some() {
            return Err(CallClosureError::DuplicateFunction);
        }
    }
    if graph
        .values()
        .flatten()
        .any(|callee| !graph.contains_key(callee))
    {
        return Err(CallClosureError::UnknownCallee);
    }

    let mut callers_by_callee = BTreeMap::<String, BTreeSet<String>>::new();
    let mut remaining_callees = BTreeMap::<String, usize>::new();
    for (function, callees) in &graph {
        remaining_callees.insert(function.clone(), callees.len());
        for callee in callees {
            callers_by_callee
                .entry(callee.clone())
                .or_default()
                .insert(function.clone());
        }
    }
    let mut ready = remaining_callees
        .iter()
        .filter_map(|(function, count)| (*count == 0).then_some(function.clone()))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(graph.len());
    while let Some(function) = ready.pop_first() {
        ordered.push(function.clone());
        for caller in callers_by_callee.get(&function).into_iter().flatten() {
            let count = remaining_callees
                .get_mut(caller)
                .ok_or(CallClosureError::UnknownCallee)?;
            *count = count
                .checked_sub(1)
                .ok_or(CallClosureError::UnknownCallee)?;
            if *count == 0 {
                ready.insert(caller.clone());
            }
        }
    }
    if ordered.len() != graph.len() {
        return Err(CallClosureError::Cycle);
    }
    Ok(ordered)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Visited,
}

fn reject_cycles(
    function: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    reachable: &BTreeSet<String>,
    state: &mut BTreeMap<String, VisitState>,
) -> Result<(), CallClosureError> {
    match state.get(function) {
        Some(VisitState::Visiting) => return Err(CallClosureError::Cycle),
        Some(VisitState::Visited) => return Ok(()),
        None => {}
    }
    state.insert(function.to_owned(), VisitState::Visiting);
    for callee in graph.get(function).ok_or(CallClosureError::UnknownCallee)? {
        if reachable.contains(callee) {
            reject_cycles(callee, graph, reachable, state)?;
        }
    }
    state.insert(function.to_owned(), VisitState::Visited);
    Ok(())
}
