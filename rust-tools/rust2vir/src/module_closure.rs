use crate::manifest::{ExpectedManifestSelection, ValidatedManifest};
use crate::path::{PortablePath, PortablePathError};
use crate::preflight::StructuralPreflight;
use crate::source_capture::{CaptureFailure, CaptureState, CapturedInput, InputKind, OpenedInput};
use std::collections::BTreeSet;

const SOURCE_FILES_MAX: usize = 256;
const SOURCE_FILE_BYTES_MAX: u64 = 1_048_576;
const SOURCE_TOTAL_BYTES_MAX: u64 = 16_777_216;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosureStatus {
    Rejected,
    SourceError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleClosureCode {
    LimitInputBytes,
    LimitInputCount,
    LimitPath,
    PreflightFileType,
    PreflightPath,
    SourceModuleMissing,
    SourceModuleAmbiguous,
    SourceModuleDuplicate,
    SourceModuleCycle,
    SubsetCfg,
    SubsetMacro,
    SubsetAttribute,
    SubsetPath,
}

impl ModuleClosureCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LimitInputBytes => "RUST_LIMIT_INPUT_BYTES",
            Self::LimitInputCount => "RUST_LIMIT_INPUT_COUNT",
            Self::LimitPath => "RUST_LIMIT_PATH",
            Self::PreflightFileType => "RUST_PREFLIGHT_FILE_TYPE",
            Self::PreflightPath => "RUST_PREFLIGHT_PATH",
            Self::SourceModuleMissing => "RUST_SOURCE_MODULE_MISSING",
            Self::SourceModuleAmbiguous => "RUST_SOURCE_MODULE_AMBIGUOUS",
            Self::SourceModuleDuplicate => "RUST_SOURCE_MODULE_DUPLICATE",
            Self::SourceModuleCycle => "RUST_SOURCE_MODULE_CYCLE",
            Self::SubsetCfg => "RUST_SUBSET_CFG",
            Self::SubsetMacro => "RUST_SUBSET_MACRO",
            Self::SubsetAttribute => "RUST_SUBSET_ATTRIBUTE",
            Self::SubsetPath => "RUST_SUBSET_PATH",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::LimitInputBytes => "input byte limit exceeded",
            Self::LimitInputCount => "input count limit exceeded",
            Self::LimitPath => "normalized path limit exceeded",
            Self::PreflightFileType => "input file type is not permitted",
            Self::PreflightPath => "input path is not portable and contained",
            Self::SourceModuleMissing => "module source is missing or unresolved",
            Self::SourceModuleAmbiguous => "module source is ambiguous",
            Self::SourceModuleDuplicate => "module source is duplicated",
            Self::SourceModuleCycle => "module source cycle is not permitted",
            Self::SubsetCfg => "conditional source configuration is not permitted",
            Self::SubsetMacro => "macro expansion is not permitted",
            Self::SubsetAttribute => "source attribute is not permitted",
            Self::SubsetPath => "explicit module paths are not permitted",
        }
    }

    pub fn status(self) -> ClosureStatus {
        match self {
            Self::SourceModuleMissing
            | Self::SourceModuleAmbiguous
            | Self::SourceModuleDuplicate
            | Self::SourceModuleCycle => ClosureStatus::SourceError,
            _ => ClosureStatus::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleClosureError {
    pub code: ModuleClosureCode,
}

impl From<ModuleClosureCode> for ModuleClosureError {
    fn from(code: ModuleClosureCode) -> Self {
        Self { code }
    }
}

#[derive(Debug)]
pub struct ModuleClosure {
    pub library_root: PortablePath,
    pub inputs: Vec<CapturedInput>,
    pub(crate) capture: CaptureState,
}

impl ModuleClosure {
    pub fn source_inputs(&self) -> impl Iterator<Item = &CapturedInput> {
        self.inputs
            .iter()
            .filter(|input| input.kind == InputKind::Source)
    }
}

pub fn discover(
    validated: ValidatedManifest,
) -> Result<(ModuleClosure, ExpectedManifestSelection), ModuleClosureError> {
    let (preflight, expected) = validated.into_parts();
    let closure = discover_at(preflight, expected.library_path().clone())?;
    Ok((closure, expected))
}

fn discover_at(
    preflight: StructuralPreflight,
    library_root: PortablePath,
) -> Result<ModuleClosure, ModuleClosureError> {
    let StructuralPreflight {
        mut inputs,
        mut capture,
    } = preflight;

    let opened = capture
        .open_candidate(&library_root)
        .map_err(map_capture_failure)?
        .ok_or(ModuleClosureCode::SourceModuleMissing)?;
    let root_identity = opened.identity;
    let root = capture_source(&mut capture, library_root.clone(), opened)?;
    let root_bytes = root.bytes.clone();
    inputs.push(root);

    let mut walker = Walker {
        capture: &mut capture,
        sources: Vec::new(),
        seen_paths: BTreeSet::from([library_root.clone()]),
        seen_identities: BTreeSet::from([root_identity.without_size()]),
        active_paths: BTreeSet::from([library_root.clone()]),
        active_identities: BTreeSet::from([root_identity.without_size()]),
        source_bytes: root_bytes.len() as u64,
        source_count: 1,
    };
    walker.walk_file(&library_root, &root_bytes, &[])?;
    inputs.append(&mut walker.sources);
    inputs.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));

    Ok(ModuleClosure {
        library_root,
        inputs,
        capture,
    })
}

