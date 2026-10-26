//! Independently reconstruct original pruned dependency summaries and the full
//! native graph stream, including pruned root expansion from original manifests.
//! This does not establish installed execution, source/program or context authority.
use super::*;
use std::cmp::Ordering;
mod expansion;
mod root_sort;
#[cfg(test)]
mod tests;
use expansion::{expand, Phase};

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0");
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceGraphModule {
    pub path: String,
    pub version: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleWire {
    path: String,
    version_hex: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SummaryWire {
    module: ModuleWire,
    replacement_path_hex: String,
    replacement_version: String,
    manifest_source_object_id: Option<String>,
    go_version: String,
    pruned: bool,
    requirements: Vec<ModuleWire>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheWire {
    capability: String,
    directory: String,
    original_path: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphWire {
    schema: String,
    source_module_observations_sha256: String,
    module_cache: CacheWire,
    metadata_hex: String,
    diagnostics_hex: String,
    initial_summaries: Vec<SummaryWire>,
    final_summaries: Vec<SummaryWire>,
    content_sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceModuleGraphError {
    Input(GoTypeKeyError),
    Linkage,
    Cache,
    Identity,
    Manifest,
    Replacement,
    Checksum,
    Inventory,
    Metadata,
    Selection,
}
impl std::fmt::Display for GoSourceModuleGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original module graph: {self:?}")
    }
}
impl std::error::Error for GoSourceModuleGraphError {}
use GoSourceModuleGraphError as G;
type GraphResult<T> = std::result::Result<T, G>;
fn ensure(ok: bool, error: G) -> GraphResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn bytes(raw: &str) -> GraphResult<Vec<u8>> {
    super::unhex(raw).map_err(|_| G::Identity)
}
fn text(raw: &[u8]) -> GraphResult<&str> {
    std::str::from_utf8(raw).map_err(|_| G::Identity)
}
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
impl GoSourceGraphModule {
    fn wire(&self) -> ModuleWire {
        ModuleWire {
            path: self.path.clone(),
            version_hex: hex(&self.version),
        }
    }
    fn virtual_node(&self) -> bool {
        matches!(self.path.as_str(), "go" | "toolchain")
    }
    fn valid(&self) -> bool {
        if self.virtual_node() {
            manifest::graph_virtual_version_valid(&self.path, &self.version)
        } else {
            std::str::from_utf8(&self.version)
                .is_ok_and(|v| data::graph_module_identity(&self.path, v))
        }
    }
    fn precedence(&self, other: &Self) -> Ordering {
        manifest::graph_version_compare(&self.path, &self.version, &other.version)
    }
    fn inventory_order(&self, other: &Self) -> Ordering {
        self.path
            .cmp(&other.path)
            .then_with(|| self.precedence(other))
            .then_with(|| self.version.cmp(&other.version))
    }
    fn append_name(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.path.as_bytes());
        if !self.version.is_empty() {
            out.push(b'@');
            out.extend_from_slice(&self.version)
        }
    }
}
#[derive(Clone)]
struct Summary {
    wire: SummaryWire,
    requirements: Vec<GoSourceGraphModule>,
    manifest: Option<GoSourceGraphManifest>,
}
fn excluded(main: &GoSourceModuleManifest, m: &GoSourceGraphModule) -> bool {
    main.exclusions()
        .iter()
        .any(|r| r.path == m.path.as_bytes() && r.version.as_bytes() == m.version)
}
fn original_roots(main: &GoSourceModuleManifest) -> GraphResult<Vec<GoSourceGraphModule>> {
    let mut roots = main
        .requirements()
        .iter()
        .map(|r| {
            Ok(GoSourceGraphModule {
                path: text(&r.path)?.into(),
                version: r.version.as_bytes().to_vec(),
            })
        })
        .collect::<GraphResult<Vec<_>>>()?;
    roots.retain(|m| !excluded(main, m));
    roots.push(GoSourceGraphModule {
        path: "go".into(),
        version: main
            .go_requirement()
            .ok_or(G::Manifest)?
            .as_bytes()
            .to_vec(),
    });
    if let Some(v) = main.toolchain_suggestion() {
        roots.push(GoSourceGraphModule {
            path: "toolchain".into(),
            version: v.to_vec(),
        })
    }
    root_sort::sort(&mut roots);
    Ok(expansion::bootstrap_roots(
        text(main.module_path())?,
        &roots,
    ))
}
fn summary_wire(m: &GoSourceGraphModule) -> SummaryWire {
    SummaryWire {
        module: m.wire(),
        replacement_path_hex: String::new(),
        replacement_version: String::new(),
        manifest_source_object_id: None,
        go_version: String::new(),
        pruned: false,
        requirements: vec![],
    }
}
fn virtual_summary(m: &GoSourceGraphModule) -> Summary {
    let mut wire = summary_wire(m);
    let mut requirements = vec![];
    if m.path == "go" && !manifest::graph_version_compare("go", &m.version, b"1.21").is_lt() {
        let mut version = b"go".to_vec();
        version.extend_from_slice(&m.version);
        requirements.push(GoSourceGraphModule {
            path: "toolchain".into(),
            version,
        });
    }
    wire.requirements = requirements.iter().map(GoSourceGraphModule::wire).collect();
    Summary {
        wire,
        requirements,
        manifest: None,
    }
}
fn source_summary(
    main: &GoSourceModuleManifest,
    m: &GoSourceGraphModule,
    mut wire: SummaryWire,
    source: GoTypeSourceBuffer,
    path: &[u8],
    version: &[u8],
) -> GraphResult<Summary> {
    let mut requirements = vec![];
    let parsed = derive_go_source_graph_manifest(source).map_err(|_| G::Manifest)?;
    if !version.is_empty() {
        ensure(
            parsed
                .module_path()
                .is_some_and(|p| p == m.path.as_bytes() || p == path),
            G::Manifest,
        )?
    }
    ensure(!parsed.requires_newer_compiler(), G::Manifest)?;
    wire.manifest_source_object_id = Some(parsed.original_source().object().id.clone());
    wire.go_version = parsed.go_requirement().unwrap_or("").into();
    wire.pruned = parsed.supports_graph_pruning();
    for r in parsed.requirements() {
        let required = GoSourceGraphModule {
            path: text(&r.path)?.into(),
            version: r.version.as_bytes().to_vec(),
        };
        if !excluded(main, &required) {
            requirements.push(required)
        }
    }
    if let Some(go) = parsed.implicit_go_requirement() {
        requirements.push(GoSourceGraphModule {
            path: "go".into(),
            version: go.as_bytes().to_vec(),
        })
    }
    wire.requirements = requirements.iter().map(GoSourceGraphModule::wire).collect();
    Ok(Summary {
        wire,
        requirements,
        manifest: Some(parsed),
    })
}
struct View<'a> {
    data: &'a GoSourceModuleData,
    main: GoSourceModuleManifest,
    main_directory: String,
    origins: BTreeMap<String, String>,
    cache: &'a CacheWire,
}
impl View<'_> {
    fn snapshot(&self) -> &GoSourceSnapshotInputs {
        self.data
            .module_observations()
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs()
    }
    fn replacement(&self, m: &GoSourceGraphModule) -> GraphResult<Option<&GoModuleReplacement>> {
        let (mut exact, mut wildcard) = (None, None);
        for r in self.main.replacements() {
            if r.old_path != m.path.as_bytes() {
                continue;
            }
            if r.old_version.as_bytes() == m.version {
                ensure(exact.replace(r).is_none(), G::Replacement)?
            } else if r.old_version.is_empty() {
                ensure(wildcard.replace(r).is_none(), G::Replacement)?
            }
        }
        Ok(exact.or(wildcard))
    }
    fn local_source(&self, target: &[u8]) -> GraphResult<GoTypeSourceBuffer> {
        let target = text(target)?;
        let target = super::clean_absolute(&if target.starts_with('/') {
            target.into()
        } else {
            super::join(&self.main_directory, target)
        })
        .map_err(|_| G::Replacement)?;
        let mut roots = self
            .origins
            .iter()
            .filter(|(_, root)| {
                target == **root || target.starts_with(&format!("{}/", root.trim_end_matches('/')))
            })
            .collect::<Vec<_>>();
        roots.sort_by_key(|(_, root)| root.len());
        let (cap, root) = roots.last().ok_or(G::Replacement)?;
        ensure(
            roots.iter().filter(|(_, r)| r.len() == root.len()).count() == 1,
            G::Replacement,
        )?;
        let relative = target
            .strip_prefix(root.as_str())
            .ok_or(G::Replacement)?
            .trim_start_matches('/');
        let path = if relative.is_empty() {
            "go.mod".into()
        } else {
            format!("{relative}/go.mod")
        };
        self.snapshot()
            .read_source(cap, &path, "go_mod")
            .cloned()
            .map_err(|_| G::Manifest)
    }
    fn summary(&self, m: &GoSourceGraphModule) -> GraphResult<Summary> {
        if m.virtual_node() {
            return Ok(virtual_summary(m));
        }
        let mut wire = summary_wire(m);
        ensure(m.valid(), G::Identity)?;
        let replacement = self.replacement(m)?;
        let (path, version) = replacement.map_or((m.path.as_bytes(), m.version.as_slice()), |r| {
            (r.new_path.as_slice(), r.new_version.as_bytes())
        });
        if replacement.is_some() {
            wire.replacement_path_hex = hex(path);
            wire.replacement_version = text(version)?.into()
        }
        let source = if version.is_empty() {
            ensure(replacement.is_some(), G::Replacement)?;
            self.local_source(path)?
        } else {
            let path = text(path)?;
            let version = text(version)?;
            ensure(data::graph_module_identity(path, version), G::Identity)?;
            let suffix = format!(
                "cache/download/{}/@v/{}.mod",
                super::escape(path),
                super::escape(version)
            );
            let logical = if self.cache.directory.is_empty() {
                suffix
            } else {
                format!("{}/{suffix}", self.cache.directory)
            };
            let source = self
                .snapshot()
                .read_source(&self.cache.capability, &logical, "go_mod")
                .map_err(|_| G::Manifest)?
                .clone();
            let sum = self
                .data
                .sums()
                .and_then(|s| s.checksum(path, &format!("{version}/go.mod")))
                .ok_or(G::Checksum)?;
            ensure(data::manifest_sum(&source) == sum, G::Checksum)?;
            source
        };
        source_summary(&self.main, m, wire, source, path, version)
    }
    fn phase(&self, roots: &[GoSourceGraphModule]) -> GraphResult<Phase> {
        expansion::phase(text(self.main.module_path())?, roots, |m| self.summary(m))
    }
}

