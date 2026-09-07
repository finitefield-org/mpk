//! T05-W01: captured boundary contracts attached to the original callable.
//! This is a validation plan, not an input decoder or an invocation capability.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundaryError {
    Shape,
    FieldIdentity,
    FieldType,
    Method,
    Partial,
    Limit,
    Missing,
    Null,
    Payload,
    Default,
    Binding,
    Codec,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundaryMissingRule {
    Reject,
    ExposeMissing,
    FrozenDefault(MonomorphicValue),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundaryValueState {
    Missing,
    Null,
    Value(MonomorphicValue),
}
/// Exact application binding, validated by the cumulative data closure and
/// rechecked by the ordinary VIR binding importer. Payload projection uses that
/// shared closure, including business and collection bindings inside a presence.
#[derive(Clone, Debug)]
pub struct BoundaryPresenceBinding {
    source: a::SemanticBindingInput,
    semantic_type_id: String,
}
impl BoundaryPresenceBinding {
    pub fn source_binding(&self) -> &a::SemanticBindingInput {
        &self.source
    }
    pub fn semantic_type_id(&self) -> &str {
        &self.semantic_type_id
    }
}
#[derive(Clone, Debug)]
pub struct BoundaryField {
    id: String,
    json_name: Vec<u16>,
    source_type_id: String,
    payload_type_id: String,
    required: bool,
    nullable: bool,
    missing: BoundaryMissingRule,
    presence: Option<BoundaryPresenceBinding>,
    codec: Option<BoundaryCodec>,
    raw_instant: bool,
    presence_default: Option<MonomorphicValue>,
}
impl BoundaryField {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn json_name(&self) -> &[u16] {
        &self.json_name
    }
    pub fn source_type_id(&self) -> &str {
        &self.source_type_id
    }
    pub fn payload_type_id(&self) -> &str {
        &self.payload_type_id
    }
    pub fn required(&self) -> bool {
        self.required
    }
    pub fn nullable(&self) -> bool {
        self.nullable
    }
    pub fn missing_rule(&self) -> &BoundaryMissingRule {
        &self.missing
    }
    pub fn presence_binding(&self) -> Option<&BoundaryPresenceBinding> {
        self.presence.as_ref()
    }
    pub fn codec(&self) -> Option<&BoundaryCodec> {
        self.codec.as_ref()
    }
    /// Actual source zero projecting to missing; invariant proof remains pending.
    pub fn presence_default_candidate(&self) -> Option<&MonomorphicValue> {
        self.presence_default.as_ref()
    }
    pub fn is_raw_instant(&self) -> bool {
        self.raw_instant
    }
    /// A typed precondition check only. W02 must still capture and decode bytes;
    /// this function cannot create boundary input evidence or invoke source.
    pub fn check_state(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        state: &BoundaryValueState,
    ) -> Result<(), BoundaryError> {
        match state {
            BoundaryValueState::Missing
                if self.required || self.missing == BoundaryMissingRule::Reject =>
            {
                Err(BoundaryError::Missing)
            }
            BoundaryValueState::Missing => Ok(()),
            BoundaryValueState::Null if !self.nullable => Err(BoundaryError::Null),
            BoundaryValueState::Null => Ok(()),
            BoundaryValueState::Value(value) => {
                if value.type_id() != self.payload_type_id {
                    return Err(BoundaryError::Payload);
                }
                validate_monomorphic_value(b, r, c, value).map_err(|_| BoundaryError::Payload)
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct ValidatedBoundaryContract {
    artifact: a::ValidatedPracticalArtifact,
    selected_callable_id: String,
    inputs: Vec<BoundaryField>,
    outputs: Vec<BoundaryField>,
    obligations: Vec<OutcomeObligation>,
}
impl ValidatedBoundaryContract {
    pub fn artifact(&self) -> &a::ValidatedPracticalArtifact {
        &self.artifact
    }
    pub fn selected_callable_id(&self) -> &str {
        &self.selected_callable_id
    }
    pub fn input_fields(&self) -> &[BoundaryField] {
        &self.inputs
    }
    pub fn output_fields(&self) -> &[BoundaryField] {
        &self.outputs
    }
    /// Fixed profile bounds, never caller-selected overrides in the sidecar.
    pub fn maximum_document_bytes(&self) -> usize {
        1_048_576
    }
    pub fn maximum_value_depth(&self) -> usize {
        32
    }
    pub fn maximum_value_cells(&self) -> u64 {
        TOTAL_VALUE_CELLS_MAX
    }
    /// Source invariants and projection commutation stay pending for T06.
    /// Merely attaching a sidecar never proves a default or permits invocation.
    pub fn obligations(&self) -> &[OutcomeObligation] {
        &self.obligations
    }
}
fn exact<'a>(v: &'a J, names: &[&str]) -> Result<&'a [(String, J)], BoundaryError> {
    let fields = v.as_object().ok_or(BoundaryError::Shape)?;
    if fields
        .iter()
        .map(|(k, _)| k.as_str())
        .ne(names.iter().copied())
    {
        return Err(BoundaryError::Shape);
    }
    Ok(fields)
}
fn text<'a>(v: &'a J, key: &str) -> Result<&'a str, BoundaryError> {
    v.get(key).and_then(J::as_str).ok_or(BoundaryError::Shape)
}
fn boolean(v: &J, key: &str) -> Result<bool, BoundaryError> {
    match v.get(key) {
        Some(J::Bool(x)) => Ok(*x),
        _ => Err(BoundaryError::Shape),
    }
}
fn canonical_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 1024
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"._-".contains(&c))
        && !s
            .as_bytes()
            .windows(2)
            .any(|w| w.iter().all(|c| b"._-".contains(c)))
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.as_bytes()[s.len() - 1].is_ascii_alphanumeric()
}
fn bounded_value(v: &J, depth: usize, cells: &mut u64) -> Result<(), BoundaryError> {
    *cells += 1;
    if depth > 32 || *cells > TOTAL_VALUE_CELLS_MAX {
        return Err(BoundaryError::Limit);
    }
    match v {
        J::Array(xs) => {
            for x in xs {
                bounded_value(x, depth + 1, cells)?;
            }
        }
        J::Object(xs) => {
            for (_, x) in xs {
                bounded_value(x, depth + 1, cells)?;
            }
        }
        _ => {}
    }
    Ok(())
}
// Validate the entire reachable field shape, not just the outer type spelling.
fn admitted(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
    depth: usize,
    cells: &mut u64,
) -> Result<(), BoundaryError> {
    *cells += 1;
    if *cells > TOTAL_VALUE_CELLS_MAX {
        return Err(BoundaryError::Limit);
    }
    if depth > 32 {
        return Err(BoundaryError::Limit);
    }
    if let Some(t) = r.source_types.get(id) {
        for m in &t.members {
            admitted(
                b,
                r,
                c,
                &closed_type_id(b, &m.ty).map_err(|_| BoundaryError::FieldType)?,
                depth + 1,
                cells,
            )?;
        }
        return Ok(());
    }
    if let Some(m) = c.metadata.get(id) {
        if matches!(template_name(&m.template_id), Some("sequence_construction")) {
            return Err(BoundaryError::FieldType);
        }
        for arg in &m.argument_ids {
            admitted(b, r, c, arg, depth + 1, cells)?;
        }
        return Ok(());
    }
    if !is_known_concrete_type(r, c, id)
        || matches!(
            id,
            "mpk.csharp.value.unit.v1" | "mpk.csharp.value.exception.v1"
        )
    {
        return Err(BoundaryError::FieldType);
    }
    Ok(())
}

