//! All expanded bounded-sequence operations, including uninvoked operations.
//! Representation/public domains remain caller obligations; read also requires
//! the generated index-range failure predicate to be false.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySequenceOperations {
    pub carrier: OrdinaryCarrier,
    pub element_type_id: String,
    pub length_definition: String,
    pub read_definition: String,
    pub index_range_definition: String,
    pub equality_definition: String,
    /// Exactly absent when the expanded foundation omits compare.
    pub compare_definition: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySequenceProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinarySequenceOperations>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinarySequenceProgram {
    pub fn definitions(&self) -> &[OrdinarySequenceOperations] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed bounded-sequence program")
    }
}

pub fn generate_csharp_practical_ordinary_sequences(
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySequenceProgram> {
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
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.bounded_sequence.v1")
    {
        definitions.push(emit_sequence(&mut r, entry)?);
    }
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let program = OrdinarySequenceProgram {
        schema: "mpk.csharp.ordinary_sequences.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}

pub fn import_csharp_practical_ordinary_sequences(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySequenceProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_sequences(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub(super) fn emit_sequence(r: &mut Relations<'_>, entry: &Value) -> R<OrdinarySequenceOperations> {
    let id = text(entry, "instance_id")?;
    let carrier = r
        .carriers
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    let OrdinaryShape::Sequence { capacity, element } = &carrier.shape else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let OrdinaryShape::Reference {
        type_id: element_type_id,
    } = element.as_ref()
    else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if *capacity != ARRAY_VALUE_LENGTH_MAX as u32 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let metadata = &r.vir.data_closed().metadata[id];
    if metadata.argument_ids.as_slice() != std::slice::from_ref(element_type_id) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let relation = r.ty(id)?;
    let mut expected = vec![
        json!({"id":format!("{id}.length"),"argument_type_ids":[id],"normal_result_type_id":"mpk.csharp.value.u32.v1","equation":"seq_length(x)","error_precedence":[]}),
        json!({"id":format!("{id}.read"),"argument_type_ids":[id,I32_TYPE_ID],"normal_result_type_id":element_type_id,"equation":"seq_read(x,i)","error_precedence":["index_range"]}),
        json!({"id":format!("{id}.equal"),"argument_type_ids":[id,id],"normal_result_type_id":BOOL_TYPE_ID,"equation":"same_length_and_all_equal(x,y)","error_precedence":[]}),
    ];
    if relation.compare.is_some() {
        expected.push(json!({"id":format!("{id}.compare"),"argument_type_ids":[id,id],"normal_result_type_id":I32_TYPE_ID,"equation":"lexicographic_compare(x,y)","error_precedence":[]}));
    }
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let prefix = n(id);
    let length_definition = format!("{prefix}.Length");
    let raw_read = format!("{prefix}.ReadAt");
    let read_definition = format!("{prefix}.CheckedRead");
    let index_range_definition = format!("{prefix}.IndexRange");
    // Source contract recipes and the later structural consumer share a builder.
    // Reuse this exact VIR-validated operation pair, never emit duplicate globals.
    let read_exists = r.b.globals.contains_key(&read_definition);
    let range_exists = r.b.globals.contains_key(&index_range_definition);
    if read_exists != range_exists {
        return Err(OrdinaryCarrierError::Linkage);
    }
    if read_exists {
        return Ok(OrdinarySequenceOperations {
            element_type_id: element_type_id.clone(),
            carrier,
            length_definition,
            read_definition,
            index_range_definition,
            equality_definition: relation.equal,
            compare_definition: relation.compare,
        });
    }
    let sequence = r.b.var(1)?;
    let index = r.b.var(0)?;
    let length = call(&mut r.b, &length_definition, vec![sequence])?;
    let within_length = ordered_fold::helper(&mut r.b, "Less", vec![index, length])?;
    let bound = ordered_fold::word(&mut r.b, *capacity)?;
    // Compare the entire unsigned i32 bit pattern before using its low
    // address bits. Negative indices exceed the physical capacity as well.
    let within_capacity = ordered_fold::helper(&mut r.b, "Less", vec![index, bound])?;
    let valid = and(&mut r.b, within_length, within_capacity)?;
    let failed = call(&mut r.b, "Std.Bool.not", vec![valid])?;
    define(
        &mut r.b,
        &index_range_definition,
        &[carrier.depth, 5],
        0,
        failed,
    )?;
    let child_depth = r
        .carriers
        .get(element_type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    // Select at Bool leaves; no function-valued recursor is introduced.
    let sequence = r.b.var(child_depth + 1)?;
    let index = r.b.var(child_depth)?;
    let failed = call(&mut r.b, &index_range_definition, vec![sequence, index])?;
    let value = call(&mut r.b, &raw_read, vec![sequence, index])?;
    let selectors = r.b.selectors(child_depth)?;
    let leaf = r.b.app(value, selectors)?;
    let zero = bit(&mut r.b, false)?;
    let leaf = mux(&mut r.b, failed, zero, leaf)?;
    let body = r.b.wrap_selectors(child_depth, leaf)?;
    define(
        &mut r.b,
        &read_definition,
        &[carrier.depth, 5],
        child_depth,
        body,
    )?;
    let element_type_id = element_type_id.clone();
    Ok(OrdinarySequenceOperations {
        carrier,
        element_type_id,
        length_definition,
        read_definition,
        index_range_definition,
        equality_definition: relation.equal,
        compare_definition: relation.compare,
    })
}
