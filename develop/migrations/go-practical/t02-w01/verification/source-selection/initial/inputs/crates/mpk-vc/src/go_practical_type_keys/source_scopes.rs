//! Independent lexical bindings over complete owned syntax, including blockers
//! such as variables shadowing type names. These are candidate-package scopes,
//! not type checking, import/module identity, or source/program authority.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceScopeError {
    Syntax,
    Duplicate,
    Import,
    Locator,
}
impl std::fmt::Display for GoSourceScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source scope error: {self:?}")
    }
}
impl std::error::Error for GoSourceScopeError {}
type Result<T> = std::result::Result<T, GoSourceScopeError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceNameKind {
    Type,
    TypeParameter,
    Constant,
    Variable,
    Function,
    Import,
    BuiltinType,
    BuiltinConstant,
    BuiltinFunction,
    BuiltinNil,
}
impl GoSourceNameKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::TypeParameter => "type_parameter",
            Self::Constant => "constant",
            Self::Variable => "variable",
            Self::Function => "function",
            Self::Import => "import",
            Self::BuiltinType => "builtin_type",
            Self::BuiltinConstant => "builtin_constant",
            Self::BuiltinFunction => "builtin_function",
            Self::BuiltinNil => "builtin_nil",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceNameBinding {
    name: String,
    kind: GoSourceNameKind,
    package_id: String,
    source_object_id: String,
    declaration_node: usize,
    name_node: usize,
    scope: usize,
    visible_from: usize,
    import_path: Option<String>,
}
impl GoSourceNameBinding {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn kind(&self) -> GoSourceNameKind {
        self.kind
    }
    pub fn package_id(&self) -> &str {
        &self.package_id
    }
    pub fn source_object_id(&self) -> &str {
        &self.source_object_id
    }
    pub fn declaration_node(&self) -> usize {
        self.declaration_node
    }
    pub fn name_node(&self) -> usize {
        self.name_node
    }
    pub fn visible_from(&self) -> usize {
        self.visible_from
    }
    pub fn import_path(&self) -> Option<&str> {
        self.import_path.as_deref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoNameLookup {
    Original(GoSourceNameBinding),
    Predeclared {
        name: String,
        kind: GoSourceNameKind,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Scope {
    parent: Option<usize>,
    file: bool,
    names: BTreeMap<String, usize>,
    dot_imports: Vec<String>,
    unresolved_default_import: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceScopes {
    syntax: BTreeMap<String, GoParsedSource>,
    scopes: Vec<Scope>,
    bindings: Vec<GoSourceNameBinding>,
    node_scopes: BTreeMap<(String, usize), usize>,
    package_scopes: BTreeMap<String, usize>,
    imports: BTreeMap<String, String>,
}
impl GoSourceScopes {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.syntax.get(id)
    }
    pub fn bindings(&self) -> &[GoSourceNameBinding] {
        &self.bindings
    }
    pub(super) fn sources(&self) -> impl Iterator<Item = (&str, &GoParsedSource)> {
        self.syntax.iter().map(|(id, source)| (id.as_str(), source))
    }
    pub(super) fn package_binding(&self,binding:&GoSourceNameBinding)->bool {
        self.package_scopes.get(binding.package_id()).is_some_and(|scope| *scope==binding.scope)
    }
    pub(super) fn source_package(&self, source: &str) -> Option<&str> {
        let parsed = self.syntax.get(source)?;
        let file = *self.node_scopes.get(&(source.to_string(), parsed.root()))?;
        let package = self.scopes[file].parent?;
        self.package_scopes
            .iter()
            .find_map(|(id, &scope)| (scope == package).then_some(id.as_str()))
    }
    pub(super) fn imported_member(
        &self,
        path: &str,
        name: &str,
    ) -> Result<Option<GoSourceNameBinding>> {
        if !name.chars().next().is_some_and(validate::exported) {
            return Ok(None);
        }
        let package = self.imports.get(path).ok_or(GoSourceScopeError::Import)?;
        Ok(self.scopes[self.package_scopes[package]]
            .names
            .get(name)
            .map(|&id| self.bindings[id].clone()))
    }
    /// Lookup in the lexical namespace at an actual original Ident. This does
    /// not assert that the syntax position is a lexical use: selector fields,
    /// labels, declaration names and typed composite keys need their own rules.
    pub fn lookup_identifier(&self, source: &str, node: usize) -> Result<Option<GoNameLookup>> {
        self.lookup_context(
            source,
            node,
            source_dependencies::context(self, source, node),
        )
    }
    // Implicitly repeated ConstSpecs reuse an original expression node but its
    // names are resolved at the repeated declaration's actual lexical point.
    pub(super) fn lookup_context(
        &self,
        source: &str,
        node: usize,
        context: Option<usize>,
    ) -> Result<Option<GoNameLookup>> {
        let parsed = self.syntax.get(source).ok_or(GoSourceScopeError::Locator)?;
        let ident = parsed
            .nodes()
            .get(node)
            .ok_or(GoSourceScopeError::Locator)?;
        if ident.kind() != "Ident" {
            return Err(GoSourceScopeError::Locator);
        }
        let name = attr(ident, "Name").ok_or(GoSourceScopeError::Syntax)?;
        let position = match context {
            Some(id) => parsed
                .nodes()
                .get(id)
                .ok_or(GoSourceScopeError::Locator)?
                .end()
                .saturating_sub(1),
            None => ident.start(),
        };
        let mut scope = *self
            .node_scopes
            .get(&(source.to_string(), context.unwrap_or(node)))
            .ok_or(GoSourceScopeError::Locator)?;
        loop {
            let current = &self.scopes[scope];
            if current.file
                && (current.unresolved_default_import
                    || name.chars().next().is_some_and(validate::exported)
                        && current
                            .dot_imports
                            .iter()
                            .any(|p| !self.imports.contains_key(p)))
            {
                return Err(GoSourceScopeError::Import);
            }
            if let Some(&id) = current.names.get(name) {
                let b = &self.bindings[id];
                // The receiver's original parameter Ident also denotes its
                // own declared parameter. Go records this binder in Uses;
                // do not widen visibility for other receiver occurrences.
                let own_parameter = b.kind == GoSourceNameKind::TypeParameter
                    && context.is_none()
                    && b.name_node == node
                    && b.source_object_id == source;
                if own_parameter || b.visible_from <= position || current.parent.is_none() {
                    return Ok(Some(GoNameLookup::Original(b.clone())));
                }
            }
            if current.file {
                let mut found = None;
                for path in &current.dot_imports {
                    let Some(package) = self.imports.get(path) else {
                        if name.chars().next().is_some_and(validate::exported) {
                            return Err(GoSourceScopeError::Import);
                        }
                        continue;
                    };
                    let imported = &self.scopes[self.package_scopes[package]];
                    if name.chars().next().is_some_and(validate::exported) {
                        if let Some(&id) = imported.names.get(name) {
                            if found.replace(id).is_some() {
                                return Err(GoSourceScopeError::Duplicate);
                            }
                        }
                    }
                }
                if let Some(id) = found {
                    return Ok(Some(GoNameLookup::Original(self.bindings[id].clone())));
                }
                // A default import's declared name cannot be inferred from the
                // path spelling. Missing metadata must not expose an outer name.
                if current.unresolved_default_import {
                    return Err(GoSourceScopeError::Import);
                }
            }
            match current.parent {
                Some(parent) => scope = parent,
                None => break,
            }
        }
        Ok(predeclared(name).map(|kind| GoNameLookup::Predeclared {
            name: name.into(),
            kind,
        }))
    }
}
fn predeclared(name: &str) -> Option<GoSourceNameKind> {
    use GoSourceNameKind::*;
    Some(match name {
        "any" | "bool" | "byte" | "comparable" | "complex64" | "complex128" | "error"
        | "float32" | "float64" | "int" | "int8" | "int16" | "int32" | "int64" | "rune"
        | "string" | "uint" | "uint8" | "uint16" | "uint32" | "uint64" | "uintptr" => BuiltinType,
        "true" | "false" | "iota" => BuiltinConstant,
        "append" | "cap" | "clear" | "close" | "complex" | "copy" | "delete" | "imag" | "len"
        | "make" | "max" | "min" | "new" | "panic" | "print" | "println" | "real" | "recover" => {
            BuiltinFunction
        }
        "nil" => BuiltinNil,
        _ => return None,
    })
}
fn attr<'a>(node: &'a GoSyntaxNode, key: &str) -> Option<&'a str> {
    node.attributes().find_map(|(k, v)| (k == key).then_some(v))
}
fn list<'a>(node: &'a GoSyntaxNode, role: &str) -> &'a [usize] {
    node.edges()
        .iter()
        .find(|e| e.role() == role)
        .map_or(&[], |e| e.targets())
}
fn child(node: &GoSyntaxNode, role: &str) -> Option<usize> {
    list(node, role).first().copied()
}
struct Builder<'a> {
    graph: GoSourceScopes,
    package_names: BTreeMap<&'a str, &'a str>,
}
impl Builder<'_> {
    fn scope(&mut self, parent: Option<usize>, file: bool) -> usize {
        let id = self.graph.scopes.len();
        self.graph.scopes.push(Scope {
            parent,
            file,
            names: BTreeMap::new(),
            dot_imports: vec![],
            unresolved_default_import: false,
        });
        id
    }
    fn binding(
        &mut self,
        parsed: &GoParsedSource,
        package: &str,
        at: (usize, usize, usize),
        kind: GoSourceNameKind,
        visible: usize,
        import_path: Option<String>,
    ) -> Result<()> {
        let (scope, decl, name) = at;
        let syntax = &parsed.nodes()[name];
        let spelling = attr(syntax, "Name").ok_or(GoSourceScopeError::Syntax)?;
        if spelling == "_" {
            return Ok(());
        }
        if self.graph.scopes[scope].names.contains_key(spelling) {
            return Err(GoSourceScopeError::Duplicate);
        }
        let id = self.graph.bindings.len();
        self.graph.scopes[scope].names.insert(spelling.into(), id);
        self.graph.bindings.push(GoSourceNameBinding {
            name: spelling.into(),
            kind,
            package_id: package.into(),
            source_object_id: parsed.source().object().id.clone(),
            declaration_node: decl,
            name_node: name,
            scope,
            visible_from: visible,
            import_path,
        });
        Ok(())
    }
    fn fields(
        &mut self,
        parsed: &GoParsedSource,
        package: &str,
        at: (usize, usize),
        field_list: Option<usize>,
        kind: GoSourceNameKind,
        visible: usize,
    ) -> Result<()> {
        let (scope, decl) = at;
        if let Some(field_list) = field_list {
            for &field in list(&parsed.nodes()[field_list], "List") {
                for &name in list(&parsed.nodes()[field], "Names") {
                    self.binding(parsed, package, (scope, decl, name), kind, visible, None)?;
                }
            }
        }
        Ok(())
    }
    fn short(
        &mut self,
        parsed: &GoParsedSource,
        package: &str,
        scope: usize,
        decl: usize,
        names: &[usize],
        visible: usize,
    ) -> Result<()> {
        let mut seen = BTreeSet::new();
        let mut fresh = false;
        for &id in names {
            let n = &parsed.nodes()[id];
            let name = attr(n, "Name")
                .filter(|_| n.kind() == "Ident")
                .ok_or(GoSourceScopeError::Syntax)?;
            if name == "_" {
                continue;
            }
            if !seen.insert(name) {
                return Err(GoSourceScopeError::Duplicate);
            }
            if let Some(&old) = self.graph.scopes[scope].names.get(name) {
                let old = &self.graph.bindings[old];
                if old.kind != GoSourceNameKind::Variable || old.visible_from > n.start() {
                    return Err(GoSourceScopeError::Duplicate);
                }
            } else {
                self.binding(
                    parsed,
                    package,
                    (scope, decl, id),
                    GoSourceNameKind::Variable,
                    visible,
                    None,
                )?;
                fresh = true;
            }
        }
        if !fresh {
            return Err(GoSourceScopeError::Duplicate);
        }
        Ok(())
    }
    fn walk(&mut self, parsed: &GoParsedSource, package: &str, file_scope: usize) -> Result<()> {
        // (syntax node, lexical scope, optional declaration keyword,
        //  scope was allocated by the function handler)
        let mut tasks = vec![(parsed.root(), file_scope, None, false)];
        let mut seen = BTreeSet::new();
        while let Some((id, mut scope, keyword, reuse)) = tasks.pop() {
            if !seen.insert((id, scope)) {
                continue;
            }
            let node = &parsed.nodes()[id];
            if !reuse
                && matches!(
                    node.kind(),
                    "BlockStmt"
                        | "IfStmt"
                        | "ForStmt"
                        | "RangeStmt"
                        | "SwitchStmt"
                        | "TypeSwitchStmt"
                        | "CaseClause"
                        | "CommClause"
                )
            {
                scope = self.scope(Some(scope), false);
            }
            let key = (parsed.source().object().id.clone(), id);
            if self
                .graph
                .node_scopes
                .insert(key, scope)
                .is_some_and(|old| old != scope)
            {
                return Err(GoSourceScopeError::Syntax);
            }
            let declaration_scope = if self.graph.scopes[scope].file {
                self.graph.scopes[scope].parent.unwrap()
            } else {
                scope
            };
            let visible = if self.graph.scopes[declaration_scope].parent.is_none() {
                0
            } else {
                node.end()
            };
            match node.kind() {
                "GenDecl" => {
                    let keyword = attr(node, "Tok").ok_or(GoSourceScopeError::Syntax)?;
                    for &spec in list(node, "Specs").iter().rev() {
                        tasks.push((spec, scope, Some(keyword), false))
                    }
                    continue;
                }
                "TypeSpec" => {
                    let name = child(node, "Name").ok_or(GoSourceScopeError::Syntax)?;
                    self.binding(
                        parsed,
                        package,
                        (declaration_scope, id, name),
                        GoSourceNameKind::Type,
                        if visible == 0 {
                            0
                        } else {
                            parsed.nodes()[name].start()
                        },
                        None,
                    )?;
                    if let Some(params) = child(node, "TypeParams") {
                        let inner = self.scope(Some(scope), false);
                        self.fields(
                            parsed,
                            package,
                            (inner, id),
                            Some(params),
                            GoSourceNameKind::TypeParameter,
                            parsed.nodes()[name].end(),
                        )?;
                        for edge in node.edges().iter().rev() {
                            for &n in edge.targets().iter().rev() {
                                tasks.push((
                                    n,
                                    if edge.role() == "Name" { scope } else { inner },
                                    None,
                                    false,
                                ))
                            }
                        }
                        continue;
                    }
                }
                "ValueSpec" => {
                    let kind = match keyword {
                        Some("var") => GoSourceNameKind::Variable,
                        Some("const") => GoSourceNameKind::Constant,
                        _ => return Err(GoSourceScopeError::Syntax),
                    };
                    for &name in list(node, "Names") {
                        self.binding(
                            parsed,
                            package,
                            (declaration_scope, id, name),
                            kind,
                            visible,
                            None,
                        )?
                    }
                }
                "FuncDecl" | "FuncLit" => {
                    let typ = child(node, "Type").ok_or(GoSourceScopeError::Syntax)?;
                    let signature = &parsed.nodes()[typ];
                    let body = child(node, "Body");
                    let name = child(node, "Name");
                    if node.kind() == "FuncDecl" && child(node, "Recv").is_none() {
                        let name = name.ok_or(GoSourceScopeError::Syntax)?;
                        if attr(&parsed.nodes()[name], "Name") != Some("init") {
                            self.binding(
                                parsed,
                                package,
                                (declaration_scope, id, name),
                                GoSourceNameKind::Function,
                                0,
                                None,
                            )?;
                        }
                    }
                    let generic = self.scope(Some(scope), false);
                    let begin = name.map_or(signature.start(), |n| parsed.nodes()[n].end());
                    self.fields(
                        parsed,
                        package,
                        (generic, id),
                        child(signature, "TypeParams"),
                        GoSourceNameKind::TypeParameter,
                        begin,
                    )?;
                    // Receiver type parameters are declared by the receiver's
                    // actual index arguments, not by the generic type's names.
                    if let Some(receiver) = child(node, "Recv") {
                        for &field in list(&parsed.nodes()[receiver], "List") {
                            let mut rt = child(&parsed.nodes()[field], "Type")
                                .ok_or(GoSourceScopeError::Syntax)?;
                            while parsed.nodes()[rt].kind() == "StarExpr" {
                                rt = child(&parsed.nodes()[rt], "X")
                                    .ok_or(GoSourceScopeError::Syntax)?
                            }
                            let r = &parsed.nodes()[rt];
                            let params = if r.kind() == "IndexExpr" {
                                list(r, "Index")
                            } else if r.kind() == "IndexListExpr" {
                                list(r, "Indices")
                            } else {
                                &[]
                            };
                            for &p in params {
                                self.binding(
                                    parsed,
                                    package,
                                    (generic, id, p),
                                    GoSourceNameKind::TypeParameter,
                                    begin,
                                    None,
                                )?
                            }
                        }
                        tasks.push((receiver, generic, None, false));
                    }
                    tasks.push((typ, generic, None, false));
                    if let Some(body) = body {
                        let bs = self.scope(Some(generic), false);
                        let begin = parsed.nodes()[body].start();
                        for fields in [
                            child(signature, "Params"),
                            child(signature, "Results"),
                            child(node, "Recv"),
                        ] {
                            self.fields(
                                parsed,
                                package,
                                (bs, id),
                                fields,
                                GoSourceNameKind::Variable,
                                begin,
                            )?;
                        }
                        tasks.push((body, bs, None, true));
                    }
                    if let Some(name) = name {
                        tasks.push((name, scope, None, false))
                    }
                    continue;
                }
                "ImportSpec" => {
                    let path = child(node, "Path").ok_or(GoSourceScopeError::Syntax)?;
                    let import_path =
                        unquote(parsed.spelling(path).ok_or(GoSourceScopeError::Syntax)?)?;
                    let import_path =
                        String::from_utf8(import_path).map_err(|_| GoSourceScopeError::Import)?;
                    let name = child(node, "Name");
                    match name.and_then(|n| attr(&parsed.nodes()[n], "Name")) {
                        Some("_") => (),
                        Some(".") => self.graph.scopes[scope].dot_imports.push(import_path),
                        Some(_) => self.binding(
                            parsed,
                            package,
                            (scope, id, name.unwrap()),
                            GoSourceNameKind::Import,
                            0,
                            Some(import_path),
                        )?,
                        None => {
                            if let Some(&actual) = self.package_names.get(import_path.as_str()) {
                                let binding = GoSourceNameBinding {
                                    name: actual.into(),
                                    kind: GoSourceNameKind::Import,
                                    package_id: package.into(),
                                    source_object_id: parsed.source().object().id.clone(),
                                    declaration_node: id,
                                    name_node: path,
                                    scope,
                                    visible_from: 0,
                                    import_path: Some(import_path),
                                };
                                if self.graph.scopes[scope]
                                    .names
                                    .insert(actual.into(), self.graph.bindings.len())
                                    .is_some()
                                {
                                    return Err(GoSourceScopeError::Duplicate);
                                }
                                self.graph.bindings.push(binding);
                            } else {
                                self.graph.scopes[scope].unresolved_default_import = true
                            }
                        }
                    }
                }
                "AssignStmt" if attr(node, "Tok") == Some(":=") => {
                    self.short(parsed, package, scope, id, list(node, "Lhs"), node.end())?
                }
                "RangeStmt" if attr(node, "Tok") == Some(":=") => {
                    let names: Vec<_> = [child(node, "Key"), child(node, "Value")]
                        .into_iter()
                        .flatten()
                        .collect();
                    let begin = child(node, "Body")
                        .map(|b| parsed.nodes()[b].start())
                        .ok_or(GoSourceScopeError::Syntax)?;
                    self.short(parsed, package, scope, id, &names, begin)?;
                }
                "TypeSwitchStmt" => {
                    let assign = child(node, "Assign").ok_or(GoSourceScopeError::Syntax)?;
                    let assign_node = &parsed.nodes()[assign];
                    if assign_node.kind() == "AssignStmt" && attr(assign_node, "Tok") == Some(":=")
                    {
                        let body = child(node, "Body").ok_or(GoSourceScopeError::Syntax)?;
                        self.graph
                            .node_scopes
                            .insert((parsed.source().object().id.clone(), body), scope);
                        self.graph
                            .node_scopes
                            .insert((parsed.source().object().id.clone(), assign), scope);
                        for &clause in list(&parsed.nodes()[body], "List").iter().rev() {
                            let cs = self.scope(Some(scope), false);
                            let c = &parsed.nodes()[clause];
                            let begin = list(c, "Body")
                                .first()
                                .map_or(c.end(), |&n| parsed.nodes()[n].start());
                            for &name in list(assign_node, "Lhs") {
                                self.binding(
                                    parsed,
                                    package,
                                    (cs, assign, name),
                                    GoSourceNameKind::Variable,
                                    begin,
                                    None,
                                )?
                            }
                            tasks.push((clause, cs, None, true));
                        }
                        for edge in node.edges().iter().rev() {
                            if edge.role() == "Body" || edge.role() == "Assign" {
                                continue;
                            }
                            for &n in edge.targets().iter().rev() {
                                tasks.push((n, scope, None, false))
                            }
                        }
                        for edge in assign_node.edges().iter().rev() {
                            for &n in edge.targets().iter().rev() {
                                tasks.push((n, scope, None, false))
                            }
                        }
                        continue;
                    }
                }
                _ => (),
            }
            for edge in node.edges().iter().rev() {
                for &n in edge.targets().iter().rev() {
                    tasks.push((n, scope, None, false))
                }
            }
        }
        Ok(())
    }
}

