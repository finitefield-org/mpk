//! Whole original native compiler metadata/export linkage. This does not
//! authenticate an installed compiler, roots, context or source/program proof.
use super::*;

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0");
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Origin {
    capability: String,
    path: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Package {
    import_path: String,
    capability: String,
    directory: String,
    export_source_object_id: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    source_capture_inputs_sha256: String,
    image_sha256: String,
    compiler_sha256: String,
    go_binary_sha256: String,
    sdk_capability: String,
    requested_package: String,
    origins: Vec<Origin>,
    packages: Vec<Package>,
    metadata_hex: String,
    diagnostics_hex: String,
    content_sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceExportInputError {
    Input(GoTypeKeyError),
    Root,
    Metadata,
    Membership,
    Source,
    Import,
    Archive,
    Package(GoSourcePackageError),
}
impl std::fmt::Display for GoSourceExportInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original export inputs: {self:?}")
    }
}
impl std::error::Error for GoSourceExportInputError {}
type ExportResult<T> = std::result::Result<T, GoSourceExportInputError>;
fn need(ok: bool, error: GoSourceExportInputError) -> ExportResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn unhex(s: &str) -> ExportResult<Vec<u8>> {
    need(
        s.len().is_multiple_of(2)
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        GoSourceExportInputError::Metadata,
    )?;
    Ok(s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect())
}
fn absolute(path: &str) -> bool {
    path.starts_with('/')
        && !path.contains('\\')
        && !path.chars().any(|c| c < ' ')
        && (path == "/" || path[1..].split('/').all(|s| !matches!(s, "" | "." | "..")))
}
fn join(base: &str, path: &str) -> String {
    if path.is_empty() {
        base.into()
    } else {
        format!("{}/{path}", base.trim_end_matches('/'))
    }
}
fn names(objects: &[GoTypeSourceObject]) -> Vec<String> {
    objects
        .iter()
        .map(|o| o.path.rsplit('/').next().unwrap().into())
        .collect()
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Native {
    dir: String,
    import_path: String,
    name: String,
    #[serde(default, rename = "BuildID")]
    build_id: String,
    #[serde(default)]
    export: String,
    #[serde(default)]
    standard: bool,
    #[serde(default)]
    goroot: bool,
    #[serde(default)]
    incomplete: bool,
    #[serde(default)]
    binary_only: bool,
    #[serde(default)]
    for_test: String,
    #[serde(default)]
    dep_only: bool,
    #[serde(default)]
    r#match: Vec<String>,
    #[serde(default)]
    go_files: Vec<String>,
    #[serde(default)]
    compiled_go_files: Vec<String>,
    #[serde(default)]
    cgo_files: Vec<String>,
    #[serde(default)]
    ignored_go_files: Vec<String>,
    #[serde(default)]
    test_go_files: Vec<String>,
    #[serde(default)]
    x_test_go_files: Vec<String>,
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    import_map: BTreeMap<String, String>,
    #[serde(default)]
    embed_patterns: Vec<String>,
    #[serde(default)]
    embed_files: Vec<String>,
    #[serde(default)]
    c_files: Vec<String>,
    #[serde(default, rename = "CXXFiles")]
    cxx_files: Vec<String>,
    #[serde(default)]
    m_files: Vec<String>,
    #[serde(default)]
    h_files: Vec<String>,
    #[serde(default)]
    f_files: Vec<String>,
    #[serde(default)]
    s_files: Vec<String>,
    #[serde(default)]
    swig_files: Vec<String>,
    #[serde(default, rename = "SwigCXXFiles")]
    swig_cxx_files: Vec<String>,
    #[serde(default)]
    syso_files: Vec<String>,
    #[serde(default)]
    ignored_other_files: Vec<String>,
    #[serde(default)]
    error: Option<serde_json::Value>,
    #[serde(default)]
    deps_errors: Vec<serde_json::Value>,
    #[serde(default)]
    module: Option<serde_json::Value>,
}
fn metadata(raw: &[u8]) -> ExportResult<Vec<Native>> {
    let mut stream = serde_json::Deserializer::from_slice(raw).into_iter::<serde_json::Value>();
    let mut start = 0;
    let mut rows = vec![];
    while let Some(row) = stream.next() {
        let row = row.map_err(|_| GoSourceExportInputError::Metadata)?;
        let end = stream.byte_offset();
        parse_strict_json(&raw[start..end], limits())
            .map_err(|_| GoSourceExportInputError::Metadata)?;
        rows.push(serde_json::from_value(row).map_err(|_| GoSourceExportInputError::Metadata)?);
        start = end;
    }
    need(
        !rows.is_empty() && rows.len() <= 32768,
        GoSourceExportInputError::Metadata,
    )?;
    Ok(rows)
}
fn archive(raw: &[u8], build_id: &str) -> ExportResult<()> {
    let bad = GoSourceExportInputError::Archive;
    need(
        raw.len() >= 68 && raw.starts_with(b"!<arch>\n") && !build_id.is_empty(),
        bad.clone(),
    )?;
    let mut at = 8;
    let mut first = true;
    while at < raw.len() {
        let header = raw.get(at..at + 60).ok_or(bad.clone())?;
        let size = std::str::from_utf8(&header[48..58])
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .ok_or(bad.clone())?;
        need(&header[58..60] == b"`\n", bad.clone())?;
        let start = at + 60;
        let end = start.checked_add(size).ok_or(bad.clone())?;
        let body = raw.get(start..end).ok_or(bad.clone())?;
        if first {
            need(header[..16].trim_ascii() == b"__.PKGDEF", bad.clone())?;
            let line = body.iter().position(|b| *b == b'\n').ok_or(bad.clone())?;
            need(
                body[..line].starts_with(b"go object linux amd64 go1.25.0 GOAMD64=v1 X:"),
                bad.clone(),
            )?;
            let rest = &body[line + 1..];
            let line = rest.iter().position(|b| *b == b'\n').ok_or(bad.clone())?;
            let quoted = rest[..line].strip_prefix(b"build id ").ok_or(bad.clone())?;
            let observed: String = serde_json::from_slice(quoted).map_err(|_| bad.clone())?;
            need(observed == build_id, bad.clone())?;
            first = false;
        }
        at = end;
        if size & 1 != 0 {
            need(raw.get(at) == Some(&0), bad.clone())?;
            at += 1;
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceExportInputs {
    capture: GoSourceCaptureInputs,
    canonical: Vec<u8>,
    metadata: Vec<u8>,
    diagnostics: Vec<u8>,
    exports: BTreeMap<String, GoTypeSourceObject>,
}
impl GoSourceExportInputs {
    pub fn capture_inputs(&self) -> &GoSourceCaptureInputs {
        &self.capture
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
    pub fn export_sources(&self) -> &BTreeMap<String, GoTypeSourceObject> {
        &self.exports
    }
}

pub fn import_go_source_export_inputs(
    record: &[u8],
    capture: GoSourceCaptureInputs,
) -> ExportResult<GoSourceExportInputs> {
    use GoSourceExportInputError as E;
    let wire: Wire = observation_json::decode(record).map_err(E::Input)?;
    need(
        wire.schema == "mpk.go.practical.source_export_inputs.v1",
        E::Input(GoTypeKeyError::Shape),
    )?;
    need(
        observation_json::identity(&wire, DOMAIN).map_err(E::Input)? == wire.content_sha256,
        E::Input(GoTypeKeyError::Identity),
    )?;
    let capture_value: serde_json::Value =
        serde_json::from_slice(capture.canonical_bytes()).map_err(|_| E::Metadata)?;
    let contracts = capture.selection_inputs().contract_inputs();
    need(
        capture_value["content_sha256"] == wire.source_capture_inputs_sha256
            && contracts.image_sha256() == wire.image_sha256
            && contracts.compiler_sha256() == wire.compiler_sha256
            && contracts.requested_package() == wire.requested_package,
        E::Root,
    )?;
    need(
        wire.go_binary_sha256.len() == 64
            && wire
                .go_binary_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        E::Root,
    )?;
    let snapshot = capture.snapshot_inputs();
    need(
        wire.origins
            .iter()
            .map(|o| o.capability.as_str())
            .collect::<Vec<_>>()
            == snapshot
                .capabilities()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
            && wire.origins.iter().all(|o| absolute(&o.path)),
        E::Root,
    )?;
    let origins = wire
        .origins
        .iter()
        .map(|o| (o.capability.as_str(), o.path.as_str()))
        .collect::<BTreeMap<_, _>>();
    need(origins.contains_key(wire.sdk_capability.as_str()), E::Root)?;
    let raw = unhex(&wire.metadata_hex)?;
    let diagnostics = unhex(&wire.diagnostics_hex)?;
    let rows = metadata(&raw)?;
    need(
        rows.len() == wire.packages.len()
            && wire
                .packages
                .windows(2)
                .all(|p| p[0].import_path < p[1].import_path),
        E::Membership,
    )?;
    let native = rows
        .iter()
        .map(|r| (r.import_path.as_str(), r))
        .collect::<BTreeMap<_, _>>();
    need(native.len() == rows.len(), E::Membership)?;
    let keys = contracts.source_inputs().components();
    let source_packages = keys
        .wire
        .packages
        .iter()
        .map(|p| (p.import_path.as_str(), p))
        .collect::<BTreeMap<_, _>>();
    let standard = snapshot
        .source_objects()
        .iter()
        .filter(|s| s.capability == wire.sdk_capability && s.kind == "go_file")
        .filter_map(|s| {
            s.path
                .strip_prefix("src/")
                .and_then(|p| p.rsplit_once('/').map(|(d, _)| d.to_string()))
        })
        .collect::<BTreeSet<_>>();
    let mut exports = BTreeMap::new();
    let mut used = BTreeSet::new();
    let mut package_names = BTreeSet::new();
    let mut intrinsic = 0;
    for package in &wire.packages {
        let row = *native
            .get(package.import_path.as_str())
            .ok_or(E::Membership)?;
        need(
            validate::logical_path(&row.import_path)
                && validate::identifier(&row.name)
                && !row.incomplete
                && !row.binary_only
                && row.for_test.is_empty()
                && row.error.is_none()
                && row.deps_errors.is_empty()
                && row.cgo_files.is_empty(),
            E::Metadata,
        )?;
        need(row.standard == row.goroot, E::Root)?;
        if row.import_path == wire.requested_package {
            need(
                !row.dep_only && row.r#match == [wire.requested_package.clone()],
                E::Root,
            )?;
        } else {
            need(row.dep_only && row.r#match.is_empty(), E::Root)?;
        }
        let origin = *origins.get(package.capability.as_str()).ok_or(E::Root)?;
        need(row.dir == join(origin, &package.directory), E::Root)?;
        if row.standard {
            need(
                package.capability == wire.sdk_capability
                    && package.directory == format!("src/{}", row.import_path)
                    && row.module.is_none(),
                E::Root,
            )?;
        }
        let files =
            derive_go_source_package_files(snapshot, &package.capability, &package.directory)
                .map_err(E::Package)?;
        need(
            files.package_name() == row.name
                && !files.is_binary_only()
                && names(files.active_sources()) == row.go_files
                && names(files.excluded_sources()) == row.ignored_go_files
                && names(files.test_sources()) == row.test_go_files
                && names(files.external_test_sources()) == row.x_test_go_files,
            E::Source,
        )?;
        for (kind, native) in [
            (GoSourceAuxiliaryKind::C, &row.c_files),
            (GoSourceAuxiliaryKind::Cxx, &row.cxx_files),
            (GoSourceAuxiliaryKind::ObjectiveC, &row.m_files),
            (GoSourceAuxiliaryKind::Header, &row.h_files),
            (GoSourceAuxiliaryKind::Fortran, &row.f_files),
            (GoSourceAuxiliaryKind::Assembly, &row.s_files),
            (GoSourceAuxiliaryKind::Swig, &row.swig_files),
            (GoSourceAuxiliaryKind::SwigCxx, &row.swig_cxx_files),
            (GoSourceAuxiliaryKind::SystemObject, &row.syso_files),
        ] {
            need(names(files.auxiliary_sources(kind)) == *native, E::Source)?;
        }
        need(
            names(files.ignored_auxiliary_sources()) == row.ignored_other_files,
            E::Source,
        )?;
        need(
            files
                .embed_patterns()
                .iter()
                .map(Vec::as_slice)
                .eq(row.embed_patterns.iter().map(|s| s.as_bytes())),
            E::Source,
        )?;
        let resources =
            derive_go_source_embedded_files(snapshot, &package.capability, &package.directory)
                .map_err(E::Package)?;
        need(resources.files() == row.embed_files, E::Source)?;
        let mut imports = vec![];
        let mut remap = BTreeMap::new();
        for name in files.imports() {
            let mut resolved = name.clone();
            if row.standard {
                let mut from = row.import_path.as_str();
                loop {
                    let candidate = if from.is_empty() {
                        format!("vendor/{name}")
                    } else {
                        format!("{from}/vendor/{name}")
                    };
                    if standard.contains(&candidate) {
                        resolved = candidate;
                        break;
                    }
                    if from.is_empty() {
                        break;
                    }
                    from = from.rsplit_once('/').map_or("", |(p, _)| p);
                }
                need(standard.contains(&resolved), E::Import)?;
            }
            need(native.contains_key(resolved.as_str()), E::Import)?;
            if resolved != *name {
                remap.insert(name.clone(), resolved.clone());
            }
            imports.push(resolved);
        }
        need(imports == row.imports && remap == row.import_map, E::Import)?;
        if row.import_path == "unsafe" {
            intrinsic += 1;
            need(
                row.standard
                    && package.export_source_object_id.is_none()
                    && row.export.is_empty()
                    && row.build_id.is_empty()
                    && !source_packages.contains_key("unsafe"),
                E::Archive,
            )?;
            continue;
        }
        let source = *source_packages
            .get(row.import_path.as_str())
            .ok_or(E::Membership)?;
        package_names.insert(row.import_path.as_str());
        need(
            source.package_name == row.name
                && source
                    .source_objects
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
                    == files
                        .active_sources()
                        .iter()
                        .map(|s| s.id.as_str())
                        .collect::<BTreeSet<_>>(),
            E::Source,
        )?;
        if !row.standard {
            let module = snapshot
                .source(&source.module_source_object_id)
                .ok_or(E::Source)?;
            let native_module = row.module.as_ref().ok_or(E::Root)?;
            let effective = native_module
                .get("Replace")
                .filter(|r| r.is_object())
                .unwrap_or(native_module);
            let origin = *origins
                .get(module.object().capability.as_str())
                .ok_or(E::Root)?;
            need(
                effective.get("GoMod").and_then(|p| p.as_str())
                    == Some(join(origin, &module.object().path).as_str()),
                E::Root,
            )?;
        }
        need(
            row.compiled_go_files.len() == row.go_files.len()
                && row
                    .compiled_go_files
                    .iter()
                    .zip(&row.go_files)
                    .all(|(p, name)| p == name || p == &join(&row.dir, name)),
            E::Source,
        )?;
        let id = package.export_source_object_id.as_ref().ok_or(E::Archive)?;
        let data = snapshot.source(id).ok_or(E::Archive)?;
        let object = data.object();
        need(
            object.kind == "export_data"
                && object.capability == "original-go-package-exports"
                && row.export
                    == join(
                        origins.get(object.capability.as_str()).ok_or(E::Root)?,
                        &object.path,
                    )
                && used.insert(id.as_str()),
            E::Archive,
        )?;
        archive(data.bytes(), &row.build_id)?;
        exports.insert(row.import_path.clone(), object.clone());
    }
    need(
        package_names == source_packages.keys().copied().collect::<BTreeSet<_>>()
            && intrinsic <= 1
            && native.contains_key(wire.requested_package.as_str()),
        E::Membership,
    )?;
    need(
        snapshot
            .source_objects()
            .iter()
            .filter(|s| s.kind == "export_data")
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>()
            == used,
        E::Archive,
    )?;
    Ok(GoSourceExportInputs {
        capture,
        canonical: canonical_json_bytes(
            &parse_strict_json(record, limits()).map_err(|_| E::Metadata)?,
        )
        .map_err(|_| E::Metadata)?,
        metadata: raw,
        diagnostics,
        exports,
    })
}
