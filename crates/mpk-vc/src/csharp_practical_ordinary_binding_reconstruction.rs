//! Unary candidates with explicit, typed constant completions and the original
//! W06 obligations. Defining a candidate never proves its admissibility.
use super::super::super::{binding_projections, literals, source_clauses};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingReconstructionCandidate {
    pub rebuild: OrdinaryBindingRebuildDefinition,
    /// Absent only for the existing identity conversion.
    pub completion_definition: Option<String>,
    /// Exactly one semantic argument; there is no hidden source operand.
    pub reconstruct_definition: String,
}

pub type OrdinaryBindingReconstructionObligation = OrdinaryBindingCondition;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingReconstructionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    construction_sha256: String,
    completions: Vec<OrdinaryLiteralDefinition>,
    candidates: Vec<OrdinaryBindingReconstructionCandidate>,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    predicates: Vec<OrdinaryBindingPredicate>,
    obligations: Vec<OrdinaryBindingReconstructionObligation>,
    unresolved_vc_symbols: Vec<String>,
    /// Includes every original binding VC, even those with a condition definition.
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingReconstructionProgram {
    pub fn candidates(&self) -> &[OrdinaryBindingReconstructionCandidate] {
        &self.candidates
    }
    pub fn obligations(&self) -> &[OrdinaryBindingReconstructionObligation] {
        &self.obligations
    }
    pub fn public_domains(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.public_domains
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
        serde_json::to_vec(self).expect("typed binding reconstruction candidates")
    }
}

pub(super) fn term(
    b: &mut Builder,
    t: &ContractTerm,
    subject: &TypedValueRef,
    symbols: &BTreeMap<String, String>,
    fragments: &[(ContractTerm, String)],
) -> R<u32> {
    // Only sealed, exact terms compiled under this same single receiver may be
    // supplied as fragments. Never descend into a binder or match by hash alone.
    if let Some((_, definition)) = fragments.iter().find(|(original, _)| original == t) {
        let receiver = b.var(0)?;
        return call(b, definition, vec![receiver]);
    }
    match t {
        ContractTerm::Var { index, type_id } if *index == 0 && *type_id == subject.type_id => {
            b.var(0)
        }
        ContractTerm::Const { name, .. } => {
            b.constant(symbols.get(name).ok_or(OrdinaryCarrierError::Linkage)?)
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            let f = term(b, function, subject, symbols, fragments)?;
            let a = term(b, argument, subject, symbols, fragments)?;
            b.app(f, vec![a])
        }
        _ => Err(OrdinaryCarrierError::Linkage),
    }
}

pub(super) fn obligation(
    r: &mut Relations<'_>,
    sequent: &BindingSequent,
    symbols: &BTreeMap<String, String>,
    scope: &str,
) -> R<OrdinaryBindingReconstructionObligation> {
    obligation_with_fragments(r, sequent, symbols, scope, &[])
}

