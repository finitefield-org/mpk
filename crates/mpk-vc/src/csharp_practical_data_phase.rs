//! T03-W14 closes source-derived data bindings and shared routing before VIR.
//! All projection and contract obligations remain obligations for T06.
use super::*;
#[path = "csharp_practical_contract_values.rs"]
mod contract_values;
use crate::csharp_practical_source_artifacts::{
    self as artifacts, CapturedInputSet, PracticalArtifactContext, SemanticBindingInput,
    ValidatedPracticalArtifact,
};
use contract_values::decode_contract_value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataPhaseError {
    Source,
    Sidecar,
    Binding,
    Unreachable,
    Cycle,
    Arguments,
    Default,
    Contract,
    Routing,
    Emission,
    Import {
        phase: &'static str,
        code: &'static str,
    },
    LaterOwner(&'static str),
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataTypeRoute {
    pub type_id: String,
    pub equality: String,
    pub ordering: Option<String>,
    pub codecs: Vec<String>,
}
/// Enumerate routes by asking the existing generators, never by reproducing
/// their equality, ordering, or codec eligibility algorithms.
pub fn derive_data_type_routes(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
) -> Result<Vec<DataTypeRoute>, DataPhaseError> {
    let mut ids: BTreeSet<String> = PRIMITIVES
        .iter()
        .map(|s| format!("mpk.csharp.value.{s}.v1"))
        .collect();
    ids.extend(r.source_types.keys().cloned());
    ids.extend(c.metadata.keys().cloned());
    let codecs = [
        "integer.i8",
        "integer.u8",
        "integer.i16",
        "integer.u16",
        "integer.i32",
        "integer.u32",
        "integer.i64",
        "integer.u64",
        "binary32",
        "binary64",
        "decimal.normalized",
        "decimal.fixed",
        "date",
        "time",
        "duration_ticks",
        "unix_milliseconds",
        "guid.n",
        "guid.d",
    ];
    let mut routes = vec![];
    for id in ids {
        if c.metadata
            .get(&id)
            .is_some_and(|m| template_name(&m.template_id) == Some("sequence_construction"))
        {
            continue;
        }
        let program =
            generate_structural_program(b, r, c, &id).map_err(|_| DataPhaseError::Routing)?;
        let mut applicable = vec![];
        for codec in codecs {
            let (scale, mode) = if codec == "decimal.fixed" {
                (Some(0), Some("ToEven"))
            } else {
                (None, None)
            };
            if BoundaryCodec::new(codec, &id, scale, mode).is_ok() {
                applicable.push(codec.into());
            }
        }
        applicable.sort();
        routes.push(DataTypeRoute {
            type_id: id,
            equality: "generate_structural_program.structural_equal".into(),
            ordering: program
                .is_total()
                .then(|| "generate_structural_program.canonical_compare".into()),
            codecs: applicable,
        });
    }
    Ok(routes)
}
pub fn validate_data_type_routes(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    candidate: &[DataTypeRoute],
) -> Result<(), DataPhaseError> {
    if derive_data_type_routes(b, r, c)? == candidate {
        Ok(())
    } else {
        Err(DataPhaseError::Routing)
    }
}

#[derive(Clone, Debug)]
pub struct DataBindingClosure {
    roots: ValidatedClosedRootSet,
    closed: ClosedInstanceSet,
    projections: BTreeMap<String, String>,
    obligations: Vec<OutcomeObligation>,
    bindings: ValidatedPracticalArtifact,
}
impl DataBindingClosure {
    /// `source_roots` is regenerated from the captured source view, not a
    /// caller-selected closed-instance allowlist. The root validator and sole
    /// T02 specialization engine independently check its concrete shape.
    pub fn derive(
        b: &ValidatedFoundationBundle,
        context: &PracticalArtifactContext,
        captures: &CapturedInputSet,
        source_roots: &ValidatedClosedRootSet,
        inputs: &[SemanticBindingInput],
        reachable: &BTreeSet<String>,
    ) -> Result<Self, DataPhaseError> {
        let bindings = artifacts::build_semantic_bindings(context, captures, inputs.to_vec())
            .map_err(|_| DataPhaseError::Sidecar)?;
        let supplied: BTreeMap<_, _> = inputs
            .iter()
            .map(|i| (i.source_type_id.clone(), i))
            .collect();
        for input in inputs {
            if !reachable.contains(&input.source_type_id) {
                return Err(DataPhaseError::Unreachable);
            }
            let source = source_roots
                .source_types
                .get(&input.source_type_id)
                .ok_or(DataPhaseError::Source)?;
            if source.source_sha256 != input.source_content_sha256
                || source.kind == SourceKind::Enum
            {
                return Err(DataPhaseError::Source);
            }
            if input
                .member_map
                .iter()
                .any(|m| !source.members.iter().any(|f| f.id == m.member_id))
            {
                return Err(DataPhaseError::Binding);
            }
        }
        let mut resolved = BTreeMap::<String, ClosedType>::new();
        fn project(
            b: &ValidatedFoundationBundle,
            r: &ValidatedClosedRootSet,
            bindings: &BTreeMap<String, &SemanticBindingInput>,
            resolved: &mut BTreeMap<String, ClosedType>,
            active: &mut BTreeSet<String>,
            ty: &ClosedType,
        ) -> Result<ClosedType, DataPhaseError> {
            match ty {
                ClosedType::Source(id) if bindings.contains_key(id) => {
                    resolve(b, r, bindings, resolved, active, id)
                }
                ClosedType::Instance {
                    template,
                    arguments,
                } => Ok(ClosedType::Instance {
                    template: template.clone(),
                    arguments: arguments
                        .iter()
                        .map(|t| project(b, r, bindings, resolved, active, t))
                        .collect::<Result<_, _>>()?,
                }),
                _ => Ok(ty.clone()),
            }
        }
        fn resolve(
            b: &ValidatedFoundationBundle,
            r: &ValidatedClosedRootSet,
            bindings: &BTreeMap<String, &SemanticBindingInput>,
            resolved: &mut BTreeMap<String, ClosedType>,
            active: &mut BTreeSet<String>,
            id: &str,
        ) -> Result<ClosedType, DataPhaseError> {
            if let Some(t) = resolved.get(id) {
                return Ok(t.clone());
            }
            if !active.insert(id.into()) {
                return Err(DataPhaseError::Cycle);
            }
            let input = bindings[id];
            let source = &r.source_types[id];
            let member = |role: &str| -> Result<&ClosedType, DataPhaseError> {
                let mapping = input
                    .member_map
                    .iter()
                    .find(|m| m.role == role)
                    .ok_or(DataPhaseError::Binding)?;
                source
                    .members
                    .iter()
                    .find(|m| m.id == mapping.member_id)
                    .map(|m| &m.ty)
                    .ok_or(DataPhaseError::Binding)
            };
            let element = |ty: &ClosedType| -> Result<ClosedType, DataPhaseError> {
                if let ClosedType::Instance {
                    template,
                    arguments,
                } = ty
                {
                    if template == "bounded_sequence" && arguments.len() == 1 {
                        return Ok(arguments[0].clone());
                    }
                }
                Err(DataPhaseError::Binding)
            };
            let raw = match input.role.as_str() {
                "instant" => {
                    if member("milliseconds")? != &ClosedType::Primitive("i64".into()) {
                        return Err(DataPhaseError::Binding);
                    }
                    vec![]
                }
                "money" => {
                    if source.kind != SourceKind::ReadonlyStruct
                        || member("amount")? != &ClosedType::Primitive("decimal".into())
                    {
                        return Err(DataPhaseError::Binding);
                    }
                    let currency = member("currency")?;
                    if !matches!(currency,ClosedType::Primitive(t) if t=="string")
                        && !matches!(currency,ClosedType::Source(id) if r.source_types.get(id).is_some_and(|s|s.kind==SourceKind::Enum))
                    {
                        return Err(DataPhaseError::Binding);
                    }
                    vec![currency.clone()]
                }
                "option" | "lookup" | "boundary_field" => vec![member("value")?.clone()],
                "result" => vec![member("value")?.clone(), member("error")?.clone()],
                "validation" => vec![member("value")?.clone(), element(member("errors")?)?],
                "bounded_sequence" | "ordered_set" => vec![element(member("elements")?)?],
                "ordered_entry" => vec![member("key")?.clone(), member("value")?.clone()],
                "ordered_map" => {
                    let entry = element(member("entries")?)?;
                    let ClosedType::Source(e) = entry else {
                        return Err(DataPhaseError::Binding);
                    };
                    if !bindings.get(&e).is_some_and(|b| b.role == "ordered_entry") {
                        return Err(DataPhaseError::Binding);
                    }
                    let ClosedType::Instance { arguments, .. } =
                        resolve(b, r, bindings, resolved, active, &e)?
                    else {
                        return Err(DataPhaseError::Binding);
                    };
                    arguments
                }
                "transition" => vec![
                    member("state")?.clone(),
                    element(member("events")?)?,
                    member("response")?.clone(),
                ],
                _ => return Err(DataPhaseError::Binding),
            };
            if !input.tag_arms.is_empty() {
                let ClosedType::Source(tag) = member("tag")? else {
                    return Err(DataPhaseError::Binding);
                };
                let en = r
                    .source_types
                    .get(tag)
                    .filter(|s| s.kind == SourceKind::Enum)
                    .ok_or(DataPhaseError::Binding)?;
                if input
                    .tag_arms
                    .iter()
                    .map(|a| &a.source_tag)
                    .collect::<BTreeSet<_>>()
                    != en.enum_values.iter().collect()
                {
                    return Err(DataPhaseError::Binding);
                }
            }
            let args = raw
                .iter()
                .map(|t| project(b, r, bindings, resolved, active, t))
                .collect::<Result<Vec<_>, _>>()?;
            let ids = args
                .iter()
                .map(|t| closed_type_id(b, t).map_err(|_| DataPhaseError::Arguments))
                .collect::<Result<Vec<_>, _>>()?;
            if ids != input.inferred_argument_ids {
                return Err(DataPhaseError::Arguments);
            }
            let semantic = if input.role == "instant" {
                ClosedType::Primitive("instant".into())
            } else {
                ClosedType::Instance {
                    template: input.role.clone(),
                    arguments: args,
                }
            };
            active.remove(id);
            resolved.insert(id.into(), semantic.clone());
            Ok(semantic)
        }
        for id in supplied.keys() {
            resolve(
                b,
                source_roots,
                &supplied,
                &mut resolved,
                &mut BTreeSet::new(),
                id,
            )?;
        }
        let mut root_values:Vec<Value>=source_roots.roots.iter().map(|r|json!({"origin":r.origin,"provenance_id":r.provenance_id,"type":r.ty.to_value()})).collect();
        for (id, ty) in &resolved {
            root_values.push(json!({"origin":"semantic_binding","provenance_id":id,"type":{"kind":"source","id":id}}));
            root_values
                .push(json!({"origin":"semantic_binding","provenance_id":id,"type":ty.to_value()}));
        }
        let source_value: Value = serde_json::from_slice(&source_roots.canonical_json)
            .map_err(|_| DataPhaseError::Source)?;
        let bytes = canonical_closed_root_set_transport(
            b,
            &json!(root_values),
            &source_value["source_types"],
        )
        .map_err(|_| DataPhaseError::Binding)?;
        let roots = validate_closed_root_set(b, &bytes).map_err(|_| DataPhaseError::Binding)?;
        let closed = derive_closed_instances(b, &roots).map_err(|_| DataPhaseError::Binding)?;
        let projections = resolved
            .iter()
            .map(|(id, t)| {
                Ok((
                    id.clone(),
                    closed_type_id(b, t).map_err(|_| DataPhaseError::Binding)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, DataPhaseError>>()?;
        let mut obligations = vec![];
        for input in inputs {
            let source = &roots.source_types[&input.source_type_id];
            let semantic = &projections[&source.id];
            let mut obligation = |kind: &str, member: &str| {
                obligations.push(OutcomeObligation {
                    source_type_id: source.id.clone(),
                    semantic_type_id: semantic.clone(),
                    kind: kind.into(),
                    member_id: member.into(),
                    discharged: false,
                })
            };
            for kind in [
                "source_invariant_implies_projection",
                "semantic_invariant_implies_reconstruction",
                "source_round_trip",
                "semantic_round_trip",
                "distinct_arms",
                "public_invariant",
                "identity_unobservable",
            ] {
                obligation(kind, "");
            }
            for member in &source.members {
                obligation("field_complete_reconstruction", &member.id);
            }
            if input.role == "money" {
                obligation("application_currency_predicate", "");
                obligation("default_ineligible", "");
            }
            if input.default_arm != "ineligible" {
                obligation("actual_default_public_invariant", "");
            }
            if matches!(input.role.as_str(), "ordered_map" | "ordered_set") {
                let args = &closed.metadata[semantic].argument_ids;
                if !generate_structural_program(b, &roots, &closed, &args[0])
                    .map_err(|_| DataPhaseError::Routing)?
                    .is_total()
                {
                    return Err(DataPhaseError::Routing);
                }
                obligation("canonical_order_and_uniqueness", "");
            }
            for op in &input.operation_map {
                for kind in [
                    "operation_normal_commutation",
                    "operation_error_commutation",
                    "operation_exception_commutation",
                ] {
                    obligation(kind, &op.member_id);
                }
            }
            if obligations
                .iter()
                .filter(|o| o.source_type_id == source.id)
                .count()
                > PROJECTION_OBLIGATIONS_PER_BINDING_MAX as usize
            {
                return Err(DataPhaseError::Binding);
            }
        }
        Ok(Self {
            roots,
            closed,
            projections,
            obligations,
            bindings,
        })
    }
    pub fn roots(&self) -> &ValidatedClosedRootSet {
        &self.roots
    }
    pub fn closed(&self) -> &ClosedInstanceSet {
        &self.closed
    }
    pub fn projections(&self) -> &BTreeMap<String, String> {
        &self.projections
    }
    pub fn obligations(&self) -> &[OutcomeObligation] {
        &self.obligations
    }
    pub fn bindings(&self) -> &ValidatedPracticalArtifact {
        &self.bindings
    }
}

/// Internal typed handoff for expression validation. Production attachment
/// derives this environment from captured declarations and validated bindings.
#[derive(Clone, Debug, Default)]
pub struct DataContractEnvironment {
    pub variables: BTreeMap<String, String>,
    pub result: Option<String>,
    pub operations: BTreeMap<String, ClosedOperationSignature>,
    pub constructors: BTreeMap<String, ClosedOperationSignature>,
    pub properties: BTreeMap<String, (String, String)>,
    pub bindings: BTreeMap<String, (String, String)>,
    pub allow_old: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedDataExpression {
    value: artifacts::PracticalJsonValue,
    type_id: String,
    nodes: usize,
}
impl ValidatedDataExpression {
    pub fn value(&self) -> &artifacts::PracticalJsonValue {
        &self.value
    }
    pub fn type_id(&self) -> &str {
        &self.type_id
    }
    pub fn nodes(&self) -> usize {
        self.nodes
    }
}
/// Field and tag shapes come from the frozen 33-arm expression union. This is
/// data-phase parsing/type checking only; logical proof generation belongs to T06.
fn check_normalized_data_expression(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    environment: &DataContractEnvironment,
    value: &Value,
    original: &artifacts::PracticalJsonValue,
) -> Result<ValidatedDataExpression, DataPhaseError> {
    static SHAPES: std::sync::OnceLock<BTreeMap<String, Vec<String>>> = std::sync::OnceLock::new();
    let shapes = SHAPES.get_or_init(|| {
        let package: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../develop/specs/vectors/csharp-practical-profile-v1.json"
        )))
        .expect("frozen package");
        package["frozen_contract"]["expression_union"]["variants"]
            .as_array()
            .expect("frozen union")
            .iter()
            .map(|v| {
                (
                    v["tag"].as_str().unwrap().into(),
                    v["ordered_fields"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| s.as_str().unwrap().into())
                        .collect(),
                )
            })
            .collect()
    });
    struct Check<'a> {
        b: &'a ValidatedFoundationBundle,
        r: &'a ValidatedClosedRootSet,
        c: &'a ClosedInstanceSet,
        env: &'a DataContractEnvironment,
        shapes: &'a BTreeMap<String, Vec<String>>,
        nodes: usize,
    }
    impl Check<'_> {
        fn expression(
            &mut self,
            v: &Value,
            variables: &mut BTreeMap<String, String>,
            depth: usize,
            old: bool,
            quantifiers: usize,
        ) -> Result<String, DataPhaseError> {
            self.nodes += 1;
            if depth > data_contract_limit("contract_depth")
                || self.nodes > data_contract_limit("contract_nodes_per_method")
            {
                return Err(DataPhaseError::Contract);
            }
            let fail = DataPhaseError::Contract;
            let object = v.as_object().ok_or(fail.clone())?;
            let tag = v["tag"].as_str().ok_or(fail.clone())?;
            let shape = self.shapes.get(tag).ok_or(fail.clone())?;
            if object.keys().collect::<BTreeSet<_>>() != shape.iter().collect() {
                return Err(fail);
            }
            let own = v["type_id"]
                .as_str()
                .filter(|id| known_concrete_type(self.r, self.c, id))
                .ok_or(fail.clone())?
                .to_owned();
            let text = |name: &str| {
                v[name]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or(DataPhaseError::Contract)
            };
            let quantifiers =
                quantifiers + usize::from(matches!(tag, "bounded_forall" | "bounded_exists"));
            if quantifiers > data_contract_limit("bounded_quantifier_nesting") {
                return Err(fail);
            }
            let mut child =
                |name: &str| self.expression(&v[name], variables, depth + 1, old, quantifiers);
            let expected = match tag {
                "literal" => {
                    let literal: MonomorphicValue =
                        serde_json::from_value(v["value"].clone()).map_err(|_| fail.clone())?;
                    validate_monomorphic_value(self.b, self.r, self.c, &literal)
                        .map_err(|_| fail.clone())?;
                    literal.type_id().to_owned()
                }
                "variable" => variables
                    .get(text("binding_id")?)
                    .cloned()
                    .ok_or(fail.clone())?,
                "result" => self.env.result.clone().ok_or(fail.clone())?,
                "old" => {
                    if !self.env.allow_old || old {
                        return Err(fail);
                    }
                    self.expression(&v["expression"], variables, depth + 1, true, quantifiers)?
                }
                "let" => {
                    let val = child("value")?;
                    let id = text("binding_id")?;
                    if variables.contains_key(id) {
                        return Err(fail);
                    }
                    variables.insert(id.into(), val);
                    let body =
                        self.expression(&v["body"], variables, depth + 1, old, quantifiers)?;
                    variables.remove(id);
                    body
                }
                "bounded_forall" | "bounded_exists" => {
                    let low = child("lower")?;
                    let high = child("upper")?;
                    if low != high
                        || !matches!(
                            low.as_str(),
                            "mpk.csharp.value.i32.v1"
                                | "mpk.csharp.value.i64.v1"
                                | "mpk.csharp.value.u32.v1"
                                | "mpk.csharp.value.u64.v1"
                        )
                    {
                        return Err(fail);
                    }
                    let id = text("binding_id")?;
                    if variables.contains_key(id) {
                        return Err(fail);
                    }
                    variables.insert(id.into(), low);
                    let body =
                        self.expression(&v["body"], variables, depth + 1, old, quantifiers)?;
                    variables.remove(id);
                    if body != BOOL_TYPE_ID {
                        return Err(fail);
                    }
                    BOOL_TYPE_ID.into()
                }
                "conditional" => {
                    let condition = child("condition")?;
                    let yes = child("when_true")?;
                    let no = child("when_false")?;
                    if condition != BOOL_TYPE_ID || yes != no {
                        return Err(fail);
                    }
                    yes
                }
                "unary" | "binary" => {
                    let args = if tag == "unary" {
                        vec![child("operand")?]
                    } else {
                        vec![child("left")?, child("right")?]
                    };
                    let generated;
                    let op = if let Some(op) = self.env.operations.get(text("operation_id")?) {
                        op
                    } else {
                        generated = scalar_operation_signature(text("operation_id")?)
                            .map_err(|_| fail.clone())?;
                        &generated
                    };
                    if op.tag == ClosedOperationTag::SourceCall || op.argument_type_ids != args {
                        return Err(fail);
                    }
                    validate_closed_operation_signature(self.r, self.c, op)
                        .map_err(|_| fail.clone())?;
                    op.normal_result_type_id.clone()
                }
                "construct" => {
                    let list = v["arguments"].as_array().ok_or(fail.clone())?;
                    let args = list
                        .iter()
                        .map(|v| self.expression(v, variables, depth + 1, old, quantifiers))
                        .collect::<Result<Vec<_>, _>>()?;
                    let op = self
                        .env
                        .constructors
                        .get(text("constructor_id")?)
                        .ok_or(fail.clone())?;
                    if op.argument_type_ids != args {
                        return Err(fail);
                    }
                    op.normal_result_type_id.clone()
                }
                "field" | "property" => {
                    let receiver = child("receiver")?;
                    let id = text("member_id")?;
                    if tag == "property" {
                        let (subject, result) = self.env.properties.get(id).ok_or(fail.clone())?;
                        if subject != &receiver {
                            return Err(fail);
                        }
                        result.clone()
                    } else {
                        let source = self.r.source_types.get(&receiver).ok_or(fail.clone())?;
                        let member = source
                            .members
                            .iter()
                            .find(|m| m.id == id)
                            .ok_or(fail.clone())?;
                        closed_type_id(self.b, &member.ty).map_err(|_| fail.clone())?
                    }
                }
                "structural_equal" | "structural_compare" => {
                    let left = child("left")?;
                    let right = child("right")?;
                    if left != right {
                        return Err(fail);
                    }
                    let program = generate_structural_program(self.b, self.r, self.c, &left)
                        .map_err(|_| fail.clone())?;
                    if tag == "structural_compare" {
                        if !program.is_total() {
                            return Err(fail);
                        }
                        I32_TYPE_ID.into()
                    } else {
                        BOOL_TYPE_ID.into()
                    }
                }
                "source_project" | "source_reconstruct" => {
                    let value = child(if tag == "source_project" {
                        "source_value"
                    } else {
                        "semantic_value"
                    })?;
                    let (source, semantic) = self
                        .env
                        .bindings
                        .get(text("binding_id")?)
                        .ok_or(fail.clone())?;
                    let (input, output) = if tag == "source_project" {
                        (source, semantic)
                    } else {
                        (semantic, source)
                    };
                    if &value != input {
                        return Err(fail);
                    }
                    output.clone()
                }
                "sequence_length" | "sequence_index" => {
                    let sequence = child("sequence")?;
                    let index = if tag == "sequence_index" {
                        Some(child("index")?)
                    } else {
                        None
                    };
                    let args = require_instance(self.c, &sequence, "bounded_sequence")
                        .map_err(|_| fail.clone())?;
                    if let Some(index) = index {
                        if index != I32_TYPE_ID {
                            return Err(fail);
                        }
                        args[0].clone()
                    } else {
                        I32_TYPE_ID.into()
                    }
                }
                "map_contains" | "map_lookup" => {
                    let map = child("map")?;
                    let key = child("key")?;
                    let args =
                        require_instance(self.c, &map, "ordered_map").map_err(|_| fail.clone())?;
                    if args[0] != key {
                        return Err(fail);
                    }
                    if tag == "map_contains" {
                        BOOL_TYPE_ID.into()
                    } else {
                        closed_type_id(
                            self.b,
                            &ClosedType::Instance {
                                template: "lookup".into(),
                                arguments: vec![concrete_type(self.r, self.c, &args[1])?],
                            },
                        )
                        .map_err(|_| fail.clone())?
                    }
                }
                "set_contains" => {
                    let set = child("set")?;
                    let element = child("element")?;
                    let args =
                        require_instance(self.c, &set, "ordered_set").map_err(|_| fail.clone())?;
                    if args[0] != element {
                        return Err(fail);
                    }
                    BOOL_TYPE_ID.into()
                }
                "tagged_is" | "tagged_payload" => {
                    let value = child("value")?;
                    let payload = self.payload_type(&value, text("arm")?)?;
                    if tag == "tagged_is" {
                        BOOL_TYPE_ID.into()
                    } else {
                        payload.ok_or(fail.clone())?
                    }
                }
                "tagged_make" => {
                    let instance = text("semantic_instance_id")?;
                    let payload = self.payload_type(instance, text("arm")?)?;
                    if v["payload"].is_null() {
                        if payload.is_some() {
                            return Err(fail);
                        }
                    } else {
                        let actual =
                            self.expression(&v["payload"], variables, depth + 1, old, quantifiers)?;
                        if payload.as_ref() != Some(&actual) {
                            return Err(fail);
                        }
                    }
                    instance.into()
                }
                "transition_state" | "transition_events" | "transition_response" => {
                    let value = child("value")?;
                    let args =
                        require_instance(self.c, &value, "transition").map_err(|_| fail.clone())?;
                    match tag {
                        "transition_state" => args[0].clone(),
                        "transition_response" => args[2].clone(),
                        _ => closed_type_id(
                            self.b,
                            &ClosedType::Instance {
                                template: "bounded_sequence".into(),
                                arguments: vec![concrete_type(self.r, self.c, &args[1])?],
                            },
                        )
                        .map_err(|_| fail.clone())?,
                    }
                }
                "parse_error_kind" => {
                    if child("value")? != PARSE_ERROR_TYPE_ID {
                        return Err(fail);
                    }
                    "mpk.csharp.value.u32.v1".to_owned()
                }
                "codec_parse" => {
                    if child("text")? != STRING_TYPE_ID {
                        return Err(fail);
                    }
                    let token = codecs::codec_token(text("codec_id")?).map_err(|_| fail.clone())?;
                    let args =
                        require_instance(self.c, &own, "result").map_err(|_| fail.clone())?;
                    if args
                        != [
                            format!("mpk.csharp.value.{token}.v1"),
                            PARSE_ERROR_TYPE_ID.into(),
                        ]
                    {
                        return Err(fail);
                    }
                    contract_codec(text("codec_id")?, &args[0], &v["codec_parameters"])?;
                    own.clone()
                }
                "codec_format" => {
                    let value = child("value")?;
                    contract_codec(text("codec_id")?, &value, &v["codec_parameters"])?
                        .validate_contract_format_mode(text("mode")?)
                        .map_err(|_| fail.clone())?;
                    STRING_TYPE_ID.into()
                }
                "exception_is" | "exception_payload" => {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T04-W05"))
                }
                _ => return Err(fail),
            };
            if expected != own {
                return Err(fail);
            }
            Ok(own)
        }
        fn payload_type(&self, id: &str, arm: &str) -> Result<Option<String>, DataPhaseError> {
            let meta = self.c.metadata.get(id).ok_or(DataPhaseError::Contract)?;
            let args = &meta.argument_ids;
            match (template_name(&meta.template_id), arm) {
                (Some("option"), "none")
                | (Some("lookup"), "missing_key")
                | (Some("boundary_field"), "missing" | "null") => Ok(None),
                (Some("option"), "some")
                | (Some("lookup"), "found")
                | (Some("result"), "ok")
                | (Some("validation"), "valid")
                | (Some("boundary_field"), "value") => Ok(Some(args[0].clone())),
                (Some("result"), "error") => Ok(Some(args[1].clone())),
                (Some("validation"), "invalid") => Ok(Some(
                    closed_type_id(
                        self.b,
                        &ClosedType::Instance {
                            template: "bounded_sequence".into(),
                            arguments: vec![concrete_type(self.r, self.c, &args[1])?],
                        },
                    )
                    .map_err(|_| DataPhaseError::Contract)?,
                )),
                _ => Err(DataPhaseError::Contract),
            }
        }
    }
    let mut checker = Check {
        b,
        r,
        c,
        env: environment,
        shapes,
        nodes: 0,
    };
    let type_id = checker.expression(value, &mut environment.variables.clone(), 1, false, 0)?;
    Ok(ValidatedDataExpression {
        value: original.clone(),
        type_id,
        nodes: checker.nodes,
    })
}
fn concrete_type(
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
) -> Result<ClosedType, DataPhaseError> {
    if let Some(token) = id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
        .filter(|t| PRIMITIVES.contains(t))
    {
        return Ok(ClosedType::Primitive(token.into()));
    }
    if r.source_types.contains_key(id) {
        return Ok(ClosedType::Source(id.into()));
    }
    let meta = c.metadata.get(id).ok_or(DataPhaseError::Contract)?;
    Ok(ClosedType::Instance {
        template: template_name(&meta.template_id)
            .ok_or(DataPhaseError::Contract)?
            .into(),
        arguments: meta
            .argument_ids
            .iter()
            .map(|id| concrete_type(r, c, id))
            .collect::<Result<_, _>>()?,
    })
}

fn contract_codec(id: &str, ty: &str, value: &Value) -> Result<BoundaryCodec, DataPhaseError> {
    use artifacts::PracticalJsonValue as J;
    let fields = value.as_object().ok_or(DataPhaseError::Contract)?;
    if fields.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != BTreeSet::from(["scale", "rounding"])
    {
        return Err(DataPhaseError::Contract);
    }
    let scale = if value["scale"].is_null() {
        J::Null
    } else {
        J::U64(value["scale"].as_u64().ok_or(DataPhaseError::Contract)?)
    };
    let rounding = if value["rounding"].is_null() {
        J::Null
    } else {
        J::string(value["rounding"].as_str().ok_or(DataPhaseError::Contract)?)
    };
    BoundaryCodec::from_contract_parameters(
        id,
        ty,
        &J::object(vec![("scale", scale), ("rounding", rounding)]),
    )
    .map_err(|_| DataPhaseError::Contract)
}

fn data_contract_limit(id: &str) -> usize {
    static LIMITS: std::sync::OnceLock<BTreeMap<String, usize>> = std::sync::OnceLock::new();
    LIMITS.get_or_init(|| {
        let package: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../develop/specs/vectors/csharp-practical-profile-v1.json"
        )))
        .expect("frozen package");
        package["frozen_contract"]["limits"]["retained_scalar_v0"]
            .as_array()
            .expect("retained counters")
            .iter()
            .chain(
                package["frozen_contract"]["limits"]["practical"]
                    .as_array()
                    .expect("practical counters")
                    .iter(),
            )
            .map(|v| {
                (
                    v["id"].as_str().unwrap().into(),
                    v["inclusive_maximum"].as_u64().unwrap() as usize,
                )
            })
            .collect()
    })[id]
}

