use crate::driver_protocol::DriverInputIdentity;
use crate::module_closure::ModuleClosure;
use crate::path::PortablePath;
use crate::source_capture::{SNAPSHOT_ENTRIES_MAX, SNAPSHOT_TOTAL_BYTES_MAX};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    InputCount,
    InputBytes,
    Path,
    FileType,
    CopyMismatch,
}

pub struct Snapshot {
    path: PathBuf,
    guard: platform::CleanupGuard,
    inputs: Vec<DriverInputIdentity>,
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Snapshot")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Snapshot {
    pub fn create(parent: &Path, closure: &ModuleClosure) -> Result<Self, SnapshotError> {
        closure
            .capture
            .validate_retained_root()
            .map_err(|_| SnapshotError::FileType)?;
        if closure.inputs.len() > SNAPSHOT_ENTRIES_MAX {
            return Err(SnapshotError::InputCount);
        }
        let total_bytes = closure.inputs.iter().try_fold(0_u64, |total, input| {
            total.checked_add(input.bytes.len() as u64)
        });
        if total_bytes.is_none_or(|total| total > SNAPSHOT_TOTAL_BYTES_MAX) {
            return Err(SnapshotError::InputBytes);
        }

        let serial = NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
        let name = format!("rust2vir-input-{}-{serial}", std::process::id());
        let (path, mut guard) = platform::CleanupGuard::create(parent, &name)?;
        for input in &closure.inputs {
            guard.copy_input(&input.normalized_path, &input.bytes, &input.sha256)?;
        }
        guard.seal()?;
        guard.validate_named_root()?;
        let inputs = closure
            .inputs
            .iter()
            .map(DriverInputIdentity::from_captured)
            .collect();
        Ok(Self {
            path,
            guard,
            inputs,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn input_path(&self, path: &PortablePath) -> PathBuf {
        self.path.join(path.as_str())
    }

    pub fn validate(&self) -> Result<(), SnapshotError> {
        self.guard.validate_named_root()
    }

    pub fn inputs(&self) -> &[DriverInputIdentity] {
        &self.inputs
    }

    pub(crate) fn try_clone_root(&self) -> Result<std::fs::File, SnapshotError> {
        self.guard.try_clone_root()
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        self.guard.cleanup();
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod platform {
    use super::SnapshotError;
    use crate::path::PortablePath;
    use crate::sha256::{digest, Sha256};
    use std::collections::BTreeMap;
    use std::ffi::{c_int, CString};
    use std::fs::{File, Metadata};
    use std::io::{self, Read, Seek, SeekFrom, Write};
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    const AT_FDCWD: c_int = -100;
    const AT_REMOVEDIR: c_int = 0x200;
    const O_RDONLY: c_int = 0;
    const O_RDWR: c_int = 2;
    const O_CREAT: c_int = 0o100;
    const O_EXCL: c_int = 0o200;
    const O_CLOEXEC: c_int = 0o2_000_000;
    const O_DIRECTORY: c_int = 0o200_000;
    const O_NOFOLLOW: c_int = 0o400_000;

    unsafe extern "C" {
        fn fchmod(fd: c_int, mode: u32) -> c_int;
        fn geteuid() -> u32;
        fn mkdirat(directory: c_int, path: *const i8, mode: u32) -> c_int;
        fn openat(directory: c_int, path: *const i8, flags: c_int, ...) -> c_int;
        fn unlinkat(directory: c_int, path: *const i8, flags: c_int) -> c_int;
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct NodeIdentity {
        device: u64,
        inode: u64,
        kind: NodeKind,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum NodeKind {
        Directory,
        File,
    }

    pub struct CleanupGuard {
        parent: File,
        root: File,
        name: CString,
        root_identity: NodeIdentity,
        directories: BTreeMap<String, NodeIdentity>,
        files: Vec<(PortablePath, NodeIdentity)>,
        cleaned: bool,
    }

    impl CleanupGuard {
        pub fn create(parent_path: &Path, name: &str) -> Result<(PathBuf, Self), SnapshotError> {
            let parent = open_directory_path(parent_path).map_err(|_| SnapshotError::FileType)?;
            if !crate::preflight::platform::supported_filesystem(parent.as_raw_fd())
                .map_err(|_| SnapshotError::FileType)?
            {
                return Err(SnapshotError::FileType);
            }
            let name = CString::new(name).map_err(|_| SnapshotError::Path)?;
            // SAFETY: `name` is NUL-terminated, `parent` is a retained directory, and mode is
            // supplied for this creation call.
            if unsafe { mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                return Err(SnapshotError::FileType);
            }
            let root = match openat_owned(
                parent.as_raw_fd(),
                name.as_bytes(),
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                None,
            ) {
                Ok(root) => root,
                Err(_) => {
                    let _ = unlink(parent.as_raw_fd(), name.as_bytes(), AT_REMOVEDIR);
                    return Err(SnapshotError::FileType);
                }
            };
            let metadata = match root.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    let _ = unlink(parent.as_raw_fd(), name.as_bytes(), AT_REMOVEDIR);
                    return Err(SnapshotError::FileType);
                }
            };
            // SAFETY: geteuid has no arguments or memory-safety preconditions.
            let effective_user = unsafe { geteuid() };
            if metadata.uid() != effective_user || metadata.mode() & 0o077 != 0 {
                let _ = unlink(parent.as_raw_fd(), name.as_bytes(), AT_REMOVEDIR);
                return Err(SnapshotError::FileType);
            }
            let root_identity = match directory_identity(&root) {
                Ok(identity) => identity,
                Err(_) => {
                    let _ = unlink(parent.as_raw_fd(), name.as_bytes(), AT_REMOVEDIR);
                    return Err(SnapshotError::FileType);
                }
            };
            Ok((
                parent_path.join(name.to_str().map_err(|_| SnapshotError::Path)?),
                Self {
                    parent,
                    root,
                    name,
                    root_identity,
                    directories: BTreeMap::new(),
                    files: Vec::new(),
                    cleaned: false,
                },
            ))
        }

        pub fn copy_input(
            &mut self,
            path: &PortablePath,
            bytes: &[u8],
            expected_hash: &[u8; 32],
        ) -> Result<(), SnapshotError> {
            if digest(bytes) != *expected_hash {
                return Err(SnapshotError::CopyMismatch);
            }
            let (directory, filename) = self.ensure_parent(path)?;
            let mut file = openat_owned(
                directory.as_raw_fd(),
                filename,
                O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                Some(0o600),
            )
            .map_err(|_| SnapshotError::FileType)?;
            let identity = file_identity(&file).map_err(|_| SnapshotError::FileType)?;
            self.files.push((path.clone(), identity));
            file.write_all(bytes)
                .map_err(|_| SnapshotError::CopyMismatch)?;
            file.seek(SeekFrom::Start(0))
                .map_err(|_| SnapshotError::CopyMismatch)?;
            let mut copied = Vec::with_capacity(bytes.len());
            let mut hasher = Sha256::new();
            let mut buffer = [0_u8; 16 * 1024];
            loop {
                let read = Read::by_ref(&mut file)
                    .take(bytes.len().saturating_add(1).saturating_sub(copied.len()) as u64)
                    .read(&mut buffer)
                    .map_err(|_| SnapshotError::CopyMismatch)?;
                if read == 0 {
                    break;
                }
                copied.extend_from_slice(&buffer[..read]);
                hasher.update(&buffer[..read]);
                if copied.len() > bytes.len() {
                    return Err(SnapshotError::CopyMismatch);
                }
            }
            if copied.as_slice() != bytes || hasher.finish() != *expected_hash {
                return Err(SnapshotError::CopyMismatch);
            }
            if file_identity(&file).map_err(|_| SnapshotError::FileType)? != identity
                || file.metadata().map_err(|_| SnapshotError::FileType)?.len() != bytes.len() as u64
            {
                return Err(SnapshotError::CopyMismatch);
            }
            chmod(&file, 0o400).map_err(|_| SnapshotError::FileType)?;
            Ok(())
        }

        pub fn seal(&mut self) -> Result<(), SnapshotError> {
            for path in self.directories.keys().rev() {
                let directory = open_relative_directory(self.root.as_raw_fd(), path.as_bytes())
                    .map_err(|_| SnapshotError::FileType)?;
                chmod(&directory, 0o500).map_err(|_| SnapshotError::FileType)?;
            }
            chmod(&self.root, 0o500).map_err(|_| SnapshotError::FileType)
        }

        pub fn validate_named_root(&self) -> Result<(), SnapshotError> {
            let named = openat_owned(
                self.parent.as_raw_fd(),
                self.name.as_bytes(),
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                None,
            )
            .map_err(|_| SnapshotError::FileType)?;
            if directory_identity(&named).map_err(|_| SnapshotError::FileType)?
                != self.root_identity
            {
                return Err(SnapshotError::FileType);
            }
            Ok(())
        }

        pub fn try_clone_root(&self) -> Result<File, SnapshotError> {
            self.validate_named_root()?;
            self.root.try_clone().map_err(|_| SnapshotError::FileType)
        }

        pub fn cleanup(&mut self) {
            if self.cleaned {
                return;
            }
            if directory_identity(&self.root).ok() != Some(self.root_identity) {
                return;
            }
            if chmod(&self.root, 0o700).is_err() {
                return;
            }
            for (path, expected) in &self.directories {
                let Ok(directory) = open_relative_directory(self.root.as_raw_fd(), path.as_bytes())
                else {
                    return;
                };
                if directory_identity(&directory).ok() != Some(*expected)
                    || chmod(&directory, 0o700).is_err()
                {
                    return;
                }
            }
            for (path, expected) in self.files.iter().rev() {
                let Ok((directory, filename)) = open_existing_parent(&self.root, path) else {
                    return;
                };
                let Ok(file) = openat_owned(
                    directory.as_raw_fd(),
                    filename,
                    O_RDONLY | O_CLOEXEC | O_NOFOLLOW,
                    None,
                ) else {
                    return;
                };
                if file_identity(&file).ok() != Some(*expected) {
                    return;
                }
                if unlink(directory.as_raw_fd(), filename, 0).is_err() {
                    return;
                }
            }
            for (path, expected) in self.directories.iter().rev() {
                let Ok((parent, name)) = open_existing_directory_parent(&self.root, path) else {
                    return;
                };
                let Ok(directory) = openat_owned(
                    parent.as_raw_fd(),
                    name,
                    O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                    None,
                ) else {
                    return;
                };
                if directory_identity(&directory).ok() != Some(*expected)
                    || chmod(&directory, 0o700).is_err()
                {
                    return;
                }
                if unlink(parent.as_raw_fd(), name, AT_REMOVEDIR).is_err() {
                    return;
                }
            }
            if directory_identity(&self.root).ok() != Some(self.root_identity) {
                return;
            }
            if self.validate_named_root().is_err() {
                return;
            }
            if unlink(self.parent.as_raw_fd(), self.name.as_bytes(), AT_REMOVEDIR).is_ok() {
                self.cleaned = true;
            }
        }

        fn ensure_parent<'a>(
            &mut self,
            path: &'a PortablePath,
        ) -> Result<(File, &'a [u8]), SnapshotError> {
            let mut components = path.as_str().split('/').collect::<Vec<_>>();
            let filename = components.pop().ok_or(SnapshotError::Path)?.as_bytes();
            let mut current =
                duplicate_directory(&self.root).map_err(|_| SnapshotError::FileType)?;
            let mut prefix = String::new();
            for component in components {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if !self.directories.contains_key(&prefix) {
                    let name = CString::new(component).map_err(|_| SnapshotError::Path)?;
                    // SAFETY: `name` is NUL-terminated, `current` is retained, and the root is
                    // private to this guard.
                    if unsafe { mkdirat(current.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                        return Err(SnapshotError::FileType);
                    }
                }
                let child = openat_owned(
                    current.as_raw_fd(),
                    component.as_bytes(),
                    O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                    None,
                )
                .map_err(|_| SnapshotError::FileType)?;
                let identity = directory_identity(&child).map_err(|_| SnapshotError::FileType)?;
                self.directories.entry(prefix.clone()).or_insert(identity);
                current = child;
            }
            Ok((current, filename))
        }
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            self.cleanup();
        }
    }

    fn open_existing_parent<'a>(
        root: &File,
        path: &'a PortablePath,
    ) -> io::Result<(File, &'a [u8])> {
        let mut components = path.as_str().split('/').collect::<Vec<_>>();
        let filename = components
            .pop()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let directory = open_components(root, &components)?;
        Ok((directory, filename.as_bytes()))
    }

    fn open_existing_directory_parent<'a>(
        root: &File,
        path: &'a str,
    ) -> io::Result<(File, &'a [u8])> {
        let mut components = path.split('/').collect::<Vec<_>>();
        let name = components
            .pop()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let directory = open_components(root, &components)?;
        Ok((directory, name.as_bytes()))
    }

    fn open_components(root: &File, components: &[&str]) -> io::Result<File> {
        let mut current = duplicate_directory(root)?;
        for component in components {
            current = openat_owned(
                current.as_raw_fd(),
                component.as_bytes(),
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                None,
            )?;
        }
        Ok(current)
    }

    fn open_relative_directory(root: RawFd, path: &[u8]) -> io::Result<File> {
        let mut current = openat_owned(
            root,
            b".",
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            None,
        )?;
        for component in path.split(|byte| *byte == b'/') {
            current = openat_owned(
                current.as_raw_fd(),
                component,
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                None,
            )?;
        }
        Ok(current)
    }

    fn duplicate_directory(directory: &File) -> io::Result<File> {
        openat_owned(
            directory.as_raw_fd(),
            b".",
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            None,
        )
    }

    fn open_directory_path(path: &Path) -> io::Result<File> {
        let initial = if path.is_absolute() { b"/" } else { b"." };
        let mut current = openat_owned(
            AT_FDCWD,
            initial,
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            None,
        )?;
        for component in path.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(name) => {
                    use std::os::unix::ffi::OsStrExt;
                    current = openat_owned(
                        current.as_raw_fd(),
                        name.as_bytes(),
                        O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
                        None,
                    )?;
                }
                Component::ParentDir | Component::Prefix(_) => {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
            }
        }
        Ok(current)
    }

    fn openat_owned(
        directory: RawFd,
        path: &[u8],
        flags: c_int,
        mode: Option<u32>,
    ) -> io::Result<File> {
        let path = CString::new(path).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        // SAFETY: `path` is NUL-terminated; creation calls supply a mode and other calls do not
        // require one.
        let descriptor = match mode {
            Some(mode) => unsafe { openat(directory, path.as_ptr(), flags, mode) },
            None => unsafe { openat(directory, path.as_ptr(), flags) },
        };
        if descriptor < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openat returned this descriptor and transfers its ownership.
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }

    fn unlink(directory: RawFd, path: &[u8], flags: c_int) -> io::Result<()> {
        let path = CString::new(path).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        // SAFETY: `path` is NUL-terminated and `directory` is a retained directory descriptor.
        if unsafe { unlinkat(directory, path.as_ptr(), flags) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn chmod(file: &File, mode: u32) -> io::Result<()> {
        // SAFETY: `file` owns a valid descriptor and mode contains only permission bits.
        if unsafe { fchmod(file.as_raw_fd(), mode) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn directory_identity(file: &File) -> io::Result<NodeIdentity> {
        identity(&file.metadata()?, NodeKind::Directory)
    }

    fn file_identity(file: &File) -> io::Result<NodeIdentity> {
        identity(&file.metadata()?, NodeKind::File)
    }

    fn identity(metadata: &Metadata, kind: NodeKind) -> io::Result<NodeIdentity> {
        let valid = match kind {
            NodeKind::Directory => metadata.is_dir(),
            NodeKind::File => metadata.is_file(),
        };
        if !valid {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        Ok(NodeIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            kind,
        })
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod platform {
    use super::SnapshotError;
    use crate::path::PortablePath;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    pub struct CleanupGuard;

    impl CleanupGuard {
        pub fn create(_parent: &Path, _name: &str) -> Result<(PathBuf, Self), SnapshotError> {
            Err(SnapshotError::FileType)
        }

        pub fn copy_input(
            &mut self,
            _path: &PortablePath,
            _bytes: &[u8],
            _expected_hash: &[u8; 32],
        ) -> Result<(), SnapshotError> {
            Err(SnapshotError::FileType)
        }

        pub fn seal(&mut self) -> Result<(), SnapshotError> {
            Err(SnapshotError::FileType)
        }

        pub fn validate_named_root(&self) -> Result<(), SnapshotError> {
            Err(SnapshotError::FileType)
        }

        pub fn try_clone_root(&self) -> Result<File, SnapshotError> {
            Err(SnapshotError::FileType)
        }

        pub fn cleanup(&mut self) {}
    }
}
