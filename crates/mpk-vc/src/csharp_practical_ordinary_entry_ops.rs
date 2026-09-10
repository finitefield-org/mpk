//! Every expanded ordered-entry operation, including standalone/uninvoked
//! entries. Input/public domains and source entry bindings remain obligations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryEntryDefinition {
    pub carrier: OrdinaryCarrier,
    pub key_type_id: String,
    pub value_type_id: String,
    pub make_definition: String,
    pub key_definition: String,
    pub value_definition: String,
    pub equality_definition: String,
    pub compare_definition: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryEntryProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryEntryDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryEntryProgram {
    pub fn definitions(&self) -> &[OrdinaryEntryDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed entry program")
    }
}
pub fn generate_csharp_practical_ordinary_entries(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryEntryProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut r = Relations {
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
    r.b.helpers(5)?;
    ordered_fold::auxiliary(&mut r.b)?;
    let mut definitions = vec![];
    for entry in vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.ordered_entry.v1")
    {
        definitions.push(emit_entry(&mut r, entry)?);
    }
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryEntryProgram {
        schema: "mpk.csharp.ordinary_entries.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_entries(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryEntryProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_entries(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

pub(super) fn emit_entry(r: &mut Relations<'_>, entry: &Value) -> R<OrdinaryEntryDefinition> {
    let id = text(entry, "instance_id")?;
    let vir = r.vir;
    let metadata = &vir.data_closed().metadata[id];
    if metadata.argument_ids.len() != 2 || !metadata.dependency_ids.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let key = &metadata.argument_ids[0];
    let value = &metadata.argument_ids[1];
    let carrier = r
        .carriers
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    if carrier.shape
        != product(vec![
            field("key", reference(key)),
            field("value", reference(value)),
        ])
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let relation = r.ty(id)?;
    let mut expected = vec![
        json!({"id":format!("{id}.make"),"argument_type_ids":[key,value],"normal_result_type_id":id,"equation":"product(k,v)","error_precedence":[]}),
        json!({"id":format!("{id}.key"),"argument_type_ids":[id],"normal_result_type_id":key,"equation":"field(x,0)","error_precedence":[]}),
        json!({"id":format!("{id}.value"),"argument_type_ids":[id],"normal_result_type_id":value,"equation":"field(x,1)","error_precedence":[]}),
        json!({"id":format!("{id}.equal"),"argument_type_ids":[id,id],"normal_result_type_id":BOOL_TYPE_ID,"equation":"fieldwise_equal(x,y)","error_precedence":[]}),
    ];
    if relation.compare.is_some() {
        expected.push(json!({"id":format!("{id}.compare"),"argument_type_ids":[id,id],"normal_result_type_id":I32_TYPE_ID,"equation":"fieldwise_lexicographic_compare(x,y)","error_precedence":[]}));
    }
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carriers = r
        .carriers
        .iter()
        .map(|(id, c)| (id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let generated = r
        .storage
        .get(&mut r.b, &carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Product { operations } = generated.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if operations.fields.len() != 2
        || operations.fields[0].field_id != "key"
        || operations.fields[1].field_id != "value"
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    Ok(OrdinaryEntryDefinition {
        carrier,
        key_type_id: key.clone(),
        value_type_id: value.clone(),
        make_definition: operations.make_definition,
        key_definition: operations.fields[0].definition.clone(),
        value_definition: operations.fields[1].definition.clone(),
        equality_definition: relation.equal,
        compare_definition: relation.compare,
    })
}
