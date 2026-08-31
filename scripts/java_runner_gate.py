#!/usr/bin/env python3
"""Explicit private T07 native gate; never substitutes emulation for evidence."""

import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
import zipfile


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec)
    # Some shared release dataclasses resolve annotations through sys.modules.
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


BUNDLES = module("java_candidate_bundles", "java_release_bundles.py")
BUILD = BUNDLES.BUILD


def trace_evidence(data):
    """Attribute policy installation and thread creation to the actual JVM."""
    events, pending = [], {}
    for line in data.decode("ascii").splitlines():
        match = re.fullmatch(r"([0-9]+)\s+(.+)", line)
        if not match:
            continue
        pid, call = int(match[1]), match[2]
        resumed = re.fullmatch(r"<\.\.\. ([a-z0-9_]+) resumed>(.*)", call)
        if resumed:
            prefix = pending.pop(pid, "")
            BUILD.require(prefix.startswith(resumed[1] + "("), "JAVA_NATIVE_TRACE")
            call = prefix + resumed[2]
        if call.endswith("<unfinished ...>"):
            BUILD.require(pid not in pending, "JAVA_NATIVE_TRACE")
            pending[pid] = call.removesuffix("<unfinished ...>")
        else:
            events.append((pid, call))
    launch = [(index, pid) for index, (pid, call) in enumerate(events)
              if call.startswith('execve("/mpk/toolchain/jdk/bin/java",') and call.endswith(" = 0")]
    BUILD.require(len(launch) == 1 and not pending, "JAVA_NATIVE_TRACE")
    start, java_pid = launch[0]
    before = [call for pid, call in events[:start] if pid == java_pid]
    BUILD.require(any(call.startswith("seccomp(SECCOMP_SET_MODE_FILTER,") and call.endswith(" = 0") for call in before),
                  "JAVA_NATIVE_TRACE")
    for family in ("AF_INET", "AF_UNIX"):
        BUILD.require(any(call.startswith("socket(" + family + ",") and " = -1 EPERM " in call for call in before),
                      "JAVA_NATIVE_TRACE")
    threads = {java_pid}
    clones = []
    for pid, call in events[start + 1:]:
        match = re.fullmatch(r"clone\(.*flags=([^,]+),.*\)\s+= ([1-9][0-9]*)", call)
        if match:
            clones.append((pid, int(match[2]), set(match[1].split("|"))))
    # A child can run before its parent's unfinished clone line is resumed.
    while True:
        expanded = threads | {child for parent, child, _ in clones if parent in threads}
        if expanded == threads:
            break
        threads = expanded
    observed = [flags for parent, _, flags in clones if parent in threads]
    expected = {"CLONE_VM", "CLONE_FS", "CLONE_FILES", "CLONE_SIGHAND", "CLONE_THREAD",
                "CLONE_SYSVSEM", "CLONE_SETTLS", "CLONE_PARENT_SETTID", "CLONE_CHILD_CLEARTID"}
    BUILD.require(observed and all(flags == expected for flags in observed), "JAVA_NATIVE_TRACE")
    BUILD.require(any(pid in threads and call.startswith("clone3(") and " = -1 ENOSYS " in call
                      for pid, call in events[start + 1:]), "JAVA_NATIVE_TRACE")
    return dict(jvm_thread_flags=sorted(expected), jvm_clone3_fallback=True,
                jvm_thread_creations=len(observed), preexec_socket_denials=["AF_INET", "AF_UNIX"])


def check_trace_parser():
    # Transport fixtures test attribution only; they are never native evidence.
    fixture = b'''7 seccomp(SECCOMP_SET_MODE_FILTER, 0, {len=1, filter=0x1}) = 0
7 socket(AF_INET, SOCK_STREAM, 0) = -1 EPERM (Operation not permitted)
7 socket(AF_UNIX, SOCK_STREAM, 0) = -1 EPERM (Operation not permitted)
7 execve("/mpk/toolchain/jdk/bin/java", ["java"], 0x1) = 0
7 clone3({flags=0}, 88) = -1 ENOSYS (Function not implemented)
7 clone(child_stack=0x1, flags=CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID, parent_tid=0x1 <unfinished ...>
7 <... clone resumed>, tls=0x1, child_tidptr=0x1) = 8
'''
    BUILD.require(trace_evidence(fixture)["jvm_thread_creations"] == 1, "JAVA_TRACE_PARSER")
    for changed in (
        fixture.replace(b"7 clone", b"9 clone").replace(b"7 <... clone", b"9 <... clone"),
        fixture.replace(b"7 seccomp", b"9 seccomp"),
        fixture.replace(b"7 socket(AF_UNIX", b"9 socket(AF_UNIX"),
        fixture.replace(b"7 clone3", b"9 clone3"),
        fixture.replace(b"|CLONE_THREAD", b"|CLONE_VFORK"),
        fixture[:fixture.index(b"7 <... clone")],
    ):
        try:
            trace_evidence(changed)
        except BUILD.BuildFailure:
            continue
        raise BUILD.BuildFailure("JAVA_TRACE_PARSER")


