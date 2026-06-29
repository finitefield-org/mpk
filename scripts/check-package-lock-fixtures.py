#!/usr/bin/env python3
"""Validate package lock fixtures against package-lock.md."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path
from typing import Any


sys.dont_write_bytecode = True

SCHEMA = "mpk.package.lock.v0"
HASH_RE = re.compile(r"^[0-9a-f]{64}$")
NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_']*(\.[A-Za-z_][A-Za-z0-9_']*)*$")
TOP_LEVEL_FIELDS = {
    "schema",
    "manifest",
    "module",
    "locked_imports",
    "checker_policy",
}
MANIFEST_REF_FIELDS = {"path", "sha256"}
LOCKED_IMPORT_FIELDS = {"module", "export_hash", "certificate_hash"}


class LockError(Exception):
    pass


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    manifest_validator = load_manifest_validator(repo_root)
    fixture_root = repo_root / "fixtures" / "package-lock"
    valid = sorted((fixture_root / "valid").glob("*.json"))
    invalid = sorted((fixture_root / "invalid").glob("*.json"))

    if not valid:
        raise LockError("missing valid package lock fixtures")
    if not invalid:
        raise LockError("missing invalid package lock fixtures")

    for path in valid:
        validate_lock(path, repo_root, manifest_validator)
        print(f"accepted valid lock fixture: {path.relative_to(repo_root)}")

    for path in invalid:
        try:
            validate_lock(path, repo_root, manifest_validator)
        except LockError as error:
            print(f"rejected invalid lock fixture: {path.relative_to(repo_root)}: {error}")
            continue
        raise LockError(f"invalid lock fixture unexpectedly accepted: {path}")

    return 0


def load_manifest_validator(repo_root: Path) -> Any:
    validator_path = repo_root / "scripts" / "check-package-manifest-fixtures.py"
    spec = importlib.util.spec_from_file_location(
        "mpk_package_manifest_fixture_validator", validator_path
    )
    if spec is None or spec.loader is None:
        raise LockError(f"failed to load manifest validator: {validator_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_lock(path: Path, repo_root: Path, manifest_validator: Any) -> None:
    lock = read_json_object(path)
    require_exact_fields(lock, TOP_LEVEL_FIELDS, "lock")
    require_equal(lock["schema"], SCHEMA, "schema")
    module = require_name(lock["module"], "module")

    manifest_path = validate_manifest_ref(lock["manifest"], repo_root)
    try:
        manifest_validator.validate_manifest(manifest_path, repo_root)
    except manifest_validator.ManifestError as error:
        raise LockError(f"manifest reference is invalid: {error}") from error
    manifest = manifest_validator.read_json_object(manifest_path)

    require_equal(manifest["module"], module, "module")
    validate_locked_imports(lock["locked_imports"])
    require_equal(
        lock["locked_imports"],
        expected_locked_imports(manifest["imports"]),
        "locked_imports",
    )
    require_equal(lock["checker_policy"], manifest["policy"], "checker_policy")


def read_json_object(path: Path) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise LockError(f"duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        with path.open("r", encoding="utf-8") as handle:
            value = json.load(handle, object_pairs_hook=reject_duplicates)
    except json.JSONDecodeError as error:
        raise LockError(f"{path}: invalid JSON: {error}") from error

    if not isinstance(value, dict):
        raise LockError(f"{path}: lock must be a JSON object")
    return value


def validate_manifest_ref(manifest_ref: Any, repo_root: Path) -> Path:
    if not isinstance(manifest_ref, dict):
        raise LockError("manifest must be an object")
    require_exact_fields(manifest_ref, MANIFEST_REF_FIELDS, "manifest")

    manifest_path = require_json_path(manifest_ref["path"], "manifest.path")
    expected_sha256 = require_hash(manifest_ref["sha256"], "manifest.sha256")
    resolved_path = resolve_package_path(repo_root, manifest_path, "manifest.path")
    actual_sha256 = hashlib.sha256(resolved_path.read_bytes()).hexdigest()
    require_equal(actual_sha256, expected_sha256, "manifest.sha256")
    return resolved_path


def validate_locked_imports(imports: Any) -> None:
    if not isinstance(imports, list):
        raise LockError("locked_imports must be a list")

    previous_key: tuple[str, str, str] | None = None
    seen_identity: set[tuple[str, str]] = set()
    for index, entry in enumerate(imports):
        field = f"locked_imports[{index}]"
        if not isinstance(entry, dict):
            raise LockError(f"{field} must be an object")
        require_exact_fields(entry, LOCKED_IMPORT_FIELDS, field)
        module = require_name(entry["module"], f"{field}.module")
        export_hash = require_hash(entry["export_hash"], f"{field}.export_hash")
        certificate_hash = require_hash(
            entry["certificate_hash"], f"{field}.certificate_hash"
        )

        identity = (module, export_hash)
        if identity in seen_identity:
            raise LockError(f"{field} duplicates import {module}:{export_hash}")
        seen_identity.add(identity)

        key = (module, export_hash, certificate_hash)
        if previous_key is not None and key <= previous_key:
            raise LockError(f"{field} is not in canonical import order")
        previous_key = key


def expected_locked_imports(imports: Any) -> list[dict[str, str]]:
    expected = []
    for index, entry in enumerate(imports):
        if "certificate_hash" not in entry:
            raise LockError(
                f"manifest imports[{index}] must include certificate_hash for locking"
            )
        expected.append(
            {
                "module": entry["module"],
                "export_hash": entry["export_hash"],
                "certificate_hash": entry["certificate_hash"],
            }
        )
    return expected


def require_exact_fields(obj: dict[str, Any], expected: set[str], field: str) -> None:
    unknown = sorted(set(obj).difference(expected))
    if unknown:
        raise LockError(f"{field} has unknown fields: {', '.join(unknown)}")
    missing = sorted(expected.difference(obj))
    if missing:
        raise LockError(f"{field} missing required fields: {', '.join(missing)}")


def require_equal(actual: Any, expected: Any, field: str) -> None:
    if actual != expected:
        raise LockError(f"{field} = {actual!r}, want {expected!r}")


def require_name(value: Any, field: str) -> str:
    if not isinstance(value, str) or not NAME_RE.fullmatch(value):
        raise LockError(f"{field} must be a canonical MPK name")
    return value


def require_hash(value: Any, field: str) -> str:
    if not isinstance(value, str) or not HASH_RE.fullmatch(value):
        raise LockError(f"{field} must be lowercase 64-character hex")
    if value == "0" * 64:
        raise LockError(f"{field} must not be all zeroes")
    return value


def require_json_path(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise LockError(f"{field} must be a nonempty string")
    if value.startswith("/") or "\\" in value:
        raise LockError(f"{field} must be a package-root relative POSIX path")
    parts = value.split("/")
    if any(part in {"", ".", ".."} for part in parts):
        raise LockError(f"{field} must not contain empty, ., or .. components")
    if not value.endswith(".json"):
        raise LockError(f"{field} must point to .json")
    return value


def resolve_package_path(repo_root: Path, manifest_path: str, field: str) -> Path:
    candidate = (repo_root / manifest_path).resolve()
    try:
        candidate.relative_to(repo_root.resolve())
    except ValueError as error:
        raise LockError(f"{field} escapes the package root") from error
    if not candidate.is_file():
        raise LockError(f"{field} does not exist: {manifest_path}")
    return candidate


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except LockError as error:
        print(f"package lock fixture check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
