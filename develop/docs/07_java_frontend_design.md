# Java Scalar Frontend Design

Status: Java-specific implementation design, subordinate to the normative
`JAVA_PROFILE_V0.md` frozen by completed `JAVA-03-T01` (2026-08-31). T02's
offline build candidate, T03's inactive validators and T04's internal
capture/compiler adapter and T05's source admission/typed sidecars are
complete. T06's private CFG/lowering and artifact emission are complete;
T07 is next. Installed execution and activation remain pending. The active release
remains Go/Rust/C# with semantic registry revision 2. This document includes
the corrections established by T01's disposable compiler/JVM probes.

Prepared: 2026-08-31.

## 1. Decision and authority

Implement `java2vir` as an untrusted, separately registered Java frontend using
the public JDK compiler APIs. The first profile accepts pure, non-generic
static methods declared in field-free source interfaces, with `boolean`,
`int`, and `long` values, local assignments, branches, returns, and acyclic
direct calls. It targets Java SE 25 without preview features on Linux x86-64.

This document refines the Java phase of
`06_multilanguage_frontend_design.md` and its todo document. It resolves the
design choices behind `Q-JAVA-01` and `Q-JAVA-02` in the historical semantic
matrix; that matrix and the compiler-feasibility record retain their original
research status. The frozen specifications and registry admission rules win
over this design if a conflict is discovered.

`JAVA-03-T01` completed the normative Java profile, conformance vectors,
pinned build-input inventory, revision-3 registry vectors, and
`java-03-implementation-traceability-ledger.md`. Exact hashes, inventories,
API observations and JVM options live in `../specs/vectors/java-profile-v0.json`;
this design does not duplicate mutable descriptor digests. T01's Linux amd64
measurements used CPU emulation on an ARM host. They establish the recorded
compiler/JVM compatibility, not the complete native Linux production
isolation gate owned by T07/T09/T10. T02 added the unregistered project and
offline build owner, with exact source/class/JAR inventories and two matching
isolated builds. See `java-tools/README.md` and the implementation ledger.
No public Java route is authorized before T10.

Certificate v0, checker inputs, the two source-free checking implementations,
and the four axiom categories remain unchanged. Java source, contracts, JDK,
frontend, differential results, VIR, VCs, and reports remain untrusted helper
data. No Java semantics axiom, compiler plugin, mixed-language module,
user-selected executable, or new proof-acceptance route is introduced.

### 1.1 Scope and exclusions

| In the first Java profile | Excluded from the first Java profile |
| --- | --- |
| Named packages and source interfaces with static method bodies | Classes, records, enums, annotation declarations, nested types, inheritance |
| `boolean`, signed 32-bit `int`, signed 64-bit `long` | `byte`, `short`, `char`, floating point, references, arrays, boxed values, `void` |
| Pure expressions, explicit local types, initialized locals, `if`/`else`, return | Loops, switch, pattern matching, assignments inside expressions, increments, compound assignments |
| Exact-signature static calls within the selected source closure | Instance/virtual calls, recursion, constructors, library calls, method references, lambdas |
| Strict JSON contract sidecars | Annotations, comments, Javadoc, processor output as contracts |
| Pinned JDK analysis without executing the selected program | Maven/Gradle/project discovery, source generation, bytecode input, runtime reflection |

Ordinary class-based Java applications require a later reviewed profile. The
interface restriction is intentional: it avoids treating `java.lang.Object`
or a metadata-only superclass initialization chain as proven inert.

## 2. Existing implementation and change boundary

The current code already implements the representation and certificate
foundations needed for this subset. Java needs its own finite validation and
dispatch branches, not a new numeric carrier or a common operation meaning.

| Current owner | Required Java work |
| --- | --- |
| `crates/mpk-vc/src/semantic_profile_registry.rs` | Compile the Java entry, parameters, selection, and all nine contract validators; activate only the frozen revision-3 root at release |
| `crates/mpk-vc/src/successor_source_artifacts.rs` | Java operation/type restrictions, context/selection linkage, source-map and manifest validation |
| `crates/mpk-vc/src/safety_check.rs` | A distinct Java required-check profile, including division semantics in section 5 |
| `crates/mpk-vc/src/successor_vc.rs` | Validated Java projection and Java-owned check dispatch; preserve Java context in every public artifact |
| `crates/mpk-vc/src/release_bundle_v1.rs` | Closed JVM launcher, toolchain, native inventory, host/layout and bundle validation |
| `crates/mpk-cli/src/frontend_sandbox.rs` | Registered JVM execution under measured isolation requirements; no ambient `java` lookup |
| `crates/mpk-cli/src/` policy, frontend and release routes | Java selection translation, runner, policy/evidence reproduction, installation and release cutover |
| `crates/mpk-cli/src/successor_ai_explain.rs`, `crates/mpk-api/src/successor_api.rs` | Java display/redaction and exact context-bound API routing |
| Planned `java-tools/java2vir/` | Public compiler adapter, capture, subset/closure checks, contracts, CFG, lowering, emission |

Do not add Java to the legacy Go/Rust `SemanticProfile` API to bypass the
successor boundary. Any reuse of the existing private structural projection
is allowed only after Java validation and must never relabel public Java
artifacts as Go or C#. Do not inherit C# check requirements by analogy.

## 3. Profile, selection, and source closure

### 3.1 Frozen, inactive identities

T01 froze the following names in the normative specification. They remain
inactive production values until the later implementation and release gates
pass; the current registry does not accept Java.

