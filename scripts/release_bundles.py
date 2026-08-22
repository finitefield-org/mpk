#!/usr/bin/env python3
"""Deterministic Go release-bundle assembler and installed-tree checker."""

from __future__ import annotations

import ctypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile


REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-0.1\0"
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
REGISTRY_ID = "mpk.release.registry.v0"
FRONTEND_ID = "frontend.go.go2vir.v0"
TOOLCHAIN_ID = "toolchain.go.go1.25.0.linux-amd64.v0"
HOST_PROFILE_ID = "mpk.host.linux-x86_64-gnu.v0"
GO_RELEASE = "go1.25.0"
GO_IMAGE = (
    "docker.io/library/golang@"
    "sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a"
)
FRONTEND_VERSION = "go1.25.0-profile-v0"
REQUIRED_PRIMITIVES = [
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
]


class BundleFailure(Exception):
    def __init__(self, code: str, exit_code: int = 65) -> None:
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


def typed_hash(domain: bytes, value: object) -> str:
    return hashlib.sha256(domain + canonical(value)).hexdigest()


def raw_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parent.parent


def docker_path() -> str:
    for candidate in ("/usr/local/bin/docker", "/usr/bin/docker"):
        if Path(candidate).is_file() and os.access(candidate, os.X_OK):
            return candidate
    raise BundleFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)


def run_checked(argv: list[str]) -> subprocess.CompletedProcess[bytes]:
    try:
        result = subprocess.run(argv, stdin=subprocess.DEVNULL, capture_output=True, check=False)
    except OSError as error:
        raise BundleFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69) from error
    if result.returncode != 0:
        raise BundleFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
    return result


def build_release_outputs(output: Path) -> None:
    root = repository_root()
    output.mkdir(mode=0o700)
    build_script = r'''
umask 077
test "$(sed -n '1p' /usr/local/go/VERSION)" = "go1.25.0"
mkdir -p /out/frontend/bin /out/toolchain/go /tmp/mpk-home /tmp/mpk-cache
cd /src/go-tools/go2vir
env -i \
  PATH=/usr/local/go/bin:/usr/bin:/bin \
  HOME=/tmp/mpk-home \
  GOCACHE=/tmp/mpk-cache \
  CGO_ENABLED=0 GOOS=linux GOARCH=amd64 GOAMD64=v1 \
  GOENV=off GOFLAGS= GOWORK=off GOPROXY=off GOSUMDB=off GOTOOLCHAIN=local \
  TZ=UTC LANG=C LC_ALL=C SOURCE_DATE_EPOCH=0 \
  /usr/local/go/bin/go build -mod=vendor -trimpath -buildvcs=false \
    -ldflags=-buildid= -o /out/frontend/bin/go2vir .
cp -a /usr/local/go/. /out/toolchain/go/
'''
    run_checked(
        [
            docker_path(),
            "run",
            "--rm",
            "--pull=never",
            "--network=none",
            "--platform=linux/amd64",
            "--read-only",
            "--tmpfs=/tmp:rw,nosuid,nodev,noexec,mode=1777",
            f"--mount=type=bind,src={root},dst=/src,readonly",
            f"--mount=type=bind,src={output.resolve()},dst=/out",
            GO_IMAGE,
            "/bin/sh",
            "-ceu",
            build_script,
        ]
    )
    prune_go_distribution(output / "toolchain/go")
    prune_empty_directories(output / "toolchain")
    normalize_tree(output / "frontend", frontend=True)
    normalize_tree(output / "toolchain", frontend=False)


def prune_go_distribution(go_root: Path) -> None:
    """Remove release tests that are not part of the executable toolchain closure."""
    test_root = go_root / "test"
    if test_root.is_dir() and not test_root.is_symlink():
        shutil.rmtree(test_root)
    for path in sorted(go_root.rglob("testdata"), reverse=True):
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
    for path in go_root.rglob("*_test.go"):
        if path.is_file() and not path.is_symlink():
            path.unlink()


def prune_empty_directories(root: Path) -> None:
    directories = sorted(
        (path for path in root.rglob("*") if path.is_dir() and not path.is_symlink()),
        key=lambda path: len(path.parts),
        reverse=True,
    )
    for path in directories:
        if not any(path.iterdir()):
            path.rmdir()


