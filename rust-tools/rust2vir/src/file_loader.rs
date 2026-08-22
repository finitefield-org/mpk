use crate::contract::ContractInput;
use crate::driver_protocol::{DriverInputIdentity, DriverRequest};
use crate::path::PortablePath;
use crate::preflight::platform::{regular_file_identity, RootDirectory};
use crate::sha256::{digest, hex};
use crate::source_gate::{validate_source, SourceGateCode, SourceGateStatus, SourceRole};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRangeError {
    External,
    Range,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedSourceRange {
    pub normalized_path: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceLoaderCode {
    Gate(SourceGateCode),
    FrontendSourceInventory,
}

impl SourceLoaderCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gate(code) => code.as_str(),
            Self::FrontendSourceInventory => "RUST_FRONTEND_SOURCE_INVENTORY",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Gate(code) => code.message(),
            Self::FrontendSourceInventory => {
                "compiler source inventory disagrees with the immutable snapshot"
            }
        }
    }

    pub fn phase(self) -> &'static str {
        match self {
            Self::Gate(code) => code.phase(),
            Self::FrontendSourceInventory => "source",
        }
    }

    pub fn status(self) -> SourceLoaderStatus {
        match self {
            Self::Gate(code) => match code.status() {
                SourceGateStatus::Rejected => SourceLoaderStatus::Rejected,
                SourceGateStatus::SourceError => SourceLoaderStatus::SourceError,
            },
            Self::FrontendSourceInventory => SourceLoaderStatus::FrontendError,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceLoaderStatus {
    Rejected,
    SourceError,
    FrontendError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLoaderError {
    pub code: SourceLoaderCode,
}

impl From<SourceGateCode> for SourceLoaderError {
    fn from(code: SourceGateCode) -> Self {
        Self {
            code: SourceLoaderCode::Gate(code),
        }
    }
}

impl From<SourceLoaderCode> for SourceLoaderError {
    fn from(code: SourceLoaderCode) -> Self {
        Self { code }
    }
}

#[derive(Clone)]
struct ImmutableSource {
    bytes: Arc<[u8]>,
}

#[derive(Default)]
struct LoaderState {
    reads: BTreeSet<PortablePath>,
    root_callbacks: BTreeSet<PortablePath>,
    failure: Option<SourceLoaderError>,
}

pub struct SnapshotFileLoader {
    root_path: PathBuf,
    crate_root: PortablePath,
    sources: BTreeMap<PortablePath, ImmutableSource>,
    contracts: Vec<ContractInput>,
    state: Mutex<LoaderState>,
}

impl fmt::Debug for SnapshotFileLoader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SnapshotFileLoader")
            .field("root_path", &self.root_path)
            .field("crate_root", &self.crate_root)
            .field("source_count", &self.sources.len())
            .field("contract_count", &self.contracts.len())
            .finish_non_exhaustive()
    }
}

impl SnapshotFileLoader {
    pub fn from_request(
        root_path: &Path,
        crate_root: &str,
        request: &DriverRequest,
    ) -> Result<Self, SourceLoaderError> {
        Self::open_with_contracts(
            root_path,
            crate_root,
            &request.source_inventory(),
            &request.contract_inventory(),
        )
    }

    pub fn open(
        root_path: &Path,
        crate_root: &str,
        inventory: &[DriverInputIdentity],
    ) -> Result<Self, SourceLoaderError> {
        Self::open_with_contracts(root_path, crate_root, inventory, &[])
    }

    pub fn open_with_contracts(
        root_path: &Path,
        crate_root: &str,
        source_inventory: &[DriverInputIdentity],
        contract_inventory: &[DriverInputIdentity],
    ) -> Result<Self, SourceLoaderError> {
        let crate_root = PortablePath::parse(crate_root)
            .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
        let directory = RootDirectory::open(root_path)
            .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
        let mut sources = BTreeMap::new();
        let mut seen_paths = BTreeSet::new();
        for expected in source_inventory {
            if expected.kind != "source"
                || !seen_paths.insert(expected.normalized_path.to_ascii_lowercase())
            {
                return Err(SourceLoaderCode::FrontendSourceInventory.into());
            }
            let (path, bytes) = read_inventory_input(&directory, expected)?;
            if sources.contains_key(&path) {
                return Err(SourceLoaderCode::FrontendSourceInventory.into());
            }
            sources.insert(path, ImmutableSource { bytes });
        }
        let mut contracts = Vec::with_capacity(contract_inventory.len());
        for expected in contract_inventory {
            if expected.kind != "contract"
                || !seen_paths.insert(expected.normalized_path.to_ascii_lowercase())
            {
                return Err(SourceLoaderCode::FrontendSourceInventory.into());
            }
            let (path, bytes) = read_inventory_input(&directory, expected)?;
            contracts.push(ContractInput {
                normalized_path: path.as_str().to_owned(),
                raw_input_sha256: expected.sha256.clone(),
                bytes,
            });
        }
        if sources.is_empty() || !sources.contains_key(&crate_root) {
            return Err(SourceLoaderCode::FrontendSourceInventory.into());
        }
        let loader = Self {
            root_path: root_path.to_owned(),
            crate_root,
            sources,
            contracts,
            state: Mutex::new(LoaderState::default()),
        };
        loader.validate_preflight_inventory()?;
        Ok(loader)
    }

    pub fn read_utf8(&self, path: &Path) -> Result<String, SourceLoaderError> {
        let normalized = self.requested_path(path)?;
        let source = self
            .sources
            .get(&normalized)
            .ok_or(SourceLoaderCode::FrontendSourceInventory)?;
        self.validate_one(&normalized, &source.bytes)?;
        let text = std::str::from_utf8(&source.bytes)
            .map_err(|_| SourceGateCode::SourceParse)?
            .to_owned();
        self.state().reads.insert(normalized);
        Ok(text)
    }

    pub fn file_exists(&self, path: &Path) -> bool {
        self.requested_path(path)
            .is_ok_and(|path| self.sources.contains_key(&path))
    }

    pub fn read_binary(&self, _path: &Path) -> Result<Arc<[u8]>, SourceLoaderError> {
        Err(SourceLoaderCode::FrontendSourceInventory.into())
    }

    pub fn read_file(&self, path: &Path) -> io::Result<String> {
        self.read_utf8(path)
            .map_err(|error| self.record_io_failure(error))
    }

    pub fn read_binary_file(&self, path: &Path) -> io::Result<Arc<[u8]>> {
        self.read_binary(path)
            .map_err(|error| self.record_io_failure(error))
    }

    pub fn crate_root_path(&self) -> PathBuf {
        self.root_path.join(self.crate_root.as_str())
    }

    pub fn crate_root_bytes(&self) -> Arc<[u8]> {
        Arc::clone(
            &self
                .sources
                .get(&self.crate_root)
                .expect("validated inventory contains the crate root")
                .bytes,
        )
    }

    pub fn contract_inputs(&self) -> Vec<ContractInput> {
        self.contracts.clone()
    }

    pub fn validate_root_ast(&self, path: &Path, bytes: &[u8]) -> Result<(), SourceLoaderError> {
        let normalized = self.requested_path(path)?;
        if normalized != self.crate_root {
            return Err(SourceLoaderCode::FrontendSourceInventory.into());
        }
        let expected = self
            .sources
            .get(&normalized)
            .ok_or(SourceLoaderCode::FrontendSourceInventory)?;
        if expected.bytes.as_ref() != bytes {
            return Err(SourceLoaderCode::FrontendSourceInventory.into());
        }
        validate_source(bytes, SourceRole::CrateRoot).map_err(|error| error.code)?;
        let mut state = self.state();
        if !state.root_callbacks.insert(normalized) {
            return Err(SourceLoaderCode::FrontendSourceInventory.into());
        }
        Ok(())
    }

    pub fn verify_inventory(&self) -> Result<(), SourceLoaderError> {
        let state = self.state();
        let observed = state
            .reads
            .union(&state.root_callbacks)
            .cloned()
            .collect::<BTreeSet<_>>();
        let expected = self.sources.keys().cloned().collect::<BTreeSet<_>>();
        if state.failure.is_some()
            || state.root_callbacks != BTreeSet::from([self.crate_root.clone()])
            || observed != expected
        {
            return Err(SourceLoaderCode::FrontendSourceInventory.into());
        }
        Ok(())
    }

    pub fn observed_paths(&self) -> Vec<String> {
        let state = self.state();
        state
            .reads
            .union(&state.root_callbacks)
            .map(|path| path.as_str().to_owned())
            .collect()
    }

    pub fn captured_source_range(
        &self,
        path: &Path,
        start: u64,
        end: u64,
    ) -> Result<CapturedSourceRange, SourceRangeError> {
        let normalized = self
            .requested_path(path)
            .map_err(|_| SourceRangeError::External)?;
        let source = self
            .sources
            .get(&normalized)
            .ok_or(SourceRangeError::External)?;
        let start_index = usize::try_from(start).map_err(|_| SourceRangeError::Range)?;
        let end_index = usize::try_from(end).map_err(|_| SourceRangeError::Range)?;
        let text = std::str::from_utf8(&source.bytes).map_err(|_| SourceRangeError::Range)?;
        if start_index >= end_index
            || end_index > source.bytes.len()
            || !text.is_char_boundary(start_index)
            || !text.is_char_boundary(end_index)
        {
            return Err(SourceRangeError::Range);
        }
        Ok(CapturedSourceRange {
            normalized_path: normalized.as_str().to_owned(),
            start,
            end,
        })
    }

    pub fn failure(&self) -> Option<SourceLoaderError> {
        self.state().failure
    }

    fn validate_preflight_inventory(&self) -> Result<(), SourceLoaderError> {
        for (path, source) in &self.sources {
            self.validate_one(path, &source.bytes)?;
        }
        Ok(())
    }

    fn validate_one(&self, path: &PortablePath, bytes: &[u8]) -> Result<(), SourceLoaderError> {
        let role = if path == &self.crate_root {
            SourceRole::CrateRoot
        } else {
            SourceRole::Module
        };
        validate_source(bytes, role).map_err(|error| error.code.into())
    }

    fn requested_path(&self, path: &Path) -> Result<PortablePath, SourceLoaderError> {
        let relative = if path.is_absolute() {
            path.strip_prefix(&self.root_path)
                .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?
        } else {
            path
        };
        let text = relative
            .to_str()
            .ok_or(SourceLoaderCode::FrontendSourceInventory)?;
        PortablePath::parse(text).map_err(|_| SourceLoaderCode::FrontendSourceInventory.into())
    }

    fn record_io_failure(&self, error: SourceLoaderError) -> io::Error {
        let mut state = self.state();
        if state.failure.is_none() {
            state.failure = Some(error);
        }
        io::Error::new(io::ErrorKind::PermissionDenied, error.code.as_str())
    }

    fn state(&self) -> MutexGuard<'_, LoaderState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn read_inventory_input(
    directory: &RootDirectory,
    expected: &DriverInputIdentity,
) -> Result<(PortablePath, Arc<[u8]>), SourceLoaderError> {
    let path = PortablePath::parse(&expected.normalized_path)
        .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    let mut file = directory
        .open_regular_file(&path)
        .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    let before =
        regular_file_identity(&file).map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    if before.size != expected.size_bytes {
        return Err(SourceLoaderCode::FrontendSourceInventory.into());
    }
    let maximum = expected
        .size_bytes
        .checked_add(1)
        .ok_or(SourceLoaderCode::FrontendSourceInventory)?;
    let capacity = usize::try_from(expected.size_bytes)
        .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    let mut bytes = Vec::with_capacity(capacity);
    file.by_ref()
        .take(maximum)
        .read_to_end(&mut bytes)
        .map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    let after =
        regular_file_identity(&file).map_err(|_| SourceLoaderCode::FrontendSourceInventory)?;
    if before != after
        || bytes.len() as u64 != expected.size_bytes
        || hex(&digest(&bytes)) != expected.sha256
    {
        return Err(SourceLoaderCode::FrontendSourceInventory.into());
    }
    Ok((path, Arc::from(bytes)))
}
