//! Bound the physical import header before independently lexing/parsing it.
//! No source object, source byte, location or subsequent body is replaced.
use super::*;
use std::result::Result;

struct Reader<'a> {
    raw: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn skip(&mut self) -> Result<(), GoSourcePackageErrorKind> {
        loop {
            while self
                .raw
                .get(self.at)
                .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b';'))
            {
                self.at += 1;
            }
            if self.raw[self.at..].starts_with(b"//") {
                while self.raw.get(self.at).is_some_and(|b| *b != b'\n') {
                    if self.raw[self.at] == 0 {
                        return Err(GoSourcePackageErrorKind::Read);
                    }
                    self.at += 1;
                }
            } else if self.raw[self.at..].starts_with(b"/*") {
                self.at += 2;
                loop {
                    if self.raw[self.at..].starts_with(b"*/") {
                        self.at += 2;
                        break;
                    }
                    match self.raw.get(self.at) {
                        None => return Err(GoSourcePackageErrorKind::Header),
                        Some(0) => return Err(GoSourcePackageErrorKind::Read),
                        _ => self.at += 1,
                    }
                }
            } else {
                return if self.raw.get(self.at) == Some(&0) {
                    Err(GoSourcePackageErrorKind::Read)
                } else {
                    Ok(())
                };
            }
        }
    }
    fn ident_byte(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
    }
    fn word(&mut self) -> Result<&[u8], GoSourcePackageErrorKind> {
        self.skip()?;
        let start = self.at;
        while self.raw.get(self.at).is_some_and(|b| Self::ident_byte(*b)) {
            self.at += 1;
        }
        if start == self.at {
            return Err(GoSourcePackageErrorKind::Header);
        }
        Ok(&self.raw[start..self.at])
    }
    fn quoted(&mut self) -> Result<(), GoSourcePackageErrorKind> {
        self.skip()?;
        let quote = *self
            .raw
            .get(self.at)
            .ok_or(GoSourcePackageErrorKind::Header)?;
        if !matches!(quote, b'"' | b'`') {
            return Err(GoSourcePackageErrorKind::Header);
        }
        self.at += 1;
        loop {
            let b = *self
                .raw
                .get(self.at)
                .ok_or(GoSourcePackageErrorKind::Header)?;
            self.at += 1;
            if b == 0 {
                return Err(GoSourcePackageErrorKind::Read);
            }
            if b == quote {
                return Ok(());
            }
            if quote == b'"' && b == b'\n' {
                return Err(GoSourcePackageErrorKind::Header);
            }
            if quote == b'"' && b == b'\\' {
                let escaped = *self
                    .raw
                    .get(self.at)
                    .ok_or(GoSourcePackageErrorKind::Header)?;
                if escaped == 0 {
                    return Err(GoSourcePackageErrorKind::Read);
                }
                self.at += 1;
            }
        }
    }
    fn spec(&mut self) -> Result<(), GoSourcePackageErrorKind> {
        self.skip()?;
        if self.raw.get(self.at) == Some(&b'.') {
            self.at += 1;
        } else if self.raw.get(self.at).is_some_and(|b| Self::ident_byte(*b)) {
            self.word()?;
        }
        self.quoted()
    }
    fn end(&mut self) -> Result<usize, GoSourcePackageErrorKind> {
        if self.word()? != b"package" {
            return Err(GoSourcePackageErrorKind::Header);
        }
        self.word()?;
        loop {
            self.skip()?;
            if self.raw.get(self.at) != Some(&b'i') {
                return Ok(self.at);
            }
            if self.word()? != b"import" {
                return Err(GoSourcePackageErrorKind::Header);
            }
            self.skip()?;
            if self.raw.get(self.at) == Some(&b'(') {
                self.at += 1;
                loop {
                    self.skip()?;
                    if self.raw.get(self.at) == Some(&b')') {
                        self.at += 1;
                        break;
                    }
                    self.spec()?;
                }
            } else {
                self.spec()?;
            }
        }
    }
}

