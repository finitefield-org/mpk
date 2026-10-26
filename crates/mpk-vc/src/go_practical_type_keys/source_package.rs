//! Independent Go-file membership from a complete immutable directory view.
//! Module/compiler/export ownership and source/program proofs remain separate.
use super::*;
mod auxiliary;
mod cgo;
mod embedded;
mod graphic;
mod header;
pub use auxiliary::GoSourceAuxiliaryKind;
pub use embedded::{derive_go_source_embedded_files, GoSourceEmbeddedFiles};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourcePackageErrorKind {
    Namespace,
    Read,
    Header,
    Import,
    Name,
    NoGo,
    Role,
    CgoTest,
    CgoDirective,
    Membership,
    EmbedPattern,
    EmbedNamespace,
    EmbedSource,
    EmbedName,
    EmbedModule,
    EmbedIrregular,
    EmbedEmpty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourcePackageError {
    kind: GoSourcePackageErrorKind,
    capability: String,
    path: String,
}
impl GoSourcePackageError {
    pub fn kind(&self) -> GoSourcePackageErrorKind {
        self.kind
    }
    pub fn capability(&self) -> &str {
        &self.capability
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}
impl std::fmt::Display for GoSourcePackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original package files: {self:?}")
    }
}
impl std::error::Error for GoSourcePackageError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourcePackageFiles {
    capability: String,
    directory: String,
    name: String,
    active: Vec<GoTypeSourceObject>,
    excluded: Vec<GoTypeSourceObject>,
    tests: Vec<GoTypeSourceObject>,
    external_tests: Vec<GoTypeSourceObject>,
    cgo_ignored: Vec<GoTypeSourceObject>,
    imports: Vec<String>,
    test_imports: Vec<String>,
    external_test_imports: Vec<String>,
    binary_only: bool,
    auxiliary: BTreeMap<GoSourceAuxiliaryKind, Vec<GoTypeSourceObject>>,
    ignored_auxiliary: Vec<GoTypeSourceObject>,
    embed_patterns: Vec<Vec<u8>>,
    test_embed_patterns: Vec<Vec<u8>>,
    external_test_embed_patterns: Vec<Vec<u8>>,
}
impl GoSourcePackageFiles {
    pub fn capability(&self) -> &str {
        &self.capability
    }
    pub fn directory(&self) -> &str {
        &self.directory
    }
    pub fn package_name(&self) -> &str {
        &self.name
    }
    pub fn active_sources(&self) -> &[GoTypeSourceObject] {
        &self.active
    }
    pub fn excluded_sources(&self) -> &[GoTypeSourceObject] {
        &self.excluded
    }
    pub fn test_sources(&self) -> &[GoTypeSourceObject] {
        &self.tests
    }
    pub fn external_test_sources(&self) -> &[GoTypeSourceObject] {
        &self.external_tests
    }
    pub fn cgo_ignored_sources(&self) -> &[GoTypeSourceObject] {
        &self.cgo_ignored
    }
    pub fn imports(&self) -> &[String] {
        &self.imports
    }
    pub fn test_imports(&self) -> &[String] {
        &self.test_imports
    }
    pub fn external_test_imports(&self) -> &[String] {
        &self.external_test_imports
    }
    pub fn is_binary_only(&self) -> bool {
        self.binary_only
    }
    pub fn auxiliary_sources(&self, kind: GoSourceAuxiliaryKind) -> &[GoTypeSourceObject] {
        self.auxiliary.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub fn ignored_auxiliary_sources(&self) -> &[GoTypeSourceObject] {
        &self.ignored_auxiliary
    }
    /// Exact Go string bytes; an invalid UTF-8 pattern remains visible here and
    /// is rejected when deriving actual embedded resources.
    pub fn embed_patterns(&self) -> &[Vec<u8>] {
        &self.embed_patterns
    }
    pub fn test_embed_patterns(&self) -> &[Vec<u8>] {
        &self.test_embed_patterns
    }
    pub fn external_test_embed_patterns(&self) -> &[Vec<u8>] {
        &self.external_test_embed_patterns
    }
}

pub fn derive_go_source_package_files(
    snapshot: &GoSourceSnapshotInputs,
    capability: &str,
    directory: &str,
) -> std::result::Result<GoSourcePackageFiles, GoSourcePackageError> {
    let failure = |kind, path: &str| GoSourcePackageError {
        kind,
        capability: capability.into(),
        path: path.into(),
    };
    let entries = snapshot
        .read_directory(capability, directory)
        .map_err(|_| failure(GoSourcePackageErrorKind::Namespace, directory))?;
    let mut result = GoSourcePackageFiles {
        capability: capability.into(),
        directory: directory.into(),
        name: String::new(),
        active: vec![],
        excluded: vec![],
        tests: vec![],
        external_tests: vec![],
        cgo_ignored: vec![],
        imports: vec![],
        test_imports: vec![],
        external_test_imports: vec![],
        binary_only: false,
        auxiliary: BTreeMap::new(),
        ignored_auxiliary: vec![],
        embed_patterns: vec![],
        test_embed_patterns: vec![],
        external_test_embed_patterns: vec![],
    };
    let mut found_name = false;
    let mut imports = BTreeSet::new();
    let mut test_imports = BTreeSet::new();
    let mut external_imports = BTreeSet::new();
    let mut embed_patterns = BTreeSet::new();
    let mut test_embed_patterns = BTreeSet::new();
    let mut external_embed_patterns = BTreeSet::new();
    for entry in entries {
        let bytes = entry.name_bytes();
        if entry.type_bits() & (1 << 31) != 0 || bytes.starts_with(b"_") || bytes.starts_with(b".")
        {
            continue;
        }
        let auxiliary_kind = GoSourceAuxiliaryKind::filename(bytes);
        if !bytes.ends_with(b".go") && auxiliary_kind.is_none() {
            continue;
        }
        let name = std::str::from_utf8(bytes)
            .map_err(|_| failure(GoSourcePackageErrorKind::Namespace, directory))?;
        let path = if directory.is_empty() {
            name.into()
        } else {
            format!("{directory}/{name}")
        };
        if !bytes.ends_with(b".go") {
            if let Some(kind) = auxiliary_kind {
                let source = snapshot
                    .read_source(capability, &path, "auxiliary")
                    .map_err(|_| failure(GoSourcePackageErrorKind::Read, &path))?;
                if auxiliary::select(source, kind) {
                    result
                        .auxiliary
                        .entry(kind)
                        .or_default()
                        .push(source.object().clone());
                } else {
                    result.ignored_auxiliary.push(source.object().clone());
                }
            }
            continue;
        }
        let source = snapshot
            .read_source(capability, &path, "go_file")
            .or_else(|e| {
                if e == GoSourceSnapshotLookupError::Role {
                    snapshot.read_source(capability, &path, "excluded_source")
                } else {
                    Err(e)
                }
            })
            .map_err(|_| failure(GoSourcePackageErrorKind::Read, &path))?;
        let matches = derive_go_source_build_match(source)
            .map_err(|_| failure(GoSourcePackageErrorKind::Header, &path))?;
        let mut ignored = !matches.matches_target();
        let mut test = false;
        let mut external = false;
        let mut cgo = false;
        let mut file_imports = vec![];
        if !ignored {
            let end = header::boundary(source).map_err(|kind| failure(kind, &path))?;
            let header = header::read(source, end).map_err(|kind| failure(kind, &path))?;
            let mut package = header.name;
            file_imports = header.imports;
            ignored = package == "documentation";
            if !ignored {
                test = matches.is_test_file();
                if test && package.ends_with("_test") && (!found_name || result.name != package) {
                    external = true;
                    package.truncate(package.len() - 5);
                }
                if !found_name {
                    result.name = package;
                    found_name = true;
                } else if result.name != package {
                    return Err(failure(GoSourcePackageErrorKind::Name, &path));
                }
                cgo = file_imports.iter().any(|p| p == "C");
                if cgo && test {
                    return Err(failure(GoSourcePackageErrorKind::CgoTest, &path));
                }
                if cgo {
                    cgo::validate(&header.cgo_docs)
                        .map_err(|_| failure(GoSourcePackageErrorKind::CgoDirective, &path))?;
                }
                ignored = cgo;
            }
        }
        // These lists are exact byte/location identities in native filename
        // order. All non-production Go buffers retain the captured exclusion role.
        let expected = if ignored || test {
            "excluded_source"
        } else {
            "go_file"
        };
        if source.object().kind != expected {
            return Err(failure(GoSourcePackageErrorKind::Role, &path));
        }
        if ignored {
            result.excluded.push(source.object().clone());
            if cgo {
                result.cgo_ignored.push(source.object().clone());
            }
        } else if external {
            result.external_tests.push(source.object().clone());
            if file_imports.iter().any(|p| p == "embed") {
                external_embed_patterns.extend(embedded::source_patterns(source));
            }
            external_imports.extend(file_imports);
        } else if test {
            result.tests.push(source.object().clone());
            if file_imports.iter().any(|p| p == "embed") {
                test_embed_patterns.extend(embedded::source_patterns(source));
            }
            test_imports.extend(file_imports);
        } else {
            result.active.push(source.object().clone());
            if file_imports.iter().any(|p| p == "embed") {
                embed_patterns.extend(embedded::source_patterns(source));
            }
            imports.extend(file_imports);
        }
        result.binary_only |= matches.is_binary_only();
    }
    if result.active.is_empty() && result.tests.is_empty() && result.external_tests.is_empty() {
        return Err(failure(GoSourcePackageErrorKind::NoGo, directory));
    }
    result.imports = imports.into_iter().collect();
    result.test_imports = test_imports.into_iter().collect();
    result.external_test_imports = external_imports.into_iter().collect();
    result.embed_patterns = embed_patterns.into_iter().collect();
    result.test_embed_patterns = test_embed_patterns.into_iter().collect();
    result.external_test_embed_patterns = external_embed_patterns.into_iter().collect();
    Ok(result)
}

pub(super) fn check_capture_package_files(
    selection: &GoSourceSelectionInputs,
    snapshot: &GoSourceSnapshotInputs,
) -> std::result::Result<(), GoSourcePackageError> {
    let keys = selection.contract_inputs().source_inputs().components();
    for package in &keys.wire.packages {
        let first = keys
            .sources
            .get(
                package
                    .source_objects
                    .first()
                    .ok_or_else(|| GoSourcePackageError {
                        kind: GoSourcePackageErrorKind::Membership,
                        capability: String::new(),
                        path: package.import_path.clone(),
                    })?,
            )
            .unwrap();
        let directory = first.object().path.rsplit_once('/').map_or("", |(d, _)| d);
        let files =
            derive_go_source_package_files(snapshot, &first.object().capability, directory)?;
        embedded::resources(snapshot, &files)?;
        let derived = files
            .active_sources()
            .iter()
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>();
        if files.package_name() != package.package_name
            || files.is_binary_only()
            || derived
                != package
                    .source_objects
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
        {
            return Err(GoSourcePackageError {
                kind: GoSourcePackageErrorKind::Membership,
                capability: first.object().capability.clone(),
                path: directory.into(),
            });
        }
    }
    Ok(())
}
