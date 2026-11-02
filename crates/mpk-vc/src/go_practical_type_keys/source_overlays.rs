//! Original sidecar syntax and source selectors, not a typed contract or context.
//! Raw bytes precede capture-dependent scope/binding/contract identities.
use super::*;
use serde_json::Value;
use std::sync::Arc;
mod expressions;
mod module;
pub(super) use module::derive_go_source_contract_module_at;
pub use module::{
    derive_go_source_contract_module, GoCapturedFunctionOverlay, GoSourceContractModule,
    GoSourceContractOrigin, GoSourceFunctionContract, GoSourceFunctionLoopContract,
    GoSourceLoopCandidateOrigin,
};
mod syntax;
pub use expressions::{
    derive_go_typed_source_overlays, GoContractExpression, GoContractExpressionReference,
    GoContractExpressionType, GoContractHeaderBinding, GoContractObservation, GoContractPhase,
    GoContractTypingError, GoContractTypingErrorKind, GoTypedOverlayLoop, GoTypedSourceOverlay,
    GoTypedSourceOverlaySet,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoSourceReceiverSelector {
    package: String,
    type_name: String,
}
impl GoSourceReceiverSelector {
    pub fn package(&self) -> &str {
        &self.package
    }
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoSourceFunctionSelector {
    package: String,
    function: String,
    receiver: Option<GoSourceReceiverSelector>,
}
impl GoSourceFunctionSelector {
    pub fn package(&self) -> &str {
        &self.package
    }
    pub fn function(&self) -> &str {
        &self.function
    }
    pub fn receiver(&self) -> Option<&GoSourceReceiverSelector> {
        self.receiver.as_ref()
    }
    fn validate(&self) -> bool {
        validate::logical_path(&self.package)
            && named(&self.function)
            && (self.function != "init" || self.receiver.is_some())
            && self
                .receiver
                .as_ref()
                .is_none_or(|r| r.package == self.package && named(&r.type_name))
    }
}
fn named(s: &str) -> bool {
    s != "_" && validate::identifier(s)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceOverlayErrorKind {
    Json(Box<crate::canonical_json::StrictJsonError>),
    Shape,
    Limit,
    Selector,
    Source,
    TypeSelector,
    Type(GoSourceTypeError),
    Signature(GoSourceSignatureError),
    EntryPath,
    DuplicateEntry,
    DuplicateCase,
    MissingEntry,
    LoopSelector,
    DuplicateLoop,
    DuplicateOverlay,
    Scope(GoSourceScopeError),
    Selection(Box<GoSourceErrorSelectionError>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceOverlayError {
    kind: GoSourceOverlayErrorKind,
    overlay: Option<String>,
    row: Option<usize>,
}
impl GoSourceOverlayError {
    pub fn kind(&self) -> &GoSourceOverlayErrorKind {
        &self.kind
    }
    pub fn overlay_object_id(&self) -> Option<&str> {
        self.overlay.as_deref()
    }
    pub fn row(&self) -> Option<usize> {
        self.row
    }
}
impl std::fmt::Display for GoSourceOverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source overlay error: {self:?}")
    }
}
impl std::error::Error for GoSourceOverlayError {}
type OverlayResult<T> = std::result::Result<T, GoSourceOverlayError>;
fn failure(kind: GoSourceOverlayErrorKind) -> GoSourceOverlayError {
    GoSourceOverlayError {
        kind,
        overlay: None,
        row: None,
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum PathStep {
    #[serde(rename = "field")]
    Field { name: String },
    #[serde(rename = "elements")]
    Elements,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    input: usize,
    path: Vec<PathStep>,
    cases: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoopSelector {
    kind: String,
    start_byte: usize,
    end_byte: usize,
    ordinal: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Variant {
    #[serde(rename = "absent")]
    Absent,
    #[serde(rename = "candidate")]
    Candidate { expression: Value },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayLoop {
    selector: LoopSelector,
    invariants: Vec<Value>,
    variant: Variant,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    source: String,
    function: GoSourceFunctionSelector,
    termination: String,
    requires: Vec<Value>,
    ensures: Vec<Value>,
    error_inputs: Vec<Entry>,
    loops: Vec<OverlayLoop>,
    modifies: Vec<Value>,
}
/// Complete closed source syntax only. Expression typing and contract authority
/// cannot be inferred from successful import or source-selector binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceOverlay {
    raw: GoTypeSourceBuffer,
    wire: Wire,
    expression_nodes: usize,
    expression_depth: usize,
}
impl GoSourceOverlay {
    pub fn import(raw: &GoTypeSourceBuffer) -> OverlayResult<Self> {
        let result = (|| {
            if raw.object().kind != "overlay" {
                return Err(failure(GoSourceOverlayErrorKind::Source));
            }
            let limits = StrictJsonLimits::new(1_048_576, 1_048_576, 128, 1_048_576);
            let original = parse_strict_json(raw.bytes(), limits)
                .map_err(|e| failure(GoSourceOverlayErrorKind::Json(Box::new(e))))?;
            let wire: Wire = serde_json::from_slice(raw.bytes())
                .map_err(|_| failure(GoSourceOverlayErrorKind::Shape))?;
            let encoded =
                serde_json::to_vec(&wire).map_err(|_| failure(GoSourceOverlayErrorKind::Shape))?;
            let expected = parse_strict_json(&encoded, limits)
                .map_err(|e| failure(GoSourceOverlayErrorKind::Json(Box::new(e))))?;
            if canonical_json_bytes(&original)
                .map_err(|_| failure(GoSourceOverlayErrorKind::Shape))?
                != canonical_json_bytes(&expected)
                    .map_err(|_| failure(GoSourceOverlayErrorKind::Shape))?
            {
                return Err(failure(GoSourceOverlayErrorKind::Shape));
            }
            let (nodes, depth) = syntax::validate(&wire)?;
            Ok(Self {
                raw: raw.clone(),
                wire,
                expression_nodes: nodes,
                expression_depth: depth,
            })
        })();
        result.map_err(|mut e: GoSourceOverlayError| {
            e.overlay = Some(raw.object().id.clone());
            e
        })
    }
    pub fn source_buffer(&self) -> &GoTypeSourceBuffer {
        &self.raw
    }
    pub fn function_selector(&self) -> &GoSourceFunctionSelector {
        &self.wire.function
    }
    pub fn source_path(&self) -> &str {
        &self.wire.source
    }
    pub fn termination(&self) -> &str {
        &self.wire.termination
    }
    pub fn requires(&self) -> &[Value] {
        &self.wire.requires
    }
    pub fn ensures(&self) -> &[Value] {
        &self.wire.ensures
    }
    pub fn expression_nodes(&self) -> usize {
        self.expression_nodes
    }
    pub fn expression_depth(&self) -> usize {
        self.expression_depth
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBoundOverlayLoop {
    node: usize,
    ordinal: usize,
    overlay_row: Option<usize>,
}
impl GoBoundOverlayLoop {
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn overlay_row(&self) -> Option<usize> {
        self.overlay_row
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBoundSourceOverlay {
    overlay: GoSourceOverlay,
    scopes: Arc<GoSourceScopes>,
    root: GoSourceSelectionRef,
    signature: GoSourceSignature,
    entries: Vec<GoErrorEntryConstraint>,
    loops: Vec<GoBoundOverlayLoop>,
}
impl GoBoundSourceOverlay {
    pub fn overlay(&self) -> &GoSourceOverlay {
        &self.overlay
    }
    pub fn root(&self) -> &GoSourceSelectionRef {
        &self.root
    }
    pub fn signature(&self) -> &GoSourceSignature {
        &self.signature
    }
    pub fn entries(&self) -> &[GoErrorEntryConstraint] {
        &self.entries
    }
    pub fn loops(&self) -> &[GoBoundOverlayLoop] {
        &self.loops
    }
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> {
        self.scopes.source(id)
    }
}
struct Binder<'a> {
    keys: &'a GoTypeKeyComponents,
    scopes: Arc<GoSourceScopes>,
}
impl Binder<'_> {
    fn package(&self, path: &str) -> OverlayResult<&GoTypePackageKey> {
        self.keys
            .wire
            .packages
            .iter()
            .find(|p| p.import_path == path)
            .ok_or_else(|| failure(GoSourceOverlayErrorKind::Selector))
    }
    fn type_ref(&self, text: &str) -> OverlayResult<(GoSourceSelectionRef, String)> {
        let (package, name) = text
            .rsplit_once('.')
            .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))?;
        if !named(name) {
            return Err(failure(GoSourceOverlayErrorKind::TypeSelector));
        }
        let package = self.package(package)?;
        let binding = self
            .scopes
            .bindings()
            .iter()
            .find(|b| {
                b.package_id() == package.id
                    && b.name() == name
                    && b.kind() == GoSourceNameKind::Type
                    && self.scopes.package_binding(b)
            })
            .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))?;
        let r = GoSourceSelectionRef::new(binding.source_object_id(), binding.declaration_node());
        let graph = derive_go_source_type_declaration(&self.scopes, r.source_object_id(), r.node())
            .map_err(|e| failure(GoSourceOverlayErrorKind::Type(e)))?;
        Ok((r, graph.roots()[0].clone()))
    }
    // Select a method by its original named receiver declaration. Identity
    // lookup must not require admitting an unselected receiver's underlying
    // map/interface/etc as a practical value type. Selected signatures still
    // pass the ordinary source type checks before a contract can be produced.
    fn receiver_declaration(
        &self,
        source: &str,
        node: usize,
    ) -> OverlayResult<Option<GoSourceSelectionRef>> {
        let mut current = GoSourceSelectionRef::new(source, node);
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(current.clone()) {
                return Err(failure(GoSourceOverlayErrorKind::TypeSelector));
            }
            let parsed = self
                .scopes
                .source(current.source_object_id())
                .ok_or_else(|| failure(GoSourceOverlayErrorKind::Source))?;
            let n = parsed
                .nodes()
                .get(current.node())
                .ok_or_else(|| failure(GoSourceOverlayErrorKind::Source))?;
            let child = |role| {
                source_signatures::list(n, role)
                    .first()
                    .copied()
                    .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))
            };
            let binding = match n.kind() {
                "TypeSpec" => {
                    if source_signatures::attr(n, "Alias") != Some("true") {
                        return Ok(Some(current));
                    }
                    current = GoSourceSelectionRef::new(current.source_object_id(), child("Type")?);
                    continue;
                }
                "ParenExpr" | "IndexExpr" | "IndexListExpr" => {
                    current = GoSourceSelectionRef::new(current.source_object_id(), child("X")?);
                    continue;
                }
                "Ident" => match self
                    .scopes
                    .lookup_identifier(current.source_object_id(), current.node())
                    .map_err(|e| failure(GoSourceOverlayErrorKind::Scope(e)))?
                {
                    Some(GoNameLookup::Original(b)) => b,
                    _ => return Ok(None),
                },
                "SelectorExpr" => {
                    let x = child("X")?;
                    let Some(GoNameLookup::Original(import)) = self
                        .scopes
                        .lookup_identifier(current.source_object_id(), x)
                        .map_err(|e| failure(GoSourceOverlayErrorKind::Scope(e)))?
                    else {
                        return Ok(None);
                    };
                    if import.kind() != GoSourceNameKind::Import {
                        return Ok(None);
                    }
                    let name = source_signatures::attr(&parsed.nodes()[child("Sel")?], "Name")
                        .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))?;
                    let path = import
                        .import_path()
                        .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))?;
                    let Some(binding) = self
                        .scopes
                        .imported_member(path, name)
                        .map_err(|e| failure(GoSourceOverlayErrorKind::Scope(e)))?
                    else {
                        return Ok(None);
                    };
                    binding
                }
                _ => return Ok(None),
            };
            if binding.kind() != GoSourceNameKind::Type {
                return Ok(None);
            }
            current =
                GoSourceSelectionRef::new(binding.source_object_id(), binding.declaration_node());
        }
    }
    fn function(&self, overlay: &GoSourceOverlay) -> OverlayResult<GoSourceSelectionRef> {
        let r = self.resolve(overlay.function_selector())?;
        if self
            .keys
            .source(r.source_object_id())
            .is_none_or(|s| s.object().path != overlay.source_path())
        {
            return Err(failure(GoSourceOverlayErrorKind::Source));
        }
        Ok(r)
    }
    fn selector_receiver(
        &self,
        selector: &GoSourceFunctionSelector,
    ) -> OverlayResult<Option<GoSourceSelectionRef>> {
        selector
            .receiver
            .as_ref()
            .map(|r| {
                let package = self.package(&r.package)?;
                let binding = self
                    .scopes
                    .bindings()
                    .iter()
                    .find(|b| {
                        b.package_id() == package.id
                            && b.name() == r.type_name
                            && b.kind() == GoSourceNameKind::Type
                            && self.scopes.package_binding(b)
                    })
                    .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))?;
                self.receiver_declaration(binding.source_object_id(), binding.declaration_node())?
                    .ok_or_else(|| failure(GoSourceOverlayErrorKind::TypeSelector))
            })
            .transpose()
    }
    fn resolve(&self, selector: &GoSourceFunctionSelector) -> OverlayResult<GoSourceSelectionRef> {
        if !selector.validate() {
            return Err(failure(GoSourceOverlayErrorKind::Selector));
        }
        let package = self.package(&selector.package)?;
        let receiver = self.selector_receiver(selector)?;
        let mut matches = vec![];
        for source in &package.source_objects {
            let parsed = self
                .scopes
                .source(source)
                .ok_or_else(|| failure(GoSourceOverlayErrorKind::Source))?;
            for (node, n) in parsed.nodes().iter().enumerate() {
                if n.kind() != "FuncDecl" {
                    continue;
                }
                let name = source_signatures::list(n, "Name")
                    .first()
                    .and_then(|&i| source_signatures::attr(&parsed.nodes()[i], "Name"));
                if name != Some(selector.function.as_str()) {
                    continue;
                }
                let recv = source_signatures::list(n, "Recv")
                    .first()
                    .and_then(|&f| source_signatures::list(&parsed.nodes()[f], "List").first())
                    .and_then(|&f| source_signatures::list(&parsed.nodes()[f], "Type").first())
                    .copied();
                let matched = match (recv, &receiver) {
                    (None, None) => true,
                    (Some(typ), Some(id)) => self
                        .receiver_declaration(source, typ)
                        .is_ok_and(|r| r.as_ref() == Some(id)),
                    _ => false,
                };
                if matched {
                    matches.push(GoSourceSelectionRef::new(source, node));
                }
            }
        }
        if matches.len() != 1 {
            return Err(failure(GoSourceOverlayErrorKind::Selector));
        }
        Ok(matches.pop().unwrap())
    }
    fn bind(&self, overlay: &GoSourceOverlay) -> OverlayResult<GoBoundSourceOverlay> {
        if self.keys.sources.values().any(|s| {
            s.object().capability == overlay.raw.object().capability
                && s.object().path == overlay.raw.object().path
        }) {
            return Err(failure(GoSourceOverlayErrorKind::Source));
        }
        let root = self.function(overlay)?;
        let signature = source_signatures::derive_shared_signature(
            &self.scopes,
            root.source_object_id(),
            root.node(),
        )
        .map_err(|e| failure(GoSourceOverlayErrorKind::Signature(e)))?;
        let graph = signature.types();
        let types = graph
            .types()
            .map(|t| (t.id(), t))
            .collect::<BTreeMap<_, _>>();
        let shape = |id: &str| -> &GoSourceTypeKey { types[graph.underlying(id).unwrap_or(id)] };
        let inputs = signature
            .slots()
            .iter()
            .filter(|s| s.role() != GoSourceSignatureRole::Result)
            .collect::<Vec<_>>();
        let mut unique = BTreeSet::new();
        let mut entries = vec![];
        let mut cases = BTreeMap::new();
        for (row, entry) in overlay.wire.error_inputs.iter().enumerate() {
            let result = (|| {
                let mut typ = inputs
                    .get(entry.input)
                    .ok_or_else(|| failure(GoSourceOverlayErrorKind::EntryPath))?
                    .type_id();
                let mut path = vec![];
                for step in &entry.path {
                    match (step, shape(typ)) {
                        (PathStep::Field { name }, GoSourceTypeKey::Struct { fields, .. }) => {
                            let Some((ordinal, field)) =
                                fields.iter().enumerate().find(|(_, f)| f.name == *name)
                            else {
                                return Err(failure(GoSourceOverlayErrorKind::EntryPath));
                            };
                            typ = &field.type_id;
                            path.push(GoErrorEntryStep::Field(ordinal));
                        }
                        (
                            PathStep::Elements,
                            GoSourceTypeKey::Array { element, .. }
                            | GoSourceTypeKey::Slice { element, .. },
                        ) => {
                            typ = element;
                            path.push(GoErrorEntryStep::Elements);
                        }
                        _ => return Err(failure(GoSourceOverlayErrorKind::EntryPath)),
                    }
                }
                if !matches!(types[typ],GoSourceTypeKey::Builtin{name,..}if name=="error") {
                    return Err(failure(GoSourceOverlayErrorKind::EntryPath));
                }
                if !unique.insert((entry.input, path.clone())) {
                    return Err(failure(GoSourceOverlayErrorKind::DuplicateEntry));
                }
                let mut tags = BTreeSet::new();
                let mut refs = vec![];
                for name in &entry.cases {
                    if !cases.contains_key(name) {
                        cases.insert(name.clone(), self.type_ref(name)?);
                    }
                    let (r, id) = &cases[name];
                    if !tags.insert(id.clone()) {
                        return Err(failure(GoSourceOverlayErrorKind::DuplicateCase));
                    }
                    refs.push(r.clone());
                }
                Ok(GoErrorEntryConstraint::new(
                    root.clone(),
                    entry.input,
                    path,
                    refs,
                ))
            })();
            entries.push(result.map_err(|mut e: GoSourceOverlayError| {
                e.row = Some(row);
                e
            })?);
        }
        // Count wildcard source-error paths over the type DAG, saturating just
        // above the supplied row count. Never expand exponentially many paths.
        let cap = entries.len() + 1;
        let mut counts = BTreeMap::<String, usize>::new();
        for slot in &inputs {
            let mut work = vec![(slot.type_id().to_string(), false)];
            while let Some((id, finish)) = work.pop() {
                if counts.contains_key(&id) {
                    continue;
                }
                let t = shape(&id);
                let children: Vec<_> = match t {
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => vec![element.clone()],
                    GoSourceTypeKey::Struct { fields, .. } => {
                        fields.iter().map(|f| f.type_id.clone()).collect()
                    }
                    _ => vec![],
                };
                if !finish {
                    work.push((id.clone(), true));
                    work.extend(children.into_iter().map(|id| (id, false)));
                    continue;
                }
                let n = if matches!(t,GoSourceTypeKey::Builtin{name,..}if name=="error") {
                    1
                } else {
                    children
                        .iter()
                        .fold(0usize, |n, id| n.saturating_add(counts[id]).min(cap))
                };
                counts.insert(id, n);
            }
        }
        if inputs.iter().fold(0usize, |n, s| {
            n.saturating_add(counts[s.type_id()]).min(cap)
        }) != entries.len()
        {
            return Err(failure(GoSourceOverlayErrorKind::MissingEntry));
        }
        let parsed = self.scopes.source(root.source_object_id()).unwrap();
        let mut loops = original_loops(&self.scopes, &root, &signature)?;
        for (row, annotation) in overlay.wire.loops.iter().enumerate() {
            let s = &annotation.selector;
            let target = loops
                .get_mut(s.ordinal)
                .ok_or_else(|| failure(GoSourceOverlayErrorKind::LoopSelector))?;
            let node = &parsed.nodes()[target.node];
            let kind = if node.kind() == "ForStmt" {
                "for"
            } else {
                "range"
            };
            if node.start() != s.start_byte || node.end() != s.end_byte || kind != s.kind {
                return Err(failure(GoSourceOverlayErrorKind::LoopSelector));
            }
            if target.overlay_row.replace(row).is_some() {
                return Err(failure(GoSourceOverlayErrorKind::DuplicateLoop));
            }
        }
        Ok(GoBoundSourceOverlay {
            overlay: overlay.clone(),
            scopes: Arc::clone(&self.scopes),
            root,
            signature,
            entries,
            loops,
        })
    }
}

