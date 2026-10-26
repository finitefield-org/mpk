//! Original strict go.mod facts for dependency-graph reconstruction. Parsing
//! neither selects module versions nor authenticates a source capability.
use super::*;
mod lax;
mod syntax;
mod versions;
pub use lax::{derive_go_source_graph_manifest, GoSourceGraphManifest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceModuleManifestError {
    Source,
    Syntax,
    Directive,
    Module,
    Version,
    CompilerRequirement,
}
impl std::fmt::Display for GoSourceModuleManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original module manifest: {self:?}")
    }
}
impl std::error::Error for GoSourceModuleManifestError {}
use GoSourceModuleManifestError as M;
type ManifestResult<T> = std::result::Result<T, M>;
fn need(ok: bool, e: M) -> ManifestResult<()> {
    if ok {
        Ok(())
    } else {
        Err(e)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleRequirement {
    pub path: Vec<u8>,
    pub version: String,
    pub indirect: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleReplacement {
    pub old_path: Vec<u8>,
    pub old_version: String,
    pub new_path: Vec<u8>,
    pub new_version: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleRetraction {
    pub low: Vec<u8>,
    pub high: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModuleManifest {
    source: GoTypeSourceBuffer,
    module: Vec<u8>,
    go: Option<String>,
    language: Option<String>,
    toolchain: Option<Vec<u8>>,
    requires: Vec<GoModuleRequirement>,
    excludes: Vec<GoModuleRequirement>,
    replacements: Vec<GoModuleReplacement>,
    retractions: Vec<GoModuleRetraction>,
}
impl GoSourceModuleManifest {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn module_path(&self) -> &[u8] {
        &self.module
    }
    pub fn go_requirement(&self) -> Option<&str> {
        self.go.as_deref()
    }
    pub fn language_version(&self) -> Option<&str> {
        self.language.as_deref()
    }
    pub fn toolchain_suggestion(&self) -> Option<&[u8]> {
        self.toolchain.as_deref()
    }
    pub fn requirements(&self) -> &[GoModuleRequirement] {
        &self.requires
    }
    pub fn exclusions(&self) -> &[GoModuleRequirement] {
        &self.excludes
    }
    pub fn replacements(&self) -> &[GoModuleReplacement] {
        &self.replacements
    }
    pub fn retractions(&self) -> &[GoModuleRetraction] {
        &self.retractions
    }
}
fn string(raw: &[u8]) -> ManifestResult<Vec<u8>> {
    if raw.starts_with(b"\"") {
        let mut normalized = Vec::new();
        let mut at = 0;
        while at < raw.len() {
            let (c, width) = syntax::rune(&raw[at..]);
            let mut buffer = [0; 4];
            normalized.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            at += width;
        }
        source_scopes::unquote(&normalized).map_err(|_| M::Syntax)
    } else {
        need(
            !raw.iter().any(|b| matches!(b, b'\'' | b'"' | b'`')),
            M::Syntax,
        )?;
        Ok(raw.to_vec())
    }
}
fn text(raw: &[u8]) -> ManifestResult<String> {
    String::from_utf8(raw.to_vec()).map_err(|_| M::Directive)
}
fn windows_name(raw: &[u8]) -> bool {
    let short = raw.split(|b| *b == b'.').next().unwrap();
    [
        b"CON".as_slice(),
        b"PRN",
        b"AUX",
        b"NUL",
        b"COM1",
        b"COM2",
        b"COM3",
        b"COM4",
        b"COM5",
        b"COM6",
        b"COM7",
        b"COM8",
        b"COM9",
        b"LPT1",
        b"LPT2",
        b"LPT3",
        b"LPT4",
        b"LPT5",
        b"LPT6",
        b"LPT7",
        b"LPT8",
        b"LPT9",
    ]
    .iter()
    .any(|bad| short.eq_ignore_ascii_case(bad))
}
fn import_path(raw: &[u8]) -> bool {
    !raw.is_empty()
        && !raw.starts_with(b"-")
        && raw.split(|b| *b == b'/').all(|s| {
            !s.is_empty()
                && !s.iter().all(|b| *b == b'.')
                && !s.ends_with(b".")
                && s.iter()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._~+".contains(b))
                && !windows_name(s)
                && !s
                    .split(|b| *b == b'.')
                    .next()
                    .unwrap()
                    .rsplit(|b| *b == b'~')
                    .next()
                    .is_some_and(|tail| {
                        s.split(|b| *b == b'.').next().unwrap().contains(&b'~')
                            && !tail.is_empty()
                            && tail.iter().all(u8::is_ascii_digit)
                    })
        })
}
fn directory(path: &[u8]) -> bool {
    matches!(path, b"." | b"..")
        || [b"./".as_slice(), b"../", b".\\", b"..\\", b"/", b"\\"]
            .iter()
            .any(|p| path.starts_with(p))
        || path.len() >= 2 && path[0].is_ascii_alphabetic() && path[1] == b':'
}
fn version(raw: &[u8]) -> ManifestResult<String> {
    versions::canonical(&string(raw)?)
}
/// Read all strict directive families accepted by the original practical
/// producer. Preserve source bytes, directive order and duplicate requirements.
/// Invalid-UTF8 comments and escaped arbitrary path bytes remain original data.
pub fn derive_go_source_module_manifest(
    source: GoTypeSourceBuffer,
    main: bool,
) -> ManifestResult<GoSourceModuleManifest> {
    need(source.object().kind == "go_mod", M::Source)?;
    let lines = syntax::lines(source.bytes())?;
    let mut out = GoSourceModuleManifest {
        source,
        module: vec![],
        go: None,
        language: None,
        toolchain: None,
        requires: vec![],
        excludes: vec![],
        replacements: vec![],
        retractions: vec![],
    };
    let mut has_module = false;
    for line in lines {
        let args = &line.arguments;
        match line.verb.as_slice() {
            b"module" => {
                need(!has_module && args.len() == 1, M::Directive)?;
                has_module = true;
                out.module = string(&args[0])?;
            }
            b"go" => {
                need(out.go.is_none() && args.len() == 1, M::Directive)?;
                let value = text(&args[0])?;
                out.language = Some(versions::language(&value, main)?);
                out.go = Some(value);
            }
            b"toolchain" => {
                need(out.toolchain.is_none() && args.len() == 1, M::Directive)?;
                versions::toolchain(&args[0], main)?;
                out.toolchain = Some(args[0].clone());
            }
            b"require" | b"exclude" => {
                need(args.len() == 2, M::Directive)?;
                let path = string(&args[0])?;
                let version = version(&args[1])?;
                versions::matches(&path, &version)?;
                let suffix = String::from_utf8_lossy(&line.suffix);
                let words = suffix
                    .split(syntax::space)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                let indirect = words == ["indirect"] || words.len() > 1 && words[0] == "indirect;";
                let row = GoModuleRequirement {
                    path,
                    version,
                    indirect: line.verb == b"require" && indirect,
                };
                if line.verb == b"require" {
                    out.requires.push(row)
                } else {
                    out.excludes.push(row)
                }
            }
            b"replace" => {
                let arrow = if args.len() >= 2 && args[1] == b"=>" {
                    1
                } else {
                    2
                };
                need(
                    args.len() >= arrow + 2 && args.len() <= arrow + 3 && args[arrow] == b"=>",
                    M::Directive,
                )?;
                let old_path = string(&args[0])?;
                versions::valid_path_major(&old_path)?;
                let old_version = if arrow == 2 {
                    let v = version(&args[1])?;
                    versions::matches(&old_path, &v)?;
                    v
                } else {
                    String::new()
                };
                let new_path = string(&args[arrow + 1])?;
                let new_version = if args.len() == arrow + 3 {
                    need(!directory(&new_path), M::Directive)?;
                    version(&args[arrow + 2])?
                } else {
                    need(
                        directory(&new_path) && !new_path.contains(&b'\\'),
                        M::Directive,
                    )?;
                    String::new()
                };
                out.replacements.push(GoModuleReplacement {
                    old_path,
                    old_version,
                    new_path,
                    new_version,
                });
            }
            b"retract" => {
                // The pinned parser's dontFixRetract callback intentionally
                // retains raw version spellings, including invalid semvers.
                let (low, high) = if args.len() == 1 && args[0] != b"(" && args[0] != b"[" {
                    let v = string(&args[0])?;
                    (v.clone(), v)
                } else {
                    need(
                        args.len() == 5 && args[0] == b"[" && args[2] == b"," && args[4] == b"]",
                        M::Directive,
                    )?;
                    (string(&args[1])?, string(&args[3])?)
                };
                out.retractions.push(GoModuleRetraction { low, high });
            }
            // The original practical producer has no frozen disposition for
            // nonempty godebug/tool/ignore directives. Empty blocks are syntax
            // with no semantic row and are retained by the source buffer.
            _ => return Err(M::Directive),
        }
    }
    need(
        has_module
            && import_path(&out.module)
            && ![b"main".as_slice(), b"all", b"std", b"cmd"].contains(&out.module.as_slice()),
        M::Module,
    )?;
    need(!main || out.go.is_some(), M::CompilerRequirement)?;
    Ok(out)
}
