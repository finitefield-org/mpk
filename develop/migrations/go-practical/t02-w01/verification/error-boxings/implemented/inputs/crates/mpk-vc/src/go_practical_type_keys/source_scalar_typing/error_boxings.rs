//! Checked concrete-to-error uses. A site can occur in an unevaluated operand;
//! only the source-selection runtime walk determines the body boxing union.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoErrorBoxing {
    source: String,
    expression: usize,
    projection: GoErrorBoxingProjection,
    value_type: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoErrorBoxingProjection {
    Result(usize),
    RangeElement,
}
impl GoErrorBoxing {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn expression_node(&self) -> usize {
        self.expression
    }
    pub fn projection(&self) -> GoErrorBoxingProjection {
        self.projection
    }
    pub fn value_type_id(&self) -> &str {
        &self.value_type
    }
}
impl Engine<'_> {
    pub(super) fn error_arguments(
        &mut self,
        args: &[Request],
        facts: &[Atom],
        types: &[String],
    ) -> Result<()> {
        self.check_types(facts, types)?;
        self.record_error_boxings(
            args,
            facts,
            &types.iter().cloned().map(Some).collect::<Vec<_>>(),
        )
    }
    pub(super) fn record_error_boxings(
        &mut self,
        args: &[Request],
        facts: &[Atom],
        targets: &[Option<String>],
    ) -> Result<()> {
        if facts.len() != targets.len() || (args.len() != facts.len() && args.len() != 1) {
            return Err(GoScalarTypingError::Arity);
        }
        for (index, (fact, target)) in facts.iter().zip(targets).enumerate() {
            let Some(target) = target else { continue };
            let Some(from) = &fact.typ else { continue };
            if fact.nil || !self.is_error(target) || self.is_error(from) {
                continue;
            }
            if !self.assignable(from, target) {
                return Err(GoScalarTypingError::Type);
            }
            let (arg, result_slot) = if args.len() == 1 {
                (&args[0], index)
            } else {
                (&args[index], 0)
            };
            self.error_boxings.insert(GoErrorBoxing {
                source: arg.source.clone(),
                expression: arg.expression()?,
                projection: GoErrorBoxingProjection::Result(result_slot),
                value_type: from.clone(),
            });
        }
        Ok(())
    }
    pub(super) fn record_range_boxing(&mut self, source: &str, expression: usize, from: &str, to: &str) -> Result<()> {
        if self.is_error(to) && !self.is_error(from) {
            if !self.assignable(from, to) { return Err(GoScalarTypingError::Type) }
            self.error_boxings.insert(GoErrorBoxing { source: source.into(), expression, projection: GoErrorBoxingProjection::RangeElement, value_type: from.into() });
        }
        Ok(())
    }
}
