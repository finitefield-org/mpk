#!/usr/bin/env python3
"""C# bundle builder used only by the active successor release assembler."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile

import csharp_build_inputs


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
PROFILE_VECTOR_PATH = REPOSITORY_ROOT / "develop/specs/vectors/csharp-profile-v0.json"
SEMANTIC_VECTOR_PATH = (
    REPOSITORY_ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json"
)
ACTIVE_REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/bundle-registry.json"
RUST_BUILD_INPUTS_PATH = REPOSITORY_ROOT / "release/build-inputs/rust/build-inputs.json"
LIBC_COMPAT_SOURCE_PATH = REPOSITORY_ROOT / "release/build-inputs/csharp/libc-compat.c"

REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
REGISTRY_SCHEMA = "mpk.release.bundle_registry.v1"
REGISTRY_ID = "mpk.release.registry.v1"
CANDIDATE_SCHEMA = "mpk.release.bundle_candidate.v1"
FRONTEND_SCHEMA = "mpk.release.frontend_bundle.v1"
TOOLCHAIN_SCHEMA = "mpk.release.toolchain_bundle.v1"
FRONTEND_ID = "frontend.csharp.csharp2vir.candidate.v1"
TOOLCHAIN_ID = "toolchain.csharp.roslyn-5_6_0.dotnet-10_0_11.candidate.v1"
FRONTEND_VERSION = "0.1.0"
HOST_PROFILE_ID = "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
RUNTIME_LAYOUT_ID = "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
PROFILE_ENTRY_SHA256 = (
    "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac"
)
TOOLCHAIN_INPUTS_SHA256 = (
    "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f"
)
REFERENCE_INVENTORY_SHA256 = (
    "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad"
)
CSHARP_FRONTEND_SHA256 = (
    "e245d50913f8589be6fd763aa9185d3812b932a5dabc0e21ed03744eafa09f49"
)
LIBC_COMPAT_SOURCE_SHA256 = (
    "bd2ba2b47f7b7ad7620565a6b3e456cf5c60607ce39c4cf6cec9ec8054b28cd6"
)
LIBC_COMPAT_BINARY_SHA256 = (
    "9a29b778058e30382c1eb6c27bb3ab40d9375997540dceb89fedd0739fa12ac7"
)
NATIVE_RUNTIME_PROJECTION = (
    ("ld-linux-x86-64.so.2", "native-runtime/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2"),
    ("libc.so.6", "native-runtime/lib/x86_64-linux-gnu/libc.so.6"),
    ("libcrypto.so.1.1", "native-sysroot/usr/lib/x86_64-linux-gnu/libcrypto.so.1.1"),
    ("libdl.so.2", "native-runtime/lib/x86_64-linux-gnu/libdl.so.2"),
    ("libgcc_s.so.1", "native-runtime/lib/x86_64-linux-gnu/libgcc_s.so.1"),
    ("libm.so.6", "native-runtime/lib/x86_64-linux-gnu/libm.so.6"),
    ("libpthread.so.0", "native-runtime/lib/x86_64-linux-gnu/libpthread.so.0"),
    ("librt.so.1", "native-runtime/lib/x86_64-linux-gnu/librt.so.1"),
    ("libssl.so.1.1", "native-sysroot/usr/lib/x86_64-linux-gnu/libssl.so.1.1"),
    ("libstdc++.so.6", "native-runtime/lib/x86_64-linux-gnu/libstdc++.so.6"),
    ("libz.so.1", "native-runtime/lib/x86_64-linux-gnu/libz.so.1"),
)
NATIVE_RUNTIME_LIBRARIES = tuple(item[0] for item in NATIVE_RUNTIME_PROJECTION)
DOTNET_DIRECT_LIBRARIES = (
    "ld-linux-x86-64.so.2",
    "libc.so.6",
    "libdl.so.2",
    "libgcc_s.so.1",
    "libm.so.6",
    "libpthread.so.0",
    "librt.so.1",
    "libstdc++.so.6",
)
FORBIDDEN_EXECUTION_NAMES = (
    "csc.dll",
    "msbuild.dll",
    "nuget",
    "vbc.dll",
    "vstest",
)


class CSharpReleaseFailure(Exception):
    def __init__(self, code: str = "CSHARP_RELEASE_INVALID", exit_code: int = 65) -> None:
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


def canonical_line(value: object) -> bytes:
    return canonical(value) + b"\n"


def typed_hash(domain: bytes, value: object) -> str:
    return hashlib.sha256(domain + canonical(value)).hexdigest()


def raw_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def read_canonical_json(path: Path) -> tuple[dict[str, object], bytes]:
    data = path.read_bytes()
    try:
        value = json.loads(
            data,
            object_pairs_hook=strict_pairs,
            parse_constant=reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise CSharpReleaseFailure() from error
    if not isinstance(value, dict) or canonical_line(value) != data:
        raise CSharpReleaseFailure()
    return value, data


def read_json(path: Path) -> dict[str, object]:
    data = path.read_bytes()
    try:
        value = json.loads(
            data,
            object_pairs_hook=strict_pairs,
            parse_constant=reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise CSharpReleaseFailure() from error
    if not isinstance(value, dict):
        raise CSharpReleaseFailure()
    return value


def strict_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError("duplicate JSON name")
        value[key] = item
    return value


def reject_constant(_: str) -> object:
    raise ValueError("non-finite JSON number")


def require_plain_directory(path: Path) -> None:
    metadata = path.lstat()
    if path.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
        raise CSharpReleaseFailure()


def copy_regular(source: Path, destination: Path, *, executable: bool) -> None:
    metadata = source.lstat()
    if source.is_symlink() or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise CSharpReleaseFailure()
    destination.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    shutil.copyfile(source, destination, follow_symlinks=False)
    destination.chmod(0o555 if executable else 0o444)


def normalize_directories(root: Path) -> None:
    for path in sorted(root.rglob("*"), key=lambda item: len(item.parts), reverse=True):
        metadata = path.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not (
            stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode)
        ):
            raise CSharpReleaseFailure()
        if stat.S_ISDIR(metadata.st_mode):
            path.chmod(0o555)
    root.chmod(0o555)


def inventory(root: Path) -> list[dict[str, object]]:
    require_plain_directory(root)
    files: list[dict[str, object]] = []
    directories: set[str] = set()
    folded: set[str] = set()
    for path in sorted(
        root.rglob("*"), key=lambda item: item.relative_to(root).as_posix().encode("utf-8")
    ):
        metadata = path.lstat()
        relative = path.relative_to(root).as_posix()
        if stat.S_ISLNK(metadata.st_mode):
            raise CSharpReleaseFailure()
        if stat.S_ISDIR(metadata.st_mode):
            if stat.S_IMODE(metadata.st_mode) != 0o555:
                raise CSharpReleaseFailure()
            directories.add(relative)
            continue
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise CSharpReleaseFailure()
        mode = stat.S_IMODE(metadata.st_mode)
        if mode not in (0o444, 0o555):
            raise CSharpReleaseFailure()
        folded_path = relative.lower()
        if folded_path in folded:
            raise CSharpReleaseFailure()
        folded.add(folded_path)
        files.append(
            {
                "path": relative,
                "executable": mode == 0o555,
                "size_bytes": metadata.st_size,
                "sha256": raw_hash(path),
            }
        )
    implied = {
        parent.as_posix()
        for record in files
        for parent in Path(str(record["path"])).parents
        if parent != Path(".")
    }
    if directories != implied:
        raise CSharpReleaseFailure()
    return files


def bundle_inventory(scope: dict[str, str], files: list[dict[str, object]]) -> dict[str, object]:
    return {
        "schema": "mpk.release.bundle_inventory.v0",
        "scope": scope,
        "files": files,
    }


def component_inventory(
    bundle_id: str, component_name: str, files: list[dict[str, object]]
) -> dict[str, object]:
    return bundle_inventory(
        {
            "kind": "component",
            "bundle_id": bundle_id,
            "component_name": component_name,
        },
        files,
    )


def rust_native_sources() -> tuple[Path, dict[str, dict[str, object]]]:
    descriptor, _ = read_canonical_json(RUST_BUILD_INPUTS_PATH)
    build_hash = descriptor.get("build_inputs_sha256")
    if not isinstance(build_hash, str) or len(build_hash) != 64:
        raise CSharpReleaseFailure()
    cache = REPOSITORY_ROOT / "release/build-input-cache/rust" / build_hash
    require_plain_directory(cache)

    active, _active_bytes = read_canonical_json(ACTIVE_REGISTRY_PATH)
    if (
        active.get("schema") != REGISTRY_SCHEMA
        or active.get("id") != REGISTRY_ID
        or active.get("registry_sha256")
        != "7877c7c04fae912815713a8a7f6f9900198721ea572788f6f48d1dbe3f00afbd"
        or len(active.get("frontend_bundles", [])) != 4
        or len(active.get("toolchain_bundles", [])) != 4
        or len(active.get("tuples", [])) != 5
    ):
        raise CSharpReleaseFailure("BUNDLE_REGISTERED_STATE")
    components = descriptor.get("components")
    if not isinstance(components, list):
        raise CSharpReleaseFailure()
    native = next(
        (
            item
            for item in components
            if isinstance(item, dict) and item.get("name") == "native-runtime"
        ),
        None,
    )
    if not isinstance(native, dict):
        raise CSharpReleaseFailure()
    files = native.get("files")
    if not isinstance(files, list):
        raise CSharpReleaseFailure()
    records = {
        str(item["path"]): item
        for item in files
        if isinstance(item, dict)
        and isinstance(item.get("path"), str)
        and str(item["path"]).startswith("native-runtime/")
    }
    sysroot = next(
        (
            item
            for item in components
            if isinstance(item, dict) and item.get("name") == "native-sysroot"
        ),
        None,
    )
    if not isinstance(sysroot, dict) or not isinstance(sysroot.get("files"), list):
        raise CSharpReleaseFailure()
    records.update(
        {
            str(item["path"]): item
            for item in sysroot["files"]
            if isinstance(item, dict)
            and isinstance(item.get("path"), str)
            and str(item["path"]).startswith("native-sysroot/")
        }
    )
    tools = next(
        (
            item
            for item in components
            if isinstance(item, dict) and item.get("name") == "toolchain"
        ),
        None,
    )
    if not isinstance(tools, dict) or not isinstance(tools.get("files"), list):
        raise CSharpReleaseFailure()
    records.update(
        {
            str(item["path"]): item
            for item in tools["files"]
            if isinstance(item, dict)
            and isinstance(item.get("path"), str)
            and str(item["path"]).startswith("toolchain/")
        }
    )
    return cache, records


def described_file(
    cache: Path, described: dict[str, dict[str, object]], relative: str
) -> Path:
    record = described.get(relative)
    source = cache / relative
    metadata = source.lstat()
    executable = isinstance(record, dict) and record.get("executable") is True
    expected_mode = 0o555 if executable else 0o444
    if (
        not isinstance(record, dict)
        or set(record) != {"executable", "path", "sha256", "size_bytes"}
        or record.get("path") != relative
        or not isinstance(record.get("executable"), bool)
        or source.is_symlink()
        or not stat.S_ISREG(metadata.st_mode)
        or metadata.st_nlink != 1
        or stat.S_IMODE(metadata.st_mode) != expected_mode
        or record.get("size_bytes") != metadata.st_size
        or record.get("sha256") != raw_hash(source)
    ):
        raise CSharpReleaseFailure()
    return source


def build_libc_compat(
    cache: Path,
    described: dict[str, dict[str, object]],
    work: Path,
    destination: Path,
) -> None:
    source_metadata = LIBC_COMPAT_SOURCE_PATH.lstat()
    if (
        LIBC_COMPAT_SOURCE_PATH.is_symlink()
        or not stat.S_ISREG(source_metadata.st_mode)
        or source_metadata.st_nlink != 1
        or stat.S_IMODE(source_metadata.st_mode) != 0o644
        or raw_hash(LIBC_COMPAT_SOURCE_PATH) != LIBC_COMPAT_SOURCE_SHA256
    ):
        raise CSharpReleaseFailure()
    clang = described_file(cache, described, "toolchain/bin/clang")
    described_file(cache, described, "toolchain/bin/clang.cfg")
    described_file(cache, described, "toolchain/bin/ld.lld")
    libtinfo = described_file(
        cache,
        described,
        "native-runtime/lib/x86_64-linux-gnu/libtinfo.so.5",
    )
    native_sysroot = cache / "native-sysroot"
    require_plain_directory(native_sysroot)
    library_root = work / "compiler-libraries"
    library_root.mkdir(mode=0o700, parents=True, exist_ok=False)
    copy_regular(libtinfo, library_root / "libtinfo.so.5", executable=False)
    output = work / "libc.so"
    command = [
        str(clang),
        "--target=x86_64-unknown-linux-gnu",
        f"--sysroot={native_sysroot}",
        "-fuse-ld=lld",
        "-shared",
        "-fPIC",
        "-O2",
        "-fno-ident",
        "-nostdlib",
        "-Wl,--build-id=none",
        "-Wl,-soname,libc.so",
        "-Wl,-z,defs",
        "-Wl,-z,relro",
        "-Wl,-z,now",
        str(LIBC_COMPAT_SOURCE_PATH),
        f"-L{native_sysroot / 'lib/x86_64-linux-gnu'}",
        "-lc",
        "-o",
        str(output),
    ]
    try:
        result = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            env={
                "LANG": "C.UTF-8",
                "LC_ALL": "C.UTF-8",
                "LD_LIBRARY_PATH": str(library_root),
                "PATH": str(cache / "toolchain/bin"),
                "SOURCE_DATE_EPOCH": "0",
                "TZ": "UTC",
            },
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=60,
            check=False,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise CSharpReleaseFailure() from error
    if (
        result.returncode != 0
        or result.stdout
        or result.stderr
        or output.stat().st_size != 2_944
        or raw_hash(output) != LIBC_COMPAT_BINARY_SHA256
    ):
        raise CSharpReleaseFailure()
    copy_regular(output, destination, executable=False)


def copy_native_runtime(destination: Path, work: Path) -> None:
    cache, described = rust_native_sources()
    library_root = destination / "lib/x86_64-linux-gnu"
    for soname, described_path in NATIVE_RUNTIME_PROJECTION:
        source = described_file(cache, described, described_path)
        copy_regular(source, library_root / soname, executable=False)
    build_libc_compat(cache, described, work, library_root / "libc.so")
    loader = library_root / "ld-linux-x86-64.so.2"
    copy_regular(loader, destination / "lib64/ld-linux-x86-64.so.2", executable=True)


def copy_runtime(runtime: Path, destination: Path) -> None:
    for path in sorted(runtime.rglob("*")):
        metadata = path.lstat()
        if stat.S_ISDIR(metadata.st_mode):
            continue
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise CSharpReleaseFailure()
        relative = path.relative_to(runtime)
        copy_regular(path, destination / relative, executable=bool(metadata.st_mode & 0o111))


def copy_references(
    toolchain: dict[str, object], reference_root: Path, destination: Path
) -> None:
    projection = toolchain.get("reference_projection")
    if (
        not isinstance(projection, dict)
        or projection.get("inventory_sha256") != REFERENCE_INVENTORY_SHA256
    ):
        raise CSharpReleaseFailure()
    records = projection.get("inventory")
    if not isinstance(records, list) or len(records) != 167:
        raise CSharpReleaseFailure()
    for record in records:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            raise CSharpReleaseFailure()
        relative = Path(str(record["path"]))
        source = reference_root / relative
        metadata = source.lstat()
        if (
            relative.is_absolute()
            or ".." in relative.parts
            or source.is_symlink()
            or not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or metadata.st_size != record.get("size_bytes")
            or raw_hash(source) != record.get("sha256")
        ):
            raise CSharpReleaseFailure()
        copy_regular(source, destination / relative, executable=False)


def candidate_roots(work: Path) -> tuple[Path, dict[str, Path], dict[str, object]]:
    _profile, toolchain = csharp_build_inputs.load_profile()
    descriptor = csharp_build_inputs.load_descriptor(toolchain)
    archives = csharp_build_inputs.check_cached_archives(toolchain)
    candidate = csharp_build_inputs.build_once(toolchain, descriptor, archives, work)
    if (
        csharp_build_inputs.candidate_inventory(candidate, descriptor)
        != csharp_build_inputs.load_candidate_inventory()
    ):
        raise CSharpReleaseFailure("CSHARP_BUILD_INVENTORY_MISMATCH")
    roots = {
        archive_id: work / "closure/extracted" / archive_id
        for archive_id in (
            "dotnet-runtime-linux-x64",
            "microsoft-netcore-app-ref",
        )
    }
    for root in roots.values():
        require_plain_directory(root)
    return candidate, roots, toolchain


def materialize_bundle_roots(work: Path, destination: Path) -> tuple[Path, Path]:
    candidate, roots, toolchain = candidate_roots(work)
    frontend = destination / "frontend"
    toolchain_output = destination / "toolchain"
    frontend.mkdir(mode=0o755, parents=True, exist_ok=False)
    toolchain_output.mkdir(mode=0o755, parents=True, exist_ok=False)

    for source in sorted((candidate / "frontend").iterdir()):
        copy_regular(source, frontend / source.name, executable=False)
    for source in sorted((candidate / "notices").iterdir()):
        copy_regular(source, frontend / "notices" / source.name, executable=False)
    if raw_hash(frontend / "csharp2vir.dll") != CSHARP_FRONTEND_SHA256:
        raise CSharpReleaseFailure()

    copy_runtime(roots["dotnet-runtime-linux-x64"], toolchain_output / "dotnet")
    copy_references(
        toolchain,
        roots["microsoft-netcore-app-ref"],
        toolchain_output / "reference-pack",
    )
    copy_native_runtime(toolchain_output / "native-runtime", work / "native-build")
    normalize_directories(frontend)
    normalize_directories(toolchain_output)
    return frontend, toolchain_output


def inventory_subset(
    files: list[dict[str, object]], prefix: str, *, exclude: set[str] | None = None
) -> list[dict[str, object]]:
    excluded = exclude or set()
    return [
        record
        for record in files
        if str(record["path"]).startswith(prefix) and record["path"] not in excluded
    ]


def inventory_file(
    files: list[dict[str, object]], path: str
) -> dict[str, object]:
    matches = [record for record in files if record.get("path") == path]
    if len(matches) != 1:
        raise CSharpReleaseFailure()
    return matches[0]


def profile_contract(profile: dict[str, object], field: str) -> dict[str, object]:
    contracts = profile.get("profile_contracts")
    if not isinstance(contracts, list):
        raise CSharpReleaseFailure()
    matches = [
        item.get("envelope")
        for item in contracts
        if isinstance(item, dict) and item.get("field") == field
    ]
    if len(matches) != 1 or not isinstance(matches[0], dict):
        raise CSharpReleaseFailure()
    return matches[0]


def dynamic_runtime_record(native_files: list[dict[str, object]]) -> dict[str, object]:
    by_name = {Path(str(item["path"])).name: item for item in native_files}
    libraries = []
    for soname in DOTNET_DIRECT_LIBRARIES:
        item = by_name.get(soname)
        if item is None:
            raise CSharpReleaseFailure()
        libraries.append(
            {
                "soname": soname,
                "component_path": f"lib/x86_64-linux-gnu/{soname}",
                "sha256": item["sha256"],
            }
        )
    libraries.sort(key=lambda item: str(item["soname"]).encode("utf-8"))
    return {
        "kind": "dynamic",
        "interpreter_mount": "/lib64/ld-linux-x86-64.so.2",
        "libraries": libraries,
    }


def build_models(
    frontend_root: Path, toolchain_root: Path
) -> tuple[dict[str, object], dict[str, object]]:
    profile, toolchain_inputs = csharp_build_inputs.load_profile()
    semantic_vectors = read_json(SEMANTIC_VECTOR_PATH)
    active, _ = read_canonical_json(ACTIVE_REGISTRY_PATH)
    semantic_registry = semantic_vectors.get("registry")
    if not isinstance(semantic_registry, dict):
        raise CSharpReleaseFailure()
    profile_registry = {
        "schema": semantic_registry.get("schema"),
        "id": semantic_registry.get("id"),
        "revision": semantic_registry.get("revision"),
        "registry_sha256": semantic_registry.get("registry_sha256"),
    }
    if profile_registry != {
        "schema": "mpk.semantic_profile.registry.v1",
        "id": "mpk.semantic_profile.registry.v1",
        "revision": 3,
        "registry_sha256": "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557",
    }:
        raise CSharpReleaseFailure()

    hosts = active.get("execution_host_profiles")
    layouts = active.get("native_runtime_layout_profiles")
    if not isinstance(hosts, list) or not isinstance(layouts, list):
        raise CSharpReleaseFailure()
    host = next((item for item in hosts if item.get("id") == HOST_PROFILE_ID), None)
    layout = next((item for item in layouts if item.get("id") == RUNTIME_LAYOUT_ID), None)
    if not isinstance(host, dict) or not isinstance(layout, dict):
        raise CSharpReleaseFailure()

    frontend_files = inventory(frontend_root)
    toolchain_files = inventory(toolchain_root)
    frontend_inventory = bundle_inventory(
        {"kind": "frontend_bundle", "bundle_id": FRONTEND_ID}, frontend_files
    )
    toolchain_inventory = bundle_inventory(
        {"kind": "toolchain_bundle", "bundle_id": TOOLCHAIN_ID}, toolchain_files
    )
    main_file = next(
        (item for item in frontend_files if item["path"] == "csharp2vir.dll"), None
    )
    dotnet_file = next(
        (item for item in toolchain_files if item["path"] == "dotnet/dotnet"), None
    )
    if (
        main_file is None
        or main_file["executable"]
        or main_file["sha256"] != CSHARP_FRONTEND_SHA256
        or dotnet_file is None
        or not dotnet_file["executable"]
    ):
        raise CSharpReleaseFailure()

    dotnet_content_files = inventory_subset(
        toolchain_files, "dotnet/", exclude={"dotnet/dotnet"}
    )
    reference_files = inventory_subset(toolchain_files, "reference-pack/")
    native_files = inventory_subset(toolchain_files, "native-runtime/")
    components: list[dict[str, object]] = []
    for name, release, files in (
        ("dotnet-runtime", "10.0.11", dotnet_content_files),
        ("native-runtime", "glibc-2.27-layout-v0", native_files),
        ("reference-pack", "Microsoft.NETCore.App.Ref-10.0.11", reference_files),
    ):
        component = component_inventory(TOOLCHAIN_ID, name, files)
        components.append(
            {
                "kind": "content",
                "name": name,
                "release": release,
                "inventory": component,
                "content_sha256": typed_hash(CONTENT_DOMAIN, component),
            }
        )
    components.append(
        {
            "kind": "executable",
            "name": "dotnet",
            "release": "10.0.11",
            "path": "dotnet/dotnet",
            "binary_sha256": dotnet_file["sha256"],
            "runtime": dynamic_runtime_record(native_files),
        }
    )
    components.sort(key=lambda item: str(item["name"]).encode("utf-8"))

    frontend = {
        "schema": FRONTEND_SCHEMA,
        "bundle_id": FRONTEND_ID,
        "name": "csharp2vir",
        "version": FRONTEND_VERSION,
        "profile_contracts": [profile_contract(profile, "frontend")],
        "main": {
            "name": "csharp2vir",
            "version": FRONTEND_VERSION,
            "path": "csharp2vir.dll",
            "binary_sha256": main_file["sha256"],
            "runtime": {"kind": "static"},
        },
        "subordinate_binaries": [
            {
                "name": name,
                "version": "5.6.0",
                "path": name,
                "binary_sha256": inventory_file(frontend_files, name)["sha256"],
                "runtime": {"kind": "static"},
            }
            for name in (
                "Microsoft.CodeAnalysis.CSharp.dll",
                "Microsoft.CodeAnalysis.dll",
            )
        ],
        "inventory": frontend_inventory,
        "bundle_sha256": typed_hash(CONTENT_DOMAIN, frontend_inventory),
    }
    toolchain = {
        "schema": TOOLCHAIN_SCHEMA,
        "bundle_id": TOOLCHAIN_ID,
        "execution_host_profile_id": HOST_PROFILE_ID,
        "profile_contracts": [profile_contract(profile, "release")],
        "components": components,
        "inventory": toolchain_inventory,
        "distribution_sha256": typed_hash(CONTENT_DOMAIN, toolchain_inventory),
    }
    if toolchain_inputs.get("toolchain_inputs_sha256") != TOOLCHAIN_INPUTS_SHA256:
        raise CSharpReleaseFailure()

    semantic_context = {
        "profile_registry": profile_registry,
        "profile_entry_sha256": PROFILE_ENTRY_SHA256,
        "source_language": "csharp",
        "semantic_profile": "mpk.csharp.scalar.v0",
        "semantic_parameters": profile.get("semantic_parameters"),
    }
    release_tuple = {
        "semantic_context": semantic_context,
        "limit_profile_id": "mpk.vir.limits.v0",
        "frontend_bundle_id": FRONTEND_ID,
        "toolchain_bundle_id": TOOLCHAIN_ID,
    }
    projection = {
        "profile_registry": profile_registry,
        "execution_host_profiles": [host],
        "native_runtime_layout_profiles": [layout],
        "frontend_bundles": [frontend],
        "toolchain_bundles": [toolchain],
        "tuples": [release_tuple],
    }
    candidate = {"schema": CANDIDATE_SCHEMA, **projection}
    registry = {"schema": REGISTRY_SCHEMA, "id": REGISTRY_ID, **projection}
    registry["registry_sha256"] = typed_hash(REGISTRY_DOMAIN, registry)
    validate_model(candidate, registry)
    return candidate, registry


def validate_model(candidate: dict[str, object], registry: dict[str, object]) -> None:
    if (
        candidate.get("schema") != CANDIDATE_SCHEMA
        or registry.get("schema") != REGISTRY_SCHEMA
        or registry.get("id") != REGISTRY_ID
    ):
        raise CSharpReleaseFailure()
    projection_fields = (
        "profile_registry",
        "execution_host_profiles",
        "native_runtime_layout_profiles",
        "frontend_bundles",
        "toolchain_bundles",
        "tuples",
    )
    if any(candidate[field] != registry[field] for field in projection_fields):
        raise CSharpReleaseFailure()
    claimed = registry.get("registry_sha256")
    payload = dict(registry)
    payload.pop("registry_sha256", None)
    if claimed != typed_hash(REGISTRY_DOMAIN, payload):
        raise CSharpReleaseFailure()
    forbidden = canonical(candidate).lower()
    if any(name.encode("ascii") in forbidden for name in FORBIDDEN_EXECUTION_NAMES):
        raise CSharpReleaseFailure()


def build_once(root: Path) -> tuple[dict[str, object], dict[str, object], Path]:
    output = root / "bundles"
    frontend, toolchain = materialize_bundle_roots(root / "work", output)
    candidate, registry = build_models(frontend, toolchain)
    return candidate, registry, output


def update_active_candidate() -> None:
    destination = REPOSITORY_ROOT / "release/bundles/candidates/csharp.json"
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-candidate-") as temporary:
        candidate, _registry, _output = build_once(Path(temporary))
    staging = destination.with_name(f".{destination.name}.tmp")
    staging.write_bytes(canonical(candidate) + b"\n")
    os.replace(staging, destination)


if __name__ == "__main__":
    try:
        if sys.argv[1:] != ["update-active-candidate"]:
            raise CSharpReleaseFailure("CSHARP_RELEASE_USAGE", 64)
        update_active_candidate()
    except (CSharpReleaseFailure, csharp_build_inputs.CSharpBuildFailure) as error:
        print(error.code, file=sys.stderr)
        raise SystemExit(error.exit_code)
