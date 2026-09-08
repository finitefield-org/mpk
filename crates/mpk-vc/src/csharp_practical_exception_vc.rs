//! T06-W05: closed exceptional outcomes, two-pass search and completion VCs.
//! Recipes are pending ordinary definitions; neither source metadata nor a
//! runtime exception object is a proof or an admissible exception value.
use super::control_vc as cv;
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use crate::csharp_practical_vir_validation::{
    PracticalControlProtocol, PracticalVirFunction, ValidatedPracticalVir,
};
const BOOL: &str = "mpk.csharp.value.bool.v1";
const EXC: &str = "mpk.csharp.value.exception.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExceptionVcError {
    Contract,
    Limit,
}
fn bad() -> ExceptionVcError {
    ExceptionVcError::Contract
}
fn cv_error(e: super::control_vc::ControlVcError) -> ExceptionVcError {
    match e {
        super::control_vc::ControlVcError::Limit => ExceptionVcError::Limit,
        _ => bad(),
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionCandidateVc {
    pub candidate: HandlerCandidate,
    pub filter_evaluation_guard: ControlPredicate,
    pub selected_guard: ControlPredicate,
    pub filter_failure_guard: Option<ControlPredicate>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionSearchVc {
    pub id: String,
    pub function_id: String,
    pub transfer: HandlerTransfer,
    /// All observations use the original pending value. Filter throws never
    /// overwrite it, and no finally is executed during this search pass.
    pub candidates: Vec<ExceptionCandidateVc>,
    pub exhausted_guard: ControlPredicate,
    pub rule: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionEdgeVc {
    pub edge: ControlFlowEdge,
    pub pre_node_id: String,
    pub post_node_id: Option<String>,
    pub original_exception_source_id: Option<String>,
    pub handler_exception_value: Option<TypedValueRef>,
    pub frozen_exception_value: Option<MonomorphicValue>,
    /// Actual abrupt SSA value, or the closed failure output named by the edge.
    /// The latter is constrained by that operation/check's pending relation.
    pub exception_value: Option<TypedValueRef>,
    pub exception_type_id: Option<String>,
    pub completion: Option<AbruptCompletion>,
    pub ownership_before: Vec<SequenceConstructionState>,
    pub ownership_after: Vec<SequenceConstructionState>,
    pub rule: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionFunctionVc {
    pub function_id: String,
    pub native: PracticalVirFunction,
    pub entry_conditions: Vec<ControlPredicate>,
    pub source_handlers: Option<HandlerFunction>,
    pub anchors: Option<PracticalControlProtocol>,
    pub regions: Vec<ExceptionHandlerRegion>,
    pub unwind_plans: Vec<ExceptionUnwindPlan>,
    pub edges: Vec<ExceptionEdgeVc>,
    pub possible_exception_types: Vec<String>,
    pub calls: Vec<OperationInvocation>,
    pub composition_rule: String,
}
/// Finite completion algebra consumed by ordinary expansion. The native graph
/// supplies the actual frozen value and target for each transfer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExceptionCompletionKind {
    Normal,
    Return,
    Break,
    Continue,
    Throw,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionFinallyRule {
    pub incoming: ExceptionCompletionKind,
    pub produced: ExceptionCompletionKind,
    pub result: ExceptionCompletionKind,
    pub uses_incoming_value_and_target: bool,
    pub restarts_search: bool,
}
fn finally_rules() -> Vec<ExceptionFinallyRule> {
    use ExceptionCompletionKind::*;
    [Normal, Return, Break, Continue, Throw]
        .into_iter()
        .flat_map(|incoming| {
            [
                ExceptionFinallyRule {
                    incoming,
                    produced: Normal,
                    result: incoming,
                    uses_incoming_value_and_target: true,
                    restarts_search: false,
                },
                ExceptionFinallyRule {
                    incoming,
                    produced: Throw,
                    result: Throw,
                    uses_incoming_value_and_target: false,
                    restarts_search: true,
                },
            ]
        })
        .collect()
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExceptionVcProgram {
    source_ir_sha256: String,
    universe: Vec<ClosedExceptionArm>,
    finally_rules: Vec<ExceptionFinallyRule>,
    functions: Vec<ExceptionFunctionVc>,
    searches: Vec<ExceptionSearchVc>,
    sequents: Vec<ControlSequent>,
    definition_names: Vec<String>,
    #[serde(skip)]
    nodes: usize,
    #[serde(skip)]
    depth: usize,
}
impl ExceptionVcProgram {
    pub fn finally_rules(&self) -> &[ExceptionFinallyRule] {
        &self.finally_rules
    }
    pub fn functions(&self) -> &[ExceptionFunctionVc] {
        &self.functions
    }
    pub fn universe(&self) -> &[ClosedExceptionArm] {
        &self.universe
    }
    pub fn searches(&self) -> &[ExceptionSearchVc] {
        &self.searches
    }
    pub fn sequents(&self) -> &[ControlSequent] {
        &self.sequents
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed exception VCs")
    }
    pub fn hash(&self) -> String {
        crate::hash::hash_domain_separated_raw(
            HashDomain::new("MPK-CSHARP-EXCEPTION-VC-1.0"),
            &self.canonical_bytes(),
        )
        .expect("typed hash")
        .to_hex()
    }
    pub(crate) fn nodes(&self) -> usize {
        self.nodes
    }
    pub(crate) fn binder_depth(&self) -> usize {
        self.depth
    }
    pub(crate) fn declarations(&self) -> usize {
        self.definition_names.len() + self.sequents.len()
    }
    fn push(&mut self, s: ControlSequent) -> Result<(), ExceptionVcError> {
        self.nodes += s
            .assumptions
            .iter()
            .chain(&s.goals)
            .map(|p| p.term.nodes())
            .sum::<usize>();
        if self.nodes > 262144 || self.sequents.len() >= 8192 {
            return Err(ExceptionVcError::Limit);
        }
        self.sequents.push(s);
        Ok(())
    }
}
fn atom(name: &str, bindings: Vec<ControlBinding>) -> ControlPredicate {
    let args = bindings
        .iter()
        .enumerate()
        .map(|(i, b)| ContractTerm::Var {
            index: i,
            type_id: b.type_id.clone(),
        })
        .collect();
    ControlPredicate {
        bindings,
        term: cv::apply(name, args, BOOL),
    }
}
fn binding(kind: &str, node: &str, id: &str, ty: &str) -> ControlBinding {
    ControlBinding {
        kind: kind.into(),
        node_id: node.into(),
        edge_id: None,
        value_id: id.into(),
        type_id: ty.into(),
    }
}
fn truth(v: bool) -> ControlPredicate {
    ControlPredicate {
        bindings: vec![],
        term: cv::boolean(v),
    }
}
fn neg(mut p: ControlPredicate) -> ControlPredicate {
    p.term = cv::not(p.term);
    p
}
fn or(ps: Vec<ControlPredicate>) -> ControlPredicate {
    let mut bindings = vec![];
    let mut terms = vec![];
    for p in ps {
        terms.push(cv::shift(&p.term, bindings.len(), 0));
        bindings.extend(p.bindings);
    }
    ControlPredicate {
        bindings,
        term: cv::combine(&terms, false),
    }
}
fn implies(a: ControlPredicate, b: ControlPredicate) -> ControlPredicate {
    or(vec![neg(a), b])
}
fn search(function: &str, t: &HandlerTransfer) -> ExceptionSearchVc {
    let original = binding("pending_exception", &t.node, &t.node, EXC);
    let mut prefix = truth(true);
    let mut candidates = vec![];
    for c in &t.candidates {
        let matching = atom(
            &format!("Mpk.CSharp.Exception.IsType.{}", c.type_id),
            vec![original.clone()],
        );
        let evaluate = cv::conjoin(prefix.clone(), matching.clone());
        let (accept, failed) = if let Some(filter) = &c.filter {
            let result = ControlPredicate {
                bindings: vec![binding("filter_result", filter, &c.catch_id, BOOL)],
                term: ContractTerm::Var {
                    index: 0,
                    type_id: BOOL.into(),
                },
            };
            let threw = ControlPredicate {
                bindings: vec![binding("filter_threw", filter, &c.catch_id, BOOL)],
                term: ContractTerm::Var {
                    index: 0,
                    type_id: BOOL.into(),
                },
            };
            (
                cv::conjoin(neg(threw.clone()), result),
                Some(cv::conjoin(evaluate.clone(), threw)),
            )
        } else {
            (truth(true), None)
        };
        let eligible = cv::conjoin(matching, accept);
        let selected = cv::conjoin(prefix.clone(), eligible.clone());
        prefix = cv::conjoin(prefix, neg(eligible));
        candidates.push(ExceptionCandidateVc {
            candidate: c.clone(),
            filter_evaluation_guard: evaluate,
            selected_guard: selected,
            filter_failure_guard: failed,
        });
    }
    ExceptionSearchVc{id:format!("exception.search.{function}.{}",t.node),function_id:function.into(),transfer:t.clone(),candidates,exhausted_guard:prefix,rule:"lexical_type_and_filter_search_before_unwind; filter_throw_is_false_preserves_original; selected_finally_entries_then_handler; exhausted_finally_entries_then_propagate_or_filter_false".into()}
}
fn sequent(f: &str, node: &str, kind: &str, goals: Vec<ControlPredicate>) -> ControlSequent {
    ControlSequent {
        id: format!("exception.{kind}.{f}.{node}"),
        function_id: f.into(),
        region_id: f.into(),
        kind: kind.into(),
        cutpoint_node_id: node.into(),
        source_node_id: node.into(),
        target_node_id: None,
        assumptions: vec![],
        goals,
        attachment_ids: vec![],
    }
}
fn clause(
    vir: &ValidatedPracticalVir,
    j: &J,
    owner: &str,
    old: bool,
    exception: Option<&str>,
    result_allowed: bool,
) -> Result<VerifiedContractExpression, ExceptionVcError> {
    let bytes = a::canonical_practical_json_bytes(j).map_err(|_| bad())?;
    vir.contract_expressions()
        .iter()
        .find(|e| {
            e.expression().as_bytes() == bytes
                && e.owner() == owner
                && e.allows_old() == old
                && e.exception_scope() == exception
                && (result_allowed || !e.subjects().iter().any(|(s, _)| s == "result"))
        })
        .cloned()
        .ok_or_else(bad)
}
fn arr<'a>(j: &'a J, k: &str) -> Result<&'a [J], ExceptionVcError> {
    j.get(k).and_then(J::as_array).ok_or_else(bad)
}
fn strv<'a>(j: &'a J, k: &str) -> Result<&'a str, ExceptionVcError> {
    j.get(k).and_then(J::as_str).ok_or_else(bad)
}
fn predicate(
    e: &VerifiedContractExpression,
    node: &str,
    entry: &str,
    result: Option<&TypedValueRef>,
    exception: Option<&str>,
) -> Result<ControlPredicate, ExceptionVcError> {
    let mut p = cv::predicate(e, node, entry, result).map_err(cv_error)?;
    for b in &mut p.bindings {
        if b.kind == "current_slot" && b.value_id == "exception" {
            b.kind = "exception_value".into();
            b.value_id = exception.ok_or_else(bad)?.into();
        }
    }
    Ok(p)
}
fn defined(
    data: &DataVcProgram,
    e: &VerifiedContractExpression,
    p: &ControlPredicate,
) -> Result<ControlPredicate, ExceptionVcError> {
    let d = data
        .contracts()
        .iter()
        .find(|d| d.attachment_sha256 == e.attachment_sha256())
        .ok_or_else(bad)?;
    Ok(ControlPredicate {
        bindings: p.bindings.clone(),
        term: d.definedness.clone(),
    })
}
fn terminal_goals(
    vir: &ValidatedPracticalVir,
    data: &DataVcProgram,
    f: &PracticalVirFunction,
    doc: Option<&J>,
    node: &str,
    result: Option<&TypedValueRef>,
    exception: Option<(&str, &str)>,
) -> Result<(Vec<ControlPredicate>, Vec<String>), ExceptionVcError> {
    let Some(doc) = doc else {
        let (name, values) = if let Some((_, value)) = exception {
            (
                "Mpk.CSharp.Exception.InferExceptionalSummary".to_owned(),
                vec![binding("exception_value", node, value, EXC)],
            )
        } else {
            (
                format!(
                    "Mpk.CSharp.Exception.InferNormalSummary.{}",
                    result.map_or("unit", |r| r.type_id.as_str())
                ),
                result.map(|v| vec![cv::bound(v, node)]).unwrap_or_default(),
            )
        };
        return Ok((vec![atom(&name, values)], vec![]));
    };
    let owner = strv(doc, "contract_sha256")?;
    let entry = &f.blocks[0].node.id;
    let mut goals = vec![];
    let mut ids = vec![];
    if let Some((_ty, value)) = exception {
        let mut prefix = truth(true);
        let mut covered = vec![];
        for case in arr(doc, "exceptional_cases")? {
            let exact = strv(case, "exception_type_id")?;
            let type_match = atom(
                &format!("Mpk.CSharp.Exception.ExactType.{exact}"),
                vec![binding("exception_value", node, value, EXC)],
            );
            // Control captures attach exact exception scope. Legacy data-only
            // captures have no exception variable/scope; retain that distinction.
            let scoped = vir
                .construction_context()
                .2
                .is_some_and(|s| s.control_lowering().is_some());
            let path = clause(
                vir,
                case.get("path_condition").ok_or_else(bad)?,
                owner,
                true,
                scoped.then_some(exact),
                false,
            )?;
            let condition = predicate(&path, node, entry, None, Some(value))?;
            for assigned in cv::assigned(&condition) {
                goals.push(implies(
                    cv::conjoin(prefix.clone(), type_match.clone()),
                    assigned,
                ));
            }
            goals.push(implies(
                cv::conjoin(prefix.clone(), type_match.clone()),
                defined(data, &path, &condition)?,
            ));
            let eligible = cv::conjoin(type_match, condition);
            let selected = cv::conjoin(prefix.clone(), eligible.clone());
            covered.push(selected.clone());
            ids.push(path.attachment_sha256().into());
            for j in arr(case, "ensures")? {
                let e = clause(vir, j, owner, true, scoped.then_some(exact), false)?;
                let p = predicate(&e, node, entry, None, Some(value))?;
                for assigned in cv::assigned(&p) {
                    goals.push(implies(selected.clone(), assigned));
                }
                goals.push(implies(selected.clone(), defined(data, &e, &p)?));
                goals.push(implies(selected.clone(), p));
                ids.push(e.attachment_sha256().into());
            }
            prefix = cv::conjoin(prefix, neg(eligible));
        }
        goals.push(or(covered)); // Empty throws set gives false, not an axiom.
    } else {
        for j in arr(doc, "ensures")? {
            let e = clause(vir, j, owner, true, None, true)?;
            let p = predicate(&e, node, entry, result, None)?;
            goals.extend(cv::assigned(&p));
            goals.push(defined(data, &e, &p)?);
            goals.push(p);
            ids.push(e.attachment_sha256().into());
        }
    }
    Ok((goals, ids))
}
pub(crate) fn generate_exception_vcs(
    vir: &ValidatedPracticalVir,
    data: &DataVcProgram,
    control: &ControlVcProgram,
) -> Result<ExceptionVcProgram, ExceptionVcError> {
    let (bundle, roots, source) = vir.construction_context();
    let captured = source
        .filter(|s| s.control_lowering().is_some())
        .map(|s| validate_control_source(bundle, s).map_err(|_| bad()))
        .transpose()?;
    let universe =
        derive_closed_exception_universe(roots, vir.data_closed(), vir.source_exceptions())
            .map_err(|_| bad())?;
    let mut p = ExceptionVcProgram {
        source_ir_sha256: vir.hash().into(),
        universe: universe.arms().to_vec(),
        finally_rules: finally_rules(),
        functions: vec![],
        searches: vec![],
        sequents: vec![],
        definition_names: vec![],
        nodes: 0,
        depth: 0,
    };
    let docs = vir
        .data_contracts()
        .iter()
        .map(|d| serde_json::from_str::<J>(d).map_err(|_| bad()))
        .collect::<Result<Vec<_>, _>>()?;
    let signatures = vir
        .operation_signatures()
        .iter()
        .map(|s| (s.id.as_str(), s))
        .collect::<BTreeMap<_, _>>();
    for f in vir.functions() {
        let doc = docs
            .iter()
            .find(|d| d.get("callable_id").and_then(J::as_str) == Some(&f.id));
        if doc.is_none()
            && f.exception_regions.is_empty()
            && f.unwind_plans.is_empty()
            && !f.blocks.iter().any(|b| {
                !b.node.exceptional_successors.is_empty()
                    || matches!(b.node.abrupt, Some(AbruptCompletion::Throw { .. }))
                    || b.invocation.as_ref().is_some_and(|inv| {
                        signatures
                            .get(inv.operation_id.as_str())
                            .is_some_and(|sig| {
                                matches!(
                                    sig.tag,
                                    ClosedOperationTag::SourceCall
                                        | ClosedOperationTag::ConstructorExecute
                                )
                            })
                    })
            })
        {
            continue;
        }
        let handlers = captured
            .as_ref()
            .and_then(|c| c.handlers().iter().find(|h| h.graph().callable_id == f.id));
        let flow = cv::edges(f, data).map_err(cv_error)?;
        let mut edges = vec![];
        let mut types = BTreeSet::new();
        let mut calls = vec![];
        let mut entry_conditions = vec![];
        if let Some(doc) = doc {
            for j in arr(doc, "requires")? {
                let e = clause(vir, j, strv(doc, "contract_sha256")?, false, None, false)?;
                let mut condition =
                    predicate(&e, &f.blocks[0].node.id, &f.blocks[0].node.id, None, None)?;
                for b in &mut condition.bindings {
                    if b.kind == "current_slot" {
                        b.kind = "entry_slot".into();
                    }
                }
                entry_conditions.push(condition);
            }
        }
        for e in flow {
            let Some(block) = f.blocks.iter().find(|b| b.node.id == e.source_node_id) else {
                edges.push(ExceptionEdgeVc {
                    pre_node_id: e.source_node_id.clone(),
                    post_node_id: e.target_node_id.clone(),
                    edge: e,
                    original_exception_source_id: None,
                    handler_exception_value: None,
                    frozen_exception_value: None,
                    exception_value: None,
                    exception_type_id: None,
                    completion: None,
                    ownership_before: vec![],
                    ownership_after: vec![],
                    rule: "bind_method_entry_parameters".into(),
                });
                continue;
            };
            let target = e
                .target_node_id
                .as_ref()
                .and_then(|id| f.blocks.iter().find(|b| &b.node.id == id));
            let exceptional = block
                .node
                .exceptional_successors
                .iter()
                .find(|s| Some(&s.check_id) == e.check_id.as_ref());
            let ty = exceptional
                .map(|s| s.exception_type_id.clone())
                .or_else(|| match &block.node.abrupt {
                    Some(AbruptCompletion::Throw {
                        exception_type_id, ..
                    }) => Some(exception_type_id.clone()),
                    _ => None,
                });
            if let Some(ty) = &ty {
                types.insert(ty.clone());
            }
            let filter = f
                .exception_regions
                .iter()
                .flat_map(|r| &r.catches)
                .filter_map(|c| c.filter.as_ref()?.execution.as_ref())
                .find(|x| x.node_ids.contains(&block.node.id));
            let terminal = e.target_node_id.is_none()
                && block.node.normal_successor_ids.is_empty()
                && block.node.exceptional_successors.is_empty();
            // A filter may catch its own exception. Only an escape from its
            // execution region is converted to false; local handlers still run.
            let filter_escape = filter.is_some_and(|filter| {
                matches!(e.kind.as_str(), "exception" | "throw")
                    && (terminal
                        || e.target_node_id
                            .as_ref()
                            .is_some_and(|id| !filter.node_ids.contains(id)))
            });
            let kind = if filter_escape {
                "filter_failure"
            } else if (terminal && e.kind == "throw")
                || (e.kind == "exception"
                    && target.is_some_and(|b| b.node.tag == ControlNodeTag::Exit))
            {
                "uncaught_result"
            } else if terminal && e.kind == "return" {
                "normal_result"
            } else if e.kind == "exception" || e.kind == "throw" {
                "exception_transfer"
            } else {
                "normal_transfer"
            };
            let payload = block
                .exception_values
                .iter()
                .find(|v| Some(&v.check_id) == e.check_id.as_ref())
                .map(|v| v.value.clone());
            let mut goals = vec![];
            let mut attachments = vec![];
            if kind == "normal_result" {
                if let Some(existing) = control.sequents().iter().find(|s| {
                    s.kind == "method_return"
                        && s.function_id == f.id
                        && s.source_node_id == block.node.id
                }) {
                    attachments.push(existing.id.clone());
                } else {
                    let result = block
                        .return_value_ids
                        .first()
                        .zip(f.result_type_ids.first())
                        .map(|(id, ty)| TypedValueRef {
                            id: id.clone(),
                            type_id: ty.clone(),
                        });
                    (goals, attachments) =
                        terminal_goals(vir, data, f, doc, &block.node.id, result.as_ref(), None)?;
                }
            } else if kind == "uncaught_result" {
                let frozen = format!("{}.exception", e.id);
                let value = block
                    .abrupt_value_id
                    .as_deref()
                    .or(block.handler_exception_source_id.as_deref())
                    .unwrap_or(&frozen);
                (goals, attachments) = terminal_goals(
                    vir,
                    data,
                    f,
                    doc,
                    &block.node.id,
                    None,
                    Some((ty.as_deref().ok_or_else(bad)?, value)),
                )?;
            } else {
                let values = target
                    .and_then(|t| t.handler_exception_value.as_ref())
                    .map(|v| vec![cv::bound(v, &block.node.id)])
                    .unwrap_or_default();
                goals.push(atom(&format!("Mpk.CSharp.Exception.Edge.{}", e.id), values));
            }
            // Postconditions observe the completed edge; entry slots remain immutable.
            for goal in &mut goals {
                for b in &mut goal.bindings {
                    if b.kind == "current_slot" || b.kind == "slot_assigned" {
                        b.edge_id = Some(e.id.clone());
                    }
                }
            }
            let mut s = sequent(&f.id, &e.id, kind, goals);
            s.cutpoint_node_id = f.blocks[0].node.id.clone();
            s.source_node_id = block.node.id.clone();
            s.target_node_id = e.target_node_id.clone();
            s.assumptions = entry_conditions.clone();
            s.assumptions.push(e.guard.clone());
            s.attachment_ids = attachments;
            if let Some(filter) = filter {
                s.attachment_ids.push(format!(
                    "filter_original_then:{}",
                    filter.next_search_node_id
                ));
            }
            p.push(s)?;
            edges.push(ExceptionEdgeVc {
                pre_node_id: block.node.id.clone(),
                post_node_id: e.target_node_id.clone(),
                original_exception_source_id: target.and_then(|b| b.handler_exception_source_id.clone()),
                handler_exception_value: target.and_then(|b| b.handler_exception_value.clone()),
                frozen_exception_value: payload,
                exception_value: ty.as_ref().map(|_|TypedValueRef {
                    id:block.abrupt_value_id.clone().unwrap_or_else(||format!("{}.exception",e.id)),
                    type_id:EXC.into(),
                }),
                exception_type_id: ty,
                completion: block.node.abrupt.clone(),
                ownership_before: block.ownership_in.clone(),
                ownership_after: target.map(|b| b.ownership_in.clone()).unwrap_or_else(|| block.ownership_out.clone()),
                edge: e,
                rule: if kind == "filter_failure" {
                    "discard_filter_exception_preserve_original_resume_search_without_unwind"
                } else {
                    "normal_uses_success_result; exceptional_uses_original_closed_value; exact_phi_edge_and_ownership_transfer"
                }.into(),
            });
        }
        for b in &f.blocks {
            if let Some(inv) = &b.invocation {
                let sig = signatures.get(inv.operation_id.as_str()).ok_or_else(bad)?;
                if matches!(
                    sig.tag,
                    ClosedOperationTag::SourceCall | ClosedOperationTag::ConstructorExecute
                ) {
                    calls.push(inv.clone());
                    p.push(sequent(
                        &f.id,
                        &b.node.id,
                        "callee_exception_set",
                        vec![atom(
                            &format!("Mpk.CSharp.Exception.Callee.{}", inv.operation_id),
                            inv.operands
                                .iter()
                                .map(|v| cv::bound(v, &b.node.id))
                                .collect(),
                        )],
                    ))?;
                }
            }
        }
        if let Some(handlers) = handlers {
            for t in &handlers.graph().transfers {
                if t.kind == "throw" {
                    let search = search(&f.id, t);
                    for (i, c) in search.candidates.iter().enumerate() {
                        let mut s = sequent(
                            &f.id,
                            &format!("{}.{i:04}", t.node),
                            "handler_selection",
                            vec![atom(
                                &format!(
                                    "Mpk.CSharp.Exception.Select.{}.{}",
                                    search.id, c.candidate.catch_id
                                ),
                                c.selected_guard.bindings.clone(),
                            )],
                        );
                        s.assumptions.push(c.selected_guard.clone());
                        s.target_node_id = Some(c.candidate.entry.clone());
                        s.attachment_ids = c.candidate.finally_entries.clone();
                        p.push(s)?;
                        if let Some(failure) = &c.filter_failure_guard {
                            let mut s = sequent(
                                &f.id,
                                &format!("{}.{i:04}", t.node),
                                "filter_throw_preserves_original",
                                vec![atom(
                                    "Mpk.CSharp.Exception.FilterPreservesOriginal",
                                    vec![binding("pending_exception", &t.node, &t.node, EXC)],
                                )],
                            );
                            s.assumptions.push(failure.clone());
                            p.push(s)?;
                        }
                    }
                    let mut exhausted = sequent(
                        &f.id,
                        &t.node,
                        "search_exhausted",
                        vec![atom(
                            "Mpk.CSharp.Exception.SearchExhausted",
                            vec![binding("pending_exception", &t.node, &t.node, EXC)],
                        )],
                    );
                    exhausted.assumptions.push(search.exhausted_guard.clone());
                    exhausted.attachment_ids = t.finally_entries.clone();
                    p.push(exhausted)?;
                    p.searches.push(search);
                }
                if !t.finally_entries.is_empty()
                    || t.candidates.iter().any(|c| !c.finally_entries.is_empty())
                {
                    let original = captured
                        .as_ref()
                        .and_then(|c| c.functions().iter().find(|s| s.callable_id == f.id));
                    let mut values = vec![];
                    if let (Some(original), Some(protocol)) = (original, &f.control_protocol) {
                        if let Some(node) = original.nodes.iter().find(|n| n.id == t.node) {
                            for input in &node.inputs {
                                if let Some(producer) =
                                    original.nodes.iter().find(|n| &n.result == input)
                                {
                                    if let Some(v) = protocol
                                        .anchors
                                        .iter()
                                        .find(|a| a.source_node_id == producer.id)
                                        .and_then(|a| a.result.as_ref())
                                    {
                                        values.push(cv::bound(v, &t.node));
                                    }
                                }
                            }
                        }
                    }
                    if t.kind == "throw" && values.is_empty() {
                        values.push(binding("pending_exception", &t.node, &t.node, EXC));
                    }
                    let mut s = sequent(
                        &f.id,
                        &t.node,
                        "finally_completion",
                        vec![atom(
                            &format!(
                                "Mpk.CSharp.Exception.FinallyPreservesOrReplaces.{}.{}",
                                f.id, t.node
                            ),
                            values,
                        )],
                    );
                    s.attachment_ids = t.finally_entries.clone();
                    p.push(s)?;
                }
            }
        }
        p.functions.push(ExceptionFunctionVc {
            function_id: f.id.clone(),
            native: f.clone(),
            entry_conditions,
            source_handlers: handlers.map(|h| h.graph().clone()),
            anchors: f.control_protocol.clone(),
            regions: f.exception_regions.clone(),
            unwind_plans: f.unwind_plans.clone(),
            edges,
            possible_exception_types: types.into_iter().collect(),
            calls,
            composition_rule: concat!(
                "exact_source_and_native_anchors; closed_tag_payload_only; original_exception_bound_at_handler_entry; ",
                "lexical_search_across_call_frames_before_any_cleanup; inner_to_outer_selected_unwind; selected_try_finally_on_later_completion; ",
                "normal_finally_preserves_value_and_target; throwing_finally_replaces_and_restarts_search; rethrow_uses_original_active_catch; ",
                "exception_case_first_applicable_exact_type_and_path; no_contract_requires_pending_summary_inference"
            ).into(),
        });
        if p.canonical_bytes().len() > 16 * 1024 * 1024 {
            return Err(ExceptionVcError::Limit);
        }
    }
    let mut names = BTreeSet::new();
    for pred in p
        .sequents
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .chain(p.functions.iter().flat_map(|f| {
            f.edges
                .iter()
                .map(|e| &e.edge.guard)
                .chain(&f.entry_conditions)
        }))
        .chain(p.searches.iter().flat_map(|s| {
            std::iter::once(&s.exhausted_guard).chain(s.candidates.iter().flat_map(|c| {
                std::iter::once(&c.selected_guard)
                    .chain(std::iter::once(&c.filter_evaluation_guard))
                    .chain(c.filter_failure_guard.iter())
            }))
        }))
    {
        cv::register_term(&pred.term, &mut names, pred.bindings.len(), &mut p.depth);
    }
    p.nodes += p
        .functions
        .iter()
        .flat_map(|f| {
            f.edges
                .iter()
                .map(|e| &e.edge.guard)
                .chain(&f.entry_conditions)
        })
        .chain(p.searches.iter().flat_map(|s| {
            std::iter::once(&s.exhausted_guard).chain(s.candidates.iter().flat_map(|c| {
                std::iter::once(&c.selected_guard)
                    .chain(std::iter::once(&c.filter_evaluation_guard))
                    .chain(c.filter_failure_guard.iter())
            }))
        }))
        .map(|p| p.term.nodes())
        .sum::<usize>();
    p.definition_names = names.into_iter().collect();
    p.sequents.sort_by(|a, b| a.id.cmp(&b.id));
    if p.nodes > 262144
        || p.depth > 256
        || p.declarations() > 8192
        || p.canonical_bytes().len() > 16 * 1024 * 1024
    {
        return Err(ExceptionVcError::Limit);
    }
    Ok(p)
}