/// Resolve all raw candidate roots against the same original source scopes.
/// Build one name index per selected package so the frozen 8,192-root bound
/// does not require rescanning the whole source universe for every root.
pub fn resolve_go_practical_named_roots(
    keys: &GoTypeKeyComponents,
    selection: &crate::go_practical_selection::GoPracticalSelection,
) -> std::result::Result<Vec<GoSourceSelectionRef>, GoPracticalSourceBindingError> {
    use GoPracticalSourceBindingError as E;
    let roots = selection.roots();
    let scopes = Arc::new(build_go_source_scopes(keys).map_err(E::Scope)?);
    let binder = Binder { keys, scopes };
    type RootKey = (String, String, Option<GoSourceSelectionRef>);
    let mut wanted = BTreeMap::<RootKey, usize>::new();
    let mut names = BTreeMap::<String, BTreeSet<String>>::new();
    for (index, root) in roots.iter().enumerate() {
        let selector = GoSourceFunctionSelector {
            package: root.package().to_owned(),
            function: root.function().to_owned(),
            receiver: root.receiver().map(|receiver| GoSourceReceiverSelector {
                package: receiver.package().to_owned(),
                type_name: receiver.type_name().to_owned(),
            }),
        };
        if !selector.validate() {
            return Err(E::Resolve {
                root: Some(index),
                error: failure(GoSourceOverlayErrorKind::Selector),
            });
        }
        let package = binder
            .package(&selector.package)
            .map_err(|error| E::Resolve {
                root: Some(index),
                error,
            })?;
        let receiver = binder
            .selector_receiver(&selector)
            .map_err(|error| E::Resolve {
                root: Some(index),
                error,
            })?;
        let key = (package.id.clone(), selector.function, receiver);
        if let Some(first) = wanted.insert(key.clone(), index) {
            return Err(E::Duplicate {
                first,
                second: index,
            });
        }
        names.entry(key.0).or_default().insert(key.1);
    }
    let mut found = vec![None; roots.len()];
    for package in &keys.wire.packages {
        let Some(package_names) = names.get(&package.id) else {
            continue;
        };
        for source in &package.source_objects {
            let parsed = binder.scopes.source(source).ok_or_else(|| E::Resolve {
                root: None,
                error: failure(GoSourceOverlayErrorKind::Source),
            })?;
            for (node, declaration) in parsed.nodes().iter().enumerate() {
                if declaration.kind() != "FuncDecl" {
                    continue;
                }
                let Some(name) = source_signatures::list(declaration, "Name")
                    .first()
                    .and_then(|&i| source_signatures::attr(&parsed.nodes()[i], "Name"))
                else {
                    continue;
                };
                if !package_names.contains(name) {
                    continue;
                }
                let recv = source_signatures::list(declaration, "Recv")
                    .first()
                    .and_then(|&f| source_signatures::list(&parsed.nodes()[f], "List").first())
                    .and_then(|&f| source_signatures::list(&parsed.nodes()[f], "Type").first())
                    .copied();
                let receiver = match recv {
                    None => None,
                    Some(typ) => match binder.receiver_declaration(source, typ) {
                        Ok(Some(receiver)) => Some(receiver),
                        _ => continue,
                    },
                };
                let key = (package.id.clone(), name.to_owned(), receiver);
                if let Some(&index) = wanted.get(&key) {
                    let reference = GoSourceSelectionRef::new(source, node);
                    if found[index].replace(reference).is_some() {
                        return Err(E::Resolve {
                            root: Some(index),
                            error: failure(GoSourceOverlayErrorKind::Selector),
                        });
                    }
                }
            }
        }
    }
    found
        .into_iter()
        .enumerate()
        .map(|(index, root)| {
            root.ok_or_else(|| E::Resolve {
                root: Some(index),
                error: failure(GoSourceOverlayErrorKind::Selector),
            })
        })
        .collect()
}

