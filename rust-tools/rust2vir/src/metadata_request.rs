use crate::manifest::ExpectedManifestSelection;
use crate::snapshot::{Snapshot, SnapshotError};

pub const SNAPSHOT_MANIFEST_PATH: &str = "/mpk/input/Cargo.toml";

const ARGUMENTS: [&str; 11] = [
    "metadata",
    "--manifest-path",
    SNAPSHOT_MANIFEST_PATH,
    "--format-version",
    "1",
    "--no-deps",
    "--locked",
    "--offline",
    "--no-default-features",
    "--color",
    "never",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataRequest {
    expected: ExpectedManifestSelection,
}

impl MetadataRequest {
    pub fn for_snapshot(
        snapshot: &Snapshot,
        expected: ExpectedManifestSelection,
    ) -> Result<Self, SnapshotError> {
        snapshot.validate()?;
        Ok(Self { expected })
    }

    pub fn arguments(&self) -> &'static [&'static str] {
        &ARGUMENTS
    }

    pub fn expected(&self) -> &ExpectedManifestSelection {
        &self.expected
    }
}
