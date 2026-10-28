//! Independently checked original captured bytes and observed native names.
//! This private handoff does not authorize filesystem roots or authenticate a
//! launcher/context. It distinguishes unread state from observed absence.
use super::*;
use sha2::{Digest, Sha256};

const SNAPSHOT: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0");
const CAPTURE: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-CAPTURE-INPUTS-1.0");
const DIR: u32 = 1 << 31;
const TYPE_BITS: u32 = DIR | 1 << 27 | 1 << 26 | 1 << 25 | 1 << 24 | 1 << 21 | 1 << 19;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryWire {
    name_hex: String,
    type_bits: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryWire {
    capability: String,
    path: String,
    entries: Vec<EntryWire>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    capabilities: Vec<String>,
    directories: Vec<DirectoryWire>,
    objects: Vec<GoTypeSourceObject>,
    snapshot_sha256: String,
    content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceSnapshotError {
    Input(GoTypeKeyError),
    Build(GoSourceBuildError),
    Package(GoSourcePackageError),
    Namespace,
}
impl std::fmt::Display for GoSourceSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source snapshot input error: {self:?}")
    }
}
impl std::error::Error for GoSourceSnapshotError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceSnapshotLookupError {
    Path,
    NotFound,
    NotCaptured,
    Spelling,
    Type,
    Role,
}
impl std::fmt::Display for GoSourceSnapshotLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source snapshot lookup: {self:?}")
    }
}
impl std::error::Error for GoSourceSnapshotLookupError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSnapshotEntry {
    name: Vec<u8>,
    type_bits: u32,
}
impl GoSourceSnapshotEntry {
    pub fn name_bytes(&self) -> &[u8] {
        &self.name
    }
    pub fn type_bits(&self) -> u32 {
        self.type_bits
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceSnapshotInputs {
    wire: Wire,
    directories: BTreeMap<(String, String), Vec<GoSourceSnapshotEntry>>,
    sources: BTreeMap<String, GoTypeSourceBuffer>,
    paths: BTreeMap<(String, String), String>,
    canonical: Vec<u8>,
}
type Lookup<T> = std::result::Result<T, GoSourceSnapshotLookupError>;
impl GoSourceSnapshotInputs {
    /// The exact directories recorded by the untrusted capture, including
    /// native entry-name bytes. This view does not authenticate their roots.
    pub fn observed_directories(
        &self,
    ) -> impl Iterator<Item = (&str, &str, &[GoSourceSnapshotEntry])> {
        self.directories
            .iter()
            .map(|((capability, path), entries)| {
                (capability.as_str(), path.as_str(), entries.as_slice())
            })
    }
    pub(super) fn directory_entry_count(&self) -> usize {
        self.directories.values().map(Vec::len).sum()
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn content_sha256(&self) -> &str {
        &self.wire.content_sha256
    }
    pub fn snapshot_sha256(&self) -> &str {
        &self.wire.snapshot_sha256
    }
    pub fn capabilities(&self) -> &[String] {
        &self.wire.capabilities
    }
    pub fn source_objects(&self) -> &[GoTypeSourceObject] {
        &self.wire.objects
    }
    pub fn source(&self, id: &str) -> Option<&GoTypeSourceBuffer> {
        self.sources.get(id)
    }

    /// Compare every directory, source identity and original byte buffer in
    /// `observed` with the same capabilities in this complete snapshot. Any
    /// other capabilities in `self` remain present and are not authenticated by
    /// this comparison. This does not authorize roots or a semantic context.
    pub fn contains_exact_capabilities(&self, observed: &Self) -> bool {
        let capabilities = observed
            .capabilities()
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        capabilities.iter().all(|capability| {
            self.wire
                .capabilities
                .binary_search_by(|c| c.as_str().cmp(capability))
                .is_ok()
        }) && self
            .directories
            .iter()
            .filter(|((capability, _), _)| capabilities.contains(capability.as_str()))
            .eq(observed.directories.iter())
            && self
                .wire
                .objects
                .iter()
                .filter(|object| capabilities.contains(object.capability.as_str()))
                .eq(observed.wire.objects.iter())
            && observed
                .sources
                .iter()
                .all(|(id, source)| self.sources.get(id) == Some(source))
    }
    fn check_path(&self, capability: &str, path: &str, empty: bool) -> Lookup<()> {
        if !validate::logical_path(capability)
            || self
                .wire
                .capabilities
                .binary_search_by(|c| c.as_str().cmp(capability))
                .is_err()
            || !(empty && path.is_empty() || validate::logical_path(path))
        {
            return Err(GoSourceSnapshotLookupError::Path);
        }
        Ok(())
    }
    fn entry(&self, capability: &str, path: &str) -> Lookup<&GoSourceSnapshotEntry> {
        self.check_path(capability, path, false)?;
        let parts = path.split('/').collect::<Vec<_>>();
        let mut parent = String::new();
        for (index, name) in parts.iter().enumerate() {
            let entries = self
                .directories
                .get(&(capability.into(), parent.clone()))
                .ok_or(GoSourceSnapshotLookupError::NotCaptured)?;
            let at = match entries.binary_search_by(|e| e.name.as_slice().cmp(name.as_bytes())) {
                Ok(at) => at,
                Err(_) => {
                    return Err(
                        if entries
                            .iter()
                            .any(|e| e.name.eq_ignore_ascii_case(name.as_bytes()))
                        {
                            GoSourceSnapshotLookupError::Spelling
                        } else {
                            GoSourceSnapshotLookupError::NotFound
                        },
                    )
                }
            };
            let entry = &entries[at];
            if index + 1 == parts.len() {
                return Ok(entry);
            }
            if entry.type_bits != DIR {
                return Err(GoSourceSnapshotLookupError::Type);
            }
            if !parent.is_empty() {
                parent.push('/')
            }
            parent.push_str(name);
        }
        Err(GoSourceSnapshotLookupError::Path)
    }
    pub fn read_directory(&self, capability: &str, path: &str) -> Lookup<&[GoSourceSnapshotEntry]> {
        self.check_path(capability, path, true)?;
        if !path.is_empty() && self.entry(capability, path)?.type_bits != DIR {
            return Err(GoSourceSnapshotLookupError::Type);
        }
        self.directories
            .get(&(capability.into(), path.into()))
            .map(Vec::as_slice)
            .ok_or(GoSourceSnapshotLookupError::NotCaptured)
    }
    pub fn read_source(
        &self,
        capability: &str,
        path: &str,
        kind: &str,
    ) -> Lookup<&GoTypeSourceBuffer> {
        if !matches!(
            kind,
            "go_file"
                | "go_mod"
                | "go_sum"
                | "overlay"
                | "metadata"
                | "export_data"
                | "excluded_source"
                | "auxiliary"
        ) {
            return Err(GoSourceSnapshotLookupError::Role);
        }
        if self.entry(capability, path)?.type_bits != 0 {
            return Err(GoSourceSnapshotLookupError::Type);
        }
        let id = self
            .paths
            .get(&(capability.into(), path.into()))
            .ok_or(GoSourceSnapshotLookupError::NotCaptured)?;
        let source = &self.sources[id];
        if source.object().kind != kind {
            return Err(GoSourceSnapshotLookupError::Role);
        }
        Ok(source)
    }
}

fn unhex(s: &str) -> std::result::Result<Vec<u8>, GoSourceSnapshotError> {
    if !s.len().is_multiple_of(2)
        || !s
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(GoSourceSnapshotError::Namespace);
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| {
            u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16)
                .map_err(|_| GoSourceSnapshotError::Namespace)
        })
        .collect()
}
fn native_digest(w: &Wire) -> std::result::Result<String, GoSourceSnapshotError> {
    fn number(h: &mut Sha256, n: usize) {
        h.update((n as u64).to_le_bytes())
    }
    fn text(h: &mut Sha256, s: &[u8]) {
        number(h, s.len());
        h.update(s)
    }
    let mut h = Sha256::new();
    h.update(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0");
    number(&mut h, w.capabilities.len());
    for c in &w.capabilities {
        text(&mut h, c.as_bytes())
    }
    number(&mut h, w.directories.len());
    for d in &w.directories {
        text(&mut h, d.capability.as_bytes());
        text(&mut h, d.path.as_bytes());
        number(&mut h, d.entries.len());
        for e in &d.entries {
            text(&mut h, &unhex(&e.name_hex)?);
            h.update((e.type_bits as u64).to_le_bytes())
        }
    }
    number(&mut h, w.objects.len());
    for o in &w.objects {
        text(&mut h, o.id.as_bytes())
    }
    Ok(format!("{:x}", h.finalize()))
}

pub fn import_go_source_snapshot_inputs(
    record: &[u8],
    sources: &[GoTypeSourceBuffer],
) -> std::result::Result<GoSourceSnapshotInputs, GoSourceSnapshotError> {
    let input = GoSourceSnapshotError::Input;
    let w: Wire = decode(record).map_err(input)?;
    if w.schema != "mpk.go.practical.source_snapshot_inputs.v1" {
        return Err(input(GoTypeKeyError::Shape));
    }
    if identity(&w, SNAPSHOT, "content_sha256").map_err(input)? != w.content_sha256
        || native_digest(&w)? != w.snapshot_sha256
    {
        return Err(input(GoTypeKeyError::Identity));
    }
    if w.capabilities.len() > 32768
        || w.directories.len() > 32768
        || w.objects.len() > 32768
        || w.directories
            .iter()
            .try_fold(0usize, |n, d| n.checked_add(d.entries.len()))
            .is_none_or(|n| n > 131072)
    {
        return Err(input(GoTypeKeyError::Limit));
    }
    if w.capabilities.is_empty()
        || w.capabilities.iter().any(|c| !validate::logical_path(c))
        || w.capabilities.windows(2).any(|c| c[0] >= c[1])
        || w.capabilities
            .iter()
            .map(|c| c.to_ascii_lowercase())
            .collect::<BTreeSet<_>>()
            .len()
            != w.capabilities.len()
    {
        return Err(GoSourceSnapshotError::Namespace);
    }
    if w.directories
        .windows(2)
        .any(|d| (&d[0].capability, &d[0].path) >= (&d[1].capability, &d[1].path))
        || w.objects.windows(2).any(|o| o[0].id >= o[1].id)
    {
        return Err(input(GoTypeKeyError::Order));
    }
    let mut directories = BTreeMap::new();
    for d in &w.directories {
        if w.capabilities.binary_search(&d.capability).is_err()
            || !(d.path.is_empty() || validate::logical_path(&d.path))
        {
            return Err(GoSourceSnapshotError::Namespace);
        }
        let mut entries = vec![];
        for e in &d.entries {
            let name = unhex(&e.name_hex)?;
            if name.is_empty()
                || matches!(name.as_slice(), b"." | b"..")
                || name.iter().any(|c| matches!(*c, b'/' | 0))
                || e.type_bits & !TYPE_BITS != 0
                || entries
                    .last()
                    .is_some_and(|last: &GoSourceSnapshotEntry| last.name >= name)
            {
                return Err(GoSourceSnapshotError::Namespace);
            }
            entries.push(GoSourceSnapshotEntry {
                name,
                type_bits: e.type_bits,
            });
        }
        directories.insert((d.capability.clone(), d.path.clone()), entries);
    }
    let canonical = canonical_json_bytes(
        &parse_strict_json(record, limits()).map_err(|_| input(GoTypeKeyError::Json))?,
    )
    .map_err(|_| input(GoTypeKeyError::Json))?;
    let mut result = GoSourceSnapshotInputs {
        wire: w,
        directories,
        sources: BTreeMap::new(),
        paths: BTreeMap::new(),
        canonical,
    };
    for (capability, path) in result.directories.keys() {
        if !path.is_empty()
            && result
                .entry(capability, path)
                .map_err(|_| GoSourceSnapshotError::Namespace)?
                .type_bits
                != DIR
        {
            return Err(GoSourceSnapshotError::Namespace);
        }
    }
    let mut supplied = BTreeMap::new();
    for s in sources {
        if supplied.insert(s.object().id.as_str(), s).is_some() {
            return Err(input(GoTypeKeyError::Source));
        }
    }
    if supplied.len() != result.wire.objects.len() {
        return Err(input(GoTypeKeyError::Source));
    }
    let mut total = 0u64;
    let mut overlays = 0u64;
    let mut folded = BTreeSet::new();
    for object in &result.wire.objects {
        let source = supplied
            .get(object.id.as_str())
            .ok_or_else(|| input(GoTypeKeyError::Source))?;
        if source.object() != object
            || !folded.insert((object.capability.clone(), object.path.to_ascii_lowercase()))
            || result
                .entry(&object.capability, &object.path)
                .map_err(|_| GoSourceSnapshotError::Namespace)?
                .type_bits
                != 0
        {
            return Err(input(GoTypeKeyError::Source));
        }
        total = total
            .checked_add(object.bytes)
            .ok_or_else(|| input(GoTypeKeyError::Limit))?;
        if object.kind == "overlay" {
            overlays = overlays
                .checked_add(object.bytes)
                .ok_or_else(|| input(GoTypeKeyError::Limit))?
        }
        result.paths.insert(
            (object.capability.clone(), object.path.clone()),
            object.id.clone(),
        );
        result.sources.insert(object.id.clone(), (*source).clone());
    }
    if total > 268435456 || overlays > 8388608 {
        return Err(input(GoTypeKeyError::Limit));
    }
    Ok(result)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureWire {
    schema: String,
    selection_inputs_sha256: String,
    snapshot_inputs_sha256: String,
    content_sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceCaptureInputs {
    selection: GoSourceSelectionInputs,
    snapshot: GoSourceSnapshotInputs,
    canonical: Vec<u8>,
}
impl GoSourceCaptureInputs {
    pub fn selection_inputs(&self) -> &GoSourceSelectionInputs {
        &self.selection
    }
    pub fn snapshot_inputs(&self) -> &GoSourceSnapshotInputs {
        &self.snapshot
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn derive_contracts(
        &self,
    ) -> std::result::Result<GoSourceContractModule, GoSourceContractDerivationError> {
        self.selection.derive_contracts()
    }
}
pub fn import_go_source_capture_inputs(
    record: &[u8],
    selection: GoSourceSelectionInputs,
    snapshot: GoSourceSnapshotInputs,
) -> std::result::Result<GoSourceCaptureInputs, GoSourceSnapshotError> {
    let input = GoSourceSnapshotError::Input;
    let wire: CaptureWire = decode(record).map_err(input)?;
    if wire.schema != "mpk.go.practical.source_capture_inputs.v1" {
        return Err(input(GoTypeKeyError::Shape));
    }
    if identity(&wire, CAPTURE, "content_sha256").map_err(input)? != wire.content_sha256
        || wire.selection_inputs_sha256 != selection.content_sha256()
        || wire.snapshot_inputs_sha256 != snapshot.content_sha256()
    {
        return Err(input(GoTypeKeyError::Identity));
    }
    let contracts = selection.contract_inputs();
    let keys = contracts.source_inputs().components();
    for source in keys
        .sources
        .values()
        .filter(|s| s.object().kind == "go_file")
    {
        source_build::require_production_source(source).map_err(GoSourceSnapshotError::Build)?;
    }
    source_package::check_capture_package_files(&selection, &snapshot)
        .map_err(GoSourceSnapshotError::Package)?;
    for source in keys
        .sources
        .values()
        .chain(contracts.overlays().iter().map(|o| o.source_buffer()))
    {
        if snapshot.source(&source.object().id) != Some(source) {
            return Err(input(GoTypeKeyError::Source));
        }
    }
    if snapshot
        .source(contracts.main_source_object_id())
        .is_none_or(|s| s.object().kind != "go_mod")
    {
        return Err(input(GoTypeKeyError::Reference));
    }
    let all_overlays = snapshot
        .source_objects()
        .iter()
        .filter(|s| s.kind == "overlay")
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    if all_overlays
        != contracts
            .overlays()
            .iter()
            .map(|o| o.source_buffer().object().id.as_str())
            .collect::<Vec<_>>()
    {
        return Err(input(GoTypeKeyError::Source));
    }
    let canonical = canonical_json_bytes(
        &parse_strict_json(record, limits()).map_err(|_| input(GoTypeKeyError::Json))?,
    )
    .map_err(|_| input(GoTypeKeyError::Json))?;
    Ok(GoSourceCaptureInputs {
        selection,
        snapshot,
        canonical,
    })
}
