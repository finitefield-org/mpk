#!/usr/bin/env python3
"""Build, verify, and install the sole successor release image."""

from __future__ import annotations

from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile

SCRIPT_ROOT = Path(__file__).resolve().parent
REPOSITORY_ROOT = SCRIPT_ROOT.parent
sys.path.insert(0, str(SCRIPT_ROOT))

import csharp_release_bundles as csharp  # noqa: E402
import release_bundles as go_release  # noqa: E402
import rust_build_inputs as rust  # noqa: E402


REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/bundle-registry.json"
SEMANTIC_REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/semantic-profile-registry.json"
CANDIDATE_PATHS = {
    language: REPOSITORY_ROOT / f"release/bundles/candidates/{language}.json"
    for language in ("go", "rust", "csharp")
}
REGISTRY_SHA256 = "00580f5ef519ae077432460d2e9e1214bb15b624b2781a96188dd81ad92f8fce"
SEMANTIC_REGISTRY_SHA256 = (
    "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75"
)
REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"
RUST_BUILD_ARGUMENTS = [
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


class SuccessorReleaseFailure(Exception):
    def __init__(self, code: str = "BUNDLE_REPRODUCIBILITY_MISMATCH", exit_code: int = 65):
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


def raw_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def read_canonical(path: Path) -> tuple[dict[str, object], bytes]:
    if path.is_symlink() or not path.is_file():
        raise SuccessorReleaseFailure()
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SuccessorReleaseFailure() from error
    if not isinstance(value, dict) or canonical_line(value) != data:
        raise SuccessorReleaseFailure()
    return value, data


def keyed_union(candidates: list[dict[str, object]], field: str, key: str) -> list[object]:
    values: dict[str, object] = {}
    for candidate in candidates:
        entries = candidate.get(field)
        if not isinstance(entries, list):
            raise SuccessorReleaseFailure()
        for entry in entries:
            if not isinstance(entry, dict) or not isinstance(entry.get(key), str):
                raise SuccessorReleaseFailure()
            identity = str(entry[key])
            previous = values.setdefault(identity, entry)
            if previous != entry:
                raise SuccessorReleaseFailure()
    return list(values.values())


def encoded_multiset(values: object) -> Counter[bytes]:
    if not isinstance(values, list):
        raise SuccessorReleaseFailure()
    return Counter(canonical(value) for value in values)


def validate_release_models() -> tuple[dict[str, object], dict[str, dict[str, object]]]:
    registry, registry_bytes = read_canonical(REGISTRY_PATH)
    semantic, semantic_bytes = read_canonical(SEMANTIC_REGISTRY_PATH)
    vector_path = REPOSITORY_ROOT / "develop/specs/vectors/semantic-profile-registry-v2.json"
    try:
        vectors = json.loads(vector_path.read_bytes())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SuccessorReleaseFailure() from error
    if (
        semantic != vectors.get("registry")
        or semantic.get("registry_sha256") != SEMANTIC_REGISTRY_SHA256
        or hashlib.sha256(semantic_bytes).hexdigest()
        != "d3ccae252f388c21fbb3c400b58454c45d28943ae7d681d385a1dd4c017c0952"
    ):
        raise SuccessorReleaseFailure()
    candidates = {language: read_canonical(path)[0] for language, path in CANDIDATE_PATHS.items()}
    candidate_values = list(candidates.values())
    if any(
        candidate.get("schema") != "mpk.release.bundle_candidate.v1"
        or candidate.get("profile_registry") != registry.get("profile_registry")
        for candidate in candidate_values
    ):
        raise SuccessorReleaseFailure()
    if (
        registry.get("schema") != "mpk.release.bundle_registry.v1"
        or registry.get("id") != "mpk.release.registry.v1"
        or registry.get("registry_sha256") != REGISTRY_SHA256
    ):
        raise SuccessorReleaseFailure()
    payload = dict(registry)
    payload.pop("registry_sha256", None)
    if hashlib.sha256(REGISTRY_DOMAIN + canonical(payload)).hexdigest() != REGISTRY_SHA256:
        raise SuccessorReleaseFailure()
    if hashlib.sha256(registry_bytes).hexdigest() != (
        "5c8e9f8a343c675a429f6cdb5299d08e6ed7a232e2d3a81d32e880091bb39253"
    ):
        raise SuccessorReleaseFailure()
    for field, key in (
        ("execution_host_profiles", "id"),
        ("native_runtime_layout_profiles", "id"),
        ("frontend_bundles", "bundle_id"),
        ("toolchain_bundles", "bundle_id"),
    ):
        if encoded_multiset(registry.get(field)) != encoded_multiset(
            keyed_union(candidate_values, field, key)
        ):
            raise SuccessorReleaseFailure()
    expected_tuples = [
        entry
        for candidate in candidate_values
        for entry in candidate.get("tuples", [])
    ]
    if encoded_multiset(registry.get("tuples")) != encoded_multiset(expected_tuples):
        raise SuccessorReleaseFailure()
    if (
        len(registry["frontend_bundles"]) != 3
        or len(registry["toolchain_bundles"]) != 3
        or len(registry["tuples"]) != 4
    ):
        raise SuccessorReleaseFailure()
    return registry, candidates


def copy_described(source: Path, destination: Path, item: dict[str, object]) -> None:
    metadata = source.lstat()
    if source.is_symlink() or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise SuccessorReleaseFailure()
    if metadata.st_size != item.get("size_bytes") or raw_hash(source) != item.get("sha256"):
        raise SuccessorReleaseFailure()
    destination.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    with source.open("rb") as input_stream, destination.open("xb") as output_stream:
        shutil.copyfileobj(input_stream, output_stream, 1024 * 1024)
    destination.chmod(0o555 if item.get("executable") is True else 0o444)


def normalize_directories(root: Path) -> None:
    for path in sorted((item for item in root.rglob("*") if item.is_dir()), reverse=True):
        if path.is_symlink():
            raise SuccessorReleaseFailure()
        path.chmod(0o555)
    root.chmod(0o555)


def inventory(root: Path) -> list[dict[str, object]]:
    files: list[dict[str, object]] = []
    for path in sorted(root.rglob("*"), key=lambda value: value.relative_to(root).as_posix().encode()):
        metadata = path.lstat()
        if path.is_symlink() or not (stat.S_ISDIR(metadata.st_mode) or stat.S_ISREG(metadata.st_mode)):
            raise SuccessorReleaseFailure()
        if stat.S_ISDIR(metadata.st_mode):
            if stat.S_IMODE(metadata.st_mode) != 0o555:
                raise SuccessorReleaseFailure()
            continue
        executable = bool(metadata.st_mode & 0o111)
        if stat.S_IMODE(metadata.st_mode) != (0o555 if executable else 0o444):
            raise SuccessorReleaseFailure()
        files.append(
            {
                "path": path.relative_to(root).as_posix(),
                "executable": executable,
                "size_bytes": metadata.st_size,
                "sha256": raw_hash(path),
            }
        )
    return files


def validate_root(root: Path, bundle: dict[str, object]) -> None:
    expected = bundle.get("inventory", {}).get("files")
    if inventory(root) != expected:
        raise SuccessorReleaseFailure()


def build_go(work: Path, candidate: dict[str, object]) -> dict[str, Path]:
    output = work / "output"
    go_release.build_release_outputs(output)
    roots = {
        str(candidate["frontend_bundles"][0]["bundle_id"]): output / "frontend",
        str(candidate["toolchain_bundles"][0]["bundle_id"]): output / "toolchain",
    }
    validate_root(output / "frontend", candidate["frontend_bundles"][0])
    validate_root(output / "toolchain", candidate["toolchain_bundles"][0])
    return roots


def build_csharp(work: Path, candidate: dict[str, object]) -> dict[str, Path]:
    generated_candidate, _registry, output = csharp.build_once(work)
    if generated_candidate != candidate:
        raise SuccessorReleaseFailure()
    roots = {
        str(candidate["frontend_bundles"][0]["bundle_id"]): output / "frontend",
        str(candidate["toolchain_bundles"][0]["bundle_id"]): output / "toolchain",
    }
    validate_root(output / "frontend", candidate["frontend_bundles"][0])
    validate_root(output / "toolchain", candidate["toolchain_bundles"][0])
    return roots


def build_rust(work: Path, candidate: dict[str, object]) -> dict[str, Path]:
    target = work / "target"
    cache = work / "cache"
    result = rust.run_hermetic(
        RUST_BUILD_ARGUMENTS,
        retained_target=target,
        retained_cache=cache,
    )
    if result.returncode != 0:
        raise SuccessorReleaseFailure()
    rust.normalize_portable_cpp_runtime(cache, target)
    frontend = work / "output/frontend"
    toolchain = work / "output/toolchain"
    frontend_bundle = candidate["frontend_bundles"][0]
    toolchain_bundle = candidate["toolchain_bundles"][0]
    release = target / "x86_64-unknown-linux-gnu/release"
    binary_sources = {
        "bin/rust2vir": release / "rust2vir",
        "bin/rust2vir-driver": release / "rust2vir-driver",
    }
    for item in frontend_bundle["inventory"]["files"]:
        copy_described(binary_sources[str(item["path"])], frontend / str(item["path"]), item)
    for item in toolchain_bundle["inventory"]["files"]:
        relative = str(item["path"])
        source = cache / relative if relative.startswith("native-runtime/") else cache / "toolchain" / relative
        copy_described(source, toolchain / relative, item)
    normalize_directories(frontend)
    normalize_directories(toolchain)
    validate_root(frontend, frontend_bundle)
    validate_root(toolchain, toolchain_bundle)
    return {
        str(frontend_bundle["bundle_id"]): frontend,
        str(toolchain_bundle["bundle_id"]): toolchain,
    }


def build_roots(work: Path, candidates: dict[str, dict[str, object]]) -> dict[str, Path]:
    roots: dict[str, Path] = {}
    for language, builder in (("go", build_go), ("rust", build_rust), ("csharp", build_csharp)):
        language_work = work / language
        language_work.mkdir(parents=True)
        generated = builder(language_work, candidates[language])
        if roots.keys() & generated.keys():
            raise SuccessorReleaseFailure()
        roots.update(generated)
    return roots


def tree_state(root: Path) -> dict[str, tuple[int, int, str]]:
    state: dict[str, tuple[int, int, str]] = {}
    for path in sorted(root.rglob("*")):
        metadata = path.lstat()
        if path.is_symlink():
            raise SuccessorReleaseFailure()
        relative = path.relative_to(root).as_posix()
        if stat.S_ISDIR(metadata.st_mode):
            state[relative] = (stat.S_IFDIR, stat.S_IMODE(metadata.st_mode), "")
        elif stat.S_ISREG(metadata.st_mode):
            state[relative] = (stat.S_IFREG, stat.S_IMODE(metadata.st_mode), raw_hash(path))
        else:
            raise SuccessorReleaseFailure()
    return state


def check() -> None:
    _registry, candidates = validate_release_models()
    with tempfile.TemporaryDirectory(prefix="mpk-successor-release-") as temporary:
        root = Path(temporary)
        first = build_roots(root / "first", candidates)
        second = build_roots(root / "second", candidates)
        if first.keys() != second.keys():
            raise SuccessorReleaseFailure()
        for bundle_id in first:
            if tree_state(first[bundle_id]) != tree_state(second[bundle_id]):
                raise SuccessorReleaseFailure()


def install(executable: Path, destination: Path) -> None:
    registry, candidates = validate_release_models()
    if (
        not executable.is_absolute()
        or executable.is_symlink()
        or not executable.is_file()
        or not destination.is_absolute()
        or destination.exists()
        or not destination.parent.is_dir()
    ):
        raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
    with tempfile.TemporaryDirectory(prefix="mpk-successor-install-") as temporary:
        roots = build_roots(Path(temporary), candidates)
        destination.mkdir(mode=0o755)
        (destination / "bin").mkdir(mode=0o755)
        (destination / "share/mpk").mkdir(mode=0o755, parents=True)
        bundles = destination / "libexec/mpk/bundles"
        bundles.mkdir(mode=0o755, parents=True)
        shutil.copyfile(executable, destination / "bin/mpk")
        (destination / "bin/mpk").chmod(0o555)
        shutil.copyfile(REGISTRY_PATH, destination / "share/mpk/bundle-registry.json")
        shutil.copyfile(
            SEMANTIC_REGISTRY_PATH,
            destination / "share/mpk/semantic-profile-registry.json",
        )
        for path in (destination / "share/mpk").iterdir():
            path.chmod(0o444)
        for bundle_id, source in roots.items():
            shutil.copytree(source, bundles / bundle_id, copy_function=shutil.copy2)
    normalize_directories(destination)
    expected_ids = {
        str(bundle["bundle_id"])
        for field in ("frontend_bundles", "toolchain_bundles")
        for bundle in registry[field]
    }
    if {path.name for path in (destination / "libexec/mpk/bundles").iterdir()} != expected_ids:
        raise SuccessorReleaseFailure()
    for field in ("frontend_bundles", "toolchain_bundles"):
        for bundle in registry[field]:
            validate_root(destination / "libexec/mpk/bundles" / str(bundle["bundle_id"]), bundle)


def run_installed(executable: Path) -> None:
    if not executable.is_absolute() or executable.name != "mpk":
        raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
    command = go_release.rust_fixture_cgroup_command(
        [str(executable), "--inside-successor-cutover"]
    )
    result = go_release.run_bounded_rust_fixture(command, cwd=executable.parent.parent, env={})
    if result.returncode != 0 or result.stderr:
        sys.stderr.buffer.write(result.stderr)
        raise SuccessorReleaseFailure()
    sys.stdout.buffer.write(result.stdout)


def main(argv: list[str]) -> int:
    try:
        if argv == ["check"]:
            check()
            print("SUCCESSOR_RELEASE_OK")
        elif len(argv) == 3 and argv[0] == "materialize-fixture":
            install(Path(argv[1]), Path(argv[2]))
        elif len(argv) == 2 and argv[0] == "run-installed":
            run_installed(Path(argv[1]))
        else:
            raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
        return 0
    except SuccessorReleaseFailure as error:
        print(error.code, file=sys.stderr)
        return error.exit_code
    except (
        csharp.CSharpReleaseFailure,
        csharp.csharp_build_inputs.CSharpBuildFailure,
        go_release.BundleFailure,
        rust.RustBuildFailure,
    ) as error:
        print(getattr(error, "code", "BUNDLE_REPRODUCIBILITY_MISMATCH"), file=sys.stderr)
        return getattr(error, "exit_code", 65)
    except (KeyError, OSError, TypeError, ValueError, subprocess.SubprocessError):
        print("BUNDLE_ASSEMBLER_IO", file=sys.stderr)
        return 74


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