struct Walker<'a> {
    capture: &'a mut CaptureState,
    sources: Vec<CapturedInput>,
    seen_paths: BTreeSet<PortablePath>,
    seen_identities: BTreeSet<crate::preflight::platform::FileIdentity>,
    active_paths: BTreeSet<PortablePath>,
    active_identities: BTreeSet<crate::preflight::platform::FileIdentity>,
    source_bytes: u64,
    source_count: usize,
}

impl Walker<'_> {
    fn walk_file(
        &mut self,
        file_path: &PortablePath,
        bytes: &[u8],
        inline_segments: &[String],
    ) -> Result<(), ModuleClosureError> {
        let tokens = tokenize(bytes)?;
        validate_expansion_forms(&tokens)?;
        self.walk_tokens(file_path, &tokens, 0, tokens.len(), inline_segments)
    }

    fn walk_tokens(
        &mut self,
        file_path: &PortablePath,
        tokens: &[Token],
        start: usize,
        end: usize,
        inline_segments: &[String],
    ) -> Result<(), ModuleClosureError> {
        let mut index = start;
        while index < end {
            if tokens[index].is_ident("mod") {
                let name = tokens
                    .get(index + 1)
                    .and_then(Token::identifier)
                    .filter(|name| valid_identifier(name))
                    .ok_or(ModuleClosureCode::SourceModuleMissing)?;
                match tokens.get(index + 2).map(|token| &token.kind) {
                    Some(TokenKind::Symbol(b';')) => {
                        self.follow_child(file_path, inline_segments, name)?;
                        index += 3;
                        continue;
                    }
                    Some(TokenKind::Symbol(b'{')) => {
                        let close = matching_brace(tokens, index + 2, end)
                            .ok_or(ModuleClosureCode::SourceModuleMissing)?;
                        let mut nested = inline_segments.to_vec();
                        nested.push(name.to_owned());
                        self.walk_tokens(file_path, tokens, index + 3, close, &nested)?;
                        index = close + 1;
                        continue;
                    }
                    _ => return Err(ModuleClosureCode::SourceModuleMissing.into()),
                }
            }
            if tokens[index].kind == TokenKind::Symbol(b'{') {
                let close = matching_brace(tokens, index, end)
                    .ok_or(ModuleClosureCode::SourceModuleMissing)?;
                self.walk_tokens(file_path, tokens, index + 1, close, inline_segments)?;
                index = close + 1;
            } else {
                index += 1;
            }
        }
        Ok(())
    }

    fn follow_child(
        &mut self,
        parent_file: &PortablePath,
        inline_segments: &[String],
        name: &str,
    ) -> Result<(), ModuleClosureError> {
        let base = module_directory(parent_file, inline_segments)?;
        let file_candidate = portable_join(&base, &format!("{name}.rs"))?;
        let module_candidate = portable_join(&base, &format!("{name}/mod.rs"))?;
        let file = self
            .capture
            .open_candidate(&file_candidate)
            .map_err(map_capture_failure)?;
        let module = self
            .capture
            .open_candidate(&module_candidate)
            .map_err(map_capture_failure)?;
        let (path, opened) = match (file, module) {
            (None, None) => return Err(ModuleClosureCode::SourceModuleMissing.into()),
            (Some(_), Some(_)) => return Err(ModuleClosureCode::SourceModuleAmbiguous.into()),
            (Some(opened), None) => (file_candidate, opened),
            (None, Some(opened)) => (module_candidate, opened),
        };

        let identity = opened.identity.without_size();
        if self.active_paths.contains(&path) || self.active_identities.contains(&identity) {
            return Err(ModuleClosureCode::SourceModuleCycle.into());
        }
        if self.seen_paths.contains(&path) || self.seen_identities.contains(&identity) {
            return Err(ModuleClosureCode::SourceModuleDuplicate.into());
        }
        if self.source_count >= SOURCE_FILES_MAX {
            return Err(ModuleClosureCode::LimitInputCount.into());
        }

        let captured = capture_source(self.capture, path.clone(), opened)?;
        let source_bytes = self
            .source_bytes
            .checked_add(captured.bytes.len() as u64)
            .ok_or(ModuleClosureCode::LimitInputBytes)?;
        if source_bytes > SOURCE_TOTAL_BYTES_MAX {
            return Err(ModuleClosureCode::LimitInputBytes.into());
        }
        self.source_bytes = source_bytes;
        self.source_count += 1;
        self.seen_paths.insert(path.clone());
        self.seen_identities.insert(identity);
        self.active_paths.insert(path.clone());
        self.active_identities.insert(identity);
        self.walk_file(&path, &captured.bytes, &[])?;
        self.active_paths.remove(&path);
        self.active_identities.remove(&identity);
        self.sources.push(captured);
        Ok(())
    }
}

