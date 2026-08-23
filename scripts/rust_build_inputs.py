#!/usr/bin/env python3
"""Frozen Rust build-input materializer, validator, launcher, and candidate builder."""

from __future__ import annotations

import base64
from collections.abc import Iterator
import copy
import ctypes
import errno
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import posixpath
import re
import secrets
import selectors
import shutil
import stat
import struct
import subprocess
import sys
import tarfile
import tempfile
import urllib.error
import urllib.request


BUILD_INPUT_DOMAIN = b"MPK-RUST-BUILD-INPUTS-0.1\0"
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
EXPECTED_RUSTC_COMMIT = "4d08223c054cf5a56d9761ca925fd46ffebe7115"
DESCRIPTOR_LIMIT = 268_435_456
FILE_COUNT_LIMIT = 1_048_576
PACKAGE_COUNT_LIMIT = 8_192
PATH_LIMIT = 1_024
FILE_SIZE_LIMIT = 4_294_967_296
AGGREGATE_LIMIT = 34_359_738_368
U64_MAX = (1 << 64) - 1
RUST_FRONTEND_ID = "frontend.rust.rust2vir.candidate.v0"
RUST_TOOLCHAIN_ID = "toolchain.rust.nightly-2025-06-01.candidate.v0"
RUST_HOST_ID = "mpk.host.linux-x86_64-gnu.glibc2_27.v0"
RUST_RUNTIME_ID = "mpk.runtime.linux-x86_64-gnu.glibc2_27.v0"
BUILD_IMAGE = (
    "docker.io/library/buildpack-deps@"
    "sha256:816cb0d4a26fd8584b27d190bdd57ba7048be4fc20c259e60a985bec812887dc"
)
RUNTIME_IMAGE = (
    "docker.io/library/ubuntu@"
    "sha256:dca176c9663a7ba4c1f0e710986f5a25e672842963d95b960191e2d9f7185ebe"
)
RUST_DIST_ROOT = "https://static.rust-lang.org/dist/2025-06-01"
LLVM_URL = (
    "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/"
    "clang%2Bllvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz"
)
CARGO_FUZZ_URL = (
    "https://github.com/rust-fuzz/cargo-fuzz/archive/refs/tags/0.13.1.tar.gz"
)


class RustBuildFailure(Exception):
    def __init__(self, code: str = "BUNDLE_BUILD_INPUTS_INVALID", exit_code: int = 65) -> None:
        super().__init__(code)
        self.code = code
        self.exit_code = exit_code


BUILD_INPUT_LIMITS = {
    "descriptor_transport": (DESCRIPTOR_LIMIT, "RUST_BUILD_INPUTS_TRANSPORT"),
    "regular_files": (FILE_COUNT_LIMIT, "RUST_BUILD_INPUTS_FILE"),
    "package_records": (PACKAGE_COUNT_LIMIT, "RUST_BUILD_INPUTS_GRAPH"),
    "inventory_path": (PATH_LIMIT, "RUST_BUILD_INPUTS_PATH"),
    "regular_file": (FILE_SIZE_LIMIT, "RUST_BUILD_INPUTS_FILE"),
    "aggregate_cache": (AGGREGATE_LIMIT, "RUST_BUILD_INPUTS_GRAPH"),
}
FROZEN_PROCESS_LIMITS = {
    "processes": 256,
    "open_files_per_process": 1_024,
    "virtual_memory_bytes": 17_179_869_184,
    "resident_memory_bytes": 8_589_934_592,
    "temp_bytes": 4_294_967_296,
    "target_bytes": 17_179_869_184,
    "output_files": 262_144,
    "stdout_bytes": 67_108_864,
    "stderr_bytes": 2_097_152,
}
TARGET_DIRECTORY_LIMIT = FROZEN_PROCESS_LIMITS["output_files"]
DESCRIPTOR_JSON_DEPTH_LIMIT = 128


def checked_boundary(value: int, maximum: int, code: str) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or not isinstance(maximum, int)
        or isinstance(maximum, bool)
        or value < 0
        or maximum < 0
        or value > U64_MAX
        or maximum > U64_MAX
        or value > maximum
    ):
        raise RustBuildFailure(code)
    return value


def checked_boundary_add(total: int, increment: int, maximum: int, code: str) -> int:
    if (
        not isinstance(total, int)
        or isinstance(total, bool)
        or not isinstance(increment, int)
        or isinstance(increment, bool)
        or total < 0
        or increment < 0
        or total > U64_MAX
        or increment > U64_MAX
        or total > U64_MAX - increment
    ):
        raise RustBuildFailure(code)
    return checked_boundary(total + increment, maximum, code)


def validate_build_input_limit(identifier: str, value: int) -> int:
    profile = BUILD_INPUT_LIMITS.get(identifier)
    if profile is None:
        raise RustBuildFailure()
    maximum, code = profile
    return checked_boundary(value, maximum, code)


def add_build_input_counter(identifier: str, total: int, increment: int) -> int:
    profile = BUILD_INPUT_LIMITS.get(identifier)
    if profile is None:
        raise RustBuildFailure()
    maximum, code = profile
    return checked_boundary_add(total, increment, maximum, code)


def dispatch_build_input_boundary(construction: object) -> int:
    if not isinstance(construction, dict):
        raise RustBuildFailure()
    fixture = construction.get("fixture")
    counter = construction.get("counter")
    if not isinstance(counter, str):
        raise RustBuildFailure()
    if fixture == "checked_counter" and set(construction) == {
        "fixture",
        "counter",
        "count",
    }:
        return validate_build_input_limit(counter, construction["count"])
    if fixture == "portable_path_bytes" and set(construction) == {
        "fixture",
        "counter",
        "component_bytes",
    }:
        component_bytes = construction["component_bytes"]
        if counter != "inventory_path" or not isinstance(component_bytes, list):
            raise RustBuildFailure()
        if any(
            not isinstance(length, int)
            or isinstance(length, bool)
            or not 1 <= length <= 255
            for length in component_bytes
        ):
            raise RustBuildFailure()
        path = "/".join("a" * length for length in component_bytes)
        count = validate_build_input_limit(counter, len(path.encode("ascii")))
        if not portable_path(path, build_input=True):
            raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
        return count
    if fixture == "checked_counter_add" and set(construction) == {
        "fixture",
        "counter",
        "initial",
        "increment",
    }:
        initial = construction["initial"]
        if initial == "u64_max":
            initial = U64_MAX
        return add_build_input_counter(
            counter, initial, construction["increment"]
        )
    raise RustBuildFailure()


def repository_root() -> Path:
    return Path(__file__).resolve().parent.parent


def vector_path() -> Path:
    return repository_root() / "develop/specs/vectors/rust-build-inputs-v0.json"


def descriptor_path() -> Path:
    return repository_root() / "release/build-inputs/rust/build-inputs.json"


def cache_parent() -> Path:
    return repository_root() / "release/build-input-cache/rust"


def candidate_path() -> Path:
    return repository_root() / "release/bundles/candidates/rust/candidate.json"


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def typed_hash(domain: bytes, value: object) -> str:
    return hashlib.sha256(domain + canonical(value)).hexdigest()


def raw_hash(
    path: Path,
    *,
    maximum: int = FILE_SIZE_LIMIT,
    code: str = "BUNDLE_BUILD_INPUTS_INVALID",
) -> str:
    checked_boundary(maximum, U64_MAX, code)
    digest = hashlib.sha256()
    observed = 0
    descriptor = -1
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
        initial = os.fstat(descriptor)
        if not stat.S_ISREG(initial.st_mode):
            raise RustBuildFailure(code)
        checked_boundary(initial.st_size, maximum, code)
        while True:
            block = os.read(descriptor, 1024 * 1024)
            if not block:
                break
            observed = checked_boundary_add(observed, len(block), maximum, code)
            digest.update(block)
        retained = os.fstat(descriptor)
        path_metadata = path.lstat()
        if (
            not stat.S_ISREG(retained.st_mode)
            or not stat.S_ISREG(path_metadata.st_mode)
            or stat.S_ISLNK(path_metadata.st_mode)
            or (retained.st_dev, retained.st_ino, retained.st_size)
            != (initial.st_dev, initial.st_ino, initial.st_size)
            or (path_metadata.st_dev, path_metadata.st_ino, path_metadata.st_size)
            != (initial.st_dev, initial.st_ino, initial.st_size)
            or retained.st_mtime_ns != initial.st_mtime_ns
            or retained.st_ctime_ns != initial.st_ctime_ns
            or observed != initial.st_size
        ):
            raise RustBuildFailure(code)
    except OSError as error:
        raise RustBuildFailure(code) from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return digest.hexdigest()


def same_file_bytes(left: Path, right: Path) -> bool:
    left_descriptor = -1
    right_descriptor = -1
    try:
        left_descriptor = os.open(
            left, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
        )
        right_descriptor = os.open(
            right, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
        )
        left_initial = os.fstat(left_descriptor)
        right_initial = os.fstat(right_descriptor)
        if (
            not stat.S_ISREG(left_initial.st_mode)
            or not stat.S_ISREG(right_initial.st_mode)
            or left_initial.st_size != right_initial.st_size
            or left_initial.st_size > FILE_SIZE_LIMIT
        ):
            return False
        while True:
            left_block = os.read(left_descriptor, 1024 * 1024)
            right_block = os.read(right_descriptor, 1024 * 1024)
            if left_block != right_block:
                return False
            if not left_block:
                break
        left_retained = os.fstat(left_descriptor)
        right_retained = os.fstat(right_descriptor)
        left_path = left.lstat()
        right_path = right.lstat()
        return (
            (left_retained.st_dev, left_retained.st_ino, left_retained.st_size)
            == (left_initial.st_dev, left_initial.st_ino, left_initial.st_size)
            and (left_path.st_dev, left_path.st_ino, left_path.st_size)
            == (left_initial.st_dev, left_initial.st_ino, left_initial.st_size)
            and (right_retained.st_dev, right_retained.st_ino, right_retained.st_size)
            == (right_initial.st_dev, right_initial.st_ino, right_initial.st_size)
            and (right_path.st_dev, right_path.st_ino, right_path.st_size)
            == (right_initial.st_dev, right_initial.st_ino, right_initial.st_size)
        )
    except OSError:
        return False
    finally:
        if left_descriptor >= 0:
            os.close(left_descriptor)
        if right_descriptor >= 0:
            os.close(right_descriptor)


def strict_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise RustBuildFailure()
        result[key] = value
    return result


def reject_number(_value: str) -> object:
    raise RustBuildFailure()


def strict_integer(value: str) -> int:
    if len(value.lstrip("-")) > 16:
        raise RustBuildFailure()
    return int(value)


class DescriptorPrescanner:
    """Scan descriptor JSON without retaining its value tree.

    The transport cap bounds the input buffer itself.  This pass additionally
    enforces the declared inventory counters and a conservative nesting bound
    before ``json.loads`` can allocate a value graph or recurse over hostile
    input.
    """

    def __init__(self, data: bytes) -> None:
        self.data = data
        self.offset = 0
        self.file_count = 0
        self.package_count = 0
        self.aggregate_size = 0

    def scan(self) -> tuple[int, int, int]:
        self._skip_space()
        self._value((), 0)
        self._skip_space()
        if self.offset != len(self.data):
            self._transport_failure()
        return self.file_count, self.package_count, self.aggregate_size

    @staticmethod
    def _transport_failure() -> None:
        raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT")

    def _skip_space(self) -> None:
        while self.offset < len(self.data) and self.data[self.offset] in b" \t\r\n":
            self.offset += 1

    def _take(self, expected: int) -> None:
        if self.offset >= len(self.data) or self.data[self.offset] != expected:
            self._transport_failure()
        self.offset += 1

    def _value(self, path: tuple[str, ...], depth: int) -> None:
        if depth > DESCRIPTOR_JSON_DEPTH_LIMIT or self.offset >= len(self.data):
            self._transport_failure()
        marker = self.data[self.offset]
        if marker == ord("{"):
            self._object(path, depth + 1)
        elif marker == ord("["):
            self._array(path, depth + 1)
        elif marker == ord('"'):
            start, end = self._string_bounds()
            if path == ("components", "*", "files", "*", "path"):
                self._validate_path_string(start, end)
        elif marker == ord("-") or ord("0") <= marker <= ord("9"):
            start, end, integer = self._number_bounds()
            if path == ("components", "*", "files", "*", "size_bytes"):
                if not integer:
                    raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
                raw = self.data[start:end]
                if raw.startswith(b"-") or len(raw) > 20:
                    raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
                try:
                    size = int(raw)
                except ValueError as error:
                    raise RustBuildFailure("RUST_BUILD_INPUTS_FILE") from error
                validate_build_input_limit("regular_file", size)
                self.aggregate_size = add_build_input_counter(
                    "aggregate_cache", self.aggregate_size, size
                )
        elif self.data.startswith(b"true", self.offset):
            self.offset += 4
        elif self.data.startswith(b"false", self.offset):
            self.offset += 5
        elif self.data.startswith(b"null", self.offset):
            self.offset += 4
        else:
            self._transport_failure()

    def _object(self, path: tuple[str, ...], depth: int) -> None:
        self._take(ord("{"))
        self._skip_space()
        if self.offset < len(self.data) and self.data[self.offset] == ord("}"):
            self.offset += 1
            return
        while True:
            if self.offset >= len(self.data) or self.data[self.offset] != ord('"'):
                self._transport_failure()
            start, end = self._string_bounds()
            key = self._short_string(start, end)
            self._skip_space()
            self._take(ord(":"))
            self._skip_space()
            self._value(path + ((key if key is not None else "<unknown>"),), depth)
            self._skip_space()
            if self.offset >= len(self.data):
                self._transport_failure()
            marker = self.data[self.offset]
            self.offset += 1
            if marker == ord("}"):
                return
            if marker != ord(","):
                self._transport_failure()
            self._skip_space()

    def _array(self, path: tuple[str, ...], depth: int) -> None:
        self._take(ord("["))
        self._skip_space()
        if self.offset < len(self.data) and self.data[self.offset] == ord("]"):
            self.offset += 1
            return
        while True:
            if path == ("components", "*", "files"):
                self.file_count = add_build_input_counter(
                    "regular_files", self.file_count, 1
                )
            elif path == ("graphs", "*", "packages"):
                self.package_count = add_build_input_counter(
                    "package_records", self.package_count, 1
                )
            self._value(path + ("*",), depth)
            self._skip_space()
            if self.offset >= len(self.data):
                self._transport_failure()
            marker = self.data[self.offset]
            self.offset += 1
            if marker == ord("]"):
                return
            if marker != ord(","):
                self._transport_failure()
            self._skip_space()

    def _string_bounds(self) -> tuple[int, int]:
        start = self.offset
        self._take(ord('"'))
        while self.offset < len(self.data):
            marker = self.data[self.offset]
            if marker == ord('"'):
                self.offset += 1
                return start, self.offset
            if marker < 0x20:
                self._transport_failure()
            if marker == ord("\\"):
                self.offset += 1
                if self.offset >= len(self.data):
                    self._transport_failure()
                escape = self.data[self.offset]
                if escape == ord("u"):
                    if self.offset + 4 >= len(self.data) or any(
                        digit not in b"0123456789abcdefABCDEF"
                        for digit in self.data[self.offset + 1 : self.offset + 5]
                    ):
                        self._transport_failure()
                    self.offset += 5
                    continue
                if escape not in b'"\\/bfnrt':
                    self._transport_failure()
            self.offset += 1
        self._transport_failure()

    def _short_string(self, start: int, end: int) -> str | None:
        raw = self.data[start:end]
        if len(raw) > 256:
            return None
        try:
            value = json.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT") from error
        return value if isinstance(value, str) else None

    def _validate_path_string(self, start: int, end: int) -> None:
        raw = self.data[start:end]
        # A JSON escape needs at most six source bytes per decoded scalar.  If
        # this bound is exceeded the decoded UTF-8 path cannot fit, so reject
        # without allocating a descriptor-sized Python string.
        if len(raw) - 2 > PATH_LIMIT * 6:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
        try:
            value = json.loads(raw.decode("utf-8"))
            encoded = value.encode("utf-8")
        except (UnicodeDecodeError, UnicodeEncodeError, json.JSONDecodeError) as error:
            raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT") from error
        validate_build_input_limit("inventory_path", len(encoded))

    def _number_bounds(self) -> tuple[int, int, bool]:
        start = self.offset
        if self.data[self.offset] == ord("-"):
            self.offset += 1
        if self.offset >= len(self.data):
            self._transport_failure()
        if self.data[self.offset] == ord("0"):
            self.offset += 1
            if self.offset < len(self.data) and ord("0") <= self.data[self.offset] <= ord("9"):
                self._transport_failure()
        elif ord("1") <= self.data[self.offset] <= ord("9"):
            while self.offset < len(self.data) and ord("0") <= self.data[self.offset] <= ord("9"):
                self.offset += 1
        else:
            self._transport_failure()
        integer = True
        if self.offset < len(self.data) and self.data[self.offset] == ord("."):
            integer = False
            self.offset += 1
            fraction_start = self.offset
            while self.offset < len(self.data) and ord("0") <= self.data[self.offset] <= ord("9"):
                self.offset += 1
            if self.offset == fraction_start:
                self._transport_failure()
        if self.offset < len(self.data) and self.data[self.offset] in b"eE":
            integer = False
            self.offset += 1
            if self.offset < len(self.data) and self.data[self.offset] in b"+-":
                self.offset += 1
            exponent_start = self.offset
            while self.offset < len(self.data) and ord("0") <= self.data[self.offset] <= ord("9"):
                self.offset += 1
            if self.offset == exponent_start:
                self._transport_failure()
        return start, self.offset, integer


def prescan_descriptor(data: bytes) -> tuple[int, int, int]:
    return DescriptorPrescanner(data).scan()


