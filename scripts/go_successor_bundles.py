#!/usr/bin/env python3
"""Build and verify the staging-only successor Go bundle candidate."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


GO_IMAGE = (
    "docker.io/library/golang@"
    "sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a"
)
GO_RELEASE = "go1.25.0"
FRONTEND_ID = "frontend.go.go2vir.candidate.v1"
FRONTEND_VERSION = "go1.25.0-profile-v1-staging"
TOOLCHAIN_ID = "toolchain.go.go1.25.0.linux-amd64.candidate.v1"
HOST_PROFILE_ID = "mpk.host.linux-x86_64-gnu.v0"
ACTIVE_REGISTRY_TRANSPORT_SHA256 = (
    "56fa7b398bd63fe2cac488063da00de151279cf65ba0d9cbd3c4fc74716c578a"
)
ACTIVE_REGISTRY_SHA256 = (
    "bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98"
)
PROFILE_ENTRY_SHA256 = "b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96"
PROFILE_REGISTRY = {
    "schema": "mpk.semantic_profile.registry.v1",
    "id": "mpk.semantic_profile.registry.v1",
    "revision": 2,
    "registry_sha256": "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75",
}
SEMANTIC_CONTEXT = {
    "profile_registry": PROFILE_REGISTRY,
    "profile_entry_sha256": PROFILE_ENTRY_SHA256,
    "source_language": "go",
    "semantic_profile": "mpk.go.fixed.v0",
    "semantic_parameters": {
        "schema": "mpk.semantic_parameters.go_fixed.v0",
        "value": {"target_id": "linux/amd64", "pointer_width": 64},
    },
}
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"


class StagingFailure(Exception):
    pass


def repository_root() -> Path:
    return Path(__file__).resolve().parent.parent


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


def raw_hash(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_overlay_module():
    path = repository_root() / "go-tools/go2vir/successor_overlay.py"
    specification = importlib.util.spec_from_file_location("mpk_go_successor_overlay", path)
    if specification is None or specification.loader is None:
        raise StagingFailure("GO_SUCCESSOR_OVERLAY_UNAVAILABLE")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def docker_path() -> str:
    for candidate in ("/usr/local/bin/docker", "/usr/bin/docker"):
        if Path(candidate).is_file() and os.access(candidate, os.X_OK):
            return candidate
    raise StagingFailure("GO_SUCCESSOR_BUILD_UNAVAILABLE")


def run_checked(argv: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> bytes:
    try:
        result = subprocess.run(
            argv,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise StagingFailure("GO_SUCCESSOR_BUILD_UNAVAILABLE") from error
    if result.returncode != 0:
        raise StagingFailure("GO_SUCCESSOR_BUILD_FAILED")
    return result.stdout


def require_static_x86_64_elf(data: bytes) -> None:
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise StagingFailure("GO_SUCCESSOR_BINARY_IDENTITY")
    if int.from_bytes(data[18:20], "little") != 62:
        raise StagingFailure("GO_SUCCESSOR_BINARY_IDENTITY")
    program_offset = int.from_bytes(data[32:40], "little")
    entry_size = int.from_bytes(data[54:56], "little")
    entry_count = int.from_bytes(data[56:58], "little")
    if entry_size < 56 or program_offset + entry_size * entry_count > len(data):
        raise StagingFailure("GO_SUCCESSOR_BINARY_IDENTITY")
    for index in range(entry_count):
        offset = program_offset + index * entry_size
        if int.from_bytes(data[offset : offset + 4], "little") == 3:
            raise StagingFailure("GO_SUCCESSOR_BINARY_IDENTITY")


def build_candidate_binary(destination: Path) -> bytes:
    root = repository_root()
    overlay_host = destination / "overlay"
    overlay = load_overlay_module()
    overlay.materialize(root, overlay_host, Path("/src"), Path("/overlay"))
    output = destination / "out"
    output.mkdir(mode=0o700)
    cache = destination / "cache"
    cache.mkdir(mode=0o700)
    script = rf'''
umask 077
test "$(sed -n '1p' /usr/local/go/VERSION)" = "{GO_RELEASE}"
mkdir -p /out/home
cd /src/go-tools/go2vir
env -i \
  PATH=/usr/local/go/bin:/usr/bin:/bin \
  HOME=/out/home GOCACHE=/cache \
  CGO_ENABLED=0 GOOS=linux GOARCH=amd64 GOAMD64=v1 \
  GOENV=off GOFLAGS= GOWORK=off GOPROXY=off GOSUMDB=off GOTOOLCHAIN=local \
  TZ=UTC LANG=C LC_ALL=C SOURCE_DATE_EPOCH=0 \
  /usr/local/go/bin/go build -mod=vendor -trimpath -buildvcs=false \
    -overlay=/overlay/overlay.json -ldflags=-buildid= -o /out/go2vir .
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
            f"--mount=type=bind,src={overlay_host},dst=/overlay,readonly",
            f"--mount=type=bind,src={output},dst=/out",
            f"--mount=type=bind,src={cache},dst=/cache",
            GO_IMAGE,
            "/bin/sh",
            "-ceu",
            script,
        ]
    )
    data = (output / "go2vir").read_bytes()
    require_static_x86_64_elf(data)
    return data


def checked_active_registry() -> dict[str, object]:
    path = repository_root() / "release/bundles/bundle-registry.json"
    raw = path.read_bytes()
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingFailure("GO_SUCCESSOR_ACTIVE_REGISTRY") from error
    if canonical(value) + b"\n" != raw:
        raise StagingFailure("GO_SUCCESSOR_ACTIVE_REGISTRY")
    if (
        raw_hash(raw) != ACTIVE_REGISTRY_TRANSPORT_SHA256
        or value.get("schema") != "mpk.release.bundle_registry.v0"
        or value.get("id") != "mpk.release.registry.v0"
        or value.get("registry_sha256") != ACTIVE_REGISTRY_SHA256
    ):
        raise StagingFailure("GO_SUCCESSOR_ACTIVE_REGISTRY")
    return value


def profile_envelope(contract: str, value: dict[str, object]) -> dict[str, object]:
    return {
        "profile_entry_sha256": PROFILE_ENTRY_SHA256,
        "contract_id": contract,
        "value": value,
    }


def assemble_candidate(binary: bytes) -> tuple[bytes, bytes]:
    active = checked_active_registry()
    frontends = [
        item for item in active["frontend_bundles"]
        if item.get("source_language") == "go" and item.get("name") == "go2vir"
    ]
    toolchains = [
        item for item in active["toolchain_bundles"]
        if item.get("source_language") == "go"
    ]
    hosts = [
        item for item in active["execution_host_profiles"]
        if item.get("id") == HOST_PROFILE_ID
    ]
    if len(frontends) != 1 or len(toolchains) != 1 or len(hosts) != 1:
        raise StagingFailure("GO_SUCCESSOR_ACTIVE_REGISTRY")
    frontend = frontends[0]
    toolchain = toolchains[0]
    host = hosts[0]

    binary_sha256 = raw_hash(binary)
    frontend_inventory = {
        "schema": "mpk.release.bundle_inventory.v0",
        "scope": {"kind": "frontend_bundle", "bundle_id": FRONTEND_ID},
        "files": [{
            "path": "bin/go2vir",
            "executable": True,
            "size_bytes": len(binary),
            "sha256": binary_sha256,
        }],
    }
    successor_frontend = {
        "schema": "mpk.release.frontend_bundle.v1",
        "bundle_id": FRONTEND_ID,
        "name": "go2vir",
        "version": FRONTEND_VERSION,
        "profile_contracts": [profile_envelope(
            "mpk.profile.frontend.go_fixed.v0",
            {
                "limit_profile_id": "mpk.vir.limits.v0",
                "environment_profile_id": "mpk.go.frontend_environment.v0",
                "argument_profile_id": "mpk.go.frontend_arguments.v0",
            },
        )],
        "main": {
            "name": "go2vir",
            "version": FRONTEND_VERSION,
            "path": "bin/go2vir",
            "binary_sha256": binary_sha256,
            "runtime": {"kind": "static"},
        },
        "subordinate_binaries": [],
        "inventory": frontend_inventory,
        "bundle_sha256": typed_hash(CONTENT_DOMAIN, frontend_inventory),
    }

    toolchain_inventory = json.loads(json.dumps(toolchain["inventory"]))
    toolchain_inventory["scope"]["bundle_id"] = TOOLCHAIN_ID
    components = json.loads(json.dumps(toolchain["components"]))
    content_hashes: dict[str, str] = {}
    for component in components:
        if component["kind"] != "content":
            continue
        component["inventory"]["scope"]["bundle_id"] = TOOLCHAIN_ID
        component["content_sha256"] = typed_hash(CONTENT_DOMAIN, component["inventory"])
        content_hashes[component["name"]] = component["content_sha256"]
    target_libraries = json.loads(json.dumps(toolchain["target_libraries"]))
    for target in target_libraries:
        target["content_sha256"] = content_hashes[target["component_name"]]
    release_value = {
        "compiler": toolchain["compiler"],
        "execution_host_profile_id": toolchain["execution_host_profile_id"],
        "native_runtime": toolchain["native_runtime"],
        "target_libraries": target_libraries,
    }
    successor_toolchain = {
        "schema": "mpk.release.toolchain_bundle.v1",
        "bundle_id": TOOLCHAIN_ID,
        "execution_host_profile_id": toolchain["execution_host_profile_id"],
        "profile_contracts": [profile_envelope(
            "mpk.profile.release.go_fixed.v0", release_value
        )],
        "components": components,
        "inventory": toolchain_inventory,
        "distribution_sha256": typed_hash(CONTENT_DOMAIN, toolchain_inventory),
    }
    candidate: dict[str, object] = {
        "schema": "mpk.release.bundle_candidate.v1",
        "profile_registry": PROFILE_REGISTRY,
        "execution_host_profiles": [host],
        "native_runtime_layout_profiles": [],
        "frontend_bundles": [successor_frontend],
        "toolchain_bundles": [successor_toolchain],
        "tuples": [{
            "semantic_context": SEMANTIC_CONTEXT,
            "limit_profile_id": "mpk.vir.limits.v0",
            "frontend_bundle_id": FRONTEND_ID,
            "toolchain_bundle_id": TOOLCHAIN_ID,
        }],
    }
    registry = dict(candidate)
    registry["schema"] = "mpk.release.bundle_registry.v1"
    registry["id"] = "mpk.release.registry.v1"
    registry["registry_sha256"] = typed_hash(REGISTRY_DOMAIN, registry)
    return canonical(candidate) + b"\n", canonical(registry) + b"\n"


def staged_paths() -> tuple[Path, Path]:
    root = repository_root() / "develop/migrations/csharp-02-staging"
    return root / "go-bundle-candidate.json", root / "go-bundle-registry.json"


def write_or_check(path: Path, content: bytes, *, update: bool) -> None:
    if update:
        path.write_bytes(content)
        return
    try:
        current = path.read_bytes()
    except OSError as error:
        raise StagingFailure("GO_SUCCESSOR_STAGING_MISSING") from error
    if current != content:
        raise StagingFailure("GO_SUCCESSOR_STAGING_STALE")


def build_and_compare(*, update: bool) -> None:
    with tempfile.TemporaryDirectory(prefix="mpk-go-successor-a-") as first_dir, tempfile.TemporaryDirectory(prefix="mpk-go-successor-b-") as second_dir:
        first = build_candidate_binary(Path(first_dir))
        second = build_candidate_binary(Path(second_dir))
    if first != second:
        raise StagingFailure("GO_SUCCESSOR_BUILD_NONDETERMINISTIC")
    candidate, registry = assemble_candidate(first)
    candidate_path, registry_path = staged_paths()
    write_or_check(candidate_path, candidate, update=update)
    write_or_check(registry_path, registry, update=update)


def check_fixtures(*, update: bool) -> None:
    root = repository_root()
    with tempfile.TemporaryDirectory(prefix="mpk-go-successor-overlay-") as directory:
        overlay_root = Path(directory) / "overlay"
        load_overlay_module().materialize(
            root,
            overlay_root,
            Path("/src"),
            Path("/overlay"),
        )
        cache = Path(directory) / "cache"
        home = Path(directory) / "home"
        cache.mkdir(mode=0o700)
        home.mkdir(mode=0o700)
        repository_mount = f"--mount=type=bind,src={root},dst=/src"
        if not update:
            repository_mount += ",readonly"
        update_environment = (
            "MPK_UPDATE_SUCCESSOR_GO_CORPUS=1 " if update else ""
        )
        script = rf'''
umask 077
test "$(sed -n '1p' /usr/local/go/VERSION)" = "{GO_RELEASE}"
mkdir -p /cache/tmp
cd /src/go-tools/go2vir
env -i \
  PATH=/usr/local/go/bin:/usr/bin:/bin \
  HOME=/home GOCACHE=/cache GOTMPDIR=/cache/tmp \
  CGO_ENABLED=0 GOOS=linux GOARCH=amd64 GOAMD64=v1 \
  GOENV=off GOFLAGS= GOWORK=off GOPROXY=off GOSUMDB=off GOTOOLCHAIN=local \
  TZ=UTC LANG=C LC_ALL=C SOURCE_DATE_EPOCH=0 \
  {update_environment}/usr/local/go/bin/go test -mod=vendor -count=1 \
    -tags=mpk_successor -overlay=/overlay/overlay.json \
    -run=^TestSuccessorGoMigrationCorpus$ .
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
                repository_mount,
                f"--mount=type=bind,src={overlay_root},dst=/overlay,readonly",
                f"--mount=type=bind,src={home},dst=/home",
                f"--mount=type=bind,src={cache},dst=/cache",
                GO_IMAGE,
                "/bin/sh",
                "-ceu",
                script,
            ],
        )


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {
        "check", "update", "check-fixtures", "update-fixtures"
    }:
        print("GO_SUCCESSOR_USAGE", file=sys.stderr)
        return 64
    try:
        action = sys.argv[1]
        if action in {"check", "update"}:
            build_and_compare(update=action == "update")
        else:
            check_fixtures(update=action == "update-fixtures")
    except (OSError, KeyError, StopIteration, TypeError, ValueError, StagingFailure):
        print("GO_SUCCESSOR_FAILED", file=sys.stderr)
        return 65
    print("GO_SUCCESSOR_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
