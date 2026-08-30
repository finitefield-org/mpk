# JAVA-03-T01 toolchain and compiler probes

These disposable developer probes measure the pinned Java 25 public compiler
API and JVM compatibility with a small, read-only Linux filesystem. They are
not the Java frontend or an installed release gate. The recorded 2026-08-31
Linux x86-64 run on an ARM Mac uses OrbStack emulation. Normative input bytes
and observations are in `../../specs/vectors/java-profile-v0.json`.

Prerequisites are Python 3.12 or later, Docker with Linux x86-64 execution,
and the exact JDK archive and container image recorded in `toolchain_inputs`.
Provision those two downloads separately. None of these scripts downloads or
pulls an image. From the repository root, collect the full archive/native
inventory; an unexpected archive digest or unavailable image fails closed:

```sh
mkdir -p /tmp/mpk-java-t01
python3 develop/probes/java-03/collect-toolchain.py /absolute/path/to/OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz > /tmp/mpk-java-t01/toolchain.json
```

The collector verifies the pinned archive size/SHA-256 before reading members,
checks paths and relative links, and records every file, directory, mode and
link. It also measures ELF `PT_INTERP`, `DT_NEEDED`, `RPATH`/`RUNPATH`, the six
native files from the fixed image, and the launcher's resolved native linkage.
It does not run any JDK executable. Compare its JSON value with the vector's
`toolchain_inputs` before executing the probes.

Extract into a new directory only, with Python's `tarfile` data filter. That
filter adds owner write permission to some read-only files, so restore the
verified inventory's exact regular-file/directory modes afterward. For
example, after successful collection with the archive at the path below:

```sh
python3 - <<'PY'
import hashlib, json, stat, tarfile
from pathlib import Path
archive = Path('/tmp/mpk-java-t01/OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz')
inputs = json.loads(Path('/tmp/mpk-java-t01/toolchain.json').read_text())
with archive.open('rb') as stream:
    assert hashlib.file_digest(stream, 'sha256').hexdigest() == inputs['archive']['sha256']
destination = Path('/tmp/mpk-java-t01/extracted')
destination.mkdir()  # fail rather than overwrite an existing tree
with tarfile.open(archive) as stream:
    stream.extractall(destination, filter='data')
root = destination / inputs['archive']['root']
for item in inputs['jdk_inventory']:
    path = root / item['path']
    mode = path.lstat().st_mode
    if item['kind'] == 'regular':
        assert stat.S_ISREG(mode)
        path.chmod(int(item['mode'], 8))
    elif item['kind'] == 'directory':
        assert stat.S_ISDIR(mode)
        path.chmod(int(item['mode'], 8))
PY
python3 develop/probes/java-03/check-jdk.py /tmp/mpk-java-t01/extracted/jdk-25.0.4.1+1
```

The checker binds the extracted tree to the checked-in inventory: exact
membership, types, regular-file sizes/hashes/modes, directory modes, and link
targets; hard-link aliases reject. Archive symlink modes remain hashed, but
extracted symlink permission bits are not compared because they do not control
POSIX access and differ between macOS and Linux extraction. Probe wrappers
repeat this check before mounting the JDK read-only. This is a trusted local
developer check; production immutable capture and race resistance belong to
the later build/runner tasks.

For a compiler-only container run:

```sh
sh develop/probes/java-03/run-api-probe.sh /absolute/path/to/jdk-25.0.4.1+1 > /tmp/mpk-java-t01/api-probe.json
```

This script mounts only the pinned JDK
and this probe directory read-only, disables networking and capabilities, runs as
UID/GID 65534, and bounds memory, PIDs and temporary storage. Bootstrap `javac`
and `jar` compile only checked-in probe and deliberately planted fixture files.
Each measured request then creates a fresh `JavacTask`, calls only `parse()` and
`analyze()`, closes the file manager, and observes any attempted class output.
The selected source is never compiled to bytecode or executed.

The JSON retains exact public tree kind/spelling/positions before and after
analysis, type and symbol observations, diagnostic codes and positions, file
manager access counts, external source/class/processor refusal, and the first
excess diagnostic listener exception. The wrapper is a measured design sketch;
the fixed `--release 25` option causes javac to use a separate platform file
manager, whose JDK reference lookups do not pass through the wrapper. The
application lookup boundary and the pinned host/JDK reference boundary must be
validated separately. Unwrapped control tasks demonstrate that the planted
source and class are otherwise resolvable with the same compiler options.
Production closure, bounded traversal and diagnostic normalization belong to
T04. Source strings and compiler observations here are developer fixtures, not
public artifacts or diagnostics. A failing or partial run must not be treated as
successful evidence.

For the recorded minimal-filesystem JVM and compiler runs:

```sh
sh develop/probes/java-03/run-runtime-probe.sh /absolute/path/to/jdk-25.0.4.1+1 runtime > /tmp/mpk-java-t01/runtime-result.json
sh develop/probes/java-03/run-runtime-probe.sh /absolute/path/to/jdk-25.0.4.1+1 compiler > /tmp/mpk-java-t01/api-chroot-result.json
```

This wrapper starts a disposable container with no network, read-only host
mounts, a 1 GiB memory limit, zero swap and 128 PIDs. Setup uses only
`SYS_ADMIN`, `SETUID`, `SETGID` and `SYS_CHROOT` to create its private mounts;
it never uses a privileged container or host PID/network namespace. The
measured JVM runs as UID/GID 65534, with zero effective capabilities and
`no_new_privs`, in a read-only chroot containing the JDK, six native libraries,
checked-in probe classes/hostile fixtures, read-only private proc, `/dev/null`,
`/dev/urandom`, and a 64 MiB `noswap,nosuid,nodev,noexec` temporary filesystem.
It has no host home, credentials, shell or writable executable input. JVM
options use the frozen interpreter-only, CDS-off, Serial-GC baseline.

`RuntimeProbe` asserts 15 compatibility properties, including the runtime pin,
closed environment, privilege drop, read-only mounts, non-executable/no-swap
temporary storage, loopback-only interfaces and bounded heap. The compiler
probe asserts 20 API cases, planted source/class control cases, and 15 closed
file-manager operations. It retains 1,024 diagnostic callbacks and aborts on
the first excess callback. Class/default-constructor and inferred-`var`
observations establish why known excluded raw forms must reject before
accepted-subtree transformation comparisons. Neither probe generates or
executes selected source.
Two consecutive minimal-root runs produced identical JSON bytes.

The outer seccomp setting is unconfined for disposable mount setup. The harness
uses probe main classes and `/mpk/frontend` class files with working directory
`/`, rather than the future frontend JAR and `/mpk/source`. Integrated user
namespace bootstrap, native x86-64 syscall/clone3 tracing, production seccomp,
resource exhaustion/timeout/descendant cleanup and the complete installed
runner are **not** established by these observations. T07 implements and
tests those enforcement paths; T09/T10 run the complete native Linux release
gates. Do not promote this emulated compatibility result to release acceptance.
