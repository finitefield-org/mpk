//! Source-derived expression types and lexical/snapshot references. This is an
//! internal typed tree, not the capture-dependent normalized-contract wire.
//! In particular, typing a partial operation does not prove its definedness.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoContractExpressionType {
    Bool,
    Nat,
    Int,
    Source(String),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoContractPhase {
    Requires,
    Ensures,
    Invariant,
    Variant,
}
impl GoContractPhase {
    fn header(self) -> bool {
        matches!(self, Self::Invariant | Self::Variant)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoContractObservation {
    EntryArguments,
    ReturnValues,
    Entry,
    Return,
    Header { loop_node: usize },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoContractExpressionReference {
    Slot {
        role: GoSourceSignatureRole,
        position: usize,
    },
    Binding {
        binding: GoSourceNameBinding,
        loop_node: usize,
    },
    Bound {
        depth: usize,
    },
    Range {
        loop_node: usize,
    },
    Field {
        owner_type_id: String,
        ordinal: usize,
    },
    ErrorCase {
        type_id: String,
    },
}
/// Read-only typed derivation. Original function/loop/name nodes remain source
/// references; no canonical binding IDs or semantic-context hashes are minted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoContractExpression {
    op: String,
    typ: GoContractExpressionType,
    children: Vec<GoContractExpression>,
    literal: Option<Value>,
    reference: Option<GoContractExpressionReference>,
    observation: Option<GoContractObservation>,
}
impl GoContractExpression {
    pub fn operator(&self) -> &str {
        &self.op
    }
    pub fn expression_type(&self) -> &GoContractExpressionType {
        &self.typ
    }
    /// Ordered args, or value; quantifiers use lower, upper, body order.
    pub fn children(&self) -> &[Self] {
        &self.children
    }
    pub fn literal(&self) -> Option<&Value> {
        self.literal.as_ref()
    }
    pub fn reference(&self) -> Option<&GoContractExpressionReference> {
        self.reference.as_ref()
    }
    pub fn observation(&self) -> Option<GoContractObservation> {
        self.observation
    }
    fn new(op: &str, typ: GoContractExpressionType, children: Vec<Self>) -> Self {
        Self {
            op: op.into(),
            typ,
            children,
            literal: None,
            reference: None,
            observation: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoContractTypingErrorKind {
    Overlay(Box<GoSourceOverlayError>),
    Domain(Box<GoSourceDomainError>),
    Source,
    Type,
    Scope,
    Literal,
    Case,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoContractTypingError {
    kind: GoContractTypingErrorKind,
    overlay: Option<String>,
    phase: Option<GoContractPhase>,
    loop_ordinal: Option<usize>,
    clause: Option<usize>,
    path: Vec<usize>,
}
impl GoContractTypingError {
    pub fn kind(&self) -> &GoContractTypingErrorKind {
        &self.kind
    }
    pub fn overlay_object_id(&self) -> Option<&str> {
        self.overlay.as_deref()
    }
    pub fn phase(&self) -> Option<GoContractPhase> {
        self.phase
    }
    pub fn loop_ordinal(&self) -> Option<usize> {
        self.loop_ordinal
    }
    pub fn clause(&self) -> Option<usize> {
        self.clause
    }
    /// Child indices from the clause root, in the same order as children().
    pub fn expression_path(&self) -> &[usize] {
        &self.path
    }
}
impl std::fmt::Display for GoContractTypingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go contract typing error: {self:?}")
    }
}
impl std::error::Error for GoContractTypingError {}
type Checked<T> = std::result::Result<T, GoContractTypingError>;
fn error(kind: GoContractTypingErrorKind) -> GoContractTypingError {
    GoContractTypingError {
        kind,
        overlay: None,
        phase: None,
        loop_ordinal: None,
        clause: None,
        path: vec![],
    }
}
fn overlay_error(e: GoSourceOverlayError) -> GoContractTypingError {
    let overlay = e.overlay_object_id().map(str::to_string);
    let mut result = error(GoContractTypingErrorKind::Overlay(Box::new(e)));
    result.overlay = overlay;
    result
}
fn require(value: bool, kind: GoContractTypingErrorKind) -> Checked<()> {
    if value {
        Ok(())
    } else {
        Err(error(kind))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoContractHeaderBinding {
    binding: GoSourceNameBinding,
    typ: String,
}
impl GoContractHeaderBinding {
    pub fn binding(&self) -> &GoSourceNameBinding {
        &self.binding
    }
    pub fn type_id(&self) -> &str {
        &self.typ
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoTypedOverlayLoop {
    node: usize,
    ordinal: usize,
    annotation: Option<usize>,
    bindings: Vec<GoContractHeaderBinding>,
    invariants: Vec<GoContractExpression>,
    variant: Option<GoContractExpression>,
}
impl GoTypedOverlayLoop {
    pub fn node(&self) -> usize {
        self.node
    }
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn annotation_row(&self) -> Option<usize> {
        self.annotation
    }
    pub fn bindings(&self) -> &[GoContractHeaderBinding] {
        &self.bindings
    }
    pub fn invariants(&self) -> &[GoContractExpression] {
        &self.invariants
    }
    pub fn variant(&self) -> Option<&GoContractExpression> {
        self.variant.as_ref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoTypedSourceOverlay {
    root: GoSourceSelectionRef,
    overlay: String,
    requires: Vec<GoContractExpression>,
    ensures: Vec<GoContractExpression>,
    loops: Vec<GoTypedOverlayLoop>,
}
impl GoTypedSourceOverlay {
    pub fn root(&self) -> &GoSourceSelectionRef {
        &self.root
    }
    pub fn overlay_object_id(&self) -> &str {
        &self.overlay
    }
    pub fn requires(&self) -> &[GoContractExpression] {
        &self.requires
    }
    pub fn ensures(&self) -> &[GoContractExpression] {
        &self.ensures
    }
    pub fn loops(&self) -> &[GoTypedOverlayLoop] {
        &self.loops
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoTypedSourceOverlaySet {
    original: GoSourceOverlaySelection,
    overlays: Vec<GoTypedSourceOverlay>,
    domain: GoSourceValueDomain,
    type_dependencies: Vec<GoSourceSelectionRef>,
}
impl GoTypedSourceOverlaySet {
    pub fn source_selection(&self) -> &GoSourceOverlaySelection {
        &self.original
    }
    pub fn overlays(&self) -> &[GoTypedSourceOverlay] {
        &self.overlays
    }
    pub fn value_domain(&self) -> &GoSourceValueDomain {
        &self.domain
    }
    /// Original TypeSpecs referenced by clauses, including aliases. These are
    /// compilation dependencies, not requests to execute any method body.
    pub fn type_dependencies(&self) -> &[GoSourceSelectionRef] {
        &self.type_dependencies
    }
}

#[derive(Clone)]
struct Context {
    phase: GoContractPhase,
    loop_node: Option<usize>,
    range: bool,
    entry: bool,
    variables: Vec<String>,
}
impl Context {
    fn observation(&self) -> GoContractObservation {
        if self.entry || self.phase == GoContractPhase::Requires {
            GoContractObservation::Entry
        } else if self.phase == GoContractPhase::Ensures {
            GoContractObservation::Return
        } else {
            GoContractObservation::Header {
                loop_node: self.loop_node.unwrap(),
            }
        }
    }
}
struct Checker<'a> {
    bound: &'a GoBoundSourceOverlay,
    domain: &'a GoSourceValueDomain,
    types: &'a BTreeMap<String, GoSourceTypeKey>,
    names: &'a BTreeMap<String, String>,
    headers: BTreeMap<usize, Vec<GoContractHeaderBinding>>,
}
impl Checker<'_> {
    fn source<'a>(&self, typ: &'a GoContractExpressionType) -> Checked<&'a str> {
        match typ {
            GoContractExpressionType::Source(id) if self.domain.properties(id).is_some() => Ok(id),
            _ => Err(error(GoContractTypingErrorKind::Type)),
        }
    }
    fn shape(&self, typ: &GoContractExpressionType) -> Checked<&GoSourceTypeKey> {
        let id = self.source(typ)?;
        self.types
            .get(self.domain.underlying(id).unwrap_or(id))
            .ok_or_else(|| error(GoContractTypingErrorKind::Type))
    }
    fn resolve(&self, name: &str) -> Checked<GoContractExpressionType> {
        self.names
            .get(name)
            .cloned()
            .map(GoContractExpressionType::Source)
            .ok_or_else(|| error(GoContractTypingErrorKind::Source))
    }
    fn word(&self, typ: &GoContractExpressionType) -> Checked<(u8, bool)> {
        match self.shape(typ)? {
            GoSourceTypeKey::Builtin {
                width: Some(width),
                signed: Some(signed),
                ..
            } => Ok((*width, *signed)),
            _ => Err(error(GoContractTypingErrorKind::Type)),
        }
    }
    fn string(&self, typ: &GoContractExpressionType) -> Checked<()> {
        require(
            matches!(self.shape(typ)?, GoSourceTypeKey::Builtin {name,..} if name == "string"),
            GoContractTypingErrorKind::Type,
        )
    }
    fn comparable(&self, id: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut todo = vec![id];
        while let Some(id) = todo.pop() {
            if !seen.insert(id) {
                continue;
            }
            match self.types.get(self.domain.underlying(id).unwrap_or(id)) {
                Some(GoSourceTypeKey::Builtin { name, .. }) if name != "error" => (),
                Some(GoSourceTypeKey::Array { element, .. }) => todo.push(element),
                Some(GoSourceTypeKey::Struct { fields, .. }) => {
                    todo.extend(fields.iter().map(|f| f.type_id.as_str()))
                }
                _ => return false,
            }
        }
        true
    }
    fn child(&self, e: &Value, ctx: &Context, index: usize) -> Checked<GoContractExpression> {
        self.expression(e, ctx).map_err(|mut e| {
            e.path.insert(0, index);
            e
        })
    }
    fn expression(&self, e: &Value, ctx: &Context) -> Checked<GoContractExpression> {
        use GoContractExpressionType::{Bool, Int, Nat, Source};
        use GoContractTypingErrorKind::{Case, Literal, Scope, Type};
        let op = e["op"].as_str().unwrap(); // closed source preflight ran first
        let make = |typ, children| GoContractExpression::new(op, typ, children);
        if matches!(op, "bool.literal" | "nat.literal" | "int.literal") {
            let mut out = make(
                match op {
                    "bool.literal" => Bool,
                    "nat.literal" => Nat,
                    _ => Int,
                },
                vec![],
            );
            out.literal = Some(e["value"].clone());
            return Ok(out);
        }
        if matches!(op, "word.literal" | "bytes.literal" | "source.zero") {
            let typ = self.resolve(e["type"].as_str().unwrap())?;
            if op == "word.literal" {
                let (width, signed) = self.word(&typ)?;
                // Every admitted 64-bit endpoint fits i128. Larger decimal
                // spellings are out of range, not a private literal-size cap.
                let n = e["value"]
                    .as_str()
                    .unwrap()
                    .parse::<i128>()
                    .map_err(|_| error(Literal))?;
                let (lo, hi) = if signed {
                    (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
                } else {
                    (0, (1i128 << width) - 1)
                };
                require(n >= lo && n <= hi, Literal)?;
            } else if op == "bytes.literal" {
                self.string(&typ)?;
            }
            self.source(&typ)?;
            let mut out = make(typ, vec![]);
            if op != "source.zero" {
                out.literal = Some(e["value"].clone());
            }
            return Ok(out);
        }
        if matches!(op, "input" | "result") {
            let result = op == "result";
            require(
                !result || ctx.phase == GoContractPhase::Ensures && !ctx.entry,
                Scope,
            )?;
            let position =
                usize::try_from(e["position"].as_u64().unwrap()).map_err(|_| error(Scope))?;
            let slot = self
                .bound
                .signature
                .slots()
                .iter()
                .filter(|s| (s.role() == GoSourceSignatureRole::Result) == result)
                .nth(position)
                .ok_or_else(|| error(Scope))?;
            let mut out = make(Source(slot.type_id().into()), vec![]);
            self.source(&out.typ)?;
            out.reference = Some(GoContractExpressionReference::Slot {
                role: slot.role(),
                position,
            });
            out.observation = Some(if result {
                GoContractObservation::ReturnValues
            } else {
                GoContractObservation::EntryArguments
            });
            return Ok(out);
        }
        if op == "local" {
            require(ctx.phase.header() && !ctx.entry, Scope)?;
            let loop_node = ctx.loop_node.ok_or_else(|| error(Scope))?;
            let name = e["name"].as_str().unwrap();
            let header = self.headers[&loop_node]
                .iter()
                .find(|b| b.binding.name() == name)
                .ok_or_else(|| error(Scope))?;
            let mut out = make(Source(header.typ.clone()), vec![]);
            self.source(&out.typ)?;
            out.reference = Some(GoContractExpressionReference::Binding {
                binding: header.binding.clone(),
                loop_node,
            });
            out.observation = Some(GoContractObservation::Header { loop_node });
            return Ok(out);
        }
        if op == "bound" {
            let name = e["name"].as_str().unwrap();
            let index = ctx
                .variables
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| error(Scope))?;
            let mut out = make(Nat, vec![]);
            out.reference = Some(GoContractExpressionReference::Bound {
                depth: ctx.variables.len() - 1 - index,
            });
            return Ok(out);
        }
        if matches!(op, "range.index" | "range.length") {
            require(ctx.phase.header() && ctx.range && !ctx.entry, Scope)?;
            let loop_node = ctx.loop_node.ok_or_else(|| error(Scope))?;
            let mut out = make(Nat, vec![]);
            out.reference = Some(GoContractExpressionReference::Range { loop_node });
            out.observation = Some(GoContractObservation::Header { loop_node });
            return Ok(out);
        }
        if op == "old" {
            require(!ctx.entry, Scope)?;
            let mut entry = ctx.clone();
            entry.entry = true;
            let value = self.child(&e["value"], &entry, 0)?;
            let mut out = make(value.typ.clone(), vec![value]);
            out.observation = Some(GoContractObservation::Entry);
            return Ok(out);
        }
        if matches!(op, "forall" | "exists" | "nat.sum" | "int.sum") {
            let name = e["name"].as_str().unwrap();
            require(!ctx.variables.iter().any(|n| n == name), Scope)?;
            let lower = self.child(&e["lower"], ctx, 0)?;
            let upper = self.child(&e["upper"], ctx, 1)?;
            require(lower.typ == Nat && upper.typ == Nat, Type)?;
            let mut inner = ctx.clone();
            inner.variables.push(name.into());
            let body = self.child(&e["body"], &inner, 2)?;
            let typ = match op {
                "nat.sum" => Nat,
                "int.sum" => Int,
                _ => Bool,
            };
            require(body.typ == typ, Type)?;
            return Ok(make(typ, vec![lower, upper, body]));
        }
        if op == "word.convert" {
            let value = self.child(&e["value"], ctx, 0)?;
            self.word(&value.typ)?;
            let typ = self.resolve(e["type"].as_str().unwrap())?;
            self.word(&typ)?;
            return Ok(make(typ, vec![value]));
        }
        if op == "field" {
            let value = self.child(&e["value"], ctx, 0)?;
            let owner = self.source(&value.typ)?.to_string();
            let GoSourceTypeKey::Struct { fields, .. } = self.shape(&value.typ)? else {
                return Err(error(Type));
            };
            let name = e["field"].as_str().unwrap();
            let (ordinal, field) = fields
                .iter()
                .enumerate()
                .find(|(_, f)| f.name == name)
                .ok_or_else(|| error(Type))?;
            let mut out = make(Source(field.type_id.clone()), vec![value]);
            out.reference = Some(GoContractExpressionReference::Field {
                owner_type_id: owner,
                ordinal,
            });
            return Ok(out);
        }
        if matches!(op, "error.is" | "error.payload") {
            let value = self.child(&e["value"], ctx, 0)?;
            require(
                matches!(self.shape(&value.typ)?, GoSourceTypeKey::Builtin {name,..} if name == "error"),
                Type,
            )?;
            let case = self.resolve(e["case"].as_str().unwrap())?;
            let id = self.source(&case)?;
            require(
                self.domain
                    .error_cases()
                    .iter()
                    .any(|c| c.value_type_id() == id),
                Case,
            )?;
            let mut out = make(
                if op == "error.is" { Bool } else { case.clone() },
                vec![value],
            );
            out.reference = Some(GoContractExpressionReference::ErrorCase { type_id: id.into() });
            return Ok(out);
        }
        // Every remaining head has an exact arity checked by syntax preflight.
        let children = e["args"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, child)| self.child(child, ctx, i))
            .collect::<Checked<Vec<_>>>()?;
        let a = &children[0].typ;
        let b = children.get(1).map(|c| &c.typ);
        let all = |typ: &GoContractExpressionType| children.iter().all(|c| &c.typ == typ);
        let typ = match op {
            "if" => {
                require(*a == Bool && children[1].typ == children[2].typ, Type)?;
                children[1].typ.clone()
            }
            "bool.not" | "bool.and" | "bool.or" | "bool.implies" => {
                require(all(&Bool), Type)?;
                Bool
            }
            "nat.add" | "nat.mul" | "nat.sub" => {
                require(all(&Nat), Type)?;
                Nat
            }
            "int.neg" | "int.add" | "int.mul" | "int.sub" => {
                require(all(&Int), Type)?;
                Int
            }
            "nat.to_int" => {
                require(*a == Nat, Type)?;
                Int
            }
            "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {
                require(Some(a) == b, Type)?;
                if matches!(op, "eq" | "ne") {
                    require(
                        match a {
                            Source(id) => self.comparable(id),
                            _ => true,
                        },
                        Type,
                    )?;
                } else if !matches!(a, Nat | Int) {
                    require(
                        matches!(
                            self.shape(a)?,
                            GoSourceTypeKey::Builtin { width: Some(_), .. }
                        ) || self.string(a).is_ok(),
                        Type,
                    )?;
                }
                Bool
            }
            "word.neg" | "word.not" | "word.add" | "word.sub" | "word.mul" | "word.and"
            | "word.or" | "word.xor" | "word.andnot" | "word.div" | "word.rem" | "word.shl"
            | "word.shr" => {
                for child in &children {
                    self.word(&child.typ)?;
                }
                require(
                    matches!(op, "word.shl" | "word.shr") || children.len() == 1 || Some(a) == b,
                    Type,
                )?;
                a.clone()
            }
            "bool.value" => {
                require(
                    matches!(self.shape(a)?, GoSourceTypeKey::Builtin {name,..} if name == "bool"),
                    Type,
                )?;
                Bool
            }
            "nat.from_unsigned" | "nat.from_signed" | "int.from_unsigned" | "int.from_signed" => {
                let (_, signed) = self.word(a)?;
                require(signed == op.ends_with("from_signed"), Type)?;
                if op.starts_with("nat.") {
                    Nat
                } else {
                    Int
                }
            }
            "length" | "capacity" | "is_nil" | "byte_at" | "element" => {
                let shape = self.shape(a)?;
                let valid = match op {
                    "length" => {
                        matches!(
                            shape,
                            GoSourceTypeKey::Array { .. } | GoSourceTypeKey::Slice { .. }
                        ) || matches!(shape, GoSourceTypeKey::Builtin {name,..} if name == "string")
                    }
                    "capacity" | "element" => matches!(
                        shape,
                        GoSourceTypeKey::Array { .. } | GoSourceTypeKey::Slice { .. }
                    ),
                    "is_nil" => {
                        matches!(shape, GoSourceTypeKey::Slice { .. })
                            || matches!(shape, GoSourceTypeKey::Builtin {name,..} if name == "error")
                    }
                    _ => matches!(shape, GoSourceTypeKey::Builtin {name,..} if name == "string"),
                };
                require(valid && b.is_none_or(|t| *t == Nat), Type)?;
                match op {
                    "is_nil" => Bool,
                    "byte_at" => self.resolve("uint8")?,
                    "element" => match shape {
                        GoSourceTypeKey::Array { element, .. }
                        | GoSourceTypeKey::Slice { element, .. } => Source(element.clone()),
                        _ => unreachable!(),
                    },
                    _ => Nat,
                }
            }
            "bytes.prefix" | "bytes.suffix" | "bytes.concat" | "bytes.slice" => {
                self.string(a)?;
                require(
                    if op == "bytes.slice" {
                        children[1..].iter().all(|c| c.typ == Nat)
                    } else {
                        Some(a) == b
                    },
                    Type,
                )?;
                if matches!(op, "bytes.prefix" | "bytes.suffix") {
                    Bool
                } else {
                    a.clone()
                }
            }
            "slice.view2" | "slice.view3" => {
                require(
                    matches!(self.shape(a)?, GoSourceTypeKey::Slice { .. })
                        && children[1..].iter().all(|c| c.typ == Nat),
                    Type,
                )?;
                a.clone()
            }
            "sequence.equal" => {
                let elements = children
                    .iter()
                    .map(|c| match self.shape(&c.typ)? {
                        GoSourceTypeKey::Array { element, .. }
                        | GoSourceTypeKey::Slice { element, .. } => Ok(element.as_str()),
                        _ => Err(error(Type)),
                    })
                    .collect::<Checked<Vec<_>>>()?;
                require(
                    elements[0] == elements[1] && self.comparable(elements[0]),
                    Type,
                )?;
                Bool
            }
            _ => return Err(error(Type)),
        };
        let mut out = make(typ, children);
        if matches!(op, "element" | "sequence.equal") {
            out.observation = Some(ctx.observation());
        }
        Ok(out)
    }
    fn clause(
        &self,
        expression: &Value,
        phase: GoContractPhase,
        clause: usize,
        loop_row: Option<&GoBoundOverlayLoop>,
    ) -> Checked<GoContractExpression> {
        let context = Context {
            phase,
            loop_node: loop_row.map(|l| l.node),
            range: loop_row.is_some_and(|l| {
                self.bound
                    .scopes
                    .source(self.bound.root.source_object_id())
                    .unwrap()
                    .nodes()[l.node]
                    .kind()
                    == "RangeStmt"
            }),
            entry: false,
            variables: vec![],
        };
        let checked = self.expression(expression, &context).and_then(|e| {
            let typ = if phase == GoContractPhase::Variant {
                GoContractExpressionType::Nat
            } else {
                GoContractExpressionType::Bool
            };
            require(e.typ == typ, GoContractTypingErrorKind::Type)?;
            Ok(e)
        });
        checked.map_err(|mut e| {
            e.overlay = Some(self.bound.overlay.raw.object().id.clone());
            e.phase = Some(phase);
            e.loop_ordinal = loop_row.map(|l| l.ordinal);
            e.clause = Some(clause);
            e
        })
    }
    fn check(&self) -> Checked<GoTypedSourceOverlay> {
        let wire = &self.bound.overlay.wire;
        let requires = wire
            .requires
            .iter()
            .enumerate()
            .map(|(i, e)| self.clause(e, GoContractPhase::Requires, i, None))
            .collect::<Checked<_>>()?;
        let ensures = wire
            .ensures
            .iter()
            .enumerate()
            .map(|(i, e)| self.clause(e, GoContractPhase::Ensures, i, None))
            .collect::<Checked<_>>()?;
        let mut loops = vec![];
        for l in &self.bound.loops {
            let mut row = GoTypedOverlayLoop {
                node: l.node,
                ordinal: l.ordinal,
                annotation: l.overlay_row,
                bindings: self.headers[&l.node].clone(),
                invariants: vec![],
                variant: None,
            };
            if let Some(index) = l.overlay_row {
                let raw = &wire.loops[index];
                row.invariants = raw
                    .invariants
                    .iter()
                    .enumerate()
                    .map(|(i, e)| self.clause(e, GoContractPhase::Invariant, i, Some(l)))
                    .collect::<Checked<_>>()?;
                if let Variant::Candidate { expression } = &raw.variant {
                    row.variant =
                        Some(self.clause(expression, GoContractPhase::Variant, 0, Some(l))?);
                }
            }
            loops.push(row);
        }
        Ok(GoTypedSourceOverlay {
            root: self.bound.root.clone(),
            overlay: self.bound.overlay.raw.object().id.clone(),
            requires,
            ensures,
            loops,
        })
    }
}

/// Bind original entry overlays, derive finite Error selection and infer all
/// clause types. Optional callee/default contracts, canonical scope/context
/// reconstruction, partial-operation obligations and proofs remain separate.
pub fn derive_go_typed_source_overlays(
    keys: &GoTypeKeyComponents,
    overlays: &[GoSourceOverlay],
) -> Checked<GoTypedSourceOverlaySet> {
    let original = derive_go_source_overlay_selection(keys, overlays).map_err(overlay_error)?;
    let mut names = BTreeMap::new();
    let mut graphs = vec![];
    let mut builtins = vec![];
    let mut dependencies = BTreeSet::new();
    for bound in &original.overlays {
        let binder = Binder {
            keys,
            scopes: bound.scopes.clone(),
        };
        let wire = &bound.overlay.wire;
        let mut todo = wire
            .requires
            .iter()
            .chain(&wire.ensures)
            .collect::<Vec<_>>();
        for l in &wire.loops {
            todo.extend(&l.invariants);
            if let Variant::Candidate { expression } = &l.variant {
                todo.push(expression);
            }
        }
        let mut refs = BTreeSet::new();
        while let Some(e) = todo.pop() {
            if let Some(name) = e
                .get("type")
                .or_else(|| e.get("case"))
                .and_then(Value::as_str)
            {
                refs.insert(name);
            }
            if e["op"] == "byte_at" {
                refs.insert("uint8");
            }
            for key in ["value", "lower", "upper", "body"] {
                if e.get(key).is_some_and(Value::is_object) {
                    todo.push(&e[key]);
                }
            }
            if let Some(args) = e["args"].as_array() {
                todo.extend(args);
            }
        }
        for name in refs {
            if names.contains_key(name) {
                continue;
            }
            let resolved = (|| {
                if !name.contains('.') {
                    let key = source_types::builtin_source_type(&bound.scopes, name)
                        .map_err(|_| error(GoContractTypingErrorKind::Source))?;
                    let id = key.id().to_string();
                    builtins.push(key);
                    Ok(id)
                } else {
                    let (r, _) = binder.type_ref(name).map_err(overlay_error)?;
                    let graph = derive_go_source_type_declaration(
                        &bound.scopes,
                        r.source_object_id(),
                        r.node(),
                    )
                    .map_err(|_| error(GoContractTypingErrorKind::Source))?;
                    let id = graph.roots()[0].clone();
                    graphs.push(graph);
                    dependencies.insert(r);
                    Ok(id)
                }
            })();
            let id = resolved.map_err(|mut e: GoContractTypingError| {
                e.overlay = Some(bound.overlay.raw.object().id.clone());
                e
            })?;
            names.insert(name.to_string(), id);
        }
    }
    let domain = source_scalar_typing::extend_contract_domain(
        original.selection.domain(),
        &graphs,
        &builtins,
    )
    .map_err(|e| error(GoContractTypingErrorKind::Domain(Box::new(e))))?;
    let types = domain
        .types()
        .map(|t| (t.id().to_string(), t.clone()))
        .collect();
    let mut checked = vec![];
    for bound in &original.overlays {
        let mut headers = BTreeMap::new();
        for l in &bound.loops {
            let variables = bound
                .scopes
                .loop_header_variables(bound.root.source_object_id(), l.node)
                .map_err(|_| error(GoContractTypingErrorKind::Scope))?;
            let bindings = variables
                .into_iter()
                .map(|binding| {
                    let typ = if binding.declaration_node() == bound.root.node() {
                        bound
                            .signature
                            .slots()
                            .iter()
                            .find(|s| s.name_node() == Some(binding.name_node()))
                            .map(|s| s.type_id())
                    } else {
                        domain.selection().checked_local_type(&bound.root, &binding)
                    }
                    .ok_or_else(|| error(GoContractTypingErrorKind::Scope))?
                    .to_string();
                    Ok(GoContractHeaderBinding { binding, typ })
                })
                .collect::<Checked<_>>()?;
            headers.insert(l.node, bindings);
        }
        checked.push(
            Checker {
                bound,
                domain: &domain,
                types: &types,
                names: &names,
                headers,
            }
            .check()?,
        );
    }
    Ok(GoTypedSourceOverlaySet {
        original,
        overlays: checked,
        domain,
        type_dependencies: dependencies.into_iter().collect(),
    })
}
