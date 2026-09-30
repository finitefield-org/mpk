//! Core propositions for exact pattern premises and source conclusions.
//! Projection theorems expose premises; they do not establish execution or
//! discharge the separately retained source-refinement obligation.
use super::*;

const EQ: &str = "Std.Eq";
const AND: &str = "Std.Logic.And";
const AND_REC: &str = "Std.Logic.And.rec";

#[path = "csharp_practical_ordinary_control_pattern_environment.rs"]
mod environment;
pub use environment::{OrdinaryControlPatternEnvironment, OrdinaryControlPatternEnvironmentField};

#[derive(Clone, Copy)]
pub(super) enum ProofEnvironment {
    None,
    SeparateArguments,
    Packed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternPremiseProof {
    pub source: OrdinaryControlStepComponent,
    pub theorem: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternProofType {
    pub source_sequent_id: String,
    pub function_id: String,
    pub native_edge_id: String,
    pub arguments: Vec<ControlBinding>,
    /// The complete ordered native, capture and observation premises.
    pub components: Vec<OrdinaryControlStepComponent>,
    /// Original goal arguments, projected from this path's full environment.
    pub goal_argument_indices: Vec<usize>,
    pub goal_definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packed_environment: Option<OrdinaryControlPatternEnvironment>,
    /// A function from physical arguments to an ordinary conjunction of their
    /// component truth propositions. It does not assert that they hold.
    pub scope_proposition_definition: Option<String>,
    /// Closed Pi type: all physical arguments, scope proof, original goal truth.
    pub refinement_proposition_definition: Option<String>,
    pub premise_proofs: Vec<OrdinaryControlPatternPremiseProof>,
    pub pending_definition_reasons: Vec<String>,
    pub source_refinement_proof_pending: bool,
    pub execution_establishment_proof_pending: bool,
}

fn truth(b: &mut Builder, predicate: &str, indices: &[usize], count: usize, offset: u32) -> R<u32> {
    let args = indices
        .iter()
        .map(|&index| {
            if index >= count {
                return Err(OrdinaryCarrierError::Linkage);
            }
            b.var((count - 1 - index) as u32 + offset)
        })
        .collect::<R<Vec<_>>>()?;
    let actual = call(b, predicate, args)?;
    let yes = bit(b, true)?;
    let boolean = b.boolean;
    call(b, EQ, vec![boolean, actual, yes])
}

fn premises(b: &mut Builder, p: &OrdinaryControlPatternExecutionScope, offset: u32) -> R<Vec<u32>> {
    p.components
        .iter()
        .map(|c| {
            truth(
                b,
                &c.definition,
                &c.argument_indices,
                p.arguments.len(),
                offset,
            )
        })
        .collect()
}

// Nonempty right-associated conjunction; no true sentinel or omitted premise.
fn conjunctions(b: &mut Builder, terms: &[u32]) -> R<Vec<u32>> {
    let mut tails = terms.to_vec();
    if tails.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    for index in (0..tails.len() - 1).rev() {
        tails[index] = call(b, AND, vec![terms[index], tails[index + 1]])?;
    }
    Ok(tails)
}

fn projection(b: &mut Builder, terms: &[u32], shifted: &[u32], selected: usize) -> R<u32> {
    if terms.len() != shifted.len() || selected >= terms.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let tails = conjunctions(b, terms)?;
    let shifted_tails = conjunctions(b, shifted)?;
    let mut proof = b.var(0)?;
    if terms.len() == 1 {
        return Ok(proof);
    }
    for index in 0..=selected.min(terms.len() - 2) {
        let left = terms[index];
        let right = tails[index + 1];
        let take_left = index == selected;
        let target = if take_left { left } else { right };
        let variable = b.var(u32::from(take_left))?;
        let minor = b.lam(shifted_tails[index + 1], variable)?;
        let minor = b.lam(left, minor)?;
        proof = call(b, AND_REC, vec![left, right, target, minor, proof])?;
    }
    Ok(proof)
}

fn emit_path(
    b: &mut Builder,
    scope: &OrdinaryControlPatternScope,
    p: &OrdinaryControlPatternExecutionScope,
    sequent: &OrdinaryControlSequentDefinition,
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
) -> R<OrdinaryControlPatternProofType> {
    let [goal] = sequent.goals.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    if !sequent.assumptions.is_empty()
        || !goal.pending_constant_names.is_empty()
        || scope.original_pattern_predicate_pending
        || !p.pending_observation_binding_indices.is_empty()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let goal_definition = goal
        .definition
        .clone()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let goal_argument_indices = goal
        .source
        .bindings
        .iter()
        .map(|a| {
            p.arguments
                .iter()
                .position(|b| b == a)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let mut result = OrdinaryControlPatternProofType {
        source_sequent_id: scope.source_sequent_id.clone(),
        function_id: scope.function_id.clone(),
        native_edge_id: p.source_execution.edge_id.clone(),
        arguments: p.arguments.clone(),
        components: p.components.clone(),
        goal_argument_indices,
        goal_definition,
        packed_environment: None,
        scope_proposition_definition: None,
        refinement_proposition_definition: None,
        premise_proofs: vec![],
        pending_definition_reasons: vec![],
        source_refinement_proof_pending: true,
        execution_establishment_proof_pending: true,
    };
    // The proof hypothesis is an additional binder. Do not omit a large native
    // environment to force it into a type that the frozen kernel can check.
    if p.arguments.len() + 1 > 256 {
        result
            .pending_definition_reasons
            .push("combined_binder_limit".into());
        return Ok(result);
    }
    if !p.pending_definition_reasons.is_empty() {
        result.pending_definition_reasons = p.pending_definition_reasons.clone();
        return Ok(result);
    }
    let binders = p
        .arguments
        .iter()
        .map(|a| {
            let depth = carriers
                .get(a.type_id.as_str())
                .ok_or(OrdinaryCarrierError::Linkage)?
                .depth;
            b.cube(depth)
        })
        .collect::<R<Vec<_>>>()?;
    let terms = premises(b, p, 0)?;
    let scope_type = conjunctions(b, &terms)?[0];
    let identity = (
        &scope.source_sequent_id,
        &p.source_execution.edge_id,
        &p.components,
        &p.arguments,
    );
    let scope_name = name("PatternScopeProposition", &identity);
    let mut value = scope_type;
    let mut ty = b.sort;
    for &binder in binders.iter().rev() {
        value = b.lam(binder, value)?;
        ty = b.pi(binder, ty)?;
    }
    b.define(&scope_name, ty, value)?;
    result.scope_proposition_definition = Some(scope_name);

    let goal = truth(
        b,
        &result.goal_definition,
        &result.goal_argument_indices,
        p.arguments.len(),
        1,
    )?;
    let mut refinement = b.pi(scope_type, goal)?;
    for &binder in binders.iter().rev() {
        refinement = b.pi(binder, refinement)?;
    }
    let refinement_name = name(
        "PatternSourceRefinementType",
        &(
            &identity,
            &result.goal_definition,
            &result.goal_argument_indices,
        ),
    );
    b.define(&refinement_name, b.sort, refinement)?;
    result.refinement_proposition_definition = Some(refinement_name);

    for (index, component) in p.components.iter().enumerate() {
        let expected = truth(
            b,
            &component.definition,
            &component.argument_indices,
            p.arguments.len(),
            1,
        )?;
        let mut ty = b.pi(scope_type, expected)?;
        let under_hypothesis = premises(b, p, 1)?;
        let under_minor = premises(b, p, 2)?;
        let proof = projection(b, &under_hypothesis, &under_minor, index)?;
        let mut proof = b.lam(scope_type, proof)?;
        for &binder in binders.iter().rev() {
            ty = b.pi(binder, ty)?;
            proof = b.lam(binder, proof)?;
        }
        let theorem = name("PatternScopePremiseProof", &(&identity, index));
        super::super::super::ownership_proofs::publish_theorem(b, &theorem, ty, proof)?;
        result
            .premise_proofs
            .push(OrdinaryControlPatternPremiseProof {
                source: component.clone(),
                theorem,
            });
    }
    Ok(result)
}

pub(super) fn emit(
    c: &mut Clauses<'_>,
    scopes: &[OrdinaryControlPatternScope],
    sequents: &[OrdinaryControlSequentDefinition],
    mode: ProofEnvironment,
) -> R<Vec<OrdinaryControlPatternProofType>> {
    if scopes.is_empty() {
        return Ok(vec![]);
    }
    super::super::super::ownership_proofs::equality(&mut c.b)?;
    super::super::super::ownership_proofs::logic(&mut c.b)?;
    let mut types = vec![];
    for scope in scopes {
        let sequent = sequents
            .iter()
            .find(|s| s.source.id == scope.source_sequent_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for path in &scope.executions {
            types.push(match mode {
                ProofEnvironment::Packed => {
                    environment::emit_path(&mut c.b, scope, path, sequent, &c.carriers)?
                }
                ProofEnvironment::SeparateArguments => {
                    emit_path(&mut c.b, scope, path, sequent, &c.carriers)?
                }
                ProofEnvironment::None => return Err(OrdinaryCarrierError::Linkage),
            });
        }
    }
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(wrong: bool) -> Vec<u8> {
        let mut b = Builder::new().unwrap();
        super::super::super::super::ownership_proofs::equality(&mut b).unwrap();
        super::super::super::super::ownership_proofs::logic(&mut b).unwrap();
        let boolean = b.boolean;
        let mut truth_at = |offset: u32| {
            (0..3u32)
                .map(|i| {
                    let value = b.var(2 - i + offset).unwrap();
                    let yes = bit(&mut b, true).unwrap();
                    call(&mut b, EQ, vec![boolean, value, yes]).unwrap()
                })
                .collect::<Vec<_>>()
        };
        let terms = truth_at(0);
        let under_hypothesis = truth_at(1);
        let under_minor = truth_at(2);
        let scope = conjunctions(&mut b, &terms).unwrap()[0];
        for selected in 0..3 {
            let expected = under_hypothesis[selected];
            let mut ty = b.pi(scope, expected).unwrap();
            let proof = if wrong {
                b.var(0).unwrap()
            } else {
                projection(&mut b, &under_hypothesis, &under_minor, selected).unwrap()
            };
            let mut proof = b.lam(scope, proof).unwrap();
            for _ in 0..3 {
                ty = b.pi(b.boolean, ty).unwrap();
                proof = b.lam(b.boolean, proof).unwrap();
            }
            super::super::super::super::ownership_proofs::publish_theorem(
                &mut b,
                &format!("Mpk.PatternScope.TestProjection.P{selected}"),
                ty,
                proof,
            )
            .unwrap();
        }
        b.finish().unwrap()
    }

    #[test]
    fn pattern_scope_projections_are_kernel_checked_under_free_arguments() {
        let bytes = sample(false);
        let report = mpk_kernel::verify_certificate_bytes(&bytes).unwrap();
        assert_eq!(report.axiom_count, 0);
        let wrong = sample(true);
        let error = mpk_kernel::verify_certificate_bytes(&wrong).unwrap_err();
        assert_eq!(error.kind(), mpk_kernel::VerificationErrorKind::CoreCheck);
        if let Ok(path) = std::env::var("MPK_W09_PATTERN_PROOF_TYPES_UNIT_OUTPUT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(std::path::Path::new(&path).join("projection.mpcert"), bytes).unwrap();
            std::fs::write(
                std::path::Path::new(&path).join("wrong-projection.mpcert"),
                wrong,
            )
            .unwrap();
        }
    }
}
