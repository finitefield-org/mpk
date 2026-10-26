//! Decimal strings preserve the pinned Go version domain without host integer
//! limits. Module shorthands and build metadata retain modfile normalization.
use super::*;
fn number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'))
}
pub(super) fn canonical(raw: &[u8]) -> ManifestResult<String> {
    let raw = std::str::from_utf8(raw).map_err(|_| M::Version)?;
    let s = raw.strip_prefix('v').ok_or(M::Version)?;
    let (s, build) = s.split_once('+').map_or((s, None), |(s, b)| (s, Some(b)));
    if let Some(build) = build {
        need(
            build.split('.').all(|s| {
                !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            }),
            M::Version,
        )?;
    }
    let (core, pre) = s.split_once('-').map_or((s, None), |(s, p)| (s, Some(p)));
    let mut parts = core.split('.').collect::<Vec<_>>();
    need(
        (1..=3).contains(&parts.len()) && parts.iter().all(|s| number(s)),
        M::Version,
    )?;
    need(
        parts.len() == 3 || pre.is_none() && build.is_none(),
        M::Version,
    )?;
    if let Some(pre) = pre {
        need(
            pre.split('.').all(|s| {
                !s.is_empty()
                    && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    && (!s.bytes().all(|b| b.is_ascii_digit()) || number(s))
            }),
            M::Version,
        )?;
    }
    while parts.len() < 3 {
        parts.push("0");
    }
    let mut out = format!("v{}", parts.join("."));
    if let Some(pre) = pre {
        out.push('-');
        out.push_str(pre);
    }
    if build == Some("incompatible") {
        out.push_str("+incompatible");
    }
    Ok(out)
}
fn path_major(path: &[u8]) -> ManifestResult<Option<Vec<u8>>> {
    if path.starts_with(b"gopkg.in/") {
        let plain = path.strip_suffix(b"-unstable").unwrap_or(path);
        let n = plain
            .iter()
            .rev()
            .take_while(|b| b.is_ascii_digit())
            .count();
        need(n > 0, M::Version)?;
        let at = plain.len() - n;
        need(
            at > 1 && plain[..at].ends_with(b".v") && (plain[at] != b'0' || &path[at..] == b"0"),
            M::Version,
        )?;
        return Ok(Some(plain[at..].to_vec()));
    }
    let n = path
        .iter()
        .rev()
        .take_while(|b| b.is_ascii_digit() || **b == b'.')
        .count();
    let at = path.len() - n;
    if n == 0 || at < 2 || &path[at - 2..at] != b"/v" {
        return Ok(None);
    }
    let major = &path[at..];
    need(
        !major.contains(&b'.') && major[0] != b'0' && major != b"1",
        M::Version,
    )?;
    Ok(Some(major.to_vec()))
}
pub(super) fn matches(path: &[u8], version: &str) -> ManifestResult<()> {
    let major = version[1..].split('.').next().unwrap();
    need(
        match path_major(path)? {
            None => matches!(major, "0" | "1") || version.ends_with("+incompatible"),
            Some(want) => {
                want == major.as_bytes()
                    || path.starts_with(b"gopkg.in/")
                        && want == b"1"
                        && version.starts_with("v0.0.0-")
            }
        },
        M::Version,
    )
}
pub(super) fn valid_path_major(path: &[u8]) -> ManifestResult<()> {
    path_major(path).map(|_| ())
}
fn decimal(a: &str, b: &str) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
#[derive(Default, Eq, PartialEq, Ord, PartialOrd)]
struct GoVersion {
    major: String,
    minor: String,
    patch: String,
    kind: String,
    pre: String,
}
fn integer(s: &str) -> Option<(&str, &str)> {
    let n = s.bytes().take_while(u8::is_ascii_digit).count();
    if n == 0 || n > 1 && s.starts_with('0') {
        None
    } else {
        Some((&s[..n], &s[n..]))
    }
}
fn go(s: &str) -> Option<GoVersion> {
    let (major, rest) = integer(s)?;
    let mut v = GoVersion {
        major: major.into(),
        ..Default::default()
    };
    if rest.is_empty() {
        v.minor = "0".into();
        v.patch = "0".into();
        return Some(v);
    }
    let (minor, rest) = integer(rest.strip_prefix('.')?)?;
    v.minor = minor.into();
    if rest.is_empty() {
        if decimal(minor, "21").is_lt() {
            v.patch = "0".into();
        }
        return Some(v);
    }
    if let Some(rest) = rest.strip_prefix('.') {
        let (patch, rest) = integer(rest)?;
        if !rest.is_empty() {
            return None;
        }
        v.patch = patch.into();
        return Some(v);
    }
    let n = rest.bytes().take_while(u8::is_ascii_lowercase).count();
    if n == 0 {
        return None;
    }
    v.kind = rest[..n].into();
    if rest.len() > n {
        let (pre, rest) = integer(&rest[n..])?;
        if !rest.is_empty() {
            return None;
        }
        v.pre = pre.into();
    }
    Some(v)
}
fn accepted(v: &GoVersion) -> bool {
    decimal(&v.major, "1")
        .then_with(|| decimal(&v.minor, "25"))
        .then_with(|| decimal(&v.patch, "0"))
        .then_with(|| v.kind.as_str().cmp(""))
        .then_with(|| decimal(&v.pre, ""))
        != std::cmp::Ordering::Greater
}
pub(super) fn compare_go(a: &str, b: &str) -> std::cmp::Ordering {
    match (go(a), go(b)) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => decimal(&a.major, &b.major)
            .then_with(|| decimal(&a.minor, &b.minor))
            .then_with(|| decimal(&a.patch, &b.patch))
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| decimal(&a.pre, &b.pre)),
    }
}
pub(super) fn language(s: &str, main: bool) -> ManifestResult<String> {
    let v = go(s).ok_or(M::Version)?;
    // modfile's GoVersionRE additionally requires a positive major, explicit
    // minor and digits after any prerelease kind.
    need(
        v.major != "0" && s.contains('.') && (v.kind.is_empty() || !v.pre.is_empty()),
        M::Version,
    )?;
    need(accepted(&v), M::CompilerRequirement)?;
    let lang = if v.major == "1" && v.minor == "0" {
        "go1".into()
    } else {
        format!("go{}.{}", v.major, v.minor)
    };
    need(
        !main || matches!(lang.as_str(), "go1.22" | "go1.23" | "go1.24" | "go1.25"),
        M::CompilerRequirement,
    )?;
    Ok(lang)
}
pub(super) fn toolchain(s: &[u8], main: bool) -> ManifestResult<()> {
    if s == b"default" {
        return Ok(());
    }
    need(s == b"go1" || s.starts_with(b"go1."), M::Version)?;
    let version = s
        .strip_prefix(b"go")
        .ok_or(M::Version)?
        .split(|b| *b == b'-')
        .next()
        .unwrap();
    let s = std::str::from_utf8(version).map_err(|_| M::Version)?;
    let v = go(s).ok_or(M::Version)?;
    need(!main || accepted(&v), M::CompilerRequirement)
}
