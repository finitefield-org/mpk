//! Primitive source conditions on original W04 operands and slot snapshots.
//! Local definitions do not establish execution or native/source equivalence.
use super::*;
use crate::csharp_practical_vir_model::{ControlVcProgram, PatternStepObservation, PatternStepVc};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternSourceDefinition {
    pub source: PatternStepObservation,
    pub source_step: PatternStepVc,
    pub source_constant: Option<String>,
    pub source_name: String,
    pub semantic_rule: String,
    pub definition: Option<String>,
    pub pending_definition_reasons: Vec<String>,
    pub native_source_equivalence_proof_pending: bool,
}

fn equal(b: &mut Builder, depth: u32, left: u32, right: u32) -> R<u32> {
    let equality = construction_data::physical_equal(b, depth)?;
    call(b, &equality, vec![left, right])
}

fn source_value(
    vir: &ValidatedPracticalVir,
    step: &PatternStepVc,
    constant: &str,
) -> R<MonomorphicValue> {
    let result = step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?;
    if constant == "null" && result.type_id == "mpk.csharp.value.unit.v1" {
        return Ok(MonomorphicValue::Unit {
            type_id: result.type_id.clone(),
        });
    }
    let value =
        crate::csharp_practical_vir_model::data_emission::source_literal(&result.type_id, constant)
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
    if let MonomorphicValue::Signed { value: carrier, .. }
    | MonomorphicValue::Unsigned { value: carrier, .. } = &value
    {
        let underlying = if result.type_id == "mpk.csharp.value.day_of_week.v1" {
            Some("i32".into())
        } else {
            vir.construction_context()
                .1
                .source_types
                .get(&result.type_id)
                .filter(|s| s.kind == SourceKind::Enum)
                .and_then(|s| s.enum_underlying.clone())
        };
        if let Some(underlying) = underlying {
            return Ok(MonomorphicValue::Enum {
                type_id: result.type_id.clone(),
                underlying,
                carrier: carrier.clone(),
            });
        }
    }
    Ok(value)
}

