//! Compare a Go snapshot with independently opened, descriptor-relative roots.
//!
//! This private observation does not authenticate which roots a launcher should
//! authorize, a prior capture epoch, a compiler, or any program theorem.
use mpk_vc::go_practical_type_keys::GoSourceSnapshotInputs;
use rustix::fs::{openat2, statat, AtFlags, FileType, Mode, OFlags, RawDir, ResolveFlags, CWD};
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::io::Read;
use std::mem::MaybeUninit;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoSourceRootError {
    Capabilities,
    Path,
    Filesystem,
    Namespace,
    Source,
    Changed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableMetadata {
    device: u64,
    inode: u64,
    mode: u32,
    links: u64,
    bytes: u64,
    mtime: i64,
    mtime_nsec: i64,
    ctime: i64,
    ctime_nsec: i64,
}

impl StableMetadata {
    fn of(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            links: metadata.nlink(),
            bytes: metadata.len(),
            mtime: metadata.mtime(),
            mtime_nsec: metadata.mtime_nsec(),
            ctime: metadata.ctime(),
            ctime_nsec: metadata.ctime_nsec(),
        }
    }
}

struct Root {
    directory: File,
    opened: StableMetadata,
}

/// Root handles are opened before comparison and retained until the returned
/// owner is dropped. No snapshot hash or file list opens a root by itself.
pub(crate) struct GoSourceRoots {
    roots: BTreeMap<String, Root>,
}

/// Owned data matched against the held roots during two complete passes. This
/// does not authorize these roots or prove that an earlier capture used them.
pub(crate) struct GoSourceRootsMatch {
    _roots: GoSourceRoots,
    snapshot: GoSourceSnapshotInputs,
}

impl GoSourceRootsMatch {
    pub(crate) fn snapshot(&self) -> &GoSourceSnapshotInputs {
        &self.snapshot
    }
}

fn normalized_root(path: &Path) -> bool {
    let mut rebuilt = PathBuf::new();
    for component in path.components() {
        if !matches!(component, Component::RootDir | Component::Normal(_)) {
            return false;
        }
        rebuilt.push(component.as_os_str());
    }
    path.is_absolute()
        && path.parent().is_some_and(|parent| parent != path)
        && path.as_os_str() == rebuilt.as_os_str()
}

impl GoSourceRoots {
    pub(crate) fn open(bindings: Vec<(String, PathBuf)>) -> Result<Self, GoSourceRootError> {
        use GoSourceRootError as E;
        let mut roots = BTreeMap::new();
        if bindings.is_empty() {
            return Err(E::Capabilities);
        }
        for (capability, path) in bindings {
            if !normalized_root(&path) {
                return Err(E::Path);
            }
            let fd = openat2(
                CWD,
                &path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
                Mode::empty(),
                ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
            )
            .map_err(|_| E::Filesystem)?;
            let directory = File::from(fd);
            let metadata = directory.metadata().map_err(|_| E::Filesystem)?;
            if !metadata.is_dir() {
                return Err(E::Filesystem);
            }
            if roots
                .insert(
                    capability,
                    Root {
                        directory,
                        opened: StableMetadata::of(&metadata),
                    },
                )
                .is_some()
            {
                return Err(E::Capabilities);
            }
        }
        Ok(Self { roots })
    }

    pub(crate) fn compare(
        self,
        snapshot: GoSourceSnapshotInputs,
    ) -> Result<GoSourceRootsMatch, GoSourceRootError> {
        self.compare_between(snapshot, || {})
    }

    fn compare_between(
        self,
        snapshot: GoSourceSnapshotInputs,
        between: impl FnOnce(),
    ) -> Result<GoSourceRootsMatch, GoSourceRootError> {
        use GoSourceRootError as E;
        if !self
            .roots
            .keys()
            .map(String::as_str)
            .eq(snapshot.capabilities().iter().map(String::as_str))
        {
            return Err(E::Capabilities);
        }
        let before = self.observe(&snapshot)?;
        between();
        let after = self.observe(&snapshot)?;
        if before != after {
            return Err(E::Changed);
        }
        Ok(GoSourceRootsMatch {
            _roots: self,
            snapshot,
        })
    }

    #[cfg(test)]
    pub(crate) fn compare_with_hook(
        self,
        snapshot: GoSourceSnapshotInputs,
        between: impl FnOnce(),
    ) -> Result<GoSourceRootsMatch, GoSourceRootError> {
        self.compare_between(snapshot, between)
    }

