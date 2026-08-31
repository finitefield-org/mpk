# Java offline frontend candidate

`JAVA-03-T02` provides the unregistered `java2vir` build project. Its only
successful invocation is `--version`; every source/frontend invocation exits
2 with `JAVA_FRONTEND_UNAVAILABLE` and no stdout. T03 adds independent Rust
contract/artifact validators. T04 adds internal immutable capture, strict
UTF-8 transport, public javac parse/analyze sessions and bounded failure
diagnostics. T05 adds source subset admission, inert initialization, conservative
acyclic call closure and typed contract sidecars, exercised through separately
compiled private test harnesses. T06 adds private CFG/lowering, original-byte
source maps and deterministic complete artifacts. Installed execution remains T07.
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
./scripts/build-java-frontend.sh --build "$(pwd -P)/target/java-candidates/t06"
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
unlisted inventory for the child's rejection. T07 owns connecting that capture
to the registered read-only mount and complete native process sandbox.

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
lowering and map-failure-before-publication contributions. `follow_on_precedence`
preserves T07's pending release-before-source outcome. All stages must execute
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
returns a complete success or artifact-free failure. It is not wired into
`Main`. The harness explicitly supplies `test.java.frontend`,
`test.java.toolchain` and a zero release-registry digest, together with the
actual built JAR hash and the frozen JDK archive hash as the test distribution
identity. These are test identities, not registered candidate bundles. The
production identity input does not equate the complete candidate distribution
digest with its JDK archive digest. Manifest components bind the frozen
`lib/server/libjvm.so`, `lib/modules` (including javac/system modules) and
`release` bytes. T07 must supply validated registered release identities and
the measured native runner; no environment toggle or alternate public route
discovers this private pipeline.

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
