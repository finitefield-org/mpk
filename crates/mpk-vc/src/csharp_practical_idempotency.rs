//! W05: complete retained snapshots, source equality, and pending replay recipes.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionCheck {
    RetainedKeyLookup,
    SnapshotEquality,
    ExpectedVersion,
    HistoryCapacity,
    VersionExhaustion,
    BusinessErrors,
    NewSuccess,
}
/// A node in the shared structural DAG. Source members include every stored
/// field, even storage not projected by a semantic binding or inactive arm.
#[derive(Clone, Debug)]
pub struct SnapshotEncodingNode {
    recipe: StructuralRecipe,
    members: Vec<String>,
}
impl SnapshotEncodingNode {
    pub fn recipe(&self) -> &StructuralRecipe {
        &self.recipe
    }
    pub fn member_ids(&self) -> &[String] {
        &self.members
    }
}
/// The source helper's result must equal the conjunction of these complete
/// Command/Context relations. These nodes specify obligations, not a proof.
#[derive(Clone, Debug)]
pub struct SnapshotEqualityObligation {
    type_id: String,
    nodes: Vec<SnapshotEncodingNode>,
}
impl SnapshotEqualityObligation {
    pub fn type_id(&self) -> &str {
        &self.type_id
    }
    pub fn nodes(&self) -> &[SnapshotEncodingNode] {
        &self.nodes
    }
    pub fn canonical_encoding_relation(&self) -> &str {
        "canonical_source_field_encoding"
    }
    pub fn discharged(&self) -> bool {
        false
    }
}
#[derive(Clone, Debug)]
pub struct ValidatedIdempotency {
    history_member_id: String,
    record_type_id: String,
    command_key_member_id: String,
    key_type_id: String,
    record_key_member_id: String,
    record_command_member_id: String,
    record_context_member_id: String,
    record_response_member_id: String,
    equality_callable_id: String,
    snapshots: Vec<SnapshotEqualityObligation>,
}
impl ValidatedIdempotency {
    pub fn history_member_id(&self) -> &str {
        &self.history_member_id
    }
    pub fn record_type_id(&self) -> &str {
        &self.record_type_id
    }
    pub fn command_key_member_id(&self) -> &str {
        &self.command_key_member_id
    }
    pub fn key_type_id(&self) -> &str {
        &self.key_type_id
    }
    pub fn record_key_member_id(&self) -> &str {
        &self.record_key_member_id
    }
    pub fn record_command_member_id(&self) -> &str {
        &self.record_command_member_id
    }
    pub fn record_context_member_id(&self) -> &str {
        &self.record_context_member_id
    }
    pub fn record_response_member_id(&self) -> &str {
        &self.record_response_member_id
    }
    pub fn equality_callable_id(&self) -> &str {
        &self.equality_callable_id
    }
    pub fn snapshot_obligations(&self) -> &[SnapshotEqualityObligation] {
        &self.snapshots
    }
    pub fn maximum_history(&self) -> u64 {
        ARRAY_VALUE_LENGTH_MAX
    }
}
pub(super) struct TransitionTypes<'a> {
    pub apply: &'a str,
    pub state: &'a str,
    pub command: &'a str,
    pub context: &'a str,
    pub response: &'a str,
}
fn snapshot(
    b: &ValidatedFoundationBundle,
    closure: &DataBindingClosure,
    id: &str,
) -> Result<SnapshotEqualityObligation, TransitionError> {
    let program = generate_structural_program(b, closure.roots(), closure.closed(), id)
        .map_err(|_| TransitionError::Snapshot)?;
    // The shared total-order eligibility rejects IEEE equality recursively,
    // including inactive source storage and empty containers containing float.
    if !program.is_total() {
        return Err(TransitionError::NonReflexive);
    }
    Ok(SnapshotEqualityObligation {
        type_id: id.into(),
        nodes: program
            .recipes()
            .values()
            .map(|recipe| SnapshotEncodingNode {
                recipe: recipe.clone(),
                members: closure
                    .roots()
                    .source_types
                    .get(&recipe.type_id)
                    .map(|ty| ty.members.iter().map(|m| m.id.clone()).collect())
                    .unwrap_or_default(),
            })
            .collect(),
    })
}
/// Traverse actual source call edges from Apply, not the union of all
/// selected roots. Merely selecting an unrelated equality helper is insufficient.
fn reachable(source: &ValidatedDataSource, apply: &str, helper: &str) -> bool {
    let getters = source
        .callables()
        .iter()
        .filter(|c| c.is_property_getter())
        .filter_map(|c| {
            let name = c.identity()["name"].as_str()?.strip_prefix("get_")?;
            let owner = c.identity()["owner"].as_str()?;
            Some((format!("{owner}.{name}"), c.id().to_owned()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut pending = vec![apply.to_owned()];
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if id == helper {
            return true;
        }
        if let Some(body) = source.body(&id) {
            for op in body {
                if matches!(op.kind(), "Invocation" | "ObjectCreation")
                    && source.body(op.symbol()).is_some()
                {
                    pending.push(op.symbol().to_owned());
                } else if op.kind() == "PropertyReference" {
                    pending.extend(getters.get(op.symbol()).cloned());
                }
            }
        }
    }
    false
}
pub(super) fn attach(
    b: &ValidatedFoundationBundle,
    source: &ValidatedDataSource,
    closure: &DataBindingClosure,
    sidecars: &DataSidecars,
    types: TransitionTypes<'_>,
    value: &J,
) -> Result<Option<ValidatedIdempotency>, TransitionError> {
    use TransitionError as E;
    if text(value, "mode")? == "disabled" {
        exact(value, &["mode"])?;
        return Ok(None);
    }
    exact(
        value,
        &[
            "mode",
            "history_member_id",
            "command_key_member_id",
            "record_key_member_id",
            "record_command_member_id",
            "record_context_member_id",
            "record_response_member_id",
            "equality_callable_id",
        ],
    )?;
    if text(value, "mode")? != "complete_snapshot" {
        return Err(E::Shape);
    }
    let roots = closure.roots();
    let history = text(value, "history_member_id")?;
    let record = match &member(roots, types.state, history)?.ty {
        ClosedType::Instance {
            template,
            arguments,
        } if template == "bounded_sequence" && arguments.len() == 1 => source_id(&arguments[0])?,
        _ => return Err(E::Snapshot),
    };
    let key = text(value, "command_key_member_id")?;
    let key_ty = &member(roots, types.command, key)?.ty;
    let record_key = text(value, "record_key_member_id")?;
    let command = text(value, "record_command_member_id")?;
    let context = text(value, "record_context_member_id")?;
    let response = text(value, "record_response_member_id")?;
    if [record_key, command, context, response]
        .into_iter()
        .collect::<BTreeSet<_>>()
        .len()
        != 4
        || &member(roots, &record, record_key)?.ty != key_ty
        || member(roots, &record, command)?.ty != ClosedType::Source(types.command.into())
        || member(roots, &record, context)?.ty != ClosedType::Source(types.context.into())
        || member(roots, &record, response)?.ty != ClosedType::Source(types.response.into())
    {
        return Err(E::Snapshot);
    }
    let key_type_id = closed_type_id(b, key_ty).map_err(|_| E::Snapshot)?;
    snapshot(b, closure, &key_type_id)?;
    let snapshots = vec![
        snapshot(b, closure, types.command)?,
        snapshot(b, closure, types.context)?,
    ];
    let helper = text(value, "equality_callable_id")?;
    let callable = source
        .callables()
        .iter()
        .find(|c| c.id() == helper)
        .ok_or(E::SnapshotHelper)?;
    let signature = callable
        .logical_signature(b)
        .map_err(|_| E::SnapshotHelper)?;
    if !callable.is_static()
        || callable.identity()["kind"] != "method"
        || signature.argument_type_ids
            != [types.command, types.context, types.command, types.context]
        || signature.normal_result_type_id != BOOL_TYPE_ID
        || !reachable(source, types.apply, helper)
        || !sidecars.contracts().iter().any(|c| {
            c.schema() == a::METHOD_CONTRACT_SCHEMA
                && c.value().get("callable_id").and_then(J::as_str) == Some(helper)
                && c.value().get("termination").and_then(J::as_str) == Some("total")
        })
    {
        return Err(E::SnapshotHelper);
    }
    Ok(Some(ValidatedIdempotency {
        history_member_id: history.into(),
        record_type_id: record,
        command_key_member_id: key.into(),
        key_type_id,
        record_key_member_id: record_key.into(),
        record_command_member_id: command.into(),
        record_context_member_id: context.into(),
        record_response_member_id: response.into(),
        equality_callable_id: helper.into(),
        snapshots,
    }))
}
