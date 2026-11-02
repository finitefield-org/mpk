//! Physical original roots bound to complete source/contract input bytes.
//! Independent parsing supplies node identities; native names/types never do.
use super::*;
use crate::go_practical_context::{GoPrivateCandidateRequest, GoPrivateCandidateSelection};

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0");

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoSourcePhysicalRoot {
    package_id: String,
    source_object_id: String,
    start_byte: u64,
    end_byte: u64,
}
impl GoSourcePhysicalRoot {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }
    pub fn source_object_id(&self) -> &str {
        &self.source_object_id
    }
    pub fn start_byte(&self) -> u64 {
        self.start_byte
    }
    pub fn end_byte(&self) -> u64 {
        self.end_byte
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    source_contract_inputs_sha256: String,
    roots: Vec<GoSourcePhysicalRoot>,
    content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceSelectionInputError {
    Input(GoTypeKeyError),
    Contract(Box<GoSourceContractInputError>),
    Scope(GoSourceScopeError),
    Root,
}
impl std::fmt::Display for GoSourceSelectionInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go physical source selection input error: {self:?}")
    }
}
impl std::error::Error for GoSourceSelectionInputError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSelectionInputs {
    contracts: GoSourceContractInputs,
    wire: Wire,
    resolved: Vec<GoSourceSelectionRef>,
    canonical: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoPracticalSourceBindingError {
    Profile,
    Scope(GoSourceScopeError),
    Resolve {
        root: Option<usize>,
        error: GoSourceOverlayError,
    },
    Duplicate {
        first: usize,
        second: usize,
    },
    Mismatch {
        root: Option<usize>,
    },
}
impl std::fmt::Display for GoPracticalSourceBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go candidate request/source selection binding: {self:?}")
    }
}
impl std::error::Error for GoPracticalSourceBindingError {}

/// The complete original physical-root owner and the independently validated
/// raw candidate request agree after source/receiver-alias resolution. The
/// borrowed owners remain live without cloning the full captured source set.
/// This does not authenticate the launcher, installed profile or a source theorem.
pub struct GoPracticalSourceBinding<'a> {
    request: &'a GoPrivateCandidateRequest,
    inputs: &'a GoSourceSelectionInputs,
}
impl GoPracticalSourceBinding<'_> {
    pub fn request(&self) -> &GoPrivateCandidateRequest {
        self.request
    }
    pub fn source_inputs(&self) -> &GoSourceSelectionInputs {
        self.inputs
    }
}

/// Resolve the request against original parsed source, then require exactly
/// the already imported physical roots. No caller-supplied locator adapter or
/// relabeled source is accepted. This does not derive a verification closure.
pub fn bind_go_practical_source_selection<'a>(
    request: &'a GoPrivateCandidateRequest,
    inputs: &'a GoSourceSelectionInputs,
) -> std::result::Result<GoPracticalSourceBinding<'a>, GoPracticalSourceBindingError> {
    let GoPrivateCandidateSelection::Practical(selection) = request.selection() else {
        return Err(GoPracticalSourceBindingError::Profile);
    };
    let resolved = source_overlays::resolve_go_practical_named_roots(
        inputs.contracts.source_inputs().components(),
        selection,
    )?;
    let expected = inputs.resolved.iter().collect::<BTreeSet<_>>();
    if let Some(root) = resolved
        .iter()
        .position(|reference| !expected.contains(reference))
    {
        return Err(GoPracticalSourceBindingError::Mismatch { root: Some(root) });
    }
    if resolved.len() != inputs.resolved.len()
        || resolved.iter().collect::<BTreeSet<_>>() != expected
    {
        return Err(GoPracticalSourceBindingError::Mismatch { root: None });
    }
    Ok(GoPracticalSourceBinding { request, inputs })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoPracticalModuleGraphBindingError {
    SourceSelection,
}
impl std::fmt::Display for GoPracticalModuleGraphBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go candidate request/module graph binding: {self:?}")
    }
}
impl std::error::Error for GoPracticalModuleGraphBindingError {}

/// The request, original source selection and independently imported module
/// graph stay borrowed together. This carries no held-root or launcher claim.
pub struct GoPracticalModuleGraphBinding<'a> {
    source: GoPracticalSourceBinding<'a>,
    graph: &'a GoSourceModuleGraphInputs,
}
impl GoPracticalModuleGraphBinding<'_> {
    pub fn source(&self) -> &GoPracticalSourceBinding<'_> {
        &self.source
    }
    pub fn graph(&self) -> &GoSourceModuleGraphInputs {
        self.graph
    }
}

