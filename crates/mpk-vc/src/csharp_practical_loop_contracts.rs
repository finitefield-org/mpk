//! T04-W01: retained sidecar attachment to Roslyn's original-source loop sites.
//! This is a private frontend handoff, not VIR acceptance or proof discharge.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopContractError {
    Source,
    Sidecar,
    Attachment,
    Scope,
    Expression,
    Modifies,
    MissingDecreases,
    Limit(&'static str),
}
impl LoopContractError {
    pub fn diagnostic_phase(&self) -> u8 {
        if matches!(self, Self::Limit(_)) {
            0
        } else {
            7
        }
    }
    pub fn diagnostic_family(&self) -> &'static str {
        if matches!(self, Self::Limit(_)) {
            "CSHARP_PRACTICAL_LIMIT"
        } else {
            "CSHARP_PRACTICAL_LOOP_CONTRACT"
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    compilation_id: String,
    selected_root_ids: Vec<String>,
    sources: Vec<SourceFile>,
    methods: Vec<Method>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    path: String,
    raw_sha256: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Method {
    callable_id: String,
    source_path: String,
    source_content_sha256: String,
    start_byte: usize,
    end_byte: usize,
    parameters: Vec<Binding>,
    locals: Vec<Binding>,
    allocations: Vec<LoopArrayAllocation>,
    result_type_id: String,
    loops: Vec<Loop>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    type_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Loop {
    loop_id: String,
    parent: Option<String>,
    kind: String,
    start_byte: usize,
    end_byte: usize,
    variables: Vec<Binding>,
    modifies: Vec<String>,
    read_borrows: Vec<String>,
    array_writes: Vec<String>,
    exits: Vec<LoopExit>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopExit {
    pub kind: String,
    pub target: String,
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoopArrayAllocation {
    pub binding_id: String,
    pub type_id: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub length_start_byte: usize,
    pub length_end_byte: usize,
    pub length_binding: Option<String>,
}
/// The predicates below are tied to retained storage/length source operands,
/// never unattached labels or a claim that the collection is already valid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopCollectionClause {
    pub subject_id: String,
    pub operation: String,
    pub member_ids: Vec<String>,
    pub allocation: Option<LoopArrayAllocation>,
    pub predicates: Vec<String>,
    pub supporting_invariants: Vec<J>,
}
/// All claims are pending obligations. No constructor accepts a discharged bit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachedLoopContract {
    pub loop_id: String,
    pub parent: Option<String>,
    pub termination: String,
    pub invariants: Vec<J>,
    pub decreases: Vec<J>,
    pub modifies: Vec<String>,
    pub read_borrows: Vec<String>,
    pub array_writes: Vec<String>,
    pub abrupt_exits: Vec<LoopExit>,
    pub normal_exit_claims: Vec<J>,
    pub return_claims: Vec<J>,
    pub exceptional_claims: Vec<J>,
    pub obligations: Vec<String>,
    pub collection_clauses: Vec<LoopCollectionClause>,
}
#[derive(Clone, Debug)]
pub struct PreparedLoopContracts {
    exceptional_cases: BTreeMap<String, Vec<J>>,
    loops: Vec<AttachedLoopContract>,
    snapshot_sha256: String,
}
impl PreparedLoopContracts {
    pub fn exceptional_cases(&self) -> &BTreeMap<String, Vec<J>> {
        &self.exceptional_cases
    }
    pub fn loops(&self) -> &[AttachedLoopContract] {
        &self.loops
    }
    pub fn snapshot_sha256(&self) -> &str {
        &self.snapshot_sha256
    }
    pub fn artifact_count(&self) -> usize {
        0
    }
}
fn array<'a>(value: &'a J, name: &str) -> Result<&'a [J], LoopContractError> {
    value
        .get(name)
        .and_then(J::as_array)
        .ok_or(LoopContractError::Sidecar)
}
fn text<'a>(value: &'a J, name: &str) -> Result<&'a str, LoopContractError> {
    value
        .get(name)
        .and_then(J::as_str)
        .ok_or(LoopContractError::Sidecar)
}
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
fn environment(
    common: &DataContractEnvironment,
    bindings: &[Binding],
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
) -> Result<DataContractEnvironment, LoopContractError> {
    let mut env = common.clone();
    env.variables.clear();
    env.result = None;
    env.exception_type = None;
    env.allow_old = false;
    for binding in bindings {
        if !known_concrete_type(r, c, &binding.type_id)
            || env
                .variables
                .insert(binding.id.clone(), binding.type_id.clone())
                .is_some()
        {
            return Err(LoopContractError::Scope);
        }
    }
    Ok(env)
}
fn expressions(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    env: &DataContractEnvironment,
    clauses: &[J],
    decreases: bool,
    nodes: &mut usize,
) -> Result<Vec<J>, LoopContractError> {
    let mut result = Vec::new();
    for clause in clauses {
        let bytes =
            a::canonical_practical_json_bytes(clause).map_err(|_| LoopContractError::Expression)?;
        let value = parse_data_contract_expression(b, r, c, env, &bytes)
            .map_err(|_| LoopContractError::Expression)?;
        let valid = if decreases {
            // Frozen finite integral carriers ordered by mathematical value;
            // non-negativity and strict lexicographic decrease remain VCs.
            matches!(
                value.type_id(),
                "mpk.csharp.value.i8.v1"
                    | "mpk.csharp.value.u8.v1"
                    | "mpk.csharp.value.i16.v1"
                    | "mpk.csharp.value.u16.v1"
                    | "mpk.csharp.value.i32.v1"
                    | "mpk.csharp.value.u32.v1"
                    | "mpk.csharp.value.i64.v1"
                    | "mpk.csharp.value.u64.v1"
            )
        } else {
            value.type_id() == BOOL_TYPE_ID
        };
        *nodes = nodes
            .checked_add(value.nodes())
            .ok_or(LoopContractError::Limit("contract_nodes_per_method"))?;
        if *nodes > data_phase::data_contract_limit("contract_nodes_per_method") {
            return Err(LoopContractError::Limit("contract_nodes_per_method"));
        }
        if !valid {
            return Err(LoopContractError::Expression);
        }
        result.push(value.value().clone());
    }
    Ok(result)
}