def require_native():
    BUILD.require(sys.platform == "linux" and os.uname().machine == "x86_64" and os.geteuid() == 0,
                  "JAVA_NATIVE_HOST_REQUIRED", 69)
    cpu = Path("/proc/cpuinfo").read_text(encoding="ascii")
    BUILD.require(any(line.startswith("flags") and {"lm", "sse2"}.issubset(line.split(':', 1)[1].split())
                      for line in cpu.splitlines()), "JAVA_NATIVE_HOST_REQUIRED", 69)
    BUILD.require(os.access("/sys/fs/cgroup/cgroup.subtree_control", os.W_OK)
                  and Path("/usr/bin/strace").is_file(), "JAVA_NATIVE_PRIMITIVES_REQUIRED", 69)


def writable(directory):
    for root, dirs, _ in os.walk(directory):
        Path(root).chmod(0o700)
        for name in dirs:
            if not Path(root, name).is_symlink():
                Path(root, name).chmod(0o700)


def rewrite(path, data, mode=0o444):
    path.chmod(0o600)
    path.write_bytes(data)
    path.chmod(mode)


def validate_mutations(image, root, check):
    mutation_ids = []
    frontend = f"libexec/mpk/bundles/{BUNDLES.FRONTEND_ID}/java2vir.jar"
    java = f"libexec/mpk/bundles/{BUNDLES.TOOLCHAIN_ID}/jdk/bin/java"
    jvm = f"libexec/mpk/bundles/{BUNDLES.TOOLCHAIN_ID}/jdk/lib/server/libjvm.so"
    for mutation in ("jar-byte", "manifest-classpath", "processor-service", "jdk-byte", "missing-native",
                     "unknown-native", "writable-input", "symlink", "hardlink", "registry-context"):
        changed = root / "changed"
        shutil.copytree(image, changed, symlinks=True)
        try:
            if mutation in ("jar-byte", "jdk-byte"):
                path = changed / (frontend if mutation == "jar-byte" else java)
                data = bytearray(path.read_bytes()); data[-1] ^= 1; rewrite(path, data)
            elif mutation in ("manifest-classpath", "processor-service"):
                path = changed / frontend
                with zipfile.ZipFile(io.BytesIO(path.read_bytes())) as original:
                    output = io.BytesIO()
                    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_STORED) as archive:
                        for name in original.namelist():
                            value = original.read(name)
                            if mutation == "manifest-classpath" and name == "META-INF/MANIFEST.MF":
                                value = b"Manifest-Version: 1.0\r\nClass-Path: /host/poison.jar\r\n\r\n"
                            archive.writestr(name, value)
                        if mutation == "processor-service":
                            archive.writestr("META-INF/services/javax.annotation.processing.Processor", "poison.Processor\n")
                rewrite(path, output.getvalue())
            elif mutation == "writable-input":
                (changed / java).chmod(0o755)
            elif mutation == "registry-context":
                path = changed / "share/mpk/bundle-registry.json"
                value = json.loads(path.read_bytes()); value["profile_registry"]["revision"] = 2
                unsigned = dict(value); unsigned.pop("registry_sha256")
                value["registry_sha256"] = BUILD.sha256(BUNDLES.REGISTRY_DOMAIN + BUILD.canonical(unsigned))
                rewrite(path, BUILD.canonical(value) + b"\n")
            else:
                path = changed / jvm
                path.parent.chmod(0o755)
                if mutation == "unknown-native":
                    (path.parent / "unregistered.so").write_bytes(b"unregistered")
                    (path.parent / "unregistered.so").chmod(0o444)
                else:
                    path.unlink()
                    if mutation == "symlink": path.symlink_to("../libjava.so")
                    elif mutation == "hardlink": os.link(path.parent.parent / "libjava.so", path)
                path.parent.chmod(0o555)
            check(changed)
            mutation_ids.append(mutation)
        finally:
            writable(changed); shutil.rmtree(changed)
    return mutation_ids


def image_gate(image, runner):
    BUILD.require(image.is_absolute() and runner.is_absolute(), "JAVA_IMAGE_GATE_USAGE", 64)
    runner_bytes = BUILD.read_bytes(runner, 512 * 1024 * 1024)
    BUNDLES.check_image(image, runner_bytes)
    def rejects(changed):
        try:
            BUNDLES.check_image(changed, runner_bytes)
        except BUILD.BuildFailure:
            return
        raise BUILD.BuildFailure("JAVA_IMAGE_MUTATION_ACCEPTED")
    with tempfile.TemporaryDirectory(prefix="mpk-java-image-gate-", dir="/tmp") as temporary:
        cases = validate_mutations(image, Path(temporary), rejects)
    BUNDLES.check_image(image, runner_bytes)
    sys.stdout.buffer.write(BUILD.canonical(dict(schema="mpk.java.image_checks.v0", mutations=cases,
        scope="regular-file byte and metadata checks only; no native enforcement claim")) + b"\n")


