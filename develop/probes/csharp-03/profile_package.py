#!/usr/bin/env python3
"""Publish the W09 C# practical freeze as the closed W10 specification package."""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
WORK_ITEM = "CSHARP-03-T01-W10"
OWNER = "crates/mpk-vc/tests/csharp_practical_spec.rs"
SCHEMA = "mpk.csharp.practical.profile.conformance.v1"
OUTPUT_PATH = "develop/specs/vectors/csharp-practical-profile-v1.json"
PROFILE_SPEC = "develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md"
SHARED_SPEC = "develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md"
FOUNDATION_SPEC = "develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md"
DESIGN = "develop/docs/08_csharp_practical_subset_design.md"
PLAN = "develop/docs/08_csharp_practical_subset_design-todo.md"
LEDGER = "develop/docs/csharp-03-implementation-traceability-ledger.md"
INVENTORY = "develop/migrations/csharp-03/artifact-consumer-inventory.json"
W09_FREEZE = "develop/migrations/csharp-03/freeze/profile-freeze.json"
W09_VECTORS = "develop/migrations/csharp-03/freeze/profile-freeze-vectors.json"
W09_COMMIT = "17525292755c4e508acd9300cfa72d20cdf9bb92"
JAVA_GATE = "scripts/check-java-frontend.sh"
AGGREGATE_GATE = "scripts/check-all.sh"
PRACTICAL_GATE = "scripts/check-csharp-practical-release.sh"
JAVA_GATE_PREDECESSOR_SHA256 = (
    "4cd941c03a111ac9222c47b33704d9494e8d12055dfd44b6e78b45f175e5bd81"
)
AGGREGATE_GATE_PREDECESSOR_SHA256 = (
    "62a7fe8c1e9a68cf4edc31775b5f3fb47176fa2768521d5a5c939d8543750da4"
)

PUBLICATION_PATHS = [PROFILE_SPEC, SHARED_SPEC, OUTPUT_PATH]

EVIDENCE_RECORDS = [
    ("develop/migrations/csharp-03/baseline.json", "CSHARP-03-T01-W01", "entry_baseline"),
    (INVENTORY, "CSHARP-03-T01-W02", "artifact_consumer_inventory"),
    (
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "CSHARP-03-T01-W03",
        "frontend_toolchain_closure",
    ),
    (
        "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
        "CSHARP-03-T01-W03",
        "private_candidate_inventory",
    ),
    (
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
        "CSHARP-03-T01-W04",
        "roslyn_data_construction",
    ),
    (
        "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
        "CSHARP-03-T01-W05",
        "roslyn_control_exception_pattern",
    ),
    (
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
        "CSHARP-03-T01-W06",
        "roslyn_dependency_generic_suspension",
    ),
    (
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
        "CSHARP-03-T01-W07",
        "runtime_primitive_string_numeric_codec",
    ),
    (
        "develop/migrations/csharp-03/foundation/foundation-definitions.json",
        "CSHARP-03-T01-W08",
        "foundation_definitions",
    ),
    (
        "develop/migrations/csharp-03/foundation/foundation-descriptor.json",
        "CSHARP-03-T01-W08",
        "foundation_descriptor",
    ),
    (
        "develop/specs/vectors/csharp-practical-foundation-v1.json",
        "CSHARP-03-T01-W08",
        "foundation_conformance",
    ),
    (
        "develop/migrations/csharp-03/probes/runtime-foundation-data.json",
        "CSHARP-03-T01-W08",
        "runtime_foundation_data",
    ),
    (
        "develop/migrations/csharp-03/probes/recursor-feasibility.json",
        "CSHARP-03-T01-W09",
        "recursor_feasibility",
    ),
    (
        "develop/migrations/csharp-03/probes/checker-capacity.json",
        "CSHARP-03-T01-W09",
        "checker_capacity",
    ),
    (W09_FREEZE, "CSHARP-03-T01-W09", "private_freeze"),
    (W09_VECTORS, "CSHARP-03-T01-W09", "private_freeze_vectors"),
]

