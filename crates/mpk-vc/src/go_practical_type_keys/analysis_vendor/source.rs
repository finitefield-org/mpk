//! Go analysis source syntax relative to an untrusted complete observation.
use super::super::{
    parse_go_source, GoParseError, GoParsedSource, GoSourceSnapshotLookupError, GoTypeSourceBuffer,
};
use super::archives::root_before;
use super::GoAnalysisModuleInputs;
use std::collections::BTreeMap;

const NON_GO_INPUTS: [&[u8]; 15] = [
    b".s",
    b".S",
    b".h",
    b".c",
    b".cc",
    b".cpp",
    b".cxx",
    b".m",
    b".f",
    b".F",
    b".for",
    b".f90",
    b".syso",
    b".swig",
    b".swigcxx",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoAnalysisSourceError {
    Snapshot(GoSourceSnapshotLookupError),
    Namespace,
    NonGoInput,
    Syntax { path: String, error: GoParseError },
    Embed { path: String },
    MissingFrontend,
}
impl std::fmt::Display for GoAnalysisSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go analysis source inputs: {self:?}")
    }
}
impl std::error::Error for GoAnalysisSourceError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisSourceInputs {
    module: GoAnalysisModuleInputs,
    frontend_sources: BTreeMap<String, GoTypeSourceBuffer>,
}
impl GoAnalysisSourceInputs {
    pub fn module(&self) -> &GoAnalysisModuleInputs {
        &self.module
    }
    pub fn frontend_sources(&self) -> &BTreeMap<String, GoTypeSourceBuffer> {
        &self.frontend_sources
    }

    /// Original files supplied to Go's vendor-mode frontend build. A stable
    /// list of input bytes is not a claim that Go compiled every file, that a
    /// caller-supplied snapshot is authentic, or that any binary is registered.
    pub fn build_sources(&self) -> Vec<&GoTypeSourceBuffer> {
        let mut files = vec![
            self.module.manifest().original_source(),
            self.module.sums().original_source(),
            self.module
                .vendor()
                .correspondence()
                .manifest()
                .original_source(),
        ];
        files.extend(self.frontend_sources.values());
        files.extend(
            self.module
                .vendor()
                .correspondence()
                .supplied_files()
                .values(),
        );
        files.sort_by(|a, b| a.object().path.cmp(&b.object().path));
        files
    }
}

// Tokens cover string, rune, raw-string and operator spellings. Comments occur
// only in gaps. Ignore synthetic semicolons, whose positions may lie inside a
// multiline comment, and apply go/scanner's CR removal to line-comment text.
fn has_embed_comment(parsed: &GoParsedSource) -> Result<bool, GoAnalysisSourceError> {
    fn gap(mut text: &[u8]) -> Result<bool, GoAnalysisSourceError> {
        while !text.is_empty() {
            if text.starts_with(b"//") {
                let end = text.iter().position(|b| *b == b'\n').unwrap_or(text.len());
                let mut chars = text[2..end].iter().copied().filter(|b| *b != b'\r');
                if b"go:embed".iter().all(|b| chars.next() == Some(*b)) {
                    return Ok(true);
                }
                text = &text[end..];
            } else if text.starts_with(b"/*") {
                let end = text[2..]
                    .windows(2)
                    .position(|p| p == b"*/")
                    .ok_or(GoAnalysisSourceError::Namespace)?
                    + 4;
                text = &text[end..];
            } else {
                text = &text[1..];
            }
        }
        Ok(false)
    }
    let raw = parsed.source().bytes();
    let mut end = 0;
    for token in parsed.lexed().tokens() {
        if token.inserted() || token.kind() == "EOF" {
            continue;
        }
        if token.start() < end || token.end() < token.start() || token.end() > raw.len() {
            return Err(GoAnalysisSourceError::Namespace);
        }
        if gap(&raw[end..token.start()])? {
            return Ok(true);
        }
        end = token.end();
    }
    gap(&raw[end..])
}

fn check(source: &GoTypeSourceBuffer) -> Result<(), GoAnalysisSourceError> {
    let parsed = parse_go_source(source).map_err(|error| GoAnalysisSourceError::Syntax {
        path: source.object().path.clone(),
        error,
    })?;
    if has_embed_comment(&parsed)? {
        return Err(GoAnalysisSourceError::Embed {
            path: source.object().path.clone(),
        });
    }
    Ok(())
}

/// Read every observed non-test frontend Go file and every retained vendor Go
/// file, rejecting unmodeled native inputs and embed comments. No filesystem,
/// source-to-binary or semantic-context authority is conferred.
pub fn check_go_analysis_source_inputs(
    module: GoAnalysisModuleInputs,
) -> Result<GoAnalysisSourceInputs, GoAnalysisSourceError> {
    use GoAnalysisSourceError as E;
    let snapshot = module.vendor().snapshot();
    let original = module
        .vendor()
        .correspondence()
        .manifest()
        .original_source();
    let root = root_before(&original.object().path, "vendor/modules.txt").ok_or(E::Namespace)?;
    let capability = &original.object().capability;
    let entries = snapshot
        .read_directory(capability, root)
        .map_err(E::Snapshot)?;
    let mut frontend_sources = BTreeMap::new();
    for entry in entries {
        let name = entry.name_bytes();
        if NON_GO_INPUTS.iter().any(|suffix| name.ends_with(suffix)) {
            return Err(E::NonGoInput);
        }
        if name.ends_with(b".go") && !name.ends_with(b"_test.go") {
            let name = std::str::from_utf8(name).map_err(|_| E::Namespace)?;
            let path = if root.is_empty() {
                name.to_owned()
            } else {
                format!("{root}/{name}")
            };
            let source = snapshot
                .read_source(capability, &path, "go_file")
                .map_err(E::Snapshot)?
                .clone();
            check(&source)?;
            frontend_sources.insert(path, source);
        }
    }
    if frontend_sources.is_empty() {
        return Err(E::MissingFrontend);
    }
    for source in module
        .vendor()
        .correspondence()
        .supplied_files()
        .values()
        .filter(|s| s.object().kind == "go_file")
    {
        check(source)?;
    }
    Ok(GoAnalysisSourceInputs {
        module,
        frontend_sources,
    })
}
