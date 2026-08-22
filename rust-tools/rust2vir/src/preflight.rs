use crate::cli::LowerRequest;
use crate::path::{PortablePath, PortablePathError, PortablePathSet};
use crate::source_capture::{CaptureFailure, CaptureState};

pub use crate::source_capture::{CapturedInput as StructuralInput, InputKind};

const MANIFEST_BYTES_MAX: u64 = 1_048_576;
const LOCKFILE_BYTES_MAX: u64 = 4_194_304;
const CONTRACT_FILES_MAX: usize = 128;
const CONTRACT_FILE_BYTES_MAX: u64 = 1_048_576;
const CONTRACT_TOTAL_BYTES_MAX: u64 = 8_388_608;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreflightCode {
    LimitInputBytes,
    LimitInputCount,
    LimitPath,
    LimitContract,
    FileType,
    Path,
    Workspace,
    Config,
    ToolchainFile,
}

impl PreflightCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LimitInputBytes => "RUST_LIMIT_INPUT_BYTES",
            Self::LimitInputCount => "RUST_LIMIT_INPUT_COUNT",
            Self::LimitPath => "RUST_LIMIT_PATH",
            Self::LimitContract => "RUST_LIMIT_CONTRACT",
            Self::FileType => "RUST_PREFLIGHT_FILE_TYPE",
            Self::Path => "RUST_PREFLIGHT_PATH",
            Self::Workspace => "RUST_PREFLIGHT_WORKSPACE",
            Self::Config => "RUST_PREFLIGHT_CONFIG",
            Self::ToolchainFile => "RUST_PREFLIGHT_TOOLCHAIN_FILE",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::LimitInputBytes => "input byte limit exceeded",
            Self::LimitInputCount => "input count limit exceeded",
            Self::LimitPath => "normalized path limit exceeded",
            Self::LimitContract => "contract input limit exceeded",
            Self::FileType => "input file type is not permitted",
            Self::Path => "input path is not portable and contained",
            Self::Workspace => "Cargo workspace authority is not permitted",
            Self::Config => "Cargo configuration is not permitted",
            Self::ToolchainFile => "target repository toolchain file is not permitted",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreflightError {
    pub code: PreflightCode,
}

impl From<PreflightCode> for PreflightError {
    fn from(code: PreflightCode) -> Self {
        Self { code }
    }
}

#[derive(Debug)]
pub struct StructuralPreflight {
    pub inputs: Vec<StructuralInput>,
    pub(crate) capture: CaptureState,
}

pub fn run(request: &LowerRequest) -> Result<StructuralPreflight, PreflightError> {
    if request.contracts.len() > CONTRACT_FILES_MAX {
        return Err(PreflightCode::LimitContract.into());
    }
    let mut path_set = PortablePathSet::default();
    let manifest_path = insert_path(&mut path_set, "Cargo.toml")?;
    let lockfile_path = insert_path(&mut path_set, "Cargo.lock")?;
    let mut contract_paths = Vec::with_capacity(request.contracts.len());
    let mut path_failure = false;
    for value in &request.contracts {
        match PortablePath::parse(value) {
            Err(PortablePathError::Limit) => return Err(PreflightCode::LimitPath.into()),
            Err(PortablePathError::Invalid | PortablePathError::Collision) => {
                path_failure = true;
            }
            Ok(path) => match path_set.insert(path.clone()) {
                Ok(()) => contract_paths.push(path),
                Err(PortablePathError::Limit) => return Err(PreflightCode::LimitPath.into()),
                Err(PortablePathError::Invalid | PortablePathError::Collision) => {
                    path_failure = true;
                }
            },
        }
    }

    let mut capture = CaptureState::open(&request.source_root).map_err(map_capture_failure)?;
    capture
        .register_path(manifest_path.clone())
        .map_err(map_capture_failure)?;
    capture
        .register_path(lockfile_path.clone())
        .map_err(map_capture_failure)?;
    for path in &contract_paths {
        capture
            .register_path(path.clone())
            .map_err(map_capture_failure)?;
    }
    let manifest = capture
        .capture_registered(manifest_path, InputKind::BuildManifest, MANIFEST_BYTES_MAX)
        .map_err(|failure| map_capture_limit(failure, PreflightCode::LimitInputBytes))?;
    let lockfile =
        match capture.capture_registered(lockfile_path, InputKind::Lockfile, LOCKFILE_BYTES_MAX) {
            Ok(lockfile) => Some(lockfile),
            Err(CaptureFailure::Missing) => None,
            Err(failure) => {
                return Err(map_capture_limit(failure, PreflightCode::LimitInputBytes));
            }
        };

    let mut contract_total = 0_u64;
    let mut contracts = Vec::with_capacity(contract_paths.len());
    for path in contract_paths {
        let input = capture
            .capture_registered(path, InputKind::Contract, CONTRACT_FILE_BYTES_MAX)
            .map_err(|failure| map_capture_limit(failure, PreflightCode::LimitContract))?;
        contract_total = contract_total
            .checked_add(input.bytes.len() as u64)
            .ok_or(PreflightCode::LimitContract)?;
        if contract_total > CONTRACT_TOTAL_BYTES_MAX {
            return Err(PreflightCode::LimitContract.into());
        }
        contracts.push(input);
    }

    let layout = capture
        .root()
        .inspect_forbidden_layout()
        .map_err(|_| PreflightCode::FileType)?;
    if path_failure {
        return Err(PreflightCode::Path.into());
    }
    if layout.workspace {
        return Err(PreflightCode::Workspace.into());
    }
    if layout.config {
        return Err(PreflightCode::Config.into());
    }
    if layout.toolchain_file {
        return Err(PreflightCode::ToolchainFile.into());
    }
    let mut inputs = vec![manifest];
    inputs.extend(lockfile);
    inputs.extend(contracts);
    inputs.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
    Ok(StructuralPreflight { inputs, capture })
}

