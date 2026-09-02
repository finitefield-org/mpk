#!/usr/bin/env python3
"""Offline builder for the private JAVA-03 frontend candidate."""

from __future__ import annotations

import base64
from contextlib import contextmanager
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import posixpath
import re
import resource
import selectors
import shutil
import signal
import socket
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import uuid
import zipfile


ROOT = Path(__file__).resolve().parent.parent
PROJECT = "java-tools/java2vir"
VECTOR = "develop/specs/vectors/java-profile-v0.json"
DESCRIPTOR = "release/build-inputs/java/build-inputs.json"
INVENTORY = "release/build-inputs/java/candidate-inventory.json"
TOOLCHAIN_HASH = "a75175ba0cce86d97a8e056d4dda7a0826bb6676ba551c454bd65e5d44d23fc4"
TOOLCHAIN_DOMAIN = b"MPK-JAVA-TOOLCHAIN-INPUTS-0.1\0"
IMAGE = "docker.io/library/python@sha256:db8e83a44af476c636a6a753adace39ad37863b63c0afd2862db7bbafeeb3944"
ARCHIVE_NAME = "OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz"
CACHE = ROOT / "release/build-input-cache/java" / TOOLCHAIN_HASH
PROJECT_FILES = (
    "META-INF/MANIFEST.MF",
    "NOTICE.txt",
    "src/mpk/java2vir/BuildIdentity.java",
    "src/mpk/java2vir/CanonicalJson.java",
    "src/mpk/java2vir/CapturedSnapshot.java",
    "src/mpk/java2vir/ClosedFileManager.java",
    "src/mpk/java2vir/CompilerDiagnostics.java",
    "src/mpk/java2vir/CompilerSession.java",
    "src/mpk/java2vir/DiagnosticRegistry.java",
    "src/mpk/java2vir/FrontendArguments.java",
    "src/mpk/java2vir/FrontendFailure.java",
    "src/mpk/java2vir/FrontendLimits.java",
    "src/mpk/java2vir/JavaAdmission.java",
    "src/mpk/java2vir/JavaContracts.java",
    "src/mpk/java2vir/JavaEmission.java",
    "src/mpk/java2vir/JavaFrontend.java",
    "src/mpk/java2vir/JavaIr.java",
    "src/mpk/java2vir/JavaLowering.java",
    "src/mpk/java2vir/JavaLoweringValidation.java",
    "src/mpk/java2vir/JavaRelease.java",
    "src/mpk/java2vir/JavaSourceMaps.java",
    "src/mpk/java2vir/JavaSubset.java",
    "src/mpk/java2vir/Main.java",
    "src/mpk/java2vir/Protocol.java",
    "src/mpk/java2vir/RuntimePreflight.java",
    "src/mpk/java2vir/ScalarType.java",
    "src/mpk/java2vir/Selection.java",
    "src/mpk/java2vir/SourceText.java",
    "src/mpk/java2vir/SourceTokens.java",
    "src/mpk/java2vir/StrictJson.java",
    "src/mpk/java2vir/TreeInventory.java",
)
MANIFEST = b"Manifest-Version: 1.0\nMain-Class: mpk.java2vir.Main\n\n"
VERSION = b"java2vir 0.1.0 (Temurin 25.0.4.1+1; inactive)\n"
UNAVAILABLE = b"JAVA_FRONTEND_UNAVAILABLE\n"
MAX_JSON = 4 * 1024 * 1024
MAX_SOURCE = 1024 * 1024
MAX_CLASSES = 1024
MAX_CLASS_BYTES = 1024 * 1024
MAX_JAR_BYTES = 16 * 1024 * 1024
MAX_REPORT = 32 * 1024 * 1024
MAX_STDERR = 64 * 1024
BUILD_SECONDS = 300
ENVIRONMENT = {
    "HOME": "/work/home", "TMPDIR": "/work/tmp", "PATH": "/nonexistent",
    "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC",
}
JVM_ARGUMENTS = (
    "-Xint", "-Xshare:off", "-XX:+UseSerialGC", "-XX:ActiveProcessorCount=1",
    "-XX:+DisableAttachMechanism", "-XX:-UsePerfData", "-Xms32m", "-Xmx512m",
    "-Xss1m", "-Dfile.encoding=UTF-8", "-Duser.language=en", "-Duser.country=US",
    "-Duser.timezone=UTC", "-Djava.io.tmpdir=/work/tmp", "-Duser.home=/work/home",
    "-Djava.library.path=/nonexistent", "-XX:ErrorFile=/work/tmp/hs_err.log",
    "-XX:-CreateCoredumpOnCrash", "-XX:-HeapDumpOnOutOfMemoryError",
    "--limit-modules=java.base,java.compiler,jdk.compiler,jdk.zipfs",
    "--add-modules=java.compiler,jdk.compiler,jdk.zipfs",
)
COMPILER_ARGUMENTS = (
    "--release", "25", "-encoding", "UTF-8", "-g:none", "-proc:none",
    "-implicit:none", "-Xlint:all", "-Werror",
    "--class-path", "/work/empty", "--source-path", "/work/empty",
    "--processor-path", "/work/empty", "--module-path", "/work/empty",
    "-d", "/work/classes",
)
RECIPE = {
    "id": "mpk.java.build_recipe.javac_direct.v0",
    "image": IMAGE, "platform": "linux/amd64", "network": "none",
    "compiler": "/mpk/toolchain/jdk/bin/javac",
    "compiler_arguments": list(COMPILER_ARGUMENTS),
    "compiler_jvm_arguments": ["-J" + argument for argument in JVM_ARGUMENTS],
    "source_order": "project_files_filtered_java",
    "environment": ENVIRONMENT, "package_restore": "forbidden",
    "jar": {
        "compression": "stored", "entry_order": "ascii_path",
        "timestamp": [1980, 1, 1, 0, 0, 0], "file_mode": "0644",
        "creator_system": 3, "extra": "", "comment": "",
        "manifest": "META-INF/MANIFEST.MF", "class_major_version": 69,
        "class_path": "forbidden", "service_providers": "forbidden",
    },
    "resources": {
        "memory_bytes": 1073741824, "swap_bytes": 0, "pids": 128,
        "address_space_bytes": 17179869184, "open_files": 1024, "core_bytes": 0,
        "work_bytes": 134217728, "timeout_seconds": BUILD_SECONDS,
        "report_bytes": MAX_REPORT, "stderr_bytes": MAX_STDERR,
    },
}


