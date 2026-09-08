//! T06-W06: source representation and concrete foundation equivalence VCs.
//! Structural import is evidence of linkage only. Every semantic relation below
//! remains a goal for ordinary expansion and proof assembly (T06-W09).
use super::control_vc as cv;
use super::*;
use crate::csharp_practical_source_artifacts::{self as a};
use crate::csharp_practical_vir_validation::{PracticalVirFunction, ValidatedPracticalVir};
const BOOL: &str = "mpk.csharp.value.bool.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingVcError {
    Binding,
    Limit,
}
fn bad() -> BindingVcError {
    BindingVcError::Binding
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BindingSequent {
    pub id: String,
    pub owner_id: String,
    pub kind: String,
    /// Universally quantified concrete source/semantic values, free-index order.
    pub subjects: Vec<TypedValueRef>,
    pub assumptions: Vec<ContractTerm>,
    pub goals: Vec<ContractTerm>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BindingRepresentationVc {
    pub projection: BindingTypeProjection,
    pub binding: Value,
    pub source: Value,
    pub invariant: Option<ConstructionTypeInvariant>,
    pub commutations: Vec<BindingOperationCommutation>,
    /// Every stored field remains observable, including inactive arm payloads.
    pub reconstruction_member_ids: Vec<String>,
    pub rule: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FoundationInstanceVc {
    pub instance_id: String,
    pub argument_ids: Vec<String>,
    pub dependency_ids: Vec<String>,
    pub provenance_ids: Vec<String>,
    pub type_definition: Value,
    pub operation_definitions: Vec<Value>,
    pub counters: Value,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BindingVcProgram {
    source_ir_sha256: String,
    foundation_descriptor: Value,
    closed_roots_sha256: String,
    closed_set_sha256: String,
    semantic_bindings_sha256: String,
    counters: Value,
    instances: Vec<FoundationInstanceVc>,
    representations: Vec<BindingRepresentationVc>,
    identity_projections: Vec<BindingTypeProjection>,
    source_functions: Vec<PracticalVirFunction>,
    sequents: Vec<BindingSequent>,
    definition_names: Vec<String>,
    #[serde(skip)]
    nodes: usize,
    #[serde(skip)]
    depth: usize,
}
impl BindingVcProgram {
    pub fn instances(&self) -> &[FoundationInstanceVc] {
        &self.instances
    }
    pub fn representations(&self) -> &[BindingRepresentationVc] {
        &self.representations
    }
    pub fn sequents(&self) -> &[BindingSequent] {
        &self.sequents
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed binding VCs")
    }
    pub fn hash(&self) -> String {
        crate::hash::hash_domain_separated_raw(
            HashDomain::new("MPK-CSHARP-BINDING-VC-1.0"),
            &self.canonical_bytes(),
        )
        .expect("typed binding hash")
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
    fn push(
        &mut self,
        owner: &str,
        kind: &str,
        subjects: Vec<TypedValueRef>,
        assumptions: Vec<ContractTerm>,
        goals: Vec<ContractTerm>,
    ) -> Result<(), BindingVcError> {
        self.nodes += assumptions
            .iter()
            .chain(&goals)
            .map(ContractTerm::nodes)
            .sum::<usize>();
        if self.nodes > 262144 || self.sequents.len() >= 8192 {
            return Err(BindingVcError::Limit);
        }
        self.sequents.push(BindingSequent {
            id: format!("binding.{kind}.{owner}"),
            owner_id: owner.into(),
            kind: kind.into(),
            subjects,
            assumptions,
            goals,
        });
        Ok(())
    }
}
fn text<'a>(j: &'a Value, k: &str) -> Result<&'a str, BindingVcError> {
    j.get(k).and_then(Value::as_str).ok_or_else(bad)
}
fn strings(j: &Value, k: &str) -> Result<Vec<String>, BindingVcError> {
    j.get(k)
        .and_then(Value::as_array)
        .ok_or_else(bad)?
        .iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or_else(bad))
        .collect()
}
fn var(index: usize, ty: &str) -> ContractTerm {
    ContractTerm::Var {
        index,
        type_id: ty.into(),
    }
}
fn subject(id: &str, ty: &str) -> TypedValueRef {
    TypedValueRef {
        id: id.into(),
        type_id: ty.into(),
    }
}
fn call(id: &str, args: Vec<ContractTerm>, ty: &str) -> ContractTerm {
    cv::apply(id, args, ty)
}
fn domain(ty: &str, x: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.PublicDomain.{ty}"), vec![x], BOOL)
}
fn equal(ty: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.Binding.Equal.{ty}"), vec![a, b], BOOL)
}
fn observe(ty: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(
        &format!("Mpk.CSharp.Binding.ObserveEqual.{ty}"),
        vec![a, b],
        BOOL,
    )
}
fn project(p: &BindingTypeProjection, x: ContractTerm) -> ContractTerm {
    call(&p.project.id, vec![x], &p.semantic_type_id)
}
fn reconstruct(p: &BindingTypeProjection, x: ContractTerm) -> ContractTerm {
    call(&p.reconstruct.id, vec![x], &p.source_type_id)
}
fn implication(a: ContractTerm, b: ContractTerm) -> ContractTerm {
    cv::combine(&[cv::not(a), b], false)
}
fn id_hash(v: &Value) -> Result<String, BindingVcError> {
    hash_value(HashDomain::new("MPK-CSHARP-CONCRETE-DEFINITION-1.0"), v).map_err(|_| bad())
}
fn representation(
    p: &mut BindingVcProgram,
    projection: &BindingTypeProjection,
    binding: &Value,
    source: &Value,
    invariant: Option<&ConstructionTypeInvariant>,
) -> Result<(), BindingVcError> {
    let id = &projection.id;
    let st = &projection.source_type_id;
    let mt = &projection.semantic_type_id;
    let x = var(0, st);
    let y = var(0, mt);
    let dx = domain(st, x.clone());
    let dy = domain(mt, y.clone());
    let sx = vec![subject("source", st)];
    let sy = vec![subject("semantic", mt)];
    let px = project(projection, x.clone());
    let ry = reconstruct(projection, y.clone());
    p.push(
        id,
        "projection_total",
        sx.clone(),
        vec![dx.clone()],
        vec![domain(mt, px.clone())],
    )?;
    p.push(
        id,
        "reconstruction",
        sy.clone(),
        vec![dy.clone()],
        vec![domain(st, ry.clone())],
    )?;
    p.push(
        id,
        "source_round_trip",
        sx.clone(),
        vec![dx.clone()],
        vec![observe(st, reconstruct(projection, px.clone()), x.clone())],
    )?;
    p.push(
        id,
        "semantic_round_trip",
        sy.clone(),
        vec![dy.clone()],
        vec![equal(mt, project(projection, ry), y)],
    )?;
    let members = source["members"].as_array().ok_or_else(bad)?;
    for member in members {
        let member_id = text(member, "id")?;
        // No inaccessible/inactive field is silently forgotten by reconstruction.
        p.push(
            &format!("{id}.{member_id}"),
            "member_reconstruction",
            sx.clone(),
            vec![dx.clone()],
            vec![call(
                &format!("Mpk.CSharp.Binding.MemberEqual.{member_id}"),
                vec![reconstruct(projection, px.clone()), x.clone()],
                BOOL,
            )],
        )?;
    }
    let mapping = binding["member_map"].as_object().ok_or_else(bad)?;
    for (role, member) in mapping {
        p.push(
            &format!("{id}.{role}"),
            "member_agreement",
            sx.clone(),
            vec![dx.clone()],
            vec![call(
                &format!(
                    "Mpk.CSharp.Binding.MemberProjection.{id}.{}.{role}",
                    member.as_str().ok_or_else(bad)?
                ),
                vec![x.clone(), px.clone()],
                BOOL,
            )],
        )?;
    }
    let arms = binding["tag_arms"]
        .as_object()
        .ok_or_else(bad)?
        .iter()
        .collect::<Vec<_>>();
    let guards = arms
        .iter()
        .map(|arm| {
            Ok(call(
                &format!(
                    "Mpk.CSharp.Binding.SourceTag.{id}.{}",
                    arm.1.as_str().ok_or_else(bad)?
                ),
                vec![x.clone()],
                BOOL,
            ))
        })
        .collect::<Result<Vec<_>, BindingVcError>>()?;
    if !guards.is_empty() {
        let mut goals = vec![cv::combine(&guards, false)];
        for i in 0..guards.len() {
            for j in i + 1..guards.len() {
                goals.push(cv::not(cv::combine(
                    &[guards[i].clone(), guards[j].clone()],
                    true,
                )));
            }
        }
        p.push(id, "exactly_one_arm", sx.clone(), vec![dx.clone()], goals)?;
        for (i, arm) in arms.iter().enumerate() {
            let semantic = arm.0;
            p.push(
                &format!("{id}.{semantic}"),
                "tag_payload_agreement",
                sx.clone(),
                vec![dx.clone(), guards[i].clone()],
                vec![
                    call(
                        &format!("Mpk.CSharp.Binding.SemanticArm.{mt}.{semantic}"),
                        vec![px.clone()],
                        BOOL,
                    ),
                    call(
                        &format!("Mpk.CSharp.Binding.Payload.{id}.{semantic}"),
                        vec![x.clone(), px.clone()],
                        BOOL,
                    ),
                ],
            )?;
        }
    }
    // Source invariants and observations are ordinary equations, never granted
    // by a sidecar's role/default/identity classification.
    p.push(
        id,
        "source_invariant",
        sx.clone(),
        vec![],
        vec![equal(
            BOOL,
            domain(st, x.clone()),
            invariant.map(|t| t.public_body.clone()).unwrap_or_else(|| {
                call(
                    &format!("Mpk.CSharp.Binding.SourceInvariant.{id}"),
                    vec![x.clone()],
                    BOOL,
                )
            }),
        )],
    )?;
    p.push(
        id,
        "identity_unobservable",
        sx.clone(),
        vec![dx.clone()],
        vec![call(
            &format!("Mpk.CSharp.Binding.ValueObservation.{id}"),
            vec![x.clone()],
            BOOL,
        )],
    )?;
    for (bid, bound) in binding["bounds"].as_object().ok_or_else(bad)? {
        let max = bound.as_u64().ok_or_else(bad)?;
        p.push(
            &format!("{id}.{bid}"),
            "bound",
            sx.clone(),
            vec![dx.clone()],
            vec![call(
                &format!("Mpk.CSharp.Binding.Bound.{id}.{bid}.{max}"),
                vec![px.clone()],
                BOOL,
            )],
        )?;
    }
    let default = text(binding, "default_arm")?;
    let actual = call(
        &format!("Mpk.CSharp.Binding.ActualDefault.{st}"),
        vec![],
        st,
    );
    let goals = if default == "ineligible" {
        vec![call(
            &format!("Mpk.CSharp.Binding.DefaultUseForbidden.{id}"),
            vec![],
            BOOL,
        )]
    } else {
        vec![
            domain(st, actual.clone()),
            call(
                &format!("Mpk.CSharp.Binding.SemanticArm.{mt}.{default}"),
                vec![project(projection, actual)],
                BOOL,
            ),
        ]
    };
    p.push(id, "actual_default", vec![], vec![], goals)?;
    if matches!(text(binding, "role")?, "ordered_map" | "ordered_set") {
        p.push(
            id,
            "canonical_order_and_uniqueness",
            sx.clone(),
            vec![dx.clone()],
            vec![call(
                &format!("Mpk.CSharp.Binding.CanonicalOrder.{mt}"),
                vec![px.clone()],
                BOOL,
            )],
        )?;
    }
    if text(binding, "role")? == "validation" {
        p.push(
            id,
            "nonempty_invalid",
            sx,
            vec![dx],
            vec![call(
                &format!("Mpk.CSharp.Binding.NonemptyInvalid.{mt}"),
                vec![px],
                BOOL,
            )],
        )?;
    }
    Ok(())
}
fn commutation(
    p: &mut BindingVcProgram,
    c: &BindingOperationCommutation,
    projections: &[BindingTypeProjection],
) -> Result<(), BindingVcError> {
    let op = &c.source_operation;
    let sem = &c.semantic_operation;
    let owner = format!("{}.{}.{}", c.binding_id, op.id, sem.id);
    let args = op
        .argument_type_ids
        .iter()
        .enumerate()
        .map(|(i, t)| var(i, t))
        .collect::<Vec<_>>();
    let subjects = op
        .argument_type_ids
        .iter()
        .enumerate()
        .map(|(i, t)| subject(&format!("operand.{i}"), t))
        .collect::<Vec<_>>();
    let mut assumptions = args
        .iter()
        .zip(&op.argument_type_ids)
        .map(|(x, t)| domain(t, x.clone()))
        .collect::<Vec<_>>();
    let mapped = args
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let id = &c.operand_projection_ids[i];
            if let Some(pr) = projections.iter().find(|p| &p.id == id) {
                Ok(project(pr, x.clone()))
            } else if let Some(r) = c.rounding_operands.iter().find(|r| r.ordinal as usize == i) {
                Ok(call(
                    &format!("Mpk.CSharp.Binding.EnumOperand.{}.{}", owner, r.ordinal),
                    vec![x.clone()],
                    &sem.argument_type_ids[i],
                ))
            } else if op.argument_type_ids[i] == sem.argument_type_ids[i] {
                Ok(x.clone())
            } else {
                Err(bad())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let source_result = call(&op.id, args.clone(), &op.normal_result_type_id);
    let semantic_result = call(&sem.id, mapped.clone(), &sem.normal_result_type_id);
    let projected = if c.returned_result.is_some() {
        let pr = projections
            .iter()
            .find(|p| p.id == c.result_projection_id)
            .ok_or_else(bad)?;
        call(
            &format!("Mpk.CSharp.Binding.SuccessPayload.{}", pr.semantic_type_id),
            vec![project(pr, source_result)],
            &sem.normal_result_type_id,
        )
    } else if let Some(pr) = projections.iter().find(|p| p.id == c.result_projection_id) {
        project(pr, source_result)
    } else if op.normal_result_type_id == sem.normal_result_type_id {
        source_result
    } else {
        return Err(bad());
    };
    let normal = call(
        &format!("Mpk.CSharp.Binding.Success.{}", op.id),
        args.clone(),
        BOOL,
    );
    assumptions.push(normal.clone());
    p.push(
        &owner,
        "operation_normal_commutation",
        subjects.clone(),
        assumptions,
        vec![
            call(
                &format!("Mpk.CSharp.Binding.Success.{}", sem.id),
                mapped.clone(),
                BOOL,
            ),
            observe(&sem.normal_result_type_id, projected, semantic_result),
        ],
    )?;
    for (i, outcome) in c.ordered_outcomes.iter().enumerate() {
        let source_failure = call(
            &format!(
                "Mpk.CSharp.Binding.Failure.{}.{}",
                op.id, outcome.source_check_id
            ),
            args.clone(),
            BOOL,
        );
        let semantic_failure = call(
            &format!(
                "Mpk.CSharp.Binding.Failure.{}.{}",
                sem.id, outcome.semantic_check_id
            ),
            mapped.clone(),
            BOOL,
        );
        let mut conditions = args
            .iter()
            .zip(&op.argument_type_ids)
            .map(|(x, t)| domain(t, x.clone()))
            .collect::<Vec<_>>();
        for earlier in op
            .ordered_checks
            .iter()
            .take_while(|c| c.id != outcome.source_check_id)
        {
            conditions.push(cv::not(call(
                &format!("Mpk.CSharp.Binding.Failure.{}.{}", op.id, earlier.id),
                args.clone(),
                BOOL,
            )));
        }
        let mut outcome_goals = sem
            .ordered_checks
            .iter()
            .take_while(|c| c.id != outcome.semantic_check_id)
            .map(|c| {
                cv::not(call(
                    &format!("Mpk.CSharp.Binding.Failure.{}.{}", sem.id, c.id),
                    mapped.clone(),
                    BOOL,
                ))
            })
            .collect::<Vec<_>>();
        outcome_goals.push(semantic_failure);
        outcome_goals.push(call(
            &format!("Mpk.CSharp.Binding.OutcomeProjection.{}.{}", owner, i),
            args.clone(),
            BOOL,
        ));
        conditions.push(source_failure);
        p.push(
            &format!("{}.{}", owner, i),
            "operation_outcome_commutation",
            subjects.clone(),
            conditions,
            outcome_goals,
        )?;
    }
    if let Some(returned) = &c.returned_result {
        for route in &returned.ordered_errors {
            let mut assumptions = args
                .iter()
                .zip(&op.argument_type_ids)
                .map(|(x, t)| domain(t, x.clone()))
                .collect::<Vec<_>>();
            assumptions.push(call(
                &format!(
                    "Mpk.CSharp.Binding.ReturnedError.{}.{}",
                    op.id, route.source_carrier
                ),
                args.clone(),
                BOOL,
            ));
            let mut goals = sem
                .ordered_checks
                .iter()
                .take_while(|c| c.id != route.semantic_check_id)
                .map(|c| {
                    cv::not(call(
                        &format!("Mpk.CSharp.Binding.Failure.{}.{}", sem.id, c.id),
                        mapped.clone(),
                        BOOL,
                    ))
                })
                .collect::<Vec<_>>();
            goals.push(call(
                &format!(
                    "Mpk.CSharp.Binding.Failure.{}.{}",
                    sem.id, route.semantic_check_id
                ),
                mapped.clone(),
                BOOL,
            ));
            p.push(
                &format!("{}.{}", owner, route.ordinal),
                "returned_error_commutation",
                subjects.clone(),
                assumptions,
                goals,
            )?;
        }
    }
    for check in &c.unmatched_source_check_ids {
        p.push(
            &format!("{}.{}", owner, check),
            "unmatched_source_check_unreachable",
            subjects.clone(),
            args.iter()
                .zip(&op.argument_type_ids)
                .map(|(x, t)| domain(t, x.clone()))
                .chain(
                    op.ordered_checks
                        .iter()
                        .take_while(|c| &c.id != check)
                        .map(|c| {
                            cv::not(call(
                                &format!("Mpk.CSharp.Binding.Failure.{}.{}", op.id, c.id),
                                args.clone(),
                                BOOL,
                            ))
                        }),
                )
                .collect(),
            vec![cv::not(call(
                &format!("Mpk.CSharp.Binding.Failure.{}.{}", op.id, check),
                args.clone(),
                BOOL,
            ))],
        )?;
    }
    // Include the reverse outcome direction and ordered returned-error recipes;
    // source-normal-only implication is insufficient for equivalence.
    p.push(
        &owner,
        "operation_outcome_equivalence",
        subjects,
        args.iter()
            .zip(&op.argument_type_ids)
            .map(|(x, t)| domain(t, x.clone()))
            .collect(),
        vec![
            equal(
                BOOL,
                normal,
                call(
                    &format!("Mpk.CSharp.Binding.Success.{}", sem.id),
                    mapped,
                    BOOL,
                ),
            ),
            call(
                &format!("Mpk.CSharp.Binding.AllOutcomes.{}", owner),
                args,
                BOOL,
            ),
        ],
    )?;
    Ok(())
}
pub(crate) fn generate_binding_vcs(
    vir: &ValidatedPracticalVir,
    construction: &ConstructionVcProgram,
) -> Result<BindingVcProgram, BindingVcError> {
    let (foundation, roots, _) = vir.construction_context();
    let closed = derive_closed_instances(foundation, roots).map_err(|_| bad())?;
    if closed.canonical_json() != vir.data_closed().canonical_json() {
        return Err(bad());
    }
    let bindings: Value = serde_json::from_slice(
        &a::canonical_practical_json_bytes(vir.binding_source()).map_err(|_| bad())?,
    )
    .map_err(|_| bad())?;
    let roots_value: Value = serde_json::from_slice(roots.canonical_json()).map_err(|_| bad())?;
    let mut p = BindingVcProgram {
        source_ir_sha256: vir.hash().into(),
        foundation_descriptor: foundation.descriptor().clone(),
        closed_roots_sha256: id_hash(&roots_value)?,
        closed_set_sha256: closed.closed_set_sha256().into(),
        semantic_bindings_sha256: text(&bindings, "binding_set_sha256")?.into(),
        counters: closed.counters().clone(),
        instances: vec![],
        representations: vec![],
        identity_projections: vec![],
        source_functions: vec![],
        sequents: vec![],
        definition_names: vec![],
        nodes: 0,
        depth: 0,
    };
    for e in closed.entries() {
        let id = text(e, "instance_id")?;
        let instance = FoundationInstanceVc {
            instance_id: id.into(),
            argument_ids: strings(e, "argument_ids")?,
            dependency_ids: strings(e, "dependency_ids")?,
            provenance_ids: strings(e, "provenance_ids")?,
            type_definition: e["type_definition"].clone(),
            operation_definitions: e["operation_definitions"]
                .as_array()
                .ok_or_else(bad)?
                .clone(),
            counters: e["counters"].clone(),
        };
        p.push(
            id,
            "concrete_type_equivalence",
            vec![subject("value", id)],
            vec![],
            vec![equal(
                BOOL,
                domain(id, var(0, id)),
                call(
                    &format!(
                        "Mpk.CSharp.Concrete.Type.{}",
                        id_hash(&instance.type_definition)?
                    ),
                    vec![var(0, id)],
                    BOOL,
                ),
            )],
        )?;
        for op in &instance.operation_definitions {
            let types = strings(op, "argument_type_ids")?;
            let result = text(op, "normal_result_type_id")?;
            let subjects = types
                .iter()
                .enumerate()
                .map(|(i, t)| subject(&format!("operand.{i}"), t))
                .collect();
            let args = types
                .iter()
                .enumerate()
                .map(|(i, t)| var(i, t))
                .collect::<Vec<_>>();
            let sig = vir
                .operation_signatures()
                .iter()
                .find(|s| s.id == text(op, "id").unwrap_or(""));
            // Definitions also include uninvoked operations of reachable instances.
            let guards = sig
                .map(|sig| {
                    sig.ordered_checks
                        .iter()
                        .map(|c| {
                            cv::not(call(
                                &format!("Mpk.CSharp.Concrete.Failure.{}.{}", sig.id, c.id),
                                args.clone(),
                                BOOL,
                            ))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            p.push(
                text(op, "id")?,
                "concrete_definition_equivalence",
                subjects,
                types
                    .iter()
                    .zip(&args)
                    .map(|(t, x)| domain(t, x.clone()))
                    .collect(),
                vec![
                    implication(
                        cv::combine(&guards, true),
                        equal(
                            result,
                            call(text(op, "id")?, args.clone(), result),
                            call(
                                &format!("Mpk.CSharp.Concrete.Definition.{}", id_hash(op)?),
                                args.clone(),
                                result,
                            ),
                        ),
                    ),
                    call(
                        &format!("Mpk.CSharp.Concrete.AllOutcomes.{}", text(op, "id")?),
                        args,
                        BOOL,
                    ),
                ],
            )?;
        }
        p.instances.push(instance);
    }
    for projection in vir.binding_projections() {
        if projection.binding_id == "binding.identity" {
            let ty = &projection.source_type_id;
            p.push(
                &projection.id,
                "identity_projection",
                vec![subject("value", ty)],
                vec![domain(ty, var(0, ty))],
                vec![
                    equal(ty, project(projection, var(0, ty)), var(0, ty)),
                    equal(ty, reconstruct(projection, var(0, ty)), var(0, ty)),
                ],
            )?;
            p.identity_projections.push(projection.clone());
            continue;
        }

        let binding = bindings["bindings"]
            .as_array()
            .ok_or_else(bad)?
            .iter()
            .find(|b| {
                format!("binding.{}", b["binding_sha256"].as_str().unwrap_or(""))
                    == projection.binding_id
            })
            .ok_or_else(bad)?;
        let source = &roots_value["source_types"][&projection.source_type_id];
        if source.is_null() {
            return Err(bad());
        }
        let invariant = construction
            .types()
            .iter()
            .find(|t| t.type_id == projection.source_type_id);
        representation(&mut p, projection, binding, source, invariant)?;
        let commutations = vir
            .binding_commutations()
            .iter()
            .filter(|c| c.binding_id == projection.binding_id)
            .cloned()
            .collect::<Vec<_>>();
        for c in &commutations {
            commutation(&mut p, c, vir.binding_projections())?;
        }
        p.representations.push(BindingRepresentationVc {
            projection: projection.clone(),
            binding: binding.clone(),
            source: source.clone(),
            invariant: invariant.cloned(),
            commutations,
            reconstruction_member_ids: source["members"]
                .as_array()
                .ok_or_else(bad)?
                .iter()
                .map(|m| text(m, "id").map(str::to_owned))
                .collect::<Result<Vec<_>, _>>()?,
            rule: "exact_source_provenance_and_members; closed_semantic_role; invariant_guarded_total_projection; all_observable_fields_reconstructed; ordered_normal_error_exception_commutation; actual_clr_default; bounded_value_predicates".into(),
        });
    }
    if !p.representations.is_empty() {
        p.source_functions = vir.functions().to_vec();
    }
    let mut names = BTreeSet::new();
    for s in &p.sequents {
        for t in s.assumptions.iter().chain(&s.goals) {
            cv::register_term(t, &mut names, s.subjects.len(), &mut p.depth);
        }
    }
    p.definition_names = names.into_iter().collect();
    p.sequents.sort_by(|a, b| a.id.cmp(&b.id));
    if p.sequents.windows(2).any(|s| s[0].id == s[1].id) {
        return Err(bad());
    }
    if p.nodes > 262144
        || p.depth > 256
        || p.declarations() > 8192
        || p.canonical_bytes().len() > 16 * 1024 * 1024
    {
        return Err(BindingVcError::Limit);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    // A finite countermodel for the universally quantified normal commutation
    // equation, independently of source execution and of representation role.
    #[test]
    fn each_role_commutation_equation_detects_a_wrong_source_operation() {
        fn eval(t: &ContractTerm, input: i64, broken: bool) -> i64 {
            if let ContractTerm::Var { index, .. } = t {
                assert_eq!(*index, 0);
                return input;
            }
            let mut args = vec![];
            let mut head = t;
            while let ContractTerm::App {
                function, argument, ..
            } = head
            {
                args.push(argument.as_ref());
                head = function;
            }
            args.reverse();
            let ContractTerm::Const { name, .. } = head else {
                panic!()
            };
            let values = args
                .iter()
                .map(|a| eval(a, input, broken))
                .collect::<Vec<_>>();
            if name == "source.operation" {
                return values[0] + 1 + i64::from(broken);
            }
            if name == "semantic.operation" {
                return values[0] + 1;
            }
            if name == "project" {
                return values[0];
            }
            if name.starts_with("Mpk.CSharp.Binding.ObserveEqual.") {
                return i64::from(values[0] == values[1]);
            }
            if name.starts_with("Mpk.CSharp.PublicDomain.")
                || name.starts_with("Mpk.CSharp.Binding.Success.")
            {
                return 1;
            }
            panic!("{name}")
        }
        for role in [
            "option",
            "lookup",
            "result",
            "validation",
            "boundary_field",
            "transition",
            "instant",
            "money",
            "bounded_sequence",
            "ordered_entry",
            "ordered_map",
            "ordered_set",
        ] {
            let st = format!("source.{role}");
            let mt = format!("semantic.{role}");
            let signature = |id: &str, tag, from: &str, to: &str| ClosedOperationSignature {
                id: id.into(),
                tag,
                argument_type_ids: vec![from.into()],
                normal_result_type_id: to.into(),
                ordered_checks: vec![],
            };
            let pr = BindingTypeProjection {
                id: "projection".into(),
                binding_id: role.into(),
                source_type_id: st.clone(),
                semantic_type_id: mt.clone(),
                project: signature("project", ClosedOperationTag::BindingProject, &st, &mt),
                reconstruct: signature(
                    "reconstruct",
                    ClosedOperationTag::BindingReconstruct,
                    &mt,
                    &st,
                ),
            };
            let c = BindingOperationCommutation {
                binding_id: role.into(),
                source_operation: signature(
                    "source.operation",
                    ClosedOperationTag::SourceCall,
                    &st,
                    &st,
                ),
                semantic_operation: signature(
                    "semantic.operation",
                    ClosedOperationTag::Foundation,
                    &mt,
                    &mt,
                ),
                operand_projection_ids: vec![pr.id.clone()],
                result_projection_id: pr.id.clone(),
                ordered_outcomes: vec![],
                returned_result: None,
                rounding_operands: vec![],
                unmatched_source_check_ids: vec![],
            };
            let mut p = BindingVcProgram {
                source_ir_sha256: String::new(),
                foundation_descriptor: Value::Null,
                closed_roots_sha256: String::new(),
                closed_set_sha256: String::new(),
                semantic_bindings_sha256: String::new(),
                counters: Value::Null,
                instances: vec![],
                representations: vec![],
                identity_projections: vec![],
                source_functions: vec![],
                sequents: vec![],
                definition_names: vec![],
                nodes: 0,
                depth: 0,
            };
            commutation(&mut p, &c, std::slice::from_ref(&pr)).unwrap();
            let s = p
                .sequents
                .iter()
                .find(|s| s.kind == "operation_normal_commutation")
                .unwrap();
            for input in [-1, 0, 7] {
                assert!(s.assumptions.iter().all(|t| eval(t, input, false) == 1));
                assert!(s.goals.iter().all(|t| eval(t, input, false) == 1));
                assert!(s.goals.iter().any(|t| eval(t, input, true) == 0));
            }
            let mut other = c.clone();
            other.semantic_operation.id = "different.semantic.operation".into();
            commutation(&mut p, &other, std::slice::from_ref(&pr)).unwrap();
            assert_eq!(
                p.sequents
                    .iter()
                    .map(|s| &s.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                p.sequents.len(),
                "each source-to-semantic mapping owns distinct obligations"
            );
            let check = |id: &str| RequiredCheck {
                id: id.into(),
                tag: RequiredCheckTag::Exception,
                failure_type_id: Some("System.ArgumentException".into()),
            };
            let mut ordered = c.clone();
            ordered.source_operation.ordered_checks =
                vec![check("pre"), check("matched"), check("late")];
            ordered.semantic_operation.ordered_checks = vec![check("mapped")];
            ordered.ordered_outcomes = vec![CheckCommutation {
                ordinal: 0,
                source_check_id: "matched".into(),
                semantic_check_id: "mapped".into(),
                failure_projection_id: None,
            }];
            ordered.unmatched_source_check_ids = vec!["pre".into(), "late".into()];
            p.sequents.clear();
            p.nodes = 0;
            commutation(&mut p, &ordered, std::slice::from_ref(&pr)).unwrap();
            let matched = p
                .sequents
                .iter()
                .find(|s| s.kind == "operation_outcome_commutation")
                .unwrap();
            assert_eq!(
                matched.assumptions.len(),
                3,
                "source prefix includes unmatched earlier checks"
            );
            let late = p
                .sequents
                .iter()
                .find(|s| {
                    s.kind == "unmatched_source_check_unreachable" && s.owner_id.ends_with(".late")
                })
                .unwrap();
            assert_eq!(
                late.assumptions.len(),
                3,
                "unreachable is guarded by the actual source prefix"
            );
        }
    }
}
