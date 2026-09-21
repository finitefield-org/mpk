//! W04 slot entry and successful source load/store relations.
//! These are open state relations, not reachability or application proofs.
use super::*;
use crate::csharp_practical_vir_model::{ControlFunctionVc, ControlSlotTransfer};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSlotArgument {
    /// entry_assigned/value, before_assigned/value, after_assigned/value, or ssa.
    pub role: String,
    pub slot_id: String,
    pub node_id: String,
    pub value_id: Option<String>,
    pub type_id: String,
    pub depth: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSlotRelation {
    /// None denotes function entry; Some retains the exact source transfer.
    pub transfer: Option<ControlSlotTransfer>,
    /// function_entry or successful_transfer; never an exceptional edge.
    pub execution_scope: String,
    pub arguments: Vec<OrdinaryControlSlotArgument>,
    pub definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_binding: Option<OrdinaryControlMemoryBinding>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlSlotFunction {
    pub source: ControlFunctionVc,
    pub relations: Vec<OrdinaryControlSlotRelation>,
    /// Explicit native storage types; original source slots remain in `source`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub slot_type_overrides: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlSlotProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    control_vc_sha256: String,
    functions: Vec<OrdinaryControlSlotFunction>,
    pending_source_function_ids: Vec<String>,
    /// Edge joins, exception entry, reachability and native execution are separate.
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryControlSlotProgram {
    pub fn functions(&self) -> &[OrdinaryControlSlotFunction] {
        &self.functions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary control slot relations")
    }
}
fn and(b: &mut Builder, left: u32, right: u32) -> R<u32> {
    call(b, "Std.Bool.and", vec![left, right])
}
fn equal(b: &mut Builder, depth: u32, left: u32, right: u32) -> R<u32> {
    let name = construction_data::physical_equal(b, depth)?;
    call(b, &name, vec![left, right])
}
// A source control slot records the nominal payload type even when its
// validated native values retain nullable presence. Widen storage only to the
// exact closed Option<payload> instance used by those native values.
pub(super) fn represented_slots(
    vir: &ValidatedPracticalVir,
    flow: &ControlFunctionVc,
) -> R<BTreeMap<String, String>> {
    let payloads = option_payloads(vir)?;
    let mut overrides = BTreeMap::new();
    for (slot, source) in &flow.slots {
        for value in flow
            .entry_values
            .iter()
            .filter(|(s, _)| s == slot)
            .map(|(_, v)| v)
            .chain(
                flow.transfers
                    .iter()
                    .filter(|t| &t.slot == slot)
                    .map(|t| &t.value),
            )
        {
            if payloads.get(&value.type_id) == Some(source)
                && overrides
                    .insert(slot.clone(), value.type_id.clone())
                    .is_some_and(|old| old != value.type_id)
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
    }
    Ok(overrides)
}
fn option_payloads(vir: &ValidatedPracticalVir) -> R<BTreeMap<String, String>> {
    vir.data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.option.v1")
        .map(|e| {
            let id = text(e, "instance_id")?;
            let args = &vir.data_closed().metadata[id].argument_ids;
            if args.len() != 1 {
                return Err(OrdinaryCarrierError::Linkage);
            }
            Ok((id.to_owned(), args[0].clone()))
        })
        .collect()
}
fn some_storage(b: &mut Builder, child: u32) -> R<String> {
    let name = format!("{PREFIX}.ControlSlots.Some.D{child}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let depth = 1 + child.max(5);
    let input = b.var(depth)?;
    let selectors = b.selectors(child)?;
    let value = b.app(input, selectors)?;
    let value = zero_padding(b, depth, 1, depth - 1 - child, value)?;
    // A Some tag is the complete u32 word 1; every padding bit is zero.
    let tag = equal_address(b, depth, depth - 5, 5, 0)?;
    let tag = zero_padding(b, depth, 1, depth - 6, tag)?;
    let role = b.var(depth - 1)?;
    let body = mux(b, role, value, tag)?;
    let body = b.wrap_selectors(depth, body)?;
    define(b, &name, &[child], depth, body)?;
    Ok(name)
}
fn transfer_equal(
    b: &mut Builder,
    depth: u32,
    slot: u32,
    ssa: u32,
    payload_depth: Option<u32>,
    memory: Option<&OrdinaryControlMemoryBinding>,
) -> R<u32> {
    let ssa = if let Some(child) = payload_depth {
        if depth != 1 + child.max(5) || memory.is_some() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let some = some_storage(b, child)?;
        call(b, &some, vec![ssa])?
    } else {
        ssa
    };
    projected_equal(b, depth, slot, ssa, memory)
}
fn emit(
    b: &mut Builder,
    flow: &ControlFunctionVc,
    entry: &str,
    depths: &BTreeMap<&str, u32>,
    payloads: &BTreeMap<String, String>,
    transfer: Option<&ControlSlotTransfer>,
    memory_binding: Option<&OrdinaryControlMemoryBinding>,
) -> R<OrdinaryControlSlotRelation> {
    let mut args = vec![];
    for (slot, ty) in &flow.slots {
        for (prefix, node) in match transfer {
            Some(t) => vec![
                ("before", t.entry_node_id.as_str()),
                ("after", t.exit_node_id.as_str()),
            ],
            None => vec![("entry", entry)],
        } {
            for assigned in [true, false] {
                args.push(OrdinaryControlSlotArgument {
                    role: format!("{prefix}_{}", if assigned { "assigned" } else { "value" }),
                    slot_id: slot.clone(),
                    node_id: node.into(),
                    value_id: None,
                    type_id: if assigned {
                        SOURCE_BOOL.into()
                    } else {
                        ty.clone()
                    },
                    depth: if assigned {
                        0
                    } else {
                        *depths
                            .get(ty.as_str())
                            .ok_or(OrdinaryCarrierError::Linkage)?
                    },
                });
            }
        }
    }
    let values = match transfer {
        Some(t) => vec![(t.slot.as_str(), &t.value)],
        None => flow
            .entry_values
            .iter()
            .map(|(s, v)| (s.as_str(), v))
            .collect(),
    };
    for (slot, value) in &values {
        if !flow.slots.iter().any(|(s, ty)| {
            s == slot
                && (ty == &value.type_id
                    || payloads.get(ty) == Some(&value.type_id)
                    || memory_binding.is_some_and(|m| {
                        transfer == Some(&m.transfer)
                            && ty == &m.source_slot_type_id
                            && m.source_slot_snapshot_definition.is_some()
                            && !m.public_slot_projection_pending
                    }))
        }) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        args.push(OrdinaryControlSlotArgument {
            role: "ssa".into(),
            slot_id: (*slot).into(),
            node_id: memory_binding.map_or_else(
                || transfer.map_or(entry, |t| t.exit_node_id.as_str()).into(),
                |m| m.arguments[1].node_id.clone(),
            ),
            value_id: Some(
                memory_binding
                    .map_or_else(|| value.id.clone(), |m| m.arguments[1].value_id.clone()),
            ),
            type_id: value.type_id.clone(),
            depth: *depths
                .get(value.type_id.as_str())
                .ok_or(OrdinaryCarrierError::Linkage)?,
        });
    }
    let mut terms = Vec::with_capacity(args.len());
    for i in 0..args.len() {
        terms.push(
            b.var(
                (args.len() - i - 1)
                    .try_into()
                    .map_err(|_| OrdinaryCarrierError::Limit)?,
            )?,
        );
    }
    let mut body = bit(b, true)?;
    for (i, (slot, ty)) in flow.slots.iter().enumerate() {
        let depth = depths[ty.as_str()];
        let condition = if let Some(t) = transfer {
            let [before_assigned, before, after_assigned, after] = [
                terms[4 * i],
                terms[4 * i + 1],
                terms[4 * i + 2],
                terms[4 * i + 3],
            ];
            let ssa = *terms.last().ok_or(OrdinaryCarrierError::Linkage)?;
            let payload_depth = (slot == &t.slot && payloads.get(ty) == Some(&t.value.type_id))
                .then(|| depths[t.value.type_id.as_str()]);
            if slot == &t.slot && matches!(t.kind.as_str(), "store" | "pattern_bind") {
                let same = transfer_equal(b, depth, after, ssa, payload_depth, memory_binding)?;
                and(b, after_assigned, same)?
            } else {
                let flags = equal(b, 0, before_assigned, after_assigned)?;
                let same = equal(b, depth, before, after)?;
                let yes = bit(b, true)?;
                // Unassigned storage is unobservable; no invented zero default.
                let framed = mux(b, before_assigned, same, yes)?;
                let mut framed = and(b, flags, framed)?;
                if slot == &t.slot {
                    if t.kind != "load" {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    let same =
                        transfer_equal(b, depth, before, ssa, payload_depth, memory_binding)?;
                    let loaded = and(b, before_assigned, same)?;
                    framed = and(b, framed, loaded)?;
                }
                framed
            }
        } else if let Some(j) = values.iter().position(|(s, _)| s == slot) {
            let payload_depth = (payloads.get(ty) == Some(&values[j].1.type_id))
                .then(|| depths[values[j].1.type_id.as_str()]);
            let same = transfer_equal(
                b,
                depth,
                terms[2 * i + 1],
                terms[2 * flow.slots.len() + j],
                payload_depth,
                None,
            )?;
            and(b, terms[2 * i], same)?
        } else {
            call(b, "Std.Bool.not", vec![terms[2 * i]])?
        };
        body = and(b, body, condition)?;
    }
    let identity = (&flow.function_id, &flow.slots, transfer, &args);
    let definition = format!(
        "{PREFIX}.ControlSlots.Relation.H{:x}",
        Sha256::digest(serde_json::to_vec(&identity).expect("typed slot relation"))
    );
    define(
        b,
        &definition,
        &args.iter().map(|a| a.depth).collect::<Vec<_>>(),
        0,
        body,
    )?;
    Ok(OrdinaryControlSlotRelation {
        transfer: transfer.cloned(),
        execution_scope: if transfer.is_some() {
            "successful_transfer"
        } else {
            "function_entry"
        }
        .into(),
        arguments: args,
        definition,
        memory_binding: memory_binding.cloned(),
    })
}
fn projected_equal(
    b: &mut Builder,
    depth: u32,
    slot: u32,
    ssa: u32,
    memory: Option<&OrdinaryControlMemoryBinding>,
) -> R<u32> {
    if let Some(memory) = memory {
        // Lowering temporaries may already use the native construction type.
        // They still read the current memory version, without a public cast.
        if memory.source_slot_type_id == memory.arguments[1].type_id {
            return equal(b, depth, slot, ssa);
        }
        let definition = memory
            .source_slot_snapshot_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        call(b, definition, vec![ssa, slot])
    } else {
        equal(b, depth, slot, ssa)
    }
}
pub fn generate_csharp_practical_ordinary_control_slots(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlSlotProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let control = crate::csharp_practical_vir_model::generate_control_vcs(vir, &data)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let depths = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c.depth))
        .collect();
    let construction_ids = vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
        .map(|e| text(e, "instance_id"))
        .collect::<R<BTreeSet<_>>>()?;
    let payloads = option_payloads(vir)?;
    let needs_memory = control.functions().iter().any(|f| {
        f.transfers
            .iter()
            .any(|t| construction_ids.contains(t.value.type_id.as_str()))
    });
    let (mut b, memory_functions) = if needs_memory {
        let (b, program) = control_edges::emit_program(vir)?;
        (b, program.functions().to_vec())
    } else {
        (Builder::new()?, vec![])
    };
    let mut functions = vec![];
    let mut pending_source_function_ids = vec![];
    for flow in control.functions() {
        if flow.source_graph.is_none() {
            pending_source_function_ids.push(flow.function_id.clone());
            continue;
        }
        let f = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let entry = &f
            .blocks
            .first()
            .ok_or(OrdinaryCarrierError::Linkage)?
            .node
            .id;
        let protocol = f
            .control_protocol
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let slot_type_overrides = represented_slots(vir, flow)?;
        let mut storage_flow = flow.clone();
        for (slot, ty) in &mut storage_flow.slots {
            if let Some(storage) = slot_type_overrides.get(slot) {
                *ty = storage.clone();
            }
        }
        let mut relations = vec![emit(
            &mut b,
            &storage_flow,
            entry,
            &depths,
            &payloads,
            None,
            None,
        )?];
        for transfer in &flow.transfers {
            // The complete W04 program was reconstructed above. Additionally
            // require the original source anchor, endpoints and result identity.
            if !protocol.anchors.iter().any(|a| {
                a.source_node_id == transfer.source_node_id
                    && a.entry_node_id == transfer.entry_node_id
                    && a.exit_node_id == transfer.exit_node_id
                    && a.result.as_ref() == Some(&transfer.value)
            }) {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let memory = memory_functions
                .iter()
                .find(|f| f.source.function_id == flow.function_id)
                .and_then(|f| f.memory_bindings.iter().find(|m| &m.transfer == transfer));
            relations.push(emit(
                &mut b,
                &storage_flow,
                entry,
                &depths,
                &payloads,
                Some(transfer),
                memory,
            )?);
        }
        functions.push(OrdinaryControlSlotFunction {
            source: flow.clone(),
            relations,
            slot_type_overrides,
        });
    }
    let certificate = b.finish()?;
    let p = OrdinaryControlSlotProgram {
        schema: "mpk.csharp.ordinary_control_slots.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        control_vc_sha256: control.hash(),
        functions,
        pending_source_function_ids,
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_control_slots(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlSlotProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_control_slots(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;

    #[test]
    fn control_nullable_some_storage_preserves_presence_payload_and_padding() {
        let mut b = Builder::new().unwrap();
        let mut definitions = vec![];
        for child in [0u32, 5, 9] {
            let depth = 1 + child.max(5);
            let slot = b.var(1).unwrap();
            let value = b.var(0).unwrap();
            let body = transfer_equal(&mut b, depth, slot, value, Some(child), None).unwrap();
            let name = format!("NullableStorageTest.D{child}");
            define(&mut b, &name, &[depth, child], 0, body).unwrap();
            definitions.push((child, depth, name));
        }
        let bytes = b.finish().unwrap();
        if let Some(path) = std::env::var_os("MPK_W09_CONTROL_SOME_OUT") {
            std::fs::write(
                path,
                format!(
                    "{}\n",
                    bytes.iter().map(|v| format!("{v:02x}")).collect::<String>()
                ),
            )
            .unwrap();
        }
        let c = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        for (child, depth, name) in definitions {
            for mode in 0..3 {
                let bits = (0..1usize << child)
                    .map(|i| match mode {
                        0 => false,
                        1 => i == (1 << child) - 1,
                        _ => i % 3 == 0,
                    })
                    .collect::<Vec<_>>();
                let raw = if child == 0 {
                    V::Bit(bits[0])
                } else {
                    V::Cube(bits.clone())
                };
                let mut some = vec![false; 1 << depth];
                some[0] = true;
                for (i, &bit) in bits.iter().enumerate() {
                    some[1 | (i << (depth - child))] = bit;
                }
                assert!(observed(run(
                    &c,
                    &name,
                    vec![V::Cube(some.clone()), raw.clone()]
                )));
                // Changing any physical tag, payload or padding bit rejects.
                for at in 0..some.len() {
                    some[at] ^= true;
                    assert!(
                        !observed(run(&c, &name, vec![V::Cube(some.clone()), raw.clone()])),
                        "D{child} mode {mode} changed physical bit {at}"
                    );
                    some[at] ^= true;
                }
            }
        }
    }
}
