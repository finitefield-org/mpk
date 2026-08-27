#!/usr/bin/env python3
"""Materialize the exact-hash staging-only successor rust2vir source tree."""

from __future__ import annotations

import hashlib
from pathlib import Path
import shutil


SOURCE_SHA256 = {
    "src/cli.rs": "dfabe94f6613977a17b0e85c21ed30c7d5a30bc263f6ab0b4b6fd6d56a214e66",
    "src/contract_typecheck.rs": "83d8ab684ee13ced4d7a521e1985c4bec94843f3ac40c2fff4b491067f6babaf",
    "src/driver_protocol.rs": "40eb44ad164d94a1ee0fb5276bff76989094c0e4460ecd6ea9a19f3010694989",
    "src/emit.rs": "6e560c91fe926bae74957c37cbf6377302c3a1f166d10c32aa2c5206af338728",
    "src/lib.rs": "310498561d132636fb6be858922f1e1e82704d0a462ec2adffaf419ff47cc0fe",
    "src/mir_call.rs": "bab1f2bffabea241f9854ee5fb181b75d857a557afd5f3cdeeaf952c825d2bf0",
    "src/mir_lower.rs": "b6165150750e268afc3cfdb6352522286b462a599924465300534a5f04ef5bbb",
    "src/source_map.rs": "0ef386555c67d9dca38c5b6d02ceb5b546e9516369865417293edf556fc07bf3",
    "tests/negative_corpus.rs": "f5c1e6829d7f732023db0ccb62e1ad69bf60b2e6c023cbb9295507ee7e06f57d",
    "tests/positive_corpus.rs": "1fd5554912778446569de7883211f74a74646f65f78f774cfbabf2144db06d75",
    "tests/support/rustc_harness.rs": "066ae7181bfb31311e64db4eb2e089848e885e09d8ecd4392311ba761ded06a1",
}

PACKAGE_VERSION = "0.1.0-profile-v1-staging"
FRONTEND_ID = "frontend.rust.rust2vir.candidate.v1"
TOOLCHAIN_ID = "toolchain.rust.nightly-2025-06-01.candidate.v1"
PROFILE_ENTRY_SHA256 = "1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057"
PROFILE_REGISTRY_SHA256 = "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75"
TOOLCHAIN_DISTRIBUTION_SHA256 = "86dab73dadd3a3184064e7d7da7e878562eba4cfc4c8a969bc8f44a5e865c90a"


SUCCESSOR_MODULE = r'''//! Staging-only compiled identities for the semantic-profile successor.

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
        (
            "semantic_profile".to_owned(),
            string("mpk.rust.checked.v0"),
        ),
        ("source_language".to_owned(), string("rust")),
    ]))
}

pub fn selection_envelope(
    package: &str,
    crate_name: &str,
    function: &str,
) -> JsonValue {
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
'''


PROTOCOL_IDENTITY_TEST = r'''use rust2vir_internal::driver_protocol::{
    parse_output_transport, parse_request_transport, DriverStatus,
};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::sha256::{hex, Sha256};

const PREDECESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
const SUCCESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");

fn parse(bytes: &[u8]) -> JsonValue {
    json::parse(bytes, bytes.len()).expect("strict driver vector")
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().expect("fixture")["value"].clone();
    canonical_transport(&value)
}

fn canonical_transport(value: &JsonValue) -> Vec<u8> {
    let mut bytes = json::canonical(value).expect("canonical fixture");
    bytes.push(b'\n');
    bytes
}

fn rehash(value: &mut JsonValue, field: &str, domain: &[u8]) {
    value
        .as_object_mut()
        .expect("hashed object")
        .remove(field);
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(&json::canonical(value).expect("canonical hash preimage"));
    value.as_object_mut().expect("hashed object").insert(
        field.to_owned(),
        JsonValue::String(hex(&hasher.finish())),
    );
}

fn replace_schema(value: &mut JsonValue, schema: &str) {
    value.as_object_mut().expect("schema object").insert(
        "schema".to_owned(),
        JsonValue::String(schema.to_owned()),
    );
}

fn replace_nested_schema(value: &mut JsonValue, field: &str, schema: &str) {
    let nested = value
        .as_object_mut()
        .expect("result object")
        .get_mut(field)
        .expect("nested protocol artifact")
        .as_object_mut()
        .expect("nested protocol object");
    nested.insert(
        "schema".to_owned(),
        JsonValue::String(schema.to_owned()),
    );
}

#[test]
fn subordinate_protocol_accepts_only_successor_identities() {
    let old = parse(PREDECESSOR);
    let old = old.as_object().expect("predecessor vector");
    let new = parse(SUCCESSOR);
    let new = new.as_object().expect("successor vector");

    assert!(parse_request_transport(&transport(&old["valid_request"])).is_err());
    let request = parse_request_transport(&transport(&new["valid_request"]))
        .expect("successor request");
    assert!(parse_output_transport(
        &transport(&old["valid_lowered"]),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .is_err());
    parse_output_transport(
        &transport(&new["valid_lowered"]),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .expect("successor result");

    let valid_request = new["valid_request"]
        .as_object()
        .expect("request envelope")["value"]
        .clone();
    let mut predecessor_request_schema = valid_request.clone();
    replace_schema(
        &mut predecessor_request_schema,
        "mpk.rust.driver.request.v0",
    );
    rehash(
        &mut predecessor_request_schema,
        "request_fingerprint",
        b"MPK-RUST-DRIVER-REQUEST-1.0",
    );
    assert!(parse_request_transport(&canonical_transport(&predecessor_request_schema)).is_err());

    let mut predecessor_request_domain = valid_request;
    rehash(
        &mut predecessor_request_domain,
        "request_fingerprint",
        b"MPK-RUST-DRIVER-REQUEST-0.1",
    );
    assert!(parse_request_transport(&canonical_transport(&predecessor_request_domain)).is_err());

    let valid_result = new["valid_lowered"]
        .as_object()
        .expect("lowered envelope")["value"]
        .clone();
    for (field, schema) in [
        (None, "mpk.rust.driver.v0"),
        (Some("raw_lowering"), "mpk.rust.driver.lowering.v0"),
        (
            Some("raw_source_map"),
            "mpk.rust.driver.raw_source_map.v0",
        ),
    ] {
        let mut predecessor_schema = valid_result.clone();
        match field {
            Some(field) => replace_nested_schema(&mut predecessor_schema, field, schema),
            None => replace_schema(&mut predecessor_schema, schema),
        }
        rehash(
            &mut predecessor_schema,
            "payload_hash",
            b"MPK-RUST-DRIVER-PAYLOAD-1.0",
        );
        assert!(parse_output_transport(
            &canonical_transport(&predecessor_schema),
            &request,
            DriverStatus::Lowered.exit_code(),
            false,
        )
        .is_err());
    }

    let mut predecessor_payload_domain = valid_result;
    rehash(
        &mut predecessor_payload_domain,
        "payload_hash",
        b"MPK-RUST-DRIVER-PAYLOAD-0.1",
    );
    assert!(parse_output_transport(
        &canonical_transport(&predecessor_payload_domain),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .is_err());

    let lowered_envelope = new["valid_lowered"]
        .as_object()
        .expect("lowered envelope");
    let lowered = lowered_envelope["value"]
        .as_object()
        .expect("lowered result");
    assert_eq!(lowered["schema"].as_str(), Some("mpk.rust.driver.v1"));
    let raw_lowering = lowered["raw_lowering"]
        .as_object()
        .expect("raw lowering");
    let raw_source_map = lowered["raw_source_map"]
        .as_object()
        .expect("raw source map");
    assert_eq!(
        raw_lowering["schema"].as_str(),
        Some("mpk.rust.driver.lowering.v1")
    );
    assert_eq!(
        raw_source_map["schema"].as_str(),
        Some("mpk.rust.driver.raw_source_map.v1")
    );
}
'''


