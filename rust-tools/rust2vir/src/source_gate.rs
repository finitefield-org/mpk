#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceGateCode {
    SourceParse,
    SubsetCfg,
    SubsetMacro,
    SubsetAttribute,
    SubsetImport,
    SubsetVisibility,
    SubsetPath,
    SubsetExpansion,
    SubsetIdentifier,
    SubsetFunctionKind,
}

impl SourceGateCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceParse => "RUST_SOURCE_PARSE",
            Self::SubsetCfg => "RUST_SUBSET_CFG",
            Self::SubsetMacro => "RUST_SUBSET_MACRO",
            Self::SubsetAttribute => "RUST_SUBSET_ATTRIBUTE",
            Self::SubsetImport => "RUST_SUBSET_IMPORT",
            Self::SubsetVisibility => "RUST_SUBSET_VISIBILITY",
            Self::SubsetPath => "RUST_SUBSET_PATH",
            Self::SubsetExpansion => "RUST_SUBSET_EXPANSION",
            Self::SubsetIdentifier => "RUST_SUBSET_IDENTIFIER",
            Self::SubsetFunctionKind => "RUST_SUBSET_FUNCTION_KIND",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::SourceParse => "Rust source could not be parsed",
            Self::SubsetCfg => "conditional source configuration is not permitted",
            Self::SubsetMacro => "macro expansion is not permitted",
            Self::SubsetAttribute => "source attribute is not permitted",
            Self::SubsetImport => "source imports are not permitted",
            Self::SubsetVisibility => "restricted visibility is not permitted",
            Self::SubsetPath => "explicit module paths are not permitted",
            Self::SubsetExpansion => "expansion-affecting source syntax is not permitted",
            Self::SubsetIdentifier => "source identifier is not canonical",
            Self::SubsetFunctionKind => "function kind is outside the closed Rust subset",
        }
    }

    pub fn phase(self) -> &'static str {
        match self {
            Self::SubsetIdentifier | Self::SubsetFunctionKind => "subset",
            _ => "source",
        }
    }

    pub fn status(self) -> SourceGateStatus {
        match self {
            Self::SourceParse => SourceGateStatus::SourceError,
            _ => SourceGateStatus::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceGateStatus {
    Rejected,
    SourceError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceGateError {
    pub code: SourceGateCode,
}

impl From<SourceGateCode> for SourceGateError {
    fn from(code: SourceGateCode) -> Self {
        Self { code }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRole {
    CrateRoot,
    Module,
}

pub fn validate_source(bytes: &[u8], role: SourceRole) -> Result<(), SourceGateError> {
    let source = std::str::from_utf8(bytes).map_err(|_| SourceGateCode::SourceParse)?;
    if !parses_as_rust_2021(source) {
        return Err(SourceGateCode::SourceParse.into());
    }
    let tokens = lex(source.as_bytes())?;
    validate_profile_tokens(&tokens, role)
}

fn parses_as_rust_2021(source: &str) -> bool {
    let Ok(tokens) = lex(source.as_bytes()) else {
        return false;
    };
    validate_delimiters(&tokens) && validate_item_heads(&tokens)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
}

impl Token {
    fn symbol(value: u8) -> Self {
        Self {
            kind: TokenKind::Symbol(value),
        }
    }

    fn is_symbol(&self, expected: u8) -> bool {
        self.kind == TokenKind::Symbol(expected)
    }

    fn identifier(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Identifier(value) => Some(value),
            _ => None,
        }
    }

    fn is_identifier(&self, expected: &str) -> bool {
        self.identifier() == Some(expected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenKind {
    Identifier(String),
    InvalidIdentifier,
    StringLiteral,
    OtherLiteral,
    Symbol(u8),
}

fn lex(bytes: &[u8]) -> Result<Vec<Token>, SourceGateError> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            index = bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |offset| index + offset + 1);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index)?;
            continue;
        }
        if let Some((end, kind)) = skip_literal(bytes, index)? {
            tokens.push(Token { kind });
            index = end;
            continue;
        }
        if bytes[index..].starts_with(b"r#")
            && bytes
                .get(index + 2)
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        {
            index += 3;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(Token {
                kind: TokenKind::InvalidIdentifier,
            });
            continue;
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            if bytes.get(index).is_some_and(|byte| !byte.is_ascii()) {
                while index < bytes.len()
                    && (!bytes[index].is_ascii()
                        || bytes[index].is_ascii_alphanumeric()
                        || bytes[index] == b'_')
                {
                    index += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::InvalidIdentifier,
                });
                continue;
            }
            tokens.push(Token {
                kind: TokenKind::Identifier(
                    std::str::from_utf8(&bytes[start..index])
                        .expect("ASCII identifiers are UTF-8")
                        .to_owned(),
                ),
            });
            continue;
        }
        if !bytes[index].is_ascii() {
            let width = std::str::from_utf8(&bytes[index..])
                .map_err(|_| SourceGateCode::SourceParse)?
                .chars()
                .next()
                .expect("nonempty UTF-8 suffix")
                .len_utf8();
            index += width;
            tokens.push(Token {
                kind: TokenKind::InvalidIdentifier,
            });
            continue;
        }
        tokens.push(Token::symbol(bytes[index]));
        index += 1;
    }
    Ok(tokens)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Result<usize, SourceGateError> {
    let mut depth = 1_usize;
    let mut index = start + 2;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"/*") {
            depth += 1;
            index += 2;
        } else if bytes[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Ok(index);
            }
        } else {
            index += 1;
        }
    }
    Err(SourceGateCode::SourceParse.into())
}

