#!/usr/bin/env python3
"""Fail if canonical frontend artifacts expose machine- or run-local state."""

from __future__ import annotations

import json
import os
import re
import socket
import stat
import sys
import tempfile
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parent.parent
RUST_CORPUS_ROOTS = (
    Path("rust-tools/rust2vir/testdata/positive"),
    Path("fixtures/rust-basic/positive"),
)
GO_CORPUS_ROOT = Path("fixtures/vir-go")
EXAMPLE_ARTIFACT_ROOTS = (Path("examples/rust-payment-policy/artifacts"),)
GO_TOP_LEVEL_ARTIFACTS = (
    Path("frontend-index.json"),
    Path("derived-index.json"),
    Path("manifest.json"),
    Path("negative-results.json"),
    Path("policy/evidence.json"),
    Path("policy/scan.json"),
    Path("ai/api-v1-response.json"),
    Path("ai/dry-run.json"),
)
MINIMUM_ARTIFACT_COUNT = 200

UNIX_PATH_RE = re.compile(r"(?:^|[\s=\"'(:])/(?!/)[^\s\"']+")
WINDOWS_PATH_RE = re.compile(r"(?:^|[\s=\"'(])(?:[A-Za-z]:[\\/]|\\\\)[^\s\"']+")
ISO_TIMESTAMP_RE = re.compile(
    r"(?:^|[^0-9])(?:19|20)[0-9]{2}-[01][0-9]-[0-3][0-9]"
    r"T[0-2][0-9]:[0-5][0-9]:[0-6][0-9](?:\.[0-9]+)?Z(?:$|[^0-9])"
)
SENTINEL_RE = re.compile(
    r"(?:MPK_(?:ENV_)?SENTINEL|HOSTILE_SENTINEL|must-not-reach)", re.IGNORECASE
)
FORBIDDEN_KEY_NAMES = {
    "generatedat",
    "generatedtimestamp",
    "hostname",
    "timestamp",
}
FIXED_PRIVATE_ROOTS = (
    "/mpk/cargo-home",
    "/mpk/driver-output",
    "/mpk/frontend",
    "/mpk/home",
    "/mpk/input",
    "/mpk/native-runtime",
    "/mpk/target",
    "/mpk/tmp",
    "/mpk/toolchain",
    "/mpk/work",
    "/not-emitted/",
)


class ArtifactPathError(Exception):
    pass


def main() -> int:
    self_test_matchers()
    artifacts = discover_artifacts()
    forbidden_paths = machine_paths()
    hostnames = machine_hostnames()
    findings: list[str] = []
    for relative in artifacts:
        try:
            value = read_canonical_json(relative)
            scan_value(value, "$", relative, forbidden_paths, hostnames, findings)
        except ArtifactPathError as error:
            findings.append(f"{relative}: {error}")

    if findings:
        for finding in findings:
            print(f"artifact path check failed: {finding}", file=sys.stderr)
        return 1
    print(f"artifact path check passed: {len(artifacts)} canonical JSON artifacts")
    return 0


def discover_artifacts() -> list[Path]:
    artifacts: set[Path] = set()
    for relative_root in RUST_CORPUS_ROOTS:
        root = require_directory(relative_root)
        artifacts.update(path.relative_to(ROOT) for path in root.glob("**/artifacts/*.json"))
        artifacts.add(relative_root / "frontend-index.json")

    go_root = require_directory(GO_CORPUS_ROOT)
    for subtree in ("frontend", "derived"):
        subtree_root = require_directory(GO_CORPUS_ROOT / subtree)
        artifacts.update(path.relative_to(ROOT) for path in subtree_root.rglob("*.json"))
    artifacts.update(GO_CORPUS_ROOT / path for path in GO_TOP_LEVEL_ARTIFACTS)
    for relative_root in EXAMPLE_ARTIFACT_ROOTS:
        root = require_directory(relative_root)
        artifacts.update(path.relative_to(ROOT) for path in root.rglob("*.json"))

    ordered = sorted(artifacts, key=lambda path: path.as_posix())
    if len(ordered) < MINIMUM_ARTIFACT_COUNT:
        raise ArtifactPathError(
            f"artifact inventory unexpectedly contains only {len(ordered)} files"
        )
    for relative in ordered:
        require_regular_file(relative)
    return ordered


def require_directory(relative: Path) -> Path:
    path = ROOT / relative
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ArtifactPathError(f"cannot inspect corpus directory {relative}: {error}") from error
    if not stat.S_ISDIR(metadata.st_mode) or path.is_symlink():
        raise ArtifactPathError(f"corpus directory is not a real directory: {relative}")
    return path