PREDECESSOR_IDENTITY_TEST = r'''use rust2vir_internal::driver_protocol::{
    parse_output_transport, parse_request_transport, DriverStatus,
};
use rust2vir_internal::json::{self, JsonValue};

const PREDECESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
const SUCCESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");

fn parse(bytes: &[u8]) -> JsonValue {
    json::parse(bytes, bytes.len()).expect("strict driver vector")
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().expect("fixture")["value"].clone();
    let mut bytes = json::canonical(&value).expect("canonical fixture");
    bytes.push(b'\n');
    bytes
}

#[test]
fn predecessor_subordinate_parser_rejects_successor_identities() {
    let old = parse(PREDECESSOR);
    let old = old.as_object().expect("predecessor vector");
    let new = parse(SUCCESSOR);
    let new = new.as_object().expect("successor vector");
    let request = parse_request_transport(&transport(&old["valid_request"]))
        .expect("predecessor request");
    assert!(parse_request_transport(&transport(&new["valid_request"])).is_err());
    assert!(parse_output_transport(
        &transport(&new["valid_lowered"]),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .is_err());
}
'''


def replace_once(source: str, old: str, new: str, path: str) -> str:
    count = source.count(old)
    if count != 1:
        raise ValueError(f"{path}: expected one migration anchor, found {count}")
    return source.replace(old, new)


def transform_lib(source: str) -> str:
    source = replace_once(
        source,
        "pub mod stable_id;",
        "pub mod stable_id;\npub mod successor;",
        "src/lib.rs",
    )
    return replace_once(
        source,
        'pub const PACKAGE_VERSION: &str = "0.1.0";',
        f'pub const PACKAGE_VERSION: &str = "{PACKAGE_VERSION}";',
        "src/lib.rs",
    )


def transform_contracts(source: str) -> str:
    source = replace_once(
        source,
        'const CONTRACT_HASH_DOMAIN: &[u8] = b"MPK-CONTRACT-0.1";',
        'const CONTRACT_HASH_DOMAIN: &[u8] = b"MPK-CONTRACT-1.0";',
        "src/contract_typecheck.rs",
    )
    old = '''        (
            "semantic_profile".to_owned(),
            JsonValue::String(RUST_SEMANTIC_PROFILE.to_owned()),
        ),
        ("semantic_parameters".to_owned(), semantic_parameters),'''
    new = '''        (
            "semantic_context".to_owned(),
            crate::successor::semantic_context(target_id, pointer_width),
        ),'''
    source = replace_once(source, old, new, "src/contract_typecheck.rs")
    return replace_once(
        source,
        '''    let semantic_parameters = object([
        ("overflow_mode", JsonValue::String("checked".to_owned())),
        ("panic_mode", JsonValue::String("abort".to_owned())),
        (
            "pointer_width",
            JsonValue::Number(pointer_width.to_string()),
        ),
        ("target_id", JsonValue::String(target_id.to_owned())),
    ]);
''',
        "",
        "src/contract_typecheck.rs",
    )