fn skip_literal(bytes: &[u8], start: usize) -> Result<Option<(usize, TokenKind)>, SourceGateError> {
    for (prefix, kind) in [
        (b"br".as_slice(), TokenKind::OtherLiteral),
        (b"cr".as_slice(), TokenKind::OtherLiteral),
        (b"r".as_slice(), TokenKind::StringLiteral),
    ] {
        if bytes[start..].starts_with(prefix) {
            let mut marker = start + prefix.len();
            let hashes = bytes[marker..]
                .iter()
                .take_while(|byte| **byte == b'#')
                .count();
            marker += hashes;
            if bytes.get(marker) == Some(&b'"') {
                let mut index = marker + 1;
                while index < bytes.len() {
                    if bytes[index] == b'"'
                        && bytes
                            .get(index + 1..index + 1 + hashes)
                            .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
                    {
                        return Ok(Some((index + 1 + hashes, kind)));
                    }
                    index += 1;
                }
                return Err(SourceGateCode::SourceParse.into());
            }
        }
    }
    for (prefix, delimiter, kind) in [
        (b"b".as_slice(), b'"', TokenKind::OtherLiteral),
        (b"c".as_slice(), b'"', TokenKind::OtherLiteral),
        (b"".as_slice(), b'"', TokenKind::StringLiteral),
        (b"b".as_slice(), b'\'', TokenKind::OtherLiteral),
    ] {
        let quote = start + prefix.len();
        if bytes[start..].starts_with(prefix) && bytes.get(quote) == Some(&delimiter) {
            return skip_quoted(bytes, quote, delimiter).map(|end| Some((end, kind)));
        }
    }
    if bytes.get(start) == Some(&b'\'') && looks_like_character_literal(bytes, start) {
        return skip_quoted(bytes, start, b'\'').map(|end| Some((end, TokenKind::OtherLiteral)));
    }
    Ok(None)
}

fn skip_quoted(bytes: &[u8], quote: usize, delimiter: u8) -> Result<usize, SourceGateError> {
    let mut index = quote + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = index.saturating_add(2);
        } else if bytes[index] == delimiter {
            return Ok(index + 1);
        } else if delimiter == b'\'' && bytes[index] == b'\n' {
            break;
        } else {
            index += 1;
        }
    }
    Err(SourceGateCode::SourceParse.into())
}

