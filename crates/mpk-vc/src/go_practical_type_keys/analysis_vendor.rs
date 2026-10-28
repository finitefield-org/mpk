//! Independent analysis-vendor manifest facts from the original owned bytes.
//! These facts provide no source-to-binary, installed-root or context authority.
use super::GoTypeSourceBuffer;
use std::collections::BTreeSet;
mod archives;
pub use archives::{
    check_go_analysis_vendor_archive_bytes, GoAnalysisVendorArchiveError,
    GoAnalysisVendorArchiveInputs, GoAnalysisVendorAssetDifference,
};

const MAXIMUM_MANIFEST_INPUTS: usize = 32_768;
const PINS: [(&str, &str); 3] = [
    ("golang.org/x/mod", "v0.27.0"),
    ("golang.org/x/sync", "v0.16.0"),
    ("golang.org/x/tools", "v0.36.0"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoAnalysisVendorError {
    Source,
    Manifest,
    Limit,
}

impl std::fmt::Display for GoAnalysisVendorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Go analysis vendor manifest: {self:?}")
    }
}

impl std::error::Error for GoAnalysisVendorError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoAnalysisVendorManifest {
    source: GoTypeSourceBuffer,
    packages: BTreeSet<String>,
}

impl GoAnalysisVendorManifest {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }

    pub fn package_paths(&self) -> &BTreeSet<String> {
        &self.packages
    }
}

// Go strings.TrimSpace/Fields use Unicode's White_Space property.
fn go_space(c: char) -> bool {
    matches!(c, '\t'..='\r' | ' ' | '\u{85}' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

// This follows the pinned x/mod CheckImportPath, including its actual handling
// of leading dots and the '+' import-path character.
fn import_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('-') {
        return false;
    }
    path.split('/').all(|element| {
        if element.is_empty()
            || element.bytes().all(|byte| byte == b'.')
            || element.ends_with('.')
            || !element
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+".contains(&byte))
        {
            return false;
        }
        let short = element.split('.').next().unwrap();
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .iter()
        .any(|bad| short.eq_ignore_ascii_case(bad))
        {
            return false;
        }
        !short.rsplit_once('~').is_some_and(|(_, suffix)| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
    })
}

/// Parse every original vendor-manifest byte and derive the exact package set.
/// The source is an independently owned untrusted input, not a registered build.
pub fn parse_go_analysis_vendor_manifest(
    source: GoTypeSourceBuffer,
) -> Result<GoAnalysisVendorManifest, GoAnalysisVendorError> {
    use GoAnalysisVendorError as E;
    if source.object().kind != "metadata" {
        return Err(E::Source);
    }
    let text = std::str::from_utf8(source.bytes()).map_err(|_| E::Manifest)?;
    let mut packages = BTreeSet::new();
    let mut modules = BTreeSet::new();
    let mut current = "";
    let mut annotated = false;
    let mut count = 0usize;
    for raw in text.split('\n') {
        let line = raw.trim_matches(go_space);
        if line.is_empty() {
            continue;
        }
        if line.starts_with("##") {
            if current.is_empty() || annotated || line != "## explicit; go 1.23.0" {
                return Err(E::Manifest);
            }
            annotated = true;
            continue;
        }
        if line.starts_with('#') {
            let fields = line
                .split(go_space)
                .filter(|field| !field.is_empty())
                .collect::<Vec<_>>();
            if (!current.is_empty() && (!annotated || count == 0))
                || fields.len() != 3
                || fields[0] != "#"
            {
                return Err(E::Manifest);
            }
            let Some((_, version)) = PINS.iter().find(|(path, _)| *path == fields[1]) else {
                return Err(E::Manifest);
            };
            if fields[2] != *version || !modules.insert(fields[1]) {
                return Err(E::Manifest);
            }
            current = fields[1];
            annotated = false;
            count = 0;
            continue;
        }
        if current.is_empty()
            || !annotated
            || packages.contains(line)
            || !import_path(line)
            || (line != current
                && !line
                    .strip_prefix(current)
                    .is_some_and(|tail| tail.starts_with('/')))
        {
            return Err(E::Manifest);
        }
        if packages.len() >= MAXIMUM_MANIFEST_INPUTS {
            return Err(E::Limit);
        }
        packages.insert(line.to_owned());
        count += 1;
    }
    if modules.len() != PINS.len() || !annotated || count == 0 {
        return Err(E::Manifest);
    }
    Ok(GoAnalysisVendorManifest { source, packages })
}
