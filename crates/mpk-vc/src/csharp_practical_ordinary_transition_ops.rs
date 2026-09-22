//! Every expanded transition product operation, including uninvoked instances.
//! Public domains and the make event-bound failure remain caller obligations.
//! These operations do not prove source execution, admission, replay or history.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionDefinition {
    pub carrier: OrdinaryCarrier,
    pub state_type_id: String,
    pub event_type_id: String,
    pub events_type_id: String,
    pub response_type_id: String,
    pub make_definition: String,
    pub state_definition: String,
    pub events_definition: String,
    pub response_definition: String,
    /// The make operation's ordered event_bound failure, applied to argument 1.
    pub event_bound_definition: String,
    pub event_bound_argument_index: usize,
    pub equality_definition: String,
    pub compare_definition: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryTransitionDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryTransitionProgram {
    pub fn definitions(&self) -> &[OrdinaryTransitionDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed transition operation program")
    }
}
pub fn generate_csharp_practical_ordinary_transition_operations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTransitionProgram> {
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
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.transition.v1")
    {
        definitions.push(emit_transition(&mut r, entry)?);
    }
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryTransitionProgram {
        schema: "mpk.csharp.ordinary_transition_operations.v1".into(),
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
pub fn import_csharp_practical_ordinary_transition_operations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTransitionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_transition_operations(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

pub(super) fn emit_transition(
    r: &mut Relations<'_>,
    entry: &Value,
) -> R<OrdinaryTransitionDefinition> {
    let id = text(entry, "instance_id")?;
    let metadata = &r.vir.data_closed().metadata[id];
    if metadata.argument_ids.len() != 3 || metadata.dependency_ids.len() != 1 {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let state = metadata.argument_ids[0].clone();
    let event = metadata.argument_ids[1].clone();
    let response = metadata.argument_ids[2].clone();
    let events = metadata.dependency_ids[0].clone();
    let dependency = r
        .vir
        .data_closed()
        .metadata
        .get(&events)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if dependency.template_id != "mpk.csharp.semantic.bounded_sequence.v1"
        || dependency.argument_ids != [event.clone()]
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carrier = r
        .carriers
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    if carrier.shape
        != product(vec![
            field("state", reference(&state)),
            field("events", reference(&events)),
            field("response", reference(&response)),
        ])
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let relation = r.ty(id)?;
    let mut expected = vec![
        json!({"id":format!("{id}.make"),"argument_type_ids":[state,events,response],"normal_result_type_id":id,"equation":"product(state,events,response)","error_precedence":["event_bound"]}),
        json!({"id":format!("{id}.state"),"argument_type_ids":[id],"normal_result_type_id":state,"equation":"field(x,0)","error_precedence":[]}),
        json!({"id":format!("{id}.events"),"argument_type_ids":[id],"normal_result_type_id":events,"equation":"field(x,1)","error_precedence":[]}),
        json!({"id":format!("{id}.response"),"argument_type_ids":[id],"normal_result_type_id":response,"equation":"field(x,2)","error_precedence":[]}),
        json!({"id":format!("{id}.equal"),"argument_type_ids":[id,id],"normal_result_type_id":BOOL_TYPE_ID,"equation":"fieldwise_equal(x,y)","error_precedence":[]}),
    ];
    if relation.compare.is_some() {
        expected.push(json!({"id":format!("{id}.compare"),"argument_type_ids":[id,id],"normal_result_type_id":I32_TYPE_ID,"equation":"fieldwise_lexicographic_compare(x,y)","error_precedence":[]}));
    }
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carriers = r.carriers.iter().map(|(id, c)| (id.as_str(), c)).collect();
    let generated = r
        .storage
        .get(&mut r.b, &carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Product { operations } = generated.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if operations
        .fields
        .iter()
        .map(|f| f.field_id.as_str())
        .collect::<Vec<_>>()
        != ["state", "events", "response"]
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let events_carrier = &r.carriers[&events];
    if events_carrier.shape
        != (OrdinaryShape::Sequence {
            capacity: 4096,
            element: Box::new(reference(&event)),
        })
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let event_bound_definition = format!("{}.Transition.EventBound", n(id));
    let input = r.b.var(0)?;
    let length = call(&mut r.b, &format!("{}.Length", n(&events)), vec![input])?;
    let bound = ordered_fold::word(&mut r.b, 4096)?;
    let failed = ordered_fold::helper(&mut r.b, "Less", vec![bound, length])?;
    define(
        &mut r.b,
        &event_bound_definition,
        &[events_carrier.depth],
        0,
        failed,
    )?;
    Ok(OrdinaryTransitionDefinition {
        carrier,
        state_type_id: state,
        event_type_id: event,
        events_type_id: events,
        response_type_id: response,
        make_definition: operations.make_definition,
        state_definition: operations.fields[0].definition.clone(),
        events_definition: operations.fields[1].definition.clone(),
        response_definition: operations.fields[2].definition.clone(),
        event_bound_definition,
        event_bound_argument_index: 1,
        equality_definition: relation.equal,
        compare_definition: relation.compare,
    })
}
