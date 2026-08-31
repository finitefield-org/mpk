# Java offline build candidate

`JAVA-03-T02` provides the unregistered `java2vir` build project. Its only
successful invocation is `--version`; every source/frontend invocation exits
2 with `JAVA_FRONTEND_UNAVAILABLE` and no stdout. Parsing, profile admission,
VIR generation and installed execution belong to later Java tasks. The active
Go/Rust/C# release and semantic registry revision 2 are unchanged.

The normative inputs are in
[`JAVA_PROFILE_V0.md`](../develop/specs/JAVA_PROFILE_V0.md) and the Java
conformance vector. The implementation plan is the
[Java ledger](../develop/docs/java-03-implementation-traceability-ledger.md).

## Provisioning

The host needs POSIX, `/usr/bin/python3` 3.9 or later, and Docker at
`/usr/bin/docker` or `/usr/local/bin/docker`, connected through the local
`/var/run/docker.sock`. The builder creates an empty Docker configuration and
does not use ambient contexts, credentials, proxies, Java homes or classpaths.

Provision the exact Temurin archive and Linux x86-64 image separately. Their
URLs, digests and sizes are frozen in
`develop/specs/vectors/java-profile-v0.json`, under `toolchain_inputs`.
The image is:

```text
docker.io/library/python@sha256:db8e83a44af476c636a6a753adace39ad37863b63c0afd2862db7bbafeeb3944
```

An explicit setup step may obtain that image with `docker pull --platform
linux/amd64` and download the archive from its frozen HTTPS URL. Neither the
builder nor any test downloads, resolves a floating version, restores packages
or pulls an image. A missing image is a failed build, never an automatic pull.

Import an already downloaded archive once:

```sh
./scripts/build-java-frontend.sh --import-build-inputs /absolute/path/to/OpenJDK25U-jdk_x64_linux_hotspot_25.0.4.1_1.tar.gz
./scripts/build-java-frontend.sh --check-build-inputs
```

Import accepts only the exact archive bytes, not an executable or extracted
JDK path. It does not replace an existing cache. The ignored cache is
`release/build-input-cache/java/<toolchain_inputs_sha256>/`; keep it outside
version control. Its archive is read-only, checked on every build, and copied
into a new private workspace before extraction. `--check-build-inputs` checks
the archive, project/recipe and candidate metadata without executing a JDK.

## Build and verification

From the repository root:

```sh
./scripts/build-java-frontend.sh --self-test
cargo test -p mpk-cli --test java_build_inputs --offline
./scripts/build-java-frontend.sh --check
mkdir -p target/java-candidates
./scripts/build-java-frontend.sh --build "$(pwd -P)/target/java-candidates/t02"
```

`--check` and `--build` each perform two isolated builds and require identical
JAR and canonical inventory bytes, also matching the checked-in candidate
inventory. `--build` requires a new absolute output directory with an existing
parent and no path aliases. It refuses existing output before starting a
compiler. Output contains:

- `java2vir.jar`: only MPK class files and the fixed manifest.
- `build-manifest.json`: the exact canonical candidate inventory plus LF.
- `notices/NOTICE.txt`: the project notice.

No JDK, native library or external class is repackaged in this candidate.
T07 owns the later runtime bundle and its redistribution notices.

The provisioned integration test exercises that export with hostile Java,
Docker and proxy environment settings. It is ignored by the ordinary test
run so an unprovisioned developer machine never starts Docker implicitly:

```sh
cargo test -p mpk-cli --test java_build_inputs --offline offline_java_candidate_builds_twice_and_refuses_ambient_options -- --ignored --exact
```

After an intentional project change, `--update-inventory` regenerates
`release/build-inputs/java/build-inputs.json` and `candidate-inventory.json`
only after two matching builds. Review source/recipe/class/JAR changes
together and update the test goldens. It is an offline maintenance action,
not an installed release operation. Ordinary builds never rewrite metadata.

## Closed inputs and deterministic output

The build owner is `scripts/java_build_inputs.py`; the shell entry clears the
environment and uses isolated Python without site initialization. There is no
Maven, Gradle, annotation processor, plugin, response file or external library.
Every project file, including the manifest and notice, has an exact path,
size, SHA-256 and mode. Missing, extra, aliased or changed files fail.

The JDK extractor accepts only the T01 member inventory. It rejects traversal,
unexpected/duplicate entries, hard links, special files, escaping links,
permission changes and byte changes. Regular-file and directory modes are
restored exactly; symlink targets are exact while OS-specific symlink mode
bits are ignored, as required by T01. The container rechecks the complete JDK,
project, native library bytes and recipe before invoking the pinned `javac`.

Compilation uses `--release 25`, UTF-8, `-g:none`, `-proc:none`,
`-implicit:none`, warnings as errors, and explicit empty application source,
class, module and processor paths. All selected build sources are supplied in
the frozen order. This compiles the checked-in frontend project, never a
customer's selected source.

JAR entries use ASCII path order, stored compression, timestamp
1980-01-01 00:00:00, Unix regular-file mode 0644, and no extra fields, comments,
directory entries, ZIP64, trailing data, services or `Class-Path`. The manifest
is exact. Class version 69 is required and preview class files reject.
Project/recipe digests are raw SHA-256 over their canonical JSON values;
class/JAR/notice digests are over raw file bytes. These are build metadata,
not new semantic-profile hash domains or proof authority.

Each build has a fresh extracted JDK, source snapshot, temporary filesystem,
Docker configuration and container. The container has no network or writable
host mount, runs as UID/GID 65534 with no capabilities and `no_new_privs`, and
uses bounded memory, PIDs, address space, open files, temporary space, output
and time. Timeout/output failure kills the local subprocess group and removes
the exact build container; absence is checked before returning. The generated
JAR must pass the version/identity check and reject all frontend invocations.

On ARM development hosts Docker may emulate Linux x86-64. This verifies the
pinned build and artifact determinism; it does not establish the complete
native Linux installed sandbox or release gate owned by T07/T09/T10.
