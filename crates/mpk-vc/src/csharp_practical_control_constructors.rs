//! Constructor fields and the private receiver participate in the same SSA
//! algorithm as source locals. The receiver is never a public return value.
use super::*;

pub(super) fn prepare(
    emitter: &Emitter<'_>,
    body: &mut Body,
    graph: &mut LoopControlFunction,
    handler: &mut HandlerFunction,
) -> Result<BTreeMap<String, String>, DataPhaseError> {
    if !body.constructor {
        return Ok(BTreeMap::new());
    }
    // An unfinished receiver cannot be carried into a handler or finally. Ordinary
    // constructor failures still discard it on the caller-visible escape.
    if !handler.regions.is_empty() {
        return Err(DataPhaseError::Emission);
    }
    let source = emitter
        .r
        .source_types
        .get(&body.owner)
        .ok_or(DataPhaseError::Source)?;
    if body.object_constructor {
        body.variables
            .insert("this".into(), body.variables["construction:this"].clone());
    } else {
        let receiver = body.literal(MonomorphicValue::Unit {
            type_id: "mpk.csharp.value.unit.v1".into(),
        });
        body.variables.insert("this".into(), receiver);
    }
    for member in &source.members {
        let ty = closed_type_id(emitter.b, &member.ty).map_err(|_| DataPhaseError::Emission)?;
        if let Ok(default) = domain_default(emitter.b, emitter.r, emitter.c, &ty) {
            let value = body.literal(default);
            body.variables
                .insert(format!("constructor.field.{}", member.name), value);
        }
    }
    let mut next = 0;
    #[allow(clippy::too_many_arguments)]
    fn append(
        graph: &mut LoopControlFunction,
        handler: &mut HandlerFunction,
        next: &mut usize,
        frames: &[HandlerFrame],
        kind: &str,
        operation: &str,
        slot: String,
        inputs: Vec<String>,
        successors: Vec<String>,
    ) -> Result<(String, String), DataPhaseError> {
        if graph.nodes.len() >= 8192 {
            return Err(DataPhaseError::ControlGraph(LoopLoweringError::Limit(
                "cfg_blocks_per_closure",
            )));
        }
        let id = format!("{}.constructor.{:06}", graph.callable_id, *next);
        *next += 1;
        let result = if kind == "evaluate" {
            format!("{id}.value")
        } else {
            String::new()
        };
        graph.nodes.push(LoopControlNode {
            id: id.clone(),
            kind: kind.into(),
            source_ordinal: None,
            operation: operation.into(),
            inputs,
            result: result.clone(),
            slot,
            successors,
            exceptional_successors: vec![],
        });
        handler.contexts.push(HandlerContext {
            node: id.clone(),
            frames: frames.to_vec(),
        });
        Ok((id, result))
    }
    let original = graph.nodes.clone();
    for original in &original {
        let frames = handler
            .contexts
            .iter()
            .find(|c| c.node == original.id)
            .ok_or(DataPhaseError::Source)?
            .frames
            .clone();
        if original.operation == "construction_assign" {
            let name = original
                .slot
                .strip_prefix(&format!("{}.", body.owner))
                .ok_or(DataPhaseError::Source)?;
            if body.object_constructor {
                for node in &mut graph.nodes {
                    for input in &mut node.inputs {
                        if *input == original.result {
                            *input = original.inputs[1].clone();
                        }
                    }
                }
                let result = format!("{}.receiver", original.id);
                let (store, _) = append(
                    graph,
                    handler,
                    &mut next,
                    &frames,
                    "evaluate",
                    "store",
                    "this".into(),
                    vec![result.clone()],
                    original.successors.clone(),
                )?;
                let (field, _) = append(
                    graph,
                    handler,
                    &mut next,
                    &frames,
                    "evaluate",
                    "store",
                    format!("constructor.field.{name}"),
                    vec![original.inputs[1].clone()],
                    vec![store],
                )?;
                let node = graph
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == original.id)
                    .unwrap();
                node.operation = "constructor_write".into();
                node.result = result;
                node.successors = vec![field];
            } else {
                let node = graph
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == original.id)
                    .unwrap();
                node.operation = "store".into();
                node.slot = format!("constructor.field.{name}");
                node.inputs = vec![original.inputs[1].clone()];
            }
        } else if original.operation == "member" && original.slot.is_empty() {
            if let Some(op) = original
                .source_ordinal
                .and_then(|i| graph.operations.get(i))
            {
                if let Some(name) = op.symbol.strip_prefix(&format!("{}.", body.owner)) {
                    if source.members.iter().any(|m| m.name == name)
                        && original
                            .source_ordinal
                            .and_then(|i| graph.operations.get(i + 1))
                            .is_some_and(|o| o.kind == "InstanceReference")
                    {
                        let node = graph
                            .nodes
                            .iter_mut()
                            .find(|n| n.id == original.id)
                            .unwrap();
                        node.operation = "load".into();
                        node.slot = format!("constructor.field.{name}");
                        node.inputs.clear();
                    }
                }
            }
        }
        if original.kind == "return"
            || original.kind == "handler_completion" && original.operation == "return"
        {
            let mut sequence = Vec::new();
            let mut operands = Vec::new();
            let slots = if body.object_constructor {
                vec!["this".into()]
            } else {
                source
                    .members
                    .iter()
                    .map(|m| format!("constructor.field.{}", m.name))
                    .collect()
            };
            for slot in slots {
                let (id, value) = append(
                    graph,
                    handler,
                    &mut next,
                    &frames,
                    "evaluate",
                    "load",
                    slot,
                    vec![],
                    vec![],
                )?;
                sequence.push(id);
                operands.push(value);
            }
            if !body.object_constructor {
                let (id, value) = append(
                    graph,
                    handler,
                    &mut next,
                    &frames,
                    "evaluate",
                    "construct_payload",
                    body.owner.clone(),
                    operands,
                    vec![],
                )?;
                sequence.push(id);
                operands = vec![value];
            }
            let (done, _) = append(
                graph,
                handler,
                &mut next,
                &frames,
                "return",
                "",
                String::new(),
                operands,
                vec![],
            )?;
            sequence.push(done);
            for pair in sequence.windows(2) {
                graph
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == pair[0])
                    .unwrap()
                    .successors = vec![pair[1].clone()];
            }
            let node = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == original.id)
                .unwrap();
            node.kind = "jump".into();
            node.operation.clear();
            node.inputs.clear();
            node.successors = vec![sequence[0].clone()];
        }
    }
    if !body.object_constructor {
        return Ok(BTreeMap::new());
    }
    // Read the unique receiver on every actual node, so exception cleanup uses
    // its current SSA version even after a loop header or a conditional join.
    let originals = graph.nodes.clone();
    let mut entries = BTreeMap::new();
    let mut receivers = BTreeMap::new();
    let structural = graph
        .loops
        .iter()
        .flat_map(|r| {
            [
                r.header.clone(),
                r.body.clone(),
                r.continue_target.clone(),
                r.exit.clone(),
            ]
        })
        .collect::<BTreeSet<_>>();
    for n in originals
        .iter()
        .filter(|n| n.kind != "entry" && !structural.contains(&n.id))
    {
        let frames = handler
            .contexts
            .iter()
            .find(|c| c.node == n.id)
            .ok_or(DataPhaseError::Source)?
            .frames
            .clone();
        let (read, value) = append(
            graph,
            handler,
            &mut next,
            &frames,
            "evaluate",
            "load",
            "this".into(),
            vec![],
            vec![n.id.clone()],
        )?;
        entries.insert(n.id.clone(), read);
        receivers.insert(n.id.clone(), value);
    }
    for n in &mut graph.nodes {
        if !originals.iter().any(|o| o.id == n.id) {
            continue;
        }
        for target in n.successors.iter_mut().chain(&mut n.exceptional_successors) {
            if let Some(entry) = entries.get(target) {
                *target = entry.clone();
            }
        }
    }
    // Loop headers stay explicit. A read preceding the header would otherwise
    // turn its canonical incoming backedge into a second, hidden cycle.
    for region in &graph.loops {
        if let Some(read) = entries.get(&region.header) {
            for n in &mut graph.nodes {
                for target in n.successors.iter_mut().chain(&mut n.exceptional_successors) {
                    if target == read {
                        *target = region.header.clone();
                    }
                }
            }
            let header = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == region.header)
                .unwrap();
            receivers.remove(&header.id);
        }
    }
    Ok(receivers)
}
