//! Native go/build's byte comment walk, independently applied to whole originals.
//! Malformed argument lines are ignored as in go/build; compiler enforcement
//! of pragma placement/variables remains part of original compilation authority.
use super::decode;

fn space(r: u32) -> bool {
    matches!(r, 0x09..=0x0d | 0x20 | 0x85 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000)
}
// strconv.Unquote in the native build reader decodes invalid literal UTF-8
// one byte at a time to RuneError, while escaped bytes retain their byte value.
fn quoted(raw: &[u8]) -> Option<Vec<u8>> {
    let end = raw.len().checked_sub(1)?;
    if raw.first() != Some(&b'"') || raw.get(end) != Some(&b'"') {
        return None;
    }
    let mut at = 1;
    let mut result = vec![];
    while at < end {
        let b = raw[at];
        at += 1;
        if b != b'\\' {
            if matches!(b, b'\n' | b'"') {
                return None;
            }
            if b < 0x80 {
                result.push(b);
            } else {
                let (r, n) = decode(&raw[at - 1..end]);
                let mut buffer = [0; 4];
                result.extend_from_slice(char::from_u32(r)?.encode_utf8(&mut buffer).as_bytes());
                at += n - 1;
            }
            continue;
        }
        let escape = *raw.get(at).filter(|_| at < end)?;
        at += 1;
        let simple = match escape {
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
        };
        if let Some(b) = simple {
            result.push(b);
            continue;
        }
        let (base, count, mut value, unicode) = match escape {
            b'0'..=b'7' => (8, 2, u32::from(escape - b'0'), false),
            b'x' => (16, 2, 0, false),
            b'u' => (16, 4, 0, true),
            b'U' => (16, 8, 0, true),
            _ => return None,
        };
        for _ in 0..count {
            if at == end {
                return None;
            }
            value = value
                .checked_mul(base)?
                .checked_add((raw[at] as char).to_digit(base)?)?;
            at += 1;
        }
        if unicode {
            let mut buffer = [0; 4];
            result.extend_from_slice(char::from_u32(value)?.encode_utf8(&mut buffer).as_bytes());
        } else {
            result.push(u8::try_from(value).ok()?);
        }
    }
    Some(result)
}
fn args(raw: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut at = 0;
    let mut result = vec![];
    loop {
        while at < raw.len() {
            let (r, n) = decode(&raw[at..]);
            if !space(r) {
                break;
            }
            at += n;
        }
        if at == raw.len() {
            return Some(result);
        }
        let start = at;
        let value = match raw[at] {
            b'`' => {
                at += 1;
                let end = at + raw[at..].iter().position(|b| *b == b'`')?;
                let value = raw[at..end].to_vec();
                at = end + 1;
                value
            }
            b'"' => {
                at += 1;
                loop {
                    match raw.get(at)? {
                        b'"' => {
                            at += 1;
                            break;
                        }
                        b'\\' => at += 2,
                        _ => at += 1,
                    }
                }
                quoted(&raw[start..at])?
            }
            _ => {
                while at < raw.len() {
                    let (r, n) = decode(&raw[at..]);
                    if space(r) {
                        break;
                    }
                    at += n;
                }
                raw[start..at].to_vec()
            }
        };
        if at < raw.len() && !space(decode(&raw[at..]).0) {
            return None;
        }
        result.push(value);
    }
}
pub(super) fn read(raw: &[u8]) -> Vec<Vec<u8>> {
    let mut at = usize::from(raw.starts_with(&[0xef, 0xbb, 0xbf])) * 3;
    let mut line_start = false;
    let mut result = vec![];
    while at < raw.len() && raw[at] != 0 {
        let c = raw[at];
        at += 1;
        match c {
            b'\n' => line_start = true,
            b' ' | b'\t' => {}
            b'"' | b'\'' | b'`' => {
                line_start = false;
                loop {
                    let Some(&next) = raw.get(at) else {
                        return result;
                    };
                    if next == 0 {
                        return result;
                    }
                    at += 1;
                    if next == c {
                        break;
                    }
                    if next == b'\\' && c != b'`' {
                        if raw.get(at).is_none_or(|b| *b == 0) {
                            return result;
                        }
                        at += 1;
                    }
                }
            }
            b'/' if raw.get(at) == Some(&b'*') => {
                at += 1;
                loop {
                    if at == raw.len() || raw[at] == 0 {
                        return result;
                    }
                    if raw[at..].starts_with(b"*/") {
                        at += 2;
                        break;
                    }
                    at += 1;
                }
                line_start = false;
            }
            b'/' if raw.get(at) == Some(&b'/') => {
                at += 1;
                let start = at;
                while at < raw.len() && !matches!(raw[at], b'\n' | 0) {
                    at += 1;
                }
                if line_start
                    && raw[start..at].starts_with(b"go:embed")
                    && matches!(raw.get(start + 8), Some(b' ' | b'\t'))
                {
                    if let Some(patterns) = args(&raw[start + 9..at]) {
                        result.extend(patterns);
                    }
                }
                line_start = true;
            }
            _ => line_start = false,
        }
    }
    result
}
