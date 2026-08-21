#!/usr/bin/env python3
"""Compare the frozen Go/GIR audit baseline with its reviewed VIR mapping.

This is a development-only migration gate. It never imports GIR into a
production crate and must be archived or deleted by GO-VIR-02-T12.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Any


BASELINE_PATH = "develop/migrations/go-gir-semantic-baseline.json"
PROFILE_VECTOR_PATH = "develop/specs/vectors/go-vir-profile-v0.json"
VECTOR_MANIFEST_PATH = "develop/specs/vectors/manifest.json"
REPORT_JSON_PATH = "develop/migrations/go-gir-to-vir-report.json"
REPORT_MARKDOWN_PATH = "develop/migrations/go-gir-to-vir-report.md"
CORPUS_MANIFEST_PATH = "fixtures/vir-go/manifest.json"
STAGING_ROOT_PATH = "develop/migrations/go-vir-staging"
BASELINE_SCHEMA = "mpk.go_gir_semantic_baseline.v0"
PROFILE_VECTOR_SCHEMA = "mpk.go.vir_profile.conformance.v0"
REPORT_SCHEMA = "mpk.go_gir_to_vir_report.v0"
CORPUS_SCHEMA = "mpk.go_vir_corpus.v0"

MIGRATION_FIELDS = {
    "integrity",
    "coverage_policy",
    "identity_disposition",
    "historical_rule_map",
    "implementation_anchors",
    "frontend_contract",
    "obligation_kind_map",
    "corpora",
    "behavioral_anchors",
    "checker_anchors",
}

EXPECTED_OBLIGATION_KINDS = [
    ("precondition", "callee_precondition"),
    ("postcondition", "postcondition"),
    ("runtime_safety", "operation_safety"),
    ("loop_invariant_initial", "loop_initialization"),
    ("loop_invariant_preservation", "loop_preservation"),
    ("loop_exit", "loop_exit"),
    ("decreases", "loop_decreases"),
]

EXPECTED_GO_BASIC_REJECTIONS = {
    "map": "GO_SUBSET_MAPS",
    "pointer": "GO_SUBSET_POINTER",
    "generic": "GO_SUBSET_GENERICS",
    "string": "GO_SUBSET_STRING",
}

EXPECTED_PAYMENT_REJECTIONS = {
    "float": "GO_SUBSET_FLOAT",
    "map": "GO_SUBSET_MAPS",
    "pointer": "GO_SUBSET_POINTER",
    "missing_postconditions": "GO_CONTRACT_ENSURES",
}

EXPECTED_FOCUSED_REJECTIONS = [
    ("go-tools/go2gir/testdata/unsupported/map", "GO_SUBSET_MAPS", None),
    (
        "go-tools/go2gir/testdata/unsupported/goroutine",
        "GO_SUBSET_GOROUTINE",
        None,
    ),
    (
        "go-tools/go2gir/testdata/unsupported/generic",
        "GO_SUBSET_GENERICS",
        None,
    ),
    (
        "go-tools/go2gir/testdata/unsupported/pointer",
        "GO_SUBSET_POINTER",
        None,
    ),
    (
        "go-tools/go2gir/testdata/unsupported/untypedint",
        "GO_LOWER_UNTYPED_INTEGER",
        "untyped integer expression has no accepted fixed-width context",
    ),
    (
        "go-tools/go2gir/testdata/contract_malformed",
        "GO_CONTRACT_JSON",
        "invalid contract sidecar JSON",
    ),
    (
        "go-tools/go2gir/testdata/contract_unknown_function",
        "GO_CONTRACT_FUNCTION",
        'contract function "unknowncontract.Missing" does not resolve to an included Go function',
    ),
    (
        "go-tools/go2gir/testdata/contract_unsupported_operator",
        "GO_CONTRACT_OPERATOR",
        'unsupported contract expression operator "float_eq"',
    ),
]

EXPECTED_CONTRACT_RULES = [
    "requires and loops default to empty",
    "ensures must be present and nonempty",
    "modifies must be empty",
    "function identity must resolve exactly",
    "loop contracts retain invariant and optional decreases expressions",
    "unknown operators and malformed typed literals reject",
]

EXPECTED_RUNTIME_CHECKS = [
    {
        "operation": "signed division",
        "test": "emits_division_by_zero_obligation",
        "obligation_kind": "runtime_safety",
        "predicate": "divisor != 0",
        "old_member_count": 1,
        "new_check": "divisor_nonzero",
        "new_obligation_kind": "operation_safety",
        "predicate_components": 1,
    },
    {
        "operation": "shift with signed count",
        "test": "emits_shift_nonnegative_obligation_for_signed_shift_count",
        "obligation_kind": "runtime_safety",
        "predicate": "count >= 0",
        "old_member_count": 1,
        "new_check": "shift_count_nonnegative",
        "new_obligation_kind": "operation_safety",
        "predicate_components": 1,
    },
    {
        "operation": "fixed-array read with signed index",
        "test": "emits_signed_array_index_bounds_obligations",
        "obligation_kind": "runtime_safety",
        "predicate": "0 <= index && index < length",
        "old_member_count": 2,
        "new_check": "index_in_bounds",
        "new_obligation_kind": "operation_safety",
        "predicate_components": 2,
    },
]


class MigrationError(Exception):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "compare the frozen Go/GIR semantic baseline with the reviewed "
            "Go/VIR migration mapping"
        )
    )
    parser.add_argument("--baseline", default=BASELINE_PATH, help="GIR audit baseline")
    parser.add_argument(
        "--profile-vector",
        default=PROFILE_VECTOR_PATH,
        help="Go/VIR profile vector containing migration_baseline",
    )
    parser.add_argument(
        "--report-json", default=REPORT_JSON_PATH, help="checked JSON report path"
    )
    parser.add_argument(
        "--report-markdown",
        default=REPORT_MARKDOWN_PATH,
        help="checked derived Markdown report path",
    )
    parser.add_argument(
        "--corpus-manifest",
        default=CORPUS_MANIFEST_PATH,
        help="regenerated Go/VIR corpus manifest",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--check",
        action="store_true",
        help="fail unless both checked reports equal freshly derived output",
    )
    mode.add_argument(
        "--write",
        action="store_true",
        help="rewrite both development-only reports from validated inputs",
    )
    parser.add_argument(
        "--format",
        choices=("json", "markdown", "summary"),
        default="json",
        help="stdout format when neither --check nor --write is selected",
    )
    args = parser.parse_args()

    try:
        repo_root = Path(__file__).resolve().parent.parent
        baseline_path = resolve_path(repo_root, args.baseline)
        vector_path = resolve_path(repo_root, args.profile_vector)
        report_json_path = resolve_path(repo_root, args.report_json)
        report_markdown_path = resolve_path(repo_root, args.report_markdown)
        corpus_manifest_path = resolve_path(repo_root, args.corpus_manifest)
        baseline_bytes, baseline = read_json_object(baseline_path)
        vector_bytes, vector = read_json_object(vector_path)
        corpus_manifest_bytes, corpus_manifest = read_json_object(corpus_manifest_path)
        report = compare(
            repo_root,
            baseline_path,
            baseline_bytes,
            baseline,
            vector_path,
            vector_bytes,
            vector,
            corpus_manifest_path,
            corpus_manifest_bytes,
            corpus_manifest,
        )
        json_output = render_json(report)
        markdown_output = render_markdown(report)

        if args.check:
            check_report(report_json_path, json_output)
            check_report(report_markdown_path, markdown_output)
            print(
                "Go GIR-to-VIR migration check passed: "
                f"{report['summary']['reviewed_disposition_count']} dispositions, "
                "0 unexplained differences"
            )
        elif args.write:
            write_report(report_json_path, json_output)
            write_report(report_markdown_path, markdown_output)
            print(f"wrote {display_path(repo_root, report_json_path)}")
            print(f"wrote {display_path(repo_root, report_markdown_path)}")
        elif args.format == "json":
            sys.stdout.write(json_output)
        elif args.format == "markdown":
            sys.stdout.write(markdown_output)
        else:
            print(
                f"{report['status']}: "
                f"{report['summary']['baseline_leaf_count']} baseline leaves, "
                f"{report['summary']['reviewed_disposition_count']} dispositions, "
                "0 findings"
            )
        return 0
    except MigrationError as error:
        print(f"{error.code}: {error.message}", file=sys.stderr)
        return 1


def compare(
    repo_root: Path,
    baseline_path: Path,
    baseline_bytes: bytes,
    baseline: dict[str, Any],
    vector_path: Path,
    vector_bytes: bytes,
    vector: dict[str, Any],
    corpus_manifest_path: Path,
    corpus_manifest_bytes: bytes,
    corpus_manifest: dict[str, Any],
) -> dict[str, Any]:
    expect_equal(
        baseline.get("schema"),
        BASELINE_SCHEMA,
        "MIGRATION_INPUT",
        "baseline.schema",
    )
    expect_equal(
        vector.get("schema"),
        PROFILE_VECTOR_SCHEMA,
        "MIGRATION_INPUT",
        "profile vector schema",
    )
    expect_equal(
        vector.get("spec_profile"),
        "mpk.go.fixed.v0",
        "MIGRATION_INPUT",
        "profile vector spec_profile",
    )
    migration = require_object(
        vector.get("migration_baseline"), "profile vector migration_baseline"
    )
    require_exact_fields(
        migration, MIGRATION_FIELDS, "MIGRATION_INPUT", "migration_baseline"
    )
    baseline_digest = sha256(baseline_bytes)
    vector_digest = sha256(vector_bytes)
    validate_input_integrity(
        repo_root,
        baseline_path,
        baseline,
        baseline_digest,
        vector_path,
        vector_digest,
        migration,
    )
    fixture_integrity = validate_fixture_anchors(repo_root, baseline, migration)

    coverage_policy = validate_coverage_policy(baseline, migration)
    identity_changes = validate_identity(baseline, migration)
    historical = validate_historical_rules(migration)
    implementation_anchors = validate_implementation_anchors(migration)
    frontend_changes = validate_frontend_contract(baseline, migration)
    obligation_changes = validate_obligation_kinds(baseline, migration)
    source_inventory = validate_corpora(baseline, migration)
    semantics = validate_behavioral_anchors(baseline, migration)
    checker_verdicts = validate_checker_anchors(repo_root, baseline, migration)
    regenerated_corpus = validate_regenerated_corpus(
        repo_root, corpus_manifest_path, corpus_manifest_bytes, corpus_manifest
    )
    allowed_changes = [
        {
            "id": "schema",
            "old": "mpk.gir.v0 and GIR-era envelopes",
            "new": "mpk.vir.v0 and generic frontend envelopes",
            "disposition": "reviewed breaking replacement; no compatibility importer",
        },
        {
            "id": "identifier",
            "old": "GIR function and theorem strings",
            "new": "canonical import-path declaration IDs and stable VC member IDs",
            "disposition": "reviewed identity replacement; semantic ownership preserved",
        },
        {
            "id": "declaration_group",
            "old": "one theorem declaration per GIR obligation",
            "new": "contract and panic-free declarations containing ordered VC members",
            "disposition": "reviewed grouping change; every member remains assigned exactly once",
        },
        {
            "id": "foundation_name",
            "old": "Std.Go.Base.*",
            "new": "Std.Program.Base.*",
            "disposition": "reviewed zero-axiom checked-foundation rename",
        },
        {
            "id": "artifact_bytes_and_hashes",
            "old": "GIR, VC v0, skeleton, policy, and evidence bytes/hashes",
            "new": "regenerated VIR, VC v1, grouped skeleton, policy, and evidence bytes/hashes",
            "disposition": "audit anchors only; byte and hash equality is not required",
        },
    ]

    dispositions = (
        allowed_changes
        + identity_changes
        + frontend_changes
        + obligation_changes
        + [
            {
                "id": "obligation.kind_inventory",
                "baseline_pointer": "/obligation_model/kinds",
                "old": [old for old, _ in EXPECTED_OBLIGATION_KINDS],
                "new": [new for _, new in EXPECTED_OBLIGATION_KINDS],
                "disposition": "complete one-to-one reviewed obligation kind inventory",
            }
        ]
        + source_inventory["dispositions"]
        + semantics["dispositions"]
        + checker_verdicts["dispositions"]
    )
    disposition_ids = [item["id"] for item in dispositions]
    expect_equal(
        len(disposition_ids),
        len(set(disposition_ids)),
        "MIGRATION_COVERAGE",
        "reviewed disposition IDs must be unique",
    )
    coverage = validate_leaf_coverage(
        baseline, migration, set(disposition_ids)
    )

    return {
        "schema": REPORT_SCHEMA,
        "status": "equivalent_with_reviewed_changes",
        "lifecycle": {
            "development_only": True,
            "production_input": False,
            "release_artifact": False,
            "archive_or_delete_owner": "GO-VIR-02-T12",
        },
        "inputs": {
            "baseline": {
                "path": display_path(repo_root, baseline_path),
                "schema": baseline["schema"],
                "sha256": baseline_digest,
                "captured_from_revision": baseline["captured_from_revision"],
            },
            "profile_vector": {
                "path": display_path(repo_root, vector_path),
                "schema": vector["schema"],
                "sha256": vector_digest,
                "profile": vector["spec_profile"],
            },
            "regenerated_corpus": {
                "path": display_path(repo_root, corpus_manifest_path),
                "schema": corpus_manifest["schema"],
                "sha256": sha256(corpus_manifest_bytes),
            },
        },
        "summary": {
            "baseline_leaf_count": coverage["baseline_leaf_count"],
            "covered_leaf_count": coverage["covered_leaf_count"],
            "reviewed_disposition_count": len(dispositions),
            "historical_accepted_rule_count": historical["accepted_rule_count"],
            "historical_rejected_rule_count": historical["rejected_rule_count"],
            "accepted_source_case_count": source_inventory[
                "accepted_source_case_count"
            ],
            "rejected_source_case_count": source_inventory[
                "rejected_source_case_count"
            ],
            "canonical_function_identity_count": source_inventory[
                "canonical_function_identity_count"
            ],
            "runtime_check_count": len(semantics["runtime_checks"]),
            "obligation_kind_count": len(obligation_changes),
            "checker_anchor_count": len(checker_verdicts["anchors"]),
            "checked_fixture_file_count": fixture_integrity["checked_file_count"],
            "regenerated_artifact_count": regenerated_corpus["artifact_count"],
            "unexplained_difference_count": 0,
        },
        "coverage_policy": coverage_policy,
        "allowed_changes": allowed_changes,
        "identity_changes": identity_changes,
        "frontend_contract": frontend_changes,
        "obligation_kinds": obligation_changes,
        "historical_rules": historical,
        "implementation_anchors": implementation_anchors,
        "fixture_integrity": fixture_integrity,
        "source_inventory": {
            key: value
            for key, value in source_inventory.items()
            if key != "dispositions"
        },
        "semantics": {
            key: value for key, value in semantics.items() if key != "dispositions"
        },
        "checker_verdicts": checker_verdicts["anchors"],
        "regenerated_corpus": regenerated_corpus,
        "coverage": coverage,
        "reviewed_dispositions": dispositions,
        "findings": [],
    }


def validate_input_integrity(
    repo_root: Path,
    baseline_path: Path,
    baseline: dict[str, Any],
    baseline_digest: str,
    vector_path: Path,
    vector_digest: str,
    migration: dict[str, Any],
) -> None:
    integrity = require_object(migration["integrity"], "migration_baseline.integrity")
    require_exact_fields(
        integrity,
        {"path", "sha256", "schema", "captured_from_revision"},
        "MIGRATION_INPUT",
        "migration_baseline.integrity",
    )
    expect_equal(
        integrity["path"],
        BASELINE_PATH,
        "MIGRATION_BASELINE_INTEGRITY",
        "baseline logical path",
    )
    expect_equal(
        baseline_digest,
        integrity["sha256"],
        "MIGRATION_BASELINE_INTEGRITY",
        f"SHA-256 for {display_path(repo_root, baseline_path)}",
    )
    expect_equal(
        baseline["schema"],
        integrity["schema"],
        "MIGRATION_BASELINE_INTEGRITY",
        "baseline schema anchor",
    )
    expect_equal(
        baseline.get("captured_from_revision"),
        integrity["captured_from_revision"],
        "MIGRATION_BASELINE_INTEGRITY",
        "baseline captured revision",
    )

    if vector_path.resolve() == (repo_root / PROFILE_VECTOR_PATH).resolve():
        _, manifest = read_json_object(repo_root / VECTOR_MANIFEST_PATH)
        entries = require_array(manifest.get("vectors"), "vector manifest vectors")
        matches = [entry for entry in entries if entry.get("path") == PROFILE_VECTOR_PATH]
        expect_equal(
            len(matches),
            1,
            "MIGRATION_VECTOR_INTEGRITY",
            "profile vector manifest entry count",
        )
        expect_equal(
            vector_digest,
            matches[0].get("sha256"),
            "MIGRATION_VECTOR_INTEGRITY",
            "profile vector manifest SHA-256",
        )


def validate_coverage_policy(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> dict[str, Any]:
    policy = require_object(migration["coverage_policy"], "coverage_policy")
    require_exact_fields(
        policy,
        {"mode", "covered_root_members", "rule"},
        "MIGRATION_COVERAGE",
        "coverage_policy",
    )
    expect_equal(
        policy["mode"],
        "complete_deep_subtree",
        "MIGRATION_COVERAGE",
        "coverage policy mode",
    )
    roots = require_string_array(policy["covered_root_members"], "covered_root_members")
    expect_equal(
        set(roots),
        set(baseline),
        "MIGRATION_COVERAGE",
        "covered root members",
    )
    expect_equal(
        len(roots),
        len(set(roots)),
        "MIGRATION_COVERAGE",
        "covered root members must be unique",
    )
    return {"mode": policy["mode"], "rule": policy["rule"]}


def validate_fixture_anchors(
    repo_root: Path, baseline: dict[str, Any], migration: dict[str, Any]
) -> dict[str, Any]:
    checked: list[dict[str, str]] = []

    def check(path: str, digest: str) -> None:
        verify_fixture_digest(repo_root, path, digest)
        checked.append({"path": path, "sha256": digest})

    historical = migration["historical_rule_map"]["source_spec"]
    check(historical["path"], historical["sha256"])
    for anchor in migration["implementation_anchors"]:
        check(anchor["path"], anchor["sha256"])

    corpora = baseline["corpora"]
    alpha = corpora["go_alpha"]
    check(alpha["manifest"]["path"], alpha["manifest"]["sha256"])
    check(alpha["module_path"], alpha["module_sha256"])
    for group in alpha["groups"]:
        check(group["source"], group["source_sha256"])
    alpha_vc = alpha["vc_fixture"]
    check(alpha_vc["manifest_path"], alpha_vc["manifest_sha256"])
    check(alpha_vc["vc"]["path"], alpha_vc["vc"]["sha256"])
    check(alpha_vc["skeleton"]["path"], alpha_vc["skeleton"]["sha256"])

    basic = corpora["go_basic"]
    check(basic["manifest_path"], basic["manifest_sha256"])
    check(basic["module_path"], basic["module_sha256"])
    for case in basic["positive"] + basic["negative"]:
        directory = repo_root / "fixtures/go-basic" / case["path"]
        try:
            sources = sorted(directory.glob("*.go"))
        except OSError as error:
            raise MigrationError(
                "MIGRATION_FIXTURE_INTEGRITY",
                f"cannot enumerate Go fixture directory {directory}: {error}",
            ) from error
        expect_equal(
            len(sources),
            1,
            "MIGRATION_FIXTURE_INTEGRITY",
            f"Go source count in {display_path(repo_root, directory)}",
        )
        check(display_path(repo_root, sources[0]), case["source_sha256"])

    payment = corpora["payment_policies"]
    check(payment["manifest_path"], payment["manifest_sha256"])
    for case in payment["positive"]:
        for path_field, digest_field in (
            ("module_path", "module_sha256"),
            ("source_path", "source_sha256"),
            ("contract_path", "contract_sha256"),
            ("gir_path", "gir_file_sha256"),
            ("vc_path", "vc_file_sha256"),
            ("skeleton_path", "skeleton_file_sha256"),
        ):
            check(case[path_field], case[digest_field])
    for case in payment["negative"]:
        root = Path("examples/payment_policies") / case["path"]
        for name, digest in case["files"].items():
            check((root / name).as_posix(), digest)

    checker = baseline["checker_baseline"]
    release = checker["release_report"]
    check(release["path"], release["sha256"])
    certificate_path = release["certificate"]["path"]
    require_regular_file(repo_root, certificate_path)
    payment_evidence = checker["payment_reserve_theory_evidence"]
    check(payment_evidence["path"], payment_evidence["sha256"])

    paths = [item["path"] for item in checked]
    require_unique(paths, "checked fixture paths")
    return {
        "checked_file_count": len(checked),
        "checked_files": sorted(checked, key=lambda item: item["path"]),
        "certificate_fixture_present": certificate_path,
    }


def validate_identity(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> list[dict[str, Any]]:
    mapping = require_object(migration["identity_disposition"], "identity_disposition")
    require_exact_fields(
        mapping,
        {"source_profile", "source_ir_schema", "purpose", "hash_policy"},
        "MIGRATION_INPUT",
        "identity_disposition",
    )
    expected = {
        "source_profile": "replaced by normative mpk.go.fixed.v0",
        "source_ir_schema": "mpk.gir.v0 rejects after cutover; regenerated artifact is mpk.vir.v0",
        "purpose": "remains audit-only and is never a runtime input",
        "hash_policy": "all old hashes are audit anchors; byte equality is not required and every semantic difference requires reviewed disposition",
    }
    expect_equal(
        mapping,
        expected,
        "MIGRATION_INPUT",
        "identity disposition",
    )
    return [
        {
            "id": "identity.captured_revision",
            "baseline_pointer": "/captured_from_revision",
            "old": baseline["captured_from_revision"],
            "new": baseline["captured_from_revision"],
            "disposition": "immutable pre-cutover audit revision preserved",
        },
    ] + [
        {
            "id": f"identity.{name}",
            "baseline_pointer": f"/{name}",
            "old": baseline[name],
            "new": disposition,
            "disposition": "reviewed identity disposition",
        }
        for name, disposition in mapping.items()
    ]


def validate_historical_rules(migration: dict[str, Any]) -> dict[str, Any]:
    mapping = require_object(migration["historical_rule_map"], "historical_rule_map")
    require_exact_fields(
        mapping,
        {"source_spec", "accepted", "rejected"},
        "MIGRATION_INPUT",
        "historical_rule_map",
    )
    accepted = require_array(mapping["accepted"], "historical accepted rules")
    rejected = require_array(mapping["rejected"], "historical rejected rules")
    expect_equal(
        len(accepted), 23, "MIGRATION_SOURCE_ACCEPTANCE", "accepted rule count"
    )
    expect_equal(
        len(rejected), 21, "MIGRATION_NEGATIVE_REJECTION", "rejected rule count"
    )
    require_unique([item.get("rule") for item in accepted], "accepted rules")
    require_unique([item.get("rule") for item in rejected], "rejected rules")
    for index, item in enumerate(accepted):
        require_exact_fields(
            require_object(item, f"accepted[{index}]"),
            {"rule", "new"},
            "MIGRATION_SOURCE_ACCEPTANCE",
            f"historical accepted[{index}]",
        )
        require_nonempty(item["rule"], f"historical accepted[{index}].rule")
        require_nonempty(item["new"], f"historical accepted[{index}].new")
    for index, item in enumerate(rejected):
        item = require_object(item, f"rejected[{index}]")
        allowed = {"rule", "code"} if "code" in item else {"rule", "codes"}
        require_exact_fields(
            item,
            allowed,
            "MIGRATION_NEGATIVE_REJECTION",
            f"historical rejected[{index}]",
        )
        codes = [item["code"]] if "code" in item else item["codes"]
        require_nonempty(item["rule"], f"historical rejected[{index}].rule")
        require_string_array(codes, f"historical rejected[{index}] codes")
    source_spec = require_object(mapping["source_spec"], "historical source_spec")
    require_exact_fields(
        source_spec,
        {"path", "sha256"},
        "MIGRATION_INPUT",
        "historical source_spec",
    )
    return {
        "source_spec": source_spec,
        "accepted_rule_count": len(accepted),
        "rejected_rule_count": len(rejected),
        "accepted_rules": accepted,
        "rejected_rules": rejected,
    }


def validate_implementation_anchors(migration: dict[str, Any]) -> list[dict[str, Any]]:
    anchors = require_array(migration["implementation_anchors"], "implementation anchors")
    expect_equal(len(anchors), 6, "MIGRATION_INPUT", "implementation anchor count")
    paths: list[str] = []
    for index, anchor in enumerate(anchors):
        anchor = require_object(anchor, f"implementation_anchors[{index}]")
        require_exact_fields(
            anchor,
            {"path", "sha256", "disposition"},
            "MIGRATION_INPUT",
            f"implementation_anchors[{index}]",
        )
        require_nonempty(anchor["disposition"], f"implementation_anchors[{index}]")
        paths.append(anchor["path"])
    require_unique(paths, "implementation anchor paths")
    return anchors


def validate_frontend_contract(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> list[dict[str, Any]]:
    old = require_object(baseline["frontend_contract"], "baseline frontend_contract")
    mappings = require_array(migration["frontend_contract"], "frontend_contract mapping")
    expect_equal(
        len(mappings), len(old), "MIGRATION_FRONTEND_CONTRACT", "mapping count"
    )
    seen: set[str] = set()
    result = []
    for index, item in enumerate(mappings):
        item = require_object(item, f"frontend_contract[{index}]")
        require_exact_fields(
            item,
            {"baseline_pointer", "old", "new", "disposition"},
            "MIGRATION_FRONTEND_CONTRACT",
            f"frontend_contract[{index}]",
        )
        pointer = item["baseline_pointer"]
        seen.add(pointer)
        key = pointer.removeprefix("/frontend_contract/")
        expect_true(
            key in old and pointer == f"/frontend_contract/{key}",
            "MIGRATION_FRONTEND_CONTRACT",
            f"invalid frontend baseline pointer {pointer!r}",
        )
        if key == "canonical_binary":
            expect_equal(
                item["old"],
                "MPK_GIR_V0 framing and hash",
                "MIGRATION_FRONTEND_CONTRACT",
                "canonical binary old summary",
            )
            expect_true(
                item["disposition"] == "no byte or hash equality",
                "MIGRATION_FRONTEND_CONTRACT",
                "canonical binary change must remain audit-only",
            )
        else:
            expect_equal(
                item["old"],
                old[key],
                "MIGRATION_FRONTEND_CONTRACT",
                f"old frontend value at {pointer}",
            )
        result.append({"id": f"frontend.{key}", **item})
    expect_equal(
        seen,
        {f"/frontend_contract/{key}" for key in old},
        "MIGRATION_FRONTEND_CONTRACT",
        "frontend mapping pointers",
    )
    return result


def validate_obligation_kinds(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> list[dict[str, Any]]:
    old_model = require_object(baseline["obligation_model"], "obligation_model")
    old_kinds = require_string_array(old_model["kinds"], "baseline obligation kinds")
    theorem_intent = require_object(old_model["theorem_intent"], "theorem_intent")
    expect_equal(
        set(theorem_intent),
        set(old_kinds),
        "MIGRATION_OBLIGATION_KIND",
        "old theorem intent kinds",
    )
    mappings = require_array(migration["obligation_kind_map"], "obligation kind map")
    pairs = [(item.get("old"), item.get("new")) for item in mappings]
    expect_equal(
        pairs,
        EXPECTED_OBLIGATION_KINDS,
        "MIGRATION_OBLIGATION_KIND",
        "complete old/new obligation kind mapping",
    )
    expect_equal(
        old_kinds,
        [old for old, _ in EXPECTED_OBLIGATION_KINDS],
        "MIGRATION_OBLIGATION_KIND",
        "baseline obligation kinds",
    )
    require_unique([new for _, new in pairs], "new obligation kinds")
    result = []
    for index, item in enumerate(mappings):
        item = require_object(item, f"obligation_kind_map[{index}]")
        require_exact_fields(
            item,
            {"old", "new", "intent"},
            "MIGRATION_OBLIGATION_KIND",
            f"obligation_kind_map[{index}]",
        )
        require_nonempty(item["intent"], f"obligation_kind_map[{index}].intent")
        result.append(
            {
                "id": f"obligation.{item['old']}",
                "baseline_pointer": f"/obligation_model/theorem_intent/{item['old']}",
                "old": item["old"],
                "new": item["new"],
                "old_intent": theorem_intent[item["old"]],
                "disposition": item["intent"],
            }
        )
    return result


def validate_corpora(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> dict[str, Any]:
    old = require_object(baseline["corpora"], "baseline corpora")
    new = require_object(migration["corpora"], "migration corpora")
    require_exact_fields(
        new,
        {
            "go_alpha",
            "go_basic",
            "payment_policies",
            "focused_frontend_negative_cases",
        },
        "MIGRATION_COVERAGE",
        "migration corpora",
    )
    alpha = validate_go_alpha(old["go_alpha"], new["go_alpha"])
    basic = validate_go_basic(old["go_basic"], new["go_basic"])
    payment = validate_payment_policies(
        old["payment_policies"], new["payment_policies"]
    )
    focused = validate_focused_rejections(
        old["focused_frontend_negative_cases"],
        new["focused_frontend_negative_cases"],
    )
    return {
        "accepted_source_case_count": alpha["function_count"]
        + len(basic["accepted"])
        + len(payment["accepted"]),
        "rejected_source_case_count": len(basic["rejected"])
        + len(payment["rejected"])
        + len(focused),
        "canonical_function_identity_count": len(alpha["function_identities"])
        + len(payment["accepted"]),
        "go_alpha": alpha,
        "go_basic": basic,
        "payment_policies": payment,
        "focused_rejections": focused,
        "dispositions": alpha["dispositions"]
        + basic["dispositions"]
        + payment["dispositions"]
        + [
            {
                "id": f"source.focused_rejection.{item['baseline_index']}",
                "baseline_pointer": f"/corpora/focused_frontend_negative_cases/{item['baseline_index']}",
                "old": item["old_reason"],
                "new": item["new_code"],
                "disposition": "reviewed deterministic rejection classification",
            }
            for item in focused
        ],
    }


def validate_go_alpha(old_value: Any, new_value: Any) -> dict[str, Any]:
    old = require_object(old_value, "baseline go_alpha")
    new = require_object(new_value, "migration go_alpha")
    require_exact_fields(
        new,
        {
            "baseline_pointer",
            "manifest_path",
            "manifest_sha256",
            "module_path",
            "module_sha256",
            "old_expected_status",
            "new_expected_status",
            "function_count",
            "groups",
            "vc_fixture",
        },
        "MIGRATION_COVERAGE",
        "migration go_alpha",
    )
    old_manifest = require_object(old["manifest"], "go_alpha manifest")
    simple = {
        "manifest_path": old_manifest["path"],
        "manifest_sha256": old_manifest["sha256"],
        "module_path": old["module_path"],
        "module_sha256": old["module_sha256"],
        "old_expected_status": old["expected_cli_status"],
        "function_count": old_manifest["function_count"],
    }
    for field, expected in simple.items():
        expect_equal(
            new.get(field),
            expected,
            "MIGRATION_SOURCE_ACCEPTANCE",
            f"go_alpha.{field}",
        )
    expect_equal(
        new.get("baseline_pointer"),
        "/corpora/go_alpha",
        "MIGRATION_COVERAGE",
        "go_alpha baseline pointer",
    )
    expect_equal(
        old["expected_outcome"],
        "accepted",
        "MIGRATION_SOURCE_ACCEPTANCE",
        "go_alpha expected outcome",
    )
    expect_equal(
        new.get("new_expected_status"),
        "ir-lowered",
        "MIGRATION_SOURCE_ACCEPTANCE",
        "go_alpha VIR status",
    )
    old_groups = require_array(old["groups"], "baseline go_alpha groups")
    new_groups = require_array(new["groups"], "migration go_alpha groups")
    expect_equal(
        len(new_groups),
        len(old_groups),
        "MIGRATION_FUNCTION_IDENTITY",
        "go_alpha group count",
    )
    module = "github.com/finitefield-org/mpk/fixtures/go-alpha"
    identities = []
    groups = []
    dispositions = [
        {
            "id": "source.go_alpha.envelope",
            "baseline_pointer": "/corpora/go_alpha",
            "old": "frozen corpus manifest, module, and gir-lowered status",
            "new": "same source inventory with ir-lowered status",
            "disposition": "corpus identity and acceptance preserved; frontend status renamed",
        }
    ]
    function_names: list[str] = []
    semantic_replacements = {
        "signed shift counts require runtime-safety coverage": "signed shift counts require operation-safety coverage",
        "fixed-array reads require bounds coverage": "fixed-array reads require complete bounds coverage",
    }
    for index, (old_group, new_group) in enumerate(zip(old_groups, new_groups)):
        expected_preservation = [
            semantic_replacements.get(item, item) for item in old_group["semantics"]
        ]
        require_exact_fields(
            require_object(new_group, f"go_alpha groups[{index}]"),
            {
                "name",
                "source",
                "source_sha256",
                "function_count",
                "functions",
                "preservation",
            },
            "MIGRATION_COVERAGE",
            f"go_alpha groups[{index}]",
        )
        for field in ("name", "source", "source_sha256", "function_count", "functions"):
            expect_equal(
                new_group.get(field),
                old_group[field],
                "MIGRATION_FUNCTION_IDENTITY",
                f"go_alpha group {index} {field}",
            )
        expect_equal(
            new_group.get("preservation"),
            expected_preservation,
            "MIGRATION_OPERATION_SEMANTICS",
            f"go_alpha group {old_group['name']} operation semantics",
        )
        expect_equal(
            len(old_group["functions"]),
            old_group["function_count"],
            "MIGRATION_FUNCTION_IDENTITY",
            f"go_alpha group {old_group['name']} function count",
        )
        for function in old_group["functions"]:
            function_names.append(function)
            identities.append(
                {
                    "old_group": old_group["name"],
                    "old_name": function,
                    "new_function_id": f"{module}/{old_group['name']}.{function}",
                }
            )
        groups.append(
            {
                "name": old_group["name"],
                "function_count": old_group["function_count"],
                "preserved_operation_semantics": expected_preservation,
            }
        )
        dispositions.append(
            {
                "id": f"source.go_alpha.{old_group['name']}",
                "baseline_pointer": f"/corpora/go_alpha/groups/{index}",
                "old": f"{old_group['function_count']} GIR function identities",
                "new": f"{old_group['function_count']} canonical VIR function identities",
                "disposition": "complete function inventory and operation intent preserved",
            }
        )
    require_unique(function_names, "go_alpha function names")
    expect_equal(
        len(identities),
        old_manifest["function_count"],
        "MIGRATION_FUNCTION_IDENTITY",
        "go_alpha total function count",
    )
    validate_alpha_vc_fixture(old["vc_fixture"], new["vc_fixture"])
    dispositions.append(
        {
            "id": "source.go_alpha.vc_fixture",
            "baseline_pointer": "/corpora/go_alpha/vc_fixture",
            "old": f"{old['vc_fixture']['obligation_count']} GIR-era obligations",
            "new": "regenerated VC v1 members and grouped declarations",
            "disposition": new["vc_fixture"]["disposition"],
        }
    )
    return {
        "function_count": len(identities),
        "groups": groups,
        "function_identities": identities,
        "vc_fixture": {
            "old_obligation_count": old["vc_fixture"]["obligation_count"],
            "preserved_intent": "postcondition and branch-path semantics",
            "byte_or_hash_equality_required": False,
        },
        "dispositions": dispositions,
    }


def validate_alpha_vc_fixture(old_value: Any, new_value: Any) -> None:
    old = require_object(old_value, "baseline go_alpha vc_fixture")
    new = require_object(new_value, "migration go_alpha vc_fixture")
    expected = {
        "baseline_pointer": "/corpora/go_alpha/vc_fixture",
        "manifest_path": old["manifest_path"],
        "manifest_sha256": old["manifest_sha256"],
        "old_source_gir_hash": old["source_gir_hash"],
        "branch_case_count": old["branch_case_count"],
        "postconditions_per_case": old["postconditions_per_case"],
        "old_obligation_count": old["obligation_count"],
        "old_obligation_kinds": old["obligation_kinds"],
        "old_first_obligation": old["first_obligation"],
        "old_last_obligation": old["last_obligation"],
        "old_vc_path": old["vc"]["path"],
        "old_vc_sha256": old["vc"]["sha256"],
        "old_skeleton_path": old["skeleton"]["path"],
        "old_skeleton_sha256": old["skeleton"]["sha256"],
        "old_theorem_declaration_count": old["skeleton"][
            "theorem_declaration_count"
        ],
        "old_checker_verdict": old["checker_verdict"],
    }
    require_exact_fields(
        new,
        set(expected) | {"disposition"},
        "MIGRATION_COVERAGE",
        "go_alpha vc_fixture",
    )
    for field, value in expected.items():
        expect_equal(
            new.get(field),
            value,
            "MIGRATION_PROPERTY_INTENT",
            f"go_alpha.vc_fixture.{field}",
        )
    disposition = require_nonempty(new.get("disposition"), "go_alpha VC disposition")
    for phrase in (
        "no hash",
        "theorem-name",
        "obligation-count equality",
        "postcondition intent",
        "branch-path semantics",
    ):
        expect_true(
            phrase in disposition,
            "MIGRATION_PROPERTY_INTENT",
            f"go_alpha VC disposition must name {phrase!r}",
        )


def validate_go_basic(old_value: Any, new_value: Any) -> dict[str, Any]:
    old = require_object(old_value, "baseline go_basic")
    new = require_object(new_value, "migration go_basic")
    require_exact_fields(
        new,
        {
            "baseline_pointer",
            "manifest_path",
            "manifest_sha256",
            "module_path",
            "module_sha256",
            "positive",
            "negative",
        },
        "MIGRATION_COVERAGE",
        "migration go_basic",
    )
    for field in ("manifest_path", "manifest_sha256", "module_path", "module_sha256"):
        expect_equal(
            new.get(field),
            old[field],
            "MIGRATION_SOURCE_ACCEPTANCE",
            f"go_basic.{field}",
        )
    expect_equal(
        new.get("baseline_pointer"),
        "/corpora/go_basic",
        "MIGRATION_COVERAGE",
        "go_basic baseline pointer",
    )
    accepted = []
    old_positive = require_array(old["positive"], "go_basic positive")
    new_positive = require_array(new["positive"], "go_basic positive mapping")
    expect_equal(
        len(new_positive),
        len(old_positive),
        "MIGRATION_SOURCE_ACCEPTANCE",
        "go_basic positive count",
    )
    dispositions = [
        {
            "id": "source.go_basic.envelope",
            "baseline_pointer": "/corpora/go_basic",
            "old": "frozen corpus manifest and module",
            "new": "same source corpus under the generic frontend",
            "disposition": "manifest and module identities preserved",
        }
    ]
    for index, (old_case, new_case) in enumerate(zip(old_positive, new_positive)):
        require_exact_fields(
            require_object(new_case, f"go_basic positive[{index}]"),
            {"name", "path", "source_sha256", "new_expected_status", "preserves"},
            "MIGRATION_COVERAGE",
            f"go_basic positive[{index}]",
        )
        for field in ("name", "path", "source_sha256"):
            expect_equal(
                new_case.get(field),
                old_case[field],
                "MIGRATION_SOURCE_ACCEPTANCE",
                f"go_basic positive[{index}].{field}",
            )
        expect_equal(
            old_case["expected"],
            "accepted",
            "MIGRATION_SOURCE_ACCEPTANCE",
            f"go_basic positive[{index}] baseline outcome",
        )
        expect_equal(
            new_case.get("new_expected_status"),
            "ir-lowered",
            "MIGRATION_SOURCE_ACCEPTANCE",
            f"go_basic positive[{index}] new status",
        )
        expect_equal(
            new_case.get("preserves"),
            old_case["covers"],
            "MIGRATION_OPERATION_SEMANTICS",
            f"go_basic positive[{index}] semantics",
        )
        accepted.append(
            {
                "name": old_case["name"],
                "path": old_case["path"],
                "old_outcome": old_case["expected"],
                "new_status": new_case["new_expected_status"],
            }
        )
        dispositions.append(
            {
                "id": f"source.go_basic.positive.{old_case['name']}",
                "baseline_pointer": f"/corpora/go_basic/positive/{index}",
                "old": "accepted/gir-lowered",
                "new": "accepted/ir-lowered",
                "disposition": "source acceptance and operation intent preserved",
            }
        )
    rejected = []
    old_negative = require_array(old["negative"], "go_basic negative")
    new_negative = require_array(new["negative"], "go_basic negative mapping")
    expect_equal(
        len(new_negative),
        len(old_negative),
        "MIGRATION_NEGATIVE_REJECTION",
        "go_basic negative count",
    )
    for index, (old_case, new_case) in enumerate(zip(old_negative, new_negative)):
        require_exact_fields(
            require_object(new_case, f"go_basic negative[{index}]"),
            {"name", "path", "source_sha256", "code"},
            "MIGRATION_COVERAGE",
            f"go_basic negative[{index}]",
        )
        for field in ("name", "path", "source_sha256"):
            expect_equal(
                new_case.get(field),
                old_case[field],
                "MIGRATION_NEGATIVE_REJECTION",
                f"go_basic negative[{index}].{field}",
            )
        expected_code = EXPECTED_GO_BASIC_REJECTIONS.get(old_case["name"])
        expect_equal(
            new_case.get("code"),
            expected_code,
            "MIGRATION_NEGATIVE_REJECTION",
            f"go_basic negative {old_case['name']} rejection class",
        )
        rejected.append(
            {
                "name": old_case["name"],
                "path": old_case["path"],
                "old_reason": old_case["reason"],
                "new_code": new_case["code"],
            }
        )
        dispositions.append(
            {
                "id": f"source.go_basic.negative.{old_case['name']}",
                "baseline_pointer": f"/corpora/go_basic/negative/{index}",
                "old": old_case["reason"],
                "new": new_case["code"],
                "disposition": "reviewed deterministic rejection classification",
            }
        )
    return {"accepted": accepted, "rejected": rejected, "dispositions": dispositions}


def validate_payment_policies(old_value: Any, new_value: Any) -> dict[str, Any]:
    old = require_object(old_value, "baseline payment_policies")
    new = require_object(new_value, "migration payment_policies")
    require_exact_fields(
        new,
        {
            "baseline_pointer",
            "manifest_path",
            "manifest_sha256",
            "shared_preservation",
            "positive",
            "negative",
        },
        "MIGRATION_COVERAGE",
        "migration payment_policies",
    )
    for field in ("manifest_path", "manifest_sha256"):
        expect_equal(
            new.get(field),
            old[field],
            "MIGRATION_PROPERTY_INTENT",
            f"payment_policies.{field}",
        )
    expect_equal(
        new.get("baseline_pointer"),
        "/corpora/payment_policies",
        "MIGRATION_COVERAGE",
        "payment baseline pointer",
    )
    old_shared = require_object(old["shared_expectation"], "payment shared expectation")
    new_shared = require_object(new["shared_preservation"], "payment shared preservation")
    require_exact_fields(
        new_shared,
        {
            "frontend_outcome",
            "new_frontend_status",
            "requires_count",
            "ensures_count",
            "postcondition_intent_count",
            "new_obligation_kind",
            "linear_theory_goal_count",
            "true_or_opaque_goal_count",
            "opaque_or_true_goal_count",
            "clause_and_branch_order",
            "classification_evidence_label",
            "classification_property_status_until_program_certificate",
            "old_checker_verdict",
            "old_artifact_hash_equality",
        },
        "MIGRATION_COVERAGE",
        "payment shared preservation",
    )
    shared_pairs = {
        "frontend_outcome": "frontend_outcome",
        "requires_count": "requires_count",
        "ensures_count": "ensures_count",
        "obligation_count": "postcondition_intent_count",
        "linear_theory_goal_count": "linear_theory_goal_count",
        "true_or_opaque_goal_count": "true_or_opaque_goal_count",
        "opaque_or_true_goal_count": "opaque_or_true_goal_count",
        "classification_evidence_label": "classification_evidence_label",
        "classification_property_status": "classification_property_status_until_program_certificate",
        "checker_verdict": "old_checker_verdict",
    }
    for old_field, new_field in shared_pairs.items():
        expect_equal(
            new_shared.get(new_field),
            old_shared[old_field],
            "MIGRATION_PROPERTY_INTENT",
            f"payment shared {old_field}",
        )
    expect_equal(
        old_shared["frontend_cli_status"],
        "gir-lowered",
        "MIGRATION_SOURCE_ACCEPTANCE",
        "payment old frontend status",
    )
    expect_equal(
        new_shared.get("new_frontend_status"),
        "ir-lowered",
        "MIGRATION_SOURCE_ACCEPTANCE",
        "payment new frontend status",
    )
    expect_equal(
        old_shared["obligation_kinds"],
        [new_shared.get("new_obligation_kind")],
        "MIGRATION_PROPERTY_INTENT",
        "payment postcondition kind",
    )
    expect_equal(
        new_shared.get("clause_and_branch_order"),
        "preserved",
        "MIGRATION_PROPERTY_INTENT",
        "payment clause and branch order",
    )
    expect_equal(
        new_shared.get("old_artifact_hash_equality"),
        False,
        "MIGRATION_PROPERTY_INTENT",
        "payment old hash equality",
    )

    accepted = []
    dispositions = [
        {
            "id": "source.payment.envelope",
            "baseline_pointer": "/corpora/payment_policies",
            "old": "frozen payment-policy manifest",
            "new": "same source-policy inventory under VIR/VC v1",
            "disposition": "payment corpus manifest identity preserved",
        },
        {
            "id": "source.payment.shared_property_intent",
            "baseline_pointer": "/corpora/payment_policies/shared_expectation",
            "old": f"{old_shared['obligation_count']} postcondition intents per policy",
            "new": f"{new_shared['postcondition_intent_count']} postcondition members per policy",
            "disposition": "clause order, branch order, classification, and proof-pending state preserved",
        }
    ]
    old_positive = require_array(old["positive"], "payment positive")
    new_positive = require_array(new["positive"], "payment positive mapping")
    expect_equal(
        len(new_positive),
        len(old_positive),
        "MIGRATION_SOURCE_ACCEPTANCE",
        "payment positive count",
    )
    for index, (old_case, new_case) in enumerate(zip(old_positive, new_positive)):
        require_exact_fields(
            require_object(new_case, f"payment positive[{index}]"),
            {
                "name",
                "function_id",
                "module_sha256",
                "source_sha256",
                "contract_sha256",
                "baseline_pointer",
                "classification_patterns",
                "disposition",
            },
            "MIGRATION_COVERAGE",
            f"payment positive[{index}]",
        )
        expected = {
            "name": old_case["name"],
            "function_id": old_case["function_id"],
            "module_sha256": old_case["module_sha256"],
            "source_sha256": old_case["source_sha256"],
            "contract_sha256": old_case["contract_sha256"],
            "baseline_pointer": f"/corpora/payment_policies/positive/{index}",
            "classification_patterns": old_case["classification_patterns"],
        }
        for field, value in expected.items():
            expect_equal(
                new_case.get(field),
                value,
                "MIGRATION_PROPERTY_INTENT",
                f"payment positive[{index}].{field}",
            )
        require_nonempty(new_case.get("disposition"), f"payment positive[{index}]")
        accepted.append(
            {
                "name": old_case["name"],
                "function_id": old_case["function_id"],
                "classification_patterns": old_case["classification_patterns"],
                "postcondition_intent_count": old_shared["obligation_count"],
            }
        )
        dispositions.append(
            {
                "id": f"source.payment.positive.{old_case['name']}",
                "baseline_pointer": expected["baseline_pointer"],
                "old": old_case["function_id"],
                "new": old_case["function_id"],
                "disposition": new_case["disposition"],
            }
        )

    rejected = []
    old_negative = require_array(old["negative"], "payment negative")
    new_negative = require_array(new["negative"], "payment negative mapping")
    expect_equal(
        len(new_negative),
        len(old_negative),
        "MIGRATION_NEGATIVE_REJECTION",
        "payment negative count",
    )
    for index, (old_case, new_case) in enumerate(zip(old_negative, new_negative)):
        require_exact_fields(
            require_object(new_case, f"payment negative[{index}]"),
            {"name", "baseline_pointer", "code", "disposition"},
            "MIGRATION_COVERAGE",
            f"payment negative[{index}]",
        )
        expect_equal(
            new_case.get("name"),
            old_case["name"],
            "MIGRATION_NEGATIVE_REJECTION",
            f"payment negative[{index}] name",
        )
        expect_equal(
            new_case.get("baseline_pointer"),
            f"/corpora/payment_policies/negative/{index}",
            "MIGRATION_COVERAGE",
            f"payment negative[{index}] pointer",
        )
        expect_equal(
            new_case.get("code"),
            EXPECTED_PAYMENT_REJECTIONS.get(old_case["name"]),
            "MIGRATION_NEGATIVE_REJECTION",
            f"payment negative {old_case['name']} rejection class",
        )
        require_nonempty(new_case.get("disposition"), f"payment negative[{index}]")
        rejected.append(
            {
                "name": old_case["name"],
                "path": old_case["path"],
                "old_reason": old_case["reason"],
                "new_code": new_case["code"],
            }
        )
        dispositions.append(
            {
                "id": f"source.payment.negative.{old_case['name']}",
                "baseline_pointer": new_case["baseline_pointer"],
                "old": old_case["reason"],
                "new": new_case["code"],
                "disposition": new_case["disposition"],
            }
        )
    return {
        "shared_property_intent": {
            "postcondition_intent_count": old_shared["obligation_count"],
            "clause_and_branch_order": "preserved",
            "classification_property_status": old_shared[
                "classification_property_status"
            ],
            "old_artifact_hash_equality": False,
        },
        "accepted": accepted,
        "rejected": rejected,
        "dispositions": dispositions,
    }


def validate_focused_rejections(old_value: Any, new_value: Any) -> list[dict[str, Any]]:
    old = require_array(old_value, "focused negative baseline")
    new = require_array(new_value, "focused negative mapping")
    expect_equal(
        len(old),
        len(EXPECTED_FOCUSED_REJECTIONS),
        "MIGRATION_NEGATIVE_REJECTION",
        "focused baseline negative count",
    )
    expect_equal(
        len(new),
        len(old),
        "MIGRATION_NEGATIVE_REJECTION",
        "focused negative mapping count",
    )
    result = []
    for index, (old_case, new_case, expected) in enumerate(
        zip(old, new, EXPECTED_FOCUSED_REJECTIONS)
    ):
        path, code, message = expected
        required_fields = {"baseline_index", "path", "new_code"}
        if message is not None:
            required_fields.add("new_message")
        require_exact_fields(
            require_object(new_case, f"focused[{index}]"),
            required_fields,
            "MIGRATION_COVERAGE",
            f"focused[{index}]",
        )
        expect_equal(old_case.get("path"), path, "MIGRATION_NEGATIVE_REJECTION", f"focused[{index}] baseline path")
        expect_equal(new_case.get("baseline_index"), index, "MIGRATION_COVERAGE", f"focused[{index}] baseline index")
        expect_equal(new_case.get("path"), path, "MIGRATION_NEGATIVE_REJECTION", f"focused[{index}] mapped path")
        expect_equal(new_case.get("new_code"), code, "MIGRATION_NEGATIVE_REJECTION", f"focused[{index}] rejection class")
        expect_equal(new_case.get("new_message"), message, "MIGRATION_NEGATIVE_REJECTION", f"focused[{index}] message correction")
        result.append(
            {
                "baseline_index": index,
                "path": path,
                "old_reason": old_case["reason"],
                "new_code": code,
                "new_message": message,
            }
        )
    return result


def validate_behavioral_anchors(
    baseline: dict[str, Any], migration: dict[str, Any]
) -> dict[str, Any]:
    old = require_object(baseline["behavioral_anchors"], "behavioral anchors")
    mappings = require_array(migration["behavioral_anchors"], "behavioral mappings")
    mapping_by_pointer = {}
    for item in mappings:
        item = require_object(item, "behavioral mapping")
        require_exact_fields(
            item,
            {"baseline_pointer", "disposition"},
            "MIGRATION_COVERAGE",
            "behavioral mapping",
        )
        mapping_by_pointer[item["baseline_pointer"]] = item["disposition"]
    expected_pointers = {f"/behavioral_anchors/{key}" for key in old}
    expect_equal(
        set(mapping_by_pointer),
        expected_pointers,
        "MIGRATION_COVERAGE",
        "behavioral anchor pointers",
    )

    contracts = require_object(old["contracts"], "contract anchor")
    expect_equal(
        contracts["rules"],
        EXPECTED_CONTRACT_RULES,
        "MIGRATION_PROPERTY_INTENT",
        "normalized contract rules",
    )
    contract_disposition = mapping_by_pointer["/behavioral_anchors/contracts"]
    for phrase in ("requires", "ensures", "modifies", "loop", "fail-closed", "true"):
        expect_true(
            phrase in contract_disposition,
            "MIGRATION_PROPERTY_INTENT",
            f"contract disposition must name {phrase!r}",
        )

    runtime_checks = validate_runtime_checks(
        old["runtime_checks"],
        mapping_by_pointer["/behavioral_anchors/runtime_checks"],
    )
    loops = validate_loop_members(
        old["loops"], mapping_by_pointer["/behavioral_anchors/loops"]
    )
    conversions = require_object(old["conversions"], "conversion anchor")
    expect_true(
        "exact fixed-BV target" in mapping_by_pointer["/behavioral_anchors/conversions"],
        "MIGRATION_OPERATION_SEMANTICS",
        "conversion disposition must preserve exact fixed-BV targets",
    )
    dispositions = [
        {
            "id": f"behavior.{key}",
            "baseline_pointer": f"/behavioral_anchors/{key}",
            "old": "complete captured behavioral subtree",
            "new": "normalized VIR/VC v1 behavior",
            "disposition": mapping_by_pointer[f"/behavioral_anchors/{key}"],
        }
        for key in old
    ]
    return {
        "normalized_contracts": {
            "old_rules": contracts["rules"],
            "new_rules": [
                "requires and loops default empty",
                "present sidecar requires nonempty ensures and empty modifies",
                "function and loop identity resolve canonically",
                "unknown operators and malformed typed literals reject",
                "absent sidecar on an acyclic function normalizes to true",
            ],
        },
        "runtime_checks": runtime_checks,
        "loop_members": loops,
        "conversion": {
            "old_intent": conversions["intent"],
            "new_intent": "explicit Convert with an exact fixed-BV target",
        },
        "dispositions": dispositions,
    }


def validate_runtime_checks(old_value: Any, disposition: str) -> list[dict[str, Any]]:
    old = require_object(old_value, "runtime check anchor")
    checks = require_array(old["checks"], "runtime checks")
    expect_equal(
        len(checks),
        len(EXPECTED_RUNTIME_CHECKS),
        "MIGRATION_RUNTIME_CHECK_SET",
        "required runtime check count",
    )
    result = []
    for index, expected in enumerate(EXPECTED_RUNTIME_CHECKS):
        actual = checks[index]
        old_member_count = actual.get("obligation_count", 1)
        for field in ("operation", "test", "obligation_kind", "predicate"):
            expect_equal(
                actual.get(field),
                expected[field],
                "MIGRATION_RUNTIME_CHECK_SET",
                f"runtime check[{index}].{field}",
            )
        expect_equal(
            old_member_count,
            expected["old_member_count"],
            "MIGRATION_RUNTIME_CHECK_SEMANTICS",
            f"runtime check[{index}] old member count",
        )
        result.append(
            {
                "operation": expected["operation"],
                "old_predicate": expected["predicate"],
                "old_member_count": expected["old_member_count"],
                "new_check": expected["new_check"],
                "new_obligation_kind": expected["new_obligation_kind"],
                "predicate_components": expected["predicate_components"],
                "safety_semantics": "preserved",
            }
        )
    for phrase in (
        "divisor nonzero",
        "signed shift nonnegative",
        "both signed array bounds",
        "one index_in_bounds record owns the two signed conjuncts",
    ):
        expect_true(
            phrase in disposition,
            "MIGRATION_RUNTIME_CHECK_SEMANTICS",
            f"runtime-check disposition must name {phrase!r}",
        )
    return result


def validate_loop_members(old_value: Any, disposition: str) -> dict[str, Any]:
    old = require_object(old_value, "loop anchor")
    expected_partial = [
        "loop_invariant_initial",
        "loop_invariant_initial",
        "loop_invariant_preservation",
        "loop_invariant_preservation",
        "loop_exit",
    ]
    expected_additional = ["decreases", "decreases"]
    expect_equal(
        old["partial_obligation_kinds"],
        expected_partial,
        "MIGRATION_LOOP_MEMBERS",
        "partial loop obligation members",
    )
    expect_equal(
        old["additional_total_kinds"],
        expected_additional,
        "MIGRATION_LOOP_MEMBERS",
        "total loop additional members",
    )
    expect_equal(
        old["total_obligation_count"],
        len(expected_partial) + len(expected_additional),
        "MIGRATION_LOOP_MEMBERS",
        "total loop member count",
    )
    kind_map = dict(EXPECTED_OBLIGATION_KINDS)
    for phrase in ("partial", "total", "missing invariant", "invalid backedge"):
        expect_true(
            phrase in disposition,
            "MIGRATION_LOOP_MEMBERS",
            f"loop disposition must name {phrase!r}",
        )
    return {
        "old_partial_members": expected_partial,
        "new_partial_members": [kind_map[item] for item in expected_partial],
        "old_total_additional_members": expected_additional,
        "new_total_additional_members": [kind_map[item] for item in expected_additional],
        "total_member_count": old["total_obligation_count"],
        "negative_cases": old["negative_tests"],
    }


def validate_regenerated_corpus(
    repo_root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    require_exact_fields(
        manifest,
        {
            "schema",
            "status",
            "generation",
            "coverage",
            "checker_audit",
            "artifacts",
            "unresolved_dispositions",
        },
        "MIGRATION_CORPUS",
        "regenerated corpus manifest",
    )
    expect_equal(manifest["schema"], CORPUS_SCHEMA, "MIGRATION_CORPUS", "corpus schema")
    expect_equal(
        manifest["status"],
        "reviewed_zero_unexplained_differences",
        "MIGRATION_CORPUS",
        "corpus status",
    )
    expect_equal(
        manifest["unresolved_dispositions"],
        [],
        "MIGRATION_CORPUS",
        "corpus unresolved dispositions",
    )

    generation = require_object(manifest["generation"], "corpus generation")
    require_exact_fields(
        generation,
        {
            "commands",
            "clean_runs",
            "byte_identical",
            "leakage_scan",
            "intentional_hash_migration",
            "compatibility_aliases",
            "active_release_selects_staging",
        },
        "MIGRATION_CORPUS",
        "corpus generation",
    )
    expect_equal(generation["clean_runs"], 2, "MIGRATION_CORPUS", "clean generation runs")
    expect_equal(generation["byte_identical"], True, "MIGRATION_CORPUS", "byte equality")
    expect_equal(
        generation["intentional_hash_migration"],
        True,
        "MIGRATION_CORPUS",
        "intentional hash migration",
    )
    expect_equal(
        generation["compatibility_aliases"],
        False,
        "MIGRATION_CORPUS",
        "compatibility aliases",
    )
    expect_equal(
        generation["active_release_selects_staging"],
        False,
        "MIGRATION_CORPUS",
        "active release staging selection",
    )
    expect_equal(
        generation["commands"],
        [
            "MPK_UPDATE_GO_VIR_CORPUS=1 go test -count=1 -run TestRegenerateGoVIRFrontendCorpus",
            "MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus",
            "python3 scripts/compare-go-gir-vir.py --write",
        ],
        "MIGRATION_CORPUS",
        "explicit regeneration commands",
    )
    leakage = require_nonempty(generation["leakage_scan"], "corpus leakage scan")
    for category in ("local_path", "temp_path", "host", "timestamp", "obsolete_interface"):
        expect_true(category in leakage, "MIGRATION_CORPUS", f"missing leakage category {category}")

    coverage = require_object(manifest["coverage"], "corpus coverage")
    expected_coverage = {
        "alpha_functions": 100,
        "positive_frontend_roots": 13,
        "vc_fixture_roots": 11,
        "frontend_only_aggregate_roots": ["alpha-array", "basic-structarray"],
        "negative_frontend_roots": 8,
        "payment_policies": 5,
        "loops": 5,
        "conversions": 5,
        "runtime_operations": 9,
        "calls": 6,
        "contracts": 21,
    }
    expect_equal(coverage, expected_coverage, "MIGRATION_CORPUS", "corpus coverage")

    checker = require_object(manifest["checker_audit"], "corpus checker audit")
    expect_equal(
        checker,
        {
            "certificate": "checker/one-theorem.hex",
            "source_free": "accepted",
            "reference": "accepted",
            "hash_agreement": True,
            "axiom_count": 0,
        },
        "MIGRATION_CORPUS",
        "corpus checker audit",
    )

    artifacts = require_array(manifest["artifacts"], "corpus artifacts")
    artifact_paths: list[str] = []
    artifact_kinds: set[str] = set()
    corpus_root = manifest_path.parent
    staging_fixture_root = repo_root / STAGING_ROOT_PATH / "fixtures/vir-go"
    for index, value in enumerate(artifacts):
        item = require_object(value, f"corpus artifacts[{index}]")
        require_exact_fields(
            item,
            {"kind", "path", "sha256", "bytes"},
            "MIGRATION_CORPUS",
            f"corpus artifacts[{index}]",
        )
        relative = require_nonempty(item["path"], f"corpus artifacts[{index}].path")
        relative_path = Path(relative)
        expect_true(
            not relative_path.is_absolute()
            and ".." not in relative_path.parts
            and relative_path.as_posix() == relative,
            "MIGRATION_CORPUS",
            f"unsafe corpus artifact path {relative!r}",
        )
        kind = require_nonempty(item["kind"], f"corpus artifacts[{index}].kind")
        expect_true(
            isinstance(item["bytes"], int) and item["bytes"] >= 0,
            "MIGRATION_CORPUS",
            f"invalid corpus artifact byte count for {relative}",
        )
        shared = require_regular_path(corpus_root / relative, relative)
        mirrored = require_regular_path(staging_fixture_root / relative, relative)
        try:
            shared_bytes = shared.read_bytes()
            mirrored_bytes = mirrored.read_bytes()
        except OSError as error:
            raise MigrationError(
                "MIGRATION_CORPUS", f"cannot read corpus artifact {relative}: {error}"
            ) from error
        expect_equal(len(shared_bytes), item["bytes"], "MIGRATION_CORPUS", f"{relative} bytes")
        expect_equal(sha256(shared_bytes), item["sha256"], "MIGRATION_CORPUS", f"{relative} SHA-256")
        expect_equal(mirrored_bytes, shared_bytes, "MIGRATION_CORPUS", f"{relative} staging mirror")
        artifact_paths.append(relative)
        artifact_kinds.add(kind)

    expect_equal(artifact_paths, sorted(artifact_paths), "MIGRATION_CORPUS", "artifact path order")
    require_unique(artifact_paths, "regenerated corpus artifact paths")
    expect_true(len(artifacts) >= 90, "MIGRATION_CORPUS", "regenerated artifact inventory is incomplete")
    expected_kinds = {
        "frontend_envelope",
        "vir",
        "source_map",
        "source_manifest_frontend",
        "vc_v1",
        "grouped_skeleton",
        "source_manifest_certificate",
        "certificate",
        "axiom_report",
        "checker_audit",
        "policy_scan_v1",
        "policy_evidence_v1",
        "ai_v1_dry_run",
        "ai_v1_output",
        "ai_api_v1",
    }
    expect_true(
        expected_kinds <= artifact_kinds,
        "MIGRATION_CORPUS",
        f"regenerated artifact kinds are incomplete: {sorted(expected_kinds - artifact_kinds)!r}",
    )
    expected_files = set(artifact_paths) | {manifest_path.name}
    expect_equal(
        regular_file_inventory(corpus_root),
        expected_files,
        "MIGRATION_CORPUS",
        "shared corpus file inventory",
    )
    expect_equal(
        regular_file_inventory(staging_fixture_root),
        expected_files,
        "MIGRATION_CORPUS",
        "staging corpus file inventory",
    )
    expect_equal(
        (staging_fixture_root / manifest_path.name).read_bytes(),
        manifest_bytes,
        "MIGRATION_CORPUS",
        "staging corpus manifest",
    )

    frontend_index = read_json_object(corpus_root / "frontend-index.json")[1]
    derived_index = read_json_object(corpus_root / "derived-index.json")[1]
    expect_equal(frontend_index["alpha_function_count"], 100, "MIGRATION_CORPUS", "frontend alpha count")
    expect_equal(len(frontend_index["cases"]), 13, "MIGRATION_CORPUS", "frontend root count")
    expect_equal(len(frontend_index["negative_cases"]), 8, "MIGRATION_CORPUS", "negative root count")
    expect_equal(frontend_index["semantic_vector"]["unresolved_cases"], 0, "MIGRATION_CORPUS", "semantic vector unresolved cases")
    expect_equal(derived_index["deterministic_runs"], 2, "MIGRATION_CORPUS", "derived clean runs")
    expect_equal(len(derived_index["cases"]), 11, "MIGRATION_CORPUS", "derived VC roots")

    example_count = validate_staged_examples(repo_root, corpus_root, frontend_index)
    validate_staged_alpha(repo_root, corpus_root)
    active_release = require_regular_file(repo_root, "release-report.json").read_bytes()
    staged_release = require_regular_file(repo_root, f"{STAGING_ROOT_PATH}/release-report.json").read_bytes()
    expect_equal(staged_release, active_release, "MIGRATION_CORPUS", "staged release report")
    expect_true(
        STAGING_ROOT_PATH.encode() not in active_release,
        "MIGRATION_CORPUS",
        "active release report selects staging",
    )

    checker_verdicts = read_json_object(corpus_root / "checker/verdicts.json")[1]
    expect_equal(checker_verdicts["unresolved"], [], "MIGRATION_CORPUS", "checker unresolved")
    expect_equal(checker_verdicts["source_free_checker"]["verdict"], "accepted", "MIGRATION_CORPUS", "source-free checker")
    expect_equal(checker_verdicts["reference_checker"]["verdict"], "accepted", "MIGRATION_CORPUS", "reference checker")
    hash_agreement = require_object(checker_verdicts["hash_agreement"], "checker hash agreement")
    expect_true(all(value is True for value in hash_agreement.values()), "MIGRATION_CORPUS", "checker hashes disagree")
    axiom_report = read_json_object(corpus_root / "checker/axiom-report.json")[1]
    expect_equal(axiom_report["report"]["summary"]["total_axiom_count"], 0, "MIGRATION_CORPUS", "axiom count")

    return {
        "path": display_path(repo_root, manifest_path),
        "sha256": sha256(manifest_bytes),
        "status": manifest["status"],
        "artifact_count": len(artifacts),
        "staging_mirror_count": len(artifacts),
        "example_replacement_count": example_count,
        "coverage": coverage,
        "generation": generation,
        "checker_audit": checker,
        "unresolved_disposition_count": 0,
    }


def validate_staged_examples(
    repo_root: Path, corpus_root: Path, frontend_index: dict[str, Any]
) -> int:
    cases = [item for item in frontend_index["cases"] if item.get("example_stage_path")]
    expect_equal(len(cases), 7, "MIGRATION_CORPUS", "staged example count")
    for item in cases:
        stage = require_nonempty(item["example_stage_path"], "example stage path")
        expect_true(
            stage.startswith(f"{STAGING_ROOT_PATH}/examples/"),
            "MIGRATION_CORPUS",
            f"example is outside staging root: {stage}",
        )
        stage_root = repo_root / stage
        expected = {
            "frontend-envelope.json",
            "vir.json",
            "source-map.json",
            "source-manifest.frontend.json",
            "vc.json",
            "vc_skeleton.json",
            "source-manifest.certificate.json",
        }
        frontend_names = {
            "frontend-envelope.json",
            "vir.json",
            "source-map.json",
            "source-manifest.frontend.json",
        }
        expect_equal(regular_file_inventory(stage_root), expected, "MIGRATION_CORPUS", f"{stage} inventory")
        frontend_root = corpus_root / "frontend" / item["id"]
        derived_root = corpus_root / "derived" / item["id"]
        for source_name, staged_name in (
            ("frontend-envelope.json", "frontend-envelope.json"),
            ("vir.json", "vir.json"),
            ("source-map.json", "source-map.json"),
            ("source-manifest.frontend.json", "source-manifest.frontend.json"),
            ("vc.json", "vc.json"),
            ("vc-skeleton.json", "vc_skeleton.json"),
            ("source-manifest.certificate.json", "source-manifest.certificate.json"),
        ):
            source_root = frontend_root if source_name in frontend_names else derived_root
            expect_equal(
                require_regular_path(stage_root / staged_name, staged_name).read_bytes(),
                require_regular_path(source_root / source_name, source_name).read_bytes(),
                "MIGRATION_CORPUS",
                f"{stage}/{staged_name}",
            )
    return len(cases)


def validate_staged_alpha(repo_root: Path, corpus_root: Path) -> None:
    stage = repo_root / STAGING_ROOT_PATH / "fixtures/vc-alpha"
    expected = {"manifest.json", "vc.json", "vc_skeleton.json"}
    expect_equal(regular_file_inventory(stage), expected, "MIGRATION_CORPUS", "staged alpha inventory")
    derived = corpus_root / "derived/alpha-branch"
    for source_name, staged_name in (
        ("vc-alpha-manifest.json", "manifest.json"),
        ("vc.json", "vc.json"),
        ("vc-skeleton.json", "vc_skeleton.json"),
    ):
        expect_equal(
            require_regular_path(stage / staged_name, staged_name).read_bytes(),
            require_regular_path(derived / source_name, source_name).read_bytes(),
            "MIGRATION_CORPUS",
            f"staged alpha {staged_name}",
        )


def regular_file_inventory(root: Path) -> set[str]:
    try:
        paths = list(root.rglob("*"))
    except OSError as error:
        raise MigrationError("MIGRATION_CORPUS", f"cannot enumerate {root}: {error}") from error
    files = set()
    for path in paths:
        if path.is_symlink():
            raise MigrationError("MIGRATION_CORPUS", f"corpus contains symlink: {path}")
        if path.is_file():
            files.add(path.relative_to(root).as_posix())
    return files


def require_regular_path(path: Path, label: str) -> Path:
    if not path.is_file() or path.is_symlink():
        raise MigrationError("MIGRATION_CORPUS", f"corpus artifact must be a regular file: {label}")
    return path


def validate_checker_anchors(
    repo_root: Path, baseline: dict[str, Any], migration: dict[str, Any]
) -> dict[str, Any]:
    old = require_object(baseline["checker_baseline"], "checker baseline")
    mappings = require_array(migration["checker_anchors"], "checker anchors")
    mapping_by_pointer = {item.get("baseline_pointer"): item for item in mappings}
    expected_pointers = {f"/checker_baseline/{key}" for key in old}
    expect_equal(
        set(mapping_by_pointer),
        expected_pointers,
        "MIGRATION_COVERAGE",
        "checker baseline pointers",
    )
    release = require_object(old["release_report"], "release checker anchor")
    expect_equal(
        release["source_free_checker_accepted"],
        release["reference_checker_accepted"],
        "MIGRATION_CHECKER_DISAGREEMENT",
        "release checker acceptance",
    )
    expect_equal(
        release["source_free_checker_accepted"],
        True,
        "MIGRATION_CHECKER_DISAGREEMENT",
        "release checker accepted verdict",
    )
    certificate = require_object(release["certificate"], "release certificate anchor")
    expect_equal(
        certificate["source_free_verdict"],
        certificate["reference_verdict"],
        "MIGRATION_CHECKER_DISAGREEMENT",
        "certificate checker verdict",
    )
    expect_equal(
        release["hash_agreement"],
        True,
        "MIGRATION_CHECKER_DISAGREEMENT",
        "release hash agreement",
    )
    verify_fixture_digest(repo_root, release["path"], release["sha256"])

    payment = require_object(
        old["payment_reserve_theory_evidence"], "payment checker anchor"
    )
    expect_equal(
        payment["source_free_checker_verdict"],
        payment["reference_checker_verdict"],
        "MIGRATION_CHECKER_DISAGREEMENT",
        "payment checker pending verdicts",
    )
    expect_equal(
        payment["source_free_checker_verdict"],
        None,
        "MIGRATION_CHECKER_DISAGREEMENT",
        "historical payment checker verdict remains an audit-only proof-pending anchor",
    )
    verify_fixture_digest(repo_root, payment["path"], payment["sha256"])
    anchors = [
        {
            "id": "release_report",
            "baseline_pointer": "/checker_baseline/release_report",
            "source_free_verdict": certificate["source_free_verdict"],
            "reference_verdict": certificate["reference_verdict"],
            "hash_agreement": release["hash_agreement"],
            "status": "preserved",
        },
        {
            "id": "payment_reserve_theory_evidence",
            "baseline_pointer": "/checker_baseline/payment_reserve_theory_evidence",
            "source_free_verdict": None,
            "reference_verdict": None,
            "hash_agreement": None,
            "status": "historical_audit_only_proof_pending",
        },
    ]
    dispositions = []
    for anchor in anchors:
        mapped = mapping_by_pointer[anchor["baseline_pointer"]]
        require_exact_fields(
            require_object(mapped, "checker mapping"),
            {"baseline_pointer", "disposition"},
            "MIGRATION_COVERAGE",
            "checker mapping",
        )
        require_nonempty(mapped["disposition"], "checker disposition")
        dispositions.append(
            {
                "id": f"checker.{anchor['id']}",
                "baseline_pointer": anchor["baseline_pointer"],
                "old": "captured independent checker facts",
                "new": anchor["status"],
                "disposition": mapped["disposition"],
            }
        )
    return {"anchors": anchors, "dispositions": dispositions}


def validate_leaf_coverage(
    baseline: dict[str, Any],
    migration: dict[str, Any],
    disposition_ids: set[str],
) -> dict[str, Any]:
    assignments = [
        ("/schema", "schema"),
        ("/captured_from_revision", "identity.captured_revision"),
        ("/source_profile", "identity.source_profile"),
        ("/source_ir_schema", "identity.source_ir_schema"),
        ("/purpose", "identity.purpose"),
        ("/hash_policy", "identity.hash_policy"),
        ("/obligation_model/kinds", "obligation.kind_inventory"),
    ]
    assignments.extend(
        (
            item["baseline_pointer"],
            f"frontend.{item['baseline_pointer'].rsplit('/', 1)[-1]}",
        )
        for item in migration["frontend_contract"]
    )
    assignments.extend(
        (
            f"/obligation_model/theorem_intent/{old}",
            f"obligation.{old}",
        )
        for old, _ in EXPECTED_OBLIGATION_KINDS
    )
    corpora = migration["corpora"]
    assignments.extend(
        [
            ("/corpora/go_alpha/manifest", "source.go_alpha.envelope"),
            ("/corpora/go_alpha/module_path", "source.go_alpha.envelope"),
            ("/corpora/go_alpha/module_sha256", "source.go_alpha.envelope"),
            ("/corpora/go_alpha/expected_outcome", "source.go_alpha.envelope"),
            ("/corpora/go_alpha/expected_cli_status", "source.go_alpha.envelope"),
            ("/corpora/go_alpha/vc_fixture", "source.go_alpha.vc_fixture"),
            ("/corpora/go_basic/manifest_path", "source.go_basic.envelope"),
            ("/corpora/go_basic/manifest_sha256", "source.go_basic.envelope"),
            ("/corpora/go_basic/module_path", "source.go_basic.envelope"),
            ("/corpora/go_basic/module_sha256", "source.go_basic.envelope"),
            ("/corpora/payment_policies/manifest_path", "source.payment.envelope"),
            ("/corpora/payment_policies/manifest_sha256", "source.payment.envelope"),
            (
                "/corpora/payment_policies/shared_expectation",
                "source.payment.shared_property_intent",
            ),
        ]
    )
    assignments.extend(
        (
            f"/corpora/go_alpha/groups/{index}",
            f"source.go_alpha.{group['name']}",
        )
        for index, group in enumerate(baseline["corpora"]["go_alpha"]["groups"])
    )
    assignments.extend(
        (
            f"/corpora/go_basic/positive/{index}",
            f"source.go_basic.positive.{case['name']}",
        )
        for index, case in enumerate(baseline["corpora"]["go_basic"]["positive"])
    )
    assignments.extend(
        (
            f"/corpora/go_basic/negative/{index}",
            f"source.go_basic.negative.{case['name']}",
        )
        for index, case in enumerate(baseline["corpora"]["go_basic"]["negative"])
    )
    assignments.extend(
        (
            f"/corpora/payment_policies/positive/{index}",
            f"source.payment.positive.{case['name']}",
        )
        for index, case in enumerate(
            baseline["corpora"]["payment_policies"]["positive"]
        )
    )
    assignments.extend(
        (
            f"/corpora/payment_policies/negative/{index}",
            f"source.payment.negative.{case['name']}",
        )
        for index, case in enumerate(
            baseline["corpora"]["payment_policies"]["negative"]
        )
    )
    assignments.extend(
        (
            f"/corpora/focused_frontend_negative_cases/{item['baseline_index']}",
            f"source.focused_rejection.{item['baseline_index']}",
        )
        for item in corpora["focused_frontend_negative_cases"]
    )
    assignments.extend(
        (
            item["baseline_pointer"],
            f"behavior.{item['baseline_pointer'].rsplit('/', 1)[-1]}",
        )
        for item in migration["behavioral_anchors"]
    )
    assignments.extend(
        (
            item["baseline_pointer"],
            f"checker.{item['baseline_pointer'].rsplit('/', 1)[-1]}",
        )
        for item in migration["checker_anchors"]
    )
    unknown_owners = sorted({owner for _, owner in assignments} - disposition_ids)
    expect_equal(
        unknown_owners,
        [],
        "MIGRATION_COVERAGE",
        "coverage owners without reviewed dispositions",
    )
    pointers = list(iter_leaf_pointers(baseline))
    uncovered = []
    ambiguous = []
    for pointer in pointers:
        owners = [
            disposition
            for root, disposition in assignments
            if pointer == root or pointer.startswith(root + "/")
        ]
        if not owners:
            uncovered.append(pointer)
        elif len(owners) > 1:
            ambiguous.append({"pointer": pointer, "owners": owners})
    expect_equal(
        uncovered,
        [],
        "MIGRATION_COVERAGE",
        "uncovered baseline leaves",
    )
    expect_equal(
        ambiguous,
        [],
        "MIGRATION_COVERAGE",
        "multiply assigned baseline leaves",
    )
    return {
        "baseline_leaf_count": len(pointers),
        "covered_leaf_count": len(pointers),
        "assignment_count": len(assignments),
        "assignments": [
            {"baseline_pointer": pointer, "disposition_id": disposition}
            for pointer, disposition in assignments
        ],
        "uncovered": [],
        "ambiguous": [],
    }


def iter_leaf_pointers(value: Any, pointer: str = ""):
    if isinstance(value, dict) and value:
        for key, item in value.items():
            escaped = key.replace("~", "~0").replace("/", "~1")
            yield from iter_leaf_pointers(item, f"{pointer}/{escaped}")
    elif isinstance(value, list) and value:
        for index, item in enumerate(value):
            yield from iter_leaf_pointers(item, f"{pointer}/{index}")
    else:
        yield pointer or "/"


def render_json(report: dict[str, Any]) -> str:
    return json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def render_markdown(report: dict[str, Any]) -> str:
    summary = report["summary"]
    lines = [
        "# Go GIR-to-VIR semantic migration report",
        "",
        "> Derived by `scripts/compare-go-gir-vir.py`; do not edit by hand.",
        "> This is a development-only audit artifact owned for archival or deletion by GO-VIR-02-T12.",
        "",
        "## Result",
        "",
        f"**{report['status']}** with {summary['unexplained_difference_count']} unexplained differences.",
        "",
        "| Measure | Count |",
        "|---|---:|",
        f"| Baseline leaves covered | {summary['covered_leaf_count']} / {summary['baseline_leaf_count']} |",
        f"| Reviewed dispositions | {summary['reviewed_disposition_count']} |",
        f"| Historical accepted/rejected rules | {summary['historical_accepted_rule_count']} / {summary['historical_rejected_rule_count']} |",
        f"| Accepted/rejected source cases | {summary['accepted_source_case_count']} / {summary['rejected_source_case_count']} |",
        f"| Canonical function identities | {summary['canonical_function_identity_count']} |",
        f"| Runtime checks | {summary['runtime_check_count']} |",
        f"| Obligation kinds | {summary['obligation_kind_count']} |",
        f"| Checker anchors | {summary['checker_anchor_count']} |",
        f"| Checked fixture files | {summary['checked_fixture_file_count']} |",
        f"| Regenerated Go/VIR artifacts | {summary['regenerated_artifact_count']} |",
        "",
        "## Explicitly allowed changes",
        "",
        "| Change | Old | New | Reviewed disposition |",
        "|---|---|---|---|",
    ]
    for item in report["allowed_changes"]:
        lines.append(
            f"| `{md(item['id'])}` | {md(item['old'])} | {md(item['new'])} | {md(item['disposition'])} |"
        )
    lines.extend(
        [
            "",
            "## Frontend contract",
            "",
            "| Baseline pointer | Old | New | Disposition |",
            "|---|---|---|---|",
        ]
    )
    for item in report["frontend_contract"]:
        lines.append(
            f"| `{md(item['baseline_pointer'])}` | {md_value(item['old'])} | {md_value(item['new'])} | {md(item['disposition'])} |"
        )
    lines.extend(
        [
            "",
            "## Obligation kinds",
            "",
            "| Old | New | Preserved intent |",
            "|---|---|---|",
        ]
    )
    for item in report["obligation_kinds"]:
        lines.append(
            f"| `{md(item['old'])}` | `{md(item['new'])}` | {md(item['disposition'])} |"
        )
    lines.extend(
        [
            "",
            "## Source inventory",
            "",
            "| Corpus | Accepted | Rejected | Semantic disposition |",
            "|---|---:|---:|---|",
            f"| Go alpha | {report['source_inventory']['go_alpha']['function_count']} | 0 | Complete 100-function inventory and operation intent preserved |",
            f"| Go basic | {len(report['source_inventory']['go_basic']['accepted'])} | {len(report['source_inventory']['go_basic']['rejected'])} | Outcomes and deterministic rejection classes preserved |",
            f"| Payment policies | {len(report['source_inventory']['payment_policies']['accepted'])} | {len(report['source_inventory']['payment_policies']['rejected'])} | Eight postcondition intents, classifications, and clause/branch order preserved |",
            f"| Focused frontend cases | 0 | {len(report['source_inventory']['focused_rejections'])} | Reviewed diagnostic corrections only |",
            "",
            "## Required runtime checks",
            "",
            "| Operation | Old predicate | New check/member kind | Predicate components |",
            "|---|---|---|---:|",
        ]
    )
    for item in report["semantics"]["runtime_checks"]:
        lines.append(
            f"| {md(item['operation'])} | `{md(item['old_predicate'])}` | `{md(item['new_check'])}` / `{md(item['new_obligation_kind'])}` | {item['predicate_components']} |"
        )
    loops = report["semantics"]["loop_members"]
    lines.extend(
        [
            "",
            "## Contracts, loops, and property intent",
            "",
            f"- Normalized contract rules: {len(report['semantics']['normalized_contracts']['new_rules'])} reviewed rules.",
            f"- Partial loop members: `{md(', '.join(loops['new_partial_members']))}`.",
            f"- Total-correctness loop members: {loops['total_member_count']} including two `{md(loops['new_total_additional_members'][0])}` members.",
            f"- Payment-policy postcondition intents per policy: {report['source_inventory']['payment_policies']['shared_property_intent']['postcondition_intent_count']}; clause and branch order preserved.",
            "",
            "## Checker anchors",
            "",
            "| Anchor | Source-free | Reference | Hash agreement | Status |",
            "|---|---|---|---|---|",
        ]
    )
    for item in report["checker_verdicts"]:
        lines.append(
            f"| `{md(item['id'])}` | {md_value(item['source_free_verdict'])} | {md_value(item['reference_verdict'])} | {md_value(item['hash_agreement'])} | `{md(item['status'])}` |"
        )
    corpus = report["regenerated_corpus"]
    corpus_coverage = corpus["coverage"]
    lines.extend(
        [
            "",
            "## Regenerated Go/VIR corpus",
            "",
            f"- `{md(corpus['path'])}` records {corpus['artifact_count']} hash-linked artifacts, mirrored byte-for-byte into the staging root.",
            f"- Frontend coverage: {corpus_coverage['alpha_functions']} alpha functions, {corpus_coverage['positive_frontend_roots']} positive roots, {corpus_coverage['negative_frontend_roots']} deterministic negative roots, and {corpus_coverage['payment_policies']} payment policies.",
            f"- VC fixture coverage: {corpus_coverage['vc_fixture_roots']} roots; aggregate roots `{md(', '.join(corpus_coverage['frontend_only_aggregate_roots']))}` remain explicit frontend-only fixtures for the current VC foundation.",
            f"- Both independent checkers accepted the staged certificate with hash agreement and {corpus['checker_audit']['axiom_count']} axioms.",
            f"- Two clean generations were byte-identical; unresolved dispositions: {corpus['unresolved_disposition_count']}.",
            "- VIR/VC hashes are recorded as an intentional migration; no compatibility alias or old byte form is installed.",
        ]
    )
    lines.extend(
        [
            "",
            "## Coverage",
            "",
            f"All {report['coverage']['baseline_leaf_count']} baseline leaves have exactly one coverage assignment. The findings list is empty.",
            "",
        ]
    )
    return "\n".join(lines)


def md(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def md_value(value: Any) -> str:
    if value is None:
        return "pending"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (dict, list)):
        return f"`{md(json.dumps(value, ensure_ascii=False, sort_keys=True))}`"
    return md(str(value))


def read_json_object(path: Path) -> tuple[bytes, dict[str, Any]]:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise MigrationError("MIGRATION_INPUT", f"cannot read {path}: {error}") from error

    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result = {}
        for key, value in pairs:
            if key in result:
                raise MigrationError(
                    "MIGRATION_INPUT", f"{path}: duplicate JSON member {key!r}"
                )
            result[key] = value
        return result

    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=reject_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise MigrationError("MIGRATION_INPUT", f"invalid JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise MigrationError("MIGRATION_INPUT", f"{path}: root must be an object")
    return data, value


def resolve_path(repo_root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else repo_root / path


def display_path(repo_root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_fixture_digest(repo_root: Path, path: str, expected: str) -> None:
    checked_path = require_regular_file(repo_root, path)
    try:
        actual = sha256(checked_path.read_bytes())
    except OSError as error:
        raise MigrationError(
            "MIGRATION_FIXTURE_INTEGRITY", f"cannot read fixture anchor {path}: {error}"
        ) from error
    expect_equal(
        actual,
        expected,
        "MIGRATION_FIXTURE_INTEGRITY",
        f"fixture anchor {path} SHA-256",
    )


def require_regular_file(repo_root: Path, path: str) -> Path:
    checked_path = repo_root / path
    if not checked_path.is_file() or checked_path.is_symlink():
        raise MigrationError(
            "MIGRATION_FIXTURE_INTEGRITY",
            f"fixture anchor must be a regular non-symlink file: {path}",
        )
    return checked_path


def check_report(path: Path, expected: str) -> None:
    try:
        actual = path.read_text(encoding="utf-8")
    except OSError as error:
        raise MigrationError("MIGRATION_REPORT_STALE", f"cannot read {path}: {error}") from error
    expect_equal(actual, expected, "MIGRATION_REPORT_STALE", f"derived report {path}")


def write_report(path: Path, content: str) -> None:
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}")
        temporary.write_text(content, encoding="utf-8", newline="\n")
        temporary.replace(path)
    except OSError as error:
        raise MigrationError("MIGRATION_REPORT_WRITE", f"cannot write {path}: {error}") from error


def require_object(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise MigrationError("MIGRATION_INPUT", f"{field} must be an object")
    return value


def require_array(value: Any, field: str) -> list[Any]:
    if not isinstance(value, list):
        raise MigrationError("MIGRATION_INPUT", f"{field} must be an array")
    return value


def require_string_array(value: Any, field: str) -> list[str]:
    result = require_array(value, field)
    if any(not isinstance(item, str) or not item for item in result):
        raise MigrationError("MIGRATION_INPUT", f"{field} must contain nonempty strings")
    return result


def require_nonempty(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise MigrationError("MIGRATION_INPUT", f"{field} must be a nonempty string")
    return value


def require_exact_fields(
    value: dict[str, Any], expected: set[str], code: str, field: str
) -> None:
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise MigrationError(
            code, f"{field} fields differ: missing={missing!r}, extra={extra!r}"
        )


def require_unique(values: list[Any], field: str) -> None:
    if len(values) != len(set(values)):
        raise MigrationError("MIGRATION_INPUT", f"{field} must be unique")


def expect_true(condition: bool, code: str, message: str) -> None:
    if not condition:
        raise MigrationError(code, message)


def expect_equal(actual: Any, expected: Any, code: str, field: str) -> None:
    if actual != expected:
        raise MigrationError(
            code,
            f"{field} differs: expected {short(expected)}, got {short(actual)}",
        )


def short(value: Any) -> str:
    rendered = repr(value)
    return rendered if len(rendered) <= 240 else rendered[:237] + "..."


if __name__ == "__main__":
    raise SystemExit(main())
