//! Source public-clause bodies in ordinary core. Representation/recursive public
//! domains, partial-operation obligations and application proofs are separate.
use super::*;
#[path = "csharp_practical_ordinary_quantifiers.rs"]
mod quantifiers;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceClauseDefinition {
    pub source_type_id: String,
    pub source_sha256: String,
    pub attachment_sha256: String,
    pub expression_sha256: String,
    pub argument_types: Vec<String>,
    pub definition: String,
    /// Separate W03 condition, to be proved at the original contract use point.
    /// The value definition is meaningful only where this predicate holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definedness_definition: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceClauseProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    construction_sha256: String,
    definitions: Vec<OrdinarySourceClauseDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinarySourceClauseProgram {
    pub fn definitions(&self) -> &[OrdinarySourceClauseDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("source clause program")
    }
}
const SOURCE_BOOL: &str = "mpk.csharp.value.bool.v1";
fn core_definition_name(name: &str) -> String {
    format!(
        "{PREFIX}.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    )
}
fn simple_literal_type(id: &str) -> bool {
    id.strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
        .is_some_and(|s| {
            matches!(
                s,
                "bool" | "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64"
            )
        })
}
struct Clauses<'a> {
    vir: Option<&'a ValidatedPracticalVir>,
    relations: relations::ContractRelationCache,
    b: Builder,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
    storage: StorageCache,
    constants: BTreeMap<String, (String, String)>,
    scalars: BTreeMap<String, (OrdinaryScalarDefinition, Vec<String>)>,
    strings: super::super::scalar_bits::ContractStringCache,
    finite: Option<Vec<OrdinaryFiniteOperation>>,
    codecs: super::super::scalar_bits::ContractCodecCache,
    binding_projections: Option<Vec<OrdinaryBindingProjectionDefinition>>,
    quantifiers: BTreeMap<String, quantifiers::QuantifierFunctions>,
}
fn signature(args: &[String], result: &str) -> String {
    args.iter()
        .rev()
        .fold(result.to_owned(), |r, a| format!("({a}->{r})"))
}
fn arrow(ty: &str) -> R<(&str, &str)> {
    let inner = ty
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or(OrdinaryCarrierError::Shape)?;
    let mut depth = 0usize;
    for (i, ch) in inner.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1).ok_or(OrdinaryCarrierError::Shape)?,
            '-' if depth == 0 && inner[i..].starts_with("->") => {
                let (a, r) = (&inner[..i], &inner[i + 2..]);
                if a.is_empty() || r.is_empty() {
                    return Err(OrdinaryCarrierError::Shape);
                }
                return Ok((a, r));
            }
            _ => {}
        }
    }
    Err(OrdinaryCarrierError::Shape)
}
impl Clauses<'_> {
    fn string_operation(&mut self, d: &ContractDefinition, op: &str) -> R<u32> {
        let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
        let expected = strings::operation_signature(vir.data_closed(), op)
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
        if expected.argument_type_ids != d.argument_types
            || expected.normal_result_type_id != d.result_type
            || expected.ordered_checks != d.ordered_checks
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let nullable = strings::operation_signature(vir.data_closed(), "string.length")
            .map_err(|_| OrdinaryCarrierError::Linkage)?
            .argument_type_ids[0]
            != STRING_TYPE_ID;
        let (definition, checks) = self.strings.emit(&mut self.b, expected, nullable)?;
        if checks.len() != d.ordered_checks.len() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        for (check, predicate) in d.ordered_checks.iter().zip(checks) {
            if !matches!(
                check.tag,
                RequiredCheckTag::Exception | RequiredCheckTag::StaticObligation
            ) {
                return Err(OrdinaryCarrierError::Linkage);
            }
            self.constants.insert(
                format!("Mpk.CSharp.Data.ContractFails.{}.{}", d.name, check.id),
                (signature(&d.argument_types, SOURCE_BOOL), predicate),
            );
        }
        self.b.constant(&definition.result_definition)
    }
    fn definedness_logic(&mut self) -> R<()> {
        for (source, core, arity) in [
            ("true", "true", 0),
            ("false", "false", 0),
            ("Not", "not", 1),
            ("And", "and", 2),
            ("Or", "or", 2),
        ] {
            let name = format!("Mpk.CSharp.Bool.{source}");
            let entry = (
                signature(&vec![SOURCE_BOOL.into(); arity], SOURCE_BOOL),
                format!("Std.Bool.{core}"),
            );
            if self
                .constants
                .insert(name, entry.clone())
                .is_some_and(|old| old != entry)
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        Ok(())
    }
    fn relation_function(
        &mut self,
        type_id: &str,
        operation: relations::ContractRelationOperation,
    ) -> R<u32> {
        let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
        // The generator propagates failures without publishing partial output;
        // preserve the builder/storage owner on either result.
        let builder = std::mem::replace(&mut self.b, Builder::new()?);
        let storage = std::mem::take(&mut self.storage);
        let (builder, storage, result) = self
            .relations
            .emit(vir, builder, storage, type_id, operation);
        self.b = builder;
        self.storage = storage;
        self.b.constant(&result?)
    }
    fn ty(&mut self, ty: &str, nesting: usize) -> R<u32> {
        if nesting > 256 {
            return Err(OrdinaryCarrierError::Limit);
        }
        if let Some(carrier) = self.carriers.get(ty) {
            return self.b.cube(carrier.depth);
        }
        let (a, r) = arrow(ty)?;
        let a = self.ty(a, nesting + 1)?;
        let r = self.ty(r, nesting + 1)?;
        self.b.pi(a, r)
    }
    fn lower(&mut self, term: &ContractTerm, env: &mut Vec<String>, nesting: usize) -> R<u32> {
        if nesting > 256 || env.len() > 256 {
            return Err(OrdinaryCarrierError::Limit);
        }
        self.ty(term.type_id(), 0)?;
        match term {
            ContractTerm::Var { index, type_id } => {
                if env.iter().rev().nth(*index) != Some(type_id) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.b.var(*index as u32)
            }
            ContractTerm::Const { name, type_id } => {
                let (registered_type, core_name) = self
                    .constants
                    .get(name)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if registered_type != type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.b.constant(core_name)
            }
            ContractTerm::App {
                function,
                argument,
                type_id,
            } => {
                let (a, r) = arrow(function.type_id())?;
                if a != argument.type_id() || r != type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let f = self.lower(function, env, nesting + 1)?;
                let a = self.lower(argument, env, nesting + 1)?;
                self.b.app(f, vec![a])
            }
            ContractTerm::Lam {
                parameter_type,
                body,
                type_id,
            } => {
                if type_id != &format!("({parameter_type}->{})", body.type_id()) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let ty = self.ty(parameter_type, 0)?;
                env.push(parameter_type.clone());
                let body = self.lower(body, env, nesting + 1);
                env.pop();
                self.b.lam(ty, body?)
            }
            ContractTerm::Let {
                value,
                body,
                type_id,
            } => {
                if type_id != body.type_id() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let ty = self.ty(value.type_id(), 0)?;
                let lowered = self.lower(value, env, nesting + 1)?;
                env.push(value.type_id().to_owned());
                let body = self.lower(body, env, nesting + 1);
                env.pop();
                self.b.term(TermNode::Let {
                    ty,
                    value: lowered,
                    body: body?,
                })
            }
        }
    }
    fn recipe(&mut self, d: &ContractDefinition) -> R<()> {
        let ty = signature(&d.argument_types, &d.result_type);
        if let Some((old, _)) = self.constants.get(&d.name) {
            return if old == &ty {
                Ok(())
            } else {
                Err(OrdinaryCarrierError::Linkage)
            };
        }
        // Explicit ordered checks are lowered by scalar unary/binary adapters.
        // Index and payload recipes register their implicit conditions below.
        if !d.ordered_checks.is_empty() && !matches!(d.tag.as_str(), "binary" | "unary") {
            return Err(OrdinaryCarrierError::Shape);
        }
        let params: Value =
            serde_json::from_str(&d.parameters).map_err(|_| OrdinaryCarrierError::Linkage)?;
        let body = match d.tag.as_str() {
            "codec_parse" | "codec_format" => {
                let parse = d.tag == "codec_parse";
                let fields = params.as_object().ok_or(OrdinaryCarrierError::Linkage)?;
                if fields.len() != if parse { 2 } else { 3 } || d.argument_types.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let id = fields
                    .get("codec_id")
                    .and_then(Value::as_str)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let configuration = fields
                    .get("codec_parameters")
                    .and_then(Value::as_object)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if configuration.len() != 2
                    || !configuration.contains_key("scale")
                    || !configuration.contains_key("rounding")
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                // Recreate the fixed parameter record order after parsing the
                // recipe transport; the artifact itself was validated by W14.
                let transport = format!(
                    "{{\"scale\":{},\"rounding\":{}}}",
                    configuration["scale"], configuration["rounding"]
                );
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let value_type = if parse {
                    let result = vir
                        .data_closed()
                        .metadata
                        .get(&d.result_type)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    if template_name(&result.template_id) != Some("result")
                        || result.argument_ids.len() != 2
                        || result.argument_ids[1] != "mpk.csharp.value.parse_error.v1"
                        || d.argument_types[0] != "mpk.csharp.value.string.v1"
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    result.argument_ids[0].clone()
                } else {
                    if d.result_type != "mpk.csharp.value.string.v1" {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    d.argument_types[0].clone()
                };
                let codec = BoundaryCodec::from_contract_parameters_json(
                    id,
                    &value_type,
                    transport.as_bytes(),
                )
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
                if !parse {
                    codec
                        .validate_contract_format_mode(
                            fields
                                .get("mode")
                                .and_then(Value::as_str)
                                .ok_or(OrdinaryCarrierError::Linkage)?,
                        )
                        .map_err(|_| OrdinaryCarrierError::Linkage)?;
                }
                let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
                let (function, shape) =
                    self.codecs
                        .emit(&mut self.b, &layouts, vir, &codec, parse)?;
                if let Some(shape) = shape {
                    if self.carriers[d.result_type.as_str()].shape != shape {
                        return Err(OrdinaryCarrierError::Shape);
                    }
                } else {
                    let symbol = format!("Mpk.CSharp.Data.ContractDefined.{}", d.name);
                    let name = core_definition_name(&symbol);
                    super::super::scalar_bits::ContractCodecCache::format_defined(
                        &mut self.b,
                        &function,
                        self.carriers[value_type.as_str()].depth,
                        &name,
                    )?;
                    self.constants
                        .insert(symbol, (signature(&d.argument_types, SOURCE_BOOL), name));
                }
                self.b.constant(&function)?
            }
            "source_project" => {
                let fields = params.as_object().ok_or(OrdinaryCarrierError::Linkage)?;
                if fields.len() != 1 || d.argument_types.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let binding_id = fields
                    .get("binding_id")
                    .and_then(Value::as_str)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if self.binding_projections.is_none() {
                    let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                    let construction = generate_construction_vcs(vir)
                        .map_err(|_| OrdinaryCarrierError::Linkage)?;
                    let vc = generate_binding_vcs(vir, &construction)
                        .map_err(|_| OrdinaryCarrierError::Linkage)?;
                    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
                    let builder = std::mem::replace(&mut self.b, Builder::new()?);
                    let (builder, projections) =
                        binding_projections::emit_binding_projections(vir, &vc, &layouts, builder)?;
                    self.b = builder;
                    self.binding_projections = Some(projections);
                }
                let matches = self
                    .binding_projections
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|p| {
                        p.projection.binding_id == binding_id
                            && p.source_carrier.type_id == d.argument_types[0]
                            && p.semantic_carrier.type_id == d.result_type
                    })
                    .collect::<Vec<_>>();
                if matches.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.b.constant(&matches[0].project_definition)?
            }
            "map_contains" | "map_lookup" | "set_contains" => {
                if params != serde_json::json!({}) || d.argument_types.len() != 2 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let metadata = vir
                    .data_closed()
                    .metadata
                    .get(&d.argument_types[0])
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let map = d.tag != "set_contains";
                if template_name(&metadata.template_id)
                    != Some(if map { "ordered_map" } else { "ordered_set" })
                    || metadata.argument_ids.len() != if map { 2 } else { 1 }
                    || metadata.argument_ids[0] != d.argument_types[1]
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let lookup = d.tag == "map_lookup";
                if lookup {
                    let result = vir
                        .data_closed()
                        .metadata
                        .get(&d.result_type)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    if template_name(&result.template_id) != Some("lookup")
                        || result.argument_ids != metadata.argument_ids[1..2]
                        || !metadata.dependency_ids.contains(&d.result_type)
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                } else if d.result_type != SOURCE_BOOL {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.relation_function(
                    &d.argument_types[0],
                    if lookup {
                        relations::ContractRelationOperation::MapLookup
                    } else {
                        relations::ContractRelationOperation::CollectionContains
                    },
                )?
            }
            "bounded_forall" | "bounded_exists" => quantifiers::recipe(self, d, &params)?,
            "structural_equal" | "structural_compare" => {
                let compare = d.tag == "structural_compare";
                if params != serde_json::json!({})
                    || d.argument_types.len() != 2
                    || d.argument_types[0] != d.argument_types[1]
                    || d.result_type
                        != if compare {
                            "mpk.csharp.value.i32.v1"
                        } else {
                            SOURCE_BOOL
                        }
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.relation_function(
                    &d.argument_types[0],
                    if compare {
                        relations::ContractRelationOperation::Compare
                    } else {
                        relations::ContractRelationOperation::Equal
                    },
                )?
            }
            "sequence_length" => {
                if params != serde_json::json!({})
                    || d.argument_types.len() != 1
                    || d.result_type != "mpk.csharp.value.i32.v1"
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                // Valid sequence lengths fit i32. Preserve the complete stored
                // word; domain validation supplies the physical bound.
                self.relation_function(
                    &d.argument_types[0],
                    relations::ContractRelationOperation::SequenceLength,
                )?
            }
            "sequence_index" => {
                if params != serde_json::json!({})
                    || d.argument_types.len() != 2
                    || d.argument_types[1] != "mpk.csharp.value.i32.v1"
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let metadata = vir
                    .data_closed()
                    .metadata
                    .get(&d.argument_types[0])
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if template_name(&metadata.template_id) != Some("bounded_sequence")
                    || metadata.argument_ids != [d.result_type.clone()]
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let failed = self.relation_function(
                    &d.argument_types[0],
                    relations::ContractRelationOperation::SequenceIndexRange,
                )?;
                let sequence = self.b.var(1)?;
                let index = self.b.var(0)?;
                let failed = self.b.app(failed, vec![sequence, index])?;
                let valid = call(&mut self.b, "Std.Bool.not", vec![failed])?;
                let symbol = format!("Mpk.CSharp.Data.ContractDefined.{}", d.name);
                let name = core_definition_name(&symbol);
                let sequence_depth = self.carriers[d.argument_types[0].as_str()].depth;
                define(&mut self.b, &name, &[sequence_depth, 5], 0, valid)?;
                self.constants
                    .insert(symbol, (signature(&d.argument_types, SOURCE_BOOL), name));
                self.relation_function(
                    &d.argument_types[0],
                    relations::ContractRelationOperation::SequenceRead,
                )?
            }
            "transition_state" | "transition_events" | "transition_response" => {
                if params != serde_json::json!({}) || d.argument_types.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let metadata = vir
                    .data_closed()
                    .metadata
                    .get(&d.argument_types[0])
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if template_name(&metadata.template_id) != Some("transition")
                    || metadata.argument_ids.len() != 3
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let field_id = match d.tag.as_str() {
                    "transition_state" => "state",
                    "transition_events" => "events",
                    _ => "response",
                };
                let carrier = self
                    .carriers
                    .get(d.argument_types[0].as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let OrdinaryShape::Product { fields } = &carrier.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let field = fields
                    .iter()
                    .find(|f| f.id == field_id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let shape = match &field.shape {
                    OrdinaryShape::RoleBound { value, .. } => value.as_ref(),
                    shape => shape,
                };
                if shape
                    != &(OrdinaryShape::Reference {
                        type_id: d.result_type.clone(),
                    })
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let structure = self
                    .storage
                    .get(&mut self.b, carrier, &self.carriers)?
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let OrdinaryStructuralOperations::Product { operations } = structure.operations
                else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let get = operations
                    .fields
                    .iter()
                    .find(|f| f.field_id == field_id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                self.b.constant(&get.definition)?
            }
            "tagged_make" => {
                let arm = params["arm"]
                    .as_str()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let instance = params["semantic_instance_id"]
                    .as_str()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let expected = if d.argument_types.is_empty() {
                    serde_json::json!({"semantic_instance_id":instance,"arm":arm,"payload":null})
                } else {
                    serde_json::json!({"semantic_instance_id":instance,"arm":arm})
                };
                if params != expected || instance != d.result_type || d.argument_types.len() > 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let metadata = vir
                    .data_closed()
                    .metadata
                    .get(instance)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if !matches!(
                    template_name(&metadata.template_id),
                    Some("option" | "lookup" | "result" | "validation" | "boundary_field")
                ) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let carrier = self
                    .carriers
                    .get(instance)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let OrdinaryShape::Sum { arms } = &carrier.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let shape = arms
                    .iter()
                    .find(|a| a.id == arm)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if shape.fields.len() != d.argument_types.len() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                for (field, ty) in shape.fields.iter().zip(&d.argument_types) {
                    let shape = match &field.shape {
                        OrdinaryShape::RoleBound { value, .. } => value.as_ref(),
                        shape => shape,
                    };
                    if shape
                        != &(OrdinaryShape::Reference {
                            type_id: ty.clone(),
                        })
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                }
                let structure = self
                    .storage
                    .get(&mut self.b, carrier, &self.carriers)?
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let OrdinaryStructuralOperations::Sum { arms, .. } = structure.operations else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let operation = arms
                    .iter()
                    .find(|a| a.arm_id == arm)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                self.b.constant(&operation.make_definition)?
            }
            "tagged_is" | "tagged_payload" => {
                let payload = d.tag == "tagged_payload";
                if d.argument_types.len() != 1 || (!payload && d.result_type != SOURCE_BOOL) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let arm = params["arm"]
                    .as_str()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if params != serde_json::json!({"arm":arm}) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let vir = self.vir.ok_or(OrdinaryCarrierError::Linkage)?;
                let metadata = vir
                    .data_closed()
                    .metadata
                    .get(&d.argument_types[0])
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if !matches!(
                    template_name(&metadata.template_id),
                    Some("option" | "lookup" | "result" | "validation" | "boundary_field")
                ) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let carrier = self
                    .carriers
                    .get(d.argument_types[0].as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let structure = self
                    .storage
                    .get(&mut self.b, carrier, &self.carriers)?
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let OrdinaryStructuralOperations::Sum { arms, .. } = structure.operations else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let operation = arms
                    .iter()
                    .find(|a| a.arm_id == arm)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if payload {
                    let OrdinaryShape::Sum { arms } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    let arm = arms
                        .iter()
                        .find(|a| a.id == arm)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    if arm.fields.len() != 1 || operation.fields.len() != 1 {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    // Validation's error sequence carries a role bound, while
                    // its payload type remains the same concrete sequence type.
                    let shape = match &arm.fields[0].shape {
                        OrdinaryShape::RoleBound { value, .. } => value.as_ref(),
                        shape => shape,
                    };
                    if shape
                        != &(OrdinaryShape::Reference {
                            type_id: d.result_type.clone(),
                        })
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    self.constants.insert(
                        format!("Mpk.CSharp.Data.ContractDefined.{}", d.name),
                        (
                            signature(&d.argument_types, SOURCE_BOOL),
                            operation.is_active_definition.clone(),
                        ),
                    );
                    self.b.constant(&operation.fields[0].definition)?
                } else {
                    self.b.constant(&operation.is_active_definition)?
                }
            }
            "exception_is" | "exception_payload" => {
                let payload = d.tag == "exception_payload";
                if d.argument_types != [EXCEPTION_TYPE_ID]
                    || (!payload && d.result_type != SOURCE_BOOL)
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let key = if payload {
                    "member_id"
                } else {
                    "exception_type_id"
                };
                let specialization = params[key].as_str().ok_or(OrdinaryCarrierError::Linkage)?;
                if params != serde_json::json!({key:specialization}) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                if self.finite.is_none() {
                    self.finite = Some(non_templates::emit_finite(
                        self.vir.ok_or(OrdinaryCarrierError::Linkage)?,
                        &mut self.b,
                        &self.carriers,
                        &mut self.storage,
                    )?);
                }
                let operation_id = format!(
                    "{EXCEPTION_TYPE_ID}.{}",
                    if payload { "payload" } else { "is_type" }
                );
                let operation = self
                    .finite
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .iter()
                    .find(|o| {
                        o.operation_id == operation_id
                            && o.specialization.as_deref() == Some(specialization)
                    })
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if operation.argument_type_ids != d.argument_types
                    || operation.result_type_id != d.result_type
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                if payload {
                    let tag = operation
                        .active_tag_requirement
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let carrier = self
                        .carriers
                        .get(EXCEPTION_TYPE_ID)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let structure = self
                        .storage
                        .get(&mut self.b, carrier, &self.carriers)?
                        .ok_or(OrdinaryCarrierError::Shape)?;
                    let OrdinaryStructuralOperations::Sum { arms, .. } = structure.operations
                    else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    let arm = arms
                        .iter()
                        .find(|a| a.tag == tag)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    // Payload access requires its exact source arm, not an
                    // ancestor match. Inactive zero storage is not a value.
                    self.constants.insert(
                        format!("Mpk.CSharp.Data.ContractDefined.{}", d.name),
                        (
                            signature(&d.argument_types, SOURCE_BOOL),
                            arm.is_active_definition.clone(),
                        ),
                    );
                } else if operation.active_tag_requirement.is_some() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                self.b.constant(&operation.definition)?
            }
            "parse_error_kind" => {
                if params != serde_json::json!({})
                    || d.argument_types != ["mpk.csharp.value.parse_error.v1"]
                    || d.result_type != "mpk.csharp.value.u32.v1"
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let carrier = self
                    .carriers
                    .get(d.argument_types[0].as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if carrier.depth != 5 || carrier.shape != (OrdinaryShape::Bits { width: 32 }) {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let ty = self.b.cube(5)?;
                let body = self.b.var(0)?;
                self.b.lam(ty, body)?
            }
            "conditional" => {
                if params != serde_json::json!({})
                    || d.argument_types
                        != [
                            SOURCE_BOOL.to_owned(),
                            d.result_type.clone(),
                            d.result_type.clone(),
                        ]
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let carrier = self
                    .carriers
                    .get(d.result_type.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let depth = carrier.depth;
                // The frozen expression importer orders condition, when_true,
                // when_false. Select leaves rather than eliminating into a
                // function carrier, preserving the ordinary Bool foundation.
                let selectors = self.b.selectors(depth)?;
                let condition = self.b.var(depth + 2)?;
                let yes = self.b.var(depth + 1)?;
                let yes = self.b.app(yes, selectors.clone())?;
                let no = self.b.var(depth)?;
                let no = self.b.app(no, selectors)?;
                let leaf = mux(&mut self.b, condition, yes, no)?;
                let mut body = self.b.wrap_selectors(depth, leaf)?;
                let ty = self.b.cube(depth)?;
                body = self.b.lam(ty, body)?;
                body = self.b.lam(ty, body)?;
                self.b.lam(self.b.boolean, body)?
            }
            "binary" | "unary" => {
                let op = params["operation_id"]
                    .as_str()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if op.starts_with("string.") {
                    let body = self.string_operation(d, op)?;
                    let core_type = self.ty(&ty, 0)?;
                    let core_name = core_definition_name(&d.name);
                    self.b.define(&core_name, core_type, body)?;
                    self.constants.insert(d.name.clone(), (ty, core_name));
                    return Ok(());
                }
                if !self.scalars.contains_key(op) {
                    let emitted = if op.starts_with("decimal.") {
                        super::super::scalar_bits::emit_decimal_contract(&mut self.b, op)?
                    } else {
                        let scalar = if op.starts_with("floating.")
                            || op.starts_with("numeric.conversion.")
                        {
                            super::super::scalar_bits::emit_floating(&mut self.b, op)?
                        } else if ["date.", "guid.", "day_of_week."]
                            .iter()
                            .any(|prefix| op.starts_with(prefix))
                        {
                            super::super::scalar_bits::emit_calendar(&mut self.b, op)?
                        } else if ["time.", "duration.", "instant."]
                            .iter()
                            .any(|prefix| op.starts_with(prefix))
                        {
                            super::super::scalar_bits::emit_temporal(&mut self.b, op)?
                        } else {
                            super::super::scalar_bits::emit_integer(&mut self.b, op)?
                        };
                        let checks = scalar.ordered_failure_definitions.clone();
                        (scalar, checks)
                    };
                    self.scalars.insert(op.into(), emitted);
                }
                let (scalar, failure_predicates) = &self.scalars[op];
                if scalar.operation.argument_type_ids != d.argument_types
                    || scalar.operation.normal_result_type_id != d.result_type
                    || scalar.operation.ordered_checks != d.ordered_checks
                    || scalar.ordered_failure_definitions.len() != d.ordered_checks.len()
                    || failure_predicates.len() != d.ordered_checks.len()
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                // Integer divide/remainder can fail for zero or min/-1; these
                // predicates are disjoint. Other integer operations have only
                // overflow. Floating arithmetic has no exception checks and
                // checked float-to-integer conversions have only overflow.
                // Decimal arithmetic explicitly exports raw conditions;
                // division may set both zero and overflow. Decimal rounding
                // and conversions have at most one range/overflow check.
                // Calendar/Time/Duration operations have at most one exception
                // check. Instant error-outcome operations still require their
                // outcome adapter and reject below; an error is not a value.
                for (check, core) in d.ordered_checks.iter().zip(failure_predicates) {
                    if check.tag != RequiredCheckTag::Exception
                        || !matches!(
                            check.id.as_str(),
                            "exception.overflow" | "exception.division_by_zero" | "exception.range"
                        )
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    self.constants.insert(
                        format!("Mpk.CSharp.Data.ContractFails.{}.{}", d.name, check.id),
                        (signature(&d.argument_types, SOURCE_BOOL), core.clone()),
                    );
                }
                self.b.constant(&scalar.result_definition)?
            }
            "field" => {
                if d.argument_types.len() != 1 {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let source = self
                    .carriers
                    .get(d.argument_types[0].as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let member = params["member_id"]
                    .as_str()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let OrdinaryShape::Product { fields } = &source.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let field = fields
                    .iter()
                    .find(|f| f.id == member)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if field.shape
                    != (OrdinaryShape::Reference {
                        type_id: d.result_type.clone(),
                    })
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let structure = self
                    .storage
                    .get(&mut self.b, source, &self.carriers)?
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let OrdinaryStructuralOperations::Product { operations } = structure.operations
                else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let get = operations
                    .fields
                    .iter()
                    .find(|f| f.field_id == member)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                self.b.constant(&get.definition)?
            }
            "literal" => {
                if !d.argument_types.is_empty() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let carrier = self
                    .carriers
                    .get(d.result_type.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let OrdinaryShape::Bits { width } = carrier.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let number = if d.result_type == SOURCE_BOOL {
                    u128::from(
                        params["value"]
                            .as_bool()
                            .ok_or(OrdinaryCarrierError::Linkage)?,
                    )
                } else {
                    let suffix = d
                        .result_type
                        .strip_prefix("mpk.csharp.value.")
                        .and_then(|s| s.strip_suffix(".v1"))
                        .ok_or(OrdinaryCarrierError::Shape)?;
                    if !["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64"].contains(&suffix) {
                        return Err(OrdinaryCarrierError::Shape);
                    }
                    let text = params["value"]
                        .as_str()
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    if suffix.starts_with('i') {
                        let n = text
                            .parse::<i128>()
                            .map_err(|_| OrdinaryCarrierError::Shape)?;
                        if n < -(1i128 << (width - 1)) || n >= 1i128 << (width - 1) {
                            return Err(OrdinaryCarrierError::Shape);
                        }
                        n as u128
                    } else {
                        let n = text
                            .parse::<u128>()
                            .map_err(|_| OrdinaryCarrierError::Shape)?;
                        if n >= 1u128 << width {
                            return Err(OrdinaryCarrierError::Shape);
                        }
                        n
                    }
                };
                let mut bits = (0..1 << carrier.depth)
                    .map(|i| bit(&mut self.b, i < width && number & (1u128 << i) != 0))
                    .collect::<R<Vec<_>>>()?;
                for level in 0..carrier.depth {
                    let selector = self.b.var(carrier.depth - 1 - level)?;
                    bits = bits
                        .chunks_exact(2)
                        .map(|p| mux(&mut self.b, selector, p[1], p[0]))
                        .collect::<R<Vec<_>>>()?;
                }
                self.b.wrap_selectors(carrier.depth, bits[0])?
            }
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        let core_type = self.ty(&ty, 0)?;
        // Contract recipe hashes may begin with a digit; ordinary global-name
        // components must begin with an identifier character. Keep the source
        // symbol as the nominal lookup key and encode it into a legal core name.
        let core_name = core_definition_name(&d.name);
        self.b.define(&core_name, core_type, body)?;
        self.constants.insert(d.name.clone(), (ty, core_name));
        Ok(())
    }
}
fn compiler<'a>(
    vir: &'a ValidatedPracticalVir,
    layouts: &'a OrdinaryCarrierProgram,
    builder: Builder,
    expressions: &[&VerifiedContractExpression],
) -> R<Clauses<'a>> {
    // Decode rich literals from their canonical UTF-16-preserving parameters.
    // Emit them together so repeated product/sequence parts share one context.
    // Retain existing Bool/integer bodies and their previously checked bytes.
    let mut literal_values = BTreeMap::new();
    let mut constants = BTreeMap::new();
    for expression in expressions {
        quantifiers::validate_static_ranges(expression)?;
        for d in expression.definitions() {
            if d.tag != "literal" || simple_literal_type(&d.result_type) {
                continue;
            }
            if !d.argument_types.is_empty() || !d.ordered_checks.is_empty() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            use crate::csharp_practical_source_artifacts as a;
            let params = a::parse_canonical_practical_json(
                a::PracticalArtifactKind::MethodContract,
                d.parameters.as_bytes(),
            )
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
            let fields = params.as_object().ok_or(OrdinaryCarrierError::Linkage)?;
            if fields.len() != 1 || fields[0].0 != "value" {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let (bundle, roots, _) = vir.construction_context();
            let value = crate::csharp_practical_vir_model::data_phase::decode_contract_value(
                bundle,
                roots,
                vir.data_closed(),
                &d.result_type,
                &fields[0].1,
            )
            .map_err(|_| OrdinaryCarrierError::Shape)?;
            if value.type_id() != d.result_type {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let name = core_definition_name(&d.name);
            if literal_values
                .insert(name.clone(), value.clone())
                .is_some_and(|old| old != value)
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            constants.insert(d.name.clone(), (d.result_type.clone(), name));
        }
    }
    let (builder, _) = literals::emit_named_values(vir, layouts, builder, literal_values)?;
    Ok(Clauses {
        vir: Some(vir),
        relations: relations::ContractRelationCache::default(),
        b: builder,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
        storage: StorageCache::default(),
        constants,
        scalars: BTreeMap::new(),
        strings: Default::default(),
        finite: None,
        codecs: Default::default(),
        binding_projections: None,
        quantifiers: BTreeMap::new(),
    })
}
pub(super) fn emit_clauses(
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
    builder: Builder,
) -> R<(
    Builder,
    StorageCache,
    relations::ContractRelationCache,
    Vec<OrdinarySourceClauseDefinition>,
    String,
)> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let expressions = construction
        .types()
        .iter()
        .flat_map(|source| source.public_clauses.iter())
        .collect::<Vec<_>>();
    let mut scopes: BTreeMap<&str, BTreeSet<Vec<String>>> = BTreeMap::new();
    for expression in &expressions {
        scopes
            .entry(expression.expression_sha256())
            .or_default()
            .insert(
                expression
                    .subjects()
                    .iter()
                    .map(|(_, ty)| ty.clone())
                    .collect(),
            );
    }
    let mut c = compiler(vir, layouts, builder, &expressions)?;
    let mut definitions = vec![];
    for source in construction.types() {
        for expression in &source.public_clauses {
            for d in expression.definitions() {
                c.recipe(d)?;
            }
            if expression.term().type_id() != SOURCE_BOOL {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let args = expression
                .subjects()
                .iter()
                .map(|(_, ty)| ty.to_string())
                .collect::<Vec<_>>();
            let mut env = args.clone();
            let mut body = c.lower(expression.term(), &mut env, 0)?;
            for arg in args.iter().rev() {
                let ty = c.ty(arg, 0)?;
                body = c.b.lam(ty, body)?;
            }
            // A constant expression may occur on source types with different
            // carriers. Preserve legacy names when unambiguous; distinguish
            // complete attachments whenever the subject signature differs.
            let suffix = if scopes[expression.expression_sha256()].len() > 1 {
                format!(".A{}", expression.attachment_sha256())
            } else {
                String::new()
            };
            let definition = format!(
                "{PREFIX}.SourceClause.H{}{suffix}",
                expression.expression_sha256()
            );
            if !c.b.globals.contains_key(&definition) {
                let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
                c.b.define(&definition, ty, body)?;
            }
            let definedness_definition = if expression.definitions().iter().any(|d| {
                !d.ordered_checks.is_empty()
                    || matches!(
                        d.tag.as_str(),
                        "sequence_index"
                            | "tagged_payload"
                            | "exception_payload"
                            | "codec_format"
                            | "bounded_forall"
                            | "bounded_exists"
                    )
            }) {
                c.definedness_logic()?;
                // Compile the canonical W03 term itself, preserving guarded
                // conditional branches and lexical scopes. No new assumptions
                // or conjunction with the public-domain equation are introduced.
                let term = crate::csharp_practical_vir_model::data_vc::definedness(
                    expression.term(),
                    expression.definitions(),
                )
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
                let mut body = c.lower(&term, &mut args.clone(), 0)?;
                for arg in args.iter().rev() {
                    let ty = c.ty(arg, 0)?;
                    body = c.b.lam(ty, body)?;
                }
                let name = format!(
                    "{PREFIX}.SourceClauseDefined.H{}{suffix}",
                    expression.expression_sha256()
                );
                if !c.b.globals.contains_key(&name) {
                    let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
                    c.b.define(&name, ty, body)?;
                }
                Some(name)
            } else {
                None
            };
            definitions.push(OrdinarySourceClauseDefinition {
                source_type_id: source.type_id.clone(),
                source_sha256: source.source_sha256.clone(),
                attachment_sha256: expression.attachment_sha256().into(),
                expression_sha256: expression.expression_sha256().into(),
                argument_types: args,
                definition,
                definedness_definition,
            });
        }
    }
    Ok((
        c.b,
        c.storage,
        c.relations,
        definitions,
        construction.hash(),
    ))
}
pub fn generate_csharp_practical_ordinary_source_clauses(
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceClauseProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, _, _, definitions, construction_sha256) = emit_clauses(vir, &layouts, Builder::new()?)?;
    let certificate = b.finish()?;
    let p = OrdinarySourceClauseProgram {
        schema: "mpk.csharp.ordinary_source_clauses.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        construction_sha256,
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_source_clauses(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceClauseProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_source_clauses(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[path = "csharp_practical_ordinary_contract_expressions.rs"]
mod expressions;
pub use expressions::{
    generate_csharp_practical_ordinary_contract_expressions,
    import_csharp_practical_ordinary_contract_expressions, OrdinaryContractExpressionDefinition,
    OrdinaryContractExpressionProgram,
};

#[path = "csharp_practical_ordinary_integer_data.rs"]
mod integer_data;
pub use integer_data::{
    generate_csharp_practical_ordinary_integer_data, import_csharp_practical_ordinary_integer_data,
    OrdinaryIntegerDataDefinition, OrdinaryIntegerDataOperation, OrdinaryIntegerDataProgram,
};

#[path = "csharp_practical_ordinary_structural_data.rs"]
mod structural_data;
pub use structural_data::{
    generate_csharp_practical_ordinary_structural_data,
    import_csharp_practical_ordinary_structural_data, OrdinaryStructuralDataDefinition,
    OrdinaryStructuralDataOperation, OrdinaryStructuralDataProgram,
};

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{bit as observed, run, V};
    use super::*;
    #[test]
    fn ordinary_contract_calendar_temporal_dispatch_preserves_frozen_definitions() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation");
        let mut pins = 0;
        let mut aliases = 0;
        let mut outcome_rejections = 0;
        for family in ["calendar-circuits", "temporal-circuits"] {
            let rows: Value = serde_json::from_slice(
                &std::fs::read(root.join(family).join("metrics.json")).unwrap(),
            )
            .unwrap();
            for row in rows.as_array().unwrap() {
                let operation: ClosedOperationSignature =
                    serde_json::from_value(row["definition"]["operation"].clone()).unwrap();
                // The new shared entry points must reproduce every standalone
                // certificate, including non-unary/binary operations.
                let mut b = Builder::new().unwrap();
                let scalar = if family == "calendar-circuits" {
                    super::super::super::scalar_bits::emit_calendar(&mut b, &operation.id)
                } else {
                    super::super::super::scalar_bits::emit_temporal(&mut b, &operation.id)
                }
                .unwrap();
                assert_eq!(serde_json::to_value(&scalar).unwrap(), row["definition"]);
                let bytes = b.finish().unwrap();
                assert_eq!(
                    mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)),
                    row["hash"].as_str().unwrap(),
                    "standalone bytes changed: {}",
                    operation.id
                );
                pins += 1;
                if !matches!(operation.argument_type_ids.len(), 1 | 2) {
                    continue;
                }
                let types = operation
                    .argument_type_ids
                    .iter()
                    .chain(std::iter::once(&operation.normal_result_type_id))
                    .map(|id| {
                        let width = match id.as_str().strip_prefix("mpk.csharp.value.").unwrap() {
                            "bool.v1" => 1,
                            "date.v1" | "day_of_week.v1" | "i32.v1" => 32,
                            "time.v1" | "duration.v1" | "instant.v1" | "i64.v1" => 64,
                            "guid.v1" => 128,
                            _ => panic!("unexpected type: {id}"),
                        };
                        OrdinaryCarrier {
                            type_id: id.clone(),
                            depth: address_bits(width),
                            shape: OrdinaryShape::Bits { width },
                        }
                    })
                    .collect::<Vec<_>>();
                let mut c = Clauses {
                    vir: None,
                    relations: relations::ContractRelationCache::default(),
                    b: Builder::new().unwrap(),
                    carriers: types.iter().map(|t| (t.type_id.as_str(), t)).collect(),
                    storage: StorageCache::default(),
                    constants: BTreeMap::new(),
                    scalars: BTreeMap::new(),
                    strings: Default::default(),
                    finite: None,
                    codecs: Default::default(),
                    binding_projections: None,
                    quantifiers: BTreeMap::new(),
                };
                let d = ContractDefinition {
                    name: format!("test.{}", operation.id),
                    tag: if operation.argument_type_ids.len() == 1 {
                        "unary"
                    } else {
                        "binary"
                    }
                    .into(),
                    parameters: serde_json::json!({"operation_id": operation.id}).to_string(),
                    argument_types: operation.argument_type_ids.clone(),
                    result_type: operation.normal_result_type_id.clone(),
                    ordered_checks: operation.ordered_checks.clone(),
                };
                if operation
                    .ordered_checks
                    .iter()
                    .any(|check| check.tag == RequiredCheckTag::ErrorOutcome)
                {
                    // An Instant error outcome cannot be silently interpreted
                    // as its zeroed normal result. Its outcome adapter is open.
                    assert!(matches!(c.recipe(&d), Err(OrdinaryCarrierError::Linkage)));
                    assert!(!c.constants.contains_key(&d.name));
                    outcome_rejections += 1;
                    continue;
                }
                c.recipe(&d)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", operation.id));
                let (actual, predicates) = &c.scalars[&operation.id];
                assert_eq!(actual, &scalar);
                assert_eq!(predicates, &scalar.ordered_failure_definitions);
                assert!(
                    predicates.len() <= 1,
                    "ordered failures need a raw-check adapter"
                );
                for (check, predicate) in operation.ordered_checks.iter().zip(predicates) {
                    assert_eq!(
                        c.constants
                            [&format!("Mpk.CSharp.Data.ContractFails.{}.{}", d.name, check.id)]
                            .1,
                        *predicate
                    );
                }
                for variant in 0..3 {
                    let mut bad = d.clone();
                    bad.name.push_str(&format!(".bad{variant}"));
                    match variant {
                        0 => bad.result_type = "test.wrong-result".into(),
                        1 => bad.argument_types[0] = "test.wrong-argument".into(),
                        _ => {
                            if bad.ordered_checks.is_empty() {
                                bad.ordered_checks.push(RequiredCheck {
                                    id: "exception.range".into(),
                                    tag: RequiredCheckTag::Exception,
                                    failure_type_id: Some(
                                        "System.ArgumentOutOfRangeException".into(),
                                    ),
                                });
                            } else {
                                bad.ordered_checks.clear();
                            }
                        }
                    }
                    assert!(
                        matches!(c.recipe(&bad), Err(OrdinaryCarrierError::Linkage)),
                        "{} mutation {variant}",
                        operation.id
                    );
                }
                aliases += 1;
            }
        }
        assert_eq!((pins, aliases, outcome_rejections), (70, 65, 3));
        eprintln!("calendar/temporal adapters: 70 unchanged standalone certificates, 65 aliases, 195 signature mutations, 3 unimplemented error-outcome rejections");
    }
    #[test]
    fn ordinary_contract_decimal_dispatch_preserves_signatures_and_raw_checks() {
        // Exercise the adapter with frozen standalone signatures. This is a
        // lowering test; captured native-source coverage is a separate gate.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation");
        let mut count = 0;
        for family in ["decimal-circuits", "decimal-arithmetic-circuits"] {
            let rows: Value = serde_json::from_slice(
                &std::fs::read(root.join(family).join("metrics.json")).unwrap(),
            )
            .unwrap();
            for row in rows.as_array().unwrap() {
                let operation: ClosedOperationSignature =
                    serde_json::from_value(row["definition"]["operation"].clone()).unwrap();
                let types = operation
                    .argument_type_ids
                    .iter()
                    .chain(std::iter::once(&operation.normal_result_type_id))
                    .map(|id| {
                        let width = match id.as_str().strip_prefix("mpk.csharp.value.").unwrap() {
                            "decimal.v1" => 512,
                            "bool.v1" => 1,
                            "i8.v1" | "u8.v1" => 8,
                            "i16.v1" | "u16.v1" | "char.v1" => 16,
                            "i32.v1" | "u32.v1" => 32,
                            "i64.v1" | "u64.v1" => 64,
                            _ => panic!("unexpected type"),
                        };
                        OrdinaryCarrier {
                            type_id: id.clone(),
                            depth: address_bits(width),
                            shape: OrdinaryShape::Bits { width },
                        }
                    })
                    .collect::<Vec<_>>();
                let mut c = Clauses {
                    vir: None,
                    relations: relations::ContractRelationCache::default(),
                    b: Builder::new().unwrap(),
                    carriers: types.iter().map(|t| (t.type_id.as_str(), t)).collect(),
                    storage: StorageCache::default(),
                    constants: BTreeMap::new(),
                    scalars: BTreeMap::new(),
                    strings: Default::default(),
                    finite: None,
                    codecs: Default::default(),
                    binding_projections: None,
                    quantifiers: BTreeMap::new(),
                };
                let d = ContractDefinition {
                    name: format!("test.{}", operation.id),
                    tag: if operation.argument_type_ids.len() == 1 {
                        "unary"
                    } else {
                        "binary"
                    }
                    .into(),
                    parameters: serde_json::json!({"operation_id": operation.id}).to_string(),
                    argument_types: operation.argument_type_ids.clone(),
                    result_type: operation.normal_result_type_id.clone(),
                    ordered_checks: operation.ordered_checks.clone(),
                };
                c.recipe(&d)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", operation.id));
                let (scalar, predicates) = &c.scalars[&operation.id];
                assert_eq!(scalar.operation, operation);
                assert_eq!(predicates.len(), operation.ordered_checks.len());
                for (check, predicate) in operation.ordered_checks.iter().zip(predicates) {
                    let alias = format!("Mpk.CSharp.Data.ContractFails.{}.{}", d.name, check.id);
                    assert_eq!(c.constants[&alias].1, *predicate);
                    if family == "decimal-arithmetic-circuits" {
                        assert!(predicate.contains(".ContractFailure.F"));
                        assert!(!scalar.ordered_failure_definitions.contains(predicate));
                    } else {
                        assert!(scalar.ordered_failure_definitions.contains(predicate));
                    }
                }
                for variant in 0..3 {
                    let mut bad = d.clone();
                    bad.name.push_str(&format!(".bad{variant}"));
                    match variant {
                        0 => bad.result_type = "test.wrong-result".into(),
                        1 => bad.argument_types[0] = "test.wrong-argument".into(),
                        _ => bad.ordered_checks.push(RequiredCheck {
                            id: "exception.extra".into(),
                            tag: RequiredCheckTag::Exception,
                            failure_type_id: Some("System.Exception".into()),
                        }),
                    }
                    assert!(matches!(c.recipe(&bad), Err(OrdinaryCarrierError::Linkage)));
                }
                let certificate = decode_canonical_certificate(&c.b.finish().unwrap()).unwrap();
                crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                    &certificate,
                )
                .unwrap();
                count += 1;
            }
        }
        assert_eq!(count, 45);
    }
    #[test]
    fn ordinary_contract_lowering_preserves_binders_and_nominal_types() {
        let types = [SOURCE_BOOL, "test.other-bool"].map(|id| OrdinaryCarrier {
            type_id: id.into(),
            depth: 0,
            shape: OrdinaryShape::Bits { width: 1 },
        });
        let mut c = Clauses {
            vir: None,
            relations: relations::ContractRelationCache::default(),
            b: Builder::new().unwrap(),
            carriers: types.iter().map(|t| (t.type_id.as_str(), t)).collect(),
            storage: StorageCache::default(),
            constants: BTreeMap::new(),
            scalars: BTreeMap::new(),
            strings: Default::default(),
            finite: None,
            codecs: Default::default(),
            binding_projections: None,
            quantifiers: BTreeMap::new(),
        };
        let var = |index| ContractTerm::Var {
            index,
            type_id: SOURCE_BOOL.into(),
        };
        let term = ContractTerm::Let {
            value: Box::new(var(1)),
            body: Box::new(ContractTerm::Let {
                value: Box::new(var(1)),
                body: Box::new(var(1)),
                type_id: SOURCE_BOOL.into(),
            }),
            type_id: SOURCE_BOOL.into(),
        };
        let body = c.lower(&term, &mut vec![SOURCE_BOOL.into(); 2], 0).unwrap();
        define(&mut c.b, "Test.NestedLet", &[0, 0], 0, body).unwrap();
        let term = ContractTerm::App {
            function: Box::new(ContractTerm::Lam {
                parameter_type: SOURCE_BOOL.into(),
                body: Box::new(var(1)),
                type_id: format!("({SOURCE_BOOL}->{SOURCE_BOOL})"),
            }),
            argument: Box::new(var(1)),
            type_id: SOURCE_BOOL.into(),
        };
        let body = c.lower(&term, &mut vec![SOURCE_BOOL.into(); 2], 0).unwrap();
        define(&mut c.b, "Test.CapturedLambda", &[0, 0], 0, body).unwrap();
        assert!(c
            .lower(&var(0), &mut vec!["test.other-bool".into()], 0)
            .is_err());
        assert!(c.lower(&var(1), &mut vec![SOURCE_BOOL.into()], 0).is_err());
        let unknown = ContractTerm::Const {
            name: "missing".into(),
            type_id: SOURCE_BOOL.into(),
        };
        assert!(c.lower(&unknown, &mut vec![], 0).is_err());
        let mut wrong = term.clone();
        if let ContractTerm::App { type_id, .. } = &mut wrong {
            *type_id = "test.other-bool".into();
        }
        assert!(c
            .lower(&wrong, &mut vec![SOURCE_BOOL.into(); 2], 0)
            .is_err());
        assert!(c.ty("(missing->mpk.csharp.value.bool.v1)", 0).is_err());
        let bytes = c.b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        for a in [false, true] {
            for b in [false, true] {
                assert_eq!(
                    observed(run(&cert, "Test.NestedLet", vec![V::Bit(a), V::Bit(b)])),
                    a
                );
                assert_eq!(
                    observed(run(
                        &cert,
                        "Test.CapturedLambda",
                        vec![V::Bit(a), V::Bit(b)]
                    )),
                    b
                );
            }
        }
        if let Some(out) = std::env::var_os("MPK_W09_SOURCE_CLAUSE_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("contract-lowering.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
    #[test]
    fn ordinary_conditional_recipe_preserves_nominal_cubes() {
        use super::super::super::test_eval::{apply, sparse_cube};
        let types = [
            OrdinaryCarrier {
                type_id: SOURCE_BOOL.into(),
                depth: 0,
                shape: OrdinaryShape::Bits { width: 1 },
            },
            OrdinaryCarrier {
                type_id: "mpk.csharp.value.i32.v1".into(),
                depth: 5,
                shape: OrdinaryShape::Bits { width: 32 },
            },
            OrdinaryCarrier {
                type_id: "test.large".into(),
                depth: 19,
                shape: OrdinaryShape::Array {
                    capacity: 16384,
                    element: Box::new(OrdinaryShape::Bits { width: 32 }),
                },
            },
            OrdinaryCarrier {
                type_id: "test.other-bool".into(),
                depth: 0,
                shape: OrdinaryShape::Bits { width: 1 },
            },
        ];
        let mut c = Clauses {
            vir: None,
            relations: relations::ContractRelationCache::default(),
            b: Builder::new().unwrap(),
            carriers: types.iter().map(|t| (t.type_id.as_str(), t)).collect(),
            storage: StorageCache::default(),
            constants: BTreeMap::new(),
            scalars: BTreeMap::new(),
            strings: Default::default(),
            finite: None,
            codecs: Default::default(),
            binding_projections: None,
            quantifiers: BTreeMap::new(),
        };
        let mut names = vec![];
        for carrier in &types[..3] {
            let d = ContractDefinition {
                name: format!("test.conditional.{}", carrier.depth),
                tag: "conditional".into(),
                parameters: "{}".into(),
                argument_types: vec![
                    SOURCE_BOOL.into(),
                    carrier.type_id.clone(),
                    carrier.type_id.clone(),
                ],
                result_type: carrier.type_id.clone(),
                ordered_checks: vec![],
            };
            c.recipe(&d).unwrap();
            names.push((carrier.depth, c.constants[&d.name].1.clone()));
        }
        let mut bad = ContractDefinition {
            name: "test.bad".into(),
            tag: "conditional".into(),
            parameters: "{}".into(),
            argument_types: vec![
                SOURCE_BOOL.into(),
                SOURCE_BOOL.into(),
                "test.other-bool".into(),
            ],
            result_type: SOURCE_BOOL.into(),
            ordered_checks: vec![],
        };
        assert!(c.recipe(&bad).is_err());
        bad.argument_types = vec![
            "test.other-bool".into(),
            SOURCE_BOOL.into(),
            SOURCE_BOOL.into(),
        ];
        assert!(c.recipe(&bad).is_err());
        bad.argument_types = vec![SOURCE_BOOL.into(); 3];
        bad.parameters = "{\"unexpected\":true}".into();
        assert!(c.recipe(&bad).is_err());
        let bytes = c.b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        for (depth, name) in names {
            let size = 1usize << depth;
            let yes = [0, size - 1].into_iter().collect::<BTreeSet<_>>();
            let no = [1, size / 2]
                .into_iter()
                .filter(|i| *i < size && !yes.contains(i))
                .collect::<BTreeSet<_>>();
            let probes = [0, 1, size / 2, size - 1]
                .into_iter()
                .filter(|i| *i < size)
                .collect::<BTreeSet<_>>();
            for condition in [false, true] {
                let value = run(
                    &cert,
                    &name,
                    vec![
                        V::Bit(condition),
                        sparse_cube(depth, yes.clone()),
                        sparse_cube(depth, no.clone()),
                    ],
                );
                for i in &probes {
                    let mut v = value.clone();
                    for k in 0..depth {
                        v = apply(&cert, v, V::Bit(i & (1 << k) != 0));
                    }
                    assert_eq!(
                        observed(v),
                        if condition {
                            yes.contains(i)
                        } else {
                            no.contains(i)
                        },
                        "depth{depth}: condition{condition} index{i}"
                    );
                }
            }
        }
        if let Some(out) = std::env::var_os("MPK_W09_CONDITIONAL_CLAUSES_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("conditional-cubes.hex"),
                bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
}

#[path = "csharp_practical_ordinary_floating_data.rs"]
mod floating_data;
pub use floating_data::{
    generate_csharp_practical_ordinary_floating_data,
    import_csharp_practical_ordinary_floating_data, OrdinaryFloatingDataDefinition,
    OrdinaryFloatingDataOperation, OrdinaryFloatingDataProgram,
};

#[path = "csharp_practical_ordinary_decimal_data.rs"]
mod decimal_data;
pub use decimal_data::{
    generate_csharp_practical_ordinary_decimal_data, import_csharp_practical_ordinary_decimal_data,
    OrdinaryDecimalDataDefinition, OrdinaryDecimalDataOperation, OrdinaryDecimalDataProgram,
};
