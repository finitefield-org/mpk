//! Expand pending finally completions into ordinary SSA stores and branches.
//! Each lexical finally has distinct slots, so an inner finally cannot overwrite
//! the completion suspended by an outer one. No mutable runtime carrier escapes.
use super::*;

struct Builder<'a> {
    graph: &'a mut LoopControlFunction,
    handler: &'a mut HandlerFunction,
    next: usize,
}
impl Builder<'_> {
    fn node(
        &mut self,
        frames: &[HandlerFrame],
        kind: &str,
        operation: &str,
        inputs: Vec<String>,
        slot: &str,
        successors: Vec<String>,
    ) -> Result<String, DataPhaseError> {
        if self.graph.nodes.len() >= 8192 {
            return Err(DataPhaseError::ControlGraph(LoopLoweringError::Limit(
                "cfg_blocks_per_closure",
            )));
        }
        let id = format!("{}.completion.{:06}", self.graph.callable_id, self.next);
        self.next += 1;
        let result = if kind == "evaluate" {
            format!("{id}.value")
        } else {
            String::new()
        };
        self.graph.nodes.push(LoopControlNode {
            id: id.clone(),
            kind: kind.into(),
            source_ordinal: None,
            operation: operation.into(),
            inputs,
            result,
            slot: slot.into(),
            successors,
            exceptional_successors: vec![],
        });
        self.handler.contexts.push(HandlerContext {
            node: id.clone(),
            frames: frames.to_vec(),
        });
        Ok(id)
    }
    fn value(&self, id: &str) -> String {
        self.graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .unwrap()
            .result
            .clone()
    }
    fn link(&mut self, from: &str, to: &str) {
        self.graph
            .nodes
            .iter_mut()
            .find(|n| n.id == from)
            .unwrap()
            .successors = vec![to.into()];
    }
    fn save(
        &mut self,
        frames: &[HandlerFrame],
        slot: &str,
        operation: &str,
        input: Vec<String>,
        literal: &str,
    ) -> Result<(String, String), DataPhaseError> {
        let value = self.node(frames, "evaluate", operation, input, literal, vec![])?;
        let store = self.node(
            frames,
            "evaluate",
            "store",
            vec![self.value(&value)],
            slot,
            vec![],
        )?;
        self.link(&value, &store);
        Ok((value, store))
    }
}
#[derive(Clone)]
struct Resume {
    kind: String,
    target: Option<String>,
    loop_id: String,
    next_finally: Option<String>,
}

fn setup(
    b: &mut Builder<'_>,
    resumptions: &mut BTreeMap<String, Vec<Resume>>,
    frames: &[HandlerFrame],
    finalies: &[String],
    resume: Resume,
    return_value: Option<String>,
    exception_value: Option<String>,
) -> Result<String, DataPhaseError> {
    let mut chain = Vec::new();
    for (index, entry) in finalies.iter().enumerate() {
        let rows = resumptions.entry(entry.clone()).or_default();
        let selector = rows.len();
        let mut row = resume.clone();
        row.next_finally = finalies.get(index + 1).cloned();
        rows.push(row);
        let prefix = format!("{entry}.pending");
        chain.push(b.save(
            frames,
            &format!("{prefix}.kind"),
            "pending_kind",
            vec![],
            &selector.to_string(),
        )?);
        chain.push(b.save(
            frames,
            &format!("{prefix}.return"),
            if return_value.is_some() {
                "pending_some"
            } else {
                "pending_none"
            },
            return_value.clone().into_iter().collect(),
            "",
        )?);
        chain.push(b.save(
            frames,
            &format!("{prefix}.exception"),
            if exception_value.is_some() {
                "pending_exception"
            } else {
                "pending_exception_default"
            },
            exception_value.clone().into_iter().collect(),
            "",
        )?);
    }
    for pair in chain.windows(2) {
        b.link(&pair[0].1, &pair[1].0);
    }
    b.link(
        &chain.last().ok_or(DataPhaseError::Emission)?.1,
        &finalies[0],
    );
    if resume.kind == "throw" {
        Ok(b.node(
            frames,
            "handler_escape",
            "",
            vec![exception_value.ok_or(DataPhaseError::Emission)?],
            "",
            vec![chain[0].0.clone()],
        )?)
    } else {
        Ok(chain[0].0.clone())
    }
}

