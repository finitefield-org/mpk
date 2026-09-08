//! T06-W04: source-bound loop cutpoints and ordered pattern/control sequents.
//! All semantic goals remain pending. In particular a requested total mode is
//! never evidence that a decrease or a property getter has been proved.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use crate::csharp_practical_vir_validation::{PracticalVirFunction, ValidatedPracticalVir};

const BOOL: &str = "mpk.csharp.value.bool.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlVcError {
    Contract,
    Limit,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlBinding {
    /// ssa, current_slot, entry_slot, result, or slot_assigned. Slots are logical observations
    /// at the indicated cutpoint, constrained by the retained slot transfers.
    pub kind: String,
    /// For current_slot/slot_assigned: None observes node entry; Some observes
    /// the state after this exact edge. SSA/result bindings use their exact value ID.
    pub edge_id: Option<String>,
    pub node_id: String,
    pub value_id: String,
    pub type_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlPredicate {
    pub bindings: Vec<ControlBinding>,
    pub term: ContractTerm,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlSequent {
    pub id: String,
    pub function_id: String,
    pub region_id: String,
    pub kind: String,
    pub cutpoint_node_id: String,
    pub source_node_id: String,
    pub target_node_id: Option<String>,
    pub assumptions: Vec<ControlPredicate>,
    pub goals: Vec<ControlPredicate>,
    pub attachment_ids: Vec<String>,
}
/// A single native CFG edge, including distinct true/false and exception arms.
/// Flow/slot state is joined on this edge, never by source text or variable name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlFlowEdge {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: Option<String>,
    pub kind: String,
    pub guard: ControlPredicate,
    pub check_id: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlSlotTransfer {
    pub source_node_id: String,
    pub entry_node_id: String,
    pub exit_node_id: String,
    pub kind: String,
    pub slot: String,
    pub value: TypedValueRef,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ControlFunctionVc {
    pub function_id: String,
    /// Exact validated source graph, retained once, including ordinal-less connectors.
    pub source_graph: Option<LoopControlFunction>,
    /// Requested/transitively checked mode. No proof/discharged flag exists.
    pub termination: String,
    pub slots: Vec<(String, String)>,
    pub entry_values: Vec<(String, TypedValueRef)>,
    pub entry_conditions: Vec<ControlPredicate>,
    pub transfers: Vec<ControlSlotTransfer>,
    pub edges: Vec<ControlFlowEdge>,
    /// Unmodified slots are framed; incoming slots join under each edge guard.
    /// Unassigned locals remain unassigned until a source store.
    pub slot_rule: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LoopEdgeVc {
    pub edge_id: String,
    pub role: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LoopVc {
    pub function_id: String,
    pub region: LoopRegion,
    pub origin: String,
    pub termination: String,
    pub member_node_ids: Vec<String>,
    pub modifies: Vec<String>,
    pub invariants: Vec<VerifiedContractExpression>,
    pub decreases: Vec<VerifiedContractExpression>,
    /// Generated construction loops retain actual header phis and ownership steps
    /// for rank synthesis. These are pending evidence, never a decrease proof.
    pub generated_header_phis: Vec<crate::csharp_practical_vir_validation::PracticalVirPhiValue>,
    pub generated_ownership_ids: Vec<String>,
    pub edges: Vec<LoopEdgeVc>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PatternStepVc {
    pub source_node_id: String,
    pub source_ordinal: Option<usize>,
    pub operation: String,
    pub source_inputs: Vec<String>,
    pub source_result: String,
    pub source_slot: String,
    pub source_kind: Option<String>,
    pub source_traits: Option<String>,
    pub entry_node_id: String,
    pub exit_node_id: String,
    pub artifact_node_ids: Vec<String>,
    pub result: Option<TypedValueRef>,
    pub successor_source_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PatternVc {
    pub id: String,
    pub function_id: String,
    pub governing_value: TypedValueRef,
    pub source_ordinal: usize,
    pub source_kind: String,
    pub equivalence_rule: String,
    pub steps: Vec<PatternStepVc>,
    pub no_match_node_ids: Vec<String>,
    /// removed_unreachable, modeled_exception, or statement_or_boolean.
    pub no_match_rule: String,
    pub total_getter_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ControlVcProgram {
    source_ir_sha256: String,
    functions: Vec<ControlFunctionVc>,
    loops: Vec<LoopVc>,
    patterns: Vec<PatternVc>,
    sequents: Vec<ControlSequent>,
    /// Earlier candidate-only VIR may have no captured source. These regions
    /// remain explicit unresolved requirements, never invented source contracts.
    unresolved_regions: Vec<String>,
    definition_names: Vec<String>,
    #[serde(skip)]
    node_count: usize,
    #[serde(skip)]
    binder_depth: usize,
}
impl ControlVcProgram {
    pub fn functions(&self) -> &[ControlFunctionVc] {
        &self.functions
    }
    pub fn loops(&self) -> &[LoopVc] {
        &self.loops
    }
    pub fn patterns(&self) -> &[PatternVc] {
        &self.patterns
    }
    pub fn sequents(&self) -> &[ControlSequent] {
        &self.sequents
    }
    pub fn unresolved_regions(&self) -> &[String] {
        &self.unresolved_regions
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed control VCs")
    }
    pub fn hash(&self) -> String {
        crate::hash::hash_domain_separated_raw(
            HashDomain::new("MPK-CSHARP-CONTROL-VC-1.0"),
            &self.canonical_bytes(),
        )
        .expect("typed hash")
        .to_hex()
    }
    pub(crate) fn nodes(&self) -> usize {
        self.node_count
    }
    pub(crate) fn binder_depth(&self) -> usize {
        self.binder_depth
    }
    pub(crate) fn declarations(&self) -> usize {
        self.definition_names.len() + self.sequents.len()
    }
}
fn fail() -> ControlVcError {
    ControlVcError::Contract
}
fn text<'a>(j: &'a J, key: &str) -> Result<&'a str, ControlVcError> {
    j.get(key).and_then(J::as_str).ok_or_else(fail)
}
fn array<'a>(j: &'a J, key: &str) -> Result<&'a [J], ControlVcError> {
    j.get(key).and_then(J::as_array).ok_or_else(fail)
}
pub(super) fn boolean(v: bool) -> ContractTerm {
    ContractTerm::Const {
        name: format!("Mpk.CSharp.Bool.{v}"),
        type_id: BOOL.into(),
    }
}
pub(super) fn apply(name: &str, args: Vec<ContractTerm>, result: &str) -> ContractTerm {
    let ty = |a: &[ContractTerm]| {
        a.iter()
            .rev()
            .fold(result.to_owned(), |s, a| format!("({}->{s})", a.type_id()))
    };
    let mut t = ContractTerm::Const {
        name: name.into(),
        type_id: ty(&args),
    };
    for (i, a) in args.iter().enumerate() {
        t = ContractTerm::App {
            function: Box::new(t),
            argument: Box::new(a.clone()),
            type_id: ty(&args[i + 1..]),
        };
    }
    t
}
pub(super) fn not(t: ContractTerm) -> ContractTerm {
    apply("Mpk.CSharp.Bool.Not", vec![t], BOOL)
}
pub(super) fn combine(ts: &[ContractTerm], and: bool) -> ContractTerm {
    match ts.len() {
        0 => boolean(and),
        1 => ts[0].clone(),
        n => apply(
            if and {
                "Mpk.CSharp.Bool.And"
            } else {
                "Mpk.CSharp.Bool.Or"
            },
            vec![combine(&ts[..n / 2], and), combine(&ts[n / 2..], and)],
            BOOL,
        ),
    }
}
pub(super) fn bound(v: &TypedValueRef, node: &str) -> ControlBinding {
    ControlBinding {
        edge_id: None,
        kind: "ssa".into(),
        node_id: node.into(),
        value_id: v.id.clone(),
        type_id: v.type_id.clone(),
    }
}
fn ssa_guard(id: Option<&str>, node: &str, positive: bool) -> ControlPredicate {
    match id {
        None => ControlPredicate {
            bindings: vec![],
            term: boolean(true),
        },
        Some(id) => ControlPredicate {
            bindings: vec![ControlBinding {
                edge_id: None,
                kind: "ssa".into(),
                node_id: node.into(),
                value_id: id.into(),
                type_id: BOOL.into(),
            }],
            term: {
                let t = ContractTerm::Var {
                    index: 0,
                    type_id: BOOL.into(),
                };
                if positive {
                    t
                } else {
                    not(t)
                }
            },
        },
    }
}
fn exception_guard(node: &str, check: &str) -> ControlPredicate {
    ControlPredicate {
        bindings: vec![],
        term: apply(
            &format!("Mpk.CSharp.Control.ExceptionEdge.{node}.{check}"),
            vec![],
            BOOL,
        ),
    }
}
pub(super) fn conjoin(mut a: ControlPredicate, b: ControlPredicate) -> ControlPredicate {
    if a.bindings.is_empty() && a.term == boolean(true) {
        return b;
    }
    let second = shift(&b.term, a.bindings.len(), 0);
    a.bindings.extend(b.bindings);
    a.term = combine(&[a.term, second], true);
    a
}
pub(super) fn edges(
    f: &PracticalVirFunction,
    data: &DataVcProgram,
) -> Result<Vec<ControlFlowEdge>, ControlVcError> {
    let mut out = vec![ControlFlowEdge {
        id: format!("control.edge.{}.entry", f.id),
        source_node_id: format!("{}.entry", f.id),
        target_node_id: Some(f.blocks.first().ok_or_else(fail)?.node.id.clone()),
        kind: "function_entry".into(),
        guard: ssa_guard(None, &f.id, true),
        check_id: None,
    }];
    for b in &f.blocks {
        for (i, target) in b.node.normal_successor_ids.iter().enumerate() {
            let mut guard = ssa_guard(b.condition_value_id.as_deref(), &b.node.id, i == 0);
            if let Some(op) = data
                .operations()
                .iter()
                .find(|o| o.function_id == f.id && o.node_id == b.node.id)
            {
                // Tagged outcomes travel along the normal CFG edge. Only
                // exception checks prevent taking that edge.
                let terms = op
                    .checks
                    .iter()
                    .filter(|c| c.check.tag == RequiredCheckTag::Exception)
                    .map(|c| not(c.failure_predicate.clone()))
                    .collect::<Vec<_>>();
                let data_guard = ControlPredicate {
                    bindings: op.subjects.iter().map(|s| bound(s, &b.node.id)).collect(),
                    term: combine(&terms, true),
                };
                guard = conjoin(guard, data_guard);
            } else if !b.node.exceptional_successors.is_empty() {
                let failures = b
                    .node
                    .exceptional_successors
                    .iter()
                    .map(|e| not(exception_guard(&b.node.id, &e.check_id).term))
                    .collect::<Vec<_>>();
                guard = conjoin(
                    guard,
                    ControlPredicate {
                        bindings: vec![],
                        term: combine(&failures, true),
                    },
                );
            }
            out.push(ControlFlowEdge {
                id: format!("control.edge.{}.normal.{i:04}", b.node.id),
                source_node_id: b.node.id.clone(),
                target_node_id: Some(target.clone()),
                kind: "normal".into(),
                guard,
                check_id: None,
            });
        }
        for (i, e) in b.node.exceptional_successors.iter().enumerate() {
            let guard = if let Some(op) = data
                .operations()
                .iter()
                .find(|o| o.function_id == f.id && o.node_id == b.node.id)
            {
                let c = op
                    .checks
                    .iter()
                    .find(|c| c.check.id == e.check_id)
                    .ok_or_else(fail)?;
                ControlPredicate {
                    bindings: op.subjects.iter().map(|s| bound(s, &b.node.id)).collect(),
                    term: c.failure_guard.clone(),
                }
            } else {
                exception_guard(&b.node.id, &e.check_id)
            };
            out.push(ControlFlowEdge {
                id: format!("control.edge.{}.exception.{i:04}", b.node.id),
                source_node_id: b.node.id.clone(),
                target_node_id: Some(e.target_id.clone()),
                kind: "exception".into(),
                guard,
                check_id: Some(e.check_id.clone()),
            });
        }
        if let Some(abrupt) = &b.node.abrupt {
            let (kind, target) = match abrupt {
                AbruptCompletion::Break { target_id, .. } => ("break", Some(target_id.clone())),
                AbruptCompletion::Continue { target_id, .. } => {
                    ("continue", Some(target_id.clone()))
                }
                AbruptCompletion::Return { .. } => ("return", None),
                AbruptCompletion::Throw { .. } => ("throw", None),
                _ => continue,
            };
            out.push(ControlFlowEdge {
                id: format!("control.edge.{}.abrupt", b.node.id),
                source_node_id: b.node.id.clone(),
                target_node_id: target,
                kind: kind.into(),
                guard: ssa_guard(None, &b.node.id, true),
                check_id: None,
            });
        }
    }
    Ok(out)
}
fn clause(
    vir: &ValidatedPracticalVir,
    j: &J,
    subjects: Option<&[(String, String)]>,
    owner: Option<&str>,
    allows_old: bool,
) -> Result<VerifiedContractExpression, ControlVcError> {
    let bytes = a::canonical_practical_json_bytes(j).map_err(|_| fail())?;
    vir.contract_expressions()
        .iter()
        .find(|e| {
            e.expression().as_bytes() == bytes
                && e.allows_old() == allows_old
                && subjects.is_none_or(|s| e.subjects() == s)
                && owner.is_none_or(|s| e.owner() == s)
        })
        .cloned()
        .ok_or_else(fail)
}
pub(super) fn predicate(
    e: &VerifiedContractExpression,
    node: &str,
    entry: &str,
    result: Option<&TypedValueRef>,
) -> Result<ControlPredicate, ControlVcError> {
    // W01 stores declaration order; ordinary free index 0 is the last subject.
    let bindings = e
        .subjects()
        .iter()
        .rev()
        .map(|(s, t)| {
            let (kind, value_id, point) = if let Some(slot) = s.strip_prefix("current:") {
                ("current_slot", slot.to_owned(), node)
            } else if let Some(slot) = s.strip_prefix("entry:") {
                ("entry_slot", slot.to_owned(), entry)
            } else if s == "result" {
                let v = result.ok_or_else(fail)?;
                if &v.type_id != t {
                    return Err(fail());
                }
                ("result", v.id.clone(), node)
            } else {
                return Err(fail());
            };
            Ok(ControlBinding {
                edge_id: None,
                kind: kind.into(),
                node_id: point.into(),
                value_id,
                type_id: t.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ControlPredicate {
        bindings,
        term: e.term().clone(),
    })
}
fn at_edge(mut p: ControlPredicate, edge: &ControlFlowEdge) -> ControlPredicate {
    for b in &mut p.bindings {
        if b.kind == "current_slot" {
            b.edge_id = Some(edge.id.clone());
        }
    }
    p
}
fn free_slots(t: &ContractTerm, depth: usize, out: &mut BTreeSet<usize>) {
    match t {
        ContractTerm::Var { index, .. } if *index >= depth => {
            out.insert(index - depth);
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            free_slots(function, depth, out);
            free_slots(argument, depth, out);
        }
        ContractTerm::Lam { body, .. } => free_slots(body, depth + 1, out),
        ContractTerm::Let { value, body, .. } => {
            free_slots(value, depth, out);
            free_slots(body, depth + 1, out);
        }
        _ => {}
    }
}
pub(super) fn assigned(p: &ControlPredicate) -> Vec<ControlPredicate> {
    let mut used = BTreeSet::new();
    free_slots(&p.term, 0, &mut used);
    used.into_iter()
        .filter_map(|i| {
            let mut b = p.bindings.get(i)?.clone();
            if b.kind != "current_slot" {
                return None;
            }
            b.kind = "slot_assigned".into();
            b.type_id = BOOL.into();
            Some(ControlPredicate {
                bindings: vec![b],
                term: ContractTerm::Var {
                    index: 0,
                    type_id: BOOL.into(),
                },
            })
        })
        .collect()
}
/// Shift only free binders when combining previous/current snapshots.
pub(super) fn shift(t: &ContractTerm, offset: usize, depth: usize) -> ContractTerm {
    match t {
        ContractTerm::Var { index, type_id } => ContractTerm::Var {
            index: if *index >= depth {
                index + offset
            } else {
                *index
            },
            type_id: type_id.clone(),
        },
        ContractTerm::Const { .. } => t.clone(),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => ContractTerm::App {
            function: Box::new(shift(function, offset, depth)),
            argument: Box::new(shift(argument, offset, depth)),
            type_id: type_id.clone(),
        },
        ContractTerm::Lam {
            parameter_type,
            body,
            type_id,
        } => ContractTerm::Lam {
            parameter_type: parameter_type.clone(),
            body: Box::new(shift(body, offset, depth + 1)),
            type_id: type_id.clone(),
        },
        ContractTerm::Let {
            value,
            body,
            type_id,
        } => ContractTerm::Let {
            value: Box::new(shift(value, offset, depth)),
            body: Box::new(shift(body, offset, depth + 1)),
            type_id: type_id.clone(),
        },
    }
}
fn decreasing(
    es: &[VerifiedContractExpression],
    before: &str,
    edge: &ControlFlowEdge,
    entry: &str,
) -> Result<ControlPredicate, ControlVcError> {
    let mut bindings = vec![];
    let mut pairs = vec![];
    for e in es {
        let a = predicate(e, before, entry, None)?;
        let b = at_edge(
            predicate(
                e,
                edge.target_node_id
                    .as_deref()
                    .unwrap_or(&edge.source_node_id),
                entry,
                None,
            )?,
            edge,
        );
        let av = shift(&a.term, bindings.len(), 0);
        bindings.extend(a.bindings);
        let bv = shift(&b.term, bindings.len(), 0);
        bindings.extend(b.bindings);
        pairs.push((av, bv));
    }
    Ok(ControlPredicate {
        bindings,
        term: lexicographic(&pairs),
    })
}
fn lexicographic(pairs: &[(ContractTerm, ContractTerm)]) -> ContractTerm {
    let mut equal = vec![];
    let mut choices = vec![];
    for (av, bv) in pairs {
        let mut choice = equal.clone();
        choice.push(apply(
            &format!("Mpk.CSharp.Integer.MathLess.{}", av.type_id()),
            vec![bv.clone(), av.clone()],
            BOOL,
        ));
        choices.push(combine(&choice, true));
        equal.push(apply(
            &format!("Mpk.CSharp.Integer.MathEqual.{}", av.type_id()),
            vec![bv.clone(), av.clone()],
            BOOL,
        ));
    }
    combine(&choices, false)
}

fn members(
    f: &PracticalVirFunction,
    region: &LoopRegion,
    site: Option<&Value>,
) -> BTreeSet<String> {
    let span = site.map(|site| {
        (
            site["start_byte"].as_u64().unwrap() as usize,
            site["end_byte"].as_u64().unwrap() as usize,
        )
    });
    let mut outside = BTreeSet::from([region.break_target_node_id.clone()]);
    let mut inside = BTreeSet::from([
        region.header_node_id.clone(),
        region.body_entry_node_id.clone(),
        region.continue_target_node_id.clone(),
    ]);
    if let (Some(protocol), Some((start, end))) = (&f.control_protocol, span) {
        for anchor in &protocol.anchors {
            if let Some(span) = &anchor.source_span {
                if span.start_byte >= start && span.end_byte <= end {
                    inside.extend(anchor.artifact_node_ids.iter().cloned());
                } else {
                    outside.extend(anchor.artifact_node_ids.iter().cloned());
                }
            }
        }
    }
    inside.remove(&region.break_target_node_id);
    let mut todo = inside.iter().cloned().collect::<Vec<_>>();
    while let Some(id) = todo.pop() {
        if let Some(b) = f.blocks.iter().find(|b| b.node.id == id) {
            let mut next = b.node.normal_successor_ids.clone();
            if let Some(
                AbruptCompletion::Continue { target_id, .. }
                | AbruptCompletion::Break { target_id, .. },
            ) = &b.node.abrupt
            {
                next.push(target_id.clone());
            }
            for next in next {
                if !outside.contains(&next) && inside.insert(next.clone()) {
                    todo.push(next);
                }
            }
        }
    }
    inside
}
fn classify(
    edge: &ControlFlowEdge,
    r: &LoopRegion,
    inside: &BTreeSet<String>,
) -> Option<&'static str> {
    let from = inside.contains(&edge.source_node_id);
    let to = edge
        .target_node_id
        .as_ref()
        .is_some_and(|t| inside.contains(t));
    if !from && !to {
        return None;
    }
    if !from {
        return Some("entry");
    }
    if edge.target_node_id.as_ref() == Some(&r.header_node_id)
        && r.backedge_source_ids.contains(&edge.source_node_id)
    {
        return Some("backedge");
    }
    if edge.kind == "continue" && edge.target_node_id.as_ref() == Some(&r.continue_target_node_id) {
        return Some("continue");
    }
    if edge.kind == "break" && edge.target_node_id.as_ref() == Some(&r.break_target_node_id) {
        return Some("break");
    }
    if to {
        return Some("internal");
    }
    Some(match edge.kind.as_str() {
        "return" => "return",
        "exception" | "throw" => "exception_exit",
        _ => "normal_exit",
    })
}
pub(super) fn register_term(
    t: &ContractTerm,
    names: &mut BTreeSet<String>,
    depth: usize,
    max: &mut usize,
) {
    *max = (*max).max(depth);
    match t {
        ContractTerm::Const { name, .. } => {
            names.insert(name.clone());
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            register_term(function, names, depth, max);
            register_term(argument, names, depth, max);
        }
        ContractTerm::Lam { body, .. } => register_term(body, names, depth + 1, max),
        ContractTerm::Let { value, body, .. } => {
            register_term(value, names, depth, max);
            register_term(body, names, depth + 1, max);
        }
        _ => {}
    }
}
impl ControlVcProgram {
    fn push(&mut self, s: ControlSequent) -> Result<(), ControlVcError> {
        self.node_count += s
            .assumptions
            .iter()
            .chain(&s.goals)
            .map(|p| p.term.nodes())
            .sum::<usize>();
        if self.node_count > 262_144 || self.sequents.len() >= 8_192 {
            return Err(ControlVcError::Limit);
        }
        self.sequents.push(s);
        Ok(())
    }
}
fn loop_sequents(
    p: &mut ControlVcProgram,
    l: &LoopVc,
    f: &PracticalVirFunction,
    flow: &ControlFunctionVc,
    data: &DataVcProgram,
    doc: Option<&J>,
) -> Result<(), ControlVcError> {
    let header = &l.region.header_node_id;
    let entry = &f.blocks[0].node.id;
    for edge in &l.edges {
        if edge.role == "internal" {
            continue;
        }
        let e = flow
            .edges
            .iter()
            .find(|e| e.id == edge.edge_id)
            .ok_or_else(fail)?;
        let point = e.target_node_id.as_deref().unwrap_or(&e.source_node_id);
        let mut s = ControlSequent {
            id: format!("control.loop.{}.{}.{}", l.region.id, edge.role, e.id),
            function_id: f.id.clone(),
            region_id: l.region.id.clone(),
            kind: edge.role.clone(),
            cutpoint_node_id: header.clone(),
            source_node_id: e.source_node_id.clone(),
            target_node_id: e.target_node_id.clone(),
            assumptions: vec![e.guard.clone()],
            goals: vec![],
            attachment_ids: vec![],
        };
        if edge.role != "entry" {
            s.assumptions.extend(
                l.invariants
                    .iter()
                    .map(|e| predicate(e, header, entry, None))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        if matches!(
            edge.role.as_str(),
            "entry" | "backedge" | "normal_exit" | "break"
        ) {
            for inv in &l.invariants {
                let goal = at_edge(predicate(inv, point, entry, None)?, e);
                s.goals.extend(assigned(&goal));
                s.goals.push(goal.clone());
                if let Some(defined) = data
                    .contracts()
                    .iter()
                    .find(|c| c.attachment_sha256 == inv.attachment_sha256())
                {
                    s.goals.push(ControlPredicate {
                        bindings: goal.bindings,
                        term: defined.definedness.clone(),
                    });
                }
                s.attachment_ids.push(inv.attachment_sha256().into());
            }
        }
        // A continue reaches the update/test path; it is not itself necessarily
        // a backedge. Decrease is checked after that path, at the actual header.
        if matches!(edge.role.as_str(), "entry" | "backedge") {
            for d in &l.decreases {
                let g = at_edge(predicate(d, point, entry, None)?, e);
                s.goals.extend(assigned(&g));
                s.goals.push(ControlPredicate {
                    bindings: g.bindings.clone(),
                    term: apply(
                        &format!("Mpk.CSharp.Integer.NonNegative.{}", d.term().type_id()),
                        vec![g.term],
                        BOOL,
                    ),
                });
                if let Some(defined) = data
                    .contracts()
                    .iter()
                    .find(|c| c.attachment_sha256 == d.attachment_sha256())
                {
                    s.goals.push(ControlPredicate {
                        bindings: g.bindings,
                        term: defined.definedness.clone(),
                    });
                }
                s.attachment_ids.push(d.attachment_sha256().into());
            }
            if edge.role == "backedge" && !l.decreases.is_empty() {
                // The current snapshot is the edge source after its updates;
                // the previous snapshot is the header at iteration entry.
                s.goals.push(decreasing(&l.decreases, header, e, entry)?);
            }
        }
        if edge.role == "exception_exit" {
            // W05 receives this exact edge and original exceptional clauses.
            if let Some(doc) = doc {
                s.attachment_ids.push(text(doc, "contract_sha256")?.into());
            }
        }
        if l.origin != "source_contract"
            && matches!(edge.role.as_str(), "entry" | "backedge" | "normal_exit")
        {
            s.goals.push(ControlPredicate {
                bindings: vec![],
                term: apply(
                    &format!(
                        "Mpk.CSharp.Control.GeneratedLoop.{}.{}",
                        l.region.id, edge.role
                    ),
                    vec![],
                    BOOL,
                ),
            });
        }
        p.push(s)?;
    }
    Ok(())
}
fn subtree_end(ops: &[LoopSourceOperation], root: usize) -> Result<usize, ControlVcError> {
    let mut pending = 1usize;
    let mut i = root;
    while pending != 0 {
        let op = ops.get(i).ok_or_else(fail)?;
        pending = pending - 1 + op.child_count;
        i += 1;
    }
    Ok(i)
}
fn pattern_program(
    f: &PracticalVirFunction,
    source: &LoopControlFunction,
    total_getters: &[String],
    getter_symbols: &BTreeMap<String, String>,
) -> Result<Vec<PatternVc>, ControlVcError> {
    let Some(protocol) = &f.control_protocol else {
        return Ok(vec![]);
    };
    let mut out = vec![];
    let mut ranges = source
        .nodes
        .iter()
        .filter(|n| n.kind == "pattern_decision")
        .map(|n| {
            let start = n.source_ordinal.ok_or_else(fail)?;
            Ok((start, subtree_end(&source.operations, start)?))
        })
        .collect::<Result<Vec<_>, ControlVcError>>()?;
    // Assign the innermost owner once. Scanning all ranges separately for
    // every node of every decision would multiply the three input dimensions.
    ranges.sort_unstable();
    let mut owners = vec![None; source.operations.len()];
    for &(start, end) in &ranges {
        owners[start..end].fill(Some(start));
    }
    for n in source.nodes.iter().filter(|n| n.kind == "pattern_decision") {
        let Some(anchor) = protocol.anchors.iter().find(|a| a.source_node_id == n.id) else {
            continue;
        };
        let ordinal = n.source_ordinal.ok_or_else(fail)?;
        let end = subtree_end(&source.operations, ordinal)?;
        let value = n.inputs.first().ok_or_else(fail)?;
        let producer = source
            .nodes
            .iter()
            .find(|n| &n.result == value)
            .ok_or_else(fail)?;
        let governing = protocol
            .anchors
            .iter()
            .find(|a| a.source_node_id == producer.id)
            .and_then(|a| a.result.clone())
            .ok_or_else(fail)?;
        let mut steps = vec![];
        let mut no_match = vec![];
        for node in source
            .nodes
            .iter()
            .filter(|n| n.source_ordinal.is_some_and(|i| owners[i] == Some(ordinal)))
        {
            let Some(a) = protocol
                .anchors
                .iter()
                .find(|a| a.source_node_id == node.id)
            else {
                continue;
            };
            if node.slot == "System.Runtime.CompilerServices.SwitchExpressionException" {
                no_match.push(a.entry_node_id.clone());
            }
            steps.push(PatternStepVc {
                source_node_id: node.id.clone(),
                source_ordinal: node.source_ordinal,
                operation: node.operation.clone(),
                source_inputs: node.inputs.clone(),
                source_result: node.result.clone(),
                source_slot: node.slot.clone(),
                source_kind: node
                    .source_ordinal
                    .map(|i| source.operations[i].kind.clone()),
                source_traits: node
                    .source_ordinal
                    .map(|i| source.operations[i].traits.clone()),
                entry_node_id: a.entry_node_id.clone(),
                exit_node_id: a.exit_node_id.clone(),
                artifact_node_ids: a.artifact_node_ids.clone(),
                result: a.result.clone(),
                successor_source_ids: node.successors.clone(),
            });
        }
        let kind = source.operations[ordinal].kind.clone();
        let rule = if kind != "SwitchExpression" {
            "statement_or_boolean"
        } else if no_match.is_empty() {
            "removed_unreachable"
        } else {
            "modeled_exception"
        };
        out.push(PatternVc {
            id: format!("control.pattern.{}", anchor.entry_node_id),
            function_id: f.id.clone(),
            governing_value: governing,
            source_ordinal: ordinal,
            source_kind: kind,
            equivalence_rule: concat!(
                "single_governing_evaluation; source_successor_order; ",
                "branch_true_first_false_second; first_applicable_arm; binding_only_on_success; ",
                "primitive_semantics_from_data_vcs; total_pure_property_calls; explicit_no_match"
            )
            .into(),
            steps,
            no_match_node_ids: no_match,
            no_match_rule: rule.into(),
            total_getter_ids: total_getters
                .iter()
                .filter(|g| {
                    source.operations[ordinal..end]
                        .iter()
                        .any(|o| o.symbol == **g || getter_symbols.get(*g) == Some(&o.symbol))
                })
                .cloned()
                .collect(),
        });
    }
    Ok(out)
}
pub(crate) fn generate_control_vcs(
    vir: &ValidatedPracticalVir,
    data: &DataVcProgram,
) -> Result<ControlVcProgram, ControlVcError> {
    let mut p = ControlVcProgram {
        source_ir_sha256: vir.hash().into(),
        functions: vec![],
        loops: vec![],
        patterns: vec![],
        sequents: vec![],
        unresolved_regions: vec![],
        definition_names: vec![],
        node_count: 0,
        binder_depth: 0,
    };
    let (b, _, source) = vir.construction_context();
    let control = source
        .filter(|s| s.control_lowering().is_some())
        .map(|s| validate_control_source(b, s).map_err(|_| fail()))
        .transpose()?;
    let docs = vir
        .data_contracts()
        .iter()
        .map(|s| {
            a::parse_canonical_practical_json(
                a::PracticalArtifactKind::MethodContract,
                s.as_bytes(),
            )
            .map_err(|_| fail())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let claims = docs
        .iter()
        .filter_map(|d| {
            Some((
                d.get("callable_id")?.as_str()?.to_owned(),
                d.get("termination")?.as_str()?.to_owned(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let modes = source
        .map(|s| derive_control_termination(s, &claims).map_err(|_| fail()))
        .transpose()?
        .unwrap_or_default();
    let getter_symbols = source
        .into_iter()
        .flat_map(|s| s.callables())
        .filter(|c| c.is_property_getter())
        .map(|c| {
            let i = c.identity();
            let name = i["name"].as_str().unwrap().strip_prefix("get_").unwrap();
            (
                c.id().to_owned(),
                format!("{}.{name}", i["owner"].as_str().unwrap()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for f in vir.functions() {
        let original = control
            .as_ref()
            .and_then(|c| c.functions().iter().find(|s| s.callable_id == f.id));
        let has_patterns =
            original.is_some_and(|f| f.nodes.iter().any(|n| n.kind == "pattern_decision"));
        if f.loops.is_empty() && f.patterns.is_empty() && !has_patterns {
            continue;
        }
        let doc = docs
            .iter()
            .find(|d| d.get("callable_id").and_then(J::as_str) == Some(&f.id));
        let mode = modes
            .get(&f.id)
            .cloned()
            .unwrap_or_else(|| "pending_generated".into());
        let method = control
            .as_ref()
            .and_then(|c| c.facts()["methods"].as_array())
            .and_then(|ms| ms.iter().find(|m| m["callable_id"] == f.id));
        let mut slots = BTreeMap::new();
        let mut entry_values = vec![];
        if let Some(method) = method {
            for v in method["parameters"]
                .as_array()
                .ok_or_else(fail)?
                .iter()
                .chain(method["locals"].as_array().ok_or_else(fail)?)
            {
                slots.insert(
                    v["id"].as_str().ok_or_else(fail)?.to_owned(),
                    v["type_id"].as_str().ok_or_else(fail)?.to_owned(),
                );
            }
            for (slot, value) in method["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&f.parameter_values)
            {
                entry_values.push((slot["id"].as_str().unwrap().into(), value.clone()));
            }
        }
        let mut transfers = vec![];
        if let (Some(original), Some(protocol)) = (original, &f.control_protocol) {
            for n in original
                .nodes
                .iter()
                .filter(|n| matches!(n.operation.as_str(), "load" | "store" | "pattern_bind"))
            {
                if let Some(a) = protocol.anchors.iter().find(|a| a.source_node_id == n.id) {
                    let value = a.result.clone().ok_or_else(fail)?;
                    slots
                        .entry(n.slot.clone())
                        .or_insert_with(|| value.type_id.clone());
                    transfers.push(ControlSlotTransfer {
                        source_node_id: n.id.clone(),
                        entry_node_id: a.entry_node_id.clone(),
                        exit_node_id: a.exit_node_id.clone(),
                        kind: n.operation.clone(),
                        slot: n.slot.clone(),
                        value,
                    });
                }
            }
        }
        let mut entry_conditions = doc
            .map(|d| {
                array(d, "requires")?
                    .iter()
                    .map(|j| {
                        predicate(
                            &clause(vir, j, None, Some(text(d, "contract_sha256")?), false)?,
                            &f.blocks[0].node.id,
                            &f.blocks[0].node.id,
                            None,
                        )
                    })
                    .collect::<Result<Vec<_>, ControlVcError>>()
            })
            .transpose()?
            .unwrap_or_default();
        for condition in &mut entry_conditions {
            for binding in &mut condition.bindings {
                if binding.kind == "current_slot" {
                    binding.kind = "entry_slot".into();
                }
            }
        }
        let flow = ControlFunctionVc {
            function_id: f.id.clone(),
            source_graph: original.cloned(),
            termination: mode.clone(),
            slots: slots.into_iter().collect(),
            entry_values,
            entry_conditions,
            transfers,
            edges: edges(f, data)?,
            slot_rule: concat!(
                "entry_parameters_assigned; locals_unassigned; load_reads_entry_slot; ",
                "successful_store_assigns_result; other_slots_frame; edge_guarded_incoming_join; ",
                "node_entry_is_iteration_snapshot; edge_id_observes_post_transfer_state"
            )
            .into(),
        };
        for r in &f.loops {
            let site = method
                .and_then(|m| m["loops"].as_array())
                .and_then(|ls| ls.iter().find(|l| l["loop_id"] == r.id));
            let row = doc
                .and_then(|d| d.get("loops"))
                .and_then(J::as_array)
                .and_then(|ls| {
                    ls.iter()
                        .find(|l| l.get("loop_id").and_then(J::as_str) == Some(&r.id))
                });
            let inside = members(f, r, site);
            let mut l = LoopVc {
                function_id: f.id.clone(),
                region: r.clone(),
                origin: if row.is_some() {
                    "source_contract"
                } else if source.is_some() {
                    "generated_construction"
                } else {
                    "unattached_candidate"
                }
                .into(),
                termination: mode.clone(),
                member_node_ids: inside.iter().cloned().collect(),
                modifies: vec![],
                invariants: vec![],
                decreases: vec![],
                generated_header_phis: if row.is_none() {
                    f.blocks
                        .iter()
                        .find(|b| b.node.id == r.header_node_id)
                        .ok_or_else(fail)?
                        .phi_values
                        .clone()
                } else {
                    vec![]
                },
                generated_ownership_ids: if row.is_none() {
                    data.ownership()
                        .iter()
                        .filter(|o| o.function_id == f.id && inside.contains(&o.node_id))
                        .map(|o| o.id.clone())
                        .collect()
                } else {
                    vec![]
                },
                edges: flow
                    .edges
                    .iter()
                    .filter_map(|e| {
                        classify(e, r, &inside).map(|role| LoopEdgeVc {
                            edge_id: e.id.clone(),
                            role: role.into(),
                        })
                    })
                    .collect(),
            };
            if let (Some(row), Some(site)) = (row, site) {
                let mut subjects = site["variables"]
                    .as_array()
                    .ok_or_else(fail)?
                    .iter()
                    .map(|v| {
                        Ok((
                            format!("current:{}", v["id"].as_str().ok_or_else(fail)?),
                            v["type_id"].as_str().ok_or_else(fail)?.to_owned(),
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                subjects.sort();
                l.invariants = array(row, "invariants")?
                    .iter()
                    .map(|e| clause(vir, e, Some(&subjects), Some(""), false))
                    .collect::<Result<_, _>>()?;
                l.decreases = array(row, "decreases")?
                    .iter()
                    .map(|e| clause(vir, e, Some(&subjects), Some(""), false))
                    .collect::<Result<_, _>>()?;
                l.modifies = array(row, "modifies")?
                    .iter()
                    .map(|e| e.as_str().map(str::to_owned).ok_or_else(fail))
                    .collect::<Result<_, _>>()?;
                if l.invariants.is_empty() || (mode == "total" && l.decreases.is_empty()) {
                    return Err(fail());
                }
            } else {
                p.unresolved_regions.push(r.id.clone());
            }
            loop_sequents(&mut p, &l, f, &flow, data, doc)?;
            p.loops.push(l);
        }
        // One postcondition per actual return, including returns reached after
        // normal/break exits. Nested loops retain their edge facts without
        // duplicating the method postcondition.
        if let Some(doc) = doc {
            for block in f
                .blocks
                .iter()
                .filter(|b| matches!(b.node.abrupt, Some(AbruptCompletion::Return { .. })))
            {
                let result = block
                    .return_value_ids
                    .first()
                    .zip(f.result_type_ids.first())
                    .map(|(id, t)| TypedValueRef {
                        id: id.clone(),
                        type_id: t.clone(),
                    });
                let mut goals = vec![];
                let mut ids = vec![];
                for j in array(doc, "ensures")? {
                    let expr = clause(vir, j, None, Some(text(doc, "contract_sha256")?), true)?;
                    let goal =
                        predicate(&expr, &block.node.id, &f.blocks[0].node.id, result.as_ref())?;
                    goals.extend(assigned(&goal));
                    goals.push(goal.clone());
                    if let Some(d) = data
                        .contracts()
                        .iter()
                        .find(|d| d.attachment_sha256 == expr.attachment_sha256())
                    {
                        goals.push(ControlPredicate {
                            bindings: goal.bindings,
                            term: d.definedness.clone(),
                        });
                    }
                    ids.push(expr.attachment_sha256().into());
                }
                p.push(ControlSequent {
                    id: format!("control.return.{}", block.node.id),
                    function_id: f.id.clone(),
                    region_id: f.id.clone(),
                    kind: "method_return".into(),
                    cutpoint_node_id: f.blocks[0].node.id.clone(),
                    source_node_id: block.node.id.clone(),
                    target_node_id: None,
                    assumptions: flow.entry_conditions.clone(),
                    goals,
                    attachment_ids: ids,
                })?;
            }
        }
        if let Some(original) = original {
            for pattern in pattern_program(
                f,
                original,
                control.as_ref().unwrap().total_getters(),
                &getter_symbols,
            )? {
                for step in &pattern.steps {
                    let kind = if step.operation == "pattern_bind" {
                        "pattern_binding"
                    } else if step.operation == "pattern_member" {
                        "pattern_property"
                    } else {
                        "pattern_decision_equivalence"
                    };
                    p.push(ControlSequent {
                        id: format!("{}.{}", pattern.id, step.source_node_id),
                        function_id: f.id.clone(),
                        region_id: pattern.id.clone(),
                        kind: kind.into(),
                        cutpoint_node_id: pattern.steps[0].entry_node_id.clone(),
                        source_node_id: step.entry_node_id.clone(),
                        target_node_id: Some(step.exit_node_id.clone()),
                        assumptions: vec![],
                        goals: vec![ControlPredicate {
                            bindings: std::iter::once(bound(
                                &pattern.governing_value,
                                &step.entry_node_id,
                            ))
                            .chain(step.result.as_ref().map(|v| bound(v, &step.exit_node_id)))
                            .collect(),
                            term: apply(
                                &format!(
                                    "Mpk.CSharp.Control.PatternStep.{}.{}",
                                    pattern.id, step.source_node_id
                                ),
                                std::iter::once(ContractTerm::Var {
                                    index: 0,
                                    type_id: pattern.governing_value.type_id.clone(),
                                })
                                .chain(step.result.as_ref().map(|v| ContractTerm::Var {
                                    index: 1,
                                    type_id: v.type_id.clone(),
                                }))
                                .collect(),
                                BOOL,
                            ),
                        }],
                        attachment_ids: vec![],
                    })?;
                }
                for getter in &pattern.total_getter_ids {
                    p.push(ControlSequent {
                        id: format!("{}.getter.{getter}", pattern.id),
                        function_id: f.id.clone(),
                        region_id: pattern.id.clone(),
                        kind: "property_getter_total_and_pure".into(),
                        cutpoint_node_id: pattern.steps[0].entry_node_id.clone(),
                        source_node_id: pattern.steps[0].entry_node_id.clone(),
                        target_node_id: None,
                        assumptions: vec![],
                        goals: vec![ControlPredicate {
                            bindings: vec![],
                            term: apply(
                                &format!("Mpk.CSharp.Control.TotalPureGetter.{getter}"),
                                vec![],
                                BOOL,
                            ),
                        }],
                        attachment_ids: vec![],
                    })?;
                }
                p.patterns.push(pattern);
            }
        }
        p.unresolved_regions
            .extend(f.patterns.iter().map(|d| d.node_id.clone()));
        p.node_count += flow
            .edges
            .iter()
            .map(|e| e.guard.term.nodes())
            .chain(flow.entry_conditions.iter().map(|p| p.term.nodes()))
            .sum::<usize>();
        p.functions.push(flow);
    }
    let mut names = BTreeSet::new();
    for pred in p
        .sequents
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .chain(
            p.functions
                .iter()
                .flat_map(|f| f.edges.iter().map(|e| &e.guard).chain(&f.entry_conditions)),
        )
    {
        register_term(
            &pred.term,
            &mut names,
            pred.bindings.len(),
            &mut p.binder_depth,
        );
    }
    p.definition_names = names.into_iter().collect();
    p.unresolved_regions.sort();
    p.unresolved_regions.dedup();
    p.sequents.sort_by(|a, b| a.id.cmp(&b.id));
    if p.node_count > 262_144
        || p.declarations() > 8_192
        || p.binder_depth > 256
        || p.canonical_bytes().len() > 16 * 1024 * 1024
    {
        return Err(ControlVcError::Limit);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    const INT: &str = "mpk.csharp.value.i32.v1";
    fn var(i: usize) -> ContractTerm {
        ContractTerm::Var {
            index: i,
            type_id: INT.into(),
        }
    }
    fn value(t: &ContractTerm, values: &[i64]) -> i64 {
        match t {
            ContractTerm::Var { index, .. } => values[*index],
            _ => panic!("integer term"),
        }
    }
    fn eval(t: &ContractTerm, values: &[i64]) -> bool {
        let mut args = vec![];
        let mut f = t;
        while let ContractTerm::App {
            function, argument, ..
        } = f
        {
            args.push(argument.as_ref());
            f = function;
        }
        args.reverse();
        let ContractTerm::Const { name, .. } = f else {
            panic!("predicate")
        };
        match name.as_str() {
            "Mpk.CSharp.Bool.true" => true,
            "Mpk.CSharp.Bool.false" => false,
            "Mpk.CSharp.Bool.And" => eval(args[0], values) && eval(args[1], values),
            "Mpk.CSharp.Bool.Or" => eval(args[0], values) || eval(args[1], values),
            n if n.starts_with("Mpk.CSharp.Integer.MathLess.") => {
                value(args[0], values) < value(args[1], values)
            }
            n if n.starts_with("Mpk.CSharp.Integer.MathEqual.") => {
                value(args[0], values) == value(args[1], values)
            }
            n if n.starts_with("Mpk.CSharp.Integer.NonNegative.") => value(args[0], values) >= 0,
            _ => panic!("{name}"),
        }
    }
    #[test]
    fn control_vc_lexicographic_positive_and_counterexamples() {
        let term = lexicographic(&[(var(0), var(1)), (var(2), var(3))]);
        for old0 in -1..=2 {
            for new0 in -1..=2 {
                for old1 in -1..=2 {
                    for new1 in -1..=2 {
                        let values = [old0, new0, old1, new1];
                        assert_eq!(eval(&term, &values), (new0, new1) < (old0, old1));
                    }
                }
            }
        }
        assert!(!eval(&term, &[0, 0, 0, 0]));
        let nonneg = apply(
            &format!("Mpk.CSharp.Integer.NonNegative.{INT}"),
            vec![var(0)],
            BOOL,
        );
        assert!(eval(&nonneg, &[0]));
        assert!(!eval(&nonneg, &[-1]));
        assert!(!eval(&lexicographic(&[]), &[]));
    }
    #[test]
    fn control_vc_snapshots_assignment_and_lexical_binders() {
        let p = ControlPredicate {
            bindings: vec![ControlBinding {
                kind: "current_slot".into(),
                edge_id: None,
                node_id: "header".into(),
                value_id: "local:0".into(),
                type_id: INT.into(),
            }],
            term: var(0),
        };
        let e = ControlFlowEdge {
            id: "backedge".into(),
            source_node_id: "update".into(),
            target_node_id: Some("header".into()),
            kind: "normal".into(),
            guard: ssa_guard(None, "update", true),
            check_id: None,
        };
        let after = at_edge(p.clone(), &e);
        assert_ne!(p.bindings, after.bindings);
        assert_eq!(
            assigned(&after)[0].bindings[0].edge_id,
            Some("backedge".into())
        );
        let mut unused = after.clone();
        unused.term = boolean(true);
        assert!(assigned(&unused).is_empty());
        let term = ContractTerm::Let {
            value: Box::new(var(0)),
            body: Box::new(apply("add", vec![var(0), var(1)], INT)),
            type_id: INT.into(),
        };
        let shifted = shift(&term, 2, 0);
        let ContractTerm::Let { value, body, .. } = shifted else {
            panic!()
        };
        assert_eq!(*value, var(2));
        let ContractTerm::App {
            function, argument, ..
        } = *body
        else {
            panic!()
        };
        assert_eq!(*argument, var(3));
        let ContractTerm::App { argument, .. } = *function else {
            panic!()
        };
        assert_eq!(*argument, var(0));
    }
}