class BuildFailure(Exception):
    def __init__(self, code="JAVA_BUILD_INPUTS_INVALID", exit_code=65):
        super().__init__(code)
        self.code = code
        self.exit_code = exit_code


def require(condition, code="JAVA_BUILD_INPUTS_INVALID", exit_code=65):
    if not condition:
        raise BuildFailure(code, exit_code)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, allow_nan=False,
                      sort_keys=True, separators=(",", ":")).encode("utf-8")


def pairs(items):
    result = {}
    for key, value in items:
        require(key not in result)
        result[key] = value
    return result


def invalid_number(_):
    raise BuildFailure()


def strict_json(data, *, maximum=MAX_JSON, canonical_transport=False):
    require(len(data) <= maximum and not data.startswith(b"\xef\xbb\xbf"))
    try:
        result = json.loads(data.decode("utf-8"), object_pairs_hook=pairs,
                            parse_float=invalid_number, parse_constant=invalid_number)
        encoded = canonical(result)
        require(not canonical_transport or data == encoded + b"\n")
        return result
    except (ValueError, UnicodeError, RecursionError) as error:
        raise BuildFailure() from error


def exact_keys(value, keys):
    require(isinstance(value, dict) and set(value) == set(keys))


def relative_path(value, *, root=False):
    require(isinstance(value, str) and value.isascii() and len(value) <= 1024)
    if root and value == ".":
        return value
    require(bool(value) and not value.startswith("/") and "\\" not in value and ":" not in value)
    require(all(part not in ("", ".", "..") for part in value.split("/")))
    return value


def plain_directory(path):
    require(stat.S_ISDIR(path.lstat().st_mode))


def plain_chain(path, anchor):
    """Refuse aliases within a caller-chosen, already canonical trusted root."""
    path.relative_to(anchor)
    plain_directory(anchor)
    current = anchor
    for part in path.relative_to(anchor).parts:
        current /= part
        plain_directory(current)


@contextmanager
def opened_regular(path, maximum):
    descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        before = os.fstat(descriptor)
        require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1)
        require(0 <= before.st_size <= maximum)
        with os.fdopen(descriptor, "rb", closefd=False) as source:
            yield source, before
        after = os.fstat(descriptor)
        current = path.lstat()
        attributes = ("st_dev", "st_ino", "st_size", "st_mode", "st_nlink",
                      "st_mtime_ns", "st_ctime_ns")
        require(all(getattr(before, key) == getattr(after, key) == getattr(current, key)
                    for key in attributes))
    finally:
        os.close(descriptor)


def read_bytes(path, maximum):
    with opened_regular(path, maximum) as (source, before):
        data = source.read(maximum + 1)
        require(len(data) == before.st_size)
    return data


def load_json(path, *, canonical_transport=False):
    return strict_json(read_bytes(path, MAX_JSON), canonical_transport=canonical_transport)