fn looks_like_character_literal(bytes: &[u8], start: usize) -> bool {
    if bytes
        .get(start + 1)
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
    {
        let mut end = start + 2;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            end += 1;
        }
        return bytes.get(end) == Some(&b'\'');
    }
    let mut escaped = false;
    for byte in bytes
        .iter()
        .skip(start + 1)
        .take_while(|byte| **byte != b'\n')
    {
        if !escaped && *byte == b'\'' {
            return true;
        }
        escaped = !escaped && *byte == b'\\';
        if *byte != b'\\' {
            escaped = false;
        }
    }
    false
}

fn validate_profile_tokens(tokens: &[Token], role: SourceRole) -> Result<(), SourceGateError> {
    let mut findings = Vec::new();
    let mut index = 0;
    let mut brace_depth = 0_usize;
    let mut crate_prefix = true;
    let mut no_std_seen = false;
    while index < tokens.len() {
        if tokens[index].kind == TokenKind::InvalidIdentifier {
            findings.push(SourceGateCode::SubsetIdentifier);
        }
        if tokens[index].is_identifier("macro") {
            findings.push(SourceGateCode::SubsetMacro);
        }
        if tokens[index].is_identifier("use")
            || (tokens[index].is_identifier("extern")
                && tokens
                    .get(index + 1)
                    .is_some_and(|token| token.is_identifier("crate")))
        {
            findings.push(SourceGateCode::SubsetImport);
        }
        if tokens[index].is_identifier("extern")
            && tokens
                .get(index + 1)
                .is_none_or(|token| !token.is_identifier("crate"))
        {
            findings.push(SourceGateCode::SubsetFunctionKind);
        }
        if tokens[index].is_identifier("pub")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.is_symbol(b'('))
        {
            findings.push(SourceGateCode::SubsetVisibility);
        }
        if tokens[index].is_symbol(b'$') {
            findings.push(SourceGateCode::SubsetExpansion);
        }
        if tokens[index].is_symbol(b'#') {
            let inner = tokens
                .get(index + 1)
                .is_some_and(|token| token.is_symbol(b'!'));
            let bracket = index + if inner { 2 } else { 1 };
            let close = matching_delimiter(tokens, bracket, b'[', b']')
                .ok_or(SourceGateCode::SourceParse)?;
            inspect_attribute(
                &tokens[bracket + 1..close],
                inner,
                role,
                brace_depth == 0 && crate_prefix,
                &mut no_std_seen,
                &mut findings,
            );
            if !inner {
                crate_prefix = false;
            }
            index = close + 1;
            continue;
        }
        if macro_invocation_at(tokens, index) {
            findings.push(SourceGateCode::SubsetMacro);
        }
        if tokens[index].is_symbol(b'{') {
            brace_depth += 1;
        } else if tokens[index].is_symbol(b'}') {
            brace_depth = brace_depth.saturating_sub(1);
        }
        crate_prefix = false;
        index += 1;
    }
    findings.sort_by_key(|code| source_gate_precedence(*code));
    findings.dedup();
    findings
        .first()
        .copied()
        .map_or(Ok(()), |code| Err(code.into()))
}

fn inspect_attribute(
    content: &[Token],
    inner: bool,
    role: SourceRole,
    crate_prefix: bool,
    no_std_seen: &mut bool,
    findings: &mut Vec<SourceGateCode>,
) {
    if content
        .iter()
        .enumerate()
        .any(|(index, _)| macro_invocation_at(content, index))
    {
        findings.push(SourceGateCode::SubsetMacro);
    }
    if content.iter().any(|token| token.is_symbol(b'$')) {
        findings.push(SourceGateCode::SubsetExpansion);
    }
    match content.first().and_then(Token::identifier) {
        Some("cfg" | "cfg_attr") => findings.push(SourceGateCode::SubsetCfg),
        Some("path") => findings.push(SourceGateCode::SubsetPath),
        Some("doc")
            if content.len() == 3
                && content[1].is_symbol(b'=')
                && content[2].kind == TokenKind::StringLiteral => {}
        Some("no_std")
            if inner
                && role == SourceRole::CrateRoot
                && crate_prefix
                && content.len() == 1
                && !*no_std_seen =>
        {
            *no_std_seen = true;
        }
        _ => findings.push(SourceGateCode::SubsetAttribute),
    }
}

