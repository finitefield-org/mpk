//! Independent original module content checks. No resolver, host filesystem,
//! source adapters, network or checksum-database authentication enters here.
use super::*;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};
mod archive;
mod identity;
mod info;
pub use archive::{inspect_go_module_archive, GoModuleArchiveInventory, GoModuleArchiveMember};
pub use info::{validate_go_module_cache_info, GoModuleCacheInfoError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceModuleDataError {
    Source,
    Identity,
    Sums,
    Missing,
    Metadata,
    Manifest,
    Archive,
    Length,
    Crc,
    Namespace,
    Checksum,
    Limit,
}
impl std::fmt::Display for GoSourceModuleDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original module data: {self:?}")
    }
}
impl std::error::Error for GoSourceModuleDataError {}
use GoSourceModuleDataError as D;
type DataResult<T> = std::result::Result<T, D>;
fn need(ok: bool, error: D) -> DataResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn space(c: char) -> bool {
    matches!(c,'\t'..='\r'|' '| '\u{85}'|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}')
}
fn canonical_sum(s: &str) -> DataResult<()> {
    let value = s.strip_prefix("h1:").ok_or(D::Sums)?;
    let raw = BASE64.decode(value).map_err(|_| D::Sums)?;
    need(raw.len() == 32 && BASE64.encode(raw) == value, D::Sums)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoModuleSums {
    source: GoTypeSourceBuffer,
    entries: BTreeMap<(String, String), String>,
}
impl GoModuleSums {
    pub fn original_source(&self) -> &GoTypeSourceBuffer {
        &self.source
    }
    pub fn entries(&self) -> &BTreeMap<(String, String), String> {
        &self.entries
    }
    pub fn checksum(&self, path: &str, version: &str) -> Option<&str> {
        self.entries
            .get(&(path.into(), version.into()))
            .map(String::as_str)
    }
}
/// Preserve every original byte and parse all checksum lines, including unused
/// entries and identical duplicates. This is content consistency, not trust in
/// the source capability or the checksum database.
pub fn import_go_module_sums(source: GoTypeSourceBuffer) -> DataResult<GoModuleSums> {
    need(source.object().kind == "go_sum", D::Source)?;
    let text = std::str::from_utf8(source.bytes()).map_err(|_| D::Sums)?;
    let mut entries = BTreeMap::new();
    for line in text.split('\n') {
        let fields = line
            .split(space)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if fields.is_empty() {
            continue;
        }
        need(fields.len() == 3, D::Sums)?;
        let (path, version, sum) = (fields[0], fields[1], fields[2]);
        identity::check(path, version.strip_suffix("/go.mod").unwrap_or(version))?;
        canonical_sum(sum)?;
        if let Some(previous) =
            entries.insert((path.to_string(), version.to_string()), sum.to_string())
        {
            need(previous == sum, D::Sums)?;
        }
    }
    Ok(GoModuleSums { source, entries })
}
pub(in crate::go_practical_type_keys) fn manifest_sum(source: &GoTypeSourceBuffer) -> String {
    let line = format!("{}  go.mod\n", source.object().raw_sha256);
    format!("h1:{}", BASE64.encode(Sha256::digest(line.as_bytes())))
}
pub(super) fn graph_module_identity(path: &str, version: &str) -> bool {
    identity::valid(path, version)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModuleData {
    observations: GoSourceModuleObservations,
    sums: Option<GoModuleSums>,
    archives: BTreeMap<String, GoModuleArchiveInventory>,
}
impl GoSourceModuleData {
    pub fn module_observations(&self) -> &GoSourceModuleObservations {
        &self.observations
    }
    pub fn sums(&self) -> Option<&GoModuleSums> {
        self.sums.as_ref()
    }
    pub fn archives(&self) -> &BTreeMap<String, GoModuleArchiveInventory> {
        &self.archives
    }
}
impl GoSourceModuleObservations {
    /// Check original sums, every selected versioned module's complete archive,
    /// cached manifest/info/ziphash and all captured expanded source contents.
    /// General MVS, root/compiler/context and source/program authority stay due.
    pub fn verify_offline_module_data(self) -> DataResult<GoSourceModuleData> {
        let snapshot = self.exports.capture_inputs().snapshot_inputs();
        let contracts = self
            .exports
            .capture_inputs()
            .selection_inputs()
            .contract_inputs();
        let main = snapshot
            .source(contracts.main_source_object_id())
            .ok_or(D::Source)?;
        let object = main.object();
        let directory = parent(&object.path);
        let entries = snapshot
            .read_directory(&object.capability, directory)
            .map_err(|_| D::Missing)?;
        let sums = if entries.iter().any(|e| e.name_bytes() == b"go.sum") {
            let path = if directory.is_empty() {
                "go.sum".into()
            } else {
                format!("{directory}/go.sum")
            };
            Some(import_go_module_sums(
                snapshot
                    .read_source(&object.capability, &path, "go_sum")
                    .map_err(|_| D::Missing)?
                    .clone(),
            )?)
        } else {
            None
        };
        let rows = super::native_rows(&self.metadata).map_err(|_| D::Metadata)?;
        let mut remaining = 131072usize
            .checked_sub(snapshot.directory_entry_count())
            .ok_or(D::Limit)?;
        let mut archives = BTreeMap::new();
        for row in rows {
            if row.main {
                continue;
            }
            identity::check(&row.path, &row.version)?;
            let target = row.replace.as_deref().unwrap_or(&row);
            if target.version.is_empty() {
                continue;
            }
            identity::check(&target.path, &target.version)?;
            let sums = sums.as_ref().ok_or(D::Missing)?;
            let source = snapshot
                .source(&self.manifests.get(&row.path).ok_or(D::Missing)?.id)
                .ok_or(D::Missing)?;
            need(source.object().kind == "go_mod", D::Source)?;
            let checksum = manifest_sum(source);
            need(
                sums.checksum(&target.path, &format!("{}/go.mod", target.version))
                    == Some(checksum.as_str()),
                D::Checksum,
            )?;
            need(
                target.go_mod_sum.is_empty() || target.go_mod_sum == checksum,
                D::Checksum,
            )?;
            let prefix = source.object().path.strip_suffix(".mod").ok_or(D::Source)?;
            let read = |suffix: &str, kind: &str| {
                snapshot
                    .read_source(
                        &source.object().capability,
                        &format!("{prefix}{suffix}"),
                        kind,
                    )
                    .map_err(|_| D::Missing)
            };
            let info = read(".info", "metadata")?;
            validate_go_module_cache_info(info, &target.version).map_err(|_| D::Metadata)?;
            let ziphash = read(".ziphash", "metadata")?;
            let expected = sums
                .checksum(&target.path, &target.version)
                .ok_or(D::Missing)?;
            need(
                std::str::from_utf8(ziphash.bytes())
                    .map_err(|_| D::Metadata)?
                    .trim_matches(space)
                    == expected,
                D::Checksum,
            )?;
            need(target.sum.is_empty() || target.sum == expected, D::Checksum)?;
            let archive = inspect_go_module_archive(
                &target.path,
                &target.version,
                read(".zip", "auxiliary")?,
                expected,
                remaining,
            )?;
            remaining = remaining
                .checked_sub(archive.namespace_entries())
                .ok_or(D::Limit)?;
            if let Some(member) = archive.members.get("go.mod") {
                need(
                    member.raw_sha256 == source.object().raw_sha256
                        && member.bytes == source.bytes().len() as u64,
                    D::Manifest,
                )?;
            }
            // Cache paths are derived from the original manifest spelling and
            // native selected identity; neither host paths nor caller maps enter.
            let suffix = format!(
                "cache/download/{}/@v/{}.mod",
                escape(&target.path),
                escape(&target.version)
            );
            let base = source
                .object()
                .path
                .strip_suffix(&suffix)
                .ok_or(D::Source)?;
            let expanded = format!(
                "{base}{}@{}/",
                escape(&target.path),
                escape(&target.version)
            );
            for object in snapshot
                .source_objects()
                .iter()
                .filter(|o| o.capability == source.object().capability)
            {
                let Some(relative) = object.path.strip_prefix(&expanded) else {
                    continue;
                };
                let member = archive.members.get(relative).ok_or(D::Source)?;
                let raw = snapshot.source(&object.id).ok_or(D::Missing)?;
                need(
                    member.raw_sha256 == object.raw_sha256
                        && member.bytes == raw.bytes().len() as u64,
                    D::Checksum,
                )?;
            }
            archives.insert(row.path.clone(), archive);
        }
        Ok(GoSourceModuleData {
            observations: self,
            sums,
            archives,
        })
    }
}
