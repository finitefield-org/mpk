//! Lower source-order control into the existing typed VIR block vocabulary.
use super::*;
#[path = "csharp_practical_control_completions.rs"]
mod completions;
#[path = "csharp_practical_control_constructors.rs"]
mod constructors;
#[path = "csharp_practical_control_handlers.rs"]
mod handlers;
#[path = "csharp_practical_control_memory.rs"]
mod memory;
#[path = "csharp_practical_control_patterns.rs"]
mod patterns;

fn find_expr<'a, 's>(nodes: &'a [Expr<'s>], ordinal: usize) -> Option<&'a Expr<'s>> {
    nodes.iter().find_map(|n| {
        if n.ordinal == ordinal {
            Some(n)
        } else {
            find_expr(&n.children, ordinal)
        }
    })
}
fn resolve_alias(
    aliases: &BTreeMap<String, String>,
    value: &str,
) -> Result<String, DataPhaseError> {
    let mut current = value;
    let mut seen = BTreeSet::new();
    while let Some(next) = aliases.get(current) {
        if !seen.insert(current) {
            return Err(DataPhaseError::Emission);
        }
        current = next;
    }
    Ok(current.into())
}
impl Emitter<'_> {
    pub(super) fn control_body(
        &mut self,
        body: &mut Body,
        function: &LoopControlFunction,
        handler: &PreparedHandlerFunction,
        universe: &ClosedExceptionUniverse,
    ) -> Result<(), DataPhaseError> {
        let mut graph = handler_control_graph(function, handler, universe)?;
        let mut handler = handler.graph().clone();
        let constructor_receivers = constructors::prepare(self, body, &mut graph, &mut handler)?;
        completions::expand_rethrows(&mut graph, &mut handler, universe)?;
        completions::expand(&mut graph, &mut handler, universe)?;
        handlers::direct_catch_graph(&mut graph, &handler, universe)?;
        let nodes = graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n))
            .collect::<BTreeMap<_, _>>();
        let mut reachable = BTreeSet::new();
        let mut pending = vec![graph.nodes[0].id.clone()];
        while let Some(id) = pending.pop() {
            if reachable.insert(id.clone()) {
                pending.extend(
                    nodes[&id]
                        .successors
                        .iter()
                        .chain(&nodes[&id].exceptional_successors)
                        .cloned(),
                );
            }
        }
        graph.nodes.retain(|n| reachable.contains(&n.id));
        graph.loops.retain(|l| reachable.contains(&l.header));
        let parameters = body
            .variables
            .keys()
            .map(|slot| (slot.clone(), format!("entry.{}.{slot}", body.function.id)))
            .collect();
        let caught = handler
            .regions
            .iter()
            .flat_map(|r| &r.catches)
            .filter(|c| !c.local.is_empty())
            .flat_map(|c| {
                std::iter::once(&c.entry)
                    .chain(c.filter.as_ref())
                    .filter(|id| graph.nodes.iter().any(|n| n.id == ***id))
                    .map(|id| (id.clone(), (c.local.clone(), format!("{id}.exception"))))
            })
            .collect();
        let ssa = derive_control_ssa_bounded(&graph, &parameters, &caught, 8192)
            .map_err(DataPhaseError::ControlGraph)?;
        let mut aliases = BTreeMap::new();
        for (node, block) in graph.nodes.iter().zip(&ssa.blocks) {
            if node.operation == "update" {
                aliases.insert(node.result.clone(), node.inputs[2].clone());
            } else if node.operation == "store"
                || (node.operation == "pattern_bind"
                    && node
                        .source_ordinal
                        .is_some_and(|i| graph.operations[i].traits.starts_with("var|")))
            {
                aliases.insert(node.result.clone(), node.inputs[0].clone());
            } else if node.operation == "load" {
                if let Some(value) = &block.load_value_id {
                    aliases.insert(node.result.clone(), value.clone());
                }
            }
        }
        for node in &graph.nodes {
            if node.operation != "convert" {
                continue;
            }
            let ordinal = node.source_ordinal.ok_or(DataPhaseError::Emission)?;
            let op = &graph.operations[ordinal];
            let child = graph
                .operations
                .get(ordinal + 1)
                .ok_or(DataPhaseError::Emission)?;
            if let (Some(target), Some(source)) =
                (op.type_key.as_deref(), child.type_key.as_deref())
            {
                if parse_data_type_key(self.b, target)? == parse_data_type_key(self.b, source)? {
                    // The normalized foreach view retains the exact array or
                    // string identity, including an array's current owned state.
                    aliases.insert(node.result.clone(), node.inputs[0].clone());
                }
            }
        }
        let memory = memory::memory_plan(&graph, &ssa, &aliases)?;
        for (value, origin) in &memory.origins {
            if value != origin {
                aliases.insert(value.clone(), origin.clone());
            }
        }
        let mut values = body
            .variables
            .iter()
            .map(|(slot, v)| (format!("entry.{}.{slot}", body.function.id), v.clone()))
            .collect::<BTreeMap<_, _>>();
        for clause in handler.regions.iter().flat_map(|r| &r.catches) {
            for id in std::iter::once(&clause.entry).chain(clause.filter.as_ref()) {
                values.insert(format!("{id}.exception"), body.value(EXCEPTION_TYPE_ID));
            }
        }
        for n in graph.nodes.iter().filter(|n| {
            matches!(
                n.kind.as_str(),
                "handler_landing" | "handler_restore" | "handler_escape"
            )
        }) {
            values.insert(format!("{}.exception", n.id), body.value(EXCEPTION_TYPE_ID));
        }
        let return_type = body
            .function
            .result_type_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "mpk.csharp.value.unit.v1".into());
        let return_is_option = self
            .c
            .metadata
            .get(&return_type)
            .is_some_and(|m| template_name(&m.template_id) == Some("option"));
        let return_option = if return_is_option {
            return_type.clone()
        } else {
            self.c
                .metadata
                .iter()
                .find(|(_, m)| {
                    template_name(&m.template_id) == Some("option")
                        && m.argument_ids == [return_type.clone()]
                })
                .map(|(id, _)| id.clone())
                .unwrap_or_default()
        };
        for n in &graph.nodes {
            if n.kind != "evaluate" || aliases.contains_key(&n.result) {
                continue;
            }
            let ty = if n.operation == "constant"
                && n.source_ordinal.is_some_and(|i| {
                    graph.operations[i].constant.as_deref() == Some("null")
                        && graph.operations[i].type_key.is_none()
                }) {
                "mpk.csharp.value.unit.v1".into()
            } else {
                match n.operation.as_str() {
                    "true" | "condition_true" | "condition_false" | "pattern_true"
                    | "pattern_false" | "pattern_equal" | "pattern_relational" | "pattern_type"
                    | "pattern_not_null" | "pattern_length" => BOOL_TYPE_ID.to_owned(),
                    "zero" | "initializer_index" | "increment" | "length" => I32_TYPE_ID.to_owned(),
                    "less" => BOOL_TYPE_ID.to_owned(),
                    "closed_exception" | "pending_exception" | "pending_exception_default" => {
                        EXCEPTION_TYPE_ID.to_owned()
                    }
                    "pending_kind" => I32_TYPE_ID.to_owned(),
                    "pending_is_kind" | "pending_exception_is" => BOOL_TYPE_ID.to_owned(),
                    "pending_some" | "pending_none" => return_option.clone(),
                    "pending_unwrap" => return_type.clone(),
                    "construction_begin" => {
                        let ordinal = n.source_ordinal.ok_or(DataPhaseError::Source)?;
                        let plan = self
                            .source
                            .callables()
                            .iter()
                            .find(|c| c.id() == body.function.id)
                            .and_then(|c| {
                                c.initialization_plans()
                                    .iter()
                                    .find(|p| p.node_ordinal == ordinal)
                            })
                            .ok_or(DataPhaseError::Source)?;
                        object_construction_signature(
                            self.r,
                            self.c,
                            &format!("object.begin.{}", plan.type_id),
                        )
                        .map_err(|_| DataPhaseError::Emission)?
                        .normal_result_type_id
                    }
                    "construct_payload" => body.owner.clone(),
                    "constructor_write" => body.function.parameter_values[0].type_id.clone(),
                    "construction_invoke" | "construction_write" => values
                        .get(&resolve_alias(&aliases, &n.inputs[0])?)
                        .ok_or(DataPhaseError::Source)?
                        .type_id
                        .clone(),
                    "allocate" => {
                        let key = graph.operations
                            [n.source_ordinal.ok_or(DataPhaseError::Emission)?]
                        .type_key
                        .as_deref()
                        .ok_or(DataPhaseError::Emission)?;
                        let ty = parse_data_type_key(self.b, key)?;
                        let ClosedType::Instance {
                            template,
                            arguments,
                        } = ClosedType::parse(&ty).map_err(|_| DataPhaseError::Emission)?
                        else {
                            return Err(DataPhaseError::Emission);
                        };
                        if template != "bounded_sequence" {
                            return Err(DataPhaseError::Emission);
                        }
                        closed_type_id(
                            self.b,
                            &ClosedType::Instance {
                                template: "sequence_construction".into(),
                                arguments,
                            },
                        )
                        .map_err(|_| DataPhaseError::Emission)?
                    }
                    "iteration_convert" => {
                        let op =
                            &graph.operations[n.source_ordinal.ok_or(DataPhaseError::Emission)?];
                        self.source
                            .control_lowering()
                            .ok_or(DataPhaseError::Source)?["facts"]["methods"]
                            .as_array()
                            .ok_or(DataPhaseError::Source)?
                            .iter()
                            .find(|m| m["callable_id"] == function.callable_id)
                            .ok_or(DataPhaseError::Source)?["locals"]
                            .as_array()
                            .ok_or(DataPhaseError::Source)?
                            .iter()
                            .find(|l| l["id"] == op.symbol)
                            .and_then(|l| l["type_id"].as_str())
                            .ok_or(DataPhaseError::Emission)?
                            .to_owned()
                    }
                    "element" => {
                        if let Some(key) = n
                            .source_ordinal
                            .and_then(|i| graph.operations[i].type_key.as_deref())
                        {
                            let ty = parse_data_type_key(self.b, key)?;
                            closed_type_id(
                                self.b,
                                &ClosedType::parse(&ty).map_err(|_| DataPhaseError::Emission)?,
                            )
                            .map_err(|_| DataPhaseError::Emission)?
                        } else {
                            let ordinal = n.source_ordinal.ok_or(DataPhaseError::Emission)?;
                            let collection = graph
                                .operations
                                .get(ordinal + 1)
                                .and_then(|o| o.type_key.as_deref())
                                .ok_or(DataPhaseError::Emission)?;
                            let ty = parse_data_type_key(self.b, collection)?;
                            match ClosedType::parse(&ty).map_err(|_| DataPhaseError::Emission)? {
                                ClosedType::Primitive(t) if t == "string" => {
                                    "mpk.csharp.value.char.v1".into()
                                }
                                ClosedType::Instance {
                                    template,
                                    arguments,
                                } if template == "bounded_sequence" => {
                                    closed_type_id(self.b, &arguments[0])
                                        .map_err(|_| DataPhaseError::Emission)?
                                }
                                _ => return Err(DataPhaseError::Emission),
                            }
                        }
                    }
                    "pattern_bind" => {
                        let op =
                            &graph.operations[n.source_ordinal.ok_or(DataPhaseError::Emission)?];
                        let key = op
                            .traits
                            .split_once('|')
                            .map(|(_, s)| s)
                            .ok_or(DataPhaseError::Emission)?;
                        let ty = parse_data_type_key(self.b, key)?;
                        closed_type_id(
                            self.b,
                            &ClosedType::parse(&ty).map_err(|_| DataPhaseError::Emission)?,
                        )
                        .map_err(|_| DataPhaseError::Emission)?
                    }
                    "pattern_element" => {
                        let receiver = values
                            .get(&resolve_alias(&aliases, &n.inputs[0])?)
                            .ok_or(DataPhaseError::Emission)?;
                        let mut ty = receiver.type_id.clone();
                        if let Some(m) = self
                            .c
                            .metadata
                            .get(&ty)
                            .filter(|m| template_name(&m.template_id) == Some("option"))
                        {
                            ty = m.argument_ids[0].clone();
                        }
                        self.c
                            .metadata
                            .get(&ty)
                            .and_then(|m| m.argument_ids.first())
                            .cloned()
                            .ok_or(DataPhaseError::Emission)?
                    }
                    "constant"
                    | "binary"
                    | "unary"
                    | "unary_update"
                    | "convert"
                    | "member"
                    | "call"
                    | "construct"
                    | "construction_finalize"
                    | "pattern_constant"
                    | "pattern_member"
                    | "join_value"
                    | "publish"
                    | "exception_payload" => {
                        let op = n
                            .source_ordinal
                            .and_then(|i| graph.operations.get(i))
                            .ok_or(DataPhaseError::Emission)?;
                        let ty = parse_data_type_key(
                            self.b,
                            op.type_key.as_deref().ok_or(DataPhaseError::Emission)?,
                        )?;
                        closed_type_id(
                            self.b,
                            &ClosedType::parse(&ty).map_err(|_| DataPhaseError::Emission)?,
                        )
                        .map_err(|_| DataPhaseError::Emission)?
                    }
                    _ => return Err(DataPhaseError::Emission),
                }
            };
            values.insert(
                n.result.clone(),
                TypedValueRef {
                    id: format!("pending.{}", n.result),
                    type_id: ty,
                },
            );
        }
        for (node, origin) in &memory.updates {
            let key = graph
                .nodes
                .iter()
                .find(|n| n.id == *node && matches!(n.operation.as_str(), "join_value" | "publish"))
                .map(|n| n.result.as_str())
                .unwrap_or(origin);
            let ty = values
                .get(key)
                .ok_or(DataPhaseError::Emission)?
                .type_id
                .clone();
            let ty = if graph
                .nodes
                .iter()
                .any(|n| n.id == *node && n.operation == "pending_some")
            {
                self.signatures
                    .get(&format!("{ty}.freeze"))
                    .map(|s| s.normal_result_type_id.clone())
                    .unwrap_or(ty)
            } else {
                ty
            };
            let id = format!("{node}.memory.value");
            values.insert(
                id.clone(),
                TypedValueRef {
                    id: format!("pending.{id}"),
                    type_id: ty,
                },
            );
        }
        let mut pending = ssa
            .blocks
            .iter()
            .flat_map(|b| &b.phis)
            .chain(memory.blocks.values().flat_map(|b| &b.phis))
            .filter(|p| !aliases.contains_key(&p.id))
            .collect::<Vec<_>>();
        while !pending.is_empty() {
            let count = pending.len();
            let mut next = Vec::new();
            for phi in pending {
                let known = phi
                    .incoming
                    .iter()
                    .map(|(_, id)| resolve_alias(&aliases, id))
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(ty) = known
                    .iter()
                    .find_map(|id| values.get(id).map(|v| v.type_id.clone()))
                {
                    if known
                        .iter()
                        .filter_map(|id| values.get(id))
                        .any(|v| v.type_id != ty)
                    {
                        return Err(DataPhaseError::Emission);
                    }
                    values.insert(phi.id.clone(), body.value(&ty));
                } else {
                    next.push(phi);
                }
            }
            if next.len() == count {
                return Err(DataPhaseError::Emission);
            }
            pending = next;
        }
        let get = |id: &str| -> Result<TypedValueRef, DataPhaseError> {
            values
                .get(&resolve_alias(&aliases, id)?)
                .cloned()
                .ok_or(DataPhaseError::Emission)
        };
        let mut starts = BTreeMap::new();
        for (i, n) in graph.nodes.iter().enumerate() {
            if n.kind == "handler_search" {
                continue;
            }
            starts.insert(
                n.id.clone(),
                if i == 0 {
                    body.current
                } else {
                    body.block(ControlNodeTag::Jump)
                },
            );
        }
        let source_body = self
            .source
            .body(&function.callable_id)
            .ok_or(DataPhaseError::Source)?;
        let expressions = trees(source_body)?;
        let mut ends = BTreeMap::new();
        let mut emitted_nodes = BTreeMap::new();
        let mut replacements = BTreeMap::<String, String>::new();
        let mut initializers = BTreeMap::<usize, v::PracticalObjectInitialization>::new();
        for n in &graph.nodes {
            if n.kind == "handler_search" {
                continue;
            }
            let first_extra_block = body.function.blocks.len();
            body.current = starts[&n.id];
            body.ended = false;
            body.expression_values.clear();
            body.live_objects.clear();
            if body.object_constructor {
                if let Some(receiver) = constructor_receivers.get(&n.id) {
                    body.live_objects
                        .insert(body.function.parameter_values[0].id.clone(), get(receiver)?);
                } else if n.kind == "entry" {
                    let receiver = body.function.parameter_values[0].clone();
                    body.live_objects.insert(receiver.id.clone(), receiver);
                }
            }
            for (origin, state) in &memory.objects[&n.id] {
                body.live_objects.insert(get(origin)?.id, get(state)?);
            }
            body.live_constructions.clear();
            for (origin, state) in &memory.blocks[&n.id].states {
                let key = get(origin)?.id;
                let state = get(state)?;
                if self
                    .c
                    .metadata
                    .get(&state.type_id)
                    .is_some_and(|m| template_name(&m.template_id) == Some("sequence_construction"))
                {
                    body.live_constructions.insert(key.clone(), state);
                }
                body.construction_lengths
                    .insert(key, get(&memory.allocations[origin])?);
            }
            let mut operands = n
                .inputs
                .iter()
                .map(|id| {
                    let canonical = resolve_alias(&aliases, id)?;
                    if let Some(origin) = memory.origins.get(&canonical) {
                        get(memory.blocks[&n.id]
                            .states
                            .get(origin)
                            .ok_or(DataPhaseError::Emission)?)
                    } else {
                        get(id)
                    }
                })
                .collect::<Result<Vec<_>, DataPhaseError>>()?;
            if matches!(n.operation.as_str(), "element" | "length") {
                if let Ok(signature) = domain::reference_value_signature(
                    self.r,
                    self.c,
                    &format!("reference.value.{}", operands[0].type_id),
                ) {
                    operands[0] = self.invoke(body, signature, vec![operands[0].clone()])?;
                }
            }
            if n.operation == "pattern_member" {
                operands[0] = self.pattern_payload(body, operands[0].clone())?;
            }
            match n.kind.as_str() {
                "entry" | "jump" | "pattern_decision" => {}
                "handler_entry"
                | "handler_filter_entry"
                | "handler_landing"
                | "handler_restore"
                | "handler_escape" => {
                    let block = &mut body.function.blocks[body.current];
                    block.node.tag = ControlNodeTag::HandlerEntry;
                    block.handler_exception_value = Some(get(&format!("{}.exception", n.id))?);
                    if matches!(n.kind.as_str(), "handler_restore" | "handler_escape") {
                        block.handler_exception_source_id = Some(operands[0].id.clone());
                    }
                }
                "handler_finally_entry" => {
                    body.function.blocks[body.current].node.tag = ControlNodeTag::FinallyEntry;
                    body.function.blocks[body.current]
                        .construction_actions
                        .extend(body.live_constructions.keys().map(|origin| {
                            v::PracticalConstructionAction::Discard {
                                construction_id: origin.clone(),
                                actor_id: body.function.id.clone(),
                            }
                        }));
                    body.live_constructions.clear();
                }
                "handler_finally_exit" => {
                    body.function.blocks[body.current].node.tag = ControlNodeTag::FinallyExit;
                }
                "handler_completion" if n.operation == "normal" => {}
                "evaluate" if aliases.contains_key(&n.result) && n.operation != "update" => {}
                "evaluate" => {
                    let result = if n.operation == "constant"
                        && get(&n.result)?.type_id == "mpk.csharp.value.unit.v1"
                    {
                        body.literal(MonomorphicValue::Unit {
                            type_id: "mpk.csharp.value.unit.v1".into(),
                        })
                    } else {
                        match n.operation.as_str() {
                            "pattern_bind" | "pattern_equal" | "pattern_relational"
                            | "pattern_type" | "pattern_not_null" | "pattern_length"
                            | "pattern_element" | "pattern_true" | "pattern_false" => {
                                self.pattern_operation(body, n, &graph, operands.clone())?
                            }
                            "pending_kind" => body.literal(MonomorphicValue::Signed {
                                type_id: I32_TYPE_ID.into(),
                                value: n.slot.clone(),
                            }),
                            "pending_is_kind" => {
                                let right = body.literal(MonomorphicValue::Signed {
                                    type_id: I32_TYPE_ID.into(),
                                    value: n.slot.clone(),
                                });
                                self.invoke(
                                    body,
                                    scalar_operation_signature("integer.i32.equal.checked")
                                        .map_err(|_| DataPhaseError::Emission)?,
                                    vec![operands[0].clone(), right],
                                )?
                            }
                            "pending_none" => body.literal(MonomorphicValue::Option {
                                type_id: return_option.clone(),
                                arm: OptionArm::None,
                                value: None,
                            }),
                            "pending_some" => {
                                let value = self.publish(body, operands[0].clone())?;
                                if memory.updates.contains_key(&n.id) {
                                    replacements.insert(
                                        get(&format!("{}.memory.value", n.id))?.id,
                                        value.id.clone(),
                                    );
                                }
                                let value = self.coerce(body, value, &return_type)?;
                                if return_is_option {
                                    value
                                } else {
                                    let signature = self
                                        .signatures
                                        .get(&format!("{return_option}.some"))
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?;
                                    self.invoke(body, signature, vec![value])?
                                }
                            }
                            "pending_unwrap" => {
                                // A void return has no saved operand. Completing
                                // its finally yields unit, not an absent-value error.
                                if return_type == "mpk.csharp.value.unit.v1" {
                                    body.literal(MonomorphicValue::Unit {
                                        type_id: return_type.clone(),
                                    })
                                } else if return_is_option {
                                    operands[0].clone()
                                } else {
                                    let signature = self
                                        .signatures
                                        .get(&format!("{return_option}.value"))
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?;
                                    self.invoke(body, signature, operands.clone())?
                                }
                            }
                            "pending_exception" => operands[0].clone(),
                            "pending_exception_default" => self.closed_exception(
                                body,
                                "System.InvalidOperationException",
                                vec![],
                            )?,
                            "pending_exception_is" => self.invoke(
                                body,
                                ClosedOperationSignature {
                                    id: format!("mpk.csharp.value.exception.v1.is_type.{}", n.slot),
                                    tag: ClosedOperationTag::ExceptionIsType,
                                    argument_type_ids: vec![EXCEPTION_TYPE_ID.into()],
                                    normal_result_type_id: BOOL_TYPE_ID.into(),
                                    ordered_checks: vec![],
                                },
                                operands.clone(),
                            )?,
                            "exception_payload" => {
                                let expected = get(&n.result)?;
                                let member = self
                                    .r
                                    .source_types
                                    .values()
                                    .find_map(|s| {
                                        s.members
                                            .iter()
                                            .find(|m| format!("{}.{}", s.id, m.name) == n.slot)
                                    })
                                    .ok_or(DataPhaseError::Emission)?;
                                self.invoke(
                                    body,
                                    ClosedOperationSignature {
                                        id: format!(
                                            "mpk.csharp.value.exception.v1.payload.{}",
                                            member.id
                                        ),
                                        tag: ClosedOperationTag::ExceptionPayload,
                                        argument_type_ids: vec![EXCEPTION_TYPE_ID.into()],
                                        normal_result_type_id: expected.type_id,
                                        ordered_checks: vec![],
                                    },
                                    operands.clone(),
                                )?
                            }
                            "closed_exception" => {
                                self.closed_exception(body, &n.slot, operands.clone())?
                            }
                            "join_value" | "publish" => {
                                let value = self.publish(body, operands[0].clone())?;
                                let expected = get(&n.result)?;
                                let result = if value.type_id == "mpk.csharp.value.unit.v1"
                                    && value.type_id != expected.type_id
                                {
                                    if !self.c.metadata.get(&expected.type_id).is_some_and(|m| {
                                        template_name(&m.template_id) == Some("option")
                                    }) {
                                        return Err(DataPhaseError::Emission);
                                    }
                                    body.literal(MonomorphicValue::Option {
                                        type_id: expected.type_id,
                                        arm: OptionArm::None,
                                        value: None,
                                    })
                                } else {
                                    self.coerce(body, value, &expected.type_id)?
                                };
                                if memory.updates.contains_key(&n.id) {
                                    replacements.insert(
                                        get(&format!("{}.memory.value", n.id))?.id,
                                        result.id.clone(),
                                    );
                                }
                                result
                            }
                            "allocate" => {
                                let expected = get(&n.result)?;
                                let expression = find_expr(
                                    &expressions,
                                    n.source_ordinal.ok_or(DataPhaseError::Emission)?,
                                )
                                .ok_or(DataPhaseError::Emission)?;
                                let payload = &self.c.metadata[&expected.type_id].argument_ids[0];
                                let default = body.literal(MonomorphicValue::Bool {
                                    type_id: BOOL_TYPE_ID.into(),
                                    value: expression.children.len() == 1
                                        && domain_default(self.b, self.r, self.c, payload).is_ok(),
                                });
                                let signature = self
                                    .signatures
                                    .get(&format!("{}.allocate", expected.type_id))
                                    .cloned()
                                    .ok_or(DataPhaseError::Emission)?;
                                self.invoke(body, signature, vec![operands[0].clone(), default])?
                            }
                            "update" => {
                                let origin = get(&memory.updates[&n.id])?.id;
                                let callable = self
                                    .source
                                    .callables()
                                    .iter()
                                    .find(|c| c.id() == function.callable_id)
                                    .ok_or(DataPhaseError::Source)?;
                                let ordinal = n.source_ordinal.ok_or(DataPhaseError::Emission)?;
                                let mode = if graph.operations[ordinal].kind == "ArrayCreation" {
                                    "fill"
                                } else if matches!(
                                    graph.operations[ordinal].kind.as_str(),
                                    "Increment" | "Decrement" | "CompoundAssignment"
                                ) {
                                    "rewrite"
                                } else {
                                    callable
                                        .data_steps()
                                        .iter()
                                        .find(|s| s.node_ordinal == ordinal && s.family == "array")
                                        .map(|s| s.operation.as_str())
                                        .ok_or(DataPhaseError::Emission)?
                                };
                                if mode == "fill_or_rewrite" {
                                    let complete = self.invoke(
                                        body,
                                        sequence_construction_complete_signature(
                                            self.c,
                                            &format!(
                                                "construction.complete.{}",
                                                operands[0].type_id
                                            ),
                                        )
                                        .map_err(|_| DataPhaseError::Emission)?,
                                        vec![operands[0].clone()],
                                    )?;
                                    let rewrite = self.signatures
                                        [&format!("{}.rewrite", operands[0].type_id)]
                                        .clone();
                                    let fill = self.signatures
                                        [&format!("{}.fill", operands[0].type_id)]
                                        .clone();
                                    self.branch_with_value(
                                        body,
                                        complete,
                                        None,
                                        None,
                                        None,
                                        None,
                                        true,
                                        Some((rewrite, operands.clone())),
                                        None,
                                        Some((fill, operands.clone())),
                                    )?;
                                } else {
                                    let signature = self
                                        .signatures
                                        .get(&format!("{}.{mode}", operands[0].type_id))
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?;
                                    self.invoke(body, signature, operands.clone())?;
                                }
                                let value = body
                                    .live_constructions
                                    .get(&origin)
                                    .ok_or(DataPhaseError::Emission)?;
                                replacements.insert(
                                    get(&format!("{}.memory.value", n.id))?.id,
                                    value.id.clone(),
                                );
                                operands[2].clone()
                            }
                            "true" | "condition_true" | "condition_false" => {
                                body.literal(MonomorphicValue::Bool {
                                    type_id: BOOL_TYPE_ID.into(),
                                    value: n.operation != "condition_false",
                                })
                            }
                            "zero" | "initializer_index" => {
                                body.literal(MonomorphicValue::Signed {
                                    type_id: I32_TYPE_ID.into(),
                                    value: if n.operation == "zero" {
                                        "0".into()
                                    } else {
                                        n.slot.clone()
                                    },
                                })
                            }
                            "less" => self.invoke(
                                body,
                                scalar_operation_signature("integer.i32.less.checked")
                                    .map_err(|_| DataPhaseError::Emission)?,
                                operands.clone(),
                            )?,
                            "increment" => {
                                let one = body.literal(MonomorphicValue::Signed {
                                    type_id: I32_TYPE_ID.into(),
                                    value: "1".into(),
                                });
                                self.invoke(
                                    body,
                                    scalar_operation_signature("integer.i32.add.checked")
                                        .map_err(|_| DataPhaseError::Emission)?,
                                    vec![operands[0].clone(), one],
                                )?
                            }
                            "unary_update" => {
                                let source = &graph.operations
                                    [n.source_ordinal.ok_or(DataPhaseError::Emission)?];
                                let target = operands[0]
                                    .type_id
                                    .strip_prefix("mpk.csharp.value.")
                                    .and_then(|s| s.strip_suffix(".v1"))
                                    .ok_or(DataPhaseError::Emission)?;
                                let promoted =
                                    if matches!(target, "i8" | "u8" | "i16" | "u16" | "char") {
                                        "i32"
                                    } else {
                                        target
                                    };
                                let mode = if source.traits.split('|').nth(1) == Some("True") {
                                    "checked"
                                } else {
                                    "unchecked"
                                };
                                let mut left = operands[0].clone();
                                if target != promoted {
                                    left = self.invoke(
                                        body,
                                        scalar_operation_signature(&format!(
                                            "integer.convert.{target}.{promoted}.{mode}"
                                        ))
                                        .map_err(|_| DataPhaseError::Emission)?,
                                        vec![left],
                                    )?;
                                }
                                let one = if matches!(promoted, "u32" | "u64") {
                                    body.literal(MonomorphicValue::Unsigned {
                                        type_id: format!("mpk.csharp.value.{promoted}.v1"),
                                        value: "1".into(),
                                    })
                                } else {
                                    body.literal(MonomorphicValue::Signed {
                                        type_id: format!("mpk.csharp.value.{promoted}.v1"),
                                        value: "1".into(),
                                    })
                                };
                                let name = if source.kind == "Increment" {
                                    "add"
                                } else {
                                    "subtract"
                                };
                                let result = self.invoke(
                                    body,
                                    scalar_operation_signature(&format!(
                                        "integer.{promoted}.{name}.{mode}"
                                    ))
                                    .map_err(|_| DataPhaseError::Emission)?,
                                    vec![left, one],
                                )?;
                                if target == promoted {
                                    result
                                } else {
                                    self.invoke(
                                        body,
                                        scalar_operation_signature(&format!(
                                            "integer.convert.{promoted}.{target}.{mode}"
                                        ))
                                        .map_err(|_| DataPhaseError::Emission)?,
                                        vec![result],
                                    )?
                                }
                            }
                            "length" => {
                                let receiver = operands[0].clone();
                                if self.c.metadata.get(&receiver.type_id).is_some_and(|m| {
                                    template_name(&m.template_id) == Some("sequence_construction")
                                }) {
                                    let origin = body
                                        .live_constructions
                                        .iter()
                                        .find(|(_, value)| value.id == receiver.id)
                                        .map(|(origin, _)| origin)
                                        .ok_or(DataPhaseError::Emission)?;
                                    body.construction_lengths
                                        .get(origin)
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?
                                } else if receiver.type_id == STRING_TYPE_ID {
                                    self.string_invoke(body, "string.length", vec![receiver])?
                                } else {
                                    let signature = self
                                        .signatures
                                        .get(&format!("{}.length", receiver.type_id))
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?;
                                    let length = self.invoke(body, signature, vec![receiver])?;
                                    self.invoke(
                                        body,
                                        scalar_operation_signature(
                                            "integer.convert.u32.i32.unchecked",
                                        )
                                        .map_err(|_| DataPhaseError::Emission)?,
                                        vec![length],
                                    )?
                                }
                            }
                            "element" => {
                                if operands[0].type_id == STRING_TYPE_ID {
                                    self.string_invoke(body, "string.index", operands.clone())?
                                } else {
                                    let signature = self
                                        .signatures
                                        .get(&format!("{}.read", operands[0].type_id))
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?;
                                    self.invoke(body, signature, operands.clone())?
                                }
                            }
                            "iteration_convert" => {
                                let expected = get(&n.result)?;
                                if operands[0].type_id == expected.type_id {
                                    operands[0].clone()
                                } else {
                                    let token = |id: &str| {
                                        id.strip_prefix("mpk.csharp.value.")
                                            .and_then(|s| s.strip_suffix(".v1"))
                                            .map(str::to_owned)
                                            .ok_or(DataPhaseError::Emission)
                                    };
                                    let signature = scalar_operation_signature(&format!(
                                        "integer.convert.{}.{}.checked",
                                        token(&operands[0].type_id)?,
                                        token(&expected.type_id)?
                                    ))
                                    .map_err(|_| DataPhaseError::Emission)?;
                                    self.invoke(body, signature, operands.clone())?
                                }
                            }
                            "constructor_write" => {
                                let name = n
                                    .slot
                                    .strip_prefix(&format!("{}.", body.owner))
                                    .ok_or(DataPhaseError::Source)?;
                                let member = self.r.source_types[&body.owner]
                                    .members
                                    .iter()
                                    .find(|m| m.name == name)
                                    .ok_or(DataPhaseError::Source)?;
                                let signature = object_construction_signature(
                                    self.r,
                                    self.c,
                                    &format!("object.write.{}", member.id),
                                )
                                .map_err(|_| DataPhaseError::Emission)?;
                                let value = self.publish(body, operands[1].clone())?;
                                operands[1] =
                                    self.coerce(body, value, &signature.argument_type_ids[1])?;
                                self.invoke(body, signature, operands.clone())?
                            }
                            "construct_payload" => {
                                let signature =
                                    source_value_constructor_operation(self.r, self.c, &body.owner)
                                        .map_err(|_| DataPhaseError::Emission)?;
                                self.invoke(body, signature, operands.clone())?
                            }
                            "construction_begin"
                            | "construction_invoke"
                            | "construction_write"
                            | "construction_finalize" => {
                                let ordinal = if n.operation == "construction_write" {
                                    n.slot.parse().map_err(|_| DataPhaseError::Source)?
                                } else {
                                    n.source_ordinal.ok_or(DataPhaseError::Source)?
                                };
                                let plan = self
                                    .source
                                    .callables()
                                    .iter()
                                    .find(|c| c.id() == body.function.id)
                                    .and_then(|c| {
                                        c.initialization_plans()
                                            .iter()
                                            .find(|p| p.node_ordinal == ordinal)
                                    })
                                    .ok_or(DataPhaseError::Source)?;
                                let owner = plan.type_id.clone();
                                let signature = match n.operation.as_str() {
                                    "construction_begin" => object_construction_signature(
                                        self.r,
                                        self.c,
                                        &format!("object.begin.{owner}"),
                                    )
                                    .map_err(|_| DataPhaseError::Emission)?,
                                    "construction_invoke" => {
                                        let signature = self
                                            .signatures
                                            .get(&graph.operations[ordinal].symbol)
                                            .cloned()
                                            .ok_or(DataPhaseError::Emission)?;
                                        let mut args = BTreeMap::new();
                                        let expression = find_expr(&expressions, ordinal)
                                            .ok_or(DataPhaseError::Source)?;
                                        for (argument, value) in expression
                                            .children
                                            .iter()
                                            .filter(|e| e.operation.kind() == "Argument")
                                            .zip(operands.iter().skip(1))
                                        {
                                            let index = argument
                                                .operation
                                                .symbol()
                                                .strip_prefix("argument:")
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .ok_or(DataPhaseError::Source)?;
                                            args.insert(index, value.clone());
                                        }
                                        operands = std::iter::once(operands[0].clone())
                                            .chain(args.into_values())
                                            .collect();
                                        signature
                                    }
                                    "construction_write" => {
                                        let expression = find_expr(
                                            &expressions,
                                            n.source_ordinal.ok_or(DataPhaseError::Source)?,
                                        )
                                        .ok_or(DataPhaseError::Source)?;
                                        let name = expression.children[0]
                                            .operation
                                            .symbol()
                                            .strip_prefix(&format!("{owner}."))
                                            .ok_or(DataPhaseError::Source)?;
                                        let member = self.r.source_types[&owner]
                                            .members
                                            .iter()
                                            .find(|m| m.name == name)
                                            .ok_or(DataPhaseError::Source)?;
                                        let signature = object_construction_signature(
                                            self.r,
                                            self.c,
                                            &format!("object.write.{}", member.id),
                                        )
                                        .map_err(|_| DataPhaseError::Emission)?;
                                        let value = self.publish(body, operands[1].clone())?;
                                        operands[1] = self.coerce(
                                            body,
                                            value,
                                            &signature.argument_type_ids[1],
                                        )?;
                                        signature
                                    }
                                    _ => object_construction_signature(
                                        self.r,
                                        self.c,
                                        &format!("object.finalize.{owner}"),
                                    )
                                    .map_err(|_| DataPhaseError::Emission)?,
                                };
                                let args =
                                    self.call_arguments(body, &signature, operands.clone())?;
                                let node_id = body.function.blocks[body.current].node.id.clone();
                                let result = self.invoke(body, signature, args)?;
                                let record = initializers.entry(ordinal).or_insert_with(|| {
                                    v::PracticalObjectInitialization {
                                        source_node_ordinal: ordinal,
                                        begin_node_id: String::new(),
                                        constructor_node_id: String::new(),
                                        assignment_node_ids: vec![],
                                        finalize_node_id: String::new(),
                                    }
                                });
                                match n.operation.as_str() {
                                    "construction_begin" => record.begin_node_id = node_id,
                                    "construction_invoke" => record.constructor_node_id = node_id,
                                    "construction_write" => {
                                        record.assignment_node_ids.push(node_id)
                                    }
                                    _ => record.finalize_node_id = node_id,
                                }
                                result
                            }
                            _ => {
                                let expression = find_expr(
                                    &expressions,
                                    n.source_ordinal.ok_or(DataPhaseError::Emission)?,
                                )
                                .ok_or(DataPhaseError::Emission)?;
                                if expression.children.len() != operands.len() {
                                    return Err(DataPhaseError::Emission);
                                }
                                for (child, value) in expression.children.iter().zip(&operands) {
                                    body.expression_values.insert(child.ordinal, value.clone());
                                    if matches!(
                                        child.operation.kind(),
                                        "LocalReference" | "ParameterReference"
                                    ) {
                                        body.variables
                                            .insert(child.operation.symbol().into(), value.clone());
                                    }
                                }
                                let result = self.expression(body, expression)?;
                                if n.operation == "unary_update" {
                                    body.variables
                                        .get(expression.children[0].operation.symbol())
                                        .cloned()
                                        .ok_or(DataPhaseError::Emission)?
                                } else {
                                    result
                                }
                            }
                        }
                    };
                    let expected = get(&n.result)?;
                    if result.type_id != expected.type_id {
                        return Err(DataPhaseError::Emission);
                    }
                    if !aliases.contains_key(&n.result) {
                        replacements.insert(expected.id, result.id);
                    }
                }
                "branch" | "loop_header" | "handler_filter_result" => {
                    if operands.len() != 1 || operands[0].type_id != BOOL_TYPE_ID {
                        return Err(DataPhaseError::Emission);
                    }
                    let block = &mut body.function.blocks[body.current];
                    block.node.tag = if n.kind != "loop_header" {
                        ControlNodeTag::Branch
                    } else {
                        ControlNodeTag::LoopHeader
                    };
                    block.condition_value_id = Some(operands[0].id.clone());
                    block.node.condition_type_id = Some(BOOL_TYPE_ID.into());
                    if n.kind == "loop_header" {
                        block.node.loop_id = Some(n.slot.clone());
                    }
                }
                "break" | "continue" | "handler_completion"
                    if matches!(n.kind.as_str(), "break" | "continue")
                        || matches!(n.operation.as_str(), "break" | "continue") =>
                {
                    let is_break = n.kind == "break" || n.operation == "break";
                    let target_id = body.function.blocks[starts[&n.successors[0]]]
                        .node
                        .id
                        .clone();
                    let block = &mut body.function.blocks[body.current];
                    block.node.loop_id = Some(n.slot.clone());
                    block.node.tag = if is_break {
                        ControlNodeTag::Break
                    } else {
                        ControlNodeTag::Continue
                    };
                    block.node.abrupt = Some(if is_break {
                        AbruptCompletion::Break {
                            loop_id: n.slot.clone(),
                            target_id,
                        }
                    } else {
                        AbruptCompletion::Continue {
                            loop_id: n.slot.clone(),
                            target_id,
                        }
                    });
                    // Ordinary VIR carries these edges in the abrupt target.
                    body.ended = true;
                }
                "handler_completion" if n.operation == "return" && n.successors.is_empty() => {
                    let value = operands
                        .first()
                        .cloned()
                        .map(|v| self.publish(body, v))
                        .transpose()?;
                    body.finish(value)?
                }
                "return" => {
                    let value = operands
                        .first()
                        .cloned()
                        .map(|v| self.publish(body, v))
                        .transpose()?;
                    let value = value
                        .map(|v| self.coerce(body, v, &return_type))
                        .transpose()?;
                    body.finish(if body.function.result_type_ids.is_empty() {
                        None
                    } else {
                        value
                    })?;
                }
                "rethrow" | "closed_rethrow" => {
                    let clause = handler
                        .regions
                        .iter()
                        .flat_map(|r| &r.catches)
                        .find(|c| {
                            c.id == if n.kind == "closed_rethrow" {
                                n.operation.as_str()
                            } else {
                                n.slot.as_str()
                            }
                        })
                        .ok_or(DataPhaseError::Source)?;
                    let value = get(&format!("{}.exception", clause.entry))?;
                    self.throw_closed(
                        body,
                        if n.kind == "closed_rethrow" {
                            &n.slot
                        } else {
                            &clause.type_id
                        },
                        value,
                    )?;
                    let catch_id = body.function.blocks[starts[&clause.entry]].node.id.clone();
                    let block = &mut body.function.blocks[body.current];
                    block.node.tag = ControlNodeTag::Rethrow;
                    if let Some(AbruptCompletion::Throw {
                        rethrow_from_catch_id,
                        ..
                    }) = &mut block.node.abrupt
                    {
                        *rethrow_from_catch_id = Some(catch_id);
                    }
                }
                "explicit_throw" | "completion_throw" => {
                    self.throw_closed(body, &n.slot, operands[0].clone())?
                }
                "builtin_throw" => {
                    let value = self.closed_exception(body, &n.slot, vec![])?;
                    self.throw_closed(body, &n.slot, value)?;
                }
                "handler_search" if n.successors.is_empty() => {
                    let block = &mut body.function.blocks[body.current];
                    block.node.tag = ControlNodeTag::Exit;
                    block.node.abrupt = Some(AbruptCompletion::Normal);
                }
                _ => return Err(DataPhaseError::Emission),
            }
            body.normal_construction_origins.insert(
                body.function.blocks[body.current].node.id.clone(),
                body.live_constructions.keys().cloned().collect(),
            );
            let mut node_ids = vec![body.function.blocks[starts[&n.id]].node.id.clone()];
            node_ids.extend(
                body.function.blocks[first_extra_block..]
                    .iter()
                    .map(|b| b.node.id.clone()),
            );
            node_ids.sort();
            emitted_nodes.insert(n.id.clone(), node_ids);
            ends.insert(n.id.clone(), body.current);
            if !body.ended {
                body.function.blocks[body.current].node.normal_successor_ids = n
                    .successors
                    .iter()
                    .map(|id| body.function.blocks[starts[id]].node.id.clone())
                    .collect();
            }
        }
        handlers::attach_direct_catches(body, &graph, &handler, universe, &starts, &emitted_nodes)?;
        for (n, block) in graph.nodes.iter().zip(&ssa.blocks) {
            if n.kind == "handler_search" {
                continue;
            }
            let mut phis = Vec::new();
            for phi in block
                .phis
                .iter()
                .filter(|p| !aliases.contains_key(&p.id))
                .chain(&memory.blocks[&n.id].phis)
            {
                let mut incoming = phi
                    .incoming
                    .iter()
                    .map(|(p, value)| {
                        let target = &body.function.blocks[starts[&n.id]].node.id;
                        let predecessors = emitted_nodes[p].iter().filter(|id| {
                            let block = body.function.blocks.iter().find(|b| b.node.id == **id).unwrap();
                            block.node.normal_successor_ids.contains(target)
                                || block.node.exceptional_successors.iter().any(|e| e.target_id == *target)
                                || matches!(&block.node.abrupt, Some(AbruptCompletion::Break{target_id,..} | AbruptCompletion::Continue{target_id,..}) if target_id == target)
                        });
                        predecessors.map(|id| Ok(v::PracticalVirPhiIncoming {
                            predecessor_node_id: id.clone(),
                            value_id: get(value)?.id,
                        })).collect::<Result<Vec<_>, DataPhaseError>>()
                    })
                    .collect::<Result<Vec<_>, DataPhaseError>>()?.into_iter().flatten().collect::<Vec<_>>();
                incoming.sort_by(|a, b| a.predecessor_node_id.cmp(&b.predecessor_node_id));
                phis.push(v::PracticalVirPhiValue {
                    value: get(&phi.id)?,
                    incoming,
                });
            }
            phis.sort_by(|a, b| a.value.id.cmp(&b.value.id));
            body.function.blocks[starts[&n.id]].phi_values = phis;
        }
        for region in &graph.loops {
            let start = |id: &String| body.function.blocks[starts[id]].node.id.clone();
            let mut backedges = region
                .backedges
                .iter()
                .filter_map(|id| {
                    ends.get(id)
                        .map(|i| body.function.blocks[*i].node.id.clone())
                })
                .collect::<Vec<_>>();
            backedges.sort_by_key(|id| {
                body.function
                    .blocks
                    .iter()
                    .find(|b| b.node.id == *id)
                    .unwrap()
                    .node
                    .ordinal
            });
            body.function.loops.push(LoopRegion {
                id: region.loop_id.clone(),
                parent_loop_id: region.parent.clone(),
                header_node_id: start(&region.header),
                body_entry_node_id: start(&region.body),
                continue_target_node_id: if starts.contains_key(&region.continue_target) {
                    start(&region.continue_target)
                } else if backedges.is_empty() {
                    start(&region.header)
                } else {
                    return Err(DataPhaseError::Emission);
                },
                break_target_node_id: start(&region.exit),
                backedge_source_ids: backedges,
            });
        }
        body.function.control_protocol = Some(v::PracticalControlProtocol {
            escape_search_node_ids: graph
                .nodes
                .iter()
                .filter(|n| n.kind == "handler_escape")
                .map(|n| body.function.blocks[starts[&n.id]].node.id.clone())
                .collect(),
            anchors: graph
                .nodes
                .iter()
                .filter(|n| n.kind != "handler_search")
                .map(|n| {
                    Ok(v::PracticalControlAnchor {
                        source_node_id: n.id.clone(),
                        source_ordinal: n.source_ordinal,
                        source_span: n
                            .source_ordinal
                            .and_then(|i| graph.operation_locations.as_ref().and_then(|v| v.get(i)))
                            .cloned(),
                        artifact_node_ids: emitted_nodes[&n.id].clone(),
                        entry_node_id: body.function.blocks[starts[&n.id]].node.id.clone(),
                        exit_node_id: body.function.blocks[ends[&n.id]].node.id.clone(),
                        result: if n.result.is_empty() {
                            None
                        } else {
                            Some(get(&n.result)?)
                        },
                    })
                })
                .collect::<Result<Vec<_>, DataPhaseError>>()?,
        });
        let replacements = replacements
            .keys()
            .map(|id| Ok((id.clone(), resolve_alias(&replacements, id)?)))
            .collect::<Result<BTreeMap<_, _>, DataPhaseError>>()?;
        let replace = |id: &mut String| {
            if let Some(actual) = replacements.get(id) {
                *id = actual.clone();
            }
        };
        for block in &mut body.function.blocks {
            for phi in &mut block.phi_values {
                for incoming in &mut phi.incoming {
                    replace(&mut incoming.value_id);
                }
            }
            if let Some(id) = &mut block.abrupt_value_id {
                replace(id);
            }
            if let Some(id) = &mut block.handler_exception_source_id {
                replace(id);
            }
            if let Some(id) = &mut block.condition_value_id {
                replace(id);
            }
            for id in &mut block.return_value_ids {
                replace(id);
            }
            if let Some(call) = &mut block.invocation {
                for operand in &mut call.operands {
                    replace(&mut operand.id);
                }
            }
            for action in &mut block.construction_actions {
                if let v::PracticalConstructionAction::Discard {
                    construction_id, ..
                } = action
                {
                    replace(construction_id);
                }
            }
        }
        if let Some(protocol) = &mut body.function.control_protocol {
            for anchor in &mut protocol.anchors {
                if let Some(result) = &mut anchor.result {
                    replace(&mut result.id);
                }
            }
        }
        if !initializers.is_empty() {
            body.object_protocol()
                .initializations
                .extend(initializers.into_values());
        }
        for origins in body
            .exception_construction_origins
            .values_mut()
            .chain(body.normal_construction_origins.values_mut())
            .chain(body.exception_object_origins.values_mut())
        {
            *origins = std::mem::take(origins)
                .into_iter()
                .map(|mut id| {
                    replace(&mut id);
                    id
                })
                .collect();
        }
        handlers::prune_native(&mut body.function);
        body.expression_values.clear();
        body.ended = true;
        Ok(())
    }
}