def normalize_tree(root: Path, *, frontend: bool) -> None:
    if not root.is_dir() or root.is_symlink():
        raise BundleFailure("BUNDLE_ASSEMBLER_IO", 74)
    executable_paths = {"bin/go2vir"} if frontend else {"go/bin/go", "go/bin/gofmt"}
    if not frontend:
        tool_root = root / "go/pkg/tool/linux_amd64"
        if not tool_root.is_dir():
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        executable_paths.update(
            path.relative_to(root).as_posix()
            for path in tool_root.iterdir()
            if path.is_file() and not path.is_symlink()
        )
    for path in sorted(root.rglob("*"), key=lambda item: item.as_posix().encode("utf-8")):
        metadata = path.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not (
            stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode)
        ):
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        relative = path.relative_to(root).as_posix()
        if stat.S_ISDIR(metadata.st_mode):
            path.chmod(0o555)
        else:
            path.chmod(0o555 if relative in executable_paths else 0o444)
    root.chmod(0o555)


def inventory(root: Path) -> list[dict[str, object]]:
    files: list[dict[str, object]] = []
    folded: set[str] = set()
    observed_directories: set[str] = set()
    for path in sorted(root.rglob("*"), key=lambda item: item.relative_to(root).as_posix().encode("utf-8")):
        metadata = path.lstat()
        relative = path.relative_to(root).as_posix()
        if stat.S_ISLNK(metadata.st_mode):
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        if stat.S_ISDIR(metadata.st_mode):
            if stat.S_IMODE(metadata.st_mode) != 0o555:
                raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
            observed_directories.add(relative)
            continue
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        mode = stat.S_IMODE(metadata.st_mode)
        if mode not in (0o444, 0o555):
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        case_key = relative.lower()
        if case_key in folded:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        folded.add(case_key)
        files.append(
            {
                "path": relative,
                "executable": mode == 0o555,
                "size_bytes": metadata.st_size,
                "sha256": raw_hash(path),
            }
        )
    implied_directories = {
        parent.as_posix()
        for item in files
        for parent in Path(str(item["path"])).parents
        if parent != Path(".")
    }
    if observed_directories != implied_directories:
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    return files


def require_static_x86_64_elf(path: Path) -> None:
    data = path.read_bytes()
    if len(data) < 64 or data[:4] != b"\x7fELF" or data[4:7] != b"\x02\x01\x01":
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    if int.from_bytes(data[18:20], "little") != 62:
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    program_offset = int.from_bytes(data[32:40], "little")
    entry_size = int.from_bytes(data[54:56], "little")
    entry_count = int.from_bytes(data[56:58], "little")
    if entry_size < 56 or program_offset + entry_size * entry_count > len(data):
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    for index in range(entry_count):
        offset = program_offset + index * entry_size
        if int.from_bytes(data[offset : offset + 4], "little") == 3:  # PT_INTERP
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")


def make_inventory(scope: dict[str, str], files: list[dict[str, object]]) -> dict[str, object]:
    return {
        "schema": "mpk.release.bundle_inventory.v0",
        "scope": scope,
        "files": files,
    }


