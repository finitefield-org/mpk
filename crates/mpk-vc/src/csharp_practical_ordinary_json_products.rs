//! Source and closed semantic JSON parsing with typed child packets and exact syntax.
use super::hex_codecs::{call, truth, word};
use super::integer_format::define;
use super::*;
#[path = "csharp_practical_ordinary_json_sequences.rs"]
mod sequences;
pub use sequences::OrdinaryJsonSequenceDefinition;
#[path = "csharp_practical_ordinary_json_sums.rs"]
mod sums;
pub use sums::{OrdinaryJsonSumArm, OrdinaryJsonSumDefinition};
#[path = "csharp_practical_ordinary_json_collections.rs"]
mod collections;
pub use collections::OrdinaryJsonCollectionDefinition;
#[path = "csharp_practical_ordinary_json_boundary_fields.rs"]
mod boundary_fields;
pub use boundary_fields::{
    generate_csharp_practical_ordinary_json_boundary_fields,
    import_csharp_practical_ordinary_json_boundary_fields, OrdinaryJsonBoundaryFieldDefinition,
    OrdinaryJsonBoundaryFieldProgram,
};

#[path = "csharp_practical_ordinary_json_envelopes.rs"]
mod envelopes;
pub use envelopes::{
    generate_csharp_practical_ordinary_json_envelopes,
    import_csharp_practical_ordinary_json_envelopes, OrdinaryJsonEnvelopeDefinition,
    OrdinaryJsonEnvelopeField, OrdinaryJsonEnvelopeProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonProductDefinition {
    pub carrier: OrdinaryCarrier,
    /// None for a source product; otherwise the reconstructed closed template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// C24 document, C5 start, C3 outer ending, C5 nesting depth (root zero).
    /// Same role/header/value packet as primitive JSON values. Child names and
    /// values are mandatory and occur in source stored-member or semantic role order.
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
    pub member_ids: Vec<String>,
    pub member_names: Vec<String>,
    /// JSON-only containers excluded from this semantic product's cell count.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub container_payload_member_ids: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonProductProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    primitives: Vec<OrdinaryJsonValueDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    enums: Vec<OrdinaryJsonEnumDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    vocabulary: Vec<OrdinaryJsonVocabularyDefinition>,
    products: Vec<OrdinaryJsonProductDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sequences: Vec<OrdinaryJsonSequenceDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sums: Vec<OrdinaryJsonSumDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    collections: Vec<OrdinaryJsonCollectionDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    key_relations: Vec<OrdinaryRelationDefinition>,
    syntax_literals: Vec<OrdinaryJsonSyntaxLiteral>,
    grammar: Option<OrdinaryJsonGrammarDefinition>,
    deferred_type_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonProductProgram {
    pub fn products(&self) -> &[OrdinaryJsonProductDefinition] {
        &self.products
    }
    pub fn collections(&self) -> &[OrdinaryJsonCollectionDefinition] {
        &self.collections
    }
    pub fn key_relations(&self) -> &[OrdinaryRelationDefinition] {
        &self.key_relations
    }
    pub fn sums(&self) -> &[OrdinaryJsonSumDefinition] {
        &self.sums
    }
    pub fn sequences(&self) -> &[OrdinaryJsonSequenceDefinition] {
        &self.sequences
    }
    pub fn primitives(&self) -> &[OrdinaryJsonValueDefinition] {
        &self.primitives
    }
    pub fn enums(&self) -> &[OrdinaryJsonEnumDefinition] {
        &self.enums
    }
    pub fn vocabulary(&self) -> &[OrdinaryJsonVocabularyDefinition] {
        &self.vocabulary
    }
    pub fn deferred_type_ids(&self) -> &[String] {
        &self.deferred_type_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON products")
    }
}

#[derive(Clone)]
struct Child {
    parse: String,
    header: String,
    value: String,
    packet_depth: u32,
    compound: bool,
}
#[derive(Default)]
struct Scope {
    bindings: Vec<(u32, u32)>,
}
impl Scope {
    fn outer(&self, b: &mut Builder, index: u32) -> R<u32> {
        b.var(self.bindings.len() as u32 + index)
    }
    fn bound(&self, b: &mut Builder, index: usize) -> R<u32> {
        b.var((self.bindings.len() - 1 - index) as u32)
    }
    fn bind(&mut self, b: &mut Builder, depth: u32, value: u32) -> R<usize> {
        let index = self.bindings.len();
        self.bindings.push((b.cube(depth)?, value));
        Ok(index)
    }
    fn finish(self, b: &mut Builder, mut body: u32) -> R<u32> {
        for (ty, value) in self.bindings.into_iter().rev() {
            body = b.term(TermNode::Let { ty, value, body })?;
        }
        Ok(body)
    }
}

fn syntax_step(
    b: &mut Builder,
    scope: &mut Scope,
    head: usize,
    matcher: &str,
    grammar: &OrdinaryJsonGrammarDefinition,
) -> R<usize> {
    let doc = scope.outer(b, 3)?;
    let head = scope.bound(b, head)?;
    let start = call(b, &grammar.cursor_definition, vec![head])?;
    let matched = call(b, matcher, vec![doc, start])?;
    let next = call(b, &grammar.syntax_definition, vec![head, matched])?;
    scope.bind(b, 7, next)
}

struct Products<'a> {
    vir: &'a ValidatedPracticalVir,
    b: &'a mut Builder,
    document: &'a OrdinaryBoundaryDocumentDefinition,
    grammar: &'a OrdinaryJsonGrammarDefinition,
    syntax: &'a OrdinaryJsonSyntaxProgram,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
    nodes: BTreeMap<String, Child>,
    products: BTreeMap<String, OrdinaryJsonProductDefinition>,
    sequences: BTreeMap<String, OrdinaryJsonSequenceDefinition>,
    sums: BTreeMap<String, OrdinaryJsonSumDefinition>,
    collections: BTreeMap<String, OrdinaryJsonCollectionDefinition>,
    key_relations: &'a [OrdinaryRelationDefinition],
    container_join: Option<String>,
    fragments: &'a OrdinaryBoundaryFragmentDefinition,
    syntax_helpers: (&'a str, &'a str),
    chunks: &'a mut BTreeMap<Vec<u8>, String>,
    arm_literals: BTreeMap<Vec<u8>, OrdinaryJsonSyntaxLiteral>,
    active: BTreeSet<String>,
    deferred: BTreeSet<String>,
}
impl Products<'_> {
    fn punctuation(&self, byte: u8) -> R<String> {
        self.syntax
            .literals()
            .iter()
            .find(|d| d.utf8 == [byte])
            .map(|d| d.match_definition.clone())
            .ok_or(OrdinaryCarrierError::Linkage)
    }
    fn container_join_definition(&mut self) -> R<String> {
        if self.container_join.is_none() {
            self.container_join = Some(super::json_grammar::emit_child(self.b, true)?);
        }
        self.container_join
            .clone()
            .ok_or(OrdinaryCarrierError::Linkage)
    }
    fn ty(&mut self, id: &str) -> R<Option<Child>> {
        if let Some(node) = self.nodes.get(id) {
            return Ok(Some(node.clone()));
        }
        if self.deferred.contains(id) {
            return Ok(None);
        }
        if !self.active.insert(id.into()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        if self.vir.data_closed().entries().iter().any(|entry| {
            entry["instance_id"] == id
                && entry["template_id"] == "mpk.csharp.semantic.bounded_sequence.v1"
        }) {
            return self.sequence(id);
        }
        if self.vir.data_closed().entries().iter().any(|entry| {
            entry["instance_id"] == id
                && sums::arm_names(entry["template_id"].as_str().unwrap_or("")).is_some()
        }) {
            return self.sum(id);
        }
        if self.vir.data_closed().entries().iter().any(|entry| {
            entry["instance_id"] == id
                && matches!(
                    entry["template_id"].as_str(),
                    Some(
                        "mpk.csharp.semantic.ordered_map.v1" | "mpk.csharp.semantic.ordered_set.v1"
                    )
                )
        }) {
            return self.collection(id);
        }
        let (_, roots, _) = self.vir.construction_context();
        let schema = if let Some(source) = roots
            .source_types
            .get(id)
            .filter(|s| s.kind != SourceKind::Enum)
        {
            Some((
                source
                    .members
                    .iter()
                    .map(|m| (m.id.clone(), m.name.clone()))
                    .collect::<Vec<_>>(),
                "source_members",
                None,
            ))
        } else {
            self.vir
                .data_closed()
                .entries()
                .iter()
                .find(|e| e["instance_id"] == id)
                .and_then(|entry| {
                    let template = entry["template_id"].as_str()?;
                    let names: &[&str] = match template {
                        "mpk.csharp.semantic.ordered_entry.v1" => &["key", "value"],
                        "mpk.csharp.semantic.money.v1" => &["amount", "currency"],
                        "mpk.csharp.semantic.transition.v1" => &["state", "events", "response"],
                        _ => return None,
                    };
                    Some((
                        names.iter().map(|n| ((*n).into(), (*n).into())).collect(),
                        "semantic_field_names",
                        Some(template.to_owned()),
                    ))
                })
        };
        let Some((source_members, group, template_id)) = schema else {
            self.active.remove(id);
            self.deferred.insert(id.into());
            return Ok(None);
        };
        let carrier = (*self.carriers.get(id).ok_or(OrdinaryCarrierError::Shape)?).clone();
        let OrdinaryShape::Product { fields } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if fields.len() != source_members.len() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let transition = template_id.as_deref() == Some("mpk.csharp.semantic.transition.v1");
        let mut container_payload_member_ids = vec![];
        let mut children = vec![];
        let mut member_names = vec![];
        for (field, (source_id, source_name)) in fields.iter().zip(source_members) {
            if field.id != source_id {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let OrdinaryShape::Reference { type_id } = &field.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            let Some(child) = self.ty(type_id)? else {
                self.active.remove(id);
                self.deferred.insert(id.into());
                return Ok(None);
            };
            if transition {
                let arguments = &self
                    .vir
                    .data_closed()
                    .metadata
                    .get(id)
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .argument_ids;
                if arguments.len() != 3 {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let valid = match field.id.as_str() {
                    "state" => type_id == &arguments[0],
                    "response" => type_id == &arguments[2],
                    "events" => self.sequences.get(type_id).is_some_and(|sequence| {
                        sequence.capacity == 4096 && sequence.element_type_id == arguments[1]
                    }),
                    _ => false,
                };
                if !valid {
                    return Err(OrdinaryCarrierError::Shape);
                }
                if field.id == "events" {
                    container_payload_member_ids.push(field.id.clone());
                }
            }
            let matcher = self
                .syntax
                .fields()
                .iter()
                .find(|f| f.group == group && f.owner == id && f.field_id == source_name)
                .ok_or(OrdinaryCarrierError::Linkage)?
                .match_definition
                .clone();
            children.push((field.id.clone(), child, matcher));
            member_names.push(source_name);
        }
        let storage = super::super::structural::emit(self.b, &carrier, &self.carriers)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let OrdinaryStructuralOperations::Product { operations } = storage.operations else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if operations
            .fields
            .iter()
            .map(|f| &f.field_id)
            .ne(fields.iter().map(|f| &f.id))
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let mut scope = Scope::default();
        let doc = scope.outer(self.b, 3)?;
        let start = scope.outer(self.b, 2)?;
        let depth = scope.outer(self.b, 0)?;
        let length = call(self.b, &self.document.length_definition, vec![doc])?;
        let has_children = truth(self.b, !children.is_empty())?;
        let begin = call(
            self.b,
            &self.grammar.begin_definition,
            vec![length, start, depth, has_children],
        )?;
        let mut head = scope.bind(self.b, 7, begin)?;
        head = syntax_step(
            self.b,
            &mut scope,
            head,
            &self.punctuation(b'{')?,
            self.grammar,
        )?;
        let mut values = vec![];
        for (i, (field_id, child, matcher)) in children.iter().enumerate() {
            if i != 0 {
                head = syntax_step(
                    self.b,
                    &mut scope,
                    head,
                    &self.punctuation(b',')?,
                    self.grammar,
                )?;
            }
            head = syntax_step(self.b, &mut scope, head, matcher, self.grammar)?;
            let doc = scope.outer(self.b, 3)?;
            let current = scope.bound(self.b, head)?;
            let cursor = call(self.b, &self.grammar.cursor_definition, vec![current])?;
            let ending = word(self.b, if i + 1 == children.len() { 3 } else { 1 }, 3)?;
            let mut args = vec![doc, cursor, ending];
            if child.compound {
                let depth = scope.outer(self.b, 0)?;
                args.push(call(
                    self.b,
                    &self.grammar.child_depth_definition,
                    vec![depth],
                )?);
            }
            let parsed = call(self.b, &child.parse, args)?;
            let value = scope.bind(self.b, child.packet_depth, parsed)?;
            let current = scope.bound(self.b, head)?;
            let parsed = scope.bound(self.b, value)?;
            let child_head = call(self.b, &child.header, vec![parsed])?;
            let join = if container_payload_member_ids.contains(field_id) {
                self.container_join_definition()?
            } else {
                self.grammar.child_definition.clone()
            };
            let joined = call(self.b, &join, vec![current, child_head])?;
            head = scope.bind(self.b, 7, joined)?;
            values.push((value, child.value.clone()));
        }
        head = syntax_step(
            self.b,
            &mut scope,
            head,
            &self.punctuation(b'}')?,
            self.grammar,
        )?;
        let doc = scope.outer(self.b, 3)?;
        let header = scope.bound(self.b, head)?;
        let ending = scope.outer(self.b, 1)?;
        let header = call(
            self.b,
            &self.grammar.finish_definition,
            vec![doc, header, ending],
        )?;
        let mut args = vec![];
        for (index, project) in values {
            let value = scope.bound(self.b, index)?;
            args.push(call(self.b, &project, vec![value])?);
        }
        let value = call(self.b, &operations.make_definition, args)?;
        let assemble = super::json_grammar::assemble(self.b, carrier.depth)?;
        let body = call(self.b, &assemble, vec![header, value])?;
        let body = scope.finish(self.b, body)?;
        let hex = id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let parse = format!("Mpk.CSharp.Ordinary.JsonProducts.T{hex}.Parse");
        let packet_depth = carrier.depth.max(7) + 1;
        define(self.b, &parse, &[24, 5, 3, 5], packet_depth, body)?;
        let header = super::json_values::projection(self.b, packet_depth, 7, false)?;
        let value = super::json_values::projection(self.b, packet_depth, carrier.depth, true)?;
        let node = Child {
            parse: parse.clone(),
            header: header.clone(),
            value: value.clone(),
            packet_depth,
            compound: true,
        };
        self.products.insert(
            id.into(),
            OrdinaryJsonProductDefinition {
                carrier,
                template_id,
                parse_definition: parse,
                packet_depth,
                header_definition: header,
                value_definition: value,
                member_ids: children.into_iter().map(|(id, _, _)| id).collect(),
                member_names,
                container_payload_member_ids,
            },
        );
        self.nodes.insert(id.into(), node.clone());
        self.active.remove(id);
        Ok(Some(node))
    }
}

pub fn generate_csharp_practical_ordinary_json_products(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonProductProgram> {
    Ok(generate_with_fields(vir, None, EnvelopeMode::None)?.0)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum EnvelopeMode {
    None,
    Parse,
    TypedDepth,
    TypedLimits,
    RawAndTypedLimits,
}

type JsonGeneration = (
    OrdinaryJsonProductProgram,
    Vec<OrdinaryJsonBoundaryFieldDefinition>,
    Vec<OrdinaryLiteralDefinition>,
    Vec<OrdinaryJsonEnvelopeDefinition>,
    Vec<OrdinaryJsonTypedDepthDefinition>,
    Vec<OrdinaryJsonTypedNodeDefinition>,
    Option<OrdinaryJsonRawLimitsDefinition>,
);

fn generate_with_fields(
    vir: &ValidatedPracticalVir,
    emitted: Option<&EmittedDataPhase>,
    envelope_mode: EnvelopeMode,
) -> R<JsonGeneration> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let mut field_plans = vec![];
    let mut field_definitions = vec![];
    let mut field_values = vec![];
    let mut envelope_definitions = vec![];
    let mut depth_definitions = vec![];
    let mut node_definitions = vec![];
    let mut raw_limits = None;
    if let Some(emitted) = emitted {
        (b, field_plans, field_values) = boundary_fields::prepare(emitted, &layouts, b)?;
    }
    let mut primitives = vec![];
    let mut enums = vec![];
    let mut vocabulary = vec![];
    let mut products = vec![];
    let mut sequences = vec![];
    let mut sums = vec![];
    let mut collections = vec![];
    let mut key_relations = vec![];
    let mut syntax_literals = vec![];
    let mut grammar = None;
    if !boundary.contracts().is_empty() {
        let keys = vir
            .data_closed()
            .metadata
            .values()
            .filter(|m| {
                matches!(
                    m.template_id.as_str(),
                    "mpk.csharp.semantic.ordered_map.v1" | "mpk.csharp.semantic.ordered_set.v1"
                )
            })
            .map(|m| m.argument_ids[0].clone())
            .collect();
        (b, key_relations) =
            super::super::structural::emit_ordered_key_relations(vir, &layouts, b, &keys)?;
        let mut primitive_carriers = layouts.carriers().to_vec();
        primitive_carriers.extend(boundary_fields::codec_carriers(&field_plans, &layouts)?);
        let tokens = super::emit_boundary_json_for_carriers(&mut b, &primitive_carriers)?;
        primitives = super::json_values::emit(&mut b, &tokens, &primitive_carriers)?;
        let syntax = generate_csharp_practical_ordinary_json_syntax(vir)?;
        let (add, finish) = super::json_syntax::helpers(&mut b)?;
        let mut chunks = BTreeMap::new();
        for d in syntax.literals() {
            syntax_literals.push(super::json_syntax::emit_literal(
                &mut b,
                &tokens.strings.fragments,
                &add,
                &finish,
                d.utf8.clone(),
                &mut chunks,
            )?);
        }
        if syntax_literals != syntax.literals() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let g = super::json_grammar::emit(&mut b, &tokens.strings.document)?;
        let mut nodes = BTreeMap::new();
        for d in &primitives {
            if d.scale.is_some() || d.rounding.is_some() || d.codec_id == "guid.d" {
                continue;
            }
            if nodes
                .insert(
                    d.carrier.type_id.clone(),
                    Child {
                        parse: d.parse_definition.clone(),
                        header: d.header_definition.clone(),
                        value: d.value_definition.clone(),
                        packet_depth: d.packet_depth,
                        compound: false,
                    },
                )
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        enums = super::json_enums::emit(&mut b, &tokens, layouts.carriers(), vir)?;
        for d in &enums {
            if nodes
                .insert(
                    d.carrier.type_id.clone(),
                    Child {
                        parse: d.parse_definition.clone(),
                        header: d.header_definition.clone(),
                        value: d.value_definition.clone(),
                        packet_depth: d.packet_depth,
                        compound: false,
                    },
                )
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        vocabulary = super::json_vocabulary::emit(
            &mut b,
            &tokens,
            &g,
            layouts.carriers(),
            (&add, &finish),
            &mut chunks,
        )?;
        for d in &vocabulary {
            if nodes
                .insert(
                    d.carrier.type_id.clone(),
                    Child {
                        parse: d.parse_definition.clone(),
                        header: d.header_definition.clone(),
                        value: d.value_definition.clone(),
                        packet_depth: d.packet_depth,
                        compound: false,
                    },
                )
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        let mut compiler = Products {
            vir,
            b: &mut b,
            document: &tokens.strings.document,
            grammar: &g,
            syntax: &syntax,
            carriers: layouts
                .carriers()
                .iter()
                .map(|c| (c.type_id.as_str(), c))
                .collect(),
            nodes,
            products: BTreeMap::new(),
            sequences: BTreeMap::new(),
            sums: BTreeMap::new(),
            collections: BTreeMap::new(),
            key_relations: &key_relations,
            container_join: None,
            fragments: &tokens.strings.fragments,
            syntax_helpers: (&add, &finish),
            chunks: &mut chunks,
            arm_literals: BTreeMap::new(),
            active: BTreeSet::new(),
            deferred: BTreeSet::new(),
        };
        for carrier in layouts.carriers() {
            compiler.ty(&carrier.type_id)?;
        }
        for plan in field_plans {
            field_definitions.push(compiler.boundary_field(plan, &primitives)?);
        }
        if envelope_mode != EnvelopeMode::None {
            envelope_definitions = compiler.envelopes(
                emitted.ok_or(OrdinaryCarrierError::Linkage)?,
                &field_definitions,
            )?;
        }
        if matches!(
            envelope_mode,
            EnvelopeMode::TypedDepth | EnvelopeMode::TypedLimits | EnvelopeMode::RawAndTypedLimits
        ) {
            depth_definitions = typed_depth::emit(
                emitted.ok_or(OrdinaryCarrierError::Linkage)?,
                &layouts,
                compiler.b,
            )?;
            envelopes::depth_guards(compiler.b, &mut envelope_definitions, &depth_definitions)?;
        }
        if matches!(
            envelope_mode,
            EnvelopeMode::TypedLimits | EnvelopeMode::RawAndTypedLimits
        ) {
            node_definitions = typed_nodes::emit(
                emitted.ok_or(OrdinaryCarrierError::Linkage)?,
                &layouts,
                compiler.b,
            )?;
            envelopes::node_guards(compiler.b, &mut envelope_definitions, &node_definitions)?;
        }
        if envelope_mode == EnvelopeMode::RawAndTypedLimits {
            let raw =
                super::json_raw_limits::emit_with_document(compiler.b, compiler.document.clone())?;
            envelopes::raw_guards(compiler.b, &mut envelope_definitions, &raw)?;
            raw_limits = Some(raw);
        }
        products = compiler.products.into_values().collect();
        sequences = compiler.sequences.into_values().collect();
        sums = compiler.sums.into_values().collect();
        collections = compiler.collections.into_values().collect();
        grammar = Some(g);
    }
    let defined = primitives
        .iter()
        .map(|d| d.carrier.type_id.as_str())
        .chain(products.iter().map(|d| d.carrier.type_id.as_str()))
        .chain(sequences.iter().map(|d| d.carrier.type_id.as_str()))
        .chain(sums.iter().map(|d| d.carrier.type_id.as_str()))
        .chain(
            collections
                .iter()
                .map(|d| d.sequence.carrier.type_id.as_str()),
        )
        .chain(enums.iter().map(|d| d.carrier.type_id.as_str()))
        .chain(vocabulary.iter().map(|d| d.carrier.type_id.as_str()))
        .collect::<BTreeSet<_>>();
    let deferred_type_ids = layouts
        .carriers()
        .iter()
        .filter(|c| !defined.contains(c.type_id.as_str()))
        .map(|c| c.type_id.clone())
        .collect();
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryJsonProductProgram {
        schema: "mpk.csharp.ordinary_json_products.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        primitives,
        enums,
        vocabulary,
        products,
        sequences,
        sums,
        collections,
        key_relations,
        syntax_literals,
        grammar,
        deferred_type_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok((
        p,
        field_definitions,
        field_values,
        envelope_definitions,
        depth_definitions,
        node_definitions,
        raw_limits,
    ))
}
pub fn import_csharp_practical_ordinary_json_products(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonProductProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_products(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[path = "csharp_practical_ordinary_json_typed_depth.rs"]
mod typed_depth;
pub use typed_depth::{
    generate_csharp_practical_ordinary_json_typed_depth,
    import_csharp_practical_ordinary_json_typed_depth, OrdinaryJsonTypedDepthDefinition,
    OrdinaryJsonTypedDepthProgram,
};

pub use envelopes::{
    generate_csharp_practical_ordinary_json_depth_guarded_envelopes,
    import_csharp_practical_ordinary_json_depth_guarded_envelopes,
};

#[path = "csharp_practical_ordinary_json_typed_nodes.rs"]
mod typed_nodes;

pub use typed_nodes::{
    generate_csharp_practical_ordinary_json_typed_nodes,
    import_csharp_practical_ordinary_json_typed_nodes, OrdinaryJsonTypedNodeDefinition,
    OrdinaryJsonTypedNodeProgram,
};

pub use envelopes::{
    generate_csharp_practical_ordinary_json_typed_guarded_envelopes,
    import_csharp_practical_ordinary_json_typed_guarded_envelopes,
};

pub use envelopes::{
    generate_csharp_practical_ordinary_json_limits_guarded_envelopes,
    import_csharp_practical_ordinary_json_limits_guarded_envelopes,
};
