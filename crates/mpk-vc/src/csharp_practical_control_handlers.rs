//! Translate lexical catch selection into closed, typed exceptional edges.
use super::*;

fn filter_fallback(
    handler: &HandlerFunction,
    universe: &ClosedExceptionUniverse,
    catch_id: &str,
) -> Result<String, DataPhaseError> {
    let mut choices = BTreeSet::new();
    for transfer in &handler.transfers {
        let Some(index) = transfer
            .candidates
            .iter()
            .position(|c| c.catch_id == catch_id)
        else {
            continue;
        };
        let candidate = &transfer.candidates[index];
        for arm in universe
            .arms()
            .iter()
            .filter(|a| universe.catch_is_ancestor(&candidate.type_id, &a.type_id))
        {
            let next = transfer.candidates[index + 1..]
                .iter()
                .find(|c| universe.catch_is_ancestor(&c.type_id, &arm.type_id));
            choices.insert(next.map(|c| c.filter.as_ref().unwrap_or(&c.entry).clone()));
        }
    }
    if choices.len() != 1 {
        return Err(DataPhaseError::Emission);
    }
    choices
        .pop_first()
        .flatten()
        .ok_or(DataPhaseError::Emission)
}

pub(super) fn direct_catch_graph(
    function: &mut LoopControlFunction,
    handler: &HandlerFunction,
    universe: &ClosedExceptionUniverse,
) -> Result<(), DataPhaseError> {
    if handler.regions.is_empty() {
        return Ok(());
    }
    let filter_exits = function
        .nodes
        .iter()
        .filter(|n| n.kind == "handler_filter_result" && n.operation == "expanded_filter")
        .map(|n| (n.slot.clone(), n.successors[1].clone()))
        .collect::<BTreeMap<_, _>>();
    let landings = function
        .nodes
        .iter()
        .filter(|n| n.kind == "handler_landing")
        .map(|n| n.id.clone())
        .collect::<BTreeSet<_>>();
    for node in &mut function.nodes {
        if node.kind == "handler_filter_result" && node.operation != "expanded_filter" {
            let clause = handler
                .regions
                .iter()
                .flat_map(|r| &r.catches)
                .find(|c| c.id == node.slot)
                .ok_or(DataPhaseError::Source)?;
            node.successors = vec![
                clause.entry.clone(),
                filter_fallback(handler, universe, &clause.id)?,
            ];
        }
        if node.exceptional_successors.is_empty() {
            continue;
        }
        let mut targets = BTreeSet::new();
        for search in &node.exceptional_successors {
            if landings.contains(search) {
                targets.insert(search.clone());
                continue;
            }
            let transfer = handler
                .transfers
                .iter()
                .find(|t| t.node == *search)
                .ok_or(DataPhaseError::Source)?;
            if let Some(catch) = &transfer.filter_catch {
                targets.insert(if let Some(next) = filter_exits.get(catch) {
                    next.clone()
                } else {
                    filter_fallback(handler, universe, catch)?
                });
                continue;
            }
            for arm in universe.arms() {
                let known = if node.kind == "rethrow" {
                    handler
                        .regions
                        .iter()
                        .flat_map(|r| &r.catches)
                        .find(|c| c.id == node.slot)
                        .map(|c| c.type_id.as_str())
                } else if matches!(
                    node.kind.as_str(),
                    "explicit_throw" | "builtin_throw" | "closed_rethrow"
                ) {
                    Some(node.slot.as_str())
                } else {
                    None
                };
                if known.is_some_and(|ty| !universe.catch_is_ancestor(ty, &arm.type_id)) {
                    continue;
                }
                if let Some(candidate) = transfer
                    .candidates
                    .iter()
                    .find(|c| universe.catch_is_ancestor(&c.type_id, &arm.type_id))
                {
                    targets.insert(
                        candidate
                            .filter
                            .as_ref()
                            .unwrap_or(&candidate.entry)
                            .clone(),
                    );
                }
            }
        }
        node.exceptional_successors = targets.into_iter().collect();
    }
    function.nodes.retain(|n| n.kind != "handler_search");
    Ok(())
}

