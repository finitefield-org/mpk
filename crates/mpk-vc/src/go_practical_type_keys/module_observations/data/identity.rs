//! Canonical module path/version checks used by original Go sums/ZIPs.
use super::*;

pub(super) fn windows_name(s: &str) -> bool {
    let short = s.split('.').next().unwrap();
    [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .any(|n| short.eq_ignore_ascii_case(n))
}
fn number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'))
}
fn version(s: &str) -> Option<&str> {
    let s = s.strip_prefix('v')?;
    let s = if let Some((s, build)) = s.split_once('+') {
        if build != "incompatible" {
            return None;
        }
        s
    } else {
        s
    };
    let core = if let Some((core, pre)) = s.split_once('-') {
        if !pre.split('.').all(|s| {
            !s.is_empty()
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && (!s.bytes().all(|b| b.is_ascii_digit()) || number(s))
        }) {
            return None;
        }
        core
    } else {
        s
    };
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || !parts.iter().all(|s| number(s)) {
        return None;
    }
    Some(parts[0])
}
fn major(path: &str) -> Option<Option<&str>> {
    if path.starts_with("gopkg.in/") {
        let plain = path.strip_suffix("-unstable").unwrap_or(path);
        let n = plain.bytes().rev().take_while(u8::is_ascii_digit).count();
        if n == 0 {
            return None;
        }
        let at = plain.len() - n;
        if !plain[..at].ends_with(".v")
            || (plain.as_bytes()[at] == b'0' && (n > 1 || plain.len() != path.len()))
        {
            return None;
        }
        return Some(Some(&plain[at..]));
    }
    let tail = path.rsplit('/').next()?;
    let Some(n) = tail
        .strip_prefix('v')
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit() || b == b'.'))
    else {
        return Some(None);
    };
    if !path.contains('/') || n.starts_with('0') || n == "1" || n.contains('.') {
        return None;
    }
    Some(Some(n))
}
pub(super) fn valid(path: &str, ver: &str) -> bool {
    if path.is_empty()
        || path.starts_with('-')
        || path.split('/').any(|s| {
            s.is_empty()
                || s.starts_with('.')
                || s.ends_with('.')
                || windows_name(s)
                || s.bytes()
                    .any(|b| !b.is_ascii_alphanumeric() && !b"-._~".contains(&b))
                || s.split('.')
                    .next()
                    .unwrap()
                    .rsplit_once('~')
                    .is_some_and(|(_, tail)| {
                        !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit())
                    })
        })
    {
        return false;
    }
    let domain = path.split('/').next().unwrap();
    if !domain.contains('.')
        || !domain
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    {
        return false;
    }
    let (Some(want), Some(found)) = (major(path), version(ver)) else {
        return false;
    };
    match want {
        None => matches!(found, "0" | "1") || ver.ends_with("+incompatible"),
        Some(want) => {
            want == found
                || path.starts_with("gopkg.in/") && want == "1" && ver.starts_with("v0.0.0-")
        }
    }
}
pub(super) fn check(path: &str, ver: &str) -> DataResult<()> {
    need(valid(path, ver), D::Identity)
}
pub(super) fn file_path(path: &str) -> bool {
    !path.is_empty()
        && path.split('/').all(|s| {
            !s.is_empty()
                && !s.bytes().all(|b| b == b'.')
                && !s.ends_with('.')
                && !windows_name(s)
                && s.chars().all(|c| {
                    if c.is_ascii() {
                        c.is_ascii_alphanumeric() || "!#$%&()+,-.=@[]^_{}~ ".contains(c)
                    } else {
                        validate::unicode::letter(c)
                    }
                })
        })
}
