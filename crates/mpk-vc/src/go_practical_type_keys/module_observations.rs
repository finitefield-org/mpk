//! Original resolver/source/export correspondence. Version selection, ZIP/sum
//! verification, root authorization and installed compiler/context authority
//! remain separate obligations; this reader does not confer those authorities.
use super::*;

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0");
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Module {
    path: String,
    version: String,
    replacement_path: String,
    replacement_version: String,
    manifest_source_object_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    source_export_inputs_sha256: String,
    modules: Vec<Module>,
    metadata_hex: String,
    diagnostics_hex: String,
    content_sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceModuleObservationError {
    Input(GoTypeKeyError),
    Metadata,
    Membership,
    Manifest,
    Replacement,
    Package,
}
impl std::fmt::Display for GoSourceModuleObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original module observations: {self:?}")
    }
}
impl std::error::Error for GoSourceModuleObservationError {}
use GoSourceModuleObservationError as E;
type ModuleResult<T> = std::result::Result<T, E>;
fn need(ok: bool, error: E) -> ModuleResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn unhex(s: &str) -> ModuleResult<Vec<u8>> {
    need(
        s.len().is_multiple_of(2)
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        E::Metadata,
    )?;
    Ok(s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect())
}
// Strict native list -m projection, matching the producer's closed Go1.25
// decoder. The whole original stream, including every projected field, is kept.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct Native {
    path: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    dir: String,
    #[serde(default)]
    go_mod: String,
    #[serde(default)]
    go_version: String,
    #[serde(default)]
    sum: String,
    #[serde(default)]
    go_mod_sum: String,
    #[serde(default)]
    main: bool,
    #[serde(default)]
    indirect: bool,
    #[serde(default)]
    time: Option<String>,
    #[serde(default)]
    replace: Option<Box<Native>>,
    #[serde(default)]
    query: String,
    #[serde(default)]
    deprecated: String,
    #[serde(default)]
    versions: Vec<String>,
    #[serde(default)]
    retracted: Vec<String>,
    #[serde(default)]
    update: Option<Box<Native>>,
    #[serde(default)]
    origin: Option<serde_json::Value>,
    #[serde(default)]
    reuse: bool,
    #[serde(default)]
    error: Option<serde_json::Value>,
}
fn native_rows(raw: &[u8]) -> ModuleResult<Vec<Native>> {
    let mut stream = serde_json::Deserializer::from_slice(raw).into_iter::<serde_json::Value>();
    let mut start = 0;
    let mut rows = vec![];
    while let Some(value) = stream.next() {
        let value = value.map_err(|_| E::Metadata)?;
        let end = stream.byte_offset();
        parse_strict_json(&raw[start..end], limits()).map_err(|_| E::Metadata)?;
        rows.push(serde_json::from_value(value).map_err(|_| E::Metadata)?);
        start = end;
    }
    need(!rows.is_empty() && rows.len() <= 32768, E::Membership)?;
    Ok(rows)
}
fn join(base: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        base.into()
    } else {
        format!("{}/{suffix}", base.trim_end_matches('/'))
    }
}
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(p, _)| p)
}
fn clean_absolute(path: &str) -> ModuleResult<String> {
    need(
        path.starts_with('/') && !path.contains('\\') && !path.chars().any(|c| c < ' '),
        E::Replacement,
    )?;
    let mut parts = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    Ok(format!("/{}", parts.join("/")))
}
fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            out.push('!');
            out.push(c.to_ascii_lowercase())
        } else {
            out.push(c)
        }
    }
    out
}

