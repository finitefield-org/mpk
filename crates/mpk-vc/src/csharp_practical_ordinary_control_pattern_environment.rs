//! Complete source environments addressed as finite ordinary Boolean cubes.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternEnvironmentField {
    pub argument_index: usize,
    pub type_id: String,
    pub depth: u32,
    pub read_definition: String,
    pub write_definition: String,
    /// Universal pointwise read-after-write equality, under free old state/value.
    pub write_read_theorem: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternEnvironment {
    pub depth: u32,
    pub payload_depth: u32,
    pub address_depth: u32,
    pub zero_definition: String,
    /// Exact original argument order, nominal types and independent addresses.
    pub fields: Vec<OrdinaryControlPatternEnvironmentField>,
}

fn field(
    b: &mut Builder,
    payload: u32,
    address: u32,
    index: usize,
    depth: u32,
    type_id: &str,
) -> R<OrdinaryControlPatternEnvironmentField> {
    let identity = (payload, address, index, depth);
    let read = name("PatternEnvironmentRead", &identity);
    let write = name("PatternEnvironmentWrite", &identity);
    let theorem = name("PatternEnvironmentWriteRead", &identity);
    let env_depth = payload
        .checked_add(address)
        .ok_or(OrdinaryCarrierError::Limit)?;
    if depth > payload || env_depth + 2 > 256 || address > usize::BITS || index >> address != 0 {
        return Err(OrdinaryCarrierError::Limit);
    }
    if !b.globals.contains_key(&read) {
        let mut selectors = b.selectors(depth)?;
        for _ in depth..payload {
            selectors.push(bit(b, false)?);
        }
        for bit_index in 0..address {
            selectors.push(bit(b, index & (1usize << bit_index) != 0)?);
        }
        let input = b.var(depth)?;
        let body = b.app(input, selectors)?;
        let body = b.wrap_selectors(depth, body)?;
        define(b, &read, &[env_depth], depth, body)?;

        let selectors = b.selectors(env_depth)?;
        let old = b.var(env_depth + 1)?;
        let old = b.app(old, selectors.clone())?;
        let input = b.var(env_depth)?;
        let mut body = b.app(input, selectors[..depth as usize].to_vec())?;
        let no = bit(b, false)?;
        // Stored payloads have canonical zero padding. Reading never observes it.
        for &selector in &selectors[depth as usize..payload as usize] {
            body = mux(b, selector, no, body)?;
        }
        // Separate prefix tests keep read-after-write normalization on literal
        // addresses within the unchanged Boolean recursor rules.
        for bit_index in 0..address {
            let selector = selectors[(payload + bit_index) as usize];
            body = if index & (1usize << bit_index) != 0 {
                mux(b, selector, body, old)?
            } else {
                mux(b, selector, old, body)?
            };
        }
        let body = b.wrap_selectors(env_depth, body)?;
        define(b, &write, &[env_depth, depth], env_depth, body)?;

        let old = b.var(depth + 1)?;
        let input = b.var(depth)?;
        let updated = call(b, &write, vec![old, input])?;
        let selected = call(b, &read, vec![updated])?;
        let selectors = b.selectors(depth)?;
        let selected = b.app(selected, selectors.clone())?;
        let expected = b.app(input, selectors)?;
        let boolean = b.boolean;
        let mut ty = call(b, EQ, vec![boolean, selected, expected])?;
        let mut proof = call(b, "Std.Eq.refl", vec![boolean, expected])?;
        for _ in 0..depth {
            ty = b.pi(boolean, ty)?;
            proof = b.lam(boolean, proof)?;
        }
        for d in [depth, env_depth] {
            let binder = b.cube(d)?;
            ty = b.pi(binder, ty)?;
            proof = b.lam(binder, proof)?;
        }
        super::super::super::super::ownership_proofs::publish_theorem(b, &theorem, ty, proof)?;
    }
    Ok(OrdinaryControlPatternEnvironmentField {
        argument_index: index,
        type_id: type_id.into(),
        depth,
        read_definition: read,
        write_definition: write,
        write_read_theorem: theorem,
    })
}

