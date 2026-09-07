//! Closed exception values and explicit abrupt exits, without CLR objects.
use super::*;
impl Emitter<'_> {
    pub(super) fn closed_exception(
        &mut self,
        body: &mut Body,
        exception: &str,
        operands: Vec<TypedValueRef>,
    ) -> Result<TypedValueRef, DataPhaseError> {
        let control = self.control.clone().ok_or(DataPhaseError::Source)?;
        let arm = control
            .universe()
            .arm(exception)
            .ok_or(DataPhaseError::Emission)?;
        if arm.tag < 9 {
            if !operands.is_empty() {
                return Err(DataPhaseError::Emission);
            }
            Ok(body.literal(MonomorphicValue::ClosedException {
                type_id: EXCEPTION_TYPE_ID.into(),
                tag: arm.tag,
                source_type_id: None,
                payload: None,
            }))
        } else {
            if operands.len() != 1 || operands[0].type_id != exception {
                return Err(DataPhaseError::Emission);
            }
            self.invoke(
                body,
                ClosedOperationSignature {
                    id: format!("{EXCEPTION_TYPE_ID}.construct.{exception}"),
                    tag: ClosedOperationTag::ExceptionConstruct,
                    argument_type_ids: vec![exception.into()],
                    normal_result_type_id: EXCEPTION_TYPE_ID.into(),
                    ordered_checks: vec![],
                },
                operands,
            )
        }
    }
    pub(super) fn throw_closed(
        &mut self,
        body: &mut Body,
        exception: &str,
        value: TypedValueRef,
    ) -> Result<(), DataPhaseError> {
        if value.type_id != EXCEPTION_TYPE_ID {
            return Err(DataPhaseError::Emission);
        }
        let block = &mut body.function.blocks[body.current];
        block.node.tag = ControlNodeTag::Throw;
        block.abrupt_value_id = Some(value.id);
        block.node.abrupt = Some(AbruptCompletion::Throw {
            exception_type_id: exception.into(),
            rethrow_from_catch_id: None,
        });
        block.node.exceptional_successors = vec![ExceptionalSuccessor {
            check_id: format!("exception.closed.{exception}"),
            exception_type_id: exception.into(),
            target_id: "pending.exit".into(),
        }];
        body.exception_construction_origins
            .entry(block.node.id.clone())
            .or_default()
            .extend(body.live_constructions.keys().cloned());
        body.exception_object_origins
            .entry(block.node.id.clone())
            .or_default()
            .extend(body.live_objects.keys().cloned());
        body.ended = true;
        Ok(())
    }
    pub(super) fn source_throw(
        &mut self,
        body: &mut Body,
        node: &Expr<'_>,
    ) -> Result<(), DataPhaseError> {
        let mut value = node.children.first().ok_or(DataPhaseError::Emission)?;
        while value.operation.kind() == "Conversion" {
            value = value.children.first().ok_or(DataPhaseError::Emission)?;
        }
        if value.operation.kind() != "ObjectCreation" {
            return Err(DataPhaseError::Emission);
        }
        let control = self.control.clone().ok_or(DataPhaseError::Source)?;
        let f = control
            .functions()
            .iter()
            .find(|f| f.callable_id == body.function.id)
            .ok_or(DataPhaseError::Emission)?;
        let exception = f
            .nodes
            .iter()
            .find(|n| n.operation == "closed_exception" && n.source_ordinal == Some(value.ordinal))
            .ok_or(DataPhaseError::Emission)?
            .slot
            .clone();
        let args = if control
            .universe()
            .arm(&exception)
            .ok_or(DataPhaseError::Emission)?
            .tag
            < 9
        {
            vec![]
        } else {
            vec![self.expression(body, value)?]
        };
        let value = self.closed_exception(body, &exception, args)?;
        self.throw_closed(body, &exception, value)
    }
}
