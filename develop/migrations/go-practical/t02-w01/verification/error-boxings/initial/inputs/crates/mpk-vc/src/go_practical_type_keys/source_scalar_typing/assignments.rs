//! Actual original assignment syntax and writable destination types. These
//! facts do not execute targets or establish store/alias/evaluation-order laws.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAssignmentTarget {
    node: usize,
    typ: Option<String>,
    declares: bool,
}
impl GoAssignmentTarget {
    pub fn node(&self) -> usize {
        self.node
    }
    /// None denotes an actual blank destination. Its discarded value still
    /// has to typecheck in default context.
    pub fn type_id(&self) -> Option<&str> {
        self.typ.as_deref()
    }
    pub fn declares_variable(&self) -> bool {
        self.declares
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAssignmentTyping {
    source: String,
    statement: usize,
    targets: Vec<GoAssignmentTarget>,
    typing: GoValueTyping,
}
impl GoAssignmentTyping {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn statement_node(&self) -> usize {
        self.statement
    }
    pub fn targets(&self) -> &[GoAssignmentTarget] {
        &self.targets
    }
    /// Ordered destination-slot types, with default value types for blanks.
    /// These are nonconstant type facts, not values produced by execution.
    pub fn typing(&self) -> &GoValueTyping {
        &self.typing
    }
}
impl Engine<'_> {
    pub(super) fn blank_target(&self, source: &str, mut node: usize) -> Result<bool> {
        loop {
            let n = self.node(source, node)?;
            if n.kind() != "ParenExpr" {
                return Ok(n.kind() == "Ident" && attr(n, "Name") == Some("_"));
            }
            node = child(n, "X")?;
        }
    }
    pub(super) fn assignment_plan(&mut self, source: &str, node: usize) -> Result<Step> {
        let n = self.node(source, node)?.clone();
        if n.kind() == "AssignStmt" && attr(&n, "Tok") == Some(":=") {
            return Ok(Step::More(Plan::ShortAssignment(Request::declaration(
                source, node,
            ))));
        }
        let targets = match n.kind() {
            "AssignStmt" => list(&n, "Lhs"),
            "IncDecStmt" => list(&n, "X"),
            _ => return Err(GoScalarTypingError::Unsupported),
        };
        if targets.is_empty() {
            return Err(GoScalarTypingError::Arity);
        }
        let lhs = targets
            .iter()
            .map(|&n| Ok((!self.blank_target(source, n)?).then(|| Request::raw(source, n))))
            .collect::<Result<Vec<_>>>()?;
        if n.kind() == "AssignStmt" && attr(&n, "Tok") == Some("=") {
            let rhs = list(&n, "Rhs")
                .iter()
                .map(|&n| Request::raw(source, n))
                .collect();
            return Ok(Step::More(Plan::AssignmentFirst(lhs, rhs)));
        }
        if lhs.len() != 1 || lhs[0].is_none() {
            return Err(GoScalarTypingError::Type);
        }
        let operation = Request {
            source: source.into(),
            target: Target::AssignmentOperation(node),
        };
        Ok(Step::More(Plan::AssignmentReady(lhs, vec![operation])))
    }
    pub(super) fn assignment_operation(&self, source: &str, node: usize) -> Result<Step> {
        let n = self.node(source, node)?;
        if n.kind() == "IncDecStmt" && matches!(attr(n, "Tok"), Some("++" | "--")) {
            return Ok(Step::More(Plan::Increment(Request::raw(
                source,
                child(n, "X")?,
            ))));
        }
        let op = match attr(n, "Tok") {
            Some("+=") => "+",
            Some("-=") => "-",
            Some("*=") => "*",
            Some("/=") => "/",
            Some("%=") => "%",
            Some("&=") => "&",
            Some("|=") => "|",
            Some("^=") => "^",
            Some("&^=") => "&^",
            Some("<<=") => "<<",
            Some(">>=") => ">>",
            _ => return Err(GoScalarTypingError::Unsupported),
        };
        Ok(Step::More(Plan::Compound(
            op.into(),
            Request::raw(source, child(n, "Lhs")?),
            Request::raw(source, child(n, "Rhs")?),
        )))
    }
    pub(super) fn assignment_targets(
        &self,
        lhs: &[Option<Request>],
    ) -> Result<Vec<Option<String>>> {
        lhs.iter()
            .map(|r| {
                let Some(r) = r else { return Ok(None) };
                let a = self.one(r)?;
                if !a.addressable || a.value.is_some() || a.nil {
                    return Err(GoScalarTypingError::Type);
                }
                Ok(Some(a.typ.ok_or(GoScalarTypingError::Type)?))
            })
            .collect()
    }
    pub(super) fn assignment_first(
        &mut self,
        lhs: Vec<Option<Request>>,
        rhs: Vec<Request>,
    ) -> Result<Step> {
        let targets = self.assignment_targets(&lhs)?;
        let facts = self.expand(&rhs)?;
        if facts.len() != targets.len() {
            return Err(GoScalarTypingError::Arity);
        }
        if rhs.len() == 1 && facts.len() != 1 {
            return self.assignment_result(&lhs, &rhs, facts);
        }
        let next = rhs
            .iter()
            .zip(targets)
            .map(|(r, typ)| Ok(Request::value(&r.source, r.expression()?, typ)))
            .collect::<Result<Vec<_>>>()?;
        Ok(Step::More(Plan::AssignmentReady(lhs, next)))
    }
    pub(super) fn assignment_result(&mut self, lhs: &[Option<Request>], rhs: &[Request], facts: Facts) -> Result<Step> {
        let targets = self.assignment_targets(lhs)?;
        if facts.len() != targets.len() {
            return Err(GoScalarTypingError::Arity);
        }
        let result = facts
            .iter()
            .zip(&targets)
            .map(|(a, target)| {
                let typ = a.typ.clone().ok_or(GoScalarTypingError::Type)?;
                let blank = target.is_none();
                let target = target.clone().unwrap_or_else(|| typ.clone());
                if !self.assignable(&typ, &target) {
                    return Err(GoScalarTypingError::Type);
                }
                Ok(self.atom(&target, !blank))
            })
            .collect::<Result<Vec<_>>>()?;
        self.record_error_boxings(rhs, &facts, &targets)?;
        Ok(Step::Done(result))
    }
}
/// Check an actual original =, :=, compound-assignment or ++/-- statement.
/// Rejects unwritable targets and checks the complete ordered destination and
/// value lists. Pointer/map targets and whole-body/state checks remain separate.
pub fn check_go_assignment_statement(
    scopes: &GoSourceScopes,
    source: &str,
    statement: usize,
) -> Result<GoAssignmentTyping> {
    let engine = Engine::new(scopes);
    let n = engine.node(source, statement)?.clone();
    let typing = engine.run(Request {
        source: source.into(),
        target: Target::Assignment(statement),
    })?;
    let nodes = if n.kind() == "IncDecStmt" {
        list(&n, "X")
    } else {
        list(&n, "Lhs")
    };
    if nodes.len() != typing.values().len() {
        return Err(GoScalarTypingError::Arity);
    }
    let mut targets = vec![];
    for (&node, value) in nodes.iter().zip(typing.values()) {
        targets.push(GoAssignmentTarget {
            node,
            typ: value
                .addressable()
                .then(|| value.source_type_key().id().into()),
            declares: attr(&n, "Tok") == Some(":=")
                && scopes.bindings().iter().any(|b| {
                    b.source_object_id() == source
                        && b.declaration_node() == statement
                        && b.name_node() == node
                }),
        });
    }
    Ok(GoAssignmentTyping {
        source: source.into(),
        statement,
        targets,
        typing,
    })
}
