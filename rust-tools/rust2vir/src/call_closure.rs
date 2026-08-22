use std::collections::{BTreeMap, BTreeSet};

pub const MAX_CALL_CLOSURE_FUNCTIONS: usize = 128;

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
