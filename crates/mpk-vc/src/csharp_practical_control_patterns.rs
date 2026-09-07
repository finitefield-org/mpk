//! Primitive operations in source-ordered pattern decision graphs.
use super::*;
impl Emitter<'_> {
    fn pattern_present(
        &mut self,
        body: &mut Body,
        value: TypedValueRef,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if self
            .c
            .metadata
            .get(&value.type_id)
            .is_some_and(|m| template_name(&m.template_id) == Some("option"))
        {
            let signature = self
                .signatures
                .get(&format!("{}.has_value", value.type_id))
                .cloned()
                .ok_or(DataPhaseError::Emission)?;
            self.invoke(body, signature, vec![value])
        } else {
            Ok(body.literal(MonomorphicValue::Bool {
                type_id: BOOL_TYPE_ID.into(),
                value: true,
            }))
        }
    }
    pub(super) fn pattern_payload(
        &mut self,
        body: &mut Body,
        value: TypedValueRef,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if self
            .c
            .metadata
            .get(&value.type_id)
            .is_some_and(|m| template_name(&m.template_id) == Some("option"))
        {
            let signature = self
                .signatures
                .get(&format!("{}.value", value.type_id))
                .cloned()
                .ok_or(DataPhaseError::Emission)?;
            self.invoke(body, signature, vec![value])
        } else {
            Ok(value)
        }
    }
    fn pattern_compare(
        &mut self,
        body: &mut Body,
        operation: &str,
        args: Vec<TypedValueRef>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        if args.len() != 2 {
            return Err(DataPhaseError::Emission);
        }
        if args[0].type_id != args[1].type_id
            && self.c.metadata.get(&args[0].type_id).is_some_and(|m| {
                template_name(&m.template_id) == Some("option")
                    && m.argument_ids == [args[1].type_id.clone()]
            })
        {
            let present = self.pattern_present(body, args[0].clone())?;
            let branch = body.current;
            let yes = body.block(ControlNodeTag::Jump);
            let no = body.block(ControlNodeTag::Jump);
            let join = body.block(ControlNodeTag::Jump);
            body.function.blocks[branch].node.tag = ControlNodeTag::Branch;
            body.function.blocks[branch].condition_value_id = Some(present.id);
            body.function.blocks[branch].node.condition_type_id = Some(BOOL_TYPE_ID.into());
            body.function.blocks[branch].node.normal_successor_ids = vec![
                body.function.blocks[yes].node.id.clone(),
                body.function.blocks[no].node.id.clone(),
            ];
            body.current = yes;
            let payload = self.pattern_payload(body, args[0].clone())?;
            let matched = self.pattern_compare(body, operation, vec![payload, args[1].clone()])?;
            let yes_end = body.current;
            body.function.blocks[yes_end].node.normal_successor_ids =
                vec![body.function.blocks[join].node.id.clone()];
            body.current = no;
            let unmatched = body.literal(MonomorphicValue::Bool {
                type_id: BOOL_TYPE_ID.into(),
                value: false,
            });
            body.function.blocks[no].node.normal_successor_ids =
                vec![body.function.blocks[join].node.id.clone()];
            let result = body.value(BOOL_TYPE_ID);
            let mut incoming = vec![
                v::PracticalVirPhiIncoming {
                    predecessor_node_id: body.function.blocks[yes_end].node.id.clone(),
                    value_id: matched.id,
                },
                v::PracticalVirPhiIncoming {
                    predecessor_node_id: body.function.blocks[no].node.id.clone(),
                    value_id: unmatched.id,
                },
            ];
            incoming.sort_by(|a, b| a.predecessor_node_id.cmp(&b.predecessor_node_id));
            body.function.blocks[join]
                .phi_values
                .push(v::PracticalVirPhiValue {
                    value: result.clone(),
                    incoming,
                });
            body.current = join;
            return Ok(result);
        }
        if args[0].type_id != args[1].type_id {
            return Err(DataPhaseError::Emission);
        }
        let ty = args[0].type_id.clone();
        if let Some(family) = match ty.as_str() {
            "mpk.csharp.value.f32.v1" => Some("floating.single"),
            "mpk.csharp.value.f64.v1" => Some("floating.double"),
            "mpk.csharp.value.decimal.v1" => Some("decimal"),
            _ => None,
        } {
            let (operation, args) = if operation == "constant_nan" {
                ("is_nan", vec![args[0].clone()])
            } else {
                (operation, args)
            };
            let id = format!("{family}.{operation}");
            let types = args.iter().map(|v| v.type_id.clone()).collect::<Vec<_>>();
            let recipe = NumericOperation::new(&id, &types, BOOL_TYPE_ID, None)
                .map_err(|_| DataPhaseError::Emission)?;
            return self.invoke(
                body,
                ClosedOperationSignature {
                    id,
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: types,
                    normal_result_type_id: BOOL_TYPE_ID.into(),
                    ordered_checks: recipe
                        .exception_types()
                        .into_iter()
                        .map(exception_check)
                        .collect::<Result<_, _>>()?,
                },
                args,
            );
        }
        if operation == "equal" {
            generate_structural_program(self.b, self.r, self.c, &ty)
                .map_err(|_| DataPhaseError::Emission)?;
            return self.invoke(
                body,
                ClosedOperationSignature {
                    id: format!("structural.equal.{ty}"),
                    tag: ClosedOperationTag::StructuralEqual,
                    argument_type_ids: vec![ty.clone(), ty],
                    normal_result_type_id: BOOL_TYPE_ID.into(),
                    ordered_checks: vec![],
                },
                args,
            );
        }
        let token = ty
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
            .ok_or(DataPhaseError::Emission)?;
        let signature =
            scalar_operation_signature(&format!("integer.{token}.{operation}.unchecked"))
                .map_err(|_| DataPhaseError::Emission)?;
        self.invoke(body, signature, args)
    }
    pub(super) fn pattern_operation(
        &mut self,
        body: &mut Body,
        n: &LoopControlNode,
        graph: &LoopControlFunction,
        args: Vec<TypedValueRef>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        match n.operation.as_str() {
            "pattern_true" | "pattern_false" => Ok(body.literal(MonomorphicValue::Bool {
                type_id: BOOL_TYPE_ID.into(),
                value: n.operation == "pattern_true",
            })),
            "pattern_bind" => self.pattern_payload(body, args[0].clone()),
            "pattern_not_null" | "pattern_type" => self.pattern_present(body, args[0].clone()),
            "pattern_equal" => {
                let nan = graph
                    .nodes
                    .iter()
                    .find(|v| v.result == n.inputs[1])
                    .and_then(|v| v.source_ordinal)
                    .and_then(|i| graph.operations[i].constant.as_deref())
                    .and_then(|constant| source_literal(&args[1].type_id, constant).ok())
                    .is_some_and(|literal| match literal {
                        MonomorphicValue::F32Bits { bits, .. } => u32::from_str_radix(&bits, 16)
                            .is_ok_and(|b| (b & 0x7f800000) == 0x7f800000 && (b & 0x007fffff) != 0),
                        MonomorphicValue::F64Bits { bits, .. } => u64::from_str_radix(&bits, 16)
                            .is_ok_and(|b| {
                                (b & 0x7ff0000000000000) == 0x7ff0000000000000
                                    && (b & 0x000fffffffffffff) != 0
                            }),
                        _ => false,
                    });
                if nan {
                    return self.pattern_compare(body, "constant_nan", args);
                }
                if args[1].type_id == "mpk.csharp.value.unit.v1" {
                    let present = self.pattern_present(body, args[0].clone())?;
                    self.invoke(
                        body,
                        scalar_operation_signature("boolean.not")
                            .map_err(|_| DataPhaseError::Emission)?,
                        vec![present],
                    )
                } else {
                    self.pattern_compare(body, "equal", args)
                }
            }
            "pattern_relational" => {
                let operation = match graph.operations
                    [n.source_ordinal.ok_or(DataPhaseError::Emission)?]
                .traits
                .as_str()
                {
                    "LessThan" => "less",
                    "LessThanOrEqual" => "less_equal",
                    "GreaterThan" => "greater",
                    "GreaterThanOrEqual" => "greater_equal",
                    _ => return Err(DataPhaseError::Emission),
                };
                self.pattern_compare(body, operation, args)
            }
            "pattern_length" | "pattern_element" => {
                let receiver = self.pattern_payload(body, args[0].clone())?;
                let index = body.literal(MonomorphicValue::Signed {
                    type_id: I32_TYPE_ID.into(),
                    value: n.slot.clone(),
                });
                if n.operation == "pattern_element" {
                    let signature = self
                        .signatures
                        .get(&format!("{}.read", receiver.type_id))
                        .cloned()
                        .ok_or(DataPhaseError::Emission)?;
                    return self.invoke(body, signature, vec![receiver, index]);
                }
                let length =
                    if self.c.metadata.get(&receiver.type_id).is_some_and(|m| {
                        template_name(&m.template_id) == Some("sequence_construction")
                    }) {
                        let origin = body
                            .live_constructions
                            .iter()
                            .find(|(_, v)| v.id == receiver.id)
                            .map(|(o, _)| o)
                            .ok_or(DataPhaseError::Emission)?;
                        body.construction_lengths
                            .get(origin)
                            .cloned()
                            .ok_or(DataPhaseError::Emission)?
                    } else {
                        let signature = self
                            .signatures
                            .get(&format!("{}.length", receiver.type_id))
                            .cloned()
                            .ok_or(DataPhaseError::Emission)?;
                        let length = self.invoke(body, signature, vec![receiver])?;
                        self.invoke(
                            body,
                            scalar_operation_signature("integer.convert.u32.i32.unchecked")
                                .map_err(|_| DataPhaseError::Emission)?,
                            vec![length],
                        )?
                    };
                self.pattern_compare(body, "equal", vec![length, index])
            }
            _ => Err(DataPhaseError::Emission),
        }
    }
}
