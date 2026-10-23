//! Independent scalar/aggregate value typing over owned original syntax.
//! Header argument/result checks do not check callee bodies or invocation state.
use super::*;
mod aggregates;
mod assignments;
mod bodies;
mod domains;
pub(super) use domains::extend_contract_domain;
mod error_boxings;
mod execution;
pub use execution::{execute_go_source, GoSourceRuntimeValue, GoSourceRuntimeData, GoSourceRuntimeSlice, GoSourceRuntimeFault, GoSourceExecution, GoSourceExecutionStatus, GoSourceExecutionEvent, GoSourceExecutionEventKind, GoSourceExecutionError, GoSourceExecutionErrorKind};
mod literals;
mod members;
mod methods;
mod operators;
mod returns;
mod selection;
pub(super) use selection::{derive_go_source_error_module_selection, GoSourceModuleSelectionError};
mod slice_builtins;
pub use assignments::{check_go_assignment_statement, GoAssignmentTarget, GoAssignmentTyping};
pub use bodies::{
    check_go_function_body, GoBodyBranch, GoBodyConstant, GoBodyErrorKind, GoBodyExpression,
    GoBodyStatement, GoBodyTyping, GoBodyTypingError,
};
pub use domains::{
    derive_go_source_value_domain, GoBoundErrorEntry, GoErrorEntryConstraint, GoErrorEntryStep,
    GoResolvedErrorEntryStep, GoSourceDomainError, GoSourceDomainErrorKind, GoSourceValueDomain,
    GoSourceValueProperties,
};
pub use error_boxings::{GoErrorBoxing, GoErrorBoxingProjection};
pub use methods::{GoErrorImplementation, GoMethodCallHeader, GoSourceMethod};
use operators::comparison;
pub use returns::{check_go_return_statement, GoReturnTyping};
pub use selection::{
    derive_go_source_error_selection, derive_go_source_selection, GoSourceErrorCall,
    GoSourceErrorDispatch, GoSourceErrorSelection, GoSourceErrorSelectionError, GoSourceSelection,
    GoSourceSelectionCall, GoSourceSelectionDeclaration, GoSourceSelectionDependency,
    GoSourceSelectionError, GoSourceSelectionErrorKind, GoSourceSelectionRef,
    GoSourceSelectionRole,
};
pub use slice_builtins::GoSliceBuiltinHeader;
mod string_operations;
pub use string_operations::{
    derive_go_source_string_operations, GoSourceStringOperation, GoSourceStringOperationKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoScalarTypingError {
    Locator,
    Name,
    Unsupported,
    Type,
    Limit,
    Arity,
    Cycle,
    Constant(GoConstantError),
    SourceType(GoSourceTypeError),
    Signature(GoSourceSignatureError),
}
impl std::fmt::Display for GoScalarTypingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go scalar typing error: {self:?}")
    }
}
impl std::error::Error for GoScalarTypingError {}
type Result<T> = std::result::Result<T, GoScalarTypingError>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceScalarValue {
    typ: GoConstantScalarType,
    constant: Option<GoEvaluatedConstant>,
    addressable: bool,
}
impl GoSourceScalarValue {
    pub fn scalar_type(&self) -> &GoConstantScalarType {
        &self.typ
    }
    pub fn constant(&self) -> Option<&GoEvaluatedConstant> {
        self.constant.as_ref()
    }
    pub fn addressable(&self) -> bool {
        self.addressable
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoScalarCallHeader {
    call_source: String,
    call_node: usize,
    declaration_source: String,
    declaration_node: usize,
    body_node: Option<usize>,
    parameters: Vec<String>,
    results: Vec<String>,
}
impl GoScalarCallHeader {
    pub fn call_source_object_id(&self) -> &str {
        &self.call_source
    }
    pub fn call_node(&self) -> usize {
        self.call_node
    }
    pub fn declaration_source_object_id(&self) -> &str {
        &self.declaration_source
    }
    pub fn declaration_node(&self) -> usize {
        self.declaration_node
    }
    pub fn body_node(&self) -> Option<usize> {
        self.body_node
    }
    pub fn parameter_type_ids(&self) -> &[String] {
        &self.parameters
    }
    pub fn result_type_ids(&self) -> &[String] {
        &self.results
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoScalarTyping {
    scopes: std::sync::Arc<GoSourceScopes>,
    values: Vec<GoSourceScalarValue>,
    calls: Vec<GoScalarCallHeader>,
    slice_builtins: Vec<GoSliceBuiltinHeader>,
    method_calls: Vec<GoMethodCallHeader>,
    error_implementations: Vec<GoErrorImplementation>,
    error_boxings: Vec<GoErrorBoxing>,
}
impl GoScalarTyping {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.scopes.source(id)
    }
    pub fn values(&self) -> &[GoSourceScalarValue] {
        &self.values
    }
    pub fn call_headers(&self) -> &[GoScalarCallHeader] {
        &self.calls
    }
    pub fn slice_builtin_headers(&self) -> &[GoSliceBuiltinHeader] {
        &self.slice_builtins
    }
    pub fn method_call_headers(&self) -> &[GoMethodCallHeader] {
        &self.method_calls
    }
    /// Potential uses observed by this value engine, including dependencies.
    /// Separate constant-reader/type-annotation checks are not a complete site list.
    pub fn error_boxings(&self) -> &[GoErrorBoxing] {
        &self.error_boxings
    }
    pub fn error_implementations(&self) -> &[GoErrorImplementation] {
        &self.error_implementations
    }
}
/// Original value-type facts. No runtime payload, store permission or body proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceTypedValue {
    typ: GoSourceTypeKey,
    scalar: Option<GoConstantScalarType>,
    constant: Option<GoEvaluatedConstant>,
    addressable: bool,
    untyped_nil: bool,
}
impl GoSourceTypedValue {
    /// Value type after contextual materialization. An untyped nil expression
    /// retains its separate marker and uses the actual expected destination.
    pub fn source_type_key(&self) -> &GoSourceTypeKey {
        &self.typ
    }
    pub fn scalar_type(&self) -> Option<&GoConstantScalarType> {
        self.scalar.as_ref()
    }
    pub fn constant(&self) -> Option<&GoEvaluatedConstant> {
        self.constant.as_ref()
    }
    pub fn addressable(&self) -> bool {
        self.addressable
    }
    pub fn is_untyped_nil(&self) -> bool {
        self.untyped_nil
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoValueTyping {
    scopes: std::sync::Arc<GoSourceScopes>,
    values: Vec<GoSourceTypedValue>,
    calls: Vec<GoScalarCallHeader>,
    slice_builtins: Vec<GoSliceBuiltinHeader>,
    method_calls: Vec<GoMethodCallHeader>,
    error_implementations: Vec<GoErrorImplementation>,
    error_boxings: Vec<GoErrorBoxing>,
    types: BTreeMap<String, GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
    declarations: BTreeMap<String, GoTypeDeclarationKey>,
}
impl GoValueTyping {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.scopes.source(id)
    }
    pub fn values(&self) -> &[GoSourceTypedValue] {
        &self.values
    }
    pub fn call_headers(&self) -> &[GoScalarCallHeader] {
        &self.calls
    }
    pub fn slice_builtin_headers(&self) -> &[GoSliceBuiltinHeader] {
        &self.slice_builtins
    }
    pub fn method_call_headers(&self) -> &[GoMethodCallHeader] {
        &self.method_calls
    }
    /// Potential uses observed by this value engine, including dependencies.
    /// Separate constant-reader/type-annotation checks are not a complete site list.
    pub fn error_boxings(&self) -> &[GoErrorBoxing] {
        &self.error_boxings
    }
    pub fn error_implementations(&self) -> &[GoErrorImplementation] {
        &self.error_implementations
    }
    pub fn types(&self) -> impl Iterator<Item = &GoSourceTypeKey> {
        self.types.values()
    }
    pub fn underlying(&self, id: &str) -> Option<&str> {
        self.underlying.get(id).map(String::as_str)
    }
    pub fn declarations(&self) -> impl Iterator<Item = &GoTypeDeclarationKey> {
        self.declarations.values()
    }
    fn scalar_only(self) -> Result<GoScalarTyping> {
        let mut values = vec![];
        for value in self.values {
            values.push(GoSourceScalarValue {
                typ: value.scalar.ok_or(GoScalarTypingError::Unsupported)?,
                constant: value.constant,
                addressable: value.addressable,
            });
        }
        Ok(GoScalarTyping {
            scopes: self.scopes,
            values,
            calls: self.calls,
            slice_builtins: self.slice_builtins,
            method_calls: self.method_calls,
            error_implementations: self.error_implementations,
            error_boxings: self.error_boxings,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Mode {
    Raw,
    Value(Option<String>),
    Fixed(String),
    Element(String),
    Index,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Target {
    Expression(usize, Mode),
    Binding(usize),
    Declaration(usize),
    Assignment(usize),
    AssignmentOperation(usize),
    Return(usize),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Request {
    source: String,
    target: Target,
}
impl Request {
    fn index(source: &str, node: usize) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node, Mode::Index),
        }
    }
    fn element(source: &str, node: usize, typ: String) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node, Mode::Element(typ)),
        }
    }
    fn raw(source: &str, node: usize) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node, Mode::Raw),
        }
    }
    fn value(source: &str, node: usize, typ: Option<String>) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node, Mode::Value(typ)),
        }
    }
    fn fixed(source: &str, node: usize, typ: String) -> Self {
        Self {
            source: source.into(),
            target: Target::Expression(node, Mode::Fixed(typ)),
        }
    }
    fn binding(b: &GoSourceNameBinding, index: usize) -> Self {
        Self {
            source: b.source_object_id().into(),
            target: Target::Binding(index),
        }
    }
    fn declaration(source: &str, node: usize) -> Self {
        Self {
            source: source.into(),
            target: Target::Declaration(node),
        }
    }
    fn expression(&self) -> Result<usize> {
        match self.target {
            Target::Expression(n, _) => Ok(n),
            _ => Err(GoScalarTypingError::Locator),
        }
    }
}
#[derive(Clone, Debug)]
struct Atom {
    typ: Option<String>,
    kind: Option<GoConstantKind>,
    nil: bool,
    value: Option<GoEvaluatedConstant>,
    addressable: bool,
}
type Facts = Vec<Atom>;
#[derive(Clone)]
enum Destination {
    Inferred,
    Declared(String),
    Reused(Request),
}
enum Plan {
    ReturnFirst(Vec<String>, Vec<Request>),
    ReturnReady(Vec<String>, Vec<Request>),
    AssignmentFirst(Vec<Option<Request>>, Vec<Request>),
    AssignmentReady(Vec<Option<Request>>, Vec<Request>),
    ShortAssignment(Request),
    Compound(String, Request, Request),
    Increment(Request),
    MethodReceiver(Request, usize, Vec<Request>),
    MethodFirst(Box<GoMethodCallHeader>, Vec<Request>),
    MethodReady(Box<GoMethodCallHeader>, Vec<Request>),
    Make(String, Vec<Request>),
    AppendFirst(Vec<Request>, bool),
    AppendReady(String, Vec<Request>, bool),
    Copy(Vec<Request>),
    IndexValue(Request),
    Forward(Request),
    Select(Request, usize),
    Materialize(Request, Option<String>),
    Unary(String, Request),
    BinaryFirst(String, Request, Request),
    BinaryReady(String, Request, Request),
    Cast(String, Request),
    BuiltinFirst(String, Vec<Request>),
    BuiltinReady(String, Vec<Request>),
    CallFirst(Box<GoScalarCallHeader>, Vec<Request>),
    CallReady(Box<GoScalarCallHeader>, Vec<Request>),
    DeclarationFirst(Vec<Destination>, Vec<Request>),
    DeclarationReady(Vec<Destination>, Vec<Request>),
    Field(Request, String),
    Index(Request, Request),
    Slice(Request, Box<[Option<Request>; 3]>, bool),
    Literal(String, Vec<Request>),
    Range(Request, usize),
}
impl Plan {
    fn inputs(&self) -> Vec<Request> {
        match self {
            Self::ReturnFirst(_, args) | Self::ReturnReady(_, args) => args.clone(),
            Self::AssignmentFirst(lhs, rhs) | Self::AssignmentReady(lhs, rhs) => lhs
                .iter()
                .flatten()
                .cloned()
                .chain(rhs.iter().cloned())
                .collect(),
            Self::ShortAssignment(r) | Self::Increment(r) => vec![r.clone()],
            Self::Compound(_, a, b) => vec![a.clone(), b.clone()],
            Self::Forward(r)
            | Self::IndexValue(r)
            | Self::Select(r, _)
            | Self::Materialize(r, _)
            | Self::Unary(_, r)
            | Self::Cast(_, r) => vec![r.clone()],
            Self::Field(r, _) | Self::Range(r, _) => vec![r.clone()],
            Self::MethodReceiver(r, _, _) => vec![r.clone()],
            Self::Index(a, b) => vec![a.clone(), b.clone()],
            Self::Slice(r, bounds, _) => std::iter::once(r.clone())
                .chain(bounds.iter().flatten().cloned())
                .collect(),
            Self::Literal(_, rs) => rs.clone(),
            Self::BinaryFirst(_, a, b) | Self::BinaryReady(_, a, b) => vec![a.clone(), b.clone()],
            Self::BuiltinFirst(_, rs)
            | Self::BuiltinReady(_, rs)
            | Self::Make(_, rs)
            | Self::AppendFirst(rs, _)
            | Self::AppendReady(_, rs, _)
            | Self::Copy(rs)
            | Self::MethodFirst(_, rs)
            | Self::MethodReady(_, rs)
            | Self::CallFirst(_, rs)
            | Self::CallReady(_, rs) => rs.clone(),
            Self::DeclarationFirst(dest, rs) | Self::DeclarationReady(dest, rs) => rs
                .iter()
                .cloned()
                .chain(dest.iter().filter_map(|d| {
                    if let Destination::Reused(r) = d {
                        Some(r.clone())
                    } else {
                        None
                    }
                }))
                .collect(),
        }
    }
}
enum Step {
    Done(Facts),
    More(Plan),
}
enum Work {
    Visit(Request),
    Finish(Request, Box<Plan>),
}
struct Engine<'a> {
    scopes: &'a GoSourceScopes,
    types: BTreeMap<String, GoConstantScalarType>,
    keys: BTreeMap<String, GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
    declarations: BTreeMap<String, GoTypeDeclarationKey>,
    done: BTreeMap<Request, Facts>,
    active: BTreeSet<Request>,
    calls: BTreeSet<GoScalarCallHeader>,
    slice_builtins: BTreeSet<GoSliceBuiltinHeader>,
    method_calls: BTreeSet<GoMethodCallHeader>,
    method_packages: methods::MethodPackages,
    method_receivers: BTreeMap<(String, usize), Option<(String, bool)>>,
    error_implementations: BTreeMap<String, Option<GoErrorImplementation>>,
    error_boxings: BTreeSet<GoErrorBoxing>,
    function_owners: BTreeMap<String, Vec<Option<usize>>>,
    return_signatures: BTreeMap<(String, usize), GoSourceSignature>,
    compilation_reads: source_dependencies::Reads,
}
fn attr<'a>(n: &'a GoSyntaxNode, key: &str) -> Option<&'a str> {
    n.attributes().find_map(|(k, v)| (k == key).then_some(v))
}
fn list<'a>(n: &'a GoSyntaxNode, role: &str) -> &'a [usize] {
    n.edges()
        .iter()
        .find(|e| e.role() == role)
        .map_or(&[], |e| e.targets())
}
fn child(n: &GoSyntaxNode, role: &str) -> Result<usize> {
    let xs = list(n, role);
    if xs.len() == 1 {
        Ok(xs[0])
    } else {
        Err(GoScalarTypingError::Locator)
    }
}
fn kind(typ: &GoConstantScalarType) -> GoConstantKind {
    match typ.underlying_builtin() {
        "bool" => GoConstantKind::Bool,
        "string" => GoConstantKind::String,
        _ => GoConstantKind::Integer,
    }
}
fn constant_error(error: GoConstantError) -> GoScalarTypingError {
    if error == GoConstantError::Unsupported {
        GoScalarTypingError::Unsupported
    } else {
        GoScalarTypingError::Constant(error)
    }
}
impl<'a> Engine<'a> {
    fn new(scopes: &'a GoSourceScopes) -> Self {
        Self {
            scopes,
            types: BTreeMap::new(),
            keys: BTreeMap::new(),
            underlying: BTreeMap::new(),
            declarations: BTreeMap::new(),
            done: BTreeMap::new(),
            active: BTreeSet::new(),
            calls: BTreeSet::new(),
            slice_builtins: BTreeSet::new(),
            method_calls: BTreeSet::new(),
            method_packages: BTreeMap::new(),
            method_receivers: BTreeMap::new(),
            error_implementations: BTreeMap::new(),
            error_boxings: BTreeSet::new(),
            function_owners: BTreeMap::new(),
            return_signatures: BTreeMap::new(),
            compilation_reads: source_dependencies::Reads::new(),
        }
    }
    fn node(&self, source: &str, node: usize) -> Result<&GoSyntaxNode> {
        self.scopes
            .source(source)
            .and_then(|s| s.nodes().get(node))
            .ok_or(GoScalarTypingError::Locator)
    }
    fn intern(&mut self, typ: GoConstantScalarType) -> Result<String> {
        let id = typ.source_type_key().id().to_string();
        self.put_key(typ.source_type_key().clone())?;
        if let Some(decl) = typ.declaration() {
            self.declarations.insert(decl.id.clone(), decl.clone());
            let base =
                GoConstantScalarType::builtin(typ.underlying_builtin()).map_err(constant_error)?;
            self.put_key(base.source_type_key().clone())?;
            self.underlying
                .insert(id.clone(), base.source_type_key().id().to_string());
        }
        if let Some(prior) = self.types.insert(id.clone(), typ.clone()) {
            if prior != typ {
                return Err(GoScalarTypingError::Type);
            }
        }
        self.ensure_error_implementation(&id)?;
        Ok(id)
    }
    fn builtin(&mut self, name: &str) -> Result<String> {
        self.intern(GoConstantScalarType::builtin(name).map_err(constant_error)?)
    }
    fn annotation(&mut self, source: &str, node: usize) -> Result<String> {
        let graph = derive_go_source_type(self.scopes, source, node)
            .map_err(GoScalarTypingError::SourceType)?;
        self.graph_type(&graph, &graph.roots()[0])
    }
    fn graph_type(&mut self, graph: &GoDerivedSourceTypes, id: &str) -> Result<String> {
        self.compilation_reads
            .extend(graph.compilation_reads.clone());
        for key in graph.types() {
            self.put_key(key.clone())?;
            if let Some(base) = graph.underlying(key.id()) {
                self.underlying.insert(key.id().into(), base.into());
            }
            if let Ok(typ) = GoConstantScalarType::from_source_graph(graph, key.id()) {
                self.types.insert(key.id().into(), typ);
            }
        }
        for decl in graph.declarations() {
            self.declarations.insert(decl.id.clone(), decl.clone());
        }
        for key in graph.types() {
            self.ensure_error_implementation(key.id())?;
        }
        Ok(id.into())
    }
    fn atom(&self, id: &str, addressable: bool) -> Atom {
        Atom {
            typ: Some(id.into()),
            kind: self.types.get(id).map(kind),
            nil: false,
            value: None,
            addressable,
        }
    }
    fn constant(&mut self, value: GoEvaluatedConstant) -> Result<Atom> {
        let typ = value
            .scalar_type()
            .cloned()
            .map(|t| self.intern(t))
            .transpose()?;
        Ok(Atom {
            typ,
            kind: Some(value.kind()),
            nil: false,
            value: Some(value),
            addressable: false,
        })
    }
    fn exact_constant(&mut self, source: &str, node: usize) -> Result<GoEvaluatedConstant> {
        let (value, reads) =
            source_dependencies::expression(self.scopes, source, node).map_err(constant_error)?;
        self.compilation_reads.extend(reads);
        Ok(value)
    }
    fn exact_binding(&mut self, binding: &GoSourceNameBinding) -> Result<GoEvaluatedConstant> {
        let (value, reads) =
            source_dependencies::binding(self.scopes, binding).map_err(constant_error)?;
        self.compilation_reads.extend(reads);
        Ok(value)
    }
    fn facts(&self, r: &Request) -> Result<Facts> {
        self.done.get(r).cloned().ok_or(GoScalarTypingError::Type)
    }
    fn one(&self, r: &Request) -> Result<Atom> {
        let v = self.facts(r)?;
        if v.len() == 1 {
            Ok(v[0].clone())
        } else {
            Err(GoScalarTypingError::Arity)
        }
    }
    fn symbol(&self, source: &str, mut node: usize) -> Result<GoNameLookup> {
        loop {
            let n = self.node(source, node)?;
            match n.kind() {
                "ParenExpr" => node = child(n, "X")?,
                "Ident" => {
                    return self
                        .scopes
                        .lookup_identifier(source, node)
                        .map_err(|_| GoScalarTypingError::Name)?
                        .ok_or(GoScalarTypingError::Name)
                }
                "SelectorExpr" => {
                    let base = child(n, "X")?;
                    if self.node(source, base)?.kind() != "Ident" {
                        return Err(GoScalarTypingError::Unsupported);
                    }
                    let Some(GoNameLookup::Original(package)) = self
                        .scopes
                        .lookup_identifier(source, base)
                        .map_err(|_| GoScalarTypingError::Name)?
                    else {
                        return Err(GoScalarTypingError::Unsupported);
                    };
                    if package.kind() != GoSourceNameKind::Import {
                        return Err(GoScalarTypingError::Unsupported);
                    }
                    let name = attr(self.node(source, child(n, "Sel")?)?, "Name")
                        .ok_or(GoScalarTypingError::Locator)?;
                    return self
                        .scopes
                        .imported_member(
                            package.import_path().ok_or(GoScalarTypingError::Name)?,
                            name,
                        )
                        .map_err(|_| GoScalarTypingError::Name)?
                        .map(GoNameLookup::Original)
                        .ok_or(GoScalarTypingError::Name);
                }
                _ => return Err(GoScalarTypingError::Unsupported),
            }
        }
    }
    fn binding_index(&self, b: &GoSourceNameBinding) -> Result<usize> {
        self.scopes
            .bindings()
            .iter()
            .position(|x| x == b)
            .ok_or(GoScalarTypingError::Locator)
    }
    fn header(
        &mut self,
        source: &str,
        node: usize,
        b: &GoSourceNameBinding,
    ) -> Result<GoScalarCallHeader> {
        let signature =
            derive_go_source_signature(self.scopes, b.source_object_id(), b.declaration_node())
                .map_err(GoScalarTypingError::Signature)?;
        let mut parameters = vec![];
        let mut results = vec![];
        for slot in signature.slots() {
            let typ = self.graph_type(signature.types(), slot.type_id())?;
            match slot.role() {
                GoSourceSignatureRole::Parameter => parameters.push(typ),
                GoSourceSignatureRole::Result => results.push(typ),
                GoSourceSignatureRole::Receiver => return Err(GoScalarTypingError::Unsupported),
            }
        }
        Ok(GoScalarCallHeader {
            call_source: source.into(),
            call_node: node,
            declaration_source: b.source_object_id().into(),
            declaration_node: b.declaration_node(),
            body_node: signature.body_node(),
            parameters,
            results,
        })
    }
    fn declaration(&mut self, source: &str, node: usize) -> Result<Plan> {
        let n = self.node(source, node)?.clone();
        let names = if n.kind() == "ValueSpec" {
            list(&n, "Names")
        } else if n.kind() == "AssignStmt" && attr(&n, "Tok") == Some(":=") {
            list(&n, "Lhs")
        } else {
            return Err(GoScalarTypingError::Unsupported);
        };
        let rhs = if n.kind() == "ValueSpec" {
            list(&n, "Values")
        } else {
            list(&n, "Rhs")
        };
        let annotation = list(&n, "Type")
            .first()
            .copied()
            .map(|typ| self.annotation(source, typ))
            .transpose()?;
        let mut targets = vec![];
        for &name in names {
            if self.node(source, name)?.kind() != "Ident" {
                return Err(GoScalarTypingError::Type);
            }
            let target = if let Some(t) = &annotation {
                Destination::Declared(t.clone())
            } else if n.kind() == "ValueSpec"
                || attr(self.node(source, name)?, "Name") == Some("_")
                || self.scopes.bindings().iter().any(|b| {
                    b.source_object_id() == source
                        && b.declaration_node() == node
                        && b.name_node() == name
                })
            {
                Destination::Inferred
            } else {
                let GoNameLookup::Original(b) = self.symbol(source, name)? else {
                    return Err(GoScalarTypingError::Type);
                };
                if b.kind() != GoSourceNameKind::Variable {
                    return Err(GoScalarTypingError::Type);
                }
                Destination::Reused(Request::binding(&b, self.binding_index(&b)?))
            };
            targets.push(target);
        }
        if rhs.is_empty() {
            return Err(GoScalarTypingError::Arity);
        }
        Ok(Plan::DeclarationFirst(
            targets,
            rhs.iter().map(|&node| Request::raw(source, node)).collect(),
        ))
    }
    fn plan(&mut self, r: &Request) -> Result<Step> {
        match &r.target {
            Target::Return(node) => self.return_plan(&r.source, *node),
            Target::Assignment(node) => self.assignment_plan(&r.source, *node),
            Target::AssignmentOperation(node) => self.assignment_operation(&r.source, *node),
            Target::Binding(index) => {
                let b = self
                    .scopes
                    .bindings()
                    .get(*index)
                    .filter(|b| {
                        b.source_object_id() == r.source && b.kind() == GoSourceNameKind::Variable
                    })
                    .ok_or(GoScalarTypingError::Locator)?;
                let n = self.node(&r.source, b.declaration_node())?;
                if n.kind() == "RangeStmt" {
                    let ordinal = if list(n, "Key").contains(&b.name_node()) {
                        0
                    } else if list(n, "Value").contains(&b.name_node()) {
                        1
                    } else {
                        return Err(GoScalarTypingError::Locator);
                    };
                    return Ok(Step::More(Plan::Range(
                        Request::raw(&r.source, child(n, "X")?),
                        ordinal,
                    )));
                }
                if matches!(n.kind(), "FuncDecl" | "FuncLit") {
                    let declared = derive_go_declared_binding_type(self.scopes, b)
                        .map_err(GoScalarTypingError::Signature)?;
                    let typ = self.graph_type(declared.types(), declared.type_id())?;
                    return Ok(Step::Done(vec![self.atom(&typ, true)]));
                }
                let names = if n.kind() == "ValueSpec" {
                    list(n, "Names")
                } else {
                    list(n, "Lhs")
                };
                let ordinal = names
                    .iter()
                    .position(|&n| n == b.name_node())
                    .ok_or(GoScalarTypingError::Unsupported)?;
                Ok(Step::More(Plan::Select(
                    Request::declaration(&r.source, b.declaration_node()),
                    ordinal,
                )))
            }
            Target::Declaration(node) => {
                let n = self.node(&r.source, *node)?;
                if n.kind() == "ValueSpec" && list(n, "Values").is_empty() {
                    let typ = self.annotation(&r.source, child(n, "Type")?)?;
                    let count = list(self.node(&r.source, *node)?, "Names").len();
                    return Ok(Step::Done(
                        (0..count).map(|_| self.atom(&typ, true)).collect(),
                    ));
                }
                Ok(Step::More(self.declaration(&r.source, *node)?))
            }
            Target::Expression(node, Mode::Element(typ)) => {
                let n = self.node(&r.source, *node)?;
                if n.kind() == "CompositeLit" && list(n, "Type").is_empty() {
                    return Ok(Step::More(self.literal(&r.source, *node, Some(typ))?));
                }
                Ok(Step::More(Plan::Forward(Request::value(
                    &r.source,
                    *node,
                    Some(typ.clone()),
                ))))
            }
            Target::Expression(node, Mode::Index) => {
                Ok(Step::More(Plan::IndexValue(Request::raw(&r.source, *node))))
            }
            Target::Expression(node, Mode::Value(expected)) => Ok(Step::More(Plan::Materialize(
                Request::raw(&r.source, *node),
                expected.clone(),
            ))),
            Target::Expression(node, Mode::Fixed(typ)) => self.fixed(&r.source, *node, typ),
            Target::Expression(node, Mode::Raw) => {
                let n = self.node(&r.source, *node)?.clone();
                let step = match n.kind() {
                    "BasicLit" => {
                        let value = self.exact_constant(&r.source, *node)?;
                        Step::Done(vec![self.constant(value)?])
                    }
                    "SelectorExpr" if !self.is_package_selector(&r.source, *node)? => {
                        Step::More(Plan::Field(
                            Request::raw(&r.source, child(&n, "X")?),
                            attr(self.node(&r.source, child(&n, "Sel")?)?, "Name")
                                .ok_or(GoScalarTypingError::Locator)?
                                .into(),
                        ))
                    }
                    "Ident" | "SelectorExpr" => match self.symbol(&r.source, *node)? {
                        GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Variable => {
                            Step::More(Plan::Forward(Request::binding(&b, self.binding_index(&b)?)))
                        }
                        GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Constant => {
                            let value = self.exact_binding(&b)?;
                            Step::Done(vec![self.constant(value)?])
                        }
                        GoNameLookup::Predeclared {
                            name,
                            kind: GoSourceNameKind::BuiltinNil,
                        } if name == "nil" => Step::Done(vec![Atom {
                            typ: None,
                            kind: None,
                            nil: true,
                            value: None,
                            addressable: false,
                        }]),
                        GoNameLookup::Predeclared {
                            kind: GoSourceNameKind::BuiltinConstant,
                            ..
                        } => {
                            let value = self.exact_constant(&r.source, *node)?;
                            Step::Done(vec![self.constant(value)?])
                        }
                        _ => return Err(GoScalarTypingError::Unsupported),
                    },
                    "ParenExpr" => {
                        Step::More(Plan::Forward(Request::raw(&r.source, child(&n, "X")?)))
                    }
                    "UnaryExpr" => Step::More(Plan::Unary(
                        attr(&n, "Op").ok_or(GoScalarTypingError::Locator)?.into(),
                        Request::raw(&r.source, child(&n, "X")?),
                    )),
                    "BinaryExpr" => Step::More(Plan::BinaryFirst(
                        attr(&n, "Op").ok_or(GoScalarTypingError::Locator)?.into(),
                        Request::raw(&r.source, child(&n, "X")?),
                        Request::raw(&r.source, child(&n, "Y")?),
                    )),
                    "CallExpr" => {
                        let fun = child(&n, "Fun")?;
                        let dots = attr(&n, "Ellipsis") == Some("true");
                        if let Ok(GoNameLookup::Predeclared {
                            name,
                            kind: GoSourceNameKind::BuiltinFunction,
                        }) = self.symbol(&r.source, fun)
                        {
                            if matches!(name.as_str(), "make" | "append" | "copy") {
                                return Ok(Step::More(
                                    self.slice_builtin_plan(&r.source, *node, &name, dots)?,
                                ));
                            }
                        }
                        if dots {
                            return Err(GoScalarTypingError::Unsupported);
                        }
                        let args = list(&n, "Args")
                            .iter()
                            .map(|&arg| Request::raw(&r.source, arg))
                            .collect::<Vec<_>>();
                        let mut selector = fun;
                        while self.node(&r.source, selector)?.kind() == "ParenExpr" {
                            selector = child(self.node(&r.source, selector)?, "X")?;
                        }
                        if self.node(&r.source, selector)?.kind() == "SelectorExpr"
                            && !self.is_package_selector(&r.source, selector)?
                        {
                            return self.method_plan(r, selector, args);
                        }
                        if self.symbol(&r.source, fun).is_err() {
                            if args.len() != 1 {
                                return Err(GoScalarTypingError::Arity);
                            }
                            let typ = self.annotation(&r.source, fun)?;
                            return Ok(Step::More(Plan::Cast(typ, args[0].clone())));
                        }
                        match self.symbol(&r.source, fun)? {
                            GoNameLookup::Predeclared {
                                kind: GoSourceNameKind::BuiltinType,
                                ..
                            } => {
                                let target = self.annotation(&r.source, fun)?;
                                if args.len() != 1 {
                                    return Err(GoScalarTypingError::Arity);
                                }
                                Step::More(Plan::Cast(target, args[0].clone()))
                            }
                            GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Type => {
                                let target = self.annotation(&r.source, fun)?;
                                if args.len() != 1 {
                                    return Err(GoScalarTypingError::Arity);
                                }
                                Step::More(Plan::Cast(target, args[0].clone()))
                            }
                            GoNameLookup::Predeclared {
                                name,
                                kind: GoSourceNameKind::BuiltinFunction,
                            } if matches!(name.as_str(), "len" | "cap" | "min" | "max") => {
                                Step::More(Plan::BuiltinFirst(name, args))
                            }
                            GoNameLookup::Original(b) if b.kind() == GoSourceNameKind::Function => {
                                Step::More(Plan::CallFirst(
                                    Box::new(self.header(&r.source, *node, &b)?),
                                    args,
                                ))
                            }
                            _ => return Err(GoScalarTypingError::Unsupported),
                        }
                    }
                    "CompositeLit" => Step::More(self.literal(&r.source, *node, None)?),
                    "IndexExpr" => Step::More(Plan::Index(
                        Request::raw(&r.source, child(&n, "X")?),
                        Request::index(&r.source, child(&n, "Index")?),
                    )),
                    "SliceExpr" => Step::More(Plan::Slice(
                        Request::raw(&r.source, child(&n, "X")?),
                        Box::new(["Low", "High", "Max"].map(|role| {
                            list(&n, role)
                                .first()
                                .map(|&n| Request::index(&r.source, n))
                        })),
                        attr(&n, "Slice3") == Some("true"),
                    )),
                    _ => return Err(GoScalarTypingError::Unsupported),
                };
                Ok(step)
            }
        }
    }
    fn fixed(&mut self, source: &str, node: usize, typ: &str) -> Result<Step> {
        let raw = self.one(&Request::raw(source, node))?;
        if raw.typ.is_some() || raw.value.is_some() {
            return Err(GoScalarTypingError::Type);
        }
        if !self.compatible_kind(raw.kind, typ) {
            return Err(GoScalarTypingError::Type);
        }
        let n = self.node(source, node)?.clone();
        Ok(Step::More(match n.kind() {
            "ParenExpr" => Plan::Forward(Request::value(source, child(&n, "X")?, Some(typ.into()))),
            "UnaryExpr" => Plan::Unary(
                attr(&n, "Op").ok_or(GoScalarTypingError::Locator)?.into(),
                Request::value(source, child(&n, "X")?, Some(typ.into())),
            ),
            "BinaryExpr" => {
                let op = attr(&n, "Op").ok_or(GoScalarTypingError::Locator)?;
                if comparison(op) {
                    return Ok(Step::Done(vec![self.atom(typ, false)]));
                }
                Plan::BinaryFirst(
                    op.into(),
                    Request::value(source, child(&n, "X")?, Some(typ.into())),
                    if matches!(op, "<<" | ">>") {
                        Request::raw(source, child(&n, "Y")?)
                    } else {
                        Request::value(source, child(&n, "Y")?, Some(typ.into()))
                    },
                )
            }
            "CallExpr" => {
                let GoNameLookup::Predeclared {
                    name,
                    kind: GoSourceNameKind::BuiltinFunction,
                } = self.symbol(source, child(&n, "Fun")?)?
                else {
                    return Err(GoScalarTypingError::Type);
                };
                if !matches!(name.as_str(), "min" | "max") {
                    return Err(GoScalarTypingError::Type);
                }
                Plan::BuiltinFirst(
                    name,
                    list(&n, "Args")
                        .iter()
                        .map(|&arg| Request::value(source, arg, Some(typ.into())))
                        .collect(),
                )
            }
            _ => return Err(GoScalarTypingError::Type),
        }))
    }
    fn finish(&mut self, r: &Request, plan: Plan) -> Result<Step> {
        match plan {
            Plan::ReturnFirst(types, args) => self.return_first(types, args),
            Plan::ReturnReady(types, args) => {
                let facts = self.expand(&args)?;
                self.return_result(&args, &types, &facts)
            }
            Plan::AssignmentFirst(lhs, rhs) => self.assignment_first(lhs, rhs),
            Plan::AssignmentReady(lhs, rhs) => {
                let facts = self.expand(&rhs)?;
                self.assignment_result(&lhs, &rhs, facts)
            }
            Plan::ShortAssignment(declaration) => {
                let mut facts = self.facts(&declaration)?;
                let Target::Assignment(node) = r.target else {
                    return Err(GoScalarTypingError::Locator);
                };
                for (a, &n) in facts
                    .iter_mut()
                    .zip(list(self.node(&r.source, node)?, "Lhs"))
                {
                    a.addressable = !self.blank_target(&r.source, n)?;
                }
                Ok(Step::Done(facts))
            }
            Plan::Compound(op, lhs, rhs) => {
                self.assignment_targets(&[Some(lhs.clone())])?;
                self.binary_first(op, lhs, rhs)
            }
            Plan::Increment(lhs) => {
                self.assignment_targets(&[Some(lhs.clone())])?;
                let a = self.one(&lhs)?;
                if !matches!(a.kind, Some(GoConstantKind::Integer | GoConstantKind::Rune)) {
                    return Err(GoScalarTypingError::Type);
                }
                Ok(Step::Done(vec![self.atom(
                    a.typ.as_deref().ok_or(GoScalarTypingError::Type)?,
                    false,
                )]))
            }
            Plan::MethodReceiver(base, selector, args) => {
                let atom = self.one(&base)?;
                let typ = atom.typ.ok_or(GoScalarTypingError::Type)?;
                let header = self.method_header(r, selector, &typ, false)?;
                Ok(Step::More(Plan::MethodFirst(Box::new(header), args)))
            }
            Plan::MethodFirst(header, args) => self.method_first(*header, args),
            Plan::MethodReady(header, args) => {
                let facts = self.expand(&args)?;
                self.error_arguments(&args, &facts, &header.parameters)?;
                self.method_result(*header)
            }
            Plan::Make(typ, args) => self.make_slice(r, typ, args),
            Plan::AppendFirst(args, dots) => self.append_first(r, args, dots),
            Plan::AppendReady(typ, args, dots) => self.append_ready(r, typ, args, dots),
            Plan::Copy(args) => self.copy_slice(r, args),
            Plan::IndexValue(raw) => {
                let a = self.one(&raw)?;
                if a.typ.is_some() {
                    return Ok(Step::Done(vec![a]));
                }
                let int = self.builtin("int")?;
                Ok(Step::More(Plan::Forward(Request::value(
                    &raw.source,
                    raw.expression()?,
                    Some(int),
                ))))
            }
            Plan::Forward(child) => Ok(Step::Done(self.facts(&child)?)),
            Plan::Select(child, index) => {
                let facts = self.facts(&child)?;
                Ok(Step::Done(vec![facts
                    .get(index)
                    .ok_or(GoScalarTypingError::Arity)?
                    .clone()]))
            }
            Plan::Materialize(raw, expected) => {
                let facts = self.facts(&raw)?;
                if expected.is_some() && facts.len() != 1 {
                    return Err(GoScalarTypingError::Arity);
                }
                if facts.len() != 1 {
                    if facts.iter().any(|a| a.typ.is_none()) {
                        return Err(GoScalarTypingError::Type);
                    }
                    return Ok(Step::Done(facts));
                }
                let mut a = facts[0].clone();
                let typ = if let Some(expected) = expected {
                    expected
                } else if let Some(t) = &a.typ {
                    t.clone()
                } else {
                    self.default_type(a.kind.ok_or(GoScalarTypingError::Type)?)?
                };
                if let Some(current) = &a.typ {
                    if !self.assignable(current, &typ) {
                        return Err(GoScalarTypingError::Type);
                    }
                    self.record_error_boxings(
                        std::slice::from_ref(&raw),
                        std::slice::from_ref(&a),
                        &[Some(typ)],
                    )?;
                    return Ok(Step::Done(vec![a]));
                }
                if a.nil {
                    if !self.has_nil(&typ) {
                        return Err(GoScalarTypingError::Type);
                    }
                    a.typ = Some(typ);
                    return Ok(Step::Done(vec![a]));
                }
                if let Some(value) = a.value {
                    let converted = value
                        .convert(
                            self.types
                                .get(&typ)
                                .ok_or(GoScalarTypingError::Type)?
                                .clone(),
                            false,
                        )
                        .map_err(constant_error)?;
                    a = self.constant(converted)?;
                    return Ok(Step::Done(vec![a]));
                }
                Ok(Step::More(Plan::Forward(Request::fixed(
                    &raw.source,
                    raw.expression()?,
                    typ,
                ))))
            }
            Plan::Unary(op, arg) => {
                let a = self.one(&arg)?;
                Ok(Step::Done(vec![self.unary(&op, a)?]))
            }
            Plan::BinaryFirst(op, x, y) => self.binary_first(op, x, y),
            Plan::BinaryReady(op, x, y) => {
                let a = self.one(&x)?;
                let b = self.one(&y)?;
                Ok(Step::Done(vec![self.binary(&op, a, b)?]))
            }
            Plan::Cast(typ, arg) => self.cast(typ, arg),
            Plan::Field(base, name) => {
                let a = self.one(&base)?;
                Ok(Step::Done(vec![self.field(&r.source, a, &name)?]))
            }
            Plan::Index(base, index) => {
                let a = self.one(&base)?;
                let i = self.one(&index)?;
                Ok(Step::Done(vec![self.index(a, i)?]))
            }
            Plan::Slice(base, bounds, full) => {
                let a = self.one(&base)?;
                let bounds = bounds
                    .iter()
                    .map(|r| r.as_ref().map(|r| self.one(r)).transpose())
                    .collect::<Result<Vec<_>>>()?;
                Ok(Step::Done(vec![self.slice(a, &bounds, full)?]))
            }
            Plan::Literal(typ, args) => {
                for arg in args {
                    self.one(&arg)?;
                }
                Ok(Step::Done(vec![self.atom(&typ, false)]))
            }
            Plan::Range(base, index) => {
                let a = self.one(&base)?;
                let typ = match self.shape(a.typ.as_deref().ok_or(GoScalarTypingError::Type)?)? {
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => element.clone(),
                    _ => return Err(GoScalarTypingError::Unsupported),
                };
                let typ = if index == 0 {
                    self.builtin("int")?
                } else {
                    typ
                };
                Ok(Step::Done(vec![self.atom(&typ, true)]))
            }
            Plan::BuiltinFirst(name, args) => self.builtin_first(r, name, args),
            Plan::BuiltinReady(name, args) => {
                let values = args
                    .iter()
                    .map(|r| self.one(r))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Step::Done(vec![self.minmax(&name, values)?]))
            }
            Plan::CallFirst(header, args) => {
                let facts = self.expand(&args)?;
                if facts.len() != header.parameters.len() {
                    return Err(GoScalarTypingError::Arity);
                }
                if args.len() == 1 && facts.len() != 1 {
                    self.error_arguments(&args, &facts, &header.parameters)?;
                    return self.call_result(*header);
                }
                let next = args
                    .iter()
                    .zip(&header.parameters)
                    .map(|(arg, typ)| {
                        Ok(Request::value(
                            &arg.source,
                            arg.expression()?,
                            Some(typ.clone()),
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Step::More(Plan::CallReady(header, next)))
            }
            Plan::CallReady(header, args) => {
                let facts = self.expand(&args)?;
                self.error_arguments(&args, &facts, &header.parameters)?;
                self.call_result(*header)
            }
            Plan::DeclarationFirst(dest, rhs) => {
                let facts = self.expand(&rhs)?;
                if facts.len() != dest.len() {
                    return Err(GoScalarTypingError::Arity);
                }
                let targets = dest
                    .iter()
                    .map(|d| self.destination(d))
                    .collect::<Result<Vec<_>>>()?;
                if rhs.len() == 1 && facts.len() != 1 {
                    return self.declared(&rhs, facts, targets);
                }
                let next = rhs
                    .iter()
                    .zip(targets)
                    .map(|(arg, typ)| Ok(Request::value(&arg.source, arg.expression()?, typ)))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Step::More(Plan::DeclarationReady(dest, next)))
            }
            Plan::DeclarationReady(dest, rhs) => {
                let facts = self.expand(&rhs)?;
                let targets = dest
                    .iter()
                    .map(|d| self.destination(d))
                    .collect::<Result<Vec<_>>>()?;
                self.declared(&rhs, facts, targets)
            }
        }
    }
    fn destination(&self, d: &Destination) -> Result<Option<String>> {
        match d {
            Destination::Inferred => Ok(None),
            Destination::Declared(t) => Ok(Some(t.clone())),
            Destination::Reused(r) => Ok(self.one(r)?.typ),
        }
    }
    fn declared(
        &mut self,
        args: &[Request],
        facts: Facts,
        targets: Vec<Option<String>>,
    ) -> Result<Step> {
        if facts.len() != targets.len() {
            return Err(GoScalarTypingError::Arity);
        }
        let mut result = vec![];
        for (a, target) in facts.iter().zip(&targets) {
            let typ = a.typ.clone().ok_or(GoScalarTypingError::Type)?;
            let target = target.clone().unwrap_or_else(|| typ.clone());
            if !self.assignable(&typ, &target) {
                return Err(GoScalarTypingError::Type);
            }
            result.push(self.atom(&target, true))
        }
        self.record_error_boxings(args, &facts, &targets)?;
        Ok(Step::Done(result))
    }
    fn expand(&self, args: &[Request]) -> Result<Facts> {
        let mut result = vec![];
        for arg in args {
            let facts = self.facts(arg)?;
            if facts.is_empty() || (args.len() != 1 && facts.len() != 1) {
                return Err(GoScalarTypingError::Arity);
            }
            result.extend(facts)
        }
        Ok(result)
    }
    fn check_types(&self, facts: &[Atom], types: &[String]) -> Result<()> {
        if facts.len() != types.len() {
            return Err(GoScalarTypingError::Arity);
        }
        if facts.iter().zip(types).any(|(a, t)| {
            !a.typ
                .as_ref()
                .is_some_and(|actual| self.assignable(actual, t))
        }) {
            return Err(GoScalarTypingError::Type);
        }
        Ok(())
    }
    fn call_result(&mut self, header: GoScalarCallHeader) -> Result<Step> {
        let values = header.results.iter().map(|t| self.atom(t, false)).collect();
        self.calls.insert(header);
        Ok(Step::Done(values))
    }
    fn compute(&mut self, root: Request) -> Result<()> {
        let mut work = vec![Work::Visit(root.clone())];
        while let Some(task) = work.pop() {
            let (r, step) = match task {
                Work::Visit(r) => {
                    if self.done.contains_key(&r) {
                        continue;
                    }
                    if !self.active.insert(r.clone()) {
                        return Err(GoScalarTypingError::Cycle);
                    }
                    let step = self.plan(&r)?;
                    (r, step)
                }
                Work::Finish(r, plan) => {
                    let step = self.finish(&r, *plan)?;
                    (r, step)
                }
            };
            match step {
                Step::Done(value) => {
                    self.done.insert(r.clone(), value);
                    self.active.remove(&r);
                }
                Step::More(plan) => {
                    let inputs = plan.inputs();
                    work.push(Work::Finish(r, Box::new(plan)));
                    for input in inputs.into_iter().rev() {
                        work.push(Work::Visit(input))
                    }
                }
            }
        }
        Ok(())
    }
    fn run(mut self, root: Request) -> Result<GoValueTyping> {
        self.compute(root.clone())?;
        let facts = self.done.remove(&root).ok_or(GoScalarTypingError::Type)?;
        self.into_typing(facts)
    }
    fn into_typing(self, facts: Facts) -> Result<GoValueTyping> {
        let mut values = vec![];
        for fact in facts {
            let id = fact.typ.ok_or(GoScalarTypingError::Type)?;
            values.push(GoSourceTypedValue {
                typ: self.keys[&id].clone(),
                scalar: self.types.get(&id).cloned(),
                constant: fact.value,
                addressable: fact.addressable,
                untyped_nil: fact.nil,
            })
        }
        Ok(GoValueTyping {
            scopes: source_dependencies::shared_sources(self.scopes),
            values,
            calls: self.calls.into_iter().collect(),
            slice_builtins: self.slice_builtins.into_iter().collect(),
            method_calls: self.method_calls.into_iter().collect(),
            error_implementations: self.error_implementations.into_values().flatten().collect(),
            error_boxings: self.error_boxings.into_iter().collect(),
            types: self.keys,
            underlying: self.underlying,
            declarations: self.declarations,
        })
    }
}
/// Type one value in default context. The containing body and dynamic accesses
/// still require their own independent checks.
pub fn infer_go_value(scopes: &GoSourceScopes, source: &str, node: usize) -> Result<GoValueTyping> {
    let result = infer_go_results(scopes, source, node)?;
    if result.values.len() != 1 {
        return Err(GoScalarTypingError::Arity);
    }
    Ok(result)
}
/// Derive the0..16 results of an original expression. Calls carry only the
/// original callee header; this is not evaluation or a callee-body proof.
pub fn infer_go_results(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoValueTyping> {
    Engine::new(scopes).run(Request::value(source, node, None))
}
/// Check one original value against an independently derived type annotation.
/// No relationship to an actual assignment statement is asserted here.
pub fn check_go_assignment(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
    type_source: &str,
    type_node: usize,
) -> Result<GoValueTyping> {
    let mut engine = Engine::new(scopes);
    let typ = engine.annotation(type_source, type_node)?;
    engine.run(Request::value(source, node, Some(typ)))
}
/// Infer an original variable, including all siblings and reused destinations
/// in its actual initializer, or the original array/slice range binding.
pub fn infer_go_binding(
    scopes: &GoSourceScopes,
    binding: &GoSourceNameBinding,
) -> Result<GoValueTyping> {
    let engine = Engine::new(scopes);
    let index = engine.binding_index(binding)?;
    if binding.kind() != GoSourceNameKind::Variable {
        return Err(GoScalarTypingError::Type);
    }
    engine.run(Request::binding(binding, index))
}
/// Type one scalar value in default value context. A void or multiple-result
/// call cannot stand in for one value. No containing body is typechecked here.
pub fn infer_go_scalar_value(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoScalarTyping> {
    let result = infer_go_scalar_results(scopes, source, node)?;
    if result.values.len() != 1 {
        return Err(GoScalarTypingError::Arity);
    }
    Ok(result)
}
/// Type an expression's0..16 scalar results in a result-list context. Consumers
/// still check the actual destination list and single-invocation state linkage.
pub fn infer_go_scalar_results(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> Result<GoScalarTyping> {
    infer_go_results(scopes, source, node)?.scalar_only()
}
/// Check assignment compatibility against one independently derived original
/// scalar type expression. No actual statement relationship is asserted here.
pub fn check_go_scalar_assignment(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
    type_source: &str,
    type_node: usize,
) -> Result<GoScalarTyping> {
    let mut engine = Engine::new(scopes);
    let typ = engine.annotation(type_source, type_node)?;
    engine
        .run(Request::value(source, node, Some(typ)))?
        .scalar_only()
}
/// Derive a real scalar variable's type, checking its complete shared initializer
/// declaration (including reused short-declaration targets) where present.
pub fn infer_go_scalar_binding(
    scopes: &GoSourceScopes,
    binding: &GoSourceNameBinding,
) -> Result<GoScalarTyping> {
    let engine = Engine::new(scopes);
    let index = engine.binding_index(binding)?;
    if binding.kind() != GoSourceNameKind::Variable {
        return Err(GoScalarTypingError::Type);
    }
    engine.run(Request::binding(binding, index))?.scalar_only()
}

/// Internal scheduler request: derive a len/cap result from actual original
/// argument typing. Compiled reads do not assert runtime argument evaluation.
pub(super) fn constant_measure(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> std::result::Result<(GoEvaluatedConstant, source_dependencies::Reads), GoConstantError> {
    fn mapped(error: GoScalarTypingError) -> GoConstantError {
        match error {
            GoScalarTypingError::Constant(e) => e,
            GoScalarTypingError::Unsupported
            | GoScalarTypingError::SourceType(GoSourceTypeError::Unsupported)
            | GoScalarTypingError::Signature(GoSourceSignatureError::Unsupported) => {
                GoConstantError::Unsupported
            }
            GoScalarTypingError::Cycle
            | GoScalarTypingError::SourceType(GoSourceTypeError::Cycle) => GoConstantError::Cycle,
            GoScalarTypingError::Limit => GoConstantError::Resource,
            _ => GoConstantError::Type,
        }
    }
    let mut engine = Engine::new(scopes);
    let root = Request::raw(source, node);
    engine.compute(root.clone()).map_err(mapped)?;
    let value = engine
        .one(&root)
        .map_err(mapped)?
        .value
        .ok_or(GoConstantError::NotConstant)?;
    let mut reads = engine.compilation_reads.clone();
    for request in engine.done.keys() {
        let Target::Expression(node, Mode::Raw) = request.target else {
            continue;
        };
        if engine
            .node(&request.source, node)
            .is_ok_and(|n| n.kind() == "Ident")
        {
            if let Ok(GoNameLookup::Original(b)) = engine.symbol(&request.source, node) {
                if b.kind() == GoSourceNameKind::Variable {
                    reads.insert(source_dependencies::Read {
                        source: request.source.clone(),
                        node,
                        definition_source: b.source_object_id().into(),
                        declaration: b.declaration_node(),
                        name: b.name_node(),
                    });
                }
            }
        }
    }
    Ok((value, reads))
}
