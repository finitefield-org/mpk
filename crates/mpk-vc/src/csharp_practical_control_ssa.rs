//! Pruned SSA for the source-order control handoff. All merges are explicit;
//! local storage never survives into an ordinary VIR operand.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlPhi {
    pub id: String,
    pub slot: String,
    pub incoming: Vec<(String, String)>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlSsaBlock {
    pub node_id: String,
    pub phis: Vec<ControlPhi>,
    pub load_value_id: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlSsa {
    pub blocks: Vec<ControlSsaBlock>,
}
/// Slot definitions contain the actual entry parameter IDs and the catch
/// subject defined at each handler/filter entry. The caller supplies types
/// separately, and the ordinary importer rechecks every phi and operand.
pub fn derive_control_ssa(
    function: &LoopControlFunction,
    parameters: &BTreeMap<String, String>,
    caught: &BTreeMap<String, (String, String)>,
) -> Result<ControlSsa, LoopLoweringError> {
    derive_control_ssa_bounded(function, parameters, caught, 1024)
}

pub(super) fn derive_control_ssa_bounded(
    function: &LoopControlFunction,
    parameters: &BTreeMap<String, String>,
    caught: &BTreeMap<String, (String, String)>,
    maximum_nodes: usize,
) -> Result<ControlSsa, LoopLoweringError> {
    use LoopLoweringError::{Graph, Operand};
    let nodes = &function.nodes;
    if nodes.is_empty() || nodes.len() > maximum_nodes {
        return Err(Graph);
    }
    let ids = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    if ids.len() != nodes.len() {
        return Err(Graph);
    }
    let mut predecessors = vec![BTreeSet::new(); nodes.len()];
    let mut successors = vec![BTreeSet::new(); nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for target in node.successors.iter().chain(&node.exceptional_successors) {
            let j = *ids.get(target.as_str()).ok_or(Graph)?;
            predecessors[j].insert(i);
            successors[i].insert(j);
        }
    }
    let mut reachable = BTreeSet::new();
    let mut pending = vec![0];
    while let Some(i) = pending.pop() {
        if reachable.insert(i) {
            pending.extend(successors[i].iter().copied());
        }
    }
    for incoming in &mut predecessors {
        incoming.retain(|i| reachable.contains(i));
    }
    let mut definitions = vec![BTreeMap::new(); nodes.len()];
    definitions[0] = parameters.clone();
    for (i, node) in nodes.iter().enumerate() {
        if node.operation == "store" || node.operation == "pattern_bind" {
            if node.inputs.len() != 1 || node.slot.is_empty() {
                return Err(Operand);
            }
            definitions[i].insert(
                node.slot.clone(),
                if node.operation == "pattern_bind" {
                    node.result.clone()
                } else {
                    node.inputs[0].clone()
                },
            );
        }
        if let Some((slot, value)) = caught.get(&node.id) {
            if slot.is_empty() || value.is_empty() {
                return Err(Operand);
            }
            definitions[i].insert(slot.clone(), value.clone());
        }
    }
    if caught.keys().any(|id| !ids.contains_key(id.as_str())) {
        return Err(Graph);
    }
    let mut live = vec![BTreeSet::<String>::new(); nodes.len()];
    loop {
        let mut changed = false;
        for &i in reachable.iter().rev() {
            let mut next = successors[i]
                .iter()
                .flat_map(|&j| live[j].iter().cloned())
                .collect::<BTreeSet<_>>();
            next.retain(|s| !definitions[i].contains_key(s));
            if nodes[i].operation == "load" {
                next.insert(nodes[i].slot.clone());
            }
            if next != live[i] {
                live[i] = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut phis = BTreeMap::<(usize, String), String>::new();
    for &i in &reachable {
        if predecessors[i].len() > 1 {
            for (ordinal, slot) in live[i].iter().enumerate() {
                phis.insert(
                    (i, slot.clone()),
                    format!("{}.phi.{ordinal:06}", nodes[i].id),
                );
            }
        }
    }
    // Walk straight-line predecessor chains iteratively. A cycle must meet a
    // preallocated loop phi; a parameter/local absent on any used path rejects.
    let resolve = |mut i: usize, slot: &str, after: bool| -> Result<String, LoopLoweringError> {
        if after {
            if let Some(value) = definitions[i].get(slot) {
                return Ok(value.clone());
            }
        }
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(i) {
                return Err(Graph);
            }
            if let Some(phi) = phis.get(&(i, slot.into())) {
                return Ok(phi.clone());
            }
            let mut p = predecessors[i].iter();
            let previous = *p.next().ok_or(Operand)?;
            if p.next().is_some() {
                return Err(Graph);
            }
            if let Some(value) = definitions[previous].get(slot) {
                return Ok(value.clone());
            }
            i = previous;
        }
    };
    let mut blocks = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let mut block = ControlSsaBlock {
            node_id: node.id.clone(),
            phis: Vec::new(),
            load_value_id: None,
        };
        if reachable.contains(&i) {
            for slot in &live[i] {
                if let Some(id) = phis.get(&(i, slot.clone())) {
                    let incoming = predecessors[i]
                        .iter()
                        .map(|&p| Ok((nodes[p].id.clone(), resolve(p, slot, true)?)))
                        .collect::<Result<Vec<_>, LoopLoweringError>>()?;
                    block.phis.push(ControlPhi {
                        id: id.clone(),
                        slot: slot.clone(),
                        incoming,
                    });
                }
            }
            if node.operation == "load" {
                block.load_value_id = Some(resolve(i, &node.slot, false)?);
            }
        }
        blocks.push(block);
    }
    Ok(ControlSsa { blocks })
}
