//! W05 private two-pass exception search and completion handoff. W06 owns
//! whole-VIR import, ownership composition, and source-map closure.
use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerFrame {
    pub region: String,
    pub zone: String,
    pub catch_id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerClause {
    pub id: String,
    pub type_id: String,
    pub local: String,
    pub filter: Option<String>,
    pub entry: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerRegion {
    pub id: String,
    pub source_ordinal: usize,
    pub try_entry: String,
    pub exit: String,
    pub finally_entry: Option<String>,
    pub catches: Vec<HandlerClause>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerContext {
    pub node: String,
    pub frames: Vec<HandlerFrame>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerCandidate {
    pub catch_id: String,
    pub type_id: String,
    pub filter: Option<String>,
    pub entry: String,
    pub local: String,
    pub finally_entries: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerTransfer {
    pub node: String,
    pub kind: String,
    pub target: Option<String>,
    pub candidates: Vec<HandlerCandidate>,
    pub finally_entries: Vec<String>,
    pub filter_catch: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerFunction {
    pub callable_id: String,
    pub regions: Vec<HandlerRegion>,
    pub contexts: Vec<HandlerContext>,
    pub transfers: Vec<HandlerTransfer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandlerError {
    Shape,
    SearchOrder,
    UnwindOrder,
    ExceptionType,
    FilterResult,
    InactiveRethrow,
    FinallyAbrupt,
}
#[derive(Clone, Debug)]
pub struct PreparedHandlerFunction {
    graph: HandlerFunction,
    universe: ClosedExceptionUniverse,
}
impl PreparedHandlerFunction {
    pub fn graph(&self) -> &HandlerFunction {
        &self.graph
    }
    pub fn artifact_count(&self) -> usize {
        0
    }
    pub fn transfer(&self, node: &str) -> Result<&HandlerTransfer, HandlerError> {
        self.graph
            .transfers
            .iter()
            .find(|t| t.node == node)
            .ok_or(HandlerError::Shape)
    }
    pub fn search<V: Clone>(
        &self,
        node: &str,
        exception_type: &str,
        value: V,
    ) -> Result<HandlerSearch<V>, HandlerError> {
        let transfer = self.transfer(node)?;
        if transfer.kind != "throw" {
            return Err(HandlerError::Shape);
        }
        let arm = self
            .universe
            .arms()
            .iter()
            .find(|a| a.type_id == exception_type)
            .ok_or(HandlerError::ExceptionType)?;
        Ok(HandlerSearch {
            candidates: transfer
                .candidates
                .iter()
                .filter(|c| arm.ancestry.contains(&c.type_id))
                .cloned()
                .collect(),
            original_type: exception_type.into(),
            original_value: value,
            finally_entries: transfer.finally_entries.clone(),
            filter_catch: transfer.filter_catch.clone(),
            cursor: 0,
            awaiting_filter: false,
            finished: false,
            universe: self.universe.clone(),
        })
    }
    /// The catch identity is lexical, and the value is the original caught
    /// tagged value. A replacement local or another active catch is rejected.
    pub fn rethrow<V: Clone>(
        &self,
        node: &LoopControlNode,
        active: &BTreeMap<String, (String, V)>,
    ) -> Result<(String, V), HandlerError> {
        let context = self
            .graph
            .contexts
            .iter()
            .find(|c| c.node == node.id)
            .ok_or(HandlerError::InactiveRethrow)?;
        let caught = context
            .frames
            .iter()
            .rev()
            .find(|f| f.zone == "catch")
            .ok_or(HandlerError::InactiveRethrow)?;
        if node.kind != "rethrow" || node.slot != caught.catch_id {
            return Err(HandlerError::InactiveRethrow);
        }
        let value = active
            .get(&caught.catch_id)
            .ok_or(HandlerError::InactiveRethrow)?;
        let clause = self
            .graph
            .regions
            .iter()
            .find(|r| r.id == caught.region)
            .and_then(|r| r.catches.iter().find(|c| c.id == caught.catch_id))
            .ok_or(HandlerError::InactiveRethrow)?;
        if !self
            .universe
            .arms()
            .iter()
            .any(|a| a.type_id == value.0 && a.ancestry.contains(&clause.type_id))
        {
            return Err(HandlerError::ExceptionType);
        }
        Ok(value.clone())
    }
}

/// Validate the private routing table against its explicit lexical contexts.
/// Source ownership, dominance and complete emission remain W06's boundary.
pub fn prepare_handler_function(
    function: &LoopControlFunction,
    graph: HandlerFunction,
    universe: &ClosedExceptionUniverse,
) -> Result<PreparedHandlerFunction, HandlerError> {
    if function.callable_id != graph.callable_id || function.nodes.len() > 1024 {
        return Err(HandlerError::Shape);
    }
    let nodes = function
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let contexts = graph
        .contexts
        .iter()
        .map(|c| (c.node.as_str(), &c.frames))
        .collect::<BTreeMap<_, _>>();
    let regions = graph
        .regions
        .iter()
        .map(|r| (r.id.as_str(), r))
        .collect::<BTreeMap<_, _>>();
    if nodes.len() != function.nodes.len()
        || contexts.len() != graph.contexts.len()
        || nodes.keys().ne(contexts.keys())
        || regions.len() != graph.regions.len()
    {
        return Err(HandlerError::Shape);
    }
    for region in &graph.regions {
        if function
            .operations
            .get(region.source_ordinal)
            .is_none_or(|o| o.kind != "Try")
            || !nodes.contains_key(region.try_entry.as_str())
            || !nodes.contains_key(region.exit.as_str())
        {
            return Err(HandlerError::Shape);
        }
        let mut entered = contexts[region.exit.as_str()].clone();
        entered.push(HandlerFrame {
            region: region.id.clone(),
            zone: "try".into(),
            catch_id: String::new(),
        });
        if *contexts[region.try_entry.as_str()] != entered {
            return Err(HandlerError::Shape);
        }
        if let Some(id) = &region.finally_entry {
            if nodes
                .get(id.as_str())
                .is_none_or(|n| n.kind != "handler_finally_entry")
            {
                return Err(HandlerError::Shape);
            }
        }
        let anchored = |id: &str, zone: &str, catch_id: &str| {
            let Some(actual) = contexts.get(id) else {
                return false;
            };
            let mut expected = contexts[region.exit.as_str()].clone();
            expected.push(HandlerFrame {
                region: region.id.clone(),
                zone: zone.into(),
                catch_id: catch_id.into(),
            });
            **actual == expected
        };
        if region
            .finally_entry
            .as_ref()
            .is_some_and(|id| !anchored(id, "finally", ""))
        {
            return Err(HandlerError::Shape);
        }
        let mut ids = BTreeSet::new();
        for (ordinal, catch) in region.catches.iter().enumerate() {
            if !ids.insert(&catch.id)
                || !anchored(&catch.entry, "catch", &catch.id)
                || catch
                    .filter
                    .as_ref()
                    .is_some_and(|id| !anchored(id, "filter", &catch.id))
                || !universe.admits_catch_type(&catch.type_id)
                || nodes
                    .get(catch.entry.as_str())
                    .is_none_or(|n| n.kind != "handler_entry")
                || catch.filter.as_ref().is_some_and(|id| {
                    nodes
                        .get(id.as_str())
                        .is_none_or(|n| n.kind != "handler_filter_entry")
                })
            {
                return Err(HandlerError::Shape);
            }
            if region.catches[..ordinal].iter().any(|earlier| {
                earlier.filter.is_none()
                    && universe.catch_is_ancestor(&earlier.type_id, &catch.type_id)
            }) {
                return Err(HandlerError::SearchOrder);
            }
        }
    }
    for context in &graph.contexts {
        let mut seen = BTreeSet::new();
        for frame in &context.frames {
            let region = regions
                .get(frame.region.as_str())
                .ok_or(HandlerError::Shape)?;
            if !seen.insert(&frame.region)
                || !matches!(frame.zone.as_str(), "try" | "catch" | "filter" | "finally")
                || (matches!(frame.zone.as_str(), "catch" | "filter")
                    && !region.catches.iter().any(|c| {
                        c.id == frame.catch_id && (frame.zone != "filter" || c.filter.is_some())
                    }))
                || (matches!(frame.zone.as_str(), "try" | "finally") && !frame.catch_id.is_empty())
            {
                return Err(HandlerError::Shape);
            }
        }
    }
    let continuations = graph
        .regions
        .iter()
        .flat_map(|r| {
            r.catches
                .iter()
                .flat_map(|c| std::iter::once(c.entry.clone()).chain(c.filter.clone()))
                .chain(std::iter::once(r.exit.clone()))
                .chain(r.finally_entry.clone())
        })
        .chain(graph.transfers.iter().filter_map(|t| t.target.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    for node in &function.nodes {
        if node
            .successors
            .iter()
            .chain(&node.exceptional_successors)
            .any(|id| !nodes.contains_key(id.as_str()))
        {
            return Err(HandlerError::Shape);
        }
        let frames = contexts[node.id.as_str()];
        if matches!(
            node.kind.as_str(),
            "handler_resume" | "handler_filter_result"
        ) && node.successors != continuations
        {
            return Err(HandlerError::Shape);
        }
        if node.kind == "builtin_throw"
            && (node.slot != "System.Runtime.CompilerServices.SwitchExpressionException"
                || !node.inputs.is_empty()
                || node.exceptional_successors.len() != 1
                || !node.successors.is_empty())
        {
            return Err(HandlerError::ExceptionType);
        }
        if node.operation == "exception_payload" {
            if node.inputs.len() != 1 || !node.exceptional_successors.is_empty() {
                return Err(HandlerError::ExceptionType);
            }
            let value = function
                .nodes
                .iter()
                .find(|n| n.result == node.inputs[0])
                .ok_or(HandlerError::ExceptionType)?;
            let catch = frames
                .iter()
                .rev()
                .filter(|f| matches!(f.zone.as_str(), "catch" | "filter"))
                .filter_map(|f| {
                    regions[f.region.as_str()]
                        .catches
                        .iter()
                        .find(|c| c.id == f.catch_id && c.local == value.slot)
                })
                .next()
                .ok_or(HandlerError::ExceptionType)?;
            if value.operation != "load"
                || !node.slot.starts_with(&(catch.type_id.clone() + "."))
                || node
                    .source_ordinal
                    .and_then(|i| function.operations.get(i))
                    .is_none_or(|o| {
                        !matches!(o.kind.as_str(), "PropertyReference" | "FieldReference")
                            || o.symbol != node.slot
                    })
            {
                return Err(HandlerError::ExceptionType);
            }
        }
        if node.kind == "handler_filter_result" {
            if node.inputs.len() != 1 {
                return Err(HandlerError::FilterResult);
            }
            let producer = function
                .nodes
                .iter()
                .find(|n| n.result == node.inputs[0])
                .ok_or(HandlerError::FilterResult)?;
            if producer
                .source_ordinal
                .and_then(|i| function.operations.get(i))
                .and_then(|o| o.type_key.as_deref())
                != Some("24:mpk.csharp.value.bool.v15:value0:")
            {
                return Err(HandlerError::FilterResult);
            }
            if frames
                .last()
                .is_none_or(|f| f.zone != "filter" || f.catch_id != node.slot)
            {
                return Err(HandlerError::FilterResult);
            }
        }
        if frames.last().is_some_and(|f| f.zone == "filter")
            && node.operation == "store"
            && !node.slot.starts_with("temporary:")
        {
            return Err(HandlerError::FilterResult);
        }
        if node.kind == "rethrow"
            && frames
                .iter()
                .rev()
                .find(|f| f.zone == "catch")
                .is_none_or(|f| f.catch_id != node.slot)
        {
            return Err(HandlerError::InactiveRethrow);
        }
        if node.kind == "handler_resume" && frames.last().is_none_or(|f| f.zone != "finally") {
            return Err(HandlerError::FinallyAbrupt);
        }
    }
    let finalies = |frames: &[HandlerFrame]| -> Vec<String> {
        frames
            .iter()
            .rev()
            .filter(|f| f.zone != "finally")
            .filter_map(|f| regions[f.region.as_str()].finally_entry.clone())
            .collect()
    };
    let mut routed = BTreeSet::new();
    for transfer in &graph.transfers {
        let node = nodes
            .get(transfer.node.as_str())
            .ok_or(HandlerError::Shape)?;
        if !routed.insert(&transfer.node) {
            return Err(HandlerError::Shape);
        }
        let frames = contexts[transfer.node.as_str()];
        if node.kind == "handler_search" {
            if transfer.kind != "throw" || transfer.target.is_some() {
                return Err(HandlerError::Shape);
            }
            let mut expected = Vec::new();
            for (depth, frame) in frames.iter().enumerate().rev() {
                if frame.zone == "filter" {
                    break;
                }
                if frame.zone != "try" {
                    continue;
                }
                for catch in &regions[frame.region.as_str()].catches {
                    expected.push(HandlerCandidate {
                        catch_id: catch.id.clone(),
                        type_id: catch.type_id.clone(),
                        filter: catch.filter.clone(),
                        entry: catch.entry.clone(),
                        local: catch.local.clone(),
                        finally_entries: finalies(&frames[depth + 1..]),
                    });
                }
            }
            let filter = frames
                .iter()
                .rev()
                .find(|f| f.zone == "filter")
                .map(|f| f.catch_id.clone());
            let unwind = if filter.is_some() {
                vec![]
            } else {
                finalies(frames)
            };
            if expected != transfer.candidates || filter != transfer.filter_catch {
                return Err(HandlerError::SearchOrder);
            }
            if unwind != transfer.finally_entries {
                return Err(HandlerError::UnwindOrder);
            }
            let edges = expected
                .iter()
                .map(|c| {
                    c.filter
                        .clone()
                        .or_else(|| c.finally_entries.first().cloned())
                        .unwrap_or_else(|| c.entry.clone())
                })
                .chain(unwind)
                .collect::<BTreeSet<_>>();
            if node.successors.iter().cloned().collect::<BTreeSet<_>>() != edges {
                return Err(HandlerError::SearchOrder);
            }
        } else if node.kind == "handler_completion" {
            if (transfer.kind == "return") != transfer.target.is_none()
                || (transfer.kind == "return" && node.inputs.len() > 1)
                || (transfer.kind != "return" && !node.inputs.is_empty())
                || node.operation != transfer.kind
                || !matches!(
                    transfer.kind.as_str(),
                    "normal" | "return" | "break" | "continue"
                )
                || !transfer.candidates.is_empty()
                || transfer.filter_catch.is_some()
            {
                return Err(HandlerError::Shape);
            }
            let target = match &transfer.target {
                Some(id) => contexts
                    .get(id.as_str())
                    .ok_or(HandlerError::Shape)?
                    .as_slice(),
                None => &[],
            };
            let common = frames
                .iter()
                .zip(target)
                .take_while(|(a, b)| a == b)
                .count();
            if frames[common..].iter().any(|f| f.zone == "finally") {
                return Err(HandlerError::FinallyAbrupt);
            }
            if finalies(&frames[common..]) != transfer.finally_entries {
                return Err(HandlerError::UnwindOrder);
            }
            let edge = transfer
                .finally_entries
                .first()
                .or(transfer.target.as_ref())
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            if node.successors != edge {
                return Err(HandlerError::UnwindOrder);
            }
        } else {
            return Err(HandlerError::Shape);
        }
    }
    if function
        .nodes
        .iter()
        .filter(|n| matches!(n.kind.as_str(), "handler_search" | "handler_completion"))
        .any(|n| !routed.contains(&n.id))
    {
        return Err(HandlerError::Shape);
    }
    Ok(PreparedHandlerFunction {
        graph,
        universe: universe.clone(),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterOutcome {
    Boolean(bool),
    Threw(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandlerSearchStep<V> {
    Filter {
        catch_id: String,
        entry: String,
        local: String,
        exception_type: String,
        value: V,
    },
    Selected {
        catch_id: String,
        entry: String,
        local: String,
        finally_entries: Vec<String>,
        exception_type: String,
        value: V,
    },
    Propagate {
        finally_entries: Vec<String>,
        exception_type: String,
        value: V,
    },
    FilterFalse {
        catch_id: String,
        discarded_type: String,
    },
}
#[derive(Clone, Debug)]
pub struct HandlerSearch<V> {
    candidates: Vec<HandlerCandidate>,
    original_type: String,
    original_value: V,
    finally_entries: Vec<String>,
    filter_catch: Option<String>,
    cursor: usize,
    awaiting_filter: bool,
    finished: bool,
    universe: ClosedExceptionUniverse,
}
impl<V: Clone> HandlerSearch<V> {
    pub fn advance(
        &mut self,
        outcome: Option<FilterOutcome>,
    ) -> Result<HandlerSearchStep<V>, HandlerError> {
        if self.finished || self.awaiting_filter != outcome.is_some() {
            return Err(HandlerError::FilterResult);
        }
        if let Some(outcome) = outcome {
            let accepted = match outcome {
                FilterOutcome::Boolean(b) => b,
                FilterOutcome::Threw(t) => {
                    if !self.universe.arms().iter().any(|a| a.type_id == t) {
                        return Err(HandlerError::ExceptionType);
                    }
                    false
                }
            };
            self.awaiting_filter = false;
            if !accepted {
                self.cursor += 1;
            } else {
                return Ok(self.select());
            }
        }
        if let Some(candidate) = self.candidates.get(self.cursor) {
            if let Some(entry) = &candidate.filter {
                self.awaiting_filter = true;
                return Ok(HandlerSearchStep::Filter {
                    catch_id: candidate.catch_id.clone(),
                    entry: entry.clone(),
                    local: candidate.local.clone(),
                    exception_type: self.original_type.clone(),
                    value: self.original_value.clone(),
                });
            }
            return Ok(self.select());
        }
        self.finished = true;
        if let Some(catch_id) = &self.filter_catch {
            return Ok(HandlerSearchStep::FilterFalse {
                catch_id: catch_id.clone(),
                discarded_type: self.original_type.clone(),
            });
        }
        Ok(HandlerSearchStep::Propagate {
            finally_entries: self.finally_entries.clone(),
            exception_type: self.original_type.clone(),
            value: self.original_value.clone(),
        })
    }
    fn select(&mut self) -> HandlerSearchStep<V> {
        self.finished = true;
        let candidate = &self.candidates[self.cursor];
        HandlerSearchStep::Selected {
            catch_id: candidate.catch_id.clone(),
            entry: candidate.entry.clone(),
            local: candidate.local.clone(),
            finally_entries: candidate.finally_entries.clone(),
            exception_type: self.original_type.clone(),
            value: self.original_value.clone(),
        }
    }
}

/// Values/targets are retained, not recomputed after cleanup. A throw starts a
/// new search at the finally's throwing node; it cannot resume the old unwind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandlerCompletion<V> {
    Normal,
    Return(Option<V>),
    Break(String),
    Continue(String),
    Throw { exception_type: String, value: V },
}
pub fn complete_handler_finally<V>(
    incoming: HandlerCompletion<V>,
    produced: HandlerCompletion<V>,
) -> Result<HandlerCompletion<V>, HandlerError> {
    match produced {
        HandlerCompletion::Normal => Ok(incoming),
        HandlerCompletion::Throw { .. } => Ok(produced),
        _ => Err(HandlerError::FinallyAbrupt),
    }
}

/// A call stack participates in the same search pass. Callee cleanup is held
/// until an outer filter has finished and a destination has been selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandlerLocation {
    pub frame: usize,
    pub node: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandlerStackStep<V> {
    pub frame: usize,
    pub step: HandlerSearchStep<V>,
    pub finally_entries: Vec<HandlerLocation>,
}
pub struct HandlerStackSearch<V> {
    searches: Vec<(usize, HandlerSearch<V>)>,
    cursor: usize,
    finally_entries: Vec<HandlerLocation>,
}
impl<V: Clone> HandlerStackSearch<V> {
    /// Input order is innermost active call frame to outermost.
    pub fn new(mut searches: Vec<(usize, HandlerSearch<V>)>) -> Result<Self, HandlerError> {
        if searches.is_empty() || searches.windows(2).any(|w| w[0].0 <= w[1].0) {
            return Err(HandlerError::Shape);
        }
        let original_type = searches[0].1.original_type.clone();
        let original_value = searches[0].1.original_value.clone();
        for (_, search) in &mut searches {
            if search.original_type != original_type
                || search.cursor != 0
                || search.awaiting_filter
                || search.finished
            {
                return Err(HandlerError::ExceptionType);
            }
            search.original_value = original_value.clone();
        }
        Ok(Self {
            searches,
            cursor: 0,
            finally_entries: vec![],
        })
    }
    pub fn advance(
        &mut self,
        mut outcome: Option<FilterOutcome>,
    ) -> Result<HandlerStackStep<V>, HandlerError> {
        let count = self.searches.len();
        loop {
            let (frame, search) = self
                .searches
                .get_mut(self.cursor)
                .ok_or(HandlerError::Shape)?;
            let step = search.advance(outcome.take())?;
            match &step {
                HandlerSearchStep::Filter { .. } => {
                    return Ok(HandlerStackStep {
                        frame: *frame,
                        step,
                        finally_entries: vec![],
                    })
                }
                HandlerSearchStep::Selected {
                    finally_entries, ..
                }
                | HandlerSearchStep::Propagate {
                    finally_entries, ..
                } => {
                    self.finally_entries
                        .extend(finally_entries.iter().map(|node| HandlerLocation {
                            frame: *frame,
                            node: node.clone(),
                        }));
                }
                HandlerSearchStep::FilterFalse { .. } => {}
            }
            if matches!(step, HandlerSearchStep::Propagate { .. }) && self.cursor + 1 < count {
                self.cursor += 1;
                continue;
            }
            return Ok(HandlerStackStep {
                frame: *frame,
                step,
                finally_entries: self.finally_entries.clone(),
            });
        }
    }
}