pub(super) fn emit(
    c: &mut Clauses<'_>,
    vir: &ValidatedPracticalVir,
    control: &ControlVcProgram,
    layouts: &OrdinaryCarrierProgram,
) -> R<Vec<OrdinaryControlPatternSourceDefinition>> {
    let mut literal_values = BTreeMap::new();
    let mut literal_names = BTreeMap::new();
    for observation in control.pattern_observations() {
        let pattern = control
            .patterns()
            .iter()
            .find(|p| p.id == observation.pattern_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let step = pattern
            .steps
            .iter()
            .find(|s| s.source_node_id == observation.source_node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if step.operation != "constant" {
            continue;
        }
        let flow = control
            .functions()
            .iter()
            .find(|f| f.function_id == observation.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let op = flow
            .source_graph
            .as_ref()
            .and_then(|g| step.source_ordinal.and_then(|i| g.operations.get(i)))
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let value = source_value(
            vir,
            step,
            op.constant
                .as_deref()
                .ok_or(OrdinaryCarrierError::Linkage)?,
        )?;
        let definition = name("PatternSourceLiteral", &value);
        literal_values.insert(definition.clone(), value);
        literal_names.insert(observation.sequent_id.clone(), definition);
    }
    if !literal_values.is_empty() {
        let builder = std::mem::replace(&mut c.b, Builder::new()?);
        (c.b, _) = literals::emit_named_values_scoped(
            vir,
            layouts,
            builder,
            literal_values,
            "PatternSourceLiteralPart",
        )?;
    }
    let payloads = crate::csharp_practical_vir_model::control_vc::option_payload_types(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut definitions = vec![];
    for observation in control.pattern_observations() {
        let pattern = control
            .patterns()
            .iter()
            .find(|p| p.id == observation.pattern_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let step = pattern
            .steps
            .iter()
            .find(|s| s.source_node_id == observation.source_node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let flow = control
            .functions()
            .iter()
            .find(|f| f.function_id == observation.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let op = flow
            .source_graph
            .as_ref()
            .and_then(|g| step.source_ordinal.and_then(|i| g.operations.get(i)))
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if step.source_kind.as_ref() != Some(&op.kind)
            || step.source_traits.as_ref() != Some(&op.traits)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let seq = control
            .sequents()
            .iter()
            .find(|s| s.id == observation.sequent_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let [goal] = seq.goals.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let bindings = &goal.bindings;
        let terms = (0..bindings.len())
            .map(|i| c.b.var((bindings.len() - 1 - i) as u32))
            .collect::<R<Vec<_>>>()?;
        let depths = bindings
            .iter()
            .map(|binding| {
                c.carriers
                    .get(binding.type_id.as_str())
                    .map(|carrier| carrier.depth)
                    .ok_or(OrdinaryCarrierError::Linkage)
            })
            .collect::<R<Vec<_>>>()?;
        let source_name = format!(
            "Mpk.CSharp.Control.PatternStep.{}.{}",
            pattern.id, step.source_node_id
        );
        let mut entry = OrdinaryControlPatternSourceDefinition {
            source: observation.clone(),
            source_step: step.clone(),
            source_constant: op.constant.clone(),
            source_name: source_name.clone(),
            semantic_rule: String::new(),
            definition: None,
            pending_definition_reasons: vec![],
            native_source_equivalence_proof_pending: true,
        };
        let result = if let Some(result) = &step.result {
            let index = observation
                .original_binding_count
                .checked_sub(1)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if bindings[index].value_id != result.id || bindings[index].type_id != result.type_id {
                return Err(OrdinaryCarrierError::Linkage);
            }
            Some(index)
        } else {
            None
        };
        let body = match step.operation.as_str() {
            "constant" => {
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                let expected = c.b.constant(
                    literal_names
                        .get(&observation.sequent_id)
                        .ok_or(OrdinaryCarrierError::Linkage)?,
                )?;
                entry.semantic_rule = "exact_source_literal_result".into();
                Some(equal(&mut c.b, depths[result], terms[result], expected)?)
            }
            "join_value" => {
                let [input] = observation.operands.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                if bindings[input.binding_index].type_id != bindings[result].type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                entry.semantic_rule = "selected_source_input_result_transport".into();
                Some(equal(
                    &mut c.b,
                    depths[result],
                    terms[input.binding_index],
                    terms[result],
                )?)
            }
            "load" | "store" | "pattern_bind" => {
                let slot = observation
                    .slot
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                let before_assigned = terms[slot.before_assigned_index];
                let after_assigned = terms[slot.after_assigned_index];
                let before = terms[slot.before_value_index];
                let after = terms[slot.after_value_index];
                let storage_depth = depths[slot.before_value_index];
                let payload_depth = (payloads.get(&slot.storage_type_id)
                    == Some(&bindings[result].type_id))
                .then_some(depths[result]);
                if step.operation == "load" {
                    if !observation.operands.is_empty() {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    let flags = equal(&mut c.b, 0, before_assigned, after_assigned)?;
                    let framed = equal(&mut c.b, storage_depth, before, after)?;
                    let loaded = control_slots::transfer_equal(
                        &mut c.b,
                        storage_depth,
                        before,
                        terms[result],
                        payload_depth,
                        None,
                    )?;
                    let a = call(&mut c.b, "Std.Bool.and", vec![before_assigned, flags])?;
                    let a = call(&mut c.b, "Std.Bool.and", vec![a, framed])?;
                    entry.semantic_rule = "assigned_source_slot_load_and_frame".into();
                    Some(call(&mut c.b, "Std.Bool.and", vec![a, loaded])?)
                } else {
                    let [input] = observation.operands.as_slice() else {
                        return Err(OrdinaryCarrierError::Linkage);
                    };
                    if bindings[input.binding_index].type_id != bindings[result].type_id {
                        entry
                            .pending_definition_reasons
                            .push("source_payload_extraction".into());
                        None
                    } else {
                        let input_result = equal(
                            &mut c.b,
                            depths[result],
                            terms[input.binding_index],
                            terms[result],
                        )?;
                        let stored = control_slots::transfer_equal(
                            &mut c.b,
                            storage_depth,
                            after,
                            terms[result],
                            payload_depth,
                            None,
                        )?;
                        let stored = call(&mut c.b, "Std.Bool.and", vec![after_assigned, stored])?;
                        entry.semantic_rule = "source_input_result_and_assigned_slot_store".into();
                        Some(call(&mut c.b, "Std.Bool.and", vec![input_result, stored])?)
                    }
                }
            }
            _ => {
                entry
                    .pending_definition_reasons
                    .push(format!("source_operation:{}", step.operation));
                None
            }
        };
        if let Some(body) = body {
            let definition = name("PatternSource", &(vir.hash(), step, observation, op));
            define(&mut c.b, &definition, &depths, 0, body)?;
            let ty = signature(
                &bindings
                    .iter()
                    .map(|b| b.type_id.clone())
                    .collect::<Vec<_>>(),
                SOURCE_BOOL,
            );
            if c.constants
                .insert(source_name, (ty, definition.clone()))
                .is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            entry.definition = Some(definition);
        }
        definitions.push(entry);
    }
    Ok(definitions)
}