| Role | Frozen identity |
| --- | --- |
| Language / semantic profile | `java` / `mpk.java.scalar.v0` |
| Parameter / selection schema | `mpk.semantic_parameters.java_scalar.v0` / `mpk.selection.java_methods.v0` |
| Sidecar / build-input schema | `mpk.java.contract.v0` / `mpk.java.toolchain_inputs.v0` |
| Limits / operation / required-check profile | `mpk.java.limits.v0` / `mpk.java.operations.v0` / `mpk.java.required_checks.v0` |
| Launcher / environment | `mpk.java.jvm_launcher.v0` / `mpk.java.frontend_environment.v0` |
| Host / runtime layout | `mpk.host.linux-x86_64-gnu.java25.v0` / `mpk.runtime.linux-x86_64-gnu.java25.v0` |

The parameter value is a closed object with exactly `language_version: "25"`,
`release: "25"`, `preview: false`, `encoding: "UTF-8"`,
`annotation_processing: "none"`, and `target_id: "linux-x64"`. JDK version and
native-host byte inventories belong to the toolchain/release contract, not a
caller override. Java integer widths never derive from host pointer width.

### 3.2 Selection envelope

Use the existing `SelectionEnvelope` shape with the Java schema. Its closed
`value` contains:

| Field | Validation |
| --- | --- |
| `compilation` | 1..64 ASCII bytes, `[a-z][a-z0-9]*([._-][a-z0-9]+)*` |
| `sources` | 1..256 strictly increasing unique portable relative paths under `src/`, ending `.java` |
| `contracts` | 1..128 strictly increasing unique portable relative paths under `contracts/`, ending `.json` |
| `methods` | 1..32 strictly increasing unique canonical method IDs, each at most 1,024 ASCII bytes |

A method ID is `package.Interface::method(T1,T2)->R`, with no whitespace;
`T1`, `T2`, and `R` are exactly `boolean`, `int`, or `long`. Zero parameters
use `()`. Every package/type/method/parameter/local identifier uses
`[A-Za-z_][A-Za-z0-9_]*`, excluding `_` and the normative profile's complete
reserved/contextual-word list. T01 deliberately rejects contextual words in
every identifier position, including `record`, `var`, `when`, and `yield`.
`$`, non-ASCII/identifier-ignorable characters, escaped spelling and alternate
separators reject. The package has at least
one segment and cannot occupy `java`, `javax`, `jdk`, `sun`, or `com.sun`
namespaces. Each file declares exactly one public top-level interface, with
path `src/<package path>/<Interface>.java`; package and file case must match.

Example selection value:

```json
{"compilation":"payment-policy","contracts":["contracts/approved.json"],"methods":["demo.Policy::approved(long,long)->boolean"],"sources":["src/demo/Policy.java"]}
```

Capture exactly the listed regular files and implied directories using the
shared no-follow immutable snapshot boundary. Reject unlisted entries,
missing files, links, hard-link aliases, special files, traversal, duplicate
or case-fold-colliding paths. Source discovery and globs are absent. Input
sources and sidecars are untrusted even after their byte hashes validate.

### 3.3 Compilation and call closure

All selected source files are parsed and analyzed in one compilation. Resolve
each selected method by declaration symbol and exact signature; string
matching alone is insufficient. No overload declarations are admitted in v0:
a method name occurs once per interface, even if signatures would differ.

Start the conservative closure at `methods`, follow every source call in
every syntactic branch, and reject cycles. Every declared method must belong
to that closure, and every closure method must have exactly one selected
sidecar. Reject unrelated source members and missing, duplicate or unused
sidecars. This applies even to statically dead branches. No selected source
is silently ignored because the compiler folded a condition.

Admitted calls are either an unqualified same-interface method name or a
fully package-qualified interface method name. `Trees.getElement` must return
the exact source `ExecutableElement`, with accepted static modifiers and exact
argument/return types. Reject expression receivers, imported names, inherited
methods, varargs, generic instantiation and all library calls. Cross-interface
calls obey the same inert-initialization rule and same-language closure.

## 4. Source subset and initialization

Each interface has no fields, superinterfaces, type parameters, annotations,
nested declarations, or initialization blocks. Its only members are methods
with explicitly written `public static`, no other modifiers, an explicit
scalar return type, scalar parameters, and a body. No `throws` clause is
accepted. Count descriptor parameter units explicitly: `boolean`/`int` use
one and `long` uses two, with maximum 255 for a static method. The T01 probe
found that `analyze()` accepts 256 units without diagnostics, so the subset
validator must enforce this bound itself. Imports, module declarations and
`package-info.java` reject.

