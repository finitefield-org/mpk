//! Boundary-run constants are exact values, not proofs of adapter/native execution.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryLiteralBinding {
    /// Typed boundary-VC symbol; not necessarily a valid core global name.
    pub boundary_symbol: String,
    pub type_id: String,
    /// Actual ordinary definition that the later proposition compiler must use.
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryLiteralProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    boundary_run_sha256: String,
    definitions: Vec<OrdinaryLiteralDefinition>,
    bindings: Vec<OrdinaryBoundaryLiteralBinding>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBoundaryLiteralProgram {
    pub fn definitions(&self) -> &[OrdinaryLiteralDefinition] {
        &self.definitions
    }
    pub fn bindings(&self) -> &[OrdinaryBoundaryLiteralBinding] {
        &self.bindings
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordinary boundary literals")
    }
}

pub fn generate_csharp_practical_ordinary_boundary_literals(
    vir: &ValidatedPracticalVir,
    run: &BoundaryRunVcProgram,
) -> R<OrdinaryBoundaryLiteralProgram> {
    // A run can only be constructed through full input/output evidence validation.
    // Still reject splicing an immutable run from another emission/source context.
    if run.source_ir_sha256() != vir.hash()
        || generate_boundary_vcs(vir)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
            .hash()
            != run.boundary_program_sha256()
        || run.values().windows(2).any(|p| p[0].name >= p[1].name)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut emitter = Literals {
        b: Builder::new()?,
        parts: BTreeMap::new(),
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
    };
    let mut roots = BTreeMap::new();
    let mut bindings = vec![];
    for literal in run.values() {
        let bytes = serde_json::to_vec(&literal.value).expect("typed boundary literal");
        let expected = BoundaryValueDefinition::canonical_name(&literal.value);
        if literal.name != expected {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let name = format!("{PREFIX}.Literal.H{:x}", Sha256::digest(&bytes));
        roots
            .entry(name.clone())
            .or_insert_with(|| literal.value.clone());
        bindings.push(OrdinaryBoundaryLiteralBinding {
            boundary_symbol: literal.name.clone(),
            type_id: literal.value.type_id().into(),
            definition: name,
        });
    }
    let mut definitions = vec![];
    for (name, value) in roots {
        let (bundle, roots, _) = vir.construction_context();
        validate_monomorphic_value(bundle, roots, vir.data_closed(), &value)
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        let encoded = emitter.value(&value)?;
        let ty = emitter.b.cube(encoded.depth)?;
        emitter.b.define(&name, ty, encoded.term)?;
        definitions.push(OrdinaryLiteralDefinition {
            name,
            carrier: emitter.carriers[value.type_id()].clone(),
            value,
        });
    }
    let certificate = emitter.b.finish()?;
    let result = OrdinaryBoundaryLiteralProgram {
        schema: "mpk.csharp.ordinary_boundary_literals.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: run.boundary_program_sha256().into(),
        boundary_run_sha256: run.hash(),
        definitions,
        bindings,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}
pub fn import_csharp_practical_ordinary_boundary_literals(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
    run: &BoundaryRunVcProgram,
) -> R<OrdinaryBoundaryLiteralProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let result = generate_csharp_practical_ordinary_boundary_literals(vir, run)?;
    if input != result.canonical_bytes() || certificate != result.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(result)
}
