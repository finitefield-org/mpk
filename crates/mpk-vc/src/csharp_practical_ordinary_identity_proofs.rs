//! Original identity projection sequents on the unchanged foundation context.
//! Binding.Equal is proof-level equality, including every bit of float values.
use super::super::super::super::super::binding_projections;
use super::super::super::super::super::ownership_proofs::{logic, publish_theorem};
use super::*;

#[path = "csharp_practical_ordinary_default_proofs.rs"]
mod default_proofs;
pub use default_proofs::{
    generate_csharp_practical_ordinary_default_proofs,
    import_csharp_practical_ordinary_default_proofs, OrdinaryActualDefaultProof,
    OrdinaryDefaultGoalProof, OrdinaryDefaultProofProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIdentityProjectionProof {
    pub sequent: BindingSequent,
    pub projection: OrdinaryBindingProjectionDefinition,
    pub proposition_definition: String,
    pub theorem: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIdentityProofProgram {
    schema: String,
    foundation: OrdinaryFoundationProofProgram,
    original_certificate_sha256: String,
    proofs: Vec<OrdinaryIdentityProjectionProof>,
    supplied_binding_sequent_ids: Vec<String>,
    remaining_binding_sequent_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    proof_check_pending: bool,
    application_scope_pending: bool,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    definition_certificate: Vec<u8>,
    #[serde(skip)]
    certificate: Vec<u8>,
}

impl OrdinaryIdentityProofProgram {
    pub fn static_transformers(&self) -> usize {
        self.static_transformers
    }
    pub fn foundation(&self) -> &OrdinaryFoundationProofProgram {
        &self.foundation
    }
    pub fn proofs(&self) -> &[OrdinaryIdentityProjectionProof] {
        &self.proofs
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
    pub fn definition_certificate_bytes(&self) -> &[u8] {
        &self.definition_certificate
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("original identity projection proof candidates")
    }
}

// Lower each original equality operand without dropping either projection or
// the receiver. Its complete carrier is the same proof-level type used by the
// original concrete-operation proofs; C# float `.equal` is a separate operation.
fn operands<'a>(goal: &'a ContractTerm, type_id: &str) -> R<(&'a ContractTerm, &'a ContractTerm)> {
    let ContractTerm::App {
        function,
        argument: right,
        type_id: result,
    } = goal
    else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let ContractTerm::App {
        function,
        argument: left,
        ..
    } = function.as_ref()
    else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    let ContractTerm::Const { name, .. } = function.as_ref() else {
        return Err(OrdinaryCarrierError::Linkage);
    };
    if name != &format!("Mpk.CSharp.Binding.Equal.{type_id}")
        || result != BOOL_TYPE_ID
        || left.type_id() != type_id
        || right.type_id() != type_id
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok((left, right))
}

fn term(
    b: &mut Builder,
    t: &ContractTerm,
    subject: &TypedValueRef,
    symbols: &BTreeMap<String, String>,
    offset: u32,
) -> R<u32> {
    match t {
        ContractTerm::Var { index, type_id } if *index == 0 && *type_id == subject.type_id => {
            b.var(offset)
        }
        ContractTerm::Const { name, .. } => {
            b.constant(symbols.get(name).ok_or(OrdinaryCarrierError::Linkage)?)
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            let f = term(b, function, subject, symbols, offset)?;
            let a = term(b, argument, subject, symbols, offset)?;
            b.app(f, vec![a])
        }
        _ => Err(OrdinaryCarrierError::Linkage),
    }
}

pub fn generate_csharp_practical_ordinary_identity_proofs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIdentityProofProgram> {
    let foundation = generate_csharp_practical_ordinary_foundation_proofs(vir)?;
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let originals = vc
        .sequents()
        .iter()
        .filter(|s| s.kind == "identity_projection")
        .collect::<Vec<_>>();
    let original_certificate_sha256 =
        mpk_cert::hash_hex(&mpk_cert::certificate_hash(foundation.certificate_bytes()));
    let mut b = Builder::resume(foundation.certificate_bytes())?;
    let mut symbols = foundation
        .definition_program()
        .public_domains()
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let projections = if originals.is_empty() {
        vec![]
    } else {
        let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
        let (next, projections) =
            binding_projections::emit_binding_projections(vir, &vc, &layouts, b)?;
        b = next;
        logic(&mut b)?;
        projections
    };
    let definition_certificate = b.clone().finish()?;
    let mut proofs = vec![];
    for sequent in originals {
        let [subject] = sequent.subjects.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let [assumption] = sequent.assumptions.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if sequent.goals.len() != 2 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let projection = projections
            .iter()
            .find(|p| p.projection.id == sequent.owner_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if projection.projection.binding_id != "binding.identity"
            || projection.projection.source_type_id != subject.type_id
            || projection.projection.semantic_type_id != subject.type_id
            || projection.source_carrier != projection.semantic_carrier
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let reconstruct = projection
            .reconstruct_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        symbols.insert(
            projection.projection.project.id.clone(),
            projection.project_definition.clone(),
        );
        symbols.insert(
            projection.projection.reconstruct.id.clone(),
            reconstruct.clone(),
        );
        let value_type = b.cube(projection.source_carrier.depth)?;
        let boolean = b.boolean;
        let yes = bit(&mut b, true)?;
        let domain = term(&mut b, assumption, subject, &symbols, 0)?;
        let premise = call(&mut b, "Std.Eq", vec![boolean, domain, yes])?;
        let mut goals = vec![];
        for goal in &sequent.goals {
            let (left, right) = operands(goal, &subject.type_id)?;
            let left = term(&mut b, left, subject, &symbols, 1)?;
            let right = term(&mut b, right, subject, &symbols, 1)?;
            let ty = call(&mut b, "Std.Eq", vec![value_type, left, right])?;
            let proof = call(&mut b, "Std.Eq.refl", vec![value_type, left])?;
            goals.push((ty, proof));
        }
        let ty = call(&mut b, "Std.Logic.And", vec![goals[0].0, goals[1].0])?;
        let proof = call(
            &mut b,
            "Std.Logic.And.intro",
            vec![goals[0].0, goals[1].0, goals[0].1, goals[1].1],
        )?;
        let ty = b.pi(premise, ty)?;
        let ty = b.pi(value_type, ty)?;
        let proof = b.lam(premise, proof)?;
        let proof = b.lam(value_type, proof)?;
        let hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&original_certificate_sha256, sequent))
                    .map_err(|_| OrdinaryCarrierError::Linkage)?
            )
        );
        let proposition_definition = format!("{PREFIX}.IdentityProjectionProof.Type.H{hash}");
        b.define(&proposition_definition, b.sort, ty)?;
        let checked_body = format!("{PREFIX}.IdentityProjectionProof.Body.H{hash}");
        publish_theorem(&mut b, &checked_body, ty, proof)?;
        let proof = b.constant(&checked_body)?;
        let theorem = format!("{PREFIX}.IdentityProjectionProof.Theorem.H{hash}");
        let ty = b.constant(&proposition_definition)?;
        publish_theorem(&mut b, &theorem, ty, proof)?;
        proofs.push(OrdinaryIdentityProjectionProof {
            sequent: sequent.clone(),
            projection: projection.clone(),
            proposition_definition,
            theorem,
        });
    }
    let static_transformers = foundation
        .definition_program
        .static_transformers
        .checked_add(b.static_transformers)
        .ok_or(OrdinaryCarrierError::Limit)?;
    if static_transformers > crate::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX as usize {
        return Err(OrdinaryCarrierError::Limit);
    }
    let certificate = b.finish()?;
    let mut supplied = foundation
        .supplied_binding_sequent_ids()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for proof in &proofs {
        if !supplied.insert(proof.sequent.id.clone()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    let pending_proof_ids = foundation.pending_proof_ids().to_vec();
    let result = OrdinaryIdentityProofProgram {
        schema: "mpk.csharp.ordinary_identity_proofs.v1".into(),
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
        foundation,
        original_certificate_sha256,
        proofs,
        pending_proof_ids,
        proof_check_pending: true,
        application_scope_pending: true,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        definition_certificate,
        certificate,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}

pub fn import_csharp_practical_ordinary_identity_proofs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIdentityProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_identity_proofs(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
