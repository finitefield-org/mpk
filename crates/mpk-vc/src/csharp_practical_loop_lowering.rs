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
}
#[derive(Clone, Debug)]
pub struct LoweredLoopControl {
    functions: Vec<LoopControlFunction>,
    contracts: PreparedLoopContracts,
    sequence_handoff: Value,
    source_facts: Value,
    normalized_syntax: Value,
}
impl LoweredLoopControl {
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
    let contracts = prepare_loop_contracts(b, r, c, context, captures, loop_source, environment)
        .map_err(LoopLoweringError::Contract)?;
    let wire: Wire = serde_json::from_slice(lowering).map_err(|_| LoopLoweringError::Source)?;
    let expected: Value =
        serde_json::from_slice(loop_source).map_err(|_| LoopLoweringError::Source)?;
    let actual: Value =
        serde_json::from_str(wire.facts.get()).map_err(|_| LoopLoweringError::Source)?;
    if wire.schema != "mpk.csharp_practical.t04_w02.loop_lowering.v1"
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
    let expected_functions = expected["methods"]
        .as_array()
        .ok_or(LoopLoweringError::Source)?
        .iter()
        .filter(|m| m["loops"].as_array().is_some_and(|l| !l.is_empty()))
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
        validate_function(function)?;
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
    Ok(LoweredLoopControl {
        functions: wire.functions,
        contracts,
        sequence_handoff: wire.sequence_handoff,
        source_facts: expected,
        normalized_syntax: syntax,
    })
}
fn validate_function(f: &LoopControlFunction) -> Result<(), LoopLoweringError> {
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
                "load" => matches!(
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
                ),
                "store" => matches!(
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
                ),
                "initializer_index" => {
                    source_kind.is_some() && n.slot.parse::<usize>().is_ok_and(|i| i < 4096)
                }
                _ => false,
            };
            if !source_matches {
                return Err(Operand);
            }
            let arity = match n.operation.as_str() {
                "load" | "constant" | "zero" | "true" | "initializer_index" => Some(0),
                "store" | "unary" | "unary_update" | "increment" | "convert"
                | "iteration_convert" | "length" | "allocate" => Some(1),
                "binary" | "less" | "element" => Some(2),
                "update" => Some(3),
                "member" | "call" | "construct" => None,
                _ => return Err(Operand),
            };
            if arity.is_some_and(|a| n.inputs.len() != a)
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
        {
            return Err(Graph);
        }
        let (edges, inputs) = match n.kind.as_str() {
            "entry" | "jump" | "break" | "continue" => (1, 0),
            "branch" | "loop_header" => (2, 1),
            "return" => (0, n.inputs.len()),
            "throw" => (0, 0),
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
            if f.nodes[*by_id.get(target.as_str()).ok_or(Graph)?].kind != "throw" {
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
