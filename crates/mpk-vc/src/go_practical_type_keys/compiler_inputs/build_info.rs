// Go 1.25 build-info decoding, adapted from debug/buildinfo and runtime/debug.
// Copyright 2018 The Go Authors. All rights reserved.
// See GO-BUILDINFO-LICENSE and the repository NOTICE for the BSD license.
//! Read facts from the actual owned executable, without executing it or trusting
//! producer metadata. Byte strings preserve Go's arbitrary string bytes.
mod formats;
mod text;

use super::{need, CompilerResult, GoSourceCompilerInputError as C};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GoExecutableModule {
    pub path: Vec<u8>,
    pub version: Vec<u8>,
    pub sum: Vec<u8>,
    pub replacement: Option<Box<GoExecutableModule>>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GoExecutableBuildInfo {
    pub go_version: Vec<u8>,
    pub path: Vec<u8>,
    pub main: GoExecutableModule,
    pub dependencies: Vec<GoExecutableModule>,
    /// Original order and duplicates are preserved. Binding rejects duplicate
    /// keys separately, including keys that the pinned compiler does not use.
    pub settings: Vec<(Vec<u8>, Vec<u8>)>,
}
impl GoExecutableBuildInfo {
    pub fn validate_compiler_tool(&self, tool: &str, host: &str) -> CompilerResult<()> {
        let (os, arch) = match host {
            "linux_amd64" => (b"linux".as_slice(), b"amd64".as_slice()),
            "darwin_arm64" => (b"darwin".as_slice(), b"arm64".as_slice()),
            _ => return Err(C::BuildInfo),
        };
        need(
            self.go_version == b"go1.25.0" && self.path == format!("cmd/{tool}").as_bytes(),
            C::BuildInfo,
        )?;
        let mut seen = BTreeSet::new();
        for (key, _) in &self.settings {
            need(seen.insert(key), C::BuildInfo)?;
        }
        let value = |key: &[u8]| {
            self.settings
                .iter()
                .find(|(k, _)| k == key)
                .map_or(b"".as_slice(), |(_, v)| v.as_slice())
        };
        for (key, expected) in [
            (b"GOOS".as_slice(), os),
            (b"GOARCH", arch),
            (b"CGO_ENABLED", b"0"),
            (b"-compiler", b"gc"),
            (b"-buildmode", b"exe"),
            (b"GOEXPERIMENT", b""),
        ] {
            need(value(key) == expected, C::BuildInfo)?;
        }
        need(arch != b"amd64" || value(b"GOAMD64") == b"v1", C::BuildInfo)
    }
}

pub fn read_go_executable_build_info(raw: &[u8]) -> CompilerResult<GoExecutableBuildInfo> {
    let image = formats::Image::parse(raw)?;
    let addr = image.search_magic()?;
    let header = image.exact(addr, 32)?;
    let (version, module) = if header[15] & 2 != 0 {
        let (version, next) = image.string(addr.checked_add(32).ok_or(C::BuildInfo)?)?;
        (version, image.string(next)?.0)
    } else {
        let width = header[14] as usize;
        need(matches!(width, 4 | 8), C::BuildInfo)?;
        let integer = |b: &[u8]| formats::number(b, 0, width, header[15] & 1 != 0);
        let string = |ptr: &[u8]| -> CompilerResult<Vec<u8>> {
            let h = image.exact(integer(ptr)?, (2 * width) as u64)?;
            Ok(image.exact(integer(h)?, integer(&h[width..])?)?.to_vec())
        };
        // Go's legacy reader permits an unreadable module string, but never an
        // unreadable/empty version. An absent module yields empty parsed facts.
        (
            string(&header[16..])?,
            string(&header[16 + width..]).unwrap_or_default(),
        )
    };
    need(!version.is_empty(), C::BuildInfo)?;
    let module = if module.len() >= 33 && module[module.len() - 17] == b'\n' {
        &module[16..module.len() - 16]
    } else {
        &[]
    };
    let mut info = text::parse(module)?;
    info.go_version = version;
    Ok(info)
}

/// Parse the module/settings text using the pinned runtime/debug grammar.
/// The executable reader alone adds GoVersion; a textual `go` line is ignored.
pub fn parse_go_executable_build_info_text(raw: &[u8]) -> CompilerResult<GoExecutableBuildInfo> {
    text::parse(raw)
}
