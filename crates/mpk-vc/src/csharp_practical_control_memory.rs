//! Construction versions are separate from source array identities. An array
//! update evaluates to its assigned element while defining a new owned state.
use super::*;
#[derive(Default)]
pub(super) struct MemoryBlock {
    pub states: BTreeMap<String, String>,
    pub phis: Vec<ControlPhi>,
}
pub(super) struct MemoryPlan {
    pub objects: BTreeMap<String, BTreeMap<String, String>>,
    pub origins: BTreeMap<String, String>,
    pub allocations: BTreeMap<String, String>,
    pub blocks: BTreeMap<String, MemoryBlock>,
    pub updates: BTreeMap<String, String>,
}
pub(super) fn memory_plan(
    graph: &LoopControlFunction,
    local: &ControlSsa,
    aliases: &BTreeMap<String, String>,
) -> Result<MemoryPlan, DataPhaseError> {
    let mut origins = graph
        .nodes
        .iter()
        .filter(|n| n.operation == "allocate")
        .map(|n| (n.result.clone(), n.result.clone()))
        .collect::<BTreeMap<_, _>>();
    let allocations = graph
        .nodes
        .iter()
        .filter(|n| n.operation == "allocate")
        .map(|n| (n.result.clone(), n.inputs[0].clone()))
        .collect::<BTreeMap<_, _>>();
    let phis = local
        .blocks
        .iter()
        .flat_map(|b| &b.phis)
        .collect::<Vec<_>>();
    loop {
        let count = origins.len();
        for phi in &phis {
            let incoming = phi
                .incoming
                .iter()
                .map(|(_, id)| resolve_alias(aliases, id))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(origin) = incoming.iter().find_map(|id| origins.get(id).cloned()) {
                if incoming
                    .iter()
                    .filter_map(|id| origins.get(id))
                    .any(|o| *o != origin)
                {
                    return Err(DataPhaseError::Emission);
                }
                origins.insert(phi.id.clone(), origin);
            }
        }
        if count == origins.len() {
            break;
        }
    }
    for phi in &phis {
        if let Some(origin) = origins.get(&phi.id) {
            for (_, input) in &phi.incoming {
                if origins.get(&resolve_alias(aliases, input)?) != Some(origin) {
                    return Err(DataPhaseError::Emission);
                }
            }
        }
    }
    let ids = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    let mut predecessors = vec![BTreeSet::new(); graph.nodes.len()];
    let mut reachable = BTreeSet::new();
    let mut pending = vec![0];
    while let Some(i) = pending.pop() {
        if reachable.insert(i) {
            for target in graph.nodes[i]
                .successors
                .iter()
                .chain(&graph.nodes[i].exceptional_successors)
            {
                let j = ids[target.as_str()];
                predecessors[j].insert(i);
                pending.push(j);
            }
        }
    }
    let mut dom = vec![reachable.clone(); graph.nodes.len()];
    dom[0] = BTreeSet::from([0]);
    loop {
        let mut changed = false;
        for &i in &reachable {
            if i == 0 {
                continue;
            }
            let mut set = reachable.clone();
            for &p in &predecessors[i] {
                set = set.intersection(&dom[p]).copied().collect();
            }
            set.insert(i);
            changed |= set != dom[i];
            dom[i] = set;
        }
        if !changed {
            break;
        }
    }
    let mut object_origins = BTreeMap::<String, String>::new();
    for n in &graph.nodes {
        if n.operation == "construction_begin" {
            object_origins.insert(n.result.clone(), n.result.clone());
        } else if matches!(
            n.operation.as_str(),
            "construction_invoke" | "construction_write" | "construction_finalize"
        ) {
            let origin = object_origins
                .get(&n.inputs[0])
                .ok_or(DataPhaseError::Source)?
                .clone();
            object_origins.insert(n.result.clone(), origin);
        }
    }
    let mut object_lifetimes = BTreeMap::<String, BTreeSet<usize>>::new();
    for source in graph
        .nodes
        .iter()
        .filter(|n| n.operation == "construction_begin")
    {
        let mut seen = BTreeSet::new();
        let mut pending = vec![ids[source.successors[0].as_str()]];
        while let Some(i) = pending.pop() {
            if matches!(
                graph.nodes[i].kind.as_str(),
                "handler_entry"
                    | "handler_filter_entry"
                    | "handler_landing"
                    | "handler_finally_entry"
            ) || !seen.insert(i)
            {
                continue;
            }
            pending.extend(
                graph.nodes[i]
                    .successors
                    .iter()
                    .chain(&graph.nodes[i].exceptional_successors)
                    .map(|id| ids[id.as_str()]),
            );
        }
        object_lifetimes.insert(source.result.clone(), seen);
    }
    let mut objects = BTreeMap::new();
    for (i, n) in graph.nodes.iter().enumerate() {
        let mut states = BTreeMap::<String, (usize, String)>::new();
        let mut published = BTreeSet::new();
        for (j, source) in graph.nodes.iter().enumerate() {
            let Some(origin) = object_origins.get(&source.result) else {
                continue;
            };
            let normal = ids[source.successors[0].as_str()];
            if !dom[i].contains(&normal) || !object_lifetimes[origin].contains(&i) {
                continue;
            }
            if source.operation == "construction_finalize" {
                published.insert(origin.clone());
                continue;
            }
            let depth = dom[j].len();
            if states.get(origin).is_none_or(|(old, _)| depth > *old) {
                states.insert(origin.clone(), (depth, source.result.clone()));
            }
        }
        states.retain(|origin, _| !published.contains(origin));
        if matches!(
            n.kind.as_str(),
            "handler_entry" | "handler_filter_entry" | "handler_landing" | "handler_finally_entry"
        ) {
            states.clear();
        }
        objects.insert(
            n.id.clone(),
            states.into_iter().map(|(o, (_, v))| (o, v)).collect(),
        );
    }
    let allocation_nodes = graph
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.operation == "allocate")
        .map(|(i, n)| (n.result.clone(), i))
        .collect::<BTreeMap<_, _>>();
    // An owned allocation stops at the first handler/finally boundary. A
    // dominating publication may carry its immutable replacement beyond it.
    let mut lifetimes = BTreeMap::<String, BTreeSet<usize>>::new();
    for (origin, &allocation) in &allocation_nodes {
        let mut seen = BTreeSet::new();
        let mut pending = vec![ids[graph.nodes[allocation].successors[0].as_str()]];
        while let Some(i) = pending.pop() {
            if !seen.insert(i) {
                continue;
            }
            if matches!(
                graph.nodes[i].kind.as_str(),
                "handler_entry"
                    | "handler_filter_entry"
                    | "handler_landing"
                    | "handler_finally_entry"
            ) {
                continue;
            }
            pending.extend(
                graph.nodes[i]
                    .successors
                    .iter()
                    .chain(&graph.nodes[i].exceptional_successors)
                    .map(|id| ids[id.as_str()]),
            );
        }
        lifetimes.insert(origin.clone(), seen);
    }
    let mut publications = BTreeMap::<String, Vec<usize>>::new();
    for (i, n) in graph.nodes.iter().enumerate().filter(|(_, n)| {
        matches!(
            n.operation.as_str(),
            "publish" | "join_value" | "pending_some"
        )
    }) {
        if let Some(origin) = n
            .inputs
            .first()
            .map(|id| resolve_alias(aliases, id))
            .transpose()?
            .and_then(|id| origins.get(&id))
        {
            publications.entry(origin.clone()).or_default().push(i);
        }
    }
    let mut blocks = BTreeMap::<String, MemoryBlock>::new();
    let mut starts = BTreeMap::new();
    let mut owner = BTreeMap::new();
    let mut before = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut updates = BTreeMap::new();
    for (i, n) in graph.nodes.iter().enumerate() {
        let mut reads = Vec::new();
        for (origin, &allocation) in &allocation_nodes {
            let normal = ids[graph.nodes[allocation].successors[0].as_str()];
            // The allocation's value exists only on its successful edge.
            // Merely being dominated by the allocation includes its failure exit.
            if reachable.contains(&i)
                && dom[i].contains(&normal)
                && (lifetimes[origin].contains(&i)
                    || publications
                        .get(origin)
                        .is_some_and(|points| points.iter().any(|p| dom[i].contains(p))))
            {
                reads.push((
                    format!("{}.memory.in.{:04}", n.id, reads.len()),
                    origin.clone(),
                ));
            }
        }
        if reads.len() > 128 {
            return Err(DataPhaseError::ControlGraph(LoopLoweringError::Limit(
                "live_constructions",
            )));
        }
        starts.insert(
            n.id.clone(),
            reads
                .first()
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| n.id.clone()),
        );
        before.insert(n.id.clone(), reads);
        blocks.insert(n.id.clone(), MemoryBlock::default());
        if matches!(
            n.operation.as_str(),
            "update" | "join_value" | "publish" | "pending_some"
        ) {
            let receiver = resolve_alias(aliases, &n.inputs[0])?;
            let Some(origin) = origins.get(&receiver) else {
                if matches!(
                    n.operation.as_str(),
                    "join_value" | "publish" | "pending_some"
                ) {
                    continue;
                }
                return Err(DataPhaseError::Emission);
            };
            updates.insert(n.id.clone(), origin.clone());
        }
    }
    let node = |id: String,
                operation: &str,
                slot: String,
                inputs: Vec<String>,
                successors: Vec<String>| LoopControlNode {
        id,
        kind: "evaluate".into(),
        source_ordinal: None,
        operation: operation.into(),
        inputs,
        result: String::new(),
        slot,
        successors,
        exceptional_successors: vec![],
    };
    let mut augmented = graph.clone();
    augmented.nodes.clear();
    augmented.loops.clear();
    for n in &graph.nodes {
        let reads = &before[&n.id];
        for (i, (id, origin)) in reads.iter().enumerate() {
            owner.insert(id.clone(), n.id.clone());
            augmented.nodes.push(node(
                id.clone(),
                "load",
                origin.clone(),
                vec![],
                vec![reads
                    .get(i + 1)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| n.id.clone())],
            ));
        }
        let mut original = n.clone();
        original.operation.clear();
        original.inputs.clear();
        original.successors = n.successors.iter().map(|id| starts[id].clone()).collect();
        original.exceptional_successors = n
            .exceptional_successors
            .iter()
            .map(|id| starts[id].clone())
            .collect();
        owner.insert(n.id.clone(), n.id.clone());
        let store = if n.operation == "allocate" {
            Some((n.result.clone(), n.result.clone()))
        } else {
            updates
                .get(&n.id)
                .map(|origin| (origin.clone(), format!("{}.memory.value", n.id)))
        };
        if let Some((origin, value)) = store {
            let id = format!("{}.memory.out", n.id);
            let targets = original.successors.clone();
            original.successors = vec![id.clone()];
            augmented.nodes.push(original);
            owner.insert(id.clone(), n.id.clone());
            augmented
                .nodes
                .push(node(id, "store", origin, vec![value], targets));
        } else {
            augmented.nodes.push(original);
        }
    }
    let state = derive_control_ssa_bounded(
        &augmented,
        &BTreeMap::new(),
        &BTreeMap::new(),
        1024 * (128 + 2),
    )
    .map_err(DataPhaseError::ControlGraph)?;
    for block in state.blocks {
        let original = &owner[&block.node_id];
        let target = blocks.get_mut(original).ok_or(DataPhaseError::Emission)?;
        if let Some((_, origin)) = before[original].iter().find(|(id, _)| *id == block.node_id) {
            target.states.insert(
                origin.clone(),
                block.load_value_id.ok_or(DataPhaseError::Emission)?,
            );
        }
        for mut phi in block.phis {
            for (predecessor, _) in &mut phi.incoming {
                *predecessor = owner[predecessor].clone();
            }
            target.phis.push(phi);
        }
    }
    Ok(MemoryPlan {
        objects,
        origins,
        allocations,
        blocks,
        updates,
    })
}
