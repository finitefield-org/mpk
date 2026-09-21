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
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlSlotFunction {
    pub source: ControlFunctionVc,
    pub relations: Vec<OrdinaryControlSlotRelation>,
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
fn emit(
    b: &mut Builder,
    flow: &ControlFunctionVc,
    entry: &str,
    depths: &BTreeMap<&str, u32>,
    transfer: Option<&ControlSlotTransfer>,
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
        if !flow
            .slots
            .iter()
            .any(|(s, ty)| s == slot && ty == &value.type_id)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        args.push(OrdinaryControlSlotArgument {
            role: "ssa".into(),
            slot_id: (*slot).into(),
            node_id: transfer.map_or(entry, |t| t.exit_node_id.as_str()).into(),
            value_id: Some(value.id.clone()),
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
            if slot == &t.slot && matches!(t.kind.as_str(), "store" | "pattern_bind") {
                let same = equal(b, depth, after, ssa)?;
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
                    let same = equal(b, depth, before, ssa)?;
                    let loaded = and(b, before_assigned, same)?;
                    framed = and(b, framed, loaded)?;
                }
                framed
            }
        } else if let Some(j) = values.iter().position(|(s, _)| s == slot) {
            let same = equal(b, depth, terms[2 * i + 1], terms[2 * flow.slots.len() + j])?;
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
    })
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
    let mut b = Builder::new()?;
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
        let mut relations = vec![emit(&mut b, flow, entry, &depths, None)?];
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
            relations.push(emit(&mut b, flow, entry, &depths, Some(transfer))?);
        }
        functions.push(OrdinaryControlSlotFunction {
            source: flow.clone(),
            relations,
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