def require_regular_file(relative: Path) -> Path:
    path = ROOT / relative
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ArtifactPathError(f"cannot inspect artifact {relative}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode) or path.is_symlink():
        raise ArtifactPathError(f"artifact is not a regular file: {relative}")
    try:
        path.resolve(strict=True).relative_to(ROOT)
    except (OSError, ValueError) as error:
        raise ArtifactPathError(f"artifact escapes the repository: {relative}") from error
    return path


def read_canonical_json(relative: Path) -> Any:
    path = require_regular_file(relative)
    try:
        transport = path.read_bytes()
    except OSError as error:
        raise ArtifactPathError(f"cannot read artifact: {error}") from error
    body = transport[:-1] if transport.endswith(b"\n") else transport
    if not body or body.endswith((b"\n", b"\r")):
        raise ArtifactPathError("artifact has empty or noncanonical trailing whitespace")
    try:
        value = json.loads(body)
        canonical = json.dumps(
            value,
            ensure_ascii=False,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
    except (UnicodeDecodeError, json.JSONDecodeError, TypeError, ValueError) as error:
        raise ArtifactPathError(f"invalid UTF-8 JSON: {error}") from error
    if body != canonical:
        raise ArtifactPathError("artifact bytes are not canonical JSON")
    return value


def scan_value(
    value: Any,
    location: str,
    artifact: Path,
    forbidden_paths: tuple[str, ...],
    hostnames: tuple[str, ...],
    findings: list[str],
) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            normalized_key = re.sub(r"[_-]", "", key).lower()
            if normalized_key in FORBIDDEN_KEY_NAMES:
                findings.append(f"{artifact}:{location}.{key}: forbidden host/time field")
            scan_value(
                child,
                f"{location}.{key}",
                artifact,
                forbidden_paths,
                hostnames,
                findings,
            )
    elif isinstance(value, list):
        for index, child in enumerate(value):
            scan_value(
                child,
                f"{location}[{index}]",
                artifact,
                forbidden_paths,
                hostnames,
                findings,
            )
    elif isinstance(value, str):
        reason = leakage_reason(value, forbidden_paths, hostnames)
        if reason is not None:
            findings.append(f"{artifact}:{location}: {reason}")


def leakage_reason(
    value: str, forbidden_paths: Iterable[str], hostnames: Iterable[str]
) -> str | None:
    if value.startswith("file://"):
        return "file URI"
    if UNIX_PATH_RE.search(value) or WINDOWS_PATH_RE.search(value):
        return "absolute filesystem path"
    if any(root in value for root in FIXED_PRIVATE_ROOTS):
        return "private sandbox path"
    if SENTINEL_RE.search(value):
        return "sentinel string"
    if ISO_TIMESTAMP_RE.search(value):
        return "wall-clock timestamp"
    for path in forbidden_paths:
        if path in value:
            return f"machine-local path {path!r}"
    for hostname in hostnames:
        if hostname in value:
            return f"hostname {hostname!r}"
    return None


def machine_paths() -> tuple[str, ...]:
    candidates = {
        str(ROOT),
        str(Path.home()),
        tempfile.gettempdir(),
    }
    for name in (
        "CARGO_HOME",
        "CARGO_TARGET_DIR",
        "CODEX_HOME",
        "HOME",
        "RUSTC",
        "RUSTUP_HOME",
        "TEMP",
        "TMP",
        "TMPDIR",
    ):
        value = os.environ.get(name)
        if value and Path(value).is_absolute():
            candidates.add(value.rstrip("/\\"))
    return tuple(sorted((value for value in candidates if len(value) > 1), key=len, reverse=True))


def machine_hostnames() -> tuple[str, ...]:
    candidates = {socket.gethostname(), os.environ.get("HOSTNAME", "")}
    return tuple(sorted(value for value in candidates if len(value) >= 8))


def self_test_matchers() -> None:
    local_paths = ("/private/workspace", "/private/home")
    hosts = ("host-a1b2c3",)
    rejected = (
        "/absolute/source/lib.rs",
        "RUSTC=/private/toolchain/bin/rustc",
        r"C:\\Users\\builder\\source.rs",
        "file:///private/source.rs",
        "MPK_ENV_SENTINEL=must-not-reach-child",
        "2099-12-31T23:59:59Z",
        "host-a1b2c3",
    )
    for value in rejected:
        if leakage_reason(value, local_paths, hosts) is None:
            raise ArtifactPathError(f"internal matcher did not reject {value!r}")
    accepted = (
        "src/lib.rs",
        "github.com/finitefield-org/mpk/fixtures/example.Function",
        "https://example.invalid/schema/v0",
        "CARGO_TARGET_DIR=rust-tools/rust2vir/target",
    )
    for value in accepted:
        if leakage_reason(value, local_paths, hosts) is not None:
            raise ArtifactPathError(f"internal matcher rejected clean value {value!r}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ArtifactPathError as error:
        print(f"artifact path check failed: {error}", file=sys.stderr)
        raise SystemExit(1) from None