// parser.ImportsOnly differs from go/build's import inventory: the former
// accepts arbitrary decoded import bytes and inspects one following token.
// This linear grammar has no AST-node budget and never parses later bodies.
pub(super) fn compiler_role_header(source: &GoTypeSourceBuffer) -> Option<(bool, bool)> {
    let lexed = lex::lex_go_compiler_import_header(source).ok()?;
    let tokens = lexed
        .tokens()
        .iter()
        .filter(|t| t.kind() != "COMMENT")
        .collect::<Vec<_>>();
    struct HeaderTokens<'a> {
        tokens: &'a [&'a GoLexToken],
        at: usize,
    }
    impl<'a> HeaderTokens<'a> {
        fn kind(&self) -> &'a str {
            self.tokens.get(self.at).map_or("EOF", |t| t.kind())
        }
        fn take(&mut self, kind: &str) -> Option<&'a GoLexToken> {
            if self.kind() != kind {
                return None;
            }
            let token = *self.tokens.get(self.at)?;
            self.at += 1;
            Some(token)
        }
        fn semi(&mut self) -> Option<()> {
            if matches!(self.kind(), ")" | "}") {
                Some(())
            } else {
                self.take(";").map(|_| ())
            }
        }
        fn import(&mut self) -> Option<bool> {
            if matches!(self.kind(), "IDENT" | ".") {
                self.at += 1;
            }
            let path = self.take("STRING")?;
            let bytes = source_scopes::unquote(path.literal()).ok()?;
            self.semi()?;
            Some(bytes == b"C")
        }
    }
    let mut parser = HeaderTokens {
        tokens: &tokens,
        at: 0,
    };
    parser.take("package")?;
    let documentation = parser.take("IDENT")?.literal() == b"documentation";
    parser.semi()?;
    let mut cgo = false;
    while parser.kind() == "import" {
        parser.at += 1;
        if parser.kind() == "(" {
            parser.at += 1;
            while parser.kind() != ")" {
                cgo |= parser.import()?;
            }
            parser.take(")")?;
            parser.semi()?;
        } else {
            cgo |= parser.import()?;
        }
    }
    Some((documentation, cgo))
}

pub(super) fn boundary(source: &GoTypeSourceBuffer) -> Result<usize, GoSourcePackageErrorKind> {
    let raw = source.bytes();
    let at = if raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        3
    } else {
        0
    };
    match (Reader { raw, at }).end() {
        Err(GoSourcePackageErrorKind::Header) => {
            // Like the native import reader, malformed header structure needs
            // full original text for syntax diagnostics, not a fabricated prefix.
            if raw.contains(&0) {
                Err(GoSourcePackageErrorKind::Read)
            } else {
                Ok(raw.len())
            }
        }
        other => other,
    }
}

// The native comment reader uses this same leading whitespace/comment walk
// for auxiliary inputs. Its read failures have a distinct native disposition.
pub(super) fn comment_prefix(raw: &[u8]) -> Option<&[u8]> {
    let at = if raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        3
    } else {
        0
    };
    let mut reader = Reader { raw, at };
    reader.skip().ok()?;
    Some(&raw[..reader.at])
}

pub(super) struct Header {
    pub name: String,
    pub imports: Vec<String>,
    pub cgo_docs: Vec<String>,
}

pub(super) fn read(
    source: &GoTypeSourceBuffer,
    end: usize,
) -> Result<Header, GoSourcePackageErrorKind> {
    let parsed =
        parse::parse_go_source_header(source, end).map_err(|_| GoSourcePackageErrorKind::Header)?;
    let root = &parsed.nodes()[parsed.root()];
    let name_node = *source_signatures::list(root, "Name")
        .first()
        .ok_or(GoSourcePackageErrorKind::Header)?;
    let name = source_signatures::attr(&parsed.nodes()[name_node], "Name")
        .ok_or(GoSourcePackageErrorKind::Header)?
        .to_string();
    let mut imports = BTreeSet::new();
    for node in parsed.nodes().iter().filter(|n| n.kind() == "ImportSpec") {
        let path_node = *source_signatures::list(node, "Path")
            .first()
            .ok_or(GoSourcePackageErrorKind::Header)?;
        let bytes = source_scopes::unquote(
            parsed
                .spelling(path_node)
                .ok_or(GoSourcePackageErrorKind::Header)?,
        )
        .map_err(|_| GoSourcePackageErrorKind::Header)?;
        let path = String::from_utf8(bytes).map_err(|_| GoSourcePackageErrorKind::Import)?;
        // Native go/build accepts graphic Unicode import spellings. Actual
        // module/import resolution remains separate from this header inventory.
        if path.is_empty()
            || path.chars().any(|c| {
                !graphic::graphic(c)
                    || c.is_whitespace()
                    || matches!(
                        c,
                        '!' | '"'
                            | '#'
                            | '$'
                            | '%'
                            | '&'
                            | '\''
                            | '('
                            | ')'
                            | '*'
                            | ','
                            | ':'
                            | ';'
                            | '<'
                            | '='
                            | '>'
                            | '?'
                            | '['
                            | '\\'
                            | ']'
                            | '^'
                            | '{'
                            | '|'
                            | '}'
                            | '`'
                            | '\u{fffd}'
                    )
            })
        {
            return Err(GoSourcePackageErrorKind::Import);
        }
        imports.insert(path);
    }
    let cgo_docs = cgo::import_docs(&parsed);
    Ok(Header {
        name,
        imports: imports.into_iter().collect(),
        cgo_docs,
    })
}
