//! Original CLR execution and instrumented search/unwind traces versus the
//! private register CFG and native handler planner. No source interpretation.
use super::lowering::{evaluate, R};
use super::*;
use mpk_vc::hash::sha256_raw_file_bytes;
use std::collections::{BTreeMap, VecDeque};
fn fixtures() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/handler-lowering/source-cases.json",
    ))
    .unwrap()
}
fn universe(case: &Value) -> ClosedExceptionUniverse {
    let b = bundle();
    let (r, c) = super::exceptions::exception_roots(&b, case);
    let definitions = case["lowering"]["exception_definitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            let id = d["type_id"].as_str().unwrap();
            SourceExceptionDefinition {
                type_id: id.into(),
                sealed: true,
                direct_base_type_id: "System.Exception".into(),
                payload_member_ids: vec![csharp_practical_stored_member_id(
                    id,
                    "Code",
                    &json!({"kind":"primitive","id":"i32"}),
                    if case["id"] == "get_only" {
                        "get_auto"
                    } else if case["id"] == "init_only" {
                        "init_auto"
                    } else {
                        "readonly_field"
                    },
                )
                .unwrap()],
            }
        })
        .collect::<Vec<_>>();
    derive_closed_exception_universe(&r, &c, &definitions).unwrap()
}
fn functions(case: &Value) -> (Vec<LoopControlFunction>, Vec<PreparedHandlerFunction>) {
    let universe = universe(case);
    let functions: Vec<LoopControlFunction> =
        serde_json::from_value(case["lowering"]["functions"].clone()).unwrap();
    let graphs: Vec<HandlerFunction> =
        serde_json::from_value(case["lowering"]["handlers"].clone()).unwrap();
    let prepared = functions
        .iter()
        .zip(graphs)
        .map(|(f, g)| {
            prepare_handler_function(f, g, &universe)
                .unwrap_or_else(|e| panic!("{}:{e:?}", case["id"]))
        })
        .collect();
    (functions, prepared)
}
struct Frame {
    function: usize,
    pc: String,
    parent: Option<usize>,
    slots: BTreeMap<String, R>,
    values: BTreeMap<String, R>,
    caught: BTreeMap<String, (String, R)>,
}
#[derive(Clone)]
enum Destination {
    Jump(HandlerLocation),
    Return(usize, Option<R>),
    Catch {
        frame: usize,
        id: String,
        entry: String,
        local: String,
        exception: (String, R),
    },
    Propagate(String, R),
    FilterFailure {
        catch_id: String,
        discarded_type: String,
    },
}
struct Cleanup {
    rest: VecDeque<HandlerLocation>,
    destination: Destination,
    current: HandlerLocation,
    saved_pc: String,
}
struct Filter {
    search: HandlerStackSearch<R>,
    frame: usize,
    saved_pc: String,
    id: String,
}
struct Execution<'a> {
    functions: &'a [LoopControlFunction],
    graphs: &'a [PreparedHandlerFunction],
    frames: Vec<Frame>,
    current: usize,
    filters: Vec<Filter>,
    cleanups: Vec<Cleanup>,
    trace: Vec<String>,
    result: Option<(Option<i64>, String, Option<i64>)>,
    exception: Option<(String, R)>,
}
impl<'a> Execution<'a> {
    fn node(&self, frame: usize) -> &LoopControlNode {
        let f = &self.frames[frame];
        self.functions[f.function]
            .nodes
            .iter()
            .find(|n| n.id == f.pc)
            .unwrap()
    }
    fn location(frame: usize, node: String) -> HandlerLocation {
        HandlerLocation { frame, node }
    }
    fn type_name(type_id: &str) -> String {
        if type_id.starts_with("System.") {
            type_id.into()
        } else {
            "Business.Fault".into()
        }
    }
    fn event_id(&self, frame: usize, catch: &str) -> String {
        let g = self.graphs[self.frames[frame].function].graph();
        for (i, r) in g.regions.iter().enumerate() {
            for (j, c) in r.catches.iter().enumerate() {
                if c.id == catch {
                    return format!("{i}:{j}");
                }
            }
        }
        panic!("catch identity")
    }
    fn destination(&mut self, destination: Destination) {
        match destination {
            Destination::Jump(loc) => {
                self.current = loc.frame;
                self.frames[loc.frame].pc = loc.node;
            }
            Destination::Catch {
                frame,
                id,
                entry,
                local,
                exception,
            } => {
                self.frames[frame].caught.insert(id, exception.clone());
                if !local.is_empty() {
                    self.frames[frame].slots.insert(local, exception.1);
                }
                self.current = frame;
                self.frames[frame].pc = entry;
            }
            Destination::Return(frame, value) => {
                if let Some(parent) = self.frames[frame].parent {
                    let node = self.node(parent).clone();
                    self.frames[parent]
                        .values
                        .insert(node.result, value.unwrap_or(R::Null));
                    self.frames[parent].pc = node.successors[0].clone();
                    self.current = parent;
                } else {
                    self.result = Some((value.and_then(|v| v.number().ok()), String::new(), None));
                }
            }
            Destination::FilterFailure {
                catch_id,
                discarded_type,
            } => {
                let filter = self.filters.pop().unwrap();
                assert_eq!(filter.id, catch_id);
                self.trace.push(format!(
                    "filter:{}:throw:{}",
                    self.event_id(filter.frame, &filter.id),
                    Self::type_name(&discarded_type)
                ));
                self.frames[filter.frame].pc = filter.saved_pc;
                self.search_step(filter.search, Some(FilterOutcome::Threw(discarded_type)));
            }
            Destination::Propagate(t, v) => {
                self.result = Some((
                    None,
                    Self::type_name(&t),
                    if t.starts_with("System.") {
                        None
                    } else {
                        v.number().ok()
                    },
                ));
            }
        }
    }
    fn unwind(&mut self, entries: Vec<HandlerLocation>, destination: Destination) {
        let mut rest = VecDeque::from(entries);
        if let Some(current) = rest.pop_front() {
            let saved_pc = self.frames[current.frame].pc.clone();
            self.frames[current.frame].pc = current.node.clone();
            self.current = current.frame;
            self.cleanups.push(Cleanup {
                rest,
                destination,
                current,
                saved_pc,
            });
        } else {
            self.destination(destination);
        }
    }
    fn search_step(&mut self, mut search: HandlerStackSearch<R>, outcome: Option<FilterOutcome>) {
        let step = search.advance(outcome).unwrap();
        match step.step {
            HandlerSearchStep::Filter {
                catch_id,
                entry,
                local,
                value,
                ..
            } => {
                let frame = step.frame;
                let saved_pc = self.frames[frame].pc.clone();
                if !local.is_empty() {
                    self.frames[frame].slots.insert(local, value);
                }
                self.trace
                    .push(format!("filter:{}:enter", self.event_id(frame, &catch_id)));
                self.filters.push(Filter {
                    search,
                    frame,
                    saved_pc,
                    id: catch_id,
                });
                self.frames[frame].pc = entry;
                self.current = frame;
            }
            HandlerSearchStep::FilterFalse {
                catch_id,
                discarded_type,
            } => {
                let filter = self.filters.last().unwrap();
                let node = self.graphs[self.frames[filter.frame].function]
                    .graph()
                    .regions
                    .iter()
                    .flat_map(|r| &r.catches)
                    .find(|c| c.id == filter.id)
                    .unwrap()
                    .filter
                    .clone()
                    .unwrap();
                self.drop_exited_cleanups(Some(Self::location(filter.frame, node)));
                self.unwind(
                    step.finally_entries,
                    Destination::FilterFailure {
                        catch_id,
                        discarded_type,
                    },
                );
            }
            HandlerSearchStep::Selected {
                catch_id,
                entry,
                local,
                exception_type,
                value,
                ..
            } => {
                // A new throw supersedes only cleanup continuations it exits;
                // an inner catch inside the same finally preserves its parent.
                let destination = Destination::Catch {
                    frame: step.frame,
                    id: catch_id,
                    entry: entry.clone(),
                    local,
                    exception: (exception_type, value),
                };
                self.drop_exited_cleanups(Some(Self::location(step.frame, entry)));
                self.unwind(step.finally_entries, destination);
            }
            HandlerSearchStep::Propagate {
                exception_type,
                value,
                ..
            } => {
                self.drop_exited_cleanups(None);
                self.unwind(
                    step.finally_entries,
                    Destination::Propagate(exception_type, value),
                );
            }
        }
    }
    fn drop_exited_cleanups(&mut self, destination: Option<HandlerLocation>) {
        self.cleanups.retain(|cleanup| {
            let Some(target) = &destination else {
                return false;
            };
            // Follow the selected handler's call parents to the cleanup frame.
            let mut frame = target.frame;
            let mut node = target.node.as_str();
            loop {
                if frame == cleanup.current.frame {
                    let graph = self.graphs[self.frames[frame].function].graph();
                    let origin = graph
                        .contexts
                        .iter()
                        .find(|c| c.node == cleanup.current.node)
                        .unwrap();
                    let context = graph.contexts.iter().find(|c| c.node == node).unwrap();
                    return context.frames.starts_with(&origin.frames);
                }
                let Some(parent) = self.frames[frame].parent else {
                    return false;
                };
                frame = parent;
                node = &self.frames[frame].pc;
            }
        });
    }
    fn begin_search(&mut self) {
        let (t, v) = self.exception.take().unwrap();
        let mut frame = self.current;
        let mut node = self.frames[frame].pc.clone();
        let mut searches = vec![];
        loop {
            searches.push((
                frame,
                self.graphs[self.frames[frame].function]
                    .search(&node, &t, v.clone())
                    .unwrap(),
            ));
            let Some(parent) = self.frames[frame].parent else {
                break;
            };
            frame = parent;
            node = self.node(frame).exceptional_successors[0].clone();
        }
        self.search_step(HandlerStackSearch::new(searches).unwrap(), None);
    }
    fn run(mut self) -> (Option<i64>, String, Option<i64>, Vec<String>) {
        for _ in 0..20000 {
            if let Some((v, e, c)) = self.result.take() {
                return (v, e, c, self.trace);
            }
            let frame = self.current;
            let function = self.frames[frame].function;
            let node = self.node(frame).clone();
            let inputs = node
                .inputs
                .iter()
                .map(|id| self.frames[frame].values[id].clone())
                .collect::<Vec<_>>();
            let graph = self.graphs[function].graph();
            if let Some(i) = graph.regions.iter().position(|r| r.try_entry == node.id) {
                self.trace.push(format!("try:{i}"));
            }
            match node.kind.as_str() {
                "handler_search" => {
                    self.begin_search();
                    continue;
                }
                "builtin_throw" => {
                    self.exception = Some((node.slot.clone(), R::Null));
                    self.frames[frame].pc = node.exceptional_successors[0].clone();
                    continue;
                }
                "explicit_throw" => {
                    self.exception = Some((node.slot.clone(), inputs[0].clone()));
                    self.frames[frame].pc = node.exceptional_successors[0].clone();
                    continue;
                }
                "rethrow" => {
                    self.exception = Some(
                        self.graphs[function]
                            .rethrow(&node, &self.frames[frame].caught)
                            .unwrap(),
                    );
                    self.frames[frame].pc = node.exceptional_successors[0].clone();
                    continue;
                }
                "handler_entry" => {
                    let caught = graph
                        .regions
                        .iter()
                        .flat_map(|r| &r.catches)
                        .find(|c| c.entry == node.id)
                        .unwrap();
                    self.trace
                        .push(format!("catch:{}", self.event_id(frame, &caught.id)));
                }
                "handler_finally_entry" => {
                    let i = graph
                        .regions
                        .iter()
                        .position(|r| r.finally_entry.as_ref() == Some(&node.id))
                        .unwrap();
                    self.trace.push(format!("finally:{i}"));
                }
                "handler_filter_result" => {
                    let accepted = inputs[0].boolean().unwrap();
                    let filter = self.filters.pop().unwrap();
                    self.trace.push(format!(
                        "filter:{}:{accepted}",
                        self.event_id(frame, &filter.id)
                    ));
                    self.frames[filter.frame].pc = filter.saved_pc;
                    self.search_step(filter.search, Some(FilterOutcome::Boolean(accepted)));
                    continue;
                }
                "handler_completion" => {
                    let transfer = self.graphs[function].transfer(&node.id).unwrap();
                    let destination = if transfer.kind == "return" {
                        Destination::Return(frame, inputs.first().cloned())
                    } else {
                        Destination::Jump(Self::location(frame, transfer.target.clone().unwrap()))
                    };
                    self.unwind(
                        transfer
                            .finally_entries
                            .iter()
                            .map(|n| Self::location(frame, n.clone()))
                            .collect(),
                        destination,
                    );
                    continue;
                }
                "handler_resume" => {
                    let mut cleanup = self.cleanups.pop().unwrap();
                    self.frames[cleanup.current.frame].pc = cleanup.saved_pc;
                    if let Some(next) = cleanup.rest.pop_front() {
                        cleanup.saved_pc = self.frames[next.frame].pc.clone();
                        self.frames[next.frame].pc = next.node.clone();
                        self.current = next.frame;
                        cleanup.current = next;
                        self.cleanups.push(cleanup);
                    } else {
                        self.destination(cleanup.destination);
                    }
                    continue;
                }
                "branch" | "loop_header" => {
                    self.frames[frame].pc =
                        node.successors[usize::from(!inputs[0].boolean().unwrap())].clone();
                    continue;
                }
                "evaluate" => {
                    let source = node
                        .source_ordinal
                        .map(|i| &self.functions[function].operations[i]);
                    if node.operation == "call" {
                        if let Some(callee) = self
                            .functions
                            .iter()
                            .position(|f| f.callable_id == source.unwrap().symbol)
                        {
                            let slots = inputs
                                .into_iter()
                                .enumerate()
                                .map(|(i, v)| (format!("parameter:{i}"), v))
                                .collect();
                            self.frames.push(Frame {
                                function: callee,
                                pc: self.functions[callee].nodes[0].id.clone(),
                                parent: Some(frame),
                                slots,
                                values: BTreeMap::new(),
                                caught: BTreeMap::new(),
                            });
                            self.current = self.frames.len() - 1;
                            continue;
                        }
                    }
                    let value = match node.operation.as_str() {
                        "closed_exception" | "construct" => {
                            Ok(inputs.first().cloned().unwrap_or(R::Null))
                        }
                        "exception_payload" if source.unwrap().symbol.ends_with(".Code") => {
                            Ok(inputs[0].clone())
                        }
                        _ => evaluate(&node, source, &inputs, &mut self.frames[frame].slots),
                    };
                    match value {
                        Ok(value) => {
                            self.frames[frame].values.insert(node.result, value);
                        }
                        Err(e) => {
                            self.exception = Some((format!("System.{e}"), R::Null));
                            self.frames[frame].pc = node.exceptional_successors[0].clone();
                            continue;
                        }
                    }
                }
                "entry" | "jump" | "handler_filter_entry" | "pattern_decision" => {}
                kind => panic!("{kind}"),
            }
            self.frames[frame].pc = node.successors[0].clone();
        }
        panic!("execution budget")
    }
}
#[test]
fn csharp_03_t04_w05_original_clr_and_two_pass_traces() {
    let mut count = 0;
    for case in fixtures().iter().filter(|c| c["accepted"] == true) {
        let (functions, graphs) = functions(case);
        assert!(graphs.iter().all(|g| g.artifact_count() == 0));
        let root = functions
            .iter()
            .position(|f| f.callable_id == case["root"].as_str().unwrap())
            .unwrap();
        for run in case["runs"].as_array().unwrap() {
            let frame = Frame {
                function: root,
                pc: functions[root].nodes[0].id.clone(),
                parent: None,
                slots: BTreeMap::from([(
                    "parameter:0".into(),
                    R::Number(run["n"].as_i64().unwrap()),
                )]),
                values: BTreeMap::new(),
                caught: BTreeMap::new(),
            };
            let execution = Execution {
                functions: &functions,
                graphs: &graphs,
                frames: vec![frame],
                current: 0,
                filters: vec![],
                cleanups: vec![],
                trace: vec![],
                result: None,
                exception: None,
            };
            assert_eq!(
                execution.run(),
                (
                    run["value"].as_i64(),
                    run["error"].as_str().unwrap().into(),
                    run["code"].as_i64(),
                    serde_json::from_value::<Vec<String>>(run["trace"].clone()).unwrap()
                ),
                "{}:{run}",
                case["id"]
            );
            count += 1;
        }
    }
    assert_eq!(count, 145);
}