    fn observe(
        &self,
        snapshot: &GoSourceSnapshotInputs,
    ) -> Result<BTreeMap<(String, u8, String), StableMetadata>, GoSourceRootError> {
        use GoSourceRootError as E;
        let mut observed = BTreeMap::new();
        for (capability, root) in &self.roots {
            let actual = root.directory.metadata().map_err(|_| E::Filesystem)?;
            let stable = StableMetadata::of(&actual);
            if stable != root.opened {
                return Err(E::Changed);
            }
            observed.insert((capability.clone(), 0, String::new()), stable);
        }
        for (capability, path, entries) in snapshot.observed_directories() {
            let root = self.roots.get(capability).ok_or(E::Capabilities)?;
            let directory = open_relative(&root.directory, path, true)?;
            let before = directory.metadata().map_err(|_| E::Filesystem)?;
            if !before.is_dir() {
                return Err(E::Namespace);
            }
            let actual = directory_entries(&directory, entries.len())?;
            if actual.len() != entries.len()
                || actual.iter().zip(entries).any(|((name, typ), entry)| {
                    name.as_slice() != entry.name_bytes() || *typ != entry.type_bits()
                })
            {
                return Err(E::Namespace);
            }
            let stable = StableMetadata::of(&before);
            if stable != StableMetadata::of(&directory.metadata().map_err(|_| E::Filesystem)?) {
                return Err(E::Changed);
            }
            observed.insert((capability.to_owned(), 1, path.to_owned()), stable);
        }
        for object in snapshot.source_objects() {
            let root = self.roots.get(&object.capability).ok_or(E::Capabilities)?;
            let source = snapshot.source(&object.id).ok_or(E::Source)?;
            let mut file = open_relative(&root.directory, &object.path, false)?;
            let before = file.metadata().map_err(|_| E::Filesystem)?;
            if !before.is_file() || before.len() != source.bytes().len() as u64 {
                return Err(E::Source);
            }
            let mut buffer = [0u8; 65536];
            for expected in source.bytes().chunks(buffer.len()) {
                file.read_exact(&mut buffer[..expected.len()])
                    .map_err(|_| E::Filesystem)?;
                if &buffer[..expected.len()] != expected {
                    return Err(E::Source);
                }
            }
            if file.read(&mut buffer[..1]).map_err(|_| E::Filesystem)? != 0 {
                return Err(E::Source);
            }
            let stable = StableMetadata::of(&before);
            if stable != StableMetadata::of(&file.metadata().map_err(|_| E::Filesystem)?) {
                return Err(E::Changed);
            }
            observed.insert((object.capability.clone(), 2, object.path.clone()), stable);
        }
        Ok(observed)
    }
}

fn open_relative(root: &File, path: &str, directory: bool) -> Result<File, GoSourceRootError> {
    let mut flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
    if directory {
        flags |= OFlags::DIRECTORY;
    }
    let descriptor = openat2(
        root,
        if path.is_empty() { "." } else { path },
        flags,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(|_| GoSourceRootError::Filesystem)?;
    Ok(File::from(descriptor))
}

// Match Go 1.25's Linux dirent type bits. DT_UNKNOWN requires no-follow stat,
// just as os.ReadDir does. Native name bytes are never decoded as UTF-8.
fn directory_entries(
    directory: &File,
    maximum: usize,
) -> Result<Vec<(Vec<u8>, u32)>, GoSourceRootError> {
    use GoSourceRootError as E;
    let mut storage = [MaybeUninit::uninit(); 65536];
    let mut reader = RawDir::new(directory, &mut storage);
    let mut entries = Vec::new();
    while let Some(entry) = reader.next() {
        let entry = entry.map_err(|_| E::Filesystem)?;
        let name = entry.file_name();
        if matches!(name.to_bytes(), b"." | b"..") {
            continue;
        }
        if entries.len() == maximum {
            return Err(E::Namespace);
        }
        let typ = match entry.file_type() {
            FileType::Unknown => {
                let stat = statat(directory, name, AtFlags::SYMLINK_NOFOLLOW)
                    .map_err(|_| E::Filesystem)?;
                FileType::from_raw_mode(stat.st_mode)
            }
            typ => typ,
        };
        let bits = match typ {
            FileType::RegularFile | FileType::Unknown => 0,
            FileType::Directory => 1 << 31,
            FileType::Symlink => 1 << 27,
            FileType::BlockDevice => 1 << 26,
            FileType::CharacterDevice => (1 << 26) | (1 << 21),
            FileType::Fifo => 1 << 25,
            FileType::Socket => 1 << 24,
        };
        entries.push((name.to_bytes().to_vec(), bits));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(E::Namespace);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;
    use std::os::unix::net::UnixListener;

    #[test]
    fn native_names_and_types_are_compared_without_utf8_conversion() {
        let temp = tempfile::tempdir().unwrap();
        let file_name = OsString::from_vec(vec![0xfe]);
        let directory_name = OsString::from_vec(vec![0xff]);
        std::fs::write(temp.path().join(&file_name), b"original byte name").unwrap();
        std::fs::create_dir(temp.path().join(&directory_name)).unwrap();
        symlink(&file_name, temp.path().join("link")).unwrap();
        let directory = File::open(temp.path()).unwrap();
        rustix::fs::mkfifoat(&directory, "fifo", Mode::RUSR | Mode::WUSR).unwrap();
        let _listener = UnixListener::bind(temp.path().join("socket")).unwrap();
        let entries = directory_entries(&directory, 5).unwrap();
        assert_eq!(
            entries,
            vec![
                (b"fifo".to_vec(), 1 << 25),
                (b"link".to_vec(), 1 << 27),
                (b"socket".to_vec(), 1 << 24),
                (vec![0xfe], 0),
                (vec![0xff], 1 << 31)
            ]
        );
        let directory = File::open(temp.path()).unwrap();
        assert!(matches!(
            directory_entries(&directory, 4),
            Err(GoSourceRootError::Namespace)
        ));
    }
}
