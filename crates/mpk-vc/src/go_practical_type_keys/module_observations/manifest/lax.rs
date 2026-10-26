//! Original dependency go.mod facts with the pinned ParseLax disposition.
//! Missing/unchecked module declarations and ignored main-only directives do
//! not acquire selected-module, replacement, root or version-selection authority.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceGraphManifest {
    source: GoTypeSourceBuffer,
    module: Option<Vec<u8>>,
    go: Option<String>,
    requires: Vec<GoModuleRequirement>,
    retractions: Vec<GoModuleRetraction>,
    ignores: Vec<Vec<u8>>,
}
impl GoSourceGraphManifest {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn module_path(&self) -> Option<&[u8]> {
        self.module.as_deref()
    }
    pub fn go_requirement(&self) -> Option<&str> {
        self.go.as_deref()
    }
    pub fn requirements(&self) -> &[GoModuleRequirement] {
        &self.requires
    }
    pub fn retractions(&self) -> &[GoModuleRetraction] {
        &self.retractions
    }
    pub fn ignored_paths(&self) -> &[Vec<u8>] {
        &self.ignores
    }
    pub fn supports_graph_pruning(&self) -> bool {
        self.go
            .as_deref()
            .is_some_and(|v| !versions::compare_go(v, "1.17").is_lt())
    }
    pub fn implicit_go_requirement(&self) -> Option<&str> {
        self.go
            .as_deref()
            .filter(|v| !versions::compare_go(v, "1.21").is_lt())
    }
    pub fn requires_newer_compiler(&self) -> bool {
        self.go
            .as_deref()
            .is_some_and(|v| versions::compare_go(v, "1.25.0").is_gt())
    }
}

fn component(raw: &[u8]) -> Option<(&[u8], &[u8])> {
    let n = raw.iter().take_while(|b| b.is_ascii_digit()).count();
    if n == 0 || n > 1 && raw[0] == b'0' {
        None
    } else {
        Some((&raw[..n], &raw[n..]))
    }
}
// Modfile's regexp admits patch+prerelease and zero-prefixed prerelease digits
// even where gover's semantic version parser does not. Keep these layers apart.
fn go_spelling(raw: &[u8]) -> bool {
    let Some((major, rest)) = component(raw) else {
        return false;
    };
    if major == b"0" {
        return false;
    }
    let Some(rest) = rest.strip_prefix(b".") else {
        return false;
    };
    let Some((_, mut rest)) = component(rest) else {
        return false;
    };
    if let Some(patch) = rest.strip_prefix(b".") {
        let Some((_, tail)) = component(patch) else {
            return false;
        };
        rest = tail;
    }
    if rest.is_empty() {
        return true;
    }
    let letters = rest.iter().take_while(|b| b.is_ascii_lowercase()).count();
    letters > 0 && letters < rest.len() && rest[letters..].iter().all(u8::is_ascii_digit)
}
fn go_requirement(raw: &[u8]) -> ManifestResult<String> {
    if go_spelling(raw) {
        return text(raw);
    }
    // Native laxGoVersionRE normalizes only a valid major.minor prefix followed
    // by a non-digit suffix. Its optional v and arbitrary suffix bytes stay in
    // the owned original source; the normalized fact is ASCII major.minor.
    let raw = raw.strip_prefix(b"v").unwrap_or(raw);
    let (major, rest) = component(raw).ok_or(M::Version)?;
    need(major != b"0", M::Version)?;
    let rest = rest.strip_prefix(b".").ok_or(M::Version)?;
    let (minor, suffix) = component(rest).ok_or(M::Version)?;
    need(
        !suffix.is_empty() && !suffix[0].is_ascii_digit() && !suffix.contains(&b'\n'),
        M::Version,
    )?;
    Ok(format!("{}.{}", text(major)?, text(minor)?))
}
fn retract(args: &[Vec<u8>]) -> ManifestResult<GoModuleRetraction> {
    let first = args.first().ok_or(M::Directive)?;
    need(first != b"(", M::Directive)?;
    let (low, high) = if first == b"[" {
        need(
            args.len() >= 5 && args[2] == b"," && args[4] == b"]",
            M::Directive,
        )?;
        (string(&args[1])?, string(&args[3])?)
    } else {
        let value = string(first)?;
        (value.clone(), value)
    };
    Ok(GoModuleRetraction { low, high })
}
/// Derive the complete dependency parser projection from original go.mod bytes.
/// This preserves native lax normalization, order/duplicates, byte paths and
/// ignored directives without imposing the selected-module admission policy.
pub fn derive_go_source_graph_manifest(
    source: GoTypeSourceBuffer,
) -> ManifestResult<GoSourceGraphManifest> {
    need(source.object().kind == "go_mod", M::Source)?;
    let lines = syntax::lax_lines(source.bytes())?;
    let mut out = GoSourceGraphManifest {
        source,
        module: None,
        go: None,
        requires: vec![],
        retractions: vec![],
        ignores: vec![],
    };
    for line in lines {
        let args = &line.arguments;
        match line.verb.as_slice() {
            b"module" => {
                need(out.module.is_none() && args.len() == 1, M::Directive)?;
                out.module = Some(string(&args[0])?);
            }
            b"go" => {
                need(out.go.is_none() && args.len() == 1, M::Directive)?;
                out.go = Some(go_requirement(&args[0])?);
            }
            b"require" => {
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
                out.requires.push(GoModuleRequirement {
                    path,
                    version,
                    indirect,
                });
            }
            b"ignore" => {
                need(args.len() == 1, M::Directive)?;
                out.ignores.push(string(&args[0])?);
            }
            b"retract" => {
                // Lax parsing skips unsupported/invalid interval syntax and
                // accepts additional trailing fields, retaining original bytes.
                if let Ok(row) = retract(args) {
                    out.retractions.push(row)
                }
            }
            _ => {}
        }
    }
    Ok(out)
}
