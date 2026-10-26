//! Original non-test resource selection from independently read Go patterns.
//! Snapshot import establishes byte/namespace consistency, not root authority.
use super::*;
mod directives;
mod glob;

const DIR: u32 = 1 << 31;
const LINK: u32 = 1 << 27;

fn decode(raw: &[u8]) -> (u32, usize) {
    let Some(&first) = raw.first() else {
        return (0xfffd, 0);
    };
    if first < 0x80 {
        return (first.into(), 1);
    }
    let n = match first {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return (0xfffd, 1),
    };
    match raw.get(..n).and_then(|b| std::str::from_utf8(b).ok()) {
        Some(s) => (s.chars().next().unwrap() as u32, n),
        None => (0xfffd, 1),
    }
}
pub(super) fn source_patterns(source: &GoTypeSourceBuffer) -> Vec<Vec<u8>> {
    directives::read(source.bytes())
}
fn bad_name(raw: &[u8]) -> bool {
    let Ok(name) = std::str::from_utf8(raw) else {
        return true;
    };
    if name.is_empty()
        || name.bytes().all(|b| b == b'.')
        || name.ends_with('.')
        || matches!(name, ".bzr" | ".hg" | ".git" | ".svn")
        || name.chars().any(|c| {
            if c.is_ascii() {
                !c.is_ascii_alphanumeric() && !"!#$%&()+,-.=@[]^_{}~ ".contains(c)
            } else {
                !validate::unicode::letter(c)
            }
        })
    {
        return true;
    }
    let short = name.split('.').next().unwrap();
    [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .any(|bad| short.eq_ignore_ascii_case(bad))
}
fn join(directory: &str, name: &str) -> String {
    if directory.is_empty() {
        name.into()
    } else {
        format!("{directory}/{name}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEmbeddedFiles {
    files: Vec<String>,
    objects: Vec<GoTypeSourceObject>,
    patterns: BTreeMap<String, Vec<GoTypeSourceObject>>,
}
impl GoSourceEmbeddedFiles {
    pub fn files(&self) -> &[String] {
        &self.files
    }
    pub fn source_objects(&self) -> &[GoTypeSourceObject] {
        &self.objects
    }
    pub fn pattern_sources(&self) -> &BTreeMap<String, Vec<GoTypeSourceObject>> {
        &self.patterns
    }
}
struct Resolver<'a> {
    snapshot: &'a GoSourceSnapshotInputs,
    capability: &'a str,
    directory: &'a str,
}
impl Resolver<'_> {
    fn error(&self, kind: GoSourcePackageErrorKind, relative: &str) -> GoSourcePackageError {
        GoSourcePackageError {
            kind,
            capability: self.capability.into(),
            path: join(self.directory, relative),
        }
    }
    fn directory(
        &self,
        relative: &str,
    ) -> std::result::Result<&[GoSourceSnapshotEntry], GoSourcePackageError> {
        self.snapshot
            .read_directory(self.capability, &join(self.directory, relative))
            .map_err(|_| self.error(GoSourcePackageErrorKind::EmbedNamespace, relative))
    }
    fn module_boundary(&self, relative: &str) -> std::result::Result<bool, GoSourcePackageError> {
        let entry = self
            .directory(relative)?
            .iter()
            .find(|e| e.name_bytes() == b"go.mod");
        if entry.is_some_and(|e| e.type_bits() & LINK != 0) {
            // A link name/type alone cannot prove whether native Stat succeeds.
            // Required missing target facts reject; no host stat is consulted.
            return Err(self.error(GoSourcePackageErrorKind::EmbedNamespace, relative));
        }
        Ok(entry.is_some())
    }
    fn source(
        &self,
        relative: &str,
    ) -> std::result::Result<GoTypeSourceObject, GoSourcePackageError> {
        let path = join(self.directory, relative);
        let object = self
            .snapshot
            .source_objects()
            .iter()
            .find(|s| s.capability == self.capability && s.path == path)
            .ok_or_else(|| self.error(GoSourcePackageErrorKind::EmbedSource, relative))?;
        let source = self
            .snapshot
            .read_source(self.capability, &path, &object.kind)
            .map_err(|_| self.error(GoSourcePackageErrorKind::EmbedSource, relative))?;
        Ok(source.object().clone())
    }
    fn glob(&self, pattern: &str) -> std::result::Result<Vec<(String, u32)>, GoSourcePackageError> {
        let mut current = vec![(String::new(), DIR)];
        for part in pattern.split('/') {
            let matcher = glob::Pattern::parse(part.as_bytes())
                .ok_or_else(|| self.error(GoSourcePackageErrorKind::EmbedPattern, pattern))?;
            let mut next = vec![];
            for (parent, kind) in current {
                if kind != DIR {
                    continue;
                }
                for entry in self.directory(&parent)? {
                    if matcher.matches(entry.name_bytes()) {
                        let name = std::str::from_utf8(entry.name_bytes()).map_err(|_| {
                            self.error(GoSourcePackageErrorKind::EmbedName, &parent)
                        })?;
                        next.push((join(&parent, name), entry.type_bits()));
                    }
                }
            }
            current = next;
        }
        Ok(current)
    }
    fn select(
        &self,
        pattern: &str,
    ) -> std::result::Result<BTreeMap<String, GoTypeSourceObject>, GoSourcePackageError> {
        let (pattern, all) = pattern
            .strip_prefix("all:")
            .map_or((pattern, false), |p| (p, true));
        if pattern == "."
            || pattern.is_empty()
            || pattern.starts_with('/')
            || pattern.split('/').any(|p| matches!(p, "" | "." | ".."))
            || glob::Pattern::parse(pattern.as_bytes()).is_none()
        {
            return Err(self.error(GoSourcePackageErrorKind::EmbedPattern, pattern));
        }
        let mut selected = BTreeMap::new();
        for (file, kind) in self.glob(pattern)? {
            let parts = file.split('/').collect::<Vec<_>>();
            for (at, name) in parts.iter().enumerate() {
                if bad_name(name.as_bytes()) {
                    return Err(self.error(GoSourcePackageErrorKind::EmbedName, &file));
                }
                if (at + 1 < parts.len() || kind == DIR)
                    && self.module_boundary(&parts[..=at].join("/"))?
                {
                    return Err(self.error(GoSourcePackageErrorKind::EmbedModule, &file));
                }
            }
            match kind {
                0 => {
                    selected.insert(file.clone(), self.source(&file)?);
                }
                DIR => {
                    let mut pending = vec![file.clone()];
                    let mut count = 0;
                    while let Some(directory) = pending.pop() {
                        if directory != file && self.module_boundary(&directory)? {
                            continue;
                        }
                        for entry in self.directory(&directory)? {
                            let name = entry.name_bytes();
                            let hidden = name.starts_with(b".") || name.starts_with(b"_");
                            let bad = bad_name(name);
                            if bad || hidden && !all {
                                if entry.type_bits() == DIR || hidden {
                                    continue;
                                }
                                return Err(
                                    self.error(GoSourcePackageErrorKind::EmbedName, &directory)
                                );
                            }
                            let name = std::str::from_utf8(name).unwrap();
                            let path = join(&directory, name);
                            if entry.type_bits() == DIR {
                                pending.push(path);
                            } else if entry.type_bits() == 0 {
                                count += 1;
                                selected.insert(path.clone(), self.source(&path)?);
                            }
                        }
                    }
                    if count == 0 {
                        return Err(self.error(GoSourcePackageErrorKind::EmbedEmpty, &file));
                    }
                }
                _ => return Err(self.error(GoSourcePackageErrorKind::EmbedIrregular, &file)),
            }
        }
        if selected.is_empty() {
            return Err(self.error(GoSourcePackageErrorKind::EmbedEmpty, pattern));
        }
        Ok(selected)
    }
}
pub(super) fn resources(
    snapshot: &GoSourceSnapshotInputs,
    package: &GoSourcePackageFiles,
) -> std::result::Result<GoSourceEmbeddedFiles, GoSourcePackageError> {
    let resolver = Resolver {
        snapshot,
        capability: package.capability(),
        directory: package.directory(),
    };
    let mut all = BTreeMap::new();
    let mut patterns = BTreeMap::new();
    for raw in package.embed_patterns() {
        let pattern = std::str::from_utf8(raw)
            .map_err(|_| resolver.error(GoSourcePackageErrorKind::EmbedPattern, ""))?;
        let selected = resolver.select(pattern)?;
        patterns.insert(pattern.into(), selected.values().cloned().collect());
        all.extend(selected);
    }
    Ok(GoSourceEmbeddedFiles {
        files: all.keys().cloned().collect(),
        objects: all.values().cloned().collect(),
        patterns,
    })
}
pub fn derive_go_source_embedded_files(
    snapshot: &GoSourceSnapshotInputs,
    capability: &str,
    directory: &str,
) -> std::result::Result<GoSourceEmbeddedFiles, GoSourcePackageError> {
    resources(
        snapshot,
        &derive_go_source_package_files(snapshot, capability, directory)?,
    )
}