fn capture_source(
    capture: &mut CaptureState,
    path: PortablePath,
    opened: OpenedInput,
) -> Result<CapturedInput, ModuleClosureError> {
    capture
        .capture_new(path, InputKind::Source, SOURCE_FILE_BYTES_MAX, opened)
        .map_err(map_capture_failure)
}

fn map_capture_failure(failure: CaptureFailure) -> ModuleClosureError {
    match failure {
        CaptureFailure::Missing => ModuleClosureCode::PreflightFileType,
        CaptureFailure::FileType => ModuleClosureCode::PreflightFileType,
        CaptureFailure::Path => ModuleClosureCode::PreflightPath,
        CaptureFailure::PathLimit => ModuleClosureCode::LimitPath,
        CaptureFailure::ByteLimit => ModuleClosureCode::LimitInputBytes,
        CaptureFailure::CountLimit => ModuleClosureCode::LimitInputCount,
    }
    .into()
}

fn module_directory(
    file_path: &PortablePath,
    inline_segments: &[String],
) -> Result<String, ModuleClosureError> {
    let mut components = file_path.as_str().split('/').collect::<Vec<_>>();
    let filename = components.pop().ok_or(ModuleClosureCode::PreflightPath)?;
    if filename != "lib.rs" && filename != "mod.rs" {
        let stem = filename
            .strip_suffix(".rs")
            .filter(|stem| !stem.is_empty())
            .ok_or(ModuleClosureCode::PreflightPath)?;
        components.push(stem);
    }
    components.extend(inline_segments.iter().map(String::as_str));
    Ok(components.join("/"))
}

fn portable_join(base: &str, suffix: &str) -> Result<PortablePath, ModuleClosureError> {
    let value = if base.is_empty() {
        suffix.to_owned()
    } else {
        format!("{base}/{suffix}")
    };
    PortablePath::parse(&value).map_err(|error| match error {
        PortablePathError::Limit => ModuleClosureCode::LimitPath.into(),
        PortablePathError::Invalid | PortablePathError::Collision => {
            ModuleClosureCode::PreflightPath.into()
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
}

impl Token {
    fn is_ident(&self, expected: &str) -> bool {
        matches!(&self.kind, TokenKind::Ident(value) if value == expected)
    }

    fn identifier(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Ident(value) => Some(value),
            TokenKind::Symbol(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenKind {
    Ident(String),
    Symbol(u8),
}

fn tokenize(bytes: &[u8]) -> Result<Vec<Token>, ModuleClosureError> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
        } else if bytes[index..].starts_with(b"//") {
            index = bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |offset| index + offset + 1);
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index)?;
        } else if let Some(end) = skip_raw_string(bytes, index)? {
            index = end;
        } else if bytes[index] == b'"' {
            index = skip_quoted(bytes, index, b'"')?;
        } else if bytes[index] == b'\'' && looks_like_character_literal(bytes, index) {
            index = skip_quoted(bytes, index, b'\'')?;
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Ident(
                    std::str::from_utf8(&bytes[start..index])
                        .expect("ASCII identifiers are UTF-8")
                        .to_owned(),
                ),
            });
        } else {
            tokens.push(Token {
                kind: TokenKind::Symbol(bytes[index]),
            });
            index += 1;
        }
    }
    Ok(tokens)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Result<usize, ModuleClosureError> {
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
    Err(ModuleClosureCode::SourceModuleMissing.into())
}

fn skip_raw_string(bytes: &[u8], start: usize) -> Result<Option<usize>, ModuleClosureError> {
    let mut index = start;
    if bytes.get(index) == Some(&b'b') || bytes.get(index) == Some(&b'c') {
        index += 1;
    }
    if bytes.get(index) != Some(&b'r') {
        return Ok(None);
    }
    index += 1;
    let hashes = bytes[index..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    index += hashes;
    if bytes.get(index) != Some(&b'"') {
        return Ok(None);
    }
    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..index + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Ok(Some(index + 1 + hashes));
        }
        index += 1;
    }
    Err(ModuleClosureCode::SourceModuleMissing.into())
}