fn ordered_rows(phase: &BTreeMap<GoSourceGraphModule, Summary>) -> Vec<SummaryWire> {
    let mut modules = phase.keys().collect::<Vec<_>>();
    modules.sort_by(|a, b| a.inventory_order(b));
    modules.into_iter().map(|m| phase[m].wire.clone()).collect()
}
fn graph_name(raw: &[u8], main: &str) -> GraphResult<GoSourceGraphModule> {
    if raw == main.as_bytes() {
        return Ok(GoSourceGraphModule {
            path: main.into(),
            version: vec![],
        });
    }
    let at = raw.iter().position(|b| *b == b'@').ok_or(G::Metadata)?;
    let m = GoSourceGraphModule {
        path: text(&raw[..at])?.into(),
        version: raw[at + 1..].to_vec(),
    };
    ensure(m.valid(), G::Identity)?;
    Ok(m)
}
fn root_edges(raw: &[u8], main: &str) -> GraphResult<Vec<GoSourceGraphModule>> {
    ensure(
        !raw.is_empty() && raw.len() <= 16_777_216 && raw.last() == Some(&b'\n'),
        G::Metadata,
    )?;
    let mut roots = vec![];
    for line in raw[..raw.len() - 1].split(|b| *b == b'\n') {
        let at = line.iter().position(|b| *b == b' ').ok_or(G::Metadata)?;
        ensure(
            !line[..at]
                .iter()
                .chain(line[at + 1..].iter())
                .any(|b| matches!(b, b' ' | b'\t' | b'\r')),
            G::Metadata,
        )?;
        let parent = graph_name(&line[..at], main)?;
        let child = graph_name(&line[at + 1..], main)?;
        if parent.path == main && parent.version.is_empty() {
            roots.push(child)
        }
    }
    ensure(!roots.is_empty(), G::Metadata)?;
    Ok(roots)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModuleGraphInputs {
    data: GoSourceModuleData,
    canonical: Vec<u8>,
    metadata: Vec<u8>,
    diagnostics: Vec<u8>,
    initial: BTreeMap<GoSourceGraphModule, GoSourceGraphManifest>,
    final_manifests: BTreeMap<GoSourceGraphModule, GoSourceGraphManifest>,
    selected: BTreeMap<String, Vec<u8>>,
    roots: Vec<GoSourceGraphModule>,
    expansion_manifests: BTreeMap<GoSourceGraphModule, GoSourceGraphManifest>,
}
impl GoSourceModuleGraphInputs {
    pub fn module_data(&self) -> &GoSourceModuleData {
        &self.data
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn original_metadata(&self) -> &[u8] {
        &self.metadata
    }
    pub fn original_diagnostics(&self) -> &[u8] {
        &self.diagnostics
    }
    pub fn initial_manifests(&self) -> &BTreeMap<GoSourceGraphModule, GoSourceGraphManifest> {
        &self.initial
    }
    pub fn final_manifests(&self) -> &BTreeMap<GoSourceGraphModule, GoSourceGraphManifest> {
        &self.final_manifests
    }
    pub fn observed_selected_versions(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.selected
    }
    pub fn derived_roots(&self) -> &[GoSourceGraphModule] {
        &self.roots
    }
    /// Every original real manifest consumed in any expansion phase, including
    /// versions absent from both the initial and final loaded graph.
    pub fn expansion_manifests(&self) -> &BTreeMap<GoSourceGraphModule, GoSourceGraphManifest> {
        &self.expansion_manifests
    }
}
/// Reconstruct all original initial/final loaded summaries and every native
/// output byte, deriving pruned roots from the original main requirements.
/// Installed source/compiler/context authority remains a separate obligation.
pub fn import_go_source_module_graph_inputs(
    record: &[u8],
    data: GoSourceModuleData,
) -> GraphResult<GoSourceModuleGraphInputs> {
    let wire: GraphWire = decode(record).map_err(G::Input)?;
    ensure(
        wire.schema == "mpk.go.practical.source_module_graph_inputs.v2",
        G::Input(GoTypeKeyError::Shape),
    )?;
    ensure(
        identity(&wire, DOMAIN, "content_sha256").map_err(G::Input)? == wire.content_sha256,
        G::Input(GoTypeKeyError::Identity),
    )?;
    let observations = data.module_observations();
    let observed: Wire = decode(observations.canonical_bytes()).map_err(G::Input)?;
    ensure(
        observed.content_sha256 == wire.source_module_observations_sha256,
        G::Linkage,
    )?;
    let exports: serde_json::Value =
        serde_json::from_slice(observations.export_inputs().canonical_bytes())
            .map_err(|_| G::Linkage)?;
    let origins = exports["origins"]
        .as_array()
        .ok_or(G::Cache)?
        .iter()
        .map(|r| {
            Ok((
                r["capability"].as_str().ok_or(G::Cache)?.to_string(),
                r["path"].as_str().ok_or(G::Cache)?.to_string(),
            ))
        })
        .collect::<GraphResult<BTreeMap<_, _>>>()?;
    let snapshot = observations
        .export_inputs()
        .capture_inputs()
        .snapshot_inputs();
    snapshot
        .read_directory(&wire.module_cache.capability, &wire.module_cache.directory)
        .map_err(|_| G::Cache)?;
    let origin = origins.get(&wire.module_cache.capability).ok_or(G::Cache)?;
    ensure(
        super::join(origin, &wire.module_cache.directory) == wire.module_cache.original_path,
        G::Cache,
    )?;
    ensure(
        super::clean_absolute(&wire.module_cache.original_path).map_err(|_| G::Cache)?
            == wire.module_cache.original_path,
        G::Cache,
    )?;
    let main_id = observations
        .export_inputs()
        .capture_inputs()
        .selection_inputs()
        .contract_inputs()
        .main_source_object_id();
    let main_source = snapshot.source(main_id).ok_or(G::Manifest)?;
    let main =
        derive_go_source_module_manifest(main_source.clone(), true).map_err(|_| G::Manifest)?;
    let main_path = text(main.module_path())?.to_string();
    let main_directory = super::parent(&super::join(
        origins
            .get(&main_source.object().capability)
            .ok_or(G::Cache)?,
        &main_source.object().path,
    ))
    .to_string();
    let view = View {
        data: &data,
        main,
        main_directory,
        origins,
        cache: &wire.module_cache,
    };
    let original_roots = original_roots(&view.main)?;
    let initial = view.phase(&original_roots)?;
    ensure(
        ordered_rows(&initial.summaries) == wire.initial_summaries,
        G::Inventory,
    )?;
    let metadata = bytes(&wire.metadata_hex)?;
    let diagnostics = bytes(&wire.diagnostics_hex)?;
    let expansion = expand(&main_path, original_roots, initial, |roots| {
        view.phase(roots)
    })?;
    let roots = expansion.roots;
    ensure(root_edges(&metadata, &main_path)? == roots, G::Selection)?;
    let selected = expansion
        .final_phase
        .selected
        .into_iter()
        .filter(|(path, _)| !matches!(path.as_str(), "go" | "toolchain"))
        .collect::<BTreeMap<_, _>>();
    let final_graph = expansion.final_phase.summaries;
    ensure(
        ordered_rows(&final_graph) == wire.final_summaries,
        G::Inventory,
    )?;
    let main = GoSourceGraphModule {
        path: main_path.clone(),
        version: vec![],
    };
    let mut queue = vec![main.clone()];
    let mut seen = BTreeSet::from([main.clone()]);
    let mut expected = vec![];
    let mut at = 0;
    while at < queue.len() {
        let parent = &queue[at];
        let required = if parent == &main {
            roots.as_slice()
        } else {
            final_graph
                .get(parent)
                .map_or(&[][..], |s| s.requirements.as_slice())
        };
        for child in required {
            parent.append_name(&mut expected);
            expected.push(b' ');
            child.append_name(&mut expected);
            expected.push(b'\n');
            ensure(expected.len() <= 16_777_216, G::Metadata)?;
        }
        // Keep traversal and graph requirements separate from selected maxima.
        let required = required.to_vec();
        at += 1;
        for child in required {
            if seen.insert(child.clone()) {
                queue.push(child)
            }
        }
    }
    ensure(expected == metadata, G::Metadata)?;
    ensure(
        selected.len() == observed.modules.len()
            && observed
                .modules
                .iter()
                .all(|m| selected.get(&m.path) == Some(&m.version.as_bytes().to_vec())),
        G::Selection,
    )?;
    let initial = expansion
        .initial
        .into_iter()
        .filter_map(|(m, s)| s.manifest.map(|f| (m, f)))
        .collect();
    let final_manifests = final_graph
        .into_iter()
        .filter_map(|(m, s)| s.manifest.map(|f| (m, f)))
        .collect();
    Ok(GoSourceModuleGraphInputs {
        data,
        canonical: record.to_vec(),
        metadata,
        diagnostics,
        initial,
        final_manifests,
        selected,
        roots,
        expansion_manifests: expansion.manifests,
    })
}
