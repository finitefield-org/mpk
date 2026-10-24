//! Source-only build constraints for the fixed Go1.25.0 linux/amd64 target.
//! These facts do not establish import/cgo selection or compiler authority.
use super::{source_languages::constraint, validate, GoTypeSourceBuffer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceBuildErrorKind {
    Source,
    Constraint,
    Excluded,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceBuildError {
    kind: GoSourceBuildErrorKind,
    source: String,
}
impl GoSourceBuildError {
    pub fn kind(&self) -> GoSourceBuildErrorKind {
        self.kind
    }
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
}
impl std::fmt::Display for GoSourceBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go target source constraint error: {self:?}")
    }
}
impl std::error::Error for GoSourceBuildError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceBuildMatch {
    source: String,
    filename: bool,
    constraints: bool,
    test: bool,
    binary_only: bool,
}
impl GoSourceBuildMatch {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    /// Fixed target filename and leading constraint checks only.
    pub fn matches_target(&self) -> bool {
        self.filename && self.constraints
    }
    pub fn is_test_file(&self) -> bool {
        self.test
    }
    pub fn is_binary_only(&self) -> bool {
        self.binary_only
    }
    pub fn is_production_candidate(&self) -> bool {
        self.matches_target() && !self.test && !self.binary_only
    }
}
fn error(source: &GoTypeSourceBuffer, kind: GoSourceBuildErrorKind) -> GoSourceBuildError {
    GoSourceBuildError {
        kind,
        source: source.object().id.clone(),
    }
}
pub(super) fn require_production_source(
    source: &GoTypeSourceBuffer,
) -> Result<(), GoSourceBuildError> {
    if derive_go_source_build_match(source)?.is_production_candidate() {
        Ok(())
    } else {
        Err(error(source, GoSourceBuildErrorKind::Excluded))
    }
}

fn tag(name: &str) -> bool {
    matches!(
        name,
        "linux"
            | "amd64"
            | "gc"
            | "unix"
            | "amd64.v1"
            | "goexperiment.regabiwrappers"
            | "goexperiment.regabiargs"
            | "goexperiment.aliastypeparams"
            | "goexperiment.swissmap"
            | "goexperiment.synchashtriemap"
            | "goexperiment.dwarf5"
    ) || name.strip_prefix("go1.").is_some_and(|minor| {
        !minor.starts_with('0') && minor.parse::<u8>().is_ok_and(|n| (1..=25).contains(&n))
    })
}
fn os(name: &str) -> bool {
    matches!(
        name,
        "aix"
            | "android"
            | "darwin"
            | "dragonfly"
            | "freebsd"
            | "hurd"
            | "illumos"
            | "ios"
            | "js"
            | "linux"
            | "nacl"
            | "netbsd"
            | "openbsd"
            | "plan9"
            | "solaris"
            | "wasip1"
            | "windows"
            | "zos"
    )
}
fn arch(name: &str) -> bool {
    matches!(
        name,
        "386"
            | "amd64"
            | "amd64p32"
            | "arm"
            | "armbe"
            | "arm64"
            | "arm64be"
            | "loong64"
            | "mips"
            | "mipsle"
            | "mips64"
            | "mips64le"
            | "mips64p32"
            | "mips64p32le"
            | "ppc"
            | "ppc64"
            | "ppc64le"
            | "riscv"
            | "riscv64"
            | "s390"
            | "s390x"
            | "sparc"
            | "sparc64"
            | "wasm"
    )
}
fn filename(name: &str) -> bool {
    if name.starts_with(['_', '.']) || !name.ends_with(".go") {
        return false;
    }
    let stem = name.split('.').next().unwrap();
    let Some(at) = stem.find('_') else {
        return true;
    };
    let mut parts = stem[at..].split('_').collect::<Vec<_>>();
    if parts.last() == Some(&"test") {
        parts.pop();
    }
    let n = parts.len();
    if n >= 2 && os(parts[n - 2]) && arch(parts[n - 1]) {
        return tag(parts[n - 2]) && tag(parts[n - 1]);
    }
    parts
        .last()
        .is_none_or(|name| !os(name) && !arch(name) || tag(name))
}
fn directive<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?;
    let trim = rest.trim();
    (rest.is_empty() || trim.len() < rest.len()).then_some(trim)
}
fn legacy(text: &str) -> Option<bool> {
    let mut result = false;
    let mut size = 0;
    for (i, clause) in text.split_whitespace().enumerate() {
        let mut all = true;
        for (j, literal) in clause.split(',').enumerate() {
            if j > 0 {
                size += 1;
            }
            if size > 100 {
                return None;
            }
            let value = if literal.starts_with("!!") || literal == "!" {
                false
            } else {
                let (negate, name) = literal
                    .strip_prefix('!')
                    .map_or((false, literal), |s| (true, s));
                let valid = !name.is_empty()
                    && name.chars().all(|c| {
                        validate::unicode::letter(c)
                            || validate::unicode::digit(c)
                            || c == '_'
                            || c == '.'
                    });
                (valid && tag(name)) != negate
            };
            all &= value;
        }
        if i > 0 {
            size += 1;
        }
        if size > 100 {
            return None;
        }
        result |= all;
    }
    Some(result)
}