pub(super) fn attach_direct_catches(
    body: &mut Body,
    function: &LoopControlFunction,
    handler: &HandlerFunction,
    universe: &ClosedExceptionUniverse,
    starts: &BTreeMap<String, usize>,
    emitted: &BTreeMap<String, Vec<String>>,
) -> Result<(), DataPhaseError> {
    if handler.regions.is_empty() {
        return Ok(());
    }
    let contexts = handler
        .contexts
        .iter()
        .map(|c| (c.node.as_str(), &c.frames))
        .collect::<BTreeMap<_, _>>();
    let regions = handler
        .regions
        .iter()
        .map(|r| (r.id.as_str(), r))
        .collect::<BTreeMap<_, _>>();
    let entry = |id: &str| {
        starts
            .get(id)
            .map(|i| body.function.blocks[*i].node.id.clone())
    };
    // Unreachable clauses are omitted from the ordinary graph; source anchors
    // still bind the accepted source and its complete lexical catch inventory.
    for region in &handler.regions {
        let Some(try_entry) = entry(&region.try_entry) else {
            continue;
        };
        let parents = contexts[region.try_entry.as_str()]
            [..contexts[region.try_entry.as_str()].len() - 1]
            .iter()
            .filter(|f| f.zone != "finally")
            .collect::<Vec<_>>();
        let catches = region
            .catches
            .iter()
            .filter_map(|c| entry(&c.entry).map(|id| (c, id)))
            .enumerate()
            .map(|(ordinal, (c, id))| {
                let filter = c
                    .filter
                    .as_ref()
                    .map(|filter_entry| {
                        let result = function
                            .nodes
                            .iter()
                            .find(|n| n.kind == "handler_filter_result" && n.slot == c.id)
                            .ok_or(DataPhaseError::Source)?;
                        let next = if result.operation == "expanded_filter" {
                            result.successors[1].clone()
                        } else {
                            filter_fallback(handler, universe, &c.id)?
                        };
                        let selected = if result.operation == "expanded_filter" {
                            result.successors[0].clone()
                        } else {
                            c.entry.clone()
                        };
                        let mut node_ids = function
                            .nodes
                            .iter()
                            .filter(|n| {
                                contexts[n.id.as_str()]
                                    .last()
                                    .is_some_and(|f| f.zone == "filter" && f.catch_id == c.id)
                            })
                            .flat_map(|n| emitted[&n.id].clone())
                            .collect::<Vec<_>>();
                        node_ids.sort();
                        let next_search_node_id = entry(&next).ok_or(DataPhaseError::Emission)?;
                        Ok::<_, DataPhaseError>(ExceptionFilterRule {
                            execution: Some(ExceptionFilterExecution {
                                selected_node_id: entry(&selected)
                                    .ok_or(DataPhaseError::Emission)?,
                                entry_node_id: entry(filter_entry)
                                    .ok_or(DataPhaseError::Emission)?,
                                result_node_id: entry(&result.id)
                                    .ok_or(DataPhaseError::Emission)?,
                                node_ids,
                                next_search_node_id: next_search_node_id.clone(),
                            }),
                            condition_type_id: BOOL_TYPE_ID.into(),
                            thrown_filter_exception_successor_id: next_search_node_id,
                            throw_means_false: true,
                            preserves_original_exception: true,
                        })
                    })
                    .transpose()?;
                Ok::<_, DataPhaseError>(CatchHandler {
                    ordinal: ordinal as u32,
                    exception_type_id: c.type_id.clone(),
                    filter,
                    handler_entry_node_id: id,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if catches.is_empty() && region.finally_entry.is_none() {
            continue;
        }
        body.function
            .exception_regions
            .push(ExceptionHandlerRegion {
                search_entry_node_ids: function
                    .nodes
                    .iter()
                    .filter(|n| {
                        matches!(
                            n.kind.as_str(),
                            "handler_landing" | "handler_restore" | "handler_escape"
                        ) && contexts[n.id.as_str()]
                            .iter()
                            .rev()
                            .find(|f| f.zone != "finally")
                            .is_some_and(|f| f.region == region.id)
                    })
                    .filter_map(|n| entry(&n.id))
                    .collect(),
                id: region.id.clone(),
                parent_region_id: parents.last().map(|f| f.region.clone()),
                nesting_depth: parents.len() as u32,
                try_entry_node_id: try_entry,
                catches,
                finally_entry_node_id: region.finally_entry.as_ref().and_then(|id| entry(id)),
            });
    }
    let retained = body
        .function
        .exception_regions
        .iter()
        .map(|r| r.id.clone())
        .collect::<BTreeSet<_>>();
    let ids = starts
        .iter()
        .map(|(id, i)| (id.clone(), body.function.blocks[*i].node.id.clone()))
        .collect::<BTreeMap<_, _>>();
    for source in &function.nodes {
        if source.kind == "handler_search" {
            continue;
        }
        let frames = contexts[source.id.as_str()];
        for id in &emitted[&source.id] {
            let block = body
                .function
                .blocks
                .iter_mut()
                .find(|b| b.node.id == *id)
                .ok_or(DataPhaseError::Emission)?;
            block.node.region_stack = frames
                .iter()
                .filter(|f| f.zone != "finally" && retained.contains(&f.region))
                .map(|f| f.region.clone())
                .collect();
            for edge in &mut block.node.exceptional_successors {
                if source.kind == "completion_throw" {
                    continue;
                }
                if let Some(landing) = source.exceptional_successors.iter().find(|id| {
                    function
                        .nodes
                        .iter()
                        .any(|n| n.id == **id && n.kind == "handler_landing")
                }) {
                    edge.target_id = ids[landing].clone();
                    let transfer = handler
                        .transfers
                        .iter()
                        .find(|t| t.node == *landing)
                        .ok_or(DataPhaseError::Source)?;
                    let candidate = transfer
                        .candidates
                        .iter()
                        .find(|c| universe.catch_is_ancestor(&c.type_id, &edge.exception_type_id));
                    let finalies = candidate
                        .map(|c| c.finally_entries.as_slice())
                        .unwrap_or(&transfer.finally_entries)
                        .iter()
                        .map(|entry| {
                            handler
                                .regions
                                .iter()
                                .find(|r| r.finally_entry.as_ref() == Some(entry))
                                .map(|r| r.id.clone())
                                .ok_or(DataPhaseError::Source)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    body.function.unwind_plans.push(ExceptionUnwindPlan {
                        search_entry_node_id: Some(edge.target_id.clone()),
                        source_node_id: block.node.id.clone(),
                        check_id: edge.check_id.clone(),
                        from_region_id: block.node.region_stack.last().cloned(),
                        selected_handler_region_id: candidate
                            .and_then(|c| {
                                handler.regions.iter().find(|r| {
                                    r.catches.iter().any(|clause| clause.id == c.catch_id)
                                })
                            })
                            .map(|r| r.id.clone()),
                        finally_region_ids: finalies,
                        destination_node_id: candidate
                            .map(|c| ids[&c.entry].clone())
                            .unwrap_or_else(|| "pending.exit".into()),
                    });
                    continue;
                }
                if let Some(filter) = frames.iter().rev().find(|f| f.zone == "filter") {
                    let expanded = function.nodes.iter().find(|n| {
                        n.kind == "handler_filter_result"
                            && n.slot == filter.catch_id
                            && n.operation == "expanded_filter"
                    });
                    let next = if let Some(result) = expanded {
                        result.successors[1].clone()
                    } else {
                        filter_fallback(handler, universe, &filter.catch_id)?
                    };
                    edge.target_id = ids.get(&next).ok_or(DataPhaseError::Emission)?.clone();
                    let selected = handler.regions.iter().find(|r| {
                        r.catches
                            .iter()
                            .any(|c| c.entry == next || c.filter.as_ref() == Some(&next))
                    });
                    body.function.unwind_plans.push(ExceptionUnwindPlan {
                        search_entry_node_id: None,
                        source_node_id: block.node.id.clone(),
                        check_id: edge.check_id.clone(),
                        from_region_id: block.node.region_stack.last().cloned(),
                        selected_handler_region_id: selected.map(|r| r.id.clone()),
                        finally_region_ids: vec![],
                        destination_node_id: edge.target_id.clone(),
                    });
                    continue;
                }
                let selected = frames
                    .iter()
                    .rev()
                    .filter(|f| f.zone == "try")
                    .find_map(|f| {
                        regions[f.region.as_str()]
                            .catches
                            .iter()
                            .find(|c| {
                                universe.catch_is_ancestor(&c.type_id, &edge.exception_type_id)
                            })
                            .map(|c| (f, c))
                    });
                if let Some((frame, clause)) = selected {
                    edge.target_id = ids
                        .get(clause.filter.as_ref().unwrap_or(&clause.entry))
                        .ok_or(DataPhaseError::Emission)?
                        .clone();
                    body.function.unwind_plans.push(ExceptionUnwindPlan {
                        search_entry_node_id: None,
                        source_node_id: block.node.id.clone(),
                        check_id: edge.check_id.clone(),
                        from_region_id: block.node.region_stack.last().cloned(),
                        selected_handler_region_id: Some(frame.region.clone()),
                        finally_region_ids: vec![],
                        destination_node_id: ids[&clause.entry].clone(),
                    });
                }
            }
            if let Some(invocation) = &mut block.invocation {
                invocation.exceptional_successors = block.node.exceptional_successors.clone();
            }
        }
    }
    Ok(())
}

/// Source lowering conservatively gives some pure comparisons a failure path.
/// Retain exactly the native checked-operation edges, then repair their phis
/// and metadata before the ordinary importer validates the complete function.
pub(super) fn prune_native(function: &mut v::PracticalVirFunction) {
    let nodes = function
        .blocks
        .iter()
        .map(|b| (b.node.id.clone(), b))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::new();
    let mut pending = vec![function.blocks[0].node.id.clone()];
    while let Some(id) = pending.pop() {
        if !reachable.insert(id.clone()) {
            continue;
        }
        let Some(block) = nodes.get(&id) else {
            continue;
        };
        pending.extend(block.node.normal_successor_ids.iter().cloned());
        pending.extend(
            block
                .node
                .exceptional_successors
                .iter()
                .map(|e| e.target_id.clone()),
        );
        if let Some(
            AbruptCompletion::Break { target_id, .. }
            | AbruptCompletion::Continue { target_id, .. },
        ) = &block.node.abrupt
        {
            pending.push(target_id.clone());
        }
    }
    function.blocks.retain(|b| reachable.contains(&b.node.id));
    for (ordinal, block) in function.blocks.iter_mut().enumerate() {
        block.node.ordinal = ordinal as u32;
        for phi in &mut block.phi_values {
            phi.incoming
                .retain(|i| reachable.contains(&i.predecessor_node_id));
        }
    }
    function
        .loops
        .retain(|l| reachable.contains(&l.header_node_id));
    for l in &mut function.loops {
        l.backedge_source_ids.retain(|id| reachable.contains(id));
        if !reachable.contains(&l.continue_target_node_id) && l.backedge_source_ids.is_empty() {
            l.continue_target_node_id = l.header_node_id.clone();
        }
    }
    function
        .unwind_plans
        .retain(|p| reachable.contains(&p.source_node_id));
    function
        .exception_regions
        .retain(|r| reachable.contains(&r.try_entry_node_id));
    for region in &mut function.exception_regions {
        region
            .catches
            .retain(|c| reachable.contains(&c.handler_entry_node_id));
        for (ordinal, c) in region.catches.iter_mut().enumerate() {
            c.ordinal = ordinal as u32;
            if let Some(execution) = c.filter.as_mut().and_then(|f| f.execution.as_mut()) {
                execution.node_ids.retain(|id| reachable.contains(id));
            }
        }
        region
            .search_entry_node_ids
            .retain(|id| reachable.contains(id));
    }
    let parents = function
        .exception_regions
        .iter()
        .map(|r| (r.id.clone(), r.parent_region_id.clone()))
        .collect::<BTreeMap<_, _>>();
    function
        .exception_regions
        .retain(|r| !r.catches.is_empty() || r.finally_entry_node_id.is_some());
    let regions = function
        .exception_regions
        .iter()
        .map(|r| r.id.clone())
        .collect::<BTreeSet<_>>();
    for region in &mut function.exception_regions {
        let mut parent = region.parent_region_id.clone();
        while parent.as_ref().is_some_and(|id| !regions.contains(id)) {
            parent = parents[parent.as_ref().unwrap()].clone();
        }
        region.parent_region_id = parent;
        let mut depth = 0;
        let mut parent = region.parent_region_id.as_ref();
        while let Some(id) = parent {
            if regions.contains(id) {
                depth += 1;
            }
            parent = parents[id].as_ref();
        }
        region.nesting_depth = depth;
    }
    for block in &mut function.blocks {
        block.node.region_stack.retain(|id| regions.contains(id));
    }
    for plan in &mut function.unwind_plans {
        plan.from_region_id = function
            .blocks
            .iter()
            .find(|b| b.node.id == plan.source_node_id)
            .and_then(|b| b.node.region_stack.last())
            .cloned();
    }
    if let Some(protocol) = &mut function.object_protocol {
        protocol
            .initializations
            .retain(|i| reachable.contains(&i.begin_node_id));
    }
    if let Some(protocol) = &mut function.control_protocol {
        protocol
            .escape_search_node_ids
            .retain(|id| reachable.contains(id));
        protocol
            .anchors
            .retain(|a| reachable.contains(&a.entry_node_id));
        for anchor in &mut protocol.anchors {
            anchor.artifact_node_ids.retain(|id| reachable.contains(id));
        }
    }
}