/// Construct lexical candidate scopes from private owned syntax. An absent
/// import's package name is never guessed from the last path component.
pub fn build_go_source_scopes(keys: &GoTypeKeyComponents) -> Result<GoSourceScopes> {
    let mut b = Builder {
        graph: GoSourceScopes {
            syntax: BTreeMap::new(),
            scopes: vec![],
            bindings: vec![],
            node_scopes: BTreeMap::new(),
            package_scopes: BTreeMap::new(),
            imports: BTreeMap::new(),
        },
        package_names: keys
            .wire
            .packages
            .iter()
            .map(|p| (p.import_path.as_str(), p.package_name.as_str()))
            .collect(),
    };
    for package in &keys.wire.packages {
        let scope = b.scope(None, false);
        b.graph.package_scopes.insert(package.id.clone(), scope);
        b.graph
            .imports
            .insert(package.import_path.clone(), package.id.clone());
    }
    for package in &keys.wire.packages {
        for source in &package.source_objects {
            if b.graph.syntax.contains_key(source) {
                return Err(GoSourceScopeError::Duplicate);
            }
            let parsed = keys
                .source_declarations()
                .source(source)
                .ok_or(GoSourceScopeError::Syntax)?;
            let file = b.scope(Some(b.graph.package_scopes[&package.id]), true);
            b.walk(parsed, &package.id, file)?;
            b.graph.syntax.insert(source.clone(), parsed.clone());
        }
    }
    for scope in &b.graph.scopes {
        if !scope.file {
            continue;
        }
        let package = &b.graph.scopes[scope.parent.unwrap()];
        let mut names: BTreeSet<_> = scope.names.keys().map(String::as_str).collect();
        for path in &scope.dot_imports {
            if let Some(imported) = b.graph.imports.get(path) {
                for name in b.graph.scopes[b.graph.package_scopes[imported]]
                    .names
                    .keys()
                {
                    if name.chars().next().is_some_and(validate::exported) && !names.insert(name) {
                        return Err(GoSourceScopeError::Duplicate);
                    }
                }
            }
        }
        if names.iter().any(|n| package.names.contains_key(*n)) {
            return Err(GoSourceScopeError::Duplicate);
        }
    }
    Ok(b.graph)
}

