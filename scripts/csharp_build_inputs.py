#!/usr/bin/env python3
"""Hermetic build-input and reproducibility gate for the inactive C# frontend."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
import platform
import shutil
import ssl
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import urllib.parse
import zipfile
from pathlib import Path, PurePosixPath
from typing import BinaryIO, Iterator
from xml.etree import ElementTree


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
VECTOR_PATH = REPOSITORY_ROOT / "develop/specs/vectors/csharp-profile-v0.json"
DESCRIPTOR_PATH = REPOSITORY_ROOT / "release/build-inputs/csharp/build-inputs.json"
INVENTORY_PATH = REPOSITORY_ROOT / "release/build-inputs/csharp/candidate-inventory.json"
PROJECT_ROOT = REPOSITORY_ROOT / "csharp-tools/csharp2vir"
CACHE_PARENT = REPOSITORY_ROOT / "release/build-input-cache/csharp"
ACTIVE_REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/bundle-registry.json"
CAPTURE_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_capture_harness.cs"
ROSLYN_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_roslyn_session_harness.cs"
SUBSET_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_subset_harness.cs"
CONTRACT_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_contracts_harness.cs"
LOWERING_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_lowering_harness.cs"
EMISSION_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_emission_harness.cs"
FRONTEND_VECTOR_HARNESS_PATH = REPOSITORY_ROOT / "crates/mpk-cli/tests/csharp_frontend_vectors_harness.cs"

TOOLCHAIN_DOMAIN = b"MPK-CSHARP-TOOLCHAIN-INPUTS-0.1\0"
REFERENCE_DOMAIN = b"MPK-CSHARP-REFERENCE-INVENTORY-0.1\0"
TOOLCHAIN_HASH = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f"
REFERENCE_HASH = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad"
PROJECT_FILES = (
    "AssemblyInfo.cs",
    "Capture.cs",
    "Cli.cs",
    "ContractAttachment.cs",
    "ContractCanonical.cs",
    "ContractModel.cs",
    "ContractParser.cs",
    "EmissionCanonical.cs",
    "EmissionModel.cs",
    "EmissionProfiles.cs",
    "FrontendDiagnostics.cs",
    "FrontendLimits.cs",
    "FrontendModel.cs",
    "FrontendProtocol.cs",
    "FrontendSuccessEmitter.cs",
    "LoweringBuilder.cs",
    "LoweringModel.cs",
    "LoweringValidation.cs",
    "NOTICE.txt",
    "Program.cs",
    "RoslynAdapters.cs",
    "RoslynSession.cs",
    "Selection.cs",
    "SourceManifestEmitter.cs",
    "SourceMapEmitter.cs",
    "SourceTransport.cs",
    "SubsetModel.cs",
    "SubsetOperations.cs",
    "SubsetSymbols.cs",
    "SubsetValidator.cs",
    "VirEmitter.cs",
    "csharp2vir.csproj",
    "csharp2vir.deps.json",
    "csharp2vir.runtimeconfig.json",
)
COMPILER_ARGUMENTS = (
    "/nologo",
    "/noconfig",
    "/nostdlib+",
    "/deterministic+",
    "/optimize+",
    "/debug-",
    "/target:exe",
    "/platform:anycpu",
    "/langversion:14.0",
    "/nullable:enable",
    "/checked+",
    "/unsafe-",
    "/warnaserror+",
    "/utf8output",
    "/filealign:512",
    "/highentropyva+",
)
EXPECTED_TAR_STATS = {
    "dotnet-sdk-linux-x64": {
        ("directory", 0o755): 724,
        ("regular", 0o644): 4_075,
        ("regular", 0o744): 811,
        ("regular", 0o755): 21,
    },
    "dotnet-runtime-linux-x64": {
        ("directory", 0o755): 7,
        ("regular", 0o644): 177,
        ("regular", 0o755): 16,
    },
}
MAX_JSON_BYTES = 64 * 1024 * 1024
MAX_ARCHIVE_BYTES = 512 * 1024 * 1024
MAX_EXTRACTED_BYTES = 2 * 1024 * 1024 * 1024
MAX_ARCHIVE_ENTRIES = 16_384
BUILD_TIMEOUT_SECONDS = 300
VERSION_OUTPUT = b"csharp2vir 0.1.0 (Roslyn 5.6.0; .NET 10.0.11 profile)\n"


class CSharpBuildFailure(Exception):
    def __init__(self, code: str = "CSHARP_BUILD_INPUTS_INVALID", exit_code: int = 65):
        super().__init__(code)
        self.code = code
        self.exit_code = exit_code


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def strict_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise CSharpBuildFailure()
        result[key] = value
    return result


def reject_constant(_: str) -> object:
    raise CSharpBuildFailure()


def strict_json_bytes(data: bytes) -> object:
    if len(data) > MAX_JSON_BYTES or data.startswith(b"\xef\xbb\xbf"):
        raise CSharpBuildFailure()
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise CSharpBuildFailure() from error
    try:
        return json.loads(
            text,
            object_pairs_hook=strict_pairs,
            parse_constant=reject_constant,
        )
    except (json.JSONDecodeError, RecursionError) as error:
        raise CSharpBuildFailure() from error


def strict_json_file(path: Path, *, canonical_transport: bool = False) -> object:
    data = read_regular_bytes(path, MAX_JSON_BYTES)
    value = strict_json_bytes(data)
    if canonical_transport and data != canonical(value) + b"\n":
        raise CSharpBuildFailure()
    return value


def exact_keys(value: object, expected: set[str]) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != expected:
        raise CSharpBuildFailure()
    return value


def text(value: object) -> str:
    if not isinstance(value, str):
        raise CSharpBuildFailure()
    return value


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise CSharpBuildFailure()
    return value


def array(value: object) -> list[object]:
    if not isinstance(value, list):
        raise CSharpBuildFailure()
    return value


def raw_sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_sha256(value: object) -> str:
    return raw_sha256(canonical(value))


def typed_sha256(domain: bytes, value: object) -> str:
    return raw_sha256(domain + canonical(value))


@contextlib.contextmanager
def opened_regular(path: Path, maximum: int) -> Iterator[tuple[BinaryIO, os.stat_result]]:
    descriptor = -1
    stream: BinaryIO | None = None
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
        before = os.fstat(descriptor)
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_nlink != 1
            or before.st_size < 0
            or before.st_size > maximum
        ):
            raise CSharpBuildFailure()
        stream = os.fdopen(descriptor, "rb", closefd=False)
        yield stream, before
        after = os.fstat(descriptor)
        current = path.lstat()
        if (
            not stat.S_ISREG(after.st_mode)
            or after.st_nlink != 1
            or stat.S_ISLNK(current.st_mode)
            or not stat.S_ISREG(current.st_mode)
            or current.st_nlink != 1
            or (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
            != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns)
            or (before.st_dev, before.st_ino, before.st_size)
            != (current.st_dev, current.st_ino, current.st_size)
        ):
            raise CSharpBuildFailure()
    except OSError as error:
        raise CSharpBuildFailure() from error
    finally:
        if stream is not None:
            stream.close()
        if descriptor >= 0:
            os.close(descriptor)


def read_regular_bytes(path: Path, maximum: int) -> bytes:
    with opened_regular(path, maximum) as (stream, before):
        data = stream.read(maximum + 1)
        if len(data) != before.st_size:
            raise CSharpBuildFailure()
        return data


def hash_regular_file(path: Path, maximum: int = MAX_ARCHIVE_BYTES) -> tuple[int, str, str]:
    sha256 = hashlib.sha256()
    sha512 = hashlib.sha512()
    observed = 0
    with opened_regular(path, maximum) as (stream, before):
        while True:
            block = stream.read(1024 * 1024)
            if not block:
                break
            observed += len(block)
            if observed > maximum:
                raise CSharpBuildFailure()
            sha256.update(block)
            sha512.update(block)
        if observed != before.st_size:
            raise CSharpBuildFailure()
    return observed, sha256.hexdigest(), sha512.hexdigest()


def validate_relative_path(value: str, *, allow_root: bool = False) -> str:
    if not value or "\\" in value or "\0" in value or len(value.encode("utf-8")) > 4096:
        raise CSharpBuildFailure()
    while value.startswith("./"):
        value = value[2:]
    value = value.rstrip("/")
    if value in ("", "."):
        if allow_root:
            return ""
        raise CSharpBuildFailure()
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        raise CSharpBuildFailure()
    if ":" in path.parts[0]:
        raise CSharpBuildFailure()
    if any(ord(character) < 0x20 or ord(character) == 0x7F for character in value):
        raise CSharpBuildFailure()
    return path.as_posix()


def descriptor_cache_name(archive: dict[str, object]) -> str:
    identifier = validate_relative_path(text(archive["id"]))
    if "/" in identifier:
        raise CSharpBuildFailure()
    kind = text(archive["kind"])
    if kind == "tar.gz":
        return identifier + ".tar.gz"
    if kind == "nupkg":
        return identifier + ".nupkg"
    raise CSharpBuildFailure()


def validate_hex(value: object, length: int) -> str:
    result = text(value)
    if len(result) != length or any(character not in "0123456789abcdef" for character in result):
        raise CSharpBuildFailure()
    return result


def load_profile() -> tuple[dict[str, object], dict[str, object]]:
    profile = exact_keys(
        strict_json_file(VECTOR_PATH),
        {
            "schema", "owner_test", "spec_schema", "mechanism_schema", "profile_identity",
            "semantic_parameters", "selection_fixture", "selection_sha256", "contract_fixture",
            "contract_sidecar_sha256", "normalized_contract_fixture", "toolchain_inputs",
            "compiler_session", "launcher_contract", "case_harness", "source_map_cases",
            "profile_contracts", "type_mappings", "roslyn_checked_state_cases", "conversion_rules",
            "operation_mappings", "semantic_rows", "accepted_cases", "rejected_cases",
            "precedence_cases", "diagnostic_registry", "diagnostic_normalization", "limit_cases",
            "hash_cases", "isolation_cases", "upgrade_cases",
        },
    )
    toolchain = exact_keys(
        profile["toolchain_inputs"],
        {
            "schema", "id", "host", "roslyn_source", "archives", "package_graph",
            "managed_projection", "reference_projection", "toolchain_inputs_sha256",
        },
    )
    if text(toolchain["schema"]) != "mpk.csharp.toolchain_inputs.v0":
        raise CSharpBuildFailure()
    claimed = validate_hex(toolchain["toolchain_inputs_sha256"], 64)
    payload = dict(toolchain)
    del payload["toolchain_inputs_sha256"]
    if len(canonical(payload)) != 29_335 or typed_sha256(TOOLCHAIN_DOMAIN, payload) != claimed:
        raise CSharpBuildFailure()
    if claimed != TOOLCHAIN_HASH:
        raise CSharpBuildFailure()
    validate_toolchain(toolchain)
    return profile, toolchain


def validate_toolchain(toolchain: dict[str, object]) -> None:
    host = exact_keys(
        toolchain["host"],
        {
            "architecture", "os", "rid", "execution_host_profile_id",
            "runtime_layout_profile_id", "minimum_kernel_abi", "interpreter",
            "native_library_roots",
        },
    )
    expected_host = {
        "architecture": "x86_64",
        "os": "linux",
        "rid": "linux-x64",
        "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
        "runtime_layout_profile_id": "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
        "minimum_kernel_abi": "6.4.0",
        "interpreter": "/lib64/ld-linux-x86-64.so.2",
        "native_library_roots": ["/mpk/toolchain/dotnet", "/lib/x86_64-linux-gnu"],
    }
    if host != expected_host:
        raise CSharpBuildFailure()
    roslyn = exact_keys(toolchain["roslyn_source"], {"commit", "repository", "release_kind"})
    if roslyn != {
        "commit": "c0573ed0a7dc3e3b4d2e70da47f97cc51a35524f",
        "repository": "https://github.com/dotnet/roslyn",
        "release_kind": "stable",
    }:
        raise CSharpBuildFailure()
    archives = array(toolchain["archives"])
    if len(archives) != 6:
        raise CSharpBuildFailure()
    identifiers: set[str] = set()
    for untyped in archives:
        archive = exact_keys(untyped, {"id", "kind", "version", "url", "size_bytes", "sha256"} | ({"sha512"} if isinstance(untyped, dict) and untyped.get("kind") == "tar.gz" else set()))
        identifier = text(archive["id"])
        if identifier in identifiers or not text(archive["url"]).startswith("https://"):
            raise CSharpBuildFailure()
        identifiers.add(identifier)
        integer(archive["size_bytes"])
        validate_hex(archive["sha256"], 64)
        if "sha512" in archive:
            validate_hex(archive["sha512"], 128)
        descriptor_cache_name(archive)
    if identifiers != {
        "dotnet-runtime-linux-x64", "dotnet-sdk-linux-x64",
        "microsoft-codeanalysis-analyzers", "microsoft-codeanalysis-common",
        "microsoft-codeanalysis-csharp", "microsoft-netcore-app-ref",
    }:
        raise CSharpBuildFailure()
    validate_reference_projection(exact_keys(
        toolchain["reference_projection"],
        {
            "package_id", "version", "selector", "install_root", "count", "total_bytes",
            "canonical_payload_bytes", "hash_domain", "inventory_sha256", "metadata", "inventory",
        },
    ))


def validate_reference_projection(projection: dict[str, object]) -> None:
    if (
        text(projection["package_id"]) != "Microsoft.NETCore.App.Ref"
        or text(projection["version"]) != "10.0.11"
        or integer(projection["count"]) != 167
        or integer(projection["total_bytes"]) != 6_046_008
        or integer(projection["canonical_payload_bytes"]) != 24_670
        or text(projection["hash_domain"]) != "MPK-CSHARP-REFERENCE-INVENTORY-0.1"
        or validate_hex(projection["inventory_sha256"], 64) != REFERENCE_HASH
    ):
        raise CSharpBuildFailure()
    inventory = array(projection["inventory"])
    previous = ""
    total = 0
    for untyped in inventory:
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        path = validate_relative_path(text(record["path"]))
        if not path.startswith("ref/net10.0/") or path.count("/") != 2 or not path.endswith(".dll") or path <= previous:
            raise CSharpBuildFailure()
        previous = path
        total += integer(record["size_bytes"])
        validate_hex(record["sha256"], 64)
    if len(inventory) != 167 or total != 6_046_008:
        raise CSharpBuildFailure()
    if len(canonical(inventory)) != 24_670 or typed_sha256(REFERENCE_DOMAIN, inventory) != REFERENCE_HASH:
        raise CSharpBuildFailure()


def load_descriptor(toolchain: dict[str, object]) -> dict[str, object]:
    descriptor = exact_keys(
        strict_json_file(DESCRIPTOR_PATH, canonical_transport=True),
        {
            "schema", "toolchain_vector", "toolchain_inputs_sha256", "project_root",
            "project_files", "build_recipe", "notice_sources", "candidate_inventory",
        },
    )
    if (
        text(descriptor["schema"]) != "mpk.csharp.build_inputs.v0"
        or text(descriptor["toolchain_vector"]) != "develop/specs/vectors/csharp-profile-v0.json"
        or text(descriptor["toolchain_inputs_sha256"]) != text(toolchain["toolchain_inputs_sha256"])
        or text(descriptor["project_root"]) != "csharp-tools/csharp2vir"
        or text(descriptor["candidate_inventory"]) != "release/build-inputs/csharp/candidate-inventory.json"
    ):
        raise CSharpBuildFailure()
    validate_project_files(array(descriptor["project_files"]))
    recipe = exact_keys(
        descriptor["build_recipe"],
        {
            "id", "compiler", "compiler_arguments", "language_version", "target_framework",
            "runtime_framework", "runtime_version", "network_namespace", "package_restore",
            "source_date_epoch",
        },
    )
    if recipe != {
        "id": "mpk.csharp.build_recipe.csc_direct.v0",
        "compiler": "sdk/10.0.400/Roslyn/bincore/csc.dll",
        "compiler_arguments": list(COMPILER_ARGUMENTS),
        "language_version": "14.0",
        "target_framework": "net10.0",
        "runtime_framework": "Microsoft.NETCore.App",
        "runtime_version": "10.0.11",
        "network_namespace": "required",
        "package_restore": "forbidden",
        "source_date_epoch": 0,
    }:
        raise CSharpBuildFailure()
    validate_notice_sources(array(descriptor["notice_sources"]))
    validate_inactive_boundary()
    return descriptor


def validate_project_files(records: list[object]) -> None:
    if len(records) != len(PROJECT_FILES):
        raise CSharpBuildFailure()
    observed: list[str] = []
    for untyped in records:
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        relative = validate_relative_path(text(record["path"]))
        observed.append(relative)
        path = PROJECT_ROOT / relative
        metadata = path.lstat()
        if stat.S_IMODE(metadata.st_mode) != 0o644:
            raise CSharpBuildFailure()
        data = read_regular_bytes(path, 1024 * 1024)
        if len(data) != integer(record["size_bytes"]) or raw_sha256(data) != validate_hex(record["sha256"], 64):
            raise CSharpBuildFailure()
    if tuple(observed) != PROJECT_FILES:
        raise CSharpBuildFailure()


def validate_notice_sources(records: list[object]) -> None:
    if len(records) != 13:
        raise CSharpBuildFailure()
    outputs: list[str] = []
    for untyped in records:
        record = exact_keys(untyped, {"source", "path", "output"})
        source = text(record["source"])
        if source not in {
            "project", "dotnet-runtime-linux-x64", "dotnet-sdk-linux-x64",
            "microsoft-codeanalysis-analyzers", "microsoft-codeanalysis-common",
            "microsoft-codeanalysis-csharp", "microsoft-netcore-app-ref",
        }:
            raise CSharpBuildFailure()
        validate_relative_path(text(record["path"]))
        output = validate_relative_path(text(record["output"]))
        if not output.startswith("notices/"):
            raise CSharpBuildFailure()
        outputs.append(output)
    if outputs != sorted(outputs) or len(set(path.casefold() for path in outputs)) != len(outputs):
        raise CSharpBuildFailure()


def validate_inactive_boundary() -> None:
    registry = strict_json_file(ACTIVE_REGISTRY_PATH)
    encoded = canonical(registry)
    forbidden = (
        b'"source_language":"csharp"', b'"semantic_profile":"mpk.csharp.scalar.v0"',
        b'csharp2vir', b'mpk.semantic_profile.registry.v1',
    )
    if any(token in encoded for token in forbidden):
        raise CSharpBuildFailure("CSHARP_BUILD_ACTIVE_ROUTE")


def version_tuple(value: str) -> tuple[int, ...]:
    parts: list[int] = []
    for component in value.split("."):
        digits = "".join(character for character in component if character.isdigit())
        if not digits:
            break
        parts.append(int(digits))
    return tuple(parts)


def validate_build_host() -> None:
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "amd64"):
        raise CSharpBuildFailure("CSHARP_BUILD_HOST")
    if version_tuple(platform.release()) < (6, 4, 0):
        raise CSharpBuildFailure("CSHARP_BUILD_HOST")
    libc_name, libc_version = platform.libc_ver()
    if libc_name != "glibc" or version_tuple(libc_version) < (2, 27):
        raise CSharpBuildFailure("CSHARP_BUILD_HOST")
    interpreter = Path("/lib64/ld-linux-x86-64.so.2")
    native_root = Path("/lib/x86_64-linux-gnu")
    unshare = Path("/usr/bin/unshare")
    if (
        not interpreter.exists()
        or not interpreter.resolve().is_file()
        or not os.access(interpreter, os.X_OK)
        or not native_root.is_dir()
        or native_root.is_symlink()
        or not unshare.is_file()
        or unshare.is_symlink()
        or not os.access(unshare, os.X_OK)
    ):
        raise CSharpBuildFailure("CSHARP_BUILD_HOST")


def archive_records(toolchain: dict[str, object]) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    for item in array(toolchain["archives"]):
        if not isinstance(item, dict):
            raise CSharpBuildFailure()
        records.append(item)
    return records


def cache_root(toolchain: dict[str, object]) -> Path:
    return CACHE_PARENT / text(toolchain["toolchain_inputs_sha256"])


def require_plain_directory(path: Path) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise CSharpBuildFailure("CSHARP_BUILD_CACHE_MISSING", 66) from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
        raise CSharpBuildFailure()


def check_cached_archives(toolchain: dict[str, object]) -> Path:
    archives_root = cache_root(toolchain) / "archives"
    require_plain_directory(CACHE_PARENT)
    require_plain_directory(cache_root(toolchain))
    require_plain_directory(archives_root)
    try:
        actual = {entry.name for entry in archives_root.iterdir()}
    except OSError as error:
        raise CSharpBuildFailure("CSHARP_BUILD_CACHE_MISSING", 66) from error
    expected = {descriptor_cache_name(record) for record in archive_records(toolchain)}
    if actual != expected:
        raise CSharpBuildFailure("CSHARP_BUILD_CACHE_MISSING", 66)
    for record in archive_records(toolchain):
        path = archives_root / descriptor_cache_name(record)
        size, sha256, sha512 = hash_regular_file(path)
        if size != integer(record["size_bytes"]) or sha256 != text(record["sha256"]):
            raise CSharpBuildFailure()
        if "sha512" in record and sha512 != text(record["sha512"]):
            raise CSharpBuildFailure()
    return archives_root


def copy_exact_stream(source: BinaryIO, output: BinaryIO, expected_size: int) -> None:
    observed = 0
    while True:
        block = source.read(min(1024 * 1024, expected_size - observed + 1))
        if not block:
            break
        observed += len(block)
        if observed > expected_size or output.write(block) != len(block):
            raise CSharpBuildFailure()
    if observed != expected_size:
        raise CSharpBuildFailure()


def provision_build_inputs(toolchain: dict[str, object]) -> None:
    root = cache_root(toolchain)
    archives_root = root / "archives"
    archives_root.mkdir(mode=0o755, parents=True, exist_ok=True)
    require_plain_directory(CACHE_PARENT)
    require_plain_directory(root)
    require_plain_directory(archives_root)
    expected = {descriptor_cache_name(record) for record in archive_records(toolchain)}
    for child in archives_root.iterdir():
        if child.name not in expected:
            raise CSharpBuildFailure()
    context = ssl.create_default_context()
    for record in archive_records(toolchain):
        destination = archives_root / descriptor_cache_name(record)
        try:
            size, sha256, sha512 = hash_regular_file(destination)
            valid = size == integer(record["size_bytes"]) and sha256 == text(record["sha256"])
            valid = valid and ("sha512" not in record or sha512 == text(record["sha512"]))
        except CSharpBuildFailure:
            valid = False
        if valid:
            continue
        temporary_descriptor, temporary_name = tempfile.mkstemp(prefix=".download-", dir=archives_root)
        os.close(temporary_descriptor)
        temporary = Path(temporary_name)
        try:
            request = urllib.request.Request(text(record["url"]), headers={"User-Agent": "mpk-csharp-build-inputs/0"})
            expected_size = integer(record["size_bytes"])
            with urllib.request.urlopen(request, context=context, timeout=60) as response, temporary.open("wb") as output:
                if urllib.parse.urlsplit(response.geturl()).scheme != "https":
                    raise CSharpBuildFailure()
                copy_exact_stream(response, output, expected_size)
            size, sha256, sha512 = hash_regular_file(temporary)
            if size != integer(record["size_bytes"]) or sha256 != text(record["sha256"]):
                raise CSharpBuildFailure()
            if "sha512" in record and sha512 != text(record["sha512"]):
                raise CSharpBuildFailure()
            os.chmod(temporary, 0o444)
            os.replace(temporary, destination)
        finally:
            if temporary.exists():
                temporary.unlink()
    check_cached_archives(toolchain)


def copy_checked_archive(source: Path, destination: Path, record: dict[str, object]) -> None:
    destination.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    descriptor = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC, 0o600)
    observed = 0
    sha256 = hashlib.sha256()
    sha512 = hashlib.sha512()
    try:
        with opened_regular(source, MAX_ARCHIVE_BYTES) as (stream, before):
            while True:
                block = stream.read(1024 * 1024)
                if not block:
                    break
                observed += len(block)
                if observed > MAX_ARCHIVE_BYTES:
                    raise CSharpBuildFailure()
                remaining = memoryview(block)
                while remaining:
                    written = os.write(descriptor, remaining)
                    if written <= 0:
                        raise CSharpBuildFailure()
                    remaining = remaining[written:]
                sha256.update(block)
                sha512.update(block)
            if observed != before.st_size:
                raise CSharpBuildFailure()
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if (
        observed != integer(record["size_bytes"])
        or sha256.hexdigest() != text(record["sha256"])
        or ("sha512" in record and sha512.hexdigest() != text(record["sha512"]))
    ):
        raise CSharpBuildFailure()
    os.chmod(destination, 0o444)


def normalized_members(names: list[str], *, allow_root: bool) -> list[str]:
    result: list[str] = []
    exact: set[str] = set()
    folded: set[str] = set()
    for name in names:
        normalized = validate_relative_path(name, allow_root=allow_root)
        if normalized in exact or normalized.casefold() in folded:
            raise CSharpBuildFailure()
        exact.add(normalized)
        folded.add(normalized.casefold())
        result.append(normalized)
    return result


def extract_tar(archive_path: Path, destination: Path, archive_id: str) -> None:
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        if len(members) > MAX_ARCHIVE_ENTRIES:
            raise CSharpBuildFailure()
        normalized = normalized_members([member.name for member in members], allow_root=True)
        stats: dict[tuple[str, int], int] = {}
        total = 0
        directories: set[str] = set()
        for member, relative in zip(members, normalized, strict=True):
            mode = member.mode & 0o7777
            if mode & 0o7000:
                raise CSharpBuildFailure()
            if member.isdir():
                kind = "directory"
                directories.add(relative)
            elif member.isfile():
                kind = "regular"
                total += member.size
                if total > MAX_EXTRACTED_BYTES:
                    raise CSharpBuildFailure()
                parent = PurePosixPath(relative).parent.as_posix()
                if parent != "." and parent not in directories:
                    raise CSharpBuildFailure()
            else:
                raise CSharpBuildFailure()
            stats[(kind, mode)] = stats.get((kind, mode), 0) + 1
        if stats != EXPECTED_TAR_STATS.get(archive_id):
            raise CSharpBuildFailure()
        destination.mkdir(mode=0o755, parents=True, exist_ok=False)
        for member, relative in zip(members, normalized, strict=True):
            if relative == "":
                continue
            target = destination / relative
            if member.isdir():
                target.mkdir(mode=member.mode & 0o777, parents=False, exist_ok=False)
                continue
            extracted = archive.extractfile(member)
            if extracted is None:
                raise CSharpBuildFailure()
            target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
            with target.open("xb") as output:
                shutil.copyfileobj(extracted, output, 1024 * 1024)
            if target.stat().st_size != member.size:
                raise CSharpBuildFailure()
            os.chmod(target, member.mode & 0o777)
            os.utime(target, ns=(0, 0), follow_symlinks=False)
        for relative in sorted((path for path in directories if path), key=lambda value: value.count("/"), reverse=True):
            target = destination / relative
            os.chmod(target, 0o755)
            os.utime(target, ns=(0, 0), follow_symlinks=False)
        os.chmod(destination, 0o755)
        os.utime(destination, ns=(0, 0), follow_symlinks=False)


def zip_entry_kind(info: zipfile.ZipInfo) -> str:
    mode = info.external_attr >> 16
    file_type = stat.S_IFMT(mode)
    if info.is_dir():
        if file_type not in (0, stat.S_IFDIR):
            raise CSharpBuildFailure()
        return "directory"
    if file_type not in (0, stat.S_IFREG):
        raise CSharpBuildFailure()
    return "regular"


def extract_zip(archive_path: Path, destination: Path) -> None:
    with zipfile.ZipFile(archive_path) as archive:
        infos = archive.infolist()
        if len(infos) > MAX_ARCHIVE_ENTRIES:
            raise CSharpBuildFailure()
        normalized = normalized_members([info.filename for info in infos], allow_root=False)
        total = 0
        for info in infos:
            if info.flag_bits & 1 or info.compress_type not in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED):
                raise CSharpBuildFailure()
            if zip_entry_kind(info) == "regular":
                total += info.file_size
                if total > MAX_EXTRACTED_BYTES:
                    raise CSharpBuildFailure()
        destination.mkdir(mode=0o755, parents=True, exist_ok=False)
        for info, relative in zip(infos, normalized, strict=True):
            target = destination / relative
            if zip_entry_kind(info) == "directory":
                target.mkdir(mode=0o755, parents=True, exist_ok=False)
                continue
            target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
            with archive.open(info, "r") as source, target.open("xb") as output:
                shutil.copyfileobj(source, output, 1024 * 1024)
            if target.stat().st_size != info.file_size:
                raise CSharpBuildFailure()
            os.chmod(target, 0o644)
            os.utime(target, ns=(0, 0), follow_symlinks=False)
        for directory in sorted(
            (path for path in destination.rglob("*") if path.is_dir()),
            key=lambda path: len(path.parts),
            reverse=True,
        ):
            os.chmod(directory, 0o755)
            os.utime(directory, ns=(0, 0), follow_symlinks=False)
        os.chmod(destination, 0o755)
        os.utime(destination, ns=(0, 0), follow_symlinks=False)


def materialize_closure(toolchain: dict[str, object], archives_root: Path, root: Path) -> dict[str, Path]:
    copied_root = root / "archives"
    extracted_root = root / "extracted"
    result: dict[str, Path] = {}
    for record in archive_records(toolchain):
        archive_id = text(record["id"])
        cache_name = descriptor_cache_name(record)
        copied = copied_root / cache_name
        copy_checked_archive(archives_root / cache_name, copied, record)
        destination = extracted_root / archive_id
        if text(record["kind"]) == "tar.gz":
            extract_tar(copied, destination, archive_id)
        else:
            extract_zip(copied, destination)
        result[archive_id] = destination
    validate_extracted_closure(toolchain, result)
    return result


def validate_extracted_file(path: Path, size: int, sha256: str) -> None:
    observed_size, observed_hash, _ = hash_regular_file(path, MAX_EXTRACTED_BYTES)
    if observed_size != size or observed_hash != sha256:
        raise CSharpBuildFailure()


def validate_extracted_closure(toolchain: dict[str, object], roots: dict[str, Path]) -> None:
    package_by_id = {
        "Microsoft.CodeAnalysis.Analyzers": "microsoft-codeanalysis-analyzers",
        "Microsoft.CodeAnalysis.Common": "microsoft-codeanalysis-common",
        "Microsoft.CodeAnalysis.CSharp": "microsoft-codeanalysis-csharp",
        "Microsoft.NETCore.App.Ref": "microsoft-netcore-app-ref",
    }
    for untyped in array(toolchain["managed_projection"]):
        record = exact_keys(untyped, {"package_id", "archive_path", "runtime_path", "size_bytes", "sha256"})
        package = package_by_id.get(text(record["package_id"]))
        if package is None:
            raise CSharpBuildFailure()
        path = roots[package] / validate_relative_path(text(record["archive_path"]))
        validate_extracted_file(path, integer(record["size_bytes"]), validate_hex(record["sha256"], 64))
    projection = exact_keys(toolchain["reference_projection"], set(toolchain["reference_projection"]))
    reference_root = roots["microsoft-netcore-app-ref"]
    expected_paths: list[str] = []
    for untyped in array(projection["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        relative = validate_relative_path(text(record["path"]))
        expected_paths.append(relative)
        validate_extracted_file(reference_root / relative, integer(record["size_bytes"]), validate_hex(record["sha256"], 64))
    actual_paths = sorted(
        path.relative_to(reference_root).as_posix()
        for path in (reference_root / "ref/net10.0").iterdir()
        if path.is_file() and not path.is_symlink() and path.name.endswith(".dll")
    )
    if actual_paths != expected_paths:
        raise CSharpBuildFailure()
    for untyped in array(projection["metadata"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        relative = validate_relative_path(text(record["path"]))
        validate_extracted_file(reference_root / relative, integer(record["size_bytes"]), validate_hex(record["sha256"], 64))
    validate_package_graph(toolchain, roots, package_by_id)


def child_text(parent: ElementTree.Element, name: str) -> str:
    child = parent.find("{*}" + name)
    if child is None or child.text is None:
        raise CSharpBuildFailure()
    return child.text


def normalize_dependency_version(value: str) -> str:
    if value.startswith("[") and value.endswith("]"):
        parts = [part.strip() for part in value[1:-1].split(",")]
        if len(parts) == 2 and parts[0] == parts[1]:
            return parts[0]
    return value


def validate_package_graph(toolchain: dict[str, object], roots: dict[str, Path], package_by_id: dict[str, str]) -> None:
    expected: dict[str, tuple[str, list[tuple[str, str]]]] = {}
    for untyped in array(toolchain["package_graph"]):
        record = exact_keys(untyped, {"package_id", "version", "use", "dependencies"})
        dependencies: list[tuple[str, str]] = []
        for dependency in array(record["dependencies"]):
            item = exact_keys(dependency, {"package_id", "version"})
            dependencies.append((text(item["package_id"]), text(item["version"])))
        expected[text(record["package_id"])] = (text(record["version"]), dependencies)
    if set(expected) != set(package_by_id):
        raise CSharpBuildFailure()
    for package_id, archive_id in package_by_id.items():
        nuspecs = sorted(roots[archive_id].glob("*.nuspec"))
        if len(nuspecs) != 1:
            raise CSharpBuildFailure()
        try:
            document = ElementTree.fromstring(read_regular_bytes(nuspecs[0], 1024 * 1024))
        except ElementTree.ParseError as error:
            raise CSharpBuildFailure() from error
        metadata = document.find("{*}metadata")
        if metadata is None or child_text(metadata, "id") != package_id or child_text(metadata, "version") != expected[package_id][0]:
            raise CSharpBuildFailure()
        dependencies: list[tuple[str, str]] = []
        container = metadata.find("{*}dependencies")
        if container is not None:
            groups = [group for group in container.findall("{*}group") if group.get("targetFramework") == "net10.0"]
            if len(groups) != 1:
                raise CSharpBuildFailure()
            for dependency in groups[0].findall("{*}dependency"):
                identifier = dependency.get("id")
                version = dependency.get("version")
                if identifier is None or version is None:
                    raise CSharpBuildFailure()
                dependencies.append((identifier, normalize_dependency_version(version)))
        if sorted(dependencies) != sorted(expected[package_id][1]):
            raise CSharpBuildFailure()


def copy_project(descriptor: dict[str, object], destination: Path) -> Path:
    project = destination / "project"
    project.mkdir(mode=0o755, parents=True, exist_ok=False)
    os.chmod(project, 0o755)
    for untyped in array(descriptor["project_files"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        relative = validate_relative_path(text(record["path"]))
        source = PROJECT_ROOT / relative
        target = project / relative
        target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
        with opened_regular(source, 1024 * 1024) as (input_stream, before), target.open("xb") as output:
            shutil.copyfileobj(input_stream, output, 1024 * 1024)
        if target.stat().st_size != before.st_size:
            raise CSharpBuildFailure()
        os.chmod(target, 0o444)
        os.utime(target, ns=(0, 0), follow_symlinks=False)
    return project


def closed_dotnet_environment(dotnet_root: Path, work: Path) -> dict[str, str]:
    empty_home = work / "empty-home"
    empty_nuget = work / "empty-nuget"
    temporary = work / "tmp"
    for path in (empty_home, empty_nuget, temporary):
        path.mkdir(mode=0o700, parents=True, exist_ok=False)
    return {
        "COMPlus_ReadyToRun": "0",
        "DOTNET_CLI_HOME": str(empty_home),
        "DOTNET_CLI_TELEMETRY_OPTOUT": "1",
        "DOTNET_MULTILEVEL_LOOKUP": "0",
        "DOTNET_NOLOGO": "1",
        "DOTNET_ROOT": str(dotnet_root),
        "DOTNET_SKIP_FIRST_TIME_EXPERIENCE": "1",
        "DOTNET_SYSTEM_GLOBALIZATION_INVARIANT": "1",
        "DOTNET_TieredCompilation": "0",
        "DOTNET_TieredPGO": "0",
        "HOME": str(empty_home),
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "NUGET_HTTP_CACHE_PATH": str(empty_nuget),
        "NUGET_PACKAGES": str(empty_nuget),
        "NUGET_PLUGINS_CACHE_PATH": str(empty_nuget),
        "PATH": "/nonexistent",
        "SOURCE_DATE_EPOCH": "0",
        "TMPDIR": str(temporary),
        "TZ": "UTC",
    }


def execute_isolated(argv: list[str], *, cwd: Path, environment: dict[str, str]) -> subprocess.CompletedProcess[bytes]:
    command = ["/usr/bin/unshare", "--user", "--map-root-user", "--net", "--"] + argv
    try:
        return subprocess.run(
            command,
            cwd=cwd,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=BUILD_TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CSharpBuildFailure("CSHARP_BUILD_EXECUTION") from error


def copy_candidate_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    os.chmod(destination.parent, 0o755)
    data = read_regular_bytes(source, MAX_EXTRACTED_BYTES)
    with destination.open("xb") as output:
        output.write(data)
    os.chmod(destination, 0o644)
    os.utime(destination, ns=(0, 0), follow_symlinks=False)


def build_once(
    toolchain: dict[str, object],
    descriptor: dict[str, object],
    archives_root: Path,
    root: Path,
    *,
    run_capture_tests: bool = False,
    run_roslyn_tests: bool = False,
    run_subset_tests: bool = False,
    run_contract_tests: bool = False,
    run_lowering_tests: bool = False,
    run_emission_tests: bool = False,
    run_frontend_vector_tests: bool = False,
) -> Path:
    roots = materialize_closure(toolchain, archives_root, root / "closure")
    project = copy_project(descriptor, root)
    candidate = root / "candidate"
    frontend = candidate / "frontend"
    frontend.mkdir(mode=0o755, parents=True, exist_ok=False)
    os.chmod(candidate, 0o755)
    os.chmod(frontend, 0o755)
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    dotnet = sdk / "dotnet"
    validate_extracted_file(compiler, compiler.stat().st_size, hash_regular_file(compiler)[1])
    references = exact_keys(toolchain["reference_projection"], set(toolchain["reference_projection"]))["inventory"]
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(frontend / "csharp2vir.dll"),
            "/pathmap:" + str(project) + "=/_/csharp2vir",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(references):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    managed_roots = {
        "Microsoft.CodeAnalysis.Common": roots["microsoft-codeanalysis-common"],
        "Microsoft.CodeAnalysis.CSharp": roots["microsoft-codeanalysis-csharp"],
    }
    for untyped in array(toolchain["managed_projection"]):
        record = exact_keys(untyped, {"package_id", "archive_path", "runtime_path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(managed_roots[text(record["package_id"])] / text(record["archive_path"])))
    arguments.extend(str(project / path) for path in PROJECT_FILES if path.endswith(".cs"))
    environment = closed_dotnet_environment(sdk, root / "build-environment")
    result = execute_isolated([str(dotnet), "exec", str(compiler)] + arguments, cwd=project, environment=environment)
    if result.returncode != 0 or result.stdout or result.stderr:
        raise CSharpBuildFailure("CSHARP_BUILD_COMPILER")
    expected_output = {"csharp2vir.dll"}
    actual_output = {entry.name for entry in frontend.iterdir()}
    if actual_output != expected_output:
        raise CSharpBuildFailure()
    copy_candidate_file(project / "csharp2vir.deps.json", frontend / "csharp2vir.deps.json")
    copy_candidate_file(project / "csharp2vir.runtimeconfig.json", frontend / "csharp2vir.runtimeconfig.json")
    for untyped in array(toolchain["managed_projection"]):
        record = exact_keys(untyped, {"package_id", "archive_path", "runtime_path", "size_bytes", "sha256"})
        output_name = PurePosixPath(text(record["runtime_path"])).name
        copy_candidate_file(managed_roots[text(record["package_id"])] / text(record["archive_path"]), frontend / output_name)
    copy_notices(descriptor, roots, project, candidate)
    validate_candidate_runtime(roots["dotnet-runtime-linux-x64"], candidate, root / "runtime-environment")
    if run_capture_tests:
        validate_capture_implementation(toolchain, roots, candidate, root / "capture-tests")
    if run_roslyn_tests:
        validate_roslyn_session_implementation(toolchain, roots, candidate, root / "roslyn-tests")
    if run_subset_tests:
        validate_subset_implementation(toolchain, roots, candidate, root / "subset-tests")
    if run_contract_tests:
        validate_contract_implementation(toolchain, roots, candidate, root / "contract-tests")
    if run_lowering_tests:
        validate_lowering_implementation(toolchain, roots, candidate, root / "lowering-tests")
    if run_emission_tests:
        validate_emission_implementation(toolchain, roots, candidate, root / "emission-tests")
    if run_frontend_vector_tests:
        validate_frontend_vector_implementation(
            toolchain,
            roots,
            candidate,
            root / "frontend-vector-tests",
        )
    for directory in (path for path in candidate.rglob("*") if path.is_dir()):
        os.chmod(directory, 0o755)
        os.utime(directory, ns=(0, 0), follow_symlinks=False)
    os.chmod(candidate, 0o755)
    os.utime(candidate, ns=(0, 0), follow_symlinks=False)
    return candidate


def validate_capture_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
) -> None:
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "CaptureHarness.cs"
    with opened_regular(CAPTURE_HARNESS_PATH, 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_CAPTURE_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-capture-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.CaptureHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_CAPTURE_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise CSharpBuildFailure("CSHARP_CAPTURE_TEST_FAILURE")
    finally:
        output.unlink(missing_ok=True)


def validate_roslyn_session_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
) -> None:
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "RoslynSessionHarness.cs"
    with opened_regular(ROSLYN_HARNESS_PATH, 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_ROSLYN_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-roslyn-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.RoslynSessionHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_ROSLYN_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise CSharpBuildFailure("CSHARP_ROSLYN_TEST_FAILURE")
    finally:
        output.unlink(missing_ok=True)


def validate_subset_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
) -> None:
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "SubsetHarness.cs"
    with opened_regular(SUBSET_HARNESS_PATH, 2 * 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_SUBSET_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-subset-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.SubsetHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_SUBSET_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise CSharpBuildFailure("CSHARP_SUBSET_TEST_FAILURE")
    finally:
        output.unlink(missing_ok=True)


def validate_contract_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
) -> None:
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "ContractHarness.cs"
    with opened_regular(CONTRACT_HARNESS_PATH, 2 * 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_CONTRACT_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-contract-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.ContractHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_CONTRACT_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise CSharpBuildFailure("CSHARP_CONTRACT_TEST_FAILURE")
    finally:
        output.unlink(missing_ok=True)


def validate_lowering_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
) -> None:
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "LoweringHarness.cs"
    with opened_regular(LOWERING_HARNESS_PATH, 2 * 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_LOWERING_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-lowering-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.LoweringHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_LOWERING_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                str(reference_root),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise CSharpBuildFailure("CSHARP_LOWERING_TEST_FAILURE")
    finally:
        output.unlink(missing_ok=True)


def validate_emission_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
    *,
    mode: str = "self-test",
) -> bytes:
    if mode not in ("self-test", "emit"):
        raise CSharpBuildFailure("CSHARP_EMISSION_TEST_MODE")
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "EmissionHarness.cs"
    with opened_regular(EMISSION_HARNESS_PATH, 2 * 1024 * 1024) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_EMISSION_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-emission-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.EmissionHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_EMISSION_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                mode,
                str(reference_root),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if (
            result.returncode != 0
            or result.stderr
            or (mode == "self-test" and result.stdout)
            or (mode == "emit" and (not result.stdout or not result.stdout.endswith(b"\n")))
        ):
            raise CSharpBuildFailure("CSHARP_EMISSION_TEST_FAILURE")
        return result.stdout
    finally:
        output.unlink(missing_ok=True)


def validate_frontend_vector_implementation(
    toolchain: dict[str, object],
    roots: dict[str, Path],
    candidate: Path,
    work: Path,
    *,
    mode: str = "self-test",
) -> bytes:
    if mode not in ("self-test", "report"):
        raise CSharpBuildFailure("CSHARP_FRONTEND_VECTOR_TEST_MODE")
    work.mkdir(mode=0o700, parents=True, exist_ok=False)
    harness = work / "FrontendVectorHarness.cs"
    with opened_regular(
        FRONTEND_VECTOR_HARNESS_PATH,
        4 * 1024 * 1024,
    ) as (input_stream, before), harness.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    if harness.stat().st_size != before.st_size:
        raise CSharpBuildFailure("CSHARP_FRONTEND_VECTOR_TEST_BUILD")
    os.chmod(harness, 0o444)
    os.utime(harness, ns=(0, 0), follow_symlinks=False)
    frontend = candidate / "frontend"
    output = frontend / "csharp2vir-frontend-vector-tests.dll"
    sdk = roots["dotnet-sdk-linux-x64"]
    compiler = sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"
    arguments = list(COMPILER_ARGUMENTS)
    arguments.extend(
        [
            "/out:" + str(output),
            "/main:Mpk.CSharp2Vir.FrontendVectorHarness",
            "/pathmap:" + str(REPOSITORY_ROOT) + "=/_/mpk",
        ]
    )
    reference_root = roots["microsoft-netcore-app-ref"]
    for untyped in array(toolchain["reference_projection"]["inventory"]):
        record = exact_keys(untyped, {"path", "size_bytes", "sha256"})
        arguments.append("/reference:" + str(reference_root / text(record["path"])))
    arguments.extend(
        [
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.dll"),
            "/reference:" + str(frontend / "Microsoft.CodeAnalysis.CSharp.dll"),
            "/reference:" + str(frontend / "csharp2vir.dll"),
            str(harness),
        ]
    )
    build_environment = closed_dotnet_environment(sdk, work / "build-environment")
    result = execute_isolated(
        [str(sdk / "dotnet"), "exec", str(compiler)] + arguments,
        cwd=REPOSITORY_ROOT,
        environment=build_environment,
    )
    if result.returncode != 0 or result.stdout or result.stderr or not output.is_file():
        raise CSharpBuildFailure("CSHARP_FRONTEND_VECTOR_TEST_BUILD")
    try:
        runtime = roots["dotnet-runtime-linux-x64"]
        runtime_environment = closed_dotnet_environment(runtime, work / "runtime-environment")
        result = execute_isolated(
            [
                str(runtime / "dotnet"),
                "exec",
                "--runtimeconfig",
                str(frontend / "csharp2vir.runtimeconfig.json"),
                "--fx-version",
                "10.0.11",
                str(output),
                mode,
                str(reference_root),
                str(VECTOR_PATH),
            ],
            cwd=candidate,
            environment=runtime_environment,
        )
        if (
            result.returncode != 0
            or result.stderr
            or (mode == "self-test" and result.stdout)
            or (mode == "report" and (not result.stdout or not result.stdout.endswith(b"\n")))
        ):
            raise CSharpBuildFailure("CSHARP_FRONTEND_VECTOR_TEST_FAILURE")
        return result.stdout
    finally:
        output.unlink(missing_ok=True)


def copy_notices(descriptor: dict[str, object], roots: dict[str, Path], project: Path, candidate: Path) -> None:
    for untyped in array(descriptor["notice_sources"]):
        record = exact_keys(untyped, {"source", "path", "output"})
        source_id = text(record["source"])
        source_root = project if source_id == "project" else roots[source_id]
        source = source_root / validate_relative_path(text(record["path"]))
        destination = candidate / validate_relative_path(text(record["output"]))
        copy_candidate_file(source, destination)


def validate_candidate_runtime(runtime: Path, candidate: Path, work: Path) -> None:
    environment = closed_dotnet_environment(runtime, work)
    frontend = candidate / "frontend"
    result = execute_isolated(
        [
            str(runtime / "dotnet"), "exec", "--depsfile", str(frontend / "csharp2vir.deps.json"),
            "--runtimeconfig", str(frontend / "csharp2vir.runtimeconfig.json"), "--fx-version", "10.0.11",
            str(frontend / "csharp2vir.dll"), "--version",
        ],
        cwd=candidate,
        environment=environment,
    )
    if result.returncode != 0 or result.stdout != VERSION_OUTPUT or result.stderr:
        raise CSharpBuildFailure("CSHARP_BUILD_RUNTIME")
    unavailable = execute_isolated(
        [
            str(runtime / "dotnet"), "exec", "--depsfile", str(frontend / "csharp2vir.deps.json"),
            "--runtimeconfig", str(frontend / "csharp2vir.runtimeconfig.json"), "--fx-version", "10.0.11",
            str(frontend / "csharp2vir.dll"),
        ],
        cwd=candidate,
        environment=environment,
    )
    if unavailable.returncode != 2 or unavailable.stdout or unavailable.stderr != b"CSHARP_FRONTEND_USAGE\n":
        raise CSharpBuildFailure("CSHARP_BUILD_RUNTIME")


def inventory_record(root: Path, path: Path) -> dict[str, object]:
    relative = path.relative_to(root).as_posix()
    metadata = path.lstat()
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode) or stat.S_IMODE(metadata.st_mode) != 0o644:
        raise CSharpBuildFailure()
    size, sha256, _ = hash_regular_file(path, MAX_EXTRACTED_BYTES)
    return {"mode": "0644", "path": relative, "sha256": sha256, "size_bytes": size}


def candidate_inventory(candidate: Path, descriptor: dict[str, object]) -> dict[str, object]:
    files = sorted(path for path in candidate.rglob("*") if path.is_file())
    frontend = [inventory_record(candidate, path) for path in files if path.relative_to(candidate).parts[0] == "frontend"]
    notices = [inventory_record(candidate, path) for path in files if path.relative_to(candidate).parts[0] == "notices"]
    if len(frontend) != 5 or len(notices) != 13 or len(frontend) + len(notices) != len(files):
        raise CSharpBuildFailure()
    return {
        "schema": "mpk.csharp.frontend_candidate_inventory.v0",
        "toolchain_inputs_sha256": TOOLCHAIN_HASH,
        "project_files_sha256": canonical_sha256(array(descriptor["project_files"])),
        "build_recipe_sha256": canonical_sha256(descriptor["build_recipe"]),
        "frontend_files": frontend,
        "notice_files": notices,
    }


def compare_candidates(left: Path, right: Path) -> None:
    left_paths = sorted(path.relative_to(left).as_posix() for path in left.rglob("*") if path.is_file())
    right_paths = sorted(path.relative_to(right).as_posix() for path in right.rglob("*") if path.is_file())
    if left_paths != right_paths:
        raise CSharpBuildFailure("CSHARP_BUILD_NONDETERMINISTIC")
    for relative in left_paths:
        left_data = read_regular_bytes(left / relative, MAX_EXTRACTED_BYTES)
        right_data = read_regular_bytes(right / relative, MAX_EXTRACTED_BYTES)
        if left_data != right_data or stat.S_IMODE((left / relative).stat().st_mode) != stat.S_IMODE((right / relative).stat().st_mode):
            raise CSharpBuildFailure("CSHARP_BUILD_NONDETERMINISTIC")


def build_twice(toolchain: dict[str, object], descriptor: dict[str, object]) -> tuple[dict[str, object], bytes]:
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-build-a-") as left_name, tempfile.TemporaryDirectory(prefix="mpk-csharp-build-b-") as right_name:
        left = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(left_name),
            run_capture_tests=True,
            run_roslyn_tests=True,
            run_subset_tests=True,
            run_contract_tests=True,
            run_lowering_tests=True,
            run_emission_tests=True,
            run_frontend_vector_tests=True,
        )
        right = build_once(toolchain, descriptor, archives_root, Path(right_name))
        compare_candidates(left, right)
        inventory = candidate_inventory(left, descriptor)
        archive = deterministic_candidate_archive(left)
        if archive != deterministic_candidate_archive(right):
            raise CSharpBuildFailure("CSHARP_BUILD_NONDETERMINISTIC")
        return inventory, archive


def deterministic_candidate_archive(candidate: Path) -> bytes:
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w", format=tarfile.USTAR_FORMAT) as archive:
        for path in sorted(candidate.rglob("*"), key=lambda item: item.relative_to(candidate).as_posix()):
            relative = path.relative_to(candidate).as_posix()
            info = tarfile.TarInfo(relative + ("/" if path.is_dir() else ""))
            info.uid = 0
            info.gid = 0
            info.uname = ""
            info.gname = ""
            info.mtime = 0
            info.mode = 0o755 if path.is_dir() else 0o644
            if path.is_dir():
                info.type = tarfile.DIRTYPE
                archive.addfile(info)
            else:
                data = read_regular_bytes(path, MAX_EXTRACTED_BYTES)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
    return output.getvalue()


def load_candidate_inventory() -> dict[str, object]:
    value = exact_keys(
        strict_json_file(INVENTORY_PATH, canonical_transport=True),
        {
            "schema", "toolchain_inputs_sha256", "project_files_sha256",
            "build_recipe_sha256", "frontend_files", "notice_files",
        },
    )
    if text(value["schema"]) != "mpk.csharp.frontend_candidate_inventory.v0" or text(value["toolchain_inputs_sha256"]) != TOOLCHAIN_HASH:
        raise CSharpBuildFailure()
    for field, expected_count in (("frontend_files", 5), ("notice_files", 13)):
        records = array(value[field])
        if len(records) != expected_count:
            raise CSharpBuildFailure()
        previous = ""
        for untyped in records:
            record = exact_keys(untyped, {"mode", "path", "sha256", "size_bytes"})
            path = validate_relative_path(text(record["path"]))
            if path <= previous or text(record["mode"]) != "0644":
                raise CSharpBuildFailure()
            previous = path
            integer(record["size_bytes"])
            validate_hex(record["sha256"], 64)
    return value


def write_candidate_inventory(value: dict[str, object]) -> None:
    INVENTORY_PATH.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(prefix=".candidate-inventory-", dir=INVENTORY_PATH.parent)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(canonical(value) + b"\n")
            output.flush()
            os.fsync(output.fileno())
        os.chmod(name, 0o644)
        os.replace(name, INVENTORY_PATH)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def check_full(update: bool) -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    generated, _ = build_twice(toolchain, descriptor)
    if update:
        write_candidate_inventory(generated)
    elif generated != load_candidate_inventory():
        raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")
    validate_project_files(array(descriptor["project_files"]))


def build_to(destination_text: str) -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    expected = load_candidate_inventory()
    destination = Path(destination_text)
    ancestors = [destination.parent, *destination.parent.parents]
    if (
        not destination.is_absolute()
        or destination.exists()
        or not destination.parent.is_dir()
        or any(path.is_symlink() for path in ancestors)
    ):
        raise CSharpBuildFailure("CSHARP_BUILD_OUTPUT")
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-build-") as temporary:
        candidate = build_once(toolchain, descriptor, archives_root, Path(temporary))
        if candidate_inventory(candidate, descriptor) != expected:
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")
        shutil.copytree(candidate, destination, symlinks=False)


def test_capture() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-capture-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_capture_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def test_roslyn() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-roslyn-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_roslyn_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def test_subset() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-subset-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_subset_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def test_contracts() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-contract-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_contract_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def test_lowering() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-lowering-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_lowering_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def test_emission() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-emission-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_emission_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def emit_test_envelope() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-emission-envelope-") as temporary:
        root = Path(temporary)
        candidate = build_once(toolchain, descriptor, archives_root, root)
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")
        envelope = validate_emission_implementation(
            toolchain,
            materialize_closure(toolchain, archives_root, root / "emission-closure"),
            candidate,
            root / "emission-tests",
            mode="emit",
        )
        sys.stdout.buffer.write(envelope)
        sys.stdout.buffer.flush()


def test_frontend_vectors() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-frontend-vector-test-") as temporary:
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            Path(temporary),
            run_frontend_vector_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")


def emit_frontend_vector_report() -> None:
    validate_build_host()
    _, toolchain = load_profile()
    descriptor = load_descriptor(toolchain)
    archives_root = check_cached_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-frontend-vector-report-") as temporary:
        root = Path(temporary)
        candidate = build_once(
            toolchain,
            descriptor,
            archives_root,
            root,
            run_capture_tests=True,
            run_roslyn_tests=True,
            run_subset_tests=True,
            run_contract_tests=True,
            run_lowering_tests=True,
            run_emission_tests=True,
        )
        if candidate_inventory(candidate, descriptor) != load_candidate_inventory():
            raise CSharpBuildFailure("CSHARP_BUILD_INVENTORY_MISMATCH")
        report = validate_frontend_vector_implementation(
            toolchain,
            materialize_closure(toolchain, archives_root, root / "frontend-vector-closure"),
            candidate,
            root / "frontend-vector-tests",
            mode="report",
        )
        sys.stdout.buffer.write(report)
        sys.stdout.buffer.flush()


def self_test() -> None:
    _, toolchain = load_profile()
    load_descriptor(toolchain)
    try:
        strict_json_bytes(b'{"a":1,"a":2}')
    except CSharpBuildFailure:
        pass
    else:
        raise CSharpBuildFailure()
    copied = io.BytesIO()
    copy_exact_stream(io.BytesIO(b"exact"), copied, 5)
    if copied.getvalue() != b"exact":
        raise CSharpBuildFailure()
    for payload, expected_size in ((b"short", 6), (b"too-long", 7)):
        try:
            copy_exact_stream(io.BytesIO(payload), io.BytesIO(), expected_size)
        except CSharpBuildFailure:
            pass
        else:
            raise CSharpBuildFailure()
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-self-test-") as temporary:
        root = Path(temporary)
        safe_tar = root / "safe.tar.gz"
        with tarfile.open(safe_tar, "w:gz") as archive:
            directory = tarfile.TarInfo("./dir")
            directory.type = tarfile.DIRTYPE
            directory.mode = 0o755
            archive.addfile(directory)
            data = b"safe"
            regular = tarfile.TarInfo("./dir/file")
            regular.size = len(data)
            regular.mode = 0o644
            archive.addfile(regular, io.BytesIO(data))
        previous = EXPECTED_TAR_STATS.get("synthetic")
        EXPECTED_TAR_STATS["synthetic"] = {("directory", 0o755): 1, ("regular", 0o644): 1}
        try:
            extract_tar(safe_tar, root / "safe", "synthetic")
        finally:
            if previous is None:
                del EXPECTED_TAR_STATS["synthetic"]
        if (root / "safe/dir/file").read_bytes() != b"safe" or stat.S_IMODE((root / "safe/dir/file").stat().st_mode) != 0o644:
            raise CSharpBuildFailure()
        unsafe_tar = root / "unsafe.tar.gz"
        with tarfile.open(unsafe_tar, "w:gz") as archive:
            regular = tarfile.TarInfo("../escape")
            regular.size = 1
            regular.mode = 0o644
            archive.addfile(regular, io.BytesIO(b"x"))
        EXPECTED_TAR_STATS["synthetic"] = {("regular", 0o644): 1}
        try:
            try:
                extract_tar(unsafe_tar, root / "unsafe", "synthetic")
            except CSharpBuildFailure:
                pass
            else:
                raise CSharpBuildFailure()
        finally:
            del EXPECTED_TAR_STATS["synthetic"]
        unsafe_zip = root / "unsafe.nupkg"
        with zipfile.ZipFile(unsafe_zip, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.writestr("A/file", b"one")
            archive.writestr("a/FILE", b"two")
        try:
            extract_zip(unsafe_zip, root / "unsafe-zip")
        except CSharpBuildFailure:
            pass
        else:
            raise CSharpBuildFailure()
        setid_tar = root / "setid.tar.gz"
        with tarfile.open(setid_tar, "w:gz") as archive:
            regular = tarfile.TarInfo("setid")
            regular.size = 1
            regular.mode = 0o4644
            archive.addfile(regular, io.BytesIO(b"x"))
        EXPECTED_TAR_STATS["synthetic"] = {("regular", 0o4644): 1}
        try:
            try:
                extract_tar(setid_tar, root / "setid", "synthetic")
            except CSharpBuildFailure:
                pass
            else:
                raise CSharpBuildFailure()
        finally:
            del EXPECTED_TAR_STATS["synthetic"]
        symlink_zip = root / "symlink.nupkg"
        with zipfile.ZipFile(symlink_zip, "w", zipfile.ZIP_DEFLATED) as archive:
            link = zipfile.ZipInfo("link")
            link.create_system = 3
            link.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(link, "target")
        try:
            extract_zip(symlink_zip, root / "symlink-zip")
        except CSharpBuildFailure:
            pass
        else:
            raise CSharpBuildFailure()


def main(argv: list[str]) -> int:
    try:
        if argv == ["provision-build-inputs"]:
            _, toolchain = load_profile()
            descriptor = load_descriptor(toolchain)
            provision_build_inputs(toolchain)
            validate_project_files(array(descriptor["project_files"]))
        elif argv == ["check-build-inputs"]:
            _, toolchain = load_profile()
            descriptor = load_descriptor(toolchain)
            archives = check_cached_archives(toolchain)
            with tempfile.TemporaryDirectory(prefix="mpk-csharp-input-check-") as temporary:
                materialize_closure(toolchain, archives, Path(temporary))
            validate_project_files(array(descriptor["project_files"]))
        elif argv == ["update-inventory"]:
            check_full(update=True)
        elif argv == ["check"]:
            check_full(update=False)
        elif argv == ["self-test"]:
            self_test()
        elif argv == ["test-capture"]:
            test_capture()
        elif argv == ["test-roslyn"]:
            test_roslyn()
        elif argv == ["test-subset"]:
            test_subset()
        elif argv == ["test-contracts"]:
            test_contracts()
        elif argv == ["test-lowering"]:
            test_lowering()
        elif argv == ["test-emission"]:
            test_emission()
        elif argv == ["emit-test-envelope"]:
            emit_test_envelope()
        elif argv == ["test-frontend-vectors"]:
            test_frontend_vectors()
        elif argv == ["emit-frontend-vector-report"]:
            emit_frontend_vector_report()
        elif len(argv) == 2 and argv[0] == "build":
            build_to(argv[1])
        else:
            raise CSharpBuildFailure("CSHARP_BUILD_USAGE", 64)
        return 0
    except CSharpBuildFailure as error:
        sys.stderr.write(error.code + "\n")
        return error.exit_code
    except (OSError, KeyError, TypeError, ValueError, zipfile.BadZipFile, tarfile.TarError):
        sys.stderr.write("CSHARP_BUILD_IO\n")
        return 74


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
