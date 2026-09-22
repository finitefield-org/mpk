//! W06 representation sequents composed from exact public domains, projections,
//! guards and canonical order. Every original proof obligation stays pending.
use super::super::super::{binding_projections, source_clauses};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingCondition {
    pub sequent: BindingSequent,
    pub assumption_definitions: Vec<String>,
    pub goal_definitions: Vec<String>,
    /// Bool predicate over the original subjects (closed when empty):
    /// assumptions imply all goals.
    /// Defining this function supplies no universally quantified proof.
    pub condition_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingConditionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    construction_sha256: String,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    projections: Vec<OrdinaryBindingProjectionDefinition>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    predicates: Vec<OrdinaryBindingPredicate>,
    conditions: Vec<OrdinaryBindingCondition>,
    unresolved_vc_symbols: Vec<String>,
    pending_condition_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingConditionProgram {
    pub fn conditions(&self) -> &[OrdinaryBindingCondition] {
        &self.conditions
    }
    pub fn projections(&self) -> &[OrdinaryBindingProjectionDefinition] {
        &self.projections
    }
    pub fn public_domains(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.public_domains
    }
    pub fn predicates(&self) -> &[OrdinaryBindingPredicate] {
        &self.predicates
    }
    pub fn pending_condition_ids(&self) -> &[String] {
        &self.pending_condition_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn unresolved_vc_symbols(&self) -> &[String] {
        &self.unresolved_vc_symbols
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary binding conditions")
    }
}

pub(super) fn boolean_symbols(symbols: &mut BTreeMap<String, String>) {
    // W06's typed Boolean connectives resolve to the checked foundation. They
    // are never substituted with observations or producer-supplied truth values.
    for (symbol, definition) in [
        ("Not", "not"),
        ("And", "and"),
        ("Or", "or"),
        ("true", "true"),
        ("false", "false"),
    ] {
        symbols.insert(
            format!("Mpk.CSharp.Bool.{symbol}"),
            format!("Std.Bool.{definition}"),
        );
    }
}

pub fn generate_csharp_practical_ordinary_binding_conditions(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingConditionProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, storage, cache, source_clauses, construction_sha256) =
        source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
    let (b, projections) = binding_projections::emit_binding_projections(vir, &vc, &layouts, b)?;
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
    let mut a = assemble_relations(r, projections, &vc, &layouts)?;
    guards::emit_guards(&mut a, &vc)?;
    orders::emit_orders(&mut a, &vc)?;
    let (mut r, public_domains) =
        super::super::domains::emit_binding_domains(a.r, layouts.carriers(), &source_clauses)?;
    let predicates = a.predicates.into_values().collect::<Vec<_>>();
    let mut symbols = predicates
        .iter()
        .map(|p| (p.symbol.clone(), p.definition.clone()))
        .chain(
            public_domains
                .iter()
                .map(|d| (d.symbol.clone(), d.valid_definition.clone())),
        )
        .collect::<BTreeMap<_, _>>();
    for p in &a.projections {
        symbols.insert(
            p.projection.project.id.clone(),
            p.project_definition.clone(),
        );
        if let Some(reconstruct) = &p.reconstruct_definition {
            symbols.insert(p.projection.reconstruct.id.clone(), reconstruct.clone());
        }
    }
    boolean_symbols(&mut symbols);
    let mut conditions = vec![];
    let mut pending_condition_ids = vec![];
    for sequent in vc.sequents() {
        if matches!(
            sequent.kind.as_str(),
            "member_agreement"
                | "exactly_one_arm"
                | "tag_payload_agreement"
                | "bound"
                | "canonical_order_and_uniqueness"
                | "nonempty_invalid"
        ) {
            conditions.push(reconstruction::obligation(
                &mut r,
                sequent,
                &symbols,
                "representation",
            )?);
        } else {
            pending_condition_ids.push(sequent.id.clone());
        }
    }
    let unresolved_vc_symbols = vc
        .definition_names()
        .iter()
        .filter(|s| !symbols.contains_key(*s))
        .cloned()
        .collect();
    let pending_proof_ids = vc.sequents().iter().map(|s| s.id.clone()).collect();
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryBindingConditionProgram {
        schema: "mpk.csharp.ordinary_binding_conditions.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        construction_sha256,
        source_clauses,
        projections: a.projections,
        public_domains,
        predicates,
        conditions,
        unresolved_vc_symbols,
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

pub fn import_csharp_practical_ordinary_binding_conditions(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingConditionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_binding_conditions(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
