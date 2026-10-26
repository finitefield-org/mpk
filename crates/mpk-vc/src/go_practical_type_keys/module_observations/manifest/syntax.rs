//! Byte-owned modfile syntax, preserving suffix comments and arbitrary bytes
//! in comments/strings. Unicode classification uses the pinned Go15 table.
use super::*;
pub(super) struct Line {
    pub verb: Vec<u8>,
    pub arguments: Vec<Vec<u8>>,
    pub suffix: Vec<u8>,
}
enum Token {
    Word(Vec<u8>),
    Punct(u8),
    Eol(Vec<u8>),
    Comment,
    End,
}
pub(super) fn space(c: char) -> bool {
    matches!(c,'\t'..='\r'|' '| '\u{85}'|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}')
}
pub(super) fn rune(raw: &[u8]) -> (char, usize) {
    let first = raw[0];
    let n = match first {
        0..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return ('\u{fffd}', 1),
    };
    raw.get(..n)
        .and_then(|b| std::str::from_utf8(b).ok())
        .map(|s| (s.chars().next().unwrap(), n))
        .unwrap_or(('\u{fffd}', 1))
}
fn ident(c: char) -> bool {
    !matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ' ')
        && !space(c)
        && source_package::graphic::graphic(c)
}
fn tokens(raw: &[u8]) -> ManifestResult<Vec<Token>> {
    let mut at = 0;
    let mut out = vec![];
    let mut line_nonempty = false;
    while at < raw.len() {
        let b = raw[at];
        if matches!(b, b' ' | b'\t' | b'\r') {
            at += 1;
            continue;
        }
        if raw[at..].starts_with(b"//") {
            let start = at + 2;
            at += raw[at..]
                .iter()
                .position(|b| *b == b'\n')
                .unwrap_or(raw.len() - at);
            if line_nonempty {
                out.push(Token::Eol(raw[start..at].to_vec()));
            } else {
                out.push(Token::Comment);
            }
            if at < raw.len() {
                at += 1;
            }
            line_nonempty = false;
            continue;
        }
        need(!raw[at..].starts_with(b"/*"), M::Syntax)?;
        if b == b'\n' {
            out.push(Token::Eol(vec![]));
            at += 1;
            line_nonempty = false;
            continue;
        }
        line_nonempty = true;
        if b"()[]{},".contains(&b) {
            out.push(Token::Punct(b));
            at += 1;
            continue;
        }
        let start = at;
        if matches!(b, b'"' | b'`') {
            at += 1;
            let mut closed = false;
            while at < raw.len() {
                let c = raw[at];
                need(c != b'\n', M::Syntax)?;
                if c == b {
                    at += 1;
                    closed = true;
                    break;
                }
                if c == b'\\' && b != b'`' {
                    at += 1;
                    need(at < raw.len(), M::Syntax)?;
                }
                // Escaped newlines are consumed by readRune in the native
                // lexer. strconv.Unquote rejects them if used as strings.
                at += rune(&raw[at..]).1;
            }
            need(closed, M::Syntax)?;
        } else {
            while at < raw.len() {
                let (c, n) = rune(&raw[at..]);
                if !ident(c) || raw[at..].starts_with(b"//") {
                    break;
                }
                need(!raw[at..].starts_with(b"/*"), M::Syntax)?;
                at += n;
            }
            need(at > start, M::Syntax)?;
        }
        out.push(Token::Word(raw[start..at].to_vec()));
    }
    out.push(Token::End);
    Ok(out)
}
fn eol(token: &Token) -> bool {
    matches!(token, Token::Eol(_) | Token::End)
}
fn spelling(token: &Token) -> Vec<u8> {
    match token {
        Token::Word(s) => s.clone(),
        Token::Punct(b) => vec![*b],
        _ => vec![],
    }
}
fn known_block(verb: &[u8]) -> bool {
    matches!(
        verb,
        b"module"
            | b"godebug"
            | b"require"
            | b"exclude"
            | b"replace"
            | b"retract"
            | b"tool"
            | b"ignore"
    )
}
pub(super) fn lines(raw: &[u8]) -> ManifestResult<Vec<Line>> {
    let tokens = tokens(raw)?;
    let mut at = 0;
    let mut out = vec![];
    while !matches!(tokens[at], Token::End) {
        if matches!(tokens[at], Token::Eol(_) | Token::Comment) {
            at += 1;
            continue;
        }
        let mut prefix = vec![spelling(&tokens[at])];
        at += 1;
        loop {
            if eol(&tokens[at]) {
                let suffix = if let Token::Eol(s) = &tokens[at] {
                    s.clone()
                } else {
                    vec![]
                };
                out.push(Line {
                    verb: prefix.remove(0),
                    arguments: prefix,
                    suffix,
                });
                if !matches!(tokens[at], Token::End) {
                    at += 1;
                }
                break;
            }
            if matches!(tokens[at], Token::Punct(b'('))
                && (eol(&tokens[at + 1])
                    || matches!(tokens[at + 1], Token::Punct(b')')) && eol(&tokens[at + 2]))
            {
                need(prefix.len() == 1 && known_block(&prefix[0]), M::Directive)?;
                let verb = prefix.remove(0);
                if matches!(tokens[at + 1], Token::Punct(b')')) {
                    at += 2;
                    if !matches!(tokens[at], Token::End) {
                        at += 1;
                    }
                    break;
                }
                at += 1;
                if !matches!(tokens[at], Token::End) {
                    at += 1;
                }
                loop {
                    need(!matches!(tokens[at], Token::End), M::Syntax)?;
                    if matches!(tokens[at], Token::Eol(_) | Token::Comment) {
                        at += 1;
                        continue;
                    }
                    if matches!(tokens[at], Token::Punct(b')')) {
                        at += 1;
                        need(eol(&tokens[at]), M::Syntax)?;
                        if !matches!(tokens[at], Token::End) {
                            at += 1;
                        }
                        break;
                    }
                    let mut arguments = vec![];
                    while !eol(&tokens[at]) {
                        need(!matches!(tokens[at], Token::Comment), M::Syntax)?;
                        arguments.push(spelling(&tokens[at]));
                        at += 1;
                    }
                    let suffix = if let Token::Eol(s) = &tokens[at] {
                        s.clone()
                    } else {
                        vec![]
                    };
                    out.push(Line {
                        verb: verb.clone(),
                        arguments,
                        suffix,
                    });
                    if !matches!(tokens[at], Token::End) {
                        at += 1;
                    }
                }
                break;
            }
            need(!matches!(tokens[at], Token::Comment), M::Syntax)?;
            prefix.push(spelling(&tokens[at]));
            at += 1;
        }
    }
    Ok(out)
}
