//! Append internal allocation equivalence without a public construction domain.
//! Allocation's original subjects are public i32/Bool; ownership is a separate
//! prerequisite of reads, updates and publication, never an allocation input.
use super::*;

/// Retain the entire previous certificate and append every reachable allocation
/// recipe, including uninvoked ones. Internal input domains and source ownership
/// remain unresolved; an internal result does not add either as a premise.
pub fn generate_csharp_practical_ordinary_concrete_operations_with_allocations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProgram> {
    let mut p = generate_csharp_practical_ordinary_concrete_operations(vir)?;
    let allocations = p
        .pending_operations
        .iter()
        .filter(|pending| {
            pending.reasons == [OrdinaryConcreteOperationPendingReason::InternalConstructionState]
                && pending.component.operation_id.ends_with(".allocate")
        })
        .cloned()
        .collect::<Vec<_>>();
    if allocations.is_empty() {
        return Ok(p);
    }
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b: Builder::resume(p.certificate_bytes())?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    let boolean_equal = p
        .source_observations
        .iter()
        .find(|d| d.carrier.type_id == BOOL_TYPE_ID)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .equality_definition
        .clone();
    let mut symbols = p
        .public_domains
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    conditions::boolean_symbols(&mut symbols);
    for d in &p.source_observations {
        symbols.insert(
            format!("Mpk.CSharp.Binding.Equal.{}", d.carrier.type_id),
            d.equality_definition.clone(),
        );
    }
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut added = BTreeSet::new();
    for allocation in allocations {
        let c = &allocation.component;
        if c.argument_type_ids != [I32_TYPE_ID, BOOL_TYPE_ID]
            || c.result_type_id != allocation.instance_id
            || !r.internal(&c.result_type_id)
            || c.currency_predicate_argument_type_id.is_some()
            || c.failures.len() != 2
            || c.failures
                .iter()
                .zip(["negative_length", "construction_bound"])
                .any(|(f, label)| {
                    f.label != label || f.definition.is_none() || f.argument_indices != [0]
                })
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let carrier = r
            .carriers
            .get(&c.result_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        // Observe every physical leaf: length, stored cells, initialization
        // bitmap and padding. No ownership bit or erased public value is added.
        let observation =
            super::super::super::source_clauses::physical_equal(&mut r.b, carrier.depth)?;
        symbols.insert(
            format!("Mpk.CSharp.Binding.Equal.{}", c.result_type_id),
            observation,
        );
        let instance = p
            .instances
            .iter()
            .find(|i| i.instance_id == allocation.instance_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let definition = emit_operation(
            &mut r,
            instance,
            &allocation.recipe,
            allocation.component,
            &mut symbols,
            Some(&boolean_equal),
        )?;
        let sequent = vc
            .sequents()
            .iter()
            .find(|s| {
                s.kind == "concrete_definition_equivalence"
                    && s.owner_id == definition.component.operation_id
            })
            .ok_or(OrdinaryCarrierError::Linkage)?;
        // Both complete original goals and every original domain/normal guard
        // are reconstructed, rather than replacing the allocation with true.
        p.conditions.push(conditions::operation_obligation(
            &mut r,
            sequent,
            &symbols,
            "concrete_operation",
        )?);
        added.insert(definition.component.operation_id.clone());
        p.definitions.push(definition);
    }
    let order = vc
        .sequents()
        .iter()
        .enumerate()
        .map(|(i, s)| (s.owner_id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    p.definitions
        .sort_by_key(|d| order[d.component.operation_id.as_str()]);
    p.conditions
        .sort_by_key(|c| order[c.sequent.owner_id.as_str()]);
    p.pending_operations
        .retain(|o| !added.contains(&o.component.operation_id));
    p.pending_condition_ids
        .retain(|id| !p.conditions.iter().any(|c| &c.sequent.id == id));
    p.static_transformers = p
        .static_transformers
        .checked_add(r.b.static_transformers)
        .ok_or(OrdinaryCarrierError::Limit)?;
    p.certificate = r.b.finish()?;
    p.certificate_sha256 = mpk_cert::hash_hex(&mpk_cert::certificate_hash(&p.certificate));
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_concrete_operations_with_allocations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_concrete_operations_with_allocations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub fn generate_csharp_practical_ordinary_concrete_operation_proofs_with_allocations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProofProgram> {
    let program = generate_csharp_practical_ordinary_concrete_operations_with_allocations(vir)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    proofs::emit(
        &program,
        &layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.depth))
            .collect(),
    )
}

pub fn import_csharp_practical_ordinary_concrete_operation_proofs_with_allocations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteOperationProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected =
        generate_csharp_practical_ordinary_concrete_operation_proofs_with_allocations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
