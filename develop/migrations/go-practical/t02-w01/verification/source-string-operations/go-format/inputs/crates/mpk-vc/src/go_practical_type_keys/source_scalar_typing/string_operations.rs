//! Derive string operation sites from every independently checked selected body.
//! These are original operand/type bindings, not operand execution or a callee proof.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoSourceStringOperationKind {
    Length,
    Index,
    /// Go evaluates this as a constant; no runtime string operand is captured.
    ConstantLength(u64),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceStringOperation {
    caller: GoSourceSelectionRef,
    node: usize,
    start: usize,
    end: usize,
    kind: GoSourceStringOperationKind,
    operands: Vec<usize>,
    operand_types: Vec<Option<String>>,
    result: Option<String>,
    calls: Vec<GoSourceSelectionCall>,
}
impl GoSourceStringOperation {
    pub fn caller(&self) -> &GoSourceSelectionRef {
        &self.caller
    }
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn start_byte(&self) -> usize {
        self.start
    }
    pub fn end_byte(&self) -> usize {
        self.end
    }
    pub fn kind(&self) -> &GoSourceStringOperationKind {
        &self.kind
    }
    pub fn operand_nodes(&self) -> &[usize] {
        &self.operands
    }
    pub fn operand_type_ids(&self) -> &[Option<String>] {
        &self.operand_types
    }
    pub fn result_type_id(&self) -> Option<&str> {
        self.result.as_deref()
    }
    /// Every selected ordinary call inside this operation, with its real caller
    /// and callee. Capturing its result does not discharge its execution/effects.
    pub fn operand_calls(&self) -> &[GoSourceSelectionCall] {
        &self.calls
    }
}

/// Complete string-length/index site inventory over the selected body set.
/// A caller cannot replace a source node's opcode, operands, result type or U.
/// Constant length is kept separate from dynamic length; arrays/slices and a
/// source function named len are not misclassified as string builtins.
pub fn derive_go_source_string_operations(
    domain: &GoSourceValueDomain,
) -> Result<Vec<GoSourceStringOperation>> {
    let selection = domain.selection();
    let scopes = selection.scopes();
    let keys: BTreeMap<_, _> = domain.types().map(|k| (k.id(), k)).collect();
    let mut result = BTreeSet::new();
    for caller in selection.checked_body_locators() {
        let source = caller.source_object_id();
        let parsed = selection
            .source(source)
            .ok_or(GoScalarTypingError::Locator)?;
        let signature = derive_go_source_signature(scopes, source, caller.node())
            .map_err(GoScalarTypingError::Signature)?;
        let body = signature.body_node().ok_or(GoScalarTypingError::Locator)?;
        let mut engine = Engine::new(scopes);
        let mut work = vec![body];
        let mut seen = BTreeSet::new();
        while let Some(node) = work.pop() {
            if !seen.insert(node) {
                continue;
            }
            let n = parsed
                .nodes()
                .get(node)
                .ok_or(GoScalarTypingError::Locator)?;
            for edge in n.edges().iter().rev() {
                work.extend(edge.targets().iter().rev().copied());
            }
            let (kind, operands, facts, output) = if n.kind() == "CallExpr" {
                let fun = child(n, "Fun")?;
                if !matches!(engine.symbol(source,fun),Ok(GoNameLookup::Predeclared{name,kind:GoSourceNameKind::BuiltinFunction}) if name=="len")
                {
                    continue;
                }
                let args = list(n, "Args");
                if args.len() != 1 {
                    return Err(GoScalarTypingError::Arity);
                }
                let arg = Request::raw(source, args[0]);
                engine.compute(arg.clone())?;
                let fact = engine.one(&arg)?;
                if fact.kind != Some(GoConstantKind::String) {
                    continue;
                }
                let r = Request::raw(source, node);
                engine.compute(r.clone())?;
                let output = engine.one(&r)?;
                let kind = match &output.value {
                    Some(value) => GoSourceStringOperationKind::ConstantLength(
                        value
                            .integer_decimal()
                            .ok_or(GoScalarTypingError::Type)?
                            .parse()
                            .map_err(|_| GoScalarTypingError::Type)?,
                    ),
                    None => GoSourceStringOperationKind::Length,
                };
                (kind, args.to_vec(), vec![fact], output)
            } else if n.kind() == "IndexExpr" {
                let base = child(n, "X")?;
                let index = child(n, "Index")?;
                let r = Request::raw(source, base);
                engine.compute(r.clone())?;
                let mut fact = engine.one(&r)?;
                if fact.kind != Some(GoConstantKind::String) {
                    continue;
                }
                if fact.typ.is_none() {
                    let materialized = Request::value(source, base, None);
                    engine.compute(materialized.clone())?;
                    fact = engine.one(&materialized)?;
                }
                let i = Request::index(source, index);
                engine.compute(i.clone())?;
                let index_fact = engine.one(&i)?;
                let r = Request::raw(source, node);
                engine.compute(r.clone())?;
                (
                    GoSourceStringOperationKind::Index,
                    vec![base, index],
                    vec![fact, index_fact],
                    engine.one(&r)?,
                )
            } else {
                continue;
            };
            let operand_types = facts.into_iter().map(|f| f.typ).collect::<Vec<_>>();
            // Materialization keys must already belong to the full original
            // selected source domain; this inventory cannot fabricate a type.
            for id in operand_types.iter().flatten().chain(output.typ.iter()) {
                if !keys.contains_key(id.as_str()) {
                    return Err(GoScalarTypingError::Type);
                }
            }
            let calls = selection
                .calls()
                .iter()
                .filter(|call| {
                    call.caller() == caller
                        && parsed
                            .nodes()
                            .get(call.call_node())
                            .is_some_and(|c| c.start() >= n.start() && c.end() <= n.end())
                })
                .cloned()
                .collect();
            result.insert(GoSourceStringOperation {
                caller: caller.clone(),
                node,
                start: n.start(),
                end: n.end(),
                kind,
                operands,
                operand_types,
                result: output.typ,
                calls,
            });
        }
    }
    Ok(result.into_iter().collect())
}
