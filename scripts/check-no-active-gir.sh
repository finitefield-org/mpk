#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  --audit)
    mode="audit"
    ;;
  --strict)
    mode="strict"
    ;;
  *)
    echo "usage: $0 --audit|--strict" >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

exec python3 - "$repo_root" "$mode" <<'PY'
from __future__ import annotations

import json
import hashlib
import os
import re
import stat
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any


REPO_ROOT = Path(sys.argv[1])
MODE = sys.argv[2]
METADATA_PATH = "develop/migrations/gir-to-vir-obsolete-terms.txt"
MANIFEST_PATH = "develop/migrations/gir-to-vir-search-fixtures/manifest.json"
FIXTURE_ROOT = "develop/migrations/gir-to-vir-search-fixtures"
SCRIPT_PATH = "scripts/check-no-active-gir.sh"
MANIFEST_SCHEMA = "mpk.gir_to_vir.search_fixtures.v1"
SCOPES = {
    "exact-global-token",
    "exact-path",
    "schema-qualified-json-field",
    "type/variant-qualified-code-symbol",
}
DISPOSITIONS = {"remove", "rename/regenerate"}
EXPECTED_SCANNER_FILES = {METADATA_PATH, SCRIPT_PATH}
EXPECTED_RETAINED_RECORDS = {
    "develop/specs/GIR_V0.md": "always",
    "develop/specs/GO_SUBSET_V0.md": "always",
    "develop/specs/AI_API_V0.md": "always",
    "develop/specs/CERT_V0.md": "always",
    "develop/specs/TRUST_BOUNDARY_V0.md": "always",
    "develop/specs/UNSAFE_POLICY_V0.md": "always",
    "develop/migrations/gir-to-vir-inventory.md": "always",
    "develop/migrations/go-gir-semantic-baseline.json": "always",
    "develop/migrations/archive/go-gir-to-vir-report.json": "always",
    "develop/migrations/archive/go-gir-to-vir-report.md": "always",
    "develop/docs/05_rust_frontend_design.md": "design_status_marker",
    "develop/docs/05_rust_frontend_design-todo.md": "design_status_marker",
}
EXPECTED_SEALED_RECORDS = {'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-operation-proofs/server-linux/final/receipt.tar.gz': ('411a8acce3b7e5ed764c0ad80b31753b80f604b574d9b4772ebb1ae0a351800e',
                                                                                                                                    4159966,
                                                                                                                                    'binary_proof_receipt',
                                                                                                                                    'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/ordinary-foundation/verification-logs/scoped-construction-proofs/server-linux/final/receipt.tar.gz': ('4c29a6b5921c5141925e243a60ee5f4a055287bfce4210be2680fac89efb1b01',
                                                                                                                                     6333306,
                                                                                                                                     'binary_proof_receipt',
                                                                                                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/capture-and-original-owner-checkpoint/completed-control-pattern-consumers-with-capture-names.json.gz': ('c2ffb8df553998247d9cd69dafc4deb6b30318bb3f6f198f7fb221949ba0679a',
                                                                                                                                                                                 6422548,
                                                                                                                                                                                 'binary_proof_receipt',
                                                                                                                                                                                 'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/native-linux/checkpoint-384/control-responses.json.gz': ('5489430948bedb01fd11377c3692b92ada93602a08864173e8e80af408cd46fd',
                                                                                                                                  3223504,
                                                                                                                                  'binary_proof_receipt',
                                                                                                                                  'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/ordinary-metadata-1/linux/mpk-w09-refreeze-ordinary-40469d29-reports.tar.gz': ('7c689086d7626c5fa43cf1bd06b1c85fb22b2677eb22d6a8296fe0050aa376e1',
                                                                                                                                                        5056572,
                                                                                                                                                        'binary_proof_receipt',
                                                                                                                                                        'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/pattern-consumer-checkpoint/completed-control-pattern-consumers-with-capture-names.json.gz': ('c2ffb8df553998247d9cd69dafc4deb6b30318bb3f6f198f7fb221949ba0679a',
                                                                                                                                                                       6422548,
                                                                                                                                                                       'binary_proof_receipt',
                                                                                                                                                                       'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/csharp-03/probes/boolean-proof-elimination-refreeze/standalone-execution-checkpoint/pattern-scope-initial-metadata-comparison.json.gz': ('3cac2976fb3ce9487b896c5ec1960a3f9237c7156279d7cfcba2106600f68f01',
                                                                                                                                                              397428,
                                                                                                                                                              'binary_proof_receipt',
                                                                                                                                                              'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/api-contracts/bundles.json': ('8c12a4b71048274a9bd9ea82760f5c750016869fff9aaa5eac8cb406093ea631',
                                                                        1589073,
                                                                        'frozen_private_fixture',
                                                                        'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/capture-artifacts/artifacts.json': ('db60b18290a603e9312ca635928e88c335e5392de723315c55e93a618423f6ac',
                                                                              1230636,
                                                                              'frozen_private_fixture',
                                                                              'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/capture-artifacts/schema.json': ('84b371003f97560b01b887c419f4aea4c9a1f0234140760e4d7946ce5fa88c59',
                                                                           6450,
                                                                           'frozen_private_fixture',
                                                                           'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/closed-carriers/fixture.json': ('5d858b7fa3b6813aa2e796ce21da4be0cbb99e6a5aaf34db16a98c056af24449',
                                                                          67943,
                                                                          'frozen_private_fixture',
                                                                          'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/closed-carriers/plain-error-types.json': ('fe0ebe2384e28057fda20f3bcb7179d79e163d2da1fe6c50b14b8a22c1a741d5',
                                                                                    51266,
                                                                                    'frozen_private_fixture',
                                                                                    'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/contract-wire/expression-witnesses.json': ('aef6d55342a2d4a3e6103f6e503f318099d03d5ea79a026b663d9b7777a16796',
                                                                                     189685,
                                                                                     'frozen_private_fixture',
                                                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/contract-wire/normalized-contracts.json': ('bb9d39fdd7c81cb0c46eee247d51edbc34d11c4e0bb9f64151583e2babe79bfb',
                                                                                     769619,
                                                                                     'frozen_private_fixture',
                                                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/requests.json': ('6b4ef593cec6b5e4087f50aeb9162d67adba7636f98ab029b5c19d6640a4c667',
                                                                             17646,
                                                                             'frozen_private_fixture',
                                                                             'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/schema.json': ('65cdddf34fecaa5ae007aea5bdfd135101d8a6e4d7635dc6542ef59a2b7336d3',
                                                                           9039,
                                                                           'frozen_private_fixture',
                                                                           'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/successes.json': ('2f477579a56fbaf927892e1758446de5630364034c93db8b0b410b876afcd6c8',
                                                                              688398,
                                                                              'frozen_private_fixture',
                                                                              'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/verification/initial-context-generation/frontend_vectors.py': ('ee0bde0df7ed178c9acff1fda4d74559ce91709b2792dbbf8b92537d0dc5961a',
                                                                                                                           14248,
                                                                                                                           'initial_source_snapshot',
                                                                                                                           'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/verification/initial-context-generation/frontend_wire.py': ('143e8d02d5047b366b72f0be889d72141149be47591cf2915436a81e77bc0969',
                                                                                                                        17027,
                                                                                                                        'initial_source_snapshot',
                                                                                                                        'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/verification/initial-request-oracle/frontend_wire.py': ('bd0fdd9d5ef9a3543e7df6a046d49200f846170775565032706225477d3fbf61',
                                                                                                                    17012,
                                                                                                                    'initial_source_snapshot',
                                                                                                                    'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/verification/initial-runner-syntax/run_frontend_wire.py': ('412835efd8c01bd1f15cac52714bfac4c028fcadb7b4be6f239079114716d4f9',
                                                                                                                       17109,
                                                                                                                       'initial_source_snapshot',
                                                                                                                       'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/frontend-envelope/verification/initial-unobserved-query/run_frontend_wire.py': ('5ecdc64512597e982a0431a7493017bc7e5da4fc311554bd9e97d85fda98a4c8',
                                                                                                                          17110,
                                                                                                                          'initial_source_snapshot',
                                                                                                                          'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/identity-dag/fixture.json': ('d680e429dbb4d5616962e57841b76abdd077a7fb9848e9c24895a3b7c645393a',
                                                                       12818,
                                                                       'frozen_private_fixture',
                                                                       'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/proof-artifacts/artifacts.json': ('5433b9436786fe8544a4cd3d41773ef4187c6d80d9a9b6570fa7821ffaa7cf21',
                                                                            1920500,
                                                                            'frozen_private_fixture',
                                                                            'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w04/shared-vir/modules.json': ('e9d10afa43b62f62ebacaeaf6888996b36c133006c661eb719520d7aa3cf4056',
                                                                     1085567,
                                                                     'frozen_private_fixture',
                                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/migrations/go-practical/t01-w05/verification/initial-vocabulary-findings/check-fast.stderr.log': ('721a858405d3615f650f9a64a4914b23b3e70baeb1442a520d237417002f9bfd',
                                                                                                            52,
                                                                                                            'failed_gate_observation',
                                                                                                            'worktree:a2d5f3a392acce5090d355706f4a9831c38ec046a549279d95bf0400e4be230f'),
 'develop/migrations/go-practical/t01-w05/verification/initial-vocabulary-findings/check-fast.stdout.log': ('c6f7191f0e928e7eed1798a0efb1af6ec1ff9b3055b8b0b1f7ddbf3ae491a030',
                                                                                                            8009,
                                                                                                            'failed_gate_observation',
                                                                                                            'worktree:a2d5f3a392acce5090d355706f4a9831c38ec046a549279d95bf0400e4be230f'),
 'develop/probes/go-practical/capture_artifacts.py': ('c7967595b04ae0b150617cd5009aecc4f12f1ef8339d5cb799b035e2beba0315',
                                                      14846,
                                                      'frozen_private_model',
                                                      'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/closed_carriers.py': ('7127bad4c5561885e97ea210a5558ba519365501fe831409c80db83f65711a34',
                                                    22305,
                                                    'frozen_private_model',
                                                    'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/contract_scope.py': ('d3a09d17e43aa43027d96867941d75a35c5cd2a5da8b82381327cebbee47c81d',
                                                   12765,
                                                   'frozen_private_model',
                                                   'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/frontend_vectors.py': ('ee0bde0df7ed178c9acff1fda4d74559ce91709b2792dbbf8b92537d0dc5961a',
                                                     14248,
                                                     'frozen_private_model',
                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/frontend_wire.py': ('0745a44af734f1f93641066e47cb8bf0ebb87afaa87718360765dc050b4acc4f',
                                                  17149,
                                                  'frozen_private_model',
                                                  'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/identity_vectors.py': ('7aa0b6d41539db6bd99403d75ceaaa20e6e1699c61d79764a19ca4d01797ab4c',
                                                     6424,
                                                     'frozen_private_model',
                                                     'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/identity_wire.py': ('0cc43b19f8132dd1ab6df2dd0551e9a4cd82c6f61a6375f7386223211a8de5fd',
                                                  12579,
                                                  'frozen_private_model',
                                                  'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/run_capture_artifacts.py': ('a5d02321d86e0e88dd5390f81b09b46327c9d7835031e9970f645282c06835f9',
                                                          17177,
                                                          'frozen_private_model',
                                                          'f1ada294ec8d81c424da4ea71627f484993db4cc'),
 'develop/probes/go-practical/run_frontend_wire.py': ('43d25f3a5feb7729815c2c5064c8a1260f31764959e5d28908c81a248fc1e974',
                                                      17302,
                                                      'frozen_private_model',
                                                      'f1ada294ec8d81c424da4ea71627f484993db4cc')}

# Immutable failed diagnostics are history, not an accepted production dialect.
EXPECTED_SEALED_RECORDS.update({'develop/migrations/go-practical/t02-w01/verification/final-input-review/changed-inputs-review.json': ('3a702ed21ab376692204a2b072053456a238eddecca0cde03ee2a9192257ea77',
                                                                                                        58326,
                                                                                                        'historical_scan_diagnostic',
                                                                                                        'worktree:0aaff41c8e1ab2ae7064a24d9c1f5bc7ca8e23e586ea252013be0925d3011d4a'),
 'develop/migrations/go-practical/t02-w01/verification/input-review/changed-inputs-review.json': ('3bd8c1a65b410b50826f5e810a08610f49a7e662480785b88ba829d4aca99a35',
                                                                                                  55689,
                                                                                                  'historical_scan_diagnostic',
                                                                                                  'worktree:1f5a0e2213557f65b50dc5c61fd88eaade5c3d7db07d479b567fd61aef7a3191')})

# Preserve a fixture-review diagnostic under the same exact-file policy.
EXPECTED_SEALED_RECORDS.update({'develop/migrations/go-practical/t02-w01/verification/reviewed-inputs/changed-inputs-review.json': ('3c1dd53a6c80539a4cd971e3e494466e367f3a3cd4975e51bc5bd72c28033ab1',
                                                                                                     60563,
                                                                                                     'historical_scan_diagnostic',
                                                                                                     'worktree:7b62d1b63b216993ff2aa9111266a353c860d246962f1d77f4bb4d1515a35cfe')})

DESIGN_PATHS = {
    path for path, activation in EXPECTED_RETAINED_RECORDS.items()
    if activation == "design_status_marker"
}


class GateError(Exception):
    pass


@dataclass(frozen=True)
class Finding:
    matcher_id: str
    owner: str
    path: str
    line: int
    excerpt: str


def fail(message: str) -> None:
    raise GateError(message)


def strict_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def read_strict_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=strict_object)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read strict JSON {path.relative_to(REPO_ROOT)}: {error}")
    if not isinstance(value, dict):
        fail(f"{path.relative_to(REPO_ROOT)} must contain one JSON object")
    return value


def normalized_relative_path(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{context} must be a nonempty string")
    if "\\" in value or value.endswith("/"):
        fail(f"{context} must be a normalized POSIX file path: {value!r}")
    parsed = PurePosixPath(value)
    if parsed.is_absolute() or value != parsed.as_posix() or ".." in parsed.parts:
        fail(f"{context} must be repository-relative and normalized: {value!r}")
    return value


def validate_regular_file(relative: str, context: str) -> Path:
    relative = normalized_relative_path(relative, context)
    current = REPO_ROOT
    for component in PurePosixPath(relative).parts:
        current = current / component
        try:
            info = current.lstat()
        except FileNotFoundError:
            fail(f"{context} is missing: {relative}")
        if stat.S_ISLNK(info.st_mode):
            fail(f"{context} must not contain a symlink component: {relative}")
    if not current.is_file():
        fail(f"{context} must name one exact regular file, not a directory: {relative}")
    return current


def require_exact_keys(value: dict[str, Any], required: set[str], optional: set[str], context: str) -> None:
    missing = required - value.keys()
    unknown = value.keys() - required - optional
    if missing:
        fail(f"{context} is missing keys: {sorted(missing)}")
    if unknown:
        fail(f"{context} has unknown keys: {sorted(unknown)}")


def load_matchers() -> list[dict[str, Any]]:
    path = validate_regular_file(METADATA_PATH, "scanner metadata")
    matchers: list[dict[str, Any]] = []
    ids: set[str] = set()
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            matcher = json.loads(line, object_pairs_hook=strict_object)
        except json.JSONDecodeError as error:
            fail(f"{METADATA_PATH}:{line_number}: invalid JSON: {error}")
        if not isinstance(matcher, dict):
            fail(f"{METADATA_PATH}:{line_number}: matcher must be an object")
        scope = matcher.get("scope")
        common = {"id", "scope", "disposition", "removal_owner"}
        if scope == "exact-global-token":
            require_exact_keys(matcher, common | {"token"}, set(), f"matcher line {line_number}")
            if not isinstance(matcher["token"], str) or not matcher["token"]:
                fail(f"{METADATA_PATH}:{line_number}: token must be nonempty")
        elif scope == "exact-path":
            require_exact_keys(matcher, common | {"path"}, set(), f"matcher line {line_number}")
            matcher["path"] = normalized_relative_path(matcher["path"], f"matcher line {line_number} path")
        elif scope == "schema-qualified-json-field":
            require_exact_keys(
                matcher,
                common | {"schemas", "field"},
                set(),
                f"matcher line {line_number}",
            )
            schemas = matcher["schemas"]
            if (
                not isinstance(schemas, list)
                or not schemas
                or any(not isinstance(item, str) or not item for item in schemas)
                or len(schemas) != len(set(schemas))
            ):
                fail(f"{METADATA_PATH}:{line_number}: schemas must be unique nonempty strings")
            if not isinstance(matcher["field"], str) or not matcher["field"]:
                fail(f"{METADATA_PATH}:{line_number}: field must be nonempty")
        elif scope == "type/variant-qualified-code-symbol":
            require_exact_keys(matcher, common | {"symbol"}, set(), f"matcher line {line_number}")
            symbol = matcher["symbol"]
            if not isinstance(symbol, str) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+", symbol):
                fail(f"{METADATA_PATH}:{line_number}: symbol must be fully qualified")
        else:
            fail(f"{METADATA_PATH}:{line_number}: unknown matcher scope {scope!r}")

        matcher_id = matcher.get("id")
        if not isinstance(matcher_id, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]*", matcher_id):
            fail(f"{METADATA_PATH}:{line_number}: invalid matcher id {matcher_id!r}")
        if matcher_id in ids:
            fail(f"{METADATA_PATH}:{line_number}: duplicate matcher id {matcher_id!r}")
        ids.add(matcher_id)
        if matcher.get("disposition") not in DISPOSITIONS:
            fail(f"{METADATA_PATH}:{line_number}: invalid disposition")
        if not re.fullmatch(r"GO-VIR-02-T12", str(matcher.get("removal_owner", ""))):
            fail(f"{METADATA_PATH}:{line_number}: every removal must be owned by GO-VIR-02-T12")
        matchers.append(matcher)
    if not matchers:
        fail(f"{METADATA_PATH} contains no matchers")
    if {matcher["scope"] for matcher in matchers} != SCOPES:
        fail("matcher metadata must exercise exactly the four frozen matcher scopes")
    return matchers


def validate_manifest() -> tuple[dict[str, Any], set[str], set[str], set[str]]:
    path = validate_regular_file(MANIFEST_PATH, "fixture manifest")
    manifest = read_strict_json(path)
    require_exact_keys(
        manifest,
        {"schema", "design_status_marker", "scanner_files", "retained_records", "sealed_records", "fixtures"},
        set(),
        "fixture manifest",
    )
    if manifest["schema"] != MANIFEST_SCHEMA:
        fail(f"fixture manifest schema must be {MANIFEST_SCHEMA!r}")
    marker = manifest["design_status_marker"]
    if not isinstance(marker, str) or not marker:
        fail("design_status_marker must be nonempty")

    scanner_values = manifest["scanner_files"]
    if not isinstance(scanner_values, list):
        fail("scanner_files must be an array")
    scanner_files = {
        normalized_relative_path(value, "scanner_files entry") for value in scanner_values
    }
    if len(scanner_files) != len(scanner_values):
        fail("scanner_files contains a duplicate")
    if scanner_files != EXPECTED_SCANNER_FILES:
        fail(
            "scanner_files must be the exact frozen scanner implementation/metadata set; "
            f"got {sorted(scanner_files)}"
        )
    for relative in scanner_files:
        validate_regular_file(relative, "scanner exclusion")

    retained_values = manifest["retained_records"]
    if not isinstance(retained_values, list):
        fail("retained_records must be an array")
    retained: dict[str, str] = {}
    for index, item in enumerate(retained_values):
        if not isinstance(item, dict):
            fail(f"retained_records[{index}] must be an object")
        require_exact_keys(item, {"path", "activation"}, set(), f"retained_records[{index}]")
        relative = normalized_relative_path(item["path"], f"retained_records[{index}].path")
        activation = item["activation"]
        if activation not in {"always", "design_status_marker"}:
            fail(f"retained_records[{index}] has unknown activation {activation!r}")
        if relative in retained:
            fail(f"retained_records contains duplicate path {relative!r}")
        retained[relative] = activation
        validate_regular_file(relative, "retained record")
    if retained != EXPECTED_RETAINED_RECORDS:
        fail("retained_records must equal the frozen exact-file exclusion set")

    sealed_values = manifest["sealed_records"]
    if not isinstance(sealed_values, list):
        fail("sealed_records must be an array")
    sealed: set[str] = set()
    for index, item in enumerate(sealed_values):
        context = f"sealed_records[{index}]"
        if not isinstance(item, dict):
            fail(f"{context} must be an object")
        require_exact_keys(item, {"path", "sha256", "size_bytes", "classification", "source_epoch", "reason"}, set(), context)
        relative = normalized_relative_path(item["path"], context + ".path")
        if relative in sealed or relative not in EXPECTED_SEALED_RECORDS:
            fail(f"{context} has a duplicate or unreviewed historical path")
        if not relative.startswith(("develop/probes/", "develop/migrations/")):
            fail(f"{context} cannot exclude production files")
        if not isinstance(item["reason"], str) or not item["reason"]:
            fail(f"{context} requires an explicit retention reason")
        if not isinstance(item["source_epoch"], str) or not re.fullmatch(r"(?:[0-9a-f]{40}|worktree:[0-9a-f]{64})", item["source_epoch"]):
            fail(f"{context} requires an exact source epoch")
        expected_hash, expected_size, expected_class, expected_epoch = EXPECTED_SEALED_RECORDS[relative]
        if type(item["size_bytes"]) is not int or (item["sha256"], item["size_bytes"], item["classification"], item["source_epoch"]) != (expected_hash, expected_size, expected_class, expected_epoch):
            fail(f"{context} does not match the reviewed historical seal")
        historical = validate_regular_file(relative, "sealed historical record")
        raw = historical.read_bytes()
        if len(raw) != expected_size or hashlib.sha256(raw).hexdigest() != expected_hash:
            fail(f"{context} historical bytes changed")
        if expected_class == "binary_proof_receipt" and not raw.startswith(b"\x1f\x8b"):
            fail(f"{context} is not the sealed gzip evidence carrier")
        sealed.add(relative)
    if sealed != set(EXPECTED_SEALED_RECORDS):
        fail("sealed_records must equal the reviewed exact-file set")

    fixture_values = manifest["fixtures"]
    if not isinstance(fixture_values, list):
        fail("fixtures must be an array")
    fixtures: set[str] = set()
    manifest_roles = 0
    for index, item in enumerate(fixture_values):
        if not isinstance(item, dict):
            fail(f"fixtures[{index}] must be an object")
        role = item.get("role")
        if role == "manifest":
            require_exact_keys(item, {"path", "role"}, set(), f"fixtures[{index}]")
            manifest_roles += 1
        elif role == "positive":
            require_exact_keys(
                item,
                {"path", "role", "expected_matchers"},
                {"virtual_path"},
                f"fixtures[{index}]",
            )
        elif role == "negative":
            require_exact_keys(
                item,
                {"path", "role"},
                {"virtual_path", "guards"},
                f"fixtures[{index}]",
            )
        else:
            fail(f"fixtures[{index}] has unknown role {role!r}")
        relative = normalized_relative_path(item["path"], f"fixtures[{index}].path")
        if relative in fixtures:
            fail(f"fixtures contains duplicate path {relative!r}")
        if not relative.startswith(FIXTURE_ROOT + "/"):
            fail(f"fixture is outside exact fixture root: {relative}")
        fixtures.add(relative)
        validate_regular_file(relative, "fixture exclusion")
        if "virtual_path" in item:
            normalized_relative_path(item["virtual_path"], f"fixtures[{index}].virtual_path")
    if manifest_roles != 1 or MANIFEST_PATH not in fixtures:
        fail("fixtures must enumerate the manifest itself exactly once")

    actual_fixture_files: set[str] = set()
    root = REPO_ROOT / FIXTURE_ROOT
    for current, directory_names, file_names in os.walk(root, followlinks=False):
        current_path = Path(current)
        for name in directory_names:
            candidate = current_path / name
            if candidate.is_symlink():
                fail(f"fixture root contains symlinked directory: {candidate.relative_to(REPO_ROOT)}")
        for name in file_names:
            candidate = current_path / name
            relative = candidate.relative_to(REPO_ROOT).as_posix()
            validate_regular_file(relative, "fixture-root file")
            actual_fixture_files.add(relative)
    if fixtures != actual_fixture_files:
        fail(
            "fixture manifest must enumerate every fixture-root regular file exactly once; "
            f"missing={sorted(actual_fixture_files - fixtures)}, unknown={sorted(fixtures - actual_fixture_files)}"
        )

    retained_files = set(retained)
    classes = {
        "scanner": scanner_files,
        "fixtures": fixtures,
        "retained": retained_files,
        "sealed": sealed,
    }
    names = list(classes)
    for index, left_name in enumerate(names):
        for right_name in names[index + 1:]:
            overlap = classes[left_name] & classes[right_name]
            if overlap:
                fail(f"exclusion classes {left_name}/{right_name} overlap: {sorted(overlap)}")
    return manifest, scanner_files, fixtures, retained_files | sealed


def token_pattern(token: str) -> re.Pattern[str]:
    prefix = r"(?<![A-Za-z0-9_])" if re.match(r"[A-Za-z0-9_]", token[0]) else ""
    suffix = r"(?![A-Za-z0-9_])" if re.match(r"[A-Za-z0-9_]", token[-1]) else ""
    return re.compile(prefix + re.escape(token) + suffix)


def symbol_pattern(symbol: str) -> re.Pattern[str]:
    components = symbol.split("::")
    body = r"\s*::\s*".join(re.escape(component) for component in components)
    return re.compile(r"(?<![A-Za-z0-9_])" + body + r"(?![A-Za-z0-9_])")


def path_matches(candidate: str, obsolete: str) -> bool:
    return candidate == obsolete or candidate.startswith(obsolete + "/")


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def excerpt_for_line(text: str, line: int) -> str:
    if line <= 0:
        return "path match"
    lines = text.splitlines()
    if line > len(lines):
        return "content match"
    value = " ".join(lines[line - 1].strip().split())
    return value[:160]


def object_has_qualified_field(value: Any, schemas: set[str], field: str, inherited: set[str] | None = None) -> bool:
    active = set() if inherited is None else set(inherited)
    if isinstance(value, dict):
        for schema_key in ("schema", "schema_version"):
            schema_value = value.get(schema_key)
            if isinstance(schema_value, str):
                active.add(schema_value)
        if field in value and active & schemas:
            return True
        return any(object_has_qualified_field(child, schemas, field, active) for child in value.values())
    if isinstance(value, list):
        return any(object_has_qualified_field(child, schemas, field, active) for child in value)
    return False


def detect(relative: str, text: str, matchers: list[dict[str, Any]]) -> list[Finding]:
    findings: list[Finding] = []
    parsed_json: Any = None
    parsed_json_available = False
    stripped = text.lstrip()
    if stripped.startswith("{") or stripped.startswith("["):
        try:
            parsed_json = json.loads(text)
            parsed_json_available = True
        except json.JSONDecodeError:
            pass

    for matcher in matchers:
        matcher_id = matcher["id"]
        owner = matcher["removal_owner"]
        scope = matcher["scope"]
        offsets: list[int] = []
        if scope == "exact-global-token":
            token = matcher["token"]
            if token in text:
                offsets = [match.start() for match in token_pattern(token).finditer(text)]
        elif scope == "exact-path":
            obsolete = matcher["path"]
            if path_matches(relative, obsolete):
                findings.append(Finding(matcher_id, owner, relative, 0, "path match"))
            if obsolete in text:
                content_pattern = re.compile(
                    r"(?<![A-Za-z0-9._/-])" + re.escape(obsolete) + r"(?![A-Za-z0-9._/-])"
                )
                offsets = [match.start() for match in content_pattern.finditer(text)]
        elif scope == "type/variant-qualified-code-symbol":
            symbol = matcher["symbol"]
            # Separators may contain whitespace, but its first component must
            # still occur literally. JSON-qualified fields are handled below
            # after decoding, including escaped keys and schema strings.
            if symbol.split("::", 1)[0] in text:
                offsets = [match.start() for match in symbol_pattern(symbol).finditer(text)]
        elif scope == "schema-qualified-json-field":
            schemas = set(matcher["schemas"])
            field = matcher["field"]
            qualified = (
                parsed_json_available
                # Without escapes, decoded schema/key strings must also occur
                # literally. Escaped JSON always takes the structured check.
                and ("\\" in text or field in text and any(schema in text for schema in schemas))
                and object_has_qualified_field(parsed_json, schemas, field)
            )
            if not parsed_json_available:
                qualified = field in text and any(
                    schema in text and token_pattern(schema).search(text) for schema in schemas
                ) and bool(token_pattern(field).search(text))
            if qualified:
                field_match = token_pattern(field).search(text)
                offsets = [field_match.start() if field_match else 0]
        for offset in offsets:
            line = line_for_offset(text, offset)
            findings.append(
                Finding(matcher_id, owner, relative, line, excerpt_for_line(text, line))
            )
    return findings


def run_fixture_self_test(manifest: dict[str, Any], matchers: list[dict[str, Any]]) -> None:
    matcher_by_id = {matcher["id"]: matcher for matcher in matchers}
    positive_scopes: set[str] = set()
    positive_context_ids: set[str] = set()
    negative_context_ids: set[str] = set()
    for index, fixture in enumerate(manifest["fixtures"]):
        role = fixture["role"]
        if role == "manifest":
            continue
        relative = fixture["path"]
        virtual_path = fixture.get("virtual_path", relative)
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        found = {finding.matcher_id for finding in detect(virtual_path, text, matchers)}
        if role == "positive":
            expected_values = fixture["expected_matchers"]
            if (
                not isinstance(expected_values, list)
                or not expected_values
                or any(not isinstance(value, str) for value in expected_values)
                or len(expected_values) != len(set(expected_values))
            ):
                fail(f"fixtures[{index}].expected_matchers must be unique matcher ids")
            expected = set(expected_values)
            unknown = expected - matcher_by_id.keys()
            if unknown:
                fail(f"positive fixture {relative} references unknown matchers: {sorted(unknown)}")
            if found != expected:
                fail(
                    f"positive fixture {relative} mismatch: expected={sorted(expected)}, found={sorted(found)}"
                )
            positive_scopes.update(matcher_by_id[matcher_id]["scope"] for matcher_id in expected)
            positive_context_ids.update(
                matcher_id for matcher_id in expected
                if matcher_by_id[matcher_id]["scope"]
                in {"schema-qualified-json-field", "type/variant-qualified-code-symbol"}
            )
        else:
            if found:
                fail(f"negative fixture {relative} was detected by {sorted(found)}")
            guards = fixture.get("guards", [])
            if (
                not isinstance(guards, list)
                or any(not isinstance(value, str) for value in guards)
                or len(guards) != len(set(guards))
            ):
                fail(f"negative fixture {relative} guards must be unique matcher ids")
            unknown = set(guards) - matcher_by_id.keys()
            if unknown:
                fail(f"negative fixture {relative} guards unknown matchers: {sorted(unknown)}")
            for matcher_id in guards:
                if matcher_by_id[matcher_id]["scope"] not in {
                    "schema-qualified-json-field",
                    "type/variant-qualified-code-symbol",
                }:
                    fail(f"negative guard {matcher_id} is not context-qualified")
            negative_context_ids.update(guards)
    if positive_scopes != SCOPES:
        fail(f"positive fixtures must detect every matcher scope; got {sorted(positive_scopes)}")
    contextual = {
        matcher["id"] for matcher in matchers
        if matcher["scope"]
        in {"schema-qualified-json-field", "type/variant-qualified-code-symbol"}
    }
    if positive_context_ids != contextual or negative_context_ids != contextual:
        fail(
            "every context-qualified matcher must have positive and retained-context negative coverage; "
            f"positive_missing={sorted(contextual - positive_context_ids)}, "
            f"negative_missing={sorted(contextual - negative_context_ids)}"
        )


def active_retained_record_exclusions(manifest: dict[str, Any]) -> set[str]:
    marker = manifest["design_status_marker"]
    design_ready = all(marker in (REPO_ROOT / path).read_text(encoding="utf-8") for path in DESIGN_PATHS)
    active: set[str] = set()
    for item in manifest["retained_records"]:
        if item["activation"] == "always" or design_ready:
            active.add(item["path"])
    active.update(item["path"] for item in manifest["sealed_records"])
    return active


def repository_files() -> list[str]:
    command = [
        "/usr/bin/git",
        "-C",
        str(REPO_ROOT),
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "-z",
    ]
    try:
        result = subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"cannot enumerate repository files with /usr/bin/git: {error}")
    try:
        paths = [entry.decode("utf-8") for entry in result.stdout.split(b"\0") if entry]
    except UnicodeDecodeError as error:
        fail(f"git returned a non-UTF-8 repository path: {error}")
    if len(paths) != len(set(paths)):
        fail("git file enumeration returned duplicate paths")
    return sorted(paths)


def scan_repository(
    matchers: list[dict[str, Any]], excluded: set[str]
) -> tuple[list[Finding], int]:
    findings: list[Finding] = []
    scanned = 0
    for relative in repository_files():
        relative = normalized_relative_path(relative, "git-enumerated path")
        if relative in excluded:
            continue
        try:
            (REPO_ROOT / relative).lstat()
        except FileNotFoundError:
            # The index still lists a tracked file deleted by an unstaged
            # cutover change. A deleted file is not active scan input.
            continue
        path = validate_regular_file(relative, "active repository file")
        scanned += 1
        try:
            raw = path.read_bytes()
        except OSError as error:
            fail(f"cannot read active repository file {relative}: {error}")
        text = raw.decode("utf-8", errors="ignore")
        findings.extend(detect(relative, text, matchers))
    return findings, scanned


def report(findings: list[Finding], scanned: int, retained_count: int) -> int:
    print(
        f"Obsolete-interface self-test: ok "
        f"({scanned} active files scanned; {retained_count} retained records excluded)"
    )
    if not findings:
        print("No active obsolete interfaces found.")
        return 0

    grouped: dict[tuple[str, str, str], list[Finding]] = defaultdict(list)
    for finding in findings:
        grouped[(finding.matcher_id, finding.owner, finding.path)].append(finding)
    print(
        f"Found {len(findings)} obsolete-interface occurrences in "
        f"{len({finding.path for finding in findings})} active files."
    )
    for (matcher_id, owner, path), values in sorted(grouped.items()):
        first = min(values, key=lambda value: value.line)
        location = path if first.line == 0 else f"{path}:{first.line}"
        print(
            f"{matcher_id}\towner={owner}\tcount={len(values)}\t{location}\t{first.excerpt}"
        )
    if MODE == "audit":
        print("Audit mode records findings without failing; strict mode must fail before cutover.")
        return 0
    print("Strict mode rejects active obsolete interfaces.", file=sys.stderr)
    return 1


def main() -> int:
    try:
        matchers = load_matchers()
        manifest, scanner_files, fixtures, _retained = validate_manifest()
        run_fixture_self_test(manifest, matchers)
        retained = active_retained_record_exclusions(manifest)
        excluded = scanner_files | fixtures | retained
        findings, scanned = scan_repository(matchers, excluded)
        return report(findings, scanned, len(retained))
    except GateError as error:
        print(f"Obsolete-interface gate configuration error: {error}", file=sys.stderr)
        return 2


raise SystemExit(main())
PY