FREEZE_REQUIREMENTS = [
    ("entry_and_active_release_baseline", "CSHARP-03-T01-W01", [EVIDENCE_RECORDS[0][0]]),
    ("artifact_and_consumer_inventory", "CSHARP-03-T01-W02", [INVENTORY]),
    (
        "frontend_and_toolchain_closure",
        "CSHARP-03-T01-W03",
        [EVIDENCE_RECORDS[2][0], EVIDENCE_RECORDS[3][0]],
    ),
    ("roslyn_data_and_construction", "CSHARP-03-T01-W04", [EVIDENCE_RECORDS[4][0]]),
    ("roslyn_control_exception_pattern", "CSHARP-03-T01-W05", [EVIDENCE_RECORDS[5][0]]),
    (
        "dependency_generic_iterator_async_rejections",
        "CSHARP-03-T01-W06",
        [EVIDENCE_RECORDS[6][0]],
    ),
    ("runtime_primitive_string_numeric_codec", "CSHARP-03-T01-W07", [EVIDENCE_RECORDS[7][0]]),
    (
        "foundation_specialization_binding_and_data_semantics",
        "CSHARP-03-T01-W08",
        [
            FOUNDATION_SPEC,
            EVIDENCE_RECORDS[8][0],
            EVIDENCE_RECORDS[9][0],
            EVIDENCE_RECORDS[10][0],
            EVIDENCE_RECORDS[11][0],
        ],
    ),
    (
        "contract_boundary_transition_identity_and_limit_freeze",
        "CSHARP-03-T01-W09",
        [EVIDENCE_RECORDS[12][0], EVIDENCE_RECORDS[13][0], W09_FREEZE, W09_VECTORS],
    ),
    (
        "normative_package_manifest_owners_upgrade_and_gate_routing",
        WORK_ITEM,
        [PROFILE_SPEC, SHARED_SPEC, OUTPUT_PATH, "develop/specs/vectors/manifest.json"],
    ),
]

EXCLUDED_SOURCE_FAMILIES = [
    (
        "mutable_identity_and_unsafe_storage",
        ["mutable shared object graph", "observable object identity", "weak reference", "finalizer", "unsafe storage", "ref-like storage"],
        "CSHARP_PRACTICAL_TYPE",
        "CSHARP-03-T03-W03",
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
    ),
    (
        "inheritance_and_dynamic_dispatch",
        ["ordinary source inheritance", "virtual dispatch", "interface dispatch", "user-defined operator", "user-defined conversion"],
        "CSHARP_PRACTICAL_TYPE",
        "CSHARP-03-T03-W03",
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
    ),
    (
        "delegates_dynamic_and_runtime_codegen",
        ["delegate", "lambda", "expression tree", "query", "LINQ", "reflection", "dynamic", "runtime code generation"],
        "CSHARP_PRACTICAL_DECLARATION",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "user_defined_generics",
        ["generic type declaration", "generic method", "closed user generic use", "type parameter", "constraint", "variance", "generic inference"],
        "CSHARP_PRACTICAL_GENERIC",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "framework_collections_and_unsupported_arrays",
        ["List<T>", "Dictionary<K,V>", "System.Collections.Immutable", "span", "caller comparer", "array covariance", "multidimensional array", "jagged array"],
        "CSHARP_PRACTICAL_COLLECTION",
        "CSHARP-03-T03-W07",
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
    ),
    (
        "ambient_or_general_text_processing",
        ["culture-sensitive text", "normalization", "regular expression", "general source parse", "general source format", "resource", "globalization", "ambient locale"],
        "CSHARP_PRACTICAL_PARSE_FORMAT",
        "CSHARP-03-T03-W10",
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
    ),
    (
        "external_effects_and_concurrency",
        ["filesystem", "database", "network", "clock", "random", "environment", "console", "process", "synchronization", "thread", "scheduler", "cancellation"],
        "CSHARP_PRACTICAL_EFFECT",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "iterator_and_enumeration_protocol",
        ["iterator method", "yield", "IEnumerable", "IEnumerator", "IAsyncEnumerable<T>", "IAsyncEnumerator<T>", "iterator state machine"],
        "CSHARP_PRACTICAL_EFFECT",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "async_and_task_protocol",
        ["async", "await", "Task", "Task<T>", "ValueTask", "ValueTask<T>", "custom awaiter", "task race", "parallel execution", "async state machine"],
        "CSHARP_PRACTICAL_EFFECT",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "catchable_resource_exhaustion",
        ["OutOfMemoryException", "StackOverflowException", "catchable resource exhaustion"],
        "CSHARP_PRACTICAL_EXCEPTION",
        "CSHARP-03-T04-W04",
        "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
    ),
    (
        "ambient_build_and_mpk_source_dependency",
        ["project discovery", "NuGet discovery", "analyzer", "source generator", "ambient reference", "source-written attribute", "MPK package", "MPK assembly", "MPK namespace", "MPK base type", "MPK generated source", "MPK runtime component"],
        "CSHARP_PRACTICAL_DEPENDENCY",
        "CSHARP-03-T03-W01",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
    ),
    (
        "infrastructure_and_serializer_claims",
        ["JSON serializer correctness", "HTTP transport correctness", "identity provider correctness", "database transaction correctness", "system clock correctness", "timezone database correctness"],
        "CSHARP_PRACTICAL_BOUNDARY",
        "CSHARP-03-T05-W01",
        "develop/migrations/csharp-03/freeze/profile-freeze-vectors.json",
    ),
]


