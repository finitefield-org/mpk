//! Expanded option/result/lookup/validation/boundary-presence operations.
//! Input representation/public domains and all failure gates remain caller
//! obligations. These definitions do not certify source bindings or app VCs.
use super::super::super::scalar_bits::{count_addition, sequence_subtraction};
use super::*;
use ordered_fold::{helper, word};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOutcomeFailure {
    pub label: String,
    pub definition: String,
    pub argument_indices: Vec<usize>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOutcomeOperation {
    pub operation_id: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub normal_definition: String,
    pub failures: Vec<OrdinaryOutcomeFailure>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOutcomeDefinition {
    pub carrier: OrdinaryCarrier,
    pub template_id: String,
    pub operations: Vec<OrdinaryOutcomeOperation>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOutcomeProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryOutcomeDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryOutcomeProgram {
    pub fn definitions(&self) -> &[OrdinaryOutcomeDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed outcome program")
    }
}
#[derive(Clone, Copy)]
enum Action {
    Make(usize),
    Is(usize),
    Get(usize),
    Tag,
    ValueOr,
    Equal,
    Compare,
    Append,
}

fn append_errors(
    r: &mut Relations<'_>,
    sequence: &OrdinaryCarrier,
    name: &str,
) -> R<(String, String)> {
    let OrdinaryShape::Sequence {
        capacity: 4096,
        element,
    } = &sequence.shape
    else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let OrdinaryShape::Reference { type_id } = element.as_ref() else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let child = r
        .carriers
        .get(type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    let depth = sequence.depth;
    if depth != child + 13 {
        return Err(OrdinaryCarrierError::Shape);
    }
    r.ty(&sequence.type_id)?;
    let b = &mut r.b;
    let length = format!("{}.Length", n(&sequence.type_id));
    let read = format!("{}.ReadAt", n(&sequence.type_id));
    let sum = count_addition(b)?;
    let subtract = sequence_subtraction(b)?;
    let left = b.var(1)?;
    let right = b.var(0)?;
    let left_length = call(b, &length, vec![left])?;
    let right_length = call(b, &length, vec![right])?;
    // Each valid input has length <= 4096. The saturated count adder is exact
    // for their sum <= 8192 and cannot wrap malformed lengths into acceptance.
    let total = call(b, &sum, vec![left_length, right_length])?;
    let bound = word(b, 256)?;
    let body = helper(b, "Less", vec![bound, total])?;
    let failure = format!("{name}.AppendBound");
    define(b, &failure, &[depth, depth], 0, body)?;

    let left = b.var(depth + 1)?;
    let right = b.var(depth)?;
    let left_length = call(b, &length, vec![left])?;
    let right_length = call(b, &length, vec![right])?;
    let total = call(b, &sum, vec![left_length, right_length])?;
    let mut index = bit(b, false)?;
    for i in 0..12 {
        let at = equal_address(b, 5, 0, 5, i)?;
        let selector = b.var(5 + depth - 2 - i)?;
        index = mux(b, at, selector, index)?;
    }
    let index = b.wrap_selectors(5, index)?;
    let in_left = helper(b, "Less", vec![index, left_length])?;
    let offset = call(b, &subtract, vec![index, left_length])?;
    let left_value = call(b, &read, vec![left, index])?;
    let right_value = call(b, &read, vec![right, offset])?;
    let selectors = b.selectors(child)?;
    let left_leaf = b.app(left_value, selectors.clone())?;
    let right_leaf = b.app(right_value, selectors)?;
    let leaf = mux(b, in_left, left_leaf, right_leaf)?;
    let active = helper(b, "Less", vec![index, total])?;
    let zero = bit(b, false)?;
    let leaf = mux(b, active, leaf, zero)?;
    let selectors = b.selectors(5)?;
    let count = b.app(total, selectors)?;
    let count = zero_padding(b, depth, 1, depth - 6, count)?;
    let role = b.var(depth - 1)?;
    let body = mux(b, role, leaf, count)?;
    let body = b.wrap_selectors(depth, body)?;
    let definition = format!("{name}.AppendErrors");
    define(b, &definition, &[depth, depth], depth, body)?;
    Ok((definition, failure))
}

pub(super) fn emit_outcome(r: &mut Relations<'_>, entry: &Value) -> R<OrdinaryOutcomeDefinition> {
    let id = text(entry, "instance_id")?;
    let template_id = text(entry, "template_id")?;
    let template = template_id
        .strip_prefix("mpk.csharp.semantic.")
        .and_then(|s| s.strip_suffix(".v1"))
        .ok_or(OrdinaryCarrierError::Shape)?;
    let metadata = &r.vir.data_closed().metadata[id];
    let args = metadata.argument_ids.clone();
    let deps = metadata.dependency_ids.clone();
    if args.len()
        != if matches!(template, "result" | "validation") {
            2
        } else {
            1
        }
        || deps.len() != usize::from(template == "validation")
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carrier = r
        .carriers
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    let v = args[0].as_str();
    let arm = |tag, name: &str, value: Option<OrdinaryShape>| OrdinaryArm {
        tag,
        id: name.into(),
        fields: value.map(|s| vec![field("0", s)]).unwrap_or_default(),
    };
    let shapes = match template {
        "option" => vec![arm(0, "none", None), arm(1, "some", Some(reference(v)))],
        "lookup" => vec![
            arm(0, "missing_key", None),
            arm(1, "found", Some(reference(v))),
        ],
        "result" => vec![
            arm(0, "ok", Some(reference(v))),
            arm(1, "error", Some(reference(&args[1]))),
        ],
        "validation" => vec![
            arm(0, "valid", Some(reference(v))),
            arm(
                1,
                "invalid",
                Some(OrdinaryShape::RoleBound {
                    maximum: 256,
                    value: Box::new(reference(&deps[0])),
                }),
            ),
        ],
        "boundary_field" => vec![
            arm(0, "missing", None),
            arm(1, "null", None),
            arm(2, "value", Some(reference(v))),
        ],
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    if carrier.shape != (OrdinaryShape::Sum { arms: shapes }) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let relation = r.ty(id)?;
    let carriers = r
        .carriers
        .iter()
        .map(|(id, c)| (id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let storage = r
        .storage
        .get(&mut r.b, &carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Sum {
        tag_definition,
        arms,
    } = storage.operations
    else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let mut specs = match template {
        "option" => vec![
            ("none", vec![], id, "sum(0,unit)", Action::Make(0)),
            ("some", vec![v], id, "sum(1,v)", Action::Make(1)),
            (
                "has_value",
                vec![id],
                BOOL_TYPE_ID,
                "tag(x)==1",
                Action::Is(1),
            ),
            ("value", vec![id], v, "active_payload(x,1)", Action::Get(1)),
            (
                "value_or",
                vec![id, v],
                v,
                "if_tag_1_then_payload_else_fallback(x,v)",
                Action::ValueOr,
            ),
        ],
        "lookup" => vec![
            ("missing", vec![], id, "sum(0,unit)", Action::Make(0)),
            ("found", vec![v], id, "sum(1,v)", Action::Make(1)),
            (
                "is_found",
                vec![id],
                BOOL_TYPE_ID,
                "tag(x)==1",
                Action::Is(1),
            ),
            ("value", vec![id], v, "active_payload(x,1)", Action::Get(1)),
        ],
        "result" => vec![
            ("ok", vec![v], id, "sum(0,v)", Action::Make(0)),
            ("error", vec![&args[1]], id, "sum(1,e)", Action::Make(1)),
            ("is_ok", vec![id], BOOL_TYPE_ID, "tag(x)==0", Action::Is(0)),
            ("value", vec![id], v, "active_payload(x,0)", Action::Get(0)),
            (
                "error_value",
                vec![id],
                args[1].as_str(),
                "active_payload(x,1)",
                Action::Get(1),
            ),
        ],
        "validation" => vec![
            ("valid", vec![v], id, "sum(0,v)", Action::Make(0)),
            (
                "invalid",
                vec![&deps[0]],
                id,
                "sum(1,nonempty_errors)",
                Action::Make(1),
            ),
            (
                "is_valid",
                vec![id],
                BOOL_TYPE_ID,
                "tag(x)==0",
                Action::Is(0),
            ),
            ("value", vec![id], v, "active_payload(x,0)", Action::Get(0)),
            (
                "errors",
                vec![id],
                deps[0].as_str(),
                "active_payload(x,1)",
                Action::Get(1),
            ),
            (
                "append_errors",
                vec![&deps[0], &deps[0]],
                deps[0].as_str(),
                "left_errors_then_right_errors(x,y)",
                Action::Append,
            ),
        ],
        "boundary_field" => vec![
            ("missing", vec![], id, "sum(0,unit)", Action::Make(0)),
            ("null", vec![], id, "sum(1,unit)", Action::Make(1)),
            ("value", vec![v], id, "sum(2,v)", Action::Make(2)),
            (
                "tag",
                vec![id],
                "mpk.csharp.value.u32.v1",
                "tag(x)",
                Action::Tag,
            ),
            (
                "payload",
                vec![id],
                v,
                "active_payload(x,2)",
                Action::Get(2),
            ),
        ],
        _ => unreachable!(),
    };
    specs.push((
        "equal",
        vec![id, id],
        BOOL_TYPE_ID,
        "tag_and_active_payload_equal(x,y)",
        Action::Equal,
    ));
    if relation.compare.is_some() {
        specs.push((
            "compare",
            vec![id, id],
            I32_TYPE_ID,
            if template == "option" {
                "null_first_then_payload_compare(x,y)"
            } else {
                "tag_then_active_payload_compare(x,y)"
            },
            Action::Compare,
        ));
    }
    let name = format!("{}.Outcome", n(id));
    let mut invalid_gates = vec![];
    let mut append = None;
    if template == "validation" {
        let sequence = r
            .carriers
            .get(&deps[0])
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        if sequence.shape != sequence_shape(&args[1]) {
            return Err(OrdinaryCarrierError::Shape);
        }
        append = Some(append_errors(r, &sequence, &name)?);
        let b = &mut r.b;
        let value = b.var(0)?;
        let count = call(b, &format!("{}.Length", n(&deps[0])), vec![value])?;
        let empty = helper(b, "Empty", vec![count])?;
        let empty_name = format!("{name}.EmptyErrors");
        define(b, &empty_name, &[sequence.depth], 0, empty)?;
        let bound = word(b, 256)?;
        let failure = helper(b, "Less", vec![bound, count])?;
        let bound_name = format!("{name}.ValidationBound");
        define(b, &bound_name, &[sequence.depth], 0, failure)?;
        invalid_gates = vec![
            ("empty_errors", empty_name),
            ("validation_bound", bound_name),
        ];
    }
    let mut operations = vec![];
    let mut expected = vec![];
    for (suffix, arguments, result, equation, action) in specs {
        let mut failures = vec![];
        let b = &mut r.b;
        let definition = match action {
            Action::Make(tag) => {
                if template == "validation" && tag == 1 {
                    failures.extend(invalid_gates.iter().map(|(label, definition)| {
                        OrdinaryOutcomeFailure {
                            label: (*label).into(),
                            definition: definition.clone(),
                            argument_indices: vec![0],
                        }
                    }));
                }
                arms[tag].make_definition.clone()
            }
            Action::Is(tag) => arms[tag].is_active_definition.clone(),
            Action::Tag => tag_definition.clone(),
            Action::Get(tag) => {
                failures.push(OrdinaryOutcomeFailure {
                    label: "invalid_operation".into(),
                    definition: arms[tag].invalid_operation_definition.clone(),
                    argument_indices: vec![0],
                });
                if arms[tag].fields.len() != 1 {
                    return Err(OrdinaryCarrierError::Shape);
                }
                arms[tag].fields[0].definition.clone()
            }
            Action::ValueOr => {
                let child = r
                    .carriers
                    .get(v)
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .depth;
                let source = b.var(child + 1)?;
                let fallback = b.var(child)?;
                let active = call(b, &arms[1].is_active_definition, vec![source])?;
                let value = call(b, &arms[1].fields[0].definition, vec![source])?;
                let selectors = b.selectors(child)?;
                let value = b.app(value, selectors.clone())?;
                let fallback = b.app(fallback, selectors)?;
                let body = mux(b, active, value, fallback)?;
                let body = b.wrap_selectors(child, body)?;
                let definition = format!("{name}.ValueOr");
                define(b, &definition, &[carrier.depth, child], child, body)?;
                definition
            }
            Action::Equal => relation.equal.clone(),
            Action::Compare => relation
                .compare
                .clone()
                .ok_or(OrdinaryCarrierError::Shape)?,
            Action::Append => {
                let (definition, failure) = append.as_ref().ok_or(OrdinaryCarrierError::Shape)?;
                failures.push(OrdinaryOutcomeFailure {
                    label: "validation_bound".into(),
                    definition: failure.clone(),
                    argument_indices: vec![0, 1],
                });
                definition.clone()
            }
        };
        let operation_id = format!("{id}.{suffix}");
        expected.push(json!({"id":operation_id,"argument_type_ids":arguments,"normal_result_type_id":result,"equation":equation,"error_precedence":failures.iter().map(|f| &f.label).collect::<Vec<_>>()}));
        operations.push(OrdinaryOutcomeOperation {
            operation_id,
            argument_type_ids: arguments.into_iter().map(str::to_owned).collect(),
            result_type_id: result.into(),
            normal_definition: definition,
            failures,
        });
    }
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(OrdinaryOutcomeDefinition {
        carrier,
        template_id: template_id.into(),
        operations,
    })
}
fn sequence_shape(id: &str) -> OrdinaryShape {
    sequence(4096, reference(id))
}
pub fn generate_csharp_practical_ordinary_outcomes(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryOutcomeProgram> {
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
    for entry in vir.data_closed().entries().iter().filter(|e| {
        matches!(
            e["template_id"].as_str(),
            Some(
                "mpk.csharp.semantic.option.v1"
                    | "mpk.csharp.semantic.lookup.v1"
                    | "mpk.csharp.semantic.result.v1"
                    | "mpk.csharp.semantic.validation.v1"
                    | "mpk.csharp.semantic.boundary_field.v1"
            )
        )
    }) {
        definitions.push(emit_outcome(&mut r, entry)?);
    }
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryOutcomeProgram {
        schema: "mpk.csharp.ordinary_outcomes.v1".into(),
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
pub fn import_csharp_practical_ordinary_outcomes(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryOutcomeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_outcomes(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