// The lexer has already validated this literal. Decode exact bytes here rather
// than JSON-decoding Go escapes or normalizing import path spelling.
fn unquote(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() < 2 {
        return Err(GoSourceScopeError::Syntax);
    }
    if raw[0] == b'`' {
        return Ok(raw[1..raw.len() - 1]
            .iter()
            .copied()
            .filter(|&b| b != b'\r')
            .collect());
    }
    if raw[0] != b'"' {
        return Err(GoSourceScopeError::Syntax);
    }
    let mut out = vec![];
    let mut at = 1;
    while at < raw.len() - 1 {
        let byte = raw[at];
        at += 1;
        if byte != b'\\' {
            out.push(byte);
            continue;
        }
        let esc = *raw.get(at).ok_or(GoSourceScopeError::Syntax)?;
        at += 1;
        let direct = match esc {
            b'a' => Some(7),
            b'b' => Some(8),
            b'f' => Some(12),
            b'n' => Some(10),
            b'r' => Some(13),
            b't' => Some(9),
            b'v' => Some(11),
            b'\\' => Some(b'\\'),
            b'"' => Some(b'"'),
            _ => None,
        };
        if let Some(byte) = direct {
            out.push(byte);
            continue;
        }
        let (base, count, mut value, unicode) = match esc {
            b'0'..=b'7' => (8, 2, (esc - b'0') as u32, false),
            b'x' => (16, 2, 0, false),
            b'u' => (16, 4, 0, true),
            b'U' => (16, 8, 0, true),
            _ => return Err(GoSourceScopeError::Syntax),
        };
        for _ in 0..count {
            let c = *raw.get(at).ok_or(GoSourceScopeError::Syntax)? as char;
            at += 1;
            value = value
                .checked_mul(base)
                .and_then(|n| n.checked_add(c.to_digit(base)?))
                .ok_or(GoSourceScopeError::Syntax)?
        }
        if unicode {
            let c = char::from_u32(value).ok_or(GoSourceScopeError::Syntax)?;
            let mut buf = [0; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes())
        } else {
            out.push(u8::try_from(value).map_err(|_| GoSourceScopeError::Syntax)?)
        }
    }
    Ok(out)
}
