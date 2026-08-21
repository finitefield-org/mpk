//! Closed semantic-profile identities and parameter objects for VIR v0.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Source-language identity recorded by a VIR module.
///
/// This value is descriptive. Value-operation semantics are selected by
/// [`SemanticProfile`], never by this enum.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    Go,
    Rust,
}

/// The closed set of semantic profiles admitted by VIR v0.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum SemanticProfile {
    #[serde(rename = "mpk.go.fixed.v0")]
    GoFixedV0,
    #[serde(rename = "mpk.rust.checked.v0")]
    RustCheckedV0,
}

impl SemanticProfile {
    pub const fn source_language(self) -> SourceLanguage {
        match self {
            Self::GoFixedV0 => SourceLanguage::Go,
            Self::RustCheckedV0 => SourceLanguage::Rust,
        }
    }
}

/// Pointer widths admitted by both initial semantic profiles.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PointerWidth {
    Bits32,
    Bits64,
}

impl PointerWidth {
    pub const fn bits(self) -> u32 {
        match self {
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }
}

impl TryFrom<u32> for PointerWidth {
    type Error = PointerWidthError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            32 => Ok(Self::Bits32),
            64 => Ok(Self::Bits64),
            _ => Err(PointerWidthError(value)),
        }
    }
}

impl From<PointerWidth> for u32 {
    fn from(value: PointerWidth) -> Self {
        value.bits()
    }
}

impl<'de> Deserialize<'de> for PointerWidth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Self::try_from(bits).map_err(serde::de::Error::custom)
    }
}

impl Serialize for PointerWidth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerWidthError(u32);

impl fmt::Display for PointerWidthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "pointer width must be 32 or 64, found {}",
            self.0
        )
    }
}

impl Error for PointerWidthError {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OverflowMode {
    Checked,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PanicMode {
    Abort,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GoFixedParameters {
    pub target_id: String,
    pub pointer_width: PointerWidth,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RustCheckedParameters {
    pub target_id: String,
    pub pointer_width: PointerWidth,
    pub overflow_mode: OverflowMode,
    pub panic_mode: PanicMode,
}

/// Exact profile-specific semantic parameter object.
///
/// The member objects deny unknown fields, so the Go and Rust shapes cannot be
/// extended or confused during deserialization.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SemanticParameters {
    GoFixed(GoFixedParameters),
    RustChecked(RustCheckedParameters),
}

impl SemanticParameters {
    pub const fn profile(&self) -> SemanticProfile {
        match self {
            Self::GoFixed(_) => SemanticProfile::GoFixedV0,
            Self::RustChecked(_) => SemanticProfile::RustCheckedV0,
        }
    }

    pub const fn pointer_width(&self) -> PointerWidth {
        match self {
            Self::GoFixed(parameters) => parameters.pointer_width,
            Self::RustChecked(parameters) => parameters.pointer_width,
        }
    }

    pub fn target_id(&self) -> &str {
        match self {
            Self::GoFixed(parameters) => &parameters.target_id,
            Self::RustChecked(parameters) => &parameters.target_id,
        }
    }
}

/// A serialized semantic context used at VIR boundaries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SemanticContext {
    pub source_language: SourceLanguage,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
}

impl<'de> Deserialize<'de> for SemanticContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireContext {
            source_language: SourceLanguage,
            semantic_profile: SemanticProfile,
            semantic_parameters: SemanticParameters,
        }

        let wire = WireContext::deserialize(deserializer)?;
        let context = Self {
            source_language: wire.source_language,
            semantic_profile: wire.semantic_profile,
            semantic_parameters: wire.semantic_parameters,
        };
        context.validate().map_err(serde::de::Error::custom)?;
        Ok(context)
    }
}

impl SemanticContext {
    pub fn validate(&self) -> Result<(), SemanticProfileError> {
        validate_semantic_context(
            self.source_language,
            self.semantic_profile,
            &self.semantic_parameters,
        )
    }
}

pub fn validate_semantic_context(
    source_language: SourceLanguage,
    semantic_profile: SemanticProfile,
    semantic_parameters: &SemanticParameters,
) -> Result<(), SemanticProfileError> {
    if semantic_profile.source_language() != source_language {
        return Err(SemanticProfileError::LanguageProfileMismatch {
            source_language,
            semantic_profile,
        });
    }
    validate_semantic_parameters(semantic_profile, semantic_parameters)
}

pub fn validate_semantic_parameters(
    semantic_profile: SemanticProfile,
    semantic_parameters: &SemanticParameters,
) -> Result<(), SemanticProfileError> {
    if semantic_parameters.profile() != semantic_profile {
        return Err(SemanticProfileError::ParameterProfileMismatch {
            semantic_profile,
            parameter_profile: semantic_parameters.profile(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticProfileError {
    LanguageProfileMismatch {
        source_language: SourceLanguage,
        semantic_profile: SemanticProfile,
    },
    ParameterProfileMismatch {
        semantic_profile: SemanticProfile,
        parameter_profile: SemanticProfile,
    },
}

impl fmt::Display for SemanticProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LanguageProfileMismatch {
                source_language,
                semantic_profile,
            } => write!(
                formatter,
                "source language {source_language:?} does not match semantic profile {semantic_profile:?}"
            ),
            Self::ParameterProfileMismatch {
                semantic_profile,
                parameter_profile,
            } => write!(
                formatter,
                "semantic profile {semantic_profile:?} does not match parameter shape {parameter_profile:?}"
            ),
        }
    }
}

impl Error for SemanticProfileError {}