fn insert_path(paths: &mut PortablePathSet, value: &str) -> Result<PortablePath, PreflightError> {
    let path = PortablePath::parse(value).map_err(|error| match error {
        PortablePathError::Limit => PreflightCode::LimitPath,
        PortablePathError::Invalid | PortablePathError::Collision => PreflightCode::Path,
    })?;
    paths.insert(path.clone()).map_err(|error| match error {
        PortablePathError::Limit => PreflightCode::LimitPath,
        PortablePathError::Invalid | PortablePathError::Collision => PreflightCode::Path,
    })?;
    Ok(path)
}

fn map_capture_failure(failure: CaptureFailure) -> PreflightError {
    match failure {
        CaptureFailure::Missing => PreflightCode::FileType,
        CaptureFailure::FileType => PreflightCode::FileType,
        CaptureFailure::Path => PreflightCode::Path,
        CaptureFailure::PathLimit => PreflightCode::LimitPath,
        CaptureFailure::ByteLimit => PreflightCode::LimitInputBytes,
        CaptureFailure::CountLimit => PreflightCode::LimitInputCount,
    }
    .into()
}

fn map_capture_limit(failure: CaptureFailure, byte_limit: PreflightCode) -> PreflightError {
    match failure {
        CaptureFailure::ByteLimit => byte_limit.into(),
        other => map_capture_failure(other),
    }
}

