//! Original return statements, enclosing signatures and named result scopes.
//! This checks result types; it does not execute or prove a function body.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoReturnTyping {
    source: String,
    statement: usize,
    signature: GoSourceSignature,
    implicit: bool,
    bindings: Vec<Option<GoSourceNameBinding>>,
    typing: GoValueTyping,
}
impl GoReturnTyping {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn statement_node(&self) -> usize {
        self.statement
    }
    /// The actual nearest enclosing FuncDecl/FuncLit, selected through syntax
    /// edges, not byte containment or a caller-supplied signature.
    pub fn signature(&self) -> &GoSourceSignature {
        &self.signature
    }
    /// True for an empty original result expression list, including void return.
    pub fn implicit_results(&self) -> bool {
        self.implicit
    }
    /// Ordered result variables. Blank and unnamed slots have no variable.
    /// An implicit return requires every nonblank variable to remain in scope.
    pub fn result_bindings(&self) -> &[Option<GoSourceNameBinding>] {
        &self.bindings
    }
    /// Nonconstant, nonaddressable result-slot types. No result payload or
    /// named-result zero-initialization/evaluation proof is asserted here.
    pub fn typing(&self) -> &GoValueTyping {
        &self.typing
    }
}
impl Engine<'_> {
    fn return_signature(&mut self, source: &str, statement: usize) -> Result<GoSourceSignature> {
        if self.node(source, statement)?.kind() != "ReturnStmt" {
            return Err(GoScalarTypingError::Locator);
        }
        if !self.function_owners.contains_key(source) {
            let parsed = self
                .scopes
                .source(source)
                .ok_or(GoScalarTypingError::Locator)?;
            let mut owners = vec![None; parsed.nodes().len()];
            let mut seen = vec![false; parsed.nodes().len()];
            let mut work = vec![(parsed.root(), None)];
            while let Some((node, outer)) = work.pop() {
                let n = &parsed.nodes()[node];
                let owner = if matches!(n.kind(), "FuncDecl" | "FuncLit") {
                    Some(node)
                } else {
                    outer
                };
                if seen[node] {
                    if owners[node] != owner {
                        return Err(GoScalarTypingError::Locator);
                    }
                    continue;
                }
                seen[node] = true;
                owners[node] = owner;
                for edge in n.edges() {
                    for &child in edge.targets() {
                        work.push((child, owner));
                    }
                }
            }
            self.function_owners.insert(source.into(), owners);
        }
        let function =
            self.function_owners[source][statement].ok_or(GoScalarTypingError::Locator)?;
        let key = (source.to_string(), function);
        if let Some(signature) = self.return_signatures.get(&key) {
            return Ok(signature.clone());
        }
        let signature = derive_go_source_signature(self.scopes, source, function)
            .map_err(GoScalarTypingError::Signature)?;
        if let Some(root) = signature.types().roots().first() {
            self.graph_type(signature.types(), root)?;
        }
        self.return_signatures.insert(key, signature.clone());
        Ok(signature)
    }
    fn result_bindings(
        &self,
        signature: &GoSourceSignature,
    ) -> Result<Vec<Option<GoSourceNameBinding>>> {
        signature
            .slots()
            .iter()
            .filter(|s| s.role() == GoSourceSignatureRole::Result)
            .map(|slot| {
                let Some(name) = slot.name().filter(|name| *name != "_") else {
                    return Ok(None);
                };
                self.scopes
                    .bindings()
                    .iter()
                    .find(|b| {
                        b.kind() == GoSourceNameKind::Variable
                            && b.source_object_id() == signature.source_object_id()
                            && b.declaration_node() == signature.function_node()
                            && Some(b.name_node()) == slot.name_node()
                            && b.name() == name
                    })
                    .cloned()
                    .map(Some)
                    .ok_or(GoScalarTypingError::Locator)
            })
            .collect()
    }
    pub(super) fn return_plan(&mut self, source: &str, statement: usize) -> Result<Step> {
        let signature = self.return_signature(source, statement)?;
        let results = signature
            .slots()
            .iter()
            .filter(|s| s.role() == GoSourceSignatureRole::Result)
            .collect::<Vec<_>>();
        let types = results
            .iter()
            .map(|s| s.type_id().to_string())
            .collect::<Vec<_>>();
        let exprs = list(self.node(source, statement)?, "Results");
        if exprs.is_empty() {
            if results.iter().any(|s| s.name().is_none()) {
                return Err(GoScalarTypingError::Arity);
            }
            for b in self.result_bindings(&signature)?.iter().flatten() {
                let actual = self
                    .scopes
                    .lookup_context(source, b.name_node(), Some(statement))
                    .map_err(|_| GoScalarTypingError::Name)?;
                if actual != Some(GoNameLookup::Original(b.clone())) {
                    return Err(GoScalarTypingError::Name);
                }
            }
            return Ok(Step::Done(
                types.iter().map(|t| self.atom(t, false)).collect(),
            ));
        }
        Ok(Step::More(Plan::ReturnFirst(
            types,
            exprs.iter().map(|&n| Request::raw(source, n)).collect(),
        )))
    }
    pub(super) fn return_first(&mut self, types: Vec<String>, args: Vec<Request>) -> Result<Step> {
        let facts = self.expand(&args)?;
        if facts.len() != types.len() {
            return Err(GoScalarTypingError::Arity);
        }
        if args.len() == 1 && facts.len() != 1 {
            return self.return_result(&types, &facts);
        }
        let next = args
            .iter()
            .zip(&types)
            .map(|(r, typ)| {
                Ok(Request::value(
                    &r.source,
                    r.expression()?,
                    Some(typ.clone()),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Step::More(Plan::ReturnReady(types, next)))
    }
    pub(super) fn return_result(&self, types: &[String], facts: &[Atom]) -> Result<Step> {
        self.check_types(facts, types)?;
        Ok(Step::Done(
            types.iter().map(|t| self.atom(t, false)).collect(),
        ))
    }
}
/// Check one actual return against its original enclosing function signature.
/// Whole-body control/termination, invocation state and return payload proofs
/// remain separate. Nested function ownership grants no closure admission.
pub fn check_go_return_statement(
    scopes: &GoSourceScopes,
    source: &str,
    statement: usize,
) -> Result<GoReturnTyping> {
    let mut engine = Engine::new(scopes);
    let signature = engine.return_signature(source, statement)?;
    let bindings = engine.result_bindings(&signature)?;
    let implicit = list(engine.node(source, statement)?, "Results").is_empty();
    let typing = engine.run(Request {
        source: source.into(),
        target: Target::Return(statement),
    })?;
    Ok(GoReturnTyping {
        source: source.into(),
        statement,
        signature,
        implicit,
        bindings,
        typing,
    })
}
