//! Exact W04 guards and transport from source exit to edge-specific target slots.
//! Node-entry merging and execution produce separate, scoped obligations.
use super::*;
use crate::csharp_practical_vir_model::data_vc::DataDefinitionFamily;
use crate::csharp_practical_vir_model::{ControlBinding, ControlFlowEdge, ControlFunctionVc};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeJoin {
    /// Guard bindings, then source-exit/edge-specific target slot pairs.
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
    /// A missing semantic definition leaves the whole guard and join pending.
    pub pending_constant_names: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeFunction {
    pub source: ControlFunctionVc,
    pub edges: Vec<OrdinaryControlEdgeDefinition>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OrdinaryControlEdgeProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    control_vc_sha256: String,
    integer_definitions: Vec<OrdinaryIntegerDataDefinition>,
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
pub fn generate_csharp_practical_ordinary_control_edges(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryControlEdgeProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let control = crate::csharp_practical_vir_model::generate_control_vcs(vir, &data)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut requested = BTreeSet::new();
    for flow in control.functions() {
        for edge in &flow.edges {
            constants(&edge.guard.term, &mut requested)?;
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
    let mut functions = vec![];
    for flow in control.functions() {
        let mut edges = vec![];
        for edge in &flow.edges {
            let mut needed = BTreeSet::new();
            constants(&edge.guard.term, &mut needed)?;
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
            edges.push(OrdinaryControlEdgeDefinition {
                source: edge.clone(),
                guard_definition,
                join,
                pending_constant_names,
            });
        }
        functions.push(OrdinaryControlEdgeFunction {
            source: flow.clone(),
            edges,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryControlEdgeProgram {
        schema: "mpk.csharp.ordinary_control_edges.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        control_vc_sha256: control.hash(),
        integer_definitions,
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