#[derive(Default)]
pub(crate) struct LayoutFindings {
    workspace: bool,
    config: bool,
    toolchain_file: bool,
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) mod platform {
    use super::{LayoutFindings, PortablePath};
    use std::ffi::{c_int, c_long, c_void, CString};
    use std::fs::{File, Metadata};
    use std::io;
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;

    const AT_FDCWD: c_int = -100;
    const O_RDONLY: c_int = 0;
    const O_CLOEXEC: c_int = 0o2_000_000;
    const O_DIRECTORY: c_int = 0o200_000;
    const O_NOFOLLOW: c_int = 0o400_000;
    const SYS_GETDENTS64: c_long = 217;
    const DT_UNKNOWN: u8 = 0;
    const DT_DIR: u8 = 4;
    const DT_LNK: u8 = 10;

    #[repr(C)]
    struct StatFs {
        filesystem_type: c_long,
        block_size: c_long,
        blocks: u64,
        blocks_free: u64,
        blocks_available: u64,
        files: u64,
        files_free: u64,
        filesystem_id: [c_int; 2],
        name_length: c_long,
        fragment_size: c_long,
        flags: c_long,
        spare: [c_long; 4],
    }

    unsafe extern "C" {
        fn fstatfs(fd: c_int, buffer: *mut StatFs) -> c_int;
        fn openat(directory: c_int, path: *const i8, flags: c_int, ...) -> c_int;
        fn syscall(number: c_long, ...) -> c_long;
    }

    pub struct RootDirectory {
        file: File,
    }

    impl RootDirectory {
        pub fn open(path: &Path) -> io::Result<Self> {
            let file = open_directory_path(path)?;
            if !file.metadata()?.is_dir() || !supported_filesystem(file.as_raw_fd())? {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            Ok(Self { file })
        }

        pub fn open_regular_file(&self, path: &PortablePath) -> io::Result<File> {
            open_relative_file(self.file.as_raw_fd(), path.as_str().as_bytes())
        }

        pub fn validate_retained(&self) -> io::Result<()> {
            if self.file.metadata()?.is_dir() {
                Ok(())
            } else {
                Err(io::Error::from(io::ErrorKind::InvalidInput))
            }
        }

        pub fn inspect_forbidden_layout(&self) -> io::Result<LayoutFindings> {
            let root = open_relative_directory(self.file.as_raw_fd(), b".", false)?;
            let mut pending = vec![(root, 0_usize, false)];
            let mut findings = LayoutFindings::default();
            while let Some((directory, depth, cargo_configuration)) = pending.pop() {
                for entry in directory_entries(&directory)? {
                    let name = entry.name.as_slice();
                    if depth == 0 && matches!(name, b"rust-toolchain" | b"rust-toolchain.toml") {
                        require_regular_entry(directory.as_raw_fd(), name)?;
                        findings.toolchain_file = true;
                        continue;
                    }
                    if depth > 0 && name == b"Cargo.toml" {
                        require_regular_entry(directory.as_raw_fd(), name)?;
                        findings.workspace = true;
                        continue;
                    }
                    if cargo_configuration && matches!(name, b"config" | b"config.toml") {
                        require_regular_entry(directory.as_raw_fd(), name)?;
                        findings.config = true;
                        continue;
                    }
                    if depth == 0 && name == b".cargo" {
                        if entry.kind == DT_LNK {
                            return Err(io::Error::from(io::ErrorKind::InvalidInput));
                        }
                        let child = open_relative_directory(directory.as_raw_fd(), name, true)?;
                        pending.push((child, 1, true));
                        continue;
                    }
                    if entry.kind == DT_LNK || matches!(name, b".git" | b"target") {
                        continue;
                    }
                    if entry.kind == DT_DIR || entry.kind == DT_UNKNOWN {
                        match open_relative_directory(directory.as_raw_fd(), name, true) {
                            Ok(child) => {
                                pending.push((child, depth.saturating_add(1), false));
                            }
                            Err(error)
                                if entry.kind == DT_UNKNOWN
                                    && matches!(error.raw_os_error(), Some(20) | Some(40)) => {}
                            Err(error) => return Err(error),
                        }
                    }
                }
            }
            Ok(findings)
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub struct FileIdentity {
        device: u64,
        inode: u64,
        pub size: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
        changed_seconds: i64,
        changed_nanoseconds: i64,
    }

    impl FileIdentity {
        pub fn without_size(self) -> Self {
            Self {
                size: 0,
                modified_seconds: 0,
                modified_nanoseconds: 0,
                changed_seconds: 0,
                changed_nanoseconds: 0,
                ..self
            }
        }
    }

    pub fn regular_file_identity(file: &File) -> io::Result<FileIdentity> {
        identity_from_metadata(&file.metadata()?)
    }

    fn identity_from_metadata(metadata: &Metadata) -> io::Result<FileIdentity> {
        if !metadata.is_file() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        Ok(FileIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        })
    }

    fn open_relative_file(directory: RawFd, path: &[u8]) -> io::Result<File> {
        let mut components = path.split(|byte| *byte == b'/').peekable();
        let mut current = openat_owned(
            directory,
            b".",
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
        )?;
        let root_device = current.metadata()?.dev();
        while let Some(component) = components.next() {
            if component.is_empty() || matches!(component, b"." | b"..") {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            if components.peek().is_none() {
                let file = openat_owned(
                    current.as_raw_fd(),
                    component,
                    O_RDONLY | O_CLOEXEC | O_NOFOLLOW,
                )?;
                if file.metadata()?.dev() != root_device {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
                return Ok(file);
            }
            let child = openat_owned(
                current.as_raw_fd(),
                component,
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            )?;
            if child.metadata()?.dev() != root_device {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            current = child;
        }
        Err(io::Error::from(io::ErrorKind::InvalidInput))
    }

    fn open_relative_directory(directory: RawFd, name: &[u8], no_xdev: bool) -> io::Result<File> {
        let child = openat_owned(
            directory,
            name,
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
        )?;
        if no_xdev {
            let parent = openat_owned(
                directory,
                b".",
                O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
            )?;
            if parent.metadata()?.dev() != child.metadata()?.dev() {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
        }
        Ok(child)
    }

    fn open_directory_path(path: &Path) -> io::Result<File> {
        use std::path::Component;

        let initial = if path.is_absolute() {
            b"/".as_slice()
        } else {
            b".".as_slice()
        };
        let mut current = openat_owned(
            AT_FDCWD,
            initial,
            O_RDONLY | O_CLOEXEC | O_DIRECTORY | O_NOFOLLOW,
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
                    )?;
                }
                Component::ParentDir | Component::Prefix(_) => {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
            }
        }
        Ok(current)
    }

    fn openat_owned(directory: RawFd, path: &[u8], flags: c_int) -> io::Result<File> {
        let path = CString::new(path).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        // SAFETY: `path` is NUL-terminated and no creation flag requiring a mode is used.
        let descriptor = unsafe { openat(directory, path.as_ptr(), flags) };
        if descriptor < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openat returned this descriptor and ownership transfers to File.
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }

    pub(crate) fn supported_filesystem(descriptor: RawFd) -> io::Result<bool> {
        let mut information = StatFs {
            filesystem_type: 0,
            block_size: 0,
            blocks: 0,
            blocks_free: 0,
            blocks_available: 0,
            files: 0,
            files_free: 0,
            filesystem_id: [0; 2],
            name_length: 0,
            fragment_size: 0,
            flags: 0,
            spare: [0; 4],
        };
        // SAFETY: `information` points to writable storage of the Linux statfs shape.
        if unsafe { fstatfs(descriptor, &mut information) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let kind = information.filesystem_type as u64;
        Ok(!matches!(
            kind,
            0x0000_6969 // NFS
                | 0x0000_9fa0 // proc
                | 0x0102_1997 // 9p
                | 0x0027_e0eb // cgroup
                | 0x517b // SMB
                | 0x6265_6572 // sysfs
                | 0x6367_7270 // cgroup2
                | 0x6573_5546 // FUSE
                | 0xff53_4d42 // CIFS
        ))
    }

    struct DirectoryEntry {
        name: Vec<u8>,
        kind: u8,
    }

    fn directory_entries(directory: &File) -> io::Result<Vec<DirectoryEntry>> {
        let mut buffer = [0_u8; 16 * 1024];
        let mut entries = Vec::new();
        loop {
            // SAFETY: the buffer is writable and its exact capacity is supplied.
            let read = unsafe {
                syscall(
                    SYS_GETDENTS64,
                    directory.as_raw_fd(),
                    buffer.as_mut_ptr().cast::<c_void>(),
                    buffer.len(),
                )
            };
            if read < 0 {
                return Err(io::Error::last_os_error());
            }
            if read == 0 {
                break;
            }
            let read =
                usize::try_from(read).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
            let mut offset = 0;
            while offset < read {
                if read - offset < 19 {
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }
                let record = &buffer[offset..read];
                let record_length = usize::from(u16::from_ne_bytes([record[16], record[17]]));
                if record_length < 20 || record_length > record.len() {
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }
                let name_bytes = &record[19..record_length];
                let name_end = name_bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
                let name = &name_bytes[..name_end];
                if name != b"." && name != b".." {
                    entries.push(DirectoryEntry {
                        name: name.to_vec(),
                        kind: record[18],
                    });
                }
                offset += record_length;
            }
        }
        Ok(entries)
    }

    fn require_regular_entry(directory: RawFd, name: &[u8]) -> io::Result<()> {
        let file = open_relative_file(directory, name)?;
        identity_from_metadata(&file.metadata()?).map(|_| ())
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub(crate) mod platform {
    use super::{LayoutFindings, PortablePath};
    use std::fs::File;
    use std::io;
    use std::path::Path;

    pub struct RootDirectory;

    impl RootDirectory {
        pub fn open(_path: &Path) -> io::Result<Self> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }

        pub fn open_regular_file(&self, _path: &PortablePath) -> io::Result<File> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }

        pub fn validate_retained(&self) -> io::Result<()> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }

        pub fn inspect_forbidden_layout(&self) -> io::Result<LayoutFindings> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub struct FileIdentity {
        pub size: u64,
    }

    impl FileIdentity {
        pub fn without_size(self) -> Self {
            Self { size: 0 }
        }
    }

    pub fn regular_file_identity(_file: &File) -> io::Result<FileIdentity> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}
