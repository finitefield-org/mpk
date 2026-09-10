//! Ordered complete input objects yielding semantic arguments. Source-domain,
//! canonical typed-value and reconstruction obligations remain separate.
use super::sequences::{mux, zero};
use super::*;
use crate::csharp_practical_source_artifacts::PracticalJsonValue as J;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonEnvelopeField {
    pub field_id: String,
    pub semantic_type_id: String,
    pub argument_projection_definition: String,
    pub step_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonEnvelopeDefinition {
    pub contract_sha256: String,
    pub arguments_shape: OrdinaryShape,
    pub arguments_depth: u32,
    pub fields: Vec<OrdinaryJsonEnvelopeField>,
    /// C24 document -> C(max(arguments depth,7)+1) header/arguments packet.
    pub parse_definition: String,
    /// Present when parse_definition also applies the canonical typed-depth guard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unguarded_parse_definition: Option<String>,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonEnvelopeProgram {
    schema: String,
    parsers: OrdinaryJsonProductProgram,
    field_decoders: Vec<OrdinaryJsonBoundaryFieldDefinition>,
    values: Vec<OrdinaryLiteralDefinition>,
    definitions: Vec<OrdinaryJsonEnvelopeDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    typed_depth: Vec<OrdinaryJsonTypedDepthDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    typed_nodes: Vec<OrdinaryJsonTypedNodeDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_limits: Option<OrdinaryJsonRawLimitsDefinition>,
}
impl OrdinaryJsonEnvelopeProgram {
    pub fn definitions(&self) -> &[OrdinaryJsonEnvelopeDefinition] {
        &self.definitions
    }
    pub fn typed_depth(&self) -> &[OrdinaryJsonTypedDepthDefinition] {
        &self.typed_depth
    }
    pub fn typed_nodes(&self) -> &[OrdinaryJsonTypedNodeDefinition] {
        &self.typed_nodes
    }
    pub fn raw_limits(&self) -> Option<&OrdinaryJsonRawLimitsDefinition> {
        self.raw_limits.as_ref()
    }
    pub fn field_decoders(&self) -> &[OrdinaryJsonBoundaryFieldDefinition] {
        &self.field_decoders
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        self.parsers.certificate_bytes()
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON envelopes")
    }
}
pub fn generate_csharp_practical_ordinary_json_envelopes(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    generate_envelopes(emitted, EnvelopeMode::Parse)
}
/// Parse the object and check every decoded/defaulted typed field at depth1.
/// Raw-node limits, representation/source domains and AcceptInput remain separate.
pub fn generate_csharp_practical_ordinary_json_depth_guarded_envelopes(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    generate_envelopes(emitted, EnvelopeMode::TypedDepth)
}
/// Parse ordered input fields and enforce each canonical typed value's depth
/// and node bounds, including defaults. Raw-tree and source-domain obligations
/// remain separate; this program is not the complete AcceptInput relation.
pub fn generate_csharp_practical_ordinary_json_typed_guarded_envelopes(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    generate_envelopes(emitted, EnvelopeMode::TypedLimits)
}
pub fn import_csharp_practical_ordinary_json_typed_guarded_envelopes(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_typed_guarded_envelopes(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
/// Raw document size/node/depth guard followed by typed decoding/default limits.
/// Source/public domains, reconstruction and application proofs remain separate.
pub fn generate_csharp_practical_ordinary_json_limits_guarded_envelopes(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    generate_envelopes(emitted, EnvelopeMode::RawAndTypedLimits)
}
pub fn import_csharp_practical_ordinary_json_limits_guarded_envelopes(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_limits_guarded_envelopes(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
fn generate_envelopes(
    emitted: &EmittedDataPhase,
    mode: EnvelopeMode,
) -> R<OrdinaryJsonEnvelopeProgram> {
    let (parsers, field_decoders, values, definitions, typed_depth, typed_nodes, raw_limits) =
        generate_with_fields(emitted.vir(), Some(emitted), mode)?;
    let p = OrdinaryJsonEnvelopeProgram {
        schema: if mode == EnvelopeMode::RawAndTypedLimits {
            "mpk.csharp.ordinary_json_limits_guarded_envelopes.v1"
        } else if mode == EnvelopeMode::TypedLimits {
            "mpk.csharp.ordinary_json_typed_guarded_envelopes.v1"
        } else if mode == EnvelopeMode::TypedDepth {
            "mpk.csharp.ordinary_json_depth_guarded_envelopes.v1"
        } else {
            "mpk.csharp.ordinary_json_envelopes.v1"
        }
        .into(),
        parsers,
        field_decoders,
        values,
        definitions,
        typed_depth,
        typed_nodes,
        raw_limits,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_envelopes(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_envelopes(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_depth_guarded_envelopes(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonEnvelopeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_depth_guarded_envelopes(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
/// Keep the original parse definition unchanged; guard its completed packet.
pub(super) fn depth_guards(
    b: &mut Builder,
    definitions: &mut [OrdinaryJsonEnvelopeDefinition],
    depths: &[OrdinaryJsonTypedDepthDefinition],
) -> R<()> {
    for d in definitions {
        let original = d.parse_definition.clone();
        let guarded = format!("{original}.TypedDepth");
        let mut scope = Scope::default();
        let doc = scope.outer(b, 0)?;
        let packet = call(b, &original, vec![doc])?;
        let packet = scope.bind(b, d.packet_depth, packet)?;
        let value = scope.bound(b, packet)?;
        let header = call(b, &d.header_definition, vec![value])?;
        let mut valid = core_read(b, header, 0, 7)?;
        for field in &d.fields {
            let depth = depths
                .iter()
                .find(|v| v.carrier.type_id == field.semantic_type_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let value = scope.bound(b, packet)?;
            let args = call(b, &d.value_definition, vec![value])?;
            let value = call(b, &field.argument_projection_definition, vec![args])?;
            let root = word(b, 1, 5)?;
            let good = call(b, &depth.definition, vec![value, root])?;
            valid = call(b, "Std.Bool.and", vec![valid, good])?;
        }
        let value = scope.bound(b, packet)?;
        let z = zero(b, d.packet_depth)?;
        let body = mux(b, d.packet_depth, valid, value, z)?;
        let body = scope.finish(b, body)?;
        define(b, &guarded, &[24], d.packet_depth, body)?;
        d.unguarded_parse_definition = Some(original);
        d.parse_definition = guarded;
    }
    Ok(())
}
/// Retain the completed depth-guarded parser and mask its entire packet if any
/// decoded/defaulted field's canonical tree exceeds the frozen node bound.
pub(super) fn node_guards(
    b: &mut Builder,
    definitions: &mut [OrdinaryJsonEnvelopeDefinition],
    nodes: &[OrdinaryJsonTypedNodeDefinition],
) -> R<()> {
    for d in definitions {
        let original = d.parse_definition.clone();
        let guarded = format!("{original}.TypedNodes");
        let mut scope = Scope::default();
        let doc = scope.outer(b, 0)?;
        let packet = call(b, &original, vec![doc])?;
        let packet = scope.bind(b, d.packet_depth, packet)?;
        let value = scope.bound(b, packet)?;
        let header = call(b, &d.header_definition, vec![value])?;
        let mut valid = core_read(b, header, 0, 7)?;
        for field in &d.fields {
            let node = nodes
                .iter()
                .find(|v| v.carrier.type_id == field.semantic_type_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let value = scope.bound(b, packet)?;
            let args = call(b, &d.value_definition, vec![value])?;
            let value = call(b, &field.argument_projection_definition, vec![args])?;
            let good = call(b, &node.valid_definition, vec![value])?;
            valid = call(b, "Std.Bool.and", vec![valid, good])?;
        }
        let value = scope.bound(b, packet)?;
        let zero = zero(b, d.packet_depth)?;
        let body = mux(b, d.packet_depth, valid, value, zero)?;
        let body = scope.finish(b, body)?;
        define(b, &guarded, &[24], d.packet_depth, body)?;
        d.parse_definition = guarded;
    }
    Ok(())
}
pub(super) fn raw_guards(
    b: &mut Builder,
    definitions: &mut [OrdinaryJsonEnvelopeDefinition],
    raw: &OrdinaryJsonRawLimitsDefinition,
) -> R<()> {
    for d in definitions {
        let original = d.parse_definition.clone();
        let guarded = format!("{original}.RawLimits");
        let doc = b.var(0)?;
        let valid = call(b, &raw.valid_definition, vec![doc])?;
        let packet = call(b, &original, vec![doc])?;
        let zero = zero(b, d.packet_depth)?;
        let body = mux(b, d.packet_depth, valid, packet, zero)?;
        define(b, &guarded, &[24], d.packet_depth, body)?;
        d.parse_definition = guarded;
    }
    Ok(())
}
fn syntax(
    b: &mut Builder,
    doc: u32,
    head: u32,
    matcher: &str,
    grammar: &OrdinaryJsonGrammarDefinition,
) -> R<u32> {
    let cursor = call(b, &grammar.cursor_definition, vec![head])?;
    let matched = call(b, matcher, vec![doc, cursor])?;
    call(b, &grammar.syntax_definition, vec![head, matched])
}
fn add_default(b: &mut Builder) -> R<String> {
    let name = format!("{PREFIX}.JsonEnvelope.AddDefaultCells");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let mut c = Circuit::new(&[128, 32]);
    let head = c.inputs[0].clone();
    let added = c.inputs[1].clone();
    let mut valid = super::super::json_grammar::header_valid(&mut c, &head);
    let positive = c.lt(&super::super::temporal::literal(0, 32), &added, false);
    valid = c.and(valid, positive);
    let (cells, overflow) = c.add(&head[34..66], &added, F);
    let no_overflow = c.not(overflow);
    valid = c.and(valid, no_overflow);
    let bounded = c.lt(
        &cells,
        &super::super::temporal::literal(u128::from(TOTAL_VALUE_CELLS_MAX) + 1, 32),
        false,
    );
    valid = c.and(valid, bounded);
    let mut out = head;
    out[34..66].copy_from_slice(&cells);
    let out = c.select(valid, &out, &vec![F; 128]);
    let helper = super::super::integer_format::circuit_with_block_bits(b, &name, c, out, 7)?;
    let head = b.var(1)?;
    let added = b.var(0)?;
    let body = call(b, &helper, vec![head, added])?;
    define(b, &name, &[7, 5], 7, body)?;
    Ok(name)
}
fn field_access(
    b: &mut Builder,
    base: &str,
    index: u32,
    roles: u32,
    depth: u32,
    field_depth: u32,
) -> R<(String, String)> {
    if roles + field_depth > depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    let get = format!("{base}.Argument.A{index}.Get");
    let write = format!("{base}.Argument.A{index}.Write");
    let input = b.var(field_depth)?;
    let mut selectors = (0..roles)
        .map(|i| truth(b, index & (1 << i) != 0))
        .collect::<R<Vec<_>>>()?;
    selectors.extend(vec![
        truth(b, false)?;
        (depth - roles - field_depth) as usize
    ]);
    selectors.extend(b.selectors(field_depth)?);
    let body = b.app(input, selectors)?;
    let body = b.wrap_selectors(field_depth, body)?;
    define(b, &get, &[depth], field_depth, body)?;
    let previous = b.var(depth + 1)?;
    let value = b.var(depth)?;
    let selectors = b.selectors(depth)?;
    let previous = b.app(previous, selectors)?;
    let selectors = b.selectors(field_depth)?;
    let mut leaf = b.app(value, selectors)?;
    let z = truth(b, false)?;
    for i in roles..depth - field_depth {
        let bit = b.var(depth - 1 - i)?;
        leaf = core_mux(b, bit, z, leaf)?;
    }
    let mut selected = truth(b, true)?;
    for i in 0..roles {
        let mut actual = b.var(depth - 1 - i)?;
        if index & (1 << i) == 0 {
            actual = call(b, "Std.Bool.not", vec![actual])?;
        }
        selected = call(b, "Std.Bool.and", vec![selected, actual])?;
    }
    let body = core_mux(b, selected, leaf, previous)?;
    let body = b.wrap_selectors(depth, body)?;
    define(b, &write, &[depth, field_depth], depth, body)?;
    Ok((get, write))
}
fn either_ending(b: &mut Builder, field: &OrdinaryJsonBoundaryFieldDefinition) -> R<String> {
    let name = format!("{}.CommaOrObjectEnd", field.parse_definition);
    let mut scope = Scope::default();
    let doc = scope.outer(b, 2)?;
    let start = scope.outer(b, 1)?;
    let depth = scope.outer(b, 0)?;
    let ending = word(b, 1, 3)?;
    let first = call(b, &field.parse_definition, vec![doc, start, ending, depth])?;
    let first = scope.bind(b, field.packet_depth, first)?;
    let first = scope.bound(b, first)?;
    let head = call(b, &field.header_definition, vec![first])?;
    let valid = core_read(b, head, 0, 7)?;
    let doc = scope.outer(b, 2)?;
    let start = scope.outer(b, 1)?;
    let depth = scope.outer(b, 0)?;
    let ending = word(b, 3, 3)?;
    let second = call(b, &field.parse_definition, vec![doc, start, ending, depth])?;
    let body = mux(b, field.packet_depth, valid, first, second)?;
    let body = scope.finish(b, body)?;
    define(b, &name, &[24, 5, 5], field.packet_depth, body)?;
    Ok(name)
}
struct State {
    depth: u32,
    pack: String,
    head: String,
    seen: String,
    arguments: String,
}
fn state(b: &mut Builder, base: &str, arguments: &OrdinaryCarrier) -> R<State> {
    let carrier = OrdinaryCarrier {
        type_id: format!("{base}.State"),
        depth: 2 + 7.max(arguments.depth),
        shape: OrdinaryShape::Product {
            fields: vec![
                OrdinaryField {
                    id: "header".into(),
                    shape: OrdinaryShape::Bits { width: 128 },
                },
                OrdinaryField {
                    id: "seen".into(),
                    shape: OrdinaryShape::Bits { width: 1 },
                },
                OrdinaryField {
                    id: "arguments".into(),
                    shape: OrdinaryShape::Reference {
                        type_id: arguments.type_id.clone(),
                    },
                },
            ],
        },
    };
    let carriers = BTreeMap::from([(arguments.type_id.as_str(), arguments)]);
    let storage = super::super::super::structural::emit(b, &carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Product { operations } = storage.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let get = |field: &str| -> R<String> {
        operations
            .fields
            .iter()
            .find(|f| f.field_id == field)
            .map(|f| f.definition.clone())
            .ok_or(OrdinaryCarrierError::Linkage)
    };
    let pack = format!("{base}.PackState");
    let head = b.var(2)?;
    let seen = b.var(1)?;
    let args = b.var(0)?;
    let value = call(b, &operations.make_definition, vec![head, seen, args])?;
    let valid = core_read(b, head, 0, 7)?;
    let z = zero(b, carrier.depth)?;
    let body = mux(b, carrier.depth, valid, value, z)?;
    define(b, &pack, &[7, 0, arguments.depth], carrier.depth, body)?;
    Ok(State {
        depth: carrier.depth,
        pack,
        head: get("header")?,
        seen: get("seen")?,
        arguments: get("arguments")?,
    })
}
impl Products<'_> {
    pub(super) fn envelopes(
        &mut self,
        emitted: &EmittedDataPhase,
        fields: &[OrdinaryJsonBoundaryFieldDefinition],
    ) -> R<Vec<OrdinaryJsonEnvelopeDefinition>> {
        let mut definitions = vec![];
        for contract in emitted.boundaries() {
            let id = contract
                .artifact()
                .value()
                .get("contract_sha256")
                .and_then(J::as_str)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let selected = fields
                .iter()
                .filter(|f| f.contract_sha256 == id)
                .collect::<Vec<_>>();
            if selected
                .iter()
                .map(|f| f.field_id.as_str())
                .ne(contract.input_fields().iter().map(|f| f.id()))
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let base = format!("{PREFIX}.JsonEnvelope.H{id}");
            let count = u32::try_from(selected.len()).map_err(|_| OrdinaryCarrierError::Limit)?;
            let roles = address_bits(count);
            let depth = roles
                + selected
                    .iter()
                    .map(|f| f.semantic_carrier.depth)
                    .max()
                    .unwrap_or(0);
            let arguments = OrdinaryCarrier {
                type_id: format!("{base}.Arguments"),
                depth,
                shape: OrdinaryShape::Product {
                    fields: selected
                        .iter()
                        .map(|f| OrdinaryField {
                            id: f.field_id.clone(),
                            shape: OrdinaryShape::Reference {
                                type_id: f.semantic_carrier.type_id.clone(),
                            },
                        })
                        .collect(),
                },
            };
            let state = state(self.b, &base, &arguments)?;
            let add = add_default(self.b)?;
            let comma = self.punctuation(b',')?;
            let mut results = vec![];
            for (index, field) in selected.iter().enumerate() {
                let (get, write) = field_access(
                    self.b,
                    &base,
                    index as u32,
                    roles,
                    depth,
                    field.semantic_carrier.depth,
                )?;
                let parse = either_ending(self.b, field)?;
                let mut scope = Scope::default();
                let old = scope.outer(self.b, 0)?;
                let head = call(self.b, &state.head, vec![old])?;
                let head = scope.bind(self.b, 7, head)?;
                let old = scope.outer(self.b, 0)?;
                let seen = call(self.b, &state.seen, vec![old])?;
                let seen = scope.bind(self.b, 0, seen)?;
                let old = scope.outer(self.b, 0)?;
                let args = call(self.b, &state.arguments, vec![old])?;
                let args = scope.bind(self.b, depth, args)?;
                let doc = scope.outer(self.b, 1)?;
                let current = scope.bound(self.b, head)?;
                let after_comma = syntax(self.b, doc, current, &comma, self.grammar)?;
                let was_seen = scope.bound(self.b, seen)?;
                let candidate = mux(self.b, 7, was_seen, after_comma, current)?;
                let candidate = scope.bind(self.b, 7, candidate)?;
                let doc = scope.outer(self.b, 1)?;
                let current = scope.bound(self.b, candidate)?;
                let named = syntax(
                    self.b,
                    doc,
                    current,
                    &field.name_match_definition,
                    self.grammar,
                )?;
                let named = scope.bind(self.b, 7, named)?;
                let named_head = scope.bound(self.b, named)?;
                let present = core_read(self.b, named_head, 0, 7)?;
                let present = scope.bind(self.b, 0, present)?;
                let doc = scope.outer(self.b, 1)?;
                let named_head = scope.bound(self.b, named)?;
                let start = call(self.b, &self.grammar.cursor_definition, vec![named_head])?;
                let raw_depth = word(self.b, 1, 5)?;
                let parsed = call(self.b, &parse, vec![doc, start, raw_depth])?;
                let parsed = scope.bind(self.b, field.packet_depth, parsed)?;
                let packet = scope.bound(self.b, parsed)?;
                let child = call(self.b, &field.header_definition, vec![packet])?;
                let named_head = scope.bound(self.b, named)?;
                let joined = call(
                    self.b,
                    &self.grammar.child_definition,
                    vec![named_head, child],
                )?;
                let joined = scope.bind(self.b, 7, joined)?;
                let missing = if let Some(cells) = field.missing_cells {
                    let cells = u32::try_from(cells).map_err(|_| OrdinaryCarrierError::Limit)?;
                    let cells = word(self.b, cells, 5)?;
                    let head = scope.bound(self.b, head)?;
                    call(self.b, &add, vec![head, cells])?
                } else {
                    zero(self.b, 7)?
                };
                let missing = scope.bind(self.b, 7, missing)?;
                let supplied = scope.bound(self.b, joined)?;
                let omitted = scope.bound(self.b, missing)?;
                let is_present = scope.bound(self.b, present)?;
                let final_head = mux(self.b, 7, is_present, supplied, omitted)?;
                let final_head = scope.bind(self.b, 7, final_head)?;
                let packet = scope.bound(self.b, parsed)?;
                let supplied = call(self.b, &field.value_definition, vec![packet])?;
                let default = if let Some(name) = &field.missing_value_definition {
                    self.b.constant(name)?
                } else {
                    zero(self.b, field.semantic_carrier.depth)?
                };
                let is_present = scope.bound(self.b, present)?;
                let value = mux(
                    self.b,
                    field.semantic_carrier.depth,
                    is_present,
                    supplied,
                    default,
                )?;
                let value = scope.bind(self.b, field.semantic_carrier.depth, value)?;
                let args = scope.bound(self.b, args)?;
                let value = scope.bound(self.b, value)?;
                let args = call(self.b, &write, vec![args, value])?;
                let is_present = scope.bound(self.b, present)?;
                let was_seen = scope.bound(self.b, seen)?;
                let seen = call(self.b, "Std.Bool.or", vec![was_seen, is_present])?;
                let head = scope.bound(self.b, final_head)?;
                let body = call(self.b, &state.pack, vec![head, seen, args])?;
                let body = scope.finish(self.b, body)?;
                let step = format!("{base}.Field.A{index}.Step");
                define(self.b, &step, &[24, state.depth], state.depth, body)?;
                results.push(OrdinaryJsonEnvelopeField {
                    field_id: field.field_id.clone(),
                    semantic_type_id: field.semantic_carrier.type_id.clone(),
                    argument_projection_definition: get,
                    step_definition: step,
                });
            }
            let doc = self.b.var(0)?;
            let len = call(self.b, &self.document.length_definition, vec![doc])?;
            let start = word(self.b, 0, 5)?;
            let raw_depth = word(self.b, 0, 5)?;
            let children = truth(self.b, !selected.is_empty())?;
            let head = call(
                self.b,
                &self.grammar.begin_definition,
                vec![len, start, raw_depth, children],
            )?;
            let open = self.punctuation(b'{')?;
            let head = syntax(self.b, doc, head, &open, self.grammar)?;
            let seen = truth(self.b, false)?;
            let args = zero(self.b, depth)?;
            let initial = call(self.b, &state.pack, vec![head, seen, args])?;
            let steps = results
                .iter()
                .map(|field| call(self.b, &field.step_definition, vec![doc]))
                .collect::<R<Vec<_>>>()?;
            // Each concrete field step is S -> S in the document binder.
            // Use the shared balanced expansion and charge every occurrence.
            let pipeline = self.b.compose(state.depth, &steps)?;
            let value = self.b.app(pipeline, vec![initial])?;
            // Bind the completed state once; field count does not add binders.
            let mut scope = Scope::default();
            let completed = scope.bind(self.b, state.depth, value)?;
            let doc = scope.outer(self.b, 0)?;
            let completed = scope.bound(self.b, completed)?;
            let head = call(self.b, &state.head, vec![completed])?;
            let close = self.punctuation(b'}')?;
            let head = syntax(self.b, doc, head, &close, self.grammar)?;
            let ending = word(self.b, 0, 3)?;
            let head = call(
                self.b,
                &self.grammar.finish_definition,
                vec![doc, head, ending],
            )?;
            let args = call(self.b, &state.arguments, vec![completed])?;
            let assemble = super::super::json_grammar::assemble(self.b, depth)?;
            let body = call(self.b, &assemble, vec![head, args])?;
            let body = scope.finish(self.b, body)?;
            let packet_depth = depth.max(7) + 1;
            let parse_definition = format!("{base}.Parse");
            define(self.b, &parse_definition, &[24], packet_depth, body)?;
            let header_definition =
                super::super::json_values::projection(self.b, packet_depth, 7, false)?;
            let value_definition =
                super::super::json_values::projection(self.b, packet_depth, depth, true)?;
            definitions.push(OrdinaryJsonEnvelopeDefinition {
                contract_sha256: id.into(),
                arguments_shape: arguments.shape,
                arguments_depth: depth,
                fields: results,
                parse_definition,
                unguarded_parse_definition: None,
                packet_depth,
                header_definition,
                value_definition,
            });
        }
        Ok(definitions)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{apply, bit, run, V};
    use super::*;
    fn at(cert: &Certificate, mut v: V, depth: u32, index: usize) -> bool {
        for i in 0..depth {
            v = apply(cert, v, V::Bit(index & (1 << i) != 0));
        }
        bit(v)
    }
    #[test]
    fn json_envelope_default_cells_preserve_cursor_and_enforce_total() {
        let mut b = Builder::new().unwrap();
        let name = add_default(&mut b).unwrap();
        let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let mut cases = 0;
        for previous in [0u32, 1, 65535, 65536, 65537, u32::MAX] {
            for added in [0u32, 1, 65534, 65535, 65536, u32::MAX] {
                for invalid in [None, Some(0), Some(66), Some(127)] {
                    let mut head = vec![false; 128];
                    head[0] = true;
                    for i in 0..32 {
                        head[2 + i] = 23 & (1 << i) != 0;
                        head[34 + i] = previous & (1 << i) != 0;
                    }
                    if let Some(i) = invalid {
                        head[i] = !head[i];
                    }
                    let actual = run(
                        &cert,
                        &name,
                        vec![
                            V::Cube(head.clone()),
                            V::Cube((0..32).map(|i| added & (1 << i) != 0).collect()),
                        ],
                    );
                    let total = u64::from(previous) + u64::from(added);
                    if invalid.is_none()
                        && (1..=65536).contains(&previous)
                        && added > 0
                        && total <= 65536
                    {
                        for i in 0..32 {
                            head[34 + i] = total & (1 << i) != 0;
                        }
                    } else {
                        head.fill(false);
                    }
                    for (i, want) in head.into_iter().enumerate() {
                        assert_eq!(
                            at(&cert, actual.clone(), 7, i),
                            want,
                            "cells{previous}+{added},invalid{invalid:?},bit{i}"
                        );
                    }
                    cases += 1;
                }
            }
        }
        eprintln!("Envelope omitted-cell join:{cases} complete headers");
    }
    #[test]
    fn json_envelope_argument_writes_preserve_other_fields_and_clear_padding() {
        let mut b = Builder::new().unwrap();
        let mut definitions = vec![];
        for (roles, depth, field_depth, indices) in [
            (8, 10, 0, vec![0, 1, 127, 128, 255]),
            (1, 10, 5, vec![0, 1]),
            (0, 9, 9, vec![0]),
        ] {
            for index in indices {
                let base = format!("Test.Envelope.R{roles}.D{depth}.F{field_depth}");
                let (get, write) =
                    field_access(&mut b, &base, index, roles, depth, field_depth).unwrap();
                definitions.push((roles, depth, field_depth, index, get, write));
            }
        }
        let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut bits = 0;
        for (roles, depth, field_depth, index, get, write) in definitions {
            let previous = (0..1usize << depth).map(|i| i % 3 != 0).collect::<Vec<_>>();
            let replacement = (0..1usize << field_depth)
                .map(|i| i % 3 == 0)
                .collect::<Vec<_>>();
            let actual = run(
                &cert,
                &write,
                vec![
                    V::Cube(previous.clone()),
                    if field_depth == 0 {
                        V::Bit(replacement[0])
                    } else {
                        V::Cube(replacement.clone())
                    },
                ],
            );
            let projected = run(&cert, &get, vec![actual.clone()]);
            for (i, want) in replacement.iter().enumerate() {
                assert_eq!(at(&cert, projected.clone(), field_depth, i), *want);
            }
            for (address, old) in previous.into_iter().enumerate() {
                let selected = address & ((1 << roles) - 1) == index as usize;
                let padding = (address >> roles) & ((1 << (depth - roles - field_depth)) - 1);
                let expected = if selected {
                    padding == 0 && replacement[address >> (depth - field_depth)]
                } else {
                    old
                };
                assert_eq!(
                    at(&cert, actual.clone(), depth, address),
                    expected,
                    "roles{roles},index{index},address{address}"
                );
                bits += 1;
            }
        }
        eprintln!("Envelope argument writes:{bits} complete storage bits,including256-field address endpoints");
    }
}

#[cfg(test)]
mod raw_guard_tests {
    use super::super::super::test_eval::{apply, bit, run, sparse_cube, V};
    use super::*;

    #[test]
    fn raw_limit_guard_masks_complete_packet_and_preserves_prior_failure() {
        let mut b = Builder::new().unwrap();
        let document = super::super::super::boundary_document::emit(&mut b).unwrap();
        let raw =
            super::super::super::json_raw_limits::emit_with_document(&mut b, document).unwrap();
        let mut cases = vec![];
        for prior_valid in [false, true] {
            let original = format!("Test.RawGuard.Parse{prior_valid}");
            // A valid predecessor packet has nonzero header and argument bits;
            // the actual typed parser returns the complete zero packet on failure.
            let pattern = (0..256)
                .map(|i| if prior_valid && i % 3 == 0 { T } else { F })
                .collect();
            let packet = super::super::super::integer_format::circuit_with_block_bits(
                &mut b,
                &format!("{original}.Packet"),
                Circuit::new(&[]),
                pattern,
                7,
            )
            .unwrap();
            let value = b.constant(&packet).unwrap();
            define(&mut b, &original, &[24], 8, value).unwrap();
            let mut definitions = vec![OrdinaryJsonEnvelopeDefinition {
                contract_sha256: original.clone(),
                arguments_shape: OrdinaryShape::Product { fields: vec![] },
                arguments_depth: 0,
                fields: vec![],
                parse_definition: original.clone(),
                unguarded_parse_definition: Some(original.clone()),
                packet_depth: 8,
                header_definition: "unused-header".into(),
                value_definition: "unused-value".into(),
            }];
            raw_guards(&mut b, &mut definitions, &raw).unwrap();
            assert_eq!(
                definitions[0].unguarded_parse_definition.as_deref(),
                Some(original.as_str())
            );
            cases.push((definitions.remove(0), prior_valid));
        }
        let bytes = b.finish().unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        for (d, prior_valid) in cases {
            for (text, raw_valid) in [
                ("{}".to_owned(), true),
                (format!("{}0{}", "[".repeat(33), "]".repeat(33)), false),
            ] {
                let mut bits = BTreeSet::new();
                for i in 0..32 {
                    if text.len() & (1 << i) != 0 {
                        bits.insert(i << 19);
                    }
                }
                for (i, ch) in text.bytes().enumerate() {
                    for k in 0..8 {
                        if ch & (1 << k) != 0 {
                            bits.insert(1 | (i << 1) | (k << 21));
                        }
                    }
                }
                let result = run(&cert, &d.parse_definition, vec![sparse_cube(24, bits)]);
                for index in 0..256 {
                    let mut value = result.clone();
                    for k in 0..8 {
                        value = apply(&cert, value, V::Bit(index & (1 << k) != 0));
                    }
                    assert_eq!(
                        bit(value),
                        raw_valid && prior_valid && index % 3 == 0,
                        "prior={prior_valid},raw={raw_valid},bit={index}"
                    );
                }
            }
        }
        if let Some(out) = std::env::var_os("MPK_W09_JSON_LIMITS_GUARD_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("raw-packet-mask.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
        eprintln!("Raw limit envelope guard:1024 complete packet bits;actual scanner depth rejection,valid passthrough,and prior failure");
    }
}
