#!/usr/bin/env python3
"""Validate package manifest fixtures against package-manifest.md."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


SCHEMA = "mpk.package.v0"
HASH_RE = re.compile(r"^[0-9a-f]{64}$")
NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_']*(\.[A-Za-z_][A-Za-z0-9_']*)*$")
CHECKER_PROFILES = {"core-bootstrap", "mvp-structural", "mvp-strict"}
AXIOM_PROFILES = {
    "zero-axiom",
    "core-mvp",
    "mvp-theory",
    "go-artifact-alpha",
    "experimental-external",
}
TOP_LEVEL_FIELDS = {"schema", "module", "imports", "certificates", "policy"}
IMPORT_FIELDS = {"module", "export_hash", "certificate_hash"}
CERTIFICATE_FIELDS = {
    "module",
    "path",
    "expected_export_hash",
    "expected_axiom_report_hash",
    "expected_certificate_hash",
}
POLICY_FIELDS = {
    "checker_profile",
    "allowed_axiom_profiles",
    "require_reference_checker",
    "require_source_free_check",
}


class ManifestError(Exception):
    pass


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    fixture_root = repo_root / "fixtures" / "package-manifest"
    valid = sorted((fixture_root / "valid").glob("*.json"))
    invalid = sorted((fixture_root / "invalid").glob("*.json"))

    if not valid:
        raise ManifestError("missing valid package manifest fixtures")
    if not invalid:
        raise ManifestError("missing invalid package manifest fixtures")

    for path in valid:
        validate_manifest(path, repo_root)
        print(f"accepted valid manifest fixture: {path.relative_to(repo_root)}")

    for path in invalid:
        try:
            validate_manifest(path, repo_root)
        except ManifestError as error:
            print(
                f"rejected invalid manifest fixture: {path.relative_to(repo_root)}: {error}"
            )
            continue
        raise ManifestError(f"invalid fixture unexpectedly accepted: {path}")

    return 0


def validate_manifest(path: Path, repo_root: Path) -> None:
    manifest = read_json_object(path)
    require_exact_fields(manifest, TOP_LEVEL_FIELDS, "manifest")
    require_equal(manifest["schema"], SCHEMA, "schema")
    require_name(manifest["module"], "module")
    validate_imports(manifest["imports"])
    validate_certificates(manifest["certificates"], repo_root)
    validate_policy(manifest["policy"])


def read_json_object(path: Path) -> dict[str, Any]:
    def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ManifestError(f"duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        with path.open("r", encoding="utf-8") as handle:
            value = json.load(handle, object_pairs_hook=reject_duplicates)
    except json.JSONDecodeError as error:
        raise ManifestError(f"{path}: invalid JSON: {error}") from error

    if not isinstance(value, dict):
        raise ManifestError(f"{path}: manifest must be a JSON object")
    return value


def validate_imports(imports: Any) -> None:
    if not isinstance(imports, list):
        raise ManifestError("imports must be a list")

    previous_key: tuple[str, str, str] | None = None
    seen_identity: set[tuple[str, str]] = set()
    for index, entry in enumerate(imports):
        field = f"imports[{index}]"
        if not isinstance(entry, dict):
            raise ManifestError(f"{field} must be an object")
        if not {"module", "export_hash"}.issubset(entry):
            raise ManifestError(f"{field} must include module and export_hash")
        require_allowed_fields(entry, IMPORT_FIELDS, field)
        module = require_name(entry["module"], f"{field}.module")
        export_hash = require_hash(entry["export_hash"], f"{field}.export_hash")
        certificate_hash = ""
        if "certificate_hash" in entry:
            certificate_hash = require_hash(
                entry["certificate_hash"], f"{field}.certificate_hash"
            )

        identity = (module, export_hash)
        if identity in seen_identity:
            raise ManifestError(f"{field} duplicates import {module}:{export_hash}")
        seen_identity.add(identity)

        key = (module, export_hash, certificate_hash)
        if previous_key is not None and key <= previous_key:
            raise ManifestError(f"{field} is not in canonical import order")
        previous_key = key


def validate_certificates(certificates: Any, repo_root: Path) -> None:
    if not isinstance(certificates, list) or not certificates:
        raise ManifestError("certificates must be a nonempty list")

    seen_paths: set[str] = set()
    for index, entry in enumerate(certificates):
        field = f"certificates[{index}]"
        if not isinstance(entry, dict):
            raise ManifestError(f"{field} must be an object")
        require_exact_fields(entry, CERTIFICATE_FIELDS, field)

        module = require_name(entry["module"], f"{field}.module")
        manifest_path = require_manifest_path(entry["path"], f"{field}.path")
        if manifest_path in seen_paths:
            raise ManifestError(f"{field}.path duplicates {manifest_path}")
        seen_paths.add(manifest_path)

        expected = {
            "export": require_hash(
                entry["expected_export_hash"], f"{field}.expected_export_hash"
            ),
            "axiom_report": require_hash(
                entry["expected_axiom_report_hash"],
                f"{field}.expected_axiom_report_hash",
            ),
            "certificate": require_hash(
                entry["expected_certificate_hash"],
                f"{field}.expected_certificate_hash",
            ),
        }

        certificate_path = resolve_package_path(repo_root, manifest_path, field)
        report = run_mpk_check(repo_root, certificate_path, field)
        require_equal(report.get("verdict"), "accepted", f"{field}.verdict")
        require_equal(report.get("module"), module, f"{field}.module")

        hashes = report.get("hashes")
        if not isinstance(hashes, dict):
            raise ManifestError(f"{field}.hashes missing from checker output")
        for key, expected_hash in expected.items():
            require_equal(hashes.get(key), expected_hash, f"{field}.hashes.{key}")


def validate_policy(policy: Any) -> None:
    if not isinstance(policy, dict):
        raise ManifestError("policy must be an object")
    require_exact_fields(policy, POLICY_FIELDS, "policy")

    profile = policy["checker_profile"]
    if profile not in CHECKER_PROFILES:
        raise ManifestError(f"policy.checker_profile is unknown: {profile!r}")

    axiom_profiles = policy["allowed_axiom_profiles"]
    if not isinstance(axiom_profiles, list) or not axiom_profiles:
        raise ManifestError("policy.allowed_axiom_profiles must be a nonempty list")
    seen_axiom_profiles: set[str] = set()
    for index, value in enumerate(axiom_profiles):
        if not isinstance(value, str) or value not in AXIOM_PROFILES:
            raise ManifestError(
                f"policy.allowed_axiom_profiles[{index}] is not registered"
            )
        if value in seen_axiom_profiles:
            raise ManifestError(
                f"policy.allowed_axiom_profiles[{index}] duplicates {value!r}"
            )
        seen_axiom_profiles.add(value)

    if not isinstance(policy["require_reference_checker"], bool):
        raise ManifestError("policy.require_reference_checker must be boolean")
    if policy["require_source_free_check"] is not True:
        raise ManifestError("policy.require_source_free_check must be true")


def require_exact_fields(
    obj: dict[str, Any], expected: set[str], field: str
) -> None:
    require_allowed_fields(obj, expected, field)
    missing = sorted(expected.difference(obj))
    if missing:
        raise ManifestError(f"{field} missing required fields: {', '.join(missing)}")


def require_allowed_fields(obj: dict[str, Any], allowed: set[str], field: str) -> None:
    unknown = sorted(set(obj).difference(allowed))
    if unknown:
        raise ManifestError(f"{field} has unknown fields: {', '.join(unknown)}")


def require_equal(actual: Any, expected: Any, field: str) -> None:
    if actual != expected:
        raise ManifestError(f"{field} = {actual!r}, want {expected!r}")


def require_name(value: Any, field: str) -> str:
    if not isinstance(value, str) or not NAME_RE.fullmatch(value):
        raise ManifestError(f"{field} must be a canonical MPK name")
    return value


def require_hash(value: Any, field: str) -> str:
    if not isinstance(value, str) or not HASH_RE.fullmatch(value):
        raise ManifestError(f"{field} must be lowercase 64-character hex")
    if value == "0" * 64:
        raise ManifestError(f"{field} must not be all zeroes")
    return value


def require_manifest_path(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise ManifestError(f"{field} must be a nonempty string")
    if value.startswith("/") or "\\" in value:
        raise ManifestError(f"{field} must be a package-root relative POSIX path")
    parts = value.split("/")
    if any(part in {"", ".", ".."} for part in parts):
        raise ManifestError(f"{field} must not contain empty, ., or .. components")
    if not (value.endswith(".mpcert") or value.endswith(".hex")):
        raise ManifestError(f"{field} must point to .mpcert or .hex")
    return value


def resolve_package_path(repo_root: Path, manifest_path: str, field: str) -> Path:
    candidate = (repo_root / manifest_path).resolve()
    try:
        candidate.relative_to(repo_root.resolve())
    except ValueError as error:
        raise ManifestError(f"{field}.path escapes the package root") from error
    if not candidate.is_file():
        raise ManifestError(f"{field}.path does not exist: {manifest_path}")
    return candidate


def run_mpk_check(repo_root: Path, certificate_path: Path, field: str) -> dict[str, Any]:
    if mpk_bin := os.environ.get("MPK_BIN"):
        command = [mpk_bin, "check", str(certificate_path)]
    else:
        command = [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "mpk-cli",
            "--",
            "check",
            str(certificate_path),
        ]

    completed = subprocess.run(
        command,
        cwd=repo_root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        raise ManifestError(
            f"{field}.path checker failed for {certificate_path}: {completed.stderr}"
        )

    try:
        output = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise ManifestError(f"{field}.path checker output was not JSON") from error
    if not isinstance(output, dict):
        raise ManifestError(f"{field}.path checker output must be a JSON object")
    return output


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ManifestError as error:
        print(f"package manifest fixture check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
