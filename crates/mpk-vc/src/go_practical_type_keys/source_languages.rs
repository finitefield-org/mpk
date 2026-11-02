//! Source-derived file languages; build selection and capture authority remain
//! separate. Module patches are checked before language-minor normalization.
use super::*;
use std::cmp::Ordering;
pub(super) mod constraint;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceLanguageErrorKind {
    Source,
    Module,
    Version,
    CompilerRequirement,
    BuildConstraint,
    SelectedLanguage,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceLanguageError {
    kind: GoSourceLanguageErrorKind,
    source: String,
    offset: usize,
    version: Option<String>,
}
impl GoSourceLanguageError {
    pub fn kind(&self) -> GoSourceLanguageErrorKind {
        self.kind
    }
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}
impl std::fmt::Display for GoSourceLanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source language error: {self:?}")
    }
}
impl std::error::Error for GoSourceLanguageError {}
type LanguageResult<T> = std::result::Result<T, GoSourceLanguageError>;
fn error(
    kind: GoSourceLanguageErrorKind,
    source: &str,
    offset: usize,
    version: Option<String>,
) -> GoSourceLanguageError {
    GoSourceLanguageError {
        kind,
        source: source.into(),
        offset,
        version,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceFileLanguage {
    package: String,
    source: String,
    module: String,
    module_requirement: Option<String>,
    build_requirement: Option<String>,
    effective: String,
    language: String,
}
impl GoSourceFileLanguage {
    pub fn package_id(&self) -> &str {
        &self.package
    }
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn module_source_object_id(&self) -> &str {
        &self.module
    }
    pub fn module_requirement(&self) -> Option<&str> {
        self.module_requirement.as_deref()
    }
    pub fn build_requirement(&self) -> Option<&str> {
        self.build_requirement.as_deref()
    }
    /// Exact FileVersions spelling: module patch suffixes survive when no
    /// versioned build constraint overrides the module requirement.
    pub fn effective_version(&self) -> &str {
        &self.effective
    }
    pub fn language_version(&self) -> &str {
        &self.language
    }
    pub fn in_practical_range(&self) -> bool {
        crate::go_practical_context::PRACTICAL_EFFECTIVE_LANGUAGES.contains(&self.language.as_str())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceLanguages {
    packages: Vec<GoTypePackageKey>,
    rows: BTreeMap<String, GoSourceFileLanguage>,
}
impl GoSourceLanguages {
    pub fn files(&self) -> impl Iterator<Item = &GoSourceFileLanguage> {
        self.rows.values()
    }
    pub fn source(&self, id: &str) -> Option<&GoSourceFileLanguage> {
        self.rows.get(id)
    }
    /// Check every actual selected/transitive body. Unused older source files
    /// stay in the complete original package and do not acquire body admission.
    pub fn check_selection(&self, selection: &GoSourceSelection) -> LanguageResult<()> {
        if selection.packages() != self.packages.as_slice() {
            return Err(error(GoSourceLanguageErrorKind::Source, "", 0, None));
        }
        for function in selection.checked_body_locators() {
            let row = self.source(function.source_object_id()).ok_or_else(|| {
                error(
                    GoSourceLanguageErrorKind::Source,
                    function.source_object_id(),
                    0,
                    None,
                )
            })?;
            if !row.in_practical_range() {
                let offset = selection
                    .source(function.source_object_id())
                    .and_then(|s| s.nodes().get(function.node()))
                    .map_or(0, |n| n.start());
                return Err(error(
                    GoSourceLanguageErrorKind::SelectedLanguage,
                    &row.source,
                    offset,
                    Some(row.language.clone()),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Version {
    major: String,
    minor: String,
    patch: String,
    kind: String,
    pre: String,
}
fn decimal(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
fn cut_number(s: &str) -> Option<(&str, &str)> {
    let n = s.bytes().take_while(u8::is_ascii_digit).count();
    if n == 0 || n > 1 && s.starts_with('0') {
        None
    } else {
        Some((&s[..n], &s[n..]))
    }
}
fn version(s: &str) -> Option<Version> {
    let (major, rest) = cut_number(s)?;
    let (minor, mut rest) = cut_number(rest.strip_prefix('.')?)?;
    let mut v = Version {
        major: major.into(),
        minor: minor.into(),
        patch: String::new(),
        kind: String::new(),
        pre: String::new(),
    };
    if rest.is_empty() {
        if decimal(minor, "21") == Ordering::Less {
            v.patch = "0".into();
        }
        return Some(v);
    }
    if let Some(patch) = rest.strip_prefix('.') {
        let (n, remaining) = cut_number(patch)?;
        if !remaining.is_empty() {
            return None;
        }
        v.patch = n.into();
        return Some(v);
    }
    let n = rest.bytes().take_while(u8::is_ascii_lowercase).count();
    if n == 0 {
        return None;
    }
    v.kind = rest[..n].into();
    rest = &rest[n..];
    let (pre, remaining) = cut_number(rest)?;
    if !remaining.is_empty() {
        return None;
    }
    v.pre = pre.into();
    Some(v)
}
fn compiler_accepts(v: &Version) -> bool {
    decimal(&v.major, "1")
        .then_with(|| decimal(&v.minor, "25"))
        .then_with(|| decimal(&v.patch, "0"))
        .then_with(|| v.kind.as_str().cmp(""))
        .then_with(|| decimal(&v.pre, ""))
        != Ordering::Greater
}
fn language(v: &Version) -> String {
    if v.major == "1" && v.minor == "0" {
        "go1".into()
    } else {
        format!("go{}.{}", v.major, v.minor)
    }
}

// Locate the unquoted top-level go directive without inventing facts about
// the other original directives. Complete module validation stays separate.
fn module_requirement(source: &GoTypeSourceBuffer) -> LanguageResult<Option<String>> {
    let id = &source.object().id;
    let text = std::str::from_utf8(source.bytes())
        .map_err(|e| error(GoSourceLanguageErrorKind::Module, id, e.valid_up_to(), None))?;
    let mut at = 0;
    let mut block = false;
    let mut line: Vec<(usize, &str)> = vec![];
    let mut result = None;
    let process = |line: &[(usize, &str)],
                   block: &mut bool,
                   result: &mut Option<String>|
     -> LanguageResult<()> {
        if line.is_empty() {
            return Ok(());
        }
        if *block {
            if line.len() == 1 && line[0].1 == ")" {
                *block = false;
            }
            return Ok(());
        }
        if line[0].1 == "go" {
            if line.len() != 2 || result.is_some() {
                return Err(error(
                    GoSourceLanguageErrorKind::Module,
                    id,
                    line[0].0,
                    None,
                ));
            }
            let v = version(line[1].1).ok_or_else(|| {
                error(
                    GoSourceLanguageErrorKind::Version,
                    id,
                    line[1].0,
                    Some(line[1].1.into()),
                )
            })?;
            if v.major == "0" {
                return Err(error(
                    GoSourceLanguageErrorKind::Version,
                    id,
                    line[1].0,
                    Some(line[1].1.into()),
                ));
            }
            if !compiler_accepts(&v) {
                return Err(error(
                    GoSourceLanguageErrorKind::CompilerRequirement,
                    id,
                    line[1].0,
                    Some(line[1].1.into()),
                ));
            }
            *result = Some(format!("go{}", line[1].1));
        } else if line.last().is_some_and(|(_, s)| *s == "(") {
            *block = true;
        }
        Ok(())
    };
    while at < text.len() {
        let c = text.as_bytes()[at];
        if matches!(c, b' ' | b'\t' | b'\r') {
            at += 1;
            continue;
        }
        if c == b'\n' {
            process(&line, &mut block, &mut result)?;
            line.clear();
            at += 1;
            continue;
        }
        if text[at..].starts_with("//") {
            at += text[at..].find('\n').unwrap_or(text.len() - at);
            continue;
        }
        if text[at..].starts_with("/*") {
            return Err(error(GoSourceLanguageErrorKind::Module, id, at, None));
        }
        let start = at;
        if c == b'"' || c == b'`' {
            at += 1;
            let mut closed = false;
            while at < text.len() {
                let b = text.as_bytes()[at];
                at += 1;
                if b == b'\n' {
                    break;
                }
                if b == c {
                    closed = true;
                    break;
                }
                if b == b'\\' && c == b'"' {
                    at = (at + 1).min(text.len());
                }
            }
            if !closed {
                return Err(error(GoSourceLanguageErrorKind::Module, id, start, None));
            }
        } else if matches!(c, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',') {
            at += 1;
        } else {
            while at < text.len() {
                let ch = text[at..].chars().next().unwrap();
                if ch.is_whitespace()
                    || matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | ',')
                    || text[at..].starts_with("//")
                    || text[at..].starts_with("/*")
                {
                    break;
                }
                at += ch.len_utf8();
            }
            if at == start {
                return Err(error(GoSourceLanguageErrorKind::Module, id, at, None));
            }
        }
        line.push((start, &text[start..at]));
    }
    process(&line, &mut block, &mut result)?;
    if block {
        return Err(error(
            GoSourceLanguageErrorKind::Module,
            id,
            text.len(),
            None,
        ));
    }
    Ok(result)
}

fn build_requirement(source: &GoTypeSourceBuffer) -> LanguageResult<Option<String>> {
    let id = &source.object().id;
    let text = std::str::from_utf8(source.bytes())
        .map_err(|e| error(GoSourceLanguageErrorKind::Source, id, e.valid_up_to(), None))?;
    let mut at = usize::from(text.starts_with('\u{feff}')) * 3;
    let mut result = None;
    let mut seen = false;
    loop {
        while at < text.len() && matches!(text.as_bytes()[at], b' ' | b'\t' | b'\r' | b'\n') {
            at += 1;
        }
        if text[at..].starts_with("/*") {
            let end = text[at + 2..]
                .find("*/")
                .ok_or_else(|| error(GoSourceLanguageErrorKind::Source, id, at, None))?;
            at += end + 4;
            continue;
        }
        if !text[at..].starts_with("//") {
            break;
        }
        let end = at + text[at..].find('\n').unwrap_or(text.len() - at);
        let comment = text[at..end].replace('\r', "");
        if let Some(rest) = comment.trim().strip_prefix("//go:build") {
            let trim = rest.trim();
            if rest.is_empty() || trim.len() != rest.len() {
                if seen {
                    return Err(error(
                        GoSourceLanguageErrorKind::BuildConstraint,
                        id,
                        at,
                        None,
                    ));
                }
                seen = true;
                result = constraint::minimum(trim)
                    .map_err(|_| error(GoSourceLanguageErrorKind::BuildConstraint, id, at, None))?
                    .map(|n| {
                        if n == 0 {
                            "go1".into()
                        } else {
                            format!("go1.{n}")
                        }
                    });
            }
        }
        at = end;
    }
    Ok(result)
}

pub fn derive_go_source_languages(inputs: &GoSourceInputs) -> LanguageResult<GoSourceLanguages> {
    let keys = inputs.components();
    let mut modules = BTreeMap::new();
    let mut rows = BTreeMap::new();
    for package in &keys.wire.packages {
        let module = &package.module_source_object_id;
        if !modules.contains_key(module) {
            let raw = keys
                .source(module)
                .ok_or_else(|| error(GoSourceLanguageErrorKind::Source, module, 0, None))?;
            modules.insert(module.clone(), module_requirement(raw)?);
        }
        let requirement = &modules[module];
        let config = requirement.clone().unwrap_or_else(|| "go1.16".into());
        for id in &package.source_objects {
            let raw = keys
                .source(id)
                .ok_or_else(|| error(GoSourceLanguageErrorKind::Source, id, 0, None))?;
            let build = build_requirement(raw)?;
            let effective = if let Some(build) = &build {
                let minor = build.strip_prefix("go1.").unwrap_or("0");
                if decimal(minor, "21") == Ordering::Less {
                    "go1.21".into()
                } else {
                    build.clone()
                }
            } else {
                config.clone()
            };
            let lang = if build.is_some() {
                effective.clone()
            } else {
                language(&version(config.strip_prefix("go").unwrap()).unwrap())
            };
            let row = GoSourceFileLanguage {
                package: package.id.clone(),
                source: id.clone(),
                module: module.clone(),
                module_requirement: requirement.clone(),
                build_requirement: build,
                effective,
                language: lang,
            };
            if rows.insert(id.clone(), row).is_some() {
                return Err(error(GoSourceLanguageErrorKind::Source, id, 0, None));
            }
        }
    }
    Ok(GoSourceLanguages {
        packages: keys.wire.packages.clone(),
        rows,
    })
}
