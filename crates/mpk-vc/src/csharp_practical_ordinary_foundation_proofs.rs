//! Integrate original concrete type and operation proofs in one ordinary context.
//! This is a partial binding proof assembly, not a checked application result.
use super::*;

#[path = "csharp_practical_ordinary_identity_proofs.rs"]
mod identity_proofs;
pub use identity_proofs::{
    generate_csharp_practical_ordinary_identity_proofs,
    import_csharp_practical_ordinary_identity_proofs, OrdinaryIdentityProjectionProof,
    OrdinaryIdentityProofProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryFoundationProofProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    original_operation_proofs_sha256: String,
    original_operation_certificate_sha256: String,
    types: OrdinaryConcreteTypeProofProgram,
    operations: OrdinaryConcreteOperationProofProgram,
    supplied_binding_sequent_ids: Vec<String>,
    remaining_binding_sequent_ids: Vec<String>,
    /// Supplied component proofs remain candidates until complete application
    /// assembly checks all original scopes and dependencies.
    pending_proof_ids: Vec<String>,
    proof_check_pending: bool,
    application_scope_pending: bool,
    #[serde(skip)]
    definition_program: OrdinaryConcreteTypeProgram,
}

impl OrdinaryFoundationProofProgram {
    pub fn definition_program(&self) -> &OrdinaryConcreteTypeProgram {
        &self.definition_program
    }
    pub fn type_proofs(&self) -> &[OrdinaryConcreteTypeProof] {
        self.types.proofs()
    }
    pub fn operation_proofs(&self) -> &[OrdinaryConcreteOperationProof] {
        self.operations.proofs()
    }
    pub fn construction_storage_domains(&self) -> &[OrdinaryConstructionStorageDomainDefinition] {
        self.types.construction_storage_domains()
    }
    pub fn pending_operations(&self) -> &[OrdinaryConcreteOperationPending] {
        self.operations.pending_operations()
    }
    pub fn supplied_binding_sequent_ids(&self) -> &[String] {
        &self.supplied_binding_sequent_ids
    }
    pub fn remaining_binding_sequent_ids(&self) -> &[String] {
        &self.remaining_binding_sequent_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        self.types.certificate_bytes()
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("integrated ordinary foundation proof candidates")
    }
}

pub fn generate_csharp_practical_ordinary_foundation_proofs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFoundationProofProgram> {
    let mut types =
        generate_csharp_practical_ordinary_concrete_types_with_construction_storage(vir)?;
    let definitions = generate_csharp_practical_ordinary_concrete_operations_with_allocations(vir)?;
    let operations =
        generate_csharp_practical_ordinary_concrete_operation_proofs_with_allocations(vir)?;
    if types.public_domains() != definitions.public_domains()
        || types.source_clauses() != definitions.source_clauses()
        || types.pending_proof_ids() != operations.pending_proof_ids()
        || !types.pending_type_instances().is_empty()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (mut b, domains) = super::super::super::domains::emit_construction_storage_domains(
        vir,
        &layouts,
        Builder::resume(operations.certificate_bytes())?,
        types.public_domains(),
    )?;
    if domains != types.construction_storage_domains {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut symbols = types
        .public_domains()
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    conditions::boolean_symbols(&mut symbols);
    // A source need not contain a Bool carrier. The original W06 type goals
    // still compare Boolean predicates, so lower that connective directly in
    // the registered Bool foundation rather than requiring a source observer.
    let left = b.var(1)?;
    let right = b.var(0)?;
    let inverse = call(&mut b, "Std.Bool.not", vec![right])?;
    let equal = mux(&mut b, left, right, inverse)?;
    let boolean_equal = format!("{PREFIX}.FoundationProofs.BooleanEqual");
    define(&mut b, &boolean_equal, &[0, 0], 0, equal)?;
    symbols.insert(
        format!("Mpk.CSharp.Binding.Equal.{BOOL_TYPE_ID}"),
        boolean_equal,
    );
    for d in &domains {
        if !d.private_storage_only
            || !d.ownership_pending
            || symbols
                .insert(d.symbol.clone(), d.valid_definition.clone())
                .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    // Reconstruct each complete original type predicate on the operation
    // context's unchanged domains. Do not merge conflicting name tables or
    // substitute a recipe hash for the domain's ordinary body.
    for d in types.definitions() {
        b.constant(&d.public_domain)?;
        let value = b.var(0)?;
        let body = call(&mut b, &d.public_domain, vec![value])?;
        define(&mut b, &d.definition, &[d.carrier.depth], 0, body)?;
        if symbols
            .insert(d.symbol.clone(), d.definition.clone())
            .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
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
    types.conditions = types
        .conditions()
        .iter()
        .map(|c| reconstruction::obligation(&mut r, &c.sequent, &symbols, "concrete_type"))
        .collect::<R<Vec<_>>>()?;
    types.static_transformers = definitions
        .static_transformers()
        .checked_add(r.b.static_transformers)
        .ok_or(OrdinaryCarrierError::Limit)?;
    if types.static_transformers
        > crate::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX as usize
    {
        return Err(OrdinaryCarrierError::Limit);
    }
    types.certificate = r.b.finish()?;
    types.certificate_sha256 = mpk_cert::hash_hex(&mpk_cert::certificate_hash(&types.certificate));
    let definition_program = types;
    let types = emit_type_proofs(&definition_program)?;

    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut supplied = BTreeSet::new();
    for sequent in types
        .proofs()
        .iter()
        .map(|p| &p.sequent)
        .chain(operations.proofs().iter().map(|p| &p.sequent))
    {
        if !supplied.insert(sequent.id.clone())
            || vc.sequents().iter().find(|s| s.id == sequent.id) != Some(sequent)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    let pending_proof_ids = vc
        .sequents()
        .iter()
        .map(|s| s.id.clone())
        .collect::<Vec<_>>();
    if pending_proof_ids != operations.pending_proof_ids() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let result = OrdinaryFoundationProofProgram {
        schema: "mpk.csharp.ordinary_foundation_proofs.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        original_operation_proofs_sha256: format!(
            "{:x}",
            Sha256::digest(operations.canonical_bytes())
        ),
        original_operation_certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(
            operations.certificate_bytes(),
        )),
        types,
        operations,
        supplied_binding_sequent_ids: pending_proof_ids
            .iter()
            .filter(|id| supplied.contains(*id))
            .cloned()
            .collect(),
        remaining_binding_sequent_ids: pending_proof_ids
            .iter()
            .filter(|id| !supplied.contains(*id))
            .cloned()
            .collect(),
        pending_proof_ids,
        proof_check_pending: true,
        application_scope_pending: true,
        definition_program,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}

pub fn import_csharp_practical_ordinary_foundation_proofs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFoundationProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_foundation_proofs(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