def raw(path: str) -> bytes:
    return (ROOT / path).read_bytes()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def output_bytes(value: Any) -> bytes:
    return canonical(value) + b"\n"


def read_json(path: str) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise AssertionError(f"{path}: duplicate JSON member {key}")
            result[key] = value
        return result

    value = json.loads(raw(path).decode("utf-8"), object_pairs_hook=reject_duplicates)
    if not isinstance(value, dict):
        raise AssertionError(f"{path}: root must be an object")
    return value


def file_record(path: str, role: str, work_item: str | None = None) -> dict[str, Any]:
    data = raw(path)
    record: dict[str, Any] = {
        "path": path,
        "raw_sha256": sha(data),
        "role": role,
        "size_bytes": len(data),
    }
    if work_item is not None:
        value = read_json(path)
        if value.get("work_item") not in (None, work_item):
            raise AssertionError(f"{path}: work-item mismatch")
        record["schema"] = value["schema"]
        record["work_item"] = work_item
    return record


def ledger_owners() -> dict[str, str]:
    text = raw(LEDGER).decode("utf-8")
    section = text.split("<!-- work-item-ledger:start -->", 1)[1].split("<!-- work-item-ledger:end -->", 1)[0]
    rows: dict[str, str] = {}
    pattern = re.compile(r"^\| `(CSHARP-03-T\d{2}-W\d{2})` \| `[^`]+` \| `([^`]+)` \| `[^`]+` \|$")
    for line in section.splitlines():
        match = pattern.fullmatch(line)
        if match:
            item, owner = match.groups()
            if item in rows:
                raise AssertionError(f"duplicate ledger owner {item}")
            if owner != f"{owner.split('#', 1)[0]}#{item}":
                raise AssertionError(f"non-exact owner prefix for {item}")
            rows[item] = owner
    if len(rows) != 73:
        raise AssertionError(f"expected 73 ledger owners, got {len(rows)}")
    return rows


def normalize_wrapped_prose(value: str) -> str:
    result = ""
    for line in value.splitlines():
        fragment = line.strip()
        if not fragment:
            continue
        if not result or result.endswith(("-", "/")):
            result += fragment
        else:
            result += f" {fragment}"
    return result


def task_contracts() -> dict[str, dict[str, str]]:
    text = raw(PLAN).decode("utf-8")
    headings = list(
        re.finditer(r"^### (CSHARP-03-T\d{2}-W\d{2}) — (.+)$", text, re.MULTILINE)
    )
    contracts: dict[str, dict[str, str]] = {}
    for index, heading in enumerate(headings):
        item, title = heading.groups()
        end = headings[index + 1].start() if index + 1 < len(headings) else len(text)
        body = text[heading.end() : end]
        fields = {"title": title}
        for label, key in [
            ("Owns", "owns"),
            ("Exit gate", "exit_gate"),
            ("Verification", "verification"),
        ]:
            match = re.search(
                rf"^{re.escape(label)}:[ \t]*(?:\n)?(.*?)(?:\n\n|\Z)",
                body,
                re.MULTILINE | re.DOTALL,
            )
            if match is None:
                raise AssertionError(f"{item}: missing {label}")
            fields[key] = normalize_wrapped_prose(match.group(1))
        if item in contracts:
            raise AssertionError(f"duplicate task heading {item}")
        contracts[item] = fields
    if len(contracts) != 73:
        raise AssertionError(f"expected 73 task contracts, got {len(contracts)}")
    return contracts


