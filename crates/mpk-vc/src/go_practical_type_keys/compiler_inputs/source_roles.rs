//! Exact original SDK source roles and complete package-directory membership.
//! These facts do not authenticate an installed SDK or prove library bodies.
use super::*;

pub fn derive_go_compiler_source_role(source: &GoTypeSourceBuffer) -> CompilerResult<&'static str> {
    let path = source.object().path.as_str();
    if !path.ends_with(".go") {
        return Ok(match path.rsplit('/').next().unwrap() {
            "go.mod" => "go_mod",
            "go.sum" => "go_sum",
            _ if path.ends_with("/vendor/modules.txt") => "metadata",
            _ => "auxiliary",
        });
    }
    need(
        matches!(source.object().kind.as_str(), "go_file" | "excluded_source"),
        C::SourceRole,
    )?;
    // Native MatchFile ignores a filename mismatch without reading bytes. For
    // matching names it reads the import header before applying constraints;
    // unexpected NUL/read errors therefore precede a false build expression.
    if !source_build::compiler_filename_matches(source) {
        return Ok("excluded_source");
    }
    source_package::compiler_header_read(source).map_err(|_| C::SourceRole)?;
    let matched = derive_go_source_build_match(source).map_err(|_| C::SourceRole)?;
    if !matched.matches_target() || matched.is_test_file() {
        return Ok("excluded_source");
    }
    // Binary-only comments are not a MatchFile exclusion. Invalid Go syntax is
    // retained as go_file for the actual checker, just as in the native owner.
    let Some((documentation, cgo)) = source_package::compiler_role_header(source) else {
        return Ok("go_file");
    };
    Ok(if documentation || cgo {
        "excluded_source"
    } else {
        "go_file"
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoCompilerSourceIndex {
    roles: BTreeMap<String, String>,
    packages: BTreeSet<String>,
    module: GoSourceStrictModuleManifest,
    go_version: String,
}
impl GoCompilerSourceIndex {
    pub fn source_roles(&self) -> &BTreeMap<String, String> {
        &self.roles
    }
    /// Every original directory containing a .go file, including excluded/test
    /// files. This is source membership, not target package/body admission.
    pub fn package_paths(&self) -> &BTreeSet<String> {
        &self.packages
    }
    pub fn module_source_object_id(&self) -> &str {
        &self.module.original_source().object().id
    }
    pub fn module_manifest(&self) -> &GoSourceStrictModuleManifest {
        &self.module
    }
    pub fn go_version(&self) -> &str {
        &self.go_version
    }

    pub(super) fn derive(
        snapshot: &GoSourceSnapshotInputs,
        sources: &BTreeMap<String, GoTypeSourceObject>,
    ) -> CompilerResult<Self> {
        let mut roles = BTreeMap::new();
        let mut packages = BTreeSet::new();
        for (path, object) in sources {
            let source = snapshot.source(&object.id).ok_or(C::Source)?;
            let role = derive_go_compiler_source_role(source)?;
            need(object.kind == role, C::SourceRole)?;
            roles.insert(path.clone(), role.into());
            if let Some(path) = path.strip_prefix("go/src/").filter(|s| s.ends_with(".go")) {
                if let Some((directory, _)) = path.rsplit_once('/') {
                    packages.insert(directory.into());
                }
            }
        }
        let std = sources.get("go/src/go.mod").ok_or(C::Source)?;
        let parsed = derive_go_source_strict_module_manifest(
            snapshot.source(&std.id).ok_or(C::Source)?.clone(),
        )
        .map_err(|_| C::StandardModule)?;
        let version = parsed
            .standard_library_go_requirement()
            .map_err(|_| C::StandardModule)?;
        let go_version = format!("go{version}");
        Ok(Self {
            roles,
            packages,
            module: parsed,
            go_version,
        })
    }
}
