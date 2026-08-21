//! SHA-256 helpers for VIR-era untrusted helper artifacts.

use crate::canonical_json::{canonical_json_bytes, CanonicalJsonError, StrictJsonValue};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

/// A compile-time/static domain tag used before the mandatory zero separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HashDomain(&'static str);

impl HashDomain {
    pub const fn new(text: &'static str) -> Self {
        Self(text)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    fn validate(self) -> Result<(), HashError> {
        if self.0.is_empty() || !self.0.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(HashError::InvalidDomain { domain: self.0 });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

        let mut output = String::with_capacity(64);
        for byte in self.0 {
            output.push(char::from(LOWER_HEX[usize::from(byte >> 4)]));
            output.push(char::from(LOWER_HEX[usize::from(byte & 0x0f)]));
        }
        output
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Sha256Digest")
            .field(&self.to_hex())
            .finish()
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[derive(Debug)]
pub enum HashError {
    InvalidDomain { domain: &'static str },
    InventoryNotArray,
    CanonicalJson(CanonicalJsonError),
}

impl fmt::Display for HashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDomain { domain } => {
                write!(formatter, "invalid nonempty ASCII hash domain {domain:?}")
            }
            Self::InventoryNotArray => {
                formatter.write_str("canonical JSON inventory must be an array")
            }
            Self::CanonicalJson(error) => write!(formatter, "canonical JSON failed: {error}"),
        }
    }
}

impl Error for HashError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDomain { .. } | Self::InventoryNotArray => None,
            Self::CanonicalJson(error) => Some(error),
        }
    }
}

impl From<CanonicalJsonError> for HashError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

/// Hashes only narrowed-JCS bytes under `domain || 0x00`.
///
/// The value is supplied structurally, so this API cannot accidentally hash a
/// pretty-printed JSON byte string.
pub fn hash_canonical_json(
    domain: HashDomain,
    value_without_hash: &StrictJsonValue,
) -> Result<Sha256Digest, HashError> {
    let canonical = canonical_json_bytes(value_without_hash)?;
    hash_domain_separated_raw(domain, &canonical)
}

/// Hashes a canonical JSON array used as a specification-owned inventory.
pub fn hash_canonical_inventory(
    domain: HashDomain,
    inventory: &StrictJsonValue,
) -> Result<Sha256Digest, HashError> {
    if inventory.as_array().is_none() {
        return Err(HashError::InventoryNotArray);
    }
    hash_canonical_json(domain, inventory)
}

/// Hashes an explicitly raw, non-JSON preimage under `domain || 0x00`.
///
/// Callers handling JSON must use [`hash_canonical_json`] instead. The `raw`
/// name makes the byte-preserving behavior explicit at call sites.
pub fn hash_domain_separated_raw(
    domain: HashDomain,
    raw_payload: &[u8],
) -> Result<Sha256Digest, HashError> {
    domain.validate()?;
    let mut hasher = Sha256::new();
    hasher.update(domain.as_str().as_bytes());
    hasher.update([0_u8]);
    hasher.update(raw_payload);
    Ok(finish(hasher))
}

/// Computes the plain SHA-256 required for exact raw source/file bytes.
///
/// This intentionally has no domain separator because the owning source and
/// bundle specifications define their raw-file fields that way.
pub fn sha256_raw_file_bytes(raw_bytes: &[u8]) -> Sha256Digest {
    let mut hasher = Sha256::new();
    hasher.update(raw_bytes);
    finish(hasher)
}

fn finish(hasher: Sha256) -> Sha256Digest {
    let output = hasher.finalize();
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&output);
    Sha256Digest::from_bytes(bytes)
}
