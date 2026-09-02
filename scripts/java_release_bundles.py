#!/usr/bin/env python3
"""Private T07 Java candidate descriptors; never edits the active release."""

import importlib.util
import base64
import os
from pathlib import Path
import posixpath
import sys
import tempfile
import uuid
import stat

SPEC = importlib.util.spec_from_file_location("java_build_owner", Path(__file__).with_name("java_build_inputs.py"))
BUILD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILD)

FRONTEND_ID = "frontend.java.java2vir.candidate.v2"
TOOLCHAIN_ID = "toolchain.java.temurin-25_0_4_1_1.candidate.v1"
HOST_ID = "mpk.host.linux-x86_64-gnu.java25.v0"
LAYOUT_ID = "mpk.runtime.linux-x86_64-gnu.java25.v0"
CANDIDATE = "release/build-inputs/java/bundle-candidate.json"
REGISTRY = "release/build-inputs/java/bundle-registry.json"
CONTENT_DOMAIN = b"MPK-BUNDLE-CONTENT-0.1\0"
REGISTRY_DOMAIN = b"MPK-BUNDLE-REGISTRY-1.0\0"
HOST_ENVIRONMENT = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"}


def inventory(kind, bundle, files, component=None):
    scope = dict(kind=kind, bundle_id=bundle)
    if component is not None:
        scope["component_name"] = component
    return dict(schema="mpk.release.bundle_inventory.v0", scope=scope, files=sorted(files, key=lambda row: row["path"]))


def content(name, release, files):
    value = inventory("component", TOOLCHAIN_ID, files, name)
    return dict(kind="content", name=name, release=release, inventory=value,
                content_sha256=BUILD.sha256(CONTENT_DOMAIN + BUILD.canonical(value)))


def jdk_file_records(inputs):
    """Resolve only frozen archive links; emitted bundle files are all regular."""
    entries = {row["path"]: row for row in inputs["jdk_inventory"]}
    result = []
    for name, row in sorted(entries.items()):
        if row["kind"] == "directory":
            continue
        target, visited = name, set()
        while entries[target]["kind"] == "symlink":
            BUILD.require(target not in visited, "JAVA_CANDIDATE_LINK")
            visited.add(target)
            target = posixpath.normpath(posixpath.join(posixpath.dirname(target), entries[target]["target"]))
            BUILD.require(target in entries and not target.startswith("../"), "JAVA_CANDIDATE_LINK")
        source = entries[target]
        BUILD.require(source["kind"] == "regular", "JAVA_CANDIDATE_LINK")
        result.append((dict(path="jdk/" + name, executable=bool(int(source["mode"], 8) & 0o111),
                            size_bytes=source["bytes"], sha256=source["sha256"]), target))
    return result