fn layout(
    b: &mut Builder,
    arguments: &[ControlBinding],
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
) -> R<OrdinaryControlPatternEnvironment> {
    if arguments.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let depths = arguments
        .iter()
        .map(|a| {
            carriers
                .get(a.type_id.as_str())
                .map(|c| c.depth)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let payload_depth = depths
        .iter()
        .copied()
        .max()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let count = u32::try_from(arguments.len()).map_err(|_| OrdinaryCarrierError::Limit)?;
    let address_depth = address_bits(count);
    let depth = payload_depth
        .checked_add(address_depth)
        .ok_or(OrdinaryCarrierError::Limit)?;
    let zero_definition = format!("{PREFIX}.Cube.D{depth}.Zero");
    if !b.globals.contains_key(&zero_definition) {
        let no = bit(b, false)?;
        let body = b.wrap_selectors(depth, no)?;
        let ty = b.cube(depth)?;
        b.define(&zero_definition, ty, body)?;
    }
    let fields = arguments
        .iter()
        .zip(depths)
        .enumerate()
        .map(|(i, (a, d))| field(b, payload_depth, address_depth, i, d, &a.type_id))
        .collect::<R<Vec<_>>>()?;
    Ok(OrdinaryControlPatternEnvironment {
        depth,
        payload_depth,
        address_depth,
        zero_definition,
        fields,
    })
}

fn truth(
    b: &mut Builder,
    env: &OrdinaryControlPatternEnvironment,
    predicate: &str,
    indices: &[usize],
    offset: u32,
) -> R<u32> {
    let state = b.var(offset)?;
    let args = indices
        .iter()
        .map(|&i| {
            let field = env
                .fields
                .get(i)
                .filter(|f| f.argument_index == i)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            call(b, &field.read_definition, vec![state])
        })
        .collect::<R<Vec<_>>>()?;
    let actual = call(b, predicate, args)?;
    let yes = bit(b, true)?;
    let boolean = b.boolean;
    call(b, EQ, vec![boolean, actual, yes])
}

fn premises(
    b: &mut Builder,
    env: &OrdinaryControlPatternEnvironment,
    p: &OrdinaryControlPatternExecutionScope,
    offset: u32,
) -> R<Vec<u32>> {
    p.components
        .iter()
        .map(|c| truth(b, env, &c.definition, &c.argument_indices, offset))
        .collect()
}

pub(super) fn emit_path(
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
    if p.pending_definition_reasons
        .iter()
        .any(|r| r != "combined_binder_limit")
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
    let env = layout(b, &p.arguments, carriers)?;
    let binder = b.cube(env.depth)?;
    let identity = (
        &scope.source_sequent_id,
        &p.source_execution.edge_id,
        &p.components,
        &p.arguments,
        &env,
    );
    let terms = premises(b, &env, p, 0)?;
    let scope_type = conjunctions(b, &terms)?[0];
    let scope_name = name("PackedPatternScopeProposition", &identity);
    let ty = b.pi(binder, b.sort)?;
    let value = b.lam(binder, scope_type)?;
    b.define(&scope_name, ty, value)?;
    let goal_term = truth(b, &env, &goal_definition, &goal_argument_indices, 1)?;
    let refinement = b.pi(scope_type, goal_term)?;
    let refinement = b.pi(binder, refinement)?;
    let refinement_name = name(
        "PackedPatternSourceRefinementType",
        &(&identity, &goal_definition, &goal_argument_indices),
    );
    b.define(&refinement_name, b.sort, refinement)?;
    let under_hypothesis = premises(b, &env, p, 1)?;
    let under_minor = premises(b, &env, p, 2)?;
    let mut premise_proofs = vec![];
    for (index, component) in p.components.iter().enumerate() {
        let ty = b.pi(scope_type, under_hypothesis[index])?;
        let ty = b.pi(binder, ty)?;
        let proof = projection(b, &under_hypothesis, &under_minor, index)?;
        let proof = b.lam(scope_type, proof)?;
        let proof = b.lam(binder, proof)?;
        let theorem = name("PackedPatternScopePremiseProof", &(&identity, index));
        super::super::super::super::ownership_proofs::publish_theorem(b, &theorem, ty, proof)?;
        premise_proofs.push(OrdinaryControlPatternPremiseProof {
            source: component.clone(),
            theorem,
        });
    }
    Ok(OrdinaryControlPatternProofType {
        source_sequent_id: scope.source_sequent_id.clone(),
        function_id: scope.function_id.clone(),
        native_edge_id: p.source_execution.edge_id.clone(),
        arguments: p.arguments.clone(),
        components: p.components.clone(),
        goal_argument_indices,
        goal_definition,
        packed_environment: Some(env),
        scope_proposition_definition: Some(scope_name),
        refinement_proposition_definition: Some(refinement_name),
        premise_proofs,
        pending_definition_reasons: vec![],
        source_refinement_proof_pending: true,
        execution_establishment_proof_pending: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csharp_practical_vir_model::ordinary_carriers::test_eval::{
        apply, bit as observed, run, V,
    };

    #[test]
    fn packed_pattern_environment_roundtrips_and_preserves_other_fields() {
        let mut b = Builder::new().unwrap();
        super::super::super::super::super::ownership_proofs::equality(&mut b).unwrap();
        let fields = (0..4)
            .map(|i| field(&mut b, 3, 2, i, i as u32, "test").unwrap())
            .collect::<Vec<_>>();
        let mut wrong = b.clone();
        let id = wrong.globals[&fields[2].write_definition] as usize;
        let input = wrong.cube(2).unwrap();
        let env = wrong.cube(5).unwrap();
        let body = wrong.var(1).unwrap();
        let body = wrong.lam(input, body).unwrap();
        let body = wrong.lam(env, body).unwrap();
        let DeclarationKind::Def { value, .. } = &mut wrong.c.declarations[id].kind else {
            panic!("write definition")
        };
        *value = body;
        let bytes = b.finish().unwrap();
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(&bytes)
                .unwrap()
                .axiom_count,
            0
        );
        let wrong_bytes = wrong.finish().unwrap();
        let error = mpk_kernel::verify_certificate_bytes(&wrong_bytes).unwrap_err();
        assert_eq!(error.kind(), mpk_kernel::VerificationErrorKind::CoreCheck);
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let mut value = V::UniformCube(false, 5);
        for f in &fields {
            let input = if f.depth == 0 {
                V::Bit(true)
            } else {
                V::UniformCube(true, f.depth)
            };
            value = run(&cert, &f.write_definition, vec![value, input]);
            for other in &fields {
                for index in 0..1usize << other.depth {
                    let mut selected = run(&cert, &other.read_definition, vec![value.clone()]);
                    for bit in 0..other.depth {
                        selected = apply(&cert, selected, V::Bit(index & (1 << bit) != 0));
                    }
                    assert_eq!(observed(selected), other.argument_index <= f.argument_index);
                }
            }
        }
        if let Ok(root) = std::env::var("MPK_W09_PACKED_PATTERN_UNIT_OUTPUT") {
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(std::path::Path::new(&root).join("roundtrip.mpcert"), bytes).unwrap();
            std::fs::write(
                std::path::Path::new(&root).join("wrong-roundtrip.mpcert"),
                wrong_bytes,
            )
            .unwrap();
        }
    }
}
