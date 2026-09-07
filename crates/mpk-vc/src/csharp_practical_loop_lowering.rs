//! W02's private source-order CFG handoff. W06 owns SSA/VIR publication.
use super::*;
use crate::csharp_practical_source_artifacts::{CapturedInputSet, PracticalArtifactContext};
use serde_json::value::RawValue;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopLoweringError {
    Contract(LoopContractError),
    Source,
    Graph,
    Operand,
    Attachment,
    Limit(&'static str),
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoopSourceOperation {
    pub constant: Option<String>,
    pub child_count: usize,
    pub implicit: bool,
    pub kind: String,
    pub symbol: String,
    pub traits: String,
    #[serde(rename = "type")]
    pub type_key: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoopControlNode {
    pub id: String,
    pub kind: String,
    pub source_ordinal: Option<usize>,
    pub operation: String,
    pub inputs: Vec<String>,
    pub result: String,
    pub slot: String,
    pub successors: Vec<String>,
    pub exceptional_successors: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoweredLoopRegion {
    pub loop_id: String,
    pub parent: Option<String>,
    pub header: String,
    pub body: String,
    pub continue_target: String,
    pub exit: String,
    pub backedges: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoopControlFunction {
    pub callable_id: String,
    pub operations: Vec<LoopSourceOperation>,
    pub nodes: Vec<LoopControlNode>,
    pub loops: Vec<LoweredLoopRegion>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    facts: Box<RawValue>,
    normalized_syntax_sha256: String,
    normalized_syntax_utf8: String,
    sequence_handoff: Value,
    functions: Vec<LoopControlFunction>,
    #[serde(default)]
    total_getters: Option<Vec<String>>,
    #[serde(default)]
    exception_definitions: Option<Vec<ExceptionDeclaration>>,
}
#[derive(Clone, Debug)]
pub struct LoweredLoopControl {
    functions: Vec<LoopControlFunction>,
    contracts: PreparedLoopContracts,
    sequence_handoff: Value,
    source_facts: Value,
    normalized_syntax: Value,
    total_getters: Vec<String>,
    exceptions: Vec<ExplicitExceptionExit>,
    exception_universe: Option<ClosedExceptionUniverse>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplicitExceptionExit {
    pub callable_id: String,
    pub throw_node_id: String,
    pub successor_id: String,
    pub value_id: String,
    pub type_id: String,
    pub tag: u32,
    pub payload_type_id: Option<String>,
    pub declared: bool,
    /// Pending until handler composition (W05/W06) or unreachable proof (T06).
    pub catch_or_unreachable: bool,
}
impl LoweredLoopControl {
    pub fn exceptions(&self) -> &[ExplicitExceptionExit] {
        &self.exceptions
    }
    pub fn functions(&self) -> &[LoopControlFunction] {
        &self.functions
    }
    pub fn contracts(&self) -> &PreparedLoopContracts {
        &self.contracts
    }
    pub fn sequence_handoff(&self) -> &Value {
        &self.sequence_handoff
    }
    pub fn source_facts(&self) -> &Value {
        &self.source_facts
    }
    pub fn normalized_syntax(&self) -> &Value {
        &self.normalized_syntax
    }
    /// Pending totality claims; T06-W04 owns semantic discharge.
    pub fn total_getters(&self) -> &[String] {
        &self.total_getters
    }
    pub fn artifact_count(&self) -> usize {
        0
    }
}
/// Contract attachment precedes graph validation. No caller-supplied proof,
/// termination discharge, or frontend-success marker is accepted here.
#[allow(clippy::too_many_arguments)]
pub fn prepare_loop_lowering(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    loop_source: &[u8],
    lowering: &[u8],
    environment: &DataContractEnvironment,
) -> Result<LoweredLoopControl, LoopLoweringError> {
    prepare_control_lowering(
        b,
        r,
        c,
        context,
        captures,
        loop_source,
        lowering,
        environment,
        None,
        None,
    )
}
/// W03 private handoff. Claims must be supplied independently by the caller;
/// they are not accepted as proofs and no frontend-success artifact is emitted.
#[allow(clippy::too_many_arguments)]
pub fn prepare_pattern_lowering(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    loop_source: &[u8],
    lowering: &[u8],
    environment: &DataContractEnvironment,
    total_getters: &BTreeSet<String>,
) -> Result<LoweredLoopControl, LoopLoweringError> {
    prepare_control_lowering(
        b,
        r,
        c,
        context,
        captures,
        loop_source,
        lowering,
        environment,
        Some(total_getters),
        None,
    )
}
#[allow(clippy::too_many_arguments)]
fn prepare_control_lowering(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    loop_source: &[u8],
    lowering: &[u8],
    environment: &DataContractEnvironment,
    total_getters: Option<&BTreeSet<String>>,
    exception_universe: Option<&ClosedExceptionUniverse>,
) -> Result<LoweredLoopControl, LoopLoweringError> {
    let patterns = total_getters.is_some();
    let exceptions = exception_universe.is_some();
    // Frozen structural budgets have phase-0 precedence over a malformed
    // phase-7 contract. Other graph errors remain behind contract attachment.
    if lowering.len() > 32 * 1024 * 1024 {
        return Err(LoopLoweringError::Limit("snapshot_total_bytes"));
    }
    if let Ok(value) = serde_json::from_slice::<Value>(lowering) {
        if let Some(functions) = value["functions"].as_array() {
            let mut total = 0usize;
            for function in functions {
                if let Some(nodes) = function["nodes"].as_array() {
                    if nodes.len() > 1024 {
                        return Err(LoopLoweringError::Limit("cfg_blocks_per_method"));
                    }
                    total = total.saturating_add(nodes.len());
                    if total > 8192 {
                        return Err(LoopLoweringError::Limit("cfg_blocks_per_closure"));
                    }
                }
            }
        }
    }
    let mut environment = environment.clone();
    environment.exception_universe = exception_universe.cloned();
    environment.exception_type = None;
    let contracts = prepare_loop_contracts(b, r, c, context, captures, loop_source, &environment)
        .map_err(LoopLoweringError::Contract)?;
    if !patterns
        && serde_json::from_slice::<Value>(lowering)
            .ok()
            .is_some_and(|v| v.get("total_getters").is_some())
    {
        return Err(LoopLoweringError::Source);
    }
    if !exceptions
        && serde_json::from_slice::<Value>(lowering)
            .ok()
            .is_some_and(|v| v.get("exception_definitions").is_some())
    {
        return Err(LoopLoweringError::Source);
    }
    let wire: Wire = serde_json::from_slice(lowering).map_err(|_| LoopLoweringError::Source)?;
    let expected: Value =
        serde_json::from_slice(loop_source).map_err(|_| LoopLoweringError::Source)?;
    let actual: Value =
        serde_json::from_str(wire.facts.get()).map_err(|_| LoopLoweringError::Source)?;
    if wire.schema
        != if exceptions {
            "mpk.csharp_practical.t04_w04.exception_lowering.v1"
        } else if patterns {
            "mpk.csharp_practical.t04_w03.pattern_lowering.v1"
        } else {
            "mpk.csharp_practical.t04_w02.loop_lowering.v1"
        }
        || wire.total_getters.as_ref()
            != total_getters
                .map(|s| s.iter().cloned().collect::<Vec<_>>())
                .as_ref()
        || wire.exception_definitions.is_some() != exceptions
        || actual != expected
        || sha256_raw_file_bytes(wire.normalized_syntax_utf8.as_bytes()).to_hex()
            != wire.normalized_syntax_sha256
        || wire.sequence_handoff["source"] != wire.normalized_syntax_sha256
    {
        return Err(LoopLoweringError::Source);
    }
    let syntax: Value = serde_json::from_str(&wire.normalized_syntax_utf8)
        .map_err(|_| LoopLoweringError::Source)?;
    let callables = syntax["callables"]
        .as_array()
        .ok_or(LoopLoweringError::Source)?;
    if wire.total_getters.as_ref().is_some_and(|claims| {
        claims
            .iter()
            .any(|id| !callables.iter().any(|c| c["id"] == *id))
    }) {
        return Err(LoopLoweringError::Source);
    }
    let expected_functions = expected["methods"]
        .as_array()
        .ok_or(LoopLoweringError::Source)?
        .iter()
        .filter(|m| {
            m["loops"].as_array().is_some_and(|l| !l.is_empty())
                || patterns
                    && callables.iter().any(|c| {
                        c["id"] == m["callable_id"]
                            && c["body"]
                                .as_str()
                                .and_then(|body| {
                                    serde_json::from_str::<Vec<LoopSourceOperation>>(body).ok()
                                })
                                .is_some_and(|ops| {
                                    ops.iter().any(|op| {
                                        exceptions && op.kind == "Throw"
                                            || matches!(
                                                op.kind.as_str(),
                                                "IsPattern" | "Switch" | "SwitchExpression"
                                            )
                                    })
                                })
                    })
        })
        .map(|m| m["callable_id"].as_str().ok_or(LoopLoweringError::Source))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if expected_functions.len() != wire.functions.len() {
        return Err(LoopLoweringError::Source);
    }
    let mut ids = BTreeSet::new();
    let mut loops = BTreeSet::new();
    let mut blocks = 0usize;
    for function in &wire.functions {
        if !expected_functions.contains(function.callable_id.as_str())
            || !ids.insert(function.callable_id.as_str())
        {
            return Err(LoopLoweringError::Source);
        }
        let callable = callables
            .iter()
            .find(|row| row["id"] == function.callable_id)
            .ok_or(LoopLoweringError::Source)?;
        let body = callable["body"].as_str().ok_or(LoopLoweringError::Source)?;
        let operations: Vec<LoopSourceOperation> =
            serde_json::from_str(body).map_err(|_| LoopLoweringError::Source)?;
        if operations != function.operations
            || callable["body_sha256"] != sha256_raw_file_bytes(body.as_bytes()).to_hex()
        {
            return Err(LoopLoweringError::Source);
        }
        blocks = blocks
            .checked_add(function.nodes.len())
            .ok_or(LoopLoweringError::Limit("cfg_blocks_per_closure"))?;
        if blocks > 8192 {
            return Err(LoopLoweringError::Limit("cfg_blocks_per_closure"));
        }
        validate_function(function, patterns, exception_universe)?;
        if function
            .nodes
            .iter()
            .any(|n| n.operation == "construction_assign")
        {
            let method = expected["methods"]
                .as_array()
                .ok_or(LoopLoweringError::Source)?
                .iter()
                .find(|m| m["callable_id"] == function.callable_id)
                .ok_or(LoopLoweringError::Source)?;
            let parameters = method["parameters"]
                .as_array()
                .ok_or(LoopLoweringError::Source)?;
            let receiver = parameters
                .first()
                .filter(|p| p["id"] == "this")
                .ok_or(LoopLoweringError::Operand)?;
            let owner = receiver["type_id"]
                .as_str()
                .ok_or(LoopLoweringError::Operand)?;
            let ty = r
                .source_types
                .get(owner)
                .ok_or(LoopLoweringError::Operand)?;
            let id=csharp_practical_declaration_id(&json!({"kind":"constructor","namespace":ty.identity.namespace,"owner":owner,"name":ty.identity.name,"parameter_type_ids":parameters[1..].iter().map(|p|p["type_id"].clone()).collect::<Vec<_>>(),"result_type_id":owner})).map_err(|_|LoopLoweringError::Operand)?;
            if id != function.callable_id {
                return Err(LoopLoweringError::Operand);
            }
            for node in function
                .nodes
                .iter()
                .filter(|n| n.operation == "construction_assign")
            {
                if !ty
                    .members
                    .iter()
                    .any(|m| node.slot == format!("{owner}.{}", m.name))
                    || !function.nodes.iter().any(|n| {
                        Some(&n.result) == node.inputs.first()
                            && n.operation == "load"
                            && n.slot == "this"
                    })
                {
                    return Err(LoopLoweringError::Operand);
                }
            }
        }
        for region in &function.loops {
            let contract = contracts
                .loops()
                .iter()
                .find(|c| c.loop_id == region.loop_id)
                .ok_or(LoopLoweringError::Attachment)?;
            if contract.parent != region.parent || !loops.insert(region.loop_id.as_str()) {
                return Err(LoopLoweringError::Attachment);
            }
        }
    }
    if loops
        != contracts
            .loops()
            .iter()
            .map(|c| c.loop_id.as_str())
            .collect()
    {
        return Err(LoopLoweringError::Attachment);
    }
    let exits = validate_explicit_exits(&wire.functions, exception_universe, &contracts)?;
    Ok(LoweredLoopControl {
        exceptions: exits,
        exception_universe: exception_universe.cloned(),
        functions: wire.functions,
        contracts,
        sequence_handoff: wire.sequence_handoff,
        source_facts: expected,
        normalized_syntax: syntax,
        total_getters: wire.total_getters.unwrap_or_default(),
    })
}
fn validate_function(
    f: &LoopControlFunction,
    patterns: bool,
    universe: Option<&ClosedExceptionUniverse>,
) -> Result<(), LoopLoweringError> {
    use LoopLoweringError::{Graph, Operand};
    if f.nodes.len() > 1024 {
        return Err(LoopLoweringError::Limit("cfg_blocks_per_method"));
    }
    if f.nodes.first().is_none_or(|n| n.kind != "entry") {
        return Err(Graph);
    }
    let mut by_id = BTreeMap::new();
    let mut values = BTreeMap::new();
    for (ordinal, n) in f.nodes.iter().enumerate() {
        if n.id != format!("{}.node.{ordinal:06}", f.callable_id)
            || by_id.insert(n.id.as_str(), ordinal).is_some()
            || n.source_ordinal.is_some_and(|i| i >= f.operations.len())
        {
            return Err(Graph);
        }
        if n.kind == "evaluate" {
            if n.result != format!("{}.value.{:06}", f.callable_id, values.len())
                || values.insert(n.result.as_str(), ordinal).is_some()
            {
                return Err(Operand);
            }
            let source_kind = n.source_ordinal.map(|i| f.operations[i].kind.as_str());
            let source_matches = match n.operation.as_str() {
                "pattern_constant" => {
                    patterns
                        && source_kind == Some("FieldReference")
                        && n.source_ordinal
                            .is_some_and(|i| f.operations[i].constant.is_some())
                }
                "pattern_true" | "pattern_false" => patterns && source_kind == Some("IsPattern"),
                "pattern_equal" => {
                    patterns && matches!(source_kind, Some("ConstantPattern" | "CaseClause"))
                }
                "pattern_relational" => patterns && source_kind == Some("RelationalPattern"),
                "pattern_type" => {
                    patterns
                        && matches!(
                            source_kind,
                            Some("DeclarationPattern" | "TypePattern" | "RecursivePattern")
                        )
                }
                "pattern_not_null" => {
                    patterns
                        && matches!(
                            source_kind,
                            Some("ListPattern" | "PropertyReference" | "FieldReference")
                        )
                }
                "pattern_length" | "pattern_element" => {
                    patterns
                        && source_kind == Some("ListPattern")
                        && n.slot.parse::<usize>().is_ok_and(|i| i <= 4096)
                }
                "pattern_member" => {
                    patterns && matches!(source_kind, Some("PropertyReference" | "FieldReference"))
                }
                "pattern_bind" => {
                    patterns
                        && matches!(
                            source_kind,
                            Some("DeclarationPattern" | "RecursivePattern" | "ListPattern")
                        )
                        && n.source_ordinal.is_some_and(|i| {
                            f.operations[i].symbol == n.slot && n.slot.starts_with("local:")
                        })
                }
                "constant" => matches!(source_kind, Some("Literal" | "DefaultValue")),
                "binary" => matches!(source_kind, Some("Binary" | "CompoundAssignment")),
                "unary" => source_kind == Some("Unary"),
                "unary_update" => matches!(source_kind, Some("Increment" | "Decrement")),
                "convert" => source_kind == Some("Conversion"),
                "iteration_convert" => source_kind == Some("VariableDeclarator"),
                "allocate" => source_kind == Some("ArrayCreation"),
                "member" => matches!(source_kind, Some("PropertyReference" | "FieldReference")),
                "call" => source_kind == Some("Invocation"),
                "construct" => source_kind == Some("ObjectCreation"),
                "zero" | "true" | "less" | "increment" | "length" => source_kind == Some("Loop"),
                "element" => matches!(
                    source_kind,
                    Some(
                        "ArrayElementReference"
                            | "CompoundAssignment"
                            | "Increment"
                            | "Decrement"
                            | "Loop"
                    )
                ),
                "update" => matches!(
                    source_kind,
                    Some(
                        "SimpleAssignment"
                            | "CompoundAssignment"
                            | "Increment"
                            | "Decrement"
                            | "ArrayCreation"
                    )
                ),
                "construction_assign" => {
                    universe.is_some()
                        && source_kind == Some("SimpleAssignment")
                        && n.source_ordinal
                            .and_then(|i| f.operations.get(i + 1))
                            .is_some_and(|o| {
                                matches!(o.kind.as_str(), "FieldReference" | "PropertyReference")
                                    && o.symbol == n.slot
                            })
                }
                "closed_exception" => universe.is_some() && source_kind == Some("ObjectCreation"),
                "load" => {
                    patterns && matches!(source_kind, Some("IsPattern" | "SwitchExpression"))
                        || matches!(
                            source_kind,
                            None | Some(
                                "LocalReference"
                                    | "ParameterReference"
                                    | "InstanceReference"
                                    | "CompoundAssignment"
                                    | "Increment"
                                    | "Decrement"
                                    | "Conditional"
                                    | "Binary"
                            )
                        )
                }
                "store" => {
                    patterns && matches!(source_kind, Some("IsPattern" | "SwitchExpression"))
                        || matches!(
                            source_kind,
                            Some(
                                "VariableDeclarator"
                                    | "SimpleAssignment"
                                    | "CompoundAssignment"
                                    | "Increment"
                                    | "Decrement"
                                    | "Conditional"
                                    | "Binary"
                                    | "Loop"
                            )
                        )
                }
                "initializer_index" => {
                    source_kind.is_some() && n.slot.parse::<usize>().is_ok_and(|i| i < 4096)
                }
                _ => false,
            };
            if !source_matches {
                return Err(Operand);
            }
            if matches!(n.operation.as_str(), "pattern_length" | "pattern_element") {
                let count = f.operations[n.source_ordinal.ok_or(Operand)?].child_count;
                let index = n.slot.parse::<usize>().map_err(|_| Operand)?;
                if n.operation == "pattern_length" && index != count
                    || n.operation == "pattern_element" && index >= count
                {
                    return Err(Operand);
                }
            }

            let arity = match n.operation.as_str() {
                "load" | "constant" | "zero" | "true" | "initializer_index" | "pattern_true"
                | "pattern_false" | "pattern_constant" => Some(0),
                "pattern_type" | "pattern_not_null" | "pattern_length" | "pattern_element"
                | "pattern_member" | "pattern_bind" => Some(1),
                "pattern_equal" | "pattern_relational" => Some(2),
                "store" | "unary" | "unary_update" | "increment" | "convert"
                | "iteration_convert" | "length" | "allocate" => Some(1),
                "binary" | "less" | "element" | "construction_assign" => Some(2),
                "update" => Some(3),
                "member" | "call" | "construct" => None,
                "closed_exception" => Some(usize::from(
                    universe.and_then(|u| u.arm(&n.slot)).ok_or(Operand)?.tag >= 9,
                )),
                _ => return Err(Operand),
            };
            if n.operation == "construction_assign" && !n.exceptional_successors.is_empty()
                || arity.is_some_and(|a| n.inputs.len() != a)
                || n.successors.len() != 1
                || n.exceptional_successors.len() > 1
            {
                return Err(Operand);
            }
            if matches!(n.operation.as_str(), "load" | "store")
                && !(n.slot.starts_with("local:")
                    || n.slot.starts_with("parameter:")
                    || n.slot.starts_with("temporary:")
                    || n.slot == "this")
            {
                return Err(Operand);
            }
        } else if !n.result.is_empty()
            || !n.operation.is_empty()
            || !n.exceptional_successors.is_empty()
                && !(universe.is_some()
                    && n.kind == "explicit_throw"
                    && n.exceptional_successors.len() == 1)
        {
            return Err(Graph);
        }
        if n.kind == "throw"
            && !n.slot.is_empty()
            && !(patterns
                && n.slot == "System.Runtime.CompilerServices.SwitchExpressionException"
                && n.source_ordinal
                    .is_some_and(|i| f.operations[i].kind == "SwitchExpression"))
        {
            return Err(Graph);
        }
        let (edges, inputs) = match n.kind.as_str() {
            "pattern_decision"
                if patterns
                    && n.source_ordinal.is_some_and(|i| {
                        matches!(
                            f.operations[i].kind.as_str(),
                            "Switch" | "SwitchExpression" | "IsPattern"
                        )
                    }) =>
            {
                (1, 1)
            }
            "entry" | "jump" | "break" | "continue" => (1, 0),
            "branch" | "loop_header" => (2, 1),
            "return" => (0, n.inputs.len()),
            "throw" => (0, 0),
            "explicit_throw" if universe.is_some() => (0, 1),
            "exception_exit" if universe.is_some() => (0, 0),
            "evaluate" => continue,
            _ => return Err(Graph),
        };
        if n.successors.len() != edges
            || n.inputs.len() != inputs
            || n.kind == "return" && n.inputs.len() > 1
        {
            return Err(Graph);
        }
    }
    if patterns {
        for (ordinal, op) in f.operations.iter().enumerate().filter(|(_, op)| {
            matches!(
                op.kind.as_str(),
                "IsPattern" | "Switch" | "SwitchExpression"
            )
        }) {
            if f.nodes
                .iter()
                .filter(|n| n.kind == "pattern_decision" && n.source_ordinal == Some(ordinal))
                .count()
                != 1
            {
                return Err(Graph);
            }
            if op.kind == "SwitchExpression"
                && f.nodes
                    .iter()
                    .filter(|n| {
                        n.kind == "throw"
                            && n.source_ordinal == Some(ordinal)
                            && n.slot == "System.Runtime.CompilerServices.SwitchExpressionException"
                    })
                    .count()
                    != 1
            {
                return Err(Graph);
            }
        }
    }
    let mut predecessors = vec![BTreeSet::new(); f.nodes.len()];
    for (ordinal, n) in f.nodes.iter().enumerate() {
        for target in n.successors.iter().chain(&n.exceptional_successors) {
            let dest = *by_id.get(target.as_str()).ok_or(Graph)?;
            predecessors[dest].insert(ordinal);
        }
        for value in &n.inputs {
            if !values.contains_key(value.as_str()) {
                return Err(Operand);
            }
        }
        for target in &n.exceptional_successors {
            if f.nodes[*by_id.get(target.as_str()).ok_or(Graph)?].kind
                != if n.kind == "explicit_throw" {
                    "exception_exit"
                } else {
                    "throw"
                }
            {
                return Err(Graph);
            }
        }
    }
    // Dominance is checked on all reachable normal/abrupt paths. Unreachable
    // structural tails retain their source identity but cannot define a value
    // used on an executable path.
    let mut reachable = BTreeSet::new();
    let mut work = vec![0usize];
    while let Some(i) = work.pop() {
        if reachable.insert(i) {
            for target in f.nodes[i]
                .successors
                .iter()
                .chain(&f.nodes[i].exceptional_successors)
            {
                work.push(by_id[target.as_str()]);
            }
        }
    }
    let mut dom = vec![reachable.clone(); f.nodes.len()];
    dom[0] = [0].into_iter().collect();
    loop {
        let mut changed = false;
        for &i in &reachable {
            if i == 0 {
                continue;
            }
            let mut d = reachable.clone();
            for p in predecessors[i].intersection(&reachable) {
                d = d.intersection(&dom[*p]).copied().collect();
            }
            d.insert(i);
            if d != dom[i] {
                dom[i] = d;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for &i in &reachable {
        for value in &f.nodes[i].inputs {
            let definition = values[value.as_str()];
            if definition == i || !dom[i].contains(&definition) {
                return Err(Operand);
            }
        }
    }
    let mut region_ids = BTreeSet::new();
    let mut allowed_backedges = BTreeSet::new();
    for region in &f.loops {
        if !region_ids.insert(region.loop_id.as_str())
            || region
                .parent
                .as_ref()
                .is_some_and(|p| !region_ids.contains(p.as_str()))
        {
            return Err(Graph);
        }
        let header = *by_id.get(region.header.as_str()).ok_or(Graph)?;
        if f.nodes[header].kind != "loop_header"
            || f.nodes[header].slot != region.loop_id
            || f.nodes[header].successors != [region.body.clone(), region.exit.clone()]
        {
            return Err(Graph);
        }
        for id in [&region.body, &region.continue_target, &region.exit] {
            if !by_id.contains_key(id.as_str()) {
                return Err(Graph);
            }
        }
        let incoming = predecessors[header]
            .iter()
            .map(|i| f.nodes[*i].id.as_str())
            .collect::<BTreeSet<_>>();
        let back = region
            .backedges
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if back.len() != region.backedges.len()
            || back.is_empty()
            || !back.is_subset(&incoming)
            || incoming.difference(&back).count() != 1
        {
            return Err(Graph);
        }
        for id in &back {
            let source = by_id[*id];
            if reachable.contains(&source) && !dom[source].contains(&header) {
                return Err(Graph);
            }
            allowed_backedges.insert((source, header));
        }
        for n in &f.nodes {
            if n.slot == region.loop_id
                && matches!(n.kind.as_str(), "break" | "continue")
                && n.successors
                    != [if n.kind == "break" {
                        region.exit.clone()
                    } else {
                        region.continue_target.clone()
                    }]
            {
                return Err(Graph);
            }
        }
    }
    for n in &f.nodes {
        if matches!(n.kind.as_str(), "loop_header" | "break" | "continue")
            && !region_ids.contains(n.slot.as_str())
        {
            return Err(Graph);
        }
    }
    // Removing exactly the declared backedges must leave an acyclic graph.
    fn visit(
        i: usize,
        f: &LoopControlFunction,
        ids: &BTreeMap<&str, usize>,
        back: &BTreeSet<(usize, usize)>,
        colors: &mut [u8],
    ) -> Result<(), LoopLoweringError> {
        if colors[i] == 1 {
            return Err(LoopLoweringError::Graph);
        }
        if colors[i] == 2 {
            return Ok(());
        }
        colors[i] = 1;
        for id in &f.nodes[i].successors {
            let j = ids[id.as_str()];
            if !back.contains(&(i, j)) {
                visit(j, f, ids, back, colors)?;
            }
        }
        colors[i] = 2;
        Ok(())
    }
    let mut colors = vec![0; f.nodes.len()];
    for i in 0..f.nodes.len() {
        visit(i, f, &by_id, &allowed_backedges, &mut colors)?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExceptionDeclaration {
    type_id: String,
    sealed_type: bool,
    direct_base_type_id: String,
    payload_member_names: Vec<String>,
}
/// W04 consumes captured declarations and the existing closed sum. This is
/// still a private control handoff; pending exits cannot certify a program.
#[allow(clippy::too_many_arguments)]
pub fn prepare_exception_lowering(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    loop_source: &[u8],
    lowering: &[u8],
    environment: &DataContractEnvironment,
    total_getters: &BTreeSet<String>,
) -> Result<LoweredLoopControl, LoopLoweringError> {
    if lowering.len() > 32 * 1024 * 1024 {
        return Err(LoopLoweringError::Limit("snapshot_total_bytes"));
    }
    let wire: Wire = serde_json::from_slice(lowering).map_err(|_| LoopLoweringError::Source)?;
    let declarations = wire
        .exception_definitions
        .as_ref()
        .ok_or(LoopLoweringError::Source)?;
    if declarations.len() > 32 {
        return Err(LoopLoweringError::Limit("source_exception_types"));
    }
    let definitions = declarations
        .iter()
        .map(|d| {
            let source = r
                .source_types
                .get(&d.type_id)
                .ok_or(LoopLoweringError::Source)?;
            let facts: Value =
                serde_json::from_str(wire.facts.get()).map_err(|_| LoopLoweringError::Source)?;
            if !facts["sources"]
                .as_array()
                .ok_or(LoopLoweringError::Source)?
                .iter()
                .any(|s| s["raw_sha256"] == source.source_sha256)
            {
                return Err(LoopLoweringError::Source);
            }
            if source
                .members
                .iter()
                .map(|m| m.name.as_str())
                .ne(d.payload_member_names.iter().map(String::as_str))
            {
                return Err(LoopLoweringError::Source);
            }
            Ok(SourceExceptionDefinition {
                type_id: d.type_id.clone(),
                sealed: d.sealed_type,
                direct_base_type_id: d.direct_base_type_id.clone(),
                payload_member_ids: source.members.iter().map(|m| m.id.clone()).collect(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let universe = derive_closed_exception_universe(r, c, &definitions)
        .map_err(|_| LoopLoweringError::Source)?;
    prepare_control_lowering(
        b,
        r,
        c,
        context,
        captures,
        loop_source,
        lowering,
        environment,
        Some(total_getters),
        Some(&universe),
    )
}

fn validate_explicit_exits(
    functions: &[LoopControlFunction],
    universe: Option<&ClosedExceptionUniverse>,
    contracts: &PreparedLoopContracts,
) -> Result<Vec<ExplicitExceptionExit>, LoopLoweringError> {
    use LoopLoweringError::Operand;
    let Some(universe) = universe else {
        return Ok(Vec::new());
    };
    let mut exits = Vec::new();
    for f in functions {
        let mut consumed = BTreeSet::new();
        let mut throw_ordinals = BTreeSet::new();
        let mut successors = BTreeSet::new();
        for n in f.nodes.iter().filter(|n| n.kind == "explicit_throw") {
            let ordinal = n.source_ordinal.ok_or(Operand)?;
            if !throw_ordinals.insert(ordinal)
                || f.operations[ordinal].kind != "Throw"
                || n.inputs.len() != 1
                || n.exceptional_successors.len() != 1
            {
                return Err(Operand);
            }
            let value = f
                .nodes
                .iter()
                .find(|v| v.result == n.inputs[0] && v.operation == "closed_exception")
                .ok_or(Operand)?;
            let arm = universe.arm(&n.slot).ok_or(Operand)?;
            if value.slot != n.slot
                || !value.exceptional_successors.is_empty()
                || !consumed.insert(value.id.as_str())
            {
                return Err(Operand);
            }
            let exit = f
                .nodes
                .iter()
                .find(|e| {
                    e.id == n.exceptional_successors[0]
                        && e.kind == "exception_exit"
                        && e.slot == n.slot
                        && e.source_ordinal == n.source_ordinal
                })
                .ok_or(Operand)?;
            if !successors.insert(exit.id.as_str()) {
                return Err(Operand);
            }
            let creation_ordinal = value.source_ordinal.ok_or(Operand)?;
            let creation = &f.operations[creation_ordinal];
            let mut end = ordinal + 1;
            let mut pending = f.operations[ordinal].child_count;
            while pending > 0 {
                let child = f.operations.get(end).ok_or(Operand)?;
                pending = pending
                    .checked_sub(1)
                    .and_then(|count| count.checked_add(child.child_count))
                    .ok_or(Operand)?;
                end += 1;
            }
            if !(ordinal < creation_ordinal && creation_ordinal < end) {
                return Err(Operand);
            }
            if arm.tag < 9 {
                if creation.child_count != 0
                    || creation.type_key.as_deref()
                        != Some(format!("source_exception:{}", arm.type_id).as_str())
                    || creation.symbol
                        != format!(
                            "System.Runtime|{}.{}()",
                            arm.type_id,
                            arm.type_id.rsplit('.').next().ok_or(Operand)?
                        )
                {
                    return Err(Operand);
                }
            } else {
                let payload = f
                    .nodes
                    .iter()
                    .find(|p| {
                        value.inputs.first() == Some(&p.result)
                            && p.operation == "construct"
                            && p.source_ordinal == value.source_ordinal
                    })
                    .ok_or(Operand)?;
                if payload.inputs.len() != creation.child_count
                    || creation.type_key.as_deref()
                        != Some(
                            format!("{}:{}13:not_annotated0:", arm.type_id.len(), arm.type_id)
                                .as_str(),
                        )
                {
                    return Err(Operand);
                }
            }
            let declared = contracts
                .exceptional_cases()
                .get(&f.callable_id)
                .is_some_and(|cases| {
                    cases.iter().any(|case| {
                        case.get("exception_type_id").and_then(|v| v.as_str()) == Some(&arm.type_id)
                    })
                });
            exits.push(ExplicitExceptionExit {
                callable_id: f.callable_id.clone(),
                throw_node_id: n.id.clone(),
                successor_id: exit.id.clone(),
                value_id: value.result.clone(),
                type_id: arm.type_id.clone(),
                tag: arm.tag,
                payload_type_id: (arm.tag >= 9).then(|| arm.type_id.clone()),
                declared,
                catch_or_unreachable: !declared,
            });
        }
        if consumed.len()
            != f.nodes
                .iter()
                .filter(|n| n.operation == "closed_exception")
                .count()
            || successors.len()
                != f.nodes
                    .iter()
                    .filter(|n| n.kind == "exception_exit")
                    .count()
            || consumed.len() != f.operations.iter().filter(|o| o.kind == "Throw").count()
        {
            return Err(Operand);
        }
    }
    Ok(exits)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationExceptionResult {
    /// Preserve the data owner's exact type, check id and successor.
    pub successor: ExceptionalSuccessor,
    pub tag: u32,
    pub declared: bool,
    pub catch_or_unreachable: bool,
}
impl LoweredLoopControl {
    /// Consume the existing T03 operation signature/edges without a second
    /// exception conversion. W06 attaches these to the composed source CFG.
    pub fn operation_exception_results(
        &self,
        roots: &ValidatedClosedRootSet,
        closed: &ClosedInstanceSet,
        callable_id: &str,
        signature: &ClosedOperationSignature,
        invocation: &OperationInvocation,
    ) -> Result<Vec<OperationExceptionResult>, LoopLoweringError> {
        let universe = self
            .exception_universe
            .as_ref()
            .ok_or(LoopLoweringError::Attachment)?;
        if !self.source_facts["methods"]
            .as_array()
            .ok_or(LoopLoweringError::Source)?
            .iter()
            .any(|m| m["callable_id"] == callable_id)
        {
            return Err(LoopLoweringError::Attachment);
        }
        validate_operation_invocation(roots, closed, signature, invocation)
            .map_err(|_| LoopLoweringError::Operand)?;
        invocation
            .exceptional_successors
            .iter()
            .map(|edge| {
                let arm = universe
                    .arm(&edge.exception_type_id)
                    .ok_or(LoopLoweringError::Operand)?;
                let declared = self
                    .contracts
                    .exceptional_cases()
                    .get(callable_id)
                    .is_some_and(|cases| {
                        cases.iter().any(|case| {
                            case.get("exception_type_id").and_then(|v| v.as_str())
                                == Some(&arm.type_id)
                        })
                    });
                Ok(OperationExceptionResult {
                    successor: edge.clone(),
                    tag: arm.tag,
                    declared,
                    catch_or_unreachable: !declared,
                })
            })
            .collect()
    }
}