/// Transport entry point: reject duplicates and noncanonical spelling before
/// constructing the typed expression. Every expression's field order is the
/// frozen schema order, including nested expressions and codec configuration.
pub fn parse_data_contract_expression(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    env: &DataContractEnvironment,
    bytes: &[u8],
) -> Result<ValidatedDataExpression, DataPhaseError> {
    use artifacts::{PracticalArtifactKind, PracticalJsonValue as J};
    if bytes.len() > data_contract_limit("contract_file_bytes") {
        return Err(DataPhaseError::Contract);
    }
    let value =
        artifacts::parse_canonical_practical_json(PracticalArtifactKind::MethodContract, bytes)
            .map_err(|_| DataPhaseError::Contract)?;
    let package: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../develop/specs/vectors/csharp-practical-profile-v1.json"
    )))
    .expect("frozen package");
    let shapes = package["frozen_contract"]["expression_union"]["variants"]
        .as_array()
        .unwrap();
    fn shape(v: &J, shapes: &[Value]) -> Result<(), DataPhaseError> {
        let fields = v.as_object().ok_or(DataPhaseError::Contract)?;
        let tag = v
            .get("tag")
            .and_then(J::as_str)
            .ok_or(DataPhaseError::Contract)?;
        let variant = shapes
            .iter()
            .find(|s| s["tag"] == tag)
            .ok_or(DataPhaseError::Contract)?;
        if fields
            .iter()
            .map(|(k, _)| k.as_str())
            .ne(variant["ordered_fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap()))
        {
            return Err(DataPhaseError::Contract);
        }
        for (key, value) in fields {
            let ty = variant["field_types"][key].as_str().unwrap();
            match ty {
                "contract_expression" | "contract_expression_bool" => shape(value, shapes)?,
                "contract_expression_or_null" => {
                    if value != &J::Null {
                        shape(value, shapes)?;
                    }
                }
                "ordered_array<contract_expression>" => {
                    for item in value.as_array().ok_or(DataPhaseError::Contract)? {
                        shape(item, shapes)?;
                    }
                }
                "codec_parameters" => {
                    let params = value.as_object().ok_or(DataPhaseError::Contract)?;
                    if params
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .ne(["scale", "rounding"])
                    {
                        return Err(DataPhaseError::Contract);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    shape(&value, shapes)?;
    validate_data_contract_value(b, r, c, env, &value)
}

fn validate_data_contract_value(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    env: &DataContractEnvironment,
    original: &artifacts::PracticalJsonValue,
) -> Result<ValidatedDataExpression, DataPhaseError> {
    use artifacts::PracticalJsonValue as J;
    fn normalize(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        v: &J,
    ) -> Result<Value, DataPhaseError> {
        Ok(match v {
            J::Null => Value::Null,
            J::Bool(v) => Value::Bool(*v),
            J::I64(v) => json!(v),
            J::U64(v) => json!(v),
            J::String(v) => json!(v),
            J::Utf16String(_) => return Err(DataPhaseError::Contract),
            J::Array(v) => Value::Array(
                v.iter()
                    .map(|v| normalize(b, r, c, v))
                    .collect::<Result<_, _>>()?,
            ),
            J::Object(fields) => {
                let literal = v.get("tag").and_then(J::as_str) == Some("literal");
                let mut object = Map::new();
                for (k, val) in fields {
                    let val = if literal && k == "value" {
                        let id = v
                            .get("type_id")
                            .and_then(J::as_str)
                            .ok_or(DataPhaseError::Contract)?;
                        serde_json::to_value(decode_contract_value(b, r, c, id, val)?)
                            .map_err(|_| DataPhaseError::Contract)?
                    } else {
                        normalize(b, r, c, val)?
                    };
                    object.insert(k.clone(), val);
                }
                Value::Object(object)
            }
        })
    }
    let typed = normalize(b, r, c, original)?;
    check_normalized_data_expression(b, r, c, env, &typed, original)
}

/// Attach captured contracts to their original declarations before publishing
/// any artifact. Proof discharge remains T06-owned; attachment is never proof.
pub(crate) fn attach_data_contracts(
    b: &ValidatedFoundationBundle,
    closure: &DataBindingClosure,
    source: &ValidatedDataSource,
    sidecars: &DataSidecars,
    operations: &BTreeMap<String, ClosedOperationSignature>,
) -> Result<(), DataPhaseError> {
    source.validate_source_call_signatures(b, operations)?;
    use artifacts::PracticalJsonValue as J;
    let r = closure.roots();
    let c = closure.closed();
    let mut common = DataContractEnvironment {
        operations: operations.clone(),
        ..Default::default()
    };
    for callable in source.callables() {
        if callable.identity()["kind"] == "constructor" {
            let signature = operations
                .get(callable.id())
                .ok_or(DataPhaseError::Contract)?;
            let logical = if signature.tag == ClosedOperationTag::ConstructorExecute {
                callable.logical_signature(b)?
            } else {
                signature.clone()
            };
            common.constructors.insert(callable.id().into(), logical);
        }
    }
    for ty in r.source_types.values() {
        for member in &ty.members {
            if member.storage != "readonly_field" {
                common.properties.insert(
                    member.id.clone(),
                    (
                        ty.id.clone(),
                        closed_type_id(b, &member.ty).map_err(|_| DataPhaseError::Contract)?,
                    ),
                );
            }
        }
    }
    for row in closure
        .bindings()
        .value()
        .get("bindings")
        .and_then(J::as_array)
        .ok_or(DataPhaseError::Contract)?
    {
        let id = row
            .get("source_type_id")
            .and_then(J::as_str)
            .ok_or(DataPhaseError::Contract)?;
        let hash = row
            .get("binding_sha256")
            .and_then(J::as_str)
            .ok_or(DataPhaseError::Contract)?;
        common.bindings.insert(
            format!("binding.{hash}"),
            (id.into(), closure.projections()[id].clone()),
        );
    }
    fn clause(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        env: &DataContractEnvironment,
        v: &J,
        nodes: &mut usize,
    ) -> Result<(), DataPhaseError> {
        let bytes =
            artifacts::canonical_practical_json_bytes(v).map_err(|_| DataPhaseError::Contract)?;
        let expression = parse_data_contract_expression(b, r, c, env, &bytes)?;
        *nodes = nodes
            .checked_add(expression.nodes())
            .ok_or(DataPhaseError::Contract)?;
        if expression.type_id() != BOOL_TYPE_ID
            || *nodes > data_contract_limit("contract_nodes_per_method")
        {
            return Err(DataPhaseError::Contract);
        }
        Ok(())
    }
    for contract in sidecars.contracts() {
        let value = contract.value();
        let mut env = common.clone();
        let mut nodes = 0;
        if contract.schema() == artifacts::METHOD_CONTRACT_SCHEMA {
            let id = value
                .get("callable_id")
                .and_then(J::as_str)
                .ok_or(DataPhaseError::Contract)?;
            let callable = source
                .callables()
                .iter()
                .find(|f| f.id() == id)
                .ok_or(DataPhaseError::Unreachable)?;
            if value.get("source_content_sha256").and_then(J::as_str)
                != Some(callable.source_sha256())
            {
                return Err(DataPhaseError::Source);
            }
            if !value
                .get("loops")
                .and_then(J::as_array)
                .ok_or(DataPhaseError::Contract)?
                .is_empty()
            {
                return Err(DataPhaseError::LaterOwner("CSHARP-03-T04-W01"));
            }
            if !callable.is_static() && callable.identity()["kind"] != "constructor" {
                env.variables.insert(
                    "this".into(),
                    callable.identity()["owner"]
                        .as_str()
                        .ok_or(DataPhaseError::Contract)?
                        .into(),
                );
            }
            for (ordinal, ty) in callable.parameters().iter().enumerate() {
                env.variables.insert(
                    format!("parameter:{ordinal}"),
                    closed_type_id(
                        b,
                        &ClosedType::parse(ty).map_err(|_| DataPhaseError::Contract)?,
                    )
                    .map_err(|_| DataPhaseError::Contract)?,
                );
            }
            for expression in value
                .get("requires")
                .and_then(J::as_array)
                .ok_or(DataPhaseError::Contract)?
            {
                clause(b, r, c, &env, expression, &mut nodes)?;
            }
            env.allow_old = true;
            let result = closed_type_id(
                b,
                &ClosedType::parse(callable.result()).map_err(|_| DataPhaseError::Contract)?,
            )
            .map_err(|_| DataPhaseError::Contract)?;
            if result != "mpk.csharp.value.unit.v1" {
                env.result = Some(result);
            }
            for expression in value
                .get("ensures")
                .and_then(J::as_array)
                .ok_or(DataPhaseError::Contract)?
            {
                clause(b, r, c, &env, expression, &mut nodes)?;
            }
            env.result = None;
            for exceptional in value
                .get("exceptional_cases")
                .and_then(J::as_array)
                .ok_or(DataPhaseError::Contract)?
            {
                if exceptional
                    .as_object()
                    .ok_or(DataPhaseError::Contract)?
                    .iter()
                    .map(|(k, _)| k.as_str())
                    .ne(["exception_type_id", "path_condition", "ensures"])
                {
                    return Err(DataPhaseError::Contract);
                }
                let id = exceptional
                    .get("exception_type_id")
                    .and_then(J::as_str)
                    .ok_or(DataPhaseError::Contract)?;
                if !builtin_exception_arms().iter().any(|arm| arm.type_id == id) {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T04-W04"));
                }
                clause(
                    b,
                    r,
                    c,
                    &env,
                    exceptional
                        .get("path_condition")
                        .ok_or(DataPhaseError::Contract)?,
                    &mut nodes,
                )?;
                for expression in exceptional
                    .get("ensures")
                    .and_then(J::as_array)
                    .ok_or(DataPhaseError::Contract)?
                {
                    clause(b, r, c, &env, expression, &mut nodes)?;
                }
            }
        } else {
            let id = value
                .get("source_type_id")
                .and_then(J::as_str)
                .ok_or(DataPhaseError::Contract)?;
            let ty = r.source_types.get(id).ok_or(DataPhaseError::Unreachable)?;
            if value.get("source_content_sha256").and_then(J::as_str)
                != Some(ty.source_sha256.as_str())
            {
                return Err(DataPhaseError::Source);
            }
            for (field, expected) in [
                (
                    "ordered_member_ids",
                    ty.members.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
                ),
                (
                    "required_member_ids",
                    ty.members
                        .iter()
                        .filter(|m| m.required)
                        .map(|m| m.id.as_str())
                        .collect(),
                ),
                (
                    "init_member_ids",
                    ty.members
                        .iter()
                        .filter(|m| m.storage == "init_auto")
                        .map(|m| m.id.as_str())
                        .collect(),
                ),
            ] {
                if value
                    .get(field)
                    .and_then(J::as_array)
                    .ok_or(DataPhaseError::Contract)?
                    .iter()
                    .map(J::as_str)
                    .ne(expected.into_iter().map(Some))
                {
                    return Err(DataPhaseError::Contract);
                }
            }
            // This is structural availability of the CLR zero. Declared
            // invariants remain separate, retained VC obligations.
            if value.get("default_eligible") != Some(&J::Bool(source.has_structural_default(id))) {
                return Err(DataPhaseError::Default);
            }
            validate_recursive_default(
                b,
                r,
                c,
                id,
                value
                    .get("recursive_default")
                    .ok_or(DataPhaseError::Default)?,
                &mut 0,
            )?;
            let structural =
                generate_structural_program(b, r, c, id).map_err(|_| DataPhaseError::Routing)?;
            if value.get("structural_equality").and_then(J::as_str) != Some("field_complete")
                || value.get("structural_order").and_then(J::as_str)
                    != Some(if structural.is_total() {
                        "canonical_field_order"
                    } else {
                        "ineligible"
                    })
            {
                return Err(DataPhaseError::Routing);
            }
            env.variables.insert("this".into(), id.into());
            if let Some(expression) = value
                .get("construction_invariant")
                .filter(|v| *v != &J::Null)
            {
                clause(b, r, c, &env, expression, &mut nodes)?;
            }
            for expression in value
                .get("invariants")
                .and_then(J::as_array)
                .ok_or(DataPhaseError::Contract)?
            {
                clause(b, r, c, &env, expression, &mut nodes)?;
            }
        }
    }
    Ok(())
}

/// CLR zeros are metadata even when they are outside the public value domain.
/// Such a zero never passes the public-literal decoder merely by appearing here.
fn validate_recursive_default(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
    value: &artifacts::PracticalJsonValue,
    cells: &mut u64,
) -> Result<(), DataPhaseError> {
    use artifacts::PracticalJsonValue as J;
    *cells = cells.checked_add(1).ok_or(DataPhaseError::Default)?;
    if *cells > TOTAL_VALUE_CELLS_MAX {
        return Err(DataPhaseError::Default);
    }
    if let Some(source) = r.source_types.get(id) {
        return match source.kind {
            SourceKind::SealedClass if value == &J::Null => Ok(()),
            SourceKind::Enum if value.as_str() == Some("0") => Ok(()),
            SourceKind::ReadonlyStruct => {
                let fields = value.as_object().ok_or(DataPhaseError::Default)?;
                if fields
                    .iter()
                    .map(|(name, _)| name)
                    .ne(source.members.iter().map(|m| &m.name))
                {
                    return Err(DataPhaseError::Default);
                }
                for ((_, value), member) in fields.iter().zip(&source.members) {
                    validate_recursive_default(
                        b,
                        r,
                        c,
                        &closed_type_id(b, &member.ty).map_err(|_| DataPhaseError::Default)?,
                        value,
                        cells,
                    )?;
                }
                Ok(())
            }
            _ => Err(DataPhaseError::Default),
        };
    }
    if id == STRING_TYPE_ID
        || c.metadata
            .get(id)
            .is_some_and(|m| template_name(&m.template_id) == Some("bounded_sequence"))
    {
        return if value == &J::Null {
            Ok(())
        } else {
            Err(DataPhaseError::Default)
        };
    }
    let actual = decode_contract_value(b, r, c, id, value)?;
    if actual != domain_default(b, r, c, id).map_err(|_| DataPhaseError::Default)? {
        return Err(DataPhaseError::Default);
    }
    Ok(())
}

/// Codec result roots are inferred from the fixed codec registry and an actual
/// contract expression. Literal payload objects are never treated as syntax.
pub(crate) fn derive_data_contract_roots(
    b: &ValidatedFoundationBundle,
    source: &ValidatedClosedRootSet,
    sidecars: &DataSidecars,
) -> Result<ValidatedClosedRootSet, DataPhaseError> {
    use artifacts::PracticalJsonValue as J;
    let mut value: Value =
        serde_json::from_slice(source.canonical_json()).map_err(|_| DataPhaseError::Source)?;
    let roots = value["roots"]
        .as_array_mut()
        .ok_or(DataPhaseError::Source)?;
    fn walk(
        b: &ValidatedFoundationBundle,
        expression: &J,
        hash: &str,
        ordinal: &mut usize,
        roots: &mut Vec<Value>,
        depth: usize,
    ) -> Result<(), DataPhaseError> {
        let tag = expression.get("tag").and_then(J::as_str);
        let is_expression = tag.is_some();
        if let Some(tag) = tag {
            *ordinal += 1;
            if depth > data_contract_limit("contract_depth")
                || *ordinal > data_contract_limit("contract_nodes_per_method")
            {
                return Err(DataPhaseError::Contract);
            }
            if tag == "literal" {
                return Ok(());
            }
            if tag == "codec_parse" {
                let codec = expression
                    .get("codec_id")
                    .and_then(J::as_str)
                    .ok_or(DataPhaseError::Contract)?;
                let token = codecs::codec_token(codec).map_err(|_| DataPhaseError::Contract)?;
                let ty = ClosedType::Instance {
                    template: "result".into(),
                    arguments: vec![
                        ClosedType::Primitive(token.into()),
                        ClosedType::Primitive("parse_error".into()),
                    ],
                };
                let id = closed_type_id(b, &ty).map_err(|_| DataPhaseError::Contract)?;
                if expression.get("type_id").and_then(J::as_str) != Some(id.as_str()) {
                    return Err(DataPhaseError::Contract);
                }
                roots.push(json!({"origin":"codec_result","provenance_id":format!("contract.{hash}.expression.{ordinal:06}"),"type":ty.to_value()}));
            }
        }
        match expression {
            J::Object(fields) => {
                for (_, child) in fields {
                    if matches!(child, J::Object(_) | J::Array(_)) {
                        walk(
                            b,
                            child,
                            hash,
                            ordinal,
                            roots,
                            depth + usize::from(is_expression),
                        )?;
                    }
                }
            }
            J::Array(elements) => {
                for child in elements {
                    walk(
                        b,
                        child,
                        hash,
                        ordinal,
                        roots,
                        depth + usize::from(is_expression),
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    for contract in sidecars.contracts() {
        let mut ordinal = 0;
        for name in [
            "requires",
            "ensures",
            "exceptional_cases",
            "construction_invariant",
            "invariants",
        ] {
            if let Some(expression) = contract.value().get(name) {
                walk(b, expression, contract.hash(), &mut ordinal, roots, 1)?;
            }
        }
    }
    let bytes = canonical_closed_root_set_transport(b, &value["roots"], &value["source_types"])
        .map_err(|_| DataPhaseError::Contract)?;
    validate_closed_root_set(b, &bytes).map_err(|_| DataPhaseError::Contract)
}