#[allow(clippy::too_many_arguments)]
fn search_route(
    b: &mut Builder<'_>,
    resumptions: &mut BTreeMap<String, Vec<Resume>>,
    filters: &mut BTreeMap<String, Vec<(String, String)>>,
    transfer: &HandlerTransfer,
    start: usize,
    exception_type: &str,
    value: &str,
    frames: &[HandlerFrame],
    universe: &ClosedExceptionUniverse,
) -> Result<String, DataPhaseError> {
    let mut search_frames = frames.to_vec();
    for frame in &mut search_frames {
        if frame.zone == "filter" {
            frame.zone = "search".into();
        }
    }
    let frames = search_frames.as_slice();
    let candidate = transfer
        .candidates
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, c)| universe.catch_is_ancestor(&c.type_id, exception_type));
    if let Some((index, candidate)) = candidate {
        if let Some(entry) = &candidate.filter {
            let mut filter_frames = b
                .handler
                .contexts
                .iter()
                .find(|c| c.node == *entry)
                .ok_or(DataPhaseError::Source)?
                .frames
                .clone();
            for frame in &mut filter_frames {
                if frame.zone == "filter" {
                    frame.zone = "search".into();
                }
            }
            let original = format!("{entry}.exception");
            let yes = if candidate.finally_entries.is_empty() {
                b.node(
                    &filter_frames,
                    "handler_restore",
                    "",
                    vec![original.clone()],
                    "",
                    vec![candidate.entry.clone()],
                )?
            } else {
                setup(
                    b,
                    resumptions,
                    &filter_frames,
                    &candidate.finally_entries,
                    Resume {
                        kind: "catch".into(),
                        target: Some(candidate.entry.clone()),
                        loop_id: String::new(),
                        next_finally: None,
                    },
                    None,
                    Some(original.clone()),
                )?
            };
            let no = search_route(
                b,
                resumptions,
                filters,
                transfer,
                index + 1,
                exception_type,
                &original,
                &filter_frames,
                universe,
            )?;
            let rows = filters.entry(candidate.catch_id.clone()).or_default();
            let selector = rows.len();
            rows.push((yes, no));
            let saved = b.save(
                frames,
                &format!("{entry}.search_selector"),
                "pending_kind",
                vec![],
                &selector.to_string(),
            )?;
            let restored = b.node(
                frames,
                "handler_restore",
                "",
                vec![value.into()],
                "",
                vec![entry.clone()],
            )?;
            b.link(&saved.1, &restored);
            return Ok(saved.0);
        }
    }
    let candidate = candidate.map(|(_, c)| c);
    let finalies = candidate
        .map(|c| c.finally_entries.as_slice())
        .unwrap_or(&transfer.finally_entries);
    let target = candidate.map(|c| c.entry.clone());
    if finalies.is_empty() {
        Ok(if let Some(target) = target {
            b.node(
                frames,
                "handler_restore",
                "",
                vec![value.into()],
                "",
                vec![target],
            )?
        } else {
            b.node(
                frames,
                "completion_throw",
                "",
                vec![value.into()],
                exception_type,
                vec![],
            )?
        })
    } else {
        setup(
            b,
            resumptions,
            frames,
            finalies,
            Resume {
                kind: if target.is_some() {
                    "catch".into()
                } else {
                    "throw".into()
                },
                target,
                loop_id: String::new(),
                next_finally: None,
            },
            None,
            Some(value.into()),
        )
    }
}

