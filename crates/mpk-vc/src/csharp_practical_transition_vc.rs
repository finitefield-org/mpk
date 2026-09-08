//! T06-W08: pure source transitions and complete-snapshot replay obligations.
//! All relations remain pending ordinary recipes for W09; no runtime receipts
//! or digest comparisons discharge source equality or purity.
use super::control_vc as cv;
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use crate::csharp_practical_vir_validation::{PracticalVirFunction, ValidatedPracticalVir};
const BOOL: &str = "mpk.csharp.value.bool.v1";
const U64: &str = "mpk.csharp.value.u64.v1";
const I32: &str = "mpk.csharp.value.i32.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionVcError {
    Linkage,
    NonReflexive,
    Limit,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransitionSequent {
    pub id: String,
    pub kind: String,
    pub subjects: Vec<TypedValueRef>,
    pub assumptions: Vec<ContractTerm>,
    pub goals: Vec<ContractTerm>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransitionSnapshotNode {
    pub type_id: String,
    pub rule: String,
    pub children: Vec<String>,
    pub member_ids: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransitionPathVc {
    pub contract_sha256: String,
    pub path: String,
    pub source_error_tag: Option<String>,
    pub guard: ContractTerm,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransitionVcProgram {
    source_ir_sha256: String,
    contracts: Vec<String>,
    source_functions: Vec<PracticalVirFunction>,
    snapshots: Vec<TransitionSnapshotNode>,
    paths: Vec<TransitionPathVc>,
    sequents: Vec<TransitionSequent>,
    definition_names: Vec<String>,
    #[serde(skip)]
    nodes: usize,
    #[serde(skip)]
    depth: usize,
}
impl TransitionVcProgram {
    pub fn contracts(&self) -> &[String] {
        &self.contracts
    }
    pub fn snapshots(&self) -> &[TransitionSnapshotNode] {
        &self.snapshots
    }
    pub fn paths(&self) -> &[TransitionPathVc] {
        &self.paths
    }
    pub fn sequents(&self) -> &[TransitionSequent] {
        &self.sequents
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed transition VCs")
    }
    pub fn hash(&self) -> String {
        crate::hash::hash_domain_separated_raw(
            HashDomain::new("MPK-CSHARP-TRANSITION-VC-1.0"),
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
    fn push(
        &mut self,
        id: &str,
        kind: &str,
        subjects: Vec<TypedValueRef>,
        assumptions: Vec<ContractTerm>,
        goals: Vec<ContractTerm>,
    ) {
        self.sequents.push(TransitionSequent {
            id: format!("transition.{kind}.{id}"),
            kind: kind.into(),
            subjects,
            assumptions,
            goals,
        });
    }
    fn finish(&mut self) -> Result<(), TransitionVcError> {
        let mut names = BTreeSet::new();
        for s in &self.sequents {
            for t in s.assumptions.iter().chain(&s.goals) {
                self.nodes += t.nodes();
                cv::register_term(t, &mut names, s.subjects.len(), &mut self.depth);
            }
        }
        // Paths are serialized and consumed separately as well as occurring in sequents.
        for p in &self.paths {
            self.nodes += p.guard.nodes();
            cv::register_term(&p.guard, &mut names, 3, &mut self.depth);
        }
        self.definition_names = names.into_iter().collect();
        self.contracts.sort();
        self.sequents.sort_by(|a, b| a.id.cmp(&b.id));
        self.snapshots.sort_by(|a, b| a.type_id.cmp(&b.type_id));
        if self.sequents.windows(2).any(|s| s[0].id == s[1].id) {
            return Err(TransitionVcError::Linkage);
        }
        if self.nodes > 262144
            || self.depth > 256
            || self.definition_names.len() + self.sequents.len() > 8192
            || self.canonical_bytes().len() > 16 * 1024 * 1024
        {
            return Err(TransitionVcError::Limit);
        }
        Ok(())
    }
}
fn bad() -> TransitionVcError {
    TransitionVcError::Linkage
}
fn text<'a>(v: &'a J, k: &str) -> Result<&'a str, TransitionVcError> {
    v.get(k).and_then(J::as_str).ok_or_else(bad)
}
fn get<'a>(v: &'a J, k: &str) -> Result<&'a J, TransitionVcError> {
    v.get(k).ok_or_else(bad)
}
fn rows<'a>(v: &'a J, k: &str) -> Result<&'a [J], TransitionVcError> {
    get(v, k)?.as_array().ok_or_else(bad)
}
fn var(i: usize, t: &str) -> ContractTerm {
    ContractTerm::Var {
        index: i,
        type_id: t.into(),
    }
}
fn subject(id: &str, t: &str) -> TypedValueRef {
    TypedValueRef {
        id: id.into(),
        type_id: t.into(),
    }
}
fn call(n: &str, args: Vec<ContractTerm>, t: &str) -> ContractTerm {
    cv::apply(n, args, t)
}
fn pred(id: &str, k: &str, args: Vec<ContractTerm>) -> ContractTerm {
    call(&format!("Mpk.CSharp.Transition.{k}.{id}"), args, BOOL)
}
fn eq(t: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.Binding.Equal.{t}"), vec![a, b], BOOL)
}
fn observe(t: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(
        &format!("Mpk.CSharp.Binding.ObserveEqual.{t}"),
        vec![a, b],
        BOOL,
    )
}
fn source_eq(t: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(
        &format!("Mpk.CSharp.Transition.SourceEqual.{t}"),
        vec![a, b],
        BOOL,
    )
}
fn domain(t: &str, v: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.PublicDomain.{t}"), vec![v], BOOL)
}
fn member(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    ty: &str,
    id: &str,
    x: ContractTerm,
) -> Result<ContractTerm, TransitionVcError> {
    let m = r
        .source_types
        .get(ty)
        .ok_or_else(bad)?
        .members
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(bad)?;
    Ok(call(
        &format!("Mpk.CSharp.Transition.Member.{id}"),
        vec![x],
        &closed_type_id(b, &m.ty).map_err(|_| bad())?,
    ))
}
// Capture-avoiding substitution of verified free subjects; local binders survive.
fn subst(
    t: &ContractTerm,
    values: &[ContractTerm],
    depth: usize,
) -> Result<ContractTerm, TransitionVcError> {
    Ok(match t {
        ContractTerm::Var { index, .. } if *index >= depth => {
            cv::shift(values.get(index - depth).ok_or_else(bad)?, depth, 0)
        }
        ContractTerm::Var { .. } | ContractTerm::Const { .. } => t.clone(),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => ContractTerm::App {
            function: Box::new(subst(function, values, depth)?),
            argument: Box::new(subst(argument, values, depth)?),
            type_id: type_id.clone(),
        },
        ContractTerm::Lam {
            parameter_type,
            body,
            type_id,
        } => ContractTerm::Lam {
            parameter_type: parameter_type.clone(),
            body: Box::new(subst(body, values, depth + 1)?),
            type_id: type_id.clone(),
        },
        ContractTerm::Let {
            value,
            body,
            type_id,
        } => ContractTerm::Let {
            value: Box::new(subst(value, values, depth)?),
            body: Box::new(subst(body, values, depth + 1)?),
            type_id: type_id.clone(),
        },
    })
}
fn clause(
    vir: &ValidatedPracticalVir,
    owner: &str,
    j: &J,
    env: &BTreeMap<&str, ContractTerm>,
) -> Result<(ContractTerm, ContractTerm), TransitionVcError> {
    let bytes = a::canonical_practical_json_bytes(j).map_err(|_| bad())?;
    let e = vir
        .contract_expressions()
        .iter()
        .find(|e| {
            e.owner() == owner
                && !e.allows_old()
                && e.expression().as_bytes() == bytes
                && e.subjects().iter().all(|(n, t)| {
                    n.strip_prefix("current:")
                        .and_then(|n| env.get(n))
                        .is_some_and(|v| v.type_id() == t)
                })
        })
        .ok_or_else(bad)?;
    let values = e
        .subjects()
        .iter()
        .rev()
        .map(|(n, _)| {
            env.get(n.strip_prefix("current:").ok_or_else(bad)?)
                .cloned()
                .ok_or_else(bad)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let defined = super::data_vc::definedness(e.term(), e.definitions()).map_err(|_| bad())?;
    Ok((subst(&defined, &values, 0)?, subst(e.term(), &values, 0)?))
}
fn snapshot(
    p: &mut TransitionVcProgram,
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    t: &str,
) -> Result<(), TransitionVcError> {
    let dag = generate_structural_program(b, r, c, t).map_err(|_| bad())?;
    if !dag.is_total() {
        return Err(TransitionVcError::NonReflexive);
    }
    for node in dag.recipes().values() {
        if !p.snapshots.iter().any(|n| n.type_id == node.type_id) {
            p.snapshots.push(TransitionSnapshotNode {
                type_id: node.type_id.clone(),
                rule: node.rule.clone(),
                children: node.children.clone(),
                member_ids: r
                    .source_types
                    .get(&node.type_id)
                    .map(|t| t.members.iter().map(|m| m.id.clone()).collect())
                    .unwrap_or_default(),
            });
        }
    }
    Ok(())
}
pub(crate) fn generate_transition_vcs(
    vir: &ValidatedPracticalVir,
) -> Result<TransitionVcProgram, TransitionVcError> {
    let mut p = TransitionVcProgram {
        source_ir_sha256: vir.hash().into(),
        contracts: vec![],
        source_functions: vec![],
        snapshots: vec![],
        paths: vec![],
        sequents: vec![],
        definition_names: vec![],
        nodes: 0,
        depth: 0,
    };
    let (b, r, _) = vir.construction_context();
    let c = vir.data_closed();
    for doc in vir.data_contracts() {
        let v = a::parse_canonical_practical_json(
            a::PracticalArtifactKind::TransitionContract,
            doc.as_bytes(),
        )
        .map_err(|_| bad())?;
        if text(&v, "schema")? != a::TRANSITION_CONTRACT_SCHEMA {
            continue;
        }
        p.contracts.push(doc.clone());
        let id = text(&v, "contract_sha256")?;
        let apply = text(&v, "selected_callable_id")?;
        let st = text(&v, "state_type_id")?;
        let ct = text(&v, "command_type_id")?;
        let xt = text(&v, "context_type_id")?;
        let args = vec![var(0, st), var(1, ct), var(2, xt)];
        let subjects = vec![
            subject("state", st),
            subject("command", ct),
            subject("context", xt),
        ];
        let mut env = BTreeMap::from([
            ("state", args[0].clone()),
            ("command", args[1].clone()),
            ("context", args[2].clone()),
        ]);
        let rb = vir
            .binding_projections()
            .iter()
            .find(|p| p.binding_id == text(&v, "apply_result_binding_id").unwrap_or(""))
            .ok_or_else(bad)?;
        let tb = vir
            .binding_projections()
            .iter()
            .find(|p| p.binding_id == text(&v, "transition_binding_id").unwrap_or(""))
            .ok_or_else(bad)?;
        let types = &c
            .entries()
            .iter()
            .find(|e| e["instance_id"] == tb.semantic_type_id)
            .ok_or_else(bad)?["argument_ids"];
        let event_type = types[1].as_str().ok_or_else(bad)?;
        let response_type = types[2].as_str().ok_or_else(bad)?;
        let events_type = closed_type_id(
            b,
            &ClosedType::Instance {
                template: "bounded_sequence".into(),
                arguments: vec![ClosedType::Source(event_type.into())],
            },
        )
        .map_err(|_| bad())?;
        let result = call(apply, args.clone(), &rb.source_type_id);
        let projected = call(&rb.project.id, vec![result.clone()], &rb.semantic_type_id);
        let transition = call(
            &format!(
                "Mpk.CSharp.Transition.SuccessPayload.{}",
                rb.semantic_type_id
            ),
            vec![projected.clone()],
            &tb.semantic_type_id,
        );
        let next = call(
            &format!("Mpk.CSharp.Transition.State.{}", tb.semantic_type_id),
            vec![transition.clone()],
            st,
        );
        let events = call(
            &format!("Mpk.CSharp.Transition.Events.{}", tb.semantic_type_id),
            vec![transition.clone()],
            &events_type,
        );
        let response = call(
            &format!("Mpk.CSharp.Transition.Response.{}", tb.semantic_type_id),
            vec![transition],
            response_type,
        );
        let mut requirements = args
            .iter()
            .map(|v| domain(v.type_id(), v.clone()))
            .collect::<Vec<_>>();
        let invariant = clause(vir, id, get(&v, "state_invariant")?, &env)?;
        requirements.push(pred(id, "MethodPreconditions", args.clone()));
        let rule = get(&v, "version_rule")?;
        let time_rule = get(rule, "effective_time")?;
        let time = member(b, r, xt, text(time_rule, "member_id")?, args[2].clone())?;
        requirements.extend([
            invariant.0.clone(),
            invariant.1.clone(),
            pred(id, "ExplicitTimeDomain", vec![time.clone()]),
        ]);
        let base = vec![pred(id, "AdmittedInputs", args.clone())];
        p.push(
            id,
            "input_contract",
            subjects.clone(),
            base.clone(),
            vec![invariant.0, pred(id, "ExplicitTimeDomain", vec![time])],
        );
        p.push(
            id,
            "pure_total_source",
            subjects.clone(),
            base.clone(),
            vec![
                pred(id, "PureTotalSource", args.clone()),
                domain(&rb.source_type_id, result.clone()),
                observe(
                    st,
                    args[0].clone(),
                    call(
                        &format!("Mpk.CSharp.Transition.InputAfterApply.{id}"),
                        args.clone(),
                        st,
                    ),
                ),
            ],
        );
        let mut accepted = vec![];
        let mut defined = vec![];
        for command in rows(&v, "accepted_commands")? {
            let pair = clause(vir, id, get(command, "condition")?, &env)?;
            defined.push(pair.0);
            accepted.push(pair.1);
        }
        defined.push(cv::combine(&accepted, false));
        p.push(
            id,
            "accepted_command_coverage",
            subjects.clone(),
            base.clone(),
            defined,
        );
        let version = member(b, r, st, text(rule, "state_member_id")?, args[0].clone())?;
        let expected = member(b, r, ct, text(rule, "expected_member_id")?, args[1].clone())?;
        let conflict = cv::not(eq(U64, version.clone(), expected));
        let exhausted = eq(
            U64,
            version.clone(),
            call("Mpk.CSharp.U64.Max", vec![], U64),
        );
        let idem = get(&v, "idempotency")?;
        let enabled = text(idem, "mode")? == "complete_snapshot";
        let mut prefix = cv::boolean(true);
        let mut branches = vec![];
        let mut retained_response = None;
        let mut history = None;
        if enabled {
            snapshot(&mut p, b, r, c, ct)?;
            snapshot(&mut p, b, r, c, xt)?;
            let key = member(
                b,
                r,
                ct,
                text(idem, "command_key_member_id")?,
                args[1].clone(),
            )?;
            snapshot(&mut p, b, r, c, key.type_id())?;
            let hist = member(b, r, st, text(idem, "history_member_id")?, args[0].clone())?;
            let record_type = c
                .entries()
                .iter()
                .find(|e| e["instance_id"] == hist.type_id())
                .ok_or_else(bad)?["argument_ids"][0]
                .as_str()
                .ok_or_else(bad)?
                .to_owned();
            let record = call(
                &format!("Mpk.CSharp.Transition.RetainedRecord.{id}"),
                vec![hist.clone(), key],
                &record_type,
            );
            let old_cmd = member(
                b,
                r,
                &record_type,
                text(idem, "record_command_member_id")?,
                record.clone(),
            )?;
            let old_ctx = member(
                b,
                r,
                &record_type,
                text(idem, "record_context_member_id")?,
                record.clone(),
            )?;
            retained_response = Some(member(
                b,
                r,
                &record_type,
                text(idem, "record_response_member_id")?,
                record,
            )?);
            let same = cv::combine(
                &[
                    source_eq(ct, args[1].clone(), old_cmd),
                    source_eq(xt, args[2].clone(), old_ctx),
                ],
                true,
            );
            let found = pred(
                id,
                "RetainedKeyPresent",
                vec![args[0].clone(), args[1].clone()],
            );
            branches.push((
                "replay".to_owned(),
                None,
                cv::combine(&[found.clone(), same.clone()], true),
            ));
            branches.push((
                "error.idempotency_conflict".to_owned(),
                Some(text(&rows(&v, "errors")?[0], "source_tag")?.to_owned()),
                cv::combine(&[found.clone(), cv::not(same)], true),
            ));
            prefix = cv::not(found);
            requirements.push(pred(id, "RetainedKeysUnique", vec![hist.clone()]));
            p.push(
                id,
                "retained_key_uniqueness",
                subjects.clone(),
                base.clone(),
                vec![pred(id, "RetainedKeysUnique", vec![hist.clone()])],
            );
            let ss = vec![
                subject("command", ct),
                subject("context", xt),
                subject("retained_command", ct),
                subject("retained_context", xt),
            ];
            let xs = vec![var(0, ct), var(1, xt), var(2, ct), var(3, xt)];
            let both = cv::combine(
                &[
                    source_eq(ct, xs[0].clone(), xs[2].clone()),
                    source_eq(xt, xs[1].clone(), xs[3].clone()),
                ],
                true,
            );
            p.push(
                id,
                "snapshot_helper_equivalence",
                ss.clone(),
                xs.iter().map(|x| domain(x.type_id(), x.clone())).collect(),
                vec![
                    eq(
                        BOOL,
                        call(text(idem, "equality_callable_id")?, xs.clone(), BOOL),
                        both.clone(),
                    ),
                    eq(BOOL, both, pred(id, "CanonicalFieldEncodingsEqual", xs)),
                ],
            );
            history = Some(hist);
        }
        // Entry requirements are checked by callers. This equation defines
        // admission; it does not assert that every State has a unique history.
        p.push(
            id,
            "input_admission",
            subjects.clone(),
            vec![],
            vec![eq(
                BOOL,
                pred(id, "AdmittedInputs", args.clone()),
                cv::combine(&requirements, true),
            )],
        );
        for (ordinal, error) in rows(&v, "errors")?.iter().enumerate() {
            let code = text(error, "code")?;
            if enabled && ordinal == 0 {
                continue;
            }
            let condition = match (enabled, ordinal) {
                (false, 0) | (true, 1) => conflict.clone(),
                (false, 1) | (true, 3) => exhausted.clone(),
                (true, 2) => pred(
                    id,
                    "HistoryCapacity4096",
                    vec![history.clone().ok_or_else(bad)?],
                ),
                _ => {
                    let (d, t) = clause(vir, id, get(error, "condition")?, &env)?;
                    let mut a = base.clone();
                    a.push(prefix.clone());
                    p.push(
                        &format!("{id}.{code}"),
                        "business_condition_defined",
                        subjects.clone(),
                        a,
                        vec![d],
                    );
                    t
                }
            };
            branches.push((
                format!("error.{code}"),
                Some(text(error, "source_tag")?.into()),
                cv::combine(&[prefix.clone(), condition.clone()], true),
            ));
            prefix = cv::combine(&[prefix, cv::not(condition)], true);
        }
        branches.push(("new_success".into(), None, prefix));
        let guards = branches
            .iter()
            .map(|(_, _, g)| g.clone())
            .collect::<Vec<_>>();
        // Reject oversized partitions before cloning their pairwise formulas.
        let partition_nodes = guards
            .iter()
            .map(ContractTerm::nodes)
            .sum::<usize>()
            .saturating_mul(guards.len())
            .saturating_add(guards.len().saturating_mul(guards.len()).saturating_mul(10));
        if partition_nodes > 262144 {
            return Err(TransitionVcError::Limit);
        }
        let mut partition = vec![cv::combine(&guards, false)];
        for i in 0..guards.len() {
            for j in i + 1..guards.len() {
                partition.push(cv::not(cv::combine(
                    &[guards[i].clone(), guards[j].clone()],
                    true,
                )));
            }
        }
        p.push(
            id,
            "ordered_path_partition",
            subjects.clone(),
            base.clone(),
            partition,
        );
        for (path, tag, guard) in branches {
            let owner = format!("{id}.{path}");
            p.paths.push(TransitionPathVc {
                contract_sha256: id.into(),
                path: path.clone(),
                source_error_tag: tag.clone(),
                guard: guard.clone(),
            });
            let mut assumptions = base.clone();
            assumptions.push(guard);
            if let Some(tag) = tag {
                p.push(
                    &owner,
                    "error_result",
                    subjects.clone(),
                    assumptions,
                    vec![
                        pred(
                            &format!("{id}.{tag}"),
                            "ExactErrorArm",
                            vec![projected.clone()],
                        ),
                        observe(
                            st,
                            args[0].clone(),
                            call(
                                &format!("Mpk.CSharp.Transition.InputAfterApply.{id}"),
                                args.clone(),
                                st,
                            ),
                        ),
                    ],
                );
                continue;
            }
            let mut goals = vec![
                pred(id, "SuccessArm", vec![projected.clone()]),
                domain(st, next.clone()),
                domain(&events_type, events.clone()),
                domain(response_type, response.clone()),
                pred(
                    id,
                    "EventAndAggregateBounds4096_65536",
                    vec![next.clone(), events.clone(), response.clone()],
                ),
            ];
            if path == "replay" {
                goals.extend([
                    observe(st, next.clone(), args[0].clone()),
                    eq(
                        I32,
                        call(
                            &format!("Mpk.CSharp.Transition.Length.{events_type}"),
                            vec![events.clone()],
                            I32,
                        ),
                        call("Mpk.CSharp.I32.Zero", vec![], I32),
                    ),
                    observe(
                        response_type,
                        response.clone(),
                        retained_response.clone().ok_or_else(bad)?,
                    ),
                ]);
            } else {
                env.insert("next_state", next.clone());
                env.insert("events", events.clone());
                env.insert("response", response.clone());
                let next_env = BTreeMap::from([("state", next.clone())]);
                let pair = clause(vir, id, get(&v, "state_invariant")?, &next_env)?;
                goals.extend([pair.0, pair.1]);
                let next_version = member(b, r, st, text(rule, "state_member_id")?, next.clone())?;
                goals.push(eq(
                    U64,
                    next_version,
                    call("Mpk.CSharp.U64.CheckedAddOne", vec![version.clone()], U64),
                ));
                for name in ["event_relation", "response_relation"] {
                    let pair = clause(vir, id, get(&v, name)?, &env)?;
                    goals.extend([pair.0, pair.1]);
                }
                if enabled {
                    goals.extend([
                        pred(
                            id,
                            "AppendCompleteSnapshot",
                            vec![
                                args[0].clone(),
                                next.clone(),
                                args[1].clone(),
                                args[2].clone(),
                                response.clone(),
                            ],
                        ),
                        pred(
                            id,
                            "PreserveRetainedHistoryOrder",
                            vec![args[0].clone(), next.clone()],
                        ),
                    ]);
                }
            }
            p.push(
                &owner,
                "successful_result",
                subjects.clone(),
                assumptions,
                goals,
            );
        }
    }
    if !p.contracts.is_empty() {
        p.source_functions = vir.functions().to_vec();
    }
    p.finish()?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transition_substitution_preserves_local_binders() {
        let body = eq(I32, var(1, I32), var(0, I32));
        let term = ContractTerm::Lam {
            parameter_type: I32.into(),
            body: Box::new(body),
            type_id: format!("({I32}->{BOOL})"),
        };
        let replaced = subst(&term, &[var(2, I32)], 0).unwrap();
        let ContractTerm::Lam { body, .. } = replaced else {
            panic!()
        };
        assert_eq!(*body, eq(I32, var(3, I32), var(0, I32)));
        let term = ContractTerm::Let {
            value: Box::new(var(0, I32)),
            body: Box::new(eq(I32, var(1, I32), var(0, I32))),
            type_id: BOOL.into(),
        };
        let replaced = subst(&term, &[var(2, I32)], 0).unwrap();
        let ContractTerm::Let { value, body, .. } = replaced else {
            panic!()
        };
        assert_eq!(*value, var(2, I32));
        assert_eq!(*body, eq(I32, var(3, I32), var(0, I32)));
    }
}
