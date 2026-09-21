//! Exact W04 guards and transport from source exit to edge-specific target slots.
//! Node-entry merging and execution produce separate, scoped obligations.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{DataDefinitionFamily, DataSemanticDefinition};
use crate::csharp_practical_vir_model::{
    ControlBinding, ControlFlowEdge, ControlFunctionVc, ControlSlotTransfer,
};
use sha2::{Digest, Sha256};

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
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeFunction {
    pub source: ControlFunctionVc,
    pub edges: Vec<OrdinaryControlEdgeDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub memory_bindings: Vec<OrdinaryControlMemoryBinding>,
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
        state_rule: "source_exit_to_edge_specific_target; node_entry_merge_pending".into(),
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
            "parallel_phi_inputs_at_source_exit_to_edge_specific_target; node_entry_merge_pending"
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
        });
    }
    Ok(bindings)
}
pub fn generate_csharp_practical_ordinary_control_edges(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlEdgeProgram> {
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
    if !requested_constructions.is_empty() {
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
                let join = join(&mut c, flow, edge, &guard)?;
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
            });
        }
        functions.push(OrdinaryControlEdgeFunction {
            source: flow.clone(),
            edges,
            memory_bindings: vec![],
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
    let certificate = c.b.finish()?;
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
        symbolic_ownership,
        symbolic_ownership_proofs,
        pending_concrete_ownership_functions,
        functions,
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
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
