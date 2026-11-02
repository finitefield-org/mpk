//! Physical Go source tokens, independently computed from owned original bytes.
//! Comments and line directives never redirect reads or physical locations.
//! Lexical success is not Go parsing, typing or program admission.
use super::{validate::unicode, GoTypeSourceBuffer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoLexErrorKind {
    Source,
    Utf8,
    Nul,
    ByteOrderMark,
    Character,
    Number,
    Escape,
    Literal,
    Comment,
    Directive,
    Limit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoLexError {
    pub offset: usize,
    pub kind: GoLexErrorKind,
}
impl std::fmt::Display for GoLexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Go lexical error {:?} at byte {}",
            self.kind, self.offset
        )
    }
}
impl std::error::Error for GoLexError {}
type Result<T> = std::result::Result<T, GoLexError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoLexToken {
    start: usize,
    end: usize,
    kind: &'static str,
    literal: Vec<u8>,
    inserted: bool,
}
impl GoLexToken {
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn kind(&self) -> &str {
        self.kind
    }
    pub fn literal(&self) -> &[u8] {
        &self.literal
    }
    pub fn inserted(&self) -> bool {
        self.inserted
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoLexedSource {
    source: GoTypeSourceBuffer,
    tokens: Vec<GoLexToken>,
}
impl GoLexedSource {
    pub fn source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn tokens(&self) -> &[GoLexToken] {
        &self.tokens
    }
    /// Exact original token spelling. A synthetic semicolon and EOF consume
    /// zero bytes; string token literals can separately discard raw-string CR.
    pub fn spelling(&self, token: usize) -> Option<&[u8]> {
        self.tokens
            .get(token)
            .map(|t| &self.source.bytes()[t.start..t.end])
    }
}
fn fail<T>(offset: usize, kind: GoLexErrorKind) -> Result<T> {
    Err(GoLexError { offset, kind })
}
fn letter(c: char) -> bool {
    c == '_' || unicode::letter(c)
}
fn digit(c: char) -> bool {
    c.is_ascii_digit() || unicode::digit(c)
}
fn keyword(s: &str) -> Option<&'static str> {
    [
        "break",
        "default",
        "func",
        "interface",
        "select",
        "case",
        "defer",
        "go",
        "map",
        "struct",
        "chan",
        "else",
        "goto",
        "package",
        "switch",
        "const",
        "fallthrough",
        "if",
        "range",
        "type",
        "continue",
        "for",
        "import",
        "return",
        "var",
    ]
    .into_iter()
    .find(|k| *k == s)
}
struct Lexer<'a> {
    text: &'a str,
    at: usize,
    line_start: usize,
    semi: bool,
    tokens: Vec<GoLexToken>,
}
impl Lexer<'_> {
    fn ch(&self) -> Option<char> {
        self.text[self.at..].chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.ch()?;
        self.at += c.len_utf8();
        if c == '\n' {
            self.line_start = self.at
        }
        Some(c)
    }
    fn push(
        &mut self,
        start: usize,
        end: usize,
        kind: &'static str,
        literal: Vec<u8>,
        inserted: bool,
    ) -> Result<()> {
        // Each real token consumes at least one source byte. Inserted newlines
        // use an existing newline byte (including block comments); only an EOF
        // semicolon and EOF add tokens. This bound follows from owned bytes,
        // not the later AST-node budget or a new source-language restriction.
        if self.tokens.len() >= self.text.len() + 2 {
            return fail(start, GoLexErrorKind::Limit);
        }
        self.tokens.push(GoLexToken {
            start,
            end,
            kind,
            literal,
            inserted,
        });
        Ok(())
    }
    fn semi(&mut self, at: usize) -> Result<()> {
        self.semi = false;
        self.push(at, at, ";", vec![b'\n'], true)
    }
    fn digits(&mut self, base: u32, invalid: &mut Option<usize>) -> u8 {
        let mut flags = 0;
        while let Some(c) = self.ch() {
            if c == '_' {
                flags |= 2
            } else if c.is_ascii_digit() || base == 16 && c.is_ascii_hexdigit() {
                flags |= 1;
                if c.to_digit(16).unwrap_or(16) >= base && invalid.is_none() {
                    *invalid = Some(self.at)
                }
            } else {
                break;
            }
            self.bump();
        }
        flags
    }
    fn number(&mut self) -> Result<&'static str> {
        let start = self.at;
        let mut base = 10;
        let mut prefix = 0u8;
        let mut flags = 0;
        let mut invalid = None;
        let mut kind = "INT";
        if self.ch() != Some('.') {
            if self.ch() == Some('0') {
                self.bump();
                match self.ch().map(|c| c.to_ascii_lowercase()) {
                    Some('x') => {
                        base = 16;
                        prefix = b'x';
                        self.bump();
                    }
                    Some('o') => {
                        base = 8;
                        prefix = b'o';
                        self.bump();
                    }
                    Some('b') => {
                        base = 2;
                        prefix = b'b';
                        self.bump();
                    }
                    _ => {
                        base = 8;
                        prefix = b'0';
                        flags = 1
                    }
                }
            }
            flags |= self.digits(base, &mut invalid);
        }
        if self.ch() == Some('.') {
            kind = "FLOAT";
            if matches!(prefix, b'o' | b'b') {
                return fail(self.at, GoLexErrorKind::Number);
            }
            self.bump();
            flags |= self.digits(base, &mut invalid);
        }
        if flags & 1 == 0 {
            return fail(self.at, GoLexErrorKind::Number);
        }
        if matches!(self.ch(), Some('e' | 'E' | 'p' | 'P')) {
            let e = self.ch().unwrap().to_ascii_lowercase();
            if e == 'e' && !matches!(prefix, 0 | b'0') || e == 'p' && prefix != b'x' {
                return fail(self.at, GoLexErrorKind::Number);
            }
            self.bump();
            kind = "FLOAT";
            if matches!(self.ch(), Some('+' | '-')) {
                self.bump();
            }
            let exp = self.digits(10, &mut None);
            flags |= exp;
            if exp & 1 == 0 {
                return fail(self.at, GoLexErrorKind::Number);
            }
        } else if prefix == b'x' && kind == "FLOAT" {
            return fail(self.at, GoLexErrorKind::Number);
        }
        if self.ch() == Some('i') {
            self.bump();
            kind = "IMAG"
        }
        if kind == "INT" {
            if let Some(offset) = invalid {
                return fail(offset, GoLexErrorKind::Number);
            }
        }
        if flags & 2 != 0 {
            let raw = &self.text.as_bytes()[start..self.at];
            let mut previous = b'.';
            let mut i = 0;
            if raw.len() >= 2
                && raw[0] == b'0'
                && matches!(raw[1].to_ascii_lowercase(), b'x' | b'o' | b'b')
            {
                previous = b'0';
                i = 2
            }
            for (j, &c) in raw.iter().enumerate().skip(i) {
                let prior = previous;
                previous = c;
                if c == b'_' {
                    if prior != b'0' {
                        return fail(start + j, GoLexErrorKind::Number);
                    }
                } else if c.is_ascii_digit() || prefix == b'x' && c.is_ascii_hexdigit() {
                    previous = b'0'
                } else {
                    if prior == b'_' {
                        return fail(start + j - 1, GoLexErrorKind::Number);
                    }
                    previous = b'.'
                }
            }
            if previous == b'_' {
                return fail(self.at - 1, GoLexErrorKind::Number);
            }
        }
        Ok(kind)
    }
    fn escape(&mut self, quote: char) -> Result<()> {
        let start = self.at;
        let Some(c) = self.ch() else {
            return fail(start, GoLexErrorKind::Escape);
        };
        if matches!(c, 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\') || c == quote {
            self.bump();
            return Ok(());
        }
        let (n, base, max) = match c {
            '0'..='7' => (3, 8, 255),
            'x' => {
                self.bump();
                (2, 16, 255)
            }
            'u' => {
                self.bump();
                (4, 16, 0x10ffff)
            }
            'U' => {
                self.bump();
                (8, 16, 0x10ffff)
            }
            _ => return fail(start, GoLexErrorKind::Escape),
        };
        let mut value = 0u64;
        for _ in 0..n {
            let digit = self
                .ch()
                .filter(char::is_ascii)
                .and_then(|c| c.to_digit(base))
                .ok_or(GoLexError {
                    offset: self.at,
                    kind: GoLexErrorKind::Escape,
                })?;
            value = value * base as u64 + digit as u64;
            self.bump();
        }
        if value > max || (0xd800..0xe000).contains(&value) {
            return fail(start, GoLexErrorKind::Escape);
        }
        Ok(())
    }
    fn quoted(&mut self, quote: char) -> Result<Vec<u8>> {
        let start = self.at;
        self.bump();
        let mut count = 0;
        loop {
            let Some(c) = self.ch() else {
                return fail(start, GoLexErrorKind::Literal);
            };
            if c == quote {
                self.bump();
                break;
            }
            if c == '\n' && quote != '`' {
                return fail(start, GoLexErrorKind::Literal);
            }
            self.bump();
            count += 1;
            if c == '\\' && quote != '`' {
                self.escape(quote)?
            }
        }
        if quote == '\'' && count != 1 {
            return fail(start, GoLexErrorKind::Literal);
        }
        let raw = &self.text.as_bytes()[start..self.at];
        Ok(if quote == '`' {
            raw.iter().copied().filter(|c| *c != b'\r').collect()
        } else {
            raw.to_vec()
        })
    }
    fn directive(&self, start: usize, end: usize, block: bool, line_start: bool) -> Result<()> {
        let mut raw = &self.text.as_bytes()[start..end];
        if block {
            raw = &raw[..raw.len() - 2]
        } else if raw.last() == Some(&b'\r') {
            raw = &raw[..raw.len() - 1]
        }
        if !block && !line_start || !raw[2..].starts_with(b"line ") {
            return Ok(());
        }
        let text = &raw[7..];
        let trailing = |s: &[u8]| -> Option<(usize, Option<u64>)> {
            let colon = s.iter().rposition(|c| *c == b':')?;
            let digits = &s[colon + 1..];
            let number = if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
                None
            } else {
                std::str::from_utf8(digits)
                    .ok()
                    .and_then(|d| d.parse().ok())
            };
            Some((colon + 1, number))
        };
        let Some((i, n)) = trailing(text) else {
            return Ok(());
        };
        let Some(n) = n else {
            return fail(start + 7 + i, GoLexErrorKind::Directive);
        };
        if let Some((j, Some(line))) = trailing(&text[..i - 1]) {
            if n == 0 || n > 1 << 30 {
                return fail(start + 7 + i, GoLexErrorKind::Directive);
            }
            if line == 0 || line > 1 << 30 {
                return fail(start + 7 + j, GoLexErrorKind::Directive);
            }
        } else if n == 0 || n > 1 << 30 {
            return fail(start + 7 + i, GoLexErrorKind::Directive);
        }
        Ok(())
    }
    fn comment(&mut self) -> Result<Option<usize>> {
        let start = self.at;
        let line_start = self.at == self.line_start;
        let block = self.text.as_bytes()[start + 1] == b'*';
        self.bump();
        self.bump();
        let mut newline = None;
        if block {
            loop {
                if self.text[self.at..].starts_with("*/") {
                    self.bump();
                    self.bump();
                    break;
                }
                let Some(c) = self.ch() else {
                    return fail(start, GoLexErrorKind::Comment);
                };
                if c == '\n' && newline.is_none() {
                    newline = Some(self.at)
                }
                self.bump();
            }
        } else {
            while self.ch().is_some_and(|c| c != '\n') {
                self.bump();
            }
        }
        self.directive(start, self.at, block, line_start)?;
        Ok(newline)
    }
    fn run(&mut self) -> Result<()> {
        loop {
            while let Some(c) = self.ch() {
                if matches!(c, ' ' | '\t' | '\r') || c == '\n' && !self.semi {
                    self.bump();
                } else {
                    break;
                }
            }
            let start = self.at;
            let Some(c) = self.ch() else {
                if self.semi {
                    self.semi(start)?
                }
                return self.push(start, start, "EOF", vec![], false);
            };
            if c == '\n' {
                self.bump();
                self.semi(start)?;
                continue;
            }
            if self.text[self.at..].starts_with("//") || self.text[self.at..].starts_with("/*") {
                let newline = self.comment()?;
                if self.semi {
                    if let Some(offset) = newline {
                        self.semi(offset)?
                    }
                }
                continue;
            }
            let (kind, literal, semi) = if letter(c) {
                self.bump();
                while self.ch().is_some_and(|ch| letter(ch) || digit(ch)) {
                    self.bump();
                }
                let raw = &self.text[start..self.at];
                let key = keyword(raw);
                (
                    key.unwrap_or("IDENT"),
                    raw.as_bytes().to_vec(),
                    key.is_none()
                        || matches!(key, Some("break" | "continue" | "fallthrough" | "return")),
                )
            } else if c.is_ascii_digit()
                || c == '.'
                    && self
                        .text
                        .as_bytes()
                        .get(self.at + 1)
                        .is_some_and(u8::is_ascii_digit)
            {
                let kind = self.number()?;
                (kind, self.text.as_bytes()[start..self.at].to_vec(), true)
            } else if matches!(c, '"' | '\'' | '`') {
                (
                    if c == '\'' { "CHAR" } else { "STRING" },
                    self.quoted(c)?,
                    true,
                )
            } else {
                let operator = [
                    "<<=", ">>=", "&^=", "...", ":=", "++", "--", "+=", "-=", "*=", "/=", "%=",
                    "^=", "<=", ">=", "==", "!=", "<<", ">>", "<-", "&&", "||", "&^", "&=", "|=",
                    "+", "-", "*", "/", "%", "^", "<", ">", "=", "!", "&", "|", "~", ".", ",", ";",
                    ":", "(", ")", "[", "]", "{", "}",
                ]
                .into_iter()
                .find(|s| self.text[start..].starts_with(s))
                .ok_or(GoLexError {
                    offset: start,
                    kind: GoLexErrorKind::Character,
                })?;
                for _ in 0..operator.len() {
                    self.bump();
                }
                (
                    operator,
                    if operator == ";" { vec![b';'] } else { vec![] },
                    matches!(operator, "++" | "--" | ")" | "]" | "}"),
                )
            };
            self.push(start, self.at, kind, literal, false)?;
            self.semi = semi;
        }
    }
}
pub fn lex_go_source(source: &GoTypeSourceBuffer) -> Result<GoLexedSource> {
    if source.object().kind != "go_file" {
        return fail(0, GoLexErrorKind::Source);
    }
    let raw = source.bytes();
    let text = std::str::from_utf8(raw).map_err(|e| GoLexError {
        offset: e.valid_up_to(),
        kind: GoLexErrorKind::Utf8,
    })?;
    if let Some(offset) = raw.iter().position(|c| *c == 0) {
        return fail(offset, GoLexErrorKind::Nul);
    }
    let at = if text.starts_with('\u{feff}') { 3 } else { 0 };
    if let Some(offset) = text[at..].find('\u{feff}') {
        return fail(at + offset, GoLexErrorKind::ByteOrderMark);
    }
    let mut lexer = Lexer {
        text,
        at,
        line_start: 0,
        semi: false,
        tokens: vec![],
    };
    lexer.run()?;
    Ok(GoLexedSource {
        source: source.clone(),
        tokens: lexer.tokens,
    })
}
