//! One ordinary program for structural/collection definitions and Money operations.
//! Shared concrete folds avoid multiplying the 16,384-transformer budget.
//! Source/public conditions and remaining units 2/4-8 stay assembly obligations.
use super::super::super::super::defaults;
use super::super::super::super::scalar_bits::emit_boundary_json_for_carriers;
use super::super::super::{construction_ops, non_templates};
use super::super::{entry_ops, money_ops, outcome_ops, sequence_ops};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDeferredFoundationInstance {
    pub instance_id: String,
    pub template_id: String,
    pub operation_ids: Vec<String>,
    /// The approved internal W09 unit that supplies these remaining definitions.
    pub internal_unit: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralFoundationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    carriers: Vec<OrdinaryCarrier>,
    storage: Vec<OrdinaryStructuralDefinition>,
    relations: Vec<OrdinaryRelationDefinition>,
    source_observations: Vec<OrdinaryObservationDefinition>,
    domains: Vec<OrdinaryDomainDefinition>,
    defaults: Vec<OrdinaryDefaultDefinition>,
    finite_operations: Vec<OrdinaryFiniteOperation>,
    sequences: Vec<OrdinarySequenceOperations>,
    constructions: Vec<OrdinaryConstructionDefinition>,
    entries: Vec<OrdinaryEntryDefinition>,
    outcomes: Vec<OrdinaryOutcomeDefinition>,
    collections: Vec<OrdinaryCollectionDefinition>,
    money: Vec<OrdinaryMoneyDefinition>,
    deferred_instances: Vec<OrdinaryDeferredFoundationInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boundary_program_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boundary_json: Option<OrdinaryJsonTokenDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_clauses: Option<Vec<OrdinarySourceClauseDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_domains: Option<Vec<OrdinaryPublicDomainDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_defaults: Option<Vec<OrdinaryPublicDefaultDefinition>>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryStructuralFoundationProgram {
    pub fn source_clauses(&self) -> Option<&[OrdinarySourceClauseDefinition]> {
        self.source_clauses.as_deref()
    }
    pub fn public_domains(&self) -> Option<&[OrdinaryPublicDomainDefinition]> {
        self.public_domains.as_deref()
    }
    pub fn public_defaults(&self) -> Option<&[OrdinaryPublicDefaultDefinition]> {
        self.public_defaults.as_deref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed structural foundation program")
    }
    pub fn carriers(&self) -> &[OrdinaryCarrier] {
        &self.carriers
    }
    pub fn storage(&self) -> &[OrdinaryStructuralDefinition] {
        &self.storage
    }
    pub fn relations(&self) -> &[OrdinaryRelationDefinition] {
        &self.relations
    }
    pub fn source_observations(&self) -> &[OrdinaryObservationDefinition] {
        &self.source_observations
    }
    pub fn domains(&self) -> &[OrdinaryDomainDefinition] {
        &self.domains
    }
    pub fn defaults(&self) -> &[OrdinaryDefaultDefinition] {
        &self.defaults
    }
    pub fn finite_operations(&self) -> &[OrdinaryFiniteOperation] {
        &self.finite_operations
    }
    pub fn sequences(&self) -> &[OrdinarySequenceOperations] {
        &self.sequences
    }
    pub fn constructions(&self) -> &[OrdinaryConstructionDefinition] {
        &self.constructions
    }
    pub fn entries(&self) -> &[OrdinaryEntryDefinition] {
        &self.entries
    }
    pub fn outcomes(&self) -> &[OrdinaryOutcomeDefinition] {
        &self.outcomes
    }
    pub fn collections(&self) -> &[OrdinaryCollectionDefinition] {
        &self.collections
    }
    pub fn money(&self) -> &[OrdinaryMoneyDefinition] {
        &self.money
    }
    pub fn deferred_instances(&self) -> &[OrdinaryDeferredFoundationInstance] {
        &self.deferred_instances
    }
    pub fn static_transformers(&self) -> usize {
        self.static_transformers
    }
    pub fn boundary_program_sha256(&self) -> Option<&str> {
        self.boundary_program_sha256.as_deref()
    }
    pub fn boundary_json(&self) -> Option<&OrdinaryJsonTokenDefinition> {
        self.boundary_json.as_ref()
    }
}

pub fn generate_csharp_practical_ordinary_structural_foundations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    generate_structural(vir, false, false)
}

