//! Complete original source/sidecar handoff. Provenance identities are private
//! input linkage, not launcher authentication or installed context authority.
use super::*;

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-1.0");
const CANDIDATE_DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-CONTRACT-INPUTS-2.0");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    image_sha256: String,
    compiler_sha256: String,
    main_source_object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    requested_package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    requested_packages: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    candidate_request_sha256: Option<String>,
    source_components_sha256: String,
    overlays: Vec<GoTypeSourceObject>,
    content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceContractInputError {
    Input(GoTypeKeyError),
    Overlay(Box<GoSourceOverlayError>),
}
impl std::fmt::Display for GoSourceContractInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source contract input error: {self:?}")
    }
}
impl std::error::Error for GoSourceContractInputError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceContractDerivationError {
    Language(GoSourceLanguageError),
    Contract(GoContractTypingError),
}
impl std::fmt::Display for GoSourceContractDerivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source contract derivation error: {self:?}")
    }
}
impl std::error::Error for GoSourceContractDerivationError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceContractInputs {
    source: GoSourceInputs,
    overlays: Vec<GoSourceOverlay>,
    wire: Wire,
    requested_packages: Vec<String>,
    canonical: Vec<u8>,
}
impl GoSourceContractInputs {
    pub fn source_inputs(&self) -> &GoSourceInputs {
        &self.source
    }
    pub fn overlays(&self) -> &[GoSourceOverlay] {
        &self.overlays
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn image_sha256(&self) -> &str {
        &self.wire.image_sha256
    }
    pub fn compiler_sha256(&self) -> &str {
        &self.wire.compiler_sha256
    }
    pub fn main_source_object_id(&self) -> &str {
        &self.wire.main_source_object_id
    }
    /// The legacy single-package provenance, if this is a v1 record.
    pub fn requested_package(&self) -> Option<&str> {
        self.wire.requested_package.as_deref()
    }
    /// Every selected native package, in strictly increasing import-path order.
    pub fn requested_packages(&self) -> &[String] {
        &self.requested_packages
    }
    pub fn candidate_request_sha256(&self) -> Option<&str> {
        self.wire.candidate_request_sha256.as_deref()
    }
    pub fn content_sha256(&self) -> &str {
        &self.wire.content_sha256
    }

    /// Derive selected/transitive contracts from this exact owned annotation
    /// set. The caller supplies roots, never replacement source or sidecars.
    pub fn derive_contracts(
        &self,
        roots: &[GoSourceFunctionSelector],
    ) -> std::result::Result<GoSourceContractModule, GoSourceContractDerivationError> {
        let languages = self
            .source
            .languages()
            .map_err(GoSourceContractDerivationError::Language)?;
        let module =
            derive_go_source_contract_module(self.source.components(), roots, &self.overlays)
                .map_err(GoSourceContractDerivationError::Contract)?;
        languages
            .check_selection(module.selection())
            .map_err(GoSourceContractDerivationError::Language)?;
        Ok(module)
    }
}

pub fn import_go_source_contract_inputs(
    record: &[u8],
    components: &[u8],
    sources: &[GoTypeSourceBuffer],
    overlays: &[GoTypeSourceBuffer],
) -> std::result::Result<GoSourceContractInputs, GoSourceContractInputError> {
    let input = |e| GoSourceContractInputError::Input(e);
    let wire: Wire = decode(record).map_err(input)?;
    let (requested_packages, domain) = match (
        wire.schema.as_str(),
        &wire.requested_package,
        &wire.requested_packages,
        &wire.candidate_request_sha256,
    ) {
        ("mpk.go.practical.source_contract_inputs.v1", Some(package), None, None)
            if !package.is_empty() =>
        {
            (vec![package.clone()], DOMAIN)
        }
        ("mpk.go.practical.source_contract_inputs.v2", None, Some(packages), Some(request))
            if !packages.is_empty()
                && packages.len() <= 8192
                && packages.iter().all(|p| !p.is_empty())
                && packages.windows(2).all(|pair| pair[0] < pair[1])
                && request.len() == 64
                && request
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) =>
        {
            (packages.clone(), CANDIDATE_DOMAIN)
        }
        _ => return Err(input(GoTypeKeyError::Shape)),
    };
    if ![
        &wire.image_sha256,
        &wire.compiler_sha256,
        &wire.main_source_object_id,
        &wire.source_components_sha256,
        &wire.content_sha256,
    ]
    .into_iter()
    .all(|s| {
        s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    }) || identity(&wire, domain, "content_sha256").map_err(input)? != wire.content_sha256
    {
        return Err(input(GoTypeKeyError::Identity));
    }
    let source = import_go_source_inputs(components, sources).map_err(input)?;
    let keys = source.components();
    let package_paths = keys
        .wire
        .packages
        .iter()
        .map(|p| p.import_path.as_str())
        .collect::<BTreeSet<_>>();
    if keys.wire.content_sha256 != wire.source_components_sha256
        || requested_packages
            .iter()
            .any(|package| !package_paths.contains(package.as_str()))
    {
        return Err(input(GoTypeKeyError::Reference));
    }
    if keys.sources.len().saturating_add(overlays.len()) > 32768 {
        return Err(input(GoTypeKeyError::Limit));
    }
    if wire.overlays.len() != overlays.len() {
        return Err(input(GoTypeKeyError::Source));
    }
    let mut total = keys
        .sources
        .values()
        .try_fold(0usize, |n, s| n.checked_add(s.bytes().len()))
        .ok_or_else(|| input(GoTypeKeyError::Limit))?;
    let mut contract_bytes = 0usize;
    let mut paths = keys
        .sources
        .values()
        .map(|s| {
            (
                s.object().capability.clone(),
                s.object().path.to_ascii_lowercase(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut originals = BTreeMap::new();
    for overlay in overlays {
        if overlay.object().kind != "overlay"
            || originals
                .insert(overlay.object().id.as_str(), overlay)
                .is_some()
            || !paths.insert((
                overlay.object().capability.clone(),
                overlay.object().path.to_ascii_lowercase(),
            ))
        {
            return Err(input(GoTypeKeyError::Source));
        }
        total = total
            .checked_add(overlay.bytes().len())
            .ok_or_else(|| input(GoTypeKeyError::Limit))?;
        contract_bytes = contract_bytes
            .checked_add(overlay.bytes().len())
            .ok_or_else(|| input(GoTypeKeyError::Limit))?;
    }
    if total > 268435456 || contract_bytes > 8388608 {
        return Err(input(GoTypeKeyError::Limit));
    }
    let mut parsed = vec![];
    for (i, object) in wire.overlays.iter().enumerate() {
        if i > 0 && wire.overlays[i - 1].id >= object.id {
            return Err(input(GoTypeKeyError::Order));
        }
        let raw = originals
            .get(object.id.as_str())
            .ok_or_else(|| input(GoTypeKeyError::Source))?;
        if raw.object() != object {
            return Err(input(GoTypeKeyError::Source));
        }
        parsed.push(
            GoSourceOverlay::import(raw)
                .map_err(|e| GoSourceContractInputError::Overlay(Box::new(e)))?,
        );
    }
    let canonical = canonical_json_bytes(
        &parse_strict_json(record, limits()).map_err(|_| input(GoTypeKeyError::Json))?,
    )
    .map_err(|_| input(GoTypeKeyError::Json))?;
    Ok(GoSourceContractInputs {
        source,
        overlays: parsed,
        wire,
        requested_packages,
        canonical,
    })
}
