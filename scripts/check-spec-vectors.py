#!/usr/bin/env python3
"""Validate the closed normative specification-vector inventory without writes."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import sys
from decimal import Decimal
from pathlib import Path, PurePosixPath
from typing import Any


MANIFEST_SCHEMA = "mpk.spec.vector_manifest.v0"
MANIFEST_PATH = "develop/specs/vectors/manifest.json"
VECTOR_PREFIX = ("develop", "specs", "vectors")
SPEC_PREFIX = ("develop", "specs")
OWNER_ROOTS = {"crates", "go-tools", "rust-tools", "scripts"}
MANIFEST_FIELDS = {"schema", "vectors"}
ENTRY_FIELDS = {
    "schema_id",
    "path",
    "sha256",
    "owning_spec",
    "implementation_test_owners",
}
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
SCHEMA_RE = re.compile(r"^mpk(?:\.[a-z0-9_]+)+\.v[0-9]+$")
PORTABLE_PATH_RE = re.compile(r"^[A-Za-z0-9._/-]+$")


class VectorManifestError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser(
        description="check the normative specification-vector manifest"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate only; no update or rewrite mode exists",
    )
    args = parser.parse_args()
    if not args.check:
        parser.error("--check is required")

    repo_root = Path(__file__).resolve().parent.parent
    entry_count = check_manifest(repo_root)
    print(f"spec vector check passed: {entry_count} vector sets")
    return 0


def check_manifest(repo_root: Path) -> int:
    manifest_path = repo_root / MANIFEST_PATH
    require_regular_repo_file(manifest_path, repo_root, "manifest")
    manifest = read_json_object(manifest_path, MANIFEST_PATH)
    require_exact_fields(manifest, MANIFEST_FIELDS, "manifest")
    require_equal(manifest["schema"], MANIFEST_SCHEMA, "manifest.schema")

    entries = manifest["vectors"]
    if not isinstance(entries, list) or not entries:
        raise VectorManifestError("manifest.vectors must be a nonempty array")

    declared_paths: set[str] = set()
    schema_ids: set[str] = set()
    ordered_paths: list[str] = []

    for index, value in enumerate(entries):
        field = f"manifest.vectors[{index}]"
        if not isinstance(value, dict):
            raise VectorManifestError(f"{field} must be an object")
        require_exact_fields(value, ENTRY_FIELDS, field)

        schema_id = require_schema_id(value["schema_id"], f"{field}.schema_id")
        vector_path = require_path(
            value["path"], f"{field}.path", required_prefix=VECTOR_PREFIX
        )
        if vector_path == MANIFEST_PATH:
            raise VectorManifestError(f"{field}.path must not declare the manifest")
        if not vector_path.endswith(".json"):
            raise VectorManifestError(f"{field}.path must name a JSON vector file")
        digest = require_sha256(value["sha256"], f"{field}.sha256")
        owning_spec = require_path(
            value["owning_spec"],
            f"{field}.owning_spec",
            required_prefix=SPEC_PREFIX,
        )
        if not owning_spec.endswith(".md"):
            raise VectorManifestError(f"{field}.owning_spec must name a Markdown file")
        owners = require_owner_list(
            value["implementation_test_owners"],
            f"{field}.implementation_test_owners",
        )

        if vector_path in declared_paths:
            raise VectorManifestError(f"{field}.path duplicates {vector_path}")
        declared_paths.add(vector_path)
        ordered_paths.append(vector_path)
        if schema_id in schema_ids:
            raise VectorManifestError(f"{field}.schema_id duplicates {schema_id}")
        schema_ids.add(schema_id)

        checked_vector = repo_root / vector_path
        require_regular_repo_file(checked_vector, repo_root, f"{field}.path")
        vector_bytes = read_bytes(checked_vector, vector_path)
        actual_digest = hashlib.sha256(vector_bytes).hexdigest()
        require_equal(actual_digest, digest, f"{field}.sha256")

        vector = parse_json_object(vector_bytes, vector_path)
        require_equal(vector.get("schema"), schema_id, f"{vector_path}.schema")
        require_equal(
            vector_test_owners(vector, vector_path),
            owners,
            f"{field}.implementation_test_owners",
        )

        checked_spec = repo_root / owning_spec
        require_regular_repo_file(checked_spec, repo_root, f"{field}.owning_spec")
        spec_text = read_utf8(checked_spec, owning_spec)
        if vector_path not in spec_text:
            raise VectorManifestError(
                f"{field}.owning_spec does not name {vector_path}"
            )
        if schema_id not in spec_text:
            raise VectorManifestError(
                f"{field}.owning_spec does not own schema {schema_id}"
            )
        for owner in owners:
            if owner not in spec_text:
                raise VectorManifestError(
                    f"{field}.owning_spec does not name test owner {owner}"
                )

    if ordered_paths != sorted(ordered_paths):
        raise VectorManifestError(
            "manifest.vectors must be ordered by path using code-point order"
        )

    discovered_paths = discover_vector_files(repo_root)
    missing = sorted(declared_paths - discovered_paths)
    extra = sorted(discovered_paths - declared_paths)
    if missing:
        raise VectorManifestError(
            f"manifest declares missing vector files: {', '.join(missing)}"
        )
    if extra:
        raise VectorManifestError(
            f"unlisted vector files exist: {', '.join(extra)}"
        )

    return len(entries)


def read_json_object(path: Path, label: str) -> dict[str, Any]:
    return parse_json_object(read_bytes(path, label), label)


def parse_json_object(data: bytes, label: str) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise VectorManifestError(f"{label}: duplicate JSON name {key!r}")
            result[key] = value
        return result

    def reject_constant(value: str) -> None:
        raise VectorManifestError(f"{label}: non-JSON numeric token {value!r}")

    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise VectorManifestError(f"{label}: invalid UTF-8: {error}") from error
    try:
        value = json.loads(
            text,
            object_pairs_hook=reject_duplicates,
            parse_constant=reject_constant,
            parse_float=Decimal,
        )
    except json.JSONDecodeError as error:
        raise VectorManifestError(f"{label}: invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise VectorManifestError(f"{label}: root must be an object")
    validate_unicode_scalars(value, label)
    return value


def validate_unicode_scalars(value: Any, field: str) -> None:
    if isinstance(value, str):
        if any(0xD800 <= ord(character) <= 0xDFFF for character in value):
            raise VectorManifestError(f"{field}: string contains a lone surrogate")
        return
    if isinstance(value, list):
        for index, item in enumerate(value):
            validate_unicode_scalars(item, f"{field}[{index}]")
        return
    if isinstance(value, dict):
        for key, item in value.items():
            validate_unicode_scalars(key, f"{field}.<object-name>")
            validate_unicode_scalars(item, f"{field}.{key}")


def read_utf8(path: Path, label: str) -> str:
    try:
        return read_bytes(path, label).decode("utf-8")
    except UnicodeDecodeError as error:
        raise VectorManifestError(f"{label}: invalid UTF-8: {error}") from error


def read_bytes(path: Path, label: str) -> bytes:
    try:
        return path.read_bytes()
    except OSError as error:
        raise VectorManifestError(f"{label}: cannot read: {error}") from error


def vector_test_owners(vector: dict[str, Any], label: str) -> list[str]:
    has_single = "owner_test" in vector
    has_multiple = "owner_tests" in vector
    if has_single == has_multiple:
        raise VectorManifestError(
            f"{label}: requires exactly one of owner_test or owner_tests"
        )
    if has_single:
        return [
            require_owner_path(vector["owner_test"], f"{label}.owner_test")
        ]
    return require_owner_list(vector["owner_tests"], f"{label}.owner_tests")


def require_owner_list(value: Any, field: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise VectorManifestError(f"{field} must be a nonempty array")
    owners = [
        require_owner_path(owner, f"{field}[{index}]")
        for index, owner in enumerate(value)
    ]
    if len(set(owners)) != len(owners):
        raise VectorManifestError(f"{field} contains a duplicate owner")
    return owners


def require_owner_path(value: Any, field: str) -> str:
    owner = require_path(value, field)
    parts = PurePosixPath(owner).parts
    if parts[0] not in OWNER_ROOTS:
        raise VectorManifestError(
            f"{field} must stay under one of {', '.join(sorted(OWNER_ROOTS))}"
        )
    if not owner.endswith((".rs", ".go", ".sh")):
        raise VectorManifestError(f"{field} must name a Rust, Go, or shell test owner")
    return owner


def require_path(
    value: Any,
    field: str,
    *,
    required_prefix: tuple[str, ...] | None = None,
) -> str:
    if not isinstance(value, str) or not value:
        raise VectorManifestError(f"{field} must be a nonempty string")
    if not PORTABLE_PATH_RE.fullmatch(value):
        raise VectorManifestError(f"{field} must use a portable POSIX path")
    path = PurePosixPath(value)
    parts = path.parts
    if path.is_absolute() or not parts or any(part in {"", ".", ".."} for part in parts):
        raise VectorManifestError(f"{field} must be a normalized repository path")
    if path.as_posix() != value:
        raise VectorManifestError(f"{field} must be lexically normalized")
    if required_prefix is not None and parts[: len(required_prefix)] != required_prefix:
        prefix = "/".join(required_prefix)
        raise VectorManifestError(f"{field} must stay under {prefix}")
    return value


def require_regular_repo_file(path: Path, repo_root: Path, field: str) -> None:
    try:
        relative = path.relative_to(repo_root)
    except ValueError as error:
        raise VectorManifestError(f"{field} escapes the repository") from error

    current = repo_root
    for part in relative.parts:
        current = current / part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError as error:
            raise VectorManifestError(f"{field} does not exist: {relative}") from error
        except OSError as error:
            raise VectorManifestError(f"{field} cannot be inspected: {error}") from error
        if stat.S_ISLNK(mode):
            raise VectorManifestError(f"{field} must not traverse a symbolic link")
    if not stat.S_ISREG(mode):
        raise VectorManifestError(f"{field} must be a regular file")


def discover_vector_files(repo_root: Path) -> set[str]:
    vector_root = repo_root.joinpath(*VECTOR_PREFIX)
    discovered: set[str] = set()
    for path in vector_root.rglob("*"):
        relative = path.relative_to(repo_root).as_posix()
        try:
            mode = path.lstat().st_mode
        except OSError as error:
            raise VectorManifestError(
                f"cannot inspect vector-directory entry {relative}: {error}"
            ) from error
        if stat.S_ISLNK(mode):
            raise VectorManifestError(
                f"vector directory must not contain symbolic links: {relative}"
            )
        if stat.S_ISDIR(mode):
            continue
        if not stat.S_ISREG(mode):
            raise VectorManifestError(
                f"vector directory contains a non-regular file: {relative}"
            )
        if relative != MANIFEST_PATH:
            discovered.add(relative)
    return discovered


def require_exact_fields(
    value: dict[str, Any], expected: set[str], field: str
) -> None:
    actual = set(value)
    missing = sorted(expected - actual)
    unknown = sorted(actual - expected)
    if missing:
        raise VectorManifestError(
            f"{field} missing required fields: {', '.join(missing)}"
        )
    if unknown:
        raise VectorManifestError(
            f"{field} has unknown fields: {', '.join(unknown)}"
        )


def require_schema_id(value: Any, field: str) -> str:
    if not isinstance(value, str) or not SCHEMA_RE.fullmatch(value):
        raise VectorManifestError(f"{field} must be a canonical MPK schema ID")
    return value


def require_sha256(value: Any, field: str) -> str:
    if not isinstance(value, str) or not SHA256_RE.fullmatch(value):
        raise VectorManifestError(f"{field} must be lowercase SHA-256")
    return value


def require_equal(actual: Any, expected: Any, field: str) -> None:
    if actual != expected:
        raise VectorManifestError(
            f"{field} mismatch: got {actual!r}, expected {expected!r}"
        )


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except VectorManifestError as error:
        print(f"spec vector check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
