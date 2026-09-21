//! Local exceptional invocation results, using ordered W03 checks and the
//! exact W04 edge. These relations do not execute handlers or prove reachability.
use super::*;
use crate::csharp_practical_vir_model::data_vc::DataCheckVc;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlNativeException {
    pub operation_id: String,
    pub source: DataCheckVc,
    pub edge_id: String,
    /// Invocation operands followed by the exception at this edge's target.
    /// There is no normal-result argument on an exceptional execution.
    pub arguments: Vec<ControlBinding>,
    pub ownership: Option<OrdinaryConstructionOwnershipUse>,
    pub literal_definition: String,
    pub pending_constant_names: Vec<String>,
    pub guard_definition: Option<String>,
    /// Ordered failure guard AND complete exception storage equality.
    pub definition: Option<String>,
}

pub(super) fn append(
    c: &mut Clauses<'_>,
    p: &mut OrdinaryControlEdgeProgram,
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
) -> R<()> {
    // Emit all literals together so repeated payloads use the ordinary literal
    // emitter's sharing; no host evaluation defines the exceptional predicate.
    let mut literals = BTreeMap::new();
    for flow in &mut p.functions {
        for operation in &flow.native_operations {
            for check in &operation.source.checks {
                if check.check.tag != RequiredCheckTag::Exception {
                    continue;
                }
                let successor = check
                    .exceptional_successor
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let value = check
                    .exception_value
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let edges = flow
                    .edges
                    .iter()
                    .filter(|e| {
                        e.source.source_node_id == operation.source.node_id
                            && e.source.kind == "exception"
                            && e.source.check_id.as_ref() == Some(&check.check.id)
                    })
                    .collect::<Vec<_>>();
                let [edge] = edges.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                if edge.source.target_node_id.as_ref() != Some(&successor.target_id)
                    || edge.source.guard.term != check.failure_guard
                    || !operation
                        .invocation
                        .exceptional_successors
                        .contains(successor)
                    || check.check.failure_type_id.as_ref() != Some(&successor.exception_type_id)
                    || successor.check_id != check.check.id
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let literal = name("NativeExceptionLiteral", value);
                literals.insert(literal.clone(), value.clone());
                let mut arguments =
                    operation.arguments[..operation.invocation.operands.len()].to_vec();
                arguments.push(ControlBinding {
                    kind: "exception_value".into(),
                    edge_id: Some(edge.source.id.clone()),
                    node_id: successor.target_id.clone(),
                    // W05 names a closed failed-check output by its complete
                    // edge, not by the check shared by unrelated invocations.
                    value_id: format!("{}.exception", edge.source.id),
                    type_id: value.type_id().into(),
                });
                flow.native_exceptions.push(OrdinaryControlNativeException {
                    operation_id: operation.source.id.clone(),
                    source: check.clone(),
                    edge_id: edge.source.id.clone(),
                    arguments,
                    ownership: operation.ownership.clone(),
                    literal_definition: literal,
                    pending_constant_names: vec![],
                    guard_definition: None,
                    definition: None,
                });
            }
        }
    }
    if literals.is_empty() {
        return Ok(());
    }
    let builder = std::mem::replace(&mut c.b, Builder::new()?);
    (c.b, _) = literals::emit_named_values(vir, layouts, builder, literals)?;
    for flow in &mut p.functions {
        for exception in &mut flow.native_exceptions {
            let operands = &exception.arguments[..exception.arguments.len() - 1];
            if let Some(owner) = &exception.ownership {
                if c.constants.contains_key(&owner.source_failure_name) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                c.constants.insert(
                    owner.source_failure_name.clone(),
                    (
                        signature(
                            &operands
                                .iter()
                                .map(|a| a.type_id.clone())
                                .collect::<Vec<_>>(),
                            SOURCE_BOOL,
                        ),
                        owner.scoped_failure_definition.clone(),
                    ),
                );
            }
            let mut needed = BTreeSet::new();
            constants(&exception.source.failure_guard, &mut needed)?;
            exception.pending_constant_names = needed
                .into_iter()
                .filter(|n| !c.constants.contains_key(n))
                .collect();
            if exception.pending_constant_names.is_empty() {
                let subjects = operands
                    .iter()
                    .map(|a| TypedValueRef {
                        id: a.value_id.clone(),
                        type_id: a.type_id.clone(),
                    })
                    .collect::<Vec<_>>();
                let guard = name(
                    "NativeExceptionGuard",
                    &(&exception.operation_id, &exception.source),
                );
                integer_data::predicate(c, &guard, &subjects, &exception.source.failure_guard)?;
                let count = exception.arguments.len();
                let terms = (0..count)
                    .map(|i| c.b.var((count - 1 - i) as u32))
                    .collect::<R<Vec<_>>>()?;
                let failed = call(&mut c.b, &guard, terms[..count - 1].to_vec())?;
                let literal = c.b.constant(&exception.literal_definition)?;
                let depth = c
                    .carriers
                    .get(exception.arguments[count - 1].type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .depth;
                let equality = construction_data::physical_equal(&mut c.b, depth)?;
                let same = call(&mut c.b, &equality, vec![terms[count - 1], literal])?;
                let mut body = call(&mut c.b, "Std.Bool.and", vec![failed, same])?;
                let types = exception
                    .arguments
                    .iter()
                    .map(|a| a.type_id.clone())
                    .collect::<Vec<_>>();
                for ty in types.iter().rev() {
                    let ty = c.ty(ty, 0)?;
                    body = c.b.lam(ty, body)?;
                }
                let ty = c.ty(&signature(&types, SOURCE_BOOL), 0)?;
                let definition = name(
                    "NativeException",
                    &(
                        &exception.operation_id,
                        &exception.source,
                        &exception.arguments,
                    ),
                );
                c.b.define(&definition, ty, body)?;
                exception.guard_definition = Some(guard);
                exception.definition = Some(definition);
            }
            if let Some(owner) = &exception.ownership {
                c.constants.remove(&owner.source_failure_name);
            }
        }
    }
    Ok(())
}