fn filter_dispatch(
    b: &mut Builder<'_>,
    frames: &[HandlerFrame],
    slot: &str,
    destinations: &[String],
) -> Result<String, DataPhaseError> {
    let load = b.node(frames, "evaluate", "load", vec![], slot, vec![])?;
    let value = b.value(&load);
    let mut tests = Vec::new();
    for (index, destination) in destinations.iter().enumerate() {
        let test = b.node(
            frames,
            "evaluate",
            "pending_is_kind",
            vec![value.clone()],
            &index.to_string(),
            vec![],
        )?;
        let branch = b.node(
            frames,
            "branch",
            "",
            vec![b.value(&test)],
            "",
            vec![destination.clone(), String::new()],
        )?;
        b.link(&test, &branch);
        tests.push((test, branch, destination.clone()));
    }
    for i in 0..tests.len() {
        let next = if i + 1 < tests.len() {
            tests[i + 1].0.clone()
        } else {
            tests[i].2.clone()
        };
        b.graph
            .nodes
            .iter_mut()
            .find(|n| n.id == tests[i].1)
            .unwrap()
            .successors[1] = next;
    }
    b.link(&load, &tests.first().ok_or(DataPhaseError::Emission)?.0);
    Ok(load)
}

pub(super) fn expand_rethrows(
    graph: &mut LoopControlFunction,
    handler: &mut HandlerFunction,
    universe: &ClosedExceptionUniverse,
) -> Result<(), DataPhaseError> {
    let originals = graph
        .nodes
        .iter()
        .filter(|n| n.kind == "rethrow")
        .cloned()
        .collect::<Vec<_>>();
    let mut b = Builder {
        graph,
        handler,
        next: 0,
    };
    for original in originals {
        let clause = b
            .handler
            .regions
            .iter()
            .flat_map(|r| &r.catches)
            .find(|c| c.id == original.slot)
            .ok_or(DataPhaseError::Source)?
            .clone();
        if universe
            .arms()
            .iter()
            .filter(|a| universe.catch_is_ancestor(&clause.type_id, &a.type_id))
            .count()
            <= 1
        {
            continue;
        }
        let frames = b
            .handler
            .contexts
            .iter()
            .find(|c| c.node == original.id)
            .ok_or(DataPhaseError::Source)?
            .frames
            .clone();
        let value = format!("{}.exception", clause.entry);
        let mut tests = Vec::new();
        for arm in universe
            .arms()
            .iter()
            .filter(|a| universe.catch_is_ancestor(&clause.type_id, &a.type_id))
        {
            let test = b.node(
                &frames,
                "evaluate",
                "pending_exception_is",
                vec![value.clone()],
                &arm.type_id,
                vec![],
            )?;
            let thrown = b.node(
                &frames,
                "closed_rethrow",
                &clause.id,
                vec![value.clone()],
                &arm.type_id,
                vec![],
            )?;
            b.graph
                .nodes
                .iter_mut()
                .find(|n| n.id == thrown)
                .unwrap()
                .exceptional_successors = original.exceptional_successors.clone();
            let branch = b.node(
                &frames,
                "branch",
                "",
                vec![b.value(&test)],
                "",
                vec![thrown.clone(), String::new()],
            )?;
            b.link(&test, &branch);
            tests.push((test, branch, thrown));
        }
        for i in 0..tests.len() {
            let next = if i + 1 < tests.len() {
                tests[i + 1].0.clone()
            } else {
                tests[i].2.clone()
            };
            b.graph
                .nodes
                .iter_mut()
                .find(|n| n.id == tests[i].1)
                .unwrap()
                .successors[1] = next;
        }
        let node = b
            .graph
            .nodes
            .iter_mut()
            .find(|n| n.id == original.id)
            .unwrap();
        node.kind = "jump".into();
        node.exceptional_successors.clear();
        node.successors = vec![tests.first().ok_or(DataPhaseError::Source)?.0.clone()];
    }
    Ok(())
}

