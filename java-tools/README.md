# Java offline frontend candidate

`JAVA-03-T02` provides the inactive `java2vir` build project. T07 adds a
private registered candidate and the fixed JVM entrypoint; malformed invocations
still exit 2 with `JAVA_FRONTEND_UNAVAILABLE` and no stdout. T03 adds independent Rust
contract/artifact validators. T04 adds internal immutable capture, strict
UTF-8 transport, public javac parse/analyze sessions and bounded failure
diagnostics. T05 adds source subset admission, inert initialization, conservative
acyclic call closure and typed contract sidecars, exercised through separately
compiled private test harnesses. T06 adds private CFG/lowering, original-byte
source maps and deterministic complete artifacts. T07's installed runner is
implemented, but its native x86-64 Linux acceptance gate has not run on the ARM
macOS development host. That gate remains necessary before T07 is accepted.
The active Go/Rust/C# release and semantic registry revision 2 are unchanged.

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
./scripts/build-java-frontend.sh --build "$(pwd -P)/target/java-candidates/frontend"
```

`--check` and `--build` each perform two isolated builds and require identical
JAR and canonical inventory bytes, also matching the checked-in candidate
inventory. `--build` requires a new absolute output directory with an existing
parent and no path aliases. It refuses existing output before starting a
compiler. Output contains:

- `java2vir.jar`: only MPK class files and the fixed manifest.
- `build-manifest.json`: the exact canonical candidate inventory plus LF.
- `notices/NOTICE.txt`: the project notice.

This frontend-only export contains no JDK or native library. T07's separate
candidate-image builder below includes the frozen JDK and native closure.

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

## Capture and compiler adapter verification

```sh
./scripts/check-java-frontend.sh --check-fixtures
cargo test -p mpk-cli --test java_frontend_vectors --offline
./scripts/check-java-frontend.sh --run
cargo test -p mpk-cli --test java_frontend_vectors --offline pinned_java_capture_compiler_and_diagnostic_vectors_execute -- --ignored --exact
```

`--check-fixtures` needs no JDK or Docker. It checks closed vector ownership
and recomputes every recorded tree spelling and UTF-16/UTF-8 coordinate from
the fixture's original bytes. The Rust tests independently reject unknown
Java issue codes, messages, phases, statuses, exits, crossed identities and
partial/noncanonical failure responses.

`--run` uses the same provisioned archive, pinned image, scrubbed environment
and offline project compiler as the build owner. It builds the current project
without rewriting inventories and compiles `java-tools/tests/FrontendTests.java`
separately; that test class and planted dependencies never enter `java2vir.jar`.
The canonical test report includes all 20 compiler observations, 17 T04
rejections, file-manager boundaries, counter checks and artifact-free failures.
Full recorded trees, types, elements, original spellings and byte positions
are compared; equal node counts alone do not pass. The ignored Rust test also
requires the measured candidate to match the checked-in build inventory and
passes the Java-produced failures through the independent Rust validator.

The private `CompilerSession` always creates a fresh pinned compiler task and
file manager, fixes `Locale.US` and all options, and only calls `parse()` and
`analyze()`. Its pre-attribution inventory retains raw children, names,
modifiers and literals, including dead branches. T05 rejects excluded raw
parents before calling `requireUnchanged` on admitted subtrees. In particular,
an implicit class constructor or an inferred `var` type is inventoried without
prematurely converting a subset rejection into an adapter failure. javac's
shared modifier/type objects in multi-declarators are counted once and marked;
sharing still fails accepted-tree comparison, after the raw statement gate.

The unit source object can be a javac wrapper. At parse completion, the adapter
binds it once to the supplied source using the immutable `CharSequence`
identity, source order and URI, then checks unit/file identity afterward.
Diagnostic sources must be the original captured `JavaFileObject`; URI or
text equality cannot admit a substitute. The listener never asks for compiler
messages or line/column positions. It maps checked UTF-16 boundaries into the
original UTF-8 bytes and retains at most 1,024 callbacks before fixed-message
normalization and deterministic sorting.

`CapturedSnapshot` is the second capture boundary over a native parent's
private, read-only snapshot. It uses no-follow secure directory streams,
regular-file/link checks, exact selection inventory, bounded byte reads and
two metadata/content passes. Its POSIX path metadata checks do not replace
the parent's descriptor-based host capture and read-only seal. The shared
native capture now recognizes Java source/selected-contract inputs and retains
unlisted inventory for the child's rejection. T07's private runner materializes
the exact selected captured inputs under the registered read-only mount;
public capture/API integration remains T08.

Application class/source/module/processor paths are closed; all four output
methods and service loading refuse access. Planted source/class positive
controls and module/service fixtures prove that the test dependencies exist
but the adapter does not consume them. `--release 25` still uses javac's
separate platform manager: zero wrapper system-file returns are expected.
The pinned JDK and T07 filesystem boundary close that view; this wrapper does
not claim to intercept it.

T04 executes the capture/encoding/parse/attribution precedence and operational
failure cases. `admission_precedence` names T05's three executable contributions:
subset before missing sidecars, and excluded class/var parents before accepted-
tree comparison. `lowering_precedence` names T06's executed contracts-before-
lowering and map-failure-before-publication contributions. `runner_precedence`
names T07's executed release-before-source JVM test; `follow_on_precedence`
is empty. Native installed precedence is also required by the T07 gate. All stages must execute
before the T09 gate.
No successful source artifact or installed Java release is claimed by this test.

## Source admission and sidecar verification

```sh
./scripts/check-java-frontend.sh --check-admission-fixtures
cargo test -p mpk-cli --test java_subset --test java_contracts --offline
./scripts/check-java-frontend.sh --run-admission
cargo test -p mpk-cli --test java_subset --test java_contracts --offline -- --ignored --test-threads=1
```

The fixture check needs no JDK or Docker. The executable uses the same pinned,
offline build and isolation setup as T04, and compiles `AdmissionTests.java`
outside the candidate JAR. It executes all 61 T05 source refusals, 14 contract
refusals, source admission for the 49 accepted vectors, all 35 conversion rules,
and the six subset/contract counter boundaries and their plus-one failures.
Extra cases cover raw identifiers and symbols, inert declaration forms, dead
branches, JSON syntax/Unicode/numbers, complete attachment and failure ordering.
Python independently checks declaration/callee order, original variable spans,
sidecar/raw-byte hashes and order-preserving normalization. Rust independently
checks the actual failure envelopes, complete Java context and normalized hashes.

`JavaSubset` validates raw spellings and all syntactic paths, closes calls over
the exact captured method symbols and exposes immutable typed bindings for T06.
`JavaContracts` validates every selected file as strict JSON before parsing the
bounded expression model, then checks attachment before type interpretation.
It never infers a contract, folds an expression or drops a duplicate clause.
The canonical sidecar hash, raw input hash and common normalized contract hash
remain distinct. Source bytes and the selection must match the admitted closure.

`JavaAdmission` sequences these internal stages and closes each compiler session.
Its successful result is an internal model, not an `ir-lowered` response or proof
verdict. The packaged `Main` is unchanged and still refuses source invocations.

## Lowering, source maps and manifest verification

```sh
./scripts/check-java-frontend.sh --check-lowering-fixtures
cargo test -p mpk-cli --test java_lowering --test java_source_maps --offline
./scripts/check-java-frontend.sh --run-lowering
cargo test -p mpk-cli --test java_lowering --test java_source_maps --offline -- --ignored --test-threads=1
```

The fixture check needs no JDK or Docker. `--run-lowering` compiles
`LoweringTests.java` separately from the candidate and uses the same pinned,
offline build boundary. It executes all 49 accepted cases, 27 operation
mappings, six symbolic CFG goldens and seven original-source map vectors.
Each successful source is analyzed twice with fresh compiler sessions and
must produce identical complete response bytes. Python independently checks
canonical JSON, all artifact/input hashes, complete origins and mathematical
Bool/BV evaluations. Rust imports the real responses and exact captured bytes
through the existing revision-3 validators, then rejects rehashed operation,
check, source-map and manifest mutations. This is not the separately compiled
JDK differential/fuzz corpus or native release rehearsal owned by T09.

`JavaLowering` constructs the graph before canonical numbering. False edges
precede true edges in BFS order. Nested eager operands and call arguments use
block parameters to carry live values across short-circuit/ternary graphs;
there are no generated source locals or cross-block temporary references.
Java shifts use the exact adjacent mask and signed/unsigned carrier pattern.
Division and remainder carry only `divisor_nonzero`; wrapping operations have
no overflow checks. The finished graph is independently checked for shape,
scope, definite assignment, acyclicity, calls, checks and carrier uses.

`JavaSourceMaps` requires original captured source/tree identity, exact valid
UTF-16 boundaries, method containment and the correct source-node role. Every
function, instruction and terminator has a UTF-8 byte origin; block parameters
have none. `JavaEmission` rechecks selection and raw source/sidecar bindings,
hashes the complete VIR/map/manifest with the successor domains, and returns
bytes only after all canonical budgets, including the final stdout LF, pass.
All nine lowering/emission counters have inclusive/plus-one tests through the
production counter consumers. These counter tests do not allocate maximal
artifacts or claim such inputs can bypass earlier syntax or native limits.

The private `JavaFrontend` sequences admission, lowering and emission and
returns a complete success or artifact-free failure. The fixed T07 `Main`
invokes it only after runtime preflight. The lowering harness explicitly supplies `test.java.frontend`,
`test.java.toolchain` and a zero release-registry digest, together with the
actual built JAR hash and the frozen JDK archive hash as the test distribution
identity. These are test identities, not registered candidate bundles. The
production identity input does not equate the complete candidate distribution
digest with its JDK archive digest. Manifest components now bind the actual
`java` executable, the complete JDK content projection and the native-runtime
inventory. T07 supplies validated candidate identities; its native enforcement
gate remains unrun. No environment toggle or alternate public MPK route
discovers this private pipeline.

## T07 private candidate and runner

The candidate descriptors live under `release/build-inputs/java/`, outside
the active `release/bundles/` registry. They contain one Java tuple, the exact
revision-3 semantic context, and Java-specific host/layout IDs. The shared
descriptor validator requires these exact candidate bytes even if an attacker
repairs outer hashes. No existing host ID or Go/Rust/C# budget changes.

The frontend bundle contains `java2vir.jar` and its notice. The toolchain
contains 399 JDK files and six frozen native files. The 205 inventoried JDK
symlinks are resolved only through frozen archive records and materialized as
separate regular files; all JDK legal-notice bytes are retained. Inventory hashes
cover these paths, executable bits, sizes and bytes. Files are sealed 0444/0555
and directories 0555; links, extra files and changed permissions reject.

The private runner is compiled only into `java_frontend_runner.rs`'s test
executable. Its installed root is discovered beside its actual `bin/mpk`
inode, through the existing no-follow descriptor loader. It checks both
registries, captures both bundles and prepares the host before accessing source
inputs. There is no caller-selected Java executable, JDK, classpath or registry.
Its fixed fixtures do not constitute a new user-source CLI.

The Java sandbox uses 1 GiB memory, zero swap, 128 PIDs, 16 GiB address space,
1,024 descriptors, zero core bytes, a 64 MiB `noswap`/noexec tmpfs and 120 seconds.
Atomic cgroup placement, pidfd supervision, bounded pipes and descendant/backing
cleanup use the existing implementation. Java adds nonroot UID/GID 65534 with
no supplementary groups or capabilities, a private read-only PID proc view,
and a finite x86-64 seccomp filter. `clone3` returns `ENOSYS` for the glibc
thread fallback; only the exact pthread `clone` flags pass. Socket/process/
namespace escapes reject. The capability probe installs and challenges this
policy before source materialization. The packaged Main checks fixed argv and
environment, JDK/JAR bytes, PID/capability state and read-only input mounts
before capturing or analyzing source. This is candidate code, not evidence of
native compatibility until the native gate below passes.

Local checks, including a provisioned fixed-JDK test:

```sh
./scripts/build-java-candidate.sh --check
cargo test --offline -p mpk-cli --test java_frontend_runner
./scripts/check-java-frontend.sh --run-runner
```

After an intentional Java source/notice change, run the existing
`--update-inventory` build first, then
`./scripts/build-java-candidate.sh --update-descriptors`; review all related
hashes together. Neither command installs or activates a release.

Build the owning test executable on Linux x86-64 with
`cargo test --offline --locked -p mpk-cli --test java_frontend_runner --no-run`.
Use the executable path Cargo prints (not the public `mpk` binary) below.
The output directory must not exist; the assembler uses two offline JDK builds,
the frozen archive and the already provisioned native image. It copies no host
JDK or library. A failed assembly is not a publishable image.

```sh
./scripts/build-java-candidate.sh --assemble /absolute/new/java-image /absolute/path/to/java_frontend_runner-HASH
./scripts/build-java-candidate.sh --check-image /absolute/new/java-image /absolute/path/to/java_frontend_runner-HASH
./scripts/check-java-runner.sh --image /absolute/new/java-image /absolute/path/to/java_frontend_runner-HASH
./scripts/check-java-runner.sh --native /absolute/new/java-image /absolute/path/to/java_frontend_runner-HASH
```

`--image` executes ten real file/registry mutations without claiming kernel
enforcement. `--native` additionally requires root on native x86-64 Linux,
kernel 6.4+, the writable initial cgroup-v2 hierarchy and `/usr/bin/strace`.
It runs the installed JVM, checks hostile environment equivalence, captures
thread/syscall evidence attributed to the JVM and its threads, rejects unbounded launch, tests real OOM/PID/
120-second timeout/pipe/tmpfs failures and cleanup with source-free test payloads,
and executes the same mutations through installed release preflight before source access. ARM
emulation or missing primitives cause failure, never a skipped passing gate.
The report binds the runner and registry digests and records the observed JVM
thread flags, clone3 fallback and pre-exec socket denials. Ordinary tests also
check that parent-only or incomplete trace records cannot pass as JVM evidence.
T07's native compatibility, resource-fault and acceptance evidence is still
outstanding. Complete release rehearsal and activation remain T09/T10.

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
JAR must pass the version/identity check and reject malformed frontend invocations.

On ARM development hosts Docker may emulate Linux x86-64. This verifies the
pinned build and artifact determinism; it does not establish the complete
native Linux installed sandbox or release gate owned by T07/T09/T10.