def file_record(path, name, maximum=MAX_SOURCE):
    with opened_regular(path, maximum) as (source, before):
        digest = hashlib.sha256()
        size = 0
        while True:
            block = source.read(min(1024 * 1024, maximum - size + 1))
            if not block:
                break
            size += len(block)
            require(size <= maximum)
            digest.update(block)
        require(size == before.st_size)
        return {"path": name, "size_bytes": size, "sha256": digest.hexdigest(),
                "mode": f"{stat.S_IMODE(before.st_mode):04o}"}


def record_bytes(name, data):
    return {"path": name, "size_bytes": len(data), "sha256": sha256(data), "mode": "0644"}


def copy_verified(source_path, destination, expected, maximum):
    with opened_regular(source_path, maximum) as (source, before):
        require(before.st_size == expected["size_bytes"])
        digest = hashlib.sha256()
        size = 0
        with destination.open("xb") as output:
            while True:
                block = source.read(min(1024 * 1024, expected["size_bytes"] - size + 1))
                if not block:
                    break
                size += len(block)
                require(size <= expected["size_bytes"])
                digest.update(block)
                output.write(block)
        require(size == expected["size_bytes"] and digest.hexdigest() == expected["sha256"])
    destination.chmod(int(expected["mode"], 8))


def tree_entries(root):
    plain_directory(root)
    found = {".": root}
    pending = [root]
    while pending:
        parent = pending.pop()
        for entry in sorted(parent.iterdir()):
            name = entry.relative_to(root).as_posix()
            relative_path(name)
            require(len(found) < 4096)
            found[name] = entry
            if stat.S_ISDIR(entry.lstat().st_mode):
                pending.append(entry)
    return found


def validate_jdk(root, inputs):
    expected = {item["path"]: item for item in inputs["jdk_inventory"]}
    actual = tree_entries(root)
    require(actual.keys() == expected.keys())
    for name, path in actual.items():
        item = expected[name]
        mode = path.lstat().st_mode
        if item["kind"] == "directory":
            require(stat.S_ISDIR(mode) and f"{stat.S_IMODE(mode):04o}" == item["mode"])
        elif item["kind"] == "symlink":
            require(stat.S_ISLNK(mode) and os.readlink(path) == item["target"])
        else:
            require(item["kind"] == "regular")
            record = file_record(path, name, inputs["archive_policy"]["max_extracted_bytes"])
            require(record == {"path": name, "mode": item["mode"],
                               "size_bytes": item["bytes"], "sha256": item["sha256"]})


def validate_toolchain(inputs):
    require(isinstance(inputs, dict) and inputs.get("toolchain_inputs_sha256") == TOOLCHAIN_HASH)
    payload = {key: value for key, value in inputs.items() if key != "toolchain_inputs_sha256"}
    require(sha256(TOOLCHAIN_DOMAIN + canonical(payload)) == TOOLCHAIN_HASH)
    require(inputs["native_image"] == IMAGE)
    require(inputs["archive"]["url"].rsplit("/", 1)[-1] == ARCHIVE_NAME)


def load_toolchain():
    inputs = load_json(ROOT / VECTOR)["toolchain_inputs"]
    validate_toolchain(inputs)
    return inputs


def archive_record(inputs):
    item = inputs["archive"]
    return {"path": ARCHIVE_NAME, "size_bytes": item["bytes"],
            "sha256": item["sha256"], "mode": "0444"}


def check_cache(inputs):
    try:
        plain_chain(CACHE, ROOT)
        require({item.name for item in CACHE.iterdir()} == {ARCHIVE_NAME})
        actual = file_record(CACHE / ARCHIVE_NAME, ARCHIVE_NAME,
                             inputs["archive_policy"]["max_archive_bytes"])
        require(actual == archive_record(inputs))
    except FileNotFoundError as error:
        raise BuildFailure("JAVA_BUILD_CACHE_MISSING", 66) from error
    return CACHE / ARCHIVE_NAME


def import_archive(path, inputs):
    source = Path(path)
    require(source.is_absolute(), "JAVA_BUILD_USAGE", 64)
    for directory in reversed([CACHE, *CACHE.parents]):
        if directory == ROOT or ROOT in directory.parents:
            if not directory.exists():
                directory.mkdir(mode=0o755)
            plain_directory(directory)
    require(not list(CACHE.iterdir()), "JAVA_BUILD_CACHE_EXISTS")
    with tempfile.TemporaryDirectory(prefix=".java-import-", dir=CACHE.parent) as temporary:
        staged = Path(temporary) / ARCHIVE_NAME
        copy_verified(source, staged, archive_record(inputs),
                      inputs["archive_policy"]["max_archive_bytes"])
        # No replacement: a concurrent importer must not be overwritten.
        os.link(staged, CACHE / ARCHIVE_NAME, follow_symlinks=False)
        staged.unlink()
    check_cache(inputs)


