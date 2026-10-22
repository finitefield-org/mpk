//! Source-derived global error cases and static Root/Plain value properties.
//! Entry rows are explicit predicates, not native type facts or proof verdicts.
//! Capture authority, contract import, dispatch and runtime proofs are separate.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoErrorEntryStep {
    Field(usize),
    Elements,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoErrorEntryConstraint {
    root: GoSourceSelectionRef,
    input: usize,
    path: Vec<GoErrorEntryStep>,
    cases: Vec<GoSourceSelectionRef>,
}
impl GoErrorEntryConstraint {
    pub fn root(&self) -> &GoSourceSelectionRef { &self.root }
    pub fn input_position(&self) -> usize { self.input }
    pub fn path(&self) -> &[GoErrorEntryStep] { &self.path }
    pub fn cases(&self) -> &[GoSourceSelectionRef] { &self.cases }
    pub fn new(
        root: GoSourceSelectionRef,
        input: usize,
        path: Vec<GoErrorEntryStep>,
        cases: Vec<GoSourceSelectionRef>,
    ) -> Self {
        Self {
            root,
            input,
            path,
            cases,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoResolvedErrorEntryStep {
    owner: String,
    step: GoErrorEntryStep,
    capacity: bool,
}
impl GoResolvedErrorEntryStep {
    pub fn owner_type_id(&self) -> &str {
        &self.owner
    }
    pub fn step(&self) -> &GoErrorEntryStep {
        &self.step
    }
    pub fn uses_capacity(&self) -> bool {
        self.capacity
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBoundErrorEntry {
    root: GoSourceSelectionRef,
    input: usize,
    path: Vec<GoResolvedErrorEntryStep>,
    cases: Vec<String>,
}
impl GoBoundErrorEntry {
    pub fn root(&self) -> &GoSourceSelectionRef {
        &self.root
    }
    pub fn input_position(&self) -> usize {
        self.input
    }
    pub fn path(&self) -> &[GoResolvedErrorEntryStep] {
        &self.path
    }
    pub fn case_type_ids(&self) -> &[String] {
        &self.cases
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceValueProperties {
    typ: String,
    plain: bool,
    depth: u32,
    weight: u64,
    contains_error: bool,
}
impl GoSourceValueProperties {
    pub fn type_id(&self) -> &str {
        &self.typ
    }
    pub fn is_plain(&self) -> bool {
        self.plain
    }
    pub fn aggregate_depth(&self) -> u32 {
        self.depth
    }
    pub fn value_weight_units(&self) -> u64 {
        self.weight
    }
    pub fn contains_source_error(&self) -> bool {
        self.contains_error
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceDomainErrorKind {
    Type(GoSourceTypeError),
    Typing(GoScalarTypingError),
    Signature(GoSourceSignatureError),
    EntryTarget,
    DuplicateEntry,
    DuplicateCase,
    MissingEntry,
    Payload,
    NamedInterface,
    NonPlainBacking,
    Cycle,
    Depth,
    Weight,
    Directive { start: usize, end: usize },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceDomainError {
    kind: GoSourceDomainErrorKind,
    source: Option<GoSourceSelectionRef>,
    input: Option<usize>,
    path: Vec<GoErrorEntryStep>,
    typ: Option<String>,
}
impl GoSourceDomainError {
    pub fn kind(&self) -> &GoSourceDomainErrorKind {
        &self.kind
    }
    pub fn source(&self) -> Option<&GoSourceSelectionRef> {
        self.source.as_ref()
    }
    pub fn input_position(&self) -> Option<usize> {
        self.input
    }
    pub fn path(&self) -> &[GoErrorEntryStep] {
        &self.path
    }
    pub fn type_id(&self) -> Option<&str> {
        self.typ.as_deref()
    }
}
impl std::fmt::Display for GoSourceDomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source domain error: {:?}", self)
    }
}
impl std::error::Error for GoSourceDomainError {}
type DomainResult<T> = std::result::Result<T, GoSourceDomainError>;
fn failure(kind: GoSourceDomainErrorKind, typ: Option<&str>) -> GoSourceDomainError {
    GoSourceDomainError {
        kind,
        source: None,
        input: None,
        path: vec![],
        typ: typ.map(str::to_string),
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceValueDomain {
    selection: GoSourceSelection,
    entries: Vec<GoBoundErrorEntry>,
    cases: Vec<GoErrorImplementation>,
    signature_dependencies: Vec<GoSourceSelectionRef>,
    properties: BTreeMap<String, GoSourceValueProperties>,
    types: BTreeMap<String, GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
}
impl GoSourceValueDomain {
    pub fn selection(&self) -> &GoSourceSelection {
        &self.selection
    }
    pub fn entries(&self) -> &[GoBoundErrorEntry] {
        &self.entries
    }
    pub fn error_cases(&self) -> &[GoErrorImplementation] {
        &self.cases
    }
    /// Original Error declarations needed only for compatibility. Their bodies
    /// are not selected by an entry predicate or by concrete boxing.
    pub fn signature_dependencies(&self) -> &[GoSourceSelectionRef] {
        &self.signature_dependencies
    }
    pub fn properties(&self, id: &str) -> Option<&GoSourceValueProperties> {
        self.properties.get(id)
    }
    pub fn types(&self) -> impl Iterator<Item = &GoSourceTypeKey> {
        self.types.values()
    }
    pub fn underlying(&self, id: &str) -> Option<&str> {
        self.underlying.get(id).map(String::as_str)
    }
}
struct Domain<'a> {
    engine: Engine<'a>,
}
impl Domain<'_> {
    fn graph(&mut self, graph: &GoDerivedSourceTypes) -> DomainResult<()> {
        if let Some(root) = graph.roots().first() {
            self.engine
                .graph_type(graph, root)
                .map_err(|e| failure(GoSourceDomainErrorKind::Typing(e), None))?;
        }
        Ok(())
    }
    fn absorb(&mut self, typing: &GoValueTyping) -> DomainResult<()> {
        for key in typing.types() {
            self.engine
                .put_key(key.clone())
                .map_err(|e| failure(GoSourceDomainErrorKind::Typing(e), Some(key.id())))?;
            if let Some(base) = typing.underlying(key.id()) {
                self.engine.underlying.insert(key.id().into(), base.into());
            }
        }
        for d in typing.declarations() {
            self.engine.declarations.insert(d.id.clone(), d.clone());
        }
        for implementation in typing.error_implementations() {
            self.engine.error_implementations.insert(
                implementation.value_type_id().into(),
                Some(implementation.clone()),
            );
        }
        Ok(())
    }
    fn shape(&self, id: &str) -> DomainResult<&GoSourceTypeKey> {
        self.engine
            .shape(id)
            .map_err(|e| failure(GoSourceDomainErrorKind::Typing(e), Some(id)))
    }
    fn is_error(&self, id: &str) -> bool {
        self.engine.is_error(id)
    }
    fn case(&mut self, r: &GoSourceSelectionRef) -> DomainResult<String> {
        let graph =
            derive_go_source_type_declaration(self.engine.scopes, r.source_object_id(), r.node())
                .map_err(|e| {
                let mut f = failure(GoSourceDomainErrorKind::Type(e), None);
                f.source = Some(r.clone());
                f
            })?;
        let id = graph.roots()[0].clone();
        self.graph(&graph)?;
        Ok(id)
    }
    fn properties(
        &self,
        cases: &[String],
    ) -> DomainResult<BTreeMap<String, GoSourceValueProperties>> {
        let mut done: BTreeMap<String, GoSourceValueProperties> = BTreeMap::new();
        let mut active = BTreeSet::new();
        for root in self.engine.keys.keys() {
            let mut work = vec![(root.clone(), false)];
            while let Some((id, finish)) = work.pop() {
                if done.contains_key(&id) {
                    continue;
                }
                let key = self.engine.keys.get(&id).ok_or_else(|| {
                    failure(
                        GoSourceDomainErrorKind::Typing(GoScalarTypingError::Type),
                        Some(&id),
                    )
                })?;
                let children: Vec<String> = match key {
                    GoSourceTypeKey::Nominal { .. } => vec![self
                        .engine
                        .underlying
                        .get(&id)
                        .ok_or_else(|| failure(GoSourceDomainErrorKind::Payload, Some(&id)))?
                        .clone()],
                    GoSourceTypeKey::Builtin { name, .. } if name == "error" => cases.to_vec(),
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => vec![element.clone()],
                    GoSourceTypeKey::Struct { fields, .. } => {
                        fields.iter().map(|f| f.type_id.clone()).collect()
                    }
                    _ => vec![],
                };
                if !finish {
                    if !active.insert(id.clone()) {
                        return Err(failure(GoSourceDomainErrorKind::Cycle, Some(&id)));
                    }
                    work.push((id.clone(), true));
                    for child in children.iter().rev() {
                        work.push((child.clone(), false));
                    }
                    continue;
                }
                active.remove(&id);
                let values = children.iter().map(|id| &done[id]).collect::<Vec<_>>();
                let cap = 2_097_153u64;
                let sum = || {
                    values
                        .iter()
                        .fold(1u64, |n, p| n.saturating_add(p.weight).min(cap))
                };
                let (plain, depth, weight, contains_error) = match key {
                    GoSourceTypeKey::Nominal { .. } => {
                        if matches!(self.shape(&id)?,GoSourceTypeKey::Builtin{name,..}if name=="error")
                        {
                            return Err(failure(
                                GoSourceDomainErrorKind::NamedInterface,
                                Some(&id),
                            ));
                        }
                        let p = values[0];
                        (p.plain, p.depth, p.weight, p.contains_error)
                    }
                    GoSourceTypeKey::Builtin { name, .. } if name == "error" => (
                        values.iter().all(|p| p.plain),
                        1 + values.iter().map(|p| p.depth).max().unwrap_or(0),
                        sum(),
                        true,
                    ),
                    GoSourceTypeKey::Builtin { name, .. } => {
                        let weight = if name == "string" { 257 } else { 1 };
                        (true, 0, weight, false)
                    }
                    GoSourceTypeKey::Array { length, .. } => {
                        let p = values[0];
                        (
                            p.plain,
                            p.depth + 1,
                            1u64.saturating_add(length.saturating_mul(p.weight))
                                .min(cap),
                            p.contains_error,
                        )
                    }
                    GoSourceTypeKey::Slice { .. } => {
                        let p = values[0];
                        if !p.plain {
                            return Err(failure(
                                GoSourceDomainErrorKind::NonPlainBacking,
                                Some(&id),
                            ));
                        }
                        (false, p.depth + 1, 37, p.contains_error)
                    }
                    GoSourceTypeKey::Struct { .. } => (
                        values.iter().all(|p| p.plain),
                        1 + values.iter().map(|p| p.depth).max().unwrap_or(0),
                        sum(),
                        values.iter().any(|p| p.contains_error),
                    ),
                };
                if depth > 16 {
                    return Err(failure(GoSourceDomainErrorKind::Depth, Some(&id)));
                }
                if weight > 2_097_152 {
                    return Err(failure(GoSourceDomainErrorKind::Weight, Some(&id)));
                }
                done.insert(
                    id.clone(),
                    GoSourceValueProperties {
                        typ: id,
                        plain,
                        depth,
                        weight,
                        contains_error,
                    },
                );
            }
        }
        Ok(done)
    }
}
/// Derive the union of selected body cases and complete explicit entry cases.
/// Input positions include the value receiver first; field ordinals and element
/// steps follow original types and normalize to the frozen typed-path shapes.
/// This does not validate an overlay/capture context or discharge entry/runtime
/// obligations, and it does not implement finite Error dispatch.
pub fn derive_go_source_value_domain(
    selection: &GoSourceSelection,
    entries: &[GoErrorEntryConstraint],
) -> DomainResult<GoSourceValueDomain> {
    let scopes = selection.scopes();
    let mut domain = Domain {
        engine: Engine::new(scopes),
    };
    let mut inputs = BTreeMap::new();
    for d in selection.declarations() {
        if d.roles().all(|r| r == GoSourceSelectionRole::TypeCheckOnly) {
            continue;
        }
        let r = d.locator();
        if d.kind() == "function" {
            let signature = derive_go_source_signature(scopes, r.source_object_id(), r.node())
                .map_err(|e| failure(GoSourceDomainErrorKind::Signature(e), None))?;
            domain.graph(signature.types())?;
            if selection.roots().contains(r) {
                inputs.insert(
                    r.clone(),
                    signature
                        .slots()
                        .iter()
                        .filter(|s| s.role() != GoSourceSignatureRole::Result)
                        .map(|s| s.type_id().to_string())
                        .collect::<Vec<_>>(),
                );
            }
            if d.roles().any(|role| {
                matches!(
                    role,
                    GoSourceSelectionRole::SelectedRoot | GoSourceSelectionRole::BodyDependency
                )
            }) {
                let body = check_go_function_body(scopes, r.source_object_id(), r.node()).map_err(
                    |e| {
                        failure(
                            GoSourceDomainErrorKind::Typing(match e.kind() {
                                GoBodyErrorKind::Typing(t) => *t,
                                _ => GoScalarTypingError::Unsupported,
                            }),
                            None,
                        )
                    },
                )?;
                domain.absorb(body.typing())?;
            }
        } else if d.kind() == "type" {
            let graph = derive_go_source_type_declaration(scopes, r.source_object_id(), r.node())
                .map_err(|e| failure(GoSourceDomainErrorKind::Type(e), None))?;
            domain.graph(&graph)?;
        }
    }
    let mut supplied = BTreeMap::new();
    let mut universe = selection
        .body_boxed_error_type_ids()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for row in entries {
        if !inputs
            .get(&row.root)
            .is_some_and(|types| row.input < types.len())
        {
            let mut e = failure(GoSourceDomainErrorKind::EntryTarget, None);
            e.source = Some(row.root.clone());
            e.input = Some(row.input);
            return Err(e);
        }
        let mut cases = BTreeSet::new();
        for r in &row.cases {
            let id = domain.case(r)?;
            if !cases.insert(id.clone()) {
                return Err(failure(GoSourceDomainErrorKind::DuplicateCase, Some(&id)));
            }
        }
        universe.extend(cases.iter().cloned());
        if supplied
            .insert(
                (row.root.clone(), row.input, row.path.clone()),
                cases.into_iter().collect::<Vec<_>>(),
            )
            .is_some()
        {
            return Err(failure(GoSourceDomainErrorKind::DuplicateEntry, None));
        }
    }
    let universe = universe.into_iter().collect::<Vec<_>>();
    let mut implementations = vec![];
    for id in &universe {
        if !matches!(
            domain.engine.keys.get(id),
            Some(GoSourceTypeKey::Nominal { .. })
        ) {
            return Err(failure(GoSourceDomainErrorKind::Payload, Some(id)));
        }
        let shape = domain.shape(id)?;
        if matches!(
            shape,
            GoSourceTypeKey::Array { .. } | GoSourceTypeKey::Slice { .. }
        ) || matches!(shape,GoSourceTypeKey::Builtin{name,..}if matches!(name.as_str(),"string"|"error"))
        {
            return Err(failure(GoSourceDomainErrorKind::Payload, Some(id)));
        }
        domain
            .engine
            .ensure_error_implementation(id)
            .map_err(|e| failure(GoSourceDomainErrorKind::Typing(e), Some(id)))?;
        let implementation = domain
            .engine
            .error_implementations
            .get(id)
            .cloned()
            .flatten()
            .ok_or_else(|| failure(GoSourceDomainErrorKind::Payload, Some(id)))?;
        if !implementation.method().receiver_path().is_empty() {
            return Err(failure(GoSourceDomainErrorKind::Payload, Some(id)));
        }
        let method = implementation.method();
        let locator = GoSourceSelectionRef::new(
            method
                .declaration_source_object_id()
                .ok_or_else(|| failure(GoSourceDomainErrorKind::Payload, Some(id)))?,
            method
                .declaration_node()
                .ok_or_else(|| failure(GoSourceDomainErrorKind::Payload, Some(id)))?,
        );
        if let Some((start, end)) = selection::compiler_directive(scopes, &locator) {
            let mut e = failure(GoSourceDomainErrorKind::Directive { start, end }, Some(id));
            e.source = Some(locator);
            return Err(e);
        }
        implementations.push(implementation);
    }
    let properties = domain.properties(&universe)?;
    let mut bound = vec![];
    for (root, types) in &inputs {
        for (input, typ) in types.iter().enumerate() {
            let mut work = vec![(
                typ.clone(),
                Vec::<GoErrorEntryStep>::new(),
                Vec::<GoResolvedErrorEntryStep>::new(),
            )];
            while let Some((id, path, resolved)) = work.pop() {
                if !properties[&id].contains_error {
                    continue;
                }
                if domain.is_error(&id) {
                    let cases = supplied
                        .remove(&(root.clone(), input, path.clone()))
                        .ok_or_else(|| GoSourceDomainError {
                            kind: GoSourceDomainErrorKind::MissingEntry,
                            source: Some(root.clone()),
                            input: Some(input),
                            path: path.clone(),
                            typ: Some(id.clone()),
                        })?;
                    bound.push(GoBoundErrorEntry {
                        root: root.clone(),
                        input,
                        path: resolved,
                        cases,
                    });
                    continue;
                }
                match domain.shape(&id)?.clone() {
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => {
                        let capacity = matches!(domain.shape(&id)?, GoSourceTypeKey::Slice { .. });
                        let mut path = path;
                        path.push(GoErrorEntryStep::Elements);
                        let mut resolved = resolved;
                        resolved.push(GoResolvedErrorEntryStep {
                            owner: id,
                            step: GoErrorEntryStep::Elements,
                            capacity,
                        });
                        work.push((element.clone(), path, resolved));
                    }
                    GoSourceTypeKey::Struct { fields, .. } => {
                        for (ordinal, field) in fields.iter().enumerate().rev() {
                            let mut path = path.clone();
                            path.push(GoErrorEntryStep::Field(ordinal));
                            let mut resolved = resolved.clone();
                            resolved.push(GoResolvedErrorEntryStep {
                                owner: id.clone(),
                                step: GoErrorEntryStep::Field(ordinal),
                                capacity: false,
                            });
                            work.push((field.type_id.clone(), path, resolved));
                        }
                    }
                    _ => return Err(failure(GoSourceDomainErrorKind::EntryTarget, Some(&id))),
                }
            }
        }
    }
    if let Some(((root, input, path), _)) = supplied.into_iter().next() {
        return Err(GoSourceDomainError {
            kind: GoSourceDomainErrorKind::EntryTarget,
            source: Some(root),
            input: Some(input),
            path,
            typ: None,
        });
    }
    let signature_dependencies = implementations
        .iter()
        .map(|i| {
            GoSourceSelectionRef::new(
                i.method().declaration_source_object_id().unwrap(),
                i.method().declaration_node().unwrap(),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(GoSourceValueDomain {
        selection: selection.clone(),
        entries: bound,
        cases: implementations,
        signature_dependencies,
        properties,
        types: domain.engine.keys,
        underlying: domain.engine.underlying,
    })
}
