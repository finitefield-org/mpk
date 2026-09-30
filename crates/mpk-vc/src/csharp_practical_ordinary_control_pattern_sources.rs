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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub native_primitive_definitions: Vec<String>,
    pub native_source_equivalence_proof_pending: bool,
}

fn equal(b: &mut Builder, depth: u32, left: u32, right: u32) -> R<u32> {
    let equality = construction_data::physical_equal(b, depth)?;
    call(b, &equality, vec![left, right])
}

fn exact_pattern_type(step: &PatternStepVc, type_id: &str) -> bool {
    let key = format!("{}:{type_id}13:not_annotated0:", type_id.len());
    match (step.source_kind.as_deref(), step.source_traits.as_deref()) {
        (Some("DeclarationPattern"), Some(traits)) => traits == format!("type|{key}"),
        (Some("RecursivePattern"), Some(traits)) => traits == key,
        _ => false,
    }
}

fn primitive<'a>(
    native: &'a OrdinaryControlEdgeProgram,
    observation: &PatternStepObservation,
    step: &PatternStepVc,
    operation_id: &str,
) -> R<&'a OrdinaryControlNativeOperation> {
    let flow = native
        .functions()
        .iter()
        .find(|f| f.source.function_id == observation.function_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut matches = flow.native_operations.iter().filter(|o| {
        o.invocation.operation_id == operation_id
            && o.source_anchors.iter().any(|a| {
                a.source_node_id == step.source_node_id
                    && a.source_ordinal == step.source_ordinal
                    && a.entry_node_id == step.entry_node_id
                    && a.exit_node_id == step.exit_node_id
                    && a.result == step.result
                    && a.artifact_node_ids.contains(&o.source.node_id)
            })
    });
    let operation = matches.next().ok_or(OrdinaryCarrierError::Linkage)?;
    if matches.next().is_some() || !operation.pending_constant_names.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(operation)
}