def assemble_registry(outputs: Path) -> bytes:
    frontend_root = outputs / "frontend"
    toolchain_root = outputs / "toolchain"
    frontend_files = inventory(frontend_root)
    toolchain_files = inventory(toolchain_root)
    if len(frontend_files) != 1 or frontend_files[0]["path"] != "bin/go2vir":
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    for item in frontend_files + toolchain_files:
        if item["executable"]:
            base = frontend_root if item in frontend_files else toolchain_root
            require_static_x86_64_elf(base / str(item["path"]))

    frontend_inventory = make_inventory(
        {"kind": "frontend_bundle", "bundle_id": FRONTEND_ID}, frontend_files
    )
    toolchain_inventory = make_inventory(
        {"kind": "toolchain_bundle", "bundle_id": TOOLCHAIN_ID}, toolchain_files
    )
    content_files = [item for item in toolchain_files if not item["executable"]]
    content_inventory = make_inventory(
        {"kind": "component", "bundle_id": TOOLCHAIN_ID, "component_name": "go-target-linux-amd64"},
        content_files,
    )
    content_digest = typed_hash(CONTENT_DOMAIN, content_inventory)
    components: list[dict[str, object]] = [
        {
            "kind": "content",
            "name": "go-target-linux-amd64",
            "release": GO_RELEASE,
            "inventory": content_inventory,
            "content_sha256": content_digest,
        }
    ]
    for item in toolchain_files:
        if not item["executable"]:
            continue
        relative = str(item["path"])
        if relative == "go/bin/go":
            name = "go"
        elif relative == "go/bin/gofmt":
            name = "gofmt"
        else:
            name = "go-tool-" + Path(relative).name
        components.append(
            {
                "kind": "executable",
                "name": name,
                "release": GO_RELEASE,
                "path": relative,
                "binary_sha256": item["sha256"],
                "runtime": {"kind": "static"},
            }
        )
    components.sort(key=lambda item: str(item["name"]).encode("utf-8"))

    registry: dict[str, object] = {
        "schema": "mpk.release.bundle_registry.v0",
        "id": REGISTRY_ID,
        "execution_host_profiles": [
            {
                "id": HOST_PROFILE_ID,
                "os": "linux",
                "architecture": "x86_64",
                "abi": "gnu",
                "minimum_kernel_abi": "5.10.0",
                "probe_profile_id": "mpk.release.probe.linux_namespaces.v0",
                "required_primitives": REQUIRED_PRIMITIVES,
            }
        ],
        "native_runtime_layout_profiles": [],
        "frontend_bundles": [
            {
                "schema": "mpk.release.frontend_bundle.v0",
                "bundle_id": FRONTEND_ID,
                "source_language": "go",
                "name": "go2vir",
                "version": FRONTEND_VERSION,
                "limit_profile_id": "mpk.vir.limits.v0",
                "environment_profile_id": "mpk.go.frontend_environment.v0",
                "argument_profile_id": "mpk.go.frontend_arguments.v0",
                "main": {
                    "name": "go2vir",
                    "version": FRONTEND_VERSION,
                    "path": "bin/go2vir",
                    "binary_sha256": frontend_files[0]["sha256"],
                    "runtime": {"kind": "static"},
                },
                "subordinate_binaries": [],
                "inventory": frontend_inventory,
                "bundle_sha256": typed_hash(CONTENT_DOMAIN, frontend_inventory),
            }
        ],
        "toolchain_bundles": [
            {
                "schema": "mpk.release.toolchain_bundle.v0",
                "bundle_id": TOOLCHAIN_ID,
                "source_language": "go",
                "compiler": {"kind": "go", "release": GO_RELEASE},
                "execution_host_profile_id": HOST_PROFILE_ID,
                "native_runtime": {"kind": "none"},
                "components": components,
                "target_libraries": [
                    {
                        "target_id": "linux/amd64",
                        "pointer_width": 64,
                        "component_name": "go-target-linux-amd64",
                        "content_sha256": content_digest,
                    }
                ],
                "inventory": toolchain_inventory,
                "distribution_sha256": typed_hash(CONTENT_DOMAIN, toolchain_inventory),
            }
        ],
        "tuples": [
            {
                "source_language": "go",
                "semantic_profile": "mpk.go.fixed.v0",
                "target_id": "linux/amd64",
                "pointer_width": 64,
                "limit_profile_id": "mpk.vir.limits.v0",
                "frontend_bundle_id": FRONTEND_ID,
                "toolchain_bundle_id": TOOLCHAIN_ID,
            }
        ],
    }
    registry["registry_sha256"] = typed_hash(REGISTRY_DOMAIN, registry)
    return canonical(registry) + b"\n"


def current_registry() -> bytes:
    path = repository_root() / "release/bundles/bundle-registry.json"
    try:
        return path.read_bytes()
    except OSError as error:
        raise BundleFailure("BUNDLE_REGISTERED_STATE") from error


def classify_registry(data: bytes) -> str:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise BundleFailure("BUNDLE_REGISTERED_STATE") from error
    if canonical(value) + b"\n" != data or value.get("id") != REGISTRY_ID:
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    claimed = value.get("registry_sha256")
    payload = dict(value)
    payload.pop("registry_sha256", None)
    if claimed != typed_hash(REGISTRY_DOMAIN, payload):
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    arrays = [
        value.get("execution_host_profiles"),
        value.get("native_runtime_layout_profiles"),
        value.get("frontend_bundles"),
        value.get("toolchain_bundles"),
        value.get("tuples"),
    ]
    if arrays == [[], [], [], [], []]:
        return "bootstrap"
    tuples = value.get("tuples")
    if isinstance(tuples, list) and tuples and all(item.get("source_language") == "go" for item in tuples):
        return "go_registered"
    raise BundleFailure("BUNDLE_REGISTERED_STATE")