def strict_json(bytes_value: bytes) -> object:
    try:
        text = bytes_value.decode("utf-8")
        return json.loads(
            text,
            object_pairs_hook=strict_object,
            parse_float=reject_number,
            parse_constant=reject_number,
            parse_int=strict_integer,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise RustBuildFailure() from error


def load_vector() -> dict[str, object]:
    value = strict_json(
        read_bounded_regular_file(
            vector_path(),
            maximum=DESCRIPTOR_LIMIT,
            code="BUNDLE_BUILD_INPUTS_INVALID",
        )
    )
    if not isinstance(value, dict) or value.get("schema") != "mpk.rust.build_inputs.conformance.v0":
        raise RustBuildFailure()
    return value


def raw_templates(vector: dict[str, object]) -> dict[str, bytes]:
    result: dict[str, bytes] = {}
    for record in vector["raw_templates"]:
        identifier = record["id"]
        try:
            decoded = base64.b64decode(record["base64"], validate=True)
        except (ValueError, TypeError) as error:
            raise RustBuildFailure() from error
        if (
            len(decoded) != record["size_bytes"]
            or hashlib.sha256(decoded).hexdigest() != record["sha256"]
            or decoded.endswith(b"\n") is not record["final_lf"]
            or identifier in result
        ):
            raise RustBuildFailure()
        result[identifier] = decoded
    return result


def validate_project_templates(vector: dict[str, object]) -> None:
    templates = raw_templates(vector)
    projection = vector["production_projection"]
    if (
        projection.get("inventory_path_equality") != "byte-exact"
        or projection["native_runtime_projection"].get("toolchain_runpath")
        != "$ORIGIN/../lib"
    ):
        raise RustBuildFailure()
    project = repository_root() / "rust-tools/rust2vir"
    for relative, identifier in (
        ("Cargo.toml", "frontend_manifest"),
        ("Cargo.lock", "frontend_lock"),
        ("rust-toolchain.toml", "rust_toolchain"),
    ):
        path = project / relative
        expected = templates[identifier]
        if read_bounded_regular_file(
            path,
            maximum=len(expected),
            code="BUNDLE_BUILD_INPUTS_INVALID",
        ) != expected:
            raise RustBuildFailure()


def descriptor_transport(value: dict[str, object]) -> bytes:
    return canonical(value) + b"\n"


def build_inputs_hash(value: dict[str, object]) -> str:
    payload = dict(value)
    payload.pop("build_inputs_sha256", None)
    return typed_hash(BUILD_INPUT_DOMAIN, payload)


def portable_path(value: str, *, build_input: bool = False) -> bool:
    if not isinstance(value, str):
        return False
    try:
        encoded = value.encode("ascii")
    except UnicodeEncodeError:
        return False
    try:
        validate_build_input_limit("inventory_path", len(encoded))
    except RustBuildFailure:
        return False
    if (
        not encoded
        or len(encoded) != len(value)
        or value.startswith("/")
        or value.endswith("/")
        or "\\" in value
    ):
        return False
    for component in value.split("/"):
        if (
            not component
            or len(component.encode("ascii")) > 255
            or component in (".", "..")
            or component.endswith(".")
            or (
                re.fullmatch(
                    r"[A-Za-z0-9._+-]+" if build_input else r"[A-Za-z0-9._-]+",
                    component,
                )
                is None
            )
        ):
            return False
        stem = component.split(".", 1)[0].upper()
        if stem in {"CON", "PRN", "AUX", "NUL"} or re.fullmatch(r"(?:COM|LPT)[1-9]", stem):
            return False
    return True


def component_prefix(name: str) -> str:
    return name + "/"


def validate_descriptor_model(value: object, vector: dict[str, object]) -> dict[str, object]:
    if not isinstance(value, dict):
        raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
    expected_keys = set(vector["valid_descriptor"].keys())
    if set(value.keys()) != expected_keys or value.get("schema") != "mpk.rust.build_inputs.v0":
        raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
    claimed = value.get("build_inputs_sha256")
    if not isinstance(claimed, str):
        raise RustBuildFailure("RUST_BUILD_INPUTS_HASH")
    frozen = vector["valid_descriptor"]
    for key in (
        "schema",
        "profile_id",
        "recipe_id",
        "execution_host_profile_id",
        "runtime_layout_profile_id",
        "rust_distribution",
        "native",
    ):
        if value.get(key) != frozen.get(key):
            raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
    if value.get("registry") != frozen.get("registry") or value.get("graphs") != frozen.get("graphs"):
        raise RustBuildFailure("RUST_BUILD_INPUTS_GRAPH")
    if value.get("licenses") != frozen.get("licenses"):
        raise RustBuildFailure("RUST_BUILD_INPUTS_LICENSE")
    cargo_fuzz = value.get("cargo_fuzz")
    frozen_cargo_fuzz = frozen["cargo_fuzz"]
    if not isinstance(cargo_fuzz, dict) or set(cargo_fuzz) != set(frozen_cargo_fuzz):
        raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
    for key in set(frozen_cargo_fuzz) - {"executable_sha256"}:
        if cargo_fuzz[key] != frozen_cargo_fuzz[key]:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
    if not re.fullmatch(r"[0-9a-f]{64}", cargo_fuzz.get("executable_sha256", "")):
        raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
    components = value.get("components")
    frozen_components = frozen["components"]
    if not isinstance(components, list) or [item.get("name") for item in components] != [
        item["name"] for item in frozen_components
    ]:
        raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
    total_files = 0
    total_bytes = 0
    all_paths: set[str] = set()
    for component, frozen_component in zip(components, frozen_components):
        if not isinstance(component, dict):
            raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
        if "provenance" not in component:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
        if "license_refs" not in component or "notice_refs" not in component:
            raise RustBuildFailure("RUST_BUILD_INPUTS_LICENSE")
        if set(component) != set(frozen_component):
            raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
        if component["name"] != frozen_component["name"]:
            raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
        if component["provenance"] != frozen_component["provenance"]:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
        if component["license_refs"] != frozen_component["license_refs"] or component[
            "notice_refs"
        ] != frozen_component["notice_refs"]:
            raise RustBuildFailure("RUST_BUILD_INPUTS_LICENSE")
        files = component["files"]
        if not isinstance(files, list) or not files:
            raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
        paths = [item.get("path") for item in files if isinstance(item, dict)]
        if (
            len(paths) != len(files)
            or any(not isinstance(path, str) for path in paths)
            or paths != sorted(paths, key=lambda item: item.encode("utf-8"))
        ):
            raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
        for item in files:
            if set(item) != {"path", "executable", "size_bytes", "sha256"}:
                raise RustBuildFailure("RUST_BUILD_INPUTS_SHAPE")
            path = item["path"]
            size = item["size_bytes"]
            if not portable_path(path, build_input=True):
                raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
            if path.startswith("rust-tools/rust2vir/") or "/rust-tools/rust2vir/" in path:
                raise RustBuildFailure("RUST_BUILD_INPUTS_SOURCE_EXCLUSION")
            if path.startswith("release/build-inputs/rust/") or "/release/build-inputs/rust/" in path:
                raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
            if not path.startswith(component_prefix(component["name"])):
                raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
            if (
                not isinstance(item["executable"], bool)
                or not isinstance(size, int)
                or isinstance(size, bool)
                or size < 0
                or re.fullmatch(r"[0-9a-f]{64}", item["sha256"]) is None
            ):
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
            validate_build_input_limit("regular_file", size)
            if path in all_paths:
                raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
            all_paths.add(path)
            total_files = add_build_input_counter("regular_files", total_files, 1)
            total_bytes = add_build_input_counter("aggregate_cache", total_bytes, size)
    expected_top = set(vector["production_projection"]["cache_top_level"])
    if {path.split("/", 1)[0] for path in all_paths} != expected_top:
        raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
    graph_packages = 0
    for graph in value["graphs"]:
        for _package in graph["packages"]:
            graph_packages = add_build_input_counter(
                "package_records", graph_packages, 1
            )
    if claimed != build_inputs_hash(value):
        raise RustBuildFailure("RUST_BUILD_INPUTS_HASH")
    return value


def validate_descriptor_transport(
    transport: bytes, vector: dict[str, object]
) -> dict[str, object]:
    validate_build_input_limit("descriptor_transport", len(transport))
    if (
        not transport.endswith(b"\n")
        or transport.endswith(b"\n\n")
    ):
        raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT")
    prescan_descriptor(transport[:-1])
    try:
        value = strict_json(transport[:-1])
    except RustBuildFailure as error:
        raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT") from error
    try:
        canonical_transport = canonical(value) + b"\n"
    except (UnicodeEncodeError, ValueError, RecursionError) as error:
        raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT") from error
    if canonical_transport != transport:
        raise RustBuildFailure("RUST_BUILD_INPUTS_TRANSPORT")
    return validate_descriptor_model(value, vector)


def read_bounded_regular_file(path: Path, *, maximum: int, code: str) -> bytes:
    checked_boundary(maximum, U64_MAX, code)
    flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK
    descriptor = -1
    try:
        descriptor = os.open(path, flags)
        initial = os.fstat(descriptor)
        if not stat.S_ISREG(initial.st_mode):
            raise RustBuildFailure(code)
        checked_boundary(initial.st_size, maximum, code)
        transport = bytearray()
        while len(transport) <= maximum:
            block = os.read(descriptor, min(65_536, maximum + 1 - len(transport)))
            if not block:
                break
            transport.extend(block)
        checked_boundary(len(transport), maximum, code)
        retained = os.fstat(descriptor)
        path_metadata = path.lstat()
        initial_identity = (initial.st_dev, initial.st_ino)
        if (
            not stat.S_ISREG(retained.st_mode)
            or not stat.S_ISREG(path_metadata.st_mode)
            or stat.S_ISLNK(path_metadata.st_mode)
            or (retained.st_dev, retained.st_ino) != initial_identity
            or (path_metadata.st_dev, path_metadata.st_ino) != initial_identity
            or retained.st_size != initial.st_size
            or path_metadata.st_size != initial.st_size
            or len(transport) != initial.st_size
            or retained.st_mtime_ns != initial.st_mtime_ns
            or retained.st_ctime_ns != initial.st_ctime_ns
        ):
            raise RustBuildFailure(code)
    except OSError as error:
        raise RustBuildFailure(code) from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return bytes(transport)


def bounded_directory_entries(
    directory: Path, *, maximum: int, code: str
) -> list[tuple[str, os.stat_result]]:
    checked_boundary(maximum, U64_MAX, code)
    descriptor = -1
    try:
        expected = directory.lstat()
        if not stat.S_ISDIR(expected.st_mode) or stat.S_ISLNK(expected.st_mode):
            raise RustBuildFailure(code)
        descriptor = os.open(
            directory,
            os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW,
        )
        opened = os.fstat(descriptor)
        if (opened.st_dev, opened.st_ino) != (expected.st_dev, expected.st_ino):
            raise RustBuildFailure(code)
        result: list[tuple[str, os.stat_result]] = []
        with os.scandir(descriptor) as scanned:
            for entry in scanned:
                checked_boundary_add(len(result), 1, maximum, code)
                result.append((entry.name, entry.stat(follow_symlinks=False)))
        retained = os.fstat(descriptor)
        current = directory.lstat()
        if (
            (retained.st_dev, retained.st_ino) != (expected.st_dev, expected.st_ino)
            or (current.st_dev, current.st_ino) != (expected.st_dev, expected.st_ino)
            or not stat.S_ISDIR(current.st_mode)
            or stat.S_ISLNK(current.st_mode)
            or retained.st_mtime_ns != expected.st_mtime_ns
            or retained.st_ctime_ns != expected.st_ctime_ns
            or current.st_mtime_ns != expected.st_mtime_ns
            or current.st_ctime_ns != expected.st_ctime_ns
        ):
            raise RustBuildFailure(code)
    except OSError as error:
        raise RustBuildFailure(code) from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    result.sort(key=lambda item: os.fsencode(item[0]))
    return result


def walk_regular_files_bounded(
    root: Path,
    *,
    file_limit: int,
    byte_limit: int,
    directory_limit: int,
    code: str,
) -> Iterator[tuple[str, Path, os.stat_result]]:
    """Yield a deterministic, no-follow tree walk under explicit counters."""

    checked_boundary(file_limit, U64_MAX, code)
    checked_boundary(byte_limit, U64_MAX, code)
    checked_boundary(directory_limit, U64_MAX, code)
    files = 0
    total = 0
    directories = checked_boundary(1, directory_limit, code)
    node_limit = checked_boundary_add(directory_limit, file_limit, U64_MAX, code)
    nodes = checked_boundary(1, node_limit, code)
    pending: list[tuple[Path, str]] = [(root, "")]
    while pending:
        directory, prefix = pending.pop()
        entries = bounded_directory_entries(
            directory, maximum=node_limit - nodes, code=code
        )
        child_directories: list[tuple[Path, str]] = []
        for name, metadata in entries:
            nodes = checked_boundary_add(nodes, 1, node_limit, code)
            relative = f"{prefix}/{name}" if prefix else name
            path = directory / name
            if stat.S_ISDIR(metadata.st_mode):
                directories = checked_boundary_add(
                    directories, 1, directory_limit, code
                )
                child_directories.append((path, relative))
            elif stat.S_ISREG(metadata.st_mode):
                files = checked_boundary_add(files, 1, file_limit, code)
                total = checked_boundary_add(total, metadata.st_size, byte_limit, code)
                yield relative, path, metadata
            else:
                raise RustBuildFailure(code)
        # Stack order is reversed so yielded paths remain bytewise sorted.
        pending.extend(reversed(child_directories))


def read_descriptor(vector: dict[str, object]) -> tuple[dict[str, object], bytes]:
    transport = read_bounded_regular_file(
        descriptor_path(),
        maximum=BUILD_INPUT_LIMITS["descriptor_transport"][0],
        code="RUST_BUILD_INPUTS_TRANSPORT",
    )
    return validate_descriptor_transport(transport, vector), transport


def expected_cache_tree(descriptor: dict[str, object]) -> dict[str, object]:
    tree: dict[str, object] = {}
    for component in descriptor["components"]:
        for item in component["files"]:
            parts = item["path"].split("/")
            node = tree
            for part in parts[:-1]:
                child = node.get(part)
                if child is None:
                    child = {}
                    node[part] = child
                if not isinstance(child, dict):
                    raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
                node = child
            leaf = parts[-1]
            if leaf in node:
                raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
            node[leaf] = (item,)
    return tree


def missing_cache_code(relative: str) -> str:
    if relative == "notices" or relative.startswith("notices/"):
        return "RUST_BUILD_INPUTS_LICENSE"
    clang_config = "toolchain/bin/clang.cfg"
    if relative == clang_config or clang_config.startswith(relative + "/"):
        return "RUST_BUILD_INPUTS_PROVENANCE"
    return "RUST_BUILD_INPUTS_INVENTORY"


def validate_cache(descriptor: dict[str, object], *, root: Path | None = None) -> Path:
    digest = descriptor["build_inputs_sha256"]
    selected = root if root is not None else cache_parent() / digest
    if (
        (root is None and selected.name != digest)
        or selected.is_symlink()
        or not selected.is_dir()
    ):
        code = "RUST_BUILD_INPUTS_CACHE_KEY" if root is None else "RUST_BUILD_INPUTS_INVENTORY"
        raise RustBuildFailure(code)
    expected_tree = expected_cache_tree(descriptor)
    identities: set[tuple[int, int]] = set()
    file_count = 0
    total = 0
    failure_code = "RUST_BUILD_INPUTS_FILE"

    def walk(directory: Path, node: dict[str, object], prefix: str) -> None:
        nonlocal file_count, total, failure_code
        failure_code = "RUST_BUILD_INPUTS_INVENTORY"
        entries = bounded_directory_entries(
            directory,
            maximum=len(node),
            code="RUST_BUILD_INPUTS_INVENTORY",
        )
        expected_names = sorted(node, key=os.fsencode)
        actual_names: list[str] = []
        for name, _metadata in entries:
            relative = f"{prefix}/{name}" if prefix else name
            if not portable_path(relative, build_input=True):
                raise RustBuildFailure("RUST_BUILD_INPUTS_PATH")
            actual_names.append(name)
        if any(name not in node for name in actual_names):
            raise RustBuildFailure("RUST_BUILD_INPUTS_INVENTORY")
        actual_set = frozenset(actual_names)
        missing = [name for name in expected_names if name not in actual_set]
        if missing:
            first = missing[0]
            relative = f"{prefix}/{first}" if prefix else first
            raise RustBuildFailure(missing_cache_code(relative))

        for name, metadata in entries:
            relative = f"{prefix}/{name}" if prefix else name
            path = directory / name
            failure_code = "RUST_BUILD_INPUTS_FILE"
            child = node[name]
            if stat.S_ISLNK(metadata.st_mode):
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
            if isinstance(child, dict):
                if not stat.S_ISDIR(metadata.st_mode):
                    raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
                walk(path, child, relative)
                continue
            if not stat.S_ISREG(metadata.st_mode):
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")

            file_count = add_build_input_counter("regular_files", file_count, 1)
            validate_build_input_limit("regular_file", metadata.st_size)
            total = add_build_input_counter(
                "aggregate_cache", total, metadata.st_size
            )
            if metadata.st_nlink != 1:
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
            identity = (metadata.st_dev, metadata.st_ino)
            if identity in identities:
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
            identities.add(identity)
            item = child[0]
            mode = stat.S_IMODE(metadata.st_mode)
            if mode != (0o555 if item["executable"] else 0o444):
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")
            if (
                metadata.st_size != item["size_bytes"]
                or raw_hash(path, code="RUST_BUILD_INPUTS_FILE")
                != item["sha256"]
            ):
                if relative.endswith("/.cargo-checksum.json"):
                    raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
                if relative == "cargo-home-seed/config.toml":
                    raise RustBuildFailure("RUST_BUILD_INPUTS_CARGO_HOME")
                if relative == "toolchain/bin/clang.cfg":
                    raise RustBuildFailure("RUST_BUILD_INPUTS_PROVENANCE")
                if relative.startswith("notices/"):
                    raise RustBuildFailure("RUST_BUILD_INPUTS_LICENSE")
                raise RustBuildFailure("RUST_BUILD_INPUTS_FILE")

    try:
        walk(selected, expected_tree, "")
    except OSError as error:
        raise RustBuildFailure(failure_code) from error
    validate_cargo_home(selected)
    validate_vendor(descriptor, selected)
    return selected


def validate_cargo_home(root: Path) -> None:
    vector = load_vector()
    expected = raw_templates(vector)["cargo_home_config"]
    seed = root / "cargo-home-seed"
    entries = bounded_directory_entries(
        seed, maximum=1, code="RUST_BUILD_INPUTS_CARGO_HOME"
    )
    if [name for name, _metadata in entries] != ["config.toml"]:
        raise RustBuildFailure("RUST_BUILD_INPUTS_CARGO_HOME")
    if read_bounded_regular_file(
        seed / "config.toml",
        maximum=len(expected),
        code="RUST_BUILD_INPUTS_CARGO_HOME",
    ) != expected:
        raise RustBuildFailure("RUST_BUILD_INPUTS_CARGO_HOME")


def validate_vendor(descriptor: dict[str, object], root: Path) -> None:
    packages: dict[tuple[str, str], str] = {}
    package_count = 0
    for graph in descriptor["graphs"]:
        for package in graph["packages"]:
            package_count = add_build_input_counter(
                "package_records", package_count, 1
            )
            checksum = package["checksum"]
            if checksum is None:
                continue
            key = (package["name"], package["version"])
            if key in packages and packages[key] != checksum:
                raise RustBuildFailure("RUST_BUILD_INPUTS_GRAPH")
            packages[key] = checksum
    vendor = root / "vendor"
    expected = sorted(
        (f"{name}-{version}" for name, version in packages), key=os.fsencode
    )
    entries = bounded_directory_entries(
        vendor, maximum=len(expected), code="RUST_BUILD_INPUTS_VENDOR"
    )
    if (
        [name for name, _metadata in entries] != expected
        or any(not stat.S_ISDIR(metadata.st_mode) for _name, metadata in entries)
    ):
        raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
    declared_checksum_sizes = {
        item["path"]: item["size_bytes"]
        for component in descriptor["components"]
        for item in component["files"]
        if item["path"].endswith("/.cargo-checksum.json")
    }
    for (name, version), checksum in packages.items():
        package_root = vendor / f"{name}-{version}"
        checksum_path = package_root / ".cargo-checksum.json"
        declared_path = f"vendor/{name}-{version}/.cargo-checksum.json"
        maximum = declared_checksum_sizes.get(declared_path)
        if not isinstance(maximum, int) or isinstance(maximum, bool):
            raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
        checksum_bytes = read_bounded_regular_file(
            checksum_path,
            maximum=maximum,
            code="RUST_BUILD_INPUTS_VENDOR",
        )
        try:
            data = strict_json(checksum_bytes)
        except RustBuildFailure as error:
            raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR") from error
        if (
            not isinstance(data, dict)
            or set(data) != {"files", "package"}
            or not isinstance(data["files"], dict)
            or data["package"] != checksum
        ):
            raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
        if canonical(data) != checksum_bytes:
            raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
        actual: dict[str, str] = {}
        for relative, path, metadata in walk_regular_files_bounded(
            package_root,
            file_limit=FILE_COUNT_LIMIT,
            byte_limit=AGGREGATE_LIMIT,
            directory_limit=FILE_COUNT_LIMIT,
            code="RUST_BUILD_INPUTS_VENDOR",
        ):
            if relative == ".cargo-checksum.json":
                continue
            if metadata.st_nlink != 1:
                raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")
            actual[relative] = raw_hash(path, code="RUST_BUILD_INPUTS_VENDOR")
        if data["files"] != actual:
            raise RustBuildFailure("RUST_BUILD_INPUTS_VENDOR")


def check_build_inputs() -> tuple[dict[str, object], Path]:
    vector = load_vector()
    validate_project_templates(vector)
    descriptor, _ = read_descriptor(vector)
    root = validate_cache(descriptor)
    git = "/usr/bin/git"
    if Path(git).is_file():
        result = subprocess.run(
            [git, "ls-files", "release/build-input-cache"],
            cwd=repository_root(),
            stdin=subprocess.DEVNULL,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0 or result.stdout:
            raise RustBuildFailure()
    return descriptor, root


class ArchiveView:
    def __init__(self, storage: Path) -> None:
        self.storage = storage
        self.files: dict[str, tuple[int, Path]] = {}
        self.links: dict[str, tuple[str, bool]] = {}

    @staticmethod
    def normalized(name: str) -> str:
        while name.startswith("./"):
            name = name[2:]
        pure = PurePosixPath(name)
        if not name or pure.is_absolute() or any(part in ("", ".", "..") for part in pure.parts):
            raise RustBuildFailure("BUNDLE_BUILD_INPUTS_INVALID")
        return pure.as_posix()

    @classmethod
    def from_tar(cls, archive: Path, storage: Path) -> "ArchiveView":
        view = cls(storage)
        storage.mkdir(parents=True)
        try:
            with tarfile.open(archive, "r:*") as source:
                view._consume(source)
        except (OSError, tarfile.TarError) as error:
            raise RustBuildFailure() from error
        return view

    @classmethod
    def from_stream(cls, stream: object, storage: Path) -> "ArchiveView":
        view = cls(storage)
        storage.mkdir(parents=True)
        try:
            with tarfile.open(fileobj=stream, mode="r|*") as source:
                view._consume(source)
        except (OSError, tarfile.TarError) as error:
            raise RustBuildFailure() from error
        return view

    def _consume(self, source: tarfile.TarFile) -> None:
        for member in source:
            name = self.normalized(member.name)
            if name in self.files or name in self.links:
                raise RustBuildFailure()
            if member.isdir():
                continue
            if member.isreg():
                extracted = source.extractfile(member)
                if extracted is None:
                    raise RustBuildFailure()
                destination = self.storage / f"{len(self.files):08x}"
                with destination.open("xb") as output:
                    shutil.copyfileobj(extracted, output, 1024 * 1024)
                self.files[name] = (member.mode, destination)
            elif member.issym():
                self.links[name] = (member.linkname, False)
            elif member.islnk():
                self.links[name] = (member.linkname, True)
            else:
                raise RustBuildFailure()

    def resolve(self, name: str) -> tuple[int, Path]:
        current = self.normalized(name)
        for _ in range(64):
            if current in self.files:
                return self.files[current]
            rewritten = self.rewrite_link(current)
            if rewritten is None:
                raise RustBuildFailure() from FileNotFoundError(
                    f"archive link target missing: {name} -> {current}"
                )
            current = rewritten
        raise RustBuildFailure()

    def rewrite_link(self, name: str) -> str | None:
        parts = PurePosixPath(name).parts
        for index in range(1, len(parts) + 1):
            prefix = PurePosixPath(*parts[:index]).as_posix()
            link = self.links.get(prefix)
            if link is None:
                continue
            target, hard = link
            suffix = parts[index:]
            if hard and suffix:
                raise RustBuildFailure()
            absolute = target.startswith("/")
            base = "" if hard or absolute else posixpath.dirname(prefix)
            rewritten = posixpath.normpath(posixpath.join(base, target.lstrip("/")))
            if suffix:
                rewritten = posixpath.join(rewritten, *suffix)
            return self.normalized(rewritten)
        return None

    def resolve_directory(self, name: str) -> str:
        current = self.normalized(name)
        names = set(self.files) | set(self.links)
        for _ in range(64):
            if any(candidate.startswith(current + "/") for candidate in names):
                return current
            rewritten = self.rewrite_link(current)
            if rewritten is None:
                raise RustBuildFailure() from FileNotFoundError(
                    f"archive directory link target missing: {name} -> {current}"
                )
            current = rewritten
        raise RustBuildFailure()

    def read_bytes(self, name: str) -> bytes:
        _mode, path = self.resolve(name)
        return path.read_bytes()

    def copy_file(self, source_name: str, destination: Path, *, executable: bool | None = None) -> None:
        mode, source = self.resolve(source_name)
        destination.parent.mkdir(parents=True, exist_ok=True)
        copy_no_replace(source, destination)
        is_executable = bool(mode & 0o111) if executable is None else executable
        destination.chmod(0o755 if is_executable else 0o644)

    def copy_entry(self, source_name: str, destination: Path, *, hops: int = 0) -> int:
        if hops >= 64:
            raise RustBuildFailure()
        try:
            self.copy_file(source_name, destination)
            return 1
        except RustBuildFailure as file_error:
            try:
                directory = self.resolve_directory(source_name)
            except RustBuildFailure:
                raise file_error
        names = sorted(
            (set(self.files) | set(self.links)), key=lambda item: item.encode("utf-8")
        )
        prefix = directory + "/"
        copied = 0
        for child in names:
            if not child.startswith(prefix):
                continue
            relative = child[len(prefix) :]
            copied += self.copy_entry(child, destination / relative, hops=hops + 1)
        if copied == 0:
            raise RustBuildFailure()
        return copied

    def copy_prefix(
        self,
        prefix: str,
        destination: Path,
        *,
        exclude: set[str] | None = None,
        exclude_prefixes: set[str] | None = None,
    ) -> None:
        normalized_prefix = self.normalized(prefix).rstrip("/") + "/"
        names = sorted(
            (set(self.files) | set(self.links)), key=lambda item: item.encode("utf-8")
        )
        copied = 0
        for name in names:
            if not name.startswith(normalized_prefix):
                continue
            relative = name[len(normalized_prefix) :]
            if (
                not relative
                or (exclude and relative in exclude)
                or (
                    exclude_prefixes
                    and any(
                        relative == excluded or relative.startswith(excluded + "/")
                        for excluded in exclude_prefixes
                    )
                )
            ):
                continue
            copied += self.copy_entry(name, destination / relative)
        if copied == 0:
            raise RustBuildFailure()


def copy_no_replace(source: Path, destination: Path) -> None:
    if destination.exists() or destination.is_symlink():
        if (
            destination.is_file()
            and not destination.is_symlink()
            and same_file_bytes(source, destination)
        ):
            return
        raise RustBuildFailure()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with source.open("rb") as input_stream, destination.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)


def download(url: str, expected_sha256: str, destination: Path, maximum: int = 2_147_483_648) -> None:
    if destination.is_file() and raw_hash(destination) == expected_sha256:
        return
    request = urllib.request.Request(url, headers={"User-Agent": "mpk-rust-build-inputs-v0"})
    digest = hashlib.sha256()
    observed = 0
    temporary = destination.with_name(destination.name + ".partial")
    try:
        with urllib.request.urlopen(request, timeout=60) as response, temporary.open("xb") as output:
            while True:
                block = response.read(1024 * 1024)
                if not block:
                    break
                observed += len(block)
                if observed > maximum:
                    raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
                digest.update(block)
                output.write(block)
        if digest.hexdigest() != expected_sha256:
            raise RustBuildFailure("BUNDLE_BUILD_INPUTS_INVALID")
        os.replace(temporary, destination)
    except (OSError, urllib.error.URLError) as error:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
    finally:
        if temporary.exists():
            temporary.unlink()


def docker_path() -> str:
    for candidate in ("/usr/local/bin/docker", "/usr/bin/docker"):
        if Path(candidate).is_file() and os.access(candidate, os.X_OK):
            return candidate
    raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)


