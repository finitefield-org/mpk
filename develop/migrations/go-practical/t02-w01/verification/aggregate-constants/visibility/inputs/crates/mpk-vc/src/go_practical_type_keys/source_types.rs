//! Derive private v2 value-type keys from original type syntax and exact source
//! constants. This supplies no complete module/capture/context/program proof.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceTypeError {
    Locator,
    Scope,
    NotType,
    Unsupported,
    Constant,
    Cycle,
    Limit,
    Mismatch,
}
impl std::fmt::Display for GoSourceTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source type error: {self:?}")
    }
}
impl std::error::Error for GoSourceTypeError {}
type Result<T> = std::result::Result<T, GoSourceTypeError>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoDerivedSourceTypes {
    scopes: std::sync::Arc<GoSourceScopes>,
    pub(super) compilation_reads: source_dependencies::Reads,
    roots: Vec<String>,
    declarations: BTreeMap<String, GoTypeDeclarationKey>,
    types: BTreeMap<String, GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
    aliases: BTreeMap<String, String>,
}
impl GoDerivedSourceTypes {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.scopes.source(id)
    }
    pub fn roots(&self) -> &[String] {
        &self.roots
    }
    pub fn types(&self) -> impl Iterator<Item = &GoSourceTypeKey> {
        self.types.values()
    }
    pub fn declarations(&self) -> impl Iterator<Item = &GoTypeDeclarationKey> {
        self.declarations.values()
    }
    pub fn underlying(&self, id: &str) -> Option<&str> {
        self.underlying.get(id).map(String::as_str)
    }
    pub fn alias_target(&self, id: &str) -> Option<&str> {
        self.aliases.get(id).map(String::as_str)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Target {
    Expression(usize),
    Declaration(usize),
    VariadicBinding(usize),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Request {
    source: String,
    target: Target,
}
impl Request {
    fn expr(source: &str, node: usize) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node),
        }
    }
    fn decl(source: &str, node: usize) -> Self {
        Self {
            source: source.into(),
            target: Target::Declaration(node),
        }
    }
}
#[derive(Clone)]
struct Field {
    name: String,
    package: Option<String>,
    tag: String,
    embedded: bool,
    typ: Request,
}
enum Plan {
    Forward(Request),
    Array(u64, Request),
    Slice(Request),
    Struct(Vec<Field>),
    Named(String, Request),
    Alias(String, Request),
}
impl Plan {
    fn inputs(&self) -> Vec<Request> {
        match self {
            Self::Forward(r)
            | Self::Array(_, r)
            | Self::Slice(r)
            | Self::Named(_, r)
            | Self::Alias(_, r) => vec![r.clone()],
            Self::Struct(fields) => fields.iter().map(|f| f.typ.clone()).collect(),
        }
    }
}
enum Work {
    Visit(Request),
    Finish(Request, Plan),
}
struct Builder<'a> {
    scopes: &'a GoSourceScopes,
    declarations: BTreeMap<String, GoTypeDeclarationKey>,
    types: BTreeMap<String, GoSourceTypeKey>,
    raw_underlying: BTreeMap<String, String>,
    aliases: BTreeMap<String, String>,
    done: BTreeMap<Request, String>,
    active: BTreeSet<Request>,
}
impl<'a> Builder<'a> {
    fn new(scopes: &'a GoSourceScopes) -> Self {
        Self {
            scopes,
            declarations: BTreeMap::new(),
            types: BTreeMap::new(),
            raw_underlying: BTreeMap::new(),
            aliases: BTreeMap::new(),
            done: BTreeMap::new(),
            active: BTreeSet::new(),
        }
    }
    fn node(&self, source: &str, id: usize) -> Result<&GoSyntaxNode> {
        self.scopes
            .source(source)
            .and_then(|s| s.nodes().get(id))
            .ok_or(GoSourceTypeError::Locator)
    }
    fn intern(&mut self, mut key: GoSourceTypeKey) -> Result<String> {
        let id = identity(&key, TYPE_DOMAIN, "id").map_err(|_| GoSourceTypeError::Mismatch)?;
        match &mut key {
            GoSourceTypeKey::Builtin { id: field, .. }
            | GoSourceTypeKey::Nominal { id: field, .. }
            | GoSourceTypeKey::Array { id: field, .. }
            | GoSourceTypeKey::Slice { id: field, .. }
            | GoSourceTypeKey::Struct { id: field, .. } => *field = id.clone(),
        }
        if self.types.len() >= 4_194_304 && !self.types.contains_key(&id) {
            return Err(GoSourceTypeError::Limit);
        }
        self.types.insert(id.clone(), key);
        Ok(id)
    }
    fn builtin(&mut self, name: &str) -> Result<String> {
        let name = match name {
            "byte" => "uint8",
            "rune" => "int32",
            n => n,
        };
        let (width, signed) = match name {
            "bool" | "string" | "error" => (None, None),
            "int8" => (Some(8), Some(true)),
            "int16" => (Some(16), Some(true)),
            "int32" => (Some(32), Some(true)),
            "int64" | "int" => (Some(64), Some(true)),
            "uint8" => (Some(8), Some(false)),
            "uint16" => (Some(16), Some(false)),
            "uint32" => (Some(32), Some(false)),
            "uint64" | "uint" => (Some(64), Some(false)),
            _ => return Err(GoSourceTypeError::Unsupported),
        };
        self.intern(GoSourceTypeKey::Builtin {
            schema: TYPE_SCHEMA.into(),
            id: String::new(),
            name: name.into(),
            width,
            signed,
        })
    }
    fn symbol(&self, source: &str, node: usize) -> Result<GoNameLookup> {
        let n = self.node(source, node)?;
        match n.kind() {
            "Ident" => self
                .scopes
                .lookup_identifier(source, node)
                .map_err(|_| GoSourceTypeError::Scope)?
                .ok_or(GoSourceTypeError::NotType),
            "SelectorExpr" => {
                let base = child(n, "X")?;
                if self.node(source, base)?.kind() != "Ident" {
                    return Err(GoSourceTypeError::NotType);
                }
                let Some(GoNameLookup::Original(package)) = self
                    .scopes
                    .lookup_identifier(source, base)
                    .map_err(|_| GoSourceTypeError::Scope)?
                else {
                    return Err(GoSourceTypeError::NotType);
                };
                if package.kind() != GoSourceNameKind::Import {
                    return Err(GoSourceTypeError::NotType);
                }
                let name = attr(self.node(source, child(n, "Sel")?)?, "Name")
                    .ok_or(GoSourceTypeError::NotType)?;
                self.scopes
                    .imported_member(package.import_path().ok_or(GoSourceTypeError::Scope)?, name)
                    .map_err(|_| GoSourceTypeError::Scope)?
                    .map(GoNameLookup::Original)
                    .ok_or(GoSourceTypeError::NotType)
            }
            _ => Err(GoSourceTypeError::NotType),
        }
    }
    fn declaration(&mut self, source: &str, spec: usize) -> Result<GoTypeDeclarationKey> {
        let node = self.node(source, spec)?;
        if node.kind() != "TypeSpec" {
            return Err(GoSourceTypeError::NotType);
        }
        if !list(node, "TypeParams").is_empty() {
            return Err(GoSourceTypeError::Unsupported);
        }
        let binding = self
            .scopes
            .bindings()
            .iter()
            .find(|b| {
                b.kind() == GoSourceNameKind::Type
                    && b.source_object_id() == source
                    && b.declaration_node() == spec
            })
            .ok_or(GoSourceTypeError::NotType)?;
        let mut decl = GoTypeDeclarationKey {
            schema: DECL_SCHEMA.into(),
            id: String::new(),
            package_id: binding.package_id().into(),
            source_object_id: source.into(),
            kind: if attr(node, "Alias") == Some("true") {
                "alias_type"
            } else {
                "type"
            }
            .into(),
            start_byte: node.start() as u64,
            end_byte: node.end() as u64,
            name: binding.name().into(),
        };
        decl.id = identity(&decl, DECL_DOMAIN, "id").map_err(|_| GoSourceTypeError::Mismatch)?;
        if self.declarations.len() >= 4_194_304 && !self.declarations.contains_key(&decl.id) {
            return Err(GoSourceTypeError::Limit);
        }
        self.declarations.insert(decl.id.clone(), decl.clone());
        Ok(decl)
    }
    fn plan(&mut self, r: &Request) -> Result<std::result::Result<String, Plan>> {
        let node = match r.target {
            Target::Expression(n) => n,
            Target::VariadicBinding(n) => {
                let typ = self.node(&r.source, n)?;
                if typ.kind() != "Ellipsis" {
                    return Err(GoSourceTypeError::NotType);
                }
                return Ok(Err(Plan::Slice(Request::expr(
                    &r.source,
                    child(typ, "Elt")?,
                ))));
            }
            Target::Declaration(spec) => {
                let decl = self.declaration(&r.source, spec)?;
                let rhs = child(self.node(&r.source, spec)?, "Type")?;
                let rhs = Request::expr(&r.source, rhs);
                if decl.kind == "alias_type" {
                    return Ok(Err(Plan::Alias(decl.id, rhs)));
                }
                let id = self.intern(GoSourceTypeKey::Nominal {
                    schema: TYPE_SCHEMA.into(),
                    id: String::new(),
                    declaration_id: decl.id,
                })?;
                // Nominal identity excludes the underlying edge. Retain that
                // real edge separately, then reject any expanded recursion.
                self.done.insert(r.clone(), id.clone());
                return Ok(Err(Plan::Named(id, rhs)));
            }
        };
        let n = self.node(&r.source, node)?;
        match n.kind() {
            "Ident" | "SelectorExpr" => match self.symbol(&r.source, node)? {
                GoNameLookup::Predeclared {
                    name,
                    kind: GoSourceNameKind::BuiltinType,
                } => Ok(Ok(self.builtin(&name)?)),
                GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Type => Ok(Err(
                    Plan::Forward(Request::decl(b.source_object_id(), b.declaration_node())),
                )),
                GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::TypeParameter => {
                    Err(GoSourceTypeError::Unsupported)
                }
                _ => Err(GoSourceTypeError::NotType),
            },
            "ParenExpr" => Ok(Err(Plan::Forward(Request::expr(&r.source, child(n, "X")?)))),
            "ArrayType" => {
                let element = Request::expr(&r.source, child(n, "Elt")?);
                if let Some(&length) = list(n, "Len").first() {
                    let value = evaluate_go_constant(self.scopes, &r.source, length)
                        .map_err(|_| GoSourceTypeError::Constant)?;
                    if !matches!(value.kind(), GoConstantKind::Integer | GoConstantKind::Rune) {
                        return Err(GoSourceTypeError::Constant);
                    }
                    let length = value
                        .integer_decimal()
                        .ok_or(GoSourceTypeError::Constant)?
                        .parse::<u64>()
                        .map_err(|_| GoSourceTypeError::Limit)?;
                    if length > 256 {
                        return Err(GoSourceTypeError::Limit);
                    }
                    Ok(Err(Plan::Array(length, element)))
                } else {
                    Ok(Err(Plan::Slice(element)))
                }
            }
            "StructType" => {
                let package = self
                    .scopes
                    .source_package(&r.source)
                    .ok_or(GoSourceTypeError::Scope)?
                    .to_string();
                let fields = child(n, "Fields")?;
                let mut result = vec![];
                let mut seen = BTreeSet::new();
                for &field in list(self.node(&r.source, fields)?, "List") {
                    let f = self.node(&r.source, field)?;
                    let typ = child(f, "Type")?;
                    let tag = if let Some(&tag) = list(f, "Tag").first() {
                        let raw = self
                            .scopes
                            .source(&r.source)
                            .and_then(|s| s.spelling(tag))
                            .ok_or(GoSourceTypeError::Locator)?;
                        source_constants::quoted(raw)
                            .map_err(|_| GoSourceTypeError::NotType)?
                            .0
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect()
                    } else {
                        String::new()
                    };
                    let names = list(f, "Names");
                    let embedded = names.is_empty();
                    let names = if embedded {
                        let t = self.node(&r.source, typ)?;
                        vec![match t.kind() {
                            "Ident" => typ,
                            "SelectorExpr" => child(t, "Sel")?,
                            _ => return Err(GoSourceTypeError::Unsupported),
                        }]
                    } else {
                        names.to_vec()
                    };
                    for name in names {
                        let name = attr(self.node(&r.source, name)?, "Name")
                            .ok_or(GoSourceTypeError::NotType)?
                            .to_string();
                        if name != "_" && !seen.insert(name.clone()) || embedded && name == "_" {
                            return Err(GoSourceTypeError::NotType);
                        }
                        let unexported = !name.chars().next().is_some_and(validate::exported);
                        result.push(Field {
                            name,
                            package: unexported.then(|| package.clone()),
                            tag: tag.clone(),
                            embedded,
                            typ: Request::expr(&r.source, typ),
                        });
                        if result.len() > 64 {
                            return Err(GoSourceTypeError::Limit);
                        }
                    }
                }
                Ok(Err(Plan::Struct(result)))
            }
            // Only the actual predeclared error (or its aliases) is in v2;
            // structurally similar named/anonymous interfaces are not that type.
            _ => Err(GoSourceTypeError::Unsupported),
        }
    }
    fn finish(&mut self, plan: Plan) -> Result<String> {
        let value = |r: &Request| self.done.get(r).cloned().ok_or(GoSourceTypeError::Mismatch);
        match plan {
            Plan::Forward(r) => value(&r),
            Plan::Named(id, r) => {
                let rhs = value(&r)?;
                self.raw_underlying.insert(id.clone(), rhs);
                Ok(id)
            }
            Plan::Alias(id, r) => {
                let rhs = value(&r)?;
                self.aliases.insert(id, rhs.clone());
                Ok(rhs)
            }
            Plan::Array(length, r) => {
                let element = value(&r)?;
                self.intern(GoSourceTypeKey::Array {
                    schema: TYPE_SCHEMA.into(),
                    id: String::new(),
                    length,
                    element,
                })
            }
            Plan::Slice(r) => {
                let element = value(&r)?;
                self.intern(GoSourceTypeKey::Slice {
                    schema: TYPE_SCHEMA.into(),
                    id: String::new(),
                    element,
                })
            }
            Plan::Struct(fields) => {
                let fields = fields
                    .into_iter()
                    .map(|f| {
                        Ok(GoTypeFieldKey {
                            name: f.name,
                            unexported_package_id: f.package,
                            type_id: value(&f.typ)?,
                            tag_bytes_hex: f.tag,
                            embedded: f.embedded,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.intern(GoSourceTypeKey::Struct {
                    schema: TYPE_SCHEMA.into(),
                    id: String::new(),
                    fields,
                })
            }
        }
    }
    fn run(mut self, roots: Vec<Request>) -> Result<GoDerivedSourceTypes> {
        let mut work: Vec<_> = roots.iter().rev().cloned().map(Work::Visit).collect();
        while let Some(task) = work.pop() {
            match task {
                Work::Visit(r) => {
                    if self.done.contains_key(&r) {
                        continue;
                    }
                    if !self.active.insert(r.clone()) {
                        return Err(GoSourceTypeError::Cycle);
                    }
                    match self.plan(&r)? {
                        Ok(id) => {
                            self.done.insert(r.clone(), id);
                            self.active.remove(&r);
                        }
                        Err(plan) => {
                            let inputs = plan.inputs();
                            work.push(Work::Finish(r, plan));
                            for input in inputs.into_iter().rev() {
                                work.push(Work::Visit(input))
                            }
                        }
                    }
                }
                Work::Finish(r, plan) => {
                    let id = self.finish(plan)?;
                    self.done.insert(r.clone(), id);
                    self.active.remove(&r);
                }
            }
        }
        let mut underlying: BTreeMap<String, String> = BTreeMap::new();
        for id in self.raw_underlying.keys() {
            let mut current = id.as_str();
            let mut seen = BTreeSet::new();
            while let Some(next) = self.raw_underlying.get(current) {
                if !seen.insert(current) {
                    return Err(GoSourceTypeError::Cycle);
                }
                current = next
            }
            if matches!(&self.types[current],GoSourceTypeKey::Builtin{name,..}if name=="error") {
                return Err(GoSourceTypeError::Unsupported);
            }
            underlying.insert(id.clone(), current.into());
        }
        let edges = |id: &str| -> Vec<&str> {
            if let Some(next) = underlying.get(id) {
                vec![next]
            } else {
                self.types[id].edges()
            }
        };
        let mut pending = BTreeMap::new();
        let mut parents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        let mut ready = BTreeSet::new();
        for id in self.types.keys().map(String::as_str) {
            let children = edges(id);
            pending.insert(id, children.len());
            if children.is_empty() {
                ready.insert(id);
            }
            for c in children {
                parents.entry(c).or_default().push(id)
            }
        }
        let mut depth = BTreeMap::new();
        while let Some(id) = ready.pop_first() {
            let base = edges(id)
                .iter()
                .map(|child| depth[child])
                .max()
                .unwrap_or(0u32);
            let d = base
                + u32::from(matches!(
                    self.types[id],
                    GoSourceTypeKey::Array { .. }
                        | GoSourceTypeKey::Slice { .. }
                        | GoSourceTypeKey::Struct { .. }
                ));
            if d > 16 {
                return Err(GoSourceTypeError::Limit);
            }
            depth.insert(id, d);
            for parent in parents.get(id).into_iter().flatten() {
                let count = pending.get_mut(parent).ok_or(GoSourceTypeError::Mismatch)?;
                *count -= 1;
                if *count == 0 {
                    ready.insert(*parent);
                }
            }
        }
        if depth.len() != self.types.len() {
            return Err(GoSourceTypeError::Cycle);
        }
        let roots: Vec<_> = roots.iter().map(|r| self.done[r].clone()).collect();
        let mut reachable = BTreeSet::new();
        let mut todo = roots.clone();
        while let Some(id) = todo.pop() {
            if reachable.insert(id.clone()) {
                todo.extend(edges(&id).into_iter().map(str::to_string));
            }
        }
        // A definition's underlying type erases intermediate named/alias
        // spelling. Keep those source declarations as trace, while the value
        // graph contains only actual semantic children of the requested roots.
        self.types.retain(|id, _| reachable.contains(id));
        underlying.retain(|id, _| reachable.contains(id));
        Ok(GoDerivedSourceTypes {
            scopes: source_dependencies::shared_sources(self.scopes),
            compilation_reads: source_dependencies::Reads::new(),
            roots,
            declarations: self.declarations,
            types: self.types,
            underlying,
            aliases: self.aliases,
        })
    }
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
fn child(node: &GoSyntaxNode, role: &str) -> Result<usize> {
    let xs = list(node, role);
    if xs.len() == 1 {
        Ok(xs[0])
    } else {
        Err(GoSourceTypeError::NotType)
    }
}
/// Resolve an original type expression in the frozen v2 value-type domain.
pub fn derive_go_source_type(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoDerivedSourceTypes> {
    source_dependencies::type_nodes(scopes, source, &[node])
}
// A signature's ordered fields share one owned source/type graph. Grouped
// names reuse an original type expression; no synthetic AST or type is added.
pub(super) fn derive_go_source_type_nodes(
    scopes: &GoSourceScopes,
    source: &str,
    nodes: &[usize],
) -> Result<GoDerivedSourceTypes> {
    source_dependencies::type_nodes(scopes, source, nodes)
}
pub(super) fn derive_nodes_raw(
    scopes: &GoSourceScopes, source: &str, nodes: &[usize],
) -> Result<GoDerivedSourceTypes> {
    Builder::new(scopes).run(
        nodes
            .iter()
            .map(|&node| Request::expr(source, node))
            .collect(),
    )
}
pub(super) fn derive_go_variadic_binding_type(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoDerivedSourceTypes> {
    Builder::new(scopes).run(vec![Request {
        source: source.into(),
        target: Target::VariadicBinding(node),
    }])
}
/// Resolve an actual original TypeSpec and every referenced type declaration.
pub fn derive_go_source_type_declaration(
    scopes: &GoSourceScopes,
    source: &str,
    spec: usize,
) -> Result<GoDerivedSourceTypes> {
    source_dependencies::type_declaration(scopes, source, spec)
}
pub(super) fn derive_declaration_raw(
    scopes: &GoSourceScopes, source: &str, spec: usize,
) -> Result<GoDerivedSourceTypes> {
    Builder::new(scopes).run(vec![Request::decl(source, spec)])
}
/// Check candidate declaration bodies/aliases and all their derived children.
/// Anonymous candidate roots with no original declaration are not certified.
pub fn check_go_source_type_declarations(
    keys: &GoTypeKeyComponents,
) -> Result<GoDerivedSourceTypes> {
    let scopes = build_go_source_scopes(keys).map_err(|_| GoSourceTypeError::Scope)?;
    check_go_source_type_declarations_with_scopes(keys, &scopes)
}
/// Check declaration bodies using a larger, immutable original source universe.
/// Observed type components may omit imports unused by their particular roots.
/// Every candidate package source must match the scope owner's original bytes
/// and package identity. This still supplies no registered capture/context proof.
pub fn check_go_source_type_declarations_with_scopes(
    keys: &GoTypeKeyComponents,
    scopes: &GoSourceScopes,
) -> Result<GoDerivedSourceTypes> {
    for package in &keys.wire.packages {
        for id in &package.source_objects {
            let source = keys.source(id).ok_or(GoSourceTypeError::Locator)?;
            if scopes.source(id).map(GoParsedSource::source) != Some(source) {
                return Err(GoSourceTypeError::Locator);
            }
            if scopes.source_package(id) != Some(package.id.as_str()) {
                return Err(GoSourceTypeError::Scope);
            }
        }
    }
    let roots = keys
        .declarations()
        .iter()
        .map(|d| {
            let b = keys
                .source_declarations()
                .declaration(&d.id)
                .ok_or(GoSourceTypeError::Locator)?;
            Ok(Request::decl(&d.source_object_id, b.type_spec_node()))
        })
        .collect::<Result<Vec<_>>>()?;
    let derived = Builder::new(scopes).run(roots)?;
    let declarations: BTreeMap<_, _> = keys.wire.declarations.iter().map(|d| (&d.id, d)).collect();
    let types: BTreeMap<_, _> = keys.wire.types.iter().map(|t| (t.id(), t)).collect();
    let underlying: BTreeMap<_, _> = keys
        .wire
        .underlying
        .iter()
        .map(|u| (&u.type_id, &u.underlying_type_id))
        .collect();
    let aliases: BTreeMap<_, _> = keys
        .wire
        .aliases
        .iter()
        .map(|a| (&a.declaration_id, &a.target_type_id))
        .collect();
    for (id, d) in &declarations {
        if derived.declarations.get(*id) != Some(*d) {
            return Err(GoSourceTypeError::Mismatch);
        }
    }
    for (id, t) in &derived.types {
        if types.get(id.as_str()) != Some(&t) {
            return Err(GoSourceTypeError::Mismatch);
        }
    }
    for (id, t) in &derived.underlying {
        if underlying.get(id) != Some(&t) {
            return Err(GoSourceTypeError::Mismatch);
        }
    }
    for (id, t) in &aliases {
        if derived.aliases.get(*id) != Some(*t) {
            return Err(GoSourceTypeError::Mismatch);
        }
    }
    Ok(derived)
}