def current_rust_candidate(active: Path) -> bytes | None:
    candidates = active / "candidates"
    if not candidates.exists() and not candidates.is_symlink():
        return None
    if candidates.is_symlink() or not candidates.is_dir():
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    if {path.name for path in candidates.iterdir()} != {"rust"}:
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    rust = candidates / "rust"
    if rust.is_symlink() or not rust.is_dir():
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    if {path.name for path in rust.iterdir()} != {"candidate.json"}:
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    candidate = rust / "candidate.json"
    metadata = candidate.lstat()
    if (
        candidate.is_symlink()
        or not stat.S_ISREG(metadata.st_mode)
        or metadata.st_nlink != 1
    ):
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    return candidate.read_bytes()


def atomic_publish_registry(registry: bytes) -> None:
    root = repository_root()
    active = root / "release/bundles"
    parent = active.parent
    candidate = current_rust_candidate(active)
    staging = Path(tempfile.mkdtemp(prefix=".bundles-stage-", dir=parent))
    try:
        shutil.copyfile(active / "README.md", staging / "README.md", follow_symlinks=False)
        (staging / "bundle-registry.json").write_bytes(registry)
        if candidate is not None:
            directory = staging / "candidates/rust"
            directory.mkdir(parents=True)
            candidate_path = directory / "candidate.json"
            candidate_path.write_bytes(candidate)
            candidate_path.chmod(0o644)
        for path in (staging / "README.md", staging / "bundle-registry.json"):
            path.chmod(0o644)
        if current_rust_candidate(active) != candidate:
            raise BundleFailure("BUNDLE_REGISTERED_STATE")
        exchange_directories(staging, active)
        shutil.rmtree(staging)
    except BundleFailure:
        raise
    except OSError as error:
        raise BundleFailure("BUNDLE_ASSEMBLER_IO", 74) from error
    finally:
        if staging.exists():
            shutil.rmtree(staging, ignore_errors=True)


def exchange_directories(left: Path, right: Path) -> None:
    library = ctypes.CDLL(None, use_errno=True)
    if sys.platform == "darwin":
        rename = library.renamex_np
        rename.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint]
        result = rename(os.fsencode(left), os.fsencode(right), 0x00000002)
    elif sys.platform.startswith("linux"):
        rename = library.renameat2
        rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
        result = rename(-100, os.fsencode(left), -100, os.fsencode(right), 0x00000002)
    else:
        raise BundleFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)
    if result != 0:
        raise BundleFailure("BUNDLE_PUBLICATION_UNAVAILABLE", 69)


def build_expected() -> tuple[bytes, Path, tempfile.TemporaryDirectory[str]]:
    temporary = tempfile.TemporaryDirectory(prefix="mpk-release-bundles-")
    output = Path(temporary.name) / "output"
    build_release_outputs(output)
    return assemble_registry(output), output, temporary


def update_go() -> None:
    state = classify_registry(current_registry())
    if state not in ("bootstrap", "go_registered"):
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    registry, _, temporary = build_expected()
    try:
        atomic_publish_registry(registry)
    finally:
        temporary.cleanup()


def check_go() -> None:
    if classify_registry(current_registry()) != "go_registered":
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    registry, _, temporary = build_expected()
    try:
        if registry != current_registry():
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    finally:
        temporary.cleanup()


def validate_generated_tree(registry_data: bytes, output: Path) -> None:
    if classify_registry(registry_data) != "go_registered":
        raise BundleFailure("BUNDLE_REGISTERED_STATE")
    rebuilt = assemble_registry(output)
    if rebuilt != registry_data:
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")