def downstream_owners(
    owners: dict[str, str], contracts: dict[str, dict[str, str]]
) -> list[dict[str, Any]]:
    result = []
    for item in sorted(owners):
        stage = int(item.split("-T", 1)[1].split("-", 1)[0])
        if not 2 <= stage <= 8:
            continue
        result.append(
            {
                "entry_state_at_publication": "ready"
                if item == "CSHARP-03-T02-W01"
                else "serially_blocked",
                "exit_gate": contracts[item]["exit_gate"],
                "owns": contracts[item]["owns"],
                "primary_test_owner": owners[item],
                "requirement_anchor": f"{PLAN}#{item}",
                "title": contracts[item]["title"],
                "verification": contracts[item]["verification"],
                "work_item": item,
            }
        )
    if len(result) != 63:
        raise AssertionError(f"expected 63 downstream owner rows, got {len(result)}")
    return result


def freeze_requirement_owners(owners: dict[str, str]) -> list[dict[str, Any]]:
    rows = []
    for requirement, item, artifacts in FREEZE_REQUIREMENTS:
        rows.append(
            {
                "artifacts": artifacts,
                "primary_test_owner": owners[item],
                "requirement": requirement,
                "work_item": item,
            }
        )
    return rows


def name_owner_inventory(freeze: dict[str, Any], owners: dict[str, str]) -> dict[str, Any]:
    names = []
    for family in freeze["identity_families"]:
        common = {
            "family": family["family"],
            "implementation_owners": family["implementation_owners"],
        }
        for name in family["successor_identities"]:
            names.append({**common, "disposition": "successor", "kind": "identity", "name": name})
        for name in family["retained_identities"]:
            names.append({**common, "disposition": "retained", "kind": "identity", "name": name})
        for name in family["successor_hash_domains"]:
            names.append({**common, "disposition": "successor", "kind": "hash_domain", "name": name})
        for retained in family["retained_hash_domains"]:
            names.append(
                {
                    **common,
                    "decision": retained["decision"],
                    "disposition": "retained",
                    "kind": "hash_domain",
                    "name": retained["id"],
                }
            )
    names.sort(key=lambda row: (row["kind"], row["name"], row["disposition"]))
    name_keys = [(row["kind"], row["name"]) for row in names]
    if len(name_keys) != len(set(name_keys)):
        raise AssertionError("duplicate identity/hash-domain name")

    shapes = []
    for schema in freeze["schemas"]:
        shapes.append(
            {
                "consumer_owners": schema["consumers"],
                "id": schema["id"],
                "kind": "strict_root",
                "primary_owner": schema["producer"],
                "primary_test_owner": owners[schema["producer"]],
            }
        )
    for record in freeze["schema_type_system"]["nested_records"]:
        shapes.append(
            {
                "consumer_owners": [],
                "id": record["id"],
                "kind": "strict_nested_record",
                "primary_owner": record["producer"],
                "primary_test_owner": owners[record["producer"]],
            }
        )
    for union in freeze["schema_type_system"]["tagged_unions"]:
        shapes.append(
            {
                "consumer_owners": [],
                "id": union["id"],
                "kind": "closed_tagged_union",
                "primary_owner": union["producer"],
                "primary_test_owner": owners[union["producer"]],
            }
        )
    for variant in freeze["expression_union"]["variants"]:
        item = "CSHARP-03-T06-W01"
        shapes.append(
            {
                "consumer_owners": [],
                "id": f"{freeze['expression_union']['schema']}#{variant['tag']}",
                "kind": "contract_expression_variant",
                "primary_owner": item,
                "primary_test_owner": owners[item],
            }
        )
    shapes.sort(key=lambda row: (row["kind"], row["id"]))

    limits = []
    for limit in freeze["limits"]["practical"]:
        item = limit["implementation_owner"]
        limits.append(
            {
                "disposition": "practical",
                "id": limit["id"],
                "primary_owner": item,
                "primary_test_owner": owners[item],
            }
        )
    for limit in freeze["limits"]["retained_scalar_v0"]:
        item = "CSHARP-03-T02-W08"
        limits.append(
            {
                "disposition": "retained_scalar_v0",
                "id": limit["id"],
                "primary_owner": item,
                "primary_test_owner": owners[item],
            }
        )
    limits.sort(key=lambda row: (row["disposition"], row["id"]))

    diagnostic_item = "CSHARP-03-T02-W08"
    diagnostics = [
        {
            "family": family,
            "primary_owner": diagnostic_item,
            "primary_test_owner": owners[diagnostic_item],
        }
        for family in freeze["diagnostics"]["families"]
    ]
    return {
        "diagnostics": diagnostics,
        "limits": limits,
        "names": names,
        "shapes": shapes,
    }


def id_hash(ids: list[str]) -> str:
    return sha(canonical(ids))


