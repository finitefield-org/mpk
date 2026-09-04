#!/usr/bin/env python3
"""Build and validate the private CSHARP-03-T01-W09 freeze artifacts."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[3]
FREEZE = ROOT / "develop/migrations/csharp-03/freeze/profile-freeze.json"
VECTORS = ROOT / "develop/migrations/csharp-03/freeze/profile-freeze-vectors.json"
INVENTORY = ROOT / "develop/migrations/csharp-03/artifact-consumer-inventory.json"
FOUNDATION = ROOT / "develop/migrations/csharp-03/foundation/foundation-descriptor.json"
RECURSOR = ROOT / "develop/migrations/csharp-03/probes/recursor-feasibility.json"
CAPACITY = ROOT / "develop/migrations/csharp-03/probes/checker-capacity.json"
LIMIT_SOURCE = ROOT / "csharp-tools/csharp2vir/FrontendLimits.cs"
OWNER = "crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W09"
DOMAIN = "MPK-CSHARP-PRACTICAL-FREEZE-1.0"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def bytes_of(value: object) -> bytes:
    return canonical(value) + b"\n"


def domain_hash(domain: str, value: object) -> str:
    return sha(domain.encode() + b"\0" + canonical(value))


def read_json(path: Path) -> dict:
    return json.loads(path.read_bytes())


def root_field_type(schema_id: str, field: str) -> str:
    if field == "schema":
        return f"literal<{schema_id}>"
    if field.endswith("_sha256"):
        return "sha256_lower_hex"
    if field == "compilation_id":
        return "compilation_id"
    exact_ids = {
        "callable_id": "canonical_source_callable_id",
        "selected_callable_id": "canonical_source_callable_id",
        "source_type_id": "closed_source_type_id",
        "state_type_id": "closed_source_type_id",
        "command_type_id": "closed_source_type_id",
        "context_type_id": "closed_source_type_id",
    }
    if field in exact_ids:
        return exact_ids[field]
    if field.endswith("_id"):
        return "canonical_id"
    common = {
        "semantic_context": "mpk.semantic_context.v2",
        "profile_registry": "semantic_profile_registry_ref_v2",
        "source_language": "enum<csharp,go,java,rust>",
        "semantic_profile": "canonical_id",
        "semantic_parameters": "semantic_parameters_envelope",
        "foundation_descriptor": "foundation_descriptor_ref_v1",
        "selection": "selection_envelope",
        "value": "csharp_practical_parameter_values_v1",
        "source_paths": "sorted_nonempty_unique_array<normalized_source_path>",
        "selected_root_ids": "sorted_nonempty_unique_array<canonical_source_callable_id>",
        "sidecar_paths": "sorted_unique_array<normalized_sidecar_path>",
        "ordered_member_ids": "ordered_unique_array<canonical_source_member_id>",
        "recursive_default": "closed_canonical_value",
        "default_eligible": "bool",
        "required_member_ids": "ordered_unique_array<canonical_source_member_id>",
        "init_member_ids": "ordered_unique_array<canonical_source_member_id>",
        "construction_invariant": "contract_expression_bool_or_null",
        "invariants": "ordered_array<contract_expression_bool>",
        "structural_equality": "enum<ineligible,field_complete>",
        "structural_order": "enum<ineligible,canonical_field_order>",
        "termination": "enum<partial,total>",
        "requires": "ordered_array<contract_expression_bool>",
        "ensures": "ordered_array<contract_expression_bool>",
        "exceptional_cases": "ordered_array<exceptional_case>",
        "modifies": "exact_empty_array",
        "loops": "ordered_array<loop_contract>",
        "bindings": "sorted_unique_array<mpk.csharp.semantic_binding.v1>",
        "input_fields": "ordered_unique_array<boundary_field>",
        "output_fields": "ordered_unique_array<boundary_field>",
        "canonical_json_profile": "literal<mpk.csharp.canonical_json.v1>",
        "parse_format_profile": "literal<mpk.csharp.parse_format.v1>",
        "evidence_linkage": "boundary_evidence_linkage",
        "raw_input": "raw_input_identity",
        "canonical_value": "closed_canonical_value",
        "source_value": "closed_canonical_value",
        "reparsed_value": "closed_canonical_value",
        "state_invariant": "contract_expression_bool",
        "version_rule": "transition_version_rule",
        "idempotency": "transition_idempotency",
        "accepted_commands": "ordered_array<accepted_command_case>",
        "event_relation": "contract_expression_bool",
        "response_relation": "contract_expression_bool",
        "errors": "ordered_array<transition_error_case>",
        "semantic_request": "mpk.validated_semantic_request.v2",
        "source_snapshot": "source_snapshot_v2",
        "sidecars": "sidecar_set_v2",
        "raw_request_size_bytes": "u32_json_integer",
        "request_linkage": "frontend_diagnostic_request_linkage",
        "artifacts": "mpk.frontend.source_artifacts.v2",
        "status": "literal<rejected>",
        "phase": "diagnostic_phase",
        "diagnostics": "ordered_nonempty_array<diagnostic_entry_v2>",
        "vir": "artifact_ref<mpk.vir.v2>",
        "source_map": "artifact_ref<mpk.source_map.v2>",
        "source_manifest": "artifact_ref<mpk.source_manifest.frontend.v2>",
        "semantic_bindings": "artifact_ref<mpk.csharp.semantic_bindings.v1>",
        "closed_instances": "artifact_ref<mpk.csharp.closed_instances.v1>",
        "boundary_contracts": "sorted_unique_array<artifact_ref<mpk.csharp.boundary.v1>>",
        "transition_contracts": "sorted_unique_array<artifact_ref<mpk.csharp.transition.v1>>",
    }
    try:
        return common[field]
    except KeyError as error:
        raise AssertionError(f"missing field type: {schema_id}.{field}") from error


def nested_record(identity: str, fields: list[tuple[str, str]]) -> dict:
    ordered = [name for name, _ in fields]
    return {
        "id": identity,
        "ordered_fields": ordered,
        "field_types": {name: ty for name, ty in fields},
        "required_fields": ordered,
        "optional_fields": [],
        "unknown_fields": "reject",
        "duplicate_keys": "reject_before_object_construction",
    }


def schema_type_system() -> dict:
    idempotency_fields = [
        ("mode", "literal<complete_snapshot>"),
        ("key_member_id", "canonical_source_member_id"),
        ("history_member_id", "canonical_source_member_id"),
        ("record_type_id", "closed_source_type_id"),
        ("record_key_member_id", "canonical_source_member_id"),
        ("record_command_member_id", "canonical_source_member_id"),
        ("record_context_member_id", "canonical_source_member_id"),
        ("record_response_member_id", "canonical_source_member_id"),
        ("command_equality_callable_id", "canonical_source_callable_id"),
        ("context_equality_callable_id", "canonical_source_callable_id"),
        ("capacity_error_arm", "source_result_arm_id"),
        ("conflict_error_arm", "source_result_arm_id"),
    ]
    records = [
        nested_record("semantic_profile_registry_ref_v2", [
            ("schema", "literal<mpk.semantic_profile.registry.v2>"), ("id", "literal<mpk.semantic_profile.registry.v2>"),
            ("revision", "u64_json_integer"), ("registry_sha256", "sha256_lower_hex")]),
        nested_record("semantic_parameters_envelope", [
            ("schema", "registered_semantic_parameters_schema_id"), ("value", "schema_selected_strict_object")]),
        nested_record("foundation_descriptor_ref_v1", [
            ("schema", "literal<mpk.csharp.foundation_descriptor.v1>"),
            ("id", "literal<mpk.csharp.practical.foundation.v1>"), ("content_sha256", "sha256_lower_hex")]),
        nested_record("exceptional_case", [
            ("exception_type_id", "closed_source_type_id"), ("path_condition", "contract_expression_bool"),
            ("ensures", "ordered_array<contract_expression_bool>")]),
        nested_record("loop_contract", [
            ("loop_id", "canonical_source_identity"), ("invariants", "ordered_nonempty_array<contract_expression_bool>"),
            ("modifies", "ordered_unique_array<local_or_construction_state_id>"),
            ("decreases", "ordered_nonempty_array<well_founded_contract_expression>")]),
        nested_record("boundary_field", [
            ("field_id", "canonical_id"), ("json_name", "utf16_string"), ("type_id", "closed_type_id"),
            ("required", "bool"), ("nullable", "bool"),
            ("missing_rule", "boundary_missing_rule"),
            ("codec_id", "registered_codec_id_or_null")]),
        nested_record("boundary_evidence_linkage", [
            ("raw_input_domain", "literal<MPK-CSHARP-BOUNDARY-INPUT-1.0>"),
            ("canonical_value_domain", "literal<MPK-CSHARP-CANONICAL-VALUE-1.0>"),
            ("canonical_output_domain", "literal<MPK-CSHARP-BOUNDARY-OUTPUT-1.0>"),
            ("reparse_equality", "literal<typed_field_complete>")]),
        nested_record("raw_input_identity", [
            ("provenance_id", "canonical_id"), ("raw_sha256", "sha256_lower_hex"),
            ("size_bytes", "u32_json_integer")]),
        nested_record("transition_version_rule", [
            ("state_member_id", "canonical_source_member_id"),
            ("expected_member_id", "canonical_source_member_id"), ("carrier", "literal<u64>"),
            ("success", "literal<checked_increment_one>"), ("replay", "literal<unchanged>"),
            ("error", "literal<unchanged>"), ("overflow_error_arm", "source_result_arm_id")]),
        nested_record("idempotency_complete_snapshot", idempotency_fields),
        nested_record("csharp_practical_parameter_values_v1", [
            ("check_overflow_default", "literal<true>"),
            ("documentation_mode", "literal<none>"),
            ("language_version", "literal<14.0>"),
            ("nullable_context", "literal<enable>"),
            ("optimization", "literal<release>"),
            ("platform", "literal<x64>"),
            ("pointer_width", "literal<64>"),
            ("preprocessor_symbols", "exact_empty_array"),
            ("source_kind", "literal<regular>"),
            ("target_framework", "literal<net10.0>"),
            ("target_id", "literal<linux-x64>"),
            ("unsafe", "literal<false>")]),
        nested_record("accepted_command_case", [
            ("arm", "source_command_arm_id"), ("condition", "contract_expression_bool"),
            ("postconditions", "ordered_nonempty_array<contract_expression_bool>"),
            ("errors", "ordered_array<transition_error_case>")]),
        nested_record("transition_error_case", [
            ("arm", "source_result_arm_id"), ("condition", "contract_expression_bool")]),
        nested_record("source_snapshot_v2", [
            ("entries", "sorted_unique_array<source_snapshot_entry>"),
            ("snapshot_sha256", "sha256_lower_hex")]),
        nested_record("source_snapshot_entry", [
            ("path", "normalized_relative_path"), ("raw_sha256", "sha256_lower_hex"),
            ("size_bytes", "u32_json_integer")]),
        nested_record("sidecar_set_v2", [
            ("entries", "sorted_unique_array<sidecar_ref>"), ("set_sha256", "sha256_lower_hex")]),
        nested_record("sidecar_ref", [
            ("schema", "registered_sidecar_schema_id"), ("path", "normalized_relative_path"),
            ("raw_sha256", "sha256_lower_hex")]),
        nested_record("artifact_ref", [
            ("schema", "registered_artifact_schema_id"), ("sha256", "sha256_lower_hex"),
            ("canonical_bytes", "u64_json_integer")]),
        nested_record("diagnostic_entry_v2", [
            ("code", "one_of_frozen_diagnostic_families"),
            ("message", "sanitized_public_message_literal"),
            ("location", "source_location_or_null")]),
        nested_record("source_location", [
            ("source_file_ordinal", "u16_json_integer"), ("start_byte", "u32_json_integer"),
            ("end_byte", "u32_json_integer")]),
    ]
    record_producers = {
        "accepted_command_case": "CSHARP-03-T05-W04",
        "artifact_ref": "CSHARP-03-T02-W04",
        "boundary_evidence_linkage": "CSHARP-03-T05-W01",
        "boundary_field": "CSHARP-03-T05-W01",
        "csharp_practical_parameter_values_v1": "CSHARP-03-T02-W01",
        "diagnostic_entry_v2": "CSHARP-03-T02-W08",
        "exceptional_case": "CSHARP-03-T06-W01",
        "foundation_descriptor_ref_v1": "CSHARP-03-T02-W01",
        "idempotency_complete_snapshot": "CSHARP-03-T05-W04",
        "loop_contract": "CSHARP-03-T06-W01",
        "raw_input_identity": "CSHARP-03-T05-W02",
        "semantic_parameters_envelope": "CSHARP-03-T02-W01",
        "semantic_profile_registry_ref_v2": "CSHARP-03-T02-W01",
        "sidecar_ref": "CSHARP-03-T02-W07",
        "sidecar_set_v2": "CSHARP-03-T02-W07",
        "source_location": "CSHARP-03-T02-W04",
        "source_snapshot_entry": "CSHARP-03-T02-W07",
        "source_snapshot_v2": "CSHARP-03-T02-W07",
        "transition_error_case": "CSHARP-03-T05-W04",
        "transition_version_rule": "CSHARP-03-T05-W04",
    }
    for record in records:
        record["producer"] = record_producers[record["id"]]
    records.sort(key=lambda row: row["id"])
    unions = [
        {
            "id": "boundary_missing_rule",
            "tag_field": "mode",
            "variants": [
                nested_record("reject", [("mode", "literal<reject>")]),
                nested_record("expose_missing", [("mode", "literal<expose_missing>")]),
                nested_record("use_frozen_typed_default", [
                    ("mode", "literal<use_frozen_typed_default>"),
                    ("value", "closed_canonical_value"),
                ]),
            ],
            "unknown_tags": "reject",
            "unknown_fields": "reject",
            "duplicate_keys": "reject_before_object_construction",
            "producer": "CSHARP-03-T05-W01",
        },
        {
            "id": "transition_idempotency",
            "tag_field": "mode",
            "variants": [
                nested_record("disabled", [("mode", "literal<disabled>")]),
                nested_record("complete_snapshot", idempotency_fields),
            ],
            "unknown_tags": "reject",
            "unknown_fields": "reject",
            "duplicate_keys": "reject_before_object_construction",
            "producer": "CSHARP-03-T05-W04",
        },
        {
            "id": "frontend_diagnostic_request_linkage",
            "tag_field": "state",
            "variants": [
                nested_record("unvalidated", [("state", "literal<unvalidated>")]),
                nested_record("validated", [
                    ("state", "literal<validated>"),
                    ("request_sha256", "sha256_lower_hex"),
                    ("semantic_context", "mpk.semantic_context.v2"),
                ]),
            ],
            "unknown_tags": "reject",
            "unknown_fields": "reject",
            "duplicate_keys": "reject_before_object_construction",
            "producer": "CSHARP-03-T02-W08",
        },
    ]
    unions.sort(key=lambda row: row["id"])
    for union in unions:
        for variant in union["variants"]:
            variant["tag"] = variant.pop("id")
            variant.pop("producer", None)
    return {
        "atoms": {
            "bool": "JSON true or false",
            "canonical_id": "nonempty printable ASCII, maximum 1024 UTF-8 bytes, grammar fixed by owning schema",
            "canonical_semantic_instance_id": "W08 domain-separated closed semantic-instance identity",
            "canonical_source_callable_id": "section 6.4 domain-separated callable identity",
            "canonical_source_member_id": "section 6.4 domain-separated member identity",
            "sha256_lower_hex": "exactly 64 lowercase hexadecimal ASCII characters",
            "canonical_source_identity": "section 6.4 domain-separated source identity",
            "closed_source_type_id": "canonical source type identity admitted by the selected closed compilation",
            "closed_type_id": "canonical monomorphic source or registered semantic-instance type identity",
            "closed_canonical_value": "value closed by its admitted type and canonical JSON codec",
            "compilation_id": "1..64 ASCII bytes matching [a-z][a-z0-9]*([._-][a-z0-9]+)*",
            "contract_binding_id": "contract-local or subject binding identity declared in the enclosing contract",
            "contract_expression": "one exact mpk.csharp.contract_expression.v1 variant",
            "contract_expression_bool": "contract_expression whose checked type_id is Bool",
            "contract_expression_bool_or_null": "JSON null or contract_expression_bool",
            "contract_expression_or_null": "JSON null or contract_expression",
            "diagnostic_phase": "canonical JSON integer equal to one phase ID in diagnostics.phase_precedence",
            "contract_local_binding_id": "contract-local binding identity introduced exactly once by let or a bounded quantifier",
            "exact_empty_array": "the exact JSON array []",
            "local_or_construction_state_id": "declared local or unique construction-state identity in the selected callable",
            "normalized_relative_path": "portable normalized repository-relative path with no empty, dot, dot-dot, absolute, or backslash component",
            "normalized_sidecar_path": "normalized_relative_path under the selected verification-overlay sidecar root ending .json",
            "normalized_source_path": "normalized_relative_path under src/ ending .cs",
            "one_of_frozen_diagnostic_families": "one exact code in diagnostics.families",
            "registered_codec_id_or_null": "JSON null or one codec ID in the frozen parse/format profile",
            "registered_codec_id": "one codec ID in the frozen parse/format profile",
            "registered_codec_mode_id": "one mode admitted by the selected registered codec",
            "registered_operation_id": "one operation ID admitted by the frozen closed operation profile",
            "registered_semantic_binding_id": "one binding ID in the context-bound validated semantic-binding set",
            "registered_sum_arm_id": "one arm ID declared by the checked closed type_id",
            "registered_semantic_parameters_schema_id": "one exact parameters schema selected by the validated semantic-profile entry",
            "registered_sidecar_schema_id": "one exact strict sidecar schema admitted by the selected semantic context",
            "registered_artifact_schema_id": "the exact schema identity required by the enclosing artifact_ref type argument",
            "sanitized_public_message_literal": "the exact diagnostics.public_message string",
            "schema_selected_strict_object": "strict object selected by semantic_parameters_envelope.schema; for the practical profile this is csharp_practical_parameter_values_v1",
            "selection_envelope": "strict selection object whose schema, shape, and hash domain are selected by the validated semantic-profile entry; for the practical profile this is mpk.selection.csharp_members.v1",
            "source_command_arm_id": "one declared arm of the bound application-owned command sum",
            "source_result_arm_id": "one declared arm of the bound application-owned result sum",
            "source_location_or_null": "JSON null or source_location",
            "u16_json_integer": "canonical JSON integer in 0..65535",
            "u32_json_integer": "canonical JSON integer in 0..4294967295",
            "u64_json_integer": "canonical JSON integer in 0..18446744073709551615",
            "utf16_string": "JSON string decoded to an exact finite UTF-16 code-unit sequence under canonical_json.strings",
            "well_founded_contract_expression": "contract_expression whose checked type and order prove a strict well-founded decrease",
        },
        "type_constructors": {
            "artifact_ref<T>": "artifact_ref whose schema field is the exact successor identity T",
            "enum<a,...>": "one exact listed ASCII token",
            "literal<x>": "the exact JSON scalar or ASCII identity x and no other value",
            "ordered_array<T>": "possibly empty array of T preserving declared order",
            "ordered_nonempty_array<T>": "nonempty array of T preserving declared order",
            "ordered_unique_array<T>": "possibly empty array of pairwise-distinct T preserving declared order",
            "sorted_unique_array<T>": "possibly empty array of pairwise-distinct T in ascending canonical-byte order",
            "sorted_nonempty_unique_array<T>": "nonempty array of pairwise-distinct T in ascending canonical-byte order",
        },
        "nested_records": records,
        "tagged_unions": unions,
        "collection_rules": "arrays preserve declared order unless the type says sorted_unique; duplicates reject before hashing",
    }


def schema(identity: str, fields: list[str], hash_field: str | None, hash_domain: str | None,
           producer: str, consumers: list[str]) -> dict:
    return {
        "id": identity,
        "version": int(identity.rsplit(".v", 1)[1]),
        "root": "object",
        "ordered_fields": fields,
        "field_types": {field: root_field_type(identity, field) for field in fields},
        "required_fields": fields,
        "optional_fields": [],
        "hash_field": hash_field,
        "hash_domain": hash_domain,
        "hash_preimage_fields": ([field for field in fields if field != hash_field]
                                 if hash_domain is not None else None),
        "unknown_fields": "reject",
        "duplicate_keys": "reject_before_object_construction",
        "later_versions": "reject",
        "producer": producer,
        "consumers": consumers,
    }


def identity_families(inventory: dict) -> list[dict]:
    profiles = ["csharp_scalar", "csharp_practical", "go_fixed", "java_scalar", "rust_checked"]
    contract_categories = ["ai", "evidence", "frontend", "manifest", "policy", "release", "source_map", "vc", "vir"]
    compiled = [f"mpk.profile.{category}.{profile}.v1" for category in contract_categories for profile in profiles]
    successors = {
        "semantic_registry": (
            ["mpk.semantic_profile.registry.v2", "mpk.semantic_profile.entry.v2",
             "mpk.semantic_profile.registry.limits.v2", "mpk.csharp.practical.v1"],
            ["MPK-SEMANTIC-PROFILE-REGISTRY-2.0", "MPK-SEMANTIC-PROFILE-ENTRY-2.0"],
        ),
        "semantic_context": (
            ["mpk.semantic_context.v2", "mpk.validated_semantic_request.v2"],
            ["MPK-SEMANTIC-CONTEXT-2.0", "MPK-VALIDATED-SEMANTIC-REQUEST-2.0"],
        ),
        "semantic_parameters": (
            ["mpk.semantic_parameters.csharp_practical.v1"],
            ["MPK-CSHARP-PRACTICAL-PARAMETERS-1.0"],
        ),
        "selection": (
            ["mpk.selection.csharp_members.v1"],
            ["MPK-CSHARP-SELECTION-1.0"],
        ),
        "profile_contract": (
            ["mpk.csharp.contract.v1", "mpk.csharp.type_contract.v1",
             "mpk.csharp.semantic_bindings.v1", "mpk.csharp.contract_expression.v1",
             "mpk.csharp.canonical_json.v1", "mpk.csharp.parse_format.v1",
             "mpk.csharp.boundary.v1", "mpk.csharp.boundary_input.v1",
             "mpk.csharp.boundary_output.v1", "mpk.csharp.transition.v1",
             "mpk.csharp.operations.v1", "mpk.csharp.required_checks.v1",
             "mpk.csharp.limits.v1"] + compiled,
            ["MPK-CONTRACT-2.0", "MPK-CSHARP-METHOD-CONTRACT-1.0",
             "MPK-CSHARP-TYPE-CONTRACT-1.0", "MPK-CSHARP-SEMANTIC-BINDING-SET-1.0",
             "MPK-CSHARP-CANONICAL-VALUE-1.0",
             "MPK-CSHARP-BOUNDARY-CONTRACT-1.0", "MPK-CSHARP-BOUNDARY-INPUT-1.0",
             "MPK-CSHARP-BOUNDARY-OUTPUT-1.0", "MPK-CSHARP-TRANSITION-CONTRACT-1.0",
             "MPK-CSHARP-OPERATIONS-1.0", "MPK-CSHARP-REQUIRED-CHECKS-1.0",
             "MPK-CSHARP-LIMITS-1.0", "MPK-COMPILED-PROFILE-CONTRACT-1.0"],
        ),
        "source_artifact": (
            ["mpk.frontend.source_artifacts.v2"],
            ["MPK-FRONTEND-SOURCE-ARTIFACTS-2.0"],
        ),
        "foundation": (
            ["mpk.csharp.practical.foundation.v1", "mpk.csharp.foundation_descriptor.v1",
             "mpk.csharp.foundation_definitions.v1", "mpk.csharp.foundation_expansion.v1",
             "mpk.csharp.semantic_binding.v1", "mpk.csharp.closed_instances.v1"],
            ["MPK-CSHARP-PRACTICAL-FOUNDATION-1.0", "MPK-CSHARP-FOUNDATION-MEMBER-1.0",
             "MPK-CSHARP-DECLARATION-1.0", "MPK-CSHARP-DECLARATION-PROVENANCE-1.0",
             "MPK-CSHARP-SEMANTIC-BINDING-1.0", "MPK-CSHARP-SEMANTIC-INSTANCE-1.0",
             "MPK-CSHARP-CLOSED-INSTANCES-1.0"],
        ),
        "vir": (["mpk.vir.v2"], ["MPK-VIR-2.0"]),
        "frontend_protocol": (
            ["mpk.frontend.cli.v2", "mpk.frontend.request.v2", "mpk.frontend.success.v2",
             "mpk.frontend.diagnostic.v2"],
            ["MPK-FRONTEND-REQUEST-2.0", "MPK-FRONTEND-SUCCESS-2.0",
             "MPK-FRONTEND-DIAGNOSTIC-2.0"],
        ),
        "source_map": (["mpk.source_map.v2"], ["MPK-SOURCE-MAP-2.0"]),
        "source_manifest": (
            ["mpk.source_manifest.frontend.v2", "mpk.source_manifest.certificate.v2"],
            ["MPK-SOURCE-MANIFEST-2.0"],
        ),
        "vc_skeleton": (["mpk.vc.v3", "mpk.vc.cert_skeleton.v3"], ["MPK-VC-3.0"]),
        "release": (
            ["mpk.release.bundle_registry.v2", "mpk.release.registry.v2",
             "mpk.release.frontend_bundle.v2", "mpk.release.toolchain_bundle.v2",
             "mpk.release.bundle_candidate.v2", "mpk.release.bundle_inventory.v1",
             "mpk.release.evidence.v2", "mpk.release.receipt.v2"],
            ["MPK-BUNDLE-REGISTRY-2.0", "MPK-BUNDLE-CONTENT-1.0",
             "MPK-RELEASE-REGISTRY-2.0", "MPK-RELEASE-RECEIPT-2.0"],
        ),
        "policy_evidence": (
            ["mpk.policy.scan.v3", "mpk.policy.evidence.v3", "mpk.policy.reproduction.v3",
             "mpk.policy.receipt.v3"],
            ["MPK-POLICY-EVIDENCE-3.0", "MPK-POLICY-REPRODUCTION-3.0",
             "MPK-POLICY-RECEIPT-3.0"],
        ),
        "program_assembly": (
            ["mpk.program_certificate.ordinary_context.v2"],
            ["MPK-PROGRAM-ASSEMBLY-2.0"],
        ),
        "ai": (["mpk.ai.explain.request.v3", "mpk.ai.explanation.v3"], []),
        "api": (
            ["mpk.ai.api.v3", "mpk.ai.api.request.v3", "mpk.ai.api.session.v3",
             "mpk.ai.api.response.v3"],
            [],
        ),
    }
    retained_domain_decisions = {
        "source_manifest": [
            {"id": "MPK-INPUT-SET-0.1", "decision": "retain_exact_preimage"},
            {"id": "MPK-RUST-SOURCE-INVENTORY-0.1", "decision": "retain_exact_preimage"},
        ],
        "program_assembly": [
            {"id": value, "decision": "retain_certificate_v0_preimage"}
            for value in ["MPK-CERT-0.1", "MPK-MODULE-EXPORT-0.1", "MPK-MODULE-CERT-0.1",
                          "MPK-AXIOM-REPORT-0.1", "MPK-LEVEL-0.1", "MPK-TERM-0.1",
                          "MPK-PROOF-NODE-0.1", "MPK-DECL-0.1", "MPK-THEORY-CERT-0.1"]
        ],
    }
    result = []
    for old in inventory["identity_families"]:
        identities, domains = successors[old["id"]]
        result.append({
            "family": old["id"],
            "successor_identities": sorted(identities),
            "successor_hash_domains": sorted(domains),
            "retained_identities": old["current_identities"],
            "retained_hash_domains": retained_domain_decisions.get(old["id"], []),
            "implementation_owners": old["implementation_owners"],
            "migration_set": inventory["atomic_migration_set"]["id"],
        })
    return result


def schemas() -> list[dict]:
    context = ["schema", "profile_registry", "profile_entry_sha256", "source_language",
               "semantic_profile", "semantic_parameters", "foundation_descriptor"]
    result = [
        schema("mpk.semantic_context.v2", context, None, "MPK-SEMANTIC-CONTEXT-2.0", "CSHARP-03-T02-W01",
               ["CSHARP-03-T02-W04", "CSHARP-03-T02-W09"]),
        schema("mpk.semantic_parameters.csharp_practical.v1", ["schema", "value"], None,
               "MPK-CSHARP-PRACTICAL-PARAMETERS-1.0", "CSHARP-03-T02-W01",
               ["CSHARP-03-T02-W08", "CSHARP-03-T02-W09"]),
        schema("mpk.selection.csharp_members.v1", ["schema", "compilation_id", "source_paths",
               "selected_root_ids", "sidecar_paths", "selection_sha256"], "selection_sha256",
               "MPK-CSHARP-SELECTION-1.0", "CSHARP-03-T02-W01",
               ["CSHARP-03-T02-W04", "CSHARP-03-T02-W08", "CSHARP-03-T02-W09"]),
        schema("mpk.validated_semantic_request.v2", ["schema", "semantic_context", "selection",
               "request_sha256"],
               "request_sha256", "MPK-VALIDATED-SEMANTIC-REQUEST-2.0", "CSHARP-03-T02-W01",
               ["CSHARP-03-T02-W04", "CSHARP-03-T02-W08", "CSHARP-03-T02-W09"]),
        schema("mpk.csharp.type_contract.v1", ["schema", "semantic_context", "compilation_id",
               "source_type_id", "source_content_sha256", "ordered_member_ids", "recursive_default",
               "default_eligible", "required_member_ids", "init_member_ids", "construction_invariant",
               "invariants", "structural_equality", "structural_order", "contract_sha256"],
               "contract_sha256", "MPK-CSHARP-TYPE-CONTRACT-1.0", "CSHARP-03-T06-W01",
               ["CSHARP-03-T03-W03", "CSHARP-03-T06-W02"]),
        schema("mpk.csharp.contract.v1", ["schema", "semantic_context", "compilation_id", "callable_id",
               "source_content_sha256", "termination", "requires", "ensures", "exceptional_cases",
               "modifies", "loops", "contract_sha256"], "contract_sha256",
               "MPK-CSHARP-METHOD-CONTRACT-1.0", "CSHARP-03-T06-W01",
               ["CSHARP-03-T04-W01", "CSHARP-03-T06-W02", "CSHARP-03-T06-W05"]),
        schema("mpk.csharp.semantic_bindings.v1", ["schema", "semantic_context", "compilation_id",
               "bindings", "binding_set_sha256"], "binding_set_sha256",
               "MPK-CSHARP-SEMANTIC-BINDING-SET-1.0", "CSHARP-03-T02-W04",
               ["CSHARP-03-T03-W14", "CSHARP-03-T06-W06"]),
        schema("mpk.csharp.boundary.v1", ["schema", "semantic_context", "compilation_id", "boundary_id",
               "selected_callable_id", "input_fields", "output_fields", "canonical_json_profile",
               "parse_format_profile", "evidence_linkage", "contract_sha256"], "contract_sha256",
               "MPK-CSHARP-BOUNDARY-CONTRACT-1.0", "CSHARP-03-T05-W01",
               ["CSHARP-03-T05-W02", "CSHARP-03-T05-W03", "CSHARP-03-T06-W07"]),
        schema("mpk.csharp.boundary_input.v1", ["schema", "semantic_context",
               "boundary_contract_sha256", "raw_input", "canonical_document_utf8_sha256",
               "canonical_value", "canonical_value_sha256", "capture_sha256"], "capture_sha256",
               "MPK-CSHARP-BOUNDARY-INPUT-1.0", "CSHARP-03-T05-W02",
               ["CSHARP-03-T06-W07", "CSHARP-03-T06-W10"]),
        schema("mpk.csharp.boundary_output.v1", ["schema", "semantic_context",
               "boundary_contract_sha256", "source_value", "source_value_sha256",
               "canonical_document_utf8_sha256", "reparsed_value", "reparsed_value_sha256",
               "capture_sha256"], "capture_sha256", "MPK-CSHARP-BOUNDARY-OUTPUT-1.0",
               "CSHARP-03-T05-W03", ["CSHARP-03-T06-W07", "CSHARP-03-T06-W10"]),
        schema("mpk.csharp.transition.v1", ["schema", "semantic_context", "compilation_id",
               "transition_id", "selected_callable_id", "state_type_id", "command_type_id",
               "context_type_id", "apply_result_binding_id", "transition_binding_id",
               "domain_error_binding_id", "state_invariant", "version_rule", "idempotency",
               "accepted_commands", "event_relation", "response_relation", "errors",
               "contract_sha256"], "contract_sha256", "MPK-CSHARP-TRANSITION-CONTRACT-1.0",
               "CSHARP-03-T05-W04", ["CSHARP-03-T05-W05", "CSHARP-03-T06-W08"]),
        schema("mpk.frontend.request.v2", ["schema", "semantic_request", "source_snapshot",
               "sidecars", "request_sha256"], "request_sha256", "MPK-FRONTEND-REQUEST-2.0",
               "CSHARP-03-T02-W07", ["CSHARP-03-T02-W08", "CSHARP-03-T02-W09"]),
        schema("mpk.frontend.success.v2", ["schema", "request_sha256", "semantic_context", "artifacts",
               "success_sha256"], "success_sha256", "MPK-FRONTEND-SUCCESS-2.0",
               "CSHARP-03-T02-W08", ["CSHARP-03-T02-W09"]),
        schema("mpk.frontend.diagnostic.v2", ["schema", "raw_request_sha256",
               "raw_request_size_bytes", "request_linkage", "status", "phase", "diagnostics",
               "diagnostic_sha256"], "diagnostic_sha256", "MPK-FRONTEND-DIAGNOSTIC-2.0",
               "CSHARP-03-T02-W08", ["CSHARP-03-T02-W09"]),
        schema("mpk.frontend.source_artifacts.v2", ["schema", "semantic_context", "selection_sha256",
               "vir", "source_map", "source_manifest", "semantic_bindings", "closed_instances",
               "foundation_descriptor", "boundary_contracts", "transition_contracts",
               "artifacts_sha256"], "artifacts_sha256", "MPK-FRONTEND-SOURCE-ARTIFACTS-2.0",
               "CSHARP-03-T02-W08", ["CSHARP-03-T02-W04", "CSHARP-03-T02-W09"]),
    ]
    return sorted(result, key=lambda row: row["id"])


def expression_union() -> dict:
    expression = "contract_expression"
    fields = {
        "literal": [("value", "closed_canonical_value")],
        "variable": [("binding_id", "contract_binding_id")],
        "result": [],
        "old": [("expression", expression)],
        "field": [("receiver", expression), ("member_id", "canonical_source_member_id")],
        "property": [("receiver", expression), ("member_id", "canonical_source_member_id")],
        "unary": [("operation_id", "registered_operation_id"), ("operand", expression)],
        "binary": [("operation_id", "registered_operation_id"), ("left", expression),
                   ("right", expression)],
        "conditional": [("condition", "contract_expression_bool"), ("when_true", expression),
                        ("when_false", expression)],
        "let": [("binding_id", "contract_local_binding_id"), ("value", expression),
                ("body", expression)],
        "construct": [("constructor_id", "canonical_source_callable_id"),
                      ("arguments", "ordered_array<contract_expression>")],
        "sequence_length": [("sequence", expression)],
        "sequence_index": [("sequence", expression), ("index", expression)],
        "map_lookup": [("map", expression), ("key", expression)],
        "map_contains": [("map", expression), ("key", expression)],
        "set_contains": [("set", expression), ("element", expression)],
        "tagged_make": [("semantic_instance_id", "canonical_semantic_instance_id"),
                        ("arm", "registered_sum_arm_id"), ("payload", "contract_expression_or_null")],
        "tagged_is": [("value", expression), ("arm", "registered_sum_arm_id")],
        "tagged_payload": [("value", expression), ("arm", "registered_sum_arm_id")],
        "source_project": [("binding_id", "registered_semantic_binding_id"),
                           ("source_value", expression)],
        "source_reconstruct": [("binding_id", "registered_semantic_binding_id"),
                               ("semantic_value", expression)],
        "structural_equal": [("left", expression), ("right", expression)],
        "structural_compare": [("left", expression), ("right", expression)],
        "codec_parse": [("codec_id", "registered_codec_id"), ("text", expression)],
        "codec_format": [("codec_id", "registered_codec_id"), ("value", expression),
                         ("mode", "registered_codec_mode_id")],
        "parse_error_kind": [("value", expression)],
        "exception_is": [("value", expression), ("exception_type_id", "closed_source_type_id")],
        "exception_payload": [("value", expression), ("member_id", "canonical_source_member_id")],
        "transition_state": [("value", expression)],
        "transition_events": [("value", expression)],
        "transition_response": [("value", expression)],
        "bounded_forall": [("binding_id", "contract_local_binding_id"), ("lower", expression),
                           ("upper", expression), ("body", "contract_expression_bool")],
        "bounded_exists": [("binding_id", "contract_local_binding_id"), ("lower", expression),
                           ("upper", expression), ("body", "contract_expression_bool")],
    }
    variants = []
    for tag in sorted(fields):
        members = [("tag", f"literal<{tag}>"), ("type_id", "closed_type_id")] + fields[tag]
        ordered = [name for name, _ in members]
        variants.append({
            "tag": tag,
            "ordered_fields": ordered,
            "field_types": {name: ty for name, ty in members},
            "required_fields": ordered,
            "optional_fields": [],
        })
    return {
        "schema": "mpk.csharp.contract_expression.v1",
        "common_ordered_fields": ["tag", "type_id"],
        "variants": variants,
        "unknown_tags": "reject",
        "unknown_fields": "reject",
        "duplicate_keys": "reject_before_object_construction",
        "producer": "CSHARP-03-T06-W01",
        "calls_source_methods": False,
        "type_rule": "every child and result type is explicit and checked against the closed operation signature",
        "closure_rule": "no free binding other than the contract subject, declared let, or bounded quantifier binding",
    }


def practical_limits(capacity: dict) -> list[dict]:
    measured = capacity["probe"]["limits"]
    values = [
        ("source_data_exception_types", 128, "declaration", "pre_invocation_structural", "retain each reachable source data or exception type", "CSHARP-03-T03-W01"),
        ("fields_properties_per_type", 32, "member", "pre_invocation_structural", "retain each declared instance field or property", "CSHARP-03-T03-W03"),
        ("constructors_per_type", 8, "constructor", "pre_invocation_structural", "retain each source constructor", "CSHARP-03-T03-W04"),
        ("structural_type_nesting", 16, "edge", "pre_invocation_structural", "descend one source value-type member edge", "CSHARP-03-T03-W03"),
        ("array_elements", 4096, "element", "runtime_value_predicate_vc", "publish one run-time array element", "CSHARP-03-T03-W07"),
        ("sequence_construction_capacity", 16384, "slot", "runtime_value_predicate_vc", "reserve or fill one linear construction slot", "CSHARP-03-T03-W08"),
        ("construction_states_per_method", 32, "state", "pre_invocation_structural", "create one construction-state SSA identity", "CSHARP-03-T03-W08"),
        ("simultaneously_live_construction_states", 8, "state", "pre_invocation_structural", "add one state to the live ownership set", "CSHARP-03-T03-W08"),
        ("ordered_map_set_entries", 4096, "entry", "runtime_value_predicate_vc", "publish one canonical map or set entry", "CSHARP-03-T03-W09"),
        ("total_collection_cells", 65536, "cell", "runtime_value_predicate_vc", "account one recursively represented collection cell", "CSHARP-03-T03-W09"),
        ("string_utf16_units", 16384, "utf16_unit", "runtime_value_predicate_vc", "publish one UTF-16 code unit", "CSHARP-03-T03-W10"),
        ("outcome_presence_nesting", 16, "edge", "pre_invocation_structural", "descend one option, lookup, result, validation, or boundary-presence type edge", "CSHARP-03-T03-W12"),
        ("validation_errors", 256, "error", "runtime_value_predicate_vc", "append one validation error in source order", "CSHARP-03-T03-W12"),
        ("semantic_bindings", 128, "binding", "pre_invocation_structural", "retain one reachable semantic binding", "CSHARP-03-T03-W14"),
        ("projection_obligations_per_binding", 64, "obligation", "pre_invocation_structural", "emit one field, arm, invariant, or operation projection obligation", "CSHARP-03-T06-W06"),
        ("closed_semantic_instances", 256, "instance", "pre_invocation_structural", "insert one deduplicated closed semantic instance", "CSHARP-03-T02-W02"),
        ("closed_instance_nesting", 16, "edge", "pre_invocation_structural", "descend one closed-instance argument or dependency edge", "CSHARP-03-T02-W02"),
        ("specialized_declarations", 1024, "declaration", "pre_invocation_structural", "emit one specialized declaration", "CSHARP-03-T02-W02"),
        ("specialized_operations", 4096, "operation", "pre_invocation_structural", "emit one specialized operation", "CSHARP-03-T02-W02"),
        ("boundary_fields", 256, "field", "pre_invocation_structural", "retain one ordered input or output boundary field", "CSHARP-03-T05-W01"),
        ("boundary_nesting", 32, "edge", "pre_invocation_structural", "descend one canonical boundary value edge", "CSHARP-03-T05-W02"),
        ("boundary_canonical_bytes", 1048576, "byte", "pre_invocation_structural", "append one canonical UTF-8 output byte before retention", "CSHARP-03-T05-W02"),
        ("transition_events", 4096, "event", "runtime_value_predicate_vc", "append one emitted event in source order", "CSHARP-03-T05-W04"),
        ("loops_per_method", 32, "loop", "pre_invocation_structural", "retain one loop syntax node", "CSHARP-03-T04-W01"),
        ("loop_nesting", 8, "edge", "pre_invocation_structural", "enter one lexically nested loop", "CSHARP-03-T04-W01"),
        ("invariant_decreases_per_loop", 64, "clause", "pre_invocation_structural", "retain one invariant or decreases clause for one loop", "CSHARP-03-T04-W01"),
        ("switch_arms_per_method", 256, "arm", "pre_invocation_structural", "retain one switch statement section or expression arm", "CSHARP-03-T04-W03"),
        ("pattern_nesting", 16, "edge", "pre_invocation_structural", "descend one admitted pattern edge", "CSHARP-03-T04-W03"),
        ("catch_finally_regions_per_method", 32, "region", "pre_invocation_structural", "retain one catch or finally region", "CSHARP-03-T04-W05"),
        ("source_exception_types", 32, "type", "pre_invocation_structural", "retain one reachable source exception type", "CSHARP-03-T04-W04"),
        ("bounded_quantifier_nesting", 4, "edge", "pre_invocation_structural", "enter one bounded quantifier expression", "CSHARP-03-T06-W01"),
        ("ordinary_term_nodes", measured["ordinary_term_nodes"], "term", "pre_invocation_structural", "retain one reachable ordinary Certificate v0 term node", "CSHARP-03-T06-W09"),
        ("generated_declarations", measured["generated_declarations"], "declaration", "pre_invocation_structural", "retain one successor-generated declaration excluding the pinned standard prelude", "CSHARP-03-T06-W09"),
        ("binder_depth", measured["binder_depth"], "binder", "pre_invocation_structural", "enter one Pi, Lam, or Let binder on the active term path", "CSHARP-03-T06-W09"),
        ("static_transformers", measured["static_transformers"], "transformer", "pre_invocation_structural", "add one concrete state transformer before balanced composition", "CSHARP-03-T06-W09"),
    ]
    rows = []
    for identity, maximum, unit, classification, site, owner in values:
        rows.append({
            "id": identity,
            "inclusive_maximum": maximum,
            "unit": unit,
            "classification": classification,
            "increment_site": site,
            "increment_rule": "checked_add(counter,1) exactly once at the increment site",
            "comparison_rule": ("prove value_measure <= inclusive_maximum before verified acceptance"
                                if classification == "runtime_value_predicate_vc"
                                else "reject before allocation or retention when candidate > inclusive_maximum"),
            "overflow_rule": "treat checked counter overflow as limit_exceeded",
            "diagnostic": "CSHARP_PRACTICAL_LIMIT",
            "implementation_owner": owner,
        })
    return sorted(rows, key=lambda row: row["id"])


def retained_limits() -> list[dict]:
    definitions = [
        ("source_files", "SourceFilesMaximum", 256, "rejected"),
        ("source_file_bytes", "SourceFileBytesMaximum", 1048576, "rejected"),
        ("source_total_bytes", "SourceTotalBytesMaximum", 16777216, "rejected"),
        ("contract_files", "ContractFilesMaximum", 128, "rejected"),
        ("contract_file_bytes", "ContractFileBytesMaximum", 1048576, "rejected"),
        ("contract_total_bytes", "ContractTotalBytesMaximum", 8388608, "rejected"),
        ("snapshot_entries", "SnapshotEntriesMaximum", 512, "rejected"),
        ("snapshot_total_bytes", "SnapshotTotalBytesMaximum", 33554432, "rejected"),
        ("normalized_path_bytes", "NormalizedPathBytesMaximum", 1024, "rejected"),
        ("canonical_method_id_bytes", "CanonicalMethodIdBytesMaximum", 1024, "rejected"),
        ("selected_methods", "SelectedMethodsMaximum", 32, "rejected"),
        ("method_closure", "MethodClosureMaximum", 128, "rejected"),
        ("syntax_nodes", "SyntaxNodesMaximum", 250000, "rejected"),
        ("operations_per_method", "OperationsPerMethodMaximum", 100000, "rejected"),
        ("operations_per_closure", "OperationsPerClosureMaximum", 250000, "rejected"),
        ("cfg_blocks_per_method", "CfgBlocksPerMethodMaximum", 1024, "rejected"),
        ("cfg_blocks_per_closure", "CfgBlocksPerClosureMaximum", 8192, "rejected"),
        ("contract_clauses", "ContractClausesMaximum", 64, "rejected"),
        ("contract_nodes_per_method", "ContractNodesPerMethodMaximum", 1024, "rejected"),
        ("contract_nodes_per_closure", "ContractNodesPerClosureMaximum", 8192, "rejected"),
        ("contract_depth", "ContractDepthMaximum", 32, "rejected"),
        ("normalized_issues", "NormalizedIssuesMaximum", 1024, "diagnostic_budget"),
        ("diagnostic_message_bytes_each", "DiagnosticMessageBytesEachMaximum", 4096, "diagnostic_budget"),
        ("diagnostic_message_bytes_total", "DiagnosticMessageBytesTotalMaximum", 2097152, "diagnostic_budget"),
        ("frontend_argument_bytes", "FrontendArgumentBytesMaximum", 131072, "rejected"),
        ("private_runtime_stdout", "PrivateRuntimeStdoutMaximum", 268435456, "output_limit"),
        ("private_runtime_stderr", "PrivateRuntimeStderrMaximum", 2097152, "output_limit"),
        ("vir_canonical_bytes", "VirCanonicalBytesMaximum", 201326592, "rejected"),
        ("source_map_canonical_bytes", "SourceMapCanonicalBytesMaximum", 33554432, "rejected"),
        ("source_manifest_canonical_bytes", "SourceManifestCanonicalBytesMaximum", 4194304, "rejected"),
        ("frontend_stdout", "FrontendStdoutMaximum", 268435456, "output_limit"),
        ("frontend_stderr", "FrontendStderrMaximum", 2097152, "output_limit"),
    ]
    source = LIMIT_SOURCE.read_text()
    rows = []
    for identity, constant, maximum, disposition in definitions:
        match = re.search(rf"internal const (?:int|uint) {constant} = ([0-9_]+);", source)
        if not match or int(match.group(1).replace("_", "")) != maximum:
            raise AssertionError(f"retained limit drift: {identity}")
        rows.append({"id": identity, "inclusive_maximum": maximum, "disposition": disposition,
                     "source_constant": constant, "status": "retained_unchanged"})
    return rows


def diagnostics() -> dict:
    families = [
        "CSHARP_PRACTICAL_PROTOCOL", "CSHARP_PRACTICAL_LIMIT", "CSHARP_PRACTICAL_DECLARATION",
        "CSHARP_PRACTICAL_TYPE", "CSHARP_PRACTICAL_DEPENDENCY", "CSHARP_PRACTICAL_GENERIC",
        "CSHARP_PRACTICAL_SOURCE_BINDING", "CSHARP_PRACTICAL_OBJECT",
        "CSHARP_PRACTICAL_INITIALIZER", "CSHARP_PRACTICAL_OWNERSHIP",
        "CSHARP_PRACTICAL_ARRAY", "CSHARP_PRACTICAL_COLLECTION", "CSHARP_PRACTICAL_ORDER",
        "CSHARP_PRACTICAL_STRING", "CSHARP_PRACTICAL_PARSE_FORMAT", "CSHARP_PRACTICAL_FLOAT",
        "CSHARP_PRACTICAL_DECIMAL", "CSHARP_PRACTICAL_NULLABLE", "CSHARP_PRACTICAL_RESULT",
        "CSHARP_PRACTICAL_BUSINESS_VALUE", "CSHARP_PRACTICAL_LOOP_CONTRACT",
        "CSHARP_PRACTICAL_SWITCH", "CSHARP_PRACTICAL_PATTERN", "CSHARP_PRACTICAL_EXCEPTION",
        "CSHARP_PRACTICAL_BOUNDARY", "CSHARP_PRACTICAL_TRANSITION", "CSHARP_PRACTICAL_EFFECT",
        "CSHARP_PRACTICAL_FOUNDATION", "CSHARP_PRACTICAL_LOWERING",
    ]
    phases = [
        ["CSHARP_PRACTICAL_PROTOCOL", "CSHARP_PRACTICAL_LIMIT"],
        ["CSHARP_PRACTICAL_DEPENDENCY"],
        ["CSHARP_PRACTICAL_DECLARATION", "CSHARP_PRACTICAL_TYPE"],
        ["CSHARP_PRACTICAL_GENERIC"],
        ["CSHARP_PRACTICAL_SOURCE_BINDING", "CSHARP_PRACTICAL_FOUNDATION"],
        ["CSHARP_PRACTICAL_BOUNDARY", "CSHARP_PRACTICAL_TRANSITION"],
        ["CSHARP_PRACTICAL_OBJECT", "CSHARP_PRACTICAL_INITIALIZER", "CSHARP_PRACTICAL_OWNERSHIP",
         "CSHARP_PRACTICAL_ARRAY", "CSHARP_PRACTICAL_COLLECTION", "CSHARP_PRACTICAL_ORDER",
         "CSHARP_PRACTICAL_STRING", "CSHARP_PRACTICAL_PARSE_FORMAT", "CSHARP_PRACTICAL_FLOAT",
         "CSHARP_PRACTICAL_DECIMAL", "CSHARP_PRACTICAL_NULLABLE", "CSHARP_PRACTICAL_RESULT",
         "CSHARP_PRACTICAL_BUSINESS_VALUE"],
        ["CSHARP_PRACTICAL_LOOP_CONTRACT", "CSHARP_PRACTICAL_SWITCH",
         "CSHARP_PRACTICAL_PATTERN", "CSHARP_PRACTICAL_EXCEPTION", "CSHARP_PRACTICAL_EFFECT"],
        ["CSHARP_PRACTICAL_LOWERING"],
    ]
    flattened = [family for phase in phases for family in phase]
    assert set(flattened) == set(families) and len(flattened) == len(set(flattened))
    return {
        "families": families,
        "phase_precedence": [{"phase": index, "families_in_precedence_order": phase}
                             for index, phase in enumerate(phases)],
        "selection": "stop after the earliest failing phase; within it sort by family precedence, source-file ordinal, start byte, end byte, and code",
        "request_linkage": "raw request hash and size are always present; linkage is unvalidated only until strict request/context validation succeeds, and is the complete validated request hash plus semantic context thereafter",
        "phase_rule": "phase is exactly 0..8 and every diagnostic code belongs to that phase; mixed-phase entries reject",
        "location_rule": "location is null or has 0 <= start_byte < end_byte <= the retained raw source size and a source_file_ordinal in the validated source snapshot",
        "public_message": "The selected construct is outside the frozen practical profile.",
        "public_message_utf8_bytes": 63,
        "forbidden_public_data": ["customer namespace", "customer member spelling", "source snippet",
                                  "compiler prose", "exception text", "host path", "culture", "stack trace"],
    }


def make_freeze() -> dict:
    inventory = read_json(INVENTORY)
    foundation = read_json(FOUNDATION)
    recursor = read_json(RECURSOR)
    capacity = read_json(CAPACITY)
    family_rows = identity_families(inventory)
    schema_rows = schemas()
    result = {
        "schema": "mpk.csharp_practical.t01_w09.freeze.v1",
        "work_item": "CSHARP-03-T01-W09",
        "status": "frozen_candidate_only",
        "semantic_profile": "mpk.csharp.practical.v1",
        "activation": "candidate_only",
        "predecessor_commit": "4ffd8b3a9918b6cae9e4d4704e4bc6b09a12cd5c",
        "identity_families": family_rows,
        "schemas": schema_rows,
        "schema_type_system": schema_type_system(),
        "expression_union": expression_union(),
        "semantic_context_binding": {
            "entry_lookup": "profile_entry_sha256 must resolve exactly one entry in the referenced immutable profile_registry revision",
            "required_equalities": [
                {"left": "semantic_context.source_language", "right": "profile_entry.source_language"},
                {"left": "semantic_context.semantic_profile", "right": "profile_entry.semantic_profile"},
                {"left": "semantic_context.semantic_parameters.schema", "right": "profile_entry.semantic_parameters_schema"},
                {"left": "semantic_context.foundation_descriptor", "right": "profile_entry.foundation_descriptor"},
            ],
            "validated_request_selection": "selection schema, strict shape, and hash domain must equal profile_entry.selection_schema",
            "context_repetition": "every repeated semantic_context is field-complete equal to the validated request context",
            "hash_only_or_projected_context": "reject",
        },
        "canonical_json": {
            "profile": "mpk.csharp.canonical_json.v1",
            "encoding": "UTF-8 without BOM",
            "outside_string_whitespace": "forbidden",
            "object_member_order": "schema-declared order; boundary value fields use boundary-contract order",
            "duplicate_members": "reject before object construction",
            "unknown_members": "reject",
            "numbers": "only schema-declared integer metadata fields use canonical base-10 JSON tokens through their exact u8/u16/u32/u64 range; semantic i64/u64 values use decimal strings; no fraction, exponent, plus, negative zero, or redundant leading zero",
            "wide_and_special_scalars": {
                "i64_u64_duration_instant": "canonical decimal string",
                "decimal": "section 10.2 normalized or fixed-scale ASCII string selected by the field codec",
                "binary32": "exactly eight lowercase hexadecimal IEEE-bit digits",
                "binary64": "exactly sixteen lowercase hexadecimal IEEE-bit digits",
                "date": "yyyy-MM-dd",
                "time": "HH:mm:ss.fffffff",
                "guid_n": "32 lowercase hexadecimal digits",
                "guid_d": "8-4-4-4-12 lowercase hexadecimal digits",
            },
            "strings": "shortest UTF-8 for Unicode scalars; quote and reverse-solidus use two-byte escapes; U+0000..U+001F and lone UTF-16 surrogates use lowercase \\uXXXX; slash is unescaped; short control escapes are forbidden",
            "sum": "object members tag then payload; payload is present only for a payload-carrying active arm",
            "map": "ordered array of typed key/value entries, never a JSON object",
            "missing_null_value": "missing omits the field; null emits JSON null; value emits exactly one canonical payload",
            "hash_preimage": "ASCII hash-domain bytes, one 0x00 byte, then canonical JSON bytes without a trailing newline",
        },
        "boundary_handoff": {
            "classification": "MPK verification-overlay transport",
            "application_protocol": False,
            "input_order": ["capture raw bytes and provenance", "strictly parse canonical document",
                            "construct complete typed value", "hash raw bytes and canonical value",
                            "bind both identities into manifest and evidence", "invoke selected original method"],
            "output_order": ["capture returned complete typed value", "encode canonical document",
                             "reparse canonical bytes", "compare field-complete typed values",
                             "bind value, bytes, and reparsed value into evidence"],
            "bypass": "reject",
            "hash_only_equivalence": "forbidden",
            "external_adapter": "untrusted and outside the certificate unless another profile verifies it",
        },
        "transition": {
            "version_rule": {"carrier": "u64", "success": "new_version = old_version + 1 by checked addition",
                             "replay": "unchanged", "error": "unchanged",
                             "overflow": "version_exhausted error arm"},
            "event_rule": "the bounded immutable event sequence preserves source append order exactly",
            "response_rule": "new success uses the computed response; replay uses the complete stored response",
            "precedence": ["ordinary_boundary_preconditions", "retained_key_lookup",
                           "equal_snapshot_replay_or_idempotency_conflict", "expected_version_conflict",
                           "idempotency_history_capacity", "version_exhausted",
                           "accepted_command_case_and_declared_business_errors", "new_success"],
            "idempotency": {
                "modes": ["disabled", "complete_snapshot"],
                "required_retained_record": ["key", "complete_command_snapshot", "complete_context_snapshot", "response"],
                "source_equality": "field-complete source equality proved equivalent to exact canonical field encodings",
                "eligible_fields": "recursively reflexive admitted equality only",
                "ineligible_fields": ["float", "double", "any recursively containing type", "any other non-reflexive equality"],
                "digest_substitution": "forbidden",
                "eviction": "none; full history returns the declared capacity error",
                "unavailable_when_incomplete": True,
            },
        },
        "diagnostics": diagnostics(),
        "frontend_linkage": {
            "success_equalities": [
                {"left": "success.request_sha256", "right": "request.request_sha256"},
                {"left": "success.semantic_context", "right": "request.semantic_request.semantic_context"},
                {"left": "success.artifacts.semantic_context", "right": "success.semantic_context"},
                {"left": "success.artifacts.selection_sha256", "right": "request.semantic_request.selection.selection_sha256"},
            ],
            "success_precondition": "all complete source artifacts and their hashes validate before success construction",
            "diagnostic_raw_identity": "raw_request_sha256 and raw_request_size_bytes are computed from the exact input bytes before parsing",
            "diagnostic_unvalidated": "allowed only until strict request and semantic-context validation succeeds",
            "diagnostic_validated_equalities": [
                {"left": "diagnostic.request_linkage.request_sha256", "right": "request.request_sha256"},
                {"left": "diagnostic.request_linkage.semantic_context", "right": "request.semantic_request.semantic_context"},
            ],
            "comparison": "field-complete typed equality; a digest never substitutes for a repeated value",
            "partial_artifacts_on_failure": "forbidden",
        },
        "limits": {
            "schema": "mpk.csharp.limits.v1",
            "comparison": "inclusive",
            "practical": practical_limits(capacity),
            "retained_scalar_v0": retained_limits(),
            "capacity_evidence": "develop/migrations/csharp-03/probes/checker-capacity.json",
            "capacity_rule": "the profile accepts limit-1 and limit and rejects limit+1 before checker invocation; both checkers must accept all three probe certificates",
        },
        "executable_dispatch": {
            "decision": "one context-dispatched C# executable and one bundle serve both C# profiles",
            "binary_name": "csharp2vir",
            "binary_path": "csharp2vir.dll",
            "successor_bundle_id": "frontend.csharp.csharp2vir.candidate.v2",
            "profiles": ["mpk.csharp.practical.v1", "mpk.csharp.scalar.v0"],
            "dispatch_key": "validated semantic_context.semantic_profile",
            "ambient_flag": "forbidden",
            "fallback": "forbidden",
            "mixed_artifact_family": "reject",
            "scalar_equivalence_gate": "byte-identical predecessor source verdicts, obligations, and Certificate v0 bytes",
        },
        "termination": {
            "call_graph": "finite and acyclic",
            "loops": "every admitted loop has invariant and well-founded decreases proof",
            "total_required_routes": ["boundary", "transition", "example", "public practical profile"],
            "partial_callee_on_total_path": "reject",
            "bounded_quantifiers": "finite bounds evaluated before body traversal",
            "static_networks": "finite counts checked before generation; balanced composition depth is bounded",
        },
        "ownership": {
            "inventory_path": "develop/migrations/csharp-03/artifact-consumer-inventory.json",
            "inventory_raw_sha256": sha(INVENTORY.read_bytes()),
            "migration_set": inventory["atomic_migration_set"],
            "rollback_set": inventory["whole_image_rollback_set"],
        },
        "evidence": {
            "freeze_generator_path": "develop/probes/csharp-03/profile_freeze.py",
            "freeze_generator_raw_sha256": sha(Path(__file__).read_bytes()),
            "retained_limit_source_path": "csharp-tools/csharp2vir/FrontendLimits.cs",
            "retained_limit_source_raw_sha256": sha(LIMIT_SOURCE.read_bytes()),
            "foundation_descriptor_path": "develop/migrations/csharp-03/foundation/foundation-descriptor.json",
            "foundation_descriptor_content_sha256": foundation["content_sha256"],
            "foundation_descriptor_raw_sha256": sha(FOUNDATION.read_bytes()),
            "recursor_evidence_path": "develop/migrations/csharp-03/probes/recursor-feasibility.json",
            "recursor_evidence_raw_sha256": sha(RECURSOR.read_bytes()),
            "capacity_evidence_path": "develop/migrations/csharp-03/probes/checker-capacity.json",
            "capacity_evidence_raw_sha256": sha(CAPACITY.read_bytes()),
            "capacity_source_inventory_sha256": capacity["source_inventory_sha256"],
            "checker_invocations": 48,
            "checker_acceptances": 48,
            "core_or_checker_change": False,
        },
        "publication_owner": "CSHARP-03-T01-W10",
        "content_hash_domain": DOMAIN,
    }
    result["content_sha256"] = domain_hash(DOMAIN, result)
    validate_freeze(result)
    return result


def validate_freeze(value: dict) -> None:
    preimage = copy.deepcopy(value)
    actual = preimage.pop("content_sha256")
    assert actual == domain_hash(DOMAIN, preimage)
    assert [row["family"] for row in value["identity_families"]] == [
        "semantic_registry", "semantic_context", "semantic_parameters", "selection",
        "profile_contract", "source_artifact", "foundation", "vir", "frontend_protocol",
        "source_map", "source_manifest", "vc_skeleton", "release", "policy_evidence",
        "program_assembly", "ai", "api"]
    identities = [(row["family"], item) for row in value["identity_families"]
                  for item in row["successor_identities"]]
    retained_identities = [item for row in value["identity_families"]
                           for item in row["retained_identities"]]
    domains = [(row["family"], item) for row in value["identity_families"]
               for item in row["successor_hash_domains"]]
    retained_domains = [item["id"] for row in value["identity_families"]
                        for item in row["retained_hash_domains"]]
    assert len(identities) == len({item for _, item in identities})
    assert len(domains) == len({item for _, item in domains})
    assert not ({item for _, item in identities} & set(retained_identities))
    assert not ({item for _, item in domains} & set(retained_domains))
    declared = {item for _, item in identities}
    assert {row["id"] for row in value["schemas"]} <= declared
    assert all(row["unknown_fields"] == "reject" and row["later_versions"] == "reject"
               and row["duplicate_keys"] == "reject_before_object_construction"
               for row in value["schemas"])
    assert all(row["hash_domain"] is None or row["hash_domain"] in {item for _, item in domains}
               for row in value["schemas"])
    assert all(row["hash_preimage_fields"] == ([field for field in row["ordered_fields"]
               if field != row["hash_field"]] if row["hash_domain"] is not None else None)
               for row in value["schemas"])
    assert all(row["hash_field"] is None or (row["ordered_fields"][-1] == row["hash_field"]
               and row["hash_domain"] is not None)
               for row in value["schemas"])
    assert all(set(row["ordered_fields"]) == set(row["field_types"])
               for row in value["schemas"])
    records = value["schema_type_system"]["nested_records"]
    assert len(records) == len({row["id"] for row in records})
    assert all(row["required_fields"] == row["ordered_fields"] and row["optional_fields"] == []
               and set(row["ordered_fields"]) == set(row["field_types"])
               and row["unknown_fields"] == "reject"
               and row["duplicate_keys"] == "reject_before_object_construction"
               and row["producer"].startswith("CSHARP-03-") for row in records)
    unions = value["schema_type_system"]["tagged_unions"]
    assert len(unions) == len({row["id"] for row in unions})
    for union in unions:
        assert union["unknown_tags"] == "reject"
        assert union["unknown_fields"] == "reject"
        assert union["duplicate_keys"] == "reject_before_object_construction"
        tags = [variant["tag"] for variant in union["variants"]]
        assert len(tags) == len(set(tags))
        for variant in union["variants"]:
            assert variant["required_fields"] == variant["ordered_fields"]
            assert variant["optional_fields"] == []
            assert set(variant["ordered_fields"]) == set(variant["field_types"])
            assert variant["ordered_fields"][0] == union["tag_field"]
    expressions = value["expression_union"]
    assert expressions["duplicate_keys"] == "reject_before_object_construction"
    for variant in expressions["variants"]:
        assert variant["required_fields"] == variant["ordered_fields"]
        assert variant["optional_fields"] == []
        assert set(variant["ordered_fields"]) == set(variant["field_types"])

    atoms = set(value["schema_type_system"]["atoms"])
    record_ids = {row["id"] for row in records}
    union_ids = {row["id"] for row in unions}
    constructors = {
        "artifact_ref", "enum", "literal", "ordered_array", "ordered_nonempty_array",
        "ordered_unique_array", "sorted_unique_array", "sorted_nonempty_unique_array",
    }

    def closed_type(expression: str) -> bool:
        if expression in atoms or expression in record_ids or expression in union_ids or expression in declared:
            return True
        match = re.fullmatch(r"([a-z_]+)<(.+)>", expression)
        if not match or match.group(1) not in constructors:
            return False
        constructor, argument = match.groups()
        if constructor in {"literal", "enum"}:
            return bool(argument) and "<" not in argument and ">" not in argument
        if constructor == "artifact_ref":
            return argument in declared
        return closed_type(argument)

    field_types = [field_type for row in value["schemas"] for field_type in row["field_types"].values()]
    field_types += [field_type for row in records for field_type in row["field_types"].values()]
    field_types += [field_type for union in unions for variant in union["variants"]
                    for field_type in variant["field_types"].values()]
    field_types += [field_type for variant in expressions["variants"]
                    for field_type in variant["field_types"].values()]
    assert all(closed_type(field_type) for field_type in field_types), sorted(
        {field_type for field_type in field_types if not closed_type(field_type)})
    used_atoms = set()

    def collect_atoms(expression: str) -> None:
        if expression in atoms:
            used_atoms.add(expression)
            return
        match = re.fullmatch(r"([a-z_]+)<(.+)>", expression)
        if match and match.group(1) in {
            "ordered_array", "ordered_nonempty_array", "ordered_unique_array",
            "sorted_unique_array", "sorted_nonempty_unique_array",
        }:
            collect_atoms(match.group(2))

    for field_type in field_types:
        collect_atoms(field_type)
    assert used_atoms == atoms, sorted(atoms - used_atoms)
    assert len(value["diagnostics"]["families"]) == len(set(value["diagnostics"]["families"]))
    assert value["diagnostics"]["public_message_utf8_bytes"] == len(
        value["diagnostics"]["public_message"].encode()
    )
    limits = value["limits"]["practical"]
    assert len(limits) == len({row["id"] for row in limits})
    assert all(row["inclusive_maximum"] > 0 for row in limits)
    assert {row["classification"] for row in limits} == {
        "pre_invocation_structural", "runtime_value_predicate_vc"}


def vector_rows(freeze: dict) -> list[dict]:
    rows = []
    default_tasks = {
        "schema": "CSHARP-03-T02-W01",
        "identity": "CSHARP-03-T02-W09",
        "context": "CSHARP-03-T02-W01",
        "frontend_linkage": "CSHARP-03-T02-W08",
        "canonical_json": "CSHARP-03-T05-W02",
        "boundary": "CSHARP-03-T05-W03",
        "transition": "CSHARP-03-T05-W05",
        "idempotency": "CSHARP-03-T05-W05",
        "diagnostic": "CSHARP-03-T02-W08",
        "limit": "CSHARP-03-T02-W08",
        "capacity": "CSHARP-03-T06-W09",
        "dispatch": "CSHARP-03-T02-W08",
        "termination": "CSHARP-03-T06-W01",
    }
    primary_tests = {
        "CSHARP-03-T02-W01": "crates/mpk-vc/tests/csharp_practical_registry.rs",
        "CSHARP-03-T02-W02": "crates/mpk-vc/tests/csharp_practical_vir_model.rs",
        "CSHARP-03-T02-W04": "crates/mpk-vc/tests/csharp_practical_source_artifacts.rs",
        "CSHARP-03-T02-W07": "crates/mpk-cli/tests/csharp_practical_frontend_protocol.rs",
        "CSHARP-03-T02-W08": "crates/mpk-cli/tests/csharp_practical_migration.rs",
        "CSHARP-03-T02-W09": "crates/mpk-cli/tests/csharp_practical_migration.rs",
        "CSHARP-03-T03-W01": "crates/mpk-cli/tests/csharp_practical_capture.rs",
        "CSHARP-03-T03-W03": "crates/mpk-cli/tests/csharp_practical_types.rs",
        "CSHARP-03-T03-W04": "crates/mpk-cli/tests/csharp_practical_types.rs",
        "CSHARP-03-T03-W07": "crates/mpk-cli/tests/csharp_practical_collections.rs",
        "CSHARP-03-T03-W08": "crates/mpk-cli/tests/csharp_practical_collections.rs",
        "CSHARP-03-T03-W09": "crates/mpk-cli/tests/csharp_practical_collections.rs",
        "CSHARP-03-T03-W10": "crates/mpk-cli/tests/csharp_practical_codecs.rs",
        "CSHARP-03-T03-W12": "crates/mpk-cli/tests/csharp_practical_domain.rs",
        "CSHARP-03-T03-W14": "crates/mpk-cli/tests/csharp_practical_domain.rs",
        "CSHARP-03-T04-W01": "crates/mpk-cli/tests/csharp_practical_control.rs",
        "CSHARP-03-T04-W03": "crates/mpk-cli/tests/csharp_practical_control.rs",
        "CSHARP-03-T04-W04": "crates/mpk-cli/tests/csharp_practical_control.rs",
        "CSHARP-03-T04-W05": "crates/mpk-cli/tests/csharp_practical_control.rs",
        "CSHARP-03-T05-W01": "crates/mpk-cli/tests/csharp_practical_boundary.rs",
        "CSHARP-03-T05-W02": "crates/mpk-cli/tests/csharp_practical_boundary.rs",
        "CSHARP-03-T05-W03": "crates/mpk-cli/tests/csharp_practical_boundary.rs",
        "CSHARP-03-T05-W04": "crates/mpk-cli/tests/csharp_practical_transition.rs",
        "CSHARP-03-T05-W05": "crates/mpk-cli/tests/csharp_practical_transition.rs",
        "CSHARP-03-T06-W01": "crates/mpk-vc/tests/csharp_practical_vc.rs",
        "CSHARP-03-T06-W06": "crates/mpk-vc/tests/csharp_practical_vc.rs",
        "CSHARP-03-T06-W09": "crates/mpk-vc/tests/csharp_practical_vc.rs",
    }

    def add(family: str, identity: str, inputs: object, expected: object,
            task: str | None = None) -> None:
        task = task or default_tasks[family]
        test = primary_tests[task]
        rows.append({"id": f"{family}.{identity}", "family": family, "inputs": inputs,
                     "expected": expected, "implementation_owner": task,
                     "production_test_owner": f"{test}#{task}"})

    for item in freeze["schemas"]:
        key = item["id"].replace("mpk.", "").replace(".", "_")
        task = item["producer"]
        add("schema", f"{key}.valid", {"schema": item["id"], "fields": item["ordered_fields"]}, {"accept": True}, task)
        add("schema", f"{key}.later_version", {"schema": item["id"], "mutation": "later_version"}, {"reject": "schema_version"}, task)
        add("schema", f"{key}.unknown_field", {"schema": item["id"], "mutation": "append_unknown_field"}, {"reject": "unknown_field"}, task)
        add("schema", f"{key}.missing_field", {"schema": item["id"], "mutation": "remove_each_required_field"}, {"reject": "missing_field"}, task)
        add("schema", f"{key}.wrong_field_type", {"schema": item["id"], "mutation": "replace_each_field_with_wrong_json_type"}, {"reject": "field_type"}, task)
        add("schema", f"{key}.duplicate_key", {"schema": item["id"], "raw_utf8": '{"schema":"' + item["id"] + '","schema":"' + item["id"] + '"}'}, {"reject": "duplicate_key"}, task)
    for item in freeze["schema_type_system"]["nested_records"]:
        key = item["id"].replace(".", "_")
        first = item["ordered_fields"][0]
        task = item["producer"]
        add("schema", f"nested_{key}.valid", {"record": item["id"], "fields": item["ordered_fields"]}, {"accept": True}, task)
        add("schema", f"nested_{key}.unknown_field", {"record": item["id"], "mutation": "append_unknown_field"}, {"reject": "unknown_field"}, task)
        add("schema", f"nested_{key}.missing_field", {"record": item["id"], "mutation": "remove_each_required_field"}, {"reject": "missing_field"}, task)
        add("schema", f"nested_{key}.wrong_field_type", {"record": item["id"], "mutation": "replace_each_field_with_wrong_json_type"}, {"reject": "field_type"}, task)
        add("schema", f"nested_{key}.duplicate_key", {"record": item["id"], "raw_utf8": '{"' + first + '":0,"' + first + '":0}'}, {"reject": "duplicate_key"}, task)
    expression = freeze["expression_union"]
    for variant in expression["variants"]:
        tag = variant["tag"]
        task = expression["producer"]
        for suffix, inputs, expected in [
            ("valid", {"tag": tag, "fields": variant["ordered_fields"]}, {"accept": True}),
            ("unknown_field", {"tag": tag, "mutation": "append_unknown_field"}, {"reject": "unknown_field"}),
            ("missing_field", {"tag": tag, "mutation": "remove_each_required_field"}, {"reject": "missing_field"}),
            ("wrong_field_type", {"tag": tag, "mutation": "replace_each_field_with_wrong_json_type"}, {"reject": "field_type"}),
            ("duplicate_key", {"tag": tag, "raw_utf8": '{"tag":"' + tag + '","tag":"' + tag + '"}'}, {"reject": "duplicate_key"}),
        ]:
            add("schema", f"expression.{tag}.{suffix}", inputs, expected, task)
    add("schema", "expression.unknown_tag", {"tag": "future"}, {"reject": "unknown_expression_tag"}, expression["producer"])
    for union in freeze["schema_type_system"]["tagged_unions"]:
        task = union["producer"]
        for variant in union["variants"]:
            tag = variant["tag"]
            for suffix, inputs, expected in [
                ("valid", {union["tag_field"]: tag, "fields": variant["ordered_fields"]}, {"accept": True}),
                ("unknown_field", {union["tag_field"]: tag, "mutation": "append_unknown_field"}, {"reject": "unknown_field"}),
                ("missing_field", {union["tag_field"]: tag, "mutation": "remove_each_required_field"}, {"reject": "missing_field"}),
                ("wrong_field_type", {union["tag_field"]: tag, "mutation": "replace_each_field_with_wrong_json_type"}, {"reject": "field_type"}),
                ("duplicate_key", {"raw_utf8": '{"' + union["tag_field"] + '":"' + tag + '","' + union["tag_field"] + '":"' + tag + '"}'}, {"reject": "duplicate_key"}),
            ]:
                add("schema", f"union_{union['id']}.{tag}.{suffix}", inputs, expected, task)
        add("schema", f"union_{union['id']}.unknown_tag",
            {union["tag_field"]: "future"}, {"reject": f"unknown_{union['id']}_tag"}, task)
    add("identity", "all_successor_names_unique", {"families": len(freeze["identity_families"])}, {"accept": True})
    add("identity", "duplicate_identity", {"mutation": "assign_one_identity_to_two_families"}, {"reject": "duplicate_successor_identity"})
    add("identity", "duplicate_hash_domain", {"mutation": "assign_one_domain_to_two_families"}, {"reject": "duplicate_successor_hash_domain"})
    add("identity", "old_new_mixed", {"mutation": "old_schema_with_successor_context"}, {"reject": "mixed_artifact_family"})

    for identity, inputs, expected in [
        ("valid", {"entry_resolves": True, "all_entry_fields_equal": True,
                   "selection_schema_matches": True, "context_repetitions_equal": True}, {"accept": True}),
        ("entry_hash_mismatch", {"entry_resolves": False}, {"reject": "profile_entry_mismatch"}),
        ("language_profile_mismatch", {"source_language": "csharp", "entry_source_language": "rust"}, {"reject": "semantic_context_mismatch"}),
        ("parameters_schema_mismatch", {"parameters_schema_matches": False}, {"reject": "semantic_parameters_mismatch"}),
        ("selection_schema_mismatch", {"selection_schema_matches": False}, {"reject": "selection_schema_mismatch"}),
        ("projected_context", {"complete_context": False, "context_sha256": "equal"}, {"reject": "projected_semantic_context"}),
    ]:
        add("context", identity, inputs, expected)

    for identity, inputs, expected in [
        ("success", {"success_equalities": True, "complete_artifacts": True}, {"accept": True}),
        ("success_request_mismatch", {"request_sha256_equal": False}, {"reject": "frontend_request_linkage"}),
        ("success_context_mismatch", {"success_request_context_equal": False}, {"reject": "semantic_context_mismatch"}),
        ("artifact_context_mismatch", {"artifact_success_context_equal": False}, {"reject": "semantic_context_mismatch"}),
        ("artifact_selection_mismatch", {"artifact_request_selection_equal": False}, {"reject": "selection_mismatch"}),
        ("diagnostic_unvalidated_before_validation", {"request_validation_succeeded": False, "linkage": "unvalidated"}, {"accept": True}),
        ("diagnostic_unvalidated_after_validation", {"request_validation_succeeded": True, "linkage": "unvalidated"}, {"reject": "diagnostic_request_linkage"}),
        ("diagnostic_validated_request_mismatch", {"linkage": "validated", "request_sha256_equal": False}, {"reject": "diagnostic_request_linkage"}),
        ("diagnostic_validated_context_mismatch", {"linkage": "validated", "semantic_context_equal": False}, {"reject": "semantic_context_mismatch"}),
        ("diagnostic_partial_artifacts", {"status": "rejected", "partial_artifacts": True}, {"reject": "partial_frontend_artifacts"}),
    ]:
        add("frontend_linkage", identity, inputs, expected)

    for identity, raw, expected in [
        ("canonical", '{"a":null,"b":"\\ud800"}', {"accept": True}),
        ("duplicate", '{"a":0,"a":1}', {"reject": "duplicate_key"}),
        ("whitespace", '{"a": null}', {"reject": "noncanonical_json"}),
        ("member_order", '{"b":0,"a":0}', {"reject": "member_order"}),
        ("uppercase_escape", '{"a":"\\uD800"}', {"reject": "noncanonical_escape"}),
        ("short_escape", '{"a":"\\n"}', {"reject": "noncanonical_escape"}),
        ("floating_token", '{"a":1.0}', {"reject": "floating_json_token"}),
    ]:
        add("canonical_json", identity, {"raw_utf8": raw}, expected)

    for identity, inputs, expected in [
        ("presence.missing", {"field_state": "missing", "wire": "absent"}, {"value_arm": "missing"}),
        ("presence.null", {"field_state": "null", "wire": None}, {"value_arm": "null"}),
        ("presence.value", {"field_state": "value", "wire": 7}, {"value_arm": "value", "payload": 7}),
        ("required_missing", {"required": True, "field_state": "missing"}, {"reject": "required_missing"}),
        ("nonnull_null", {"nullable": False, "field_state": "null"}, {"reject": "null_for_nonnullable"}),
        ("optional_default", {"missing_default": 7, "field_state": "missing"}, {"typed_value": 7}),
        ("raw_canonical_link", {"raw_sha256": "raw", "canonical_value_sha256": "value", "both_retained": True}, {"accept": True}),
        ("input_bypass", {"canonical_object_supplied_without_parser": True}, {"reject": "boundary_bypass"}),
        ("output_reparse", {"source_value": [1, 2], "reparsed_value": [1, 2]}, {"accept": True}),
        ("output_mismatch", {"source_value": [1, 2], "reparsed_value": [2, 1]}, {"reject": "output_reparse_mismatch"}),
    ]:
        add("boundary", identity, inputs, expected)

    for identity, inputs, expected in [
        ("replay_precedes_version", {"retained_key": True, "snapshots_equal": True, "version_matches": False}, {"outcome": "replay", "events": [], "state": "unchanged"}),
        ("conflict_precedes_version", {"retained_key": True, "snapshots_equal": False, "version_matches": False}, {"error": "idempotency_conflict", "state": "unchanged"}),
        ("version_precedes_capacity", {"retained_key": False, "version_matches": False, "history_full": True}, {"error": "version_conflict", "state": "unchanged"}),
        ("capacity_precedes_exhaustion", {"retained_key": False, "version_matches": True, "history_full": True, "version": "18446744073709551615"}, {"error": "idempotency_capacity", "state": "unchanged"}),
        ("version_exhausted", {"idempotency": "disabled", "version_matches": True, "version": "18446744073709551615"}, {"error": "version_exhausted", "state": "unchanged"}),
        ("business_error_order", {"declared_errors": ["first", "second"], "both_hold": True}, {"error": "first", "state": "unchanged"}),
        ("success", {"version": "41", "events": ["e1", "e2"], "response": "r"}, {"version": "42", "events": ["e1", "e2"], "response": "r"}),
    ]:
        add("transition", identity, inputs, expected)

    for identity, inputs, expected in [
        ("eligible", {"complete_command": True, "complete_context": True, "reflexive": True, "encoding_equivalence_proved": True}, {"mode": "complete_snapshot"}),
        ("missing_command_field", {"complete_command": False}, {"mode": "unavailable"}),
        ("missing_context_field", {"complete_context": False}, {"mode": "unavailable"}),
        ("float_field", {"field_type": "float"}, {"mode": "unavailable"}),
        ("double_field", {"field_type": "double"}, {"mode": "unavailable"}),
        ("nested_nonreflexive", {"field_type": "record<array<double>>"}, {"mode": "unavailable"}),
        ("unproved_equality", {"encoding_equivalence_proved": False}, {"mode": "unavailable"}),
        ("digest_only", {"source_equality": False, "digest_equal": True}, {"mode": "unavailable"}),
    ]:
        add("idempotency", identity, inputs, expected)

    flattened = [family for phase in freeze["diagnostics"]["phase_precedence"] for family in phase["families_in_precedence_order"]]
    for index in range(len(flattened) - 1):
        add("diagnostic", f"precedence_{index:02}", {"failures": [flattened[index + 1], flattened[index]]}, {"primary": flattened[index]})
    for identity, inputs, expected in [
        ("phase_min", {"phase": 0, "code": "CSHARP_PRACTICAL_PROTOCOL"}, {"accept": True}),
        ("phase_max", {"phase": 8, "code": "CSHARP_PRACTICAL_LOWERING"}, {"accept": True}),
        ("phase_above", {"phase": 9, "code": "CSHARP_PRACTICAL_LOWERING"}, {"reject": "diagnostic_phase"}),
        ("mixed_phase", {"phase": 0, "code": "CSHARP_PRACTICAL_LOWERING"}, {"reject": "diagnostic_phase_code"}),
        ("invalid_location", {"source_size_bytes": 10, "location": {"source_file_ordinal": 0, "start_byte": 8, "end_byte": 11}}, {"reject": "diagnostic_location"}),
        ("forbidden_public_data", {"message_contains": "customer member spelling"}, {"reject": "diagnostic_public_data"}),
    ]:
        add("diagnostic", identity, inputs, expected)

    for group, entries in [("practical", freeze["limits"]["practical"]),
                           ("retained", freeze["limits"]["retained_scalar_v0"])]:
        for item in entries:
            maximum = item["inclusive_maximum"]
            for suffix, value in [("below", maximum - 1), ("at", maximum), ("above", maximum + 1)]:
                if suffix != "above":
                    expected = {"accept": True}
                else:
                    disposition = item.get("disposition", "rejected")
                    expected = {"reject": {"rejected": "limit_exceeded", "diagnostic_budget": "diagnostic_budget",
                                           "output_limit": "output_limit"}[disposition]}
                add("limit", f"{group}.{item['id']}.{suffix}", {"counter": item["id"], "value": value,
                    "inclusive_maximum": maximum}, expected,
                    item["implementation_owner"] if group == "practical" else "CSHARP-03-T02-W08")

    for case in read_json(CAPACITY)["probe"]["cases"]:
        add("capacity", case["id"], {"counter": case["family"], "value": case["counter_value"],
            "certificate_sha256": case["raw_sha256"]}, {"profile": case["profile_expected"], "rust_checker": "accepted", "reference_checker": "accepted"})

    for identity, inputs, expected in [
        ("scalar", {"semantic_profile": "mpk.csharp.scalar.v0"}, {"bundle": "frontend.csharp.csharp2vir.candidate.v2"}),
        ("practical", {"semantic_profile": "mpk.csharp.practical.v1"}, {"bundle": "frontend.csharp.csharp2vir.candidate.v2"}),
        ("unknown", {"semantic_profile": "mpk.csharp.future.v1"}, {"reject": "unknown_semantic_context"}),
        ("ambient_override", {"semantic_profile": "mpk.csharp.scalar.v0", "environment_override": "practical"}, {"reject": "ambient_profile_selection"}),
    ]:
        add("dispatch", identity, inputs, expected)
    for identity, inputs, expected in [
        ("boundary_total", {"route": "boundary", "termination": "total"}, {"accept": True}),
        ("transition_partial", {"route": "transition", "termination": "partial"}, {"reject": "total_required"}),
        ("partial_callee", {"route": "example", "reachable_partial_callee": True}, {"reject": "partial_callee_on_total_path"}),
    ]:
        add("termination", identity, inputs, expected)
    rows.sort(key=lambda row: row["id"])
    return rows


def make_vectors(freeze: dict) -> dict:
    rows = vector_rows(freeze)
    ids = [row["id"] for row in rows]
    return {
        "schema": "mpk.csharp_practical.t01_w09.freeze_vectors.v1",
        "work_item": "CSHARP-03-T01-W09",
        "freeze_content_sha256": freeze["content_sha256"],
        "owner_test": OWNER,
        "publication_owner": "CSHARP-03-T01-W10",
        "vector_count": len(rows),
        "vector_ids_sha256": sha(canonical(ids)),
        "vectors": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--update", action="store_true")
    group.add_argument("--check", action="store_true")
    args = parser.parse_args()
    freeze = make_freeze()
    vectors = make_vectors(freeze)
    if args.update:
        FREEZE.parent.mkdir(parents=True, exist_ok=True)
        FREEZE.write_bytes(bytes_of(freeze))
        VECTORS.write_bytes(bytes_of(vectors))
    else:
        assert FREEZE.read_bytes() == bytes_of(freeze), "profile freeze drift"
        assert VECTORS.read_bytes() == bytes_of(vectors), "profile freeze vector drift"
    print(f"W09 freeze: 17 identity families, {len(freeze['schemas'])} strict schemas, "
          f"{len(freeze['limits']['practical'])} practical limits, {len(vectors['vectors'])} vectors.")


if __name__ == "__main__":
    main()