def assemble_installed_fixture(registry_data: bytes, output: Path, destination: Path) -> None:
    registry = json.loads(registry_data)
    (destination / "bin").mkdir(parents=True)
    (destination / "share/mpk").mkdir(parents=True)
    bundles = destination / "libexec/mpk/bundles"
    bundles.mkdir(parents=True)
    (destination / "bin/mpk").write_bytes(b"mpk-installed-fixture-v0\n")
    (destination / "share/mpk/bundle-registry.json").write_bytes(registry_data)
    shutil.copytree(
        output / "frontend",
        bundles / FRONTEND_ID,
        copy_function=shutil.copy2,
    )
    shutil.copytree(
        output / "toolchain",
        bundles / TOOLCHAIN_ID,
        copy_function=shutil.copy2,
    )
    (destination / "bin/mpk").chmod(0o555)
    (destination / "share/mpk/bundle-registry.json").chmod(0o444)
    for path in sorted(destination.rglob("*"), reverse=True):
        if path.is_dir() and not path.is_symlink():
            path.chmod(0o555)
    destination.chmod(0o555)
    validate_installed_fixture(registry, registry_data, destination)


def validate_installed_fixture(
    registry: dict[str, object], registry_data: bytes, root: Path
) -> None:
    exact_directories = {
        ".": {"bin", "libexec", "share"},
        "bin": {"mpk"},
        "share": {"mpk"},
        "share/mpk": {"bundle-registry.json"},
        "libexec": {"mpk"},
        "libexec/mpk": {"bundles"},
        "libexec/mpk/bundles": {FRONTEND_ID, TOOLCHAIN_ID},
    }
    for relative, expected in exact_directories.items():
        directory = root if relative == "." else root / relative
        metadata = directory.lstat()
        if not stat.S_ISDIR(metadata.st_mode) or stat.S_IMODE(metadata.st_mode) != 0o555:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        if {path.name for path in directory.iterdir()} != expected:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    registry_path = root / "share/mpk/bundle-registry.json"
    registry_metadata = registry_path.lstat()
    if (
        not stat.S_ISREG(registry_metadata.st_mode)
        or registry_metadata.st_nlink != 1
        or stat.S_IMODE(registry_metadata.st_mode) != 0o444
        or registry_path.read_bytes() != registry_data
    ):
        raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    described = {
        bundle["bundle_id"]: bundle["inventory"]["files"]
        for key in ("frontend_bundles", "toolchain_bundles")
        for bundle in registry[key]
    }
    for bundle_id, files in described.items():
        observed = inventory(root / "libexec/mpk/bundles" / bundle_id)
        if observed != files:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")


def go_frontend_environment() -> list[str]:
    values = {
        "CGO_ENABLED": "0",
        "GO111MODULE": "on",
        "GOAMD64": "v1",
        "GOARCH": "amd64",
        "GOCACHE": "/mpk/cache/go-build",
        "GODEBUG": "",
        "GOENV": "off",
        "GOEXPERIMENT": "",
        "GOFLAGS": "",
        "GOMAXPROCS": "1",
        "GOMODCACHE": "/mpk/cache/go-mod",
        "GONOPROXY": "",
        "GONOSUMDB": "",
        "GOOS": "linux",
        "GOPACKAGESDRIVER": "off",
        "GOPATH": "/mpk/gopath",
        "GOPRIVATE": "",
        "GOPROXY": "off",
        "GOROOT": "/mpk/toolchain/go",
        "GOSUMDB": "off",
        "GOTELEMETRY": "off",
        "GOTOOLCHAIN": "local",
        "GOVCS": "*:off",
        "GOWORK": "off",
        "HOME": "/mpk/empty/home",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": "/mpk/toolchain/go/bin",
        "TMPDIR": "/mpk/tmp",
        "TZ": "UTC",
    }
    return [f"{key}={values[key]}" for key in sorted(values)]


def prepare_frontend_fixture_inputs(temporary: Path) -> tuple[Path, Path]:
    root = repository_root()
    source = temporary / "source"
    shutil.copytree(root / "go-tools/go2vir/testdata/preflight/valid", source)
    empty = temporary / "empty"
    for relative in ["cache/go-mod", "gopath", "home", "native-runtime"]:
        (empty / relative).mkdir(parents=True)
    for tree in [source, empty]:
        for path in sorted(tree.rglob("*"), reverse=True):
            metadata = path.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
            path.chmod(0o555 if stat.S_ISDIR(metadata.st_mode) else 0o444)
        tree.chmod(0o555)
    return source, empty


