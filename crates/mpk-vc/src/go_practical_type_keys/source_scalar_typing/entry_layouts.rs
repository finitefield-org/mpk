//! Source-owned function-entry bindings. Locals remain absent until their
//! declarations execute; unnamed/blank slots do not create variable cells.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceEntryBindingOrigin {
    Input(usize),
    NamedResult(usize),
    Local,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntryBinding {
    binding: GoSourceNameBinding,
    typ: String,
    origin: GoSourceEntryBindingOrigin,
}
impl GoSourceEntryBinding {
    pub fn binding(&self) -> &GoSourceNameBinding {
        &self.binding
    }
    pub fn type_id(&self) -> &str {
        &self.typ
    }
    pub fn origin(&self) -> GoSourceEntryBindingOrigin {
        self.origin
    }
    pub fn exists_at_entry(&self) -> bool {
        self.origin != GoSourceEntryBindingOrigin::Local
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntrySlot {
    typ: String,
    binding: Option<usize>,
}
impl GoSourceEntrySlot {
    pub fn type_id(&self) -> &str {
        &self.typ
    }
    pub fn binding_index(&self) -> Option<usize> {
        self.binding
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntryLayout {
    function: GoSourceSelectionRef,
    signature: GoSourceSignature,
    bindings: Vec<GoSourceEntryBinding>,
    inputs: Vec<GoSourceEntrySlot>,
    results: Vec<GoSourceEntrySlot>,
}
impl GoSourceEntryLayout {
    pub fn function(&self) -> &GoSourceSelectionRef {
        &self.function
    }
    pub fn signature(&self) -> &GoSourceSignature {
        &self.signature
    }
    pub fn bindings(&self) -> &[GoSourceEntryBinding] {
        &self.bindings
    }
    /// Complete argument positions, including blank or unnamed inputs.
    pub fn inputs(&self) -> &[GoSourceEntrySlot] {
        &self.inputs
    }
    /// Complete result positions. Only named, nonblank results have a cell.
    pub fn results(&self) -> &[GoSourceEntrySlot] {
        &self.results
    }
}

pub fn derive_go_source_entry_layout(
    domain: &GoSourceValueDomain,
    function: &GoSourceSelectionRef,
) -> Result<GoSourceEntryLayout> {
    let selection = domain.selection();
    let body = selection
        .checked_body_types(function)
        .ok_or(GoScalarTypingError::Locator)?;
    let signature = derive_go_source_signature(
        selection.scopes(),
        function.source_object_id(),
        function.node(),
    )
    .map_err(GoScalarTypingError::Signature)?;
    if signature.source_object_id() != function.source_object_id()
        || signature.function_node() != function.node()
        || signature.body_node().is_none()
    {
        return Err(GoScalarTypingError::Locator);
    }
    if signature
        .slots()
        .iter()
        .filter(|s| s.role() != GoSourceSignatureRole::Result)
        .map(|s| s.type_id())
        .ne(body.inputs.iter().map(String::as_str))
    {
        return Err(GoScalarTypingError::Type);
    }
    let keys = domain.types().map(|k| k.id()).collect::<BTreeSet<_>>();
    let mut bindings = vec![];
    let mut seen = BTreeSet::new();
    let mut inputs = vec![];
    let mut results = vec![];
    for slot in signature.slots() {
        if !keys.contains(slot.type_id()) {
            return Err(GoScalarTypingError::Type);
        }
        let is_result = slot.role() == GoSourceSignatureRole::Result;
        let position = if is_result {
            results.len()
        } else {
            inputs.len()
        };
        let origin = if is_result {
            GoSourceEntryBindingOrigin::NamedResult(position)
        } else {
            GoSourceEntryBindingOrigin::Input(position)
        };
        let binding_index = match slot.name() {
            None | Some("_") => None,
            Some(name) => {
                let name_node = slot.name_node().ok_or(GoScalarTypingError::Locator)?;
                let binding = selection
                    .scopes()
                    .bindings()
                    .iter()
                    .find(|b| {
                        b.source_object_id() == signature.source_object_id()
                            && b.declaration_node() == signature.function_node()
                            && b.name_node() == name_node
                            && b.name() == name
                            && b.kind() == GoSourceNameKind::Variable
                    })
                    .ok_or(GoScalarTypingError::Name)?;
                if !seen.insert((binding.declaration_node(), binding.name_node())) {
                    return Err(GoScalarTypingError::Name);
                }
                let index = bindings.len();
                bindings.push(GoSourceEntryBinding {
                    binding: binding.clone(),
                    typ: slot.type_id().into(),
                    origin,
                });
                Some(index)
            }
        };
        let row = GoSourceEntrySlot {
            typ: slot.type_id().into(),
            binding: binding_index,
        };
        if is_result {
            results.push(row);
        } else {
            inputs.push(row);
        }
    }
    let parsed = selection
        .source(function.source_object_id())
        .ok_or(GoScalarTypingError::Locator)?;
    let mut locals = selection
        .scopes()
        .bindings()
        .iter()
        .filter_map(|binding| {
            selection
                .checked_local_type(function, binding)
                .map(|typ| (binding, typ))
        })
        .map(|(binding, typ)| {
            let start = parsed
                .nodes()
                .get(binding.name_node())
                .ok_or(GoScalarTypingError::Locator)?
                .start();
            Ok((start, binding, typ))
        })
        .collect::<Result<Vec<_>>>()?;
    locals.sort_by_key(|(start, _, _)| *start);
    for (_, binding, typ) in locals {
        if !keys.contains(typ)
            || binding.source_object_id() != function.source_object_id()
            || binding.name() == "_"
            || !seen.insert((binding.declaration_node(), binding.name_node()))
        {
            return Err(GoScalarTypingError::Name);
        }
        bindings.push(GoSourceEntryBinding {
            binding: binding.clone(),
            typ: typ.into(),
            origin: GoSourceEntryBindingOrigin::Local,
        });
    }
    Ok(GoSourceEntryLayout {
        function: function.clone(),
        signature,
        bindings,
        inputs,
        results,
    })
}

/// Every checked original body, including transitive and closed-error callees.
pub fn derive_go_source_entry_layouts(
    domain: &GoSourceValueDomain,
) -> Result<Vec<GoSourceEntryLayout>> {
    domain
        .selection()
        .checked_body_locators()
        .map(|function| derive_go_source_entry_layout(domain, function))
        .collect()
}
