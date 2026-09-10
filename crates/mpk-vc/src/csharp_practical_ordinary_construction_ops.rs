//! Concrete construction storage operations. These are normal-result bodies,
//! not an ownership authority: source state, lifetime, defaults, element domains
//! and ordered failure obligations must be discharged during VC assembly.
use super::*;
use ordered_fold::{helper, read_bit, word};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionFailure {
    pub label: String,
    /// None only for ownership, which is a source-state obligation, not a
    /// caller-supplied Boolean or a field in the construction storage carrier.
    pub definition: Option<String>,
    /// Zero-based normal-operation arguments supplied to the predicate.
    pub argument_indices: Vec<usize>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionOperation {
    pub operation_id: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub normal_definition: String,
    pub failures: Vec<OrdinaryConstructionFailure>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionDefinition {
    pub carrier: OrdinaryCarrier,
    pub element_type_id: String,
    pub published_type_id: String,
    pub length_definition: String,
    pub initialized_definition: String,
    pub complete_definition: String,
    pub operations: Vec<OrdinaryConstructionOperation>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryConstructionDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryConstructionProgram {
    pub fn definitions(&self) -> &[OrdinaryConstructionDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed construction program")
    }
}

// The selector group starts at `start` in the outer low-first address. Its
// high word bits are zero. This does not truncate a caller-provided index.
fn address_word(b: &mut Builder, outer: u32, start: u32, width: u32) -> R<u32> {
    if start + width > outer || width > 32 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut value = bit(b, false)?;
    for i in 0..width {
        let at = equal_address(b, 5, 0, 5, i)?;
        let selector = b.var(5 + outer - 1 - start - i)?;
        value = mux(b, at, selector, value)?;
    }
    b.wrap_selectors(5, value)
}
fn negate(b: &mut Builder, value: u32) -> R<u32> {
    let yes = bit(b, true)?;
    let no = bit(b, false)?;
    mux(b, value, no, yes)
}
fn conjunction(b: &mut Builder, a: u32, v: u32) -> R<u32> {
    let no = bit(b, false)?;
    mux(b, a, v, no)
}
fn equal_word(b: &mut Builder, a: u32, v: u32) -> R<u32> {
    let mut result = bit(b, true)?;
    for i in 0..32 {
        let left = read_bit(b, a, i)?;
        let right = read_bit(b, v, i)?;
        let different = negate(b, right)?;
        let same = mux(b, left, right, different)?;
        result = conjunction(b, same, result)?;
    }
    Ok(result)
}
fn read_array(b: &mut Builder, array: u32, index: u32) -> R<u32> {
    let address = (0..14)
        .map(|i| read_bit(b, index, i))
        .collect::<R<Vec<_>>>()?;
    b.app(array, address)
}

pub(super) fn emit_construction(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    element: &OrdinaryCarrier,
    published: &OrdinaryCarrier,
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
    entry: &Value,
) -> R<OrdinaryConstructionDefinition> {
    let expected_shape = product(vec![
        field("length", bits(32)),
        field(
            "cells",
            OrdinaryShape::Array {
                capacity: 16384,
                element: Box::new(reference(&element.type_id)),
            },
        ),
        field(
            "initialized",
            OrdinaryShape::Array {
                capacity: 16384,
                element: Box::new(bits(1)),
            },
        ),
    ]);
    if carrier.shape != expected_shape
        || published.shape != sequence(4096, reference(&element.type_id))
        || shape_depth(&carrier.shape, carriers)? != carrier.depth
        || shape_depth(&published.shape, carriers)? != published.depth
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let id = &carrier.type_id;
    let hex = id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let name = format!("{PREFIX}.Construction.T{hex}");
    let depth = carrier.depth;
    let child = element.depth;
    let array_depth = 14 + child;
    let make = format!("{name}.MakeStorage");
    let body = product_leaf(b, &[5, array_depth, 14], depth, 0)?;
    let body = b.wrap_selectors(depth, body)?;
    define(b, &make, &[5, array_depth, 14], depth, body)?;
    let mut getters = vec![];
    for (role, d) in [5, array_depth, 14].into_iter().enumerate() {
        let getter = format!("{name}.Field{role}");
        let mut address = prefix(2, role as u32);
        address.extend(vec![false; (depth - 2 - d) as usize]);
        project(b, &getter, depth, d, &address, None)?;
        getters.push(getter);
    }
    let length = getters[0].clone();
    let cells = &getters[1];
    let bitmap = &getters[2];
    let initialized = format!("{name}.Initialized");
    let value = b.var(1)?;
    let index = b.var(0)?;
    let array = call(b, bitmap, vec![value])?;
    let body = read_array(b, array, index)?;
    define(b, &initialized, &[depth, 5], 0, body)?;
    let read = format!("{name}.Read");
    let array = call(b, cells, vec![value])?;
    let body = read_array(b, array, index)?;
    define(b, &read, &[depth, 5], child, body)?;

    let range = format!("{name}.IndexRange");
    let count = call(b, &length, vec![value])?;
    let in_length = helper(b, "Less", vec![index, count])?;
    let capacity = word(b, 16384)?;
    let in_capacity = helper(b, "Less", vec![index, capacity])?;
    let valid = conjunction(b, in_length, in_capacity)?;
    let body = negate(b, valid)?;
    define(b, &range, &[depth, 5], 0, body)?;
    let uninitialized = format!("{name}.Uninitialized");
    let body = call(b, &initialized, vec![value, index])?;
    let body = negate(b, body)?;
    define(b, &uninitialized, &[depth, 5], 0, body)?;

    let complete = format!("{name}.Complete");
    let fold = aggregate_fold::emit_fold(b, 14)?;
    let value = b.var(0)?;
    let count = call(b, &length, vec![value])?;
    let array = call(b, bitmap, vec![value])?;
    let all = call(b, &fold.all_definition, vec![array, count])?;
    let too_large = helper(b, "Less", vec![capacity, count])?;
    let in_capacity = negate(b, too_large)?;
    let body = conjunction(b, in_capacity, all)?;
    define(b, &complete, &[depth], 0, body)?;
    let incomplete = format!("{name}.Incomplete");
    let body = call(b, &complete, vec![value])?;
    let body = negate(b, body)?;
    define(b, &incomplete, &[depth], 0, body)?;
    let publication = format!("{name}.PublicationBound");
    let bound = word(b, 4096)?;
    let body = helper(b, "Less", vec![bound, count])?;
    define(b, &publication, &[depth], 0, body)?;
    let negative = format!("{name}.NegativeLength");
    let value = b.var(0)?;
    let body = read_bit(b, value, 31)?;
    define(b, &negative, &[5], 0, body)?;
    let construction_bound = format!("{name}.ConstructionBound");
    let body = helper(b, "Less", vec![capacity, value])?;
    define(b, &construction_bound, &[5], 0, body)?;

    let allocate = format!("{name}.Allocate");
    let count = b.var(15)?;
    let default = b.var(14)?;
    let index = address_word(b, 14, 0, 14)?;
    let in_length = helper(b, "Less", vec![index, count])?;
    let body = conjunction(b, default, in_length)?;
    let bitmap_value = b.wrap_selectors(14, body)?;
    let zero = bit(b, false)?;
    let zero_cells = b.wrap_selectors(array_depth, zero)?;
    let count = b.var(1)?;
    let body = call(b, &make, vec![count, zero_cells, bitmap_value])?;
    define(b, &allocate, &[5, 0], depth, body)?;

    // Functional update replaces one entire stored element and sets precisely
    // its bitmap entry. Range, initialization and ownership are ordered gates.
    // Fill and rewrite have the same successful storage effect, different gates.
    let update = format!("{name}.Update");
    let source = b.var(array_depth + 2)?;
    let index = b.var(array_depth + 1)?;
    let replacement = b.var(array_depth)?;
    let at = address_word(b, array_depth, 0, 14)?;
    let selected = equal_word(b, at, index)?;
    let selectors = b.selectors(child)?;
    let replacement = b.app(replacement, selectors)?;
    let previous = call(b, cells, vec![source])?;
    let selectors = b.selectors(array_depth)?;
    let previous = b.app(previous, selectors)?;
    let body = mux(b, selected, replacement, previous)?;
    let updated_cells = b.wrap_selectors(array_depth, body)?;
    let source = b.var(16)?;
    let index = b.var(15)?;
    let at = address_word(b, 14, 0, 14)?;
    let selected = equal_word(b, at, index)?;
    let previous = call(b, bitmap, vec![source])?;
    let selectors = b.selectors(14)?;
    let previous = b.app(previous, selectors)?;
    let yes = bit(b, true)?;
    let body = mux(b, selected, yes, previous)?;
    let updated_bitmap = b.wrap_selectors(14, body)?;
    let source = b.var(2)?;
    let count = call(b, &length, vec![source])?;
    let body = call(b, &make, vec![count, updated_cells, updated_bitmap])?;
    define(b, &update, &[depth, 5, child], depth, body)?;

    let freeze = format!("{name}.Freeze");
    let output_depth = published.depth;
    let source = b.var(output_depth)?;
    let count = call(b, &length, vec![source])?;
    let selectors = b.selectors(5)?;
    let count_leaf = b.app(count, selectors)?;
    let count_leaf = zero_padding(b, output_depth, 1, output_depth - 1 - 5, count_leaf)?;
    let at = address_word(b, output_depth, 1, 12)?;
    let value = call(b, &read, vec![source, at])?;
    let selectors = b.selectors(child)?;
    let leaf = b.app(value, selectors)?;
    let role = b.var(output_depth - 1)?;
    let body = mux(b, role, leaf, count_leaf)?;
    let body = b.wrap_selectors(output_depth, body)?;
    define(b, &freeze, &[depth], output_depth, body)?;

    let gate = |label: &str, definition: &str, args: &[usize]| OrdinaryConstructionFailure {
        label: label.into(),
        definition: Some(definition.into()),
        argument_indices: args.to_vec(),
    };
    let owner = OrdinaryConstructionFailure {
        label: "ownership".into(),
        definition: None,
        argument_indices: vec![0],
    };
    let definitions = [
        (
            "allocate",
            vec![I32_TYPE_ID, BOOL_TYPE_ID],
            id.as_str(),
            "allocate_cells_and_init_bitmap(n,default_eligible)",
            allocate,
            vec![
                gate("negative_length", &negative, &[0]),
                gate("construction_bound", &construction_bound, &[0]),
            ],
        ),
        (
            "read",
            vec![id, I32_TYPE_ID],
            element.type_id.as_str(),
            "read_initialized_owned_cell(x,i)",
            read,
            vec![
                owner.clone(),
                gate("index_range", &range, &[0, 1]),
                gate("uninitialized", &uninitialized, &[0, 1]),
            ],
        ),
        (
            "fill",
            vec![id, I32_TYPE_ID, &element.type_id],
            id.as_str(),
            "first_write_then_mark_initialized(x,i,v)",
            update.clone(),
            vec![
                owner.clone(),
                gate("index_range", &range, &[0, 1]),
                gate("already_initialized", &initialized, &[0, 1]),
            ],
        ),
        (
            "rewrite",
            vec![id, I32_TYPE_ID, &element.type_id],
            id.as_str(),
            "functional_update_complete_unique_cells(x,i,v)",
            update,
            vec![
                owner.clone(),
                gate("index_range", &range, &[0, 1]),
                gate("incomplete", &incomplete, &[0]),
            ],
        ),
        (
            "freeze",
            vec![id],
            published.type_id.as_str(),
            "publish_all_initialized_at_role_bound(x)",
            freeze,
            vec![
                owner,
                gate("incomplete", &incomplete, &[0]),
                gate("publication_bound", &publication, &[0]),
            ],
        ),
    ];
    let expected = definitions.iter().map(|(suffix,args,result,equation,_,failures)| json!({"id":format!("{id}.{suffix}"), "argument_type_ids":args,"normal_result_type_id":result,"equation":equation,"error_precedence":failures.iter().map(|f| &f.label).collect::<Vec<_>>()})).collect::<Vec<_>>();
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let operations = definitions
        .into_iter()
        .map(|(suffix, args, result, _, normal_definition, failures)| {
            OrdinaryConstructionOperation {
                operation_id: format!("{id}.{suffix}"),
                argument_type_ids: args.into_iter().map(str::to_owned).collect(),
                result_type_id: result.into(),
                normal_definition,
                failures,
            }
        })
        .collect();
    Ok(OrdinaryConstructionDefinition {
        carrier: carrier.clone(),
        element_type_id: element.type_id.clone(),
        published_type_id: published.type_id.clone(),
        length_definition: length,
        initialized_definition: initialized,
        complete_definition: complete,
        operations,
    })
}

pub fn generate_csharp_practical_ordinary_constructions(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConstructionProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let carriers = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let mut b = Builder::new()?;
    ordered_fold::auxiliary(&mut b)?;
    let mut definitions = vec![];
    for entry in vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
    {
        let id = text(entry, "instance_id")?;
        let metadata = &vir.data_closed().metadata[id];
        if metadata.argument_ids.len() != 1 || metadata.dependency_ids.len() != 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let get = |id: &str| {
            carriers
                .get(id)
                .copied()
                .ok_or(OrdinaryCarrierError::Linkage)
        };
        definitions.push(emit_construction(
            &mut b,
            get(id)?,
            get(&metadata.argument_ids[0])?,
            get(&metadata.dependency_ids[0])?,
            &carriers,
            entry,
        )?);
    }
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let program = OrdinaryConstructionProgram {
        schema: "mpk.csharp.ordinary_constructions.v1".into(),
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
pub fn import_csharp_practical_ordinary_constructions(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConstructionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_constructions(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