fn original_loops(
    scopes: &GoSourceScopes,
    root: &GoSourceSelectionRef,
    signature: &GoSourceSignature,
) -> OverlayResult<Vec<GoBoundOverlayLoop>> {
    let parsed = scopes
        .source(root.source_object_id())
        .ok_or_else(|| failure(GoSourceOverlayErrorKind::Source))?;
    let mut nodes = vec![];
    let mut work = signature.body_node().into_iter().collect::<Vec<_>>();
    while let Some(node) = work.pop() {
        let n = &parsed.nodes()[node];
        if n.kind() == "FuncLit" {
            continue;
        }
        if matches!(n.kind(), "ForStmt" | "RangeStmt") {
            nodes.push(node);
        }
        for edge in n.edges() {
            work.extend(edge.targets());
        }
    }
    nodes.sort_by_key(|&n| parsed.nodes()[n].start());
    if nodes.len() > 1024 {
        return Err(failure(GoSourceOverlayErrorKind::Limit));
    }
    Ok(nodes
        .into_iter()
        .enumerate()
        .map(|(ordinal, node)| GoBoundOverlayLoop {
            node,
            ordinal,
            overlay_row: None,
        })
        .collect())
}
pub fn bind_go_source_overlay(
    keys: &GoTypeKeyComponents,
    overlay: &GoSourceOverlay,
) -> OverlayResult<GoBoundSourceOverlay> {
    let scopes = Arc::new(
        build_go_source_scopes(keys).map_err(|e| failure(GoSourceOverlayErrorKind::Scope(e)))?,
    );
    Binder { keys, scopes }.bind(overlay).map_err(|mut e| {
        e.overlay = Some(overlay.raw.object().id.clone());
        e
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceOverlaySelection {
    overlays: Vec<GoBoundSourceOverlay>,
    selection: GoSourceErrorSelection,
}
impl GoSourceOverlaySelection {
    pub fn overlays(&self) -> &[GoBoundSourceOverlay] {
        &self.overlays
    }
    pub fn selection(&self) -> &GoSourceErrorSelection {
        &self.selection
    }
}
/// Bind explicitly supplied ENTRY overlays and derive their finite source
/// selection. This is not a whole-module annotation set, normalized contract,
/// capture context, expression typing judgment or source/program proof.
pub fn derive_go_source_overlay_selection(
    keys: &GoTypeKeyComponents,
    overlays: &[GoSourceOverlay],
) -> OverlayResult<GoSourceOverlaySelection> {
    let scopes = Arc::new(
        build_go_source_scopes(keys).map_err(|e| failure(GoSourceOverlayErrorKind::Scope(e)))?,
    );
    let binder = Binder { keys, scopes };
    let mut bound = BTreeMap::new();
    let mut objects = BTreeSet::new();
    for overlay in overlays {
        if !objects.insert((&overlay.raw.object().capability, &overlay.raw.object().path)) {
            return Err(failure(GoSourceOverlayErrorKind::DuplicateOverlay));
        }
        let b = binder.bind(overlay).map_err(|mut e| {
            e.overlay = Some(overlay.raw.object().id.clone());
            e
        })?;
        if bound.insert(b.root.clone(), b).is_some() {
            return Err(failure(GoSourceOverlayErrorKind::DuplicateOverlay));
        }
    }
    let roots = bound.keys().cloned().collect::<Vec<_>>();
    let entries = bound
        .values()
        .flat_map(|b| b.entries.clone())
        .collect::<Vec<_>>();
    let selected = derive_go_source_error_selection(keys, &roots, &entries)
        .map_err(|e| failure(GoSourceOverlayErrorKind::Selection(Box::new(e))))?;
    Ok(GoSourceOverlaySelection {
        overlays: bound.into_values().collect(),
        selection: selected,
    })
}