def main(arguments):
    if arguments == ["--check-trace-parser"]:
        check_trace_parser()
        return
    if len(arguments) == 3 and arguments[0] == "--image":
        return image_gate(Path(arguments[1]), Path(arguments[2]))
    BUILD.require(len(arguments) == 2, "JAVA_NATIVE_GATE_USAGE", 64)
    require_native()  # No image/source reads before the native host admission.
    image, runner = map(Path, arguments)
    BUILD.require(image.is_absolute() and runner.is_absolute(), "JAVA_NATIVE_GATE_USAGE", 64)
    runner_bytes = BUILD.read_bytes(runner, 512 * 1024 * 1024)
    BUNDLES.check_image(image, runner_bytes)
    release = module("java_native_release_owner", "release_bundles.py")
    executable = image / "bin/mpk"
    def run(case, hostile=False):
        command = release.rust_fixture_cgroup_command([str(executable), "--native-hostile-case" if hostile else "--native-case", case])
        result = release.run_bounded_rust_fixture(command, cwd=image, env={})
        BUILD.require(result.returncode == 0 and not result.stderr, "JAVA_NATIVE_RUN")
        envelope = BUILD.strict_json(result.stdout, maximum=268_435_456, canonical_transport=True)
        BUILD.require(envelope["status"] == "ir-lowered", "JAVA_NATIVE_ENVELOPE")
        return result.stdout

    baseline = run("int.identity")
    BUILD.require(run("int.identity", hostile=True) == baseline, "JAVA_NATIVE_AMBIENT")
    reports = {"int.identity": BUILD.sha256(baseline)}
    for case in ("int.division", "int.shift_unsigned_right"):
        reports[case] = BUILD.sha256(run(case))
    # A complete image without a delegated cgroup must fail before source.
    code, stdout, _stderr = BUILD.execute([str(executable), "--native-case", "int.identity"],
        environment={}, cwd=image, timeout=120)
    BUILD.require(code != 0 and not stdout, "JAVA_NATIVE_CGROUP_REQUIRED")

    faults = ["oom", "pids", "timeout", "stdout", "stderr", "tmpfs"]
    for case in faults:
        command = release.rust_fixture_cgroup_command([str(executable), "--native-resource", case])
        result = release.run_bounded_rust_fixture(command, cwd=image, env={})
        BUILD.require(result.returncode == 0 and not result.stdout and not result.stderr, "JAVA_NATIVE_RESOURCE_FAULT")

    mutation_ids = []
    with tempfile.TemporaryDirectory(prefix="mpk-java-native-gate-", dir="/tmp") as temporary:
        root = Path(temporary)
        # Record the actual native JVM thread/syscall behavior. The production
        # execution above is untraced and must already pass the same policy.
        trace = root / "syscalls.log"
        command = release.rust_fixture_cgroup_command([str(executable), "--native-case", "int.identity"])
        traced = release.run_bounded_rust_fixture(["/usr/bin/strace", "-f", "-qq", "-s", "256", "-o", str(trace),
            "-e", "trace=execve,execveat,clone,clone3,seccomp,prctl,capset,socket,unshare", *command], cwd=image, env={})
        BUILD.require(traced.returncode == 0 and not traced.stderr and traced.stdout == baseline, "JAVA_NATIVE_TRACE")
        trace_bytes = BUILD.read_bytes(trace, 8 * 1024 * 1024)
        observed_trace = trace_evidence(trace_bytes)
        def native_rejection(changed):
            result = release.run_bounded_rust_fixture([str(changed / "bin/mpk"), "--release-before-source"], cwd=changed, env={})
            BUILD.require(result.returncode == 0 and not result.stdout and not result.stderr, "JAVA_NATIVE_MUTATION")
        mutation_ids = validate_mutations(image, root, native_rejection)
        report = dict(schema="mpk.java.native_runner_report.v0", native_architecture="x86_64",
            release_registry_sha256=BUNDLES.descriptors()[1]["registry_sha256"],
            runner_sha256=BUILD.sha256(runner_bytes),
            cases=reports, hostile_environment_equal=True, resource_faults=faults, source_precedence_mutations=mutation_ids,
            syscall_trace_sha256=BUILD.sha256(trace_bytes),
            syscall_observation=observed_trace,
            scope="T07 candidate runner; complete release rehearsal and activation remain T09/T10")
        sys.stdout.buffer.write(BUILD.canonical(report) + b"\n")


if __name__ == "__main__":
    try:
        main(sys.argv[1:])
    except BUILD.BuildFailure as error:
        print(error.code, file=sys.stderr)
        sys.exit(error.exit_code)