def transform_mir(source: str) -> str:
    source = replace_once(
        source,
        'const VIR_HASH_DOMAIN: &[u8] = b"MPK-VIR-0.1";',
        'const VIR_HASH_DOMAIN: &[u8] = b"MPK-VIR-1.0";',
        "src/mir_lower.rs",
    )
    source = replace_once(
        source,
        '''    let semantic_parameters = JsonValue::Object(BTreeMap::from([
        (
            "target_id".to_owned(),
            JsonValue::String(request.target().to_owned()),
        ),
        (
            "pointer_width".to_owned(),
            JsonValue::Number(request.pointer_width().to_string()),
        ),
        (
            "overflow_mode".to_owned(),
            JsonValue::String("checked".to_owned()),
        ),
        (
            "panic_mode".to_owned(),
            JsonValue::String("abort".to_owned()),
        ),
    ]));
''',
        "",
        "src/mir_lower.rs",
    )
    source = replace_once(
        source,
        '''        ("schema".to_owned(), string("mpk.vir.v0")),
        ("source_language".to_owned(), string("rust")),
        ("semantic_profile".to_owned(), string("mpk.rust.checked.v0")),
        ("semantic_parameters".to_owned(), semantic_parameters),''',
        '''        ("schema".to_owned(), string("mpk.vir.v1")),
        (
            "semantic_context".to_owned(),
            request.semantic_context().clone(),
        ),''',
        "src/mir_lower.rs",
    )
    source = replace_once(
        source,
        "let source_map = raw_source_map(&vir_hash, entries);",
        "let source_map = raw_source_map(&vir_hash, entries, request.semantic_context().clone());",
        "src/mir_lower.rs",
    )
    return replace_once(
        source,
        'string("mpk.rust.driver.lowering.v0")',
        'string("mpk.rust.driver.lowering.v1")',
        "src/mir_lower.rs",
    )


def transform_mir_call(source: str) -> str:
    source = replace_once(
        source,
        '''use rust2vir_internal::contract::{
    ContractSet, ContractType, NormalizedContract, RUST_SEMANTIC_PROFILE,
};''',
        '''use rust2vir_internal::contract::{ContractSet, ContractType, NormalizedContract};''',
        "src/mir_call.rs",
    )
    source = replace_once(
        source,
        '''        vector.semantic_context_matches = match (caller_contract, callee_contract) {
            (Some(caller), Some(callee)) => {
                contract_member(&caller.value, "semantic_profile")
                    == contract_member(&callee.value, "semantic_profile")
                    && caller
                        .value
                        .as_object()
                        .and_then(|value| value.get("semantic_parameters"))
                        == callee
                            .value
                            .as_object()
                            .and_then(|value| value.get("semantic_parameters"))
            }
            _ => false,
        };''',
        '''        vector.semantic_context_matches = match (caller_contract, callee_contract) {
            (Some(caller), Some(callee)) => {
                let caller_context = caller
                    .value
                    .as_object()
                    .and_then(|value| value.get("semantic_context"));
                let callee_context = callee
                    .value
                    .as_object()
                    .and_then(|value| value.get("semantic_context"));
                caller_context == Some(context.request.semantic_context())
                    && callee_context == caller_context
            }
            _ => false,
        };''',
        "src/mir_call.rs",
    )
    source = replace_once(
        source,
        '''    let unit_id = request.selection().1;
    let expected_parameters = expected_semantic_parameters(request);
    contract.function_id == function.function_id''',
        '''    let unit_id = request.selection().1;
    contract.function_id == function.function_id''',
        "src/mir_call.rs",
    )
    return replace_once(
        source,
        '''        && contract_member(&contract.value, "semantic_profile") == Some(RUST_SEMANTIC_PROFILE)
        && contract_member(&contract.value, "contract_hash") == Some(&contract.contract_hash)
        && contract
            .value
            .as_object()
            .and_then(|value| value.get("semantic_parameters"))
            == Some(&expected_parameters)
}

fn expected_semantic_parameters(request: &DriverRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "target_id".to_owned(),
            JsonValue::String(request.target().to_owned()),
        ),
        (
            "pointer_width".to_owned(),
            JsonValue::Number(request.pointer_width().to_string()),
        ),
        (
            "overflow_mode".to_owned(),
            JsonValue::String("checked".to_owned()),
        ),
        (
            "panic_mode".to_owned(),
            JsonValue::String("abort".to_owned()),
        ),
    ]))
}''',
        '''        && contract_member(&contract.value, "contract_hash") == Some(&contract.contract_hash)
        && contract
            .value
            .as_object()
            .and_then(|value| value.get("semantic_context"))
            == Some(request.semantic_context())
}''',
        "src/mir_call.rs",
    )


def transform_source_map(source: str) -> str:
    source = replace_once(
        source,
        "pub fn raw_source_map(source_ir_hash: &str, mut entries: Vec<SourceMapEntry>) -> JsonValue {",
        "pub fn raw_source_map(\n    source_ir_hash: &str,\n    mut entries: Vec<SourceMapEntry>,\n    semantic_context: JsonValue,\n) -> JsonValue {",
        "src/source_map.rs",
    )
    source = replace_once(
        source,
        '''        (
            "schema".to_owned(),
            string("mpk.rust.driver.raw_source_map.v0"),
        ),
        ("source_ir_schema".to_owned(), string("mpk.vir.v0")),''',
        '''        (
            "schema".to_owned(),
            string("mpk.rust.driver.raw_source_map.v1"),
        ),
        (
            "semantic_context".to_owned(),
            semantic_context,
        ),
        ("source_ir_schema".to_owned(), string("mpk.vir.v1")),''',
        "src/source_map.rs",
    )
    return source


