//! Original declared binding and function-slot types, not whole-body checking
//! or call/state/program authority. Inferred bindings require expression typing.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceSignatureError {
    Locator,
    NotFunction,
    NotVariable,
    InferenceRequired,
    Duplicate,
    Unsupported,
    Limit,
    Type(GoSourceTypeError),
}
impl std::fmt::Display for GoSourceSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source signature error: {self:?}")
    }
}
impl std::error::Error for GoSourceSignatureError {}
type Result<T> = std::result::Result<T, GoSourceSignatureError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceSignatureRole {
    Receiver,
    Parameter,
    Result,
}
impl GoSourceSignatureRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Receiver => "receiver",
            Self::Parameter => "parameter",
            Self::Result => "result",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSignatureSlot {
    role: GoSourceSignatureRole,
    field_node: usize,
    name_node: Option<usize>,
    name: Option<String>,
    type_node: usize,
    type_id: String,
}
impl GoSourceSignatureSlot {
    pub fn role(&self) -> GoSourceSignatureRole {
        self.role
    }
    pub fn field_node(&self) -> usize {
        self.field_node
    }
    pub fn name_node(&self) -> Option<usize> {
        self.name_node
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn type_node(&self) -> usize {
        self.type_node
    }
    pub fn type_id(&self) -> &str {
        &self.type_id
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSignature {
    source: String,
    function_node: usize,
    type_node: usize,
    body_node: Option<usize>,
    name: Option<String>,
    slots: Vec<GoSourceSignatureSlot>,
    types: GoDerivedSourceTypes,
}
impl GoSourceSignature {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn function_node(&self) -> usize {
        self.function_node
    }
    pub fn type_node(&self) -> usize {
        self.type_node
    }
    pub fn body_node(&self) -> Option<usize> {
        self.body_node
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn slots(&self) -> &[GoSourceSignatureSlot] {
        &self.slots
    }
    pub fn types(&self) -> &GoDerivedSourceTypes {
        &self.types
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoDeclaredSourceBindingType {
    binding: GoSourceNameBinding,
    type_node: usize,
    types: GoDerivedSourceTypes,
}
impl GoDeclaredSourceBindingType {
    pub fn binding(&self) -> &GoSourceNameBinding {
        &self.binding
    }
    pub fn type_node(&self) -> usize {
        self.type_node
    }
    pub fn type_id(&self) -> &str {
        &self.types.roots()[0]
    }
    pub fn types(&self) -> &GoDerivedSourceTypes {
        &self.types
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
    let nodes = list(node, role);
    if nodes.len() == 1 {
        Ok(nodes[0])
    } else {
        Err(GoSourceSignatureError::Locator)
    }
}
struct Fields<'a> {
    parsed: &'a GoParsedSource,
    slots: Vec<GoSourceSignatureSlot>,
    names: BTreeSet<String>,
    roots: Vec<usize>,
    root_indices: BTreeMap<usize, usize>,
}
impl Fields<'_> {
    fn add(&mut self, fields: Option<usize>, role: GoSourceSignatureRole) -> Result<()> {
        let Some(fields) = fields else { return Ok(()) };
        let fields = &self.parsed.nodes()[fields];
        if fields.kind() != "FieldList" {
            return Err(GoSourceSignatureError::Locator);
        }
        let mut named = None;
        let begin = self.slots.len();
        for &field in list(fields, "List") {
            let f = &self.parsed.nodes()[field];
            if f.kind() != "Field" {
                return Err(GoSourceSignatureError::Locator);
            }
            let typ = child(f, "Type")?;
            if self.parsed.nodes()[typ].kind() == "Ellipsis" {
                return Err(GoSourceSignatureError::Unsupported);
            }
            let names = list(f, "Names");
            let has_names = !names.is_empty();
            if named.is_some_and(|old| old != has_names) {
                return Err(GoSourceSignatureError::Unsupported);
            }
            named = Some(has_names);
            let names: Vec<_> = if has_names {
                names.iter().copied().map(Some).collect()
            } else {
                vec![None]
            };
            for name_node in names {
                let name = name_node
                    .map(|id| {
                        attr(&self.parsed.nodes()[id], "Name")
                            .map(str::to_string)
                            .ok_or(GoSourceSignatureError::Locator)
                    })
                    .transpose()?;
                if let Some(name) = &name {
                    if name != "_" && !self.names.insert(name.clone()) {
                        return Err(GoSourceSignatureError::Duplicate);
                    }
                }
                self.slots.push(GoSourceSignatureSlot {
                    role,
                    field_node: field,
                    name_node,
                    name,
                    type_node: typ,
                    type_id: String::new(),
                });
                if self.slots.len() > 4_194_304 {
                    return Err(GoSourceSignatureError::Limit);
                }
            }
            if let std::collections::btree_map::Entry::Vacant(entry) = self.root_indices.entry(typ)
            {
                entry.insert(self.roots.len());
                self.roots.push(typ);
            }
        }
        let count = self.slots.len() - begin;
        if role == GoSourceSignatureRole::Receiver && count != 1
            || role == GoSourceSignatureRole::Result && count > 16
        {
            return Err(GoSourceSignatureError::Limit);
        }
        Ok(())
    }
}
/// Derive actual ordered receiver/input/result slot types. Function bodies are
/// retained by original locator but not typechecked or proved by this API.
pub fn derive_go_source_signature(
    scopes: &GoSourceScopes,
    source: &str,
    function: usize,
) -> Result<GoSourceSignature> {
    let parsed = scopes
        .source(source)
        .ok_or(GoSourceSignatureError::Locator)?;
    let node = parsed
        .nodes()
        .get(function)
        .ok_or(GoSourceSignatureError::Locator)?;
    let typ = match node.kind() {
        "FuncDecl" | "FuncLit" => child(node, "Type")?,
        "FuncType" => function,
        _ => return Err(GoSourceSignatureError::NotFunction),
    };
    let signature = &parsed.nodes()[typ];
    if signature.kind() != "FuncType" {
        return Err(GoSourceSignatureError::NotFunction);
    }
    if !list(signature, "TypeParams").is_empty() {
        return Err(GoSourceSignatureError::Unsupported);
    }
    let mut fields = Fields {
        parsed,
        slots: vec![],
        names: BTreeSet::new(),
        roots: vec![],
        root_indices: BTreeMap::new(),
    };
    fields.add(
        list(node, "Recv").first().copied(),
        GoSourceSignatureRole::Receiver,
    )?;
    fields.add(
        Some(child(signature, "Params")?),
        GoSourceSignatureRole::Parameter,
    )?;
    fields.add(
        list(signature, "Results").first().copied(),
        GoSourceSignatureRole::Result,
    )?;
    let types = source_types::derive_go_source_type_nodes(scopes, source, &fields.roots)
        .map_err(GoSourceSignatureError::Type)?;
    for slot in &mut fields.slots {
        slot.type_id = types.roots()[fields.root_indices[&slot.type_node]].clone();
    }
    let name = list(node, "Name")
        .first()
        .and_then(|&n| attr(&parsed.nodes()[n], "Name"))
        .map(str::to_string);
    Ok(GoSourceSignature {
        source: source.into(),
        function_node: function,
        type_node: typ,
        body_node: list(node, "Body").first().copied(),
        name,
        slots: fields.slots,
        types,
    })
}
/// Resolve the declared type of one actual original variable binding. An
/// initializer/body is not checked. Inferred locals retain InferenceRequired;
/// no default type or guessed source origin can stand in for expression typing.
pub fn derive_go_declared_binding_type(
    scopes: &GoSourceScopes,
    binding: &GoSourceNameBinding,
) -> Result<GoDeclaredSourceBindingType> {
    if !scopes.bindings().contains(binding) {
        return Err(GoSourceSignatureError::Locator);
    }
    if binding.kind() != GoSourceNameKind::Variable {
        return Err(GoSourceSignatureError::NotVariable);
    }
    let source = binding.source_object_id();
    let parsed = scopes
        .source(source)
        .ok_or(GoSourceSignatureError::Locator)?;
    let node = parsed
        .nodes()
        .get(binding.declaration_node())
        .ok_or(GoSourceSignatureError::Locator)?;
    let typ = match node.kind() {
        "ValueSpec" => {
            if !list(node, "Names").contains(&binding.name_node()) {
                return Err(GoSourceSignatureError::Locator);
            }
            list(node, "Type")
                .first()
                .copied()
                .ok_or(GoSourceSignatureError::InferenceRequired)?
        }
        "FuncDecl" | "FuncLit" => {
            // A binding's own annotation does not depend on unrelated slots
            // being in the profile (e.g. another pointer or variadic input).
            let signature = &parsed.nodes()[child(node, "Type")?];
            let mut found = None;
            for fields in [
                list(node, "Recv"),
                list(signature, "Params"),
                list(signature, "Results"),
            ]
            .into_iter()
            .flatten()
            {
                for &field in list(&parsed.nodes()[*fields], "List") {
                    let f = &parsed.nodes()[field];
                    if list(f, "Names").contains(&binding.name_node()) {
                        if found.is_some() {
                            return Err(GoSourceSignatureError::Locator);
                        }
                        found = Some(child(f, "Type")?);
                    }
                }
            }
            found.ok_or(GoSourceSignatureError::Locator)?
        }
        "AssignStmt" | "RangeStmt" | "TypeSwitchStmt" => {
            return Err(GoSourceSignatureError::InferenceRequired)
        }
        _ => return Err(GoSourceSignatureError::Unsupported),
    };
    let types = if parsed.nodes()[typ].kind() == "Ellipsis" {
        source_types::derive_go_variadic_binding_type(scopes, source, typ)
    } else {
        derive_go_source_type(scopes, source, typ)
    }
    .map_err(GoSourceSignatureError::Type)?;
    Ok(GoDeclaredSourceBindingType {
        binding: binding.clone(),
        type_node: typ,
        types,
    })
}
