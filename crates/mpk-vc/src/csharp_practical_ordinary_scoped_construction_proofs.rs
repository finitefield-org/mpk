//! Complete original operation equations at exact source ownership points.
//! Generic ownership and native execution/application establishment stay separate.
use super::super::super::super::{ownership_flow, ownership_proofs, source_clauses};
use super::*;
use crate::csharp_practical_vir_model::data_vc::{generate_data_vcs, DataOperationVc};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScopedConstructionOwnership {
    pub function_id: String,
    pub node_id: String,
    pub receiver_id: String,
    pub state_id: String,
    pub flow_definition: String,
    pub flow_theorem: String,
    pub witness_definition: String,
    pub receiver_theorem: String,
    pub failure_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScopedConstructionOperationCandidate {
    pub source: DataOperationVc,
    pub ownership: OrdinaryScopedConstructionOwnership,
    pub program: OrdinaryConcreteOperationProofProgram,
    #[serde(skip)]
    definition_program: OrdinaryConcreteOperationProgram,
}
impl OrdinaryScopedConstructionOperationCandidate {
    pub fn definition_program(&self) -> &OrdinaryConcreteOperationProgram {
        &self.definition_program
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScopedConstructionOperationProofProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    data_vc_sha256: String,
    candidates: Vec<OrdinaryScopedConstructionOperationCandidate>,
    pending_source_operation_ids: Vec<String>,
    /// Source-specific candidates do not resolve generic uninvoked operations.
    generic_pending_operations: Vec<OrdinaryConcreteOperationPending>,
    pending_proof_ids: Vec<String>,
    proof_check_pending: bool,
    application_scope_pending: bool,
}
impl OrdinaryScopedConstructionOperationProofProgram {
    pub fn candidates(&self) -> &[OrdinaryScopedConstructionOperationCandidate] {
        &self.candidates
    }
    pub fn pending_source_operation_ids(&self) -> &[String] {
        &self.pending_source_operation_ids
    }
    pub fn generic_pending_operations(&self) -> &[OrdinaryConcreteOperationPending] {
        &self.generic_pending_operations
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("scoped construction operation proof candidates")
    }
}

pub fn generate_csharp_practical_ordinary_scoped_construction_operation_proofs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryScopedConstructionOperationProofProgram> {
    let base = generate_csharp_practical_ordinary_concrete_operations_with_allocations(vir)?;
    let data = generate_data_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let depths = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.depth))
        .collect();
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut candidates = vec![];
    let mut pending_source_operation_ids = vec![];
    let mut seen = BTreeSet::new();
    for source in data.operations() {
        let original = data
            .definitions()
            .iter()
            .find(|d| d.id == source.definition_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let Some(pending) = base.pending_operations.iter().find(|p| {
            p.component.operation_id == original.signature.id
                && p.reasons
                    == [
                        OrdinaryConcreteOperationPendingReason::InternalConstructionState,
                        OrdinaryConcreteOperationPendingReason::SourceOwnership,
                    ]
        }) else {
            continue;
        };
        if !seen.insert(source.id.clone())
            || original.signature.argument_type_ids != pending.component.argument_type_ids
            || original.signature.normal_result_type_id != pending.component.result_type_id
            || source.subjects.len() < pending.component.argument_type_ids.len()
            || source
                .subjects
                .iter()
                .zip(&pending.component.argument_type_ids)
                .any(|(s, t)| &s.type_id != t)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let (b, functions, _) =
            ownership_flow::emit_definitions(vir, Builder::resume(base.certificate_bytes())?)?;
        let (b, ownership_proofs) = ownership_proofs::emit_proofs(b, &functions)?;
        let Some(function) = functions
            .iter()
            .find(|f| f.source.function_id == source.function_id)
        else {
            pending_source_operation_ids.push(source.id.clone());
            continue;
        };
        let point = function
            .points
            .iter()
            .find(|p| p.node_id == source.node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if source.subjects.first().map(|s| s.id.as_str()) != Some(point.receiver_id.as_str()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let proof = ownership_proofs
            .iter()
            .find(|p| p.function_id == source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let receiver_theorem = proof
            .point_theorems
            .get(&source.node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let (b, domains) = super::super::super::domains::emit_construction_storage_domains(
            vir,
            &layouts,
            b,
            &base.public_domains,
        )?;
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
        let mut symbols = base
            .public_domains
            .iter()
            .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
            .collect::<BTreeMap<_, _>>();
        conditions::boolean_symbols(&mut symbols);
        for domain in &domains {
            if !domain.private_storage_only
                || !domain.ownership_pending
                || symbols
                    .insert(domain.symbol.clone(), domain.valid_definition.clone())
                    .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        for observation in &base.source_observations {
            symbols.insert(
                format!("Mpk.CSharp.Binding.Equal.{}", observation.carrier.type_id),
                observation.equality_definition.clone(),
            );
        }
        for id in pending
            .component
            .argument_type_ids
            .iter()
            .chain(std::iter::once(&pending.component.result_type_id))
        {
            if r.internal(id) {
                let equal = source_clauses::physical_equal(&mut r.b, r.carriers[id].depth)?;
                symbols.insert(format!("Mpk.CSharp.Binding.Equal.{id}"), equal);
            }
        }
        let boolean_equal = base
            .source_observations
            .iter()
            .find(|d| d.carrier.type_id == BOOL_TYPE_ID)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .equality_definition
            .clone();
        let hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(vir.hash(), source, &point.state_id))
                    .map_err(|_| OrdinaryCarrierError::Linkage)?
            )
        );
        let failure_definition = format!("{PREFIX}.ScopedConstructionOwnership.H{hash}");
        let boolean = r.b.boolean;
        let flow = r.b.constant(&function.flow_definition)?;
        let yes = bit(&mut r.b, true)?;
        let flow_type = call(&mut r.b, "Std.Eq", vec![boolean, flow, yes])?;
        let flow_proof = r.b.constant(&proof.flow_theorem)?;
        let witness = r.b.constant(&point.witness_definition)?;
        let no = bit(&mut r.b, false)?;
        let receiver_type = call(&mut r.b, "Std.Eq", vec![boolean, witness, no])?;
        let receiver_proof = r.b.constant(receiver_theorem)?;
        // Both checked source dependencies remain in the failure body. No
        // carrier flag or caller-supplied capability interprets ownership.
        let body = r.b.term(TermNode::Let {
            ty: receiver_type,
            value: receiver_proof,
            body: witness,
        })?;
        let body = r.b.term(TermNode::Let {
            ty: flow_type,
            value: flow_proof,
            body,
        })?;
        let inputs = vec![r.carriers[&pending.component.argument_type_ids[0]].depth];
        define(&mut r.b, &failure_definition, &inputs, 0, body)?;
        let mut component = pending.component.clone();
        if component.currency_predicate_argument_type_id.is_some()
            || component.failures.first().is_none_or(|f| {
                f.label != "ownership" || f.definition.is_some() || f.argument_indices != [0]
            })
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        component.failures[0].definition = Some(failure_definition.clone());
        let instance = base
            .instances
            .iter()
            .find(|i| i.instance_id == pending.instance_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let definition = emit_operation(
            &mut r,
            instance,
            &pending.recipe,
            component,
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
        let condition =
            conditions::operation_obligation(&mut r, sequent, &symbols, "concrete_operation")?;
        let mut p = base.clone();
        let id = definition.component.operation_id.clone();
        p.definitions.push(definition);
        p.conditions.push(condition);
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
            .retain(|o| o.component.operation_id != id);
        p.pending_condition_ids
            .retain(|id| !p.conditions.iter().any(|c| &c.sequent.id == id));
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
        let program = proofs::emit(&p, &depths)?;
        candidates.push(OrdinaryScopedConstructionOperationCandidate {
            source: source.clone(),
            ownership: OrdinaryScopedConstructionOwnership {
                function_id: source.function_id.clone(),
                node_id: source.node_id.clone(),
                receiver_id: point.receiver_id.clone(),
                state_id: point.state_id.clone(),
                flow_definition: function.flow_definition.clone(),
                flow_theorem: proof.flow_theorem.clone(),
                witness_definition: point.witness_definition.clone(),
                receiver_theorem: receiver_theorem.clone(),
                failure_definition,
            },
            program,
            definition_program: p,
        });
    }
    let result = OrdinaryScopedConstructionOperationProofProgram {
        schema: "mpk.csharp.ordinary_scoped_construction_operation_proofs.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: base.foundation_sha256.clone(),
        binding_vc_sha256: base.binding_vc_sha256.clone(),
        data_vc_sha256: data.hash(),
        candidates,
        pending_source_operation_ids,
        generic_pending_operations: base.pending_operations.clone(),
        pending_proof_ids: base.pending_proof_ids.clone(),
        proof_check_pending: true,
        application_scope_pending: true,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}

pub fn import_csharp_practical_ordinary_scoped_construction_operation_proofs(
    metadata: &[u8],
    certificates: &[Vec<u8>],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryScopedConstructionOperationProofProgram> {
    if metadata.len() > 16 * 1024 * 1024 || certificates.iter().any(|c| c.len() > 16 * 1024 * 1024)
    {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_scoped_construction_operation_proofs(vir)?;
    if metadata != expected.canonical_bytes()
        || certificates.len() != expected.candidates.len()
        || certificates
            .iter()
            .zip(&expected.candidates)
            .any(|(c, p)| c != p.program.certificate_bytes())
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
