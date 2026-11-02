//! Original-source selection and static dependencies. Native rows are never
//! inputs. This supplies no whole-package/capture/contract/state/program proof.
use super::*;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceSelectionRef {
    source: String,
    node: usize,
}
impl GoSourceSelectionRef {
    pub fn new(source: impl Into<String>, node: usize) -> Self {
        Self {
            source: source.into(),
            node,
        }
    }
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn node(&self) -> usize {
        self.node
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoSourceSelectionRole {
    SelectedRoot,
    BodyDependency,
    TypeDependency,
    ConstantDependency,
    SignatureDependency,
    BodyDeclaration,
    TypeCheckOnly,
}
impl GoSourceSelectionRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelectedRoot => "selected_root",
            Self::BodyDependency => "body_dependency",
            Self::TypeDependency => "type_dependency",
            Self::ConstantDependency => "constant_dependency",
            Self::SignatureDependency => "signature_dependency",
            Self::BodyDeclaration => "body_declaration",
            Self::TypeCheckOnly => "type_check_only",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSelectionDeclaration {
    locator: GoSourceSelectionRef,
    package: String,
    kind: &'static str,
    names: Vec<String>,
    roles: BTreeSet<GoSourceSelectionRole>,
}
impl GoSourceSelectionDeclaration {
    pub fn locator(&self) -> &GoSourceSelectionRef {
        &self.locator
    }
    pub fn package_id(&self) -> &str {
        &self.package
    }
    pub fn kind(&self) -> &str {
        self.kind
    }
    pub fn names(&self) -> &[String] {
        &self.names
    }
    pub fn roles(&self) -> impl Iterator<Item = GoSourceSelectionRole> + '_ {
        self.roles.iter().copied()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceSelectionDependency {
    from: GoSourceSelectionRef,
    to: GoSourceSelectionRef,
    role: GoSourceSelectionRole,
}
impl GoSourceSelectionDependency {
    pub fn from(&self) -> &GoSourceSelectionRef {
        &self.from
    }
    pub fn to(&self) -> &GoSourceSelectionRef {
        &self.to
    }
    pub fn role(&self) -> GoSourceSelectionRole {
        self.role
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceSelectionCall {
    caller: GoSourceSelectionRef,
    callee: GoSourceSelectionRef,
    node: usize,
}
impl GoSourceSelectionCall {
    pub fn caller(&self) -> &GoSourceSelectionRef {
        &self.caller
    }
    pub fn callee(&self) -> &GoSourceSelectionRef {
        &self.callee
    }
    pub fn call_node(&self) -> usize {
        self.node
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceSelectionErrorKind {
    Locator,
    DuplicateRoot,
    Body(GoBodyTypingError),
    Typing(GoScalarTypingError),
    Signature(GoSourceSignatureError),
    Constant(GoConstantError),
    MutablePackageState,
    Unsupported,
    MissingBody,
    CallCycle,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSelectionError {
    source: String,
    node: usize,
    kind: GoSourceSelectionErrorKind,
    path: Vec<GoSourceSelectionRef>,
}
impl GoSourceSelectionError {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn kind(&self) -> &GoSourceSelectionErrorKind {
        &self.kind
    }
    pub fn path(&self) -> &[GoSourceSelectionRef] {
        &self.path
    }
}
impl std::fmt::Display for GoSourceSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Go source selection error at {}:{}: {:?}",
            self.source, self.node, self.kind
        )
    }
}
impl std::error::Error for GoSourceSelectionError {}
type SelectionResult<T> = std::result::Result<T, GoSourceSelectionError>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSelection {
    scopes: Arc<GoSourceScopes>,
    packages: Vec<GoTypePackageKey>,
    roots: Vec<GoSourceSelectionRef>,
    declarations: Vec<GoSourceSelectionDeclaration>,
    dependencies: Vec<GoSourceSelectionDependency>,
    calls: Vec<GoSourceSelectionCall>,
}
impl GoSourceSelection {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.scopes.source(id)
    }
    pub fn packages(&self) -> &[GoTypePackageKey] {
        &self.packages
    }
    pub fn roots(&self) -> &[GoSourceSelectionRef] {
        &self.roots
    }
    pub fn declarations(&self) -> &[GoSourceSelectionDeclaration] {
        &self.declarations
    }
    pub fn dependencies(&self) -> &[GoSourceSelectionDependency] {
        &self.dependencies
    }
    pub fn calls(&self) -> &[GoSourceSelectionCall] {
        &self.calls
    }
}
struct Original {
    declaration: GoSourceSelectionDeclaration,
    typ: Option<usize>,
    values: Vec<usize>,
}
struct Pending {
    locator: GoSourceSelectionRef,
    role: GoSourceSelectionRole,
    path: Vec<GoSourceSelectionRef>,
}
struct Selection<'a> {
    scopes: &'a GoSourceScopes,
    originals: BTreeMap<GoSourceSelectionRef, Original>,
    work: Vec<Pending>,
    done: BTreeSet<(GoSourceSelectionRef, GoSourceSelectionRole)>,
    dependencies: BTreeSet<GoSourceSelectionDependency>,
    calls: BTreeSet<GoSourceSelectionCall>,
}
fn fail(
    locator: &GoSourceSelectionRef,
    node: usize,
    kind: GoSourceSelectionErrorKind,
    path: &[GoSourceSelectionRef],
) -> GoSourceSelectionError {
    GoSourceSelectionError {
        source: locator.source.clone(),
        node,
        kind,
        path: path.to_vec(),
    }
}
impl<'a> Selection<'a> {
    fn node(&self, source: &str, node: usize) -> &GoSyntaxNode {
        &self
            .scopes
            .source(source)
            .expect("indexed original source")
            .nodes()[node]
    }
    fn index(scopes: &'a GoSourceScopes, intrinsic: &BTreeSet<String>) -> Self {
        let mut originals = BTreeMap::new();
        for (source, parsed) in scopes.sources() {
            if scopes
                .source_package(source)
                .is_some_and(|p| intrinsic.contains(p))
            {
                continue;
            }
            for (node, n) in parsed.nodes().iter().enumerate() {
                if n.kind() == "FuncDecl" {
                    let name = attr(&parsed.nodes()[list(n, "Name")[0]], "Name").unwrap_or("");
                    let locator = GoSourceSelectionRef::new(source, node);
                    originals.insert(
                        locator.clone(),
                        Original {
                            declaration: GoSourceSelectionDeclaration {
                                locator,
                                package: scopes.source_package(source).unwrap_or("").into(),
                                kind: "function",
                                names: vec![name.into()],
                                roles: BTreeSet::new(),
                            },
                            typ: None,
                            values: vec![],
                        },
                    );
                }
                if n.kind() != "GenDecl" {
                    continue;
                }
                let kind = match attr(n, "Tok") {
                    Some("const") => "const",
                    Some("var") => "var",
                    Some("type") => "type",
                    _ => continue,
                };
                let mut inherited = (None, vec![]);
                for &spec in list(n, "Specs") {
                    let s = &parsed.nodes()[spec];
                    let (typ, values) = if kind == "const" {
                        if !list(s, "Values").is_empty() {
                            inherited =
                                (list(s, "Type").first().copied(), list(s, "Values").to_vec());
                        }
                        inherited.clone()
                    } else {
                        (list(s, "Type").first().copied(), list(s, "Values").to_vec())
                    };
                    let names = list(s, if kind == "type" { "Name" } else { "Names" })
                        .iter()
                        .map(|&id| attr(&parsed.nodes()[id], "Name").unwrap_or("").to_string())
                        .collect();
                    let locator = GoSourceSelectionRef::new(source, spec);
                    originals.insert(
                        locator.clone(),
                        Original {
                            declaration: GoSourceSelectionDeclaration {
                                locator,
                                package: scopes.source_package(source).unwrap_or("").into(),
                                kind,
                                names,
                                roles: BTreeSet::new(),
                            },
                            typ,
                            values,
                        },
                    );
                }
            }
        }
        Self {
            scopes,
            originals,
            work: vec![],
            done: BTreeSet::new(),
            dependencies: BTreeSet::new(),
            calls: BTreeSet::new(),
        }
    }
    fn need(
        &mut self,
        locator: GoSourceSelectionRef,
        role: GoSourceSelectionRole,
        path: &[GoSourceSelectionRef],
    ) -> SelectionResult<()> {
        let original = self.originals.get_mut(&locator).ok_or_else(|| {
            fail(
                &locator,
                locator.node,
                GoSourceSelectionErrorKind::Locator,
                path,
            )
        })?;
        original.declaration.roles.insert(role);
        if role != GoSourceSelectionRole::SelectedRoot
            && role != GoSourceSelectionRole::BodyDeclaration
        {
            if let Some(from) = path.last() {
                self.dependencies.insert(GoSourceSelectionDependency {
                    from: from.clone(),
                    to: locator.clone(),
                    role,
                });
            }
        }
        let mut next = path.to_vec();
        if next.last() != Some(&locator) {
            next.push(locator.clone());
        }
        self.work.push(Pending {
            locator,
            role,
            path: next,
        });
        Ok(())
    }
    fn binding(
        &mut self,
        binding: &GoSourceNameBinding,
        role: GoSourceSelectionRole,
        path: &[GoSourceSelectionRef],
    ) -> SelectionResult<()> {
        self.need(
            GoSourceSelectionRef::new(binding.source_object_id(), binding.declaration_node()),
            role,
            path,
        )
    }
    fn lookup(
        &self,
        source: &str,
        mut node: usize,
        context: Option<usize>,
    ) -> std::result::Result<GoNameLookup, GoScalarTypingError> {
        loop {
            let n = self.node(source, node);
            match n.kind() {
                "ParenExpr" => node = child(n, "X")?,
                "Ident" => {
                    return self
                        .scopes
                        .lookup_context(source, node, context)
                        .map_err(|_| GoScalarTypingError::Name)?
                        .ok_or(GoScalarTypingError::Name)
                }
                "SelectorExpr" => {
                    let Some(GoNameLookup::Original(package)) = self
                        .scopes
                        .lookup_context(source, child(n, "X")?, context)
                        .map_err(|_| GoScalarTypingError::Name)?
                    else {
                        return Err(GoScalarTypingError::Unsupported);
                    };
                    if package.kind() != GoSourceNameKind::Import {
                        return Err(GoScalarTypingError::Unsupported);
                    }
                    return self
                        .scopes
                        .imported_member(
                            package.import_path().ok_or(GoScalarTypingError::Name)?,
                            attr(self.node(source, child(n, "Sel")?), "Name")
                                .ok_or(GoScalarTypingError::Name)?,
                        )
                        .map_err(|_| GoScalarTypingError::Name)?
                        .map(GoNameLookup::Original)
                        .ok_or(GoScalarTypingError::Name);
                }
                _ => return Err(GoScalarTypingError::Unsupported),
            }
        }
    }
    fn graph(
        &mut self,
        from: &GoSourceSelectionRef,
        graph: &GoDerivedSourceTypes,
        path: &[GoSourceSelectionRef],
    ) -> SelectionResult<()> {
        let mut engine = Engine::new(self.scopes);
        if let Some(root) = graph.roots().first() {
            engine
                .graph_type(graph, root)
                .map_err(|e| fail(from, from.node, GoSourceSelectionErrorKind::Typing(e), path))?;
        }
        for implementation in engine.error_implementations.values().flatten() {
            let method = implementation.method();
            if !method.receiver_path().is_empty() {
                return Err(fail(
                    from,
                    from.node,
                    GoSourceSelectionErrorKind::Unsupported,
                    path,
                ));
            }
            if let (Some(source), Some(node)) = (
                method.declaration_source_object_id(),
                method.declaration_node(),
            ) {
                self.need(
                    GoSourceSelectionRef::new(source, node),
                    GoSourceSelectionRole::SignatureDependency,
                    path,
                )?;
            }
        }
        Ok(())
    }
    fn is_struct(&self, source: &str, mut node: usize, mut context: Option<usize>) -> bool {
        let mut source = source.to_string();
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert((source.clone(), node)) {
                return false;
            }
            let n = self.node(&source, node);
            match n.kind() {
                "StructType" => return true,
                "ParenExpr" => node = list(n, "X")[0],
                "Ident" | "SelectorExpr" => {
                    let Ok(GoNameLookup::Original(b)) = self.lookup(&source, node, context) else {
                        return false;
                    };
                    if b.kind() != GoSourceNameKind::Type {
                        return false;
                    }
                    source = b.source_object_id().into();
                    node = list(self.node(&source, b.declaration_node()), "Type")[0];
                    context = None;
                }
                _ => return false,
            }
        }
    }
    fn walk(
        &mut self,
        from: &GoSourceSelectionRef,
        roots: Vec<(usize, bool)>,
        context: Option<usize>,
        typing: Option<&GoValueTyping>,
        path: &[GoSourceSelectionRef],
    ) -> SelectionResult<()> {
        let mut engine = Engine::new(self.scopes);
        let mut tasks = roots;
        let mut seen = BTreeSet::new();
        while let Some((node, runtime)) = tasks.pop() {
            if !seen.insert((node, runtime)) {
                continue;
            }
            let n = self.node(&from.source, node).clone();
            let err = |e| fail(from, node, GoSourceSelectionErrorKind::Typing(e), path);
            match n.kind() {
                "RangeStmt" => {
                    tasks.push((child(&n, "X").map_err(err)?, runtime));
                    tasks.extend(list(&n, "Body").iter().map(|&n| (n, runtime)));
                    if attr(&n, "Tok") != Some(":=") {
                        tasks.extend(
                            list(&n, "Key")
                                .iter()
                                .chain(list(&n, "Value"))
                                .map(|&n| (n, runtime)),
                        );
                    }
                    continue;
                }
                "AssignStmt" if attr(&n, "Tok") == Some(":=") => {
                    tasks.extend(list(&n, "Rhs").iter().map(|&n| (n, runtime)));
                    for &lhs in list(&n, "Lhs") {
                        if !self.scopes.bindings().iter().any(|b| {
                            b.source_object_id() == from.source
                                && b.declaration_node() == node
                                && b.name_node() == lhs
                        }) {
                            tasks.push((lhs, runtime));
                        }
                    }
                    continue;
                }

                "GenDecl" => {
                    let role = match attr(&n, "Tok") {
                        Some("type") => GoSourceSelectionRole::TypeDependency,
                        Some("const") => GoSourceSelectionRole::ConstantDependency,
                        Some("var") => GoSourceSelectionRole::BodyDeclaration,
                        _ => {
                            return Err(fail(
                                from,
                                node,
                                GoSourceSelectionErrorKind::Unsupported,
                                path,
                            ))
                        }
                    };
                    for &spec in list(&n, "Specs") {
                        let locator = GoSourceSelectionRef::new(&from.source, spec);
                        self.need(locator, role, path)?;
                        if role == GoSourceSelectionRole::BodyDeclaration {
                            tasks.push((spec, runtime));
                        }
                    }
                    continue;
                }
                "ValueSpec" => {
                    tasks.extend(list(&n, "Type").iter().map(|&n| (n, false)));
                    tasks.extend(list(&n, "Values").iter().map(|&n| (n, runtime)));
                    continue;
                }
                "TypeSpec" => {
                    tasks.extend(list(&n, "Type").iter().map(|&n| (n, false)));
                    continue;
                }
                "Field" => {
                    tasks.extend(list(&n, "Type").iter().map(|&n| (n, false)));
                    continue;
                }
                "CompositeLit" => {
                    let typ = list(&n, "Type").first().copied();
                    let structure = typ.is_some_and(|t| self.is_struct(&from.source, t, context));
                    tasks.extend(typ.map(|n| (n, false)));
                    for &element in list(&n, "Elts") {
                        let e = self.node(&from.source, element);
                        if e.kind() == "KeyValueExpr" && structure {
                            tasks.push((list(e, "Value")[0], runtime));
                        } else {
                            tasks.push((element, runtime));
                        }
                    }
                    continue;
                }
                "CallExpr" => {
                    let fun = child(&n, "Fun").map_err(err)?;
                    let mut arguments_runtime = runtime;
                    match self.lookup(&from.source, fun, context) {
                        Ok(GoNameLookup::Predeclared {
                            kind: GoSourceNameKind::BuiltinFunction,
                            name,
                        }) => {
                            if !matches!(name.as_str(), "append" | "cap" | "copy" | "len" | "make")
                            {
                                return Err(fail(
                                    from,
                                    node,
                                    GoSourceSelectionErrorKind::Unsupported,
                                    path,
                                ));
                            }
                            if runtime && matches!(name.as_str(), "len" | "cap") {
                                let request = Request::raw(&from.source, node);
                                engine.compute(request.clone()).map_err(err)?;
                                arguments_runtime =
                                    engine.one(&request).map_err(err)?.value.is_none();
                            }
                            tasks.push((fun, false));
                        }
                        Ok(GoNameLookup::Original(b)) if b.kind() == GoSourceNameKind::Function => {
                            self.binding(&b, GoSourceSelectionRole::SignatureDependency, path)?;
                            if runtime {
                                let target = GoSourceSelectionRef::new(
                                    b.source_object_id(),
                                    b.declaration_node(),
                                );
                                self.calls.insert(GoSourceSelectionCall {
                                    caller: from.clone(),
                                    callee: target.clone(),
                                    node,
                                });
                                self.need(target, GoSourceSelectionRole::BodyDependency, path)?;
                            }
                            tasks.push((fun, false));
                        }
                        Ok(GoNameLookup::Original(b)) if b.kind() == GoSourceNameKind::Type => {
                            tasks.push((fun, false))
                        }
                        Ok(GoNameLookup::Predeclared {
                            kind: GoSourceNameKind::BuiltinType,
                            ..
                        }) => tasks.push((fun, false)),
                        _ => {
                            let header = typing
                                .and_then(|t| {
                                    t.method_call_headers().iter().find(|h| {
                                        h.call_source_object_id() == from.source
                                            && h.call_node() == node
                                    })
                                })
                                .ok_or_else(|| {
                                    fail(from, node, GoSourceSelectionErrorKind::Unsupported, path)
                                })?;
                            let method = header.method();
                            if !method.receiver_path().is_empty() {
                                return Err(fail(
                                    from,
                                    node,
                                    GoSourceSelectionErrorKind::Unsupported,
                                    path,
                                ));
                            }
                            let (Some(source), Some(declaration)) = (
                                method.declaration_source_object_id(),
                                method.declaration_node(),
                            ) else {
                                return Err(fail(
                                    from,
                                    node,
                                    GoSourceSelectionErrorKind::Unsupported,
                                    path,
                                ));
                            };
                            let target = GoSourceSelectionRef::new(source, declaration);
                            self.need(
                                target.clone(),
                                GoSourceSelectionRole::SignatureDependency,
                                path,
                            )?;
                            if runtime {
                                self.calls.insert(GoSourceSelectionCall {
                                    caller: from.clone(),
                                    callee: target.clone(),
                                    node,
                                });
                                self.need(target, GoSourceSelectionRole::BodyDependency, path)?;
                            }
                            let selector = self.node(&from.source, header.function_node());
                            tasks.push((
                                child(selector, "X").map_err(err)?,
                                runtime && !header.method_expression(),
                            ));
                        }
                    }
                    tasks.extend(list(&n, "Args").iter().map(|&n| (n, arguments_runtime)));
                    continue;
                }
                "SelectorExpr" => {
                    if let Ok(GoNameLookup::Original(b)) = self.lookup(&from.source, node, context)
                    {
                        self.reference(from, node, b, runtime, path)?;
                        continue;
                    }
                    // Independently require a direct field, not an embedding promotion.
                    let base = child(&n, "X").map_err(err)?;
                    let request = Request::raw(&from.source, base);
                    engine.compute(request.clone()).map_err(err)?;
                    let atom = engine.one(&request).map_err(err)?;
                    let name = attr(
                        self.node(&from.source, child(&n, "Sel").map_err(err)?),
                        "Name",
                    )
                    .ok_or_else(|| err(GoScalarTypingError::Name))?;
                    let Some(typ) = atom.typ else {
                        return Err(err(GoScalarTypingError::Type));
                    };
                    if !matches!(engine.shape(&typ).map_err(err)?,GoSourceTypeKey::Struct{fields,..}if fields.iter().any(|f|f.name==name))
                    {
                        return Err(fail(
                            from,
                            node,
                            GoSourceSelectionErrorKind::Unsupported,
                            path,
                        ));
                    }
                    tasks.push((base, runtime));
                    continue;
                }
                "Ident" => {
                    if attr(&n, "Name") == Some("_") {
                        continue;
                    }
                    match self.lookup(&from.source, node, context) {
                        Ok(GoNameLookup::Original(b)) => {
                            self.reference(from, node, b, runtime, path)?
                        }
                        Ok(GoNameLookup::Predeclared { .. }) => (),
                        Err(e) => return Err(err(e)),
                    };
                    continue;
                }
                "FuncLit" | "GoStmt" | "DeferStmt" | "SelectStmt" | "TypeSwitchStmt"
                | "LabeledStmt" | "TypeAssertExpr" | "StarExpr" | "MapType" | "ChanType"
                | "InterfaceType" | "IndexListExpr" | "SendStmt" => {
                    return Err(fail(
                        from,
                        node,
                        GoSourceSelectionErrorKind::Unsupported,
                        path,
                    ))
                }
                _ => (),
            }
            for edge in n.edges() {
                tasks.extend(edge.targets().iter().map(|&n| (n, runtime)));
            }
        }
        Ok(())
    }
    fn variable_types(&mut self,from:&GoSourceSelectionRef,node:usize,b:&GoSourceNameBinding,path:&[GoSourceSelectionRef])->SelectionResult<()> {
        let mut engine=Engine::new(self.scopes);
        let index=engine.binding_index(b).map_err(|e|fail(from,node,GoSourceSelectionErrorKind::Typing(e),path))?;
        let request=Request::binding(b,index);engine.compute(request.clone()).map_err(|e|fail(from,node,GoSourceSelectionErrorKind::Typing(e),path))?;
        let atom=engine.one(&request).map_err(|e|fail(from,node,GoSourceSelectionErrorKind::Typing(e),path))?;
        let mut work=atom.typ.into_iter().collect::<Vec<_>>();let mut seen=BTreeSet::new();
        while let Some(id)=work.pop(){if !seen.insert(id.clone()){continue}
            match engine.keys.get(&id){
                Some(GoSourceTypeKey::Nominal{declaration_id,..})=>{
                    let declaration=engine.declarations.get(declaration_id).ok_or_else(||fail(from,node,GoSourceSelectionErrorKind::Locator,path))?;
                    let parsed=self.scopes.source(&declaration.source_object_id).ok_or_else(||fail(from,node,GoSourceSelectionErrorKind::Locator,path))?;
                    let typ=parsed.nodes().iter().position(|n|n.kind()=="TypeSpec"&&n.start()as u64==declaration.start_byte&&n.end()as u64==declaration.end_byte).ok_or_else(||fail(from,node,GoSourceSelectionErrorKind::Locator,path))?;
                    self.need(GoSourceSelectionRef::new(&declaration.source_object_id,typ),GoSourceSelectionRole::TypeDependency,path)?;
                    if let Some(base)=engine.underlying.get(&id){work.push(base.clone())}
                }
                Some(GoSourceTypeKey::Array{element,..}|GoSourceTypeKey::Slice{element,..})=>work.push(element.clone()),
                Some(GoSourceTypeKey::Struct{fields,..})=>work.extend(fields.iter().map(|f|f.type_id.clone())),
                _=>(),
            }
            if let Some(Some(implementation))=engine.error_implementations.get(&id){let method=implementation.method();if let (Some(source),Some(node))=(method.declaration_source_object_id(),method.declaration_node()){self.need(GoSourceSelectionRef::new(source,node),GoSourceSelectionRole::SignatureDependency,path)?;}}
        }
        Ok(())
    }
    fn reference(
        &mut self,
        from: &GoSourceSelectionRef,
        node: usize,
        b: GoSourceNameBinding,
        runtime: bool,
        path: &[GoSourceSelectionRef],
    ) -> SelectionResult<()> {
        match b.kind() {
            GoSourceNameKind::Type => {
                self.binding(&b, GoSourceSelectionRole::TypeDependency, path)?;
                let graph = derive_go_source_type_declaration(
                    self.scopes,
                    b.source_object_id(),
                    b.declaration_node(),
                )
                .map_err(|e| {
                    fail(
                        from,
                        node,
                        GoSourceSelectionErrorKind::Typing(GoScalarTypingError::SourceType(e)),
                        path,
                    )
                })?;
                self.graph(from, &graph, path)?;
            }
            GoSourceNameKind::Constant => {
                self.binding(&b, GoSourceSelectionRole::ConstantDependency, path)?
            }
            GoSourceNameKind::Function => {
                if runtime {
                    return Err(fail(
                        from,
                        node,
                        GoSourceSelectionErrorKind::Unsupported,
                        path,
                    ));
                }
                self.binding(&b, GoSourceSelectionRole::SignatureDependency, path)?;
            }
            GoSourceNameKind::Variable => {
                self.variable_types(from,node,&b,path)?;
                if self.scopes.package_binding(&b) {
                    if runtime {
                        let mut p = path.to_vec();
                        p.push(GoSourceSelectionRef::new(
                            b.source_object_id(),
                            b.declaration_node(),
                        ));
                        return Err(fail(
                            from,
                            node,
                            GoSourceSelectionErrorKind::MutablePackageState,
                            &p,
                        ));
                    }
                    self.binding(&b, GoSourceSelectionRole::TypeDependency, path)?;
                }
            }
            GoSourceNameKind::Import => (),
            _ => {
                return Err(fail(
                    from,
                    node,
                    GoSourceSelectionErrorKind::Unsupported,
                    path,
                ))
            }
        }
        Ok(())
    }
    fn process(&mut self, w: &Pending) -> SelectionResult<()> {
        if w.role == GoSourceSelectionRole::BodyDeclaration {
            return Ok(());
        }
        let role = if w.role == GoSourceSelectionRole::SelectedRoot {
            GoSourceSelectionRole::BodyDependency
        } else {
            w.role
        };
        if !self.done.insert((w.locator.clone(), role)) {
            return Ok(());
        }
        let n = self.node(&w.locator.source, w.locator.node).clone();
        match n.kind() {
            "FuncDecl" => {
                let signature =
                    derive_go_source_signature(self.scopes, &w.locator.source, w.locator.node)
                        .map_err(|e| {
                            fail(
                                &w.locator,
                                w.locator.node,
                                GoSourceSelectionErrorKind::Signature(e),
                                &w.path,
                            )
                        })?;
                self.graph(&w.locator, signature.types(), &w.path)?;
                self.walk(
                    &w.locator,
                    list(&n, "Recv")
                        .iter()
                        .chain(list(&n, "Type"))
                        .map(|&n| (n, false))
                        .collect(),
                    None,
                    None,
                    &w.path,
                )?;
                if role == GoSourceSelectionRole::BodyDependency {
                    if signature.name() == Some("init") {
                        return Err(fail(
                            &w.locator,
                            w.locator.node,
                            GoSourceSelectionErrorKind::Unsupported,
                            &w.path,
                        ));
                    }
                    if signature.body_node().is_none() {
                        return Err(fail(
                            &w.locator,
                            w.locator.node,
                            GoSourceSelectionErrorKind::MissingBody,
                            &w.path,
                        ));
                    }
                    let body =
                        check_go_function_body(self.scopes, &w.locator.source, w.locator.node)
                            .map_err(|e| {
                                fail(
                                    &w.locator,
                                    e.node(),
                                    GoSourceSelectionErrorKind::Body(e),
                                    &w.path,
                                )
                            })?;
                    self.walk(
                        &w.locator,
                        vec![(signature.body_node().unwrap(), true)],
                        None,
                        Some(body.typing()),
                        &w.path,
                    )?;
                }
            }
            "TypeSpec" => self.walk(
                &w.locator,
                vec![(w.locator.node, false)],
                None,
                None,
                &w.path,
            )?,
            "ValueSpec" => {
                if role==GoSourceSelectionRole::TypeDependency {
                    let bindings=self.scopes.bindings().iter().filter(|b|b.source_object_id()==w.locator.source&&b.declaration_node()==w.locator.node&&b.kind()==GoSourceNameKind::Variable).cloned().collect::<Vec<_>>();
                    for b in bindings{self.variable_types(&w.locator,w.locator.node,&b,&w.path)?;}
                }
                let original = &self.originals[&w.locator];
                let typ = original.typ;
                let values = original.values.clone();
                let constant = role == GoSourceSelectionRole::ConstantDependency;
                if constant {
                    evaluate_go_constant_declaration(
                        self.scopes,
                        &w.locator.source,
                        w.locator.node,
                    )
                    .map_err(|e| {
                        fail(
                            &w.locator,
                            w.locator.node,
                            GoSourceSelectionErrorKind::Constant(e),
                            &w.path,
                        )
                    })?;
                }
                let roots = typ
                    .into_iter()
                    .chain(if constant { values } else { vec![] })
                    .map(|n| (n, false))
                    .collect();
                self.walk(
                    &w.locator,
                    roots,
                    constant.then_some(w.locator.node),
                    None,
                    &w.path,
                )?;
            }
            _ => {
                return Err(fail(
                    &w.locator,
                    w.locator.node,
                    GoSourceSelectionErrorKind::Locator,
                    &w.path,
                ))
            }
        }
        Ok(())
    }
    fn cycles(&self, roots: &[GoSourceSelectionRef]) -> SelectionResult<()> {
        let mut edges: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for call in &self.calls {
            edges.entry(call.caller.clone()).or_default().push(call);
        }
        let mut state: BTreeMap<_, u8> = BTreeMap::new();
        for root in roots {
            if state.contains_key(root) {
                continue;
            }
            let mut stack = vec![(root.clone(), 0)];
            state.insert(root.clone(), 1);
            while let Some((current, next)) = stack.last_mut() {
                let calls = edges.get(current).map_or(&[][..], Vec::as_slice);
                if *next == calls.len() {
                    state.insert(current.clone(), 2);
                    stack.pop();
                    continue;
                }
                let call = calls[*next];
                *next += 1;
                match state.get(&call.callee) {
                    Some(1) => {
                        let mut path = stack.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>();
                        path.push(call.callee.clone());
                        return Err(fail(
                            &call.caller,
                            call.node,
                            GoSourceSelectionErrorKind::CallCycle,
                            &path,
                        ));
                    }
                    Some(_) => (),
                    None => {
                        state.insert(call.callee.clone(), 1);
                        stack.push((call.callee.clone(), 0));
                    }
                }
            }
        }
        Ok(())
    }
}
/// Reconstruct supported original selection dependencies. Roots are locators,
/// not compiler facts. Native metadata is never consulted or trusted.
/// Other unselected bodies/packages and capture/contract/program authority need
/// their separate complete validation; a successful graph is not a proof.
pub fn derive_go_source_selection(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceSelectionRef],
) -> SelectionResult<GoSourceSelection> {
    let first = roots
        .first()
        .cloned()
        .unwrap_or_else(|| GoSourceSelectionRef::new("", 0));
    let scopes = build_go_source_scopes(keys)
        .map_err(|_| fail(&first, first.node, GoSourceSelectionErrorKind::Locator, &[]))?;
    let intrinsic = keys
        .wire
        .packages
        .iter()
        .filter(|p| p.import_path == "unsafe")
        .map(|p| p.id.clone())
        .collect();
    let mut builder = Selection::index(&scopes, &intrinsic);
    let mut roots = roots.to_vec();
    roots.sort();
    if roots.is_empty() {
        return Err(fail(
            &first,
            first.node,
            GoSourceSelectionErrorKind::Locator,
            &[],
        ));
    }
    for (i, r) in roots.iter().enumerate() {
        if i > 0 && roots[i - 1] == *r {
            return Err(fail(
                r,
                r.node,
                GoSourceSelectionErrorKind::DuplicateRoot,
                &[],
            ));
        }
        if !builder
            .originals
            .get(r)
            .is_some_and(|d| d.declaration.kind == "function")
        {
            return Err(fail(r, r.node, GoSourceSelectionErrorKind::Locator, &[]));
        }
        builder.need(r.clone(), GoSourceSelectionRole::SelectedRoot, &[])?;
    }
    let mut cursor = 0;
    while cursor < builder.work.len() {
        let w = &builder.work[cursor];
        let current = Pending {
            locator: w.locator.clone(),
            role: w.role,
            path: w.path.clone(),
        };
        cursor += 1;
        builder.process(&current)?;
    }
    builder.cycles(&roots)?;
    let declarations = builder
        .originals
        .into_values()
        .map(|mut o| {
            if o.declaration.roles.is_empty() {
                o.declaration
                    .roles
                    .insert(GoSourceSelectionRole::TypeCheckOnly);
            }
            o.declaration
        })
        .collect();
    let dependencies = builder.dependencies.into_iter().collect();
    let calls = builder.calls.into_iter().collect();
    Ok(GoSourceSelection {
        scopes: Arc::new(scopes),
        packages: keys.wire.packages.clone(),
        roots,
        declarations,
        dependencies,
        calls,
    })
}