pub(super) fn attach_boundary_contracts(
    b: &ValidatedFoundationBundle,
    context: &a::PracticalArtifactContext,
    source: &ValidatedDataSource,
    closure: &DataBindingClosure,
    sidecars: &DataSidecars,
    operations: &BTreeMap<String, ClosedOperationSignature>,
) -> Result<Vec<ValidatedBoundaryContract>, BoundaryError> {
    let r = closure.roots();
    let c = closure.closed();
    let mut result = vec![];
    let mut ids = BTreeSet::new();
    for artifact in sidecars
        .contracts()
        .iter()
        .filter(|a| a.schema() == a::BOUNDARY_CONTRACT_SCHEMA)
    {
        let v = artifact.value();
        if !ids.insert(text(v, "boundary_id")?) {
            return Err(BoundaryError::FieldIdentity);
        }
        let id = text(v, "selected_callable_id")?;
        if !context.selected_root_ids().iter().any(|s| s == id) {
            return Err(BoundaryError::Method);
        }
        let callable = source
            .callables()
            .iter()
            .find(|f| f.id() == id)
            .ok_or(BoundaryError::Method)?;
        if callable.identity()["kind"] != "method" {
            return Err(BoundaryError::Method);
        }
        let signature = callable
            .logical_signature(b)
            .map_err(|_| BoundaryError::Method)?;
        let operation = operations.get(id).ok_or(BoundaryError::Method)?;
        if operation.tag != ClosedOperationTag::SourceCall
            || operation.argument_type_ids != signature.argument_type_ids
            || operation.normal_result_type_id != signature.normal_result_type_id
        {
            return Err(BoundaryError::Method);
        }
        // The required method contract is captured and source-hash checked by
        // attach_data_contracts. Propagated partial callees are rejected there.
        let method = sidecars
            .contracts()
            .iter()
            .find(|a| {
                a.schema() == a::METHOD_CONTRACT_SCHEMA
                    && a.value().get("callable_id").and_then(J::as_str) == Some(id)
            })
            .ok_or(BoundaryError::Method)?;
        if method.value().get("termination").and_then(J::as_str) != Some("total") {
            return Err(BoundaryError::Partial);
        }
        // Keep dependent payload bindings as well as the outer presence.
        let mut obligations = closure.obligations().to_vec();
        let mut attach = |key: &str,
                          types: &[String]|
         -> Result<Vec<BoundaryField>, BoundaryError> {
            let rows = v
                .get(key)
                .and_then(J::as_array)
                .ok_or(BoundaryError::Shape)?;
            if rows.len() > 256 {
                return Err(BoundaryError::Limit);
            }
            if rows.len() != types.len() {
                return Err(BoundaryError::Method);
            }
            let mut field_ids = BTreeSet::new();
            let mut names = BTreeSet::new();
            let mut fields = vec![];
            for (row, source_type) in rows.iter().zip(types) {
                exact(
                    row,
                    &[
                        "field_id",
                        "json_name",
                        "type_id",
                        "required",
                        "nullable",
                        "missing_rule",
                        "codec_id",
                        "codec_parameters",
                    ],
                )?;
                let field_id = text(row, "field_id")?;
                let json_name = match row.get("json_name") {
                    Some(J::String(s)) => s.encode_utf16().collect::<Vec<_>>(),
                    Some(J::Utf16String(s)) => s.clone(),
                    _ => return Err(BoundaryError::Shape),
                };
                if !canonical_id(field_id)
                    || !field_ids.insert(field_id)
                    || !names.insert(json_name.clone())
                {
                    return Err(BoundaryError::FieldIdentity);
                }
                if json_name.len() > 16_384 {
                    return Err(BoundaryError::Limit);
                }
                if text(row, "type_id")? != source_type {
                    return Err(BoundaryError::FieldType);
                }
                admitted(b, r, c, source_type, 0, &mut 0)?;
                let projected = closure
                    .projections()
                    .get(source_type)
                    .unwrap_or(source_type);
                admitted(b, r, c, projected, 0, &mut 0)?;
                let meta = c.metadata.get(projected);
                let presence = if meta
                    .is_some_and(|m| template_name(&m.template_id) == Some("boundary_field"))
                {
                    let binding = sidecars
                        .bindings()
                        .iter()
                        .find(|i| &i.source_type_id == source_type && i.role == "boundary_field")
                        .ok_or(BoundaryError::Binding)?;
                    let model = OutcomeModel::new(b, r, c, projected)
                        .map_err(|_| BoundaryError::Binding)?;
                    if model.role() != "boundary_field"
                        || model.payload_type_ids() != binding.inferred_argument_ids
                    {
                        return Err(BoundaryError::Binding);
                    }
                    Some(BoundaryPresenceBinding {
                        source: binding.clone(),
                        semantic_type_id: projected.clone(),
                    })
                } else {
                    None
                };
                let option = meta.is_some_and(|m| template_name(&m.template_id) == Some("option"));
                let payload = if presence.is_some() || option {
                    meta.unwrap().argument_ids[0].clone()
                } else {
                    projected.clone()
                };
                if presence.is_some()
                    && c.metadata.get(&payload).is_some_and(|m| {
                        matches!(
                            template_name(&m.template_id),
                            Some("option" | "boundary_field")
                        )
                    })
                {
                    return Err(BoundaryError::Binding);
                }
                let required = boolean(row, "required")?;
                let nullable = boolean(row, "nullable")?;
                if nullable && presence.is_none() && !option {
                    return Err(BoundaryError::Null);
                }
                let codec_id = match row.get("codec_id") {
                    Some(J::Null) => None,
                    Some(J::String(s)) => Some(s.as_str()),
                    _ => return Err(BoundaryError::Shape),
                };
                // The frozen codec field is the sole explicit classification of
                // an otherwise ordinary Int64 as a raw Unix-millisecond carrier.
                let raw_instant = source_type == "mpk.csharp.value.i64.v1"
                    && codec_id == Some("unix_milliseconds");
                let codec_type = if raw_instant {
                    "mpk.csharp.value.instant.v1"
                } else {
                    &payload
                };
                let codec = BoundaryCodec::from_optional_contract_parameters(
                    codec_id,
                    codec_type,
                    row.get("codec_parameters").ok_or(BoundaryError::Shape)?,
                )
                .map_err(|_| BoundaryError::Codec)?;
                let missing = row.get("missing_rule").ok_or(BoundaryError::Shape)?;
                let rule = match text(missing, "mode")? {
                    "reject" => {
                        exact(missing, &["mode"])?;
                        if !required {
                            return Err(BoundaryError::Missing);
                        }
                        BoundaryMissingRule::Reject
                    }
                    "expose_missing" => {
                        exact(missing, &["mode"])?;
                        if required || presence.is_none() {
                            return Err(BoundaryError::Missing);
                        }
                        BoundaryMissingRule::ExposeMissing
                    }
                    "use_frozen_typed_default" => {
                        exact(missing, &["mode", "value"])?;
                        if required {
                            return Err(BoundaryError::Default);
                        }
                        let value = missing.get("value").ok_or(BoundaryError::Default)?;
                        bounded_value(value, 0, &mut 0)?;
                        if a::canonical_practical_json_bytes(value)
                            .map_err(|_| BoundaryError::Default)?
                            .len()
                            > 1_048_576
                        {
                            return Err(BoundaryError::Limit);
                        }
                        let decoded = data_phase::decode_boundary_default(
                            b,
                            r,
                            c,
                            projected,
                            value,
                            codec.as_ref(),
                        )
                        .map_err(|_| BoundaryError::Default)?;
                        // An omission default cannot collapse into explicit
                        // null. A presence missing arm remains distinct.
                        if matches!(
                            decoded,
                            MonomorphicValue::Option {
                                arm: OptionArm::None,
                                ..
                            }
                        ) {
                            return Err(BoundaryError::Default);
                        }
                        if matches!(
                            decoded,
                            MonomorphicValue::BoundaryPresence {
                                arm: BoundaryArm::Null,
                                ..
                            }
                        ) {
                            return Err(BoundaryError::Default);
                        }
                        obligations.push(OutcomeObligation {
                            source_type_id: source_type.clone(),
                            semantic_type_id: projected.clone(),
                            kind: "boundary_frozen_default_public_invariant".into(),
                            member_id: format!("{}.{key}.{field_id}", text(v, "boundary_id")?),
                            discharged: false,
                        });
                        BoundaryMissingRule::FrozenDefault(decoded)
                    }
                    _ => return Err(BoundaryError::Shape),
                };
                let presence_default = presence.as_ref().and_then(|p| {
                    let value =
                        domain::default_with_obligations(b, r, c, source_type, false).ok()?;
                    let MonomorphicValue::Product { fields, .. } = &value else {
                        return None;
                    };
                    let tag_id = &p
                        .source
                        .member_map
                        .iter()
                        .find(|m| m.role == "tag")?
                        .member_id;
                    let tag_name = &r.source_types[source_type]
                        .members
                        .iter()
                        .find(|m| &m.id == tag_id)?
                        .name;
                    let tag = fields.iter().find(|f| &f.name == tag_name)?;
                    let MonomorphicValue::Enum { carrier, .. } = tag.value.as_ref() else {
                        return None;
                    };
                    (carrier
                        == &p
                            .source
                            .tag_arms
                            .iter()
                            .find(|a| a.semantic_arm == "missing")?
                            .source_tag)
                        .then_some(value)
                });
                if presence_default.is_some() {
                    obligations.push(OutcomeObligation {
                        source_type_id: source_type.clone(),
                        semantic_type_id: projected.clone(),
                        kind: "boundary_actual_default_public_invariant".into(),
                        member_id: format!("{}.{key}.{field_id}", text(v, "boundary_id")?),
                        discharged: false,
                    });
                }
                fields.push(BoundaryField {
                    id: field_id.into(),
                    json_name,
                    source_type_id: source_type.clone(),
                    payload_type_id: payload,
                    required,
                    nullable,
                    missing: rule,
                    presence,
                    codec,
                    raw_instant,
                    presence_default,
                });
            }
            Ok(fields)
        };
        let inputs = attach("input_fields", &signature.argument_type_ids)?;
        let output_types = if signature.normal_result_type_id == "mpk.csharp.value.unit.v1" {
            vec![]
        } else {
            vec![signature.normal_result_type_id]
        };
        let outputs = attach("output_fields", &output_types)?;
        result.push(ValidatedBoundaryContract {
            artifact: artifact.clone(),
            selected_callable_id: id.into(),
            inputs,
            outputs,
            obligations,
        });
    }
    Ok(result)
}