/// Compare the complete imported selection owner, not a caller-provided digest
/// or its projection. Both owners retain their original bytes and metadata.
pub fn bind_go_practical_module_graph<'a>(
    source: GoPracticalSourceBinding<'a>,
    graph: &'a GoSourceModuleGraphInputs,
) -> std::result::Result<GoPracticalModuleGraphBinding<'a>, GoPracticalModuleGraphBindingError> {
    let graph_selection = graph
        .module_data()
        .module_observations()
        .export_inputs()
        .capture_inputs()
        .selection_inputs();
    if source.source_inputs() != graph_selection {
        return Err(GoPracticalModuleGraphBindingError::SourceSelection);
    }
    Ok(GoPracticalModuleGraphBinding { source, graph })
}
impl GoSourceSelectionInputs {
    pub fn contract_inputs(&self) -> &GoSourceContractInputs {
        &self.contracts
    }
    pub fn roots(&self) -> &[GoSourcePhysicalRoot] {
        &self.wire.roots
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn content_sha256(&self) -> &str {
        &self.wire.content_sha256
    }

    /// No replacement selector, source or annotation list is accepted here.
    pub fn derive_contracts(
        &self,
    ) -> std::result::Result<GoSourceContractModule, GoSourceContractDerivationError> {
        let languages = self
            .contracts
            .source_inputs()
            .languages()
            .map_err(GoSourceContractDerivationError::Language)?;
        let module = source_overlays::derive_go_source_contract_module_at(
            self.contracts.source_inputs().components(),
            &self.resolved,
            self.contracts.overlays(),
        )
        .map_err(GoSourceContractDerivationError::Contract)?;
        languages
            .check_selection(module.selection())
            .map_err(GoSourceContractDerivationError::Language)?;
        Ok(module)
    }
}

pub fn import_go_source_selection_inputs(
    record: &[u8],
    contracts_record: &[u8],
    components: &[u8],
    sources: &[GoTypeSourceBuffer],
    overlays: &[GoTypeSourceBuffer],
) -> std::result::Result<GoSourceSelectionInputs, GoSourceSelectionInputError> {
    let input = GoSourceSelectionInputError::Input;
    let wire: Wire = decode(record).map_err(input)?;
    if wire.schema != "mpk.go.practical.source_selection_inputs.v1" {
        return Err(input(GoTypeKeyError::Shape));
    }
    if identity(&wire, DOMAIN, "content_sha256").map_err(input)? != wire.content_sha256 {
        return Err(input(GoTypeKeyError::Identity));
    }
    if wire.roots.len() > 8192 {
        return Err(input(GoTypeKeyError::Limit));
    }
    if wire.roots.is_empty() {
        return Err(GoSourceSelectionInputError::Root);
    }
    if wire.roots.windows(2).any(|r| r[0] >= r[1]) {
        return Err(input(GoTypeKeyError::Order));
    }
    let contracts =
        import_go_source_contract_inputs(contracts_record, components, sources, overlays)
            .map_err(|e| GoSourceSelectionInputError::Contract(Box::new(e)))?;
    if wire.source_contract_inputs_sha256 != contracts.content_sha256() {
        return Err(input(GoTypeKeyError::Identity));
    }
    let keys = contracts.source_inputs().components();
    let scopes = contracts
        .source_inputs()
        .source_scopes()
        .map_err(GoSourceSelectionInputError::Scope)?;
    let mut resolved = vec![];
    let mut distinct = BTreeSet::new();
    let mut functions = BTreeMap::new();
    for root in &wire.roots {
        let package = keys
            .wire
            .packages
            .iter()
            .find(|p| p.id == root.package_id)
            .ok_or(GoSourceSelectionInputError::Root)?;
        if package
            .source_objects
            .binary_search(&root.source_object_id)
            .is_err()
        {
            return Err(GoSourceSelectionInputError::Root);
        }
        let source = scopes
            .source(&root.source_object_id)
            .ok_or(GoSourceSelectionInputError::Root)?;
        if !functions.contains_key(&root.source_object_id) {
            let mut indexed = BTreeMap::new();
            for (node, n) in source
                .nodes()
                .iter()
                .enumerate()
                .filter(|(_, n)| n.kind() == "FuncDecl")
            {
                if indexed
                    .insert((n.start() as u64, n.end() as u64), node)
                    .is_some()
                {
                    return Err(GoSourceSelectionInputError::Root);
                }
            }
            functions.insert(root.source_object_id.clone(), indexed);
        }
        let node = *functions[&root.source_object_id]
            .get(&(root.start_byte, root.end_byte))
            .ok_or(GoSourceSelectionInputError::Root)?;
        let original = &source.nodes()[node];
        let name = source_signatures::list(original, "Name")
            .first()
            .and_then(|&n| source_signatures::attr(&source.nodes()[n], "Name"));
        if !name.is_some_and(|n| {
            n != "_" && (n != "init" || !source_signatures::list(original, "Recv").is_empty())
        }) {
            return Err(GoSourceSelectionInputError::Root);
        }
        let reference = GoSourceSelectionRef::new(&root.source_object_id, node);
        if !distinct.insert(reference.clone()) {
            return Err(GoSourceSelectionInputError::Root);
        }
        resolved.push(reference);
    }
    let canonical = canonical_json_bytes(
        &parse_strict_json(record, limits()).map_err(|_| input(GoTypeKeyError::Json))?,
    )
    .map_err(|_| input(GoTypeKeyError::Json))?;
    Ok(GoSourceSelectionInputs {
        contracts,
        wire,
        resolved,
        canonical,
    })
}
