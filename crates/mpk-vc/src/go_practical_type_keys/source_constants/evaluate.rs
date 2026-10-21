use super::*;
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Target {
    Expression { node: usize, context: Option<usize> },
    Binding { spec: usize, name: usize },
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Request {
    source: String,
    target: Target,
}
impl Request {
    fn expression(source: &str, node: usize, context: Option<usize>) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression { node, context },
        }
    }
    fn binding(b: &GoSourceNameBinding) -> Self {
        Self {
            source: b.source_object_id().into(),
            target: Target::Binding {
                spec: b.declaration_node(),
                name: b.name_node(),
            },
        }
    }
}
#[derive(Clone)]
struct Spec {
    ordinal: u64,
    names: Vec<usize>,
    values: Vec<usize>,
    typ: Option<usize>,
    valid: bool,
}
enum Plan {
    Forward(Request),
    Unary(String, Request),
    Binary(String, Request, Request),
    Convert(Box<GoConstantScalarType>, Request, bool),
    Builtin(String, Vec<Request>),
}
impl Plan {
    fn inputs(&self) -> Vec<Request> {
        match self {
            Self::Forward(a) | Self::Unary(_, a) | Self::Convert(_, a, _) => vec![a.clone()],
            Self::Binary(_, a, b) => vec![a.clone(), b.clone()],
            Self::Builtin(_, args) => args.clone(),
        }
    }
}
enum Work {
    Visit(Request),
    Finish(Request, Plan),
}
struct Engine<'a> {
    scopes: &'a GoSourceScopes,
    specs: BTreeMap<(String, usize), Spec>,
    done: BTreeMap<Request, GoEvaluatedConstant>,
    active: BTreeSet<Request>,
}
impl<'a> Engine<'a> {
    fn new(scopes: &'a GoSourceScopes) -> Self {
        let mut specs = BTreeMap::new();
        for (source, parsed) in scopes.sources() {
            for decl in parsed.nodes() {
                if decl.kind() != "GenDecl" || attr(decl, "Tok") != Some("const") {
                    continue;
                }
                let mut previous: Option<(Option<usize>, Vec<usize>)> = None;
                for (ordinal, &id) in list(decl, "Specs").iter().enumerate() {
                    let spec = &parsed.nodes()[id];
                    let typ = list(spec, "Type").first().copied();
                    let values = list(spec, "Values");
                    let valid = if !values.is_empty() {
                        previous = Some((typ, values.to_vec()));
                        true
                    } else {
                        typ.is_none() && previous.is_some()
                    };
                    let (typ, values) = previous.clone().unwrap_or((None, vec![]));
                    let names = list(spec, "Names").to_vec();
                    let valid = valid && names.len() == values.len();
                    specs.insert(
                        (source.into(), id),
                        Spec {
                            ordinal: ordinal as u64,
                            names,
                            values,
                            typ,
                            valid,
                        },
                    );
                }
            }
        }
        Self {
            scopes,
            specs,
            done: BTreeMap::new(),
            active: BTreeSet::new(),
        }
    }
    fn node(&self, source: &str, id: usize) -> Result<&GoSyntaxNode> {
        self.scopes
            .source(source)
            .and_then(|s| s.nodes().get(id))
            .ok_or(GoConstantError::Locator)
    }
    fn context(&self, source: &str, node: usize) -> Result<Option<usize>> {
        let query = self.node(source, node)?;
        Ok(self
            .specs
            .keys()
            .filter(|(s, _)| s == source)
            .filter_map(|(_, id)| {
                let n = self.node(source, *id).ok()?;
                (n.start() <= query.start() && query.end() <= n.end())
                    .then_some((*id, n.end() - n.start()))
            })
            .min_by_key(|(_, len)| *len)
            .map(|(id, _)| id))
    }
    fn symbol(
        &self,
        source: &str,
        mut node: usize,
        context: Option<usize>,
    ) -> Result<GoNameLookup> {
        loop {
            let n = self.node(source, node)?;
            match n.kind() {
                "ParenExpr" => node = child(n, "X")?,
                "Ident" => {
                    return self
                        .scopes
                        .lookup_context(source, node, context)
                        .map_err(|_| GoConstantError::Scope)?
                        .ok_or(GoConstantError::NotConstant)
                }
                "SelectorExpr" => {
                    let base = child(n, "X")?;
                    let select = child(n, "Sel")?;
                    if self.node(source, base)?.kind() != "Ident" {
                        return Err(GoConstantError::NotConstant);
                    }
                    let Some(GoNameLookup::Original(package)) = self
                        .scopes
                        .lookup_context(source, base, context)
                        .map_err(|_| GoConstantError::Scope)?
                    else {
                        return Err(GoConstantError::NotConstant);
                    };
                    if package.kind() != GoSourceNameKind::Import {
                        return Err(GoConstantError::NotConstant);
                    }
                    let name =
                        attr(self.node(source, select)?, "Name").ok_or(GoConstantError::Type)?;
                    return self
                        .scopes
                        .imported_member(package.import_path().ok_or(GoConstantError::Scope)?, name)
                        .map_err(|_| GoConstantError::Scope)?
                        .map(GoNameLookup::Original)
                        .ok_or(GoConstantError::NotConstant);
                }
                _ => return Err(GoConstantError::Unsupported),
            }
        }
    }
    fn scalar_type(
        &self,
        source: &str,
        node: usize,
        context: Option<usize>,
    ) -> Result<GoConstantScalarType> {
        let mut source = source.to_string();
        let mut node = node;
        let mut context = context;
        let mut nominal = None;
        let mut seen = BTreeSet::new();
        loop {
            match self.symbol(&source, node, context)? {
                GoNameLookup::Predeclared {
                    name,
                    kind: GoSourceNameKind::BuiltinType,
                } => {
                    let mut typ = GoConstantScalarType::builtin(&name)?;
                    if let Some(declaration) = nominal {
                        let declaration: GoTypeDeclarationKey = declaration;
                        let mut key = GoSourceTypeKey::Nominal {
                            schema: TYPE_SCHEMA.into(),
                            id: String::new(),
                            declaration_id: declaration.id.clone(),
                        };
                        let id =
                            identity(&key, TYPE_DOMAIN, "id").map_err(|_| GoConstantError::Type)?;
                        if let GoSourceTypeKey::Nominal { id: field, .. } = &mut key {
                            *field = id
                        }
                        typ.key = key;
                        typ.declaration = Some(declaration);
                    }
                    return Ok(typ);
                }
                GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Type => {
                    let spec = self.node(b.source_object_id(), b.declaration_node())?;
                    if spec.kind() != "TypeSpec" || !list(spec, "TypeParams").is_empty() {
                        return Err(GoConstantError::Unsupported);
                    }
                    if !seen.insert((b.source_object_id().to_string(), b.declaration_node())) {
                        return Err(GoConstantError::Cycle);
                    }
                    if nominal.is_none() && attr(spec, "Alias") != Some("true") {
                        let mut decl = GoTypeDeclarationKey {
                            schema: DECL_SCHEMA.into(),
                            id: String::new(),
                            package_id: b.package_id().into(),
                            source_object_id: b.source_object_id().into(),
                            kind: "type".into(),
                            start_byte: spec.start() as u64,
                            end_byte: spec.end() as u64,
                            name: b.name().into(),
                        };
                        decl.id = identity(&decl, DECL_DOMAIN, "id")
                            .map_err(|_| GoConstantError::Type)?;
                        nominal = Some(decl);
                    }
                    node = child(spec, "Type")?;
                    source = b.source_object_id().into();
                    context = None;
                }
                _ => return Err(GoConstantError::Type),
            }
        }
    }
    fn plan(&self, r: &Request) -> Result<std::result::Result<GoEvaluatedConstant, Plan>> {
        let (node, context) = match r.target {
            Target::Expression { node, context } => (node, context),
            Target::Binding { spec, name } => {
                let meta = self
                    .specs
                    .get(&(r.source.clone(), spec))
                    .ok_or(GoConstantError::NotConstant)?;
                if !meta.valid {
                    return Err(GoConstantError::Type);
                }
                let index = meta
                    .names
                    .iter()
                    .position(|&n| n == name)
                    .ok_or(GoConstantError::Locator)?;
                let child = Request::expression(&r.source, meta.values[index], Some(spec));
                return Ok(Err(match meta.typ {
                    Some(typ) => Plan::Convert(
                        Box::new(self.scalar_type(&r.source, typ, Some(spec))?),
                        child,
                        false,
                    ),
                    None => Plan::Forward(child),
                }));
            }
        };
        let syntax = self.node(&r.source, node)?;
        let expression = |n| Request::expression(&r.source, n, context);
        match syntax.kind() {
            "BasicLit" => {
                let raw = self
                    .scopes
                    .source(&r.source)
                    .unwrap()
                    .spelling(node)
                    .ok_or(GoConstantError::Locator)?;
                let value = match attr(syntax, "Kind") {
                    Some("INT") => GoEvaluatedConstant::integer(
                        Integer::literal(raw).ok_or(GoConstantError::Precision)?,
                        false,
                    ),
                    Some("CHAR") => GoEvaluatedConstant::integer(
                        Integer::from_u64(quoted(raw)?.1.ok_or(GoConstantError::Type)? as u64),
                        true,
                    ),
                    Some("STRING") => GoEvaluatedConstant {
                        kind: GoConstantKind::String,
                        value: Value::Bytes(quoted(raw)?.0),
                        typ: None,
                    },
                    _ => return Err(GoConstantError::Unsupported),
                };
                Ok(Ok(value.checked()?))
            }
            "Ident" | "SelectorExpr" => match self.symbol(&r.source, node, context)? {
                GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Constant => {
                    Ok(Err(Plan::Forward(Request::binding(&b))))
                }
                GoNameLookup::Predeclared {
                    name,
                    kind: GoSourceNameKind::BuiltinConstant,
                } => Ok(Ok(match name.as_str() {
                    "true" => GoEvaluatedConstant::boolean_value(true),
                    "false" => GoEvaluatedConstant::boolean_value(false),
                    "iota" => GoEvaluatedConstant::integer(
                        Integer::from_u64(
                            context
                                .and_then(|id| self.specs.get(&(r.source.clone(), id)))
                                .ok_or(GoConstantError::NotConstant)?
                                .ordinal,
                        ),
                        false,
                    ),
                    _ => return Err(GoConstantError::NotConstant),
                })),
                _ => Err(GoConstantError::NotConstant),
            },
            "ParenExpr" => Ok(Err(Plan::Forward(expression(child(syntax, "X")?)))),
            "UnaryExpr" => Ok(Err(Plan::Unary(
                attr(syntax, "Op").ok_or(GoConstantError::Type)?.into(),
                expression(child(syntax, "X")?),
            ))),
            "BinaryExpr" => Ok(Err(Plan::Binary(
                attr(syntax, "Op").ok_or(GoConstantError::Type)?.into(),
                expression(child(syntax, "X")?),
                expression(child(syntax, "Y")?),
            ))),
            "CallExpr" => {
                if attr(syntax, "Ellipsis") == Some("true") {
                    return Err(GoConstantError::Type);
                }
                let fun = child(syntax, "Fun")?;
                let args = list(syntax, "Args");
                match self.symbol(&r.source, fun, context)? {
                    GoNameLookup::Predeclared {
                        name,
                        kind: GoSourceNameKind::BuiltinFunction,
                    } => {
                        if !matches!(name.as_str(), "len" | "min" | "max") {
                            return Err(GoConstantError::Unsupported);
                        }
                        Ok(Err(Plan::Builtin(
                            name,
                            args.iter().map(|&n| expression(n)).collect(),
                        )))
                    }
                    GoNameLookup::Predeclared {
                        kind: GoSourceNameKind::BuiltinType,
                        ..
                    }
                    | GoNameLookup::Original(_) => {
                        if args.len() != 1 {
                            return Err(GoConstantError::Type);
                        }
                        Ok(Err(Plan::Convert(
                            Box::new(self.scalar_type(&r.source, fun, context)?),
                            expression(args[0]),
                            true,
                        )))
                    }
                    _ => Err(GoConstantError::NotConstant),
                }
            }
            _ => Err(GoConstantError::Unsupported),
        }
    }
    fn finish(&self, plan: Plan) -> Result<GoEvaluatedConstant> {
        let value = |r: &Request| self.done.get(r).cloned().ok_or(GoConstantError::Type);
        match plan {
            Plan::Forward(r) => value(&r),
            Plan::Unary(op, r) => unary(&op, value(&r)?),
            Plan::Binary(op, a, b) => binary(&op, value(&a)?, value(&b)?),
            Plan::Convert(typ, r, explicit) => value(&r)?.convert(*typ, explicit),
            Plan::Builtin(name, args) => {
                if name == "len" {
                    if args.len() != 1 {
                        return Err(GoConstantError::Type);
                    }
                    let Value::Bytes(raw) = value(&args[0])?.value else {
                        return Err(GoConstantError::Unsupported);
                    };
                    return GoEvaluatedConstant::integer(
                        Integer::from_u64(
                            u64::try_from(raw.len()).map_err(|_| GoConstantError::Resource)?,
                        ),
                        false,
                    )
                    .convert(GoConstantScalarType::builtin("int")?, false);
                }
                min_max(&name, args.iter().map(value).collect::<Result<Vec<_>>>()?)
            }
        }
    }
    fn run(mut self, root: Request) -> Result<GoEvaluatedConstant> {
        let mut tasks = vec![Work::Visit(root.clone())];
        while let Some(task) = tasks.pop() {
            match task {
                Work::Visit(r) => {
                    if self.done.contains_key(&r) {
                        continue;
                    }
                    if !self.active.insert(r.clone()) {
                        return Err(GoConstantError::Cycle);
                    }
                    match self.plan(&r)? {
                        Ok(value) => {
                            self.active.remove(&r);
                            self.done.insert(r, value);
                        }
                        Err(plan) => {
                            let inputs = plan.inputs();
                            tasks.push(Work::Finish(r, plan));
                            for child in inputs.into_iter().rev() {
                                tasks.push(Work::Visit(child))
                            }
                        }
                    }
                }
                Work::Finish(r, plan) => {
                    let value = self.finish(plan)?;
                    self.active.remove(&r);
                    self.done.insert(r, value);
                }
            }
        }
        self.done.remove(&root).ok_or(GoConstantError::Type)
    }
}
/// Evaluate one actual original expression. Name lookup and implicit iota
/// context come from owned syntax, never caller-supplied values or callbacks.
/// Float/complex operations and aggregate-value constant builtins remain outside
/// this scalar reader; success is not whole-source type or program authority.
pub fn evaluate_go_constant(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoEvaluatedConstant> {
    let engine = Engine::new(scopes);
    let context = engine.context(source, node)?;
    engine.run(Request::expression(source, node, context))
}
/// Evaluate an actual const declaration member, including inferred/repeated
/// expression lists, current iota, inherited type annotations and exact scope.
pub fn evaluate_go_constant_binding(
    scopes: &GoSourceScopes,
    binding: &GoSourceNameBinding,
) -> Result<GoEvaluatedConstant> {
    if binding.kind() != GoSourceNameKind::Constant || !scopes.bindings().contains(binding) {
        return Err(GoConstantError::Locator);
    }
    Engine::new(scopes).run(Request::binding(binding))
}
