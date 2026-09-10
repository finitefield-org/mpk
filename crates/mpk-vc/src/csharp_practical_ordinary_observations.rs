//! Complete admitted value observation for source binding round trips.
//! This emits definitions, not proofs of any source reconstruction invariant.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryObservationDefinition {
    pub carrier: OrdinaryCarrier,
    pub equality_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryObservationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryObservationDefinition>,
    erased_construction_type_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryObservationProgram {
    pub fn definitions(&self) -> &[OrdinaryObservationDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed source observation program")
    }
}

pub fn generate_csharp_practical_ordinary_observations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryObservationProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut relations = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b: Builder::new()?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    relations.b.helpers(5)?;
    let mut definitions = vec![];
    let mut erased_construction_type_ids = vec![];
    for carrier in layouts.carriers() {
        if relations.internal(&carrier.type_id) {
            erased_construction_type_ids.push(carrier.type_id.clone());
            continue;
        }
        let node = relations.source_observation(&carrier.type_id)?;
        definitions.push(OrdinaryObservationDefinition {
            carrier: carrier.clone(),
            equality_definition: node.equal,
        });
    }
    let static_transformers = relations.b.static_transformers;
    let certificate = relations.b.finish()?;
    let p = OrdinaryObservationProgram {
        schema: "mpk.csharp.ordinary_source_observations.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        erased_construction_type_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_observations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryObservationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_observations(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