/// Complete existing structural/Money definitions and the boundary lexical
/// environment share one owning Builder. Typed grammar/source/application
/// obligations remain pending; this is not final W09 certificate assembly.
pub fn generate_csharp_practical_ordinary_structural_boundary(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    generate_structural(vir, true, false)
}

pub fn generate_csharp_practical_ordinary_structural_public(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    generate_structural(vir, false, true)
}
fn generate_structural(
    vir: &ValidatedPracticalVir,
    with_boundary: bool,
    with_public: bool,
) -> R<OrdinaryStructuralFoundationProgram> {
    let boundary_program = if with_boundary {
        Some(generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?)
    } else {
        None
    };
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let carriers = layouts.carriers().to_vec();
    let references = carriers
        .iter()
        .map(|c| (c.type_id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let (builder, source_storage, relation_cache, source_clauses) = if with_public {
        let (b, storage, relation_cache, clauses, _) =
            super::super::super::source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
        (b, storage, relation_cache, Some(clauses))
    } else {
        (
            Builder::new()?,
            StorageCache::default(),
            ContractRelationCache::default(),
            None,
        )
    };
    let mut d = Domains {
        public_clauses: None,
        r: Relations {
            vir,
            shared_folds: true,
            observations: false,
            carriers: carriers
                .iter()
                .map(|c| (c.type_id.clone(), c.clone()))
                .collect(),
            b: builder,
            nodes: BTreeMap::new(),
            active: BTreeSet::new(),
            raw: BTreeMap::new(),
            special: BTreeMap::new(),
            storage: source_storage,
        },
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    relation_cache.seed(&mut d.r)?;
    if !d.r.b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
        d.r.b.helpers(5)?;
    }
    ordered_fold::auxiliary(&mut d.r.b)?;
    let mut storage = vec![];
    let mut relations = vec![];
    let mut domains = vec![];
    for carrier in &carriers {
        if let Some(value) = d.r.storage.get(&mut d.r.b, carrier, &references)? {
            storage.push(value);
        }
        if d.r.internal(&carrier.type_id) {
            continue;
        }
        let relation = d.r.ty(&carrier.type_id)?;
        relations.push(OrdinaryRelationDefinition {
            carrier: carrier.clone(),
            equality_definition: relation.equal,
            compare_definition: relation.compare,
        });
        let count = d.ty(&carrier.type_id)?;
        let source = d.r.b.var(0)?;
        let cells = call(&mut d.r.b, &count.definition, vec![source])?;
        let valid = valid_count(&mut d.r.b, cells)?;
        let valid_definition = format!("{}.Valid", name(&carrier.type_id));
        define(&mut d.r.b, &valid_definition, &[carrier.depth], 0, valid)?;
        domains.push(OrdinaryDomainDefinition {
            carrier: carrier.clone(),
            count_definition: count.definition,
            valid_definition,
        });
    }
    let defaults = defaults::emit_defaults(vir, &carriers, &mut d.r.b)?;
    let finite_operations =
        non_templates::emit_finite(vir, &mut d.r.b, &references, &mut d.r.storage)?;
    let mut sequences = vec![];
    let mut constructions = vec![];
    let mut entries = vec![];
    let mut outcomes = vec![];
    let mut collections = vec![];
    let mut deferred_instances = vec![];
    for entry in vir.data_closed().entries() {
        let id = text(entry, "instance_id")?;
        let template = text(entry, "template_id")?;
        match template {
            "mpk.csharp.semantic.bounded_sequence.v1" => {
                sequences.push(sequence_ops::emit_sequence(&mut d.r, entry)?)
            }
            "mpk.csharp.semantic.ordered_entry.v1" => {
                entries.push(entry_ops::emit_entry(&mut d.r, entry)?)
            }
            "mpk.csharp.semantic.ordered_map.v1" | "mpk.csharp.semantic.ordered_set.v1" => {
                collections.push(collection_ops::emit_collection(&mut d, entry)?)
            }
            "mpk.csharp.semantic.option.v1"
            | "mpk.csharp.semantic.lookup.v1"
            | "mpk.csharp.semantic.result.v1"
            | "mpk.csharp.semantic.validation.v1"
            | "mpk.csharp.semantic.boundary_field.v1" => {
                outcomes.push(outcome_ops::emit_outcome(&mut d.r, entry)?)
            }
            "mpk.csharp.semantic.sequence_construction.v1" => {
                let metadata = &vir.data_closed().metadata[id];
                if metadata.argument_ids.len() != 1 || metadata.dependency_ids.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let get = |id: &str| {
                    references
                        .get(id)
                        .copied()
                        .ok_or(OrdinaryCarrierError::Linkage)
                };
                constructions.push(construction_ops::emit_construction(
                    &mut d.r.b,
                    get(id)?,
                    get(&metadata.argument_ids[0])?,
                    get(&metadata.dependency_ids[0])?,
                    &references,
                    entry,
                )?);
            }
            "mpk.csharp.semantic.money.v1" => {
                // Emitted together below so decimal operations share one cache.
            }
            "mpk.csharp.semantic.transition.v1" => {
                // Explicitly retained, not silently treated as generated/proved.
                deferred_instances.push(OrdinaryDeferredFoundationInstance {
                    instance_id: id.into(),
                    template_id: template.into(),
                    operation_ids: array(entry, "operation_definitions")?
                        .iter()
                        .map(|op| text(op, "id").map(str::to_owned))
                        .collect::<R<_>>()?,
                    internal_unit: 6,
                });
            }
            _ => return Err(OrdinaryCarrierError::Linkage),
        }
    }
    let money = money_ops::emit_money(&mut d.r)?;
    // Emit observation after every semantic operation so its separate cache and
    // namespace cannot replace IEEE value equality in an existing operation.
    let mut source_observations = vec![];
    for carrier in &carriers {
        if d.r.internal(&carrier.type_id) {
            continue;
        }
        let observation = d.r.source_observation(&carrier.type_id)?;
        source_observations.push(OrdinaryObservationDefinition {
            carrier: carrier.clone(),
            equality_definition: observation.equal,
        });
    }
    let boundary_json = if boundary_program
        .as_ref()
        .is_some_and(|p| !p.contracts().is_empty())
    {
        Some(emit_boundary_json_for_carriers(&mut d.r.b, &carriers)?)
    } else {
        None
    };
    // Existing collection operations retain representation predicates. Add the
    // public profile only after those definitions have been emitted; separate
    // count caches keep public source clauses from changing their contracts.
    let (public_domains, public_defaults) = if let Some(clauses) = &source_clauses {
        d.public_clauses = Some(public_domains::clauses_by_type(clauses)?);
        d.counts.clear();
        let domains = public_domains::emit_membership(&mut d, &carriers)?;
        let public_defaults =
            public_domains::emit_default_predicates(&mut d, defaults.clone(), &domains)?;
        (Some(domains), Some(public_defaults))
    } else {
        (None, None)
    };
    let static_transformers = d.r.b.static_transformers;
    let certificate = d.r.b.finish()?;
    let p = OrdinaryStructuralFoundationProgram {
        schema: if with_public {
            "mpk.csharp.ordinary_structural_public.v1"
        } else if with_boundary {
            "mpk.csharp.ordinary_structural_boundary.v1"
        } else {
            "mpk.csharp.ordinary_structural_foundations.v1"
        }
        .into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        carriers,
        storage,
        relations,
        source_observations,
        domains,
        defaults,
        finite_operations,
        sequences,
        constructions,
        entries,
        outcomes,
        collections,
        money,
        deferred_instances,
        boundary_program_sha256: boundary_program.as_ref().map(|p| p.hash()),
        boundary_json,
        source_clauses,
        public_domains,
        public_defaults,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_structural_foundations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_structural_foundations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub fn import_csharp_practical_ordinary_structural_boundary(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_structural_boundary(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub fn import_csharp_practical_ordinary_structural_public(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralFoundationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_structural_public(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