This shape closes initialization structurally: invoking a static interface
method initializes that interface, which has no state or initialization
actions in the accepted subset. It does not depend on interpreting a compiled
class's initializer. The frontend itself only analyzes source; it does not
load selected classes. Any later relaxation must revisit JLS initialization
and the `M27`/`M30` feasibility boundary. See
[JLS 12.4](https://docs.oracle.com/javase/specs/jls/se25/html/jls-12.html#jls-12.4)
and [JLS interface methods](https://docs.oracle.com/javase/specs/jls/se25/html/jls-9.html#jls-9.4).

Normal-completion claims concern this scalar program model. They do not
promise that an actual JVM cannot suffer resource exhaustion, linkage failure
or another environmental failure; those are not new logical assumptions.

Statements are blocks, one explicitly typed local with one initializer,
simple assignment statements to an existing local, `if` with optional `else`,
and value-return statements. Parameter assignment, shadowing, repeated local
names, `final`/`var`, empty statements, labeled statements, and uninitialized
locals reject. Every path returns. Unreachable statement sequences following
a terminating statement reject; both arms of a constant conditional are
nevertheless subset-checked and included in conservative call closure.

Expressions are accepted literals, parameter/local reads, parentheses, the
operators in section 5, accepted casts, same-type conditional `?:`, and direct
static calls. An assignment cannot occur inside an expression. Expression
statements other than the local assignment form reject. No `assert`, `throw`,
`try`, synchronization, allocation, I/O, time, randomness, reflection, native
method, or host-environment access is admitted.

Illustrative accepted source, with a separate sidecar:

```java
package demo;
public interface Policy {
    public static boolean approved(long amount, long limit) {
        return amount >= 0L && amount <= limit;
    }
}
```

## 5. Exact scalar semantics and VIR mapping

### 5.1 Types, literals, and conversions

Map `boolean` to VIR `bool`, `int` to signed BV32 and `long` to signed BV64.
Source type spelling is the primitive keyword only. The source value domain
contains no unsigned type. Internal unsigned BV32/BV64 temporaries are allowed
only in the canonical logical-right-shift pattern below; they cannot occur
in function signatures, source locals, contracts, calls, or return values.

Allow `true`, `false`, canonical decimal `int` literals, and decimal `long`
literals ending uppercase `L`. Decimal digits are `0` or a nonzero digit
followed by digits. Reject other radices, separators, leading zeroes, lowercase
suffixes, character/string/text-block literals, `null`, and unary plus.
Minus followed by an integer literal is accepted when the pinned compiler
confirms an in-range negative value, including the two minimum values; emit
one signed `Const`. Parenthesized forms do not create an alternate
out-of-range positive-literal acceptance path. Other unary minus emits
`bv_neg`. Do not replace source arithmetic with host arithmetic or compiler
constant values; lower the original accepted tree. A missing or rewritten
tree outside the frozen adapter patterns is an adapter error.

Identity conversions need no instruction. Implicit `int -> long` is allowed
only for a local initializer, local assignment, or return; emit sign-extending
`Convert`. Explicit `(long)` on an `int` sign-extends; explicit `(int)` on a
`long` retains the low 32 bits and interprets them as signed. Identity casts
are allowed. No overflow check is added. Calls, binary expressions and `?:`
require already identical source operand types, except the shift rule.
Mixed `int`/`long` arithmetic and implicit call widening reject; a source cast
can make the types match. Boxing, unboxing and all other conversions reject.
These restrictions intentionally accept less than Java's conversion rules;
see [JLS conversions](https://docs.oracle.com/javase/specs/jls/se25/html/jls-5.html).

### 5.2 Operations and required checks

| Source operation | Exact lowering | Required safety checks |
| --- | --- | --- |
| `!b` | `not` | none |
| `==`, `!=` on identical accepted types | `eq`, `not_eq` | none |
| `<`, `<=`, `>`, `>=` on identical integers | signed comparisons | none |
| Integer `+`, `-`, `*`, unary `-` | wrapping `bv_add/sub/mul/neg` at 32 or 64 bits | none |
| Integer `~`, `&`, `\|`, `^` | `bv_not/and/or/xor` | none |
| Integer `/`, `%` | `bv_sdiv`, `bv_srem` | exactly `[divisor_nonzero]` |
| `<<`, `>>` with integer LHS and exact `int` RHS | mask count, then `bv_shl` / `bv_ashr` | none |
| `>>>` with integer LHS and exact `int` RHS | mask count and unsigned-temporary pattern | none |
| Boolean `&&`, `\|\|`, same-type `?:` | explicit conditional control flow | checks only on executed branches |

The Java required-check validator rejects missing, extra, duplicate or
reordered checks. In particular, `integer_no_overflow` and
`signed_divrem_representable` are invalid on Java source arithmetic. Boolean
`&`, `|`, `^`, unsigned comparisons and unsigned division/remainder reject.

Java `MIN / -1` yields `MIN`, and `MIN % -1` yields zero. Existing total VIR
bitvector equations already encode both results; no new operation, branch
repair or excluded-overflow precondition is needed. Division by zero remains
an operation-safety VC, because Java can throw whereas the profile requires
normal completion. A lowered program is not verified until its path-specific
divisor checks and contracts are proved. See
[JLS integer division/remainder](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html#jls-15.17.2)
and `../specs/VIR_V0.md`, section 8.1.

Evaluate operands left-to-right exactly once. For a shift of an `int`, use
`n & 31`; for a shift of a `long`, use `n & 63`. Both mask operands are signed
BV32. A `long` shift count is rejected, even though Java permits it. For
`x >>> n`, convert `x` to an unsigned BV of the same width, perform `bv_lshr`
with the masked BV32 count, and convert the result back to signed. The two
conversions preserve all bits. The Java validator recognizes this exact
internal pattern and refuses arbitrary unsigned intermediates. Negative and
oversized counts are valid after masking. See
[JLS shifts](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html#jls-15.19).

For all three shift operators, validate the exact mask width/value and its
data-flow linkage to the count operand. An unmasked count or unrelated mask
instruction does not satisfy the Java operation profile. Generated mask and
conversion operations have no additional safety checks.

### 5.3 Public-tree CFG and stable IDs

Use only `JavaCompiler.getTask`, `JavacTask.parse/analyze`, `Trees`,
`SourcePositions`, `Elements`, `Types`, and the exported tree/language-model
interfaces. There is no supported public javac CFG API. Construct an MPK-owned
CFG by exhaustive dispatch over the accepted tree kinds; private
`com.sun.tools.javac.*`, reflection, `--add-exports`, and `--add-opens` are
forbidden. The public API boundaries are documented in
[`JavacTask`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/JavacTask.html)
and [`jdk.compiler`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/module-summary.html).

Snapshot source trees before attribution; finish attribution diagnostics
before subset classification. After diagnostic-free attribution, reject known
excluded raw source parents/forms before checking compiler transformations
inside candidate accepted subtrees. The measured valid-class case gains a
synthetic constructor with missing end positions, but still rejects as
`JAVA_SUBSET_DECLARATION`; its generated descendants are neither admitted AST
nor origin-mapped source. The measured `var` declaration has no type child
before analysis and gains a `PRIMITIVE_TYPE` with missing end afterward;
reject its raw source shape as `JAVA_SUBSET_TYPE` before those accepted-tree
checks. Apply the same ordering to records/enums, disallowed raw identifiers
and other ordinary excluded forms.

Only source subtrees that survive those gates are compared with the frozen
pre/post-attribution inventory. An erroneous tree, missing/error type,
unresolved element, unexpected synthesized member, or unknown tree kind in
that accepted candidate fails closed as an adapter error. Compiler/resource
failure and attribution diagnostics keep their earlier precedence. Derive
promotions only from the closed source rules and resolved types, never javac
internal operator symbols. Count all public nodes under the bounded inventory,
but do not treat counting an excluded subtree's synthetic nodes as acceptance
or as an accepted-source transformation check. Every syntactic branch under
an admitted parent, including a constant-dead arm, is checked before lowering.

The CFG has one entry, no loop/back-edge or exceptional region, explicit
conditional terminators, and return terminators on all complete paths.
Short-circuit RHS expressions and conditional arms occupy separate blocks;
their calls and safety checks cannot be hoisted. Merge local values through
the existing VIR local/copy rules, requiring assignment on every incoming
path. Lower callee-before-caller with method-ID lexical tie breaking. Emit
all conservative closure methods, even if a call occurs in a source-dead arm.

Use `arg0...` by parameter order, `local0...` only for source locals by
declaration byte position, `t0...` by canonical emitted instruction order,
`result0`, and `bb0...` by breadth-first traversal with false before true.
Short-circuit/conditional expression results join through existing block
parameters `p0...` in canonical block/parameter order; do not invent synthetic
source locals. The normative profile and `cfg_patterns` freeze these shapes.
Compiler object identities, hash-map order and compiler tree numbering never
enter public IDs. T01 froze representative branch/join/nested-call/conversion
golden patterns; constant conditions preserve the same branch shape.

## 6. Contract sidecars and verification

Use a closed root with exactly `schema`, `semantic_profile`, `method`,
`requires`, `ensures`, `modifies`, `abrupt_completion`, and `termination`.
Fix the first two to the Java identities, bind `method` to the exact method
ID, require `modifies=[]`, `abrupt_completion="forbidden"`, and
`termination="total"`. `requires` may be empty; `ensures` is nonempty; their
combined clause count is at most 64. Every clause has Boolean type. Parameter
references resolve only to that method's parameters; locals never appear. Result references are permitted
only in `ensures`, never anywhere inside `requires`.

The expression encoding follows the existing sidecar approach, with this
smaller closed Java vocabulary:

| Expression | Exact members / accepted values |
| --- | --- |
| Parameter / result / Boolean | `{"parameter":Name}`, `{"result":0}`, `{"bool":Boolean}` |
| Integer | `{"int":{"decimal":CanonicalDecimal,"type":"i32" or "i64"}}`, in range |
| Unary | `{"op":Op,"args":[Expr]}`, `Op` is `not`, `bv_neg`, or `bv_not` |
| Boolean n-ary | `{"op":"and" or "or","args":[Expr,...]}`, 2..64 Boolean operands |
| Binary | `{"op":Op,"args":[Expr,Expr]}`; exact same operand type |

Binary operators are `eq`, `not_eq`, `signed_lt/le/gt/ge`, `bv_add/sub/mul`,
and `bv_and/or/xor`, with each slash-separated family denoting the individual
VIR names. Signed comparisons and bitvector operators require integers.
Contract division, remainder, shifts, conversions, fields, calls, source
operator spellings, arbitrary JSON and unsigned types reject. Canonical
decimal uses `0` or optional minus plus a nonzero digit and remaining digits;
no leading plus/zero, negative zero, suffix or separators.

Reject duplicate JSON keys, unknown members, unresolved names, wrong types,
extra/missing sidecars and profile mismatches. Normalize parameter names to
`argN`; do not fold, reorder, deduplicate or convert contract expressions.
Use the successor contract and VC encodings, `MPK-CONTRACT-1.0`, empty loop
contracts, and the complete Java `SemanticContext`. Preserve raw sidecar
bytes separately in the source manifest.

For calls, generate the callee-precondition obligation at the call site and
use the validated callee postcondition through the existing call WP path.
Each closure method's body must satisfy its own contract. Emit a
self-contained canonical certificate through the existing program-certificate
assembler; both source-free checkers must accept the identical certificate
bytes. A compiler-success result, empty diagnostic list, or matching
differential run is never a verification verdict.

## 7. JDK, build, and execution closure

### 7.1 Selected toolchain and freeze artifacts

Use Eclipse Temurin **25.0.4.1+1**, Linux x64, HotSpot JDK, as the design's
compiler/runtime build, with `--release 25`. Its
[official release](https://github.com/adoptium/temurin25-binaries/releases/tag/jdk-25.0.4.1%2B1)
is the discovery source, not a runtime download route. No `latest` tag,
automatic update, host JDK, SDK manager, or version-range match is permitted.

T01 recorded the exact archive URL, byte length, SHA-256, safe extraction
rules, `release` metadata, executable/library modes, complete JDK/reference/
native inventory, dependency linkage, redistribution notices, and canonical
descriptor self-hash in the Java vector. The native dependency closure adds
six exact host files: the ELF loader and libc, libdl, libm, libpthread, and
librt; JDK-owned native libraries remain in the complete JDK inventory. Verify
the archive against the publisher checksum and the independently recorded digest. JDK 25 Temurin
archives need not contain `jmods`; inventory the actual runtime modules and
`ct.sym` rather than assume that directory exists. See
[Temurin 25 packaging](https://adoptium.net/news/2025/09/eclipse-temurin-25-available).

Build the frontend with that same pinned JDK from checked-in Java sources,
without external dependencies or Maven/Gradle. Freeze the source inventory,
compiler arguments, class-file inventory and deterministic JAR entry order,
timestamps and manifest bytes. No `Class-Path` or service-provider entry is
allowed in the frontend JAR. Two isolated offline builds must produce
identical JAR and descriptor bytes. Provisioning downloads are a separate
explicit setup step; validation/build/release gates never download.

The runtime image and its public compiler classes are toolchain inputs. The
analyzed program does not obtain permission to call a JDK API merely because
that API's class file is visible during attribution.

### 7.2 Compiler task and file manager

Create a fresh compiler task per request, with an explicit US/English locale,
UTF-8 decoder, bounded diagnostic listener and source objects ordered by the
selection array. The fixed compiler option set is `--release 25`,
`-encoding UTF-8`, `-proc:none`, `-implicit:none`, `-Xlint:none`, plus pinned
error/warning count limits from section 9. Do not call `generate()` or
`CompilationTask.call()`; stop after `analyze()`. No preview, plugin, processor,
user option, response file, `--source`, `--target`, or `--system` override is
accepted. See [javac options](https://docs.oracle.com/en/java/javase/25/docs/specs/man/javac.html).

Wrap the application standard file manager with a closed, audited lookup
boundary exposing only captured source objects. T01 observed that
`--release 25` obtains platform references through a separate internal javac
file manager outside that wrapper; successful JDK type resolution produced
zero wrapper system-file returns. Close this separate platform view through
the exact pinned JDK inventory, fixed options/runtime image and OS filesystem
boundary. Do not claim the wrapper intercepts all platform reads, and do not
call internal compiler APIs to replace it. Set application class, source,
module, upgrade, patch and processor locations to empty where the public API permits; refuse unsupported location queries
according to the API contract. Never pass an empty string as a classpath
element, since that can denote the working directory. A controlled empty
directory or an empty location collection supplies an explicit empty path.

Cover `list`, input lookup, `contains`, binary-name/module-name inference,
module-location enumeration, and any classloader/service-loader access.
Unknown locations return the API-specified empty/absent result or a bounded
adapter error; they never delegate to a host default. Output methods always
refuse writes. System-file lookup is permitted only within the inventoried
pinned JDK, without exposing the frontend JAR as an analyzed dependency. Keep
the compiler-host module graph separate from the analyzed program's empty
application module path. See
[`JavaFileManager`](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/JavaFileManager.html).

`-implicit:none` prevents implicit class-file generation; it does not close
source discovery. The file manager must independently reject every unselected
source and external class. Tests must plant resolvable sources, processor
services and JARs outside the selection and prove they are never consumed.

### 7.3 Registered JVM launcher and isolation

The planned launcher directly executes the registered
`/mpk/toolchain/jdk/bin/java`, with frontend classpath exactly
`/mpk/frontend/java2vir.jar` and main class `mpk.java2vir.Main`. The main class
loads only the frontend and frozen JDK. `ToolProvider.getSystemJavaCompiler()`
must resolve the expected compiler in that JDK; absence or a different
provider is a frontend error. No subordinate `javac` process is launched.

Freeze an interpreter-only baseline (`-Xint`), shared-archive loading off
(`-Xshare:off`), Serial GC, one reported processor, attach and performance-data
facilities disabled, explicit heap/stack ceilings, fixed UTF-8/English/UTC
properties, and bounded private temporary/error paths. T01 froze the exact
JVM option array, supported-option probe, root module graph and memory/process
budgets in `launcher_contract` and `toolchain_inputs.host`. The heap is
32 MiB initial/512 MiB maximum, each stack 1 MiB; JVM root modules are
`java.base,java.compiler,jdk.compiler,jdk.zipfs` (the resolved graph also
includes `jdk.internal.opt`). Native requirements are glibc 2.36 and kernel
ABI 6.4.0 or later, 16 GiB address space, 1 GiB cgroup memory, zero swap,
128 PIDs, 1,024 open files, zero core bytes, 64 MiB private `noswap` tmpfs,
and 120 seconds per request. These flags and requirements do not by
themselves establish production sandbox enforcement. The
[Java launcher manual](https://docs.oracle.com/en/java/javase/25/docs/specs/man/java.html)
is the option reference. No fallback to JIT or a less restricted launch is
allowed if the baseline fails the local Linux probe.

Build an environment from an allowlist: fixed `HOME=/mpk/empty-home`,
`TMPDIR=/mpk/tmp`, `PATH=/nonexistent`, `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`,
and `TZ=UTC`. Do not inherit `JAVA_HOME`, `CLASSPATH`, `JAVA_TOOL_OPTIONS`,
`JDK_JAVA_OPTIONS`, `JDK_JAVAC_OPTIONS`, `_JAVA_OPTIONS`, `LD_PRELOAD`, or
`LD_LIBRARY_PATH`; the executable and native dependency closure come from
validated descriptors. There are no provider credentials or host home mounts.

Java uses its own host/layout profile; do not widen or silently reuse the
.NET-specific layout. T01 freezes declarative Linux/glibc/JDK/native-library,
ELF-interpreter, file/device/proc-layout, permission, temporary-storage and
resource-budget requirements. It records the disposable compiler/JVM
compatibility measurements and their stated limitations. It does not invent
unmeasured native syscall or clone restrictions.

T07 implements the registered native runner and verifies its syscall/clone
policy, cgroup/resource-failure enforcement, privilege drop and descendant
cleanup; T09/T10 run the complete native Linux release gates. Those tasks
must preserve the frozen read-only source/toolchain mounts, private bounded
`noswap` tmpfs, cgroup memory/PID limits, denied network and absence of
writable executable inputs. An emulated T01 success cannot discharge those
later enforcement obligations or authorize a weaker sandbox.

The generic runner may gain a finite JVM launcher branch. It may not expose
raw caller paths or relax Go/Rust/C# contracts. If the current descriptor
shape or isolation contract cannot express the measured closure, stop T01
and review the affected successor schema before implementing that branch;
do not amend an immutable existing ID's meaning. No Java production route
is authorized by an unproven host requirement.

## 8. Source maps, diagnostics, and failure behavior

Require nonempty strict UTF-8 source, no BOM, NUL, CR, surrogate or Unicode
noncharacter, LF line endings and a final LF. ASCII identifiers are enforced
on raw spelling; Unicode comments are allowed. Reject every raw ASCII
backslash immediately followed by `u`, even inside comments. This deliberately
conservative rule excludes all Java Unicode-escape translation and some
otherwise harmless comments; there is no preprocessing rewrite of source.
See [JLS lexical translation](https://docs.oracle.com/javase/specs/jls/se25/html/jls-3.html#jls-3.3).

Use public source start/end positions over the original `JavaFileObject`
contents and a checked UTF-16-boundary-to-UTF-8-byte table. An absent span,
out-of-range position or position inside a surrogate pair is an adapter/map
error for any emitted source operation. Do not calculate byte offsets from
`LineMap.getColumnNumber`, which expands tabs. Public origins contain logical
paths and UTF-8 byte ranges only; line/column values are not serialized or
accepted as source-map input. Compiler URI prefixes and absolute host paths
never enter artifacts. See
[`SourcePositions`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/SourcePositions.html)
and [`LineMap`](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/tree/LineMap.html).

Each lowering instruction, including masks and conversion helpers, maps to
the owning source expression span. Declare an empty synthetic-origin
allowlist. Sidecars and raw sources enter the manifest as distinct `contract`
and `source` inputs; frontend-stage manifests have no VC hash, and VC
finalization changes only the fields permitted by the successor manifest
contract. Bind source, selection, profile-entry, registry-root, toolchain,
bundle and artifact hashes across the entire graph.

Error ordering is release/context validation, capture/transport, parse,
attribution, subset/closure/contracts, lowering, then emission. Parse errors
stop before attribution. Compilation diagnostics with error, warning or
mandatory-warning kind reject as `source-error`; allowed note codes must be
explicitly enumerated in the frozen adapter vectors. Unknown diagnostic
codes/kinds, unavailable source provenance, diagnostic truncation and API
behavior outside those vectors are frontend errors. The listener retains no
unbounded compiler prose. See
[`Diagnostic`](https://docs.oracle.com/en/java/javase/25/docs/api/java.compiler/javax/tools/Diagnostic.html).

| Failure class | Result / owner |
| --- | --- |
| Malformed caller selection or crossed profile | Pre-launch configuration error, exit 2; no child output |
| Missing/tampered registry, JDK, launcher or native input | Release-phase `frontend-error`; do not start compiler |
| Invalid source encoding, syntax or attributed source | `source-error`, Java source diagnostic family |
| Unsupported declaration/type/operator/initialization/call/contract | `rejected`, exact Java subset/contract family |
| Unknown compiler API state, invalid map, exhausted diagnostic/output budget, timeout or killed child | `frontend-error`; no partial success artifacts |

T01 froze a closed `JAVA_*` diagnostic registry with phase, status, stable
message and exit mappings under the shared frontend protocol. The exact
compiler code/kind allowlist, including the six observed ERROR codes, is in
`diagnostic_normalization`; unknown codes or NOPOS provenance fail closed.
Normalize issues to logical path, byte span, severity and code; sort using those fields
and a frozen tie rule. Messages contain no source snippet, compiler prose,
absolute path, environment content or stack trace. Normalize resource/signal
failures at the parent when the child cannot emit valid bounded JSON. A
failure cannot publish VIR/maps/manifests/VCs/evidence from a partial result.

## 9. Resource limits

Adopt the following inclusive logical limits; a lower applicable shared
successor limit still wins. Counters use checked arithmetic and reject before
retaining the first excess item. These bounds constrain parsing, attribution,
and emission as well as the final output.

| Counter | Maximum |
| --- | ---: |
| Source files / bytes per file / total bytes | 256 / 1,048,576 / 16,777,216 |
| Sidecars / bytes per sidecar / total bytes | 128 / 1,048,576 / 8,388,608 |
| Snapshot entries / total file bytes | 512 / 33,554,432 |
| Path / method ID bytes | 1,024 / 1,024 |
| Selected methods / closure methods | 32 / 128 |
| Descriptor parameter units per static method | 255 |
| Public syntax nodes per compilation / nesting depth | 250,000 / 256 |
| Emitted instructions per method / closure | 100,000 / 250,000 |
| CFG blocks per method / closure | 1,024 / 8,192 |
| Contract clauses per method | 64 |
| Contract expression nodes per method / closure / depth | 1,024 / 8,192 / 32 |
| Diagnostics, including notes, before public sorting | 1,024 |
| Canonical message bytes per issue / total | 4,096 / 2,097,152 |
| Expanded argv bytes, including each NUL | 131,072 |
| Frontend stdout including LF / stderr | 268,435,456 / 2,097,152 |
| VIR / source-map / manifest canonical bytes | 201,326,592 / 33,554,432 / 4,194,304 |

Source and sidecar byte counts use raw captured bytes. Snapshot entries count
files plus distinct nonempty parent directories, excluding the root. Syntax
nodes count each source public tree once before attribution; the post-analysis
inventory is separately bounded by the same count/depth. Transformation
comparison follows raw source rejection gates and applies only to candidate
accepted subtrees, as specified in section 5.3. Counting synthesized children
of an excluded parent does not reinterpret them as accepted source or replace
that parent's subset rejection with an adapter error. Instructions include masks, casts, copies
and branch temporaries; contract roots have depth one. Check depth during
iterative traversal, before recursive lowering. The isolated compiler may
hit its hard memory/time bound before a logical count is available; report
that as a frontend resource failure, not a supported-source rejection.

Configure javac's error/warning ceilings above the public issue budget (1,025
each), and enforce the 1,024 all-kind limit in the listener so silent compiler
truncation cannot produce success. T01 vectors record the exact
listener/abort behavior for the pinned build: the 1,025th callback aborts,
1,024 issues are retained, and no output is generated. JVM heap, native
memory, process count, timeout and tmpfs bounds belong to the measured host profile in section
7, and must be numerical and frozen before production work.

## 10. Registry admission and all consumers

Revision 3 uses the same registry schema only if the existing common
representation remains unchanged. Preserve every revision-2 Go/Rust/C# entry
byte-for-byte, insert exactly one Java entry, sort by the normative tuple key,
and freeze the new root and predecessor diff. The runtime accepts only its
embedded revision/root; a valid revision-2 artifact is not a revision-3 input.
See `../specs/SEMANTIC_PROFILE_REGISTRY_V1.md`, section 11.

Java owns all nine IDs `mpk.profile.<field>.java_scalar.v0`:

| Field | Required payload responsibility |
| --- | --- |
| `vir` | Accepted types, operators, internal shift pattern and shared VIR limits |
| `vc` | Sidecar normalization, Java required checks, existing verification limits |
| `source_map` | Source-origin coverage, UTF-16/UTF-8 mapping, no synthetic origins |
| `manifest` | Compilation unit, `.java` sources, source/contract inventory |
| `frontend` | Fixed arguments, environment, limits, JVM launcher; no private driver |
| `release` | Pinned JDK/build-input hash, measured host/layout and native closure |
| `policy` | `payment-policy-java-alpha`, `mvp-strict`, `mvp-theory` |
| `evidence` | `mpk.java.evidence_recipe.v0`, both checkers, certificate-only authority |
| `ai` | `mpk.java.ai_projection.v0`, label `Java`, `minimal-v1`, no source access or proof authority |

Profiles select finite compiled validators; a registry is not executable
configuration. T01 froze exact payload fields, envelope sizes, applicable hashes and
mutations in the normative vectors. A recognized language with any missing contract is an
invalid release, not partial Java support.

The existing CLI selection route gains a Java branch that builds the validated
selection envelope; contracts come from that envelope, as for C#. No separate
Java-only public verifier, raw compiler path, `--java-home`, classpath option,
registry override or schema-compatibility flag is added. The same policy scan,
verify, evidence and explain routes operate after installed tuple validation.
API sessions bind the full semantic context and reject crossed root/profile/
selection artifacts before state mutation. Java sidecars/source/compiler
diagnostics and host metadata never enter provider requests.

Activation replaces the installed release atomically: preserve the current
four Go/Rust/C# language/profile/target combinations, add one Linux-x64 Java
tuple, and rebuild the descriptors and all producer
and consumer context bindings for revision 3. Existing profile entry bytes,
value semantics and checker behavior stay unchanged; artifacts that embed the
registry root necessarily receive new hashes. Do not promise all old helper
bytes remain identical. Prove semantic regression separately from expected
context/hash changes.

Do not dual-load revision 2 and 3, translate old helper artifacts on import,
or expose a staging release through the public CLI. Rollback replaces the
entire installed image with the previous release and its registries; it does
not mix binaries, registries or bundle generations. Certificate v0 remains
source-free and does not require a registry compatibility adapter.

## 11. Serial implementation plan

These tasks refine `JAVA-03`. T01-T06 are complete; T07 is next and T07-T10
remain pending. Each task depends on the previous row, starting from
completed `CSHARP-02-T20`. Private artifact generation does not establish
registered installed execution or the complete native Linux release gate.

| Task | Deliverable | Exit condition |
| --- | --- | --- |
| `JAVA-03-T01` | `JAVA_PROFILE_V0.md`, `java-profile-v0.json`, `semantic-profile-registry-v3.json`, manifest entries and implementation traceability ledger | All 34 matrix rows classified; exact profile/payloads/hashes/diagnostics/limits and JDK/host closure frozen; disposable public-API/Linux probe validates the design; specification/vector review has zero findings |
| `JAVA-03-T02` | Inactive `java-tools/java2vir` build and `build-java-frontend.sh` | Offline deterministic build; complete JDK/frontend inventories; no active tuple or public Java route |
| `JAVA-03-T03` | Inactive compiled registry/contracts and Java source-artifact validators | All profile/selection/payload/hash mutations reject; revision-2 entry preservation proved; existing release remains active |
| `JAVA-03-T04` | Capture, public javac adapter and bounded diagnostics | Source/processor/module closure, UTF-8 transport, attribution and exact diagnostic precedence pass |
| `JAVA-03-T05` | Source subset, inert initialization, call closure and sidecars | Exhaustive positive/negative forms; sidecar coverage and acyclicity; no ignored selected content |
| `JAVA-03-T06` | CFG, lowering, source maps and manifests | Exact operations/checks, short circuit, MIN/-1, shifts/conversions, deterministic origins and emission |
| `JAVA-03-T07` | Registered candidate bundles and measured JVM runner | Installed candidate image launches only registered bytes; offline, native closure, cgroup and hostile-environment tests pass |
| `JAVA-03-T08` | VC, policy/evidence, certificate, AI and API integration | End-to-end Java certificate accepted from identical bytes by both checkers; context/redaction/regression gates pass without public activation |
| `JAVA-03-T09` | Conformance/differential/fuzz/upgrade and release rehearsal | Two-build/two-run determinism, all vector owners execute, zero open findings and unchanged axiom categories |
| `JAVA-03-T10` | Atomic four-language release, examples and active docs | Only revision 3 is installed, all five tuples work, old/crossed contexts reject, final local Linux release gate passes; then `DART-04` is eligible |

The T01 ledger assigns each normative section, payload field, vector family
and concrete test file exactly one primary implementation owner. Staging is a
private test/build boundary; do not add runtime environment toggles or
alternate public input routes. The final task removes executable staging
entrypoints and retains review evidence only as archived documentation.

T01 repaired assumptions falsified by its probes before freezing, including
the platform-file-manager boundary and the unchecked descriptor-unit limit.
An implementation finding must follow the profile's reviewed upgrade rules;
it cannot guess archive hashes, relax isolation, or accept unknown compiler
behavior. No Dart or later-language phase runs concurrently.

## 12. Validation and acceptance criteria

### 12.1 Required test families

`java_profile_spec.rs` owns the T01 specification/hash model. The build,
profile, capture, source/contract admission and lowering/emission owners through T06 are
implemented. Later owners and downstream contributions remain assigned to
their ledger tasks. The manifest records the currently available tests.

| Owner | Required coverage |
| --- | --- |
| `crates/mpk-vc/tests/java_profile_spec.rs` | Strict specification vectors, canonical hashes, all nine contracts, exact predecessor entries |
| `crates/mpk-cli/tests/java_build_inputs.rs` | Archive/path/link/native inventory mutations, version/option mismatches, offline reconstruction |
| `crates/mpk-cli/tests/java_frontend_vectors.rs` | Capture, parse/analyze, tree inventories, diagnostics and protocol; downstream precedence contributions |
| `crates/mpk-vc/tests/java_profile_vectors.rs` | All typed VIR/VC mappings, missing/extra checks, signed/unsigned internal-pattern mutations |
| `crates/mpk-cli/tests/java_subset.rs` | Source forms, inert initialization, typed bindings/conversions, complete acyclic call closure and subset limits |
| `crates/mpk-cli/tests/java_contracts.rs` | Strict sidecar JSON, complete attachment, type/operator rules, ordered normalization, canonical hashes and contract limits |
| `crates/mpk-cli/tests/java_lowering.rs` | All accepted cases/operations/CFGs, exact checks and evaluation order, deterministic complete artifacts and lowering/emission limits |
| `crates/mpk-cli/tests/java_source_maps.rs` | Original UTF-8 origins, complete coverage, source/sidecar/manifest binding and artifact-free emission failures |
| `crates/mpk-cli/tests/java_policy_verify.rs` | Contract/call obligations, evidence regeneration, same-byte dual checking, AI/API rejection and redaction |
| `crates/mpk-cli/tests/java_release_gate.rs` | Candidate rehearsal and active four-language installation, isolation, cross-generation rejection, determinism and upgrade corpus |

The positive/differential corpus covers both widths at zero, one, minus one,
MIN and MAX; wrapping arithmetic; signed division/remainder with all signs;
MIN/-1; negative and boundary shift counts including 31/32/63/64; `>>>` of a
negative number; widening/narrowing; branch joins; same-type ternaries;
short-circuited zero-divisor/call branches; and multiple selected entrypoints.
Compare canonical VIR evaluation with separately compiled execution using the
exact pinned JDK in disposable offline test sandboxes. Production never
executes selected source. Include zero-divisor runtime exceptions as negative
normal-completion cases rather than comparing them to total VIR values.

Negative tests cover every excluded matrix row and source form, hidden static
effects, source-dead unsupported code, recursion, missing/unused contracts,
mixed types, compiler-recovered nodes, unknown diagnostics, Unicode escapes,
tab/non-BMP map boundaries, malicious paths, duplicate keys, huge/deep inputs,
unlisted classes/processors, ambient JVM options and malicious JAR manifests.
For every numeric limit, test the exact boundary and boundary-plus-one.

Fuzz the source decoder/parser adapter, contract parser, compiler-diagnostic
normalizer, frontend protocol and resource/capture boundary with bounded,
hash-recorded seeds. Two independent clean builds and repeated runs must
yield byte-identical frontend bundles, VIR, maps, manifests, VCs and helper
reports for fixed inputs. Compare Go/Rust/C# verdicts and profile-owned
semantic payloads, while checking expected revision-root hash changes.

### 12.2 Local-only gates and completion

Use `./scripts/check-fast.sh` for ordinary repository checks. Until Java
activation, the existing release owner remains
`sudo ./scripts/check-csharp-frontend.sh` and the README's local Linux gate.
The future `scripts/check-java-frontend.sh` must become the single composed
Go/Rust/C#/Java offline two-pass release gate at T10, with existing checks
preserved and documentation updated together. `check-all.sh` must route
through that owner without skipping a predecessor language.

Release checks run only on a reviewed Linux host with the required cgroup,
kernel/native closure, privilege drop, externally denied egress and frozen
pre-provisioned inputs. macOS development checks cannot establish JVM Linux
isolation or release acceptance. Do not create, update, run, monitor or depend
on GitHub Actions; no `.github/workflows/`, `gh run`, `gh workflow`, or
`workflow_dispatch` belongs in this work.

Java implementation is complete only when the frozen package, every owning
test, four-language installed release, deterministic corpus, same-byte
dual-checker certificate corpus, unchanged axiom inventory, upgrade procedure
and final zero-finding review all pass. A documentation-only change does not
satisfy that implementation gate.

### 12.3 Compiler and profile upgrades

An upgrade starts with a proposed exact replacement archive and a reviewed
diff of JDK/reference/native inputs, public tree/type/diagnostic observations,
compiler/JVM options and host requirements. Re-run specification vectors,
positive/negative/differential cases, deterministic builds, isolation and the
complete four-language installed release gate before adoption. Keep the prior
image available for whole-image rollback. Never update a runtime in place.

Any changed accepted values, operation/check semantics, adapter acceptance
rules or compiled contract meaning require new immutable profile/contract IDs
and reviewed registry admission. Common shape or hash-meaning changes require
the affected successor schemas. A byte-only toolchain replacement with
unchanged profile meaning still requires new pinned bundle/input hashes and
the full upgrade evidence; a patch-version label alone proves neither
compatibility nor safety.