def upgrade_observation_sets() -> list[dict[str, Any]]:
    definitions = [
        (
            "CSHARP-03-T01-W04",
            "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
            "shape_index",
            "shape_id",
        ),
        (
            "CSHARP-03-T01-W05",
            "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
            "upgrade_mutations",
            "mutation_id",
        ),
        (
            "CSHARP-03-T01-W06",
            "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
            "upgrade_mutations",
            "mutation_id",
        ),
        (
            "CSHARP-03-T01-W07",
            "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
            "upgrade_mutations",
            "mutation_id",
        ),
    ]
    result = []
    for item, path, field, id_field in definitions:
        value = read_json(path)
        ids = sorted(row[id_field] for row in value[field])
        if len(ids) != len(set(ids)):
            raise AssertionError(f"duplicate upgrade ID in {path}")
        result.append(
            {
                "case_count": len(value[field]),
                "ids_sha256": id_hash(ids),
                "path": path,
                "source_field": field,
                "work_item": item,
            }
        )
    return result


def upgrade_matrix(owners: dict[str, str]) -> dict[str, Any]:
    rows = []
    seen_forms: set[str] = set()
    for identifier, forms, diagnostic, item, evidence in EXCLUDED_SOURCE_FAMILIES:
        if forms != sorted(forms):
            forms = sorted(forms)
        overlap = seen_forms.intersection(forms)
        if overlap:
            raise AssertionError(f"duplicate excluded source forms: {sorted(overlap)}")
        seen_forms.update(forms)
        rows.append(
            {
                "current_disposition": "reject_before_VIR_without_partial_artifacts",
                "diagnostic_family": diagnostic,
                "evidence_path": evidence,
                "future_profile_required": True,
                "id": identifier,
                "positive_vectors": "forbidden",
                "primary_test_owner": owners[item],
                "rejection_owner": item,
                "source_forms_or_claims": forms,
            }
        )
    return {
        "excluded_families": rows,
        "future_change_rule": "Any positive admission requires a new semantic-profile identity, regenerated specifications and vectors, and an atomic whole-release gate; mpk.csharp.practical.v1 remains unchanged.",
        "nullable_exception": "Only exact value-type T? source syntax is admitted; explicit System.Nullable<T>, open types, and unsupported payloads reject.",
        "observation_sets": upgrade_observation_sets(),
    }


def incorporated_design() -> dict[str, Any]:
    text = raw(DESIGN).decode("utf-8")
    start_heading = "## 6. Source closure and declaration model"
    end_heading = "## 24. Implementation stages and gates"
    start = text.index(start_heading)
    end = text.index(end_heading)
    projection = text[start:end].encode("utf-8")
    return {
        "end_before_heading": end_heading,
        "path": DESIGN,
        "raw_projection_sha256": sha(projection),
        "role": "incorporated_detailed_profile_rules",
        "size_bytes": len(projection),
        "start_heading": start_heading,
    }