def extract_jdk(archive_path, destination, inputs):
    """Materialize only inventoried entries; never use permissive tar extraction."""
    expected = {item["path"]: item for item in inputs["jdk_inventory"]}
    require(len(expected) == len(inputs["jdk_inventory"]) <= inputs["archive_policy"]["max_entries"])
    members = {}
    with opened_regular(archive_path, inputs["archive_policy"]["max_archive_bytes"]) as (source, _):
        with tarfile.open(fileobj=source, mode="r:gz") as archive:
            total = 0
            for member in archive:
                raw = member.name.rstrip("/")
                prefix = inputs["archive"]["root"]
                require(raw == prefix or raw.startswith(prefix + "/"))
                name = "." if raw == prefix else relative_path(raw[len(prefix) + 1:])
                require(name in expected and name not in members)
                item = expected[name]
                require(f"{member.mode:04o}" == item["mode"])
                require(not member.pax_headers)
                if member.isdir():
                    require(item["kind"] == "directory")
                elif member.isreg():
                    require(item["kind"] == "regular" and member.size == item["bytes"])
                    total += member.size
                    require(total <= inputs["archive_policy"]["max_extracted_bytes"])
                elif member.issym():
                    require(item["kind"] == "symlink" and member.linkname == item["target"])
                    target = posixpath.normpath(posixpath.join(posixpath.dirname(name), member.linkname))
                    require(not member.linkname.startswith("/") and target in expected)
                    require(expected[target]["kind"] == "regular")
                else:
                    raise BuildFailure()
                members[name] = member
                require(len(members) <= inputs["archive_policy"]["max_entries"])
            require(members.keys() == expected.keys())
            destination.mkdir(mode=0o755)
            for name in sorted(expected, key=lambda value: (value.count("/"), value)):
                item = expected[name]
                path = destination / name
                if item["kind"] == "directory":
                    if name != ".":
                        path.mkdir(mode=int(item["mode"], 8))
                    path.chmod(int(item["mode"], 8))
                elif item["kind"] == "regular":
                    digest = hashlib.sha256()
                    count = 0
                    with archive.extractfile(members[name]) as entry, path.open("xb") as output:
                        while True:
                            block = entry.read(min(1024 * 1024, item["bytes"] - count + 1))
                            if not block:
                                break
                            count += len(block)
                            require(count <= item["bytes"])
                            digest.update(block)
                            output.write(block)
                    require(count == item["bytes"] and digest.hexdigest() == item["sha256"])
                    path.chmod(int(item["mode"], 8))
            for name, item in expected.items():
                if item["kind"] == "symlink":
                    (destination / name).symlink_to(item["target"])
    validate_jdk(destination, inputs)


def project_records(root):
    entries = tree_entries(root)
    directories = {"."}
    for name in PROJECT_FILES:
        directories.update(str(parent) for parent in PurePosixPath(name).parents)
    require(entries.keys() == directories | set(PROJECT_FILES))
    for name in directories:
        plain_directory(entries[name])
    records = [file_record(entries[name], name) for name in PROJECT_FILES]
    require(all(item["mode"] == "0644" for item in records))
    require(read_bytes(root / "META-INF/MANIFEST.MF", MAX_SOURCE) == MANIFEST)
    return records


def descriptor_for(records):
    return {"schema": "mpk.java.build_inputs.v0", "project_root": PROJECT,
            "toolchain_vector": VECTOR, "toolchain_inputs_sha256": TOOLCHAIN_HASH,
            "candidate_inventory": INVENTORY, "build_recipe": RECIPE, "project_files": records}


def load_descriptor(*, update=False):
    plain_chain(ROOT / PROJECT, ROOT)
    expected = descriptor_for(project_records(ROOT / PROJECT))
    if not update:
        require(load_json(ROOT / DESCRIPTOR, canonical_transport=True) == expected)
    return expected


