//! W04/W05: source-bound pure transitions and pending, path-specific obligations.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
#[path = "csharp_practical_idempotency.rs"]
mod idempotency;
pub use idempotency::{
    SnapshotEncodingNode, SnapshotEqualityObligation, TransitionCheck, ValidatedIdempotency,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    Shape,
    Snapshot,
    SnapshotHelper,
    NonReflexive,
    Method,
    Binding,
    Type,
    Version,
    Time,
    Predicate,
    Coverage,
    Limit,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionPath {
    NewSuccess,
    Replay,
    Error(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionObligationKind {
    InputStateInvariant,
    EffectiveTimeDomain,
    AcceptedCommandCoverage,
    ErrorPrecedence,
    NewStateInvariant,
    CheckedVersionIncrement,
    EventRelation,
    ResponseRelation,
    EventAndValueBounds,
    UnchangedInputState,
    SnapshotEqualityEquivalence,
    RetainedKeyUniqueness,
    ReplayNoEvents,
    ReplayStoredResponse,
    AppendCompleteSnapshot,
    PreserveRetainedHistory,
}
/// Each recipe is tied to a complete source/contract plan. It is a requirement,
/// never a proof result or a caller-provided assertion of successful execution.
#[derive(Clone, Debug)]
pub struct TransitionObligation {
    path: Option<TransitionPath>,
    kind: TransitionObligationKind,
    predicate: Option<ValidatedDataExpression>,
}
impl TransitionObligation {
    pub fn path(&self) -> Option<&TransitionPath> {
        self.path.as_ref()
    }
    pub fn kind(&self) -> &TransitionObligationKind {
        &self.kind
    }
    pub fn predicate(&self) -> Option<&ValidatedDataExpression> {
        self.predicate.as_ref()
    }
    pub fn discharged(&self) -> bool {
        false
    }
}
#[derive(Clone, Debug)]
pub struct TransitionCommandCase {
    id: String,
    condition: ValidatedDataExpression,
}
impl TransitionCommandCase {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn condition(&self) -> &ValidatedDataExpression {
        &self.condition
    }
}
#[derive(Clone, Debug)]
pub struct TransitionBusinessError {
    code: String,
    source_tag: String,
    condition: Option<ValidatedDataExpression>,
}
impl TransitionBusinessError {
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn source_tag(&self) -> &str {
        &self.source_tag
    }
    pub fn condition(&self) -> Option<&ValidatedDataExpression> {
        self.condition.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct TransitionVersionRule {
    state_member_id: String,
    expected_member_id: String,
    effective_member_id: String,
    effective_codec: String,
    raw_instant: bool,
}
impl TransitionVersionRule {
    pub fn state_member_id(&self) -> &str {
        &self.state_member_id
    }
    pub fn expected_member_id(&self) -> &str {
        &self.expected_member_id
    }
    pub fn effective_member_id(&self) -> &str {
        &self.effective_member_id
    }
    pub fn effective_codec(&self) -> &str {
        &self.effective_codec
    }
    pub fn is_raw_instant(&self) -> bool {
        self.raw_instant
    }
    pub fn increment(&self) -> u64 {
        1
    }
}
#[derive(Clone, Debug)]
pub struct ValidatedTransitionContract {
    artifact: a::ValidatedPracticalArtifact,
    callable_id: String,
    state: String,
    command: String,
    context: String,
    event: String,
    response: String,
    error: String,
    result_source: String,
    transition_source: String,
    result_semantic: String,
    transition_semantic: String,
    version: TransitionVersionRule,
    idempotency: Option<ValidatedIdempotency>,
    commands: Vec<TransitionCommandCase>,
    errors: Vec<TransitionBusinessError>,
    obligations: Vec<TransitionObligation>,
}
impl ValidatedTransitionContract {
    pub fn artifact(&self) -> &a::ValidatedPracticalArtifact {
        &self.artifact
    }
    pub fn selected_callable_id(&self) -> &str {
        &self.callable_id
    }
    pub fn state_type_id(&self) -> &str {
        &self.state
    }
    pub fn command_type_id(&self) -> &str {
        &self.command
    }
    pub fn context_type_id(&self) -> &str {
        &self.context
    }
    pub fn event_type_id(&self) -> &str {
        &self.event
    }
    pub fn response_type_id(&self) -> &str {
        &self.response
    }
    pub fn error_type_id(&self) -> &str {
        &self.error
    }
    pub fn result_source_type_id(&self) -> &str {
        &self.result_source
    }
    pub fn transition_source_type_id(&self) -> &str {
        &self.transition_source
    }
    pub fn result_semantic_type_id(&self) -> &str {
        &self.result_semantic
    }
    pub fn transition_semantic_type_id(&self) -> &str {
        &self.transition_semantic
    }
    pub fn version_rule(&self) -> &TransitionVersionRule {
        &self.version
    }
    pub fn accepted_commands(&self) -> &[TransitionCommandCase] {
        &self.commands
    }
    pub fn idempotency(&self) -> Option<&ValidatedIdempotency> {
        self.idempotency.as_ref()
    }
    pub fn check_order(&self) -> &'static [TransitionCheck] {
        use TransitionCheck::*;
        if self.idempotency.is_some() {
            &[
                RetainedKeyLookup,
                SnapshotEquality,
                ExpectedVersion,
                HistoryCapacity,
                VersionExhaustion,
                BusinessErrors,
                NewSuccess,
            ]
        } else {
            &[
                ExpectedVersion,
                VersionExhaustion,
                BusinessErrors,
                NewSuccess,
            ]
        }
    }
    /// Fixed infrastructure errors followed by source-declared business errors.
    /// Replay precedes this list when full snapshots are enabled.
    pub fn errors(&self) -> &[TransitionBusinessError] {
        &self.errors
    }
    pub fn obligations(&self) -> &[TransitionObligation] {
        &self.obligations
    }
    pub fn maximum_events(&self) -> u64 {
        ARRAY_VALUE_LENGTH_MAX
    }
    pub fn maximum_value_cells(&self) -> u64 {
        TOTAL_VALUE_CELLS_MAX
    }
}
fn text<'a>(v: &'a J, key: &str) -> Result<&'a str, TransitionError> {
    v.get(key).and_then(J::as_str).ok_or(TransitionError::Shape)
}
fn exact(v: &J, keys: &[&str]) -> Result<(), TransitionError> {
    if v.as_object()
        .ok_or(TransitionError::Shape)?
        .iter()
        .map(|(k, _)| k.as_str())
        .ne(keys.iter().copied())
    {
        return Err(TransitionError::Shape);
    }
    Ok(())
}
fn named_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 1024
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
}
fn member<'a>(
    r: &'a ValidatedClosedRootSet,
    owner: &str,
    id: &str,
) -> Result<&'a StoredMember, TransitionError> {
    r.source_types
        .get(owner)
        .and_then(|s| s.members.iter().find(|m| m.id == id))
        .ok_or(TransitionError::Type)
}
fn binding<'a>(
    closure: &DataBindingClosure,
    sidecars: &'a DataSidecars,
    id: &str,
    role: &str,
) -> Result<&'a a::SemanticBindingInput, TransitionError> {
    let row = closure
        .bindings()
        .value()
        .get("bindings")
        .and_then(J::as_array)
        .ok_or(TransitionError::Binding)?
        .iter()
        .find(|row| {
            row.get("binding_sha256")
                .and_then(J::as_str)
                .is_some_and(|hash| id == format!("binding.{hash}"))
        })
        .ok_or(TransitionError::Binding)?;
    sidecars
        .bindings()
        .iter()
        .find(|b| {
            Some(b.source_type_id.as_str()) == row.get("source_type_id").and_then(J::as_str)
                && b.role == role
        })
        .ok_or(TransitionError::Binding)
}
fn bound_member<'a>(
    r: &'a ValidatedClosedRootSet,
    binding: &a::SemanticBindingInput,
    role: &str,
) -> Result<&'a StoredMember, TransitionError> {
    member(
        r,
        &binding.source_type_id,
        &binding
            .member_map
            .iter()
            .find(|m| m.role == role)
            .ok_or(TransitionError::Binding)?
            .member_id,
    )
}
fn source_id(ty: &ClosedType) -> Result<String, TransitionError> {
    match ty {
        ClosedType::Source(id) => Ok(id.clone()),
        _ => Err(TransitionError::Type),
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn attach_transition_contracts(
    b: &ValidatedFoundationBundle,
    context: &a::PracticalArtifactContext,
    source: &ValidatedDataSource,
    closure: &DataBindingClosure,
    sidecars: &DataSidecars,
    operations: &BTreeMap<String, ClosedOperationSignature>,
    verification_environment: Option<&DataContractEnvironment>,
) -> Result<Vec<ValidatedTransitionContract>, DataPhaseError> {
    let mut result = vec![];
    let mut ids = BTreeSet::new();
    for artifact in sidecars
        .contracts()
        .iter()
        .filter(|a| a.schema() == a::TRANSITION_CONTRACT_SCHEMA)
    {
        let plan = attach(
            b,
            context,
            source,
            closure,
            sidecars,
            operations,
            artifact,
            verification_environment,
        )?;
        if !ids.insert(
            text(artifact.value(), "transition_id")
                .map_err(DataPhaseError::Transition)?
                .to_owned(),
        ) {
            return Err(DataPhaseError::Transition(TransitionError::Shape));
        }
        result.push(plan);
    }
    Ok(result)
}
#[allow(clippy::too_many_arguments)]
fn attach(
    b: &ValidatedFoundationBundle,
    context: &a::PracticalArtifactContext,
    source: &ValidatedDataSource,
    closure: &DataBindingClosure,
    sidecars: &DataSidecars,
    operations: &BTreeMap<String, ClosedOperationSignature>,
    artifact: &a::ValidatedPracticalArtifact,
    verification_environment: Option<&DataContractEnvironment>,
) -> Result<ValidatedTransitionContract, DataPhaseError> {
    use TransitionError as E;
    let run = || -> Result<ValidatedTransitionContract, E> {
        let v = artifact.value();
        let r = closure.roots();
        let c = closure.closed();
        let id = text(v, "selected_callable_id")?;
        let callable = source
            .callables()
            .iter()
            .find(|f| f.id() == id)
            .ok_or(E::Method)?;
        if !context.selected_root_ids().iter().any(|s| s == id)
            || callable.identity()["name"] != "Apply"
            || callable.identity()["kind"] != "method"
            || !callable.is_static()
            || callable.parameters().len() != 3
        {
            return Err(E::Method);
        }
        let signature = callable.logical_signature(b).map_err(|_| E::Method)?;
        let (state, command, ctx) = (
            text(v, "state_type_id")?,
            text(v, "command_type_id")?,
            text(v, "context_type_id")?,
        );
        if signature.tag != ClosedOperationTag::SourceCall
            || signature.argument_type_ids != [state, command, ctx]
            || !operations.get(id).is_some_and(|actual| {
                actual.tag == signature.tag
                    && actual.argument_type_ids == signature.argument_type_ids
                    && actual.normal_result_type_id == signature.normal_result_type_id
            })
        {
            return Err(E::Method);
        }
        let method = sidecars
            .contracts()
            .iter()
            .find(|a| {
                a.schema() == a::METHOD_CONTRACT_SCHEMA
                    && a.value().get("callable_id").and_then(J::as_str) == Some(id)
            })
            .ok_or(E::Method)?;
        if method.value().get("termination").and_then(J::as_str) != Some("total") {
            return Err(E::Method);
        }
        for ty in [state, command, ctx] {
            if !r.source_types.contains_key(ty) {
                return Err(E::Type);
            }
        }
        let rb = binding(
            closure,
            sidecars,
            text(v, "apply_result_binding_id")?,
            "result",
        )?;
        let tb = binding(
            closure,
            sidecars,
            text(v, "transition_binding_id")?,
            "transition",
        )?;
        if rb.source_type_id != signature.normal_result_type_id
            || source_id(&bound_member(r, rb, "value")?.ty)? != tb.source_type_id
        {
            return Err(E::Binding);
        }
        let error = source_id(&bound_member(r, rb, "error")?.ty)?;
        // The error enum already has a source-bound structural identity. No new
        // semantic-binding role or unfrozen error-enum vocabulary is introduced.
        let en = r
            .source_types
            .get(&error)
            .filter(|s| s.kind == SourceKind::Enum)
            .ok_or(E::Binding)?;
        if text(v, "domain_error_binding_id")? != error {
            return Err(E::Binding);
        }
        if source_id(&bound_member(r, tb, "state")?.ty)? != state {
            return Err(E::Type);
        }
        let event = match &bound_member(r, tb, "events")?.ty {
            ClosedType::Instance {
                template,
                arguments,
            } if template == "bounded_sequence" && arguments.len() == 1 => {
                source_id(&arguments[0])?
            }
            _ => return Err(E::Type),
        };
        let response = source_id(&bound_member(r, tb, "response")?.ty)?;
        let rs = closure
            .projections()
            .get(&rb.source_type_id)
            .ok_or(E::Binding)?;
        let ts = closure
            .projections()
            .get(&tb.source_type_id)
            .ok_or(E::Binding)?;
        if require_instance(c, rs, "result").map_err(|_| E::Binding)? != [ts.clone(), error.clone()]
            || require_instance(c, ts, "transition").map_err(|_| E::Binding)?
                != [state.to_owned(), event.clone(), response.clone()]
        {
            return Err(E::Binding);
        }
        let rule = v.get("version_rule").ok_or(E::Shape)?;
        exact(
            rule,
            &[
                "state_member_id",
                "expected_member_id",
                "increment",
                "effective_time",
            ],
        )?;
        let old = text(rule, "state_member_id")?;
        let expected = text(rule, "expected_member_id")?;
        for (owner, m) in [(state, old), (command, expected)] {
            if member(r, owner, m)?.ty != ClosedType::Primitive("u64".into()) {
                return Err(E::Version);
            }
        }
        if text(rule, "increment")? != "checked_u64_one" {
            return Err(E::Version);
        }
        let time = rule.get("effective_time").ok_or(E::Shape)?;
        exact(time, &["member_id", "codec_id"])?;
        let time_member = text(time, "member_id")?;
        let codec = text(time, "codec_id")?;
        let time_ty = &member(r, ctx, time_member)?.ty;
        let raw = time_ty == &ClosedType::Primitive("i64".into());
        let instant = matches!(time_ty,ClosedType::Source(id)if closure.projections().get(id).is_some_and(|s|s=="mpk.csharp.value.instant.v1"));
        if !((codec == "unix_milliseconds" && (raw || instant))
            || (codec == "date" && time_ty == &ClosedType::Primitive("date".into())))
        {
            return Err(E::Time);
        }
        let idempotency = idempotency::attach(
            b,
            source,
            closure,
            sidecars,
            idempotency::TransitionTypes {
                apply: id,
                state,
                command,
                context: ctx,
                response: &response,
            },
            v.get("idempotency").ok_or(E::Shape)?,
        )?;
        let fixed_errors: &[&str] = if idempotency.is_some() {
            &[
                "idempotency_conflict",
                "version_conflict",
                "history_capacity",
                "version_exhausted",
            ]
        } else {
            &["version_conflict", "version_exhausted"]
        };
        let mut common = if let Some(common) = verification_environment {
            common.clone()
        } else {
            data_phase::data_contract_environment(b, source, closure, operations)
                .map_err(|_| E::Predicate)?
        };
        common.verification_owner = text(artifact.value(), "contract_sha256")?.to_owned();
        let mut env = DataContractEnvironment {
            variables: BTreeMap::from([
                ("state".into(), state.into()),
                ("command".into(), command.into()),
                ("context".into(), ctx.into()),
            ]),
            ..common.clone()
        };
        let mut nodes = 0;
        let mut predicate =
            |e: &J, scope: &DataContractEnvironment| -> Result<ValidatedDataExpression, E> {
                let bytes = a::canonical_practical_json_bytes(e).map_err(|_| E::Predicate)?;
                let e = parse_data_contract_expression(b, r, c, scope, &bytes)
                    .map_err(|_| E::Predicate)?;
                nodes += e.nodes();
                if nodes > data_phase::data_contract_limit("contract_nodes_per_method") {
                    return Err(E::Limit);
                }
                if e.type_id() != BOOL_TYPE_ID {
                    return Err(E::Predicate);
                }
                Ok(e)
            };
        let invariant = predicate(
            v.get("state_invariant").ok_or(E::Shape)?,
            &DataContractEnvironment {
                variables: BTreeMap::from([("state".into(), state.into())]),
                ..common
            },
        )?;
        let mut commands = vec![];
        let mut names = BTreeSet::new();
        for row in v
            .get("accepted_commands")
            .and_then(J::as_array)
            .ok_or(E::Shape)?
        {
            exact(row, &["case_id", "condition"])?;
            let id = text(row, "case_id")?;
            if !named_id(id) || !names.insert(id.to_owned()) {
                return Err(E::Coverage);
            }
            commands.push(TransitionCommandCase {
                id: id.into(),
                condition: predicate(row.get("condition").ok_or(E::Shape)?, &env)?,
            });
        }
        if commands.is_empty() || commands.len() > 256 {
            return Err(E::Limit);
        }
        let mut errors = vec![];
        let mut names = BTreeSet::new();
        let mut carriers = BTreeSet::new();
        for (i, row) in v
            .get("errors")
            .and_then(J::as_array)
            .ok_or(E::Shape)?
            .iter()
            .enumerate()
        {
            exact(row, &["code", "source_tag", "condition"])?;
            let code = text(row, "code")?;
            let tag = text(row, "source_tag")?;
            if !named_id(code)
                || !names.insert(code.to_owned())
                || !carriers.insert(tag.to_owned())
                || !en.enum_values.iter().any(|v| v == tag)
            {
                return Err(E::Coverage);
            }
            let condition = row.get("condition").ok_or(E::Shape)?;
            let condition = if i < fixed_errors.len() {
                if code != fixed_errors[i] || condition != &J::Null {
                    return Err(E::Coverage);
                }
                None
            } else {
                Some(predicate(condition, &env)?)
            };
            errors.push(TransitionBusinessError {
                code: code.into(),
                source_tag: tag.into(),
                condition,
            });
        }
        if errors.len() < fixed_errors.len()
            || errors.len() > 256
            || carriers != en.enum_values.iter().cloned().collect()
        {
            return Err(E::Coverage);
        }
        env.variables.extend([
            ("next_state".into(), state.into()),
            (
                "events".into(),
                closed_type_id(b, &bound_member(r, tb, "events")?.ty).map_err(|_| E::Type)?,
            ),
            ("response".into(), response.clone()),
        ]);
        let event_relation = predicate(v.get("event_relation").ok_or(E::Shape)?, &env)?;
        let response_relation = predicate(v.get("response_relation").ok_or(E::Shape)?, &env)?;
        // Rebind the invariant to the returned state. Keeping its input-state
        // variable here would silently prove the wrong postcondition.
        let next = rebind_invariant(v.get("state_invariant").ok_or(E::Shape)?);
        let mut next_scope = env.clone();
        next_scope.variables = BTreeMap::from([("next_state".into(), state.into())]);
        let next = parse_data_contract_expression(
            b,
            r,
            c,
            &next_scope,
            &a::canonical_practical_json_bytes(&next).map_err(|_| E::Predicate)?,
        )
        .map_err(|_| E::Predicate)?;
        let mut obligations = vec![];
        let mut add = |path, kind, predicate| {
            obligations.push(TransitionObligation {
                path,
                kind,
                predicate,
            })
        };
        add(
            None,
            TransitionObligationKind::InputStateInvariant,
            Some(invariant.clone()),
        );
        add(None, TransitionObligationKind::EffectiveTimeDomain, None);
        add(
            None,
            TransitionObligationKind::AcceptedCommandCoverage,
            None,
        );
        add(None, TransitionObligationKind::ErrorPrecedence, None);
        for (kind, predicate) in [
            (TransitionObligationKind::NewStateInvariant, Some(next)),
            (TransitionObligationKind::CheckedVersionIncrement, None),
            (
                TransitionObligationKind::EventRelation,
                Some(event_relation),
            ),
            (
                TransitionObligationKind::ResponseRelation,
                Some(response_relation),
            ),
            (TransitionObligationKind::EventAndValueBounds, None),
        ] {
            add(Some(TransitionPath::NewSuccess), kind, predicate);
        }
        if idempotency.is_some() {
            add(
                None,
                TransitionObligationKind::SnapshotEqualityEquivalence,
                None,
            );
            add(None, TransitionObligationKind::RetainedKeyUniqueness, None);
            for kind in [
                TransitionObligationKind::UnchangedInputState,
                TransitionObligationKind::ReplayNoEvents,
                TransitionObligationKind::ReplayStoredResponse,
                TransitionObligationKind::EventAndValueBounds,
            ] {
                add(Some(TransitionPath::Replay), kind, None);
            }
            for kind in [
                TransitionObligationKind::AppendCompleteSnapshot,
                TransitionObligationKind::PreserveRetainedHistory,
            ] {
                add(Some(TransitionPath::NewSuccess), kind, None);
            }
        }
        for error in &errors {
            add(
                Some(TransitionPath::Error(error.code.clone())),
                TransitionObligationKind::UnchangedInputState,
                None,
            );
        }
        Ok(ValidatedTransitionContract {
            artifact: artifact.clone(),
            callable_id: id.into(),
            state: state.into(),
            command: command.into(),
            context: ctx.into(),
            event,
            response,
            error,
            result_source: rb.source_type_id.clone(),
            transition_source: tb.source_type_id.clone(),
            result_semantic: rs.clone(),
            transition_semantic: ts.clone(),
            version: TransitionVersionRule {
                state_member_id: old.into(),
                expected_member_id: expected.into(),
                effective_member_id: time_member.into(),
                effective_codec: codec.into(),
                raw_instant: raw,
            },
            idempotency,
            commands,
            errors,
            obligations,
        })
    };
    run().map_err(DataPhaseError::Transition)
}

/// Capture-avoiding substitution of the invariant's sole free state variable.
fn rebind_invariant(original: &J) -> J {
    fn bindings(v: &J, names: &mut BTreeSet<String>) {
        if v.get("tag").and_then(J::as_str) == Some("literal") {
            return;
        }
        if let Some(id) = v.get("binding_id").and_then(J::as_str) {
            names.insert(id.into());
        }
        match v {
            J::Object(fields) => {
                for (_, child) in fields {
                    bindings(child, names);
                }
            }
            J::Array(values) => {
                for child in values {
                    bindings(child, names);
                }
            }
            _ => {}
        }
    }
    let mut names = BTreeSet::new();
    bindings(original, &mut names);
    let fresh = (0..)
        .map(|n| format!("transition.invariant.local.{n}"))
        .find(|name| !names.contains(name))
        .unwrap();
    fn rewrite(v: &mut J, fresh: &str) {
        if v.get("tag").and_then(J::as_str) == Some("literal") {
            return;
        }
        // A next_state identifier in the input-only scope must be local.
        // Rename its declarations and uses before inserting the new free name.
        if let J::Object(fields) = v {
            if let Some((_, J::String(id))) = fields.iter_mut().find(|(k, _)| k == "binding_id") {
                if id == "next_state" {
                    *id = fresh.into();
                } else if id == "state" {
                    *id = "next_state".into();
                }
            }
            for (_, child) in fields {
                rewrite(child, fresh);
            }
        } else if let J::Array(values) = v {
            for child in values {
                rewrite(child, fresh);
            }
        }
    }
    let mut result = original.clone();
    rewrite(&mut result, &fresh);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csharp_03_t05_w04_success_invariant_substitution_avoids_local_capture() {
        let var = |id| {
            J::object(vec![
                ("tag", J::string("variable")),
                ("binding_id", J::string(id)),
            ])
        };
        let invariant = J::object(vec![
            ("tag", J::string("let")),
            ("binding_id", J::string("next_state")),
            ("value", var("state")),
            ("body", var("next_state")),
            ("occupied", var("transition.invariant.local.0")),
        ]);
        let next = rebind_invariant(&invariant);
        assert_eq!(
            next.get("binding_id").and_then(J::as_str),
            Some("transition.invariant.local.1")
        );
        assert_eq!(
            next.get("value")
                .unwrap()
                .get("binding_id")
                .and_then(J::as_str),
            Some("next_state")
        );
        assert_eq!(
            next.get("body")
                .unwrap()
                .get("binding_id")
                .and_then(J::as_str),
            Some("transition.invariant.local.1")
        );
        assert_eq!(
            invariant
                .get("value")
                .unwrap()
                .get("binding_id")
                .and_then(J::as_str),
            Some("state")
        );
    }
}
