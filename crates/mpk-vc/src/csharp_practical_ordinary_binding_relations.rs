//! W06 field-complete observations and projected result agreement.
//! Reconstruction witnesses, domains, native outcomes and proofs remain required.
use super::*;
use sha2::{Digest, Sha256};
#[path = "csharp_practical_ordinary_boundary_rules.rs"]
mod boundary_rules;
pub use boundary_rules::{
    generate_csharp_practical_ordinary_boundary_rules,
    import_csharp_practical_ordinary_boundary_rules, OrdinaryBoundaryRuleProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingPredicate {
    pub symbol: String,
    pub definition: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingAgreement {
    pub projection_id: String,
    pub source_type_id: String,
    pub semantic_type_id: String,
    pub comparison_symbol: String,
    /// ObserveEqual(Project(source_result), semantic_result), matching W06 normal results.
    pub result_agreement_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingRelationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    projections: Vec<OrdinaryBindingProjectionDefinition>,
    predicates: Vec<OrdinaryBindingPredicate>,
    agreements: Vec<OrdinaryBindingAgreement>,
    unresolved_vc_symbols: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingRelationProgram {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed binding relations")
    }
    pub fn predicates(&self) -> &[OrdinaryBindingPredicate] {
        &self.predicates
    }
    pub fn agreements(&self) -> &[OrdinaryBindingAgreement] {
        &self.agreements
    }
    pub fn projections(&self) -> &[OrdinaryBindingProjectionDefinition] {
        &self.projections
    }
    pub fn unresolved_vc_symbols(&self) -> &[String] {
        &self.unresolved_vc_symbols
    }
}
fn name(value: &str) -> String {
    format!(
        "{PREFIX}.BindingRelation.H{:x}",
        Sha256::digest(value.as_bytes())
    )
}
fn reference(shape: &OrdinaryShape) -> R<&str> {
    match shape {
        OrdinaryShape::Reference { type_id } => Ok(type_id),
        OrdinaryShape::RoleBound { value, .. } => reference(value),
        _ => Err(OrdinaryCarrierError::Shape),
    }
}
fn insert(
    map: &mut BTreeMap<String, OrdinaryBindingPredicate>,
    symbol: String,
    definition: String,
    args: Vec<String>,
) -> R<()> {
    let value = OrdinaryBindingPredicate {
        symbol: symbol.clone(),
        definition,
        argument_type_ids: args,
        result_type_id: "mpk.csharp.value.bool.v1".into(),
    };
    if map.get(&symbol).is_some_and(|old| old != &value) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    map.insert(symbol, value);
    Ok(())
}
struct BindingAssembly<'a> {
    r: Relations<'a>,
    projections: Vec<OrdinaryBindingProjectionDefinition>,
    predicates: BTreeMap<String, OrdinaryBindingPredicate>,
    agreements: Vec<OrdinaryBindingAgreement>,
}
fn assemble<'a>(
    vir: &'a ValidatedPracticalVir,
    vc: &BindingVcProgram,
    layouts: &OrdinaryCarrierProgram,
) -> R<BindingAssembly<'a>> {
    let (b, projections) = super::super::binding_projections::emit_binding_projections(
        vir,
        vc,
        layouts,
        Builder::new()?,
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
    // Projection emission already installs the shared word helpers.
    ordered_fold::auxiliary(&mut r.b)?;
    let mut predicates = BTreeMap::new();
    let mut agreements = vec![];
    for projection in &projections {
        let st = &projection.source_carrier.type_id;
        let mt = &projection.semantic_carrier.type_id;
        let source = r.source_observation(st)?;
        insert(
            &mut predicates,
            format!("Mpk.CSharp.Binding.ObserveEqual.{st}"),
            source.equal,
            vec![st.clone(), st.clone()],
        )?;
        // Binding.Equal is proof-level agreement for semantic round trips,
        // not the C# foundation `.equal` operation (which is non-reflexive on NaN).
        let semantic = r.source_observation(mt)?;
        insert(
            &mut predicates,
            format!("Mpk.CSharp.Binding.Equal.{mt}"),
            semantic.equal.clone(),
            vec![mt.clone(), mt.clone()],
        )?;
        let result_observation = r.source_observation(mt)?;
        insert(
            &mut predicates,
            format!("Mpk.CSharp.Binding.ObserveEqual.{mt}"),
            result_observation.equal.clone(),
            vec![mt.clone(), mt.clone()],
        )?;
        let agreement = name(&format!("result.{}", projection.projection.id));
        let x = r.b.var(1)?;
        let y = r.b.var(0)?;
        let px = call(&mut r.b, &projection.project_definition, vec![x])?;
        let body = call(&mut r.b, &result_observation.equal, vec![px, y])?;
        define(
            &mut r.b,
            &agreement,
            &[
                projection.source_carrier.depth,
                projection.semantic_carrier.depth,
            ],
            0,
            body,
        )?;
        agreements.push(OrdinaryBindingAgreement {
            projection_id: projection.projection.id.clone(),
            source_type_id: st.clone(),
            semantic_type_id: mt.clone(),
            comparison_symbol: format!("Mpk.CSharp.Binding.ObserveEqual.{mt}"),
            result_agreement_definition: agreement,
        });
        let OrdinaryShape::Product { fields } = &projection.source_carrier.shape else {
            continue;
        };
        let refs = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect();
        let storage = r
            .storage
            .get(&mut r.b, &projection.source_carrier, &refs)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let OrdinaryStructuralOperations::Product { operations } = storage.operations else {
            return Err(OrdinaryCarrierError::Shape);
        };
        for (field, get) in fields.iter().zip(&operations.fields) {
            if field.id != get.field_id {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let child = r.source_observation(reference(&field.shape)?)?;
            let symbol = format!("Mpk.CSharp.Binding.MemberEqual.{}", field.id);
            let definition = name(&symbol);
            let x = r.b.var(1)?;
            let y = r.b.var(0)?;
            let x = call(&mut r.b, &get.definition, vec![x])?;
            let y = call(&mut r.b, &get.definition, vec![y])?;
            let body = call(&mut r.b, &child.equal, vec![x, y])?;
            define(
                &mut r.b,
                &definition,
                &[projection.source_carrier.depth; 2],
                0,
                body,
            )?;
            insert(
                &mut predicates,
                symbol,
                definition,
                vec![st.clone(), st.clone()],
            )?;
        }
    }
    // Boolean equality occurs in W06 source-invariant equivalences even when
    // no source signature otherwise requires the Bool carrier.
    let bool_id = "mpk.csharp.value.bool.v1";
    let eq = r.raw(1, false)?.equal;
    insert(
        &mut predicates,
        format!("Mpk.CSharp.Binding.Equal.{bool_id}"),
        eq,
        vec![bool_id.into(), bool_id.into()],
    )?;
    // W06 normal commutation can compare primitive/payload results as well as
    // whole projected wrappers. Resolve every demanded comparison at its exact
    // concrete result type, independently of the projection inventory.
    for symbol in vc.definition_names() {
        if predicates.contains_key(symbol) {
            continue;
        }
        let id = if let Some(id) = symbol.strip_prefix("Mpk.CSharp.Binding.ObserveEqual.") {
            id
        } else if let Some(id) = symbol.strip_prefix("Mpk.CSharp.Binding.Equal.") {
            id
        } else {
            continue;
        };
        // Linear construction states have ownership/initialization relations,
        // not public value observation. Their symbols stay explicitly pending.
        if r.internal(id) {
            continue;
        }
        let node = r.source_observation(id)?;
        insert(
            &mut predicates,
            symbol.clone(),
            node.equal,
            vec![id.into(), id.into()],
        )?;
    }
    Ok(BindingAssembly {
        r,
        projections,
        predicates,
        agreements,
    })
}
fn finish(
    vir: &ValidatedPracticalVir,
    vc: &BindingVcProgram,
    assembly: BindingAssembly<'_>,
    schema: &str,
) -> R<OrdinaryBindingRelationProgram> {
    let BindingAssembly {
        r,
        projections,
        predicates,
        agreements,
    } = assembly;
    let resolved = predicates
        .keys()
        .cloned()
        .chain(projections.iter().map(|p| p.projection.project.id.clone()))
        .chain(
            projections
                .iter()
                .filter(|p| p.reconstruct_definition.is_some())
                .map(|p| p.projection.reconstruct.id.clone()),
        )
        .collect::<BTreeSet<_>>();
    let unresolved_vc_symbols = vc
        .definition_names()
        .iter()
        .filter(|s| !resolved.contains(*s))
        .cloned()
        .collect();
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let program = OrdinaryBindingRelationProgram {
        schema: schema.into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        projections,
        predicates: predicates.into_values().collect(),
        agreements,
        unresolved_vc_symbols,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}
pub fn generate_csharp_practical_ordinary_binding_relations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let assembly = assemble(vir, &vc, &layouts)?;
    finish(
        vir,
        &vc,
        assembly,
        "mpk.csharp.ordinary_binding_relations.v1",
    )
}
pub fn import_csharp_practical_ordinary_binding_relations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_binding_relations(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[path = "csharp_practical_ordinary_binding_guards.rs"]
mod guards;
pub use guards::{
    generate_csharp_practical_ordinary_binding_guards,
    import_csharp_practical_ordinary_binding_guards,
};

#[path = "csharp_practical_ordinary_binding_orders.rs"]
mod orders;
pub use orders::{
    generate_csharp_practical_ordinary_binding_orders,
    import_csharp_practical_ordinary_binding_orders,
};
