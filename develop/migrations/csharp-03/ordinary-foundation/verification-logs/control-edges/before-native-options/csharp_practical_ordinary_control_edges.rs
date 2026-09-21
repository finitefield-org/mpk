//! Exact W04 guards and transport from source exit to edge-specific target slots.
//! Node-entry merging and execution produce separate, scoped obligations.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{DataDefinitionFamily, DataSemanticDefinition};
use crate::csharp_practical_vir_model::{
    ControlBinding, ControlFlowEdge, ControlFunctionVc, ControlSlotTransfer,
};
use sha2::{Digest, Sha256};
#[path = "csharp_practical_ordinary_control_execution.rs"]
mod execution;
pub use execution::{OrdinaryControlNativeDefinition, OrdinaryControlNativeOperation};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeJoin {
    /// Guard bindings, then source-exit/edge-specific target state pairs.
    pub arguments: Vec<ControlBinding>,
    pub guard_argument_count: usize,
    /// Never equate a backedge result to the previous header-entry snapshot.
    pub state_rule: String,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlEntryIncoming {
    pub edge_id: String,
    pub selected_argument: usize,
    pub guard_argument_start: usize,
    pub guard_argument_count: usize,
    pub state_argument_start: usize,
    pub pending_constant_names: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlEntryComponent {
    pub role: String,
    /// Indices into the parent entry's shared argument list, in call order.
    pub argument_indices: Vec<usize>,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlNodeEntry {
    pub node_id: String,
    /// Shared entry state, then (selected, guard inputs, incoming state) per edge.
    /// Incoming state has exactly entry_state_argument_count arguments.
    pub arguments: Vec<ControlBinding>,
    pub entry_state_argument_count: usize,
    pub incoming: Vec<OrdinaryControlEntryIncoming>,
    /// Exactly one incoming edge must be selected and its guard must hold.
    /// Only that edge's slot/phi snapshot determines this execution's entry.
    pub state_rule: String,
    /// A compact complete relation, when it fits within the binder limit.
    pub definition: Option<String>,
    /// Otherwise every component must hold at its exact indexed arguments.
    /// No definition and no components means incoming guards remain pending.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<OrdinaryControlEntryComponent>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlMemoryEffect {
    pub source_node_id: String,
    pub receiver_transfer: ControlSlotTransfer,
    /// The source update returns its element; this is not the new array SSA.
    pub source_result: TypedValueRef,
    pub operation: crate::csharp_practical_vir_model::data_vc::DataOperationVc,
    pub allocation_origin_id: String,
    pub before_state_id: String,
    pub after_state_id: String,
    pub ownership: OrdinaryConstructionOwnershipUse,
    pub snapshot_definition: String,
    /// Before/after assignedness and slot values, then exact native subjects.
    pub arguments: Vec<ControlBinding>,
    /// A successful update of this receiver slot only; other frames are separate.
    pub execution_scope: String,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSourceFrame {
    pub source_node_id: String,
    pub source_operation: String,
    pub entry_node_id: Option<String>,
    pub exit_node_id: Option<String>,
    /// A store's target is constrained by its separate successful transfer.
    pub transfer: Option<ControlSlotTransfer>,
    pub framed_slots: Vec<String>,
    pub arguments: Vec<ControlBinding>,
    /// Local-slot framing alone does not establish execution or its result.
    pub state_rule: String,
    pub pending_reason: Option<String>,
    pub definition: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeDefinition {
    pub source: ControlFlowEdge,
    pub guard_definition: Option<String>,
    pub join: Option<OrdinaryControlEdgeJoin>,
    /// Native phi values at this incoming snapshot, selected by predecessor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phi_join: Option<OrdinaryControlEdgeJoin>,
    /// This binding applies only at the retained source function/node/receiver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<OrdinaryConstructionOwnershipUse>,
    /// A missing semantic definition leaves the whole guard and join pending.
    pub pending_constant_names: Vec<String>,
    /// An unconditional built-in throw at this exact captured source anchor.
    /// This says nothing about reachability, handler search or exception entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin_throw_source_node_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeFunction {
    pub source: ControlFunctionVc,
    pub edges: Vec<OrdinaryControlEdgeDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub memory_bindings: Vec<OrdinaryControlMemoryBinding>,
    /// Exact native storage for nominal source slots, retaining nullable presence.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub slot_type_overrides: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub node_entries: Vec<OrdinaryControlNodeEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub memory_effects: Vec<OrdinaryControlMemoryEffect>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending_memory_effect_node_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_frames: Vec<OrdinaryControlSourceFrame>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub native_operations: Vec<OrdinaryControlNativeOperation>,
    /// Invocations without a W03 data operation still require native semantics.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending_native_invocation_node_ids: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlMemoryBinding {
    pub transfer: ControlSlotTransfer,
    pub source_slot_type_id: String,
    /// Immutable allocation identity, distinct from its current SSA storage.
    pub allocation_origin_id: String,
    pub state_id: String,
    pub flow_theorem: String,
    /// Native slot view followed by the exact current memory SSA value.
    pub arguments: Vec<ControlBinding>,
    pub definition: String,
    /// A 16384-cell private array is not a 4096-cell published sequence.
    pub public_slot_projection_pending: bool,
    /// Latest native storage -> initialized and within the source array bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_projection_definedness: Option<String>,
    /// (Latest native storage, source slot snapshot) -> defined and exact copy.
    /// Does not consume the ownership token or establish these preconditions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_projection_definition: Option<String>,
    /// Bounded raw source-slot snapshot, including partially initialized storage.
    /// Initialization remains on native read/fill/rewrite/publication operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_slot_snapshot_definition: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    control_vc_sha256: String,
    integer_definitions: Vec<OrdinaryIntegerDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    construction_definitions: Vec<OrdinaryConstructionDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    string_definitions: Vec<OrdinaryStringDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sequence_definitions: Vec<OrdinarySequenceDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reference_definitions: Vec<OrdinaryReferenceDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    option_definitions: Vec<OrdinaryOptionDataDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    native_definitions: Vec<OrdinaryControlNativeDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    symbolic_ownership: Vec<OrdinaryOwnershipFunction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    symbolic_ownership_proofs: Vec<OrdinaryOwnershipProof>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pending_concrete_ownership_functions: Vec<String>,
    functions: Vec<OrdinaryControlEdgeFunction>,
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryControlEdgeProgram {
    pub fn native_definitions(&self) -> &[OrdinaryControlNativeDefinition] {
        &self.native_definitions
    }
    pub fn functions(&self) -> &[OrdinaryControlEdgeFunction] {
        &self.functions
    }
    pub fn integer_definitions(&self) -> &[OrdinaryIntegerDataDefinition] {
        &self.integer_definitions
    }
    pub fn construction_definitions(&self) -> &[OrdinaryConstructionDataDefinition] {
        &self.construction_definitions
    }
    pub fn string_definitions(&self) -> &[OrdinaryStringDataDefinition] {
        &self.string_definitions
    }
    pub fn sequence_definitions(&self) -> &[OrdinarySequenceDataDefinition] {
        &self.sequence_definitions
    }
    pub fn reference_definitions(&self) -> &[OrdinaryReferenceDataDefinition] {
        &self.reference_definitions
    }
    pub fn option_definitions(&self) -> &[OrdinaryOptionDataDefinition] {
        &self.option_definitions
    }
    pub fn symbolic_ownership(&self) -> &[OrdinaryOwnershipFunction] {
        &self.symbolic_ownership
    }
    pub fn symbolic_ownership_proofs(&self) -> &[OrdinaryOwnershipProof] {
        &self.symbolic_ownership_proofs
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary control edges")
    }
}
fn constants(term: &ContractTerm, names: &mut BTreeSet<String>) -> R<()> {
    match term {
        ContractTerm::Const { name, .. } => {
            names.insert(name.clone());
        }
        ContractTerm::Var { .. } => {}
        ContractTerm::App {
            function, argument, ..
        } => {
            constants(function, names)?;
            constants(argument, names)?;
        }
        // W04 edge guards are binder-free W03 checks and SSA conditions.
        // Do not silently impose this convention on contract/loop sequents.
        _ => return Err(OrdinaryCarrierError::Shape),
    }
    Ok(())
}
fn name(role: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.ControlEdge.{role}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed edge identity"))
    )
}
fn builtin_throw_source(
    flow: &ControlFunctionVc,
    native: &crate::csharp_practical_vir_validation::PracticalVirFunction,
    edge: &ControlFlowEdge,
) -> R<Option<String>> {
    use crate::csharp_practical_vir_model::{AbruptCompletion, ControlNodeTag};
    const EXCEPTION: &str = "System.Runtime.CompilerServices.SwitchExpressionException";
    let (Some(source), Some(protocol)) = (&flow.source_graph, &native.control_protocol) else {
        return Ok(None);
    };
    let Some((node, anchor)) = source.nodes.iter().find_map(|node| {
        (node.kind == "builtin_throw" && node.slot == EXCEPTION)
            .then(|| {
                protocol
                    .anchors
                    .iter()
                    .find(|a| a.source_node_id == node.id && a.exit_node_id == edge.source_node_id)
                    .map(|a| (node, a))
            })
            .flatten()
    }) else {
        return Ok(None);
    };
    // Only the exception edge is authorized here, not a second abrupt edge or
    // a general exception predicate inferred from its name.
    if edge.kind != "exception" {
        return Ok(None);
    }
    let check = format!("exception.closed.{EXCEPTION}");
    let symbol = format!(
        "Mpk.CSharp.Control.ExceptionEdge.{}.{check}",
        edge.source_node_id
    );
    let block = native
        .blocks
        .iter()
        .find(|b| b.node.id == edge.source_node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let expected_abrupt = AbruptCompletion::Throw {
        exception_type_id: EXCEPTION.into(),
        rethrow_from_catch_id: None,
    };
    if !node.inputs.is_empty()
        || !node.result.is_empty()
        || !node.operation.is_empty()
        || !node.successors.is_empty()
        || node.exceptional_successors.len() != 1
        || node
            .source_ordinal
            .and_then(|i| source.operations.get(i))
            .is_none_or(|o| o.kind != "SwitchExpression")
        || anchor.source_ordinal != node.source_ordinal
        || !anchor.artifact_node_ids.contains(&block.node.id)
        || block.node.tag != ControlNodeTag::Throw
        || block.node.abrupt.as_ref() != Some(&expected_abrupt)
        || !block.node.normal_successor_ids.is_empty()
        || block.invocation.is_some()
        || block.condition_value_id.is_some()
        || block.node.exceptional_successors.len() != 1
        || block.node.exceptional_successors[0].check_id != check
        || block.node.exceptional_successors[0].exception_type_id != EXCEPTION
        || Some(&block.node.exceptional_successors[0].target_id) != edge.target_node_id.as_ref()
        || edge.check_id.as_ref() != Some(&check)
        || !edge.guard.bindings.is_empty()
        || edge.guard.term
            != (ContractTerm::Const {
                name: symbol,
                type_id: SOURCE_BOOL.into(),
            })
        || !block.literal_values.iter().any(|l| {
            Some(&l.result.id) == block.abrupt_value_id.as_ref()
                && l.result.type_id == EXCEPTION_TYPE_ID
                && l.value
                    == (MonomorphicValue::ClosedException {
                        type_id: EXCEPTION_TYPE_ID.into(),
                        tag: 8,
                        source_type_id: None,
                        payload: None,
                    })
        })
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(Some(node.id.clone()))
}
fn join(
    c: &mut Clauses<'_>,
    flow: &ControlFunctionVc,
    edge: &ControlFlowEdge,
    guard: &str,
) -> R<Option<OrdinaryControlEdgeJoin>> {
    let Some(target) = &edge.target_node_id else {
        return Ok(None);
    };
    // Entry assignment is the separately source-bound slot entry relation.
    if edge.kind == "function_entry" {
        return Ok(None);
    }
    let mut args = edge.guard.bindings.clone();
    for (slot, ty) in &flow.slots {
        for source in [true, false] {
            for assigned in [true, false] {
                args.push(ControlBinding {
                    kind: if assigned {
                        "slot_assigned"
                    } else {
                        "current_slot"
                    }
                    .into(),
                    edge_id: Some(edge.id.clone()),
                    // W04 at_edge/decreasing attaches post-edge observations
                    // to the target cutpoint. The edge ID distinguishes the
                    // incoming snapshot from the previous iteration entry.
                    // The producer side is the source exit for this exact edge.
                    node_id: if source {
                        edge.source_node_id.clone()
                    } else {
                        target.clone()
                    },
                    value_id: slot.clone(),
                    type_id: if assigned {
                        SOURCE_BOOL.into()
                    } else {
                        ty.clone()
                    },
                });
            }
        }
    }
    let terms = (0..args.len())
        .map(|i| {
            c.b.var(
                (args.len() - 1 - i)
                    .try_into()
                    .map_err(|_| OrdinaryCarrierError::Limit)?,
            )
        })
        .collect::<R<Vec<_>>>()?;
    let n = edge.guard.bindings.len();
    let guarded = call(&mut c.b, guard, terms[..n].to_vec())?;
    let mut body = bit(&mut c.b, true)?;
    let bool_equal = construction_data::physical_equal(&mut c.b, 0)?;
    for (i, (_, ty)) in flow.slots.iter().enumerate() {
        let offset = n + 4 * i;
        let (source_assigned, source, target_assigned, target) = (
            terms[offset],
            terms[offset + 1],
            terms[offset + 2],
            terms[offset + 3],
        );
        let flags = call(
            &mut c.b,
            &bool_equal,
            vec![source_assigned, target_assigned],
        )?;
        let depth = c
            .carriers
            .get(ty.as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?
            .depth;
        let equal = construction_data::physical_equal(&mut c.b, depth)?;
        let same = call(&mut c.b, &equal, vec![source, target])?;
        let yes = bit(&mut c.b, true)?;
        let same = mux(&mut c.b, source_assigned, same, yes)?;
        let frame = call(&mut c.b, "Std.Bool.and", vec![flags, same])?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, frame])?;
    }
    let yes = bit(&mut c.b, true)?;
    body = mux(&mut c.b, guarded, body, yes)?;
    for arg in args.iter().rev() {
        let ty = c.ty(&arg.type_id, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let definition = name("Join", &(&flow.function_id, edge, &args));
    let ty = c.ty(
        &signature(
            &args.iter().map(|a| a.type_id.clone()).collect::<Vec<_>>(),
            SOURCE_BOOL,
        ),
        0,
    )?;
    c.b.define(&definition, ty, body)?;
    Ok(Some(OrdinaryControlEdgeJoin {
        arguments: args,
        guard_argument_count: n,
        state_rule: "source_exit_to_edge_specific_target; node_entry_merge_separate".into(),
        definition,
    }))
}
fn phi_join(
    c: &mut Clauses<'_>,
    function: &crate::csharp_practical_vir_validation::PracticalVirFunction,
    edge: &ControlFlowEdge,
    guard: &str,
) -> R<Option<OrdinaryControlEdgeJoin>> {
    let Some(target_id) = &edge.target_node_id else {
        return Ok(None);
    };
    let target = function
        .blocks
        .iter()
        .find(|b| &b.node.id == target_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if target.phi_values.is_empty() {
        return Ok(None);
    }
    let mut args = edge.guard.bindings.clone();
    for phi in &target.phi_values {
        let incoming = phi
            .incoming
            .iter()
            .find(|i| i.predecessor_node_id == edge.source_node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for (node, value) in [
            (&edge.source_node_id, &incoming.value_id),
            (target_id, &phi.value.id),
        ] {
            args.push(ControlBinding {
                kind: "ssa".into(),
                node_id: node.clone(),
                edge_id: Some(edge.id.clone()),
                value_id: value.clone(),
                type_id: phi.value.type_id.clone(),
            });
        }
    }
    let terms = (0..args.len())
        .map(|i| {
            c.b.var(
                (args.len() - 1 - i)
                    .try_into()
                    .map_err(|_| OrdinaryCarrierError::Limit)?,
            )
        })
        .collect::<R<Vec<_>>>()?;
    let n = edge.guard.bindings.len();
    let guarded = call(&mut c.b, guard, terms[..n].to_vec())?;
    let mut body = bit(&mut c.b, true)?;
    // Read every predecessor value before assigning any target phi. In
    // particular, a backedge may swap two phis or refer to its previous value.
    for (i, phi) in target.phi_values.iter().enumerate() {
        let depth = c
            .carriers
            .get(phi.value.type_id.as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?
            .depth;
        let equal = construction_data::physical_equal(&mut c.b, depth)?;
        let same = call(
            &mut c.b,
            &equal,
            vec![terms[n + 2 * i], terms[n + 2 * i + 1]],
        )?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, same])?;
    }
    let yes = bit(&mut c.b, true)?;
    body = mux(&mut c.b, guarded, body, yes)?;
    for arg in args.iter().rev() {
        let ty = c.ty(&arg.type_id, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let definition = name("PhiJoin", &(&function.id, edge, &target.phi_values, &args));
    let ty = c.ty(
        &signature(
            &args.iter().map(|a| a.type_id.clone()).collect::<Vec<_>>(),
            SOURCE_BOOL,
        ),
        0,
    )?;
    c.b.define(&definition, ty, body)?;
    Ok(Some(OrdinaryControlEdgeJoin {
        arguments: args,
        guard_argument_count: n,
        state_rule:
            "parallel_phi_inputs_at_source_exit_to_edge_specific_target; node_entry_merge_separate"
                .into(),
        definition,
    }))
}
fn memory_bindings(
    c: &mut Clauses<'_>,
    flow: &ControlFunctionVc,
    native: &crate::csharp_practical_vir_validation::PracticalVirFunction,
    construction_ids: &BTreeSet<&str>,
    ownership: &[OrdinaryOwnershipFunction],
    proofs: &[OrdinaryOwnershipProof],
) -> R<Vec<OrdinaryControlMemoryBinding>> {
    let mut bindings = vec![];
    for transfer in &flow.transfers {
        if !construction_ids.contains(transfer.value.type_id.as_str()) {
            continue;
        }
        let owner = ownership
            .iter()
            .find(|o| o.source.function_id == flow.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let proof = proofs
            .iter()
            .find(|p| p.function_id == flow.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if !c.b.globals.contains_key(&proof.flow_theorem) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        // The source anchor carries an array identity. Its immutable SSA name
        // may still denote the allocation's initial contents after a rewrite.
        // Resolve that identity through the independently reconstructed flow.
        let origins = owner
            .states
            .iter()
            .flat_map(|s| &s.live)
            .filter(|(origin, value)| *origin == &transfer.value.id || *value == &transfer.value.id)
            .map(|(origin, _)| origin)
            .collect::<BTreeSet<_>>();
        if origins.len() != 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let origin = *origins.first().ok_or(OrdinaryCarrierError::Linkage)?;
        let (node, role) = match transfer.kind.as_str() {
            "load" => (&transfer.entry_node_id, "invoke"),
            "store" | "pattern_bind" => (&transfer.exit_node_id, "normal"),
            _ => return Err(OrdinaryCarrierError::Linkage),
        };
        let state_id = format!("{node}.{role}");
        let state = owner
            .states
            .iter()
            .find(|s| s.id == state_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let current_id = state
            .live
            .get(origin)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let current = native
            .parameter_values
            .iter()
            .chain(native.blocks.iter().flat_map(|b| {
                b.phi_values
                    .iter()
                    .map(|p| &p.value)
                    .chain(b.literal_values.iter().map(|v| &v.result))
                    .chain(b.invocation.iter().map(|i| &i.result))
            }))
            .find(|v| &v.id == current_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if current.type_id != transfer.value.type_id {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let source_slot_type_id = flow
            .slots
            .iter()
            .find(|(s, _)| s == &transfer.slot)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .1
            .clone();
        let arguments = vec![
            ControlBinding {
                kind: "native_slot".into(),
                node_id: node.clone(),
                edge_id: None,
                value_id: transfer.slot.clone(),
                type_id: current.type_id.clone(),
            },
            ControlBinding {
                kind: "ssa".into(),
                node_id: node.clone(),
                edge_id: None,
                value_id: current.id.clone(),
                type_id: current.type_id.clone(),
            },
        ];
        let depth = c
            .carriers
            .get(current.type_id.as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?
            .depth;
        let equal = construction_data::physical_equal(&mut c.b, depth)?;
        let slot = c.b.var(1)?;
        let memory = c.b.var(0)?;
        let body = call(&mut c.b, &equal, vec![slot, memory])?;
        let definition = name(
            "MemorySlot",
            &(&flow.function_id, transfer, origin, &state_id, &arguments),
        );
        define(&mut c.b, &definition, &[depth, depth], 0, body)?;
        bindings.push(OrdinaryControlMemoryBinding {
            transfer: transfer.clone(),
            source_slot_type_id,
            allocation_origin_id: origin.clone(),
            state_id,
            flow_theorem: proof.flow_theorem.clone(),
            arguments,
            definition,
            public_slot_projection_pending: true,
            public_projection_definedness: None,
            public_projection_definition: None,
            source_slot_snapshot_definition: None,
        });
    }
    Ok(bindings)
}
fn node_entry(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
    block: &crate::csharp_practical_vir_validation::PracticalVirBlock,
) -> R<OrdinaryControlNodeEntry> {
    let state_count = flow
        .source
        .slots
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(block.phi_values.len()))
        .filter(|n| *n <= 256)
        .ok_or(OrdinaryCarrierError::Limit)?;
    let mut state = vec![];
    for (slot, nominal) in &flow.source.slots {
        let ty = flow.slot_type_overrides.get(slot).unwrap_or(nominal);
        for assigned in [true, false] {
            state.push(ControlBinding {
                kind: if assigned {
                    "slot_assigned"
                } else {
                    "current_slot"
                }
                .into(),
                edge_id: None,
                node_id: block.node.id.clone(),
                value_id: slot.clone(),
                type_id: if assigned {
                    SOURCE_BOOL.into()
                } else {
                    ty.clone()
                },
            });
        }
    }
    for phi in &block.phi_values {
        state.push(ControlBinding {
            kind: "ssa".into(),
            edge_id: None,
            node_id: block.node.id.clone(),
            value_id: phi.value.id.clone(),
            type_id: phi.value.type_id.clone(),
        });
    }
    let mut arguments = state.clone();
    let mut incoming = vec![];
    let edges = flow
        .edges
        .iter()
        .filter(|e| e.source.target_node_id.as_ref() == Some(&block.node.id))
        .collect::<Vec<_>>();
    // Factor only when needed. Bound both the selection component and each
    // arrival before materializing the expanded shared metadata.
    let expanded = edges.iter().try_fold(state_count, |count, edge| {
        count
            .checked_add(1 + edge.source.guard.bindings.len())
            .and_then(|n| n.checked_add(state_count))
            .ok_or(OrdinaryCarrierError::Limit)
    })?;
    if expanded > 256
        && (edges.len() > 256
            || edges
                .iter()
                .any(|edge| state_count * 2 + 1 + edge.source.guard.bindings.len() > 256))
    {
        return Err(OrdinaryCarrierError::Limit);
    }
    for edge in &edges {
        let selected_argument = arguments.len();
        arguments.push(ControlBinding {
            kind: "incoming_edge_selected".into(),
            edge_id: Some(edge.source.id.clone()),
            node_id: block.node.id.clone(),
            value_id: edge.source.id.clone(),
            type_id: SOURCE_BOOL.into(),
        });
        let guard_argument_start = arguments.len();
        arguments.extend(edge.source.guard.bindings.clone());
        let state_argument_start = arguments.len();
        arguments.extend(state.iter().cloned().map(|mut a| {
            a.edge_id = Some(edge.source.id.clone());
            a
        }));
        incoming.push(OrdinaryControlEntryIncoming {
            edge_id: edge.source.id.clone(),
            selected_argument,
            guard_argument_start,
            guard_argument_count: edge.source.guard.bindings.len(),
            state_argument_start,
            pending_constant_names: edge.pending_constant_names.clone(),
        });
    }
    let mut entry = OrdinaryControlNodeEntry {
        node_id: block.node.id.clone(),
        entry_state_argument_count: state.len(),
        arguments,
        incoming,
        state_rule: concat!(
            "exactly_one_enabled_incoming_edge; ",
            "selected_edge_snapshot_to_fresh_node_entry; native_execution_pending"
        )
        .into(),
        definition: None,
        components: vec![],
    };
    if edges.iter().any(|e| e.guard_definition.is_none()) {
        return Ok(entry);
    }
    if expanded > 256 {
        factor_node_entry(c, flow, &mut entry, &edges)?;
        return Ok(entry);
    }
    let terms = (0..entry.arguments.len())
        .map(|i| c.b.var((entry.arguments.len() - 1 - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let mut seen = bit(&mut c.b, false)?;
    let mut body = bit(&mut c.b, true)?;
    for (edge, incoming) in edges.iter().zip(&entry.incoming) {
        let selected = terms[incoming.selected_argument];
        let duplicate = call(&mut c.b, "Std.Bool.and", vec![seen, selected])?;
        let unique = call(&mut c.b, "Std.Bool.not", vec![duplicate])?;
        seen = call(&mut c.b, "Std.Bool.or", vec![seen, selected])?;
        let guard = edge
            .guard_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let start = incoming.guard_argument_start;
        let mut allowed = call(
            &mut c.b,
            guard,
            terms[start..start + incoming.guard_argument_count].to_vec(),
        )?;
        for (i, arg) in state.iter().enumerate() {
            let depth = c
                .carriers
                .get(arg.type_id.as_str())
                .ok_or(OrdinaryCarrierError::Linkage)?
                .depth;
            let equal = construction_data::physical_equal(&mut c.b, depth)?;
            let mut same = call(
                &mut c.b,
                &equal,
                vec![terms[i], terms[incoming.state_argument_start + i]],
            )?;
            if arg.kind == "current_slot" {
                // The preceding assignedness flag is compared independently.
                // Unassigned payload bits are irrelevant; native phis are not.
                let yes = bit(&mut c.b, true)?;
                same = mux(
                    &mut c.b,
                    terms[incoming.state_argument_start + i - 1],
                    same,
                    yes,
                )?;
            }
            allowed = call(&mut c.b, "Std.Bool.and", vec![allowed, same])?;
        }
        let yes = bit(&mut c.b, true)?;
        let selected_valid = mux(&mut c.b, selected, allowed, yes)?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, unique])?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, selected_valid])?;
    }
    body = call(&mut c.b, "Std.Bool.and", vec![body, seen])?;
    for arg in entry.arguments.iter().rev() {
        let ty = c.ty(&arg.type_id, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let definition = name("NodeEntry", &(&flow.source.function_id, &entry));
    let ty = c.ty(
        &signature(
            &entry
                .arguments
                .iter()
                .map(|a| a.type_id.clone())
                .collect::<Vec<_>>(),
            SOURCE_BOOL,
        ),
        0,
    )?;
    c.b.define(&definition, ty, body)?;
    entry.definition = Some(definition);
    Ok(entry)
}

fn define_entry_component(
    c: &mut Clauses<'_>,
    function_id: &str,
    entry: &OrdinaryControlNodeEntry,
    role: String,
    argument_indices: Vec<usize>,
    mut body: u32,
) -> R<OrdinaryControlEntryComponent> {
    let definition = name(
        "NodeEntryComponent",
        &(function_id, entry, &role, &argument_indices),
    );
    for &i in argument_indices.iter().rev() {
        let ty = c.ty(&entry.arguments[i].type_id, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let ty = c.ty(
        &signature(
            &argument_indices
                .iter()
                .map(|&i| entry.arguments[i].type_id.clone())
                .collect::<Vec<_>>(),
            SOURCE_BOOL,
        ),
        0,
    )?;
    c.b.define(&definition, ty, body)?;
    Ok(OrdinaryControlEntryComponent {
        role,
        argument_indices,
        definition,
    })
}

fn factor_node_entry(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
    entry: &mut OrdinaryControlNodeEntry,
    edges: &[&OrdinaryControlEdgeDefinition],
) -> R<()> {
    // Selection and arrivals share original indexed arguments. In particular,
    // an arrival cannot substitute a fresh, unconstrained selection Boolean.
    let selection = entry
        .incoming
        .iter()
        .map(|i| i.selected_argument)
        .collect::<Vec<_>>();
    let mut seen = bit(&mut c.b, false)?;
    let mut body = bit(&mut c.b, true)?;
    for i in 0..selection.len() {
        let selected = c.b.var((selection.len() - 1 - i) as u32)?;
        let duplicate = call(&mut c.b, "Std.Bool.and", vec![seen, selected])?;
        let unique = call(&mut c.b, "Std.Bool.not", vec![duplicate])?;
        seen = call(&mut c.b, "Std.Bool.or", vec![seen, selected])?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, unique])?;
    }
    body = call(&mut c.b, "Std.Bool.and", vec![body, seen])?;
    let mut components = vec![define_entry_component(
        c,
        &flow.source.function_id,
        entry,
        "exactly_one_incoming".into(),
        selection,
        body,
    )?];
    let n = entry.entry_state_argument_count;
    for (incoming, edge) in entry.incoming.iter().zip(edges) {
        let indices = (0..n)
            .chain(incoming.selected_argument..incoming.state_argument_start + n)
            .collect::<Vec<_>>();
        let terms = (0..indices.len())
            .map(|i| c.b.var((indices.len() - 1 - i) as u32))
            .collect::<R<Vec<_>>>()?;
        let guard_start = n + 1;
        let state_start = guard_start + incoming.guard_argument_count;
        let mut allowed = call(
            &mut c.b,
            edge.guard_definition
                .as_ref()
                .ok_or(OrdinaryCarrierError::Linkage)?,
            terms[guard_start..state_start].to_vec(),
        )?;
        for (i, arg) in entry.arguments[..n].iter().enumerate() {
            let depth = c
                .carriers
                .get(arg.type_id.as_str())
                .ok_or(OrdinaryCarrierError::Linkage)?
                .depth;
            let equal = construction_data::physical_equal(&mut c.b, depth)?;
            let mut same = call(&mut c.b, &equal, vec![terms[i], terms[state_start + i]])?;
            if arg.kind == "current_slot" {
                let yes = bit(&mut c.b, true)?;
                same = mux(&mut c.b, terms[state_start + i - 1], same, yes)?;
            }
            allowed = call(&mut c.b, "Std.Bool.and", vec![allowed, same])?;
        }
        let yes = bit(&mut c.b, true)?;
        let body = mux(&mut c.b, terms[n], allowed, yes)?;
        components.push(define_entry_component(
            c,
            &flow.source.function_id,
            entry,
            format!("selected_arrival:{}", incoming.edge_id),
            indices,
            body,
        )?);
    }
    entry.components = components;
    Ok(())
}
fn memory_effect(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
    native: &crate::csharp_practical_vir_validation::PracticalVirFunction,
    operation: &crate::csharp_practical_vir_model::data_vc::DataOperationVc,
    definitions: &[OrdinaryConstructionDataDefinition],
    owners: &[OrdinaryOwnershipFunction],
) -> R<Option<OrdinaryControlMemoryEffect>> {
    let (Some(graph), Some(protocol)) = (&flow.source.source_graph, &native.control_protocol)
    else {
        return Ok(None);
    };
    let Some((source, anchor)) = graph
        .nodes
        .iter()
        .filter(|n| n.operation == "update")
        .find_map(|n| {
            protocol
                .anchors
                .iter()
                .find(|a| {
                    a.source_node_id == n.id && a.artifact_node_ids.contains(&operation.node_id)
                })
                .map(|a| (n, a))
        })
    else {
        return Ok(None);
    };
    let Some(receiver) = graph
        .nodes
        .iter()
        .find(|n| source.inputs.first() == Some(&n.result) && n.operation == "load")
    else {
        return Ok(None);
    };
    let Some(transfer) = flow
        .source
        .transfers
        .iter()
        .find(|t| t.source_node_id == receiver.id && t.kind == "load")
    else {
        return Ok(None);
    };
    // Receiver evaluation happens before index/value evaluation. If those
    // expressions can reassign its source variable, updating that variable's
    // current snapshot needs a separate alias/identity relation.
    let mut seen = BTreeSet::new();
    let mut todo = receiver.successors.clone();
    let mut reached = false;
    while let Some(id) = todo.pop() {
        if id == source.id {
            reached = true;
            continue;
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        let node = graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if node.slot == transfer.slot && matches!(node.operation.as_str(), "store" | "pattern_bind")
        {
            return Ok(None);
        }
        todo.extend(node.successors.iter().cloned());
    }
    if !reached {
        return Ok(None);
    }
    let Some(binding) = flow
        .memory_bindings
        .iter()
        .find(|b| &b.transfer == transfer)
    else {
        return Ok(None);
    };
    let Some(snapshot) = &binding.source_slot_snapshot_definition else {
        return Ok(None);
    };
    let Some(definition) = definitions
        .iter()
        .find(|d| d.source.id == operation.definition_id)
    else {
        return Ok(None);
    };
    let Some(ownership) = flow
        .edges
        .iter()
        .filter(|e| e.source.source_node_id == operation.node_id)
        .find_map(|e| e.ownership.as_ref())
    else {
        return Ok(None);
    };
    let Some(owner) = owners
        .iter()
        .find(|f| f.source.function_id == operation.function_id)
    else {
        return Ok(None);
    };
    let before_state_id = format!("{}.invoke", operation.node_id);
    let after_state_id = format!("{}.normal", operation.node_id);
    let before = owner
        .states
        .iter()
        .find(|s| s.id == before_state_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let after = owner
        .states
        .iter()
        .find(|s| s.id == after_state_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let Some(result) = &anchor.result else {
        return Ok(None);
    };
    if operation.subjects.len() != 4
        || source.inputs.len() != 3
        || definition.source.signature.argument_type_ids.len() != 3
        || before.live.get(&binding.allocation_origin_id)
            != operation.subjects.first().map(|s| &s.id)
        || after.live.get(&binding.allocation_origin_id) != operation.subjects.last().map(|s| &s.id)
        || operation.subjects[0].type_id != transfer.value.type_id
        || operation.subjects[3].type_id != transfer.value.type_id
        || ownership.receiver_id != operation.subjects[0].id
        || ownership.state_id != before_state_id
        || ownership.flow_theorem != binding.flow_theorem
        || result != &operation.subjects[2]
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut arguments = vec![];
    for side in ["before", "after"] {
        for assigned in [true, false] {
            arguments.push(ControlBinding {
                kind: format!(
                    "source_{side}_{}",
                    if assigned { "assigned" } else { "slot" }
                ),
                edge_id: None,
                node_id: if side == "before" {
                    operation.node_id.clone()
                } else {
                    operation.normal_successor_id.clone()
                },
                value_id: transfer.slot.clone(),
                type_id: if assigned {
                    SOURCE_BOOL.into()
                } else {
                    binding.source_slot_type_id.clone()
                },
            });
        }
    }
    arguments.extend(operation.subjects.iter().map(|s| ControlBinding {
        kind: "ssa".into(),
        edge_id: None,
        node_id: operation.node_id.clone(),
        value_id: s.id.clone(),
        type_id: s.type_id.clone(),
    }));
    let terms = (0..arguments.len())
        .map(|i| c.b.var((arguments.len() - 1 - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let inputs = terms[4..7].to_vec();
    let mut body = call(&mut c.b, "Std.Bool.and", vec![terms[0], terms[2]])?;
    for (i, failure) in definition.failure_definitions.iter().enumerate() {
        let failure = match failure {
            Some(f) => f,
            None if definition.source.signature.ordered_checks[i].id == "ownership" => {
                &ownership.scoped_failure_definition
            }
            None => return Ok(None),
        };
        let failed = call(&mut c.b, failure, inputs.clone())?;
        let passed = call(&mut c.b, "Std.Bool.not", vec![failed])?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, passed])?;
    }
    let update = call(
        &mut c.b,
        &definition.relation_definition,
        terms[4..8].to_vec(),
    )?;
    body = call(&mut c.b, "Std.Bool.and", vec![body, update])?;
    for (slot, value) in [(terms[1], terms[4]), (terms[3], terms[7])] {
        let same = call(&mut c.b, snapshot, vec![value, slot])?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, same])?;
    }
    let mut effect=OrdinaryControlMemoryEffect {
        source_node_id:source.id.clone(),receiver_transfer:transfer.clone(),source_result:result.clone(),operation:operation.clone(),
        allocation_origin_id:binding.allocation_origin_id.clone(),before_state_id,after_state_id,
        ownership:ownership.clone(),snapshot_definition:snapshot.clone(),arguments,
        execution_scope:"successful_native_update_of_receiver_slot; other_source_frames_and_exception_execution_pending".into(),definition:String::new(),
    };
    let name = name("MemoryEffect", &(&flow.source.function_id, &effect));
    for arg in effect.arguments.iter().rev() {
        let ty = c.ty(&arg.type_id, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let ty = c.ty(
        &signature(
            &effect
                .arguments
                .iter()
                .map(|a| a.type_id.clone())
                .collect::<Vec<_>>(),
            SOURCE_BOOL,
        ),
        0,
    )?;
    c.b.define(&name, ty, body)?;
    effect.definition = name;
    Ok(Some(effect))
}

fn source_frames(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
    native: &crate::csharp_practical_vir_validation::PracticalVirFunction,
) -> R<Vec<OrdinaryControlSourceFrame>> {
    let (Some(graph), Some(protocol)) = (&flow.source.source_graph, &native.control_protocol)
    else {
        return Ok(vec![]);
    };
    let nodes = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::new();
    let mut todo = vec![graph
        .nodes
        .first()
        .ok_or(OrdinaryCarrierError::Linkage)?
        .id
        .as_str()];
    while let Some(id) = todo.pop() {
        if reachable.insert(id) {
            let node = nodes.get(id).ok_or(OrdinaryCarrierError::Linkage)?;
            todo.extend(
                node.successors
                    .iter()
                    .chain(&node.exceptional_successors)
                    .map(String::as_str),
            );
        }
    }
    let mut frames = vec![];
    for node in &graph.nodes {
        let anchor = protocol
            .anchors
            .iter()
            .find(|a| a.source_node_id == node.id);
        let transfer = flow
            .source
            .transfers
            .iter()
            .find(|t| t.source_node_id == node.id);
        let pending = if !reachable.contains(node.id.as_str()) {
            Some("unreachable_source_node")
        } else if node.kind == "entry" {
            Some("function_entry_relation_separate")
        } else if matches!(node.kind.as_str(), "handler_search" | "builtin_throw") {
            Some("exception_execution_separate")
        } else if node.operation == "update" {
            Some("memory_alias_frames_pending")
        } else if anchor.is_none() {
            Some("native_anchor_missing")
        } else if matches!(node.operation.as_str(), "load" | "store" | "pattern_bind") {
            if transfer.is_none() {
                Some("source_transfer_missing")
            } else {
                None
            }
        } else if matches!(
            node.operation.as_str(),
            "constant"
                | "true"
                | "zero"
                | "binary"
                | "unary"
                | "unary_update"
                | "increment"
                | "less"
                | "convert"
                | "iteration_convert"
                | "join_value"
                | "length"
                | "element"
                | "member"
                | "construct"
                | "allocate"
                | "pattern_equal"
                | "pattern_relational"
                | "pattern_type"
                | "pattern_member"
                | "break"
                | "continue"
                | "return"
                | "normal"
        ) || (node.operation.is_empty()
            && matches!(
                node.kind.as_str(),
                "loop_header" | "jump" | "branch" | "pattern_decision"
            ))
        {
            None
        } else {
            Some("source_operation_frame_pending")
        };
        let mut frame = OrdinaryControlSourceFrame {
            source_node_id: node.id.clone(),
            source_operation: node.operation.clone(),
            entry_node_id: anchor.map(|a| a.entry_node_id.clone()),
            exit_node_id: anchor.map(|a| a.exit_node_id.clone()),
            transfer: transfer.cloned(),
            framed_slots: vec![],
            arguments: vec![],
            state_rule: concat!(
                "source_entry_to_exit_local_slots; transfer_target_separate; ",
                "native_result_execution_and_exception_state_separate"
            )
            .into(),
            pending_reason: pending.map(str::to_owned),
            definition: None,
        };
        if pending.is_some() {
            frames.push(frame);
            continue;
        }
        let anchor = anchor.ok_or(OrdinaryCarrierError::Linkage)?;
        if let Some(t) = transfer {
            if t.entry_node_id != anchor.entry_node_id
                || t.exit_node_id != anchor.exit_node_id
                || anchor.result.as_ref() != Some(&t.value)
                || t.kind != node.operation
                || t.slot != node.slot
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        for (slot, nominal) in &flow.source.slots {
            if transfer.is_some_and(|t| {
                t.slot == *slot && matches!(t.kind.as_str(), "store" | "pattern_bind")
            }) {
                continue;
            }
            frame.framed_slots.push(slot.clone());
            let ty = flow.slot_type_overrides.get(slot).unwrap_or(nominal);
            if frame.arguments.len() + 4 > 256 {
                return Err(OrdinaryCarrierError::Limit);
            }
            for (side, point) in [
                ("source_entry", &anchor.entry_node_id),
                ("source_exit", &anchor.exit_node_id),
            ] {
                for assigned in [true, false] {
                    frame.arguments.push(ControlBinding {
                        kind: format!("{side}_{}", if assigned { "assigned" } else { "slot" }),
                        edge_id: None,
                        node_id: point.clone(),
                        value_id: slot.clone(),
                        type_id: if assigned {
                            SOURCE_BOOL.into()
                        } else {
                            ty.clone()
                        },
                    });
                }
            }
        }
        // The equality body depends on slot storage types, not on a source
        // operation's result or reachability. Reuse it at exact anchored points.
        let types = frame
            .arguments
            .iter()
            .map(|a| a.type_id.clone())
            .collect::<Vec<_>>();
        let definition = name("SourceFrame", &types);
        if !c.b.globals.contains_key(&definition) {
            let terms = (0..types.len())
                .map(|i| c.b.var((types.len() - 1 - i) as u32))
                .collect::<R<Vec<_>>>()?;
            let mut body = bit(&mut c.b, true)?;
            for (i, args) in terms.chunks_exact(4).enumerate() {
                let flags = construction_data::physical_equal(&mut c.b, 0)?;
                let flags = call(&mut c.b, &flags, vec![args[0], args[2]])?;
                let depth = c
                    .carriers
                    .get(types[4 * i + 1].as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .depth;
                let same = construction_data::physical_equal(&mut c.b, depth)?;
                let same = call(&mut c.b, &same, vec![args[1], args[3]])?;
                let yes = bit(&mut c.b, true)?;
                let same = mux(&mut c.b, args[0], same, yes)?;
                let same = call(&mut c.b, "Std.Bool.and", vec![flags, same])?;
                body = call(&mut c.b, "Std.Bool.and", vec![body, same])?;
            }
            for ty in types.iter().rev() {
                let ty = c.ty(ty, 0)?;
                body = c.b.lam(ty, body)?;
            }
            let ty = c.ty(&signature(&types, SOURCE_BOOL), 0)?;
            c.b.define(&definition, ty, body)?;
        }
        frame.definition = Some(definition);
        frames.push(frame);
    }
    Ok(frames)
}

pub fn generate_csharp_practical_ordinary_control_edges(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlEdgeProgram> {
    let (builder, mut p) = emit_program(vir)?;
    // Slot generation consumes only the private edge/memory builder. Entry
    // merging is appended here, once the complete incoming guards are known.
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, builder, &[])?;
    for flow in &mut p.functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for block in &native.blocks {
            let entry = node_entry(&mut c, flow, block)?;
            flow.node_entries.push(entry);
        }
    }
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    for flow in &mut p.functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for operation in data
            .operations()
            .iter()
            .filter(|o| o.function_id == flow.source.function_id)
        {
            let block = native
                .blocks
                .iter()
                .find(|b| b.node.id == operation.node_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let Some(invocation) = &block.invocation else {
                continue;
            };
            if !invocation.operation_id.ends_with(".fill")
                && !invocation.operation_id.ends_with(".rewrite")
            {
                continue;
            }
            let effect = memory_effect(
                &mut c,
                flow,
                native,
                operation,
                &p.construction_definitions,
                &p.symbolic_ownership,
            )?;
            if let Some(effect) = effect {
                flow.memory_effects.push(effect);
            } else {
                flow.pending_memory_effect_node_ids
                    .push(operation.node_id.clone());
            }
        }
    }
    for flow in &mut p.functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        flow.source_frames = source_frames(&mut c, flow, native)?;
    }
    execution::append(&mut c, &mut p, vir)?;
    let certificate = c.b.finish()?;
    p.certificate_sha256 = mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate));
    p.certificate = certificate;
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub(super) fn emit_program(
    vir: &ValidatedPracticalVir,
) -> R<(Builder, OrdinaryControlEdgeProgram)> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let control = crate::csharp_practical_vir_model::generate_control_vcs(vir, &data)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut requested = BTreeSet::new();
    for flow in control.functions() {
        for edge in &flow.edges {
            constants(&edge.guard.term, &mut requested)?;
        }
    }
    let construction_ids = vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
        .map(|e| text(e, "instance_id"))
        .collect::<R<BTreeSet<_>>>()?;
    let requested_constructions = data
        .definitions()
        .iter()
        .filter(|d| {
            matches!(
                d.family,
                DataDefinitionFamily::Foundation | DataDefinitionFamily::SequenceOwnership
            ) && d
                .signature
                .id
                .strip_prefix("construction.complete.")
                .or_else(|| d.signature.id.rsplit_once('.').map(|(id, _)| id))
                .is_some_and(|id| construction_ids.contains(id))
                && (requested.contains(&d.relation_name)
                    || d.failure_names.iter().any(|n| requested.contains(n)))
        })
        .collect::<Vec<_>>();
    let needed_definition = |d: &DataSemanticDefinition| {
        requested.contains(&d.relation_name)
            || d.failure_names.iter().any(|n| requested.contains(n))
    };
    let sequence_ids = vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.bounded_sequence.v1")
        .map(|e| text(e, "instance_id"))
        .collect::<R<BTreeSet<_>>>()?;
    let requested_sequences = data
        .definitions()
        .iter()
        .filter(|d| {
            d.family == DataDefinitionFamily::Foundation
                && d.signature
                    .id
                    .rsplit_once('.')
                    .is_some_and(|(id, _)| sequence_ids.contains(id))
                && needed_definition(d)
        })
        .collect::<Vec<_>>();
    let requested_references = data
        .definitions()
        .iter()
        .filter(|d| {
            d.family == DataDefinitionFamily::NullableOutcome
                && d.signature.id.starts_with("reference.value.")
                && needed_definition(d)
        })
        .collect::<Vec<_>>();
    let option_ids = vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.option.v1")
        .map(|e| text(e, "instance_id"))
        .collect::<R<BTreeSet<_>>>()?;
    let requested_options = data
        .definitions()
        .iter()
        .filter(|d| {
            d.family == DataDefinitionFamily::Foundation
                && d.signature
                    .id
                    .rsplit_once('.')
                    .is_some_and(|(id, _)| option_ids.contains(id))
                && needed_definition(d)
        })
        .collect::<Vec<_>>();
    let mut builder = Builder::new()?;
    let mut foundations = vec![];
    let mut symbolic_ownership = vec![];
    let mut symbolic_ownership_proofs = vec![];
    let mut pending_concrete_ownership_functions = vec![];
    let has_memory_transfers = control.functions().iter().any(|f| {
        f.transfers
            .iter()
            .any(|t| construction_ids.contains(t.value.type_id.as_str()))
    });
    if !requested_constructions.is_empty() || has_memory_transfers {
        (builder, foundations) =
            super::super::construction_ops::emit_definitions(vir, &layouts, builder)?;
        (
            builder,
            symbolic_ownership,
            pending_concrete_ownership_functions,
        ) = super::super::ownership_flow::emit_definitions(vir, builder)?;
        (builder, symbolic_ownership_proofs) =
            super::super::ownership_proofs::emit_proofs(builder, &symbolic_ownership)?;
    }
    let mut sequences = vec![];
    let mut options = vec![];
    if !requested_sequences.is_empty()
        || !requested_references.is_empty()
        || !requested_options.is_empty()
    {
        (builder, sequences, options) = relations::emit_data_foundations(
            vir,
            &layouts,
            builder,
            !requested_sequences.is_empty(),
            !requested_references.is_empty() || !requested_options.is_empty(),
        )?;
    }
    let mut c = compiler(vir, &layouts, builder, &[])?;
    c.definedness_logic()?;
    let mut construction_definitions = vec![];
    for d in requested_constructions {
        let id = d
            .signature
            .id
            .strip_prefix("construction.complete.")
            .or_else(|| d.signature.id.rsplit_once('.').map(|(id, _)| id));
        if let Some(foundation) = foundations
            .iter()
            .find(|f| Some(f.carrier.type_id.as_str()) == id)
        {
            construction_definitions.push(construction_data::emit(&mut c, d, foundation)?);
        }
    }
    let mut integer_definitions = vec![];
    for d in data.definitions().iter().filter(|d| {
        d.family == DataDefinitionFamily::IntegerBoolean
            && (requested.contains(&d.relation_name)
                || d.failure_names.iter().any(|n| requested.contains(n)))
    }) {
        integer_definitions.push(integer_data::emit_definition(&mut c, d)?);
    }
    let mut string_definitions = vec![];
    for d in data
        .definitions()
        .iter()
        .filter(|d| d.family == DataDefinitionFamily::String && needed_definition(d))
    {
        string_definitions.push(string_data::emit(&mut c, d)?);
    }
    let mut sequence_definitions = vec![];
    for d in requested_sequences {
        let id = d
            .signature
            .id
            .rsplit_once('.')
            .ok_or(OrdinaryCarrierError::Linkage)?
            .0;
        let operation = sequences
            .iter()
            .find(|s| s.carrier.type_id == id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        sequence_definitions.push(sequence_data::emit(&mut c, d, operation)?);
    }
    let mut reference_definitions = vec![];
    for d in requested_references {
        let id = d
            .signature
            .id
            .strip_prefix("reference.value.")
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let operation = options
            .iter()
            .flat_map(|o| &o.operations)
            .find(|o| o.operation_id == format!("{id}.value"))
            .ok_or(OrdinaryCarrierError::Linkage)?;
        reference_definitions.push(reference_data::emit(&mut c, d, operation)?);
    }
    let mut option_definitions = vec![];
    for d in requested_options {
        let operation = options
            .iter()
            .flat_map(|o| &o.operations)
            .find(|o| o.operation_id == d.signature.id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        option_definitions.push(option_data::emit(&mut c, d, operation)?);
    }
    let mut functions = vec![];
    // The same operation may feed multiple edges. Emit its source-bound proof
    // once, but install the failure alias only while lowering that exact edge.
    let mut ownership_uses = BTreeMap::new();
    for flow in control.functions() {
        let slot_type_overrides = control_slots::represented_slots(vir, flow)?;
        let mut storage_flow = flow.clone();
        for (slot, ty) in &mut storage_flow.slots {
            if let Some(storage) = slot_type_overrides.get(slot) {
                *ty = storage.clone();
            }
        }
        let mut edges = vec![];
        for edge in &flow.edges {
            let mut needed = BTreeSet::new();
            constants(&edge.guard.term, &mut needed)?;
            let operation = data
                .operations()
                .iter()
                .find(|o| o.function_id == flow.function_id && o.node_id == edge.source_node_id);
            let definition = operation.and_then(|o| {
                construction_definitions.iter().find(|d| {
                    d.source.id == o.definition_id
                        && d.source
                            .failure_names
                            .iter()
                            .zip(&d.failure_definitions)
                            .any(|(n, f)| f.is_none() && needed.contains(n))
                })
            });
            let mut ownership = None;
            if let (Some(o), Some(d)) = (operation, definition) {
                if !ownership_uses.contains_key(&o.id) {
                    let binding = construction_data::ownership_use(
                        &mut c,
                        o,
                        d,
                        &symbolic_ownership,
                        &symbolic_ownership_proofs,
                    )?;
                    if let Some(binding) = &binding {
                        c.constants.remove(&binding.source_failure_name);
                    }
                    ownership_uses.insert(o.id.clone(), binding);
                }
                ownership = ownership_uses[&o.id].clone();
                if let Some(binding) = &ownership {
                    if c.constants
                        .insert(
                            binding.source_failure_name.clone(),
                            (
                                signature(&d.source.signature.argument_type_ids, SOURCE_BOOL),
                                binding.scoped_failure_definition.clone(),
                            ),
                        )
                        .is_some()
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                }
            }
            let pending_constant_names = needed
                .into_iter()
                .filter(|n| !c.constants.contains_key(n))
                .collect::<Vec<_>>();
            let (guard_definition, join) = if pending_constant_names.is_empty() {
                let guard = name("Guard", &(&flow.function_id, edge));
                let subjects = edge
                    .guard
                    .bindings
                    .iter()
                    .map(|b| TypedValueRef {
                        id: b.value_id.clone(),
                        type_id: b.type_id.clone(),
                    })
                    .collect::<Vec<_>>();
                integer_data::predicate(&mut c, &guard, &subjects, &edge.guard.term)?;
                let join = join(&mut c, &storage_flow, edge, &guard)?;
                (Some(guard), join)
            } else {
                (None, None)
            };
            if let Some(binding) = &ownership {
                c.constants.remove(&binding.source_failure_name);
            }
            edges.push(OrdinaryControlEdgeDefinition {
                source: edge.clone(),
                guard_definition,
                join,
                phi_join: None,
                ownership,
                pending_constant_names,
                builtin_throw_source_node_id: None,
            });
        }
        functions.push(OrdinaryControlEdgeFunction {
            source: flow.clone(),
            edges,
            memory_bindings: vec![],
            slot_type_overrides,
            node_entries: vec![],
            memory_effects: vec![],
            pending_memory_effect_node_ids: vec![],
            source_frames: vec![],
            native_operations: vec![],
            pending_native_invocation_node_ids: vec![],
        });
    }
    // Append after the existing guard/slot declarations so their exact terms
    // remain intact. A pending guard cannot authorize a phi transition.
    for flow in &mut functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for edge in &mut flow.edges {
            if let Some(guard) = &edge.guard_definition {
                edge.phi_join = phi_join(&mut c, native, &edge.source, guard)?;
            }
        }
    }
    for flow in &mut functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        flow.memory_bindings = memory_bindings(
            &mut c,
            &flow.source,
            native,
            &construction_ids,
            &symbolic_ownership,
            &symbolic_ownership_proofs,
        )?;
    }
    // These are read-only snapshots, not native freeze operations. Their
    // complete/bound conditions remain obligations at each successful use.
    for flow in &mut functions {
        for binding in &mut flow.memory_bindings {
            let Some(foundation) = foundations.iter().find(|f| {
                f.carrier.type_id == binding.transfer.value.type_id
                    && f.published_type_id == binding.source_slot_type_id
            }) else {
                continue;
            };
            let freeze = foundation
                .operations
                .iter()
                .find(|o| o.operation_id.ends_with(".freeze"))
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let bound = freeze
                .failures
                .iter()
                .find(|f| f.label == "publication_bound")
                .and_then(|f| f.definition.as_ref())
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let definedness = name("SnapshotDefined", &foundation.carrier.type_id);
            if !c.b.globals.contains_key(&definedness) {
                let native = c.b.var(0)?;
                let prefix = construction_data::initialized_prefix(&mut c.b, 14)?;
                let length = call(&mut c.b, &foundation.length_definition, vec![native])?;
                // Initialization is product role 2, padded before its C14 child.
                let mut address = vec![false, true];
                address.extend(vec![false; (foundation.carrier.depth - 16) as usize]);
                let mut bitmap = native;
                for on in address {
                    let selector = bit(&mut c.b, on)?;
                    bitmap = c.b.app(bitmap, vec![selector])?;
                }
                let complete = call(&mut c.b, &prefix, vec![length, bitmap])?;
                let exceeds = call(&mut c.b, bound, vec![native])?;
                let bounded = call(&mut c.b, "Std.Bool.not", vec![exceeds])?;
                let body = call(&mut c.b, "Std.Bool.and", vec![bounded, complete])?;
                define(&mut c.b, &definedness, &[foundation.carrier.depth], 0, body)?;
            }
            let definition = name("PublicSnapshot", &foundation.carrier.type_id);
            if !c.b.globals.contains_key(&definition) {
                let published = c
                    .carriers
                    .get(foundation.published_type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let native = c.b.var(1)?;
                let valid = call(&mut c.b, &definedness, vec![native])?;
                let same =
                    construction_data::freeze_relation(&mut c.b, foundation, published.depth - 13)?;
                let body = call(&mut c.b, "Std.Bool.and", vec![valid, same])?;
                define(
                    &mut c.b,
                    &definition,
                    &[foundation.carrier.depth, published.depth],
                    0,
                    body,
                )?;
            }
            // A source identifier can refer to a partial construction for
            // first writes and initialized reads. Copy its storage without
            // imposing publication completeness on the identifier itself.
            let snapshot = name("SourceSlotSnapshot", &foundation.carrier.type_id);
            if !c.b.globals.contains_key(&snapshot) {
                let published = c
                    .carriers
                    .get(foundation.published_type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let native = c.b.var(1)?;
                let exceeds = call(&mut c.b, bound, vec![native])?;
                let bounded = call(&mut c.b, "Std.Bool.not", vec![exceeds])?;
                let same =
                    construction_data::freeze_relation(&mut c.b, foundation, published.depth - 13)?;
                let body = call(&mut c.b, "Std.Bool.and", vec![bounded, same])?;
                define(
                    &mut c.b,
                    &snapshot,
                    &[foundation.carrier.depth, published.depth],
                    0,
                    body,
                )?;
            }
            binding.source_slot_snapshot_definition = Some(snapshot);
            binding.public_projection_definedness = Some(definedness);
            binding.public_projection_definition = Some(definition);
            binding.public_slot_projection_pending = false;
        }
    }
    // Append local throw semantics after existing relations. Reaching this
    // captured built-in throw produces the frozen tag-8 exception; no test or
    // host reachability observation is used to decide whether it is reached.
    for flow in &mut functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let mut storage_flow = flow.source.clone();
        for (slot, ty) in &mut storage_flow.slots {
            if let Some(storage) = flow.slot_type_overrides.get(slot) {
                *ty = storage.clone();
            }
        }
        for edge in &mut flow.edges {
            let Some(source) = builtin_throw_source(&flow.source, native, &edge.source)? else {
                continue;
            };
            if edge.guard_definition.is_some() || edge.pending_constant_names.len() != 1 {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let symbol = edge.pending_constant_names[0].clone();
            let alias = name(
                "BuiltinThrow",
                &(&flow.source.function_id, &source, &edge.source),
            );
            let body = bit(&mut c.b, true)?;
            define(&mut c.b, &alias, &[], 0, body)?;
            if c.constants
                .insert(symbol.clone(), (SOURCE_BOOL.into(), alias))
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let guard = name("Guard", &(&flow.source.function_id, &edge.source));
            integer_data::predicate(&mut c, &guard, &[], &edge.source.guard.term)?;
            c.constants.remove(&symbol);
            edge.join = join(&mut c, &storage_flow, &edge.source, &guard)?;
            edge.phi_join = phi_join(&mut c, native, &edge.source, &guard)?;
            edge.guard_definition = Some(guard);
            edge.pending_constant_names.clear();
            edge.builtin_throw_source_node_id = Some(source);
        }
    }
    let p = OrdinaryControlEdgeProgram {
        schema: "mpk.csharp.ordinary_control_edges.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        control_vc_sha256: control.hash(),
        integer_definitions,
        construction_definitions,
        string_definitions,
        sequence_definitions,
        reference_definitions,
        option_definitions,
        native_definitions: vec![],
        symbolic_ownership,
        symbolic_ownership_proofs,
        pending_concrete_ownership_functions,
        functions,
        application_scope_pending: true,
        certificate_sha256: String::new(),
        certificate: vec![],
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok((c.b, p))
}
pub fn import_csharp_practical_ordinary_control_edges(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlEdgeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_control_edges(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