def transform_driver(source: str) -> str:
    replacements = {
        'b"MPK-RUST-DRIVER-REQUEST-0.1"': 'b"MPK-RUST-DRIVER-REQUEST-1.0"',
        'b"MPK-RUST-DRIVER-PAYLOAD-0.1"': 'b"MPK-RUST-DRIVER-PAYLOAD-1.0"',
        'b"MPK-VIR-0.1"': 'b"MPK-VIR-1.0"',
        '"mpk.rust.driver.request.v0"': '"mpk.rust.driver.request.v1"',
        '"mpk.rust.driver.v0"': '"mpk.rust.driver.v1"',
        '"mpk.rust.driver.lowering.v0"': '"mpk.rust.driver.lowering.v1"',
        '"mpk.rust.driver.raw_source_map.v0"': '"mpk.rust.driver.raw_source_map.v1"',
        '"mpk.vir.v0"': '"mpk.vir.v1"',
        '"mpk.release.bundle_registry.v0"': '"mpk.release.bundle_registry.v1"',
        '"mpk.release.registry.v0"': '"mpk.release.registry.v1"',
    }
    for old, new in replacements.items():
        if old not in source:
            raise ValueError(f"src/driver_protocol.rs: missing migration anchor {old}")
        source = source.replace(old, new)
    source = replace_once(
        source,
        '''    "selection",
    "semantic_parameters",
    "semantic_profile",
    "source_inventory",
    "source_inventory_hash",
    "source_language",''',
        '''    "selection",
    "semantic_context",
    "source_inventory",
    "source_inventory_hash",''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    "selection",
    "semantic_parameters",
    "semantic_profile",
    "source_inventory_hash",
    "source_language",''',
        '''    "selection",
    "semantic_context",
    "source_inventory_hash",''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        let selection = object_member(self.root(), "selection")
            .and_then(JsonValue::as_object)
            .expect("validated request selection");''',
        '''        let selection = object_member(self.root(), "selection")
            .and_then(JsonValue::as_object)
            .and_then(|selection| selection.get("value"))
            .and_then(JsonValue::as_object)
            .expect("validated request selection");''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    pub fn source_inventory_hash(&self) -> &str {
        &self.source_inventory_hash
    }

    pub fn selection(&self) -> (&str, &str, &str) {''',
        '''    pub fn source_inventory_hash(&self) -> &str {
        &self.source_inventory_hash
    }

    pub fn semantic_context(&self) -> &JsonValue {
        object_member(self.root(), "semantic_context")
            .expect("validated request semantic context")
    }

    pub fn selection(&self) -> (&str, &str, &str) {''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        object_member(self.root(), "semantic_parameters")
            .and_then(JsonValue::as_object)
            .and_then(|parameters| parameters.get("target_id"))''',
        '''        semantic_parameter_value(self.root())
            .and_then(|parameters| parameters.get("target_id"))''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        let width = object_member(self.root(), "semantic_parameters")
            .and_then(JsonValue::as_object)
            .and_then(|parameters| parameters.get("pointer_width"))''',
        '''        let width = semantic_parameter_value(self.root())
            .and_then(|parameters| parameters.get("pointer_width"))''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        (
            "selection".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "crate".to_owned(),
                    JsonValue::String(request.selection.crate_name.clone()),
                ),
                (
                    "function".to_owned(),
                    JsonValue::String(request.selection.function.clone()),
                ),
                ("kind".to_owned(), JsonValue::String("lib".to_owned())),
                (
                    "package".to_owned(),
                    JsonValue::String(request.selection.package.clone()),
                ),
            ])),
        ),
        (
            "semantic_parameters".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "overflow_mode".to_owned(),
                    JsonValue::String("checked".to_owned()),
                ),
                (
                    "panic_mode".to_owned(),
                    JsonValue::String("abort".to_owned()),
                ),
                (
                    "pointer_width".to_owned(),
                    JsonValue::Number(request.target.pointer_width().to_string()),
                ),
                (
                    "target_id".to_owned(),
                    JsonValue::String(request.target.id().to_owned()),
                ),
            ])),
        ),
        (
            "semantic_profile".to_owned(),
            JsonValue::String("mpk.rust.checked.v0".to_owned()),
        ),
        ("source_inventory".to_owned(), source_inventory),
        (
            "source_language".to_owned(),
            JsonValue::String("rust".to_owned()),
        ),''',
        '''        (
            "selection".to_owned(),
            crate::successor::selection_envelope(
                &request.selection.package,
                &request.selection.crate_name,
                &request.selection.function,
            ),
        ),
        (
            "semantic_context".to_owned(),
            crate::successor::semantic_context(
                request.target.id(),
                request.target.pointer_width(),
            ),
        ),
        ("source_inventory".to_owned(), source_inventory),''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    if string(root, "schema")? != "mpk.rust.driver.request.v1"
        || string(root, "source_language")? != "rust"
        || string(root, "semantic_profile")? != "mpk.rust.checked.v0"
        || string(root, "limit_profile")? != "mpk.rust.limits.v0"''',
        '''    if string(root, "schema")? != "mpk.rust.driver.request.v1"
        || string(root, "limit_profile")? != "mpk.rust.limits.v0"''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    validate_semantic_parameters(object(root, "semantic_parameters")?)?;
    validate_selection(object(root, "selection")?)?;''',
        '''    validate_semantic_context(object(root, "semantic_context")?)?;
    validate_selection_envelope(object(root, "selection")?)?;''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    let parameters = object(root, "semantic_parameters")?;
    if string(compiler, "target")? != string(parameters, "target_id")? {''',
        '''    let parameters =
        semantic_parameter_value(root).ok_or(DriverProtocolCode::Shape)?;
    if string(compiler, "target")? != string(parameters, "target_id")? {''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        "fn validate_semantic_parameters(\n",
        '''fn semantic_parameter_value(
    root: &BTreeMap<String, JsonValue>,
) -> Option<&BTreeMap<String, JsonValue>> {
    root.get("semantic_context")?
        .as_object()?
        .get("semantic_parameters")?
        .as_object()?
        .get("value")?
        .as_object()
}

fn validate_semantic_context(
    context: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(
        context,
        &[
            "profile_entry_sha256",
            "profile_registry",
            "semantic_parameters",
            "semantic_profile",
            "source_language",
        ],
    )?;
    let parameters = object(context, "semantic_parameters")?;
    closed(parameters, &["schema", "value"])?;
    if string(parameters, "schema")? != crate::successor::PARAMETERS_SCHEMA {
        return Err(DriverProtocolCode::Shape.into());
    }
    let value = object(parameters, "value")?;
    validate_semantic_parameters(value)?;
    let width = u8::try_from(integer(value, "pointer_width")?)
        .map_err(|_| DriverProtocolCode::Shape)?;
    let expected = crate::successor::semantic_context(string(value, "target_id")?, width);
    if JsonValue::Object(context.clone()) != expected {
        return Err(DriverProtocolCode::Identity.into());
    }
    Ok(())
}

fn validate_semantic_parameters(
''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        "fn validate_selection(selection: &BTreeMap<String, JsonValue>) -> Result<(), DriverProtocolError> {",
        '''fn validate_selection_envelope(
    selection: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(selection, &["schema", "value"])?;
    if string(selection, "schema")? != crate::successor::SELECTION_SCHEMA {
        return Err(DriverProtocolCode::Shape.into());
    }
    validate_selection(object(selection, "value")?)
}

fn validate_selection(selection: &BTreeMap<String, JsonValue>) -> Result<(), DriverProtocolError> {''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    if string(registry, "schema")? != "mpk.release.bundle_registry.v1"
        || string(registry, "id")? != "mpk.release.registry.v1"''',
        '''    if string(registry, "schema")? != "mpk.release.bundle_registry.v1"
        || string(registry, "id")? != "mpk.release.registry.v1"''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        || !release_id(string(frontend, "bundle_id")?)
        || !sha256(string(frontend, "binary_sha256")?)''',
        '''        || string(frontend, "bundle_id")? != crate::successor::FRONTEND_ID
        || !sha256(string(frontend, "binary_sha256")?)''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    if !release_id(string(toolchain, "bundle_id")?)
        || !sha256(string(toolchain, "distribution_sha256")?)''',
        '''    if string(toolchain, "bundle_id")? != crate::successor::TOOLCHAIN_ID
        || string(toolchain, "distribution_sha256")?
            != crate::successor::TOOLCHAIN_DISTRIBUTION_SHA256''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''    if vir.get("source_language") != root.get("source_language")
        || vir.get("semantic_profile") != root.get("semantic_profile")
        || vir.get("semantic_parameters") != root.get("semantic_parameters")
    {
        return Err(DriverProtocolCode::Identity.into());
    }''',
        '''    if vir.get("semantic_context") != root.get("semantic_context") {
        return Err(DriverProtocolCode::Identity.into());
    }''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        &["entries", "schema", "source_ir_hash", "source_ir_schema"],''',
        '''        &[
            "entries",
            "schema",
            "semantic_context",
            "source_ir_hash",
            "source_ir_schema",
        ],''',
        "src/driver_protocol.rs",
    )
    source = replace_once(
        source,
        '''        || string(map, "source_ir_hash")? != vir_hash
    {
        return Err(DriverProtocolCode::Identity.into());
    }''',
        '''        || string(map, "source_ir_hash")? != vir_hash
        || map.get("semantic_context") != request.root().get("semantic_context")
        || map.get("semantic_context") != vir.get("semantic_context")
    {
        return Err(DriverProtocolCode::Identity.into());
    }''',
        "src/driver_protocol.rs",
    )
    return source


def transform_emit(source: str) -> str:
    for old, new in {
        'b"MPK-VIR-0.1"': 'b"MPK-VIR-1.0"',
        'b"MPK-SOURCE-MAP-0.1"': 'b"MPK-SOURCE-MAP-1.0"',
        'b"MPK-SOURCE-MANIFEST-0.1"': 'b"MPK-SOURCE-MANIFEST-1.0"',
        '"mpk.source_map.v0"': '"mpk.source_map.v1"',
        '"mpk.source_manifest.v0"': '"mpk.source_manifest.v1"',
        '"mpk.vir.v0"': '"mpk.vir.v1"',
        '"mpk.frontend.cli.v0"': '"mpk.frontend.cli.v1"',
    }.items():
        if old not in source:
            raise ValueError(f"src/emit.rs: missing migration anchor {old}")
        source = source.replace(old, new)
    source = replace_once(
        source,
        "use crate::session;\n",
        "",
        "src/emit.rs",
    )
    source = replace_once(
        source,
        '''    let language_configuration = rust_language_configuration(request, core_prelude)?;
    let mut manifest = JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string_value("mpk.source_manifest.v1")),
        ("source_language".to_owned(), string_value("rust")),
        (
            "semantic_profile".to_owned(),
            required(request_root, "semantic_profile")?.clone(),
        ),
        (
            "semantic_parameters".to_owned(),
            required(request_root, "semantic_parameters")?.clone(),
        ),''',
        '''    let _ = core_prelude;
    let mut manifest = JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string_value("mpk.source_manifest.v1")),
        (
            "semantic_context".to_owned(),
            required(request_root, "semantic_context")?.clone(),
        ),''',
        "src/emit.rs",
    )
    source = replace_once(
        source,
        '''                ("language_configuration".to_owned(), language_configuration),
''',
        "",
        "src/emit.rs",
    )
    source = replace_once(
        source,
        '''        (
            "selection".to_owned(),
            required(request_root, "selection")?.clone(),
        ),
        (
            "semantic_parameters".to_owned(),
            required(request_root, "semantic_parameters")?.clone(),
        ),
        (
            "semantic_profile".to_owned(),
            required(request_root, "semantic_profile")?.clone(),
        ),
        ("source_language".to_owned(), string_value("rust")),''',
        '''        (
            "selection".to_owned(),
            required(request_root, "selection")?.clone(),
        ),
        (
            "semantic_context".to_owned(),
            required(request_root, "semantic_context")?.clone(),
        ),''',
        "src/emit.rs",
    )
    source = replace_once(
        source,
        '''        ("selection".to_owned(), selection(request)),
        (
            "semantic_parameters".to_owned(),
            semantic_parameters(request),
        ),
        (
            "semantic_profile".to_owned(),
            string_value("mpk.rust.checked.v0"),
        ),
        ("source_language".to_owned(), string_value("rust")),''',
        '''        ("selection".to_owned(), selection(request)),
        (
            "semantic_context".to_owned(),
            semantic_context(request),
        ),''',
        "src/emit.rs",
    )
    source = replace_once(
        source,
        '''fn selection(request: &LowerRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "crate".to_owned(),
            string_value(&request.selection.crate_name),
        ),
        (
            "function".to_owned(),
            string_value(&request.selection.function),
        ),
        ("kind".to_owned(), string_value("lib")),
        (
            "package".to_owned(),
            string_value(&request.selection.package),
        ),
    ]))
}

fn semantic_parameters(request: &LowerRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("overflow_mode".to_owned(), string_value("checked")),
        ("panic_mode".to_owned(), string_value("abort")),
        (
            "pointer_width".to_owned(),
            number_value(i64::from(request.target.pointer_width())),
        ),
        ("target_id".to_owned(), string_value(request.target.id())),
    ]))
}''',
        '''fn selection(request: &LowerRequest) -> JsonValue {
    crate::successor::selection_envelope(
        &request.selection.package,
        &request.selection.crate_name,
        &request.selection.function,
    )
}

fn semantic_context(request: &LowerRequest) -> JsonValue {
    crate::successor::semantic_context(request.target.id(), request.target.pointer_width())
}''',
        "src/emit.rs",
    )
    start = source.find("fn rust_language_configuration(")
    end = source.find("fn public_units(", start)
    if start < 0 or end < 0:
        raise ValueError("src/emit.rs: missing language-configuration migration anchor")
    source = source[:start] + source[end:]
    return source


def transform_cli(source: str) -> str:
    source = replace_once(
        source,
        '''    format!(
        concat!(
            "{{\\"diagnostics\\":[{{\\"code\\":\\"{}\\",\\"message\\":\\"{}\\"}}],",
            "\\"phase\\":\\"{}\\",\\"rejected_features\\":[],",
            "\\"schema\\":\\"mpk.frontend.cli.v0\\",",
            "\\"selection\\":{{\\"crate\\":\\"{}\\",\\"function\\":\\"{}\\",",
            "\\"kind\\":\\"lib\\",\\"package\\":\\"{}\\"}},",
            "\\"semantic_parameters\\":{{\\"overflow_mode\\":\\"checked\\",",
            "\\"panic_mode\\":\\"abort\\",\\"pointer_width\\":{},\\"target_id\\":\\"{}\\"}},",
            "\\"semantic_profile\\":\\"mpk.rust.checked.v0\\",",
            "\\"source_language\\":\\"rust\\",\\"status\\":\\"{}\\"}}"
        ),
        code,
        message,
        phase,
        request.selection.crate_name,
        request.selection.function,
        request.selection.package,
        request.target.pointer_width(),
        request.target.id(),
        status.as_str()
    )''',
        '''    let envelope = crate::json::JsonValue::Object(BTreeMap::from([
        (
            "diagnostics".to_owned(),
            crate::json::JsonValue::Array(vec![crate::json::JsonValue::Object(
                BTreeMap::from([
                    ("code".to_owned(), crate::json::JsonValue::String(code.to_owned())),
                    (
                        "message".to_owned(),
                        crate::json::JsonValue::String(message.to_owned()),
                    ),
                ]),
            )]),
        ),
        (
            "phase".to_owned(),
            crate::json::JsonValue::String(phase.to_owned()),
        ),
        (
            "rejected_features".to_owned(),
            crate::json::JsonValue::Array(Vec::new()),
        ),
        (
            "schema".to_owned(),
            crate::json::JsonValue::String("mpk.frontend.cli.v1".to_owned()),
        ),
        (
            "selection".to_owned(),
            crate::successor::selection_envelope(
                &request.selection.package,
                &request.selection.crate_name,
                &request.selection.function,
            ),
        ),
        (
            "semantic_context".to_owned(),
            crate::successor::semantic_context(
                request.target.id(),
                request.target.pointer_width(),
            ),
        ),
        (
            "status".to_owned(),
            crate::json::JsonValue::String(status.as_str().to_owned()),
        ),
    ]));
    String::from_utf8(crate::json::canonical(&envelope).expect("constructed JSON"))
        .expect("canonical JSON is UTF-8")''',
        "src/cli.rs",
    )
    source = replace_once(
        source,
        '''    let frontend_bundle_id = take_identifier(&mut singleton, "--frontend-bundle-id")?;''',
        '''    if take_identifier(&mut singleton, "--profile-registry-id")?
        != crate::successor::PROFILE_REGISTRY_ID
        || take(&mut singleton, "--profile-registry-revision")?
            != crate::successor::PROFILE_REGISTRY_REVISION.to_string()
        || take_sha256(&mut singleton, "--profile-registry-sha256")?
            != crate::successor::PROFILE_REGISTRY_SHA256
        || take_sha256(&mut singleton, "--profile-entry-sha256")?
            != crate::successor::PROFILE_ENTRY_SHA256
    {
        return Err(CliError);
    }
    let frontend_bundle_id = take_identifier(&mut singleton, "--frontend-bundle-id")?;''',
        "src/cli.rs",
    )
    source = replace_once(
        source,
        '''            | "--frontend-bundle-id"''',
        '''            | "--profile-registry-id"
            | "--profile-registry-revision"
            | "--profile-registry-sha256"
            | "--profile-entry-sha256"
            | "--frontend-bundle-id"''',
        "src/cli.rs",
    )
    return source


def transform(path: str, source: str) -> str:
    if path == "src/lib.rs":
        return transform_lib(source)
    if path == "src/contract_typecheck.rs":
        return transform_contracts(source)
    if path == "src/mir_call.rs":
        return transform_mir_call(source)
    if path == "src/mir_lower.rs":
        return transform_mir(source)
    if path == "src/source_map.rs":
        return transform_source_map(source)
    if path == "src/driver_protocol.rs":
        return transform_driver(source)
    if path == "src/emit.rs":
        return transform_emit(source)
    if path == "src/cli.rs":
        return transform_cli(source)
    raise ValueError(f"unsupported production overlay source {path}")


def transform_positive_test(source: str, identities: dict[str, str]) -> str:
    source = source.replace(
        "fn positive_corpus_is_complete_snapshot_backed_and_byte_deterministic()",
        "fn successor_positive_corpus_is_complete_snapshot_backed_and_byte_deterministic()",
    )
    source = replace_once(
        source,
        'const SHA_FRONTEND: &str = "60b148614f2a22734b45c8ba0366c94505ea735e82e05f3df7c2b03b3ba2b2c4";',
        f'const SHA_FRONTEND: &str = "{identities["frontend_sha256"]}";',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        'const SHA_DRIVER: &str = "e18ada1ff29d0a9dce87230698cd89d77274633de716559ada1dc34f40e0f3ee";',
        f'const SHA_DRIVER: &str = "{identities["driver_sha256"]}";',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        'const SHA_TOOLCHAIN: &str = "cdaa0ae4d4f56da86f403d58799fd2298f078b043d8392311487315cbcc2c63f";',
        f'const SHA_TOOLCHAIN: &str = "{TOOLCHAIN_DISTRIBUTION_SHA256}";',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        'const REGISTRY_SHA256: &str = "bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98";',
        f'const REGISTRY_SHA256: &str = "{identities["registry_sha256"]}";',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        '''    source_manifest: Vec<u8>,
    captured_sources: Vec<String>,''',
        '''    source_manifest: Vec<u8>,
    private_request: Vec<u8>,
    private_result: Vec<u8>,
    raw_lowering: Vec<u8>,
    raw_source_map: Vec<u8>,
    captured_sources: Vec<String>,''',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        '''    let update = std::env::var_os(UPDATE_ENV).is_some();
    let expected_root = fixture_base();
    let output_root = expected_root.clone();''',
        '''    let _ = UPDATE_ENV;
    let update = true;
    let expected_root = fixture_base();
    let output_root = PathBuf::from("/mpk/work/generated");''',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        '''            (
                "source_manifest_frontend",
                "source-manifest.frontend.json",
                &first.source_manifest,
            ),
        ];''',
        '''            (
                "source_manifest_frontend",
                "source-manifest.frontend.json",
                &first.source_manifest,
            ),
            ("private_request", "driver-request.json", &first.private_request),
            ("private_result", "driver-result.json", &first.private_result),
            ("raw_lowering", "raw-lowering.json", &first.raw_lowering),
            ("raw_source_map", "raw-source-map.json", &first.raw_source_map),
        ];''',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        '''    let driver_transport = encode_lowered(
        &private_request,
        lowering.raw_lowering,
        lowering.raw_source_map,
    )''',
        '''    let raw_lowering = json::canonical(&lowering.raw_lowering)
        .expect("canonical raw lowering");
    let raw_source_map = json::canonical(&lowering.raw_source_map)
        .expect("canonical raw source map");
    let driver_transport = encode_lowered(
        &private_request,
        lowering.raw_lowering,
        lowering.raw_source_map,
    )''',
        "tests/positive_corpus.rs",
    )
    source = replace_once(
        source,
        '''        source_manifest,
        captured_sources,''',
        '''        source_manifest,
        private_request: private_request.transport().to_vec(),
        private_result: driver_transport,
        raw_lowering,
        raw_source_map,
        captured_sources,''',
        "tests/positive_corpus.rs",
    )
    source = source.replace(
        '"frontend.rust.rust2vir.candidate.v0"',
        f'"{FRONTEND_ID}"',
    )
    source = source.replace(
        '"mpk.release.registry.v0"',
        '"mpk.release.registry.v1"',
    )
    source = source.replace(
        '"toolchain.rust.nightly-2025-06-01.candidate.v0"',
        f'"{TOOLCHAIN_ID}"',
    )
    component_hashes = {
        "0f448df12a3bb58ca6ab51fcee4c470b117ce7072a02b489ab214454f302a479": "6d8ebe276575c5019abdc97051baf78e166354249eca4d6b65f638c5fb171005",
        "3f61be824744b3ad52281dbebaba6718c10ed6af9a82b936a02419b7f43f5693": "7698b22d00656113340f692fd9212a1494077fd470f924948945e690da401292",
        "a1c72b8bdb5dd4d589f386fc0142adee3274ebcb104d69203ad1f4ce5600c5c9": "8f606996b669eb0f4314309d145d93c6eeaad8b261791584387bcff46ccafb0a",
        "73019eb46832161dad2e55a17cc044ff4523441643e5bc1b1ab1c68408961956": "d8c45533753e17186cefde3e0830f7b358a8b4c818eb732d8814a31861335a15",
    }
    for old, new in component_hashes.items():
        source = source.replace(old, new)
    return source


def transform_negative_test(source: str) -> str:
    source = source.replace(
        'include_bytes!("../testdata/rust-driver-v0.json")',
        'include_bytes!("../testdata/rust-driver-v1.json")',
    )
    return replace_once(
        source,
        '''    let selection = root["selection"].as_object().unwrap();''',
        '''    let selection = root["selection"].as_object().unwrap()["value"]
        .as_object()
        .unwrap();''',
        "tests/negative_corpus.rs",
    )


def transform_rustc_harness(source: str) -> str:
    source = replace_once(
        source,
        'const VECTOR: &[u8] = include_bytes!("../../testdata/rust-driver-v0.json");',
        'const VECTOR: &[u8] = include_bytes!("../../testdata/rust-driver-v1.json");',
        "tests/support/rustc_harness.rs",
    )
    source = replace_once(
        source,
        '''    let parameters = root
        .get_mut("semantic_parameters")
        .expect("semantic parameters")
        .as_object_mut()
        .expect("semantic parameters object");''',
        '''    let context = root
        .get_mut("semantic_context")
        .expect("semantic context")
        .as_object_mut()
        .expect("semantic context object");
    let parameters = context
        .get_mut("semantic_parameters")
        .expect("semantic parameters envelope")
        .as_object_mut()
        .expect("semantic parameters envelope object")
        .get_mut("value")
        .expect("semantic parameters value")
        .as_object_mut()
        .expect("semantic parameters object");''',
        "tests/support/rustc_harness.rs",
    )
    source = replace_once(
        source,
        '''    root.get_mut("selection")
        .expect("selection")
        .as_object_mut()
        .expect("selection object")
        .insert(''',
        '''    root.get_mut("selection")
        .expect("selection envelope")
        .as_object_mut()
        .expect("selection envelope object")
        .get_mut("value")
        .expect("selection value")
        .as_object_mut()
        .expect("selection object")
        .insert(''',
        "tests/support/rustc_harness.rs",
    )
    return replace_once(
        source,
        'hasher.update(b"MPK-RUST-DRIVER-REQUEST-0.1");',
        'hasher.update(b"MPK-RUST-DRIVER-REQUEST-1.0");',
        "tests/support/rustc_harness.rs",
    )


def _copy_active_project(repository: Path, destination: Path) -> None:
    source = repository / "rust-tools/rust2vir"
    shutil.copytree(
        source,
        destination,
        ignore=shutil.ignore_patterns("target"),
        copy_function=shutil.copy2,
    )
    for relative, expected in SOURCE_SHA256.items():
        data = (source / relative).read_bytes()
        actual = hashlib.sha256(data).hexdigest()
        if actual != expected:
            raise ValueError(f"{relative}: active source changed ({actual})")


def materialize(
    repository: Path,
    destination: Path,
    *,
    identities: dict[str, str] | None = None,
    driver_vector: bytes | None = None,
    predecessor_only: bool = False,
) -> None:
    _copy_active_project(repository, destination)
    if predecessor_only:
        if driver_vector is None:
            raise ValueError("predecessor rejection needs the successor vector")
        (destination / "testdata/rust-driver-v1.json").write_bytes(driver_vector)
        (destination / "tests/predecessor_identity_rejection.rs").write_text(
            PREDECESSOR_IDENTITY_TEST,
            encoding="utf-8",
        )
        return

    for relative in sorted(SOURCE_SHA256):
        if not relative.startswith("src/"):
            continue
        path = destination / relative
        path.write_text(transform(relative, path.read_text(encoding="utf-8")), encoding="utf-8")
    (destination / "src/successor.rs").write_text(SUCCESSOR_MODULE, encoding="utf-8")

    if identities is not None:
        if driver_vector is None:
            raise ValueError("successor fixture tests need a driver vector")
        positive = (destination / "tests/positive_corpus.rs").read_text(encoding="utf-8")
        negative = (destination / "tests/negative_corpus.rs").read_text(encoding="utf-8")
        (destination / "tests/successor_positive_corpus.rs").write_text(
            transform_positive_test(positive, identities),
            encoding="utf-8",
        )
        (destination / "tests/successor_negative_corpus.rs").write_text(
            transform_negative_test(negative),
            encoding="utf-8",
        )
        harness_path = destination / "tests/support/rustc_harness.rs"
        harness_path.write_text(
            transform_rustc_harness(harness_path.read_text(encoding="utf-8")),
            encoding="utf-8",
        )
        (destination / "tests/successor_protocol_identity.rs").write_text(
            PROTOCOL_IDENTITY_TEST,
            encoding="utf-8",
        )
        (destination / "testdata/rust-driver-v1.json").write_bytes(driver_vector)


if __name__ == "__main__":
    raise SystemExit("use scripts/rust_successor_bundles.py")
