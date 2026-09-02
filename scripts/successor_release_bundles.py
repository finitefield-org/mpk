#!/usr/bin/env python3
"""Build, verify, and install the sole successor release image."""

from __future__ import annotations

from collections import Counter
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import zipfile

SCRIPT_ROOT = Path(__file__).resolve().parent
REPOSITORY_ROOT = SCRIPT_ROOT.parent
sys.path.insert(0, str(SCRIPT_ROOT))

import csharp_release_bundles as csharp  # noqa: E402
import java_release_bundles as java  # noqa: E402
import release_bundles as go_release  # noqa: E402
import rust_build_inputs as rust  # noqa: E402


REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/bundle-registry.json"
SEMANTIC_REGISTRY_PATH = REPOSITORY_ROOT / "release/bundles/semantic-profile-registry.json"
CANDIDATE_PATHS = {
    language: REPOSITORY_ROOT / f"release/bundles/candidates/{language}.json"
    for language in ("go", "rust", "csharp", "java")
}
REGISTRY_SHA256 = "7877c7c04fae912815713a8a7f6f9900198721ea572788f6f48d1dbe3f00afbd"
SEMANTIC_REGISTRY_SHA256 = (
    "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557"
)
REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
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
    vector_path = REPOSITORY_ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json"
    try:
        vectors = json.loads(vector_path.read_bytes())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SuccessorReleaseFailure() from error
    if (
        semantic != vectors.get("registry")
        or semantic.get("registry_sha256") != SEMANTIC_REGISTRY_SHA256
        or hashlib.sha256(semantic_bytes).hexdigest()
        != "1c9671f7db1f872f93d21757c271b39ddedd4fe612f4f388fe1700acd9808080"
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
        "f2da0168c30ac72b2d624a2af76959231114b5ad862fad9d893c4e15b48395d0"
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
        len(registry["frontend_bundles"]) != 4
        or len(registry["toolchain_bundles"]) != 4
        or len(registry["tuples"]) != 5
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


def build_java(work: Path, candidate: dict[str, object]) -> dict[str, Path]:
    roots = java.build_roots(work, candidate)
    for bundle in candidate["frontend_bundles"] + candidate["toolchain_bundles"]:
        validate_root(roots[str(bundle["bundle_id"])], bundle)
    return roots


def build_roots(work: Path, candidates: dict[str, dict[str, object]]) -> dict[str, Path]:
    roots: dict[str, Path] = {}
    for language, builder in (
        ("go", build_go),
        ("rust", build_rust),
        ("csharp", build_csharp),
        ("java", build_java),
    ):
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
    check_java_trace_parser()
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


def run_installed(executable: Path, *, hostile: bool = False) -> None:
    if not executable.is_absolute() or executable.name != "mpk":
        raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
    reports: list[object] = []
    for language in ("go", "rust", "csharp", "java"):
        suffix = "-hostile" if hostile else ""
        command = go_release.rust_fixture_cgroup_command(
            [str(executable), f"--inside-successor-cutover-{language}{suffix}"]
        )
        result = go_release.run_bounded_rust_fixture(
            command, cwd=executable.parent.parent, env={}
        )
        if result.returncode != 0 or result.stderr:
            sys.stderr.buffer.write(result.stderr)
            raise SuccessorReleaseFailure()
        try:
            report = json.loads(result.stdout)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise SuccessorReleaseFailure() from error
        if (
            not isinstance(report, dict)
            or report.get("language") != language
            or canonical_line(report) != result.stdout
        ):
            raise SuccessorReleaseFailure()
        reports.append(report)
    sys.stdout.buffer.write(
        canonical_line(
            {
                "languages": reports,
                "registry_sha256": REGISTRY_SHA256,
                "semantic_registry_sha256": SEMANTIC_REGISTRY_SHA256,
                "status": "active_successor",
            }
        )
    )


def parse_java_trace(data: bytes) -> dict[str, object]:
    """Attribute policy installation and thread creation to the active JVM."""
    try:
        lines = data.decode("ascii").splitlines()
    except UnicodeDecodeError as error:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE") from error
    events: list[tuple[int, str]] = []
    pending: dict[int, str] = {}
    for line in lines:
        match = re.fullmatch(r"([0-9]+)\s+(.+)", line)
        if match is None:
            continue
        pid, call = int(match[1]), match[2]
        resumed = re.fullmatch(r"<\.\.\. ([a-z0-9_]+) resumed>(.*)", call)
        if resumed is not None:
            prefix = pending.pop(pid, "")
            if not prefix.startswith(resumed[1] + "("):
                raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
            call = prefix + resumed[2]
        if call.endswith("<unfinished ...>"):
            if pid in pending:
                raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
            pending[pid] = call.removesuffix("<unfinished ...>")
        else:
            events.append((pid, call))
    launches = [
        (index, pid)
        for index, (pid, call) in enumerate(events)
        if call.startswith('execve("/mpk/toolchain/jdk/bin/java",') and call.endswith(" = 0")
    ]
    if len(launches) != 1 or pending:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
    start, java_pid = launches[0]
    before = [call for pid, call in events[:start] if pid == java_pid]
    if not any(
        call.startswith("seccomp(SECCOMP_SET_MODE_FILTER,") and call.endswith(" = 0")
        for call in before
    ):
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
    for family in ("AF_INET", "AF_UNIX"):
        if not any(
            call.startswith(f"socket({family},") and " = -1 EPERM " in call
            for call in before
        ):
            raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
    clones: list[tuple[int, int, set[str]]] = []
    for pid, call in events[start + 1 :]:
        match = re.fullmatch(
            r"clone\(.*flags=([^,]+),.*\)\s+= ([1-9][0-9]*)"
            r"(?: /\* ([1-9][0-9]*) in strace's PID NS \*/)?",
            call,
        )
        if match is not None:
            clones.append(
                (
                    pid,
                    int(match[3] or match[2]),
                    {flag.strip() for flag in match[1].split("|")},
                )
            )
    threads = {java_pid}
    while True:
        expanded = threads | {child for parent, child, _flags in clones if parent in threads}
        if expanded == threads:
            break
        threads = expanded
    observed = [flags for parent, _child, flags in clones if parent in threads]
    expected = {
        "CLONE_VM",
        "CLONE_FS",
        "CLONE_FILES",
        "CLONE_SIGHAND",
        "CLONE_THREAD",
        "CLONE_SYSVSEM",
        "CLONE_SETTLS",
        "CLONE_PARENT_SETTID",
        "CLONE_CHILD_CLEARTID",
    }
    if not observed or len(observed) != len(clones) or any(flags != expected for flags in observed):
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
    clone3_fallbacks = [
        pid
        for pid, call in events[start + 1 :]
        if call.startswith("clone3(") and " = -1 ENOSYS " in call
    ]
    if not clone3_fallbacks or any(pid not in threads for pid in clone3_fallbacks):
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
    return {
        "jvm_thread_flags": sorted(expected),
        "jvm_clone3_fallback": True,
        "jvm_thread_creations": len(observed),
        "preexec_socket_denials": ["AF_INET", "AF_UNIX"],
    }


def check_java_trace_parser() -> None:
    fixture = b'''7 seccomp(SECCOMP_SET_MODE_FILTER, 0, {len=1, filter=0x1}) = 0
7 socket(AF_INET, SOCK_STREAM, 0) = -1 EPERM (Operation not permitted)
7 socket(AF_UNIX, SOCK_STREAM, 0) = -1 EPERM (Operation not permitted)
7 execve("/mpk/toolchain/jdk/bin/java", ["java"], 0x1) = 0
7 clone3({flags=0}, 88) = -1 ENOSYS (Function not implemented)
7 clone(child_stack=0x1, flags=CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID <unfinished ...>
7 <... clone resumed>, parent_tid=0x1, tls=0x1, child_tidptr=0x1) = 2 /* 8 in strace's PID NS */
8 clone(child_stack=0x2, flags=CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID, parent_tid=0x2, tls=0x2, child_tidptr=0x2) = 3 /* 9 in strace's PID NS */
'''
    if parse_java_trace(fixture)["jvm_thread_creations"] != 2:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_TRACE_PARSER")
    mutations = (
        fixture.replace(b"7 clone", b"9 clone").replace(
            b"7 <... clone", b"9 <... clone"
        ),
        fixture.replace(b"8 clone", b"19 clone"),
        fixture.replace(b"7 seccomp", b"9 seccomp"),
        fixture.replace(b"7 socket(AF_UNIX", b"9 socket(AF_UNIX"),
        fixture.replace(b"7 clone3", b"19 clone3"),
        fixture.replace(b"|CLONE_THREAD", b"|CLONE_VFORK"),
        fixture[: fixture.index(b"7 <... clone")],
    )
    for changed in mutations:
        try:
            parse_java_trace(changed)
        except SuccessorReleaseFailure:
            continue
        raise SuccessorReleaseFailure("BUNDLE_JAVA_TRACE_PARSER")


def run_java_case(executable: Path, case: str, *, hostile: bool = False) -> bytes:
    if case not in {"int.identity", "int.division", "int.shift_unsigned_right"}:
        raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
    mode = (
        "--inside-successor-cutover-java-hostile-case"
        if hostile
        else "--inside-successor-cutover-java-case"
    )
    command = go_release.rust_fixture_cgroup_command([str(executable), mode, case])
    result = go_release.run_bounded_rust_fixture(
        command, cwd=executable.parent.parent, env={}
    )
    if result.returncode != 0 or result.stderr:
        sys.stderr.buffer.write(result.stderr)
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_RUN")
    try:
        report = json.loads(result.stdout)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_RUN") from error
    if (
        not isinstance(report, dict)
        or report.get("language") != "java"
        or report.get("status") != "ir-lowered"
        or canonical_line(report) != result.stdout
    ):
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_RUN")
    return result.stdout


def check_cgroup_cleanup_attribution() -> None:
    """Reject cleanup accounting that accidentally consumes sibling churn."""
    command = go_release.rust_fixture_cgroup_command(["/usr/bin/sleep", "1"])
    unrelated = Path(f"/sys/fs/cgroup/mpk-java-unrelated-{os.getpid()}")
    process: subprocess.Popen[bytes] | None = None
    unrelated_created = False
    try:
        process = subprocess.Popen(
            command,
            cwd="/tmp",
            env={},
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        owned = Path(f"/sys/fs/cgroup/mpk-rust-fixture-{process.pid}")
        deadline = time.monotonic() + 2
        while (
            not (owned / "domain").is_dir()
            and process.poll() is None
            and time.monotonic() < deadline
        ):
            time.sleep(0.005)
        if not (owned / "domain").is_dir() or unrelated.exists():
            raise SuccessorReleaseFailure("BUNDLE_JAVA_CGROUP_ATTRIBUTION")
        unrelated.mkdir()
        unrelated_created = True
        stdout, stderr = process.communicate(timeout=10)
        if process.returncode != 0 or stdout or stderr or owned.exists():
            raise SuccessorReleaseFailure("BUNDLE_JAVA_CGROUP_ATTRIBUTION")
    except (OSError, subprocess.SubprocessError) as error:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_CGROUP_ATTRIBUTION") from error
    finally:
        cleanup_failed = False
        if process is not None and process.poll() is None:
            try:
                process.terminate()
                try:
                    process.communicate(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.communicate()
            except (OSError, subprocess.SubprocessError):
                cleanup_failed = True
        if unrelated_created:
            try:
                unrelated.rmdir()
            except OSError:
                cleanup_failed = True
        if cleanup_failed:
            raise SuccessorReleaseFailure("BUNDLE_JAVA_CGROUP_ATTRIBUTION")


def make_tree_writable(root: Path) -> None:
    for directory, names, _files in os.walk(root):
        Path(directory).chmod(0o700)
        for name in names:
            path = Path(directory, name)
            if not path.is_symlink():
                path.chmod(0o700)


def rewrite_file(path: Path, data: bytes, mode: int = 0o444) -> None:
    path.chmod(0o600)
    path.write_bytes(data)
    path.chmod(mode)


def validate_java_active_mutations(executable: Path) -> list[str]:
    image = executable.parent.parent
    control = go_release.rust_fixture_cgroup_command(
        [str(executable), "--inside-successor-java-release-control"]
    )
    control_result = go_release.run_bounded_rust_fixture(
        control, cwd=image, env={}
    )
    if control_result.returncode != 0 or control_result.stdout or control_result.stderr:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_ACTIVE_MUTATION_CONTROL")
    frontend = f"libexec/mpk/bundles/{java.FRONTEND_ID}/java2vir.jar"
    java_executable = f"libexec/mpk/bundles/{java.TOOLCHAIN_ID}/jdk/bin/java"
    modules = f"libexec/mpk/bundles/{java.TOOLCHAIN_ID}/jdk/lib/modules"
    jvm = f"libexec/mpk/bundles/{java.TOOLCHAIN_ID}/jdk/lib/server/libjvm.so"
    mutations = [
        "jar-byte",
        "manifest-classpath",
        "processor-service",
        "jdk-byte",
        "missing-native",
        "unknown-native",
        "writable-input",
        "symlink",
        "hardlink",
        "registry-context",
    ]
    completed: list[str] = []
    with tempfile.TemporaryDirectory(prefix="mpk-java-active-mutations-") as temporary:
        root = Path(temporary)
        for mutation in mutations:
            changed = root / mutation
            shutil.copytree(image, changed, symlinks=True)
            try:
                if mutation in {"jar-byte", "jdk-byte"}:
                    path = changed / (frontend if mutation == "jar-byte" else modules)
                    data = bytearray(path.read_bytes())
                    data[-1] ^= 1
                    rewrite_file(path, bytes(data))
                elif mutation in {"manifest-classpath", "processor-service"}:
                    path = changed / frontend
                    with zipfile.ZipFile(io.BytesIO(path.read_bytes())) as original:
                        output = io.BytesIO()
                        with zipfile.ZipFile(
                            output, "w", compression=zipfile.ZIP_STORED
                        ) as archive:
                            for name in original.namelist():
                                value = original.read(name)
                                if (
                                    mutation == "manifest-classpath"
                                    and name == "META-INF/MANIFEST.MF"
                                ):
                                    value = (
                                        b"Manifest-Version: 1.0\r\n"
                                        b"Class-Path: /host/poison.jar\r\n\r\n"
                                    )
                                archive.writestr(name, value)
                            if mutation == "processor-service":
                                archive.writestr(
                                    "META-INF/services/javax.annotation.processing.Processor",
                                    "poison.Processor\n",
                                )
                    rewrite_file(path, output.getvalue())
                elif mutation == "writable-input":
                    (changed / java_executable).chmod(0o755)
                elif mutation == "registry-context":
                    path = changed / "share/mpk/bundle-registry.json"
                    value = json.loads(path.read_bytes())
                    value["profile_registry"]["revision"] = 2
                    payload = dict(value)
                    payload.pop("registry_sha256", None)
                    value["registry_sha256"] = hashlib.sha256(
                        REGISTRY_DOMAIN + canonical(payload)
                    ).hexdigest()
                    rewrite_file(path, canonical_line(value))
                else:
                    path = changed / jvm
                    path.parent.chmod(0o755)
                    if mutation == "unknown-native":
                        unregistered = path.parent / "unregistered.so"
                        unregistered.write_bytes(b"unregistered")
                        unregistered.chmod(0o444)
                    else:
                        path.unlink()
                        if mutation == "symlink":
                            path.symlink_to("../libjava.so")
                        elif mutation == "hardlink":
                            os.link(path.parent.parent / "libjava.so", path)
                    path.parent.chmod(0o555)
                command = go_release.rust_fixture_cgroup_command(
                    [
                        str(changed / "bin/mpk"),
                        "--inside-successor-java-release-before-source",
                    ]
                )
                result = go_release.run_bounded_rust_fixture(
                    command, cwd=changed, env={}
                )
                if result.returncode != 0 or result.stdout or result.stderr:
                    raise SuccessorReleaseFailure("BUNDLE_JAVA_ACTIVE_MUTATION")
                completed.append(mutation)
            finally:
                make_tree_writable(changed)
                shutil.rmtree(changed)
    return completed


def run_java_native_gate(executable: Path) -> None:
    try:
        cpuinfo = Path("/proc/cpuinfo").read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_HOST_REQUIRED", 69) from error
    required_cpu_flags = {"lm", "sse2"}
    has_required_cpu_flags = any(
        line.startswith("flags")
        and required_cpu_flags.issubset(line.split(":", 1)[1].split())
        for line in cpuinfo.splitlines()
        if ":" in line
    )
    if (
        sys.platform != "linux"
        or os.uname().machine != "x86_64"
        or os.geteuid() != 0
        or not has_required_cpu_flags
        or not executable.is_absolute()
        or executable.name != "mpk"
        or not os.access("/sys/fs/cgroup/cgroup.subtree_control", os.W_OK)
        or not Path("/usr/bin/strace").is_file()
    ):
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_HOST_REQUIRED", 69)

    check_java_trace_parser()
    check_cgroup_cleanup_attribution()
    cases = {
        case: hashlib.sha256(run_java_case(executable, case)).hexdigest()
        for case in ("int.identity", "int.division", "int.shift_unsigned_right")
    }
    hostile = run_java_case(executable, "int.identity", hostile=True)
    if hashlib.sha256(hostile).hexdigest() != cases["int.identity"]:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_AMBIENT")

    # A complete active image must not reach source processing unless the
    # release supervisor has delegated the exact owned cgroup hierarchy.
    try:
        undelegated = subprocess.run(
            [
                str(executable),
                "--inside-successor-cutover-java-case",
                "int.identity",
            ],
            cwd=executable.parent.parent,
            env={},
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=120,
            check=False,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_CGROUP_REQUIRED") from error
    if undelegated.returncode == 0 or undelegated.stdout:
        raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_CGROUP_REQUIRED")

    faults = ["oom", "pids", "timeout", "stdout", "stderr", "tmpfs"]
    for case in faults:
        command = go_release.rust_fixture_cgroup_command(
            [str(executable), "--inside-successor-java-resource", case]
        )
        result = go_release.run_bounded_rust_fixture(
            command, cwd=executable.parent.parent, env={}
        )
        if result.returncode != 0 or result.stdout or result.stderr:
            raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_RESOURCE")

    mutation_ids = validate_java_active_mutations(executable)

    with tempfile.TemporaryDirectory(prefix="mpk-java-active-native-") as temporary:
        trace = Path(temporary) / "syscalls.log"
        command = go_release.rust_fixture_cgroup_command(
            [
                str(executable),
                "--inside-successor-java-trace-probe",
            ]
        )
        traced = go_release.run_bounded_rust_fixture(
            [
                "/usr/bin/strace",
                "-f",
                "-qq",
                "--decode-pids=pidns",
                "-s",
                "256",
                "-o",
                str(trace),
                "-e",
                "trace=execve,execveat,clone,clone3,seccomp,prctl,capset,socket,unshare",
                *command,
            ],
            cwd=executable.parent.parent,
            env={},
        )
        if traced.returncode != 0 or traced.stderr:
            sys.stderr.buffer.write(traced.stderr)
            raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
        if traced.stdout:
            raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
        trace_bytes = trace.read_bytes()
        if len(trace_bytes) > 8 * 1024 * 1024:
            raise SuccessorReleaseFailure("BUNDLE_JAVA_NATIVE_TRACE")
        observation = parse_java_trace(trace_bytes)

    sys.stdout.buffer.write(
        canonical_line(
            {
                "cases": cases,
                "hostile_environment_equal": True,
                "native_architecture": "x86_64",
                "registry_sha256": REGISTRY_SHA256,
                "resource_faults": faults,
                "schema": "mpk.java.active_native_gate.v0",
                "source_precedence_mutations": mutation_ids,
                "syscall_observation": observation,
                "syscall_trace_sha256": hashlib.sha256(trace_bytes).hexdigest(),
                "undelegated_cgroup_rejected": True,
            }
        )
    )


def write_canonical(path: Path, value: object) -> None:
    temporary = path.with_name(f".{path.name}.tmp")
    temporary.write_bytes(canonical_line(value))
    os.replace(temporary, path)


def update_descriptors() -> None:
    semantic_vector = json.loads(
        (REPOSITORY_ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json").read_bytes()
    )
    semantic = semantic_vector["registry"]
    identity = {key: semantic[key] for key in ("schema", "id", "revision", "registry_sha256")}
    candidates: dict[str, dict[str, object]] = {}
    for language in ("go", "rust", "csharp"):
        current = read_canonical(CANDIDATE_PATHS[language])[0]
        candidate = copy.deepcopy(current)
        candidate["profile_registry"] = identity
        for release_tuple in candidate["tuples"]:
            release_tuple["semantic_context"]["profile_registry"] = identity
        candidates[language] = candidate
    candidates["java"] = copy.deepcopy(java.descriptors())
    registry: dict[str, object] = {
        "schema": "mpk.release.bundle_registry.v1",
        "id": "mpk.release.registry.v1",
        "profile_registry": identity,
        "execution_host_profiles": sorted(
            keyed_union(list(candidates.values()), "execution_host_profiles", "id"),
            key=lambda value: value["id"],
        ),
        "native_runtime_layout_profiles": sorted(
            keyed_union(list(candidates.values()), "native_runtime_layout_profiles", "id"),
            key=lambda value: value["id"],
        ),
        "frontend_bundles": sorted(
            keyed_union(list(candidates.values()), "frontend_bundles", "bundle_id"),
            key=lambda value: value["bundle_id"],
        ),
        "toolchain_bundles": sorted(
            keyed_union(list(candidates.values()), "toolchain_bundles", "bundle_id"),
            key=lambda value: value["bundle_id"],
        ),
        "tuples": [
            release_tuple
            for candidate in candidates.values()
            for release_tuple in candidate["tuples"]
        ],
    }
    registry["registry_sha256"] = hashlib.sha256(
        REGISTRY_DOMAIN + canonical(registry)
    ).hexdigest()
    write_canonical(SEMANTIC_REGISTRY_PATH, semantic)
    for language, candidate in candidates.items():
        write_canonical(CANDIDATE_PATHS[language], candidate)
    write_canonical(REGISTRY_PATH, registry)
    print(registry["registry_sha256"])


def rebound_candidate(language: str) -> dict[str, object]:
    semantic = json.loads(
        (REPOSITORY_ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json").read_bytes()
    )["registry"]
    identity = {key: semantic[key] for key in ("schema", "id", "revision", "registry_sha256")}
    candidate = copy.deepcopy(read_canonical(CANDIDATE_PATHS[language])[0])
    candidate["profile_registry"] = identity
    for release_tuple in candidate["tuples"]:
        release_tuple["semantic_context"]["profile_registry"] = identity
    return candidate


def update_go_candidate() -> None:
    candidate = rebound_candidate("go")
    with tempfile.TemporaryDirectory(
        prefix=".mpk-go-candidate-", dir=REPOSITORY_ROOT / "release"
    ) as temporary:
        output = Path(temporary) / "output"
        go_release.build_release_outputs(output)
        refresh_frontend_candidate(candidate, output / "frontend")
    write_canonical(CANDIDATE_PATHS["go"], candidate)


def update_rust_candidate() -> None:
    candidate = rebound_candidate("rust")
    with tempfile.TemporaryDirectory(
        prefix=".mpk-rust-candidate-", dir=REPOSITORY_ROOT / "release"
    ) as temporary:
        work = Path(temporary)
        target = portable_rust_build(work)
        refresh_frontend_candidate(
            candidate,
            target / "x86_64-unknown-linux-gnu/release",
            {"bin/rust2vir": "rust2vir", "bin/rust2vir-driver": "rust2vir-driver"},
        )
    write_canonical(CANDIDATE_PATHS["rust"], candidate)


def portable_rust_build(work: Path) -> Path:
    vector = rust.load_vector()
    _descriptor, cache = rust.check_build_inputs()
    rust.require_image(rust.RUNTIME_IMAGE)
    target = work / "target"
    target.mkdir(mode=0o777)
    target.chmod(0o777)
    environment = rust.docker_build_environment(vector)
    command = [
        rust.docker_path(),
        "run",
        "--rm",
        "--pull=never",
        "--network=none",
        "--ipc=none",
        "--platform=linux/amd64",
        "--read-only",
        "--user=65534:65534",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        "--security-opt=seccomp=unconfined",
        "--pids-limit=-1",
        "--ulimit=nofile=4096:4096",
        "--tmpfs=/mpk/home:rw,nosuid,nodev,noexec,mode=0777",
        "--tmpfs=/mpk/cargo-home:rw,nosuid,nodev,noexec,mode=0777",
        "--tmpfs=/mpk/tmp:rw,nosuid,nodev,noexec,mode=1777",
        "--tmpfs=/mpk/work:rw,nosuid,nodev,noexec,mode=0777",
        f"--mount=type=bind,src={REPOSITORY_ROOT / 'rust-tools/rust2vir'},dst=/mpk/frontend,readonly",
        f"--mount=type=bind,src={cache / 'toolchain'},dst=/mpk/toolchain,readonly",
        f"--mount=type=bind,src={cache / 'vendor'},dst=/mpk/vendor,readonly",
        f"--mount=type=bind,src={cache / 'cargo-home-seed/config.toml'},dst=/mpk/cargo-home/config.toml,readonly",
        f"--mount=type=bind,src={cache / 'native-sysroot'},dst=/mpk/native-sysroot,readonly",
        f"--mount=type=bind,src={cache / 'native-runtime'},dst=/mpk/native-runtime,readonly",
        f"--mount=type=bind,src={cache / 'native-runtime/lib64'},dst=/lib64,readonly",
        f"--mount=type=bind,src={cache / 'native-runtime/lib/x86_64-linux-gnu'},dst=/lib/x86_64-linux-gnu,readonly",
        f"--mount=type=bind,src={cache / 'native-runtime/lib/x86_64-linux-gnu'},dst=/usr/lib/x86_64-linux-gnu,readonly",
        f"--mount=type=bind,src={target},dst=/mpk/target",
        "--workdir=/mpk/frontend",
        rust.RUNTIME_IMAGE,
        "/usr/bin/env",
        "-i",
        *environment,
        "/mpk/toolchain/bin/cargo",
        *RUST_BUILD_ARGUMENTS[1:],
    ]
    result = subprocess.run(command, stdin=subprocess.DEVNULL, capture_output=True, check=False)
    if result.returncode != 0 or result.stdout:
        sys.stderr.buffer.write(result.stderr)
        raise SuccessorReleaseFailure()
    return target


def refresh_frontend_candidate(
    candidate: dict[str, object],
    source: Path,
    source_paths: dict[str, str] | None = None,
) -> None:
    frontend = candidate["frontend_bundles"][0]
    current = frontend["inventory"]
    records = []
    for item in current["files"]:
        relative = str(item["path"])
        path = source / (source_paths or {}).get(relative, relative)
        metadata = path.lstat()
        if path.is_symlink() or not stat.S_ISREG(metadata.st_mode):
            raise SuccessorReleaseFailure()
        records.append(
            {
                "path": relative,
                "executable": bool(item["executable"]),
                "size_bytes": metadata.st_size,
                "sha256": raw_hash(path),
            }
        )
    current["files"] = records
    frontend["bundle_sha256"] = hashlib.sha256(
        CONTENT_DOMAIN + canonical(current)
    ).hexdigest()
    by_path = {item["path"]: item for item in records}
    frontend["main"]["binary_sha256"] = by_path[frontend["main"]["path"]]["sha256"]
    for binary in frontend["subordinate_binaries"]:
        binary["binary_sha256"] = by_path[binary["path"]]["sha256"]


def main(argv: list[str]) -> int:
    try:
        if argv == ["check"]:
            check()
            print("SUCCESSOR_RELEASE_OK")
        elif argv == ["update-descriptors"]:
            update_descriptors()
        elif argv == ["update-go-candidate"]:
            update_go_candidate()
        elif argv == ["update-rust-candidate"]:
            update_rust_candidate()
        elif len(argv) == 3 and argv[0] == "materialize-fixture":
            install(Path(argv[1]), Path(argv[2]))
        elif len(argv) == 2 and argv[0] == "run-installed":
            run_installed(Path(argv[1]))
        elif len(argv) == 2 and argv[0] == "run-installed-hostile":
            run_installed(Path(argv[1]), hostile=True)
        elif len(argv) == 2 and argv[0] == "run-installed-java-native-gate":
            run_java_native_gate(Path(argv[1]))
        else:
            raise SuccessorReleaseFailure("BUNDLE_ASSEMBLER_USAGE", 64)
        return 0
    except SuccessorReleaseFailure as error:
        print(error.code, file=sys.stderr)
        return error.exit_code
    except (
        csharp.CSharpReleaseFailure,
        csharp.csharp_build_inputs.CSharpBuildFailure,
        java.BUILD.BuildFailure,
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