fn macro_invocation_at(tokens: &[Token], index: usize) -> bool {
    let Some(name) = tokens.get(index).and_then(Token::identifier) else {
        return false;
    };
    if !tokens
        .get(index + 1)
        .is_some_and(|token| token.is_symbol(b'!'))
    {
        return false;
    }
    name == "macro_rules"
        || tokens.get(index + 2).is_some_and(|token| {
            token.is_symbol(b'(') || token.is_symbol(b'[') || token.is_symbol(b'{')
        })
}

fn matching_delimiter(tokens: &[Token], open: usize, opening: u8, closing: u8) -> Option<usize> {
    if !tokens.get(open)?.is_symbol(opening) {
        return None;
    }
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        if token.is_symbol(opening) {
            depth += 1;
        } else if token.is_symbol(closing) {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn validate_delimiters(tokens: &[Token]) -> bool {
    let mut stack = Vec::new();
    for token in tokens {
        match token.kind {
            TokenKind::Symbol(value @ (b'(' | b'[' | b'{')) => stack.push(value),
            TokenKind::Symbol(b')') if stack.pop() != Some(b'(') => return false,
            TokenKind::Symbol(b']') if stack.pop() != Some(b'[') => return false,
            TokenKind::Symbol(b'}') if stack.pop() != Some(b'{') => return false,
            _ => {}
        }
    }
    stack.is_empty()
}

fn validate_item_heads(tokens: &[Token]) -> bool {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].is_symbol(b'#') {
            let inner = tokens
                .get(index + 1)
                .is_some_and(|token| token.is_symbol(b'!'));
            let bracket = index + if inner { 2 } else { 1 };
            let Some(close) = matching_delimiter(tokens, bracket, b'[', b']') else {
                return false;
            };
            index = close + 1;
            continue;
        }
        if let Some(close) = macro_token_tree_close(tokens, index) {
            index = close + 1;
            continue;
        }
        if tokens[index].is_identifier("fn")
            && tokens.get(index + 1).is_some_and(|name| {
                name.identifier().is_some() || name.kind == TokenKind::InvalidIdentifier
            })
            && !valid_function_head(tokens, index)
        {
            return false;
        }
        index += 1;
    }
    true
}

fn macro_token_tree_close(tokens: &[Token], index: usize) -> Option<usize> {
    let opening = if tokens[index].is_identifier("macro_rules")
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.is_symbol(b'!'))
    {
        index + 3
    } else if tokens[index].identifier().is_some()
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.is_symbol(b'!'))
    {
        index + 2
    } else if tokens[index].is_identifier("macro") {
        tokens
            .iter()
            .enumerate()
            .skip(index + 1)
            .find_map(|(position, token)| token.is_symbol(b'{').then_some(position))?
    } else {
        return None;
    };
    let opening_symbol = match tokens.get(opening)?.kind {
        TokenKind::Symbol(value @ (b'(' | b'[' | b'{')) => value,
        _ => return None,
    };
    let closing_symbol = match opening_symbol {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        _ => return None,
    };
    matching_delimiter(tokens, opening, opening_symbol, closing_symbol)
}