fn skip_quoted(bytes: &[u8], start: usize, delimiter: u8) -> Result<usize, ModuleClosureError> {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = index.saturating_add(2);
        } else if bytes[index] == delimiter {
            return Ok(index + 1);
        } else if bytes[index] == b'\n' && delimiter == b'\'' {
            break;
        } else {
            index += 1;
        }
    }
    Err(ModuleClosureCode::SourceModuleMissing.into())
}

fn looks_like_character_literal(bytes: &[u8], start: usize) -> bool {
    let mut index = start + 1;
    let mut escaped = false;
    while index < bytes.len() && bytes[index] != b'\n' {
        if !escaped && bytes[index] == b'\'' {
            return true;
        }
        escaped = !escaped && bytes[index] == b'\\';
        if bytes[index] != b'\\' {
            escaped = false;
        }
        index += 1;
    }
    false
}

fn validate_expansion_forms(tokens: &[Token]) -> Result<(), ModuleClosureError> {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].is_ident("macro") {
            return Err(ModuleClosureCode::SubsetMacro.into());
        }
        if tokens[index].kind == TokenKind::Symbol(b'#') {
            let bracket = if tokens.get(index + 1).is_some_and(|token| {
                token.kind == TokenKind::Symbol(b'!')
                    && tokens
                        .get(index + 2)
                        .is_some_and(|next| next.kind == TokenKind::Symbol(b'['))
            }) {
                index + 2
            } else if tokens
                .get(index + 1)
                .is_some_and(|token| token.kind == TokenKind::Symbol(b'['))
            {
                index + 1
            } else {
                return Err(ModuleClosureCode::SubsetAttribute.into());
            };
            let close =
                matching_square(tokens, bracket).ok_or(ModuleClosureCode::SubsetAttribute)?;
            let name = tokens
                .get(bracket + 1)
                .and_then(Token::identifier)
                .ok_or(ModuleClosureCode::SubsetAttribute)?;
            match name {
                "cfg" | "cfg_attr" => return Err(ModuleClosureCode::SubsetCfg.into()),
                "path" => return Err(ModuleClosureCode::SubsetPath.into()),
                "doc" | "no_std" => {
                    if contains_macro_invocation(&tokens[bracket + 1..close]) {
                        return Err(ModuleClosureCode::SubsetMacro.into());
                    }
                }
                _ => return Err(ModuleClosureCode::SubsetAttribute.into()),
            }
            index = close + 1;
            continue;
        }
        if macro_invocation_at(tokens, index) {
            return Err(ModuleClosureCode::SubsetMacro.into());
        }
        index += 1;
    }
    Ok(())
}

fn contains_macro_invocation(tokens: &[Token]) -> bool {
    (0..tokens.len()).any(|index| macro_invocation_at(tokens, index))
}

fn macro_invocation_at(tokens: &[Token], index: usize) -> bool {
    let Some(name) = tokens.get(index).and_then(Token::identifier) else {
        return false;
    };
    if !tokens
        .get(index + 1)
        .is_some_and(|token| token.kind == TokenKind::Symbol(b'!'))
    {
        return false;
    }
    name == "macro_rules"
        || tokens
            .get(index + 2)
            .is_some_and(|token| matches!(token.kind, TokenKind::Symbol(b'(' | b'[' | b'{')))
}

fn matching_brace(tokens: &[Token], open: usize, end: usize) -> Option<usize> {
    matching_delimiter(tokens, open, end, b'{', b'}')
}

fn matching_square(tokens: &[Token], open: usize) -> Option<usize> {
    matching_delimiter(tokens, open, tokens.len(), b'[', b']')
}

fn matching_delimiter(
    tokens: &[Token],
    open: usize,
    end: usize,
    opening: u8,
    closing: u8,
) -> Option<usize> {
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(open) {
        match token.kind {
            TokenKind::Symbol(value) if value == opening => depth += 1,
            TokenKind::Symbol(value) if value == closing => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn valid_identifier(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && value
            .as_bytes()
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        && value != "_"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_ignores_comments_and_literals() {
        let tokens = tokenize(
            br##"// mod a;
            const S: &str = r#"mod b;"#;
            /* mod c; */ mod real;
        "##,
        )
        .unwrap();
        assert_eq!(
            tokens
                .iter()
                .filter_map(Token::identifier)
                .filter(|name| *name == "mod")
                .count(),
            1
        );
    }
}