pub(super) fn obligation_with_fragments(
    r: &mut Relations<'_>,
    sequent: &BindingSequent,
    symbols: &BTreeMap<String, String>,
    scope: &str,
    fragments: &[(ContractTerm, String)],
) -> R<OrdinaryBindingReconstructionObligation> {
    let [subject] = sequent.subjects.as_slice() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let depth = r
        .carriers
        .get(&subject.type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    let mut groups = vec![];
    let mut conjunctions = vec![];
    for (kind, terms) in [
        ("assumption", &sequent.assumptions),
        ("goal", &sequent.goals),
    ] {
        let mut names = vec![];
        let mut combined = bit(&mut r.b, true)?;
        for (i, t) in terms.iter().enumerate() {
            if t.type_id() != "mpk.csharp.value.bool.v1" {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let definition = name(&format!("{scope}.{}.{kind}.{i}", sequent.id));
            let body = term(&mut r.b, t, subject, symbols, fragments)?;
            define(&mut r.b, &definition, &[depth], 0, body)?;
            combined = and(&mut r.b, combined, body)?;
            names.push(definition);
        }
        groups.push(names);
        conjunctions.push(combined);
    }
    let condition_definition = name(&format!("{scope}.{}.condition", sequent.id));
    let no_assumptions = call(&mut r.b, "Std.Bool.not", vec![conjunctions[0]])?;
    let body = call(
        &mut r.b,
        "Std.Bool.or",
        vec![no_assumptions, conjunctions[1]],
    )?;
    define(&mut r.b, &condition_definition, &[depth], 0, body)?;
    Ok(OrdinaryBindingReconstructionObligation {
        sequent: sequent.clone(),
        assumption_definitions: groups.remove(0),
        goal_definitions: groups.remove(0),
        condition_definition,
    })
}

/// Each nonidentity source type needs exactly one explicit completion value.
/// Constant completions are one candidate construction strategy, not a profile
/// restriction or a source-invariant assumption. Other witnesses remain future
/// proof-producer work. This API supplies no application-acceptance result.
pub fn generate_csharp_practical_ordinary_binding_reconstruction(
    vir: &ValidatedPracticalVir,
    completion_values: &[MonomorphicValue],
) -> R<OrdinaryBindingReconstructionProgram> {
    let required = vir
        .binding_projections()
        .iter()
        .filter(|p| p.binding_id != "binding.identity")
        .map(|p| p.source_type_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut completion_names = BTreeMap::new();
    let mut values = BTreeMap::new();
    for value in completion_values {
        if !required.contains(value.type_id()) || completion_names.contains_key(value.type_id()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        // Completions are explicit producer input, unlike values already sealed
        // in the VIR. Enforce value limits before hashing or cloning them.
        let (bundle, roots, _) = vir.construction_context();
        validate_monomorphic_value(bundle, roots, vir.data_closed(), value)
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        let definition = format!(
            "{PREFIX}.BindingReconstruction.Completion.H{:x}",
            Sha256::digest(serde_json::to_vec(value).map_err(|_| OrdinaryCarrierError::Shape)?)
        );
        if completion_names
            .insert(value.type_id().to_owned(), definition.clone())
            .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        values.insert(definition, value.clone());
    }
    if completion_names.len() != required.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, storage, cache, source_clauses, construction_sha256) =
        source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
    let (b, projections) = binding_projections::emit_binding_projections(vir, &vc, &layouts, b)?;
    let (b, rebuilds, _) =
        binding_projections::emit_rebuilds(vir, &vc, &layouts, b, projections.clone())?;
    let (b, completions) =
        literals::emit_named_values_scoped(vir, &layouts, b, values, "BindingCompletionPart")?;
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
    let mut symbols = BTreeMap::new();
    let mut candidates = vec![];
    for rebuild in rebuilds {
        let p = &rebuild.projection;
        let completion_definition = completion_names.get(&p.source_carrier.type_id).cloned();
        let reconstruct_definition = if let Some(identity) = &p.reconstruct_definition {
            identity.clone()
        } else {
            let completion = completion_definition
                .as_ref()
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let definition = name(&format!("reconstruct.{}.{completion}", p.projection.id));
            let y = r.b.var(0)?;
            let seed = r.b.constant(completion)?;
            let body = call(&mut r.b, &rebuild.rebuild_definition, vec![y, seed])?;
            define(
                &mut r.b,
                &definition,
                &[p.semantic_carrier.depth],
                p.source_carrier.depth,
                body,
            )?;
            definition
        };
        symbols.insert(
            p.projection.project.id.clone(),
            p.project_definition.clone(),
        );
        symbols.insert(
            p.projection.reconstruct.id.clone(),
            reconstruct_definition.clone(),
        );
        candidates.push(OrdinaryBindingReconstructionCandidate {
            rebuild,
            completion_definition,
            reconstruct_definition,
        });
    }
    let assembly = assemble_relations(r, projections, &vc, &layouts)?;
    let (mut r, public_domains) = super::super::domains::emit_binding_domains(
        assembly.r,
        layouts.carriers(),
        &source_clauses,
    )?;
    let predicates = assembly.predicates.into_values().collect::<Vec<_>>();
    for predicate in &predicates {
        symbols.insert(predicate.symbol.clone(), predicate.definition.clone());
    }
    for domain in &public_domains {
        symbols.insert(domain.symbol.clone(), domain.valid_definition.clone());
    }
    let mut obligations = vec![];
    for sequent in vc.sequents() {
        if matches!(
            sequent.kind.as_str(),
            "projection_total"
                | "reconstruction"
                | "source_round_trip"
                | "semantic_round_trip"
                | "member_reconstruction"
                | "identity_projection"
        ) {
            obligations.push(obligation(&mut r, sequent, &symbols, "reconstruction")?);
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
    let program = OrdinaryBindingReconstructionProgram {
        schema: "mpk.csharp.ordinary_binding_reconstruction.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        construction_sha256,
        completions,
        candidates,
        source_clauses,
        public_domains,
        predicates,
        obligations,
        unresolved_vc_symbols,
        pending_proof_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}

pub fn import_csharp_practical_ordinary_binding_reconstruction(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
    completion_values: &[MonomorphicValue],
) -> R<OrdinaryBindingReconstructionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected =
        generate_csharp_practical_ordinary_binding_reconstruction(vir, completion_values)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
