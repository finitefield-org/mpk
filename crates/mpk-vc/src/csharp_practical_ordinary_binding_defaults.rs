//! Original closed W06 default conditions over actual recursive CLR defaults.
//! A definition is not a proof of public admission or absence of forbidden uses.
use super::super::super::super::defaults;
use super::super::super::{binding_projections, source_clauses};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingActualDefault {
    pub projection_id: String,
    pub source_type_id: String,
    pub declared_arm: String,
    pub symbol: String,
    /// The exact structural default definition, without an admission assumption.
    /// None means the CLR default is unavailable in this carrier/default graph.
    pub definition: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryBindingDefaultPendingReason {
    SourceUseProofRequired,
    UnrepresentableClrDefault,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingDefaultPending {
    pub sequent: BindingSequent,
    pub reason: OrdinaryBindingDefaultPendingReason,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingDefaultProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    construction_sha256: String,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    defaults: Vec<OrdinaryDefaultDefinition>,
    actual_defaults: Vec<OrdinaryBindingActualDefault>,
    projections: Vec<OrdinaryBindingProjectionDefinition>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    predicates: Vec<OrdinaryBindingPredicate>,
    conditions: Vec<OrdinaryBindingCondition>,
    pending_defaults: Vec<OrdinaryBindingDefaultPending>,
    unresolved_vc_symbols: Vec<String>,
    pending_condition_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingDefaultProgram {
    pub fn defaults(&self) -> &[OrdinaryDefaultDefinition] {
        &self.defaults
    }
    pub fn actual_defaults(&self) -> &[OrdinaryBindingActualDefault] {
        &self.actual_defaults
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
    pub fn conditions(&self) -> &[OrdinaryBindingCondition] {
        &self.conditions
    }
    pub fn pending_defaults(&self) -> &[OrdinaryBindingDefaultPending] {
        &self.pending_defaults
    }
    pub fn unresolved_vc_symbols(&self) -> &[String] {
        &self.unresolved_vc_symbols
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
        serde_json::to_vec(self).expect("ordinary binding default conditions")
    }
}

fn closed_term(b: &mut Builder, term: &ContractTerm, symbols: &BTreeMap<String, String>) -> R<u32> {
    match term {
        ContractTerm::Const { name, .. } => {
            b.constant(symbols.get(name).ok_or(OrdinaryCarrierError::Linkage)?)
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            let function = closed_term(b, function, symbols)?;
            let argument = closed_term(b, argument, symbols)?;
            b.app(function, vec![argument])
        }
        // The original actual_default sequent has no receiver or binders.
        _ => Err(OrdinaryCarrierError::Linkage),
    }
}

fn closed_condition(
    b: &mut Builder,
    sequent: &BindingSequent,
    symbols: &BTreeMap<String, String>,
) -> R<OrdinaryBindingCondition> {
    if !sequent.subjects.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut groups = vec![];
    let mut conjunctions = vec![];
    for (kind, terms) in [
        ("assumption", &sequent.assumptions),
        ("goal", &sequent.goals),
    ] {
        let mut names = vec![];
        let mut combined = bit(b, true)?;
        for (i, term) in terms.iter().enumerate() {
            if term.type_id() != "mpk.csharp.value.bool.v1" {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let definition = name(&format!("actual_default.{}.{kind}.{i}", sequent.id));
            let body = closed_term(b, term, symbols)?;
            define(b, &definition, &[], 0, body)?;
            combined = and(b, combined, body)?;
            names.push(definition);
        }
        groups.push(names);
        conjunctions.push(combined);
    }
    let no_assumptions = call(
        b,
        symbols
            .get("Mpk.CSharp.Bool.Not")
            .ok_or(OrdinaryCarrierError::Linkage)?,
        vec![conjunctions[0]],
    )?;
    let body = call(
        b,
        symbols
            .get("Mpk.CSharp.Bool.Or")
            .ok_or(OrdinaryCarrierError::Linkage)?,
        vec![no_assumptions, conjunctions[1]],
    )?;
    let condition_definition = name(&format!("actual_default.{}.condition", sequent.id));
    define(b, &condition_definition, &[], 0, body)?;
    Ok(OrdinaryBindingCondition {
        sequent: sequent.clone(),
        assumption_definitions: groups.remove(0),
        goal_definitions: groups.remove(0),
        condition_definition,
    })
}

pub fn generate_csharp_practical_ordinary_binding_defaults(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingDefaultProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (mut b, storage, cache, source_clauses, construction_sha256) =
        source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
    let defaults = defaults::emit_defaults(vir, layouts.carriers(), &mut b)?;
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
    conditions::boolean_symbols(&mut symbols);
    let mut actual_defaults = vec![];
    for rep in vc.representations() {
        let source_type_id = &rep.projection.source_type_id;
        let default = defaults
            .iter()
            .find(|d| d.carrier.type_id == *source_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let symbol = format!("Mpk.CSharp.Binding.ActualDefault.{source_type_id}");
        let definition = default
            .structural_candidate
            .as_ref()
            .map(|d| d.definition.clone());
        if let Some(definition) = &definition {
            symbols.insert(symbol.clone(), definition.clone());
        }
        actual_defaults.push(OrdinaryBindingActualDefault {
            projection_id: rep.projection.id.clone(),
            source_type_id: source_type_id.clone(),
            declared_arm: rep.binding["default_arm"]
                .as_str()
                .ok_or(OrdinaryCarrierError::Linkage)?
                .into(),
            symbol,
            definition,
        });
    }
    let mut conditions = vec![];
    let mut pending_defaults = vec![];
    let mut pending_condition_ids = vec![];
    for sequent in vc.sequents() {
        if sequent.kind != "actual_default" {
            pending_condition_ids.push(sequent.id.clone());
            continue;
        }
        let actual = actual_defaults
            .iter()
            .find(|d| d.projection_id == sequent.owner_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let reason = if actual.declared_arm == "ineligible" {
            // A declaration cannot establish absence of a forbidden source use.
            Some(OrdinaryBindingDefaultPendingReason::SourceUseProofRequired)
        } else if actual.definition.is_none() {
            Some(OrdinaryBindingDefaultPendingReason::UnrepresentableClrDefault)
        } else {
            None
        };
        if let Some(reason) = reason {
            pending_condition_ids.push(sequent.id.clone());
            pending_defaults.push(OrdinaryBindingDefaultPending {
                sequent: sequent.clone(),
                reason,
            });
        } else {
            conditions.push(closed_condition(&mut r.b, sequent, &symbols)?);
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
    let p = OrdinaryBindingDefaultProgram {
        schema: "mpk.csharp.ordinary_binding_defaults.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        construction_sha256,
        source_clauses,
        defaults,
        actual_defaults,
        projections: a.projections,
        public_domains,
        predicates,
        conditions,
        pending_defaults,
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

pub fn import_csharp_practical_ordinary_binding_defaults(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingDefaultProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_binding_defaults(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