pub(super) fn expand(
    graph: &mut LoopControlFunction,
    handler: &mut HandlerFunction,
    universe: &ClosedExceptionUniverse,
) -> Result<(), DataPhaseError> {
    if !handler
        .regions
        .iter()
        .any(|r| r.finally_entry.is_some() || r.catches.iter().any(|c| c.filter.is_some()))
    {
        return Ok(());
    }
    let transfers = handler.transfers.clone();
    let contexts = handler
        .contexts
        .iter()
        .map(|c| (c.node.clone(), c.frames.clone()))
        .collect::<BTreeMap<_, _>>();
    let regions = handler.regions.clone();
    let original_nodes = graph.nodes.clone();
    let graph_completion_count = graph
        .nodes
        .iter()
        .filter(|n| n.id.contains(".completion."))
        .count();
    let mut b = Builder {
        graph,
        handler,
        next: graph_completion_count,
    };
    let mut resumptions = BTreeMap::<String, Vec<Resume>>::new();
    let mut filter_routes = BTreeMap::<String, Vec<(String, String)>>::new();
    for transfer in transfers.iter().filter(|t| {
        t.filter_catch.is_none()
            && (!t.finally_entries.is_empty() || (t.kind == "throw" && !t.candidates.is_empty()))
    }) {
        let original = original_nodes
            .iter()
            .find(|n| n.id == transfer.node)
            .ok_or(DataPhaseError::Source)?;
        let frames = &contexts[&transfer.node];
        let destination = if transfer.kind == "throw" {
            let value = format!("{}.exception", original.id);
            let mut routes = BTreeMap::<String, String>::new();
            let mut arms = Vec::new();
            for arm in universe.arms() {
                if transfer.candidates.iter().any(|c| {
                    c.filter.is_some() && universe.catch_is_ancestor(&c.type_id, &arm.type_id)
                }) {
                    let route = search_route(
                        &mut b,
                        &mut resumptions,
                        &mut filter_routes,
                        transfer,
                        0,
                        &arm.type_id,
                        &value,
                        frames,
                        universe,
                    )?;
                    arms.push((arm.type_id.clone(), route));
                    continue;
                }
                let candidate = transfer
                    .candidates
                    .iter()
                    .find(|c| universe.catch_is_ancestor(&c.type_id, &arm.type_id));
                let finalies = candidate
                    .map(|c| c.finally_entries.as_slice())
                    .unwrap_or(&transfer.finally_entries);
                let target = candidate.map(|c| c.entry.clone());
                let key = format!(
                    "{}|{}",
                    target.as_deref().unwrap_or("escape"),
                    finalies.join("|")
                );
                let route = if let Some(id) = routes.get(&key) {
                    id.clone()
                } else {
                    let result = if finalies.is_empty() {
                        if let Some(target) = &target {
                            b.node(
                                frames,
                                "handler_restore",
                                "",
                                vec![value.clone()],
                                "",
                                vec![target.clone()],
                            )?
                        } else {
                            b.node(
                                frames,
                                "completion_throw",
                                "",
                                vec![value.clone()],
                                &arm.type_id,
                                vec![],
                            )?
                        }
                    } else {
                        setup(
                            &mut b,
                            &mut resumptions,
                            frames,
                            finalies,
                            Resume {
                                kind: if target.is_some() {
                                    "catch".into()
                                } else {
                                    "throw".into()
                                },
                                target,
                                loop_id: String::new(),
                                next_finally: None,
                            },
                            None,
                            Some(value.clone()),
                        )?
                    };
                    // An escaping throw without finally needs its exact arm.
                    if !finalies.is_empty() || candidate.is_some() {
                        routes.insert(key, result.clone());
                    }
                    result
                };
                arms.push((arm.type_id.clone(), route));
            }
            let unique = arms.iter().map(|(_, d)| d).collect::<BTreeSet<_>>();
            if unique.len() == 1 {
                arms[0].1.clone()
            } else {
                let mut tests = Vec::new();
                for (ty, destination) in arms {
                    let test = b.node(
                        frames,
                        "evaluate",
                        "pending_exception_is",
                        vec![value.clone()],
                        &ty,
                        vec![],
                    )?;
                    let branch = b.node(
                        frames,
                        "branch",
                        "",
                        vec![b.value(&test)],
                        "",
                        vec![destination.clone(), String::new()],
                    )?;
                    b.link(&test, &branch);
                    tests.push((test, branch, destination));
                }
                for i in 0..tests.len() {
                    let next = if i + 1 < tests.len() {
                        tests[i + 1].0.clone()
                    } else {
                        tests[i].2.clone()
                    };
                    b.graph
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == tests[i].1)
                        .unwrap()
                        .successors[1] = next;
                }
                tests[0].0.clone()
            }
        } else {
            setup(
                &mut b,
                &mut resumptions,
                frames,
                &transfer.finally_entries,
                Resume {
                    kind: transfer.kind.clone(),
                    target: transfer.target.clone(),
                    loop_id: original.slot.clone(),
                    next_finally: None,
                },
                original
                    .inputs
                    .first()
                    .filter(|_| transfer.kind == "return")
                    .cloned(),
                None,
            )?
        };
        let node = b
            .graph
            .nodes
            .iter_mut()
            .find(|n| n.id == transfer.node)
            .unwrap();
        node.kind = if transfer.kind == "throw" {
            "handler_landing".into()
        } else {
            "jump".into()
        };
        node.operation.clear();
        node.inputs.clear();
        node.result.clear();
        node.successors = vec![destination];
    }
    for (catch, routes) in &filter_routes {
        let clause = regions
            .iter()
            .flat_map(|r| &r.catches)
            .find(|c| c.id == *catch)
            .ok_or(DataPhaseError::Source)?;
        let entry = clause.filter.as_ref().ok_or(DataPhaseError::Source)?;
        let result = original_nodes
            .iter()
            .find(|n| n.kind == "handler_filter_result" && n.slot == *catch)
            .ok_or(DataPhaseError::Source)?;
        let frames = &contexts[&result.id];
        let slot = format!("{entry}.search_selector");
        let yes = filter_dispatch(
            &mut b,
            frames,
            &slot,
            &routes.iter().map(|r| r.0.clone()).collect::<Vec<_>>(),
        )?;
        let no = filter_dispatch(
            &mut b,
            frames,
            &slot,
            &routes.iter().map(|r| r.1.clone()).collect::<Vec<_>>(),
        )?;
        let result = b
            .graph
            .nodes
            .iter_mut()
            .find(|n| n.id == result.id)
            .unwrap();
        result.operation = "expanded_filter".into();
        result.successors = vec![yes, no];
    }
    for original in original_nodes.iter().filter(|n| n.kind == "handler_resume") {
        let frames = &contexts[&original.id];
        let frame = frames.last().ok_or(DataPhaseError::Source)?;
        let entry = regions
            .iter()
            .find(|r| r.id == frame.region)
            .and_then(|r| r.finally_entry.as_ref())
            .ok_or(DataPhaseError::Source)?;
        let Some(resumes) = resumptions.get(entry) else {
            continue;
        };
        let outside = &frames[..frames.len() - 1];
        let prefix = format!("{entry}.pending");
        let load = b.node(
            outside,
            "evaluate",
            "load",
            vec![],
            &format!("{prefix}.kind"),
            vec![],
        )?;
        let selector_value = b.value(&load);
        let mut tests = Vec::new();
        for (index, resume) in resumes.iter().enumerate() {
            let destination = if let Some(next) = &resume.next_finally {
                next.clone()
            } else if resume.kind == "normal" {
                resume.target.clone().ok_or(DataPhaseError::Source)?
            } else if resume.kind == "return" {
                let read = b.node(
                    outside,
                    "evaluate",
                    "load",
                    vec![],
                    &format!("{prefix}.return"),
                    vec![],
                )?;
                let unwrap = b.node(
                    outside,
                    "evaluate",
                    "pending_unwrap",
                    vec![b.value(&read)],
                    "",
                    vec![],
                )?;
                let done = b.node(outside, "return", "", vec![b.value(&unwrap)], "", vec![])?;
                b.link(&read, &unwrap);
                b.link(&unwrap, &done);
                read
            } else if matches!(resume.kind.as_str(), "break" | "continue") {
                b.node(
                    outside,
                    &resume.kind,
                    "",
                    vec![],
                    &resume.loop_id,
                    vec![resume.target.clone().ok_or(DataPhaseError::Source)?],
                )?
            } else if resume.kind == "catch" {
                let read = b.node(
                    outside,
                    "evaluate",
                    "load",
                    vec![],
                    &format!("{prefix}.exception"),
                    vec![],
                )?;
                let restore = b.node(
                    outside,
                    "handler_restore",
                    "",
                    vec![b.value(&read)],
                    "",
                    vec![resume.target.clone().ok_or(DataPhaseError::Source)?],
                )?;
                b.link(&read, &restore);
                read
            } else if resume.kind == "throw" {
                let read = b.node(
                    outside,
                    "evaluate",
                    "load",
                    vec![],
                    &format!("{prefix}.exception"),
                    vec![],
                )?;
                let value = b.value(&read);
                let mut tags = Vec::new();
                for arm in universe.arms() {
                    let test = b.node(
                        outside,
                        "evaluate",
                        "pending_exception_is",
                        vec![value.clone()],
                        &arm.type_id,
                        vec![],
                    )?;
                    let thrown = b.node(
                        outside,
                        "completion_throw",
                        "",
                        vec![value.clone()],
                        &arm.type_id,
                        vec![],
                    )?;
                    let branch = b.node(
                        outside,
                        "branch",
                        "",
                        vec![b.value(&test)],
                        "",
                        vec![thrown.clone(), String::new()],
                    )?;
                    b.link(&test, &branch);
                    tags.push((test, branch, thrown));
                }
                for i in 0..tags.len() {
                    let next = if i + 1 < tags.len() {
                        tags[i + 1].0.clone()
                    } else {
                        tags[i].2.clone()
                    };
                    b.graph
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == tags[i].1)
                        .unwrap()
                        .successors[1] = next;
                }
                b.link(&read, &tags[0].0);
                read
            } else {
                return Err(DataPhaseError::Emission);
            };
            let test = b.node(
                outside,
                "evaluate",
                "pending_is_kind",
                vec![selector_value.clone()],
                &index.to_string(),
                vec![],
            )?;
            let branch = b.node(
                outside,
                "branch",
                "",
                vec![b.value(&test)],
                "",
                vec![destination.clone(), String::new()],
            )?;
            b.link(&test, &branch);
            tests.push((test, branch, destination));
        }
        for i in 0..tests.len() {
            let next = if i + 1 < tests.len() {
                tests[i + 1].0.clone()
            } else {
                tests[i].2.clone()
            };
            b.graph
                .nodes
                .iter_mut()
                .find(|n| n.id == tests[i].1)
                .unwrap()
                .successors[1] = next;
        }
        b.link(&load, &tests[0].0);
        let node = b
            .graph
            .nodes
            .iter_mut()
            .find(|n| n.id == original.id)
            .unwrap();
        node.kind = "handler_finally_exit".into();
        node.successors = vec![load];
    }
    // Every dispatch slot has a defined entry value. Correlated completion
    // branches may join before a later setup overwrites a slot; these inert
    // defaults prevent such joins from manufacturing undefined SSA operands.
    let mut initial = Vec::new();
    for entry in resumptions.keys() {
        let prefix = format!("{entry}.pending");
        initial.push(b.save(&[], &format!("{prefix}.kind"), "pending_kind", vec![], "-1")?);
        initial.push(b.save(&[], &format!("{prefix}.return"), "pending_none", vec![], "")?);
        initial.push(b.save(
            &[],
            &format!("{prefix}.exception"),
            "pending_exception_default",
            vec![],
            "",
        )?);
    }
    for catch in filter_routes.keys() {
        let entry = regions
            .iter()
            .flat_map(|r| &r.catches)
            .find(|c| c.id == *catch)
            .and_then(|c| c.filter.as_ref())
            .ok_or(DataPhaseError::Source)?;
        initial.push(b.save(
            &[],
            &format!("{entry}.search_selector"),
            "pending_kind",
            vec![],
            "-1",
        )?);
    }
    if !initial.is_empty() {
        for pair in initial.windows(2) {
            b.link(&pair[0].1, &pair[1].0);
        }
        let next = b.graph.nodes[0].successors[0].clone();
        b.link(&initial.last().unwrap().1, &next);
        b.graph.nodes[0].successors = vec![initial[0].0.clone()];
    }
    if b.graph.nodes.len() > 8192 {
        return Err(DataPhaseError::Emission);
    }
    Ok(())
}
