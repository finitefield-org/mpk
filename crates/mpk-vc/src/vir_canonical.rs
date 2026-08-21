//! Canonical JSON and domain-separated hashes for VIR v0.

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, CanonicalJsonError, ObjectFieldsError,
    StrictJsonError, StrictJsonLimits, StrictJsonValue,
};
use crate::hash::{hash_canonical_json, HashDomain, HashError};
use crate::vir::{
    LowercaseSha256, LowercaseSha256Error, VirContract, VirModule, VIR_INPUT_JSON_BYTES_MAX,
    VIR_JSON_NESTING_MAX, VIR_STRING_BYTES_MAX,
};
use serde::Serialize;
use std::error::Error;
use std::fmt;

pub const VIR_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-0.1");
pub const CONTRACT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CONTRACT-0.1");

const SERIALIZED_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_JSON_NESTING_MAX,
    VIR_STRING_BYTES_MAX,
);

pub fn canonical_vir_json(module: &VirModule) -> Result<Vec<u8>, VirCanonicalError> {
    canonical_json_bytes(&strict_value(module)?).map_err(Into::into)
}

pub fn canonical_contract_json(contract: &VirContract) -> Result<Vec<u8>, VirCanonicalError> {
    canonical_json_bytes(&strict_value(contract)?).map_err(Into::into)
}

/// Returns the exact JCS payload covered by [`vir_hash`].
pub fn canonical_vir_hash_payload(module: &VirModule) -> Result<Vec<u8>, VirCanonicalError> {
    canonical_without_field(module, "vir_hash")
}

/// Returns the exact JCS payload covered by [`contract_hash`].
pub fn canonical_contract_hash_payload(
    contract: &VirContract,
) -> Result<Vec<u8>, VirCanonicalError> {
    canonical_without_field(contract, "contract_hash")
}

pub fn vir_hash(module: &VirModule) -> Result<LowercaseSha256, VirCanonicalError> {
    hash_without_field(module, "vir_hash", VIR_HASH_DOMAIN)
}

pub fn contract_hash(contract: &VirContract) -> Result<LowercaseSha256, VirCanonicalError> {
    hash_without_field(contract, "contract_hash", CONTRACT_HASH_DOMAIN)
}

fn hash_without_field<T: Serialize>(
    value: &T,
    field: &str,
    domain: HashDomain,
) -> Result<LowercaseSha256, VirCanonicalError> {
    let strict = strict_value(value)?;
    let payload = strict.clone_without_fields(&[field])?;
    let digest = hash_canonical_json(domain, &payload)?;
    LowercaseSha256::new(digest.to_hex()).map_err(Into::into)
}

fn canonical_without_field<T: Serialize>(
    value: &T,
    field: &str,
) -> Result<Vec<u8>, VirCanonicalError> {
    let strict = strict_value(value)?;
    let payload = strict.clone_without_fields(&[field])?;
    canonical_json_bytes(&payload).map_err(Into::into)
}

fn strict_value<T: Serialize>(value: &T) -> Result<StrictJsonValue, VirCanonicalError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| VirCanonicalError::Serialization(error.to_string()))?;
    parse_strict_json(&bytes, SERIALIZED_LIMITS).map_err(Into::into)
}

#[derive(Debug)]
pub enum VirCanonicalError {
    Serialization(String),
    StrictJson(StrictJsonError),
    CanonicalJson(CanonicalJsonError),
    MissingHashField(ObjectFieldsError),
    Hash(HashError),
    HashFormat(LowercaseSha256Error),
}

impl fmt::Display for VirCanonicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(message) => {
                write!(formatter, "VIR serialization failed: {message}")
            }
            Self::StrictJson(error) => {
                write!(formatter, "serialized VIR is not strict JSON: {error}")
            }
            Self::CanonicalJson(error) => write!(formatter, "VIR canonical JSON failed: {error}"),
            Self::MissingHashField(error) => {
                write!(formatter, "VIR hash field exclusion failed: {error}")
            }
            Self::Hash(error) => write!(formatter, "VIR domain-separated hash failed: {error}"),
            Self::HashFormat(error) => write!(formatter, "VIR digest formatting failed: {error}"),
        }
    }
}

impl Error for VirCanonicalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialization(_) => None,
            Self::StrictJson(error) => Some(error),
            Self::CanonicalJson(error) => Some(error),
            Self::MissingHashField(error) => Some(error),
            Self::Hash(error) => Some(error),
            Self::HashFormat(error) => Some(error),
        }
    }
}

impl From<StrictJsonError> for VirCanonicalError {
    fn from(error: StrictJsonError) -> Self {
        Self::StrictJson(error)
    }
}

impl From<CanonicalJsonError> for VirCanonicalError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<ObjectFieldsError> for VirCanonicalError {
    fn from(error: ObjectFieldsError) -> Self {
        Self::MissingHashField(error)
    }
}

impl From<HashError> for VirCanonicalError {
    fn from(error: HashError) -> Self {
        Self::Hash(error)
    }
}

impl From<LowercaseSha256Error> for VirCanonicalError {
    fn from(error: LowercaseSha256Error) -> Self {
        Self::HashFormat(error)
    }
}