def descriptors():
    BUILD.validate_active_boundary()
    inputs = BUILD.load_toolchain()
    descriptor = BUILD.load_descriptor()
    measured = BUILD.load_inventory(descriptor)
    vector = BUILD.load_json(BUILD.ROOT / BUILD.VECTOR)
    semantic = BUILD.load_json(BUILD.ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json")["registry"]
    contracts = {row["field"]: row["envelope"] for row in vector["profile_contracts"]}
    frontend_files = [dict(path=row["path"], executable=False, size_bytes=row["size_bytes"], sha256=row["sha256"])
                      for row in measured["frontend_files"]]
    notice = next(row for row in descriptor["project_files"] if row["path"] == "NOTICE.txt")
    frontend_files.append(dict(path="NOTICE.txt", executable=False, size_bytes=notice["size_bytes"], sha256=notice["sha256"]))
    frontend_inventory = inventory("frontend_bundle", FRONTEND_ID, frontend_files)
    frontend = dict(schema="mpk.release.frontend_bundle.v1", bundle_id=FRONTEND_ID, name="java2vir", version="0.1.0",
                    profile_contracts=[contracts["frontend"]],
                    main=dict(name="java2vir", version="0.1.0", path="java2vir.jar",
                              binary_sha256=measured["frontend_files"][0]["sha256"], runtime=dict(kind="static")),
                    subordinate_binaries=[], inventory=frontend_inventory,
                    bundle_sha256=BUILD.sha256(CONTENT_DOMAIN + BUILD.canonical(frontend_inventory)))
    jdk = [row for row, _ in jdk_file_records(inputs)]
    native = [dict(path="native-runtime/" + row["path"], executable=bool(int(row["mode"], 8) & 0o111),
                   size_bytes=row["bytes"], sha256=row["sha256"]) for row in inputs["native_inventory"]]
    java = next(row for row in jdk if row["path"] == "jdk/bin/java")
    runtime = dict(kind="dynamic", interpreter_mount=inputs["host"]["interpreter"], libraries=[
        dict(soname=Path(row["path"]).name, component_path=row["path"], sha256=row["sha256"])
        for row in sorted(inputs["native_inventory"], key=lambda row: Path(row["path"]).name)
        if not row["path"].startswith("lib64/")])
    components = [dict(kind="executable", name="java", release="25.0.4.1+1", path=java["path"],
                       binary_sha256=java["sha256"], runtime=runtime),
                  content("jdk", "25.0.4.1+1", [row for row in jdk if row != java]),
                  content("native-runtime", "glibc-2.36", native)]
    toolchain_inventory = inventory("toolchain_bundle", TOOLCHAIN_ID, jdk + native)
    toolchain = dict(schema="mpk.release.toolchain_bundle.v1", bundle_id=TOOLCHAIN_ID,
                     execution_host_profile_id=HOST_ID, profile_contracts=[contracts["release"]],
                     components=components, inventory=toolchain_inventory,
                     distribution_sha256=BUILD.sha256(CONTENT_DOMAIN + BUILD.canonical(toolchain_inventory)))
    host = dict(id=HOST_ID, os="linux", architecture="x86_64", abi="glibc-2.36", minimum_kernel_abi="6.4.0",
                probe_profile_id="mpk.release.probe.java25.v0", required_primitives=sorted([
                    "cgroup2.atomic_clone3", "cgroup2.descendant_cleanup", "cgroup2.memory_1073741824",
                    "cgroup2.pids_128", "cgroup2.swap_0", "fs.readonly_inputs", "fs.tmpfs_67108864_noswap",
                    "linux.namespaces", "process.closed_environment", "process.no_new_privileges",
                    "process.no_network", "process.nonroot_no_capabilities", "process.rlimit_as_17179869184",
                    "process.rlimit_core_0", "process.rlimit_nofile_1024", "process.seccomp_java_threads",
                    "process.timeout_120", "proc.readonly_private_pid_namespace"]))
    layout = dict(id=LAYOUT_ID, execution_host_profile_id=HOST_ID, runtime_root="/mpk/native-runtime",
                  interpreter_mounts=[dict(component_path="lib64/ld-linux-x86-64.so.2", sandbox_path="/lib64/ld-linux-x86-64.so.2")],
                  library_mounts=[dict(component_path="lib/x86_64-linux-gnu", sandbox_path="/lib/x86_64-linux-gnu")],
                  loader_search_paths=["/lib/x86_64-linux-gnu"], forbidden_host_roots=["/lib", "/lib64", "/usr/lib"])
    identity = {key: semantic[key] for key in ("schema", "id", "revision", "registry_sha256")}
    candidate = dict(schema="mpk.release.bundle_candidate.v1", profile_registry=identity,
                     execution_host_profiles=[host], native_runtime_layout_profiles=[layout],
                     frontend_bundles=[frontend], toolchain_bundles=[toolchain], tuples=[dict(
                         semantic_context=vector["semantic_context_fixture"], limit_profile_id="mpk.vir.limits.v0",
                         frontend_bundle_id=FRONTEND_ID, toolchain_bundle_id=TOOLCHAIN_ID)])
    registry = dict(candidate, schema="mpk.release.bundle_registry.v1", id="mpk.release.registry.v1")
    registry["registry_sha256"] = BUILD.sha256(REGISTRY_DOMAIN + BUILD.canonical(registry))
    return candidate, registry


def native_bytes(inputs):
    """Read only the six frozen files from the already provisioned image."""
    worker = """import base64,hashlib,json,os,stat,sys
result={}
for row in json.loads(sys.argv[1]):
    path=row['source_path']
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW)
    with os.fdopen(fd,'rb') as stream:
        before=os.fstat(stream.fileno())
        assert stat.S_ISREG(before.st_mode) and before.st_size==row['bytes']
        assert format(stat.S_IMODE(before.st_mode),'04o')==row['mode']
        data=stream.read(row['bytes']+1)
        after=os.fstat(stream.fileno())
        assert all(getattr(before,key)==getattr(after,key) for key in ('st_dev','st_ino','st_size','st_mode','st_mtime_ns','st_ctime_ns'))
    assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
    result[row['path']]=base64.b64encode(data).decode('ascii')
print(json.dumps(result,sort_keys=True,separators=(',',':')))
"""
    with tempfile.TemporaryDirectory(prefix="mpk-java-native-", dir="/tmp") as directory:
        docker = BUILD.docker_prefix(Path(directory))
        name = "mpk-java-native-" + uuid.uuid4().hex
        try:
            code, stdout, stderr = BUILD.execute([
                *docker, "run", "--rm", "--pull=never", "--platform=linux/amd64", "--name", name,
                "--network=none", "--ipc=none", "--read-only", "--user=65534:65534", "--cap-drop=ALL",
                "--security-opt=no-new-privileges", "--pids-limit=16", "--memory=134217728",
                "--memory-swap=134217728", inputs["native_image"], "/usr/bin/env", "-i",
                "/usr/local/bin/python3", "-I", "-S", "-B", "-c", worker,
                BUILD.canonical(inputs["native_inventory"]).decode("ascii")],
                environment=HOST_ENVIRONMENT, limit=16 * 1024 * 1024, timeout=120)
            BUILD.require(code == 0 and not stderr, "JAVA_CANDIDATE_NATIVE")
            report = BUILD.strict_json(stdout, maximum=16 * 1024 * 1024, canonical_transport=True)
            BUILD.require(set(report) == {row["path"] for row in inputs["native_inventory"]}, "JAVA_CANDIDATE_NATIVE")
            result = {}
            for row in inputs["native_inventory"]:
                data = base64.b64decode(report[row["path"]], validate=True)
                BUILD.require(len(data) == row["bytes"] and BUILD.sha256(data) == row["sha256"], "JAVA_CANDIDATE_NATIVE")
                result["native-runtime/" + row["path"]] = data
            return result
        finally:
            BUILD.execute([*docker, "rm", "--force", name], environment=HOST_ENVIRONMENT, timeout=30)
            code, remaining, stderr = BUILD.execute([*docker, "container", "ls", "--all", "--filter",
                f"name=^/{name}$", "--format", "{{.ID}}"], environment=HOST_ENVIRONMENT, timeout=30)
            BUILD.require(code == 0 and not remaining and not stderr, "JAVA_CANDIDATE_CLEANUP")


def put(path, data, executable=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(data)
    path.chmod(0o555 if executable else 0o444)


def check_image(root, runner_bytes):
    """Exact regular-file tree, byte identities, modes and absence of aliases."""
    candidate, registry = descriptors()
    semantic = BUILD.load_json(BUILD.ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json")["registry"]
    expected = {
        "bin/mpk": dict(size_bytes=len(runner_bytes), sha256=BUILD.sha256(runner_bytes), executable=True),
        "share/mpk/bundle-registry.json": dict(size_bytes=len(BUILD.canonical(registry)) + 1,
            sha256=BUILD.sha256(BUILD.canonical(registry) + b"\n"), executable=False),
        "share/mpk/semantic-profile-registry.json": dict(size_bytes=len(BUILD.canonical(semantic)) + 1,
            sha256=BUILD.sha256(BUILD.canonical(semantic) + b"\n"), executable=False),
    }
    for bundle in candidate["frontend_bundles"] + candidate["toolchain_bundles"]:
        for row in bundle["inventory"]["files"]:
            expected[f"libexec/mpk/bundles/{bundle['bundle_id']}/{row['path']}"] = row
    directories = {"."}
    for name in expected:
        directories.update(str(path) for path in Path(name).parents)
    observed = set()
    identities = set()
    for directory, dirs, files in os.walk(root, followlinks=False):
        for name in [".", *dirs, *files]:
            path = Path(directory) if name == "." else Path(directory) / name
            relative = str(path.relative_to(root))
            metadata = path.lstat()
            BUILD.require(not stat.S_ISLNK(metadata.st_mode), "JAVA_CANDIDATE_IMAGE")
            if stat.S_ISDIR(metadata.st_mode):
                BUILD.require(relative in directories and stat.S_IMODE(metadata.st_mode) == 0o555, "JAVA_CANDIDATE_IMAGE")
                continue
            BUILD.require(relative in expected and relative not in observed, "JAVA_CANDIDATE_IMAGE")
            row = expected[relative]
            identity = (metadata.st_dev, metadata.st_ino)
            BUILD.require(stat.S_ISREG(metadata.st_mode) and metadata.st_nlink == 1 and identity not in identities,
                          "JAVA_CANDIDATE_IMAGE")
            identities.add(identity)
            BUILD.require(stat.S_IMODE(metadata.st_mode) == (0o555 if row["executable"] else 0o444), "JAVA_CANDIDATE_IMAGE")
            data = BUILD.read_bytes(path, row["size_bytes"])
            BUILD.require(len(data) == row["size_bytes"] and BUILD.sha256(data) == row["sha256"], "JAVA_CANDIDATE_IMAGE")
            observed.add(relative)
    BUILD.require(observed == set(expected), "JAVA_CANDIDATE_IMAGE")


def assemble(destination_text, runner_text):
    destination = BUILD.validate_destination(destination_text)
    runner = Path(runner_text)
    BUILD.require(runner.is_absolute(), "JAVA_CANDIDATE_RUNNER")
    BUILD.plain_chain(runner.parent, Path(runner.anchor))
    runner_bytes = BUILD.read_bytes(runner, 512 * 1024 * 1024)
    BUILD.require(runner_bytes[:6] == b"\x7fELF\x02\x01" and runner_bytes[18:20] == b"\x3e\x00", "JAVA_CANDIDATE_RUNNER")
    candidate, registry = descriptors()
    for name, value in ((CANDIDATE, candidate), (REGISTRY, registry)):
        BUILD.require(BUILD.load_json(BUILD.ROOT / name, canonical_transport=True) == value, "JAVA_CANDIDATE_DESCRIPTOR")
    inputs, descriptor = BUILD.load_toolchain(), BUILD.load_descriptor()
    measured, jar = BUILD.build_twice(inputs, descriptor)
    BUILD.require(measured == BUILD.load_inventory(descriptor), "JAVA_CANDIDATE_DESCRIPTOR")
    native = native_bytes(inputs)
    with tempfile.TemporaryDirectory(prefix="mpk-java-image-", dir="/tmp") as temporary:
        jdk = Path(temporary) / "jdk"
        BUILD.extract_jdk(BUILD.check_cache(inputs), jdk, inputs)
        # Reserve the requested destination exclusively, initially with 0700.
        # Only the final whole-image check certifies this output; a failed
        # assembly must not be used, and no existing output is replaced.
        destination.mkdir(mode=0o700)
        put(destination / "bin/mpk", runner_bytes, True)
        put(destination / "share/mpk/bundle-registry.json", BUILD.canonical(registry) + b"\n")
        semantic = BUILD.load_json(BUILD.ROOT / "develop/specs/vectors/semantic-profile-registry-v3.json")["registry"]
        put(destination / "share/mpk/semantic-profile-registry.json", BUILD.canonical(semantic) + b"\n")
        frontend = destination / "libexec/mpk/bundles" / FRONTEND_ID
        put(frontend / "java2vir.jar", jar)
        put(frontend / "NOTICE.txt", BUILD.read_bytes(BUILD.ROOT / BUILD.PROJECT / "NOTICE.txt", BUILD.MAX_SOURCE))
        toolchain = destination / "libexec/mpk/bundles" / TOOLCHAIN_ID
        for row, target in jdk_file_records(inputs):
            data = BUILD.read_bytes(jdk / target, row["size_bytes"])
            BUILD.require(len(data) == row["size_bytes"] and BUILD.sha256(data) == row["sha256"], "JAVA_CANDIDATE_JDK")
            put(toolchain / row["path"], data, row["executable"])
        for row in candidate["toolchain_bundles"][0]["inventory"]["files"]:
            if row["path"] in native:
                put(toolchain / row["path"], native[row["path"]], row["executable"])
        for directory, _, _ in os.walk(destination, topdown=False):
            Path(directory).chmod(0o555)
    check_image(destination, runner_bytes)
    BUILD.validate_active_boundary()
    print(BUILD.canonical(dict(schema="mpk.java.candidate_image.v0", release_registry_sha256=registry["registry_sha256"],
        runner_sha256=BUILD.sha256(runner_bytes), frontend_sha256=BUILD.sha256(jar),
        toolchain_distribution_sha256=candidate["toolchain_bundles"][0]["distribution_sha256"])).decode("ascii"))


def main(arguments):
    if len(arguments) == 3 and arguments[0] == "assemble":
        assemble(arguments[1], arguments[2])
        return
    if len(arguments) == 3 and arguments[0] == "check-image":
        check_image(Path(arguments[1]), BUILD.read_bytes(Path(arguments[2]), 512 * 1024 * 1024))
        return
    BUILD.require(arguments in (["check"], ["update-descriptors"]), "JAVA_CANDIDATE_USAGE", 64)
    values = descriptors()
    for name, value in zip((CANDIDATE, REGISTRY), values):
        if arguments == ["update-descriptors"]:
            BUILD.atomic_json(BUILD.ROOT / name, value)
        else:
            BUILD.require(BUILD.load_json(BUILD.ROOT / name, canonical_transport=True) == value, "JAVA_CANDIDATE_DESCRIPTOR")
    BUILD.validate_active_boundary()


if __name__ == "__main__":
    try:
        os.umask(0o022)
        main(sys.argv[1:])
    except (BUILD.BuildFailure, OSError, ValueError, KeyError, TypeError) as error:
        print(error.code if isinstance(error, BUILD.BuildFailure) else "JAVA_CANDIDATE_INVALID", file=sys.stderr)
        sys.exit(error.exit_code if isinstance(error, BUILD.BuildFailure) else 65)