fn valid_function_head(tokens: &[Token], index: usize) -> bool {
    let Some(name) = tokens.get(index + 1) else {
        return false;
    };
    if name.identifier().is_none() && name.kind != TokenKind::InvalidIdentifier {
        return false;
    }
    let mut open = index + 2;
    if tokens.get(open).is_some_and(|token| token.is_symbol(b'<')) {
        let mut depth = 0_usize;
        loop {
            let Some(token) = tokens.get(open) else {
                return false;
            };
            if token.is_symbol(b'<') {
                depth += 1;
            } else if token.is_symbol(b'>') {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
                if depth == 0 {
                    open += 1;
                    break;
                }
            }
            open += 1;
        }
    }
    if !tokens.get(open).is_some_and(|token| token.is_symbol(b'(')) {
        return false;
    }
    let Some(close) = matching_delimiter(tokens, open, b'(', b')') else {
        return false;
    };
    if !valid_parameter_list(tokens, open, close) {
        return false;
    }
    if tokens
        .get(open + 1)
        .is_some_and(|token| token.is_symbol(b'-'))
        && tokens
            .get(open + 2)
            .is_some_and(|token| token.is_symbol(b'>'))
    {
        return false;
    }
    let mut after = close + 1;
    if tokens.get(after).is_some_and(|token| token.is_symbol(b'-'))
        && tokens
            .get(after + 1)
            .is_some_and(|token| token.is_symbol(b'>'))
    {
        after += 2;
        if tokens.get(after).is_none_or(|token| {
            token.is_symbol(b'{') || token.is_symbol(b';') || token.is_identifier("where")
        }) {
            return false;
        }
    }
    true
}

fn valid_parameter_list(tokens: &[Token], open: usize, close: usize) -> bool {
    let mut start = open + 1;
    let mut paren_depth = 0_usize;
    let mut square_depth = 0_usize;
    let mut brace_depth = 0_usize;
    let mut angle_depth = 0_usize;
    for index in open + 1..=close {
        let at_end = index == close;
        let token = &tokens[index];
        if !at_end
            && token.is_symbol(b',')
            && paren_depth == 0
            && square_depth == 0
            && brace_depth == 0
            && angle_depth == 0
        {
            if !valid_parameter(&tokens[start..index]) {
                return false;
            }
            start = index + 1;
            continue;
        }
        if at_end {
            return start == close || valid_parameter(&tokens[start..close]);
        }
        match token.kind {
            TokenKind::Symbol(b'(') => paren_depth += 1,
            TokenKind::Symbol(b')') => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::Symbol(b'[') => square_depth += 1,
            TokenKind::Symbol(b']') => square_depth = square_depth.saturating_sub(1),
            TokenKind::Symbol(b'{') => brace_depth += 1,
            TokenKind::Symbol(b'}') => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::Symbol(b'<') => angle_depth += 1,
            TokenKind::Symbol(b'>') => angle_depth = angle_depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

fn valid_parameter(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let colon = tokens.iter().enumerate().find_map(|(index, token)| {
        let previous_is_colon = index > 0 && tokens[index - 1].is_symbol(b':');
        let next_is_colon = tokens
            .get(index + 1)
            .is_some_and(|next| next.is_symbol(b':'));
        (token.is_symbol(b':') && !previous_is_colon && !next_is_colon).then_some(index)
    });
    if let Some(colon) = colon {
        return colon > 0 && colon + 1 < tokens.len();
    }
    valid_receiver(tokens) || tokens.iter().all(|token| token.is_symbol(b'.'))
}

fn valid_receiver(tokens: &[Token]) -> bool {
    let mut index = 0;
    if tokens.get(index).is_some_and(|token| token.is_symbol(b'&')) {
        index += 1;
        if tokens
            .get(index)
            .is_some_and(|token| token.is_symbol(b'\''))
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.identifier().is_some())
        {
            index += 2;
        }
    }
    if tokens
        .get(index)
        .is_some_and(|token| token.is_identifier("mut"))
    {
        index += 1;
    }
    index + 1 == tokens.len()
        && tokens
            .get(index)
            .is_some_and(|token| token.is_identifier("self"))
}

fn source_gate_precedence(code: SourceGateCode) -> u8 {
    match code {
        SourceGateCode::SourceParse => 0,
        SourceGateCode::SubsetCfg => 1,
        SourceGateCode::SubsetMacro => 2,
        SourceGateCode::SubsetAttribute => 3,
        SourceGateCode::SubsetImport => 4,
        SourceGateCode::SubsetVisibility => 5,
        SourceGateCode::SubsetPath => 6,
        SourceGateCode::SubsetExpansion => 7,
        SourceGateCode::SubsetIdentifier => 8,
        SourceGateCode::SubsetFunctionKind => 9,
    }
}
