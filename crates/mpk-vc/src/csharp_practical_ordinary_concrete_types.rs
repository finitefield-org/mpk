//! Exact closed foundation type predicates and the original W06 equivalence
//! conditions. A recipe identity selects a compiled domain, never a truth value.
use super::super::super::source_clauses;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteTypeDefinition {
    pub instance: FoundationInstanceVc,
    pub carrier: OrdinaryCarrier,
    pub symbol: String,
    pub definition: String,
    pub public_domain: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConcreteTypeProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    construction_sha256: String,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    definitions: Vec<OrdinaryConcreteTypeDefinition>,
    /// Internal construction states require their separate ownership model.
    pending_type_instances: Vec<FoundationInstanceVc>,
    conditions: Vec<OrdinaryBindingCondition>,
    pending_condition_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryConcreteTypeProgram {
    pub fn definitions(&self) -> &[OrdinaryConcreteTypeDefinition] {
        &self.definitions
    }
    pub fn public_domains(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.public_domains
    }
    pub fn source_clauses(&self) -> &[OrdinarySourceClauseDefinition] {
        &self.source_clauses
    }
    pub fn pending_type_instances(&self) -> &[FoundationInstanceVc] {
        &self.pending_type_instances
    }
    pub fn conditions(&self) -> &[OrdinaryBindingCondition] {
        &self.conditions
    }
    pub fn pending_condition_ids(&self) -> &[String] {
        &self.pending_condition_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary concrete type program")
    }
}

pub fn generate_csharp_practical_ordinary_concrete_types(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    // The carrier generator reconstructs the exact substituted representation,
    // child types and frozen role bounds from the independently closed VIR.
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, storage, cache, source_clauses, construction_sha256) =
        source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
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
        storage,
    };
    cache.seed(&mut r)?;
    if !r.b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
        r.b.helpers(5)?;
    }
    ordered_fold::auxiliary(&mut r.b)?;
    let (mut r, public_domains) =
        super::super::domains::emit_binding_domains(r, layouts.carriers(), &source_clauses)?;
    let mut symbols = public_domains
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    conditions::boolean_symbols(&mut symbols);
    symbols.insert(
        "Mpk.CSharp.Binding.Equal.mpk.csharp.value.bool.v1".into(),
        r.raw(1, false)?.equal,
    );
    let mut definitions = vec![];
    let mut pending_type_instances = vec![];
    for instance in vc.instances() {
        if r.internal(&instance.instance_id) {
            pending_type_instances.push(instance.clone());
            continue;
        }
        let domain = public_domains
            .iter()
            .find(|d| d.carrier.type_id == instance.instance_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if instance.type_definition["id"] != instance.instance_id {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let hash = hash_value(
            HashDomain::new("MPK-CSHARP-CONCRETE-DEFINITION-1.0"),
            &instance.type_definition,
        )
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
        let symbol = format!("Mpk.CSharp.Concrete.Type.{hash}");
        if !vc.definition_names().contains(&symbol) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let definition = name(&symbol);
        let value = r.b.var(0)?;
        // This calls the full recursive definition: tag/active payload, padding,
        // role/cumulative bounds, scalar rules and child source public clauses.
        // No recipe hash comparison or precomputed admission flag is evaluated.
        let body = call(&mut r.b, &domain.valid_definition, vec![value])?;
        define(&mut r.b, &definition, &[domain.carrier.depth], 0, body)?;
        if symbols.insert(symbol.clone(), definition.clone()).is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        definitions.push(OrdinaryConcreteTypeDefinition {
            instance: instance.clone(),
            carrier: domain.carrier.clone(),
            symbol,
            definition,
            public_domain: domain.valid_definition.clone(),
        });
    }
    let mut conditions = vec![];
    let mut pending_condition_ids = vec![];
    for sequent in vc.sequents() {
        if sequent.kind == "concrete_type_equivalence"
            && definitions
                .iter()
                .any(|d| d.instance.instance_id == sequent.owner_id)
        {
            conditions.push(reconstruction::obligation(
                &mut r,
                sequent,
                &symbols,
                "concrete_type",
            )?);
        } else {
            pending_condition_ids.push(sequent.id.clone());
        }
    }
    let pending_proof_ids = vc.sequents().iter().map(|s| s.id.clone()).collect();
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryConcreteTypeProgram {
        schema: "mpk.csharp.ordinary_concrete_types.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        construction_sha256,
        source_clauses,
        public_domains,
        definitions,
        pending_type_instances,
        conditions,
        pending_condition_ids,
        pending_proof_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_concrete_types(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConcreteTypeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_concrete_types(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