fn normal_relation(
    c: &mut Clauses<'_>,
    operation: &OrdinaryControlNativeOperation,
    inputs: &[TypedValueRef],
    result: &TypedValueRef,
    terms: Vec<u32>,
    entry: &mut OrdinaryControlPatternSourceDefinition,
) -> R<u32> {
    let subjects = inputs
        .iter()
        .chain(std::iter::once(result))
        .cloned()
        .collect::<Vec<_>>();
    if operation.invocation.operands != inputs
        || operation.invocation.result != *result
        || operation.source.subjects != subjects
        || subjects.len() != terms.len()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let definition = operation
        .predicates
        .get("normal_execution")
        .ok_or(OrdinaryCarrierError::Linkage)?;
    entry.native_primitive_definitions.push(definition.clone());
    // Apply the guard AND relation to free source values. No reached native
    // execution, computed native result, or successful guard is assumed.
    call(&mut c.b, definition, terms)
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
    native: &OrdinaryControlEdgeProgram,
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
        if step.operation != "constant" && step.operation != "convert" {
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
        let value = if step.operation == "convert" {
            let result = step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?;
            if op.constant.as_deref() != Some("null") {
                continue;
            }
            MonomorphicValue::Option {
                type_id: result.type_id.clone(),
                arm: OptionArm::None,
                value: None,
            }
        } else {
            source_value(
                vir,
                step,
                op.constant
                    .as_deref()
                    .ok_or(OrdinaryCarrierError::Linkage)?,
            )?
        };
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
            native_primitive_definitions: vec![],
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
        let inputs = observation
            .operands
            .iter()
            .map(|o| o.native_value.clone())
            .collect::<Vec<_>>();
        let mut input_terms = observation
            .operands
            .iter()
            .map(|o| terms[o.binding_index])
            .collect::<Vec<_>>();
        if let Some(result) = result {
            input_terms.push(terms[result]);
        }
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
                        if step.operation != "pattern_bind"
                            || payloads.get(&inputs[0].type_id) != Some(&bindings[result].type_id)
                            || !exact_pattern_type(step, &bindings[result].type_id)
                        {
                            entry
                                .pending_definition_reasons
                                .push("source_payload_extraction".into());
                            None
                        } else {
                            let operation = primitive(
                                native,
                                observation,
                                step,
                                &format!("{}.value", inputs[0].type_id),
                            )?;
                            let extracted = normal_relation(
                                c,
                                operation,
                                &inputs,
                                step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                                input_terms,
                                &mut entry,
                            )?;
                            let stored = control_slots::transfer_equal(
                                &mut c.b,
                                storage_depth,
                                after,
                                terms[result],
                                payload_depth,
                                None,
                            )?;
                            let stored =
                                call(&mut c.b, "Std.Bool.and", vec![after_assigned, stored])?;
                            entry.semantic_rule =
                                "successful_source_payload_extraction_and_assigned_slot_store"
                                    .into();
                            Some(call(&mut c.b, "Std.Bool.and", vec![extracted, stored])?)
                        }
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
            "binary" | "pattern_equal" | "pattern_relational" => {
                let [left, right] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                if left.type_id != right.type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let operator = if step.operation == "pattern_equal" {
                    "Equals"
                } else {
                    op.traits
                        .split('|')
                        .next()
                        .ok_or(OrdinaryCarrierError::Linkage)?
                };
                let operation_id = if operator == "Equals" {
                    format!("structural.equal.{}", left.type_id)
                } else {
                    let token = left
                        .type_id
                        .strip_prefix("mpk.csharp.value.")
                        .and_then(|s| s.strip_suffix(".v1"))
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let operation = match operator {
                        "GreaterThan" => "greater",
                        "GreaterThanOrEqual" => "greater_equal",
                        "LessThan" => "less",
                        "LessThanOrEqual" => "less_equal",
                        "Divide" => "divide",
                        "Add" => "add",
                        "Subtract" => "subtract",
                        "Multiply" => "multiply",
                        _ => return Err(OrdinaryCarrierError::Linkage),
                    };
                    let mode = if step.operation == "binary"
                        && op.traits.split('|').nth(1) == Some("True")
                    {
                        "checked"
                    } else {
                        "unchecked"
                    };
                    format!("integer.{token}.{operation}.{mode}")
                };
                let operation = primitive(native, observation, step, &operation_id)?;
                entry.semantic_rule = "source_typed_primitive_guard_and_result".into();
                Some(normal_relation(
                    c,
                    operation,
                    &inputs,
                    step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                    input_terms,
                    &mut entry,
                )?)
            }
            "unary_update" => {
                let [input] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let token = input
                    .type_id
                    .strip_prefix("mpk.csharp.value.")
                    .and_then(|s| s.strip_suffix(".v1"))
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let operator = match op.kind.as_str() {
                    "Increment" => "add",
                    "Decrement" => "subtract",
                    _ => return Err(OrdinaryCarrierError::Linkage),
                };
                let mode = if op.traits.split('|').nth(1) == Some("True") {
                    "checked"
                } else {
                    "unchecked"
                };
                let operation = primitive(
                    native,
                    observation,
                    step,
                    &format!("integer.{token}.{operator}.{mode}"),
                )?;
                let [left, one] = operation.invocation.operands.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                if left != input || one.type_id != input.type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let expected = if matches!(token, "u32" | "u64") {
                    MonomorphicValue::Unsigned {
                        type_id: input.type_id.clone(),
                        value: "1".into(),
                    }
                } else {
                    MonomorphicValue::Signed {
                        type_id: input.type_id.clone(),
                        value: "1".into(),
                    }
                };
                let literal = native
                    .functions()
                    .iter()
                    .find(|f| f.source.function_id == observation.function_id)
                    .and_then(|f| {
                        f.native_literals
                            .iter()
                            .find(|l| l.source.result == *one && l.source.value == expected)
                    })
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let one_term = c.b.constant(&literal.literal_definition)?;
                entry
                    .native_primitive_definitions
                    .push(literal.literal_definition.clone());
                entry.semantic_rule = "source_slot_update_primitive_with_literal_one".into();
                Some(normal_relation(
                    c,
                    operation,
                    &operation.invocation.operands,
                    step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                    vec![
                        input_terms[0],
                        one_term,
                        *input_terms.last().ok_or(OrdinaryCarrierError::Linkage)?,
                    ],
                    &mut entry,
                )?)
            }
            "pattern_type" => {
                let [input] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                let target = payloads.get(&input.type_id).unwrap_or(&input.type_id);
                if !exact_pattern_type(step, target) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                if payloads.contains_key(&input.type_id) {
                    let operation = primitive(
                        native,
                        observation,
                        step,
                        &format!("{}.has_value", input.type_id),
                    )?;
                    entry.semantic_rule = "source_nullable_type_presence".into();
                    Some(normal_relation(
                        c,
                        operation,
                        &inputs,
                        step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                        input_terms,
                        &mut entry,
                    )?)
                } else {
                    let literal = native
                        .functions()
                        .iter()
                        .find(|f| f.source.function_id == observation.function_id)
                        .and_then(|f| {
                            f.native_literals.iter().find(|l| {
                                Some(&l.source.result) == step.result.as_ref()
                                    && l.source.value
                                        == MonomorphicValue::Bool {
                                            type_id: SOURCE_BOOL.into(),
                                            value: true,
                                        }
                            })
                        })
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let present = c.b.constant(&literal.literal_definition)?;
                    entry.semantic_rule = "source_nonnullable_type_presence".into();
                    Some(equal(&mut c.b, depths[result], terms[result], present)?)
                }
            }
            "member" => {
                let [input] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let roots = vir.construction_context().1;
                let member = roots
                    .source_types
                    .get(&input.type_id)
                    .and_then(|s| {
                        s.members
                            .iter()
                            .find(|m| op.symbol == format!("{}.{}", input.type_id, m.name))
                    })
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let signature = crate::csharp_practical_vir_model::source_field_operation(
                    roots,
                    vir.data_closed(),
                    &member.id,
                )
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
                let operation = primitive(native, observation, step, &signature.id)?;
                entry.semantic_rule = "source_declared_stored_member_read".into();
                Some(normal_relation(
                    c,
                    operation,
                    &inputs,
                    step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                    input_terms,
                    &mut entry,
                )?)
            }
            "element" => {
                let [receiver, _] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let operation = primitive(
                    native,
                    observation,
                    step,
                    &format!("{}.read", receiver.type_id),
                )?;
                entry.semantic_rule = "source_element_range_guard_and_result".into();
                Some(normal_relation(
                    c,
                    operation,
                    &inputs,
                    step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                    input_terms,
                    &mut entry,
                )?)
            }
            "pattern_member" if op.symbol == "System.Runtime|string.Length" => {
                let [input] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                let length = primitive(native, observation, step, "string.length")?;
                let [receiver] = length.invocation.operands.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                if payloads.get(&receiver.type_id) != Some(&input.type_id) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let some = primitive(
                    native,
                    observation,
                    step,
                    &format!("{}.some", receiver.type_id),
                )?;
                let input_index = observation.operands[0].binding_index;
                let constructor = control_slots::some_storage(&mut c.b, depths[input_index])?;
                let option = call(&mut c.b, &constructor, vec![terms[input_index]])?;
                let wrapped = normal_relation(
                    c,
                    some,
                    &inputs,
                    receiver,
                    vec![terms[input_index], option],
                    &mut entry,
                )?;
                let read = normal_relation(
                    c,
                    length,
                    &length.invocation.operands,
                    step.result.as_ref().ok_or(OrdinaryCarrierError::Linkage)?,
                    vec![option, terms[result]],
                    &mut entry,
                )?;
                entry.semantic_rule = "source_string_length_through_canonical_some".into();
                Some(call(&mut c.b, "Std.Bool.and", vec![wrapped, read])?)
            }
            "convert" if op.constant.as_deref() == Some("null") => {
                let [input] = inputs.as_slice() else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let result = result.ok_or(OrdinaryCarrierError::Linkage)?;
                if input.type_id != "mpk.csharp.value.unit.v1"
                    || !payloads.contains_key(&bindings[result].type_id)
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let unit = c.b.constant("Std.Bool.false")?;
                let unit_input = equal(&mut c.b, 0, input_terms[0], unit)?;
                let none = c.b.constant(
                    literal_names
                        .get(&observation.sequent_id)
                        .ok_or(OrdinaryCarrierError::Linkage)?,
                )?;
                let converted = equal(&mut c.b, depths[result], terms[result], none)?;
                entry.semantic_rule = "source_null_unit_to_complete_nullable_none".into();
                Some(call(&mut c.b, "Std.Bool.and", vec![unit_input, converted])?)
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