/// Parse all selected method sidecars before any lowering. `source_bytes` must
/// be the captured Roslyn loop-source handoff; it is independently bound here
/// to the original snapshot. W02 owns CFG reconstruction and alias analysis.
/// `common` carries only T03's validated operation/member/binding signatures;
/// subject variables and result/old permissions are always regenerated here.
#[allow(clippy::too_many_arguments)]
pub fn prepare_loop_contracts(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    context: &a::PracticalArtifactContext,
    captures: &a::CapturedInputSet,
    source_bytes: &[u8],
    common: &DataContractEnvironment,
) -> Result<PreparedLoopContracts, LoopContractError> {
    if source_bytes.len() > a::PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX {
        return Err(LoopContractError::Source);
    }
    let source: Source =
        serde_json::from_slice(source_bytes).map_err(|_| LoopContractError::Source)?;
    if source.compilation_id != context.compilation_id()
        || source.selected_root_ids != context.selected_root_ids()
        || source
            .sources
            .iter()
            .map(|s| &s.path)
            .ne(context.source_paths())
    {
        return Err(LoopContractError::Source);
    }
    for file in &source.sources {
        if captures
            .entry(&file.path)
            .is_none_or(|e| e.raw_sha256() != file.raw_sha256)
        {
            return Err(LoopContractError::Source);
        }
    }
    let mut methods = BTreeMap::new();
    // Structural budgets precede sidecar expression checks, including in
    // methods that have missing or malformed contracts.
    for method in &source.methods {
        if method.loops.len() > 32 {
            return Err(LoopContractError::Limit("loops_per_method"));
        }
        if methods.insert(method.callable_id.clone(), method).is_some()
            || !method.callable_id.starts_with("mpk.csharp.source.")
            || !context.source_paths().contains(&method.source_path)
        {
            return Err(LoopContractError::Source);
        }
        let original = captures
            .entry(&method.source_path)
            .ok_or(LoopContractError::Source)?;
        if original.raw_sha256() != method.source_content_sha256
            || method.start_byte >= method.end_byte
            || method.end_byte > original.bytes().len()
        {
            return Err(LoopContractError::Source);
        }
        let all_bindings = method
            .parameters
            .iter()
            .chain(&method.locals)
            .cloned()
            .collect::<Vec<_>>();
        let method_env = environment(common, &all_bindings, r, c)?;
        for allocation in &method.allocations {
            if method_env.variables.get(&allocation.binding_id) != Some(&allocation.type_id)
                || c.metadata
                    .get(&allocation.type_id)
                    .is_none_or(|m| template_name(&m.template_id) != Some("bounded_sequence"))
                || allocation.start_byte < method.start_byte
                || allocation.start_byte >= allocation.end_byte
                || allocation.end_byte > method.end_byte
                || allocation.length_start_byte < allocation.start_byte
                || allocation.length_start_byte > allocation.length_end_byte
                || allocation.length_end_byte > allocation.end_byte
                || allocation
                    .length_binding
                    .as_ref()
                    .is_some_and(|id| !method_env.variables.contains_key(id))
            {
                return Err(LoopContractError::Source);
            }
        }
        let mut stack: Vec<&Loop> = vec![];
        let mut previous = None;
        for (ordinal, loop_) in method.loops.iter().enumerate() {
            if loop_.loop_id != format!("{}#loop#{ordinal:04}", method.callable_id)
                || loop_.start_byte < method.start_byte
                || loop_.start_byte >= loop_.end_byte
                || loop_.end_byte > method.end_byte
                || previous.is_some_and(|p| p >= loop_.start_byte)
                || !matches!(loop_.kind.as_str(), "for" | "while" | "do" | "foreach")
                || !sorted_unique(&loop_.modifies)
                || !sorted_unique(&loop_.array_writes)
                || !sorted_unique(&loop_.read_borrows)
            {
                return Err(LoopContractError::Source);
            }
            for binding in &loop_.variables {
                if method_env.variables.get(&binding.id) != Some(&binding.type_id) {
                    return Err(LoopContractError::Scope);
                }
            }
            for id in &loop_.modifies {
                if !method_env
                    .variables
                    .contains_key(id.strip_prefix("construction:").unwrap_or(id))
                {
                    return Err(LoopContractError::Modifies);
                }
            }
            for id in &loop_.array_writes {
                if !loop_.modifies.contains(&format!("construction:{id}"))
                    || method_env
                        .variables
                        .get(id)
                        .and_then(|ty| c.metadata.get(ty))
                        .is_none_or(|m| template_name(&m.template_id) != Some("bounded_sequence"))
                {
                    return Err(LoopContractError::Modifies);
                }
            }
            previous = Some(loop_.start_byte);
            while stack.last().is_some_and(|p| p.end_byte <= loop_.start_byte) {
                stack.pop();
            }
            if loop_.parent.as_deref() != stack.last().map(|p| p.loop_id.as_str())
                || stack.last().is_some_and(|p| loop_.end_byte > p.end_byte)
            {
                return Err(LoopContractError::Source);
            }
            stack.push(loop_);
            if stack.len() > 8 {
                return Err(LoopContractError::Limit("loop_nesting"));
            }
            let source_text =
                std::str::from_utf8(&original.bytes()[loop_.start_byte..loop_.end_byte])
                    .map_err(|_| LoopContractError::Source)?;
            if !source_text.starts_with(&loop_.kind)
                || source_text[loop_.kind.len()..]
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
            {
                return Err(LoopContractError::Source);
            }
            for exit in &loop_.exits {
                if exit.start_byte < loop_.start_byte
                    || exit.start_byte >= exit.end_byte
                    || exit.end_byte > loop_.end_byte
                    || !matches!(
                        exit.kind.as_str(),
                        "break" | "continue" | "return" | "throw"
                    )
                    || exit.target
                        != if matches!(exit.kind.as_str(), "break" | "continue") {
                            &loop_.loop_id
                        } else {
                            &method.callable_id
                        }
                        .as_str()
                {
                    return Err(LoopContractError::Source);
                }
            }
        }
    }
    if context
        .selected_root_ids()
        .iter()
        .any(|id| !methods.contains_key(id))
    {
        return Err(LoopContractError::Source);
    }
    let sidecars =
        DataSidecars::capture(context, captures).map_err(|_| LoopContractError::Sidecar)?;
    // Collection roles must come from the same independently derived T03
    // source-to-semantic projection, never a sidecar label alone.
    let reachable = methods
        .keys()
        .cloned()
        .chain(r.source_types.keys().cloned())
        .collect();
    let binding_closure =
        DataBindingClosure::derive(b, context, captures, r, sidecars.bindings(), &reachable)
            .map_err(|_| LoopContractError::Attachment)?;
    for binding in sidecars.bindings() {
        if !matches!(binding.role.as_str(), "ordered_map" | "ordered_set") {
            continue;
        }
        let semantic = &binding_closure.projections()[&binding.source_type_id];
        let model = OrderedCollectionModel::new(
            b,
            binding_closure.roots(),
            binding_closure.closed(),
            semantic,
        )
        .map_err(|_| LoopContractError::Attachment)?;
        for operation in &binding.operation_map {
            let method = methods
                .get(&operation.member_id)
                .ok_or(LoopContractError::Attachment)?;
            let signature = model
                .operations()
                .iter()
                .find(|op| op.name == operation.operation)
                .ok_or(LoopContractError::Attachment)?;
            let project = |id: &str| {
                binding_closure
                    .projections()
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| id.to_owned())
            };
            if method
                .parameters
                .iter()
                .map(|p| project(&p.type_id))
                .collect::<Vec<_>>()
                != signature.argument_type_ids
                || project(&method.result_type_id) != signature.result_type_id
            {
                return Err(LoopContractError::Attachment);
            }
        }
    }
    let r = binding_closure.roots();
    let c = binding_closure.closed();
    let mut expression_context = common.clone();
    expression_context.bindings.clear();
    expression_context.properties.clear();
    for row in binding_closure
        .bindings()
        .value()
        .get("bindings")
        .and_then(J::as_array)
        .ok_or(LoopContractError::Sidecar)?
    {
        let id = text(row, "source_type_id")?;
        expression_context.bindings.insert(
            format!("binding.{}", text(row, "binding_sha256")?),
            (id.into(), binding_closure.projections()[id].clone()),
        );
    }
    for source_type in r.source_types.values() {
        for member in &source_type.members {
            if member.storage != "readonly_field" {
                expression_context.properties.insert(
                    member.id.clone(),
                    (
                        source_type.id.clone(),
                        closed_type_id(b, &member.ty).map_err(|_| LoopContractError::Scope)?,
                    ),
                );
            }
        }
    }
    let common = &expression_context;
    let mut contracts = BTreeMap::new();
    for contract in sidecars
        .contracts()
        .iter()
        .filter(|v| v.schema() == a::METHOD_CONTRACT_SCHEMA)
    {
        let value = contract.value();
        let id = text(value, "callable_id")?;
        if !methods.contains_key(id) || contracts.insert(id, value).is_some() {
            return Err(LoopContractError::Attachment);
        }
        // Scan every loop budget before typing any clause in this selection.
        for row in array(value, "loops")? {
            if array(row, "invariants")?
                .len()
                .saturating_add(array(row, "decreases")?.len())
                > 64
            {
                return Err(LoopContractError::Limit("invariant_decreases_per_loop"));
            }
        }
    }
    let mut attached = Vec::new();
    let mut exceptional_cases = BTreeMap::new();
    for method in &source.methods {
        let Some(contract) = contracts.get(method.callable_id.as_str()) else {
            if !method.loops.is_empty() {
                return Err(LoopContractError::Attachment);
            }
            continue;
        };
        if text(contract, "source_content_sha256")? != method.source_content_sha256 {
            return Err(LoopContractError::Attachment);
        }
        let rows = array(contract, "loops")?;
        if rows.len() != method.loops.len() {
            return Err(LoopContractError::Attachment);
        }
        let termination = text(contract, "termination")?;
        let mut nodes = 0;
        let mut env = environment(common, &method.parameters, r, c)?;
        expressions(
            b,
            r,
            c,
            &env,
            array(contract, "requires")?,
            false,
            &mut nodes,
        )?;
        env.allow_old = true;
        if method.result_type_id != "mpk.csharp.value.unit.v1" {
            if !known_concrete_type(r, c, &method.result_type_id) {
                return Err(LoopContractError::Scope);
            }
            env.result = Some(method.result_type_id.clone());
        }
        let ensures = expressions(
            b,
            r,
            c,
            &env,
            array(contract, "ensures")?,
            false,
            &mut nodes,
        )?;
        env.result = None;
        let exceptional = array(contract, "exceptional_cases")?;
        for case in exceptional {
            if case
                .as_object()
                .ok_or(LoopContractError::Sidecar)?
                .iter()
                .map(|(k, _)| k.as_str())
                .ne(["exception_type_id", "path_condition", "ensures"])
                || !common.exception_universe.as_ref().map_or_else(
                    || {
                        builtin_exception_arms()
                            .iter()
                            .any(|arm| arm.type_id == text(case, "exception_type_id").unwrap_or(""))
                    },
                    |universe| {
                        universe
                            .arm(text(case, "exception_type_id").unwrap_or(""))
                            .is_some()
                    },
                )
            {
                return Err(LoopContractError::Expression);
            }
            if common.exception_universe.is_some() {
                env.exception_type = Some(text(case, "exception_type_id")?.into());
                env.variables
                    .insert("exception".into(), EXCEPTION_TYPE_ID.into());
            }
            expressions(
                b,
                r,
                c,
                &env,
                std::slice::from_ref(
                    case.get("path_condition")
                        .ok_or(LoopContractError::Expression)?,
                ),
                false,
                &mut nodes,
            )?;
            expressions(b, r, c, &env, array(case, "ensures")?, false, &mut nodes)?;
        }
        env.exception_type = None;
        env.variables.remove("exception");
        exceptional_cases.insert(method.callable_id.clone(), exceptional.to_vec());
        for (loop_, row) in method.loops.iter().zip(rows) {
            let fields = row.as_object().ok_or(LoopContractError::Sidecar)?;
            if fields.iter().map(|(k, _)| k.as_str()).ne([
                "loop_id",
                "invariants",
                "modifies",
                "decreases",
            ]) || text(row, "loop_id")? != loop_.loop_id
                || array(row, "invariants")?.is_empty()
            {
                return Err(LoopContractError::Attachment);
            }
            let modifies = array(row, "modifies")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or(LoopContractError::Modifies)
                })
                .collect::<Result<Vec<_>, _>>()?;
            // Sidecar order is preserved; set equality, not spelling or source
            // variable names, establishes the exact transitive modification set.
            if modifies.iter().collect::<BTreeSet<_>>().len() != modifies.len()
                || modifies.iter().collect::<BTreeSet<_>>() != loop_.modifies.iter().collect()
            {
                return Err(LoopContractError::Modifies);
            }
            if termination == "total" && array(row, "decreases")?.is_empty() {
                return Err(LoopContractError::MissingDecreases);
            }
            let env = environment(common, &loop_.variables, r, c)?;
            let invariants =
                expressions(b, r, c, &env, array(row, "invariants")?, false, &mut nodes)?;
            let decreases = expressions(b, r, c, &env, array(row, "decreases")?, true, &mut nodes)?;
            let mut obligations: BTreeSet<String> = [
                "invariant_entry",
                "invariant_backedge",
                "normal_exit",
                "abrupt_exit",
                "modifies_frame",
                "ownership_frame",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect();
            if !decreases.is_empty() {
                obligations.extend([
                    "decreases_nonnegative".into(),
                    "decreases_lexicographic".into(),
                ]);
            }
            let mut collection_clauses = Vec::new();
            for allocation in &method.allocations {
                let fill = loop_.array_writes.contains(&allocation.binding_id);
                let count = allocation.start_byte >= loop_.end_byte
                    && allocation
                        .length_binding
                        .as_ref()
                        .is_some_and(|id| loop_.modifies.contains(id));
                if !fill && !count {
                    continue;
                }
                let predicates = if fill {
                    vec![
                        "exact_output_count",
                        "initialized_prefix",
                        "index_bounds",
                        "publication_complete",
                    ]
                } else {
                    vec!["exact_output_count", "count_allocation_agreement"]
                };
                obligations.extend(predicates.iter().map(|s| (*s).to_owned()));
                collection_clauses.push(LoopCollectionClause {
                    subject_id: allocation.binding_id.clone(),
                    operation: if fill { "fill" } else { "count" }.into(),
                    member_ids: vec![],
                    allocation: Some(allocation.clone()),
                    predicates: predicates.into_iter().map(str::to_owned).collect(),
                    supporting_invariants: invariants.clone(),
                });
            }
            for binding in sidecars.bindings() {
                for operation in &binding.operation_map {
                    if operation.member_id != method.callable_id {
                        continue;
                    }
                    let mut predicates = match binding.role.as_str() {
                        "bounded_sequence" => vec![
                            "exact_output_count",
                            "initialized_prefix",
                            "index_bounds",
                            "publication_complete",
                        ],
                        "ordered_map" | "ordered_set" => {
                            vec!["canonical_order", "uniqueness", "collection_bound"]
                        }
                        _ => continue,
                    };
                    if matches!(binding.role.as_str(), "ordered_map" | "ordered_set") {
                        match operation.operation.as_str() {
                            "add" => predicates.extend([
                                "duplicate_policy_reject",
                                "insertion_order_independence",
                            ]),
                            "replace" => predicates.push("duplicate_policy_replace"),
                            _ => (),
                        }
                    }
                    obligations.extend(predicates.iter().map(|s| (*s).to_owned()));
                    collection_clauses.push(LoopCollectionClause {
                        subject_id: binding.source_type_id.clone(),
                        operation: operation.operation.clone(),
                        member_ids: binding
                            .member_map
                            .iter()
                            .map(|m| m.member_id.clone())
                            .collect(),
                        allocation: None,
                        predicates: predicates.into_iter().map(str::to_owned).collect(),
                        supporting_invariants: invariants.clone(),
                    });
                }
            }
            attached.push(AttachedLoopContract {
                loop_id: loop_.loop_id.clone(),
                parent: loop_.parent.clone(),
                termination: termination.into(),
                normal_exit_claims: invariants.clone(),
                invariants,
                decreases,
                modifies,
                read_borrows: loop_.read_borrows.clone(),
                array_writes: loop_.array_writes.clone(),
                abrupt_exits: loop_.exits.clone(),
                return_claims: ensures.clone(),
                exceptional_claims: exceptional.to_vec(),
                obligations: obligations.into_iter().collect(),
                collection_clauses,
            });
        }
    }
    Ok(PreparedLoopContracts {
        exceptional_cases,
        loops: attached,
        snapshot_sha256: captures.snapshot_sha256().into(),
    })
}