def validate_active_boundary():
    semantic = load_json(ROOT / "release/bundles/semantic-profile-registry.json")
    baseline = load_json(ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json")["registry"]
    require(semantic == baseline and semantic["revision"] == 3, "JAVA_BUILD_RELEASE_ROUTE")
    registry = load_json(ROOT / "release/bundles/bundle-registry.json")
    require(len(registry["tuples"]) == 5, "JAVA_BUILD_RELEASE_ROUTE")
    require({item["semantic_context"]["source_language"] for item in registry["tuples"]}
            == {"go", "rust", "csharp", "java"}, "JAVA_BUILD_RELEASE_ROUTE")
    java_tuples = [item for item in registry["tuples"]
                   if item["semantic_context"]["source_language"] == "java"]
    require(len(java_tuples) == 1, "JAVA_BUILD_RELEASE_ROUTE")
    require(java_tuples[0]["frontend_bundle_id"] == "frontend.java.java2vir.candidate.v2"
            and java_tuples[0]["toolchain_bundle_id"]
            == "toolchain.java.temurin-25_0_4_1_1.candidate.v1", "JAVA_BUILD_RELEASE_ROUTE")


def execute(argv, *, environment, cwd=None, limit=MAX_REPORT, timeout=BUILD_SECONDS):
    """Bound each pipe while draining both; never accumulate unlimited diagnostics."""
    process = subprocess.Popen(argv, cwd=cwd, env=environment, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    streams = selectors.DefaultSelector()
    streams.register(process.stdout, selectors.EVENT_READ, (bytearray(), limit))
    streams.register(process.stderr, selectors.EVENT_READ, (bytearray(), MAX_STDERR))
    outputs = {key.fileobj: key.data[0] for key in streams.get_map().values()}
    deadline = time.monotonic() + timeout
    try:
        while streams.get_map():
            remaining = deadline - time.monotonic()
            require(remaining > 0, "JAVA_BUILD_TIMEOUT")
            for key, _ in streams.select(min(remaining, 0.25)):
                chunk = os.read(key.fd, 65536)
                if not chunk:
                    streams.unregister(key.fileobj)
                    continue
                data, maximum = key.data
                require(len(data) + len(chunk) <= maximum, "JAVA_BUILD_OUTPUT_LIMIT")
                data.extend(chunk)
        process.wait(timeout=max(0.01, deadline - time.monotonic()))
        return process.returncode, bytes(outputs[process.stdout]), bytes(outputs[process.stderr])
    except BaseException as error:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
        if isinstance(error, subprocess.TimeoutExpired):
            raise BuildFailure("JAVA_BUILD_TIMEOUT") from error
        raise
    finally:
        streams.close()
        process.stdout.close()
        process.stderr.close()


def class_name(name):
    return re.fullmatch(r"mpk/java2vir/[A-Za-z_$][A-Za-z0-9_$]*\.class", name) is not None


def make_jar(classes, manifest=MANIFEST):
    require(manifest == MANIFEST and "mpk/java2vir/Main.class" in classes)
    require(0 < len(classes) <= MAX_CLASSES)
    files = {"META-INF/MANIFEST.MF": manifest, **classes}
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_STORED, allowZip64=False) as jar:
        for name, data in sorted(files.items()):
            if name != "META-INF/MANIFEST.MF":
                require(class_name(name) and 8 <= len(data) <= MAX_CLASS_BYTES)
                require(data[:8] == b"\xca\xfe\xba\xbe\x00\x00\x00\x45")
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | 0o644) << 16
            info.compress_type = zipfile.ZIP_STORED
            jar.writestr(info, data)
            require(output.tell() <= MAX_JAR_BYTES)
    data = output.getvalue()
    require(len(data) <= MAX_JAR_BYTES)
    return data


def inspect_jar(data):
    require(len(data) <= MAX_JAR_BYTES)
    with zipfile.ZipFile(io.BytesIO(data)) as jar:
        entries = jar.infolist()
        require(1 < len(entries) <= MAX_CLASSES + 1)
        require(sum(entry.file_size for entry in entries) <= MAX_JAR_BYTES)
        names = [entry.filename for entry in entries]
        require(names == sorted(set(names)) and names[0] == "META-INF/MANIFEST.MF")
        classes = {}
        for entry in entries:
            require(entry.compress_type == zipfile.ZIP_STORED)
            require(entry.file_size == entry.compress_size <= MAX_CLASS_BYTES)
            require(entry.date_time == (1980, 1, 1, 0, 0, 0) and not entry.is_dir())
            require(entry.create_system == 3 and entry.external_attr == (stat.S_IFREG | 0o644) << 16)
            require(not entry.extra and not entry.comment and entry.flag_bits == 0)
            content = jar.read(entry)
            if entry.filename == "META-INF/MANIFEST.MF":
                require(content == MANIFEST)
            else:
                require(class_name(entry.filename))
                classes[entry.filename] = content
        require(not jar.comment)
    # Also rejects trailing data, alternate headers, duplicate names and ZIP64.
    require(make_jar(classes) == data)
    return classes


def candidate_inventory(jar, descriptor):
    classes = inspect_jar(jar)
    notice = next(item for item in descriptor["project_files"] if item["path"] == "NOTICE.txt")
    return {
        "schema": "mpk.java.frontend_candidate_inventory.v0",
        "toolchain_inputs_sha256": TOOLCHAIN_HASH,
        "project_files_sha256": sha256(canonical(descriptor["project_files"])),
        "build_recipe_sha256": sha256(canonical(RECIPE)),
        "class_files": [record_bytes(name, data) for name, data in sorted(classes.items())],
        "frontend_files": [record_bytes("java2vir.jar", jar)],
        "notice_files": [{**notice, "path": "notices/NOTICE.txt"}],
    }