#[test]
fn csharp_03_t04_w05_region_edge_order_and_filter_mutations() {
    let cases = fixtures();
    let mut mutations = 0;
    for case in cases.iter().filter(|c| c["accepted"] == true) {
        let universe = universe(case);
        let (functions, graphs) = functions(case);
        for (f, g) in functions.iter().zip(graphs) {
            for (i, t) in g.graph().transfers.iter().enumerate() {
                if !t.candidates.is_empty() {
                    let mut changed = g.graph().clone();
                    changed.transfers[i].candidates[0].type_id =
                        "System.OutOfMemoryException".into();
                    assert!(prepare_handler_function(f, changed, &universe).is_err());
                    mutations += 1;
                    let mut changed = g.graph().clone();
                    changed.transfers[i].candidates.remove(0);
                    assert!(prepare_handler_function(f, changed, &universe).is_err());
                    mutations += 1;
                }
                if !t.finally_entries.is_empty() {
                    let mut changed = g.graph().clone();
                    changed.transfers[i].finally_entries.clear();
                    assert!(prepare_handler_function(f, changed, &universe).is_err());
                    mutations += 1;
                }
                if t.candidates.len() > 1 {
                    let mut changed = g.graph().clone();
                    changed.transfers[i].candidates.swap(0, 1);
                    assert!(prepare_handler_function(f, changed, &universe).is_err());
                    mutations += 1;
                }
            }
            for (i, n) in f.nodes.iter().enumerate() {
                if n.kind == "handler_search" && !n.successors.is_empty() {
                    let mut changed = f.clone();
                    changed.nodes[i].successors.clear();
                    assert!(
                        prepare_handler_function(&changed, g.graph().clone(), &universe).is_err()
                    );
                    mutations += 1;
                }
                if n.operation == "exception_payload" {
                    let mut changed = f.clone();
                    changed.nodes[i].exceptional_successors = vec![f.nodes[0].id.clone()];
                    assert_eq!(
                        prepare_handler_function(&changed, g.graph().clone(), &universe)
                            .unwrap_err(),
                        HandlerError::ExceptionType
                    );
                    mutations += 1;
                    let mut changed = f.clone();
                    changed.nodes[i].slot = "System.ArgumentException.Message".into();
                    assert_eq!(
                        prepare_handler_function(&changed, g.graph().clone(), &universe)
                            .unwrap_err(),
                        HandlerError::ExceptionType
                    );
                    mutations += 1;
                }
                if n.kind == "handler_filter_result" {
                    let mut changed = f.clone();
                    let producer = changed
                        .nodes
                        .iter()
                        .find(|p| p.result == n.inputs[0])
                        .unwrap()
                        .source_ordinal
                        .unwrap();
                    changed.operations[producer].type_key =
                        Some("23:mpk.csharp.value.i32.v15:value0:".into());
                    assert_eq!(
                        prepare_handler_function(&changed, g.graph().clone(), &universe)
                            .unwrap_err(),
                        HandlerError::FilterResult
                    );
                    mutations += 1;
                }
                if n.kind == "rethrow" {
                    assert!(g.rethrow::<i32>(n, &BTreeMap::new()).is_err());
                    let catch_type = g
                        .graph()
                        .regions
                        .iter()
                        .flat_map(|r| &r.catches)
                        .find(|c| c.id == n.slot)
                        .unwrap()
                        .type_id
                        .clone();
                    let active = BTreeMap::from([(n.slot.clone(), (catch_type.clone(), 17))]);
                    assert_eq!(g.rethrow(n, &active).unwrap(), (catch_type, 17));
                    let invalid = BTreeMap::from([(
                        n.slot.clone(),
                        ("System.NullReferenceException".into(), 17),
                    )]);
                    assert_eq!(
                        g.rethrow(n, &invalid).unwrap_err(),
                        HandlerError::ExceptionType
                    );
                    let mut changed = f.clone();
                    changed.nodes[i].slot = "wrong.catch".into();
                    assert_eq!(
                        prepare_handler_function(&changed, g.graph().clone(), &universe)
                            .unwrap_err(),
                        HandlerError::InactiveRethrow
                    );
                    mutations += 1;
                }
            }
        }
    }
    assert!(mutations > 100);
}
#[test]
fn csharp_03_t04_w05_finally_completion_preserves_values_and_replaces_throws() {
    let incoming = [
        HandlerCompletion::Normal,
        HandlerCompletion::Return(Some(17)),
        HandlerCompletion::Break("loop.1".into()),
        HandlerCompletion::Continue("loop.2".into()),
        HandlerCompletion::Throw {
            exception_type: "System.ArgumentException".into(),
            value: 23,
        },
    ];
    for value in incoming {
        assert_eq!(
            complete_handler_finally(value.clone(), HandlerCompletion::Normal).unwrap(),
            value
        );
        let thrown = HandlerCompletion::Throw {
            exception_type: "System.InvalidOperationException".into(),
            value: 91,
        };
        assert_eq!(
            complete_handler_finally(value.clone(), thrown.clone()).unwrap(),
            thrown
        );
        for forbidden in [
            HandlerCompletion::Return(Some(18)),
            HandlerCompletion::Break("outward".into()),
            HandlerCompletion::Continue("outward".into()),
        ] {
            assert_eq!(
                complete_handler_finally(value.clone(), forbidden).unwrap_err(),
                HandlerError::FinallyAbrupt
            );
        }
    }
}
#[test]
fn csharp_03_t04_w05_rejection_barriers_are_artifact_free() {
    let cases = fixtures();
    let rejected = cases
        .iter()
        .filter(|c| c["accepted"] == false)
        .collect::<Vec<_>>();
    assert_eq!(rejected.len(), 13);
    for case in rejected {
        assert!(case["lowering"].is_null());
        assert_eq!(case["runs"], json!([]));
        assert!(!case["diagnostic"].as_str().unwrap().is_empty());
    }
    for id in ["filter_write", "filter_array_write"] {
        assert_eq!(
            cases.iter().find(|c| c["id"] == id).unwrap()["code"],
            "exception_filter_impure"
        );
    }
    for id in ["payload_identity", "payload_message", "payload_escape"] {
        assert_eq!(
            cases.iter().find(|c| c["id"] == id).unwrap()["code"],
            "exception_payload_only"
        );
    }
}

