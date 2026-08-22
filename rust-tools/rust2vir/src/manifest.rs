use crate::cli::LowerRequest;
use crate::path::{PortablePath, PortablePathError};
use crate::preflight::StructuralPreflight;
use crate::source_capture::{CaptureFailure, InputKind};
use std::collections::{BTreeMap, BTreeSet};

const TOML_NESTING_MAX: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestStatus {
    Rejected,
    SourceError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestCode {
    LimitPath,
    PreflightFileType,
    PreflightPath,
    SourceManifestParse,
    ManifestField,
    Workspace,
    Dependency,
    BuildScript,
    Feature,
    Target,
    Lockfile,
}

impl ManifestCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LimitPath => "RUST_LIMIT_PATH",
            Self::PreflightFileType => "RUST_PREFLIGHT_FILE_TYPE",
            Self::PreflightPath => "RUST_PREFLIGHT_PATH",
            Self::SourceManifestParse => "RUST_SOURCE_MANIFEST_PARSE",
            Self::ManifestField => "RUST_PREFLIGHT_MANIFEST_FIELD",
            Self::Workspace => "RUST_PREFLIGHT_WORKSPACE",
            Self::Dependency => "RUST_PREFLIGHT_DEPENDENCY",
            Self::BuildScript => "RUST_PREFLIGHT_BUILD_SCRIPT",
            Self::Feature => "RUST_PREFLIGHT_FEATURE",
            Self::Target => "RUST_PREFLIGHT_TARGET",
            Self::Lockfile => "RUST_PREFLIGHT_LOCKFILE",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::LimitPath => "normalized path limit exceeded",
            Self::PreflightFileType => "input file type is not permitted",
            Self::PreflightPath => "input path is not portable and contained",
            Self::SourceManifestParse => "Cargo manifest could not be parsed",
            Self::ManifestField => "Cargo manifest field is not permitted",
            Self::Workspace => "Cargo workspace authority is not permitted",
            Self::Dependency => "Cargo dependencies are not permitted",
            Self::BuildScript => "Cargo build scripts are not permitted",
            Self::Feature => "Cargo features are not permitted",
            Self::Target => "Cargo package or target selection is not permitted",
            Self::Lockfile => "Cargo lockfile does not match the selected package",
        }
    }

    pub fn status(self) -> ManifestStatus {
        match self {
            Self::SourceManifestParse => ManifestStatus::SourceError,
            _ => ManifestStatus::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestError {
    pub code: ManifestCode,
}

impl From<ManifestCode> for ManifestError {
    fn from(code: ManifestCode) -> Self {
        Self { code }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedManifestSelection {
    package_name: String,
    package_version: String,
    crate_name: String,
    library_path: PortablePath,
    edition: &'static str,
    kind: &'static str,
}

impl ExpectedManifestSelection {
    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn package_version(&self) -> &str {
        &self.package_version
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn library_path(&self) -> &PortablePath {
        &self.library_path
    }

    pub fn edition(&self) -> &'static str {
        self.edition
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }
}

#[derive(Debug)]
pub struct ValidatedManifest {
    preflight: StructuralPreflight,
    selection: ExpectedManifestSelection,
}

impl ValidatedManifest {
    pub fn selection(&self) -> &ExpectedManifestSelection {
        &self.selection
    }

    pub(crate) fn into_parts(self) -> (StructuralPreflight, ExpectedManifestSelection) {
        (self.preflight, self.selection)
    }
}

pub fn validate(
    request: &LowerRequest,
    preflight: StructuralPreflight,
) -> Result<ValidatedManifest, ManifestError> {
    let manifest_bytes = input_bytes(&preflight, InputKind::BuildManifest)?;
    let lockfile_bytes = preflight
        .inputs
        .iter()
        .find(|input| input.kind == InputKind::Lockfile)
        .map(|input| input.bytes.as_ref());

    // TOML lexical/structural parsing owns the phase before any closed-shape finding.
    let manifest =
        Document::parse(manifest_bytes).map_err(|_| ManifestCode::SourceManifestParse)?;
    let lockfile = lockfile_bytes
        .map(Document::parse)
        .transpose()
        .map_err(|_| ManifestCode::SourceManifestParse)?;

    let (selection, mut findings) = validate_manifest_document(request, &manifest);
    match lockfile.as_ref() {
        Some(lockfile) => validate_lockfile(lockfile, selection.as_ref(), &mut findings),
        None => findings.lockfile = true,
    }
    inspect_implicit_build_script(&preflight, &mut findings)?;

    if let Some(code) = findings.primary() {
        return Err(code.into());
    }
    let selection = selection.ok_or(ManifestCode::Target)?;
    Ok(ValidatedManifest {
        preflight,
        selection,
    })
}

fn input_bytes(preflight: &StructuralPreflight, kind: InputKind) -> Result<&[u8], ManifestError> {
    preflight
        .inputs
        .iter()
        .find(|input| input.kind == kind)
        .map(|input| input.bytes.as_ref())
        .ok_or_else(|| ManifestCode::PreflightFileType.into())
}

fn inspect_implicit_build_script(
    preflight: &StructuralPreflight,
    findings: &mut Findings,
) -> Result<(), ManifestError> {
    let path = PortablePath::parse("build.rs").expect("fixed portable path");
    match preflight.capture.open_candidate(&path) {
        Ok(Some(_)) => findings.build_script = true,
        Ok(None) => {}
        Err(CaptureFailure::Missing) => return Err(ManifestCode::PreflightFileType.into()),
        Err(CaptureFailure::FileType) => return Err(ManifestCode::PreflightFileType.into()),
        Err(CaptureFailure::Path) => return Err(ManifestCode::PreflightPath.into()),
        Err(CaptureFailure::PathLimit) => return Err(ManifestCode::LimitPath.into()),
        Err(CaptureFailure::ByteLimit | CaptureFailure::CountLimit) => {
            return Err(ManifestCode::PreflightFileType.into());
        }
    }
    Ok(())
}

#[derive(Default)]
struct Findings {
    limit_path: bool,
    path: bool,
    manifest_field: bool,
    workspace: bool,
    dependency: bool,
    build_script: bool,
    feature: bool,
    target: bool,
    lockfile: bool,
}

impl Findings {
    fn primary(&self) -> Option<ManifestCode> {
        if self.limit_path {
            Some(ManifestCode::LimitPath)
        } else if self.path {
            Some(ManifestCode::PreflightPath)
        } else if self.manifest_field {
            Some(ManifestCode::ManifestField)
        } else if self.workspace {
            Some(ManifestCode::Workspace)
        } else if self.dependency {
            Some(ManifestCode::Dependency)
        } else if self.build_script {
            Some(ManifestCode::BuildScript)
        } else if self.feature {
            Some(ManifestCode::Feature)
        } else if self.target {
            Some(ManifestCode::Target)
        } else if self.lockfile {
            Some(ManifestCode::Lockfile)
        } else {
            None
        }
    }

    fn portable_path(&mut self, value: &str) -> Option<PortablePath> {
        match PortablePath::parse(value) {
            Ok(path) => Some(path),
            Err(PortablePathError::Limit) => {
                self.limit_path = true;
                None
            }
            Err(PortablePathError::Invalid | PortablePathError::Collision) => {
                self.path = true;
                None
            }
        }
    }
}

fn validate_manifest_document(
    request: &LowerRequest,
    document: &Document,
) -> (Option<ExpectedManifestSelection>, Findings) {
    let mut findings = Findings::default();
    let mut package_name = None;
    let mut package_version = None;
    let mut edition = None;
    let mut library_name = None;
    let mut library_path = None;
    let mut feature_table = false;
    let mut feature_default = None;

    for header in &document.headers {
        let keys = header.key_names();
        match keys.as_slice() {
            ["package"] if !header.array => {}
            ["lib"] if !header.array => {}
            ["features"] if !header.array => feature_table = true,
            [name] if dependency_table(name) && !header.array => {}
            [name] if nonselected_target(name) && header.array => {}
            ["workspace", ..] => findings.workspace = true,
            ["target", ..] => findings.dependency = true,
            [name, ..] if dependency_table(name) => findings.dependency = true,
            ["lib", ..] => findings.target = true,
            [name] if nonselected_target(name) => findings.target = true,
            _ => findings.manifest_field = true,
        }
    }

    for entry in &document.expanded_entries() {
        let keys = entry.key_names();
        if keys.contains(&"workspace") {
            findings.workspace = true;
            continue;
        }
        let Some(first) = keys.first().copied() else {
            findings.manifest_field = true;
            continue;
        };
        if first == "target" || dependency_table(first) {
            if keys.len() == 1 && entry.value.is_empty_inline_table() {
                continue;
            }
            findings.dependency = true;
            continue;
        }

        match keys.as_slice() {
            ["package", "name"] => match entry.value.string() {
                Some(value) => package_name = Some(value.to_owned()),
                None => findings.manifest_field = true,
            },
            ["package", "version"] => match entry.value.string() {
                Some(value) => package_version = Some(value.to_owned()),
                None => findings.manifest_field = true,
            },
            ["package", "edition"] => match entry.value.string() {
                Some(value) => edition = Some(value.to_owned()),
                None => findings.manifest_field = true,
            },
            ["package", "publish"] => {
                if entry.value != Value::Bool(false) {
                    findings.manifest_field = true;
                }
            }
            ["package", field] if descriptive_string_field(field) => {
                if entry.value.string().is_none() {
                    findings.manifest_field = true;
                }
            }
            ["package", field] if descriptive_string_array_field(field) => {
                if !entry.value.is_string_array() {
                    findings.manifest_field = true;
                }
            }
            ["package", "build"] => findings.build_script = true,
            ["package", _] | ["package", ..] => findings.manifest_field = true,
            ["lib", "name"] => match entry.value.string() {
                Some(value) => library_name = Some(value.to_owned()),
                None => findings.target = true,
            },
            ["lib", "path"] => match entry.value.string() {
                Some(value) => library_path = findings.portable_path(value),
                None => findings.target = true,
            },
            ["lib", "crate-type" | "proc-macro"] => findings.target = true,
            ["lib", _] | ["lib", ..] => findings.manifest_field = true,
            ["features", "default"] => {
                feature_table = true;
                feature_default = Some(entry.value.is_empty_string_array());
            }
            ["features", ..] => findings.feature = true,
            [name] if dependency_table(name) && entry.value.is_empty_inline_table() => {}
            [name, ..] if dependency_table(name) => findings.dependency = true,
            [name, field] if nonselected_target(name) => {
                validate_nonselected_target_field(entry, field, &mut findings)
            }
            [name] if nonselected_target(name) => findings.target = true,
            _ => findings.manifest_field = true,
        }
    }

    if feature_table && feature_default != Some(true) {
        findings.feature = true;
    }

    let package_name_valid = package_name.as_deref().is_some_and(valid_package_name);
    let version_valid = package_version.as_deref().is_some_and(valid_semver);
    let edition_valid = edition.as_deref() == Some("2021");
    if !package_name_valid
        || !version_valid
        || !edition_valid
        || package_name.as_deref() != Some(request.selection.package.as_str())
    {
        findings.target = true;
    }

    let crate_name = library_name.unwrap_or_else(|| {
        package_name
            .as_deref()
            .unwrap_or_default()
            .replace('-', "_")
    });
    if !valid_rust_identifier(&crate_name) || crate_name != request.selection.crate_name {
        findings.target = true;
    }
    let library_path = library_path
        .or_else(|| findings.portable_path("src/lib.rs"))
        .expect("fixed default library path is portable");

    let selection = match (package_name, package_version) {
        (Some(package_name), Some(package_version)) => Some(ExpectedManifestSelection {
            package_name,
            package_version,
            crate_name,
            library_path,
            edition: "2021",
            kind: "lib",
        }),
        _ => None,
    };
    (selection, findings)
}

fn validate_nonselected_target_field(entry: &Entry, field: &str, findings: &mut Findings) {
    if entry.first_index().is_none() {
        findings.target = true;
        return;
    }
    match field {
        "name" => {
            if entry
                .value
                .string()
                .is_none_or(|value| !valid_rust_identifier(value))
            {
                findings.target = true;
            }
        }
        "path" => match entry.value.string() {
            Some(value) => {
                findings.portable_path(value);
            }
            None => findings.target = true,
        },
        "test" | "bench" | "doc" => {
            if !matches!(entry.value, Value::Bool(_)) {
                findings.target = true;
            }
        }
        "required-features" => {
            if !entry.value.is_empty_string_array() {
                findings.feature = true;
            }
        }
        "crate-type" | "proc-macro" => findings.target = true,
        _ => findings.manifest_field = true,
    }
}

fn validate_lockfile(
    document: &Document,
    selection: Option<&ExpectedManifestSelection>,
    findings: &mut Findings,
) {
    let mut lock_version = None;
    let mut packages = BTreeSet::new();
    let mut package_name = None;
    let mut package_version = None;

    for header in &document.headers {
        let keys = header.key_names();
        if keys.as_slice() != ["package"] || !header.array {
            findings.lockfile = true;
        } else if let Some(index) = header.first_index() {
            packages.insert(index);
        } else {
            findings.lockfile = true;
        }
    }
    for entry in &document.entries {
        let keys = entry.key_names();
        match keys.as_slice() {
            ["version"] if entry.first_index().is_none() => {
                lock_version = entry.value.bare().map(str::to_owned)
            }
            ["package", "name"] => match entry.value.string() {
                Some(value) => package_name = Some(value.to_owned()),
                None => findings.lockfile = true,
            },
            ["package", "version"] => match entry.value.string() {
                Some(value) => package_version = Some(value.to_owned()),
                None => findings.lockfile = true,
            },
            _ => findings.lockfile = true,
        }
    }
    if lock_version.as_deref() != Some("4") || packages.len() != 1 {
        findings.lockfile = true;
    }
    match selection {
        Some(selection)
            if package_name.as_deref() == Some(selection.package_name())
                && package_version.as_deref() == Some(selection.package_version()) => {}
        _ => findings.lockfile = true,
    }
}

fn descriptive_string_field(field: &str) -> bool {
    matches!(
        field,
        "description" | "homepage" | "documentation" | "repository" | "license"
    )
}

fn descriptive_string_array_field(field: &str) -> bool {
    matches!(field, "authors" | "keywords" | "categories")
}

fn dependency_table(value: &str) -> bool {
    matches!(
        value,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

fn nonselected_target(value: &str) -> bool {
    matches!(value, "bin" | "example" | "test" | "bench")
}

fn valid_package_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn valid_rust_identifier(value: &str) -> bool {
    if value == "_" || !value.is_ascii() {
        return false;
    }
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_semver(value: &str) -> bool {
    if value.is_empty() || !value.is_ascii() || value.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return false;
    }
    let mut plus = value.split('+');
    let before_build = plus.next().unwrap_or_default();
    let build = plus.next();
    if plus.next().is_some() || build.is_some_and(|part| !valid_semver_identifiers(part, false)) {
        return false;
    }
    let mut dash = before_build.splitn(2, '-');
    let core = dash.next().unwrap_or_default();
    if dash
        .next()
        .is_some_and(|part| !valid_semver_identifiers(part, true))
    {
        return false;
    }
    let components = core.split('.').collect::<Vec<_>>();
    components.len() == 3 && components.into_iter().all(valid_decimal_identifier)
}

fn valid_semver_identifiers(value: &str, numeric_canonical: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!numeric_canonical
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || valid_decimal_identifier(identifier))
        })
}

fn valid_decimal_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Header {
    path: Vec<PathSegment>,
    array: bool,
}

impl Header {
    fn key_names(&self) -> Vec<&str> {
        key_names(&self.path)
    }

    fn first_index(&self) -> Option<usize> {
        self.path.iter().find_map(|segment| match segment {
            PathSegment::Index(index) => Some(*index),
            PathSegment::Key(_) => None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Entry {
    path: Vec<PathSegment>,
    value: Value,
}

impl Entry {
    fn key_names(&self) -> Vec<&str> {
        key_names(&self.path)
    }

    fn first_index(&self) -> Option<usize> {
        self.path.iter().find_map(|segment| match segment {
            PathSegment::Index(index) => Some(*index),
            PathSegment::Key(_) => None,
        })
    }
}

fn key_names(path: &[PathSegment]) -> Vec<&str> {
    path.iter()
        .filter_map(|segment| match segment {
            PathSegment::Key(key) => Some(key.as_str()),
            PathSegment::Index(_) => None,
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Value {
    String(String),
    Bool(bool),
    Bare(String),
    Array(Vec<Value>),
    InlineTable(Vec<(Vec<String>, Value)>),
}

impl Value {
    fn string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn bare(&self) -> Option<&str> {
        match self {
            Self::Bare(value) => Some(value),
            _ => None,
        }
    }

    fn is_string_array(&self) -> bool {
        matches!(self, Self::Array(values) if values.iter().all(|value| matches!(value, Self::String(_))))
    }

    fn is_empty_string_array(&self) -> bool {
        matches!(self, Self::Array(values) if values.is_empty())
    }

    fn is_empty_inline_table(&self) -> bool {
        matches!(self, Self::InlineTable(values) if values.is_empty())
    }
}

#[derive(Debug)]
struct Document {
    headers: Vec<Header>,
    entries: Vec<Entry>,
}

impl Document {
    fn parse(bytes: &[u8]) -> Result<Self, TomlError> {
        std::str::from_utf8(bytes).map_err(|_| TomlError)?;
        if bytes
            .iter()
            .any(|byte| (*byte < 0x20 && !matches!(*byte, b'\t' | b'\n' | b'\r')) || *byte == 0x7f)
        {
            return Err(TomlError);
        }
        Parser::new(bytes).parse()
    }

    fn expanded_entries(&self) -> Vec<Entry> {
        let mut expanded = Vec::new();
        for entry in &self.entries {
            expand_inline_entry(entry.path.clone(), entry.value.clone(), &mut expanded);
        }
        expanded
    }
}

fn expand_inline_entry(path: Vec<PathSegment>, value: Value, expanded: &mut Vec<Entry>) {
    match value {
        Value::InlineTable(values) if !values.is_empty() => {
            for (keys, value) in values {
                let mut nested = path.clone();
                nested.extend(keys.into_iter().map(PathSegment::Key));
                expand_inline_entry(nested, value, expanded);
            }
        }
        value => expanded.push(Entry { path, value }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TomlError;

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    current_table: Vec<PathSegment>,
    array_counts: BTreeMap<Vec<String>, usize>,
    declared_tables: BTreeSet<Vec<String>>,
    declared_arrays: BTreeSet<Vec<String>>,
    sealed_dotted_tables: BTreeSet<Vec<PathSegment>>,
    seen_values: BTreeSet<Vec<PathSegment>>,
    headers: Vec<Header>,
    entries: Vec<Entry>,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            current_table: Vec::new(),
            array_counts: BTreeMap::new(),
            declared_tables: BTreeSet::new(),
            declared_arrays: BTreeSet::new(),
            sealed_dotted_tables: BTreeSet::new(),
            seen_values: BTreeSet::new(),
            headers: Vec::new(),
            entries: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<Document, TomlError> {
        self.skip_document_trivia()?;
        while self.position < self.bytes.len() {
            if self.peek() == Some(b'[') {
                self.parse_header()?;
            } else {
                self.parse_key_value()?;
            }
            self.finish_statement()?;
            self.skip_document_trivia()?;
        }
        Ok(Document {
            headers: self.headers,
            entries: self.entries,
        })
    }

    fn parse_header(&mut self) -> Result<(), TomlError> {
        self.expect(b'[')?;
        let array = self.consume(b'[');
        self.skip_inline_space();
        let keys = self.parse_key_path()?;
        self.skip_inline_space();
        self.expect(b']')?;
        if array {
            self.expect(b']')?;
        }
        if keys.is_empty() {
            return Err(TomlError);
        }
        let key_path = keys
            .iter()
            .cloned()
            .map(PathSegment::Key)
            .collect::<Vec<_>>();
        if self
            .seen_values
            .iter()
            .any(|value| value == &key_path || key_path.starts_with(value))
            || self.sealed_dotted_tables.contains(&key_path)
            || (array && self.declared_tables.contains(&keys))
            || (!array && self.declared_arrays.contains(&keys))
        {
            return Err(TomlError);
        }
        let path = if array {
            self.declared_arrays.insert(keys.clone());
            let index = self.array_counts.entry(keys.clone()).or_insert(0);
            let current = *index;
            *index = index.checked_add(1).ok_or(TomlError)?;
            let mut path = keys
                .iter()
                .cloned()
                .map(PathSegment::Key)
                .collect::<Vec<_>>();
            path.push(PathSegment::Index(current));
            path
        } else {
            if !self.declared_tables.insert(keys.clone()) {
                return Err(TomlError);
            }
            keys.iter().cloned().map(PathSegment::Key).collect()
        };
        self.current_table = path.clone();
        self.headers.push(Header { path, array });
        Ok(())
    }

    fn parse_key_value(&mut self) -> Result<(), TomlError> {
        let keys = self.parse_key_path()?;
        self.skip_inline_space();
        self.expect(b'=')?;
        self.skip_inline_space();
        let value = self.parse_value(0)?;
        let mut path = self.current_table.clone();
        let key_count = keys.len();
        for (index, key) in keys.into_iter().enumerate() {
            path.push(PathSegment::Key(key));
            if index + 1 < key_count {
                self.sealed_dotted_tables.insert(path.clone());
            }
        }
        if self
            .seen_values
            .iter()
            .any(|existing| paths_conflict(existing, &path))
            || !self.seen_values.insert(path.clone())
        {
            return Err(TomlError);
        }
        self.entries.push(Entry { path, value });
        Ok(())
    }

    fn parse_key_path(&mut self) -> Result<Vec<String>, TomlError> {
        let mut keys = vec![self.parse_key()?];
        loop {
            self.skip_inline_space();
            if !self.consume(b'.') {
                break;
            }
            self.skip_inline_space();
            keys.push(self.parse_key()?);
        }
        Ok(keys)
    }

    fn parse_key(&mut self) -> Result<String, TomlError> {
        match self.peek() {
            Some(b'"') => self.parse_basic_string(false),
            Some(b'\'') => self.parse_literal_string(false),
            Some(byte) if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') => {
                let start = self.position;
                while self
                    .peek()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
                {
                    self.position += 1;
                }
                Ok(std::str::from_utf8(&self.bytes[start..self.position])
                    .map_err(|_| TomlError)?
                    .to_owned())
            }
            _ => Err(TomlError),
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Value, TomlError> {
        if depth >= TOML_NESTING_MAX {
            return Err(TomlError);
        }
        match self.peek() {
            Some(b'"') if self.starts_with(b"\"\"\"") => {
                self.parse_basic_string(true).map(Value::String)
            }
            Some(b'\'') if self.starts_with(b"'''") => {
                self.parse_literal_string(true).map(Value::String)
            }
            Some(b'"') => self.parse_basic_string(false).map(Value::String),
            Some(b'\'') => self.parse_literal_string(false).map(Value::String),
            Some(b'[') => self.parse_array(depth + 1),
            Some(b'{') => self.parse_inline_table(depth + 1),
            Some(_) => self.parse_bare_value(),
            None => Err(TomlError),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, TomlError> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        self.skip_value_trivia()?;
        if self.consume(b']') {
            return Ok(Value::Array(values));
        }
        loop {
            values.push(self.parse_value(depth)?);
            self.skip_value_trivia()?;
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
            self.skip_value_trivia()?;
            if self.consume(b']') {
                break;
            }
        }
        Ok(Value::Array(values))
    }

    fn parse_inline_table(&mut self, depth: usize) -> Result<Value, TomlError> {
        self.expect(b'{')?;
        self.skip_inline_space();
        let mut entries = Vec::new();
        let mut seen = BTreeSet::<Vec<String>>::new();
        if self.consume(b'}') {
            return Ok(Value::InlineTable(entries));
        }
        loop {
            let key = self.parse_key_path()?;
            if seen
                .iter()
                .any(|existing| string_paths_conflict(existing, &key))
                || !seen.insert(key.clone())
            {
                return Err(TomlError);
            }
            self.skip_inline_space();
            self.expect(b'=')?;
            self.skip_inline_space();
            let value = self.parse_value(depth)?;
            entries.push((key, value));
            self.skip_inline_space();
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
            self.skip_inline_space();
            if self.peek() == Some(b'}') {
                return Err(TomlError);
            }
        }
        Ok(Value::InlineTable(entries))
    }

    fn parse_bare_value(&mut self) -> Result<Value, TomlError> {
        let start = self.position;
        while self.peek().is_some_and(|byte| {
            !byte.is_ascii_whitespace() && !matches!(byte, b'#' | b',' | b']' | b'}')
        }) {
            self.position += 1;
        }
        let first_end = self.position;
        let first = std::str::from_utf8(&self.bytes[start..first_end]).map_err(|_| TomlError)?;
        if valid_date(first)
            && self.peek() == Some(b' ')
            && self
                .bytes
                .get(self.position + 1)
                .is_some_and(|byte| byte.is_ascii_digit())
        {
            self.position += 1;
            while self.peek().is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b'#' | b',' | b']' | b'}')
            }) {
                self.position += 1;
            }
        }
        let value =
            std::str::from_utf8(&self.bytes[start..self.position]).map_err(|_| TomlError)?;
        match value {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ if valid_toml_bare_value(value) => Ok(Value::Bare(value.to_owned())),
            _ => Err(TomlError),
        }
    }

    fn parse_basic_string(&mut self, multiline: bool) -> Result<String, TomlError> {
        if multiline {
            self.expect_bytes(b"\"\"\"")?;
        } else {
            self.expect(b'"')?;
        }
        if multiline {
            self.consume_line_ending();
        }
        let mut output = String::new();
        let mut segment_start = self.position;
        loop {
            if self.position >= self.bytes.len() {
                return Err(TomlError);
            }
            if multiline && self.peek() == Some(b'"') {
                let quote_count = self.bytes[self.position..]
                    .iter()
                    .take_while(|byte| **byte == b'"')
                    .count();
                if quote_count >= 3 {
                    if quote_count > 5 {
                        return Err(TomlError);
                    }
                    self.push_utf8_segment(&mut output, segment_start, self.position)?;
                    for _ in 0..quote_count - 3 {
                        output.push('"');
                    }
                    self.position += quote_count;
                    return Ok(output);
                }
            }
            let byte = self.bytes[self.position];
            if !multiline && byte == b'"' {
                self.push_utf8_segment(&mut output, segment_start, self.position)?;
                self.position += 1;
                return Ok(output);
            }
            if byte == b'\\' {
                self.push_utf8_segment(&mut output, segment_start, self.position)?;
                self.position += 1;
                if multiline && self.consume_line_ending() {
                    self.skip_multiline_escape_whitespace()?;
                    segment_start = self.position;
                    continue;
                }
                let escaped = self.peek().ok_or(TomlError)?;
                self.position += 1;
                match escaped {
                    b'b' => output.push('\u{0008}'),
                    b't' => output.push('\t'),
                    b'n' => output.push('\n'),
                    b'f' => output.push('\u{000c}'),
                    b'r' => output.push('\r'),
                    b'"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'u' => output.push(self.parse_unicode_escape(4)?),
                    b'U' => output.push(self.parse_unicode_escape(8)?),
                    _ => return Err(TomlError),
                }
                segment_start = self.position;
            } else {
                if (!multiline && matches!(byte, b'\n' | b'\r'))
                    || (byte == b'\r' && !self.starts_with(b"\r\n"))
                {
                    return Err(TomlError);
                }
                self.position += 1;
            }
        }
    }

    fn parse_literal_string(&mut self, multiline: bool) -> Result<String, TomlError> {
        if multiline {
            self.expect_bytes(b"'''")?;
        } else {
            self.expect(b'\'')?;
        }
        if multiline {
            self.consume_line_ending();
        }
        let start = self.position;
        loop {
            if self.position >= self.bytes.len() {
                return Err(TomlError);
            }
            if multiline && self.peek() == Some(b'\'') {
                let quote_count = self.bytes[self.position..]
                    .iter()
                    .take_while(|byte| **byte == b'\'')
                    .count();
                if quote_count >= 3 {
                    if quote_count > 5 {
                        return Err(TomlError);
                    }
                    let mut value = std::str::from_utf8(&self.bytes[start..self.position])
                        .map_err(|_| TomlError)?
                        .to_owned();
                    for _ in 0..quote_count - 3 {
                        value.push('\'');
                    }
                    self.position += quote_count;
                    return Ok(value);
                }
            }
            let byte = self.bytes[self.position];
            if !multiline && byte == b'\'' {
                let value = std::str::from_utf8(&self.bytes[start..self.position])
                    .map_err(|_| TomlError)?
                    .to_owned();
                self.position += 1;
                return Ok(value);
            }
            if (!multiline && matches!(byte, b'\n' | b'\r'))
                || (byte == b'\r' && !self.starts_with(b"\r\n"))
            {
                return Err(TomlError);
            }
            self.position += 1;
        }
    }

    fn parse_unicode_escape(&mut self, digits: usize) -> Result<char, TomlError> {
        let mut value = 0_u32;
        for _ in 0..digits {
            let digit = self
                .peek()
                .and_then(|byte| (byte as char).to_digit(16))
                .ok_or(TomlError)?;
            self.position += 1;
            value = value
                .checked_mul(16)
                .and_then(|value| value.checked_add(digit))
                .ok_or(TomlError)?;
        }
        char::from_u32(value).ok_or(TomlError)
    }

    fn push_utf8_segment(
        &self,
        output: &mut String,
        start: usize,
        end: usize,
    ) -> Result<(), TomlError> {
        output.push_str(std::str::from_utf8(&self.bytes[start..end]).map_err(|_| TomlError)?);
        Ok(())
    }

    fn finish_statement(&mut self) -> Result<(), TomlError> {
        self.skip_inline_space();
        if self.consume(b'#') {
            while self
                .peek()
                .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
            {
                self.position += 1;
            }
        }
        if self.position == self.bytes.len() {
            return Ok(());
        }
        if self.consume_line_ending() {
            Ok(())
        } else {
            Err(TomlError)
        }
    }

    fn skip_document_trivia(&mut self) -> Result<(), TomlError> {
        loop {
            self.skip_inline_space();
            if self.consume(b'#') {
                while self
                    .peek()
                    .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
                {
                    self.position += 1;
                }
            }
            if !self.consume_line_ending() {
                break;
            }
        }
        if self.peek() == Some(b'\r') {
            return Err(TomlError);
        }
        Ok(())
    }

    fn skip_value_trivia(&mut self) -> Result<(), TomlError> {
        loop {
            loop {
                match self.peek() {
                    Some(b' ' | b'\t') => self.position += 1,
                    Some(b'\n') => self.position += 1,
                    Some(b'\r') if self.starts_with(b"\r\n") => self.position += 2,
                    Some(b'\r') => return Err(TomlError),
                    _ => break,
                }
            }
            if !self.consume(b'#') {
                return Ok(());
            }
            while self
                .peek()
                .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
            {
                self.position += 1;
            }
        }
    }

    fn skip_multiline_escape_whitespace(&mut self) -> Result<(), TomlError> {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n') => self.position += 1,
                Some(b'\r') if self.starts_with(b"\r\n") => self.position += 2,
                Some(b'\r') => return Err(TomlError),
                _ => return Ok(()),
            }
        }
    }

    fn skip_inline_space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.position += 1;
        }
    }

    fn consume_line_ending(&mut self) -> bool {
        if self.starts_with(b"\r\n") {
            self.position += 2;
            true
        } else {
            self.consume(b'\n')
        }
    }

    fn starts_with(&self, value: &[u8]) -> bool {
        self.bytes[self.position..].starts_with(value)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), TomlError> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(TomlError)
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), TomlError> {
        if self.starts_with(expected) {
            self.position += expected.len();
            Ok(())
        } else {
            Err(TomlError)
        }
    }
}

fn paths_conflict(left: &[PathSegment], right: &[PathSegment]) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn string_paths_conflict(left: &[String], right: &[String]) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn valid_toml_bare_value(value: &str) -> bool {
    matches!(value, "inf" | "+inf" | "-inf" | "nan" | "+nan" | "-nan")
        || valid_toml_integer(value)
        || valid_toml_float(value)
        || valid_toml_datetime(value)
}

fn valid_toml_integer(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if unsigned.len() != value.len() {
        return valid_decimal_groups(unsigned) && value.replace('_', "").parse::<i64>().is_ok();
    }
    if let Some(digits) = unsigned.strip_prefix("0x") {
        valid_based_integer(digits, 16, |byte| byte.is_ascii_hexdigit())
    } else if let Some(digits) = unsigned.strip_prefix("0o") {
        valid_based_integer(digits, 8, |byte| matches!(byte, b'0'..=b'7'))
    } else if let Some(digits) = unsigned.strip_prefix("0b") {
        valid_based_integer(digits, 2, |byte| matches!(byte, b'0' | b'1'))
    } else {
        valid_decimal_groups(unsigned) && unsigned.replace('_', "").parse::<i64>().is_ok()
    }
}

fn valid_based_integer(value: &str, radix: u32, valid_digit: impl Fn(u8) -> bool) -> bool {
    valid_digit_groups(value, valid_digit)
        && u64::from_str_radix(&value.replace('_', ""), radix)
            .is_ok_and(|value| value <= i64::MAX as u64)
}

fn valid_toml_float(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(index) => (&unsigned[..index], Some(&unsigned[index + 1..])),
        None => (unsigned, None),
    };
    if exponent.is_some_and(|value| {
        let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
        !valid_digit_groups(digits, |byte| byte.is_ascii_digit())
    }) {
        return false;
    }
    if let Some((integer, fraction)) = mantissa.split_once('.') {
        valid_decimal_groups(integer) && valid_digit_groups(fraction, |byte| byte.is_ascii_digit())
    } else {
        exponent.is_some() && valid_decimal_groups(mantissa)
    }
}

fn valid_decimal_groups(value: &str) -> bool {
    valid_digit_groups(value, |byte| byte.is_ascii_digit())
        && (value == "0" || (!value.starts_with('0') && !value.starts_with("0_")))
}

fn valid_digit_groups(value: &str, valid_digit: impl Fn(u8) -> bool) -> bool {
    !value.is_empty()
        && !value.starts_with('_')
        && !value.ends_with('_')
        && !value.contains("__")
        && value.bytes().all(|byte| byte == b'_' || valid_digit(byte))
}

fn valid_toml_datetime(value: &str) -> bool {
    if value.len() == 10 {
        return valid_date(value);
    }
    if value.len() >= 8 && value.as_bytes().get(2) == Some(&b':') {
        return valid_time(value);
    }
    if value.len() < 19 || !valid_date(&value[..10]) {
        return false;
    }
    let separator = value.as_bytes()[10];
    if !matches!(separator, b'T' | b't' | b' ') {
        return false;
    }
    let remainder = &value[11..];
    if remainder.ends_with(['Z', 'z']) {
        return valid_time(&remainder[..remainder.len() - 1]);
    }
    if let Some(offset) = remainder
        .char_indices()
        .skip(8)
        .find(|(_, character)| matches!(character, '+' | '-'))
        .map(|(index, _)| index)
    {
        let time = &remainder[..offset];
        let zone = &remainder[offset + 1..];
        return valid_time(time) && valid_hour_minute(zone);
    }
    valid_time(remainder)
}

fn valid_date(value: &str) -> bool {
    if value.len() != 10 || value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' {
        return false;
    }
    let Some(year) = decimal(&value[..4]) else {
        return false;
    };
    let Some(month) = decimal(&value[5..7]) else {
        return false;
    };
    let Some(day) = decimal(&value[8..10]) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximum).contains(&day)
}

fn valid_time(value: &str) -> bool {
    let (whole, fraction) = value
        .split_once('.')
        .map_or((value, None), |(whole, fraction)| (whole, Some(fraction)));
    if whole.len() != 8 || whole.as_bytes()[2] != b':' || whole.as_bytes()[5] != b':' {
        return false;
    }
    if !valid_hour_minute(&whole[..5]) {
        return false;
    }
    let Some(second) = decimal(&whole[6..8]) else {
        return false;
    };
    second <= 59
        && fraction.is_none_or(|digits| {
            !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn valid_hour_minute(value: &str) -> bool {
    if value.len() != 5 || value.as_bytes()[2] != b':' {
        return false;
    }
    matches!((decimal(&value[..2]), decimal(&value[3..])), (Some(hour), Some(minute)) if hour <= 23 && minute <= 59)
}

fn decimal(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_handles_descriptive_multiline_and_unicode_escape() {
        let document = Document::parse(
            b"[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n\
              description=\"\"\"\n[workspace] is text\n\"\"\"\n\
              [lib]\npath=\"\\u006cibrary/root.rs\"\n",
        )
        .unwrap();
        let path = document
            .entries
            .iter()
            .find(|entry| entry.key_names() == ["lib", "path"])
            .and_then(|entry| entry.value.string())
            .unwrap();
        assert_eq!(path, "library/root.rs");
    }

    #[test]
    fn semver_is_canonical_and_complete() {
        for value in ["0.1.0", "1.2.3-alpha.1+build.5", "10.20.30-rc-1"] {
            assert!(valid_semver(value), "{value}");
        }
        for value in ["1", "1.2", "01.2.3", "1.2.3-01", "v1.2.3", "1.2.3+"] {
            assert!(!valid_semver(value), "{value}");
        }
    }

    #[test]
    fn duplicate_and_unterminated_toml_reject_at_parse() {
        assert!(Document::parse(b"a=1\na=2\n").is_err());
        assert!(Document::parse(b"a={}\n[a]\n").is_err());
        assert!(Document::parse(b"a=1\na.b=2\n").is_err());
        assert!(Document::parse(b"a.b=1\n[a]\n").is_err());
        assert!(Document::parse(b"a={x=1,x.y=2}\n").is_err());
        assert!(Document::parse(b"a=\"unterminated\n").is_err());
        assert!(Document::parse(b"a=1not-a-value\n").is_err());
        assert!(Document::parse(b"a=1979-13-01\n").is_err());
        assert!(Document::parse(b"[a\n").is_err());
    }

    #[test]
    fn an_implicit_header_parent_can_be_defined_later() {
        assert!(Document::parse(b"[a.b]\nvalue=1\n[a]\nother=2\n").is_ok());
    }

    #[test]
    fn all_toml_datetime_transport_forms_parse_before_shape_validation() {
        for value in [
            "1979-05-27",
            "07:32:00",
            "1979-05-27T07:32:00Z",
            "1979-05-27 07:32:00-07:00",
        ] {
            let document = Document::parse(format!("value={value}\n").as_bytes()).unwrap();
            assert_eq!(document.entries[0].value.bare(), Some(value));
        }
    }

    #[test]
    fn multiline_strings_accept_one_or_two_quotes_before_the_close() {
        let document = Document::parse(b"a=\"\"\"one \"\"\"\"\"\n").unwrap();
        assert_eq!(document.entries[0].value.string(), Some("one \"\""));
        let document = Document::parse(b"a='''one '''''\n").unwrap();
        assert_eq!(document.entries[0].value.string(), Some("one ''"));
    }
}
