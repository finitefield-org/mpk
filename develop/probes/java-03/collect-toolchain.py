#!/usr/bin/env python3
"""Offline T01 inventory measurement; not a production build/provision command."""

import argparse
import hashlib
import inspect
import json
import os
from pathlib import Path, PurePosixPath
import stat
import struct
import subprocess
import tarfile

IMAGE = "docker.io/library/python@sha256:db8e83a44af476c636a6a753adace39ad37863b63c0afd2862db7bbafeeb3944"
ARCHIVE = "OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz"
ARCHIVE_SHA256 = "dbb698396d478e7fa2b1e50f4103324b2a99b90569ee27c33f2261f9215cf41e"
ARCHIVE_BYTES = 141329719
NATIVE_NAMES = ["libc.so.6", "libm.so.6", "libdl.so.2", "libpthread.so.0", "librt.so.1"]


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def elf_linkage(data):
    if not data.startswith(b"\x7fELF"):
        return None
    if data[4:6] != b"\x02\x01" or struct.unpack_from("<H", data, 18)[0] != 62:
        raise ValueError("expected ELF64 little-endian x86-64")
    offset = struct.unpack_from("<Q", data, 40)[0]
    size, count, _ = struct.unpack_from("<HHH", data, 58)
    sections = [struct.unpack_from("<IIQQQQIIQQ", data, offset + size * i) for i in range(count)]
    ph_offset = struct.unpack_from("<Q", data, 32)[0]
    ph_size, ph_count = struct.unpack_from("<HH", data, 54)
    interpreter = None
    for i in range(ph_count):
        header = struct.unpack_from("<IIQQQQQQ", data, ph_offset + i * ph_size)
        if header[0] == 3:
            interpreter = data[header[2]:header[2] + header[5]].rstrip(b"\0").decode("ascii")
    result = {"needed": [], "rpath": [], "runpath": [], "interpreter": interpreter}
    for section in sections:
        if section[1] != 6:
            continue
        strings_section = sections[section[6]]
        strings = data[strings_section[4]:strings_section[4] + strings_section[5]]
        for index in range(section[4], section[4] + section[5], 16):
            tag, value = struct.unpack_from("<qQ", data, index)
            key = {1: "needed", 15: "rpath", 29: "runpath"}.get(tag)
            if key:
                end = strings.index(b"\0", value)
                result[key].append(strings[value:end].decode("ascii"))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()
    with args.archive.open("rb") as source:
        metadata = os.fstat(source.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size != ARCHIVE_BYTES:
            raise SystemExit("pinned archive size or file type differs")
        # Bound the read even if a local writer grows the file after fstat.
        raw = source.read(ARCHIVE_BYTES + 1)
    if len(raw) != ARCHIVE_BYTES or digest(raw) != ARCHIVE_SHA256:
        raise SystemExit("pinned archive bytes differ")
    inventory = []
    release = {}
    with tarfile.open(args.archive) as archive:
        members = archive.getmembers()
        names = {member.name.rstrip("/") for member in members}
        if len(names) != len(members):
            raise ValueError("duplicate archive member")
        for member in members:
            name = PurePosixPath(member.name)
            if name.is_absolute() or ".." in name.parts or name.parts[0] != "jdk-25.0.4.1+1":
                raise ValueError("invalid archive path")
            relative = str(PurePosixPath(*name.parts[1:]))
            entry = {"path": relative, "mode": f"{member.mode:04o}"}
            if member.isdir():
                entry["kind"] = "directory"
            elif member.isfile():
                content = archive.extractfile(member).read()
                entry.update(kind="regular", bytes=len(content), sha256=digest(content))
                linkage = elf_linkage(content)
                if linkage is not None:
                    entry["elf"] = linkage
                if relative == "release":
                    release = dict(line.split("=", 1) for line in content.decode().splitlines())
                    release = {key: value.strip('"') for key, value in release.items()}
            elif member.issym():
                target = PurePosixPath(member.linkname)
                if target.is_absolute():
                    raise ValueError("absolute archive link")
                parts = list(name.parts[:-1])
                for part in target.parts:
                    if part == "..":
                        if len(parts) <= 1:
                            raise ValueError("escaping archive link")
                        parts.pop()
                    elif part != ".":
                        parts.append(part)
                if "/".join(parts) not in names:
                    raise ValueError("dangling archive link")
                entry.update(kind="symlink", target=member.linkname)
            else:
                raise ValueError("unsupported archive member kind")
            inventory.append(entry)
    inventory.sort(key=lambda value: value["path"])
    # Fixed image, no downloads, no host mounts, no capabilities; stdout only.
    code = "import struct\n" + inspect.getsource(elf_linkage) + "\n" + """import hashlib,json,pathlib,os
names = NAMES
entries=[]
for name in names:
 p=pathlib.Path('/lib/x86_64-linux-gnu')/name
 data=p.read_bytes()
 entries.append(dict(path='lib/x86_64-linux-gnu/'+name,source_path=str(p.resolve()),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),mode=format(p.stat().st_mode & 0o777,'04o'),elf=elf_linkage(data)))
p=pathlib.Path('/lib64/ld-linux-x86-64.so.2'); data=p.read_bytes()
entries.append(dict(path='lib64/ld-linux-x86-64.so.2',source_path=str(p.resolve()),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),mode=format(p.stat().st_mode & 0o777,'04o'),elf=elf_linkage(data)))
print(json.dumps(dict(glibc=os.confstr('CS_GNU_LIBC_VERSION'),files=sorted(entries,key=lambda v:v['path']))))
""".replace("NAMES", repr(NATIVE_NAMES))
    result = subprocess.run(["docker", "run", "--rm", "--pull=never", "--platform=linux/amd64",
                             "--network=none", "--read-only", "--user=65534:65534", "--cap-drop=ALL",
                             "--security-opt=no-new-privileges", IMAGE, "/usr/local/bin/python3", "-c", code],
                            check=True, capture_output=True, text=True, timeout=60)
    native = json.loads(result.stdout)
    if native["glibc"] != "glibc 2.36":
        raise ValueError("unexpected measured libc")
    system_files = [entry for entry in inventory if entry["path"] in ("lib/modules", "lib/ct.sym", "release")]
    jdk_by_path = {entry["path"]: entry for entry in inventory}
    runtime_paths = ["bin/java", "lib/libjava.so", "lib/libjimage.so", "lib/libjli.so",
                     "lib/libnet.so", "lib/libnio.so", "lib/libzip.so", "lib/server/libjvm.so"]
    linkage_files = {"/mpk/toolchain/jdk/" + path: jdk_by_path[path] for path in runtime_paths}
    linkage_files.update({"/" + item["path"]: item for item in native["files"]})
    by_name = {PurePosixPath(path).name: path for path in linkage_files}
    edges = []
    for path, item in sorted(linkage_files.items()):
        for needed in item["elf"]["needed"]:
            if needed not in by_name:
                raise ValueError(f"unresolved runtime dependency: {path}: {needed}")
            edges.append({"from": path, "needed": needed, "to": by_name[needed]})
        interpreter = item["elf"]["interpreter"]
        if interpreter is not None and interpreter not in linkage_files:
            raise ValueError("unresolved ELF interpreter")
    if jdk_by_path["bin/java"]["elf"]["interpreter"] != "/lib64/ld-linux-x86-64.so.2":
        raise ValueError("unexpected measured launcher interpreter")
    value = {
        "schema": "mpk.java.toolchain_inputs.v0",
        "id": "mpk.java.temurin_25_0_4_1_1.linux_x64.v0",
        "compiler_profile_id": "mpk.java.javac_25_0_4_1_1.v0",
        "runtime_profile_id": "mpk.java.hotspot_25_0_4_1_1.v0",
        "system_modules_profile_id": "mpk.java.system_modules_25.v0",
        "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.java25.v0",
        "runtime_layout_profile_id": "mpk.runtime.linux-x86_64-gnu.java25.v0",
        "archive": {"url": "https://github.com/adoptium/temurin25-binaries/releases/download/jdk-25.0.4.1%2B1/" + ARCHIVE,
                    "bytes": ARCHIVE_BYTES, "sha256": ARCHIVE_SHA256,
                    "checksum_url": "https://github.com/adoptium/temurin25-binaries/releases/download/jdk-25.0.4.1%2B1/" + ARCHIVE + ".sha256.txt",
                    "checksum_file_sha256": "72d3771b5f73aa0110de9a51d5ea84f733b95c11b2503491e57324c009132e80",
                    "root": "jdk-25.0.4.1+1"},
        "release_metadata": release,
        "jdk_inventory": inventory,
        "native_image": IMAGE,
        "native_inventory": native["files"],
        "runtime_linkage": {"jdk_runtime_files": runtime_paths, "resolved_needed": edges,
                            "scope": "fixed java launcher and compiler API runtime; other JDK executables/native libraries are not launch targets"},
        "host": {"os": "linux", "architecture": "x86_64", "glibc": "2.36", "minimum_kernel_abi": "6.4.0",
                 "interpreter": "/lib64/ld-linux-x86-64.so.2", "native_library_roots": ["/mpk/toolchain/jdk", "/lib/x86_64-linux-gnu"],
                 "memory_max": 1073741824, "memory_swap_max": 0, "pids_max": 128, "address_space_bytes": 17179869184,
                 "open_files": 1024, "core_bytes": 0, "tmpfs_bytes": 67108864, "timeout_seconds": 120,
                 "tmpfs_mount_flags": ["nosuid", "nodev", "noexec", "noswap"],
                 "namespaces": ["user", "mount", "network", "pid", "ipc", "uts"],
                 "devices": ["/dev/null", "/dev/urandom"], "proc": "readonly_private_pid_namespace"},
        "system_module_inventory": system_files,
        "archive_policy": {"max_archive_bytes": 268435456, "max_extracted_bytes": 1073741824, "max_entries": 2048,
                           "accepted_kinds": ["directory", "regular", "symlink"], "links": "exact_inventory_relative_within_root",
                           "permissions": "exact_regular_directory_modes_no_setid; symlink_archive_modes_are_hashed_but_extracted_link_modes_are_not_access_controls",
                           "unexpected_entry": "reject"},
    }
    value["toolchain_inputs_sha256"] = digest(b"MPK-JAVA-TOOLCHAIN-INPUTS-0.1\0" + canonical(value))
    print(json.dumps(value, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