#[test]
fn csharp_03_t04_w05_search_subject_and_filter_protocol_are_closed() {
    let cases = fixtures();
    let case = cases
        .iter()
        .find(|c| c["id"] == "call_filter_before_finally")
        .unwrap();
    let (functions, graphs) = functions(case);
    let root = functions
        .iter()
        .position(|f| f.callable_id == case["root"].as_str().unwrap())
        .unwrap();
    let inner = (0..graphs.len())
        .find(|i| *i != root && !graphs[*i].graph().regions.is_empty())
        .unwrap();
    let inner_node = &graphs[inner]
        .graph()
        .transfers
        .iter()
        .find(|t| t.kind == "throw" && !t.finally_entries.is_empty())
        .unwrap()
        .node;
    let outer_node = &graphs[root]
        .graph()
        .transfers
        .iter()
        .find(|t| t.kind == "throw" && !t.candidates.is_empty())
        .unwrap()
        .node;
    let mut stack = HandlerStackSearch::new(vec![
        (
            1,
            graphs[inner]
                .search(inner_node, "System.ArgumentException", 17)
                .unwrap(),
        ),
        (
            0,
            graphs[root]
                .search(outer_node, "System.ArgumentException", 99)
                .unwrap(),
        ),
    ])
    .unwrap();
    let step = stack.advance(None).unwrap();
    assert!(step.finally_entries.is_empty());
    assert!(matches!(
        step.step,
        HandlerSearchStep::Filter { value: 17, .. }
    ));
    assert_eq!(stack.advance(None).unwrap_err(), HandlerError::FilterResult);
    assert_eq!(
        stack
            .advance(Some(FilterOutcome::Threw(
                "System.OutOfMemoryException".into()
            )))
            .unwrap_err(),
        HandlerError::ExceptionType
    );
    let selected = stack
        .advance(Some(FilterOutcome::Threw(
            "System.InvalidOperationException".into(),
        )))
        .unwrap();
    assert!(
        matches!(selected.step,HandlerSearchStep::Selected{value:17,exception_type,..}if exception_type=="System.ArgumentException")
    );
    assert_eq!(selected.finally_entries.len(), 1);
    assert_eq!(selected.finally_entries[0].frame, 1);
    assert!(stack.advance(None).is_err());
    assert!(HandlerStackSearch::new(vec![
        (
            1,
            graphs[inner]
                .search(inner_node, "System.ArgumentException", 17)
                .unwrap()
        ),
        (
            0,
            graphs[root]
                .search(outer_node, "System.OverflowException", 17)
                .unwrap()
        )
    ])
    .is_err());
}
#[test]
fn csharp_03_t04_w05_conformance_binds_sources_graphs_and_pinned_inputs() {
    let conformance: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/handler-lowering/conformance.json",
    ))
    .unwrap();
    assert_eq!(conformance["work_item"], "CSHARP-03-T04-W05");
    for key in ["source_cases", "frozen_probe", "input_manifest"] {
        let binding = &conformance[key];
        let bytes = read(binding["path"].as_str().unwrap());
        assert_eq!(binding["sha256"], sha256_raw_file_bytes(&bytes).to_hex());
        assert_eq!(binding["size_bytes"], bytes.len());
    }
    let cases = fixtures();
    assert_eq!(cases.len(), conformance["cases"].as_array().unwrap().len());
    for (case, binding) in cases.iter().zip(conformance["cases"].as_array().unwrap()) {
        assert_eq!(case["id"], binding["id"]);
        assert_eq!(
            binding["source_sha256"],
            sha256_raw_file_bytes(case["source"].as_str().unwrap().as_bytes()).to_hex()
        );
        if case["accepted"] == true {
            assert_eq!(
                binding["graph_sha256"],
                sha256_raw_file_bytes(&serde_json::to_vec(&case["lowering"]).unwrap()).to_hex()
            );
            assert_eq!(
                binding["runs_sha256"],
                sha256_raw_file_bytes(&serde_json::to_vec(&case["runs"]).unwrap()).to_hex()
            );
        }
    }
    let inputs: Value = serde_json::from_slice(&read(
        conformance["input_manifest"]["path"].as_str().unwrap(),
    ))
    .unwrap();
    for file in inputs["files"].as_array().unwrap() {
        let bytes = read(file["path"].as_str().unwrap());
        assert_eq!(file["sha256"], sha256_raw_file_bytes(&bytes).to_hex());
        assert_eq!(file["size_bytes"], bytes.len());
    }
}