def fresh_container_name(label: str) -> str:
    if re.fullmatch(r"[a-z0-9][a-z0-9_.-]{0,31}", label) is None:
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    return f"mpk-{label}-{os.getpid()}-{secrets.token_hex(8)}"


def remove_docker_container(executable: str, name: str) -> None:
    """Best-effort daemon-side cleanup, including after the Docker CLI is killed."""

    try:
        subprocess.run(
            [executable, "rm", "--force", name],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        # The primary operation still reports its original deterministic error.
        # The unique name avoids ever broadening cleanup to unrelated state.
        pass


def run_checked(
    argv: list[str],
    *,
    environment: dict[str, str] | None = None,
    target_root: Path | None = None,
    target_file_limit: int | None = None,
    target_byte_limit: int | None = None,
    docker_cleanup: tuple[str, str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = run_bounded(
        argv,
        stdout_limit=FROZEN_PROCESS_LIMITS["stdout_bytes"],
        stderr_limit=FROZEN_PROCESS_LIMITS["stderr_bytes"],
        target_root=target_root,
        target_file_limit=target_file_limit,
        target_byte_limit=target_byte_limit,
        environment=environment,
        docker_cleanup=docker_cleanup,
    )
    if result.returncode != 0:
        detail = (result.stdout[-8_192:] + result.stderr[-8_192:]).decode(
            "utf-8", "replace"
        )
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH") from RuntimeError(
            f"command exited {result.returncode}: {argv!r}\n{detail}"
        )
    return result


def ensure_image(reference: str) -> None:
    inspect = subprocess.run(
        [docker_path(), "image", "inspect", reference],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if inspect.returncode == 0:
        return
    run_checked([docker_path(), "pull", "--platform=linux/amd64", reference])


def require_image(reference: str) -> None:
    try:
        result = subprocess.run(
            [docker_path(), "image", "inspect", reference],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    except OSError as error:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
    if result.returncode != 0:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)


def run_bounded(
    argv: list[str],
    *,
    stdout_limit: int,
    stderr_limit: int,
    target_root: Path | None = None,
    target_file_limit: int | None = None,
    target_byte_limit: int | None = None,
    environment: dict[str, str] | None = None,
    docker_cleanup: tuple[str, str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    target_values = (target_root, target_file_limit, target_byte_limit)
    if any(value is not None for value in target_values) and any(
        value is None for value in target_values
    ):
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    checked_boundary(0, stdout_limit, "BUNDLE_REPRODUCIBILITY_MISMATCH")
    checked_boundary(0, stderr_limit, "BUNDLE_REPRODUCIBILITY_MISMATCH")
    if target_file_limit is not None and target_byte_limit is not None:
        checked_boundary(0, target_file_limit, "BUNDLE_REPRODUCIBILITY_MISMATCH")
        checked_boundary(0, target_byte_limit, "BUNDLE_REPRODUCIBILITY_MISMATCH")

    def check_target() -> None:
        if (
            target_root is not None
            and target_file_limit is not None
            and target_byte_limit is not None
        ):
            checked_directory_usage(
                target_root,
                file_limit=target_file_limit,
                byte_limit=target_byte_limit,
                allow_churn=True,
            )

    if docker_cleanup is not None:
        executable, name = docker_cleanup
        if not executable or re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]+", name) is None:
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    process: subprocess.Popen[bytes] | None = None
    streams: selectors.BaseSelector | None = None
    try:
        try:
            process = subprocess.Popen(
                argv,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=environment,
            )
        except OSError as error:
            raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
        if process.stdout is None or process.stderr is None:
            raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
        streams = selectors.DefaultSelector()
        streams.register(
            process.stdout, selectors.EVENT_READ, (bytearray(), stdout_limit)
        )
        streams.register(
            process.stderr, selectors.EVENT_READ, (bytearray(), stderr_limit)
        )
        buffers = [
            streams.get_key(process.stdout).data[0],
            streams.get_key(process.stderr).data[0],
        ]
        while streams.get_map() or process.poll() is None:
            events = streams.select(timeout=0.1) if streams.get_map() else ()
            for key, _events in events:
                buffer, limit = key.data
                block = os.read(key.fileobj.fileno(), min(65_536, limit - len(buffer) + 1))
                if not block:
                    streams.unregister(key.fileobj)
                    continue
                buffer.extend(block)
                if len(buffer) > limit:
                    process.kill()
                    process.wait()
                    raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
            check_target()
            if not streams.get_map() and process.poll() is None:
                try:
                    process.wait(timeout=0.1)
                except subprocess.TimeoutExpired:
                    pass
        return_code = process.wait()
        check_target()
        return subprocess.CompletedProcess(argv, return_code, bytes(buffers[0]), bytes(buffers[1]))
    finally:
        if streams is not None:
            streams.close()
        if process is not None and process.poll() is None:
            process.kill()
            process.wait()
        if docker_cleanup is not None:
            remove_docker_container(*docker_cleanup)


def docker_image_view(reference: str, storage: Path) -> ArchiveView:
    ensure_image(reference)
    created = run_checked(
        [docker_path(), "create", "--platform=linux/amd64", "--entrypoint", "/bin/true", reference]
    )
    identifier = created.stdout.decode("ascii").strip()
    if not re.fullmatch(r"[0-9a-f]{64}", identifier):
        raise RustBuildFailure()
    process: subprocess.Popen[bytes] | None = None
    try:
        process = subprocess.Popen(
            [docker_path(), "export", identifier],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if process.stdout is None:
            raise RustBuildFailure()
        view = ArchiveView.from_stream(process.stdout, storage)
        stderr = process.stderr.read() if process.stderr is not None else b""
        if process.wait() != 0 or stderr:
            raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
        return view
    finally:
        subprocess.run(
            [docker_path(), "rm", "-f", identifier],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )


def inventory_component(root: Path, name: str) -> dict[str, object]:
    component_root = root / name
    files: list[dict[str, object]] = []
    for path in sorted(component_root.rglob("*"), key=lambda item: item.relative_to(root).as_posix().encode("utf-8")):
        metadata = path.lstat()
        if stat.S_ISDIR(metadata.st_mode):
            continue
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise RustBuildFailure()
        relative = path.relative_to(root).as_posix()
        files.append(
            {
                "path": relative,
                "executable": bool(metadata.st_mode & 0o111),
                "size_bytes": metadata.st_size,
                "sha256": raw_hash(path),
            }
        )
    if not files:
        raise RustBuildFailure()
    return {"name": name, "files": files}


def normalize_cache_modes(root: Path, descriptor: dict[str, object]) -> None:
    executable = {
        item["path"]
        for component in descriptor["components"]
        for item in component["files"]
        if item["executable"]
    }
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink():
            raise RustBuildFailure()
        relative = path.relative_to(root).as_posix()
        path.chmod(0o555 if path.is_dir() or relative in executable else 0o444)
    root.chmod(0o555)


def atomic_write(path: Path, bytes_value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".partial")
    if temporary.exists() or temporary.is_symlink():
        raise RustBuildFailure("BUNDLE_ASSEMBLER_IO", 74)
    try:
        with temporary.open("xb") as output:
            output.write(bytes_value)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
        fsync_directory(path.parent)
    except OSError as error:
        raise RustBuildFailure("BUNDLE_ASSEMBLER_IO", 74) from error
    finally:
        if temporary.exists():
            temporary.unlink()


def fsync_directory(path: Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def rename_no_replace(source: Path, destination: Path) -> None:
    library = ctypes.CDLL(None, use_errno=True)
    if sys.platform == "darwin":
        rename = library.renamex_np
        rename.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint]
        result = rename(os.fsencode(source), os.fsencode(destination), 0x00000004)
    elif sys.platform.startswith("linux"):
        rename = library.renameat2
        rename.argtypes = [
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_uint,
        ]
        result = rename(
            -100,
            os.fsencode(source),
            -100,
            os.fsencode(destination),
            0x00000001,
        )
    else:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
    if result == 0:
        return
    error = ctypes.get_errno()
    if error in (errno.EEXIST, errno.ENOTEMPTY):
        raise FileExistsError(error, os.strerror(error), destination)
    raise OSError(error, os.strerror(error), destination)


def publish_cache(staging: Path, final: Path, descriptor: dict[str, object]) -> None:
    if final.name != descriptor["build_inputs_sha256"]:
        raise RustBuildFailure("RUST_BUILD_INPUTS_CACHE_KEY")
    if final.exists() or final.is_symlink():
        try:
            validate_cache(descriptor, root=final)
        except RustBuildFailure as error:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PUBLICATION") from error
        return
    try:
        rename_no_replace(staging, final)
        fsync_directory(final.parent)
    except FileExistsError:
        try:
            validate_cache(descriptor, root=final)
        except RustBuildFailure as error:
            raise RustBuildFailure("RUST_BUILD_INPUTS_PUBLICATION") from error
    except OSError as error:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
    validate_cache(descriptor, root=final)


def archive_top(view: ArchiveView) -> str:
    tops = {name.split("/", 1)[0] for name in set(view.files) | set(view.links)}
    if len(tops) != 1:
        raise RustBuildFailure()
    return next(iter(tops))


def materialize_rust_distribution(
    vector: dict[str, object], downloads: Path, staging: Path, work: Path
) -> None:
    distribution = vector["rust_distribution"]
    manifest_bytes = fetch_bytes(f"{RUST_DIST_ROOT}/channel-rust-nightly.toml", 16 * 1024 * 1024)
    checksum_bytes = fetch_bytes(f"{RUST_DIST_ROOT}/channel-rust-nightly.toml.sha256", 1024)
    checksum_text = checksum_bytes.decode("ascii").strip().split()[0]
    if not re.fullmatch(r"[0-9a-f]{64}", checksum_text):
        raise RustBuildFailure()
    if hashlib.sha256(manifest_bytes).hexdigest() != checksum_text:
        raise RustBuildFailure()
    toolchain = staging / "toolchain"
    toolchain.mkdir(parents=True)
    rust_notice_source: ArchiveView | None = None
    rust_notice_top = ""
    for index, record in enumerate(distribution["archives"]):
        name = record["name"]
        target = record["target"]
        request_name = {
            "clippy-preview": "clippy",
            "llvm-tools-preview": "llvm-tools",
            "rustfmt-preview": "rustfmt",
        }.get(name, name)
        filename = f"{request_name}-nightly-{target}.tar.xz"
        archive = downloads / filename
        download(f"{RUST_DIST_ROOT}/{filename}", record["sha256"], archive)
        manifest_text = manifest_bytes.decode("utf-8", "strict")
        if filename not in manifest_text or record["sha256"] not in manifest_text:
            raise RustBuildFailure()
        view = ArchiveView.from_tar(archive, work / f"rust-archive-{index}")
        top = archive_top(view)
        components_name = f"{top}/components"
        components = view.read_bytes(components_name).decode("utf-8").splitlines()
        if not components:
            raise RustBuildFailure()
        for component in components:
            if re.fullmatch(r"[A-Za-z0-9._-]+", component) is None:
                raise RustBuildFailure()
            view.copy_prefix(
                f"{top}/{component}",
                toolchain,
                exclude={"manifest.in"},
                exclude_prefixes={"lib/rustlib/rustc-src"},
            )
        if rust_notice_source is None and all(
            f"{top}/{notice}" in view.files
            for notice in ("COPYRIGHT", "LICENSE-APACHE", "LICENSE-MIT")
        ):
            rust_notice_source = view
            rust_notice_top = top
    if rust_notice_source is None:
        raise RustBuildFailure()
    notices = staging / "notices"
    notices.mkdir(parents=True)
    for source, destination in (
        ("COPYRIGHT", "Rust-COPYRIGHT.txt"),
        ("LICENSE-APACHE", "Rust-LICENSE-APACHE.txt"),
        ("LICENSE-MIT", "Rust-LICENSE-MIT.txt"),
    ):
        rust_notice_source.copy_file(
            f"{rust_notice_top}/{source}", notices / destination, executable=False
        )
    rustc = toolchain / "bin/rustc"
    cargo = toolchain / "bin/cargo"
    if not rustc.is_file() or not cargo.is_file():
        raise RustBuildFailure()


def fetch_bytes(url: str, maximum: int) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "mpk-rust-build-inputs-v0"})
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            value = response.read(maximum + 1)
    except (OSError, urllib.error.URLError) as error:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
    if len(value) > maximum:
        raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
    return value


def materialize_llvm(
    vector: dict[str, object], downloads: Path, staging: Path, work: Path
) -> None:
    native = vector["native_origins"]
    archive = downloads / native["llvm"]["archive"]
    download(LLVM_URL, native["llvm"]["sha256"], archive)
    view = ArchiveView.from_tar(archive, work / "llvm-archive")
    top = archive_top(view)
    toolchain = staging / "toolchain"
    for executable in ("clang", "clang++", "ld.lld", "llvm-ar", "llvm-ranlib", "llvm-strip"):
        view.copy_file(f"{top}/bin/{executable}", toolchain / "bin" / executable, executable=True)
    clang_roots = [
        name
        for name in set(view.files) | set(view.links)
        if name.startswith(f"{top}/lib/clang/")
    ]
    if not clang_roots:
        raise RustBuildFailure()
    view.copy_prefix(f"{top}/lib/clang", toolchain / "lib/clang")
    view.copy_file(
        f"{top}/include/llvm/Support/LICENSE.TXT",
        staging / "notices/LLVM-LICENSE.txt",
        executable=False,
    )
    templates = raw_templates(vector)
    config = toolchain / "bin/clang.cfg"
    config.write_bytes(templates["clang_config"])
    config.chmod(0o644)
    view.copy_file(
        f"{top}/lib/clang/18/lib/x86_64-unknown-linux-gnu/libclang_rt.asan.so",
        toolchain
        / "lib/clang/18/lib/x86_64-unknown-linux-gnu/libclang_rt.asan-x86_64.so",
        executable=True,
    )
    for required in vector["production_projection"]["compiler_rt_files"]:
        if not (staging / required).is_file():
            raise RustBuildFailure()


def materialize_native_sysroot(
    vector: dict[str, object], staging: Path, work: Path
) -> ArchiveView:
    build_view = docker_image_view(BUILD_IMAGE, work / "build-image")
    projection = vector["production_projection"]["native_development_projection"]
    output = staging / projection["output_root"]
    output.mkdir(parents=True)
    for source in projection["source_recursive_roots"]:
        relative = source.lstrip("/")
        build_view.copy_prefix(relative, output / relative)
    for required in projection["required_output_paths"]:
        if not (staging / required).is_file() or (staging / required).is_symlink():
            raise RustBuildFailure()
    return build_view


def registry_packages(vector: dict[str, object]) -> dict[tuple[str, str], dict[str, object]]:
    result: dict[tuple[str, str], dict[str, object]] = {}
    for graph in vector["dependency_graphs"].values():
        for package in graph:
            checksum = package.get("checksum")
            if checksum is None:
                continue
            if package.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
                raise RustBuildFailure()
            key = (package["name"], package["version"])
            if key in result and result[key] != package:
                raise RustBuildFailure()
            result[key] = package
    return result


def materialize_sources_and_vendor(
    vector: dict[str, object], downloads: Path, staging: Path, work: Path
) -> None:
    cargo_fuzz = vector["valid_descriptor"]["cargo_fuzz"]
    archive = downloads / "cargo-fuzz-0.13.1.tar.gz"
    download(CARGO_FUZZ_URL, cargo_fuzz["archive_sha256"], archive, 128 * 1024 * 1024)
    source_view = ArchiveView.from_tar(archive, work / "cargo-fuzz-source")
    top = archive_top(source_view)
    source_root = staging / "tool-sources/cargo-fuzz"
    source_view.copy_prefix(top, source_root)
    templates = raw_templates(vector)
    if (
        (source_root / "Cargo.toml").read_bytes() != templates["cargo_fuzz_manifest"]
        or (source_root / "Cargo.lock").read_bytes() != templates["cargo_fuzz_lock"]
    ):
        raise RustBuildFailure()
    vendor = staging / "vendor"
    vendor.mkdir(parents=True)
    package_groups: dict[str, list[str]] = {}
    license_records = {
        record["spdx"]: record["id"]
        for record in vector["valid_descriptor"]["licenses"]
        if record["id"].startswith("vendor-")
    }
    legacy = vector["license_normalization"]["legacy_exact_mappings"]
    for index, ((name, version), package) in enumerate(sorted(registry_packages(vector).items())):
        filename = f"{name}-{version}.crate"
        crate = downloads / filename
        url = f"https://static.crates.io/crates/{name}/{filename}"
        download(url, package["checksum"], crate, 256 * 1024 * 1024)
        view = ArchiveView.from_tar(crate, work / f"crate-{index}")
        top = archive_top(view)
        expected_top = f"{name}-{version}"
        if top != expected_top:
            raise RustBuildFailure()
        package_root = vendor / expected_top
        view.copy_prefix(top, package_root)
        files: dict[str, str] = {}
        for path in sorted(package_root.rglob("*")):
            if path.is_file() and not path.is_symlink():
                files[path.relative_to(package_root).as_posix()] = raw_hash(path)
        checksum_value = {"files": files, "package": package["checksum"]}
        (package_root / ".cargo-checksum.json").write_bytes(canonical(checksum_value))
        normalized_license = legacy.get(package.get("license"), package.get("license"))
        license_id = license_records.get(normalized_license)
        if license_id is None:
            raise RustBuildFailure()
        license_files = sorted(
            path.relative_to(package_root).as_posix()
            for path in package_root.rglob("*")
            if path.is_file()
            and re.match(r"^(?:LICENSE|COPYING|NOTICE|UNLICENSE)", path.name, re.IGNORECASE)
        )
        package_groups.setdefault(license_id, []).append(
            f"{name}@{version} {normalized_license} {' '.join(license_files)}"
        )
    notice_root = staging / "notices/vendor"
    notice_root.mkdir(parents=True)
    for identifier in sorted(license_records.values()):
        entries = sorted(package_groups.get(identifier, []), key=lambda item: item.encode("utf-8"))
        if not entries:
            raise RustBuildFailure()
        (notice_root / f"{identifier}.txt").write_text(
            "mpk.rust.build_inputs.v0 license group\n" + "\n".join(entries) + "\n",
            encoding="utf-8",
        )
    ubuntu_notice = (
        "Ubuntu 18.04 build and runtime projection\n"
        f"development={vector['native_origins']['development_sysroot']['platform_digest']}\n"
        f"runtime={vector['native_origins']['runtime']['platform_digest']}\n"
    )
    (staging / "notices/Ubuntu-Bionic-NOTICE.txt").write_text(ubuntu_notice, encoding="utf-8")


def materialize_cargo_home(vector: dict[str, object], staging: Path) -> None:
    root = staging / "cargo-home-seed"
    root.mkdir(parents=True)
    (root / "config.toml").write_bytes(raw_templates(vector)["cargo_home_config"])


def docker_build_environment(vector: dict[str, object]) -> list[str]:
    launcher = vector["launcher"]
    values = dict(launcher["evidence_environment"])
    for name in launcher["build_environment_remove"]:
        values.pop(name, None)
    values.update(launcher["build_environment_add"])
    values["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
        launcher["non_fuzz_encoded_rustflags_elements"]
    )
    return [f"{name}={values[name]}" for name in sorted(values)]


def common_build_docker(
    staging: Path,
    cargo_home: Path,
    target: Path,
    container_name: str,
) -> tuple[list[str], int]:
    resource_arguments, virtual_memory_kib = launcher_resource_controls(
        FROZEN_PROCESS_LIMITS
    )
    return [
        docker_path(),
        "run",
        f"--name={container_name}",
        "--rm",
        "--pull=never",
        "--network=none",
        "--platform=linux/amd64",
        "--read-only",
        "--hostname=mpk-build",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        *resource_arguments,
        f"--mount=type=bind,src={staging / 'toolchain'},dst=/mpk/toolchain,readonly",
        f"--mount=type=bind,src={staging / 'native-sysroot'},dst=/mpk/native-sysroot,readonly",
        f"--mount=type=bind,src={staging / 'vendor'},dst=/mpk/vendor,readonly",
        f"--mount=type=bind,src={cargo_home},dst=/mpk/cargo-home",
        f"--mount=type=bind,src={target},dst=/mpk/target",
        RUNTIME_IMAGE,
    ], virtual_memory_kib


def frozen_shell_command(command: list[str], virtual_memory_kib: int) -> list[str]:
    return [
        "/bin/sh",
        "-ceu",
        f'ulimit -v {virtual_memory_kib}; umask 022; exec "$@"',
        "mpk-build",
        *command,
    ]


def fresh_cargo_home(vector: dict[str, object], parent: Path, name: str) -> Path:
    result = parent / name
    result.mkdir()
    (result / "config.toml").write_bytes(raw_templates(vector)["cargo_home_config"])
    (result / "config.toml").chmod(0o444)
    return result


def build_cargo_fuzz_twice(
    vector: dict[str, object], staging: Path, work: Path
) -> None:
    observed: list[Path] = []
    for index in range(2):
        target = work / f"cargo-fuzz-target-{index}"
        target.mkdir()
        cargo_home = fresh_cargo_home(vector, work, f"cargo-fuzz-home-{index}")
        container_name = fresh_container_name("cargo-fuzz-build")
        argv, virtual_memory_kib = common_build_docker(
            staging, cargo_home, target, container_name
        )
        argv[-1:-1] = [
            f"--mount=type=bind,src={staging / 'tool-sources/cargo-fuzz'},dst=/mpk/frontend,readonly",
            "--workdir=/mpk/frontend",
        ]
        argv.extend(
            frozen_shell_command(
                [
                "/usr/bin/env",
                "-i",
                *docker_build_environment(vector),
                *vector["valid_descriptor"]["cargo_fuzz"]["build_argv"],
                ],
                virtual_memory_kib,
            )
        )
        run_checked(
            argv,
            target_root=target,
            target_file_limit=FROZEN_PROCESS_LIMITS["output_files"],
            target_byte_limit=FROZEN_PROCESS_LIMITS["target_bytes"],
            docker_cleanup=(docker_path(), container_name),
        )
        validate_post_run_cargo_home(cargo_home, vector)
        checked_directory_usage(
            target,
            file_limit=FROZEN_PROCESS_LIMITS["output_files"],
            byte_limit=FROZEN_PROCESS_LIMITS["target_bytes"],
        )
        executable = target / "x86_64-unknown-linux-gnu/release/cargo-fuzz"
        if not executable.is_file() or executable.is_symlink():
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        observed.append(executable)
    if not same_file_bytes(observed[0], observed[1]):
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    destination = staging / "toolchain/bin/cargo-fuzz"
    copy_no_replace(observed[0], destination)
    destination.chmod(0o755)


def build_libfuzzer_twice(vector: dict[str, object], staging: Path, work: Path) -> None:
    recipe = vector["fuzz_smoke"]["bounded_child_process_graph"]["prebuilt_libfuzzer"]
    archives: list[Path] = []
    for build_index in range(2):
        scratch = work / f"libfuzzer-{build_index}"
        output = scratch / "libfuzzer-build"
        objects = output / "objects"
        objects.mkdir(parents=True)

        def run_recipe(command: list[str], label: str) -> None:
            container_name = fresh_container_name(label)
            resource_arguments, virtual_memory_kib = launcher_resource_controls(
                FROZEN_PROCESS_LIMITS
            )
            # /mpk/tmp is a monitored bind for this recipe, rather than a
            # second unobservable tmpfs mounted over the build output.
            resource_arguments = [
                argument
                for argument in resource_arguments
                if not argument.startswith("--tmpfs=/mpk/tmp:")
            ]
            argv = [
                docker_path(),
                "run",
                f"--name={container_name}",
                "--rm",
                "--pull=never",
                "--network=none",
                "--platform=linux/amd64",
                "--read-only",
                "--hostname=mpk-build",
                "--cap-drop=ALL",
                "--security-opt=no-new-privileges",
                *resource_arguments,
                f"--mount=type=bind,src={staging / 'toolchain'},dst=/mpk/toolchain,readonly",
                f"--mount=type=bind,src={staging / 'native-sysroot'},dst=/mpk/native-sysroot,readonly",
                f"--mount=type=bind,src={staging / 'vendor'},dst=/mpk/vendor,readonly",
                f"--mount=type=bind,src={scratch},dst=/mpk/tmp",
                RUNTIME_IMAGE,
                *frozen_shell_command(command, virtual_memory_kib),
            ]
            run_checked(
                argv,
                target_root=scratch,
                target_file_limit=FROZEN_PROCESS_LIMITS["output_files"],
                target_byte_limit=FROZEN_PROCESS_LIMITS["temp_bytes"],
                docker_cleanup=(docker_path(), container_name),
            )

        for source in recipe["sources"]:
            object_name = source[:-4] + ".o"
            compile_argv = [
                item.replace("SOURCE", source).replace("OBJECT", object_name)
                for item in recipe["compile_argv_template"]
            ]
            run_recipe(compile_argv, "libfuzzer-compile")
        run_recipe(recipe["archive_argv"], "libfuzzer-archive")
        run_recipe(recipe["ranlib_argv"], "libfuzzer-ranlib")
        checked_directory_usage(
            scratch,
            file_limit=FROZEN_PROCESS_LIMITS["output_files"],
            byte_limit=FROZEN_PROCESS_LIMITS["temp_bytes"],
        )
        archive = output / "libfuzzer.a"
        if not archive.is_file():
            raise RustBuildFailure()
        archives.append(archive)
    if not same_file_bytes(archives[0], archives[1]):
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    destination = staging / recipe["installed_component_path"]
    copy_no_replace(archives[0], destination)
    destination.chmod(0o644)


def elf_dynamic(
    path: Path, *, allowed_runpath: str | None = None
) -> tuple[str | None, list[str]]:
    data = path.read_bytes()
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise RustBuildFailure()
    if struct.unpack_from("<H", data, 18)[0] != 62:
        raise RustBuildFailure()
    phoff = struct.unpack_from("<Q", data, 32)[0]
    phentsize = struct.unpack_from("<H", data, 54)[0]
    phnum = struct.unpack_from("<H", data, 56)[0]
    if phentsize < 56 or phoff + phentsize * phnum > len(data):
        raise RustBuildFailure()
    interpreter: str | None = None
    dynamic: tuple[int, int] | None = None
    loads: list[tuple[int, int, int, int]] = []
    for index in range(phnum):
        offset = phoff + index * phentsize
        kind, _flags, file_offset, virtual, _physical, file_size, memory_size, _align = struct.unpack_from(
            "<IIQQQQQQ", data, offset
        )
        if kind == 3:
            if interpreter is not None:
                raise RustBuildFailure()
            raw = data[file_offset : file_offset + file_size]
            if not raw.endswith(b"\0"):
                raise RustBuildFailure()
            interpreter = raw[:-1].decode("ascii")
        elif kind == 2:
            if dynamic is not None:
                raise RustBuildFailure()
            dynamic = (file_offset, file_size)
        elif kind == 1:
            loads.append((virtual, memory_size, file_offset, file_size))
    if dynamic is None:
        if interpreter is not None:
            raise RustBuildFailure()
        return None, []
    tags: list[tuple[int, int]] = []
    start, size = dynamic
    if start + size > len(data) or size % 16:
        raise RustBuildFailure()
    terminated = False
    for offset in range(start, start + size, 16):
        tag, value = struct.unpack_from("<qQ", data, offset)
        if tag == 0:
            terminated = True
            break
        tags.append((tag, value))
    if not terminated:
        raise RustBuildFailure()
    strtab_va = next((value for tag, value in tags if tag == 5), None)
    strtab_size = next((value for tag, value in tags if tag == 10), None)
    if strtab_va is None or strtab_size is None:
        raise RustBuildFailure()
    strtab_offset = None
    for virtual, memory_size, file_offset, file_size in loads:
        if virtual <= strtab_va < virtual + memory_size:
            delta = strtab_va - virtual
            if delta + strtab_size > file_size:
                raise RustBuildFailure()
            strtab_offset = file_offset + delta
            break
    if strtab_offset is None or strtab_offset + strtab_size > len(data):
        raise RustBuildFailure()
    table = data[strtab_offset : strtab_offset + strtab_size]

    def dynamic_string(offset: int) -> str:
        if offset >= len(table):
            raise RustBuildFailure()
        end = table.find(b"\0", offset)
        if end < 0:
            raise RustBuildFailure()
        return table[offset:end].decode("ascii")

    loader_paths = [(tag, dynamic_string(value)) for tag, value in tags if tag in (15, 29)]
    if loader_paths and loader_paths != [(29, allowed_runpath)]:
        raise RustBuildFailure() from ValueError(
            f"ELF RPATH/RUNPATH rejected: {path}: {loader_paths!r}"
        )
    needed: list[str] = []
    for tag, value in tags:
        if tag != 1:
            continue
        soname = dynamic_string(value)
        if not soname or "/" in soname or "\\" in soname or soname in needed:
            raise RustBuildFailure()
        needed.append(soname)
    return interpreter, needed


def internal_library(toolchain: Path, soname: str) -> Path | None:
    candidates = [
        path
        for path in toolchain.rglob(soname)
        if path.is_file() and not path.is_symlink()
    ]
    if not candidates:
        return None
    first = candidates[0]
    if any(not same_file_bytes(path, first) for path in candidates[1:]):
        raise RustBuildFailure()
    return first


def materialize_native_runtime(
    vector: dict[str, object], staging: Path, work: Path
) -> None:
    runtime_view = docker_image_view(RUNTIME_IMAGE, work / "runtime-image")
    projection = vector["production_projection"]["native_runtime_projection"]
    interpreter_source = projection["interpreter_source"].lstrip("/")
    runtime_view.copy_file(
        interpreter_source,
        staging / projection["interpreter_output"],
        executable=True,
    )
    toolchain = staging / "toolchain"
    queue: list[tuple[Path, bool]] = []
    for relative in vector["production_projection"]["toolchain_executables"]:
        executable = staging / relative
        if not executable.is_file():
            raise RustBuildFailure()
        queue.append((executable, True))
    copied: dict[str, Path] = {}
    inspected: set[str] = set()
    while queue:
        executable, requires_interpreter = queue.pop(0)
        digest = raw_hash(executable)
        if digest in inspected:
            continue
        inspected.add(digest)
        allowed_runpath = (
            projection["toolchain_runpath"]
            if executable.is_relative_to(toolchain)
            else None
        )
        interpreter, needed = elf_dynamic(executable, allowed_runpath=allowed_runpath)
        expected_interpreter = vector["native_origins"]["runtime"]["interpreter"]
        if (requires_interpreter and interpreter != expected_interpreter) or (
            not requires_interpreter and interpreter not in (None, expected_interpreter)
        ):
            raise RustBuildFailure() from ValueError(
                f"ELF interpreter rejected: {executable}: {interpreter!r}: "
                f"entrypoint={requires_interpreter}"
            )
        for soname in needed:
            internal = internal_library(toolchain, soname)
            if internal is not None:
                queue.append((internal, False))
                continue
            terminal_candidates: list[tuple[int, Path]] = []
            for directory in projection["soname_source_directories"]:
                name = f"{directory.lstrip('/')}/{soname}"
                try:
                    terminal_candidates.append(runtime_view.resolve(name))
                except RustBuildFailure:
                    pass
            if not terminal_candidates:
                raise RustBuildFailure()
            first = terminal_candidates[0][1]
            if any(
                not same_file_bytes(candidate, first)
                for _mode, candidate in terminal_candidates[1:]
            ):
                raise RustBuildFailure()
            if soname not in copied:
                destination = staging / projection["library_output_directory"] / soname
                copy_no_replace(first, destination)
                destination.chmod(0o644)
                copied[soname] = destination
                queue.append((destination, False))


def create_production_descriptor(vector: dict[str, object], staging: Path) -> dict[str, object]:
    descriptor = copy.deepcopy(vector["valid_descriptor"])
    components_by_name = {component["name"]: component for component in descriptor["components"]}
    for name in sorted(components_by_name):
        observed = inventory_component(staging, name)
        components_by_name[name]["files"] = observed["files"]
    descriptor["cargo_fuzz"]["executable_sha256"] = raw_hash(
        staging / "toolchain/bin/cargo-fuzz"
    )
    descriptor.pop("build_inputs_sha256", None)
    descriptor["build_inputs_sha256"] = build_inputs_hash(descriptor)
    validate_descriptor_model(descriptor, vector)
    return descriptor


def materialize_production_cache(vector: dict[str, object], staging: Path, work: Path) -> dict[str, object]:
    downloads = work / "downloads"
    downloads.mkdir()
    materialize_rust_distribution(vector, downloads, staging, work)
    materialize_llvm(vector, downloads, staging, work)
    materialize_native_sysroot(vector, staging, work)
    materialize_sources_and_vendor(vector, downloads, staging, work)
    materialize_cargo_home(vector, staging)
    build_cargo_fuzz_twice(vector, staging, work)
    build_libfuzzer_twice(vector, staging, work)
    materialize_native_runtime(vector, staging, work)
    descriptor = create_production_descriptor(vector, staging)
    normalize_cache_modes(staging, descriptor)
    validate_cache(descriptor, root=staging)
    return descriptor


def update_build_inputs(*, provision: bool = False) -> None:
    vector = load_vector()
    validate_project_templates(vector)
    existing: dict[str, object] | None = None
    existing_transport: bytes | None = None
    if provision:
        existing, existing_transport = read_descriptor(vector)
    parent = cache_parent()
    parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".rust-build-inputs-stage-", dir=parent))
    try:
        with tempfile.TemporaryDirectory(prefix="mpk-rust-build-inputs-") as temporary:
            descriptor = materialize_production_cache(vector, staging, Path(temporary))
        transport = descriptor_transport(descriptor)
        if provision and (descriptor != existing or transport != existing_transport):
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        final = parent / descriptor["build_inputs_sha256"]
        publish_cache(staging, final, descriptor)
        if provision:
            return
        atomic_write(descriptor_path(), transport)
    finally:
        if staging.exists():
            staging.chmod(0o700)
            for path in staging.rglob("*"):
                try:
                    path.chmod(0o700 if path.is_dir() else 0o600)
                except OSError:
                    pass
            shutil.rmtree(staging, ignore_errors=True)


def tree_source_inventory(
    root: Path, *, excluded_top_levels: frozenset[str] = frozenset()
) -> list[tuple[str, int, str, bool, int, int]]:
    root_metadata = root.lstat()
    if root.is_symlink() or not stat.S_ISDIR(root_metadata.st_mode):
        raise RustBuildFailure()
    result: list[tuple[str, int, str, bool, int, int]] = []
    identities: set[tuple[int, int]] = set()
    for path in sorted(root.rglob("*"), key=lambda item: item.relative_to(root).as_posix().encode("utf-8")):
        relative = path.relative_to(root).as_posix()
        if relative.split("/", 1)[0] in excluded_top_levels:
            continue
        metadata = path.lstat()
        if stat.S_ISDIR(metadata.st_mode):
            if path.is_symlink():
                raise RustBuildFailure()
            continue
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise RustBuildFailure()
        identity = (metadata.st_dev, metadata.st_ino)
        if identity in identities:
            raise RustBuildFailure()
        identities.add(identity)
        result.append(
            (
                relative,
                metadata.st_size,
                raw_hash(path),
                bool(metadata.st_mode & 0o111),
                metadata.st_dev,
                metadata.st_ino,
            )
        )
    return result


def tree_inventory(root: Path) -> list[tuple[str, int, str, bool]]:
    return [item[:4] for item in tree_source_inventory(root)]


def copy_tree_snapshot(source: Path, destination: Path) -> list[tuple[str, int, str, bool]]:
    before = tree_source_inventory(source)
    destination.mkdir(parents=True)
    for relative, size, digest, executable, device, inode in before:
        source_file = source / relative
        destination_file = destination / relative
        destination_file.parent.mkdir(parents=True, exist_ok=True)
        with source_file.open("rb") as input_stream, destination_file.open("xb") as output_stream:
            opened = os.fstat(input_stream.fileno())
            if (opened.st_dev, opened.st_ino, opened.st_size) != (device, inode, size):
                raise RustBuildFailure()
            shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
            closed = os.fstat(input_stream.fileno())
            if (closed.st_dev, closed.st_ino, closed.st_size) != (device, inode, size):
                raise RustBuildFailure()
        if destination_file.stat().st_size != size or raw_hash(destination_file) != digest:
            raise RustBuildFailure()
        destination_file.chmod(0o555 if executable else 0o444)
    if tree_source_inventory(source) != before:
        raise RustBuildFailure()
    for path in sorted(destination.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
    destination.chmod(0o555)
    return [item[:4] for item in before]


def frontend_source_state() -> list[tuple[str, int, str, bool, int, int]]:
    project = repository_root() / "rust-tools/rust2vir"
    allowed = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src", "tests", "testdata", "fuzz"}
    observed = {path.name for path in project.iterdir() if path.name != "target"}
    if not observed <= allowed or not {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src", "tests"} <= observed:
        raise RustBuildFailure()
    state = tree_source_inventory(project, excluded_top_levels=frozenset({"target"}))
    folded: set[str] = set()
    for relative, _size, _digest, _executable, _device, _inode in state:
        if not portable_path(relative):
            raise RustBuildFailure()
        key = relative.lower()
        if key in folded:
            raise RustBuildFailure()
        folded.add(key)
        if relative.startswith("tests/") and relative.endswith(".rs"):
            stem = PurePosixPath(relative).stem
            if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", stem) is None:
                raise RustBuildFailure()
    return state


def current_frontend_inventory() -> list[tuple[str, int, str, bool]]:
    return [item[:4] for item in frontend_source_state()]


def capture_frontend_project(destination: Path) -> list[tuple[str, int, str, bool]]:
    project = repository_root() / "rust-tools/rust2vir"
    before = frontend_source_state()
    destination.mkdir(parents=True)
    for relative, size, digest, executable, device, inode in before:
        source = project / relative
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        with source.open("rb") as input_stream, target.open("xb") as output_stream:
            opened = os.fstat(input_stream.fileno())
            if (opened.st_dev, opened.st_ino, opened.st_size) != (device, inode, size):
                raise RustBuildFailure()
            shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
            closed = os.fstat(input_stream.fileno())
            if (closed.st_dev, closed.st_ino, closed.st_size) != (device, inode, size):
                raise RustBuildFailure()
        if target.stat().st_size != size or raw_hash(target) != digest:
            raise RustBuildFailure()
        target.chmod(0o555 if executable else 0o444)
    if frontend_source_state() != before:
        raise RustBuildFailure()
    for path in sorted(destination.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
    destination.chmod(0o555)
    return [item[:4] for item in before]


def copy_cache_snapshot(source: Path, destination: Path) -> None:
    copy_tree_snapshot(source, destination)


def capture_candidate_outputs(source: Path, destination: Path) -> None:
    relatives = (
        "x86_64-unknown-linux-gnu/release/rust2vir",
        "x86_64-unknown-linux-gnu/release/rust2vir-driver",
    )
    destination.mkdir(parents=True)
    for relative in relatives:
        source_file = source / relative
        metadata = source_file.lstat()
        if (
            source_file.is_symlink()
            or not stat.S_ISREG(metadata.st_mode)
            or not metadata.st_mode & 0o111
        ):
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        digest = raw_hash(source_file)
        destination_file = destination / relative
        destination_file.parent.mkdir(parents=True, exist_ok=True)
        with source_file.open("rb") as input_stream, destination_file.open(
            "xb"
        ) as output_stream:
            opened = os.fstat(input_stream.fileno())
            if (opened.st_dev, opened.st_ino, opened.st_size) != (
                metadata.st_dev,
                metadata.st_ino,
                metadata.st_size,
            ):
                raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
            shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
            closed = os.fstat(input_stream.fileno())
            if (closed.st_dev, closed.st_ino, closed.st_size) != (
                metadata.st_dev,
                metadata.st_ino,
                metadata.st_size,
            ):
                raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        if raw_hash(source_file) != digest or raw_hash(destination_file) != digest:
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        destination_file.chmod(0o555)
    for path in sorted(destination.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
    destination.chmod(0o555)


def accepted_launcher_arguments(vector: dict[str, object], arguments: list[str]) -> bool:
    if arguments in vector["launcher"]["modes"]:
        return True
    integration = vector["launcher"]["integration_test_mode"]["argv"]
    if len(arguments) == len(integration):
        variable_index = integration.index("TEST")
        if all(
            actual == expected
            for index, (actual, expected) in enumerate(zip(arguments, integration))
            if index != variable_index
        ):
            test_name = arguments[variable_index]
            if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", test_name):
                test_file = repository_root() / f"rust-tools/rust2vir/tests/{test_name}.rs"
                return test_file.is_file() and not test_file.is_symlink()
    return False


def checked_directory_usage(
    root: Path,
    *,
    file_limit: int,
    byte_limit: int,
    directory_limit: int = TARGET_DIRECTORY_LIMIT,
    allow_churn: bool = False,
) -> tuple[int, int]:
    files = 0
    size = 0
    code = "BUNDLE_REPRODUCIBILITY_MISMATCH"
    checked_boundary(file_limit, U64_MAX, code)
    checked_boundary(byte_limit, U64_MAX, code)
    checked_boundary(directory_limit, U64_MAX, code)
    node_limit = checked_boundary_add(directory_limit, file_limit, U64_MAX, code)
    try:
        root_metadata = root.lstat()
        if (
            not stat.S_ISDIR(root_metadata.st_mode)
            or stat.S_ISLNK(root_metadata.st_mode)
        ):
            raise RustBuildFailure(code)
        directories = checked_boundary(1, directory_limit, code)
        nodes = checked_boundary(1, node_limit, code)
        pending = [(root, root_metadata.st_dev, root_metadata.st_ino)]
        while pending:
            directory, expected_device, expected_inode = pending.pop()
            descriptor = -1
            try:
                descriptor = os.open(
                    directory,
                    os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW,
                )
                directory_metadata = os.fstat(descriptor)
                if (
                    directory_metadata.st_dev != expected_device
                    or directory_metadata.st_ino != expected_inode
                ):
                    if allow_churn:
                        continue
                    raise RustBuildFailure(code)
                with os.scandir(descriptor) as entries:
                    for entry in entries:
                        try:
                            metadata = entry.stat(follow_symlinks=False)
                        except FileNotFoundError:
                            if allow_churn:
                                continue
                            raise
                        nodes = checked_boundary_add(nodes, 1, node_limit, code)
                        if stat.S_ISDIR(metadata.st_mode):
                            directories = checked_boundary_add(
                                directories, 1, directory_limit, code
                            )
                            pending.append(
                                (
                                    directory / entry.name,
                                    metadata.st_dev,
                                    metadata.st_ino,
                                )
                            )
                        elif stat.S_ISREG(metadata.st_mode):
                            files = checked_boundary_add(files, 1, file_limit, code)
                            size = checked_boundary_add(
                                size, metadata.st_size, byte_limit, code
                            )
                        else:
                            raise RustBuildFailure(code)
            except (FileNotFoundError, NotADirectoryError):
                if not allow_churn:
                    raise
            finally:
                if descriptor >= 0:
                    os.close(descriptor)
    except OSError as error:
        raise RustBuildFailure(code) from error
    return files, size


def validate_post_run_cargo_home(root: Path, vector: dict[str, object]) -> None:
    allowed = frozenset(vector["cargo_home_post_run_allowlist"])
    scanned = bounded_directory_entries(
        root,
        maximum=len(allowed),
        code="BUNDLE_REPRODUCIBILITY_MISMATCH",
    )
    entries = {name: metadata for name, metadata in scanned}
    if not entries.keys() <= allowed or "config.toml" not in entries:
        raise RustBuildFailure() from ValueError(
            "post-run Cargo home entries rejected: "
            + repr(
                [
                    (name, metadata.st_size, stat.S_IMODE(metadata.st_mode))
                    for name, metadata in scanned
                ]
            )
        )
    expected_config = raw_templates(vector)["cargo_home_config"]
    if read_bounded_regular_file(
        root / "config.toml",
        maximum=len(expected_config),
        code="BUNDLE_REPRODUCIBILITY_MISMATCH",
    ) != expected_config:
        raise RustBuildFailure()
    for name in (".package-cache", ".package-cache-mutate"):
        metadata = entries.get(name)
        if metadata is not None and (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or stat.S_IMODE(metadata.st_mode) != 0o644
            or metadata.st_size != 0
        ):
            raise RustBuildFailure()
    metadata = entries.get(".global-cache")
    if metadata is not None:
        profile = vector["cargo_home_post_run_global_cache"]
        magic = bytes.fromhex(profile["magic_hex"])
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or stat.S_IMODE(metadata.st_mode) != 0o644
            or not len(magic) <= metadata.st_size <= profile["maximum_size_bytes"]
        ):
            raise RustBuildFailure()
        global_bytes = read_bounded_regular_file(
            root / ".global-cache",
            maximum=profile["maximum_size_bytes"],
            code="BUNDLE_REPRODUCIBILITY_MISMATCH",
        )
        if global_bytes[: len(magic)] != magic:
            raise RustBuildFailure()


def launcher_resource_controls(limits: object) -> tuple[list[str], int]:
    if not isinstance(limits, dict) or limits != FROZEN_PROCESS_LIMITS:
        raise RustBuildFailure()
    for value in limits.values():
        checked_boundary(value, U64_MAX, "BUNDLE_REPRODUCIBILITY_MISMATCH")
    if limits["virtual_memory_bytes"] % 1024 != 0:
        raise RustBuildFailure()
    return (
        [
            f"--pids-limit={limits['processes']}",
            f"--ulimit=nofile={limits['open_files_per_process']}:{limits['open_files_per_process']}",
            f"--memory={limits['resident_memory_bytes']}",
            f"--memory-swap={limits['resident_memory_bytes']}",
            "--tmpfs=/mpk/home:rw,nosuid,nodev,noexec,mode=700",
            f"--tmpfs=/mpk/tmp:rw,nosuid,nodev,noexec,mode=700,size={limits['temp_bytes']}",
        ],
        limits["virtual_memory_bytes"] // 1024,
    )


def hermetic_docker_argv(
    vector: dict[str, object],
    snapshot: Path,
    arguments: list[str],
    container_name: str,
) -> list[str]:
    runtime = snapshot / "cache/native-runtime"
    limits = vector["launcher"]["process_limits"]
    resource_arguments, virtual_memory_kib = launcher_resource_controls(limits)
    command = ["/mpk/toolchain/bin/cargo", *arguments[1:]]
    return [
        docker_path(),
        "run",
        f"--name={container_name}",
        "--rm",
        "--pull=never",
        "--network=none",
        "--platform=linux/amd64",
        "--read-only",
        "--hostname=mpk-build",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        *resource_arguments,
        f"--mount=type=bind,src={snapshot / 'frontend'},dst=/mpk/frontend,readonly",
        f"--mount=type=bind,src={snapshot / 'cache/toolchain'},dst=/mpk/toolchain,readonly",
        f"--mount=type=bind,src={snapshot / 'cache/vendor'},dst=/mpk/vendor,readonly",
        f"--mount=type=bind,src={snapshot / 'cache/native-sysroot'},dst=/mpk/native-sysroot,readonly",
        f"--mount=type=bind,src={snapshot / 'cache/native-runtime'},dst=/mpk/native-runtime,readonly",
        f"--mount=type=bind,src={runtime / 'lib64'},dst=/lib64,readonly",
        f"--mount=type=bind,src={runtime / 'lib/x86_64-linux-gnu'},dst=/lib/x86_64-linux-gnu,readonly",
        f"--mount=type=bind,src={runtime / 'lib/x86_64-linux-gnu'},dst=/usr/lib/x86_64-linux-gnu,readonly",
        f"--mount=type=bind,src={snapshot / 'cargo-home'},dst=/mpk/cargo-home",
        f"--mount=type=bind,src={snapshot / 'target'},dst=/mpk/target",
        "--workdir=/mpk/frontend",
        RUNTIME_IMAGE,
        "/bin/sh",
        "-ceu",
        f"ulimit -v {virtual_memory_kib}; umask 022; exec /usr/bin/env \"$@\"",
        "mpk-launch",
        "-i",
        *docker_build_environment(vector),
        *command,
    ]


def run_hermetic(
    arguments: list[str], *, retained_target: Path | None = None, retained_cache: Path | None = None
) -> subprocess.CompletedProcess[bytes]:
    vector = load_vector()
    if not accepted_launcher_arguments(vector, arguments):
        raise RustBuildFailure("BUNDLE_ASSEMBLER_USAGE", 64)
    descriptor, cache = check_build_inputs()
    require_image(RUNTIME_IMAGE)
    with tempfile.TemporaryDirectory(
        prefix=".rust2vir-launch-", dir=cache_parent()
    ) as temporary:
        snapshot = Path(temporary) / "snapshot"
        snapshot.mkdir()
        source_inventory = capture_frontend_project(snapshot / "frontend")
        copy_cache_snapshot(cache, snapshot / "cache")
        validate_cache(descriptor, root=snapshot / "cache")
        cargo_home = snapshot / "cargo-home"
        cargo_home.mkdir()
        seed = snapshot / "cache/cargo-home-seed/config.toml"
        copy_no_replace(seed, cargo_home / "config.toml")
        (cargo_home / "config.toml").chmod(0o444)
        target = snapshot / "target"
        target.mkdir()
        container_name = fresh_container_name("rust2vir")
        argv = hermetic_docker_argv(vector, snapshot, arguments, container_name)
        limits = vector["launcher"]["process_limits"]
        result = run_bounded(
            argv,
            stdout_limit=limits["stdout_bytes"],
            stderr_limit=limits["stderr_bytes"],
            target_root=target,
            target_file_limit=limits["output_files"],
            target_byte_limit=limits["target_bytes"],
            docker_cleanup=(docker_path(), container_name),
        )
        validate_post_run_cargo_home(cargo_home, vector)
        checked_directory_usage(
            target,
            file_limit=limits["output_files"],
            byte_limit=limits["target_bytes"],
        )
        if current_frontend_inventory() != source_inventory:
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        validate_cache(descriptor, root=cache)
        if retained_target is not None and result.returncode == 0:
            if retained_target.exists() or retained_target.is_symlink():
                raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
            capture_candidate_outputs(target, retained_target)
        if retained_cache is not None and result.returncode == 0:
            if retained_cache.exists() or retained_cache.is_symlink():
                raise RustBuildFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
            copy_tree_snapshot(snapshot / "cache", retained_cache)
            validate_cache(descriptor, root=retained_cache)
        return result


def launch(arguments: list[str]) -> int:
    result = run_hermetic(arguments)
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)
    return result.returncode


def bundle_inventory(scope: dict[str, str], files: list[dict[str, object]]) -> dict[str, object]:
    return {"schema": "mpk.release.bundle_inventory.v0", "scope": scope, "files": files}


def inventory_file(path: str, source: Path, executable: bool) -> dict[str, object]:
    metadata = source.lstat()
    if source.is_symlink() or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    return {
        "path": path,
        "executable": executable,
        "size_bytes": metadata.st_size,
        "sha256": raw_hash(source),
    }


def content_hash(inventory: dict[str, object]) -> str:
    return typed_hash(CONTENT_DOMAIN, inventory)


def runtime_library_closure(executable: Path, toolchain: Path, runtime: Path) -> list[str]:
    queue = [(executable, True)]
    inspected: set[str] = set()
    external: set[str] = set()
    while queue:
        current, requires_interpreter = queue.pop(0)
        digest = raw_hash(current)
        if digest in inspected:
            continue
        inspected.add(digest)
        allowed_runpath = "$ORIGIN/../lib" if current.is_relative_to(toolchain) else None
        interpreter, needed = elf_dynamic(current, allowed_runpath=allowed_runpath)
        expected_interpreter = "/lib64/ld-linux-x86-64.so.2"
        if (requires_interpreter and interpreter != expected_interpreter) or (
            not requires_interpreter and interpreter not in (None, expected_interpreter)
        ):
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        for soname in needed:
            internal = internal_library(toolchain, soname)
            if internal is not None:
                queue.append((internal, False))
                continue
            library = runtime / "lib/x86_64-linux-gnu" / soname
            if library.is_symlink() or not library.is_file():
                raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
            external.add(soname)
            queue.append((library, False))
    return sorted(external, key=lambda item: item.encode("utf-8"))


def dynamic_runtime(executable: Path, toolchain: Path, runtime: Path) -> dict[str, object]:
    libraries = [
        {
            "soname": soname,
            "component_path": f"lib/x86_64-linux-gnu/{soname}",
            "sha256": raw_hash(runtime / "lib/x86_64-linux-gnu" / soname),
        }
        for soname in runtime_library_closure(executable, toolchain, runtime)
    ]
    if not libraries:
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    return {
        "kind": "dynamic",
        "interpreter_mount": "/lib64/ld-linux-x86-64.so.2",
        "libraries": libraries,
    }


def select_execution_toolchain_files(cache: Path) -> dict[str, list[tuple[str, Path, bool]]]:
    toolchain = cache / "toolchain"
    groups: dict[str, list[tuple[str, Path, bool]]] = {
        "rust-compiler-runtime": [],
        "rust-target-i686": [],
        "rust-target-x86_64": [],
        "native-runtime": [],
    }
    for relative in ("bin/cargo", "bin/rustc"):
        path = toolchain / relative
        if not path.is_file() or path.is_symlink():
            raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    for path in sorted((toolchain / "lib").rglob("*")):
        if not path.is_file() or path.is_symlink():
            continue
        relative = path.relative_to(toolchain).as_posix()
        executable = False
        if "/rustlib/i686-unknown-linux-gnu/" in f"/{relative}/":
            groups["rust-target-i686"].append((relative, path, executable))
        elif "/rustlib/x86_64-unknown-linux-gnu/" in f"/{relative}/":
            groups["rust-target-x86_64"].append((relative, path, executable))
        elif relative.endswith(".so") or ".so." in relative:
            groups["rust-compiler-runtime"].append((relative, path, executable))
    runtime = cache / "native-runtime"
    for path in sorted(runtime.rglob("*")):
        if path.is_file() and not path.is_symlink():
            relative = path.relative_to(cache).as_posix()
            groups["native-runtime"].append(
                (relative, path, bool(path.stat().st_mode & 0o111))
            )
    if any(not values for values in groups.values()):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    return groups


def assemble_candidate(
    descriptor: dict[str, object], cache: Path, target: Path
) -> bytes:
    release = target / "x86_64-unknown-linux-gnu/release"
    main = release / "rust2vir"
    driver = release / "rust2vir-driver"
    if not main.is_file() or not driver.is_file():
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    toolchain_root = cache / "toolchain"
    runtime_root = cache / "native-runtime"
    frontend_files = [
        inventory_file("bin/rust2vir", main, True),
        inventory_file("bin/rust2vir-driver", driver, True),
    ]
    frontend_files.sort(key=lambda item: item["path"].encode("utf-8"))
    frontend_inventory = bundle_inventory(
        {"kind": "frontend_bundle", "bundle_id": RUST_FRONTEND_ID}, frontend_files
    )
    groups = select_execution_toolchain_files(cache)
    root_files: list[dict[str, object]] = []
    components: list[dict[str, object]] = []
    for name in sorted(groups, key=lambda item: item.encode("utf-8")):
        files = [inventory_file(relative, path, executable) for relative, path, executable in groups[name]]
        files.sort(key=lambda item: item["path"].encode("utf-8"))
        root_files.extend(files)
        component_inventory = bundle_inventory(
            {
                "kind": "component",
                "bundle_id": RUST_TOOLCHAIN_ID,
                "component_name": name,
            },
            files,
        )
        components.append(
            {
                "kind": "content",
                "name": name,
                "release": "nightly-2025-06-01",
                "inventory": component_inventory,
                "content_sha256": content_hash(component_inventory),
            }
        )
    for name in ("cargo", "rustc"):
        path = toolchain_root / f"bin/{name}"
        item = inventory_file(f"bin/{name}", path, True)
        root_files.append(item)
        components.append(
            {
                "kind": "executable",
                "name": name,
                "release": "1.89.0-nightly",
                "path": item["path"],
                "binary_sha256": item["sha256"],
                "runtime": dynamic_runtime(path, toolchain_root, runtime_root),
            }
        )
    components.sort(key=lambda item: item["name"].encode("utf-8"))
    root_files.sort(key=lambda item: item["path"].encode("utf-8"))
    if len({item["path"] for item in root_files}) != len(root_files):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    toolchain_inventory = bundle_inventory(
        {"kind": "toolchain_bundle", "bundle_id": RUST_TOOLCHAIN_ID}, root_files
    )
    component_by_name = {item["name"]: item for item in components if item["kind"] == "content"}
    frontend_runtime_main = dynamic_runtime(main, toolchain_root, runtime_root)
    frontend_runtime_driver = dynamic_runtime(driver, toolchain_root, runtime_root)
    host = {
        "id": RUST_HOST_ID,
        "os": "linux",
        "architecture": "x86_64",
        "abi": "gnu",
        "minimum_kernel_abi": "5.10.0",
        "probe_profile_id": "mpk.release.probe.linux_namespaces.v0",
        "required_primitives": [
            "filesystem.atomic_no_replace",
            "filesystem.immutable_handle",
            "filesystem.no_follow_open",
            "isolation.mount_namespace",
            "isolation.network_namespace",
            "isolation.user_namespace",
            "mount.no_exec",
            "mount.read_only",
            "process.closed_environment",
            "process.no_new_privileges",
        ],
    }
    layout = {
        "id": RUST_RUNTIME_ID,
        "execution_host_profile_id": RUST_HOST_ID,
        "runtime_root": "/mpk/native-runtime",
        "interpreter_mounts": [
            {
                "component_path": "lib64/ld-linux-x86-64.so.2",
                "sandbox_path": "/lib64/ld-linux-x86-64.so.2",
            }
        ],
        "library_mounts": [
            {
                "component_path": "lib/x86_64-linux-gnu",
                "sandbox_path": "/lib/x86_64-linux-gnu",
            }
        ],
        "loader_search_paths": ["/lib/x86_64-linux-gnu"],
        "forbidden_host_roots": ["/lib", "/lib64", "/usr/lib"],
    }
    frontend = {
        "schema": "mpk.release.frontend_bundle.v0",
        "bundle_id": RUST_FRONTEND_ID,
        "source_language": "rust",
        "name": "rust2vir",
        "version": "0.1.0",
        "limit_profile_id": "mpk.vir.limits.v0",
        "environment_profile_id": "mpk.rust.frontend_environment.v0",
        "argument_profile_id": "mpk.rust.frontend_arguments.v0",
        "main": {
            "name": "rust2vir",
            "version": "0.1.0",
            "path": "bin/rust2vir",
            "binary_sha256": raw_hash(main),
            "runtime": frontend_runtime_main,
        },
        "subordinate_binaries": [
            {
                "name": "rust2vir-driver",
                "version": "0.1.0",
                "path": "bin/rust2vir-driver",
                "binary_sha256": raw_hash(driver),
                "runtime": frontend_runtime_driver,
            }
        ],
        "inventory": frontend_inventory,
        "bundle_sha256": content_hash(frontend_inventory),
    }
    target_libraries = []
    for target_id, pointer_width, component_name in (
        ("i686-unknown-linux-gnu", 32, "rust-target-i686"),
        ("x86_64-unknown-linux-gnu", 64, "rust-target-x86_64"),
    ):
        target_libraries.append(
            {
                "target_id": target_id,
                "pointer_width": pointer_width,
                "component_name": component_name,
                "content_sha256": component_by_name[component_name]["content_sha256"],
            }
        )
    toolchain = {
        "schema": "mpk.release.toolchain_bundle.v0",
        "bundle_id": RUST_TOOLCHAIN_ID,
        "source_language": "rust",
        "compiler": {
            "kind": "rust",
            "release": "1.89.0-nightly",
            "rustc_commit": EXPECTED_RUSTC_COMMIT,
        },
        "execution_host_profile_id": RUST_HOST_ID,
        "native_runtime": {
            "kind": "component",
            "component_name": "native-runtime",
            "component_root": "native-runtime",
            "layout_profile_id": RUST_RUNTIME_ID,
        },
        "components": components,
        "target_libraries": target_libraries,
        "inventory": toolchain_inventory,
        "distribution_sha256": content_hash(toolchain_inventory),
    }
    candidate = {
        "schema": "mpk.release.bundle_candidate.v0",
        "source_language": "rust",
        "execution_host_profiles": [host],
        "native_runtime_layout_profiles": [layout],
        "frontend_bundles": [frontend],
        "toolchain_bundles": [toolchain],
        "tuples": [
            {
                "source_language": "rust",
                "semantic_profile": "mpk.rust.checked.v0",
                "target_id": "i686-unknown-linux-gnu",
                "pointer_width": 32,
                "limit_profile_id": "mpk.vir.limits.v0",
                "frontend_bundle_id": RUST_FRONTEND_ID,
                "toolchain_bundle_id": RUST_TOOLCHAIN_ID,
            },
            {
                "source_language": "rust",
                "semantic_profile": "mpk.rust.checked.v0",
                "target_id": "x86_64-unknown-linux-gnu",
                "pointer_width": 64,
                "limit_profile_id": "mpk.vir.limits.v0",
                "frontend_bundle_id": RUST_FRONTEND_ID,
                "toolchain_bundle_id": RUST_TOOLCHAIN_ID,
            },
        ],
    }
    if descriptor["rust_distribution"]["rustc_commit"] != EXPECTED_RUSTC_COMMIT:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    validate_candidate_model(candidate)
    return canonical(candidate) + b"\n"


def validate_candidate_model(candidate: object) -> None:
    if not isinstance(candidate, dict) or set(candidate) != {
        "schema",
        "source_language",
        "execution_host_profiles",
        "native_runtime_layout_profiles",
        "frontend_bundles",
        "toolchain_bundles",
        "tuples",
    }:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    if candidate["schema"] != "mpk.release.bundle_candidate.v0" or candidate[
        "source_language"
    ] != "rust":
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    for field in (
        "execution_host_profiles",
        "native_runtime_layout_profiles",
        "frontend_bundles",
        "toolchain_bundles",
    ):
        if not isinstance(candidate[field], list) or len(candidate[field]) != 1:
            raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    frontend = candidate["frontend_bundles"][0]
    subordinate = frontend.get("subordinate_binaries") if isinstance(frontend, dict) else None
    if (
        not isinstance(frontend, dict)
        or frontend.get("schema") != "mpk.release.frontend_bundle.v0"
        or frontend.get("bundle_id") != RUST_FRONTEND_ID
        or frontend.get("source_language") != "rust"
        or frontend.get("main", {}).get("name") != "rust2vir"
        or not isinstance(subordinate, list)
        or len(subordinate) != 1
        or subordinate[0].get("name") != "rust2vir-driver"
        or subordinate[0].get("path") != "bin/rust2vir-driver"
    ):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    frontend_paths = [
        item.get("path") for item in frontend.get("inventory", {}).get("files", [])
    ]
    if frontend_paths != ["bin/rust2vir", "bin/rust2vir-driver"]:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    toolchain = candidate["toolchain_bundles"][0]
    if (
        not isinstance(toolchain, dict)
        or toolchain.get("schema") != "mpk.release.toolchain_bundle.v0"
        or toolchain.get("bundle_id") != RUST_TOOLCHAIN_ID
        or toolchain.get("compiler", {}).get("rustc_commit") != EXPECTED_RUSTC_COMMIT
    ):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    tuples = candidate["tuples"]
    if not isinstance(tuples, list) or [item.get("target_id") for item in tuples] != [
        "i686-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ]:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    if any(
        item.get("source_language") != "rust"
        or item.get("frontend_bundle_id") != RUST_FRONTEND_ID
        or item.get("toolchain_bundle_id") != RUST_TOOLCHAIN_ID
        for item in tuples
    ):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")


def read_active_registry(*, require_go_only: bool = True) -> bytes:
    path = repository_root() / "release/bundles/bundle-registry.json"
    data = path.read_bytes()
    value = strict_json(data[:-1]) if data.endswith(b"\n") else None
    if (
        not isinstance(value, dict)
        or canonical(value) + b"\n" != data
        or not value.get("tuples")
        or any(item.get("source_language") not in ("go", "rust") for item in value["tuples"])
        or (
            require_go_only
            and any(item.get("source_language") != "go" for item in value["tuples"])
        )
    ):
        raise RustBuildFailure("BUNDLE_REGISTERED_STATE")
    return data


def materialize_candidate_tree(
    candidate_data: bytes, cache: Path, target: Path, destination: Path
) -> None:
    candidate = strict_json(candidate_data[:-1])
    validate_candidate_model(candidate)
    release = target / "x86_64-unknown-linux-gnu/release"
    sources = {
        "frontend": {
            "bin/rust2vir": release / "rust2vir",
            "bin/rust2vir-driver": release / "rust2vir-driver",
        },
        "toolchain": {},
    }
    for item in candidate["toolchain_bundles"][0]["inventory"]["files"]:
        relative = item["path"]
        if relative.startswith("native-runtime/"):
            source = cache / relative
        else:
            source = cache / "toolchain" / relative
        sources["toolchain"][relative] = source
    for kind in ("frontend", "toolchain"):
        root = destination / kind
        for relative, source in sources[kind].items():
            expected = next(
                item
                for item in candidate[f"{kind}_bundles"][0]["inventory"]["files"]
                if item["path"] == relative
            )
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, path)
            path.chmod(0o555 if expected["executable"] else 0o444)
        for path in sorted(root.rglob("*"), reverse=True):
            if path.is_dir() and not path.is_symlink():
                path.chmod(0o555)
        root.chmod(0o555)


def build_candidate_bytes(
    *, output: Path | None = None, require_go_only: bool = True
) -> bytes:
    descriptor, cache = check_build_inputs()
    registry_before = read_active_registry(require_go_only=require_go_only)
    with tempfile.TemporaryDirectory(
        prefix=".mpk-rust-candidate-", dir=cache_parent()
    ) as temporary:
        work = Path(temporary)
        first = work / "first"
        second = work / "second"
        first_cache = work / "first-cache"
        arguments = [
            "cargo",
            "build",
            "--locked",
            "--offline",
            "--release",
            "--bins",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--jobs",
            "1",
        ]
        first_result = run_hermetic(
            arguments, retained_target=first, retained_cache=first_cache
        )
        second_result = run_hermetic(arguments, retained_target=second)
        if first_result.returncode != 0 or second_result.returncode != 0:
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        normalize_portable_cpp_runtime(first_cache, first)
        normalize_portable_cpp_runtime(None, second)
        for relative in (
            "x86_64-unknown-linux-gnu/release/rust2vir",
            "x86_64-unknown-linux-gnu/release/rust2vir-driver",
        ):
            if not same_file_bytes(first / relative, second / relative):
                raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        validate_cache(descriptor, root=cache)
        result = assemble_candidate(descriptor, first_cache, first)
        if output is not None:
            materialize_candidate_tree(result, first_cache, first, output)
    if read_active_registry(require_go_only=require_go_only) != registry_before:
        raise RustBuildFailure("BUNDLE_REGISTERED_STATE")
    return result


def normalize_portable_cpp_runtime(cache: Path | None, target: Path) -> None:
    old = b"libstdc++.so.6"
    new = b"libstdcxx.so.6"
    release = target / "x86_64-unknown-linux-gnu/release"
    candidates = [release / "rust2vir", release / "rust2vir-driver"]
    if cache is not None:
        old_path = cache / "native-runtime/lib/x86_64-linux-gnu/libstdc++.so.6"
        new_path = cache / "native-runtime/lib/x86_64-linux-gnu/libstdcxx.so.6"
        if old_path.is_symlink() or not old_path.is_file() or new_path.exists() or new_path.is_symlink():
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        old_path.rename(new_path)
        toolchain = cache / "toolchain"
        candidates.extend([toolchain / "bin/cargo", toolchain / "bin/rustc"])
        candidates.extend(
            path
            for path in (toolchain / "lib").rglob("*")
            if path.is_file()
            and not path.is_symlink()
            and (path.name.endswith(".so") or ".so." in path.name)
        )
        candidates.extend(
            path
            for path in (cache / "native-runtime").rglob("*")
            if path.is_file() and not path.is_symlink()
        )
    for path in sorted(candidates):
        if path.is_symlink() or not path.is_file():
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        data = path.read_bytes()
        if old not in data:
            continue
        mode = stat.S_IMODE(path.lstat().st_mode)
        path.chmod(0o600)
        path.write_bytes(data.replace(old, new))
        path.chmod(mode)


def publish_candidate(
    data: bytes, source_inventory: list[tuple[str, int, str, bool]]
) -> None:
    import release_bundles

    root = repository_root()
    active = root / "release/bundles"
    registry = read_active_registry()
    readme_path = active / "README.md"
    readme_metadata = readme_path.lstat()
    if readme_path.is_symlink() or not stat.S_ISREG(readme_metadata.st_mode):
        raise RustBuildFailure("BUNDLE_REGISTERED_STATE")
    readme = readme_path.read_bytes()
    if not data.endswith(b"\n") or data.endswith(b"\n\n"):
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    candidate = strict_json(data[:-1])
    if canonical(candidate) + b"\n" != data:
        raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    validate_candidate_model(candidate)
    staging = Path(tempfile.mkdtemp(prefix=".bundles-stage-", dir=active.parent))
    try:
        (staging / "README.md").write_bytes(readme)
        (staging / "bundle-registry.json").write_bytes(registry)
        directory = staging / "candidates/rust"
        directory.mkdir(parents=True)
        (directory / "candidate.json").write_bytes(data)
        for path in (
            staging / "README.md",
            staging / "bundle-registry.json",
            directory / "candidate.json",
        ):
            path.chmod(0o644)
            with path.open("rb") as stream:
                os.fsync(stream.fileno())
        for path in (directory, directory.parent, staging):
            fsync_directory(path)
        if read_active_registry() != registry or readme_path.read_bytes() != readme:
            raise RustBuildFailure("BUNDLE_REGISTERED_STATE")
        if current_frontend_inventory() != source_inventory:
            raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        release_bundles.exchange_directories(staging, active)
        fsync_directory(active.parent)
        if read_active_registry() != registry:
            raise RustBuildFailure("BUNDLE_REGISTERED_STATE")
        shutil.rmtree(staging)
    except RustBuildFailure:
        raise
    except release_bundles.BundleFailure as error:
        raise RustBuildFailure(error.code, error.exit_code) from error
    except Exception as error:
        raise RustBuildFailure("BUNDLE_ASSEMBLER_IO", 74) from error
    finally:
        if staging.exists():
            shutil.rmtree(staging, ignore_errors=True)


def update_candidate(*, check: bool) -> None:
    read_active_registry()
    source_inventory = current_frontend_inventory()
    expected = build_candidate_bytes()
    if current_frontend_inventory() != source_inventory:
        raise RustBuildFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    if check:
        path = candidate_path()
        if path.is_symlink() or not path.is_file() or path.read_bytes() != expected:
            raise RustBuildFailure("BUNDLE_CANDIDATE_STATE")
    else:
        publish_candidate(expected, source_inventory)


def self_test_expect(code: str, operation: object) -> None:
    try:
        operation()
    except RustBuildFailure as error:
        if error.code != code:
            raise RustBuildFailure() from AssertionError(
                f"expected {code}, observed {error.code}"
            )
        return
    raise RustBuildFailure() from AssertionError(f"expected {code}")


def self_test_rehash(descriptor: dict[str, object]) -> None:
    descriptor["build_inputs_sha256"] = build_inputs_hash(descriptor)


def self_test_descriptor_mutation(
    vector: dict[str, object], code: str, mutation: object, *, rehash: bool = True
) -> None:
    descriptor = copy.deepcopy(vector["valid_descriptor"])
    mutation(descriptor)
    if rehash:
        self_test_rehash(descriptor)
    self_test_expect(code, lambda: validate_descriptor_model(descriptor, vector))


def synthetic_cache_bytes(
    vector: dict[str, object], descriptor: dict[str, object], path: str
) -> bytes:
    overrides = {
        item["path"]: item["raw_template_id"]
        for item in vector["valid_descriptor_cache_content"]["raw_template_overrides"]
    }
    if path in overrides:
        return raw_templates(vector)[overrides[path]]
    if path.endswith("/.cargo-checksum.json"):
        packages = {
            f"vendor/{package['name']}-{package['version']}/.cargo-checksum.json": package[
                "checksum"
            ]
            for graph in descriptor["graphs"]
            for package in graph["packages"]
            if package["checksum"] is not None
        }
        checksum = packages.get(path)
        if checksum is None:
            raise RustBuildFailure()
        return canonical({"files": {}, "package": checksum})
    return path.encode("utf-8")


def materialize_synthetic_cache(
    vector: dict[str, object], descriptor: dict[str, object], root: Path
) -> None:
    root.mkdir()
    for component in descriptor["components"]:
        for item in component["files"]:
            path = root / item["path"]
            path.parent.mkdir(parents=True, exist_ok=True)
            data = synthetic_cache_bytes(vector, descriptor, item["path"])
            if len(data) != item["size_bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
                raise RustBuildFailure()
            path.write_bytes(data)
    normalize_cache_modes(root, descriptor)


def make_cache_writable(root: Path) -> None:
    root.chmod(0o700)
    for path in root.rglob("*"):
        if path.is_dir():
            path.chmod(0o700)
        elif path.is_file() and not path.is_symlink():
            path.chmod(0o600)


def run_self_test() -> None:
    vector = load_vector()
    validate_project_templates(vector)
    descriptor = copy.deepcopy(vector["valid_descriptor"])
    validate_descriptor_model(descriptor, vector)
    transport = descriptor_transport(descriptor)
    expected = vector["valid_descriptor_expect"]
    if (
        len(transport) != expected["transport_utf8_length"]
        or descriptor["build_inputs_sha256"] != expected["build_inputs_sha256"]
        or validate_descriptor_transport(transport, vector) != descriptor
    ):
        raise RustBuildFailure()

    invalid_ids = {
        "descriptor.wrong_self_hash",
        "descriptor.wrong_cache_key",
        "descriptor.self_entry",
        "descriptor.missing_file",
        "descriptor.extra_file",
        "descriptor.source_included",
        "vendor.checksum_mismatch",
        "vendor.lock_checksum_missing",
        "vendor.alternate_registry",
        "seed.source_replacement",
        "seed.credential",
        "provenance.missing",
        "native.projection_escape",
        "native.clang_config",
        "native.elf_unresolved",
        "native.soname_collision",
        "native.post_build_dependency",
        "notice.missing",
        "license.legacy_unmapped",
        "shape.unknown",
        "transport.no_lf",
        "transport.second_lf",
        "graph.feature_change",
        "graph.edge_change",
        "fuzz.unknown_child",
        "fuzz.argv_change",
        "fuzz.linker_backend_missing",
        "fuzz.environment_change",
        "fuzz.dag_edge_change",
        "fuzz.scratch_escape",
        "fuzz.run_rebuild",
        "fuzz.libfuzzer_source_order",
        "fuzz.libfuzzer_archive_missing",
        "path.parent",
        "path.machine_local",
        "file.size_mismatch",
        "publication.cache_unequal",
        "publication.repair_in_check",
    }
    publication_ids = {
        "update.valid",
        "update.cache_equal",
        "update.cache_unequal",
        "provision.valid",
        "provision.equal",
        "check.missing",
    }
    limit_ids = {
        "descriptor_transport",
        "regular_files",
        "package_records",
        "inventory_path",
        "regular_file",
        "aggregate_cache",
        "aggregate_checked_overflow",
    }
    groups = (
        ("invalid_cases", invalid_ids),
        ("publication_cases", publication_ids),
        ("limit_cases", limit_ids),
    )
    all_case_ids: set[str] = set()
    for group, expected_ids in groups:
        cases = vector[group]
        observed_ids = {case["id"] for case in cases}
        if len(observed_ids) != len(cases) or observed_ids != expected_ids:
            raise RustBuildFailure()
        if all_case_ids & observed_ids:
            raise RustBuildFailure()
        all_case_ids |= observed_ids
    expected_invalid_codes = {
        "descriptor.wrong_self_hash": "RUST_BUILD_INPUTS_HASH",
        "descriptor.wrong_cache_key": "RUST_BUILD_INPUTS_CACHE_KEY",
        "descriptor.self_entry": "RUST_BUILD_INPUTS_INVENTORY",
        "descriptor.missing_file": "RUST_BUILD_INPUTS_INVENTORY",
        "descriptor.extra_file": "RUST_BUILD_INPUTS_INVENTORY",
        "descriptor.source_included": "RUST_BUILD_INPUTS_SOURCE_EXCLUSION",
        "vendor.checksum_mismatch": "RUST_BUILD_INPUTS_VENDOR",
        "vendor.lock_checksum_missing": "RUST_BUILD_INPUTS_GRAPH",
        "vendor.alternate_registry": "RUST_BUILD_INPUTS_GRAPH",
        "seed.source_replacement": "RUST_BUILD_INPUTS_CARGO_HOME",
        "seed.credential": "RUST_BUILD_INPUTS_CARGO_HOME",
        "provenance.missing": "RUST_BUILD_INPUTS_PROVENANCE",
        "native.projection_escape": "RUST_BUILD_INPUTS_PROVENANCE",
        "native.clang_config": "RUST_BUILD_INPUTS_PROVENANCE",
        "native.elf_unresolved": "RUST_BUILD_INPUTS_INVENTORY",
        "native.soname_collision": "RUST_BUILD_INPUTS_INVENTORY",
        "native.post_build_dependency": "RUST_BUILD_INPUTS_INVENTORY",
        "notice.missing": "RUST_BUILD_INPUTS_LICENSE",
        "license.legacy_unmapped": "RUST_BUILD_INPUTS_LICENSE",
        "shape.unknown": "RUST_BUILD_INPUTS_SHAPE",
        "transport.no_lf": "RUST_BUILD_INPUTS_TRANSPORT",
        "transport.second_lf": "RUST_BUILD_INPUTS_TRANSPORT",
        "graph.feature_change": "RUST_BUILD_INPUTS_GRAPH",
        "graph.edge_change": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.unknown_child": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.argv_change": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.linker_backend_missing": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.environment_change": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.dag_edge_change": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.scratch_escape": "RUST_BUILD_INPUTS_PATH",
        "fuzz.run_rebuild": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.libfuzzer_source_order": "RUST_BUILD_INPUTS_GRAPH",
        "fuzz.libfuzzer_archive_missing": "RUST_BUILD_INPUTS_INVENTORY",
        "path.parent": "RUST_BUILD_INPUTS_PATH",
        "path.machine_local": "RUST_BUILD_INPUTS_PATH",
        "file.size_mismatch": "RUST_BUILD_INPUTS_FILE",
        "publication.cache_unequal": "RUST_BUILD_INPUTS_PUBLICATION",
        "publication.repair_in_check": "RUST_BUILD_INPUTS_PUBLICATION",
    }
    if {
        case["id"]: case["code"] for case in vector["invalid_cases"]
    } != expected_invalid_codes:
        raise RustBuildFailure()
    expected_publication = {
        "update.valid": "publish-cache-then-atomic-descriptor-commit",
        "update.cache_equal": "reuse-cache-then-atomic-descriptor-commit",
        "update.cache_unequal": "fail-without-repair",
        "provision.valid": "publish-fixed-key",
        "provision.equal": "reuse-after-full-validation",
        "check.missing": "fail-without-fetch-or-repair",
    }
    if {
        case["id"]: case["expect"] for case in vector["publication_cases"]
    } != expected_publication:
        raise RustBuildFailure()

    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_HASH",
        lambda value: value.__setitem__("build_inputs_sha256", "0" * 64),
        rehash=False,
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_SHAPE",
        lambda value: value.__setitem__("machine_path", "/tmp"),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_PROVENANCE",
        lambda value: value["components"][0].pop("provenance"),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_LICENSE",
        lambda value: next(
            component for component in value["components"] if component["notice_refs"]
        ).__setitem__("notice_refs", []),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_GRAPH",
        lambda value: value["graphs"][0]["packages"][0].__setitem__(
            "features", ["unexpected"]
        ),
    )

    def append_file(value: dict[str, object], path: str) -> None:
        component = value["components"][0]
        item = copy.deepcopy(component["files"][0])
        item["path"] = path
        component["files"].append(item)
        component["files"].sort(key=lambda record: record["path"].encode("utf-8"))

    first_component = descriptor["components"][0]["name"]
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_SOURCE_EXCLUSION",
        lambda value: append_file(
            value, f"{first_component}/rust-tools/rust2vir/src/lib.rs"
        ),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_INVENTORY",
        lambda value: append_file(
            value,
            f"{first_component}/release/build-inputs/rust/build-inputs.json",
        ),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_PATH",
        lambda value: value["components"][0]["files"][0].__setitem__(
            "path", f"{first_component}/../escape"
        ),
    )
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_PATH",
        lambda value: value["components"][0]["files"][0].__setitem__(
            "path", "/Users/name"
        ),
    )
    case_distinct_descriptor = copy.deepcopy(descriptor)
    append_file(case_distinct_descriptor, f"{first_component}/Config.toml")
    self_test_rehash(case_distinct_descriptor)
    validate_descriptor_model(case_distinct_descriptor, vector)
    self_test_descriptor_mutation(
        vector,
        "RUST_BUILD_INPUTS_FILE",
        lambda value: value["components"][0]["files"][0].__setitem__(
            "size_bytes", FILE_SIZE_LIMIT + 1
        ),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_TRANSPORT",
        lambda: validate_descriptor_transport(transport[:-1], vector),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_TRANSPORT",
        lambda: validate_descriptor_transport(transport + b"\n", vector),
    )

    limits = {case["id"]: case for case in vector["limit_cases"]}
    limit_units = {
        "descriptor_transport": "bytes_including_lf",
        "regular_files": "entries",
        "package_records": "records",
        "inventory_path": "bytes",
        "regular_file": "bytes",
        "aggregate_cache": "bytes",
    }
    for identifier, (maximum, code) in BUILD_INPUT_LIMITS.items():
        case = limits[identifier]
        if set(case) != {
            "id",
            "limit_id",
            "maximum",
            "unit",
            "below",
            "at",
            "above",
        } or (
            case["limit_id"] != identifier
            or case["maximum"] != maximum
            or case["unit"] != limit_units[identifier]
        ):
            raise RustBuildFailure()
        for point, count, outcome in (
            ("below", maximum - 1, "accept"),
            ("at", maximum, "accept"),
            ("above", maximum + 1, "reject"),
        ):
            boundary = case[point]
            expected_outcome = {"outcome": outcome}
            if outcome == "reject":
                expected_outcome["code"] = code
            if identifier == "inventory_path":
                component_bytes = {
                    "below": [204, 204, 204, 204, 203],
                    "at": [204, 204, 204, 204, 204],
                    "above": [204, 204, 204, 204, 205],
                }[point]
                expected_construction = {
                    "fixture": "portable_path_bytes",
                    "counter": identifier,
                    "component_bytes": component_bytes,
                }
            else:
                expected_construction = {
                    "fixture": "checked_counter",
                    "counter": identifier,
                    "count": count,
                }
            if (
                not isinstance(boundary, dict)
                or set(boundary) != {"construction", "expect"}
                or boundary["construction"] != expected_construction
                or boundary["expect"] != expected_outcome
            ):
                raise RustBuildFailure()
            if outcome == "accept":
                if dispatch_build_input_boundary(boundary["construction"]) != count:
                    raise RustBuildFailure()
            else:
                self_test_expect(
                    code,
                    lambda construction=boundary["construction"]: (
                        dispatch_build_input_boundary(construction)
                    ),
                )

    overflow = limits["aggregate_checked_overflow"]
    overflow_construction = {
        "fixture": "checked_counter_add",
        "counter": "aggregate_cache",
        "initial": "u64_max",
        "increment": 1,
    }
    if (
        set(overflow) != {"id", "limit_id", "construction", "expect"}
        or overflow["limit_id"] != "aggregate_cache"
        or overflow["construction"] != overflow_construction
        or overflow["expect"]
        != {
            "outcome": "reject-before-mount",
            "code": "RUST_BUILD_INPUTS_GRAPH",
        }
    ):
        raise RustBuildFailure()
    self_test_expect(
        "RUST_BUILD_INPUTS_GRAPH",
        lambda: dispatch_build_input_boundary(overflow_construction),
    )

    below_path = "/".join("a" * length for length in [204, 204, 204, 204, 203])
    at_path = "/".join("a" * 204 for _ in range(5))
    above_path = "/".join("a" * length for length in [204, 204, 204, 204, 205])
    if (
        len(below_path.encode("ascii")) != PATH_LIMIT - 1
        or len(at_path.encode("ascii")) != PATH_LIMIT
        or len(above_path.encode("ascii")) != PATH_LIMIT + 1
        or not portable_path(below_path)
        or not portable_path(at_path)
        or portable_path(above_path)
        or not portable_path("a" * 255)
        or portable_path("a" * 256)
        or portable_path("a+b")
        or not portable_path("a+b", build_input=True)
    ):
        raise RustBuildFailure()
    self_test_expect(
        "BUNDLE_BUILD_INPUTS_INVALID", lambda: strict_json(b"1" * 17)
    )

    prescan_fixture = (
        b'{"components":[{"files":[{"path":"toolchain/bin/rustc",'
        b'"size_bytes":7}]}],"graphs":[{"packages":[{}]}]}'
    )
    if prescan_descriptor(prescan_fixture) != (1, 1, 7):
        raise RustBuildFailure()
    self_test_expect(
        "RUST_BUILD_INPUTS_PATH",
        lambda: prescan_descriptor(
            b'{"components":[{"files":[{"path":"'
            + b"a" * (PATH_LIMIT + 1)
            + b'","size_bytes":0}]}]}'
        ),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_FILE",
        lambda: prescan_descriptor(
            b'{"components":[{"files":[{"path":"x","size_bytes":'
            + str(FILE_SIZE_LIMIT + 1).encode("ascii")
            + b"}]}]}"
        ),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_GRAPH",
        lambda: prescan_descriptor(
            b'{"components":[{"files":['
            + b",".join(
                b'{"path":"x","size_bytes":'
                + str(FILE_SIZE_LIMIT).encode("ascii")
                + b"}"
                for _index in range(9)
            )
            + b"]}]}"
        ),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_GRAPH",
        lambda: prescan_descriptor(
            b'{"graphs":[{"packages":['
            + b",".join(b"{}" for _index in range(PACKAGE_COUNT_LIMIT + 1))
            + b"]}]}"
        ),
    )
    self_test_expect(
        "RUST_BUILD_INPUTS_TRANSPORT",
        lambda: prescan_descriptor(
            b"[" * (DESCRIPTOR_JSON_DEPTH_LIMIT + 1)
            + b"0"
            + b"]" * (DESCRIPTOR_JSON_DEPTH_LIMIT + 1)
        ),
    )

    process_limits = vector["launcher"]["process_limits"]
    if process_limits != FROZEN_PROCESS_LIMITS:
        raise RustBuildFailure()
    resource_arguments, virtual_memory_kib = launcher_resource_controls(process_limits)
    if resource_arguments != [
        "--pids-limit=256",
        "--ulimit=nofile=1024:1024",
        "--memory=8589934592",
        "--memory-swap=8589934592",
        "--tmpfs=/mpk/home:rw,nosuid,nodev,noexec,mode=700",
        "--tmpfs=/mpk/tmp:rw,nosuid,nodev,noexec,mode=700,size=4294967296",
    ] or virtual_memory_kib != 16_777_216:
        raise RustBuildFailure()
    for maximum in FROZEN_PROCESS_LIMITS.values():
        if (
            checked_boundary(
                maximum - 1, maximum, "BUNDLE_REPRODUCIBILITY_MISMATCH"
            )
            != maximum - 1
            or checked_boundary(
                maximum, maximum, "BUNDLE_REPRODUCIBILITY_MISMATCH"
            )
            != maximum
        ):
            raise RustBuildFailure()
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda maximum=maximum: checked_boundary(
                maximum + 1, maximum, "BUNDLE_REPRODUCIBILITY_MISMATCH"
            ),
        )

    for stream_name in ("stdout", "stderr"):
        for count, accepted in ((15, True), (16, True), (17, False)):
            command = [
                "/usr/bin/python3",
                "-c",
                f"import sys; sys.{stream_name}.buffer.write(b'x' * {count})",
            ]
            operation = lambda command=command: run_bounded(
                command, stdout_limit=16, stderr_limit=16
            )
            if accepted:
                output = operation()
                observed = output.stdout if stream_name == "stdout" else output.stderr
                other = output.stderr if stream_name == "stdout" else output.stdout
                if output.returncode != 0 or observed != b"x" * count or other:
                    raise RustBuildFailure()
            else:
                self_test_expect("BUNDLE_REPRODUCIBILITY_MISMATCH", operation)

    with tempfile.TemporaryDirectory(
        prefix="mpk-rust-process-limit-self-test-"
    ) as process_temporary:
        process_root = Path(process_temporary)

        cleanup_log = process_root / "docker-cleanup.log"
        cleanup_program = process_root / "fake-docker"
        cleanup_program.write_text(
            "#!/usr/bin/python3\n"
            "import pathlib, sys\n"
            f"pathlib.Path({str(cleanup_log)!r}).write_text('|'.join(sys.argv[1:]))\n",
            encoding="utf-8",
        )
        cleanup_program.chmod(0o755)
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: run_bounded(
                [
                    "/usr/bin/python3",
                    "-c",
                    "import sys; sys.stdout.buffer.write(b'x' * 17)",
                ],
                stdout_limit=16,
                stderr_limit=0,
                docker_cleanup=(str(cleanup_program), "mpk-cleanup-self-test"),
            ),
        )
        if read_bounded_regular_file(
            cleanup_log,
            maximum=256,
            code="BUNDLE_REPRODUCIBILITY_MISMATCH",
        ) != b"rm|--force|mpk-cleanup-self-test":
            raise RustBuildFailure()

        bounded_file = process_root / "bounded-file"
        bounded_file.write_bytes(b"1234")
        if read_bounded_regular_file(
            bounded_file,
            maximum=4,
            code="RUST_BUILD_INPUTS_TRANSPORT",
        ) != b"1234":
            raise RustBuildFailure()
        bounded_file.write_bytes(b"12345")
        self_test_expect(
            "RUST_BUILD_INPUTS_TRANSPORT",
            lambda: read_bounded_regular_file(
                bounded_file,
                maximum=4,
                code="RUST_BUILD_INPUTS_TRANSPORT",
            ),
        )
        bounded_symlink = process_root / "bounded-symlink"
        bounded_symlink.symlink_to(bounded_file.name)
        self_test_expect(
            "RUST_BUILD_INPUTS_TRANSPORT",
            lambda: read_bounded_regular_file(
                bounded_symlink,
                maximum=4,
                code="RUST_BUILD_INPUTS_TRANSPORT",
            ),
        )
        bounded_fifo = process_root / "bounded-fifo"
        os.mkfifo(bounded_fifo)
        self_test_expect(
            "RUST_BUILD_INPUTS_TRANSPORT",
            lambda: read_bounded_regular_file(
                bounded_fifo,
                maximum=4,
                code="RUST_BUILD_INPUTS_TRANSPORT",
            ),
        )

        def usage_fixture(name: str, sizes: list[int]) -> Path:
            root = process_root / name
            root.mkdir()
            for index, size in enumerate(sizes):
                (root / f"file-{index}").write_bytes(b"x" * size)
            return root

        if checked_directory_usage(
            usage_fixture("bytes-below", [3]), file_limit=1, byte_limit=4
        ) != (1, 3) or checked_directory_usage(
            usage_fixture("bytes-at", [4]), file_limit=1, byte_limit=4
        ) != (1, 4):
            raise RustBuildFailure()
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: checked_directory_usage(
                usage_fixture("bytes-above", [5]), file_limit=1, byte_limit=4
            ),
        )
        if checked_directory_usage(
            usage_fixture("files-below", [0]), file_limit=2, byte_limit=0
        ) != (1, 0) or checked_directory_usage(
            usage_fixture("files-at", [0, 0]), file_limit=2, byte_limit=0
        ) != (2, 0):
            raise RustBuildFailure()
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: checked_directory_usage(
                usage_fixture("files-above", [0, 0, 0]),
                file_limit=2,
                byte_limit=0,
            ),
        )

        directories_at = process_root / "directories-at"
        directories_at.mkdir()
        (directories_at / "child").mkdir()
        if checked_directory_usage(
            directories_at, file_limit=0, byte_limit=0, directory_limit=2
        ) != (0, 0):
            raise RustBuildFailure()
        directories_above = process_root / "directories-above"
        directories_above.mkdir()
        (directories_above / "first").mkdir()
        (directories_above / "second").mkdir()
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: checked_directory_usage(
                directories_above,
                file_limit=0,
                byte_limit=0,
                directory_limit=2,
            ),
        )

        special_root = process_root / "special-node"
        special_root.mkdir()
        os.mkfifo(special_root / "fifo")
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: checked_directory_usage(
                special_root, file_limit=1, byte_limit=0
            ),
        )
        (special_root / "fifo").unlink()
        (special_root / "symlink").symlink_to(process_root)
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: checked_directory_usage(
                special_root, file_limit=1, byte_limit=0
            ),
        )

        monitored = process_root / "monitored"
        monitored.mkdir()
        finished = process_root / "finished"
        self_test_expect(
            "BUNDLE_REPRODUCIBILITY_MISMATCH",
            lambda: run_bounded(
                [
                    "/usr/bin/python3",
                    "-c",
                    (
                        "import os,pathlib,sys,time; os.close(1); os.close(2); "
                        "pathlib.Path(sys.argv[1]).joinpath('output').write_bytes(b'x'*5); "
                        "time.sleep(1); pathlib.Path(sys.argv[2]).write_text('finished')"
                    ),
                    str(monitored),
                    str(finished),
                ],
                stdout_limit=0,
                stderr_limit=0,
                target_root=monitored,
                target_file_limit=1,
                target_byte_limit=4,
            ),
        )
        if finished.exists():
            raise RustBuildFailure()

    launcher_modes = vector["launcher"]["modes"]
    if not launcher_modes or any(
        not accepted_launcher_arguments(vector, list(arguments))
        for arguments in launcher_modes
    ):
        raise RustBuildFailure()
    rejected_arguments = (
        [],
        ["cargo"],
        [*launcher_modes[0], "--extra"],
        ["cargo", "install"],
        ["rustup", "run", "nightly", "cargo", "test"],
    )
    if any(accepted_launcher_arguments(vector, arguments) for arguments in rejected_arguments):
        raise RustBuildFailure()
    integration = ["cargo", "test", "--locked", "--test", "build_inputs_conformance"]
    if not accepted_launcher_arguments(vector, integration):
        raise RustBuildFailure()

    with tempfile.TemporaryDirectory(prefix="mpk-rust-build-input-self-test-") as temporary:
        work = Path(temporary)
        valid = work / "valid"
        materialize_synthetic_cache(vector, descriptor, valid)
        validate_cache(descriptor, root=valid)

        def invalid_cache(name: str, mutation: object, code: str) -> None:
            root = work / name
            materialize_synthetic_cache(vector, descriptor, root)
            make_cache_writable(root)
            mutation(root)
            normalize_cache_modes(root, descriptor)
            self_test_expect(code, lambda: validate_cache(descriptor, root=root))

        first_path = descriptor["components"][0]["files"][0]["path"]
        invalid_cache(
            "missing",
            lambda root: (root / first_path).unlink(),
            "RUST_BUILD_INPUTS_INVENTORY",
        )
        invalid_cache(
            "extra",
            lambda root: (root / "toolchain/extra").write_bytes(b"extra"),
            "RUST_BUILD_INPUTS_INVENTORY",
        )
        invalid_cache(
            "seed",
            lambda root: (root / "cargo-home-seed/config.toml").write_bytes(b"changed"),
            "RUST_BUILD_INPUTS_CARGO_HOME",
        )
        vendor_checksum = next(
            item["path"]
            for component in descriptor["components"]
            for item in component["files"]
            if item["path"].endswith("/.cargo-checksum.json")
        )
        invalid_cache(
            "vendor",
            lambda root: (root / vendor_checksum).write_bytes(b"{}"),
            "RUST_BUILD_INPUTS_VENDOR",
        )
        notice = next(
            item["path"]
            for component in descriptor["components"]
            for item in component["files"]
            if item["path"].startswith("notices/")
        )
        invalid_cache(
            "notice",
            lambda root: (root / notice).unlink(),
            "RUST_BUILD_INPUTS_LICENSE",
        )

        def mixed_cache(name: str, order: tuple[str, str]) -> None:
            root = work / name
            holding = work / f"{name}-holding"
            holding.mkdir()
            materialize_synthetic_cache(vector, descriptor, root)
            make_cache_writable(root)
            (root / "cargo-home-seed/config.toml").write_bytes(b"changed")
            (root / notice).write_bytes(b"changed")
            for directory in order:
                (root / directory).rename(holding / directory)
            for directory in order:
                (holding / directory).rename(root / directory)
            normalize_cache_modes(root, descriptor)
            self_test_expect(
                "RUST_BUILD_INPUTS_CARGO_HOME",
                lambda: validate_cache(descriptor, root=root),
            )

        mixed_cache("mixed-order-a", ("cargo-home-seed", "notices"))
        mixed_cache("mixed-order-b", ("notices", "cargo-home-seed"))
        self_test_expect(
            "RUST_BUILD_INPUTS_CACHE_KEY",
            lambda: publish_cache(valid, work / ("0" * 64), descriptor),
        )

        occupied = work / descriptor["build_inputs_sha256"]
        materialize_synthetic_cache(vector, descriptor, occupied)
        make_cache_writable(occupied)
        (occupied / first_path).write_bytes(b"unequal")
        normalize_cache_modes(occupied, descriptor)
        staging = work / "staging"
        materialize_synthetic_cache(vector, descriptor, staging)
        self_test_expect(
            "RUST_BUILD_INPUTS_PUBLICATION",
            lambda: publish_cache(staging, occupied, descriptor),
        )
        if (occupied / first_path).read_bytes() != b"unequal":
            raise RustBuildFailure()
        for child in work.iterdir():
            if child.is_dir() and not child.is_symlink():
                make_cache_writable(child)


def main(argv: list[str]) -> int:
    try:
        if argv == ["check-build-inputs"]:
            check_build_inputs()
        elif argv == ["update-build-inputs"]:
            update_build_inputs()
        elif argv == ["provision-build-inputs"]:
            update_build_inputs(provision=True)
        elif argv and argv[0] == "launch":
            return launch(argv[1:])
        elif argv == ["update-candidate"]:
            update_candidate(check=False)
        elif argv == ["check-candidate"]:
            update_candidate(check=True)
        elif argv == ["self-test"]:
            run_self_test()
        else:
            raise RustBuildFailure("BUNDLE_ASSEMBLER_USAGE", 64)
        return 0
    except RustBuildFailure as error:
        code = (
            "BUNDLE_BUILD_INPUTS_INVALID"
            if error.code.startswith("RUST_BUILD_INPUTS_")
            else error.code
        )
        sys.stderr.write(code + "\n")
        return error.exit_code
    except (OSError, KeyError, TypeError, ValueError, struct.error):
        sys.stderr.write("BUNDLE_ASSEMBLER_IO\n")
        return 74


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