def load_inventory(descriptor):
    value = load_json(ROOT / INVENTORY, canonical_transport=True)
    exact_keys(value, ("schema", "toolchain_inputs_sha256", "project_files_sha256",
                       "build_recipe_sha256", "class_files", "frontend_files", "notice_files"))
    require(value["schema"] == "mpk.java.frontend_candidate_inventory.v0")
    require(value["toolchain_inputs_sha256"] == TOOLCHAIN_HASH)
    require(value["project_files_sha256"] == sha256(canonical(descriptor["project_files"])))
    require(value["build_recipe_sha256"] == sha256(canonical(RECIPE)))
    for key in ("class_files", "frontend_files", "notice_files"):
        records = value[key]
        require(isinstance(records, list) and 0 < len(records) <= MAX_CLASSES)
        previous = ""
        for item in records:
            exact_keys(item, ("path", "mode", "sha256", "size_bytes"))
            name = relative_path(item["path"])
            require(name > previous and item["mode"] == "0644")
            previous = name
            require(type(item["size_bytes"]) is int and 0 < item["size_bytes"] <= MAX_JAR_BYTES)
            require(re.fullmatch("[0-9a-f]{64}", item["sha256"]) is not None)
            require(key != "class_files" or class_name(name))
    require([item["path"] for item in value["frontend_files"]] == ["java2vir.jar"])
    notice = next(item for item in descriptor["project_files"] if item["path"] == "NOTICE.txt")
    require(value["notice_files"] == [{**notice, "path": "notices/NOTICE.txt"}])
    require(any(item["path"] == "mpk/java2vir/Main.class" for item in value["class_files"]))
    return value


def compile_project():
    """Internal fixed-path container entry; no Docker socket or writable host mount."""
    require(os.getuid() == os.getgid() == 65534 and dict(os.environ) == ENVIRONMENT, "JAVA_BUILD_HOST")
    # Docker deliberately does not expose RLIMIT_AS as a --ulimit type.
    # Apply it before any build-input processing or compiler subprocess.
    address_space = RECIPE["resources"]["address_space_bytes"]
    resource.setrlimit(resource.RLIMIT_AS, (address_space, address_space))
    require(resource.getrlimit(resource.RLIMIT_NOFILE) == (1024, 1024), "JAVA_BUILD_HOST")
    require(resource.getrlimit(resource.RLIMIT_CORE) == (0, 0), "JAVA_BUILD_HOST")
    for control, expected in (("memory.max", "1073741824"), ("memory.swap.max", "0"),
                              ("pids.max", "128")):
        require(Path("/sys/fs/cgroup", control).read_text().strip() == expected, "JAVA_BUILD_HOST")
    status = Path("/proc/self/status").read_text()
    require(re.search(r"^CapEff:\s+0+$", status, re.MULTILINE) is not None, "JAVA_BUILD_HOST")
    require(re.search(r"^NoNewPrivs:\s+1$", status, re.MULTILINE) is not None, "JAVA_BUILD_HOST")
    require([name for _, name in socket.if_nameindex()] == ["lo"], "JAVA_BUILD_HOST")
    inputs = load_json(Path("/mpk/inputs/toolchain.json"))
    validate_toolchain(inputs)
    validate_jdk(Path("/mpk/toolchain/jdk"), inputs)
    for item in inputs["native_inventory"]:
        path = Path("/" + item["path"])
        require(str(path.resolve()) == item["source_path"], "JAVA_BUILD_NATIVE_INPUT")
        record = file_record(path.resolve(), item["path"], MAX_JAR_BYTES)
        require(record == {"path": item["path"], "mode": item["mode"],
                           "size_bytes": item["bytes"], "sha256": item["sha256"]},
                "JAVA_BUILD_NATIVE_INPUT")
    descriptor = load_json(Path("/mpk/inputs/build-inputs.json"), canonical_transport=True)
    require(descriptor == descriptor_for(project_records(Path("/mpk/project"))))
    for directory in ("home", "tmp", "empty", "classes"):
        Path("/work", directory).mkdir()
    sources = [name for name in PROJECT_FILES if name.endswith(".java")]
    argv = ["/mpk/toolchain/jdk/bin/javac", *RECIPE["compiler_jvm_arguments"],
            *COMPILER_ARGUMENTS, *sources]
    code, stdout, stderr = execute(argv, cwd="/mpk/project", environment=ENVIRONMENT, limit=MAX_STDERR)
    require(code == 0 and not stdout and not stderr, "JAVA_BUILD_COMPILER_FAILED")
    classes = {}
    for name, path in tree_entries(Path("/work/classes")).items():
        if stat.S_ISDIR(path.lstat().st_mode):
            continue
        require(class_name(name))
        classes[name] = read_bytes(path, MAX_CLASS_BYTES)
        require(len(classes) <= MAX_CLASSES and sum(map(len, classes.values())) <= MAX_JAR_BYTES)
    jar = make_jar(classes)
    Path("/work/java2vir.jar").write_bytes(jar)
    launch = ["/mpk/toolchain/jdk/bin/java", *JVM_ARGUMENTS, "-cp", "/work/java2vir.jar",
              "mpk.java2vir.Main"]
    code, stdout, stderr = execute([*launch, "--version"], environment=ENVIRONMENT, limit=MAX_STDERR)
    require((code, stdout, stderr) == (0, VERSION, b""), "JAVA_BUILD_IDENTITY")
    for arguments in ([], ["--help"], ["lower", "--source-root", "/unselected/source"],
                      ["--version", "--extra"]):
        code, stdout, stderr = execute([*launch, *arguments], environment=ENVIRONMENT, limit=MAX_STDERR)
        require((code, stdout, stderr) == (2, b"", UNAVAILABLE), "JAVA_BUILD_INACTIVE_BOUNDARY")
    return descriptor, jar


