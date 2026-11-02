//! General pinned modfile.Parse facts, separate from application admission and
//! dependency ParseLax. Original arbitrary bytes, ordering and duplicates survive.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleDebugSetting {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceStrictModuleManifest {
    source: GoTypeSourceBuffer,
    module: Option<Vec<u8>>,
    go: Option<String>,
    toolchain: Option<Vec<u8>>,
    requires: Vec<GoModuleRequirement>,
    excludes: Vec<GoModuleRequirement>,
    replacements: Vec<GoModuleReplacement>,
    retractions: Vec<GoModuleRetraction>,
    godebug: Vec<GoModuleDebugSetting>,
    tools: Vec<Vec<u8>>,
    ignores: Vec<Vec<u8>>,
}
impl GoSourceStrictModuleManifest {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn module_path(&self) -> Option<&[u8]> {
        self.module.as_deref()
    }
    pub fn go_requirement(&self) -> Option<&str> {
        self.go.as_deref()
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
    pub fn debug_settings(&self) -> &[GoModuleDebugSetting] {
        &self.godebug
    }
    pub fn tools(&self) -> &[Vec<u8>] {
        &self.tools
    }
    pub fn ignored_paths(&self) -> &[Vec<u8>] {
        &self.ignores
    }
    /// The native SDK owner requires std, an explicit Go directive and no newer
    /// compiler. A toolchain suggestion neither selects nor invokes a compiler.
    pub fn standard_library_go_requirement(&self) -> ManifestResult<&str> {
        need(self.module_path() == Some(b"std".as_slice()), M::Module)?;
        let value = self.go_requirement().ok_or(M::CompilerRequirement)?;
        need(
            !versions::compare_go(value, "1.25.0").is_gt(),
            M::CompilerRequirement,
        )?;
        Ok(value)
    }
}

pub fn derive_go_source_strict_module_manifest(
    source: GoTypeSourceBuffer,
) -> ManifestResult<GoSourceStrictModuleManifest> {
    need(source.object().kind == "go_mod", M::Source)?;
    let lines = syntax::lines(source.bytes())?;
    let mut out = GoSourceStrictModuleManifest {
        source,
        module: None,
        go: None,
        toolchain: None,
        requires: vec![],
        excludes: vec![],
        replacements: vec![],
        retractions: vec![],
        godebug: vec![],
        tools: vec![],
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
                need(lax::go_spelling(&args[0]), M::Version)?;
                out.go = Some(text(&args[0])?);
            }
            b"toolchain" => {
                need(out.toolchain.is_none() && args.len() == 1, M::Directive)?;
                need(
                    args[0] == b"default" || args[0] == b"go1" || args[0].starts_with(b"go1."),
                    M::Version,
                )?;
                out.toolchain = Some(args[0].clone());
            }
            b"godebug" => {
                need(
                    args.len() == 1 && !args[0].iter().any(|b| b"\"`',".contains(b)),
                    M::Directive,
                )?;
                let at = args[0]
                    .iter()
                    .position(|&b| b == b'=')
                    .ok_or(M::Directive)?;
                out.godebug.push(GoModuleDebugSetting {
                    key: args[0][..at].to_vec(),
                    value: args[0][at + 1..].to_vec(),
                });
            }
            b"tool" | b"ignore" => {
                need(args.len() == 1, M::Directive)?;
                let value = string(&args[0])?;
                if line.verb == b"tool" {
                    out.tools.push(value);
                } else {
                    out.ignores.push(value);
                }
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
                let row = GoModuleRequirement {
                    path,
                    version,
                    indirect: line.verb == b"require"
                        && (words == ["indirect"] || words.len() > 1 && words[0] == "indirect;"),
                };
                if line.verb == b"require" {
                    out.requires.push(row);
                } else {
                    out.excludes.push(row);
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
                // Parse(..., nil) deliberately leaves dontFixRetract spellings
                // intact. It does not require a module directive or normalize
                // these strings through the dependency-version canonicalizer.
                let (low, high) = if args.len() == 1 && args[0] != b"(" && args[0] != b"[" {
                    let value = string(&args[0])?;
                    (value.clone(), value)
                } else {
                    need(
                        args.len() == 5 && args[0] == b"[" && args[2] == b"," && args[4] == b"]",
                        M::Directive,
                    )?;
                    (string(&args[1])?, string(&args[3])?)
                };
                out.retractions.push(GoModuleRetraction { low, high });
            }
            _ => return Err(M::Directive),
        }
    }
    Ok(out)
}
