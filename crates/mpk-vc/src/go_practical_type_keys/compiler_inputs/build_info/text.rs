// Copyright 2018 The Go Authors. All rights reserved.
// BSD license: ../GO-BUILDINFO-LICENSE; see the repository NOTICE.
use super::{CompilerResult, GoExecutableBuildInfo, GoExecutableModule, C};

fn module(raw: &[u8], replacement: bool) -> CompilerResult<GoExecutableModule> {
    let cols = raw.split(|&b| b == b'\t').collect::<Vec<_>>();
    if !(cols.len() == 3 || cols.len() == 2 && !replacement) {
        return Err(C::BuildInfo);
    }
    Ok(GoExecutableModule {
        path: cols[0].to_vec(),
        version: cols[1].to_vec(),
        sum: cols.get(2).copied().unwrap_or_default().to_vec(),
        replacement: None,
    })
}
fn quote_needed(raw: &[u8]) -> bool {
    raw.iter().any(|b| b" \t\r\n\"`".contains(b))
}
// strconv.QuotedPrefix/Unquote accept raw arbitrary bytes in backticks, decode
// interpreted escapes, and replace each invalid unescaped UTF-8 byte with U+FFFD.
// Source-lexer unquote cannot be used without its source-lexer preconditions.
fn quoted(raw: &[u8]) -> CompilerResult<(Vec<u8>, usize)> {
    let quote = *raw.first().ok_or(C::BuildInfo)?;
    if quote == b'`' {
        let end = raw[1..]
            .iter()
            .position(|&b| b == b'`')
            .ok_or(C::BuildInfo)?
            + 1;
        return Ok((
            raw[1..end]
                .iter()
                .copied()
                .filter(|&b| b != b'\r')
                .collect(),
            end + 1,
        ));
    }
    if quote != b'"' {
        return Err(C::BuildInfo);
    }
    let mut out = vec![];
    let mut at = 1;
    loop {
        let b = *raw.get(at).ok_or(C::BuildInfo)?;
        at += 1;
        match b {
            b'"' => return Ok((out, at)),
            b'\n' => return Err(C::BuildInfo),
            b'\\' => {
                let escape = *raw.get(at).ok_or(C::BuildInfo)?;
                at += 1;
                if let Some(direct) = match escape {
                    b'a' => Some(7),
                    b'b' => Some(8),
                    b'f' => Some(12),
                    b'n' => Some(10),
                    b'r' => Some(13),
                    b't' => Some(9),
                    b'v' => Some(11),
                    b'\\' => Some(b'\\'),
                    b'"' => Some(b'"'),
                    _ => None,
                } {
                    out.push(direct);
                    continue;
                }
                let (base, count, mut value, unicode) = match escape {
                    b'0'..=b'7' => (8, 2, (escape - b'0') as u32, false),
                    b'x' => (16, 2, 0, false),
                    b'u' => (16, 4, 0, true),
                    b'U' => (16, 8, 0, true),
                    _ => return Err(C::BuildInfo),
                };
                for _ in 0..count {
                    let digit = (*raw.get(at).ok_or(C::BuildInfo)? as char)
                        .to_digit(base)
                        .ok_or(C::BuildInfo)?;
                    at += 1;
                    value = value
                        .checked_mul(base)
                        .and_then(|v| v.checked_add(digit))
                        .ok_or(C::BuildInfo)?;
                }
                if unicode {
                    let ch = char::from_u32(value).ok_or(C::BuildInfo)?;
                    out.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                } else {
                    out.push(u8::try_from(value).map_err(|_| C::BuildInfo)?);
                }
            }
            0..=127 => out.push(b),
            _ => {
                let tail = &raw[at - 1..];
                let valid = match std::str::from_utf8(tail) {
                    Ok(s) => s,
                    Err(e) => {
                        std::str::from_utf8(&tail[..e.valid_up_to()]).map_err(|_| C::BuildInfo)?
                    }
                };
                if let Some(ch) = valid.chars().next() {
                    out.extend_from_slice(&tail[..ch.len_utf8()]);
                    at += ch.len_utf8() - 1;
                } else {
                    out.extend_from_slice("\u{fffd}".as_bytes());
                }
            }
        }
    }
}
fn setting(raw: &[u8]) -> CompilerResult<(Vec<u8>, Vec<u8>)> {
    let first = *raw.first().ok_or(C::BuildInfo)?;
    if first == b'=' {
        return Err(C::BuildInfo);
    }
    let (key, value) = if matches!(first, b'`' | b'"') {
        let (key, end) = quoted(raw)?;
        if raw.get(end) != Some(&b'=') {
            return Err(C::BuildInfo);
        }
        (key, &raw[end + 1..])
    } else {
        let end = raw.iter().position(|&b| b == b'=').ok_or(C::BuildInfo)?;
        let key = &raw[..end];
        if key.is_empty() || quote_needed(key) {
            return Err(C::BuildInfo);
        }
        (key.to_vec(), &raw[end + 1..])
    };
    let value = if matches!(value.first(), Some(b'`' | b'"')) {
        let (out, end) = quoted(value)?;
        if end != value.len() {
            return Err(C::BuildInfo);
        }
        out
    } else {
        if quote_needed(value) {
            return Err(C::BuildInfo);
        }
        value.to_vec()
    };
    Ok((key, value))
}
pub(super) fn parse(raw: &[u8]) -> CompilerResult<GoExecutableBuildInfo> {
    let mut info = GoExecutableBuildInfo::default();
    // None means no preceding mod/dep eligible for a replacement. Some(None)
    // selects main; Some(Some(i)) selects a dependency, even across unknown lines.
    let mut last = None;
    for line in raw.split_inclusive(|&b| b == b'\n') {
        let Some(line) = line.strip_suffix(b"\n") else {
            break;
        };
        if let Some(value) = line.strip_prefix(b"path\t") {
            info.path = value.to_vec();
        } else if let Some(value) = line.strip_prefix(b"mod\t") {
            info.main = module(value, false)?;
            last = Some(None);
        } else if let Some(value) = line.strip_prefix(b"dep\t") {
            info.dependencies.push(module(value, false)?);
            last = Some(Some(info.dependencies.len() - 1));
        } else if let Some(value) = line.strip_prefix(b"=>\t") {
            let replacement = module(value, true)?;
            let target = match last.take().ok_or(C::BuildInfo)? {
                None => &mut info.main,
                Some(index) => &mut info.dependencies[index],
            };
            target.replacement = Some(Box::new(replacement));
        } else if let Some(value) = line.strip_prefix(b"build\t") {
            info.settings.push(setting(value)?);
        }
    }
    Ok(info)
}
