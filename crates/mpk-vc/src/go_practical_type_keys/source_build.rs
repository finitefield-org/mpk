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
    name.ends_with(".go") && target_filename(name)
}
fn target_filename(name: &str) -> bool {
    if name.starts_with(['_', '.']) {
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
fn trim_space(mut raw: &[u8]) -> &[u8] {
    const SPACE: &[&[u8]] = &[
        b"\xc2\x85",
        b"\xc2\xa0",
        b"\xe1\x9a\x80",
        b"\xe2\x80\x80",
        b"\xe2\x80\x81",
        b"\xe2\x80\x82",
        b"\xe2\x80\x83",
        b"\xe2\x80\x84",
        b"\xe2\x80\x85",
        b"\xe2\x80\x86",
        b"\xe2\x80\x87",
        b"\xe2\x80\x88",
        b"\xe2\x80\x89",
        b"\xe2\x80\x8a",
        b"\xe2\x80\xa8",
        b"\xe2\x80\xa9",
        b"\xe2\x80\xaf",
        b"\xe2\x81\x9f",
        b"\xe3\x80\x80",
    ];
    loop {
        if raw
            .first()
            .is_some_and(|b| matches!(b, b'\t'..=b'\r' | b' '))
        {
            raw = &raw[1..];
        } else if let Some(s) = SPACE.iter().find(|s| raw.starts_with(s)) {
            raw = &raw[s.len()..];
        } else {
            break;
        }
    }
    loop {
        if raw
            .last()
            .is_some_and(|b| matches!(b, b'\t'..=b'\r' | b' '))
        {
            raw = &raw[..raw.len() - 1];
        } else if let Some(s) = SPACE.iter().find(|s| raw.ends_with(s)) {
            raw = &raw[..raw.len() - s.len()];
        } else {
            break;
        }
    }
    raw
}
fn directive<'a>(line: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    let rest = line.strip_prefix(prefix)?;
    let trim = trim_space(rest);
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
    match_source(source, source.bytes(), true)
}

pub(super) fn auxiliary_match(
    source: &GoTypeSourceBuffer,
    header: Option<&[u8]>,
) -> Result<bool, GoSourceBuildError> {
    if source.object().kind != "auxiliary" {
        return Err(error(source, GoSourceBuildErrorKind::Source));
    }
    let name = source.object().path.rsplit('/').next().unwrap();
    if !target_filename(name) {
        return Ok(false);
    }
    // Native matchFile returns a non-nil record on comment-read failure;
    // ImportDir retains non-Go records while ignoring that read error.
    let Some(header) = header else {
        return Ok(true);
    };
    Ok(match_source(source, header, false)?.matches_target())
}

fn match_source(
    source: &GoTypeSourceBuffer,
    raw: &[u8],
    go: bool,
) -> Result<GoSourceBuildMatch, GoSourceBuildError> {
    let name = source.object().path.rsplit('/').next().unwrap();
    let mut result = GoSourceBuildMatch {
        source: source.object().id.clone(),
        filename: if go {
            filename(name)
        } else {
            target_filename(name)
        },
        constraints: true,
        test: go && name.ends_with("_test.go"),
        binary_only: false,
    };
    // Go ignores a filename mismatch before reading its header.
    if !result.filename {
        return Ok(result);
    }
    let text = raw.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(raw);
    let (mut end, mut offset, mut ended, mut block) = (0, 0, false, false);
    let mut modern = None;
    'lines: for raw in text.split_inclusive(|b| *b == b'\n') {
        offset += raw.len();
        let mut line = trim_space(raw);
        if line.is_empty() && !ended {
            end = offset;
            continue;
        }
        if !line.starts_with(b"//") {
            ended = true;
        }
        if !block {
            if let Some(expr) = directive(line, b"//go:build") {
                if modern.replace(expr).is_some() {
                    return Err(error(source, GoSourceBuildErrorKind::Constraint));
                }
            }
            if line == b"//go:binary-only-package" {
                result.binary_only = true;
            }
        }
        while !line.is_empty() {
            if block {
                let Some(at) = line.windows(2).position(|s| s == b"*/") else {
                    continue 'lines;
                };
                block = false;
                line = trim_space(&line[at + 2..]);
            } else if line.starts_with(b"//") {
                continue 'lines;
            } else if let Some(rest) = line.strip_prefix(b"/*") {
                block = true;
                line = trim_space(rest);
            } else {
                break 'lines;
            }
        }
    }
    if let Some(expr) = modern {
        result.constraints = constraint::evaluate(&String::from_utf8_lossy(expr), tag)
            .map_err(|_| error(source, GoSourceBuildErrorKind::Constraint))?;
    } else {
        for line in text[..end].split(|b| *b == b'\n').map(trim_space) {
            if let Some(expr) = line
                .strip_prefix(b"//")
                .and_then(|s| directive(trim_space(s), b"+build"))
            {
                if let Some(value) = legacy(&String::from_utf8_lossy(expr)) {
                    result.constraints &= value;
                }
            }
        }
    }
    // The Go package loader ignores this marker for test files.
    if !go || result.test || !result.matches_target() {
        result.binary_only = false;
    }
    Ok(result)
}
