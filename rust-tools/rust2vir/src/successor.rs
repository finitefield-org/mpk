//! Staging-only compiled identities for the semantic-profile successor.

use crate::json::JsonValue;
use std::collections::BTreeMap;

pub const PROFILE_REGISTRY_SCHEMA: &str = "mpk.semantic_profile.registry.v1";
pub const PROFILE_REGISTRY_ID: &str = "mpk.semantic_profile.registry.v1";
pub const PROFILE_REGISTRY_REVISION: i64 = 2;
pub const PROFILE_REGISTRY_SHA256: &str =
    "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75";
pub const PROFILE_ENTRY_SHA256: &str =
    "1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057";
pub const PARAMETERS_SCHEMA: &str = "mpk.semantic_parameters.rust_checked.v0";
pub const SELECTION_SCHEMA: &str = "mpk.selection.rust_function.v0";
pub const FRONTEND_ID: &str = "frontend.rust.rust2vir.candidate.v1";
pub const TOOLCHAIN_ID: &str = "toolchain.rust.nightly-2025-06-01.candidate.v1";
pub const TOOLCHAIN_DISTRIBUTION_SHA256: &str =
    "86dab73dadd3a3184064e7d7da7e878562eba4cfc4c8a969bc8f44a5e865c90a";

pub fn semantic_parameters(target: &str, pointer_width: u8) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("overflow_mode".to_owned(), string("checked")),
        ("panic_mode".to_owned(), string("abort")),
        (
            "pointer_width".to_owned(),
            JsonValue::Number(pointer_width.to_string()),
        ),
        ("target_id".to_owned(), string(target)),
    ]))
}

pub fn semantic_context(target: &str, pointer_width: u8) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "profile_entry_sha256".to_owned(),
            string(PROFILE_ENTRY_SHA256),
        ),
        (
            "profile_registry".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("id".to_owned(), string(PROFILE_REGISTRY_ID)),
                (
                    "registry_sha256".to_owned(),
                    string(PROFILE_REGISTRY_SHA256),
                ),
                (
                    "revision".to_owned(),
                    JsonValue::Number(PROFILE_REGISTRY_REVISION.to_string()),
                ),
                ("schema".to_owned(), string(PROFILE_REGISTRY_SCHEMA)),
            ])),
        ),
        (
            "semantic_parameters".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("schema".to_owned(), string(PARAMETERS_SCHEMA)),
                (
                    "value".to_owned(),
                    semantic_parameters(target, pointer_width),
                ),
            ])),
        ),
        ("semantic_profile".to_owned(), string("mpk.rust.checked.v0")),
        ("source_language".to_owned(), string("rust")),
    ]))
}

pub fn selection_envelope(package: &str, crate_name: &str, function: &str) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string(SELECTION_SCHEMA)),
        (
            "value".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("crate".to_owned(), string(crate_name)),
                ("function".to_owned(), string(function)),
                ("kind".to_owned(), string("lib")),
                ("package".to_owned(), string(package)),
            ])),
        ),
    ]))
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}
