use crate::limits::RustLimitId;
use crate::path::{PortablePath, PortablePathError, PortablePathSet};
use crate::preflight::platform;
use crate::sha256::Sha256;
use std::collections::BTreeSet;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

pub const SNAPSHOT_ENTRIES_MAX: usize = RustLimitId::SnapshotEntries.maximum() as usize;
pub const SNAPSHOT_TOTAL_BYTES_MAX: u64 = RustLimitId::SnapshotBytes.maximum();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputKind {
    BuildManifest,
    Lockfile,
    Contract,
    Source,
}

#[derive(Clone)]
pub struct CapturedInput {
    pub kind: InputKind,
    pub normalized_path: PortablePath,
    pub bytes: Arc<[u8]>,
    pub sha256: [u8; 32],
    pub(crate) identity: platform::FileIdentity,
}

impl CapturedInput {
    pub fn sha256_hex(&self) -> String {
        crate::sha256::hex(&self.sha256)
    }

    pub fn has_same_original_identity(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl fmt::Debug for CapturedInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CapturedInput")
            .field("kind", &self.kind)
            .field("normalized_path", &self.normalized_path)
            .field("size_bytes", &self.bytes.len())
            .field("sha256", &self.sha256_hex())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureFailure {
    Missing,
    FileType,
    Path,
    PathLimit,
    ByteLimit,
    CountLimit,
}

pub(crate) struct OpenedInput {
    file: File,
    pub identity: platform::FileIdentity,
}

pub(crate) struct CaptureState {
    root: platform::RootDirectory,
    paths: PortablePathSet,
    identities: BTreeSet<platform::FileIdentity>,
    total_bytes: u64,
    input_count: usize,
}

impl fmt::Debug for CaptureState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CaptureState")
            .field("total_bytes", &self.total_bytes)
            .field("input_count", &self.input_count)
            .finish_non_exhaustive()
    }
}

impl CaptureState {
    pub(crate) fn open(source_root: &Path) -> Result<Self, CaptureFailure> {
        let root =
            platform::RootDirectory::open(source_root).map_err(|_| CaptureFailure::FileType)?;
        Ok(Self {
            root,
            paths: PortablePathSet::default(),
            identities: BTreeSet::new(),
            total_bytes: 0,
            input_count: 0,
        })
    }

    pub(crate) fn root(&self) -> &platform::RootDirectory {
        &self.root
    }

    pub(crate) fn validate_retained_root(&self) -> Result<(), CaptureFailure> {
        self.root
            .validate_retained()
            .map_err(|_| CaptureFailure::FileType)
    }

    pub(crate) fn register_path(&mut self, path: PortablePath) -> Result<(), CaptureFailure> {
        self.paths.insert(path).map_err(map_path_error)
    }

    pub(crate) fn open_candidate(
        &self,
        path: &PortablePath,
    ) -> Result<Option<OpenedInput>, CaptureFailure> {
        let file = match self.root.open_regular_file(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(CaptureFailure::FileType),
        };
        let identity =
            platform::regular_file_identity(&file).map_err(|_| CaptureFailure::FileType)?;
        Ok(Some(OpenedInput { file, identity }))
    }

    pub(crate) fn capture_registered(
        &mut self,
        path: PortablePath,
        kind: InputKind,
        maximum_bytes: u64,
    ) -> Result<CapturedInput, CaptureFailure> {
        let opened = self.open_candidate(&path)?.ok_or(CaptureFailure::Missing)?;
        self.capture_opened(path, kind, maximum_bytes, u64::MAX, opened)
    }

    pub(crate) fn capture_registered_with_aggregate(
        &mut self,
        path: PortablePath,
        kind: InputKind,
        maximum_bytes: u64,
        aggregate_remaining: u64,
    ) -> Result<CapturedInput, CaptureFailure> {
        let opened = self.open_candidate(&path)?.ok_or(CaptureFailure::Missing)?;
        self.capture_opened(path, kind, maximum_bytes, aggregate_remaining, opened)
    }

    pub(crate) fn capture_new_with_aggregate(
        &mut self,
        path: PortablePath,
        kind: InputKind,
        maximum_bytes: u64,
        aggregate_remaining: u64,
        opened: OpenedInput,
    ) -> Result<CapturedInput, CaptureFailure> {
        self.register_path(path.clone())?;
        self.capture_opened(path, kind, maximum_bytes, aggregate_remaining, opened)
    }

    fn capture_opened(
        &mut self,
        path: PortablePath,
        kind: InputKind,
        maximum_bytes: u64,
        aggregate_remaining: u64,
        mut opened: OpenedInput,
    ) -> Result<CapturedInput, CaptureFailure> {
        if self.input_count >= SNAPSHOT_ENTRIES_MAX {
            return Err(CaptureFailure::CountLimit);
        }
        let snapshot_remaining = SNAPSHOT_TOTAL_BYTES_MAX
            .checked_sub(self.total_bytes)
            .ok_or(CaptureFailure::ByteLimit)?;
        let read_maximum = maximum_bytes
            .min(aggregate_remaining)
            .min(snapshot_remaining);
        if opened.identity.size > read_maximum {
            return Err(CaptureFailure::ByteLimit);
        }
        if !self.identities.insert(opened.identity.without_size()) {
            return Err(CaptureFailure::FileType);
        }

        let capacity =
            usize::try_from(opened.identity.size).map_err(|_| CaptureFailure::ByteLimit)?;
        let mut bytes = Vec::with_capacity(capacity);
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let read_limit = read_maximum
                .checked_add(1)
                .and_then(|limit| limit.checked_sub(bytes.len() as u64))
                .ok_or(CaptureFailure::ByteLimit)?;
            let read = opened
                .file
                .by_ref()
                .take(read_limit)
                .read(&mut buffer)
                .map_err(|_| CaptureFailure::FileType)?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..read]);
            hasher.update(&buffer[..read]);
            if bytes.len() as u64 > read_maximum {
                return Err(CaptureFailure::ByteLimit);
            }
        }
        let after =
            platform::regular_file_identity(&opened.file).map_err(|_| CaptureFailure::FileType)?;
        if opened.identity != after || after.size != bytes.len() as u64 {
            return Err(CaptureFailure::FileType);
        }
        let total_bytes = self
            .total_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(CaptureFailure::ByteLimit)?;
        if total_bytes > SNAPSHOT_TOTAL_BYTES_MAX {
            return Err(CaptureFailure::ByteLimit);
        }
        self.total_bytes = total_bytes;
        self.input_count += 1;
        Ok(CapturedInput {
            kind,
            normalized_path: path,
            bytes: Arc::from(bytes),
            sha256: hasher.finish(),
            identity: after,
        })
    }
}

fn map_path_error(error: PortablePathError) -> CaptureFailure {
    match error {
        PortablePathError::Limit => CaptureFailure::PathLimit,
        PortablePathError::Invalid | PortablePathError::Collision => CaptureFailure::Path,
    }
}