def fixture_go() -> None:
    registry, output, temporary = build_expected()
    try:
        if registry != current_registry():
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        validate_generated_tree(registry, output)
        snapshot = Path(temporary.name) / "installed"
        assemble_installed_fixture(registry, output, snapshot)
        original = snapshot / f"libexec/mpk/bundles/{FRONTEND_ID}/bin/go2vir"
        expected_digest = raw_hash(original)
        source = output / "frontend/bin/go2vir"
        source.chmod(0o644)
        source.write_bytes(b"changed-after-validation")
        if raw_hash(original) != expected_digest:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        registry_value = json.loads(registry)
        frontend = registry_value["frontend_bundles"][0]
        toolchain = registry_value["toolchain_bundles"][0]
        source_root, empty_root = prepare_frontend_fixture_inputs(Path(temporary.name))
        common_mounts = [
            f"--mount=type=bind,src={snapshot / f'libexec/mpk/bundles/{FRONTEND_ID}'},dst=/mpk/frontend,readonly",
            f"--mount=type=bind,src={snapshot / f'libexec/mpk/bundles/{TOOLCHAIN_ID}'},dst=/mpk/toolchain,readonly",
            f"--mount=type=bind,src={source_root},dst=/mpk/source,readonly",
            f"--mount=type=bind,src={empty_root / 'cache/go-mod'},dst=/mpk/cache/go-mod,readonly",
            f"--mount=type=bind,src={empty_root / 'gopath'},dst=/mpk/gopath,readonly",
            f"--mount=type=bind,src={empty_root / 'home'},dst=/mpk/empty/home,readonly",
            f"--mount=type=bind,src={empty_root / 'native-runtime'},dst=/mpk/native-runtime,readonly",
        ]
        result = run_checked(
            [
                docker_path(),
                "run",
                "--rm",
                "--pull=never",
                "--network=none",
                "--platform=linux/amd64",
                "--read-only",
                "--tmpfs=/tmp:rw,nosuid,nodev,noexec,mode=1777",
                "--tmpfs=/mpk/cache/go-build:rw,nosuid,nodev,noexec,mode=700",
                "--tmpfs=/mpk/tmp:rw,nosuid,nodev,noexec,mode=700",
                *common_mounts,
                GO_IMAGE,
                "/usr/bin/env",
                "-i",
                *go_frontend_environment(),
                "/mpk/frontend/bin/go2vir",
                "lower",
                "/mpk/source",
                "--package",
                "example.com/mpk/vector",
                "--semantic-profile",
                "mpk.go.fixed.v0",
                "--target",
                "linux/amd64",
                "--function",
                "example.com/mpk/vector.Identity",
                "--frontend-bundle-id",
                FRONTEND_ID,
                "--frontend-sha256",
                frontend["main"]["binary_sha256"],
                "--release-registry-id",
                registry_value["id"],
                "--release-registry-sha256",
                registry_value["registry_sha256"],
                "--toolchain-bundle-id",
                TOOLCHAIN_ID,
                "--toolchain-root",
                "/mpk/toolchain",
                "--toolchain-distribution-sha256",
                toolchain["distribution_sha256"],
                "--contract",
                "identity_contract.json",
            ]
        )
        try:
            envelope = json.loads(result.stdout)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH") from error
        if (
            result.stderr
            or canonical(envelope) + b"\n" != result.stdout
            or envelope.get("status") != "ir-lowered"
            or envelope.get("source_language") != "go"
        ):
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
        version = run_checked(
            [
                docker_path(),
                "run",
                "--rm",
                "--pull=never",
                "--network=none",
                "--platform=linux/amd64",
                "--read-only",
                f"--mount=type=bind,src={snapshot / f'libexec/mpk/bundles/{TOOLCHAIN_ID}'},dst=/mpk/toolchain,readonly",
                GO_IMAGE,
                "/mpk/toolchain/go/bin/go",
                "version",
            ]
        )
        if version.stderr or version.stdout != b"go version go1.25.0 linux/amd64\n":
            raise BundleFailure("BUNDLE_REPRODUCIBILITY_MISMATCH")
    finally:
        temporary.cleanup()


def main(argv: list[str]) -> int:
    try:
        if argv == ["update-go"]:
            update_go()
        elif argv == ["check-go"]:
            check_go()
        elif argv == ["fixture-go"]:
            fixture_go()
        else:
            raise BundleFailure("BUNDLE_ASSEMBLER_USAGE", 64)
        return 0
    except BundleFailure as error:
        sys.stderr.write(error.code + "\n")
        return error.exit_code
    except (OSError, ValueError, KeyError, TypeError):
        sys.stderr.write("BUNDLE_ASSEMBLER_IO\n")
        return 74


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
