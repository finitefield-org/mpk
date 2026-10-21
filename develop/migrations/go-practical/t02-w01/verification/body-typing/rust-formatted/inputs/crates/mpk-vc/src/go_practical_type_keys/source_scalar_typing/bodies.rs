//! Original function-body typing and syntactic control checks. This is not
//! evaluation, a termination theorem, selected-callee closure or program proof.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoBodyErrorKind {
    Typing(GoScalarTypingError),
    Unsupported,
    Branch,
    DuplicateCase,
    MissingReturn,
    UnusedVariable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyTypingError {
    source: String,
    node: usize,
    kind: GoBodyErrorKind,
}
impl GoBodyTypingError {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn kind(&self) -> &GoBodyErrorKind {
        &self.kind
    }
}
impl std::fmt::Display for GoBodyTypingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Go body typing error at {}:{}: {:?}",
            self.source, self.node, self.kind
        )
    }
}
impl std::error::Error for GoBodyTypingError {}
type BodyResult<T> = std::result::Result<T, GoBodyTypingError>;
fn error(source: &str, node: usize, kind: GoBodyErrorKind) -> GoBodyTypingError {
    GoBodyTypingError {
        source: source.into(),
        node,
        kind,
    }
}
fn at<T>(source: &str, node: usize, result: Result<T>) -> BodyResult<T> {
    result.map_err(|kind| error(source, node, GoBodyErrorKind::Typing(kind)))
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyStatement {
    node: usize,
    types: Vec<String>,
    terminating: bool,
}
impl GoBodyStatement {
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn value_type_ids(&self) -> &[String] {
        &self.types
    }
    /// Go's syntactic terminating-statement classification, not runtime halting.
    pub fn syntactically_terminating(&self) -> bool {
        self.terminating
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyExpression {
    node: usize,
    role: &'static str,
    typ: String,
    nil: bool,
}
impl GoBodyExpression {
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn role(&self) -> &str {
        self.role
    }
    pub fn type_id(&self) -> &str {
        &self.typ
    }
    pub fn is_untyped_nil(&self) -> bool {
        self.nil
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyBranch {
    node: usize,
    target: usize,
}
impl GoBodyBranch {
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn target_node(&self) -> usize {
        self.target
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyConstant {
    spec: usize,
    values: Vec<GoEvaluatedConstant>,
}
impl GoBodyConstant {
    pub fn spec_node(&self) -> usize {
        self.spec
    }
    pub fn values(&self) -> &[GoEvaluatedConstant] {
        &self.values
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBodyTyping {
    signature: GoSourceSignature,
    statements: Vec<GoBodyStatement>,
    expressions: Vec<GoBodyExpression>,
    branches: Vec<GoBodyBranch>,
    constants: Vec<GoBodyConstant>,
    locals: Vec<GoSourceNameBinding>,
    local_types: BTreeMap<(usize, usize), String>,
    terminating: bool,
    typing: GoValueTyping,
}
impl GoBodyTyping {
    pub fn signature(&self) -> &GoSourceSignature {
        &self.signature
    }
    pub fn statements(&self) -> &[GoBodyStatement] {
        &self.statements
    }
    pub fn expressions(&self) -> &[GoBodyExpression] {
        &self.expressions
    }
    pub fn branches(&self) -> &[GoBodyBranch] {
        &self.branches
    }
    pub fn constants(&self) -> &[GoBodyConstant] {
        &self.constants
    }
    pub fn local_variables(&self) -> &[GoSourceNameBinding] {
        &self.locals
    }
    pub fn local_type_id(&self, binding: &GoSourceNameBinding) -> Option<&str> {
        if !self.locals.contains(binding) {
            return None;
        }
        self.local_types
            .get(&(binding.declaration_node(), binding.name_node()))
            .map(String::as_str)
    }
    pub fn syntactically_terminating(&self) -> bool {
        self.terminating
    }
    /// Owned type/source/call dependencies; no runtime result values are emitted.
    pub fn typing(&self) -> &GoValueTyping {
        &self.typing
    }
}
enum Task {
    Visit(usize, Option<usize>, Option<usize>),
    Finish(usize),
}
struct Body<'a> {
    engine: Engine<'a>,
    source: &'a str,
    rows: BTreeMap<usize, GoBodyStatement>,
    expressions: Vec<GoBodyExpression>,
    branches: Vec<GoBodyBranch>,
    constants: Vec<GoBodyConstant>,
    declarations: BTreeSet<usize>,
    no_read: BTreeSet<usize>,
    breaks: BTreeSet<usize>,
}
impl Body<'_> {
    fn node(&self, n: usize) -> BodyResult<&GoSyntaxNode> {
        at(self.source, n, self.engine.node(self.source, n))
    }
    fn compute(&mut self, node: usize, r: Request) -> BodyResult<Facts> {
        at(self.source, node, self.engine.compute(r.clone()))?;
        at(self.source, node, self.engine.facts(&r))
    }
    fn one(&mut self, node: usize, expected: Option<String>) -> BodyResult<Atom> {
        let r = Request::value(self.source, node, expected);
        self.compute(node, r.clone())?;
        at(self.source, node, self.engine.one(&r))
    }
    fn record(&mut self, node: usize, role: &'static str, a: &Atom) -> BodyResult<()> {
        let typ = a.typ.clone().ok_or_else(|| {
            error(
                self.source,
                node,
                GoBodyErrorKind::Typing(GoScalarTypingError::Type),
            )
        })?;
        self.expressions.push(GoBodyExpression {
            node,
            role,
            typ,
            nil: a.nil,
        });
        Ok(())
    }
    fn condition(&mut self, node: usize) -> BodyResult<()> {
        let a = self.one(node, None)?;
        if a.kind != Some(GoConstantKind::Bool) {
            return Err(error(
                self.source,
                node,
                GoBodyErrorKind::Typing(GoScalarTypingError::Type),
            ));
        }
        self.record(node, "condition", &a)
    }
    fn write_only(&mut self, mut node: usize) -> BodyResult<()> {
        while self.node(node)?.kind() == "ParenExpr" {
            node = at(self.source, node, child(self.node(node)?, "X"))?;
        }
        if self.node(node)?.kind() == "Ident" {
            self.no_read.insert(node);
        }
        Ok(())
    }
    fn declaration(&mut self, node: usize) -> BodyResult<()> {
        let decl = at(self.source, node, child(self.node(node)?, "Decl"))?;
        let d = self.node(decl)?.clone();
        for &spec in list(&d, "Specs") {
            self.declarations.insert(spec);
            match attr(&d, "Tok") {
                Some("var") => {
                    self.compute(spec, Request::declaration(self.source, spec))?;
                }
                Some("const") => {
                    let values = at(
                        self.source,
                        spec,
                        evaluate_go_constant_declaration(self.engine.scopes, self.source, spec)
                            .map_err(constant_error),
                    )?;
                    for value in &values {
                        at(self.source, spec, self.engine.constant(value.clone()))?;
                    }
                    self.constants.push(GoBodyConstant { spec, values });
                }
                Some("type") => {
                    let s = self.node(spec)?.clone();
                    if !list(&s, "TypeParams").is_empty() {
                        return Err(error(self.source, spec, GoBodyErrorKind::Unsupported));
                    }
                    let name = at(self.source, spec, child(&s, "Name"))?;
                    let typ = if attr(self.node(name)?, "Name") == Some("_") {
                        at(self.source, spec, child(&s, "Type"))?
                    } else {
                        name
                    };
                    at(self.source, spec, self.engine.annotation(self.source, typ))?;
                }
                _ => return Err(error(self.source, decl, GoBodyErrorKind::Unsupported)),
            }
        }
        Ok(())
    }
    fn expression_statement(&mut self, node: usize) -> BodyResult<()> {
        let x = at(self.source, node, child(self.node(node)?, "X"))?;
        let mut call = x;
        while self.node(call)?.kind() == "ParenExpr" {
            call = at(self.source, call, child(self.node(call)?, "X"))?;
        }
        if self.node(call)?.kind() != "CallExpr" {
            return Err(error(self.source, x, GoBodyErrorKind::Unsupported));
        }
        let fun = at(self.source, call, child(self.node(call)?, "Fun"))?;
        let mut selector = fun;
        while self.node(selector)?.kind() == "ParenExpr" {
            selector = at(self.source, selector, child(self.node(selector)?, "X"))?;
        }
        let method = self.node(selector)?.kind() == "SelectorExpr"
            && !at(
                self.source,
                selector,
                self.engine.is_package_selector(self.source, selector),
            )?;
        let allowed = method
            || matches!(self.engine.symbol(self.source, fun), Ok(GoNameLookup::Original(b)) if b.kind() == GoSourceNameKind::Function)
            || matches!(self.engine.symbol(self.source, fun), Ok(GoNameLookup::Predeclared { name, kind: GoSourceNameKind::BuiltinFunction }) if name == "copy");
        if !allowed {
            return Err(error(self.source, x, GoBodyErrorKind::Unsupported));
        }
        self.compute(node, Request::raw(self.source, x))?;
        Ok(())
    }
    fn range(&mut self, node: usize, n: &GoSyntaxNode) -> BodyResult<()> {
        let x = at(self.source, node, child(n, "X"))?;
        let a = self.one(x, None)?;
        let shape = at(
            self.source,
            x,
            self.engine.shape(a.typ.as_deref().ok_or_else(|| {
                error(
                    self.source,
                    x,
                    GoBodyErrorKind::Typing(GoScalarTypingError::Type),
                )
            })?),
        )?
        .clone();
        let element = match shape {
            GoSourceTypeKey::Array { element, .. } | GoSourceTypeKey::Slice { element, .. } => {
                element
            }
            _ => return Err(error(self.source, x, GoBodyErrorKind::Unsupported)),
        };
        self.record(x, "range_source", &a)?;
        let int = at(self.source, node, self.engine.builtin("int"))?;
        self.declarations.insert(node);
        for (role, typ, name) in [("Key", int, "range_key"), ("Value", element, "range_value")] {
            for &target in list(n, role) {
                self.write_only(target)?;
                if at(
                    self.source,
                    target,
                    self.engine.blank_target(self.source, target),
                )? {
                    continue;
                }
                let a = if attr(n, "Tok") == Some(":=") {
                    let b = self
                        .engine
                        .scopes
                        .bindings()
                        .iter()
                        .find(|b| {
                            b.source_object_id() == self.source
                                && b.declaration_node() == node
                                && b.name_node() == target
                        })
                        .cloned()
                        .ok_or_else(|| {
                            error(
                                self.source,
                                target,
                                GoBodyErrorKind::Typing(GoScalarTypingError::Name),
                            )
                        })?;
                    let i = at(self.source, target, self.engine.binding_index(&b))?;
                    let r = Request::binding(&b, i);
                    self.compute(target, r.clone())?;
                    at(self.source, target, self.engine.one(&r))?
                } else {
                    if attr(n, "Tok") != Some("=") {
                        return Err(error(self.source, node, GoBodyErrorKind::Unsupported));
                    }
                    let r = Request::raw(self.source, target);
                    self.compute(target, r.clone())?;
                    at(
                        self.source,
                        target,
                        self.engine.assignment_targets(&[Some(r.clone())]),
                    )?;
                    at(self.source, target, self.engine.one(&r))?
                };
                if !a
                    .typ
                    .as_ref()
                    .is_some_and(|t| self.engine.assignable(&typ, t))
                {
                    return Err(error(
                        self.source,
                        target,
                        GoBodyErrorKind::Typing(GoScalarTypingError::Type),
                    ));
                }
                self.record(target, name, &a)?;
            }
        }
        Ok(())
    }
    fn switch(&mut self, node: usize, n: &GoSyntaxNode) -> BodyResult<()> {
        let tag = if let Some(&tag) = list(n, "Tag").first() {
            let a = self.one(tag, None)?;
            self.record(tag, "switch_tag", &a)?;
            a
        } else {
            let bool_type = at(self.source, node, self.engine.builtin("bool"))?;
            self.engine.atom(&bool_type, false)
        };
        let typ = tag
            .typ
            .as_ref()
            .ok_or_else(|| {
                error(
                    self.source,
                    node,
                    GoBodyErrorKind::Typing(GoScalarTypingError::Type),
                )
            })?
            .clone();
        if !at(self.source, node, self.engine.comparable(&typ))? && !self.engine.has_nil(&typ) {
            return Err(error(
                self.source,
                node,
                GoBodyErrorKind::Typing(GoScalarTypingError::Type),
            ));
        }
        let body = at(self.source, node, child(n, "Body"))?;
        let clauses = list(self.node(body)?, "List").to_vec();
        let mut seen = BTreeSet::new();
        let mut default = false;
        for clause in clauses {
            let c = self.node(clause)?.clone();
            if c.kind() != "CaseClause" {
                return Err(error(self.source, clause, GoBodyErrorKind::Unsupported));
            }
            if list(&c, "List").is_empty() {
                if default {
                    return Err(error(self.source, clause, GoBodyErrorKind::DuplicateCase));
                }
                default = true;
            }
            for &value in list(&c, "List") {
                let a = self.one(value, Some(typ.clone()))?;
                let mut compared = a.clone();
                if compared.nil {
                    compared.typ = None;
                }
                at(
                    self.source,
                    value,
                    self.engine.aggregate_comparison("==", &tag, &compared),
                )?;
                if let Some(v) = &a.value {
                    let key = (
                        a.typ.clone(),
                        v.integer_decimal(),
                        v.boolean(),
                        v.bytes().map(<[u8]>::to_vec),
                    );
                    if !seen.insert(key) {
                        return Err(error(self.source, value, GoBodyErrorKind::DuplicateCase));
                    }
                }
                self.record(value, "case_value", &a)?;
            }
        }
        Ok(())
    }
    fn visit(
        &mut self,
        node: usize,
        break_to: Option<usize>,
        continue_to: Option<usize>,
        tasks: &mut Vec<Task>,
    ) -> BodyResult<()> {
        let n = self.node(node)?.clone();
        let mut children = vec![];
        let mut types = vec![];
        match n.kind() {
            "EmptyStmt" => {}
            "BlockStmt" | "CaseClause" => {
                let role = if n.kind() == "BlockStmt" {
                    "List"
                } else {
                    "Body"
                };
                children.extend(list(&n, role).iter().map(|&n| (n, break_to, continue_to)));
            }
            "AssignStmt" | "IncDecStmt" => {
                self.declarations.insert(node);
                if matches!(attr(&n, "Tok"), Some("=" | ":=")) {
                    for &target in list(&n, "Lhs") {
                        self.write_only(target)?;
                    }
                }
                let facts = self.compute(
                    node,
                    Request {
                        source: self.source.into(),
                        target: Target::Assignment(node),
                    },
                )?;
                types.extend(facts.iter().filter_map(|a| a.typ.clone()));
            }
            "ReturnStmt" => {
                let facts = self.compute(
                    node,
                    Request {
                        source: self.source.into(),
                        target: Target::Return(node),
                    },
                )?;
                types.extend(facts.iter().filter_map(|a| a.typ.clone()));
            }
            "DeclStmt" => self.declaration(node)?,
            "ExprStmt" => self.expression_statement(node)?,
            "IfStmt" => {
                if let Some(&init) = list(&n, "Init").first() {
                    children.push((init, break_to, continue_to));
                }
                self.condition(at(self.source, node, child(&n, "Cond"))?)?;
                children.push((
                    at(self.source, node, child(&n, "Body"))?,
                    break_to,
                    continue_to,
                ));
                if let Some(&alternative) = list(&n, "Else").first() {
                    children.push((alternative, break_to, continue_to));
                }
            }
            "ForStmt" => {
                if let Some(&init) = list(&n, "Init").first() {
                    children.push((init, break_to, continue_to));
                }
                if let Some(&cond) = list(&n, "Cond").first() {
                    self.condition(cond)?;
                }
                if let Some(&post) = list(&n, "Post").first() {
                    if attr(self.node(post)?, "Tok") == Some(":=") {
                        return Err(error(self.source, post, GoBodyErrorKind::Unsupported));
                    }
                    children.push((post, break_to, continue_to));
                }
                children.push((
                    at(self.source, node, child(&n, "Body"))?,
                    Some(node),
                    Some(node),
                ));
            }
            "RangeStmt" => {
                self.range(node, &n)?;
                children.push((
                    at(self.source, node, child(&n, "Body"))?,
                    Some(node),
                    Some(node),
                ));
            }
            "SwitchStmt" => {
                if let Some(&init) = list(&n, "Init").first() {
                    children.push((init, break_to, continue_to));
                }
                self.switch(node, &n)?;
                children.push((
                    at(self.source, node, child(&n, "Body"))?,
                    Some(node),
                    continue_to,
                ));
            }
            "BranchStmt" => {
                if !list(&n, "Label").is_empty() {
                    return Err(error(self.source, node, GoBodyErrorKind::Unsupported));
                }
                let target = match attr(&n, "Tok") {
                    Some("break") => break_to,
                    Some("continue") => continue_to,
                    _ => return Err(error(self.source, node, GoBodyErrorKind::Unsupported)),
                }
                .ok_or_else(|| error(self.source, node, GoBodyErrorKind::Branch))?;
                if attr(&n, "Tok") == Some("break") {
                    self.breaks.insert(target);
                }
                self.branches.push(GoBodyBranch { node, target });
            }
            _ => return Err(error(self.source, node, GoBodyErrorKind::Unsupported)),
        }
        self.rows.insert(
            node,
            GoBodyStatement {
                node,
                types,
                terminating: false,
            },
        );
        tasks.push(Task::Finish(node));
        for (n, b, c) in children.into_iter().rev() {
            tasks.push(Task::Visit(n, b, c));
        }
        Ok(())
    }
    fn ends(&self, nodes: &[usize]) -> bool {
        nodes
            .iter()
            .rev()
            .find(|&&n| {
                self.engine
                    .node(self.source, n)
                    .is_ok_and(|n| n.kind() != "EmptyStmt")
            })
            .is_some_and(|n| self.rows.get(n).is_some_and(|r| r.terminating))
    }
    fn finish(&mut self, node: usize) -> BodyResult<()> {
        let n = self.node(node)?;
        let terminating = match n.kind() {
            "ReturnStmt" => true,
            "BlockStmt" => self.ends(list(n, "List")),
            "CaseClause" => self.ends(list(n, "Body")),
            "IfStmt" => {
                list(n, "Else")
                    .first()
                    .is_some_and(|n| self.rows[n].terminating)
                    && self.rows[&at(self.source, node, child(n, "Body"))?].terminating
            }
            "ForStmt" => list(n, "Cond").is_empty() && !self.breaks.contains(&node),
            "SwitchStmt" => {
                let clauses = list(self.node(at(self.source, node, child(n, "Body"))?)?, "List");
                !self.breaks.contains(&node)
                    && clauses.iter().any(|&n| {
                        self.engine
                            .node(self.source, n)
                            .is_ok_and(|n| list(n, "List").is_empty())
                    })
                    && clauses.iter().all(|n| self.rows[n].terminating)
            }
            _ => false,
        };
        self.rows
            .get_mut(&node)
            .ok_or_else(|| error(self.source, node, GoBodyErrorKind::Unsupported))?
            .terminating = terminating;
        Ok(())
    }
    fn locals(&self) -> BodyResult<Vec<GoSourceNameBinding>> {
        let mut read = BTreeSet::new();
        for request in self.engine.done.keys() {
            let Target::Expression(node, Mode::Raw) = request.target else {
                continue;
            };
            if request.source != self.source
                || self.no_read.contains(&node)
                || !self
                    .engine
                    .node(self.source, node)
                    .is_ok_and(|n| n.kind() == "Ident")
            {
                continue;
            }
            if let Ok(GoNameLookup::Original(b)) = self.engine.symbol(self.source, node) {
                if b.kind() == GoSourceNameKind::Variable {
                    read.insert((b.declaration_node(), b.name_node()));
                }
            }
        }
        let mut locals = vec![];
        for b in self.engine.scopes.bindings() {
            if b.source_object_id() == self.source
                && b.kind() == GoSourceNameKind::Variable
                && self.declarations.contains(&b.declaration_node())
            {
                if !read.contains(&(b.declaration_node(), b.name_node())) {
                    return Err(error(
                        self.source,
                        b.name_node(),
                        GoBodyErrorKind::UnusedVariable,
                    ));
                }
                locals.push(b.clone());
            }
        }
        locals.sort_by_key(|b| {
            self.engine
                .node(self.source, b.name_node())
                .map(|n| n.start())
                .unwrap_or(usize::MAX)
        });
        Ok(locals)
    }
}
/// Check all supported statements of one original function body, including
/// unreachable syntax, local uses, branches and Go's final-return rule.
/// Does not validate other functions/packages, select callees, admit a profile,
/// execute the body, or prove runtime safety/termination/state/program behavior.
pub fn check_go_function_body(
    scopes: &GoSourceScopes,
    source: &str,
    function: usize,
) -> BodyResult<GoBodyTyping> {
    let signature = derive_go_source_signature(scopes, source, function).map_err(|e| {
        error(
            source,
            function,
            GoBodyErrorKind::Typing(GoScalarTypingError::Signature(e)),
        )
    })?;
    let body = signature
        .body_node()
        .ok_or_else(|| error(source, function, GoBodyErrorKind::Unsupported))?;
    let mut state = Body {
        engine: Engine::new(scopes),
        source,
        rows: BTreeMap::new(),
        expressions: vec![],
        branches: vec![],
        constants: vec![],
        declarations: BTreeSet::new(),
        no_read: BTreeSet::new(),
        breaks: BTreeSet::new(),
    };
    if let Some(root) = signature.types().roots().first() {
        at(
            source,
            function,
            state.engine.graph_type(signature.types(), root),
        )?;
    }
    state
        .engine
        .return_signatures
        .insert((source.into(), function), signature.clone());
    let mut tasks = vec![Task::Visit(body, None, None)];
    while let Some(task) = tasks.pop() {
        match task {
            Task::Visit(n, b, c) => state.visit(n, b, c, &mut tasks)?,
            Task::Finish(n) => state.finish(n)?,
        }
    }
    let terminating = state.rows[&body].terminating;
    if signature
        .slots()
        .iter()
        .any(|s| s.role() == GoSourceSignatureRole::Result)
        && !terminating
    {
        return Err(error(source, body, GoBodyErrorKind::MissingReturn));
    }
    let locals = state.locals()?;
    let mut local_types = BTreeMap::new();
    for b in &locals {
        let i = at(source, b.name_node(), state.engine.binding_index(b))?;
        let r = Request::binding(b, i);
        state.compute(b.name_node(), r.clone())?;
        let value = at(source, b.name_node(), state.engine.one(&r))?;
        let typ = value.typ.ok_or_else(|| {
            error(
                source,
                b.name_node(),
                GoBodyErrorKind::Typing(GoScalarTypingError::Type),
            )
        })?;
        local_types.insert((b.declaration_node(), b.name_node()), typ);
    }
    let mut statements = state.rows.into_values().collect::<Vec<_>>();
    statements.sort_by_key(|s| {
        (
            scopes.source(source).unwrap().nodes()[s.node].start(),
            s.node,
        )
    });
    let typing = at(source, body, state.engine.into_typing(vec![]))?;
    Ok(GoBodyTyping {
        signature,
        statements,
        expressions: state.expressions,
        branches: state.branches,
        constants: state.constants,
        locals,
        local_types,
        terminating,
        typing,
    })
}