#[derive(Default)]
struct Manifest {
    module: String,
    go: Option<String>,
    replacements: Vec<(ModuleVersion, ModuleVersion)>,
}
type ModuleVersion = (String, String);
// Read original tokens/line blocks. This deliberately projects only identity,
// Go requirement and replacements; it is not a complete modfile validator or
// MVS implementation. Unknown directive semantics are neither erased from the
// original buffer nor promoted to proof facts.
fn manifest(raw: &[u8]) -> ModuleResult<Manifest> {
    let text = std::str::from_utf8(raw).map_err(|_| E::Manifest)?;
    let mut at = 0;
    let mut lines = vec![];
    let mut line = vec![];
    while at < text.len() {
        let b = text.as_bytes()[at];
        if matches!(b, b' ' | b'\t' | b'\r') {
            at += 1;
            continue;
        }
        if b == b'\n' {
            lines.push(std::mem::take(&mut line));
            at += 1;
            continue;
        }
        if text[at..].starts_with("//") {
            at += text[at..].find('\n').unwrap_or(text.len() - at);
            continue;
        }
        need(!text[at..].starts_with("/*"), E::Manifest)?;
        let start = at;
        if b == b'"' || b == b'`' {
            at += 1;
            let mut closed = false;
            while at < text.len() {
                let c = text.as_bytes()[at];
                at += 1;
                need(c != b'\n', E::Manifest)?;
                if c == b {
                    closed = true;
                    break;
                }
                if c == b'\\' && b == b'"' {
                    need(at < text.len() && text.as_bytes()[at] != b'\n', E::Manifest)?;
                    at += 1
                }
            }
            need(closed, E::Manifest)?;
        } else if matches!(b, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',') {
            at += 1
        } else {
            while at < text.len() {
                let c = text[at..].chars().next().unwrap();
                if c.is_whitespace()
                    || matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',')
                    || text[at..].starts_with("//")
                    || text[at..].starts_with("/*")
                {
                    break;
                }
                need(!c.is_control(), E::Manifest)?;
                at += c.len_utf8();
            }
            need(at > start, E::Manifest)?;
        }
        line.push(text[start..at].to_string());
    }
    lines.push(line);
    let string = |s: &str| -> ModuleResult<String> {
        if s.starts_with('"') {
            String::from_utf8(source_scopes::unquote(s.as_bytes()).map_err(|_| E::Manifest)?)
                .map_err(|_| E::Manifest)
        } else {
            need(!s.contains(['\'', '"', '`']), E::Manifest)?;
            Ok(s.into())
        }
    };
    let mut result = Manifest::default();
    let mut block: Option<String> = None;
    for mut line in lines {
        if line.is_empty() {
            continue;
        }
        if line == [")"] && block.is_some() {
            block = None;
            continue;
        }
        if block.is_none() && line.len() == 2 && line[1] == "(" {
            block = Some(line[0].clone());
            continue;
        }
        if block.is_none() && line.len() == 3 && line[1..] == ["(", ")"] {
            continue;
        }
        let verb = if let Some(block) = &block {
            block.clone()
        } else {
            line.remove(0)
        };
        match verb.as_str() {
            "module" => {
                need(result.module.is_empty() && line.len() == 1, E::Manifest)?;
                result.module = string(&line[0])?;
            }
            "go" => {
                need(result.go.is_none() && line.len() == 1, E::Manifest)?;
                result.go = Some(line[0].clone());
            }
            "replace" => {
                let arrow = line.iter().position(|s| s == "=>").ok_or(E::Manifest)?;
                need(
                    matches!(arrow, 1 | 2) && (line.len() == arrow + 2 || line.len() == arrow + 3),
                    E::Manifest,
                )?;
                let old = (
                    string(&line[0])?,
                    if arrow == 2 {
                        string(&line[1])?
                    } else {
                        String::new()
                    },
                );
                let new = (
                    string(&line[arrow + 1])?,
                    if line.len() == arrow + 3 {
                        string(&line[arrow + 2])?
                    } else {
                        String::new()
                    },
                );
                result.replacements.push((old, new));
            }
            _ => {}
        }
    }
    need(block.is_none() && !result.module.is_empty(), E::Manifest)?;
    Ok(result)
}
fn replacement<'a>(
    main: &'a Manifest,
    path: &str,
    version: &str,
) -> ModuleResult<Option<&'a ModuleVersion>> {
    let (mut exact, mut wildcard) = (None, None);
    for ((old_path, old_version), target) in &main.replacements {
        if old_path != path {
            continue;
        }
        if old_version == version {
            need(exact.replace(target).is_none(), E::Replacement)?;
        } else if old_version.is_empty() {
            need(wildcard.replace(target).is_none(), E::Replacement)?;
        }
    }
    Ok(exact.or(wildcard))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModuleObservations {
    exports: GoSourceExportInputs,
    canonical: Vec<u8>,
    metadata: Vec<u8>,
    diagnostics: Vec<u8>,
    manifests: BTreeMap<String, GoTypeSourceObject>,
}
impl GoSourceModuleObservations {
    pub fn export_inputs(&self) -> &GoSourceExportInputs {
        &self.exports
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
    pub fn module_manifest_sources(&self) -> &BTreeMap<String, GoTypeSourceObject> {
        &self.manifests
    }
}
pub fn import_go_source_module_observations(
    record: &[u8],
    exports: GoSourceExportInputs,
) -> ModuleResult<GoSourceModuleObservations> {
    let wire: Wire = decode(record).map_err(E::Input)?;
    need(
        wire.schema == "mpk.go.practical.source_module_observations.v1",
        E::Input(GoTypeKeyError::Shape),
    )?;
    need(
        identity(&wire, DOMAIN, "content_sha256").map_err(E::Input)? == wire.content_sha256,
        E::Input(GoTypeKeyError::Identity),
    )?;
    let export_wire: serde_json::Value =
        serde_json::from_slice(exports.canonical_bytes()).map_err(|_| E::Metadata)?;
    need(
        export_wire["content_sha256"] == wire.source_export_inputs_sha256,
        E::Membership,
    )?;
    let origins = export_wire["origins"]
        .as_array()
        .ok_or(E::Metadata)?
        .iter()
        .map(|r| {
            Ok((
                r["capability"].as_str().ok_or(E::Metadata)?.to_string(),
                r["path"].as_str().ok_or(E::Metadata)?.to_string(),
            ))
        })
        .collect::<ModuleResult<BTreeMap<_, _>>>()?;
    let snapshot = exports.capture_inputs().snapshot_inputs();
    let contracts = exports
        .capture_inputs()
        .selection_inputs()
        .contract_inputs();
    let main_source = snapshot
        .source(contracts.main_source_object_id())
        .ok_or(E::Manifest)?;
    let main = manifest(main_source.bytes())?;
    let physical = |o: &GoTypeSourceObject| -> ModuleResult<String> {
        Ok(join(
            origins.get(&o.capability).ok_or(E::Manifest)?,
            &o.path,
        ))
    };
    let main_directory = parent(&physical(main_source.object())?).to_string();
    let raw = unhex(&wire.metadata_hex)?;
    let diagnostics = unhex(&wire.diagnostics_hex)?;
    let rows = native_rows(&raw)?;
    let native = rows
        .iter()
        .map(|r| (r.path.as_str(), r))
        .collect::<BTreeMap<_, _>>();
    need(
        rows.len() == native.len()
            && rows.len() == wire.modules.len()
            && wire.modules.windows(2).all(|r| r[0].path < r[1].path),
        E::Membership,
    )?;
    let mut manifests = BTreeMap::new();
    let mut roots = BTreeMap::new();
    let mut main_count = 0;
    for module in &wire.modules {
        let row = *native.get(module.path.as_str()).ok_or(E::Membership)?;
        // Consume even informational fields through the typed decoder; they
        // remain in the whole raw stream and confer no resolution authority.
        let _information = (
            &row.indirect,
            &row.time,
            &row.sum,
            &row.go_mod_sum,
            &row.deprecated,
            &row.origin,
        );
        need(
            row.version == module.version
                && row.error.is_none()
                && row.query.is_empty()
                && row.versions.is_empty()
                && row.retracted.is_empty()
                && row.update.is_none()
                && !row.reuse,
            E::Metadata,
        )?;
        let source = snapshot
            .source(&module.manifest_source_object_id)
            .ok_or(E::Manifest)?;
        need(source.object().kind == "go_mod", E::Manifest)?;
        let facts = manifest(source.bytes())?;
        let target = if let Some(target) = &row.replace {
            need(
                !row.main && !target.main && target.replace.is_none() && target.error.is_none(),
                E::Replacement,
            )?;
            need(
                target.path == module.replacement_path
                    && target.version == module.replacement_version,
                E::Replacement,
            )?;
            need(
                replacement(&main, &row.path, &row.version)?
                    == Some(&(target.path.clone(), target.version.clone())),
                E::Replacement,
            )?;
            target.as_ref()
        } else {
            need(
                module.replacement_path.is_empty()
                    && module.replacement_version.is_empty()
                    && (row.main || replacement(&main, &row.path, &row.version)?.is_none()),
                E::Replacement,
            )?;
            row
        };
        need(
            row.go_mod == physical(source.object())?
                && target.go_mod == row.go_mod
                && row.dir == target.dir,
            E::Manifest,
        )?;
        if let Some(go) = facts.go {
            need(row.go_version == go && target.go_version == go, E::Manifest)?;
        }
        let directory = if row.main {
            main_count += 1;
            need(
                row.path == main.module
                    && row.version.is_empty()
                    && row.replace.is_none()
                    && source.object() == main_source.object()
                    && row.dir == main_directory,
                E::Manifest,
            )?;
            row.dir.clone()
        } else if target.version.is_empty() {
            let intended = if target.path.starts_with('/') {
                target.path.clone()
            } else {
                join(&main_directory, &target.path)
            };
            let intended = clean_absolute(&intended)?;
            need(
                row.replace.is_some()
                    && row.dir == intended
                    && row.go_mod == join(&intended, "go.mod"),
                E::Replacement,
            )?;
            let mut candidates = origins
                .iter()
                .filter(|(_, p)| {
                    intended == **p
                        || intended.starts_with(&format!("{}/", p.trim_end_matches('/')))
                })
                .collect::<Vec<_>>();
            candidates.sort_by_key(|(_, p)| p.len());
            let (cap, origin) = candidates.last().ok_or(E::Replacement)?;
            need(
                candidates
                    .iter()
                    .filter(|(_, p)| p.len() == origin.len())
                    .count()
                    == 1
                    && source.object().capability == **cap,
                E::Replacement,
            )?;
            intended
        } else {
            let suffix = format!(
                "/cache/download/{}/@v/{}.mod",
                escape(&target.path),
                escape(&target.version)
            );
            let cache = row.go_mod.strip_suffix(&suffix).ok_or(E::Manifest)?;
            let directory = join(
                cache,
                &format!("{}@{}", escape(&target.path), escape(&target.version)),
            );
            need(row.dir.is_empty() || row.dir == directory, E::Manifest)?;
            directory
        };
        roots.insert(row.path.clone(), directory);
        manifests.insert(row.path.clone(), source.object().clone());
    }
    need(main_count == 1, E::Membership)?;
    let packages = serde_json::Deserializer::from_slice(exports.original_metadata())
        .into_iter::<serde_json::Value>();
    let keys = contracts.source_inputs().components();
    for package in packages {
        let package = package.map_err(|_| E::Metadata)?;
        if package["Standard"] == true {
            continue;
        }
        let p = &package["Module"];
        let observed: Native = serde_json::from_value(p.clone()).map_err(|_| E::Package)?;
        need(
            observed.error.is_none()
                && observed
                    .replace
                    .as_ref()
                    .is_none_or(|r| !r.main && r.error.is_none() && r.replace.is_none()),
            E::Package,
        )?;
        let path = p["Path"].as_str().ok_or(E::Package)?;
        let row = *native.get(path).ok_or(E::Package)?;
        let target = row.replace.as_deref().unwrap_or(row);
        need(
            p["Version"].as_str().unwrap_or("") == row.version
                && p["Main"].as_bool().unwrap_or(false) == row.main
                && p["GoMod"].as_str() == Some(row.go_mod.as_str())
                && p["GoVersion"].as_str().unwrap_or("") == row.go_version,
            E::Package,
        )?;
        need(
            p["Dir"].as_str() == roots.get(path).map(String::as_str),
            E::Package,
        )?;
        if let Some(replaced) = &row.replace {
            need(
                p["Replace"]["Path"].as_str() == Some(replaced.path.as_str())
                    && p["Replace"]["Version"].as_str().unwrap_or("") == replaced.version
                    && p["Replace"]["GoMod"].as_str() == Some(target.go_mod.as_str())
                    && p["Replace"]["Dir"].as_str() == roots.get(path).map(String::as_str)
                    && p["Replace"]["GoVersion"].as_str().unwrap_or("") == target.go_version,
                E::Package,
            )?;
        } else {
            need(
                p.get("Replace").is_none_or(serde_json::Value::is_null),
                E::Package,
            )?;
        }
        let name = package["ImportPath"].as_str().ok_or(E::Package)?;
        let suffix = if name == path {
            ""
        } else {
            name.strip_prefix(&format!("{path}/")).ok_or(E::Package)?
        };
        need(
            package["Dir"].as_str()
                == Some(join(roots.get(path).ok_or(E::Package)?, suffix).as_str()),
            E::Package,
        )?;
        let key = keys
            .wire
            .packages
            .iter()
            .find(|p| p.import_path == name)
            .ok_or(E::Package)?;
        need(
            key.module_source_object_id == manifests.get(path).ok_or(E::Package)?.id,
            E::Package,
        )?;
    }
    let canonical =
        canonical_json_bytes(&parse_strict_json(record, limits()).map_err(|_| E::Metadata)?)
            .map_err(|_| E::Metadata)?;
    Ok(GoSourceModuleObservations {
        exports,
        canonical,
        metadata: raw,
        diagnostics,
        manifests,
    })
}