/// Reconstruct filename and leading-comment constraints from exact original
/// Go bytes. No caller tags, host defaults, version facts or filesystem reads.
/// Import/cgo classification, complete package membership and compiler/context
/// ownership are separate checks; this is a necessary source eligibility guard.
pub fn derive_go_source_build_match(
    source: &GoTypeSourceBuffer,
) -> Result<GoSourceBuildMatch, GoSourceBuildError> {
    if !matches!(source.object().kind.as_str(), "go_file" | "excluded_source") {
        return Err(error(source, GoSourceBuildErrorKind::Source));
    }
    let name = source.object().path.rsplit('/').next().unwrap();
    let mut result = GoSourceBuildMatch {
        source: source.object().id.clone(),
        filename: filename(name),
        constraints: true,
        test: name.ends_with("_test.go"),
        binary_only: false,
    };
    // Go ignores a filename mismatch before reading its header.
    if !result.filename {
        return Ok(result);
    }
    let text = std::str::from_utf8(source.bytes())
        .map_err(|_| error(source, GoSourceBuildErrorKind::Source))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let (mut end, mut offset, mut ended, mut block) = (0, 0, false, false);
    let mut modern = None;
    'lines: for raw in text.split_inclusive('\n') {
        offset += raw.len();
        let mut line = raw.trim();
        if line.is_empty() && !ended {
            end = offset;
            continue;
        }
        if !line.starts_with("//") {
            ended = true;
        }
        if !block {
            if let Some(expr) = directive(line, "//go:build") {
                if modern.replace(expr).is_some() {
                    return Err(error(source, GoSourceBuildErrorKind::Constraint));
                }
            }
            if line == "//go:binary-only-package" {
                result.binary_only = true;
            }
        }
        while !line.is_empty() {
            if block {
                let Some(at) = line.find("*/") else {
                    continue 'lines;
                };
                block = false;
                line = line[at + 2..].trim();
            } else if line.starts_with("//") {
                continue 'lines;
            } else if let Some(rest) = line.strip_prefix("/*") {
                block = true;
                line = rest.trim();
            } else {
                break 'lines;
            }
        }
    }
    if let Some(expr) = modern {
        result.constraints = constraint::evaluate(expr, tag)
            .map_err(|_| error(source, GoSourceBuildErrorKind::Constraint))?;
    } else {
        for line in text[..end].lines().map(str::trim) {
            if let Some(expr) = line
                .strip_prefix("//")
                .and_then(|s| directive(s.trim(), "+build"))
            {
                if let Some(value) = legacy(expr) {
                    result.constraints &= value;
                }
            }
        }
    }
    // The Go package loader ignores this marker for test files.
    if result.test || !result.matches_target() {
        result.binary_only = false;
    }
    Ok(result)
}