def worker():
    descriptor, jar = compile_project()
    report = {"inventory": candidate_inventory(jar, descriptor),
              "jar_base64": base64.b64encode(jar).decode("ascii")}
    sys.stdout.buffer.write(canonical(report) + b"\n")


def docker_prefix(config):
    executable = next((path for path in ("/usr/bin/docker", "/usr/local/bin/docker")
                       if os.path.isfile(path) and os.access(path, os.X_OK)), None)
    require(executable is not None and Path("/var/run/docker.sock").exists(), "JAVA_BUILD_DOCKER_REQUIRED")
    return [executable, "--config", str(config), "--host", "unix:///var/run/docker.sock"]


def build_once(inputs, descriptor):
    archive = check_cache(inputs)
    with tempfile.TemporaryDirectory(prefix="mpk-java-build-", dir="/tmp") as directory:
        work = Path(directory).resolve()
        # Docker's unprivileged UID must be able to traverse read-only inputs.
        work.chmod(0o755)
        snapshot = work / ARCHIVE_NAME
        copy_verified(archive, snapshot, archive_record(inputs),
                      inputs["archive_policy"]["max_archive_bytes"])
        extract_jdk(snapshot, work / "jdk", inputs)
        project = work / "project"
        project.mkdir()
        for record in descriptor["project_files"]:
            target = project / record["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            plain_chain((ROOT / PROJECT / record["path"]).parent, ROOT / PROJECT)
            copy_verified(ROOT / PROJECT / record["path"], target, record, MAX_SOURCE)
        require(project_records(project) == descriptor["project_files"])
        frozen = work / "inputs"
        frozen.mkdir()
        (frozen / "toolchain.json").write_bytes(canonical(inputs) + b"\n")
        (frozen / "build-inputs.json").write_bytes(canonical(descriptor) + b"\n")
        scripts = work / "build"
        scripts.mkdir()
        implementation = read_bytes(Path(__file__), MAX_SOURCE)
        (scripts / "java_build_inputs.py").write_bytes(implementation)
        config = work / "docker-config"
        config.mkdir(mode=0o700)
        docker = docker_prefix(config)
        name = "mpk-java-build-" + uuid.uuid4().hex
        argv = [*docker, "run", "--rm", "--pull=never", "--platform=linux/amd64",
                "--name", name, "--hostname=mpk-java-build", "--network=none", "--ipc=none",
                "--read-only", "--user=65534:65534", "--cap-drop=ALL",
                "--security-opt=no-new-privileges", "--pids-limit=128",
                "--memory=1073741824", "--memory-swap=1073741824",
                "--ulimit=core=0:0", "--ulimit=nofile=1024:1024",
                "--tmpfs=/work:rw,nosuid,nodev,noexec,size=134217728,uid=65534,gid=65534,mode=0700",
                "--workdir=/work"]
        for source, target in (("jdk", "/mpk/toolchain/jdk"), ("project", "/mpk/project"),
                               ("inputs", "/mpk/inputs"), ("build", "/mpk/build")):
            require("," not in str(work / source))
            argv.extend(["--mount", f"type=bind,src={work / source},dst={target},readonly"])
        argv.extend([IMAGE, "/usr/bin/env", "-i"])
        argv.extend(key + "=" + value for key, value in ENVIRONMENT.items())
        argv.extend(["/usr/local/bin/python3", "-I", "-S", "-B",
                     "/mpk/build/java_build_inputs.py", "_worker"])
        host_environment = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"}
        try:
            code, stdout, stderr = execute(argv, environment=host_environment)
            require(code == 0 and not stderr, "JAVA_BUILD_CONTAINER_FAILED")
            report = strict_json(stdout, maximum=MAX_REPORT, canonical_transport=True)
            exact_keys(report, ("inventory", "jar_base64"))
            jar = base64.b64decode(report["jar_base64"], validate=True)
            require(report["inventory"] == candidate_inventory(jar, descriptor))
            require(project_records(ROOT / PROJECT) == descriptor["project_files"])
            require(read_bytes(Path(__file__), MAX_SOURCE) == implementation)
            return report["inventory"], jar
        finally:
            # Killing the Docker CLI alone does not kill a running container.
            # Removal is scoped to this unpredictable, per-build container name.
            execute([*docker, "rm", "--force", name], environment=host_environment,
                    limit=MAX_STDERR, timeout=30)
            code, remaining, stderr = execute(
                [*docker, "container", "ls", "--all", "--filter", f"name=^/{name}$", "--format", "{{.ID}}"],
                environment=host_environment, limit=MAX_STDERR, timeout=30)
            require(code == 0 and not remaining and not stderr, "JAVA_BUILD_CLEANUP")


def build_twice(inputs, descriptor):
    left = build_once(inputs, descriptor)
    right = build_once(inputs, descriptor)
    require(left[1] == right[1] and canonical(left[0]) == canonical(right[0]),
            "JAVA_BUILD_NONDETERMINISTIC")
    return left


def atomic_json(path, value):
    plain_chain(path.parent, ROOT)
    fd, temporary = tempfile.mkstemp(prefix=".java-inputs-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as output:
            output.write(canonical(value) + b"\n")
            output.flush()
            os.fsync(output.fileno())
        os.chmod(temporary, 0o644)
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def validate_destination(destination_text):
    destination = Path(destination_text)
    require(destination.is_absolute() and not destination.exists() and not destination.is_symlink(),
            "JAVA_BUILD_OUTPUT")
    require(destination.parent.resolve() == destination.parent, "JAVA_BUILD_OUTPUT")
    plain_directory(destination.parent)
    return destination


def export_candidate(destination_text, jar, inventory, descriptor):
    destination = validate_destination(destination_text)
    # Never overwrite an existing directory, including one created concurrently.
    destination.mkdir(mode=0o755)
    try:
        (destination / "java2vir.jar").write_bytes(jar)
        (destination / "build-manifest.json").write_bytes(canonical(inventory) + b"\n")
        (destination / "notices").mkdir(mode=0o755)
        notice = next(item for item in descriptor["project_files"] if item["path"] == "NOTICE.txt")
        copy_verified(ROOT / PROJECT / "NOTICE.txt", destination / "notices/NOTICE.txt", notice, MAX_SOURCE)
    except BaseException:
        shutil.rmtree(destination)
        raise


def main(arguments):
    os.umask(0o022)
    if arguments == ["_worker"]:
        worker()
        return
    if arguments == ["self-test"]:
        # Kept separate from the builder so its adversarial fixtures cannot
        # become project inputs or class-path dependencies.
        import runpy
        runpy.run_path(str(ROOT / "scripts/java_build_inputs_test.py"), run_name="__main__")
        return
    require(arguments and (arguments[0] in ("check", "check-build-inputs", "update-inventory")
                           and len(arguments) == 1
                           or arguments[0] in ("import-build-inputs", "build") and len(arguments) == 2),
            "JAVA_BUILD_USAGE", 64)
    inputs = load_toolchain()
    validate_active_boundary()
    action = arguments[0]
    if action == "build":
        validate_destination(arguments[1])
    if action == "import-build-inputs":
        import_archive(arguments[1], inputs)
        return
    descriptor = load_descriptor(update=action == "update-inventory")
    if action == "check-build-inputs":
        check_cache(inputs)
        load_inventory(descriptor)
        return
    expected = None if action == "update-inventory" else load_inventory(descriptor)
    inventory, jar = build_twice(inputs, descriptor)
    validate_active_boundary()
    if action == "update-inventory":
        parent = (ROOT / DESCRIPTOR).parent
        parent.mkdir(parents=True, exist_ok=True)
        atomic_json(ROOT / DESCRIPTOR, descriptor)
        atomic_json(ROOT / INVENTORY, inventory)
    else:
        require(inventory == expected, "JAVA_BUILD_INVENTORY_MISMATCH")
        if action == "build":
            export_candidate(arguments[1], jar, inventory, descriptor)
    validate_active_boundary()


if __name__ == "__main__":
    try:
        main(sys.argv[1:])
    except BuildFailure as failure:
        print(failure.code, file=sys.stderr)
        raise SystemExit(failure.exit_code)
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError, zipfile.BadZipFile,
            subprocess.SubprocessError, RecursionError):
        print("JAVA_BUILD_INPUTS_INVALID", file=sys.stderr)
        raise SystemExit(65)