def package() -> dict[str, Any]:
    freeze = read_json(W09_FREEZE)
    private_vectors = read_json(W09_VECTORS)
    owners = ledger_owners()
    contracts = task_contracts()
    if freeze["content_sha256"] != private_vectors["freeze_content_sha256"]:
        raise AssertionError("W09 freeze/vector linkage mismatch")
    vector_rows = private_vectors["vectors"]
    vector_ids = [row["id"] for row in vector_rows]
    if vector_ids != sorted(vector_ids) or len(vector_ids) != len(set(vector_ids)):
        raise AssertionError("W09 vectors must be sorted and unique")
    downstream = downstream_owners(owners, contracts)
    owner_by_item = {row["work_item"]: row["primary_test_owner"] for row in downstream}
    for row in vector_rows:
        item = row["implementation_owner"]
        if owner_by_item.get(item) != row["production_test_owner"]:
            raise AssertionError(f"vector owner mismatch: {row['id']}")

    evidence = [file_record(path, role, item) for path, item, role in EVIDENCE_RECORDS]
    evidence_items = {record["work_item"] for record in evidence}
    if evidence_items != {f"CSHARP-03-T01-W{index:02}" for index in range(1, 10)}:
        raise AssertionError("canonical evidence does not cover W01-W09")

    specifications = [
        file_record(PROFILE_SPEC, "normative_profile_specification"),
        file_record(SHARED_SPEC, "normative_successor_shared_artifact_specification"),
        file_record(FOUNDATION_SPEC, "normative_foundation_specification"),
    ]
    inventory_raw = sha(raw(INVENTORY))
    if freeze["ownership"]["inventory_raw_sha256"] != inventory_raw:
        raise AssertionError("W09 inventory binding drift")

    return {
        "canonical_evidence": evidence,
        "downstream_work_item_owners": downstream,
        "freeze_requirement_owners": freeze_requirement_owners(owners),
        "frozen_contract": freeze,
        "historical_inventory_extension": {
            "baseline_inventory_path": INVENTORY,
            "baseline_inventory_raw_sha256": inventory_raw,
            "owner": WORK_ITEM,
            "owner_test": f"{OWNER}#{WORK_ITEM}",
            "publication_paths": PUBLICATION_PATHS,
            "rule": "W02 search fingerprints remain the immutable pre-publication inventory; these exact normative publication consumers are excluded only from that historical fingerprint and are closed by the W10 package owner test.",
        },
        "incorporated_design": incorporated_design(),
        "name_owner_inventory": name_owner_inventory(freeze, owners),
        "owner_test": OWNER,
        "publication_generator": file_record(
            "develop/probes/csharp-03/profile_package.py", "deterministic_publication_generator"
        ),
        "release_gate": {
            "activation_owner": "CSHARP-03-T08-W10",
            "aggregate_gate_path": AGGREGATE_GATE,
            "aggregate_gate_predecessor_raw_sha256": AGGREGATE_GATE_PREDECESSOR_SHA256,
            "candidate_and_release_command": f"sudo ./{PRACTICAL_GATE}",
            "implementation_owner": "CSHARP-03-T07-W05",
            "invocation_owners": [
                "CSHARP-03-T07-W05",
                "CSHARP-03-T07-W06",
                "CSHARP-03-T08-W06",
                "CSHARP-03-T08-W09",
                "CSHARP-03-T08-W10",
            ],
            "post_activation_relation": "replace_and_retire_java_named_gate_atomically",
            "practical_gate_path": PRACTICAL_GATE,
            "pre_activation_gate_path": JAVA_GATE,
            "pre_activation_gate_raw_sha256": JAVA_GATE_PREDECESSOR_SHA256,
            "pre_activation_relation": "Java-named installed gate remains sole active release gate; practical gate path remains absent until its T07-W05 private implementation.",
            "receipt_owner": "CSHARP-03-T07-W06",
            "script_ownership_after_activation": "The practical release gate becomes the sole complete installed-release gate; check-all delegates to it exactly once and does not invoke the retired Java gate.",
        },
        "schema": SCHEMA,
        "semantic_profile": freeze["semantic_profile"],
        "source_w09": {
            "commit": W09_COMMIT,
            "freeze_content_hash_domain": freeze["content_hash_domain"],
            "freeze_content_sha256": freeze["content_sha256"],
            "freeze_path": W09_FREEZE,
            "freeze_raw_sha256": sha(raw(W09_FREEZE)),
            "freeze_schema": freeze["schema"],
            "vector_count": private_vectors["vector_count"],
            "vector_ids_sha256": private_vectors["vector_ids_sha256"],
            "vectors_path": W09_VECTORS,
            "vectors_raw_sha256": sha(raw(W09_VECTORS)),
            "vectors_schema": private_vectors["schema"],
        },
        "specification_members": specifications,
        "status": "normative_frozen_inactive",
        "upgrade_matrix": upgrade_matrix(owners),
        "vector_count": len(vector_rows),
        "vector_ids_sha256": id_hash(vector_ids),
        "vectors": vector_rows,
        "work_item": WORK_ITEM,
    }


def write_atomic(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=".w10-package-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.chmod(temporary, 0o644)
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


def main() -> None:
    if sys.argv[1:] not in (["--check"], ["--update"]):
        raise SystemExit("usage: profile_package.py --check | --update")
    value = package()
    data = output_bytes(value)
    target = ROOT / OUTPUT_PATH
    if sys.argv[1:] == ["--update"]:
        write_atomic(target, data)
        print(f"W10 package: {len(data)} bytes; {len(value['vectors'])} vectors")
        return
    if not target.is_file() or target.read_bytes() != data:
        raise SystemExit(f"W10 publication drift: {OUTPUT_PATH}")
    print(
        "W10 package: "
        f"{len(value['freeze_requirement_owners'])} freeze owners, "
        f"{len(value['downstream_work_item_owners'])} downstream owners, "
        f"{value['vector_count']} vectors"
    )


if __name__ == "__main__":
    main()
