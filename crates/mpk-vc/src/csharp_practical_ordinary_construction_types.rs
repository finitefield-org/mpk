//! Append the original private construction type equations while retaining
//! every public domain and source ownership/application obligation separately.
use super::*;

pub fn generate_csharp_practical_ordinary_concrete_types_with_construction_storage(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProgram> {
    let mut p = generate_csharp_practical_ordinary_concrete_types(vir)?;
    if p.pending_type_instances.is_empty() {
        return Ok(p);
    }
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (mut b, domains) = super::super::super::domains::emit_construction_storage_domains(
        vir,
        &layouts,
        Builder::resume(p.certificate_bytes())?,
        &p.public_domains,
    )?;
    if domains.len() != p.pending_type_instances.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut symbols = p
        .public_domains
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    conditions::boolean_symbols(&mut symbols);
    let left = b.var(1)?;
    let right = b.var(0)?;
    let inverse = call(&mut b, "Std.Bool.not", vec![right])?;
    let equal = mux(&mut b, left, right, inverse)?;
    let boolean_equal = format!("{PREFIX}.ConstructionStorage.BooleanEqual");
    define(&mut b, &boolean_equal, &[0, 0], 0, equal)?;
    symbols.insert(
        format!("Mpk.CSharp.Binding.Equal.{BOOL_TYPE_ID}"),
        boolean_equal,
    );
    for instance in &p.pending_type_instances {
        let domain = domains
            .iter()
            .find(|d| d.carrier.type_id == instance.instance_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if instance.argument_ids != [domain.element_type_id.clone()]
            || instance.type_definition
                != json!({"id":instance.instance_id,"representation":{"kind":"construction","element":{"kind":"concrete","type_id":domain.element_type_id}}})
            || !domain.private_storage_only
            || !domain.ownership_pending
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        if symbols
            .insert(domain.symbol.clone(), domain.valid_definition.clone())
            .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let hash = hash_value(
            HashDomain::new("MPK-CSHARP-CONCRETE-DEFINITION-1.0"),
            &instance.type_definition,
        )
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
        let symbol = format!("Mpk.CSharp.Concrete.Type.{hash}");
        let definition = name(&symbol);
        let value = b.var(0)?;
        let body = call(&mut b, &domain.valid_definition, vec![value])?;
        define(&mut b, &definition, &[domain.carrier.depth], 0, body)?;
        if symbols.insert(symbol.clone(), definition.clone()).is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        p.definitions.push(OrdinaryConcreteTypeDefinition {
            instance: instance.clone(),
            carrier: domain.carrier.clone(),
            symbol,
            definition,
            public_domain: domain.valid_definition.clone(),
        });
    }
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    for sequent in vc.sequents().iter().filter(|s| {
        s.kind == "concrete_type_equivalence"
            && p.pending_type_instances
                .iter()
                .any(|i| i.instance_id == s.owner_id)
    }) {
        p.conditions.push(reconstruction::obligation(
            &mut r,
            sequent,
            &symbols,
            "concrete_type",
        )?);
    }
    if p.definitions.len() != vc.instances().len() || p.conditions.len() != p.definitions.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let order = vc
        .instances()
        .iter()
        .enumerate()
        .map(|(i, d)| (d.instance_id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    p.definitions
        .sort_by_key(|d| order[d.instance.instance_id.as_str()]);
    p.conditions
        .sort_by_key(|c| order[c.sequent.owner_id.as_str()]);
    p.pending_condition_ids
        .retain(|id| !p.conditions.iter().any(|c| &c.sequent.id == id));
    p.pending_type_instances.clear();
    p.construction_storage_domains = domains;
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

pub fn generate_csharp_practical_ordinary_concrete_type_proofs_with_construction_storage(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProofProgram> {
    emit_type_proofs(
        &generate_csharp_practical_ordinary_concrete_types_with_construction_storage(vir)?,
    )
}

pub fn import_csharp_practical_ordinary_concrete_types_with_construction_storage(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected =
        generate_csharp_practical_ordinary_concrete_types_with_construction_storage(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub fn import_csharp_practical_ordinary_concrete_type_proofs_with_construction_storage(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected =
        generate_csharp_practical_ordinary_concrete_type_proofs_with_construction_storage(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
